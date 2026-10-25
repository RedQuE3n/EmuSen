//! The project's S19 programs (Sapphire_Native.md §37.6): the pixel operations after texturing, each a script in S16's
//! form. Sources are DECAL sprites over a texture of generated data, so that every pixel brings its own colour and
//! alpha (texel addressing being exact on both lineages, S18), over frame and Z buffers filled with generated data; every
//! page drawn is read back raw as PSMCT32. The expected record is S17's model's, whose `pixel_ops` (written apart from
//! `src/gs/`) carries §3.6 to §3.9.

#![allow(dead_code)]

use crate::s16::manual::{CT16, CT16S, CT24, CT32, T8, Z32};
use crate::s16::{self, Op, Script, upload};
use crate::s17::{self, ALWAYS, Buf, GEQUAL, Reg, SPRITE, frame, prim, r, scissor, test, xyz2, zbuf};
use crate::s18::{self, DECAL, FST, TME, Tex, fill, rgbaq};

pub const ABE: u64 = 1 << 6;
pub const AA1: u64 = 1 << 7;
pub const CONTEXT_2: u64 = 1 << 9;
pub const DIMX: u8 = 0x44;

pub const KEEP: u64 = 0;
pub const FB_ONLY: u64 = 1;
pub const ZB_ONLY: u64 = 2;
pub const RGB_ONLY: u64 = 3;

/// The texture every program's sources read: one page of generated data at page 300, PSMCT32, DECAL with TCC RGBA.
pub const TEXTURE: u32 = 300;

/// §3.9.1's example dither matrix, rows Y % 4 and columns X % 4.
pub const EXAMPLE: [[i32; 4]; 4] = [[-4, 2, -3, 3], [0, -2, 1, -1], [-3, 3, -4, 2], [1, -1, 0, -2]];

/// TEST from its fields (TEST's page).
#[allow(clippy::too_many_arguments)]
pub fn test_reg(ate: bool, atst: u64, aref: u32, afail: u64, date: bool, datm: bool, zte: bool, ztst: u64) -> u64 {
    u64::from(ate) | atst << 1 | u64::from(aref) << 4 | afail << 12 | u64::from(date) << 14 | u64::from(datm) << 15 | u64::from(zte) << 16 | ztst << 17
}

/// An alpha test with the depth test ALWAYS.
pub fn atest(atst: u64, aref: u32, afail: u64) -> u64 {
    test_reg(true, atst, aref, afail, false, false, true, ALWAYS)
}

/// ALPHA from A, B, C, D and FIX (ALPHA's page: FIX at bits 39:32).
pub fn alpha(a: u64, b: u64, c: u64, d: u64, fix: u32) -> u64 {
    a | b << 2 | c << 4 | d << 6 | u64::from(fix) << 32
}

/// DIMX from a matrix of values -4 to 3, DMyx at bits 16y + 4x.
pub fn dimx(m: [[i32; 4]; 4]) -> u64 {
    let mut v = 0;
    for (y, row) in m.iter().enumerate() {
        for (x, &e) in row.iter().enumerate() {
            v |= u64::from(e as u32 & 7) << (16 * y + 4 * x);
        }
    }
    v
}

/// S18's start (the texture registers) and S17's, then the registers S19 adds: DIMX, and the texture all sources read.
pub fn begin(ops: &mut Vec<Op>) {
    s18::begin(ops);
    let t = Tex::new(TEXTURE, 1, CT32, 6, 5).func(DECAL, true);
    ops.push(Op::Regs(vec![(DIMX, 0), (s18::r::TEX0_1, t.bits()), (s18::r::TEX0_2, t.bits())]));
    fill(ops, TEXTURE, 1, 0x1919_0001);
}

/// A frame page (or two, for a 16-bit format's 64×64) at `page`, filled, context 1's target with an optional Z buffer.
pub fn frame_page(ops: &mut Vec<Op>, page: u32, psm: u32, z: Option<u32>, seed: u32) -> (Buf, Option<Buf>) {
    let h = if matches!(psm, CT16 | CT16S) { 64 } else { 32 };
    let f = Buf::new(page, 1, psm, 64, h);
    f.fill(ops, seed);
    let zb = z.map(|zp| {
        let b = Buf::new(zp, 1, Z32, 64, 32);
        b.fill(ops, seed ^ 0x00FF_0000);
        b
    });
    s17::target(ops, &f, zb.as_ref().map(|b| (b, false)), test(true, ALWAYS));
    (f, zb)
}

