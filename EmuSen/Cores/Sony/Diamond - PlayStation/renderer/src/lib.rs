//! Diamond's renderer (stage E3): the PlayStation's picture drawn on a Vulkan compute device from the GPU's command
//! stream, beside the software rasteriser, which keeps VRAM. A `Layer`: told each command, each word that continues
//! one and each display line, it keeps its own copy of VRAM in step and reads its own picture from it. Nothing it makes
//! reaches the machine (Diamond_Enhancements.md §14 on). Paired with the precision layer it draws a polygon whose
//! vertices are precise between the pixels, its texture in perspective, and keeps a block in the state (§22); such a
//! polygon is filled where the console's own covers a pixel it does not, so that none shows through beside it (§33).

pub mod blocks;
pub mod device;
pub mod record;

use blackmoon_gte::Observer;
use diamond::gpu::Gpu;
use diamond::gpu::decode::{self, Drawing};
use diamond::gpu::raster::{self, Vertex};
use diamond::gpu::texture::Texture;
use diamond::gpu::vram::{Mask, Rect};
use diamond::layer::{Command, Layer, Part, Shown};
use emusen_gpu::GpuDevice;
use emusen_native::{StateReader, StateWriter};
use precision::{Draw, Fraction};

use blocks::{Blocks, TILES, Written};
use device::{Back, CopyStep, Device, ENTRIES, LINES, LOADS, MAX_SCALE, PICTURE_WORDS, PIXELS, RECORDS, SLOTS, Step, TILE_RECORDS, Used, VRAM_WORDS};
use record::{Made, Record, Span, WORDS};

/// The tag of the renderer's block, written when it draws precisely (Diamond_Enhancements.md §24.4).
pub const TAG: [u8; 4] = *b"RNDR";

/// What the renderer did, for its tools and its record.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub records: u64,
    pub stretches: u64,
    pub copies: u64,
    pub submissions: u64,
    pub loads: u64,
    pub lines: u64,
    /// Records whose texels lie in VRAM they draw themselves, whose order across pixels the walk does not keep.
    pub feedback: u64,
    /// Precise triangles drawn, those of them in perspective, and polygons given precise vertices but drawn on the
    /// integer path because a gradient did not fit 32 bits.
    pub precise: u64,
    pub perspective: u64,
    pub integer: u64,
    /// Why stretches ended: a palette load, a copy, a transfer's row, a display line, a record's texels, a record's
    /// pixels over texels read.
    pub ends: [u64; 6],
}

/// How the renderer draws: its scale, the internal resolution, 1 to `MAX_SCALE` (Diamond_Enhancements.md §39); true
/// colour (§40); and which textures it filters (§41).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Options {
    pub scale: u32,
    pub colour: bool,
    pub filter: Filter,
}

/// Texture filtering: none, the console's; textured polygons but those drawn a texel a pixel and unrotated, and not
/// sprites; or every textured primitive.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Filter {
    #[default]
    Off,
    Polygons,
    All,
}

impl Default for Options {
    fn default() -> Options {
        Options { scale: 1, colour: false, filter: Filter::Off }
    }
}

impl Options {
    /// The options' bits the device's programs read.
    fn bits(&self) -> u32 {
        if self.colour { device::COLOUR } else { 0 }
    }
}

/// What the renderer keeps across a state when its picture is not the machine's: its mirror a halfword a sample and,
/// paired with the precision layer, the mark beside each (§33.3), its picture at the largest size at its scale with its
/// own size, and the palette its CLUT slot holds; and whether a state's block is to seed it (§42.2).
struct Kept {
    mirror: Vec<u16>,
    marks: Option<Vec<u16>>,
    /// True colour's plane, a halfword a sample for its nine bits.
    fine: Option<Vec<u16>>,
    picture: Vec<u8>,
    palette: Vec<u16>,
    size: [u32; 2],
    restored: bool,
}

/// A CPU-to-VRAM transfer the GPU is taking: its rectangle and mask, the pixels in, the open record in this stretch
/// that holds the latest of them, and the row last marked written.
#[derive(Clone, Copy, Debug)]
struct Transfer {
    rect: Rect,
    mask: Mask,
    done: u32,
    record: Option<usize>,
    row: Option<u32>,
}

