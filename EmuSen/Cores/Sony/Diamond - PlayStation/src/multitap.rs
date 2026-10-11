//! The multitap (SCPH-1070) on one port (psx-spx "Controller and Memory Card Multitap Adaptor"): four controller and
//! four card slots, addressed 01h-04h and 81h-84h, and slot A's long read of all four by the request bit of the third
//! byte, toggling as psx-spx's table gives. Diamond_Pad.md §9.

use crate::card::Card;
use crate::pad::{Controller, HIGH_Z, Reply};

/// The multitap's own ID, sent before the four slots' data in a long read.
pub const TAP_ID: u8 = 0x80;

/// What slot A's access returned last.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Answer {
    #[default]
    SlotA,
    Long,
    Garbage,
}

#[derive(Clone, Debug)]
pub struct Multitap {
    pub pads: [Option<Controller>; 4],
    pub cards: [Option<Card>; 4],
    /// The request bit the last access to slot A carried, what that access returned, and what this one returns.
    pub request: bool,
    pub last: Answer,
    pub now: Answer,
    /// A long read's bytes after the address: the ID, 5Ah, and each slot's four halfwords.
    pub long: [u8; 34],
}

impl Multitap {
    pub fn new(pads: [Option<Controller>; 4]) -> Multitap {
        Multitap { pads, cards: [None, None, None, None], request: false, last: Answer::SlotA, now: Answer::SlotA, long: [0xFF; 34] }
    }

    /// Slot A addressed: what this access returns, decided by the last one; false when nobody answers, since an empty
    /// slot A ends the access after its address.
    pub fn select_a(&mut self) -> bool {
        if self.pads[0].is_none() {
            self.now = Answer::SlotA;
            return false;
        }
        self.now = match (self.request, self.last) {
            (false, _) => Answer::SlotA,
            (true, Answer::SlotA | Answer::Garbage) => Answer::Long,
            (true, Answer::Long) => Answer::Garbage,
        };
        self.request = false;
        self.last = self.now;
        if self.now == Answer::Long {
            self.long[0] = TAP_ID;
            self.long[1] = 0x5A;
            for (i, p) in self.pads.iter().enumerate() {
                self.long[2 + 8 * i..10 + 8 * i].copy_from_slice(&p.map_or([0xFF; 8], |p| p.report()));
            }
        }
        true
    }

    /// Slot A's byte at `step`; the third byte's bit 0 is the request for the next access.
    pub fn exchange_a(&mut self, step: u8, byte: u8) -> (Reply, bool) {
        if step == 2 {
            self.request = byte & 1 != 0;
        }
        match self.now {
            Answer::SlotA => self.pads[0].as_mut().map_or((HIGH_Z, false), |p| p.exchange(step, byte)),
            Answer::Long => match step {
                1..=33 => (Reply::more(self.long[step as usize - 1]), true),
                34 => (Reply::last(self.long[33]), false),
                _ => (HIGH_Z, false),
            },
            Answer::Garbage => match step {
                1 => (Reply::more(TAP_ID), true),
                2 => (Reply::more(0x5A), true),
                3 => (Reply::last(self.pads[0].map_or(0xFF, |p| p.report()[0])), false),
                _ => (HIGH_Z, false),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pad::DigitalPad;

    fn access(t: &mut Multitap, bytes: &[u8]) -> Vec<u8> {
        if !t.select_a() {
            return vec![];
        }
        let mut out = Vec::new();
        for (i, &b) in bytes.iter().enumerate() {
            let (r, more) = t.exchange_a(i as u8 + 1, b);
            out.push(r.byte);
            if !more {
                break;
            }
        }
        out
    }

    #[test]
    fn the_request_bit_toggles_slot_a_long_and_garbage_as_psx_spx_tabulates() {
        let pad = |p: u16| Some(Controller::Digital(DigitalPad { pressed: p }));
        let mut t = Multitap::new([pad(0), None, pad(1 << 3), None]);
        let read = |req: u8| {
            let mut v = vec![0x42, req];
            v.resize(34, 0);
            v
        };
        assert_eq!(access(&mut t, &read(1)), [0x41, 0x5A, 0xFF, 0xFF], "the request leaves this access as slot A's");
        let long = access(&mut t, &read(0));
        assert_eq!(long.len(), 34);
        assert_eq!(long[..10], [0x80, 0x5A, 0x41, 0x5A, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
        assert_eq!((long[10..18].to_vec(), long[18..22].to_vec()), (vec![0xFF; 8], vec![0x41, 0x5A, 0xF7, 0xFF]), "an empty slot B, slot C's Start");
        assert_eq!(access(&mut t, &read(1)), [0x41, 0x5A, 0xFF, 0xFF], "no request last time");
        assert_eq!(access(&mut t, &read(1)).len(), 34);
        assert_eq!(access(&mut t, &read(1)), [0x80, 0x5A, 0x41], "requesting every time: garbage after a long read");
        assert_eq!(access(&mut t, &read(0)).len(), 34);
        t.pads[0] = None;
        assert!(access(&mut t, &read(1)).is_empty(), "an empty slot A ends at the address");
    }
}
