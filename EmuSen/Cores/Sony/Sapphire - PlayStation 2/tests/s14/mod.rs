//! The project's S14 programs (Sapphire_Native.md §28.6): bare EE programs that feed VIF0 and VIF1 VIF packets by
//! normal and chain DMA and through their FIFOs, then read back what the EE can see: the VUs' memories at 11000000h,
//! the VIF registers, I_STAT. Every UNPACK format and mode, the CYCLE register's skipping and filling, MPG and the MSCAL
//! family with microprograms that report XTOP and XITOP, VIF1's double buffer, the i bit's interrupt and stall, FBRST,
//! the error masks, DIRECT and the FLUSH family. Like S11's they run from kseg0 with everything else through kseg1.
//! The expected words come from `manual`, apart from `src/vif`.

#![allow(dead_code, clippy::needless_range_loop)]

pub mod console;
pub mod manual;

use crate::s6::Program;
use crate::s11::reg::*;
use crate::s11::{K1, li, mem_quad, peek, poke, rec, rec_mem, sx};
use manual::{Double, Memory, Regs};

pub const VIF_REGS: [u32; 2] = [0x1000_3800, 0x1000_3C00];
pub const VIF_FIFO: [u32; 2] = [0x1000_4000, 0x1000_5000];
pub const VU_DATA: [u32; 2] = [0x1100_4000, 0x1100_C000];
pub const VU_MICRO: [u32; 2] = [0x1100_0000, 0x1100_8000];
pub const QWORDS: [u32; 2] = [256, 1024];
/// Where the packets are built, in main memory.
pub const BUF: u32 = 0x0034_0000;

pub mod r {
    pub const STAT: u32 = 0x00;
    pub const FBRST: u32 = 0x10;
    pub const ERR: u32 = 0x20;
    pub const MARK: u32 = 0x30;
    pub const CYCLE: u32 = 0x40;
    pub const MODE: u32 = 0x50;
    pub const NUM: u32 = 0x60;
    pub const MASK: u32 = 0x70;
    pub const CODE: u32 = 0x80;
    pub const ITOPS: u32 = 0x90;
    pub const BASE: u32 = 0xA0;
    pub const OFST: u32 = 0xB0;
    pub const TOPS: u32 = 0xC0;
    pub const ITOP: u32 = 0xD0;
    pub const TOP: u32 = 0xE0;
    pub const R0: u32 = 0x100;
    pub const C0: u32 = 0x140;
}

pub fn vreg(n: usize, off: u32) -> u32 {
    VIF_REGS[n] + off
}

const STR: u32 = 0x100;
const CHAIN: u32 = 1 << 2;
const TTE: u32 = 1 << 6;

/// A VIF stream as words.
#[derive(Clone, Debug, Default)]
pub struct Stream(pub Vec<u32>);

impl Stream {
    pub fn new() -> Stream {
        Stream(Vec::new())
    }

    pub fn code(&mut self, cmd: u32, num: u32, imm: u32) -> &mut Stream {
        self.0.push(cmd << 24 | (num & 0xFF) << 16 | (imm & 0xFFFF));
        self
    }

    pub fn words(&mut self, w: &[u32]) -> &mut Stream {
        self.0.extend_from_slice(w);
        self
    }

    /// NOPs until the stream's length leaves `rem` over a multiple of `n`.
    pub fn align(&mut self, n: usize, rem: usize) -> &mut Stream {
        while self.0.len() % n != rem {
            self.0.push(0);
        }
        self
    }

    /// MPG at an odd word, so its instructions start on a doubleword (§6.3.3's patterns).
    pub fn mpg(&mut self, addr: u32, code: &[u64]) -> &mut Stream {
        self.align(2, 1);
        self.code(0x4A, code.len() as u32, addr);
        for c in code {
            self.0.push(*c as u32);
            self.0.push((*c >> 32) as u32);
        }
        self
    }

    /// DIRECT (or DIRECTHL) at the third word of a qword, so its data starts on one (§6.3.3).
    pub fn direct(&mut self, hl: bool, data: &[[u32; 4]]) -> &mut Stream {
        self.align(4, 3);
        self.code(if hl { 0x51 } else { 0x50 }, 0, data.len() as u32);
        for q in data {
            self.0.extend_from_slice(q);
        }
        self
    }

    pub fn unpack(&mut self, cmd: u32, num: u32, imm: u32, data: &[u32]) -> &mut Stream {
        self.code(cmd, num, imm);
        self.words(data)
    }

    pub fn row(&mut self, v: [u32; 4]) -> &mut Stream {
        self.code(0x30, 0, 0).words(&v)
    }

    pub fn col(&mut self, v: [u32; 4]) -> &mut Stream {
        self.code(0x31, 0, 0).words(&v)
    }

    pub fn mask(&mut self, v: u32) -> &mut Stream {
        self.code(0x20, 0, 0).words(&[v])
    }

    pub fn qwords(&self) -> Vec<[u32; 4]> {
        let mut w = self.0.clone();
        while !w.len().is_multiple_of(4) {
            w.push(0);
        }
        w.chunks(4).map(|c| [c[0], c[1], c[2], c[3]]).collect()
    }
}

/// Waits, at most `limit` passes, until the word at `pa` ANDed with the 32-bit `mask` is zero.
pub fn wait_clear(p: &mut Program, pa: u32, mask: u32, limit: u32) {
    let l = p.label();
    let out = p.label();
    li(p, "t6", K1 | pa);
    li(p, "t4", mask);
    li(p, "t5", limit);
    p.place(&l);
    p.emit("lw t2, 0(t6)");
    p.emit("and t2, t2, t4");
    p.emit(format!("beq t2, zero, {out}"));
    p.emit("nop");
    p.emit("addiu t5, t5, -1");
    p.emit(format!("bne t5, zero, {l}"));
    p.emit("nop");
    p.place(&out);
}

/// Waits for VIF `n` to be idle with its FIFO empty (VPS 0, FQC 0).
pub fn idle(p: &mut Program, n: usize) {
    wait_clear(p, vreg(n, r::STAT), 0x1F00_0003, 20_000);
}

