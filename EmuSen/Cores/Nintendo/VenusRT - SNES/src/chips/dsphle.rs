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
    St010,
}

impl Program {
    /// By the file stem `Cartridge::nec_firmware` gives.
    pub fn for_stem(stem: &str) -> Option<Program> {
        match stem {
            "dsp1" => Some(Program::Dsp1),
            "dsp1b" => Some(Program::Dsp1b),
            "dsp2" => Some(Program::Dsp2),
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
        }
    }
}

/// The chip side's next edge, due at `DspHle::due`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Next {
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
enum Stage {
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
    command: u8,
    stage: Stage,
    next: Next,
    due: u64,
    inputs: [u16; WORDS],
    outputs: [u16; WORDS],
    /// The DSP-2's idle word: 00h after power-on, FFh after a command (VenusRT_Native.md §37.2).
    idle_word: u16,
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
        if self.st() {
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

    fn schedule(&mut self, next: Next, after: u64) {
        self.next = next;
        self.due = self.cycles + after;
    }

    /// Catches the chip up to the master clock `clock`, making its due edge on the way.
    pub fn run_to(&mut self, clock: u64) {
        let target = (clock as u128 * self.ratio.0 as u128 / self.ratio.1 as u128) as u64;
        if target <= self.cycles {
            return;
        }
        while self.next != Next::None && self.due <= target {
            self.cycles = self.cycles.max(self.due);
            self.edge();
        }
        self.cycles = target;
    }

    fn edge(&mut self) {
        match std::mem::replace(&mut self.next, Next::None) {
            Next::None => {}
            Next::Request { wide } => {
                let narrow = self.sr & sr::DRC != 0;
                self.sr = (self.sr & !sr::DRC) | if wide && !narrow { 0 } else { sr::DRC } | sr::RQM;
                if wide && narrow {
                    self.schedule(Next::Widen, 3);
                }
            }
            Next::Widen => self.sr &= !sr::DRC,
            Next::Offer(v) => {
                self.dr = v;
                self.sr = (self.sr & !sr::DRC) | if self.program == Program::Dsp2 { sr::DRC } else { 0 } | sr::RQM;
            }
            Next::Idle => {
                self.stage = Stage::Idle;
                self.dr = self.idle_word;
                self.sr = (self.sr & !sr::DRS) | sr::DRC | sr::RQM;
            }
            Next::Done => self.ram[0x10] = 0,
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
                if i & 0xFFF == 0x21 && value & 0x80 != 0 && self.stage == Stage::StMailbox {
                    self.command = self.ram[0x10] as u8;
                    self.schedule(Next::Done, 16);
                }
            }
            Port::Dr => {
                let high = self.sr & (sr::DRC | sr::DRS) == sr::DRS;
                self.dr = if high { (self.dr & 0x00FF) | (value as u16) << 8 } else { (self.dr & 0xFF00) | value as u16 };
                if let Stage::Giving { ref mut overwritten, .. } = self.stage {
                    *overwritten = true;
                }
                if self.byte_done() {
                    self.completed();
                }
            }
        }
    }

    /// One byte through DR as the handshake counts it, true when it completed the transfer.
    fn byte_done(&mut self) -> bool {
        if self.sr & sr::DRC != 0 {
            self.sr &= !sr::RQM;
            true
        } else if self.sr & sr::DRS == 0 {
            self.sr |= sr::DRS;
            false
        } else {
            self.sr &= !(sr::DRS | sr::RQM);
            true
        }
    }

    /// The S-CPU completed a transfer in either direction; fullsnes's chip is "oblivious" to which.
    fn completed(&mut self) {
        match self.stage {
            Stage::StStart => self.stage = Stage::StMailbox,
            Stage::StMailbox => {}
            Stage::Idle => self.command(self.dr as u8),
            Stage::Taking { want, taken } => {
                if (taken as usize) < WORDS {
                    self.inputs[taken as usize] = self.dr;
                }
                let taken = taken + 1;
                if taken < want {
                    self.stage = Stage::Taking { want, taken };
                    self.schedule(Next::Request { wide: true }, NOTICE);
                } else {
                    self.compute();
                }
            }
            Stage::Giving { total, given, run, overwritten } => {
                let given = given + 1;
                if given < total {
                    self.stage = Stage::Giving { total, given, run, overwritten };
                    let v = self.outputs.get(given as usize).copied().unwrap_or(0);
                    self.schedule(Next::Offer(v), NOTICE);
                } else if run && !overwritten {
                    self.stage = Stage::Giving { total, given: 0, run, overwritten: false };
                    self.schedule(Next::Offer(self.outputs[0]), NOTICE);
                } else {
                    self.schedule(Next::Idle, NOTICE);
                }
            }
        }
    }

    fn command(&mut self, command: u8) {
        self.command = command;
        if self.program == Program::Dsp2 {
            // Only 0Fh, the documented no-op, is known; every byte returns to idle with DR FFh.
            self.idle_word = 0xFF;
            self.schedule(Next::Idle, 14);
            return;
        }
        let Some((inputs, _, _)) = dsp1_shape(command) else {
            // 40h-FFh are passed over: the next byte is taken as a command.
            self.schedule(Next::Request { wide: false }, NOTICE);
            return;
        };
        self.stage = Stage::Taking { want: inputs.max(1), taken: 0 };
        self.schedule(Next::Request { wide: true }, NOTICE);
    }

    /// The command's results; the frame answers only what fullsnes fixes.
    fn compute(&mut self) {
        let (_, total, run) = dsp1_shape(self.command).unwrap_or((0, 0, false));
        self.outputs = [0; WORDS];
        if self.command == 0x2F {
            self.outputs[0] = if self.program == Program::Dsp1 { 0x0100 } else { 0x0101 };
        }
        if total == 0 {
            self.schedule(Next::Idle, NOTICE);
        } else {
            self.stage = Stage::Giving { total, given: 0, run, overwritten: false };
            self.schedule(Next::Offer(self.outputs[0]), NOTICE);
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
        for w in self.ram.iter() {
            o.extend(w.to_le_bytes());
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
        for (i, r) in self.ram.iter_mut().enumerate() {
            *r = u16::from_le_bytes([d[at + 16 + i * 2], d[at + 17 + i * 2]]);
        }
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
        0x0E => (2, 2, false),
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

    // A raster run repeats its line until the S-CPU writes over a result, then the chip is idle.
    #[test]
    fn a_raster_run_ends_on_a_write() {
        let mut d = DspHle::new(Program::Dsp1b);
        tick(&mut d, 10);
        put(&mut d, 0x0A);
        tick(&mut d, 10);
        put(&mut d, 0x0040);
        for _ in 0..8 {
            tick(&mut d, 10);
            assert_eq!(d.sr & sr::RQM, sr::RQM);
            d.host_read(Port::Dr, true);
            d.host_read(Port::Dr, true);
        }
        for _ in 0..4 {
            tick(&mut d, 10);
            put(&mut d, 0);
        }
        tick(&mut d, 10);
        assert_eq!((d.host_read(Port::Sr, false), d.dr), (0x84, 0x80));
    }

    // VenusRT_DspHle.md §5.4: no array literal of more than 16 numbers in a replacement's source; tables are generated.
    #[test]
    fn no_replacement_source_holds_a_table_literal() {
        for (name, source) in [("dsphle.rs", include_str!("dsphle.rs"))] {
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
