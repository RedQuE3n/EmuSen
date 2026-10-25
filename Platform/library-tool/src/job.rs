//! A request run as a job on a thread of its own, its events queued as JSON lines for whoever reads them: the shell's
//! module through the `emusen.library` service and Mistress's window through the platform library, which never calls
//! its host (EmuSen_RustPlatform.md §3). The request and every event are listed in EmuSen_GamePackages.md §12.3.

use crate::chdman::Chdman;
use crate::convert::{self, obj, Options, Outcome};
use crate::discs::{self, Item};
use crate::Context;
use emusen_galaxia::dotnet_path::{self as path, Style};
use emusen_galaxia::json::{self, Value};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

/// What a request asks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Op {
    Convert,
    Check,
    List,
}

#[derive(Clone, Debug)]
pub struct Request {
    pub op: Op,
    pub paths: Vec<String>,
    pub options: Options,
}

/// A request's JSON: `{"op": "convert" | "check" | "ls", "paths": [...], "fast": false, "threads": 0}`.
pub fn request(text: &str) -> Result<Request, String> {
    let doc = json::parse(text).map_err(|e| format!("the request is not JSON: {e}"))?;
    let op = match doc.get("op").and_then(Value::as_str) {
        Some("convert") => Op::Convert,
        Some("check") => Op::Check,
        Some("ls") => Op::List,
        _ => return Err("the request's op is not convert, check or ls".into()),
    };
    let paths = match doc.get("paths") {
        None | Some(Value::Null) => Vec::new(),
        Some(Value::Array(items)) => items.iter().map(|p| p.as_str().map(str::to_string).ok_or("each of the request's paths is a string")).collect::<Result<_, _>>()?,
        Some(_) => return Err("the request's paths are not a list".into()),
    };
    let threads = match doc.get("threads") {
        Some(Value::Number(t)) => t.parse::<usize>().map_err(|_| "the request's threads are not a whole number")?,
        _ => 0,
    };
    Ok(Request { op, paths, options: Options { fast: doc.get("fast").and_then(Value::as_bool).unwrap_or(false), threads } })
}

struct Queue {
    events: Mutex<(VecDeque<String>, bool)>,
    arrived: Condvar,
}

