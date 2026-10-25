//! `emusen.library`: `chd ls`, `chd convert`, `chd check` and `chd wait`, each a job of the library tool reached
//! through the `emusen.library` service. A job is followed, one line a disc, until it ends or the line's budget is
//! spent; a longer one goes on in the background, listed by `ps` as `chd<n>`, followed again by `chd wait` and stopped
//! by `kill`. What each word means is the tool's; this module asks and prints. See EmuSen_GamePackages.md §12.3 and
//! `man chd`.

use dianash_module::{Answer, Call, Manifest, Module};
use emusen_dotnet::json::{self, Value};
use std::ffi::c_void;

pub const SERVICE: &str = "emusen.library";
pub const MAJOR: u32 = 1;

/// `emusen_library_tool.h`'s table.
#[repr(C)]
pub struct LibraryToolV1 {
    pub size: u32,
    pub start: unsafe extern "C" fn(*const u8, usize, *mut *mut c_void) -> i32,
    pub next: unsafe extern "C" fn(*mut c_void, u32, *mut u8, usize) -> i64,
    pub stop: unsafe extern "C" fn(*mut c_void) -> i32,
    pub free: unsafe extern "C" fn(*mut c_void) -> i32,
    pub last_error: unsafe extern "C" fn(*mut u8, usize) -> i64,
}

/// The platform library's ABSENT: no event yet.
const ABSENT: i64 = -1030;

pub const COMMANDS: &[(&str, bool, &str)] = &[(
    "chd",
    false,
    "  chd ls [<path>...]            the library's disc games under each path (relative to the library), each with its state\n  chd convert <path>... [--fast] [-j <n>]\n                                convert them to CHD in Compressed, each checked three ways and given a package\n  chd check [<path>...]         check converted discs again: chdman's verify, every track, the table of contents\n  chd wait [chd<n>]             follow a chd job that went on in the background (ps lists them, kill stops one)",
)];

pub struct LibraryCommands;

pub static MODULE: LibraryCommands = LibraryCommands;

const USAGE: &str = "Usage: chd ls [<path>...] | chd convert <path>... [--fast] [-j <threads>] | chd check [<path>...] | chd wait [chd<n>]\n  Paths are relative to the library (AppSettings.RomDirectory); / is all of it. See man chd.";

/// The request a command line makes, or the usage.
pub fn request(args: &[String]) -> Result<String, String> {
    let mut words = args.iter().skip(1);
    let op = match words.next().map(String::as_str) {
        Some("ls") => "ls",
        Some("convert") => "convert",
        Some("check") => "check",
        _ => return Err(USAGE.into()),
    };
    let (mut paths, mut fast, mut threads) = (Vec::new(), false, 0u64);
    while let Some(w) = words.next() {
        match w.as_str() {
            "--fast" if op == "convert" => fast = true,
            "-j" if op == "convert" => threads = words.next().and_then(|n| n.parse().ok()).filter(|&n| n > 0).ok_or_else(|| format!("chd: -j needs a number of threads above 0\n{USAGE}"))?,
            o if o.starts_with('-') && o.len() > 1 => return Err(format!("chd: {o} is not an option of chd {op}\n{USAGE}")),
            p => paths.push(Value::string(p)),
        }
    }
    if op == "convert" && paths.is_empty() {
        return Err(format!("chd convert: name what to convert, or / for the whole library\n{USAGE}"));
    }
    let mut members = vec![("op", Value::string(op)), ("paths", Value::Array(paths))];
    if op == "convert" {
        members.extend([("fast", Value::Bool(fast)), ("threads", Value::number(threads))]);
    }
    let mut out = String::new();
    json::write_compact(&mut out, &Value::Object(members.into_iter().map(|(k, v)| (json::Text::new(k), v)).collect()));
    Ok(out)
}

/// A whole number with a comma between each three digits.
pub fn grouped(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

fn text<'a>(e: &'a Value, name: &str) -> &'a str {
    e.get(name).and_then(Value::as_str).unwrap_or("")
}

fn number(e: &Value, name: &str) -> Option<u64> {
    match e.get(name) {
        Some(Value::Number(t)) => t.parse().ok(),
        _ => None,
    }
}

fn token(e: &Value, name: &str) -> String {
    match e.get(name) {
        Some(Value::Number(t)) => t.clone(),
        _ => String::new(),
    }
}

