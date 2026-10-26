//! Game packages made, listed, checked and given the stores a game already had: `pkg make`, `pkg ls`, `pkg check` and
//! `pkg migrate` (EmuSen_GamePackages.md §3.4–§3.7, §14), and since step 7 self-contained packages, a package's zip
//! and `pkg identity` (§19). The library is read and never written, a package is made only when asked, making one
//! self-contained copies its game in and never moves or removes the library's, and migrating copies and never
//! overwrites or removes anything.

use crate::convert::{obj, Outcome};
use crate::Context;
use emusen_dotnet::text::{compare_ignore_case, equals_ignore_case};
use emusen_galaxia::claims::{self, Index, Roots};
use emusen_galaxia::dotnet_files::{self, Options};
use emusen_galaxia::dotnet_path::{self as path, Style};
use emusen_galaxia::json::Value;
use emusen_galaxia::rom_hash::{self, Hashes};
use emusen_galaxia::{archive, cheats, image_sets, package};

/// One game of the library as `pkg make` takes it: its path relative to the library, its full path, and its system.
#[derive(Clone, Debug)]
pub struct Game {
    pub rel: String,
    pub path: String,
    pub system: &'static emusen_systems::System,
}

fn every_system() -> emusen_library::scan::Systems {
    let ids: Vec<String> = emusen_systems::table().map(|t| t.iter().map(|s| s.id.clone()).collect()).unwrap_or_default();
    emusen_library::scan::Systems::named(&ids)
}

/// The system a manifest names, by its ES-DE name, else by its id.
pub fn system_named(name: &str) -> Option<&'static emusen_systems::System> {
    let table = emusen_systems::table().ok()?;
    table.iter().find(|s| equals_ignore_case(&s.esde_system, name)).or_else(|| table.iter().find(|s| equals_ignore_case(&s.id, name)))
}

/// Where the scan's routing names no system, the nearest folder above the game named as a system's ES-DE folder.
fn by_folder(rel: &str) -> Option<&'static emusen_systems::System> {
    let folders: Vec<&str> = rel.split('/').collect();
    let table = emusen_systems::table().ok()?;
    folders[..folders.len().saturating_sub(1)].iter().rev().find_map(|f| table.iter().find(|s| s.esde_system.eq_ignore_ascii_case(f)))
}

/// The games under each named path, as the library's scan shelves and routes them, in order of their relative paths
/// ignoring case; the words for a path that names nothing, and for a file no system takes.
pub fn games(library: &str, given: &[String]) -> (Vec<Game>, Vec<String>) {
    let mut files: Vec<String> = Vec::new();
    let mut direct: Vec<String> = Vec::new();
    let mut refused = Vec::new();
    for g in given {
        let rel = match crate::discs::relative(g) {
            Ok(rel) => rel,
            Err(words) => {
                refused.push(words);
                continue;
            }
        };
        let at = claims::under(library, &rel);
        if dotnet_files::directory_exists(&at) {
            let _ = dotnet_files::visit_files_with(&at, Options { skip_hidden: true, ignore_inaccessible: true }, |f, _| files.push(f.to_string()));
        } else if dotnet_files::file_exists(&at) {
            direct.push(at.clone());
            files.push(at);
        } else {
            refused.push(format!("{g}: no such file or folder in the library"));
        }
    }
    files.sort();
    files.dedup();
    let systems = every_system();
    let extensions = systems.extensions();
    let mut found = Vec::new();
    for at in image_sets::shelved(files) {
        let rel = crate::discs::rel_of(library, &at);
        if !extensions.iter().any(|e| equals_ignore_case(e, path::extension(Style::HOST, &at))) {
            if direct.contains(&at) {
                refused.push(format!("{rel}: no system of this build takes its kind of file"));
            }
            continue;
        }
        let system = emusen_library::scan::route(&at, &systems).and_then(|id| emusen_systems::find(&id)).or_else(|| by_folder(&rel));
        match system {
            Some(system) => found.push(Game { rel, path: at, system }),
            None => refused.push(format!("{rel}: no system of this build takes it")),
        }
    }
    found.sort_by(|a, b| compare_ignore_case(&a.rel, &b.rel).then_with(|| a.rel.cmp(&b.rel)));
    (found, refused)
}

/// A package's folder for a library game: `Packages/<console>/<the path less the library's console folder>/`, named
/// as the game's file is, as ES-DE names a folder it takes as one game (§3.1, §3.2).
pub fn place(ctx: &Context, game: &Game) -> (String, String) {
    let rest = game.rel.split_once('/').map_or(game.rel.as_str(), |(_, r)| r);
    let rel = format!("{}/{}", game.system.console, rest);
    (claims::under(&ctx.packages(), &rel), rel)
}

/// A package's folder relative to `Packages`, `/` between its parts.
pub fn package_rel(ctx: &Context, folder: &str) -> String {
    crate::discs::rel_of(&ctx.packages(), folder)
}

fn text(s: &str) -> Value {
    Value::string(s)
}

fn hashes_json(h: &Hashes, extra: Vec<(&str, Value)>) -> Value {
    let mut members = extra;
    members.extend([("size", Value::number(h.size)), ("crc32", text(&h.crc32)), ("md5", text(&h.md5)), ("sha1", h.sha1.as_deref().map_or(Value::Null, text))]);
    obj(members)
}

fn entry(role: &str, place: &str, rel: &str, at: &str) -> Value {
    let mut e = package::file_entry(role, place, rel, claims::seen_of(at).map(|s| s.0));
    if let Some((size, mtime)) = claims::seen_of(at) {
        claims::set(&mut e, "seen", obj(vec![("size", Value::number(size)), ("mtime", text(&mtime))]));
    }
    e
}

fn hash(at: &str, stop: &dyn Fn() -> bool) -> Result<Hashes, String> {
    rom_hash::hashes_with(at, None, true, stop).map_err(|e| if e.kind() == std::io::ErrorKind::Interrupted { "stopped".into() } else { format!("{at}: {e}") })
}

/// The comparable form's hashes where the system pack's transform changes the bytes (§3.3.1), none where it does not.
fn transformed(system: &str, bytes: &[u8]) -> Value {
    rom_hash::transformed(bytes, |f| emusen_systems::openvgdb::bytes(system, f)).map_or(Value::Null, |h| hashes_json(&h, vec![]))
}

/// What `pkg make` is asked beyond a linked package: the game copied into it, and the package written as one zip.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Making {
    pub self_contained: bool,
    pub zip: bool,
}

