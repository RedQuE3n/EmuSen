//! The DSP-2's open replacement (Dungeon Master): its commands as characterised and graded in VenusRT_Native.md §42,
//! over the ports of `dsphle`. Every edge is made at the later of its work after the last edge and its notice after
//! the S-CPU's completion, or, for the few phases that do not wait, at its work alone.

use super::dsphle::{DspHle, Next, Stage};

const RQM: u16 = 0x8000;
const USF1: u16 = 0x4000;
const USF0: u16 = 0x2000;
const DRS: u16 = 0x1000;
const DRC: u16 = 0x0400;

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

#[derive(Clone, Debug)]
pub(super) struct Dsp2 {
    /// A write of SR due at a cycle: DR to 8-bit (0) or 16-bit (1), USF1 cleared (2) or set to the overflow (3), USF0
    /// set while dividing (4) and cleared (5), the zero divisor's USF1 (6), and USF1 set while scaling (7) and cleared (8).
    pub(super) mode_at: Option<(u64, u8)>,
    overflow: bool,
    wait: Wait,
    base: u8,
    inputs: Box<[u8]>,
    taken: u16,
    want: u16,
    reads: u16,
    outputs: Box<[u16]>,
    total: u16,
    given: u16,
    /// Edges made since the command's read.
    k: u16,
    colour: u8,
    /// 05h's count of opaque nibbles per overlay byte, against the colour of its time, kept in the chip's RAM between
    /// commands (VenusRT_Native.md §42).
    overlay: Box<[u8]>,
    /// 0Dh's row buffer in the chip's RAM, 104 bytes, kept between commands.
    row: Box<[u8]>,
    usf1: bool,
    usf0: bool,
    wide: bool,
    /// The word the command leaves in DR at idle.
    idle_next: u16,
    /// The result's notice, set when the inputs are complete.
    compute: u16,
}

impl Default for Dsp2 {
    fn default() -> Dsp2 {
        Dsp2 {
            mode_at: None,
            overflow: false,
            wait: Wait::None,
            base: 0,
            inputs: vec![0; 256].into(),
            taken: 0,
            want: 0,
            reads: 0,
            outputs: vec![0; 256].into(),
            total: 0,
            given: 0,
            k: 0,
            colour: 0,
            overlay: vec![0; 80].into(),
            row: vec![0; 104].into(),
            usf1: false,
            usf0: false,
            wide: false,
            idle_next: 0,
            compute: 0,
        }
    }
}

impl Dsp2 {
    /// fullsnes, "Reset": the program starts again; the colour, in the chip's RAM, is kept.
    pub(super) fn reset(&mut self) {
        *self = Dsp2 { colour: self.colour, overlay: self.overlay.clone(), row: self.row.clone(), ..Dsp2::default() };
    }

    fn input(&self, i: usize) -> u8 {
        self.inputs.get(i).copied().unwrap_or(0)
    }

    fn word32(&self, at: usize) -> u32 {
        u32::from_le_bytes([self.input(at), self.input(at + 1), self.input(at + 2), self.input(at + 3)])
    }

    /// Nibbles of `b` that are not the transparent colour.
    fn opaque(&self, b: u8) -> u16 {
        (b >> 4 != self.colour) as u16 + (b & 15 != self.colour) as u16
    }

    pub(super) fn pack(&self, o: &mut Vec<u8>) {
        let (mode, mode_at) = self.mode_at.map_or((0, 0), |(at, k)| (1 + k, at));
        let (wait, wn, wv, ww, wno) = match self.wait {
            Wait::None => (0, 0, 0, 0, 0),
            Wait::Silent => (1, 0, 0, 0, 0),
            Wait::Edge(n, w, no) => {
                let (k, v) = match n {
                    Next::Request { .. } => (1, 0),
                    Next::Offer(v) => (2, v),
                    _ => (3, 0),
                };
                (2, k, v, w, no)
            }
        };
        o.extend([mode, wait, wn, self.base, self.colour, self.usf1 as u8 | (self.usf0 as u8) << 1 | (self.overflow as u8) << 2, self.wide as u8]);
        o.extend(mode_at.to_le_bytes());
        for w in [wv, ww, wno, self.taken, self.want, self.reads, self.total, self.given, self.k, self.idle_next, self.compute] {
            o.extend(w.to_le_bytes());
        }
        o.extend(self.inputs.iter());
        for w in self.outputs.iter() {
            o.extend(w.to_le_bytes());
        }
        o.extend(self.overlay.iter());
        o.extend(self.row.iter());
    }

