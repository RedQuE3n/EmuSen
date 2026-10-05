//! VenusRT's open replacements for the NEC DSP programs (VenusRT_DspHle.md), at step 2 of its §8: the frame. The
//! ports behave as the low-level path's do (VenusRT_Native.md §37.2); of the commands, only those fullsnes fixes are
//! answered exactly, and the rest take their documented transfers and give zeros. See VenusRT_Native.md §38.

use super::necdsp::{DSPN_RATIO, Port, ST_RATIO};

mod sr {
    pub const RQM: u16 = 0x8000;
    pub const DRS: u16 = 0x1000;
    pub const DRC: u16 = 0x0400;
}

/// The programs a replacement exists for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Program {
    Dsp1,
    Dsp1b,
    Dsp2,
    Dsp4,
    St010,
}

impl Program {
    /// By the file stem `Cartridge::nec_firmware` gives.
    pub fn for_stem(stem: &str) -> Option<Program> {
        match stem {
            "dsp1" => Some(Program::Dsp1),
            "dsp1b" => Some(Program::Dsp1b),
            "dsp2" => Some(Program::Dsp2),
            "dsp4" => Some(Program::Dsp4),
            "st010" => Some(Program::St010),
            _ => None,
        }
    }

    fn tag(self) -> u8 {
        match self {
            Program::Dsp1 => 1,
            Program::Dsp1b => 2,
            Program::Dsp2 => 3,
            Program::St010 => 4,
            Program::Dsp4 => 5,
        }
    }
}

/// The chip side's next edge, due at `DspHle::due`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Next {
    None,
    /// RQM up for the next write, in 16-bit mode (`wide`) or 8-bit.
    Request { wide: bool },
    /// A result written to DR.
    Offer(u16),
    /// Back to idle with DR holding the idle word.
    Idle,
    /// ST010: the mailbox's busy bit cleared.
    Done,
    /// DR to 16-bit mode, 3 cycles after the command's request as on the chip (VenusRT_Native.md §37.2).
    Widen,
}

/// Where the transaction stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Stage {
    /// Waiting for a command byte; `fresh` until the first.
    Idle,
    /// Taking inputs: `taken` of `want`, `dummy` for a command without inputs whose request still stands.
    Taking { want: u8, taken: u8 },
    /// Giving results: `given` of `total`; a raster run repeats its line until the S-CPU writes over a result.
    Giving { total: u16, given: u16, run: bool, overwritten: bool },
    /// ST010: its power-on word not yet read, then serving the mailbox.
    StStart,
    StMailbox,
}

/// The number of words a replacement keeps between transfers.
const WORDS: usize = 8;

#[derive(Clone, Debug)]
pub struct DspHle {
    pub program: Program,
    pub sr: u16,
    pub dr: u16,
    /// The ST010's RAM, 2K words as the low-level path keeps it; empty for a DSP-n.
    pub ram: Box<[u16]>,
    /// The chip's clock, as the low-level path counts it.
    pub cycles: u64,
    pub ratio: (u64, u64),
    pub(super) command: u8,
    pub(super) stage: Stage,
    pub(super) next: Next,
    pub(super) due: u64,
    inputs: [u16; WORDS],
    outputs: [u16; WORDS],
    /// The DSP-2's idle word: 00h after power-on, FFh after a command (VenusRT_Native.md §37.2).
    pub(super) idle_word: u16,
    /// The transaction's phase, and the cycle RQM last rose, for the work and notice of VenusRT_Native.md §37.3.
    phase: u8,
    pub(super) rise: u64,
    /// The DSP-2's command state (VenusRT_Native.md §42); default for the other programs.
    pub(super) d2: super::dsp2::Dsp2,
    /// The ST010's mailbox poll (VenusRT_Native.md §43).
    pub(super) mail: super::st010::Mail,
    /// The DSP-4's command state (VenusRT_Native.md §48.3).
    pub(super) d4: super::dsp4::Dsp4,
    /// The DSP-1's three attitude matrices, m/2 times the rotation in Q15 (VenusRT_Native.md §44).
    pub(super) att: [[i16; 9]; 3],
    /// The DSP-1's projection, Parameter's seven inputs, which Raster and Project read (VenusRT_Native.md §49).
    pub(super) proj: [u16; 7],
    /// A phase's work and notice that depend on the inputs, by phase, set when the inputs are complete.
    pub(super) varying: Option<(u8, u16, u16)>,
    /// Edges made, and the last one's place (0 idle, 1 command, 2 other) and written word, for the oracle; not in the state.
    pub edges: u64,
    pub edge_place: u8,
    pub edge_write: Option<u16>,
}

/// Cycles from the S-CPU's completion to the chip's next edge, the DSP-1B's common notice (VenusRT_Native.md §37.3).
const NOTICE: u64 = 2;

impl DspHle {
    pub fn new(program: Program) -> DspHle {
        let st = program == Program::St010;
        let mut d = DspHle {
            program,
            sr: 0,
            dr: 0,
            ram: vec![0; if st { 2048 } else { 0 }].into(),
            cycles: 0,
            ratio: if st { ST_RATIO } else { DSPN_RATIO },
            command: 0,
            stage: Stage::Idle,
            next: Next::None,
            due: 0,
            inputs: [0; WORDS],
            outputs: [0; WORDS],
            idle_word: 0,
            phase: 0,
            rise: 0,
            d2: Default::default(),
            mail: Default::default(),
            d4: Default::default(),
            att: [[0; 9]; 3],
            proj: [0; 7],
            varying: None,
            edges: 0,
            edge_place: 2,
            edge_write: None,
        };
        d.reset();
        d
    }

    pub fn st(&self) -> bool {
        self.program == Program::St010
    }

    /// fullsnes, "Reset": the chip starts again; the ST010's RAM is kept.
    pub fn reset(&mut self) {
        self.sr = 0;
        self.dr = 0;
        self.inputs = [0; WORDS];
        self.outputs = [0; WORDS];
        self.idle_word = if self.program == Program::Dsp2 { 0 } else { 0x80 };
        if self.program == Program::Dsp2 {
            self.d2.reset();
            self.stage = Stage::Idle;
            self.schedule(Next::Idle, NOTICE);
        } else if self.program == Program::Dsp4 {
            self.d4 = Default::default();
            self.stage = Stage::Idle;
            self.schedule(Next::Idle, NOTICE);
        } else if self.st() {
            self.stage = Stage::StStart;
            self.schedule(Next::Offer(0), 1);
        } else {
            self.stage = Stage::Idle;
            self.schedule(Next::Idle, NOTICE);
        }
    }

    /// At idle with DR holding the idle word, for the oracle's places.
    pub fn idle(&self) -> bool {
        self.stage == Stage::Idle && self.next == Next::None && self.sr & sr::DRC != 0 && self.dr == self.idle_word
    }

