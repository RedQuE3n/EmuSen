//! The project's S22 programs (Sapphire_Native.md §26): paired programs, one for the EE and one for the IOP, that pass
//! messages both ways through the SIF's registers and its three DMA channels and each check the other's record. The EE
//! program places the IOP program in IOP RAM through the EE's window at 1C000000h and points the IOP's exception vector
//! at it (S21's route, `s21::wrap`); the two then step together through a mailbox of two words in IOP RAM that the EE
//! reaches through the same window, so that neither the handshake nor the record depends on the unit under test. The
//! EE writes its record of 32-bit words from 00200000h, waits for the IOP's done word, copies the IOP's record after its
//! own and writes 5A99D00Eh at 001FFFF0h. Every expected word is derived here from ps2tek's "SIF Registers" and "SIF
//! DMA", its "IOP DMA" and the EE User's Manual's chapter 5, written apart from `src/sif/`.

#![allow(dead_code)]

use crate::s21;

/// The EE program's origin, its record and done word (EE physical addresses), and where it keeps the IOP program.
pub const EE_ORIGIN: u32 = 0x0010_0000;
pub const EE_RECORD: u32 = 0x0020_0000;
pub const EE_DONE_AT: u32 = 0x001F_FFF0;
pub const BLOB: u32 = 0x0014_0000;
/// EE memory the programs use for their buffers.
pub const EE_AREA: u32 = 0x0030_0000;
/// The mailbox in IOP RAM: the EE's step, the IOP's step.
pub const MB_EE: u32 = 0x0017_F000;
pub const MB_IOP: u32 = 0x0017_F004;
/// IOP memory the programs use for their buffers.
pub const IOP_AREA: u32 = 0x001C_0000;
pub const K1: u32 = 0xA000_0000;
/// The EE's window onto IOP RAM, through kseg1.
pub const WINDOW: u32 = 0xBC00_0000;
pub const DONE: u32 = s21::DONE;

/// The SIF's registers, by offset from each side's base (ps2tek "SIF Registers").
pub mod sif {
    pub const EE: u32 = 0x1000_F200;
    pub const IOP: u32 = 0xBD00_0000;
    pub const MSCOM: u32 = 0x00;
    pub const SMCOM: u32 = 0x10;
    pub const MSFLG: u32 = 0x20;
    pub const SMFLG: u32 = 0x30;
    pub const CTRL: u32 = 0x40;
    pub const BD6: u32 = 0x60;
}

/// The EE's registers the programs reach (EE User's Manual §2.2.1).
pub mod ee {
    pub const D_CTRL: u32 = 0x1000_E000;
    pub const D_STAT: u32 = 0x1000_E010;
    pub const D_PCR: u32 = 0x1000_E020;
    pub const D_STADR: u32 = 0x1000_E060;
    pub const D_ENABLEW: u32 = 0x1000_F590;
    pub const I_STAT: u32 = 0x1000_F000;
    pub const I_MASK: u32 = 0x1000_F010;
    pub const BASES: [u32; 10] =
        [0x1000_8000, 0x1000_9000, 0x1000_A000, 0x1000_B000, 0x1000_B400, 0x1000_C000, 0x1000_C400, 0x1000_C800, 0x1000_D000, 0x1000_D400];
    pub const SIF0: usize = 5;
    pub const SIF1: usize = 6;
    pub const SIF2: usize = 7;
    pub const fn chcr(c: usize) -> u32 {
        BASES[c]
    }
    pub const fn madr(c: usize) -> u32 {
        BASES[c] + 0x10
    }
    pub const fn qwc(c: usize) -> u32 {
        BASES[c] + 0x20
    }
    pub const fn tadr(c: usize) -> u32 {
        BASES[c] + 0x30
    }
    pub const STR: u32 = 0x100;
    pub const CHAIN: u32 = 1 << 2;
    pub const TTE: u32 = 1 << 6;
    pub const TIE: u32 = 1 << 7;
    pub const DIR: u32 = 1;
    pub const fn t_count(n: u32) -> u32 {
        0x1000_0000 + n * 0x800
    }
    pub const fn t_mode(n: u32) -> u32 {
        0x1000_0010 + n * 0x800
    }
    pub const fn t_hold(n: u32) -> u32 {
        0x1000_0030 + n * 0x800
    }
}

/// The IOP's SIF channels and their CHCR fields (ps2tek "IOP DMA").
pub mod iop {
    pub const SIF2: u32 = 2;
    pub const SIF0: u32 = 9;
    pub const SIF1: u32 = 10;
    pub const DIR: u32 = 1;
    pub const TAG4: u32 = 1 << 8;
    pub const SLICE: u32 = 1 << 9;
    pub const CHAIN: u32 = 3 << 9;
    pub const STR: u32 = 1 << 24;
}

/// An EE program in the R5900's dialect, in S7's form: entered at `EE_ORIGIN`, run from kseg0 after its first jump,
/// registers, buffers and records reached through kseg1; its record is of 32-bit words.
pub struct Ee {
    pub name: &'static str,
    lines: Vec<String>,
    count: u32,
    labels: u32,
    pub expected: Vec<Option<u32>>,
}

impl Ee {
    pub fn new(name: &'static str) -> Ee {
        let mut p = Ee { name, lines: Vec::new(), count: 0, labels: 0, expected: Vec::new() };
        p.emit("lui s1, 0xa020");
        let target = (p.here() + 16) | 0x8000_0000;
        p.li("t9", target);
        p.emit("jr t9");
        p.emit("nop");
        p
    }

    pub fn emit(&mut self, line: impl Into<String>) {
        self.lines.push(line.into());
        self.count += 1;
    }

    pub fn here(&self) -> u32 {
        EE_ORIGIN + 4 * self.count
    }

    pub fn label(&mut self) -> String {
        self.labels += 1;
        format!("e{}", self.labels)
    }

    pub fn place(&mut self, label: &str) {
        self.lines.push(format!("{label}:"));
    }

    pub fn li(&mut self, reg: &str, v: u32) {
        self.emit(format!("lui {reg}, 0x{:x}", v >> 16));
        self.emit(format!("ori {reg}, {reg}, 0x{:x}", v & 0xFFFF));
    }

    /// Stores `reg`'s low word at s1 and moves s1 on, expecting `v` (none: any value).
    pub fn record(&mut self, reg: &str, v: Option<u32>) {
        self.emit(format!("sw {reg}, 0(s1)"));
        self.emit("addiu s1, s1, 4");
        self.expected.push(v);
    }

