//! The words the screens show that no row carries, written once (EmuSen_Qt.md §1.2: QML holds no words), with the
//! counted phrases the rows' words are made of.

/// The wordmark: Moonrise's title and Home's (EmuSen_Interface.md §22, §24.3).
pub const TITLE: &str = "EMUSEN";

/// "1 game", "1,566 games": a count with its noun, grouped by thousands as the mockups write them.
pub fn counted(n: usize, one: &str, many: &str) -> String {
    format!("{} {}", grouped(n), if n == 1 { one } else { many })
}

/// A number grouped by thousands with commas.
pub fn grouped(n: usize) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// Moonrise's caption: the count of planets in capitals.
pub fn splash_caption(planets: usize) -> String {
    counted(planets, "CONSOLE", "CONSOLES")
}

/// Choose a console's title and line, as the courts canvas words them (EmuSen_Interface.md §24.1).
pub const GAMES: &str = "GAMES";
pub const CHOOSE_A_CONSOLE: &str = "Choose a console";
/// The grid's path before the console's name (§24.2).
pub const GAMES_PATH: &str = "GAMES ▸";
/// Home's headings (§24.3).
pub const CONTINUE_PLAYING: &str = "CONTINUE PLAYING";
pub const RECENT: &str = "RECENT";
/// Home's seven places, in order, as the Home canvas words them.
pub const HOME_PLACES: [&str; 7] = ["GAMES", "SCRAPER", "PACKAGES", "LIBRARY", "CORES", "SETTINGS", "POWER"];
/// The engine line's tags (§24.6).
pub const EMUSEN_TAG: &str = "EMUSEN";
pub const LIBRETRO_TAG: &str = "LIBRETRO";
/// The Favourites place's line.
pub const FAVOURITES_LINE: &str = "Your hearts, every console";

/// A count of games.
pub fn games(n: usize) -> String {
    counted(n, "game", "games")
}

/// The Recently played place's line: the game played last, or none yet.
pub fn last_played(title: Option<&str>) -> String {
    title.map_or_else(|| "Nothing played yet".to_owned(), |t| format!("Last: {t}"))
}

/// Choose a console's summary at the top right: "11 consoles · 1,566 games".
pub fn summary(consoles: usize, games: usize) -> String {
    format!("{} · {}", counted(consoles, "console", "consoles"), self::games(games))
}

/// The facts' last words: which engine runs the console now.
pub fn runs_on(name: &str, libretro: bool) -> String {
    format!("runs on {name}, {}", if libretro { "a libretro core" } else { "an EmuSen core" })
}

/// A focused console's facts: "Nintendo · 1983 · 2 games · runs on Moon, an EmuSen core", the empty parts left out.
pub fn facts(maker: &str, year: i32, games: usize, runs_on: &str) -> String {
    let year = if year > 0 { year.to_string() } else { String::new() };
    [maker, year.as_str(), self::games(games).as_str(), runs_on].iter().filter(|s| !s.is_empty()).copied().collect::<Vec<_>>().join(" · ")
}
