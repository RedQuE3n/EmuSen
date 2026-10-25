//! The PCRTC (GS User's Manual chapter 5; Sapphire_Native.md §29): the video timing (`timing`), the two rectangular
//! area read circuits (DISPFB1/2, DISPLAY1/2) and the merge circuit (PMODE, BGCOLOR), which make the picture handed to
//! the frontend. Each raster is read from the GS's local memory through `gs::local` at the start of its line, with the
//! registers as they stand then, so a change in the middle of a field shows from the next line. The picture's size is
//! fixed at each field's start from the circuits switched on; an interlaced picture holds both fields' rasters woven.
//! The feedback write (EXTBUF, EXTDATA, EXTWRITE) is not modelled (§29.6).

pub mod timing;

use crate::gs::psm::Psm;
use crate::gs::{Gs, privileged};
use timing::{Field, Mode, Timing};

/// The largest picture handed on: 1080i's width, and its frame's rasters.
pub const MAX_WIDTH: u32 = 1920;
pub const MAX_HEIGHT: u32 = 1125;

/// DISPFB's PS-GPU24 code, read by circuit 1 alone (§5.2.2).
const PS_GPU24: u32 = 0x12;

/// One read circuit's registers, decoded (DISPFB, DISPLAY; §7.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Circuit {
    pub fbp: u32,
    pub fbw: u32,
    pub psm: u32,
    pub dbx: u32,
    pub dby: u32,
    pub dx: u64,
    pub dy: u32,
    pub magh: u64,
    pub magv: u32,
    pub dw: u64,
    pub dh: u32,
}

impl Circuit {
    pub fn decode(dispfb: u64, display: u64) -> Circuit {
        Circuit {
            fbp: (dispfb & 0x1FF) as u32,
            fbw: (dispfb >> 9 & 0x3F) as u32,
            psm: (dispfb >> 15 & 0x1F) as u32,
            dbx: (dispfb >> 32 & 0x7FF) as u32,
            dby: (dispfb >> 43 & 0x7FF) as u32,
            dx: display & 0xFFF,
            dy: (display >> 12 & 0x7FF) as u32,
            magh: (display >> 23 & 0xF) + 1,
            magv: (display >> 27 & 3) as u32 + 1,
            dw: display >> 32 & 0xFFF,
            dh: (display >> 44 & 0x7FF) as u32,
        }
    }

    /// The buffer line raster `r` reads, if `r` lies in the circuit's rectangle: in FRAME mode each field's lines step
    /// the buffer by one, otherwise each raster does (§5.2.2's Interlace Read).
    fn line(&self, r: u32, frame: bool) -> Option<u32> {
        let d = r.checked_sub(self.dy).filter(|&d| d <= self.dh)?;
        let scan = if frame { d / 2 } else { d };
        Some(self.dby + scan / self.magv)
    }

    /// The colour and alpha of the buffer's pixel `x` of line `y` (§5.2.2's Read Format, §5.2.3's alpha and 16-bit
    /// colour); a format outside DISPFB's table that is not a depth format of the same width reads black with alpha 0.
    fn pixel(&self, gs: &Gs, x: u32, y: u32, second: bool) -> ([u8; 3], u8) {
        let (bp, bw) = (self.fbp * 32, self.fbw);
        let (x, y) = (x & 0x7FF, y & 0x7FF);
        let rgb = |v: u32| [v as u8, (v >> 8) as u8, (v >> 16) as u8];
        let half = |v: u32| ([((v & 31) << 3) as u8, ((v >> 5 & 31) << 3) as u8, ((v >> 10 & 31) << 3) as u8], if v & 0x8000 != 0 { 0x80 } else { 0 });
        if self.psm == PS_GPU24 {
            if second {
                return ([0; 3], 0);
            }
            let byte = |k: u32| (gs.local.read(Psm::Ct16, bp, bw, (self.dbx + k / 2) & 0x7FF, y) >> (8 * (k & 1))) as u8;
            let k = 3 * (x.wrapping_sub(self.dbx) & 0x7FF);
            return ([byte(k), byte(k + 1), byte(k + 2)], 0x80);
        }
        match Psm::from_code(self.psm) {
            Some(p @ (Psm::Ct32 | Psm::Z32)) => {
                let v = gs.local.read(p, bp, bw, x, y);
                (rgb(v), (v >> 24) as u8)
            }
            Some(p @ (Psm::Ct24 | Psm::Z24)) => (rgb(gs.local.read(p, bp, bw, x, y)), 0x80),
            Some(p @ (Psm::Ct16 | Psm::Ct16s | Psm::Z16 | Psm::Z16s)) => half(gs.local.read(p, bp, bw, x, y)),
            _ => ([0; 3], 0),
        }
    }
}

/// PMODE's fields (§7.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pmode {
    pub en1: bool,
    pub en2: bool,
    pub mmod: bool,
    pub amod: bool,
    pub slbg: bool,
    pub alp: u8,
}

