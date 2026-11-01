//! `rom hash`, `rom check` and `rom dups` as the tool's jobs (EmuSen_LibraryTools.md §4, §7): the library's games
//! under the paths named, each hashed as dumped or in its system's comparable form, held to the structural checks a
//! dump can be held to without a database, and grouped where two are the same dump. The library is only read.

use crate::chdman::Chdman;
use crate::checks::{Bytes, ISO_FORM};
use crate::convert::obj;
use crate::discs::{rel_of, relative};
use crate::job::{Op, Request};
use crate::{packages, Context};
use emusen_disc::cue::{FileKind, Sheet};
use emusen_disc::{Kind, Lone, Medium};
use emusen_dotnet::text::{compare_ignore_case, ends_with_ignore_case, equals_ignore_case};
use emusen_galaxia::dotnet_files::{self, Options};
use emusen_galaxia::dotnet_path::{self as path, Style};
use emusen_galaxia::json::Value;
use emusen_galaxia::mapped::Mapped;
use emusen_galaxia::rom_hash::{self, Hasher, Hashes};
use emusen_galaxia::{archive, claims, image_sets};
use emusen_systems::{openvgdb, System};
use std::io::Read;

/// A file of the library a job reads: its path relative to the library, its full path, and the system it routes to.
#[derive(Clone, Debug)]
pub struct File {
    pub rel: String,
    pub path: String,
    pub system: Option<&'static System>,
}

/// One line of `rom hash`: the hashes, and what they are of (a file, or `<sheet>#<track>`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    pub hashes: Hashes,
    pub name: String,
}

/// A check a file failed: its name (`size`, `sheet`, `archive`, `chd`) and why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Failed {
    pub check: &'static str,
    pub words: String,
}

const STOPPED: &str = "stopped";

fn takes(system: Option<&System>, extension: &str) -> bool {
    system.is_some_and(|s| s.extensions.iter().any(|x| equals_ignore_case(x, extension)))
}

/// The files under each named path, shelved as the library's scan shelves them: a folder only with `recursive`, and
/// then only the files some system takes; a file named directly whatever it is. Each `.m3u` is followed by the discs
/// it lists that are there. In order of relative path ignoring case; the words for each path refused.
pub fn files(library: &str, given: &[String], recursive: bool, console: Option<&System>) -> (Vec<File>, Vec<String>) {
    let (mut walked, mut direct, mut refused) = (Vec::new(), Vec::new(), Vec::new());
    for g in given {
        let rel = match relative(g) {
            Ok(rel) => rel,
            Err(words) => {
                refused.push(words);
                continue;
            }
        };
        let at = claims::under(library, &rel);
        if dotnet_files::directory_exists(&at) {
            if !recursive {
                refused.push(format!("{}: a folder; -r takes what is under it", if g.is_empty() { "/" } else { g }));
                continue;
            }
            let _ = dotnet_files::visit_files_with(&at, Options { skip_hidden: true, ignore_inaccessible: true }, |f, _| walked.push(f.to_string()));
        } else if dotnet_files::file_exists(&at) {
            direct.push(at);
        } else {
            refused.push(format!("{g}: no such file or folder in the library"));
        }
    }
    let systems = packages::every_system();
    let extensions = systems.extensions();
    let mut all: Vec<String> = walked.iter().chain(direct.iter()).cloned().collect();
    all.sort();
    all.dedup();
    let mut found: Vec<File> = Vec::new();
    for at in image_sets::shelved(all) {
        if !direct.contains(&at) && !extensions.iter().any(|e| equals_ignore_case(e, path::extension(Style::HOST, &at))) {
            continue;
        }
        let rel = rel_of(library, &at);
        let system = emusen_library::scan::route(&at, &systems).and_then(|id| emusen_systems::find(&id)).or_else(|| packages::by_folder(&rel));
        if console.is_some_and(|c| system.is_none_or(|s| s.id != c.id)) {
            continue;
        }
        found.push(File { rel, path: at, system });
    }
    found.sort_by(|a, b| compare_ignore_case(&a.rel, &b.rel).then_with(|| a.rel.cmp(&b.rel)));
    let mut out: Vec<File> = Vec::with_capacity(found.len());
    for f in found {
        let discs = if image_sets::is_playlist(&f.path) { image_sets::discs(&f.path).unwrap_or_default() } else { Vec::new() };
        let system = f.system;
        out.push(f);
        for d in discs.into_iter().filter(|d| dotnet_files::file_exists(d)) {
            if !out.iter().any(|o| o.path == d) {
                out.push(File { rel: rel_of(library, &d), path: d, system });
            }
        }
    }
    (out, refused)
}