    /// The word at virtual `va` written with `v`.
    pub fn store(&mut self, va: u32, v: u32) {
        self.li("t6", va);
        self.li("t7", v);
        self.emit("sw t7, 0(t6)");
    }

    /// The word at virtual `va` read into t2.
    pub fn load(&mut self, va: u32) {
        self.li("t6", va);
        self.emit("lw t2, 0(t6)");
        self.emit("nop");
    }

    /// A register or memory word at physical `pa`, through kseg1.
    pub fn poke(&mut self, pa: u32, v: u32) {
        self.store(K1 | pa, v);
    }

    pub fn peek(&mut self, pa: u32) {
        self.load(K1 | pa);
    }

    pub fn rec(&mut self, pa: u32, want: Option<u32>) {
        self.peek(pa);
        self.record("t2", want);
    }

    pub fn rec_masked(&mut self, pa: u32, mask: u32, want: u32) {
        self.peek(pa);
        self.li("t7", mask);
        self.emit("and t2, t2, t7");
        self.record("t2", Some(want));
    }

    /// An IOP RAM word at physical `a`, through the EE's window.
    pub fn iop_poke(&mut self, a: u32, v: u32) {
        self.store(WINDOW | a, v);
    }

    pub fn iop_rec(&mut self, a: u32, want: Option<u32>) {
        self.load(WINDOW | a);
        self.record("t2", want);
    }

    /// Waits, at most `limit` passes, until the word at virtual `va` ANDed with `mask` is non-zero (`set`) or zero.
    pub fn wait_va(&mut self, va: u32, mask: u32, set: bool, limit: u32) {
        let l = self.label();
        let out = self.label();
        self.li("t6", va);
        self.li("t4", mask);
        self.li("t5", limit);
        self.place(&l);
        self.emit("lw t2, 0(t6)");
        self.emit("nop");
        self.emit("and t2, t2, t4");
        self.emit(format!("{} t2, zero, {out}", if set { "bne" } else { "beq" }));
        self.emit("nop");
        self.emit("addiu t5, t5, -1");
        self.emit(format!("bne t5, zero, {l}"));
        self.emit("nop");
        self.place(&out);
    }

    pub fn wait(&mut self, pa: u32, mask: u32, set: bool, limit: u32) {
        self.wait_va(K1 | pa, mask, set, limit);
    }

    /// A delay of `n` passes of a two-instruction loop.
    pub fn delay(&mut self, n: u32) {
        let l = self.label();
        self.li("t5", n);
        self.place(&l);
        self.emit("addiu t5, t5, -1");
        self.emit(format!("bne t5, zero, {l}"));
        self.emit("nop");
    }

    /// Tells the IOP the EE has reached step `n`.
    pub fn step(&mut self, n: u32) {
        self.emit("sync");
        self.iop_poke(MB_EE, n);
    }

    /// Waits, a bounded time, until the IOP has reached step `n`.
    pub fn until(&mut self, n: u32) {
        let l = self.label();
        let out = self.label();
        self.li("t6", WINDOW | MB_IOP);
        self.li("t4", n);
        self.li("t5", 20_000_000);
        self.place(&l);
        self.emit("lw t2, 0(t6)");
        self.emit("nop");
        self.emit(format!("beq t2, t4, {out}"));
        self.emit("nop");
        self.emit("addiu t5, t5, -1");
        self.emit(format!("bne t5, zero, {l}"));
        self.emit("nop");
        self.place(&out);
    }

    /// Interrupts off, Status cleared, the DMAC enabled with every channel stopped and nothing pending, the INTC's
    /// mask and status cleared, the timers stopped; whatever a BIOS left behind on a referee is undone.
    pub fn prologue(&mut self) {
        self.emit("di");
        self.emit("mtc0 zero, Status");
        self.emit("sync.p");
        self.poke(ee::D_CTRL, 0);
        for c in 0..10 {
            self.poke(ee::chcr(c), 0);
        }
        self.poke(ee::D_PCR, 0);
        self.poke(ee::D_ENABLEW, 0);
        self.clear_dstat();
        self.peek(ee::I_MASK);
        self.emit("sw t2, 0(t6)");
        self.poke(ee::I_STAT, 0x7FFF);
        for n in 0..4 {
            self.poke(ee::t_mode(n), 0xC00);
        }
        self.poke(ee::D_CTRL, 1);
    }

    /// D_STAT's flags cleared and its masks reversed to zero.
    pub fn clear_dstat(&mut self) {
        self.peek(ee::D_STAT);
        self.li("t7", 0x63FF_0000);
        self.emit("and t2, t2, t7");
        self.emit("ori t2, t2, 0xe3ff");
        self.emit("sw t2, 0(t6)");
    }

    /// Copies the IOP program of `words` words from `BLOB` into IOP RAM at `s21::ORIGIN`, clears the IOP's done word,
    /// its handler's flag and the mailbox, and writes `s21::vector` at the IOP's general exception vector, last.
    pub fn place_iop(&mut self, words: usize) {
        for a in [s21::DONE_AT, s21::FLAG, MB_EE, MB_IOP] {
            self.iop_poke(a, 0);
        }
        self.li("t0", WINDOW | s21::ORIGIN);
        self.li("t1", K1 | BLOB);
        self.li("t2", words as u32);
        let l = self.label();
        self.place(&l);
        self.emit("lw t3, 0(t1)");
        self.emit("sw t3, 0(t0)");
        self.emit("addiu t1, t1, 4");
        self.emit("addiu t0, t0, 4");
        self.emit("addiu t2, t2, -1");
        self.emit(format!("bne t2, zero, {l}"));
        self.emit("nop");
        self.emit("sync");
        for (i, w) in s21::vector().iter().enumerate() {
            self.iop_poke(0x80 + 4 * i as u32, *w);
        }
        self.emit("sync");
    }

