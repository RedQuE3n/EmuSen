//! `emusen.cores`: `cores ls`, `info`, `fetch`, `get`, `update`, `rollback`, `rm`, `verify` and `wait`, each a job of the
//! core tool reached through the `emusen.library` service from its minor 5. Plain lines that pipe; a job the line's budget cuts short goes on in
//! the background as `cores<n>`. What each word means is the tool's; this module asks and prints. See EmuSen_Cores.md
//! §8 and `man cores`.

use dianash_module::{Answer, Call, Manifest, Module};
use emusen_dotnet::json::{self, Text, Value};
use emusen_library_module::{last_error, next_event, LibraryToolV1, Next, MAJOR, SERVICE};
use std::ffi::c_void;

/// The service's minor that runs the core tool's requests.
pub const MINOR: u32 = 5;

pub const COMMANDS: &[(&str, bool, &str)] = &[(
    "cores",
    false,
    "  cores ls [--installed|--available|--updates|--unsupported] [<console>]\n                                one line per libretro core: state, console, id, source, version, size, licence\n  cores info <core>             what is known of one core, as key: value lines\n  cores fetch                   libretro's catalogue and this platform's index fetched again (a network request)\n  cores get <core>...           download, check and install; prints the SHA-256 to approve\n  cores update [<core>...]      the installed cores the index offers a new build of, or those named\n  cores rollback <core>         the build kept by the last update put back in place\n  cores rm <core>...            a downloaded core removed, with its kept build; saves and settings stay\n  cores verify [<core>...]      each installed core checked against its NOTICE, the project's copies refreshed\n  cores wait [cores<n>]         follow a cores job that went on in the background (ps lists them, kill stops one)",
)];

const USAGE: &str = "Usage: cores ls [--installed|--available|--updates|--unsupported] [<console>] | cores info <core> | cores fetch | cores get <core>... | cores update [<core>...] | cores rollback <core> | cores rm <core>... | cores verify [<core>...] | cores wait [cores<n>]\n  A core is named by its file's stem, as cores ls prints it. See man cores.";

pub struct CoresCommands;

pub static MODULE: CoresCommands = CoresCommands;

fn compact(members: Vec<(&str, Value)>) -> String {
    let mut out = String::new();
    json::write_compact(&mut out, &Value::Object(members.into_iter().map(|(k, v)| (Text::new(k), v)).collect()));
    out
}

/// The request a command line makes, or the usage.
pub fn request(args: &[String]) -> Result<String, String> {
    let words: Vec<&str> = args.iter().skip(1).map(String::as_str).collect();
    match words.as_slice() {
        ["ls", rest @ ..] => {
            let (mut show, mut console) = ("all", None);
            for w in rest {
                match *w {
                    "--installed" | "--available" | "--updates" | "--unsupported" if show == "all" => show = &w[2..],
                    o if o.starts_with('-') => return Err(format!("cores ls: {o} is not an option of cores ls, or a second one\n{USAGE}")),
                    c if console.is_none() => console = Some(c),
                    _ => return Err(format!("cores ls: one console at most\n{USAGE}")),
                }
            }
            Ok(compact(vec![("op", Value::string("core-ls")), ("show", Value::string(show)), ("console", console.map_or(Value::Null, Value::string))]))
        }
        ["info", core] => Ok(compact(vec![("op", Value::string("core-info")), ("core", Value::string(*core))])),
        ["info", ..] => Err(format!("cores info: name one core\n{USAGE}")),
        ["fetch"] => Ok(compact(vec![("op", Value::string("core-fetch"))])),
        [verb @ ("get" | "update" | "rollback" | "rm" | "verify"), names @ ..] => {
            if let Some(o) = names.iter().find(|n| n.starts_with('-')) {
                return Err(format!("cores {verb}: {o} is not an option of cores {verb}\n{USAGE}"));
            }
            match (*verb, names.len()) {
                ("get" | "rm", 0) => return Err(format!("cores {verb}: name a core or more\n{USAGE}")),
                ("rollback", n) if n != 1 => return Err(format!("cores rollback: name one core\n{USAGE}")),
                _ => {}
            }
            let op = match *verb {
                "rm" => "core-rm".to_string(),
                v => format!("core-{v}"),
            };
            Ok(compact(vec![("op", Value::string(op)), ("cores", Value::Array(names.iter().map(|n| Value::string(*n)).collect()))]))
        }
        _ => Err(USAGE.into()),
    }
}

fn text<'a>(e: &'a Value, k: &str) -> &'a str {
    e.get(k).and_then(Value::as_str).unwrap_or("")
}

