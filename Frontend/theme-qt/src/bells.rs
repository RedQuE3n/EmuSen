//! `Bells`, the kit's ladder of bells as a theme rings it (EmuSen_Qt.md §3.2): `ladder(count)` asks the host to have the
//! first `count` rungs rendered, `strike(i)` rings rung i, as Moonrise rings its world i. Each is handed to the host's
//! striker, which renders through the kit's `Bells` and rings at the Navigation volume. With no striker, silence.

use core::pin::Pin;
use cxx_qt::CxxQtType;

#[cxx_qt::bridge]
pub mod qobject {
    extern "RustQt" {
        #[qobject]
        #[qml_element = "Bells"]
        #[qml_singleton]
        type ThemeBells = super::ThemeBellsRust;
    }

    extern "RustQt" {
        /// The ladder's first `count` rungs made ready; the count, 0 for none.
        #[qinvokable]
        fn ladder(self: Pin<&mut ThemeBells>, count: i32) -> i32;
        /// Rings rung `i` of the ladder.
        #[qinvokable]
        fn strike(self: Pin<&mut ThemeBells>, i: i32);
    }
}

/// The host's side: renders and rings the ladder's bells.
pub trait Striker {
    fn ladder(&mut self, count: usize);
    fn strike(&mut self, rung: usize);
}

#[derive(Default)]
pub struct ThemeBellsRust {
    striker: Option<Box<dyn Striker>>,
}

impl qobject::ThemeBells {
    /// The host's striker; none is silence.
    pub fn set_striker(mut self: Pin<&mut Self>, striker: Option<Box<dyn Striker>>) {
        self.as_mut().rust_mut().striker = striker;
    }

    pub fn ladder(mut self: Pin<&mut Self>, count: i32) -> i32 {
        let count = count.max(0);
        if let Some(striker) = self.as_mut().rust_mut().striker.as_mut() {
            striker.ladder(count as usize);
        }
        count
    }

    pub fn strike(mut self: Pin<&mut Self>, i: i32) {
        if i < 0 {
            return;
        }
        if let Some(striker) = self.as_mut().rust_mut().striker.as_mut() {
            striker.strike(i as usize);
        }
    }
}
