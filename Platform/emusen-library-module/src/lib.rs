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

pub const COMMANDS: &[(&str, bool, &str)] = &[
    (
        "chd",
        false,
        "  chd ls [<path>...]            the library's disc games under each path (relative to the library), each with its state\n  chd convert <path>... [--fast] [-j <n>] [--output <folder> | --beside]\n                                convert them to CHD in Compressed (or ChdOutputDirectory, or the folder named),\n                                or beside each original in the library; each checked three ways and given a package\n  chd check [<path>...]         check converted discs again: chdman's verify, every track, the table of contents\n  chd wait [chd<n>]             follow a chd job that went on in the background (ps lists them, kill stops one)",
    ),
    (
        "pkg",
        false,
        "  pkg make <path>... [--self-contained] [--zip]\n                                a package for each game under each path (relative to the library), linked\n                                unless the game is to be copied in; --zip also writes the package as one zip\n  pkg ls [<console>]            the packages, with mode and state (ok, moved, missing, changed)\n  pkg check [<package>...]      the game and every file hashed against the manifest (relative to Packages); a package's zip too\n  pkg migrate <package>...      the game's saves, states, records, settings, art and cheats copied in\n  pkg patch add <package> <file>\n                                an IPS, UPS or BPS patch checked, copied in and applied at load\n  pkg patch rm <package> <n>    patch n taken out of the package's list\n  pkg zip <package>...          a self-contained package written as one zip beside it, saves and all\n  pkg identity <package> original|patched\n                                which identity the scraper sends for a patched package\n  pkg wait [pkg<n>]             follow a pkg job that went on in the background",
    ),
];

pub struct LibraryCommands;

pub static MODULE: LibraryCommands = LibraryCommands;

const USAGE: &str = "Usage: chd ls [<path>...] | chd convert <path>... [--fast] [-j <threads>] [--output <folder> | --beside] | chd check [<path>...] | chd wait [chd<n>]\n  Paths are relative to the library (AppSettings.RomDirectory); / is all of it. See man chd.";

const PKG_USAGE: &str = "Usage: pkg make <path>... [--self-contained] [--zip] | pkg ls [<console>] | pkg check [<package>...] | pkg migrate <package>... | pkg patch add <package> <file> | pkg patch rm <package> <n> | pkg zip <package>... | pkg identity <package> original|patched | pkg wait [pkg<n>]\n  Paths are relative to the library, / for all of it; packages are relative to Packages. See man pkg.";

fn compact(members: Vec<(&str, Value)>) -> String {
    let mut out = String::new();
    json::write_compact(&mut out, &Value::Object(members.into_iter().map(|(k, v)| (json::Text::new(k), v)).collect()));
    out
}

/// `pkg patch add <package> <file>` and `pkg patch rm <package> <n>`: the patch file is a shell path, resolved here.
fn patch_request(args: &[String], resolve: &dyn Fn(&str) -> Result<String, String>) -> Result<String, String> {
    let words: Vec<&str> = args.iter().skip(2).map(String::as_str).collect();
    match words.as_slice() {
        ["add", package, file] => {
            let real = resolve(file).map_err(|w| format!("pkg patch add: {w}"))?;
            Ok(compact(vec![("op", Value::string("pkg-patch-add")), ("paths", Value::Array(vec![Value::string(*package)])), ("patch", Value::string(&real))]))
        }
        ["rm", package, n] => {
            let n: u64 = n.parse().ok().filter(|&n| n > 0).ok_or_else(|| format!("pkg patch rm: {n} is not a patch's number; pkg ls numbers them from 1\n{PKG_USAGE}"))?;
            Ok(compact(vec![("op", Value::string("pkg-patch-rm")), ("paths", Value::Array(vec![Value::string(*package)])), ("index", Value::number(n))]))
        }
        _ => Err(format!("pkg patch: add <package> <file> or rm <package> <n>\n{PKG_USAGE}")),
    }
}