/// Writes a packet's qwords at `at` and sends them to VIF `n` by normal-mode DMA, then waits for the channel and the
/// VIF.
pub fn send(p: &mut Program, n: usize, s: &Stream, at: u32) {
    let q = s.qwords();
    for (i, w) in q.iter().enumerate() {
        mem_quad(p, at + 16 * i as u32, *w);
    }
    poke(p, madr(n), at);
    poke(p, qwc(n), q.len() as u32);
    poke(p, chcr(n), STR | u32::from(n == 1));
    wait_clear(p, chcr(n), STR, 20_000);
    idle(p, n);
}

/// Sends a packet by DMA and waits for the channel only (the VIF may be left stalled).
pub fn send_only(p: &mut Program, n: usize, s: &Stream, at: u32) {
    let q = s.qwords();
    for (i, w) in q.iter().enumerate() {
        mem_quad(p, at + 16 * i as u32, *w);
    }
    poke(p, madr(n), at);
    poke(p, qwc(n), q.len() as u32);
    poke(p, chcr(n), STR | u32::from(n == 1));
    wait_clear(p, chcr(n), STR, 20_000);
}

/// Stores a packet's qwords into VIF `n`'s FIFO with SQ.
pub fn fifo(p: &mut Program, n: usize, s: &Stream) {
    for q in s.qwords() {
        let v = q.iter().rev().fold(0u128, |a, &w| a << 32 | u128::from(w));
        p.quad("t7", v);
        li(p, "t6", K1 | VIF_FIFO[n]);
        p.emit("sq t7, 0(t6)");
    }
}

/// FBRST.RST, then ERR's ME0 and ME1 set as the manual asks (§6.5, VIFn_ERR: ME0 always, ME1 for V3-16).
pub fn reset(p: &mut Program, n: usize) {
    poke(p, vreg(n, r::FBRST), 1);
    poke(p, vreg(n, r::ERR), 6);
}

/// Every S14 program starts here: COP2 usable, interrupts off, the DMAC as S11's start leaves it, the INTC cleared,
/// both VUs reset by FBRST, both VIFs reset.
pub fn prologue(p: &mut Program) {
    p.emit("di");
    li(p, "t0", 0x4000_0000);
    p.emit("mtc0 t0, Status");
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
    poke(p, D_CTRL, 1);
    li(p, "t0", 0x0202);
    p.emit("ctc2 t0, vi28");
    for n in 0..2 {
        reset(p, n);
    }
}

/// The value the programs leave in VU data memory before a case writes it, so a field not written shows.
pub fn pattern(n: usize, a: u32) -> [u32; 4] {
    core::array::from_fn(|f| 0xE000_0000 | (n as u32) << 24 | a << 8 | f as u32)
}

pub fn prefill(p: &mut Program, mem: &mut Memory, n: usize, from: u32, count: u32) {
    for a in from..from + count {
        let q = pattern(n, a);
        mem.0[(a % QWORDS[n]) as usize] = q;
        mem_quad(p, VU_DATA[n] + 16 * (a % QWORDS[n]), q);
    }
}

pub fn rec_vu(p: &mut Program, mem: &Memory, n: usize, a: u32) {
    let a = a % QWORDS[n];
    rec_mem(p, VU_DATA[n] + 16 * a, Some(mem.0[a as usize]));
}

/// Data words of a deterministic mix, both signs in every element width.
pub fn data(seed: u32, n: usize) -> Vec<u32> {
    (0..n as u32).map(|i| (seed.wrapping_add(i).wrapping_mul(0x9E37_79B9)).rotate_left(i % 13) ^ 0x8040_2010).collect()
}

/// The 21 UNPACK cases of ps2autotests' `dma/vif/unpack` text, by CMD's low bits and USN.
pub fn formats() -> Vec<(u32, bool)> {
    let mut v = Vec::new();
    for vn in 0..4 {
        for vl in 0..3 {
            let f = vn << 2 | vl;
            v.push((f, false));
            if vl != 0 {
                v.push((f, true));
            }
        }
    }
    v.push((0xF, false));
    v
}

/// `s14-unpack`: every format with sign and zero extension on both VIFs, three vectors each over a prefilled memory;
/// an UNPACK wrapping at the memory's end and one whose ADDR exceeds VU0's memory; NUM and CODE after; and an UNPACK
/// with no data, the last word of its qword (the console text's last lines): VPS 1, CODE, and NUM, VIF0's 8 bits and
/// VIF1's more (D-185).
pub fn unpack() -> Program {
    let mut p = Program::kernel("s14-unpack");
    let p = &mut p;
    prologue(p);
    for n in 0..2 {
        let mut mem = Memory::new(QWORDS[n] as usize, |a| [a as u32; 4]);
        let mut regs = Regs { cl: 1, wl: 1, ..Regs::default() };
        let cases = formats();
        prefill(p, &mut mem, n, 16, 4 * cases.len() as u32);
        let mut s = Stream::new();
        s.code(0x01, 0, 0x0101);
        let mut last = 0;
        for (k, &(f, usn)) in cases.iter().enumerate() {
            let cmd = 0x60 | f;
            let addr = 16 + 4 * k as u32;
            let words = manual::data_words(cmd, 3, 1, 1) as usize;
            let d = data(k as u32 * 7 + n as u32, words);
            let imm = addr | u32::from(usn) << 14;
            s.unpack(cmd, 3, imm, &d);
            last = cmd << 24 | 3 << 16 | imm;
            mem.apply(&manual::unpack(&mut regs, cmd, 3, addr, usn, &d));
        }
        send(p, n, &s, BUF + 0x1_0000 * n as u32);
        for k in 0..cases.len() as u32 {
            for j in 0..3 {
                rec_vu(p, &mem, n, 16 + 4 * k + j);
            }
        }
        rec(p, vreg(n, r::NUM), Some(0));
        rec(p, vreg(n, r::CODE), Some(last));
        rec(p, vreg(n, r::CYCLE), Some(0x0101));
        // Wrapping at the memory's end, and an ADDR beyond VU0's 256 qwords.
        let top = QWORDS[n] - 2;
        prefill(p, &mut mem, n, top, 2);
        prefill(p, &mut mem, n, 0, 2);
        let d = data(99 + n as u32, 16);
        let mut s = Stream::new();
        s.unpack(0x6C, 4, top, &d);
        mem.apply(&manual::unpack(&mut regs, 0x6C, 4, top, false, &d));
        let far = 0x3F8;
        prefill(p, &mut mem, n, far, 2);
        let d2 = data(7 + n as u32, 2);
        s.unpack(0x60, 2, far, &d2);
        mem.apply(&manual::unpack(&mut regs, 0x60, 2, far, false, &d2));
        send(p, n, &s, BUF + 0x1_0000 * n as u32 + 0x8000);
        for a in [top, top + 1, 0, 1, far, far + 1] {
            rec_vu(p, &mem, n, a);
        }
        // The console's "No data UNPACK" lines: NUM 1, 255 and 256 (0).
        for num in [1u32, 255, 0] {
            reset(p, n);
            rec(p, vreg(n, r::STAT), Some(0));
            let mut s = Stream::new();
            s.align(4, 3).code(0x62, num, 0);
            fifo(p, n, &s);
            crate::s11::delay(p, 50);
            rec(p, vreg(n, r::STAT), Some(1));
            rec(p, vreg(n, r::CODE), Some(0x62 << 24 | num << 16));
            let shown = if num == 0 { if n == 0 { 0 } else { 256 } } else { num };
            rec(p, vreg(n, r::NUM), Some(shown));
        }
        reset(p, n);
    }
    std::mem::replace(p, Program::new("s14-unpack"))
}