/// A package made for one library game, linked unless asked otherwise (§3.4, §9 Q6). A game a package already claims
/// is `already`, unless the package is to be made self-contained or zipped, which is done to that package (§19).
pub fn make(ctx: &Context, index: &mut Index, game: &Game, how: Making, stop: &dyn Fn() -> bool) -> Outcome {
    let places = ctx.places();
    let roots = places.roots();
    let full = path::full_path(&game.path);
    if let Some(p) = index.claimant(&full) {
        let folder = p.folder.clone();
        if how == Making::default() {
            return Outcome::new("already").with("package", text(&package_rel(ctx, &folder)));
        }
        return further(ctx, index, &folder, how, "already", stop);
    }
    let (folder, folder_rel) = place(ctx, game);
    let manifest = dotnet_files::join(&folder, package::FILE_NAME);
    if dotnet_files::file_exists(&manifest) {
        return Outcome::said("failed", format!("Packages/{folder_rel} already holds a package for another file, so none was made")).with("package", text(&folder_rel));
    }
    let library = places.library.clone().unwrap_or_default();
    let rel_of = |at: &str| crate::discs::rel_of(&library, at);
    let dump: Vec<(&str, Value)>;
    let mut container = Value::Null;
    let companions: Vec<String>;
    let id = game.system.id.as_str();
    if image_sets::is_sheet(&game.path) {
        let tracks = match rom_hash::tracks(&game.path, stop) {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => return Outcome::new("stopped"),
            Err(e) => return Outcome::said("failed", format!("its tracks could not be hashed: {e}")),
        };
        let sheet = match hash(&game.path, stop) {
            Ok(h) => h,
            Err(words) => return Outcome::said(if words == "stopped" { "stopped" } else { "failed" }, words),
        };
        dump = vec![
            ("file", Value::Null),
            ("transformed", Value::Null),
            ("tracks", Value::Array(tracks.iter().map(|t| hashes_json(&t.hashes, vec![("file", text(&t.file))])).collect())),
            ("sheet", hashes_json(&sheet, vec![("file", text(path::file_name(Style::HOST, &game.path)))])),
            ("form", text("cue-bin")),
        ];
        companions = image_sets::files_for(&game.path).unwrap_or_default();
    } else if archive::is_archive(&game.path) {
        let takes = |e: &str| game.system.extensions.iter().any(|x| equals_ignore_case(x, e));
        let Ok(Some(name)) = archive::image_entry(&game.path, takes) else { return Outcome::said("failed", "the archive holds no image of its system") };
        let bytes = match archive::read_entry(&game.path, &name, usize::MAX) {
            Ok(b) => b,
            Err(words) => return Outcome::said("failed", format!("{name} could not be read from the archive: {words}")),
        };
        let whole = match hash(&game.path, stop) {
            Ok(h) => h,
            Err(words) => return Outcome::said(if words == "stopped" { "stopped" } else { "failed" }, words),
        };
        dump = vec![("file", hashes_json(&rom_hash::hashes_of(&bytes), vec![])), ("transformed", transformed(id, &bytes))];
        container = hashes_json(&whole, vec![("type", text("zip")), ("entry", text(&name))]);
        companions = Vec::new();
    } else {
        let h = match hash(&game.path, stop) {
            Ok(h) => h,
            Err(words) => return Outcome::said(if words == "stopped" { "stopped" } else { "failed" }, words),
        };
        let t = if emusen_systems::openvgdb::transforms(id) { std::fs::read(&game.path).map(|b| transformed(id, &b)).unwrap_or(Value::Null) } else { Value::Null };
        dump = vec![("file", hashes_json(&h, vec![])), ("transformed", t)];
        companions = image_sets::all_files(&game.path).map(|a| a.into_iter().skip(1).collect()).unwrap_or_default();
    }
    let playlist = image_sets::is_playlist(&game.path);
    let companion_entries: Vec<Value> = companions
        .iter()
        .map(|c| {
            let role = if playlist { "disc" } else if path::extension(Style::HOST, c).eq_ignore_ascii_case(".sbi") { "sbi" } else { "track" };
            match claims::entry_place(&roots, &path::full_path(c)) {
                ("library", rel) => entry(role, "library", &rel, c),
                (_, full) => entry(role, "absolute", &full, c),
            }
        })
        .collect();
    let title = path::file_name_without_extension(Style::HOST, &game.path).to_string();
    let mut doc = package::new(Some(&title), &game.system.esde_system, "linked", entry("image", "library", &rel_of(&full), &game.path));
    claims::set(&mut doc, "dump", obj(dump));
    claims::set(&mut doc, "companions", Value::Array(companion_entries));
    claims::set(&mut doc, "container", container);
    match package::write(&manifest, &doc) {
        Ok(Ok(())) => {}
        Ok(Err(words)) => return Outcome::said("failed", words),
        Err(refusal) => return Outcome::said("failed", refusal.words().to_string()),
    }
    if let Some(p) = claims::package_of(&roots, &folder, &doc) {
        index.folders.push(folder.clone());
        index.packages.push(p);
    }
    if how != Making::default() {
        return further(ctx, index, &folder, how, "made", stop);
    }
    Outcome::new("made").with("package", text(&folder_rel)).with("mode", text("linked")).with("size", Value::number(claims::seen_of(&game.path).map_or(0, |s| s.0)))
}

