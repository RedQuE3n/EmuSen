//! The drawing kick (GS User's Manual §3.2; ps2tek "GS Primitives", "GS Vertex Attributes"): PRIM and the attributes
//! PRMODECONT selects from PRIM or PRMODE, the vertex queue the XYZ registers advance, and the drawing kick that hands
//! a primitive to the rasteriser (`raster`) with the environment of its context (`Environment`), whose pixels then go
//! through the pipeline (`pixel`). Sapphire_Native.md §27.

use super::psm::Psm;
use super::texture::Sampler;
use super::{Gs, raster, reg};

/// A vertex (§3.2.3's figure 3-2): its coordinates in sixteenths of a pixel, primitive coordinates in the queue and
/// window coordinates once the drawing kick has subtracted XYOFFSET (signed, since the offset may exceed them); Z; and
/// RGBAQ, ST, UV and the fog coefficient as they stood at its vertex kick.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Vertex {
    pub x: i32,
    pub y: i32,
    pub z: u32,
    pub rgbaq: u64,
    pub st: u64,
    pub uv: u64,
    pub f: u8,
}

impl Vertex {
    /// R, G, B and A, each 8 bits (RGBAQ's page).
    pub fn rgba(&self) -> [u32; 4] {
        let c = self.rgbaq as u32;
        [c & 0xFF, c >> 8 & 0xFF, c >> 16 & 0xFF, c >> 24]
    }
}

/// PRIM's primitive types (§7.1, PRIM bits 2:0).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Point,
    Line,
    LineStrip,
    Triangle,
    TriangleStrip,
    TriangleFan,
    Sprite,
    /// 111, "Specification Prohibited": vertices advance the queue and nothing is drawn [A].
    Prohibited,
}

impl Kind {
    pub const fn from_prim(prim: u64) -> Kind {
        match prim & 7 {
            0 => Kind::Point,
            1 => Kind::Line,
            2 => Kind::LineStrip,
            3 => Kind::Triangle,
            4 => Kind::TriangleStrip,
            5 => Kind::TriangleFan,
            6 => Kind::Sprite,
            _ => Kind::Prohibited,
        }
    }

    /// The vertices in the queue at a drawing kick (§3.2.4).
    pub const fn vertices(self) -> usize {
        match self {
            Kind::Point | Kind::Prohibited => 1,
            Kind::Line | Kind::LineStrip | Kind::Sprite => 2,
            Kind::Triangle | Kind::TriangleStrip | Kind::TriangleFan => 3,
        }
    }
}

/// The attributes IIP, TME, FGE, ABE, AA1, FST, CTXT and FIX, as bits 3 to 10 of PRIM or PRMODE.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Attributes(pub u64);

impl Attributes {
    pub const fn gouraud(self) -> bool {
        self.0 >> 3 & 1 != 0
    }
    pub const fn textured(self) -> bool {
        self.0 >> 4 & 1 != 0
    }
    pub const fn fogged(self) -> bool {
        self.0 >> 5 & 1 != 0
    }
    pub const fn blended(self) -> bool {
        self.0 >> 6 & 1 != 0
    }
    pub const fn antialiased(self) -> bool {
        self.0 >> 7 & 1 != 0
    }
    pub const fn uv(self) -> bool {
        self.0 >> 8 & 1 != 0
    }
    /// The context: 0 for the registers named _1, 1 for those named _2 (§3.3.2).
    pub const fn context(self) -> usize {
        (self.0 >> 9 & 1) as usize
    }
    pub const fn fixed(self) -> bool {
        self.0 >> 10 & 1 != 0
    }
}

/// The vertex queue (§3.2.3): up to three vertices, emptied by a write to PRIM.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Queue {
    pub v: [Vertex; 3],
    pub n: usize,
}

impl Queue {
    pub fn reset(&mut self) {
        self.n = 0;
    }

    /// What a drawing kick leaves, or a vertex kick that fills the queue without one (§3.2.4, §3.2.5): a strip keeps
    /// its last vertex or two, a fan its first and last, the others nothing.
    fn advance(&mut self, kind: Kind) {
        match kind {
            Kind::LineStrip => {
                self.v[0] = self.v[1];
                self.n = 1;
            }
            Kind::TriangleStrip => {
                self.v[0] = self.v[1];
                self.v[1] = self.v[2];
                self.n = 2;
            }
            Kind::TriangleFan => {
                self.v[1] = self.v[2];
                self.n = 2;
            }
            _ => self.n = 0,
        }
    }
}

/// The frame buffer as FRAME gives it (§7.1): format (any of §2.4.5's, the Z formats included), base pointer in
/// blocks (FBP counts 2,048 words, 32 blocks), width in units of 64 pixels, and FBMSK.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Frame {
    pub psm: Option<Psm>,
    pub bp: u32,
    pub bw: u32,
    pub mask: u32,
}

/// FRAME's PSM: the colour and Z formats its page lists; the reserved texture codes draw nothing [A].
pub const fn frame_format(code: u32) -> Option<Psm> {
    match Psm::from_code(code) {
        Some(p @ (Psm::Ct32 | Psm::Ct24 | Psm::Ct16 | Psm::Ct16s | Psm::Z32 | Psm::Z24 | Psm::Z16 | Psm::Z16s)) if code == p.code() => Some(p),
        _ => None,
    }
}

/// The Z buffer as ZBUF gives it: its format (PSM's four bits are the Z codes' low bits), base pointer and ZMSK; its
/// width is the frame buffer's (§2.5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Depth {
    pub psm: Option<Psm>,
    pub bp: u32,
    pub masked: bool,
}

