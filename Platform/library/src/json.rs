//! The requests `emusen_library_get` and `emusen_library_change` take, read, and their answers written, as
//! `emusen_library.h` describes them: one `what` or `op` a call, each a `GameRecords` method's. See EmuSen_Backend.md
//! §28.4.

use crate::{Error, Library, clock, records};
use emusen_dotnet::json::{self as dj, Value};

pub(crate) fn parse(text: &str) -> Result<Value, Error> {
    let value = dj::parse(text).map_err(|e| Error::BadRequest(format!("The request is not JSON: {e}")))?;
    if !matches!(value, Value::Object(_)) {
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
        Some(Value::String(t)) if !t.broken => Ok(t.text.clone()),
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
        Some(Value::Number(token)) => token.parse::<i64>().map_err(|_| wrong(name, "a whole number")),
        Some(_) => Err(wrong(name, "a whole number")),
        None => Err(missing(name)),
    }
}

/// A double from its token, or from the names .NET writes for one JSON has no number for.
fn real(r: &Value, name: &str) -> Result<f64, Error> {
    match r.get(name) {
        Some(Value::Number(token)) => token.parse::<f64>().map_err(|_| wrong(name, "a number")),
        Some(Value::String(t)) => match t.text.as_str() {
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
        Some(Value::Array(items)) => items.iter().map(|v| match v {
            Value::String(t) if !t.broken => Ok(t.text.clone()),
            _ => Err(wrong(name, "a list of texts")),
        }).collect(),
        Some(_) => Err(wrong(name, "a list of texts")),
        None => Err(missing(name)),
    }
}

/// The time to record: the request's `at`, else now.
fn at(r: &Value) -> Result<String, Error> {
    Ok(text_or_none(r, "at")?.unwrap_or_else(clock::now))
}

pub(crate) fn write_text_or_null(out: &mut String, text: Option<&str>) {
    match text {
        Some(t) => out.push_str(&dj::quoted(t)),
        None => out.push_str("null"),
    }
}

pub(crate) fn write_strings<'a>(out: &mut String, items: impl Iterator<Item = &'a str>) {
    out.push('[');
    for (i, item) in items.enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&dj::quoted(item));
    }
    out.push(']');
}

/// A double as .NET writes it, or the name it writes for one JSON has no number for.
fn write_real(out: &mut String, x: f64) {
    match dj::format_f64(x) {
        Some(token) => out.push_str(&token),
        None => out.push_str(if x.is_nan() { "\"NaN\"" } else if x > 0.0 { "\"Infinity\"" } else { "\"-Infinity\"" }),
    }
}

fn write_record(out: &mut String, r: &records::Record) {
    out.push_str(&format!("{{\"favourite\":{},\"last_played\":", r.favourite));
    write_text_or_null(out, r.last_played.as_deref());
    out.push_str(&format!(",\"play_count\":{},\"play_seconds\":", r.play_count));
    write_real(out, r.play_seconds);
    out.push('}');
}

fn write_crop(out: &mut String, crop: Option<records::Crop>) {
    let Some(c) = crop else { return out.push_str("null") };
    for (i, (name, x)) in [("left", c.left), ("right", c.right), ("top", c.top), ("bottom", c.bottom)].into_iter().enumerate() {
        out.push_str(if i == 0 { "{\"" } else { ",\"" });
        out.push_str(name);
        out.push_str("\":");
        write_real(out, x);
    }
    out.push('}');
}

fn write_edits(out: &mut String, edits: &[(String, String)]) {
    out.push('{');
    for (i, (field, value)) in edits.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&dj::quoted(field));
        out.push(':');
        out.push_str(&dj::quoted(value));
    }
    out.push('}');
}

fn write_ids(out: &mut String, ids: &[i64]) {
    out.push('[');
    out.push_str(&ids.iter().map(i64::to_string).collect::<Vec<_>>().join(","));
    out.push(']');
}

/// One read.
pub(crate) fn get(lib: &Library, r: &Value) -> Result<String, Error> {
    let what = text(r, "what")?;
    let mut out = String::new();
    match what.as_str() {
        "record" => match records::find(&*lib.db()?, &text(r, "path")?)? {
            Some(record) => write_record(&mut out, &record),
            None => out.push_str("null"),
        },
        "records" => {
            out.push('{');
            for (i, (path, record)) in records::all(&*lib.db()?)?.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&dj::quoted(path));
                out.push(':');
                write_record(&mut out, record);
            }
            out.push('}');
        }
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
                    write_record(&mut out, &record);
                    out.pop();
                    out.push_str(",\"md5\":");
                    write_text_or_null(&mut out, md5.as_deref());
                    out.push_str(",\"bytes\":");
                    out.push_str(&bytes.map_or("null".to_string(), |b| b.to_string()));
                    out.push_str(",\"crop\":");
                    write_crop(&mut out, records::crop(&db, &path)?);
                    out.push_str(",\"edits\":");
                    write_edits(&mut out, &records::edits(&db, &path)?);
                    out.push_str(",\"collections\":");
                    write_ids(&mut out, &records::collections_of(&db, &path)?);
                    out.push('}');
                }
                _ => out.push_str("null"),
            }
        }
        "edits" => write_edits(&mut out, &records::edits(&*lib.db()?, &text(r, "path")?)?),
        "all_edits" => {
            out.push('{');
            for (i, (path, edits)) in records::all_edits(&*lib.db()?)?.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&dj::quoted(path));
                out.push(':');
                write_edits(&mut out, edits);
            }
            out.push('}');
        }
        "collections" => {
            out.push('[');
            for (i, c) in records::collections(&*lib.db()?)?.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&format!("{{\"id\":{},\"name\":{},\"count\":{}}}", c.id, dj::quoted(&c.name), c.count));
            }
            out.push(']');
        }
        "members" => write_strings(&mut out, records::members(&*lib.db()?, integer(r, "id")?)?.iter().map(String::as_str)),
        "collections_of" => write_ids(&mut out, &records::collections_of(&*lib.db()?, &text(r, "path")?)?),
        "crop" => write_crop(&mut out, records::crop(&*lib.db()?, &text(r, "path")?)?),
        "cover_lookup" => write_text_or_null(&mut out, records::cover_lookup(&*lib.db()?, &text(r, "path")?)?.as_deref()),
        "known_hash" => write_text_or_null(&mut out, records::known_hash(&*lib.db()?, &text(r, "path")?, integer(r, "bytes")?, integer(r, "modified")?)?.as_deref()),
        "path_by_hash" => write_text_or_null(&mut out, records::path_by_hash(&*lib.db()?, &text(r, "md5")?)?.as_deref()),
        "orphans" => {
            let present = strings(r, "present")?.into_iter().collect();
            out.push('[');
            for (i, (path, md5, bytes)) in records::orphans(&*lib.db()?, &present)?.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&format!("{{\"path\":{},\"md5\":{},\"bytes\":{bytes}}}", dj::quoted(path), dj::quoted(md5)));
            }
            out.push(']');
        }
        "recent" => {
            let limit = integer(r, "limit")?.max(0) as usize;
            write_strings(&mut out, lib.recent(limit, games(r)?)?.iter().map(String::as_str));
        }
        other => return Err(Error::BadRequest(format!("No read is called {other}."))),
    }
    Ok(out)
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
                Value::Null => Ok((key.text.clone(), None)),
                Value::String(t) if !t.broken => Ok((key.text.clone(), Some(t.text.clone()))),
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
        return Ok(format!("{{\"moved\":{}}}", lib.reclaim_guarded(games(r)?)?));
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
                    let mut out = String::new();
                    write_real(&mut out, kept);
                    out
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
