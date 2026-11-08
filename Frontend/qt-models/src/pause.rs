//! The Pause Card's pages as plain data (EmuSen_Interface.md §2.8, §18.5, §20; EmuSen_Qt.md §7.1 step 7, §7.2): the
//! card's state (`Card`), its rows and words from the facts the running session gives, its context pane, and what a
//! choice, a step, B or a shoulder does. No Qt here; `PauseModel` puts it on Qt. The rows and words are Mistress's
//! (`GameMenuRows`, `LeaveRows`, `ReelRows`, `PauseCardPane`), so the two front ends say the same.

use crate::card;
use crate::help::Screen;
use crate::reel::{self, Reel};
use emusen_crystal_client::shapes::{RewindReel, SessionInfo};
use serde_json::{json, Value};

/// A page of the card.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    First,
    Leave,
    /// Rewind's reel (§20.1).
    Reel,
    /// One of Leave's questions, on the card form's question page (`card::question`).
    Ask(Ask),
}

/// Leave's questions, each asked before an end (Mistress's `LeaveRows`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ask {
    Restart,
    QuitGame,
    QuitEmuSen,
}

impl Ask {
    /// The row on Leave that asks it.
    fn key(self) -> &'static str {
        match self {
            Ask::Restart => RESTART,
            Ask::QuitGame => QUIT_GAME,
            Ask::QuitEmuSen => QUIT_EMUSEN,
        }
    }

    /// The question as the card form's question page: its title, its sentence, No then Yes.
    fn page(self) -> card::Page {
        match self {
            Ask::Restart => card::question("Restart Game?", "The game starts again from the beginning. Progress since your last save is lost."),
            Ask::QuitGame => card::question("Quit Game?", "The game closes and the library comes back."),
            Ask::QuitEmuSen => card::question("Quit EmuSen?", "The game closes, then EmuSen."),
        }
    }
}

/// The slots Save and Load step through.
pub const SLOTS: u32 = 8;

/// The speeds the Speed row steps through, in percent of the game's own; 0 is unthrottled (Mistress's `StepSpeed`).
pub const SPEEDS: [u32; 3] = [25, 100, 300];

/// One state slot as the pane shows it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SlotFact {
    pub slot: u32,
    pub exists: bool,
    /// When it was saved, worded in the player's time ("Sun 8 Nov, 21:42"); empty where nothing is saved.
    pub when: String,
    /// Its picture's address, empty for none.
    pub picture: String,
}

/// What the session says the card's words are made from.
#[derive(Clone, Debug, PartialEq)]
pub struct Facts {
    /// The slot Save and Load act on, 1 to 8.
    pub slot: u32,
    /// Slots 1 to 8, as far as they were read.
    pub slots: Vec<SlotFact>,
    /// Why Rewind cannot open, where it cannot.
    pub rewind_why: Option<String>,
    /// How far back the reel reaches, in seconds.
    pub rewind_seconds: f64,
    /// The frames of the reel's moments that hold a picture, oldest first.
    pub held: Vec<u64>,
    /// The session's frame count: the present.
    pub now: u64,
    /// The game's frames a second.
    pub hz: f64,
    /// The game's speed, in percent of its own; 0 unthrottled.
    pub speed: u32,
    /// The address the session's pictures are served under, ending in a slash ("image://session/3/").
    pub pictures: String,
}

impl Default for Facts {
    fn default() -> Self {
        Facts {
            slot: 1,
            slots: Vec::new(),
            rewind_why: None,
            rewind_seconds: 0.0,
            held: Vec::new(),
            now: 0,
            hz: 60.0,
            speed: 100,
            pictures: String::new(),
        }
    }
}

impl Facts {
    /// The facts as plain data, the way they cross from `SessionModel` to `PauseModel`.
    pub fn to_json(&self) -> String {
        let slots: Vec<Value> =
            self.slots.iter().map(|s| json!({ "slot": s.slot, "exists": s.exists, "when": s.when, "picture": s.picture })).collect();
        json!({ "slot": self.slot, "slots": slots, "rewindWhy": self.rewind_why, "rewindSeconds": self.rewind_seconds,
                "held": self.held, "now": self.now, "hz": self.hz, "speed": self.speed, "pictures": self.pictures })
        .to_string()
    }

