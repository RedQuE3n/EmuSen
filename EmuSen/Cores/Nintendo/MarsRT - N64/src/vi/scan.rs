//! The VI's scan-out at one: C#'s `Vi.Scan` then `MarsCore.Compose`, at once or deferred to a presenter thread. See Mars_Native.md §5.4 and §5.6.

mod filters;
mod presenter;
#[cfg(test)]
mod tests;
mod walker;

use crate::memory::bus::MemoryBus;
use crate::memory::dp_threads::site;
use crate::vi::Vi;
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
    /// `Vi._raster`: four bytes a pixel, the fourth coverage, kept between scans and across a load, as C# keeps it.
    raster: Vec<u8>,
    walker: Walker,
    /// `ScanJob`'s capture: the bytes the last deferred walk read, and its geometry.
    capture: Capture,
    /// A border or a blank changed the raster since the last walk, so a repeated scan may not be skipped (Mars_Native.md §5.6.5).
    raster_edited: bool,
    /// The frame the next deferred walk composes into, swapped with `frame` at the join.
    pending: Vec<u8>,
    presenter: Option<Presenter>,
}

impl Scanout {
    /// The raster, 640 by 625, as C#'s `Vi._raster` holds it; empty before the first scan and while a walk is out.
    pub fn raster(&self) -> &[u8] {
        &self.raster
    }

    fn ensure_raster(&mut self) {
        if self.raster.len() != RASTER_BYTES {
            self.raster = vec![0; RASTER_BYTES];
        }
    }

    /// `ScanJob.Forget`: a loaded state's first deferred scan walks, since the raster it presents is not the last walk's.
    pub fn forget(&mut self) {
        self.capture.shape = None;
    }

    /// `JoinPresentation`: the walk out, if one is, finished and its picture made the shown one; true when it was.
    pub fn join(&mut self) -> bool {
        let Some(work) = self.presenter.as_mut().and_then(Presenter::take) else { return false };
        let work = *work;
        if let Some(fault) = work.fault {
            panic!("the deferred scan-out failed: {fault}");
        }
        self.raster = work.raster;
        self.walker = work.walker;
        self.capture = work.capture;
        self.pending = std::mem::replace(&mut self.frame, work.frame);
        (self.width, self.height, self.row_repeat) = work.shown;
        true
    }

    /// True while a deferred walk is out.
    pub fn in_flight(&self) -> bool {
        self.presenter.as_ref().is_some_and(Presenter::busy)
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

/// `ScanJob`'s capture: the lines a walk can reach, copied out of RDRAM with their hidden bits, and the geometry they were taken for.
#[derive(Default)]
struct Capture {
    rdram: Vec<u8>,
    hidden: Vec<u8>,
    from: u32,
    count: u32,
    length: u32,
    shape: Option<Shape>,
}

/// `Repeated`'s shape: everything the walk reads besides the bytes.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Shape {
    job: Job,
    from: u32,
    count: u32,
}

impl Capture {
    fn view(&self) -> View<'_> {
        View { rdram: &self.rdram[..self.count as usize], hidden: &self.hidden[..(self.count / 2) as usize], base: self.from, length: self.length }
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
}

/// The raster as a scan's borders edit it, noting whether any byte changed.
struct Canvas<'a> {
    bytes: &'a mut [u8],
    edited: bool,
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

/// `ScanJob`'s fields the walk at one reads.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Job {
    picture: Picture,
    origin: u32,
    width: i32,
    wide: bool,
    resample: bool,
    divot: bool,
    anti_alias: i32,
    dither: bool,
    gamma: bool,
}

/// C#'s `Vi.Scan` then `MarsCore.Compose` over whole, unshared memory. True when a walk ran; the frame is composed either way, as `Present` composes it.
pub fn scan(vi: &mut Vi, rdram: &[u8], hidden: &[u8], out: &mut Scanout) -> bool {
    out.join();
    out.ensure_raster();
    let walked = match prepare(vi, &mut out.raster).0 {
        Some(job) => {
            out.walker.walk(&job, View::whole(rdram, hidden), &mut out.raster);
            true
        }
        None => false,
    };
    compose(vi, out);
    walked
}

/// `MarsCore.Present`: the scan at once; with a drain running, the walk waits for the lines it can reach (site 8) and reads those alone.
pub fn present_now(bus: &mut MemoryBus, out: &mut Scanout) -> Presented {
    if bus.dp.threads.is_none() {
        let walked = scan(&mut bus.vi, &bus.rdram, &bus.rdram_hidden, out);
        return Presented { walked, replaced: true };
    }
    out.join();
    out.ensure_raster();
    let walked = match prepare(&mut bus.vi, &mut out.raster).0 {
        Some(job) => {
            let (from, count) = reach(&job, bus.rdram.len() as u32);
            bus.dp.wait_read_range(from, count, site::VI);
            let (at, n) = (from as usize, count as usize);
            let view = View { rdram: &bus.rdram[at..at + n], hidden: &bus.rdram_hidden[at / 2..(at + n) / 2], base: from, length: bus.rdram.len() as u32 };
            out.walker.walk(&job, view, &mut out.raster);
            true
        }
        None => false,
    };
    compose(&bus.vi, out);
    Presented { walked, replaced: true }
}

