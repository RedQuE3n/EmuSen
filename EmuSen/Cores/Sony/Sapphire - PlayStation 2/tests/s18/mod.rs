//! The project's S18 programs (Sapphire_Native.md §31.6): texture mapping and fog, each a script in S16's form that
//! fills its textures, CLUTs and frames with generated data through IMAGE (a texture is read in whatever format the
//! program names, so memory written raw as PSMCT32 is reinterpreted, as a render target used as a texture is), draws
//! through A+D, and reads every frame page back raw as PSMCT32 (S16's route). The expected record is S17's model's
//! (`crate::s17::manual`), which hands texturing and fog to `crate::s17::manual::texture`, written apart from `src/gs/`.

#![allow(dead_code)]

use crate::s16::manual::{CT16, CT16S, CT24, CT32, T4, T4HH, T4HL, T8, T8H, Z16, Z16S, Z24, Z32};
use crate::s16::{Op, Script, upload};
use crate::s17::{self, ALWAYS, Buf, LINE, POINT, Reg, SPRITE, TRIANGLE, TRIANGLE_FAN, TRIANGLE_STRIP, prim, test, xyz2};

pub mod r {
    pub const RGBAQ: u8 = 0x01;
    pub const ST: u8 = 0x02;
    pub const UV: u8 = 0x03;
    pub const XYZF2: u8 = 0x04;
    pub const TEX0_1: u8 = 0x06;
    pub const TEX0_2: u8 = 0x07;
    pub const CLAMP_1: u8 = 0x08;
    pub const CLAMP_2: u8 = 0x09;
    pub const FOG: u8 = 0x0A;
    pub const TEX1_1: u8 = 0x14;
    pub const TEX1_2: u8 = 0x15;
    pub const TEX2_1: u8 = 0x16;
    pub const TEX2_2: u8 = 0x17;
    pub const TEXCLUT: u8 = 0x1C;
    pub const MIPTBP1_1: u8 = 0x34;
    pub const MIPTBP1_2: u8 = 0x35;
    pub const MIPTBP2_1: u8 = 0x36;
    pub const MIPTBP2_2: u8 = 0x37;
    pub const TEXA: u8 = 0x3B;
    pub const FOGCOL: u8 = 0x3D;
    pub const TEXFLUSH: u8 = 0x3F;
}

pub const GOURAUD: u64 = 1 << 3;
pub const TME: u64 = 1 << 4;
pub const FGE: u64 = 1 << 5;
pub const FST: u64 = 1 << 8;

pub const MODULATE: u32 = 0;
pub const DECAL: u32 = 1;
pub const HIGHLIGHT: u32 = 2;
pub const HIGHLIGHT2: u32 = 3;

/// TEX0's fields (its page).
#[derive(Clone, Copy, Debug)]
pub struct Tex {
    pub tbp: u32,
    pub tbw: u32,
    pub psm: u32,
    pub tw: u32,
    pub th: u32,
    pub tcc: bool,
    pub tfx: u32,
    pub cbp: u32,
    pub cpsm: u32,
    pub csm2: bool,
    pub csa: u32,
    pub cld: u32,
}

impl Tex {
    /// A texture at page `page` (blocks 32 × page), DECAL with TCC RGBA, no CLUT load.
    pub const fn new(page: u32, tbw: u32, psm: u32, tw: u32, th: u32) -> Tex {
        Tex { tbp: page * 32, tbw, psm, tw, th, tcc: true, tfx: DECAL, cbp: 0, cpsm: CT32, csm2: false, csa: 0, cld: 0 }
    }

    pub const fn clut(self, page: u32, cpsm: u32, csa: u32, cld: u32) -> Tex {
        Tex { cbp: page * 32, cpsm, csa, cld, ..self }
    }

    pub const fn func(self, tfx: u32, tcc: bool) -> Tex {
        Tex { tfx, tcc, ..self }
    }

    pub fn bits(&self) -> u64 {
        u64::from(self.tbp)
            | u64::from(self.tbw) << 14
            | u64::from(self.psm) << 20
            | u64::from(self.tw) << 26
            | u64::from(self.th) << 30
            | u64::from(self.tcc) << 34
            | u64::from(self.tfx) << 35
            | u64::from(self.cbp) << 37
            | u64::from(self.cpsm & 0xF) << 51
            | u64::from(self.csm2) << 55
            | u64::from(self.csa) << 56
            | u64::from(self.cld) << 61
    }
}

/// TEX1: LCM, MXL, MMAG, MMIN, MTBA, L and K (K in sixteenths, signed).
pub fn tex1(lcm: bool, mxl: u32, mmag: bool, mmin: u32, mtba: bool, l: u32, k: i32) -> u64 {
    u64::from(lcm) | u64::from(mxl) << 2 | u64::from(mmag) << 5 | u64::from(mmin) << 6 | u64::from(mtba) << 9 | u64::from(l) << 19 | u64::from(k as u32 & 0xFFF) << 32
}

/// A fixed LOD of `k` sixteenths with the filters given.
pub fn fixed(k: i32, mmag: bool, mmin: u32, mxl: u32) -> u64 {
    tex1(true, mxl, mmag, mmin, false, 0, k)
}

pub fn clamp(wms: u32, wmt: u32, minu: u32, maxu: u32, minv: u32, maxv: u32) -> u64 {
    u64::from(wms) | u64::from(wmt) << 2 | u64::from(minu) << 4 | u64::from(maxu) << 14 | u64::from(minv) << 24 | u64::from(maxv) << 34
}

pub fn texa(ta0: u32, aem: bool, ta1: u32) -> u64 {
    u64::from(ta0) | u64::from(aem) << 15 | u64::from(ta1) << 32
}

pub fn texclut(cbw: u32, cou: u32, cov: u32) -> u64 {
    u64::from(cbw) | u64::from(cou) << 6 | u64::from(cov) << 12
}

/// MIPTBP1 or MIPTBP2 from three (page, width) pairs.
pub fn miptbp(l: [(u32, u32); 3]) -> u64 {
    l.iter().enumerate().fold(0, |a, (i, &(p, w))| a | (u64::from(p * 32) | u64::from(w) << 14) << (20 * i))
}

pub fn rgbaq(c: [u32; 4], q: f32) -> Reg {
    (r::RGBAQ, u64::from(c[0] | c[1] << 8 | c[2] << 16 | c[3] << 24) | u64::from(q.to_bits()) << 32)
}

