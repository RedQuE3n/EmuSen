//! The project's S26 programs (Sapphire_Native.md §32): bare IOP programs on the SPU2, generated as R3000A assembly for
//! `emusen-mips` on S21's frame (`s21::Program`), each writing its record from 00180000h and its done word at 0017FFF0h.
//! A referee runs them through an EE program (`s21::wrap`). The words the documents give are graded; what they leave to
//! the referee is recorded ungraded and weighed in `Sapphire_Disputes.md` D-260 on.

#![allow(dead_code)]

use crate::s21::{self, Program};

pub mod gate;

/// The SPU2's registers from the IOP, through kseg1 (ps2tek "I/O Maps").
pub const SPU2: u32 = 0xBF90_0000;
pub const CORE: u32 = 0x400;

pub mod r {
    pub const PMON: u32 = 0x180;
    pub const NON: u32 = 0x184;
    pub const VMIXL: u32 = 0x188;
    pub const VMIXEL: u32 = 0x18C;
    pub const VMIXR: u32 = 0x190;
    pub const VMIXER: u32 = 0x194;
    pub const MMIX: u32 = 0x198;
    pub const ATTR: u32 = 0x19A;
    pub const IRQAH: u32 = 0x19C;
    pub const IRQAL: u32 = 0x19E;
    pub const KON: u32 = 0x1A0;
    pub const KOFF: u32 = 0x1A4;
    pub const TSAH: u32 = 0x1A8;
    pub const TSAL: u32 = 0x1AA;
    pub const DATA: u32 = 0x1AC;
    pub const ADMA: u32 = 0x1B0;
    pub const ESAH: u32 = 0x2E0;
    pub const ESAL: u32 = 0x2E2;
    pub const REVERB: u32 = 0x2E4;
    pub const EEA: u32 = 0x33C;
    pub const ENDX: u32 = 0x340;
    pub const STATX: u32 = 0x344;
    /// ps2tek's rows for key on and key off.
    pub const KON_PS2TEK: u32 = 0x190;
    pub const KOFF_PS2TEK: u32 = 0x194;
    pub const fn voice(v: u32, i: u32) -> u32 {
        0x10 * v + 2 * i
    }
    pub const fn addr(v: u32, i: u32) -> u32 {
        0x1C0 + 12 * v + 2 * i
    }
}

/// A register of core `c` at offset `o`, as an IOP address.
pub const fn reg(c: u32, o: u32) -> u32 {
    SPU2 + CORE * c + o
}

/// A volume register of core `c`, `i` halfwords into its block at 760h + 28h × c.
pub const fn vol(c: u32, i: u32) -> u32 {
    SPU2 + 0x760 + 0x28 * c + 2 * i
}

pub fn sh(p: &mut Program, address: u32, v: u32) {
    p.li("t6", address);
    p.li("t7", v);
    p.emit("sh t7, 0(t6)");
}

pub fn lh(p: &mut Program, address: u32) {
    p.li("t6", address);
    p.emit("lhu t2, 0(t6)");
    p.emit("nop");
}

pub fn rec_h(p: &mut Program, address: u32, want: Option<u32>) {
    lh(p, address);
    p.record("t2", want);
}

/// Halfwords written at core `c`'s transfer address through the data port.
pub fn upload(p: &mut Program, c: u32, at: u32, halves: &[u16]) {
    sh(p, reg(c, r::TSAH), at >> 16);
    sh(p, reg(c, r::TSAL), at & 0xFFFF);
    p.li("t6", reg(c, r::DATA));
    for &h in halves {
        p.li("t7", h as u32);
        p.emit("sh t7, 0(t6)");
    }
}

/// A block of SPU-ADPCM: shift, filter, flags and 28 nibbles (psx-spx "SPU ADPCM Samples").
pub fn block(shift: u8, filter: u8, flags: u8, nibble: impl Fn(usize) -> u8) -> [u16; 8] {
    let mut h = [0u16; 8];
    h[0] = (shift as u16 & 0xF) | (filter as u16 & 7) << 4 | (flags as u16) << 8;
    for w in 0..7 {
        h[w + 1] = (0..4).fold(0u16, |a, k| a | ((nibble(4 * w + k) & 0xF) as u16) << (4 * k));
    }
    h
}

/// The square block the probes play: 14 samples of 7000h, 14 of -8000h, looping on itself.
pub fn square() -> [u16; 8] {
    block(0, 0, 7, |i| if i < 14 { 7 } else { 8 })
}

/// A constant block, for tests on the unit.
pub fn constant_block() -> [u16; 8] {
    gate::constant(4)
}

/// Timer 5 free-running on the IOP's clock, read into t2.
pub fn clock_start(p: &mut Program) {
    p.poke(s21::reg::mode(5), 0);
    p.poke(s21::reg::count(5), 0);
}

/// The DMA channel of core `c`: 4 or 7 (ps2tek "IOP DMA").
pub const fn channel(c: u32) -> u32 {
    if c == 0 { 4 } else { 7 }
}