/// `MarsCore.PresentDeferred`: the last walk joined, then this scan prepared and captured here and walked and composed on the presenter.
pub fn present_deferred(bus: &mut MemoryBus, out: &mut Scanout) -> Presented {
    let replaced = out.join();
    out.ensure_raster();
    let (job, edited) = prepare(&mut bus.vi, &mut out.raster);
    out.raster_edited |= edited;
    let (rows, serrate, repeat_rows) = (frame_height(&bus.vi.registers), serrate(&bus.vi.registers), out.repeat_rows);

    if let Some(job) = job {
        let (from, count) = reach(&job, bus.rdram.len() as u32);
        bus.dp.wait_read_range(from, count, site::VI);
        let shape = Shape { job, from, count };
        let repeats = !out.walk_repeats && out.capture.repeats(shape, bus);
        if !repeats {
            out.capture.take(bus, from, count);
        }
        out.capture.shape = Some(shape);

        // A repeat writes the raster the last walk left, which the shown picture holds unless a border or blank has changed it since.
        if repeats && !out.raster_edited {
            out.repeated_scans += 1;
            return Presented { walked: false, replaced };
        }
    }

    out.raster_edited = false;
    let work = Work {
        job,
        capture: std::mem::take(&mut out.capture),
        raster: std::mem::take(&mut out.raster),
        walker: std::mem::take(&mut out.walker),
        frame: std::mem::take(&mut out.pending),
        rows,
        serrate,
        repeat_rows,
        shown: (0, 0, 0),
        fault: None,
    };
    out.presenter.get_or_insert_with(Presenter::start).submit(Box::new(work));
    Presented { walked: job.is_some(), replaced }
}

/// `Vi.Reach`: from a line and a row's span before the window's first line to two of each after its last, clamped (Mars_Video.md §2.7).
fn reach(job: &Job, length: u32) -> (u32, u32) {
    let picture = &job.picture;
    let (width, bytes) = (job.width as i64, if job.wide { 4 } else { 2 });
    let origin = if job.wide { job.origin & 0xFF_FFFC } else { job.origin & 0xFF_FFFE } as i64;
    let row_span = walker::ROW_SPAN as i64;
    let first_line = (picture.start_y >> 10) as i64 - 3;
    let last_line = ((picture.start_y as i64 + (picture.rows - 1).max(0) as i64 * picture.step_y as i64) >> 10) + 4;
    let from = origin + ((first_line - 1) * width - row_span - 4) * bytes;
    let to = origin + ((last_line + 2) * width + 2 * row_span + 4) * bytes;
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

/// `Vi.Prepare` at one with no device: the geometry, the blank rule and the borders; the job when a walk is due, and whether the raster changed.
fn prepare(vi: &mut Vi, raster: &mut [u8]) -> (Option<Job>, bool) {
    let mut canvas = Canvas { bytes: raster, edited: false };
    let job = prepare_on(vi, &mut canvas);
    (job, canvas.edited)
}

fn prepare_on(vi: &mut Vi, raster: &mut Canvas) -> Option<Job> {
    let r = vi.registers;
    let origin = r[ORIGIN] & 0xFF_FFFF;
    if origin == 0 {
        return None;
    }
    let picture = measure(&r)?;

    let kind = (r[CONTROL] & 3) as i32;
    let blank = kind & 2 == 0;
    if blank && vi.was_blank {
        return None;
    }
    vi.was_blank = blank;

    let signal = picture.columns > 0 && picture.left < RASTER_WIDTH as i32;
    if blank {
        vi.held.fill(0);
        if raster.bytes.iter().any(|&b| b != 0) {
            raster.bytes.fill(0);
            raster.edited = true;
        }
    } else {
        borders(&picture, signal, serrate(&r), &mut vi.held[..], raster);
    }
    if !signal {
        return None;
    }

    let anti_alias = ((r[CONTROL] >> 8) & 3) as i32;
    Some(Job {
        picture,
        origin,
        width: (r[WIDTH] & 0xFFF) as i32,
        wide: kind & 1 != 0,
        resample: anti_alias != REPLICATE,
        divot: r[CONTROL] & (1 << 4) != 0,
        anti_alias,
        dither: r[CONTROL] & (1 << 16) != 0,
        gamma: r[CONTROL] & (1 << 3) != 0,
    })
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

/// `Vi.Darken` at one: the colour and coverage of `count` pixels cleared.
fn darken(raster: &mut Canvas, line: i32, from: i32, count: i32) {
    if line as usize >= RASTER_HEIGHT || count <= 0 {
        return;
    }
    let start = (line as usize * RASTER_WIDTH + from as usize) * 4;
    let span = &mut raster.bytes[start..start + count as usize * 4];
    if span.iter().any(|&b| b != 0) {
        span.fill(0);
        raster.edited = true;
    }
}

/// `MarsCore.Compose` at one: the raster's first rows with the coverage byte made opaque, each repeated when asked.
fn compose(vi: &Vi, out: &mut Scanout) {
    let (rows, serrate) = (frame_height(&vi.registers), serrate(&vi.registers));
    (out.width, out.height, out.row_repeat) = compose_into(&out.raster, rows, serrate, out.repeat_rows, &mut out.frame);
}

/// The composition from a raster into a frame; returns its width, height and row repeat.
fn compose_into(raster: &[u8], rows: usize, serrate: bool, repeat_rows: bool, frame: &mut Vec<u8>) -> (u32, u32, u32) {
    let repeat = if serrate || !repeat_rows { 1 } else { 2 };
    let row_bytes = RASTER_WIDTH * 4;
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

    (RASTER_WIDTH as u32, (rows * repeat) as u32, if serrate || repeat_rows { 1 } else { 2 })
}
