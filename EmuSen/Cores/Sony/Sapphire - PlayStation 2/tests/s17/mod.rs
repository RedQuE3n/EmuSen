//! The project's S17 programs (Sapphire_Native.md §27.6): drawing, each a script in S16's form (`s16::Op`) that sets
//! the drawing environment through A+D, fills its frame and Z buffers with generated data through IMAGE so that every
//! pixel written shows, draws through the vertex registers, and reads every page it drew back local to host raw as
//! PSMCT32, the form every referee reads back (S16's route, Sapphire_Referees.md §6's form C). The expected record comes
//! from `manual`, written apart from `src/gs/`.

#![allow(dead_code)]

pub mod manual;

use crate::s16::manual::{CT16, CT16S, CT24, CT32, Z16, Z16S, Z24, Z32};
use crate::s16::{self, Op, Script, download, upload};

pub mod r {
    pub const PRIM: u8 = 0x00;
    pub const RGBAQ: u8 = 0x01;
    pub const XYZF2: u8 = 0x04;
    pub const XYZ2: u8 = 0x05;
    pub const FOG: u8 = 0x0A;
    pub const XYZF3: u8 = 0x0C;
    pub const XYZ3: u8 = 0x0D;
    pub const XYOFFSET_1: u8 = 0x18;
    pub const XYOFFSET_2: u8 = 0x19;
    pub const PRMODECONT: u8 = 0x1A;
    pub const PRMODE: u8 = 0x1B;
    pub const SCANMSK: u8 = 0x22;
    pub const SCISSOR_1: u8 = 0x40;
    pub const SCISSOR_2: u8 = 0x41;
    pub const ALPHA_1: u8 = 0x42;
    pub const ALPHA_2: u8 = 0x43;
    pub const DTHE: u8 = 0x45;
    pub const COLCLAMP: u8 = 0x46;
    pub const TEST_1: u8 = 0x47;
    pub const TEST_2: u8 = 0x48;
    pub const PABE: u8 = 0x49;
    pub const FBA_1: u8 = 0x4A;
    pub const FBA_2: u8 = 0x4B;
    pub const FRAME_1: u8 = 0x4C;
    pub const FRAME_2: u8 = 0x4D;
    pub const ZBUF_1: u8 = 0x4E;
    pub const ZBUF_2: u8 = 0x4F;
}

pub const POINT: u64 = 0;
pub const LINE: u64 = 1;
pub const LINE_STRIP: u64 = 2;
pub const TRIANGLE: u64 = 3;
pub const TRIANGLE_STRIP: u64 = 4;
pub const TRIANGLE_FAN: u64 = 5;
pub const SPRITE: u64 = 6;
pub const GOURAUD: u64 = 1 << 3;
pub const CONTEXT_2: u64 = 1 << 9;

pub const NEVER: u64 = 0;
pub const ALWAYS: u64 = 1;
pub const GEQUAL: u64 = 2;
pub const GREATER: u64 = 3;

/// XYOFFSET in both contexts unless a program says otherwise: window (0, 0) at primitive (1536.0, 1792.0).
pub const OFF: (u32, u32) = (1536 * 16, 1792 * 16);

pub type Reg = (u8, u64);

pub fn prim(kind: u64, attributes: u64) -> Reg {
    (r::PRIM, kind | attributes)
}

/// RGBAQ with Q 1.0.
pub fn rgba(c: [u32; 4]) -> Reg {
    (r::RGBAQ, u64::from(c[0] | c[1] << 8 | c[2] << 16 | c[3] << 24) | 0x3F80_0000u64 << 32)
}

/// A vertex at window coordinates (`x`, `y`) in sixteenths, through the register `reg`, with XYOFFSET at `OFF`.
pub fn vertex(reg: u8, x: i32, y: i32, z: u32) -> Reg {
    let px = (OFF.0 as i32 + x) as u64 & 0xFFFF;
    let py = (OFF.1 as i32 + y) as u64 & 0xFFFF;
    let zf = if matches!(reg, r::XYZF2 | r::XYZF3) { u64::from(z & 0xFF_FFFF) } else { u64::from(z) };
    (reg, px | py << 16 | zf << 32)
}

pub fn xyz2(x: i32, y: i32, z: u32) -> Reg {
    vertex(r::XYZ2, x, y, z)
}

pub fn frame(fbp: u32, fbw: u32, psm: u32, mask: u32) -> u64 {
    u64::from(fbp) | u64::from(fbw) << 16 | u64::from(psm) << 24 | u64::from(mask) << 32
}

pub fn zbuf(zbp: u32, psm: u32, zmsk: bool) -> u64 {
    u64::from(zbp) | u64::from(psm & 0xF) << 24 | u64::from(zmsk) << 32
}

pub fn scissor(x0: u32, x1: u32, y0: u32, y1: u32) -> u64 {
    u64::from(x0) | u64::from(x1) << 16 | u64::from(y0) << 32 | u64::from(y1) << 48
}

pub fn test(zte: bool, ztst: u64) -> u64 {
    u64::from(zte) << 16 | ztst << 17
}

pub fn offset(x: u32, y: u32) -> u64 {
    u64::from(x) | u64::from(y) << 32
}

/// A buffer the programs draw into and read back: FBP or ZBP (pages of 2,048 words), width in units of 64, format, and
/// its pages' rectangle in its own pixels, which must be whole pages.
#[derive(Clone, Copy)]
pub struct Buf {
    pub page: u32,
    pub bw: u32,
    pub psm: u32,
    pub w: u32,
    pub h: u32,
}

impl Buf {
    pub const fn new(page: u32, bw: u32, psm: u32, w: u32, h: u32) -> Buf {
        Buf { page, bw, psm, w, h }
    }

