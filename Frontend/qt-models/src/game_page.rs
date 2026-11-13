//! A game's page as plain Rust (EmuSen_Qt.md §7.8; EmuSen_Interface.md §2.5, §4 screen 3): its facts, its actions, its
//! five sections and what each press does, over one read of the game a source answers. No Qt; the bridge,
//! `game_page_model.rs`, only puts it on Qt. The media viewer is ES-DE's (`emusen_esde::scene::Viewer`) over the page's
//! media; Scrape This Game runs the scraper over the game; Edit Details is a deeper page of the game's edits; Cheats is a
//! deeper page of Options, the game's own cheat list, each a switch saved to its file (cheats.rs). Display crop is a
//! deeper page of Options stepping the game's own crop through Settings' overscan choices, stored in games.db; Package
//! is one showing the game's package and running the library tool's job that makes or checks it. Cover art opens
//! nothing yet, so its row is not shown; Open Folder is the program's folder action.

use crate::help::Screen;
use crate::question::Question;
use emusen_kit_core::input::UiButton;
use emusen_platform_client::library::GameFile;
use crate::words::BETWEEN;
use crate::display::Crop;
use crate::tools::jobs::Run;

/// What the page reads of one game and asks of the back end; the client crates or a test's fake answer.
pub trait GamePageSource: Send + Sync {
    /// Everything the page shows of the game, as the scan found its file, read afresh.
    fn read(&self, game: &GameFile) -> Result<Read, String>;
    /// One change, saved at once.
    fn change(&self, game: &GameFile, change: &Change) -> Result<(), String>;
    /// The file a state was saved from where its contents are not the game's, as the runtime recorded the state; none
    /// where they are the same or either is not known (Mistress's `SameGameOrConfirmedAsync`).
    fn other_copy(&self, _game: &GameFile, _state: &str) -> Result<Option<String>, String> {
        Ok(None)
    }
    /// A job of the platform library's tools started from its request, its events drained as it runs.
    fn start_job(&self, _request: &serde_json::Value) -> Result<Box<dyn crate::tools::cores::backend::Job + Send>, String> {
        Err("no tool jobs here".into())
    }
}

/// A change the page makes to the game's records.
#[derive(Clone, Debug, PartialEq)]
pub enum Change {
    /// The heart turned on or off.
    Favourite,
    /// The engine the game runs on; none is the console's own.
    Engine(Option<String>),
    /// The controller profile the game starts with; none is the console's.
    Profile(Option<String>),
    /// The television standard the game's picture is drawn for; none follows the game.
    Standard(Option<String>),
    /// A saved state deleted, by its file, with its picture and its record.
    DeleteState(String),
    /// The game hidden from every library view; its file is not touched.
    Hide,
    /// One of the game's details edited, by its field; none goes back to what was scraped.
    Edit(&'static str, Option<String>),
    /// The game's cheat at that place of its list switched on or off.
    Cheat(usize, bool),
    /// The game's own crop; none follows the console's overscan.
    Crop(Option<Crop>),
}

/// What the player is told when a change could not be made, and why (EmuSen_Qt.md §7.9).
pub fn change_failed(change: &Change, why: &str) -> String {
    let what = match change {
        Change::Favourite => "The favourite could not be changed",
        Change::Engine(_) => "The game's engine could not be stored",
        Change::Profile(_) => "The game's controller profile could not be stored",
        Change::Standard(_) => "The game's television standard could not be stored",
        Change::DeleteState(_) => "The state could not be deleted",
        Change::Hide => "The game could not be hidden",
        Change::Edit(..) => "The game's details could not be stored",
        Change::Cheat(..) => "The cheat was not switched",
        Change::Crop(_) => "The game's crop could not be stored",
    };
    format!("{what}: {why}")
}

/// One value of a stepped row: what is stored, none for the default, and its words.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Choice {
    pub value: Option<String>,
    pub text: String,
}

/// A saved state's place.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
    Number(u32),
    /// The state kept where the player left off.
    Auto,
}

/// One saved state: its file, its place, its picture (empty for none) and when it was saved, in seconds since 1970.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    pub path: String,
    pub slot: Slot,
    pub picture: String,
    pub when: i64,
}

/// What a picture or clip of the game is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaKind {
    /// The player's own screenshot, taken while playing.
    Taken,
    Screenshot,
    TitleScreen,
    Clip,
    Marquee,
    Fanart,
    /// The game's PDF manual.
    Manual,
}

/// One picture or clip of the game, the player's own or the scraper's, by its file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Media {
    pub path: String,
    pub kind: MediaKind,
}

