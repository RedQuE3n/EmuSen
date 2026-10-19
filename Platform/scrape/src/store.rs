//! `media.db` beside the media it describes, `MediaStore` ported: scraped games by hash, their files, the path index,
//! the queue, the day's quota, the layout step and the orphans' cleanup. Opened as Microsoft.Data.Sqlite opens it, and
//! brought to this build's schema by the C#'s three migrations to the byte. See EmuSen_BigPicture.md §5.6, §17.5, §30,
//! §38 and EmuSen_Backend.md §30.6.

use crate::Error;
use crate::kinds;
use emusen_dotnet::date::{instant, utc_millis};
use emusen_sqlite::{Bound, Connection, Row, Sqlite};
use std::sync::Arc;

/// Each entry takes the file from the version before it; append, never edit.
pub const MIGRATIONS: [&str; 3] = [
    r#"CREATE TABLE scrape_game (
    md5           TEXT    NOT NULL,
    bytes         INTEGER NOT NULL,
    status        TEXT    NOT NULL CHECK (status IN ('Found', 'Unknown', 'Error')),
    game_id       INTEGER,
    rom_id        INTEGER,
    system_id     INTEGER,
    name          TEXT,
    description   TEXT,
    developer     TEXT,
    publisher     TEXT,
    genre         TEXT,
    players       TEXT,
    rating        REAL,
    release_date  TEXT,
    region        TEXT,
    language      TEXT,
    matched_by    TEXT,
    offered       TEXT,
    detail        TEXT,
    fetched_at    TEXT    NOT NULL,
    PRIMARY KEY (md5, bytes)
);
CREATE TABLE scrape_media (
    md5         TEXT    NOT NULL,
    bytes       INTEGER NOT NULL,
    type        TEXT    NOT NULL,
    path        TEXT    NOT NULL,
    region      TEXT,
    sha1        TEXT,
    fetched_at  TEXT    NOT NULL,
    PRIMARY KEY (md5, bytes, type)
);
CREATE TABLE scrape_file (
    path      TEXT    PRIMARY KEY,
    bytes     INTEGER NOT NULL,
    modified  INTEGER NOT NULL,
    md5       TEXT    NOT NULL
);
CREATE TABLE scrape_queue (
    path      TEXT    PRIMARY KEY,
    system    TEXT    NOT NULL,
    priority  INTEGER NOT NULL,
    seq       INTEGER NOT NULL,
    attempts  INTEGER NOT NULL DEFAULT 0,
    next_try  TEXT
);
CREATE INDEX scrape_queue_order ON scrape_queue(priority, seq);
CREATE TABLE quota_day (
    day             TEXT    PRIMARY KEY,
    requests        INTEGER NOT NULL DEFAULT 0,
    ko              INTEGER NOT NULL DEFAULT 0,
    max_per_day     INTEGER,
    max_ko_per_day  INTEGER,
    stopped_until   TEXT,
    stop_reason     TEXT
);"#,
    r#"CREATE TABLE store_move (
    from_path  TEXT    NOT NULL,
    to_path    TEXT    NOT NULL,
    how        TEXT    NOT NULL CHECK (how IN ('moved', 'copied', 'found')),
    at         TEXT    NOT NULL,
    PRIMARY KEY (from_path, to_path)
);
CREATE TABLE store_step (
    name     TEXT    PRIMARY KEY,
    done_at  TEXT    NOT NULL
);"#,
    r#"ALTER TABLE scrape_game ADD COLUMN kinds_known TEXT;"#,
];

pub const SCHEMA_VERSION: i64 = MIGRATIONS.len() as i64;

pub const FILE_NAME: &str = "media.db";
pub const FOLDERS_STEP: &str = "folders";
pub const CLEANUP_FOLDER: &str = "CLEANUP";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Found,
    Unknown,
    Error,
}

impl State {
    pub fn word(self) -> &'static str {
        match self {
            State::Found => "Found",
            State::Unknown => "Unknown",
            State::Error => "Error",
        }
    }

    fn parse(text: &str) -> State {
        match text {
            "Found" => State::Found,
            "Unknown" => State::Unknown,
            _ => State::Error,
        }
    }
}

