//! The disk corpus: Galaxia's files as the platform wrote them on 2026-11-07, hand-edited variants, and third-party
//! zips, each beside the model it reads to. Every change to how a file is read is held to it. See
//! EmuSen_RustPlatform.md §47.

use emusen_galaxia::{archive, config, disc_set, model, models, package, package_zip};
use serde_json::Value;
use std::path::{Path, PathBuf};

const DISK: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/disk");

fn files(folder: &str, extension: &str) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = std::fs::read_dir(Path::new(DISK).join(folder)).unwrap().map(|e| e.unwrap().path()).filter(|p| p.extension().is_some_and(|e| e == extension)).collect();
    found.sort();
    assert!(!found.is_empty(), "{folder} holds no .{extension}");
    found
}

fn expected(file: &Path, extension: &str) -> String {
    std::fs::read_to_string(file.with_extension(extension)).unwrap_or_else(|_| panic!("{} has no .{extension}", file.display())).trim_end().to_string()
}

/// Two models as JSON: the same members in the same order, numbers equal as a double or as a float.
fn same(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => {
            let (x, y) = (x.as_f64().unwrap(), y.as_f64().unwrap());
            x == y || x as f32 == y as f32
        }
        (Value::Array(x), Value::Array(y)) => x.len() == y.len() && x.iter().zip(y).all(|(x, y)| same(x, y)),
        (Value::Object(x), Value::Object(y)) => x.len() == y.len() && x.iter().zip(y).all(|((kx, vx), (ky, vy))| kx == ky && same(vx, vy)),
        _ => a == b,
    }
}

fn assert_same_model(file: &Path, got: &str, want: &str) {
    let parsed = |t: &str| serde_json::from_str::<Value>(t).unwrap_or_else(|e| panic!("{}: {t} is not JSON: {e}", file.display()));
    assert!(same(&parsed(got), &parsed(want)), "{} reads to\n{got}\nand the corpus holds\n{want}", file.display());
}

/// A config file's outcome: its model as JSON, `null`, or `refused`.
fn config_outcome(model: &models::Model, bytes: &[u8]) -> String {
    match config::bind_text(model, &emusen_galaxia::text_file::decode(bytes), true) {
        Ok(Some(bound)) => model::transport(&bound),
        Ok(None) => "null".to_string(),
        Err(_) => "refused".to_string(),
    }
}

fn config_written(model: &models::Model, bytes: &[u8]) -> String {
    let bound = config::bind_text(model, &emusen_galaxia::text_file::decode(bytes), true).unwrap().unwrap();
    model::format(model.schema, &bound).unwrap()
}

const CONFIGS: [(&str, &models::Model); 5] =
    [("appsettings", &models::MODELS[0]), ("audio", &models::MODELS[1]), ("graphics", &models::MODELS[2]), ("cheats", &models::MODELS[3]), ("consoles", &models::CONSOLES_MODEL)];

#[test]
fn every_config_file_reads_to_its_recorded_model() {
    for (folder, model) in CONFIGS {
        for file in files(&format!("config/{folder}"), "json") {
            let got = config_outcome(model, &std::fs::read(&file).unwrap());
            let want = expected(&file, "model");
            if want == "null" || want == "refused" || got == "null" || got == "refused" {
                assert_eq!(got, want, "{}", file.display());
            } else {
                assert_same_model(&file, &got, &want);
            }
        }
    }
}

#[test]
fn a_config_read_written_and_read_again_is_the_same_model() {
    for (folder, model) in CONFIGS {
        for file in files(&format!("config/{folder}"), "json") {
            if matches!(expected(&file, "model").as_str(), "null" | "refused") {
                continue;
            }
            let first = config_outcome(model, &std::fs::read(&file).unwrap());
            let again = config_outcome(model, config_written(model, &std::fs::read(&file).unwrap()).as_bytes());
            assert_same_model(&file, &again, &first);
        }
    }
}

/// A manifest's outcome: its document as compact JSON, or the kind of refusal.
fn package_outcome(bytes: &[u8]) -> String {
    match package::parse(&emusen_galaxia::text_file::decode(bytes)) {
        Ok(doc) => {
            let mut out = String::new();
            emusen_dotnet::json::write_compact(&mut out, &doc);
            out
        }
        Err(package::Refusal::NotJson(_)) => "refused: not JSON".to_string(),
        Err(package::Refusal::Invalid(_)) => "refused: invalid".to_string(),
        Err(package::Refusal::Newer(_)) => "refused: newer".to_string(),
    }
}

#[test]
fn every_manifest_reads_to_its_recorded_document_and_is_written_back_the_same() {
    for file in files("package", "json") {
        let got = package_outcome(&std::fs::read(&file).unwrap());
        let want = expected(&file, "model");
        if want.starts_with("refused") {
            assert_eq!(got, want, "{}", file.display());
            continue;
        }
        assert_same_model(&file, &got, &want);
        let doc = package::parse(&emusen_galaxia::text_file::decode(&std::fs::read(&file).unwrap())).unwrap();
        assert_same_model(&file, &package_outcome(package::format(&doc).as_bytes()), &want);
    }
}