/// One read of the game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Read {
    pub title: String,
    /// The console's name, as the facts begin.
    pub console: String,
    pub year: String,
    pub developer: String,
    pub publisher: String,
    pub description: String,
    pub played_seconds: i64,
    /// When it was last played, in seconds since 1970; none for never.
    pub last_played: Option<i64>,
    pub favourite: bool,
    /// The console's box, its width per unit of its height.
    pub cover_aspect: f64,
    /// Pictures and the clip as files; empty for none.
    pub cover: String,
    pub hero: String,
    pub screenshot: String,
    pub clip: String,
    /// The share of full the clip's sound plays at: the player's Volume (R-20261111-100).
    pub clip_volume: f64,
    /// The saved states, the most recent first.
    pub states: Vec<State>,
    pub media: Vec<Media>,
    /// The engines the game may run on, the console's own first (its value none); empty where none runs the console.
    pub engines: Vec<Choice>,
    pub engine: Option<String>,
    /// The console's controller profiles, the console's first; none where it has only one, so the row is left out.
    pub profiles: Option<Vec<Choice>>,
    pub profile: Option<String>,
    /// The game's own cheats, in its list's order; none where it has no list or an empty one.
    pub cheats: Vec<crate::cheats::Switch>,
    /// The game's crop and what it may be; none where its console has no overscan to crop.
    pub crop: Option<CropOffer>,
    /// The television standards, the game's own first (its value none); none where its console drove no television.
    pub standards: Option<Vec<Choice>>,
    pub standard: Option<String>,
    /// The game's package, and the patches layered on it.
    pub package: Package,
    pub layers: usize,
    /// The library tool's job for this game's package: making one where it has none, else checking it; none where
    /// neither can run (a game outside the library folder).
    pub package_job: Option<PackageJob>,
    /// Whether a scrape run is going, which Scrape This Game waits for.
    pub scraping: bool,
    /// The details Edit Details offers, in its order.
    pub details: Vec<Detail>,
}

/// One detail of the game as Edit Details shows it: its field, its value, and whether the player's edit holds it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Detail {
    pub field: &'static str,
    pub value: String,
    pub edited: bool,
}

/// Settings' overscan choices for a console as a game's own crop may take them, and the game's crop.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CropOffer {
    /// Settings' row's words for it.
    pub label: String,
    /// Each choice that hides a fixed part: its words and what it hides.
    pub choices: Vec<(String, Crop)>,
    /// The words for a crop of the player's own, which no choice names.
    pub custom: String,
    /// The game's own crop; none for the console's.
    pub own: Option<Crop>,
}

/// The words for the console's overscan, where the game has no crop of its own.
pub const CONSOLES: &str = "Console's";

impl CropOffer {
    /// The crop's words: the console's, a choice's, or the player's own.
    pub fn words(&self) -> String {
        match &self.own {
            None => CONSOLES.into(),
            Some(own) => self.choices.iter().find(|(_, c)| c.near(own)).map_or_else(|| self.custom.clone(), |(w, _)| w.clone()),
        }
    }

    /// One step on from the crop, round: the console's, then each choice; a crop of the player's own steps to the ends.
    pub fn stepped(&self, by: i32) -> Option<Crop> {
        let all: Vec<Option<Crop>> = std::iter::once(None).chain(self.choices.iter().map(|(_, c)| Some(*c))).collect();
        let at = match &self.own {
            None => Some(0),
            Some(own) => all.iter().position(|c| c.is_some_and(|c| c.near(own))),
        };
        let n = all.len() as i32;
        let next = match at {
            Some(at) => (at as i32 + by.signum()).rem_euclid(n),
            None if by > 0 => 0,
            None => n - 1,
        };
        all[next as usize]
    }
}

/// The library tool's job for a game's package: its key (`make` or `check`) and its request.
#[derive(Clone, Debug, PartialEq)]
pub struct PackageJob {
    pub key: &'static str,
    pub request: serde_json::Value,
}

pub const MAKE: &str = "make";
pub const CHECK: &str = "check";

/// What the game's package is (Mistress's `IGamePageHost.Package`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Package {
    #[default]
    None,
    /// Linked to the game where it is in the library.
    Linked,
    /// The game kept in the package.
    SelfContained,
    /// A package claims the game, but the game's file is not as it recorded, so the game plays as a plain one.
    Changed,
}

/// The standards' row's values: the game's own, then each standard by its name, the value in capitals ("ntsc", NTSC).
pub fn standard_choices(values: &[String]) -> Vec<Choice> {
    std::iter::once(Choice { value: None, text: "The game's".into() })
        .chain(values.iter().map(|v| Choice { value: Some(v.clone()), text: v.to_uppercase() }))
        .collect()
}

/// The package row's words: its mode, and its layers where it has any.
pub fn package_words(package: Package, layers: usize) -> String {
    let mode = match package {
        Package::None => return "Not a package".into(),
        Package::Linked => "Linked",
        Package::SelfContained => "Self-contained",
        Package::Changed => "Changed: plays as a plain game",
    };
    match layers {
        0 => mode.into(),
        1 => format!("{mode} {BETWEEN} 1 layer"),
        n => format!("{mode} {BETWEEN} {n} layers"),
    }
}

/// Asked before a state saved from another copy of the game is loaded into it (Mistress's `SameGameOrConfirmedAsync`).
pub fn other_copy_question(saved_from: &str, file: &str) -> Question {
    let sentence = format!(
        "This state was saved from {saved_from}, whose contents are not those of {file}. A state loaded into another version of a game can crash it. Load it anyway?"
    );
    let mut q = crate::question::confirm("A Different Copy of the Game", &[sentence.as_str()]);
    q.choices = vec!["Cancel".into(), "Load Anyway".into()];
    q
}

