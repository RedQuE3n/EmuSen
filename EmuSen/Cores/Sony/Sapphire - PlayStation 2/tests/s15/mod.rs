//! The project's S15 programs (Sapphire_Native.md §33.6): bare EE programs in S14's form that send GS packets down the
//! three paths, PATH3 by normal and chain DMA and by the EE's stores into the GIF's FIFO, PATH2 by VIF1's DIRECT and
//! DIRECTHL, PATH1 by a microprogram's XGKICK started through CMSAR1, and hold one path mid-packet while another asks,
//! so that the arbitration's order, PATH3's masks and the intermittent mode show in what reaches the GS. The record
//! is the GS's local memory read back local to host (S16's route), SIGLBLID's LBLID after LABELs, and the GIF's and
//! VIF1's registers. The expected record comes from `manual` (chapter 7) over S17's model of the GS, the arrival order
//! each program states from §7.1.

#![allow(dead_code)]

pub mod manual;

use crate::s6::{Program, RECORD};
use crate::s11::reg::{chcr, madr, qwc, tadr};
use crate::s11::{K1, li, poke, wait};
use crate::s14::{self, Stream, VU_DATA, vreg};
use crate::s16::{self, bitbltbuf, ee_address, ga, r as gr, trxpos, trxreg};
use crate::s17::{self, ALWAYS, Buf, SPRITE, begin, target, test, zbuf};
use crate::s16::manual::{CT32, Z32};
use manual::{Gif as Model, ad, quad, regs_packet, tag};

pub const GIF_CTRL: u32 = 0x1000_3000;
pub const GIF_MODE: u32 = 0x1000_3010;
pub const GIF_STAT: u32 = 0x1000_3020;
pub const GIF_TAG0: u32 = 0x1000_3040;
pub const GIF_CNT: u32 = 0x1000_3080;
pub const GIF_P3CNT: u32 = 0x1000_3090;
pub const GIF_P3TAG: u32 = 0x1000_30A0;
pub const GIF_FIFO: u32 = 0x1000_6000;
const VIF1_STAT: u32 = 0x1000_3C00;

/// GIF_STAT's fields (EE User's Manual §7.6), for the records.
pub mod st {
    pub const M3R: u32 = 1;
    pub const M3P: u32 = 1 << 1;
    pub const IMT: u32 = 1 << 2;
    pub const PSE: u32 = 1 << 3;
    pub const IP3: u32 = 1 << 5;
    pub const P3Q: u32 = 1 << 6;
    pub const P2Q: u32 = 1 << 7;
    pub const P1Q: u32 = 1 << 8;
    pub const OPH: u32 = 1 << 9;
    pub const DIR: u32 = 1 << 12;
    pub const fn apath(p: u32) -> u32 {
        p << 10
    }
    pub const fn fqc(n: u32) -> u32 {
        n << 24
    }
    /// Every defined field.
    pub const ALL: u32 = 0x1F00_1FEF;
}

/// The frame every program draws into and reads back: 64×32 PSMCT32 at page 100; a Z buffer at page 101.
pub const FRAME: Buf = Buf::new(100, 1, CT32, 64, 32);
pub const ZBUF: Buf = Buf::new(101, 1, Z32, 64, 32);
/// The packets' buffers in main memory, each 128-byte aligned and used once.
const BUF0: u32 = 0x0040_0000;
/// The qword of VU1's memory the kick microprogram reads its packet's address from.
const KICK_SLOT: u32 = 0x200;

/// LABEL with every IDMSK bit: SIGLBLID's LBLID becomes `id`.
pub fn label(id: u32) -> (u8, u64) {
    (gr::LABEL, u64::from(id) | 0xFFFF_FFFF << 32)
}

/// A packet of one LABEL.
pub fn label_packet(id: u32) -> Vec<u128> {
    regs_packet(&[label(id)])
}

/// A sprite's registers: PRIM, RGBAQ and the two corners in whole pixels of the frame.
pub fn sprite(c: [u32; 4], (x0, y0): (i32, i32), (x1, y1): (i32, i32)) -> Vec<(u8, u64)> {
    vec![s17::prim(SPRITE, 0), s17::rgba(c), s17::xyz2(16 * x0, 16 * y0, 0), s17::xyz2(16 * x1, 16 * y1, 0)]
}

/// A packed RGBAQ qword (§7.3.2), with `junk` in the bits the packing drops.
pub fn p_rgba(c: [u32; 4], junk: u32) -> u128 {
    quad(c.map(|v| v & 0xFF | (junk & 0xFF_FFFF) << 8))
}

/// A packed XYZ2 qword for window pixel (`x`, `y`) at Z `z`, `adc` its bit 111, with `junk` in the dropped bits.
pub fn p_xyz(x: i32, y: i32, z: u32, adc: bool, junk: u32) -> u128 {
    let v = s17::xyz2(16 * x, 16 * y, 0).1;
    let w3 = (junk & !0x8000) | u32::from(adc) << 15;
    quad([v as u32 & 0xFFFF | junk << 16, (v >> 16) as u32 & 0xFFFF | junk << 16, z, w3])
}

/// A packed XYZF2 qword: Z's 24 bits at 91..68 and F at 107..100.
pub fn p_xyzf(x: i32, y: i32, z: u32, f: u32, adc: bool, junk: u32) -> u128 {
    let v = s17::xyz2(16 * x, 16 * y, 0).1;
    let w2 = (z & 0xFF_FFFF) << 4 | junk & 0xF0000000 | junk & 0xF;
    let w3 = (f & 0xFF) << 4 | u32::from(adc) << 15 | junk & 0xFFFF_0000 | junk & 0x700F;
    quad([v as u32 & 0xFFFF | junk << 16, (v >> 16) as u32 & 0xFFFF | junk << 16, w2, w3])
}