/// The stretch being planned: what it has written and read as texels, its records' tiles in order, where its palette
/// loads and display lines begin in the staged arrays, and whether it starts by clearing the picture.
struct Stretch {
    written: Written,
    read: Blocks,
    pairs: Vec<(u16, u32)>,
    loads_from: usize,
    lines_from: usize,
    clear: bool,
    /// Each picture row's line in this stretch, by the stretch's serial number.
    serial: u32,
    rows: Vec<(u32, usize)>,
}

pub struct Renderer {
    device: Device,
    steps: Vec<Step>,
    used: Used,
    stretch: Stretch,
    counts_per_tile: Vec<u32>,
    touched: Vec<u16>,
    polyline: Option<(u32, Vertex)>,
    transfer: Option<Transfer>,
    slot: u32,
    /// How many of the slot's entries its load wrote: the others hold whatever the slot held before, and are never read.
    slot_entries: u32,
    next_slot: u32,
    width: u32,
    height: u32,
    counts: Counts,
    kept: Option<Kept>,
    /// The picture slot, with its size, the host is handed now, and the one the next `frame_end` hands (§38.1).
    handed: (usize, u32, u32),
    ahead: (usize, u32, u32),
    options: Options,
}

impl Observer for Renderer {}

impl Renderer {
    /// A renderer on the device `name` names a part of (`EMUSEN_GPU_DEVICE` when none), or the reason there is none.
    pub fn new(name: Option<&str>) -> Result<Renderer, String> {
        Renderer::with(name, Options::default())
    }

    /// `new`, drawing by `options`.
    pub fn with(name: Option<&str>, options: Options) -> Result<Renderer, String> {
        let options = Options { scale: options.scale.clamp(1, MAX_SCALE), ..options };
        let gpu = GpuDevice::try_create(name)?;
        Ok(Renderer {
            device: Device::new(gpu, options.scale, options.bits())?,
            steps: Vec::with_capacity(256),
            used: Used::default(),
            stretch: Stretch { written: Written::default(), read: Blocks::default(), pairs: Vec::with_capacity(1 << 16), loads_from: 0, lines_from: 0, clear: false, serial: 1, rows: vec![(0, 0); 576] },
            counts_per_tile: vec![0; TILES],
            touched: Vec::with_capacity(TILES),
            polyline: None,
            transfer: None,
            slot: 0,
            slot_entries: 256,
            next_slot: 1,
            width: 256,
            height: 240,
            counts: Counts::default(),
            kept: None,
            handed: (0, 0, 0),
            ahead: (0, 0, 0),
            options,
        })
    }

    /// The renderer as an enhancement attaches it: keeping a block in the state, since what it draws is no longer what
    /// the machine's VRAM gives (§22.5, §42.2); with `marks` when paired with the precision layer, whose triangles write
    /// them.
    pub fn keeping(mut self, marks: bool) -> Renderer {
        self.keep(marks);
        self
    }

    fn keep(&mut self, marks: bool) {
        let samples = VRAM_WORDS * (self.options.scale * self.options.scale) as usize;
        let picture = PICTURE_WORDS * 4 * (self.options.scale * self.options.scale) as usize;
        self.kept = Some(Kept { mirror: vec![0; samples], marks: marks.then(|| vec![0; samples]), fine: self.options.colour.then(|| vec![0; samples]), picture: vec![0; picture], palette: vec![0; 256], size: [256, 240], restored: false });
    }

    pub fn options(&self) -> Options {
        self.options
    }

    /// The renderer drawing by other options from the next seed on: its buffers made again at the new scale, what it
    /// drew dropped. Detached first and attached again after, so that the machine seeds it (§39.6).
    pub fn configure(&mut self, options: Options) -> Result<(), String> {
        let options = Options { scale: options.scale.clamp(1, MAX_SCALE), ..options };
        if (options.scale, options.bits()) != (self.options.scale, self.options.bits()) {
            self.steps.clear();
            self.used = Used::default();
            self.device.rebuild(options.scale, options.bits())?;
        }
        self.options = options;
        if let Some(marks) = self.kept.as_ref().map(|k| k.marks.is_some()) {
            self.keep(marks);
        }
        Ok(())
    }