/// The page's five sections, in the strip's order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section {
    Overview,
    States,
    Options,
    Media,
    Details,
}

pub const SECTIONS: [Section; 5] = [Section::Overview, Section::States, Section::Options, Section::Media, Section::Details];

impl Section {
    pub fn name(self) -> &'static str {
        match self {
            Section::Overview => "Overview",
            Section::States => "States",
            Section::Options => "Options",
            Section::Media => "Media",
            Section::Details => "Details",
        }
    }
}

/// The actions' keys.
pub const PLAY: &str = "play";
pub const RESUME: &str = "resume";
pub const FAVOURITE: &str = "favourite";

/// The rows' keys; Cover art is a later piece's page (EmuSen_Interface.md §18.2).
pub const CHEATS: &str = "cheats";
/// A cheat on Cheats' page.
pub const CHEAT: &str = "cheat";
pub const PACKAGE: &str = "package";
pub const CROP: &str = "crop";
/// Display crop's row: the game's crop stepped.
pub const OVERSCAN: &str = "overscan";
/// Package's rows: what the package is, and its job.
pub const PACKAGED: &str = "packaged";
pub const JOB: &str = "job";
pub const SCRAPE: &str = "scrape";
pub const EDIT: &str = "edit";
pub const COVER: &str = "cover";
pub const FOLDER: &str = "folder";
pub const HIDE: &str = "hide";
pub const STANDARD: &str = "standard";

/// The fields Edit Details offers, the scraper's names for them (its `metadata.rs` FIELDS), and their words.
pub const DETAILS: [(&str, &str); 7] = [
    (crate::edits::NAME, "Name"),
    ("description", "Description"),
    ("releasedate", "Release date"),
    ("developer", "Developer"),
    ("publisher", "Publisher"),
    ("genre", "Genre"),
    ("players", "Players"),
];

pub const NO_DESCRIPTION: &str = "No description yet. Details ▸ Scrape This Game fetches one, and Edit Details writes one.";

/// The pad buttons whose glyphs lead Play and Favourite: A, and Y as the help bar's heart is.
const SOUTH: i32 = UiButton::Accept as i32;
const NORTH: i32 = UiButton::Search as i32;

/// An action as the page frame draws it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Action {
    pub key: &'static str,
    pub label: &'static str,
    pub value: String,
    /// The pad button whose glyph leads it; below zero for none.
    pub button: i32,
    pub filled: bool,
    /// Whether Left and Right step its value, and which way it can step from where it stands.
    pub stepped: bool,
    pub step_left: bool,
    pub step_right: bool,
}

/// A tile: a state, an empty slot or a picture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tile {
    pub key: String,
    pub label: String,
    pub line: String,
    pub picture: String,
    /// The state Resume would load.
    pub marked: bool,
}

/// A row: its words, whether Left and Right step it, and whether A opens a page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub key: &'static str,
    pub label: String,
    pub value: String,
    pub steps: bool,
    pub opens: bool,
    pub sentence: &'static str,
}

/// What the chosen section shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Content {
    /// The Overview: the description and a picture.
    Text { words: String, picture: String },
    Tiles(Vec<Tile>),
    Rows(Vec<Row>),
}

