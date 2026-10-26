//! A game package as one zip, to carry it whole: every file of the package's folder under one top folder named as the
//! package, each with its modification time to the tick, a CHD and what is already compressed stored and the rest
//! deflated. A package's zip is opened by unpacking it once into `Packages`, never over a folder that is there, its
//! times kept so that a state's record still names its state; the zip itself is only read. See
//! EmuSen_GamePackages.md §3.4 and §19, and EmuSen_Galaxia.md §3.13.

use crate::archive::{self, ZipEntry, ZipWriter};
use crate::dotnet_files;
use crate::dotnet_path::{self as path, Style};
use crate::json::Value;
use crate::package;

const STYLE: Style = Style::HOST;

/// A package's zip beside its folder: the folder's path with `.zip`.
pub fn zip_beside(folder: &str) -> String {
    format!("{}.zip", path::trim_ending_separator(STYLE, folder))
}

fn stored(name: &str) -> bool {
    let extension = path::extension(Style::Unix, name);
    archive::STORED_EXTENSIONS.iter().any(|e| emusen_dotnet::text::equals_ignore_case(e, extension))
}

/// Every file of a package's folder, as zip names relative to it with `/` between their parts, in ordinal order with
/// the manifest first; hidden files and folders and links are passed over.
pub fn files_of(folder: &str) -> Result<Vec<(String, String)>, String> {
    let mut found = Vec::new();
    collect(folder, "", &mut found)?;
    found.sort_by(|a, b| (a.0 != package::FILE_NAME).cmp(&(b.0 != package::FILE_NAME)).then_with(|| a.0.cmp(&b.0)));
    Ok(found)
}

