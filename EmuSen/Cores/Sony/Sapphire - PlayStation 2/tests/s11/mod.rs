//! The project's S11 programs (Sapphire_Native.md §20.7): bare EE programs on the DMAC's scratchpad channels and common
//! registers, its source and destination chains with their tag interrupts, the address stack, interleave, stall
//! control and the MFIFO; the INTC and the core's interrupt lines; the four timers in every mode. Each runs from kseg0
//! with its constants, buffers and record reached through kseg1, so the core's caches never hold what the DMAC moves.
//! Every expected word is derived here from the EE User's Manual's chapters 3 to 5 (and, for interleave, checked
//! against ps2autotests' console text), written apart from `src/dmac.rs`, `src/timer.rs` and `src/intc.rs`.

#![allow(dead_code)]

use crate::s6::Program;

pub const K1: u32 = 0xA000_0000;
/// The scratchpad as the programs reach it, through the S entry at 70000000h.
pub const SPR: u32 = 0x7000_0000;
/// The interrupt handler's log, in kseg1.
pub const LOG: u32 = 0x0038_0000;

pub fn sx(v: u32) -> Option<u64> {
    Some(v as i32 as i64 as u64)
}

pub fn li(p: &mut Program, reg: &str, v: u32) {
    p.emit(format!("lui {reg}, 0x{:x}", v >> 16));
    p.emit(format!("ori {reg}, {reg}, 0x{:x}", v & 0xFFFF));
}

/// A register or memory word at physical `pa` written through kseg1.
pub fn poke(p: &mut Program, pa: u32, v: u32) {
    li(p, "t6", K1 | pa);
    li(p, "t7", v);
    p.emit("sw t7, 0(t6)");
}

/// The word at physical `pa` read through kseg1 into t2.
pub fn peek(p: &mut Program, pa: u32) {
    li(p, "t6", K1 | pa);
    p.emit("lw t2, 0(t6)");
}

/// Records the word at physical `pa`.
pub fn rec(p: &mut Program, pa: u32, want: Option<u32>) {
    peek(p, pa);
    p.record("t2", want.and_then(sx));
}

/// Records the word at physical `pa` ANDed with `mask` (a 16-bit mask).
pub fn rec_masked(p: &mut Program, pa: u32, mask: u32, want: u32) {
    peek(p, pa);
    if mask > 0xFFFF {
        li(p, "t7", mask);
        p.emit("and t2, t2, t7");
    } else {
        p.emit(format!("andi t2, t2, 0x{mask:x}"));
    }
    p.record("t2", sx(want));
}

/// Waits, at most `limit` passes, until the word at `pa` ANDed with `mask` is non-zero (`set`) or zero.
pub fn wait(p: &mut Program, pa: u32, mask: u32, set: bool, limit: u32) {
    let l = p.label();
    let out = p.label();
    li(p, "t6", K1 | pa);
    li(p, "t5", limit);
    p.place(&l);
    p.emit("lw t2, 0(t6)");
    p.emit(format!("andi t2, t2, 0x{mask:x}"));
    p.emit(format!("{} t2, zero, {out}", if set { "bne" } else { "beq" }));
    p.emit("nop");
    p.emit("addiu t5, t5, -1");
    p.emit(format!("bne t5, zero, {l}"));
    p.emit("nop");
    p.place(&out);
}

/// A delay of about `n` passes of a two-instruction loop.
pub fn delay(p: &mut Program, n: u32) {
    let l = p.label();
    li(p, "t5", n);
    p.place(&l);
    p.emit("addiu t5, t5, -1");
    p.emit(format!("bne t5, zero, {l}"));
    p.emit("nop");
}

/// Writes four words to the scratchpad at offset `at`, through the S entry.
pub fn spr_quad(p: &mut Program, at: u32, q: [u32; 4]) {
    li(p, "t6", SPR + at);
    for (i, w) in q.iter().enumerate() {
        li(p, "t7", *w);
        p.emit(format!("sw t7, {}(t6)", 4 * i));
    }
}

/// Writes four words to physical `pa` through kseg1.
pub fn mem_quad(p: &mut Program, pa: u32, q: [u32; 4]) {
    li(p, "t6", K1 | pa);
    for (i, w) in q.iter().enumerate() {
        li(p, "t7", *w);
        p.emit(format!("sw t7, {}(t6)", 4 * i));
    }
}

/// Records the four words at the scratchpad's offset `at`.
pub fn rec_spr(p: &mut Program, at: u32, want: Option<[u32; 4]>) {
    li(p, "t6", SPR + at);
    for i in 0..4 {
        p.emit(format!("lw t2, {}(t6)", 4 * i));
        p.record("t2", want.and_then(|q| sx(q[i])));
    }
}

/// Records the four words at physical `pa`.
pub fn rec_mem(p: &mut Program, pa: u32, want: Option<[u32; 4]>) {
    li(p, "t6", K1 | pa);
    for i in 0..4 {
        p.emit(format!("lw t2, {}(t6)", 4 * i));
        p.record("t2", want.and_then(|q| sx(q[i])));
    }
}

/// The pattern quadword `i`: its words (4i + j) × 01010101h, the bytes ps2autotests' interleave text prints.
pub fn pat(i: u32) -> [u32; 4] {
    core::array::from_fn(|j| ((4 * i + j as u32) & 0xFF) * 0x0101_0101)
}

/// A second pattern, distinct from `pat`, for buffers that must not be confused with it.
pub fn alt(i: u32) -> [u32; 4] {
    core::array::from_fn(|j| 0xA000_0000 | i << 8 | j as u32)
}

const CC: [u32; 4] = [0xCCCC_CCCC; 4];

/// A source chain tag's low doubleword as four words: QWC, PCE, ID, IRQ in the first, ADDR and SPR in the second
/// (EE User's Manual §5.6).
pub fn tag(id: u32, qwc: u32, addr: u32, irq: bool, pce: u32) -> [u32; 4] {
    [qwc | pce << 26 | id << 28 | u32::from(irq) << 31, addr, 0, 0]
}

pub mod reg {
    pub const D_CTRL: u32 = 0x1000_E000;
    pub const D_STAT: u32 = 0x1000_E010;
    pub const D_PCR: u32 = 0x1000_E020;
    pub const D_SQWC: u32 = 0x1000_E030;
    pub const D_RBSR: u32 = 0x1000_E040;
    pub const D_RBOR: u32 = 0x1000_E050;
    pub const D_STADR: u32 = 0x1000_E060;
    pub const D_ENABLER: u32 = 0x1000_F520;
    pub const D_ENABLEW: u32 = 0x1000_F590;
    pub const I_STAT: u32 = 0x1000_F000;
    pub const I_MASK: u32 = 0x1000_F010;
    pub const BASES: [u32; 10] =
        [0x1000_8000, 0x1000_9000, 0x1000_A000, 0x1000_B000, 0x1000_B400, 0x1000_C000, 0x1000_C400, 0x1000_C800, 0x1000_D000, 0x1000_D400];
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
    pub const fn asr0(c: usize) -> u32 {
        BASES[c] + 0x40
    }
    pub const fn asr1(c: usize) -> u32 {
        BASES[c] + 0x50
    }
    pub const fn sadr(c: usize) -> u32 {
        BASES[c] + 0x80
    }
    pub const fn t_count(n: u32) -> u32 {
        0x1000_0000 + n * 0x800
    }
    pub const fn t_mode(n: u32) -> u32 {
        0x1000_0010 + n * 0x800
    }
    pub const fn t_comp(n: u32) -> u32 {
        0x1000_0020 + n * 0x800
    }
    pub const fn t_hold(n: u32) -> u32 {
        0x1000_0030 + n * 0x800
    }
}