    /// Asking for its first input or for a byte after one passed over, for the oracle's places.
    pub fn reading_command(&self) -> bool {
        matches!(self.stage, Stage::Taking { taken: 0, .. }) || self.stage == Stage::Idle && self.dr != self.idle_word
    }

    /// The last edge was a result written.
    pub fn offering(&self) -> bool {
        matches!(self.stage, Stage::Giving { .. }) || self.stage == Stage::StStart
    }

    pub(super) fn schedule(&mut self, next: Next, after: u64) {
        self.next = next;
        self.due = self.cycles + after;
    }

    /// The transaction's next edge, at the later of its phase's work after the last rise and its notice after now.
    fn next_phase(&mut self, next: Next) {
        let (work, notice) = match self.varying {
            Some((phase, w, n)) if phase == self.phase => (w, n),
            // A raster run: the first line, each later line, and the run's end (VenusRT_Native.md §49.5).
            _ if self.command & 0xCF == 0x0A => match (next, self.stage, Projection::new(&self.proj).past) {
                (Next::Offer(_), Stage::Giving { given: 0, .. }, false) if self.phase == 1 => (128, 116),
                (Next::Offer(_), Stage::Giving { given: 0, .. }, false) => (0, 115),
                (Next::Offer(_), Stage::Giving { given: 0, .. }, true) if self.phase == 1 => (130, 118),
                (Next::Offer(_), Stage::Giving { given: 0, .. }, true) => (0, 117),
                (Next::Idle, _, _) => (0, 6),
                _ => (0, NOTICE as u16),
            },
            _ => dsp1_timing(self.command).get(self.phase as usize).copied().unwrap_or((0, NOTICE as u16)),
        };
        self.next = next;
        self.due = (self.rise + work as u64).max(self.cycles + notice as u64);
        self.phase = self.phase.saturating_add(1);
    }

    /// Catches the chip up to the master clock `clock`, making its due edge on the way.
    pub fn run_to(&mut self, clock: u64) {
        let target = (clock as u128 * self.ratio.0 as u128 / self.ratio.1 as u128) as u64;
        if target <= self.cycles {
            return;
        }
        loop {
            let mode = self.d2.mode_at.filter(|&(at, _)| at <= target);
            let edge = (self.next != Next::None && self.due <= target).then_some(self.due);
            match (mode, edge) {
                (Some((at, kind)), e) if e.is_none_or(|e| at <= e) => {
                    self.cycles = self.cycles.max(at);
                    self.d2.mode_at = None;
                    self.d2_event(kind);
                }
                (_, Some(due)) => {
                    self.cycles = self.cycles.max(due);
                    self.edge();
                }
                _ => break,
            }
        }
        self.cycles = target;
    }

    fn edge(&mut self) {
        if self.program == Program::Dsp2 {
            let next = std::mem::replace(&mut self.next, Next::None);
            self.d2_edge(next);
            return;
        }
        if self.program == Program::Dsp4 {
            let next = std::mem::replace(&mut self.next, Next::None);
            self.d4_edge(next);
            return;
        }
        let next = std::mem::replace(&mut self.next, Next::None);
        let counted = matches!(next, Next::Request { .. } | Next::Offer(_) | Next::Idle);
        self.edge_of(next);
        if counted {
            self.edges += 1;
            self.edge_place = if self.idle() { 0 } else if self.reading_command() { 1 } else { 2 };
            self.edge_write = (self.offering() || self.edge_place == 0).then_some(self.dr);
        }
    }

    fn edge_of(&mut self, next: Next) {
        match next {
            Next::None => {}
            Next::Request { wide } => {
                self.rise = self.cycles;
                let narrow = self.sr & sr::DRC != 0;
                self.sr = (self.sr & !sr::DRC) | if wide && !narrow { 0 } else { sr::DRC } | sr::RQM;
                if wide && narrow {
                    self.schedule(Next::Widen, 3);
                }
            }
            Next::Widen => self.sr &= !sr::DRC,
            Next::Offer(v) => {
                self.rise = self.cycles;
                self.dr = v;
                self.sr = (self.sr & !sr::DRC) | if self.program == Program::Dsp2 { sr::DRC } else { 0 } | sr::RQM;
            }
            Next::Idle => {
                self.rise = self.cycles;
                self.stage = Stage::Idle;
                self.dr = self.idle_word;
                self.sr = (self.sr & !sr::DRS) | sr::DRC | sr::RQM;
            }
            Next::Done => self.st_done(),
        }
    }

    /// The S-CPU's read; `side_effects` false is a look that moves no handshake.
    pub fn host_read(&mut self, port: Port, side_effects: bool) -> u8 {
        match port {
            Port::Sr => (self.sr >> 8) as u8,
            Port::Ram(i) => match self.ram.get((i >> 1) & self.ram.len().saturating_sub(1)) {
                Some(&w) => if i & 1 == 0 { w as u8 } else { (w >> 8) as u8 },
                None => 0,
            },
            Port::Dr => {
                let high = self.sr & (sr::DRC | sr::DRS) == sr::DRS;
                let v = if high { (self.dr >> 8) as u8 } else { self.dr as u8 };
                if side_effects && self.byte_done() {
                    self.completed();
                }
                v
            }
        }
    }

    pub fn host_write(&mut self, port: Port, value: u8) {
        match port {
            Port::Sr => {}
            Port::Ram(i) => {
                if self.ram.is_empty() {
                    return;
                }
                let w = &mut self.ram[(i >> 1) & (self.ram.len() - 1)];
                *w = if i & 1 == 0 { (*w & 0xFF00) | value as u16 } else { (*w & 0x00FF) | (value as u16) << 8 };
                if i & 0xFFF == 0x21 && value & 0x80 != 0 && self.stage == Stage::StMailbox && self.next != Next::Done {
                    self.st_command();
                }
            }
            Port::Dr => {
                let high = self.sr & (sr::DRC | sr::DRS) == sr::DRS;
                self.dr = if high { (self.dr & 0x00FF) | (value as u16) << 8 } else { (self.dr & 0xFF00) | value as u16 };
                if self.byte_done() {
                    self.completed();
                }
            }
        }
    }

    /// One byte through DR as the handshake counts it, true when it completed the transfer.
    fn byte_done(&mut self) -> bool {
        let rqm = self.sr & sr::RQM != 0 || !matches!(self.program, Program::Dsp2 | Program::Dsp4);
        if self.sr & sr::DRC != 0 {
            self.sr &= !sr::RQM;
            rqm
        } else if self.sr & sr::DRS == 0 {
            self.sr |= sr::DRS;
            false
        } else {
            self.sr &= !(sr::DRS | sr::RQM);
            rqm
        }
    }