    pub(super) fn unpack(&mut self, d: &[u8]) {
        let w = |i: usize| u16::from_le_bytes([d[15 + i * 2], d[16 + i * 2]]);
        let at = u64::from_le_bytes(d[7..15].try_into().expect("eight bytes"));
        self.mode_at = match d[0] {
            0 => None,
            m => Some((at, m - 1)),
        };
        self.wait = match d[1] {
            1 => Wait::Silent,
            2 => Wait::Edge([Next::Request { wide: false }, Next::Offer(w(0)), Next::Idle][d[2] as usize - 1], w(1), w(2)),
            _ => Wait::None,
        };
        (self.base, self.colour, self.wide) = (d[3], d[4], d[6] != 0);
        (self.usf1, self.usf0, self.overflow) = (d[5] & 1 != 0, d[5] & 2 != 0, d[5] & 4 != 0);
        (self.taken, self.want, self.reads, self.total, self.given, self.k, self.idle_next, self.compute) = (w(3), w(4), w(5), w(6), w(7), w(8), w(9), w(10));
        let base = 15 + 2 * 11;
        self.inputs.copy_from_slice(&d[base..base + 256]);
        for (i, o) in self.outputs.iter_mut().enumerate() {
            *o = u16::from_le_bytes([d[base + 256 + i * 2], d[base + 257 + i * 2]]);
        }
        self.overlay.copy_from_slice(&d[base + 768..base + 848]);
        self.row.copy_from_slice(&d[base + 848..base + 952]);
    }
}

/// A row of 8 pixels, two to a byte with the left one high, to its four bitplanes, pixel x at bit 7 - x.
fn planes(row: [u8; 4]) -> [u8; 4] {
    let mut p = [0u8; 4];
    for x in 0..8 {
        let pixel = (row[x / 2] >> if x % 2 == 0 { 4 } else { 0 }) & 15;
        for (plane, out) in p.iter_mut().enumerate() {
            *out |= ((pixel >> plane) & 1) << (7 - x);
        }
    }
    p
}

/// 02h's output phases with a work of 7, by position q in a tile (bit q), measured (VenusRT_Native.md §42.1): the first
/// tile's, a later tile's.
const TILE_FIRST: u32 = 0xFBBB_B330;
const TILE_LATER: u32 = 0xFBBB_3BBA;

impl DspHle {
    /// The DSP-2's transaction as a line, for the lockstep runner's report.
    pub fn d2_describe(&self) -> String {
        let d = &self.d2;
        format!("command {:02X} edges {} taken {}/{} reads {} given {}/{} wait {:?} next {:?} due {} at {} sr {:04X} dr {:04X}", self.command, d.k, d.taken, d.want, d.reads, d.given, d.total, d.wait, self.next, self.due, self.cycles, self.sr, self.dr)
    }

    fn d2_sr(&self) -> u16 {
        RQM | (self.sr & DRS) | if self.d2.wide { 0 } else { DRC } | if self.d2.usf1 { USF1 } else { 0 } | if self.d2.usf0 { USF0 } else { 0 }
    }

    fn d2_note(&mut self, place: u8, write: Option<u16>) {
        self.edges += 1;
        self.edge_place = place;
        self.edge_write = write;
    }