    /// The same pages as a PSMCT32 rectangle: a 16-bit page is 64×64 pixels, a 32-bit one 64×32.
    fn raw(&self) -> (u32, u32) {
        if matches!(self.psm, CT16 | CT16S | Z16 | Z16S) { (self.w, self.h / 2) } else { (self.w, self.h) }
    }

    /// Generated data over every word of its pages.
    pub fn fill(&self, ops: &mut Vec<Op>, seed: u32) {
        let (w, h) = self.raw();
        upload(ops, CT32, (self.page * 32, self.bw), (0, 0, w, h), seed);
    }

    /// Its pages read back raw as PSMCT32.
    pub fn read(&self, ops: &mut Vec<Op>) {
        let (w, h) = self.raw();
        download(ops, CT32, (self.page * 32, self.bw), (0, 0, w, h));
    }
}

/// The registers every program sets first, so that nothing a referee's start-up left in them matters: PRIM's
/// attributes in force, no SCANMSK, dithering, PABE, FBA or blending, and no depth test or Z write.
pub fn begin(ops: &mut Vec<Op>) {
    let mut regs = vec![(r::PRMODECONT, 1), (r::PRMODE, 0), (r::SCANMSK, 0), (r::DTHE, 0), (r::COLCLAMP, 1), (r::PABE, 0)];
    for c in 0..2 {
        regs.extend([
            (r::FBA_1 + c, 0),
            (r::ALPHA_1 + c, 0),
            (r::TEST_1 + c, test(true, ALWAYS)),
            (r::ZBUF_1 + c, zbuf(0, Z32, true)),
            (r::XYOFFSET_1 + c, offset(OFF.0, OFF.1)),
        ]);
    }
    ops.push(Op::Regs(regs));
}

/// Context 1 drawing into `f`, with the scissoring area its whole picture, optionally a Z buffer, and a test.
pub fn target(ops: &mut Vec<Op>, f: &Buf, z: Option<(&Buf, bool)>, t: u64) {
    let zb = z.map_or(zbuf(0, Z32, true), |(b, masked)| zbuf(b.page, b.psm, masked));
    ops.push(Op::Regs(vec![
        (r::FRAME_1, frame(f.page, f.bw, f.psm, 0)),
        (r::ZBUF_1, zb),
        (r::SCISSOR_1, scissor(0, f.w - 1, 0, f.h - 1)),
        (r::TEST_1, t),
    ]));
}

/// A distinct, recognisable colour for item `k`.
pub fn colour(k: u32) -> [u32; 4] {
    [(k * 37 + 11) & 0xFF, (k * 101 + 50) & 0xFF, (k * 59 + 200) & 0xFF, (k * 13 + 3) & 0xFF]
}

/// `s17-points`: 128 points, one to a 4×4 cell, at every fractional X (sixteenths 0 to 15) and eight fractional Ys,
/// the halves included, with the scissoring area two pixels in on the left and one on the other sides; then points
/// about window 0 with an unclipped scissor, a coordinate of -0.5 rounding to pixel 0 and -0.5625 to pixel -1.
pub fn points() -> Script {
    let mut o = Vec::new();
    let ops = &mut o;
    begin(ops);
    let f = Buf::new(10, 1, CT32, 64, 32);
    f.fill(ops, 0x1717_0001);
    target(ops, &f, None, test(true, ALWAYS));
    ops.push(Op::Regs(vec![(r::SCISSOR_1, scissor(2, 61, 1, 30))]));
    let fys = [0, 1, 7, 8, 9, 15, 4, 12];
    let mut regs = vec![prim(POINT, 0)];
    for k in 0..128u32 {
        let (cx, cy) = (k % 16, k / 16);
        let (fx, fy) = ((k % 16) as i32, fys[(k / 16) as usize]);
        regs.push(rgba(colour(k)));
        regs.push(xyz2((4 * cx as i32 + 1) * 16 + fx, (4 * cy as i32 + 1) * 16 + fy, 0));
    }
    ops.push(Op::Regs(regs));
    f.read(ops);
    let g = Buf::new(11, 1, CT32, 64, 32);
    g.fill(ops, 0x1717_0002);
    target(ops, &g, None, test(true, ALWAYS));
    let mut regs = vec![prim(POINT, 0)];
    for (k, (x, y)) in [(-8, 40), (-9, 72), (40, -8), (72, -9), (-8, -8), (-24, 100), (1016, 504), (1015, 503), (1007, 120), (1000, 488)].into_iter().enumerate() {
        regs.push(rgba(colour(200 + k as u32)));
        regs.push(xyz2(x, y, 0));
    }
    ops.push(Op::Regs(regs));
    g.read(ops);
    Script { name: "s17-points", ops: o }
}

