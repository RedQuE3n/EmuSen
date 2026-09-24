//! The VI's scan-out: C#'s `Vi.Scan` then `MarsCore.Compose`, at once or deferred to a presenter thread, at one or at the multiple the display
//! processor drew beside the machine's picture, walked on the processor or on the device. See Mars_Native.md §5.4, §5.6 and §6.4.

mod bands;
mod filters;
mod presenter;
#[cfg(test)]
mod tests;
mod walker;

use std::sync::Arc;

use crate::memory::bus::MemoryBus;
use crate::memory::dp::DpInterface;
use crate::memory::dp_threads::site;
use crate::rdp::gpu::{MAX_SPANS, Pictures, SCAN_DITHER, SCAN_DIVOT, SCAN_GAMMA, SCAN_RESAMPLE, SCAN_WIDE, ScanParameters};
use crate::vi::Vi;
pub use bands::default_bands;
use bands::Bands;
use presenter::{Presenter, Work};
use walker::Walker;

/// `Vi.RasterWidth` and `Vi.RasterHeight`: the widest and tallest signal the VI can raise.
pub const RASTER_WIDTH: usize = 640;
pub const RASTER_HEIGHT: usize = 625;
const RASTER_BYTES: usize = RASTER_WIDTH * RASTER_HEIGHT * 4;

const CONTROL: usize = 0;
const ORIGIN: usize = 1;
const WIDTH: usize = 2;
const CURRENT_LINE: usize = 4;
const VERTICAL_SYNC: usize = 6;
const HORIZONTAL_START: usize = 9;
const VERTICAL_START: usize = 10;
const SCALE_X: usize = 12;
const SCALE_Y: usize = 13;

const NTSC_HEIGHT: usize = 480;
const PAL_HEIGHT: usize = 576;
const NTSC_SYNC_LINES: i32 = 525;
const REPLICATE: i32 = 3;

/// C#'s raster and composed frame, which no state holds, and the deferred walk's capture and presenter.
#[derive(Default)]
pub struct Scanout {
    /// The composed RGBA frame, `width * height * 4` bytes; when deferred, the one joined last.
    pub frame: Vec<u8>,
    pub width: u32,
    pub height: u32,
    /// C#'s `RowRepeat`: how many times each row of `frame` is shown.
    pub row_repeat: u32,
    /// C#'s `RepeatRows`; false, as Mistress sets it, sends a progressive field's rows once.
    pub repeat_rows: bool,
    /// C#'s `SkipRepeatedScans` inverted: true walks a deferred scan that repeats the last walk's geometry and bytes.
    pub walk_repeats: bool,
    /// `RepeatedScans`: deferred scans not walked because they repeated the last.
    pub repeated_scans: i64,
    /// `ScanJob.Repeats` counted: deferred captures that repeated the last, walked or not.
    pub repeated_captures: i64,
    /// How many bands a deferred walk is split into; zero takes `default_bands` (Mars_Native.md §6.11).
    pub bands: usize,
    /// A test's hold on one band of the next deferred walks.
    #[cfg(test)]
    pub hold_band: bands::Hold,
    /// The deferred jobs joined, and the presenter's time over them.
    pub joined: i64,
    pub presenter_nanos: i64,
    /// The deferred walks joined that ran in more than one band.
    pub banded_walks: i64,
    /// `Vi._raster` and the raster at the multiple: four bytes a pixel, the fourth coverage, kept between scans and across a load, as C# keeps them.
    rasters: Rasters,
    /// `Vi.Average`: how many pixels each way of the multiple's raster are averaged into one of the picture's (Mars_Video.md §2.10).
    pub average: i32,
    walker: Walker,
    /// `ScanJob`'s capture: the bytes the last deferred walk read, and its geometry.
    capture: Capture,
    /// A border or a blank changed the raster since the last walk, so a repeated scan may not be skipped (Mars_Native.md §5.6.5).
    raster_edited: bool,
    /// The frame the next deferred walk composes into, swapped with `frame` at the join.
    pending: Vec<u8>,
    presenter: Option<Presenter>,
    /// While the device averages, the raster at the multiple is the device's, and these are the clears not yet sent to it (Mars_Gpu.md §15).
    device_spans: Vec<u32>,
    device_clear: bool,
    device_raster: bool,
    shown_from_device: bool,
    /// `_seedValid` inverted, so that the default is valid: the host raster at the multiple may seed the device's when the averaging moves there.
    seed_stale: bool,
}

impl Scanout {
    /// The raster, 640 by 625, as C#'s `Vi._raster` holds it; empty before the first scan and while a walk is out.
    pub fn raster(&self) -> &[u8] {
        &self.rasters.raster
    }

    /// The raster at the multiple the last walk wrote, `Vi._rasterScaled`, and that multiple; empty at one.
    pub fn raster_scaled(&self) -> (&[u8], i32) {
        (&self.rasters.scaled, self.rasters.output_scale.max(1))
    }

    /// True while the shown picture is the device's averaged raster (Mars_Gpu.md §15).
    pub fn shown_from_device(&self) -> bool {
        self.shown_from_device
    }

    fn ensure_raster(&mut self) {
        self.rasters.ensure();
    }

    /// `ScanJob.Forget`: a loaded state's first deferred scan walks, since the raster it presents is not the last walk's.
    pub fn forget(&mut self) {
        self.capture.shape = None;
    }

