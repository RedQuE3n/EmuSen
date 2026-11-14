//! The C layer over emusen-cheats, for the C# `ChtFile`, `CheatDatabase` and `CheatDatabasePruner` (EmuSen_Backend.md
//! §15.3, §40.1); a database's install is the session table's cheat op `DB_UPDATE` since the frontend API's 2.0, and
//! since interface 48 also `emusen_cheats_install`, of bytes the host downloaded (EmuSen_RustPlatform.md §56). A
//! database is a handle, scanned once and never changed, so any thread may ask it.

use crate::{fail, given, put, required, status};
use emusen_cheats::cht::{self, Cheat};
use emusen_cheats::database::Database;
use emusen_cheats::install::{self, Refusal};
use emusen_cheats::prune::{self, Pruned};
use emusen_cheats::registry;
use emusen_galaxia::models::ConfigModel;
use serde_json::Value;
use emusen_systems::cheats::{Kind, Write, WriteType};

fn object(members: Vec<(&str, Value)>) -> Value {
    Value::Object(members.into_iter().map(|(k, v)| (String::from(k), v)).collect())
}

fn compact(value: &Value) -> String {
    let mut out = String::new();
    out.push_str(&serde_json::to_string(value).expect("a value is JSON"));
    out
}

fn request(data: *const u8, len: usize) -> Result<Value, i32> {
    serde_json::from_str::<Value>(unsafe { given(data, len) }?).map_err(|e| fail(status::PARSE, e.to_string()))
}

fn text<'a>(v: &'a Value, name: &str) -> Result<Option<&'a str>, i32> {
    match v.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(t)) => Ok(Some(t)),
        _ => Err(fail(status::PARSE, format!("{name} is not text"))),
    }
}

fn number(v: &Value, name: &str) -> Result<i64, i32> {
    match v.get(name) {
        Some(Value::Number(n)) => n.as_i64().ok_or_else(|| fail(status::PARSE, format!("{name} is not a whole number"))),
        _ => Err(fail(status::PARSE, format!("{name} is not a number"))),
    }
}

fn words(v: &Value, name: &str) -> Result<Vec<String>, i32> {
    match v.get(name) {
        Some(Value::Array(items)) => items.iter().map(|i| i.as_str().map(str::to_string).ok_or_else(|| fail(status::PARSE, format!("{name} holds a value that is not text")))).collect(),
        _ => Err(fail(status::PARSE, format!("{name} is not a list"))),
    }
}

fn answer(result: Result<String, i32>, out: *mut u8, len: usize) -> i64 {
    match result {
        Ok(text) => unsafe { put(text.as_bytes(), out, len) },
        Err(s) => s as i64,
    }
}

const TYPES: [WriteType; 7] = [WriteType::Set, WriteType::Increase, WriteType::Decrease, WriteType::IfEqual, WriteType::IfNotEqual, WriteType::IfLess, WriteType::IfGreater];

fn write_from(v: &Value) -> Result<Write, i32> {
    let kind = TYPES.get(number(v, "type")? as usize).copied().ok_or_else(|| fail(status::PARSE, "a write's type is not one there is"))?;
    let bit = match v.get("bit") {
        None | Some(Value::Null) => None,
        _ => Some(number(v, "bit")? as i32),
    };
    Ok(Write {
        space: text(v, "space")?.unwrap_or("").to_string(),
        address: number(v, "address")? as i32,
        value: number(v, "value")? as u32,
        width: number(v, "width")? as i32,
        kind,
        bit_position: bit,
        big_endian: v.get("big_endian").and_then(Value::as_bool).unwrap_or(false),
        repeat_count: number(v, "repeat")? as i32,
        repeat_add_address: number(v, "repeat_address")? as i32,
        repeat_add_value: number(v, "repeat_value")? as u32,
    })
}

