//! Cheats on the tool screen: whether the player's cheat database is installed, how many files it holds and the date of
//! its newest, and the library's games it has a file for, by console, each with the state of the game's own list (the
//! list the Pause Card and the game page switch, cheats.rs). The database is read through the base's own calls: its
//! folder and attribution from the session table's DB_STATUS, its files from `emusen_cheats_database_*`, and a game's
//! file by the database's own `find`, kept to the console's cheat systems. The library is matched on a thread of its own.
//! Getting the database and making a game's list from it have no call in the base yet, so neither is offered. No Qt here.

use super::{Action, Bar, Column, Item, Mark, Pane, Source, Tone, Words};
use crate::card::{Pane as CardPane, Row as CardRow};
use crate::cheats::{count_words, on_words, List, Switch};
use crate::clients::ClientSource;
use crate::library::LibrarySource;
use emusen_crystal_client::consts::CHEAT_DB_STATUS;
use emusen_crystal_client::service_flags::NO_GAME;
use emusen_platform_client::cheats::CheatsDatabase;
use serde_json::{json, Value};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;

/// A library game the database has a file for: its path, name, console, the file's system folder and game, and the
/// game's own list, none where it has none yet.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Game {
    pub path: String,
    pub name: String,
    pub console: String,
    pub file: (String, String),
    pub list: Vec<Switch>,
}

/// What a read of the database found: its folder, whether it is there, its attribution, how many files it holds, the
/// newest file's time in seconds since 1970, the consoles with games in it by id and name, and those games.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Found {
    pub directory: String,
    pub exists: bool,
    pub attribution: String,
    pub files: usize,
    pub newest: Option<i64>,
    pub consoles: Vec<(String, String)>,
    pub games: Vec<Game>,
}

/// The database and the library read and matched, as one read does it; what failed in words.
pub fn read(source: &ClientSource) -> Result<Found, String> {
    let words = |e: emusen_crystal_client::Error| e.to_string();
    let status = source.runtime.session_service(0, NO_GAME).and_then(|s| s.cheat_op(0, CHEAT_DB_STATUS, &json!({}))).map_err(words)?;
    let text = |k: &str| status[k].as_str().unwrap_or_default().to_owned();
    let mut found = Found { directory: text("directory"), exists: status["exists"].as_bool() == Some(true), attribution: text("attribution"), ..Found::default() };
    if !found.exists {
        return Ok(found);
    }
    let db = CheatsDatabase::open(&source.platform, &found.directory).map_err(|e| e.to_string())?;
    let entries = db.entries().map_err(|e| e.to_string())?;
    found.files = entries.len();
    let seconds = |p: &str| std::fs::metadata(p).and_then(|m| m.modified()).ok()?.duration_since(std::time::UNIX_EPOCH).ok().map(|d| d.as_secs() as i64);
    found.newest = entries.iter().filter_map(|e| seconds(&e.2)).max();
    if entries.is_empty() {
        return Ok(found);
    }
    let systems = source.systems()?;
    for game in source.games()? {
        let Some(system) = systems.iter().find(|s| Some(&s.id) == game.system.as_ref()).filter(|s| !s.cheat_systems.is_empty()) else { continue };
        let name = std::path::Path::new(&game.path).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        let places = db.query(&json!({ "op": "find", "text": name, "limit": entries.len() })).map_err(|e| e.to_string())?;
        let ours = |e: &&(String, String, String)| system.cheat_systems.iter().any(|c| c.eq_ignore_ascii_case(&e.0));
        let Some(entry) = places.as_array().into_iter().flatten().filter_map(Value::as_u64).filter_map(|at| entries.get(at as usize)).find(ours) else { continue };
        let list = List::of(&source.platform, &system.console, &game.path)?.map(|l| l.switches()).unwrap_or_default();
        if !found.consoles.iter().any(|c| c.0 == system.id) {
            found.consoles.push((system.id.clone(), crate::library::console_name(system)));
        }
        found.games.push(Game { path: game.path.clone(), name, console: system.id.clone(), file: (entry.0.clone(), entry.1.clone()), list });
    }
    Ok(found)
}

/// The tool: the clients it reads through, what the last read found, the read going, and what failed.
pub struct Cheats {
    source: Arc<ClientSource>,
    found: Option<Found>,
    reading: Option<Receiver<Result<Found, String>>>,
    problem: Option<String>,
}

impl Cheats {
    pub fn new(source: Arc<ClientSource>) -> Cheats {
        Cheats { source, found: None, reading: None, problem: None }
    }

    fn words() -> (&'static str, &'static str) {
        crate::home_card::tool_words(crate::home_card::CHEATS).unwrap_or_default()
    }

    fn game(&self, key: &str) -> Option<&Game> {
        self.found.as_ref()?.games.iter().find(|g| g.path == key)
    }