use reg::*;

const VIF0: usize = 0;
const VIF1: usize = 1;
const GIF: usize = 2;
const FROM_SPR: usize = 8;
const TO_SPR: usize = 9;
const STR: u32 = 0x100;
const CHAIN: u32 = 1 << 2;
const INTERLEAVE: u32 = 2 << 2;
const TTE: u32 = 1 << 6;
const TIE: u32 = 1 << 7;

/// Every S11 program starts here: interrupts off, Status cleared, the DMAC enabled with nothing pending, the INTC's
/// mask and status cleared, the timers stopped.
fn prologue(p: &mut Program) {
    p.emit("di");
    p.emit("mtc0 zero, Status");
    p.emit("sync.p");
    poke(p, D_CTRL, 0);
    for c in 0..10 {
        poke(p, chcr(c), 0);
    }
    poke(p, D_PCR, 0);
    poke(p, D_ENABLEW, 0);
    peek(p, D_STAT);
    li(p, "t7", 0x63FF_0000);
    p.emit("and t2, t2, t7");
    p.emit("ori t2, t2, 0xe3ff");
    p.emit("sw t2, 0(t6)");
    peek(p, I_MASK);
    p.emit("sw t2, 0(t6)");
    poke(p, I_STAT, 0x7FFF);
    for n in 0..4 {
        poke(p, t_mode(n), 0xC00);
    }
    poke(p, D_CTRL, 1);
}

/// Starts channel `c` with CHCR `chcr` (STR set) and waits for STR to clear.
fn kick_wait(p: &mut Program, c: usize, chcr_v: u32) {
    poke(p, chcr(c), chcr_v | STR);
    wait(p, chcr(c), STR, false, 20_000);
}

/// Sets STR in channel `c`'s CHCR as it reads, keeping ASP and TAG as §5.7 asks, and waits for STR to clear.
fn restart(p: &mut Program, c: usize) {
    peek(p, chcr(c));
    p.emit("ori t2, t2, 0x100");
    p.emit("sw t2, 0(t6)");
    wait(p, chcr(c), STR, false, 20_000);
}

fn clear_cis(p: &mut Program) {
    poke(p, D_STAT, 0x3FF);
}

/// `s11-spr`: the scratchpad channels in normal mode, SADR's and MADR's fixed bits, the scratchpad's wrap, D_STAT's
/// clearing and reversing writes, the holds (D_ENABLEW, DMAE, PCE with CDEn), CPCOND0, and the registers' widths.
pub fn spr() -> Program {
    let mut p = Program::kernel("s11-spr");
    let p = &mut p;
    prologue(p);
    const A: u32 = 0x0030_0000;
    const B: u32 = 0x0030_0100;
    for i in 0..4 {
        mem_quad(p, A + 16 * i, pat(i));
        mem_quad(p, B + 16 * i, CC);
    }
    for i in 0..8 {
        spr_quad(p, 16 * i, [0; 4]);
    }
    spr_quad(p, 0x3FF0, [0; 4]);
    // toSPR: SADR's bits above 13 and MADR's SPR bit are not kept on channels 8 and 9 (§5.9).
    poke(p, sadr(TO_SPR), 0x0001_0020);
    rec(p, sadr(TO_SPR), Some(0x20));
    poke(p, madr(TO_SPR), 0x8000_0000 | A);
    rec(p, madr(TO_SPR), Some(A));
    poke(p, qwc(TO_SPR), 2);
    kick_wait(p, TO_SPR, 0);
    rec(p, chcr(TO_SPR), Some(0));
    rec(p, madr(TO_SPR), Some(A + 0x20));
    rec(p, qwc(TO_SPR), Some(0));
    rec(p, sadr(TO_SPR), Some(0x40));
    rec_masked(p, D_STAT, 0x3FF, 0x200);
    rec_spr(p, 0x10, Some([0; 4]));
    rec_spr(p, 0x20, Some(pat(0)));
    rec_spr(p, 0x30, Some(pat(1)));
    // fromSPR: SADR's low four bits not kept; the scratchpad from 20h to memory.
    poke(p, sadr(FROM_SPR), 0x24);
    rec(p, sadr(FROM_SPR), Some(0x20));
    poke(p, madr(FROM_SPR), B);
    poke(p, qwc(FROM_SPR), 2);
    kick_wait(p, FROM_SPR, 0);
    rec(p, madr(FROM_SPR), Some(B + 0x20));
    rec(p, sadr(FROM_SPR), Some(0x40));
    rec_masked(p, D_STAT, 0x3FF, 0x300);
    rec_mem(p, B, Some(pat(0)));
    rec_mem(p, B + 16, Some(pat(1)));
    rec_mem(p, B + 32, Some(CC));
    // The scratchpad's end: SADR wraps to 0.
    poke(p, sadr(TO_SPR), 0x3FF0);
    poke(p, madr(TO_SPR), A + 0x20);
    poke(p, qwc(TO_SPR), 2);
    kick_wait(p, TO_SPR, 0);
    rec(p, sadr(TO_SPR), Some(0x10));
    rec_spr(p, 0x3FF0, Some(pat(2)));
    rec_spr(p, 0, Some(pat(3)));
    // D_STAT: CIS cleared by writing 1, CIM reversed by writing 1 (§5.8).
    poke(p, D_STAT, 0x100);
    rec_masked(p, D_STAT, 0x3FF, 0x200);
    poke(p, D_STAT, 0x200);
    rec(p, D_STAT, Some(0));
    poke(p, D_STAT, 0x0300_0000);
    rec(p, D_STAT, Some(0x0300_0000));
    poke(p, D_STAT, 0x0100_0000);
    rec(p, D_STAT, Some(0x0200_0000));
    poke(p, D_STAT, 0x0200_0000);
    rec(p, D_STAT, Some(0));
    // D_ENABLEW's CPND holds every channel, and D_ENABLER reads it (§5.8).
    poke(p, D_ENABLEW, 0x1_0000);
    rec(p, D_ENABLER, Some(0x1_0000));
    poke(p, madr(TO_SPR), A);
    poke(p, sadr(TO_SPR), 0x80);
    poke(p, qwc(TO_SPR), 1);
    poke(p, chcr(TO_SPR), STR);
    delay(p, 200);
    rec(p, qwc(TO_SPR), Some(1));
    rec(p, chcr(TO_SPR), Some(STR));
    poke(p, D_ENABLEW, 0);
    rec(p, D_ENABLER, Some(0));
    wait(p, chcr(TO_SPR), STR, false, 20_000);
    rec(p, qwc(TO_SPR), Some(0));
    rec_spr(p, 0x80, Some(pat(0)));
    clear_cis(p);
    // DMAE clear holds every channel.
    poke(p, D_CTRL, 0);
    poke(p, madr(TO_SPR), A + 0x10);
    poke(p, qwc(TO_SPR), 1);
    poke(p, chcr(TO_SPR), STR);
    delay(p, 200);
    rec(p, qwc(TO_SPR), Some(1));
    poke(p, D_CTRL, 1);
    wait(p, chcr(TO_SPR), STR, false, 20_000);
    rec(p, qwc(TO_SPR), Some(0));
    rec_spr(p, 0x90, Some(pat(1)));
    clear_cis(p);
    // PCE with CDE9 clear holds channel 9; CDE9 set lets it run (§5.8, D_PCR).
    poke(p, D_PCR, 0x8000_0000 | 1 << (16 + FROM_SPR));
    poke(p, madr(TO_SPR), A + 0x20);
    poke(p, qwc(TO_SPR), 1);
    poke(p, chcr(TO_SPR), STR);
    delay(p, 200);
    rec(p, qwc(TO_SPR), Some(1));
    poke(p, D_PCR, 0x8000_0000 | 1 << (16 + TO_SPR));
    wait(p, chcr(TO_SPR), STR, false, 20_000);
    rec(p, qwc(TO_SPR), Some(0));
    rec_spr(p, 0xA0, Some(pat(2)));
    poke(p, D_PCR, 0);
    clear_cis(p);
    // CPCOND0: true when every channel with CPCn has its CIS (§5.8, D_PCR), as ps2autotests' cpcond prints it.
    let cpcond = |p: &mut Program, want: u32| {
        let l = p.label();
        p.emit("ori t2, zero, 1");
        p.emit(format!("bc0t {l}"));
        p.emit("nop");
        p.emit("ori t2, zero, 0");
        p.place(&l);
        p.record("t2", Some(u64::from(want)));
    };
    cpcond(p, 1);
    poke(p, D_PCR, 1 << TO_SPR);
    cpcond(p, 0);
    poke(p, madr(TO_SPR), A);
    poke(p, qwc(TO_SPR), 1);
    kick_wait(p, TO_SPR, 0);
    cpcond(p, 1);
    poke(p, D_PCR, 0);
    cpcond(p, 1);
    poke(p, D_PCR, 1 << TO_SPR);
    cpcond(p, 1);
    clear_cis(p);
    cpcond(p, 0);
    poke(p, D_PCR, 0);
    // The registers' widths and the absent ones (§5.8, §5.9).
    poke(p, qwc(TO_SPR), 0x0001_2345);
    rec(p, qwc(TO_SPR), Some(0x2345));
    poke(p, qwc(TO_SPR), 0);
    poke(p, D_SQWC, 0xFFFF_FFFF);
    rec(p, D_SQWC, Some(0x00FF_00FF));
    poke(p, D_SQWC, 0);
    poke(p, chcr(3) + 0x30, 0x1234_5670);
    rec(p, chcr(3) + 0x30, Some(0));
    poke(p, chcr(FROM_SPR) + 0x30, 0x1234_5670);
    rec(p, chcr(FROM_SPR) + 0x30, Some(0));
    poke(p, sadr(VIF0), 0x1230);
    rec(p, sadr(VIF0), Some(0));
    p.emit("nop");
    std::mem::replace(p, Program::new("s11-spr"))
}