/// What a press asks of the program, past the page's own redraw.
#[derive(Clone, Debug, PartialEq)]
pub enum Outcome {
    Nothing,
    /// Saved through the source; the page is read again.
    Changed(Change),
    /// The game started, from a state's file where one is named.
    Play(Option<String>),
    /// Edit Details opened, Details' deeper page.
    Edit,
    /// Cheats opened, Options' deeper page.
    Cheats,
    /// Display crop opened, Options' deeper page.
    Crop,
    /// Package opened, Options' deeper page.
    Package,
    /// The package's job started from its request.
    Job(PackageJob),
    /// The package's job running asked to stop.
    StopJob,
    /// The media viewer opened on the Media section's tile at this index.
    View(usize),
    /// A scrape of the game started.
    Scrape,
    /// The run going asked first whether to stop it; Yes stops it.
    Stop(Question),
    /// Words typed for a detail: its field, the question's title, and the words it starts from.
    Type { field: &'static str, title: String, start: String },
    /// The folder holding the game's file, shown in the system's file manager.
    OpenFolder,
    /// A question first; the change is made when its second answer, Yes, is chosen.
    Ask(Question, Change),
}

/// The page over one read: the section shown and the state Resume stands on.
#[derive(Clone, Debug, Default)]
pub struct GamePage {
    pub read: Read,
    pub section: usize,
    resume_at: usize,
    /// Now, in seconds since 1970, for how long ago things were.
    pub now: i64,
    /// Whether Details shows Edit Details, its deeper page.
    pub editing: bool,
    /// Whether Options shows Cheats, its deeper page.
    pub cheating: bool,
    /// Whether Options shows Display crop or Package, its deeper pages.
    pub cropping: bool,
    pub packaging: bool,
    /// The package's job: its key, whether it is running, and what it has said; none before one started.
    pub job: Option<(&'static str, bool, Run)>,
    /// This page's scrape: whether it is going, and what it last said; none before one started.
    pub run: Option<(bool, String)>,
}

/// How long ago, as the page says it: Just now, 12 min ago, 2 h ago, Yesterday, 3 days ago, then the date (Mistress's
/// `GamePage.Ago`); days are counted in UTC.
pub fn ago(at: i64, now: i64) -> String {
    let since = (now - at).max(0);
    let days = now.div_euclid(86_400) - at.div_euclid(86_400);
    match since {
        s if s < 60 => "Just now".into(),
        s if s < 3_600 => format!("{} min ago", s / 60),
        s if days == 0 => format!("{} h ago", s / 3_600),
        _ if days == 1 => "Yesterday".into(),
        _ if days < 14 => format!("{days} days ago"),
        _ => date(at),
    }
}

/// A question that does something cannot be undone: No first and focused, then the word for doing it; B answers No.
fn ask(title: &str, sentence: &str, yes: &str) -> Question {
    let mut q = crate::question::confirm(title, &[sentence]);
    q.choices[1] = yes.to_owned();
    q
}

/// The media store's type for a kind of the page's media, as ES-DE's viewer names its items.
pub fn kind_word(kind: MediaKind) -> &'static str {
    match kind {
        MediaKind::Taken | MediaKind::Screenshot => "screenshot",
        MediaKind::TitleScreen => "titlescreen",
        MediaKind::Clip => "video",
        MediaKind::Marquee => "marquee",
        MediaKind::Fanart => "fanart",
        MediaKind::Manual => "manual",
    }
}

/// What a scrape's progress says of one game's run (`scrape_run`, its one owner).
pub use crate::scrape_run::run_words;

/// "1 Nov 2026".
fn date(at: i64) -> String {
    const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    let utc = crate::words::utc(at);
    let (y, m, d) = (&utc[0..4], utc[5..7].parse::<usize>().unwrap_or(1), utc[8..10].parse::<u32>().unwrap_or(1));
    format!("{d} {} {y}", MONTHS[m.clamp(1, 12) - 1])
}

/// Time played: "3 h 12 min", "45 min".
pub fn played(seconds: i64) -> String {
    let (h, m) = (seconds / 3_600, seconds % 3_600 / 60);
    if h >= 1 {
        format!("{h} h {m} min")
    } else {
        format!("{} min", m.max(1))
    }
}

fn slot_name(slot: Slot) -> String {
    match slot {
        Slot::Number(n) => format!("Slot {n}"),
        Slot::Auto => "Auto".into(),
    }
}

impl GamePage {
    pub fn new(read: Read, now: i64) -> GamePage {
        GamePage { read, now, ..GamePage::default() }
    }

    /// A new read of the same game, the section and Resume's state kept where they still stand; Cheats gives way to
    /// Options where the game has none to show.
    pub fn reread(&mut self, read: Read) {
        self.resume_at = self.resume_at.min(read.states.len().saturating_sub(1));
        self.cheating &= !read.cheats.is_empty();
        self.read = read;
    }

    pub fn shown(&self) -> Section {
        SECTIONS[self.section.min(SECTIONS.len() - 1)]
    }

    /// The facts line: console · year · developer · publisher · time played · last played (§4 screen 3).
    pub fn facts(&self) -> String {
        let r = &self.read;
        let mut parts = vec![r.console.clone()];
        parts.extend([&r.year, &r.developer].into_iter().filter(|p| !p.is_empty()).cloned());
        if !r.publisher.is_empty() && r.publisher != r.developer {
            parts.push(r.publisher.clone());
        }
        if r.played_seconds >= 60 {
            parts.push(played(r.played_seconds));
        }
        parts.push(match r.last_played {
            Some(at) => format!("Last played {}", ago(at, self.now).to_lowercase()),
            None => "Never played".into(),
        });
        parts.join(&format!(" {BETWEEN} "))
    }

    /// The strip's names, States with its count where it has any; a reader hears the count, the strip draws the name.
    pub fn sections(&self) -> Vec<String> {
        SECTIONS.iter().map(|s| s.name().to_owned()).collect()
    }

    /// The page opened at a part of it by a door: `cheats` is Options' Cheats; anything else the page's start.
    pub fn open_at(&mut self, part: &str) {
        if part == CHEATS {
            self.section = SECTIONS.iter().position(|s| *s == Section::Options).unwrap_or(0);
            self.cheating = true;
        }
    }

    pub fn choose_section(&mut self, index: usize) {
        self.section = index.min(SECTIONS.len() - 1);
        self.editing = false;
        self.cheating = false;
        self.cropping = false;
        self.packaging = false;
    }

    /// B on the page: Edit Details goes back to Details, and Cheats, Display crop and Package to Options, and says so;
    /// else the page is left.
    pub fn back(&mut self) -> bool {
        let deeper = [&mut self.editing, &mut self.cheating, &mut self.cropping, &mut self.packaging];
        deeper.into_iter().fold(false, |went, d| std::mem::replace(d, false) | went)
    }

