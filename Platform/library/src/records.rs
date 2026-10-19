//! The player's records in `games.db`, the C# `GameRecords`' every row operation with its SQL: favourites, play time,
//! the hash cache, a game's identity and its move, cover lookups, collections, metadata edits and a game's crop. See
//! EmuSen_Settings_Reference.md §4.32 to §4.100 and EmuSen_Backend.md §28.4.

use crate::Error;
use crate::db::{scalar_i64, scalar_text, transaction};
use emusen_sqlite::{Bound, Connection, Row};

/// The edit fields that are the player's choices rather than metadata, kept by the editor's Clear: `GameMetadata`'s
/// `CoverFrom` and `TelevisionStandard`.
pub const COVER_FROM: &str = "coverfrom";
pub const TELEVISION_STANDARD: &str = "tvstandard";

/// What the library remembers of one game: the C# `GameRecord`, its time as the text it was stored as.
#[derive(Clone, Debug, PartialEq)]
pub struct Record {
    pub favourite: bool,
    pub last_played: Option<String>,
    pub play_count: i32,
    pub play_seconds: f64,
}

/// One of the player's collections, with how many games it holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Collection {
    pub id: i64,
    pub name: String,
    pub count: i32,
}

/// A game's own crop, a percentage of its picture hidden at each edge.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Crop {
    pub left: f64,
    pub right: f64,
    pub top: f64,
    pub bottom: f64,
}

fn record(row: &Row, first: i32) -> Record {
    Record { favourite: row.int64(first) != 0, last_played: row.text_or_null(first + 1), play_count: row.int64(first + 2) as i32, play_seconds: row.double(first + 3) }
}

fn write(db: &Connection, path: &str, sql: &str, value: Option<Bound>) -> Result<(), Error> {
    match value {
        Some(value) => db.query(sql, &[("$path", Bound::Text(path)), ("$value", value)], |_| {})?,
        None => db.query(sql, &[("$path", Bound::Text(path))], |_| {})?,
    };
    Ok(())
}

pub fn find(db: &Connection, path: &str) -> Result<Option<Record>, Error> {
    let mut found = None;
    db.query("SELECT favourite, last_played, play_count, play_seconds FROM game WHERE path = $path", &[("$path", Bound::Text(path))], |row| {
        if found.is_none() {
            found = Some(record(row, 0));
        }
    })?;
    Ok(found)
}

/// Every stored row, in the order SQLite gives them.
pub fn all(db: &Connection) -> Result<Vec<(String, Record)>, Error> {
    let mut rows = Vec::new();
    db.query("SELECT path, favourite, last_played, play_count, play_seconds FROM game", &[], |row| rows.push((row.text(0), record(row, 1))))?;
    Ok(rows)
}

pub fn toggle_favourite(db: &Connection, path: &str) -> Result<(), Error> {
    write(db, path, "INSERT INTO game (path, favourite) VALUES ($path, 1) ON CONFLICT(path) DO UPDATE SET favourite = 1 - favourite", None)
}

pub fn set_favourite(db: &Connection, path: &str, favourite: bool) -> Result<(), Error> {
    write(db, path, "INSERT INTO game (path, favourite) VALUES ($path, $value) ON CONFLICT(path) DO UPDATE SET favourite = excluded.favourite", Some(Bound::Integer(favourite as i64)))
}

pub fn started(db: &Connection, path: &str, at: &str) -> Result<(), Error> {
    write(
        db,
        path,
        "INSERT INTO game (path, play_count, last_played) VALUES ($path, 1, $value) ON CONFLICT(path) DO UPDATE SET play_count = play_count + 1, last_played = excluded.last_played",
        Some(Bound::Text(at)),
    )
}

pub fn played(db: &Connection, path: &str, seconds: f64) -> Result<(), Error> {
    write(db, path, "INSERT INTO game (path, play_seconds) VALUES ($path, $value) ON CONFLICT(path) DO UPDATE SET play_seconds = play_seconds + excluded.play_seconds", Some(Bound::Real(seconds)))
}

/// `PlayTime.Tracked`: what one launch of `seconds` adds under a limit of `max_hours`, none where tracking is off (0
/// or less) or the launch ran past the limit; 24 or more is no limit.
pub fn tracked(seconds: f64, max_hours: i32) -> Option<f64> {
    match max_hours {
        h if h <= 0 => None,
        h if h >= 24 => Some(seconds),
        h => {
            if seconds > h as f64 * 3600.0 { None } else { Some(seconds) }
        }
    }
}

/// `Math.Max(0, x)` as .NET has it, a NaN given back.
fn at_least_zero(x: f64) -> f64 {
    if x.is_nan() { x } else { x.max(0.0) }
}