fn cheat_json(c: &Cheat) -> Value {
    object(vec![
        ("desc", Value::String(String::from(c.description.as_str()))),
        ("enabled", Value::Bool(c.enabled)),
        ("kind", Value::String(String::from(if c.kind == Kind::RomPatch { "RomPatch" } else { "RamPoke" }))),
        ("writes", Value::Array(c.writes.iter().map(emusen_systems::cheats::write_json).collect())),
    ])
}

/// A `.cht` text read: `{"text", "codec", "space", "patch"}` in, each codec an emusen-systems id or null; since 48 `path`
/// may name a file to read in place of `text`, and `system` its pair of codecs, as a session of it reads one; out `{"cheats": [{desc,
/// enabled, kind, writes}], "skipped", "file"}`, `file` (since 48) the cheats as a game's own list, the CheatFile
/// `cheats/<game>.json` holds. BAD_ARGUMENT where a codec's range fault escaped the C#'s read.
///
/// # Safety
/// `request` must be valid for `request_len` bytes; `out` for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_cheats_parse(request_data: *const u8, request_len: usize, out: *mut u8, len: usize) -> i64 {
    let run = || -> Result<String, i32> {
        let r = request(request_data, request_len)?;
        let words = match text(&r, "path")? {
            Some(path) => emusen_galaxia::text_file::decode(&std::fs::read(path).map_err(|e| crate::fail_io(&e))?),
            None => text(&r, "text")?.unwrap_or("").to_string(),
        };
        let parsed = match text(&r, "system")? {
            Some(system) => {
                emusen_systems::find(system).ok_or_else(|| fail(status::BAD_ARGUMENT, &format!("the systems table holds no system {system}")))?;
                let (auto, explicit) = emusen_systems::cheats::pair(system);
                cht::parse_with(&words, auto, explicit)
            }
            None => cht::parse(&words, text(&r, "codec")?, text(&r, "space")?.unwrap_or(""), text(&r, "patch")?),
        }
        .map_err(|_| fail(status::BAD_ARGUMENT, "a codec's index fell outside its code"))?;
        let (list, left_out) = registry::list_of(&parsed.cheats);
        let file = list.file_text().map_err(|w| fail(status::PARSE, w))?;
        let file: Value = serde_json::from_str(&file).map_err(|e| fail(status::PARSE, e.to_string()))?;
        Ok(compact(&object(vec![
            ("cheats", Value::Array(parsed.cheats.iter().map(cheat_json).collect())),
            ("skipped", Value::from(parsed.skipped)),
            ("file", file),
            ("left_out", Value::from(left_out)),
        ])))
    };
    answer(run(), out, len)
}

/// A downloaded cheat database installed under `directory` (since 48): `install::install_staged`, so one refused leaves
/// the database as it was. Out `{"installed", "skipped"}`; PARSE for bytes that are no zip the reader takes, an I/O
/// status where a folder could not be made or a file moved.
///
/// # Safety
/// `zip` must be valid for `zip_len` bytes; `directory` for `directory_len` bytes; `out` for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_cheats_install(zip: *const u8, zip_len: usize, directory: *const u8, directory_len: usize, out: *mut u8, len: usize) -> i64 {
    thread_local! {
        static WAITING: crate::reply::Waiting = const { std::cell::RefCell::new(None) };
    }
    let given = unsafe { (crate::bytes(zip, zip_len), required(directory, directory_len)) };
    let (zip, directory) = match given {
        (Ok(z), Ok(d)) => (z, d),
        (Err(s), _) | (_, Err(s)) => return s as i64,
    };
    // Done once: the length query installs and the fill takes its answer, keyed by the folder and the archive's digest.
    let key = format!("{directory}\0{}\0{:x}", zip.len(), {
        use std::hash::{Hash, Hasher};
        let mut h = std::hash::DefaultHasher::new();
        zip.hash(&mut h);
        h.finish()
    });
    let run = || -> Result<String, i32> {
        let done = install::install_staged(zip, directory).map_err(|r| match r {
            Refusal::Unreadable(words) => fail(status::PARSE, words),
            Refusal::Io(e) => crate::fail_io(&e),
        })?;
        Ok(compact(&object(vec![("installed", Value::from(done.installed)), ("skipped", Value::from(done.skipped))])))
    };
    unsafe { crate::reply::answer(&WAITING, 0, &key, out, len, run, |s| s) }
}

