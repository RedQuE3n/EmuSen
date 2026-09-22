//! C#'s `Vi.Walker`: the rows of one walk, with the caches that make it cheap. Every cache is a pure function of memory and the registers. See Mars_Native.md §5.4.

use super::filters::{Pixel, divot, gamma, mix, pull, step};
use super::{Job, Picture, View};

/// `Vi.RowSpan`: every source pixel one row can reach, and the neighbour beyond.
const ROW_SPAN: i32 = super::RASTER_WIDTH as i32 * 4 + 2;
const WINDOW_LINES: i32 = 32;
/// `Vi.Covered`: above this anti-alias mode every pixel is whole.
const COVERED: i32 = 1;

/// What one walk reads: memory from `base`, memory's length, where the frame buffer starts, and how it is laid out.
struct Source<'a> {
    rdram: &'a [u8],
    hidden: &'a [u8],
    base: u32,
    length: u32,
    origin: u32,
    wide: bool,
    width: i32,
}

impl Source<'_> {
    /// `Walker.Within`: an address inside memory that the view does not hold is a defect in `reach`, said so rather than read stale.
    #[inline(always)]
    fn within(&self, address: u32, size: usize) -> usize {
        let index = address.wrapping_sub(self.base) as usize;
        if index + size > self.rdram.len() {
            panic!("the scan reached frame buffer address {address:X} outside the lines captured for it, {:X} for {:X} bytes", self.base, self.rdram.len());
        }
        index
    }

    /// `Walker.Fetch`: a read past memory is a whole zero (Mars_Video.md §2.6).
    #[inline(always)]
    fn fetch(&self, at: i32) -> Pixel {
        if self.wide {
            let address = self.origin.wrapping_add((at as u32).wrapping_mul(4));
            if address.wrapping_add(3) >= self.length {
                return Pixel::default();
            }
            let i = self.within(address, 4);
            let b = &self.rdram[i..i + 4];
            return Pixel { red: b[0] as i32, green: b[1] as i32, blue: b[2] as i32, coverage: ((b[3] >> 5) & 7) as i32 };
        }
        let word = self.origin.wrapping_add((at as u32).wrapping_mul(2));
        if word.wrapping_add(1) >= self.length {
            return Pixel::default();
        }
        let i = self.within(word, 2);
        let pixel = ((self.rdram[i] as i32) << 8) | self.rdram[i + 1] as i32;
        let coverage = ((pixel & 1) << 2) | self.hidden[i >> 1] as i32;
        Pixel { red: (pixel >> 8) & 0xF8, green: (pixel & 0x7C0) >> 3, blue: (pixel & 0x3E) << 2, coverage }
    }
}

/// One walker, kept between scans as C#'s `_walkers[0]` is; scratch, never state.
#[derive(Default)]
pub struct Walker {
    samples: Vec<Pixel>,
    sampled_row: Vec<i32>,
    row: i32,
    plain: Vec<Pixel>,
    plain_row: Vec<i32>,
    window: Vec<Pixel>,
    window_width: i32,
    window_from: i32,
    window_count: i32,
    slot_line: [i32; 2],
    slot_folded: [bool; 2],
    slot_live: [bool; 2],
    slot_stamp: [i32; 2],
    anti_alias: i32,
    dither: bool,
    gamma: bool,
}

impl Walker {
    /// `Vi.Walk` at one, over live memory or a capture of it, as one band.
    pub fn walk(&mut self, job: &Job, view: View, raster: &mut [u8]) {
        let aligned = if job.wide { job.origin & 0xFF_FFFC } else { job.origin & 0xFF_FFFE };
        let source = Source { rdram: view.rdram, hidden: view.hidden, base: view.base, length: view.length, origin: aligned, wide: job.wide, width: job.width };
        self.begin(job);
        self.rows(&job.picture, &source, job.resample, job.divot, raster, 0, job.picture.rows);
    }

