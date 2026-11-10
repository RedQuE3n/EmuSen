//! `Words`, every word a theme may show that no row carries (EmuSen_Qt.md §3.2; §1.3 bans words in QML): the labels of
//! the three screens and the counted and timed phrases, each from emusen-qt-models' words.rs, none written here.

use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use emusen_qt_models::words;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element = "Words"]
        #[qml_singleton]
        #[qproperty(QString, title, READ, CONSTANT)]
        #[qproperty(QString, games_heading, cxx_name = "gamesHeading", READ, CONSTANT)]
        #[qproperty(QString, choose_a_console, cxx_name = "chooseAConsole", READ, CONSTANT)]
        #[qproperty(QString, games_path, cxx_name = "gamesPath", READ, CONSTANT)]
        #[qproperty(QString, continue_playing, cxx_name = "continuePlaying", READ, CONSTANT)]
        #[qproperty(QString, recent, READ, CONSTANT)]
        #[qproperty(QString, between, READ, CONSTANT)]
        #[qproperty(QString, search, READ, CONSTANT)]
        #[qproperty(QString, emusen_tag, cxx_name = "emusenTag", READ, CONSTANT)]
        #[qproperty(QString, libretro_tag, cxx_name = "libretroTag", READ, CONSTANT)]
        #[qproperty(QString, no_games, cxx_name = "noGames", READ, CONSTANT)]
        #[qproperty(QString, no_recent, cxx_name = "noRecent", READ, CONSTANT)]
        #[qproperty(QString, no_consoles, cxx_name = "noConsoles", READ, CONSTANT)]
        #[qproperty(QString, add_folder, cxx_name = "addFolder", READ, CONSTANT)]
        #[qproperty(QString, places, READ, CONSTANT)]
        #[qproperty(QString, continue_entry, cxx_name = "continueEntry", READ, CONSTANT)]
        #[qproperty(QString, recent_entry, cxx_name = "recentEntry", READ, CONSTANT)]
        /// The time the timed phrases are said against, in seconds since 1970; 0, the default, is the wall clock. The
        /// program sets it from its clock, which a test holds (decided 2026-11-09 by the tester); a theme only reads it.
        #[qproperty(f64, now, READ, NOTIFY)]
        type ThemeWords = super::ThemeWordsRust;
    }

    extern "RustQt" {
        /// "1 game", "1,566 games".
        #[qinvokable]
        fn games(self: &ThemeWords, count: i32) -> QString;
        /// How long ago a time in seconds since 1970 was: "12 min ago".
        #[qinvokable]
        fn ago(self: &ThemeWords, at: f64) -> QString;
        /// Continue's facts: the console's name and how long ago the game was paused.
        #[qinvokable]
        #[cxx_name = "pausedFacts"]
        fn paused_facts(self: &ThemeWords, console: &QString, paused_since: f64) -> QString;
        /// A console's facts: "Nintendo · 1983 · 2 games · runs on Moon, an EmuSen core", the empty parts left out.
        #[qinvokable]
        fn facts(self: &ThemeWords, maker: &QString, year: i32, count: i32, runs_on: &QString) -> QString;
        /// The facts' last words, from a console's `engineName` and `engineKind`.
        #[qinvokable]
        #[cxx_name = "runsOn"]
        fn runs_on(self: &ThemeWords, engine_name: &QString, engine_kind: &QString) -> QString;
        /// Home's status line: the controllers connected, where there are any, and the clock.
        #[qinvokable]
        #[cxx_name = "statusLine"]
        fn status_line(self: &ThemeWords, controllers: i32, clock: &QString) -> QString;
        /// The splash's caption: the consoles' count in capitals.
        #[qinvokable]
        #[cxx_name = "splashCaption"]
        fn splash_caption(self: &ThemeWords, consoles: i32) -> QString;
        /// Choose a console's summary: "11 consoles · 1,566 games".
        #[qinvokable]
        fn summary(self: &ThemeWords, consoles: i32, games: i32) -> QString;
        /// How long ago the suspended game was paused: "Paused 12 minutes ago".
        #[qinvokable]
        #[cxx_name = "pausedAgo"]
        fn paused_ago(self: &ThemeWords, paused_since: f64) -> QString;
    }
}

