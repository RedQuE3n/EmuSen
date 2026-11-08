//! The pad router's heart, with no Qt: a queue of pad events, the held buttons and kit-core's grammar on a clock the host hands it (EmuSen_Kit.md §7.4).

use std::collections::VecDeque;
use std::time::Duration;

use emusen_kit_core::input::{Grammar, Press, RepeatTiers, UiButton};

/// One pad event, as a test or a pad reader pushes it: a button went down or up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PadEvent {
    pub button: UiButton,
    pub down: bool,
}

/// What one pump of the router produced.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Pumped {
    pub presses: Vec<Press>,
    pub menu_chord: bool,
}

#[derive(Clone, Debug)]
pub struct Router {
    grammar: Grammar,
    held: [bool; 16],
    queue: VecDeque<PadEvent>,
    swap_accept_back: bool,
}

impl Router {
    pub fn new(tiers: RepeatTiers) -> Router {
        Router { grammar: Grammar::new(tiers), held: [false; 16], queue: VecDeque::new(), swap_accept_back: false }
    }

    pub fn set_tiers(&mut self, tiers: RepeatTiers) {
        self.grammar.set_tiers(tiers);
    }

    pub fn set_swap_accept_back(&mut self, swap: bool) {
        self.swap_accept_back = swap;
    }

    pub fn is_held(&self, button: UiButton) -> bool {
        self.held[button.index()]
    }

    /// Queues an event; nothing changes until the next pump, so a test states the order and the time exactly.
    pub fn push(&mut self, event: PadEvent) {
        self.queue.push_back(event);
    }

    /// Applies the queued events, then asks the grammar for this tick's presses, each with the player's swap applied.
    pub fn pump(&mut self, now: Duration) -> Pumped {
        while let Some(event) = self.queue.pop_front() {
            self.held[event.button.index()] = event.down;
        }
        let held = self.held;
        let is_held = |button: UiButton| held[button.index()];
        let menu_chord = self.grammar.menu_chord(is_held);
        if menu_chord {
            self.grammar.forget(is_held);
        }
        let swap = self.swap_accept_back;
        let presses = self.grammar.feed(is_held, now).iter().map(|p| Press { button: p.button.swapped(swap), ..*p }).collect();
        Pumped { presses, menu_chord }
    }
}