pub fn read(ops: &mut Vec<Op>, f: &Buf, z: Option<&Buf>) {
    f.read(ops);
    if let Some(z) = z {
        z.read(ops);
    }
}

/// A DECAL sprite at pixels (x, y) of size w × h reading the texture 1:1 from texel (u, v), at Z `z`, through `reg`.
pub fn cell(regs: &mut Vec<Reg>, (x, y, w, h): (i32, i32, i32, i32), (u, v): (u32, u32), attributes: u64, z: u32) {
    regs.push(prim(SPRITE, TME | FST | attributes));
    regs.extend([rgbaq([0x80; 4], 1.0), s18::uv(16 * u + 8, 16 * v + 8), xyz2(16 * x, 16 * y, z), s18::uv(16 * (u + w as u32) + 8, 16 * (v + h as u32) + 8), xyz2(16 * (x + w), 16 * (y + h), z)]);
}

/// A flat sprite of colour `c` at pixels (x, y) of size w × h.
pub fn flat(regs: &mut Vec<Reg>, (x, y, w, h): (i32, i32, i32, i32), c: [u32; 4], attributes: u64, z: u32) {
    regs.push(prim(SPRITE, attributes));
    regs.extend([rgbaq(c, 1.0), xyz2(16 * x, 16 * y, z), xyz2(16 * (x + w), 16 * (y + h), z)]);
}

/// Texels for cell `k`, spread over the texture.
pub fn texel(k: u32) -> (u32, u32) {
    ((k * 13) % 56, (k * 7) % 24)
}

/// `s19-atest`: every ATST under every AFAIL (§3.7.2), a cell of 8×8 each: six rows of texels' alphas and four flat
/// pixels of 2×2 at AREF - 1, AREF, AREF + 1 and 80h, each primitive at Z 13579BDFh over a PSMZ32 buffer of generated
/// data with the depth test ALWAYS and Z written, so that FB_ONLY, ZB_ONLY and RGB_ONLY show in both buffers. First with
/// AREF 80h in PSMCT32; then AREF 00h, FFh, 3Ch and C3h by rows with AFAIL turning; then AREF 80h in PSMCT16 and PSMCT24,
/// where RGB_ONLY acts as FB_ONLY.
pub fn atests() -> Script {
    let mut o = Vec::new();
    let ops = &mut o;
    begin(ops);
    let sets: [(u32, u32, bool); 4] = [(10, CT32, false), (12, CT32, true), (14, CT16, false), (17, CT24, false)];
    for (n, &(page, psm, turning)) in sets.iter().enumerate() {
        let (f, z) = frame_page(ops, page, psm, Some(page + if psm == CT16 { 2 } else { 1 }), 0x1919_0100 + n as u32);
        let mut regs = Vec::new();
        for row in 0..4u64 {
            for atst in 0..8u64 {
                let (aref, afail) = if turning { ([0x00, 0xFF, 0x3C, 0xC3][row as usize], (atst + row) % 4) } else { (0x80, row) };
                let k = (row * 8 + atst) as u32;
                let (x, y) = (8 * atst as i32, 8 * row as i32);
                regs.push((r::TEST_1, atest(atst, aref, afail)));
                cell(&mut regs, (x, y, 8, 6), texel(k + 32 * n as u32), 0, 0x1357_9BDF);
                for (i, a) in [aref.wrapping_sub(1) & 0xFF, aref, (aref + 1) & 0xFF, 0x80].into_iter().enumerate() {
                    flat(&mut regs, (x + 2 * i as i32, y + 6, 2, 2), [17 * k & 0xFF, 0x5A, 0xA5 ^ k, a], 0, 0x1357_9BDF);
                }
            }
        }
        ops.push(Op::Regs(regs));
        read(ops, &f, z.as_ref());
    }
    Script { name: "s19-atest", ops: o }
}