    fn state_words(&self, s: &State) -> String {
        format!("{} {BETWEEN} {}", slot_name(s.slot), ago(s.when, self.now).to_lowercase())
    }

    pub fn actions(&self) -> Vec<Action> {
        let mut out = vec![Action { key: PLAY, label: "Play", value: String::new(), button: SOUTH, filled: true, stepped: false, step_left: false, step_right: false }];
        let states = &self.read.states;
        if let Some(s) = states.get(self.resume_at.min(states.len().saturating_sub(1))) {
            let many = states.len() > 1;
            out.push(Action {
                key: RESUME,
                label: "Resume",
                value: self.state_words(s),
                button: -1,
                filled: false,
                stepped: many,
                step_left: many && self.resume_at > 0,
                step_right: many && self.resume_at + 1 < states.len(),
            });
        }
        let heart = if self.read.favourite { "♥" } else { "♡" };
        out.push(Action { key: FAVOURITE, label: "Favourite", value: heart.into(), button: NORTH, filled: false, stepped: false, step_left: false, step_right: false });
        out
    }

    /// A on an action.
    pub fn act(&self, index: usize) -> Outcome {
        match self.actions().get(index).map(|a| a.key) {
            Some(PLAY) => Outcome::Play(None),
            Some(RESUME) => Outcome::Play(self.read.states.get(self.resume_at).map(|s| s.path.clone())),
            Some(FAVOURITE) => Outcome::Changed(Change::Favourite),
            _ => Outcome::Nothing,
        }
    }

    /// Left or Right on an action: Resume steps its state while it can, and says whether it did; past either end the
    /// focus moves on, so every action is reached (LunaP's `MoveInActions`).
    pub fn step_action(&mut self, index: usize, by: i32) -> bool {
        let Some(a) = self.actions().get(index).cloned() else { return false };
        if !(a.key == RESUME && ((by < 0 && a.step_left) || (by > 0 && a.step_right))) {
            return false;
        }
        self.resume_at = (self.resume_at as i64 + i64::from(by.signum())) as usize;
        true
    }

    pub fn content(&self) -> Content {
        match self.shown() {
            Section::Overview => {
                let r = &self.read;
                let words = if r.description.is_empty() { NO_DESCRIPTION.to_owned() } else { r.description.clone() };
                let picture = if r.screenshot.is_empty() { r.hero.clone() } else { r.screenshot.clone() };
                Content::Text { words, picture }
            }
            Section::States => Content::Tiles(self.state_tiles()),
            Section::Options if self.cheating => Content::Rows(self.cheat_rows()),
            Section::Options if self.cropping => Content::Rows(self.crop_rows()),
            Section::Options if self.packaging => Content::Rows(self.package_rows()),
            Section::Options => Content::Rows(self.option_rows()),
            Section::Media => Content::Tiles(self.media_tiles()),
            Section::Details if self.editing => Content::Rows(self.edit_rows()),
            Section::Details => Content::Rows(self.detail_rows()),
        }
    }

    /// Slots 1 to 8, the automatic state after the last slot in use, before the empty ones, as the design draws it.
    fn state_tiles(&self) -> Vec<Tile> {
        let states = &self.read.states;
        let resumed = states.get(self.resume_at).map(|s| s.path.as_str());
        let tile = |s: &State, line: String| Tile {
            key: s.path.clone(),
            label: slot_name(s.slot),
            line,
            picture: s.picture.clone(),
            marked: Some(s.path.as_str()) == resumed,
        };
        let of = |slot: Slot| states.iter().find(|s| s.slot == slot);
        let last_used = (1..=crate::pause::SLOTS).filter(|n| of(Slot::Number(*n)).is_some()).max().unwrap_or(1);
        let mut out = Vec::new();
        for n in 1..=crate::pause::SLOTS {
            match of(Slot::Number(n)) {
                Some(s) => out.push(tile(s, ago(s.when, self.now))),
                None => out.push(Tile { key: format!("empty.{n}"), label: format!("Slot {n}"), line: "Empty".into(), picture: String::new(), marked: false }),
            }
            if n == last_used {
                if let Some(auto) = of(Slot::Auto) {
                    out.push(tile(auto, format!("Where you left off {BETWEEN} {}", ago(auto.when, self.now))));
                }
            }
        }
        out
    }

    /// Each picture's, clip's and manual's words, in the media's order: the player's screenshots numbered.
    pub fn media_labels(&self) -> Vec<String> {
        let mut taken = 0;
        let mut label = |k: MediaKind| match k {
            MediaKind::Taken => {
                taken += 1;
                format!("Screenshot {taken}")
            }
            MediaKind::Screenshot => "Scraped screenshot".into(),
            MediaKind::TitleScreen => "Title screen".into(),
            MediaKind::Clip => "Clip".into(),
            MediaKind::Marquee => "Marquee".into(),
            MediaKind::Fanart => "Fan art".into(),
            MediaKind::Manual => "Manual".into(),
        };
        self.read.media.iter().map(|m| label(m.kind)).collect()
    }

