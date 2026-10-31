//! The device path's plan (Sapphire_Gs.md §4): each primitive as a record of 32 words beside its context's environment,
//! the records grouped into stretches, one dispatch of the walk each, and each stretch's records binned into tiles of
//! 16 by 8 pixels in drawing order. A stretch ends before a record whose texels meet what the stretch has drawn, whose
//! stores meet another surface's pixels or the stretch's texels, or when a buffer fills; within it, a tile's records are
//! applied in order by one workgroup, so each pixel sees the one-thread path's order. No GPU here: the plan is plain data.

use super::super::draw::{Environment, Kind, Vertex};
use super::super::local::{self, Units};
use super::super::psm::Psm;
use super::super::raster;
use super::super::reg;

pub const RECORD_WORDS: usize = 32;
pub const ENV_WORDS: usize = 32;
pub const CLUT_WORDS: usize = 256;
pub const TILE_W: i32 = 16;
pub const TILE_H: i32 = 8;
const TILES_X: usize = 2048 / TILE_W as usize;
const TILES: usize = TILES_X * (2048 / TILE_H as usize);

/// What one submission holds at most; the plan asks to be submitted when any would be passed.
pub const MAX_RECORDS: usize = 1 << 16;
pub const MAX_ENVS: usize = 1 << 14;
pub const MAX_ENTRIES: usize = 1 << 22;
pub const MAX_TILES: usize = 1 << 18;
pub const MAX_CLUTS: usize = 1 << 12;
pub const MAX_STAGED_PAGES: usize = 1 << 10;
pub const PAGE_WORDS: usize = 2048;

/// A command of a submission, in order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    /// The walk over tiles `first` to `first + count - 1`.
    Walk { first: u32, count: u32, class: u32, ordered: bool },
    /// Pages of local memory copied to the snapshot that the next stretch's records read their texels from.
    Snapshot { page: u32, pages: u32 },
    /// Pages of the staging buffer copied into local memory: `pages` from staged page `from` to memory page `to`.
    Upload { from: u32, to: u32, pages: u32 },
}

/// Why a stretch ended, for the counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Close {
    Texels,
    Surfaces,
    Overwrite,
    Upload,
    Full,
    Sync,
    Snapshot,
}

/// What the plan counted, cumulatively.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub records: u64,
    pub empty: u64,
    /// Primitives handed back to the processor: they read where they draw (with exact feedback), their frame and Z meet,
    /// or their pixels alias between tiles.
    pub fallback: [u64; 3],
    /// Records whose texels meet the blocks they draw, applied one after another by one workgroup (Sapphire_Gs.md §4.4),
    /// and their stretches.
    pub snapshot: [u64; 2],
    pub stretches: u64,
    pub closes: [u64; 7],
    pub pairs: u64,
    pub tiles: u64,
    pub envs: u64,
    pub cluts: u64,
    pub staged_pages: u64,
    pub submissions: u64,
}

/// One submission's data, ready for the device.
#[derive(Default)]
pub struct Batch {
    pub records: Vec<u32>,
    pub envs: Vec<u32>,
    pub entries: Vec<u32>,
    pub tiles: Vec<[u32; 4]>,
    pub cluts: Vec<u32>,
    pub staged: Vec<u32>,
    pub ops: Vec<Op>,
}

impl Batch {
    fn with_capacity() -> Batch {
        Batch {
            records: Vec::with_capacity(MAX_RECORDS * RECORD_WORDS),
            envs: Vec::with_capacity(MAX_ENVS * ENV_WORDS),
            entries: Vec::with_capacity(MAX_ENTRIES),
            tiles: Vec::with_capacity(MAX_TILES),
            cluts: Vec::with_capacity(MAX_CLUTS * CLUT_WORDS),
            staged: Vec::with_capacity(MAX_STAGED_PAGES * PAGE_WORDS),
            ops: Vec::with_capacity(4096),
        }
    }

    pub fn clear(&mut self) {
        self.records.clear();
        self.envs.clear();
        self.entries.clear();
        self.tiles.clear();
        self.cluts.clear();
        self.staged.clear();
        self.ops.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }
}

/// A buffer a stretch draws into: pixel (x, y) is the same word in every record naming it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Surface {
    psm: Psm,
    bp: u32,
    bw: u32,
}

