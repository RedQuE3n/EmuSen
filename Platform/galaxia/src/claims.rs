//! The claim index of game packages: every package under `Packages`, the files each claims (its game, its companions
//! and the library files it was made from), and a game file opened by its plain path resolved to the package that
//! claims it, so that both routes reach one save. A linked game that moved in the library is found again by its size
//! and then its hashes, and the manifest names the new place. The manifests are the index: nothing else is kept, so
//! the index cannot disagree with them. The C# `GamePackages` is the twin. See EmuSen_GamePackages.md §3.4–§3.6 and
//! §14, and EmuSen_Galaxia.md §3.11.

use crate::dotnet_files::{self, Options};
use crate::dotnet_path::{self as path, Style};
use crate::json::{Text, Value};
use crate::package;
use crate::rom_hash::{self, Hashes};

const STYLE: Style = Style::HOST;

/// How deep the walk of `Packages` looks for a package's folder.
pub const MAX_DEPTH: usize = 32;

/// The three folders a package's paths are relative to: `Packages`, `AppSettings.RomDirectory` and `Compressed`.
#[derive(Clone, Copy, Debug)]
pub struct Roots<'a> {
    pub packages: &'a str,
    pub library: Option<&'a str>,
    pub compressed: &'a str,
}

/// The three folders as a program has them: its tree's `Packages` and `Compressed`, and the library its settings name.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Places {
    pub packages: String,
    pub library: Option<String>,
    pub compressed: String,
}

impl Places {
    /// The tree's folders, and `AppSettings.RomDirectory` as the settings file holds it, checked as a folder setting is.
    pub fn of(tree: &crate::tree::Tree) -> Places {
        use crate::models::{APP_SETTINGS_FILE, APP_SETTINGS_ID, MODELS};
        let (settings, _) = crate::config::load_or_new(tree, &MODELS[APP_SETTINGS_ID as usize], None, APP_SETTINGS_FILE);
        let library = crate::setting_folders::checked(settings.get("RomDirectory").and_then(|v| v.as_str()), &crate::tree::user_home()).path;
        let folder = |which| tree.directory(which).unwrap_or_default();
        Places { packages: folder(crate::tree::Directory::Packages), library, compressed: folder(crate::tree::Directory::Compressed) }
    }

    pub fn roots(&self) -> Roots<'_> {
        Roots { packages: &self.packages, library: self.library.as_deref(), compressed: &self.compressed }
    }
}

/// One package the index holds: its folder, its system, its game and every file it claims, each a full path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Package {
    pub folder: String,
    pub system: String,
    pub game: String,
    pub claims: Vec<String>,
}

/// Every folder under `Packages` holding a manifest, refused ones included, and the packages whose manifests read.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Index {
    pub folders: Vec<String>,
    pub packages: Vec<Package>,
}

impl Index {
    /// The package claiming the file at `full`, a full path.
    pub fn claimant(&self, full: &str) -> Option<&Package> {
        self.packages.iter().find(|p| p.claims.iter().any(|c| c == full))
    }

    /// Whether `full` lies inside a package's folder.
    pub fn inside_package(&self, full: &str) -> bool {
        self.folders.iter().any(|f| within(full, f))
    }
}

/// Whether `full` is `folder` or lies under it.
pub fn within(full: &str, folder: &str) -> bool {
    let folder = path::trim_ending_separator(STYLE, folder);
    full == folder || full.strip_prefix(folder).is_some_and(|rest| rest.as_bytes().first().is_some_and(|&b| STYLE.is_separator(b)))
}

/// A path relative to a root, `/` or `\` between its parts, under that root as `Path.Combine` joins each part.
pub fn under(root: &str, rel: &str) -> String {
    rel.split(['/', '\\']).filter(|p| !p.is_empty()).fold(root.to_string(), |at, part| path::combine(STYLE, &at, part))
}

/// The full path an entry's `where` and `path` name, none for `library` with no library set.
pub fn place(roots: &Roots, folder: &str, entry: &Value) -> Option<String> {
    let rel = entry.get("path")?.as_str()?;
    let at = match entry.get("where")?.as_str()? {
        "library" => under(roots.library?, rel),
        "compressed" => under(roots.compressed, rel),
        "package" => under(folder, rel),
        "absolute" => rel.to_string(),
        _ => return None,
    };
    Some(path::full_path(&at))
}

