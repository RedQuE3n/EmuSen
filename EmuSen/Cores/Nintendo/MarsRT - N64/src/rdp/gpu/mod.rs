//! The multiple's shading on a compute device: the host walks and bins rows, the device shades them, with the C# path's five SPIR-V
//! binaries byte for byte, its 64 by 1 tiles, its pending walk and its device average. C#'s `GpuRasteriser.cs`. See Mars_Gpu.md §5, §13,
//! §14 and §15, and Mars_Native.md §6.4.

pub mod device;
mod record;

use std::sync::{Arc, Mutex};

pub use device::{GpuDevice, Pending};
use device::{Commands, GpuBuffer, GpuProgram, Where};

/// The C# path's compiled shaders, the same files, so the differential between the two cores' device paths has no shader in it.
pub mod shaders {
    pub const SHADE: &[u8] = include_bytes!("../../../../Mars - N64/Rdp/Gpu/Shaders/shade.spv");
    pub const SCAN: &[u8] = include_bytes!("../../../../Mars - N64/Rdp/Gpu/Shaders/scan.spv");
    pub const CLEAR: &[u8] = include_bytes!("../../../../Mars - N64/Rdp/Gpu/Shaders/clear.spv");
    pub const AVERAGE: &[u8] = include_bytes!("../../../../Mars - N64/Rdp/Gpu/Shaders/average.spv");
    pub const ADD: &[u8] = include_bytes!("../../../../Mars - N64/Rdp/Gpu/Shaders/add.spv");
}

pub const TILE_WIDTH: usize = 64;
pub const TILE_HEIGHT: usize = 1;
pub const PRIMITIVE_WORDS: usize = 80;
pub const ROW_WORDS: usize = 32;
/// A snapshot of the processor's four kilobytes, in sixteen-bit words, and the eight tile descriptors packed into four words each (Mars_Gpu.md §7).
pub const TEXTURE_MEMORY_WORDS: usize = 2048;
pub const TILE_WORDS: usize = 32;
pub const MAX_SPANS: usize = 65535;

pub const SCAN_WIDE: u32 = 1;
pub const SCAN_RESAMPLE: u32 = 2;
pub const SCAN_DIVOT: u32 = 4;
pub const SCAN_DITHER: u32 = 8;
pub const SCAN_GAMMA: u32 = 16;
pub const SCAN_RASTER: u32 = 32;

/// `shade.comp`'s push constants, in its order.
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Push {
    image_word: u32,
    width: u32,
    pixel_words: u32,
    memory_words: u32,
    tile_x0: u32,
    tile_y0: u32,
    tiles_wide: u32,
    tiles_high: u32,
    depth_word: u32,
}

/// What the VI's walk needs to know about one scan, as `scan.comp` reads it (Mars_Gpu.md §13), and where it lands in the raster at the
/// multiple for a walk into it (§15).
#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct ScanParameters {
    pub origin: u32,
    pub width: u32,
    pub source_bytes: u32,
    pub flags: u32,
    pub start_x: u32,
    pub step_x: u32,
    pub start_y: u32,
    pub step_y: u32,
    pub rows: u32,
    pub columns: u32,
    pub anti_alias: u32,
    pub raster_pixels: u32,
    pub line_base: u32,
    pub stride: u32,
    pub first_column: i32,
    pub last_column: i32,
    pub unused: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ClearPush {
    count: u32,
    pixels: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct AveragePush {
    width: u32,
    side: u32,
    columns: u32,
    rows: u32,
}

/// Why a primitive was not shaded (Mars_Gpu.md §9).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Declined {
    Carry,
    TwoCycle,
    Copy,
    Image,
}

/// What the batch asked that the device does not shade, and columns past the image's width, which the CPU path wraps into the next row (Mars_Gpu.md §5).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counters {
    pub flushes: i64,
    pub rows_shaded: i64,
    pub primitives_not_shaded: i64,
    pub columns_past_the_width: i64,
    pub rows_of_unsupported_images: i64,
    pub not_shaded_carry: i64,
    pub not_shaded_two_cycle: i64,
    pub not_shaded_copy: i64,
    pub not_shaded_image: i64,
    pub scans: i64,
}

