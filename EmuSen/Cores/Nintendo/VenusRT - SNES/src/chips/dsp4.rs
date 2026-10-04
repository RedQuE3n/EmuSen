//! The DSP-4's open replacement (Top Gear 3000), the limited step decided 2026-10-04: its protocol, 00h exact, and the
//! short commands' transfers and timing with their results not yet characterised (VenusRT_Native.md §48.3).

use super::dsphle::{DspHle, Next, Stage};

const RQM: u16 = 0x8000;
const DRS: u16 = 0x1000;

/// What the chip waits on before its next edge.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Wait {
    #[default]
    None,
    /// The edge, its work and its notice.
    Edge(Next, u16, u16),
    /// The last input, taken at the S-CPU's completion without an edge; then the results.
    Silent,
}

#[derive(Clone, Debug, Default)]
pub(super) struct Dsp4 {
    wait: Wait,
    base: u8,
    inputs: [u16; 8],
    taken: u16,
    want: u16,
    reads: u16,
    outputs: [u16; 16],
    total: u16,
    given: u16,
    /// Edges made since the command's offer.
    k: u16,
    /// The last input is taken from DR at the first result's edge, which does not wait for it.
    late_input: bool,
}

impl Dsp4 {
    pub(super) fn pack(&self, o: &mut Vec<u8>) {
        let (w, n, v, wk, wn) = match self.wait {
            Wait::None => (0u8, 0u8, 0u16, 0u16, 0u16),
            Wait::Silent => (1, 0, 0, 0, 0),
            Wait::Edge(next, wk, wn) => match next {
                Next::Request { .. } => (2, 1, 0, wk, wn),
                Next::Offer(v) => (2, 2, v, wk, wn),
                _ => (2, 3, 0, wk, wn),
            },
        };
        o.extend([w, n, self.base | (self.late_input as u8) << 7]);
        for x in [v, wk, wn, self.taken, self.want, self.reads, self.total, self.given, self.k].into_iter().chain(self.inputs).chain(self.outputs) {
            o.extend(x.to_le_bytes());
        }
    }

    pub(super) fn unpack(&mut self, d: &[u8]) {
        let x = |i: usize| u16::from_le_bytes([d[3 + 2 * i], d[4 + 2 * i]]);
        self.wait = match d[0] {
            1 => Wait::Silent,
            2 => Wait::Edge([Next::Request { wide: true }, Next::Offer(x(0)), Next::Idle][d[1] as usize - 1], x(1), x(2)),
            _ => Wait::None,
        };
        (self.base, self.late_input) = (d[2] & 0x7F, d[2] & 0x80 != 0);
        (self.taken, self.want, self.reads, self.total, self.given, self.k) = (x(3), x(4), x(5), x(6), x(7), x(8));
        for i in 0..8 {
            self.inputs[i] = x(9 + i);
        }
        for i in 0..16 {
            self.outputs[i] = x(17 + i);
        }
    }
}

impl DspHle {
    fn d4_note(&mut self, place: u8, write: Option<u16>) {
        self.edges += 1;
        self.edge_place = place;
        self.edge_write = write;
    }

    /// The edge `next` made now; 16-bit throughout (fullsnes).
    pub(super) fn d4_edge(&mut self, next: Next) {
        self.rise = self.cycles;
        self.sr = RQM | (self.sr & DRS);
        let d = &mut self.d4;
        match next {
            Next::Request { .. } => {
                let t = d.taken as usize;
                if t < 8 {
                    d.inputs[t] = self.dr;
                }
                d.taken += 1;
                d.k += 1;
                self.d4_note(2, None);
                self.d4_after_read();
            }
            Next::Offer(v) => {
                self.dr = v;
                let v = if self.d4.late_input {
                    let d = &mut self.d4;
                    d.late_input = false;
                    let t = d.taken as usize;
                    if t < 8 {
                        d.inputs[t] = self.dr;
                    }
                    d.taken += 1;
                    self.d4_compute();
                    self.d4.outputs[0]
                } else {
                    v
                };
                self.dr = v;
                self.d4_note(2, Some(v));
                let d = &mut self.d4;
                if d.k > 0 || d.want == 0 {
                    d.given += 1;
                }
                d.k += 1;
                if d.want > 0 && d.k == 1 {
                    self.d4_after_offer();
                } else if self.d4.given < self.d4.total {
                    let v = self.d4.outputs.get(self.d4.given as usize).copied().unwrap_or(0);
                    self.d4_plan(Next::Offer(v), false);
                } else {
                    self.d4_plan(Next::Idle, false);
                }
            }
            Next::Idle => {
                self.stage = Stage::Idle;
                self.dr = 0xFFFF;
                self.d4_note(0, Some(0xFFFF));
                self.d4.wait = Wait::None;
            }
            _ => {}
        }
    }

    /// The S-CPU completed a transfer while RQM was up; at idle, DR's low byte is the next command, a read included.
    pub(super) fn d4_completed(&mut self) {
        if self.stage == Stage::Idle {
            self.d4_command(self.dr as u8);
            return;
        }
        match std::mem::take(&mut self.d4.wait) {
            Wait::None => {}
            Wait::Edge(next, work, notice) => self.d4_at(next, work, notice),
            Wait::Silent => {
                let d = &mut self.d4;
                let t = d.taken as usize;
                if t < 8 {
                    d.inputs[t] = self.dr;
                }
                d.taken += 1;
                self.d4_inputs_done(true);
            }
        }
    }