fn collect(dir: &str, prefix: &str, found: &mut Vec<(String, String)>) -> Result<(), String> {
    let listing = std::fs::read_dir(dir).map_err(|e| format!("{dir}: {e}"))?;
    for entry in listing.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if dotnet_files::is_hidden(&name) {
            continue;
        }
        let Ok(kind) = entry.file_type() else { continue };
        let at = dotnet_files::join(dir, &name);
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
pub fn write(folder: &str, zip: &str, stop: &dyn Fn() -> bool) -> Result<(usize, u64), String> {
    let folder = path::trim_ending_separator(STYLE, folder).to_string();
    if !dotnet_files::file_exists(&dotnet_files::join(&folder, package::FILE_NAME)) {
        return Err(format!("{folder} holds no {}", package::FILE_NAME));
    }
    if std::fs::symlink_metadata(zip).is_ok() {
        return Err(format!("{zip} is already there, and it was left as it is"));
    }
    let top = path::file_name(STYLE, &folder).to_string();
    let files = files_of(&folder)?;
    let partial = path::combine(STYLE, &path::directory_name(STYLE, zip).unwrap_or_default(), &format!(".{}.partial-{}", path::file_name(STYLE, zip), std::process::id()));
    let _ = std::fs::remove_file(&partial);
    let written = (|| {
        let mut writer = ZipWriter::create(&partial)?;
        for (rel, at) in &files {
            if stop() {
                return Err("stopped".to_string());
            }
            let modified = std::fs::metadata(at).and_then(|m| m.modified()).map_err(|e| format!("{at}: {e}"))?;
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
        return Err(format!("{zip} appeared while it was written, and it was left as it is"));
    }
    std::fs::rename(&partial, zip).map_err(|e| {
        let _ = std::fs::remove_file(&partial);
        format!("{zip}: {e}")
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
pub fn manifest_in(zip: &str) -> Result<Option<(String, Value)>, String> {
    let Ok(entries) = archive::entries_of(zip) else { return Ok(None) };
    let tops: Vec<&str> = entries.iter().filter_map(|e| e.full_name.strip_suffix(package::FILE_NAME).and_then(|t| t.strip_suffix('/'))).filter(|t| !t.is_empty() && !t.contains('/')).collect();
    let [top] = tops.as_slice() else { return Ok(None) };
    let top = top.to_string();
    if let Some(bad) = entries.iter().find(|e| !safe(&e.full_name, &top)) {
        return Err(format!("{zip} holds {}, which is not inside its package's folder {top}, so it was not unpacked", bad.full_name));
    }
    let manifest = entries.iter().find(|e| e.full_name == format!("{top}/{}", package::FILE_NAME)).ok_or("no manifest")?;
    let mut bytes = Vec::new();
    archive::copy_entry(zip, manifest, &mut bytes).map_err(|e| format!("{zip}: {e}"))?;
    let doc = package::parse(&emusen_dotnet::file::decode(&bytes)).map_err(|r| format!("{zip}: {}", r.words()))?;
    Ok(Some((top, doc)))
}

/// Where a package's zip is unpacked: beside it where it lies under `packages`, else straight under `packages`; never
/// beside a zip elsewhere, which may be in the library.
pub fn target(packages: &str, zip: &str, top: &str) -> String {
    let full = path::full_path(zip);
    let parent = path::directory_name(STYLE, &full).unwrap_or_default();
    let packages = path::full_path(packages);
    if crate::claims::within(&parent, &packages) { path::combine(STYLE, &parent, top) } else { path::combine(STYLE, &packages, top) }
}

fn put_time(at: &str, e: &ZipEntry) {
    if let (Some(t), Ok(f)) = (e.modified, std::fs::File::options().write(true).open(at)) {
        let _ = f.set_modified(t);
    }
}

/// The zip unpacked into `target`, which must not be there: into a hidden folder beside it first, each entry checked
/// against its CRC-32 and given its time, then renamed into place whole.
pub fn unpack(zip: &str, top: &str, target: &str, stop: &dyn Fn() -> bool) -> Result<(), String> {
    if std::fs::symlink_metadata(target).is_ok() {
        return Err(format!("{target} is already there, so {zip} was not unpacked over it"));
    }
    let entries = archive::entries_of(zip).map_err(|e| format!("{zip}: {e}"))?;
    let parent = path::directory_name(STYLE, target).unwrap_or_default();
    std::fs::create_dir_all(&parent).map_err(|e| format!("{parent}: {e}"))?;
    let partial = path::combine(STYLE, &parent, &format!(".{}.partial-{}", path::file_name(STYLE, target), std::process::id()));
    if dotnet_files::directory_exists(&partial) {
        std::fs::remove_dir_all(&partial).map_err(|e| format!("{partial}: {e}"))?;
    }
    let done = (|| {
        std::fs::create_dir_all(&partial).map_err(|e| format!("{partial}: {e}"))?;
        for e in &entries {
            if stop() {
                return Err("stopped".to_string());
            }
            if !safe(&e.full_name, top) {
                return Err(format!("{zip} holds {}, which is not inside its package's folder {top}", e.full_name));
            }
            let rest = &e.full_name[top.len() + 1..];
            if rest.is_empty() || rest.ends_with('/') {
                std::fs::create_dir_all(crate::claims::under(&partial, rest)).map_err(|x| x.to_string())?;
                continue;
            }
            let at = crate::claims::under(&partial, rest);
            if let Some(dir) = path::directory_name(STYLE, &at) {
                std::fs::create_dir_all(dir).map_err(|x| x.to_string())?;
            }
            let mut out = std::fs::File::create(&at).map_err(|x| format!("{at}: {x}"))?;
            archive::copy_entry(zip, e, &mut out).map_err(|w| format!("{zip}: {w}"))?;
            out.sync_all().map_err(|x| format!("{at}: {x}"))?;
            drop(out);
            put_time(&at, e);
        }
        Ok(())
    })();
    if let Err(words) = done {
        let _ = std::fs::remove_dir_all(&partial);
        return Err(words);
    }
    if std::fs::symlink_metadata(target).is_ok() {
        let _ = std::fs::remove_dir_all(&partial);
        return Err(format!("{target} appeared while {zip} was unpacked, and was left as it is"));
    }
    std::fs::rename(&partial, target).map_err(|e| {
        let _ = std::fs::remove_dir_all(&partial);
        format!("{target}: {e}")
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
                    crate::json::write_compact(&mut s, v);
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
    let manifest = dotnet_files::join(&target, package::FILE_NAME);
    if std::fs::symlink_metadata(&target).is_ok() {
        return match package::read(&manifest) {
            Ok(Ok(there)) if same_game(&there, &doc) => Ok(Some(target)),
            Ok(Ok(_)) => Err(format!("{target} holds another game's package, so {zip} was not unpacked over it")),
            _ => Err(format!("{target} is there and holds no package this build reads, so {zip} was not unpacked over it")),
        };
    }
    unpack(zip, &top, &target, &|| false)?;
    Ok(Some(target))
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::test_support::TempDir;

    fn put(at: &str, bytes: &[u8]) {
        std::fs::create_dir_all(path::directory_name(STYLE, at).unwrap()).unwrap();
        std::fs::write(at, bytes).unwrap();
    }

    #[test]
    fn a_package_zipped_and_unpacked_keeps_every_file_and_its_time_to_the_tick() {
        let t = TempDir::new("package-zip");
        let folder = t.join("Packages/SNES/Game.sfc");
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
        let method = |n: &str| entries.iter().find(|e| e.full_name == n).unwrap().method;
        assert_eq!((method("Game.sfc/Game.sfc"), method("Game.sfc/disc.chd"), method("Game.sfc/media/covers/Game.png")), (8, 0, 0));
        assert_eq!(entries[0].full_name, "Game.sfc/package.json");
        // Opened where its folder is: the folder itself, the zip only read.
        assert_eq!(opened(&t.join("Packages"), &zip).unwrap().as_deref(), Some(folder.as_str()));
        // Carried to another machine: unpacked once beside it, every byte and time as it was.
        let elsewhere = t.join("Other/Packages/SNES/Game.sfc.zip");
        put(&elsewhere, &std::fs::read(&zip).unwrap());
        let unpacked = opened(&t.join("Other/Packages"), &elsewhere).unwrap().unwrap();
        assert_eq!(unpacked, t.join("Other/Packages/SNES/Game.sfc"));
        assert_eq!(std::fs::read(format!("{unpacked}/Game.sfc")).unwrap(), rom);
        assert_eq!(std::fs::metadata(format!("{unpacked}/states/Game.slot2.state")).unwrap().modified().unwrap(), odd);
        assert!(!std::path::Path::new(&format!("{unpacked}/.hidden")).exists());
        // A zip outside Packages goes straight under it; a folder there of another game refuses it.
        let outside = t.join("Downloads/Game.sfc.zip");
        put(&outside, &std::fs::read(&zip).unwrap());
        assert_eq!(opened(&t.join("Third"), &outside).unwrap().unwrap(), t.join("Third/Game.sfc"));
        std::fs::write(t.join("Third/Game.sfc/package.json"), manifest.replace("00000000\",\"md5", "11111111\",\"md5").replace("\"sha1\":\"0000000000000000000000000000000000000000\"", "\"sha1\":\"1111111111111111111111111111111111111111\"")).unwrap();
        assert!(opened(&t.join("Third"), &outside).unwrap_err().contains("another game's package"));
        // A damaged entry is refused and nothing is left.
        let mut bytes = std::fs::read(&zip).unwrap();
        let at = bytes.windows(13).position(|w| w == b"SAVEDATA-1234").unwrap();
        bytes[at] = b'X';
        let damaged = t.join("Fourth/Packages/Game.sfc.zip");
        put(&damaged, &bytes);
        assert!(opened(&t.join("Fourth/Packages"), &damaged).unwrap_err().contains("is damaged"));
        assert!(!std::path::Path::new(&t.join("Fourth/Packages/Game.sfc")).exists());
        // A game's own zip is no package.
        let plain = t.join("plain.zip");
        let mut w = ZipWriter::create(&plain).unwrap();
        w.add("inside.sfc", &format!("{folder}/Game.sfc"), odd, false, &|| false).unwrap();
        w.finish().unwrap();
        assert_eq!(opened(&t.join("Packages"), &plain).unwrap(), None);
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
