//! The H-blank and V-blank stand-in (Sapphire_Native.md §20.5): the two signals the EE timers count and gate on and the
//! INTC's V-blank sources take, generated from a television standard's line and field until the PCRTC (S20) drives
//! them. A field starts at time 0 with line 0; H-blank is high for the first `clock::HBLANK` bus cycles of each line,
//! V-blank from line `active` to the field's end.

use crate::clock::{self, BUS, Standard};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Blank {
    pub standard: Standard,
}

/// What changes at an edge.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Edges {
    pub h_rise: bool,
    pub h_fall: bool,
    pub v_rise: bool,
    pub v_fall: bool,
}

impl Blank {
    pub fn new(standard: Standard) -> Blank {
        Blank { standard }
    }

    fn line(&self) -> u64 {
        self.standard.line * BUS
    }

    fn field(&self) -> u64 {
        self.line() * self.standard.lines
    }

    pub fn h(&self, t: u64) -> bool {
        t % self.line() < clock::HBLANK * BUS
    }

    pub fn v(&self, t: u64) -> bool {
        t % self.field() >= self.standard.active * self.line()
    }

    /// The first time after `t` at which a signal changes; the H-blank's edges only when `want_h`, since a V-blank edge
    /// falls on a line's start.
    pub fn next_edge(&self, t: u64, want_h: bool) -> u64 {
        let field = self.field();
        let base = t - t % field;
        let mut next = base + self.standard.active * self.line();
        if next <= t {
            next = base + field;
        }
        if want_h {
            let line = self.line();
            let lb = t - t % line;
            let fall = lb + clock::HBLANK * BUS;
            next = next.min(if fall > t { fall } else { lb + line });
        }
        next
    }

    /// The edges at exactly `t`.
    pub fn edges(&self, t: u64) -> Edges {
        let line = self.line();
        let field = self.field();
        let at_line = t.is_multiple_of(line);
        Edges {
            h_rise: at_line,
            h_fall: t % line == clock::HBLANK * BUS,
            v_rise: t % field == self.standard.active * line,
            v_fall: t.is_multiple_of(field),
        }
    }
}