/// `s19-date`: the destination alpha test (§3.7.3) in PSMCT32, PSMCT16 and PSMCT24 over generated frames, by cells of
/// 16×8: off; DATM 0; DATM 1; each DATM with the alpha test failing into ZB_ONLY, so that a pixel the destination test
/// stops writes no Z; DATM 0 with a GEQUAL depth test; DATM 1 with FB_ONLY; DATE in context 2 alone; DATM 1 over a
/// blend by Ad; and a second sprite over a first that wrote alpha, its test reading the first's alpha.
pub fn dates() -> Script {
    let mut o = Vec::new();
    let ops = &mut o;
    begin(ops);
    for (n, &(page, psm)) in [(20u32, CT32), (22, CT16), (25, CT24)].iter().enumerate() {
        let zp = page + if psm == CT16 { 2 } else { 1 };
        let (f, z) = frame_page(ops, page, psm, Some(zp), 0x1919_0200 + n as u32);
        let d = |datm: bool, atst: u64, afail: u64, ztst: u64| test_reg(true, atst, 0x80, afail, true, datm, true, ztst);
        let cells: [(u64, u64); 12] = [
            (test(true, ALWAYS), 0),
            (d(false, ALWAYS, KEEP, ALWAYS), 0),
            (d(true, ALWAYS, KEEP, ALWAYS), 0),
            (d(false, 0, ZB_ONLY, ALWAYS), 0),
            (d(true, 0, ZB_ONLY, ALWAYS), 0),
            (d(false, ALWAYS, KEEP, GEQUAL), 0),
            (d(true, 0, FB_ONLY, ALWAYS), 0),
            (d(true, ALWAYS, KEEP, ALWAYS), CONTEXT_2),
            (d(true, ALWAYS, KEEP, ALWAYS), ABE),
            (d(false, 0, RGB_ONLY, ALWAYS), 0),
            (test_reg(true, 0, 0x80, ZB_ONLY, false, false, true, ALWAYS), 0),
            (test(true, ALWAYS), 1),
        ];
        let mut regs = vec![
            (r::FRAME_2, frame(f.page, 1, psm, 0)),
            (r::ZBUF_2, zbuf(zp, Z32, false)),
            (r::SCISSOR_2, scissor(0, 63, 0, 31)),
            (r::TEST_1, test(true, ALWAYS)),
            (r::ALPHA_1, alpha(0, 1, 1, 1, 0)),
        ];
        for (k, &(t, attributes)) in cells.iter().enumerate() {
            let (x, y) = (16 * (k as i32 % 4), 8 * (k as i32 / 4));
            if attributes == CONTEXT_2 {
                regs.extend([(r::TEST_2, t), (r::TEST_1, test(true, ALWAYS))]);
            } else {
                regs.push((r::TEST_1, t));
            }
            if attributes == 1 {
                // A sprite writing alpha 0 on the left half and FFh on the right, then DATM 1 over the whole cell.
                flat(&mut regs, (x, y, 8, 8), [1, 2, 3, 0], 0, 0x4000_0000);
                flat(&mut regs, (x + 8, y, 8, 8), [4, 5, 6, 0xFF], 0, 0x4000_0000);
                regs.push((r::TEST_1, d(true, ALWAYS, KEEP, ALWAYS)));
                cell(&mut regs, (x, y, 16, 8), texel(k as u32), 0, 0x6000_0000);
            } else {
                cell(&mut regs, (x, y, 16, 8), texel(k as u32 + 12 * n as u32), attributes & (ABE | CONTEXT_2), 0x8000_0000);
            }
        }
        ops.push(Op::Regs(regs));
        read(ops, &f, z.as_ref());
    }
    Script { name: "s19-date", ops: o }
}

/// The 81 selections of A, B, C and D (§3.8.1, ALPHA's page), selection `k` read as four base-3 digits A, B, C, D, with
/// FIX from a list of eight by `k`.
pub fn selection(k: u32) -> u64 {
    const FIX: [u32; 8] = [0x00, 0x01, 0x40, 0x7F, 0x80, 0x81, 0xC0, 0xFF];
    let d = |i: u32| u64::from(k / 3u32.pow(i) % 3);
    alpha(d(0), d(1), d(2), d(3), FIX[(k % 8) as usize])
}