/// The kick microprogram: VU1 reads its packet's address from KICK_SLOT and gives it to XGKICK.
pub fn kick_micro() -> Vec<u64> {
    s14::micro(&format!(
        "nop | ilw.x vi1, 0x{KICK_SLOT:x}(vi0)\nnop | move vf0, vf0\nnop | move vf0, vf0\nnop | move vf0, vf0\nnop | move vf0, vf0\nnop | xgkick vi1\nnop | move vf0, vf0\n[e] nop | move vf0, vf0\nnop | move vf0, vf0"
    ))
}

/// One program and the model of what it does to the GS.
pub struct Build {
    pub p: Program,
    pub gs: s17::manual::Model,
    pub gif: Model,
    buf: u32,
}

impl Build {
    /// S14's start (COP2 usable, the DMAC and INTC cleared, both VUs and VIFs reset), then the GIF reset with GIF_MODE
    /// 0, MSKPATH3 cleared and the kick microprogram loaded into VU1 by VIF1, CSR's events cleared, and S17's drawing
    /// environment into the frame.
    pub fn new(name: &'static str) -> Build {
        let mut b = Build { p: Program::kernel(name), gs: s17::manual::Model::default(), gif: Model::default(), buf: BUF0 };
        s14::prologue(&mut b.p);
        poke(&mut b.p, GIF_CTRL, 1);
        poke(&mut b.p, GIF_MODE, 0);
        let mut s = Stream::new();
        s.code(0x06, 0, 0).mpg(0, &kick_micro());
        b.vif1(&s, true);
        b.privileged(ga::CSR, 0x1F);
        let mut ops = Vec::new();
        begin(&mut ops);
        target(&mut ops, &FRAME, None, test(true, ALWAYS));
        for op in ops {
            if let s16::Op::Regs(list) = op {
                b.regs(&list);
            }
        }
        b.regs(&sprite([0x10, 0x20, 0x30, 0x40], (0, 0), (64, 32)));
        b
    }

    pub fn finish(self) -> Program {
        self.p
    }

    fn alloc(&mut self, qwords: usize) -> u32 {
        let at = self.buf;
        self.buf += (16 * qwords as u32).next_multiple_of(128);
        at
    }

    /// Qwords stored at physical `at` through kseg1.
    pub fn store(&mut self, at: u32, data: &[u128]) {
        for (i, &q) in data.iter().enumerate() {
            if i % 1024 == 0 {
                li(&mut self.p, "t6", K1 | (at + 16 * i as u32));
            }
            self.p.quad("t7", q);
            self.p.emit(format!("sq t7, {}(t6)", 16 * (i % 1024)));
        }
    }

    /// A privileged GS register written by SD at its GS address, and in the model.
    pub fn privileged(&mut self, g: u32, v: u64) {
        li(&mut self.p, "t6", K1 | ee_address(g));
        self.p.constant("t7", v);
        self.p.emit("sd t7, 0(t6)");
        self.gs.write_privileged(g, v);
    }

    /// A packet down PATH3 by normal-mode DMA from `at`; waits for the channel's end when asked.
    pub fn dma3_at(&mut self, at: u32, data: &[u128], wait_end: bool) {
        self.store(at, data);
        poke(&mut self.p, madr(2), at);
        poke(&mut self.p, qwc(2), data.len() as u32);
        poke(&mut self.p, chcr(2), 0x101);
        if wait_end {
            wait(&mut self.p, chcr(2), 0x100, false, 200_000);
        }
    }

    pub fn dma3(&mut self, data: &[u128], wait_end: bool) {
        let at = self.alloc(data.len());
        self.dma3_at(at, data, wait_end);
    }

    /// Packets down PATH3 by source-chain DMA, a cnt tag before each and an end tag after, the tags sent with TTE when
    /// asked (their upper 64 bits zero).
    pub fn chain3(&mut self, packets: &[Vec<u128>], tte: bool) {
        let mut q = Vec::new();
        for pk in packets {
            q.push(quad(crate::s11::tag(1, pk.len() as u32, 0, false, 0)));
            q.extend_from_slice(pk);
        }
        q.push(quad(crate::s11::tag(7, 0, 0, false, 0)));
        let at = self.alloc(q.len());
        self.store(at, &q);
        poke(&mut self.p, tadr(2), at);
        poke(&mut self.p, qwc(2), 0);
        poke(&mut self.p, chcr(2), 0x105 | if tte { 0x40 } else { 0 });
        wait(&mut self.p, chcr(2), 0x100, false, 200_000);
    }

    /// A packet stored qword by qword into the GIF's FIFO at 10006000h (PATH3 from the EE).
    pub fn fifo3(&mut self, data: &[u128]) {
        li(&mut self.p, "t6", K1 | GIF_FIFO);
        for &q in data {
            self.p.quad("t7", q);
            self.p.emit("sq t7, 0(t6)");
        }
    }

    /// A VIF1 stream by normal-mode DMA; with `idle`, waits for VIF1 to finish it too.
    pub fn vif1(&mut self, s: &Stream, idle: bool) {
        let at = self.alloc(s.qwords().len());
        if idle { s14::send(&mut self.p, 1, s, at) } else { s14::send_only(&mut self.p, 1, s, at) }
    }

    /// A packet down PATH2 by DIRECT, or DIRECTHL.
    pub fn path2(&mut self, data: &[u128], hl: bool, idle: bool) {
        let words: Vec<[u32; 4]> = data.iter().map(|&q| core::array::from_fn(|i| (q >> (32 * i)) as u32)).collect();
        let mut s = Stream::new();
        s.direct(hl, &words);
        self.vif1(&s, idle);
    }

    /// MSKPATH3 by VIF1.
    pub fn mskpath3(&mut self, on: bool) {
        let mut s = Stream::new();
        s.code(0x06, 0, if on { 0x8000 } else { 0 });
        self.vif1(&s, true);
    }

