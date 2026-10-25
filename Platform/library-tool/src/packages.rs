//! Game packages made, listed, checked and given the stores a game already had: `pkg make`, `pkg ls`, `pkg check` and
//! `pkg migrate` (EmuSen_GamePackages.md §3.4–§3.7, §14). Linked packages only: the library is read and never
//! written, a package is made only when asked, and migrating copies and never overwrites or removes anything.

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

/// A linked package made for one library game, unless a package already claims it (§3.4, §9 Q6).
pub fn make(ctx: &Context, index: &mut Index, game: &Game, stop: &dyn Fn() -> bool) -> Outcome {
    let places = ctx.places();
    let roots = places.roots();
    let full = path::full_path(&game.path);
    if let Some(p) = index.claimant(&full) {
        return Outcome::new("already").with("package", text(&package_rel(ctx, &p.folder)));
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
    Outcome::new("made").with("package", text(&folder_rel)).with("size", Value::number(claims::seen_of(&game.path).map_or(0, |s| s.0)))
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
        let found = claims::recorded(&doc).and_then(|r| claims::recorded_size(&doc).and_then(|size| claims::find_in_library(&roots, size, r)));
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
        let found = claims::recorded(&doc).cloned().and_then(|r| claims::recorded_size(&doc).and_then(|size| claims::find_in_library(&roots, size, &r)));
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
/// pictures from the state folder, and its cheat list. Nothing is overwritten, moved or removed; a file already in the
/// package wins. `migrated` with what was copied, or `nothing`.
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
    for f in files_in(&state_folder, |n| state_of(n, &stem)) {
        from.push((path::combine_all(Style::HOST, &[folder, package::STATES_FOLDER, &renamed(&f)]), f));
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
    let list = Value::Array(copied.iter().map(|(to, source)| obj(vec![("from", text(source)), ("to", text(&package_rel(ctx, to)))])).collect());
    let outcome = if !failed.is_empty() {
        Outcome::said("failed", format!("{} file(s) could not be copied: {}", failed.len(), failed.join("; ")))
    } else if copied.is_empty() {
        Outcome::said("nothing", if kept > 0 { "everything was already in the package" } else { "the game has no saves, states or cheats to copy" })
    } else {
        Outcome::new("migrated")
    };
    head(&doc, outcome.with("copied", list).with("kept", Value::number(kept)))
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