    fn media_tiles(&self) -> Vec<Tile> {
        let r = &self.read;
        // A clip's tile shows the hero, a manual's the cover.
        let picture = |m: &Media| match m.kind {
            MediaKind::Clip => r.hero.clone(),
            MediaKind::Manual if !r.cover.is_empty() => r.cover.clone(),
            MediaKind::Manual => r.hero.clone(),
            _ => m.path.clone(),
        };
        r.media.iter().zip(self.media_labels()).map(|(m, label)| Tile { key: m.path.clone(), label, line: String::new(), picture: picture(m), marked: false }).collect()
    }

    fn text_of(choices: &[Choice], value: &Option<String>) -> String {
        choices.iter().find(|c| &c.value == value).or(choices.first()).map(|c| c.text.clone()).unwrap_or_default()
    }

    pub fn option_rows(&self) -> Vec<Row> {
        let r = &self.read;
        let mut rows = Vec::new();
        if !r.engines.is_empty() {
            rows.push(Row {
                key: "engine",
                label: "Engine".into(),
                value: Self::text_of(&r.engines, &r.engine),
                steps: r.engines.len() > 1,
                opens: false,
                sentence: "The engine this game runs on; the console's own unless one is chosen here. Cores lists them all.",
            });
        }
        if let Some(profiles) = &r.profiles {
            rows.push(Row {
                key: "profile",
                label: "Controller profile".into(),
                value: Self::text_of(profiles, &r.profile),
                steps: true,
                opens: false,
                sentence: "The bindings this game starts with; the console's profile unless one is chosen here. Settings ▸ Controllers ▸ Profiles makes them.",
            });
        }
        if let Some(crop) = &r.crop {
            rows.push(Row {
                key: CROP,
                label: "Display crop".into(),
                value: if crop.own.is_some() { "This game's" } else { CONSOLES }.into(),
                steps: false,
                opens: true,
                sentence: "How much of the picture's edge is cut away for this game.",
            });
        }
        if let Some(standards) = &r.standards {
            rows.push(Row {
                key: STANDARD,
                label: "Television standard".into(),
                value: Self::text_of(standards, &r.standard),
                steps: true,
                opens: false,
                sentence: "The television the picture is drawn for: the game's own, NTSC or PAL.",
            });
        }
        if !r.cheats.is_empty() {
            rows.push(Row {
                key: CHEATS,
                label: "Cheats".into(),
                value: crate::cheats::count_words(&r.cheats),
                steps: false,
                opens: true,
                sentence: "The game's cheats, on or off; the same switches the Pause Card shows.",
            });
        }
        if r.package_job.is_some() {
            let package = package_words(r.package, r.layers);
            rows.push(Row { key: PACKAGE, label: "Package".into(), value: package, steps: false, opens: true, sentence: "The game's package and its layers, from Packages." });
        }
        rows
    }

    /// Display crop's row: the game's crop, stepped through Settings' overscan choices.
    fn crop_rows(&self) -> Vec<Row> {
        let Some(crop) = &self.read.crop else { return Vec::new() };
        vec![Row { key: OVERSCAN, label: crop.label.clone(), value: crop.words(), steps: true, opens: false, sentence: "" }]
    }

    /// Package's rows: what the package is, and the job that makes or checks it with how far it has gone.
    fn package_rows(&self) -> Vec<Row> {
        let r = &self.read;
        let mut rows = vec![Row { key: PACKAGED, label: "Package".into(), value: package_words(r.package, r.layers), steps: false, opens: false, sentence: "" }];
        if let Some(job) = &r.package_job {
            let label = if job.key == MAKE { "Make Package" } else { "Check Package" };
            let value = match &self.job {
                Some((key, running, run)) if *key == job.key => crate::tools::jobs::state_words(*running, Some(run)).words,
                _ => "…".into(),
            };
            rows.push(Row { key: JOB, label: label.into(), value, steps: false, opens: false, sentence: "" });
        }
        rows
    }

    /// Whether the package's job is running.
    pub fn job_running(&self) -> bool {
        self.job.as_ref().is_some_and(|(_, running, _)| *running)
    }

    /// Cheats' rows: each of the game's cheats, a switch.
    fn cheat_rows(&self) -> Vec<Row> {
        let row = |c: &crate::cheats::Switch| Row {
            key: CHEAT,
            label: c.description.clone(),
            value: crate::cheats::on_words(c.on).into(),
            steps: true,
            opens: false,
            sentence: "",
        };
        self.read.cheats.iter().map(row).collect()
    }

    fn detail_rows(&self) -> Vec<Row> {
        let row = |key, label: &str, value: &str, opens| Row { key, label: label.into(), value: value.to_owned(), steps: false, opens, sentence: "" };
        let scrape = match &self.run {
            Some((_, words)) => words.as_str(),
            None if self.read.scraping => "A run is going",
            None => "…",
        };
        vec![row(SCRAPE, "Scrape This Game", scrape, false), row(EDIT, "Edit Details", "", true), row(FOLDER, "Open Folder", "", false), row(HIDE, "Hide from Library", "…", false)]
    }