    /// The S-CPU completed a transfer in either direction; fullsnes's chip is "oblivious" to which.
    fn completed(&mut self) {
        if self.program == Program::Dsp2 {
            self.d2_completed();
            return;
        }
        if self.program == Program::Dsp4 {
            self.d4_completed();
            return;
        }
        match self.stage {
            Stage::StStart => {
                self.stage = Stage::StMailbox;
                self.st_begin();
            }
            Stage::StMailbox => {}
            Stage::Idle => self.command(self.dr as u8),
            Stage::Taking { want, taken } => {
                if (taken as usize) < WORDS {
                    self.inputs[taken as usize] = self.dr;
                }
                let taken = taken + 1;
                if taken < want {
                    self.stage = Stage::Taking { want, taken };
                    self.next_phase(Next::Request { wide: true });
                } else {
                    self.compute();
                }
            }
            Stage::Giving { total, given, run, overwritten } => {
                // A raster run ends when DR no longer holds a line's last result: the S-CPU wrote another word over it
                // (VenusRT_Native.md §49.5).
                let overwritten = overwritten || run && given + 1 == total && self.dr != self.outputs[given as usize];
                let given = given + 1;
                if given < total {
                    self.stage = Stage::Giving { total, given, run, overwritten };
                    let v = self.outputs.get(given as usize).copied().unwrap_or(0);
                    self.next_phase(Next::Offer(v));
                } else if run && !overwritten {
                    self.stage = Stage::Giving { total, given: 0, run, overwritten: false };
                    self.inputs[0] = self.inputs[0].wrapping_add(1);
                    let line = Projection::new(&self.proj).raster(self.inputs[0] as i16 as i64);
                    self.outputs[..4].copy_from_slice(&line.map(|v| v as u16));
                    self.next_phase(Next::Offer(self.outputs[0]));
                } else {
                    self.next_phase(Next::Idle);
                }
            }
        }
    }

    fn command(&mut self, command: u8) {
        self.command = command;
        let Some((inputs, _, _)) = dsp1_shape(command) else {
            // 40h-FFh are passed over: the next byte is taken as a command.
            self.schedule(Next::Request { wide: false }, NOTICE);
            return;
        };
        self.stage = Stage::Taking { want: inputs.max(1), taken: 0 };
        self.phase = 0;
        self.next_phase(Next::Request { wide: true });
    }

    /// The command's results, by the formulas and the members VenusRT_Native.md §39 chose; zeros where none is built.
    fn compute(&mut self) {
        let (_, total, run) = dsp1_shape(self.command).unwrap_or((0, 0, false));
        let i = self.inputs.map(|w| w as i16 as i64);
        self.varying = None;
        let squares = i[0] * i[0] + i[1] * i[1] + i[2] * i[2];
        self.outputs = [0; WORDS];
        match self.command {
            // SNESdev's K·I scaled by 2^-15, the multiplier's own floor; 20h with its low bit set.
            0x00 => self.outputs[0] = ((i[0] * i[1]) >> 15) as u16,
            0x20 => self.outputs[0] = ((i[0] * i[1]) >> 15) as u16 | 1,
            // x² + y² + z² in units of 2^-1 (SNESdev's L2 and H2), wrapping in 32 bits.
            0x08 => {
                let v = (squares << 1) as u32;
                self.outputs[0] = v as u16;
                self.outputs[1] = (v >> 16) as u16;
            }
            // x² + y² + z² - r² in units of 2^-1, its high word; 38h one more.
            0x18 | 0x38 => {
                let v = ((squares - i[3] * i[3]) << 1) as i32;
                self.outputs[0] = ((v >> 16) + (self.command == 0x38) as i32) as u16;
            }
            0x2F => self.outputs[0] = if self.program == Program::Dsp1 { 0x0100 } else { 0x0101 },
            // Approximate (VenusRT_Native.md §40.3): §40.2's closest member, not the chip's sine.
            0x04 => {
                let (s, c) = (sine(self.inputs[0]), sine(self.inputs[0].wrapping_add(0x4000)));
                self.outputs[0] = ((i[1] * s) >> 15) as u16;
                self.outputs[1] = ((i[1] * c) >> 15) as u16;
            }
            // SNESdev's matrix with the row vector on the left (§40.3), the exact sum scaled once, wrapping.
            0x0C => {
                let (s, c) = (sine(self.inputs[0]), sine(self.inputs[0].wrapping_add(0x4000)));
                self.outputs[0] = ((i[1] * c + i[2] * s) >> 15) as u16;
                self.outputs[1] = ((i[2] * c - i[1] * s) >> 15) as u16;
            }
            // The row vector times SNESdev's matrices about Z by I1, Y by I2 and X by I3, in that order (§40.4).
            0x1C => {
                let sc = |k: usize| (sine(self.inputs[k]), sine(self.inputs[k].wrapping_add(0x4000)));
                let ((s1, c1), (s2, c2), (s3, c3)) = (sc(0), sc(1), sc(2));
                let w = |v: i64| (v >> 15) as i16 as i64;
                let (x, y, z) = (i[3], i[4], i[5]);
                let (x, y) = (w(x * c1 + y * s1), w(y * c1 - x * s1));
                let (x, z) = (w(x * c2 - z * s2), w(x * s2 + z * c2));
                let (y, z) = (w(y * c3 + z * s3), w(z * c3 - y * s3));
                self.outputs[..3].copy_from_slice(&[x as u16, y as u16, z as u16]);
            }
            // Approximate (§44): the exponent and timing exact, the mantissa within two of the chip's.
            0x10 | 0x30 => {
                let (mantissa, exponent, shifts, edge) = inverse(self.inputs[0] as i16, self.inputs[1] as i16);
                self.outputs[..2].copy_from_slice(&[mantissa as u16, exponent as u16]);
                let notice = match self.inputs[0] as i16 {
                    0 => 14,
                    a if a > 0 => 36 + 2 * shifts + 2 * edge as u16,
                    _ => 40 + 2 * shifts + edge as u16,
                };
                self.varying = Some((2, 0, notice));
            }
            // Approximate (§44): the root floored, within four of the DSP-1B's; the DSP-1's bug is not modelled.
            0x28 => {
                let v = (squares as u64) & 0xFFFF_FFFF;
                self.outputs[0] = (v.isqrt() as u16).min(0x7FFF);
                let k = 64 - v.leading_zeros() as u16;
                let notice = if k >= 16 { 122 - 4 * (k / 2) } else if k >= 4 { 83 - 4 * (k / 2) } else { 79 };
                self.varying = Some((3, 0, notice - (self.program == Program::Dsp1) as u16));
            }
            // Parameter, Raster and Project: approximate (VenusRT_Native.md §49), §49.5's member; no limit branch (§49.3).
            0x02 => {
                self.proj.copy_from_slice(&self.inputs[..7]);
                let p = Projection::new(&self.proj);
                self.outputs[..4].copy_from_slice(&p.parameter().map(|v| v as u16));
                // Past the limit the commands take longer: the medians of VenusRT_Native.md §52.5.
                if p.past {
                    self.varying = Some((7, 541, 499));
                }
            }
            0x0A | 0x1A | 0x2A | 0x3A => {
                let line = Projection::new(&self.proj).raster(i[0]);
                self.outputs[..4].copy_from_slice(&line.map(|v| v as u16));
            }
            0x06 => {
                let p = Projection::new(&self.proj);
                self.outputs[..3].copy_from_slice(&p.project([i[0], i[1], i[2]]).map(|v| v as u16));
                if p.past {
                    self.varying = Some((3, 0, 354));
                }
            }
            0x0E | 0x1E | 0x2E | 0x3E => {
                let p = Projection::new(&self.proj);
                self.outputs[..2].copy_from_slice(&p.target(i[0], i[1]).map(|v| v as u16));
                if p.past {
                    self.varying = Some((2, 0, 118));
                }
            }
            // Gyrate: approximate (VenusRT_Native.md §50), §50.3's member.
            0x14 => self.outputs[..3].copy_from_slice(&gyrate(&i).map(|v| v as u16)),
            // Attitude: matrix n set to m/2 times the rotations about X by I4, Y by I3 and Z by I2, in that product (§44).
            0x01 | 0x11 | 0x21 => {
                let n = (self.command >> 4) as usize;
                self.att[n] = attitude(self.inputs[0] as i16, self.inputs[1], self.inputs[2], self.inputs[3]);
            }
            // Objective, Subjective and Scalar: the matrix, its transpose and its first row, times the vector.
            c @ (0x0D | 0x1D | 0x2D | 0x03 | 0x13 | 0x23 | 0x0B | 0x1B | 0x2B) => {
                let m = self.att[(c >> 4) as usize];
                let v = [i[0], i[1], i[2]];
                let row = |r: usize, t: bool| (0..3).map(|j| (2 * m[if t { j * 3 + r } else { r * 3 + j }] as i64 * v[j]) >> 16).sum::<i64>() as u16;
                match c & 0x0F {
                    0x0D => self.outputs[..3].copy_from_slice(&[row(0, false), row(1, false), row(2, false)]),
                    0x03 => self.outputs[..3].copy_from_slice(&[row(0, true), row(1, true), row(2, true)]),
                    _ => self.outputs[0] = row(0, false),
                }
            }
            _ => {}
        }
        if total == 0 {
            self.next_phase(Next::Idle);
        } else {
            self.stage = Stage::Giving { total, given: 0, run, overwritten: false };
            self.next_phase(Next::Offer(self.outputs[0]));
        }
    }

