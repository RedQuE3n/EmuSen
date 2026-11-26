//! Cheats on the tool screen: whether the player's cheat database is installed, how many files it holds and the date of
//! its newest, and the library's games it has a file for, by console, each with the state of the game's own list (the
//! list the Pause Card and the game page switch, cheats.rs). The database is read through the base's own calls: its
//! folder and attribution from the session table's DB_STATUS, its files from `emusen_cheats_database_*`, and a game's
//! file by the database's own `find`, kept to the console's cheat systems. The library is matched on a thread of its own.
//! The List card gets or updates the database: downloaded by the base's HTTP client (`emusen_net_download_*`), its
//! progress on the bar and a stop on the List card, then installed by `emusen_cheats_install`, which leaves the database
//! as it was when the download is stopped or fails. A on a game with no list of its own makes it from the game's file,
//! read with its console's codecs by `emusen_cheats_parse` (EmuSen_Qt.md §7.7.V). No Qt here.

use super::cores::words::size;
use super::{Action, Bar, Column, Item, Mark, Pane, Source, Tone, Words};
use crate::card::{Pane as CardPane, Row as CardRow};
use crate::cheats::{count_words, on_words, List, Switch};
use crate::clients::ClientSource;
use crate::library::LibrarySource;
use emusen_crystal_client::consts::CHEAT_DB_STATUS;
use emusen_crystal_client::service_flags::NO_GAME;
use emusen_platform_client::cheats::CheatsDatabase;
use emusen_platform_client::net::{Download, State};
use serde_json::{json, Value};
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::sync::Arc;

/// The List card's rows: the database got or updated, and the download stopped; A's action on a game with no list.
pub const GET: &str = "get";
pub const STOP: &str = "stop";
pub const MAKE: &str = "make";

/// A library game the database has a file for: its path, name, system, the file's system folder, game and path, the
/// console its own list is kept under, and that list, none where it has none yet.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Game {
    pub path: String,
    pub name: String,
    pub console: String,
    pub file: (String, String),
    pub cht: String,
    pub list_console: String,
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
        found.games.push(Game {
            path: game.path.clone(),
            name,
            console: system.id.clone(),
            file: (entry.0.clone(), entry.1.clone()),
            cht: entry.2.clone(),
            list_console: system.console.clone(),
            list,
        });
    }
    Ok(found)
}

/// A download waited on and, once done, installed into `directory`, on the thread this is called from: the words of
/// what was installed, or of why nothing was, each said where it happened. A download stopped installs nothing.
fn fetch_and_install(source: &ClientSource, download: &Download, directory: &str) -> Words {
    let error = |words: String| Words::new(words, Tone::Error);
    let state = loop {
        // The thread's own check period, a constant of the design: a stop ends the wait within it.
        match download.wait(100) {
            State::Running => continue,
            other => break other,
        }
    };
    if let Some(words) = ended(state, download.failure()) {
        return words;
    }
    let zip = match archive(download.body()) {
        Ok(zip) => zip,
        Err(words) => return words,
    };
    match source.platform.cheats_install(&zip, directory) {
        Ok(answer) => match answer["installed"].as_u64() {
            Some(n) => Words::new(format!("Installed {n} cheat file(s)"), Tone::Success),
            None => error(format!("It was installed, but the answer could not be read: {answer}")),
        },
        Err(e) => error(format!("It could not be installed: {e}")),
    }
}

/// How a download that has ended is said, from its state and its failure record: none where it is done and its archive
/// is to be read, else the words of why nothing is installed.
pub fn ended(state: State, failure: Result<Option<Value>, impl std::fmt::Display>) -> Option<Words> {
    let error = |words: String| Some(Words::new(words, Tone::Error));
    match (state, failure) {
        (State::Done, _) => None,
        (State::Cancelled, _) => Some(Words::new("Stopped", Tone::Muted)),
        (_, Ok(Some(why))) => match why["message"].as_str() {
            Some(message) => error(format!("It could not be downloaded: {message}")),
            None => error(format!("It could not be downloaded, and why could not be read: {why}")),
        },
        (_, Ok(None)) => error("It could not be downloaded, and no reason was given".to_owned()),
        (_, Err(e)) => error(format!("It could not be downloaded, and why could not be read: {e}")),
    }
}