fn sizes(e: &Value) -> String {
    match (number(e, "original"), number(e, "chd")) {
        (Some(o), Some(c)) if o > 0 && c > 0 => format!("{} -> {} bytes ({:.1}%)", grouped(o), grouped(c), c as f64 * 100.0 / o as f64),
        _ => String::new(),
    }
}

fn failed_checks(e: &Value) -> String {
    let Some(checks) = e.get("checks") else { return String::new() };
    let said: Vec<String> = ["chdman", "tracks", "toc"].iter().filter_map(|c| checks.get(c).and_then(Value::as_str).filter(|w| *w != "passed").map(|w| format!("{c} {w}"))).collect();
    if said.is_empty() { String::new() } else { format!(" ({})", said.join("; ")) }
}

/// One event as the line it prints, none for an event that prints nothing.
pub fn line(e: &Value) -> Option<String> {
    match text(e, "event") {
        "start" => match text(e, "op") {
            "convert" => Some(format!("Converting {} disc(s) into {} with chdman {} ({}, {} threads).", token(e, "discs"), text(e, "output"), text(e, "chdman"), text(e, "codecs"), token(e, "threads"))),
            "check" => Some(format!("Checking {} disc(s) with chdman {}.", token(e, "discs"), text(e, "chdman"))),
            _ => None,
        },
        "disc" if text(e, "state") == "working" => None,
        "disc" if e.get("index").is_none() => {
            let tail = sizes(e);
            Some(format!("{:<14}{}{}", text(e, "state"), text(e, "source"), if tail.is_empty() { String::new() } else { format!("  {tail}") }))
        }
        "disc" => {
            let head = format!("[{}/{}] {}: {}", token(e, "index"), token(e, "of"), text(e, "source"), text(e, "state"));
            let detail = match text(e, "state") {
                "converted" => format!(", {}, three checks passed, {} s", sizes(e), token(e, "seconds")),
                "verified" => ", three checks passed".into(),
                "already" => " converted".into(),
                _ => {
                    let words = text(e, "words");
                    format!("{}{}", if words.is_empty() { String::new() } else { format!(": {words}") }, failed_checks(e))
                }
            };
            Some(head + &detail)
        }
        "end" if e.get("error").is_some() => None,
        "end" => {
            let counts = match e.get("counts") {
                Some(Value::Object(c)) => c.iter().map(|(k, v)| format!("{} {}", if let Value::Number(t) = v { t.as_str() } else { "" }, k.text)).collect::<Vec<_>>().join(", "),
                _ => String::new(),
            };
            Some(if counts.is_empty() { format!("No disc, {} s.", token(e, "seconds")) } else { format!("{counts}; {} s.", token(e, "seconds")) })
        }
        _ => None,
    }
}

/// One job this shell started: its number, what it does, the tool's handle, and how far it has got.
struct Running {
    id: u32,
    op: String,
    job: *mut c_void,
    table: *const LibraryToolV1,
    done: u64,
    of: u64,
    current: String,
    stopping: bool,
}

/// This shell's jobs, kept in its slot and freed with it.
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

fn last_error(table: &LibraryToolV1) -> String {
    let mut buffer = vec![0u8; 1024];
    let n = unsafe { (table.last_error)(buffer.as_mut_ptr(), buffer.len()) };
    String::from_utf8_lossy(&buffer[..(n.max(0) as usize).min(buffer.len())]).into_owned()
}

/// How following a job ended.
enum Followed {
    Ended(i32),
    Detached,
}

/// Prints a job's events as they come, until it ends or the line is cancelled, which its budget also does.
fn follow(call: &Call, r: &mut Running, status: &mut i32) -> Followed {
    let table = unsafe { &*r.table };
    let mut buffer = vec![0u8; 4096];
    loop {
        if call.cancelled() {
            return Followed::Detached;
        }
        let n = unsafe { (table.next)(r.job, 100, buffer.as_mut_ptr(), buffer.len()) };
        if n == 0 {
            return Followed::Ended(*status);
        }
        if n == ABSENT {
            continue;
        }
        if n < 0 {
            call.err(&format!("chd: {}\n", last_error(table)));
            return Followed::Ended(1);
        }
        if n as usize > buffer.len() {
            buffer.resize(n as usize, 0);
            continue;
        }
        let Ok(event) = json::parse(&String::from_utf8_lossy(&buffer[..n as usize])) else { continue };
        if let Some(Value::Array(refused)) = event.get("refused") {
            for words in refused.iter().filter_map(Value::as_str) {
                call.err(&format!("chd: {words}\n"));
                *status = 1;
            }
        }
        if let Some(words) = event.get("error").and_then(Value::as_str) {
            call.err(&format!("chd: {words}\n"));
            *status = 1;
        }
        match text(&event, "event") {
            "start" => r.of = number(&event, "discs").unwrap_or(0),
            "disc" if text(&event, "state") == "working" => r.current = text(&event, "source").to_string(),
            "disc" => {
                r.done += 1;
                if text(&event, "state") == "failed" {
                    *status = 1;
                }
            }
            _ => {}
        }
        if let Some(l) = line(&event) {
            call.out(&format!("{l}\n"));
        }
    }
}