    pub(super) fn d2_event(&mut self, kind: u8) {
        match kind {
            0 | 1 => {
                let wide = kind == 1;
                self.d2.wide = wide;
                self.d2.usf1 = false;
                self.sr = (self.sr & !(DRC | USF1)) | if wide { 0 } else { DRC };
                if self.d2.base == 0x3E && wide {
                    self.d2.mode_at = Some((self.cycles + 5, 0));
                }
            }
            2 | 3 => self.d2.usf1 = kind == 3 && self.d2.overflow,
            4 => {
                self.d2.usf0 = true;
                if self.d2.base == 0x05 {
                    self.d2.mode_at = Some((self.due - 1, 5));
                } else if self.d2.total == 0 {
                    self.d2.mode_at = Some((self.cycles + 6, 6));
                }
            }
            5 => self.d2.usf0 = false,
            7 => self.d2.usf1 = true,
            8 => self.d2.usf1 = false,
            _ => (self.d2.usf0, self.d2.usf1) = (false, true),
        }
        if kind >= 2 {
            self.sr = (self.sr & !(USF1 | USF0)) | if self.d2.usf1 { USF1 } else { 0 } | if self.d2.usf0 { USF0 } else { 0 };
        }
    }

    /// The edge `next` made now.
    pub(super) fn d2_edge(&mut self, next: Next) {
        self.rise = self.cycles;
        match next {
            Next::Request { .. } => {
                self.sr = self.d2_sr();
                let v = self.dr as u8;
                if self.stage == Stage::Idle {
                    self.d2_note(1, None);
                    self.d2_command(v);
                } else {
                    self.d2_note(2, None);
                    self.d2.k += 1;
                    let t = self.d2.taken as usize;
                    if t < 256 {
                        self.d2.inputs[t] = v;
                    }
                    self.d2.taken += 1;
                    self.d2_after_read();
                }
            }
            Next::Offer(v) => {
                self.dr = v;
                self.sr = self.d2_sr();
                self.d2_note(2, Some(v));
                self.d2.k += 1;
                self.d2.given += 1;
                if self.d2.given == 2 && matches!(self.d2.base, 0x07 | 0x08) {
                    self.d2.mode_at = Some((self.cycles + 2, 3));
                }
                if self.d2.given == 1 && matches!(self.d2.base, 0x0B | 0x0C | 0x0D) {
                    self.d2.mode_at = Some((self.cycles + 1, if self.d2.base == 0x0D { 8 } else { 5 }));
                }
                if self.d2.given < self.d2.total {
                    let v = self.d2_output(self.d2.given);
                    self.d2_plan(Next::Offer(v), false);
                } else {
                    self.d2_plan(Next::Idle, false);
                }
            }
            Next::Idle => {
                self.stage = Stage::Idle;
                if self.d2.base == 0x05 && self.d2.total == 0 && self.d2.taken == 1 {
                    self.d2.usf1 = true;
                }
                self.dr = self.idle_word;
                self.sr = self.d2_sr();
                self.d2_note(0, Some(self.dr));
                self.d2.wait = Wait::Edge(Next::Request { wide: false }, 0, 2);
            }
            _ => {}
        }
    }

    fn d2_output(&self, j: u16) -> u16 {
        if self.d2.base == 0x1E { 0 } else { self.d2.outputs.get(j as usize).copied().unwrap_or(0) }
    }

    /// The S-CPU completed a transfer while RQM was up.
    pub(super) fn d2_completed(&mut self) {
        match std::mem::take(&mut self.d2.wait) {
            Wait::None => {}
            Wait::Edge(next, work, notice) => self.d2_at(next, work, notice),
            Wait::Silent => {
                let t = self.d2.taken as usize;
                if t < 256 {
                    self.d2.inputs[t] = self.dr as u8;
                }
                self.d2.taken += 1;
                self.d2_inputs_done(true);
            }
        }
    }

    /// The edge at the later of `work` after the last and `notice` after now; a 16-bit command's idle narrows DR first.
    fn d2_at(&mut self, next: Next, work: u16, notice: u16) {
        self.next = next;
        self.due = (self.rise + work as u64).max(self.cycles + notice as u64);
        if next == Next::Idle && self.d2.wide {
            self.d2.mode_at = Some((self.due - 4, 0));
        }
    }