/// The diagonal mask of the console's `stmod` text: Row in X on the first write, Y on the second, Z, then W.
pub const DIAGONAL: u32 = 1 | 1 << 10 | 1 << 20 | 1 << 30;

/// `s14-mask`: the console's `stmod` cases (modes 0 to 3 without and with the diagonal mask, Row 1000h, V4-32 of 0 to
/// 15); every specifier in every write cycle in normal and difference modes; the m bit clear ignoring MASK; V2-16
/// in difference mode with Z and W filled from Col (§6.3.7's example 4); V3-8 in offset mode; the registers after.
pub fn mask() -> Program {
    let mut p = Program::kernel("s14-mask");
    let p = &mut p;
    prologue(p);
    for n in 0..2 {
        let mut mem = Memory::new(QWORDS[n] as usize, |a| [a as u32; 4]);
        let mut at = BUF + 0x1_0000 * n as u32;
        let mut base = 0x20;
        // The stmod cases.
        let values: Vec<u32> = (0..16).collect();
        for mode in 0..4 {
            for diag in [false, true] {
                let mut regs = Regs { row: [0x1000; 4], cl: 4, wl: 4, mode, mask: DIAGONAL, ..Regs::default() };
                prefill(p, &mut mem, n, base, 4);
                let mut s = Stream::new();
                s.code(0x01, 0, 0x0404).row([0x1000; 4]).code(0x05, 0, mode).mask(DIAGONAL);
                let cmd = 0x6C | if diag { 0x10 } else { 0 };
                s.unpack(cmd, 4, base, &values);
                mem.apply(&manual::unpack(&mut regs, cmd, 4, base, false, &values));
                send(p, n, &s, at);
                at += 0x400;
                for j in 0..4 {
                    rec_vu(p, &mem, n, base + j);
                }
                for f in 0..4 {
                    rec(p, vreg(n, r::R0 + 16 * f), Some(regs.row[f as usize]));
                }
                rec(p, vreg(n, r::MODE), Some(mode));
                base += 4;
            }
        }
        // Every specifier in every cycle: each row of MASK a rotation of 0, 1, 2, 3.
        let rot = 0x934E_39E4;
        for mode in [0, 2] {
            let mut regs = Regs { row: [0x100, 0x200, 0x300, 0x400], col: [0xC0, 0xC1, 0xC2, 0xC3], mask: rot, mode, cl: 8, wl: 8 };
            prefill(p, &mut mem, n, base, 6);
            let d = data(31 + mode, 24);
            let mut s = Stream::new();
            s.code(0x01, 0, 0x0808).row(regs.row).col(regs.col).mask(rot).code(0x05, 0, mode);
            s.unpack(0x7C, 6, base, &d);
            mem.apply(&manual::unpack(&mut regs, 0x7C, 6, base, false, &d));
            send(p, n, &s, at);
            at += 0x400;
            for j in 0..6 {
                rec_vu(p, &mem, n, base + j);
            }
            for f in 0..4 {
                rec(p, vreg(n, r::R0 + 16 * f), Some(regs.row[f as usize]));
                rec(p, vreg(n, r::C0 + 16 * f), Some(regs.col[f as usize]));
            }
            rec(p, vreg(n, r::MASK), Some(rot));
            base += 6;
        }
        // The m bit clear: MASK ignored.
        {
            let mut regs = Regs { row: [0x100, 0x200, 0x300, 0x400], mask: 0xFFFF_FFFF, mode: 1, cl: 8, wl: 8, ..Regs::default() };
            prefill(p, &mut mem, n, base, 3);
            let d = data(41, 12);
            let mut s = Stream::new();
            s.row(regs.row).mask(0xFFFF_FFFF).code(0x05, 0, 1);
            s.unpack(0x6C, 3, base, &d);
            mem.apply(&manual::unpack(&mut regs, 0x6C, 3, base, false, &d));
            send(p, n, &s, at);
            at += 0x400;
            for j in 0..3 {
                rec_vu(p, &mem, n, base + j);
            }
            base += 3;
        }
        // V2-16 in difference mode, Z and W from Col (example 4), then V3-8 in offset mode with W masked.
        {
            let m4 = (2 << 4 | 2 << 6) * 0x0101_0101;
            let mut regs = Regs { row: [5, 6, 7, 8], col: [0xA0, 0xA1, 0xA2, 0xA3], mask: m4, mode: 2, cl: 8, wl: 8 };
            prefill(p, &mut mem, n, base, 7);
            let d = data(51, 4);
            let mut s = Stream::new();
            s.row(regs.row).col(regs.col).mask(m4).code(0x05, 0, 2);
            s.unpack(0x75, 4, base, &d);
            mem.apply(&manual::unpack(&mut regs, 0x75, 4, base, false, &d));
            let m3 = 3 << 6 | (3 << 6) << 8 | (3 << 6) << 16 | (3 << 6) << 24;
            regs.mask = m3;
            regs.mode = 1;
            let d3 = data(61, 3);
            s.mask(m3).code(0x05, 0, 1);
            s.unpack(0x7A, 3, base + 4, &d3);
            mem.apply(&manual::unpack(&mut regs, 0x7A, 3, base + 4, false, &d3));
            send(p, n, &s, at);
            for j in 0..7 {
                rec_vu(p, &mem, n, base + j);
            }
            for f in 0..4 {
                rec(p, vreg(n, r::R0 + 16 * f), Some(regs.row[f as usize]));
            }
        }
        reset(p, n);
    }
    std::mem::replace(p, Program::new("s14-mask"))
}