/// Takes a job's waiting events without printing them, as `ps` reads how far it has got; whether it has ended. What
/// they said is in the tool's log (`Logs/chd_<day>.log`).
fn drain(r: &mut Running) -> bool {
    let table = unsafe { &*r.table };
    let mut buffer = vec![0u8; 4096];
    loop {
        let n = unsafe { (table.next)(r.job, 0, buffer.as_mut_ptr(), buffer.len()) };
        if n == 0 {
            return true;
        }
        if n < 0 {
            return false;
        }
        if n as usize > buffer.len() {
            buffer.resize(n as usize, 0);
            continue;
        }
        let Ok(event) = json::parse(&String::from_utf8_lossy(&buffer[..n as usize])) else { continue };
        match (text(&event, "event"), text(&event, "state")) {
            ("start", _) => r.of = number(&event, "discs").unwrap_or(0),
            ("disc", "working") => r.current = text(&event, "source").to_string(),
            ("disc", _) => r.done += 1,
            _ => {}
        }
    }
}

/// Follows job `r` of `jobs`; one that ends is freed, one still running is said to go on in the background.
fn follow_or_detach(call: &Call, jobs: &mut Jobs, at: usize) -> i32 {
    let mut status = 0;
    match follow(call, &mut jobs.running[at], &mut status) {
        Followed::Ended(status) => {
            let r = jobs.running.remove(at);
            unsafe { ((*r.table).free)(r.job) };
            status
        }
        Followed::Detached => {
            let id = jobs.running[at].id;
            call.out(&format!("chd{id} goes on in the background: chd wait chd{id} follows it, ps lists it, kill chd{id} stops it.\n"));
            status
        }
    }
}

fn start(call: &Call, jobs: &mut Jobs) -> Answer {
    let args = call.args();
    let request = match request(&args) {
        Ok(r) => r,
        Err(usage) => return Answer::fail(usage),
    };
    let Some(table) = call.service(SERVICE, MAJOR).map(|t| t as *const LibraryToolV1) else { return Answer::fail("chd: needs emusen.library, which this program does not provide") };
    let mut job: *mut c_void = std::ptr::null_mut();
    if unsafe { ((*table).start)(request.as_ptr(), request.len(), &mut job) } != 0 {
        return Answer::fail(format!("chd: {}", last_error(unsafe { &*table })));
    }
    jobs.next += 1;
    jobs.running.push(Running { id: jobs.next, op: args[1].clone(), job, table, done: 0, of: 0, current: String::new(), stopping: false });
    let at = jobs.running.len() - 1;
    Answer::with(String::new(), follow_or_detach(call, jobs, at))
}

fn wait(call: &Call, jobs: &mut Jobs) -> Answer {
    let args = call.args();
    let at = match args.get(2) {
        None => jobs.running.len().checked_sub(1),
        Some(id) => id.strip_prefix("chd").and_then(|n| n.parse::<u32>().ok()).and_then(|n| jobs.running.iter().position(|r| r.id == n)),
    };
    match at {
        Some(at) => Answer::with(String::new(), follow_or_detach(call, jobs, at)),
        None => Answer::fail(format!("chd wait: no such chd job{}", args.get(2).map(|a| format!(" {a}")).unwrap_or_default())),
    }
}

fn jobs_of<'a>(call: &'a Call) -> &'a mut Jobs {
    call.slot(Jobs::default)
}