/// Every package folder under `packages`: a folder holding `package.json` is one, and is not looked inside; hidden
/// folders are passed over; each folder's subfolders are taken in ordinal order of their names.
pub fn folders(packages: &str) -> Vec<String> {
    let mut found = Vec::new();
    walk(packages, 0, &mut found);
    found
}

fn walk(dir: &str, depth: usize, found: &mut Vec<String>) {
    let Ok(listing) = std::fs::read_dir(dir) else { return };
    let mut names: Vec<String> = listing
        .flatten()
        .filter(dotnet_files::listed_as_directory)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| !dotnet_files::is_hidden(n))
        .collect();
    names.sort();
    for name in names {
        let sub = dotnet_files::join(dir, &name);
        if dotnet_files::file_exists(&dotnet_files::join(&sub, package::FILE_NAME)) {
            found.push(sub);
        } else if depth + 1 < MAX_DEPTH {
            walk(&sub, depth + 1, found);
        }
    }
}

fn entries(doc: &Value, name: &str) -> Vec<Value> {
    match doc.get(name) {
        Some(Value::Array(items)) => items.clone(),
        _ => Vec::new(),
    }
}

/// The package a manifest describes, with its claims in the order game, companions, the container's sources.
pub fn package_of(roots: &Roots, folder: &str, doc: &Value) -> Option<Package> {
    let game = place(roots, folder, doc.get("game")?)?;
    let mut claims = vec![game.clone()];
    let sources = doc.get("container").map(|c| entries(c, "sources")).unwrap_or_default();
    for entry in entries(doc, "companions").iter().chain(sources.iter()) {
        if let Some(at) = place(roots, folder, entry).filter(|at| !claims.contains(at)) {
            claims.push(at);
        }
    }
    Some(Package { folder: folder.to_string(), system: doc.get("system")?.as_str()?.to_string(), game, claims })
}

/// The index: every package folder, and the packages whose manifests this build reads.
pub fn index(roots: &Roots) -> Index {
    let folders = folders(roots.packages);
    let packages = folders
        .iter()
        .filter_map(|f| match package::read(&dotnet_files::join(f, package::FILE_NAME)) {
            Ok(Ok(doc)) => package_of(roots, f, &doc),
            _ => None,
        })
        .collect();
    Index { folders, packages }
}

/// A file's size and modification time as a manifest's `seen` holds them: UTC to the second.
pub fn seen_of(at: &str) -> Option<(u64, String)> {
    let meta = std::fs::metadata(at).ok().filter(|m| m.is_file())?;
    Some((meta.len(), mtime_text(meta.modified().ok()?)))
}

/// A time as the manifest's `mtime`: UTC to the second, `yyyy-MM-ddTHH:mm:ssZ`.
pub fn mtime_text(t: std::time::SystemTime) -> String {
    let secs = t.duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
    let (y, m, d) = emusen_dotnet::date::civil(secs.div_euclid(86_400));
    let s = secs.rem_euclid(86_400);
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z", s / 3600, s / 60 % 60, s % 60)
}

fn number(v: Option<&Value>) -> Option<u64> {
    match v {
        Some(Value::Number(t)) => t.parse().ok(),
        _ => None,
    }
}

/// The hashes a manifest records for its game: `dump.file`, else the sheet's for a disc whose game is its sheet; none
/// for a game held in a container, whose dump is the original's and not the container's (the CHD tool checks those).
pub fn recorded(doc: &Value) -> Option<&Value> {
    if matches!(doc.get("container"), Some(Value::Object(_))) {
        return None;
    }
    let dump = doc.get("dump")?;
    [dump.get("file"), dump.get("sheet")].into_iter().flatten().find(|h| matches!(h, Value::Object(_)) && ["sha1", "md5", "crc32"].iter().any(|k| h.get(k).and_then(Value::as_str).is_some()))
}

