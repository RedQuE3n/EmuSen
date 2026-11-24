//! A game's page as plain Rust (EmuSen_Qt.md §7.8; EmuSen_Interface.md §2.5, §4 screen 3): its facts, its actions, its
//! five sections and what each press does, over one read of the game a source answers. No Qt; the bridge,
//! `game_page_model.rs`, only puts it on Qt. The media viewer is ES-DE's (`emusen_esde::scene::Viewer`) over the page's
//! media; Scrape This Game runs the scraper over the game; Edit Details is a deeper page of the game's edits; Cheats is a
//! deeper page of Options, the game's own cheat list, each a switch saved to its file (cheats.rs). Display crop is a
//! deeper page of Options stepping the game's own crop through Settings' overscan choices, stored in games.db; Package
//! is one showing the game's package and running the library tool's job that makes or checks it. Cover art is a deeper
//! page of Details choosing which of the game's own pictures is its cover, as the media viewer's Set as cover does; the
//! choice is the scraper's `cover` (EmuSen_Backend.md §30.6). Open Folder is the program's folder action.

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
    /// One of the game's own pictures, by its media type, made its cover; none puts back the cover found.
    Cover(Option<String>),
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
        Change::Cover(_) => "The game's cover could not be chosen",
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
    /// Cover art's cards: a cover borrowed from another game while it is shown, the cover found where one is, then each of
    /// the game's own pictures that can be its cover; empty where there is none of them or no pictures are looked for.
    pub covers: Vec<CoverCard>,
    /// Why Cover art has no cards where its pictures could not be looked for: the words of what failed.
    pub covers_unread: Option<String>,
    /// What the game shows as its cover.
    pub cover_in_use: CoverChoice,
}

/// What a card of Cover art stands for (decided 2026-11-14, R-20261114-10).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum CoverChoice {
    /// Another game's cover, by that game's file: a card while the game shows it, on which A changes nothing.
    Borrowed(String),
    /// The cover found for the game.
    #[default]
    Found,
    /// One of the game's own pictures, by its media type.
    Own(String),
}

/// One card of Cover art: what it stands for, its words and its picture's file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoverCard {
    pub choice: CoverChoice,
    pub text: String,
    pub picture: String,
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
    /// The most a crop may hide at an edge, a share of the picture, by Serenity's catalogue.
    pub most: f64,
}

/// The words for the console's overscan, where the game has no crop of its own.
pub const CONSOLES: &str = "Console's";

impl CropOffer {
    /// Whether the crop is the player's own: chosen as such (`chosen`), or one no choice names.
    pub fn is_custom(&self, chosen: bool) -> bool {
        chosen || self.own.is_some_and(|own| !self.choices.iter().any(|(_, c)| c.near(&own)))
    }

    /// The crop's words: the console's, a choice's, or the player's own.
    pub fn words(&self, chosen: bool) -> String {
        match &self.own {
            _ if self.is_custom(chosen) => self.custom.clone(),
            None => CONSOLES.into(),
            Some(own) => self.choices.iter().find(|(_, c)| c.near(own)).map_or_else(|| self.custom.clone(), |(w, _)| w.clone()),
        }
    }

    /// One step on from the crop, round: the console's, each choice, then a crop of the player's own, which starts from
    /// the crop shown; and whether that last is where it landed.
    pub fn stepped(&self, by: i32, chosen: bool) -> (Option<Crop>, bool) {
        let n = self.choices.len() as i32 + 2;
        let at = match &self.own {
            _ if self.is_custom(chosen) => n - 1,
            None => 0,
            Some(own) => self.choices.iter().position(|(_, c)| c.near(own)).map_or(n - 1, |i| i as i32 + 1),
        };
        match (at + by.signum()).rem_euclid(n) {
            0 => (None, false),
            next if next == n - 1 => (Some(self.own.unwrap_or_default()), true),
            next => (Some(self.choices[next as usize - 1].1), false),
        }
    }

