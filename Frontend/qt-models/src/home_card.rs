//! The Home card's pages (EmuSen_Interface.md §2.3; EmuSen_Qt.md §7.1, stage 2): Mistress's `HomeRows`, `ToolRows` and
//! `PowerRows` in her words, made from the card's facts, and what each row does. On the big screen's own look the card
//! opens at Power from Home's place (EmuSen_BigPicture.md §51.2) and at its first page by Start on Home (EmuSen_Qt.md
//! §7.9); over a theme Start opens it at its first page.

use crate::card::{end_section_before, question, Choice, Page, Pages, Pane, Row};
use serde_json::{json, Value};

pub const CARD: &str = "home";

pub const FIRST: &str = "home";
pub const TOOLS: &str = "tools";
pub const POWER: &str = "power";
pub const QUIT: &str = "quit";
/// Start Console's page, a row a console that starts empty (Mistress's `StartConsolePage`).
pub const START_CONSOLE: &str = "startConsole";

/// The actions the host carries out, by key.
pub const RESUME: &str = "resume";
pub const GAME_PAGE: &str = "gamePage";
pub const SEARCH: &str = "search";
pub const SETTINGS: &str = "settings";
pub const BIG_PICTURE: &str = "bigPicture";
pub const QUIT_EMUSEN: &str = "quitEmuSen";
/// The tools whose screens the program has, by their rows' keys: Cores, and Themes (the Themes tab).
pub const CORES: &str = "cores";
pub const THEMES: &str = "themes";
pub const SCRAPER: &str = "scraper";

/// A console started empty, as the action the host carries out: `startConsole:<console>`.
pub fn start_console_action(console: &str) -> String {
    format!("{START_CONSOLE}:{console}")
}

/// The console an action starts empty; none for every other action.
pub fn console_to_start(action: &str) -> Option<&str> {
    action.strip_prefix(START_CONSOLE)?.strip_prefix(':').filter(|c| !c.is_empty())
}

/// A console Start Console offers: its key, its name and the engine it starts on.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Startable {
    pub console: String,
    pub name: String,
    pub engine: String,
    /// The firmware line its pane ends with (`library::firmware_line`); empty where it needs none.
    #[serde(default)]
    pub firmware: String,
    /// The system it is listed by, its first.
    #[serde(default)]
    pub system: String,
}

const TITLE: &str = "Home";

/// What the card's words are made from, as the host knows them when it opens.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Facts {
    /// The suspended game's name; empty with none.
    pub suspended: String,
    /// Its console and how long ago it was paused, the pane's caption ("Super Nintendo · Paused just now").
    pub paused: String,
    /// The game under the selection, its name; empty with none.
    pub in_hand: String,
    /// Its console, its cover's URL, and whether it is a favourite.
    pub in_hand_console: String,
    pub in_hand_cover: String,
    pub in_hand_favourite: bool,
    /// Whether the view shown has a search.
    pub search: bool,
    /// The consoles that start with no disc in it, in Start Console's order: the row is offered where there is one.
    pub startable: Vec<Startable>,
    /// Steam's Game Mode, where Power holds Quit alone (Mistress's `_bigScreenForced`).
    pub game_mode: bool,
    /// The tools' summaries by key (`scraper`, `packages`, `library`, `cores`, `cheats`): `brief`, `sentence`, `attention`.
    pub tools: Value,
    /// Words under the first page's rows where the consoles or the tools' summaries could not be read.
    pub unread: String,
}

/// The consoles that start empty and the tools' summaries from the library's words for them, and the words said where
/// either cannot be read, as the List card says a console it cannot read (EmuSen_Qt.md §7.9). An empty text is none, as
/// the library holds before it has read either, not a text it could not read.
pub fn read_lists(startable: &str, tools: &str) -> (Vec<Startable>, Value, String) {
    let mut unread = Vec::new();
    let (startable, tools) = (if startable.is_empty() { "[]" } else { startable }, if tools.is_empty() { "null" } else { tools });
    let startable = serde_json::from_str(startable).unwrap_or_else(|why| {
        unread.push(format!("The consoles that start with no disc could not be read: {why}"));
        Vec::new()
    });
    let tools = serde_json::from_str(tools).unwrap_or_else(|why| {
        unread.push(format!("The tools' lines could not be read: {why}"));
        Value::Null
    });
    (startable, tools, unread.join(" "))
}

impl Facts {
    pub fn to_json(&self) -> String {
        json!({
            "suspended": self.suspended, "paused": self.paused, "inHand": self.in_hand, "inHandConsole": self.in_hand_console,
            "inHandCover": self.in_hand_cover, "inHandFavourite": self.in_hand_favourite, "search": self.search,
            "startable": self.startable, "gameMode": self.game_mode, "tools": self.tools, "unread": self.unread,
        })
        .to_string()
    }

