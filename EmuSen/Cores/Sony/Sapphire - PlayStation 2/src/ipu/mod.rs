//! The IPU (EE User's Manual chapter 8; ps2tek "Image Processing Unit"): its ten commands, IPU_CMD, IPU_CTRL, IPU_BP
//! and IPU_TOP, the input FIFO and its internal buffer (`bits`), the output FIFO, its interrupt and the DMAC's channels
//! 3 and 4. IDEC and BDEC decode MPEG-2 (or, with IPU_CTRL.MP1, MPEG-1) macroblocks as ITU-T H.262 (02/2000) defines
//! the decoding: the variable-length codes of Annex B (`vlc`), the inverse scan of §7.3, the inverse quantisation,
//! saturation and mismatch control of §7.4, and Annex A's reference IDCT (`idct`); CSC, PACK and IDEC's output go
//! through §8.6's post-processing (`csc`). A command runs symbol by symbol as its bits arrive, each symbol waiting for
//! the 32 bits the manual asks to be present, and its output waits for room in the output FIFO; the decoding itself
//! takes no time (a named loss, Sapphire_Native.md §34.8). Sapphire_Native.md §34.

pub mod bits;
pub mod csc;
pub mod idct;
pub mod vlc;

use std::collections::VecDeque;

use bits::Input;
use vlc::{Coef, mb, tables};

/// The registers and FIFOs in the EE's map (§2.2.1, §8.3).
pub const CMD: u32 = 0x1000_2000;
pub const CTRL: u32 = 0x1000_2010;
pub const BP: u32 = 0x1000_2020;
pub const TOP: u32 = 0x1000_2030;
pub const REGS_END: u32 = 0x1000_2040;
pub const OUT_FIFO: u32 = 0x1000_7000;
pub const IN_FIFO: u32 = 0x1000_7010;
pub const FIFO_END: u32 = 0x1000_7020;

/// IPU_out_FIFO's depth (§8.3).
pub const OUT_QWORDS: usize = 8;

/// IPU_CTRL's fields (§8.3).
pub mod ctrl {
    pub const ECD: u32 = 1 << 14;
    pub const SCD: u32 = 1 << 15;
    pub const AS: u32 = 1 << 20;
    pub const IVF: u32 = 1 << 21;
    pub const QST: u32 = 1 << 22;
    pub const MP1: u32 = 1 << 23;
    pub const RST: u32 = 1 << 30;
    pub const BUSY: u32 = 1 << 31;
    /// IDP, AS, IVF, QST, MP1 and PCT, the fields a write keeps.
    pub const WRITABLE: u32 = 3 << 16 | AS | IVF | QST | MP1 | 7 << 24;
}

/// The commands by IPU_CMD's CODE (§8.4).
pub mod code {
    pub const BCLR: u32 = 0;
    pub const IDEC: u32 = 1;
    pub const BDEC: u32 = 2;
    pub const VDEC: u32 = 3;
    pub const FDEC: u32 = 4;
    pub const SETIQ: u32 = 5;
    pub const SETVQ: u32 = 6;
    pub const CSC: u32 = 7;
    pub const PACK: u32 = 8;
    pub const SETTH: u32 = 9;
}

/// quantiser_scale by q_scale_type and quantiser_scale_code (H.262 Table 7-6).
const Q_SCALE: [[u8; 32]; 2] = [
    [0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32, 34, 36, 38, 40, 42, 44, 46, 48, 50, 52, 54, 56, 58, 60, 62],
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 10, 12, 14, 16, 18, 20, 22, 24, 28, 32, 36, 40, 44, 48, 52, 56, 64, 72, 80, 88, 96, 104, 112],
];

/// scan[alternate_scan][v][u], the index n of QFS at row v, column u (H.262 Figures 7-2 and 7-3).
pub const SCAN: [[u8; 64]; 2] = [
    [
        0, 1, 5, 6, 14, 15, 27, 28, 2, 4, 7, 13, 16, 26, 29, 42, 3, 8, 12, 17, 25, 30, 41, 43, 9, 11, 18, 24, 31, 40, 44, 53, 10, 19, 23, 32,
        39, 45, 52, 54, 20, 22, 33, 38, 46, 51, 55, 60, 21, 34, 37, 47, 50, 56, 59, 61, 35, 36, 48, 49, 57, 58, 62, 63,
    ],
    [
        0, 4, 6, 20, 22, 36, 38, 52, 1, 5, 7, 21, 23, 37, 39, 53, 2, 8, 19, 24, 34, 40, 50, 54, 3, 9, 18, 25, 35, 41, 51, 55, 10, 17, 26, 30,
        42, 46, 56, 60, 11, 16, 27, 31, 43, 47, 57, 61, 12, 15, 28, 32, 44, 48, 58, 62, 13, 14, 29, 33, 45, 49, 59, 63,
    ],
];