    /// `Walker.Begin`: the modes, the slots emptied, and the window opened.
    fn begin(&mut self, job: &Job) {
        let span = 2 * ROW_SPAN as usize;
        if self.samples.len() != span {
            self.samples = vec![Pixel::default(); span];
            self.sampled_row = vec![0; span];
            self.plain = vec![Pixel::default(); span];
            self.plain_row = vec![0; span];
        }
        self.anti_alias = job.anti_alias;
        self.dither = job.dither;
        self.gamma = job.gamma;
        self.slot_live = [false; 2];
        if job.width != self.window_width {
            self.window = vec![Pixel::default(); (WINDOW_LINES * job.width) as usize];
            self.window_width = job.width;
        }
        self.window_count = 0;
    }

    /// `Walker.Rows`: from one row to another, into the raster.
    #[allow(clippy::too_many_arguments)]
    fn rows(&mut self, picture: &Picture, source: &Source, resample: bool, divot: bool, raster: &mut [u8], from: i32, to: i32) {
        let width = source.width;
        let raster_width = super::RASTER_WIDTH as i32;
        let first = (picture.start_x >> 10) as i32;
        let line_of = |row: i32| picture.start_y.wrapping_add((row as u32).wrapping_mul(picture.step_y)) >> 10;
        let mut bug = if from > 0 && line_of(from - 1) == line_of(from) { 2 } else { 0 };
        let length = raster.len() as i64;

        for row in from..to {
            let down = picture.start_y.wrapping_add((row as u32).wrapping_mul(picture.step_y));
            let source_at = width.wrapping_mul((down >> 10) as i32);
            let below = source_at.wrapping_add(width);
            let fraction_y = ((down >> 5) & 0x1F) as i32;
            let line = picture
                .top
                .wrapping_mul(picture.stride)
                .wrapping_add(picture.left)
                .wrapping_add(if picture.lower { raster_width } else { 0 })
                .wrapping_add(picture.stride.wrapping_mul(row));

            bug = if down >> 10 == line_of(row + 1) { 2 } else { bug >> 1 };
            self.lines(source, (down >> 10) as i32 - 2, (down >> 10) as i32 + 3);

            let here = self.slot(source_at, false, -1);
            let under = self.slot(below, bug == 1, here);

            let mut across = picture.start_x;
            for column in 0..picture.columns {
                let at = (across >> 10) as i32;
                let fraction_x = ((across >> 5) & 0x1F) as i32;
                across = across.wrapping_add(picture.step_x);

                let mut color = self.remembered(source, here, at - first, source_at.wrapping_add(at), 0, divot);
                if resample && fraction_x != 0 {
                    let mut next = self.remembered(source, here, at + 1 - first, source_at.wrapping_add(at + 1), 0, divot);
                    if fraction_y != 0 {
                        let lower = self.remembered(source, under, at - first, below.wrapping_add(at), bug, divot);
                        let lower_next = self.remembered(source, under, at + 1 - first, below.wrapping_add(at + 1), bug, divot);
                        color = mix(color, lower, fraction_y);
                        next = mix(next, lower_next, fraction_y);
                    }
                    color = mix(color, next, fraction_x);
                } else if resample && fraction_y != 0 {
                    let lower = self.remembered(source, under, at - first, below.wrapping_add(at), bug, divot);
                    color = mix(color, lower, fraction_y);
                }

                let pixel = (line as i64 + column as i64) * 4;
                if pixel < 0 || pixel + 3 >= length {
                    continue;
                }
                let pixel = pixel as usize;
                let shown = column >= picture.first_column && column < picture.last_column;
                if self.gamma {
                    color = gamma(color);
                }
                let out = &mut raster[pixel..pixel + 4];
                if shown {
                    out[0] = color.red as u8;
                    out[1] = color.green as u8;
                    out[2] = color.blue as u8;
                    out[3] = color.coverage as u8;
                } else {
                    out[0] = 0;
                    out[1] = 0;
                    out[2] = 0;
                }
            }
        }
    }