    /// Waits for the IOP's done word, records it, copies the IOP's record of `n` words after the EE's own, and
    /// writes the EE's done word.
    pub fn finish(mut self, iop: &[Option<u32>]) -> (String, Vec<Option<u32>>) {
        let l = self.label();
        let out = self.label();
        self.li("t0", WINDOW | s21::DONE_AT);
        self.li("t4", DONE);
        self.li("t5", 100_000_000);
        self.place(&l);
        self.emit("lw t3, 0(t0)");
        self.emit(format!("beq t3, t4, {out}"));
        self.emit("nop");
        self.emit("addiu t5, t5, -1");
        self.emit(format!("bne t5, zero, {l}"));
        self.emit("nop");
        self.place(&out);
        self.record("t3", Some(DONE));
        if !iop.is_empty() {
            let c = self.label();
            self.li("t0", WINDOW | s21::RECORD);
            self.li("t2", iop.len() as u32);
            self.place(&c);
            self.emit("lw t3, 0(t0)");
            self.emit("sw t3, 0(s1)");
            self.emit("addiu t0, t0, 4");
            self.emit("addiu s1, s1, 4");
            self.emit("addiu t2, t2, -1");
            self.emit(format!("bne t2, zero, {c}"));
            self.emit("nop");
            self.expected.extend_from_slice(iop);
        }
        self.li("t0", DONE);
        self.li("t1", K1 | EE_DONE_AT);
        self.emit("sw t0, 0(t1)");
        self.place("end");
        self.emit("beq zero, zero, end");
        self.emit("nop");
        let mut text = self.lines.join("\n");
        text.push('\n');
        (text, self.expected)
    }
}

/// The IOP's side of a pair: S21's program form, with the mailbox.
pub trait IopSide {
    fn step(&mut self, n: u32);
    fn until(&mut self, n: u32);
}

impl IopSide for s21::Program {
    /// Tells the EE the IOP has reached step `n`.
    fn step(&mut self, n: u32) {
        self.poke(K1 | MB_IOP, n);
    }

    /// Waits, a bounded time, until the EE has reached step `n`.
    fn until(&mut self, n: u32) {
        let l = self.label();
        let out = self.label();
        self.li("t6", K1 | MB_EE);
        self.li("t4", n);
        self.li("t5", 5_000_000);
        self.place(&l);
        self.emit("lw t2, 0(t6)");
        self.emit("nop");
        self.emit(format!("beq t2, t4, {out}"));
        self.emit("nop");
        self.emit("addiu t5, t5, -1");
        self.emit(format!("bne t5, zero, {l}"));
        self.emit("nop");
        self.place(&out);
    }
}

/// A finished pair: the EE program's image (its code, then the IOP program at `BLOB`), and the whole record expected,
/// the EE's words, the IOP's done word and then the IOP's words.
pub struct Pair {
    pub name: &'static str,
    pub image: Vec<u32>,
    pub expected: Vec<Option<u32>>,
    pub ee_words: usize,
}

/// Builds a pair from its two halves; `ee` must have placed the IOP program (`Ee::place_iop`) with `iop`'s length,
/// which `pair` checks by assembling the IOP program first.
pub fn pair(name: &'static str, build: impl Fn(usize) -> (Ee, s21::Program)) -> Pair {
    let (probe_ee, probe_iop) = build(0);
    drop(probe_ee);
    let (src, _) = probe_iop.finish();
    let n = s21::assemble(&src).len();
    let (ee, iop) = build(n);
    let (isrc, iexp) = iop.finish();
    let iwords = s21::assemble(&isrc);
    assert_eq!(iwords.len(), n, "{name}: the IOP program's length is stable");
    let ee_words = ee.expected.len();
    let (esrc, expected) = ee.finish(&iexp);
    let mut image = emusen_mips::assemble(&esrc, EE_ORIGIN, &emusen_mips::Dialect::R5900).unwrap_or_else(|e| panic!("{name}: {e:?}"));
    let blob = ((BLOB - EE_ORIGIN) / 4) as usize;
    assert!(image.len() <= blob, "{name}: the EE program reaches the IOP program's place");
    image.resize(blob, 0);
    image.extend_from_slice(&iwords);
    Pair { name, image, expected, ee_words }
}

/// The IOP's side, begun: S21's program, its quiescing stopping every channel the SIF's among them.
pub fn iop_side(name: &'static str) -> s21::Program {
    s21::Program::new(name)
}

/// The EE's side, begun: the prologue and the IOP program placed.
pub fn ee_side(name: &'static str, iop_words: usize) -> Ee {
    let mut e = Ee::new(name);
    e.prologue();
    e.place_iop(iop_words);
    e
}

/// The mailbox's data words in IOP RAM, which the IOP fills for the EE to copy into its own record at once.
pub const MB_DATA: u32 = 0x0017_F010;

/// The IOP's side: the word in t2 written to the mailbox's data word `k`.
fn to_mailbox(i: &mut s21::Program, k: u32) {
    i.li("t6", K1 | (MB_DATA + 4 * k));
    i.emit("sw t2, 0(t6)");
}

/// `s22-probe-iop`: what the documents leave open about the registers, recorded and not graded, for the disputes:
/// each side's view of the six registers at the start; each bit of CTRL written by the IOP, with the IOP's CTRL and
/// the EE's I_STAT (SBUS is bit 1), timer 0's HOLD and CTRL after it; SMFLG and SMCOM written by the IOP with the EE's
/// I_STAT after each; BD6 written by each side and read by the other.
pub fn probe_iop() -> Pair {
    pair("s22-probe-iop", |n| {
        let mut e = ee_side("s22-probe-iop", n);
        let mut i = iop_side("s22-probe-iop");
        for (k, r) in [sif::MSCOM, sif::SMCOM, sif::MSFLG, sif::SMFLG, sif::CTRL, sif::BD6].into_iter().enumerate() {
            e.rec(sif::EE + r, None);
            i.peek(sif::IOP + r);
            to_mailbox(&mut i, k as u32);
        }
        i.step(1);
        e.until(1);
        for k in 0..6 {
            e.iop_rec(MB_DATA + 4 * k, None);
        }
        e.poke(ee::t_mode(0), 0x80);
        e.poke(ee::I_STAT, 0x7FFF);
        e.step(1);
        for b in 0..32 {
            i.until(1 + b);
            i.poke(sif::IOP + sif::CTRL, 1 << b);
            i.peek(sif::IOP + sif::CTRL);
            to_mailbox(&mut i, 0);
            i.step(2 + b);
            e.until(2 + b);
            e.iop_rec(MB_DATA, None);
            e.rec(ee::I_STAT, None);
            e.rec(ee::t_hold(0), None);
            e.rec(sif::EE + sif::CTRL, None);
            e.poke(ee::I_STAT, 0x7FFF);
            e.step(2 + b);
        }
        i.until(33);
        i.poke(sif::IOP + sif::SMFLG, 0x100);
        i.step(40);
        e.until(40);
        e.rec(ee::I_STAT, None);
        e.rec(sif::EE + sif::SMFLG, None);
        e.poke(ee::I_STAT, 0x7FFF);
        e.step(40);
        i.until(40);
        i.poke(sif::IOP + sif::SMCOM, 0x1234_5678);
        i.step(41);
        e.until(41);
        e.rec(ee::I_STAT, None);
        e.rec(sif::EE + sif::SMCOM, None);
        e.poke(sif::EE + sif::BD6, 0xA5A5_0F0F);
        e.rec(sif::EE + sif::BD6, None);
        e.step(41);
        i.until(41);
        i.rec(sif::IOP + sif::BD6, None);
        i.poke(sif::IOP + sif::BD6, 0x3C3C_5A5A);
        i.rec(sif::IOP + sif::BD6, None);
        for r in [sif::MSCOM, sif::SMCOM, sif::MSFLG, sif::SMFLG, sif::CTRL] {
            i.rec(sif::IOP + r, None);
        }
        i.step(42);
        e.until(42);
        e.rec(sif::EE + sif::BD6, None);
        for r in [sif::MSCOM, sif::SMCOM, sif::MSFLG, sif::SMFLG, sif::CTRL] {
            e.rec(sif::EE + r, None);
        }
        (e, i)
    })
}

