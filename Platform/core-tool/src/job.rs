//! A core request run as a job, its events JSON lines (EmuSen_Cores.md §6): `core-fetch`, `core-ls` and `core-info`,
//! then `core-get`, `core-update`, `core-rollback`, `core-rm` and `core-verify`, all since 2026-10-31; the requests and
//! every event are listed in §6.1 and §6.2 there.

use crate::catalogue::{self, Catalogue, Row, State};
use crate::install::{self, Installed, Note, Verified};
use crate::notes::{self, Match};
use crate::{Context, Placement};
use serde_json::Value;
use emusen_tool_job::Events as Queue;
pub use emusen_tool_job::Job;
use std::sync::Arc;

/// What a request asks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Fetch,
    List,
    Info,
    Get,
    Update,
    Rollback,
    Remove,
    Verify,
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
    /// `core-ls` gives each row's whole detail, `core-info`'s, so a window has it without asking row by row.
    pub detail: bool,
    pub core: Option<String>,
    /// The cores a changing request names, from `"core"` or `"cores"`; none for `core-update` and `core-verify` means
    /// every core they apply to.
    pub cores: Vec<String>,
    /// The request's own placement, from its `"systems"` object, in place of the context's.
    pub placement: Option<Placement>,
    /// A test's own server for the buildbot, from its `"test"` object: a loopback address and the roots it is signed by.
    pub test: Option<(std::net::SocketAddr, Vec<emusen_net::CertificateDer<'static>>)>,
}

/// Whether a request's op is the core tool's, so the platform library routes it here.
pub fn is_core_op(op: &str) -> bool {
    op.starts_with("core-")
}

