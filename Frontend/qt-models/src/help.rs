//! The help bar's entries for each screen, as the C# screens and the mockups name them (EmuSen_Interface.md §24.1-§24.3;
//! Home's by the row focused, as `HomeScreen.HintsFor` gives them):
//! the button by what it asks of the interface, its label, and the action it invokes where the walk does not answer it
//! itself. The A/B swap is the router's job, not this list's.

use emusen_kit_core::input::UiButton;

/// A screen with a help bar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    /// Home with Continue focused.
    Home,
    /// Home with a game of the shelf focused.
    HomeShelf,
    /// Home with a place focused.
    HomePlaces,
    Consoles,
    Grid,
    /// The grid with words searched for: B clears them (`GamesScreen.HintsFor` with `searched`).
    GridSearched,
    /// The Pause Card's first page.
    Pause,
    /// A page of the Pause Card past the first.
    PausePage,
    /// A card over the library: the Home card and the List card (Mistress's `ShowCard` away from a game).
    Card,
    /// Start Console's page of a card: A starts the console (Mistress's `AcceptWord`, "Start" on that page).
    CardStart,
    /// The grid's search with the keyboard docked (`GamesScreen.HintsFor` in the keyboard).
    Search,
    /// The Pause Card's first page on Save State or Load State: Left and Right change the slot.
    PauseSlots,
    /// The Pause Card's reel (EmuSen_Interface.md §20.1): A goes back, Left and Right step the moment, Start is not offered.
    PauseReel,
    /// A question whose B answers No (EmuSen_Interface.md §15.6).
    Question,
    /// A question whose B goes back, the Resume question's (§18.3).
    QuestionBack,
    /// The Pause Card's Change Disc (EmuSen_Interface.md §20.2): A puts the disc in, Start is not offered.
    PauseDisc,
    /// The Themes tab on a theme Use is offered for (EmuSen_Qt.md §3.8).
    Themes,
    /// The Themes tab on a theme in use, not checked clean, or of a kind not chosen here: Check alone.
    ThemesChecked,
    /// The Themes tab on a choice of the ES-DE theme in use: A and Left and Right step it.
    ThemesChoice,
    /// Settings on a section (EmuSen_BigPicture.md §52): the shoulders step the sections.
    Settings,
    /// Settings on a page under a section: the shoulders are the page's.
    SettingsPage,
    /// A tool's search with the keyboard docked: Type, Delete, Space and Done (EmuSen_Interface.md §15.2).
    ToolSearch,
    /// A console's page under Consoles: the shoulders step to the console before or after it.
    SettingsConsole,
    /// A page of choices (EmuSen_Interface.md §18.11): A chooses, B keeps what was chosen.
    SettingsChoices,
    /// Controllers' bindings page (EmuSen_Input.md §10.6): A rebinds, Left and Right the regions, Y resets, the shoulders
    /// the console, X the player where the console has more than one, Select the tester, B back.
    SettingsBindings,
    /// The bindings page of a console of one player: no X.
    SettingsBindingsOne,
    /// The keyboard's page: X clears, no tester.
    SettingsKeys,
    /// A binding or a hotkey listening for a press: no hints, as Mistress shows none.
    SettingsListening,
    /// The tester: B held stops it.
    SettingsTesting,
    /// Controllers ▸ Hotkeys: A listens, X clears.
    SettingsHotkeys,
    /// A game's page (EmuSen_Qt.md §7.8), by what is focused: Play, Resume (with its slots where it steps them),
    /// Favourite; a state, a picture, a stepped row, a door, a row of Details, or nothing A acts on.
    GamePage,
    GamePageResume,
    GamePageResumeSlots,
    GamePageFavourite,
    GamePageLoad,
    GamePageView,
    GamePageValue,
    GamePageChange,
    GamePageChoose,
    GamePageNone,
    /// Edit Details' rows: A types the detail, Y puts back what was scraped.
    GamePageEdit,
    /// The media viewer: Left and Right browse, Up opens the manual where the game has one.
    GamePageViewer,
    GamePageViewerManual,
    /// The viewer's manual: Left and Right turn the page, the shoulders zoom, Down goes back to the media.
    GamePageManual,
    /// The media viewer on a picture that can be the game's cover: X sets it (EmuSen_Interface.md §18.2).
    GamePageViewerCover,
    GamePageViewerManualCover,
    /// Details' Cover art: A uses the focused card, B goes back to Details (EmuSen_Interface.md §18.2).
    GamePageCoverArt,
    /// Eclipse's page in its boards' words (R-20261116-30): a state's card, Start on a game with no state, and the shoulders'
    /// Section at the line's right (EclipseGame:86-96, EclipseGameNoState:73-84).
    GamePageState,
    GamePageStart,
    GamePageSections,
    /// Eclipse's boards for Overview (Resume), Details/Options doors, Details actions and Options stepped rows (R-20261116-30).
    GamePageResumeBoard,
    GamePageOpen,
    GamePageDetail,
    GamePageOption,
}