/// `s14-cycle`: skipping writes (CL 4 WL 2, CL 3 WL 1), filling writes with the fills from Col and with the m bit
/// clear (D-183), CL 0 (no data at all), WL 0 (D-181), the console's CL 253 WL 4 on each VIF, a VIF1 UNPACK wrapping
/// at 1024; each UNPACK followed by a MARK, so a wrong count of data words shows in MARK.
pub fn cycle() -> Program {
    let mut p = Program::kernel("s14-cycle");
    let p = &mut p;
    prologue(p);
    for n in 0..2 {
        let mut mem = Memory::new(QWORDS[n] as usize, |a| [a as u32; 4]);
        let mut at = BUF + 0x1_0000 * n as u32;
        let cols = [0xC0, 0xC1, 0xC2, 0xC3];
        let col_mask = 0xAAAA_AAAA;
        let cases: [(u32, u32, u32, u32, u32); 10] = [
            // (CL, WL, CMD, NUM, ADDR)
            (4, 2, 0x6C, 6, 0x40),
            (3, 1, 0x69, 5, 0x60),
            (2, 4, 0x7C, 9, 0x70),
            (2, 4, 0x68, 7, 0x80),
            (0, 3, 0x7E, 5, 0x90),
            (253, 4, 0x70, 8, 0),
            (1, 1, 0x6C, 8, QWORDS[n] - 4),
            (5, 2, 0x65, 5, 0xB0),
            (1, 0, 0x6D, 4, 0xA0),
            (0, 0, 0x7C, 5, 0xA8),
        ];
        for (k, &(cl, wl, cmd, num, addr)) in cases.iter().enumerate() {
            let mut regs = Regs { col: cols, mask: col_mask, cl, wl, ..Regs::default() };
            let writes = manual::unpack(&mut regs.clone(), cmd, num, addr, false, &[]);
            for (a, _) in &writes {
                prefill(p, &mut mem, n, *a, 1);
            }
            let words = manual::data_words(cmd, num, cl, wl) as usize;
            let d = data(70 + k as u32, words);
            let mut s = Stream::new();
            s.code(0x01, 0, wl << 8 | cl).col(cols).mask(col_mask);
            s.unpack(cmd, num, addr, &d);
            s.code(0x07, 0, 0x100 + k as u32);
            let mut stream = d.clone();
            stream.push(0x07 << 24 | (0x100 + k as u32));
            let w = manual::unpack(&mut regs, cmd, num, addr, false, &stream);
            mem.apply(&w);
            send(p, n, &s, at);
            at += 0x800;
            let mut seen = Vec::new();
            for (a, _) in &w {
                let a = a % QWORDS[n];
                if !seen.contains(&a) {
                    seen.push(a);
                    rec_vu(p, &mem, n, a);
                }
            }
            rec(p, vreg(n, r::MARK), Some(0x100 + k as u32));
            rec(p, vreg(n, r::CYCLE), Some(if wl == 0 { cl } else { wl << 8 | cl }));
        }
        reset(p, n);
    }
    std::mem::replace(p, Program::new("s14-cycle"))
}

/// A microprogram that stores XITOP (and on VU1 XTOP) into the qword at `slot`, x and y, then ends.
pub fn report(unit: usize, slot: u32) -> String {
    if unit == 1 {
        format!(
            "nop | xtop vi1\nnop | xitop vi2\nnop | isw.x vi1, 0x{slot:x}(vi0)\nnop | isw.y vi2, 0x{slot:x}(vi0)\n[e] nop | move vf0, vf0\nnop | move vf0, vf0"
        )
    } else {
        format!("nop | xitop vi2\nnop | isw.y vi2, 0x{slot:x}(vi0)\n[e] nop | move vf0, vf0\nnop | move vf0, vf0")
    }
}

/// A microprogram that counts `count` down, then stores `mark` into the qword at `slot`'s x, then ends: long enough
/// for a VIFcode to arrive while it runs.
pub fn long(count: u32, mark: u32, slot: u32) -> String {
    format!(
        "nop | iaddiu vi3, vi0, 0x{count:x}\nloop: nop | iaddi vi3, vi3, -1\nnop | move vf0, vf0\nnop | ibne vi3, vi0, loop\nnop | move vf0, vf0\n\
         nop | iaddiu vi4, vi0, 0x{mark:x}\nnop | isw.x vi4, 0x{slot:x}(vi0)\n[e] nop | move vf0, vf0\nnop | move vf0, vf0"
    )
}

/// A microprogram that copies the x of the qword at `from` into the x of the qword at `to`.
pub fn copy(from: u32, to: u32) -> String {
    format!(
        "nop | ilw.x vi5, 0x{from:x}(vi0)\nnop | move vf0, vf0\nnop | move vf0, vf0\nnop | move vf0, vf0\nnop | move vf0, vf0\n\
         nop | isw.x vi5, 0x{to:x}(vi0)\n[e] nop | move vf0, vf0\nnop | move vf0, vf0"
    )
}

pub fn micro(src: &str) -> Vec<u64> {
    emusen_mips::vu::assemble(src).unwrap_or_else(|e| panic!("{e}\n{src}"))
}

/// Records an ISW'd qword's x and y halves as the microprograms leave them (ISW zero-extends into the fields named).
fn rec_slot(p: &mut Program, n: usize, slot: u32, want: [Option<u32>; 4]) {
    li(p, "t6", K1 | (VU_DATA[n] + 16 * slot));
    for (i, w) in want.iter().enumerate() {
        p.emit(format!("lw t2, {}(t6)", 4 * i));
        p.record("t2", w.and_then(sx));
    }
}