/// What `draw` decided.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Drawn {
    /// Nothing covered.
    Empty,
    /// Recorded; the lanes it may change.
    Recorded(Units),
    /// The processor must draw it (Sapphire_Gs.md §4.4); the lanes it touches, read or written, and those it may change.
    Fallback(Units, Units),
}

pub struct Planner {
    pub batch: Batch,
    pub counts: Counts,
    /// The open stretch: its surfaces with the lanes each touches, every surface lane, the texels' lanes and its pairs.
    surfaces: Vec<(Surface, Units)>,
    touched: Units,
    texels: Units,
    pairs: Vec<u32>,
    per_tile: Box<[u32]>,
    used: Vec<u16>,
    /// The environment last recorded and its index in this batch.
    env: Option<([u32; ENV_WORDS], u32)>,
    /// The CLUT slot in this batch, or none since the CLUT changed or the batch began.
    clut: Option<u32>,
    /// When the open stretch is ordered: its records, applied one after another by one workgroup.
    ordered: Option<Vec<u32>>,
    /// When the open stretch reads its texels from a snapshot: the environment its records share.
    snapshot: Option<[u32; ENV_WORDS]>,
    /// Self-reading records applied one after another by one workgroup (`SAPPHIRE_GS_ORDERED`) rather than read from a
    /// snapshot of their run, the default (Sapphire_Gs.md §10.4).
    pub ordered_mode: bool,
    /// The open stretch's class: every kind and feature some record of it has (`class`).
    class: u32,
    /// Exact mode, for proof: records that read where they draw go to the processor, the reference's order, rather than
    /// to a snapshot, and the walk takes the texture coordinates in the reference's exact arithmetic.
    pub exact_feedback: bool,
    /// The last sampler's whole reach.
    reach: Option<(super::super::texture::Sampler, Units)>,
}

impl Default for Planner {
    fn default() -> Planner {
        Planner::new()
    }
}

/// The environment's words (Sapphire_Gs.md §4.2): parsed fields where the walk needs them, raw registers where it decodes them.
fn pack_env(regs: &[u64], env: &Environment) -> [u32; ENV_WORDS] {
    let c = env.attributes.context() as u8;
    let r = |a: u8| regs[(a + c) as usize];
    let mut e = [0u32; ENV_WORDS];
    let f = &env.frame;
    e[0] = f.psm.map_or(0, |p| p.code() | 0x40) | f.bw << 8;
    e[1] = f.bp;
    e[2] = f.mask;
    e[3] = env.depth.psm.map_or(0, |p| p.code() | 0x40) | u32::from(env.depth.masked) << 8;
    e[4] = env.depth.bp;
    let (sx0, sx1, sy0, sy1) = env.scissor;
    e[5] = sx0 as u32 | (sx1 as u32) << 16;
    e[6] = sy0 as u32 | (sy1 as u32) << 16;
    e[7] = env.test as u32;
    let b = &env.blend;
    e[8] = (env.attributes.0 >> 3) as u32 & 0xFF
        | u32::from(b.pabe) << 8
        | u32::from(env.dither.is_some()) << 9
        | u32::from(env.colclamp) << 10
        | u32::from(env.fba) << 11
        | env.scanmsk << 12
        | u32::from(b.antialias) << 14;
    e[9] = b.alpha as u32;
    e[10] = (b.alpha >> 32) as u32;
    let d = env.dither.unwrap_or(0);
    e[11] = d as u32;
    e[12] = (d >> 32) as u32;
    let fc = env.fog.unwrap_or([0; 3]);
    e[13] = fc[0] | fc[1] << 8 | fc[2] << 16;
    if env.texture.is_some() {
        let raw = [r(reg::TEX0_1), r(reg::TEX1_1), r(reg::CLAMP_1), r(reg::MIPTBP1_1), r(reg::MIPTBP2_1), regs[reg::TEXA as usize]];
        for (k, v) in raw.iter().enumerate() {
            e[14 + 2 * k] = *v as u32;
            e[15 + 2 * k] = (*v >> 32) as u32;
        }
    }
    e
}

/// The PSMs a CLUT serves.
fn indexed(tex0: u64) -> bool {
    matches!(Psm::from_code((tex0 >> 20) as u32 & 0x3F), Some(Psm::T8 | Psm::T8h | Psm::T4 | Psm::T4hl | Psm::T4hh))
}