pub fn st(s: f32, t: f32) -> Reg {
    (r::ST, u64::from(s.to_bits()) | u64::from(t.to_bits()) << 32)
}

/// UV in sixteenths of a texel.
pub fn uv(u: u32, v: u32) -> Reg {
    (r::UV, u64::from(u & 0x3FFF) | u64::from(v & 0x3FFF) << 16)
}

pub fn fog(f: u32) -> Reg {
    (r::FOG, u64::from(f) << 56)
}

/// XYZF2 at window coordinates in sixteenths with fog coefficient `f`.
pub fn xyzf2(x: i32, y: i32, z: u32, f: u32) -> Reg {
    let (reg, v) = s17::vertex(r::XYZF2, x, y, z);
    (reg, v | u64::from(f) << 56)
}

/// The texture registers every program sets first, both contexts: TEXA, FOGCOL and TEXCLUT, a fixed LOD of 0 with
/// nearest filters, REPEAT, no mipmap bases, a PSMCT32 TEX0 that loads nothing; then TEXFLUSH.
pub fn begin(ops: &mut Vec<Op>) {
    s17::begin(ops);
    let mut regs = vec![(r::TEXA, texa(0x40, false, 0xC0)), (r::FOGCOL, 0), (r::TEXCLUT, 0)];
    for c in 0..2 {
        regs.extend([(r::TEX1_1 + c, fixed(-16, false, 0, 0)), (r::CLAMP_1 + c, 0), (r::MIPTBP1_1 + c, 0), (r::MIPTBP2_1 + c, 0), (r::TEX0_1 + c, Tex::new(0, 1, CT32, 6, 6).bits())]);
    }
    regs.push((r::TEXFLUSH, 0));
    ops.push(Op::Regs(regs));
}

/// `pages` pages from `page` filled raw with generated data, then TEXFLUSH (§3.4.13's step 4).
pub fn fill(ops: &mut Vec<Op>, page: u32, pages: u32, seed: u32) {
    upload(ops, CT32, (page * 32, 1), (0, 0, 64, 32 * pages), seed);
    ops.push(Op::Regs(vec![(r::TEXFLUSH, 0)]));
}

/// A frame page at `page`, filled with generated data and made context 1's target with the whole page scissored.
pub fn page(ops: &mut Vec<Op>, page: u32, seed: u32) -> Buf {
    let f = Buf::new(page, 1, CT32, 64, 32);
    f.fill(ops, seed);
    s17::target(ops, &f, None, test(true, ALWAYS));
    f
}

/// A sprite from pixel (x0, y0) to (x1, y1) with UV corners (u0, v0), (u1, v1) in sixteenths, flat colour `c`.
pub fn sprite_uv(regs: &mut Vec<Reg>, (x0, y0, x1, y1): (i32, i32, i32, i32), (u0, v0, u1, v1): (u32, u32, u32, u32), c: [u32; 4]) {
    regs.extend([rgbaq(c, 1.0), uv(u0, v0), xyz2(16 * x0, 16 * y0, 0), uv(u1, v1), xyz2(16 * x1, 16 * y1, 0)]);
}

/// The same with S and T, Q being `q` at both corners.
pub fn sprite_st(regs: &mut Vec<Reg>, (x0, y0, x1, y1): (i32, i32, i32, i32), (s0, t0, s1, t1): (f32, f32, f32, f32), q: f32, c: [u32; 4]) {
    regs.extend([rgbaq(c, q), st(s0, t0), xyz2(16 * x0, 16 * y0, 0), st(s1, t1), xyz2(16 * x1, 16 * y1, 0)]);
}

/// A whole-page sprite reading texels 1:1 at their centres from (u0, v0) texels.
pub fn whole_uv(regs: &mut Vec<Reg>, u0: u32, v0: u32, c: [u32; 4]) {
    sprite_uv(regs, (0, 0, 64, 32), (16 * u0 + 8, 16 * v0 + 8, 16 * (u0 + 64) + 8, 16 * (v0 + 32) + 8), c);
}

/// A texture, drawn with a whole-page UV sprite into a new frame page.
fn draw_page(ops: &mut Vec<Op>, at: u32, seed: u32, setup: Vec<Reg>, c: [u32; 4]) {
    let f = page(ops, at, seed);
    let mut regs = setup;
    regs.push(prim(SPRITE, TME | FST));
    whole_uv(&mut regs, 0, 0, c);
    ops.push(Op::Regs(regs));
    f.read(ops);
}

/// `s18-formats`: every format TEX0's PSM names, a 64×32 texture in each (the indexed ones through a PSMCT32 CLUT),
/// from memory written raw, drawn 1:1 with DECAL and TCC RGBA so each texel's colour and alpha (by TEXA for the 24- and
/// 16-bit formats) shows in the frame.
pub fn formats() -> Script {
    let mut o = Vec::new();
    let ops = &mut o;
    begin(ops);
    let list = [CT32, CT24, CT16, CT16S, T8, T4, T8H, T4HL, T4HH, Z32, Z24, Z16, Z16S];
    fill(ops, 300, list.len() as u32, 0x1818_0001);
    fill(ops, 320, 1, 0x1818_0002);
    for (i, psm) in list.into_iter().enumerate() {
        let tbw = if matches!(psm, T8 | T4) { 2 } else { 1 };
        let t = Tex::new(300 + i as u32, tbw, psm, 6, 5).clut(320, CT32, 0, 1);
        draw_page(ops, 20 + i as u32, 0x1818_0100 + i as u32, vec![(r::TEXA, texa(0x40, false, 0xC0)), (r::TEX0_1, t.bits())], [0x80; 4]);
    }
    Script { name: "s18-formats", ops: o }
}