/// One entry: Accept and Back are the walk's own, so they name no action.
pub type Entry = (UiButton, &'static str, Option<&'static str>);

/// The entries, in the bar's order.
pub fn entries(screen: Screen) -> &'static [Entry] {
    match screen {
        Screen::Home => &[
            (UiButton::Accept, "Resume", None),
            (UiButton::Screensaver, "Game page", Some(GAME_PAGE)),
            (UiButton::Search, "Favourite", Some(FAVOURITE)),
            (UiButton::Options, "Search", Some("search")),
            (UiButton::Menu, "Home", Some(HOME)),
        ],
        Screen::HomeShelf => &[
            (UiButton::Accept, "Play", None),
            (UiButton::Screensaver, "Game page", Some(GAME_PAGE)),
            (UiButton::Search, "Favourite", Some(FAVOURITE)),
            (UiButton::Options, "Search", Some("search")),
            (UiButton::Menu, "Home", Some(HOME)),
        ],
        Screen::HomePlaces => &[
            (UiButton::Accept, "Open", None),
            (UiButton::Options, "Search", Some("search")),
            (UiButton::Menu, "Home", Some(HOME)),
        ],
        Screen::Consoles => &[
            (UiButton::Accept, "Open", None),
            (UiButton::Back, "Back", None),
            (UiButton::Search, "Favourites", Some(crate::library::FAVOURITES)),
            (UiButton::Screensaver, "Random game", Some(RANDOM)),
        ],
        Screen::Grid => &[
            (UiButton::Accept, "Play", None),
            (UiButton::Screensaver, "Game page", Some(GAME_PAGE)),
            (UiButton::Search, "Favourite", Some(FAVOURITE)),
            (UiButton::PageUp, "Console", Some(PREVIOUS)),
            (UiButton::Options, "List", Some("list")),
            (UiButton::Back, "Consoles", None),
            (UiButton::Menu, "Home", Some(HOME)),
        ],
        Screen::GridSearched => &[
            (UiButton::Accept, "Play", None),
            (UiButton::Screensaver, "Game page", Some(GAME_PAGE)),
            (UiButton::Search, "Favourite", Some(FAVOURITE)),
            (UiButton::PageUp, "Console", Some(PREVIOUS)),
            (UiButton::Options, "List", Some("list")),
            (UiButton::Back, "Clear search", None),
            (UiButton::Menu, "Home", Some(HOME)),
        ],
        // Mistress's ShowCard and SetCardActions: B resumes on the first page and goes back on the others; Start resumes.
        Screen::Pause => &[
            (UiButton::Accept, "Select", None),
            (UiButton::Back, "Resume", None),
            (UiButton::Screensaver, "Screenshot", Some(crate::pause::SCREENSHOT)),
            (UiButton::Menu, "Resume", Some(crate::pause::RESUME)),
        ],
        Screen::PausePage => &[
            (UiButton::Accept, "Choose", None),
            (UiButton::Back, "Back", None),
            (UiButton::Screensaver, "Screenshot", Some(crate::pause::SCREENSHOT)),
            (UiButton::Menu, "Resume", Some(crate::pause::RESUME)),
        ],
        Screen::Question => &[(UiButton::Accept, "Choose", None), (UiButton::Back, "No", None)],
        Screen::QuestionBack => &[(UiButton::Accept, "Choose", None), (UiButton::Back, "Back", None)],
        Screen::Card => &[(UiButton::Accept, "Choose", None), (UiButton::Back, "Back", None), (UiButton::Menu, "Close", Some(CLOSE))],
        Screen::CardStart => &[(UiButton::Accept, "Start", None), (UiButton::Back, "Back", None), (UiButton::Menu, "Close", Some(CLOSE))],
        Screen::Search => &[
            (UiButton::Accept, "Type", None),
            (UiButton::Back, "Delete", None),
            (UiButton::Search, "Space", Some("space")),
            (UiButton::Screensaver, "Done", Some("done")),
            (UiButton::PageUp, "Move to grid", Some(crate::screens::TO_GRID)),
            (UiButton::Menu, "Home", Some("home")),
        ],
        Screen::PauseSlots => &[
            (UiButton::Accept, "Select", None),
            (UiButton::Back, "Resume", None),
            (UiButton::Left, "Change slot", None),
            (UiButton::Screensaver, "Screenshot", Some(crate::pause::SCREENSHOT)),
            (UiButton::Menu, "Resume", Some(crate::pause::RESUME)),
        ],
        Screen::PauseReel => &[
            (UiButton::Accept, "Go back", None),
            (UiButton::Back, "Back", None),
            (UiButton::Left, "Moment", None),
            (UiButton::Screensaver, "Screenshot", Some(crate::pause::SCREENSHOT)),
        ],
        Screen::PauseDisc => &[
            (UiButton::Accept, "Put in", None),
            (UiButton::Back, "Back", None),
            (UiButton::Screensaver, "Screenshot", Some(crate::pause::SCREENSHOT)),
        ],
        Screen::Themes => &[
            (UiButton::Accept, "Use", None),
            (UiButton::Screensaver, "Check", Some(CHECK)),
            (UiButton::PageUp, "Kind", None),
            (UiButton::Back, "Back", None),
            (UiButton::Menu, "Home", Some(HOME)),
        ],
        Screen::ThemesChecked => &[
            (UiButton::Screensaver, "Check", Some(CHECK)),
            (UiButton::PageUp, "Kind", None),
            (UiButton::Back, "Back", None),
            (UiButton::Menu, "Home", Some(HOME)),
        ],
        Screen::ThemesChoice => &[
            (UiButton::Accept, "Change", None),
            (UiButton::Left, "Value", None),
            (UiButton::PageUp, "Kind", None),
            (UiButton::Back, "Back", None),
            (UiButton::Menu, "Home", Some(HOME)),
        ],
        // Mistress's SettingsView hints: Change, Value, Section, Back, Home.
        Screen::Settings => &[
            (UiButton::Accept, "Change", None),
            (UiButton::Left, "Value", None),
            (UiButton::PageUp, "Section", None),
            (UiButton::Back, "Back", None),
            (UiButton::Menu, "Home", Some(HOME)),
        ],
        Screen::SettingsPage => &[
            (UiButton::Accept, "Change", None),
            (UiButton::Left, "Value", None),
            (UiButton::Back, "Back", None),
            (UiButton::Menu, "Home", Some(HOME)),
        ],
        Screen::ToolSearch => &[
            (UiButton::Accept, "Type", None),
            (UiButton::Back, "Delete", None),
            (UiButton::Search, "Space", Some("space")),
            (UiButton::Screensaver, "Done", Some("done")),
            (UiButton::PageUp, "Move to list", Some(crate::screens::TO_GRID)),
        ],
        Screen::SettingsConsole => &[
            (UiButton::Accept, "Change", None),
            (UiButton::Left, "Value", None),
            (UiButton::PageUp, "Console", None),
            (UiButton::Back, "Back", None),
            (UiButton::Menu, "Home", Some(HOME)),
        ],
        Screen::SettingsChoices => &[(UiButton::Accept, "Choose", None), (UiButton::Back, "Back", None)],
        // Mistress's BindingHints: Rebind, Region, Reset, Console, Player, Test, Back.
        Screen::SettingsBindings => &[
            (UiButton::Accept, "Rebind", None),
            (UiButton::Left, "Region", None),
            (UiButton::Search, "Reset", Some(RESET)),
            (UiButton::PageUp, "Console", None),
            (UiButton::Screensaver, "Player", Some(PLAYER)),
            (UiButton::Options, "Test", Some(TEST)),
            (UiButton::Back, "Back", None),
        ],
        Screen::SettingsBindingsOne => &[
            (UiButton::Accept, "Rebind", None),
            (UiButton::Left, "Region", None),
            (UiButton::Search, "Reset", Some(RESET)),
            (UiButton::PageUp, "Console", None),
            (UiButton::Options, "Test", Some(TEST)),
            (UiButton::Back, "Back", None),
        ],
        Screen::SettingsKeys => &[
            (UiButton::Accept, "Rebind", None),
            (UiButton::Left, "Region", None),
            (UiButton::Search, "Reset", Some(RESET)),
            (UiButton::PageUp, "Console", None),
            (UiButton::Screensaver, "Clear", Some(CLEAR)),
            (UiButton::Back, "Back", None),
        ],
        Screen::SettingsListening => &[],
        Screen::SettingsTesting => &[(UiButton::Back, "Hold to stop", None)],
        Screen::SettingsHotkeys => &[
            (UiButton::Accept, "Change", None),
            (UiButton::Screensaver, "Clear", Some(CLEAR)),
            (UiButton::Back, "Back", None),
            (UiButton::Menu, "Home", Some(HOME)),
        ],
        // Mistress's GamePage.Hints: A's word for what is focused, Left and Right where they step, Y the heart, the
        // shoulders the sections, B and Start.
        Screen::GamePage => &[(UiButton::Accept, "Play", None), PAGE_HEART, PAGE_SECTION, BACK, PAGE_HOME],
        Screen::GamePageResume => &[(UiButton::Accept, "Resume", None), PAGE_HEART, PAGE_SECTION, BACK, PAGE_HOME],
        Screen::GamePageResumeSlots => &[(UiButton::Accept, "Resume", None), (UiButton::Left, "Slot", None), PAGE_HEART, PAGE_SECTION, BACK, PAGE_HOME],
        Screen::GamePageFavourite => &[(UiButton::Accept, "Favourite", None), PAGE_HEART, PAGE_SECTION, BACK, PAGE_HOME],
        Screen::GamePageLoad => &[(UiButton::Accept, "Load", None), (UiButton::Screensaver, "Delete…", Some(DELETE)), PAGE_HEART, PAGE_SECTION, BACK, PAGE_HOME],
        Screen::GamePageView => &[(UiButton::Accept, "View", None), PAGE_HEART, PAGE_SECTION, BACK, PAGE_HOME],
        Screen::GamePageValue => &[(UiButton::Left, "Value", None), PAGE_HEART, PAGE_SECTION, BACK, PAGE_HOME],
        Screen::GamePageChange => &[(UiButton::Accept, "Change", None), PAGE_HEART, PAGE_SECTION, BACK, PAGE_HOME],
        Screen::GamePageChoose => &[(UiButton::Accept, "Choose", None), PAGE_HEART, PAGE_SECTION, BACK, PAGE_HOME],
        Screen::GamePageNone => &[PAGE_HEART, PAGE_SECTION, BACK, PAGE_HOME],
        Screen::GamePageEdit => &[(UiButton::Accept, "Change", None), (UiButton::Search, "Reset", Some(RESET)), BACK],
        Screen::GamePageViewer => &[(UiButton::Left, "Browse", None), BACK],
        Screen::GamePageViewerManual => &[(UiButton::Left, "Browse", None), (UiButton::Up, "Manual", None), BACK],
        Screen::GamePageManual => &[(UiButton::Left, "Page", None), (UiButton::PageUp, "Zoom", None), (UiButton::Down, "Media", None), BACK],
        Screen::GamePageViewerCover => &[(UiButton::Left, "Browse", None), (UiButton::Screensaver, "Set as cover", Some(SET_COVER)), BACK],
        Screen::GamePageViewerManualCover => &[(UiButton::Left, "Browse", None), (UiButton::Up, "Manual", None), (UiButton::Screensaver, "Set as cover", Some(SET_COVER)), BACK],
        Screen::GamePageCoverArt => &[(UiButton::Accept, "Use this one", None), PAGE_HEART, (UiButton::Back, "Details", None), PAGE_HOME],
        Screen::GamePageState => &[(UiButton::Accept, "Load this state", None), (UiButton::Screensaver, "Delete", Some(DELETE)), BACK],
        Screen::GamePageStart => &[(UiButton::Accept, "Start", None), PAGE_HEART, BACK, PAGE_HOME],
        Screen::GamePageSections => &[PAGE_SECTION],
        Screen::GamePageResumeBoard => &[(UiButton::Accept, "Resume", None), PAGE_HEART, BACK, PAGE_HOME],
        Screen::GamePageOpen => &[(UiButton::Accept, "Open", None), BACK, PAGE_HOME],
        Screen::GamePageDetail => &[(UiButton::Accept, "Choose", None), BACK, PAGE_HOME],
        Screen::GamePageOption => &[
            (UiButton::Accept, "Change", None),
            (UiButton::Left, "Value", None),
            (UiButton::Screensaver, "Back to the console's", Some(DEFAULT)),
            BACK,
            PAGE_HOME,
        ],
    }
}

