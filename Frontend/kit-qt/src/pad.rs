//! `PadRouter`: the kit's one pad entry, a singleton whose queue tests and Endymion's reader push `PadEvent`s into (EmuSen_Kit.md §7.4, §7.15).
//! `clockRunning` lets a `Frame`'s timer pump on the wall clock while `pending`; a test turns it off and pumps at stated times.

use core::pin::Pin;

use cxx_qt::CxxQtType;
use std::time::{Duration, Instant};

use emusen_kit_core::input::RepeatTiers;

use crate::kinds::{self, PadButton, Tiers};
use crate::router::{PadEvent, Router};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("emusen-kit-qt/src/kinds.cxxqt.h");
        #[namespace = "Pad"]
        type PadButton = crate::kinds::ffi::PadButton;
        #[namespace = "Repeat"]
        type Tiers = crate::kinds::ffi::Tiers;
    }

    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        #[qproperty(Tiers, tiers)]
        #[qproperty(bool, swap_accept_back)]
        #[qproperty(bool, clock_running)]
        #[qproperty(bool, chord_only)]
        #[qproperty(bool, pending, READ, NOTIFY)]
        type PadRouter = super::PadRouterRust;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        /// A press of the grammar: the button after the player's swap, whether it is a repeat, and how many items a fast repeat jumps.
        #[qsignal]
        fn pressed(self: Pin<&mut PadRouter>, button: PadButton, repeat: bool, jump: i32);

        /// The guide button, or Options and Menu together, went down.
        #[qsignal]
        fn menu_chord(self: Pin<&mut PadRouter>);

        /// Queues a button going down or up; it takes effect at the next pump.
        #[qinvokable]
        fn push(self: Pin<&mut PadRouter>, button: PadButton, down: bool);

        /// Applies the queue and emits this tick's presses on the router's own clock.
        #[qinvokable]
        fn pump(self: Pin<&mut PadRouter>);

        /// The same at a stated time in milliseconds, for tests.
        #[qinvokable]
        fn pump_at(self: Pin<&mut PadRouter>, now_ms: i32);

        #[qinvokable]
        fn is_held(self: &PadRouter, button: PadButton) -> bool;
    }

    impl cxx_qt::Initialize for PadRouter {}
}

pub struct PadRouterRust {
    tiers: Tiers,
    swap_accept_back: bool,
    clock_running: bool,
    chord_only: bool,
    pending: bool,
    router: Router,
    started: Instant,
}

impl Default for PadRouterRust {
    fn default() -> Self {
        PadRouterRust {
            tiers: Tiers::Own,
            swap_accept_back: false,
            clock_running: true,
            chord_only: false,
            pending: false,
            router: Router::new(RepeatTiers::OWN),
            started: Instant::now(),
        }
    }
}

impl cxx_qt::Initialize for qobject::PadRouter {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut()
            .on_tiers_changed(|mut this| {
                let tiers = kinds::repeat_tiers(this.tiers);
                this.as_mut().rust_mut().router.set_tiers(tiers);
            })
            .release();
        self.as_mut()
            .on_swap_accept_back_changed(|mut this| {
                let swap = this.swap_accept_back;
                this.as_mut().rust_mut().router.set_swap_accept_back(swap);
            })
            .release();
        self.as_mut()
            .on_chord_only_changed(|mut this| {
                let on = this.chord_only;
                this.as_mut().rust_mut().router.set_chord_only(on);
            })
            .release();
    }
}

impl qobject::PadRouter {
    pub fn push(mut self: Pin<&mut Self>, button: PadButton, down: bool) {
        if let Some(button) = kinds::ui_button(button) {
            self.as_mut().rust_mut().router.push(PadEvent { button, down });
            self.update_pending();
        }
    }

    /// `pending` follows the router: a frame's clock runs only while a pump has work, so an idle screen is not woken.
    fn update_pending(mut self: Pin<&mut Self>) {
        let pending = self.router.pending();
        if pending != self.pending {
            self.as_mut().rust_mut().pending = pending;
            self.as_mut().pending_changed();
        }
    }

    pub fn pump(mut self: Pin<&mut Self>) {
        let now = self.started.elapsed();
        self.as_mut().pump_now(now);
    }

    pub fn pump_at(mut self: Pin<&mut Self>, now_ms: i32) {
        let now = Duration::from_millis(now_ms.max(0) as u64);
        self.as_mut().pump_now(now);
    }

    fn pump_now(mut self: Pin<&mut Self>, now: Duration) {
        let pumped = self.as_mut().rust_mut().router.pump(now);
        self.as_mut().update_pending();
        if pumped.menu_chord {
            self.as_mut().menu_chord();
        }
        for press in pumped.presses {
            let jump = i32::try_from(press.jump).unwrap_or(i32::MAX);
            self.as_mut().pressed(kinds::pad_button(press.button), press.repeat, jump);
        }
    }

    pub fn is_held(&self, button: PadButton) -> bool {
        kinds::ui_button(button).is_some_and(|button| self.router.is_held(button))
    }
}