    /// The database's state in a line, as `cheat db status` says it.
    fn status(&self) -> Words {
        match &self.found {
            None if self.problem.is_some() => Words::new(format!("It could not be read: {}", self.problem.as_deref().unwrap_or_default()), Tone::Error),
            None => Words::new("Reading", Tone::Muted),
            Some(f) if !f.exists => Words::new(format!("No cheat database at {}", f.directory), Tone::Attention),
            Some(f) => Words::new(format!("{} cheat file(s) in {}", f.files, f.directory), Tone::Muted),
        }
    }
}

impl Source for Cheats {
    fn title(&self) -> &str {
        Cheats::words().0
    }

    fn section_word(&self) -> &str {
        "Console"
    }

    fn columns(&self) -> Vec<Column> {
        vec![
            Column { title: "Game", fill: 1.0, least: 190.0, trailing: false, gives_way: false },
            Column { title: "Cheats", fill: 0.0, least: 0.0, trailing: true, gives_way: false },
        ]
    }

    fn sections(&self) -> Vec<(String, String)> {
        let mut all = vec![("all".to_owned(), "All".to_owned())];
        all.extend(self.found.iter().flat_map(|f| f.consoles.iter().cloned()));
        all
    }

    fn items(&self) -> Vec<Item> {
        let Some(found) = &self.found else { return Vec::new() };
        found
            .games
            .iter()
            .map(|g| {
                let state = if g.list.is_empty() { Words::new("No list yet", Tone::Muted) } else { Words::text(count_words(&g.list)) };
                Item { key: g.path.clone(), cells: vec![Words::text(&g.name), state], sections: vec![g.console.clone()], search: g.name.clone(), ..Item::default() }
            })
            .collect()
    }

    /// Whether the database is there, how many files it holds and the date of its newest.
    fn facts(&self) -> Vec<Words> {
        let Some(found) = &self.found else { return Vec::new() };
        if !found.exists {
            return vec![Words::new("no database", Tone::Attention)];
        }
        let mut facts = vec![Words::new(format!("{} files", found.files), Tone::Muted)];
        if let Some(at) = found.newest {
            facts.push(Words::new(format!("newest file {}", super::cores::words::day(&crate::words::utc(at)[..10])), Tone::Muted));
        }
        facts
    }

    /// A game's file in the database and its own list; with none, what the tool does, the database's state and its
    /// attribution.
    fn pane(&self, key: Option<&str>) -> Pane {
        let mut sentences = vec![Words::text(Cheats::words().1)];
        let Some(game) = key.and_then(|k| self.game(k)) else {
            sentences.push(self.status());
            if let Some(f) = self.found.as_ref().filter(|f| f.exists) {
                sentences.extend(f.attribution.lines().map(|l| Words::new(l, Tone::Muted)));
            }
            return Pane { title: self.title().to_owned(), caption: String::new(), sentences };
        };
        sentences.push(Words::new(format!("{}: {}", game.file.0, game.file.1), Tone::Muted));
        if !game.list.is_empty() {
            sentences.push(Words::text(format!("Cheats: {}", count_words(&game.list))));
            sentences.push(Words::new(format!("X lists all {} cheats.", game.list.len()), Tone::Muted));
        }
        Pane { title: game.name.clone(), caption: String::new(), sentences }
    }

    fn bar(&self) -> Bar {
        Bar { line: self.status(), ..Bar::default() }
    }

    fn action(&self, _key: &str) -> Option<Action> {
        None
    }

    fn plans(&self) -> bool {
        false
    }

    /// The game's own cheats, each by its list's name and whether it is on.
    fn page(&self, key: &str, _marked: Option<&Mark>) -> Vec<CardRow> {
        let Some(game) = self.game(key) else { return Vec::new() };
        game.list
            .iter()
            .enumerate()
            .map(|(at, c)| CardRow::new(&format!("cheat{at}"), &c.description).value(on_words(c.on)).pane(CardPane::said(&c.description, "")))
            .collect()
    }

    fn act(&mut self, _action: &str, _key: Option<&str>, _section: &str) -> super::Effect {
        super::Effect::Done
    }

    fn answer(&mut self, _asked: &str, _yes: bool) {}

    /// The database and the library read anew off the GUI thread, as the screen opens.
    fn refresh(&mut self) {
        let (tx, rx) = channel();
        let source = self.source.clone();
        std::thread::spawn(move || drop(tx.send(read(&source))));
        self.reading = Some(rx);
    }

    fn poll(&mut self) -> bool {
        let Some(read) = self.reading.as_ref().and_then(|r| r.try_recv().ok()) else { return false };
        self.reading = None;
        match read {
            Ok(found) => {
                self.found = Some(found);
                self.problem = None;
            }
            Err(why) => self.problem = Some(why),
        }
        true
    }

    /// While the read goes, so the screen's timer polls for it.
    fn running(&self) -> bool {
        self.reading.is_some()
    }

    fn acted_on(&mut self) -> Option<String> {
        None
    }
}