/// The Themes tab's check of the focused theme.
pub const CHECK: &str = "check";

/// The shoulders' entry where a screen's shoulders jump a group of its own row, and B's where a screen has none (the
/// themes' help bars, EmuSen_Qt.md §3.2): no action, since the walk and the theme answer them.
pub const GROUP: Entry = (UiButton::PageUp, "Group", None);
/// The group's entry on the triggers, where a screen's triggers step its makers (Eclipse's Choose a console, R-20261115-17).
pub const MAKER: Entry = (UiButton::First, "Maker", None);
pub const BACK: Entry = (UiButton::Back, "Back", None);
/// The triggers on a games view that steps letters (R-20261115-12): LT the letter before, drawn as one entry with RT, the letter after; the List card's Jump to by the same actions.
pub const LETTER: Entry = (UiButton::First, "Letter", Some(crate::list_card::JUMP_BACK));
pub const LETTER_ON: Entry = (UiButton::Last, "Letter", Some(crate::list_card::JUMP_ON));

/// Where the letters' entry goes in a screen's bar that takes it: after the shoulders' Console.
pub fn letters_at(screen: Screen) -> Option<usize> {
    match screen {
        Screen::Grid | Screen::GridSearched => entries(screen).iter().position(|e| e.0 == UiButton::PageUp).map(|at| at + 1),
        _ => None,
    }
}