pub struct ThemeWordsRust {
    title: QString,
    games_heading: QString,
    choose_a_console: QString,
    games_path: QString,
    continue_playing: QString,
    recent: QString,
    between: QString,
    search: QString,
    emusen_tag: QString,
    libretro_tag: QString,
    no_games: QString,
    no_recent: QString,
    no_consoles: QString,
    add_folder: QString,
    places: QString,
    continue_entry: QString,
    recent_entry: QString,
    now: f64,
}

impl Default for ThemeWordsRust {
    fn default() -> Self {
        ThemeWordsRust {
            title: QString::from(words::TITLE),
            games_heading: QString::from(words::GAMES),
            choose_a_console: QString::from(words::CHOOSE_A_CONSOLE),
            games_path: QString::from(words::GAMES_PATH),
            continue_playing: QString::from(words::CONTINUE_PLAYING),
            recent: QString::from(words::RECENT),
            between: QString::from(words::BETWEEN),
            search: QString::from(words::SEARCH),
            emusen_tag: QString::from(words::EMUSEN_TAG),
            libretro_tag: QString::from(words::LIBRETRO_TAG),
            no_games: QString::from(words::NO_GAMES),
            no_recent: QString::from(words::NO_RECENT),
            no_consoles: QString::from(words::NO_CONSOLES),
            add_folder: QString::from(words::ADD_FOLDER),
            places: QString::from(words::PLACES),
            continue_entry: QString::from(words::CONTINUE_ENTRY),
            recent_entry: QString::from(words::RECENT_ENTRY),
            now: 0.0,
        }
    }
}

fn said(text: String) -> QString {
    QString::from(text.as_str())
}

impl qobject::ThemeWords {
    /// The host's: the time the phrases are said against, 0 for the wall clock.
    pub fn show_now(mut self: core::pin::Pin<&mut Self>, now: f64) {
        if self.now != now {
            self.as_mut().rust_mut().now = now;
            self.now_changed();
        }
    }

    /// Now, held or the wall clock's.
    fn at(&self) -> i64 {
        if self.now > 0.0 { self.now as i64 } else { words::now() }
    }

    pub fn games(&self, count: i32) -> QString {
        said(words::games(count.max(0) as usize))
    }

    pub fn ago(&self, at: f64) -> QString {
        said(words::ago(at as i64, self.at()))
    }

    pub fn paused_facts(&self, console: &QString, paused_since: f64) -> QString {
        said(words::paused_facts(&console.to_string(), paused_since as i64, self.at()))
    }

    pub fn facts(&self, maker: &QString, year: i32, count: i32, runs_on: &QString) -> QString {
        said(words::facts(&maker.to_string(), year, count.max(0) as usize, &runs_on.to_string()))
    }

    pub fn runs_on(&self, engine_name: &QString, engine_kind: &QString) -> QString {
        let name = engine_name.to_string();
        if name.is_empty() {
            return QString::default();
        }
        said(words::runs_on(&name, engine_kind.to_string() == crate::view::engine_kind(true)))
    }

    pub fn status_line(&self, controllers: i32, clock: &QString) -> QString {
        said(words::status_line(controllers.max(0) as usize, &clock.to_string()))
    }

    pub fn summary(&self, consoles: i32, games: i32) -> QString {
        said(words::summary(consoles.max(0) as usize, games.max(0) as usize))
    }

    pub fn paused_ago(&self, paused_since: f64) -> QString {
        said(words::paused_ago(paused_since as i64, self.at()))
    }

    pub fn splash_caption(&self, consoles: i32) -> QString {
        said(words::splash_caption(consoles.max(0) as usize))
    }
}