/// `s19-blend`: every selection of A, B, C and D with FIX (§3.8.1), a cell of 8×8 each, three pages a frame: in PSMCT32
/// with COLCLAMP 1 and with COLCLAMP 0, in PSMCT16 (Cd its five bits, Ad 0 or 80h) and in PSMCT24 (Ad 80h), both with
/// COLCLAMP 1; then the reserved selection 11 of each field, one cell each, in PSMCT32.
pub fn blends() -> Script {
    let mut o = Vec::new();
    let ops = &mut o;
    begin(ops);
    let variants: [(u32, u32, u64); 4] = [(30, CT32, 1), (33, CT32, 0), (36, CT16, 1), (42, CT24, 1)];
    for (n, &(page, psm, clamp)) in variants.iter().enumerate() {
        let tall = psm == CT16;
        for p in 0..3u32 {
            let (f, _) = frame_page(ops, page + p * if tall { 2 } else { 1 }, psm, None, 0x1919_0300 + 8 * n as u32 + p);
            let mut regs = vec![(r::COLCLAMP, clamp)];
            for i in 0..32u32 {
                let k = 32 * p + i;
                let (x, y) = (8 * (i as i32 % 8), 8 * (i as i32 / 8));
                let sel = match k {
                    0..=80 => selection(k),
                    81..=84 => alpha(0, 1, 0, 1, 0x60) | 3 << (2 * (k - 81)),
                    _ => continue,
                };
                if k > 80 && psm != CT32 {
                    continue;
                }
                regs.push((r::ALPHA_1, sel));
                cell(&mut regs, (x, y, 8, 8), texel(k + 96 * n as u32), ABE, 0);
            }
            regs.push((r::COLCLAMP, 1));
            ops.push(Op::Regs(regs));
            f.read(ops);
        }
    }
    Script { name: "s19-blend", ops: o }
}

/// `s19-pabe`: PABE (§3.8.2) in PSMCT32 and PSMCT16, cells of 16×8: ABE 0 and 1 with PABE 0 and 1, under the normal
/// blend (Cs, Cd, As, Cd) and an additive one (Cs, 0, FIX 80h, Cd); PABE with FBA; flat alphas 7Fh and 80h.
pub fn pabes() -> Script {
    let mut o = Vec::new();
    let ops = &mut o;
    begin(ops);
    for (n, &(page, psm)) in [(50u32, CT32), (52, CT16)].iter().enumerate() {
        let (f, _) = frame_page(ops, page, psm, None, 0x1919_0400 + n as u32);
        let mut regs = Vec::new();
        let mut k = 0;
        for &al in &[alpha(0, 1, 0, 1, 0), alpha(0, 2, 2, 1, 0x80)] {
            for &(abe, pabe, fba) in &[(false, false, 0), (false, true, 0), (true, false, 0), (true, true, 0), (true, true, 1), (false, true, 1)] {
                let (x, y) = (16 * (k % 4), 8 * (k / 4));
                regs.extend([(r::ALPHA_1, al), (r::PABE, u64::from(pabe)), (r::FBA_1, fba)]);
                let at = if abe { ABE } else { 0 };
                cell(&mut regs, (x, y, 16, 6), texel(k as u32 + 16 * n as u32), at, 0);
                flat(&mut regs, (x, y + 6, 8, 2), [0xF0, 0x0F, 0x77, 0x7F], at, 0);
                flat(&mut regs, (x + 8, y + 6, 8, 2), [0xF0, 0x0F, 0x77, 0x80], at, 0);
                k += 1;
            }
        }
        regs.extend([(r::PABE, 0), (r::FBA_1, 0)]);
        ops.push(Op::Regs(regs));
        f.read(ops);
    }
    Script { name: "s19-pabe", ops: o }
}