/// `s11-chain`: toSPR's source chain walked one tag at a time by tag interrupts (cnt, next, ref, refs, cnt with PCE
/// set, refe with PCE cleared), IRQ without TIE ignored, the resumption of a packet CHCR.TAG names, call on a channel
/// without an address stack; fromSPR's destination chain (cnt, cnts updating D_STADR, end) and an undefined tag.
pub fn chain() -> Program {
    let mut p = Program::kernel("s11-chain");
    let p = &mut p;
    prologue(p);
    const T: u32 = 0x0031_0000;
    const D: u32 = 0x0031_8000;
    for i in 0..8 {
        mem_quad(p, D + 16 * i, alt(i));
    }
    mem_quad(p, T, tag(1, 2, 0, true, 0));
    mem_quad(p, T + 0x10, pat(0));
    mem_quad(p, T + 0x20, pat(1));
    mem_quad(p, T + 0x30, tag(2, 1, T + 0x100, true, 0));
    mem_quad(p, T + 0x40, pat(2));
    mem_quad(p, T + 0x100, tag(3, 2, D, true, 0));
    mem_quad(p, T + 0x110, tag(4, 1, D + 0x40, true, 0));
    mem_quad(p, T + 0x120, tag(1, 0, 0, true, 3));
    mem_quad(p, T + 0x130, tag(0, 1, D + 0x50, false, 2));
    for i in 0..10 {
        spr_quad(p, 0x100 + 16 * i, [0; 4]);
    }
    poke(p, D_PCR, 1 << (16 + TO_SPR));
    poke(p, tadr(TO_SPR), T);
    poke(p, sadr(TO_SPR), 0x100);
    poke(p, qwc(TO_SPR), 0);
    // (tag bits 31..16, MADR, TADR, SADR) after each stop, by §5.2.3's table and §5.4's tag interrupt.
    let stops: [(u32, u32, u32, u32); 6] = [
        (0x9000, T + 0x30, T + 0x30, 0x120),
        (0xA000, T + 0x50, T + 0x100, 0x130),
        (0xB000, D + 0x20, T + 0x110, 0x150),
        (0xC000, D + 0x50, T + 0x120, 0x160),
        (0x9C00, T + 0x130, T + 0x130, 0x160),
        (0x0800, D + 0x60, T + 0x140, 0x170),
    ];
    for (k, (t, m, ta, s)) in stops.iter().enumerate() {
        kick_wait(p, TO_SPR, CHAIN | TIE);
        rec(p, chcr(TO_SPR), Some(t << 16 | CHAIN | TIE));
        rec(p, madr(TO_SPR), Some(*m));
        rec(p, qwc(TO_SPR), Some(0));
        rec(p, tadr(TO_SPR), Some(*ta));
        rec(p, sadr(TO_SPR), Some(*s));
        rec_masked(p, D_STAT, 0x3FF, 0x200);
        if k == 4 {
            rec(p, D_PCR, Some(0x8000_0000 | 1 << (16 + TO_SPR)));
        }
        clear_cis(p);
    }
    rec(p, D_PCR, Some(1 << (16 + TO_SPR)));
    for (i, q) in [pat(0), pat(1), pat(2), alt(0), alt(1), alt(4), alt(5)].iter().enumerate() {
        rec_spr(p, 0x100 + 16 * i as u32, Some(*q));
    }
    rec_spr(p, 0x170, Some([0; 4]));
    // IRQ without TIE: the chain runs to its end (§5.4; ps2autotests' tagintr).
    poke(p, tadr(TO_SPR), T + 0x100);
    poke(p, sadr(TO_SPR), 0x200);
    kick_wait(p, TO_SPR, CHAIN);
    rec(p, chcr(TO_SPR), Some(0x0800 << 16 | CHAIN));
    rec(p, tadr(TO_SPR), Some(T + 0x140));
    rec(p, madr(TO_SPR), Some(D + 0x60));
    rec(p, sadr(TO_SPR), Some(0x240));
    rec_masked(p, D_STAT, 0x3FF, 0x200);
    rec(p, D_PCR, Some(1 << (16 + TO_SPR)));
    poke(p, D_PCR, 0);
    clear_cis(p);
    // A start with QWC left moves that packet first and acts on CHCR.TAG (§5.2.3's first paragraph): end stops after it,
    // ref reads the next tag at TADR.
    poke(p, madr(TO_SPR), D + 0x70);
    poke(p, qwc(TO_SPR), 1);
    poke(p, sadr(TO_SPR), 0x300);
    poke(p, tadr(TO_SPR), T + 0x130);
    kick_wait(p, TO_SPR, 0x7000_0000 | CHAIN);
    rec(p, madr(TO_SPR), Some(D + 0x80));
    rec(p, sadr(TO_SPR), Some(0x310));
    rec_spr(p, 0x300, Some(alt(7)));
    clear_cis(p);
    poke(p, madr(TO_SPR), D + 0x60);
    poke(p, qwc(TO_SPR), 1);
    poke(p, tadr(TO_SPR), T + 0x130);
    kick_wait(p, TO_SPR, 0x3000_0000 | CHAIN);
    rec(p, madr(TO_SPR), Some(D + 0x60));
    rec(p, sadr(TO_SPR), Some(0x330));
    rec(p, tadr(TO_SPR), Some(T + 0x140));
    rec_spr(p, 0x310, Some(alt(6)));
    rec_spr(p, 0x320, Some(alt(5)));
    clear_cis(p);
    // TTE on toSPR: each tag's quadword is written to the scratchpad before its packet [A, Sapphire_Disputes.md D-36].
    mem_quad(p, T + 0x200, [0x7000_0001, 0x1111_1111, 0x2222_2222, 0x3333_3333]);
    mem_quad(p, T + 0x210, alt(3));
    poke(p, tadr(TO_SPR), T + 0x200);
    poke(p, sadr(TO_SPR), 0x400);
    kick_wait(p, TO_SPR, CHAIN | TTE);
    rec(p, sadr(TO_SPR), Some(0x420));
    rec_spr(p, 0x400, Some([0x7000_0001, 0x1111_1111, 0x2222_2222, 0x3333_3333]));
    rec_spr(p, 0x410, Some(alt(3)));
    clear_cis(p);
    // call on toSPR, which has no address stack (§5.2.3: "Effective only on the VIF0, VIF1, and GIF channels"): read
    // as an undefined tag, the transfer ended with TADR at the tag [A, D-37].
    mem_quad(p, T + 0x300, tag(5, 1, T + 0x400, false, 0));
    poke(p, tadr(TO_SPR), T + 0x300);
    poke(p, sadr(TO_SPR), 0x500);
    kick_wait(p, TO_SPR, CHAIN);
    rec(p, chcr(TO_SPR), Some(0x5000 << 16 | CHAIN));
    rec(p, tadr(TO_SPR), Some(T + 0x300));
    rec(p, sadr(TO_SPR), Some(0x500));
    rec_masked(p, D_STAT, 0x3FF, 0x200);
    clear_cis(p);
    // fromSPR's destination chain, its tags in the scratchpad (§5.2.4); cnts copies MADR to D_STADR with STS fromSPR.
    const M: u32 = 0x0032_0000;
    for i in 0..0x30 {
        mem_quad(p, M + 16 * i, CC);
    }
    spr_quad(p, 0x600, tag(1, 2, M, true, 0));
    spr_quad(p, 0x610, alt(0));
    spr_quad(p, 0x620, alt(1));
    spr_quad(p, 0x630, tag(0, 1, M + 0x100, false, 0));
    spr_quad(p, 0x640, alt(2));
    spr_quad(p, 0x650, tag(7, 1, M + 0x200, false, 0));
    spr_quad(p, 0x660, alt(3));
    poke(p, D_CTRL, 1 | 2 << 4);
    poke(p, D_STADR, 0);
    poke(p, sadr(FROM_SPR), 0x600);
    poke(p, qwc(FROM_SPR), 0);
    kick_wait(p, FROM_SPR, CHAIN | TIE);
    rec(p, chcr(FROM_SPR), Some(0x9000 << 16 | CHAIN | TIE));
    rec(p, madr(FROM_SPR), Some(M + 0x20));
    rec(p, sadr(FROM_SPR), Some(0x630));
    rec(p, D_STADR, Some(0));
    rec_masked(p, D_STAT, 0x3FF, 0x100);
    clear_cis(p);
    kick_wait(p, FROM_SPR, CHAIN | TIE);
    rec(p, chcr(FROM_SPR), Some(0x7000 << 16 | CHAIN | TIE));
    rec(p, madr(FROM_SPR), Some(M + 0x210));
    rec(p, sadr(FROM_SPR), Some(0x670));
    rec(p, D_STADR, Some(M + 0x110));
    rec_masked(p, D_STAT, 0x3FF, 0x100);
    clear_cis(p);
    rec_mem(p, M, Some(alt(0)));
    rec_mem(p, M + 0x10, Some(alt(1)));
    rec_mem(p, M + 0x20, Some(CC));
    rec_mem(p, M + 0x100, Some(alt(2)));
    rec_mem(p, M + 0x200, Some(alt(3)));
    rec_mem(p, M + 0x210, Some(CC));
    // An undefined destination tag (ID 3): the transfer ends, CHCR keeping the tag's bits (§5.4).
    spr_quad(p, 0x700, tag(3, 1, M + 0x280, false, 0));
    poke(p, sadr(FROM_SPR), 0x700);
    kick_wait(p, FROM_SPR, CHAIN);
    rec(p, chcr(FROM_SPR), Some(0x3000 << 16 | CHAIN));
    rec(p, sadr(FROM_SPR), Some(0x710));
    rec_masked(p, D_STAT, 0x3FF, 0x100);
    rec_mem(p, M + 0x280, Some(CC));
    clear_cis(p);
    poke(p, D_CTRL, 1);
    std::mem::replace(p, Program::new("s11-chain"))
}

