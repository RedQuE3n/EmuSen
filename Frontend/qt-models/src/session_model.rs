//! `SessionModel`, the running game (EmuSen_Qt.md §1.2, §1.8.2): open a game, run, pause, close. session.rs's frame
//! thread put on Qt; its reports come back through qt_thread().queue, the one way in.

use crate::picture::PictureFeed;
use crate::session::{GameIo, Report, Session};
use core::pin::Pin;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;
use crate::input::GameInput;
use crate::pause::{self, Facts};
use emusen_crystal_client::{Api, Config, Runtime};
use emusen_platform_client::audio::{GameSound, SoundSettings};
use emusen_platform_client::pace::DisplayClock;
use emusen_platform_client::{galaxia, Lent, Platform};
use emusen_qt_endymion::game::GamePads;
use std::sync::Arc;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, platform_library, cxx_name = "platformLibrary", READ, WRITE = set_platform_library, NOTIFY)]
        #[qproperty(u64, feed, READ, WRITE = set_feed, NOTIFY)]
        #[qproperty(QString, cores)]
        #[qproperty(QString, engine)]
        #[qproperty(u64, pads)]
        #[qproperty(QString, game, READ, WRITE = set_game, NOTIFY)]
        #[qproperty(bool, running, READ, NOTIFY)]
        #[qproperty(bool, paused, READ, NOTIFY)]
        /// Whether the open game's sound reaches a player.
        #[qproperty(bool, sounding, READ, NOTIFY)]
        #[qproperty(QString, problem, READ, NOTIFY)]
        #[qproperty(f64, first_picture_ms, cxx_name = "firstPictureMs", READ, NOTIFY)]
        /// What the Pause Card's words are made from, read as the game pauses (pause.rs's facts as plain data).
        #[qproperty(QString, card_facts, cxx_name = "cardFacts", READ, NOTIFY)]
        type SessionModel = super::SessionModelRust;

        #[cxx_name = "setPlatformLibrary"]
        fn set_platform_library(self: Pin<&mut SessionModel>, path: QString);

        #[cxx_name = "setFeed"]
        fn set_feed(self: Pin<&mut SessionModel>, feed: u64);

        /// The game named by its path: naming one opens it, as `open` does; an empty name closes it.
        #[cxx_name = "setGame"]
        fn set_game(self: Pin<&mut SessionModel>, path: QString);

        /// Opens the game at `path` and runs it; a game already open is closed first.
        #[qinvokable]
        fn open(self: Pin<&mut SessionModel>, path: &QString);

        #[qinvokable]
        fn pause(self: Pin<&mut SessionModel>);

        #[qinvokable]
        fn resume(self: Pin<&mut SessionModel>);

        /// Paused if running, else resumed: emusen-qt-play's own window's answer to the menu chord, which has no Pause Card.
        #[qinvokable]
        #[cxx_name = "togglePause"]
        fn toggle_pause(self: Pin<&mut SessionModel>);

        #[qinvokable]
        fn close(self: Pin<&mut SessionModel>);
    }

    impl cxx_qt::Threading for SessionModel {}
}

#[derive(Default)]
pub struct SessionModelRust {
    platform_library: QString,
    feed: u64,
    cores: QString,
    engine: QString,
    pads: u64,
    game: QString,
    running: bool,
    paused: bool,
    sounding: bool,
    problem: QString,
    first_picture_ms: f64,
    card_facts: QString,
    /// The open session's number, once the frame thread has opened it.
    opened: Option<u32>,
    runtime: Option<Runtime>,
    platform: Option<Arc<Platform>>,
    session: Option<Session>,
}

impl qobject::SessionModel {
    fn set_problem(mut self: Pin<&mut Self>, value: QString) {
        if self.problem != value {
            self.as_mut().rust_mut().problem = value;
            self.problem_changed();
        }
    }

    fn set_running(mut self: Pin<&mut Self>, value: bool) {
        if self.running != value {
            self.as_mut().rust_mut().running = value;
            self.running_changed();
        }
    }

    fn set_paused(mut self: Pin<&mut Self>, value: bool) {
        if self.paused != value {
            self.as_mut().rust_mut().paused = value;
            self.paused_changed();
        }
    }

