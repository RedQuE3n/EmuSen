//! The Pause Card's pages as plain data (EmuSen_Interface.md §2.8, §18.5; EmuSen_Qt.md §7.1 step 7): the rows and
//! their words, from the facts the running session gives, and what a choice or B does. No Qt here; `PauseModel` puts
//! it on Qt. The rows and words are Mistress's (`GameMenuRows`, `LeaveRows`), so the two front ends say the same.

use emusen_crystal_client::shapes::{RewindReel, SessionInfo, StateSlot};
use serde_json::{json, Value};

/// A page of the card: the first, and Leave's (its one row so far, EmuSen_Qt.md §7.1 step 7).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    First,
    Leave,
}

/// What the session says the first page's words are made from.
#[derive(Clone, Debug, PartialEq)]
pub struct Facts {
    /// The slot Save and Load act on, 1 to 8.
    pub slot: u32,
    /// Whether that slot holds a state.
    pub saved: bool,
    /// Why Rewind cannot open, where it cannot.
    pub rewind_why: Option<String>,
    /// How far back the reel reaches, in seconds.
    pub rewind_seconds: f64,
    /// The game's speed, in percent of its own.
    pub speed: u32,
}

impl Default for Facts {
    fn default() -> Self {
        Facts { slot: 1, saved: false, rewind_why: None, rewind_seconds: 0.0, speed: 100 }
    }
}

impl Facts {
    /// The facts as plain data, the way they cross from `SessionModel` to `PauseModel`.
    pub fn to_json(&self) -> String {
        json!({ "slot": self.slot, "saved": self.saved, "rewindWhy": self.rewind_why, "rewindSeconds": self.rewind_seconds, "speed": self.speed })
            .to_string()
    }

    /// The facts read back; what is missing keeps its default.
    pub fn from_json(text: &str) -> Facts {
        let v: Value = serde_json::from_str(text).unwrap_or(Value::Null);
        let d = Facts::default();
        Facts {
            slot: v["slot"].as_u64().map_or(d.slot, |n| n as u32),
            saved: v["saved"].as_bool().unwrap_or(d.saved),
            rewind_why: v["rewindWhy"].as_str().map(str::to_string),
            rewind_seconds: v["rewindSeconds"].as_f64().unwrap_or(d.rewind_seconds),
            speed: v["speed"].as_u64().map_or(d.speed, |n| n as u32),
        }
    }
}

/// The facts from the runtime's reads of a paused session, as Mistress reads them (`RewindUnavailable`, `RewindHeld`,
/// `SlotValue`); `reel` is none where it could not be read.
pub fn facts_of(info: &SessionInfo, reel: Option<&RewindReel>, slots: &[StateSlot], slot: u32, speed: u32) -> Facts {
    let rewind_why = if info.rewind_capture != Some(true) {
        Some(info.rewind_withheld.clone().unwrap_or_else(|| NOT_KEPT.to_string()))
    } else if reel.is_none_or(|r| r.snapshot_bytes == 0) {
        Some(NOTHING_YET.to_string())
    } else {
        None
    };
    let rewind_seconds = match (reel.map(|r| r.moments.as_slice()), info.frame_rate_hz) {
        (Some([first, .., last]), hz) if hz > 0.0 => (last.frame - first.frame) as f64 / hz,
        _ => 0.0,
    };
    let saved = slots.iter().any(|s| s.slot == slot && s.exists);
    Facts { slot, saved, rewind_why, rewind_seconds, speed }
}

const NOT_KEPT: &str = "not kept for this engine";
const NOTHING_YET: &str = "nothing to go back to yet";

/// One row: its key (what a choice of it does, `choose`), its words, and how it is drawn.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub key: &'static str,
    pub label: String,
    pub value: String,
    /// Left and Right step its value: drawn ◂ value ▸.
    pub steps: bool,
    /// A choice of it turns to a page of its own: drawn ▸.
    pub opens: bool,
    /// The last row of a section, its rule drawn lighter.
    pub section_end: bool,
}

fn row(key: &'static str, label: &str, value: String) -> Row {
    Row { key, label: label.to_string(), value, steps: false, opens: false, section_end: false }
}

pub const RESUME: &str = "resume";
pub const LEAVE: &str = "leave";
pub const LIBRARY: &str = "library";

/// The slot the Save and Load rows act on, and whether anything is in it (Mistress's `SlotValue`).
pub fn slot_value(slot: u32, saved: bool) -> String {
    if saved { format!("Slot {slot}") } else { format!("Slot {slot} (Empty)") }
}

/// The Rewind row's value: why it cannot open, else the whole seconds the reel holds, else nothing.
pub fn rewind_value(facts: &Facts) -> String {
    match &facts.rewind_why {
        Some(why) => why.clone(),
        None if facts.rewind_seconds >= 1.0 => format!("{} s", facts.rewind_seconds.floor()),
        None => String::new(),
    }
}

/// A page's rows. The first page holds Mistress's seven; its console rows (Change Disc, Swap Players) are owed.
pub fn rows(page: Page, facts: &Facts) -> Vec<Row> {
    match page {
        Page::First => {
            let slot = slot_value(facts.slot, facts.saved);
            vec![
                row(RESUME, "Resume", String::new()),
                row("rewind", "Rewind", rewind_value(facts)),
                Row { steps: true, ..row("save", "Save State", slot.clone()) },
                Row { steps: true, ..row("load", "Load State", slot) },
                Row { steps: true, section_end: true, ..row("speed", "Speed", format!("{}%", facts.speed)) },
                Row { opens: true, ..row("options", "Game Options", String::new()) },
                Row { opens: true, ..row(LEAVE, "Leave", String::new()) },
            ]
        }
        Page::Leave => vec![row(LIBRARY, "Back to Library", String::new())],
    }
}

/// A page's title over its rows: the game's on the first, the page's own name on the others.
pub fn title(page: Page, game: &str) -> String {
    match page {
        Page::First => game.to_string(),
        Page::Leave => "Leave".to_string(),
    }
}

/// The way back under a page's title: the game, on every page but the first.
pub fn breadcrumb(page: Page, game: &str) -> String {
    match page {
        Page::First => String::new(),
        Page::Leave => game.to_string(),
    }
}

/// What a choice does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Choice {
    /// The card closes and the game resumes once it is gone.
    Resume,
    /// The card turns to a page, focused on its first row.
    Turn(Page),
    /// Home, the game kept suspended behind it.
    Library,
    /// A row whose action is a later step's: nothing happens.
    Owed,
}

/// A row chosen by its key, or a declared action (`help::Screen::Pause`'s `resume`).
pub fn choose(key: &str) -> Choice {
    match key {
        RESUME => Choice::Resume,
        LEAVE => Choice::Turn(Page::Leave),
        LIBRARY => Choice::Library,
        _ => Choice::Owed,
    }
}

/// B on a page: back to the first page on the row that opened it, or, on the first page, the game resumed.
pub fn back(page: Page) -> Option<(Page, &'static str)> {
    match page {
        Page::First => None,
        Page::Leave => Some((Page::First, LEAVE)),
    }
}