/// `s22-probe-ee`: each bit of CTRL written by the EE, from bit 0 up, with the EE's CTRL after it and the IOP's
/// I_STAT, CTRL and step as the IOP answers; a step the IOP misses leaves the mailbox's marks in place.
pub fn probe_ee() -> Pair {
    pair("s22-probe-ee", |n| {
        let mut e = ee_side("s22-probe-ee", n);
        let mut i = iop_side("s22-probe-ee");
        i.poke(s21::reg::I_STAT, 0);
        i.step(1);
        for b in 0..32 {
            e.until(1 + b);
            e.iop_poke(MB_DATA, 0xDEAD_0000 | b);
            e.iop_poke(MB_DATA + 4, 0xDEAD_0000 | b);
            e.poke(sif::EE + sif::CTRL, 1 << b);
            e.rec(sif::EE + sif::CTRL, None);
            e.step(1 + b);
            i.until(1 + b);
            i.peek(s21::reg::I_STAT);
            to_mailbox(&mut i, 0);
            i.peek(sif::IOP + sif::CTRL);
            to_mailbox(&mut i, 1);
            i.poke(s21::reg::I_STAT, 0);
            i.step(2 + b);
            e.until(2 + b);
            e.iop_rec(MB_DATA, None);
            e.iop_rec(MB_DATA + 4, None);
            e.iop_rec(MB_IOP, None);
        }
        (e, i)
    })
}


/// The wrapping sum of words, as the programs' summing loops compute it.
pub fn sum(words: impl IntoIterator<Item = u32>) -> u32 {
    words.into_iter().fold(0u32, u32::wrapping_add)
}

impl Ee {
    /// Words written to EE memory at physical `pa`.
    pub fn fill(&mut self, pa: u32, words: &[u32]) {
        self.li("t6", K1 | pa);
        for (k, w) in words.iter().enumerate() {
            self.li("t7", *w);
            self.emit(format!("sw t7, {}(t6)", 4 * k));
        }
    }

    /// Records the words at physical `pa`, expecting `want`.
    pub fn rec_words(&mut self, pa: u32, want: &[u32]) {
        self.li("t6", K1 | pa);
        for (k, w) in want.iter().enumerate() {
            self.emit(format!("lw t2, {}(t6)", 4 * k));
            self.record("t2", Some(*w));
        }
    }

    /// Records the wrapping sum of `n` words at physical `pa`, expecting `want`.
    pub fn rec_sum(&mut self, pa: u32, n: u32, want: u32) {
        let l = self.label();
        self.li("t0", K1 | pa);
        self.li("t2", n);
        self.emit("or t3, zero, zero");
        self.place(&l);
        self.emit("lw t1, 0(t0)");
        self.emit("addu t3, t3, t1");
        self.emit("addiu t0, t0, 4");
        self.emit("addiu t2, t2, -1");
        self.emit(format!("bne t2, zero, {l}"));
        self.emit("nop");
        self.record("t3", Some(want));
    }

    /// Sets STR in channel `c`'s CHCR as it reads, keeping its other bits, as a chain stopped by a tag is resumed.
    pub fn restart(&mut self, c: usize) {
        self.peek(ee::chcr(c));
        self.emit("ori t2, t2, 0x100");
        self.emit("sw t2, 0(t6)");
    }
}

/// The IOP's side: words written to IOP memory at physical `a`.
pub fn iop_fill(i: &mut s21::Program, a: u32, words: &[u32]) {
    i.li("t6", K1 | a);
    for (k, w) in words.iter().enumerate() {
        i.li("t7", *w);
        i.emit(format!("sw t7, {}(t6)", 4 * k));
    }
}

/// The IOP's side: records the words at physical `a`, expecting `want`.
pub fn iop_rec_words(i: &mut s21::Program, a: u32, want: &[u32]) {
    i.li("t6", K1 | a);
    for (k, w) in want.iter().enumerate() {
        i.emit(format!("lw t2, {}(t6)", 4 * k));
        i.emit("nop");
        i.record("t2", Some(*w));
    }
}

/// The IOP's side: records the wrapping sum of `n` words at physical `a`, expecting `want`.
pub fn iop_rec_sum(i: &mut s21::Program, a: u32, n: u32, want: u32) {
    let l = i.label();
    i.li("t0", K1 | a);
    i.li("t2", n);
    i.emit("or t3, zero, zero");
    i.place(&l);
    i.emit("lw t1, 0(t0)");
    i.emit("nop");
    i.emit("addu t3, t3, t1");
    i.emit("addiu t0, t0, 4");
    i.emit("addiu t2, t2, -1");
    i.emit(format!("bne t2, zero, {l}"));
    i.emit("nop");
    i.record("t3", Some(want));
}

/// Both sides meet at point `n`: the IOP says it is there, the EE waits for it and says so, the IOP waits for that.
pub fn meet(e: &mut Ee, i: &mut s21::Program, n: u32) {
    i.step(n);
    e.until(n);
    e.step(n);
    i.until(n);
}

/// The IOP's SIF channels enabled (DPCR's channel 2, DPCR2's 9 and 10), the master interrupt enable set, the flags
/// of DICR2 cleared and I_STAT acknowledged.
pub fn iop_sif_setup(i: &mut s21::Program) {
    use s21::reg::*;
    i.poke(DPCR, 0x0000_0800);
    i.poke(DPCR2, 0x0000_8800);
    i.poke(DICR, 0x0080_0000);
    i.poke(DICR2, 0x3F00_0000);
    i.poke(I_STAT, 0);
}

