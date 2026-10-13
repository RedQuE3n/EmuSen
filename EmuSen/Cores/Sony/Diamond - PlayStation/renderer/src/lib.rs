//! Diamond's renderer (stage E3): the PlayStation's picture drawn on a Vulkan compute device from the GPU's command
//! stream, beside the software rasteriser, which keeps VRAM. A `Layer`: told each command, each word that continues
//! one and each display line, it keeps its own copy of VRAM in step and reads its own picture from it. Nothing it makes
//! reaches the machine (Diamond_Enhancements.md §14 on).

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

use blocks::{Blocks, TILES};
use device::{Back, CopyStep, Device, ENTRIES, LINES, LOADS, PIXELS, RECORDS, SLOTS, Step, TILE_RECORDS, Used};
use record::{Record, Span, WORDS};

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
    written: Blocks,
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
    next_slot: u32,
    width: u32,
    height: u32,
    counts: Counts,
}

impl Observer for Renderer {}

impl Renderer {
    /// A renderer on the device `name` names a part of (`EMUSEN_GPU_DEVICE` when none), or the reason there is none.
    pub fn new(name: Option<&str>) -> Result<Renderer, String> {
        let gpu = GpuDevice::try_create(name)?;
        Ok(Renderer {
            device: Device::new(gpu)?,
            steps: Vec::with_capacity(256),
            used: Used::default(),
            stretch: Stretch { written: Blocks::default(), read: Blocks::default(), pairs: Vec::with_capacity(1 << 16), loads_from: 0, lines_from: 0, clear: false, serial: 1, rows: vec![(0, 0); 576] },
            counts_per_tile: vec![0; TILES],
            touched: Vec::with_capacity(TILES),
            polyline: None,
            transfer: None,
            slot: 0,
            next_slot: 1,
            width: 256,
            height: 240,
            counts: Counts::default(),
        })
    }

    /// The device's report, `"{name}, Vulkan {major}.{minor}"`.
    pub fn report(&self) -> String {
        self.device.gpu.report()
    }

    pub fn counts(&self) -> Counts {
        self.counts
    }

    /// The picture as it stands, RGBA at its own size, once the device has drawn everything it was told.
    pub fn picture(&mut self) -> (&[u8], u32, u32) {
        let words = (self.width * self.height) as usize;
        self.submit(Back::Picture(words));
        (&self.device.held().back.slice::<u8>()[..words * 4], self.width, self.height)
    }

    /// The renderer's VRAM, a halfword a pixel as the GPU's `Vram` holds them, once everything told is drawn.
    pub fn vram(&mut self, out: &mut [u16]) {
        self.submit(Back::Vram);
        for (o, &w) in out.iter_mut().zip(self.device.held().back.slice::<u32>()) {
            *o = w as u16;
        }
    }

    // --- The stream ---