/// `pkg identity <package> original|patched`.
fn identity_request(args: &[String]) -> Result<String, String> {
    let words: Vec<&str> = args.iter().skip(2).map(String::as_str).collect();
    match words.as_slice() {
        [package, which @ ("original" | "patched")] => Ok(compact(vec![("op", Value::string("pkg-identity")), ("paths", Value::Array(vec![Value::string(*package)])), ("identity", Value::string(*which))])),
        _ => Err(format!("pkg identity: <package> original or <package> patched\n{PKG_USAGE}")),
    }
}

/// The request a `pkg` command line makes, or the usage; `resolve` turns a shell path into a real one.
pub fn pkg_request(args: &[String], resolve: &dyn Fn(&str) -> Result<String, String>) -> Result<String, String> {
    let mut words = args.iter().skip(1);
    let sub = words.next().map(String::as_str).unwrap_or("");
    let op = match sub {
        "make" => "pkg-make",
        "ls" => "pkg-ls",
        "check" => "pkg-check",
        "migrate" => "pkg-migrate",
        "zip" => "pkg-zip",
        "patch" => return patch_request(args, resolve),
        "identity" => return identity_request(args),
        _ => return Err(PKG_USAGE.into()),
    };
    let mut paths = Vec::new();
    let (mut contained, mut zip) = (false, false);
    for w in words {
        match w.as_str() {
            "--self-contained" if op == "pkg-make" => contained = true,
            "--zip" if op == "pkg-make" => zip = true,
            o if o.starts_with('-') && o.len() > 1 => return Err(format!("pkg: {o} is not an option of pkg {sub}\n{PKG_USAGE}")),
            p => paths.push(Value::string(p)),
        }
    }
    if matches!(op, "pkg-make" | "pkg-migrate" | "pkg-zip") && paths.is_empty() {
        return Err(format!("pkg {sub}: name what to {sub}, or / for all of it\n{PKG_USAGE}"));
    }
    if op == "pkg-ls" {
        if paths.len() > 1 {
            return Err(format!("pkg ls: one console at most\n{PKG_USAGE}"));
        }
        let console = paths.pop().unwrap_or(Value::Null);
        return Ok(compact(vec![("op", Value::string(op)), ("paths", Value::Array(Vec::new())), ("console", console)]));
    }
    if contained || zip {
        return Ok(compact(vec![("op", Value::string(op)), ("paths", Value::Array(paths)), ("self_contained", Value::Bool(true)), ("zip", Value::Bool(zip))]));
    }
    Ok(compact(vec![("op", Value::string(op)), ("paths", Value::Array(paths))]))
}