const CC: u32 = 0xCCCC_CCCC;

/// A pattern of `n` words, `base + k`.
pub fn pattern(base: u32, n: u32) -> Vec<u32> {
    (0..n).map(|k| base + k).collect()
}

/// `n` words of CCCCCCCCh, the mark of memory no transfer reached.
pub fn marks(n: usize) -> Vec<u32> {
    vec![CC; n]
}

fn cat(parts: &[&[u32]]) -> Vec<u32> {
    parts.concat()
}

/// An EE source chain tag's quadword: QWC, ID and IRQ in the first word, ADDR in the second (EE User's Manual §5.6).
pub fn ee_tag(id: u32, qwc: u32, addr: u32, irq: bool) -> [u32; 4] {
    [qwc | id << 28 | u32::from(irq) << 31, addr, 0, 0]
}

/// The EE's tag IDs used here (§5.6): refe 0, cnt 1, ref 3, end 7.
pub mod id {
    pub const CNT: u32 = 1;
    pub const REF: u32 = 3;
    pub const END: u32 = 7;
}

/// An IOP chain tag's first word: the address, IRQ (bit 30) and the end (bit 31) (ps2tek "IOP DMA").
pub fn iop_tag(addr: u32, irq: bool, end: bool) -> u32 {
    addr | u32::from(irq) << 30 | u32::from(end) << 31
}

/// `s22-registers`: MSCOM written by the EE and SMCOM by the IOP, each read by both and each other side's write
/// ignored; MSFLG set by the EE and cleared by the IOP, SMFLG the reverse, a write of 0 changing neither; CTRL's
/// documented bits on each side; the EE's CTRL write with bit 18 raising the IOP's interrupt 1, taken by the IOP, and one
/// without it raising nothing (ps2tek "SIF Registers"; Sapphire_Disputes.md D-141).
pub fn registers() -> Pair {
    use sif::*;
    pair("s22-registers", |n| {
        let mut e = ee_side("s22-registers", n);
        let mut i = iop_side("s22-registers");
        i.poke(IOP + MSFLG, 0xFFFF_FFFF);
        i.poke(IOP + SMCOM, 0x0F0F_0F0F);
        i.poke(s21::reg::I_STAT, 0);
        meet(&mut e, &mut i, 1);
        e.poke(EE + SMFLG, 0xFFFF_FFFF);
        e.rec(EE + MSFLG, Some(0));
        e.rec(EE + SMFLG, Some(0));
        e.rec(EE + SMCOM, Some(0x0F0F_0F0F));
        e.poke(EE + MSCOM, 0x1357_9BDF);
        e.poke(EE + SMCOM, 0x0BAD_0BAD);
        e.poke(EE + MSFLG, 0x0002_0000);
        e.poke(EE + MSFLG, 0x0000_0101);
        e.rec(EE + MSFLG, Some(0x0002_0101));
        e.rec(EE + MSCOM, Some(0x1357_9BDF));
        e.rec(EE + SMCOM, Some(0x0F0F_0F0F));
        meet(&mut e, &mut i, 2);
        i.rec(IOP + MSCOM, Some(0x1357_9BDF));
        i.rec(IOP + SMCOM, Some(0x0F0F_0F0F));
        i.rec(IOP + SMFLG, Some(0));
        i.rec(IOP + MSFLG, Some(0x0002_0101));
        i.poke(IOP + MSCOM, 0xFFFF_FFFF);
        i.rec(IOP + MSCOM, Some(0x1357_9BDF));
        i.poke(IOP + MSFLG, 0x0000_0100);
        i.rec(IOP + MSFLG, Some(0x0002_0001));
        i.poke(IOP + MSFLG, 0);
        i.rec(IOP + MSFLG, Some(0x0002_0001));
        i.poke(IOP + SMFLG, 0x0004_0000);
        i.poke(IOP + SMFLG, 0x0000_0010);
        i.rec(IOP + SMFLG, Some(0x0004_0010));
        i.poke(IOP + SMCOM, 0x2468_ACE0);
        i.rec(IOP + SMCOM, Some(0x2468_ACE0));
        meet(&mut e, &mut i, 3);
        e.rec(EE + SMCOM, Some(0x2468_ACE0));
        e.rec(EE + SMFLG, Some(0x0004_0010));
        e.rec(EE + MSFLG, Some(0x0002_0001));
        e.poke(EE + SMFLG, 0x10);
        e.rec(EE + SMFLG, Some(0x0004_0000));
        e.poke(EE + SMFLG, 0);
        e.rec(EE + SMFLG, Some(0x0004_0000));
        e.rec_masked(EE + CTRL, 0xF000_0102, 0xF000_0102);
        e.poke(EE + CTRL, 0x100);
        meet(&mut e, &mut i, 4);
        i.rec(IOP + SMFLG, Some(0x0004_0000));
        i.rec_masked(IOP + CTRL, 0xF000_0102, 0xF000_0002);
        i.rec_masked(s21::reg::I_STAT, 2, 0);
        meet(&mut e, &mut i, 5);
        e.poke(EE + CTRL, 0x0004_0000);
        meet(&mut e, &mut i, 6);
        i.rec_masked(s21::reg::I_STAT, 2, 2);
        i.guarded(|i| {
            i.poke(s21::reg::I_MASK, 2);
            i.poke(s21::reg::I_CTRL, 1);
            i.li("t0", 0x401);
            i.emit("mtc0 t0, SR");
            i.delay(200);
        });
        i.rec_log(1, Some(0), None);
        i.emit("mtc0 zero, SR");
        i.poke(s21::reg::I_MASK, 0);
        i.poke(s21::reg::I_STAT, 0);
        i.poke(s21::reg::I_CTRL, 0);
        meet(&mut e, &mut i, 7);
        (e, i)
    })
}