/// `s14-micro`: on both units, MPG's instructions read back from micro memory; ITOP and MSCAL with XITOP (and XTOP
/// on VU1, where TOP is 0 without double buffering); MSCAL arriving while a microprogram runs, which waits (the second
/// copies what the first stores at its end); MPG arriving while one runs, which waits; FLUSHE before an UNPACK into
/// the qword the microprogram writes last; MSCNT continuing at TPC.
pub fn micro_program() -> Program {
    let mut p = Program::kernel("s14-micro");
    let p = &mut p;
    prologue(p);
    for n in 0..2 {
        let mut at = BUF + 0x1_0000 * n as u32;
        let slots = 0x40;
        for a in slots..slots + 8 {
            mem_quad(p, VU_DATA[n] + 16 * a, [0; 4]);
        }
        // Programs at doublewords 0 (report), 16 (long), 48 (copy), 64 (MSCNT's pair of programs).
        let rep = micro(&report(n, slots));
        let lng = micro(&long(300, 0x77, slots + 1));
        let cpy = micro(&copy(slots + 1, slots + 2));
        let first = micro("nop | iaddiu vi6, vi0, 0x61\nnop | isw.x vi6, 0x43(vi0)\n[e] nop | move vf0, vf0\nnop | move vf0, vf0");
        let second = micro("nop | iaddiu vi7, vi0, 0x62\nnop | isw.x vi7, 0x44(vi0)\n[e] nop | move vf0, vf0\nnop | move vf0, vf0");
        let mut s = Stream::new();
        if n == 1 {
            s.code(0x03, 0, 0).code(0x02, 0, 0);
        }
        s.mpg(0, &rep).mpg(16, &lng).mpg(48, &cpy).mpg(64, &first).mpg(64 + first.len() as u32, &second);
        send(p, n, &s, at);
        at += 0x800;
        for (base, code) in [(0u32, &rep), (16, &lng), (48, &cpy), (64, &first)] {
            for (i, c) in code.iter().enumerate() {
                rec(p, VU_MICRO[n] + 8 * (base + i as u32), Some(*c as u32));
                rec(p, VU_MICRO[n] + 8 * (base + i as u32) + 4, Some((*c >> 32) as u32));
            }
        }
        rec(p, vreg(n, r::NUM), Some(0));
        // ITOP, then MSCAL of the report.
        let mut s = Stream::new();
        s.code(0x04, 0, 0x2AB);
        send(p, n, &s, at);
        at += 0x400;
        rec(p, vreg(n, r::ITOPS), Some(0x2AB));
        rec(p, vreg(n, r::ITOP), Some(0));
        let mut s = Stream::new();
        s.code(0x14, 0, 0).code(0x10, 0, 0);
        send(p, n, &s, at);
        at += 0x400;
        rec(p, vreg(n, r::ITOP), Some(0x2AB));
        rec_slot(p, n, slots, [Some(0), Some(0x2AB), Some(0), Some(0)]);
        // MSCAL while the long program runs: the copy waits for its store.
        let mut s = Stream::new();
        s.code(0x14, 0, 16).code(0x14, 0, 48).code(0x10, 0, 0);
        send(p, n, &s, at);
        at += 0x400;
        rec_slot(p, n, slots + 1, [Some(0x77), Some(0), Some(0), Some(0)]);
        rec_slot(p, n, slots + 2, [Some(0x77), Some(0), Some(0), Some(0)]);
        // MPG while it runs: the program's mark changed only for its next run.
        let lng2 = micro(&long(300, 0x78, slots + 1));
        let mut s = Stream::new();
        s.code(0x14, 0, 16).mpg(16, &lng2).code(0x14, 0, 48).code(0x10, 0, 0);
        send(p, n, &s, at);
        at += 0x400;
        rec_slot(p, n, slots + 2, [Some(0x77), Some(0), Some(0), Some(0)]);
        let mut s = Stream::new();
        s.code(0x14, 0, 16).code(0x10, 0, 0);
        send(p, n, &s, at);
        at += 0x400;
        rec_slot(p, n, slots + 1, [Some(0x78), Some(0), Some(0), Some(0)]);
        // FLUSHE: the UNPACK after it lands after the program's last store.
        let mut s = Stream::new();
        s.code(0x01, 0, 0x0101).code(0x14, 0, 16).code(0x10, 0, 0).unpack(0x60, 1, slots + 1, &[0x55]);
        send(p, n, &s, at);
        at += 0x400;
        rec_slot(p, n, slots + 1, [Some(0x55); 4]);
        // MSCNT: the second program after the first's end.
        let mut s = Stream::new();
        s.code(0x14, 0, 64).code(0x10, 0, 0).code(0x17, 0, 0).code(0x10, 0, 0);
        send(p, n, &s, at);
        rec_slot(p, n, 0x43, [Some(0x61), Some(0), Some(0), Some(0)]);
        rec_slot(p, n, 0x44, [Some(0x62), Some(0), Some(0), Some(0)]);
        reset(p, n);
    }
    std::mem::replace(p, Program::new("s14-micro"))
}

