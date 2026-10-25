//! The project's S16 programs (Sapphire_Native.md §24.6): each a script of what the host does to the GS (general
//! registers through A+D, image data through IMAGE, privileged registers through SD and LD, a FINISH handshake, a
//! local-to-host read with BUSDIR reversed, the INTC's GS bit), written two ways. As an EE program (`program`) it sends
//! its packets through the GIF's DMA channel and reads local-to-host data through VIF1's channel into its record, for the
//! referees; on Sapphire, until the GIF (S15) and VIF1 (S14) exist, the tests drive the GS's host interface with the
//! same script. The expected record comes from `manual`, written apart from `src/gs/`.

#![allow(dead_code)]

pub mod manual;

use crate::s6::{Program, RECORD};
use crate::s11::{K1, li, peek, poke, wait};

/// One step of a script.
#[derive(Clone, Debug)]
pub enum Op {
    /// General registers, one A+D packet (EE User's Manual §7.2.4).
    Regs(Vec<(u8, u64)>),
    /// `qwords` of image data from the generator seeded with `seed`, one IMAGE packet.
    Image { seed: u32, qwords: u32 },
    /// A privileged register written by SD, by its GS address.
    Priv(u32, u64),
    /// A privileged register written through another EE address (an image, or a narrower store of `bytes`).
    PrivAt(u32, usize, u64),
    /// The FINISH handshake of §4.2.2's steps 1 to 3.
    Finish,
    /// §4.2.2's steps 5 to 7: BUSDIR 1, `qwords` read into the record, BUSDIR 0.
    Readback(u32),
    /// A privileged register read by LD at its GS address, ANDed with the mask, recorded; graded by the model if true.
    Record(u32, u64, bool),
    /// The same through another EE address and width.
    RecordAt(u32, usize, u64, bool),
    /// I_STAT's GS bit recorded, then cleared.
    Intc,
    /// I_STAT's GS bit cleared, unrecorded.
    ClearIntc,
    /// A write-only register read (ungraded), written with the pattern, read again (graded on CSR's events and FIFO),
    /// and left at 0.
    Probe(u32, u64),
    /// Waits, bounded, until a privileged register ANDed with the mask is non-zero.
    Wait(u32, u64),
}

pub struct Script {
    pub name: &'static str,
    pub ops: Vec<Op>,
}

pub mod ga {
    pub const PMODE: u32 = 0x00;
    pub const SMODE1: u32 = 0x01;
    pub const SMODE2: u32 = 0x02;
    pub const SRFSH: u32 = 0x03;
    pub const SYNCH1: u32 = 0x04;
    pub const SYNCH2: u32 = 0x05;
    pub const SYNCV: u32 = 0x06;
    pub const DISPFB1: u32 = 0x07;
    pub const DISPLAY1: u32 = 0x08;
    pub const DISPFB2: u32 = 0x09;
    pub const DISPLAY2: u32 = 0x0A;
    pub const EXTBUF: u32 = 0x0B;
    pub const EXTDATA: u32 = 0x0C;
    pub const EXTWRITE: u32 = 0x0D;
    pub const BGCOLOR: u32 = 0x0E;
    pub const CSR: u32 = 0x40;
    pub const IMR: u32 = 0x41;
    pub const BUSDIR: u32 = 0x44;
    pub const SIGLBLID: u32 = 0x48;
}

/// A GS address's EE address (EE User's Manual §2.2.3's table).
pub fn ee_address(ga: u32) -> u32 {
    0x1200_0000 | (ga >> 6) << 12 | (ga & 0x3F) << 4
}

pub mod r {
    pub const BITBLTBUF: u8 = 0x50;
    pub const TRXPOS: u8 = 0x51;
    pub const TRXREG: u8 = 0x52;
    pub const TRXDIR: u8 = 0x53;
    pub const HWREG: u8 = 0x54;
    pub const SIGNAL: u8 = 0x60;
    pub const FINISH: u8 = 0x61;
    pub const LABEL: u8 = 0x62;
}