    /// The facts read back; what is missing keeps its default.
    pub fn from_json(text: &str) -> Facts {
        let v: Value = serde_json::from_str(text).unwrap_or(Value::Null);
        let d = Facts::default();
        let words = |v: &Value| v.as_str().unwrap_or_default().to_string();
        let slots = v["slots"].as_array().map_or_else(Vec::new, |a| {
            a.iter()
                .map(|s| SlotFact {
                    slot: s["slot"].as_u64().unwrap_or(0) as u32,
                    exists: s["exists"].as_bool().unwrap_or(false),
                    when: words(&s["when"]),
                    picture: words(&s["picture"]),
                })
                .collect()
        });
        Facts {
            slot: v["slot"].as_u64().map_or(d.slot, |n| n as u32),
            slots,
            rewind_why: v["rewindWhy"].as_str().map(str::to_string),
            rewind_seconds: v["rewindSeconds"].as_f64().unwrap_or(d.rewind_seconds),
            held: v["held"].as_array().map_or_else(Vec::new, |a| a.iter().filter_map(Value::as_u64).collect()),
            now: v["now"].as_u64().unwrap_or(d.now),
            hz: v["hz"].as_f64().unwrap_or(d.hz),
            speed: v["speed"].as_u64().map_or(d.speed, |n| n as u32),
            pictures: words(&v["pictures"]),
        }
    }

    /// Whether the slot Save and Load act on holds a state.
    pub fn saved(&self) -> bool {
        self.slot_fact(self.slot).is_some_and(|s| s.exists)
    }

    fn slot_fact(&self, slot: u32) -> Option<&SlotFact> {
        self.slots.iter().find(|s| s.slot == slot)
    }

    /// The reel's picture of the moment at `frame`, or of the present.
    pub fn moment_picture(&self, mark: &reel::Mark) -> String {
        if self.pictures.is_empty() {
            String::new()
        } else if mark.now {
            format!("{}now", self.pictures)
        } else {
            format!("{}reel/{}", self.pictures, mark.frame)
        }
    }
}

/// The rewind facts from the runtime's reads of a paused session, as Mistress reads them (`RewindUnavailable`,
/// `RewindHeld`, `ReelOf`); `reel` is none where it could not be read. The slots, the speed and the pictures' address
/// are the session model's to fill.
pub fn facts_of(info: &SessionInfo, reel: Option<&RewindReel>) -> Facts {
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
    let held = reel.map_or_else(Vec::new, |r| r.moments.iter().filter(|m| m.picture.is_some()).map(|m| m.frame).collect());
    let hz = if info.frame_rate_hz > 0.0 { info.frame_rate_hz } else { 60.0 };
    Facts { rewind_why, rewind_seconds, held, now: info.total_frames, hz, ..Facts::default() }
}

/// What a slot with nothing in it, or a picture not to be had, says.
pub const EMPTY: &str = "Empty";
const NOT_KEPT: &str = "not kept for this engine";
const NOTHING_YET: &str = "nothing to go back to yet";

/// One row: its key (what a choice of it does), its words, and how it is drawn.
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
    /// Greyed: a hint, chosen to no effect.
    pub dimmed: bool,
}

fn row(key: &'static str, label: &str, value: String) -> Row {
    Row { key, label: label.to_string(), value, steps: false, opens: false, section_end: false, dimmed: false }
}

pub const RESUME: &str = "resume";
pub const REWIND: &str = "rewind";
pub const SAVE: &str = "save";
pub const LOAD: &str = "load";
pub const SPEED: &str = "speed";
pub const LEAVE: &str = "leave";
pub const LIBRARY: &str = "library";
pub const RESTART: &str = "restart";
pub const QUIT_GAME: &str = "quitGame";
pub const QUIT_EMUSEN: &str = "quitEmuSen";
pub const GO_BACK: &str = "goBack";
pub const MOMENT: &str = "moment";
pub const HOLD: &str = "hold";

/// The slot Save and Load act on, and whether anything is in it (Mistress's `SlotValue`).
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

/// The Speed row's value (Mistress's `DescribeSpeed`).
pub fn speed_value(percent: u32) -> String {
    if percent == 0 { "unthrottled".to_string() } else { format!("{percent}%") }
}

/// What the card asks of the session or the window after a press.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Nothing changed.
    Nothing,
    /// The page, its rows or its pane changed.
    Shown,
    /// The card closes and the game resumes once it is gone.
    Resume,
    /// Home, the game kept suspended behind it.
    Library,
    /// The slot Save and Load act on, chosen.
    Slot(u32),
    /// The game's speed, chosen.
    Speed(u32),
    /// The slot saved, and the card closed.
    Save(u32),
    /// The slot loaded, and the card closed.
    Load(u32),
    /// The game returned to the moment at that frame, and the card closed.
    RewindTo(u64),
    /// The game started again from the beginning, and the card closed.
    Restart,
    /// The game closed and the library shown.
    QuitGame,
    /// The game closed, then the program.
    QuitEmuSen,
}