/// The request a command line makes, or the usage.
/// A `chd` command line as a request; `resolve` turns `--output`'s folder, a shell path, into the host's.
pub fn request(args: &[String], resolve: &dyn Fn(&str) -> Result<String, String>) -> Result<String, String> {
    let mut words = args.iter().skip(1);
    let op = match words.next().map(String::as_str) {
        Some("ls") => "ls",
        Some("convert") => "convert",
        Some("check") => "check",
        _ => return Err(USAGE.into()),
    };
    let (mut paths, mut fast, mut threads, mut output, mut beside) = (Vec::new(), false, 0u64, None, false);
    while let Some(w) = words.next() {
        match w.as_str() {
            "--fast" if op == "convert" => fast = true,
            "--beside" if op == "convert" => beside = true,
            "--output" if op == "convert" => {
                let folder = words.next().filter(|f| !f.starts_with('-')).ok_or_else(|| format!("chd: --output needs a folder\n{USAGE}"))?;
                output = Some(resolve(folder).map_err(|e| format!("chd convert --output: {e}"))?);
            }
            "-j" if op == "convert" => threads = words.next().and_then(|n| n.parse().ok()).filter(|&n| n > 0).ok_or_else(|| format!("chd: -j needs a number of threads above 0\n{USAGE}"))?,
            o if o.starts_with('-') && o.len() > 1 => return Err(format!("chd: {o} is not an option of chd {op}\n{USAGE}")),
            p => paths.push(Value::string(p)),
        }
    }
    if op == "convert" && paths.is_empty() {
        return Err(format!("chd convert: name what to convert, or / for the whole library\n{USAGE}"));
    }
    if beside && output.is_some() {
        return Err(format!("chd convert: --output and --beside name two places; choose one\n{USAGE}"));
    }
    let mut members = vec![("op", Value::string(op)), ("paths", Value::Array(paths))];
    if op == "convert" {
        members.extend([("fast", Value::Bool(fast)), ("threads", Value::number(threads))]);
    }
    if let Some(folder) = output {
        members.push(("output", Value::string(folder)));
    }
    if beside {
        members.push(("beside", Value::Bool(true)));
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
            "convert" if e.get("beside") == Some(&Value::Bool(true)) => Some(format!(
                "Converting {} disc(s) beside their originals, writing into the library {}: up to {} CHD file(s) added, no original changed, moved or removed, a name already taken skipped. chdman {} ({}, {} threads).",
                token(e, "discs"),
                text(e, "output"),
                token(e, "adds"),
                text(e, "chdman"),
                text(e, "codecs"),
                token(e, "threads")
            )),
            "convert" => Some(format!("Converting {} disc(s) into {} with chdman {} ({}, {} threads).", token(e, "discs"), text(e, "output"), text(e, "chdman"), text(e, "codecs"), token(e, "threads"))),
            "check" => Some(format!("Checking {} disc(s) with chdman {}.", token(e, "discs"), text(e, "chdman"))),
            _ => None,
        },
        "disc" if text(e, "state") == "working" => None,
        "disc" if e.get("index").is_none() => {
            let tail = sizes(e);
            let beside = if text(e, "where") == "library" { "  beside the original" } else { "" };
            Some(format!("{:<14}{}{}{beside}", text(e, "state"), text(e, "source"), if tail.is_empty() { String::new() } else { format!("  {tail}") }))
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

/// One package event of a batch as its line: its index, what it named and how it went.
pub fn package_line(e: &Value) -> String {
    let words = text(e, "words");
    let said = if words.is_empty() { String::new() } else { format!(": {words}") };
    let named = if e.get("source").is_some() { text(e, "source") } else { text(e, "package") };
    match text(e, "state") {
        "made" | "already" | "contained" | "zipped" => {
            let mode = match text(e, "mode") {
                "self-contained" => ", self-contained",
                "linked" => ", linked",
                _ => "",
            };
            let zip = e.get("zip").and_then(|z| z.get("path")).and_then(Value::as_str).map_or(String::new(), |z| format!(", zip Packages/{z}"));
            format!("[{}/{}] {named}: {}, Packages/{}{mode}{zip}", token(e, "index"), token(e, "of"), text(e, "state"), text(e, "package"))
        }
        "set" => format!("{named}: identity {}, the scraper sends {}", text(e, "identity"), if text(e, "identity") == "patched" { "the patched image's hashes first" } else { "the dump's" }),
        "added" | "removed" => {
            let rule = if e.get("retroarch") == Some(&Value::Bool(true)) { "named as RetroArch finds it" } else { "RetroArch will not apply this list, whose files keep their own names" };
            let n = match e.get("patches") {
                Some(Value::Array(p)) => p.len(),
                _ => 0,
            };
            format!("{named}: {} {}, {n} patch(es) now; {rule}", text(e, "state"), text(e, "patch"))
        }
        "migrated" | "nothing" => {
            let copied = match e.get("copied") {
                Some(Value::Array(c)) => c.len(),
                _ => 0,
            };
            let records = match token(e, "records").as_str() {
                "" | "0" => String::new(),
                n => format!(", {n} state record(s)"),
            };
            let unread = text(e, "records_words");
            let unread = if unread.is_empty() { String::new() } else { format!("; {unread}") };
            format!("[{}/{}] {named}: {}, {copied} file(s) copied, {} already there{records}{said}{unread}", token(e, "index"), token(e, "of"), text(e, "state"), token(e, "kept"))
        }
        state => {
            let to = text(e, "to");
            format!("[{}/{}] {named}: {state}{said}{}", token(e, "index"), token(e, "of"), if to.is_empty() { String::new() } else { format!(", now {to}") })
        }
    }
}

/// A listing's line: the state, the package, its mode and system, and why.
pub fn listing_line(e: &Value) -> String {
    let words = text(e, "words");
    let to = text(e, "to");
    let why = if !to.is_empty() { format!("  now {to}") } else if words.is_empty() { String::new() } else { format!("  {words}") };
    let patched = match e.get("patches") {
        Some(Value::Array(p)) if !p.is_empty() => format!(", {} patch(es){}", p.len(), if text(e, "identity") == "patched" { ", scraped as patched" } else { "" }),
        _ => String::new(),
    };
    format!("{:<9}{}  ({}, {}{patched}){why}", text(e, "state"), text(e, "package"), text(e, "mode"), text(e, "system"))
}

/// One job this shell started: its number, what it does, the tool's handle, and how far it has got.
struct Running {
    id: u32,
    prefix: &'static str,
    listing: bool,
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
            call.err(&format!("{}: {}\n", r.prefix, last_error(table)));
            return Followed::Ended(1);
        }
        if n as usize > buffer.len() {
            buffer.resize(n as usize, 0);
            continue;
        }
        let Ok(event) = json::parse(&String::from_utf8_lossy(&buffer[..n as usize])) else { continue };
        if let Some(Value::Array(refused)) = event.get("refused") {
            for words in refused.iter().filter_map(Value::as_str) {
                call.err(&format!("{}: {words}\n", r.prefix));
                *status = 1;
            }
        }
        if let Some(words) = event.get("error").and_then(Value::as_str) {
            call.err(&format!("{}: {words}\n", r.prefix));
            *status = 1;
        }
        match text(&event, "event") {
            "start" => r.of = number(&event, "discs").or_else(|| number(&event, "packages")).unwrap_or(0),
            "disc" if text(&event, "state") == "working" => r.current = text(&event, "source").to_string(),
            "disc" | "package" => {
                r.done += 1;
                if matches!(text(&event, "state"), "failed" | "changed" | "missing" | "refused") {
                    *status = 1;
                }
            }
            _ => {}
        }
        let printed = match (text(&event, "event"), r.listing) {
            ("package", true) => Some(listing_line(&event)),
            ("package", false) => Some(package_line(&event)),
            ("start", _) if r.prefix == "pkg" => start_line(&event),
            _ => line(&event),
        };
        if let Some(l) = printed {
            call.out(&format!("{l}\n"));
        }
    }
}

