//! The display: VRAM read a line at a time as the video timing reaches it (Diamond_Plan.md §5.4), from GP1(05h)'s
//! start for the pixels GP1(06h)'s range holds at GP1(08h)'s dot clock, 15- or 24-bit, and in 480-line interlace a
//! field's lines interleaved with the other's. The picture is RGBA at the display area's own size, as it stands when a
//! field's first displayed line is read (Diamond_GPU.md §7). What it holds between fields is the machine's state: VRAM
//! as it was when each line was read, which VRAM as it is cannot give again (Diamond_Native.md §23).

use super::vram::Vram;
use crate::video::Video;

/// The largest picture: 640 dots, and 288 lines a field interleaved.
pub const MAX_WIDTH: u32 = 640;
pub const MAX_HEIGHT: u32 = 576;

#[derive(Clone)]
pub struct Display {
    pub picture: Vec<u8>,
    pub width: u32,
    pub height: u32,
    /// The field being scanned: its first absolute line, and the next of its lines to read.
    pub field_line: u64,
    pub next: u32,
    /// GP1(05h)'s start, and GP1(03h)'s blanking.
    pub start: u32,
    pub blanked: bool,
}

impl Default for Display {
    fn default() -> Display {
        Display { picture: [0, 0, 0, 255].repeat((MAX_WIDTH * MAX_HEIGHT) as usize), width: 256, height: 240, field_line: u64::MAX, next: 0, start: 0, blanked: true }
    }
}

/// The dots a line shows: `(((X2-X1)/dot clock)+2) AND NOT 3` (psx-spx "GP1(06h)"), and the lines a field shows.
pub fn size(video: &Video) -> (u32, u32) {
    let clocks = video.x2.saturating_sub(video.x1).min(0xFFF) as u64;
    let w = (((clocks / video.dot_divider()) as u32 + 2) & !3).min(MAX_WIDTH);
    let lines = video.y2.saturating_sub(video.y1);
    let h = if hires(video) { lines * 2 } else { lines }.min(MAX_HEIGHT);
    if w == 0 || h == 0 { (horizontal(video), if hires(video) { 480 } else { 240 }) } else { (w, h) }
}

fn horizontal(video: &Video) -> u32 {
    if video.mode & 1 << 6 != 0 { 368 } else { [256, 320, 512, 640][(video.mode & 3) as usize] }
}

fn hires(video: &Video) -> bool {
    video.interlaced() && video.mode & 1 << 2 != 0
}

impl Display {
    pub fn frame(&self) -> &[u8] {
        &self.picture[..(self.width * self.height * 4) as usize]
    }

    /// Reads every line of the current field that began before CPU cycle `t`, from VRAM as it is now.
    pub fn scan(&mut self, vram: &Vram, video: &Video, t: u64) {
        let (line, field_line) = video.position(t);
        if field_line != self.field_line {
            self.field_line = field_line;
            self.next = 0;
        }
        let upto = (line.saturating_sub(field_line) as u32).min(video.field_lines());
        while self.next < upto {
            let l = self.next;
            self.next += 1;
            if l == video.y1 {
                self.resize(size(video));
            }
            if l >= video.y1 && l < video.y2 {
                self.read_line(vram, video, l - video.y1, video.odd as u32);
            }
        }
    }

    /// The picture's size, fixed as a field's first displayed line is read; a new size starts black.
    fn resize(&mut self, (w, h): (u32, u32)) {
        if (w, h) != (self.width, self.height) {
            (self.width, self.height) = (w, h);
            self.clear();
        }
    }

    fn clear(&mut self) {
        self.picture.fill(0);
        for p in self.picture.as_chunks_mut::<4>().0 {
            p[3] = 255;
        }
    }

    /// The picture read whole from VRAM as it is now, both fields of an interlaced one: what a state from before the
    /// picture was the state's is given, in place of whatever the machine loading it showed (Diamond_Native.md §23).
    pub fn reread(&mut self, vram: &Vram, video: &Video) {
        self.clear();
        for row in 0..video.y2.saturating_sub(video.y1) {
            for field in 0..=hires(video) as u32 {
                self.read_line(vram, video, row, field);
            }
        }
    }

    fn read_line(&mut self, vram: &Vram, video: &Video, row: u32, field: u32) {
        let hi = hires(video);
        let out_row = if hi { row * 2 + field } else { row };
        if out_row >= self.height {
            return;
        }
        let (sx, sy) = (self.start & 0x3FF, self.start >> 10 & 0x1FF);
        let src_row = sy + if hi { row * 2 + field } else { row };
        let deep = video.mode & 1 << 4 != 0;
        let at = (out_row * self.width * 4) as usize;
        let out = &mut self.picture[at..at + (self.width * 4) as usize];
        for (i, px) in out.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            let i = i as u32;
            let rgb = if self.blanked {
                [0, 0, 0]
            } else if deep {
                let byte = sx * 2 + i * 3;
                let b = |k: u32| {
                    let h = vram.get((byte + k) / 2, src_row);
                    if (byte + k) & 1 == 0 { h as u8 } else { (h >> 8) as u8 }
                };
                [b(0), b(1), b(2)]
            } else {
                let p = vram.get(sx + i, src_row);
                [p & 31, p >> 5 & 31, p >> 10 & 31].map(|c| (c << 3 | c >> 2) as u8)
            };
            px[..3].copy_from_slice(&rgb);
        }
    }
}