/// `s11-callret`: VIF0's source chain with call and ret, walked by tag interrupts: two calls filling ASR0 and ASR1,
/// a third refused with the stack full, two rets popping them, and a ret with nothing pushed ending the chain. VIF0
/// receives only zeros, VIFcode NOPs.
pub fn callret() -> Program {
    let mut p = Program::kernel("s11-callret");
    let p = &mut p;
    prologue(p);
    const T: u32 = 0x0033_0000;
    mem_quad(p, T, tag(5, 1, T + 0x1000, true, 0));
    mem_quad(p, T + 0x10, [0; 4]);
    mem_quad(p, T + 0x20, tag(6, 1, 0, false, 0));
    mem_quad(p, T + 0x30, [0; 4]);
    mem_quad(p, T + 0x1000, tag(5, 0, T + 0x2000, true, 0));
    mem_quad(p, T + 0x1010, tag(6, 0, 0, true, 0));
    mem_quad(p, T + 0x2000, tag(5, 1, T + 0x3000, true, 0));
    mem_quad(p, T + 0x2010, tag(6, 1, 0, true, 0));
    mem_quad(p, T + 0x2020, [0; 4]);
    poke(p, tadr(VIF0), T);
    poke(p, qwc(VIF0), 0);
    poke(p, asr0(VIF0), 0);
    poke(p, asr1(VIF0), 0);
    // (CHCR without STR, MADR, TADR, ASR0, ASR1) after each stop.
    let base = CHAIN | TIE;
    let stops: [(u32, u32, u32, u32, u32); 3] = [
        (0xD000 << 16 | 1 << 4 | base, T + 0x20, T + 0x1000, T + 0x20, 0),
        (0xD000 << 16 | 2 << 4 | base, T + 0x1010, T + 0x2000, T + 0x20, T + 0x1010),
        (0xD000 << 16 | 2 << 4 | base, T + 0x1010, T + 0x2000, T + 0x20, T + 0x1010),
    ];
    poke(p, chcr(VIF0), base);
    for (c, m, t, a0, a1) in stops {
        restart(p, VIF0);
        rec(p, chcr(VIF0), Some(c));
        rec(p, madr(VIF0), Some(m));
        rec(p, qwc(VIF0), Some(0));
        rec(p, tadr(VIF0), Some(t));
        rec(p, asr0(VIF0), Some(a0));
        rec(p, asr1(VIF0), Some(a1));
        rec_masked(p, D_STAT, 0x3FF, 1);
        clear_cis(p);
    }
    poke(p, tadr(VIF0), T + 0x2010);
    let stops: [(u32, u32, u32); 3] = [
        (0xE000 << 16 | 1 << 4 | base, T + 0x2030, T + 0x1010),
        (0xE000 << 16 | base, T + 0x1020, T + 0x20),
        (0x6000 << 16 | base, T + 0x40, T + 0x40),
    ];
    for (c, m, t) in stops {
        restart(p, VIF0);
        rec(p, chcr(VIF0), Some(c));
        rec(p, madr(VIF0), Some(m));
        rec(p, qwc(VIF0), Some(0));
        rec(p, tadr(VIF0), Some(t));
        rec_masked(p, D_STAT, 0x3FF, 1);
        clear_cis(p);
    }
    std::mem::replace(p, Program::new("s11-callret"))
}