impl Pmode {
    pub fn decode(v: u64) -> Pmode {
        Pmode { en1: v & 1 != 0, en2: v & 2 != 0, mmod: v & 0x20 != 0, amod: v & 0x40 != 0, slbg: v & 0x80 != 0, alp: (v >> 8) as u8 }
    }
}

/// The merge of one pixel (§5.2.3): OUT2 is circuit 2's colour inside its rectangle and BGCOLOR elsewhere; OUT1
/// blends circuit 1 over OUT2 or BGCOLOR by its doubled alpha or ALP, `(Rs·A + Rd·(FFh − A)) / FFh` to the nearest
/// (Sapphire_Disputes.md D-203); where circuit 1
/// gives nothing, OUT1 is what it would blend over.
pub fn merge(pm: &Pmode, bg: [u8; 3], one: Option<([u8; 3], u8)>, two: Option<([u8; 3], u8)>) -> [u8; 3] {
    let out2 = two.map_or(bg, |p| p.0);
    let d = if pm.slbg { bg } else { out2 };
    let Some((s, a1)) = one else { return d };
    let a = if pm.mmod { u32::from(pm.alp) } else { (2 * u32::from(a1)).min(0xFF) };
    std::array::from_fn(|i| ((u32::from(s[i]) * a + u32::from(d[i]) * (0xFF - a) + 0x7F) / 0xFF) as u8)
}

/// The picture's place on the screen: its left edge in VCK and VCK a pixel, its first raster, and its size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Geometry {
    pub x0: u64,
    pub pitch: u64,
    pub width: u32,
    pub y0: u32,
    pub height: u32,
}

/// The picture's geometry from the registers: the union of the switched-on circuits' rectangles inside the line and
/// the frame's rasters, a pixel the finest circuit's, at most `MAX_WIDTH` wide; with none on, the mode's own.
pub fn geometry(mode: &Mode, int: bool, pm: &Pmode, circuits: &[Circuit; 2]) -> Geometry {
    let rasters = mode.rasters(int);
    let on = [pm.en1, pm.en2];
    let mut x = (u64::MAX, 0);
    let mut y = (u32::MAX, 0);
    let mut pitch = u64::MAX;
    for (c, _) in circuits.iter().zip(on).filter(|p| p.1) {
        x = (x.0.min(c.dx), x.1.max((c.dx + c.dw + 1).min(mode.line)));
        y = (y.0.min(c.dy), y.1.max((c.dy + c.dh + 1).min(rasters)));
        pitch = pitch.min(c.magh);
    }
    if x.0 >= x.1 || y.0 >= y.1 {
        let (pitch, width, height) = mode.picture;
        return Geometry { x0: 0, pitch, width, y0: 0, height: height.min(rasters) };
    }
    let span = x.1 - x.0;
    let pitch = pitch.max(span.div_ceil(u64::from(MAX_WIDTH)));
    Geometry { x0: x.0, pitch, width: span.div_ceil(pitch) as u32, y0: y.0, height: (y.1 - y.0).min(MAX_HEIGHT) }
}

/// What the frontend is handed: RGBA rows, the geometry they cover, the shape on the screen, and the last field read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Picture {
    pub geometry: Geometry,
    pub aspect: (u32, u32),
    pub odd: bool,
    pub rgba: Vec<u8>,
}

impl Picture {
    /// The picture as the core ABI describes it (EmuSen_CoreAPI.md §6.6): RGBA8888 rows at the picture's own size, the
    /// television's shape as its aspect.
    pub fn frame_info(&self) -> emusen_native::core::FrameInfo {
        let g = &self.geometry;
        emusen_native::core::FrameInfo { aspect_num: self.aspect.0, aspect_den: self.aspect.1, ..emusen_native::core::FrameInfo::rgba(g.width as i32, g.height as i32) }
    }
}

pub struct Pcrtc {
    pub timing: Timing,
    pub picture: Picture,
    field: Field,
    /// The next line of the field to read, and the EE cycle it starts; none left in the field.
    line: u64,
    read_at: u64,
    /// Fields begun since the PCRTC was made.
    pub fields: u64,
}

impl Pcrtc {
    pub fn new(mode: Mode) -> Pcrtc {
        let timing = Timing::new(mode);
        let g = geometry(&mode, false, &Pmode::decode(0), &[Circuit::decode(0, 0); 2]);
        let picture = Picture { geometry: g, aspect: mode.aspect, odd: false, rgba: [0, 0, 0, 255].repeat((g.width * g.height) as usize) };
        let mut p = Pcrtc { timing, picture, field: timing.field(0), line: 0, read_at: u64::MAX, fields: 0 };
        p.schedule(0);
        p
    }

    /// The next time the PCRTC needs attention: a signal's edge (H-blank's when `want_h`) or a line to read.
    pub fn next_event(&self, t: u64, want_h: bool) -> u64 {
        self.timing.next_edge(t, want_h).min(self.read_at)
    }

