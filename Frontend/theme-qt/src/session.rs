//! `Session`, the suspended game as a theme reads it (EmuSen_Qt.md §3.2): whether there is one, its title and console,
//! when it was paused and how long it was played, and its paused picture as an item a theme draws. The host feeds it.

use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!(<QtQuick/QQuickItem>);
        type QQuickItem;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element = "Session"]
        #[qml_singleton]
        #[qproperty(bool, has_game, cxx_name = "hasGame", READ, NOTIFY)]
        #[qproperty(QString, title, READ, NOTIFY)]
        #[qproperty(QString, console_id, cxx_name = "consoleId", READ, NOTIFY)]
        #[qproperty(f64, paused_since, cxx_name = "pausedSince", READ, NOTIFY)]
        #[qproperty(f64, played, READ, NOTIFY)]
        #[qproperty(*mut QQuickItem, frame, READ, NOTIFY)]
        #[qproperty(QString, key, READ, NOTIFY)]
        #[qproperty(QString, console, READ, NOTIFY)]
        #[qproperty(f64, hue, READ, NOTIFY)]
        type ThemeSession = super::ThemeSessionRust;
    }
}

/// What the host knows of the suspended game: times in seconds, `paused_since` since 1970.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SessionFacts {
    pub has_game: bool,
    /// The game's file, as a game row's `key`.
    pub key: String,
    pub title: String,
    pub console_id: String,
    /// The console's name as the game's facts begin, and its hue; -1 where the library does not hold the game.
    pub console: String,
    pub hue: f64,
    pub paused_since: i64,
    pub played: i64,
}

pub struct ThemeSessionRust {
    has_game: bool,
    key: QString,
    title: QString,
    console_id: QString,
    console: QString,
    hue: f64,
    paused_since: f64,
    played: f64,
    frame: *mut qobject::QQuickItem,
}

impl Default for ThemeSessionRust {
    fn default() -> Self {
        ThemeSessionRust {
            has_game: false,
            key: QString::default(),
            title: QString::default(),
            console_id: QString::default(),
            console: QString::default(),
            hue: -1.0,
            paused_since: 0.0,
            played: 0.0,
            frame: std::ptr::null_mut(),
        }
    }
}

impl qobject::ThemeSession {
    /// The host's feed; each property notifies only when it changed.
    pub fn show(mut self: Pin<&mut Self>, facts: &SessionFacts) {
        if self.has_game != facts.has_game {
            self.as_mut().rust_mut().has_game = facts.has_game;
            self.as_mut().has_game_changed();
        }
        let key = QString::from(facts.key.as_str());
        if self.key != key {
            self.as_mut().rust_mut().key = key;
            self.as_mut().key_changed();
        }
        let title = QString::from(facts.title.as_str());
        if self.title != title {
            self.as_mut().rust_mut().title = title;
            self.as_mut().title_changed();
        }
        let console = QString::from(facts.console_id.as_str());
        if self.console_id != console {
            self.as_mut().rust_mut().console_id = console;
            self.as_mut().console_id_changed();
        }
        let name = QString::from(facts.console.as_str());
        if self.console != name {
            self.as_mut().rust_mut().console = name;
            self.as_mut().console_changed();
        }
        if self.hue != facts.hue {
            self.as_mut().rust_mut().hue = facts.hue;
            self.as_mut().hue_changed();
        }
        if self.paused_since != facts.paused_since as f64 {
            self.as_mut().rust_mut().paused_since = facts.paused_since as f64;
            self.as_mut().paused_since_changed();
        }
        if self.played != facts.played as f64 {
            self.as_mut().rust_mut().played = facts.played as f64;
            self.as_mut().played_changed();
        }
    }

    /// The item the game's picture is drawn in, which a theme samples for Continue; null for none.
    pub fn show_frame(mut self: Pin<&mut Self>, frame: *mut qobject::QQuickItem) {
        if self.frame != frame {
            self.as_mut().rust_mut().frame = frame;
            self.as_mut().frame_changed();
        }
    }
}