    /// `JoinPresentation`: the walk out, if one is, finished and its picture made the shown one; true when it was.
    pub fn join(&mut self) -> bool {
        let Some(work) = self.presenter.as_mut().and_then(Presenter::take) else { return false };
        let work = *work;
        self.joined += 1;
        self.presenter_nanos += work.nanos;
        self.banded_walks += (work.walked_in > 1) as i64;
        if let Some(fault) = work.fault {
            panic!("the deferred scan-out failed: {fault}");
        }
        self.rasters = work.rasters;
        self.walker = work.walker;
        self.capture = work.capture;
        self.shown_from_device = work.shown_from_device;
        self.pending = std::mem::replace(&mut self.frame, work.frame);
        (self.width, self.height, self.row_repeat) = work.shown;
        true
    }

    /// The joins that waited for the presenter, and the nanoseconds they waited (Mars_Native.md §6.11).
    pub fn presenter_waits(&self) -> (i64, i64) {
        self.presenter.as_ref().map_or((0, 0), |p| (p.waits, p.waited_nanos))
    }

    /// True while a deferred walk is out.
    pub fn in_flight(&self) -> bool {
        self.presenter.as_ref().is_some_and(Presenter::busy)
    }

    fn raster_edits_pending(&self) -> bool {
        self.device_clear || !self.device_spans.is_empty()
    }
}

/// What a present did: whether a scan walked, and whether the shown frame was replaced, which advances the frame serial.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Presented {
    pub walked: bool,
    pub replaced: bool,
}

/// The bytes one walk reads: memory from `base`, and memory's whole length, past which a fetch reads zero.
#[derive(Clone, Copy)]
pub struct View<'a> {
    pub rdram: &'a [u8],
    pub hidden: &'a [u8],
    pub base: u32,
    pub length: u32,
}

impl<'a> View<'a> {
    fn whole(rdram: &'a [u8], hidden: &'a [u8]) -> View<'a> {
        View { rdram, hidden, base: 0, length: rdram.len().min(hidden.len() * 2) as u32 }
    }
}

/// `ScanJob`'s capture: the lines a walk can reach, copied out of RDRAM with their hidden bits, the same lines of the shadow at the
/// multiple when the walk reads it (`ReachScaled`), the geometry they were taken for, and whether the device walked them (`DeviceScanned`).
#[derive(Default)]
struct Capture {
    rdram: Vec<u8>,
    hidden: Vec<u8>,
    from: u32,
    count: u32,
    length: u32,
    scaled_rdram: Vec<u8>,
    scaled_hidden: Vec<u8>,
    scaled_from: u32,
    scaled_count: u32,
    scaled_length: u32,
    shape: Option<Shape>,
    device_scanned: bool,
    device_scan_at: u64,
}

/// C#'s `_raster`, `_rasterScaled`, `_rasterScale`, `_outputScale` and `_rasterAveraged`: the raster at one, the raster at the multiple the
/// last walk read, which multiple that is, and the averaged one the composition reads (Mars_Video.md §2.9 and §2.10).
#[derive(Default)]
pub(super) struct Rasters {
    pub raster: Vec<u8>,
    pub scaled: Vec<u8>,
    /// The multiple `scaled` is sized for; one while it is empty.
    pub scale: i32,
    /// The multiple the last walk wrote, which the composition divides by.
    pub output_scale: i32,
    pub averaged: Vec<u8>,
}

impl Rasters {
    fn ensure(&mut self) {
        if self.raster.len() != RASTER_BYTES {
            self.raster = vec![0; RASTER_BYTES];
        }
        self.scale = self.scale.max(1);
        self.output_scale = self.output_scale.max(1);
    }

    /// `Vi.Walk`'s raster: the console's at one, else the one at the multiple, made when the multiple changes.
    fn at(&mut self, scale: i32) -> &mut [u8] {
        if scale <= 1 {
            return &mut self.raster;
        }
        let bytes = RASTER_BYTES * (scale * scale) as usize;
        if self.scale != scale || self.scaled.len() != bytes {
            self.scaled = vec![0; bytes];
            self.scale = scale;
        }
        &mut self.scaled
    }
}

/// `Repeated`'s shape: everything the walk reads besides the bytes, the device's presence included.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Shape {
    job: Job,
    from: u32,
    count: u32,
    can_scan_out: bool,
}

impl Capture {
    fn view(&self) -> View<'_> {
        View { rdram: &self.rdram[..self.count as usize], hidden: &self.hidden[..(self.count / 2) as usize], base: self.from, length: self.length }
    }

    /// The shadow's captured lines, as `Walk` reads `ScaledRdram` from `ScaledBase`.
    fn scaled_view(&self) -> View<'_> {
        View { rdram: &self.scaled_rdram[..self.scaled_count as usize], hidden: &self.scaled_hidden[..(self.scaled_count / 2) as usize], base: self.scaled_from, length: self.scaled_length }
    }

    /// `Repeated`: the geometry and the live bytes are the capture's, so a walk would write the raster the last one left.
    fn repeats(&self, shape: Shape, bus: &MemoryBus) -> bool {
        let (from, count) = (shape.from as usize, shape.count as usize);
        self.shape == Some(shape) && bus.rdram[from..from + count] == self.rdram[..count] && bus.rdram_hidden[from / 2..(from + count) / 2] == self.hidden[..count / 2]
    }

    /// `Capture`'s copy, into buffers kept between scans.
    fn take(&mut self, bus: &MemoryBus, from: u32, count: u32) {
        let (at, n) = (from as usize, count as usize);
        self.rdram.resize(n.max(self.rdram.len()), 0);
        self.hidden.resize((n / 2).max(self.hidden.len()), 0);
        self.rdram[..n].copy_from_slice(&bus.rdram[at..at + n]);
        self.hidden[..n / 2].copy_from_slice(&bus.rdram_hidden[at / 2..(at + n) / 2]);
        (self.from, self.count, self.length) = (from, count, bus.rdram.len() as u32);
    }

    /// `Capture`'s copy of the shadow's lines, due exactly when the machine's is, since the shadow changes only when the machine's memory does (Mars_Video.md §2.9).
    fn take_scaled(&mut self, shadow: &[u8], hidden: &[u8], from: u32, count: u32) {
        let (at, n) = (from as usize, count as usize);
        self.scaled_rdram.resize(n.max(self.scaled_rdram.len()), 0);
        self.scaled_hidden.resize((n / 2).max(self.scaled_hidden.len()), 0);
        self.scaled_rdram[..n].copy_from_slice(&shadow[at..at + n]);
        self.scaled_hidden[..n / 2].copy_from_slice(&hidden[at / 2..(at + n) / 2]);
        (self.scaled_from, self.scaled_count, self.scaled_length) = (from, count, shadow.len() as u32);
    }
}