    /// A packet down PATH1: written into VU1's memory at qword `addr`, its address into KICK_SLOT, VU1 started at 0
    /// by CMSAR1 and its end awaited (VPU-STAT's VBS1); the transfer itself not awaited.
    pub fn kick(&mut self, data: &[u128], addr: u32) {
        for (i, &q) in data.iter().enumerate() {
            let a = (addr + i as u32) % 1024;
            self.store(VU_DATA[1] + 16 * a, &[q]);
        }
        self.store(VU_DATA[1] + 16 * KICK_SLOT, &[u128::from(addr)]);
        self.p.emit("ctc2 zero, vi31");
        let (l, out) = (self.p.label(), self.p.label());
        li(&mut self.p, "t5", 20_000);
        self.p.place(&l);
        self.p.emit("cfc2 t2, vi29");
        self.p.emit("andi t2, t2, 0x100");
        self.p.emit(format!("beq t2, zero, {out}"));
        self.p.emit("nop");
        self.p.emit("addiu t5, t5, -1");
        self.p.emit(format!("bne t5, zero, {l}"));
        self.p.emit("nop");
        self.p.place(&out);
    }

    /// Waits, bounded, for the GIF to be idle: no path transferring or waiting, the FIFO empty.
    pub fn gif_idle(&mut self) {
        s14::wait_clear(&mut self.p, GIF_STAT, 0x1F00_0FE0, 20_000);
    }

    /// The packet's writes reach the model's GS now; the caller states the arbitration's order.
    pub fn arrive(&mut self, data: &[u128]) {
        let (w, n) = self.gif.packet(data);
        assert_eq!(n, data.len(), "{}: a whole packet or whole primitives", self.p.name);
        self.apply(&w);
    }

    pub fn apply(&mut self, writes: &[(u8, u64)]) {
        for &(r, v) in writes {
            self.gs.write(r, v);
        }
    }

    /// A packet down PATH3, waited for, and in the model.
    pub fn send3(&mut self, data: &[u128]) {
        self.dma3(data, true);
        self.arrive(data);
    }

    /// General registers through one A+D packet down PATH3.
    pub fn regs(&mut self, list: &[(u8, u64)]) {
        self.send3(&regs_packet(list));
    }

    /// A GIF register's word ANDed with `mask` recorded, expecting `want` (none: ungraded).
    pub fn rec_reg(&mut self, pa: u32, mask: u32, want: Option<u32>) {
        li(&mut self.p, "t6", K1 | pa);
        self.p.emit("lw t2, 0(t6)");
        li(&mut self.p, "t7", mask);
        self.p.emit("and t2, t2, t7");
        self.p.record("t2", want.map(|w| u64::from(w & mask)));
    }

    pub fn rec_stat(&mut self, want: u32) {
        self.rec_reg(GIF_STAT, st::ALL, Some(want));
    }

    /// SIGLBLID's LBLID recorded, the model's expected.
    pub fn rec_label(&mut self) {
        li(&mut self.p, "t6", K1 | ee_address(ga::SIGLBLID));
        self.p.emit("lw t2, 4(t6)");
        let want = (self.gs.mem.read_privileged(ga::SIGLBLID) >> 32) as u32;
        self.p.record("t2", Some(want as i32 as i64 as u64));
    }

    /// GS User's Manual §4.2.2's steps 1 to 3 through PATH3.
    pub fn finish_handshake(&mut self) {
        self.privileged(ga::CSR, 2);
        self.send3(&regs_packet(&[(gr::FINISH, 0)]));
        wait(&mut self.p, ee_address(ga::CSR), 2, true, 200_000);
        self.privileged(ga::CSR, 2);
    }

    /// A rectangle of local memory read back local to host through VIF1 into the record (§4.2.2's steps 4 to 7), as
    /// S16's programs do; the model's words expected.
    pub fn readback(&mut self, (bp, bw, psm): (u32, u32, u32), (x, y, w, h): (u32, u32, u32, u32)) {
        let q = s16::qwords(psm, w, h);
        assert!(q.is_multiple_of(8), "whole 128-byte units");
        self.finish_handshake();
        self.regs(&[(gr::BITBLTBUF, bitbltbuf(bp, bw, psm, 0, 0, 0)), (gr::TRXPOS, trxpos(x, y, 0, 0, 0)), (gr::TRXREG, trxreg(w, h)), (gr::TRXDIR, 1)]);
        if self.p.expected.len() % 2 == 1 {
            self.p.record("zero", None);
        }
        let target = RECORD + 8 * self.p.expected.len() as u32;
        self.privileged(ga::BUSDIR, 1);
        poke(&mut self.p, VIF1_STAT, 0x0080_0000);
        poke(&mut self.p, madr(1), target);
        poke(&mut self.p, qwc(1), q);
        poke(&mut self.p, chcr(1), 0x100);
        wait(&mut self.p, chcr(1), 0x100, false, 400_000);
        poke(&mut self.p, VIF1_STAT, 0);
        self.privileged(ga::BUSDIR, 0);
        self.gs.write_privileged(ga::BUSDIR, 1);
        for _ in 0..2 * q {
            let v = self.gs.read();
            self.p.expected.push(Some(v));
        }
        self.gs.write_privileged(ga::BUSDIR, 0);
        li(&mut self.p, "t7", 16 * q);
        self.p.emit("daddu s1, s1, t7");
    }

    /// The whole frame read back.
    pub fn read_frame(&mut self) {
        self.readback((FRAME.page * 32, 1, CT32), (0, 0, 64, 32));
    }
}