/// `s14-dbuf`: VIF1's double buffer: BASE and OFFSET, then MSCAL, MSCAL, MSCNT and OFFSET again, each microprogram
/// storing XTOP and XITOP at the qword TOP names; UNPACKs with FLG landing at TOPS + ADDR; BASE + OFFSET wrapping at
/// 10 bits; the registers after each step.
pub fn dbuf() -> Program {
    let mut p = Program::kernel("s14-dbuf");
    let p = &mut p;
    prologue(p);
    let n = 1;
    let mut at = BUF + 0x2_0000;
    let mut mem = Memory::new(1024, |a| [a as u32; 4]);
    let store_at_top =
        "nop | xtop vi1\nnop | xitop vi2\nnop | isw.x vi1, 0x0(vi1)\nnop | isw.y vi2, 0x0(vi1)\n[e] nop | move vf0, vf0\nnop | move vf0, vf0";
    let store_at_top3 =
        "nop | xtop vi1\nnop | xitop vi2\nnop | isw.z vi1, 0x3(vi1)\nnop | isw.w vi2, 0x3(vi1)\n[e] nop | move vf0, vf0\nnop | move vf0, vf0";
    let a = micro(store_at_top);
    let b = micro(store_at_top3);
    let mut s = Stream::new();
    s.mpg(0, &a).mpg(a.len() as u32, &b).code(0x01, 0, 0x0101);
    send(p, n, &s, at);
    at += 0x800;
    let mut d = Double::default();
    let regs_rec = |p: &mut Program, d: &Double, itop: u32, itops: u32| {
        rec(p, vreg(1, r::BASE), Some(d.base));
        rec(p, vreg(1, r::OFST), Some(d.ofst));
        rec(p, vreg(1, r::TOPS), Some(d.tops));
        rec(p, vreg(1, r::TOP), Some(d.top));
        rec(p, vreg(1, r::ITOP), Some(itop));
        rec(p, vreg(1, r::ITOPS), Some(itops));
        peek(p, vreg(1, r::STAT));
        p.emit("andi t2, t2, 0x80");
        p.record("t2", sx(if d.dbf { 0x80 } else { 0 }));
    };
    for a in [0x200, 0x201, 0x202, 0x203, 0x300, 0x301, 0x302, 0x010, 0x011, 0x3F0] {
        prefill(p, &mut mem, n, a, 1);
    }
    let mut s = Stream::new();
    s.code(0x03, 0, 0x200).code(0x02, 0, 0x100);
    d.base = 0x200;
    d.offset(0x100);
    send(p, n, &s, at);
    at += 0x400;
    regs_rec(p, &d, 0, 0);
    // MSCAL, an UNPACK into TOPS + 1, MSCAL, an UNPACK into TOPS + 2, MSCNT.
    let mut itops = 0;
    let mut itop;
    for (step, (cmd, start, val)) in [(0x14u32, 0u32, 0x11u32), (0x14, 0, 0x22), (0x17, 0, 0x33)].into_iter().enumerate() {
        let mut s = Stream::new();
        s.code(0x04, 0, val).code(cmd, 0, start).code(0x10, 0, 0);
        itops = val;
        itop = val;
        d.start();
        let top = d.top;
        if cmd == 0x17 {
            mem.0[(top + 3) as usize][2] = top;
            mem.0[(top + 3) as usize][3] = itop;
        } else {
            mem.0[top as usize][0] = top;
            mem.0[top as usize][1] = itop;
        }
        if step < 2 {
            let v = 0xD0 + step as u32;
            s.unpack(0x60, 1, 0x8000 | (1 + step as u32), &[v]);
            mem.0[(d.tops + 1 + step as u32) as usize] = [v; 4];
        }
        send(p, n, &s, at);
        at += 0x400;
        regs_rec(p, &d, itop, itops);
    }
    for a in [0x200, 0x201, 0x202, 0x203, 0x300, 0x301, 0x302] {
        rec_vu(p, &mem, n, a);
    }
    // OFFSET again: DBF cleared, TOPS = BASE; then BASE + OFFSET past 10 bits.
    let mut s = Stream::new();
    s.code(0x02, 0, 0x40);
    d.offset(0x40);
    send(p, n, &s, at);
    at += 0x400;
    regs_rec(p, &d, 0x33, itops);
    let mut s = Stream::new();
    s.code(0x03, 0, 0x3F0).code(0x02, 0, 0x20).code(0x14, 0, 0).code(0x10, 0, 0);
    d.base = 0x3F0;
    d.offset(0x20);
    d.start();
    mem.0[d.top as usize][0] = d.top;
    mem.0[d.top as usize][1] = 0x33;
    s.unpack(0x60, 1, 0x8001, &[0xEE]);
    mem.0[((d.tops + 1) & 0x3FF) as usize] = [0xEE; 4];
    send(p, n, &s, at);
    regs_rec(p, &d, 0x33, itops);
    for a in [0x3F0, 0x010, 0x011] {
        rec_vu(p, &mem, n, a);
    }
    reset(p, n);
    std::mem::replace(p, Program::new("s14-dbuf"))
}

