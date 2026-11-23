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
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
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
        #[qproperty(QStringList, fact_labels, cxx_name = "factLabels", READ, CONSTANT)]
        #[qproperty(QStringList, console_shows, cxx_name = "consoleShows", READ, CONSTANT)]
        #[qproperty(QStringList, game_fact_labels, cxx_name = "gameFactLabels", READ, CONSTANT)]
        #[qproperty(QString, crumb, READ, CONSTANT)]
        #[qproperty(QString, no_games_line, cxx_name = "noGamesLine", READ, CONSTANT)]
        #[qproperty(QStringList, console_sorts, cxx_name = "consoleSorts", READ, CONSTANT)]
        /// The time the timed phrases are said against, in seconds since 1970; 0, the default, is the wall clock. The
        /// program sets it from its clock, which a test holds (decided 2026-11-09 by the tester); a theme only reads it.
        #[qproperty(f64, now, READ, NOTIFY)]
        #[qproperty(QString, name, READ, CONSTANT)]
        /// From 1.11: Left with Right named as one key, "◀ ▶", as the Settings boards' help line draws it.
        #[qproperty(QString, key_pair, cxx_name = "keyPair", READ, CONSTANT)]
        type ThemeWords = super::ThemeWordsRust;
    }

    extern "RustQt" {
        /// "1 game", "1,566 games".
        #[qinvokable]
        fn games(self: &ThemeWords, count: i32) -> QString;
        /// A count grouped by thousands: "1,214".
        #[qinvokable]
        fn count(self: &ThemeWords, count: i32) -> QString;
        /// Parts said as one line, the empty ones left out: "Nintendo · 1990".
        #[qinvokable]
        fn line(self: &ThemeWords, parts: &QStringList) -> QString;
        /// A battery's charge: "82%"; empty below 0, where the machine has none.
        #[qinvokable]
        fn percent(self: &ThemeWords, charge: i32) -> QString;
        /// Where an item stands among its list: "3 of 14", for the item at `index` from 0.
        #[qinvokable]
        fn place(self: &ThemeWords, index: i32, count: i32) -> QString;
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
        /// The counts under Eclipse's splash wordmark (1.10): "1,284 GAMES · 8 CORES".
        #[qinvokable]
        #[cxx_name = "splashCounts"]
        fn splash_counts(self: &ThemeWords, games: i32, cores: i32) -> QString;
        /// Choose a console's summary: "11 consoles · 1,566 games".
        #[qinvokable]
        fn summary(self: &ThemeWords, consoles: i32, games: i32) -> QString;
        /// How long ago the suspended game was paused: "Paused 12 minutes ago".
        #[qinvokable]
        #[cxx_name = "pausedAgo"]
        fn paused_ago(self: &ThemeWords, paused_since: f64) -> QString;
        /// A game's last play and its time played: "Last played yesterday · 3 h 20 min played".
        #[qinvokable]
        #[cxx_name = "playedFacts"]
        fn played_facts(self: &ThemeWords, at: f64, seconds: f64) -> QString;
        /// The line over a suspended game's name: "Continue · Super Nintendo".
        #[qinvokable]
        #[cxx_name = "continueLine"]
        fn continue_line(self: &ThemeWords, console: &QString) -> QString;

        /// "Resume point · saved 9:12 PM": a suspended game's picture and when it was paused, in the machine's short form.
        #[qinvokable]
        #[cxx_name = "resumePoint"]
        fn resume_point(self: &ThemeWords, paused_since: f64) -> QString;
        /// The art behind Eclipse's disc named: "Game art · Aurora Drift".
        #[qinvokable]
        #[cxx_name = "gameArt"]
        fn game_art(self: &ThemeWords, title: &QString) -> QString;
        /// A moment, in seconds since 1970, in the machine's zone and short time form: "9:12 PM" under en_US.
        #[qinvokable]
        #[cxx_name = "shortTime"]
        fn short_time(self: &ThemeWords, at: f64) -> QString;
    }
}

pub struct ThemeWordsRust {
    title: QString,
    name: QString,
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
    fact_labels: cxx_qt_lib::QStringList,
    console_shows: cxx_qt_lib::QStringList,
    game_fact_labels: cxx_qt_lib::QStringList,
    crumb: QString,
    no_games_line: QString,
    console_sorts: cxx_qt_lib::QStringList,
    key_pair: QString,
    now: f64,
}

impl Default for ThemeWordsRust {
    fn default() -> Self {
        ThemeWordsRust {
            title: QString::from(words::TITLE),
            name: QString::from(words::NAME),
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
            fact_labels: string_list(&words::FACT_LABELS),
            console_shows: string_list(&words::CONSOLE_SHOWS),
            game_fact_labels: string_list(&words::GAME_FACT_LABELS),
            crumb: QString::from(words::CRUMB),
            no_games_line: QString::from(words::NO_GAMES_LINE),
            console_sorts: string_list(&words::CONSOLE_SORTS),
            key_pair: QString::from(words::KEY_PAIR),
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

    pub fn line(&self, parts: &cxx_qt_lib::QStringList) -> QString {
        let parts: Vec<String> = cxx_qt_lib::QList::<QString>::from(parts).iter().map(|p| p.to_string()).collect();
        said(words::line(&parts.iter().map(String::as_str).collect::<Vec<_>>()))
    }

    pub fn count(&self, count: i32) -> QString {
        said(words::grouped(count.max(0) as usize))
    }

    pub fn percent(&self, charge: i32) -> QString {
        said(words::percent(charge))
    }

    pub fn place(&self, index: i32, count: i32) -> QString {
        said(emusen_qt_models::game_page::place(index.max(0) as usize, count.max(0) as usize))
    }

    pub fn ago(&self, at: f64) -> QString {
        said(words::ago(at as i64, self.at()))
    }

    pub fn paused_facts(&self, console: &QString, paused_since: f64) -> QString {
        said(words::paused_facts(&console.to_string(), paused_since as i64, self.at()))
    }

    pub fn played_facts(&self, at: f64, seconds: f64) -> QString {
        said(words::played_facts(at as i64, self.at(), seconds as i64))
    }

    pub fn continue_line(&self, console: &QString) -> QString {
        said(words::continue_line(&console.to_string()))
    }

    pub fn resume_point(&self, paused_since: f64) -> QString {
        said(words::resume_point(&emusen_qt_models::clock::short_time(paused_since as i64)))
    }

    pub fn game_art(&self, title: &QString) -> QString {
        said(words::game_art(&title.to_string()))
    }

    pub fn short_time(&self, at: f64) -> QString {
        QString::from(emusen_qt_models::clock::short_time(at as i64).as_str())
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

    pub fn splash_counts(&self, games: i32, cores: i32) -> QString {
        said(words::splash_counts(games.max(0) as usize, cores.max(0) as usize))
    }
}

/// Words as a QML list of strings.
fn string_list(words: &[&str]) -> cxx_qt_lib::QStringList {
    let mut list = cxx_qt_lib::QList::<QString>::default();
    words.iter().for_each(|w| list.append(QString::from(*w)));
    cxx_qt_lib::QStringList::from(&list)
}
