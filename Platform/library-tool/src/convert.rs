//! One disc converted, listed or checked: EmuSen_GamePackages.md §4.3's three checks, §4.4's linked package, and
//! §4.6's rules — a partial file renamed into place only after every check, free space and an unchanged source
//! required, and a disc done when its package names a CHD still at the size and time recorded with its SHA-1.

use crate::chdman::Chdman;
use crate::checks::{self, Toc};
use crate::discs::{self, Item};
use crate::Context;
use emusen_dotnet::date;
use emusen_galaxia::dotnet_path::{self as path, Style};
use emusen_galaxia::json::{Text, Value};
use emusen_galaxia::package;
use emusen_galaxia::rom_hash::{self, Hashes};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// `createcd`'s codecs for *Faster to decode* (§4.8, Q7); otherwise chdman's own defaults, `cdlz,cdzl,cdfl`.
pub const FAST_CODECS: &str = "cdzs,cdfl";
/// Space kept free at the output beyond the original's size (§4.6).
pub const MARGIN: u64 = 64 << 20;
/// A partial file or scratch folder older than this was left by a process that stopped (§4.6).
pub const PARTIAL_AGE: std::time::Duration = std::time::Duration::from_secs(3600);

#[derive(Clone, Copy, Debug, Default)]
pub struct Options {
    pub fast: bool,
    pub threads: usize,
}

/// What became of one disc: its state word, the words that explain it, and the facts an event carries.
pub struct Outcome {
    pub state: &'static str,
    pub words: Option<String>,
    pub facts: Vec<(Text, Value)>,
}

impl Outcome {
    fn new(state: &'static str) -> Outcome {
        Outcome { state, words: None, facts: Vec::new() }
    }

    fn said(state: &'static str, words: impl Into<String>) -> Outcome {
        Outcome { state, words: Some(words.into()), facts: Vec::new() }
    }

    fn with(mut self, name: &str, value: Value) -> Outcome {
        self.facts.push((Text::new(name), value));
        self
    }
}

pub fn obj(members: Vec<(&str, Value)>) -> Value {
    Value::Object(members.into_iter().map(|(n, v)| (Text::new(n), v)).collect())
}

fn text(s: &str) -> Value {
    Value::string(s)
}

/// Where a disc's CHD and package go: the CHD at the library's relative path under `Compressed`, the package under
/// `Packages/<console>/` at the same path less the library's console folder (§3.1, §4.2).
pub struct Placed {
    pub chd: String,
    pub chd_rel: String,
    pub package: String,
    pub package_rel: String,
}

fn with_chd(rel: &str) -> String {
    let (dir, name) = rel.rsplit_once('/').map_or(("", rel), |(d, n)| (d, n));
    let stem = path::file_name_without_extension(Style::Unix, name);
    if dir.is_empty() { format!("{stem}.chd") } else { format!("{dir}/{stem}.chd") }
}

fn under(root: &str, rel: &str) -> String {
    rel.split('/').fold(root.to_string(), |at, part| path::combine(Style::HOST, &at, part))
}

pub fn place(ctx: &Context, item: &Item) -> Placed {
    let chd_rel = with_chd(&item.rel);
    let rest = item.rel.split_once('/').map_or(item.rel.as_str(), |(_, r)| r);
    let package_rel = format!("{}/{}", item.system.console, with_chd(rest));
    Placed { chd: under(&ctx.compressed(), &chd_rel), chd_rel, package: under(&ctx.packages(), &package_rel), package_rel }
}

/// A file's size and modification time, the time to the nanosecond and as the manifest writes it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Seen {
    pub size: u64,
    pub modified: SystemTime,
}

pub fn seen(at: &str) -> Option<Seen> {
    let meta = std::fs::metadata(at).ok()?;
    Some(Seen { size: meta.len(), modified: meta.modified().ok()? })
}

/// A time as the manifest's `mtime`: UTC to the second, `yyyy-MM-ddTHH:mm:ssZ`.
pub fn utc(t: SystemTime) -> String {
    let secs = t.duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
    let (y, m, d) = date::civil(secs.div_euclid(86_400));
    let s = secs.rem_euclid(86_400);
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z", s / 3600, s / 60 % 60, s % 60)
}

fn seen_json(s: &Seen) -> Value {
    obj(vec![("size", Value::number(s.size)), ("mtime", text(&utc(s.modified)))])
}