/// As `entries_with`, the letters' entry added where the screen takes it and `letters` asks for it.
pub fn entries_with_letters(screen: Screen, group: bool, back: bool, letters: bool) -> Vec<Entry> {
    with_letters(screen, entries_with(screen, group, back), letters)
}

/// A screen's list with the letters' entry added after its shoulders' entry where the screen takes it and `letters` asks.
pub fn with_letters(screen: Screen, mut list: Vec<Entry>, letters: bool) -> Vec<Entry> {
    let at = list.iter().position(|e| e.0 == UiButton::PageUp).map(|at| at + 1);
    if let Some(at) = at.filter(|_| letters && letters_at(screen).is_some()) {
        list.insert(at, LETTER);
    }
    list
}

/// As `action_lines`, with the triggers' two actions where the letters' entry is added.
pub fn action_lines_with(screen: Screen, letters: bool) -> String {
    action_lines(screen) + &letter_lines(screen, letters)
}

/// The triggers' two action lines where the screen takes the letters' entry and `letters` asks for it.
pub fn letter_lines(screen: Screen, letters: bool) -> String {
    let added = if letters && letters_at(screen).is_some() { [LETTER, LETTER_ON].as_slice() } else { &[] };
    added.iter().filter_map(|(button, _, action)| action.map(|key| format!("{key} {button:?}\n"))).collect()
}