/// A slice transfer of `blocks` of `size` words on core `c`'s channel, from IOP memory at `madr` (`from_ram`) or into
/// it, waited for.
pub fn dma(p: &mut Program, c: u32, madr: u32, size: u32, blocks: u32, from_ram: bool) {
    let ch = channel(c);
    p.peek(s21::reg::DPCR);
    p.li("t7", 0x0008_0000);
    p.emit("or t2, t2, t7");
    p.emit("sw t2, 0(t6)");
    p.peek(s21::reg::DPCR2);
    p.emit("ori t2, t2, 8");
    p.emit("sw t2, 0(t6)");
    p.poke(s21::reg::madr(ch), madr);
    p.poke(s21::reg::bcr(ch), blocks << 16 | size);
    p.poke(s21::reg::chcr(ch), 0x0100_0200 | from_ram as u32);
    p.wait(s21::reg::chcr(ch), 0x0100_0000, false, 200_000);
}

/// `s26-probe`: what the referee shows of the SPU2's registers, ungraded: the data port's write and read, each voice
/// register read back, key on at ps2tek's row and at 1A0h, ENVX, NAX and ENDX as a voice plays, the IRQ address, pitch
/// modulation, the reverb's writes into its area, AutoDMA's block in MEMIN, and the sound the whole run makes.
pub fn probe() -> Program {
    let mut p = Program::new("s26-probe");
    let sq = square();
    for c in 0..2 {
        sh(&mut p, reg(c, r::ATTR), 0);
        sh(&mut p, reg(c, r::KOFF), 0xFFFF);
        sh(&mut p, reg(c, r::KOFF + 2), 0xFF);
        sh(&mut p, reg(c, r::ADMA), 0);
    }
    p.delay(2000);
    // The data port: a block written at 5000h, then read back from 5000h.
    upload(&mut p, 0, 0x5000, &sq);
    rec_h(&mut p, reg(0, r::TSAH), None);
    rec_h(&mut p, reg(0, r::TSAL), None);
    sh(&mut p, reg(0, r::TSAH), 0);
    sh(&mut p, reg(0, r::TSAL), 0x5000);
    for _ in 0..8 {
        rec_h(&mut p, reg(0, r::DATA), None);
    }
    // The same through DMA channel 4, eight words into IOP memory.
    sh(&mut p, reg(0, r::TSAH), 0);
    sh(&mut p, reg(0, r::TSAL), 0x5000);
    sh(&mut p, reg(0, r::ATTR), 0x0030);
    dma(&mut p, 0, s21::AREA, 8, 1, false);
    sh(&mut p, reg(0, r::ATTR), 0);
    for i in 0..8 {
        p.rec(s21::K1 | (s21::AREA + 4 * i), None);
    }
    // Voice 0's registers written and read back, and SSAH's width.
    let regs = [0x1000, 0x2000, 0x1000, 0x000F, 0x1FC0];
    for (i, v) in regs.iter().enumerate() {
        sh(&mut p, reg(0, r::voice(0, i as u32)), *v);
    }
    sh(&mut p, reg(0, r::addr(0, 0)), 0xFFFF);
    rec_h(&mut p, reg(0, r::addr(0, 0)), None);
    sh(&mut p, reg(0, r::addr(0, 0)), 0);
    sh(&mut p, reg(0, r::addr(0, 1)), 0x5000);
    for i in 0..8 {
        rec_h(&mut p, reg(0, r::voice(0, i)), None);
    }
    for i in 0..6 {
        rec_h(&mut p, reg(0, r::addr(0, i)), None);
    }
    for c in 0..2 {
        sh(&mut p, vol(c, 0), 0x3FFF);
        sh(&mut p, vol(c, 1), 0x3FFF);
        sh(&mut p, vol(c, 4), 0x7FFF);
        sh(&mut p, vol(c, 5), 0x7FFF);
        sh(&mut p, reg(c, r::MMIX), 0x0FFF);
    }
    sh(&mut p, reg(0, r::VMIXL), 1);
    sh(&mut p, reg(0, r::VMIXR), 0);
    sh(&mut p, reg(0, r::ATTR), 0xC000);
    sh(&mut p, reg(1, r::ATTR), 0xC000);
    // Key on at ps2tek's row.
    sh(&mut p, reg(0, r::KON_PS2TEK), 1);
    p.delay(20_000);
    rec_h(&mut p, reg(0, r::voice(0, 5)), None);
    rec_h(&mut p, reg(0, r::addr(0, 4)), None);
    rec_h(&mut p, reg(0, r::addr(0, 5)), None);
    rec_h(&mut p, reg(0, r::ENDX), None);
    // Key on at 1A0h.
    sh(&mut p, reg(0, r::VMIXR), 1);
    sh(&mut p, reg(0, r::KON), 1);
    p.delay(20_000);
    for i in [5, 6, 7] {
        rec_h(&mut p, reg(0, r::voice(0, i)), None);
    }
    rec_h(&mut p, reg(0, r::addr(0, 4)), None);
    rec_h(&mut p, reg(0, r::addr(0, 5)), None);
    rec_h(&mut p, reg(0, r::ENDX), None);
    // NAX twice around a delay timed by timer 5.
    clock_start(&mut p);
    rec_h(&mut p, reg(0, r::addr(0, 5)), None);
    p.delay(5000);
    p.read_count(5);
    p.record("t2", None);
    rec_h(&mut p, reg(0, r::addr(0, 5)), None);
    // The IRQ address inside the block, IRQ enabled.
    p.poke(s21::reg::I_STAT, 0);
    sh(&mut p, reg(0, r::IRQAH), 0);
    sh(&mut p, reg(0, r::IRQAL), 0x5003);
    sh(&mut p, reg(0, r::ATTR), 0xC040);
    p.delay(5000);
    p.rec(s21::reg::I_STAT, None);
    rec_h(&mut p, reg(0, r::STATX), None);
    rec_h(&mut p, SPU2 + 0x7C2, None);
    sh(&mut p, reg(0, r::ATTR), 0xC000);
    p.rec(s21::reg::I_STAT, None);
    rec_h(&mut p, reg(0, r::STATX), None);
    rec_h(&mut p, SPU2 + 0x7C2, None);
    // Pitch modulation: voices 1 and 2 on the same block, 2 modulated by 1.
    for v in [1, 2] {
        for (i, x) in regs.iter().enumerate() {
            sh(&mut p, reg(0, r::voice(v, i as u32)), *x);
        }
        sh(&mut p, reg(0, r::addr(v, 0)), 0);
        sh(&mut p, reg(0, r::addr(v, 1)), 0x5000);
    }
    sh(&mut p, reg(0, r::PMON), 4);
    sh(&mut p, reg(0, r::KON), 6);
    p.delay(20_000);
    for v in [1, 2] {
        rec_h(&mut p, reg(0, r::addr(v, 4)), None);
        rec_h(&mut p, reg(0, r::addr(v, 5)), None);
    }
    sh(&mut p, reg(0, r::PMON), 0);
    // The reverb: area from 10000h to 1FFFFh, mLSAME at 100h, vLIN and vIIR full, voice 0 wet on both sides.
    sh(&mut p, reg(0, r::ESAH), 1);
    sh(&mut p, reg(0, r::ESAL), 0);
    sh(&mut p, reg(0, r::EEA), 1);
    for k in 0..22 {
        sh(&mut p, reg(0, r::REVERB + 4 * k), 0);
        sh(&mut p, reg(0, r::REVERB + 4 * k + 2), if k == 2 || k == 3 { 0x100 } else { 0 });
    }
    sh(&mut p, vol(0, 10), 0x7FFF);
    sh(&mut p, vol(0, 18), 0x7FFF);
    sh(&mut p, vol(0, 19), 0x7FFF);
    sh(&mut p, reg(0, r::VMIXEL), 1);
    sh(&mut p, reg(0, r::VMIXER), 1);
    sh(&mut p, reg(0, r::ATTR), 0xC080);
    p.delay(20_000);
    sh(&mut p, reg(0, r::ATTR), 0xC000);
    sh(&mut p, reg(0, r::TSAH), 1);
    sh(&mut p, reg(0, r::TSAL), 0);
    sh(&mut p, reg(0, r::ATTR), 0xC030);
    dma(&mut p, 0, s21::AREA, 16, 16, false);
    sh(&mut p, reg(0, r::ATTR), 0xC000);
    // The count of nonzero halfwords in the first 512 of the area, and the first nonzero's index.
    p.li("t0", s21::K1 | s21::AREA);
    p.li("t1", 512);
    p.li("t3", 0);
    p.li("t4", 0xFFFF);
    let (l, skip) = (p.label(), p.label());
    p.place(&l);
    p.emit("lhu t2, 0(t0)");
    p.emit("nop");
    p.emit(format!("beq t2, zero, {skip}"));
    p.emit("nop");
    p.emit("addiu t3, t3, 1");
    p.li("t5", 0xFFFF);
    p.emit(format!("bne t4, t5, {skip}"));
    p.emit("nop");
    p.li("t5", s21::K1 | s21::AREA);
    p.emit("subu t4, t0, t5");
    p.emit("srl t4, t4, 1");
    p.place(&skip);
    p.emit("addiu t0, t0, 2");
    p.emit("addiu t1, t1, -1");
    p.emit(format!("bne t1, zero, {l}"));
    p.emit("nop");
    p.record("t3", None);
    p.record("t4", None);
    // AutoDMA on core 0: a block of 256 words of a ramp, into MEMIN.
    for i in 0..256u32 {
        let lo = (i * 2) & 0xFFFF;
        p.poke(s21::K1 | (s21::AREA + 4 * i), lo | (lo + 1) << 16 | 0x4000_4000);
    }
    sh(&mut p, reg(0, r::ADMA), 1);
    rec_h(&mut p, reg(0, r::ADMA), None);
    rec_h(&mut p, reg(0, r::STATX), None);
    dma(&mut p, 0, s21::AREA, 16, 16, true);
    p.record("t5", None);
    rec_h(&mut p, reg(0, r::ADMA), None);
    rec_h(&mut p, reg(0, r::STATX), None);
    p.delay(20_000);
    sh(&mut p, reg(0, r::ADMA), 0);
    for at in [0x2000u32, 0x20FF, 0x2100, 0x2200, 0x2300] {
        sh(&mut p, reg(0, r::TSAH), 0);
        sh(&mut p, reg(0, r::TSAL), at);
        rec_h(&mut p, reg(0, r::DATA), None);
    }
    // Let the voices sound a while for the referee's wave log.
    p.delay(400_000);
    sh(&mut p, reg(0, r::KOFF), 0xFFFF);
    p
}