fn same_seen(v: Option<&Value>, s: &Option<Seen>) -> bool {
    match (v, s) {
        (Some(v), Some(s)) => v.get("size").and_then(number) == Some(s.size) && v.get("mtime").and_then(Value::as_str) == Some(utc(s.modified).as_str()),
        _ => false,
    }
}

fn number(v: &Value) -> Option<u64> {
    match v {
        Value::Number(t) => t.parse().ok(),
        _ => None,
    }
}

fn hashes_json(h: &Hashes, extra: Vec<(&str, Value)>) -> Value {
    let mut members = extra;
    members.extend([("size", Value::number(h.size)), ("crc32", text(&h.crc32)), ("md5", text(&h.md5)), ("sha1", h.sha1.as_deref().map_or(Value::Null, text))]);
    obj(members)
}

fn hashes_from(v: &Value) -> Option<Hashes> {
    Some(Hashes { md5: v.get("md5")?.as_str()?.to_lowercase(), crc32: v.get("crc32")?.as_str()?.to_lowercase(), sha1: Some(v.get("sha1")?.as_str()?.to_lowercase()), size: v.get("size").and_then(number)? })
}

/// The package's manifest, refused words where it is absent or not one this build reads.
pub fn manifest(placed: &Placed) -> Result<Value, String> {
    let at = path::combine(Style::HOST, &placed.package, package::FILE_NAME);
    match package::read(&at) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err("not converted".into()),
        Err(e) => Err(format!("{at}: {e}")),
        Ok(Err(refusal)) => Err(format!("{at}: {}", refusal.words())),
        Ok(Ok(doc)) => Ok(doc),
    }
}

/// Whether a disc is done: its package names this CHD at the size and time recorded, and the library files it was
/// converted from still have the size and time they had.
pub fn done(ctx: &Context, placed: &Placed, doc: &Value) -> bool {
    let game = doc.get("game");
    let container = doc.get("container");
    let names_it = game.and_then(|g| g.get("path")).and_then(Value::as_str) == Some(placed.chd_rel.as_str());
    let has_sha1 = container.and_then(|c| c.get("sha1")).and_then(Value::as_str).is_some();
    let chd_same = same_seen(game.and_then(|g| g.get("seen")), &seen(&placed.chd));
    names_it && has_sha1 && chd_same && sources_same(ctx, doc)
}

fn sources_same(ctx: &Context, doc: &Value) -> bool {
    let library = ctx.library.clone().unwrap_or_default();
    let Some(Value::Array(sources)) = doc.get("container").and_then(|c| c.get("sources")) else { return false };
    !sources.is_empty() && sources.iter().all(|s| s.get("path").and_then(Value::as_str).is_some_and(|p| same_seen(Some(s), &seen(&under(&library, p)))))
}

#[cfg(unix)]
fn free_bytes(dir: &str) -> Option<u64> {
    let c = std::ffi::CString::new(dir).ok()?;
    let mut s: libc::statvfs = unsafe { std::mem::zeroed() };
    // SAFETY: a NUL-terminated path and a zeroed struct the call fills.
    (unsafe { libc::statvfs(c.as_ptr(), &mut s) } == 0).then(|| s.f_bavail as u64 * s.f_frsize as u64)
}

#[cfg(not(unix))]
fn free_bytes(_: &str) -> Option<u64> {
    None
}

/// Partial files and scratch folders under `Compressed` that a stopped process left more than an hour ago, removed.
pub fn sweep(ctx: &Context) {
    let mut found = Vec::new();
    let _ = emusen_galaxia::dotnet_files::visit_files_with(&ctx.compressed(), emusen_galaxia::dotnet_files::Options { skip_hidden: false, ignore_inaccessible: true }, |f, name| {
        if name.contains(".partial-") {
            found.push(f.to_string());
        }
    });
    let old = |at: &str| std::fs::metadata(at).and_then(|m| m.modified()).ok().and_then(|t| t.elapsed().ok()).is_some_and(|age| age > PARTIAL_AGE);
    for f in found.iter().filter(|f| old(f)) {
        let _ = std::fs::remove_file(f);
    }
    if let Ok(entries) = std::fs::read_dir(ctx.compressed()) {
        for e in entries.flatten() {
            let at = e.path().to_string_lossy().into_owned();
            if e.file_name().to_string_lossy().starts_with(".scratch-") && old(&at) {
                let _ = std::fs::remove_dir_all(&at);
            }
        }
    }
}