/// The shoulders' entry where a screen draws the program's sections (crate::sections): no action, since the walk steps
/// them and the theme opens the one chosen.
pub const SECTIONS: Entry = (UiButton::PageUp, "Sections", None);

/// A game's page's shared entries: Y the heart, the shoulders the sections (the frame steps them), Start Home.
const PAGE_HEART: Entry = (UiButton::Search, "Favourite", Some(FAVOURITE));
const PAGE_SECTION: Entry = (UiButton::PageUp, "Section", None);
const PAGE_HOME: Entry = (UiButton::Menu, "Home", Some(HOME));

/// Where the group entry goes in a screen's bar, after the entries that act on the focused item; none where the shoulders
/// already mean something (the grid's Console, the search's Move to grid) or the screen has no row of groups.
pub fn group_at(screen: Screen) -> Option<usize> {
    match screen {
        Screen::Home | Screen::HomeShelf => Some(3),
        Screen::HomePlaces => Some(1),
        Screen::Consoles => Some(2),
        _ => None,
    }
}

/// Where B Back goes in a screen's bar that has no B: after the entries that act on the focused item.
pub fn back_at(screen: Screen) -> Option<usize> {
    match screen {
        Screen::Home | Screen::HomeShelf => Some(3),
        Screen::HomePlaces => Some(1),
        _ => None,
    }
}