/// `s18-clut`: CSM1 in every CPSM for PSMT8 and PSMT4 with CSA offsets, CSM2 at TEXCLUT's offsets; CLD's six codes
/// against CBP0 and CBP1 with the CLUT's memory rewritten between draws, so that a CLUT not reloaded shows its old
/// colours; TEX2 changing the CLUT and PSM alone; a 16-bit CLUT at CSA 20 overwriting the upper halves of a 32-bit one;
/// TEX0 with a non-indexed PSM and CLD 1, and CLD 6; CSA wrapping past the buffer, CSA in CSM2, and CPSM 1 (D-244).
pub fn clut() -> Script {
    let mut o = Vec::new();
    let ops = &mut o;
    begin(ops);
    fill(ops, 330, 2, 0x1818_0201);
    fill(ops, 340, 11, 0x1818_0202);
    let t8 = Tex::new(330, 2, T8, 6, 5);
    let t4 = Tex::new(331, 2, T4, 6, 5);
    let mut k = 0;
    let mut draw = |ops: &mut Vec<Op>, regs: Vec<Reg>| {
        draw_page(ops, 40 + k, 0x1818_0300 + k, regs, [0x80; 4]);
        k += 1;
    };
    draw(ops, vec![(r::TEX0_1, t8.clut(340, CT32, 0, 1).bits())]);
    draw(ops, vec![(r::TEX0_1, t8.clut(341, CT16, 0, 1).bits())]);
    draw(ops, vec![(r::TEX0_1, t8.clut(342, CT16S, 0, 1).bits())]);
    draw(ops, vec![(r::TEX0_1, t4.clut(343, CT32, 5, 1).bits())]);
    draw(ops, vec![(r::TEX0_1, t4.clut(344, CT16, 20, 1).bits())]);
    draw(ops, vec![(r::TEX0_1, t4.clut(345, CT16S, 31, 1).bits())]);
    let csm2 = |t: Tex, p: u32| Tex { csm2: true, ..t.clut(p, CT16, 0, 1) };
    draw(ops, vec![(r::TEXCLUT, texclut(4, 3, 5)), (r::TEX0_1, csm2(t8, 346).bits())]);
    draw(ops, vec![(r::TEXCLUT, texclut(2, 9, 33)), (r::TEX0_1, csm2(t4, 347).bits())]);
    // CLD 2 loads and sets CBP0; the CLUT's memory is rewritten; CLD 0 and CLD 4 at the same CBP keep the old CLUT.
    draw(ops, vec![(r::TEX0_1, t8.clut(348, CT32, 0, 2).bits())]);
    fill(ops, 348, 1, 0x1818_0203);
    draw(ops, vec![(r::TEX0_1, t8.clut(348, CT32, 0, 0).bits())]);
    draw(ops, vec![(r::TEX0_1, t8.clut(348, CT32, 0, 4).bits())]);
    // CLD 4 at another CBP loads; CLD 3 sets CBP1, whose memory is rewritten; CLD 5 at CBP1 keeps, elsewhere loads.
    draw(ops, vec![(r::TEX0_1, t8.clut(349, CT32, 0, 4).bits())]);
    draw(ops, vec![(r::TEX0_1, t8.clut(350, CT32, 0, 3).bits())]);
    fill(ops, 350, 1, 0x1818_0204);
    draw(ops, vec![(r::TEX0_1, t8.clut(350, CT32, 0, 5).bits())]);
    draw(ops, vec![(r::TEX0_1, t8.clut(348, CT32, 0, 5).bits())]);
    draw(ops, vec![(r::TEX0_1, t8.clut(349, CT32, 0, 4).bits())]);
    // TEX2: PSM, CBP and CPSM changed, the base and size kept from TEX0.
    let tex2 = |t: Tex| t.bits() & (0x3F << 20 | !((1u64 << 37) - 1));
    draw(ops, vec![(r::TEX0_1, t4.clut(343, CT32, 0, 1).bits()), (r::TEX2_1, tex2(Tex::new(0, 0, T8, 0, 0).clut(341, CT16, 0, 1)))]);
    // A 16-bit CLUT at CSA 20 lands on the upper halves of a 32-bit CLUT's entries 64 to 79.
    draw(ops, vec![(r::TEX0_1, t8.clut(340, CT32, 0, 1).bits()), (r::TEX0_1, t4.clut(344, CT16, 20, 1).bits()), (r::TEX0_1, t8.clut(340, CT32, 0, 0).bits())]);
    // A non-indexed PSM with CLD 1 (D-243), then CLD 6, which TEX0's page does not define.
    draw(ops, vec![(r::TEX0_1, t8.clut(340, CT32, 0, 1).bits()), (r::TEX0_1, Tex::new(330, 1, CT32, 6, 5).clut(341, CT32, 0, 1).bits()), (r::TEX0_1, t8.clut(340, CT32, 0, 0).bits())]);
    draw(ops, vec![(r::TEX0_1, t8.clut(342, CT32, 0, 6).bits())]);
    // CSA past a 32-bit CLUT's 256 entries and a 16-bit one's 512, CSA in CSM2, and a CPSM TEX0's page does not list.
    draw(ops, vec![(r::TEX0_1, t4.clut(343, CT32, 20, 1).bits())]);
    draw(ops, vec![(r::TEX0_1, t8.clut(341, CT16, 24, 1).bits())]);
    draw(ops, vec![(r::TEXCLUT, texclut(2, 9, 33)), (r::TEX0_1, Tex { csm2: true, ..t4.clut(347, CT16, 3, 1) }.bits())]);
    draw(ops, vec![(r::TEX0_1, t4.clut(343, CT24, 0, 1).bits())]);
    Script { name: "s18-clut", ops: o }
}

/// The fragment colours of `s18-tfx`'s sprites: the ends of each channel's range, 80h (1.0), and generated ones.
fn fragment(j: u32) -> [u32; 4] {
    const SET: [[u32; 4]; 8] = [[0x80; 4], [0xFF; 4], [0; 4], [0x7F, 0x81, 0x40, 0xC0], [0xFF, 0x00, 0x80, 0x01], [0x10, 0xF0, 0x88, 0xFF], [0x81, 0x7F, 0xC1, 0x3F], [0x01, 0xFE, 0x55, 0x80]];
    if j < 8 { SET[j as usize] } else { s17::colour(j * 7 + 3) }
}

/// Thirty-two flat 8×8 sprites, each reading texels 1:1, with the fragment colours above.
fn tiles(regs: &mut Vec<Reg>, attributes: u64) {
    regs.push(prim(SPRITE, TME | FST | attributes));
    for j in 0..32u32 {
        let (x, y) = (8 * (j % 8) as i32, 8 * (j / 8) as i32);
        sprite_uv(regs, (x, y, x + 8, y + 8), (16 * x as u32 + 8, 16 * y as u32 + 8, 16 * (x as u32 + 8) + 8, 16 * (y as u32 + 8) + 8), fragment(j));
    }
}