    /// Edit Details' rows: each detail with its value; A types it, Y puts back what was scraped.
    fn edit_rows(&self) -> Vec<Row> {
        DETAILS
            .iter()
            .map(|(field, label)| {
                let value = self.read.details.iter().find(|d| d.field == *field).map(|d| d.value.clone()).unwrap_or_default();
                Row { key: field, label: label.to_string(), value, steps: false, opens: true, sentence: "" }
            })
            .collect()
    }

    /// Whether this page's scrape is going.
    pub fn running(&self) -> bool {
        self.run.as_ref().is_some_and(|(going, _)| *going)
    }

    /// A on a row: Hide from Library asks first; any other row is its door.
    pub fn choose_row(&self, index: usize) -> Outcome {
        let rows = match self.content() {
            Content::Rows(rows) => rows,
            _ => return Outcome::Nothing,
        };
        match rows.get(index) {
            Some(row) if row.key == HIDE && self.shown() == Section::Details => {
                let sentence = "It leaves every library view until Settings lists hidden games again. Its file is not touched.";
                Outcome::Ask(ask(&format!("Hide {}?", self.read.title), sentence, "Hide"), Change::Hide)
            }
            Some(row) if self.editing => {
                let start = self.read.details.iter().find(|d| d.field == row.key).map(|d| d.value.clone()).unwrap_or_default();
                Outcome::Type { field: row.key, title: format!("{} › {}", self.read.title, row.label), start }
            }
            Some(row) if row.key == SCRAPE && self.running() => {
                Outcome::Stop(crate::scrape_run::stop_question())
            }
            // Another run going (the Scraper tool's) holds Scrape This Game until it ends.
            Some(row) if row.key == SCRAPE && self.read.scraping => Outcome::Nothing,
            Some(row) if row.key == SCRAPE => Outcome::Scrape,
            Some(row) if row.key == FOLDER => Outcome::OpenFolder,
            Some(row) if row.key == EDIT => Outcome::Edit,
            Some(row) if row.key == CHEATS => Outcome::Cheats,
            Some(row) if row.key == CHEAT => self.switch(index),
            Some(row) if row.key == CROP => Outcome::Crop,
            Some(row) if row.key == PACKAGE => Outcome::Package,
            Some(row) if row.key == JOB && self.job_running() => Outcome::StopJob,
            Some(row) if row.key == JOB => self.read.package_job.clone().map_or(Outcome::Nothing, Outcome::Job),
            _ => Outcome::Nothing,
        }
    }

    /// X on a state's tile: deleting it asks first, naming the state and when it was saved (Mistress's
    /// `DeleteStateAsync`).
    pub fn delete(&self, index: usize) -> Outcome {
        let Content::Tiles(tiles) = self.content() else { return Outcome::Nothing };
        if self.shown() != Section::States {
            return Outcome::Nothing;
        }
        let Some(state) = tiles.get(index).and_then(|t| self.read.states.iter().find(|s| s.path == t.key)) else { return Outcome::Nothing };
        let slot = slot_name(state.slot);
        let sentence = format!("{}'s {slot}, saved {}, is deleted. This cannot be undone.", self.read.title, ago(state.when, self.now).to_lowercase());
        Outcome::Ask(ask(&format!("Delete {slot}?"), &sentence, "Delete"), Change::DeleteState(state.path.clone()))
    }

    /// A change saved, taken into the page's facts at once, so the next step steps from it rather than from the facts read
    /// before it; the read that follows the save confirms them (EmuSen_Qt.md §7.8).
    pub fn changed(&mut self, change: &Change) {
        let r = &mut self.read;
        match change {
            Change::Favourite => r.favourite = !r.favourite,
            Change::Engine(v) => r.engine = v.clone(),
            Change::Profile(v) => r.profile = v.clone(),
            Change::Standard(v) => r.standard = v.clone(),
            Change::Edit(field, value) => {
                if let Some(d) = r.details.iter_mut().find(|d| d.field == *field) {
                    d.edited = value.is_some();
                    if let Some(v) = value {
                        d.value = v.clone();
                    }
                }
            }
            Change::Cheat(index, on) => {
                if let Some(c) = r.cheats.get_mut(*index) {
                    c.on = *on;
                }
            }
            Change::Crop(v) => {
                if let Some(crop) = r.crop.as_mut() {
                    crop.own = *v;
                }
            }
            Change::DeleteState(_) | Change::Hide => {}
        }
    }

    /// Left or Right on a row: the engine, the profile or the standard stepped, wrapping, and saved.
    pub fn step_row(&self, index: usize, by: i32) -> Outcome {
        let Content::Rows(rows) = self.content() else { return Outcome::Nothing };
        let step = |choices: &[Choice], value: &Option<String>| {
            let at = choices.iter().position(|c| &c.value == value).unwrap_or(0) as i64;
            let n = choices.len() as i64;
            choices[(at + i64::from(by.signum())).rem_euclid(n) as usize].value.clone()
        };
        match rows.get(index).filter(|r| r.steps).map(|r| r.key) {
            Some("engine") => Outcome::Changed(Change::Engine(step(&self.read.engines, &self.read.engine))),
            Some("profile") => match &self.read.profiles {
                Some(p) => Outcome::Changed(Change::Profile(step(p, &self.read.profile))),
                None => Outcome::Nothing,
            },
            Some(CHEAT) => self.switch(index),
            Some(OVERSCAN) => match &self.read.crop {
                Some(c) => Outcome::Changed(Change::Crop(c.stepped(by))),
                None => Outcome::Nothing,
            },
            Some(STANDARD) => match &self.read.standards {
                Some(s) => Outcome::Changed(Change::Standard(step(s, &self.read.standard))),
                None => Outcome::Nothing,
            },
            _ => Outcome::Nothing,
        }
    }

