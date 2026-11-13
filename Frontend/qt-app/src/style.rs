//! `ThemeStyle`: the chosen EmuSen theme's token overrides on the program's own engine, whose Settings, cards, game page,
//! Pause Card, tools and notices read the kit's `Tokens` (decided by the tester, 2026-11-13; EmuSen_Qt.md §3.5.1). The
//! window hands it the chosen theme's host, or none for no theme, the reference and an ES-DE theme, which leave the kit's.

use core::pin::Pin;
use emusen_theme_qt::host;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("emusen-theme-host/src/host.cxxqt.h");
        type ThemeHost = emusen_theme_host::host::qobject::ThemeHost;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        /// The chosen theme's host, whose package's overrides the program's engine takes; null for none.
        #[qproperty(*mut ThemeHost, host)]
        type ThemeStyle = super::ThemeStyleRust;

        /// The host's package as it is now on the program's engine: the window calls it as the host makes a new engine.
        #[qinvokable]
        fn restyle(self: Pin<&mut ThemeStyle>);
    }

    impl cxx_qt::Initialize for ThemeStyle {}
}

pub struct ThemeStyleRust {
    host: *mut qobject::ThemeHost,
}

impl Default for ThemeStyleRust {
    fn default() -> Self {
        ThemeStyleRust { host: std::ptr::null_mut() }
    }
}

impl cxx_qt::Initialize for qobject::ThemeStyle {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut().on_host_changed(|style| style.restyle()).release();
    }
}

impl qobject::ThemeStyle {
    /// The overrides `ThemeFeed` sets on the theme's engine, by the same call, on the engine this object lives in.
    pub fn restyle(self: Pin<&mut Self>) {
        // Safety: the host is the window's chosen theme's, alive while it is handed here; the window hands null as it goes.
        let overrides = unsafe { self.host.as_ref() }.map(|h| h.overrides()).unwrap_or_default();
        use cxx_qt::casting::Upcast;
        let object: Pin<&mut cxx_qt::QObject> = self.upcast_pin();
        host::set_token_overrides(object, &overrides);
    }
}