/// `s14-regs`: on both VIFs, MARK by VIFcode and by the EE's write; the console's `intstall` shapes (NOP, MARK and STMASK
/// with the i bit, each followed by an STMASK) with STAT, MASK and I_STAT, then STC; the stall holding a packet's
/// later qwords in the FIFO; FBRST's FBK and STP with STC; RST, which keeps the registers a program sets (D-187); ERR's
/// MII and ME1;
/// an undefined VIFcode's ER1 and interrupt; VIF1's codes on VIF0 and an UNPACK of no format (D-188); STROW through
/// the FIFO; a chain with TTE, whose tags' upper halves are VIFcodes.
pub fn regs() -> Program {
    let mut p = Program::kernel("s14-regs");
    let p = &mut p;
    prologue(p);
    for n in 0..2 {
        let mut at = BUF + 0x1_0000 * n as u32;
        let vif_bit = 1 << (4 + n);
        let mut next = || {
            at += 0x200;
            at
        };
        // MARK.
        let mut s = Stream::new();
        s.code(0x07, 0, 0xBEEF);
        send(p, n, &s, next());
        rec(p, vreg(n, r::MARK), Some(0xBEEF));
        rec(p, vreg(n, r::STAT), Some(0x40));
        poke(p, vreg(n, r::MARK), 0x1_55AA);
        rec(p, vreg(n, r::MARK), Some(0x55AA));
        rec(p, vreg(n, r::STAT), Some(0));
        // The intstall shapes.
        for (first, mask_after, stat_after, stat_resumed) in [
            ([0x8000_0000, 0x2000_0000, 0xCCCC_CCCC, 0], 0xAAAA_AAAAu32, 0xC02u32, 0xCCCC_CCCCu32),
            ([0x8700_1234, 0x2000_0000, 0xCCCC_CCCC, 0], 0xCCCC_CCCC, 0x840, 0xCCCC_CCCC),
            ([0xA000_0000, 0xFFFF_FFFF, 0x2000_0000, 0xCCCC_CCCC], 0xFFFF_FFFF, 0xC02, 0xCCCC_CCCC),
        ] {
            reset(p, n);
            poke(p, I_STAT, 0x7FFF);
            let mut s = Stream::new();
            s.mask(0xAAAA_AAAA);
            send(p, n, &s, next());
            let mut s = Stream::new();
            s.words(&first);
            send_only(p, n, &s, next());
            crate::s11::delay(p, 200);
            rec(p, vreg(n, r::MASK), Some(mask_after));
            rec(p, vreg(n, r::STAT), Some(stat_after));
            peek(p, I_STAT);
            p.emit(format!("andi t2, t2, 0x{vif_bit:x}"));
            p.record("t2", sx(vif_bit));
            poke(p, vreg(n, r::FBRST), 8);
            idle(p, n);
            rec(p, vreg(n, r::MASK), Some(stat_resumed));
            rec(p, vreg(n, r::STAT), Some(if first[0] == 0x8700_1234 { 0x40 } else { 0 }));
        }
        // The stall holding a packet's later qwords: three more qwords wait in the FIFO.
        reset(p, n);
        let mut s = Stream::new();
        s.words(&[0x8000_0000, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x07 << 24 | 0x4321]);
        send_only(p, n, &s, next());
        crate::s11::delay(p, 200);
        rec(p, vreg(n, r::STAT), Some(0x0300_0C02));
        rec(p, chcr(n), Some(u32::from(n == 1)));
        poke(p, vreg(n, r::FBRST), 8);
        idle(p, n);
        rec(p, vreg(n, r::MARK), Some(0x4321));
        rec(p, vreg(n, r::STAT), Some(0x40));
        // FBK while waiting for data, then STC.
        reset(p, n);
        let mut s = Stream::new();
        s.align(4, 2).code(0x01, 0, 0x0101).code(0x60, 2, 0x10);
        fifo(p, n, &s);
        crate::s11::delay(p, 50);
        rec(p, vreg(n, r::STAT), Some(1));
        poke(p, vreg(n, r::FBRST), 2);
        let mut s = Stream::new();
        s.words(&[0x1111_1111, 0x2222_2222, 0x07 << 24 | 0x99, 0]);
        fifo(p, n, &s);
        crate::s11::delay(p, 50);
        rec(p, vreg(n, r::STAT), Some(0x0100_0203));
        rec(p, vreg(n, r::NUM), Some(2));
        poke(p, vreg(n, r::FBRST), 8);
        idle(p, n);
        rec(p, vreg(n, r::NUM), Some(0));
        rec(p, vreg(n, r::MARK), Some(0x99));
        rec_mem(p, VU_DATA[n] + 0x110, Some([0x2222_2222; 4]));
        // STP while idle stops at once; STP during an UNPACK stops before the next VIFcode.
        reset(p, n);
        poke(p, vreg(n, r::FBRST), 4);
        rec(p, vreg(n, r::STAT), Some(0x100));
        let mut s = Stream::new();
        s.code(0x07, 0, 0x11);
        fifo(p, n, &s);
        crate::s11::delay(p, 50);
        rec(p, vreg(n, r::MARK), Some(0x99));
        rec(p, vreg(n, r::STAT), Some(0x0100_0100));
        poke(p, vreg(n, r::FBRST), 8);
        idle(p, n);
        rec(p, vreg(n, r::MARK), Some(0x11));
        let mut s = Stream::new();
        s.align(4, 2).code(0x01, 0, 0x0101).code(0x60, 1, 0x12);
        fifo(p, n, &s);
        crate::s11::delay(p, 50);
        poke(p, vreg(n, r::FBRST), 4);
        let mut s = Stream::new();
        s.words(&[0x3333_3333, 0x07 << 24 | 0x22, 0, 0]);
        fifo(p, n, &s);
        crate::s11::delay(p, 50);
        rec(p, vreg(n, r::MARK), Some(0x11));
        rec(p, vreg(n, r::STAT), Some(0x140));
        rec_mem(p, VU_DATA[n] + 0x120, Some([0x3333_3333; 4]));
        poke(p, vreg(n, r::FBRST), 8);
        idle(p, n);
        rec(p, vreg(n, r::MARK), Some(0x22));
        // RST clears the registers.
        let mut s = Stream::new();
        s.code(0x01, 0, 0x0203).code(0x05, 0, 2).mask(0x1234_5678).row([1, 2, 3, 4]).col([5, 6, 7, 8]).code(0x04, 0, 0x55);
        send(p, n, &s, next());
        rec(p, vreg(n, r::CYCLE), Some(0x0203));
        poke(p, vreg(n, r::FBRST), 1);
        let kept = [(r::STAT, 0), (r::ERR, 0), (r::MARK, 0x22), (r::CYCLE, 0x0203), (r::MODE, 2), (r::NUM, 0), (r::MASK, 0x1234_5678), (r::CODE, 0)];
        let kept2 = [(r::ITOPS, 0x55), (r::R0, 1), (r::R0 + 0x30, 4), (r::C0, 5), (r::C0 + 0x30, 8)];
        for (off, v) in kept.into_iter().chain(kept2) {
            rec(p, vreg(n, off), Some(v));
        }
        poke(p, vreg(n, r::ERR), 2);
        // MII: the i bit ignored.
        poke(p, vreg(n, r::ERR), 7);
        poke(p, I_STAT, 0x7FFF);
        let mut s = Stream::new();
        s.code(0x80, 0, 0).code(0x07, 0, 0x33);
        send(p, n, &s, next());
        rec(p, vreg(n, r::MARK), Some(0x33));
        rec(p, vreg(n, r::STAT), Some(0x40));
        peek(p, I_STAT);
        p.emit(format!("andi t2, t2, 0x{vif_bit:x}"));
        p.record("t2", sx(0));
        // An undefined VIFcode: ER1 and the interrupt, the next VIFcode held; ME1 makes it a NOP.
        poke(p, vreg(n, r::ERR), 2);
        poke(p, vreg(n, r::FBRST), 8);
        let mut s = Stream::new();
        s.code(0x08, 0, 0).code(0x07, 0, 0x44);
        send_only(p, n, &s, next());
        crate::s11::delay(p, 200);
        rec(p, vreg(n, r::MARK), Some(0x33));
        rec(p, vreg(n, r::STAT), Some(0x2042));
        rec(p, vreg(n, r::CODE), Some(0x0800_0000));
        peek(p, I_STAT);
        p.emit(format!("andi t2, t2, 0x{vif_bit:x}"));
        p.record("t2", sx(vif_bit));
        poke(p, vreg(n, r::FBRST), 8);
        idle(p, n);
        rec(p, vreg(n, r::MARK), Some(0x44));
        poke(p, vreg(n, r::ERR), 6);
        let mut s = Stream::new();
        s.code(0x08, 0, 0).code(0x07, 0, 0x45);
        send(p, n, &s, next());
        rec(p, vreg(n, r::MARK), Some(0x45));
        rec(p, vreg(n, r::STAT), Some(0x40));
        // VIF1's OFFSET on VIF0, and an UNPACK of no format (63h), each ER1.
        for code in [0x02u32, 0x63] {
            reset(p, n);
            poke(p, vreg(n, r::ERR), 2);
            let mut s = Stream::new();
            s.code(code, 0, 0x10).code(0x07, 0, 0x46);
            send_only(p, n, &s, next());
            crate::s11::delay(p, 200);
            let undefined = code == 0x63 || n == 0;
            rec(p, vreg(n, r::STAT), Some(if undefined { 0x2002 } else { 0x40 }));
            poke(p, vreg(n, r::FBRST), 8);
            idle(p, n);
            rec(p, vreg(n, r::MARK), Some(0x46));
        }
        // STROW through the FIFO.
        reset(p, n);
        let mut s = Stream::new();
        s.row([0x1337_C0DE, 0x1337_C1DE, 0x1337_C2DE, 0x1337_C3DE]);
        fifo(p, n, &s);
        idle(p, n);
        for f in 0..4 {
            rec(p, vreg(n, r::R0 + 16 * f), Some(0x1337_C0DE + 0x100 * f));
        }
        // A chain with TTE: cnt with a MARK in its tag's upper half and one in its data, then end with a MARK.
        reset(p, n);
        let t = next();
        mem_quad(p, t, [1 | 1 << 28, 0, 0x07 << 24 | 0x66, 0]);
        mem_quad(p, t + 0x10, [0x07 << 24 | 0x67, 0x01 << 24 | 0x0303, 0, 0]);
        mem_quad(p, t + 0x20, [7 << 28, 0, 0x05 << 24 | 2, 0x07 << 24 | 0x68]);
        poke(p, tadr(n), t);
        poke(p, qwc(n), 0);
        poke(p, chcr(n), STR | CHAIN | TTE | u32::from(n == 1));
        wait_clear(p, chcr(n), STR, 20_000);
        idle(p, n);
        rec(p, vreg(n, r::MARK), Some(0x68));
        rec(p, vreg(n, r::CYCLE), Some(0x0303));
        rec(p, vreg(n, r::MODE), Some(2));
        reset(p, n);
    }
    std::mem::replace(p, Program::new("s14-regs"))
}

