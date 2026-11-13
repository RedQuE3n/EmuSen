//! `SessionModel`, the running game (EmuSen_Qt.md §1.2, §1.8.2): open a game, run, pause, close. session.rs's frame
//! thread put on Qt; its reports come back through qt_thread().queue, the one way in.

use crate::display::{self, Display};
use crate::picture::{Handoff, PictureFeed};
use crate::settings::{Model, SettingsSource};
use crate::settings_source::ClientSettings;
use crate::session::{GameIo, Opening, Report, Session};
use core::pin::Pin;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{DateFormat, QDateTime, QRectF, QString};
use crate::input::GameInput;
use crate::pause::{self, SlotFact};
use crate::resume::{self, Answer, Start};
use crate::pictures::{self, PictureSource, Shown};
use cxx_qt::casting::Upcast;
use emusen_crystal_client::{Api, Config, Runtime};
use crate::play::PlaySettings;
use emusen_platform_client::audio::{GameSound, SoundSettings};
use emusen_platform_client::pace::DisplayClock;
use emusen_platform_client::{galaxia, Lent, Platform};
use emusen_qt_endymion::game::{GamePads, KeyUse};
use emusen_qt_endymion::hotkeys::Hotkey;
use emusen_qt_endymion::keys;
use std::sync::Arc;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qrectf.h");
        type QRectF = cxx_qt_lib::QRectF;
        include!("emusen-kit-qt/src/kinds.cxxqt.h");
        #[namespace = "Pad"]
        type PadButton = emusen_kit_qt::kinds::ffi::PadButton;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, platform_library, cxx_name = "platformLibrary", READ, WRITE = set_platform_library, NOTIFY)]
        #[qproperty(u64, feed, READ, WRITE = set_feed, NOTIFY)]
        #[qproperty(QString, cores)]
        #[qproperty(QString, engine)]
        #[qproperty(u64, pads)]
        /// The player's Rumble switch as Settings binds it, which a running game takes at once (EmuSen_Input.md §9.7); a game
        /// opened reads the setting itself, so the value before Settings has read it acts on nothing.
        #[qproperty(bool, rumble, READ, WRITE = set_rumble, NOTIFY)]
        /// The game's Volume, Output as stored, Sync to display and Pause in background as Settings binds them, which a
        /// running game takes at once; a game opened reads the settings itself (EmuSen_Qt.md §1.8.6).
        #[qproperty(f64, volume, READ, WRITE = set_volume, NOTIFY)]
        #[qproperty(QString, output, READ, WRITE = set_output, NOTIFY)]
        #[qproperty(bool, sync_to_display, cxx_name = "syncToDisplay", READ, WRITE = set_sync_to_display, NOTIFY)]
        #[qproperty(bool, pause_in_background, cxx_name = "pauseInBackground", READ, WRITE = set_pause_in_background, NOTIFY)]
        /// Whether another window is in front of the program's: a game running then is paused where Pause in background
        /// says so, and resumed when the program is in front again.
        #[qproperty(bool, background, READ, WRITE = set_background, NOTIFY)]
        /// The open game's sound player as it holds itself: its volume, and the device it plays on, empty for the system default.
        #[qproperty(f64, sound_volume, cxx_name = "soundVolume", READ, NOTIFY)]
        #[qproperty(QString, sound_device, cxx_name = "soundDevice", READ, NOTIFY)]
        /// Frames run per second over the last second the game ran.
        #[qproperty(f64, fps, READ, NOTIFY)]
        /// That rate as the line over the game says it.
        #[qproperty(QString, fps_words, cxx_name = "fpsWords", READ, NOTIFY)]
        /// The status line's words: the game's name, Paused, or the last thing a save, load or failure said.
        #[qproperty(QString, status, READ, NOTIFY)]
        #[qproperty(QString, game, READ, WRITE = set_game, NOTIFY)]
        /// The console started with nothing in it (`openConsole`), by the name it was started by; empty for a game.
        #[qproperty(QString, console, READ, NOTIFY)]
        /// The saved state the next game named starts from, by its file, as a game's page asks (EmuSen_Qt.md §7.8);
        /// taken by that start, and given to the open game when it is the one named. Empty starts as Resume says.
        #[qproperty(QString, start_state, cxx_name = "startState")]
        #[qproperty(bool, running, READ, NOTIFY)]
        #[qproperty(bool, paused, READ, NOTIFY)]
        /// Whether the open game's sound reaches a player.
        #[qproperty(bool, sounding, READ, NOTIFY)]
        #[qproperty(QString, problem, READ, NOTIFY)]
        #[qproperty(f64, first_picture_ms, cxx_name = "firstPictureMs", READ, NOTIFY)]
        /// The picture's width over its height as shown; 0 before the first picture, where it fills its item.
        #[qproperty(f64, picture_aspect, cxx_name = "pictureAspect", READ, NOTIFY)]
        /// The part of the drawn picture the window shows, each a share of it: the crop a filter does not make itself.
        #[qproperty(QRectF, picture_crop, cxx_name = "pictureCrop", READ, NOTIFY)]
        /// What the Pause Card's words are made from, read as the game pauses (pause.rs's facts as plain data).
        #[qproperty(QString, card_facts, cxx_name = "cardFacts", READ, NOTIFY)]
        /// The state slot Save and Load act on, 1 to 8, kept while the program runs.
        #[qproperty(u32, slot, READ, NOTIFY)]
        /// The game's speed in percent of its own, the player's choice kept while the program runs; 0 unthrottled.
        #[qproperty(u32, speed, READ, NOTIFY)]
        /// What the last save, load or rewind came to, in the player's words.
        #[qproperty(QString, notice, READ, NOTIFY)]
        // The session's frame as its service reads it, asked when read: for a test to count frames run and gone back.
        #[qproperty(u64, frame, READ = frame_now)]
        /// Whether that notice says something was not done.
        #[qproperty(bool, notice_attention, cxx_name = "noticeAttention", READ, NOTIFY)]
        /// Whether the Resume question is waiting for an answer, the game held unplayed (EmuSen_Interface.md §18.3).
        #[qproperty(bool, resume_asked, cxx_name = "resumeAsked", READ, NOTIFY)]
        /// The game the Resume question names.
        #[qproperty(QString, resume_subject, cxx_name = "resumeSubject", READ, NOTIFY)]
        type SessionModel = super::SessionModelRust;

        #[cxx_name = "setPlatformLibrary"]
        fn set_platform_library(self: Pin<&mut SessionModel>, path: QString);

        #[cxx_name = "setFeed"]
        fn set_feed(self: Pin<&mut SessionModel>, feed: u64);

        #[cxx_name = "setRumble"]
        fn set_rumble(self: Pin<&mut SessionModel>, on: bool);

        #[cxx_name = "setVolume"]
        fn set_volume(self: Pin<&mut SessionModel>, volume: f64);

        #[cxx_name = "setOutput"]
        fn set_output(self: Pin<&mut SessionModel>, output: QString);

        #[cxx_name = "setSyncToDisplay"]
        fn set_sync_to_display(self: Pin<&mut SessionModel>, on: bool);

        #[cxx_name = "setPauseInBackground"]
        fn set_pause_in_background(self: Pin<&mut SessionModel>, on: bool);

        #[cxx_name = "setBackground"]
        fn set_background(self: Pin<&mut SessionModel>, on: bool);

        /// The game named by its path: naming one opens it, as `open` does; an empty name closes it.
        #[cxx_name = "setGame"]
        fn set_game(self: Pin<&mut SessionModel>, path: QString);

        /// Opens the game at `path` and runs it; a game already open is closed first.
        #[qinvokable]
        fn open(self: Pin<&mut SessionModel>, path: &QString);

        /// Starts a console with nothing in it, by its key (Start Console); a game already open is closed first, and no
        /// game is named.
        #[qinvokable]
        #[cxx_name = "openConsole"]
        fn open_console(self: Pin<&mut SessionModel>, console: &QString);

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

        /// The slot Save and Load act on.
        #[qinvokable]
        #[cxx_name = "chooseSlot"]
        fn choose_slot(self: Pin<&mut SessionModel>, slot: u32);

        /// The game's speed chosen, applied to the running game at once and to each game opened after.
        #[qinvokable]
        #[cxx_name = "chooseSpeed"]
        fn choose_speed(self: Pin<&mut SessionModel>, percent: u32);

        /// The display settings of the open game's console read again (Settings saved, the game resumed): the picture is
        /// drawn through what they ask for from its next draw, and its shape and crop follow.
        #[qinvokable]
        #[cxx_name = "readPicture"]
        fn read_picture(self: Pin<&mut SessionModel>);

        /// A lever of `console`'s shader moved to `value`: set on the picture's chain at once when the game drawn is of
        /// that console and its chain declares `id`, else the picture read again from the settings saved.
        #[qinvokable]
        #[cxx_name = "setLever"]
        fn set_lever(self: Pin<&mut SessionModel>, console: &QString, id: &QString, value: f64);

        /// The picture's settings read anew: the picture is drawn again, a still one too.
        #[qsignal]
        #[cxx_name = "pictureRead"]
        fn picture_read(self: Pin<&mut SessionModel>);

        /// A game that ran has stopped: its path and how long it ran, in seconds, which the library records as play time
        /// under the player's limit (LibraryModel.played).
        #[qsignal]
        fn played(self: Pin<&mut SessionModel>, path: QString, seconds: f64);

        /// The game or console named did not open, or stopped on a failure: `problem` says why.
        #[qsignal]
        #[cxx_name = "openFailed"]
        fn open_failed(self: Pin<&mut SessionModel>);

        /// The game saved to a slot, off the GUI thread; a resume waits for it.
        #[qinvokable]
        #[cxx_name = "saveState"]
        fn save_state(self: Pin<&mut SessionModel>, slot: u32);

        /// A slot loaded into the game, off the GUI thread; a resume waits for it.
        #[qinvokable]
        #[cxx_name = "loadState"]
        fn load_state(self: Pin<&mut SessionModel>, slot: u32);

        /// The picture on show written to the screenshots folder, off the GUI thread.
        #[qinvokable]
        fn screenshot(self: Pin<&mut SessionModel>);

        /// The game started again from the beginning, off the GUI thread; a resume waits for it.
        #[qinvokable]
        fn restart(self: Pin<&mut SessionModel>);

        /// The game returned to the reel's moment at `frame`, off the GUI thread; a resume waits for it.
        #[qinvokable]
        #[cxx_name = "rewindTo"]
        fn rewind_to(self: Pin<&mut SessionModel>, frame: u64);

        /// The Resume question answered, by the index the question gives: Continue, Start again, or -1 for B.
        #[qinvokable]
        #[cxx_name = "answerResume"]
        fn answer_resume(self: Pin<&mut SessionModel>, index: i32);

        /// The game's own disc of that index put in, off the GUI thread; a resume waits for it.
        #[qinvokable]
        #[cxx_name = "changeDisc"]
        fn change_disc(self: Pin<&mut SessionModel>, index: u32);

        /// A disc image by its file put in, off the GUI thread.
        #[qinvokable]
        #[cxx_name = "insertDisc"]
        fn insert_disc(self: Pin<&mut SessionModel>, path: &QString);

        /// The tray opened with no disc in, off the GUI thread.
        #[qinvokable]
        #[cxx_name = "ejectDisc"]
        fn eject_disc(self: Pin<&mut SessionModel>);

        /// Players 1 and 2 swap pads, said as a notice.
        #[qinvokable]
        #[cxx_name = "swapPlayers"]
        fn swap_players(self: Pin<&mut SessionModel>);

        /// A setting of the running console stored, for its games from the next start; the card's facts read again.
        #[qinvokable]
        #[cxx_name = "setConsoleSetting"]
        fn set_console_setting(self: Pin<&mut SessionModel>, key: &QString, value: &QString);

        /// The game's cheat at `index` of its list switched in the running game at once and saved to the list; the
        /// card's facts read again.
        #[qinvokable]
        #[cxx_name = "switchCheat"]
        fn switch_cheat(self: Pin<&mut SessionModel>, index: i32, on: bool);

        /// Guide held with `button` over the running game (`PadRouter.guideChord`): pause.rs's `guide_chord` says what it asks.
        #[qinvokable]
        #[cxx_name = "guideChord"]
        fn guide_chord(self: Pin<&mut SessionModel>, button: PadButton);

        /// A key pressed or let go over the running game, by Qt's number and modifiers: a control of the console's for
        /// the game, or a hotkey's action (EmuSen_Qt.md §7.7.H6); whether the key was the game's or a hotkey's.
        #[qinvokable]
        fn key(self: Pin<&mut SessionModel>, key: i32, modifiers: i32, pressed: bool, repeated: bool) -> bool;

        /// A key no screen took away from the game (over the library, Settings or a card), by Qt's number: a hotkey's
        /// action as Mistress ran it there (EmuSen_Qt.md §7.7.H8); `library` false over the Pause Card, the game under
        /// it. Whether the key was a hotkey's.
        #[qinvokable]
        #[cxx_name = "awayKey"]
        fn away_key(self: Pin<&mut SessionModel>, key: i32, modifiers: i32, pressed: bool, repeated: bool, library: bool) -> bool;

        #[rust_name = "frame_now"]
        fn frameNow(self: &SessionModel) -> u64;

        /// The Exit to Library hotkey over the library with a game suspended: back to the game, as Mistress's
        /// `ToggleLibrary` returns to it.
        #[qsignal]
        #[cxx_name = "gameAsked"]
        fn game_asked(self: Pin<&mut SessionModel>);

        /// The Exit to Library hotkey, whose action is the window's.
        #[qsignal]
        #[cxx_name = "exitAsked"]
        fn exit_asked(self: Pin<&mut SessionModel>);

        /// The Fullscreen hotkey, whose action is the window's.
        #[qsignal]
        #[cxx_name = "fullScreenAsked"]
        fn full_screen_asked(self: Pin<&mut SessionModel>);
    }

    impl cxx_qt::Threading for SessionModel {}
}

