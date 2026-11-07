//! The cues and the bells as sample synthesis: Moonrise's bell on its ladder, rendered on demand and never recorded from anyone's sounds (EmuSen_Interface.md §5.6, LunaP §219.4 and §220, EmuSen_Kit.md §5).

mod bell;
mod cues;

pub use bell::{Bell, Chime, Partial, RATE, loudest, render, render_bells};
pub use cues::{Cue, DEFAULT_VOLUME, ON_BY_DEFAULT, QUICK_SELECT, UNDER_GAMEPLAY_DECIBELS, bell_samples, quick_select_samples};