/// `s17-sprites`: 32 sprites, one to an 8×8 cell, their corners at fractions that put a side on a pixel centre, just
/// short of one, just past one and between two, and half of them given with the second vertex at the top left; a sprite
/// with no centre inside it, one of no width, one across the scissoring area's edges.
pub fn sprites() -> Script {
    let mut o = Vec::new();
    let ops = &mut o;
    begin(ops);
    let f = Buf::new(12, 1, CT32, 64, 32);
    f.fill(ops, 0x1717_0003);
    target(ops, &f, None, test(true, ALWAYS));
    ops.push(Op::Regs(vec![(r::SCISSOR_1, scissor(1, 62, 1, 30))]));
    let fr = [0, 1, 8, 15];
    let mut regs = vec![prim(SPRITE, 0)];
    for k in 0..32u32 {
        let (cx, cy) = (8 * (k % 8) as i32, 8 * (k / 8) as i32);
        let (x0, y0) = ((cx + 1) * 16 + fr[(k % 4) as usize], (cy + 1) * 16 + fr[((k / 4) % 4) as usize]);
        let (x1, y1) = ((cx + 5) * 16 + fr[((k / 2) % 4) as usize], (cy + 6) * 16 + fr[((k / 8 + k) % 4) as usize]);
        regs.push(rgba(colour(k ^ 0x55)));
        regs.push(xyz2(if k % 2 == 0 { x0 } else { x1 }, if k % 2 == 0 { y0 } else { y1 }, 0));
        regs.push(rgba(colour(k)));
        regs.push(xyz2(if k % 2 == 0 { x1 } else { x0 }, if k % 2 == 0 { y1 } else { y0 }, 0));
    }
    ops.push(Op::Regs(regs));
    f.read(ops);
    let g = Buf::new(13, 1, CT32, 64, 32);
    g.fill(ops, 0x1717_0004);
    target(ops, &g, None, test(true, ALWAYS));
    ops.push(Op::Regs(vec![(r::SCISSOR_1, scissor(3, 60, 2, 29))]));
    let cases: [(i32, i32, i32, i32); 6] = [(20, 20, 28, 300), (40, 40, 40, 200), (-50, -40, 160, 120), (900, 400, 1200, 700), (200, 37, 600, 37), (500, 20, 700, 36)];
    let mut regs = vec![prim(SPRITE, 0)];
    for (k, (x0, y0, x1, y1)) in cases.into_iter().enumerate() {
        regs.push(rgba(colour(k as u32 + 64)));
        regs.push(xyz2(x0, y0, 0));
        regs.push(xyz2(x1, y1, 0));
    }
    ops.push(Op::Regs(regs));
    g.read(ops);
    Script { name: "s17-sprites", ops: o }
}

/// The lines of `s17-lines`, relative to a 16×16 cell's centre in sixteenths: level, upright and diagonal lines on
/// centres and on halves, every octant both ways, slopes of exactly a half, a diagonal through diamonds' corners, ends
/// on a diamond's corner, a line inside one diamond, and one of no length.
const LINES: [(i32, i32, i32, i32); 32] = [
    (-96, 0, 96, 0),
    (-96, 8, 96, 8),
    (0, -96, 0, 96),
    (8, -96, 8, 96),
    (-96, -96, 96, 96),
    (96, -96, -96, 96),
    (-100, -37, 90, 41),
    (90, 41, -100, -37),
    (-37, -100, 41, 90),
    (41, 90, -37, -100),
    (-100, 37, 90, -41),
    (37, -100, -41, 90),
    (-99, -13, 101, 13),
    (-5, -99, 7, 101),
    (-96, 0, 96, 32),
    (-88, -8, 88, 8),
    (-91, -3, 93, 5),
    (-8, -8, 8, 8),
    (0, 0, 8, 0),
    (0, 0, 0, 0),
    (3, 2, 5, 4),
    (-100, -50, 100, 50),
    (-50, -100, 50, 100),
    (-64, -64, 64, 64),
    (-72, -64, 72, 80),
    (-104, 8, 104, -8),
    (5, -103, -5, 103),
    (-97, -97, 95, 95),
    (-96, 4, 96, 12),
    (-120, -120, 120, -100),
    (0, -120, 0, 120),
    (120, 120, -120, -110),
];

/// `s17-lines`: the lines above, Gouraud and flat by turns; then a self-crossing line strip and a set of lines that
/// meet end to start, drawn with GREATER over Z already written at the same value, so that a pixel drawn twice keeps
/// its first colour and shows that §3.2.9's "drawn only once" holds.
pub fn lines() -> Script {
    let mut o = Vec::new();
    let ops = &mut o;
    begin(ops);
    let f = Buf::new(14, 2, CT32, 128, 64);
    f.fill(ops, 0x1717_0005);
    target(ops, &f, None, test(true, ALWAYS));
    let mut regs = Vec::new();
    for (k, &(sx, sy, ex, ey)) in LINES.iter().enumerate() {
        let (cx, cy) = (16 * 16 * (k as i32 % 8) + 128, 16 * 16 * (k as i32 / 8) + 128);
        regs.push(prim(LINE, if k % 2 == 0 { GOURAUD } else { 0 }));
        regs.push(rgba(colour(k as u32)));
        regs.push(xyz2(cx + sx, cy + sy, 0));
        regs.push(rgba(colour(k as u32 + 100)));
        regs.push(xyz2(cx + ex, cy + ey, 0));
    }
    ops.push(Op::Regs(regs));
    f.read(ops);
    let g = Buf::new(18, 1, CT32, 64, 32);
    let z = Buf::new(19, 1, Z32, 64, 32);
    g.fill(ops, 0x1717_0006);
    z.fill(ops, 0x1717_0007);
    target(ops, &g, Some((&z, false)), test(true, ALWAYS));
    let v = 0xFFFF_0000;
    reset_z(ops, &g, &z, v);
    let strip = [(40, 40), (300, 200), (600, 40), (200, 420), (900, 300), (100, 100), (980, 90), (500, 500), (504, 24)];
    let mut regs = vec![prim(LINE_STRIP, 0)];
    for (k, &(x, y)) in strip.iter().enumerate() {
        regs.push(rgba(colour(k as u32 * 3)));
        regs.push(xyz2(x, y, v));
    }
    // Separate lines meeting end to start, and a pair crossing at a pixel.
    regs.push(prim(LINE, 0));
    for (k, &(ax, ay, bx, by)) in [(16, 470, 260, 380), (260, 380, 420, 500), (420, 500, 16, 470), (700, 380, 1000, 500), (1000, 380, 700, 500)].iter().enumerate() {
        regs.push(rgba(colour(50 + 2 * k as u32)));
        regs.push(xyz2(ax, ay, v));
        regs.push(rgba(colour(51 + 2 * k as u32)));
        regs.push(xyz2(bx, by, v));
    }
    ops.push(Op::Regs(regs));
    g.read(ops);
    z.read(ops);
    Script { name: "s17-lines", ops: o }
}