/// `s15-packed`: PACKED mode through PATH3 under every descriptor that draws, with junk in every bit the packing
/// drops: PRE's PRIM field; RGBAQ; XYZ2 and XYZF2 with ADC 0 and 1 (no drawing kick); PRIM as data with PRE 0; A+D;
/// NOP; sixteen descriptors (NREG 0); a tag with NLOOP 0 inside a packet and one ending it; XYZF3 and XYZ3 as raw
/// descriptors; and the descriptors no picture shows (ST, UV, FOG, TEX0, CLAMP, 0Bh). The frame and a corner of the
/// Z buffer read back.
pub fn packed() -> Program {
    let mut b = Build::new("s15-packed");
    let colours: Vec<[u32; 4]> = (0..24).map(s17::colour).collect();
    b.regs(&[(s17::r::ZBUF_1, zbuf(ZBUF.page, Z32, false))]);
    b.regs(&sprite(colours[23], (0, 0), (16, 16)));
    b.regs(&[(s17::r::ZBUF_1, zbuf(0, Z32, true))]);
    // Three sprites from PRE's PRIM field, REGS {RGBAQ, XYZ2, XYZ2}.
    let mut pk = vec![tag(3, true, Some(SPRITE as u32), 0, &[0x1, 0x5, 0x5])];
    for k in 0..3 {
        let x = 2 + 7 * k as i32;
        pk.extend([p_rgba(colours[k], 0xA5_5A00 | k as u32), p_xyz(x, 1, 0, false, 0x5A5A_1234), p_xyz(x + 5, 6, 0, false, 0x2468_ACE0)]);
    }
    b.send3(&pk);
    // XYZF2 into the Z buffer, ADC 0, and a sprite whose second vertex has ADC 1 (to XYZF3: no drawing kick).
    b.regs(&[(s17::r::ZBUF_1, zbuf(ZBUF.page, Z32, false))]);
    let mut pk = vec![tag(2, true, Some(SPRITE as u32), 0, &[0x1, 0x4, 0x4])];
    pk.extend([p_rgba(colours[3], 0xFF_FFFF), p_xyzf(1, 8, 0x12_3456, 0x9C, false, 0xFFFF_FFFF), p_xyzf(6, 13, 0xAB_CDEF, 0x37, false, 0x8765_4321)]);
    pk.extend([p_rgba(colours[4], 0), p_xyzf(8, 8, 0x55_5555, 0, false, 0), p_xyzf(13, 13, 0x66_6666, 0, true, 0)]);
    b.send3(&pk);
    b.regs(&[(s17::r::ZBUF_1, zbuf(0, Z32, true))]);
    // PRE 0, PRIM as data (junk above bit 10), and A+D with junk above bit 71, NOP between.
    let mut pk = vec![tag(1, false, None, 0, &[0x0, 0x1, 0xF, 0x5, 0x5])];
    pk.extend([quad([SPRITE as u32 | 0xFFFF_F800, 0xFFFF_FFFF, 0xFFFF_FFFF, 0xFFFF_FFFF]), p_rgba(colours[5], 0x11), p_xyz(30, 30, 0, false, 0), p_xyz(15, 8, 0, false, 0), p_xyz(20, 13, 0, false, 0)]);
    pk.push(tag(2, true, None, 0, &[0xE]));
    pk.push(ad(gr::LABEL, 0xFFFF_FFFF_0000_0A01) | 0xFF_FFFF_FFFF_FFFFu128 << 72);
    pk.push(ad(s17::r::RGBAQ, s17::rgba(colours[6]).1) | 0xDEAD_BEEFu128 << 96);
    b.send3(&pk);
    b.rec_label();
    // Sixteen descriptors (NREG 0): five sprites and a NOP, PRE's PRIM.
    let mut regs = Vec::new();
    for _ in 0..5 {
        regs.extend([0x1, 0x5, 0x5]);
    }
    regs.push(0xF);
    let mut pk = vec![tag(1, true, Some(SPRITE as u32), 0, &regs)];
    for k in 0..5 {
        let x = 1 + 6 * k as i32;
        pk.extend([p_rgba(colours[7 + k], 0), p_xyz(x, 16, 0, false, 0), p_xyz(x + 4, 20, 0, false, 0)]);
    }
    pk.push(p_xyz(0, 0, 0, false, 0));
    b.send3(&pk);
    // NLOOP 0 inside a packet (EOP clear, then a sprite), a packet ending in an NLOOP-0 EOP tag, and a second packet
    // sent in the same DMA.
    let mut pk = vec![tag(0, false, Some(0), 2, &[0x1, 0x2, 0x3])];
    pk.extend([tag(1, false, Some(SPRITE as u32), 0, &[0x1, 0x5, 0x5]), p_rgba(colours[12], 0), p_xyz(32, 16, 0, false, 0), p_xyz(36, 20, 0, false, 0)]);
    pk.push(tag(0, true, None, 0, &[]));
    let second = [tag(1, true, Some(SPRITE as u32), 0, &[0x1, 0x5, 0x5]), p_rgba(colours[13], 0), p_xyz(38, 16, 0, false, 0), p_xyz(42, 20, 0, false, 0)];
    let mut both = pk.clone();
    both.extend_from_slice(&second);
    b.dma3(&both, true);
    b.arrive(&pk);
    b.arrive(&second);
    // XYZ3 and XYZF3 as raw descriptors (the lower 64 bits, no kick), each first vertex of a sprite kicked by XYZ2.
    let mut pk = vec![tag(1, true, Some(SPRITE as u32), 0, &[0x1, 0xD, 0x5, 0x1, 0xC, 0x5])];
    pk.extend([p_rgba(colours[14], 0), u128::from(s17::xyz2(16 * 44, 16 * 16, 0).1) | 0xFFFFu128 << 112, p_xyz(48, 20, 0, false, 0)]);
    pk.extend([p_rgba(colours[15], 0), u128::from(s17::vertex(s17::r::XYZF3, 16 * 50, 16 * 16, 0).1), p_xyz(54, 20, 0, false, 0)]);
    b.send3(&pk);
    // What no picture shows: ST (Q), RGBAQ after it, UV, FOG, TEX0_1, TEX0_2, CLAMP_1, CLAMP_2 and the reserved 0Bh.
    let mut pk = vec![tag(1, true, None, 0, &[0x2, 0x1, 0x3, 0xA, 0x6, 0x7, 0x8, 0x9, 0xB])];
    pk.extend([quad([0x3F00_0000, 0x3E80_0000, 0x4000_0000, 0xFFFF_FFFF]), p_rgba(colours[16], 0), quad([0x3FFF, 0xFFFF_C000 | 0x1234, 0, 0])]);
    pk.extend([quad([0, 0, 0, 0xA5 << 4]), ad(0, 0), ad(0, 0), ad(0, 0), ad(0, 0), quad([0xFFFF_FFFF; 4])]);
    b.send3(&pk);
    b.rec_label();
    b.read_frame();
    b.readback((ZBUF.page * 32, 1, CT32), (0, 0, 16, 16));
    b.finish()
}