/// The context pane's kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaneKind {
    None,
    /// The reel's newest pictures, on the Rewind row.
    Reel,
    /// The focused moment and the strip, on the reel's page.
    Moment,
    /// The slots, on Save and Load.
    Slots,
}

impl PaneKind {
    pub fn name(self) -> &'static str {
        match self {
            PaneKind::None => "none",
            PaneKind::Reel => "reel",
            PaneKind::Moment => "moment",
            PaneKind::Slots => "slots",
        }
    }
}

/// A picture of the pane with its words under it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tile {
    /// Its picture's address; empty for none, drawn as the empty slot.
    pub picture: String,
    pub name: String,
    pub when: String,
    pub ringed: bool,
}

/// The context pane: a large picture where it has one, a row of tiles, a sentence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pane {
    pub kind: PaneKind,
    pub big: String,
    pub tiles: Vec<Tile>,
    pub sentence: String,
}

impl Pane {
    fn none() -> Pane {
        Pane { kind: PaneKind::None, big: String::new(), tiles: Vec::new(), sentence: String::new() }
    }

    /// The pane as plain data, the way it crosses from `PauseModel` to `PaneModel`.
    pub fn to_json(&self) -> String {
        let tiles: Vec<Value> = self
            .tiles
            .iter()
            .map(|t| json!({ "picture": t.picture, "name": t.name, "when": t.when, "ringed": t.ringed }))
            .collect();
        json!({ "kind": self.kind.name(), "big": self.big, "tiles": tiles, "sentence": self.sentence, "blank": EMPTY }).to_string()
    }
}

/// The card: its page, the row focused, the reel while its page is open, and the session's facts.
#[derive(Clone, Debug, PartialEq)]
pub struct Card {
    pub page: Page,
    pub current: usize,
    pub reel: Option<Reel>,
    pub facts: Facts,
}

impl Default for Card {
    fn default() -> Self {
        Card { page: Page::First, current: 0, reel: None, facts: Facts::default() }
    }
}

impl Card {
    /// The card opened: its first page, on Resume.
    pub fn open(&mut self) {
        self.turn(Page::First, 0);
    }

    fn turn(&mut self, page: Page, at: usize) {
        self.page = page;
        self.current = at;
        if page != Page::Reel {
            self.reel = None;
        }
    }

    /// The page's rows. The first page holds Mistress's seven; its console rows (Change Disc, Swap Players) are owed.
    pub fn rows(&self) -> Vec<Row> {
        let facts = &self.facts;
        match self.page {
            Page::First => {
                let slot = slot_value(facts.slot, facts.saved());
                vec![
                    row(RESUME, "Resume", String::new()),
                    row(REWIND, "Rewind", rewind_value(facts)),
                    Row { steps: true, ..row(SAVE, "Save State", slot.clone()) },
                    Row { steps: true, ..row(LOAD, "Load State", slot) },
                    Row { steps: true, section_end: true, ..row(SPEED, "Speed", speed_value(facts.speed)) },
                    Row { opens: true, ..row("options", "Game Options", String::new()) },
                    Row { opens: true, ..row(LEAVE, "Leave", String::new()) },
                ]
            }
            Page::Leave => vec![
                row(LIBRARY, "Back to Library", String::new()),
                row(RESTART, "Restart Game...", String::new()),
                row(QUIT_GAME, "Quit Game...", String::new()),
                row(QUIT_EMUSEN, "Quit EmuSen...", String::new()),
            ],
            Page::Ask(ask) => ask
                .page()
                .rows
                .iter()
                .map(|r| row(if r.key == card::YES { card::YES } else { card::NO }, &r.label, String::new()))
                .collect(),
            Page::Reel => {
                let reel = self.reel.as_ref();
                let label = reel.and_then(Reel::focused).map(reel::Mark::label).unwrap_or_default();
                vec![
                    row(GO_BACK, &reel.map(Reel::go_back_words).unwrap_or_default(), String::new()),
                    Row { steps: true, ..row(MOMENT, "Moment", label) },
                    Row { dimmed: true, ..row(HOLD, "Hold ◂ to scrub", String::new()) },
                ]
            }
        }
    }