/// The interleave cases ps2autotests' `dma/spr/interleave` prints, as (transfer, skip, count): the scratchpad side
/// moves `count` qwords, the memory side `count / transfer` blocks of `transfer` then `skip` qwords (§5.2.5).
pub const INTERLEAVE_CASES: [(u32, u32, u32); 8] = [(1, 1, 3), (1, 3, 3), (2, 1, 4), (3, 3, 9), (1, 14, 2), (1, 256, 2), (257, 1, 3), (1, 255, 4)];

/// MADR's advance for a case: every block and its skip, the last skip included (the console text's "MADR did not
/// increase as expected" lines), with TQWC and SQWC their low eight bits (D_SQWC).
pub fn interleave_advance(t: u32, s: u32, count: u32) -> u32 {
    let (t, s) = (t & 0xFF, s & 0xFF);
    count / t * (t + s) * 16
}

/// `s11-interleave`: toSPR and fromSPR in interleave mode over the console text's cases.
pub fn interleave() -> Program {
    let mut p = Program::kernel("s11-interleave");
    let p = &mut p;
    prologue(p);
    const SRC: u32 = 0x0034_0000;
    for i in 0..48 {
        mem_quad(p, SRC + 16 * i, pat(i));
    }
    for &(t, s, count) in &INTERLEAVE_CASES {
        for i in 0..count + 1 {
            spr_quad(p, 16 * i, CC);
        }
        poke(p, D_SQWC, t << 16 | s);
        poke(p, madr(TO_SPR), SRC);
        poke(p, sadr(TO_SPR), 0);
        poke(p, qwc(TO_SPR), count);
        kick_wait(p, TO_SPR, INTERLEAVE);
        rec(p, madr(TO_SPR), Some(SRC + interleave_advance(t, s, count)));
        rec(p, sadr(TO_SPR), Some(16 * count));
        let (tq, sq) = (t & 0xFF, s & 0xFF);
        for i in 0..count {
            let src = i / tq * (tq + sq) + i % tq;
            rec_spr(p, 16 * i, if src < 48 { Some(pat(src)) } else { None });
        }
        rec_spr(p, 16 * count, Some(CC));
        clear_cis(p);
    }
    const DST: u32 = 0x0034_8000;
    for i in 0..16 {
        spr_quad(p, 16 * i, pat(i));
    }
    for &(t, s, count) in &[(1u32, 1u32, 3u32), (2, 2, 6)] {
        let span = count / t * (t + s);
        for i in 0..span + 1 {
            mem_quad(p, DST + 16 * i, CC);
        }
        poke(p, D_SQWC, t << 16 | s);
        poke(p, madr(FROM_SPR), DST);
        poke(p, sadr(FROM_SPR), 0);
        poke(p, qwc(FROM_SPR), count);
        kick_wait(p, FROM_SPR, INTERLEAVE);
        rec(p, madr(FROM_SPR), Some(DST + interleave_advance(t, s, count)));
        rec(p, sadr(FROM_SPR), Some(16 * count));
        for i in 0..span + 1 {
            let (b, r) = (i / (t + s), i % (t + s));
            let want = if i < span && r < t { pat(b * t + r) } else { CC };
            rec_mem(p, DST + 16 * i, Some(want));
        }
        clear_cis(p);
    }
    poke(p, D_SQWC, 0);
    std::mem::replace(p, Program::new("s11-interleave"))
}

/// `s11-stall`: stall control with fromSPR as the source and VIF1 as the drain, both in normal mode (§5.3.2): the drain
/// held while fewer than eight qwords lie between its MADR and D_STADR, SIS set, resumed as fromSPR moves D_STADR on,
/// and released by STD cleared.
pub fn stall() -> Program {
    let mut p = Program::kernel("s11-stall");
    let p = &mut p;
    prologue(p);
    const BUF: u32 = 0x0035_0000;
    for i in 0..32 {
        spr_quad(p, 16 * i, [0; 4]);
    }
    poke(p, D_CTRL, 1 | 2 << 4 | 1 << 6);
    poke(p, D_STADR, BUF);
    poke(p, madr(VIF1), BUF);
    poke(p, qwc(VIF1), 20);
    poke(p, chcr(VIF1), STR | 1);
    delay(p, 300);
    rec_masked(p, D_STAT, 0x2000, 0x2000);
    rec(p, chcr(VIF1), Some(STR | 1));
    rec(p, madr(VIF1), Some(BUF));
    rec(p, qwc(VIF1), Some(20));
    poke(p, D_STAT, 0x2000);
    let from_spr = |p: &mut Program, at: u32, sadr_v: u32, n: u32| {
        poke(p, madr(FROM_SPR), at);
        poke(p, sadr(FROM_SPR), sadr_v);
        poke(p, qwc(FROM_SPR), n);
        kick_wait(p, FROM_SPR, 0);
    };
    from_spr(p, BUF, 0, 12);
    rec(p, D_STADR, Some(BUF + 0xC0));
    delay(p, 300);
    rec(p, madr(VIF1), Some(BUF + 0x80));
    rec(p, qwc(VIF1), Some(12));
    rec(p, chcr(VIF1), Some(STR | 1));
    rec_masked(p, D_STAT, 0x2000, 0x2000);
    from_spr(p, BUF + 0xC0, 0xC0, 12);
    rec(p, D_STADR, Some(BUF + 0x180));
    wait(p, chcr(VIF1), STR, false, 20_000);
    rec(p, madr(VIF1), Some(BUF + 0x140));
    rec(p, qwc(VIF1), Some(0));
    rec(p, chcr(VIF1), Some(1));
    poke(p, D_STAT, 0x2000);
    rec_masked(p, D_STAT, 0x2000, 0);
    rec_masked(p, D_STAT, 0x3FF, 0x102);
    clear_cis(p);
    // Released by STD cleared, as §5.3.2's "Normal to Normal" example clears it at the source's end.
    poke(p, madr(VIF1), BUF + 0x200);
    poke(p, qwc(VIF1), 8);
    poke(p, chcr(VIF1), STR | 1);
    delay(p, 300);
    rec(p, qwc(VIF1), Some(8));
    rec_masked(p, D_STAT, 0x2000, 0x2000);
    poke(p, D_CTRL, 1 | 2 << 4);
    wait(p, chcr(VIF1), STR, false, 20_000);
    rec(p, qwc(VIF1), Some(0));
    rec(p, madr(VIF1), Some(BUF + 0x280));
    poke(p, D_STAT, 0x23FF);
    poke(p, D_CTRL, 1);
    std::mem::replace(p, Program::new("s11-stall"))
}