/// The rasters as a scan's borders edit them, noting whether any byte changed; while the device averages, the clears at the multiple go to it as spans.
struct Canvas<'a> {
    rasters: &'a mut Rasters,
    edited: bool,
    device: Option<DeviceEdits<'a>>,
}

struct DeviceEdits<'a> {
    spans: &'a mut Vec<u32>,
    scale: usize,
}

/// `Vi.Picture`: where the picture sits in the raster, its steps, and which columns carry signal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Picture {
    left: i32,
    top: i32,
    columns: i32,
    rows: i32,
    stride: i32,
    active_lines: i32,
    start_x: u32,
    step_x: u32,
    start_y: u32,
    step_y: u32,
    first_column: i32,
    last_column: i32,
    lower: bool,
}

impl Picture {
    /// `ScaledPicture`: the picture at the multiple, its steps unchanged since source and raster grow alike (Mars_Video.md §2.9).
    fn at(&self, n: i32) -> Picture {
        Picture {
            left: self.left * n,
            top: self.top * n,
            columns: self.columns * n,
            rows: self.rows * n,
            stride: self.stride * n,
            active_lines: self.active_lines * n,
            start_x: self.start_x.wrapping_mul(n as u32),
            step_x: self.step_x,
            start_y: self.start_y.wrapping_mul(n as u32),
            step_y: self.step_y,
            first_column: self.first_column * n,
            last_column: self.last_column * n,
            lower: self.lower,
        }
    }
}

/// `ScanJob`'s fields the walk reads: the console's geometry, and the multiple's when the shadow is what it walks (`Scale`, `ScaledPicture`).
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct Job {
    picture: Picture,
    origin: u32,
    width: i32,
    wide: bool,
    resample: bool,
    divot: bool,
    anti_alias: i32,
    dither: bool,
    gamma: bool,
    /// The multiple the walk reads and writes at: the display processor's once it has drawn there, one otherwise.
    scale: i32,
    scaled_picture: Picture,
}

impl Job {
    #[inline(always)]
    pub(super) fn scale(&self) -> i32 {
        self.scale
    }

    fn aligned_origin(&self) -> u32 {
        if self.wide { self.origin & 0xFF_FFFC } else { self.origin & 0xFF_FFFE }
    }
}

/// `job.Scale`: the shadow is read in place of the machine's memory once something has drawn into it (Mars_Video.md §2.9). Until it has,
/// the drain is joined first, so the answer is the words handed over and not how far the workers have got (Mars_Native.md §6.4.4).
fn shown_scale(dp: &mut DpInterface) -> i32 {
    if dp.scale() > 1 && !dp.scaled_drawn() && dp.threads.is_some() {
        dp.join();
    }
    if dp.scaled_drawn() { dp.scale() } else { 1 }
}

/// C#'s `Vi.Scan` then `MarsCore.Compose` over whole, unshared memory at one. True when a walk ran; the frame is composed either way, as `Present` composes it.
pub fn scan(vi: &mut Vi, rdram: &[u8], hidden: &[u8], out: &mut Scanout) -> bool {
    out.join();
    out.ensure_raster();
    let walked = match prepare(vi, None, out).0 {
        Some(job) => {
            out.walker.walk(&job, View::whole(rdram, hidden), None, &mut out.rasters);
            true
        }
        None => false,
    };
    compose(vi, out, None);
    walked
}

/// `MarsCore.Present`: the scan at once; with a drain running, the walk waits for the lines it can reach (site 8) and reads those alone.
/// At a multiple it reads the live shadow whole, whose bytes for those lines the same words drew (Mars_Video.md §2.9), or the device
/// walks its own memory and the picture is written into the raster here (Mars_Gpu.md §13).
pub fn present_now(bus: &mut MemoryBus, out: &mut Scanout) -> Presented {
    out.join();
    out.ensure_raster();
    let (vi, dp) = (&mut bus.vi, &mut bus.dp);
    let walked = match prepare(vi, Some(dp), out).0 {
        Some(job) => {
            let (from, count) = reach(&job, bus.rdram.len() as u32);
            bus.dp.wait_read_range(from, count, site::VI);
            out.capture.device_scanned = false;
            if job.scale > 1 && bus.dp.multiple.can_scan_out() {
                // C#'s immediate scan is a job of its own: its picture replaces the device's, and the deferred capture's record must not say it is that capture's.
                let deferred_scan_at = out.capture.device_scan_at;
                device_scan(bus, out, &job);
                out.capture.device_scan_at = deferred_scan_at;
            } else if job.scale > 1 {
                let (scaled_from, scaled_count) = reach_scaled(&job, bus.dp.multiple.rdram.len() as u32);
                bus.dp.read_back_scaled(scaled_from, scaled_count);
            }
            let (at, n) = (from as usize, count as usize);
            let view = View { rdram: &bus.rdram[at..at + n], hidden: &bus.rdram_hidden[at / 2..(at + n) / 2], base: from, length: bus.rdram.len() as u32 };
            let shadow = &bus.dp.multiple.0;
            let scaled = (job.scale > 1).then(|| View::whole(&shadow.rdram, &shadow.hidden));
            let pictures = shadow.gpu.as_ref().map(|g| g.pictures());
            out.shown_from_device = walk(&mut out.walker, None, &mut out.rasters, &job, view, scaled, out.capture.device_scanned, out.device_raster, pictures.as_deref());
            true
        }
        None => false,
    };
    let pictures = bus.dp.multiple.gpu.as_ref().map(|g| g.pictures());
    compose(&bus.vi, out, pictures.as_deref());
    Presented { walked, replaced: true }
}