    /// The device's report, `"{name}, Vulkan {major}.{minor}"`.
    pub fn report(&self) -> String {
        self.device.gpu.report()
    }

    /// Whether the device is one the processor implements (llvmpipe), which draws the same picture slowly.
    pub fn software(&self) -> bool {
        self.device.gpu.software()
    }

    pub fn counts(&self) -> Counts {
        self.counts
    }

    /// The picture's size: the display's at the scale.
    fn size(&self) -> (u32, u32) {
        (self.width * self.options.scale, self.height * self.options.scale)
    }

    /// The picture as it stands, RGBA at its own size, once the device has drawn everything it was told.
    pub fn picture(&mut self) -> (&[u8], u32, u32) {
        let (w, h) = self.size();
        let words = (w * h) as usize;
        self.submit(Back::Picture(words));
        (&self.device.held().back.slice::<u8>()[..words * 4], w, h)
    }

    /// The frame's end, as a host shows the renderer's picture: everything planned goes to the device as its pending
    /// submission, which copies the picture into a slot, and the host is handed the slot of the frame before; the
    /// machine waits only if the device has not finished that frame yet (Diamond_Enhancements.md §38.1).
    pub fn frame_end(&mut self) {
        let slot = self.ahead.0 ^ 1;
        let (w, h) = self.size();
        self.close();
        self.device.submit_pending(&self.steps, self.used, Back::Slot(slot, (w * h) as usize));
        self.submitted();
        self.handed = self.ahead;
        self.ahead = (slot, w, h);
    }

    /// The picture a host is handed, RGBA at its own size: the frame before the last `frame_end`'s, or the seed's.
    pub fn handed(&self) -> (&[u8], u32, u32) {
        let (slot, w, h) = self.handed;
        (&self.device.held().slots[slot].slice::<u8>()[..(w * h * 4) as usize], w, h)
    }

    /// The renderer's VRAM, a halfword a pixel as the GPU's `Vram` holds them, once everything told is drawn; at a scale
    /// the first sample of each pixel's block, the console's point.
    pub fn vram(&mut self, out: &mut [u16]) {
        let n = self.options.scale as usize;
        let samples = self.samples();
        for (i, o) in out.iter_mut().enumerate().take(VRAM_WORDS) {
            *o = samples[(i / 1024) * n * 1024 * n + (i % 1024) * n] as u16;
        }
    }

    /// True colour's plane, a word a sample with its nine bits, once everything told is drawn; empty without it.
    pub fn fine(&mut self) -> &[u32] {
        if !self.options.colour {
            return &[];
        }
        self.submit(Back::Fine);
        let n = (self.options.scale * self.options.scale) as usize;
        &self.device.held().back.slice::<u32>()[..VRAM_WORDS * n]
    }

    /// Every sample of the mirror, `1024 N` by `512 N` words, the console's halfword low and the mark high, once
    /// everything told is drawn.
    pub fn samples(&mut self) -> &[u32] {
        self.submit(Back::Vram);
        let n = (self.options.scale * self.options.scale) as usize;
        &self.device.held().back.slice::<u32>()[..VRAM_WORDS * n]
    }

    // --- The stream ---

    fn command(&mut self, w: &[u32], d: &Drawing, precise: Option<&[Fraction; 4]>) {
        self.end_transfer_record();
        self.polyline = None;
        self.transfer = None;
        let op = w[0] >> 24;
        match op >> 5 {
            1 => self.polygon(w, d, precise),
            2 => {
                let (a, b, shaded) = decode::line(w, d.env[4]);
                if op & 8 != 0 {
                    self.polyline = Some((w[0], b));
                }
                self.line(w[0], a, b, shaded, d);
            }
            3 => {
                let r = decode::rectangle(w, d.env[4], d.flips());
                let texture = r.textured.then(|| d.texture(r.clut));
                let slot = texture.map_or(0, |t| self.palette(&t, d));
                let paint = d.paint(r.blended, texture, r.raw, false);
                self.add(self.filtered(record::rectangle(r.sprite, &paint, slot), false));
            }
            4 => self.copy(Rect::copy_params(w[1], w[3]), Rect::copy_params(w[2], w[3]), d.mask()),
            5 => self.transfer = Some(Transfer { rect: Rect::copy_params(w[1], w[2]), mask: d.mask(), done: 0, record: None, row: None }),
            _ if op == 0x02 => self.add(record::fill(Rect::fill_params(w[1], w[2]), w[0], d.skip_parity)),
            _ => {}
        }
    }