/// Starts a cell of `s17-triangles`: GREATER against Z at V, so that each pixel takes the first primitive to draw it.
fn reset_z(ops: &mut Vec<Op>, f: &Buf, z: &Buf, v: u32) {
    ops.push(Op::Regs(vec![
        (r::FRAME_1, frame(f.page, f.bw, f.psm, 0xFFFF_FFFF)),
        (r::ZBUF_1, zbuf(z.page, z.psm, false)),
        (r::TEST_1, test(true, ALWAYS)),
        prim(SPRITE, 0),
        rgba([0, 0, 0, 0]),
        xyz2(0, 0, v - 1),
        xyz2(16 * f.w as i32, 16 * f.h as i32, v - 1),
        (r::FRAME_1, frame(f.page, f.bw, f.psm, 0)),
        (r::TEST_1, test(true, GREATER)),
    ]));
}

fn tri(regs: &mut Vec<Reg>, attributes: u64, v: [(i32, i32); 3], k: u32, z: u32) {
    regs.push(prim(TRIANGLE, attributes));
    for (i, (x, y)) in v.into_iter().enumerate() {
        regs.push(rgba(colour(k * 3 + i as u32)));
        regs.push(xyz2(x, y, z));
    }
}

/// `s17-triangles`: in 16×16 cells, each pixel kept by the first primitive that draws it (`reset_z`): right triangles
/// on centres both ways round; quads split on either diagonal, on centres and off them, both orders; level and upright
/// sides through centres on both sides; a strip, a fan, a strip with a triangle skipped by XYZ3 (§3.2.5) and a strip
/// whose shading PRMODE changes (§3.2.6); slivers, a degenerate triangle and one with no centre inside; Gouraud
/// triangles both ways round; a large triangle across the scissoring area's edges.
pub fn triangles() -> Script {
    let mut o = Vec::new();
    let ops = &mut o;
    begin(ops);
    let f = Buf::new(20, 2, CT32, 128, 64);
    let z = Buf::new(24, 2, Z32, 128, 64);
    f.fill(ops, 0x1717_0008);
    target(ops, &f, Some((&z, false)), test(true, ALWAYS));
    let v = 0x8000_0000;
    reset_z(ops, &f, &z, v);
    let cell = |k: i32| (256 * (k % 8) + 16, 256 * (k / 8) + 16);
    let mut regs = Vec::new();
    let mut k = 0u32;
    let mut put = |regs: &mut Vec<Reg>, attributes: u64, pts: [(i32, i32); 3]| {
        let (cx, cy) = cell(k as i32 / 2);
        tri(regs, attributes, pts.map(|(x, y)| (cx + x, cy + y)), k, v);
        k += 1;
    };
    // Cells 0 to 15, two triangles each.
    let pairs: [[(i32, i32); 6]; 16] = [
        [(0, 0), (160, 0), (0, 160), (176, 16), (176, 192), (16, 176)],
        [(0, 0), (0, 160), (160, 0), (176, 16), (16, 176), (176, 192)],
        [(0, 0), (192, 0), (0, 192), (192, 0), (192, 192), (0, 192)],
        [(0, 0), (192, 0), (192, 192), (0, 0), (192, 192), (0, 192)],
        [(3, 5), (189, 2), (7, 187), (189, 2), (190, 191), (7, 187)],
        [(190, 191), (7, 187), (189, 2), (3, 5), (189, 2), (7, 187)],
        [(0, 32), (192, 32), (96, 160), (0, 192), (192, 192), (96, 64)],
        [(32, 0), (32, 192), (160, 96), (192, 0), (192, 192), (64, 96)],
        [(8, 8), (184, 8), (96, 184), (8, 24), (184, 40), (8, 56)],
        [(0, 0), (200, 3), (0, 6), (2, 10), (6, 200), (10, 9)],
        [(0, 0), (100, 100), (200, 200), (4, 4), (12, 4), (4, 12)],
        [(17, 17), (31, 17), (17, 31), (40, 100), (100, 40), (180, 180)],
        [(0, 96), (96, 0), (192, 96), (0, 96), (192, 96), (96, 192)],
        [(96, 0), (192, 96), (96, 192), (96, 0), (96, 192), (0, 96)],
        [(13, 7), (181, 33), (59, 177), (181, 33), (59, 177), (190, 190)],
        [(0, 0), (192, 64), (64, 192), (192, 64), (64, 192), (192, 192)],
    ];
    for (i, p) in pairs.iter().enumerate() {
        let attributes = if i >= 14 { GOURAUD } else { 0 };
        put(&mut regs, attributes, [p[0], p[1], p[2]]);
        put(&mut regs, attributes, [p[3], p[4], p[5]]);
    }
    // Cells 16 to 31: strips, fans, XYZ3 and PRMODE.
    let at = |c: i32, x: i32, y: i32| (256 * (c % 8) + 16 + x, 256 * (c / 8) + 16 + y);
    let zig = [(0, 0), (8, 180), (60, 10), (70, 190), (130, 5), (140, 185), (192, 20)];
    regs.push(prim(TRIANGLE_STRIP, 0));
    for (i, &(x, y)) in zig.iter().enumerate() {
        let (px, py) = at(16, x, y);
        regs.push(rgba(colour(300 + i as u32)));
        regs.push(xyz2(px, py, v));
    }
    regs.push(prim(TRIANGLE_FAN, 0));
    for (i, &(x, y)) in [(96, 96), (96, 0), (190, 40), (180, 170), (90, 192), (5, 150), (10, 20), (96, 0)].iter().enumerate() {
        let (px, py) = at(17, x, y);
        regs.push(rgba(colour(320 + i as u32)));
        regs.push(xyz2(px, py, v));
    }
    regs.push(prim(TRIANGLE_STRIP, 0));
    for (i, &(x, y)) in zig.iter().enumerate() {
        let (px, py) = at(18, x, y);
        regs.push(rgba(colour(340 + i as u32)));
        regs.push(vertex(if i == 3 { r::XYZ3 } else { r::XYZ2 }, px, py, v));
    }
    regs.push((r::PRMODECONT, 0));
    regs.push(prim(TRIANGLE_STRIP, 0));
    regs.push((r::PRMODE, GOURAUD));
    for (i, &(x, y)) in zig.iter().enumerate() {
        if i == 3 {
            regs.push((r::PRMODE, 0));
        }
        if i == 5 {
            regs.push((r::PRMODE, GOURAUD));
        }
        let (px, py) = at(19, x, y);
        regs.push(rgba(colour(360 + i as u32)));
        regs.push(xyz2(px, py, v));
    }
    regs.push((r::PRMODECONT, 1));
    ops.push(Op::Regs(std::mem::take(&mut regs)));
    // Cells 20 to 23: the scissoring area narrowed, a large Gouraud triangle across it and a large flat one.
    ops.push(Op::Regs(vec![(r::SCISSOR_1, scissor(70, 120, 33, 62))]));
    let (a, b, c) = (at(20, -50, -40), at(23, 300, 100), at(20, 60, 400));
    tri(&mut regs, GOURAUD, [a, b, c], 500, v);
    tri(&mut regs, 0, [at(22, 0, 200), at(23, 400, 220), at(23, 200, -100)], 501, v);
    ops.push(Op::Regs(std::mem::take(&mut regs)));
    f.read(ops);
    Script { name: "s17-triangles", ops: o }
}