/// Cheats written as a `.cht` text: `{"cheats": [{desc, enabled, kind, writes}]}` in.
///
/// # Safety
/// As `emusen_cheats_parse`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_cheats_write(request_data: *const u8, request_len: usize, out: *mut u8, len: usize) -> i64 {
    let run = || -> Result<String, i32> {
        let r = request(request_data, request_len)?;
        let Some(Value::Array(items)) = r.get("cheats") else { return Err(fail(status::PARSE, "the request has no cheats")) };
        let cheats = items
            .iter()
            .map(|c| {
                let writes = match c.get("writes") {
                    Some(Value::Array(ws)) => ws.iter().map(write_from).collect::<Result<Vec<_>, _>>()?,
                    _ => return Err(fail(status::PARSE, "a cheat has no writes")),
                };
                let kind = if text(c, "kind")? == Some("RomPatch") { Kind::RomPatch } else { Kind::RamPoke };
                Ok(Cheat { description: text(c, "desc")?.unwrap_or("").to_string(), enabled: c.get("enabled").and_then(Value::as_bool).unwrap_or(false), writes, kind })
            })
            .collect::<Result<Vec<_>, i32>>()?;
        Ok(cht::write(&cheats))
    };
    answer(run(), out, len)
}

/// A database scanned from its folder: a handle, freed with `emusen_cheats_database_free`; null for a null or bad path.
///
/// # Safety
/// `directory` must be valid for `directory_len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_cheats_database_open(directory: *const u8, directory_len: usize) -> *mut Database {
    match unsafe { required(directory, directory_len) } {
        Ok(d) => Box::into_raw(Box::new(Database::scan(d))),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Lets go of a database.
///
/// # Safety
/// `db` must be a handle `emusen_cheats_database_open` gave and not yet freed, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_cheats_database_free(db: *mut Database) {
    if !db.is_null() {
        drop(unsafe { Box::from_raw(db) });
    }
}

fn database<'a>(db: *const Database) -> Result<&'a Database, i32> {
    if db.is_null() { Err(fail(status::NULL, "no database")) } else { Ok(unsafe { &*db }) }
}

/// Every file of the database, in its order, as `[[system, game, path], ...]`.
///
/// # Safety
/// `db` must be a live handle; `out` valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_cheats_database_entries(db: *const Database, out: *mut u8, len: usize) -> i64 {
    let run = || -> Result<String, i32> {
        let d = database(db)?;
        Ok(compact(&Value::Array(d.entries.iter().map(|e| Value::Array(vec![Value::String(String::from(e.system.as_str())), Value::String(String::from(e.game.as_str())), Value::String(String::from(e.path.as_str()))])).collect())))
    };
    answer(run(), out, len)
}

/// A question of the database: `{"op": "systems"}` gives `[[system, count], ...]`; `{"op": "games", "system", "filter"}`
/// and `{"op": "find", "text", "limit"}` give the files' places in `emusen_cheats_database_entries`' list, in order.
///
/// # Safety
/// `db` must be a live handle; `request` valid for `request_len` bytes; `out` for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_cheats_database_query(db: *const Database, request_data: *const u8, request_len: usize, out: *mut u8, len: usize) -> i64 {
    let run = || -> Result<String, i32> {
        let d = database(db)?;
        let r = request(request_data, request_len)?;
        let value = match text(&r, "op")? {
            Some("systems") => Value::Array(d.systems().into_iter().map(|(s, n)| Value::Array(vec![Value::String(String::from(s)), Value::from(n)])).collect()),
            Some("games") => Value::Array(d.games(text(&r, "system")?.unwrap_or(""), text(&r, "filter")?).into_iter().map(Value::from).collect()),
            Some("find") => Value::Array(d.find(text(&r, "text")?.unwrap_or(""), number(&r, "limit")?.clamp(i32::MIN as i64, i32::MAX as i64) as i32).into_iter().map(Value::from).collect()),
            _ => return Err(fail(status::BAD_ARGUMENT, "no such question")),
        };
        Ok(compact(&value))
    };
    answer(run(), out, len)
}