/// A list as Eclipse's Home names it (Main.dc.html:94-97): X the game's options and Start the Home card it opens, and no
/// Select search, which the theme draws as its own button. The actions are the list's own.
pub fn card_named(list: Vec<Entry>) -> Vec<Entry> {
    list.into_iter()
        .filter(|e| e.0 != UiButton::Options)
        .map(|(button, label, action)| match (button, action) {
            (UiButton::Screensaver, Some(GAME_PAGE)) => (button, crate::words::OPTIONS, action),
            (UiButton::Menu, Some(HOME)) => (button, crate::words::HOME_CARD, action),
            _ => (button, label, action),
        })
        .collect()
}

/// X on a Settings row as Eclipse's boards name it: the row's value back to its default (R-20261116-22).
pub const DEFAULT: &str = "default";
pub const TO_DEFAULT: Entry = (UiButton::Screensaver, "Back to the default", Some(DEFAULT));
/// Settings Consoles' Y filter and LT/RT letter-jump actions (board #27, R-20261116-22).
pub const CONSOLES_FILTER: &str = "consolesFilter";
pub const CONSOLES_LETTER_BACK: &str = "consolesLetterBack";
pub const CONSOLES_LETTER_ON: &str = "consolesLetterOn";

/// Settings Consoles' help entries as board #27 and R-20261123-20 name and order them.
pub fn consoles_settings_named(filtered: bool) -> Vec<Entry> {
    vec![
        (UiButton::Accept, "Open the console", None),
        (UiButton::First, "Jump a letter", Some(CONSOLES_LETTER_BACK)),
        (UiButton::Search, if filtered { "All consoles" } else { "Only consoles with games" }, Some(CONSOLES_FILTER)),
        (UiButton::Back, "Back", None),
        (UiButton::PageUp, "Section", None),
        (UiButton::Menu, "Home", Some(HOME)),
    ]
}

