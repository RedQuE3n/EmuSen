//! The pad router's heart, with no Qt: a queue of pad events, the held buttons and kit-core's grammar on a clock the host hands it (EmuSen_Kit.md §7.4).

use std::collections::VecDeque;
use std::time::Duration;

use emusen_kit_core::input::{Grammar, GuideChord, GuideChords, Press, RepeatTiers, UiButton};

use crate::qt_keys;

/// One pad event, as a test or a pad reader pushes it: a button went down or up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PadEvent {
    pub button: UiButton,
    pub down: bool,
}

/// What waits for a pump: a pad's button going down or up, or a key's press (`Router::push_key`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Queued {
    Pad(PadEvent),
    Key(UiButton),
}

/// What one pump of the router produced.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Pumped {
    pub presses: Vec<Press>,
    pub menu_chord: bool,
    /// The button Guide was chorded with (`GuideChords::BUTTONS`), while the Guide chords are read.
    pub guide_chord: Option<UiButton>,
    /// The pad's buttons let go at this pump whose press was raised, with the swap applied as it was to the press;
    /// raised before this pump's presses (EmuSen_Kit.md §7.28).
    pub released: Vec<UiButton>,
    /// The keys' presses at this pump, each a press and then its release, raised after the pad's presses, never swapped.
    pub keyed: Vec<UiButton>,
}

#[derive(Clone, Debug)]
pub struct Router {
    grammar: Grammar,
    held: [bool; 16],
    /// The buttons whose press was raised and not yet let go, so a release is raised once and only after a press.
    raised: [bool; 16],
    queue: VecDeque<Queued>,
    swap_accept_back: bool,
    chord_only: bool,
    /// Buttons held while only the chord left, kept from the grammar until let go.
    muted: [bool; 16],
    guide_chords: bool,
    guide: GuideChords,
}

impl Router {
    pub fn new(tiers: RepeatTiers) -> Router {
        Router { grammar: Grammar::new(tiers), held: [false; 16], raised: [false; 16], queue: VecDeque::new(), swap_accept_back: false, chord_only: false, muted: [false; 16], guide_chords: false, guide: GuideChords::default() }
    }

    pub fn set_tiers(&mut self, tiers: RepeatTiers) {
        self.grammar.set_tiers(tiers);
    }

    pub fn set_swap_accept_back(&mut self, swap: bool) {
        self.swap_accept_back = swap;
    }

    /// While on, a pump gives the menu chord alone, and a button held then is the grammar's only once let go and pressed
    /// again: a game on screen owns the pad (EmuSen_Kit.md §7.17).
    pub fn set_chord_only(&mut self, on: bool) {
        self.chord_only = on;
    }

    /// While on, and while a game owns the pad (`chord_only`), Guide is read for its chords: alone it opens the menu as it
    /// is let go, and Options with Menu open it at once only while Guide is up (EmuSen_Kit.md §7.26).
    pub fn set_guide_chords(&mut self, on: bool) {
        self.guide_chords = on;
    }

    /// Guide held while the Guide chords are read over a game, as the last pump left it: the game is to hear nothing
    /// then, as Mistress keeps the pad from it (EmuSen_Kit.md §7.26).
    pub fn guide_held(&self) -> bool {
        self.guide_chords && self.chord_only && self.held[UiButton::Guide.index()]
    }

    /// Whether a pump has work: an event queued or a button held, whose repeats and chord the clock times. With neither,
    /// a host's clock may sleep until the next push (EmuSen_Kit.md §7.4).
    pub fn pending(&self) -> bool {
        !self.queue.is_empty() || self.held.iter().any(|&down| down)
    }

    pub fn is_held(&self, button: UiButton) -> bool {
        self.held[button.index()]
    }

    /// Queues an event; nothing changes until the next pump, so a test states the order and the time exactly.
    pub fn push(&mut self, event: PadEvent) {
        self.queue.push_back(Queued::Pad(event));
    }

    /// Queues a key's press by the kit's key map (Qt's key number and modifiers, `qt_keys::press_of`); a key the map does
    /// not know is declined, false. A key is a press and its release at one pump, Return always Accept whatever the
    /// player's swap, and nothing while a game owns the pad (EmuSen_Kit.md §7.28).
    pub fn push_key(&mut self, qt_key: i32, modifiers: i32) -> bool {
        let Some(button) = qt_keys::press_of(qt_key, modifiers) else {
            return false;
        };
        self.queue.push_back(Queued::Key(button));
        true
    }

    /// Applies the queued events, then asks the grammar for this tick's presses, each with the player's swap applied.
    pub fn pump(&mut self, now: Duration) -> Pumped {
        let before = self.held;
        let mut keyed = Vec::new();
        // Guide is not part of the menu chord while it is read for its own chords over a game.
        let guide_read = self.guide_chords && self.chord_only;
        while let Some(queued) = self.queue.pop_front() {
            match queued {
                Queued::Pad(event) => {
                    self.held[event.button.index()] = event.down;
                    let held = self.held;
                    if !Grammar::chord_down(|button: UiButton| (button != UiButton::Guide || !guide_read) && held[button.index()]) {
                        self.grammar.chord_let_go();
                    }
                }
                Queued::Key(button) if !self.chord_only => keyed.push(button),
                Queued::Key(_) => {}
            }
        }
        let raw = self.held;
        let swap = self.swap_accept_back;
        let mut released = Vec::new();
        for button in UiButton::ALL {
            let i = button.index();
            if before[i] && !raw[i] && std::mem::take(&mut self.raised[i]) {
                released.push(button.swapped(swap));
            }
        }
        let mut guide_chord = None;
        let menu_chord = if self.guide_chords && self.chord_only {
            let chorded = self.guide.feed(|button: UiButton| raw[button.index()]);
            let pair = self.grammar.menu_chord(|button: UiButton| button != UiButton::Guide && raw[button.index()]);
            match chorded {
                Some(GuideChord::Alone) => true,
                Some(GuideChord::With(button)) => {
                    guide_chord = Some(button);
                    false
                }
                None => pair && !self.guide.guide_held(),
            }
        } else {
            self.guide.reset(raw[UiButton::Guide.index()]);
            self.grammar.menu_chord(|button: UiButton| raw[button.index()])
        };
        for (muted, &down) in self.muted.iter_mut().zip(&raw) {
            *muted = down && (*muted || self.chord_only);
        }
        let muted = self.muted;
        let is_held = |button: UiButton| raw[button.index()] && !muted[button.index()];
        if menu_chord {
            self.grammar.forget(is_held);
            self.raised = [false; 16];
        }
        let presses: Vec<Press> = self.grammar.feed(is_held, now).iter().map(|p| Press { button: p.button.swapped(swap), ..*p }).collect();
        for press in presses.iter().filter(|p| !p.repeat) {
            self.raised[press.button.swapped(swap).index()] = true;
        }
        Pumped { presses, menu_chord, guide_chord, released, keyed }
    }
}
