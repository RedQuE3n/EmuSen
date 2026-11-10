//! The requests `emusen_library_get` and `emusen_library_change` take, read, and their answers written, as
//! `emusen_library.h` describes them: one `what` or `op` a call, each a `GameRecords` method's. A request is a
//! `serde_json::Value` and an answer is built with `json!`; serde_json reads and writes the text. See
//! EmuSen_Backend.md §28.4 and EmuSen_RustPlatform.md §47.13.

use crate::{Error, Library, clock, records};
use serde_json::{Map, Value, json};

pub(crate) fn parse(text: &str) -> Result<Value, Error> {
    let value: Value = serde_json::from_str(text).map_err(|e| Error::BadRequest(format!("The request is not JSON: {e}")))?;
    if !value.is_object() {
        return Err(Error::BadRequest("The request is not a JSON object.".into()));
    }
    Ok(value)
}

fn missing(name: &str) -> Error {
    Error::BadRequest(format!("The request has no {name}."))
}

fn wrong(name: &str, what: &str) -> Error {
    Error::BadRequest(format!("The request's {name} is not {what}."))
}

pub(crate) fn text(r: &Value, name: &str) -> Result<String, Error> {
    match r.get(name) {
        Some(Value::String(t)) => Ok(t.clone()),
        Some(_) => Err(wrong(name, "text")),
        None => Err(missing(name)),
    }
}

pub(crate) fn text_or_none(r: &Value, name: &str) -> Result<Option<String>, Error> {
    match r.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(_) => text(r, name).map(Some),
    }
}

pub(crate) fn flag(r: &Value, name: &str) -> Result<bool, Error> {
    match r.get(name) {
        None | Some(Value::Null) => Ok(false),
        Some(Value::Bool(b)) => Ok(*b),
        Some(_) => Err(wrong(name, "true or false")),
    }
}

fn integer(r: &Value, name: &str) -> Result<i64, Error> {
    match r.get(name) {
        Some(v) => v.as_i64().ok_or_else(|| wrong(name, "a whole number")),
        None => Err(missing(name)),
    }
}

/// A double, or the name of one JSON has no number for (`NaN`, `Infinity`, `-Infinity`).
fn real(r: &Value, name: &str) -> Result<f64, Error> {
    match r.get(name) {
        Some(Value::Number(n)) => n.as_f64().ok_or_else(|| wrong(name, "a number")),
        Some(Value::String(t)) => match t.as_str() {
            "NaN" => Ok(f64::NAN),
            "Infinity" => Ok(f64::INFINITY),
            "-Infinity" => Ok(f64::NEG_INFINITY),
            _ => Err(wrong(name, "a number")),
        },
        Some(_) => Err(wrong(name, "a number")),
        None => Err(missing(name)),
    }
}

pub(crate) fn strings(r: &Value, name: &str) -> Result<Vec<String>, Error> {
    match r.get(name) {
        Some(Value::Array(items)) => items.iter().map(|v| v.as_str().map(str::to_string).ok_or_else(|| wrong(name, "a list of texts"))).collect(),
        Some(_) => Err(wrong(name, "a list of texts")),
        None => Err(missing(name)),
    }
}

/// The time to record: the request's `at`, else now.
fn at(r: &Value) -> Result<String, Error> {
    Ok(text_or_none(r, "at")?.unwrap_or_else(clock::now))
}

/// A double as a number, or the name of one JSON has no number for.
fn real_value(x: f64) -> Value {
    if x.is_finite() {
        json!(x)
    } else {
        json!(if x.is_nan() { "NaN" } else if x > 0.0 { "Infinity" } else { "-Infinity" })
    }
}

fn record_value(r: &records::Record) -> Map<String, Value> {
    let Value::Object(m) = json!({ "favourite": r.favourite, "last_played": r.last_played, "play_count": r.play_count, "play_seconds": real_value(r.play_seconds) }) else { unreachable!() };
    m
}

fn crop_value(crop: Option<records::Crop>) -> Value {
    crop.map_or(Value::Null, |c| json!({ "left": real_value(c.left), "right": real_value(c.right), "top": real_value(c.top), "bottom": real_value(c.bottom) }))
}