    /// A cheat turned the other way and saved to its list.
    fn switch(&self, index: usize) -> Outcome {
        match self.read.cheats.get(index) {
            Some(c) if self.cheating => Outcome::Changed(Change::Cheat(index, !c.on)),
            _ => Outcome::Nothing,
        }
    }

    /// Y on Edit Details' row: the detail goes back to what was scraped, where the player's edit holds it.
    pub fn reset(&self, index: usize) -> Outcome {
        if !self.editing {
            return Outcome::Nothing;
        }
        match DETAILS.get(index).and_then(|(field, _)| self.read.details.iter().find(|d| d.field == *field)) {
            Some(d) if d.edited => Outcome::Changed(Change::Edit(d.field, None)),
            _ => Outcome::Nothing,
        }
    }

    /// The media viewer on the Media section's tile at `index`: ES-DE's, its items the pictures and clips in the page's
    /// order and its manual the game's PDF, opened on the manual where that tile was chosen.
    pub fn viewer(&self, index: usize) -> Option<emusen_esde::scene::Viewer> {
        use emusen_esde::scene::{viewer::Entry, ManualView, Viewer};
        let chosen = self.read.media.get(index)?;
        let shown: Vec<&Media> = self.read.media.iter().filter(|m| m.kind != MediaKind::Manual).collect();
        let entries = shown.iter().map(|m| Entry { kind: kind_word(m.kind).to_owned(), source: m.path.clone() }).collect();
        let manual = self.read.media.iter().find(|m| m.kind == MediaKind::Manual).map(|m| m.path.clone());
        let at = shown.iter().position(|m| m.path == chosen.path).unwrap_or(0);
        let reading = (chosen.kind == MediaKind::Manual).then(ManualView::default);
        Some(Viewer { entries, index: at, manual, reading, pages: 0 })
    }

    /// The viewer's caption: what is shown and where it stands among the media, or the manual's page.
    pub fn caption(&self, viewer: &emusen_esde::scene::Viewer) -> String {
        if let Some(view) = viewer.reading {
            return match viewer.pages {
                0 => "Manual".into(),
                n => format!("Manual {BETWEEN} Page {} of {n}", view.page + 1),
            };
        }
        let labels: Vec<(String, &Media)> = self.media_labels().into_iter().zip(&self.read.media).filter(|(_, m)| m.kind != MediaKind::Manual).collect();
        match labels.get(viewer.index) {
            Some((label, _)) => format!("{label} {BETWEEN} {} of {}", viewer.index + 1, labels.len()),
            None => String::new(),
        }
    }

    /// A on a tile: a state loads, a picture opens the viewer.
    pub fn choose_tile(&self, index: usize) -> Outcome {
        match (self.shown(), self.content()) {
            (Section::States, Content::Tiles(tiles)) => match tiles.get(index) {
                Some(t) if !t.key.starts_with("empty.") => Outcome::Play(Some(t.key.clone())),
                _ => Outcome::Nothing,
            },
            (Section::Media, Content::Tiles(tiles)) if index < tiles.len() => Outcome::View(index),
            _ => Outcome::Nothing,
        }
    }

    /// The help bar for what is focused: an action by its index, else the content's row or tile.
    pub fn help(&self, action: Option<usize>, content: usize) -> Screen {
        if let Some(i) = action {
            return match self.actions().get(i) {
                Some(a) if a.key == RESUME && a.stepped => Screen::GamePageResumeSlots,
                Some(a) if a.key == RESUME => Screen::GamePageResume,
                Some(a) if a.key == FAVOURITE => Screen::GamePageFavourite,
                _ => Screen::GamePage,
            };
        }
        match self.content() {
            Content::Tiles(tiles) if self.shown() == Section::States => match tiles.get(content) {
                Some(t) if !t.key.starts_with("empty.") => Screen::GamePageLoad,
                _ => Screen::GamePageNone,
            },
            Content::Tiles(_) => Screen::GamePageView,
            Content::Rows(rows) => match rows.get(content) {
                Some(_) if self.editing => Screen::GamePageEdit,
                Some(r) if r.steps => Screen::GamePageValue,
                Some(_) if self.shown() == Section::Options => Screen::GamePageChange,
                Some(_) => Screen::GamePageChoose,
                None => Screen::GamePageNone,
            },
            Content::Text { .. } => Screen::GamePage,
        }
    }

    /// What a reader hears for the States tab: "States, 4".
    pub fn states_heard(&self) -> String {
        match self.read.states.len() {
            0 => "States".into(),
            n => format!("States, {n}"),
        }
    }
}
