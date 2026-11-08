//! `Clock`: the time of day as Home's status line shows it, read again on each `tick` (EmuSen_Qt.md §7.1).

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
}

impl cxx_qt::Initialize for qobject::Clock {
    fn initialize(self: Pin<&mut Self>) {
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
    }
}