    fn set_sounding(mut self: Pin<&mut Self>, value: bool) {
        if self.sounding != value {
            self.as_mut().rust_mut().sounding = value;
            self.sounding_changed();
        }
    }

    fn set_first_picture_ms(mut self: Pin<&mut Self>, value: f64) {
        if self.first_picture_ms != value {
            self.as_mut().rust_mut().first_picture_ms = value;
            self.first_picture_ms_changed();
        }
    }

    fn open(mut self: Pin<&mut Self>, path: &QString) {
        self.as_mut().close();
        // SAFETY: the window's host names a live feed, which outlives the window and every model in it.
        let Some((handoff, device)) = (unsafe { PictureFeed::at(self.feed) }) else {
            return self.as_mut().set_problem(QString::from("No picture is named."));
        };
        if self.runtime.is_none() {
            let library = self.platform_library.to_string();
            let config = match self.as_mut().runtime_config() {
                Ok(config) => config,
                Err(words) => return self.as_mut().set_problem(QString::from(&words)),
            };
            match Api::load(&library).and_then(|api| Runtime::new(&api, &config)) {
                Ok(runtime) => self.as_mut().rust_mut().runtime = Some(runtime),
                Err(e) => return self.as_mut().set_problem(QString::from(&e.to_string())),
            }
        }
        let Some(runtime) = self.runtime.clone() else { return };
        let pads = match self.as_mut().game_input() {
            Ok(pads) => pads,
            Err(words) => return self.as_mut().set_problem(QString::from(&words)),
        };
        let (sound, unsounded) = match self.as_mut().game_sound() {
            Ok(sound) => (Some(sound), None),
            Err(words) => (None, Some(format!("The game plays without sound: {words}"))),
        };
        let sounding = sound.is_some();
        let queue = self.qt_thread();
        let report = move |report: Report| {
            let _ = queue.queue(move |model| model.reported(report));
        };
        let clock = match self.as_mut().platform() {
            Ok(platform) => DisplayClock::new(&platform),
            Err(words) => return self.as_mut().set_problem(QString::from(&words)),
        };
        let engine = (!self.engine.is_empty()).then(|| self.engine.to_string());
        let device = std::ptr::NonNull::new(device.as_ptr().cast_mut());
        let session = Session::start(runtime, path.to_string(), engine, device, handoff, GameIo { pads, sound }, clock, report);
        self.as_mut().set_sounding(sounding);
        self.as_mut().rust_mut().session = Some(session);
        self.as_mut().set_problem(QString::from(unsounded.as_deref().unwrap_or_default()));
        self.as_mut().set_paused(false);
        self.as_mut().set_first_picture_ms(0.0);
    }

    /// The cores folder a host names, else the runtime's own default (emusen_crystal.h, the config's `cores`); the
    /// player's cores folder Galaxia's either way, which a named folder would otherwise leave unscanned (EmuSen_Qt.md §1.8.7).
    fn runtime_config(self: Pin<&mut Self>) -> Result<Config, String> {
        if self.cores.is_empty() {
            return Ok(Config::default());
        }
        let cores = vec![self.cores.to_string()];
        let platform = self.platform()?;
        let player = platform.directory(galaxia::consts::DIR_CORES).map_err(|e| e.to_string())?;
        Ok(Config { cores: Some(cores), player_cores: Some(player.into_iter().map(|p| p.to_string_lossy().into_owned()).collect()), ..Config::default() })
    }

    /// The game's side of the pads a `PadReader` hands over, when one is named by its `gamePads`; the router is made on the
    /// platform library loaded once.
    fn game_input(mut self: Pin<&mut Self>) -> Result<Option<GameInput>, String> {
        // SAFETY: a window names a reader's `gamePads` while the reader lives; `at` takes a reference of its own.
        let Some(pads) = (unsafe { GamePads::at(self.pads) }) else { return Ok(None) };
        let platform = self.as_mut().platform()?;
        GameInput::new(pads, &platform).map(Some).map_err(|e| e.to_string())
    }