/// The MFIFO's ring: 64 qwords at 360000h.
pub const RING: u32 = 0x0036_0000;

/// `s11-mfifo`: fromSPR filling a ring of 64 qwords and VIF1 draining it in source chain mode (§5.5): the drain held
/// with MEIS while its TADR equals fromSPR's MADR, both addresses wrapping, a packet's data across the wrap, and the
/// chain's end.
pub fn mfifo() -> Program {
    let mut p = Program::kernel("s11-mfifo");
    let p = &mut p;
    prologue(p);
    let mut stream: Vec<[u32; 4]> = Vec::new();
    let mut packet = |n: u32, id: u32| {
        stream.push(tag(id, n, 0, false, 0));
        for _ in 0..n {
            stream.push([0; 4]);
        }
    };
    for _ in 0..5 {
        packet(7, 1);
    }
    packet(9, 1);
    packet(9, 1);
    packet(9, 1);
    packet(8, 1);
    packet(0, 7);
    assert_eq!(stream.len(), 80);
    for (i, q) in stream.iter().enumerate() {
        spr_quad(p, 16 * i as u32, *q);
    }
    for i in 0..64 {
        mem_quad(p, RING + 16 * i, CC);
    }
    poke(p, D_CTRL, 1 | 2 << 2);
    poke(p, D_RBOR, RING);
    poke(p, D_RBSR, 0x3F0);
    poke(p, madr(FROM_SPR), RING);
    poke(p, tadr(VIF1), RING);
    poke(p, qwc(VIF1), 0);
    poke(p, chcr(VIF1), STR | CHAIN | 1);
    delay(p, 300);
    rec_masked(p, D_STAT, 0x4000, 0x4000);
    rec(p, chcr(VIF1), Some(STR | CHAIN | 1));
    rec(p, tadr(VIF1), Some(RING));
    poke(p, sadr(FROM_SPR), 0);
    poke(p, qwc(FROM_SPR), 40);
    kick_wait(p, FROM_SPR, 0);
    rec(p, madr(FROM_SPR), Some(RING + 0x280));
    delay(p, 300);
    rec_masked(p, D_STAT, 0x4000, 0x4000);
    rec(p, tadr(VIF1), Some(RING + 0x280));
    rec(p, madr(VIF1), Some(RING + 0x280));
    rec(p, qwc(VIF1), Some(0));
    rec(p, chcr(VIF1), Some(0x1000 << 16 | STR | CHAIN | 1));
    poke(p, sadr(FROM_SPR), 0x280);
    poke(p, qwc(FROM_SPR), 40);
    kick_wait(p, FROM_SPR, 0);
    rec(p, madr(FROM_SPR), Some(RING + 0x100));
    wait(p, chcr(VIF1), STR, false, 20_000);
    rec(p, chcr(VIF1), Some(0x7000 << 16 | CHAIN | 1));
    rec(p, tadr(VIF1), Some(RING + 0x100));
    rec(p, madr(VIF1), Some(RING + 0x100));
    poke(p, D_STAT, 0x4000);
    rec_masked(p, D_STAT, 0x4000, 0);
    rec_masked(p, D_STAT, 0x3FF, 0x102);
    rec_mem(p, RING, Some(stream[64]));
    rec_mem(p, RING + 0xF0, Some(stream[79]));
    rec_mem(p, RING + 0x3F0, Some(stream[63]));
    rec_mem(p, RING + 0x100, Some(stream[16]));
    poke(p, D_STAT, 0x43FF);
    poke(p, D_CTRL, 1);
    std::mem::replace(p, Program::new("s11-mfifo"))
}

/// The interrupt handler at 80000200h: logs Cause's IP3, IP2 and ExcCode, I_STAT and D_STAT at s3, masks every
/// source it finds enabled (I_MASK and D_STAT's masks reversed, BEIS cleared) and returns.
fn interrupt_handler() -> String {
    [
        "mfc0 k0, Cause",
        "ori k1, zero, 0x0c7c",
        "and k0, k0, k1",
        "sw k0, 0(s3)",
        "lui k1, 0xb000",
        "ori k1, k1, 0xf000",
        "lw k0, 0(k1)",
        "sw k0, 4(s3)",
        "lw k0, 0x10(k1)",
        "sw k0, 0x10(k1)",
        "lui k1, 0xb000",
        "ori k1, k1, 0xe010",
        "lw k0, 0(k1)",
        "sw k0, 8(s3)",
        "and k0, k0, s4",
        "ori k0, k0, 0x8000",
        "sw k0, 0(k1)",
        "addiu s3, s3, 16",
        "sync.p",
        "eret",
    ]
    .join("\n")
}

/// Writes the handler at 80000200h through kseg1 and invalidates the instruction cache, as S7's programs do.
fn install_handler(p: &mut Program) {
    let words = emusen_mips::assemble(&interrupt_handler(), 0x8000_0200, &emusen_mips::Dialect::R5900).unwrap_or_else(|e| panic!("{e:?}"));
    li(p, "t9", 0xA000_0200);
    for (i, w) in words.iter().enumerate() {
        li(p, "t3", *w);
        p.emit(format!("sw t3, {}(t9)", 4 * i));
    }
    let l = p.label();
    p.emit("ori t0, zero, 0");
    p.emit("ori t1, zero, 128");
    p.place(&l);
    for line in ["sync.p", "cache 0x07, 0(t0)", "sync.p", "cache 0x07, 1(t0)", "sync.p"] {
        p.emit(line);
    }
    p.emit("addiu t1, t1, -1");
    p.emit(format!("bne t1, zero, {l}"));
    p.emit("addiu t0, t0, 64");
    li(p, "s3", K1 | LOG);
    li(p, "s4", 0x63FF_0000);
}