/// The image generator: a 32-bit linear congruential sequence, each word the state after a step.
pub fn lcg(seed: u32, words: u32) -> Vec<u32> {
    let mut x = seed;
    (0..words)
        .map(|_| {
            x = x.wrapping_mul(0x41C6_4E6D).wrapping_add(0x3039);
            x
        })
        .collect()
}

/// The image data as the host's 64-bit units.
pub fn units(seed: u32, qwords: u32) -> Vec<u64> {
    lcg(seed, qwords * 4).chunks(2).map(|c| u64::from(c[0]) | u64::from(c[1]) << 32).collect()
}

pub fn bitbltbuf(sbp: u32, sbw: u32, spsm: u32, dbp: u32, dbw: u32, dpsm: u32) -> u64 {
    u64::from(sbp) | u64::from(sbw) << 16 | u64::from(spsm) << 24 | u64::from(dbp) << 32 | u64::from(dbw) << 48 | u64::from(dpsm) << 56
}

pub fn trxpos(ssax: u32, ssay: u32, dsax: u32, dsay: u32, dir: u32) -> u64 {
    u64::from(ssax) | u64::from(ssay) << 16 | u64::from(dsax) << 32 | u64::from(dsay) << 48 | u64::from(dir) << 59
}

pub fn trxreg(w: u32, h: u32) -> u64 {
    u64::from(w) | u64::from(h) << 32
}

/// The qwords `w` × `h` pixels of `psm` take in transfer data, rounded up.
pub fn qwords(psm: u32, w: u32, h: u32) -> u32 {
    (w * h * manual::bits(psm)).div_ceil(128)
}

/// Host to local: the registers, then the image.
pub fn upload(ops: &mut Vec<Op>, psm: u32, (bp, bw): (u32, u32), (x, y, w, h): (u32, u32, u32, u32), seed: u32) {
    ops.push(Op::Regs(vec![(r::BITBLTBUF, bitbltbuf(0, 0, 0, bp, bw, psm)), (r::TRXPOS, trxpos(0, 0, x, y, 0)), (r::TRXREG, trxreg(w, h)), (r::TRXDIR, 0)]));
    ops.push(Op::Image { seed, qwords: qwords(psm, w, h) });
}

/// Local to host, after the FINISH handshake.
pub fn download(ops: &mut Vec<Op>, psm: u32, (bp, bw): (u32, u32), (x, y, w, h): (u32, u32, u32, u32)) {
    let q = qwords(psm, w, h);
    assert!(q.is_multiple_of(8), "a DMA download is whole 128-byte units (GS User's Manual §4.2.2)");
    ops.push(Op::Finish);
    ops.push(Op::Regs(vec![(r::BITBLTBUF, bitbltbuf(bp, bw, psm, 0, 0, 0)), (r::TRXPOS, trxpos(x, y, 0, 0, 0)), (r::TRXREG, trxreg(w, h)), (r::TRXDIR, 1)]));
    ops.push(Op::Readback(q));
}

/// Local to local.
pub fn copy(ops: &mut Vec<Op>, src: (u32, u32, u32), dst: (u32, u32, u32), (sx, sy, dx, dy): (u32, u32, u32, u32), (w, h): (u32, u32), dir: u32) {
    ops.push(Op::Regs(vec![
        (r::BITBLTBUF, bitbltbuf(src.0, src.1, src.2, dst.0, dst.1, dst.2)),
        (r::TRXPOS, trxpos(sx, sy, dx, dy, dir)),
        (r::TRXREG, trxreg(w, h)),
        (r::TRXDIR, 2),
    ]));
}

fn page_of(psm: u32) -> (u32, u32) {
    manual::geometry(psm).page
}

/// The formats whose pixels leave part of a word, so the page is filled first.
fn partial(psm: u32) -> bool {
    matches!(psm, manual::CT24 | manual::Z24 | manual::T8H | manual::T4HL | manual::T4HH)
}