/// Host and device twins of one input, grown together.
#[derive(Default)]
struct Staged {
    host: Option<GpuBuffer>,
    device: Option<GpuBuffer>,
}

/// A picture the device wrote into a host buffer, read by the presenter's thread once the pending submission is done.
#[derive(Default)]
pub struct HostPicture {
    buffer: Option<GpuBuffer>,
    pixels: usize,
}

/// What the presenter's thread reaches: the pending submission to wait for, and the two host pictures.
pub struct Pictures {
    pub pending: Arc<Pending>,
    pub scanned: Mutex<HostPicture>,
    pub averaged: Mutex<HostPicture>,
}

impl Pictures {
    /// `ScannedPicture`: the last scan's picture, one word a pixel, once the device has finished it.
    pub fn scanned(&self, read: impl FnOnce(&[u32])) {
        self.pending.wait();
        let picture = self.scanned.lock().unwrap_or_else(|e| e.into_inner());
        read(picture.buffer.as_ref().map_or(&[][..], |b| &b.slice::<u32>()[..picture.pixels]));
    }

    /// `AveragedRaster`: the averaged raster, once the device has finished it.
    pub fn averaged(&self, read: impl FnOnce(&[u32])) {
        self.pending.wait();
        let picture = self.averaged.lock().unwrap_or_else(|e| e.into_inner());
        read(picture.buffer.as_ref().map_or(&[][..], |b| &b.slice::<u32>()[..picture.pixels]));
    }
}

pub struct GpuRasteriser {
    device: GpuDevice,
    shade: GpuProgram,
    scan: GpuProgram,
    clear: GpuProgram,
    average: GpuProgram,
    memory: GpuBuffer,
    divide: GpuBuffer,
    memory_words: u32,

    primitives: Vec<u32>,
    rows: Vec<u32>,
    tile_offsets: Vec<u32>,
    tile_rows: Vec<u32>,
    texture_memory: Vec<u32>,
    tiles: Vec<u32>,
    primitive_count: usize,
    row_count: usize,
    texture_memory_count: usize,
    tile_count: usize,

    image_word: u32,
    width: u32,
    pixel_words: u32,
    depth_word: u32,
    min_x: i32,
    min_y: i32,
    max_x: i32,
    max_y: i32,

    inputs: [Staged; 6],
    readback: Option<GpuBuffer>,
    scan_device: Option<GpuBuffer>,
    raster: Option<GpuBuffer>,
    averaged: Option<GpuBuffer>,
    spans: Option<GpuBuffer>,
    raster_width: usize,
    raster_height: usize,
    averaged_pixels: usize,
    pictures: Arc<Pictures>,

    pub counters: Counters,
}

/// A normalised w's top six bits pick a reciprocal and a slope, as `Rdp.BuildDivideTable` builds them (Mars_Gpu.md §7.2).
fn build_divide_table() -> Vec<i32> {
    let reciprocal_point = |segment: i32| -> i32 { if segment == 6 { 0x3A83 } else { (0x10_0000 as f64 / (64.0 + segment as f64)).round_ties_even() as i32 } };
    (0..0x8000i32)
        .map(|w| {
            let mut k = 1;
            while k <= 14 && ((w << k) & 0x8000) == 0 {
                k += 1;
            }
            let shift = k - 1;
            let normalised = (w << shift) & 0x3FFF;
            let fraction = (normalised & 0xFF) << 2;
            let segment = normalised >> 8;
            let point = reciprocal_point(segment);
            let slope = reciprocal_point(segment + 1) - point;
            shift | (((((slope * fraction) >> 10) + point) & 0x7FFF) << 4)
        })
        .collect()
}

fn round_up(bytes: u64) -> u64 {
    bytes.max(1).next_power_of_two()
}