/// Counts the nonzero halfwords of sound RAM in each of `regions` regions of 100h halfwords from 0, read through core
/// 0's channel into IOP memory a region at a time; a count a word.
pub fn census(p: &mut Program, regions: u32) {
    for k in 0..regions {
        sh(p, reg(0, r::TSAH), (k * 0x100) >> 16);
        sh(p, reg(0, r::TSAL), (k * 0x100) & 0xFFFF);
        dma(p, 0, s21::AREA, 16, 8, false);
        p.li("t0", s21::K1 | s21::AREA);
        p.li("t1", 256);
        p.li("t3", 0);
        let (l, skip) = (p.label(), p.label());
        p.place(&l);
        p.emit("lhu t2, 0(t0)");
        p.emit("nop");
        p.emit(format!("beq t2, zero, {skip}"));
        p.emit("nop");
        p.emit("addiu t3, t3, 1");
        p.place(&skip);
        p.emit("addiu t0, t0, 2");
        p.emit("addiu t1, t1, -1");
        p.emit(format!("bne t1, zero, {l}"));
        p.emit("nop");
        p.record("t3", None);
    }
}

/// `s26-outputs`: a voice plays on core 0 for a while, its output dry to core 1; then where sound RAM was written.
pub fn outputs() -> Program {
    let mut p = Program::new("s26-outputs");
    for c in 0..2 {
        sh(&mut p, reg(c, r::ATTR), 0);
        sh(&mut p, reg(c, r::KOFF), 0xFFFF);
        sh(&mut p, reg(c, r::KOFF + 2), 0xFF);
        sh(&mut p, reg(c, r::ADMA), 0);
    }
    upload(&mut p, 0, 0x5000, &square());
    census(&mut p, 0x30);
    for (i, v) in [0x1000, 0x2000, 0x1000, 0x000F, 0x1FC0].iter().enumerate() {
        sh(&mut p, reg(0, r::voice(0, i as u32)), *v);
    }
    sh(&mut p, reg(0, r::addr(0, 0)), 0);
    sh(&mut p, reg(0, r::addr(0, 1)), 0x5000);
    for c in 0..2 {
        sh(&mut p, vol(c, 0), 0x3FFF);
        sh(&mut p, vol(c, 1), 0x3FFF);
        sh(&mut p, vol(c, 4), 0x7FFF);
        sh(&mut p, vol(c, 5), 0x7FFF);
        sh(&mut p, reg(c, r::MMIX), 0x0FFF);
        sh(&mut p, reg(c, r::ATTR), 0xC000);
    }
    sh(&mut p, reg(0, r::VMIXL), 1);
    sh(&mut p, reg(0, r::VMIXR), 1);
    sh(&mut p, reg(0, r::KON), 1);
    p.delay(2_000_000);
    census(&mut p, 0x30);
    dump(&mut p, 0x0, 0x1000);
    sh(&mut p, reg(0, r::KOFF), 1);
    p
}