    /// The next edge, with its phase: waited for, or, at an edge that does not wait, scheduled from this one.
    fn d2_plan(&mut self, next: Next, at_completion: bool) {
        let (work, notice, waits) = self.d2_phase(self.d2.k + 1);
        if at_completion {
            self.d2_at(next, work, notice);
        } else if waits {
            self.d2.wait = Wait::Edge(next, work, notice);
        } else {
            self.schedule(next, work as u64);
        }
    }

    fn d2_command(&mut self, c: u8) {
        let base = match c & 15 {
            0x0E => c & 0x3F,
            0x0A => 0x09,
            b => b,
        };
        self.command = c;
        let d = &mut self.d2;
        (d.base, d.k, d.taken, d.given, d.total, d.compute) = (base, 0, 0, 0, 0, 0);
        self.stage = Stage::Taking { want: 0, taken: 0 };
        let (want, reads) = match base {
            0x00 | 0x09 => (4, 3),
            0x01 => (32, 32),
            0x02 | 0x05 | 0x06 | 0x0D => (1, 1),
            0x03 => (1, 0),
            0x04 => (2, 1),
            0x07 | 0x08 | 0x0B | 0x0C => (8, 7),
            0x0E | 0x1E | 0x2E => (1, 0),
            _ => (0, 0),
        };
        (self.d2.want, self.d2.reads) = (want, reads);
        match base {
            0x0F => {
                self.idle_word = 0x00FF;
                self.d2.mode_at = Some((self.cycles + 10, 2));
                self.schedule(Next::Idle, 14);
            }
            0x3E => {
                self.idle_word = 0x00FF;
                self.d2.mode_at = Some((self.cycles + 10, 1));
                self.schedule(Next::Idle, 19);
            }
            _ => {
                if matches!(base, 0x0E | 0x1E | 0x2E) {
                    self.d2.mode_at = Some((self.cycles + 10, 1));
                }
                if matches!(base, 0x07 | 0x08 | 0x0B | 0x0C) {
                    self.d2.mode_at = Some((self.cycles + 10, 2));
                }
                if reads > 0 {
                    self.d2_plan(Next::Request { wide: false }, false);
                } else {
                    self.d2.wait = Wait::Silent;
                }
            }
        }
    }

    fn d2_after_read(&mut self) {
        let d = &mut self.d2;
        if d.taken == 1 {
            let n = d.inputs[0] as u16;
            match d.base {
                0x02 => {
                    let m = if (1..=3).contains(&n) { n } else { 4 };
                    (d.want, d.reads) = (1 + 32 * m, 32 * m);
                }
                0x05 if n > 0 => {
                    let n = n.min(80);
                    (d.want, d.reads) = (1 + 2 * n, 2 * n);
                }
                0x06 if n > 0 => (d.want, d.reads) = (1 + n, n),
                0x0D => {
                    let bytes = n.div_ceil(2);
                    (d.want, d.reads) = (2 + bytes, (1 + bytes).max(2));
                }
                _ => {}
            }
        }
        if self.d2.taken < self.d2.reads {
            self.d2_plan(Next::Request { wide: false }, false);
        } else if self.d2.taken < self.d2.want {
            self.d2.wait = Wait::Silent;
        } else {
            self.d2_inputs_done(false);
        }
    }

    /// Every input taken: the results computed, and the first result or idle planned.
    fn d2_inputs_done(&mut self, at_completion: bool) {
        self.d2_compute();
        if matches!(self.d2.base, 0x0B | 0x0C | 0x0D) || self.d2.base == 0x05 && self.d2.total > 0 {
            self.d2.mode_at = Some((self.cycles + 3, if self.d2.base == 0x0D { 7 } else { 4 }));
        }
        if self.d2.total > 0 {
            let v = self.d2_output(0);
            self.d2_plan(Next::Offer(v), at_completion);
        } else {
            self.d2_plan(Next::Idle, at_completion);
        }
        self.idle_word = self.d2.idle_next;
    }