fn edits_value(edits: &[(String, String)]) -> Value {
    Value::Object(edits.iter().map(|(field, value)| (field.clone(), json!(value))).collect())
}

/// One read.
pub(crate) fn get(lib: &Library, r: &Value) -> Result<String, Error> {
    let what = text(r, "what")?;
    let answer = match what.as_str() {
        "record" => records::find(&*lib.db()?, &text(r, "path")?)?.map_or(Value::Null, |record| Value::Object(record_value(&record))),
        "records" => Value::Object(records::all(&*lib.db()?)?.iter().map(|(path, record)| (path.clone(), Value::Object(record_value(record)))).collect()),
        "game" => {
            let path = text(r, "path")?;
            let db = lib.db()?;
            let mut found = None;
            db.query("SELECT md5, bytes FROM game WHERE path = $path", &[("$path", emusen_sqlite::Bound::Text(&path))], |row| {
                if found.is_none() {
                    found = Some((row.text_or_null(0), if row.is_null(1) { None } else { Some(row.int64(1)) }));
                }
            })?;
            match (records::find(&db, &path)?, found) {
                (Some(record), Some((md5, bytes))) => {
                    let mut game = record_value(&record);
                    game.insert("md5".into(), json!(md5));
                    game.insert("bytes".into(), json!(bytes));
                    game.insert("crop".into(), crop_value(records::crop(&db, &path)?));
                    game.insert("edits".into(), edits_value(&records::edits(&db, &path)?));
                    game.insert("collections".into(), json!(records::collections_of(&db, &path)?));
                    Value::Object(game)
                }
                _ => Value::Null,
            }
        }
        "edits" => edits_value(&records::edits(&*lib.db()?, &text(r, "path")?)?),
        "all_edits" => Value::Object(records::all_edits(&*lib.db()?)?.iter().map(|(path, edits)| (path.clone(), edits_value(edits))).collect()),
        "collections" => Value::Array(records::collections(&*lib.db()?)?.iter().map(|c| json!({ "id": c.id, "name": c.name, "count": c.count })).collect()),
        "members" => json!(records::members(&*lib.db()?, integer(r, "id")?)?),
        "collections_of" => json!(records::collections_of(&*lib.db()?, &text(r, "path")?)?),
        "crop" => crop_value(records::crop(&*lib.db()?, &text(r, "path")?)?),
        "standard" => json!({ "value": records::standard(&*lib.db()?, &text(r, "path")?)?, "values": records::STANDARDS }),
        "cover_lookup" => json!(records::cover_lookup(&*lib.db()?, &text(r, "path")?)?),
        "known_hash" => json!(records::known_hash(&*lib.db()?, &text(r, "path")?, integer(r, "bytes")?, integer(r, "modified")?)?),
        "path_by_hash" => json!(records::path_by_hash(&*lib.db()?, &text(r, "md5")?)?),
        "orphans" => {
            let present = strings(r, "present")?.into_iter().collect();
            Value::Array(records::orphans(&*lib.db()?, &present)?.iter().map(|(path, md5, bytes)| json!({ "path": path, "md5": md5, "bytes": bytes })).collect())
        }
        "recent" => {
            let limit = integer(r, "limit")?.max(0) as usize;
            json!(lib.recent(limit, games(r)?)?)
        }
        other => return Err(Error::BadRequest(format!("No read is called {other}."))),
    };
    Ok(answer.to_string())
}

/// The games a reclaim or `recent` is given in place of the last scan's.
fn games(r: &Value) -> Result<Option<Vec<String>>, Error> {
    match r.get("games") {
        None | Some(Value::Null) => Ok(None),
        Some(_) => strings(r, "games").map(Some),
    }
}

fn changes(r: &Value) -> Result<Vec<(String, Option<String>)>, Error> {
    match r.get("changes") {
        Some(Value::Object(members)) => members
            .iter()
            .map(|(key, value)| match value {
                Value::Null => Ok((key.clone(), None)),
                Value::String(t) => Ok((key.clone(), Some(t.clone()))),
                _ => Err(wrong("changes", "an object of texts and nulls")),
            })
            .collect(),
        Some(_) => Err(wrong("changes", "an object of texts and nulls")),
        None => Err(missing("changes")),
    }
}