/// `ScrapePriority`: shown, a console, the whole library.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Shown = 0,
    Console = 1,
    Library = 2,
}

impl Priority {
    pub fn from_i64(v: i64) -> Priority {
        match v {
            0 => Priority::Shown,
            1 => Priority::Console,
            _ => Priority::Library,
        }
    }

    pub fn word(self) -> &'static str {
        match self {
            Priority::Shown => "shown",
            Priority::Console => "console",
            Priority::Library => "library",
        }
    }

    pub fn named(word: &str) -> Option<Priority> {
        match word {
            "shown" => Some(Priority::Shown),
            "console" => Some(Priority::Console),
            "library" => Some(Priority::Library),
            _ => None,
        }
    }
}

/// What a game's scrape kept: `ScrapedRecord`.
#[derive(Clone, Debug, PartialEq)]
pub struct Record {
    pub md5: String,
    pub bytes: i64,
    pub state: State,
    pub game_id: Option<i64>,
    pub rom_id: Option<i64>,
    pub system_id: Option<i32>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub genre: Option<String>,
    pub players: Option<String>,
    pub rating: Option<f32>,
    pub release_date: Option<String>,
    pub region: Option<String>,
    pub language: Option<String>,
    pub matched_by: Option<String>,
    pub offered: Option<String>,
    pub detail: Option<String>,
    /// UTC ticks.
    pub fetched_at: i64,
    pub kinds_known: Option<String>,
}

impl Record {
    pub fn new(md5: &str, bytes: i64, state: State, fetched_at: i64) -> Record {
        Record {
            md5: md5.to_string(),
            bytes,
            state,
            game_id: None,
            rom_id: None,
            system_id: None,
            name: None,
            description: None,
            developer: None,
            publisher: None,
            genre: None,
            players: None,
            rating: None,
            release_date: None,
            region: None,
            language: None,
            matched_by: None,
            offered: None,
            detail: None,
            fetched_at,
            kinds_known: None,
        }
    }