#[test]
fn an_unpacked_folder_keeps_its_name() {
    let table = std::fs::read_to_string(Path::new(DISK).join("unpacked.tsv")).unwrap();
    for line in table.lines().filter(|l| !l.starts_with('#')) {
        let [full, length, ticks, name] = line.split('\t').collect::<Vec<_>>()[..] else { panic!("{line}") };
        assert_eq!(disc_set::folder_name(full, length.parse().unwrap(), ticks.parse().unwrap()), name, "{line}");
    }
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = emusen_galaxia::rom_hash::Crc32::default();
    crc.append(bytes);
    crc.value()
}

/// A zip as one line per entry, `name<TAB>length<TAB>CRC-32`; with `times`, each entry's own time in nanoseconds since 1970.
fn listing<R: std::io::Read + std::io::Seek>(mut zip: archive::Zip<R>, times: bool) -> Result<String, String> {
    let mut out = String::new();
    for e in zip.entries()? {
        let mut bytes = Vec::new();
        zip.copy(&e, &mut bytes)?;
        out.push_str(&format!("{}\t{}\t{:08x}", e.full_name, bytes.len(), crc32(&bytes)));
        if times {
            let nanos = zip.modified(&e).map_or(-1, |t| t.duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as i128);
            out.push_str(&format!("\t{nanos}"));
        }
        out.push('\n');
    }
    Ok(out.trim_end().to_string())
}

/// Zips refused since the zip crate reads them (§47.15), each recorded as it was read before: a name written twice.
const REFUSED: &[&str] = &["duplicates.zip"];

/// Every zip lists its entries the same read from its file, as a game's is, and from memory, as the cheat database's is.
#[test]
fn every_zip_lists_its_recorded_entries_from_a_file_and_from_memory() {
    for zip in files("zip", "zip") {
        let want = expected(&zip, "entries");
        let name = zip.file_name().unwrap().to_str().unwrap();
        if REFUSED.contains(&name) {
            assert!(archive::Zip::open(zip.to_str().unwrap()).is_err(), "{name}");
            continue;
        }
        let plain: String = want.lines().map(|l| l.splitn(4, '\t').take(3).collect::<Vec<_>>().join("\t")).collect::<Vec<_>>().join("\n");
        let bytes = std::fs::read(&zip).unwrap();
        assert_eq!(listing(archive::Zip::over(std::io::Cursor::new(bytes.as_slice())).unwrap(), false).as_deref(), Ok(plain.as_str()), "{name}, from memory");
        let times = name == "package.zip";
        assert_eq!(listing(archive::Zip::open(zip.to_str().unwrap()).unwrap(), times).as_deref(), Ok(if times { want.as_str() } else { plain.as_str() }), "{name}, from its file");
    }
}

#[test]
fn a_package_zip_unpacks_to_its_files_with_their_times_to_the_tick() {
    let zip = Path::new(DISK).join("zip/package.zip");
    let (top, doc) = package_zip::manifest_in(zip.to_str().unwrap()).unwrap().unwrap();
    assert_eq!(top, "Game (USA)");
    assert_same_model(&zip, &package_outcome(package::format(&doc).as_bytes()), &expected(&Path::new(DISK).join("package/written.json"), "model"));
    let target = std::env::temp_dir().join(format!("galaxia-disk-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&target);
    package_zip::unpack(zip.to_str().unwrap(), &top, target.to_str().unwrap(), &|| false).unwrap();
    for line in expected(&zip, "entries").lines() {
        let [name, length, crc, nanos] = line.split('\t').collect::<Vec<_>>()[..] else { panic!("{line}") };
        let at = target.join(name.strip_prefix("Game (USA)/").unwrap());
        let bytes = std::fs::read(&at).unwrap();
        assert_eq!((bytes.len().to_string(), format!("{:08x}", crc32(&bytes))), (length.to_string(), crc.to_string()), "{name}");
        let modified = std::fs::metadata(&at).unwrap().modified().unwrap().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        assert_eq!(modified.to_string(), nanos, "{name}'s time");
    }
    std::fs::remove_dir_all(&target).unwrap();
}

#[test]
fn a_stored_path_key_is_still_the_key_of_its_path() {
    let table = std::fs::read_to_string(Path::new(DISK).join("keys.tsv")).unwrap();
    for line in table.lines().filter(|l| !l.starts_with('#')) {
        let [given, key] = line.split('\t').collect::<Vec<_>>()[..] else { panic!("{line}") };
        assert_eq!(emusen_galaxia::dotnet_path::full_path(given), key, "{line}");
    }
}