    /// The registers and RAM for the machine's state, in a fixed size per program.
    pub fn pack(&self) -> Vec<u8> {
        let mut o = vec![self.program.tag(), self.command];
        let (stage, a, b, c, d) = match self.stage {
            Stage::Idle => (0, 0, 0, 0, 0),
            Stage::Taking { want, taken } => (1, want as u16, taken as u16, 0, 0),
            Stage::Giving { total, given, run, overwritten } => (2, total, given, run as u16, overwritten as u16),
            Stage::StStart => (3, 0, 0, 0, 0),
            Stage::StMailbox => (4, 0, 0, 0, 0),
        };
        let (next, v) = match self.next {
            Next::None => (0, 0),
            Next::Request { wide } => (1, wide as u16),
            Next::Offer(v) => (2, v),
            Next::Idle => (3, 0),
            Next::Done => (4, 0),
            Next::Widen => (5, 0),
        };
        o.extend([stage, next]);
        for w in [a, b, c, d, v, self.sr, self.dr, self.idle_word].into_iter().chain(self.inputs).chain(self.outputs) {
            o.extend(w.to_le_bytes());
        }
        o.extend(self.cycles.to_le_bytes());
        o.extend(self.due.to_le_bytes());
        o.extend(self.rise.to_le_bytes());
        o.push(self.phase);
        for w in self.ram.iter() {
            o.extend(w.to_le_bytes());
        }
        if self.program == Program::Dsp2 {
            self.d2.pack(&mut o);
        }
        if self.program == Program::Dsp4 {
            self.d4.pack(&mut o);
        }
        if self.st() {
            o.extend(self.mail.poll.to_le_bytes());
        }
        if matches!(self.program, Program::Dsp1 | Program::Dsp1b) {
            for w in self.att.iter().flatten() {
                o.extend(w.to_le_bytes());
            }
            let (p, w, n) = self.varying.unwrap_or((0xFF, 0, 0));
            o.push(p);
            o.extend(w.to_le_bytes());
            o.extend(n.to_le_bytes());
            for w in self.proj {
                o.extend(w.to_le_bytes());
            }
        }
        o
    }

    pub fn unpack(&mut self, d: &[u8]) {
        let w = |i: usize| u16::from_le_bytes([d[4 + i * 2], d[5 + i * 2]]);
        self.command = d[1];
        let (a, b, c, e, v) = (w(0), w(1), w(2), w(3), w(4));
        self.stage = match d[2] {
            1 => Stage::Taking { want: a as u8, taken: b as u8 },
            2 => Stage::Giving { total: a, given: b, run: c != 0, overwritten: e != 0 },
            3 => Stage::StStart,
            4 => Stage::StMailbox,
            _ => Stage::Idle,
        };
        self.next = match d[3] {
            1 => Next::Request { wide: v != 0 },
            2 => Next::Offer(v),
            3 => Next::Idle,
            4 => Next::Done,
            5 => Next::Widen,
            _ => Next::None,
        };
        (self.sr, self.dr, self.idle_word) = (w(5), w(6), w(7));
        for i in 0..WORDS {
            self.inputs[i] = w(8 + i);
            self.outputs[i] = w(8 + WORDS + i);
        }
        let at = 4 + 2 * (8 + 2 * WORDS);
        self.cycles = u64::from_le_bytes(d[at..at + 8].try_into().expect("eight bytes"));
        self.due = u64::from_le_bytes(d[at + 8..at + 16].try_into().expect("eight bytes"));
        self.rise = u64::from_le_bytes(d[at + 16..at + 24].try_into().expect("eight bytes"));
        self.phase = d[at + 24];
        let base = at + 25;
        for (i, r) in self.ram.iter_mut().enumerate() {
            *r = u16::from_le_bytes([d[base + i * 2], d[base + 1 + i * 2]]);
        }
        if self.program == Program::Dsp2 {
            self.d2.unpack(&d[base + self.ram.len() * 2..]);
        }
        if self.program == Program::Dsp4 {
            self.d4.unpack(&d[base + self.ram.len() * 2..]);
        }
        if self.st() {
            let at = base + self.ram.len() * 2;
            self.mail.poll = u64::from_le_bytes(d[at..at + 8].try_into().expect("eight bytes"));
        }
        if matches!(self.program, Program::Dsp1 | Program::Dsp1b) {
            let at = base;
            for (k, w) in self.att.iter_mut().flatten().enumerate() {
                *w = i16::from_le_bytes([d[at + 2 * k], d[at + 2 * k + 1]]);
            }
            let at = at + 54;
            let (w, n) = (u16::from_le_bytes([d[at + 1], d[at + 2]]), u16::from_le_bytes([d[at + 3], d[at + 4]]));
            self.varying = (d[at] != 0xFF).then_some((d[at], w, n));
            for (k, p) in self.proj.iter_mut().enumerate() {
                *p = u16::from_le_bytes([d[at + 5 + 2 * k], d[at + 6 + 2 * k]]);
            }
        }
    }
}