    /// Whether this answer says the game has a kind, or cannot say because the kind came after it was kept.
    pub fn may_offer(&self, esde_type: &str) -> bool {
        let offered = self.offered.as_deref().unwrap_or("");
        if offered.split(',').filter(|s| !s.is_empty()).any(|s| s == esde_type) {
            return true;
        }
        match &self.kinds_known {
            Some(known) => !known.split(',').filter(|s| !s.is_empty()).any(|s| s == esde_type),
            None => !kinds::FIRST.iter().any(|k| k.esde_type == esde_type),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stored {
    pub ty: String,
    pub relative_path: String,
    pub region: Option<String>,
    pub sha1: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Queued {
    pub path: String,
    pub system: String,
    pub priority: Priority,
    pub attempts: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Day {
    pub day: String,
    pub requests: i32,
    pub ko: i32,
    pub max_per_day: Option<i32>,
    pub max_ko_per_day: Option<i32>,
    /// UTC ticks.
    pub stopped_until: Option<i64>,
    pub stop_reason: Option<String>,
}

impl Day {
    pub fn empty(day: &str) -> Day {
        Day { day: day.to_string(), requests: 0, ko: 0, max_per_day: None, max_ko_per_day: None, stopped_until: None, stop_reason: None }
    }
}

pub fn stamp(ticks: i64) -> String {
    utc_millis(ticks)
}

fn text(v: &Option<String>) -> Bound<'_> {
    match v {
        Some(t) => Bound::Text(t),
        None => Bound::Null,
    }
}

fn int(v: Option<i64>) -> Bound<'static> {
    match v {
        Some(i) => Bound::Integer(i),
        None => Bound::Null,
    }
}

const GAME_COLUMNS: &str = "SELECT md5, bytes, status, game_id, rom_id, system_id, name, description, developer, publisher, genre, players, rating, release_date, region, language, matched_by, offered, detail, fetched_at, kinds_known FROM scrape_game";

fn read_game(row: &Row, o: i32) -> Record {
    let opt = |i: i32| row.text_or_null(o + i);
    let int = |i: i32| if row.is_null(o + i) { None } else { Some(row.int64(o + i)) };
    Record {
        md5: row.text(o),
        bytes: row.int64(o + 1),
        state: State::parse(&row.text(o + 2)),
        game_id: int(3),
        rom_id: int(4),
        system_id: int(5).map(|v| v as i32),
        name: opt(6),
        description: opt(7),
        developer: opt(8),
        publisher: opt(9),
        genre: opt(10),
        players: opt(11),
        rating: if row.is_null(o + 12) { None } else { Some(row.double(o + 12) as f32) },
        release_date: opt(13),
        region: opt(14),
        language: opt(15),
        matched_by: opt(16),
        offered: opt(17),
        detail: opt(18),
        fetched_at: instant(&row.text(o + 19)).unwrap_or(0),
        kinds_known: opt(20),
    }
}

/// One `media.db` and its folder.
pub struct Store {
    db: Connection,
    pub root: String,
}

fn begin(db: &Connection) -> Result<(), Error> {
    db.execute("PRAGMA read_uncommitted = 0;")?;
    db.execute("BEGIN IMMEDIATE;")?;
    Ok(())
}

fn transaction<T>(db: &Connection, work: impl FnOnce(&Connection) -> Result<T, Error>) -> Result<T, Error> {
    begin(db)?;
    match work(db) {
        Ok(v) => {
            db.execute("COMMIT;")?;
            Ok(v)
        }
        Err(e) => {
            let _ = db.execute("ROLLBACK;");
            Err(e)
        }
    }
}

fn scalar_i64(db: &Connection, sql: &str, parameters: &[(&str, Bound)]) -> Result<Option<i64>, Error> {
    let mut found = None;
    db.query(sql, parameters, |row| {
        if found.is_none() {
            found = Some(if row.is_null(0) { None } else { Some(row.int64(0)) });
        }
    })?;
    Ok(found.flatten())
}

fn scalar_text(db: &Connection, sql: &str, parameters: &[(&str, Bound)]) -> Result<Option<String>, Error> {
    let mut found = None;
    db.query(sql, parameters, |row| {
        if found.is_none() {
            found = Some(if row.kind(0) == emusen_sqlite::Kind::Text { Some(row.text(0)) } else { None });
        }
    })?;
    Ok(found.flatten())
}

impl Store {
    /// Opens `<root>/media.db`, the folder made first, migrating it; a newer file is refused unwritten.
    pub fn open(sqlite: Arc<Sqlite>, root: &str) -> Result<Store, Error> {
        std::fs::create_dir_all(root).map_err(|e| Error::Io(emusen_dotnet::file::io_message(&e, root)))?;
        let path = emusen_galaxia::dotnet_path::combine(emusen_galaxia::dotnet_path::Style::HOST, root, FILE_NAME);
        let mut db = Connection::open(sqlite, &path)?;
        db.set_timeout(emusen_sqlite::COMMAND_TIMEOUT);
        let mut version = scalar_i64(&db, "PRAGMA user_version", &[])?.unwrap_or(0);
        if version > SCHEMA_VERSION {
            return Err(Error::Newer(format!("media.db is schema {version}; this build knows {SCHEMA_VERSION}.")));
        }
        while version < SCHEMA_VERSION {
            transaction(&db, |db| {
                db.execute(MIGRATIONS[version as usize])?;
                db.execute(&format!("PRAGMA user_version = {}", version + 1))?;
                Ok(())
            })?;
            version += 1;
        }
        Ok(Store { db, root: root.to_string() })
    }

    fn write(&self, sql: &str, parameters: &[(&str, Bound)]) -> Result<i32, Error> {
        Ok(self.db.query(sql, parameters, |_| {})?)
    }

    // --- the queue ---

    /// A game already queued keeps its place unless the new priority is higher; a new one goes to the back of its priority.
    pub fn enqueue(&self, path: &str, system: &str, priority: Priority) -> Result<bool, Error> {
        Ok(self.write(
            "INSERT INTO scrape_queue (path, system, priority, seq) VALUES ($path, $system, $priority, (SELECT COALESCE(MAX(seq), 0) + 1 FROM scrape_queue))
ON CONFLICT(path) DO UPDATE SET priority = MIN(priority, excluded.priority), next_try = CASE WHEN excluded.priority < priority THEN NULL ELSE next_try END",
            &[("$path", Bound::Text(path)), ("$system", Bound::Text(system)), ("$priority", Bound::Integer(priority as i64))],
        )? > 0)
    }

    /// The first due game by priority then arrival, skipping those another worker holds.
    pub fn next_due(&self, now: i64, held: &dyn Fn(&str) -> bool) -> Result<Option<Queued>, Error> {
        let mut found = None;
        let now = stamp(now);
        self.db.query("SELECT path, system, priority, attempts FROM scrape_queue WHERE next_try IS NULL OR next_try <= $now ORDER BY priority, seq", &[("$now", Bound::Text(&now))], |row| {
            if found.is_none() && !held(&row.text(0)) {
                found = Some(Queued { path: row.text(0), system: row.text(1), priority: Priority::from_i64(row.int64(2)), attempts: row.int64(3) as i32 });
            }
        })?;
        Ok(found)
    }

    pub fn earliest_retry(&self) -> Result<Option<i64>, Error> {
        Ok(scalar_text(&self.db, "SELECT MIN(next_try) FROM scrape_queue WHERE next_try IS NOT NULL", &[])?.and_then(|t| instant(&t)))
    }

    pub fn retry(&self, path: &str, next_try: i64, count_attempt: bool) -> Result<(), Error> {
        let sql = if count_attempt {
            "UPDATE scrape_queue SET attempts = attempts + 1, next_try = $a WHERE path = $path"
        } else {
            "UPDATE scrape_queue SET next_try = $a WHERE path = $path"
        };
        let at = stamp(next_try);
        self.write(sql, &[("$path", Bound::Text(path)), ("$a", Bound::Text(&at))])?;
        Ok(())
    }

    pub fn dequeue(&self, path: &str) -> Result<(), Error> {
        self.write("DELETE FROM scrape_queue WHERE path = $path", &[("$path", Bound::Text(path))])?;
        Ok(())
    }

    pub fn clear_queue(&self) -> Result<(), Error> {
        self.write("DELETE FROM scrape_queue", &[])?;
        Ok(())
    }

    /// "Scrape this game" asks again even where ScreenScraper once had nothing: an Unknown or Error answer is forgotten.
    pub fn forget_unfound(&self, path: &str) -> Result<(), Error> {
        self.write(
            "DELETE FROM scrape_game WHERE status <> 'Found' AND EXISTS (SELECT 1 FROM scrape_file f WHERE f.path = $path AND f.md5 = scrape_game.md5 AND f.bytes = scrape_game.bytes)",
            &[("$path", Bound::Text(path))],
        )?;
        Ok(())
    }

    pub fn queue_length(&self) -> Result<i64, Error> {
        Ok(scalar_i64(&self.db, "SELECT COUNT(*) FROM scrape_queue", &[])?.unwrap_or(0))
    }

    pub fn queue(&self) -> Result<Vec<Queued>, Error> {
        let mut all = Vec::new();
        self.db.query("SELECT path, system, priority, attempts FROM scrape_queue ORDER BY priority, seq", &[], |row| {
            all.push(Queued { path: row.text(0), system: row.text(1), priority: Priority::from_i64(row.int64(2)), attempts: row.int64(3) as i32 });
        })?;
        Ok(all)
    }

    // --- the path index ---

    pub fn cached_md5(&self, path: &str, bytes: i64, modified: i64) -> Result<Option<String>, Error> {
        scalar_text(&self.db, "SELECT md5 FROM scrape_file WHERE path = $path AND bytes = $bytes AND modified = $modified", &[
            ("$path", Bound::Text(path)),
            ("$bytes", Bound::Integer(bytes)),
            ("$modified", Bound::Integer(modified)),
        ])
    }

    pub fn remember_file(&self, path: &str, bytes: i64, modified: i64, md5: &str) -> Result<(), Error> {
        self.write(
            "INSERT INTO scrape_file (path, bytes, modified, md5) VALUES ($path, $bytes, $modified, $md5) ON CONFLICT(path) DO UPDATE SET bytes = excluded.bytes, modified = excluded.modified, md5 = excluded.md5",
            &[("$path", Bound::Text(path)), ("$bytes", Bound::Integer(bytes)), ("$modified", Bound::Integer(modified)), ("$md5", Bound::Text(md5))],
        )?;
        Ok(())
    }

    /// Every file last hashed to a game, for telling a renamed file from a second copy.
    pub fn paths_of(&self, md5: &str, bytes: i64) -> Result<Vec<String>, Error> {
        let mut all = Vec::new();
        self.db.query("SELECT path FROM scrape_file WHERE md5 = $md5 AND bytes = $bytes", &[("$md5", Bound::Text(md5)), ("$bytes", Bound::Integer(bytes))], |row| all.push(row.text(0)))?;
        Ok(all)
    }

    // --- games and media ---

    pub fn record(&self, r: &Record) -> Result<(), Error> {
        let fetched = stamp(r.fetched_at);
        let rating = match r.rating {
            Some(f) => Bound::Real(f as f64),
            None => Bound::Null,
        };
        self.write(
            "INSERT OR REPLACE INTO scrape_game (md5, bytes, status, game_id, rom_id, system_id, name, description, developer, publisher, genre, players, rating,
    release_date, region, language, matched_by, offered, detail, fetched_at, kinds_known)
VALUES ($md5, $bytes, $status, $game, $rom, $system, $name, $description, $developer, $publisher, $genre, $players, $rating,
    $release, $region, $language, $matched, $offered, $detail, $fetched, $known)",
            &[
                ("$md5", Bound::Text(&r.md5)),
                ("$bytes", Bound::Integer(r.bytes)),
                ("$status", Bound::Text(r.state.word())),
                ("$game", int(r.game_id)),
                ("$rom", int(r.rom_id)),
                ("$system", int(r.system_id.map(i64::from))),
                ("$name", text(&r.name)),
                ("$description", text(&r.description)),
                ("$developer", text(&r.developer)),
                ("$publisher", text(&r.publisher)),
                ("$genre", text(&r.genre)),
                ("$players", text(&r.players)),
                ("$rating", rating),
                ("$release", text(&r.release_date)),
                ("$region", text(&r.region)),
                ("$language", text(&r.language)),
                ("$matched", text(&r.matched_by)),
                ("$offered", text(&r.offered)),
                ("$detail", text(&r.detail)),
                ("$fetched", Bound::Text(&fetched)),
                ("$known", text(&r.kinds_known)),
            ],
        )?;
        Ok(())
    }

    pub fn game(&self, md5: &str, bytes: i64) -> Result<Option<Record>, Error> {
        let mut found = None;
        self.db.query(&format!("{GAME_COLUMNS} WHERE md5 = $md5 AND bytes = $bytes"), &[("$md5", Bound::Text(md5)), ("$bytes", Bound::Integer(bytes))], |row| {
            if found.is_none() {
                found = Some(read_game(row, 0));
            }
        })?;
        Ok(found)
    }

    const FOUND_COLUMNS: &'static str = "g.md5, g.bytes, g.status, g.game_id, g.rom_id, g.system_id, g.name, g.description, g.developer, g.publisher, g.genre, g.players, g.rating, g.release_date, g.region, g.language, g.matched_by, g.offered, g.detail, g.fetched_at, g.kinds_known";

    /// Every found game by the path last hashed to it.
    pub fn found_by_path(&self) -> Result<Vec<(String, Record)>, Error> {
        let mut all: Vec<(String, Record)> = Vec::new();
        self.db.query(
            &format!("SELECT f.path, {} FROM scrape_file f JOIN scrape_game g ON g.md5 = f.md5 AND g.bytes = f.bytes WHERE g.status = 'Found'", Self::FOUND_COLUMNS),
            &[],
            |row| all.push((row.text(0), read_game(row, 1))),
        )?;
        Ok(all)
    }

    pub fn found_for(&self, path: &str) -> Result<Option<Record>, Error> {
        let mut found = None;
        self.db.query(
            &format!("SELECT {} FROM scrape_file f JOIN scrape_game g ON g.md5 = f.md5 AND g.bytes = f.bytes WHERE g.status = 'Found' AND f.path = $path", Self::FOUND_COLUMNS),
            &[("$path", Bound::Text(path))],
            |row| {
                if found.is_none() {
                    found = Some(read_game(row, 0));
                }
            },
        )?;
        Ok(found)
    }

    /// Every answered file by path: whether ScreenScraper knew it, and whether a cover was kept.
    pub fn outcomes_by_path(&self) -> Result<Vec<(String, State, bool)>, Error> {
        let mut all = Vec::new();
        self.db.query(
            "SELECT f.path, g.status, EXISTS (SELECT 1 FROM scrape_media m WHERE m.md5 = g.md5 AND m.bytes = g.bytes AND m.type = 'cover')
FROM scrape_file f JOIN scrape_game g ON g.md5 = f.md5 AND g.bytes = f.bytes",
            &[],
            |row| all.push((row.text(0), State::parse(&row.text(1)), row.int64(2) != 0)),
        )?;
        Ok(all)
    }

    pub fn record_media(&self, md5: &str, bytes: i64, media: &Stored, now: i64) -> Result<(), Error> {
        let at = stamp(now);
        self.write("INSERT OR REPLACE INTO scrape_media (md5, bytes, type, path, region, sha1, fetched_at) VALUES ($md5, $bytes, $type, $path, $region, $sha1, $at)", &[
            ("$md5", Bound::Text(md5)),
            ("$bytes", Bound::Integer(bytes)),
            ("$type", Bound::Text(&media.ty)),
            ("$path", Bound::Text(&media.relative_path)),
            ("$region", text(&media.region)),
            ("$sha1", text(&media.sha1)),
            ("$at", Bound::Text(&at)),
        ])?;
        Ok(())
    }

    pub fn media(&self, md5: &str, bytes: i64) -> Result<Vec<Stored>, Error> {
        let mut all = Vec::new();
        self.db.query("SELECT type, path, region, sha1 FROM scrape_media WHERE md5 = $md5 AND bytes = $bytes ORDER BY type", &[("$md5", Bound::Text(md5)), ("$bytes", Bound::Integer(bytes))], |row| {
            all.push(Stored { ty: row.text(0), relative_path: row.text(1), region: row.text_or_null(2), sha1: row.text_or_null(3) });
        })?;
        Ok(all)
    }

    /// The store's files a game's row names, and the rows of `table` keyed by that game's hash, for `forget`.
    pub(crate) fn forget_rows(&self, path: &str) -> Result<Vec<String>, Error> {
        let mut files = Vec::new();
        self.db.query("SELECT m.path FROM scrape_file f JOIN scrape_media m ON m.md5 = f.md5 AND m.bytes = f.bytes WHERE f.path = $path", &[("$path", Bound::Text(path))], |row| {
            files.push(row.text(0))
        })?;
        transaction(&self.db, |db| {
            for table in ["scrape_media", "scrape_game"] {
                db.query(
                    &format!("DELETE FROM {table} WHERE EXISTS (SELECT 1 FROM scrape_file f WHERE f.path = $path AND f.md5 = {table}.md5 AND f.bytes = {table}.bytes)"),
                    &[("$path", Bound::Text(path))],
                    |_| {},
                )?;
            }
            db.query("DELETE FROM scrape_queue WHERE path = $path", &[("$path", Bound::Text(path))], |_| {})?;
            Ok(())
        })?;
        Ok(files)
    }

    // --- the layout step and the cleanup's rows ---

    pub fn done(&self, step: &str) -> Result<bool, Error> {
        Ok(scalar_i64(&self.db, "SELECT 1 FROM store_step WHERE name = $name", &[("$name", Bound::Text(step))])?.is_some())
    }

    pub(crate) fn all_media_rows(&self) -> Result<Vec<(String, i64, String, String)>, Error> {
        let mut rows = Vec::new();
        self.db.query("SELECT md5, bytes, type, path FROM scrape_media", &[], |row| rows.push((row.text(0), row.int64(1), row.text(2), row.text(3))))?;
        Ok(rows)
    }

    pub(crate) fn all_files(&self) -> Result<Vec<(String, String, i64)>, Error> {
        let mut rows = Vec::new();
        self.db.query("SELECT path, md5, bytes FROM scrape_file", &[], |row| rows.push((row.text(0), row.text(1), row.int64(2))))?;
        Ok(rows)
    }

    pub(crate) fn record_move(&self, from: &str, to: &str, how: &str, now: i64) -> Result<(), Error> {
        let at = stamp(now);
        self.write("INSERT OR REPLACE INTO store_move (from_path, to_path, how, at) VALUES ($from, $to, $how, $at)", &[
            ("$from", Bound::Text(from)),
            ("$to", Bound::Text(to)),
            ("$how", Bound::Text(how)),
            ("$at", Bound::Text(&at)),
        ])?;
        Ok(())
    }

    pub(crate) fn repoint_media(&self, md5: &str, bytes: i64, ty: &str, path: &str) -> Result<(), Error> {
        self.write("UPDATE scrape_media SET path = $path WHERE md5 = $md5 AND bytes = $bytes AND type = $type", &[
            ("$path", Bound::Text(path)),
            ("$md5", Bound::Text(md5)),
            ("$bytes", Bound::Integer(bytes)),
            ("$type", Bound::Text(ty)),
        ])?;
        Ok(())
    }

    pub(crate) fn step_done(&self, name: &str, now: i64) -> Result<(), Error> {
        let at = stamp(now);
        self.write("INSERT INTO store_step (name, done_at) VALUES ($name, $at)", &[("$name", Bound::Text(name)), ("$at", Bound::Text(&at))])?;
        Ok(())
    }

    pub(crate) fn drop_media_at(&self, relative: &str) -> Result<(), Error> {
        self.write("DELETE FROM scrape_media WHERE path = $path", &[("$path", Bound::Text(relative))])?;
        Ok(())
    }

    /// What the layout step did, file by file, oldest first.
    pub fn moves(&self) -> Result<Vec<(String, String, String)>, Error> {
        let mut all = Vec::new();
        self.db.query("SELECT from_path, to_path, how FROM store_move ORDER BY at, from_path, to_path", &[], |row| all.push((row.text(0), row.text(1), row.text(2))))?;
        Ok(all)
    }

    // --- the quota's day ---

    pub fn day(&self, day: &str) -> Result<Day, Error> {
        let mut found = None;
        self.db.query("SELECT requests, ko, max_per_day, max_ko_per_day, stopped_until, stop_reason FROM quota_day WHERE day = $day", &[("$day", Bound::Text(day))], |row| {
            if found.is_none() {
                let opt = |i: i32| if row.is_null(i) { None } else { Some(row.int64(i) as i32) };
                found = Some(Day {
                    day: day.to_string(),
                    requests: row.int64(0) as i32,
                    ko: row.int64(1) as i32,
                    max_per_day: opt(2),
                    max_ko_per_day: opt(3),
                    stopped_until: row.text_or_null(4).and_then(|t| instant(&t)),
                    stop_reason: row.text_or_null(5),
                });
            }
        })?;
        Ok(found.unwrap_or_else(|| Day::empty(day)))
    }

    pub fn save_day(&self, d: &Day) -> Result<(), Error> {
        let stopped = d.stopped_until.map(stamp);
        self.write(
            "INSERT INTO quota_day (day, requests, ko, max_per_day, max_ko_per_day, stopped_until, stop_reason) VALUES ($day, $r, $k, $m, $mk, $s, $why)
ON CONFLICT(day) DO UPDATE SET requests = excluded.requests, ko = excluded.ko, max_per_day = excluded.max_per_day,
    max_ko_per_day = excluded.max_ko_per_day, stopped_until = excluded.stopped_until, stop_reason = excluded.stop_reason",
            &[
                ("$day", Bound::Text(&d.day)),
                ("$r", Bound::Integer(d.requests as i64)),
                ("$k", Bound::Integer(d.ko as i64)),
                ("$m", int(d.max_per_day.map(i64::from))),
                ("$mk", int(d.max_ko_per_day.map(i64::from))),
                ("$s", text(&stopped)),
                ("$why", text(&d.stop_reason)),
            ],
        )?;
        Ok(())
    }
}