impl GpuRasteriser {
    /// The multiple's memory bound whole: None, with the reason, when it is more than the device will bind, which is the CPU path's case.
    #[allow(clippy::result_large_err)]
    pub fn try_create(device: GpuDevice, scaled_rdram_bytes: usize) -> Result<GpuRasteriser, (GpuDevice, String)> {
        let bytes = scaled_rdram_bytes as u64 / 2 * 4;
        if bytes > device.max_buffer_bytes {
            let report = format!("{} binds {} MB at most and the memory at this multiple is {} MB", device.name, device.max_buffer_bytes >> 20, bytes >> 20);
            return Err((device, report));
        }
        match GpuRasteriser::new(&device, (scaled_rdram_bytes / 2) as u32) {
            Ok(make) => Ok(make(device)),
            Err(e) => {
                let report = format!("{}: {e}", device.name);
                Err((device, report))
            }
        }
    }

    #[allow(clippy::type_complexity)]
    fn new(device: &GpuDevice, memory_words: u32) -> Result<Box<dyn FnOnce(GpuDevice) -> GpuRasteriser>, String> {
        let memory = device.create_buffer(memory_words as u64 * 4, Where::Device)?;
        let shade = device.create_program(shaders::SHADE, 8, 36)?;
        let scan = device.create_program(shaders::SCAN, 2, 68)?;
        let clear = device.create_program(shaders::CLEAR, 2, 8)?;
        let average = device.create_program(shaders::AVERAGE, 2, 16)?;

        // The reciprocals perspective division reads are a table of the host's, uploaded once: their construction rounds in floating point.
        let table = build_divide_table();
        let staged = device.create_buffer(table.len() as u64 * 4, Where::Host)?;
        staged.slice_mut::<i32>()[..table.len()].copy_from_slice(&table);
        let divide = device.create_buffer(table.len() as u64 * 4, Where::Device)?;
        device.submit(|c| c.copy(&staged, &divide, 0, 0, 0));
        device.destroy_buffer(staged);
        let pictures = Arc::new(Pictures { pending: device.pending(), scanned: Mutex::new(HostPicture::default()), averaged: Mutex::new(HostPicture::default()) });

        Ok(Box::new(move |device: GpuDevice| {
            let mut made = GpuRasteriser {
                device,
                shade,
                scan,
                clear,
                average,
                memory,
                divide,
                memory_words,
                primitives: vec![0; PRIMITIVE_WORDS * 256],
                rows: vec![0; ROW_WORDS * 4096],
                tile_offsets: vec![0; 1],
                tile_rows: vec![0; 4096],
                texture_memory: vec![0; TEXTURE_MEMORY_WORDS * 8],
                tiles: vec![0; TILE_WORDS * 64],
                primitive_count: 0,
                row_count: 0,
                texture_memory_count: 0,
                tile_count: 0,
                image_word: 0,
                width: 0,
                pixel_words: 0,
                depth_word: 0,
                min_x: i32::MAX,
                min_y: i32::MAX,
                max_x: i32::MIN,
                max_y: i32::MIN,
                inputs: Default::default(),
                readback: None,
                scan_device: None,
                raster: None,
                averaged: None,
                spans: None,
                raster_width: 0,
                raster_height: 0,
                averaged_pixels: 0,
                pictures,
                counters: Counters::default(),
            };
            made.clear();
            made
        }))
    }

    pub fn device_name(&self) -> &str {
        &self.device.name
    }

    pub fn memory_words(&self) -> u32 {
        self.memory_words
    }

    /// What the presenter's thread holds to read the device's pictures.
    pub fn pictures(&self) -> Arc<Pictures> {
        self.pictures.clone()
    }

    /// `Clear`: the memory emptied, as the shadow is when a state is read.
    pub fn clear(&mut self) {
        self.reset_batch();
        self.device.submit(|c| c.fill(&self.memory, 0));
    }

    /// `Image`: the image the rows that follow are drawn into; a change of image ends the batch (Mars_Gpu.md §5).
    pub fn image(&mut self, address: u32, width: i32, pixel_bytes: i32) {
        let (word, words) = (address >> 1, (pixel_bytes / 2) as u32);
        if word == self.image_word && width as u32 == self.width && words == self.pixel_words {
            return;
        }
        self.flush();
        self.image_word = word;
        self.width = width as u32;
        self.pixel_words = words;
    }

    /// `DepthImage`: a change of it ends the batch, since an invocation carries one depth word (Mars_Gpu.md §6).
    pub fn depth_image(&mut self, address: u32) {
        if address >> 1 == self.depth_word {
            return;
        }
        self.flush();
        self.depth_word = address >> 1;
    }