fn kind_code(kind: Kind) -> u32 {
    match kind {
        Kind::Point => 0,
        Kind::Line | Kind::LineStrip => 1,
        Kind::Triangle | Kind::TriangleStrip | Kind::TriangleFan => 2,
        Kind::Sprite => 3,
        Kind::Prohibited => 7,
    }
}

impl Planner {
    pub fn new() -> Planner {
        Planner {
            batch: Batch::with_capacity(),
            counts: Counts::default(),
            surfaces: Vec::with_capacity(8),
            touched: Units::default(),
            texels: Units::default(),
            pairs: Vec::with_capacity(1 << 20),
            per_tile: vec![0; TILES].into_boxed_slice(),
            used: Vec::with_capacity(TILES),
            env: None,
            clut: None,
            ordered: None,
            snapshot: None,
            ordered_mode: false,
            class: 0,
            exact_feedback: false,
            reach: None,
        }
    }

    /// The CLUT's buffer changed: the next indexed record takes a new slot.
    pub fn clut_changed(&mut self) {
        self.clut = None;
    }

    /// Whether the batch has room for another record, stretch and CLUT, or a staged page; the device submits otherwise.
    pub fn full(&self) -> bool {
        let b = &self.batch;
        b.records.len() / RECORD_WORDS + 1 >= MAX_RECORDS
            || b.envs.len() / ENV_WORDS + 1 >= MAX_ENVS
            || b.cluts.len() / CLUT_WORDS + 1 >= MAX_CLUTS
            || b.entries.len() + self.pairs.len() + 2 * TILES >= MAX_ENTRIES
            || b.tiles.len() + self.used.len() + TILES >= MAX_TILES
            || b.staged.len() / PAGE_WORDS + local::PAGES >= MAX_STAGED_PAGES
    }