/// A running or finished request.
pub struct Job {
    queue: Arc<Queue>,
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Job {
    /// The next event, waiting up to `wait` for one; `None` when none came, and none ever after the last.
    pub fn peek(&self, wait: Duration) -> Option<String> {
        let mut guard = self.queue.events.lock().unwrap_or_else(|p| p.into_inner());
        if guard.0.is_empty() && !guard.1 {
            guard = self.queue.arrived.wait_timeout(guard, wait).unwrap_or_else(|p| p.into_inner()).0;
        }
        guard.0.front().cloned()
    }

    /// Takes the event `peek` gave.
    pub fn take(&self) {
        self.queue.events.lock().unwrap_or_else(|p| p.into_inner()).0.pop_front();
    }

    /// Whether the job has ended and every event was taken.
    pub fn finished(&self) -> bool {
        let guard = self.queue.events.lock().unwrap_or_else(|p| p.into_inner());
        guard.1 && guard.0.is_empty()
    }

    /// Asks the job to stop: a running chdman is ended, its partial file removed, and the job ends with what it did.
    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

impl Drop for Job {
    fn drop(&mut self) {
        self.stop();
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

fn emit(queue: &Queue, event: Value) {
    let mut line = String::new();
    json::write_compact(&mut line, &event);
    let mut guard = queue.events.lock().unwrap_or_else(|p| p.into_inner());
    guard.0.push_back(line);
    queue.arrived.notify_all();
}

/// Starts a request; a request that is not one is refused before anything runs.
pub fn start(text: &str, ctx: Context) -> Result<Job, String> {
    let req = request(text)?;
    let queue = Arc::new(Queue { events: Mutex::new((VecDeque::new(), false)), arrived: Condvar::new() });
    let stop = Arc::new(AtomicBool::new(false));
    let (q, s) = (queue.clone(), stop.clone());
    let thread = std::thread::Builder::new()
        .name("emusen-library-tool".into())
        .spawn(move || {
            run(&ctx, &req, &q, &|| s.load(Ordering::Relaxed));
            let mut guard = q.events.lock().unwrap_or_else(|p| p.into_inner());
            guard.1 = true;
            q.arrived.notify_all();
        })
        .map_err(|e| e.to_string())?;
    Ok(Job { queue, stop, thread: Some(thread) })
}

fn op_word(op: &Op) -> &'static str {
    match op {
        Op::Convert => "convert",
        Op::Check => "check",
        Op::List => "ls",
    }
}

fn end(queue: &Queue, op: &Op, counts: &[(&'static str, u64)], started: Instant, error: Option<String>) {
    let counts = Value::Object(counts.iter().map(|(k, v)| (json::Text::new(*k), Value::number(*v))).collect());
    let mut members = vec![("event", Value::string("end")), ("op", Value::string(op_word(op))), ("counts", counts), ("seconds", Value::number(format!("{:.1}", started.elapsed().as_secs_f64())))];
    if let Some(words) = error {
        members.push(("error", Value::string(&words)));
    }
    emit(queue, obj(members));
}

fn count(counts: &mut Vec<(&'static str, u64)>, state: &'static str) {
    match counts.iter_mut().find(|(k, _)| *k == state) {
        Some(c) => c.1 += 1,
        None => counts.push((state, 1)),
    }
}

fn run(ctx: &Context, req: &Request, queue: &Queue, stop: &dyn Fn() -> bool) {
    let started = Instant::now();
    let mut counts: Vec<(&'static str, u64)> = Vec::new();
    let library = match ctx.checked_library() {
        Ok(l) => l,
        Err(words) => return end(queue, &req.op, &counts, started, Some(words)),
    };
    let chdman = match req.op {
        Op::List => None,
        _ => match Chdman::open(&ctx.chdman, ctx.pin.as_deref(), &ctx.release) {
            Ok(c) => Some(c),
            Err(words) => return end(queue, &req.op, &counts, started, Some(words)),
        },
    };
    let (items, refused) = discs::items(&library, &req.paths);
    let threads = if req.options.threads > 0 { req.options.threads } else { std::thread::available_parallelism().map_or(1, |n| n.get()) };
    let mut start = vec![("event", Value::string("start")), ("op", Value::string(op_word(&req.op))), ("discs", Value::number(items.len()))];
    if let Some(c) = &chdman {
        start.push(("chdman", Value::string(&c.version)));
    }
    if req.op == Op::Convert {
        start.push(("codecs", Value::string(if req.options.fast { convert::FAST_CODECS } else { "cdlz,cdzl,cdfl" })));
        start.push(("threads", Value::number(threads)));
        start.push(("output", Value::string(&ctx.compressed())));
    }
    start.push(("refused", Value::Array(refused.iter().map(Value::string).collect())));
    emit(queue, obj(start));
    if req.op == Op::Convert {
        convert::sweep(ctx);
    }
    let options = Options { threads, ..req.options };
    for (i, item) in items.iter().enumerate() {
        if stop() {
            count(&mut counts, "stopped");
            continue;
        }
        let head = |state: &str| {
            let mut h = vec![("event", Value::string("disc"))];
            if req.op != Op::List {
                h.extend([("index", Value::number(i + 1)), ("of", Value::number(items.len()))]);
            }
            h.extend([("source", Value::string(&item.rel)), ("system", Value::string(&item.system.console)), ("state", Value::string(state))]);
            h
        };
        if req.op != Op::List {
            emit(queue, obj(head("working")));
        }
        let begun = Instant::now();
        let outcome: Outcome = match (&req.op, &chdman) {
            (Op::Convert, Some(c)) => convert::convert(ctx, c, item, i, options, stop),
            (Op::Check, Some(c)) => convert::check(ctx, c, item, stop),
            _ => convert::list(ctx, item),
        };
        count(&mut counts, outcome.state);
        let mut event = head(outcome.state);
        if let Some(words) = &outcome.words {
            event.push(("words", Value::string(words)));
        }
        let mut members: Vec<(json::Text, Value)> = event.into_iter().map(|(k, v)| (json::Text::new(k), v)).collect();
        members.extend(outcome.facts);
        let event = Value::Object(members);
        if req.op != Op::List {
            log(ctx, &req.op, item, &event, begun.elapsed());
        }
        emit(queue, event);
    }
    end(queue, &req.op, &counts, started, None);
}

/// One line per disc in `Logs/chd_<day>.log` (§4.6): when, what, the disc and its event.
fn log(ctx: &Context, op: &Op, item: &Item, event: &Value, took: Duration) {
    let now = std::time::SystemTime::now();
    let stamp = convert::utc(now);
    let day = &stamp[..10];
    let at = path::combine(Style::HOST, &ctx.logs(), &format!("chd_{day}.log"));
    let mut line = String::new();
    json::write_compact(&mut line, event);
    let text = format!("{stamp}\t{}\t{}\t{:.1}s\t{line}\n", op_word(op), item.rel, took.as_secs_f64());
    let _ = std::fs::create_dir_all(ctx.logs());
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&at) {
        let _ = f.write_all(text.as_bytes());
    }
}