    /// False for an image this cannot shade: an eight-bit one shares a word between two pixels, which two invocations cannot both write.
    pub fn shades(&self) -> bool {
        self.pixel_words != 0
    }

    /// `Primitive`: a primitive's record, zeroed, for the caller to fill; its index is what its rows name.
    pub fn primitive(&mut self) -> (usize, &mut [u32]) {
        if (self.primitive_count + 1) * PRIMITIVE_WORDS > self.primitives.len() {
            self.primitives.resize(self.primitives.len() * 2, 0);
        }
        let index = self.primitive_count;
        self.primitive_count += 1;
        let record = &mut self.primitives[index * PRIMITIVE_WORDS..(index + 1) * PRIMITIVE_WORDS];
        record.fill(0);
        (index, record)
    }

    /// `Row`: a row's record with its first four words set; the span keeps its true ends, which the values at its first pixel are measured from.
    /// The caller says how many columns past the width the CPU path would have written, which is what the counter is for (Mars_Gpu.md §5.4).
    pub fn row(&mut self, primitive: usize, y: i32, left: i32, right: i32, written_past_the_width: i32) -> Option<&mut [u32]> {
        if !self.shades() {
            self.counters.rows_of_unsupported_images += 1;
            return None;
        }
        self.counters.columns_past_the_width += written_past_the_width as i64;
        let (shown_left, shown_right) = (left.max(0), right.min(self.width as i32 - 1));
        if shown_right < shown_left || y < 0 {
            return None;
        }
        if (self.row_count + 1) * ROW_WORDS > self.rows.len() {
            self.rows.resize(self.rows.len() * 2, 0);
        }
        let index = self.row_count;
        self.row_count += 1;
        self.min_x = self.min_x.min(shown_left);
        self.max_x = self.max_x.max(shown_right);
        self.min_y = self.min_y.min(y);
        self.max_y = self.max_y.max(y);
        let record = &mut self.rows[index * ROW_WORDS..(index + 1) * ROW_WORDS];
        record.fill(0);
        record[0] = primitive as u32;
        record[1] = y as u32;
        record[2] = left as u32;
        record[3] = right as u32;
        Some(record)
    }

    pub fn not_shaded(&mut self, why: Declined) {
        self.counters.primitives_not_shaded += 1;
        match why {
            Declined::Carry => self.counters.not_shaded_carry += 1,
            Declined::TwoCycle => self.counters.not_shaded_two_cycle += 1,
            Declined::Copy => self.counters.not_shaded_copy += 1,
            Declined::Image => self.counters.not_shaded_image += 1,
        }
    }

    /// `TextureMemory`: the snapshot the primitives that follow sample, pushed only when the processor's own has changed since the last (Mars_Gpu.md §7.1).
    pub fn texture_memory(&mut self, memory: &[u8], changed: bool) -> u32 {
        if !changed && self.texture_memory_count > 0 {
            return (self.texture_memory_count - 1) as u32;
        }
        if (self.texture_memory_count + 1) * TEXTURE_MEMORY_WORDS > self.texture_memory.len() {
            self.texture_memory.resize(self.texture_memory.len() * 2, 0);
        }
        let at = self.texture_memory_count * TEXTURE_MEMORY_WORDS;
        for i in 0..TEXTURE_MEMORY_WORDS {
            self.texture_memory[at + i] = ((memory[i * 2] as u32) << 8) | memory[i * 2 + 1] as u32;
        }
        self.texture_memory_count += 1;
        (self.texture_memory_count - 1) as u32
    }

    /// `Tiles`: the index of the tile set the primitives that follow use, and the set's words to fill when it is a new one.
    pub fn tiles(&mut self, changed: bool) -> (u32, Option<&mut [u32]>) {
        if !changed && self.tile_count > 0 {
            return ((self.tile_count - 1) as u32, None);
        }
        if (self.tile_count + 1) * TILE_WORDS > self.tiles.len() {
            self.tiles.resize(self.tiles.len() * 2, 0);
        }
        let index = self.tile_count;
        self.tile_count += 1;
        (index as u32, Some(&mut self.tiles[index * TILE_WORDS..(index + 1) * TILE_WORDS]))
    }