fn start_line(e: &Value) -> Option<String> {
    let n = token(e, "packages");
    match text(e, "op") {
        "pkg-make" if text(e, "mode") == "self-contained" => Some(format!("Making packages for {n} game(s) in Packages, each game copied in{}; the library is only read.", if e.get("zip") == Some(&Value::Bool(true)) { " and each package written as one zip" } else { "" })),
        "pkg-make" => Some(format!("Making packages for {n} game(s) in Packages, each linked to its game where it is.")),
        "pkg-check" => Some(format!("Checking {n} package(s) against their manifests.")),
        "pkg-migrate" => Some(format!("Copying the saves, states, records, settings, art and cheats of {n} package(s) into them.")),
        "pkg-zip" => Some(format!("Writing {n} package(s) each as one zip beside its folder.")),
        _ => None,
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
            ("start", _) => r.of = number(&event, "discs").or_else(|| number(&event, "packages")).unwrap_or(0),
            ("disc", "working") => r.current = text(&event, "source").to_string(),
            ("disc" | "package", _) => r.done += 1,
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
            let (id, p) = (jobs.running[at].id, jobs.running[at].prefix);
            call.out(&format!("{p}{id} goes on in the background: {p} wait {p}{id} follows it, ps lists it, kill {p}{id} stops it.\n"));
            status
        }
    }
}