/// `halves` halfwords of sound RAM from `at`, read through core 0's channel straight into the record, ungraded.
pub fn dump(p: &mut Program, at: u32, halves: u32) {
    let words = halves / 2;
    sh(p, reg(0, r::TSAH), at >> 16);
    sh(p, reg(0, r::TSAL), at & 0xFFFF);
    p.emit("lui t0, 0x1fff");
    p.emit("ori t0, t0, 0xffff");
    p.emit("and t0, s1, t0");
    let ch = channel(0);
    p.peek(s21::reg::DPCR);
    p.li("t7", 0x0008_0000);
    p.emit("or t2, t2, t7");
    p.emit("sw t2, 0(t6)");
    p.li("t6", s21::reg::madr(ch));
    p.emit("sw t0, 0(t6)");
    p.poke(s21::reg::bcr(ch), (words / 16) << 16 | 16);
    p.poke(s21::reg::chcr(ch), 0x0100_0200);
    p.wait(s21::reg::chcr(ch), 0x0100_0000, false, 2_000_000);
    p.li("t7", 4 * words);
    p.emit("addu s1, s1, t7");
    p.expected.extend(std::iter::repeat_n(None, words as usize));
}

/// Silences both cores: every voice keyed off, ATTR, AutoDMA and the mixes cleared.
pub fn reset(p: &mut Program) {
    for c in 0..2 {
        sh(p, reg(c, r::ATTR), 0);
        sh(p, reg(c, r::KOFF), 0xFFFF);
        sh(p, reg(c, r::KOFF + 2), 0xFF);
        sh(p, reg(c, r::ADMA), 0);
        for o in [r::PMON, r::NON, r::VMIXL, r::VMIXEL, r::VMIXR, r::VMIXER] {
            sh(p, reg(c, o), 0);
            sh(p, reg(c, o + 2), 0);
        }
        sh(p, reg(c, r::MMIX), 0);
        for i in 0..20 {
            sh(p, vol(c, i), 0);
        }
    }
}