    fn reset_batch(&mut self) {
        self.primitive_count = 0;
        self.row_count = 0;
        self.texture_memory_count = 0;
        self.tile_count = 0;
        (self.min_x, self.min_y) = (i32::MAX, i32::MAX);
        (self.max_x, self.max_y) = (i32::MIN, i32::MIN);
    }

    /// `Bin`: rows into the tiles they touch, in the order they were drawn: counted, summed, then placed (Mars_Gpu.md §5).
    fn bin(&mut self, tile_x0: i32, tile_y0: i32, tiles_wide: i32, tiles_high: i32) -> usize {
        let tiles = (tiles_wide * tiles_high) as usize;
        if self.tile_offsets.len() < tiles + 1 {
            self.tile_offsets = vec![0; tiles + 1];
        }
        self.tile_offsets[..tiles + 1].fill(0);
        let width = self.width as i32;
        let (tw, th) = (TILE_WIDTH as i32, TILE_HEIGHT as i32);

        for r in 0..self.row_count {
            let at = r * ROW_WORDS;
            let slot = (self.rows[at + 1] as i32 / th - tile_y0) * tiles_wide - tile_x0;
            let (from, to) = ((self.rows[at + 2] as i32).max(0) / tw, (self.rows[at + 3] as i32).min(width - 1) / tw);
            for t in from..=to {
                self.tile_offsets[(slot + t + 1) as usize] += 1;
            }
        }
        for t in 0..tiles {
            self.tile_offsets[t + 1] += self.tile_offsets[t];
        }
        let total = self.tile_offsets[tiles] as usize;
        if self.tile_rows.len() < total {
            self.tile_rows = vec![0; total.max(self.tile_rows.len() * 2)];
        }

        // Placed from each tile's start with a cursor, which the offsets are restored from afterwards.
        let mut cursor: Vec<u32> = self.tile_offsets[..tiles].to_vec();
        for r in 0..self.row_count {
            let at = r * ROW_WORDS;
            let slot = (self.rows[at + 1] as i32 / th - tile_y0) * tiles_wide - tile_x0;
            let (from, to) = ((self.rows[at + 2] as i32).max(0) / tw, (self.rows[at + 3] as i32).min(width - 1) / tw);
            for t in from..=to {
                let c = &mut cursor[(slot + t) as usize];
                self.tile_rows[*c as usize] = r as u32;
                *c += 1;
            }
        }
        total
    }

    fn upload(&mut self, which: usize, count: usize) {
        let bytes = count.max(1) as u64 * 4;
        let staged = &mut self.inputs[which];
        if staged.host.as_ref().is_none_or(|h| h.bytes < bytes) {
            if let Some(h) = staged.host.take() {
                self.device.destroy_buffer(h);
            }
            if let Some(d) = staged.device.take() {
                self.device.destroy_buffer(d);
            }
            let capacity = round_up(bytes);
            staged.host = Some(self.device.create_buffer(capacity, Where::Host).expect("a staging buffer"));
            staged.device = Some(self.device.create_buffer(capacity, Where::Device).expect("a device buffer"));
        }
        let source: &[u32] = match which {
            0 => &self.primitives,
            1 => &self.rows,
            2 => &self.tile_offsets,
            3 => &self.tile_rows,
            4 => &self.texture_memory,
            _ => &self.tiles,
        };
        staged.host.as_ref().expect("staged").slice_mut::<u32>()[..count].copy_from_slice(&source[..count]);
    }

    /// `Flush`: everything recorded is shaded, and nothing of it is in flight when this returns.
    pub fn flush(&mut self) {
        if let Some(shade) = self.stage() {
            let (device, this) = (&self.device, &*self);
            device.submit(|c| this.record_shading(c, &shade));
        }
    }