    fn d4_at(&mut self, next: Next, work: u16, notice: u16) {
        self.next = next;
        self.due = (self.rise + work as u64).max(self.cycles + notice as u64);
    }

    fn d4_plan(&mut self, next: Next, at_completion: bool) {
        let (work, notice, waits) = self.d4_phase(self.d4.k + 1);
        if at_completion {
            self.d4_at(next, work, notice);
        } else if waits {
            self.d4.wait = Wait::Edge(next, work, notice);
        } else {
            self.schedule(next, work as u64);
        }
    }

    fn d4_command(&mut self, c: u8) {
        let base = if c >= 0x20 { 0x10 | (c & 15) } else if c == 0x0E { 0x03 } else { c };
        self.command = c;
        self.stage = Stage::Taking { want: 0, taken: 0 };
        // Inputs in all, and how many are read by an edge; the commands not listed take none (§48.3).
        let (want, reads) = match base {
            0x00 => (2, 2),
            0x0A => (4, 3),
            0x11 => (4, 4),
            0x0B | 0x0C => (3, 2),
            _ => (0, 0),
        };
        self.d4 = Dsp4 { base, want, reads, ..Dsp4::default() };
        let (total, first) = match base {
            0x02 | 0x12 | 0x14 => (1, [13, 4119, 12][(base == 0x12) as usize + 2 * (base == 0x14) as usize]),
            0x06 => (16, 14),
            0x13 => (1024, 15),
            _ => (0, 0),
        };
        if want > 0 {
            let notice = match base {
                0x0B | 0x0C => 14,
                0x11 => 12,
                _ => 13,
            };
            self.d4_at(Next::Offer(0), 0, notice);
        } else if total > 0 {
            self.d4.total = total;
            if base == 0x14 {
                self.d4.outputs[0] = 0x0400;
            }
            self.d4_at(Next::Offer(self.d4.outputs[0]), 0, first);
        } else {
            let notice = match base {
                0x03 => 377,
                0x05 => 52,
                0x1F => 12,
                _ => 13,
            };
            self.d4_at(Next::Idle, 0, notice);
        }
    }

    /// The offer was written; the S-CPU writes the first input over it.
    fn d4_after_offer(&mut self) {
        if self.d4.reads > 0 {
            self.d4_plan(Next::Request { wide: true }, false);
        } else {
            self.d4.wait = Wait::Silent;
        }
    }

    fn d4_after_read(&mut self) {
        let d = &self.d4;
        if d.taken < d.reads {
            self.d4_plan(Next::Request { wide: true }, false);
        } else if d.taken < d.want {
            let (work, _, waits) = self.d4_phase(self.d4.k + 1);
            if waits {
                self.d4.wait = Wait::Silent;
            } else {
                self.d4.late_input = true;
                self.schedule(Next::Offer(0), work as u64);
            }
        } else {
            self.d4_inputs_done(false);
        }
    }

    fn d4_inputs_done(&mut self, at_completion: bool) {
        self.d4_compute();
        let v = self.d4.outputs[0];
        self.d4_plan(Next::Offer(v), at_completion);
    }

    fn d4_compute(&mut self) {
        let d = &mut self.d4;
        d.outputs = [0; 16];
        match d.base {
            // fullsnes's multiplier, K·L·2, halved arithmetically: K·L as 32 bits, low word first (§45.2).
            0x00 => {
                let p = (d.inputs[0] as i16 as i32).wrapping_mul(d.inputs[1] as i16 as i32);
                (d.outputs[0], d.outputs[1], d.total) = (p as u16, (p >> 16) as u16, 2);
            }
            0x0A => d.total = 4,
            _ => d.total = 1,
        }
    }

    /// Edge `k`'s work, notice and wait, counted from the command's offer (k = 1 its first edge after it), measured.
    fn d4_phase(&self, k: u16) -> (u16, u16, bool) {
        let d = &self.d4;
        let (r, t) = (d.reads + 1, d.total.max(1));
        if k <= r {
            return if d.base == 0x0A && k == r { (21, 2, true) } else { (0, 2, true) };
        }
        let j = k - r - 1;
        if d.want == 0 {
            let idle = k > d.total;
            return (0, if matches!(d.base, 0x06 | 0x13) && idle || d.base == 0x13 { 4 } else { 3 }, true);
        }
        if j >= t {
            return (0, 3, true);
        }
        match (d.base, j) {
            // The first result comes unhandshaken after the last input's read, a cycle later when its bit 15 is set.
            (0x00, 0) => (7 + (d.outputs[0] >> 15), 0, false),
            (0x00, _) => (0, 2, true),
            (0x11, _) => (6, 0, false),
            (0x0A, 0) => (23, 0, false),
            (0x0A, _) => (0, 7, true),
            (0x0B | 0x0C, _) => (0, 17, true),
            _ => (0, 2, true),
        }
    }
}