fn io_words(at: &str, e: std::io::Error) -> String {
    if e.kind() == std::io::ErrorKind::Interrupted { STOPPED.into() } else { format!("{}: {e}", path::file_name(Style::HOST, at)) }
}

/// Whether a file is read as a disc for its comparable form: a sheet, a CHD, an `.iso`, or a raw data track of a
/// system that takes sheets.
fn is_disc(f: &File) -> bool {
    let ext = path::extension(Style::HOST, &f.path);
    if image_sets::is_sheet(&f.path) || equals_ignore_case(ext, ".chd") || equals_ignore_case(ext, ".iso") {
        return true;
    }
    equals_ignore_case(ext, ".bin") && takes(f.system, ".cue") && emusen_galaxia::image_probe::head_of(&f.path, 16).is_ok_and(|h| h.starts_with(&emusen_disc::ecma130::SYNC))
}

/// The disc a sheet, CHD or lone image makes, its files mapped; 2,048-byte sectors a compact disc where the system
/// takes sheets, else a DVD.
fn medium(f: &File) -> Result<Medium<Bytes>, String> {
    let image = Mapped::open(&f.path).map_err(|e| io_words(&f.path, e))?;
    let companions = if image_sets::is_sheet(&f.path) {
        image_sets::companions(&f.path)?.iter().map(|c| Mapped::open(c).map(Bytes).map_err(|e| io_words(c, e))).collect::<Result<Vec<_>, _>>()?
    } else {
        Vec::new()
    };
    let user = if takes(f.system, ".cue") { Kind::Cd } else { Kind::Dvd };
    emusen_disc::open_medium(Bytes(image), companions, ISO_FORM, Lone::Detect, user).map_err(|e| e.to_string())
}

/// Every sector of a disc in order, to `take`: a compact disc's raw 2,352 bytes from track 1's index 01 to the
/// lead-out, a DVD's 2,048 user bytes; the first damaged sector's words, or a stop, end it.
fn walk(medium: &mut Medium<Bytes>, stop: &dyn Fn() -> bool, take: &mut dyn FnMut(&[u8])) -> Result<(), String> {
    match medium {
        Medium::Cd(d) => {
            let from = d.tracks().first().map_or(0, |t| t.start);
            let mut raw = [0u8; emusen_disc::ecma130::RAW];
            for at in from..d.lead_out() {
                if (at - from) % 4096 == 0 && stop() {
                    return Err(STOPPED.into());
                }
                d.read(at, &mut raw);
                take(&raw);
            }
        }
        Medium::Dvd(d) => {
            let mut user = [0u8; emusen_disc::ecma267::USER];
            for lba in 0..d.sectors() {
                if lba % 4096 == 0 && stop() {
                    return Err(STOPPED.into());
                }
                d.read_user(lba, &mut user);
                take(&user);
            }
        }
    }
    medium.damage().map_or(Ok(()), |w| Err(w.to_string()))
}

struct HashWriter(Hasher);