/// `s17-gouraud`: interpolation at full scale: a Gouraud triangle over a whole PSMCT32 page from colours at the ends
/// of their range, a quad of two triangles with Z interpolated into PSMZ32 from 10000000h to F0000000h and read back,
/// Gouraud lines in every direction across a page, and the same triangle into PSMCT24 and PSMCT16.
pub fn gouraud() -> Script {
    let mut o = Vec::new();
    let ops = &mut o;
    begin(ops);
    let big = |regs: &mut Vec<Reg>| {
        regs.push(prim(TRIANGLE, GOURAUD));
        for (x, y, c) in [(-5, -11, [255, 0, 17, 128]), (1030, 82, [0, 255, 99, 0]), (323, 542, [31, 62, 255, 255])] {
            regs.push(rgba(c));
            regs.push(xyz2(x, y, 0));
        }
    };
    let f = Buf::new(28, 1, CT32, 64, 32);
    f.fill(ops, 0x1717_0009);
    target(ops, &f, None, test(true, ALWAYS));
    let mut regs = Vec::new();
    big(&mut regs);
    ops.push(Op::Regs(std::mem::take(&mut regs)));
    f.read(ops);
    let g = Buf::new(29, 1, CT32, 64, 32);
    let z = Buf::new(30, 1, Z32, 64, 32);
    g.fill(ops, 0x1717_000A);
    z.fill(ops, 0x1717_000B);
    target(ops, &g, Some((&z, false)), test(true, ALWAYS));
    regs.push(prim(TRIANGLE_STRIP, GOURAUD));
    for (x, y, zv, c) in [(0, 0, 0x1000_0000, [10, 20, 30, 40]), (1024, 0, 0x7654_3210, [250, 5, 128, 7]), (0, 512, 0xA987_6543, [3, 200, 60, 255]), (1024, 512, 0xF000_0000, [129, 127, 1, 254])] {
        regs.push(rgba(c));
        regs.push(xyz2(x, y, zv));
    }
    ops.push(Op::Regs(std::mem::take(&mut regs)));
    g.read(ops);
    z.read(ops);
    let h = Buf::new(31, 1, CT32, 64, 32);
    h.fill(ops, 0x1717_000C);
    target(ops, &h, None, test(true, ALWAYS));
    regs.push(prim(LINE, GOURAUD));
    for k in 0..24 {
        let a = (k as f64) * std::f64::consts::PI / 12.0;
        let (dx, dy) = ((a.cos() * 470.0) as i32, (a.sin() * 230.0) as i32);
        regs.push(rgba(colour(k)));
        regs.push(xyz2(512 + dx / 9 + k as i32, 256 + dy / 9, 0));
        regs.push(rgba(colour(k + 40)));
        regs.push(xyz2(512 + dx, 256 + dy, 0));
    }
    ops.push(Op::Regs(std::mem::take(&mut regs)));
    h.read(ops);
    for (page, psm, seed) in [(32, CT24, 0x1717_000D), (33, CT16, 0x1717_000E)] {
        let b = Buf::new(page, 1, psm, 64, if psm == CT16 { 64 } else { 32 });
        b.fill(ops, seed);
        target(ops, &b, None, test(true, ALWAYS));
        big(&mut regs);
        ops.push(Op::Regs(std::mem::take(&mut regs)));
        b.read(ops);
    }
    Script { name: "s17-gouraud", ops: o }
}