/// The drawing environment of a primitive's context at its drawing kick, and its attributes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Environment {
    pub attributes: Attributes,
    pub frame: Frame,
    pub depth: Depth,
    /// SCISSOR's SCAX0, SCAX1, SCAY0, SCAY1, inclusive, in window coordinates (§3.7.1).
    pub scissor: (i32, i32, i32, i32),
    pub test: u64,
    pub scanmsk: u32,
    /// The context's texture registers when TME is set (S18).
    pub texture: Option<Sampler>,
    /// FOGCOL's colour when FGE is set (S18).
    pub fog: Option<[u32; 3]>,
}

impl Environment {
    pub fn read(gs: &Gs, attributes: Attributes) -> Environment {
        let c = attributes.context() as u8;
        let frame = gs.regs[(reg::FRAME_1 + c) as usize];
        let zbuf = gs.regs[(reg::ZBUF_1 + c) as usize];
        let s = gs.regs[(reg::SCISSOR_1 + c) as usize];
        let f = |shift: u32, bits: u32| (frame >> shift) as u32 & ((1 << bits) - 1);
        Environment {
            attributes,
            frame: Frame {
                psm: frame_format(f(24, 6)),
                bp: f(0, 9) * 32,
                bw: f(16, 6),
                mask: (frame >> 32) as u32,
            },
            depth: Depth { psm: Psm::from_code(0x30 | (zbuf >> 24) as u32 & 0xF), bp: (zbuf as u32 & 0x1FF) * 32, masked: zbuf >> 32 & 1 != 0 },
            scissor: ((s & 0x7FF) as i32, (s >> 16 & 0x7FF) as i32, (s >> 32 & 0x7FF) as i32, (s >> 48 & 0x7FF) as i32),
            test: gs.regs[(reg::TEST_1 + c) as usize],
            scanmsk: gs.regs[reg::SCANMSK as usize] as u32 & 3,
            texture: attributes.textured().then(|| {
                let r = |a: u8| gs.regs[(a + c) as usize];
                Sampler::new([r(reg::TEX0_1), r(reg::TEX1_1), r(reg::CLAMP_1), r(reg::MIPTBP1_1), r(reg::MIPTBP2_1), gs.regs[reg::TEXA as usize]], attributes.uv())
            }),
            fog: attributes.fogged().then(|| {
                let f = gs.regs[reg::FOGCOL as usize];
                [f as u32 & 0xFF, (f >> 8) as u32 & 0xFF, (f >> 16) as u32 & 0xFF]
            }),
        }
    }
}

impl Gs {
    /// The attributes in force: PRIM's when PRMODECONT's AC is 1, PRMODE's when it is 0 (§7.1, PRMODECONT).
    pub fn attributes(&self) -> Attributes {
        let from = if self.regs[reg::PRMODECONT as usize] & 1 != 0 { reg::PRIM } else { reg::PRMODE };
        Attributes(self.regs[from as usize] & 0x7F8)
    }

    /// A vertex kick by XYZ2, XYZF2, XYZ3 or XYZF3 (§3.2.3): the vertex information is copied into the queue, and once
    /// the queue holds the primitive's vertices a drawing kick follows for XYZ2 and XYZF2, never for XYZ3 and XYZF3
    /// (§3.2.5), the queue advancing either way.
    pub(super) fn vertex_kick(&mut self, r: u8, data: u64) {
        let fogged = matches!(r, reg::XYZF2 | reg::XYZF3);
        if fogged {
            self.regs[reg::FOG as usize] = data & 0xFF00_0000_0000_0000;
        }
        let v = Vertex {
            x: (data & 0xFFFF) as i32,
            y: (data >> 16 & 0xFFFF) as i32,
            z: if fogged { (data >> 32) as u32 & 0xFF_FFFF } else { (data >> 32) as u32 },
            rgbaq: self.regs[reg::RGBAQ as usize],
            st: self.regs[reg::ST as usize],
            uv: self.regs[reg::UV as usize],
            f: (self.regs[reg::FOG as usize] >> 56) as u8,
        };
        let kind = Kind::from_prim(self.regs[reg::PRIM as usize]);
        let q = &mut self.queue;
        q.v[q.n] = v;
        q.n += 1;
        if q.n < kind.vertices() {
            return;
        }
        if matches!(r, reg::XYZ2 | reg::XYZF2) {
            // XYOFFSET of the drawing's context, at the drawing kick (Sapphire_Disputes.md D-162).
            let env = Environment::read(self, self.attributes());
            let offset = self.regs[(reg::XYOFFSET_1 + env.attributes.context() as u8) as usize];
            let (ox, oy) = ((offset & 0xFFFF) as i32, (offset >> 32 & 0xFFFF) as i32);
            let v = self.queue.v.map(|v| Vertex { x: v.x - ox, y: v.y - oy, ..v });
            let (local, clut) = (&mut self.local, &self.clut);
            match kind {
                Kind::Point => raster::point(&env, local, clut, &v[0]),
                Kind::Line | Kind::LineStrip => raster::line(&env, local, clut, &v[0], &v[1]),
                Kind::Triangle | Kind::TriangleStrip | Kind::TriangleFan => raster::triangle(&env, local, clut, [&v[0], &v[1], &v[2]]),
                Kind::Sprite => raster::sprite(&env, local, clut, &v[0], &v[1]),
                Kind::Prohibited => {}
            }
        }
        self.queue.advance(kind);
    }
}