    /// `Stage`: the batch binned and its inputs staged, as what the commands that shade it need; None when nothing was recorded.
    fn stage(&mut self) -> Option<Shading> {
        if self.row_count == 0 {
            self.reset_batch();
            return None;
        }
        // The staging buffers and the program's set belong to the pending submission until it finishes (Mars_Gpu.md §14).
        self.device.wait_for_pending();

        let (tile_x0, tile_y0) = (self.min_x / TILE_WIDTH as i32, self.min_y / TILE_HEIGHT as i32);
        let (tiles_wide, tiles_high) = (self.max_x / TILE_WIDTH as i32 - tile_x0 + 1, self.max_y / TILE_HEIGHT as i32 - tile_y0 + 1);
        let listed = self.bin(tile_x0, tile_y0, tiles_wide, tiles_high);
        let counts = [
            self.primitive_count * PRIMITIVE_WORDS,
            self.row_count * ROW_WORDS,
            (tiles_wide * tiles_high) as usize + 1,
            listed,
            self.texture_memory_count * TEXTURE_MEMORY_WORDS,
            self.tile_count * TILE_WORDS,
        ];
        for (i, &count) in counts.iter().enumerate() {
            self.upload(i, count);
        }
        let push = Push {
            image_word: self.image_word,
            width: self.width,
            pixel_words: self.pixel_words,
            memory_words: self.memory_words,
            tile_x0: tile_x0 as u32,
            tile_y0: tile_y0 as u32,
            tiles_wide: tiles_wide as u32,
            tiles_high: tiles_high as u32,
            depth_word: self.depth_word,
        };
        self.counters.flushes += 1;
        self.counters.rows_shaded += self.row_count as i64;
        self.reset_batch();
        Some(Shading { counts, push })
    }

    /// The staged batch's commands: six copies, then one dispatch over its tiles.
    fn record_shading(&self, c: &mut Commands, shading: &Shading) {
        for (i, &count) in shading.counts.iter().enumerate() {
            let staged = &self.inputs[i];
            c.copy(staged.host.as_ref().expect("staged"), staged.device.as_ref().expect("staged"), count.max(1) as u64 * 4, 0, 0);
        }
        let d = |i: usize| self.inputs[i].device.as_ref().expect("staged");
        c.dispatch(&self.shade, &[&self.memory, d(0), d(1), d(2), d(3), d(4), d(5), &self.divide], &shading.push, shading.push.tiles_wide, shading.push.tiles_high, 1);
    }

    /// `Read`: the device's words from one byte address for so many bytes, laid into the shadow's two arrays as the CPU path would have left them.
    pub fn read(&mut self, address: usize, bytes: usize, rdram: &mut [u8], hidden: &mut [u8]) {
        self.flush();
        let first_word = address >> 1;
        let words = ((bytes + 1) >> 1).min((self.memory_words as usize).saturating_sub(first_word));
        if words == 0 {
            return;
        }
        if self.readback.as_ref().is_none_or(|r| r.bytes < words as u64 * 4) {
            if let Some(r) = self.readback.take() {
                self.device.destroy_buffer(r);
            }
            self.readback = Some(self.device.create_buffer(round_up(words as u64 * 4), Where::Host).expect("a readback buffer"));
        }
        let readback = self.readback.as_ref().expect("readback");
        self.device.submit(|c| c.copy(&self.memory, readback, words as u64 * 4, first_word as u64 * 4, 0));
        let read = &readback.slice::<u32>()[..words];
        for (i, &value) in read.iter().enumerate() {
            let word = first_word + i;
            rdram[word * 2] = (value >> 8) as u8;
            rdram[word * 2 + 1] = value as u8;
            hidden[word] = (value >> 16) as u8;
        }
    }