    /// The game's sound player on SDL lent to the library, at audio.json's defaults until the settings are read (§1.8.6).
    fn game_sound(self: Pin<&mut Self>) -> Result<GameSound, String> {
        let platform = self.platform()?;
        let words = |e: emusen_platform_client::Error| e.to_string();
        platform.lend_sdl(&Lent::open("libSDL3.so.0").map_err(words)?).map_err(words)?;
        GameSound::new(&platform, SoundSettings::default()).map_err(words)
    }

    /// The platform library, loaded once.
    fn platform(mut self: Pin<&mut Self>) -> Result<Arc<Platform>, String> {
        if self.platform.is_none() {
            let platform = Platform::load(self.platform_library.to_string()).map_err(|e| e.to_string())?;
            self.as_mut().rust_mut().platform = Some(platform);
        }
        self.platform.clone().ok_or_else(|| "no platform library".to_string())
    }

    fn set_platform_library(mut self: Pin<&mut Self>, path: QString) {
        if self.platform_library != path {
            self.as_mut().rust_mut().platform_library = path;
            self.as_mut().platform_library_changed();
            self.start_named();
        }
    }

    fn set_feed(mut self: Pin<&mut Self>, feed: u64) {
        if self.feed != feed {
            self.as_mut().rust_mut().feed = feed;
            self.as_mut().feed_changed();
            self.start_named();
        }
    }

    fn set_game(mut self: Pin<&mut Self>, path: QString) {
        if self.game != path {
            self.as_mut().rust_mut().game = path;
            self.as_mut().game_changed();
            self.start_named();
        }
    }

    /// The game the properties name opened once all three are named, whatever order QML sets them in; none closes it.
    fn start_named(mut self: Pin<&mut Self>) {
        let game = self.game.clone();
        if game.is_empty() {
            self.close();
        } else if self.feed != 0 && !self.platform_library.is_empty() {
            self.as_mut().open(&game);
        }
    }

    fn pause(mut self: Pin<&mut Self>) {
        if let Some(session) = &self.session {
            session.set_paused(true);
            self.as_mut().set_paused(true);
            self.read_card_facts();
        }
    }

    /// The Pause Card's facts read from the runtime, whose reads answer on any thread (emusen_crystal.h). The slot and
    /// the speed are the defaults until the card steps them (owed, EmuSen_Qt.md §7.1 step 7).
    fn read_card_facts(mut self: Pin<&mut Self>) {
        let (Some(runtime), Some(session)) = (self.runtime.clone(), self.opened) else { return };
        let facts = Facts::default();
        let facts = match runtime.session_info(session) {
            Ok(info) => {
                let slots = runtime.state_slots(session).unwrap_or_default();
                pause::facts_of(&info, runtime.rewind_reel(session).ok().as_ref(), &slots, facts.slot, facts.speed)
            }
            Err(_) => facts,
        };
        let text = QString::from(facts.to_json().as_str());
        if self.card_facts != text {
            self.as_mut().rust_mut().card_facts = text;
            self.card_facts_changed();
        }
    }

    fn resume(mut self: Pin<&mut Self>) {
        if let Some(session) = &self.session {
            session.set_paused(false);
            self.as_mut().set_paused(false);
        }
    }

    fn toggle_pause(self: Pin<&mut Self>) {
        if self.paused {
            self.resume();
        } else {
            self.pause();
        }
    }

    fn close(mut self: Pin<&mut Self>) {
        drop(self.as_mut().rust_mut().session.take());
        self.as_mut().rust_mut().opened = None;
        self.as_mut().set_running(false);
        self.as_mut().set_paused(false);
        self.as_mut().set_sounding(false);
    }

    fn reported(mut self: Pin<&mut Self>, report: Report) {
        match report {
            Report::Opened { session } => {
                self.as_mut().rust_mut().opened = Some(session);
                self.as_mut().set_running(true);
            }
            Report::FirstPicture { after } => self.as_mut().set_first_picture_ms(after.as_secs_f64() * 1000.0),
            Report::Failed(words) => {
                self.as_mut().set_problem(QString::from(&words));
                self.as_mut().set_running(false);
            }
            Report::Closed => self.as_mut().set_running(false),
        }
    }
}