    /// The signals' edges at `t`, a waiting change taken first.
    pub fn edges(&mut self, t: u64) -> timing::Edges {
        self.timing.settle(t);
        self.timing.edges(t)
    }

    /// Whether the field in progress is odd (CSR's FIELD).
    pub fn odd(&self) -> bool {
        self.field.odd
    }

    fn circuits(gs: &Gs) -> [Circuit; 2] {
        let r = |ga| gs.privileged_register(ga);
        [Circuit::decode(r(privileged::DISPFB1), r(privileged::DISPLAY1)), Circuit::decode(r(privileged::DISPFB2), r(privileged::DISPLAY2))]
    }

    /// A field's start (V-blank's rise): the picture's geometry fixed from the registers now, a new size starting black.
    pub fn begin_field(&mut self, t: u64, gs: &Gs) {
        self.field = self.timing.field(t);
        self.fields += 1;
        let (mode, int) = self.timing.mode(t);
        let g = geometry(&mode, int, &Pmode::decode(gs.privileged_register(privileged::PMODE)), &Self::circuits(gs));
        if g != self.picture.geometry || mode.aspect != self.picture.aspect {
            self.picture.geometry = g;
            self.picture.aspect = mode.aspect;
            self.picture.rgba.clear();
            self.picture.rgba.extend([0, 0, 0, 255].repeat((g.width * g.height) as usize));
        }
        self.schedule(t);
    }

    /// The field's first line whose raster lies in the picture, and its time.
    fn schedule(&mut self, t: u64) {
        let g = self.picture.geometry;
        let (mode, int) = self.timing.mode(t);
        self.line = if mode.interlaced(int) { u64::from(g.y0.saturating_sub(self.field.odd as u32).div_ceil(2)) } else { u64::from(g.y0) };
        self.time_line(t);
    }

    fn time_line(&mut self, t: u64) {
        let g = self.picture.geometry;
        let r = self.timing.raster(t, &self.field, self.line);
        self.read_at = if r >= g.y0 + g.height { u64::MAX } else { self.timing.line_start(self.field.start, self.line).unwrap_or(u64::MAX) };
    }

    /// Reads every line due by `t` from the GS as it is now.
    pub fn read_due(&mut self, t: u64, gs: &Gs) {
        while self.read_at <= t {
            let at = self.read_at;
            let r = self.timing.raster(at, &self.field, self.line);
            self.read_raster(at, r, gs);
            self.line += 1;
            self.time_line(at);
        }
    }

    /// Raster `r`'s row of the picture, read and merged with the registers as they stand.
    fn read_raster(&mut self, t: u64, r: u32, gs: &Gs) {
        let g = self.picture.geometry;
        let Some(row) = r.checked_sub(g.y0).filter(|&row| row < g.height) else { return };
        let (mode, int) = self.timing.mode(t);
        let frame = mode.interlaced(int) && gs.privileged_register(privileged::SMODE2) & 2 != 0;
        let pm = Pmode::decode(gs.privileged_register(privileged::PMODE));
        let bgv = gs.privileged_register(privileged::BGCOLOR);
        let bg = [bgv as u8, (bgv >> 8) as u8, (bgv >> 16) as u8];
        let circuits = Self::circuits(gs);
        let lines = [circuits[0].line(r, frame).filter(|_| pm.en1), circuits[1].line(r, frame).filter(|_| pm.en2)];
        self.picture.odd = self.field.odd;
        let at = (row * g.width * 4) as usize;
        let out = &mut self.picture.rgba[at..at + (g.width * 4) as usize];
        for (x, px) in out.chunks_exact_mut(4).enumerate() {
            let v = g.x0 + x as u64 * g.pitch;
            let read = |n: usize| {
                let c = &circuits[n];
                let y = lines[n]?;
                (v >= c.dx && v <= c.dx + c.dw).then(|| c.pixel(gs, c.dbx + ((v - c.dx) / c.magh) as u32, y, n == 1))
            };
            let rgb = merge(&pm, bg, read(0), read(1));
            px.copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
        }
    }

    /// A privileged register written at `t`: SMODE2's INT changes the timing from the next field (§29.3). Whether the
    /// timing changed, so that the board can tell the IOP.
    pub fn written(&mut self, t: u64, gs: &Gs) -> bool {
        let (mode, was) = self.timing.target();
        let int = gs.privileged_register(privileged::SMODE2) & 1 != 0;
        self.timing.change(t, mode, int);
        int != was
    }

    /// Selects the video mode from the next field, as the kernel's mode setting does through SMODE1 and the
    /// synchronisation registers, whose fields no document gives (§29.3).
    pub fn set_mode(&mut self, t: u64, mode: Mode) -> bool {
        let (old, int) = self.timing.target();
        let changed = old != mode;
        self.timing.change(t, mode, int);
        changed
    }
}