/// `s15-reglist`: REGLIST mode (PRE's PRIM ignored, A+D and NOP descriptors skipped, an odd count's last half
/// dropped) and IMAGE mode (FLG 2 and 3, two primitives in one packet, a PACKED primitive between two IMAGE ones).
pub fn reglist() -> Program {
    let mut b = Build::new("s15-reglist");
    let c = |k: u32| s17::colour(k + 30);
    // REGLIST {RGBAQ, XYZ2, XYZ2} NLOOP 3 under PRE with PRIM = POINT (ignored), the last qword's upper half an RGBAQ.
    b.regs(&[s17::prim(SPRITE, 0)]);
    let mut pk = vec![tag(3, true, Some(0), 1, &[0x1, 0x5, 0x5])];
    let mut halves = Vec::new();
    for k in 0..3 {
        let x = 2 + 7 * k as i32;
        halves.extend([s17::rgba(c(k)).1, s17::xyz2(16 * x, 16, 0).1, s17::xyz2(16 * (x + 5), 16 * 6, 0).1]);
    }
    halves.push(s17::rgba([0xFF, 0, 0, 0xFF]).1);
    pk.extend(halves.chunks(2).map(|h| u128::from(h[0]) | u128::from(h[1]) << 64));
    b.send3(&pk);
    // A sprite by A+D with no RGBAQ: the colour is REGLIST's last, not the dropped half's.
    b.regs(&[s17::prim(SPRITE, 0), s17::xyz2(16 * 24, 16, 0), s17::xyz2(16 * 29, 16 * 6, 0)]);
    // REGLIST {PRIM, A+D, RGBAQ, NOP, XYZ2, XYZ2} NLOOP 2.
    let mut pk = vec![tag(2, true, None, 1, &[0x0, 0xE, 0x1, 0xF, 0x5, 0x5])];
    for k in 0..2u32 {
        let x = 32 + 7 * k as i32;
        let h = [SPRITE, 0x0000_0000_0000_0000, s17::rgba(c(5 + k)).1, s17::xyz2(0, 0, 0).1, s17::xyz2(16 * x, 16, 0).1, s17::xyz2(16 * (x + 5), 16 * 6, 0).1];
        pk.extend(h.chunks(2).map(|h| u128::from(h[0]) | u128::from(h[1]) << 64));
    }
    b.send3(&pk);
    b.regs(&[label(0x0B01)]);
    b.rec_label();
    // IMAGE: a 16×8 rectangle by FLG 2 then FLG 3 in one packet, and another with a PACKED primitive between.
    let image = |seed: u32, n: usize| -> Vec<u128> { (0..n as u32).map(|i| quad(core::array::from_fn(|j| (seed ^ (4 * i + j as u32)).wrapping_mul(0x9E37_79B9)))).collect() };
    let up = |b: &mut Build, x: u32, y: u32| b.regs(&[(gr::BITBLTBUF, bitbltbuf(0, 0, 0, FRAME.page * 32, 1, CT32)), (gr::TRXPOS, trxpos(0, 0, x, y, 0)), (gr::TRXREG, trxreg(16, 8)), (gr::TRXDIR, 0)]);
    up(&mut b, 0, 24);
    let mut pk = vec![tag(16, false, None, 2, &[])];
    pk.extend(image(1, 16));
    pk.push(tag(16, true, Some(3), 3, &[0x1]));
    pk.extend(image(2, 16));
    b.send3(&pk);
    up(&mut b, 16, 24);
    let mut pk = vec![tag(8, false, None, 2, &[])];
    pk.extend(image(3, 8));
    pk.extend([tag(1, false, None, 0, &[0xE]), ad(gr::LABEL, label(0x0B02).1)]);
    pk.push(tag(24, true, None, 2, &[]));
    pk.extend(image(4, 24));
    b.send3(&pk);
    b.rec_label();
    b.read_frame();
    b.finish()
}

/// Two sprites and a LABEL in region `k` of the frame.
fn region_packet(k: u32) -> Vec<u128> {
    let x = 8 * k as i32;
    let mut regs = sprite(s17::colour(40 + 2 * k), (x, 2), (x + 7, 9));
    regs.extend(sprite(s17::colour(41 + 2 * k), (x + 2, 12), (x + 6, 20)));
    regs.push(label(0x0C00 + k));
    regs_packet(&regs)
}

/// `s15-paths`: the same kind of packet down every path, each into its own region of the frame: PATH3 by normal DMA,
/// by chain DMA without and with TTE, and by the EE's stores into the FIFO; PATH2 by DIRECT and DIRECTHL; PATH1 by
/// XGKICK, once inside VU1's memory and once across its end. SIGLBLID after each, then the frame.
pub fn paths() -> Program {
    let mut b = Build::new("s15-paths");
    let pk = region_packet(0);
    b.send3(&pk);
    b.rec_label();
    let pk = region_packet(1);
    b.chain3(&[pk.clone()], false);
    b.arrive(&pk);
    b.rec_label();
    let pk = region_packet(2);
    b.chain3(&[pk.clone()], true);
    b.arrive(&pk);
    b.rec_label();
    let pk = region_packet(3);
    b.fifo3(&pk);
    b.gif_idle();
    b.arrive(&pk);
    b.rec_label();
    let pk = region_packet(4);
    b.path2(&pk, false, true);
    b.arrive(&pk);
    b.rec_label();
    let pk = region_packet(5);
    b.path2(&pk, true, true);
    b.arrive(&pk);
    b.rec_label();
    let pk = region_packet(6);
    b.kick(&pk, 0x40);
    b.gif_idle();
    b.arrive(&pk);
    b.rec_label();
    let pk = region_packet(7);
    b.kick(&pk, 1024 - 4);
    b.gif_idle();
    b.arrive(&pk);
    b.rec_label();
    b.read_frame();
    b.finish()
}