/// Settings Consoles' declared action lines, including RT's forward letter jump beside LT's backward jump.
pub fn consoles_settings_lines(list: &[Entry]) -> String {
    format!("{}{CONSOLES_LETTER_ON} {:?}\n", lines_of(Screen::Settings, list), UiButton::Last)
}

/// Settings Memory cards' help entries as boards #58, #63 and #64 name and order them.
pub fn cards_settings_named() -> Vec<Entry> {
    vec![
        (UiButton::Left, "Value", None),
        (UiButton::Accept, "Change", None),
        (UiButton::Back, "Back", None),
        (UiButton::Menu, "Home", Some(HOME)),
    ]
}

/// A Settings list as Eclipse's boards name and order it, X added where a value steps and Home kept (EmuSen_Qt.md §3.4.11, R-20261123-20).
pub fn settings_named(list: Vec<Entry>) -> Vec<Entry> {
    const ORDER: [UiButton; 6] = [UiButton::Left, UiButton::Accept, UiButton::Screensaver, UiButton::Back, UiButton::PageUp, UiButton::Menu];
    let steps = list.iter().any(|e| e.0 == UiButton::Left);
    let mut named: Vec<Entry> = list
        .into_iter()
        .map(|(button, label, action)| match (button, label) {
            (UiButton::Accept, "Change") => (button, "Choose or open", action),
            (UiButton::Left, "Value") => (button, "Change", action),
            _ => (button, label, action),
        })
        .collect();
    if steps {
        named.push(TO_DEFAULT);
    }
    if !named.iter().any(|e| e.0 == UiButton::Menu) {
        named.push((UiButton::Menu, "Home", Some(HOME)));
    }
    named.sort_by_key(|e| ORDER.iter().position(|b| *b == e.0).unwrap_or(ORDER.len()));
    named
}

/// Choose a console's list as Eclipse's board gives it: Y steps the order and X opens options (EclipseConsoles:46-50, 94).
pub fn options_named(list: Vec<Entry>) -> Vec<Entry> {
    list.into_iter()
        .map(|(button, label, action)| match (button, action) {
            (UiButton::Search, Some(crate::library::FAVOURITES)) => (button, crate::words::SORT, Some(SORT)),
            (UiButton::Screensaver, Some(RANDOM)) => (button, crate::words::OPTIONS, Some(CONSOLE_OPTIONS)),
            _ => (button, label, action),
        })
        .collect()
}

/// A list as Eclipse's games view's foot names and orders it (EclipseLibrary:84-86): A Start, B Back, Y Favourite; X is the Game page button the column draws, and the triggers' letters stand at the right.
pub fn games_named(list: Vec<Entry>) -> Vec<Entry> {
    const ORDER: [UiButton; 3] = [UiButton::Accept, UiButton::Back, UiButton::Search];
    let mut named: Vec<Entry> = list
        .into_iter()
        .filter_map(|(button, label, action)| match button {
            UiButton::Accept => Some((button, crate::words::START, action)),
            UiButton::Back => Some((button, BACK.1, action)),
            UiButton::Search => Some((button, label, action)),
            _ => None,
        })
        .collect();
    named.sort_by_key(|e| ORDER.iter().position(|b| *b == e.0));
    named
}

/// The frame's action lines of a list as it stands, with the screen's unnamed actions after.
pub fn lines_of(screen: Screen, list: &[Entry]) -> String {
    list.iter().chain(unnamed(screen)).filter_map(|(button, _, action)| action.map(|key| format!("{key} {button:?}\n"))).collect()
}

/// A screen's entries with the group's and B's added where the screen takes them, B before the group.
pub fn entries_with(screen: Screen, group: bool, back: bool) -> Vec<Entry> {
    entries_grouped(screen, group.then_some(GROUP), back)
}