/// `s22-sif1`, the EE to the IOP: a normal-mode transfer, its first quadword the IOP's tag, read by an IOP chain
/// without CHCR's bit 8; an EE chain stopped by a tag interrupt and resumed while the IOP waits for the rest; the EE
/// started first, its channel holding with eight quadwords in the FIFO until the IOP starts; and, last, an EE source
/// chain (cnt, ref, end) read by an IOP chain with bit 8, the middle tag with IRQ setting DICR2's flag and the master
/// flag's interrupt 3 and the chain going on (ps2tek "SIF DMA", "IOP DMA"; EE User's Manual §5.6; D-140, D-143, D-144).
pub fn sif1() -> Pair {
    use ee::*;
    use s21::reg::{DICR, DICR2, chcr as ichcr, madr as imadr};
    pair("s22-sif1", |n| {
        let mut e = ee_side("s22-sif1", n);
        let mut i = iop_side("s22-sif1");
        iop_sif_setup(&mut i);
        let (d0, d1, d2, d3, d4, d5, d6) =
            (IOP_AREA, IOP_AREA + 0x100, IOP_AREA + 0x200, IOP_AREA + 0x300, IOP_AREA + 0x1000, IOP_AREA + 0x400, IOP_AREA + 0x500);
        let pad = [0xEEEE_0000, 0xEEEE_0001];
        // A normal-mode transfer: the IOP's tag in the first quadword, eight words in two more.
        let e0 = EE_AREA;
        let a = pattern(0x5100_0000, 8);
        e.fill(e0, &cat(&[&[iop_tag(d0, false, true), 8], &pad, &a]));
        iop_fill(&mut i, d0, &marks(9));
        i.poke(ichcr(iop::SIF1), iop::STR | iop::CHAIN);
        meet(&mut e, &mut i, 1);
        e.poke(madr(SIF1), e0);
        e.poke(qwc(SIF1), 3);
        e.poke(chcr(SIF1), STR | DIR);
        e.wait(chcr(SIF1), STR, false, 200_000);
        e.rec(qwc(SIF1), Some(0));
        e.rec(madr(SIF1), Some(e0 + 48));
        e.rec(chcr(SIF1), Some(DIR));
        e.rec_masked(D_STAT, 0x40, 0x40);
        e.poke(D_STAT, 0x40);
        meet(&mut e, &mut i, 2);
        i.wait(ichcr(iop::SIF1), iop::STR, false, 100_000);
        i.rec(ichcr(iop::SIF1), Some(iop::CHAIN));
        i.rec(imadr(iop::SIF1), Some(d0 + 32));
        iop_rec_words(&mut i, d0, &cat(&[&a, &[CC]]));
        // An EE chain stopped by its tag interrupt after the first packet, and resumed.
        let t2 = EE_AREA + 0x300;
        let (b5, b6) = (pattern(0x5500_0000, 4), pattern(0x5600_0000, 4));
        e.fill(t2, &cat(&[&ee_tag(id::CNT, 2, 0, true), &[d5, 4], &pad, &b5, &ee_tag(id::END, 2, 0, false), &[iop_tag(d6, false, true), 4], &pad, &b6]));
        iop_fill(&mut i, d5, &marks(5));
        iop_fill(&mut i, d6, &marks(5));
        i.poke(ichcr(iop::SIF1), iop::STR | iop::CHAIN | iop::TAG4);
        meet(&mut e, &mut i, 3);
        e.poke(tadr(SIF1), t2);
        e.poke(qwc(SIF1), 0);
        e.poke(chcr(SIF1), STR | TIE | CHAIN | DIR);
        e.wait(chcr(SIF1), STR, false, 200_000);
        e.rec(tadr(SIF1), Some(t2 + 0x30));
        e.rec(qwc(SIF1), Some(0));
        e.rec(chcr(SIF1), Some(0x9000_0000 | TIE | CHAIN | DIR));
        e.rec_masked(D_STAT, 0x40, 0x40);
        e.poke(D_STAT, 0x40);
        meet(&mut e, &mut i, 4);
        i.delay(2_000);
        iop_rec_words(&mut i, d5, &cat(&[&b5, &[CC]]));
        i.rec_masked(ichcr(iop::SIF1), iop::STR, iop::STR);
        i.rec(imadr(iop::SIF1), Some(d5 + 16));
        meet(&mut e, &mut i, 5);
        e.restart(SIF1);
        e.wait(chcr(SIF1), STR, false, 200_000);
        e.rec(tadr(SIF1), Some(t2 + 0x30));
        e.rec(chcr(SIF1), Some(0x7000_0000 | TIE | CHAIN | DIR));
        e.rec_masked(D_STAT, 0x40, 0x40);
        e.poke(D_STAT, 0x40);
        meet(&mut e, &mut i, 6);
        i.wait(ichcr(iop::SIF1), iop::STR, false, 100_000);
        i.rec(ichcr(iop::SIF1), Some(iop::CHAIN | iop::TAG4));
        iop_rec_words(&mut i, d6, &cat(&[&b6, &[CC]]));
        i.rec(imadr(iop::SIF1), Some(d6 + 16));
        // The EE first: sixteen quadwords, of which the FIFO holds eight until the IOP's channel starts.
        let e4 = EE_AREA + 0x1000;
        let b4 = pattern(0x5700_0000, 60);
        e.fill(e4, &cat(&[&[iop_tag(d4, false, true), 60], &pad, &b4]));
        meet(&mut e, &mut i, 7);
        e.poke(madr(SIF1), e4);
        e.poke(qwc(SIF1), 16);
        e.poke(chcr(SIF1), STR | DIR);
        e.delay(20_000);
        e.rec(qwc(SIF1), Some(8));
        e.rec_masked(chcr(SIF1), STR, STR);
        meet(&mut e, &mut i, 8);
        i.poke(ichcr(iop::SIF1), iop::STR | iop::CHAIN);
        i.wait(ichcr(iop::SIF1), iop::STR, false, 100_000);
        i.rec(imadr(iop::SIF1), Some(d4 + 4 * 60));
        iop_rec_sum(&mut i, d4, 60, sum(b4.iter().copied()));
        meet(&mut e, &mut i, 9);
        e.wait(chcr(SIF1), STR, false, 200_000);
        e.rec(qwc(SIF1), Some(0));
        e.rec(madr(SIF1), Some(e4 + 256));
        e.rec_masked(D_STAT, 0x40, 0x40);
        e.poke(D_STAT, 0x40);
        // Last: an EE source chain, cnt, ref and end, the IOP's middle tag with IRQ.
        let t = EE_AREA + 0x100;
        let r = EE_AREA + 0x200;
        let (b1, b2, b3) = (pattern(0x5200_0000, 8), pattern(0x5300_0000, 4), pattern(0x5400_0000, 4));
        e.fill(t, &cat(&[&ee_tag(id::CNT, 3, 0, false), &[d1, 8], &pad, &b1, &ee_tag(id::REF, 2, r, false), &ee_tag(id::END, 2, 0, false)]));
        e.fill(t + 0x60, &cat(&[&[iop_tag(d3, false, true), 4], &pad, &b3]));
        e.fill(r, &cat(&[&[iop_tag(d2, true, false), 4], &pad, &b2]));
        for d in [d1, d2, d3] {
            iop_fill(&mut i, d, &marks(9));
        }
        i.poke(DICR2, 0x3F00_0000 | 1 << iop::SIF1);
        i.poke(s21::reg::I_STAT, 0);
        i.poke(ichcr(iop::SIF1), iop::STR | iop::CHAIN | iop::TAG4);
        meet(&mut e, &mut i, 10);
        e.poke(tadr(SIF1), t);
        e.poke(qwc(SIF1), 0);
        e.poke(chcr(SIF1), STR | CHAIN | DIR);
        e.wait(chcr(SIF1), STR, false, 200_000);
        e.rec(tadr(SIF1), Some(t + 0x50));
        e.rec(madr(SIF1), Some(t + 0x80));
        e.rec(qwc(SIF1), Some(0));
        e.rec(chcr(SIF1), Some(0x7000_0000 | CHAIN | DIR));
        e.rec_masked(D_STAT, 0x40, 0x40);
        e.poke(D_STAT, 0x40);
        meet(&mut e, &mut i, 11);
        i.wait(ichcr(iop::SIF1), iop::STR, false, 100_000);
        i.rec(ichcr(iop::SIF1), Some(iop::CHAIN | iop::TAG4));
        i.rec(imadr(iop::SIF1), Some(d3 + 16));
        iop_rec_words(&mut i, d1, &cat(&[&b1, &[CC]]));
        iop_rec_words(&mut i, d2, &cat(&[&b2, &[CC]]));
        iop_rec_words(&mut i, d3, &cat(&[&b3, &[CC]]));
        i.rec_masked(DICR2, 0x3F00_0000, 1 << (24 + iop::SIF1 - 7));
        i.rec_masked(DICR, 0x8000_0000, 0x8000_0000);
        i.rec_masked(s21::reg::I_STAT, 8, 8);
        i.poke(DICR2, 0x3F00_0000);
        i.poke(s21::reg::I_STAT, 0);
        (e, i)
    })
}