/// `s17-formats`: every format FRAME may name, the four colour formats and the four Z formats used as a frame buffer,
/// each a page drawn with a Gouraud triangle and sprites under FBMSK 0, a mixed mask, the alpha bit alone and every bit.
pub fn formats() -> Script {
    let mut o = Vec::new();
    let ops = &mut o;
    begin(ops);
    let masks = [0, 0x0F3C_A5F0, 0x8000_0000, 0xFFFF_FFFF, 0x7FFF_FFFF];
    for (i, psm) in [CT32, CT24, CT16, CT16S, Z32, Z24, Z16, Z16S].into_iter().enumerate() {
        let tall = matches!(psm, CT16 | CT16S | Z16 | Z16S);
        let b = Buf::new(36 + i as u32, 1, psm, 64, if tall { 64 } else { 32 });
        b.fill(ops, 0x1717_0100 + i as u32);
        target(ops, &b, None, test(true, ALWAYS));
        let mut regs = vec![prim(TRIANGLE, GOURAUD)];
        for (x, y, c) in [(10, 5, [255, 128, 7, 255]), (600, 30, [3, 250, 129, 0]), (100, 16 * b.h as i32 - 20, [77, 1, 255, 130])] {
            regs.push(rgba(c));
            regs.push(xyz2(x, y, 0));
        }
        for (k, m) in masks.into_iter().enumerate() {
            let x = 640 + 72 * k as i32;
            regs.push((r::FRAME_1, frame(b.page, 1, psm, m)));
            regs.push(prim(SPRITE, 0));
            regs.push(rgba([0x5A, 0xC3, 0x96, 0x81]));
            regs.push(xyz2(x, 8, 0));
            regs.push(rgba([0xF1, 0x0E, 0x77, 0xFF ^ (k as u32 * 0x41)]));
            regs.push(xyz2(x + 64, 16 * b.h as i32 - 8, 0));
        }
        ops.push(Op::Regs(regs));
        b.read(ops);
    }
    Script { name: "s17-formats", ops: o }
}

/// The depth cases of one Z format against its frame format: each test with Z written, GEQUAL with ZMSK, and ZTE 0;
/// a Gouraud triangle whose Z spans the format's range over generated Z, then a sprite writing Z = M with ALWAYS and
/// two sprites at M and M - 1 across it under the test, which decides equality.
fn depth_cases(ops: &mut Vec<Op>, first_page: u32, fpsm: u32, zpsm: u32) {
    let tall = matches!(fpsm, CT16 | CT16S);
    let ztall = matches!(zpsm, Z16 | Z16S);
    let top: u64 = match zpsm {
        Z32 => 0xFFFF_FFFF,
        Z24 => 0xFF_FFFF,
        _ => 0xFFFF,
    };
    let m = (top / 2 + 0x1234) as u32;
    let cases: [(bool, u64, bool); 6] = [(true, NEVER, false), (true, ALWAYS, false), (true, GEQUAL, false), (true, GREATER, false), (true, GEQUAL, true), (false, ALWAYS, false)];
    for (i, (zte, ztst, zmsk)) in cases.into_iter().enumerate() {
        let f = Buf::new(first_page + 2 * i as u32, 1, fpsm, 64, if tall { 64 } else { 32 });
        let z = Buf::new(first_page + 2 * i as u32 + 1, 1, zpsm, 64, if ztall { 64 } else { 32 });
        f.fill(ops, 0x1717_0200 ^ (first_page << 8) ^ i as u32);
        z.fill(ops, 0x1717_0300 ^ (first_page << 8) ^ i as u32);
        target(ops, &f, Some((&z, zmsk)), test(zte, ztst));
        let h = 16 * f.h as i32;
        let mut regs = vec![prim(TRIANGLE, GOURAUD)];
        for (x, y, zv, c) in [(-3, -5, 0u32, [200, 10, 10, 255]), (1030, h / 3, top as u32, [10, 200, 10, 128]), (200, h + 7, (top / 3) as u32, [10, 10, 200, 0])] {
            regs.push(rgba(c));
            regs.push(xyz2(x, y, zv));
        }
        // Z = M written over the right half with ALWAYS, whatever the case's test, then the test against it.
        regs.extend([(r::ZBUF_1, zbuf(z.page, zpsm, false)), (r::TEST_1, test(true, ALWAYS)), prim(SPRITE, 0), rgba([1, 2, 3, 4]), xyz2(512, 0, m), xyz2(1024, h, m)]);
        regs.extend([(r::ZBUF_1, zbuf(z.page, zpsm, zmsk)), (r::TEST_1, test(zte, ztst))]);
        regs.extend([rgba([90, 91, 92, 93]), xyz2(400, 16, m), xyz2(1000, h / 2, m)]);
        regs.extend([rgba([150, 151, 152, 153]), xyz2(400, h / 2 + 16, m - 1), xyz2(1000, h - 16, m - 1)]);
        ops.push(Op::Regs(regs));
        f.read(ops);
        z.read(ops);
    }
}

/// `s17-depth-32`: PSMZ32 and PSMZ24 under PSMCT32, PSMZ24 under PSMCT24 (Group 1 of §2.5.4).
pub fn depth32() -> Script {
    let mut o = Vec::new();
    begin(&mut o);
    depth_cases(&mut o, 60, CT32, Z32);
    depth_cases(&mut o, 72, CT32, Z24);
    depth_cases(&mut o, 84, CT24, Z24);
    Script { name: "s17-depth-32", ops: o }
}