/// `s19-fba`: FBA (§3.9.3, FBA's page) in PSMCT32 and PSMCT16, cells of 16×8: FBA 0 and 1 in context 1 and context 2,
/// with and without blending, under AFAIL RGB_ONLY (alpha not written whatever FBA says), and with FBMSK keeping the
/// alpha bit.
pub fn fbas() -> Script {
    let mut o = Vec::new();
    let ops = &mut o;
    begin(ops);
    for (n, &(page, psm)) in [(55u32, CT32), (57, CT16)].iter().enumerate() {
        let (f, _) = frame_page(ops, page, psm, None, 0x1919_0500 + n as u32);
        let mut regs = vec![
            (r::FRAME_2, frame(f.page, 1, psm, 0)),
            (r::ZBUF_2, zbuf(0, Z32, true)),
            (r::SCISSOR_2, scissor(0, 63, 0, 31)),
            (r::TEST_2, test(true, ALWAYS)),
            (r::ALPHA_1, alpha(0, 1, 0, 1, 0)),
            (r::ALPHA_2, alpha(1, 0, 2, 0, 0x40)),
        ];
        let cases: [(u64, u64, u64, u64, u64, u32); 12] = [
            (0, 0, 0, 0, 0, 0),
            (1, 0, 0, 0, 0, 0),
            (0, 1, CONTEXT_2, 0, 0, 0),
            (1, 0, CONTEXT_2, 0, 0, 0),
            (0, 1, CONTEXT_2, 1, 0, 0),
            (1, 0, ABE, 0, 0, 0),
            (1, 1, ABE | CONTEXT_2, 1, 0, 0),
            (1, 0, 0, 0, RGB_ONLY, 0),
            (0, 0, 0, 0, RGB_ONLY, 0),
            (1, 0, 0, 0, 0, 0x8000_0000),
            (1, 0, 0, 0, 0, 0x7FFF_FFFF),
            (1, 0, 0, 0, FB_ONLY, 0),
        ];
        for (k, &(fba1, fba2, at, _, afail, mask)) in cases.iter().enumerate() {
            let (x, y) = (16 * (k as i32 % 4), 8 * (k as i32 / 4));
            let t = if afail != 0 { atest(0, 0, afail) } else { test(true, ALWAYS) };
            regs.extend([(r::FBA_1, fba1), (r::FBA_2, fba2), (r::TEST_1, t), (r::TEST_2, t), (r::FRAME_1, frame(f.page, 1, psm, mask))]);
            cell(&mut regs, (x, y, 16, 8), texel(k as u32 + 12 * n as u32), at, 0);
        }
        regs.extend([(r::FBA_1, 0), (r::FBA_2, 0), (r::TEST_1, test(true, ALWAYS)), (r::FRAME_1, frame(f.page, 1, psm, 0))]);
        ops.push(Op::Regs(regs));
        f.read(ops);
    }
    Script { name: "s19-fba", ops: o }
}

/// `s19-dither`: DTHE and DIMX (§3.9.1) in PSMCT16, PSMCT16S, PSMCT32 and PSMCT24, cells of 16×8: §3.9.1's example
/// matrix, every element 3, every element -4, and a generated matrix, each under COLCLAMP 1 and 0, over texels and
/// over flat colours at both ends of the range (00h to 03h and FCh to FFh, where the offset leaves the range); the
/// example matrix after a blend; DTHE 0 with a matrix set.
pub fn dithers() -> Script {
    let mut o = Vec::new();
    let ops = &mut o;
    begin(ops);
    let random = [[2, -1, 3, -4], [-3, 0, 1, 2], [3, 3, -2, -1], [0, -4, 2, 1]];
    let mats = [dimx(EXAMPLE), dimx([[3; 4]; 4]), dimx([[-4; 4]; 4]), dimx(random)];
    for (n, &(page, psm)) in [(60u32, CT16), (62, CT16S), (64, CT32), (65, CT24)].iter().enumerate() {
        let (f, _) = frame_page(ops, page, psm, None, 0x1919_0600 + n as u32);
        let mut regs = vec![(r::DTHE, 1), (r::ALPHA_1, alpha(0, 1, 0, 1, 0))];
        let mut k = 0;
        for (m, &mat) in mats.iter().enumerate() {
            for clamp in [1u64, 0] {
                let (x, y) = (16 * (k % 4), 8 * (k / 4));
                regs.extend([(DIMX, mat), (r::COLCLAMP, clamp)]);
                cell(&mut regs, (x, y, 16, 4), texel(k as u32 + 16 * n as u32), 0, 0);
                for (i, v) in [0u32, 1, 2, 3, 0xFC, 0xFD, 0xFE, 0xFF].into_iter().enumerate() {
                    let c = [v, v ^ (m as u32 & 1), 0xFF - v, 0x40];
                    flat(&mut regs, (x + 2 * i as i32, y + 4, 2, 4), c, 0, 0);
                }
                k += 1;
            }
        }
        regs.extend([(DIMX, dimx(EXAMPLE)), (r::COLCLAMP, 1)]);
        cell(&mut regs, (0, 32 - 8, 16, 8), texel(40), ABE, 0);
        regs.push((r::COLCLAMP, 0));
        cell(&mut regs, (16, 32 - 8, 16, 8), texel(41), ABE, 0);
        regs.extend([(r::DTHE, 0), (r::COLCLAMP, 1)]);
        cell(&mut regs, (32, 32 - 8, 16, 8), texel(42), 0, 0);
        flat(&mut regs, (48, 32 - 8, 16, 8), [0xFF, 0x00, 0x83, 0xFF], 0, 0);
        // The 16-bit frames' lower half: blends past the range, added and subtracted, with the offset after them.
        regs.extend([(r::DTHE, 1), (DIMX, dimx(EXAMPLE))]);
        for (i, (al, clamp)) in [(alpha(0, 2, 2, 1, 0x80), 0u64), (alpha(0, 2, 2, 1, 0x80), 1), (alpha(2, 0, 2, 1, 0x80), 0), (alpha(2, 0, 2, 1, 0x80), 1)].into_iter().enumerate() {
            regs.extend([(r::ALPHA_1, al), (r::COLCLAMP, clamp)]);
            cell(&mut regs, (16 * i as i32, 32, 16, 16), texel(43 + i as u32), ABE, 0);
        }
        regs.extend([(r::DTHE, 0), (r::COLCLAMP, 1)]);
        ops.push(Op::Regs(regs));
        f.read(ops);
    }
    Script { name: "s19-dither", ops: o }
}