    /// The crop with one edge a step on, in the catalogue's bounds.
    pub fn edge_stepped(&self, edge: crate::display::Edge, by: i32) -> Crop {
        let mut crop = self.own.unwrap_or_default();
        let percent = *edge.of(&mut crop) * 100.0;
        *edge.of(&mut crop) = crate::display::edge_stepped(self.most, percent, by) / 100.0;
        crop
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

/// The rows' keys.
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
/// Cover art's row, and its page's title.
pub const COVER_ART: &str = "Cover Art";
/// A card on Cover art's page, by its place.
pub const COVER_CARD: &str = "coverCard";
/// The card that clears the cover chosen where no cover was found for the game (R-20261114-28).
pub const COVER_NONE: &str = "None";
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

/// Where an item stands among its list: "2 of 8".
pub fn place(index: usize, count: usize) -> String {
    format!("{} of {count}", index + 1)
}

/// What A on Cover art's row says where its pictures could not be looked for, before why.
pub const COVER_UNREAD: &str = "Cover art could not be opened";

/// Cover art's mark on the card in use, as EmuSen_Interface.md §18.2 words it.
pub const SHOWN: &str = "In use";

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
    /// Cover art opened, Details' deeper page.
    CoverArt,
    /// Words the page says, as it says a failure.
    Said(String),
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
    /// Whether Display crop's row was stepped to a crop of the player's own, whose edges follow it.
    pub custom: bool,
    pub packaging: bool,
    /// Whether Details shows Cover art, its deeper page.
    pub covering: bool,
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

/// The words for a card Cover art offers: the other game a borrowed cover is from, or the cover found; one of the game's
/// own pictures is named by its kind's words, which the source reads (`game_page_source.rs` `kind_words`), so here by
/// its type alone.
pub fn cover_words(choice: &CoverChoice) -> String {
    match choice {
        CoverChoice::Borrowed(from) => format!("Borrowed from {}", crate::library::title_of(from)),
        CoverChoice::Found => "Its own".into(),
        CoverChoice::Own(t) => t.clone(),
    }
}

/// The Media section's word for a kind of the page's media; the player's own screenshots are numbered by `media_labels`.
pub fn media_word(kind: MediaKind) -> &'static str {
    match kind {
        MediaKind::Taken => "Screenshot",
        MediaKind::Screenshot => "Scraped screenshot",
        MediaKind::TitleScreen => "Title screen",
        MediaKind::Clip => "Clip",
        MediaKind::Marquee => "Marquee",
        MediaKind::Fanart => "Fan art",
        MediaKind::Manual => "Manual",
    }
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
        self.covering &= !read.covers.is_empty();
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
            None => NEVER_PLAYED.into(),
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
        self.custom = false;
        self.packaging = false;
        self.covering = false;
    }