/// A done download's archive as its body read gives it, or the words of why there is none to install.
pub fn archive(body: Result<Option<Vec<u8>>, impl std::fmt::Display>) -> Result<Vec<u8>, Words> {
    match body {
        Ok(Some(zip)) => Ok(zip),
        Ok(None) => Err(Words::new("It was downloaded, but there was no archive to install".to_owned(), Tone::Error)),
        Err(e) => Err(Words::new(format!("The downloaded archive could not be read: {e}"), Tone::Error)),
    }
}

/// What the bar says once a list is made: nothing where every cheat of the file is in it, else how many were left out.
pub fn made_words(left_out: u64) -> Option<Words> {
    (left_out > 0).then(|| Words::new(format!("Its list was made; {left_out} cheat(s) of the file could not be held in it"), Tone::Attention))
}

/// The tool: the clients it reads through, the address it downloads from, what the last read found, the read going,
/// the download going with what its thread tells, how the last download ended, and what failed.
pub struct Cheats {
    pub(super) source: Arc<ClientSource>,
    url: String,
    found: Option<Found>,
    reading: Option<Receiver<Result<(Found, Option<super::cheats_eclipse::EclipseRead>), String>>>,
    pub(super) download: Option<(Arc<Download>, Receiver<Words>)>,
    outcome: Option<Words>,
    problem: Option<String>,
    acted: Option<String>,
    eclipse: bool,
    eclipse_state: super::cheats_eclipse::State,
}

impl Cheats {
    /// The tool over `source`, its database downloaded from `url` where one is named, else the header's address.
    pub fn new(source: Arc<ClientSource>, url: Option<String>) -> Cheats {
        let url = url.unwrap_or_else(|| emusen_platform_client::cheats::database_url().to_owned());
        Cheats { source, url, found: None, reading: None, download: None, outcome: None, problem: None, acted: None, eclipse: false, eclipse_state: super::cheats_eclipse::State::default() }
    }

    /// The download started off the GUI thread, into the folder the last read found; installed when it ends.
    pub fn get(&mut self) {
        let Some(directory) = self.found.as_ref().map(|f| f.directory.clone()).filter(|_| !self.running()) else { return };
        let download = match Download::start(&self.source.platform, &json!({ "url": self.url, "timeout_ms": null })) {
            Ok(d) => Arc::new(d),
            Err(e) => {
                self.outcome = Some(Words::new(format!("It could not be downloaded: {e}"), Tone::Error));
                return;
            }
        };
        let (tx, rx) = channel();
        let (source, held) = (self.source.clone(), download.clone());
        std::thread::spawn(move || drop(tx.send(fetch_and_install(&source, &held, &directory))));
        self.download = Some((download, rx));
        self.outcome = None;
    }

    /// The download stopped; the database stays as it was.
    pub fn stop(&mut self) {
        if let Some((download, _)) = &self.download {
            download.cancel();
        }
    }