fn placement_of(map: &serde_json::Map<String, Value>) -> Result<Placement, String> {
    let mut pairs: Vec<(String, Vec<String>)> = Vec::new();
    for (name, ids) in map {
        let ids = match ids {
            Value::String(s) => vec![s.clone()],
            Value::Array(a) => a.iter().map(|v| v.as_str().map(str::to_string).ok_or("a system id is not text")).collect::<Result<_, _>>()?,
            _ => return Err(format!("systems' {name} is not an id or a list of ids")),
        };
        pairs.push((name.clone(), ids));
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
/// `"systems": {<libretro database or systemid>: <system id> | [<system id>...]}` and an optional
/// `"test": {"connect": <loopback address:port>, "roots": <PEM>}`, the buildbot's name kept and its address a test's.
pub fn request(text: &str) -> Result<Request, String> {
    let doc = serde_json::from_str::<Value>(text).map_err(|e| format!("the request is not JSON: {e}"))?;
    let op = match doc.get("op").and_then(Value::as_str) {
        Some("core-fetch") => Op::Fetch,
        Some("core-ls") => Op::List,
        Some("core-info") => Op::Info,
        Some("core-get") => Op::Get,
        Some("core-update") => Op::Update,
        Some("core-rollback") => Op::Rollback,
        Some("core-rm") => Op::Remove,
        Some("core-verify") => Op::Verify,
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
    let mut cores: Vec<String> = core.iter().cloned().collect();
    match doc.get("cores") {
        None | Some(Value::Null) => {}
        Some(Value::Array(a)) => {
            for v in a {
                cores.push(v.as_str().ok_or("a core in cores is not text")?.to_string());
            }
        }
        Some(_) => return Err("cores is not a list".into()),
    }
    for c in &cores {
        install::named(c)?;
    }
    match op {
        Op::Get | Op::Remove if cores.is_empty() => return Err(format!("{} names no core", op_word(op))),
        Op::Rollback if cores.len() != 1 => return Err("core-rollback names one core".into()),
        _ => {}
    }
    let placement = match doc.get("systems") {
        None | Some(Value::Null) => None,
        Some(Value::Object(map)) => Some(placement_of(map)?),
        Some(_) => return Err("systems is not an object".into()),
    };
    let detail = doc.get("detail").and_then(Value::as_bool).unwrap_or(false);
    let test = match doc.get("test") {
        None | Some(Value::Null) => None,
        Some(t @ Value::Object(_)) => Some(test_of(t)?),
        Some(_) => return Err("test is not an object".into()),
    };
    Ok(Request { op, show, console, detail, core, cores, placement, test })
}

/// A test's server: only a loopback address, so a request can never send the buildbot's traffic to another machine
/// (EmuSen_Cores.md §3.6).
fn test_of(t: &Value) -> Result<(std::net::SocketAddr, Vec<emusen_net::CertificateDer<'static>>), String> {
    let connect: std::net::SocketAddr = t.get("connect").and_then(Value::as_str).ok_or("test names no connect address")?.parse().map_err(|_| "test's connect is not an address and port".to_string())?;
    if !connect.ip().is_loopback() {
        return Err(format!("test's connect {connect} is not a loopback address"));
    }
    let roots = emusen_net::pem_certificates(t.get("roots").and_then(Value::as_str).ok_or("test names no roots")?).filter(|r| !r.is_empty()).ok_or("test's roots hold no certificate that reads")?;
    Ok((connect, roots))
}

/// Starts a request as a job.
pub fn start(text: &str, mut ctx: Context) -> Result<Job, String> {
    let req = request(text)?;
    if let Some(p) = req.placement.clone() {
        ctx.placement = Some(p);
    }
    if let (Some((connect, roots)), Some(source)) = (req.test.clone(), ctx.source.as_mut()) {
        source.test_connect = Some(connect);
        source.test_roots = roots;
    }
    Job::spawn("emusen-core-tool", move |q, stop| run(&ctx, &req, q, stop))
}

fn obj(members: Vec<(&str, Value)>) -> Value {
    Value::Object(members.into_iter().map(|(k, v)| (String::from(k), v)).collect())
}

fn emit(queue: &Queue, event: Value) {
    let mut line = String::new();
    line.push_str(&serde_json::to_string(&event).expect("a value is JSON"));
    queue.emit(line);
}

fn strings(list: &[String]) -> Value {
    Value::Array(list.iter().map(|s| Value::String(String::from(s))).collect())
}

fn op_word(op: Op) -> &'static str {
    match op {
        Op::Fetch => "core-fetch",
        Op::List => "core-ls",
        Op::Info => "core-info",
        Op::Get => "core-get",
        Op::Update => "core-update",
        Op::Rollback => "core-rollback",
        Op::Remove => "core-rm",
        Op::Verify => "core-verify",
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
        ("event", Value::String(String::from("core"))),
        ("stem", Value::String(String::from(&row.stem))),
        ("name", Value::String(String::from(row.name()))),
        ("corename", info.and_then(|i| i.get("corename")).or(row.notice.as_ref().map(|n| n.corename.as_str())).map_or(Value::Null, Value::from)),
        ("systems", strings(&row.systems)),
        ("source", Value::String(String::from("libretro"))),
        ("version", info.and_then(|i| i.get("display_version")).map_or(Value::Null, Value::from)),
        ("build", row.notice.as_ref().map(|n| n.build.clone()).or(row.index.as_ref().map(|l| l.date.clone())).map_or(Value::Null, Value::from)),
        ("offered", row.index.as_ref().map_or(Value::Null, |l| Value::String(String::from(&l.date)))),
        ("state", Value::String(String::from(row.state.word()))),
        ("license", info.and_then(|i| i.get("license")).or(row.notice.as_ref().map(|n| n.license.as_str())).map_or(Value::Null, Value::from)),
        ("authors", info.map(|i| i.list("authors")).or(row.notice.as_ref().map(|n| n.authors.clone())).map_or(Value::Array(Vec::new()), |a| strings(&a))),
        ("size", row.size.map_or(Value::Null, Value::from)),
    ];
    if let Some(why) = &row.reason {
        m.push(("reason", Value::String(String::from(why))));
    }
    obj(m)
}

/// A row's detail, the listing line and everything the detail pane shows (EmuSen_Cores.md §2.1, §6.1).
pub fn info_event(row: &Row, ctx: &Context) -> Value {
    let Value::Object(mut members) = row_event(row) else { unreachable!() };
    let mut add = |k: &str, v: Value| {
        members.insert(String::from(k), v);
    };
    if let Some(info) = &row.info {
        for (key, field) in [("description", "description"), ("hw_render", "hw_render"), ("required_hw_api", "required_hw_api"), ("savestate", "savestate")] {
            add(key, info.get(field).map_or(Value::Null, |s| Value::String(String::from(s))));
        }
        add("notes", strings(&info.list("notes")));
        add("permissions", strings(&info.list("permissions")));
        add("extensions", strings(&info.list("supported_extensions")));
        add("needs_fullpath", Value::Bool(info.flag("needs_fullpath")));
        add("experimental", Value::Bool(info.flag("is_experimental")));
        add(
            "firmware",
            Value::Array(info.firmware().into_iter().map(|f| obj(vec![("desc", Value::String(String::from(f.desc))), ("path", Value::String(String::from(f.path))), ("optional", Value::Bool(f.optional))])).collect()),
        );
    }
    let (standing, tested) = notes::standing(&ctx.notes, &row.stem, row.notice.as_ref().map(|n| n.sha256.as_str()));
    add("tested", tested_value(standing, tested));
    if let Ok(prev) = emusen_core_entry::Entry::at(ctx.cores.join(emusen_core_entry::LIBRETRO).join(emusen_core_entry::PREVIOUS).join(&row.stem), &row.stem)
        && let Ok(n) = prev.read_notice()
    {
        add("previous", obj(vec![("build", Value::String(String::from(&n.build))), ("sha256", Value::String(String::from(&n.sha256)))]));
    }
    if let Some(n) = &row.notice {
        add("sha256", Value::String(String::from(&n.sha256)));
        add("from", Value::String(String::from(&n.source)));
        add("fetched", Value::String(String::from(&n.fetched)));
        add("index_crc32", Value::String(String::from(format!("{:08x}", n.index_crc32))));
    } else if let (Some(l), Some(s)) = (&row.index, &ctx.source) {
        add("from", Value::String(String::from(s.zip_url(&l.file))));
        add("index_crc32", Value::String(String::from(format!("{:08x}", l.crc32))));
    }
    Value::Object(members)
}

fn tested_value(standing: Match, tested: Option<&notes::Tested>) -> Value {
    let mut m = vec![("match", Value::String(String::from(standing.word())))];
    if let Some(t) = tested {
        m.push(("date", Value::String(String::from(&t.date))));
        m.push(("sha256", Value::String(String::from(&t.sha256))));
        m.push(("kit", Value::String(String::from(&t.kit))));
        m.push(("note", Value::String(String::from(&t.note))));
    }
    obj(m)
}

fn done(queue: &Queue, op: Op, counts: Vec<(&str, u64)>, catalogue: Option<&str>, error: Option<String>) {
    let counts = Value::Object(counts.into_iter().map(|(k, v)| (String::from(k), Value::from(v))).collect());
    let mut m = vec![("event", Value::String(String::from("done"))), ("op", Value::String(String::from(op_word(op)))), ("counts", counts), ("catalogue", catalogue.map_or(Value::Null, |s| Value::String(String::from(s))))];
    if let Some(words) = error {
        m.push(("error", Value::String(String::from(words))));
    }
    emit(queue, obj(m));
}

fn refused(queue: &Queue, op: Op, reason: String) {
    emit(queue, obj(vec![("event", Value::String(String::from("refused"))), ("reason", Value::String(String::from(&reason)))]));
    done(queue, op, Vec::new(), None, Some(reason));
}

fn count(counts: &mut Vec<(&'static str, u64)>, word: &'static str) {
    match counts.iter_mut().find(|(k, _)| *k == word) {
        Some((_, n)) => *n += 1,
        None => counts.push((word, 1)),
    }
}

fn refused_core(queue: &Queue, stem: &str, reason: &str) {
    emit(queue, obj(vec![("event", Value::String(String::from("refused"))), ("stem", Value::String(String::from(stem))), ("reason", Value::String(String::from(reason)))]));
}

fn installed_event(ctx: &Context, i: &Installed, word: &str) -> Value {
    let (standing, tested) = notes::standing(&ctx.notes, &i.stem, Some(&i.sha256));
    obj(vec![
        ("event", Value::String(String::from(word))),
        ("stem", Value::String(String::from(&i.stem))),
        ("sha256", Value::String(String::from(&i.sha256))),
        ("build", Value::String(String::from(&i.build))),
        ("previous", i.previous.as_ref().map_or(Value::Null, |p| Value::String(p.clone()))),
        ("revoke", strings(&i.revoke)),
        ("tested", tested_value(standing, tested)),
    ])
}

/// The stems a changing request works on: those it names, else every row the op applies to.
fn targets(req: &Request, rows: &[Row]) -> Vec<String> {
    if !req.cores.is_empty() {
        return req.cores.clone();
    }
    rows.iter()
        .filter(|r| match req.op {
            Op::Update => r.state == State::Update,
            Op::Verify => matches!(r.state, State::Installed | State::Update | State::Added),
            _ => false,
        })
        .map(|r| r.stem.clone())
        .collect()
}

/// The lock taken the first time a core passes the checks that need nothing written, so a request refused outright writes
/// nothing at all; stale staging cleared once it is held.
fn hold(ctx: &Context, lock: &mut Option<std::fs::File>) -> Result<(), String> {
    if lock.is_none() {
        *lock = Some(install::lock(ctx)?);
        install::clear_staging(ctx);
    }
    Ok(())
}

/// `core-get`, `core-update`, `core-rollback`, `core-rm` and `core-verify`, each core in turn under the lock, one
/// refusal not stopping the others (EmuSen_Cores.md §6.2).
fn change(ctx: &Context, req: &Request, queue: &Queue, stop: &dyn Fn() -> bool) {
    let mut lock = None;
    let (cat, rows) = match catalogue::load(ctx).and_then(|c| catalogue::rows(ctx, &c).map(|r| (c, r))) {
        Ok(both) => both,
        Err(words) => return refused(queue, req.op, words),
    };
    let (mut counts, mut failed): (Vec<(&'static str, u64)>, u64) = (Vec::new(), 0);
    let stems = targets(req, &rows);
    for stem in &stems {
        if stop() {
            break;
        }
        let outcome = one(ctx, &cat, &rows, req, stem, queue, stop, &mut lock);
        match outcome {
            Ok(word) => count(&mut counts, word),
            Err(reason) => {
                refused_core(queue, stem, &reason);
                count(&mut counts, "refused");
                failed += 1;
            }
        }
    }
    let error = (failed > 0).then(|| format!("{failed} of {} refused", stems.len()));
    let error = if stop() && error.is_none() { Some("stopped".to_string()) } else { error };
    done(queue, req.op, counts, cat.fetched.as_deref(), error);
}

#[allow(clippy::too_many_arguments)]
fn one(ctx: &Context, cat: &Catalogue, rows: &[Row], req: &Request, stem: &str, queue: &Queue, stop: &dyn Fn() -> bool, lock: &mut Option<std::fs::File>) -> Result<&'static str, String> {
    let mut note = |n: Note| match n {
        Note::Progress { step, bytes, total } => emit(
            queue,
            obj(vec![("event", Value::String(String::from("progress"))), ("stem", Value::String(String::from(stem))), ("step", Value::String(String::from(step))), ("bytes", Value::from(bytes)), ("total", total.map_or(Value::Null, Value::from))]),
        ),
        Note::Checked(check) => emit(queue, obj(vec![("event", Value::String(String::from("checked"))), ("stem", Value::String(String::from(stem))), ("check", Value::String(String::from(check)))])),
    };
    let row = rows.iter().find(|r| r.stem == stem);
    let plan = |queue: &Queue| {
        if let (Some(line), Some(source)) = (row.and_then(|r| r.index.as_ref()), &ctx.source) {
            emit(queue, obj(vec![("event", Value::String(String::from("plan"))), ("stem", Value::String(String::from(stem))), ("from", Value::String(String::from(source.zip_url(&line.file)))), ("build", Value::String(String::from(&line.date))), ("crc32", Value::String(String::from(format!("{:08x}", line.crc32))))]));
        }
    };
    let on_disk = || {
        let libretro = ctx.cores.join(emusen_core_entry::LIBRETRO);
        libretro.join(stem).is_dir() || libretro.join(emusen_core_entry::PREVIOUS).join(stem).is_dir()
    };
    if matches!(req.op, Op::Get | Op::Update)
        && let Some(why) = ctx.refuses(stem)
    {
        return Err(why);
    }
    match req.op {
        Op::Get => match row {
            Some(r) if r.state == State::Unsupported => return Err(format!("{stem} is not supported: {}", r.reason.as_deref().unwrap_or("EmuSen's host cannot run it"))),
            Some(r) if r.index.is_some() || on_disk() => {}
            _ if cat.fetched.is_none() => return Err(format!("{stem} is not in the catalogue, which was never fetched; cores fetch reads it")),
            _ => return Err(format!("{stem} is not in this platform's index; cores fetch reads it again")),
        },
        _ if !on_disk() => return Err(format!("{stem} is not installed")),
        _ => {}
    }
    hold(ctx, lock)?;
    match req.op {
        Op::Get => {
            plan(queue);
            let i = install::get(ctx, cat, stem, stop, &mut note)?;
            emit(queue, installed_event(ctx, &i, "installed"));
            Ok("installed")
        }
        Op::Update => {
            if !req.cores.is_empty() && row.is_some_and(|r| r.state == State::Installed) {
                emit(queue, obj(vec![("event", Value::String(String::from("current"))), ("stem", Value::String(String::from(stem)))]));
                return Ok("current");
            }
            plan(queue);
            let i = install::update(ctx, cat, stem, stop, &mut note)?;
            emit(queue, installed_event(ctx, &i, "installed"));
            Ok("installed")
        }
        Op::Rollback => {
            let i = install::rollback(ctx, stem)?;
            emit(queue, installed_event(ctx, &i, "restored"));
            Ok("restored")
        }
        Op::Remove => {
            let revoke = install::remove(ctx, stem)?;
            emit(queue, obj(vec![("event", Value::String(String::from("removed"))), ("stem", Value::String(String::from(stem))), ("revoke", strings(&revoke))]));
            Ok("removed")
        }
        Op::Verify => {
            let (word, mut m): (&'static str, Vec<(&str, Value)>) = match install::verify(ctx, stem)? {
                Verified::Ok { sha256 } => ("ok", vec![("sha256", Value::String(String::from(sha256)))]),
                Verified::Refreshed { sha256, files } => ("refreshed", vec![("sha256", Value::String(String::from(sha256))), ("files", strings(&files))]),
                Verified::Changed { reason } => ("changed", vec![("reason", Value::String(String::from(reason)))]),
                Verified::Added => ("added", Vec::new()),
            };
            let mut event = vec![("event", Value::String(String::from("verified"))), ("stem", Value::String(String::from(stem))), ("result", Value::String(String::from(word)))];
            event.append(&mut m);
            emit(queue, obj(event));
            Ok(word)
        }
        _ => unreachable!(),
    }
}

fn run(ctx: &Context, req: &Request, queue: &Queue, stop: &dyn Fn() -> bool) {
    match req.op {
        Op::Get | Op::Update | Op::Rollback | Op::Remove | Op::Verify => change(ctx, req, queue, stop),
        Op::Fetch => {
            emit(queue, obj(vec![("event", Value::String(String::from("progress"))), ("step", Value::String(String::from("catalogue")))]));
            match catalogue::fetch(ctx, stop) {
                Ok(f) => {
                    emit(queue, obj(vec![("event", Value::String(String::from("fetched"))), ("cores", Value::from(f.cores)), ("skipped", Value::from(f.skipped)), ("date", Value::String(String::from(&f.date)))]));
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
                emit(queue, if req.detail { info_event(row, ctx) } else { row_event(row) });
                match counts.iter_mut().find(|(k, _)| *k == row.state.word()) {
                    Some((_, n)) => *n += 1,
                    None => counts.push((row.state.word(), 1)),
                }
            }
            done(queue, req.op, counts, cat.fetched.as_deref(), None);
        }
    }
}