/// Records the handler's entries so far: their count, then each entry's Cause, I_STAT and D_STAT, ANDed with masks.
fn rec_log(p: &mut Program, entries: &[(u32, u32, u32)], stat_mask: u32, dstat_mask: u32) {
    li(p, "t6", K1 | LOG);
    p.emit("subu t2, s3, t6");
    p.emit("srl t2, t2, 4");
    p.record("t2", Some(entries.len() as u64));
    for (k, (cause, istat, dstat)) in entries.iter().enumerate() {
        let at = 16 * k as u32;
        p.emit(format!("lw t2, {at}(t6)"));
        p.record("t2", sx(*cause));
        p.emit(format!("lw t2, {}(t6)", at + 4));
        p.emit(format!("andi t2, t2, 0x{stat_mask:x}"));
        p.record("t2", sx(*istat));
        p.emit(format!("lw t2, {}(t6)", at + 8));
        li(p, "t7", dstat_mask);
        p.emit("and t2, t2, t7");
        p.record("t2", sx(*dstat));
    }
}

const IE_EIE_IM23: u32 = 1 | 1 << 16 | 1 << 10 | 1 << 11;

/// `s11-intc`: I_MASK's reversing and I_STAT's clearing writes; a timer's compare latched in I_STAT and held by its
/// mask, then taken as the core's interrupt once unmasked; the DMAC's end of transfer taken through D_STAT's mask;
/// both lines at once; V-blank's start and end (Sapphire_Disputes.md D-15 by which Cause bit each line sets).
pub fn intc() -> Program {
    let mut p = Program::kernel("s11-intc");
    let p = &mut p;
    prologue(p);
    install_handler(p);
    poke(p, I_MASK, 0x0204);
    rec(p, I_MASK, Some(0x0204));
    poke(p, I_MASK, 0x0004);
    rec(p, I_MASK, Some(0x0200));
    poke(p, I_MASK, 0x0200);
    rec(p, I_MASK, Some(0));
    poke(p, I_STAT, 0x7FFF);
    rec_masked(p, I_STAT, 0x7FF3, 0);
    li(p, "t0", IE_EIE_IM23);
    p.emit("mtc0 t0, Status");
    p.emit("sync.p");
    // Timer 0's compare, latched in I_STAT and held while I_MASK's bit is clear.
    poke(p, t_count(0), 0);
    poke(p, t_comp(0), 0x200);
    poke(p, t_mode(0), 0x180);
    wait(p, I_STAT, 0x200, true, 20_000);
    rec_masked(p, I_STAT, 0x200, 0x200);
    rec_masked(p, t_mode(0), 0xC00, 0x400);
    rec_log(p, &[], 0x7FF3, 0);
    // Unmasked: INT0, Cause.IP2.
    poke(p, I_MASK, 0x200);
    p.emit("nop");
    p.emit("nop");
    rec_log(p, &[(0x400, 0x200, 0)], 0x7FF3, 0x63FF_E3FF);
    rec(p, I_MASK, Some(0));
    poke(p, I_STAT, 0x200);
    poke(p, t_mode(0), 0xC00);
    // The DMAC's end of transfer with CIM9: INT1, Cause.IP3.
    poke(p, D_STAT, 1 << (16 + TO_SPR));
    poke(p, madr(TO_SPR), 0x0030_0000);
    poke(p, sadr(TO_SPR), 0);
    poke(p, qwc(TO_SPR), 1);
    poke(p, chcr(TO_SPR), STR);
    wait(p, chcr(TO_SPR), STR, false, 20_000);
    p.emit("nop");
    rec_log(p, &[(0x400, 0x200, 0), (0x800, 0, 0x0200_0200)], 0x7FF3, 0x63FF_E3FF);
    rec(p, D_STAT, Some(0x200));
    clear_cis(p);
    // Both lines pending while IE is clear, then one interrupt with IP2 and IP3 set.
    p.emit("mtc0 zero, Status");
    p.emit("sync.p");
    poke(p, t_count(0), 0);
    poke(p, t_mode(0), 0x180);
    wait(p, I_STAT, 0x200, true, 20_000);
    poke(p, I_MASK, 0x200);
    poke(p, D_STAT, 1 << (16 + TO_SPR));
    poke(p, qwc(TO_SPR), 1);
    poke(p, chcr(TO_SPR), STR);
    wait(p, chcr(TO_SPR), STR, false, 20_000);
    li(p, "t0", IE_EIE_IM23);
    p.emit("mtc0 t0, Status");
    p.emit("sync.p");
    p.emit("nop");
    p.emit("nop");
    rec_log(p, &[(0x400, 0x200, 0), (0x800, 0, 0x0200_0200), (0xC00, 0x200, 0x0200_0200)], 0x7FF3, 0x63FF_E3FF);
    p.emit("mtc0 zero, Status");
    p.emit("sync.p");
    poke(p, t_mode(0), 0xC00);
    poke(p, I_STAT, 0x7FFF);
    clear_cis(p);
    // V-blank's start and end, the stand-in's edges (§3.1: VB_ON on V-blank's rising edge, VB_OFF on its falling).
    poke(p, I_STAT, 0xC);
    wait(p, I_STAT, 0x4, true, 2_000_000);
    rec_masked(p, I_STAT, 0x4, 0x4);
    p.emit("mfc0 a0, Count");
    poke(p, I_STAT, 0xC);
    wait(p, I_STAT, 0x8, true, 2_000_000);
    p.emit("mfc0 a1, Count");
    rec_masked(p, I_STAT, 0xC, 0x8);
    p.emit("subu t2, a1, a0");
    p.record("t2", None);
    std::mem::replace(p, Program::new("s11-intc"))
}

/// Records whether `cond` (a sequence leaving 1 or 0 in t2) holds, expecting `want`.
fn rec_bool(p: &mut Program, want: u32) {
    p.record("t2", Some(u64::from(want)));
}