    fn d2_compute(&mut self) {
        let d = &mut self.d2;
        let mut out: Vec<u16> = Vec::new();
        let mut idle = 0u16;
        let pair = |o: &[u16], i: usize| (o[i] << 8) | o[i + 1];
        match d.base {
            0x00 => planes([d.input(0), d.input(1), d.input(2), d.input(3)]).iter().for_each(|&b| out.push(b as u16)),
            0x01 | 0x02 => {
                let (first, tiles) = if d.base == 0x01 { (0, 1) } else { (1, (d.want as usize - 1) / 32) };
                for t in 0..tiles {
                    let mut tile = [0u8; 32];
                    for r in 0..8 {
                        let at = first + 32 * t + 4 * r;
                        let p = planes([d.input(at), d.input(at + 1), d.input(at + 2), d.input(at + 3)]);
                        (tile[2 * r], tile[2 * r + 1], tile[16 + 2 * r], tile[17 + 2 * r]) = (p[0], p[1], p[2], p[3]);
                    }
                    tile.iter().for_each(|&b| out.push(b as u16));
                }
                idle = if d.base == 0x01 { 3 } else { 0 };
            }
            0x03 => d.colour = d.input(0) & 15,
            0x04 | 0x05 => {
                let (n, a, b) = if d.base == 0x04 { (1, 0, 1) } else { let n = (d.want as usize - 1) / 2; (n, 1, 1 + n) };
                d.compute = if d.base == 0x04 { 15 + d.opaque(d.input(1)) } else { 10 * n as u16 + 9 };
                for i in 0..n {
                    let (under, over) = (d.input(a + i), d.input(b + i));
                    let hi = if over >> 4 == d.colour { under & 0xF0 } else { over & 0xF0 };
                    let lo = if over & 15 == d.colour { under & 15 } else { over & 15 };
                    out.push((hi | lo) as u16);
                    if d.base == 0x05 {
                        d.compute += d.opaque(over);
                    }
                }
                if d.base == 0x05 {
                    for i in 0..n {
                        d.overlay[i] = d.opaque(d.input(b + i)) as u8;
                    }
                    if n % 16 == 1 {
                        let start = n - 1;
                        d.compute += 160 + (start..start + 16).map(|i| d.overlay[i.min(79)] as u16).sum::<u16>();
                    }
                }
                idle = if n > 0 { out[n - 1] } else { 0 };
            }
            0x06 => {
                let n = d.want as usize - 1;
                for i in 0..n {
                    out.push(d.input(n - i).rotate_left(4) as u16);
                }
                idle = 0xFFFF;
            }
            0x07 | 0x08 => {
                let (a, b) = (d.word32(0), d.word32(4));
                let (v, o) = if d.base == 0x07 { (a as i32).overflowing_add(b as i32) } else { (a as i32).overflowing_sub(b as i32) };
                let v = v as u32;
                d.overflow = o;
                v.to_le_bytes().iter().for_each(|&x| out.push(x as u16));
                idle = pair(&out, 0);
            }
            0x09 => {
                let k = i16::from_le_bytes([d.input(0), d.input(1)]) as i32;
                let l = i16::from_le_bytes([d.input(2), d.input(3)]) as i32;
                let m = (k * l).wrapping_mul(2) as u32;
                let v = ((m >> 16) >> 1) << 16 | ((m as i16) >> 1) as u16 as u32;
                v.to_le_bytes().iter().for_each(|&x| out.push(x as u16));
                idle = pair(&out, 2);
            }
            0x0B | 0x0C => {
                let (a, b) = (d.word32(0), d.word32(4));
                if b != 0 {
                    let (q, r, extra) = if d.base == 0x0B {
                        (a / b, a % b, 4)
                    } else {
                        let (na, nb) = ((a as i32) < 0, (b as i32) < 0);
                        let (ma, mb) = ((a as i32).unsigned_abs() & 0x7FFF_FFFF, (b as i32).unsigned_abs());
                        let (q, r) = (ma / mb, ma % mb);
                        let q = if na != nb { q.wrapping_neg() } else { q };
                        let r = if na { r.wrapping_neg() } else { r };
                        // Each 32-bit negation takes a cycle more when the value's low word is zero.
                        let lo0 = |v: u32| v & 0xFFFF == 0;
                        let s = 18 * na as u16 + 10 * nb as u16 + 9 * (na != nb) as u16 + (na && lo0(a)) as u16 + (nb && lo0(b)) as u16 + (na != nb && lo0(q)) as u16 + (na && lo0(r)) as u16;
                        (q, r, s)
                    };
                    let mq = if d.base == 0x0B { q } else { (q as i32).unsigned_abs() };
                    d.compute = if d.base == 0x0B { 739 + 2 * mq.count_ones() as u16 + 4 * (mq & 1) as u16 } else { 735 + 2 * mq.count_ones() as u16 + 4 * (mq & 1) as u16 + extra };
                    q.to_le_bytes().iter().chain(r.to_le_bytes().iter()).for_each(|&x| out.push(x as u16));
                    idle = pair(&out, 4);
                }
            }
            0x0D => {
                let (a, b) = (d.input(0) as usize, d.input(1) as usize);
                for i in 0..a.div_ceil(2).min(104) {
                    d.row[i] = d.input(2 + i);
                }
                let row = &d.row;
                let pixel = |x: usize| (row.get(x / 2).copied().unwrap_or(0) >> if x % 2 == 0 { 4 } else { 0 }) & 15;
                let step = (a << 10) / (b + 1);
                let pick = |j: usize| if b < a { pixel((j * step) >> 10) } else { pixel(j) };
                for j in (0..b).step_by(2) {
                    let lo = if j + 1 < b { pick(j + 1) } else { 0 };
                    out.push(((pick(j) << 4) | lo) as u16);
                }
                d.compute = scale_estimate(a as u32, b as u32);
            }
            0x0E => out.push(0),
            0x2E => out.push(0x0100),
            _ => {}
        }
        idle = match d.base {
            0x0E => 0xFFFF,
            0x1E => 0x0400,
            0x2E => 0x00FF,
            _ => idle,
        };
        let total = if d.base == 0x1E { 1024 } else { out.len() as u16 };
        out.resize(256, 0);
        d.outputs.copy_from_slice(&out[..256]);
        (d.total, d.idle_next) = (total, idle);
    }