/// `MarsCore.PresentDeferred`: the last walk joined, then this scan prepared and captured here and walked and composed on the presenter.
pub fn present_deferred(bus: &mut MemoryBus, out: &mut Scanout) -> Presented {
    let replaced = out.join();
    out.ensure_raster();
    let (vi, dp) = (&mut bus.vi, &mut bus.dp);
    let (job, edited) = prepare(vi, Some(dp), out);
    out.raster_edited |= edited;
    let (rows, serrate, repeat_rows) = (frame_height(&bus.vi.registers), serrate(&bus.vi.registers), out.repeat_rows);

    if let Some(job) = job {
        let (from, count) = reach(&job, bus.rdram.len() as u32);
        bus.dp.wait_read_range(from, count, site::VI);
        let can_scan_out = bus.dp.multiple.can_scan_out();
        let shape = Shape { job, from, count, can_scan_out };
        let repeats = out.capture.repeats(shape, bus);
        out.repeated_captures += repeats as i64;
        out.capture.device_scanned = false;
        if repeats {
            // The device captures no scaled bytes, so its repeat is its last picture, walked again only if another scan has replaced it (Mars_Gpu.md §14);
            // a repeat over an edited raster is walked as well, which C# does not do (Mars_Native.md §5.6.5), so it takes the same path (§6.4.4).
            if (out.walk_repeats || out.raster_edited) && job.scale > 1 && can_scan_out {
                if out.capture.device_scan_at == bus.dp.multiple.scan_outs && !(out.device_raster && out.raster_edits_pending()) {
                    out.capture.device_scanned = true;
                } else {
                    device_scan(bus, out, &job);
                }
            }
        } else {
            out.capture.take(bus, from, count);
            if job.scale > 1 {
                if can_scan_out {
                    // The device walks its own memory here, where it is idle, so nothing of that memory needs copying (Mars_Gpu.md §13).
                    device_scan(bus, out, &job);
                } else {
                    let (scaled_from, scaled_count) = reach_scaled(&job, bus.dp.multiple.rdram.len() as u32);
                    bus.dp.read_back_scaled(scaled_from, scaled_count);
                    let shadow = &bus.dp.multiple.0;
                    out.capture.take_scaled(&shadow.rdram, &shadow.hidden, scaled_from, scaled_count);
                }
            }
        }
        out.capture.shape = Some(shape);

        // A repeat writes the raster the last walk left, which the shown picture holds unless a border or blank has changed it since.
        if repeats && !out.walk_repeats && !out.raster_edited {
            out.repeated_scans += 1;
            return Presented { walked: false, replaced };
        }
    }

    out.raster_edited = false;
    let work = Work {
        job,
        capture: std::mem::take(&mut out.capture),
        rasters: std::mem::take(&mut out.rasters),
        walker: std::mem::take(&mut out.walker),
        frame: std::mem::take(&mut out.pending),
        rows,
        serrate,
        repeat_rows,
        average: out.average,
        device_raster: out.device_raster,
        shown_from_device: out.shown_from_device,
        pictures: bus.dp.multiple.gpu.as_ref().map(|g| g.pictures()),
        shown: (0, 0, 0),
        fault: None,
        nanos: 0,
        walked_in: 0,
        bands: if out.bands == 0 { default_bands() } else { out.bands },
        #[cfg(test)]
        hold: out.hold_band.clone(),
    };
    out.presenter.get_or_insert_with(Presenter::start).submit(Box::new(work));
    Presented { walked: job.is_some(), replaced }
}

/// `Vi.Walk`'s head: the raster at the multiple made, then the device's picture written into it, or the device's averaged raster shown as it is,
/// or the walk on the processor, in bands when a presenter's helpers are given; returns `_shownFromDevice`.
#[allow(clippy::too_many_arguments)]
fn walk(walker: &mut Walker, bands: Option<(&mut Bands, usize, Hold)>, rasters: &mut Rasters, job: &Job, view: View, scaled: Option<View>, device_scanned: bool, device_raster: bool, pictures: Option<&Pictures>) -> bool {
    let scale = job.scale;
    rasters.output_scale = scale;
    let raster_width = RASTER_WIDTH as i32 * scale;
    let raster = rasters.at(scale);
    if device_scanned && device_raster {
        return true;
    }
    if device_scanned {
        let picture = job.scaled_picture;
        pictures.expect("a device scanned").scanned(|words| write_device_picture(&picture, raster, raster_width, words));
        return false;
    }
    match bands {
        #[cfg(test)]
        Some((pool, count, hold)) => pool.walk(walker, count, job, view, scaled, rasters, hold),
        #[cfg(not(test))]
        Some((pool, count, _)) => pool.walk(walker, count, job, view, scaled, rasters),
        None => walker.walk(job, view, scaled, rasters),
    }
    false
}

/// A test's hold on a band, or nothing outside tests.
#[cfg(test)]
pub(super) type Hold = bands::Hold;
#[cfg(not(test))]
pub(super) type Hold = ();

