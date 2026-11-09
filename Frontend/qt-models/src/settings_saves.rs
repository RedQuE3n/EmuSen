//! The card manager on the settings frame (EmuSen_Qt.md §7.7.E): a medium's cards, the saves on the one chosen, and a
//! save copied or moved to another card, or deleted after a question, over the runtime's `cards_op`, in the words of
//! Mistress's Memory Cards window. Reached from Settings ▸ Consoles ▸ <console> ▸ Memory cards ▸ Saves on the cards.

use crate::cards::{Card, Medium, Save};
use crate::settings::{Outcome, Page, Row, SettingsView};
use emusen_crystal_client::consts::{CARDS_COPY, CARDS_DELETE, CARDS_IMPORT, CARDS_MOVE};
use emusen_kit_core::settings::RowKind;
use serde_json::{json, Value};

const CARD_HINT: &str = "A card in the drawer, or a game's own.";
const SAVE_HINT: &str = "Choose a save to copy, move or delete it.";
const IMPORT: &str = "Import Every Game's Saves";
const IMPORT_HINT: &str = "Every save on every game's own card copied onto the chosen card, all or none; each game's own card stays as it is.";
const TARGET: &str = "Copy or move to";
const COPY: &str = "Copy Save";
const MOVE: &str = "Move Save";
const DELETE: &str = "Delete Save…";

impl SettingsView {
    fn saves_medium(&self) -> Option<&Medium> {
        let medium = match self.page()? {
            Page::Saves(_, m) | Page::Save(_, m, _) | Page::Delete(_, m, _) => m,
            _ => return None,
        };
        self.console()?.media.iter().find(|x| &x.medium == medium)
    }

    /// The card the page shows: the one chosen, else the one the first slot holds, else the first.
    fn chosen(&self) -> Option<&Card> {
        let m = self.saves_medium()?;
        self.chosen_card.as_ref().and_then(|p| m.cards.iter().find(|c| &c.path == p)).or_else(|| m.first_card())
    }

    /// The card a save goes to: the one chosen, else the first other card.
    fn target(&self) -> Option<&Card> {
        let (m, chosen) = (self.saves_medium()?, self.chosen()?.path.clone());
        let others: Vec<&Card> = m.cards.iter().filter(|c| c.path != chosen).collect();
        self.target_card.as_ref().and_then(|p| others.iter().find(|c| &c.path == p).copied()).or(others.first().copied())
    }

    fn save(&self, id: &str) -> Option<&Save> {
        let (m, card) = (self.saves_medium()?, self.chosen()?);
        m.opened(&card.path)?.saves.iter().find(|s| s.id == id)
    }

    fn page_save(&self) -> Option<String> {
        match self.page()? {
            Page::Save(_, _, id) | Page::Delete(_, _, id) => Some(id.clone()),
            _ => None,
        }
    }

    /// The rows of the card manager's pages.
    pub(crate) fn saves_entries(&self) -> Vec<Value> {
        let (Some(m), Some(page)) = (self.saves_medium(), self.page()) else { return Vec::new() };
        match page {
            Page::Saves(..) => {
                let mut rows = vec![json!({ "id": "savescard", "savescard": true })];
                let saves = self.chosen().and_then(|c| m.opened(&c.path)).map(|o| o.saves.clone()).unwrap_or_default();
                rows.extend(saves.iter().map(|s| json!({ "id": format!("save:{}", s.id), "save": s.id })));
                rows.push(json!({ "id": "import", "import": true }));
                rows
            }
            Page::Save(..) => ["target", "copy", "move", "delete"].iter().map(|id| json!({ "id": id, "saveop": id })).collect(),
            Page::Delete(..) => vec![json!({ "id": "no", "answer": false }), json!({ "id": "yes", "answer": true })],
            _ => Vec::new(),
        }
    }

    /// Whether a row of the pages is held: an action the card, its holder or the medium does not allow.
    fn saves_held(&self, entry: &Value) -> bool {
        let (Some(m), Some(card)) = (self.saves_medium(), self.chosen()) else { return true };
        let held = card.held_by.is_some();
        match entry["saveop"].as_str().or(entry["import"].as_bool().map(|_| "import")) {
            Some("copy") => !m.readable() || self.target().is_none(),
            Some("move") => !m.readable() || self.target().is_none() || held,
            Some("delete") => !m.readable() || held,
            Some("import") => !m.readable() || held || card.games_own,
            _ => false,
        }
    }

    pub(crate) fn saves_row(&self, entry: &Value) -> Option<Row> {
        let m = self.saves_medium()?;
        let id = entry["id"].as_str().unwrap_or_default().to_owned();
        let row = |label: &str, value: String, kind: RowKind| Some(Row { id: id.clone(), label: label.to_owned(), value, kind, held: self.saves_held(entry), marked: false });
        if entry.get("savescard").is_some() {
            return row("Card", self.chosen().map(|c| c.label.clone()).unwrap_or_default(), RowKind::Steps);
        }
        if let Some(save) = entry["save"].as_str().and_then(|id| self.save(id)) {
            return row(&save.title, m.save_value(save), RowKind::Opens);
        }
        if entry.get("import").is_some() {
            return row(IMPORT, String::new(), RowKind::Action);
        }
        match entry["saveop"].as_str() {
            Some("target") => row(TARGET, self.target().map(|c| c.label.clone()).unwrap_or_default(), RowKind::Steps),
            Some("copy") => row(COPY, String::new(), RowKind::Action),
            Some("move") => row(MOVE, String::new(), RowKind::Action),
            Some("delete") => row(DELETE, String::new(), RowKind::Asks),
            _ => match entry["answer"].as_bool() {
                Some(yes) => row(if yes { "Yes" } else { "No" }, String::new(), RowKind::Action),
                None => None,
            },
        }
    }