    /// Edge `k`'s work, notice and wait, counted from the command's read (VenusRT_Native.md §42.1).
    fn d2_phase(&self, k: u16) -> (u16, u16, bool) {
        let d = &self.d2;
        let (r, t) = (d.reads, d.total);
        let n = d.input(0) as u16;
        if k > r + t || (k == r + 1 && t == 0 && d.taken == d.want) {
            return match d.base {
                0x03 => (20, 9, true),
                0x05 if d.want == 1 => (8, 0, false),
                0x0B | 0x0C if t == 0 => (0, 13, true),
                0x0E | 0x1E | 0x2E => (0, 8, true),
                _ => (0, 5, true),
            };
        }
        if k <= r {
            return match d.base {
                0x00 => if k == 1 { (14, 2, true) } else { (7, 2, true) },
                0x01 => if k == 1 { (15, 2, true) } else if k % 4 == 3 { (0, 2, true) } else { (7, 2, true) },
                0x02 => if k == 1 { (11, 2, true) } else { (7, 2, true) },
                0x04 | 0x05 => {
                    if k == 1 {
                        (11, 2, true)
                    } else {
                        let j = k as i32 - 2 - n.min(80) as i32;
                        if j >= 1 && j % 16 != 0 || j == 0 && n > 80 { (7, 2, true) } else { (0, 2, true) }
                    }
                }
                0x06 => if k == 1 { (13, 2, true) } else { (7, 2, true) },
                0x07 | 0x08 | 0x09 => if k == 1 { (12, 2, true) } else { (0, 2, true) },
                0x0B | 0x0C => if k == 1 { (13, 2, true) } else { (0, 2, true) },
                0x0D => if k == 1 { (13, 2, true) } else { (7, 2, true) },
                _ => (0, 2, true),
            };
        }
        let j = k - r - 1;
        if j == 0 {
            return match d.base {
                0x00 => (12, 7, true),
                0x01 => (15, 0, false),
                0x02 => (20, 15, true),
                0x05 if n > 80 => (d.compute + 6, d.compute + 1, true),
                0x04 | 0x05 | 0x0B | 0x0C => (0, d.compute, true),
                0x06 => (15, 10, true),
                0x07 => (0, 6, true),
                0x08 => (0, 7, true),
                0x09 => (0, 9, true),
                0x0D => (d.compute, d.compute - 5, true),
                0x0E => (4123, 3337, true),
                0x1E => (20, 2, true),
                0x2E => (16, 2, true),
                _ => (0, 2, true),
            };
        }
        let seven = match d.base {
            0x00 => j == 1,
            0x02 => {
                let (tile, q) = (j / 32, j % 32);
                let last = tile + 1 == t / 32;
                if q == 0 {
                    true
                } else if last && q >= 30 {
                    false
                } else {
                    (if tile == 0 { TILE_FIRST } else { TILE_LATER }) >> q & 1 != 0
                }
            }
            0x05 => j != 1 && j % 16 != 0,
            0x06 => j + 1 < t,
            _ => false,
        };
        if d.base == 0x01 && j == 1 {
            return (5, 2, true);
        }
        if seven { (7, 2, true) } else { (0, 2, true) }
    }
}

