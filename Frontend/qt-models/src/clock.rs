//! `Clock`: the time of day and Home's status line, "1 controller · 21:42", read again on each `tick` (EmuSen_Qt.md §7.1).

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
        type Clock = super::ClockRust;
    }

    extern "RustQt" {
        /// Reads the time again; the property changes only when the minute does.
        #[qinvokable]
        fn tick(self: Pin<&mut Clock>);
    }

    impl cxx_qt::Initialize for Clock {}
}

#[derive(Default)]
pub struct ClockRust {
    time: QString,
    /// The pads connected, as Endymion counts them.
    controllers: i32,
    line: QString,
}

impl cxx_qt::Initialize for qobject::Clock {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut().on_controllers_changed(|clock| clock.say()).release();
        self.tick();
    }
}

impl qobject::Clock {
    pub fn tick(mut self: Pin<&mut Self>) {
        let time = QTime::current_time().format(&QString::from("hh:mm"));
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
    }
}