    /// A primitive at its drawing kick, with the GS's registers, its environment, its vertices in window coordinates and
    /// the CLUT's buffer; the caller has submitted first if `full`.
    pub fn draw(&mut self, regs: &[u64], kind: Kind, env: &Environment, v: &[Vertex; 3], clut: &[u16; 512]) -> Drawn {
        let Some((x0, x1, y0, y1)) = raster::bounds(kind, env, v) else {
            self.counts.empty += 1;
            return Drawn::Empty;
        };
        let fpsm = env.frame.psm.expect("bounds holds a frame format");
        let (xs, ys) = ((x0 as u32, x1 as u32), (y0 as u32, y1 as u32));
        let frame = Surface { psm: fpsm, bp: env.frame.bp, bw: env.frame.bw };
        let fu = local::units(fpsm, frame.bp, frame.bw, xs, ys);
        let depth = env.depth.psm.filter(|_| env.test >> 16 & 1 != 0).map(|p| Surface { psm: p, bp: env.depth.bp, bw: env.frame.bw });
        let zu = depth.map(|z| local::units(z.psm, z.bp, z.bw, xs, ys));
        let mut writes = fu;
        let mut all = fu;
        // Pixels past the buffer's width, or a span of more pages than the memory has, share words between tiles.
        let bw = env.frame.bw.max(1);
        if x1 as u32 >= env.frame.bw * 64 || (y1 as u32 / 32 + 1) * bw >= local::PAGES as u32 {
            if let Some(u) = zu {
                all.or(&u);
            }
            self.counts.fallback[2] += 1;
            return Drawn::Fallback(all, writes_all(fu, zu, env.depth.masked));
        }
        if let (Some(z), Some(u)) = (depth, zu) {
            all.or(&u);
            if !env.depth.masked {
                writes.or(&u);
            }
            if z != frame && u.meets(&fu) {
                self.counts.fallback[1] += 1;
                return Drawn::Fallback(all, writes);
            }
        }
        let texels = match &env.texture {
            None => Units::default(),
            // The sampler's whole reach, kept for the next record with the same registers; the vertices' only where it meets.
            Some(t) => {
                let full = match &self.reach {
                    Some((s, u)) if s == t => *u,
                    _ => {
                        let u = t.units();
                        self.reach = Some((*t, u));
                        u
                    }
                };
                let q = |x: &Vertex| (x.rgbaq >> 32) as u32;
                if !full.meets(&all) && !full.meets(&self.touched) {
                    full
                } else {
                    match kind {
                    Kind::Sprite => t.reach(&[(&v[0], q(&v[1])), (&v[1], q(&v[1]))]),
                    Kind::Point => t.reach(&[(&v[0], q(&v[0]))]),
                    Kind::Triangle | Kind::TriangleStrip | Kind::TriangleFan => t.reach(&[(&v[0], q(&v[0])), (&v[1], q(&v[1])), (&v[2], q(&v[2]))]),
                    _ => full,
                    }
                }
            }
        };
        // Texels in the pages it draws: in the same blocks, the record reads them from a snapshot taken before its stretch.
        let snapshot = texels.meets(&writes) && self.self_reading(kind, env, v, xs, ys, depth.filter(|_| !env.depth.masked));
        if snapshot && self.exact_feedback {
            let mut u = all;
            u.or(&texels);
            self.counts.fallback[0] += 1;
            return Drawn::Fallback(u, writes);
        }
        let e = pack_env(regs, env);
        if snapshot && self.ordered_mode {
            if self.ordered.is_none() {
                self.close(Close::Snapshot);
                self.ordered = Some(Vec::with_capacity(4096));
                self.counts.snapshot[1] += 1;
            }
            self.counts.snapshot[0] += 1;
        } else if snapshot {
            if self.snapshot != Some(e) {
                self.close(Close::Snapshot);
                let full = env.texture.as_ref().expect("textured").units();
                let mut pages: Vec<u32> = full.each().map(|u| (u / 2) as u32).collect();
                pages.dedup();
                let mut i = 0;
                while i < pages.len() {
                    let mut j = i + 1;
                    while j < pages.len() && pages[j] == pages[j - 1] + 1 {
                        j += 1;
                    }
                    self.batch.ops.push(Op::Snapshot { page: pages[i], pages: (j - i) as u32 });
                    i = j;
                }
                self.snapshot = Some(e);
                self.counts.snapshot[1] += 1;
            }
            self.counts.snapshot[0] += 1;
        } else if self.ordered.is_some() || self.snapshot.is_some() {
            self.close(Close::Snapshot);
        }
        // The stretch ends before a record that would race one of it across tiles.
        let close = if snapshot {
            None
        } else if texels.meets(&self.touched) {
            Some(Close::Texels)
        } else if self.surfaces.iter().any(|(s, u)| (*s != frame && u.meets(&fu)) || zu.is_some_and(|z| Some(*s) != depth && u.meets(&z))) {
            Some(Close::Surfaces)
        } else if writes.meets(&self.texels) {
            Some(Close::Overwrite)
        } else {
            None
        };
        if let Some(why) = close {
            self.close(why);
        }
        self.add_surface(frame, fu);
        if let (Some(z), Some(u)) = (depth, zu) {
            self.add_surface(z, u);
        }
        if !snapshot {
            self.texels.or(&texels);
        }
        // The environment, recorded again only when it differs from the last.
        let env_index = match self.env {
            Some((last, i)) if last == e => i,
            _ => {
                let i = (self.batch.envs.len() / ENV_WORDS) as u32;
                self.batch.envs.extend_from_slice(&e);
                self.env = Some((e, i));
                self.counts.envs += 1;
                i
            }
        };
        let tex0 = regs[(reg::TEX0_1 + env.attributes.context() as u8) as usize];
        let clut_slot = if env.texture.is_some() && indexed(tex0) {
            match self.clut {
                Some(s) => s,
                None => {
                    let s = (self.batch.cluts.len() / CLUT_WORDS) as u32;
                    self.batch.cluts.extend(clut.as_chunks::<2>().0.iter().map(|p| u32::from(p[0]) | u32::from(p[1]) << 16));
                    self.clut = Some(s);
                    self.counts.cluts += 1;
                    s
                }
            }
        } else {
            0
        };
        let index = (self.batch.records.len() / RECORD_WORDS) as u32;
        let rec = &mut self.batch.records;
        self.class |= class(kind, env, regs);
        // An ordered record carries its bounds: y0 and y1 above its flag in word 3, x0 and x1 in word 31.
        // Word 3: bit 0 a self-reading record, bit 1 one whose texels come from the snapshot, then its y0 and y1.
        let ordered = u32::from(snapshot) | u32::from(snapshot && !self.ordered_mode) << 1 | (y0 as u32) << 8 | (y1 as u32) << 20;
        rec.extend_from_slice(&[kind_code(kind), env_index, clut_slot, ordered]);
        for p in v {
            rec.extend_from_slice(&[p.x as u32, p.y as u32, p.z, p.rgbaq as u32, (p.rgbaq >> 32) as u32, p.st as u32, (p.st >> 32) as u32, p.uv as u32, u32::from(p.f)]);
        }
        rec.push(x0 as u32 | (x1 as u32) << 16);
        self.counts.records += 1;
        match self.ordered.as_mut() {
            Some(list) => list.push(index),
            None => self.bin(kind, v, index, (x0, x1, y0, y1)),
        }
        Drawn::Recorded(writes)
    }