/// One page-rectangle of `psm` written from block 1 (§8.5's "pointing to block 1"), read back raw as PSMCT32 over the
/// same 32 blocks, and read back in its own format where §4.1.1 allows it (not the 4-bit formats).
fn format_case(ops: &mut Vec<Op>, psm: u32, seed: u32) {
    let (pw, ph) = page_of(psm);
    if partial(psm) {
        upload(ops, manual::CT32, (1, 1), (0, 0, 64, 32), seed ^ 0x5A5A_0000);
    }
    upload(ops, psm, (1, pw / 64), (0, 0, pw, ph), seed);
    download(ops, manual::CT32, (1, 1), (0, 0, 64, 32));
    if !matches!(psm, manual::T4 | manual::T4HL | manual::T4HH) {
        download(ops, psm, (1, pw / 64), (0, 0, pw, ph));
    }
}

pub fn formats(name: &'static str, list: &[u32]) -> Script {
    let mut ops = Vec::new();
    if list.contains(&manual::Z32) {
        // The depth formats first at a page's start, where a base inside a page cannot matter (D-105).
        for psm in [manual::Z32, manual::Z16] {
            let (pw, ph) = page_of(psm);
            upload(&mut ops, psm, (0x40, pw / 64), (0, 0, pw, ph), 0x2468_ACE0 ^ psm);
            download(&mut ops, manual::CT32, (0x40, 1), (0, 0, 64, 32));
        }
    }
    for (i, &psm) in list.iter().enumerate() {
        format_case(&mut ops, psm, 0x1357_9BDF ^ (psm << 8) ^ i as u32);
    }
    Script { name, ops }
}

/// `s16-places`: TRXPOS's offsets in both directions, the wrap at 2048, pages across a wide buffer, a PSMT8 buffer one
/// unit wide (D-104), local to local in one format and across formats of one width, an overlapping copy, a transfer
/// restarted and one deactivated, and image data outside a transfer.
pub fn places() -> Script {
    use manual::*;
    let mut o = Vec::new();
    let ops = &mut o;
    // Offsets: written at (24, 12), read raw around it from (16, 8), and read back at its own place.
    upload(ops, CT32, (0, 2), (16, 8, 32, 24), 0x0BAD_F00D);
    upload(ops, CT32, (0, 2), (24, 12, 16, 8), 0x1111_2222);
    download(ops, CT32, (0, 2), (16, 8, 32, 24));
    download(ops, CT32, (0, 2), (24, 12, 16, 8));
    // The wrap: 16×4 from (2040, 2046) lands partly at (0..8, 0..2).
    upload(ops, CT32, (0x40, 1), (0, 0, 16, 8), 0x2222_3333);
    upload(ops, CT32, (0x40, 1), (2040, 2046, 16, 4), 0x3333_4444);
    download(ops, CT32, (0x40, 1), (2040, 2046, 16, 4));
    download(ops, CT32, (0x40, 1), (0, 0, 16, 8));
    // Two pages across: x 48..80 of a buffer two units wide, the right part in the second page.
    upload(ops, CT32, (0x80, 1), (0, 0, 16, 8), 0x4444_5555);
    upload(ops, CT32, (0x60, 2), (48, 0, 32, 8), 0x5555_6666);
    download(ops, CT32, (0x80, 1), (0, 0, 16, 8));
    download(ops, CT32, (0x60, 1), (48, 0, 16, 8));
    // PSMT8 one unit wide (D-104): 16×128 from page 0x10; rows 64 and on fall in page 0x10 again or in page 0x11.
    upload(ops, CT32, (0x200, 1), (0, 0, 8, 8), 0x6666_7777);
    upload(ops, CT32, (0x220, 1), (0, 0, 8, 8), 0x7777_8888);
    upload(ops, T8, (0x200, 1), (0, 0, 16, 128), 0x8888_9999);
    download(ops, CT32, (0x200, 1), (0, 0, 16, 8));
    download(ops, CT32, (0x220, 1), (0, 0, 16, 8));
    // Local to local in PSMCT32, then PSMCT16 to PSMZ16 and PSMT8H to PSMT8 (one width each, §4.2.3).
    copy(ops, (0, 2, CT32), (0, 2, CT32), (24, 12, 8, 40), (16, 8), 0);
    download(ops, CT32, (0, 2), (8, 40, 16, 8));
    upload(ops, CT16, (0x300, 1), (0, 0, 32, 8), 0x9999_AAAA);
    upload(ops, CT32, (0x358, 1), (0, 0, 8, 8), 0xAAAA_BBBB);
    copy(ops, (0x300, 1, CT16), (0x340, 1, Z16), (0, 0, 0, 0), (32, 8), 0);
    download(ops, Z16, (0x340, 1), (0, 0, 32, 8));
    download(ops, CT32, (0x358, 1), (0, 0, 16, 8));
    upload(ops, CT32, (0x380, 1), (0, 0, 16, 8), 0xBBBB_CCCC);
    upload(ops, CT32, (0x3A0, 1), (0, 0, 16, 8), 0xCCCC_DDDD);
    copy(ops, (0x380, 1, T8H), (0x3A0, 2, T8), (0, 0, 0, 0), (16, 8), 0);
    download(ops, CT32, (0x3A0, 1), (0, 0, 16, 8));
    // An overlapping copy in order 0, one pixel to the right (S19's question, recorded here).
    upload(ops, CT32, (0x3C0, 1), (0, 0, 16, 8), 0xDDDD_EEEE);
    copy(ops, (0x3C0, 1, CT32), (0x3C0, 1, CT32), (0, 0, 1, 0), (8, 2), 0);
    download(ops, CT32, (0x3C0, 1), (0, 0, 16, 8));
    // TRXDIR again before the end (§4.2.1), the rest of the data to the new transfer; TRXDIR 3 ends one; data beyond.
    upload(ops, CT32, (0x400, 1), (0, 0, 16, 16), 0xEEEE_FFFF);
    ops.push(Op::Regs(vec![(r::BITBLTBUF, bitbltbuf(0, 0, 0, 0x400, 1, CT32)), (r::TRXPOS, trxpos(0, 0, 0, 0, 0)), (r::TRXREG, trxreg(16, 2)), (r::TRXDIR, 0)]));
    ops.push(Op::Image { seed: 0x0101_0101, qwords: 4 });
    ops.push(Op::Regs(vec![(r::TRXPOS, trxpos(0, 0, 0, 8, 0)), (r::TRXDIR, 0)]));
    ops.push(Op::Image { seed: 0x0202_0202, qwords: 24 });
    ops.push(Op::Regs(vec![(r::TRXPOS, trxpos(0, 0, 0, 12, 0)), (r::TRXDIR, 0)]));
    ops.push(Op::Image { seed: 0x0303_0303, qwords: 2 });
    ops.push(Op::Regs(vec![(r::TRXDIR, 3)]));
    ops.push(Op::Image { seed: 0x0404_0404, qwords: 8 });
    download(ops, CT32, (0x400, 1), (0, 0, 16, 16));
    Script { name: "s16-places", ops: o }
}