/// `s17-depth-16`: PSMZ16S under PSMCT32 and PSMCT16S, PSMZ16 under PSMCT16 (both groups of §2.5.4).
pub fn depth16() -> Script {
    let mut o = Vec::new();
    begin(&mut o);
    depth_cases(&mut o, 100, CT32, Z16S);
    depth_cases(&mut o, 112, CT16S, Z16S);
    depth_cases(&mut o, 124, CT16, Z16);
    Script { name: "s17-depth-16", ops: o }
}

/// `s17-zrange`: Z beyond a format's range, written with ALWAYS into PSMZ24 and PSMZ16 and then tested against, and a
/// 24-bit Z through XYZF2 with bits set above it in the register's Z field.
pub fn zrange() -> Script {
    let mut o = Vec::new();
    let ops = &mut o;
    begin(ops);
    for (i, zpsm) in [Z24, Z16].into_iter().enumerate() {
        let f = Buf::new(140 + 2 * i as u32, 1, CT32, 64, 32);
        let z = Buf::new(141 + 2 * i as u32, 1, zpsm, 64, if zpsm == Z16 { 64 } else { 32 });
        f.fill(ops, 0x1717_0400 + i as u32);
        z.fill(ops, 0x1717_0500 + i as u32);
        target(ops, &f, Some((&z, false)), test(true, ALWAYS));
        let mut regs = vec![prim(SPRITE, 0), rgba([9, 8, 7, 6]), xyz2(0, 0, 0x0123_4567), xyz2(512, 512, 0x0123_4567)];
        regs.extend([prim(TRIANGLE, GOURAUD), rgba([1, 1, 1, 1]), xyz2(520, 0, 0), rgba([2, 2, 2, 2]), xyz2(1024, 0, 0xFFFF_FFFF), rgba([3, 3, 3, 3]), xyz2(520, 512, 0x8000_0000)]);
        regs.extend([(r::TEST_1, test(true, GREATER)), prim(SPRITE, 0), rgba([200, 201, 202, 203]), xyz2(0, 256, 0x00FF_FFFF), xyz2(1024, 384, 0x00FF_FFFF)]);
        ops.push(Op::Regs(regs));
        f.read(ops);
        z.read(ops);
    }
    Script { name: "s17-zrange", ops: o }
}

/// `s17-scissor`: SCISSOR's bounds inclusive on every side, under a sprite larger than the page; SCANMSK keeping odd
/// rows, then even; a scissoring area with SCAX0 above SCAX1; and a sprite 128 pixels wide into a buffer one unit
/// wide, whose right half falls in the page below as §8.2's arrangement places it.
pub fn scissors() -> Script {
    let mut o = Vec::new();
    let ops = &mut o;
    begin(ops);
    let f = Buf::new(150, 1, CT32, 64, 32);
    f.fill(ops, 0x1717_0600);
    target(ops, &f, None, test(true, ALWAYS));
    let big = |k: u32| vec![prim(SPRITE, 0), rgba(colour(k)), xyz2(-100, -100, 0), xyz2(2000, 2000, 0)];
    ops.push(Op::Regs([vec![(r::SCISSOR_1, scissor(3, 9, 2, 6))], big(1)].concat()));
    ops.push(Op::Regs([vec![(r::SCISSOR_1, scissor(12, 30, 10, 28)), (r::SCANMSK, 2)], big(2)].concat()));
    ops.push(Op::Regs([vec![(r::SCISSOR_1, scissor(33, 50, 10, 28)), (r::SCANMSK, 3)], big(3)].concat()));
    ops.push(Op::Regs([vec![(r::SCISSOR_1, scissor(52, 63, 0, 31)), (r::SCANMSK, 1)], vec![prim(SPRITE, 0), rgba(colour(4)), xyz2(52 * 16, 0, 0), xyz2(58 * 16, 8 * 16, 0)]].concat()));
    ops.push(Op::Regs([vec![(r::SCANMSK, 0), (r::SCISSOR_1, scissor(40, 20, 0, 31))], big(5)].concat()));
    ops.push(Op::Regs(vec![(r::SCISSOR_1, scissor(0, 63, 0, 31)), prim(TRIANGLE, 0), rgba(colour(6)), xyz2(1000, 300, 0), xyz2(1100, 600, 0), xyz2(900, 600, 0)]));
    f.read(ops);
    let g = Buf::new(152, 1, CT32, 64, 64);
    g.fill(ops, 0x1717_0601);
    ops.push(Op::Regs(vec![(r::FRAME_1, frame(g.page, 1, CT32, 0)), (r::SCISSOR_1, scissor(0, 127, 0, 15)), prim(SPRITE, 0), rgba(colour(7)), xyz2(8, 8, 0), xyz2(120 * 16, 12 * 16, 0)]));
    g.read(ops);
    Script { name: "s17-scissor", ops: o }
}