pub fn set_play_stats(db: &Connection, path: &str, count: i32, seconds: f64) -> Result<(), Error> {
    db.query(
        "INSERT INTO game (path, play_count, play_seconds) VALUES ($path, $count, $seconds) ON CONFLICT(path) DO UPDATE SET play_count = excluded.play_count, play_seconds = excluded.play_seconds",
        &[("$path", Bound::Text(path)), ("$count", Bound::Integer(count.max(0) as i64)), ("$seconds", Bound::Real(at_least_zero(seconds)))],
        |_| {},
    )?;
    Ok(())
}

pub fn known_hash(db: &Connection, path: &str, bytes: i64, modified: i64) -> Result<Option<String>, Error> {
    scalar_text(
        db,
        "SELECT md5 FROM file_hash WHERE path = $path AND bytes = $bytes AND modified = $modified",
        &[("$path", Bound::Text(path)), ("$bytes", Bound::Integer(bytes)), ("$modified", Bound::Integer(modified))],
    )
}

pub fn store_hash(db: &Connection, path: &str, bytes: i64, modified: i64, md5: &str) -> Result<(), Error> {
    db.query(
        "INSERT INTO file_hash (path, bytes, modified, md5) VALUES ($path, $bytes, $modified, $md5) ON CONFLICT(path) DO UPDATE SET bytes = excluded.bytes, modified = excluded.modified, md5 = excluded.md5",
        &[("$path", Bound::Text(path)), ("$bytes", Bound::Integer(bytes)), ("$modified", Bound::Integer(modified)), ("$md5", Bound::Text(md5))],
        |_| {},
    )?;
    Ok(())
}

pub fn path_by_hash(db: &Connection, md5: &str) -> Result<Option<String>, Error> {
    scalar_text(db, "SELECT path FROM file_hash WHERE md5 = $md5 ORDER BY path LIMIT 1", &[("$md5", Bound::Text(md5))])
}

pub fn identify(db: &Connection, path: &str, md5: &str, bytes: i64) -> Result<(), Error> {
    db.query(
        "INSERT INTO game (path, md5, bytes) VALUES ($path, $md5, $bytes) ON CONFLICT(path) DO UPDATE SET md5 = excluded.md5, bytes = excluded.bytes",
        &[("$path", Bound::Text(path)), ("$md5", Bound::Text(md5)), ("$bytes", Bound::Integer(bytes))],
        |_| {},
    )?;
    Ok(())
}

/// Rows whose file is not among those present, and which carry the hash that could find it again.
pub fn orphans(db: &Connection, present: &std::collections::HashSet<String>) -> Result<Vec<(String, String, i64)>, Error> {
    let mut found = Vec::new();
    db.query("SELECT path, md5, bytes FROM game WHERE md5 IS NOT NULL AND bytes IS NOT NULL", &[], |row| {
        let path = row.text(0);
        if !present.contains(&path) {
            found.push((path, row.text(1), row.int64(2)));
        }
    })?;
    Ok(found)
}

/// A renamed or moved game: its row, collections, edits and cover lookup follow it, a row the new path had merged in.
pub fn move_game(db: &Connection, from: &str, to: &str) -> Result<(), Error> {
    let both = [("$from", Bound::Text(from)), ("$to", Bound::Text(to))];
    let from_only = [("$from", Bound::Text(from))];
    let to_only = [("$to", Bound::Text(to))];
    transaction(db, |db| {
        db.query("INSERT INTO game (path) VALUES ($to) ON CONFLICT(path) DO NOTHING", &to_only, |_| {})?;
        db.query(
            "UPDATE game SET
    favourite    = MAX(favourite, (SELECT favourite FROM game WHERE path = $from)),
    play_count   = play_count + (SELECT play_count FROM game WHERE path = $from),
    play_seconds = play_seconds + (SELECT play_seconds FROM game WHERE path = $from),
    last_played  = MAX(COALESCE(last_played, ''), COALESCE((SELECT last_played FROM game WHERE path = $from), '')),
    md5          = COALESCE(md5, (SELECT md5 FROM game WHERE path = $from)),
    bytes        = COALESCE(bytes, (SELECT bytes FROM game WHERE path = $from)),
    crop_right   = CASE WHEN crop_left IS NULL THEN (SELECT crop_right FROM game WHERE path = $from) ELSE crop_right END,
    crop_top     = CASE WHEN crop_left IS NULL THEN (SELECT crop_top FROM game WHERE path = $from) ELSE crop_top END,
    crop_bottom  = CASE WHEN crop_left IS NULL THEN (SELECT crop_bottom FROM game WHERE path = $from) ELSE crop_bottom END,
    crop_left    = COALESCE(crop_left, (SELECT crop_left FROM game WHERE path = $from))
WHERE path = $to",
            &both,
            |_| {},
        )?;
        db.query("UPDATE game SET last_played = NULL WHERE path = $to AND last_played = ''", &to_only, |_| {})?;
        db.query("UPDATE OR IGNORE collection_game SET path = $to WHERE path = $from", &both, |_| {})?;
        db.query("DELETE FROM collection_game WHERE path = $from", &from_only, |_| {})?;
        db.query("UPDATE OR IGNORE game_edit SET path = $to WHERE path = $from", &both, |_| {})?;
        db.query("DELETE FROM game_edit WHERE path = $from", &from_only, |_| {})?;
        db.query(&format!("UPDATE game_edit SET value = $to WHERE field = '{COVER_FROM}' AND value = $from"), &both, |_| {})?;
        db.query("UPDATE OR IGNORE cover_lookup SET path = $to WHERE path = $from", &both, |_| {})?;
        db.query("DELETE FROM cover_lookup WHERE path = $from", &from_only, |_| {})?;
        db.query("DELETE FROM game WHERE path = $from", &from_only, |_| {})?;
        Ok(())
    })
}