/// Microsoft.Data.Sqlite's refusal of a NaN bound as a parameter, before the statement runs.
fn storable(values: &[f64]) -> Result<(), Error> {
    if values.iter().any(|v| v.is_nan()) { Err(Error::BadRequest("Cannot store 'NaN' values.".into())) } else { Ok(()) }
}

/// One change; `null` where the C# method returns nothing.
pub(crate) fn change(lib: &Library, r: &Value) -> Result<String, Error> {
    let op = text(r, "op")?;
    if op == "reclaim" {
        return Ok(json!({ "moved": lib.reclaim_guarded(games(r)?)? }).to_string());
    }
    let db = lib.db()?;
    let db = &*db;
    let path = || text(r, "path");
    match op.as_str() {
        "toggle_favourite" => records::toggle_favourite(db, &path()?)?,
        "set_favourite" => records::set_favourite(db, &path()?, flag(r, "favourite")?)?,
        "started" => records::started(db, &path()?, &at(r)?)?,
        "played" => {
            let seconds = real(r, "seconds")?;
            let limit = match r.get("max_hours") {
                None | Some(Value::Null) => 24,
                Some(_) => integer(r, "max_hours")?.clamp(i32::MIN as i64, i32::MAX as i64) as i32,
            };
            return Ok(match records::tracked(seconds, limit) {
                Some(kept) => {
                    storable(&[kept])?;
                    records::played(db, &path()?, kept)?;
                    real_value(kept).to_string()
                }
                None => "null".to_string(),
            });
        }
        "set_play_stats" => {
            let seconds = real(r, "seconds")?;
            storable(&[seconds])?;
            records::set_play_stats(db, &path()?, integer(r, "count")?.clamp(i32::MIN as i64, i32::MAX as i64) as i32, seconds)?
        }
        "set_crop" => {
            let crop = records::Crop { left: real(r, "left")?, right: real(r, "right")?, top: real(r, "top")?, bottom: real(r, "bottom")? };
            storable(&[crop.left, crop.right, crop.top, crop.bottom])?;
            records::set_crop(db, &path()?, crop)?
        }
        "clear_crop" => records::clear_crop(db, &path()?)?,
        "set_standard" => records::set_standard(db, &path()?, text_or_none(r, "standard")?.as_deref(), &at(r)?)?,
        "save_edits" => records::save_edits(db, &path()?, &changes(r)?, &at(r)?)?,
        "clear_edits" => records::clear_edits(db, &path()?)?,
        "create_collection" => {
            return Ok(records::create_collection(db, &text(r, "name")?, &at(r)?)?.map_or("null".to_string(), |id| id.to_string()));
        }
        "rename_collection" => return Ok(records::rename_collection(db, integer(r, "id")?, &text(r, "name")?)?.to_string()),
        "delete_collection" => records::delete_collection(db, integer(r, "id")?)?,
        "add_to_collection" => records::add_to_collection(db, integer(r, "id")?, &path()?)?,
        "remove_from_collection" => records::remove_from_collection(db, integer(r, "id")?, &path()?)?,
        "store_hash" => records::store_hash(db, &path()?, integer(r, "bytes")?, integer(r, "modified")?, &text(r, "md5")?)?,
        "identify" => records::identify(db, &path()?, &text(r, "md5")?, integer(r, "bytes")?)?,
        "move" => records::move_game(db, &text(r, "from")?, &text(r, "to")?)?,
        "record_cover_lookup" => records::record_cover_lookup(db, &path()?, &text(r, "outcome")?, text_or_none(r, "rom_name")?.as_deref(), &at(r)?)?,
        "forget_cover_lookup" => records::forget_cover_lookup(db, &path()?)?,
        other => return Err(Error::BadRequest(format!("No change is called {other}."))),
    }
    Ok("null".to_string())
}