/// A field for a whitespace-separated line: `-` when empty, inner spaces joined by `_`.
fn word(s: &str) -> String {
    let s = s.split_whitespace().collect::<Vec<_>>().join("_");
    if s.is_empty() { "-".into() } else { s }
}

fn strings(e: &Value, k: &str) -> Vec<String> {
    match e.get(k) {
        Some(Value::Array(a)) => a.iter().filter_map(Value::as_str).map(str::to_string).collect(),
        _ => Vec::new(),
    }
}

fn size(e: &Value) -> String {
    match e.get("size") {
        Some(Value::Number(n)) => n.parse::<f64>().map_or_else(|_| n.clone(), |b| format!("{:.1}MB", b / 1_048_576.0)),
        _ => "-".into(),
    }
}

/// A `core` event as `cores ls` prints it: state, console, id, source, version, size, licence (the rest of the line).
pub fn listing_line(e: &Value) -> String {
    let systems = strings(e, "systems");
    let version = [text(e, "version"), text(e, "build")].into_iter().find(|v| !v.is_empty()).unwrap_or("");
    format!("{:<12} {:<8} {:<20} {:<9} {:<12} {:>8} {}", text(e, "state"), word(&systems.join(",")), text(e, "stem"), text(e, "source"), word(version), size(e), text(e, "license"))
}

/// A `core` event as `cores info` prints it, one `key: value` line each, lists joined by `, `.
pub fn info_lines(e: &Value) -> String {
    let mut out = String::new();
    if let Value::Object(members) = e {
        for (k, v) in members.iter().filter(|(k, _)| k.text != "event") {
            let value = match v {
                Value::Null => continue,
                Value::String(s) => s.text.clone(),
                Value::Bool(b) => b.to_string(),
                Value::Number(n) => n.clone(),
                Value::Array(a) if k.text == "firmware" => a
                    .iter()
                    .map(|f| format!("{} ({}{})", text(f, "desc"), text(f, "path"), if f.get("optional") == Some(&Value::Bool(true)) { ", optional" } else { ", required" }))
                    .collect::<Vec<_>>()
                    .join("; "),
                Value::Array(a) => a.iter().filter_map(Value::as_str).collect::<Vec<_>>().join(", "),
                Value::Object(_) => continue,
            };
            if !value.is_empty() {
                out.push_str(&format!("{}: {value}\n", k.text));
            }
        }
    }
    out
}

/// What an event prints, to the output or the errors; none for an event that prints nothing.
pub fn printed(e: &Value, listing: bool) -> Option<(bool, String)> {
    match text(e, "event") {
        "core" if listing => Some((true, listing_line(e))),
        "core" => Some((true, info_lines(e).trim_end().to_string())),
        "fetched" => Some((true, format!("Fetched libretro's catalogue on {}: {} cores for this platform{}.", text(e, "date"), number(e, "cores"), match number(e, "skipped") { 0 => String::new(), n => format!(", {n} index lines not a core of it") }))),
        "refused" if !text(e, "stem").is_empty() => Some((false, format!("cores: {}: {}", text(e, "stem"), text(e, "reason")))),
        "refused" => Some((false, format!("cores: {}", text(e, "reason")))),
        "installed" => Some((true, installed_lines(e))),
        "restored" => Some((true, format!("Rolled {} back to the build of {}, SHA-256 {}; the newer build is kept, and cores rollback {} returns to it.", text(e, "stem"), text(e, "build"), text(e, "sha256"), text(e, "stem")))),
        "removed" => Some((true, removed_lines(e))),
        "current" => Some((true, format!("{} is the build the index offers.", text(e, "stem")))),
        "verified" => Some((true, verified_line(e))),
        "done" if listing && e.get("catalogue").is_some_and(Value::is_null) => Some((false, "cores: no catalogue yet; cores fetch reads it from buildbot.libretro.com".into())),
        _ => None,
    }
}

/// What a finished install or update says: the hash to approve, as trust on first download, and what EmuSen tested.
pub fn installed_lines(e: &Value) -> String {
    let stem = text(e, "stem");
    let updated = !text(e, "previous").is_empty();
    let mut out = format!("{} {stem}, build {}, SHA-256 {}.\n", if updated { "Updated" } else { "Installed" }, text(e, "build"), text(e, "sha256"));
    out.push_str("  Nothing downloaded runs until you approve it. libretro signs no build, so approving trusts the build the buildbot served, on first download, and the hash holds it to exactly that build.\n");
    let tested = e.get("tested");
    let date = tested.map_or("", |t| text(t, "date"));
    out.push_str(&match tested.map_or("none", |t| text(t, "match")) {
        "exact" => format!("  This exact build was tested by EmuSen on {date}.\n"),
        "other" => format!("  Another build of it was tested by EmuSen on {date}; this one was not.\n"),
        _ => "  Not tested by EmuSen.\n".into(),
    });
    if updated {
        out.push_str(&format!("  The build before it is kept: cores rollback {stem} puts it back.\n"));
    }
    let revoke = strings(e, "revoke");
    if !revoke.is_empty() {
        out.push_str(&format!("  No longer on disk, their approvals now match no file: {}\n", revoke.join(" ")));
    }
    out.trim_end().to_string()
}