/// A voice's registers: volumes, pitch, ADSR and its start address.
pub fn voice(p: &mut Program, c: u32, v: u32, vols: [u32; 2], pitch: u32, adsr: [u32; 2], start: u32) {
    for (i, x) in [vols[0], vols[1], pitch, adsr[0], adsr[1]].iter().enumerate() {
        sh(p, reg(c, r::voice(v, i as u32)), *x);
    }
    sh(p, reg(c, r::addr(v, 0)), start >> 16);
    sh(p, reg(c, r::addr(v, 1)), start & 0xFFFF);
}

/// `s26-rings`: where the data port writes as DMA reads it, and what the referee writes into sound RAM as voices play:
/// voice 0 of core 0 dry, voice 1 wet, voice 0 of core 1 dry, each at its own volumes.
pub fn rings() -> Program {
    let mut p = Program::new("s26-rings");
    reset(&mut p);
    let pattern: Vec<u16> = (1..=16).map(|i| 0x1111u16.wrapping_mul(i)).collect();
    upload(&mut p, 0, 0x6000, &pattern);
    for at in [0x6000u32, 0x3000, 0x6008] {
        dump(&mut p, at, 32);
    }
    upload(&mut p, 0, 0x5000, &square());
    voice(&mut p, 0, 0, [0x1000, 0x2000], 0x1000, [0x000F, 0x1FC0], 0x5000);
    voice(&mut p, 0, 1, [0x0800, 0x0400], 0x0800, [0x000F, 0x1FC0], 0x5000);
    voice(&mut p, 1, 0, [0x0200, 0x0100], 0x2000, [0x000F, 0x1FC0], 0x5000);
    sh(&mut p, reg(0, r::VMIXL), 1);
    sh(&mut p, reg(0, r::VMIXR), 1);
    sh(&mut p, reg(0, r::VMIXEL), 2);
    sh(&mut p, reg(0, r::VMIXER), 2);
    sh(&mut p, reg(1, r::VMIXL), 1);
    sh(&mut p, reg(1, r::VMIXR), 1);
    for c in 0..2 {
        sh(&mut p, vol(c, 0), 0x3FFF);
        sh(&mut p, vol(c, 1), 0x3FFF);
        sh(&mut p, reg(c, r::ATTR), 0xC000);
    }
    sh(&mut p, reg(0, r::KON), 3);
    sh(&mut p, reg(1, r::KON), 1);
    p.delay(2_000_000);
    dump(&mut p, 0, 0x2000);
    p
}

/// The largest and smallest signed halfword of each 200h-halfword ring of sound RAM from 0 to 1FFFh, read through core
/// 0's channel into IOP memory: 32 words.
pub fn ring_stats(p: &mut Program) {
    sh(p, reg(0, r::TSAH), 0);
    sh(p, reg(0, r::TSAL), 0);
    dma(p, 0, s21::AREA, 16, 0x100, false);
    p.li("t0", s21::K1 | s21::AREA);
    p.li("t1", 16);
    let (ring, l, a, b) = (p.label(), p.label(), p.label(), p.label());
    p.place(&ring);
    p.li("t3", 0xFFFF_8000);
    p.li("t4", 0x7FFF);
    p.li("t5", 0x200);
    p.place(&l);
    p.emit("lh t2, 0(t0)");
    p.emit("nop");
    p.emit("slt t7, t3, t2");
    p.emit(format!("beq t7, zero, {a}"));
    p.emit("nop");
    p.emit("addu t3, t2, zero");
    p.place(&a);
    p.emit("slt t7, t2, t4");
    p.emit(format!("beq t7, zero, {b}"));
    p.emit("nop");
    p.emit("addu t4, t2, zero");
    p.place(&b);
    p.emit("addiu t0, t0, 2");
    p.emit("addiu t5, t5, -1");
    p.emit(format!("bne t5, zero, {l}"));
    p.emit("nop");
    p.record("t3", None);
    p.record("t4", None);
    p.emit("addiu t1, t1, -1");
    p.emit(format!("bne t1, zero, {ring}"));
    p.emit("nop");
    for _ in 0..30 {
        p.expected.push(None);
    }
}

/// A scene's settling: about 1,300 samples on either machine.
pub fn settle(p: &mut Program) {
    p.delay(300_000);
}