/// Inverse (10h): a·2^b normalised, a's mantissa in [4000h, 8000h) or [-8000h, -4000h), and 2^29 over it floored, as
/// the mantissa and exponent of the reciprocal; also the shifts and whether the normalised value was a power of two.
fn inverse(a: i16, b: i16) -> (i16, i16, u16, bool) {
    if a == 0 {
        return (0x7FFF, 0x2F, 0, false);
    }
    let (mut m, mut s) = (a as i32, 0u16);
    let r = if m > 0 {
        while m < 0x4000 {
            m <<= 1;
            s += 1;
        }
        ((1i32 << 29) / m).min(0x7FFF)
    } else {
        while m >= -0x4000 {
            m *= 2;
            s += 1;
        }
        -((1i32 << 29) / -m)
    };
    (r as i16, (1 + s as i32 - b as i32) as i16, s, m == 0x4000 || m == -0x8000)
}

/// The DSP-1's projection from Parameter's inputs, by the member of §49.2's family chosen by exact share
/// (VenusRT_Native.md §52.2): exact quotients floored, the eye's height whole, its horizontal offset scaled once.
struct Projection {
    f: [i64; 3],
    lfe: i64,
    les: i64,
    sa: i64,
    ca: i64,
    sz: i64,
    cz: i64,
    /// Fz + Lfe·cos z, floored.
    ez: i64,
    /// Past the limit (VenusRT_Native.md §52.5): the view's angle is the limit's, the screen stands at Les/cos Δ.
    past: bool,
    sv: i64,
    cv: i64,
    cd: i64,
    /// Les_v·cos z_v in Q15: Les·cos z below the limit.
    lv: i64,
    vof: i64,
}

/// The DSP-1's limit angle on Azs (VenusRT_Native.md §52.3): measured from the image to within one step, the one
/// constant VenusRT_DspHle.md §1.3's amendment admits.
const LIMIT: u16 = 0x38CE;

impl Projection {
    fn new(p: &[u16; 7]) -> Projection {
        let w = |k: usize| p[k] as i16 as i64;
        let (a, z) = (p[5], p[6]);
        let (sa, ca, sz, cz) = (sine(a), sine(a.wrapping_add(0x4000)), sine(z), sine(z.wrapping_add(0x4000)));
        let past = (LIMIT..0x8000).contains(&z) || (0x8000..=LIMIT.wrapping_neg()).contains(&z);
        let zv = if !past { z } else if z < 0x8000 { LIMIT } else { LIMIT.wrapping_neg() };
        let d = z.wrapping_sub(zv);
        let (sv, cv, sd, cd) = (sine(zv), sine(zv.wrapping_add(0x4000)), sine(d), sine(d.wrapping_add(0x4000)));
        let les = w(4);
        let (lv, vof) = if past {
            let num = les as i128 * sd as i128;
            let vof = Self::floor_div(2 * num + cd as i128, 2 * cd as i128, (-0x8000, 0x7FFF));
            (Self::floor_div(((les * cv) as i128) << 15, cd as i128, (i64::MIN / 4, i64::MAX / 4)), vof)
        } else {
            (les * cz, 0)
        };
        Projection { f: [w(0), w(1), w(2)], lfe: w(3), les, sa, ca, sz, cz, ez: w(2) + ((w(3) * cz) >> 15), past, sv, cv, cd, lv, vof }
    }

    /// num/den exactly, floored and saturated; a zero denominator saturates by the numerator's sign.
    fn quotient(num: i128, den: i128) -> i64 {
        Self::floor_div(num, den, (-0x8000, 0x7FFF))
    }

    /// num/den exactly, floored, held within `range`.
    fn floor_div(num: i128, den: i128, range: (i64, i64)) -> i64 {
        if den == 0 {
            return if num >= 0 { range.1 } else { range.0 };
        }
        let (n, d) = if den < 0 { (-num, -den) } else { (num, den) };
        n.div_euclid(d).clamp(range.0 as i128, range.1 as i128) as i64
    }

    /// num/den through the Inverse routine, rounded half up or floored, held within `range`.
    fn divide(num: i128, den: i128, half: bool, range: (i64, i64)) -> i64 {
        let (mut d, mut extra) = (den, 0u32);
        while d > i64::MAX as i128 / 4 || d < i64::MIN as i128 / 4 {
            d >>= 1;
            extra += 1;
        }
        let Some((r, shift)) = reciprocal(d as i64) else { return if num >= 0 { range.1 } else { range.0 } };
        let sh = shift + extra;
        let p = num * r as i128 + if half { 1i128 << (sh - 1) } else { 0 };
        (p >> sh).clamp(range.0 as i128, range.1 as i128) as i64
    }

    /// Vof, Vva, Cx, Cy: the horizon's line and where the view's axis meets the ground from the eye; past the limit
    /// the view's angle is the limit's and Vof = Les·tan Δ (§52.5).
    fn parameter(&self) -> [i64; 4] {
        let (sa, ca, sz) = (self.sa, self.ca, self.sz);
        let ex = self.f[0] - ((self.lfe * sz * sa) >> 30);
        let ey = self.f[1] + ((self.lfe * sz * ca) >> 30);
        let t = Self::quotient(self.ez as i128 * self.sv as i128, self.cv as i128);
        let vva = Self::quotient(-(self.lv as i128), self.sv as i128);
        [self.vof, vva, (ex + ((t * sa) >> 15)).clamp(-0x8000, 0x7FFF), (ey - ((t * ca) >> 15)).clamp(-0x8000, 0x7FFF)]
    }

    /// The mode 7 matrix for line v: the scale 256·Ez over the whole denominator Les·cos z + v·sin z, and over cos z;
    /// past the limit, on the limit's view and screen, and K times cos Δ.
    fn raster(&self, v: i64) -> [i64; 4] {
        let n = (self.lv >> 15) + ((v * self.sv) >> 15);
        let k = Self::quotient(256 * self.ez as i128, n as i128);
        let kk = if self.past { Self::quotient(k as i128 * self.cd as i128, self.cv as i128) } else { Self::quotient((k as i128) << 15, self.cv as i128) };
        [(k * self.ca) >> 15, (-kk * self.sa) >> 15, (k * self.sa) >> 15, (kk * self.ca) >> 15]
    }