impl Module for LibraryCommands {
    fn manifest(&self) -> &'static str {
        static MANIFEST: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        MANIFEST.get_or_init(|| Manifest { id: "emusen.library", version: env!("CARGO_PKG_VERSION"), services: &[(SERVICE, MAJOR, 0, true)], commands: COMMANDS, aliases: &[], jobs: &["chd"] }.json())
    }

    fn run(&self, call: &Call) -> i32 {
        call.answer(match call.command().as_str() {
            "chd" if call.args().get(1).map(String::as_str) == Some("wait") => wait(call, jobs_of(call)),
            "chd" => start(call, jobs_of(call)),
            other => Answer::fault(format!("emusen.library has no command '{other}'")),
        })
    }

    fn jobs(&self, call: &Call) {
        let jobs = jobs_of(call);
        for r in jobs.running.iter_mut() {
            let ended = drain(r);
            let progress = if ended { "ended".to_string() } else if r.stopping { "stopping".to_string() } else { format!("{}/{} discs", r.done, r.of) };
            call.out(&format!("  chd{:<7} {:<11} {progress}{}\n", r.id, r.op, if r.current.is_empty() || ended { String::new() } else { format!(", {}", r.current) }));
        }
        jobs.running.retain(|r| {
            let ended = unsafe { ((*r.table).next)(r.job, 0, std::ptr::null_mut(), 0) } == 0;
            if ended {
                unsafe { ((*r.table).free)(r.job) };
            }
            !ended
        });
    }

    fn kill(&self, call: &Call, id: &str) -> Option<i32> {
        let n = id.strip_prefix("chd")?.parse::<u32>().ok()?;
        let jobs = jobs_of(call);
        let Some(r) = jobs.running.iter_mut().find(|r| r.id == n) else {
            call.err(&format!("kill: no chd job chd{n} found.\n"));
            return Some(1);
        };
        unsafe { ((*r.table).stop)(r.job) };
        r.stopping = true;
        call.out(&format!("chd{n} is stopping: its current disc is abandoned and nothing of it is left behind; chd wait chd{n} shows how far it got.\n"));
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
        assert_eq!(request(&args("chd ls")).unwrap(), r#"{"op":"ls","paths":[]}"#);
        assert_eq!(request(&args("chd convert psx SCD --fast -j 4")).unwrap(), r#"{"op":"convert","paths":["psx","SCD"],"fast":true,"threads":4}"#);
        assert_eq!(request(&args("chd check psx/a.cue")).unwrap(), r#"{"op":"check","paths":["psx/a.cue"]}"#);
        assert!(request(&args("chd convert")).unwrap_err().contains("name what to convert"));
        assert!(request(&args("chd check --fast")).unwrap_err().contains("--fast is not an option of chd check"));
        assert!(request(&args("chd convert psx -j 0")).unwrap_err().contains("-j needs"));
        assert!(request(&args("chd")).unwrap_err().starts_with("Usage: chd"));
    }

    #[test]
    fn each_event_is_one_line() {
        let e = |t: &str| json::parse(t).unwrap();
        assert_eq!(grouped(617_209_488), "617,209,488");
        assert_eq!(grouped(12), "12");
        assert_eq!(
            line(&e(r#"{"event":"disc","index":1,"of":2,"source":"psx/a.cue","state":"converted","original":1000,"chd":523,"seconds":8.2,"checks":{"chdman":"passed","tracks":"passed","toc":"passed"}}"#)).unwrap(),
            "[1/2] psx/a.cue: converted, 1,000 -> 523 bytes (52.3%), three checks passed, 8.2 s"
        );
        assert_eq!(
            line(&e(r#"{"event":"disc","index":2,"of":2,"source":"psx/b.cue","state":"failed","words":"a check failed","checks":{"chdman":"passed","tracks":"failed: track 2","toc":"passed"}}"#)).unwrap(),
            "[2/2] psx/b.cue: failed: a check failed (tracks failed: track 2)"
        );
        assert_eq!(line(&e(r#"{"event":"disc","source":"psx/c.bin","state":"not converted","size":5}"#)).unwrap(), "not converted psx/c.bin");
        assert_eq!(line(&e(r#"{"event":"disc","index":1,"of":1,"source":"psx/a.cue","state":"working"}"#)), None);
        assert_eq!(line(&e(r#"{"event":"end","op":"convert","counts":{"converted":2,"failed":1},"seconds":3.5}"#)).unwrap(), "2 converted, 1 failed; 3.5 s.");
    }

    #[test]
    fn the_manifest_names_chd_its_jobs_and_requires_the_service() {
        let m = LibraryCommands.manifest();
        assert!(m.starts_with("{\"manifest\":1,\"id\":\"emusen.library\"") && m.contains("\"name\":\"chd\"") && m.contains("\"required\":true") && m.contains("\"jobs\":[\"chd\"]"), "{m}");
    }
}