    /// `Walker.Slot`: the slot holding this line as this fold samples it, or the one the row's other line does not need.
    fn slot(&mut self, line: i32, folded: bool, avoid: i32) -> i32 {
        for slot in 0..2 {
            if self.slot_live[slot] && self.slot_line[slot] == line && self.slot_folded[slot] == folded {
                return slot as i32;
            }
        }
        let taken = if avoid == 0 { 1 } else { 0 };
        self.slot_line[taken] = line;
        self.slot_folded[taken] = folded;
        self.slot_live[taken] = true;
        self.slot_stamp[taken] = self.next_stamp();
        taken as i32
    }

    #[cfg(test)]
    pub fn set_stamp(&mut self, stamp: i32) {
        self.row = stamp;
    }

    /// A new stamp; at the wrap both caches are emptied, where C# empties only one (Mars_Native.md §5.4).
    fn next_stamp(&mut self) -> i32 {
        self.row += 1;
        if self.row != i32::MAX {
            return self.row;
        }
        self.sampled_row.fill(0);
        self.plain_row.fill(0);
        self.row = 1;
        self.row
    }

    /// `Walker.Lines`: the lines from first to last fetched into the window, which slides down when they no longer fit.
    fn lines(&mut self, source: &Source, first: i32, last: i32) {
        let w = self.window_width;
        if w == 0 {
            return;
        }
        if self.window_count == 0 || first < self.window_from {
            self.window_from = first;
            self.window_count = 0;
        } else if last - self.window_from >= WINDOW_LINES {
            let dropped = first - self.window_from;
            let kept = (self.window_count - dropped).max(0);
            if kept > 0 {
                let from = (dropped * w) as usize;
                self.window.copy_within(from..from + (kept * w) as usize, 0);
            }
            self.window_from = first;
            self.window_count = kept;
        }
        while self.window_from + self.window_count <= last {
            let at = (self.window_from + self.window_count).wrapping_mul(w);
            let slot = (self.window_count * w) as usize;
            for x in 0..w {
                self.window[slot + x as usize] = source.fetch(at.wrapping_add(x));
            }
            self.window_count += 1;
        }
    }

    /// `Walker.Fetched`: from the window, or fetched when the index falls outside it.
    #[inline(always)]
    fn fetched(&self, source: &Source, at: i32) -> Pixel {
        let index = at.wrapping_sub(self.window_from.wrapping_mul(self.window_width)) as u32;
        if index < (self.window_count * self.window_width) as u32 { self.window[index as usize] } else { source.fetch(at) }
    }

    /// `Walker.Remembered`: a sample after divot, made once per slot and offset.
    #[inline]
    fn remembered(&mut self, source: &Source, slot: i32, offset: i32, at: i32, bug: i32, divot_on: bool) -> Pixel {
        if offset as u32 >= ROW_SPAN as u32 {
            return self.across(source, at, bug, divot_on);
        }
        let index = (slot * ROW_SPAN + offset) as usize;
        let stamp = self.slot_stamp[slot as usize];
        if self.sampled_row[index] == stamp {
            return self.samples[index];
        }
        let pixel = if divot_on {
            let centre = self.sampled(source, slot, offset, at, bug);
            let left = self.sampled(source, slot, offset - 1, at.wrapping_sub(1), bug);
            let right = self.sampled(source, slot, offset + 1, at.wrapping_add(1), bug);
            divot(centre, left, right)
        } else {
            self.sampled(source, slot, offset, at, bug)
        };
        self.samples[index] = pixel;
        self.sampled_row[index] = stamp;
        pixel
    }

    /// `Walker.Sampled`: a sample before divot, made once per slot and offset.
    #[inline]
    fn sampled(&mut self, source: &Source, slot: i32, offset: i32, at: i32, bug: i32) -> Pixel {
        if offset as u32 >= ROW_SPAN as u32 {
            return self.sample(source, at, bug);
        }
        let index = (slot * ROW_SPAN + offset) as usize;
        let stamp = self.slot_stamp[slot as usize];
        if self.plain_row[index] == stamp {
            return self.plain[index];
        }
        let pixel = self.sample(source, at, bug);
        self.plain[index] = pixel;
        self.plain_row[index] = stamp;
        pixel
    }