/// `s26-mmix`: the referee's rings as MMIX, the input volumes, AutoDMA and the reverb are changed one at a time.
pub fn mix() -> Program {
    let mut p = Program::new("s26-mmix");
    reset(&mut p);
    upload(&mut p, 0, 0x5000, &square());
    voice(&mut p, 0, 0, [0x1000, 0x2000], 0x1000, [0x000F, 0x1FC0], 0x5000);
    voice(&mut p, 0, 1, [0x0800, 0x0400], 0x0800, [0x000F, 0x1FC0], 0x5000);
    voice(&mut p, 1, 0, [0x0200, 0x0100], 0x2000, [0x000F, 0x1FC0], 0x5000);
    voice(&mut p, 1, 1, [0x0080, 0x0040], 0x0400, [0x000F, 0x1FC0], 0x5000);
    voice(&mut p, 1, 3, [0x0300, 0x0600], 0x0C00, [0x000F, 0x1FC0], 0x5000);
    sh(&mut p, reg(0, r::VMIXL), 1);
    sh(&mut p, reg(0, r::VMIXR), 1);
    sh(&mut p, reg(0, r::VMIXEL), 2);
    sh(&mut p, reg(0, r::VMIXER), 2);
    sh(&mut p, reg(1, r::VMIXL), 1);
    sh(&mut p, reg(1, r::VMIXR), 1);
    sh(&mut p, reg(1, r::VMIXEL), 2);
    sh(&mut p, reg(1, r::VMIXER), 2);
    for c in 0..2 {
        for i in [0, 1, 4, 5, 6, 7] {
            sh(&mut p, vol(c, i), 0x3FFF);
        }
        sh(&mut p, reg(c, r::ATTR), 0xC000);
    }
    sh(&mut p, reg(0, r::KON), 3);
    sh(&mut p, reg(1, r::KON), 0xB);
    settle(&mut p);
    ring_stats(&mut p);
    for k in 0..12 {
        sh(&mut p, reg(0, r::MMIX), 1 << k);
        settle(&mut p);
        ring_stats(&mut p);
    }
    sh(&mut p, reg(0, r::MMIX), 0xFFF);
    for k in 0..12 {
        sh(&mut p, reg(1, r::MMIX), 1 << k);
        settle(&mut p);
        ring_stats(&mut p);
    }
    // The input volumes of core 1 halved one at a time, all of its MMIX set.
    sh(&mut p, reg(1, r::MMIX), 0xFFF);
    for i in [4, 5, 6, 7] {
        sh(&mut p, vol(1, i), 0x1000);
        settle(&mut p);
        ring_stats(&mut p);
        sh(&mut p, vol(1, i), 0x3FFF);
    }
    // ATTR's top bits on core 0.
    for a in [0x8000u32, 0x4000, 0x0000, 0xC000] {
        sh(&mut p, reg(0, r::ATTR), a);
        settle(&mut p);
        ring_stats(&mut p);
    }
    p
}

/// `s26-out`: the final output as core 1's MMIX, its input volumes, master volume and ATTR change, a scene each of
/// about 0.2 s, for the referee's captured sound.
pub fn out() -> Program {
    let mut p = Program::new("s26-out");
    reset(&mut p);
    upload(&mut p, 0, 0x5000, &square());
    voice(&mut p, 0, 0, [0x3FFF, 0x3FFF], 0x1000, [0x000F, 0x1FC0], 0x5000);
    voice(&mut p, 1, 0, [0x3FFF, 0x3FFF], 0x0800, [0x000F, 0x1FC0], 0x5000);
    sh(&mut p, reg(0, r::VMIXL), 1);
    sh(&mut p, reg(0, r::VMIXR), 1);
    sh(&mut p, reg(1, r::VMIXL), 1);
    sh(&mut p, reg(1, r::VMIXR), 1);
    sh(&mut p, reg(0, r::MMIX), 0xC00);
    for c in 0..2 {
        for i in [0, 1, 4, 5, 6, 7] {
            sh(&mut p, vol(c, i), 0x3FFF);
        }
        sh(&mut p, reg(c, r::ATTR), 0xC000);
    }
    sh(&mut p, reg(0, r::KON), 1);
    let scene = |p: &mut Program| p.delay(2_400_000);
    scene(&mut p);
    for k in 0..8 {
        sh(&mut p, reg(1, r::MMIX), 1 << k);
        scene(&mut p);
    }
    sh(&mut p, reg(1, r::MMIX), 0xFF);
    for i in [4, 6] {
        sh(&mut p, vol(1, i), 0x1000);
        sh(&mut p, vol(1, i + 1), 0x1000);
        scene(&mut p);
        sh(&mut p, vol(1, i), 0x3FFF);
        sh(&mut p, vol(1, i + 1), 0x3FFF);
    }
    sh(&mut p, vol(1, 0), 0x1000);
    scene(&mut p);
    sh(&mut p, vol(1, 0), 0x3FFF);
    for a in [0x0000u32, 0x8000, 0x4000] {
        sh(&mut p, reg(1, r::ATTR), a);
        scene(&mut p);
    }
    sh(&mut p, reg(1, r::ATTR), 0xC000);
    sh(&mut p, reg(0, r::ATTR), 0);
    scene(&mut p);
    sh(&mut p, reg(0, r::ATTR), 0xC000);
    sh(&mut p, reg(1, r::MMIX), 0xC00);
    sh(&mut p, reg(1, r::KON), 1);
    scene(&mut p);
    sh(&mut p, reg(0, r::KOFF), 1);
    sh(&mut p, reg(1, r::KOFF), 1);
    scene(&mut p);
    p
}