/// Whether hashes taken of a file are the recorded ones: SHA-1 where both have it, else MD5, else CRC-32.
pub fn same_dump(recorded: &Value, taken: &Hashes) -> bool {
    let want = |k: &str| recorded.get(k).and_then(Value::as_str).map(str::to_ascii_lowercase);
    if let (Some(w), Some(t)) = (want("sha1"), taken.sha1.as_deref()) {
        return w == t;
    }
    if let Some(w) = want("md5") {
        return w == taken.md5;
    }
    want("crc32").is_some_and(|w| w == taken.crc32)
}

/// The words for a recorded hash: its SHA-1, else its MD5, else its CRC-32, named.
pub fn named(recorded: &Value) -> String {
    ["sha1", "md5", "crc32"].iter().find_map(|k| recorded.get(k).and_then(Value::as_str).map(|h| format!("{} {}", k.to_uppercase().replace("CRC32", "CRC-32").replace("SHA1", "SHA-1"), h.to_ascii_lowercase()))).unwrap_or_default()
}

fn taken_named(recorded: &Value, taken: &Hashes) -> String {
    match (recorded.get("sha1").and_then(Value::as_str), taken.sha1.as_deref()) {
        (Some(_), Some(s)) => format!("SHA-1 {s}"),
        _ if recorded.get("md5").and_then(Value::as_str).is_some() => format!("MD5 {}", taken.md5),
        _ => format!("CRC-32 {}", taken.crc32),
    }
}

/// The recorded size of a manifest's game: `game.size`, else `game.seen.size`.
pub fn recorded_size(doc: &Value) -> Option<u64> {
    let game = doc.get("game")?;
    number(game.get("size")).or_else(|| number(game.get("seen").and_then(|s| s.get("size"))))
}

fn hash(at: &str) -> Option<Hashes> {
    rom_hash::hashes_with(at, None, true, &|| false).ok()
}

/// Sets a member of an object, in its place where it is there, else at the end.
pub fn set(object: &mut Value, name: &str, value: Value) {
    if let Value::Object(members) = object {
        match members.iter_mut().rev().find(|(k, _)| k.text == name) {
            Some((_, v)) => *v = value,
            None => members.push((Text::new(name), value)),
        }
    }
}

fn member_mut<'a>(object: &'a mut Value, name: &str) -> Option<&'a mut Value> {
    match object {
        Value::Object(members) => members.iter_mut().rev().find(|(k, _)| k.text == name).map(|(_, v)| v),
        _ => None,
    }
}

fn seen_value(size: u64, mtime: &str) -> Value {
    Value::Object(vec![(Text::new("size"), Value::number(size)), (Text::new("mtime"), Value::string(mtime))])
}

/// Where a file is as an entry names it: `library` and its path below the library where it lies there, else `absolute`.
pub fn entry_place(roots: &Roots, full: &str) -> (&'static str, String) {
    if let Some(library) = roots.library {
        let root = path::full_path(library);
        if within(full, &root) && full.len() > path::trim_ending_separator(STYLE, &root).len() {
            let rest = &full[path::trim_ending_separator(STYLE, &root).len()..];
            return ("library", rest.trim_start_matches(['/', '\\']).replace('\\', "/"));
        }
    }
    ("absolute", full.to_string())
}