/// A package made self-contained where asked and it is not, then written as its zip where asked: `made` or `already`
/// as it was found, `contained` for a linked package whose game was copied in now, `zipped` for one only zipped;
/// `failed` with words where its game is not the dump, a copy is not the file it was copied from, or the zip could
/// not be written, the package left as it was found or as far as it got (§19).
fn further(ctx: &Context, index: &mut Index, folder: &str, how: Making, found: &'static str, stop: &dyn Fn() -> bool) -> Outcome {
    let rel = package_rel(ctx, folder);
    let mut doc = match read(folder) {
        Ok(d) => d,
        Err(words) => return Outcome::said("failed", words).with("package", text(&rel)),
    };
    let mut state = found;
    let mut copied = Vec::new();
    if how.self_contained && doc.get("mode").and_then(Value::as_str) != Some("self-contained") {
        match contain(ctx, folder, &mut doc, stop) {
            Ok(names) => copied = names,
            Err(words) if words == "stopped" => return Outcome::new("stopped").with("package", text(&rel)),
            Err(words) => {
                let words = if found == "made" { format!("the package was made linked, and its game was not copied in: {words}") } else { words };
                return Outcome::said("failed", words).with("package", text(&rel)).with("mode", text("linked"));
            }
        }
        if found == "already" {
            state = "contained";
        }
        let roots = ctx.places();
        if let Some(p) = claims::package_of(&roots.roots(), folder, &doc) {
            if let Some(slot) = index.packages.iter_mut().find(|q| q.folder == folder) {
                *slot = p;
            }
        }
    }
    let mode = doc.get("mode").and_then(Value::as_str).unwrap_or("linked").to_string();
    let mut outcome_zip = Value::Null;
    if how.zip {
        if mode != "self-contained" {
            return Outcome::said("failed", "a linked package's zip would hold no game, so none was written").with("package", text(&rel)).with("mode", text(&mode));
        }
        let zip = emusen_galaxia::package_zip::zip_beside(folder);
        match emusen_galaxia::package_zip::write(folder, &zip, stop) {
            Ok((_, bytes)) => {
                outcome_zip = obj(vec![("path", text(&package_rel(ctx, &zip))), ("bytes", Value::number(bytes))]);
                if state == "already" {
                    state = "zipped";
                }
            }
            Err(words) if words == "stopped" => return Outcome::new("stopped").with("package", text(&rel)),
            Err(words) => return Outcome::said("failed", format!("its zip was not written: {words}")).with("package", text(&rel)).with("mode", text(&mode)),
        }
    }
    let mut outcome = Outcome::new(state).with("package", text(&rel)).with("mode", text(&mode));
    if !copied.is_empty() {
        outcome = outcome.with("copied", Value::Array(copied.iter().map(|c| text(c)).collect()));
    }
    if how.zip {
        outcome = outcome.with("zip", outcome_zip);
    }
    outcome
}

/// `pkg zip`: a self-contained package written as one zip beside its folder, as it now is, its saves and states with
/// it; `zipped`, or `failed` for a linked package or a zip already there (§19).
pub fn zip(ctx: &Context, folder: &str, stop: &dyn Fn() -> bool) -> Outcome {
    further(ctx, &mut Index::default(), folder, Making { self_contained: false, zip: true }, "already", stop)
}

/// A file copied into the package as `rel`, its time kept, and checked: the copy's SHA-1 is the SHA-1 of the bytes read
/// from the original in the same run. A copy already there of the same bytes is kept, any other file there refuses it.
/// The original is only read. Its SHA-1 and size.
fn copy_in(folder: &str, from: &str, rel: &str, stop: &dyn Fn() -> bool) -> Result<(String, u64), String> {
    use std::io::{Read, Write};
    let target = claims::under(folder, rel);
    let source_sha1 = || rom_hash::hashes_with(from, None, true, stop).map(|h| (h.sha1.unwrap_or_default(), h.size)).map_err(|e| if e.kind() == std::io::ErrorKind::Interrupted { "stopped".to_string() } else { format!("{from}: {e}") });
    if std::fs::symlink_metadata(&target).is_ok() {
        let (want, size) = source_sha1()?;
        let there = rom_hash::sha1(&target).map_err(|e| format!("{target}: {e}"))?;
        return if there == want { Ok((want, size)) } else { Err(format!("{rel} is already in the package and is another file, so it was left as it is")) };
    }
    let modified = std::fs::metadata(from).and_then(|m| m.modified()).map_err(|e| format!("{from}: {e}"))?;
    let dir = path::directory_name(Style::HOST, &target).unwrap_or_default();
    std::fs::create_dir_all(&dir).map_err(|e| format!("{dir}: {e}"))?;
    let partial = path::combine(Style::HOST, &dir, &format!(".{}.partial-{}", path::file_name(Style::HOST, &target), std::process::id()));
    let copied = (|| {
        let mut input = std::fs::File::open(from).map_err(|e| format!("{from}: {e}"))?;
        let mut out = std::fs::File::create(&partial).map_err(|e| format!("{partial}: {e}"))?;
        let mut hasher = rom_hash::Hasher::new(true);
        let mut buffer = vec![0u8; 1 << 20];
        loop {
            if stop() {
                return Err("stopped".to_string());
            }
            let n = input.read(&mut buffer).map_err(|e| format!("{from}: {e}"))?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
            out.write_all(&buffer[..n]).map_err(|e| format!("{partial}: {e}"))?;
        }
        out.sync_all().map_err(|e| format!("{partial}: {e}"))?;
        out.set_modified(modified).map_err(|e| format!("{partial}: {e}"))?;
        drop(out);
        let read = hasher.finish();
        let written = rom_hash::sha1(&partial).map_err(|e| format!("{partial}: {e}"))?;
        let want = read.sha1.unwrap_or_default();
        if written != want {
            return Err(format!("the copy of {from} has SHA-1 {written}, and the bytes read have {want}: nothing was kept"));
        }
        Ok((want, read.size))
    })();
    match copied {
        Ok(done) => {
            std::fs::rename(&partial, &target).map_err(|e| {
                let _ = std::fs::remove_file(&partial);
                format!("{target}: {e}")
            })?;
            Ok(done)
        }
        Err(words) => {
            let _ = std::fs::remove_file(&partial);
            Err(words)
        }
    }
}

fn seen_json(at: &str) -> Value {
    claims::seen_of(at).map_or(Value::Null, |(size, mtime)| obj(vec![("size", Value::number(size)), ("mtime", text(&mtime))]))
}