    fn polygon(&mut self, w: &[u32], d: &Drawing, precise: Option<&[Fraction; 4]>) {
        let p = decode::polygon(w, d.env[4]);
        let d = if p.textured { d.with_page(p.page) } else { *d };
        let texture = p.textured.then(|| d.texture(p.clut));
        let slot = texture.map_or(0, |t| self.palette(&t, &d));
        let modulated = texture.is_some() && !p.raw;
        let paint = d.paint(p.blended, texture, p.raw && texture.is_some(), modulated || (p.shaded && texture.is_none()));
        let tex = texture.is_some();
        if let Some(f) = precise {
            let made = |i: [usize; 3]| record::precise_triangle(raster::fold(i.map(|k| p.v[k])), i.map(|k| f[k]), p.shaded, tex, &paint, slot);
            let made = [made([0, 1, 2]), if p.quad { made([1, 2, 3]) } else { Made::Nothing }];
            if made.iter().all(|m| !matches!(m, Made::Integer)) {
                for m in made {
                    if let Made::Drawn(r) = m {
                        self.counts.precise += 1;
                        self.counts.perspective += (r.words[0] as u32 & record::PERSPECTIVE != 0) as u64;
                        self.add(self.filtered(Some(r), true));
                    }
                }
                return;
            }
            self.counts.integer += 1;
        }
        self.add(self.filtered(record::triangle(raster::fold([p.v[0], p.v[1], p.v[2]]), p.shaded, tex, &paint, slot), true));
        if p.quad {
            self.add(self.filtered(record::triangle(raster::fold([p.v[1], p.v[2], p.v[3]]), p.shaded, tex, &paint, slot), true));
        }
    }

    /// A textured record marked to be filtered when the options ask it of its kind: every one with Everything; with 3D
    /// polygons a polygon's, but not one drawn a texel a pixel and unrotated, nor a sprite's (§41).
    fn filtered(&self, record: Option<Record>, polygon: bool) -> Option<Record> {
        let mut r = record?;
        let textured = r.words[0] as u32 & record::TEXTURED != 0;
        let on = match self.options.filter {
            Filter::Off => false,
            Filter::All => true,
            Filter::Polygons => polygon && (r.words[0] as u32 & 0xF == record::PRECISE || !record::texel_a_pixel(&r)),
        };
        if textured && on {
            r.words[0] |= record::FILTERED as i32;
        }
        Some(r)
    }

    fn line(&mut self, command: u32, a: Vertex, b: Vertex, shaded: bool, d: &Drawing) {
        let paint = d.paint(command >> 24 & 2 != 0, None, false, true);
        let [a, b] = raster::fold([a, b]);
        self.add(record::line(a, b, shaded, &paint));
    }

    fn polyline_vertex(&mut self, w: &[u32], d: &Drawing) {
        let Some((command, last)) = self.polyline else { return };
        let b = decode::place(d.env[4], last, decode::vertex(d.env[4], w[1], w[0], 0));
        self.line(command, last, b, command >> 24 & 0x10 != 0, d);
        self.polyline = Some((command, b));
    }

    /// The palette slot a 4- or 8-bit texture reads: a new one, loaded at the start of its stretch, exactly when the GPU's
    /// CLUT cache loads; the one before, while the cache keeps its palette.
    fn palette(&mut self, t: &Texture, d: &Drawing) -> u32 {
        if !d.loads_clut(t) {
            return self.slot;
        }
        let span = record::clut_span(t);
        if self.stretch.written.meets(span) || self.used.loads - self.stretch.loads_from >= SLOTS - 2 {
            self.counts.ends[0] += 1;
            self.close();
        }
        if self.used.loads == LOADS {
            self.flush();
        }
        let slot = self.next_slot;
        self.next_slot = (self.next_slot + 1) % SLOTS as u32;
        let l = self.used.loads;
        self.device.staged().loads.slice_mut::<u32>()[l * 4..l * 4 + 4].copy_from_slice(&[slot, t.clut_x, t.clut_y, t.entries()]);
        self.used.loads += 1;
        self.counts.loads += 1;
        (self.slot, self.slot_entries) = (slot, t.entries());
        slot
    }