/// `Vi.WriteDevicePicture`: the device's words into the raster as the walk writes them: a shown pixel whole, a dark one's colour cleared and its coverage kept.
fn write_device_picture(picture: &Picture, raster: &mut [u8], raster_width: i32, words: &[u32]) {
    let limit = raster.len() as i64 / 4;
    for row in 0..picture.rows {
        let line = picture.top as i64 * picture.stride as i64 + picture.left as i64 + if picture.lower { raster_width as i64 } else { 0 } + picture.stride as i64 * row as i64;
        let from = row as i64 * picture.columns as i64;
        for column in 0..picture.columns {
            let pixel = line + column as i64;
            if pixel < 0 || pixel >= limit {
                continue;
            }
            let at = pixel as usize * 4;
            let shown = column >= picture.first_column && column < picture.last_column;
            if shown {
                let word = words.get((from + column as i64) as usize).copied().unwrap_or(0);
                raster[at..at + 4].copy_from_slice(&word.to_le_bytes());
            } else {
                raster[at] = 0;
                raster[at + 1] = 0;
                raster[at + 2] = 0;
            }
        }
    }
}

/// `Vi.DeviceScan`: the walk's parameters for the device, which walks exactly what `Walk` would over the same memory (Mars_Gpu.md §13); into the raster when it averages there.
fn device_scan(bus: &mut MemoryBus, out: &mut Scanout, job: &Job) {
    if out.device_raster {
        publish_raster(bus, out, Some(job));
        return;
    }
    bus.dp.scan_out(&scan_parameters_of(job));
    out.capture.device_scanned = true;
    out.capture.device_scan_at = bus.dp.multiple.scan_outs;
}

fn scan_parameters_of(job: &Job) -> ScanParameters {
    let (p, n) = (job.scaled_picture, job.scale);
    ScanParameters {
        origin: job.aligned_origin().wrapping_mul((n * n) as u32),
        width: (job.width * n) as u32,
        flags: (if job.wide { SCAN_WIDE } else { 0 }) | (if job.resample { SCAN_RESAMPLE } else { 0 }) | (if job.divot { SCAN_DIVOT } else { 0 }) | (if job.dither { SCAN_DITHER } else { 0 }) | (if job.gamma { SCAN_GAMMA } else { 0 }),
        start_x: p.start_x,
        step_x: p.step_x,
        start_y: p.start_y,
        step_y: p.step_y,
        rows: p.rows.max(0) as u32,
        columns: p.columns.max(0) as u32,
        anti_alias: job.anti_alias as u32,
        ..Default::default()
    }
}

/// `Vi.PublishRaster`: the clears, the walk if there is one, and the average, as one submission the deferred thread waits for (Mars_Gpu.md §15).
fn publish_raster(bus: &mut MemoryBus, out: &mut Scanout, job: Option<&Job>) {
    let n = bus.dp.scale() as usize;
    let (width, height) = (RASTER_WIDTH * n, RASTER_HEIGHT * n);
    let holds = bus.dp.multiple.gpu.as_ref().is_some_and(|g| g.holds_raster(width, height));
    // Entering, the host's raster seeds the device's; leaving, the host's is stale and is cleared, as a blank would (Mars_Gpu.md §15).
    let seeding = !holds && !out.seed_stale && out.rasters.scale as usize == n && out.rasters.scaled.len() == width * height * 4;
    out.seed_stale = true;
    let mut scan = ScanParameters::default();
    if let Some(job) = job {
        let p = job.scaled_picture;
        scan = scan_parameters_of(job);
        scan.line_base = (p.top * p.stride + p.left + if p.lower { width as i32 } else { 0 }) as u32;
        scan.stride = p.stride as u32;
        scan.first_column = p.first_column;
        scan.last_column = p.last_column;
    }
    let seed: &[u8] = if seeding { &out.rasters.scaled } else { &[] };
    bus.dp.scan_into_raster(width, height, out.average as usize, seed, out.device_clear, &out.device_spans, job.is_some(), &scan);
    out.device_spans.clear();
    out.device_clear = false;
    if job.is_some() {
        out.capture.device_scanned = true;
        out.capture.device_scan_at = bus.dp.multiple.scan_outs;
    }
}

/// `Vi.Reach` narrowed to the walker's own reads: its window's lines two above the first line to three below the last, and a sample's
/// neighbours a line and three pixels past the last column below the last line; C#'s slack of lines and row spans is not kept (Mars_Native.md §6.14).
fn reach(job: &Job, length: u32) -> (u32, u32) {
    let picture = &job.picture;
    let (width, bytes) = (job.width as i64, if job.wide { 4 } else { 2 });
    let origin = job.aligned_origin() as i64;
    let line = |row: i64| (picture.start_y as i64 + row * picture.step_y as i64) >> 10;
    let (top, bottom) = (line(0), line((picture.rows - 1).max(0) as i64));
    let last_column = (picture.start_x as i64 + (picture.columns - 1).max(0) as i64 * picture.step_x as i64) >> 10;
    let first = ((top - 2) * width).min((top - 1) * width - 2);
    let last = ((bottom + 4) * width - 1).max((bottom + 2) * width + last_column + 3);
    let from = (origin + first * bytes).clamp(0, length as i64) & !1;
    let to = (origin + (last + 1) * bytes).clamp(from, length as i64);
    (from as u32, (to - from) as u32)
}