/// The manifest given its game's new place: `where`, `path` and `seen`, and each companion that lay under the game's
/// old folder moved under its new one, as a sheet's tracks move with it.
pub fn moved(roots: &Roots, doc: &mut Value, found: &str) {
    let (place, rel) = entry_place(roots, found);
    let old = doc.get("game").and_then(|g| g.get("path")).and_then(Value::as_str).map(str::to_string).unwrap_or_default();
    let old_where = doc.get("game").and_then(|g| g.get("where")).and_then(Value::as_str).map(str::to_string).unwrap_or_default();
    if let Some(game) = member_mut(doc, "game") {
        set(game, "where", Value::string(place));
        set(game, "path", Value::string(&rel));
        if let Some((size, mtime)) = seen_of(found) {
            set(game, "seen", seen_value(size, &mtime));
        }
    }
    let folder_of = |p: &str| p.rsplit_once(['/', '\\']).map(|(d, _)| d.to_string());
    let (Some(old_dir), new_dir) = (folder_of(&old), folder_of(&rel)) else { return };
    if let Some(Value::Array(items)) = member_mut(doc, "companions") {
        for c in items.iter_mut() {
            let same_root = c.get("where").and_then(Value::as_str) == Some(old_where.as_str());
            let Some(p) = c.get("path").and_then(Value::as_str).map(str::to_string) else { continue };
            if let (true, Some(tail)) = (same_root, p.strip_prefix(&format!("{old_dir}/"))) {
                let rebased = match &new_dir {
                    Some(d) => format!("{d}/{tail}"),
                    None => tail.to_string(),
                };
                set(c, "where", Value::string(place));
                set(c, "path", Value::string(if place == "absolute" { under(path::directory_name(STYLE, found).as_deref().unwrap_or(""), tail) } else { rebased }));
            }
        }
    }
}

/// The library's files of `size` bytes, in ordinal order of their paths, the one of `recorded`'s hashes first found;
/// none where there is no library or no such file.
pub fn find_in_library(roots: &Roots, size: u64, recorded: &Value) -> Option<String> {
    let library = roots.library?;
    let mut same: Vec<String> = Vec::new();
    let _ = dotnet_files::visit_files_with(library, Options { skip_hidden: true, ignore_inaccessible: true }, |at, _| {
        if std::fs::metadata(at).is_ok_and(|m| m.len() == size) {
            same.push(path::full_path(at));
        }
    });
    same.sort();
    same.into_iter().find(|at| hash(at).is_some_and(|h| same_dump(recorded, &h)))
}

/// What a path opens as: the game file to open, and the package whose saves, states and cheats it uses, none for a
/// game no package claims; `moved` where the manifest was rewritten to name the game's new place.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Resolution {
    pub game: String,
    pub package: Option<String>,
    pub moved: bool,
}

/// A package's game, checked as §3.4 has it: a game whose size and time are those recorded opens as it is; one that
/// changed is hashed, its time recorded when it is still the dump and refused with both hashes named when it is not;
/// one that is gone is looked for in the library by size and then hashes, and the manifest names where it was found.
/// `opened` is a claimed file the player opened, played with the package's stores where the package's own game is gone.
pub fn open_package(roots: &Roots, folder: &str, opened: Option<&str>) -> Result<Resolution, String> {
    let manifest = dotnet_files::join(folder, package::FILE_NAME);
    let mut doc = match package::read(&manifest) {
        Ok(Ok(doc)) => doc,
        Ok(Err(refusal)) => return Err(format!("{manifest}: {}", refusal.words())),
        Err(e) => return Err(format!("{manifest}: {}", emusen_dotnet::file::io_message(&e, &manifest))),
    };
    let game = doc.get("game").and_then(|g| place(roots, folder, g)).ok_or_else(|| format!("{manifest}: its game's library is not set (AppSettings.RomDirectory)"))?;
    let resolution = |game: String, moved: bool| Resolution { game, package: Some(folder.to_string()), moved };
    if let Some((size, mtime)) = seen_of(&game) {
        let seen = doc.get("game").and_then(|g| g.get("seen"));
        let unchanged = seen.is_none() || (number(seen.and_then(|s| s.get("size"))) == Some(size) && seen.and_then(|s| s.get("mtime")).and_then(Value::as_str) == Some(mtime.as_str()));
        if unchanged {
            return Ok(resolution(game, false));
        }
        let Some(recorded) = recorded(&doc).cloned() else { return Ok(resolution(game, false)) };
        let taken = hash(&game).ok_or_else(|| format!("{game} could not be read to compare it with the package {folder}"))?;
        if !same_dump(&recorded, &taken) {
            return Err(format!(
                "{game} has {}, but the package {folder} was made for {}: it is not the dump the package's saves belong to. pkg check says more; pkg make makes a package for this file once the old one is moved aside.",
                taken_named(&recorded, &taken),
                named(&recorded)
            ));
        }
        if let Some(g) = member_mut(&mut doc, "game") {
            set(g, "seen", seen_value(size, &mtime));
        }
        let _ = package::write(&manifest, &doc);
        return Ok(resolution(game, false));
    }
    let where_ = doc.get("game").and_then(|g| g.get("where")).and_then(Value::as_str).unwrap_or("");
    if let (true, Some(size), Some(recorded)) = (where_ == "library" || where_ == "absolute", recorded_size(&doc), recorded(&doc).cloned())
        && let Some(found) = find_in_library(roots, size, &recorded)
    {
        moved(roots, &mut doc, &found);
        let _ = package::write(&manifest, &doc);
        return Ok(resolution(found, true));
    }
    match opened {
        Some(at) if dotnet_files::file_exists(at) => Ok(resolution(at.to_string(), false)),
        _ => Err(format!("The package {folder} names {game}, which is gone, and no file of its size and hashes was found in the library.")),
    }
}