fn pruned_json(p: &Pruned) -> Value {
    object(vec![("system", Value::String(String::from(p.system.as_str()))), ("path", Value::String(String::from(p.path.as_str()))), ("files", Value::from(p.files)), ("bytes", Value::from(p.bytes))])
}

/// A prune's plan: `{"keep": [...]}` in; out `{"removing": [{system, path, files, bytes}], "keeping", "reason"}`.
///
/// # Safety
/// As `emusen_cheats_database_query`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_cheats_prune_plan(db: *const Database, request_data: *const u8, request_len: usize, out: *mut u8, len: usize) -> i64 {
    let run = || -> Result<String, i32> {
        let d = database(db)?;
        let r = request(request_data, request_len)?;
        let plan = prune::plan(d, &words(&r, "keep")?);
        Ok(compact(&object(vec![
            ("removing", Value::Array(plan.removing.iter().map(pruned_json).collect())),
            ("keeping", Value::Array(plan.keeping.iter().map(|k| Value::String(String::from(k.as_str()))).collect())),
            ("reason", plan.reason.map_or(Value::Null, |s| Value::String(String::from(s)))),
        ])))
    };
    answer(run(), out, len)
}

/// A plan applied: `{"directory", "removing": [{system, path}]}` in; out `{"removed", "failed": [...]}`.
///
/// # Safety
/// As `emusen_cheats_parse`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_cheats_prune_apply(request_data: *const u8, request_len: usize, out: *mut u8, len: usize) -> i64 {
    thread_local! {
        static WAITING: crate::reply::Waiting = const { std::cell::RefCell::new(None) };
    }
    let key = match unsafe { crate::reply::text(request_data, request_len, |s, w| fail(s, w)) } {
        Ok(k) => k,
        Err(s) => return s as i64,
    };
    // Done once: the length query applies the plan and the fill takes its answer - see EmuSen_RustPlatform.md §47.25.
    let run = || -> Result<String, i32> {
        let r = request(request_data, request_len)?;
        let Some(Value::Array(items)) = r.get("removing") else { return Err(fail(status::PARSE, "the request has no removing")) };
        let removing = items
            .iter()
            .map(|i| Ok(Pruned { system: text(i, "system")?.unwrap_or("").to_string(), path: text(i, "path")?.unwrap_or("").to_string(), files: 0, bytes: 0 }))
            .collect::<Result<Vec<_>, i32>>()?;
        let db = Database { directory: text(&r, "directory")?.unwrap_or("").to_string(), entries: Vec::new() };
        let (removed, failed) = prune::apply(&db, &removing);
        Ok(compact(&object(vec![("removed", Value::from(removed)), ("failed", Value::Array(failed.into_iter().map(|s| Value::String(String::from(s))).collect()))])))
    };
    unsafe { crate::reply::answer(&WAITING, 0, key, out, len, run, |s| s) }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The length query applies the plan and the fill takes that answer, so each folder is removed once and said removed (EmuSen_RustPlatform.md §47.25).
    #[test]
    fn a_prune_applied_by_length_then_fill_removes_once_and_says_so() {
        let dir = std::env::temp_dir().join(format!("emusen-platform-prune-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        for system in ["Keep", "Drop"] {
            std::fs::create_dir_all(dir.join(system)).unwrap();
            std::fs::write(dir.join(system).join("a.cht"), b"cheats = 0").unwrap();
        }
        let drop = dir.join("Drop").to_string_lossy().into_owned();
        let request = serde_json::json!({ "directory": dir.to_string_lossy(), "removing": [{ "system": "Drop", "path": drop }] }).to_string();
        let call = |out: *mut u8, len: usize| unsafe { emusen_cheats_prune_apply(request.as_ptr(), request.len(), out, len) };
        let length = call(std::ptr::null_mut(), 0);
        assert!(length > 0);
        let mut out = vec![0u8; length as usize];
        assert_eq!(call(out.as_mut_ptr(), out.len()), length);
        assert_eq!(String::from_utf8(out).unwrap(), r#"{"removed":1,"failed":[]}"#);
        assert!(!dir.join("Drop").exists() && dir.join("Keep/a.cht").exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    fn answered(call: impl Fn(*mut u8, usize) -> i64) -> Value {
        let length = call(std::ptr::null_mut(), 0);
        assert!(length > 0, "status {length}");
        let mut out = vec![0u8; length as usize];
        assert_eq!(call(out.as_mut_ptr(), out.len()), length);
        serde_json::from_slice(&out).unwrap()
    }

    // Interface 48: an archive the test makes, installed by length then fill, is installed once and moved whole into the folder.
    #[test]
    fn an_archive_installed_by_length_then_fill_is_moved_into_the_folder_once() {
        use emusen_galaxia::archive::ZipWriter;
        let dir = std::env::temp_dir().join(format!("emusen-platform-install-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("source"), "cheats = 0\n").unwrap();
        let zip = dir.join("made.zip");
        let mut w = ZipWriter::create(zip.to_str().unwrap()).unwrap();
        for name in ["Nintendo - SNES/Game.cht", "Sega/Other.cht", "readme.txt"] {
            w.add(name, dir.join("source").to_str().unwrap(), std::time::SystemTime::now(), false, &|| false).unwrap();
        }
        w.finish().unwrap();
        let zip = std::fs::read(&zip).unwrap();
        let target = dir.join("db").to_string_lossy().into_owned();
        let call = |out: *mut u8, len: usize| unsafe { emusen_cheats_install(zip.as_ptr(), zip.len(), target.as_ptr(), target.len(), out, len) };
        assert_eq!(answered(call), serde_json::json!({ "installed": 2, "skipped": 1 }));
        assert!(dir.join("db/Nintendo - SNES/Game.cht").exists() && dir.join("db/Sega/Other.cht").exists());
        assert!(!dir.join("db.part").exists());
        let half = &zip[..zip.len() / 2];
        let refused = unsafe { emusen_cheats_install(half.as_ptr(), half.len(), target.as_ptr(), target.len(), std::ptr::null_mut(), 0) };
        assert_eq!(refused, status::PARSE as i64, "a damaged archive is refused");
        assert!(dir.join("db/Sega/Other.cht").exists(), "and the database stays as it was");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    // Interface 48: a system the table does not hold is refused in words; one it holds with no codecs still reads.
    #[test]
    fn a_cht_read_for_an_unknown_system_is_refused_and_one_with_no_codecs_reads() {
        use serde_json::json;
        let cht = "cheats = 2\ncheat0_desc = \"Code\"\ncheat0_code = \"7E0DBE09\"\ncheat1_desc = \"Poked\"\ncheat1_address = 4660\ncheat1_value = 7\n";
        let unknown = json!({ "text": cht, "system": "no-such-console" }).to_string();
        let status = unsafe { emusen_cheats_parse(unknown.as_ptr(), unknown.len(), std::ptr::null_mut(), 0) };
        assert_eq!(status, status::BAD_ARGUMENT as i64);
        let mut words = vec![0u8; unsafe { crate::emusen_platform_last_error(std::ptr::null_mut(), 0) }.max(0) as usize];
        unsafe { crate::emusen_platform_last_error(words.as_mut_ptr(), words.len()) };
        assert!(String::from_utf8_lossy(&words).contains("no-such-console"), "the refusal names the system: {}", String::from_utf8_lossy(&words));
        assert!(emusen_systems::find("dreamcast").is_some() && emusen_systems::cheats::pair("dreamcast") == (None, None), "the table holds the Dreamcast, with no codecs");
        let held = json!({ "text": cht, "system": "dreamcast" }).to_string();
        let answer = answered(|out, len| unsafe { emusen_cheats_parse(held.as_ptr(), held.len(), out, len) });
        let named: Vec<_> = answer["file"]["Cheats"].as_array().unwrap().iter().map(|c| c["Description"].clone()).collect();
        assert_eq!(named, [json!("Poked")], "with no codec a code is no cheat, an address still one: {answer}");
    }

    // Interface 48: a .cht read with the console's own codecs by its system, and its cheats as the game's own list.
    #[test]
    fn a_cht_read_for_a_system_answers_the_games_own_list() {
        use serde_json::json;
        let cht = "cheats = 3\ncheat0_desc = \"Infinite lives\"\ncheat0_code = \"7E0DBE09\"\ncheat0_enable = true\ncheat1_desc = \"Broken\"\ncheat1_code = \"ZZ\"\n\
                   cheat2_desc = \"Poked\"\ncheat2_address = 4660\ncheat2_value = 7\n";
        let request = serde_json::json!({ "text": cht, "system": "snes" }).to_string();
        let answer = answered(|out, len| unsafe { emusen_cheats_parse(request.as_ptr(), request.len(), out, len) });
        assert_eq!((&answer["skipped"], &answer["left_out"]), (&json!(1), &json!(0)));
        assert_eq!(answer["cheats"][0]["writes"][0]["space"], "WRAM", "the Pro Action Replay code's own space for bank 7E: {answer}");
        assert_eq!(answer["cheats"][1]["writes"][0]["space"], "CpuBus", "an address with no code is in the SNES's cheat space: {answer}");
        let file = &answer["file"]["Cheats"];
        assert_eq!(file.as_array().map(Vec::len), Some(2), "{answer}");
        assert_eq!(file[0]["Description"], "Infinite lives");
        assert_eq!(file[0]["Enabled"], true);
        assert_eq!(file[0]["Kind"], "RamPoke");
        assert_eq!((&file[0]["Writes"][0]["Space"], &file[0]["Writes"][0]["Address"], &file[0]["Writes"][0]["Value"]), (&json!("WRAM"), &json!("DBE"), &json!("09")));
        assert_eq!((&file[1]["Description"], &file[1]["Enabled"]), (&json!("Poked"), &json!(false)));
        assert_eq!((&file[1]["Writes"][0]["Space"], &file[1]["Writes"][0]["Address"], &file[1]["Writes"][0]["Value"]), (&json!("CpuBus"), &json!("1234"), &json!("07")));
        let file = std::env::temp_dir().join(format!("emusen-platform-cht-{}.cht", std::process::id()));
        std::fs::write(&file, [b"\xEF\xBB\xBF".as_slice(), cht.as_bytes()].concat()).unwrap();
        let by_path = serde_json::json!({ "path": file.to_string_lossy(), "system": "snes" }).to_string();
        let read = answered(|out, len| unsafe { emusen_cheats_parse(by_path.as_ptr(), by_path.len(), out, len) });
        std::fs::remove_file(&file).unwrap();
        assert_eq!(read["file"], answer["file"], "a file named by its path, its mark dropped, reads as its text");
        let plain = serde_json::json!({ "text": cht }).to_string();
        let none = answered(|out, len| unsafe { emusen_cheats_parse(plain.as_ptr(), plain.len(), out, len) });
        let named: Vec<_> = none["file"]["Cheats"].as_array().unwrap().iter().map(|c| c["Description"].clone()).collect();
        assert_eq!(named, [json!("Poked")], "with no codec a code is no cheat, an address still one: {none}");
    }
}