    fn copy(&mut self, src: Rect, dst: Rect, mask: Mask) {
        self.counts.ends[1] += 1;
        self.close();
        let dd = (dst.y.wrapping_sub(src.y)) & 511;
        let chunk = if dd >= 1 && dd < src.h { dd } else { src.h };
        let mut row = 0;
        while row < src.h {
            let rows = chunk.min(src.h - row);
            let step = CopyStep { write: 0, sx: src.x, sy: src.y, dx: dst.x, dy: dst.y, width: src.w, row, rows, mask: (mask.check as u32) << 1 | (mask.set != 0) as u32, scale: 1, options: 0 };
            self.steps.push(Step::Copy(step));
            self.steps.push(Step::Copy(CopyStep { write: 1, ..step }));
            row += rows;
        }
        self.counts.copies += 1;
        if self.steps.len() > 4096 {
            self.flush();
        }
    }

    fn data(&mut self, word: u32) {
        let Some(mut x) = self.transfer else { return };
        for half in 0..2 {
            if x.done >= x.rect.pixels() {
                break;
            }
            let row = x.done / x.rect.w;
            if x.row != Some(row) {
                let span = Span::sized(x.rect.x, x.rect.y + row, x.rect.w, 1);
                if self.stretch.read.meets(span) {
                    self.counts.ends[2] += 1;
                    self.transfer = Some(x);
                    self.close();
                    x = self.transfer.expect("kept");
                }
                self.stretch.written.mark(span, None);
                x.row = Some(row);
            }
            if self.used.pixels == PIXELS || self.used.records == RECORDS {
                self.transfer = Some(x);
                self.flush();
                x = self.transfer.expect("kept");
            }
            let r = match x.record {
                Some(r) => r,
                None => {
                    let r = self.used.records;
                    self.put(&record::transfer(x.rect, x.done, x.done, self.used.pixels as u32, x.mask));
                    x.record = Some(r);
                    r
                }
            };
            let p = self.used.pixels;
            let pixels = self.device.staged().pixels.slice_mut::<u32>();
            let value = word >> (16 * half) & 0xFFFF;
            pixels[p / 2] = if p % 2 == 0 { value } else { pixels[p / 2] & 0xFFFF | value << 16 };
            self.used.pixels += 1;
            x.done += 1;
            self.device.staged().records.slice_mut::<i32>()[r * WORDS + 15] = x.done as i32;
        }
        self.transfer = Some(x);
    }

    fn shown(&mut self, s: Shown) {
        match s {
            Shown::Size { width, height } => {
                (self.width, self.height) = (width, height);
                self.stretch.clear = true;
                self.used.lines = self.stretch.lines_from;
                self.stretch.serial += 1;
            }
            Shown::Line { row, x, y, width, deep, blanked } => {
                if !blanked {
                    let n = if deep { (width * 3).div_ceil(2) + 1 } else { width };
                    if self.stretch.written.meets_row(Span::sized(x, y, n, 1)) {
                        self.counts.ends[3] += 1;
                        self.close();
                    }
                }
                let line = [row, x, y, width, deep as u32, blanked as u32, 0, 0];
                let at = match self.stretch.rows[row as usize] {
                    (serial, at) if serial == self.stretch.serial => at,
                    _ => {
                        if self.used.lines == LINES {
                            self.flush();
                        }
                        let at = self.used.lines;
                        self.used.lines += 1;
                        self.stretch.rows[row as usize] = (self.stretch.serial, at);
                        at
                    }
                };
                self.device.staged().lines.slice_mut::<u32>()[at * 8..at * 8 + 8].copy_from_slice(&line);
                self.counts.lines += 1;
            }
        }
    }

    // --- Stretches ---

