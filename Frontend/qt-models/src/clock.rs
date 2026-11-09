//! `Clock`: Home's lines that change with the time, read again on each `tick` (EmuSen_Qt.md §7.1): the status line,
//! "1 controller · 21:42", and a paused game's facts, "Super Nintendo · Paused 3 minutes ago".

use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QString, QTime};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, time, READ, NOTIFY)]
        #[qproperty(i32, controllers)]
        #[qproperty(QString, line, READ, NOTIFY)]
        #[qproperty(QString, console)]
        #[qproperty(bool, game_paused, cxx_name = "gamePaused")]
        #[qproperty(QString, paused, READ, NOTIFY)]
        /// The time the clock reads, in seconds since 1970; 0, the default, is the wall clock. A test holds it, as
        /// esde-qt's tests hold EsdeModel's, so no state depends on the minute it is read in.
        #[qproperty(i64, now)]
        type Clock = super::ClockRust;
    }

    extern "RustQt" {
        /// Reads the time again; the property changes only when the minute does.
        #[qinvokable]
        fn tick(self: Pin<&mut Clock>);

        /// When the game was paused, in seconds since 1970; 0 while none is.
        #[qinvokable]
        #[cxx_name = "pausedAt"]
        fn paused_at(self: &Clock) -> f64;
    }

    impl cxx_qt::Initialize for Clock {}
}

#[derive(Default)]
pub struct ClockRust {
    time: QString,
    /// The pads connected, as Endymion counts them.
    controllers: i32,
    line: QString,
    /// The paused game's console, whether a game is paused, and when it was paused in seconds since 1970.
    console: QString,
    game_paused: bool,
    paused_at: i64,
    paused: QString,
    now: i64,
}

impl cxx_qt::Initialize for qobject::Clock {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut().on_controllers_changed(|clock| clock.say()).release();
        self.as_mut().on_console_changed(|clock| clock.say()).release();
        self.as_mut().on_now_changed(|clock| clock.tick()).release();
        self.as_mut().on_game_paused_changed(|mut clock| {
            // The moment of pausing is the wall clock's, which the session and a theme's Words also read; a held clock
            // holds only how it is said.
            clock.as_mut().rust_mut().paused_at = crate::words::now();
            clock.say();
        })
        .release();
        self.tick();
    }
}

impl qobject::Clock {
    pub fn paused_at(&self) -> f64 {
        if self.game_paused { self.paused_at as f64 } else { 0.0 }
    }

    /// Now, held or the wall clock's.
    fn at(&self) -> i64 {
        if self.now > 0 { self.now } else { crate::words::now() }
    }

    pub fn tick(mut self: Pin<&mut Self>) {
        // A held time reads its hours and minutes in UTC; the wall clock's are the player's own.
        let time = if self.now > 0 {
            QString::from(format!("{:02}:{:02}", self.now.rem_euclid(86_400) / 3600, self.now.rem_euclid(3600) / 60).as_str())
        } else {
            QTime::current_time().format(&QString::from("hh:mm"))
        };
        if self.time != time {
            self.as_mut().rust_mut().time = time;
            self.as_mut().time_changed();
        }
        self.say();
    }

    fn say(mut self: Pin<&mut Self>) {
        let line = QString::from(crate::words::status_line(self.controllers.max(0) as usize, &self.time.to_string()).as_str());
        if self.line != line {
            self.as_mut().rust_mut().line = line;
            self.as_mut().line_changed();
        }
        let paused =
            if self.game_paused { crate::words::paused_facts(&self.console.to_string(), self.paused_at, self.at()) } else { String::new() };
        let paused = QString::from(paused.as_str());
        if self.paused != paused {
            self.as_mut().rust_mut().paused = paused;
            self.as_mut().paused_changed();
        }
    }
}
