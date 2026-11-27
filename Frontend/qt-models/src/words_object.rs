//! `Words`, the screens' fixed words as a QML singleton read from words.rs, so a QML file names a word and never writes one.

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
        #[qml_element]
        #[qml_singleton]
        #[qproperty(QString, title, READ, CONSTANT)]
        #[qproperty(QString, games, READ, CONSTANT)]
        #[qproperty(QString, choose_a_console, cxx_name = "chooseAConsole", READ, CONSTANT)]
        #[qproperty(QString, games_path, cxx_name = "gamesPath", READ, CONSTANT)]
        #[qproperty(QString, continue_playing, cxx_name = "continuePlaying", READ, CONSTANT)]
        #[qproperty(QString, recent, READ, CONSTANT)]
        #[qproperty(QStringList, home_places, cxx_name = "homePlaces", READ, CONSTANT)]
        #[qproperty(QString, emusen_tag, cxx_name = "emusenTag", READ, CONSTANT)]
        #[qproperty(QString, between, READ, CONSTANT)]
        #[qproperty(QString, search, READ, CONSTANT)]
        #[qproperty(QString, libretro_tag, cxx_name = "libretroTag", READ, CONSTANT)]
        #[qproperty(QString, name, READ, CONSTANT)]
        #[qproperty(QString, continue_entry, cxx_name = "continueEntry", READ, CONSTANT)]
        #[qproperty(QString, key_pair, cxx_name = "keyPair", READ, CONSTANT)]
        #[qproperty(QString, resume_launch, cxx_name = "resumeLaunch", READ, CONSTANT)]
        #[qproperty(QString, resume_here, cxx_name = "resumeHere", READ, CONSTANT)]
        #[qproperty(QString, resume_start_again_sub, cxx_name = "resumeStartAgainSub", READ, CONSTANT)]
        #[qproperty(QString, resume_continue_fallback_sub, cxx_name = "resumeContinueFallbackSub", READ, CONSTANT)]
        #[qproperty(QString, resume_no_preview, cxx_name = "resumeNoPreview", READ, CONSTANT)]
        #[qproperty(QString, resume_state_ready, cxx_name = "resumeStateReady", READ, CONSTANT)]
        type Words = super::WordsRust;
    }

    extern "RustQt" {
        /// Battery charge text ("82%"), or empty when no battery is present.
        #[qinvokable]
        fn percent(self: &Words, level: i32) -> QString;

        /// Eclipse Home Card header clock line ("82% · 9:41 PM", or clock alone).
        #[qinvokable]
        #[cxx_name = "statusClock"]
        fn status_clock(self: &Words, battery: i32, time: &QString) -> QString;

        /// Eclipse Card footer counter ("2 of 7", or empty for zero rows).
        #[qinvokable]
        #[cxx_name = "positionLine"]
        fn position_line(self: &Words, index: i32, total: i32) -> QString;

        /// Footer B label for an Eclipse dialog: the choice B answers, or "Back".
        #[qinvokable]
        #[cxx_name = "questionBackLabel"]
        fn question_back_label(self: &Words, choices: &QStringList, back_index: i32) -> QString;

        /// Eclipse Resume Card saved pill line ("Saved Today, 9:42 PM" or "Saved state ready").
        #[qinvokable]
        #[cxx_name = "resumeSavedLine"]
        fn resume_saved_line(self: &Words, when: &QString) -> QString;

        /// Console prefix before CardPlanet on the Eclipse Resume Card eyebrow.
        #[qinvokable]
        #[cxx_name = "resumeEyebrowLeft"]
        fn resume_eyebrow_left(self: &Words, console: &QString, has_core: bool) -> QString;

        /// Core label after CardPlanet on the Eclipse Resume Card eyebrow ("<CORE> CORE").
        #[qinvokable]
        #[cxx_name = "resumeEyebrowCore"]
        fn resume_eyebrow_core(self: &Words, core: &QString) -> QString;

        /// Eclipse Resume Card stage footer copy naming the saved slot timestamp.
        #[qinvokable]
        #[cxx_name = "resumeFootLine"]
        fn resume_foot_line(self: &Words, when: &QString) -> QString;

        /// Reduced aspect ratio and dimensions pill ("8:7 · 256×224") or "STATE READY".
        #[qinvokable]
        #[cxx_name = "resumeStageMeta"]
        fn resume_stage_meta(self: &Words, width: i32, height: i32) -> QString;
    }
}