/// 0Dh's compute in cycles: no rule was found, so a least-squares estimate over measured cases, within 11% scaling down
/// and 6% up (VenusRT_Native.md §42).
fn scale_estimate(a: u32, b: u32) -> u16 {
    let w = if b < a { 311 + 62 * b } else { 377 + 61 * b + 45 * (0..b).map(|j| j / a.max(1)).sum::<u32>() / 2 };
    w.min(0xFFFF) as u16
}

#[cfg(test)]
mod tests {
    use super::super::dsphle::{DspHle, Program};
    use super::super::dsporacle::{Hle, Host, power_on, transact};

    fn ask(h: &mut Hle, command: u8, inputs: &[u16]) -> Vec<u16> {
        transact(h, &mut Host::steady(), command, inputs).outputs()
    }

    // The formulas with no image: fullsnes's 4-bit tile layout, the 32-bit sum and difference, the reversed row, and
    // the transparent colour's overlay.
    #[test]
    fn the_dsp2_formulas_answer_through_the_ports() {
        let mut h = Hle::new(Program::Dsp2);
        power_on(&mut h, &Host::steady()).unwrap();
        // Every pixel of the tile colour 5 (0101): planes 0 and 2 all ones, planes 1 and 3 all zeros.
        let tile = ask(&mut h, 0x01, &[0x55; 32]);
        assert!((0..8).all(|r| tile[2 * r] == 0xFF && tile[2 * r + 1] == 0 && tile[16 + 2 * r] == 0xFF && tile[17 + 2 * r] == 0));
        assert_eq!(ask(&mut h, 0x07, &[0xFF, 0xFF, 0, 0, 1, 0, 0, 0]), [0, 0, 1, 0]);
        assert_eq!(ask(&mut h, 0x08, &[0, 0, 0, 0, 1, 0, 0, 0]), [0xFF; 4]);
        assert_eq!(ask(&mut h, 0x06, &[3, 0x12, 0x34, 0x56]), [0x65, 0x43, 0x21]);
        ask(&mut h, 0x03, &[0x0A]);
        assert_eq!(ask(&mut h, 0x05, &[2, 0x12, 0x34, 0xAA, 0xA5]), [0x12, 0x35]);
    }

    #[test]
    fn the_dsp2_state_round_trips_mid_command() {
        let mut h = Hle::new(Program::Dsp2);
        power_on(&mut h, &Host::steady()).unwrap();
        ask(&mut h, 0x03, &[0x07]);
        ask(&mut h, 0x05, &[3, 1, 2, 3, 4, 5, 6]);
        let mut back = DspHle::new(Program::Dsp2);
        back.unpack(&h.dsp.pack());
        assert_eq!(back.pack(), h.dsp.pack());
    }
}