/// The two halves of a packet of two LABEL primitives, the first without EOP: the first half leaves its path holding
/// the GIF mid-packet.
fn halves(a: u32, z: u32) -> (Vec<u128>, Vec<u128>) {
    (vec![tag(1, false, None, 0, &[0xE]), ad(gr::LABEL, label(a).1)], label_packet(z))
}

/// `s15-arbitration`: one path held mid-packet while others ask (§7.1.2): PATH2 waits for PATH3's packet to end, PATH1
/// is queued behind it without stalling VU1, and PATH1 goes before PATH2 when both wait; PATH3 held by MSKPATH3 and by
/// GIF_MODE's M3R, its data kept in the FIFO (and the DMA held once the FIFO is full), M3R set during a packet taking
/// effect after it (§7.1.4); and the intermittent mode (§7.1.3): DIRECT and XGKICK taking the GIF from PATH3's IMAGE
/// data at a slice's end, DIRECTHL and the continuous mode not. GIF_STAT, VIF1's VGW and SIGLBLID at each step.
pub fn arbitration() -> Program {
    let mut b = Build::new("s15-arbitration");
    let held = st::OPH | st::apath(3);
    // PATH2 behind PATH3.
    let (first, rest) = halves(0x0D31, 0x0D32);
    b.dma3(&first, true);
    b.arrive(&first);
    b.rec_stat(held);
    b.path2(&label_packet(0x0D21), false, false);
    b.rec_stat(held | st::P2Q);
    b.rec_reg(VIF1_STAT, 0x8, Some(0x8));
    b.rec_label();
    b.dma3(&rest, true);
    b.arrive(&rest);
    s14::idle(&mut b.p, 1);
    b.arrive(&label_packet(0x0D21));
    b.rec_label();
    b.rec_stat(0);
    // PATH1 queued behind PATH3; VU1 runs to its end.
    let (first, rest) = halves(0x0D33, 0x0D34);
    b.dma3(&first, true);
    b.arrive(&first);
    b.kick(&label_packet(0x0D11), 0x40);
    b.rec_stat(held | st::P1Q);
    b.rec_label();
    b.dma3(&rest, true);
    b.arrive(&rest);
    b.gif_idle();
    b.arrive(&label_packet(0x0D11));
    b.rec_label();
    b.rec_stat(0);
    // PATH1 before PATH2 when both wait.
    let (first, rest) = halves(0x0D35, 0x0D36);
    b.dma3(&first, true);
    b.arrive(&first);
    b.path2(&label_packet(0x0D22), false, false);
    b.kick(&label_packet(0x0D12), 0x40);
    b.rec_stat(held | st::P1Q | st::P2Q);
    b.dma3(&rest, true);
    b.arrive(&rest);
    s14::idle(&mut b.p, 1);
    b.gif_idle();
    b.arrive(&label_packet(0x0D12));
    b.arrive(&label_packet(0x0D22));
    b.rec_label();
    b.rec_stat(0);
    // MSKPATH3: a short packet kept in the FIFO while PATH2 goes, sent once the mask is cleared.
    b.mskpath3(true);
    b.rec_stat(st::M3P);
    let pk = label_packet(0x0D37);
    b.dma3(&pk, true);
    b.rec_stat(st::M3P | st::P3Q | st::fqc(2));
    b.rec_label();
    b.path2(&label_packet(0x0D23), false, true);
    b.arrive(&label_packet(0x0D23));
    b.rec_label();
    b.mskpath3(false);
    b.gif_idle();
    b.arrive(&pk);
    b.rec_label();
    b.rec_stat(0);
    // A masked packet longer than the FIFO: the DMA holds with 16 qwords in it.
    b.mskpath3(true);
    let mut long = vec![tag(23, true, None, 0, &[0xE])];
    long.extend((0..23).map(|k| ad(gr::LABEL, label(0x0D40 + k).1)));
    b.dma3(&long, false);
    crate::s11::delay(&mut b.p, 2000);
    b.rec_reg(qwc(2), 0xFFFF, Some(8));
    b.rec_reg(chcr(2), 0x100, Some(0x100));
    b.rec_stat(st::M3P | st::P3Q | st::fqc(16));
    b.mskpath3(false);
    wait(&mut b.p, chcr(2), 0x100, false, 200_000);
    b.gif_idle();
    b.arrive(&long);
    b.rec_label();
    // GIF_MODE's M3R: set during a packet, which ends; the next packet held until it is cleared.
    let (first, rest) = halves(0x0D38, 0x0D39);
    b.dma3(&first, true);
    b.arrive(&first);
    poke(&mut b.p, GIF_MODE, 1);
    b.rec_stat(st::M3R | held);
    b.dma3(&rest, true);
    b.arrive(&rest);
    b.rec_label();
    b.rec_stat(st::M3R);
    let pk = label_packet(0x0D3A);
    b.dma3(&pk, true);
    b.rec_stat(st::M3R | st::P3Q | st::fqc(2));
    poke(&mut b.p, GIF_MODE, 0);
    b.gif_idle();
    b.arrive(&pk);
    b.rec_label();
    // The intermittent mode: a 16×8 IMAGE upload of 32 qwords, held after its first slice of 8.
    let image = |seed: u32| -> Vec<u128> { (0..32u32).map(|i| quad(core::array::from_fn(|j| (seed.wrapping_add(4 * i + j as u32)).wrapping_mul(0x2545_F491)))).collect() };
    let trx = [(gr::BITBLTBUF, bitbltbuf(0, 0, 0, FRAME.page * 32, 1, CT32)), (gr::TRXPOS, trxpos(0, 0, 48, 24, 0)), (gr::TRXREG, trxreg(16, 8)), (gr::TRXDIR, 0)];
    let cases: [(u32, Option<bool>, u32); 4] = [(mode_imt(), Some(false), 0x0D24), (mode_imt(), Some(true), 0x0D25), (0, Some(false), 0x0D26), (mode_imt(), None, 0x0D13)];
    for (k, &(m, two, id)) in cases.iter().enumerate() {
        poke(&mut b.p, GIF_MODE, m);
        b.regs(&trx);
        let data = image(k as u32 * 0x1000);
        let mut start = vec![tag(32, true, None, 2, &[])];
        start.extend_from_slice(&data[..8]);
        let at = b.alloc(16) + 0x70;
        b.dma3_at(at, &start, true);
        crate::s11::delay(&mut b.p, 100);
        let imt = m != 0;
        let base = if imt { st::IMT } else { 0 };
        b.rec_stat(base | held);
        let other = label_packet(id);
        match two {
            Some(hl) => b.path2(&other, hl, false),
            None => b.kick(&other, 0x40),
        }
        let (all, _) = b.gif.packet(&[&start[..], &data[8..]].concat());
        let interrupts = imt && two != Some(true);
        if interrupts {
            if two.is_some() {
                s14::idle(&mut b.p, 1);
            } else {
                crate::s11::delay(&mut b.p, 100);
            }
            b.apply(&all[..16]);
            b.arrive(&other);
            b.rec_stat(base | held);
            poke(&mut b.p, GIF_CTRL, 8);
            b.rec_reg(GIF_P3CNT, 0x7FFF, Some(24));
            b.rec_reg(GIF_P3TAG, 0xFFFF, Some(0x8020));
            poke(&mut b.p, GIF_CTRL, 0);
        } else {
            b.rec_stat(base | held | if two.is_some() { st::P2Q } else { st::P1Q });
        }
        b.rec_label();
        let rest = data[8..].to_vec();
        b.dma3(&rest, true);
        if two.is_some() {
            s14::idle(&mut b.p, 1);
        }
        b.gif_idle();
        if interrupts {
            b.apply(&all[16..]);
        } else {
            b.apply(&all);
            b.arrive(&other);
        }
        b.rec_label();
        b.rec_stat(base);
    }
    poke(&mut b.p, GIF_MODE, 0);
    b.readback((FRAME.page * 32, 1, CT32), (48, 24, 16, 8));
    b.finish()
}

