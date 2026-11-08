//! `NoticeModel`: a window's notices over kit-core's rule, one shown at a time, the rest waiting while gameplay runs; the
//! kit's `NoticeLayer` draws the one shown and ends it when its hold runs out (EmuSen_Kit.md §7.23).

use core::pin::Pin;

use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use emusen_kit_core::notice::{Notice, Notices, Tone, HOLD_MS};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, words, READ, NOTIFY)]
        #[qproperty(bool, attention, READ, NOTIFY)]
        #[qproperty(bool, shown, READ, NOTIFY)]
        #[qproperty(i32, hold_ms, READ, CONSTANT)]
        #[qproperty(bool, game_running)]
        type NoticeModel = super::NoticeModelRust;
    }

    unsafe extern "RustQt" {
        /// A notice was shown, or shown again in place of the last: its hold starts over.
        #[qsignal]
        fn raised(self: Pin<&mut NoticeModel>);

        /// Raises a notice in the host's words: in Attention's glyph when `attention`, and fit for running gameplay when
        /// `over_game` (Saved, Loaded, the screenshot).
        #[qinvokable]
        fn raise(self: Pin<&mut NoticeModel>, words: &QString, attention: bool, over_game: bool);

        /// The hold ran out.
        #[qinvokable]
        fn expire(self: Pin<&mut NoticeModel>);
    }

    impl cxx_qt::Initialize for NoticeModel {}
}

pub struct NoticeModelRust {
    words: QString,
    attention: bool,
    shown: bool,
    hold_ms: i32,
    game_running: bool,
    notices: Notices,
}

impl Default for NoticeModelRust {
    fn default() -> Self {
        NoticeModelRust {
            words: QString::default(),
            attention: false,
            shown: false,
            hold_ms: HOLD_MS as i32,
            game_running: false,
            notices: Notices::default(),
        }
    }
}

impl cxx_qt::Initialize for qobject::NoticeModel {
    fn initialize(self: Pin<&mut Self>) {
        self.on_game_running_changed(|mut model| {
            let running = model.game_running;
            if model.as_mut().rust_mut().notices.set_game_running(running) {
                model.as_mut().publish();
            }
        })
        .release();
    }
}

impl qobject::NoticeModel {
    pub fn raise(mut self: Pin<&mut Self>, words: &QString, attention: bool, over_game: bool) {
        let tone = if attention { Tone::Attention } else { Tone::Success };
        let notice = Notice { words: words.to_string(), tone, over_game };
        if self.as_mut().rust_mut().notices.raise(notice) {
            self.as_mut().publish();
        }
    }

    pub fn expire(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().notices.expire();
        self.publish();
    }

    /// The notice shown on the properties; a notice shown raises `raised`, so the layer's hold starts again.
    fn publish(mut self: Pin<&mut Self>) {
        let (words, attention, shown) = match self.notices.shown() {
            Some(n) => (QString::from(n.words.as_str()), n.tone == Tone::Attention, true),
            None => (self.words.clone(), self.attention, false),
        };
        if words != self.words {
            self.as_mut().rust_mut().words = words;
            self.as_mut().words_changed();
        }
        if attention != self.attention {
            self.as_mut().rust_mut().attention = attention;
            self.as_mut().attention_changed();
        }
        if shown != self.shown {
            self.as_mut().rust_mut().shown = shown;
            self.as_mut().shown_changed();
        }
        if shown {
            self.as_mut().raised();
        }
    }
}
