//! `Bells`: the cues rendered through kit-core and kept; `BellPlayer` in emusen-qt-endymion plays them through Endymion's interface player (EmuSen_Kit.md §7.5, §7.15).

use core::pin::Pin;

use cxx_qt::CxxQtType;
use emusen_kit_core::sound::{self, RATE};

use crate::bellcache::BellCache;
use crate::kinds::{self, Cue};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("emusen-kit-qt/src/kinds.cxxqt.h");
        #[namespace = "Cues"]
        type Cue = crate::kinds::ffi::Cue;
    }

    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(i32, renders, READ, NOTIFY)]
        #[qproperty(i32, rate, READ, CONSTANT)]
        type Bells = super::BellsRust;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        /// Renders a cue the first time and keeps it; the number of stereo frames.
        #[qinvokable]
        fn prepare(self: Pin<&mut Bells>, cue: Cue) -> i32;
        /// Every cue and the quick select, rendered once.
        #[qinvokable]
        fn prepare_all(self: Pin<&mut Bells>);
        #[qinvokable]
        fn duration_ms(self: Pin<&mut Bells>, cue: Cue) -> i32;
        /// The cue's loudest sample, 0 to 1.
        #[qinvokable]
        fn peak(self: Pin<&mut Bells>, cue: Cue) -> f64;
        #[qinvokable]
        fn quick_select_frames(self: Pin<&mut Bells>) -> i32;
    }

    impl cxx_qt::Initialize for Bells {}
}

pub struct BellsRust {
    renders: i32,
    rate: i32,
    cache: BellCache,
}

impl Default for BellsRust {
    fn default() -> Self {
        BellsRust { renders: 0, rate: RATE as i32, cache: BellCache::default() }
    }
}

impl cxx_qt::Initialize for qobject::Bells {
    fn initialize(self: Pin<&mut Self>) {}
}

impl qobject::Bells {
    fn count_renders(mut self: Pin<&mut Self>) {
        let renders = self.cache.renders() as i32;
        if self.renders != renders {
            self.as_mut().rust_mut().renders = renders;
            self.as_mut().renders_changed();
        }
    }

    pub fn prepare(mut self: Pin<&mut Self>, cue: Cue) -> i32 {
        let Some(cue) = kinds::sound_cue(cue) else { return 0 };
        let frames = self.as_mut().rust_mut().cache.cue(cue).len() / 2;
        self.count_renders();
        frames as i32
    }

    pub fn prepare_all(mut self: Pin<&mut Self>) {
        for cue in sound::Cue::ALL {
            self.as_mut().rust_mut().cache.cue(cue);
        }
        self.as_mut().rust_mut().cache.quick_select();
        self.count_renders();
    }

    pub fn duration_ms(mut self: Pin<&mut Self>, cue: Cue) -> i32 {
        let Some(cue) = kinds::sound_cue(cue) else { return 0 };
        let ms = BellCache::duration_ms(self.as_mut().rust_mut().cache.cue(cue));
        self.count_renders();
        ms
    }

    pub fn peak(mut self: Pin<&mut Self>, cue: Cue) -> f64 {
        let Some(cue) = kinds::sound_cue(cue) else { return 0.0 };
        let peak = f64::from(sound::loudest(self.as_mut().rust_mut().cache.cue(cue)));
        self.count_renders();
        peak
    }

    pub fn quick_select_frames(mut self: Pin<&mut Self>) -> i32 {
        let frames = self.as_mut().rust_mut().cache.quick_select().len() / 2;
        self.count_renders();
        frames as i32
    }
}