impl std::io::Write for HashWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.update(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// The comparable form of bytes held in memory: the system pack's transform taken off, where it changes them.
fn transformed(system: Option<&System>, bytes: &[u8]) -> Hashes {
    system.and_then(|s| rom_hash::transformed(bytes, |b| openvgdb::bytes(&s.id, b))).unwrap_or_else(|| rom_hash::hashes_of(bytes))
}

/// A zip's comparable form: the entry play opens, checked against its CRC-32, in its system's comparable form.
fn entry(f: &File) -> Result<Hashes, String> {
    let name = archive::image_entry(&f.path, |e| f.system.is_none_or(|s| s.extensions.iter().any(|x| equals_ignore_case(x, e))))?.ok_or("the archive holds no image of its system")?;
    let entries = archive::entries_of(&f.path)?;
    let e = entries.iter().find(|e| e.full_name == name).ok_or_else(|| format!("the archive holds no {name}"))?;
    if f.system.is_some_and(|s| openvgdb::transforms(&s.id)) {
        let mut bytes = Vec::with_capacity(e.length.min(1 << 30) as usize);
        archive::copy_entry(&f.path, e, &mut bytes)?;
        return Ok(transformed(f.system, &bytes));
    }
    let mut w = HashWriter(Hasher::new(true));
    archive::copy_entry(&f.path, e, &mut w)?;
    Ok(w.0.finish())
}

/// A file's comparable form (EmuSen_LibraryTools.md §3, §7.2): a disc's sectors as `emusen-disc` reads them, a zip's
/// image, or the file with its system's header taken off.
pub fn comparable(f: &File, stop: &dyn Fn() -> bool) -> Result<Hashes, String> {
    if is_disc(f) {
        let mut m = medium(f)?;
        let mut h = Hasher::new(true);
        walk(&mut m, stop, &mut |b| h.update(b))?;
        return Ok(h.finish());
    }
    if archive::is_archive(&f.path) {
        return entry(f);
    }
    if f.system.is_some_and(|s| openvgdb::transforms(&s.id)) {
        let bytes = std::fs::read(&f.path).map_err(|e| io_words(&f.path, e))?;
        return Ok(transformed(f.system, &bytes));
    }
    rom_hash::hashes_with(&f.path, None, true, stop).map_err(|e| io_words(&f.path, e))
}

/// `rom hash`'s lines for one file: a sheet's tracks as dumped, each `<sheet>#<nn>`, or the file; with `comparable`
/// one line of its comparable form. An `.m3u` has none: its discs follow it.
pub fn hash(f: &File, comparable_form: bool, stop: &dyn Fn() -> bool) -> Result<Vec<Line>, String> {
    if image_sets::is_playlist(&f.path) {
        return Ok(Vec::new());
    }
    if comparable_form {
        return Ok(vec![Line { hashes: comparable(f, stop)?, name: f.rel.clone() }]);
    }
    if image_sets::is_sheet(&f.path) {
        let tracks = rom_hash::tracks(&f.path, stop).map_err(|e| io_words(&f.path, e))?;
        return Ok(tracks.into_iter().map(|t| Line { hashes: t.hashes, name: format!("{}#{:02}", f.rel, t.number) }).collect());
    }
    Ok(vec![Line { hashes: rom_hash::hashes_with(&f.path, None, true, stop).map_err(|e| io_words(&f.path, e))?, name: f.rel.clone() }])
}

fn failed(check: &'static str, words: impl Into<String>) -> Failed {
    Failed { check, words: words.into() }
}

/// The size check: a file that will not open or read through, or holds no byte; its size.
fn read_through(at: &str, stop: &dyn Fn() -> bool) -> Result<u64, Failed> {
    let name = path::file_name(Style::HOST, at).to_string();
    let mut file = std::fs::File::open(at).map_err(|e| failed("size", format!("{name} is unreadable: {e}")))?;
    let (mut total, mut buffer) = (0u64, vec![0u8; 1 << 20]);
    loop {
        if stop() {
            return Err(failed("size", STOPPED));
        }
        match file.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => total += n as u64,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(failed("size", format!("{name} is unreadable after {total} bytes: {e}"))),
        }
    }
    if total == 0 {
        return Err(failed("size", format!("{name} is empty")));
    }
    Ok(total)
}

fn grouped(n: u64) -> String {
    let digits = n.to_string();
    digits.chars().enumerate().fold(String::new(), |mut out, (i, c)| {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
        out
    })
}

