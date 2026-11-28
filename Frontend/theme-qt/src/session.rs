//! `Session`, the suspended and running game state as a theme reads it (EmuSen_Qt.md §3.2).

use crate::rows::qobject::ThemeRows;
use crate::view::Row;
use core::pin::Pin;
use cxx_qt::casting::Upcast;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!(<QtQuick/QQuickItem>);
        type QQuickItem;
        include!("emusen-theme-qt/src/rows.cxxqt.h");
        type ThemeRows = crate::rows::qobject::ThemeRows;
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
        #[qproperty(bool, running, READ, NOTIFY = live_changed)]
        #[qproperty(bool, paused, READ, NOTIFY = live_changed)]
        #[qproperty(QString, core, READ, NOTIFY = live_changed)]
        #[qproperty(i32, planet, READ, NOTIFY = live_changed)]
        #[qproperty(QString, fps_words, cxx_name = "fpsWords", READ, NOTIFY = live_changed)]
        #[qproperty(QString, status, READ, NOTIFY = live_changed)]
        #[qproperty(i32, slot, READ, NOTIFY = live_changed)]
        #[qproperty(i32, speed, READ, NOTIFY = live_changed)]
        #[qproperty(bool, master_cheats, cxx_name = "masterCheats", READ, NOTIFY = live_changed)]
        #[qproperty(bool, has_rewind, cxx_name = "hasRewind", READ, NOTIFY = live_changed)]
        #[qproperty(f64, latest_rewind_frame, cxx_name = "latestRewindFrame", READ, NOTIFY = live_changed)]
        #[qproperty(f64, sound_volume, cxx_name = "soundVolume", READ, NOTIFY = live_changed)]
        #[qproperty(bool, resume_asked, cxx_name = "resumeAsked", READ, NOTIFY = live_changed)]
        #[qproperty(QString, resume_subject, cxx_name = "resumeSubject", READ, NOTIFY = live_changed)]
        #[qproperty(QString, resume_picture, cxx_name = "resumePicture", READ, NOTIFY = live_changed)]
        #[qproperty(QString, resume_when, cxx_name = "resumeWhen", READ, NOTIFY = live_changed)]
        #[qproperty(QString, resume_console, cxx_name = "resumeConsole", READ, NOTIFY = live_changed)]
        #[qproperty(QString, resume_core, cxx_name = "resumeCore", READ, NOTIFY = live_changed)]
        #[qproperty(i32, resume_planet, cxx_name = "resumePlanet", READ, NOTIFY = live_changed)]
        #[qproperty(*mut ThemeRows, slots, READ, CONSTANT)]
        #[qproperty(*mut ThemeRows, cheats, READ, CONSTANT)]
        #[qproperty(*mut ThemeRows, console_options, cxx_name = "consoleOptions", READ, CONSTANT)]
        #[qproperty(*mut ThemeRows, rewind_marks, cxx_name = "rewindMarks", READ, CONSTANT)]
        type ThemeSession = super::ThemeSessionRust;
    }

    unsafe extern "RustQt" {
        #[qsignal]
        #[cxx_name = "liveChanged"]
        fn live_changed(self: Pin<&mut ThemeSession>);
    }

    impl cxx_qt::Initialize for ThemeSession {}
}

pub const SESSION_SLOT_ROLES: [&str; 5] = ["slot", "exists", "when", "picture", "active"];
pub const SESSION_CHEAT_ROLES: [&str; 4] = ["index", "description", "on", "code"];
pub const SESSION_OPTION_ROLES: [&str; 5] = ["key", "label", "value", "displayValue", "choiceCount"];
pub const SESSION_REWIND_ROLES: [&str; 5] = ["frame", "label", "short", "picture", "now"];

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

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SessionLiveFacts {
    pub running: bool,
    pub paused: bool,
    pub core: String,
    pub planet: i32,
    pub fps_words: String,
    pub status: String,
    pub slot: i32,
    pub speed: i32,
    pub master_cheats: bool,
    pub has_rewind: bool,
    pub latest_rewind_frame: f64,
    pub sound_volume: f64,
    pub resume_asked: bool,
    pub resume_subject: String,
    pub resume_picture: String,
    pub resume_when: String,
    pub resume_console: String,
    pub resume_core: String,
    pub resume_planet: i32,
    pub slots: Vec<Row>,
    pub cheats: Vec<Row>,
    pub console_options: Vec<Row>,
    pub rewind_marks: Vec<Row>,
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
    running: bool,
    paused: bool,
    core: QString,
    planet: i32,
    fps_words: QString,
    status: QString,
    slot: i32,
    speed: i32,
    master_cheats: bool,
    has_rewind: bool,
    latest_rewind_frame: f64,
    sound_volume: f64,
    resume_asked: bool,
    resume_subject: QString,
    resume_picture: QString,
    resume_when: QString,
    resume_console: QString,
    resume_core: QString,
    resume_planet: i32,
    slots: *mut ThemeRows,
    cheats: *mut ThemeRows,
    console_options: *mut ThemeRows,
    rewind_marks: *mut ThemeRows,
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
            running: false,
            paused: false,
            core: QString::default(),
            planet: -1,
            fps_words: QString::default(),
            status: QString::default(),
            slot: 1,
            speed: 100,
            master_cheats: false,
            has_rewind: false,
            latest_rewind_frame: 0.0,
            sound_volume: 1.0,
            resume_asked: false,
            resume_subject: QString::default(),
            resume_picture: QString::default(),
            resume_when: QString::default(),
            resume_console: QString::default(),
            resume_core: QString::default(),
            resume_planet: -2,
            slots: std::ptr::null_mut(),
            cheats: std::ptr::null_mut(),
            console_options: std::ptr::null_mut(),
            rewind_marks: std::ptr::null_mut(),
        }
    }
}