    /// B on the page: Edit Details and Cover art go back to Details, and Cheats, Display crop and Package to Options, and
    /// says so; else the page is left.
    pub fn back(&mut self) -> bool {
        self.custom = false;
        let deeper = [&mut self.editing, &mut self.cheating, &mut self.cropping, &mut self.packaging, &mut self.covering];
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
        let heart = if self.read.favourite { HEART_ON } else { HEART_OFF };
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
            Section::Details if self.covering => Content::Tiles(self.cover_tiles()),
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
                format!("{} {taken}", media_word(k))
            }
            other => media_word(other).into(),
        };
        self.read.media.iter().map(|m| label(m.kind)).collect()
    }

    pub fn media_tiles(&self) -> Vec<Tile> {
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
                label: CHEATS_WORD.into(),
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

    /// Display crop's rows: the game's crop, stepped through Settings' overscan choices, and a crop of the player's own's
    /// four edges (display.rs `EDGES`).
    fn crop_rows(&self) -> Vec<Row> {
        let Some(crop) = &self.read.crop else { return Vec::new() };
        let mut rows = vec![Row { key: OVERSCAN, label: crop.label.clone(), value: crop.words(self.custom), steps: true, opens: false, sentence: "" }];
        if crop.is_custom(self.custom) {
            let mut own = crop.own.unwrap_or_default();
            for e in crate::display::EDGES {
                let value = crate::display::edge_words(*e.of(&mut own) * 100.0);
                rows.push(Row { key: e.key, label: e.label.into(), value, steps: true, opens: false, sentence: e.hint });
            }
        }
        rows
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
        let mut rows = vec![row(SCRAPE, "Scrape This Game", scrape, false), row(EDIT, "Edit Details", "", true)];
        if !self.read.covers.is_empty() {
            let in_use = self.read.covers.iter().find(|c| c.choice == self.read.cover_in_use).map(|c| c.text.as_str()).unwrap_or_default();
            rows.push(row(COVER, COVER_ART, in_use, true));
        } else if self.read.covers_unread.is_some() {
            // The row stays where its pictures could not be looked for, and A says why.
            rows.push(row(COVER, COVER_ART, "", false));
        }
        rows.extend([row(FOLDER, "Open Folder", "", false), row(HIDE, "Hide from Library", "…", false)]);
        rows
    }

    /// The way back above Cover art's title: the game, then Details.
    pub fn cover_trail(&self) -> String {
        format!("{} / {}", self.read.title, Section::Details.name())
    }

    /// Cover art's cards as offered: the source's, and where no cover was found for the game and a choice is in use, a
    /// card in the found cover's place whose A clears the choice (R-20261114-28).
    fn cover_cards(&self) -> Vec<CoverCard> {
        let mut cards = self.read.covers.clone();
        let found = cards.iter().any(|c| c.choice == CoverChoice::Found);
        if !found && self.read.cover_in_use != CoverChoice::Found {
            let at = cards.iter().take_while(|c| matches!(c.choice, CoverChoice::Borrowed(_))).count();
            cards.insert(at, CoverCard { choice: CoverChoice::Found, text: COVER_NONE.into(), picture: String::new() });
        }
        cards
    }

    /// Cover art's cards, the one in use marked and its mark's words its line.
    fn cover_tiles(&self) -> Vec<Tile> {
        let tile = |(i, c): (usize, &CoverCard)| Tile {
            key: format!("{COVER_CARD}.{i}"),
            label: c.text.clone(),
            line: if c.choice == self.read.cover_in_use { SHOWN.into() } else { String::new() },
            picture: c.picture.clone(),
            marked: c.choice == self.read.cover_in_use,
        };
        self.cover_cards().iter().enumerate().map(tile).collect()
    }

    /// A on Cover art's card: the cover found (or none, where none was found) or one of the game's own pictures made the
    /// cover; nothing for the card in use.
    fn choose_cover(&self, index: usize) -> Outcome {
        match self.cover_cards().get(index).map(|c| &c.choice) {
            Some(c) if *c == self.read.cover_in_use => Outcome::Nothing,
            Some(CoverChoice::Found) => Outcome::Changed(Change::Cover(None)),
            Some(CoverChoice::Own(kind)) => Outcome::Changed(Change::Cover(Some(kind.clone()))),
            Some(CoverChoice::Borrowed(_)) | None => Outcome::Nothing,
        }
    }

    /// The media type of the picture the viewer shows where it can be the game's cover and is not already.
    pub fn viewer_cover(&self, viewer: &emusen_esde::scene::Viewer) -> Option<String> {
        if viewer.reading.is_some() {
            return None;
        }
        let shown: Vec<&Media> = self.read.media.iter().filter(|m| m.kind != MediaKind::Manual).collect();
        let media = shown.get(viewer.index).filter(|m| m.kind != MediaKind::Taken)?;
        let kind = kind_word(media.kind);
        let own = CoverChoice::Own(kind.to_owned());
        let offered = self.read.covers.iter().any(|c| c.choice == own);
        (offered && self.read.cover_in_use != own).then(|| kind.to_owned())
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
            Some(row) if row.key == COVER => match &self.read.covers_unread {
                Some(why) => Outcome::Said(format!("{COVER_UNREAD}: {why}")),
                None => Outcome::CoverArt,
            },
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
            // Any choice ends a borrowed cover, so its card goes.
            Change::Cover(v) => {
                r.cover_in_use = v.clone().map_or(CoverChoice::Found, CoverChoice::Own);
                r.covers.retain(|c| !matches!(c.choice, CoverChoice::Borrowed(_)));
            }
            Change::DeleteState(_) | Change::Hide => {}
        }
    }

    /// Left or Right on a row: the engine, the profile or the standard stepped, wrapping, and saved.
    pub fn step_row(&mut self, index: usize, by: i32) -> Outcome {
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
                Some(c) => {
                    let (next, custom) = c.stepped(by, self.custom);
                    self.custom = custom;
                    Outcome::Changed(Change::Crop(next))
                }
                None => Outcome::Nothing,
            },
            Some(key) if self.cropping => match (&self.read.crop, crate::display::Edge::by_key(key)) {
                (Some(c), Some(edge)) => Outcome::Changed(Change::Crop(Some(c.edge_stepped(edge, by)))),
                _ => Outcome::Nothing,
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
            Some((label, _)) => format!("{label} {BETWEEN} {}", place(viewer.index, labels.len())),
            None => String::new(),
        }
    }

    /// A on a tile: a state loads, a picture opens the viewer.
    pub fn choose_tile(&self, index: usize) -> Outcome {
        if self.covering {
            return self.choose_cover(index);
        }
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
            Content::Tiles(_) if self.covering => Screen::GamePageCoverArt,
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

/// Favourite's value, the heart full for a favourite, else empty.
pub const HEART_ON: &str = "♥";
pub const HEART_OFF: &str = "♡";
/// Cheats' words, its row's on Options and its tab on Eclipse's page.
pub const CHEATS_WORD: &str = "Cheats";
/// A game with no last play, as the facts and Eclipse's line say it (EclipseGameNoState:45).
pub const NEVER_PLAYED: &str = "Never played";
/// Eclipse's empty card, the slot a state could be saved to (EclipseGame:82, R-20261116-36).
pub const EMPTY_SLOT: &str = "Empty slot";
/// The words beside the empty card where the game has no state (EclipseGameNoState:69).
pub const NO_STATES: &str = "No states yet. A state is saved from the Pause Card while the game runs.";
/// The badge on the card of the state Resume loads, drawn in capitals (EclipseGame:69).
pub const RESUMES_HERE: &str = "Resumes here";
/// The empty card's key: no state's file, and no content row behind it.
pub const EMPTY_CARD: &str = "empty";
/// The words shown on Eclipse's Media tab when the game has no media.
pub const NO_MEDIA: &str = "No media yet. Details ▸ Scrape This Game fetches pictures, videos and manuals.";
/// Cheats tab's aside sentence (EclipseGameCheats:79).
pub const CHEATS_SENTENCE: &str = "The game's cheats, on or off; the same switches the Pause Card shows.";
/// Package inner page's heading and subtitle (EclipseGamePackage:38-39).
pub const PACKAGE_TITLE: &str = "Package";
pub const PACKAGE_SUBTITLE: &str = "The game's package and its layers, from Packages.";

/// A deeper page of the game page that a theme may draw (`ThemeScreen.pageSubpages`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Subpage {
    None,
    Package,
}