/// `s16-regs`: CSR and HSINT and VSINT first, before anything else is written; the privileged registers' reads after
/// writes, their images, narrower accesses; LABEL and SIGNAL with their masks, the stall of a second SIGNAL, IMR and the
/// INTC's GS bit; and CSR's reset. A record of a privileged register is graded by the manual's model where `true`.
pub fn regs() -> Script {
    use ga::*;
    let mut o = Vec::new();
    let ops = &mut o;
    let all = u64::MAX;
    // CSR's events, FIFO, REV and ID; the PCRTC's fields (bits 12 and 13) are S20's and never graded here.
    let csr_low = 0xC01F;
    ops.push(Op::Priv(CSR, 0x1F));
    ops.push(Op::Priv(IMR, 0x7F00));
    ops.push(Op::ClearIntc);
    ops.push(Op::Record(CSR, 0x3, true));
    ops.push(Op::Record(CSR, 0xC000, true));
    ops.push(Op::Record(CSR, 0xFFFF_0000, false));
    ops.push(Op::Record(CSR, 0xFFFF_FFFF_0000_0000, false));
    // VSINT and HSINT: cleared, then set by the next V-sync and H-sync; VSINT unmasked reaches the INTC.
    ops.push(Op::Priv(CSR, 0x8));
    ops.push(Op::Wait(CSR, 0x8));
    ops.push(Op::Record(CSR, 0x8, true));
    ops.push(Op::Priv(CSR, 0x4));
    ops.push(Op::Wait(CSR, 0x4));
    ops.push(Op::Record(CSR, 0x4, true));
    ops.push(Op::Intc);
    ops.push(Op::Priv(CSR, 0x8));
    ops.push(Op::Priv(IMR, 0x7700));
    ops.push(Op::Wait(CSR, 0x8));
    ops.push(Op::Priv(IMR, 0x7F00));
    ops.push(Op::Intc);
    ops.push(Op::Priv(CSR, 0x1F));
    // The PCRTC's timing registers read only (never written here, their values unknown and the picture's); the
    // output registers written, read and left at 0; IMR and BUSDIR (bit 0 clear) likewise.
    for g in [SMODE1, SMODE2, SRFSH, SYNCH1, SYNCH2, SYNCV] {
        ops.push(Op::Record(g, csr_low, true));
    }
    for g in [PMODE, DISPFB1, DISPLAY1, DISPFB2, DISPLAY2, EXTBUF, EXTDATA, BGCOLOR] {
        ops.push(Op::Probe(g, 0x0123_4567_89AB_CDEF));
    }
    ops.push(Op::Priv(IMR, all));
    ops.push(Op::Record(IMR, csr_low, true));
    ops.push(Op::Priv(IMR, 0x7F00));
    ops.push(Op::Priv(BUSDIR, 0xFFFF_FFFF_FFFF_FFFE));
    ops.push(Op::Record(BUSDIR, csr_low, true));
    ops.push(Op::Priv(BUSDIR, 0));
    // Reserved GS addresses.
    for g in [0x0F, 0x3F, 0x42, 0x43, 0x45, 0x49, 0x7F] {
        ops.push(Op::Record(g, csr_low, true));
    }
    // SIGLBLID and its images (EE User's Manual §2.2.3: bits 10 and 11 not decoded; bits 13 to 15 by the same
    // reading, D-103), and stores narrower than SD.
    ops.push(Op::Priv(SIGLBLID, 0x89AB_CDEF_0123_4567));
    ops.push(Op::Record(SIGLBLID, all, true));
    for a in [0x1200_1480, 0x1200_1880, 0x1200_1C80, 0x1200_3080, 0x1200_F080] {
        ops.push(Op::RecordAt(a, 8, all, true));
    }
    ops.push(Op::PrivAt(0x1200_1880, 8, 0x1111_2222_3333_4444));
    ops.push(Op::Record(SIGLBLID, all, true));
    ops.push(Op::PrivAt(0x1200_1080, 4, 0x5555_6666));
    ops.push(Op::Record(SIGLBLID, all, true));
    ops.push(Op::PrivAt(0x1200_1084, 4, 0x7777_8888));
    ops.push(Op::Record(SIGLBLID, all, true));
    ops.push(Op::RecordAt(0x1200_1084, 4, 0xFFFF_FFFF, true));
    ops.push(Op::RecordAt(0x1200_1400, 4, 0xC000, true));
    // LABEL: only the bits IDMSK sets (§7.1); the console texts' own case (gs/label) is checked in the tests.
    ops.push(Op::Priv(SIGLBLID, 0));
    ops.push(Op::Regs(vec![(r::LABEL, 0x0000_00F0_FFFF_FFFF)]));
    ops.push(Op::Record(SIGLBLID, all, true));
    ops.push(Op::Regs(vec![(r::LABEL, 0xFFFF_0000_1234_5678)]));
    ops.push(Op::Record(SIGLBLID, all, true));
    ops.push(Op::Record(CSR, 0x3, true));
    ops.push(Op::Intc);
    // SIGNAL, masked: the flag rises, SIGID takes its masked bits, the INTC sees nothing (§6.2.1).
    ops.push(Op::Regs(vec![(r::SIGNAL, 0xFFFF_0000_ABCD_1234)]));
    ops.push(Op::Record(CSR, 0x3, true));
    ops.push(Op::Record(SIGLBLID, all, true));
    ops.push(Op::Intc);
    // Unmasked with the flag set: the line rises now.
    ops.push(Op::Priv(IMR, 0x7E00));
    ops.push(Op::Intc);
    // A second SIGNAL while the flag is set stalls, with a LABEL and a FINISH behind it (§6.2.1).
    ops.push(Op::Regs(vec![(r::SIGNAL, 0x0000_FFFF_0000_5678), (r::LABEL, 0xFFFF_FFFF_0000_0001), (r::FINISH, 0)]));
    ops.push(Op::Record(SIGLBLID, all, true));
    ops.push(Op::Record(CSR, 0x3, true));
    ops.push(Op::Intc);
    // Clearing the flag takes the second SIGNAL at once: the flag is set again with no edge, then the rest proceeds.
    ops.push(Op::Priv(CSR, 1));
    ops.push(Op::Record(SIGLBLID, all, true));
    ops.push(Op::Record(CSR, 0x3, true));
    ops.push(Op::Intc);
    // "...setting the SIGMSK flag of the IMR register to 1 and then to 0": the second SIGNAL's interrupt.
    ops.push(Op::Priv(IMR, 0x7F00));
    ops.push(Op::Priv(IMR, 0x7E00));
    ops.push(Op::Intc);
    // FINISH unmasked while SIGNAL holds the line: no new edge; SIGNAL cleared, FINISH still holds it; then FINISH.
    ops.push(Op::Priv(IMR, 0x7C00));
    ops.push(Op::Priv(CSR, 2));
    ops.push(Op::Intc);
    ops.push(Op::Regs(vec![(r::FINISH, 0)]));
    ops.push(Op::Intc);
    ops.push(Op::Priv(CSR, 1));
    ops.push(Op::Intc);
    ops.push(Op::Priv(CSR, 2));
    ops.push(Op::Intc);
    ops.push(Op::Regs(vec![(r::FINISH, 0)]));
    ops.push(Op::Record(CSR, 0x3, true));
    ops.push(Op::Intc);
    ops.push(Op::Priv(CSR, 3));
    ops.push(Op::Priv(IMR, 0x7F00));
    ops.push(Op::Intc);
    // CSR's reset: the events cleared and IMR masked again, so an unmasked SIGNAL before it reaches no INTC after.
    ops.push(Op::Priv(IMR, 0x7E00));
    ops.push(Op::Regs(vec![(r::FINISH, 0)]));
    ops.push(Op::Priv(CSR, 0x200));
    ops.push(Op::Record(CSR, 0x3, true));
    ops.push(Op::Intc);
    ops.push(Op::Regs(vec![(r::SIGNAL, 0)]));
    ops.push(Op::Record(CSR, 0x1, true));
    ops.push(Op::Intc);
    ops.push(Op::Record(SIGLBLID, all, false));
    ops.push(Op::Priv(CSR, 0x1F));
    Script { name: "s16-regs", ops: o }
}