use crate::words;
use cxx_qt_lib::{QList, QString, QStringList};

/// The words, each from words.rs.
pub struct WordsRust {
    title: QString,
    games: QString,
    choose_a_console: QString,
    games_path: QString,
    continue_playing: QString,
    recent: QString,
    home_places: QStringList,
    emusen_tag: QString,
    between: QString,
    search: QString,
    libretro_tag: QString,
    name: QString,
    continue_entry: QString,
    key_pair: QString,
    resume_launch: QString,
    resume_here: QString,
    resume_start_again_sub: QString,
    resume_continue_fallback_sub: QString,
    resume_no_preview: QString,
    resume_state_ready: QString,
}

impl Default for WordsRust {
    fn default() -> Self {
        let places: QList<QString> = words::HOME_PLACES.iter().map(|p| QString::from(*p)).collect();
        WordsRust {
            title: QString::from(words::TITLE),
            games: QString::from(words::GAMES),
            choose_a_console: QString::from(words::CHOOSE_A_CONSOLE),
            games_path: QString::from(words::GAMES_PATH),
            continue_playing: QString::from(words::CONTINUE_PLAYING),
            recent: QString::from(words::RECENT),
            home_places: QStringList::from(&places),
            emusen_tag: QString::from(words::EMUSEN_TAG),
            between: QString::from(words::BETWEEN),
            search: QString::from(words::SEARCH),
            libretro_tag: QString::from(words::LIBRETRO_TAG),
            name: QString::from(words::NAME),
            continue_entry: QString::from(words::CONTINUE_ENTRY),
            key_pair: QString::from(words::KEY_PAIR),
            resume_launch: QString::from(words::RESUME_LAUNCH),
            resume_here: QString::from(words::RESUME_HERE),
            resume_start_again_sub: QString::from(words::RESUME_START_AGAIN_SUB),
            resume_continue_fallback_sub: QString::from(words::RESUME_CONTINUE_FALLBACK_SUB),
            resume_no_preview: QString::from(words::RESUME_NO_PREVIEW),
            resume_state_ready: QString::from(words::RESUME_STATE_READY),
        }
    }
}

impl qobject::Words {
    pub fn percent(&self, level: i32) -> QString {
        QString::from(words::percent(level).as_str())
    }

    pub fn status_clock(&self, battery: i32, time: &QString) -> QString {
        QString::from(words::status_clock(battery, &time.to_string()).as_str())
    }

    pub fn position_line(&self, index: i32, total: i32) -> QString {
        QString::from(words::position_line(index.max(0) as usize, total.max(0) as usize).as_str())
    }

    pub fn question_back_label(&self, choices: &QStringList, back_index: i32) -> QString {
        let items: Vec<String> = choices.iter().map(|s| s.to_string()).collect();
        QString::from(crate::question::back_label(&items, back_index).as_str())
    }

    pub fn resume_saved_line(&self, when: &QString) -> QString {
        QString::from(words::resume_saved_line(&when.to_string()).as_str())
    }

    pub fn resume_eyebrow_left(&self, console: &QString, has_core: bool) -> QString {
        QString::from(words::resume_eyebrow_left(&console.to_string(), has_core).as_str())
    }

    pub fn resume_eyebrow_core(&self, core: &QString) -> QString {
        QString::from(words::resume_eyebrow_core(&core.to_string()).as_str())
    }

    pub fn resume_foot_line(&self, when: &QString) -> QString {
        QString::from(words::resume_foot_line(&when.to_string()).as_str())
    }

    pub fn resume_stage_meta(&self, width: i32, height: i32) -> QString {
        QString::from(words::resume_stage_meta(width, height).as_str())
    }
}