pub struct SessionModelRust {
    platform_library: QString,
    feed: u64,
    cores: QString,
    engine: QString,
    pads: u64,
    rumble: bool,
    volume: f64,
    output: QString,
    sync_to_display: bool,
    pause_in_background: bool,
    background: bool,
    sound_volume: f64,
    sound_device: QString,
    fps: f64,
    fps_words: QString,
    status: QString,
    /// Whether the game was paused because the program went to the background, so it resumes as it comes back.
    backgrounded: bool,
    /// Output's own initial value, the system default, read once from Galaxia's model.
    output_default: Option<String>,
    game: QString,
    console: QString,
    start_state: QString,
    running: bool,
    paused: bool,
    sounding: bool,
    problem: QString,
    /// What a failure of the opening is said after, Mistress's "Failed to load {name}" or "Failed to start {console}".
    failing: String,
    first_picture_ms: f64,
    picture_aspect: f64,
    picture_crop: QRectF,
    /// The frame's shape on its console's screen and with its pixels square, as the frame thread last reported them.
    shapes: (f64, f64),
    /// The open session's console, and what its display settings ask for.
    display_console: Option<String>,
    display: Display,
    /// Serenity's catalogue, read once.
    catalogue: Option<serde_json::Value>,
    /// The feed's hand-off, to ask for the picture's preset.
    handoff: Option<Handoff>,
    card_facts: QString,
    slot: u32,
    speed: u32,
    notice: QString,
    notice_attention: bool,
    resume_asked: bool,
    resume_subject: QString,
    /// The setting Resume while the opened game is held for it, until the runtime says whether it was left off.
    holding: Option<resume::Setting>,
    /// The state the opened game is held to start from, until the runtime has opened it.
    from_state: Option<String>,
    /// The session the card's pictures are read from, shared with the picture provider.
    shown: Shown,
    /// Read again at each pause, so a picture's address names the pause it was read in.
    serial: u64,
    /// Whether the picture provider is served in this model's engine.
    served: bool,
    /// Saves, loads and rewinds under way off the GUI thread, and a resume asked while they run.
    jobs: u32,
    resume_waiting: bool,
    /// The open session's number, once the frame thread has opened it.
    opened: Option<u32>,
    runtime: Option<Runtime>,
    platform: Option<Arc<Platform>>,
    session: Option<Session>,
    /// The game's own cheat list as given to the open session.
    cheats: Option<crate::cheats::Given>,
    /// The runtime holding the game after a Frame Advance, until a resume lets it run (Mistress's `AdvanceOneFrame`).
    stepped: bool,
    /// The game running and when it began to, for the play time recorded when it stops.
    playing: Option<(QString, std::time::Instant)>,
}

