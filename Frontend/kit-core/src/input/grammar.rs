use std::time::Duration;

use super::UiButton;

/// The faster tier a held direction reaches after a while: from when, how often, and how many items the switch moves at once.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FastTier {
    pub after: Duration,
    pub interval: Duration,
    pub jump: u32,
}

/// A held button's repeats: a first delay, an interval, and a faster tier if one was measured (EmuSen_BigPicture.md §14.7).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RepeatTiers {
    pub delay: Duration,
    pub interval: Duration,
    pub fast: Option<FastTier>,
}

impl RepeatTiers {
    /// The interface's own: 400 ms, then every 80 ms, no faster tier (EmuSen_Settings_Reference.md §4.29).
    pub const OWN: RepeatTiers = RepeatTiers { delay: Duration::from_millis(400), interval: Duration::from_millis(80), fast: None };

    /// A text list held, as ES-DE 3.4.1 measured: 500 ms, 114 ms, a four-entry step at 1.703 s, then every 15.9 ms.
    pub const ESDE_LIST: RepeatTiers = RepeatTiers {
        delay: Duration::from_millis(500),
        interval: Duration::from_millis(114),
        fast: Some(FastTier { after: Duration::from_millis(1703), interval: Duration::from_micros(15_900), jump: 4 }),
    };

    /// A carousel held with `fastScrolling`: 500 ms, 180 ms, then 80 ms from 1.58 s.
    pub const ESDE_CAROUSEL_FAST: RepeatTiers = RepeatTiers {
        delay: Duration::from_millis(500),
        interval: Duration::from_millis(180),
        fast: Some(FastTier { after: Duration::from_millis(1580), interval: Duration::from_millis(80), jump: 1 }),
    };

    /// A carousel held without it, and a grid: 500 ms, then a constant 200 ms.
    pub const ESDE_CAROUSEL: RepeatTiers =
        RepeatTiers { delay: Duration::from_millis(500), interval: Duration::from_millis(200), fast: None };
}

/// One press the grammar made: the button, whether it came from holding, and how many items it moves (more than one only at a fast tier's switch).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Press {
    pub button: UiButton,
    pub repeat: bool,
    pub jump: u32,
}

/// The stick's travel, 0 to 1, past which it is a held direction.
pub const STICK_THRESHOLD: f64 = 0.55;

/// A stick axis as a direction: -1, 0 or 1.
pub fn stick_direction(value: f64) -> i8 {
    if value >= STICK_THRESHOLD {
        1
    } else if value <= -STICK_THRESHOLD {
        -1
    } else {
        0
    }
}

const COUNT: usize = UiButton::ALL.len();

/// Held buttons turned into presses, on the host's clock: one on the way down, and for the ones that repeat, more while held.
#[derive(Clone, Debug)]
pub struct Grammar {
    tiers: RepeatTiers,
    held: [bool; COUNT],
    since: [Duration; COUNT],
    next: [Duration; COUNT],
    fast: [bool; COUNT],
    chord_held: bool,
    presses: Vec<Press>,
}

impl Grammar {
    pub fn new(tiers: RepeatTiers) -> Grammar {
        Grammar {
            tiers,
            held: [false; COUNT],
            since: [Duration::ZERO; COUNT],
            next: [Duration::ZERO; COUNT],
            fast: [false; COUNT],
            chord_held: false,
            presses: Vec::with_capacity(COUNT),
        }
    }

    pub fn tiers(&self) -> RepeatTiers {
        self.tiers
    }

    /// Changes the tiers for the next hold; a hold in progress keeps the ones it began with.
    pub fn set_tiers(&mut self, tiers: RepeatTiers) {
        self.tiers = tiers;
    }

    pub fn is_held(&self, button: UiButton) -> bool {
        self.held[button.index()]
    }

    /// Whether a held button has reached its faster tier.
    pub fn is_fast(&self, button: UiButton) -> bool {
        self.fast[button.index()]
    }

    /// The presses this tick, in the order the buttons are declared; every repeat that fell due since the last tick is given.
    pub fn feed(&mut self, held: impl Fn(UiButton) -> bool, now: Duration) -> &[Press] {
        self.presses.clear();
        for button in UiButton::ALL {
            let i = button.index();
            let down = held(button);
            if down && !self.held[i] {
                self.presses.push(Press { button, repeat: false, jump: 1 });
                self.since[i] = now;
                self.next[i] = now + self.tiers.delay;
                self.fast[i] = false;
            } else if down && button.repeats() {
                self.repeats_due(button, now);
            }
            self.held[i] = down;
        }
        &self.presses
    }

    fn repeats_due(&mut self, button: UiButton, now: Duration) {
        let i = button.index();
        let tiers = self.tiers;
        loop {
            if let (false, Some(fast)) = (self.fast[i], tiers.fast) {
                let at = self.since[i] + fast.after;
                if at <= self.next[i] && at <= now {
                    self.fast[i] = true;
                    self.next[i] = at + fast.interval;
                    self.presses.push(Press { button, repeat: true, jump: fast.jump.max(1) });
                    continue;
                }
            }
            if self.next[i] > now {
                return;
            }
            let due = self.next[i];
            self.next[i] = due
                + match (self.fast[i], tiers.fast) {
                    (true, Some(fast)) => fast.interval,
                    _ => tiers.interval,
                };
            self.presses.push(Press { button, repeat: true, jump: 1 });
        }
    }

    /// True once, as the menu chord goes down: the guide button, or the two middle buttons together (§4.29).
    pub fn menu_chord(&mut self, held: impl Fn(UiButton) -> bool) -> bool {
        let chord = Self::chord(&held);
        let pressed = chord && !self.chord_held;
        self.chord_held = chord;
        pressed
    }

    /// Nothing is a press until it has been let go, so the button that closed a menu does not reach what is under it.
    pub fn forget(&mut self, held: impl Fn(UiButton) -> bool) {
        for button in UiButton::ALL {
            self.held[button.index()] = held(button);
        }
        self.chord_held = Self::chord(&held);
    }

    fn chord(held: &impl Fn(UiButton) -> bool) -> bool {
        held(UiButton::Guide) || (held(UiButton::Options) && held(UiButton::Menu))
    }
}