impl Subpage {
    pub fn flag(self) -> i32 {
        match self {
            Subpage::None => 0,
            Subpage::Package => 1 << 1,
        }
    }
}

/// The page's tabs as Eclipse's board draws them (EclipseGame:55-62, R-20261116-29).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tab {
    Overview,
    States,
    Media,
    Details,
    Options,
    Cheats,
}

/// The board's six, in its order.
pub const TABS: [Tab; 6] = [Tab::Overview, Tab::States, Tab::Media, Tab::Details, Tab::Options, Tab::Cheats];

impl Tab {
    /// The tab's flag, by which a theme's screen names the tabs it draws: one bit at its place in `TABS`, as theme-qt's
    /// `Page.PageSection` numbers it (EmuSen_Qt.md §3.2, `ThemeScreen.pageSections`).
    pub fn flag(self) -> i32 {
        1 << TABS.iter().position(|t| *t == self).unwrap_or(0)
    }

    pub fn name(self) -> &'static str {
        if self == Tab::Cheats { CHEATS_WORD } else { self.section().name() }
    }

    /// The program's section the tab opens; Cheats is Options' deeper page.
    pub fn section(self) -> Section {
        match self {
            Tab::Overview => Section::Overview,
            Tab::States => Section::States,
            Tab::Media => Section::Media,
            Tab::Details => Section::Details,
            Tab::Options | Tab::Cheats => Section::Options,
        }
    }
}

/// One of Eclipse's state cards (EclipseGame:64-83): a state, or the one empty slot, which is not pressed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Card {
    pub key: String,
    pub label: String,
    pub line: String,
    pub picture: String,
    /// The state Resume would load, and the badge's words on it, `RESUMES_HERE`.
    pub marked: bool,
    pub badge: String,
    /// The state's tile among the States section's content, which the page's calls take; none for the empty card.
    pub row: Option<usize>,
}

/// A row of Eclipse's Details or Options tab, carrying its aside hint and underlying content index.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoardRow {
    pub key: String,
    pub label: String,
    pub value: String,
    pub steps: bool,
    pub opens: bool,
    pub is_switch: bool,
    pub switch_on: bool,
    pub sentence: &'static str,
    pub row: usize,
}

/// The plain core name without default or console suffixes (R-20261116-14).
fn plain_core(text: &str) -> String {
    for suffix in [" (default)", " (the console's)", " (the only one)"] {
        if let Some(name) = text.strip_suffix(suffix) {
            return name.to_owned();
        }
    }
    text.to_owned()
}

/// The aside hint for a Details row, as board #79 and the owner set it.
fn detail_sentence(key: &str) -> &'static str {
    match key {
        SCRAPE => "Fetch art and facts for this game from your scraping sources.",
        EDIT => "The game's name, dates, people and counts, as the scraper found them or as typed here.",
        COVER => "Which picture stands as this game's cover in the library.",
        FOLDER => "Opens the folder holding this game's file in the file manager.",
        HIDE => "Hides this game from the library without touching its file.",
        _ => "",
    }
}