/// `s17-kick`: the vertex queue and the attributes: PRIM written between a triangle's vertices empties the queue;
/// XYZ3 completing a triangle draws nothing and empties it; XYZF2's 24-bit Z and its fog field; RGBAQ written after the
/// last vertex changes nothing drawn; PRMODECONT 0, PRIM's attributes ignored for PRMODE's; context 2 by PRIM's CTXT
/// (its FRAME, SCISSOR, XYOFFSET, TEST and ZBUF), and by PRMODE in the middle of a strip; type 7; a line strip
/// continued after PRIM is written again; points through XYZ3.
pub fn kick() -> Script {
    let mut o = Vec::new();
    let ops = &mut o;
    begin(ops);
    let f = Buf::new(160, 1, CT32, 64, 32);
    let f2 = Buf::new(161, 1, CT32, 64, 32);
    let z = Buf::new(162, 1, Z32, 64, 32);
    f.fill(ops, 0x1717_0700);
    f2.fill(ops, 0x1717_0701);
    z.fill(ops, 0x1717_0702);
    target(ops, &f, Some((&z, false)), test(true, ALWAYS));
    ops.push(Op::Regs(vec![
        (r::FRAME_2, frame(f2.page, 1, CT32, 0)),
        (r::SCISSOR_2, scissor(4, 59, 2, 29)),
        (r::XYOFFSET_2, offset(OFF.0 - 5 * 16 - 3, OFF.1 + 2 * 16 + 9)),
        (r::TEST_2, test(true, ALWAYS)),
        (r::ZBUF_2, zbuf(z.page, Z32, true)),
    ]));
    let mut regs = Vec::new();
    // PRIM between vertices: only the last three make a triangle.
    regs.extend([prim(TRIANGLE, 0), rgba(colour(1)), xyz2(16, 16, 5), xyz2(200, 16, 5), prim(TRIANGLE, 0), rgba(colour(2)), xyz2(16, 100, 5), xyz2(200, 100, 5), xyz2(16, 200, 5)]);
    // XYZ3 third: nothing; the next three draw.
    regs.extend([rgba(colour(3)), xyz2(250, 16, 6), xyz2(400, 16, 6), vertex(r::XYZ3, 250, 160, 6), rgba(colour(4)), xyz2(250, 200, 6), xyz2(400, 200, 6), xyz2(400, 120, 6)]);
    // XYZF2: Z is bits 55:32, the fog field above it (the Z buffer shows the 24 bits).
    regs.extend([prim(SPRITE, 0), rgba(colour(5)), vertex(r::XYZF2, 420, 16, 0), (r::XYZF2, vertex(r::XYZF2, 600, 100, 0).1 | 0xAB12_3456u64 << 32)]);
    // RGBAQ after the last vertex: the flat colour is the last vertex's as it was kicked.
    regs.extend([prim(TRIANGLE, 0), rgba(colour(6)), xyz2(620, 16, 7), xyz2(1000, 16, 7), rgba(colour(7)), xyz2(620, 200, 7), rgba(colour(8))]);
    // PRMODECONT 0: PRIM says Gouraud and context 2, PRMODE flat and context 1.
    regs.extend([(r::PRMODECONT, 0), (r::PRMODE, 0), prim(TRIANGLE, GOURAUD | CONTEXT_2), rgba(colour(9)), xyz2(16, 260, 8), rgba(colour(10)), xyz2(300, 260, 8), rgba(colour(11)), xyz2(16, 500, 8), (r::PRMODECONT, 1)]);
    // Context 2 by PRIM.
    regs.extend([prim(TRIANGLE, GOURAUD | CONTEXT_2), rgba(colour(12)), xyz2(0, 0, 9), rgba(colour(13)), xyz2(1100, 40, 9), rgba(colour(14)), xyz2(300, 600, 9)]);
    // Context switched by PRMODE inside a strip (§3.3.2's second example).
    regs.extend([(r::PRMODECONT, 0), (r::PRMODE, 0), prim(TRIANGLE_STRIP, 0)]);
    for (i, &(x, y)) in [(620, 260), (700, 500), (760, 270), (840, 490), (900, 280), (990, 480)].iter().enumerate() {
        regs.push((r::PRMODE, if i == 3 { CONTEXT_2 } else { 0 }));
        regs.push(rgba(colour(20 + i as u32)));
        regs.push(xyz2(x, y, 10));
    }
    regs.push((r::PRMODECONT, 1));
    // Type 7 draws nothing; a line strip continued after PRIM again starts afresh.
    regs.extend([prim(7, 0), rgba(colour(30)), xyz2(330, 300, 0), xyz2(500, 300, 0), xyz2(400, 480, 0)]);
    regs.extend([prim(LINE_STRIP, 0), rgba(colour(31)), xyz2(330, 260, 0), xyz2(560, 330, 0), prim(LINE_STRIP, 0), rgba(colour(32)), xyz2(330, 500, 0), xyz2(560, 420, 0), xyz2(450, 300, 0)]);
    regs.extend([prim(POINT, 0), rgba(colour(33)), vertex(r::XYZ3, 600, 8, 0), xyz2(608, 8, 0)]);
    ops.push(Op::Regs(regs));
    f.read(ops);
    f2.read(ops);
    z.read(ops);
    Script { name: "s17-kick", ops: o }
}

pub fn all() -> Vec<Script> {
    vec![points(), sprites(), lines(), triangles(), gouraud(), formats(), depth32(), depth16(), zrange(), scissors(), kick()]
}

/// The script run on the manual's model: the expected record, 64 bits an entry.
pub fn expected(script: &Script) -> Vec<Option<u64>> {
    let mut gs = manual::Model::default();
    let mut out = Vec::new();
    for op in &script.ops {
        match op {
            Op::Regs(list) => list.iter().for_each(|&(reg, v)| gs.write(reg, v)),
            Op::Image { seed, qwords } => s16::units(*seed, *qwords).into_iter().for_each(|u| gs.write(s16::r::HWREG, u)),
            Op::Finish => {
                gs.write_privileged(s16::ga::CSR, 2);
                gs.write(s16::r::FINISH, 0);
                gs.write_privileged(s16::ga::CSR, 2);
            }
            Op::Readback(q) => {
                gs.write_privileged(s16::ga::BUSDIR, 1);
                out.extend((0..2 * q).map(|_| Some(gs.read())));
                gs.write_privileged(s16::ga::BUSDIR, 0);
            }
            other => panic!("{}: S17's programs do not use {other:?}", script.name),
        }
    }
    out
}

/// The script as an EE program graded by the model.
pub fn program(script: &Script) -> crate::s6::Program {
    s16::program_for(script, &expected(script))
}
