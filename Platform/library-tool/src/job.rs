//! A request run as a job on a thread of its own, its events queued as JSON lines for whoever reads them: the shell's
//! module through the `emusen.library` service and Mistress's window through the platform library, which never calls
//! its host (EmuSen_RustPlatform.md §3). The request and every event are listed in EmuSen_GamePackages.md §12.3.

use crate::chdman::Chdman;
use crate::convert::{self, obj, Options, Outcome};
use crate::discs::{self, Item};
use crate::{packages, Context};
use emusen_galaxia::claims;
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
    /// `pkg make`, `pkg ls`, `pkg check` and `pkg migrate` (EmuSen_GamePackages.md §3.7, §14).
    PkgMake,
    PkgList,
    PkgCheck,
    PkgMigrate,
    /// `pkg patch add` and `pkg patch rm` (EmuSen_GamePackages.md §15), since 1.2.
    PkgPatchAdd,
    PkgPatchRm,
}

impl Op {
    fn packages(&self) -> bool {
        matches!(self, Op::PkgMake | Op::PkgList | Op::PkgCheck | Op::PkgMigrate | Op::PkgPatchAdd | Op::PkgPatchRm)
    }
}

#[derive(Clone, Debug)]
pub struct Request {
    pub op: Op,
    pub paths: Vec<String>,
    pub options: Options,
    /// `pkg ls`'s console, none for every package.
    pub console: Option<String>,
    /// `pkg patch add`'s patch file, a full path; `pkg patch rm`'s patch, from 1.
    pub patch: Option<String>,
    pub index: usize,
}

/// A request's JSON: `{"op": "convert" | "check" | "ls", "paths": [...], "fast": false, "threads": 0}`, or since 1.1
/// `{"op": "pkg-make" | "pkg-ls" | "pkg-check" | "pkg-migrate", "paths": [...], "console": null}`, or since 1.2
/// `{"op": "pkg-patch-add", "paths": [<package>], "patch": <file>}` and `{"op": "pkg-patch-rm", "paths": [<package>], "index": n}`.
pub fn request(text: &str) -> Result<Request, String> {
    let doc = json::parse(text).map_err(|e| format!("the request is not JSON: {e}"))?;
    let op = match doc.get("op").and_then(Value::as_str) {
        Some("convert") => Op::Convert,
        Some("check") => Op::Check,
        Some("ls") => Op::List,
        Some("pkg-make") => Op::PkgMake,
        Some("pkg-ls") => Op::PkgList,
        Some("pkg-check") => Op::PkgCheck,
        Some("pkg-migrate") => Op::PkgMigrate,
        Some("pkg-patch-add") => Op::PkgPatchAdd,
        Some("pkg-patch-rm") => Op::PkgPatchRm,
        _ => return Err("the request's op is not convert, check, ls, pkg-make, pkg-ls, pkg-check, pkg-migrate, pkg-patch-add or pkg-patch-rm".into()),
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
    let console = doc.get("console").and_then(Value::as_str).map(str::to_string);
    let patch = doc.get("patch").and_then(Value::as_str).map(str::to_string);
    let index = match doc.get("index") {
        Some(Value::Number(t)) => t.parse::<usize>().map_err(|_| "the request's index is not a whole number")?,
        _ => 0,
    };
    if op == Op::PkgPatchAdd && patch.as_deref().is_none_or(str::is_empty) {
        return Err("pkg-patch-add names its patch file in \"patch\"".into());
    }
    if matches!(op, Op::PkgPatchAdd | Op::PkgPatchRm) && paths.len() != 1 {
        return Err("a patch is added to or removed from one package, named in \"paths\"".into());
    }
    Ok(Request { op, paths, options: Options { fast: doc.get("fast").and_then(Value::as_bool).unwrap_or(false), threads }, console, patch, index })
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
        Op::PkgMake => "pkg-make",
        Op::PkgList => "pkg-ls",
        Op::PkgCheck => "pkg-check",
        Op::PkgMigrate => "pkg-migrate",
        Op::PkgPatchAdd => "pkg-patch-add",
        Op::PkgPatchRm => "pkg-patch-rm",
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
        Err(words) if req.op.packages() => return end(queue, &req.op, &counts, started, Some(words.replace("there is no library to convert.", "there is no library for a package to link to."))),
        Err(words) => return end(queue, &req.op, &counts, started, Some(words)),
    };
    if req.op.packages() {
        return run_packages(ctx, &library, req, queue, stop, started);
    }
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
    let log_at = log_path(ctx, std::time::SystemTime::now());
    if req.op != Op::List {
        start.push(("log", Value::string(&log_at)));
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
            log(&log_at, &req.op, item, &event, begun.elapsed());
        }
        emit(queue, event);
    }
    end(queue, &req.op, &counts, started, None);
}