/// What a removal says: the folder gone, the player's saves and the core's settings kept, and the hashes it was approved by.
pub fn removed_lines(e: &Value) -> String {
    let mut out = format!("Removed {}; your saves and its settings are kept.", text(e, "stem"));
    let revoke = strings(e, "revoke");
    if !revoke.is_empty() {
        out.push_str(&format!("\n  Approvals recorded for it, now matching no file: {}", revoke.join(" ")));
    }
    out
}

/// One `verify` result as a line that pipes: result, core, then the files refreshed or the reason.
pub fn verified_line(e: &Value) -> String {
    let (result, stem) = (text(e, "result"), text(e, "stem"));
    match result {
        "refreshed" => format!("refreshed {stem} {}", strings(e, "files").join(" ")),
        "changed" => format!("changed {stem} {}", text(e, "reason")),
        "added" => format!("added {stem} your own folder, not checked"),
        _ => format!("{result} {stem}"),
    }
}

fn number(e: &Value, k: &str) -> u64 {
    match e.get(k) {
        Some(Value::Number(n)) => n.parse().unwrap_or(0),
        _ => 0,
    }
}

struct Running {
    id: u32,
    op: String,
    listing: bool,
    job: *mut c_void,
    table: *const LibraryToolV1,
    stopping: bool,
}

#[derive(Default)]
struct Jobs {
    next: u32,
    running: Vec<Running>,
}

// SAFETY: a job's handle and the service's table are the platform library's, safe from any thread (emusen_platform.h).
unsafe impl Send for Jobs {}

impl Drop for Jobs {
    fn drop(&mut self) {
        for r in self.running.drain(..) {
            unsafe { ((*r.table).free)(r.job) };
        }
    }
}

/// Prints a job's events until it ends (its status) or the line is cancelled (none: it goes on in the background).
fn follow(call: &Call, r: &Running) -> Option<i32> {
    let table = unsafe { &*r.table };
    let (mut buffer, mut status) = (vec![0u8; 4096], 0);
    loop {
        if call.cancelled() {
            return None;
        }
        let line = match next_event(table, r.job, 100, &mut buffer) {
            Next::Ended => return Some(status),
            Next::Absent => continue,
            Next::Failed(words) => {
                call.err(&format!("cores: {words}\n"));
                return Some(1);
            }
            Next::Event(line) => line,
        };
        let Ok(event) = json::parse(&line) else { continue };
        if text(&event, "event") == "refused" {
            status = 1;
        }
        match printed(&event, r.listing) {
            Some((true, l)) => call.out(&format!("{l}\n")),
            Some((false, l)) => call.err(&format!("{l}\n")),
            None => {}
        }
    }
}

fn follow_or_detach(call: &Call, jobs: &mut Jobs, at: usize) -> i32 {
    match follow(call, &jobs.running[at]) {
        Some(status) => {
            let r = jobs.running.remove(at);
            unsafe { ((*r.table).free)(r.job) };
            status
        }
        None => {
            let id = jobs.running[at].id;
            call.out(&format!("cores{id} goes on in the background: cores wait cores{id} follows it, ps lists it, kill cores{id} stops it.\n"));
            0
        }
    }
}

fn start(call: &Call, jobs: &mut Jobs) -> Answer {
    let args = call.args();
    let request = match request(&args) {
        Ok(r) => r,
        Err(usage) => return Answer::fail(usage),
    };
    let Some(table) = call.service(SERVICE, MAJOR).map(|t| t as *const LibraryToolV1) else { return Answer::fail("cores: needs emusen.library, which this program does not provide") };
    let mut job: *mut c_void = std::ptr::null_mut();
    if unsafe { ((*table).start)(request.as_ptr(), request.len(), &mut job) } != 0 {
        return Answer::fail(format!("cores: {}", last_error(unsafe { &*table })));
    }
    jobs.next += 1;
    let op = args[1].clone();
    jobs.running.push(Running { id: jobs.next, listing: op == "ls", op, job, table, stopping: false });
    let at = jobs.running.len() - 1;
    Answer::with(String::new(), follow_or_detach(call, jobs, at))
}