/// `s19-masks`: FBMSK with every other operation (§3.9.5 after §3.8 and §3.9.1 to §3.9.3): generated masks over a blend,
/// with RGB_ONLY, with FBA, in PSMCT16 with dithering; ZMSK (ZBUF's page) with ZB_ONLY and with GEQUAL, so that the
/// test still decides and nothing is written.
pub fn masks() -> Script {
    let mut o = Vec::new();
    let ops = &mut o;
    begin(ops);
    let list = [0x0F3C_A5F0u32, 0xFF00_00FF, 0x8000_0000, 0x00FF_FF00, 0x1234_5678, 0xFFFF_FFFF, 0x0000_0000, 0x7F7F_7F7F];
    for (n, &(page, psm)) in [(70u32, CT32), (72, CT16)].iter().enumerate() {
        let zp = page + if psm == CT16 { 2 } else { 1 };
        let (f, z) = frame_page(ops, page, psm, Some(zp), 0x1919_0700 + n as u32);
        let mut regs = vec![(r::ALPHA_1, alpha(0, 1, 0, 1, 0)), (DIMX, dimx(EXAMPLE))];
        for (k, &mask) in list.iter().enumerate() {
            let (x, y) = (16 * (k as i32 % 4), 8 * (k as i32 / 4));
            let (t, at, extra): (u64, u64, Vec<Reg>) = match k % 4 {
                0 => (test(true, ALWAYS), ABE, vec![]),
                1 => (atest(0, 0, RGB_ONLY), 0, vec![]),
                2 => (test(true, ALWAYS), 0, vec![(r::FBA_1, 1)]),
                _ => (test(true, ALWAYS), ABE, vec![(r::DTHE, 1)]),
            };
            regs.extend([(r::TEST_1, t), (r::FRAME_1, frame(f.page, 1, psm, mask))]);
            regs.extend(extra);
            cell(&mut regs, (x, y, 16, 8), texel(k as u32 + 8 * n as u32), at, 0x2468_ACE0);
            regs.extend([(r::FBA_1, 0), (r::DTHE, 0)]);
        }
        regs.push((r::FRAME_1, frame(f.page, 1, psm, 0)));
        // ZMSK: ZB_ONLY with the buffer masked writes nothing; GEQUAL over generated Z decides with nothing written.
        let zcases = [(atest(0, 0, ZB_ONLY), true), (test(true, GEQUAL), true), (atest(0, 0, ZB_ONLY), false), (test(true, GEQUAL), false)];
        for (k, &(t, zmsk)) in zcases.iter().enumerate() {
            regs.extend([(r::TEST_1, t), (r::ZBUF_1, zbuf(zp, Z32, zmsk))]);
            cell(&mut regs, (16 * k as i32, 16, 16, 16), texel(20 + k as u32), 0, 0x8000_0000);
        }
        ops.push(Op::Regs(regs));
        read(ops, &f, z.as_ref());
    }
    Script { name: "s19-masks", ops: o }
}