/// A `pkg` request: one `package` event for each game made or package listed, checked or migrated, between the start
/// and the end (EmuSen_GamePackages.md §14). `source` is the library path a game was made from, `package` the package's
/// folder under `Packages`.
fn run_packages(ctx: &Context, library: &str, req: &Request, queue: &Queue, stop: &dyn Fn() -> bool, started: Instant) {
    let mut counts: Vec<(&'static str, u64)> = Vec::new();
    let (games, folders, refused) = match req.op {
        Op::PkgMake => {
            let (g, r) = packages::games(library, &req.paths);
            (g, Vec::new(), r)
        }
        Op::PkgList if req.console.is_some() => (Vec::new(), packages::of_console(ctx, req.console.as_deref().unwrap_or("")), Vec::new()),
        Op::PkgPatchAdd | Op::PkgPatchRm => {
            let (f, mut r) = packages::named(ctx, &req.paths);
            if f.len() > 1 {
                r.push(format!("{}: names {} packages, and a patch belongs to one", req.paths[0], f.len()));
                (Vec::new(), Vec::new(), r)
            } else {
                (Vec::new(), f, r)
            }
        }
        _ => {
            let (f, r) = packages::named(ctx, &req.paths);
            (Vec::new(), f, r)
        }
    };
    let total = games.len() + folders.len();
    let log_at = path::combine(Style::HOST, &ctx.logs(), &format!("pkg_{}.log", &convert::utc(std::time::SystemTime::now())[..10]));
    let mut start = vec![("event", Value::string("start")), ("op", Value::string(op_word(&req.op))), ("packages", Value::number(total))];
    if req.op != Op::PkgList {
        start.push(("log", Value::string(&log_at)));
    }
    start.push(("refused", Value::Array(refused.iter().map(Value::string).collect())));
    emit(queue, obj(start));
    let mut index = if req.op == Op::PkgMake { claims::index(&ctx.places().roots()) } else { claims::Index::default() };
    let jobs: Vec<(Option<&packages::Game>, Option<&String>)> = games.iter().map(|g| (Some(g), None)).chain(folders.iter().map(|f| (None, Some(f)))).collect();
    for (i, (game, folder)) in jobs.into_iter().enumerate() {
        if stop() {
            count(&mut counts, "stopped");
            continue;
        }
        let begun = Instant::now();
        let outcome = match (&req.op, game, folder) {
            (Op::PkgMake, Some(g), _) => packages::make(ctx, &mut index, g, stop),
            (Op::PkgList, _, Some(f)) => packages::list(ctx, f),
            (Op::PkgCheck, _, Some(f)) => packages::check(ctx, f, stop),
            (Op::PkgPatchAdd, _, Some(f)) => crate::patches::add(ctx, f, req.patch.as_deref().unwrap_or("")),
            (Op::PkgPatchRm, _, Some(f)) => crate::patches::remove(ctx, f, req.index),
            (_, _, Some(f)) => packages::migrate(ctx, f),
            _ => continue,
        };
        count(&mut counts, outcome.state);
        let mut members = vec![("event", Value::string("package")), ("index", Value::number(i + 1)), ("of", Value::number(total))];
        match (game, folder) {
            (Some(g), _) => members.extend([("source", Value::string(&g.rel)), ("system", Value::string(&g.system.console))]),
            (_, Some(f)) => members.push(("package", Value::string(packages::package_rel(ctx, f)))),
            _ => {}
        }
        members.push(("state", Value::string(outcome.state)));
        if let Some(words) = &outcome.words {
            members.push(("words", Value::string(words)));
        }
        let mut event: Vec<(json::Text, Value)> = members.into_iter().map(|(k, v)| (json::Text::new(k), v)).collect();
        for (k, v) in outcome.facts {
            match event.iter_mut().find(|(e, _)| e.text == k.text) {
                Some(slot) => slot.1 = v,
                None => event.push((k, v)),
            }
        }
        let event = Value::Object(event);
        if req.op != Op::PkgList {
            let mut line = String::new();
            json::write_compact(&mut line, &event);
            let text = format!("{}	{}	{:.1}s	{line}
", convert::utc(std::time::SystemTime::now()), op_word(&req.op), begun.elapsed().as_secs_f64());
            append(&log_at, &text);
        }
        emit(queue, event);
    }
    end(queue, &req.op, &counts, started, None);
}

fn append(at: &str, text: &str) {
    if let Some(dir) = std::path::Path::new(at).parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(at) {
        let _ = f.write_all(text.as_bytes());
    }
}

/// The batch's log, `Logs/chd_<day>.log` by the day it started, so that one batch keeps to one file (§4.6).
fn log_path(ctx: &Context, started: std::time::SystemTime) -> String {
    let stamp = convert::utc(started);
    path::combine(Style::HOST, &ctx.logs(), &format!("chd_{}.log", &stamp[..10]))
}

/// One line per disc in the batch's log: when, what, the disc and its event.
fn log(at: &str, op: &Op, item: &Item, event: &Value, took: Duration) {
    let stamp = convert::utc(std::time::SystemTime::now());
    let mut line = String::new();
    json::write_compact(&mut line, event);
    let text = format!("{stamp}\t{}\t{}\t{:.1}s\t{line}\n", op_word(op), item.rel, took.as_secs_f64());
    append(at, &text);
}