/// `s22-sif0`, the IOP to the EE: the IOP started first with CHCR's bit 8, its channel holding with the FIFO full of
/// the EE's tag and data until the EE's destination chain starts; and, last, an IOP chain with bit 8, the last two words
/// of each tag the EE's destination tag, into an EE destination chain (cnt, end), the first IOP tag with IRQ (ps2tek
/// "SIF DMA", "IOP DMA"; EE Overview §2.6 and §3.1.2; D-140, D-142, D-144).
pub fn sif0() -> Pair {
    use ee::*;
    use s21::reg::{DICR2, chcr as ichcr, madr as imadr, tadr as itadr};
    pair("s22-sif0", |n| {
        let mut e = ee_side("s22-sif0", n);
        let mut i = iop_side("s22-sif0");
        iop_sif_setup(&mut i);
        // The IOP first: its 64 words wait in the FIFO, 32 words with the EE's tag, until the EE starts.
        let s3 = IOP_AREA + 0x1000;
        let f = EE_AREA + 0x1000;
        let p4 = pattern(0x6400_0000, 64);
        let end16 = ee_tag(id::END, 16, f, false);
        iop_fill(&mut i, s3, &[iop_tag(s3 + 0x10, false, true), 64, end16[0], end16[1]]);
        iop_fill(&mut i, s3 + 0x10, &p4);
        meet(&mut e, &mut i, 4);
        i.poke(itadr(iop::SIF0), s3);
        i.poke(ichcr(iop::SIF0), iop::STR | iop::CHAIN | iop::TAG4 | iop::DIR);
        i.delay(3_000);
        i.rec(imadr(iop::SIF0), Some(s3 + 0x10 + 4 * 30));
        i.rec_masked(ichcr(iop::SIF0), iop::STR, iop::STR);
        meet(&mut e, &mut i, 5);
        e.poke(qwc(SIF0), 0);
        e.poke(chcr(SIF0), STR | CHAIN);
        e.wait(chcr(SIF0), STR, false, 200_000);
        e.rec_sum(f, 64, sum(p4.iter().copied()));
        e.rec(madr(SIF0), Some(f + 256));
        e.rec(qwc(SIF0), Some(0));
        e.rec(chcr(SIF0), Some(0x7000_0000 | CHAIN));
        e.rec_masked(D_STAT, 0x20, 0x20);
        e.poke(D_STAT, 0x20);
        meet(&mut e, &mut i, 6);
        i.wait(ichcr(iop::SIF0), iop::STR, false, 100_000);
        i.rec(imadr(iop::SIF0), Some(s3 + 0x110));
        i.rec(ichcr(iop::SIF0), Some(iop::CHAIN | iop::TAG4 | iop::DIR));
        // Last: four-word IOP tags carrying the EE's destination tags, cnt then end, the first IOP tag with IRQ.
        let s = IOP_AREA;
        let (a, b) = (EE_AREA + 0x400, EE_AREA + 0x500);
        let (p1, p2) = (pattern(0x6100_0000, 8), pattern(0x6200_0000, 4));
        let cnt = ee_tag(id::CNT, 2, a, false);
        let end = ee_tag(id::END, 1, b, false);
        iop_fill(&mut i, s, &[iop_tag(s + 0x100, true, false), 8, cnt[0], cnt[1], iop_tag(s + 0x200, false, true), 4, end[0], end[1]]);
        iop_fill(&mut i, s + 0x100, &p1);
        iop_fill(&mut i, s + 0x200, &p2);
        i.poke(DICR2, 0x3F00_0000 | 1 << iop::SIF0);
        e.fill(a, &marks(12));
        e.fill(b, &marks(8));
        meet(&mut e, &mut i, 7);
        e.poke(qwc(SIF0), 0);
        e.poke(chcr(SIF0), STR | CHAIN);
        meet(&mut e, &mut i, 8);
        i.poke(itadr(iop::SIF0), s);
        i.poke(ichcr(iop::SIF0), iop::STR | iop::CHAIN | iop::TAG4 | iop::DIR);
        i.wait(ichcr(iop::SIF0), iop::STR, false, 100_000);
        i.rec(itadr(iop::SIF0), Some(s + 32));
        i.rec(imadr(iop::SIF0), Some(s + 0x210));
        i.rec(ichcr(iop::SIF0), Some(iop::CHAIN | iop::TAG4 | iop::DIR));
        i.rec_masked(DICR2, 0x3F00_0000, 1 << (24 + iop::SIF0 - 7));
        i.rec_masked(s21::reg::I_STAT, 8, 8);
        i.poke(DICR2, 0x3F00_0000);
        i.poke(s21::reg::I_STAT, 0);
        meet(&mut e, &mut i, 9);
        e.wait(chcr(SIF0), STR, false, 200_000);
        e.rec_words(a, &cat(&[&p1, &marks(4)]));
        e.rec_words(b, &cat(&[&p2, &marks(4)]));
        e.rec(madr(SIF0), Some(b + 16));
        e.rec(qwc(SIF0), Some(0));
        e.rec(chcr(SIF0), Some(0x7000_0000 | CHAIN));
        e.rec_masked(D_STAT, 0x20, 0x20);
        e.poke(D_STAT, 0x20);
        (e, i)
    })
}