    /// A record drawn after everything before it: the stretch ends first if the record reads texels drawn in it or
    /// draws over texels read in it.
    fn add(&mut self, record: Option<Record>) {
        let Some(r) = record else { return };
        self.end_transfer_record();
        let texels = r.texels.is_some_and(|t| self.stretch.written.meets(t));
        if texels || self.stretch.read.meets(r.writes) {
            self.counts.ends[if texels { 4 } else { 5 }] += 1;
            self.close();
        }
        let mut tiles = 0;
        blocks::tiles(r.writes, |_| tiles += 1);
        if self.used.records == RECORDS || self.used.entries + self.stretch.pairs.len() + tiles > ENTRIES || self.used.tiles + TILES > TILE_RECORDS {
            self.flush();
        }
        if let Some(t) = r.texels {
            let mut own = Blocks::default();
            own.mark(r.writes);
            if own.meets(t) {
                self.counts.feedback += 1;
            }
            self.stretch.read.mark(t);
        }
        let flags = r.words[0] as u32;
        let rows = (flags & record::SKIP != 0).then(|| (flags >> 19 & 1) ^ 1);
        self.stretch.written.mark(r.writes, rows);
        self.put(&r);
    }

    /// The record staged at the next index, its tiles listed in the stretch.
    fn put(&mut self, r: &Record) {
        let i = self.used.records;
        self.device.staged().records.slice_mut::<i32>()[i * WORDS..(i + 1) * WORDS].copy_from_slice(&r.words);
        self.used.records += 1;
        self.counts.records += 1;
        if r.words[0] as u32 & 0xF != record::TRANSFER {
            let pairs = &mut self.stretch.pairs;
            blocks::tiles(r.writes, |t| pairs.push((t as u16, i as u32)));
        }
    }

    /// The open transfer's record listed in its tiles, now that no more of its pixels go into it.
    fn end_transfer_record(&mut self) {
        let Some(mut x) = self.transfer else { return };
        let Some(r) = x.record.take() else { return };
        let words = &self.device.staged().records.slice::<i32>()[r * WORDS..(r + 1) * WORDS];
        let (i0, i1) = (words[14] as u32, words[15] as u32);
        let span = record::transfer(x.rect, i0, i1, 0, x.mask).writes;
        let pairs = &mut self.stretch.pairs;
        blocks::tiles(span, |t| pairs.push((t as u16, r as u32)));
        x.row = None;
        self.transfer = Some(x);
    }

    /// The stretch planned as a step: its tiles counted, placed in order and staged.
    fn close(&mut self) {
        self.end_transfer_record();
        let s = &mut self.stretch;
        let (loads, lines) = ((s.loads_from as u32, (self.used.loads - s.loads_from) as u32), (s.lines_from as u32, (self.used.lines - s.lines_from) as u32));
        let first_tile = self.used.tiles;
        if !s.pairs.is_empty() {
            let staged = self.device.staged();
            let (entries, tiles) = (staged.entries.slice_mut::<u32>(), staged.tiles.slice_mut::<u32>());
            for &(t, _) in &s.pairs {
                if self.counts_per_tile[t as usize] == 0 {
                    self.touched.push(t);
                }
                self.counts_per_tile[t as usize] += 1;
            }
            let mut cursor = self.used.entries as u32;
            for (n, &t) in self.touched.iter().enumerate() {
                let count = self.counts_per_tile[t as usize];
                let at = (first_tile + n) * 4;
                tiles[at..at + 4].copy_from_slice(&[t as u32, cursor, count, 0]);
                self.counts_per_tile[t as usize] = cursor;
                cursor += count;
            }
            for &(t, r) in &s.pairs {
                let at = &mut self.counts_per_tile[t as usize];
                entries[*at as usize] = r;
                *at += 1;
            }
            for &t in &self.touched {
                self.counts_per_tile[t as usize] = 0;
            }
            self.used.entries = cursor as usize;
            self.used.tiles += self.touched.len();
        }
        let tiles = (first_tile as u32, self.touched.len() as u32);
        self.touched.clear();
        if s.clear || loads.1 > 0 || lines.1 > 0 || tiles.1 > 0 {
            self.steps.push(Step::Stretch { clear: s.clear, loads, lines, tiles });
            self.counts.stretches += 1;
        }
        s.written.clear();
        s.read.clear();
        s.pairs.clear();
        s.clear = false;
        s.loads_from = self.used.loads;
        s.lines_from = self.used.lines;
        s.serial += 1;
    }

    /// Everything planned submitted and waited for, and what `back` asks read back.
    fn submit(&mut self, back: Back) {
        self.close();
        self.device.submit(&self.steps, self.used, back);
        self.submitted();
    }