    /// Whether the texels a record can read share a block with the pixels it may change.
    fn self_reading(&self, kind: Kind, env: &Environment, v: &[Vertex; 3], xs: (u32, u32), ys: (u32, u32), depth: Option<Surface>) -> bool {
        const LIMIT: u32 = 8192;
        let t = env.texture.as_ref().expect("textured");
        let q = |x: &Vertex| (x.rgbaq >> 32) as u32;
        let range = match kind {
            Kind::Sprite => t.range(&[(&v[0], q(&v[1])), (&v[1], q(&v[1]))]),
            Kind::Point => t.range(&[(&v[0], q(&v[0]))]),
            Kind::Triangle | Kind::TriangleStrip | Kind::TriangleFan => t.range(&[(&v[0], q(&v[0])), (&v[1], q(&v[1])), (&v[2], q(&v[2]))]),
            _ => None,
        };
        let (mut drawn, mut read) = (local::BlockSet::default(), local::BlockSet::default());
        let f = &env.frame;
        let mut known = drawn.add_rect(f.psm.expect("a frame"), f.bp, f.bw, xs, ys, LIMIT);
        if let Some(z) = depth {
            known &= drawn.add_rect(z.psm, z.bp, z.bw, xs, ys, LIMIT);
        }
        t.rects(range, |psm, bp, bw, x, y| known &= read.add_rect(psm, bp, bw, x, y, LIMIT));
        !known || drawn.meets(&read)
    }

    fn add_surface(&mut self, s: Surface, u: Units) {
        match self.surfaces.iter_mut().find(|(t, _)| *t == s) {
            Some((_, t)) => t.or(&u),
            None => self.surfaces.push((s, u)),
        }
        self.touched.or(&u);
    }

    /// The record's tiles: those of its bounds, less, for a triangle, those every pixel centre of which one edge excludes.
    fn bin(&mut self, kind: Kind, v: &[Vertex; 3], index: u32, (x0, x1, y0, y1): (i32, i32, i32, i32)) {
        let (tx0, tx1, ty0, ty1) = (x0 / TILE_W, x1 / TILE_W, y0 / TILE_H, y1 / TILE_H);
        let triangle = matches!(kind, Kind::Triangle | Kind::TriangleStrip | Kind::TriangleFan) && (tx1 - tx0 + 1) * (ty1 - ty0 + 1) > 2;
        let edges = triangle.then(|| raster::edges(v)).flatten();
        for ty in ty0..=ty1 {
            for tx in tx0..=tx1 {
                if let Some(e) = &edges {
                    let (px0, py0) = (16 * i64::from((tx * TILE_W).max(x0)), 16 * i64::from((ty * TILE_H).max(y0)));
                    let (px1, py1) = (16 * i64::from((tx * TILE_W + TILE_W - 1).min(x1)), 16 * i64::from((ty * TILE_H + TILE_H - 1).min(y1)));
                    if e.iter().any(|&(a, b, c, bias)| [(px0, py0), (px1, py0), (px0, py1), (px1, py1)].iter().all(|&(x, y)| a * x + b * y + c + bias <= 0)) {
                        continue;
                    }
                }
                let t = ty as usize * TILES_X + tx as usize;
                if self.per_tile[t] == 0 {
                    self.used.push(t as u16);
                }
                self.per_tile[t] += 1;
                self.pairs.push((t as u32) << 16 | (index & 0xFFFF));
            }
        }
    }