/// `s19-overlap`: local-to-local copies whose rectangles overlap (§4.1.2's figure 4-1, D-107), each of the four orders
/// at six offsets, a 6×4 rectangle each in a cell of 10×8 of a PSMCT32 buffer 64×64 of generated data; then PSMCT16 at
/// (1, 1) in each order, and PSMT8 one pixel left and right in orders 0 and 2.
pub fn overlaps() -> Script {
    let mut o = Vec::new();
    let ops = &mut o;
    begin(ops);
    let f = Buf::new(80, 1, CT32, 64, 64);
    f.fill(ops, 0x1919_0801);
    let offsets = [(1i32, 0i32), (-1, 0), (0, 1), (0, -1), (2, 1), (-1, -2)];
    for dir in 0..4u32 {
        for (i, &(dx, dy)) in offsets.iter().enumerate() {
            let (cx, cy) = (2 + 10 * i as u32, 2 + 16 * dir);
            s16::copy(ops, (80 * 32, 1, CT32), (80 * 32, 1, CT32), (cx, cy, (cx as i32 + dx) as u32, (cy as i32 + dy) as u32), (6, 4), dir);
        }
    }
    f.read(ops);
    let g = Buf::new(84, 1, CT16, 64, 64);
    g.fill(ops, 0x1919_0802);
    for dir in 0..4u32 {
        let (cx, cy) = (4 + 14 * dir, 4);
        s16::copy(ops, (84 * 32, 1, CT16), (84 * 32, 1, CT16), (cx, cy, cx + 1, cy + 1), (9, 7), dir);
    }
    g.read(ops);
    let h = Buf::new(86, 2, CT32, 128, 32);
    upload(ops, CT32, (86 * 32, 2), (0, 0, 128, 32), 0x1919_0803);
    for (j, &(dir, dx)) in [(0u32, 1i32), (0, -1), (2, 1), (2, -1)].iter().enumerate() {
        let (cx, cy) = (8 + 28 * j as u32, 8);
        s16::copy(ops, (86 * 32, 2, T8), (86 * 32, 2, T8), (cx, cy, (cx as i32 + dx) as u32, cy), (17, 9), dir);
    }
    s16::download(ops, CT32, (86 * 32, 2), (0, 0, 128, 32));
    let _ = h;
    Script { name: "s19-overlap", ops: o }
}

pub fn all() -> Vec<Script> {
    vec![atests(), dates(), blends(), pabes(), fbas(), dithers(), masks(), overlaps(), antialias()]
}

/// `s19-aa`: antialiasing (§3.6): triangles and lines with AA1 under ABE 0 and 1, alpha 80h and 40h, over a PSMCT32
/// frame and a PSMZ32 buffer of generated data with Z written, ALPHA (Cs, Cd, As, Cd) as §3.6.2 asks; the same
/// primitives without AA1 beside them; a sprite and a point with AA1, which §3.2.2 draws without it.
pub fn antialias() -> Script {
    let mut o = Vec::new();
    let ops = &mut o;
    begin(ops);
    let (f, z) = frame_page(ops, 90, CT32, Some(91), 0x1919_0901);
    let mut regs = vec![(r::ALPHA_1, alpha(0, 1, 0, 1, 0))];
    let cases: [(u64, u32); 6] = [(AA1, 0x80), (AA1, 0x40), (AA1 | ABE, 0x80), (AA1 | ABE, 0x40), (0, 0x80), (ABE, 0x40)];
    for (k, &(at, a)) in cases.iter().enumerate() {
        let (x, y) = (16 * 10 * (k as i32 % 3) + 8, 16 * 16 * (k as i32 / 3) + 8);
        regs.extend([prim(s17::TRIANGLE, at), rgbaq([0xF0, 0x80, 0x10, a], 1.0), xyz2(x, y, 0x3000_0000), xyz2(x + 150, y + 37, 0x3000_0000), xyz2(x + 23, y + 120, 0x3000_0000)]);
        regs.extend([prim(s17::LINE, at), rgbaq([0x10, 0xE0, 0x90, a], 1.0), xyz2(x + 100, y + 70, 0x5000_0000), xyz2(x + 150, y + 230, 0x5000_0000)]);
    }
    flat(&mut regs, (50, 2, 8, 4), [0x20, 0x40, 0x60, 0x40], AA1, 0x7000_0000);
    regs.extend([prim(s17::POINT, AA1), rgbaq([0x99, 0x88, 0x77, 0x40], 1.0), xyz2(60 * 16, 28 * 16, 0x7000_0000)]);
    ops.push(Op::Regs(regs));
    read(ops, &f, z.as_ref());
    Script { name: "s19-aa", ops: o }
}