    /// `ScanOut`: the last batch and the walk, submitted without waiting; `Pictures::scanned` waits for them. Drawing must be finished (Mars_Gpu.md §14).
    pub fn scan_out(&mut self, scan: &ScanParameters) {
        let shade = self.stage();
        self.device.wait_for_pending();

        let pixels = (scan.rows as usize) * (scan.columns as usize);
        let bytes = pixels.max(1) as u64 * 4;
        {
            let mut scanned = self.pictures.scanned.lock().unwrap_or_else(|e| e.into_inner());
            if self.scan_device.as_ref().is_none_or(|d| d.bytes < bytes) {
                if let Some(d) = self.scan_device.take() {
                    self.device.destroy_buffer(d);
                }
                if let Some(h) = scanned.buffer.take() {
                    self.device.destroy_buffer(h);
                }
                let capacity = round_up(bytes);
                self.scan_device = Some(self.device.create_buffer(capacity, Where::Device).expect("a scan buffer"));
                scanned.buffer = Some(self.device.create_buffer(capacity, Where::Host).expect("a scan buffer"));
            }
            scanned.pixels = pixels;
            let mut push = *scan;
            push.source_bytes = self.memory_words * 2;
            let device = self.scan_device.as_ref().expect("scan");
            let host = scanned.buffer.as_ref().expect("scan");
            let (groups_x, groups_y) = (push.columns.div_ceil(8), push.rows.div_ceil(8));
            let this = &*self;
            self.device.submit_pending(|c| {
                if let Some(shade) = &shade {
                    this.record_shading(c, shade);
                }
                c.dispatch(&this.scan, &[&this.memory, device], &push, groups_x, groups_y, 1);
                c.copy(device, host, bytes, 0, 0);
            });
        }
        self.counters.scans += 1;
    }

    pub fn holds_raster(&self, width: usize, height: usize) -> bool {
        self.raster.is_some() && self.raster_width == width && self.raster_height == height
    }

    /// `ScanIntoRaster`: the spans cleared, the walk written into the raster if there is one, and the raster averaged and sent back, without waiting (Mars_Gpu.md §15).
    #[allow(clippy::too_many_arguments)]
    pub fn scan_into_raster(&mut self, width: usize, height: usize, side: usize, seed: &[u8], clear: bool, spans: &[u32], walk: bool, scan: &ScanParameters) {
        assert!(spans.len() <= MAX_SPANS * 2, "at most {MAX_SPANS} spans a submission");
        let shade = if walk { self.stage() } else { None };
        self.device.wait_for_pending();

        let pixels = width * height;
        let (columns, rows) = (width / side, height / side);
        let fresh = !self.holds_raster(width, height);
        if fresh {
            if let Some(r) = self.raster.take() {
                self.device.destroy_buffer(r);
            }
            self.raster = Some(self.device.create_buffer(pixels as u64 * 4, Where::Device).expect("a raster"));
            self.raster_width = width;
            self.raster_height = height;
        }
        let mut averaged_host = self.pictures.averaged.lock().unwrap_or_else(|e| e.into_inner());
        if self.averaged.is_none() || self.averaged_pixels != columns * rows {
            if let Some(a) = self.averaged.take() {
                self.device.destroy_buffer(a);
            }
            if let Some(h) = averaged_host.buffer.take() {
                self.device.destroy_buffer(h);
            }
            self.averaged_pixels = columns * rows;
            self.averaged = Some(self.device.create_buffer(self.averaged_pixels.max(1) as u64 * 4, Where::Device).expect("an averaged raster"));
            averaged_host.buffer = Some(self.device.create_buffer(self.averaged_pixels.max(1) as u64 * 4, Where::Host).expect("an averaged raster"));
        }
        averaged_host.pixels = self.averaged_pixels;

        let seeded = (fresh && seed.len() == pixels * 4).then(|| {
            let buffer = self.device.create_buffer(pixels as u64 * 4, Where::Host).expect("a seed");
            buffer.slice_mut::<u8>()[..seed.len()].copy_from_slice(seed);
            buffer
        });
        if self.spans.as_ref().is_none_or(|s| s.bytes < spans.len().max(1) as u64 * 4) {
            if let Some(s) = self.spans.take() {
                self.device.destroy_buffer(s);
            }
            self.spans = Some(self.device.create_buffer(round_up(spans.len().max(2) as u64 * 4), Where::Host).expect("a span list"));
        }
        self.spans.as_ref().expect("spans").slice_mut::<u32>()[..spans.len()].copy_from_slice(spans);

        let mut push = *scan;
        push.source_bytes = self.memory_words * 2;
        push.flags |= SCAN_RASTER;
        push.raster_pixels = pixels as u32;
        let cleared = ClearPush { count: (spans.len() / 2) as u32, pixels: pixels as u32 };
        let averaging = AveragePush { width: width as u32, side: side as u32, columns: columns as u32, rows: rows as u32 };
        let raster = self.raster.as_ref().expect("raster");
        let averaged = self.averaged.as_ref().expect("averaged");
        let host = averaged_host.buffer.as_ref().expect("averaged");
        let listed = self.spans.as_ref().expect("spans");
        let this = &*self;
        self.device.submit_pending(|c| {
            if let Some(seeded) = &seeded {
                c.copy(seeded, raster, pixels as u64 * 4, 0, 0);
            } else if fresh {
                c.fill(raster, 0);
            }
            if clear {
                c.fill(raster, 0);
            }
            if cleared.count > 0 {
                c.dispatch(&this.clear, &[raster, listed], &cleared, cleared.count, 1, 1);
            }
            if walk {
                if let Some(shade) = &shade {
                    this.record_shading(c, shade);
                }
                c.dispatch(&this.scan, &[&this.memory, raster], &push, push.columns.div_ceil(8), push.rows.div_ceil(8), 1);
            }
            c.dispatch(&this.average, &[raster, averaged], &averaging, (columns as u32).div_ceil(8), (rows as u32).div_ceil(8), 1);
            c.copy(averaged, host, (columns * rows) as u64 * 4, 0, 0);
        });
        drop(averaged_host);

        // The seed's host copy is read by the submission, so it goes when that has finished.
        if let Some(seeded) = seeded {
            self.device.wait_for_pending();
            self.device.destroy_buffer(seeded);
        }
        if walk {
            self.counters.scans += 1;
        }
    }
}