/// `Vi.ReachScaled`: the shadow's bytes a walk at the multiple can reach, by the same rule over the scaled origin, width and lines (Mars_Video.md §2.9).
fn reach_scaled(job: &Job, length: u32) -> (u32, u32) {
    let picture = &job.scaled_picture;
    let n = job.scale as i64;
    let (width, bytes) = (job.width as i64 * n, if job.wide { 4 } else { 2 });
    let origin = job.aligned_origin() as i64 * n * n;
    let row_span = walker::ROW_SPAN as i64;
    let first_line = (picture.start_y >> 10) as i64 - 3;
    let last_line = ((picture.start_y as i64 + (picture.rows - 1).max(0) as i64 * picture.step_y as i64) >> 10) + 4;
    let from = origin + ((first_line - 1) * width - row_span * n - 4) * bytes;
    let to = origin + ((last_line + 2) * width + 2 * row_span * n + 4) * bytes;
    let from = from.clamp(0, length as i64) & !1;
    let to = to.clamp(from, length as i64);
    (from as u32, (to - from) as u32)
}

fn serrate(r: &[u32; 14]) -> bool {
    r[CONTROL] & (1 << 6) != 0
}

fn is_pal(r: &[u32; 14]) -> bool {
    (r[VERTICAL_SYNC] & 0x3FF) as i32 > NTSC_SYNC_LINES + 25
}

/// `Vi.FrameHeight`.
fn frame_height(r: &[u32; 14]) -> usize {
    (if is_pal(r) { PAL_HEIGHT } else { NTSC_HEIGHT }) >> if serrate(r) { 0 } else { 1 }
}

/// `Vi.Prepare`: the device followed, then the geometry, the blank rule and the borders; the job when a walk is due, and whether a raster changed.
/// Without an interface (the scan-out's own tests) the scale is one and there is no device.
fn prepare(vi: &mut Vi, mut dp: Option<&mut DpInterface>, out: &mut Scanout) -> (Option<Job>, bool) {
    let (scale, drawn_scale, can_scan_out) = match dp.as_deref_mut() {
        Some(dp) => (shown_scale(dp), dp.scale(), dp.multiple.can_scan_out()),
        None => (1, 1, false),
    };

    // `FollowTheDevice`: entering, the host's raster seeds the device's; leaving, the host's is stale and is cleared, as a blank would (Mars_Gpu.md §15).
    let device = out.average > 1 && can_scan_out && drawn_scale % out.average == 0;
    if out.device_raster && !device {
        out.rasters.scaled.fill(0);
        out.device_spans.clear();
        out.device_clear = false;
        out.shown_from_device = false;
        out.seed_stale = false;
    }
    out.device_raster = device;

    let mut canvas = Canvas { rasters: &mut out.rasters, edited: false, device: device.then_some(DeviceEdits { spans: &mut out.device_spans, scale: drawn_scale as usize }) };
    let (job, signal) = prepare_on(vi, &mut canvas, scale, &mut out.device_clear);
    let edited = canvas.edited;

    if let Some(dp) = dp
        && out.device_raster
    {
        // Nothing is walked, but what was darkened must still reach the raster the device averages; and a list too long for one submission goes early.
        if (job.is_none() && !signal && out.raster_edits_pending()) || out.device_spans.len() / 2 >= MAX_SPANS - drawn_scale as usize {
            publish_raster_on(dp, out, None);
        }
    }
    (job, edited)
}

/// `publish_raster` with the interface alone, for the clears of a scan with no signal.
fn publish_raster_on(dp: &mut DpInterface, out: &mut Scanout, job: Option<&Job>) {
    let n = dp.scale() as usize;
    let (width, height) = (RASTER_WIDTH * n, RASTER_HEIGHT * n);
    let holds = dp.multiple.gpu.as_ref().is_some_and(|g| g.holds_raster(width, height));
    let seeding = !holds && !out.seed_stale && out.rasters.scale as usize == n && out.rasters.scaled.len() == width * height * 4;
    out.seed_stale = true;
    let seed: &[u8] = if seeding { &out.rasters.scaled } else { &[] };
    let scan = ScanParameters::default();
    dp.scan_into_raster(width, height, out.average as usize, seed, out.device_clear, &out.device_spans, job.is_some(), &scan);
    out.device_spans.clear();
    out.device_clear = false;
}

/// The job when a walk is due, and whether the picture carries a signal.
fn prepare_on(vi: &mut Vi, raster: &mut Canvas, scale: i32, device_clear: &mut bool) -> (Option<Job>, bool) {
    let r = vi.registers;
    let origin = r[ORIGIN] & 0xFF_FFFF;
    if origin == 0 {
        return (None, false);
    }
    let Some(picture) = measure(&r) else { return (None, false) };

    let kind = (r[CONTROL] & 3) as i32;
    let blank = kind & 2 == 0;
    if blank && vi.was_blank {
        return (None, false);
    }
    vi.was_blank = blank;

    let signal = picture.columns > 0 && picture.left < RASTER_WIDTH as i32;
    if blank {
        vi.held.fill(0);
        if raster.rasters.raster.iter().any(|&b| b != 0) {
            raster.rasters.raster.fill(0);
            raster.edited = true;
        }
        if let Some(device) = raster.device.as_mut() {
            *device_clear = true;
            device.spans.clear();
            raster.edited = true;
        } else if raster.rasters.scaled.iter().any(|&b| b != 0) {
            raster.rasters.scaled.fill(0);
            raster.edited = true;
        }
    } else {
        borders(&picture, signal, serrate(&r), &mut vi.held[..], raster);
    }
    if !signal {
        return (None, false);
    }

    let anti_alias = ((r[CONTROL] >> 8) & 3) as i32;
    let scale = scale.max(1);
    let job = Job {
        picture,
        origin,
        width: (r[WIDTH] & 0xFFF) as i32,
        wide: kind & 1 != 0,
        resample: anti_alias != REPLICATE,
        divot: r[CONTROL] & (1 << 4) != 0,
        anti_alias,
        dither: r[CONTROL] & (1 << 16) != 0,
        gamma: r[CONTROL] & (1 << 3) != 0,
        scale,
        scaled_picture: if scale > 1 { picture.at(scale) } else { picture },
    };
    (Some(job), true)
}