impl Default for SessionModelRust {
    fn default() -> Self {
        SessionModelRust {
            platform_library: QString::default(),
            feed: 0,
            cores: QString::default(),
            engine: QString::default(),
            pads: 0,
            rumble: false,
            volume: 1.0,
            output: QString::default(),
            sync_to_display: true,
            pause_in_background: true,
            background: false,
            sound_volume: 0.0,
            sound_device: QString::default(),
            fps: 0.0,
            fps_words: QString::default(),
            status: QString::default(),
            backgrounded: false,
            output_default: None,
            game: QString::default(),
            console: QString::default(),
            start_state: QString::default(),
            running: false,
            paused: false,
            sounding: false,
            problem: QString::default(),
            failing: String::new(),
            first_picture_ms: 0.0,
            picture_aspect: 0.0,
            picture_crop: QRectF::new(0.0, 0.0, 1.0, 1.0),
            shapes: (0.0, 0.0),
            display_console: None,
            display: Display::default(),
            catalogue: None,
            handoff: None,
            card_facts: QString::default(),
            slot: 1,
            speed: 100,
            notice: QString::default(),
            notice_attention: false,
            resume_asked: false,
            resume_subject: QString::default(),
            holding: None,
            from_state: None,
            shown: Shown::default(),
            serial: 0,
            served: false,
            jobs: 0,
            resume_waiting: false,
            opened: None,
            runtime: None,
            platform: None,
            session: None,
            cheats: None,
            stepped: false,
            playing: None,
        }
    }
}

/// A state's time as ISO 8601, worded in the player's time as Mistress words a slot's ("Sun 8 Nov, 21:42").
fn when_words(saved_at: Option<&str>) -> String {
    let Some(at) = saved_at.and_then(|at| QDateTime::from_string(&QString::from(at), DateFormat::ISODateWithMs)) else {
        return String::new();
    };
    let local = at.to_local_time();
    format!("{}, {}", local.date().format(&QString::from("ddd d MMM")), local.time().format(&QString::from("HH:mm")))
}

impl qobject::SessionModel {
    fn set_problem(mut self: Pin<&mut Self>, value: QString) {
        if self.problem != value {
            self.as_mut().rust_mut().problem = value.clone();
            self.as_mut().problem_changed();
            if !value.is_empty() && self.session.is_some() {
                self.set_status(value.to_string());
            }
        }
    }

    /// The opening failed: said in Mistress's form, after what was being opened.
    fn failed(mut self: Pin<&mut Self>, words: String) {
        let said = format!("{}: {words}", self.failing);
        self.as_mut().set_problem(QString::from(&said));
        // Named no longer, so choosing the same game again opens it afresh.
        if !self.game.is_empty() {
            self.as_mut().rust_mut().game = QString::default();
            self.as_mut().game_changed();
        }
        self.open_failed();
    }

    fn set_running(mut self: Pin<&mut Self>, value: bool) {
        if self.running == value {
            return;
        }
        let game = self.game.clone();
        let ran = if value {
            self.as_mut().rust_mut().playing = Some((game, std::time::Instant::now())).filter(|(g, _)| !g.is_empty());
            None
        } else {
            self.as_mut().rust_mut().playing.take()
        };
        self.as_mut().rust_mut().running = value;
        self.as_mut().running_changed();
        if let Some((path, since)) = ran {
            self.played(path, since.elapsed().as_secs_f64());
        }
    }

    fn set_paused(mut self: Pin<&mut Self>, value: bool) {
        if self.paused != value {
            self.as_mut().rust_mut().paused = value;
            self.as_mut().paused_changed();
            if self.session.is_some() {
                let words = if value { pause::PAUSED.to_owned() } else { self.as_mut().title() };
                self.set_status(words);
            }
        }
    }

    /// The open game's name as the status line says it: its title, or the console started empty.
    fn title(self: Pin<&mut Self>) -> String {
        if self.console.is_empty() { crate::library::title_of(&self.game.to_string()) } else { crate::session::console_session_name(&self.console.to_string()) }
    }

