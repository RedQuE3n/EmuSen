//! `SessionModel`, the running game (EmuSen_Qt.md §1.2, §1.8.2): open a game, run, pause, close. session.rs's frame
//! thread put on Qt; its reports come back through qt_thread().queue, the one way in.

use crate::picture::PictureFeed;
use crate::session::{Report, Session};
use core::pin::Pin;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;
use emusen_crystal_client::{Api, Config, Runtime};

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
        #[qproperty(QString, game, READ, WRITE = set_game, NOTIFY)]
        #[qproperty(bool, running, READ, NOTIFY)]
        #[qproperty(bool, paused, READ, NOTIFY)]
        #[qproperty(QString, problem, READ, NOTIFY)]
        #[qproperty(f64, first_picture_ms, cxx_name = "firstPictureMs", READ, NOTIFY)]
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
    game: QString,
    running: bool,
    paused: bool,
    problem: QString,
    first_picture_ms: f64,
    runtime: Option<Runtime>,
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
            // The cores folder a host names, else the runtime's own default (emusen_crystal.h, the config's `cores`).
            let config = Config { cores: (!self.cores.is_empty()).then(|| vec![self.cores.to_string()]), ..Config::default() };
            match Api::load(&library).and_then(|api| Runtime::new(&api, &config)) {
                Ok(runtime) => self.as_mut().rust_mut().runtime = Some(runtime),
                Err(e) => return self.as_mut().set_problem(QString::from(&e.to_string())),
            }
        }
        let Some(runtime) = self.runtime.clone() else { return };
        let queue = self.qt_thread();
        let report = move |report: Report| {
            let _ = queue.queue(move |model| model.reported(report));
        };
        let engine = (!self.engine.is_empty()).then(|| self.engine.to_string());
        let session =
            Session::start(runtime, path.to_string(), engine, std::ptr::NonNull::new(device.as_ptr().cast_mut()), handoff, report);
        self.as_mut().rust_mut().session = Some(session);
        self.as_mut().set_problem(QString::default());
        self.as_mut().set_paused(false);
        self.as_mut().set_first_picture_ms(0.0);
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
        }
    }

    fn resume(mut self: Pin<&mut Self>) {
        if let Some(session) = &self.session {
            session.set_paused(false);
            self.as_mut().set_paused(false);
        }
    }

    fn close(mut self: Pin<&mut Self>) {
        drop(self.as_mut().rust_mut().session.take());
        self.as_mut().set_running(false);
        self.as_mut().set_paused(false);
    }

    fn reported(mut self: Pin<&mut Self>, report: Report) {
        match report {
            Report::Opened { .. } => self.as_mut().set_running(true),
            Report::FirstPicture { after } => self.as_mut().set_first_picture_ms(after.as_secs_f64() * 1000.0),
            Report::Failed(words) => {
                self.as_mut().set_problem(QString::from(&words));
                self.as_mut().set_running(false);
            }
            Report::Closed => self.as_mut().set_running(false),
        }
    }
}
