//! A theme's `fade` transitions as ES-DE 3.4.1 plays them: black over the whole screen between two views, and black
//! between a system's elements and the primary element between two systems (EmuSen_Qt.md §4.8.8).

use crate::loader::Color;

use super::draw::key;
use super::frame::{Paint, Placed};
use super::motion::{Motion, Time};

/// A fade between two views, from a time: the black rises, holds, and falls (ES-DE's `ViewController::playViewTransition`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewFade {
    at: Time,
    /// How far into its run it started: a fade begun while another was part way starts from that one's black.
    skip: Time,
    fade: Time,
    hold: Time,
}

impl ViewFade {
    /// A fade starting at `at` over whatever black is on the screen then.
    pub fn new(at: Time, black: f64, motion: &Motion) -> Self {
        let skip = (black.clamp(0.0, 1.0) * motion.view_fade as f64) as Time;
        ViewFade { at, skip, fade: motion.view_fade, hold: motion.view_fade_hold }
    }

    fn since(&self, now: Time) -> Time {
        now - self.at + self.skip
    }

    /// The black's opacity at a time; None once the fade has run.
    pub fn black(&self, now: Time) -> Option<f64> {
        let e = self.since(now).max(0);
        let fade = self.fade.max(1) as f64;
        if e < self.fade {
            Some(e as f64 / fade)
        } else if e < self.fade + self.hold {
            Some(1.0)
        } else if e < 2 * self.fade + self.hold {
            Some(1.0 - (e - self.fade - self.hold) as f64 / fade)
        } else {
            None
        }
    }

    /// Whether the view left is still the one shown: until the black first covers the screen, when ES-DE moves its camera.
    pub fn shows_leaving(&self, now: Time) -> bool {
        self.since(now) < self.fade
    }
}

/// A press of a direction held from a time until its release, which ES-DE's `SystemView::input` counts as navigating.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Navigating {
    pub pressed: Option<Time>,
    pub released: Option<Time>,
}

impl Navigating {
    pub fn at(&self, t: Time) -> bool {
        self.pressed.is_some_and(|p| p <= t) && self.released.is_none_or(|r| r > t)
    }
}

/// A fade between two systems in the system view (ES-DE's `SystemView::onCursorChanged`, its `FADE` animation).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SystemFade {
    at: Time,
    run: Time,
    /// The black when it started, which its first part rises from.
    start: f64,
    /// The system whose elements show until half way.
    pub from: usize,
    /// ES-DE's `mMaxFade` was set when this fade started, and stays until the black is whole again.
    held_over: bool,
}

impl SystemFade {
    /// A fade starting at `at` from the black on the screen then; `grid` when the primary element is a grid, and
    /// `held_over` when ES-DE's mMaxFade was still set.
    pub fn new(at: Time, from: usize, grid: bool, motion: &Motion, start: f64, held_over: bool) -> Self {
        let run = if grid { motion.grid_system_fade } else { motion.system_fade };
        SystemFade { at, run, start, from, held_over }
    }

    // The run's progress, 0 to 1.
    fn t(&self, now: Time) -> f64 {
        ((now - self.at) as f64 / self.run.max(1) as f64).clamp(0.0, 1.0)
    }

    /// The black over the system's elements below the primary (ES-DE 3.4.1 SystemView.cpp 449-458, mFadeOpacity).
    pub fn black(&self, now: Time) -> f64 {
        let t = self.t(now);
        if t < 0.3 {
            (t / 0.2 + self.start).clamp(0.0, 1.0)
        } else if t < 0.7 {
            1.0
        } else {
            1.0 - ((t - 0.6) / 0.3).clamp(0.0, 1.0)
        }
    }

    /// Whether the system left still shows: the camera moves to the new one past half way (SystemView.cpp 460).
    pub fn shows_from(&self, now: Time) -> bool {
        self.t(now) <= 0.5
    }

    pub fn is_running(&self, now: Time) -> bool {
        now < self.at + self.run
    }

    /// ES-DE's `mMaxFade` at a time: set from 0.7 of the run while a direction is held (SystemView.cpp 462-463), cleared
    /// once the black is whole again (471-472).
    pub fn max_fade(&self, now: Time, navigating: &Navigating) -> bool {
        let set = self.at + (self.run as f64 * 0.7).ceil() as Time;
        if now >= set && navigating.at(set) {
            return true;
        }
        let whole = self.at + (self.run as f64 * 0.2 * (1.0 - self.start)).max(0.0).ceil() as Time;
        self.held_over && now < whole
    }

    /// While a direction is held at max fade, the elements are not drawn at all (SystemView.cpp 191-192, 207).
    pub fn hides(&self, now: Time, navigating: &Navigating) -> bool {
        navigating.at(now) && self.max_fade(now, navigating)
    }
}

/// A black box over the whole screen at an opacity: ES-DE's `drawRect` of a fade, or its dimming of everything beneath
/// (core.glsl multiplies colour only, so dimming a stack by d is black at 1 - d over it).
pub fn veil(view: &str, w: f64, h: f64, opacity: f64) -> Placed {
    Placed {
        id: key(&[view, "fade", "veil"]),
        kind: "fade".to_owned(),
        name: "veil".to_owned(),
        x: 0.0,
        y: 0.0,
        w,
        h,
        rotation: 0.0,
        rotation_origin: (0.5, 0.5),
        opacity,
        // Black, a colour of the design: ES-DE's fadeColor is 0x00000000 with the opacity as alpha.
        paint: Paint::fill(Color(0x0000_00FF)),
    }
}