    /// Everything planned submitted without waiting, when an array is full in the middle of a frame.
    fn flush(&mut self) {
        self.close();
        self.device.submit_pending(&self.steps, self.used, Back::Nothing);
        self.submitted();
    }

    fn submitted(&mut self) {
        self.counts.submissions += 1;
        self.steps.clear();
        self.used = Used::default();
        self.stretch.loads_from = 0;
        self.stretch.lines_from = 0;
    }
}

impl Draw for Renderer {
    fn executed_with(&mut self, c: &Command, precise: Option<&[Fraction; 4]>) {
        match c.part {
            Part::Whole => self.command(c.words, c.drawing, precise),
            Part::Vertex => self.polyline_vertex(c.words, c.drawing),
            Part::Data => self.data(c.words[0]),
        }
    }
}

impl Layer for Renderer {
    fn executed(&mut self, c: &Command) {
        self.executed_with(c, None);
    }

    fn tags(&self) -> Vec<[u8; 4]> {
        if self.kept.is_some() { vec![TAG] } else { Vec::new() }
    }

    /// Everything planned drawn, and the mirror, the slot's palette and the picture read back to where `save` reads them.
    fn settle(&mut self) {
        if self.kept.is_none() {
            return;
        }
        let samples = self.samples();
        let mirror: Vec<u16> = samples.iter().map(|&w| w as u16).collect();
        let marks: Vec<u16> = samples.iter().map(|&w| (w >> 16) as u16).collect();
        let fine: Option<Vec<u16>> = self.options.colour.then(|| {
            self.submit(Back::Fine);
            self.device.held().back.slice::<u32>()[..mirror.len()].iter().map(|&w| w as u16).collect()
        });
        self.submit(Back::Palette(self.slot));
        let entries = self.slot_entries as usize;
        let palette: Vec<u16> = self.device.held().back.slice::<u32>()[..256].iter().enumerate().map(|(i, &w)| if i < entries { w as u16 } else { 0 }).collect();
        let (width, height) = (self.width, self.height);
        let picture = self.picture().0.to_vec();
        let n = picture.len();
        let kept = self.kept.as_mut().expect("kept");
        kept.mirror.copy_from_slice(&mirror);
        if let Some(m) = kept.marks.as_mut() {
            m.copy_from_slice(&marks);
        }
        if let (Some(k), Some(f)) = (kept.fine.as_mut(), fine) {
            k.copy_from_slice(&f);
        }
        kept.palette.copy_from_slice(&palette);
        kept.picture[..n].copy_from_slice(&picture);
        kept.picture[n..].fill(0);
        kept.size = [width, height];
    }

    fn save(&self, _tag: [u8; 4], w: &mut StateWriter) {
        let Some(k) = &self.kept else { return };
        w.u32s("Size", &k.size);
        w.u16s("Mirror", &k.mirror);
        w.bytes("Picture", &k.picture);
        w.u16s("Palette", &k.palette);
        if let Some(m) = &k.marks {
            w.u16s("Marks", m);
        }
        if let Some(f) = &k.fine {
            w.u16s("Fine", f);
        }
    }

    fn load(&mut self, tag: [u8; 4], block: &[u8]) {
        let Some(k) = self.kept.as_mut() else { return };
        let mut r = StateReader::new(block);
        let mut size = [0u32; 2];
        let read = (|| {
            r.u32s(&mut size)?;
            r.u16s(&mut k.mirror)?;
            r.bytes(&mut k.picture)?;
            r.u16s(&mut k.palette)?;
            // A state of stage E4's has no marks: none are taken, and its first frame fills as it finds.
            match k.marks.as_mut() {
                Some(m) if r.position() == block.len() => m.fill(0),
                Some(m) => r.u16s(m)?,
                None => {}
            }
            match k.fine.as_mut() {
                Some(f) => r.u16s(f),
                None => Ok(()),
            }
        })();
        if read.is_err() || r.position() != block.len() || size[0] * size[1] > PICTURE_WORDS as u32 {
            return self.clear(tag);
        }
        k.size = size;
        k.restored = true;
    }

    fn clear(&mut self, _tag: [u8; 4]) {
        if let Some(k) = self.kept.as_mut() {
            k.restored = false;
        }
    }

