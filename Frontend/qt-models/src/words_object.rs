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
        #[qproperty(QString, libretro_tag, cxx_name = "libretroTag", READ, CONSTANT)]
        type Words = super::WordsRust;
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
    libretro_tag: QString,
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
            libretro_tag: QString::from(words::LIBRETRO_TAG),
        }
    }
}
