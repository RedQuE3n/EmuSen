//! A game package as one zip, to carry it whole: every file of the package's folder under one top folder named as the
//! package, each with its modification time to the tick, a CHD and what is already compressed stored and the rest
//! deflated. A package's zip is opened by unpacking it once into `Packages`, never over a folder that is there, its
//! times kept so that a state's record still names its state; the zip itself is only read. See
//! EmuSen_GamePackages.md §3.4 and §19, and EmuSen_Galaxia.md §3.13.

use crate::archive::{self, ZipWriter};
use crate::files;
use crate::path_key;
use std::path::{Path, PathBuf};
use emusen_dotnet::json::Value;
use crate::package;


/// A package's zip beside its folder: the folder's path with `.zip`.
pub fn zip_beside(folder: impl AsRef<Path>) -> PathBuf {
    let folder = folder.as_ref();
    folder.with_file_name(format!("{}.zip", folder.file_name().unwrap_or_default().to_string_lossy()))
}

/// A hidden name beside `at` for the copy being made, which is renamed over `at` only when whole.
fn partial_beside(at: &Path) -> PathBuf {
    at.with_file_name(format!(".{}.partial-{}", at.file_name().unwrap_or_default().to_string_lossy(), std::process::id()))
}

fn stored(name: &str) -> bool {
    let Some(extension) = Path::new(name).extension() else { return false };
    archive::STORED_EXTENSIONS.iter().any(|e| e.strip_prefix('.').is_some_and(|e| extension.eq_ignore_ascii_case(e)))
}

/// Every file of a package's folder, as zip names relative to it with `/` between their parts, in ordinal order with
/// the manifest first; hidden files and folders and links are passed over.
pub fn files_of(folder: &Path) -> Result<Vec<(String, PathBuf)>, String> {
    let mut found = Vec::new();
    collect(folder, "", &mut found)?;
    found.sort_by(|a, b| (a.0 != package::FILE_NAME).cmp(&(b.0 != package::FILE_NAME)).then_with(|| a.0.cmp(&b.0)));
    Ok(found)
}

fn collect(dir: &Path, prefix: &str, found: &mut Vec<(String, PathBuf)>) -> Result<(), String> {
    let listing = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for entry in listing.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if files::is_hidden(&name) {
            continue;
        }
        let Ok(kind) = entry.file_type() else { continue };
        let at = dir.join(&name);
        let rel = if prefix.is_empty() { name.clone() } else { format!("{prefix}/{name}") };
        if kind.is_dir() {
            collect(&at, &rel, found)?;
        } else if kind.is_file() {
            found.push((rel, at));
        }
    }
    Ok(())
}

/// The package at `folder` written as the zip `zip`, which must not be there: written beside it under a hidden name and
/// renamed into place only when whole. The number of files and the zip's bytes.
pub fn write(folder: impl AsRef<Path>, zip: impl AsRef<Path>, stop: &dyn Fn() -> bool) -> Result<(usize, u64), String> {
    let (folder, zip) = (folder.as_ref(), zip.as_ref());
    let shown = zip.display();
    if !files::file_exists(folder.join(package::FILE_NAME)) {
        return Err(format!("{} holds no {}", folder.display(), package::FILE_NAME));
    }
    if std::fs::symlink_metadata(zip).is_ok() {
        return Err(format!("{shown} is already there, and it was left as it is"));
    }
    let top = folder.file_name().unwrap_or_default().to_string_lossy().into_owned();
    let files = files_of(folder)?;
    let partial = partial_beside(zip);
    let _ = std::fs::remove_file(&partial);
    let written = (|| {
        let mut writer = ZipWriter::create(&partial)?;
        for (rel, at) in &files {
            if stop() {
                return Err("stopped".to_string());
            }
            let modified = std::fs::metadata(at).and_then(|m| m.modified()).map_err(|e| format!("{}: {e}", at.display()))?;
            writer.add(&format!("{top}/{rel}"), at, modified, stored(rel), stop)?;
        }
        writer.finish()
    })();
    if let Err(words) = written {
        let _ = std::fs::remove_file(&partial);
        return Err(words);
    }
    if std::fs::symlink_metadata(zip).is_ok() {
        let _ = std::fs::remove_file(&partial);
        return Err(format!("{shown} appeared while it was written, and it was left as it is"));
    }
    std::fs::rename(&partial, zip).map_err(|e| {
        let _ = std::fs::remove_file(&partial);
        format!("{shown}: {e}")
    })?;
    Ok((files.len(), std::fs::metadata(zip).map(|m| m.len()).unwrap_or(0)))
}