/// As `entries_with`, the group's entry given: `GROUP` on the shoulders or `MAKER` on the triggers.
pub fn entries_grouped(screen: Screen, group: Option<Entry>, back: bool) -> Vec<Entry> {
    let mut list = entries(screen).to_vec();
    let has_back = list.iter().any(|e| e.0 == UiButton::Back);
    let back_place = back_at(screen).filter(|_| back && !has_back);
    if let Some(at) = back_place {
        list.insert(at, BACK);
    }
    if let (Some(at), Some(entry)) = (group_at(screen), group) {
        list.insert(at + usize::from(back_place.is_some_and(|b| b <= at)), entry);
    }
    list
}

/// A list without the entries carrying the screen's own actions (Favourites, Random game); the frame's own stay (EmuSen_Qt.md §3.2).
pub fn without_actions(mut list: Vec<Entry>) -> Vec<Entry> {
    list.retain(|e| e.2.is_none());
    list
}

/// Start's action away from a game and a card: Home (EmuSen_BigPicture.md §51.2, §51.3).
pub const HOME: &str = "home";
/// The bindings page's own actions (Mistress's HintReset, HintPlayer, HintClear, HintTest).
pub const RESET: &str = "reset";
pub const PLAYER: &str = "player";
pub const CLEAR: &str = "clear";
pub const TEST: &str = "test";

/// The focused game's page, its heart, and a random game: actions a later stage carries out.
pub const GAME_PAGE: &str = "gamePage";
pub const FAVOURITE: &str = "favourite";
/// X on a game's page's state: its deletion, asked first.
pub const DELETE: &str = "delete";
/// X in a game's page's media viewer (EmuSen_Interface.md §18.2): the picture shown made the game's cover.
pub const SET_COVER: &str = "setCover";
pub const RANDOM: &str = "random";
/// Y on Eclipse's Choose a console: the next order of the consoles, which the theme keeps.
pub const SORT: &str = "sort";
/// X on Eclipse's Choose a console: the focused console's options, Settings' page of it.
pub const CONSOLE_OPTIONS: &str = "consoleOptions";

/// The shoulders on the grid: the console before and the console after (Mistress's `StepSection`).
pub const PREVIOUS: &str = "previous";
pub const NEXT: &str = "next";

/// A card's declared action that closes it, as Start closes a card away from a game.
pub const CLOSE: &str = "close";

/// The second button an entry names: LB with RB, the d-pad's Left with Right, LT with RT (EmuSen_Qt.md §3.2, `Help`).
pub fn pair_of(button: UiButton) -> Option<UiButton> {
    match button {
        UiButton::PageUp => Some(UiButton::PageDown),
        UiButton::First => Some(UiButton::Last),
        UiButton::Left => Some(UiButton::Right),
        _ => None,
    }
}

/// The screen's actions as the frame declares them: `key Button`, one a line (EmuSen_Kit.md §7.19).
pub fn action_lines(screen: Screen) -> String {
    lines_of(screen, entries(screen))
}

/// A screen's actions its help bar does not name: Start on Choose a console goes Home, as Mistress's
/// `ConsolesPadCommand` has it, though the mockup's bar shows four entries; the right shoulder of a pair the bar draws
/// as one entry (the grid's Console, the search's Move to grid).
pub fn unnamed(screen: Screen) -> &'static [Entry] {
    match screen {
        Screen::Consoles => &[(UiButton::Menu, "Home", Some(HOME))],
        Screen::Grid | Screen::GridSearched => &[(UiButton::PageDown, "Console", Some(NEXT))],
        Screen::Search => &[(UiButton::PageDown, "Move to grid", Some(crate::screens::TO_GRID))],
        Screen::ToolSearch => &[(UiButton::PageDown, "Move to list", Some(crate::screens::TO_GRID))],
        _ => &[],
    }
}