fn start(call: &Call, jobs: &mut Jobs, prefix: &'static str) -> Answer {
    let args = call.args();
    let request = match if prefix == "pkg" { pkg_request(&args, &|p| call.resolve(p)) } else { request(&args, &|p| call.resolve(p)) } {
        Ok(r) => r,
        Err(usage) => return Answer::fail(usage),
    };
    let Some(table) = call.service(SERVICE, MAJOR).map(|t| t as *const LibraryToolV1) else { return Answer::fail(format!("{prefix}: needs emusen.library, which this program does not provide")) };
    let mut job: *mut c_void = std::ptr::null_mut();
    if unsafe { ((*table).start)(request.as_ptr(), request.len(), &mut job) } != 0 {
        return Answer::fail(format!("{prefix}: {}", last_error(unsafe { &*table })));
    }
    jobs.next += 1;
    let listing = prefix == "pkg" && args.get(1).map(String::as_str) == Some("ls");
    jobs.running.push(Running { id: jobs.next, prefix, listing, op: args[1].clone(), job, table, done: 0, of: 0, current: String::new(), stopping: false });
    let at = jobs.running.len() - 1;
    Answer::with(String::new(), follow_or_detach(call, jobs, at))
}

fn wait(call: &Call, jobs: &mut Jobs, prefix: &'static str) -> Answer {
    let args = call.args();
    let at = match args.get(2) {
        None => jobs.running.iter().rposition(|r| r.prefix == prefix),
        Some(id) => id.strip_prefix(prefix).and_then(|n| n.parse::<u32>().ok()).and_then(|n| jobs.running.iter().position(|r| r.id == n && r.prefix == prefix)),
    };
    match at {
        Some(at) => Answer::with(String::new(), follow_or_detach(call, jobs, at)),
        None => Answer::fail(format!("{prefix} wait: no such {prefix} job{}", args.get(2).map(|a| format!(" {a}")).unwrap_or_default())),
    }
}

fn jobs_of<'a>(call: &'a Call) -> &'a mut Jobs {
    call.slot(Jobs::default)
}

