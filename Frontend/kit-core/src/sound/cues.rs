use super::bell::{Bell, Chime, render_bells};

/// The interface's nine cues, every one Moonrise's bell on its ladder (EmuSen_Interface.md §5.6, LunaP §220).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Cue {
    /// The focus moved one place.
    Move,
    /// A section or a console was stepped.
    Section,
    /// Something was chosen.
    Choose,
    /// Back one level.
    Back,
    /// A favourite was marked or cleared.
    Favourite,
    /// A game is starting.
    Launch,
    /// A state or a setting was saved.
    Saved,
    /// A state was loaded.
    Loaded,
    /// A press was refused.
    Refused,
}

/// The cues' volume until the player changes it.
pub const DEFAULT_VOLUME: f64 = 0.6;

/// Whether the cues are on until the player turns them off.
pub const ON_BY_DEFAULT: bool = true;

/// How far under the game a cue that may play over gameplay is mixed, in decibels.
pub const UNDER_GAMEPLAY_DECIBELS: f64 = -12.0;

/// ES-DE's quick system select, which has no cue of its own: E5 then A5.
pub const QUICK_SELECT: [Bell; 2] = [Bell::new(4, 0.0, 0.45, 0.22), Bell::new(6, 0.06, 0.55, 0.22)];

const MOVE: [Bell; 1] = [Bell::new(8, 0.0, 0.3, 0.12)];
const SECTION: [Bell; 1] = [Bell::new(5, 0.0, 0.55, 0.18)];
const CHOOSE: [Bell; 2] = [Bell::new(5, 0.0, 0.5, 0.22), Bell::new(8, 0.07, 0.6, 0.22)];
const BACK: [Bell; 2] = [Bell::new(8, 0.0, 0.45, 0.18), Bell::new(5, 0.07, 0.5, 0.18)];
const FAVOURITE: [Bell; 4] =
    [Bell::new(5, 0.0, 0.55, 0.17), Bell::new(7, 0.055, 0.55, 0.17), Bell::new(8, 0.11, 0.55, 0.17), Bell::new(10, 0.165, 0.55, 0.17)];
const LAUNCH: [Bell; 6] = [
    Bell::new(0, 0.0, 0.7, 0.24),
    Bell::new(1, 0.075, 0.7, 0.24),
    Bell::new(2, 0.15, 0.7, 0.24),
    Bell::new(3, 0.225, 0.7, 0.24),
    Bell::new(4, 0.3, 0.7, 0.24),
    Bell::new(5, 0.375, 1.2, 0.24),
];
const SAVED: [Bell; 2] = [Bell::new(7, 0.0, 0.45, 0.18), Bell::new(9, 0.08, 0.6, 0.18)];
const LOADED: [Bell; 2] = [Bell::new(9, 0.0, 0.45, 0.18), Bell::new(7, 0.08, 0.6, 0.18)];
const REFUSED: [Bell; 2] = [Bell::dull(0, 0.0, 0.25, 0.26, 900.0), Bell::dull(0, 0.11, 0.25, 0.22, 900.0)];

impl Cue {
    pub const ALL: [Cue; 9] =
        [Cue::Move, Cue::Section, Cue::Choose, Cue::Back, Cue::Favourite, Cue::Launch, Cue::Saved, Cue::Loaded, Cue::Refused];

    /// The strikes a cue is made of: rung, start, decay scale, gain (LunaP §220.2).
    pub const fn bells(self) -> &'static [Bell] {
        match self {
            Cue::Move => &MOVE,
            Cue::Section => &SECTION,
            Cue::Choose => &CHOOSE,
            Cue::Back => &BACK,
            Cue::Favourite => &FAVOURITE,
            Cue::Launch => &LAUNCH,
            Cue::Saved => &SAVED,
            Cue::Loaded => &LOADED,
            Cue::Refused => &REFUSED,
        }
    }

    /// Whether a cue may play over running gameplay, under the game; the others play only in menus and cards.
    pub const fn plays_over_gameplay(self) -> bool {
        matches!(self, Cue::Saved | Cue::Loaded)
    }

    /// The cue as 48 kHz stereo samples; a host renders each once and keeps it.
    pub fn samples(self) -> Vec<f32> {
        render_bells(&Chime::MOONRISE, self.bells())
    }
}

/// The quick system select's samples.
pub fn quick_select_samples() -> Vec<f32> {
    render_bells(&Chime::MOONRISE, &QUICK_SELECT)
}

/// One bell on a ladder rung, as a console rings its own note in a picker: the same rung it rings in Moonrise (LunaP §220.3).
pub fn bell_samples(rung: usize) -> Vec<f32> {
    render_bells(&Chime::MOONRISE, &[Bell::new(rung, 0.0, 0.6, 0.18)])
}
