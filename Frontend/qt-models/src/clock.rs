//! `Clock`: Home's lines that change with the time, read again on each `tick` (EmuSen_Qt.md §7.1): the status line,
//! "1 controller · 21:42", and a paused game's facts, "Super Nintendo · Paused 3 minutes ago".

use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QDateTime, QString, QTimeZone};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("short_time.h");

        /// `seconds`' time of day, in UTC or the machine's zone, in the short form of `locale` ("" is the machine's own).
        #[cxx_name = "clockShortTime"]
        fn short_time_of(seconds: i64, utc: bool, locale: &QString) -> QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, time, READ, NOTIFY)]
        /// The time in the machine's own short form, "9:41 PM" where its locale is en_US (EmuSen_Qt.md §7.1).
        #[qproperty(QString, short_time, cxx_name = "shortTime", READ, NOTIFY)]
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
    short_time: QString,
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
            // The moment of pausing is the clock's, held or the wall clock's, as a theme's Words read it (EmuSen_Qt.md §7.8).
            let at = clock.at();
            clock.as_mut().rust_mut().paused_at = at;
            clock.say();
        })
        .release();
        self.tick();
    }
}

/// A moment, in seconds since 1970, in the form the clock reads the wall clock: the player's own hours and minutes, "hh:mm".
pub fn clock_time(at: i64) -> String {
    QDateTime::from_secs_since_epoch(at, &QTimeZone::system_time_zone()).time().format(&QString::from("hh:mm")).to_string()
}

/// A moment, in seconds since 1970, in the machine's zone and its locale's short time form, `QLocale::ShortFormat`:
/// "9:41 PM" under en_US, "21:41" under de_DE.
pub fn short_time(at: i64) -> String {
    short_time_in(at, false, "")
}

/// A moment's time in UTC or the machine's zone, in the short form of the named locale ("" is the machine's own).
pub fn short_time_in(at: i64, utc: bool, locale: &str) -> String {
    qobject::short_time_of(at, utc, &QString::from(locale)).to_string()
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
            QString::from(clock_time(self.at()).as_str())
        };
        if self.time != time {
            self.as_mut().rust_mut().time = time;
            self.as_mut().time_changed();
        }
        let short = QString::from(short_time_in(self.at(), self.now > 0, "").as_str());
        if self.short_time != short {
            self.as_mut().rust_mut().short_time = short;
            self.as_mut().short_time_changed();
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