fn scratch_for(ctx: &Context, index: usize) -> String {
    path::combine(Style::HOST, &ctx.compressed(), &format!(".scratch-{}-{index}", std::process::id()))
}

/// A check's word for the event and the log.
fn word(r: &Result<(), String>) -> Value {
    match r {
        Ok(()) => text("passed"),
        Err(w) => text(&format!("failed: {w}")),
    }
}

/// Checks 3 and 2, in that order, through EmuSen's reader, whatever check 1 found: the reader refuses a damaged hunk in
/// words rather than stopping the program (EmuSen_Disc.md §13, EmuSen_GamePackages.md §12.6).
fn reader_checks(at: &str, toc: &Toc, sheet: &str, expected: &[Hashes], stop: &dyn Fn() -> bool) -> (Result<(), String>, Result<(), String>) {
    match checks::chd(at) {
        Err(w) => (Err(w.clone()), Err(w)),
        Ok(mut disc) => {
            let t = checks::toc(&disc, toc);
            let k = if t.is_ok() { checks::tracks(&mut disc, sheet, expected, stop) } else { Err("not run, since the table of contents differs".into()) };
            (t, k)
        }
    }
}

/// Converts one disc. `stop` ends chdman and leaves nothing behind.
pub fn convert(ctx: &Context, chdman: &Chdman, item: &Item, index: usize, options: Options, stop: &dyn Fn() -> bool) -> Outcome {
    let placed = place(ctx, item);
    if manifest(&placed).is_ok_and(|doc| done(ctx, &placed, &doc)) {
        return Outcome::new("already").with("output", text(&placed.chd_rel)).with("package", text(&placed.package_rel));
    }
    let scratch = scratch_for(ctx, index);
    let outcome = convert_in(ctx, chdman, item, &placed, &scratch, options, stop);
    let _ = std::fs::remove_dir_all(&scratch);
    outcome
}