pub fn all() -> Vec<Script> {
    use manual::*;
    vec![
        formats("s16-formats-c", &[CT32, CT24, CT16, CT16S]),
        formats("s16-formats-z", &[Z32, Z24, Z16, Z16S]),
        formats("s16-formats-t", &[T8, T8H, T4, T4HL, T4HH]),
        places(),
        regs(),
    ]
}

/// The script run on the manual's model: the expected record, 64 bits an entry.
pub fn expected(script: &Script) -> Vec<Option<u64>> {
    let mut gs = manual::Gs::default();
    let mut out = Vec::new();
    for op in &script.ops {
        match op {
            Op::Regs(list) => list.iter().for_each(|&(reg, v)| gs.write(reg, v)),
            Op::Image { seed, qwords } => units(*seed, *qwords).into_iter().for_each(|u| gs.write(r::HWREG, u)),
            Op::Priv(g, v) => gs.write_privileged(*g, *v),
            Op::PrivAt(a, bytes, v) => {
                let g = (a >> 12 & 1) << 6 | (a >> 4 & 0x3F);
                let shift = 8 * (a & 7);
                let m = if *bytes == 8 { u64::MAX } else { ((1u64 << (8 * bytes)) - 1) << shift };
                let old = if g == ga::SIGLBLID { gs.siglblid } else { gs.privileged.get(&g).copied().unwrap_or(0) };
                gs.write_privileged(g, (old & !m) | ((v << shift) & m));
            }
            Op::Finish => {
                gs.write_privileged(ga::CSR, 2);
                gs.write(r::FINISH, 0);
                gs.write_privileged(ga::CSR, 2);
            }
            Op::Readback(q) => {
                gs.write_privileged(ga::BUSDIR, 1);
                out.extend((0..2 * q).map(|_| Some(gs.read())));
                gs.write_privileged(ga::BUSDIR, 0);
            }
            Op::Record(g, mask, graded) => out.push(graded.then(|| gs.read_privileged(*g) & mask)),
            Op::RecordAt(a, bytes, mask, graded) => {
                let g = (a >> 12 & 1) << 6 | (a >> 4 & 0x3F);
                let v = gs.read_privileged(g) >> (8 * (a & 7));
                let v = if *bytes == 8 { v } else { v & ((1 << (8 * bytes)) - 1) };
                out.push(graded.then_some(v & mask));
            }
            Op::Intc => out.push(Some(u64::from(std::mem::take(&mut gs.intc)))),
            Op::ClearIntc => gs.intc = false,
            Op::Probe(g, pattern) => {
                out.push(None);
                gs.write_privileged(*g, *pattern);
                out.push(Some(gs.read_privileged(*g) & 0xC01F));
                gs.write_privileged(*g, 0);
            }
            Op::Wait(_, m) => gs.event(*m as u32),
        }
    }
    out
}