    /// The game's own list made from its file in the database, read with its console's codecs, and saved.
    pub fn make(&mut self, key: &str) {
        let platform = self.source.platform.clone();
        let Some(game) = self.found.as_mut().and_then(|f| f.games.iter_mut().find(|g| g.path == key)) else { return };
        let made = (|| -> Result<(Vec<Switch>, u64), String> {
            let parsed = platform.cheats_parse(&json!({ "path": game.cht, "system": game.console })).map_err(|e| e.to_string())?;
            let left_out = parsed["left_out"].as_u64().ok_or_else(|| format!("the answer could not be read: {parsed}"))?;
            let mut list = List::of(&platform, &game.list_console, &game.path)?.ok_or_else(|| format!("{} can have no list", game.name))?;
            list.file = parsed["file"].clone();
            list.save(&platform)?;
            Ok((list.switches(), left_out))
        })();
        match made {
            Ok((list, left_out)) => {
                game.list = list;
                self.outcome = made_words(left_out);
            }
            Err(why) => self.outcome = Some(Words::new(format!("Its list could not be made: {why}"), Tone::Error)),
        }
        self.acted = Some(key.to_owned());
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

    /// The download's progress while it goes, then how it ended, else the database's state.
    fn bar(&self) -> Bar {
        if let Some((download, _)) = &self.download {
            let (_, received, total) = download.progress();
            let line = match total {
                Some(total) => format!("Downloading: {} of {}", size(received), size(total)),
                None => format!("Downloading: {}", size(received)),
            };
            let share = total.filter(|t| *t > 0).map_or(0.0, |t| received as f64 / t as f64);
            return Bar { line: Words::text(line), progress: Some(share), ..Bar::default() };
        }
        Bar { line: self.outcome.clone().unwrap_or_else(|| self.status()), ..Bar::default() }
    }

    /// A on a game with no list of its own makes it from the database's file.
    fn action(&self, key: &str) -> Option<Action> {
        self.game(key).filter(|g| g.list.is_empty()).map(|_| Action::new(MAKE, "Make its list"))
    }

    /// Get the database with none, Update it with one; Stop the download while it goes.
    fn list_rows(&self) -> Vec<CardRow> {
        match (&self.download, &self.found) {
            (Some(_), _) => vec![CardRow::new(STOP, "Stop the download")],
            (None, Some(f)) if self.reading.is_none() => vec![CardRow::new(GET, if f.exists { "Update the database" } else { "Get the database" })],
            _ => Vec::new(),
        }
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

    fn act(&mut self, action: &str, key: Option<&str>, _section: &str) -> super::Effect {
        if self.eclipse && matches!(action, super::cheats_eclipse::ACT_EDIT_ROW | super::cheats_eclipse::ACT_REMOVE_ROW) {
            let platform = self.source.platform.clone();
            return self.eclipse_state.eclipse_act(&platform, action, key, self.download.is_some()).0;
        }
        match (action, key) {
            (GET, _) => self.get(),
            (STOP, _) => self.stop(),
            (MAKE, Some(key)) if !self.running() => self.make(key),
            _ => {}
        }
        super::Effect::Done
    }

    fn answer(&mut self, asked: &str, yes: bool) {
        if self.eclipse {
            let platform = self.source.platform.clone();
            self.eclipse_state.answer(asked, yes, &platform);
        }
    }

    /// The database and the library read anew off the GUI thread, as the screen opens.
    fn refresh(&mut self) {
        let (tx, rx) = channel();
        let source = self.source.clone();
        let want_eclipse = self.eclipse;
        std::thread::spawn(move || {
            let res = read(&source).and_then(|found| {
                let ec = if want_eclipse { Some(super::cheats_eclipse::read_eclipse(&source, &found)?) } else { None };
                Ok((found, ec))
            });
            drop(tx.send(res));
        });
        self.reading = Some(rx);
    }

    fn poll(&mut self) -> bool {
        let told = self.download.as_ref().map(|(_, told)| told.try_recv());
        if let Some(ended @ (Ok(_) | Err(TryRecvError::Disconnected))) = told {
            self.download = None;
            let words = ended.unwrap_or_else(|_| Words::new("The download's thread ended without a word", Tone::Error));
            if self.eclipse {
                self.eclipse_state.status_note = words.words.clone();
            }
            self.outcome = Some(words);
            self.refresh();
            return true;
        }
        if self.download.is_some() {
            return true;
        }
        let Some(read) = self.reading.as_ref().and_then(|r| r.try_recv().ok()) else { return false };
        self.reading = None;
        match read {
            Ok((found, ec)) => {
                if let Some(eclipse_read) = ec {
                    self.eclipse_state.take_read(eclipse_read);
                } else if self.eclipse {
                    if let Ok(eclipse_read) = super::cheats_eclipse::read_eclipse(&self.source, &found) {
                        self.eclipse_state.take_read(eclipse_read);
                    }
                }
                self.found = Some(found);
                self.problem = None;
            }
            Err(why) => self.problem = Some(why),
        }
        true
    }

    /// While the read or the download goes, so the screen's timer polls for it.
    fn running(&self) -> bool {
        self.reading.is_some() || self.download.is_some()
    }

    fn acted_on(&mut self) -> Option<String> {
        self.acted.take()
    }

    fn set_eclipse(&mut self, eclipse: bool) {
        self.eclipse = eclipse;
        if eclipse && self.eclipse_state.read.is_none() {
            if let Some(found) = &self.found {
                if let Ok(eclipse_read) = super::cheats_eclipse::read_eclipse(&self.source, found) {
                    self.eclipse_state.take_read(eclipse_read);
                }
            }
        }
    }

    fn eclipse_screen_code(&self) -> i32 {
        8
    }

    fn eclipse_view(&self) -> Option<Value> {
        self.eclipse.then(|| self.eclipse_state.projection(self.download.is_some()))
    }

    fn eclipse_choose_filter(&mut self, key: &str) {
        self.eclipse_state.choose_filter(key);
    }

    fn eclipse_focus_core(&mut self, key: &str) {
        self.eclipse_state.focus_cheat_key(key);
    }

    fn eclipse_focus_zone_code(&mut self, code: i32) {
        self.eclipse_state.zone = super::cheats_eclipse::Zone::from_code_pub(code);
    }

    fn eclipse_act(&mut self, action: &str, key: Option<&str>) -> super::Effect {
        let platform = self.source.platform.clone();
        let (eff, db_toggle) = self.eclipse_state.eclipse_act(&platform, action, key, self.download.is_some());
        if db_toggle {
            if self.download.is_some() {
                self.stop();
            } else {
                self.get();
            }
        }
        eff
    }

    fn eclipse_action_key(&mut self, key: &str) -> super::Effect {
        self.eclipse_act(key, None)
    }

    fn eclipse_act_code(&mut self, code: i32, key: Option<&str>) -> super::Effect {
        let action = match code {
            0 => super::cheats_eclipse::ACT_TOGGLE,
            1 => super::cheats_eclipse::ACT_MASTER,
            2 => super::cheats_eclipse::ACT_MORE,
            3 => super::cheats_eclipse::ACT_BROWSE,
            4 => super::cheats_eclipse::ACT_ADD,
            5 => super::cheats_eclipse::ACT_UPDATE_DB,
            6 => "chooseFilter",
            7 => "focusField",
            8 => "saveEditor",
            9 => "cancelEditor",
            10 => "closeOverlay",
            11 => "stopTyping",
            12 => "focusField0",
            13 => "focusField1",
            _ => return super::Effect::Done,
        };
        self.eclipse_act(action, key)
    }

    fn eclipse_navigate(&mut self, dir: &str) -> super::Effect {
        self.eclipse_state.navigate(dir)
    }

    fn eclipse_nav(&mut self, code: i32) -> super::Effect {
        match code {
            0 => self.eclipse_state.navigate("up"),
            1 => self.eclipse_state.navigate("down"),
            2 => self.eclipse_state.navigate("left"),
            3 => self.eclipse_state.navigate("right"),
            4 => self.eclipse_act("activate", None),
            5 => self.eclipse_act("prevFilter", None),
            6 => self.eclipse_act("nextFilter", None),
            7 => self.eclipse_act("prevGame", None),
            8 => self.eclipse_act("nextGame", None),
            _ => super::Effect::Done,
        }
    }

    fn eclipse_set_cheats_target(&mut self, game: &str, session_game: &str, master_on: bool) {
        self.eclipse_state.set_target(game, session_game, master_on);
    }

    fn eclipse_set_field(&mut self, field: i32, text: &str) {
        self.eclipse_state.set_field(field, text);
    }

    fn take_cheats_changed(&mut self) -> Option<String> {
        self.eclipse_state.cheats_dirty_for.take()
    }

    fn take_master_changed(&mut self) -> Option<bool> {
        self.eclipse_state.master_dirty.take()
    }
}