    /// Target (VenusRT_Native.md §50.3): (Cx, Cy) plus Raster's line-V matrix, at full width, applied to (H, V)/256,
    /// with H along (cos a, -sin a) as the chip turns it.
    fn target(&self, h: i64, v: i64) -> [i64; 2] {
        let [_, _, cx, cy] = self.parameter();
        let wide = (i64::MIN / 4, i64::MAX / 4);
        let n = (self.lv >> 15) + ((v * self.sv) >> 15);
        let k = Self::floor_div(256 * self.ez as i128, n as i128, wide) as i128;
        let kk = Self::floor_div(k << 15, self.cv as i128, wide) as i128;
        let (h, v, sa, ca) = (h as i128, v as i128, self.sa as i128, self.ca as i128);
        let dx = (((h * k * ca) >> 15) - ((v * kk * sa) >> 15)) >> 8;
        let dy = (((v * kk * ca) >> 15) - ((h * k * sa) >> 15)) >> 8;
        [(cx as i128 + dx).clamp(-0x8000, 0x7FFF) as i64, (cy as i128 + dy).clamp(-0x8000, 0x7FFF) as i64]
    }

    /// H, V and M for a point: its offset from the eye along the screen's right, down and forward axes; the view's
    /// elements floored, the eye with its fractions, and the depth w taken whole.
    fn project(&self, p: [i64; 3]) -> [i64; 3] {
        let (sa, ca, sz, cz) = (self.sa, self.ca, self.sz, self.cz);
        let m = |a: i64, b: i64| ((a * b) >> 15) << 15;
        let r = [ca << 15, sa << 15, 0];
        let u = [m(-sa, cz), m(ca, cz), -sz << 15];
        let f = [m(sa, sz), m(-ca, sz), -cz << 15];
        let e = [(self.f[0] << 15) - ((self.lfe * f[0]) >> 15), (self.f[1] << 15) - ((self.lfe * f[1]) >> 15), (self.f[2] << 15) + self.lfe * cz];
        let d = [(p[0] << 15) - e[0], (p[1] << 15) - e[1], (p[2] << 15) - e[2]];
        let dot = |row: [i64; 3]| (0..3).map(|k| d[k] as i128 * row[k] as i128).sum::<i128>() >> 30;
        let (x, y, w) = (dot(r), dot(u), (dot(f) >> 15) << 15);
        let les = self.les as i128;
        [Self::quotient(les * x, w), Self::quotient(les * y, w), Self::quotient((256 * les) << 15, w)]
    }
}

/// Gyrate (VenusRT_Native.md §50.1, §50.3): the angles (Az, Ax, Ay) turned by (U, F, L) about the body's axes, each
/// increment saturated and added with wrapping; 1/cos Ax through the Inverse routine, floored.
fn gyrate(i: &[i64; WORDS]) -> [i64; 3] {
    let (az, ax, ay, u, f, l) = (i[0], i[1], i[2], i[3], i[4], i[5]);
    let (sx, cx) = (sine(ax as u16), sine((ax as u16).wrapping_add(0x4000)));
    let (sy, cy) = (sine(ay as u16), sine((ay as u16).wrapping_add(0x4000)));
    let sat = |v: i64| v.clamp(-0x8000, 0x7FFF);
    let daz = Projection::divide((u * cy - f * sy) as i128, cx as i128, false, (-0x8000, 0x7FFF));
    let dax = sat((u * sy + f * cy) >> 15);
    let tan = Projection::divide((sx as i128) << 15, cx as i128, false, (i64::MIN / 4, i64::MAX / 4));
    let day = sat(l - ((((u * cy + f * sy) >> 15) as i128 * tan as i128) >> 15) as i64);
    let wrap = |a: i64, d: i64| (a + d) as u16 as i16 as i64;
    [wrap(az, daz), wrap(ax, dax), wrap(ay, day)]
}

/// The Inverse routine's reciprocal at any width (§44.2, VenusRT_Native.md §49.2): w normalised to a mantissa in
/// [4000h, 8000h) or [-8000h, -4000h), and 2^29 over it, floored; 1/w is about r / 2^shift. None for 0.
pub fn reciprocal(w: i64) -> Option<(i64, u32)> {
    if w == 0 {
        return None;
    }
    let (mut m, mut s) = (w, 0i32);
    let r = if m > 0 {
        while m >= 0x8000 {
            m >>= 1;
            s -= 1;
        }
        while m < 0x4000 {
            m <<= 1;
            s += 1;
        }
        ((1i64 << 29) / m).min(0x7FFF)
    } else {
        while m < -0x8000 {
            m >>= 1;
            s -= 1;
        }
        while m >= -0x4000 {
            m *= 2;
            s += 1;
        }
        -((1i64 << 29) / -m)
    };
    Some((r, (29 - s) as u32))
}

/// Attitude's matrix: m/2 times Rx(I4)·Ry(I3)·Rz(I2), Q15, with the replacement's sine (§44).
fn attitude(m: i16, z: u16, y: u16, x: u16) -> [i16; 9] {
    let sc = |a: u16| (sine(a) as f64 / 32768.0, sine(a.wrapping_add(0x4000)) as f64 / 32768.0);
    let ((sz, cz), (sy, cy), (sx, cx)) = (sc(z), sc(y), sc(x));
    let rz = [cz, -sz, 0.0, sz, cz, 0.0, 0.0, 0.0, 1.0];
    let ry = [cy, 0.0, sy, 0.0, 1.0, 0.0, -sy, 0.0, cy];
    let rx = [1.0, 0.0, 0.0, 0.0, cx, -sx, 0.0, sx, cx];
    let mul = |a: [f64; 9], b: [f64; 9]| std::array::from_fn(|k| (0..3).map(|t| a[k / 3 * 3 + t] * b[t * 3 + k % 3]).sum::<f64>());
    let r: [f64; 9] = mul(mul(rx, ry), rz);
    r.map(|v| (v * m as f64 / 2.0).floor().clamp(-32768.0, 32767.0) as i16)
}

/// The sine and derivative tables of VenusRT_Native.md §40.3, generated once from the formula, 256 words each.
fn tables() -> &'static [i32; 512] {
    static TABLES: std::sync::OnceLock<[i32; 512]> = std::sync::OnceLock::new();
    TABLES.get_or_init(|| {
        let mut t = [0; 512];
        for (k, (scale, phase)) in [(1.0, 0), (2.0 * std::f64::consts::PI / 256.0, 64)].into_iter().enumerate() {
            for i0 in 0..256 {
                let i = (i0 + phase) % 256;
                let half = i % 128;
                let q = if half > 64 { 128 - half } else { half };
                let m = (32768.0 * scale * quarter_sine(q)).floor() as i32;
                t[k * 256 + i0] = if i >= 128 { -m } else { m }.clamp(-0x8000, 0x7FFF);
            }
        }
        t
    })
}