/// The GS's SIGLBLID privileged register, whose LBLID an A+D write of LABEL through PATH2 sets on the referees.
pub const SIGLBLID: u32 = 0x1200_1080;

/// `s14-direct`: VIF1's DIRECT with NUM and CODE after; MSKPATH3; FLUSH, FLUSHA and MSCALF with a microprogram that
/// stores a mark; DIRECTHL last. Each packet writes LABEL by A+D, which the referees' GS shows in SIGLBLID; on
/// Sapphire PATH2 ends at its stand-in until S15, so those words are graded there by the tests instead.
pub fn direct() -> Program {
    let mut p = Program::kernel("s14-direct");
    let p = &mut p;
    prologue(p);
    let n = 1;
    let mut at = BUF + 0x3_0000;
    poke(p, 0x1200_1000, 1);
    // A GIFtag of one loop, EOP, PACKED, one register A+D (EE User's Manual §7.2.2); LABEL's ID and IDMSK.
    let giftag = [1 | 1 << 15, 0x1000_0000, 0xE, 0];
    let signal = [0x0000_1234, 0x0000_FFFF, 0x62, 0];
    let mut s = Stream::new();
    s.direct(false, &[giftag, signal]);
    send(p, n, &s, at);
    at += 0x400;
    rec(p, vreg(n, r::NUM), Some(0));
    rec(p, vreg(n, r::CODE), Some(0x5000_0002));
    li(p, "t6", K1 | SIGLBLID);
    p.emit("lw t2, 4(t6)");
    p.record("t2", None);
    let mut s = Stream::new();
    s.direct(false, &[giftag, [0x0000_5678, 0x0000_FFFF, 0x62, 0]]);
    s.code(0x06, 0, 0x8000).code(0x11, 0, 0).code(0x13, 0, 0).code(0x06, 0, 0);
    send(p, n, &s, at);
    at += 0x400;
    rec(p, vreg(n, r::CODE), Some(0x0600_0000));
    rec(p, vreg(n, r::STAT), Some(0));
    li(p, "t6", K1 | SIGLBLID);
    p.emit("lw t2, 4(t6)");
    p.record("t2", None);
    let mark = micro("nop | iaddiu vi1, vi0, 0x5a\nnop | isw.x vi1, 0x50(vi0)\n[e] nop | move vf0, vf0\nnop | move vf0, vf0");
    mem_quad(p, VU_DATA[1] + 0x500, [0; 4]);
    let mut s = Stream::new();
    s.mpg(0, &mark).code(0x15, 0, 0).code(0x11, 0, 0);
    send(p, n, &s, at);
    rec_slot(p, n, 0x50, [Some(0x5A), Some(0), Some(0), Some(0)]);
    rec(p, vreg(n, r::CODE), Some(0x1100_0000));
    // DIRECTHL last, a LABEL again.
    let mut s = Stream::new();
    s.direct(true, &[giftag, [0x0000_9ABC, 0x0000_FFFF, 0x62, 0]]);
    send(p, n, &s, at + 0x400);
    rec(p, vreg(n, r::CODE), Some(0x5100_0002));
    peek(p, vreg(n, r::STAT));
    p.emit("andi t2, t2, 0x3f3f");
    p.record("t2", Some(0));
    li(p, "t6", K1 | SIGLBLID);
    p.emit("lw t2, 4(t6)");
    p.record("t2", None);
    reset(p, n);
    std::mem::replace(p, Program::new("s14-direct"))
}

pub fn all() -> Vec<Program> {
    vec![unpack(), mask(), cycle(), micro_program(), dbuf(), regs(), direct()]
}