    fn set_status(mut self: Pin<&mut Self>, words: String) {
        let words = QString::from(&words);
        if self.status != words {
            self.as_mut().rust_mut().status = words;
            self.status_changed();
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

    fn open(self: Pin<&mut Self>, path: &QString) {
        self.start(Opening::Game(path.to_string()));
    }

    fn open_console(mut self: Pin<&mut Self>, console: &QString) {
        if !self.game.is_empty() {
            // No game is named, so the game open before is opened afresh when it is named again.
            self.as_mut().rust_mut().game = QString::default();
            self.as_mut().game_changed();
        }
        self.start(Opening::Console(console.to_string()));
    }

    fn start(mut self: Pin<&mut Self>, opening: Opening) {
        self.as_mut().close();
        // Cleared first, so the same failure twice is said twice.
        self.as_mut().set_problem(QString::default());
        self.as_mut().rust_mut().failing = crate::session::failing(&opening);
        let console = QString::from(match &opening {
            Opening::Console(console) => console.as_str(),
            Opening::Game(_) => "",
        });
        if self.console != console {
            self.as_mut().rust_mut().console = console;
            self.as_mut().console_changed();
        }
        // SAFETY: the window's host names a live feed, which outlives the window and every model in it.
        let Some((handoff, device)) = (unsafe { PictureFeed::at(self.feed) }) else {
            return self.as_mut().failed("No picture is named.".to_owned());
        };
        self.as_mut().rust_mut().handoff = Some(handoff.clone());
        self.as_mut().rust_mut().display_console = None;
        self.as_mut().rust_mut().display = Display::default();
        // Said once, as the runtime is made: a runtime lent no SQLite keeps no records of states (§7.9).
        let mut unkept = None;
        if self.runtime.is_none() {
            let library = self.platform_library.to_string();
            let config = match self.as_mut().runtime_config() {
                Ok(config) => config,
                Err(words) => return self.as_mut().failed(words),
            };
            match Api::load(&library).map_err(|e| e.to_string()).and_then(|api| crate::clients::runtime(&api, &config)) {
                Ok((runtime, note)) => {
                    self.as_mut().rust_mut().runtime = Some(runtime);
                    unkept = note;
                }
                Err(e) => return self.as_mut().failed(e.to_string()),
            }
        }
        let Some(runtime) = self.runtime.clone() else { return };
        // Cores changed since the last game are listed; a failed listing is said and the game opens (EmuSen_Qt.md §7.7, §7.9).
        let unlisted = runtime.rescan().err().map(|e| format!("The engines could not be listed again, so one approved or installed since the last game may not be used: {e}"));
        let pads = match self.as_mut().game_input() {
            Ok(pads) => pads,
            Err(words) => return self.as_mut().failed(words),
        };
        // The settings a game takes as it opens; unread, the last ones bound stand, and the failure is said.
        let unread = self.as_mut().read_play().err().map(|e| format!("The sound and display settings could not be read: {e}"));
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
            Err(words) => return self.as_mut().failed(words),
        };
        let engine = (!self.engine.is_empty()).then(|| self.engine.to_string());
        let device = std::ptr::NonNull::new(device.as_ptr().cast_mut());
        // A state chosen is the start's answer: the Resume question is not asked (Mistress's `PlayFromPageAsync`).
        let from_state = Some(std::mem::take(&mut self.as_mut().rust_mut().start_state).to_string()).filter(|s| !s.is_empty());
        // A console started empty has no game to resume, so nothing is asked (Mistress's `StartConsole`).
        let from_state = from_state.filter(|_| matches!(opening, Opening::Game(_)));
        let setting = if matches!(opening, Opening::Game(_)) { self.as_mut().resume_setting() } else { resume::Setting::Never };
        let holding = (from_state.is_none() && setting != resume::Setting::Never).then_some(setting);
        self.as_mut().rust_mut().holding = holding;
        let held = holding.is_some() || from_state.is_some();
        self.as_mut().rust_mut().from_state = from_state;
        let subject = match &opening {
            Opening::Game(path) => resume::subject(path),
            Opening::Console(_) => String::new(),
        };
        self.as_mut().set_resume_subject(QString::from(&subject));
        let sync = self.sync_to_display;
        let cheats = self.as_mut().platform().ok();
        let session = Session::start(runtime, opening, engine, device, handoff, GameIo { pads, sound, sync, cheats }, clock, held, report);
        self.as_mut().set_sounding(sounding);
        self.as_mut().rust_mut().session = Some(session);
        let title = self.as_mut().title();
        self.as_mut().set_status(title);
        let said: Vec<String> = [unkept, unlisted, unread, unsounded].into_iter().flatten().collect();
        self.as_mut().set_problem(QString::from(&said.join(" ")));
        self.as_mut().set_paused(false);
        self.as_mut().set_first_picture_ms(0.0);
    }

    /// The setting Resume from the player's settings, Galaxia's model; asking where they cannot be read.
    fn resume_setting(self: Pin<&mut Self>) -> resume::Setting {
        let settings = self.platform().ok().and_then(|p| p.model_load(galaxia::consts::MODEL_APP_SETTINGS, None, "appsettings.json").ok().flatten());
        resume::Setting::read(settings.as_ref().and_then(|s| s.get(resume::KEY)).and_then(|v| v.as_str()).unwrap_or_default())
    }

    /// Where the opened game was left off decides its start: from the beginning, there without asking, or the question.
    fn left_off(mut self: Pin<&mut Self>, session: u32) {
        if let Some(state) = self.as_mut().rust_mut().from_state.take() {
            if !self.paused {
                self.as_mut().rust_mut().resume_waiting = true;
            }
            return self.start_from(state);
        }
        let Some(setting) = self.as_mut().rust_mut().holding else { return };
        let exists = self.runtime.as_ref().and_then(|r| r.state_slots(session).ok()).is_some_and(|slots| slots.iter().any(|s| s.slot == 0 && s.exists));
        match resume::start(setting, exists) {
            Start::Fresh => self.as_mut().let_go(false),
            Start::Load => self.as_mut().let_go(true),
            Start::Ask => self.as_mut().set_resume_asked(true),
        }
    }

    /// The held game let go, from where it was left off or the beginning, and run unless the player paused it meanwhile.
    fn let_go(mut self: Pin<&mut Self>, load: bool) {
        self.as_mut().rust_mut().holding = None;
        self.as_mut().set_resume_asked(false);
        if !self.paused {
            self.as_mut().rust_mut().resume_waiting = true;
        }
        if load {
            self.as_mut().job(|runtime, session| match runtime.resume_load(session) {
                Ok(_) => Ok(String::new()),
                Err(e) => Err(format!("Could not resume, started from the beginning: {e}")),
            });
        } else if std::mem::take(&mut self.as_mut().rust_mut().resume_waiting) {
            self.resume();
        }
    }

    /// The open game loaded from a saved state by its file: the runtime's slot whose file it is, the resume state's
    /// as slot 0; a resume waits for it.
    fn start_from(self: Pin<&mut Self>, state: String) {
        self.job(move |runtime, session| {
            let slots = runtime.state_slots(session).map_err(|e| format!("Could not load the state, started from the beginning: {e}"))?;
            let Some(slot) = slots.iter().find(|s| s.path == state).map(|s| s.slot) else {
                return Err("Could not load the state, started from the beginning: it is not one of this game's".to_string());
            };
            let loaded = if slot == 0 { runtime.resume_load(session) } else { runtime.state_slot_load(session, slot) };
            match loaded {
                Ok(true) => Ok(String::new()),
                Ok(false) => Err("Could not load the state, started from the beginning: it is gone".to_string()),
                Err(e) => Err(format!("Could not load the state, started from the beginning: {e}")),
            }
        });
    }

    fn answer_resume(mut self: Pin<&mut Self>, index: i32) {
        if !self.resume_asked {
            return;
        }
        match resume::answer(index) {
            Answer::Load => self.let_go(true),
            Answer::Fresh => self.let_go(false),
            // The game let go unplayed: no longer named, so choosing it again opens it again.
            Answer::Back => {
                self.as_mut().set_resume_asked(false);
                self.set_game(QString::default());
            }
        }
    }

    fn set_resume_asked(mut self: Pin<&mut Self>, value: bool) {
        if self.resume_asked != value {
            self.as_mut().rust_mut().resume_asked = value;
            self.resume_asked_changed();
        }
    }

    fn set_resume_subject(mut self: Pin<&mut Self>, value: QString) {
        if self.resume_subject != value {
            self.as_mut().rust_mut().resume_subject = value;
            self.resume_subject_changed();
        }
    }

    /// The cores folder a host names, else the runtime's own default (emusen_crystal.h, the config's `cores`); the
    /// player's cores folder Galaxia's either way, which a named folder would otherwise leave unscanned (EmuSen_Qt.md §1.8.7).
    fn runtime_config(self: Pin<&mut Self>) -> Result<Config, String> {
        let cores = self.cores.to_string();
        let platform = self.platform()?;
        crate::clients::runtime_config(&platform, &cores)
    }

    /// The game's side of the pads a `PadReader` hands over, when one is named by its `gamePads`; the router is made on the
    /// platform library loaded once.
    fn game_input(mut self: Pin<&mut Self>) -> Result<Option<GameInput>, String> {
        // SAFETY: a window names a reader's `gamePads` while the reader lives; `at` takes a reference of its own.
        let Some(pads) = (unsafe { GamePads::at(self.pads) }) else { return Ok(None) };
        let platform = self.as_mut().platform()?;
        GameInput::new(pads, &platform).map(Some).map_err(|e| e.to_string())
    }

    /// The settings a game takes as it opens, through Settings' own reading (`play`), the properties set from them.
    fn read_play(mut self: Pin<&mut Self>) -> Result<(), String> {
        let platform = self.as_mut().platform()?;
        let play = PlaySettings::read(&ClientSettings::on(platform.clone()))?;
        if self.output_default.is_none() {
            let description: serde_json::Value = serde_json::from_str(crate::settings::DESCRIPTION).map_err(|e| e.to_string())?;
            let initial = platform.model_new(galaxia::consts::MODEL_APP_SETTINGS).map_err(|e| e.to_string())?;
            let graphics = platform.model_new(galaxia::consts::MODEL_GRAPHICS_CONFIG).map_err(|e| e.to_string())?;
            self.as_mut().rust_mut().output_default = Some(PlaySettings::of(&description, &initial, &graphics).output);
        }
        self.as_mut().set_volume(play.volume);
        self.as_mut().set_output(QString::from(&play.output));
        self.as_mut().set_sync_to_display(play.sync_to_display);
        self.set_pause_in_background(play.pause_in_background);
        Ok(())
    }

    /// The device Output names, none for its initial value, the system default.
    fn device(&self) -> Option<String> {
        crate::play::device(&self.output.to_string(), self.output_default.as_deref().unwrap_or_default())
    }

    /// The game's sound player on SDL lent to the library, at audio.json's defaults, on the device and at the volume
    /// the settings give (§1.8.6).
    fn game_sound(mut self: Pin<&mut Self>) -> Result<GameSound, String> {
        let platform = self.as_mut().platform()?;
        let words = |e: emusen_platform_client::Error| e.to_string();
        platform.lend_sdl(&Lent::open("libSDL3.so.0").map_err(words)?).map_err(words)?;
        let mut sound = GameSound::new(&platform, SoundSettings::default()).map_err(words)?;
        sound.set_volume(self.volume as f32).map_err(words)?;
        let device = crate::session::play_on(&mut sound, self.device())?;
        self.as_mut().heard_volume(sound.volume());
        self.heard_device(device);
        Ok(sound)
    }

    fn heard_volume(mut self: Pin<&mut Self>, volume: f32) {
        let volume = f64::from(volume);
        if self.sound_volume != volume {
            self.as_mut().rust_mut().sound_volume = volume;
            self.sound_volume_changed();
        }
    }

    fn heard_device(mut self: Pin<&mut Self>, device: Option<String>) {
        let device = QString::from(&device.unwrap_or_default());
        if self.sound_device != device {
            self.as_mut().rust_mut().sound_device = device;
            self.sound_device_changed();
        }
    }

    fn set_volume(mut self: Pin<&mut Self>, volume: f64) {
        if self.volume != volume {
            self.as_mut().rust_mut().volume = volume;
            self.as_mut().volume_changed();
        }
        if let Some(session) = &self.session {
            session.set_volume(volume as f32);
        }
    }

    fn set_output(mut self: Pin<&mut Self>, output: QString) {
        if self.output == output {
            return;
        }
        self.as_mut().rust_mut().output = output;
        self.as_mut().output_changed();
        if let Some(session) = &self.session {
            session.set_device(self.device());
        }
    }

    fn set_sync_to_display(mut self: Pin<&mut Self>, on: bool) {
        if self.sync_to_display != on {
            self.as_mut().rust_mut().sync_to_display = on;
            self.as_mut().sync_to_display_changed();
        }
        if let Some(session) = &self.session {
            session.set_sync(on);
        }
    }

    fn set_pause_in_background(mut self: Pin<&mut Self>, on: bool) {
        if self.pause_in_background != on {
            self.as_mut().rust_mut().pause_in_background = on;
            self.pause_in_background_changed();
        }
    }

    /// The program gone to the background pauses a game running in front, where Pause in background is on; come back,
    /// the game it paused runs again, and one paused before stays paused.
    fn set_background(mut self: Pin<&mut Self>, on: bool) {
        if self.background == on {
            return;
        }
        self.as_mut().rust_mut().background = on;
        self.as_mut().background_changed();
        if on {
            let playing = self.session.is_some() && !self.paused && !self.stepped && self.holding.is_none() && self.from_state.is_none();
            if self.pause_in_background && playing {
                self.as_mut().pause();
                self.as_mut().rust_mut().backgrounded = true;
            }
        } else if std::mem::take(&mut self.as_mut().rust_mut().backgrounded) && self.paused {
            self.resume();
        }
    }

    /// The platform library, loaded once.
    fn platform(mut self: Pin<&mut Self>) -> Result<Arc<Platform>, String> {
        if self.platform.is_none() {
            let platform = Platform::load(self.platform_library.to_string()).map_err(|e| e.to_string())?;
            self.as_mut().rust_mut().platform = Some(platform);
        }
        self.platform.clone().ok_or_else(|| "no platform library".to_string())
    }

    fn set_rumble(mut self: Pin<&mut Self>, on: bool) {
        if self.rumble == on {
            return;
        }
        self.as_mut().rust_mut().rumble = on;
        self.as_mut().rumble_changed();
        let (Some(runtime), Some(session)) = (self.runtime.clone(), self.opened) else { return };
        if let Err(e) = runtime.input_rumble_switch(session, on) {
            self.as_mut().rust_mut().notice_attention = true;
            self.as_mut().notice_attention_changed();
            self.as_mut().rust_mut().notice = QString::from(format!("Rumble was not changed in the game: {e}").as_str());
            self.as_mut().notice_changed();
        }
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
        } else if !self.start_state.is_empty() && self.opened.is_some() {
            // The game named is the one open: the state chosen is loaded into it.
            let state = std::mem::take(&mut self.as_mut().rust_mut().start_state).to_string();
            self.start_from(state);
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
        self.as_mut().rust_mut().resume_waiting = false;
        if let Some(session) = &self.session {
            session.set_paused(true);
            self.as_mut().set_paused(true);
            self.read_card_facts();
        }
    }

    /// The Pause Card's facts read from the runtime, whose reads answer on any thread (emusen_crystal.h): the reel, the
    /// slots with their pictures beside them (Galaxia's names), the slot chosen, and where the pictures are served.
    fn read_card_facts(mut self: Pin<&mut Self>) {
        let (Some(runtime), Some(session)) = (self.runtime.clone(), self.opened) else { return };
        let Ok(info) = runtime.session_info(session) else { return };
        let mut facts = pause::facts_of(&info, runtime.rewind_reel(session).ok().as_ref());
        let serial = self.serial + 1;
        self.as_mut().rust_mut().serial = serial;
        facts.slot = self.slot;
        facts.speed = self.speed;
        // SAFETY: as in `game_input`.
        facts.second_player = unsafe { GamePads::at(self.pads) }.is_some_and(|pads| pads.seated() & 0b10 != 0);
        facts.pictures = format!("image://{}/{serial}/", pictures::NAME);
        let platform = self.as_mut().platform().ok();
        let picture_of = |path: &str| {
            let picture = platform.as_ref()?.save_path(emusen_platform_client::galaxia::consts::SAVE_PICTURE, path, None, 0).ok()?;
            picture.is_file().then(|| format!("{}file/{}", facts.pictures, picture.display()))
        };
        let slots = runtime.state_slots(session).unwrap_or_default();
        facts.slots = slots
            .iter()
            .filter(|s| (1..=pause::SLOTS).contains(&s.slot))
            .map(|s| SlotFact {
                slot: s.slot,
                exists: s.exists,
                when: when_words(s.saved_at.as_deref()),
                picture: if s.exists { picture_of(&s.path).unwrap_or_default() } else { String::new() },
            })
            .collect();
        facts.cards = card_facts(&runtime, session, &info.console);
        facts.cheats = self.cheats.as_ref().map(|g| g.list.switches()).unwrap_or_default();
        let text = QString::from(facts.to_json().as_str());
        if self.card_facts != text {
            self.as_mut().rust_mut().card_facts = text;
            self.card_facts_changed();
        }
    }

    fn resume(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().backgrounded = false;
        if self.jobs > 0 || self.holding.is_some() || self.from_state.is_some() {
            self.as_mut().rust_mut().resume_waiting = true;
            return;
        }
        if let Some(session) = &self.session {
            session.set_paused(false);
            self.as_mut().set_paused(false);
            self.as_mut().read_picture();
        }
        self.unstep();
    }

    fn set_lever(self: Pin<&mut Self>, console: &QString, id: &QString, value: f64) {
        let (Some(shown), Some(handoff)) = (self.display_console.clone(), self.handoff.clone()) else { return };
        if shown != console.to_string() {
            return;
        }
        if !handoff.set_value(&id.to_string(), value as f32) {
            self.read_picture();
        }
    }

    fn read_picture(mut self: Pin<&mut Self>) {
        let (Some(console), Some(handoff)) = (self.display_console.clone(), self.handoff.clone()) else { return };
        match self.as_mut().display_preset(&console, &handoff) {
            Ok((display, preset)) => {
                handoff.draw_through(preset, display.uniforms());
                self.as_mut().rust_mut().display = display;
                self.as_mut().place_picture();
                self.picture_read();
            }
            Err(words) => self.set_problem(QString::from(&format!("The picture's display settings could not be read: {words}"))),
        }
    }

    /// What `console`'s display settings ask for, and the preset written for it under the feed's folder.
    fn display_preset(mut self: Pin<&mut Self>, console: &str, handoff: &Handoff) -> Result<(Display, Vec<u8>), String> {
        let platform = self.as_mut().platform()?;
        let settings = ClientSettings::open(&self.platform_library.to_string())?;
        if self.catalogue.is_none() {
            self.as_mut().rust_mut().catalogue = Some(settings.catalogue()?);
        }
        let catalogue = self.catalogue.as_ref().ok_or("no catalogue")?;
        let display = display::read(catalogue, &settings.load(Model::Graphics)?, console);
        let preset = match &display.preset {
            Some(path) => {
                let pack = platform.directory(emusen_platform_client::galaxia::consts::DIR_SHADERS).map_err(|e| e.to_string())?;
                let pack = pack.ok_or("the shader pack's folder is not known")?;
                emusen_platform_client::serenity::preset_document(&platform, &pack.join(path).to_string_lossy())
            }
            None => {
                let root = handoff.presets().to_string_lossy();
                let chosen = display.chosen_json();
                emusen_platform_client::serenity::filter_preset(&platform, &display.filter, Some(console), chosen.as_ref(), &root)
            }
        };
        Ok((display, preset.map_err(|e| e.to_string())?))
    }

    /// The picture's shape and crop from the frame's shapes and the console's display settings.
    fn place_picture(mut self: Pin<&mut Self>) {
        let shown = self.display.shown(self.shapes.0, self.shapes.1);
        if self.picture_aspect != shown.aspect {
            self.as_mut().rust_mut().picture_aspect = shown.aspect;
            self.as_mut().picture_aspect_changed();
        }
        let [x, y, w, h] = shown.clip.kept();
        let crop = QRectF::new(x, y, w, h);
        if self.picture_crop != crop {
            self.as_mut().rust_mut().picture_crop = crop;
            self.picture_crop_changed();
        }
    }

    fn toggle_pause(self: Pin<&mut Self>) {
        if self.paused || self.stepped {
            self.resume();
        } else {
            self.pause();
        }
    }

    /// The runtime let run again after a Frame Advance held it.
    fn unstep(mut self: Pin<&mut Self>) {
        if !std::mem::take(&mut self.as_mut().rust_mut().stepped) {
            return;
        }
        let (Some(runtime), Some(session)) = (self.runtime.clone(), self.opened) else { return };
        let resumed = runtime.session_service(session, 0).and_then(|service| service.run_control(session, emusen_crystal_client::consts::RUN_RESUME, 0, 0));
        if let Err(e) = resumed {
            self.say(format!("The game could not be resumed: {e}"), true);
        }
    }

    /// One frame run and the game held after it, by the runtime's frame advance (Mistress's `AdvanceOneFrame`).
    fn frame_advance(mut self: Pin<&mut Self>) {
        let (Some(runtime), Some(session)) = (self.runtime.clone(), self.opened) else { return };
        if let Err(e) = runtime.frame_advance(session) {
            return self.say(format!("Frame Advance failed: {e}"), true);
        }
        self.as_mut().rust_mut().stepped = true;
        if let Some(session) = &self.session {
            session.set_paused(false);
        }
        self.as_mut().set_paused(false);
        self.say(pause::ADVANCED.to_string(), false);
    }

    /// Words over the game, as a job's are said.
    fn say(mut self: Pin<&mut Self>, words: String, attention: bool) {
        if self.notice_attention != attention {
            self.as_mut().rust_mut().notice_attention = attention;
            self.as_mut().notice_attention_changed();
        }
        self.as_mut().rust_mut().notice = QString::from(&words);
        self.as_mut().notice_changed();
        self.set_status(words);
    }

    fn key(self: Pin<&mut Self>, key: i32, modifiers: i32, pressed: bool, repeated: bool) -> bool {
        let Some(word) = keys::word(key, modifiers) else { return false };
        // SAFETY: as in `game_input`.
        let Some(pads) = (unsafe { GamePads::at(self.pads) }) else { return false };
        match pads.key(word, pressed) {
            KeyUse::Control => true,
            KeyUse::Hotkey(hotkey) => {
                if !repeated {
                    self.hotkey_acted(hotkey, pressed);
                }
                true
            }
            KeyUse::Nothing => false,
        }
    }

    fn frame_now(&self) -> u64 {
        let (Some(runtime), Some(session)) = (self.runtime.as_ref(), self.opened) else { return 0 };
        runtime.session_service(session, 0).and_then(|service| service.info(session)).map_or(0, |info| info.frame)
    }

    fn away_key(self: Pin<&mut Self>, key: i32, modifiers: i32, pressed: bool, repeated: bool, library: bool) -> bool {
        // The interface's own keys (the router's, a menu's) are never a hotkey's away from the game, as Mistress's
        // themed keys and its router's raised keys were not (MainWindow.axaml.cs:274-286).
        if emusen_kit_qt::qt_keys::press_of(key, modifiers).is_some() {
            return false;
        }
        let Some(word) = keys::word(key, modifiers) else { return false };
        // SAFETY: as in `game_input`.
        let Some(hotkey) = (unsafe { GamePads::at(self.pads) }).and_then(|pads| pads.hotkey(word)) else { return false };
        if pressed && !repeated {
            self.away_hotkey(hotkey, library);
        }
        true
    }

    /// A hotkey's action away from the game, by the states of Mistress's `SetButtonFromKey` (the table in EmuSen_Qt.md
    /// §7.7.H8): the game is suspended, so nothing is held faster or back, paused or stepped.
    fn away_hotkey(mut self: Pin<&mut Self>, hotkey: Hotkey, library: bool) {
        let open = self.opened.is_some();
        match hotkey {
            Hotkey::FastForward | Hotkey::Rewind | Hotkey::TogglePause | Hotkey::FrameAdvance | Hotkey::ToggleBigPicture => {}
            Hotkey::SaveState if !open => self.say(pause::NO_GAME_SAVE.to_string(), false),
            Hotkey::LoadState if !open => self.say(pause::NO_GAME_LOAD.to_string(), false),
            Hotkey::SaveState => {
                let slot = self.slot;
                self.save_state(slot);
            }
            Hotkey::LoadState => {
                let slot = self.slot;
                self.load_state(slot);
            }
            Hotkey::Screenshot if open => self.screenshot(),
            Hotkey::Screenshot => {}
            Hotkey::ExitToLibrary if !library => self.exit_asked(),
            Hotkey::ExitToLibrary if open => self.as_mut().game_asked(),
            Hotkey::ExitToLibrary => {}
            Hotkey::ToggleFullscreen => self.full_screen_asked(),
        }
    }

    /// A hotkey's action, held ones while the key is down and the rest as it goes down (Mistress's `SetButtonFromKey`).
    fn hotkey_acted(self: Pin<&mut Self>, hotkey: Hotkey, pressed: bool) {
        if !pressed && !hotkey.held() {
            return;
        }
        let (Some(runtime), Some(session)) = (self.runtime.clone(), self.opened) else { return };
        match hotkey {
            Hotkey::FastForward => {
                let mode = if pressed { emusen_crystal_client::consts::SPEED_TURBO } else { emusen_crystal_client::consts::SPEED_NORMAL };
                match runtime.speed_mode(session, mode) {
                    Err(e) => self.say(format!("Fast forward failed: {e}"), true),
                    Ok(()) if pressed => self.say(pause::FAST_FORWARD.to_string(), false),
                    Ok(()) => {}
                }
            }
            Hotkey::Rewind => {
                if let Some(session) = &self.session {
                    session.set_rewinding(pressed);
                }
            }
            Hotkey::SaveState => {
                let slot = self.slot;
                self.save_state(slot);
            }
            Hotkey::LoadState => {
                let slot = self.slot;
                self.load_state(slot);
            }
            Hotkey::TogglePause => self.toggle_pause(),
            Hotkey::FrameAdvance => self.frame_advance(),
            Hotkey::Screenshot => self.screenshot(),
            Hotkey::ExitToLibrary => self.exit_asked(),
            Hotkey::ToggleFullscreen => self.full_screen_asked(),
            // The program has no desktop view to leave to (EmuSen_Qt.md §7.7.H6).
            Hotkey::ToggleBigPicture => {}
        }
    }

    fn close(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().stepped = false;
        self.as_mut().rust_mut().backgrounded = false;
        *self.shown.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        drop(self.as_mut().rust_mut().session.take());
        self.as_mut().rust_mut().opened = None;
        self.as_mut().rust_mut().cheats = None;
        self.as_mut().rust_mut().holding = None;
        self.as_mut().rust_mut().from_state = None;
        self.as_mut().set_resume_asked(false);
        self.as_mut().set_running(false);
        self.as_mut().set_paused(false);
        self.as_mut().set_sounding(false);
        self.as_mut().set_fps(0.0);
        self.set_status(String::new());
    }

    fn set_fps(mut self: Pin<&mut Self>, fps: f64) {
        if self.fps != fps {
            self.as_mut().rust_mut().fps = fps;
            self.as_mut().rust_mut().fps_words = QString::from(&pause::fps_words(fps));
            self.as_mut().fps_changed();
            self.fps_words_changed();
        }
    }

    fn reported(mut self: Pin<&mut Self>, report: Report) {
        match report {
            Report::Opened { session } => {
                self.as_mut().rust_mut().opened = Some(session);
                if let Some(runtime) = self.runtime.clone() {
                    *self.shown.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = Some((runtime, session));
                }
                if !self.served {
                    let served = pictures::ffi::serve(self.as_ref().get_ref().upcast(), &QString::from(pictures::NAME), PictureSource::new(self.shown.clone()));
                    self.as_mut().rust_mut().served = served;
                }
                self.as_mut().apply_speed();
                let console = self.runtime.as_ref().and_then(|r| r.session_info(session).ok()).map(|info| info.console);
                self.as_mut().rust_mut().display_console = console;
                self.as_mut().read_picture();
                self.as_mut().set_running(true);
                self.as_mut().left_off(session);
            }
            Report::FirstPicture { after } => self.as_mut().set_first_picture_ms(after.as_secs_f64() * 1000.0),
            Report::Shape { screen, square } => {
                self.as_mut().rust_mut().shapes = (screen, square);
                self.as_mut().place_picture();
            }
            Report::Failed(words) => {
                self.as_mut().failed(words);
                self.as_mut().set_running(false);
            }
            Report::Closed => self.as_mut().set_running(false),
            Report::Said(words) => self.as_mut().say(words, true),
            Report::Volume(volume) => self.as_mut().heard_volume(volume),
            Report::Device(device) => self.as_mut().heard_device(device),
            Report::Rate { fps } => self.as_mut().set_fps(fps),
            Report::Cheats(given) => self.as_mut().rust_mut().cheats = given,
        }
    }

    fn switch_cheat(mut self: Pin<&mut Self>, index: i32, on: bool) {
        let (Some(runtime), Some(session)) = (self.runtime.clone(), self.opened) else { return };
        let Ok(platform) = self.as_mut().platform() else { return };
        let Some(mut given) = self.as_mut().rust_mut().cheats.take() else { return };
        let switched = given.switch(&platform, &runtime, session, index.max(0) as usize, on);
        self.as_mut().rust_mut().cheats = Some(given);
        if let Err(words) = switched {
            self.as_mut().say(format!("The cheat was not switched: {words}"), true);
        }
        self.read_card_facts();
    }

    fn choose_slot(mut self: Pin<&mut Self>, slot: u32) {
        let slot = slot.clamp(1, pause::SLOTS);
        if self.slot != slot {
            self.as_mut().rust_mut().slot = slot;
            self.as_mut().slot_changed();
            if self.paused {
                self.read_card_facts();
            }
        }
    }

    fn choose_speed(mut self: Pin<&mut Self>, percent: u32) {
        if self.speed != percent {
            self.as_mut().rust_mut().speed = percent;
            self.as_mut().speed_changed();
            self.as_mut().apply_speed();
            if self.paused {
                self.read_card_facts();
            }
        }
    }

    /// The speed chosen given to the open game, with turbo and slow motion at the card's fastest and slowest
    /// (Mistress's `ApplySpeed`); a change the runtime makes between two frames.
    fn apply_speed(self: Pin<&mut Self>) {
        let (Some(runtime), Some(session)) = (self.runtime.clone(), self.opened) else { return };
        let (slow, turbo) = (pause::SPEEDS[0], pause::SPEEDS[pause::SPEEDS.len() - 1]);
        if let Err(e) = runtime.speed_set(session, self.speed, turbo, slow, 0) {
            self.set_problem(QString::from(&e.to_string()));
        }
    }

    fn save_state(self: Pin<&mut Self>, slot: u32) {
        self.job(move |runtime, session| match runtime.state_slot_save(session, slot) {
            Ok(()) => Ok(format!("State saved to slot {slot}")),
            Err(e) => Err(format!("Save State failed: {e}")),
        });
    }

    fn load_state(self: Pin<&mut Self>, slot: u32) {
        self.job(move |runtime, session| match runtime.state_slot_load(session, slot) {
            Ok(true) => Ok(format!("State loaded from slot {slot}")),
            Ok(false) => Err(format!("Load State: slot {slot} is empty")),
            Err(e) => Err(format!("Load State failed: {e}")),
        });
    }

    /// Named as Mistress names one: the game's file name, then the local time to the millisecond, in Galaxia's
    /// screenshots folder.
    fn screenshot(mut self: Pin<&mut Self>) {
        let folder = self.as_mut().platform().ok().and_then(|p| p.directory(emusen_platform_client::galaxia::consts::DIR_SCREENSHOTS).ok().flatten());
        let Some(folder) = folder else { return self.set_problem(QString::from("No screenshots folder.")) };
        let game = self.game.to_string();
        let now = QDateTime::current_date_time();
        let (date, time) = (now.date().format(&QString::from(crate::screenshots::DATE)), now.time().format(&QString::from(crate::screenshots::TIME)));
        let path = folder.join(crate::screenshots::name(&game, &date.to_string(), &time.to_string()));
        self.job(move |runtime, session| {
            let saved = std::fs::create_dir_all(path.parent().unwrap_or(&path)).is_ok() && pictures::save_now(runtime, session, &path.to_string_lossy());
            if saved { Ok("Screenshot saved".to_string()) } else { Err(format!("Screenshot failed: {}", path.display())) }
        });
    }

    fn restart(self: Pin<&mut Self>) {
        self.job(|runtime, session| match runtime.reset(session) {
            Ok(()) => Ok("Restarted".to_string()),
            Err(e) => Err(format!("Restart failed: {e}")),
        });
    }

    fn rewind_to(self: Pin<&mut Self>, frame: u64) {
        self.job(move |runtime, session| match runtime.rewind_to(session, frame) {
            Ok(()) => Ok("Rewound".to_string()),
            Err(_) => Err("Rewind: that moment is no longer held".to_string()),
        });
    }

    fn change_disc(self: Pin<&mut Self>, index: u32) {
        self.job(move |runtime, session| match runtime.disc_change(session, index) {
            Ok(()) => Ok(format!("Disc {} is in", index + 1)),
            Err(e) => Err(format!("Change Disc failed: {e}")),
        });
    }

    fn insert_disc(self: Pin<&mut Self>, path: &QString) {
        let path = path.to_string();
        let name = std::path::Path::new(&path).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        self.job(move |runtime, session| match runtime.disc_insert(session, &path) {
            Ok(()) => Ok(format!("{name} is in")),
            Err(e) => Err(format!("Insert Disc failed: {e}")),
        });
    }

    fn eject_disc(self: Pin<&mut Self>) {
        self.job(|runtime, session| match runtime.disc_eject(session) {
            Ok(()) => Ok("The disc is out".to_string()),
            Err(e) => Err(format!("Eject Disc failed: {e}")),
        });
    }

    fn set_console_setting(mut self: Pin<&mut Self>, key: &QString, value: &QString) {
        let (Some(runtime), Some(session)) = (self.runtime.clone(), self.opened) else { return };
        let set = runtime.settings_set(session, emusen_crystal_client::Scope::Console, None, &key.to_string(), Some(&value.to_string()));
        if let Err(e) = set {
            self.as_mut().rust_mut().notice_attention = true;
            self.as_mut().notice_attention_changed();
            self.as_mut().rust_mut().notice = QString::from(format!("{key} was not changed: {e}").as_str());
            self.as_mut().notice_changed();
        }
        self.read_card_facts();
    }

    fn swap_players(mut self: Pin<&mut Self>) {
        // SAFETY: as in `game_input`.
        let swapped = unsafe { GamePads::at(self.pads) }.is_some_and(|pads| pads.swap_players());
        if swapped {
            self.as_mut().rust_mut().notice_attention = false;
            self.as_mut().notice_attention_changed();
            self.as_mut().rust_mut().notice = QString::from(pause::SWAPPED);
            self.as_mut().notice_changed();
        }
    }

    fn guide_chord(self: Pin<&mut Self>, button: qobject::PadButton) {
        let Some(button) = emusen_kit_qt::kinds::ui_button(button) else { return };
        match pause::guide_chord(button, self.slot) {
            pause::Outcome::Save(slot) => self.save_state(slot),
            pause::Outcome::Load(slot) => self.load_state(slot),
            pause::Outcome::Screenshot => self.screenshot(),
            _ => {}
        }
    }

    /// A change to the session made on a thread of its own, as Mistress makes it, its words brought back through the
    /// queue; the runtime applies it between two frames, so a paused game takes it at once (emusen_crystal.h). Its words
    /// are Ok for what was done, Err for what was not, which the notice marks for attention.
    fn job(mut self: Pin<&mut Self>, work: impl FnOnce(&Runtime, u32) -> Result<String, String> + Send + 'static) {
        let (Some(runtime), Some(session)) = (self.runtime.clone(), self.opened) else { return };
        self.as_mut().rust_mut().jobs += 1;
        let queue = self.qt_thread();
        std::thread::spawn(move || {
            let said = work(&runtime, session);
            let _ = queue.queue(move |model| model.job_done(said));
        });
    }

    fn job_done(mut self: Pin<&mut Self>, said: Result<String, String>) {
        self.as_mut().rust_mut().jobs -= 1;
        let attention = said.is_err();
        let words = said.unwrap_or_else(|e| e);
        if self.notice_attention != attention {
            self.as_mut().rust_mut().notice_attention = attention;
            self.as_mut().notice_attention_changed();
        }
        // Said on every job with words, so the same words twice are two notices; a resume that loaded says nothing.
        if !words.is_empty() {
            let words = QString::from(&words);
            if self.notice != words {
                self.as_mut().rust_mut().notice = words.clone();
            }
            self.as_mut().notice_changed();
            self.as_mut().set_status(words.to_string());
        }
        if self.paused {
            self.as_mut().read_card_facts();
        }
        if self.jobs == 0 && std::mem::take(&mut self.as_mut().rust_mut().resume_waiting) {
            self.resume();
        }
    }
}

/// The running console's cards for Game Options ▸ Memory Cards: the card manager's list, the session's schema rows of
/// the media's keys, and the values the console holds for them.
fn card_facts(runtime: &Runtime, session: u32, console: &str) -> pause::CardFacts {
    use emusen_crystal_client::Scope;
    let list = runtime.cards_op(emusen_crystal_client::consts::CARDS_LIST, &serde_json::json!({ "console": console })).ok().flatten().unwrap_or_default();
    let media = crate::cards::media(&list);
    let schema = runtime.settings_schema(session, None, None).unwrap_or_default();
    let schema: Vec<_> = schema.into_iter().filter(|r| media.iter().any(|m| r.key == m.mode_key || r.shown_with.as_ref().is_some_and(|w| w.key == m.mode_key))).collect();
    let values = schema.iter().filter_map(|r| Some((r.key.clone(), runtime.settings_get(session, Scope::Console, None, &r.key).ok().flatten()?))).collect();
    pause::CardFacts { console: console.to_owned(), media, schema, values }
}