/// A sheet's checks: it parses, every file it names is there and reads, each track ends on a whole sector, and the
/// disc opens through `emusen-disc`.
fn check_sheet(f: &File, stop: &dyn Fn() -> bool, out: &mut Vec<Failed>) {
    let text = match std::fs::read(&f.path) {
        Ok(b) => emusen_dotnet::file::decode(&b),
        Err(e) => return out.push(failed("size", format!("{} is unreadable: {e}", path::file_name(Style::HOST, &f.path)))),
    };
    let sheet = match Sheet::parse(&text) {
        Ok(s) => s,
        Err(e) => return out.push(failed("sheet", format!("it does not parse: {e}"))),
    };
    let files = match image_sets::companions(&f.path) {
        Ok(c) => c,
        Err(words) => return out.push(failed("sheet", words)),
    };
    let mut sizes = Vec::with_capacity(files.len());
    for (name, at) in sheet.files.iter().zip(&files) {
        if !dotnet_files::file_exists(at) {
            out.push(failed("sheet", format!("it names {name}, which is not there")));
            sizes.push(None);
            continue;
        }
        match read_through(at, stop) {
            Ok(n) => sizes.push(Some(n)),
            Err(x) => {
                out.push(x);
                sizes.push(None);
            }
        }
    }
    if !out.is_empty() {
        return;
    }
    let spans = match emusen_disc::set::track_spans(&text) {
        Ok(s) => s,
        Err(words) => return out.push(failed("sheet", words)),
    };
    for (t, span) in sheet.tracks.iter().zip(&spans) {
        if sheet.kinds[span.file] == FileKind::Wave {
            continue;
        }
        let Some(size) = sizes[span.file] else { continue };
        let length = span.length.unwrap_or_else(|| size.saturating_sub(span.offset));
        let sector = t.mode.bytes() as u64;
        if span.offset > size || length % sector != 0 {
            out.push(failed("sheet", format!("track {:02} is {} bytes, not a whole number of {}-byte sectors", t.number, grouped(length), grouped(sector))));
        }
    }
    if out.is_empty() {
        if let Err(words) = medium(f) {
            out.push(failed("sheet", words));
        }
    }
}

/// A zip's check: its directory reads, it holds a file, and every entry inflates to its recorded length and CRC-32.
fn check_archive(at: &str, stop: &dyn Fn() -> bool, out: &mut Vec<Failed>) {
    let entries = match archive::entries_of(at) {
        Ok(e) => e,
        Err(words) => return out.push(failed("archive", format!("its directory does not read: {words}"))),
    };
    let files: Vec<_> = entries.iter().filter(|e| !e.full_name.ends_with('/')).collect();
    if files.is_empty() {
        return out.push(failed("archive", "it holds no file"));
    }
    for e in files {
        if stop() {
            return out.push(failed("archive", STOPPED));
        }
        if let Err(words) = archive::copy_entry(at, e, &mut std::io::sink()) {
            out.push(failed("archive", words));
        }
    }
}

/// A CHD's check: chdman's verify of the whole where chdman is there, and then, unless it failed, every sector read
/// through EmuSen's reader, each hunk against the map's CRC.
fn check_chd(f: &File, chdman: Option<&Chdman>, stop: &dyn Fn() -> bool, out: &mut Vec<Failed>) {
    if let Some(c) = chdman {
        if let Err(words) = c.verify(&f.path, stop) {
            return out.push(failed("chd", if words == STOPPED { STOPPED.to_string() } else { words }));
        }
    }
    match medium(f) {
        Ok(mut m) => {
            if let Err(words) = walk(&mut m, stop, &mut |_| {}) {
                out.push(failed("chd", words));
            }
        }
        Err(words) => out.push(failed("chd", words)),
    }
}