    /// The page's title over its rows: the game's on the first, the page's own name on the others.
    pub fn title(&self, game: &str) -> String {
        match self.page {
            Page::First => game.to_string(),
            Page::Leave => "Leave".to_string(),
            Page::Reel => "Rewind".to_string(),
            Page::Ask(ask) => ask.page().title,
        }
    }

    /// The way back under a page's title: the game, on every page but the first.
    pub fn breadcrumb(&self, game: &str) -> String {
        match self.page {
            Page::First => String::new(),
            Page::Ask(ask) => ask.page().breadcrumb,
            _ => game.to_string(),
        }
    }

    fn key_at(&self, index: usize) -> Option<&'static str> {
        self.rows().get(index).map(|r| r.key)
    }

    /// The row focused moved.
    pub fn focus(&mut self, index: usize) -> Outcome {
        if index == self.current || index >= self.rows().len() {
            return Outcome::Nothing;
        }
        self.current = index;
        Outcome::Shown
    }

    /// A row chosen by its index on the page.
    pub fn choose(&mut self, index: usize) -> Outcome {
        self.current = index.min(self.rows().len().saturating_sub(1));
        match self.key_at(index) {
            Some(key) => self.act(key),
            None => Outcome::Nothing,
        }
    }

    /// A row's or a declared action's key acted on (help.rs's `resume`).
    pub fn act(&mut self, key: &str) -> Outcome {
        match key {
            RESUME => Outcome::Resume,
            REWIND => self.open_reel(),
            SAVE => Outcome::Save(self.facts.slot),
            LOAD if self.facts.saved() => Outcome::Load(self.facts.slot),
            SPEED => self.step_speed(1),
            LEAVE => {
                self.turn(Page::Leave, 0);
                Outcome::Shown
            }
            LIBRARY => Outcome::Library,
            RESTART | QUIT_GAME | QUIT_EMUSEN if self.page == Page::Leave => {
                let ask = match key {
                    RESTART => Ask::Restart,
                    QUIT_GAME => Ask::QuitGame,
                    _ => Ask::QuitEmuSen,
                };
                self.turn(Page::Ask(ask), 0);
                Outcome::Shown
            }
            card::YES => match self.page {
                Page::Ask(Ask::Restart) => Outcome::Restart,
                Page::Ask(Ask::QuitGame) => Outcome::QuitGame,
                Page::Ask(Ask::QuitEmuSen) => Outcome::QuitEmuSen,
                _ => Outcome::Nothing,
            },
            card::NO => self.back(),
            GO_BACK => match self.reel.as_ref().and_then(Reel::focused) {
                Some(m) if !m.now => Outcome::RewindTo(m.frame),
                _ => Outcome::Resume,
            },
            _ => Outcome::Nothing,
        }
    }

    /// The reel's page, on its first stop before the present; nothing where Rewind cannot open.
    fn open_reel(&mut self) -> Outcome {
        if self.facts.rewind_why.is_some() {
            return Outcome::Nothing;
        }
        match Reel::open(reel::marks_of(&self.facts.held, self.facts.now, self.facts.hz)) {
            Some(r) => {
                self.turn(Page::Reel, 0);
                self.reel = Some(r);
                Outcome::Shown
            }
            None => Outcome::Nothing,
        }
    }

    /// Left (-1) or Right (+1) on a row; `held` for a held repeat.
    pub fn step(&mut self, index: usize, by: i32, held: bool) -> Outcome {
        match self.key_at(index) {
            Some(SAVE | LOAD) => Outcome::Slot((self.facts.slot as i64 - 1 + by as i64).rem_euclid(SLOTS as i64) as u32 + 1),
            Some(SPEED) => self.step_speed(by),
            Some(MOMENT | GO_BACK | HOLD) => self.reel_moved(|r| r.step(by, held)),
            _ => Outcome::Nothing,
        }
    }

    /// A shoulder (+1 further back) or a trigger (`end`) on the reel's page.
    pub fn stride(&mut self, back: i32, end: bool) -> Outcome {
        if self.page != Page::Reel {
            return Outcome::Nothing;
        }
        self.reel_moved(|r| if end { r.end(back) } else { r.stride(back) })
    }

    fn reel_moved(&mut self, change: impl FnOnce(&mut Reel)) -> Outcome {
        let Some(reel) = self.reel.as_mut() else { return Outcome::Nothing };
        let before = reel.at;
        change(reel);
        if reel.at == before { Outcome::Nothing } else { Outcome::Shown }
    }

    fn step_speed(&self, by: i32) -> Outcome {
        let at = SPEEDS.iter().position(|&s| s == self.facts.speed).unwrap_or(1) as i64;
        let to = SPEEDS[(at + by as i64).clamp(0, SPEEDS.len() as i64 - 1) as usize];
        if to == self.facts.speed { Outcome::Nothing } else { Outcome::Speed(to) }
    }

    /// B: back to the first page on the row that opened the page, or, on the first page, the game resumed.
    pub fn back(&mut self) -> Outcome {
        let (to, from) = match self.page {
            Page::First => return Outcome::Resume,
            Page::Leave => (Page::First, LEAVE),
            Page::Reel => (Page::First, REWIND),
            Page::Ask(ask) => (Page::Leave, ask.key()),
        };
        self.turn(to, 0);
        self.current = self.rows().iter().position(|r| r.key == from).unwrap_or(0);
        Outcome::Shown
    }

    /// The help bar the page shows: the first page's, with Change slot on Save and Load; the reel's; any other page's.
    pub fn help(&self) -> Screen {
        match (self.page, self.key_at(self.current)) {
            (Page::First, Some(SAVE | LOAD)) => Screen::PauseSlots,
            (Page::First, _) => Screen::Pause,
            (Page::Reel, _) => Screen::PauseReel,
            _ => Screen::PausePage,
        }
    }

    /// The context pane for the row focused (Mistress's `UpdateCardPane`).
    pub fn pane(&self) -> Pane {
        match (self.page, self.key_at(self.current)) {
            (Page::Reel, _) => self.moment_pane(),
            (Page::First, Some(REWIND)) => self.reel_pane(),
            (Page::First, Some(key @ (SAVE | LOAD))) => self.slots_pane(key == LOAD),
            _ => Pane::none(),
        }
    }

    fn reel_pane(&self) -> Pane {
        let facts = &self.facts;
        if let Some(why) = &facts.rewind_why {
            return Pane { kind: PaneKind::Reel, sentence: format!("Rewind: {why}"), ..Pane::none() };
        }
        let newest = facts.held.last().copied().unwrap_or(0);
        let start = facts.held.len().saturating_sub(3);
        let tiles = facts.held[start..]
            .iter()
            .map(|&frame| {
                let mark = reel::Mark { frame, seconds_back: (newest - frame) as f64 / facts.hz, now: false };
                Tile { picture: facts.moment_picture(&mark), name: reel::ago(mark.seconds_back), ..Tile::default() }
            })
            .collect();
        Pane { kind: PaneKind::Reel, tiles, sentence: reel::held_words(facts.rewind_seconds), ..Pane::none() }
    }

    fn moment_pane(&self) -> Pane {
        let Some(reel) = self.reel.as_ref() else { return Pane::none() };
        let Some(focused) = reel.focused() else { return Pane::none() };
        let tiles = reel
            .strip()
            .into_iter()
            .map(|i| {
                let mark = &reel.marks[i];
                Tile { picture: self.facts.moment_picture(mark), name: mark.short(), ringed: i == reel.at, ..Tile::default() }
            })
            .collect();
        Pane { kind: PaneKind::Moment, big: self.facts.moment_picture(focused), tiles, sentence: reel.sentence() }
    }

    /// Three slots with the current one among them, as near the middle as the ends allow, and what A does to it.
    fn slots_pane(&self, load: bool) -> Pane {
        let facts = &self.facts;
        let current = facts.slot;
        let first = (current as i64 - 2).clamp(0, SLOTS as i64 - 3) as u32 + 1;
        let tiles = (first..first + 3)
            .map(|slot| {
                let fact = facts.slot_fact(slot);
                let saved = fact.is_some_and(|f| f.exists);
                Tile {
                    picture: fact.filter(|_| saved).map(|f| f.picture.clone()).unwrap_or_default(),
                    name: format!("Slot {slot}"),
                    when: if saved { fact.map(|f| f.when.clone()).unwrap_or_default() } else { EMPTY.to_string() },
                    ringed: slot == current,
                }
            })
            .collect();
        let saved = facts.saved();
        let sentence = match (load, saved) {
            (true, true) => format!("A loads Slot {current}."),
            (true, false) => format!("Slot {current} is empty."),
            (false, true) => format!("A saves to Slot {current}, replacing it."),
            (false, false) => format!("A saves to Slot {current}."),
        };
        Pane { kind: PaneKind::Slots, tiles, sentence, ..Pane::none() }
    }
}
