//! A core request run as a job, its events JSON lines (EmuSen_Cores.md §6): `core-fetch`, `core-ls` and `core-info`
//! since 2026-10-31; the request and every event are listed in §6.1 there.

use crate::catalogue::{self, Row, State};
use crate::{Context, Placement};
use emusen_galaxia::json::{self, Text, Value};
use emusen_tool_job::Events as Queue;
pub use emusen_tool_job::Job;
use std::sync::Arc;

/// What a request asks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Fetch,
    List,
    Info,
}

/// Which rows `core-ls` gives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Show {
    All,
    Installed,
    Available,
    Updates,
    Unsupported,
}

#[derive(Clone)]
pub struct Request {
    pub op: Op,
    pub show: Show,
    pub console: Option<String>,
    pub core: Option<String>,
    /// The request's own placement, from its `"systems"` object, in place of the context's.
    pub placement: Option<Placement>,
}

/// Whether a request's op is the core tool's, so the platform library routes it here.
pub fn is_core_op(op: &str) -> bool {
    op.starts_with("core-")
}

fn placement_of(map: &[(Text, Value)]) -> Result<Placement, String> {
    let mut pairs: Vec<(String, Vec<String>)> = Vec::new();
    for (name, ids) in map {
        let ids = match ids {
            Value::String(s) => vec![s.text.clone()],
            Value::Array(a) => a.iter().map(|v| v.as_str().map(str::to_string).ok_or("a system id is not text")).collect::<Result<_, _>>()?,
            _ => return Err(format!("systems' {} is not an id or a list of ids", name.text.as_str())),
        };
        pairs.push((name.text.as_str().to_string(), ids));
    }
    Ok(Arc::new(move |systemid: &str, databases: &[&str]| {
        let mut out: Vec<String> = Vec::new();
        let names: Vec<&str> = if databases.is_empty() { vec![systemid] } else { databases.to_vec() };
        for name in names {
            for (k, ids) in &pairs {
                if k == name {
                    out.extend(ids.iter().filter(|i| !out.contains(i)).cloned().collect::<Vec<_>>());
                }
            }
        }
        out
    }))
}

/// A request's JSON: `{"op": "core-fetch"}`, `{"op": "core-ls", "show": "all" | "installed" | "available" | "updates" |
/// "unsupported", "console": null | <system id>}` or `{"op": "core-info", "core": <stem>}`, each with an optional
/// `"systems": {<libretro database or systemid>: <system id> | [<system id>...]}`.
pub fn request(text: &str) -> Result<Request, String> {
    let doc = json::parse(text).map_err(|e| format!("the request is not JSON: {e}"))?;
    let op = match doc.get("op").and_then(Value::as_str) {
        Some("core-fetch") => Op::Fetch,
        Some("core-ls") => Op::List,
        Some("core-info") => Op::Info,
        Some(other) => return Err(format!("{other} is not an operation of the core tool")),
        None => return Err("the request has no op".into()),
    };
    let show = match doc.get("show").and_then(Value::as_str).unwrap_or("all") {
        "all" => Show::All,
        "installed" => Show::Installed,
        "available" => Show::Available,
        "updates" => Show::Updates,
        "unsupported" => Show::Unsupported,
        other => return Err(format!("show {other} is not all, installed, available, updates or unsupported")),
    };
    let console = doc.get("console").and_then(Value::as_str).map(str::to_string);
    let core = doc.get("core").and_then(Value::as_str).map(str::to_string);
    if op == Op::Info && core.is_none() {
        return Err("core-info names no core".into());
    }
    let placement = match doc.get("systems") {
        None | Some(Value::Null) => None,
        Some(Value::Object(map)) => Some(placement_of(map)?),
        Some(_) => return Err("systems is not an object".into()),
    };
    Ok(Request { op, show, console, core, placement })
}

/// Starts a request as a job.
pub fn start(text: &str, mut ctx: Context) -> Result<Job, String> {
    let req = request(text)?;
    if let Some(p) = req.placement.clone() {
        ctx.placement = Some(p);
    }
    Job::spawn("emusen-core-tool", move |q, stop| run(&ctx, &req, q, stop))
}

fn obj(members: Vec<(&str, Value)>) -> Value {
    Value::Object(members.into_iter().map(|(k, v)| (Text::new(k), v)).collect())
}

fn emit(queue: &Queue, event: Value) {
    let mut line = String::new();
    json::write_compact(&mut line, &event);
    queue.emit(line);
}

fn strings(list: &[String]) -> Value {
    Value::Array(list.iter().map(Value::string).collect())
}

fn op_word(op: Op) -> &'static str {
    match op {
        Op::Fetch => "core-fetch",
        Op::List => "core-ls",
        Op::Info => "core-info",
    }
}

fn shown(row: &Row, show: Show) -> bool {
    match show {
        Show::All => true,
        Show::Installed => matches!(row.state, State::Installed | State::Update | State::Added),
        Show::Available => row.state == State::Available,
        Show::Updates => row.state == State::Update,
        Show::Unsupported => row.state == State::Unsupported,
    }
}