/// `Vi.Measure`: the registers as lengths and steps, pulled back inside the raster (Mars_Video.md §2.1).
fn measure(r: &[u32; 14]) -> Option<Picture> {
    let mut top = ((r[VERTICAL_START] >> 16) & 0x3FF) as i32;
    let mut left = ((r[HORIZONTAL_START] >> 16) & 0x3FF) as i32;
    let mut columns = (r[HORIZONTAL_START] & 0x3FF) as i32 - left;
    let rows = ((r[VERTICAL_START] & 0x3FF) as i32 - top) >> 1;

    let step_x = r[SCALE_X] & 0xFFF;
    let mut start_x = (r[SCALE_X] >> 16) & 0xFFF;
    let step_y = r[SCALE_Y] & 0xFFF;
    let mut start_y = (r[SCALE_Y] >> 16) & 0xFFF;

    let pal = is_pal(r);
    left -= if pal { 128 } else { 108 };

    let left_clamped = left < 0;
    if left_clamped {
        start_x = start_x.wrapping_add(step_x.wrapping_mul((-left) as u32));
        columns += left;
        left = 0;
    }

    top = (top - if pal { 44 } else { 34 }) / 2;
    if top < 0 {
        start_y = start_y.wrapping_add(step_y.wrapping_mul((-top) as u32));
        top = 0;
    }

    let columns_clamped = columns + left > RASTER_WIDTH as i32;
    if columns_clamped {
        columns = RASTER_WIDTH as i32 - left;
    }

    let mut active_lines = (r[VERTICAL_SYNC] & 0x3FF) as i32 - if pal { 44 } else { 34 };
    if active_lines < 0 {
        return None;
    }
    let serrate = serrate(r);
    if !serrate {
        active_lines >>= 1;
    }
    let lower = serrate && r[CURRENT_LINE] & 1 == 0;

    Some(Picture {
        left,
        top,
        columns,
        rows,
        stride: (RASTER_WIDTH as i32) << if serrate { 1 } else { 0 },
        active_lines,
        start_x,
        step_x,
        start_y,
        step_y,
        first_column: if left_clamped { 0 } else { 8 },
        last_column: if columns_clamped { columns } else { columns - 7 },
        lower,
    })
}

/// `Vi.Borders`: what the picture does not cover goes dark once its two frames of grace run out (Mars_Video.md §2.4).
fn borders(picture: &Picture, signal: bool, serrate: bool, held: &mut [i32], raster: &mut Canvas) {
    let width = RASTER_WIDTH as i32;
    let right = picture.columns + picture.left;

    if picture.left > 0 && picture.left < width {
        for line in 0..picture.active_lines {
            darken(raster, line, 0, picture.left);
        }
    }
    if (0..width).contains(&right) {
        for line in 0..picture.active_lines {
            darken(raster, line, right, width - right);
        }
    }

    let mut at = 0;
    while at < (picture.top << if serrate { 1 } else { 0 }) + picture.lower as i32 {
        fade(picture, signal, held, raster, at);
        at += 1;
    }

    for _ in 0..picture.rows {
        hold(signal, held, raster, at);
        if serrate {
            fade(picture, signal, held, raster, at + 1);
        }
        at += if serrate { 2 } else { 1 };
    }

    // C# indexes `_held` here unguarded and throws past its end, which only an interlaced sync above 669 half lines reaches; this stops there.
    while at < picture.active_lines && (at as usize) < RASTER_HEIGHT {
        let line = &mut held[at as usize];
        if *line != 0 {
            *line -= 1;
        }
        if *line == 0 {
            expire(picture, signal, raster, at);
        }
        at += 1;
    }
}

fn hold(signal: bool, held: &mut [i32], raster: &mut Canvas, line: i32) {
    if line as usize >= RASTER_HEIGHT {
        return;
    }
    let count = &mut held[line as usize];
    if signal {
        *count = 2;
    } else if *count != 0 {
        *count -= 1;
        if *count == 0 {
            darken(raster, line, 0, RASTER_WIDTH as i32);
        }
    }
}

fn fade(picture: &Picture, signal: bool, held: &mut [i32], raster: &mut Canvas, line: i32) {
    if line as usize >= RASTER_HEIGHT {
        return;
    }
    let count = &mut held[line as usize];
    if *count != 0 {
        *count -= 1;
        if *count == 0 {
            expire(picture, signal, raster, line);
        }
    }
}

/// `Vi.Expire`: a spent line keeps what lies outside the picture.
fn expire(picture: &Picture, signal: bool, raster: &mut Canvas, line: i32) {
    if signal {
        darken(raster, line, picture.left, picture.columns);
    } else {
        darken(raster, line, 0, RASTER_WIDTH as i32);
    }
}

/// `Vi.Darken`: the colour and coverage of `count` pixels cleared, and the same lines and columns of the raster at the multiple, each at
/// the multiple: on the host, or as spans for the device while it averages (`DarkenOnTheDevice`).
fn darken(raster: &mut Canvas, line: i32, from: i32, count: i32) {
    if line as usize >= RASTER_HEIGHT || count <= 0 {
        return;
    }
    let start = (line as usize * RASTER_WIDTH + from as usize) * 4;
    let span = &mut raster.rasters.raster[start..start + count as usize * 4];
    if span.iter().any(|&b| b != 0) {
        span.fill(0);
        raster.edited = true;
    }
    if let Some(device) = raster.device.as_mut() {
        let (n, width) = (device.scale, RASTER_WIDTH * device.scale);
        for i in 0..n {
            device.spans.push(((line as usize * n + i) * width + from as usize * n) as u32);
            device.spans.push((count as usize * n) as u32);
        }
        raster.edited = true;
        return;
    }
    let n = raster.rasters.scale;
    if n > 1 && !raster.rasters.scaled.is_empty() {
        let (n, width) = (n as usize, RASTER_WIDTH * n as usize);
        for i in 0..n {
            let start = ((line as usize * n + i) * width + from as usize * n) * 4;
            let span = &mut raster.rasters.scaled[start..start + count as usize * n * 4];
            if span.iter().any(|&b| b != 0) {
                span.fill(0);
                raster.edited = true;
            }
        }
    }
}