/// `s22-sif0-normal`, the IOP to the EE without CHCR's bit 8: the IOP's two-word tag, its data alone crossing into an
/// EE transfer in normal mode, the two words after the tag left in memory and TADR past the tag alone (ps2tek "IOP DMA";
/// D-142).
pub fn sif0_normal() -> Pair {
    use ee::*;
    use s21::reg::{chcr as ichcr, madr as imadr, tadr as itadr};
    pair("s22-sif0-normal", |n| {
        let mut e = ee_side("s22-sif0-normal", n);
        let mut i = iop_side("s22-sif0-normal");
        iop_sif_setup(&mut i);
        // An IOP tag of two words into the EE's normal mode: the data alone crosses.
        let s2 = IOP_AREA + 0x400;
        let c = EE_AREA + 0x600;
        let p3 = pattern(0x6300_0000, 8);
        iop_fill(&mut i, s2, &[iop_tag(s2 + 0x10, false, true), 8, 0xEEEE_0002, 0xEEEE_0003]);
        iop_fill(&mut i, s2 + 0x10, &p3);
        e.fill(c, &marks(12));
        meet(&mut e, &mut i, 1);
        e.poke(madr(SIF0), c);
        e.poke(qwc(SIF0), 2);
        e.poke(chcr(SIF0), STR);
        meet(&mut e, &mut i, 2);
        i.poke(itadr(iop::SIF0), s2);
        i.poke(ichcr(iop::SIF0), iop::STR | iop::CHAIN | iop::DIR);
        i.wait(ichcr(iop::SIF0), iop::STR, false, 100_000);
        i.rec(imadr(iop::SIF0), Some(s2 + 0x30));
        i.rec(itadr(iop::SIF0), Some(s2 + 8));
        meet(&mut e, &mut i, 3);
        e.wait(chcr(SIF0), STR, false, 200_000);
        e.rec_words(c, &cat(&[&p3, &marks(4)]));
        e.rec(madr(SIF0), Some(c + 32));
        e.rec(qwc(SIF0), Some(0));
        e.rec_masked(D_STAT, 0x20, 0x20);
        e.poke(D_STAT, 0x20);
        (e, i)
    })
}

/// `s22-sif2`, both ways: the EE's channel 7 from memory in normal mode into the IOP's channel 2 in slice mode, then
/// the IOP's channel 2 from memory into the EE's channel 7 (ps2tek "SIF DMA", "IOP DMA"; EE User's Manual §5.1).
pub fn sif2() -> Pair {
    use ee::*;
    use s21::reg::{bcr as ibcr, chcr as ichcr, madr as imadr};
    pair("s22-sif2", |n| {
        let mut e = ee_side("s22-sif2", n);
        let mut i = iop_side("s22-sif2");
        iop_sif_setup(&mut i);
        let (g, h) = (EE_AREA + 0x700, IOP_AREA + 0x800);
        let p = pattern(0x6500_0000, 8);
        e.fill(g, &p);
        iop_fill(&mut i, h, &marks(9));
        meet(&mut e, &mut i, 1);
        e.poke(madr(SIF2), g);
        e.poke(qwc(SIF2), 2);
        e.poke(chcr(SIF2), STR | DIR);
        meet(&mut e, &mut i, 2);
        i.poke(imadr(iop::SIF2), h);
        i.poke(ibcr(iop::SIF2), 2 << 16 | 4);
        i.poke(ichcr(iop::SIF2), iop::STR | iop::SLICE);
        i.wait(ichcr(iop::SIF2), iop::STR, false, 100_000);
        iop_rec_words(&mut i, h, &cat(&[&p, &[CC]]));
        i.rec(imadr(iop::SIF2), Some(h + 32));
        i.rec(ibcr(iop::SIF2), Some(4));
        i.rec(ichcr(iop::SIF2), Some(iop::SLICE));
        meet(&mut e, &mut i, 3);
        e.wait(chcr(SIF2), STR, false, 200_000);
        e.rec(qwc(SIF2), Some(0));
        e.rec(madr(SIF2), Some(g + 32));
        e.rec_masked(D_STAT, 0x80, 0x80);
        e.poke(D_STAT, 0x80);
        let (g2, h2) = (EE_AREA + 0x800, IOP_AREA + 0x900);
        let q = pattern(0x6600_0000, 8);
        iop_fill(&mut i, h2, &q);
        e.fill(g2, &marks(12));
        meet(&mut e, &mut i, 4);
        e.poke(madr(SIF2), g2);
        e.poke(qwc(SIF2), 2);
        e.poke(chcr(SIF2), STR);
        meet(&mut e, &mut i, 5);
        i.poke(imadr(iop::SIF2), h2);
        i.poke(ibcr(iop::SIF2), 2 << 16 | 4);
        i.poke(ichcr(iop::SIF2), iop::STR | iop::SLICE | iop::DIR);
        i.wait(ichcr(iop::SIF2), iop::STR, false, 100_000);
        i.rec(imadr(iop::SIF2), Some(h2 + 32));
        i.rec(ichcr(iop::SIF2), Some(iop::SLICE | iop::DIR));
        meet(&mut e, &mut i, 6);
        e.wait(chcr(SIF2), STR, false, 200_000);
        e.rec_words(g2, &cat(&[&q, &marks(4)]));
        e.rec(madr(SIF2), Some(g2 + 32));
        e.rec_masked(D_STAT, 0x80, 0x80);
        e.poke(D_STAT, 0x80);
        (e, i)
    })
}

/// The gate's programs.
pub fn gate() -> Vec<Pair> {
    vec![registers(), sif1(), sif0(), sif0_normal(), sif2()]
}

/// The gate's programs and the probes, for the referees.
pub fn all() -> Vec<Pair> {
    let mut v = gate();
    v.extend([probe_iop(), probe_ee()]);
    v
}