/// A row's listing line (EmuSen_Cores.md §6.1).
pub fn row_event(row: &Row) -> Value {
    let info = row.info.as_ref();
    let mut m = vec![
        ("event", Value::string("core")),
        ("stem", Value::string(&row.stem)),
        ("name", Value::string(row.name())),
        ("corename", info.and_then(|i| i.get("corename")).or(row.notice.as_ref().map(|n| n.corename.as_str())).map_or(Value::Null, Value::string)),
        ("systems", strings(&row.systems)),
        ("source", Value::string("libretro")),
        ("version", info.and_then(|i| i.get("display_version")).map_or(Value::Null, Value::string)),
        ("build", row.notice.as_ref().map(|n| n.build.clone()).or(row.index.as_ref().map(|l| l.date.clone())).map_or(Value::Null, Value::string)),
        ("offered", row.index.as_ref().map_or(Value::Null, |l| Value::string(&l.date))),
        ("state", Value::string(row.state.word())),
        ("license", info.and_then(|i| i.get("license")).or(row.notice.as_ref().map(|n| n.license.as_str())).map_or(Value::Null, Value::string)),
        ("authors", info.map(|i| i.list("authors")).or(row.notice.as_ref().map(|n| n.authors.clone())).map_or(Value::Array(Vec::new()), |a| strings(&a))),
        ("size", row.size.map_or(Value::Null, Value::number)),
    ];
    if let Some(why) = &row.reason {
        m.push(("reason", Value::string(why)));
    }
    obj(m)
}

/// A row's detail, the listing line and everything the detail pane shows (EmuSen_Cores.md §2.1, §6.1).
pub fn info_event(row: &Row, ctx: &Context) -> Value {
    let Value::Object(mut members) = row_event(row) else { unreachable!() };
    let mut add = |k: &str, v: Value| members.push((Text::new(k), v));
    if let Some(info) = &row.info {
        for (key, field) in [("description", "description"), ("hw_render", "hw_render"), ("required_hw_api", "required_hw_api"), ("savestate", "savestate")] {
            add(key, info.get(field).map_or(Value::Null, Value::string));
        }
        add("notes", strings(&info.list("notes")));
        add("permissions", strings(&info.list("permissions")));
        add("extensions", strings(&info.list("supported_extensions")));
        add("needs_fullpath", Value::Bool(info.flag("needs_fullpath")));
        add("experimental", Value::Bool(info.flag("is_experimental")));
        add(
            "firmware",
            Value::Array(info.firmware().into_iter().map(|f| obj(vec![("desc", Value::string(f.desc)), ("path", Value::string(f.path)), ("optional", Value::Bool(f.optional))])).collect()),
        );
    }
    if let Some(n) = &row.notice {
        add("sha256", Value::string(&n.sha256));
        add("from", Value::string(&n.source));
        add("fetched", Value::string(&n.fetched));
        add("index_crc32", Value::string(format!("{:08x}", n.index_crc32)));
    } else if let (Some(l), Some(s)) = (&row.index, &ctx.source) {
        add("from", Value::string(s.zip_url(&l.file)));
        add("index_crc32", Value::string(format!("{:08x}", l.crc32)));
    }
    Value::Object(members)
}

fn done(queue: &Queue, op: Op, counts: Vec<(&str, u64)>, catalogue: Option<&str>, error: Option<String>) {
    let counts = Value::Object(counts.into_iter().map(|(k, v)| (Text::new(k), Value::number(v))).collect());
    let mut m = vec![("event", Value::string("done")), ("op", Value::string(op_word(op))), ("counts", counts), ("catalogue", catalogue.map_or(Value::Null, Value::string))];
    if let Some(words) = error {
        m.push(("error", Value::string(words)));
    }
    emit(queue, obj(m));
}

fn refused(queue: &Queue, op: Op, reason: String) {
    emit(queue, obj(vec![("event", Value::string("refused")), ("reason", Value::string(&reason))]));
    done(queue, op, Vec::new(), None, Some(reason));
}

fn run(ctx: &Context, req: &Request, queue: &Queue, stop: &dyn Fn() -> bool) {
    match req.op {
        Op::Fetch => {
            emit(queue, obj(vec![("event", Value::string("progress")), ("step", Value::string("catalogue"))]));
            match catalogue::fetch(ctx, stop) {
                Ok(f) => {
                    emit(queue, obj(vec![("event", Value::string("fetched")), ("cores", Value::number(f.cores)), ("skipped", Value::number(f.skipped)), ("date", Value::string(&f.date))]));
                    done(queue, req.op, vec![("cores", f.cores as u64)], Some(&f.date), None);
                }
                Err(words) => refused(queue, req.op, words),
            }
        }
        Op::List | Op::Info => {
            let rows = match catalogue::load(ctx).and_then(|c| catalogue::rows(ctx, &c).map(|r| (c, r))) {
                Ok(both) => both,
                Err(words) => return refused(queue, req.op, words),
            };
            let (cat, rows) = rows;
            if req.op == Op::Info {
                let name = req.core.as_deref().unwrap_or("");
                match rows.iter().find(|r| r.stem == name) {
                    Some(row) => {
                        emit(queue, info_event(row, ctx));
                        done(queue, req.op, vec![(row.state.word(), 1)], cat.fetched.as_deref(), None);
                    }
                    None => refused(queue, req.op, format!("no core {name} is installed or in the catalogue{}", if cat.fetched.is_none() { "; cores fetch reads the catalogue" } else { "" })),
                }
                return;
            }
            let mut counts: Vec<(&str, u64)> = Vec::new();
            for row in rows.iter().filter(|r| shown(r, req.show)).filter(|r| req.console.as_ref().is_none_or(|c| r.systems.contains(c))) {
                if stop() {
                    break;
                }
                emit(queue, row_event(row));
                match counts.iter_mut().find(|(k, _)| *k == row.state.word()) {
                    Some((_, n)) => *n += 1,
                    None => counts.push((row.state.word(), 1)),
                }
            }
            done(queue, req.op, counts, cat.fetched.as_deref(), None);
        }
    }
}