/// `rom check` of one file (EmuSen_LibraryTools.md §3, §7.3): each check it failed, none where it is sound; a stop
/// is a failure whose words are `stopped`.
pub fn check(f: &File, chdman: Option<&Chdman>, stop: &dyn Fn() -> bool) -> Vec<Failed> {
    let mut out = Vec::new();
    let ext = path::extension(Style::HOST, &f.path);
    if image_sets::is_sheet(&f.path) {
        check_sheet(f, stop, &mut out);
    } else if image_sets::is_playlist(&f.path) {
        match image_sets::discs(&f.path) {
            Ok(discs) if discs.is_empty() => out.push(failed("sheet", "it lists no disc")),
            Ok(discs) => out.extend(discs.iter().filter(|d| !dotnet_files::file_exists(d)).map(|d| failed("sheet", format!("it lists {}, which is not there", path::file_name(Style::HOST, d))))),
            Err(words) => out.push(failed("sheet", words)),
        }
    } else if std::fs::metadata(&f.path).map(|m| m.len() == 0).unwrap_or(false) {
        out.push(failed("size", format!("{} is empty", path::file_name(Style::HOST, &f.path))));
    } else if archive::is_archive(&f.path) {
        check_archive(&f.path, stop, &mut out);
    } else if equals_ignore_case(ext, ".chd") {
        check_chd(f, chdman, stop, &mut out);
    } else {
        match read_through(&f.path, stop) {
            Err(x) => out.push(x),
            Ok(size) if is_disc(f) => {
                let sector: u64 = if equals_ignore_case(ext, ".iso") { 2048 } else { 2352 };
                if size % sector != 0 {
                    out.push(failed("size", format!("its {} bytes are not a whole number of {}-byte sectors", grouped(size), grouped(sector))));
                }
            }
            Ok(_) => {}
        }
    }
    out
}

/// One file of a duplicate group: what it is, and the hashes of its comparable form.
#[derive(Clone, Debug)]
pub struct Member {
    pub rel: String,
    pub path: String,
    pub hashes: Hashes,
}

/// The groups of the same dump (EmuSen_LibraryTools.md §3, §7.4): equal comparable hashes and size, two or more files
/// that are not one game already, a package and the library files it claims counting as one. Each group in order of
/// relative path ignoring case, the groups by their first file.
pub fn groups(mut members: Vec<Member>, index: &claims::Index) -> Vec<Vec<Member>> {
    let key = |m: &Member| (m.hashes.sha1.clone(), m.hashes.md5.clone(), m.hashes.crc32.clone(), m.hashes.size);
    members.sort_by(|a, b| key(a).cmp(&key(b)).then_with(|| compare_ignore_case(&a.rel, &b.rel)));
    let mut out: Vec<Vec<Member>> = Vec::new();
    let mut run: Vec<Member> = Vec::new();
    let mut close = |run: &mut Vec<Member>| {
        let mut units: Vec<String> = run.iter().map(|m| index.claimant(&path::full_path(&m.path)).map_or_else(|| m.path.clone(), |p| p.folder.clone())).collect();
        units.sort();
        units.dedup();
        if units.len() > 1 {
            out.push(std::mem::take(run));
        }
        run.clear();
    };
    for m in members {
        if run.first().is_some_and(|r| key(r) != key(&m)) {
            close(&mut run);
        }
        run.push(m);
    }
    close(&mut run);
    out.sort_by(|a, b| compare_ignore_case(&a[0].rel, &b[0].rel));
    out
}

fn hashes_json(h: &Hashes, name: &str) -> Value {
    obj(vec![("crc32", Value::string(&h.crc32)), ("md5", Value::string(&h.md5)), ("sha1", Value::string(h.sha1.as_deref().unwrap_or(""))), ("size", Value::number(h.size)), ("name", Value::string(name))])
}

fn head(f: &File, index: usize, of: usize, state: &str) -> Vec<(&'static str, Value)> {
    vec![
        ("event", Value::string("file")),
        ("index", Value::number(index + 1)),
        ("of", Value::number(of)),
        ("source", Value::string(&f.rel)),
        ("system", f.system.map_or(Value::Null, |s| Value::string(&s.id))),
        ("state", Value::string(state)),
    ]
}

fn state_of(words: &str) -> &'static str {
    if words == STOPPED { "stopped" } else { "failed" }
}