/// A day of the calendar in the player's zone, as the when-lines count days: year, month (1 to 12) and day.
pub type Day = (i32, u32, u32);

/// The months as Eclipse's when-lines write them (EclipseGame:110, "8 November").
const MONTHS_LONG: [&str; 12] = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];

impl GamePage {
    /// The tabs shown: the board's six, Cheats only for a game with cheats, as its row on Options is (R-20261116-29).
    pub fn tabs(&self) -> Vec<Tab> {
        TABS.into_iter().filter(|t| *t != Tab::Cheats || !self.read.cheats.is_empty()).collect()
    }

    /// The tab shown, by its place among `tabs`: Cheats while Options shows Cheats, else the section's.
    pub fn tab(&self) -> usize {
        let shown = if self.cheating { Tab::Cheats } else { TABS.into_iter().find(|t| t.section() == self.shown()).unwrap_or(Tab::Overview) };
        self.tabs().iter().position(|t| *t == shown).unwrap_or(0)
    }

    /// The tab of that place chosen: its section, Cheats as Options' Cheats opened by its door (`open_at`); whether there was one.
    pub fn choose_tab(&mut self, index: usize) -> bool {
        let Some(tab) = self.tabs().get(index).copied() else { return false };
        let section = SECTIONS.iter().position(|s| *s == tab.section()).unwrap_or(0);
        self.choose_section(section);
        if tab == Tab::Cheats {
            self.open_at(CHEATS);
        }
        true
    }

    /// The tab a theme would draw now; none for a deeper page, which stays the program's (EmuSen_Qt.md §3.2).
    pub fn tab_drawn(&self) -> Option<Tab> {
        let deeper = self.editing || self.cropping || self.packaging || self.covering;
        (!deeper).then(|| self.tabs()[self.tab()])
    }

    /// Whether a theme's game screen whose `pageSections` flags are `drawn` draws the page as it stands: every tab where it
    /// names none (0), else the tab shown where its flag is set, never a deeper page or a question (EmuSen_Qt.md §3.2).
    pub fn drawn_by(&self, drawn: i32, asking: bool) -> bool {
        drawn == 0 || (!asking && self.tab_drawn().is_some_and(|t| drawn & t.flag() != 0))
    }

    /// The deeper page open if one is, else `Subpage::None`.
    pub fn subpage(&self) -> Subpage {
        if self.packaging { Subpage::Package } else { Subpage::None }
    }

    /// Whether a theme whose `pageSections` is `drawn` and `pageSubpages` is `subpages` draws the page as it stands.
    pub fn drawn_by_subpages(&self, drawn: i32, subpages: i32, asking: bool) -> bool {
        if drawn == 0 {
            return true;
        }
        if asking {
            return false;
        }
        if let Some(tab) = self.tab_drawn() {
            return drawn & tab.flag() != 0;
        }
        let sub = self.subpage();
        sub != Subpage::None && (subpages & sub.flag() != 0)
    }

    /// The board's line over the title (EclipseGame:43): console · year · genre, each the game has.
    pub fn kicker(&self) -> String {
        let r = &self.read;
        let parts: Vec<&str> = [r.console.as_str(), r.year.as_str(), self.detail("genre")].into_iter().filter(|p| !p.is_empty()).collect();
        parts.join(&format!(" {BETWEEN} "))
    }

    /// The board's line under the title (EclipseGame:45, EclipseGameNoState:45): when it was last played and the time played
    /// as Eclipse's Home says them, or Never played, then its players.
    pub fn line(&self) -> String {
        let r = &self.read;
        let played = match r.last_played {
            Some(at) => crate::words::played_facts(at, self.now, r.played_seconds),
            None => NEVER_PLAYED.to_owned(),
        };
        let players = crate::words::players(self.detail("players"));
        [played, players].into_iter().filter(|p| !p.is_empty()).collect::<Vec<_>>().join(&format!(" {BETWEEN} "))
    }

    fn detail(&self, field: &str) -> &str {
        self.read.details.iter().find(|d| d.field == field).map_or("", |d| d.value.as_str())
    }