/// Counts the nonzero halfwords of each of `n` regions of `size` halfwords from `at`, a word each; `size` a multiple of
/// 20h and at most 2000h.
pub fn census_at(p: &mut Program, at: u32, size: u32, n: u32) {
    for k in 0..n {
        let a = at + k * size;
        sh(p, reg(0, r::TSAH), a >> 16);
        sh(p, reg(0, r::TSAL), a & 0xFFFF);
        dma(p, 0, s21::AREA, 16, size / 32, false);
        p.li("t0", s21::K1 | s21::AREA);
        p.li("t1", size);
        p.li("t3", 0);
        let (l, skip) = (p.label(), p.label());
        p.place(&l);
        p.emit("lhu t2, 0(t0)");
        p.emit("nop");
        p.emit(format!("beq t2, zero, {skip}"));
        p.emit("nop");
        p.emit("addiu t3, t3, 1");
        p.place(&skip);
        p.emit("addiu t0, t0, 2");
        p.emit("addiu t1, t1, -1");
        p.emit(format!("bne t1, zero, {l}"));
        p.emit("nop");
        p.record("t3", None);
    }
}

/// `s26-explore`: the reverb's writes into its area, AutoDMA's input and where it shows, and which accesses hit the IRQ
/// address.
pub fn explore() -> Program {
    let mut p = Program::new("s26-explore");
    reset(&mut p);
    upload(&mut p, 0, 0x5000, &square());
    voice(&mut p, 0, 0, [0x3FFF, 0x3FFF], 0x1000, [0x000F, 0x1FC0], 0x5000);
    voice(&mut p, 0, 1, [0x3FFF, 0x3FFF], 0x1000, [0x000F, 0x1FC0], 0x5000);
    sh(&mut p, reg(0, r::VMIXL), 1);
    sh(&mut p, reg(0, r::VMIXR), 1);
    sh(&mut p, reg(0, r::VMIXEL), 1);
    sh(&mut p, reg(0, r::VMIXER), 1);
    sh(&mut p, reg(0, r::MMIX), 0xFFF);
    sh(&mut p, reg(1, r::MMIX), 0xFFF);
    for c in 0..2 {
        for i in [0, 1, 2, 3, 4, 5, 6, 7] {
            sh(&mut p, vol(c, i), 0x3FFF);
        }
        sh(&mut p, reg(c, r::ATTR), 0xC000);
    }
    // The reverb: area 10000h-1FFFFh, mLSAME and mRSAME 100h, vLIN, vRIN and vIIR full.
    sh(&mut p, reg(0, r::ESAH), 1);
    sh(&mut p, reg(0, r::ESAL), 0);
    sh(&mut p, reg(0, r::EEA), 1);
    for k in 0..22 {
        sh(&mut p, reg(0, r::REVERB + 4 * k), 0);
        sh(&mut p, reg(0, r::REVERB + 4 * k + 2), if k == 2 || k == 3 { 0x100 } else { 0 });
    }
    sh(&mut p, vol(0, 10), 0x7FFF);
    sh(&mut p, vol(0, 18), 0x7FFF);
    sh(&mut p, vol(0, 19), 0x7FFF);
    sh(&mut p, reg(0, r::KON), 1);
    settle(&mut p);
    census_at(&mut p, 0x10000, 0x800, 32);
    sh(&mut p, reg(0, r::ATTR), 0xC080);
    settle(&mut p);
    census_at(&mut p, 0x10000, 0x800, 32);
    census_at(&mut p, 0x0, 0x2000, 8);
    rec_h(&mut p, reg(0, r::ATTR), None);
    for i in 0..4 {
        rec_h(&mut p, reg(0, r::ESAH + 2 * i), None);
    }
    rec_h(&mut p, reg(0, r::EEA), None);
    sh(&mut p, reg(0, r::ATTR), 0xC000);
    // AutoDMA on core 0: one block, left 1000h + i, right E000h + i.
    for i in 0..128u32 {
        p.poke(s21::K1 | (s21::AREA + 0x1000 + 4 * i), (0x1000 + 2 * i) | (0x1001 + 2 * i) << 16);
        p.poke(s21::K1 | (s21::AREA + 0x1200 + 4 * i), (0xE000 + 2 * i) | (0xE001 + 2 * i) << 16);
    }
    sh(&mut p, reg(0, r::KOFF), 1);
    sh(&mut p, reg(0, r::ADMA), 1);
    dma(&mut p, 0, s21::AREA + 0x1000, 16, 16, true);
    p.record("t5", None);
    p.delay(20_000);
    ring_stats(&mut p);
    dump(&mut p, 0x2000, 0x400);
    dump(&mut p, 0x0, 0x400);
    settle(&mut p);
    ring_stats(&mut p);
    dump(&mut p, 0x0, 0x400);
    sh(&mut p, reg(0, r::ADMA), 0);
    // IRQ addresses: the block's header, a sample word, voice 1's ring, the dry ring; each with core 0's IRQ enabled.
    sh(&mut p, reg(0, r::KON), 3);
    for at in [0x5000u32, 0x5005, 0x0410, 0x1010, 0x5008] {
        sh(&mut p, reg(0, r::ATTR), 0xC000);
        p.poke(s21::reg::I_STAT, 0);
        sh(&mut p, reg(0, r::IRQAH), at >> 16);
        sh(&mut p, reg(0, r::IRQAL), at & 0xFFFF);
        sh(&mut p, reg(0, r::ATTR), 0xC040);
        p.delay(20_000);
        p.rec(s21::reg::I_STAT, None);
        rec_h(&mut p, SPU2 + 0x7C2, None);
    }
    // Core 1's IRQ address at the block, core 1's IRQ enabled, core 0's off: whether core 0's voice hits core 1's.
    sh(&mut p, reg(0, r::ATTR), 0xC000);
    p.poke(s21::reg::I_STAT, 0);
    sh(&mut p, reg(1, r::IRQAH), 0);
    sh(&mut p, reg(1, r::IRQAL), 0x5003);
    sh(&mut p, reg(1, r::ATTR), 0xC040);
    p.delay(20_000);
    p.rec(s21::reg::I_STAT, None);
    rec_h(&mut p, SPU2 + 0x7C2, None);
    rec_h(&mut p, reg(1, r::STATX), None);
    sh(&mut p, reg(1, r::ATTR), 0xC000);
    rec_h(&mut p, SPU2 + 0x7C2, None);
    p
}