    fn command(&mut self, w: &[u32], d: &Drawing) {
        self.end_transfer_record();
        self.polyline = None;
        self.transfer = None;
        let op = w[0] >> 24;
        match op >> 5 {
            1 => self.polygon(w, d),
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
                self.add(record::rectangle(r.sprite, &paint, slot));
            }
            4 => self.copy(Rect::copy_params(w[1], w[3]), Rect::copy_params(w[2], w[3]), d.mask()),
            5 => self.transfer = Some(Transfer { rect: Rect::copy_params(w[1], w[2]), mask: d.mask(), done: 0, record: None, row: None }),
            _ if op == 0x02 => self.add(record::fill(Rect::fill_params(w[1], w[2]), w[0], d.skip_parity)),
            _ => {}
        }
    }

    fn polygon(&mut self, w: &[u32], d: &Drawing) {
        let p = decode::polygon(w, d.env[4]);
        let d = if p.textured { d.with_page(p.page) } else { *d };
        let texture = p.textured.then(|| d.texture(p.clut));
        let slot = texture.map_or(0, |t| self.palette(&t, &d));
        let modulated = texture.is_some() && !p.raw;
        let paint = d.paint(p.blended, texture, p.raw && texture.is_some(), modulated || (p.shaded && texture.is_none()));
        let tex = texture.is_some();
        self.add(record::triangle(raster::fold([p.v[0], p.v[1], p.v[2]]), p.shaded, tex, &paint, slot));
        if p.quad {
            self.add(record::triangle(raster::fold([p.v[1], p.v[2], p.v[3]]), p.shaded, tex, &paint, slot));
        }
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
            self.close();
        }
        if self.used.loads == LOADS {
            self.submit(Back::Nothing);
        }
        let slot = self.next_slot;
        self.next_slot = (self.next_slot + 1) % SLOTS as u32;
        let l = self.used.loads;
        self.device.held().loads.0.slice_mut::<u32>()[l * 4..l * 4 + 4].copy_from_slice(&[slot, t.clut_x, t.clut_y, t.entries()]);
        self.used.loads += 1;
        self.counts.loads += 1;
        self.slot = slot;
        slot
    }

    fn copy(&mut self, src: Rect, dst: Rect, mask: Mask) {
        self.close();
        let dd = (dst.y.wrapping_sub(src.y)) & 511;
        let chunk = if dd >= 1 && dd < src.h { dd } else { src.h };
        let mut row = 0;
        while row < src.h {
            let rows = chunk.min(src.h - row);
            let step = CopyStep { write: 0, sx: src.x, sy: src.y, dx: dst.x, dy: dst.y, width: src.w, row, rows, mask: (mask.check as u32) << 1 | (mask.set != 0) as u32 };
            self.steps.push(Step::Copy(step));
            self.steps.push(Step::Copy(CopyStep { write: 1, ..step }));
            row += rows;
        }
        self.counts.copies += 1;
        if self.steps.len() > 4096 {
            self.submit(Back::Nothing);
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
                    self.transfer = Some(x);
                    self.close();
                    x = self.transfer.expect("kept");
                }
                self.stretch.written.mark(span);
                x.row = Some(row);
            }
            if self.used.pixels == PIXELS || self.used.records == RECORDS {
                self.transfer = Some(x);
                self.submit(Back::Nothing);
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
            let pixels = self.device.held().pixels.0.slice_mut::<u32>();
            let value = word >> (16 * half) & 0xFFFF;
            pixels[p / 2] = if p % 2 == 0 { value } else { pixels[p / 2] & 0xFFFF | value << 16 };
            self.used.pixels += 1;
            x.done += 1;
            self.device.held().records.0.slice_mut::<i32>()[r * WORDS + 15] = x.done as i32;
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
                    if self.stretch.written.meets(Span::sized(x, y, n, 1)) {
                        self.close();
                    }
                }
                let line = [row, x, y, width, deep as u32, blanked as u32, 0, 0];
                let at = match self.stretch.rows[row as usize] {
                    (serial, at) if serial == self.stretch.serial => at,
                    _ => {
                        if self.used.lines == LINES {
                            self.submit(Back::Nothing);
                        }
                        let at = self.used.lines;
                        self.used.lines += 1;
                        self.stretch.rows[row as usize] = (self.stretch.serial, at);
                        at
                    }
                };
                self.device.held().lines.0.slice_mut::<u32>()[at * 8..at * 8 + 8].copy_from_slice(&line);
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
        if r.texels.is_some_and(|t| self.stretch.written.meets(t)) || self.stretch.read.meets(r.writes) {
            self.close();
        }
        let mut tiles = 0;
        blocks::tiles(r.writes, |_| tiles += 1);
        if self.used.records == RECORDS || self.used.entries + self.stretch.pairs.len() + tiles > ENTRIES || self.used.tiles + TILES > TILE_RECORDS {
            self.submit(Back::Nothing);
        }
        if let Some(t) = r.texels {
            let mut own = Blocks::default();
            own.mark(r.writes);
            if own.meets(t) {
                self.counts.feedback += 1;
            }
            self.stretch.read.mark(t);
        }
        self.stretch.written.mark(r.writes);
        self.put(&r);
    }

    /// The record staged at the next index, its tiles listed in the stretch.
    fn put(&mut self, r: &Record) {
        let i = self.used.records;
        self.device.held().records.0.slice_mut::<i32>()[i * WORDS..(i + 1) * WORDS].copy_from_slice(&r.words);
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
        let words = &self.device.held().records.0.slice::<i32>()[r * WORDS..(r + 1) * WORDS];
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
            let held = self.device.held();
            let (entries, tiles) = (held.entries.0.slice_mut::<u32>(), held.tiles.0.slice_mut::<u32>());
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
        self.counts.submissions += 1;
        self.steps.clear();
        self.used = Used::default();
        self.stretch.loads_from = 0;
        self.stretch.lines_from = 0;
    }
}

impl Layer for Renderer {
    fn executed(&mut self, c: &Command) {
        match c.part {
            Part::Whole => self.command(c.words, c.drawing),
            Part::Vertex => self.polyline_vertex(c.words, c.drawing),
            Part::Data => self.data(c.words[0]),
        }
    }

    fn shown(&mut self, s: Shown) {
        Renderer::shown(self, s);
    }

    /// The GPU's VRAM, picture, CLUT cache and the transfer or polyline it has open, taken as the renderer's own; what was
    /// planned before is dropped, being of another machine.
    fn seed(&mut self, gpu: &Gpu) {
        self.steps.clear();
        self.used = Used::default();
        let s = &mut self.stretch;
        (s.loads_from, s.lines_from, s.clear) = (0, 0, false);
        s.written.clear();
        s.read.clear();
        s.pairs.clear();
        s.serial += 1;
        (self.width, self.height) = (gpu.display.width, gpu.display.height);
        let held = self.device.held();
        for (o, h) in held.back.slice_mut::<u32>().iter_mut().zip(gpu.vram.bytes.chunks_exact(2)) {
            *o = u16::from_le_bytes([h[0], h[1]]) as u32;
        }
        let words = (self.width * self.height) as usize;
        let staged = held.records.0.slice_mut::<u32>();
        for (o, p) in staged.iter_mut().zip(gpu.display.frame().chunks_exact(4)) {
            *o = u32::from_le_bytes([p[0], p[1], p[2], p[3]]);
        }
        for (o, &c) in staged[words..words + 256].iter_mut().zip(gpu.clut.colours.iter()) {
            *o = c as u32;
        }
        (self.slot, self.next_slot) = (0, 1);
        self.device.gpu.submit(|c| {
            c.copy(&held.back, &held.vram, 0, 0, 0);
            c.copy(&held.records.0, &held.picture, (words * 4) as u64, 0, 0);
            c.copy(&held.records.0, &held.cluts, 256 * 4, (words * 4) as u64, 0);
        });
        self.counts.submissions += 1;
        self.polyline = gpu.polyline.map(|p| (p.command, p.last));
        self.transfer = gpu.write.map(|w| Transfer { rect: w.rect, mask: Mask::from_e6(gpu.env[5]), done: w.done, record: None, row: None });
    }
}