/// A staged batch: what its commands copy and dispatch.
struct Shading {
    counts: [usize; 6],
    push: Push,
}

impl Drop for GpuRasteriser {
    fn drop(&mut self) {
        self.device.wait_for_pending();
        for buffer in [self.raster.take(), self.averaged.take(), self.spans.take(), self.scan_device.take(), self.readback.take()].into_iter().flatten() {
            self.device.destroy_buffer(buffer);
        }
        for staged in &mut self.inputs {
            for buffer in [staged.host.take(), staged.device.take()].into_iter().flatten() {
                self.device.destroy_buffer(buffer);
            }
        }
        for picture in [&self.pictures.scanned, &self.pictures.averaged] {
            if let Some(buffer) = picture.lock().unwrap_or_else(|e| e.into_inner()).buffer.take() {
                self.device.destroy_buffer(buffer);
            }
        }
        // SAFETY: the programs and the two buffers are destroyed once, here, before the device that made them.
        unsafe {
            let (shade, scan, clear, average) = (std::ptr::read(&self.shade), std::ptr::read(&self.scan), std::ptr::read(&self.clear), std::ptr::read(&self.average));
            let (memory, divide) = (std::ptr::read(&self.memory), std::ptr::read(&self.divide));
            self.device.destroy_program(shade);
            self.device.destroy_program(scan);
            self.device.destroy_program(clear);
            self.device.destroy_program(average);
            self.device.destroy_buffer(memory);
            self.device.destroy_buffer(divide);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The five binaries are the C# path's own files, whole words with the SPIR-V magic, so nothing here can have changed a shader.
    #[test]
    fn the_shaders_are_the_csharp_paths_binaries() {
        for (name, bytes) in [("shade", shaders::SHADE), ("scan", shaders::SCAN), ("clear", shaders::CLEAR), ("average", shaders::AVERAGE), ("add", shaders::ADD)] {
            assert!(bytes.len() % 4 == 0 && bytes.len() > 20, "{name}");
            assert_eq!(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]), 0x0723_0203, "{name}: the SPIR-V magic");
        }
    }

    /// The divide table as `Rdp.BuildDivideTable` builds it: the first, a middle and the last entry, and every shift in range.
    #[test]
    fn the_divide_table_is_the_hosts() {
        let table = build_divide_table();
        assert_eq!(table.len(), 0x8000);
        assert_eq!(table[0] & 0xF, 14);
        assert_eq!(table[0x4000] & 0xF, 0);
        assert_eq!(table[0x4000] >> 4, 0x4000);
        assert!(table.iter().all(|&t| (t & 0xF) <= 14));
    }
}