/// `s18-tfx`: MODULATE, DECAL, HIGHLIGHT and HIGHLIGHT2 with TCC RGB and RGBA over a PSMCT32 texture and thirty-two
/// fragment colours; PSMCT24 and PSMCT16 textures, whose alpha TEXA gives, under MODULATE, HIGHLIGHT and HIGHLIGHT2;
/// a Gouraud triangle under MODULATE.
pub fn tfx() -> Script {
    let mut o = Vec::new();
    let ops = &mut o;
    begin(ops);
    fill(ops, 360, 3, 0x1818_0401);
    let mut k = 0;
    let mut cases: Vec<(Tex, u64)> = Vec::new();
    for tcc in [false, true] {
        for tfx in [MODULATE, DECAL, HIGHLIGHT, HIGHLIGHT2] {
            cases.push((Tex::new(360, 1, CT32, 6, 5).func(tfx, tcc), texa(0x40, false, 0xC0)));
        }
    }
    cases.push((Tex::new(361, 1, CT24, 6, 5).func(MODULATE, true), texa(0x90, false, 0x20)));
    cases.push((Tex::new(361, 1, CT24, 6, 5).func(HIGHLIGHT, true), texa(0x90, false, 0x20)));
    cases.push((Tex::new(362, 1, CT16, 6, 5).func(HIGHLIGHT2, true), texa(0x30, false, 0xF0)));
    cases.push((Tex::new(362, 1, CT16, 6, 5).func(MODULATE, true), texa(0x30, false, 0xF0)));
    for (t, ta) in cases {
        let f = page(ops, 70 + k, 0x1818_0500 + k);
        let mut regs = vec![(r::TEXA, ta), (r::TEX0_1, t.bits())];
        tiles(&mut regs, 0);
        ops.push(Op::Regs(regs));
        f.read(ops);
        k += 1;
    }
    let f = page(ops, 70 + k, 0x1818_0500 + k);
    let mut regs = vec![(r::TEX0_1, Tex::new(360, 1, CT32, 6, 5).func(MODULATE, true).bits()), prim(TRIANGLE, TME | FST | GOURAUD)];
    for (x, y, u, v, c) in [(0, 0, 8, 8, [0xFF, 0x00, 0x80, 0xFF]), (1024, 0, 1032, 8, [0x00, 0xFF, 0x20, 0x00]), (0, 512, 8, 520, [0x80, 0x80, 0xFF, 0x80])] {
        regs.extend([rgbaq(c, 1.0), uv(u, v), xyz2(x, y, 0)]);
    }
    ops.push(Op::Regs(regs));
    f.read(ops);
    Script { name: "s18-tfx", ops: o }
}

/// `s18-uv`: UV coordinates on every primitive over a 64×64 PSMCT32 texture with REPEAT: an affine quad of two
/// triangles; sprites magnified, minified, with their corners reversed, with coordinates on texel boundaries and a
/// sixteenth either side; lines and points; a strip and a fan; coordinates past the texture, wrapped.
pub fn uv_program() -> Script {
    let mut o = Vec::new();
    let ops = &mut o;
    begin(ops);
    fill(ops, 370, 2, 0x1818_0601);
    let t = Tex::new(370, 1, CT32, 6, 6);
    let set = vec![(r::TEX0_1, t.bits())];
    // Two triangles, rotated and sheared in texture space.
    let f = page(ops, 90, 0x1818_0700);
    let mut regs = set.clone();
    regs.push(prim(TRIANGLE, TME | FST));
    for (x, y, u, v) in [(0, 0, 100, 37), (1024, 0, 900, 260), (0, 512, 333, 801), (1024, 0, 900, 260), (1024, 512, 1180, 1003), (0, 512, 333, 801)] {
        regs.extend([rgbaq([0x80; 4], 1.0), uv(u, v), xyz2(x, y, 0)]);
    }
    ops.push(Op::Regs(regs));
    f.read(ops);
    // Sprites.
    let f = page(ops, 91, 0x1818_0701);
    let mut regs = set.clone();
    regs.push(prim(SPRITE, TME | FST));
    sprite_uv(&mut regs, (0, 0, 16, 16), (8, 8, 8 + 128, 8 + 128), [0x80; 4]);
    sprite_uv(&mut regs, (16, 0, 32, 16), (8, 8, 8 + 512, 8 + 512), [0x80; 4]);
    sprite_uv(&mut regs, (48, 16, 32, 0), (16 * 40 + 8, 16 * 3 + 8, 16 * 24 + 8, 16 * 19 + 8), [0x80; 4]);
    sprite_uv(&mut regs, (48, 0, 64, 16), (0, 0, 256, 256), [0x80; 4]);
    sprite_uv(&mut regs, (0, 16, 16, 32), (1, 15, 257, 271), [0x80; 4]);
    sprite_uv(&mut regs, (16, 16, 32, 32), (15, 1, 271, 257), [0x80; 4]);
    sprite_uv(&mut regs, (32, 16, 48, 32), (16 * 60 + 8, 16 * 50 + 8, 16 * 76 + 8, 16 * 66 + 8), [0x80; 4]);
    sprite_uv(&mut regs, (48, 16, 64, 32), (16383 - 255, 16383 - 255, 16383, 16383), [0x80; 4]);
    ops.push(Op::Regs(regs));
    f.read(ops);
    // Lines and points.
    let f = page(ops, 92, 0x1818_0702);
    let mut regs = set.clone();
    regs.push(prim(LINE, TME | FST));
    for (k, &(x0, y0, x1, y1)) in [(8, 8, 1000, 40), (8, 500, 990, 60), (40, 8, 60, 500), (1000, 20, 30, 480), (100, 250, 900, 260), (500, 10, 520, 500)].iter().enumerate() {
        let k = k as u32;
        regs.extend([rgbaq([0x80; 4], 1.0), uv(37 * k + 3, 91 * k + 5), xyz2(x0, y0, 0), uv(613 + 40 * k, 17 + 150 * k), xyz2(x1, y1, 0)]);
    }
    regs.push(prim(POINT, TME | FST));
    for k in 0..24u32 {
        regs.extend([rgbaq([0x80; 4], 1.0), uv(53 * k + 7, 29 * k + 11), xyz2((40 * k % 1000) as i32 + 12, (17 * k % 480) as i32 + 9, 0)]);
    }
    ops.push(Op::Regs(regs));
    f.read(ops);
    // A strip and a fan with coordinates beyond the texture, wrapped.
    let f = page(ops, 93, 0x1818_0703);
    let mut regs = set;
    regs.push(prim(TRIANGLE_STRIP, TME | FST));
    for (x, y, u, v) in [(0, 0, 3000, 70), (0, 250, 3100, 900), (300, 0, 4000, 100), (300, 250, 4200, 1300), (500, 10, 5000, 300)] {
        regs.extend([rgbaq([0x80; 4], 1.0), uv(u, v), xyz2(x, y, 0)]);
    }
    regs.push(prim(TRIANGLE_FAN, TME | FST));
    for (x, y, u, v) in [(700, 380, 10, 10), (520, 260, 900, 33), (1000, 270, 1500, 400), (1010, 500, 200, 1700), (530, 505, 16000, 9000)] {
        regs.extend([rgbaq([0x80; 4], 1.0), uv(u, v), xyz2(x, y, 0)]);
    }
    ops.push(Op::Regs(regs));
    f.read(ops);
    Script { name: "s18-uv", ops: o }
}