/// The DMA buffer the packets are built in, physical.
const BUF: u32 = 0x0030_0000;
const D1_CHCR: u32 = 0x1000_9000;
const D1_MADR: u32 = 0x1000_9010;
const D1_QWC: u32 = 0x1000_9020;
const D2_CHCR: u32 = 0x1000_A000;
const D2_MADR: u32 = 0x1000_A010;
const D2_QWC: u32 = 0x1000_A020;
const D_CTRL: u32 = 0x1000_E000;
const GIF_CTRL: u32 = 0x1000_3000;
const GIF_MODE: u32 = 0x1000_3010;
const VIF1_STAT: u32 = 0x1000_3C00;
const VIF1_FBRST: u32 = 0x1000_3C10;
const I_STAT: u32 = 0x1000_F000;

fn sd(p: &mut Program, pa: u32, v: u64) {
    li(p, "t6", K1 | pa);
    p.constant("t7", v);
    p.emit("sd t7, 0(t6)");
}

/// Sends the packet of `qwords` at BUF through the GIF's channel in normal mode and waits for it.
fn send(p: &mut Program, qwords: u32) {
    poke(p, D2_MADR, BUF);
    poke(p, D2_QWC, qwords);
    poke(p, D2_CHCR, 0x101);
    wait(p, D2_CHCR, 0x100, false, 200_000);
}