    pub fn of(v: &Value) -> Facts {
        let words = |k: &str| v.get(k).and_then(Value::as_str).unwrap_or_default().to_owned();
        let flag = |k: &str| v.get(k).and_then(Value::as_bool).unwrap_or(false);
        Facts {
            suspended: words("suspended"),
            paused: words("paused"),
            in_hand: words("inHand"),
            in_hand_console: words("inHandConsole"),
            in_hand_cover: words("inHandCover"),
            in_hand_favourite: flag("inHandFavourite"),
            search: flag("search"),
            startable: v.get("startable").cloned().and_then(|s| serde_json::from_value(s).ok()).unwrap_or_default(),
            game_mode: flag("gameMode"),
            tools: v.get("tools").cloned().unwrap_or(Value::Null),
            unread: words("unread"),
        }
    }
}

/// The tools of the Tools page, each by its row's key, its name, the summary's key, and what it does until it has said
/// its state (Mistress's `ToolRows`, `HomeScreen.About` and `Description`).
const TOOL_ROWS: [(&str, &str, &str, &str); 5] = [
    (SCRAPER, "Scraper", "Art and facts", "Fetches covers, screenshots, videos and facts for the library from ScreenScraper."),
    ("packages", "Packages", "Game packages", "Makes, checks and zips game packages, and converts discs to CHD."),
    ("library", "Library Manager", "Checks dumps", "Finds duplicate and damaged dumps; it reads every file when it checks."),
    (CORES, "Cores", "The engines", "The engines each console runs on: EmuSen's own and libretro's."),
    ("cheats", "Cheats", "Codes for games", "Codes for games, switched on game by game."),
];

/// The tools after the five with summaries: each opens its own window, said by what A and B do.
const OTHER_TOOLS: [(&str, &str); 4] =
    [(THEMES, "Themes"), ("collections", "Collections"), ("compressed", "Compressed Discs"), ("shell", "Shell")];

/// Compressed Discs' sentence, as the tools with summaries have theirs.
const COMPRESSED_SENTENCE: &str = "Converts the library's disc games to CHD and checks the converted discs.";

/// A tool's name and what it does, by its row's key; none for no tool.
pub fn tool_words(key: &str) -> Option<(&'static str, &'static str)> {
    TOOL_ROWS.iter().find(|r| r.0 == key).map(|r| (r.1, r.3)).or_else(|| OTHER_TOOLS.iter().find(|r| r.0 == key).map(|r| (r.1, if r.0 == "compressed" { COMPRESSED_SENTENCE } else { "" })))
}

/// Whether a tool has a screen; one that has none yet is not listed (screens::tool_screen).
fn screened(key: &str) -> bool {
    crate::screens::tool_screen(key).is_some()
}

/// The order the Tools row's value is taken in: the first tool whose line needs the player (`AttentionValue`).
const ATTENTION_ORDER: [&str; 5] = ["cores", "library", "scraper", "packages", "cheats"];

fn summary<'a>(tools: &'a Value, key: &str, field: &str) -> Option<&'a str> {
    tools.get(key).and_then(|s| s.get(field)).and_then(Value::as_str).filter(|w| !w.is_empty())
}

fn needs_player(tools: &Value, key: &str) -> bool {
    tools.get(key).and_then(|s| s.get("attention")).and_then(Value::as_bool).unwrap_or(false)
}

fn hint(name: &str) -> String {
    format!("A opens {name}; B comes back here.")
}

/// The tools' lines as the Tools row's pane lists them: each name, and its line or what it does.
fn tool_lines(tools: &Value) -> Pane {
    let lines = TOOL_ROWS.iter().filter(|r| screened(r.0)).map(|(key, name, about, _)| (name.to_string(), summary(tools, key, "marked").or(summary(tools, key, "brief")).unwrap_or(about).to_owned()));
    Pane { lines: lines.collect(), ..Pane::default() }
}

