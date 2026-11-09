//! The C layer over emusen-cheats, for the C# `ChtFile`, `CheatDatabase` and `CheatDatabasePruner` (EmuSen_Backend.md
//! §15.3, §40.1); a database's install is the session table's cheat op `DB_UPDATE` since the frontend API's 2.0. A
//! database is a handle, scanned once and never changed, so any thread may ask it.

use crate::{fail, given, put, required, status};
use emusen_cheats::cht::{self, Cheat};
use emusen_cheats::database::Database;
use emusen_cheats::prune::{self, Pruned};
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

/// A `.cht` text read: `{"text", "codec", "space", "patch"}` in, each codec an emusen-systems id or null; out `{"cheats":
/// [{desc, enabled, kind, writes}], "skipped"}`. BAD_ARGUMENT where a codec's range fault escaped the C#'s read.
///
/// # Safety
/// `request` must be valid for `request_len` bytes; `out` for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_cheats_parse(request_data: *const u8, request_len: usize, out: *mut u8, len: usize) -> i64 {
    let run = || -> Result<String, i32> {
        let r = request(request_data, request_len)?;
        let parsed = cht::parse(text(&r, "text")?.unwrap_or(""), text(&r, "codec")?, text(&r, "space")?.unwrap_or(""), text(&r, "patch")?)
            .map_err(|_| fail(status::BAD_ARGUMENT, "a codec's index fell outside its code"))?;
        Ok(compact(&object(vec![("cheats", Value::Array(parsed.cheats.iter().map(cheat_json).collect())), ("skipped", Value::from(parsed.skipped))])))
    };
    answer(run(), out, len)
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
}