/// A package made self-contained (§3.4, §19): its game and every companion copied in beside the manifest, in the places
/// the game's sheet or list names them, each checked; then the manifest names the copies (`where: package`), keeps the
/// entries they were copied from as `sources` with their sizes, times and SHA-1s, and says `self-contained`. The game
/// must still be the dump. Nothing is moved, removed or written outside the package. The names copied.
fn contain(ctx: &Context, folder: &str, doc: &mut Value, stop: &dyn Fn() -> bool) -> Result<Vec<String>, String> {
    let places = ctx.places();
    let roots = places.roots();
    let game = doc.get("game").and_then(|g| claims::place(&roots, folder, g)).ok_or("its game's library is not set (AppSettings.RomDirectory)")?;
    if claims::seen_of(&game).is_none() {
        return Err(format!("{} is gone; pkg check finds a moved game first", shown(&roots, &game)));
    }
    let resolved = claims::open_package(&roots, folder, None)?;
    if resolved.game != game {
        return Err(format!("{} is gone; pkg check finds a moved game first", shown(&roots, &game)));
    }
    let doc_now = read(folder)?;
    *doc = doc_now;
    let base = path::directory_name(Style::HOST, &game).unwrap_or_default();
    let within = |at: &str| -> Result<String, String> {
        if claims::within(at, &base) && at.len() > base.len() {
            Ok(at[base.len()..].trim_start_matches(['/', '\\']).replace('\\', "/"))
        } else {
            Err(format!("{} is not in its game's folder, so the package cannot hold it where the game names it", shown(&roots, at)))
        }
    };
    let companions: Vec<Value> = match doc.get("companions") {
        Some(Value::Array(items)) => items.clone(),
        _ => Vec::new(),
    };
    let mut plan: Vec<(Value, String, String)> = vec![(doc.get("game").cloned().unwrap_or(Value::Null), game.clone(), path::file_name(Style::HOST, &game).to_string())];
    for c in &companions {
        let at = claims::place(&roots, folder, c).ok_or("a companion's library is not set")?;
        plan.push((c.clone(), at.clone(), within(&at)?));
    }
    for (_, _, rel) in &plan {
        let first = rel.split('/').next().unwrap_or("");
        if [package::FILE_NAME, package::SAVES_FOLDER, package::STATES_FOLDER, package::MEDIA_FOLDER, package::CHEATS_FILE, package::SETTINGS_FILE].iter().any(|r| equals_ignore_case(r, first)) {
            return Err(format!("{rel} would take the place of the package's own {first}"));
        }
    }
    let mut sources = Vec::new();
    let mut moved_in = Vec::new();
    let mut names = Vec::new();
    for (entry, at, rel) in &plan {
        let (sha1, size) = copy_in(folder, at, rel, stop)?;
        let mut source = entry.clone();
        claims::set(&mut source, "seen", seen_json(at));
        claims::set(&mut source, "size", Value::number(size));
        claims::set(&mut source, "sha1", text(&sha1));
        sources.push(source);
        let copy = claims::under(folder, rel);
        let role = entry.get("role").and_then(Value::as_str).unwrap_or("image");
        let mut e = package::file_entry(role, "package", rel, Some(size));
        claims::set(&mut e, "seen", seen_json(&copy));
        moved_in.push(e);
        names.push(rel.clone());
    }
    let game_entry = moved_in.remove(0);
    claims::set(doc, "mode", text("self-contained"));
    claims::set(doc, "game", game_entry);
    claims::set(doc, "companions", Value::Array(moved_in));
    claims::set(doc, "sources", Value::Array(sources));
    let manifest = dotnet_files::join(folder, package::FILE_NAME);
    match package::write(&manifest, doc) {
        Ok(Ok(())) => Ok(names),
        Ok(Err(words)) => Err(words),
        Err(refusal) => Err(refusal.words().to_string()),
    }
}

/// The packages each path names, relative to `Packages` or full, a folder holding packages naming each of them, in
/// order; every package where none is named; the words for a path that names none.
pub fn named(ctx: &Context, given: &[String]) -> (Vec<String>, Vec<String>) {
    let packages = ctx.packages();
    let all = claims::folders(&packages);
    if given.is_empty() {
        return (all, Vec::new());
    }
    let (mut found, mut refused) = (Vec::new(), Vec::new());
    for g in given {
        let rooted = path::is_rooted(Style::HOST, g) && g.trim_matches(['/', '\\']) != "";
        let at = if rooted { path::full_path(g) } else { claims::under(&packages, &crate::discs::relative(g).unwrap_or_default()) };
        let at = path::trim_ending_separator(Style::HOST, &at).to_string();
        let mine: Vec<&String> = all.iter().filter(|f| claims::within(f, &at)).collect();
        if mine.is_empty() {
            refused.push(format!("{g}: no package there"));
        }
        for f in mine {
            if !found.contains(f) {
                found.push(f.clone());
            }
        }
    }
    (found, refused)
}

/// Whether a path a job was handed is a package's zip rather than its folder.
pub fn is_zip(at: &str) -> bool {
    emusen_galaxia::archive::is_archive(at) && dotnet_files::file_exists(at)
}

/// `named`, and where `zips` allows it, a package's zip named by its path, relative to `Packages` or full (§19).
pub fn named_or_zipped(ctx: &Context, given: &[String], zips: bool) -> (Vec<String>, Vec<String>) {
    let packages = ctx.packages();
    let (mut zipped, mut rest) = (Vec::new(), Vec::new());
    for g in given {
        let rooted = path::is_rooted(Style::HOST, g);
        let at = if rooted { path::full_path(g) } else { claims::under(&packages, &crate::discs::relative(g).unwrap_or_default()) };
        if is_zip(&at) { zipped.push((g.clone(), at)) } else { rest.push(g.clone()) }
    }
    let (mut found, mut refused) = if rest.is_empty() && !zipped.is_empty() { (Vec::new(), Vec::new()) } else { named(ctx, &rest) };
    for (g, at) in zipped {
        match emusen_galaxia::package_zip::manifest_in(&at) {
            Ok(Some(_)) if zips => found.push(at),
            Ok(Some(_)) => refused.push(format!("{g}: a package's zip is checked with pkg check, and is unpacked into Packages when it is opened")),
            Ok(None) => refused.push(format!("{g}: a zip that holds no package")),
            Err(words) => refused.push(format!("{g}: {words}")),
        }
    }
    (found, refused)
}

/// `pkg check` of a package's zip: unpacked into a hidden folder of `Packages`, every entry checked against its CRC-32
/// and given its time, the package checked there as a folder is, and the folder removed; the zip is only read (§19).
pub fn check_zip(ctx: &Context, zip: &str, stop: &dyn Fn() -> bool) -> Outcome {
    let (top, _) = match emusen_galaxia::package_zip::manifest_in(zip) {
        Ok(Some(found)) => found,
        Ok(None) => return Outcome::said("refused", "the zip holds no package"),
        Err(words) => return Outcome::said("refused", words),
    };
    let scratch = path::combine(Style::HOST, &ctx.packages(), &format!(".check-{}-{}", std::process::id(), SCRATCH.fetch_add(1, std::sync::atomic::Ordering::Relaxed)));
    let folder = path::combine(Style::HOST, &scratch, &top);
    let outcome = match emusen_galaxia::package_zip::unpack(zip, &top, &folder, stop) {
        Ok(()) => check(ctx, &folder, stop),
        Err(words) if words == "stopped" => Outcome::new("stopped"),
        Err(words) => Outcome::said("changed", words),
    };
    let _ = std::fs::remove_dir_all(&scratch);
    outcome.with("zip", Value::Bool(true))
}