fn store_words(p: &mut Program, at: u32, words: &[u32]) {
    li(p, "t6", K1 | at);
    for (i, w) in words.iter().enumerate() {
        li(p, "t7", *w);
        p.emit(format!("sw t7, {}(t6)", 4 * i));
    }
}

/// The script as an EE program, with its expected record (64 bits an entry).
pub fn program(script: &Script) -> Program {
    let mut prog = Program::kernel(script.name);
    let p = &mut prog;
    p.emit("di");
    p.emit("mtc0 zero, Status");
    p.emit("sync.p");
    poke(p, D_CTRL, 0);
    poke(p, D1_CHCR, 0);
    poke(p, D2_CHCR, 0);
    poke(p, D_CTRL, 1);
    poke(p, GIF_CTRL, 1);
    poke(p, GIF_MODE, 0);
    poke(p, VIF1_FBRST, 1);
    poke(p, VIF1_STAT, 0);
    let want = expected(script);
    let mut at = 0;
    for op in &script.ops {
        match op {
            Op::Regs(list) => {
                let mut words = vec![list.len() as u32 | 0x8000, 0x1000_0000, 0xE, 0];
                for &(reg, v) in list {
                    words.extend([v as u32, (v >> 32) as u32, u32::from(reg), 0]);
                }
                store_words(p, BUF, &words);
                send(p, list.len() as u32 + 1);
            }
            Op::Image { seed, qwords } => {
                store_words(p, BUF, &[qwords | 0x8000, 0x0800_0000, 0, 0]);
                let l = p.label();
                li(p, "t0", *seed);
                li(p, "t1", 0x41C6_4E6D);
                li(p, "t2", K1 | (BUF + 16));
                li(p, "t3", qwords * 4);
                p.place(&l);
                p.emit("multu t0, t1");
                p.emit("mflo t0");
                p.emit("addiu t0, t0, 0x3039");
                p.emit("sw t0, 0(t2)");
                p.emit("addiu t2, t2, 4");
                p.emit("addiu t3, t3, -1");
                p.emit(format!("bne t3, zero, {l}"));
                p.emit("nop");
                send(p, qwords + 1);
            }
            Op::Priv(g, v) => sd(p, ee_address(*g), *v),
            Op::PrivAt(a, bytes, v) => {
                li(p, "t6", K1 | a);
                p.constant("t7", *v);
                p.emit(match bytes {
                    8 => "sd t7, 0(t6)",
                    4 => "sw t7, 0(t6)",
                    _ => panic!("a store of {bytes} bytes"),
                });
            }
            Op::Finish => {
                sd(p, ee_address(ga::CSR), 2);
                store_words(p, BUF, &[1 | 0x8000, 0x1000_0000, 0xE, 0, 0, 0, u32::from(r::FINISH), 0]);
                send(p, 2);
                wait(p, ee_address(ga::CSR), 2, true, 200_000);
                sd(p, ee_address(ga::CSR), 2);
            }
            Op::Readback(q) => {
                let target = RECORD + 8 * p.expected.len() as u32;
                sd(p, ee_address(ga::BUSDIR), 1);
                poke(p, VIF1_STAT, 0x0080_0000);
                poke(p, D1_MADR, target);
                poke(p, D1_QWC, *q);
                poke(p, D1_CHCR, 0x100);
                wait(p, D1_CHCR, 0x100, false, 400_000);
                poke(p, VIF1_STAT, 0);
                sd(p, ee_address(ga::BUSDIR), 0);
                p.expected.extend_from_slice(&want[at..at + 2 * *q as usize]);
                at += 2 * *q as usize;
                li(p, "t7", 16 * q);
                p.emit("daddu s1, s1, t7");
            }
            Op::Record(g, mask, _) | Op::RecordAt(g, _, mask, _) => {
                let (addr, bytes) = match op {
                    Op::RecordAt(a, b, _, _) => (*a, *b),
                    _ => (ee_address(*g), 8),
                };
                li(p, "t6", K1 | addr);
                p.emit(if bytes == 8 { "ld t2, 0(t6)" } else { "lwu t2, 0(t6)" });
                if *mask != u64::MAX {
                    p.constant("t7", *mask);
                    p.emit("and t2, t2, t7");
                }
                p.record("t2", want[at]);
                at += 1;
            }
            Op::ClearIntc => poke(p, I_STAT, 1),
            Op::Probe(g, pattern) => {
                li(p, "t6", K1 | ee_address(*g));
                p.emit("ld t3, 0(t6)");
                p.record("t3", want[at]);
                p.constant("t7", *pattern);
                li(p, "t6", K1 | ee_address(*g));
                p.emit("sd t7, 0(t6)");
                p.emit("ld t2, 0(t6)");
                p.constant("t7", 0xC01F);
                p.emit("and t2, t2, t7");
                p.record("t2", want[at + 1]);
                p.emit("sd zero, 0(t6)");
                at += 2;
            }
            Op::Intc => {
                peek(p, I_STAT);
                p.emit("andi t2, t2, 1");
                p.record("t2", want[at]);
                at += 1;
                poke(p, I_STAT, 1);
            }
            Op::Wait(g, mask) => {
                let l = p.label();
                let out = p.label();
                li(p, "t6", K1 | ee_address(*g));
                li(p, "t5", 4_000_000);
                p.place(&l);
                p.emit("ld t2, 0(t6)");
                p.emit(format!("andi t2, t2, 0x{mask:x}"));
                p.emit(format!("bne t2, zero, {out}"));
                p.emit("nop");
                p.emit("addiu t5, t5, -1");
                p.emit(format!("bne t5, zero, {l}"));
                p.emit("nop");
                p.place(&out);
            }
        }
    }
    assert_eq!(at, want.len(), "{}: every expected entry placed", script.name);
    prog
}
