//! `SessionModel`, the running game (EmuSen_Qt.md §1.2, §1.8.2): open a game, run, pause, close. session.rs's frame
//! thread put on Qt; its reports come back through qt_thread().queue, the one way in.

use crate::picture::PictureFeed;
use crate::session::{GameIo, Report, Session};
use core::pin::Pin;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{DateFormat, QDateTime, QString};
use crate::input::GameInput;
use crate::pause::{self, SlotFact};
use crate::resume::{self, Answer, Start};
use crate::pictures::{self, PictureSource, Shown};
use cxx_qt::casting::Upcast;
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
        #[qproperty(QString, game, READ, WRITE = set_game, NOTIFY)]
        #[qproperty(bool, running, READ, NOTIFY)]
        #[qproperty(bool, paused, READ, NOTIFY)]
        /// Whether the open game's sound reaches a player.
        #[qproperty(bool, sounding, READ, NOTIFY)]
        #[qproperty(QString, problem, READ, NOTIFY)]
        #[qproperty(f64, first_picture_ms, cxx_name = "firstPictureMs", READ, NOTIFY)]
        /// The picture's width over its height as shown; 0 before the first picture, where it fills its item.
        #[qproperty(f64, picture_aspect, cxx_name = "pictureAspect", READ, NOTIFY)]
        /// What the Pause Card's words are made from, read as the game pauses (pause.rs's facts as plain data).
        #[qproperty(QString, card_facts, cxx_name = "cardFacts", READ, NOTIFY)]
        /// The state slot Save and Load act on, 1 to 8, kept while the program runs.
        #[qproperty(u32, slot, READ, NOTIFY)]
        /// The game's speed in percent of its own, the player's choice kept while the program runs; 0 unthrottled.
        #[qproperty(u32, speed, READ, NOTIFY)]
        /// What the last save, load or rewind came to, in the player's words.
        #[qproperty(QString, notice, READ, NOTIFY)]
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

        /// The slot Save and Load act on.
        #[qinvokable]
        #[cxx_name = "chooseSlot"]
        fn choose_slot(self: Pin<&mut SessionModel>, slot: u32);

        /// The game's speed chosen, applied to the running game at once and to each game opened after.
        #[qinvokable]
        #[cxx_name = "chooseSpeed"]
        fn choose_speed(self: Pin<&mut SessionModel>, percent: u32);

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

        /// Guide held with `button` over the running game (`PadRouter.guideChord`): pause.rs's `guide_chord` says what it asks.
        #[qinvokable]
        #[cxx_name = "guideChord"]
        fn guide_chord(self: Pin<&mut SessionModel>, button: PadButton);
    }

    impl cxx_qt::Threading for SessionModel {}
}

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
    picture_aspect: f64,
    card_facts: QString,
    slot: u32,
    speed: u32,
    notice: QString,
    notice_attention: bool,
    resume_asked: bool,
    resume_subject: QString,
    /// The setting Resume while the opened game is held for it, until the runtime says whether it was left off.
    holding: Option<resume::Setting>,
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
}

impl Default for SessionModelRust {
    fn default() -> Self {
        SessionModelRust {
            platform_library: QString::default(),
            feed: 0,
            cores: QString::default(),
            engine: QString::default(),
            pads: 0,
            game: QString::default(),
            running: false,
            paused: false,
            sounding: false,
            problem: QString::default(),
            first_picture_ms: 0.0,
            picture_aspect: 0.0,
            card_facts: QString::default(),
            slot: 1,
            speed: 100,
            notice: QString::default(),
            notice_attention: false,
            resume_asked: false,
            resume_subject: QString::default(),
            holding: None,
            shown: Shown::default(),
            serial: 0,
            served: false,
            jobs: 0,
            resume_waiting: false,
            opened: None,
            runtime: None,
            platform: None,
            session: None,
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
        let setting = self.as_mut().resume_setting();
        let holding = (setting != resume::Setting::Never).then_some(setting);
        self.as_mut().rust_mut().holding = holding;
        self.as_mut().set_resume_subject(QString::from(&resume::subject(&path.to_string())));
        let session = Session::start(runtime, path.to_string(), engine, device, handoff, GameIo { pads, sound }, clock, holding.is_some(), report);
        self.as_mut().set_sounding(sounding);
        self.as_mut().rust_mut().session = Some(session);
        self.as_mut().set_problem(QString::from(unsounded.as_deref().unwrap_or_default()));
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
        let text = QString::from(facts.to_json().as_str());
        if self.card_facts != text {
            self.as_mut().rust_mut().card_facts = text;
            self.card_facts_changed();
        }
    }

    fn resume(mut self: Pin<&mut Self>) {
        if self.jobs > 0 || self.holding.is_some() {
            self.as_mut().rust_mut().resume_waiting = true;
            return;
        }
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
        *self.shown.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        drop(self.as_mut().rust_mut().session.take());
        self.as_mut().rust_mut().opened = None;
        self.as_mut().rust_mut().holding = None;
        self.as_mut().set_resume_asked(false);
        self.as_mut().set_running(false);
        self.as_mut().set_paused(false);
        self.as_mut().set_sounding(false);
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
                self.as_mut().set_running(true);
                self.as_mut().left_off(session);
            }
            Report::FirstPicture { after } => self.as_mut().set_first_picture_ms(after.as_secs_f64() * 1000.0),
            Report::Shape { aspect } => {
                self.as_mut().rust_mut().picture_aspect = aspect;
                self.picture_aspect_changed();
            }
            Report::Failed(words) => {
                self.as_mut().set_problem(QString::from(&words));
                self.as_mut().set_running(false);
            }
            Report::Closed => self.as_mut().set_running(false),
        }
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
        let stem = std::path::Path::new(&game).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        let now = QDateTime::current_date_time();
        let stamp = format!("{} {}", now.date().format(&QString::from("yyyy-MM-dd")), now.time().format(&QString::from("HH.mm.ss.zzz")));
        let path = folder.join(format!("{stem} {stamp}.png"));
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
                self.as_mut().rust_mut().notice = words;
            }
            self.as_mut().notice_changed();
        }
        if self.paused {
            self.as_mut().read_card_facts();
        }
        if self.jobs == 0 && std::mem::take(&mut self.as_mut().rust_mut().resume_waiting) {
            self.resume();
        }
    }
}