/// Whether a zip entry's name is one a package's zip may hold: under `top`, no part empty, `.` or `..`, no backslash,
/// colon or NUL, so that no entry can name a place outside the folder it is unpacked into.
fn safe(name: &str, top: &str) -> bool {
    let Some(rest) = name.strip_prefix(top).and_then(|r| r.strip_prefix('/')) else { return false };
    let rest = rest.strip_suffix('/').unwrap_or(rest);
    rest.is_empty() || (!rest.contains(['\\', ':', '\0']) && rest.split('/').all(|p| !p.is_empty() && p != "." && p != ".."))
}

/// The package a zip holds: its top folder's name and its manifest, read without unpacking; none for a zip that holds
/// no package (a game's own zip) or that this reader does not take, which every other reader takes as before; a
/// refusal for a package's zip this build will not unpack, with why.
pub fn manifest_in(zip: impl AsRef<Path>) -> Result<Option<(String, Value)>, String> {
    let (path, zip) = (zip.as_ref(), zip.as_ref().display());
    let Ok(mut reader) = archive::Zip::open(path) else { return Ok(None) };
    let Ok(entries) = reader.entries() else { return Ok(None) };
    let tops: Vec<&str> = entries.iter().filter_map(|e| e.full_name.strip_suffix(package::FILE_NAME).and_then(|t| t.strip_suffix('/'))).filter(|t| !t.is_empty() && !t.contains('/')).collect();
    let [top] = tops.as_slice() else { return Ok(None) };
    let top = top.to_string();
    if let Some(bad) = entries.iter().find(|e| !safe(&e.full_name, &top)) {
        return Err(format!("{zip} holds {}, which is not inside its package's folder {top}, so it was not unpacked", bad.full_name));
    }
    let manifest = entries.iter().find(|e| e.full_name == format!("{top}/{}", package::FILE_NAME)).ok_or("no manifest")?;
    let mut bytes = Vec::new();
    reader.copy(manifest, &mut bytes).map_err(|e| format!("{zip}: {e}"))?;
    let doc = package::parse(&crate::text_file::decode(&bytes)).map_err(|r| format!("{zip}: {}", r.words()))?;
    Ok(Some((top, doc)))
}

/// Where a package's zip is unpacked: beside it where it lies under `packages`, else straight under `packages`; never
/// beside a zip elsewhere, which may be in the library.
pub fn target(packages: impl AsRef<Path>, zip: impl AsRef<Path>, top: &str) -> PathBuf {
    let full = path_key::key(zip);
    let parent = full.parent().unwrap_or(Path::new(""));
    let packages = path_key::key(packages);
    if parent.starts_with(&packages) { parent.join(top) } else { packages.join(top) }
}

/// The zip unpacked into `target`, which must not be there: into a hidden folder beside it first, each entry checked
/// against its CRC-32 and given its time, then renamed into place whole.
pub fn unpack(zip: impl AsRef<Path>, top: &str, target: impl AsRef<Path>, stop: &dyn Fn() -> bool) -> Result<(), String> {
    let (zip, target) = (zip.as_ref(), target.as_ref());
    let (zip_shown, target_shown) = (zip.display(), target.display());
    if std::fs::symlink_metadata(target).is_ok() {
        return Err(format!("{target_shown} is already there, so {zip_shown} was not unpacked over it"));
    }
    let mut reader = archive::Zip::open(zip).map_err(|e| format!("{zip_shown}: {e}"))?;
    let entries = reader.entries().map_err(|e| format!("{zip_shown}: {e}"))?;
    let parent = target.parent().unwrap_or(Path::new(""));
    std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    let partial = partial_beside(target);
    let partial_shown = partial.display();
    if files::directory_exists(&partial) {
        std::fs::remove_dir_all(&partial).map_err(|e| format!("{partial_shown}: {e}"))?;
    }
    let done = (|| {
        std::fs::create_dir_all(&partial).map_err(|e| format!("{partial_shown}: {e}"))?;
        for e in &entries {
            if stop() {
                return Err("stopped".to_string());
            }
            if !safe(&e.full_name, top) {
                return Err(format!("{zip_shown} holds {}, which is not inside its package's folder {top}", e.full_name));
            }
            let rest = &e.full_name[top.len() + 1..];
            if rest.is_empty() || rest.ends_with('/') {
                std::fs::create_dir_all(crate::claims::under(&partial.to_string_lossy(), rest)).map_err(|x| x.to_string())?;
                continue;
            }
            let at = PathBuf::from(crate::claims::under(&partial.to_string_lossy(), rest));
            if let Some(dir) = at.parent() {
                std::fs::create_dir_all(dir).map_err(|x| x.to_string())?;
            }
            let mut out = std::fs::File::create(&at).map_err(|x| format!("{}: {x}", at.display()))?;
            reader.copy(e, &mut out).map_err(|w| format!("{zip_shown}: {w}"))?;
            if let Some(t) = reader.modified(e) {
                let _ = out.set_modified(t);
            }
            out.sync_all().map_err(|x| format!("{}: {x}", at.display()))?;
        }
        Ok(())
    })();
    if let Err(words) = done {
        let _ = std::fs::remove_dir_all(&partial);
        return Err(words);
    }
    if std::fs::symlink_metadata(target).is_ok() {
        let _ = std::fs::remove_dir_all(&partial);
        return Err(format!("{target_shown} appeared while {zip_shown} was unpacked, and was left as it is"));
    }
    std::fs::rename(&partial, target).map_err(|e| {
        let _ = std::fs::remove_dir_all(&partial);
        format!("{target_shown}: {e}")
    })
}