    /// A row's pane: the card's words, a save's, what Import does, or the question; the last change's words under each.
    pub(crate) fn saves_pane(&self, entry: &Value) -> Option<crate::firmware::Words> {
        let (m, card) = (self.saves_medium()?, self.chosen()?);
        let mut muted: Vec<String> = Vec::new();
        let sentence = if entry.get("savescard").is_some() {
            m.about(card)
        } else if let Some(save) = entry["save"].as_str().and_then(|id| self.save(id)) {
            muted.extend(save.game.clone());
            SAVE_HINT.to_owned()
        } else if entry.get("import").is_some() {
            IMPORT_HINT.to_owned()
        } else if entry.get("answer").is_some() {
            let title = self.page_save().and_then(|id| self.save(&id)).map(|s| s.title.clone()).unwrap_or_default();
            format!("Delete {title} from {}? It cannot be brought back.", card.label)
        } else {
            let title = self.page_save().and_then(|id| self.save(&id)).map(|s| s.title.clone()).unwrap_or_default();
            format!("{title} on {}.", card.label)
        };
        if !self.said.is_empty() {
            muted.push(self.said.clone());
        }
        Some(crate::firmware::Words { lines: vec![sentence], muted, ..Default::default() })
    }

    /// Left or Right on the card or the target: the next card.
    pub(crate) fn saves_step(&mut self, entry: &Value, by: i32) -> Option<Outcome> {
        let m = self.saves_medium()?.clone();
        let step = |list: Vec<&Card>, now: Option<&Card>| -> Option<String> {
            if list.is_empty() {
                return None;
            }
            let at = now.and_then(|n| list.iter().position(|c| c.path == n.path)).unwrap_or(0) as i32;
            Some(list[(at + by).rem_euclid(list.len() as i32) as usize].path.clone())
        };
        if entry.get("savescard").is_some() {
            let next = step(m.cards.iter().collect(), self.chosen());
            self.chosen_card = next;
            self.target_card = None;
            return Some(Outcome::Shown);
        }
        if entry["saveop"] == "target" {
            let chosen = self.chosen()?.path.clone();
            let next = step(m.cards.iter().filter(|c| c.path != chosen).collect(), self.target());
            self.target_card = next;
            return Some(Outcome::Shown);
        }
        Some(Outcome::Nothing)
    }

    /// A on a row of the pages: a save's page, a copy, a move, the question and its answer, an import.
    pub(crate) fn saves_choose(&mut self, entry: &Value, index: usize) -> Option<Outcome> {
        let page = self.page()?.clone();
        let (console, medium) = match &page {
            Page::Saves(c, m) | Page::Save(c, m, _) | Page::Delete(c, m, _) => (c.clone(), m.clone()),
            _ => return None,
        };
        if entry.get("savescard").is_some() || entry["saveop"] == "target" {
            return self.saves_step(entry, 1);
        }
        if self.saves_held(entry) {
            return Some(Outcome::Nothing);
        }
        let card = self.chosen()?.path.clone();
        if let Some(id) = entry["save"].as_str() {
            let title = self.save(id).map(|s| s.title.clone()).unwrap_or_default();
            return Some(self.open(Page::Save(console, medium, id.to_owned()), title, index));
        }
        if entry.get("import").is_some() {
            let from: Vec<String> = self.saves_medium()?.cards.iter().filter(|c| c.games_own).map(|c| c.path.clone()).collect();
            return Some(Outcome::Cards { op: CARDS_IMPORT, args: json!({ "to": card, "from": from }) });
        }
        let save = self.page_save()?;
        let target = self.target().map(|c| c.path.clone());
        match (entry["saveop"].as_str(), entry["answer"].as_bool()) {
            (Some("copy"), _) => Some(Outcome::Cards { op: CARDS_COPY, args: json!({ "from": card, "save": save, "to": target? }) }),
            (Some("move"), _) => Some(Outcome::Cards { op: CARDS_MOVE, args: json!({ "from": card, "save": save, "to": target? }) }),
            (Some("delete"), _) => Some(self.open(Page::Delete(console, medium, save), "Delete Save".to_owned(), index)),
            (_, Some(false)) => {
                self.pages.pop();
                Some(Outcome::Opened)
            }
            (_, Some(true)) => Some(Outcome::Cards { op: CARDS_DELETE, args: json!({ "card": card, "save": save }) }),
            _ => None,
        }
    }

    /// The card manager's answer to a change: its words shown on the page, and after a save moved or deleted, its pages
    /// closed back to the card's, since the save is no longer there.
    pub fn cards_said(&mut self, op: u32, done: bool, words: String) {
        self.said = words;
        if done && (op == CARDS_MOVE || op == CARDS_DELETE) {
            while matches!(self.page(), Some(Page::Save(..) | Page::Delete(..))) {
                self.pages.pop();
            }
        }
    }
}