/// `s18-stq`: S, T and Q over the same texture: sprites with Q 1 mapping texel centres 1:1, beyond 1.0 and below 0
/// (wrapped); a quad in perspective with Q from 0.25 to 2; Q negative at every vertex; S, T and Q small and large in
/// exponent; points and lines.
pub fn stq() -> Script {
    let mut o = Vec::new();
    let ops = &mut o;
    begin(ops);
    fill(ops, 370, 2, 0x1818_0601);
    let set = vec![(r::TEX0_1, Tex::new(370, 1, CT32, 6, 6).bits())];
    let c = [0x80; 4];
    let f = page(ops, 100, 0x1818_0800);
    let mut regs = set.clone();
    regs.push(prim(SPRITE, TME));
    sprite_st(&mut regs, (0, 0, 32, 16), (0.5 / 64.0, 0.5 / 64.0, 32.5 / 64.0, 16.5 / 64.0), 1.0, c);
    sprite_st(&mut regs, (32, 0, 64, 16), (1.25 + 0.5 / 64.0, 2.0 + 0.5 / 64.0, 1.25 + 32.5 / 64.0, 2.0 + 16.5 / 64.0), 1.0, c);
    sprite_st(&mut regs, (0, 16, 32, 32), (-0.75 + 0.5 / 64.0, -1.5 + 0.5 / 64.0, -0.75 + 32.5 / 64.0, -1.5 + 16.5 / 64.0), 1.0, c);
    sprite_st(&mut regs, (32, 16, 64, 32), (0.1, 0.2, 0.7, 0.4), 1.0, c);
    ops.push(Op::Regs(regs));
    f.read(ops);
    let f = page(ops, 101, 0x1818_0801);
    let mut regs = set.clone();
    regs.push(prim(TRIANGLE, TME));
    let quad = [(0, 0, 0.0f32, 0.0f32, 1.0f32), (1024, 0, 1.0, 0.0, 0.25), (0, 512, 0.0, 1.0, 2.0), (1024, 512, 1.0, 1.0, 0.5)];
    for i in [0usize, 1, 2, 1, 3, 2] {
        let (x, y, s, t, q) = quad[i];
        regs.extend([rgbaq(c, q), st(s * q, t * q), xyz2(x, y, 0)]);
    }
    ops.push(Op::Regs(regs));
    f.read(ops);
    let f = page(ops, 102, 0x1818_0802);
    let mut regs = set.clone();
    regs.push(prim(TRIANGLE, TME));
    for (x, y, s, t, q) in [(0, 0, -0.1f32, -0.2f32, -1.0f32), (500, 0, -0.9, -0.1, -0.5), (0, 500, -0.05, -0.7, -0.75)] {
        regs.extend([rgbaq(c, q), st(s, t), xyz2(x, y, 0)]);
    }
    for (x, y, s, t, q) in [(520, 0, 3.0e-4f32, 1.0e-4f32, 1.0e-3f32), (1020, 0, 7.1e-4, 2.0e-4, 1.1e-3), (520, 500, 1.0e-4, 9.0e-4, 0.9e-3)] {
        regs.extend([rgbaq(c, q), st(s, t), xyz2(x, y, 0)]);
    }
    for (x, y, s, t, q) in [(1020, 20, 300.0f32, 100.0f32, 1000.0f32), (1020, 510, 710.0, 900.0, 1100.0), (530, 510, 100.0, 600.0, 900.0)] {
        regs.extend([rgbaq(c, q), st(s, t), xyz2(x, y, 0)]);
    }
    ops.push(Op::Regs(regs));
    f.read(ops);
    let f = page(ops, 103, 0x1818_0803);
    let mut regs = set;
    regs.push(prim(LINE, TME));
    for k in 0..6 {
        let kf = k as f32;
        regs.extend([rgbaq(c, 1.0 + kf * 0.25), st(0.05 * kf, 0.9 - 0.1 * kf), xyz2(8 + 40 * k, 8 + 80 * k, 0), rgbaq(c, 0.5), st(0.4 + 0.07 * kf, 0.1 * kf), xyz2(1000 - 30 * k, 480 - 70 * k, 0)]);
    }
    regs.push(prim(POINT, TME));
    for k in 0..24 {
        let kf = k as f32;
        regs.extend([rgbaq(c, 0.5 + kf / 16.0), st(kf / 23.0, 1.0 - kf / 29.0), xyz2(40 * k + 12, 20 * k + 9, 0)]);
    }
    ops.push(Op::Regs(regs));
    f.read(ops);
    Script { name: "s18-stq", ops: o }
}

/// Four 32×16 quarters of a page, each a sprite over the whole of a 64×64 texture sampled at u = 1.5 + 2i and
/// v = 1.5 + 4j texels, never on a texel's boundary at any level, with its own TEX1 (and the rest where given).
fn quarters(ops: &mut Vec<Op>, at: u32, seed: u32, t0: Tex, cases: [Vec<Reg>; 4]) {
    let f = page(ops, at, seed);
    let mut regs = vec![(r::TEX0_1, t0.bits())];
    for (i, mut set) in cases.into_iter().enumerate() {
        let (x, y) = (32 * (i as i32 % 2), 16 * (i as i32 / 2));
        regs.append(&mut set);
        regs.push(prim(SPRITE, TME | FST));
        sprite_uv(&mut regs, (x, y, x + 32, y + 16), (24, 24, 24 + 1024, 24 + 1024), [0x80; 4]);
    }
    ops.push(Op::Regs(regs));
    f.read(ops);
}