/// `MarsCore.Compose`: the raster's first rows with the coverage byte made opaque, each repeated when asked; at a multiple the raster the walk wrote, averaged down when asked.
fn compose(vi: &Vi, out: &mut Scanout, pictures: Option<&Pictures>) {
    let (rows, serrate) = (frame_height(&vi.registers), serrate(&vi.registers));
    let shown = compose_from(&mut out.rasters, rows, serrate, out.repeat_rows, out.average, out.shown_from_device, pictures, &mut out.frame);
    (out.width, out.height, out.row_repeat) = shown;
}

/// `compose_into`, reading the device's averaged raster when that is what is shown.
#[allow(clippy::too_many_arguments)]
pub(super) fn compose_from(rasters: &mut Rasters, rows: usize, serrate: bool, repeat_rows: bool, average: i32, shown_from_device: bool, pictures: Option<&Pictures>, frame: &mut Vec<u8>) -> (u32, u32, u32) {
    match (shown_from_device, pictures) {
        (true, Some(pictures)) => {
            let mut shown = (0, 0, 0);
            pictures.averaged(|words| shown = compose_into(rasters, rows, serrate, repeat_rows, average, Some(words), frame));
            shown
        }
        _ => compose_into(rasters, rows, serrate, repeat_rows, average, None, frame),
    }
}

/// `Vi.BoxAverage`: each pixel of the result is the rounded mean of a square of the source, channel by channel (Mars_Video.md §2.10).
pub fn box_average(source: &[u8], source_width: usize, side: usize, into: &mut [u8]) {
    let width = source_width / side;
    let lines = into.len() / (width * 4);
    let area = side * side;
    let half = area / 2;
    let mut sums = vec![0u32; width * 4];
    for y in 0..lines {
        sums.fill(0);
        for j in 0..side {
            let row = &source[(y * side + j) * source_width * 4..(y * side + j + 1) * source_width * 4];
            for x in 0..width {
                for i in 0..side {
                    let at = (x * side + i) * 4;
                    sums[x * 4] += row[at] as u32;
                    sums[x * 4 + 1] += row[at + 1] as u32;
                    sums[x * 4 + 2] += row[at + 2] as u32;
                    sums[x * 4 + 3] += row[at + 3] as u32;
                }
            }
        }
        let line = &mut into[y * width * 4..(y + 1) * width * 4];
        for (out, sum) in line.iter_mut().zip(&sums) {
            *out = ((sum + half as u32) / area as u32) as u8;
        }
    }
}

/// `Vi.Raster` then `Compose`: the raster the last walk wrote, at one or at the multiple, averaged down when the multiple divides by the
/// averaging (`Averaging`), or the device's averaged raster when that is what is shown; returns the frame's width, height and row repeat.
pub(super) fn compose_into(rasters: &mut Rasters, rows: usize, serrate: bool, repeat_rows: bool, average: i32, device_averaged: Option<&[u32]>, frame: &mut Vec<u8>) -> (u32, u32, u32) {
    let repeat = if serrate || !repeat_rows { 1 } else { 2 };
    let output_scale = rasters.output_scale.max(1);
    let averaging = if output_scale > 1 && average > 1 && output_scale % average == 0 { average } else { 1 };
    let shown_scale = (output_scale / averaging) as usize;
    let width = RASTER_WIDTH * shown_scale;
    let rows = rows * shown_scale;
    let row_bytes = width * 4;
    let needed = row_bytes * rows;
    let device: Option<&[u8]> = match device_averaged {
        Some(words) if averaging > 1 => {
            // SAFETY: the device's words are four little-endian bytes a pixel, R, G, B, coverage, as the raster holds them.
            let bytes = unsafe { std::slice::from_raw_parts(words.as_ptr().cast::<u8>(), words.len() * 4) };
            Some(&bytes[..needed.min(bytes.len())])
        }
        _ => None,
    };
    if averaging > 1 && device.is_none() {
        if rasters.averaged.len() < needed {
            rasters.averaged = vec![0; needed];
        }
        box_average(&rasters.scaled, RASTER_WIDTH * output_scale as usize, averaging as usize, &mut rasters.averaged[..needed]);
    }
    let raster: &[u8] = if let Some(device) = device {
        device
    } else if output_scale == 1 {
        &rasters.raster
    } else if averaging == 1 {
        &rasters.scaled
    } else {
        &rasters.averaged
    };
    frame.resize(row_bytes * rows * repeat, 0);

    for row in 0..rows {
        let from = &raster[row * row_bytes..(row + 1) * row_bytes];
        let at = row * repeat * row_bytes;
        let into = &mut frame[at..at + row_bytes];
        for (to, pixel) in into.as_chunks_mut::<4>().0.iter_mut().zip(from.as_chunks::<4>().0) {
            *to = [pixel[0], pixel[1], pixel[2], 0xFF];
        }
        for copy in 1..repeat {
            frame.copy_within(at..at + row_bytes, at + copy * row_bytes);
        }
    }

    (width as u32, (rows * repeat) as u32, if serrate || repeat_rows { 1 } else { 2 })
}

/// The device's pictures for a presenter, kept alive by the reference.
pub(super) type SharedPictures = Option<Arc<Pictures>>;
