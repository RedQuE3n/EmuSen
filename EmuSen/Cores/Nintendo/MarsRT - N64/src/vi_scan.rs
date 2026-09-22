//! The VI's scan-out at one, immediate and on one thread: C#'s `Vi.Scan` then `MarsCore.Compose`. See Mars_Native.md §5.4.

mod filters;
#[cfg(test)]
mod tests;
mod walker;

use crate::vi::Vi;
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

/// C#'s raster and composed frame, which no state holds.
#[derive(Default)]
pub struct Scanout {
    /// The composed RGBA frame, `width * height * 4` bytes.
    pub frame: Vec<u8>,
    pub width: u32,
    pub height: u32,
    /// C#'s `RowRepeat`: how many times each row of `frame` is shown.
    pub row_repeat: u32,
    /// C#'s `RepeatRows`; false, as Mistress sets it, sends a progressive field's rows once.
    pub repeat_rows: bool,
    /// `Vi._raster`: four bytes a pixel, the fourth coverage, kept between scans and across a load, as C# keeps it.
    raster: Vec<u8>,
    walker: Walker,
}

impl Scanout {
    /// The raster, 640 by 625, as C#'s `Vi._raster` holds it; empty before the first scan.
    pub fn raster(&self) -> &[u8] {
        &self.raster
    }
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

/// C#'s `Vi.Scan` then `MarsCore.Compose`. True when a walk ran, as `Scan()` returns; the frame is composed from the raster either way, as `Present` composes it.
pub fn scan(vi: &mut Vi, rdram: &[u8], hidden: &[u8], out: &mut Scanout) -> bool {
    if out.raster.len() != RASTER_BYTES {
        out.raster = vec![0; RASTER_BYTES];
    }
    let walked = match prepare(vi, &mut out.raster) {
        Some(job) => {
            out.walker.walk(&job, rdram, hidden, &mut out.raster);
            true
        }
        None => false,
    };
    compose(vi, out);
    walked
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

/// `Vi.Prepare` at one with no device: the geometry, the blank rule and the borders; the job when a walk is due.
fn prepare(vi: &mut Vi, raster: &mut [u8]) -> Option<Job> {
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
        raster.fill(0);
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
fn borders(picture: &Picture, signal: bool, serrate: bool, held: &mut [i32], raster: &mut [u8]) {
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

fn hold(signal: bool, held: &mut [i32], raster: &mut [u8], line: i32) {
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

fn fade(picture: &Picture, signal: bool, held: &mut [i32], raster: &mut [u8], line: i32) {
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
fn expire(picture: &Picture, signal: bool, raster: &mut [u8], line: i32) {
    if signal {
        darken(raster, line, picture.left, picture.columns);
    } else {
        darken(raster, line, 0, RASTER_WIDTH as i32);
    }
}

/// `Vi.Darken` at one: the colour and coverage of `count` pixels cleared.
fn darken(raster: &mut [u8], line: i32, from: i32, count: i32) {
    if line as usize >= RASTER_HEIGHT || count <= 0 {
        return;
    }
    let start = (line as usize * RASTER_WIDTH + from as usize) * 4;
    raster[start..start + count as usize * 4].fill(0);
}

/// `MarsCore.Compose` at one: the raster's first rows with the coverage byte made opaque, each repeated when asked.
fn compose(vi: &Vi, out: &mut Scanout) {
    let serrate = serrate(&vi.registers);
    let rows = frame_height(&vi.registers);
    let repeat = if serrate || !out.repeat_rows { 1 } else { 2 };
    let row_bytes = RASTER_WIDTH * 4;
    out.frame.resize(row_bytes * rows * repeat, 0);

    for row in 0..rows {
        let from = &out.raster[row * row_bytes..(row + 1) * row_bytes];
        let at = row * repeat * row_bytes;
        let into = &mut out.frame[at..at + row_bytes];
        for (to, pixel) in into.as_chunks_mut::<4>().0.iter_mut().zip(from.as_chunks::<4>().0) {
            *to = [pixel[0], pixel[1], pixel[2], 0xFF];
        }
        for copy in 1..repeat {
            out.frame.copy_within(at..at + row_bytes, at + copy * row_bytes);
        }
    }

    out.width = RASTER_WIDTH as u32;
    out.height = (rows * repeat) as u32;
    out.row_repeat = if serrate || out.repeat_rows { 1 } else { 2 };
}