/// `s18-filter`: bilinear magnification of an 8×8 texture through every sixteenth with REPEAT and CLAMP at its edges;
/// bilinear over PSMCT16 with TEXA and over PSMT8 through its CLUT; an LOD of exactly 0 (D-241); level selection by a
/// fixed K for every level and past MXL; the mipmapping filters' rounding and blending; LCM 0 with Q per quarter, with
/// L, per pixel in perspective, and with UV; MTBA's automatic base pointers (D-246).
pub fn filter() -> Script {
    let mut o = Vec::new();
    let ops = &mut o;
    begin(ops);
    fill(ops, 380, 1, 0x1818_0901);
    fill(ops, 382, 8, 0x1818_0902);
    fill(ops, 390, 10, 0x1818_0903);
    let small = Tex::new(380, 1, CT32, 3, 3);
    let mag = |t: Tex, wrap: u64| vec![(r::TEX0_1, t.bits()), (r::TEX1_1, fixed(-16, true, 0, 0)), (r::CLAMP_1, wrap)];
    let mut at = 110;
    // Bilinear magnification: one sixteenth a pixel across, two down, crossing the texture's right edge.
    for wrap in [clamp(0, 0, 0, 0, 0, 0), clamp(1, 1, 0, 0, 0, 0)] {
        let f = next_page(ops, &mut at);
        let mut regs = mag(small, wrap);
        regs.push(prim(SPRITE, TME | FST));
        sprite_uv(&mut regs, (0, 0, 64, 32), (100, 3, 164, 67), [0x80; 4]);
        ops.push(Op::Regs(regs));
        f.read(ops);
    }
    // Bilinear over PSMCT16 with TEXA, and over PSMT8 through a PSMCT32 CLUT.
    let f = next_page(ops, &mut at);
    let mut regs = mag(Tex::new(390, 1, CT16, 3, 3), clamp(0, 0, 0, 0, 0, 0));
    regs.push((r::TEXA, texa(0x12, false, 0xED)));
    regs.push(prim(SPRITE, TME | FST));
    sprite_uv(&mut regs, (0, 0, 32, 32), (37, 21, 37 + 32 * 3, 21 + 32 * 5), [0x80; 4]);
    regs.extend(mag(Tex::new(391, 2, T8, 3, 3).clut(392, CT32, 0, 1), clamp(0, 0, 0, 0, 0, 0)));
    regs.push(prim(SPRITE, TME | FST));
    sprite_uv(&mut regs, (32, 0, 64, 32), (37, 21, 37 + 32 * 3, 21 + 32 * 5), [0x80; 4]);
    ops.push(Op::Regs(regs));
    f.read(ops);
    // An LOD of exactly 0 with MMAG nearest and MMIN linear, beside K = -1/16 and K = +1/16 (D-241).
    let f = next_page(ops, &mut at);
    let mut regs = vec![(r::TEX0_1, small.bits()), (r::CLAMP_1, 0)];
    for (i, k) in [0, -1, 1, 0].into_iter().enumerate() {
        let x = 16 * i as i32;
        regs.push((r::TEX1_1, fixed(k, i == 3, 1, 0)));
        regs.push(prim(SPRITE, TME | FST));
        sprite_uv(&mut regs, (x, 0, x + 16, 32), (40, 3, 40 + 16 * 5, 3 + 32 * 3), [0x80; 4]);
    }
    ops.push(Op::Regs(regs));
    f.read(ops);
    // A 64×64 mipmap of seven levels.
    let chain = Tex::new(382, 1, CT32, 6, 6);
    let bases = vec![(r::MIPTBP1_1, miptbp([(384, 1), (385, 1), (386, 1)])), (r::MIPTBP2_1, miptbp([(387, 1), (388, 1), (389, 1)]))];
    // LODs between 0 and 1: levels 0 and 1 blended by 7/16, bilinear at level 0, trilinear by 15/16, and LOD 1.0.
    let c = [(7, 3), (8, 1), (15, 5), (16, 1)].map(|(k, mmin)| {
        let mut v = bases.clone();
        v.push((r::TEX1_1, fixed(k, false, mmin, 6)));
        v
    });
    at += 1;
    quarters(ops, at - 1, 0x1818_0B40, chain, c);
    for list in [[(7, 2, 6), (8, 2, 6), (0x18, 2, 6), (0x2F, 2, 6)], [(0x40, 2, 6), (0x58, 2, 6), (0x70, 2, 6), (0x40, 2, 2)], [(0x14, 3, 6), (0x1F, 3, 6), (0x2A, 5, 6), (0x28, 4, 6)]] {
        let c = list.map(|(k, mmin, mxl)| {
            let mut v = bases.clone();
            v.push((r::TEX1_1, fixed(k, false, mmin, mxl)));
            v
        });
        at += 1;
        quarters(ops, at - 1, 0x1818_0B00 + at, chain, c);
    }
    // LCM 0: Q per quarter, then L.
    let f = next_page(ops, &mut at);
    let mut regs = vec![(r::TEX0_1, chain.bits())];
    regs.extend(bases.clone());
    for (i, (q, l, k, mmin)) in [(1.0f32, 0, 16, 2), (0.5, 0, 0, 2), (0.3, 0, 0, 3), (0.3, 2, -32, 3)].into_iter().enumerate() {
        let (x, y) = (32 * (i as i32 % 2), 16 * (i as i32 / 2));
        regs.push((r::TEX1_1, tex1(false, 6, false, mmin, false, l, k)));
        regs.push(prim(SPRITE, TME));
        sprite_st(&mut regs, (x, y, x + 32, y + 16), (q * 1.5 / 64.0, q * 1.5 / 64.0, q * 65.5 / 64.0, q * 65.5 / 64.0), q, [0x80; 4]);
    }
    ops.push(Op::Regs(regs));
    f.read(ops);
    // LCM 0 per pixel: a quad in perspective, Q from 1 to 1/8, trilinear and nearest-linear.
    let f = next_page(ops, &mut at);
    let mut regs = vec![(r::TEX0_1, chain.bits())];
    regs.extend(bases.clone());
    for (half, mmin) in [(0, 3), (1, 5)] {
        regs.push((r::TEX1_1, tex1(false, 6, true, mmin, false, 0, 0)));
        regs.push(prim(TRIANGLE, TME));
        let x0 = 512 * half;
        let quad = [(x0, 0, 0.0f32, 0.0f32, 1.0f32), (x0 + 512, 0, 1.0, 0.0, 0.125), (x0, 512, 0.0, 1.0, 0.75), (x0 + 512, 512, 1.0, 1.0, 0.2)];
        for i in [0usize, 1, 2, 1, 3, 2] {
            let (x, y, s, t, q) = quad[i];
            regs.extend([rgbaq([0x80; 4], q), st(s * q, t * q), xyz2(x, y, 0)]);
        }
    }
    ops.push(Op::Regs(regs));
    f.read(ops);
    // LCM 0 with UV: Q taken from RGBAQ; a triangle whose vertices' Q differ, the last 1.0.
    let f = next_page(ops, &mut at);
    let mut regs = vec![(r::TEX0_1, chain.bits())];
    regs.extend(bases.clone());
    regs.push((r::TEX1_1, tex1(false, 6, false, 2, false, 0, 0)));
    regs.push(prim(SPRITE, TME | FST));
    regs.extend([rgbaq([0x80; 4], 0.25), uv(16, 32), xyz2(0, 0, 0), uv(16 + 1024, 32 + 1024), xyz2(512, 512, 0)]);
    regs.push(prim(TRIANGLE, TME | FST));
    for (x, y, u, v, q) in [(520, 0, 16, 16, 0.25f32), (1024, 0, 1000, 40, 0.25), (520, 512, 50, 1000, 1.0)] {
        regs.extend([rgbaq([0x80; 4], q), uv(u, v), xyz2(x, y, 0)]);
    }
    ops.push(Op::Regs(regs));
    f.read(ops);
    // MTBA: levels 1 to 3 placed by the GS; then MTBA 0, which reads MIPTBP1 as the calculation left it.
    let mtba = |k: i32, on: bool| tex1(true, 3, false, 2, on, 0, k);
    let auto = Tex::new(393, 1, CT32, 6, 6);
    at += 1;
    quarters(ops, at - 1, 0x1818_0B80, auto, [vec![(r::TEX1_1, mtba(0x10, true)), (r::TEX0_1, auto.bits())], vec![(r::TEX1_1, mtba(0x20, true))], vec![(r::TEX1_1, mtba(0x30, true))], vec![(r::TEX1_1, mtba(0x10, false))]]);
    let wide = Tex { tbw: 4, ..auto };
    let t8 = Tex::new(396, 2, T8, 5, 5).clut(392, CT32, 0, 1);
    at += 1;
    quarters(ops, at - 1, 0x1818_0B81, wide, [vec![(r::TEX1_1, mtba(0x10, true)), (r::TEX0_1, wide.bits())], vec![(r::TEX1_1, mtba(0x20, true)), (r::TEX0_1, t8.bits())], vec![(r::TEX1_1, mtba(0x10, true))], vec![(r::TEX1_1, mtba(0x30, true))]]);
    Script { name: "s18-filter", ops: o }
}