/// What `at` opens as (§3.5, "one game, one save"): a package's folder or manifest opens its game; a file a package
/// claims opens that package's game with its stores; a file no package claims, whose size and hashes are those of a
/// package's game that is gone, is that game moved, and the manifest is rewritten; anything else opens as it is.
pub fn resolve(roots: &Roots, at: &str) -> Result<Resolution, String> {
    let full = path::full_path(at);
    if dotnet_files::directory_exists(&full) && dotnet_files::file_exists(&dotnet_files::join(&full, package::FILE_NAME)) {
        return open_package(roots, &full, None);
    }
    if path::file_name(STYLE, &full) == package::FILE_NAME && dotnet_files::file_exists(&full) {
        return open_package(roots, &path::directory_name(STYLE, &full).unwrap_or_default(), None);
    }
    let plain = || Ok(Resolution { game: at.to_string(), package: None, moved: false });
    if !dotnet_files::file_exists(&full) {
        return plain();
    }
    let index = index(roots);
    if let Some(p) = index.claimant(&full) {
        return open_package(roots, &p.folder.clone(), Some(&full));
    }
    let Some(size) = std::fs::metadata(&full).ok().map(|m| m.len()) else { return plain() };
    let mut taken: Option<Hashes> = None;
    for p in index.packages.iter().filter(|p| !dotnet_files::file_exists(&p.game)) {
        let manifest = dotnet_files::join(&p.folder, package::FILE_NAME);
        let Ok(Ok(mut doc)) = package::read(&manifest) else { continue };
        let where_ = doc.get("game").and_then(|g| g.get("where")).and_then(Value::as_str).unwrap_or("");
        if !(where_ == "library" || where_ == "absolute") || recorded_size(&doc) != Some(size) {
            continue;
        }
        let Some(recorded) = recorded(&doc).cloned() else { continue };
        if taken.is_none() {
            taken = hash(&full);
        }
        if taken.as_ref().is_some_and(|t| same_dump(&recorded, t)) {
            moved(roots, &mut doc, &full);
            let _ = package::write(&manifest, &doc);
            return Ok(Resolution { game: full, package: Some(p.folder.clone()), moved: true });
        }
    }
    plain()
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::test_support::TempDir;

    fn put(at: &str, bytes: &[u8]) {
        std::fs::create_dir_all(path::directory_name(STYLE, at).unwrap()).unwrap();
        std::fs::write(at, bytes).unwrap();
    }

    fn linked(t: &TempDir, folder: &str, rel: &str, bytes: &[u8], file: &str) -> String {
        let h = rom_hash::hashes_of(bytes);
        let (size, mtime) = seen_of(file).unwrap();
        let text = format!(
            r#"{{"format":"emusen-package","version":"1.0","system":"snes","mode":"linked","game":{{"role":"image","where":"library","path":"{rel}","size":{size},"seen":{{"size":{size},"mtime":"{mtime}"}}}},"dump":{{"file":{{"crc32":"{}","md5":"{}","sha1":"{}"}}}}}}"#,
            h.crc32,
            h.md5,
            h.sha1.unwrap()
        );
        let at = t.join(&format!("Packages/{folder}"));
        put(&format!("{at}/package.json"), text.as_bytes());
        at
    }

    #[test]
    fn a_claimed_file_resolves_to_its_package_and_a_moved_one_is_found_again_by_hash() {
        let t = TempDir::new("claims");
        let (lib, packages, compressed) = (t.join("Roms"), t.join("Packages"), t.join("Compressed"));
        let roots = Roots { packages: &packages, library: Some(&lib), compressed: &compressed };
        let bytes = b"a cartridge of sixteen bytes!!!!";
        let game = t.join("Roms/SNES/Game.sfc");
        put(&game, bytes);
        put(&t.join("Roms/SNES/Other.sfc"), b"another cartridge, same length..");
        let folder = linked(&t, "SNES/Game.sfc", "SNES/Game.sfc", bytes, &game);
        put(&t.join("Packages/SNES/Game.sfc/saves/Game.srm"), b"save");
        put(&t.join("Packages/.hidden/x/package.json"), b"{}");
        let idx = index(&roots);
        assert_eq!(idx.folders, vec![folder.clone()]);
        assert_eq!(idx.packages[0].claims, vec![game.clone()]);
        assert!(idx.inside_package(&t.join("Packages/SNES/Game.sfc/saves/Game.srm")));
        assert!(!idx.inside_package(&t.join("Packages/SNES/Game.sfcx")));
        let r = resolve(&roots, &game).unwrap();
        assert_eq!((r.game.as_str(), r.package.as_deref(), r.moved), (game.as_str(), Some(folder.as_str()), false));
        assert_eq!(resolve(&roots, &folder).unwrap().package.as_deref(), Some(folder.as_str()));
        assert_eq!(resolve(&roots, &t.join("Roms/SNES/Other.sfc")).unwrap().package, None);
        // Moved: the plain path at its new place finds the package by size, then hash, and the manifest follows it.
        let elsewhere = t.join("Roms/SNES/USA/Game (U).sfc");
        std::fs::create_dir_all(t.join("Roms/SNES/USA")).unwrap();
        std::fs::rename(&game, &elsewhere).unwrap();
        let r = resolve(&roots, &elsewhere).unwrap();
        assert_eq!((r.game.as_str(), r.package.as_deref(), r.moved), (elsewhere.as_str(), Some(folder.as_str()), true));
        let doc = package::read(&format!("{folder}/package.json")).unwrap().unwrap();
        assert_eq!(doc.get("game").unwrap().get("path").unwrap().as_str(), Some("SNES/USA/Game (U).sfc"));
        // Moved again, and opened by its package: the library is searched.
        std::fs::rename(&elsewhere, &game).unwrap();
        let r = resolve(&roots, &folder).unwrap();
        assert_eq!((r.game.as_str(), r.moved), (game.as_str(), true));
    }

    #[test]
    fn a_changed_game_is_refused_with_both_hashes_and_a_touched_one_is_recorded() {
        let t = TempDir::new("claims-changed");
        let (lib, packages, compressed) = (t.join("Roms"), t.join("Packages"), t.join("Compressed"));
        let roots = Roots { packages: &packages, library: Some(&lib), compressed: &compressed };
        let bytes = b"sixteen bytes!!!";
        let game = t.join("Roms/NES/G.nes");
        put(&game, bytes);
        let folder = linked(&t, "NES/G.nes", "NES/G.nes", bytes, &game);
        let later = std::time::SystemTime::now() + std::time::Duration::from_secs(7200);
        std::fs::File::options().write(true).open(&game).unwrap().set_modified(later).unwrap();
        assert!(!resolve(&roots, &game).unwrap().moved);
        let doc = package::read(&format!("{folder}/package.json")).unwrap().unwrap();
        assert_eq!(doc.get("game").unwrap().get("seen").unwrap().get("mtime").unwrap().as_str(), Some(mtime_text(later).as_str()));
        std::fs::write(&game, b"sixteen bytes?!!").unwrap();
        let words = resolve(&roots, &game).unwrap_err();
        assert!(words.contains("has SHA-1") && words.contains("was made for SHA-1"), "{words}");
        std::fs::remove_file(&game).unwrap();
        assert!(resolve(&roots, &folder).unwrap_err().contains("which is gone"));
    }
}