const fn mode_imt() -> u32 {
    4
}

/// `s15-regs`: GIF_STAT at rest and after GIF_MODE's writes; GIF_CTRL.PSE pausing a packet mid-way, with GIF_TAG0 to
/// GIF_TAG3 and GIF_CNT read meanwhile, and pausing a packet's start; RST ending a packet in progress, clearing
/// GIF_MODE and keeping MSKPATH3's mask; DIR with BUSDIR; PATH3 by the EE's stores while M3R holds it; and the
/// write-only registers read (ungraded).
pub fn regs() -> Program {
    let mut b = Build::new("s15-regs");
    b.rec_stat(0);
    for m in [1, 4, 5, 0] {
        poke(&mut b.p, GIF_MODE, m);
        b.rec_stat(m);
    }
    b.rec_reg(GIF_CTRL, u32::MAX, None);
    b.rec_reg(GIF_MODE, u32::MAX, None);
    // PSE mid-packet: NLOOP 3 of {A+D, NOP}, PRE with PRIM 0, held after its third data qword.
    let t = tag(3, true, Some(0x123), 0, &[0xE, 0xF]);
    let full = [t, ad(gr::LABEL, label(0x0E51).1), quad([0xFFFF_FFFF; 4]), ad(gr::LABEL, label(0x0E52).1), quad([0x1234; 4]), ad(gr::LABEL, label(0x0E53).1), quad([0; 4])];
    b.dma3(&full[..4], true);
    poke(&mut b.p, GIF_CTRL, 8);
    b.rec_stat(st::PSE | st::OPH | st::apath(3));
    for (i, w) in [t as u32, (t >> 32) as u32, (t >> 64) as u32, (t >> 96) as u32].into_iter().enumerate() {
        b.rec_reg(GIF_TAG0 + 0x10 * i as u32, u32::MAX, Some(w));
    }
    b.rec_reg(GIF_CNT, 0x000F_7FFF, Some(2 | 2 << 16));
    poke(&mut b.p, GIF_CTRL, 0);
    b.dma3(&full[4..], true);
    b.gif_idle();
    b.arrive(&full);
    b.rec_label();
    // PSE before a packet: it waits in the FIFO.
    poke(&mut b.p, GIF_CTRL, 8);
    let pk = label_packet(0x0E54);
    b.dma3(&pk, true);
    b.rec_stat(st::PSE | st::P3Q | st::fqc(2));
    b.rec_label();
    poke(&mut b.p, GIF_CTRL, 0);
    b.gif_idle();
    b.arrive(&pk);
    b.rec_label();
    // RST mid-packet: the GIF free for PATH2 at once.
    let (first, _) = halves(0x0E55, 0);
    b.dma3(&first, true);
    b.arrive(&first);
    poke(&mut b.p, GIF_CTRL, 1);
    b.rec_stat(0);
    b.path2(&label_packet(0x0E27), false, true);
    b.arrive(&label_packet(0x0E27));
    b.rec_label();
    // RST clears GIF_MODE and keeps MSKPATH3's mask (D-284).
    poke(&mut b.p, GIF_MODE, 5);
    b.mskpath3(true);
    poke(&mut b.p, GIF_CTRL, 1);
    b.rec_stat(st::M3P);
    b.mskpath3(false);
    b.rec_stat(0);
    // DIR follows BUSDIR.
    b.privileged(ga::BUSDIR, 1);
    b.rec_stat(st::DIR);
    b.privileged(ga::BUSDIR, 0);
    b.rec_stat(0);
    // The EE's stores into the FIFO while M3R holds PATH3.
    poke(&mut b.p, GIF_MODE, 1);
    let pk = label_packet(0x0E58);
    b.fifo3(&pk);
    b.rec_stat(st::M3R | st::P3Q | st::fqc(2));
    poke(&mut b.p, GIF_MODE, 0);
    b.gif_idle();
    b.arrive(&pk);
    b.rec_label();
    b.rec_stat(0);
    b.finish()
}

pub fn all() -> Vec<Program> {
    vec![packed(), reglist(), paths(), arbitration(), regs(), intermittent()]
}