static SCRATCH: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// `pkg identity`: which identity the scraper sends for a patched package, `original` or `patched` (§5.5, §19). The
/// saves a patched game keeps are named by its patch list, not by this (§16), so nothing else changes. `set`,
/// `already`, or `failed` for `patched` on a package with no soft patches.
pub fn identity(folder: &str, want: &str) -> Outcome {
    let manifest = dotnet_files::join(folder, package::FILE_NAME);
    let mut doc = match read(folder) {
        Ok(d) => d,
        Err(words) => return Outcome::said("refused", words),
    };
    if want == "patched" && emusen_patch::package::entries(&doc).is_empty() {
        return head(&doc, Outcome::said("failed", "it has no soft patches, so it has no patched identity: the scraper sends its dump"));
    }
    if doc.get("identity").and_then(Value::as_str).unwrap_or("original") == want {
        return head(&doc, Outcome::new("already"));
    }
    claims::set(&mut doc, "identity", text(want));
    match package::write(&manifest, &doc) {
        Ok(Ok(())) => head(&doc, Outcome::new("set")),
        Ok(Err(words)) => head(&doc, Outcome::said("failed", words)),
        Err(refusal) => head(&doc, Outcome::said("failed", refusal.words().to_string())),
    }
}

/// The packages of one console, by its name, its system's id or ES-DE name, or the folder under `Packages`.
pub fn of_console(ctx: &Context, console: &str) -> Vec<String> {
    let packages = ctx.packages();
    claims::folders(&packages)
        .into_iter()
        .filter(|f| {
            let top = package_rel(ctx, f).split('/').next().unwrap_or("").to_string();
            let system = read(f).ok().and_then(|d| d.get("system").and_then(Value::as_str).map(str::to_string)).unwrap_or_default();
            let s = system_named(&system);
            equals_ignore_case(&top, console) || equals_ignore_case(&system, console) || s.is_some_and(|s| equals_ignore_case(&s.id, console) || equals_ignore_case(&s.console, console))
        })
        .collect()
}

pub(crate) fn read(folder: &str) -> Result<Value, String> {
    let manifest = dotnet_files::join(folder, package::FILE_NAME);
    match package::read(&manifest) {
        Ok(Ok(doc)) => Ok(doc),
        Ok(Err(refusal)) => Err(refusal.words().to_string()),
        Err(e) => Err(e.to_string()),
    }
}

pub(crate) fn head(doc: &Value, outcome: Outcome) -> Outcome {
    let game = doc.get("game");
    let patches = emusen_patch::package::entries(doc);
    let formats: Vec<(emusen_patch::Format, String)> = patches.iter().map(|e| (e.get("format").and_then(Value::as_str).and_then(emusen_patch::Format::named).unwrap_or(emusen_patch::Format::Ips), String::new())).collect();
    let outcome = if patches.is_empty() {
        outcome
    } else {
        outcome
            .with("patches", Value::Array(patches.iter().map(|e| text(e.get("file").and_then(Value::as_str).unwrap_or(""))).collect()))
            .with("retroarch", Value::Bool(crate::patches::names("", &formats).1))
            .with("identity", text(doc.get("identity").and_then(Value::as_str).unwrap_or("original")))
    };
    outcome
        .with("mode", text(doc.get("mode").and_then(Value::as_str).unwrap_or("")))
        .with("system", text(doc.get("system").and_then(Value::as_str).unwrap_or("")))
        .with("where", text(game.and_then(|g| g.get("where")).and_then(Value::as_str).unwrap_or("")))
        .with("game", text(game.and_then(|g| g.get("path")).and_then(Value::as_str).unwrap_or("")))
}

/// Whether a manifest's game is looked for in the library when it is gone: a game linked there or anywhere else, never
/// a game kept in its package or in `Compressed`, so that a self-contained package is never linked again unasked.
fn linked(doc: &Value) -> bool {
    matches!(doc.get("game").and_then(|g| g.get("where")).and_then(Value::as_str), Some("library" | "absolute"))
}

fn shown(roots: &Roots, at: &str) -> String {
    claims::entry_place(roots, at).1
}

fn chd(doc: &Value) -> bool {
    doc.get("container").and_then(|c| c.get("type")).and_then(Value::as_str) == Some("chd")
}

fn same_seen(doc: &Value, now: &(u64, String)) -> bool {
    let seen = doc.get("game").and_then(|g| g.get("seen"));
    seen.and_then(|s| s.get("size")).and_then(|v| if let Value::Number(t) = v { t.parse::<u64>().ok() } else { None }) == Some(now.0) && seen.and_then(|s| s.get("mtime")).and_then(Value::as_str) == Some(now.1.as_str())
}

/// `pkg ls`'s view of one package, read and never written: `ok`, `changed` (another dump now), `moved` (found again
/// by size and hashes in the library, where `to` names), `missing`, or `refused` (a manifest this build does not read).
pub fn list(ctx: &Context, folder: &str) -> Outcome {
    let places = ctx.places();
    let roots = places.roots();
    let doc = match read(folder) {
        Ok(d) => d,
        Err(words) => return Outcome::said("refused", words),
    };
    let Some(game) = doc.get("game").and_then(|g| claims::place(&roots, folder, g)) else { return head(&doc, Outcome::said("missing", "its game's library is not set (AppSettings.RomDirectory)")) };
    let Some(now) = claims::seen_of(&game) else {
        let found = linked(&doc).then(|| claims::recorded(&doc).and_then(|r| claims::recorded_size(&doc).and_then(|size| claims::find_in_library(&roots, size, r)))).flatten();
        return head(
            &doc,
            match found {
                Some(at) => Outcome::new("moved").with("to", text(&shown(&roots, &at))),
                None => Outcome::said("missing", format!("{} is gone", shown(&roots, &game))),
            },
        );
    };
    if let Some(gone) = companions_gone(&roots, folder, &doc).first() {
        return head(&doc, Outcome::said("missing", format!("{gone} is gone")));
    }
    if same_seen(&doc, &now) || doc.get("game").and_then(|g| g.get("seen")).is_none() {
        return head(&doc, Outcome::new("ok"));
    }
    if chd(&doc) {
        return head(&doc, Outcome::said("changed", "the CHD's size or time is not the one recorded: chd check checks it"));
    }
    match claims::recorded(&doc) {
        None => head(&doc, Outcome::new("ok")),
        Some(recorded) => match hash(&game, &|| false) {
            Ok(h) if claims::same_dump(recorded, &h) => head(&doc, Outcome::new("ok")),
            Ok(_) => head(&doc, Outcome::said("changed", format!("{} is not the dump recorded ({})", shown(&roots, &game), claims::named(recorded)))),
            Err(words) => head(&doc, Outcome::said("changed", words)),
        },
    }
}