    /// The board's buttons (EclipseGame:47-50, EclipseGameNoState:47-49; R-20261116-30): Resume, filled with A, Start from the
    /// beginning and Favourite where the game has a resume point, else Start, filled, and Favourite (R-20261123-36); Resume
    /// loads the resume point and is never stepped (R-20261123-35), the others act as the program's action of their key.
    pub fn pills(&self) -> Vec<Action> {
        let resumes = self.resume_point().is_some();
        let pill = |key: &'static str, label: &'static str, button: i32, filled: bool| Action { key, label, value: String::new(), button, filled, stepped: false, step_left: false, step_right: false };
        let mut out = if resumes {
            vec![pill(RESUME, "Resume", SOUTH, true), pill(PLAY, "Start from the beginning", -1, false)]
        } else {
            vec![pill(PLAY, "Start", SOUTH, true)]
        };
        out.push(Action { value: if self.read.favourite { HEART_ON } else { HEART_OFF }.into(), ..pill(FAVOURITE, "Favourite", -1, false) });
        out
    }

    /// The board's buttons, each with the place among `actions` of the action it is, which a theme's `pageAct` takes, and for
    /// Resume the resume point's tile it chooses (R-20261123-35), else -1.
    pub fn placed_pills(&self) -> Vec<(Action, i32, i32)> {
        let actions = self.actions();
        let point = self.resume_point_row().map_or(-1, |r| r as i32);
        self.pills()
            .into_iter()
            .map(|p| {
                let at = actions.iter().position(|a| a.key == p.key).map_or(-1, |i| i as i32);
                let row = if p.key == RESUME { point } else { -1 };
                (p, at, row)
            })
            .collect()
    }

    /// The state cards with their when-lines in the machine's zone and clock form at the page's moment.
    pub fn cards_now(&self) -> Vec<Card> {
        let now = self.now;
        let when = |at: i64| state_when(crate::clock::local_day(at, 0), crate::clock::local_day(now, 0), crate::clock::local_day(now, 1), &crate::clock::short_time(at));
        self.state_cards(&when)
    }

    /// The resume point, the state kept where the player left off, which Eclipse's Resume loads (R-20261123-35).
    pub fn resume_point(&self) -> Option<&State> {
        self.read.states.iter().find(|s| s.slot == Slot::Auto)
    }

    /// The resume point's tile among the States section's content, which Resume chooses.
    pub fn resume_point_row(&self) -> Option<usize> {
        let point = self.resume_point()?;
        self.state_tiles().iter().position(|t| t.key == point.path)
    }

    /// The board's trail (EclipseGame:29-35): the Consoles section, the game's console, its title, and the deeper page's name.
    pub fn trail(&self) -> Vec<String> {
        let mut out: Vec<String> = [crate::words::CONSOLES_SECTION, self.read.console.as_str(), self.read.title.as_str()]
            .into_iter()
            .filter(|w| !w.is_empty())
            .map(str::to_owned)
            .collect();
        if self.packaging {
            out.push(PACKAGE_TITLE.to_owned());
        }
        out
    }

    /// The state cards in the board's order and words (EclipseGame:64-83, R-20261116-36): the resume point, each slot saved in
    /// its number's order, then one empty slot while a slot is free (R-20261123-33); the resume point alone carries the badge
    /// (R-20261123-35); `when` writes a state's line from its moment.
    pub fn state_cards(&self, when: &dyn Fn(i64) -> String) -> Vec<Card> {
        let tiles = self.state_tiles();
        let row = |s: &State| tiles.iter().position(|t| t.key == s.path);
        let card = |s: &State, label: String| {
            let marked = s.slot == Slot::Auto;
            Card { key: s.path.clone(), label, line: when(s.when), picture: s.picture.clone(), marked, badge: if marked { RESUMES_HERE.into() } else { String::new() }, row: row(s) }
        };
        let mut out: Vec<Card> = self.read.states.iter().filter(|s| s.slot == Slot::Auto).map(|s| card(s, crate::words::RESUME_POINT.into())).collect();
        let mut slots: Vec<&State> = self.read.states.iter().filter(|s| matches!(s.slot, Slot::Number(_))).collect();
        slots.sort_by_key(|s| match s.slot {
            Slot::Number(n) => n,
            Slot::Auto => 0,
        });
        out.extend(slots.into_iter().map(|s| card(s, slot_name(s.slot))));
        if (1..=crate::pause::SLOTS).any(|n| !self.read.states.iter().any(|s| s.slot == Slot::Number(n))) {
            out.push(Card { key: EMPTY_CARD.into(), label: EMPTY_SLOT.into(), line: String::new(), picture: String::new(), marked: false, badge: String::new(), row: None });
        }
        out
    }