pub fn cover_lookup(db: &Connection, path: &str) -> Result<Option<String>, Error> {
    scalar_text(db, "SELECT outcome FROM cover_lookup WHERE path = $path", &[("$path", Bound::Text(path))])
}

/// A failed lookup is not recorded, so the next session asks again; an answer is, until the player asks again by hand.
pub fn record_cover_lookup(db: &Connection, path: &str, outcome: &str, rom_name: Option<&str>, at: &str) -> Result<(), Error> {
    if outcome == "Failed" {
        return Ok(());
    }
    db.query(
        "INSERT INTO cover_lookup (path, outcome, rom_name, checked) VALUES ($path, $outcome, $name, $now) ON CONFLICT(path) DO UPDATE SET outcome = excluded.outcome, rom_name = excluded.rom_name, checked = excluded.checked",
        &[("$path", Bound::Text(path)), ("$outcome", Bound::Text(outcome)), ("$name", rom_name.map_or(Bound::Null, Bound::Text)), ("$now", Bound::Text(at))],
        |_| {},
    )?;
    Ok(())
}

pub fn forget_cover_lookup(db: &Connection, path: &str) -> Result<(), Error> {
    write(db, path, "DELETE FROM cover_lookup WHERE path = $path", None)
}

pub fn collections(db: &Connection) -> Result<Vec<Collection>, Error> {
    let mut all = Vec::new();
    db.query(
        "SELECT c.id, c.name, COUNT(g.path) FROM collection c LEFT JOIN collection_game g ON g.collection = c.id GROUP BY c.id ORDER BY c.name COLLATE NOCASE",
        &[],
        |row| all.push(Collection { id: row.int64(0), name: row.text(1), count: row.int64(2) as i32 }),
    )?;
    Ok(all)
}

/// The new collection's id, or none when a collection of that name, in any case, already exists.
pub fn create_collection(db: &Connection, name: &str, at: &str) -> Result<Option<i64>, Error> {
    scalar_i64(db, "INSERT INTO collection (name, created) VALUES ($name, $now) ON CONFLICT(name) DO NOTHING RETURNING id", &[("$name", Bound::Text(name)), ("$now", Bound::Text(at))])
}

/// False when another collection already has the name.
pub fn rename_collection(db: &Connection, id: i64, name: &str) -> Result<bool, Error> {
    db.query("UPDATE OR IGNORE collection SET name = $name WHERE id = $id", &[("$name", Bound::Text(name)), ("$id", Bound::Integer(id))], |_| {})?;
    Ok(collections(db)?.iter().any(|c| c.id == id && c.name == name))
}

pub fn delete_collection(db: &Connection, id: i64) -> Result<(), Error> {
    db.query("DELETE FROM collection WHERE id = $id", &[("$id", Bound::Integer(id))], |_| {})?;
    Ok(())
}

pub fn add_to_collection(db: &Connection, id: i64, path: &str) -> Result<(), Error> {
    db.query("INSERT OR IGNORE INTO collection_game (collection, path) VALUES ($id, $path)", &[("$id", Bound::Integer(id)), ("$path", Bound::Text(path))], |_| {})?;
    Ok(())
}

pub fn remove_from_collection(db: &Connection, id: i64, path: &str) -> Result<(), Error> {
    db.query("DELETE FROM collection_game WHERE collection = $id AND path = $path", &[("$id", Bound::Integer(id)), ("$path", Bound::Text(path))], |_| {})?;
    Ok(())
}