    /// The open stretch planned as a walk: its tiles placed, each with its records in drawing order.
    pub fn close(&mut self, why: Close) {
        let ordered = self.ordered.take();
        self.snapshot = None;
        let class = self.class;
        self.class = 0;
        if let Some(list) = ordered {
            self.surfaces.clear();
            self.touched = Units::default();
            self.texels = Units::default();
            if !list.is_empty() {
                self.counts.stretches += 1;
                self.counts.closes[why as usize] += 1;
                let first = self.batch.tiles.len() as u32;
                self.batch.tiles.push([0, 0, self.batch.entries.len() as u32, list.len() as u32]);
                self.batch.entries.extend_from_slice(&list);
                self.batch.ops.push(Op::Walk { first, count: 1, class, ordered: true });
            }
            return;
        }
        self.surfaces.clear();
        self.touched = Units::default();
        self.texels = Units::default();
        if self.pairs.is_empty() {
            return;
        }
        self.counts.stretches += 1;
        self.counts.closes[why as usize] += 1;
        self.counts.pairs += self.pairs.len() as u64;
        self.counts.tiles += self.used.len() as u64;
        let first_tile = self.batch.tiles.len() as u32;
        let base = self.batch.entries.len() as u32;
        let mut at = base;
        for &t in &self.used {
            let n = self.per_tile[t as usize];
            let (tx, ty) = (t as u32 % TILES_X as u32, t as u32 / TILES_X as u32);
            self.batch.tiles.push([tx, ty, at, n]);
            self.per_tile[t as usize] = at;
            at += n;
        }
        self.batch.entries.resize(at as usize, 0);
        for &p in &self.pairs {
            let t = (p >> 16) as usize;
            let slot = self.per_tile[t];
            self.batch.entries[slot as usize] = p & 0xFFFF;
            self.per_tile[t] = slot + 1;
        }
        for &t in &self.used {
            self.per_tile[t as usize] = 0;
        }
        self.batch.ops.push(Op::Walk { first: first_tile, count: self.used.len() as u32, class, ordered: false });
        self.used.clear();
        self.pairs.clear();
    }

    /// Pages the processor wrote, copied in at this point of the order: the stretch ends first if they meet it.
    pub fn upload(&mut self, pages: &[usize], memory: &[u32]) {
        let mut u = Units::default();
        for &p in pages {
            u.add_page(p, 3);
        }
        if u.meets(&self.touched) || u.meets(&self.texels) || self.ordered.is_some() || self.snapshot.is_some() {
            self.close(Close::Upload);
        }
        let mut i = 0;
        while i < pages.len() {
            let mut j = i + 1;
            while j < pages.len() && pages[j] == pages[j - 1] + 1 {
                j += 1;
            }
            let from = (self.batch.staged.len() / PAGE_WORDS) as u32;
            for &p in &pages[i..j] {
                self.batch.staged.extend_from_slice(&memory[p * PAGE_WORDS..(p + 1) * PAGE_WORDS]);
            }
            self.batch.ops.push(Op::Upload { from, to: pages[i] as u32, pages: (j - i) as u32 });
            self.counts.staged_pages += (j - i) as u64;
            i = j;
        }
    }

    /// The batch taken for submission: the stretch closed and the next batch begun empty.
    pub fn take(&mut self, why: Close) -> &mut Batch {
        self.close(why);
        self.env = None;
        self.clut = None;
        self.counts.submissions += 1;
        &mut self.batch
    }
}

/// The lanes a record may change: its frame's, and its Z's unless ZMSK.
fn writes_all(fu: Units, zu: Option<Units>, masked: bool) -> Units {
    let mut w = fu;
    if let Some(u) = zu.filter(|_| !masked) {
        w.or(&u);
    }
    w
}

/// The walk's class bits (its specialisation constants 1 to 8, `shaders/walk.comp`): triangles, sprites, lines or points,
/// a texture, STQ coordinates, a LOD a pixel, fog, antialiasing.
pub const CLASS_ALL: u32 = 0xFF;

fn class(kind: Kind, env: &Environment, regs: &[u64]) -> u32 {
    let mut c = match kind {
        Kind::Triangle | Kind::TriangleStrip | Kind::TriangleFan => 1,
        Kind::Sprite => 2,
        _ => 4,
    };
    if env.texture.is_some() {
        c |= 8;
        if !env.attributes.uv() {
            c |= 16;
            let tex1 = regs[(reg::TEX1_1 + env.attributes.context() as u8) as usize];
            let (lcm, mmag, mmin) = (tex1 & 1 != 0, tex1 >> 5 & 1 != 0, (tex1 >> 6 & 7) as u32);
            let lod_matters = !((mmin < 2 || mmin > 5) && (mmin & 1 == 1) == mmag);
            if !lcm && lod_matters {
                c |= 32;
            }
        }
    }
    if env.fog.is_some() {
        c |= 64;
    }
    if env.blend.antialias {
        c |= 128;
    }
    c
}