/// `s11-timers`: the dividers' ratios, CUE, COUNT's and MODE's writes, the compare and overflow flags with and without
/// their enables and their edges into I_STAT, ZRET, the V-blank gate in modes 0 and 1, H-blank as a clock, HOLD's
/// width (EE User's Manual chapter 4).
pub fn timers() -> Program {
    let mut p = Program::kernel("s11-timers");
    let p = &mut p;
    prologue(p);
    let read = |p: &mut Program, n: u32, dst: &str| {
        li(p, "t6", K1 | t_count(n));
        p.emit(format!("lw {dst}, 0(t6)"));
    };
    // The dividers: T0 on BUSCLK, T1 on BUSCLK/16, T2 on BUSCLK/256, read after the same span.
    for n in 0..3 {
        poke(p, t_count(n), 0);
    }
    poke(p, t_mode(2), 0x82);
    poke(p, t_mode(1), 0x81);
    poke(p, t_mode(0), 0x80);
    delay(p, 15_000);
    li(p, "t6", K1 | t_count(0));
    p.emit("lw a0, 0(t6)");
    p.emit("lw a1, 0x800(t6)");
    p.emit("lw a2, 0x1000(t6)");
    p.record("a0", None);
    p.record("a1", None);
    p.record("a2", None);
    for (hi, lo) in [("a0", "a1"), ("a1", "a2")] {
        p.emit(format!("srl t2, {hi}, 4"));
        p.emit(format!("subu t2, t2, {lo}"));
        p.emit("addiu t2, t2, 2");
        p.emit("sltiu t2, t2, 5");
        rec_bool(p, 1);
    }
    p.emit("sltiu t2, a2, 16");
    p.emit("xori t2, t2, 1");
    rec_bool(p, 1);
    // CUE clear holds the count.
    poke(p, t_mode(0), 0);
    read(p, 0, "a0");
    delay(p, 500);
    read(p, 0, "a1");
    p.emit("subu t2, a1, a0");
    p.record("t2", Some(0));
    // COUNT's write, and MODE's write leaving COUNT as it is [A, Sapphire_Disputes.md D-39].
    poke(p, t_count(0), 0x1_1234);
    rec(p, t_count(0), Some(0x1234));
    poke(p, t_mode(0), 0x80);
    read(p, 0, "a0");
    p.emit("addiu t2, a0, -0x1234");
    p.emit("sltiu t2, t2, 0x100");
    rec_bool(p, 1);
    poke(p, t_mode(0), 0);
    // EQUF with CMPE, its edge into I_STAT, cleared by writing 1.
    poke(p, I_STAT, 0x7FFF);
    poke(p, t_count(1), 0);
    poke(p, t_comp(1), 0x40);
    poke(p, t_mode(1), 0x180);
    wait(p, t_mode(1), 0x400, true, 20_000);
    rec(p, t_mode(1), Some(0x580));
    rec_masked(p, I_STAT, 0x400, 0x400);
    poke(p, t_mode(1), 0x580);
    rec_masked(p, t_mode(1), 0xC00, 0);
    // A flag already set gives no new edge: I_STAT stays clear across a second match.
    poke(p, I_STAT, 0x400);
    poke(p, t_mode(1), 0);
    poke(p, t_count(1), 0);
    poke(p, t_mode(1), 0x180);
    wait(p, t_mode(1), 0x400, true, 20_000);
    poke(p, I_STAT, 0x400);
    poke(p, t_count(1), 0);
    delay(p, 200);
    rec_masked(p, t_mode(1), 0x400, 0x400);
    rec_masked(p, I_STAT, 0x400, 0);
    poke(p, t_mode(1), 0xD80);
    poke(p, t_count(1), 0);
    wait(p, t_mode(1), 0x400, true, 20_000);
    rec_masked(p, I_STAT, 0x400, 0x400);
    // No EQUF without CMPE.
    poke(p, t_mode(1), 0xC00);
    poke(p, t_count(1), 0);
    poke(p, t_comp(1), 0x10);
    poke(p, t_mode(1), 0x80);
    delay(p, 200);
    rec(p, t_mode(1), Some(0x80));
    // ZRET: on BUSCLK/16 with COMP 8, the count reaches 8 and returns to 0 [A, D-38: COMP is visible for a tick].
    poke(p, t_mode(1), 0);
    poke(p, t_count(1), 0);
    poke(p, t_comp(1), 8);
    poke(p, t_mode(1), 0xC1);
    li(p, "t6", K1 | t_count(1));
    p.emit("ori a1, zero, 0");
    p.emit("ori t5, zero, 600");
    let l = p.label();
    let skip = p.label();
    p.place(&l);
    p.emit("lw a0, 0(t6)");
    p.emit("sltu t2, a1, a0");
    p.emit(format!("beq t2, zero, {skip}"));
    p.emit("nop");
    p.emit("or a1, a0, zero");
    p.place(&skip);
    p.emit("addiu t5, t5, -1");
    p.emit(format!("bne t5, zero, {l}"));
    p.emit("nop");
    p.record("a1", Some(8));
    poke(p, t_mode(1), 0);
    // OVFF with OVFE and its edge; none without OVFE.
    poke(p, I_STAT, 0x7FFF);
    poke(p, t_count(2), 0xFF00);
    poke(p, t_mode(2), 0x280);
    wait(p, t_mode(2), 0x800, true, 20_000);
    rec(p, t_mode(2), Some(0xA80));
    rec_masked(p, I_STAT, 0x800, 0x800);
    read(p, 2, "t2");
    p.emit("sltiu t2, t2, 0x200");
    rec_bool(p, 1);
    poke(p, t_mode(2), 0xC00);
    poke(p, t_count(2), 0xFFF0);
    poke(p, t_mode(2), 0x80);
    delay(p, 300);
    rec(p, t_mode(2), Some(0x80));
    read(p, 2, "t2");
    li(p, "t7", 0xFF00);
    p.emit("sltu t2, t2, t7");
    rec_bool(p, 1);
    poke(p, t_mode(2), 0);
    // MODE's writable bits; HOLD's width; no HOLD on timer 2.
    poke(p, t_comp(2), 0xFFFF);
    poke(p, t_mode(2), 0xFFFF_F37F);
    rec_masked(p, t_mode(2), 0x37F, 0x37F);
    poke(p, t_mode(2), 0xC00);
    poke(p, t_hold(0), 0x1_2345);
    rec(p, t_hold(0), Some(0x2345));
    poke(p, t_comp(3), 0xFFFF_ABCD);
    rec(p, t_comp(3), Some(0xABCD));
    // The V-blank gate, mode 0: held while V-blank is high, counting once it falls.
    poke(p, I_STAT, 0xC);
    wait(p, I_STAT, 0x4, true, 2_000_000);
    poke(p, t_count(3), 0);
    poke(p, t_mode(3), 0x80 | 0x8 | 0x4 | 2);
    delay(p, 300);
    read(p, 3, "a0");
    p.record("a0", Some(0));
    poke(p, I_STAT, 0x8);
    wait(p, I_STAT, 0x8, true, 2_000_000);
    delay(p, 2_000);
    read(p, 3, "t2");
    p.emit("sltiu t2, t2, 2");
    p.emit("xori t2, t2, 1");
    rec_bool(p, 1);
    // Mode 1: counting before the first edge [A, D-38], reset at V-blank's rising edge.
    poke(p, t_mode(3), 0);
    poke(p, t_count(3), 0);
    poke(p, t_mode(3), 0x80 | 0x10 | 0x8 | 0x4 | 2);
    read(p, 3, "a0");
    delay(p, 2_000);
    read(p, 3, "a1");
    p.emit("sltu t2, a0, a1");
    rec_bool(p, 1);
    poke(p, I_STAT, 0x4);
    wait(p, I_STAT, 0x4, true, 2_000_000);
    read(p, 3, "t2");
    p.emit("sltiu t2, t2, 8");
    rec_bool(p, 1);
    // H-blank as the clock: a field's lines between two V-blank starts.
    poke(p, t_mode(3), 0);
    poke(p, t_count(3), 0);
    poke(p, t_mode(3), 0x83);
    poke(p, I_STAT, 0x4);
    wait(p, I_STAT, 0x4, true, 2_000_000);
    read(p, 3, "a0");
    poke(p, I_STAT, 0x4);
    wait(p, I_STAT, 0x4, true, 2_000_000);
    read(p, 3, "a1");
    p.emit("subu a2, a1, a0");
    p.record("a2", None);
    p.emit("addiu t2, a2, -261");
    p.emit("sltiu t2, t2, 3");
    rec_bool(p, 1);
    poke(p, t_mode(3), 0xC00);
    poke(p, I_STAT, 0x7FFF);
    std::mem::replace(p, Program::new("s11-timers"))
}

pub fn all() -> Vec<Program> {
    vec![spr(), chain(), callret(), interleave(), stall(), mfifo(), intc(), timers()]
}