/// The next frame page of `s18-filter`.
fn next_page(ops: &mut Vec<Op>, at: &mut u32) -> Buf {
    *at += 1;
    page(ops, *at - 1, 0x1818_0A00 + *at)
}

/// A whole-page ST sprite with Q 1 taking `u = u0 + du × x` texels at pixel x's centre and `v = v0 + dv × y`.
fn st_page(regs: &mut Vec<Reg>, (u0, du): (f32, f32), (v0, dv): (f32, f32), size: f32) {
    regs.push(prim(SPRITE, TME));
    sprite_st(regs, (0, 0, 64, 32), (u0 / size, v0 / size, (u0 + 64.0 * du) / size, (v0 + 32.0 * dv) / size), 1.0, [0x80; 4]);
}

/// `s18-clamp`: a 16×16 texture inside generated memory, read from 24 texels before it to 40 past it across and 8
/// before to 24 past down, in each wrap mode and pairs of them, point sampled at texel centres and bilinear; a region
/// with its bounds crossed (MINU above MAXU), REGION_REPEAT reaching past the texture; both region modes at level 1.
pub fn clamp_program() -> Script {
    let mut o = Vec::new();
    let ops = &mut o;
    begin(ops);
    fill(ops, 400, 3, 0x1818_0C01);
    let t = Tex::new(400, 1, CT32, 4, 4);
    let near = (-24.0f32 + 0.5, 1.0f32);
    let nearv = (-8.0f32 + 0.5, 1.0f32);
    let lin = (-20.0f32 + 0.3, 0.75f32);
    let linv = (-6.0f32 + 0.1, 0.8125f32);
    let cases: [(u64, bool); 14] = [
        (clamp(0, 0, 0, 0, 0, 0), false),
        (clamp(1, 1, 0, 0, 0, 0), false),
        (clamp(2, 2, 3, 11, 2, 13), false),
        (clamp(3, 3, 7, 8, 3, 4), false),
        (clamp(0, 1, 0, 0, 0, 0), false),
        (clamp(2, 3, 3, 11, 3, 4), false),
        (clamp(2, 2, 9, 5, 12, 6), false),
        (clamp(3, 3, 15, 32, 15, 16), false),
        (clamp(0, 0, 0, 0, 0, 0), true),
        (clamp(1, 1, 0, 0, 0, 0), true),
        (clamp(2, 2, 3, 11, 2, 13), true),
        (clamp(3, 3, 7, 8, 3, 4), true),
        (clamp(2, 2, 3, 11, 2, 13), false),
        (clamp(3, 3, 7, 8, 3, 4), false),
    ];
    for (k, (wrap, bilinear)) in cases.into_iter().enumerate() {
        let f = page(ops, 130 + k as u32, 0x1818_0D00 + k as u32);
        let level1 = k >= 12;
        let lod = if level1 { fixed(0x10, bilinear, 2, 1) } else { fixed(-16, bilinear, 0, 0) };
        let mut regs = vec![(r::TEX0_1, t.bits()), (r::TEX1_1, lod), (r::CLAMP_1, wrap), (r::MIPTBP1_1, miptbp([(402, 1), (0, 1), (0, 1)]))];
        let (u, v) = if bilinear { (lin, linv) } else { (near, nearv) };
        st_page(&mut regs, u, v, 16.0);
        ops.push(Op::Regs(regs));
        f.read(ops);
    }
    Script { name: "s18-clamp", ops: o }
}