/// `s26-adma`: AutoDMA's input in the referee's rings: two blocks on core 0, MMIX's input bits one at a time.
pub fn adma_probe() -> Program {
    let mut p = Program::new("s26-adma-probe");
    reset(&mut p);
    for c in 0..2 {
        for i in [0, 1, 4, 5, 6, 7] {
            sh(&mut p, vol(c, i), 0x3FFF);
        }
        sh(&mut p, reg(c, r::ATTR), 0xC000);
    }
    for b in 0..2u32 {
        for i in 0..128u32 {
            let base = s21::K1 | (s21::AREA + 0x1000 + 0x400 * b);
            p.poke(base + 4 * i, (0x1000 * (b + 1) + 2 * i) | (0x1001 * (b + 1) + 2 * i) << 16);
            p.poke(base + 0x200 + 4 * i, (0x4000 + 0x1000 * b + 2 * i) | (0x4001 + 0x1000 * b + 2 * i) << 16);
        }
    }
    for k in [0u32, 4, 5, 6, 7] {
        sh(&mut p, reg(0, r::MMIX), if k == 0 { 0 } else { 1 << k });
        sh(&mut p, reg(0, r::ADMA), 1);
        dma(&mut p, 0, s21::AREA + 0x1000, 16, 32, true);
        p.record("t5", None);
        p.delay(5_000);
        dump(&mut p, 0x800, 0x400);
        sh(&mut p, reg(0, r::ADMA), 0);
        p.delay(100_000);
    }
    p
}

/// `s26-adma-sound`: AutoDMA streaming for about half a second on core 0, its input to core 1 and the output, for the
/// referee's captured sound; TSA before and after a block.
pub fn adma_sound() -> Program {
    let mut p = Program::new("s26-adma-sound");
    reset(&mut p);
    for c in 0..2 {
        for i in [0, 1, 4, 5, 6, 7] {
            sh(&mut p, vol(c, i), 0x3FFF);
        }
        sh(&mut p, reg(c, r::ATTR), 0xC000);
    }
    sh(&mut p, reg(0, r::MMIX), 0xF0);
    sh(&mut p, reg(1, r::MMIX), 0xFF);
    for i in 0..128u32 {
        let base = s21::K1 | (s21::AREA + 0x1000);
        let l = |k: u32| if k % 64 < 32 { 0x2000u32 } else { 0xE000 };
        let r = |k: u32| if k % 32 < 16 { 0x1000u32 } else { 0xF000 };
        p.poke(base + 4 * i, l(2 * i) | l(2 * i + 1) << 16);
        p.poke(base + 0x200 + 4 * i, r(2 * i) | r(2 * i + 1) << 16);
    }
    sh(&mut p, reg(0, r::TSAH), 0);
    sh(&mut p, reg(0, r::TSAL), 0x7000);
    sh(&mut p, reg(0, r::ADMA), 1);
    dma(&mut p, 0, s21::AREA + 0x1000, 16, 16, true);
    rec_h(&mut p, reg(0, r::TSAH), None);
    rec_h(&mut p, reg(0, r::TSAL), None);
    let l = p.label();
    p.li("s2", 200);
    p.place(&l);
    dma(&mut p, 0, s21::AREA + 0x1000, 16, 16, true);
    p.emit("addiu s2, s2, -1");
    p.emit(format!("bne s2, zero, {l}"));
    p.emit("nop");
    rec_h(&mut p, reg(0, r::TSAL), None);
    sh(&mut p, reg(0, r::ADMA), 0);
    p.delay(1_000_000);
    p
}

/// The measurement programs: what the referee shows, ungraded, for the disputes (Sapphire_Native.md §32.3).
pub fn measures() -> Vec<Program> {
    vec![probe(), outputs(), rings(), mix(), out(), explore(), adma_probe(), adma_sound()]
}
