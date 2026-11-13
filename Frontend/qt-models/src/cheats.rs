//! A game's own cheat list: the file Galaxia names for it (`cheat_game_list`, a CheatFile of the `cheats` category), its
//! switches read, turned and saved, and the list given to a running session through the session service's cheat ops
//! (emusen_session.h, EMUSEN_SESSION_CHEAT_*), so a cheat switched on is in force from the game's first frame.

use emusen_crystal_client::SessionService;
use emusen_crystal_client::{consts, Runtime};
use emusen_platform_client::galaxia::consts::MODEL_CHEAT_FILE;
use emusen_platform_client::Platform;
use serde_json::{json, Value};

/// One cheat as the list shows it: its words and whether it is on.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Switch {
    pub description: String,
    pub on: bool,
}

/// The value a switch's row shows, as a settings switch says it (`schema::shown`).
pub fn on_words(on: bool) -> &'static str {
    if on { "On" } else { "Off" }
}

/// A game's list: where Galaxia keeps it, and its file as the model reads it (none saved yet reads as no cheats).
#[derive(Clone, Debug, PartialEq)]
pub struct List {
    pub name: String,
    pub category: String,
    pub file: Value,
}

impl List {
    /// The list of the game at `path` on `console`; none for a path no list may be named after.
    pub fn of(platform: &Platform, console: &str, path: &str) -> Result<Option<List>, String> {
        let Some((name, category)) = platform.cheat_game_list(Some(console), path).map_err(|e| e.to_string())? else { return Ok(None) };
        let file = platform.model_load(MODEL_CHEAT_FILE, Some(&category), &format!("{name}.json")).map_err(|e| e.to_string())?;
        Ok(Some(List { name, category, file: file.filter(Value::is_object).unwrap_or_else(|| json!({ "Cheats": [] })) }))
    }

    /// The file's cheats, in its order.
    pub fn entries(&self) -> &[Value] {
        self.file["Cheats"].as_array().map_or(&[], Vec::as_slice)
    }

    pub fn switches(&self) -> Vec<Switch> {
        self.entries()
            .iter()
            .map(|c| Switch { description: c["Description"].as_str().unwrap_or_default().to_owned(), on: c["Enabled"].as_bool() == Some(true) })
            .collect()
    }

    /// The cheat at `index` switched; false where there is none.
    pub fn set(&mut self, index: usize, on: bool) -> bool {
        match self.file["Cheats"].as_array_mut().and_then(|a| a.get_mut(index)).and_then(Value::as_object_mut) {
            Some(entry) => {
                entry.insert("Enabled".into(), Value::Bool(on));
                true
            }
            None => false,
        }
    }

    pub fn save(&self, platform: &Platform) -> Result<(), String> {
        platform.model_save(MODEL_CHEAT_FILE, Some(&self.category), &format!("{}.json", self.name), &self.file).map_err(|e| e.to_string())
    }
}

/// A list given to a running session: each of its cheats by the session's id, none for one the session skipped.
#[derive(Clone, Debug, PartialEq)]
pub struct Given {
    pub list: List,
    pub ids: Vec<Option<i32>>,
}

/// The list of the game `session` runs given to it, each cheat imported alone so the session's id for each is known;
/// none where the game has no list or no cheats.
pub fn give(platform: &Platform, runtime: &Runtime, session: u32, path: &str) -> Result<Option<Given>, String> {
    let words = |e: emusen_crystal_client::Error| e.to_string();
    let console = runtime.session_info(session).map_err(words)?.console;
    let Some(list) = List::of(platform, &console, path)? else { return Ok(None) };
    if list.entries().is_empty() {
        return Ok(None);
    }
    let service = runtime.session_service(session, 0).map_err(words)?;
    let before = listed(&service, session)?.len();
    let mut loaded = Vec::with_capacity(list.entries().len());
    for entry in list.entries() {
        let answer = service.cheat_op(session, consts::CHEAT_IMPORT, &json!({ "Cheats": [entry] })).map_err(words)?;
        loaded.push(answer.as_i64() == Some(1));
    }
    let mut ids = listed(&service, session)?.into_iter().skip(before);
    let ids = loaded.into_iter().map(|ok| if ok { ids.next() } else { None }).collect();
    Ok(Some(Given { list, ids }))
}

/// The session's cheats' ids, in the order it holds them.
fn listed(service: &SessionService, session: u32) -> Result<Vec<i32>, String> {
    let list = service.cheat_op(session, consts::CHEAT_LIST, &json!({})).map_err(|e| e.to_string())?;
    Ok(list["cheats"].as_array().into_iter().flatten().filter_map(|c| c["id"].as_i64()).map(|id| id as i32).collect())
}

impl Given {
    /// The cheat at `index` switched in the session at once, then in its file, which is saved.
    pub fn switch(&mut self, platform: &Platform, runtime: &Runtime, session: u32, index: usize, on: bool) -> Result<(), String> {
        let words = |e: emusen_crystal_client::Error| e.to_string();
        if let Some(Some(id)) = self.ids.get(index) {
            let op = if on { consts::CHEAT_ENABLE } else { consts::CHEAT_DISABLE };
            runtime.session_service(session, 0).and_then(|s| s.cheat_op(session, op, &json!({ "id": id }))).map_err(words)?;
        }
        if !self.list.set(index, on) {
            return Err(format!("no cheat {index}"));
        }
        self.list.save(platform)
    }
}