/// `s18-texa`: TEXA's expansion with AEM 0 and 1 for PSMCT16 and PSMCT24 textures and a PSMT4 texture's PSMCT16 CLUT,
/// their black texels and entries drawn into them by the GS first (black with A 0 and with A 1); DECAL with TCC RGBA
/// shows the alpha, MODULATE with TCC RGB keeps the fragment's.
pub fn texa_program() -> Script {
    let mut o = Vec::new();
    let ops = &mut o;
    begin(ops);
    fill(ops, 410, 4, 0x1818_0E01);
    // Black squares into the textures and the CLUT: with A 0 (alpha below 80h) and with A 1.
    for (pg, psm, h) in [(410, CT16, 64), (411, CT24, 32), (412, CT16, 64)] {
        let b = Buf::new(pg, 1, psm, 64, h);
        s17::target(ops, &b, None, test(true, ALWAYS));
        let mut regs = vec![prim(SPRITE, 0)];
        for j in 0..12 {
            let (x, y) = if pg == 412 { (j % 8, j / 8) } else { ((j * 5) % 60, (j * 11) % 28) };
            let (w, hh) = if pg == 412 { (1, 1) } else { (3, 2) };
            regs.extend([s17::rgba([0, 0, 0, if j % 2 == 0 { 0x10 } else { 0xA0 }]), xyz2(16 * x, 16 * y, 0), xyz2(16 * (x + w), 16 * (y + hh), 0)]);
        }
        ops.push(Op::Regs(regs));
    }
    ops.push(Op::Regs(vec![(r::TEXFLUSH, 0)]));
    let t16 = Tex::new(410, 1, CT16, 6, 5);
    let t24 = Tex::new(411, 1, CT24, 6, 5);
    let t4 = Tex::new(413, 2, T4, 6, 5).clut(412, CT16, 0, 1);
    let cases = [
        (t16, texa(0x11, false, 0xEE)),
        (t16, texa(0x11, true, 0xEE)),
        (t24, texa(0x22, false, 0x99)),
        (t24, texa(0x22, true, 0x99)),
        (t4, texa(0x33, true, 0x77)),
        (t16.func(MODULATE, true), texa(0x44, true, 0xF0)),
        (t24.func(MODULATE, false), texa(0x55, true, 0x66)),
    ];
    for (k, (t, ta)) in cases.into_iter().enumerate() {
        draw_page(ops, 150 + k as u32, 0x1818_0F00 + k as u32, vec![(r::TEXA, ta), (r::TEX0_1, t.bits())], [0x90, 0x70, 0x50, 0x60]);
    }
    Script { name: "s18-texa", ops: o }
}

/// `s18-fog`: FOGCOL blended with flat sprites at F 0, 1, 40h, 7Fh, 80h, FEh and FFh, set by FOG and by XYZF2's field,
/// the first vertex's F ignored; F interpolated over Gouraud and flat triangles, along lines (held at the range past an
/// end) and at points; fog after MODULATE on a texture.
pub fn fog_program() -> Script {
    let mut o = Vec::new();
    let ops = &mut o;
    begin(ops);
    fill(ops, 370, 2, 0x1818_0601);
    let f = page(ops, 170, 0x1818_1000);
    let mut regs = vec![(r::FOGCOL, 0x00F0_9A12), prim(SPRITE, FGE)];
    for (j, fv) in [0u32, 1, 0x40, 0x7F, 0x80, 0xFE, 0xFF, 0x33].into_iter().enumerate() {
        let x = 8 * j as i32;
        regs.extend([s17::rgba(fragment(j as u32)), fog(0xFF - fv), xyz2(16 * x, 0, 0), fog(fv), xyz2(16 * (x + 8), 16 * 16, 0)]);
        regs.extend([s17::rgba(fragment(j as u32 + 8)), xyzf2(16 * x, 16 * 16, 0, 0x5A), xyzf2(16 * (x + 8), 16 * 32, 0, fv)]);
    }
    ops.push(Op::Regs(regs));
    f.read(ops);
    let f = page(ops, 171, 0x1818_1001);
    let mut regs = vec![(r::FOGCOL, 0x0033_CC77), prim(TRIANGLE, FGE | GOURAUD)];
    for (x, y, c, fv) in [(0, 0, [0xFF, 0, 0x80, 0xFF], 0), (1000, 0, [0, 0xFF, 0x40, 0x80], 0xFF), (0, 500, [0x80, 0x80, 0xFF, 0], 0x80)] {
        regs.extend([s17::rgba(c), xyzf2(x, y, 0, fv)]);
    }
    regs.push(prim(TRIANGLE, FGE));
    for (x, y, fv) in [(1020, 20, 0x10), (1020, 510, 0xF0), (40, 510, 0x77)] {
        regs.extend([s17::rgba([0xC0, 0x30, 0x90, 0x80]), xyzf2(x, y, 0, fv)]);
    }
    ops.push(Op::Regs(regs));
    f.read(ops);
    let f = page(ops, 172, 0x1818_1002);
    let mut regs = vec![(r::FOGCOL, 0x0011_2233), prim(LINE, FGE | GOURAUD)];
    for k in 0..8 {
        regs.extend([s17::rgba(fragment(k as u32)), xyzf2(5 + 128 * k, 3 + 8 * k, 0, (31 * k) as u32), s17::rgba(fragment(k as u32 + 9)), xyzf2(120 + 120 * k, 500 - 40 * k, 0, 0xFF - (29 * k) as u32)]);
    }
    regs.push(prim(POINT, FGE));
    for k in 0..16 {
        regs.extend([s17::rgba(fragment(k as u32 + 3)), xyzf2(60 * k + 20, 256 + 8 * k, 0, (17 * k) as u32)]);
    }
    ops.push(Op::Regs(regs));
    f.read(ops);
    let f = page(ops, 173, 0x1818_1003);
    let mut regs = vec![(r::FOGCOL, 0x0080_40C0), (r::TEX0_1, Tex::new(370, 1, CT32, 6, 6).func(MODULATE, true).bits()), prim(TRIANGLE, FGE | TME | FST)];
    for (x, y, u, v, fv) in [(0, 0, 8, 8, 0xFF), (1024, 0, 1032, 8, 0x00), (0, 512, 8, 520, 0x80)] {
        regs.extend([rgbaq([0xA0, 0x80, 0x60, 0x80], 1.0), uv(u, v), xyzf2(x, y, 0, fv)]);
    }
    ops.push(Op::Regs(regs));
    f.read(ops);
    Script { name: "s18-fog", ops: o }
}

pub fn all() -> Vec<Script> {
    vec![formats(), clut(), tfx(), uv_program(), stq(), filter(), clamp_program(), texa_program(), fog_program()]
}