impl cxx_qt::Initialize for qobject::ThemeSession {
    fn initialize(mut self: Pin<&mut Self>) {
        let mut make = |roles: &'static [&'static str]| {
            let list = crate::glue::ffi::new_rows(self.as_mut().upcast_pin());
            if let Some(rows) = unsafe { list.as_mut() } {
                unsafe { Pin::new_unchecked(rows) }.set_roles(roles);
            }
            list
        };
        let (slots, cheats, console_options, rewind_marks) = (
            make(&SESSION_SLOT_ROLES),
            make(&SESSION_CHEAT_ROLES),
            make(&SESSION_OPTION_ROLES),
            make(&SESSION_REWIND_ROLES),
        );
        let mut r = self.as_mut().rust_mut();
        r.slots = slots;
        r.cheats = cheats;
        r.console_options = console_options;
        r.rewind_marks = rewind_marks;
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

    pub fn show_live(mut self: Pin<&mut Self>, facts: &SessionLiveFacts) {
        let mut changed = false;
        macro_rules! set_plain {
            ($field:ident, $val:expr) => {{
                let v = $val;
                if self.$field != v {
                    self.as_mut().rust_mut().$field = v;
                    changed = true;
                }
            }};
        }
        macro_rules! set_text {
            ($field:ident, $val:expr) => {{
                let v = QString::from($val.as_str());
                if self.$field != v {
                    self.as_mut().rust_mut().$field = v;
                    changed = true;
                }
            }};
        }
        set_plain!(running, facts.running);
        set_plain!(paused, facts.paused);
        set_text!(core, facts.core);
        set_plain!(planet, facts.planet);
        set_text!(fps_words, facts.fps_words);
        set_text!(status, facts.status);
        set_plain!(slot, facts.slot);
        set_plain!(speed, facts.speed);
        set_plain!(master_cheats, facts.master_cheats);
        set_plain!(has_rewind, facts.has_rewind);
        set_plain!(latest_rewind_frame, facts.latest_rewind_frame);
        set_plain!(sound_volume, facts.sound_volume);
        set_plain!(resume_asked, facts.resume_asked);
        set_text!(resume_subject, facts.resume_subject);
        set_text!(resume_picture, facts.resume_picture);
        set_text!(resume_when, facts.resume_when);
        set_text!(resume_console, facts.resume_console);
        set_text!(resume_core, facts.resume_core);
        set_plain!(resume_planet, facts.resume_planet);

        for (ptr, rows) in [
            (self.slots, &facts.slots),
            (self.cheats, &facts.cheats),
            (self.console_options, &facts.console_options),
            (self.rewind_marks, &facts.rewind_marks),
        ] {
            if let Some(list) = unsafe { ptr.as_mut() } {
                if list.rows() != rows.as_slice() {
                    changed = true;
                }
                unsafe { Pin::new_unchecked(list) }.replace(rows.clone());
            }
        }
        if changed {
            self.live_changed();
        }
    }
}

pub fn live_facts_of(
    data: &emusen_qt_models::desktop::DesktopPlayingData,
    sound_volume: f64,
    resume: (bool, &str, &str, &str, &str, &str, i32),
    url_map: &dyn Fn(String) -> String,
) -> SessionLiveFacts {
    use crate::view::Cell;
    let map_url = |u: &str| if u.is_empty() { String::new() } else { url_map(u.to_owned()) };
    SessionLiveFacts {
        running: data.running,
        paused: data.paused,
        core: data.core.clone(),
        planet: data.planet,
        fps_words: data.fps_words.clone(),
        status: data.status.clone(),
        slot: data.slot,
        speed: data.speed,
        master_cheats: data.master_cheats,
        has_rewind: data.has_rewind,
        latest_rewind_frame: data.latest_rewind_frame,
        sound_volume,
        resume_asked: resume.0,
        resume_subject: resume.1.to_owned(),
        resume_picture: map_url(resume.2),
        resume_when: resume.3.to_owned(),
        resume_console: resume.4.to_owned(),
        resume_core: resume.5.to_owned(),
        resume_planet: resume.6,
        slots: data
            .slots
            .iter()
            .map(|s| vec![Cell::Whole(s.slot), Cell::Flag(s.exists), Cell::Text(s.when.clone()), Cell::Text(map_url(&s.picture)), Cell::Flag(s.active)])
            .collect(),
        cheats: data
            .cheats
            .iter()
            .map(|c| vec![Cell::Whole(c.index), Cell::Text(c.description.clone()), Cell::Flag(c.on), Cell::Text(c.code.clone())])
            .collect(),
        console_options: data
            .console_options
            .iter()
            .map(|o| {
                vec![
                    Cell::Text(o.key.clone()),
                    Cell::Text(o.label.clone()),
                    Cell::Text(o.value.clone()),
                    Cell::Text(o.display_value.clone()),
                    Cell::Whole(o.choice_count),
                ]
            })
            .collect(),
        rewind_marks: data
            .rewind_marks
            .iter()
            .map(|m| vec![Cell::Number(m.frame), Cell::Text(m.label.clone()), Cell::Text(m.short.clone()), Cell::Text(map_url(&m.picture)), Cell::Flag(m.now)])
            .collect(),
    }
}