    /// `Walker.Across`: the sample and its two neighbours across the row, for divot, uncached.
    fn across(&self, source: &Source, at: i32, bug: i32, divot_on: bool) -> Pixel {
        let pixel = self.sample(source, at, bug);
        if !divot_on {
            return pixel;
        }
        divot(pixel, self.sample(source, at.wrapping_sub(1), bug), self.sample(source, at.wrapping_add(1), bug))
    }

    /// `Walker.Sample`: a whole pixel takes the dither filter if it is on, any other the anti-aliasing filter (Mars_VideoFilter.md §1).
    #[inline]
    fn sample(&self, source: &Source, at: i32, bug: i32) -> Pixel {
        let mut pixel = self.fetched(source, at);
        if self.anti_alias > COVERED {
            pixel.coverage = 7;
        }
        if pixel.coverage == 7 {
            return if self.dither { self.undither(source, at, bug, pixel) } else { pixel };
        }
        self.filter(source, at, bug, pixel)
    }

    /// `Walker.Filter`: six neighbours, only the whole ones counted; the fetch bug folds the row below onto this one (Mars_VideoFilter.md §2.1, §3).
    #[inline(always)]
    fn filter(&self, source: &Source, at: i32, bug: i32, centre: Pixel) -> Pixel {
        let width = source.width;
        let mut red = [0i32; 7];
        let mut green = [0i32; 7];
        let mut blue = [0i32; 7];
        red[0] = centre.red;
        green[0] = centre.green;
        blue[0] = centre.blue;
        let mut full = 1;

        let folded = bug == 1;
        let around = [
            at.wrapping_sub(width).wrapping_sub(1),
            at.wrapping_sub(width).wrapping_add(1),
            at.wrapping_sub(2),
            at.wrapping_add(2),
            if folded { at.wrapping_sub(2) } else { at.wrapping_add(width).wrapping_sub(1) },
            if folded { at.wrapping_add(2) } else { at.wrapping_add(width).wrapping_add(1) },
        ];
        for neighbour in around {
            let pixel = self.fetched(source, neighbour);
            if pixel.coverage != 7 {
                continue;
            }
            red[full] = pixel.red;
            green[full] = pixel.green;
            blue[full] = pixel.blue;
            full += 1;
        }

        let missing = 7i32.wrapping_sub(centre.coverage);
        Pixel {
            red: pull(&red[..full], centre.red, missing),
            green: pull(&green[..full], centre.green, missing),
            blue: pull(&blue[..full], centre.blue, missing),
            coverage: centre.coverage,
        }
    }

    /// `Walker.Dither`: all eight neighbours, each weighed against the pixel as it arrived (Mars_VideoPasses.md §1).
    #[inline(always)]
    fn undither(&self, source: &Source, at: i32, bug: i32, centre: Pixel) -> Pixel {
        let width = source.width;
        let folded = bug == 1;
        let around = [
            at.wrapping_sub(width).wrapping_sub(1),
            at.wrapping_sub(width),
            at.wrapping_sub(width).wrapping_add(1),
            if folded { at.wrapping_sub(1) } else { at.wrapping_add(width).wrapping_sub(1) },
            if folded { at } else { at.wrapping_add(width) },
            if folded { at.wrapping_add(1) } else { at.wrapping_add(width).wrapping_add(1) },
            at.wrapping_sub(1),
            at.wrapping_add(1),
        ];
        let (mut red, mut green, mut blue) = (centre.red, centre.green, centre.blue);
        for neighbour in around {
            let pixel = self.fetched(source, neighbour);
            red += step(centre.red, pixel.red);
            green += step(centre.green, pixel.green);
            blue += step(centre.blue, pixel.blue);
        }
        Pixel { red, green, blue, coverage: centre.coverage }
    }
}