/// Whether two manifests describe the same game: the same recorded hashes of the game as it lies, else the same dump.
pub fn same_game(a: &Value, b: &Value) -> bool {
    match (crate::claims::recorded(a), crate::claims::recorded(b)) {
        (Some(x), Some(y)) => ["sha1", "md5", "crc32"].iter().find_map(|k| match (x.get(k).and_then(Value::as_str), y.get(k).and_then(Value::as_str)) {
            (Some(p), Some(q)) => Some(p.eq_ignore_ascii_case(q)),
            _ => None,
        }) == Some(true),
        _ => {
            let text = |v: Option<&Value>| {
                let mut s = String::new();
                if let Some(v) = v {
                    emusen_dotnet::json::write_compact(&mut s, v);
                }
                s
            };
            text(a.get("dump")) == text(b.get("dump")) && a.get("dump").is_some()
        }
    }
}

/// The package folder a zip opens as (EmuSen_GamePackages.md §19): none for a zip that holds no package; its folder
/// where that is there and holds the same game, the zip left unread beyond its manifest; else the zip unpacked there.
/// A folder there that holds another package, or none, is refused with words and nothing is written.
pub fn opened(packages: &str, zip: &str) -> Result<Option<String>, String> {
    let Some((top, doc)) = manifest_in(zip)? else { return Ok(None) };
    if packages.trim().is_empty() {
        return Err(format!("{zip} holds a package, and there is no Packages folder to unpack it into"));
    }
    let target = target(packages, zip, &top);
    let shown = target.display();
    if std::fs::symlink_metadata(&target).is_ok() {
        return match package::read(target.join(package::FILE_NAME)) {
            Ok(Ok(there)) if same_game(&there, &doc) => Ok(Some(target.to_string_lossy().into_owned())),
            Ok(Ok(_)) => Err(format!("{shown} holds another game's package, so {zip} was not unpacked over it")),
            _ => Err(format!("{shown} is there and holds no package this build reads, so {zip} was not unpacked over it")),
        };
    }
    unpack(zip, &top, &target, &|| false)?;
    Ok(Some(target.to_string_lossy().into_owned()))
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::test_support::TempDir;

    fn put(at: impl AsRef<Path>, bytes: &[u8]) {
        let at = at.as_ref();
        std::fs::create_dir_all(at.parent().unwrap()).unwrap();
        std::fs::write(at, bytes).unwrap();
    }

    #[test]
    fn a_package_zipped_and_unpacked_keeps_every_file_and_its_time_to_the_tick() {
        let t = TempDir::new("package-zip");
        let folder = t.text("Packages/SNES/Game.sfc");
        let manifest = r#"{"format":"emusen-package","version":"1.0","system":"snes","mode":"self-contained","game":{"role":"image","where":"package","path":"Game.sfc"},"dump":{"file":{"crc32":"00000000","md5":"00000000000000000000000000000000","sha1":"0000000000000000000000000000000000000000"}}}"#;
        put(&format!("{folder}/package.json"), manifest.as_bytes());
        let rom: Vec<u8> = (0..70_000u32).map(|i| (i % 7) as u8).collect();
        put(&format!("{folder}/Game.sfc"), &rom);
        put(&format!("{folder}/saves/Game.srm"), b"SAVEDATA-1234");
        put(&format!("{folder}/states/Game.slot2.state"), b"state");
        put(&format!("{folder}/media/covers/Game.png"), b"\x89PNG not really");
        put(&format!("{folder}/disc.chd"), b"MComprHD pretend");
        put(&format!("{folder}/.hidden"), b"x");
        let odd = std::time::UNIX_EPOCH + std::time::Duration::from_nanos(1_761_390_123_456_789_100);
        for f in ["Game.sfc", "states/Game.slot2.state"] {
            std::fs::File::options().write(true).open(format!("{folder}/{f}")).unwrap().set_modified(odd).unwrap();
        }
        let zip = zip_beside(&folder);
        assert_eq!(write(&folder, &zip, &|| false).unwrap().0, 6);
        assert!(write(&folder, &zip, &|| false).unwrap_err().contains("already there"));
        let entries = archive::entries_of(&zip).unwrap();
        let stored = |n: &str| entries.iter().find(|e| e.full_name == n).unwrap().stored;
        assert_eq!((stored("Game.sfc/Game.sfc"), stored("Game.sfc/disc.chd"), stored("Game.sfc/media/covers/Game.png")), (false, true, true));
        assert_eq!(entries[0].full_name, "Game.sfc/package.json");
        // Opened where its folder is: the folder itself, the zip only read.
        assert_eq!(opened(&t.text("Packages"), &zip.to_string_lossy()).unwrap().as_deref(), Some(folder.as_str()));
        // Carried to another machine: unpacked once beside it, every byte and time as it was.
        let elsewhere = t.text("Other/Packages/SNES/Game.sfc.zip");
        put(&elsewhere, &std::fs::read(&zip).unwrap());
        let unpacked = opened(&t.text("Other/Packages"), &elsewhere).unwrap().unwrap();
        assert_eq!(unpacked, t.text("Other/Packages/SNES/Game.sfc"));
        assert_eq!(std::fs::read(format!("{unpacked}/Game.sfc")).unwrap(), rom);
        assert_eq!(std::fs::metadata(format!("{unpacked}/states/Game.slot2.state")).unwrap().modified().unwrap(), odd);
        assert!(!std::path::Path::new(&format!("{unpacked}/.hidden")).exists());
        // A zip outside Packages goes straight under it; a folder there of another game refuses it.
        let outside = t.text("Downloads/Game.sfc.zip");
        put(&outside, &std::fs::read(&zip).unwrap());
        assert_eq!(opened(&t.text("Third"), &outside).unwrap().unwrap(), t.text("Third/Game.sfc"));
        std::fs::write(t.text("Third/Game.sfc/package.json"), manifest.replace("00000000\",\"md5", "11111111\",\"md5").replace("\"sha1\":\"0000000000000000000000000000000000000000\"", "\"sha1\":\"1111111111111111111111111111111111111111\"")).unwrap();
        assert!(opened(&t.text("Third"), &outside).unwrap_err().contains("another game's package"));
        // A damaged entry, the stored CHD's, is refused and nothing is left.
        let mut bytes = std::fs::read(&zip).unwrap();
        let at = bytes.windows(16).position(|w| w == b"MComprHD pretend").unwrap();
        bytes[at] = b'X';
        let damaged = t.text("Fourth/Packages/Game.sfc.zip");
        put(&damaged, &bytes);
        assert!(opened(&t.text("Fourth/Packages"), &damaged).unwrap_err().contains("is damaged"));
        assert!(!std::path::Path::new(&t.text("Fourth/Packages/Game.sfc")).exists());
        // A game's own zip is no package.
        let plain = t.text("plain.zip");
        let mut w = ZipWriter::create(&plain).unwrap();
        w.add("inside.sfc", &format!("{folder}/Game.sfc"), odd, false, &|| false).unwrap();
        w.finish().unwrap();
        assert_eq!(opened(&t.text("Packages"), &plain).unwrap(), None);
        assert_eq!(archive::read_entry(&plain, "inside.sfc", usize::MAX).unwrap(), rom);
    }

    #[test]
    fn an_entry_that_climbs_out_of_its_folder_is_refused() {
        assert!(safe("Game.sfc/saves/a.srm", "Game.sfc"));
        assert!(safe("Game.sfc/", "Game.sfc"));
        for bad in ["Game.sfc/../x", "Other/x", "Game.sfc/a\\..\\b", "Game.sfc//x", "Game.sfc/./x", "/Game.sfc/x"] {
            assert!(!safe(bad, "Game.sfc"), "{bad}");
        }
    }
}