    /// The words beside the cards: the board's where the game has no state, else none.
    pub fn cards_note(&self) -> &'static str {
        if self.read.states.is_empty() { NO_STATES } else { "" }
    }

    /// Whether the game has its own description rather than the empty fallback.
    pub fn has_description(&self) -> bool {
        !self.read.description.is_empty()
    }

    /// The words shown on Eclipse's Media tab when the game has no media.
    pub fn media_note(&self) -> &'static str {
        if self.read.media.is_empty() { NO_MEDIA } else { "" }
    }

    /// The media type of the Media tile at `index` where it can be the game's cover and is not already.
    pub fn media_cover(&self, index: usize) -> Option<String> {
        let media = self.read.media.get(index).filter(|m| !matches!(m.kind, MediaKind::Taken | MediaKind::Clip | MediaKind::Manual))?;
        let kind = kind_word(media.kind);
        let own = CoverChoice::Own(kind.to_owned());
        let offered = self.read.covers.iter().any(|c| c.choice == own);
        (offered && self.read.cover_in_use != own).then(|| kind.to_owned())
    }

    /// Eclipse's Details, Options, Cheats or Package rows with board wording, aside hints, and content indices.
    pub fn board_rows(&self) -> Vec<BoardRow> {
        if self.packaging {
            return self
                .package_rows()
                .into_iter()
                .enumerate()
                .map(|(row, r)| {
                    let value = if r.key == JOB && r.value == "…" { String::new() } else { r.value };
                    BoardRow {
                        key: r.key.to_owned(),
                        label: r.label,
                        value,
                        steps: false,
                        opens: false,
                        is_switch: false,
                        switch_on: false,
                        sentence: "",
                        row,
                    }
                })
                .collect();
        }
        let Some(tab) = self.tab_drawn() else { return Vec::new() };
        let Content::Rows(rows) = self.content() else { return Vec::new() };
        match tab {
            Tab::Details => rows
                .into_iter()
                .enumerate()
                .map(|(row, r)| {
                    let value = if (r.key == SCRAPE || r.key == HIDE) && r.value == "…" { String::new() } else { r.value };
                    BoardRow {
                        key: r.key.to_owned(),
                        label: r.label,
                        value,
                        steps: r.steps,
                        opens: r.opens,
                        is_switch: false,
                        switch_on: false,
                        sentence: detail_sentence(r.key),
                        row,
                    }
                })
                .collect(),
            Tab::Options => rows
                .into_iter()
                .enumerate()
                .filter(|(_, r)| r.key != CHEATS)
                .map(|(row, r)| {
                    let (label, value, sentence) = match r.key {
                        "engine" => (
                            "Core".to_owned(),
                            plain_core(&r.value),
                            "The core this game runs on; the console's own unless one is chosen here. Cores lists them all.",
                        ),
                        "profile" => (r.label, if self.read.profile.is_none() { CONSOLES.to_owned() } else { r.value }, r.sentence),
                        STANDARD => (r.label, if self.read.standard.is_none() { "The game's own".to_owned() } else { r.value }, r.sentence),
                        _ => (r.label, r.value, r.sentence),
                    };
                    BoardRow { key: r.key.to_owned(), label, value, steps: r.steps, opens: r.opens, is_switch: false, switch_on: false, sentence, row }
                })
                .collect(),
            Tab::Cheats => rows
                .into_iter()
                .enumerate()
                .map(|(row, r)| {
                    let on = self.read.cheats.get(row).is_some_and(|c| c.on);
                    BoardRow {
                        key: format!("{CHEAT}.{row}"),
                        label: r.label,
                        value: r.value,
                        steps: false,
                        opens: false,
                        is_switch: true,
                        switch_on: on,
                        sentence: CHEATS_SENTENCE,
                        row,
                    }
                })
                .collect(),
            _ => Vec::new(),
        }
    }

    /// The focused board item's aside sentence and position ("2 of 5", "2 of 12 · 3 of 12 on"); empty on pills.
    pub fn board_aside(&self, action: Option<usize>, content: usize) -> (String, String) {
        if action.is_some() {
            return (String::new(), String::new());
        }
        if self.packaging {
            let rows = self.board_rows();
            let Some(at) = rows.iter().position(|r| r.row == content) else { return (String::new(), String::new()) };
            return (String::new(), place(at, rows.len()));
        }
        if self.tab_drawn() == Some(Tab::Media) {
            return if content < self.read.media.len() {
                (String::new(), place(content, self.read.media.len()))
            } else {
                (String::new(), String::new())
            };
        }
        let rows = self.board_rows();
        let Some(at) = rows.iter().position(|r| r.row == content) else { return (String::new(), String::new()) };
        if self.tab_drawn() == Some(Tab::Cheats) {
            let count = format!("{} {BETWEEN} {}", place(at, rows.len()), crate::cheats::count_words(&self.read.cheats));
            return (rows[at].sentence.to_owned(), count);
        }
        (rows[at].sentence.to_owned(), place(at, rows.len()))
    }

    /// X on an Options row: resets the game's override back to the console's default where one is set.
    pub fn to_default(&self, index: usize) -> Outcome {
        if self.tab_drawn() != Some(Tab::Options) {
            return Outcome::Nothing;
        }
        let Content::Rows(rows) = self.content() else { return Outcome::Nothing };
        match rows.get(index).map(|r| r.key) {
            Some("engine") if self.read.engine.is_some() => Outcome::Changed(Change::Engine(None)),
            Some("profile") if self.read.profile.is_some() => Outcome::Changed(Change::Profile(None)),
            Some(STANDARD) if self.read.standard.is_some() => Outcome::Changed(Change::Standard(None)),
            Some(CROP) if self.read.crop.as_ref().is_some_and(|c| c.own.is_some()) => Outcome::Changed(Change::Crop(None)),
            _ => Outcome::Nothing,
        }
    }

    /// Whether X on the Options row at `index` would reset an override back to the console's default.
    pub fn defaultable(&self, index: usize) -> bool {
        !matches!(self.to_default(index), Outcome::Nothing)
    }

    /// The help as Eclipse's boards word it for the focus (R-20261116-30); none where the program's own is shown.
    pub fn board_help(&self, action: Option<usize>, content: usize) -> Option<Screen> {
        if self.packaging {
            let rows = self.package_rows();
            return match rows.get(content).map(|r| r.key) {
                Some(JOB) if self.job_running() => Some(Screen::GamePagePackageStop),
                Some(JOB) if self.read.package_job.as_ref().is_some_and(|j| j.key == MAKE) => Some(Screen::GamePagePackageMake),
                Some(JOB) => Some(Screen::GamePagePackageCheck),
                _ => Some(Screen::GamePagePackageInfo),
            };
        }
        let key = action.and_then(|i| self.actions().get(i).map(|a| a.key));
        match (action, key) {
            (Some(_), Some(PLAY)) if self.resume_point().is_none() => Some(Screen::GamePageStart),
            (Some(_), Some(RESUME)) => Some(Screen::GamePageResumeBoard),
            (Some(_), _) => None,
            (None, _) => match self.tab_drawn() {
                Some(Tab::Overview) => Some(if self.resume_point().is_some() { Screen::GamePageResumeBoard } else { Screen::GamePageStart }),
                Some(Tab::States) if self.read.states.is_empty() => Some(Screen::GamePageStart),
                Some(Tab::States) if self.help(None, content) == Screen::GamePageLoad => Some(Screen::GamePageState),
                Some(Tab::Media) if content < self.read.media.len() => Some(Screen::GamePageMedia),
                Some(Tab::Media) => Some(if self.resume_point().is_some() { Screen::GamePageResumeBoard } else { Screen::GamePageStart }),
                Some(Tab::Details) => match self.board_rows().iter().find(|r| r.row == content) {
                    Some(r) if r.opens => Some(Screen::GamePageOpen),
                    Some(_) => Some(Screen::GamePageDetail),
                    None => None,
                },
                Some(Tab::Options) => match self.board_rows().iter().find(|r| r.row == content) {
                    Some(r) if r.steps => Some(Screen::GamePageOption),
                    Some(r) if r.opens => Some(Screen::GamePageOpen),
                    _ => None,
                },
                Some(Tab::Cheats) => match self.read.cheats.get(content) {
                    Some(c) if c.on => Some(Screen::GamePageCheatOn),
                    Some(_) => Some(Screen::GamePageCheatOff),
                    None => None,
                },
                _ => None,
            },
        }
    }
}

/// A state's when-line as Eclipse's board writes it (EclipseGame:108-111, R-20261123-34): "Today, 9:12 PM", "Yesterday,
/// 9:12 PM", "8 November, 7:05 PM" for another day of the year, "8 November 2025, 7:05 PM" for another year, with `time`
/// the moment's clock time in the player's form.
pub fn state_when(day: Day, today: Day, yesterday: Day, time: &str) -> String {
    let month = MONTHS_LONG[(day.1.clamp(1, 12) - 1) as usize];
    match day {
        d if d == today => format!("Today, {time}"),
        d if d == yesterday => format!("Yesterday, {time}"),
        d if d.0 == today.0 => format!("{} {month}, {time}", d.2),
        d => format!("{} {month} {}, {time}", d.2, d.0),
    }
}