fn companions_gone(roots: &Roots, folder: &str, doc: &Value) -> Vec<String> {
    match doc.get("companions") {
        Some(Value::Array(items)) => items.iter().filter_map(|c| claims::place(roots, folder, c)).filter(|at| !dotnet_files::file_exists(at)).map(|at| shown(roots, &at)).collect(),
        _ => Vec::new(),
    }
}

/// `pkg check`: the game and every file hashed against the manifest (§3.7). A game gone is looked for by size and
/// hashes in the library and the manifest names where it was found; a game whose time alone changed has its time
/// recorded. `verified`, `moved`, `changed`, `missing`, `refused` or `stopped`.
pub fn check(ctx: &Context, folder: &str, stop: &dyn Fn() -> bool) -> Outcome {
    let places = ctx.places();
    let roots = places.roots();
    let manifest = dotnet_files::join(folder, package::FILE_NAME);
    let mut doc = match read(folder) {
        Ok(d) => d,
        Err(words) => return Outcome::said("refused", words),
    };
    let Some(mut game) = doc.get("game").and_then(|g| claims::place(&roots, folder, g)) else { return head(&doc, Outcome::said("missing", "its game's library is not set (AppSettings.RomDirectory)")) };
    let mut state = "verified";
    let mut words: Option<String> = None;
    if claims::seen_of(&game).is_none() {
        let found = linked(&doc).then(|| claims::recorded(&doc).cloned().and_then(|r| claims::recorded_size(&doc).and_then(|size| claims::find_in_library(&roots, size, &r)))).flatten();
        match found {
            Some(at) => {
                claims::moved(&roots, &mut doc, &at);
                let _ = package::write(&manifest, &doc);
                words = Some(format!("found at {}, and the package now names it", shown(&roots, &at)));
                state = "moved";
                game = at;
            }
            None => return head(&doc, Outcome::said("missing", format!("{} is gone, and no file of its size and hashes is in the library", shown(&roots, &game)))),
        }
    }
    if chd(&doc) {
        let now = claims::seen_of(&game).unwrap_or_default();
        return head(&doc, if same_seen(&doc, &now) { Outcome::said(state, "the CHD is the one recorded; chd check runs its three checks") } else { Outcome::said("changed", "the CHD's size or time is not the one recorded: chd check checks it") });
    }
    if let Some(recorded) = claims::recorded(&doc).cloned() {
        match hash(&game, stop) {
            Err(w) if w == "stopped" => return Outcome::new("stopped"),
            Err(w) => return head(&doc, Outcome::said("changed", w)),
            Ok(h) if !claims::same_dump(&recorded, &h) => {
                let taken = if recorded.get("sha1").is_some() { format!("SHA-1 {}", h.sha1.unwrap_or_default()) } else { format!("MD5 {}", h.md5) };
                return head(&doc, Outcome::said("changed", format!("{} has {taken}, and the package was made for {}", shown(&roots, &game), claims::named(&recorded))));
            }
            Ok(_) => {}
        }
    }
    if let Some(Value::Object(_)) = doc.get("container")
        && let (Some(name), Some(file)) = (doc.get("container").and_then(|c| c.get("entry")).and_then(Value::as_str), doc.get("dump").and_then(|d| d.get("file")))
    {
        match archive::read_entry(&game, name, usize::MAX) {
            Ok(bytes) if claims::same_dump(file, &rom_hash::hashes_of(&bytes)) => {}
            Ok(_) => return head(&doc, Outcome::said("changed", format!("{name} in {} is not the dump recorded ({})", shown(&roots, &game), claims::named(file)))),
            Err(w) => return head(&doc, Outcome::said("changed", w)),
        }
    }
    if let Some(gone) = companions_gone(&roots, folder, &doc).first() {
        return head(&doc, Outcome::said("missing", format!("{gone} is gone")));
    }
    if let Some(Value::Array(tracks)) = doc.get("dump").and_then(|d| d.get("tracks")).filter(|_| image_sets::is_sheet(&game)) {
        let taken = match rom_hash::tracks(&game, stop) {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => return Outcome::new("stopped"),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return head(&doc, Outcome::said("missing", e.to_string())),
            Err(e) => return head(&doc, Outcome::said("changed", e.to_string())),
        };
        if taken.len() != tracks.len() {
            return head(&doc, Outcome::said("changed", format!("the sheet now has {} tracks, and the package recorded {}", taken.len(), tracks.len())));
        }
        for (t, want) in taken.iter().zip(tracks) {
            if !claims::same_dump(want, &t.hashes) {
                return head(&doc, Outcome::said("changed", format!("track {} ({}) is not the one recorded ({})", t.number, t.file, claims::named(want))));
            }
        }
    }
    if let Err(words) = crate::patches::check(ctx, folder, &doc) {
        return head(&doc, Outcome::said("changed", words));
    }
    if let Some(now) = claims::seen_of(&game).filter(|n| !same_seen(&doc, n))
        && doc.get("game").and_then(|g| g.get("seen")).is_some()
    {
        if let Value::Object(members) = &mut doc
            && let Some((_, g)) = members.iter_mut().rev().find(|(k, _)| k.text == "game")
        {
            claims::set(g, "seen", obj(vec![("size", Value::number(now.0)), ("mtime", text(&now.1))]));
        }
        let _ = package::write(&manifest, &doc);
        words = words.or(Some("its time changed and it is still the dump; the time is recorded".into()));
    }
    let outcome = match words {
        Some(w) => Outcome::said(state, w),
        None => Outcome::new(state),
    };
    head(&doc, outcome)
}

/// Whether a file of a save folder is one of the game's: its stem, then one or two parts of a dot and up to sixteen
/// letters or digits (`.srm`, `.1.mcr`), so that "Super Mario Bros. 3" is never "Super Mario Bros"'s.
fn save_of(name: &str, stem: &str) -> bool {
    let Some(rest) = name.strip_prefix(stem) else { return false };
    let parts: Vec<&str> = rest.split('.').collect();
    parts.len() >= 2 && parts.len() <= 3 && parts[0].is_empty() && parts[1..].iter().all(|p| !p.is_empty() && p.len() <= 16 && p.chars().all(|c| c.is_ascii_alphanumeric()))
}