impl Module for LibraryCommands {
    fn manifest(&self) -> &'static str {
        static MANIFEST: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        MANIFEST.get_or_init(|| Manifest { id: "emusen.library", version: env!("CARGO_PKG_VERSION"), services: &[(SERVICE, MAJOR, 4, true)], commands: COMMANDS, aliases: &[], jobs: &["chd", "pkg"] }.json())
    }

    fn run(&self, call: &Call) -> i32 {
        call.answer(match call.command().as_str() {
            "chd" if call.args().get(1).map(String::as_str) == Some("wait") => wait(call, jobs_of(call), "chd"),
            "chd" => start(call, jobs_of(call), "chd"),
            "pkg" if call.args().get(1).map(String::as_str) == Some("wait") => wait(call, jobs_of(call), "pkg"),
            "pkg" => start(call, jobs_of(call), "pkg"),
            other => Answer::fault(format!("emusen.library has no command '{other}'")),
        })
    }

    fn jobs(&self, call: &Call) {
        let jobs = jobs_of(call);
        for r in jobs.running.iter_mut() {
            let ended = drain(r);
            let unit = if r.prefix == "pkg" { "packages" } else { "discs" };
            let progress = if ended { "ended".to_string() } else if r.stopping { "stopping".to_string() } else { format!("{}/{} {unit}", r.done, r.of) };
            call.out(&format!("  {}{:<7} {:<11} {progress}{}\n", r.prefix, r.id, r.op, if r.current.is_empty() || ended { String::new() } else { format!(", {}", r.current) }));
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
        let prefix = ["chd", "pkg"].into_iter().find(|p| id.starts_with(p))?;
        let n = id.strip_prefix(prefix)?.parse::<u32>().ok()?;
        let jobs = jobs_of(call);
        let Some(r) = jobs.running.iter_mut().find(|r| r.id == n && r.prefix == prefix) else {
            call.err(&format!("kill: no {prefix} job {prefix}{n} found.\n"));
            return Some(1);
        };
        unsafe { ((*r.table).stop)(r.job) };
        r.stopping = true;
        let what = if prefix == "pkg" { "the packages not reached are left as they are" } else { "its current disc is abandoned and nothing of it is left behind" };
        call.out(&format!("{prefix}{n} is stopping: {what}; {prefix} wait {prefix}{n} shows how far it got.\n"));
        Some(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pkg_request_plain(a: &[String]) -> Result<String, String> {
        pkg_request(a, &|p| Ok(format!("/real{p}")))
    }

    fn args(line: &str) -> Vec<String> {
        line.split_whitespace().map(str::to_string).collect()
    }

    fn request_plain(a: &[String]) -> Result<String, String> {
        request(a, &|p| Ok(format!("/host{p}")))
    }

    #[test]
    fn a_command_line_is_a_request_or_the_usage() {
        let request = request_plain;
        assert_eq!(request(&args("chd ls")).unwrap(), r#"{"op":"ls","paths":[]}"#);
        assert_eq!(request(&args("chd convert psx SCD --fast -j 4")).unwrap(), r#"{"op":"convert","paths":["psx","SCD"],"fast":true,"threads":4}"#);
        assert_eq!(request(&args("chd check psx/a.cue")).unwrap(), r#"{"op":"check","paths":["psx/a.cue"]}"#);
        assert!(request(&args("chd convert")).unwrap_err().contains("name what to convert"));
        assert!(request(&args("chd check --fast")).unwrap_err().contains("--fast is not an option of chd check"));
        assert!(request(&args("chd convert psx -j 0")).unwrap_err().contains("-j needs"));
        assert!(request(&args("chd")).unwrap_err().starts_with("Usage: chd"));
        assert_eq!(request(&args("chd convert psx --output /chds")).unwrap(), r#"{"op":"convert","paths":["psx"],"fast":false,"threads":0,"output":"/host/chds"}"#);
        assert_eq!(request(&args("chd convert psx --beside")).unwrap(), r#"{"op":"convert","paths":["psx"],"fast":false,"threads":0,"beside":true}"#);
        assert!(request(&args("chd convert psx --beside --output /chds")).unwrap_err().contains("choose one"));
        assert!(request(&args("chd convert psx --output")).unwrap_err().contains("--output needs a folder"));
        assert!(request(&args("chd ls --beside")).unwrap_err().contains("--beside is not an option of chd ls"));
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
        assert!(line(&e(r#"{"event":"start","op":"convert","discs":3,"chdman":"0.289","codecs":"cdlz,cdzl,cdfl","threads":8,"output":"/r","beside":true,"adds":2,"refused":[]}"#)).unwrap().starts_with("Converting 3 disc(s) beside their originals, writing into the library /r: up to 2 CHD file(s) added"));
        assert_eq!(line(&e(r#"{"event":"disc","source":"psx/a.cue","state":"converted","where":"library","original":1000,"chd":523}"#)).unwrap(), "converted     psx/a.cue  1,000 -> 523 bytes (52.3%)  beside the original");
    }

    #[test]
    fn the_manifest_names_chd_and_pkg_their_jobs_and_requires_the_service() {
        let m = LibraryCommands.manifest();
        assert!(m.starts_with("{\"manifest\":1,\"id\":\"emusen.library\"") && m.contains("\"name\":\"chd\"") && m.contains("\"name\":\"pkg\"") && m.contains("\"required\":true") && m.contains("\"jobs\":[\"chd\",\"pkg\"]"), "{m}");
    }

    #[test]
    fn a_pkg_command_line_is_a_request_or_the_usage() {
        assert_eq!(pkg_request_plain(&args("pkg make SNES nes/a.nes")).unwrap(), r#"{"op":"pkg-make","paths":["SNES","nes/a.nes"]}"#);
        assert_eq!(pkg_request_plain(&args("pkg ls")).unwrap(), r#"{"op":"pkg-ls","paths":[],"console":null}"#);
        assert_eq!(pkg_request_plain(&args("pkg ls SNES")).unwrap(), r#"{"op":"pkg-ls","paths":[],"console":"SNES"}"#);
        assert_eq!(pkg_request_plain(&args("pkg check")).unwrap(), r#"{"op":"pkg-check","paths":[]}"#);
        assert_eq!(pkg_request_plain(&args("pkg migrate SNES/a.sfc")).unwrap(), r#"{"op":"pkg-migrate","paths":["SNES/a.sfc"]}"#);
        assert!(pkg_request_plain(&args("pkg make")).unwrap_err().contains("name what to make"));
        assert_eq!(pkg_request_plain(&args("pkg make a.nes --self-contained")).unwrap(), r#"{"op":"pkg-make","paths":["a.nes"],"self_contained":true,"zip":false}"#);
        assert_eq!(pkg_request_plain(&args("pkg make --zip a.nes")).unwrap(), r#"{"op":"pkg-make","paths":["a.nes"],"self_contained":true,"zip":true}"#);
        assert!(pkg_request_plain(&args("pkg check --zip")).unwrap_err().contains("not an option of pkg check"));
        assert_eq!(pkg_request_plain(&args("pkg zip SNES/a.sfc")).unwrap(), r#"{"op":"pkg-zip","paths":["SNES/a.sfc"]}"#);
        assert_eq!(pkg_request_plain(&args("pkg identity SNES/a.sfc patched")).unwrap(), r#"{"op":"pkg-identity","paths":["SNES/a.sfc"],"identity":"patched"}"#);
        assert!(pkg_request_plain(&args("pkg identity SNES/a.sfc hacked")).unwrap_err().starts_with("pkg identity:"));
        assert_eq!(pkg_request_plain(&args("pkg patch add SNES/a.sfc /x.bps")).unwrap(), r#"{"op":"pkg-patch-add","paths":["SNES/a.sfc"],"patch":"/real/x.bps"}"#);
        assert_eq!(pkg_request_plain(&args("pkg patch rm SNES/a.sfc 2")).unwrap(), r#"{"op":"pkg-patch-rm","paths":["SNES/a.sfc"],"index":2}"#);
        assert!(pkg_request_plain(&args("pkg patch rm a 0")).unwrap_err().contains("not a patch's number"));
        assert!(pkg_request_plain(&args("pkg patch add a")).unwrap_err().starts_with("pkg patch:"));
        assert!(pkg_request_plain(&args("pkg ls a b")).unwrap_err().contains("one console"));
        assert!(pkg_request_plain(&args("pkg")).unwrap_err().starts_with("Usage: pkg"));
        let e = |t: &str| json::parse(t).unwrap();
        assert_eq!(package_line(&e(r#"{"event":"package","index":1,"of":2,"source":"SNES/a.sfc","state":"made","package":"SNES/a.sfc"}"#)), "[1/2] SNES/a.sfc: made, Packages/SNES/a.sfc");
        assert_eq!(package_line(&e(r#"{"event":"package","index":2,"of":2,"package":"NES/b.nes","state":"changed","words":"b.nes has SHA-1 1"}"#)), "[2/2] NES/b.nes: changed: b.nes has SHA-1 1");
        assert_eq!(package_line(&e(r#"{"event":"package","index":1,"of":1,"package":"NES/b.nes","state":"migrated","copied":[{"from":"a","to":"b"}],"kept":0}"#)), "[1/1] NES/b.nes: migrated, 1 file(s) copied, 0 already there");
        assert_eq!(
            package_line(&e(r#"{"event":"package","index":1,"of":1,"package":"NES/b.nes","state":"migrated","copied":[],"kept":1,"records":2,"records_words":"the states' records were not copied: why"}"#)),
            "[1/1] NES/b.nes: migrated, 0 file(s) copied, 1 already there, 2 state record(s); the states' records were not copied: why"
        );
        assert_eq!(listing_line(&e(r#"{"event":"package","package":"NES/b.nes","state":"moved","mode":"linked","system":"nes","to":"NES/USA/b.nes"}"#)), "moved    NES/b.nes  (linked, nes)  now NES/USA/b.nes");
    }
}