fn first_page(f: &Facts) -> Page {
    let mut rows = Vec::new();
    if !f.suspended.is_empty() {
        rows.push(Row::new(RESUME, "Continue").value(&f.suspended).pane(Pane { caption: f.paused.clone(), ..Pane::default() }));
    }
    if !f.in_hand.is_empty() {
        let console = if f.in_hand_favourite { format!("{} · Favourite", f.in_hand_console) } else { f.in_hand_console.clone() };
        let pane = Pane {
            lines: vec![(f.in_hand.clone(), console), (String::new(), "A opens its page: states, options, media and details.".to_owned())],
            picture: f.in_hand_cover.clone(),
            caption: String::new(),
        };
        rows.push(Row::new(GAME_PAGE, "Selected game").value(&f.in_hand).opens().pane(pane));
    }
    if f.search {
        rows.push(Row::new(SEARCH, "Search"));
    }
    if !f.startable.is_empty() {
        rows.push(Row::new(START_CONSOLE, "Start Console").opens());
    }
    let tools_at = rows.len();
    let attention = ATTENTION_ORDER.iter().filter(|k| screened(k) && needs_player(&f.tools, k)).find_map(|k| summary(&f.tools, k, "markedBrief").or(summary(&f.tools, k, "brief")));
    rows.push(Row::new(TOOLS, "Tools").value(attention.unwrap_or_default()).opens().pane(tool_lines(&f.tools)));
    if tools_at > 0 {
        end_section_before(&mut rows, tools_at);
    }
    rows.push(Row::new(SETTINGS, "Settings").opens());
    let power_at = rows.len();
    rows.push(Row::new(POWER, "Power").opens());
    end_section_before(&mut rows, power_at);
    Page { title: TITLE.to_owned(), breadcrumb: String::new(), rows, footer: f.unread.clone() }
}

fn tools_page(f: &Facts) -> Page {
    let mut rows: Vec<Row> = TOOL_ROWS
        .iter()
        .filter(|r| screened(r.0))
        .map(|(key, name, _, sentence)| {
            let said = summary(&f.tools, key, "sentence").unwrap_or(sentence);
            Row::new(key, name).value(summary(&f.tools, key, "markedBrief").or(summary(&f.tools, key, "brief")).unwrap_or_default()).opens().pane(Pane {
                lines: vec![(name.to_string(), said.to_owned()), (String::new(), hint(name))],
                ..Pane::default()
            })
        })
        .collect();
    rows.extend(OTHER_TOOLS.iter().filter(|r| screened(r.0)).map(|(key, name)| Row::new(key, name).opens().pane(Pane::said(name, &hint(name)))));
    Page { title: "Tools".to_owned(), breadcrumb: TITLE.to_owned(), rows, footer: String::new() }
}

fn article(name: &str) -> &'static str {
    if name.chars().next().is_some_and(|c| "AEIOU".contains(c.to_ascii_uppercase())) { "an" } else { "a" }
}

/// Start Console: each console that starts empty by its name with its engine, in Mistress's words.
pub fn start_console_page(startable: &[Startable], breadcrumb: &str) -> Page {
    let rows = startable
        .iter()
        .map(|s| {
            let said = format!("Starts {} {} with no disc in it, on {}.", article(&s.name), s.name, s.engine);
            let mut pane = Pane::said(&said, "Its BIOS screen comes up; Change Disc on the Pause Card puts a game in.");
            if !s.firmware.is_empty() {
                pane.lines.push((s.firmware.clone(), String::new()));
            }
            Row::new(&start_console_action(&s.console), &s.name).value(&s.engine).pane(pane)
        })
        .collect();
    Page { title: "Start Console".to_owned(), breadcrumb: breadcrumb.to_owned(), rows, footer: String::new() }
}

fn power_page(f: &Facts) -> Page {
    let mut rows = Vec::new();
    if !f.game_mode {
        rows.push(Row::new(BIG_PICTURE, "Exit Big Picture"));
    }
    rows.push(Row::new(QUIT, "Quit EmuSen..."));
    Page { title: "Power".to_owned(), breadcrumb: TITLE.to_owned(), rows, footer: crate::words::version_line() }
}

/// The card's pages.
pub struct HomeCard;

impl Pages for HomeCard {
    fn first(&self) -> &'static str {
        FIRST
    }

    fn page(&self, key: &str, facts: &Value) -> Page {
        let f = Facts::of(facts);
        match key {
            TOOLS => tools_page(&f),
            POWER => power_page(&f),
            START_CONSOLE => start_console_page(&f.startable, TITLE),
            QUIT => question("Quit EmuSen?", if f.suspended.is_empty() { "EmuSen closes." } else { "The game closes, then EmuSen." }),
            _ => first_page(&f),
        }
    }

    fn choose(&self, page: &str, row: &str) -> Choice {
        match (page, row) {
            (QUIT, crate::card::YES) => Choice::Act(QUIT_EMUSEN.to_owned()),
            (QUIT, _) => Choice::Back,
            (_, TOOLS | POWER | QUIT | START_CONSOLE) => Choice::Turn(row.to_owned()),
            (START_CONSOLE, console) => Choice::Act(console.to_owned()),
            (_, RESUME | GAME_PAGE | SEARCH | SETTINGS | BIG_PICTURE) => Choice::Act(row.to_owned()),
            (TOOLS, tool) => Choice::Act(tool.to_owned()),
            _ => Choice::Owed,
        }
    }

    /// Start Console's page names A "Start" (Mistress's `AcceptWord`).
    fn help(&self, page: &str) -> crate::help::Screen {
        if page == START_CONSOLE { crate::help::Screen::CardStart } else { crate::help::Screen::Card }
    }
}