/// Whether a file of the state folder is one of the game's states or their pictures: `<stem>.state`, `.slotN.state`,
/// `.resume.state`, and each with `.png`.
fn state_of(name: &str, stem: &str) -> bool {
    let Some(rest) = name.strip_prefix(stem) else { return false };
    let rest = match rest.strip_suffix(".state").or_else(|| rest.strip_suffix(".png")) {
        Some(r) => r,
        None => return false,
    };
    rest.is_empty() || rest == ".resume" || rest.strip_prefix(".slot").is_some_and(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
}

fn files_in(folder: &str, keep: impl Fn(&str) -> bool) -> Vec<String> {
    let Ok(listing) = std::fs::read_dir(folder) else { return Vec::new() };
    let mut found: Vec<String> = listing
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| keep(n))
        .map(|n| dotnet_files::join(folder, &n))
        .collect();
    found.sort();
    found
}

/// `pkg migrate`: the game's existing stores copied into its package once (§3.5, `DataMigration`'s rule): its battery
/// saves and per-game save media from `Saves/<console>` and the places earlier builds kept them, its states and their
/// pictures from the state folder with each state's record, its cheat list, its own settings with its crop, and its art
/// from the media store (§18.4). Nothing is overwritten, moved or removed; a file already in the package wins.
/// `migrated` with what was copied, or `nothing`.
pub fn migrate(ctx: &Context, folder: &str) -> Outcome {
    let places = ctx.places();
    let roots = places.roots();
    let doc = match read(folder) {
        Ok(d) => d,
        Err(words) => return Outcome::said("refused", words),
    };
    let system_name = doc.get("system").and_then(Value::as_str).unwrap_or("");
    let Some(system) = system_named(system_name) else { return head(&doc, Outcome::said("failed", format!("no system of this build is named {system_name}"))) };
    let game = doc.get("game").and_then(|g| claims::place(&roots, folder, g));
    let rel = doc.get("game").and_then(|g| g.get("path")).and_then(Value::as_str).unwrap_or("");
    let stem = path::file_name_without_extension(Style::Unix, &rel.replace('\\', "/")).to_string();
    let saves_root = ctx.tree.saves();
    let console = system.console.as_str();
    let mut from: Vec<(String, String)> = Vec::new();
    // Each file takes the package's own name in place of the game's, as the runtime names a packaged game's saves.
    let own = emusen_galaxia::saves::package_stem(folder).to_string();
    let renamed = |f: &str| format!("{own}{}", &path::file_name(Style::HOST, f)[stem.len()..]);
    let into_saves = |f: &str| path::combine_all(Style::HOST, &[folder, package::SAVES_FOLDER, &renamed(f)]);
    let mut save_folders = vec![path::combine(Style::HOST, &saves_root, console)];
    match system.id.as_str() {
        "snes" | "n64" => save_folders.push(saves_root.clone()),
        "md" => save_folders.push(path::combine(Style::HOST, &saves_root, "MD")),
        "mcd" => save_folders.push(path::combine(Style::HOST, &saves_root, "MCD")),
        _ => {}
    }
    for f in save_folders.iter().flat_map(|d| files_in(d, |n| save_of(n, &stem))) {
        from.push((into_saves(&f), f));
    }
    if matches!(system.id.as_str(), "nes" | "gb" | "gbc")
        && let Some(beside) = game.as_deref().and_then(|g| path::directory_name(Style::HOST, g))
    {
        for f in files_in(&beside, |n| n == format!("{stem}.srm")) {
            from.push((into_saves(&f), f));
        }
    }
    let state_folder = ctx.state_folder.clone().filter(|s| !s.trim().is_empty()).unwrap_or_else(|| ctx.tree.save_states());
    let mut states: Vec<(String, String)> = Vec::new();
    for f in files_in(&state_folder, |n| state_of(n, &stem)) {
        let to = path::combine_all(Style::HOST, &[folder, package::STATES_FOLDER, &renamed(&f)]);
        if f.ends_with(emusen_galaxia::saves::STATE_EXTENSION) {
            states.push((to.clone(), f.clone()));
        }
        from.push((to, f));
    }
    for (to, art) in art_of(ctx, &doc, folder, game.as_deref()) {
        from.push((to, art));
    }
    let lists = [cheats::game_list(Style::HOST, Some(console), game.as_deref().or(Some(rel))), cheats::game_list(Style::HOST, None, game.as_deref().or(Some(rel)))];
    if let Some(list) = lists.iter().flatten().map(|l| path::combine_all(Style::HOST, &[&ctx.tree.config(), &l.category, &l.file_name()])).find(|p| dotnet_files::file_exists(p)) {
        from.push((dotnet_files::join(folder, package::CHEATS_FILE), list));
    }
    let (mut copied, mut kept, mut failed) = (Vec::new(), 0u64, Vec::new());
    for (to, source) in from {
        if dotnet_files::file_exists(&to) || copied.iter().any(|(t, _): &(String, String)| *t == to) {
            kept += 1;
            continue;
        }
        match std::fs::read(&source).map_err(|e| e.to_string()).and_then(|b| emusen_galaxia::atomic::write(&to, &b)) {
            Ok(()) => copied.push((to, source)),
            Err(w) => failed.push(format!("{source}: {w}")),
        }
    }
    match settings_into(ctx, &system.id, folder, game.as_deref().unwrap_or(rel)) {
        Ok(Some((to, source))) => copied.push((to, source)),
        Ok(None) => {}
        Err(Some(w)) => failed.push(w),
        Err(None) => kept += 1,
    }
    let records = records_into(ctx, &states, &mut failed);
    let list = Value::Array(copied.iter().map(|(to, source)| obj(vec![("from", text(source)), ("to", text(&package_rel(ctx, to)))])).collect());
    let outcome = if !failed.is_empty() {
        Outcome::said("failed", format!("{} file(s) could not be copied: {}", failed.len(), failed.join("; ")))
    } else if copied.is_empty() && records.copied == 0 {
        Outcome::said("nothing", if kept > 0 { "everything was already in the package" } else { "the game has no saves, states, records, settings, art or cheats to copy" })
    } else {
        Outcome::new("migrated")
    };
    let outcome = outcome.with("copied", list).with("kept", Value::number(kept)).with("records", Value::number(records.copied));
    head(&doc, match records.unread {
        Some(why) => outcome.with("records_words", text(&format!("the states' records were not copied: {why}"))),
        None => outcome,
    })
}

/// What `pkg migrate` did with the states' records: how many it copied, and why none could be read, if so.
struct Records {
    copied: u64,
    unread: Option<String>,
}