    fn shown(&mut self, s: Shown) {
        Renderer::shown(self, s);
    }

    /// The renderer watches the GPU alone, so the machine keeps the plain frame for it (§38.3).
    fn moves(&self) -> bool {
        false
    }

    /// The GPU's VRAM, picture, CLUT cache and the transfer or polyline it has open, taken as the renderer's own, or in
    /// their place the mirror, picture and palette a state's block gave; what was planned before is dropped, being of
    /// another machine.
    fn seed(&mut self, gpu: &Gpu) {
        self.steps.clear();
        self.used = Used::default();
        let s = &mut self.stretch;
        (s.loads_from, s.lines_from, s.clear) = (0, 0, false);
        s.written.clear();
        s.read.clear();
        s.pairs.clear();
        s.serial += 1;
        let restored = self.kept.as_mut().filter(|k| k.restored).map(|k| {
            k.restored = false;
            &*k
        });
        self.device.gpu.wait_for_pending();
        let n = self.options.scale as usize;
        let held = self.device.held();
        let (staged, back, slot) = (self.device.staged().records.slice_mut::<u32>(), held.back.slice_mut::<u32>(), held.slots[0].slice_mut::<u32>());
        match restored {
            Some(k) => {
                (self.width, self.height) = (k.size[0], k.size[1]);
                for (i, (o, &h)) in back.iter_mut().zip(k.mirror.iter()).enumerate() {
                    *o = h as u32 | k.marks.as_ref().map_or(0, |m| (m[i] as u32) << 16);
                }
                for (o, p) in slot.iter_mut().zip(k.picture.chunks_exact(4)) {
                    *o = u32::from_le_bytes([p[0], p[1], p[2], p[3]]);
                }
                for (o, &c) in staged[..256].iter_mut().zip(k.palette.iter()) {
                    *o = c as u32;
                }
            }
            None => {
                // At a scale each pixel of the machine's VRAM and picture is a block of the same value (§39.6).
                (self.width, self.height) = (gpu.display.width, gpu.display.height);
                for (i, o) in back[..VRAM_WORDS * n * n].iter_mut().enumerate() {
                    let (x, y) = ((i % (1024 * n)) / n, i / (1024 * n) / n);
                    *o = gpu.vram.get(x as u32, y as u32) as u32;
                }
                let (w, frame) = (self.width as usize * n, gpu.display.frame());
                for (i, o) in slot[..w * self.height as usize * n].iter_mut().enumerate() {
                    let at = ((i / w / n) * self.width as usize + (i % w) / n) * 4;
                    *o = u32::from_le_bytes([frame[at], frame[at + 1], frame[at + 2], frame[at + 3]]);
                }
                for (o, &c) in staged[..256].iter_mut().zip(gpu.clut.colours.iter()) {
                    *o = c as u32;
                }
            }
        }
        let (w, h) = (self.width * n as u32, self.height * n as u32);
        (self.slot, self.slot_entries, self.next_slot) = (0, 256, 1);
        let records = &self.device.staged().records;
        (self.handed, self.ahead) = ((0, w, h), (0, w, h));
        self.device.gpu.submit(|c| {
            c.copy(&held.back, &held.vram, 0, 0, 0);
            c.copy(&held.slots[0], &held.picture, (w * h * 4) as u64, 0, 0);
            c.copy(records, &held.cluts, 256 * 4, 0, 0);
        });
        if self.options.colour {
            // True colour's plane: the state's, or none, the machine's VRAM having only the console's five bits.
            match restored.and_then(|k| k.fine.as_ref()) {
                Some(f) => {
                    for (o, &l) in back.iter_mut().zip(f.iter()) {
                        *o = l as u32;
                    }
                    self.device.gpu.submit(|c| c.copy(&held.back, &held.fine, 0, 0, 0));
                }
                None => self.device.gpu.submit(|c| c.fill(&held.fine, 0)),
            }
            self.counts.submissions += 1;
        }
        self.counts.submissions += 1;
        self.polyline = gpu.polyline.map(|p| (p.command, p.last));
        self.transfer = gpu.write.map(|w| Transfer { rect: w.rect, mask: Mask::from_e6(gpu.env[5]), done: w.done, record: None, row: None });
    }
}