fn convert_in(ctx: &Context, chdman: &Chdman, item: &Item, placed: &Placed, scratch: &str, options: Options, stop: &dyn Fn() -> bool) -> Outcome {
    let started = Instant::now();
    let prepared = match discs::prepare(&ctx.tree, item, scratch) {
        Ok(p) => p,
        Err(words) => return Outcome::said("skipped", words),
    };
    let before: Vec<Option<Seen>> = prepared.sources.iter().map(|s| seen(s)).collect();
    let tracks = match rom_hash::tracks(&prepared.scratch_sheet, stop) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::Interrupted => return Outcome::new("stopped"),
        Err(e) => return Outcome::said("failed", format!("the original could not be hashed: {e}")),
    };
    let original_toc = match checks::original(&prepared.sheet, &prepared.files) {
        Ok(disc) => checks::toc_of(&disc),
        Err(words) => return Outcome::said("skipped", format!("the original does not read as a disc: {words}")),
    };
    let original_bytes: u64 = prepared.files.iter().filter_map(|f| seen(f)).map(|s| s.size).sum();
    let out_dir = path::directory_name(Style::HOST, &placed.chd).unwrap_or_default();
    if let Err(e) = std::fs::create_dir_all(&out_dir) {
        return Outcome::said("failed", format!("{out_dir} could not be made: {e}"));
    }
    if let Some(free) = free_bytes(&out_dir).filter(|&f| f < original_bytes + MARGIN) {
        return Outcome::said("failed", format!("{free} bytes are free at {out_dir}, and the disc needs {} with the margin", original_bytes + MARGIN));
    }
    let partial = format!("{}.partial-{}", placed.chd, std::process::id());
    let codecs = options.fast.then_some(FAST_CODECS);
    let remove = || {
        let _ = std::fs::remove_file(&partial);
    };
    match chdman.create_cd(&prepared.scratch_sheet, &partial, codecs, options.threads, stop) {
        Ok(()) => {}
        Err(w) if w == "stopped" => {
            remove();
            return Outcome::new("stopped");
        }
        Err(w) => {
            remove();
            return Outcome::said("failed", w);
        }
    }
    let check1 = chdman.verify(&partial, stop);
    let expected: Vec<Hashes> = tracks.iter().map(|t| t.hashes.clone()).collect();
    let (check3, check2) = reader_checks(&partial, &original_toc, &prepared.sheet, &expected, stop);
    if stop() || [&check1, &check2, &check3].iter().any(|c| c.as_ref().is_err_and(|w| w == "stopped")) {
        remove();
        return Outcome::new("stopped");
    }
    let checks_json = obj(vec![("chdman", word(&check1)), ("tracks", word(&check2)), ("toc", word(&check3))]);
    let after: Vec<Option<Seen>> = prepared.sources.iter().map(|s| seen(s)).collect();
    let failure = if check1.is_err() || check2.is_err() || check3.is_err() {
        Some("a check failed".to_string())
    } else if after != before {
        Some("the original changed during the conversion, so the result was discarded".to_string())
    } else {
        None
    };
    let info = chdman.info(&partial);
    let failure = failure.or_else(|| info.as_ref().err().cloned());
    if let Some(words) = failure {
        remove();
        return Outcome::said("failed", words).with("checks", checks_json);
    }
    let info = info.expect("checked above");
    if let Err(e) = std::fs::rename(&partial, &placed.chd) {
        remove();
        return Outcome::said("failed", format!("the CHD could not be put in place: {e}"));
    }
    let chd_seen = seen(&placed.chd);
    let doc = package_of(ctx, item, placed, &prepared, &tracks, &original_toc, chdman, &info, codecs, chd_seen.as_ref(), &before, &checks_json);
    let manifest_at = path::combine(Style::HOST, &placed.package, package::FILE_NAME);
    if let Some((name, bytes)) = &prepared.original {
        if let Err(w) = emusen_galaxia::atomic::write(&path::combine(Style::HOST, &placed.package, name), bytes) {
            return Outcome::said("failed", w);
        }
    }
    match package::write(&manifest_at, &doc) {
        Ok(Ok(())) => {}
        Ok(Err(w)) => return Outcome::said("failed", w),
        Err(refusal) => return Outcome::said("failed", refusal.words().to_string()),
    }
    let chd_bytes = chd_seen.map_or(0, |s| s.size);
    Outcome::new("converted")
        .with("output", text(&placed.chd_rel))
        .with("package", text(&placed.package_rel))
        .with("original", Value::number(original_bytes))
        .with("chd", Value::number(chd_bytes))
        .with("seconds", Value::number(format!("{:.1}", started.elapsed().as_secs_f64())))
        .with("checks", checks_json)
}

#[allow(clippy::too_many_arguments)]
fn package_of(
    ctx: &Context,
    item: &Item,
    placed: &Placed,
    prepared: &discs::Prepared,
    tracks: &[rom_hash::TrackHashes],
    toc: &Toc,
    chdman: &Chdman,
    info: &crate::chdman::Info,
    codecs: Option<&str>,
    chd_seen: Option<&Seen>,
    before: &[Option<Seen>],
    checks_json: &Value,
) -> Value {
    let library = ctx.library.clone().unwrap_or_default();
    let title = path::file_name_without_extension(Style::Unix, &item.rel).to_string();
    let mut game = package::file_entry("image", "compressed", &placed.chd_rel, chd_seen.map(|s| s.size));
    if let (Value::Object(members), Some(s)) = (&mut game, chd_seen) {
        members.push((Text::new("seen"), seen_json(s)));
    }
    let lone = prepared.original.is_none();
    let dump = obj(vec![
        ("file", if lone { tracks.first().map_or(Value::Null, |t| hashes_json(&t.hashes, vec![])) } else { Value::Null }),
        ("transformed", Value::Null),
        ("tracks", Value::Array(tracks.iter().map(|t| hashes_json(&t.hashes, vec![("file", text(&t.file))])).collect())),
        ("sheet", prepared.original.as_ref().map_or(Value::Null, |(name, bytes)| hashes_json(&rom_hash::hashes_of(bytes), vec![("file", text(name))]))),
        ("form", text(prepared.form.word())),
    ]);
    let sources = prepared
        .sources
        .iter()
        .zip(before)
        .map(|(s, seen)| {
            let mut m = vec![("where", text("library")), ("path", text(&discs::rel_of(&library, s)))];
            if let Some(seen) = seen {
                m.extend([("size", Value::number(seen.size)), ("mtime", text(&utc(seen.modified)))]);
            }
            obj(m)
        })
        .collect();
    let container = obj(vec![
        ("type", text("chd")),
        ("program", text("chdman")),
        ("version", text(&chdman.version)),
        ("sha256", text(&chdman.sha256)),
        ("codecs", text(codecs.unwrap_or("cdlz,cdzl,cdfl"))),
        ("compression", text(&info.compression)),
        ("sha1", text(&info.sha1)),
        ("data_sha1", text(&info.data_sha1)),
        ("source", text(&item.rel)),
        ("sources", Value::Array(sources)),
        ("sheet", text(&prepared.sheet)),
        ("toc", toc.to_json()),
        ("checks", checks_json.clone()),
        ("converted", text(&utc(SystemTime::now()))),
    ]);
    let mut doc = package::new(Some(&title), &item.system.esde_system, "linked", game);
    if let Value::Object(members) = &mut doc {
        for (name, value) in members.iter_mut() {
            match name.text.as_str() {
                "dump" => *value = dump.clone(),
                "container" => *value = container.clone(),
                _ => {}
            }
        }
    }
    doc
}