/// sin(2π·q/256) for q in 0..=64 by its Taylor series in f64 arithmetic alone, so every platform generates the same
/// table; the series converges far past a 16-bit table's precision there.
pub(super) fn quarter_sine(q: usize) -> f64 {
    let x = 2.0 * std::f64::consts::PI * q as f64 / 256.0;
    let (mut term, mut sum) = (x, 0.0);
    for n in 0..14 {
        sum += term;
        term *= -x * x / ((2 * n + 2) as f64 * (2 * n + 3) as f64);
    }
    sum
}

/// The replacement's sine of angle `a`, 2^16 a turn: the table at its top 8 bits and a first-order step by the
/// derivative table over the rest, half up (VenusRT_Native.md §40.2's closest member), held to 16 bits.
pub fn sine(a: u16) -> i64 {
    let t = tables();
    let i = (a >> 8) as usize;
    let f = ((a & 0xFF) as i64) << 7;
    (t[i] as i64 + ((f * t[256 + i] as i64 + (1 << 14)) >> 15)).clamp(-0x8000, 0x7FFF)
}

/// The tables as 16-bit little-endian words, sine then derivative, for firmwarecheck and the independent generator.
pub fn tables_image() -> Vec<u8> {
    tables().iter().flat_map(|&w| (w as i16).to_le_bytes()).collect()
}

/// Each phase's work and notice in chip cycles, as `dsporacle::timing` measured them (VenusRT_Native.md §39.3); a
/// phase not listed takes notice 2.
fn dsp1_timing(command: u8) -> &'static [(u16, u16)] {
    match command {
        0x00 | 0x20 => &[(0, 2), (14, 2), (0, 4), (0, 3)],
        0x08 => &[(0, 2), (16, 2), (0, 2), (0, 6), (0, 2), (0, 3)],
        0x18 | 0x38 => &[(0, 2), (14, 2), (0, 2), (0, 2), (0, 6), (0, 3)],
        0x0F => &[(0, 2), (4121, 3337), (0, 3)],
        // The medians of phases that vary with the inputs (VenusRT_Native.md §40.3).
        0x04 => &[(0, 2), (14, 2), (32, 4), (0, 2), (0, 3)],
        0x0C => &[(0, 2), (14, 2), (0, 2), (31, 4), (0, 2), (0, 3)],
        0x1C => &[(0, 2), (15, 2), (0, 2), (0, 2), (0, 2), (0, 2), (93, 33), (0, 2), (0, 2), (0, 3)],
        0x2F => &[(0, 2), (16, 2), (0, 3)],
        // Measured constants (§37.3); Attitude's last phase, which varies, takes its medians (§44).
        0x01 | 0x11 | 0x21 => &[(0, 2), (15, 2), (0, 2), (0, 2), (125, 95)],
        0x03 | 0x13 | 0x23 | 0x0D | 0x1D | 0x2D => &[(0, 2), (15, 2), (0, 2), (0, 5), (0, 2), (0, 2), (0, 3)],
        0x0B | 0x1B | 0x2B => &[(0, 2), (17, 2), (0, 2), (0, 5), (0, 3)],
        0x10 | 0x30 => &[(0, 2), (15, 2), (0, 40), (0, 2), (0, 3)],
        0x28 => &[(0, 2), (17, 2), (0, 2), (0, 66), (0, 3)],
        // The medians of Parameter's and Project's phases that vary, after a Parameter (VenusRT_Native.md §49.5).
        0x02 => &[(0, 2), (13, 2), (0, 2), (0, 2), (0, 2), (0, 2), (0, 2), (527, 485), (0, 2), (10, 2), (0, 2), (0, 3)],
        0x06 => &[(0, 2), (14, 2), (0, 2), (0, 362), (0, 2), (0, 2), (0, 3)],
        // Target's and Gyrate's medians (VenusRT_Native.md §50.3).
        0x0E | 0x1E | 0x2E | 0x3E => &[(0, 2), (13, 2), (0, 116), (0, 2), (0, 3)],
        0x14 => &[(0, 2), (16, 2), (0, 2), (0, 2), (0, 2), (0, 2), (237, 3), (0, 2), (0, 2), (0, 3)],
        _ => &[],
    }
}