/// Each copied state's record, from states.db (or the records.db it was copied from), written beside the state in the
/// package; a state already there takes its record only while it is still the same bytes as the one copied from
/// (EmuSen_GamePackages.md §18.4). Nothing is written to either database.
fn records_into(ctx: &Context, states: &[(String, String)], failed: &mut Vec<String>) -> Records {
    if states.is_empty() {
        return Records { copied: 0, unread: None };
    }
    let Some(sqlite) = ctx.sqlite.clone() else { return Records { copied: 0, unread: Some("this program has no SQLite loaded to read states.db with".into()) } };
    let library = ctx.tree.directory(emusen_galaxia::tree::Directory::Library).unwrap_or_default();
    let mut copied = 0;
    for (to, source) in states {
        let Some(at) = emusen_galaxia::saves::package_record_path(to) else { continue };
        if dotnet_files::file_exists(&at) || !dotnet_files::file_exists(to) || std::fs::read(to).ok() != std::fs::read(source).ok() {
            continue;
        }
        let Some(record) = emusen_crystal::states::kept_record(sqlite.clone(), &library, source) else { continue };
        match emusen_crystal::states::write_package_copy(to, &record) {
            Ok(()) => copied += 1,
            Err(w) => failed.push(format!("{source}'s record: {w}")),
        }
    }
    Records { copied, unread: None }
}

/// The game's own settings, the runtime's file of them, copied into the package's `settings.json`, with the crop
/// games.db holds for the game added as its `Crop` where the package's has none yet (§18.2): the file written and what
/// it came from; none for nothing to copy; `Err(None)` for a package that already has them, `Err(Some)` for a failure.
fn settings_into(ctx: &Context, system: &str, folder: &str, game: &str) -> Result<Option<(String, String)>, Option<String>> {
    let plain = emusen_crystal::settings::game_settings_path(&ctx.tree, system, game);
    let target = emusen_crystal::settings::package_settings_path(folder);
    let crop = crop_of(ctx, game).map(|c| [c.left, c.right, c.top, c.bottom].iter().map(|v| emusen_dotnet::json::format_f64(*v).unwrap_or_default()).collect::<Vec<_>>().join(" "));
    let has_plain = dotnet_files::file_exists(&plain);
    let has_crop = |values: &[(String, String)]| values.iter().any(|(k, _)| k == CROP_KEY);
    if dotnet_files::file_exists(&target) {
        let mut values = emusen_crystal::settings::values_at(&target);
        let Some(crop) = crop.filter(|_| !has_crop(&values)) else { return if has_plain { Err(None) } else { Ok(None) } };
        values.push((CROP_KEY.into(), crop));
        return emusen_crystal::settings::write_values(&target, &values).map(|()| Some((target, "games.db".into()))).map_err(Some);
    }
    match crop {
        None if has_plain => std::fs::read(&plain).map_err(|e| Some(format!("{plain}: {e}"))).and_then(|b| emusen_galaxia::atomic::write(&target, &b).map_err(Some)).map(|()| Some((target, plain))),
        None => Ok(None),
        Some(crop) => {
            let mut values = if has_plain { emusen_crystal::settings::values_at(&plain) } else { Vec::new() };
            if !has_crop(&values) {
                values.push((CROP_KEY.into(), crop));
            }
            emusen_crystal::settings::write_values(&target, &values).map(|()| Some((target, if has_plain { plain } else { "games.db".into() }))).map_err(Some)
        }
    }
}

/// The game's crop in its settings, `GameSettings.CropKey`.
const CROP_KEY: &str = "Crop";

/// The crop games.db holds for a game, read only; none where there is none, or no SQLite to read it with.
fn crop_of(ctx: &Context, game: &str) -> Option<emusen_library::records::Crop> {
    let sqlite = ctx.sqlite.clone()?;
    let library = ctx.tree.directory(emusen_galaxia::tree::Directory::Library).unwrap_or_default();
    let at = path::combine(Style::HOST, &library, "games.db");
    if !dotnet_files::file_exists(&at) {
        return None;
    }
    let db = emusen_sqlite::Connection::open_read_only(sqlite, &at).ok()?;
    emusen_library::records::crop(&db, &path::full_path(game)).ok().flatten()
}

/// The game's art in the media store, each type's file to copy and where in the package it goes, named by the
/// package's name (§18.3); a type the package already has a file of is left.
fn art_of(ctx: &Context, doc: &Value, folder: &str, game: Option<&str>) -> Vec<(String, String)> {
    let (Some(game), Some(system)) = (game, doc.get("system").and_then(Value::as_str)) else { return Vec::new() };
    let store = ctx.tree.directory(emusen_galaxia::tree::Directory::Media).unwrap_or_default();
    let named = emusen_scrape::media::Game { file: game, folder_path: emusen_galaxia::game_folders::of(ctx.library.as_deref(), game), folder: false };
    let mut found = Vec::new();
    for kind in emusen_scrape::kinds::ALL_FOLDER_ORDER {
        if emusen_scrape::media::find_in_package(folder, kind.esde_type).is_some() {
            continue;
        }
        if let Some(art) = emusen_scrape::media::find_in(&store, system, &named, kind.esde_type) {
            found.push((package::media_path(folder, kind.folder, path::extension(Style::HOST, &art)), art));
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_games_saves_and_states_are_told_from_another_games() {
        assert!(save_of("Zelda.srm", "Zelda") && save_of("Zelda.1.mcr", "Zelda") && save_of("Zelda.eep", "Zelda"));
        assert!(!save_of("Super Mario Bros. 3.srm", "Super Mario Bros") && !save_of("Zelda", "Zelda") && !save_of("Zeldas.srm", "Zelda") && !save_of("Zelda.a.b.c", "Zelda"));
        assert!(state_of("G.state", "G") && state_of("G.slot3.state", "G") && state_of("G.resume.png", "G") && state_of("G.png", "G"));
        assert!(!state_of("G.slot.state", "G") && !state_of("G 2.state", "G") && !state_of("G.srm", "G") && !state_of("G.slot3.json", "G"));
    }

    #[test]
    fn a_text_member_is_set_in_its_place() {
        let mut o = obj(vec![("a", Value::Null), ("b", Value::Null)]);
        claims::set(&mut o, "a", text("x"));
        claims::set(&mut o, "c", text("y"));
        let Value::Object(m) = &o else { panic!() };
        assert_eq!(m.iter().map(|(k, _)| k.text.as_str()).collect::<Vec<_>>(), ["a", "b", "c"]);
    }
}