fn wait(call: &Call, jobs: &mut Jobs) -> Answer {
    let args = call.args();
    let at = match args.get(2) {
        None => jobs.running.len().checked_sub(1),
        Some(id) => id.strip_prefix("cores").and_then(|n| n.parse::<u32>().ok()).and_then(|n| jobs.running.iter().position(|r| r.id == n)),
    };
    match at {
        Some(at) => Answer::with(String::new(), follow_or_detach(call, jobs, at)),
        None => Answer::fail(format!("cores wait: no such cores job{}", args.get(2).map(|a| format!(" {a}")).unwrap_or_default())),
    }
}

fn jobs_of<'a>(call: &'a Call) -> &'a mut Jobs {
    call.slot(Jobs::default)
}

impl Module for CoresCommands {
    fn manifest(&self) -> &'static str {
        static MANIFEST: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        MANIFEST.get_or_init(|| Manifest { id: "emusen.cores", version: env!("CARGO_PKG_VERSION"), services: &[(SERVICE, MAJOR, MINOR, true)], commands: COMMANDS, aliases: &[], jobs: &["cores"] }.json())
    }

    fn run(&self, call: &Call) -> i32 {
        call.answer(match call.command().as_str() {
            "cores" if call.args().get(1).map(String::as_str) == Some("wait") => wait(call, jobs_of(call)),
            "cores" => start(call, jobs_of(call)),
            other => Answer::fault(format!("emusen.cores has no command '{other}'")),
        })
    }

    fn jobs(&self, call: &Call) {
        let jobs = jobs_of(call);
        jobs.running.retain(|r| {
            let ended = unsafe { ((*r.table).next)(r.job, 0, std::ptr::null_mut(), 0) } == 0;
            call.out(&format!("  cores{:<5} {:<11} {}\n", r.id, r.op, if ended { "ended" } else if r.stopping { "stopping" } else { "running" }));
            if ended {
                unsafe { ((*r.table).free)(r.job) };
            }
            !ended
        });
    }

    fn kill(&self, call: &Call, id: &str) -> Option<i32> {
        let n = id.strip_prefix("cores")?.parse::<u32>().ok()?;
        let Some(r) = jobs_of(call).running.iter_mut().find(|r| r.id == n) else {
            call.err(&format!("kill: no cores job cores{n} found.\n"));
            return Some(1);
        };
        unsafe { ((*r.table).stop)(r.job) };
        r.stopping = true;
        call.out(&format!("cores{n} is stopping: nothing it had not finished is kept; cores wait cores{n} shows how far it got.\n"));
        Some(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(line: &str) -> Vec<String> {
        line.split_whitespace().map(str::to_string).collect()
    }

    #[test]
    fn a_command_line_is_a_request_or_the_usage() {
        assert_eq!(request(&args("cores ls")).unwrap(), r#"{"op":"core-ls","show":"all","console":null}"#);
        assert_eq!(request(&args("cores ls --updates ps2")).unwrap(), r#"{"op":"core-ls","show":"updates","console":"ps2"}"#);
        assert_eq!(request(&args("cores info pcsx2")).unwrap(), r#"{"op":"core-info","core":"pcsx2"}"#);
        assert_eq!(request(&args("cores fetch")).unwrap(), r#"{"op":"core-fetch"}"#);
        assert!(request(&args("cores ls --installed --updates")).unwrap_err().contains("or a second one"));
        assert!(request(&args("cores ls ps2 psx")).unwrap_err().contains("one console at most"));
        assert!(request(&args("cores info")).unwrap_err().contains("name one core"));
        assert_eq!(request(&args("cores get pcsx2 mednafen_psx")).unwrap(), r#"{"op":"core-get","cores":["pcsx2","mednafen_psx"]}"#);
        assert_eq!(request(&args("cores update")).unwrap(), r#"{"op":"core-update","cores":[]}"#);
        assert_eq!(request(&args("cores rm pcsx2")).unwrap(), r#"{"op":"core-rm","cores":["pcsx2"]}"#);
        assert_eq!(request(&args("cores rollback pcsx2")).unwrap(), r#"{"op":"core-rollback","cores":["pcsx2"]}"#);
        assert_eq!(request(&args("cores verify")).unwrap(), r#"{"op":"core-verify","cores":[]}"#);
        assert!(request(&args("cores get")).unwrap_err().contains("name a core or more"));
        assert!(request(&args("cores rollback a b")).unwrap_err().contains("name one core"));
        assert!(request(&args("cores verify --all")).unwrap_err().contains("not an option"));
        assert!(request(&args("cores approve pcsx2")).unwrap_err().starts_with("Usage: cores"));
        assert!(request(&args("cores")).unwrap_err().starts_with("Usage: cores"));
    }

    #[test]
    fn each_event_prints_as_lines_that_pipe() {
        let e = |t: &str| json::parse(t).unwrap();
        let row = e(r#"{"event":"core","stem":"pcsx2","name":"Sony - PlayStation 2 (LRPS2)","corename":"LRPS2","systems":["ps2"],"source":"libretro","version":"2.3 beta","build":"2026-10-31","state":"available","license":"GPLv3","authors":["PCSX2 Team"],"size":null}"#);
        let line = listing_line(&row);
        let fields: Vec<&str> = line.split_whitespace().collect();
        assert_eq!(fields, ["available", "ps2", "pcsx2", "libretro", "2.3_beta", "-", "GPLv3"]);
        let sized = e(r#"{"event":"core","stem":"x","systems":[],"source":"libretro","state":"installed","size":8388608,"license":"GPLv2 or later"}"#);
        assert_eq!(listing_line(&sized).split_whitespace().collect::<Vec<_>>(), ["installed", "-", "x", "libretro", "-", "8.0MB", "GPLv2", "or", "later"]);
        let info = info_lines(&e(r#"{"event":"core","stem":"pcsx2","authors":["A","B"],"size":null,"needs_fullpath":true,"firmware":[{"desc":"BIOS","path":"pcsx2/bios","optional":false}]}"#));
        assert_eq!(info, "stem: pcsx2\nauthors: A, B\nneeds_fullpath: true\nfirmware: BIOS (pcsx2/bios, required)\n");
        assert_eq!(printed(&e(r#"{"event":"refused","reason":"no core x"}"#), false), Some((false, "cores: no core x".into())));
        assert_eq!(printed(&e(r#"{"event":"fetched","cores":5,"skipped":0,"date":"2026-10-31"}"#), false).unwrap().1, "Fetched libretro's catalogue on 2026-10-31: 5 cores for this platform.");
        assert!(printed(&e(r#"{"event":"done","op":"core-ls","counts":{},"catalogue":null}"#), true).unwrap().1.contains("cores fetch"));
        assert_eq!(printed(&e(r#"{"event":"done","op":"core-ls","counts":{},"catalogue":"2026-10-31"}"#), true), None);
        let h = "a".repeat(64);
        let installed = printed(&e(&format!(r#"{{"event":"installed","stem":"pcsx2","sha256":"{h}","build":"2026-10-31","previous":null,"revoke":[],"tested":{{"match":"exact","date":"2026-10-30"}}}}"#)), false).unwrap();
        assert!(installed.0);
        assert!(installed.1.starts_with(&format!("Installed pcsx2, build 2026-10-31, SHA-256 {h}.\n")), "{}", installed.1);
        assert!(installed.1.contains("on first download") && installed.1.contains("This exact build was tested by EmuSen on 2026-10-30."));
        let updated = printed(&e(&format!(r#"{{"event":"installed","stem":"pcsx2","sha256":"{h}","build":"2026-10-31","previous":"{h}","revoke":["{h}"],"tested":{{"match":"none"}}}}"#)), false).unwrap().1;
        assert!(updated.starts_with("Updated pcsx2") && updated.contains("cores rollback pcsx2") && updated.contains("Not tested by EmuSen.") && updated.contains("match no file"));
        assert_eq!(printed(&e(r#"{"event":"verified","stem":"pcsx2","result":"refreshed","files":["emusen-libretro-host.so","broker.target"]}"#), false).unwrap().1, "refreshed pcsx2 emusen-libretro-host.so broker.target");
        assert_eq!(printed(&e(r#"{"event":"verified","stem":"pcsx2","result":"ok","sha256":"x"}"#), false).unwrap().1, "ok pcsx2");
        assert_eq!(printed(&e(r#"{"event":"refused","stem":"pcsx2","reason":"no"}"#), false), Some((false, "cores: pcsx2: no".into())));
        assert_eq!(printed(&e(r#"{"event":"removed","stem":"pcsx2","revoke":[]}"#), false).unwrap().1, "Removed pcsx2; your saves and its settings are kept.");
        assert_eq!(printed(&e(r#"{"event":"checked","stem":"pcsx2","check":"crc32"}"#), false), None);
    }

    #[test]
    fn the_manifest_names_cores_its_jobs_and_requires_minor_five() {
        let m = MODULE.manifest();
        assert!(m.contains(r#""id":"emusen.cores""#), "{m}");
        assert!(m.contains("\"cores\""));
        assert!(m.contains("emusen.library"));
    }
}