/// Runs a `rom-*` request: its `start`, a `file` event for each file and, for `rom-dups`, a `group` event for each
/// group; the caller sends the `end`. The events are listed in EmuSen_LibraryTools.md §7.1.
pub fn run(ctx: &Context, library: &str, req: &Request, emit: &dyn Fn(Value), count: &mut dyn FnMut(&'static str), stop: &dyn Fn() -> bool) -> Result<(), String> {
    let console = match (&req.op, req.console.as_deref()) {
        (Op::RomDups, Some(name)) => Some(packages::system_named(name).ok_or_else(|| format!("{name} is no console of this build"))?),
        _ => None,
    };
    let (files, refused) = match req.op {
        Op::RomDups => files(library, &[String::new()], true, console),
        _ => files(library, &req.paths, req.recursive, None),
    };
    let word = match req.op {
        Op::RomHash => "rom-hash",
        Op::RomCheck => "rom-check",
        _ => "rom-dups",
    };
    let mut start = vec![("event", Value::string("start")), ("op", Value::string(word)), ("files", Value::number(files.len()))];
    let mut chdman = None;
    if req.op == Op::RomCheck && files.iter().any(|f| ends_with_ignore_case(&f.path, ".chd")) {
        match Chdman::open(&ctx.chdman, ctx.pin.as_deref(), &ctx.release) {
            Ok(c) => {
                start.push(("chdman", Value::string(&c.version)));
                chdman = Some(c);
            }
            Err(words) => start.push(("chdman_words", Value::string(&words.replace(", and nothing was converted.", ".").replace(": nothing was converted.", ".")))),
        }
    }
    if let Some(c) = console {
        start.push(("console", Value::string(&c.id)));
    }
    start.push(("refused", Value::Array(refused.iter().map(Value::string).collect())));
    emit(obj(start));
    let mut members = Vec::new();
    let mut stopped = false;
    for (i, f) in files.iter().enumerate() {
        if stop() {
            stopped = true;
            count("stopped");
            continue;
        }
        let event = match req.op {
            Op::RomCheck => {
                let failures = check(f, chdman.as_ref(), stop);
                let state = match failures.iter().find(|x| x.words == STOPPED) {
                    Some(_) => "stopped",
                    None if failures.is_empty() => "ok",
                    None => "failed",
                };
                count(state);
                let mut e = head(f, i, files.len(), state);
                e.push(("failed", Value::Array(failures.iter().map(|x| obj(vec![("check", Value::string(x.check)), ("words", Value::string(&x.words))])).collect())));
                e
            }
            Op::RomHash => match hash(f, req.comparable, stop) {
                Ok(lines) => {
                    count("hashed");
                    let mut e = head(f, i, files.len(), "hashed");
                    e.push(("lines", Value::Array(lines.iter().map(|l| hashes_json(&l.hashes, &l.name)).collect())));
                    e
                }
                Err(words) => {
                    count(state_of(&words));
                    let mut e = head(f, i, files.len(), state_of(&words));
                    e.push(("words", Value::string(&words)));
                    e
                }
            },
            _ if image_sets::is_playlist(&f.path) => continue,
            _ => match comparable(f, stop) {
                Ok(h) => {
                    count("hashed");
                    members.push(Member { rel: f.rel.clone(), path: f.path.clone(), hashes: h });
                    head(f, i, files.len(), "hashed")
                }
                Err(words) => {
                    count(state_of(&words));
                    let mut e = head(f, i, files.len(), state_of(&words));
                    e.push(("words", Value::string(&words)));
                    e
                }
            },
        };
        stopped |= event.iter().any(|(k, v)| *k == "state" && v.as_str() == Some("stopped"));
        emit(obj(event));
    }
    if req.op == Op::RomDups && !stopped {
        let index = claims::index(&ctx.places().roots());
        for g in groups(members, &index) {
            count("groups");
            let listed = g
                .iter()
                .map(|m| {
                    let mut e = hashes_json(&m.hashes, &m.rel);
                    if let Some(p) = index.claimant(&path::full_path(&m.path)) {
                        claims::set(&mut e, "package", Value::string(&crate::packages::package_rel(ctx, &p.folder)));
                    }
                    e
                })
                .collect();
            emit(obj(vec![("event", Value::string("group")), ("members", Value::Array(listed))]));
        }
    }
    Ok(())
}