/// A DSP-1 command's transfers, inputs, results and whether it runs on: fullsnes's codes, SnesLab's parameter lists,
/// SNESdev's two words for Radius, and Gyrate's six and three as characterised (VenusRT_Native.md §37.5). A byte no
/// document names takes its one transfer and gives nothing; None for 40h-FFh, which are passed over.
fn dsp1_shape(command: u8) -> Option<(u8, u16, bool)> {
    if command >= 0x40 {
        return None;
    }
    Some(match command {
        0x00 | 0x20 => (2, 1, false),
        0x10 => (2, 2, false),
        0x04 => (2, 2, false),
        0x08 => (3, 2, false),
        0x18 | 0x38 => (4, 1, false),
        0x28 => (3, 1, false),
        0x0C => (3, 2, false),
        0x1C => (6, 3, false),
        0x14 => (6, 3, false),
        0x02 => (7, 4, false),
        0x06 => (3, 3, false),
        0x0E | 0x1E | 0x2E | 0x3E => (2, 2, false),
        0x0A | 0x1A | 0x2A | 0x3A => (1, 4, true),
        0x01 | 0x11 | 0x21 => (4, 0, false),
        0x03 | 0x13 | 0x23 | 0x0D | 0x1D | 0x2D => (3, 3, false),
        0x0B | 0x1B | 0x2B => (3, 1, false),
        0x0F | 0x2F => (0, 1, false),
        0x1F => (0, 1024, false),
        _ => (0, 0, false),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tick(d: &mut DspHle, cycles: u64) {
        let clock = ((d.cycles + cycles) as u128 * d.ratio.1 as u128 / d.ratio.0 as u128) as u64 + 1;
        d.run_to(clock);
    }

    fn put(d: &mut DspHle, w: u16) {
        d.host_write(Port::Dr, w as u8);
        if d.sr & sr::DRS != 0 {
            d.host_write(Port::Dr, (w >> 8) as u8);
        }
    }

    // fullsnes: 2Fh answers 0100h on the DSP-1 and 0101h on the DSP-1B, and DR holds 80h on completion.
    #[test]
    fn the_rom_version_answers_as_fullsnes_says() {
        for (program, version) in [(Program::Dsp1, 0x0100), (Program::Dsp1b, 0x0101)] {
            let mut d = DspHle::new(program);
            tick(&mut d, 10);
            assert_eq!((d.host_read(Port::Sr, false), d.dr), (0x84, 0x80));
            put(&mut d, 0x2F);
            tick(&mut d, 10);
            assert_eq!(d.sr & sr::RQM, sr::RQM);
            put(&mut d, 0);
            tick(&mut d, 10);
            let got = d.host_read(Port::Dr, true) as u16 | (d.host_read(Port::Dr, true) as u16) << 8;
            assert_eq!(got, version);
            tick(&mut d, 10);
            assert_eq!((d.host_read(Port::Sr, false), d.dr), (0x84, 0x80));
        }
    }

    // SNESdev's formulas through the ports: 4000h·4000h·2^-15 is 2000h; 3² + 4² is 25, sent as 50 halves.
    #[test]
    fn the_documented_formulas_answer_through_the_ports() {
        let mut d = DspHle::new(Program::Dsp1b);
        let mut ask = |command: u16, inputs: &[u16], outputs: usize| -> Vec<u16> {
            tick(&mut d, 10);
            put(&mut d, command);
            for &w in inputs {
                tick(&mut d, 20);
                put(&mut d, w);
            }
            (0..outputs)
                .map(|_| {
                    tick(&mut d, 20);
                    d.host_read(Port::Dr, true) as u16 | (d.host_read(Port::Dr, true) as u16) << 8
                })
                .collect()
        };
        assert_eq!(ask(0x00, &[0x4000, 0x4000], 1), [0x2000]);
        assert_eq!(ask(0x08, &[3, 4, 0], 2), [50, 0]);
        assert_eq!(ask(0x18, &[3, 4, 0, 5], 1), [0]);
    }

    // fullsnes's mailbox: the command at byte 0020h, bit 7 of 0021h set, and cleared on completion; the start word read first.
    #[test]
    fn the_st010_mailbox_completes_after_the_start_word() {
        let mut d = DspHle::new(Program::St010);
        tick(&mut d, 10);
        d.host_write(Port::Ram(0x20), 0x00);
        d.host_write(Port::Ram(0x21), 0x80);
        tick(&mut d, 100);
        assert_eq!(d.ram[0x10], 0x8000, "the mailbox runs before the start word is read");
        d.host_read(Port::Dr, true);
        d.host_read(Port::Dr, true);
        d.host_write(Port::Ram(0x21), 0x80);
        tick(&mut d, 100);
        assert_eq!(d.ram[0x10], 0);
    }

    // A raster run gives line after line until the S-CPU writes another word over a line's last result, then the chip
    // is idle (VenusRT_Native.md §49.5).
    #[test]
    fn a_raster_run_ends_on_a_write() {
        let mut d = DspHle::new(Program::Dsp1b);
        tick(&mut d, 10);
        put(&mut d, 0x02);
        for w in [0, 0, 100, 64, 256, 0, 0x3400] {
            tick(&mut d, 20);
            put(&mut d, w);
        }
        let mut get = |d: &mut DspHle| {
            tick(d, 600);
            assert_eq!(d.sr & sr::RQM, sr::RQM);
            d.host_read(Port::Dr, true) as u16 | (d.host_read(Port::Dr, true) as u16) << 8
        };
        (0..4).for_each(|_| _ = get(&mut d));
        tick(&mut d, 10);
        put(&mut d, 0x0A);
        tick(&mut d, 10);
        put(&mut d, 0x0000);
        let lines: Vec<u16> = (0..16).map(|_| get(&mut d)).collect();
        assert!(lines[0] > lines[12] && lines[12] > 0, "the scale falls line by line: {lines:04X?}");
        for _ in 0..3 {
            tick(&mut d, 200);
            put(&mut d, 0);
        }
        tick(&mut d, 200);
        put(&mut d, 0x8000);
        tick(&mut d, 10);
        assert_eq!((d.host_read(Port::Sr, false), d.dr), (0x84, 0x80));
    }

    // VenusRT_DspHle.md §5.4: no array literal of more than 16 numbers in a replacement's source; tables are generated.
    #[test]
    fn no_replacement_source_holds_a_table_literal() {
        for (name, source) in [("dsphle.rs", include_str!("dsphle.rs")), ("dsp2.rs", include_str!("dsp2.rs")), ("st010.rs", include_str!("st010.rs")), ("dsp4.rs", include_str!("dsp4.rs"))] {
            let mut depth = 0usize;
            let mut items = Vec::<String>::new();
            for ch in source.chars() {
                match ch {
                    '[' => {
                        depth += 1;
                        items = vec![String::new()];
                    }
                    ']' if depth > 0 => {
                        depth -= 1;
                        let numbers = items.iter().filter(|t| !t.trim().is_empty() && t.trim().trim_start_matches("0x").chars().all(|c| c.is_ascii_hexdigit() || c == '_')).count();
                        assert!(numbers <= 16, "{name}: an array literal of {numbers} numbers");
                        items.clear();
                    }
                    ',' if depth > 0 => items.push(String::new()),
                    c if depth > 0 => {
                        if let Some(t) = items.last_mut() {
                            t.push(c);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // VenusRT_DspHle.md §5.2, rule 3: the tables equal an independent generator's, written from the record's formula.
    #[test]
    fn the_tables_equal_the_independent_generator() {
        let script = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../../EmuSen.WiseMan/Reference/analysis/dsp1_tables.py");
        let out = std::env::temp_dir().join(format!("venusrt-dsp1-tables-{}.bin", std::process::id()));
        match std::process::Command::new("python3").arg(script).arg(&out).output() {
            Ok(o) if o.status.success() => {
                let theirs = std::fs::read(&out).unwrap();
                let _ = std::fs::remove_file(&out);
                assert_eq!(theirs.len(), 1024);
                let differ = theirs.iter().zip(tables_image()).filter(|(a, b)| **a != *b).count();
                assert_eq!(differ, 0, "{differ} bytes differ from dsp1_tables.py");
            }
            _ => eprintln!("python3 or dsp1_tables.py unavailable, not run"),
        }
    }

    #[test]
    fn the_dsp1_state_round_trips_with_its_matrices() {
        let mut d = DspHle::new(Program::Dsp1b);
        d.att[2][4] = -1234;
        d.varying = Some((3, 0, 66));
        let mut back = DspHle::new(Program::Dsp1b);
        back.unpack(&d.pack());
        assert_eq!(back.pack(), d.pack());
    }

    #[test]
    fn the_state_round_trips() {
        let mut d = DspHle::new(Program::St010);
        tick(&mut d, 10);
        put(&mut d, 0x1234);
        d.ram[7] = 0xBEEF;
        let mut back = DspHle::new(Program::St010);
        back.unpack(&d.pack());
        assert_eq!(back.pack(), d.pack());
    }
}