/// The bits a symbol waits for: "a code of 32 bits or more must be supplied" (§8.4, VDEC's and FDEC's notes).
const SYMBOL: u32 = 32;

/// Where a macroblock command is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    /// IDEC: the slice's first macroblock, whose MBAI the program has read.
    First,
    /// IDEC: a start code, or the next macroblock's MBAI.
    Next,
    MbType,
    Modes,
    Cbp,
    Block,
    EndOfMacroblock,
    /// BDEC: the bits after the last block's EOB.
    After,
}

/// A block's next symbol.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum In {
    DcSize,
    Coef,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Job {
    Idle,
    Idec,
    Bdec,
    Vdec,
    Fdec,
    SetIq,
    SetVq,
    Csc,
    Pack,
    /// The command has ended; its output is still leaving for the FIFO.
    Ending,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Step {
    Wait,
    Go,
    End,
}

/// The macroblock being decoded.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Macroblock {
    intra: bool,
    quant: bool,
    field_dct: bool,
    qsc: u32,
    /// pattern_code, block i coded when bit 5 − i is set (H.262 §6.3.17.4).
    pattern: u8,
    block: usize,
    at: In,
    n: usize,
    qfs: [i32; 64],
    out: [[i16; 64]; 6],
}

impl Default for Macroblock {
    fn default() -> Self {
        Macroblock { intra: true, quant: false, field_dct: false, qsc: 0, pattern: 0, block: 0, at: In::DcSize, n: 0, qfs: [0; 64], out: [[0; 64]; 6] }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ipu {
    pub input: Input,
    out: VecDeque<u128>,
    /// A command's output not yet in the FIFO.
    pending: [u128; 64],
    pending_at: usize,
    pending_len: usize,
    /// IPU_CTRL but IFC, OFC and BUSY.
    ctrl: u32,
    busy: bool,
    cmd: u32,
    job: Job,
    phase: Phase,
    /// FB's bits still to skip before the command proper.
    skip: u32,
    /// IPU_CMD's DATA, and whether a command has been issued since the reset (ps2tek's quirk, D-311).
    data: u32,
    issued: bool,
    th: (u32, u32),
    /// The intra and non-intra matrices W[v][u], row by row.
    iq: [[u8; 64]; 2],
    vq: [u16; 16],
    dc_pred: [i32; 3],
    mb: Macroblock,
    /// Bytes gathered for SETIQ, SETVQ, CSC and PACK, and the macroblocks CSC and PACK still owe.
    raw: Box<[u8; 1024]>,
    gathered: usize,
    left: u32,
    irq: bool,
    wake: bool,
}

impl Default for Ipu {
    fn default() -> Self {
        Ipu {
            input: Input::default(),
            out: VecDeque::with_capacity(OUT_QWORDS),
            pending: [0; 64],
            pending_at: 0,
            pending_len: 0,
            ctrl: 0,
            busy: false,
            cmd: 0,
            job: Job::Idle,
            phase: Phase::First,
            skip: 0,
            data: 0,
            issued: false,
            th: (0, 0),
            iq: [[16; 64]; 2],
            vq: [0; 16],
            dc_pred: [0; 3],
            mb: Macroblock::default(),
            raw: Box::new([0; 1024]),
            gathered: 0,
            left: 0,
            irq: false,
            wake: false,
        }
    }
}

impl Ipu {
    pub fn new() -> Ipu {
        Ipu::default()
    }

    pub fn busy(&self) -> bool {
        self.busy
    }

    /// A command's end, the interrupt INT_IPU asks of the INTC (§8.1.2); taken once.
    pub fn take_irq(&mut self) -> bool {
        std::mem::take(&mut self.irq)
    }

    /// Whether the FIFOs have changed in a way that may let a held DMA channel go on; taken once.
    pub fn take_wake(&mut self) -> bool {
        std::mem::take(&mut self.wake)
    }

    /// Qwords channel 4 (toIPU) may give now, of `want`.
    pub fn request_in(&self, want: usize) -> usize {
        want.min(self.input.room())
    }

    /// Qwords channel 3 (fromIPU) may take now, of `want`.
    pub fn request_out(&self, want: usize) -> usize {
        want.min(self.out.len())
    }

    /// A qword into IPU_in_FIFO, from channel 4 or the EE; dropped when the FIFO is full.
    pub fn put(&mut self, q: u128) {
        self.input.push(q);
        self.advance();
    }

    /// A qword from IPU_out_FIFO, to channel 3 or the EE; 0 when it is empty.
    pub fn get(&mut self) -> u128 {
        let q = self.out.pop_front().unwrap_or(0);
        self.advance();
        q
    }

    pub fn ofc(&self) -> usize {
        self.out.len()
    }

    /// IPU_CTRL as read.
    pub fn read_ctrl(&self) -> u32 {
        self.ctrl | (self.input.ifc() as u32) | (self.out.len() as u32) << 4 | u32::from(self.busy) << 31
    }

    /// IPU_BP as read.
    pub fn read_bp(&self) -> u32 {
        self.input.bp() | (self.input.ifc() as u32) << 8 | (self.input.fp() as u32) << 16
    }

    /// IPU_CMD as read: DATA, VDEC's or FDEC's result, and BUSY; before any command, the stream's next 32 bits.
    pub fn read_cmd(&self) -> u64 {
        let data = if self.issued { self.data } else { self.input.peek32() };
        u64::from(data) | u64::from(self.busy) << 63
    }

    /// IPU_TOP as read: the stream's next 32 bits, BUSY while a command runs or fewer than 32 bits are in the IPU.
    pub fn read_top(&self) -> u64 {
        let busy = self.busy || self.input.available() < SYMBOL;
        u64::from(self.input.peek32()) | u64::from(busy) << 63
    }

    /// The register window and the FIFOs, as the EE reads them.
    pub fn read(&mut self, paddr: u32, bytes: usize) -> u128 {
        if (OUT_FIFO..FIFO_END).contains(&paddr) {
            return if paddr < IN_FIFO && bytes == 16 { self.get() } else { 0 };
        }
        let reg = match paddr & !0xF {
            CMD => self.read_cmd(),
            CTRL => u64::from(self.read_ctrl()),
            BP => u64::from(self.read_bp()),
            TOP => self.read_top(),
            _ => 0,
        };
        let v = u128::from(reg) >> (8 * (paddr & 7));
        if bytes >= 16 { v } else { v & ((1u128 << (8 * bytes)) - 1) }
    }

    /// The register window and the FIFOs, as the EE writes them: IPU_CMD's low 32 bits start a command, IPU_CTRL keeps
    /// its writable fields and resets with RST, IPU_in_FIFO takes a qword.
    pub fn write(&mut self, paddr: u32, bytes: usize, value: u128) {
        match paddr & !0xF {
            IN_FIFO if bytes == 16 => self.put(value),
            CMD if bytes >= 4 && paddr & 7 == 0 => self.command(value as u32),
            CTRL if bytes >= 4 => self.write_ctrl(value as u32),
            _ => {}
        }
    }

    pub fn write_ctrl(&mut self, v: u32) {
        if v & ctrl::RST != 0 {
            if self.busy {
                self.irq = true;
            }
            self.input.clear(0);
            self.out.clear();
            self.pending_len = 0;
            self.busy = false;
            self.job = Job::Idle;
            self.issued = false;
            self.ctrl = 0;
            self.wake = true;
        }
        self.ctrl = self.ctrl & !ctrl::WRITABLE | v & ctrl::WRITABLE;
    }

    fn pct(&self) -> u32 {
        self.ctrl >> 24 & 7
    }

    fn mpeg1(&self) -> bool {
        self.ctrl & ctrl::MP1 != 0
    }

    /// IPU_CMD's write: a command begins, ECD and SCD cleared; ignored while one runs.
    pub fn command(&mut self, v: u32) {
        if self.busy {
            return;
        }
        self.cmd = v;
        self.issued = true;
        self.ctrl &= !(ctrl::ECD | ctrl::SCD);
        self.busy = true;
        self.skip = 0;
        self.wake = true;
        let fb = v & 0x3F;
        self.job = match v >> 28 {
            code::BCLR => {
                self.input.clear(v & 0x7F);
                Job::Ending
            }
            code::IDEC => {
                self.skip = fb;
                self.dc_reset();
                self.mb.qsc = v >> 16 & 0x1F;
                self.phase = Phase::First;
                Job::Idec
            }
            code::BDEC => {
                self.skip = fb;
                if v & 1 << 26 != 0 {
                    self.dc_reset();
                }
                let m = &mut self.mb;
                m.qsc = v >> 16 & 0x1F;
                m.field_dct = v & 1 << 25 != 0;
                m.intra = v & 1 << 27 != 0;
                if m.intra {
                    self.begin_blocks(0x3F);
                    self.phase = Phase::Block;
                } else {
                    self.phase = Phase::Cbp;
                }
                Job::Bdec
            }
            code::VDEC => {
                self.skip = fb;
                Job::Vdec
            }
            code::FDEC => {
                self.skip = fb;
                Job::Fdec
            }
            code::SETIQ => {
                self.skip = fb;
                self.gathered = 0;
                Job::SetIq
            }
            code::SETVQ => {
                self.gathered = 0;
                Job::SetVq
            }
            code::CSC | code::PACK => {
                self.gathered = 0;
                self.left = v & 0x7FF;
                if v >> 28 == code::CSC { Job::Csc } else { Job::Pack }
            }
            code::SETTH => {
                self.th = (v & 0x1FF, v >> 16 & 0x1FF);
                Job::Ending
            }
            _ => Job::Ending,
        };
        self.advance();
    }

    fn dc_reset(&mut self) {
        self.dc_pred = [128 << (self.ctrl >> 16 & 3); 3];
    }

    /// Runs the command as far as the input and the output FIFO allow.
    fn advance(&mut self) {
        let before = (self.input.ifc(), self.out.len());
        loop {
            while self.pending_len > 0 && self.out.len() < OUT_QWORDS {
                self.out.push_back(self.pending[self.pending_at]);
                self.pending_at += 1;
                self.pending_len -= 1;
            }
            if !self.busy || self.pending_len > 0 {
                break;
            }
            if self.job == Job::Ending {
                self.busy = false;
                self.job = Job::Idle;
                self.irq = true;
                break;
            }
            if self.skip > 0 {
                if self.input.available() < self.skip {
                    break;
                }
                self.input.skip(self.skip);
                self.skip = 0;
            }
            match self.step() {
                Step::Wait => break,
                Step::Go => {}
                Step::End => self.job = Job::Ending,
            }
        }
        if before != (self.input.ifc(), self.out.len()) {
            self.wake = true;
        }
    }

    fn emit(&mut self, q: &[u128]) {
        self.pending[..q.len()].copy_from_slice(q);
        self.pending_at = 0;
        self.pending_len = q.len();
    }

    /// An error code: ECD, the pointer left at the code (§8.4).
    fn error(&mut self) -> Step {
        self.ctrl |= ctrl::ECD;
        Step::End
    }

    /// At eight or more zero bits: a start code if a one ends a byte after 23 or more of them, the pointer then left at
    /// its prefix, the 23 zeros and the one (H.262 §6.2.1); an error code otherwise (D-305).
    fn start_code(&mut self) -> Step {
        let z = self.input.zeros(u32::MAX);
        if z == self.input.available() {
            return Step::Wait;
        }
        if z >= 23 && (self.input.bp() + z + 1) % 8 == 0 {
            self.input.skip(z - 23);
            self.ctrl |= ctrl::SCD;
            return Step::End;
        }
        self.error()
    }

    fn step(&mut self) -> Step {
        match self.job {
            Job::Idec | Job::Bdec => self.macroblock(),
            Job::Vdec => self.vdec(),
            Job::Fdec => {
                if self.input.available() < SYMBOL {
                    return Step::Wait;
                }
                self.data = self.input.peek32();
                Step::End
            }
            Job::SetIq | Job::SetVq | Job::Csc | Job::Pack => self.gather(),
            Job::Idle | Job::Ending => Step::End,
        }
    }

    /// VDEC (§8.4, §8.5.3): one code of the table TBL names, as its length in bits 31..16 and its value in 15..0.
    fn vdec(&mut self) -> Step {
        if self.input.available() < SYMBOL {
            return Step::Wait;
        }
        let t = tables();
        let bits = self.input.peek32();
        let hit = match self.cmd >> 26 & 3 {
            0 => (if self.mpeg1() { &t.mbai_mpeg1 } else { &t.mbai }).decode(bits).map(|(l, v)| (l, u16::from(v))),
            1 => match self.pct() {
                p @ 1..=4 => t.mb_type[p as usize - 1].decode(bits).map(|(l, v)| (l, u16::from(v))),
                _ => None,
            },
            2 => t.motion_code.decode(bits).map(|(l, v)| (l, v as i16 as u16)),
            _ => t.dmvector.decode(bits).map(|(l, v)| (l, v as i16 as u16)),
        };
        match hit {
            Some((len, v)) => {
                self.data = u32::from(len) << 16 | u32::from(v);
                self.input.skip(u32::from(len));
                Step::End
            }
            None => {
                self.data = 0;
                if self.input.zeros(SYMBOL) >= 8 { self.start_code() } else { self.error() }
            }
        }
    }

    /// SETIQ, SETVQ, CSC and PACK take whole bytes; each acts once it has its bytes.
    fn gather(&mut self) -> Step {
        let need = match self.job {
            Job::SetIq => 64,
            Job::SetVq => 32,
            Job::Csc => 384,
            _ => 1024,
        };
        if matches!(self.job, Job::Csc | Job::Pack) && self.left == 0 {
            return Step::End;
        }
        while self.gathered < need {
            if self.input.available() < 8 {
                return Step::Wait;
            }
            self.raw[self.gathered] = self.input.take(8) as u8;
            self.gathered += 1;
        }
        self.gathered = 0;
        let raw = &self.raw;
        let (dither, out16) = (self.cmd & 1 << 26 != 0, self.cmd & 1 << 27 != 0);
        let mut q = [0u128; 64];
        match self.job {
            Job::SetIq => {
                let w = &mut self.iq[usize::from(self.cmd & 1 << 27 != 0)];
                for (i, &n) in SCAN[0].iter().enumerate() {
                    w[i] = raw[usize::from(n)];
                }
                return Step::End;
            }
            Job::SetVq => {
                for (i, e) in self.vq.iter_mut().enumerate() {
                    *e = u16::from(raw[2 * i]) | u16::from(raw[2 * i + 1]) << 8;
                }
                return Step::End;
            }
            Job::Csc => {
                let mut px = [0u32; 256];
                csc::convert_raw8(raw[..384].try_into().unwrap(), self.th.0, self.th.1, &mut px);
                let n = csc::pack(&px, out16, dither && out16, &mut q);
                self.emit(&q[..n]);
            }
            _ => {
                let px: [u32; 256] = core::array::from_fn(|i| u32::from_le_bytes(raw[4 * i..4 * i + 4].try_into().unwrap()));
                let n = if out16 { csc::pack(&px, true, dither, &mut q) } else { csc::pack_indx4(&px, dither, &self.vq, &mut q) };
                self.emit(&q[..n]);
            }
        }
        self.left -= 1;
        Step::Go
    }

    fn begin_blocks(&mut self, pattern: u8) {
        let m = &mut self.mb;
        m.pattern = pattern;
        m.block = 0;
        m.at = In::DcSize;
        m.n = 0;
        m.qfs = [0; 64];
    }

    /// IDEC and BDEC: one symbol of the macroblock layer, or a block's.
    fn macroblock(&mut self) -> Step {
        let t = tables();
        let idec = self.job == Job::Idec;
        match self.phase {
            Phase::First => {
                self.phase = Phase::MbType;
                Step::Go
            }
            Phase::Next => {
                if self.input.available() < SYMBOL {
                    return Step::Wait;
                }
                if self.input.zeros(SYMBOL) >= 8 {
                    return self.start_code();
                }
                let table = if self.mpeg1() { &t.mbai_mpeg1 } else { &t.mbai };
                match table.decode(self.input.peek32()) {
                    Some((len, vlc::STUFFING)) => {
                        self.input.skip(u32::from(len));
                        Step::Go
                    }
                    Some((len, 1)) => {
                        self.input.skip(u32::from(len));
                        self.phase = Phase::MbType;
                        Step::Go
                    }
                    _ => self.error(),
                }
            }
            Phase::MbType => {
                if self.input.available() < SYMBOL {
                    return Step::Wait;
                }
                let table = &t.mb_type[if self.pct() == 4 { 3 } else { 0 }];
                let Some((len, v)) = table.decode(self.input.peek32()) else { return self.error() };
                self.input.skip(u32::from(len));
                self.mb.quant = v & mb::QUANT != 0;
                self.mb.intra = true;
                self.mb.field_dct = false;
                self.phase = Phase::Modes;
                Step::Go
            }
            Phase::Modes => {
                if self.input.available() < SYMBOL {
                    return Step::Wait;
                }
                if self.cmd & 1 << 24 != 0 {
                    self.mb.field_dct = self.input.take(1) == 1;
                }
                if self.mb.quant {
                    self.mb.qsc = self.input.take(5);
                }
                self.begin_blocks(0x3F);
                self.phase = Phase::Block;
                Step::Go
            }
            Phase::Cbp => {
                if self.input.available() < SYMBOL {
                    return Step::Wait;
                }
                let Some((len, cbp)) = t.cbp.decode(self.input.peek32()) else { return self.error() };
                self.input.skip(u32::from(len));
                self.ctrl = self.ctrl & !(0x3F << 8) | u32::from(cbp) << 8;
                self.begin_blocks(cbp);
                self.phase = Phase::Block;
                Step::Go
            }
            Phase::Block => {
                if self.mb.block == 6 {
                    self.phase = if idec && self.pct() == 4 { Phase::EndOfMacroblock } else { self.finish_macroblock() };
                    return Step::Go;
                }
                self.block()
            }
            Phase::EndOfMacroblock => {
                if self.input.available() < SYMBOL {
                    return Step::Wait;
                }
                if self.input.take(1) != 1 {
                    return self.error();
                }
                self.phase = self.finish_macroblock();
                Step::Go
            }
            Phase::After => {
                if self.input.available() < 8 {
                    return Step::Wait;
                }
                if self.input.zeros(8) < 8 { Step::End } else { self.start_code() }
            }
        }
    }

    /// The macroblock's output, and what follows it: IDEC's next macroblock, BDEC's look at the bits after.
    fn finish_macroblock(&mut self) -> Phase {
        let mut q = [0u128; 64];
        if self.job == Job::Idec {
            let mut raw = [0u8; 384];
            for y in 0..16 {
                for x in 0..16 {
                    raw[16 * y + x] = self.luma(x, y).clamp(0, 255) as u8;
                }
            }
            for i in 0..64 {
                raw[256 + i] = self.mb.out[4][i].clamp(0, 255) as u8;
                raw[320 + i] = self.mb.out[5][i].clamp(0, 255) as u8;
            }
            let mut px = [0u32; 256];
            csc::convert_raw8(&raw, self.th.0, self.th.1, &mut px);
            if self.cmd & 1 << 25 != 0 {
                for p in &mut px {
                    *p = csc::signed(*p);
                }
            }
            let out16 = self.cmd & 1 << 27 != 0;
            let n = csc::pack(&px, out16, out16 && self.cmd & 1 << 26 != 0, &mut q);
            self.emit(&q[..n]);
            Phase::Next
        } else {
            let mut s = [0i16; 384];
            for y in 0..16 {
                for x in 0..16 {
                    s[16 * y + x] = self.luma(x, y);
                }
            }
            s[256..320].copy_from_slice(&self.mb.out[4]);
            s[320..].copy_from_slice(&self.mb.out[5]);
            for (k, w) in q.iter_mut().take(48).enumerate() {
                *w = (0..8).rev().fold(0u128, |a, j| a << 16 | u128::from(s[8 * k + j] as u16));
            }
            self.emit(&q[..48]);
            Phase::After
        }
    }

    /// The luminance sample at (x, y) of the macroblock: from the frame DCT's blocks, or the field DCT's, whose upper
    /// two hold the top field's lines (H.262 Figures 6-13 and 6-14).
    fn luma(&self, x: usize, y: usize) -> i16 {
        let (b, row) = if self.mb.field_dct { (2 * (y & 1) + x / 8, y / 2) } else { (2 * (y / 8) + x / 8, y % 8) };
        self.mb.out[b][8 * row + x % 8]
    }

    /// One symbol of block `mb.block` (H.262 §7.2), and the block's reconstruction at its End of Block.
    fn block(&mut self) -> Step {
        let i = self.mb.block;
        if self.mb.pattern >> (5 - i) & 1 == 0 {
            self.mb.out[i] = [0; 64];
            self.next_block();
            return Step::Go;
        }
        if self.input.available() < SYMBOL {
            return Step::Wait;
        }
        let t = tables();
        let cc = if i < 4 { 0 } else { i - 3 };
        if self.mb.at == In::DcSize {
            if !self.mb.intra {
                self.mb.at = In::Coef;
                return Step::Go;
            }
            let bits = self.input.peek32();
            let Some((len, size)) = t.dc_size[usize::from(cc > 0)].decode(bits) else { return self.error() };
            let size = u32::from(size);
            let diff = if size == 0 {
                0
            } else {
                let d = (bits << len >> (32 - size)) as i32;
                if d >= 1 << (size - 1) { d } else { d + 1 - (1 << size) }
            };
            self.input.skip(u32::from(len) + size);
            self.dc_pred[cc] += diff;
            self.mb.qfs[0] = self.dc_pred[cc];
            self.mb.n = 1;
            if self.job == Job::Idec && self.pct() == 4 {
                self.reconstruct();
                return Step::Go;
            }
            self.mb.at = In::Coef;
            return Step::Go;
        }
        let bits = self.input.peek32();
        let table = if self.mb.intra && self.ctrl & ctrl::IVF != 0 {
            &t.dct_one
        } else if !self.mb.intra && self.mb.n == 0 {
            &t.dct_first
        } else {
            &t.dct_zero
        };
        let Some((len, c)) = table.decode(bits) else { return self.error() };
        let (run, level, used) = match c {
            Coef::Eob => {
                self.input.skip(u32::from(len));
                self.reconstruct();
                return Step::Go;
            }
            Coef::RunLevel(r, l) => {
                let s = bits << len >> 31;
                (u32::from(r), if s == 1 { -i32::from(l) } else { i32::from(l) }, u32::from(len) + 1)
            }
            Coef::Escape => {
                let rest = bits << len;
                let run = rest >> 26;
                if self.mpeg1() {
                    let first = (rest << 6 >> 24) as i32;
                    match first {
                        0 => (run, (rest << 14 >> 24) as i32, u32::from(len) + 22),
                        0x80 => (run, (rest << 14 >> 24) as i32 - 256, u32::from(len) + 22),
                        f => (run, f as i8 as i32, u32::from(len) + 14),
                    }
                } else {
                    (run, ((rest << 6) as i32) >> 20, u32::from(len) + 18)
                }
            }
        };
        let n = self.mb.n + run as usize;
        if n > 63 {
            return self.error();
        }
        self.input.skip(used);
        self.mb.qfs[n] = level;
        self.mb.n = n + 1;
        Step::Go
    }

    fn next_block(&mut self) {
        let m = &mut self.mb;
        m.block += 1;
        m.at = In::DcSize;
        m.n = 0;
        m.qfs = [0; 64];
    }

    /// The block's inverse scan, inverse quantisation, saturation and mismatch control (H.262 §7.3, §7.4; MPEG-1's
    /// oddification with MP1, §D.9.1, D-307) and IDCT (Annex A).
    fn reconstruct(&mut self) {
        let alt = usize::from(self.ctrl & ctrl::AS != 0);
        let mpeg1 = self.mpeg1();
        let intra = self.mb.intra;
        let w = &self.iq[usize::from(!intra)];
        let qs = if mpeg1 { 2 * self.mb.qsc as i32 } else { i32::from(Q_SCALE[usize::from(self.ctrl & ctrl::QST != 0)][self.mb.qsc as usize]) };
        let mut f = [0i32; 64];
        let mut sum = 0i32;
        for k in 0..64 {
            let qf = self.mb.qfs[usize::from(SCAN[alt][k])];
            let v = if k == 0 && intra {
                (8 >> (self.ctrl >> 16 & 3)) * qf
            } else {
                let k2 = if intra { 2 * qf } else { 2 * qf + qf.signum() };
                let mut v = k2 * i32::from(w[k]) * qs / 32;
                if mpeg1 && v & 1 == 0 {
                    v -= v.signum();
                }
                v
            };
            f[k] = v.clamp(-2048, 2047);
            sum += f[k];
        }
        if !mpeg1 && sum & 1 == 0 {
            f[63] ^= 1;
        }
        let i = self.mb.block;
        idct::idct(&f, &mut self.mb.out[i]);
        self.next_block();
    }
}
