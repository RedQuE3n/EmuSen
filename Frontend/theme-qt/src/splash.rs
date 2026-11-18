//! `Splash`, a splash that enters Home as the theme's screens read it, fed by the host (EmuSen_Qt.md §3.2, 1.10).

use core::pin::Pin;
use cxx_qt::CxxQtType;

#[cxx_qt::bridge]
pub mod qobject {
    extern "RustQt" {
        #[qobject]
        #[qml_element = "Splash"]
        #[qml_singleton]
        /// A splash whose screen says `entersHome` is drawn, from its first frame until it has ended or been skipped.
        #[qproperty(bool, shown, READ, NOTIFY)]
        /// The splash's time in ms, the moment its screen and Home draw (EmuSen_Qt.md §3.5.2).
        #[qproperty(f64, time, READ, NOTIFY)]
        type ThemeSplash = super::ThemeSplashRust;
    }
}

#[derive(Default)]
pub struct ThemeSplashRust {
    shown: bool,
    time: f64,
}

impl qobject::ThemeSplash {
    /// The host's feed; each property notifies only when it changed.
    pub fn show(mut self: Pin<&mut Self>, shown: bool, time: f64) {
        if self.shown != shown {
            self.as_mut().rust_mut().shown = shown;
            self.as_mut().shown_changed();
        }
        if self.time != time {
            self.as_mut().rust_mut().time = time;
            self.as_mut().time_changed();
        }
    }
}