pub fn members(db: &Connection, id: i64) -> Result<Vec<String>, Error> {
    let mut found = Vec::new();
    db.query("SELECT path FROM collection_game WHERE collection = $id", &[("$id", Bound::Integer(id))], |row| found.push(row.text(0)))?;
    Ok(found)
}

pub fn collections_of(db: &Connection, path: &str) -> Result<Vec<i64>, Error> {
    let mut found = Vec::new();
    db.query("SELECT collection FROM collection_game WHERE path = $path", &[("$path", Bound::Text(path))], |row| found.push(row.int64(0)))?;
    Ok(found)
}

pub fn edits(db: &Connection, path: &str) -> Result<Vec<(String, String)>, Error> {
    let mut found: Vec<(String, String)> = Vec::new();
    db.query("SELECT field, value FROM game_edit WHERE path = $path", &[("$path", Bound::Text(path))], |row| {
        let (field, value) = (row.text(0), row.text(1));
        match found.iter_mut().find(|(f, _)| *f == field) {
            Some(slot) => slot.1 = value,
            None => found.push((field, value)),
        }
    })?;
    Ok(found)
}

/// Every game's edits, by path in the order each path was first met.
pub fn all_edits(db: &Connection) -> Result<Vec<(String, Vec<(String, String)>)>, Error> {
    let mut all: Vec<(String, Vec<(String, String)>)> = Vec::new();
    let mut at: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    db.query("SELECT path, field, value FROM game_edit", &[], |row| {
        let path = row.text(0);
        let index = *at.entry(path.clone()).or_insert_with(|| {
            all.push((path, Vec::new()));
            all.len() - 1
        });
        let (field, value) = (row.text(1), row.text(2));
        let edits = &mut all[index].1;
        match edits.iter_mut().find(|(f, _)| *f == field) {
            Some(slot) => slot.1 = value,
            None => edits.push((field, value)),
        }
    })?;
    Ok(all)
}

/// One save of the editor, in the changes' order: a value is kept as the player's edit, a none returns the field to
/// its scraped or default value.
pub fn save_edits(db: &Connection, path: &str, changes: &[(String, Option<String>)], at: &str) -> Result<(), Error> {
    transaction(db, |db| {
        for (field, value) in changes {
            match value {
                None => db.query("DELETE FROM game_edit WHERE path = $path AND field = $field", &[("$path", Bound::Text(path)), ("$field", Bound::Text(field))], |_| {})?,
                Some(value) => db.query(
                    "INSERT INTO game_edit (path, field, value, edited) VALUES ($path, $field, $value, $now) ON CONFLICT(path, field) DO UPDATE SET value = excluded.value, edited = excluded.edited",
                    &[("$path", Bound::Text(path)), ("$field", Bound::Text(field)), ("$value", Bound::Text(value)), ("$now", Bound::Text(at))],
                    |_| {},
                )?,
            };
        }
        Ok(())
    })
}

/// The editor's Clear: the cover borrowed from another game and the television standard chosen stay.
pub fn clear_edits(db: &Connection, path: &str) -> Result<(), Error> {
    write(db, path, &format!("DELETE FROM game_edit WHERE path = $path AND field NOT IN ('{COVER_FROM}', '{TELEVISION_STANDARD}')"), None)
}

/// None for a game with no crop of its own.
pub fn crop(db: &Connection, path: &str) -> Result<Option<Crop>, Error> {
    let mut found = None;
    db.query("SELECT crop_left, crop_right, crop_top, crop_bottom FROM game WHERE path = $path AND crop_left IS NOT NULL", &[("$path", Bound::Text(path))], |row| {
        if found.is_none() {
            found = Some(Crop { left: row.double(0), right: row.double(1), top: row.double(2), bottom: row.double(3) });
        }
    })?;
    Ok(found)
}

pub fn set_crop(db: &Connection, path: &str, crop: Crop) -> Result<(), Error> {
    db.query(
        "INSERT INTO game (path, crop_left, crop_right, crop_top, crop_bottom) VALUES ($path, $left, $right, $top, $bottom)
ON CONFLICT(path) DO UPDATE SET crop_left = excluded.crop_left, crop_right = excluded.crop_right, crop_top = excluded.crop_top, crop_bottom = excluded.crop_bottom",
        &[("$path", Bound::Text(path)), ("$left", Bound::Real(crop.left)), ("$right", Bound::Real(crop.right)), ("$top", Bound::Real(crop.top)), ("$bottom", Bound::Real(crop.bottom))],
        |_| {},
    )?;
    Ok(())
}

pub fn clear_crop(db: &Connection, path: &str) -> Result<(), Error> {
    write(db, path, "UPDATE game SET crop_left = NULL, crop_right = NULL, crop_top = NULL, crop_bottom = NULL WHERE path = $path", None)
}