/// `chd ls`'s view of one disc: converted, changed since, its CHD gone, or not converted.
pub fn list(ctx: &Context, item: &Item) -> Outcome {
    let placed = place(ctx, item);
    let size = seen(&item.path).map_or(0, |s| s.size);
    let base = |state| Outcome::new(state).with("size", Value::number(size));
    let Ok(doc) = manifest(&placed) else { return base("not converted") };
    let chd = seen(&placed.chd);
    let outcome = if chd.is_none() {
        base("missing")
    } else if !sources_same(ctx, &doc) {
        base("changed")
    } else if done(ctx, &placed, &doc) {
        base("converted")
    } else {
        base("changed")
    };
    let original: u64 = match doc.get("dump").and_then(|d| d.get("tracks")) {
        Some(Value::Array(tracks)) => tracks.iter().filter_map(|t| t.get("size").and_then(number)).sum(),
        _ => 0,
    };
    outcome.with("output", text(&placed.chd_rel)).with("original", Value::number(original)).with("chd", Value::number(chd.map_or(0, |s| s.size)))
}

/// `chd check`'s three checks of one converted disc against its package, the original not read.
pub fn check(ctx: &Context, chdman: &Chdman, item: &Item, stop: &dyn Fn() -> bool) -> Outcome {
    let placed = place(ctx, item);
    let doc = match manifest(&placed) {
        Ok(doc) => doc,
        Err(words) if words == "not converted" => return Outcome::new("not converted"),
        Err(words) => return Outcome::said("failed", words),
    };
    let Some(container) = doc.get("container") else { return Outcome::said("failed", "the package names no container") };
    let at = under(&ctx.compressed(), doc.get("game").and_then(|g| g.get("path")).and_then(Value::as_str).unwrap_or(""));
    if seen(&at).is_none() {
        return Outcome::said("failed", format!("the CHD {} is not there", placed.chd_rel));
    }
    let toc = container.get("toc").and_then(Toc::from_json);
    let sheet = container.get("sheet").and_then(Value::as_str);
    let expected: Option<Vec<Hashes>> = match doc.get("dump").and_then(|d| d.get("tracks")) {
        Some(Value::Array(t)) => t.iter().map(hashes_from).collect(),
        _ => None,
    };
    let (Some(toc), Some(sheet), Some(expected)) = (toc, sheet, expected) else { return Outcome::said("failed", "the package does not record the table of contents, the sheet and every track's hashes") };
    let verified = chdman.verify(&at, stop);
    let (check3, check2) = reader_checks(&at, &toc, sheet, &expected, stop);
    let check1 = verified.and_then(|()| match chdman.info(&at) {
        Ok(info) if Some(info.sha1.as_str()) == container.get("sha1").and_then(Value::as_str) => Ok(()),
        Ok(info) => Err(format!("the CHD's SHA-1 is {} and the package records {}", info.sha1, container.get("sha1").and_then(Value::as_str).unwrap_or(""))),
        Err(w) => Err(w),
    });
    if stop() {
        return Outcome::new("stopped");
    }
    let checks_json = obj(vec![("chdman", word(&check1)), ("tracks", word(&check2)), ("toc", word(&check3))]);
    let state = if check1.is_ok() && check2.is_ok() && check3.is_ok() { "verified" } else { "failed" };
    Outcome::new(state).with("output", text(&placed.chd_rel)).with("checks", checks_json)
}