/// The second-DIRECT probe (Sapphire_Native.md §33.8): S14's `s14-direct` opening, two DIRECT packets of a LABEL each
/// with VIF1's CODE and STAT and SIGLBLID recorded after each, in four forms: as S14 sent it (no GIF reset, the
/// second stream followed by MSKPATH3 and the FLUSH family), with the GIF reset by GIF_CTRL first, with the second
/// stream DIRECT alone, and with a PATH3 packet between the two.
pub fn second_direct(form: u32) -> Program {
    let name = ["s15-direct-a", "s15-direct-b", "s15-direct-c", "s15-direct-d"][form as usize];
    let mut p = Program::kernel(name);
    s14::prologue(&mut p);
    let mut m = s17::manual::Model::default();
    if form == 1 {
        poke(&mut p, GIF_CTRL, 1);
        poke(&mut p, GIF_MODE, 0);
    }
    poke(&mut p, 0x1200_1000, 1);
    crate::s11::rec(&mut p, GIF_STAT, None);
    crate::s11::rec(&mut p, chcr(2), None);
    let mut at = 0x0034_0000 + 0x3_0000;
    let rec = |p: &mut Program, m: &mut s17::manual::Model, code: u32| {
        crate::s11::rec(p, s14::vreg(1, s14::r::CODE), Some(code));
        crate::s11::rec_masked(p, s14::vreg(1, s14::r::STAT), 0x1F00_000F, 0);
        crate::s11::rec(p, GIF_STAT, None);
        li(p, "t6", K1 | ee_address(ga::SIGLBLID));
        p.emit("lw t2, 4(t6)");
        let want = (m.mem.read_privileged(ga::SIGLBLID) >> 32) as u32 & 0xFFFF;
        p.emit("andi t2, t2, 0xffff");
        p.record("t2", Some(u64::from(want)));
    };
    let direct = |p: &mut Program, m: &mut s17::manual::Model, at: &mut u32, id: u32, tail: bool| {
        let pk = label_packet(id);
        let words: Vec<[u32; 4]> = pk.iter().map(|&q| core::array::from_fn(|i| (q >> (32 * i)) as u32)).collect();
        let mut s = Stream::new();
        s.direct(false, &words);
        if tail {
            s.code(0x06, 0, 0x8000).code(0x11, 0, 0).code(0x13, 0, 0).code(0x06, 0, 0);
        }
        s14::send(p, 1, &s, *at);
        *at += 0x400;
        for (r, v) in Model::default().packet(&pk).0 {
            m.write(r, v);
        }
        if tail { 0x0600_0000 } else { 0x5000_0002 }
    };
    let c = direct(&mut p, &mut m, &mut at, 0x1234, false);
    rec(&mut p, &mut m, c);
    if form == 3 {
        let pk = label_packet(0x2345);
        let mut b = Build { p: std::mem::replace(&mut p, Program::new(name)), gs: m, gif: Model::default(), buf: 0x0050_0000 };
        b.send3(&pk);
        (p, m) = (b.p, b.gs);
    }
    let c = direct(&mut p, &mut m, &mut at, 0x5678, form != 2);
    rec(&mut p, &mut m, c);
    s14::reset(&mut p, 1);
    p
}

/// `s15-intermittent`: the intermittent mode against a transfer in flight, which both referees can show: an IMAGE
/// packet of 4,096 qwords (a 128×128 PSMCT32 upload, its data whatever memory holds) started by DMA, and at once a
/// LABEL by DIRECT, DIRECTHL or XGKICK; SIGLBLID and D2_CHCR.STR read as soon as the other path has been given its
/// packet, and again once both are done. In the intermittent mode DIRECT and XGKICK reach the GS while the DMA still
/// runs, DIRECTHL and the continuous mode only after it (§7.1.3).
pub fn intermittent() -> Program {
    let mut b = Build::new("s15-intermittent");
    let n = 30720;
    let cases: [(u32, Option<bool>, u32); 4] = [(mode_imt(), Some(false), 0x0F24), (mode_imt(), Some(true), 0x0F25), (0, Some(false), 0x0F26), (mode_imt(), None, 0x0F13)];
    for &(m, two, id) in &cases {
        poke(&mut b.p, GIF_MODE, m);
        b.regs(&[(gr::BITBLTBUF, bitbltbuf(0, 0, 0, 110 * 32, 4, CT32)), (gr::TRXPOS, 0), (gr::TRXREG, trxreg(240, 512)), (gr::TRXDIR, 0)]);
        let at = b.alloc(n + 1);
        b.store(at, &[tag(n as u32, true, None, 2, &[])]);
        poke(&mut b.p, madr(2), at);
        poke(&mut b.p, qwc(2), n as u32 + 1);
        poke(&mut b.p, chcr(2), 0x101);
        let other = label_packet(id);
        match two {
            Some(hl) => b.path2(&other, hl, false),
            None => b.kick(&other, 0x40),
        }
        if two.is_some() {
            s14::wait_clear(&mut b.p, vreg(1, s14::r::STAT), 0x1F00_0003, 100);
        } else {
            crate::s11::delay(&mut b.p, 100);
        }
        let before = b.gs.mem.read_privileged(ga::SIGLBLID) >> 32;
        let overtakes = m != 0 && two != Some(true);
        li(&mut b.p, "t6", K1 | ee_address(ga::SIGLBLID));
        b.p.emit("lw t2, 4(t6)");
        b.p.record("t2", Some(if overtakes { u64::from(id) } else { before }));
        b.rec_reg(chcr(2), 0x100, Some(0x100));
        b.rec_reg(GIF_STAT, st::ALL, None);
        wait(&mut b.p, chcr(2), 0x100, false, 400_000);
        if two.is_some() {
            s14::idle(&mut b.p, 1);
        }
        b.gif_idle();
        b.gs.write_privileged(ga::BUSDIR, 0);
        b.gs.write(gr::BITBLTBUF, 0);
        b.apply(&[label(id)]);
        b.rec_label();
    }
    poke(&mut b.p, GIF_MODE, 0);
    b.finish()
}
