//! The VDP's picture in mode 5, a line at a time (stage 4, step 2): planes A and B with full, cell and line
//! horizontal scrolling and full or 2-cell vertical scrolling, the window, sprites through the link list with their
//! per-line and per-frame limits and masking, priority, and shadow/highlight. MacDonald's "Sega Genesis VDP
//! documentation" §12-§17 is the source, with Nemesis's sprite masking and overflow findings (SpritesMind topic 541),
//! plutiedev's shadow/highlight page and TmEE's measured output levels (topic 2188). Nephrite_Native.md §14 is the
//! record, with what is argued.

use crate::vdp::Vdp;

/// The output levels of the 15-step ladder a channel can take: shadow uses steps 0-7, normal the even steps,
/// highlight steps 7-14 (TmEE's measurements).
pub const LADDER: [u8; 15] = [0, 29, 52, 70, 87, 101, 116, 130, 144, 158, 172, 187, 206, 228, 255];

/// The widest line and the tallest frame.
pub const MAX_W: usize = 320;
pub const MAX_H: usize = 240;

/// A pixel of a layer: its CRAM index with 0 transparent in its low nibble, and its priority bit.
#[derive(Clone, Copy, Default)]
struct Px {
    colour: u8,
    pri: bool,
}

impl Px {
    fn opaque(self) -> bool {
        self.colour & 0xF != 0
    }
}

/// What the sprite pass of one line leaves: the sprites' pixels, and whether the line overflowed or collided.
pub struct SpriteLine {
    px: [Px; MAX_W],
    pub overflow: bool,
    pub dot_overflow: bool,
    pub collision: bool,
}

/// The frame being drawn, as RGBA, with the width and height it is drawn at.
pub struct Frame {
    pub rgba: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

impl Frame {
    pub fn new() -> Frame {
        Frame { rgba: vec![0; MAX_W * MAX_H * 4], width: 320, height: 224 }
    }
}

impl Default for Frame {
    fn default() -> Self {
        Self::new()
    }
}

fn word(m: &[u8], a: usize) -> u16 {
    (m[a & 0xFFFF] as u16) << 8 | m[(a + 1) & 0xFFFF] as u16
}

/// The colour index of a pattern's pixel: `entry` a name table word or sprite attribute, `x` and `y` within the
/// cell before flipping.
fn pattern_px(vram: &[u8], name: u16, hflip: bool, vflip: bool, x: usize, y: usize) -> u8 {
    let (x, y) = (if hflip { 7 - x } else { x }, if vflip { 7 - y } else { y });
    let b = vram[((name as usize & 0x7FF) * 32 + y * 4 + x / 2) & 0xFFFF];
    if x & 1 == 0 { b >> 4 } else { b & 0xF }
}

fn tile_px(vram: &[u8], entry: u16, x: usize, y: usize) -> Px {
    let p = pattern_px(vram, entry, entry & 0x800 != 0, entry & 0x1000 != 0, x, y);
    Px { colour: if p == 0 { 0 } else { (((entry >> 13) & 3) as u8) << 4 | p }, pri: entry & 0x8000 != 0 }
}

impl Vdp {
    pub fn width(&self) -> usize {
        if self.h40() { 320 } else { 256 }
    }

    pub fn height(&self) -> usize {
        self.vint_line() as usize
    }

    /// Planes A and B's size in cells: 32, 64 or 128 each way, the prohibited setting taken as 32 and the pair held to
    /// 4,096 cells.
    fn plane_cells(&self) -> (usize, usize) {
        let size = |v: u8| match v & 3 {
            1 => 64,
            3 => 128,
            _ => 32,
        };
        let w = size(self.regs[16]);
        let h = size(self.regs[16] >> 4).min(4096 / w);
        (w, h)
    }

    /// The scroll values a line's fetches read, taken as it begins: a line interrupt's write lands after the line's
    /// first fetches and so is the next line's (OutRun's road, which writes VSRAM every line, shows it).
    pub fn latch_line(&mut self, line: usize) {
        self.line_vsram.copy_from_slice(&self.vsram);
        self.line_hscroll = self.hscroll(line);
    }

    /// The horizontal scroll of planes A and B on `line` (register 11's full, cell and line modes).
    fn hscroll(&self, line: usize) -> (u16, u16) {
        let base = ((self.regs[13] & 0x3F) as usize) << 10;
        let i = match self.regs[11] & 3 {
            0 => 0,
            1 => line & 7,
            2 => line & !7,
            _ => line,
        };
        (word(&self.vram, base + i * 4) & 0x3FF, word(&self.vram, base + i * 4 + 2) & 0x3FF)
    }

    fn vscroll(&self, column: usize, b: bool) -> u16 {
        let i = (if self.regs[11] & 4 != 0 { column * 2 } else { 0 } + b as usize).min(39);
        ((self.line_vsram[2 * i] as u16) << 8 | self.line_vsram[2 * i + 1] as u16) & 0x3FF
    }

    /// One plane's pixels on `line`: A (or B) scrolled, through its name table.
    fn plane(&mut self, line: usize, b: bool, out: &mut [Px]) {
        let (wc, hc) = self.plane_cells();
        let base = if b { ((self.regs[4] & 7) as usize) << 13 } else { ((self.regs[2] & 0x38) as usize) << 10 };
        let (hsa, hsb) = self.line_hscroll;
        let hs = if b { hsb } else { hsa } as usize;
        let (pw, ph) = (wc * 8, hc * 8);
        let fine = hs & 15;
        let mut last = 0;
        let mut cached = (usize::MAX, [Px::default(); 8]);
        for (x, px) in out.iter_mut().enumerate() {
            // The 2-cell column the VDP fetched this pixel in; the partial one at the left reads column 0 (argued).
            let vs = self.vscroll(if x < fine { 0 } else { (x - fine) / 16 }, b) as usize;
            last = vs;
            let plx = (x + pw - hs % pw) % pw;
            let ply = if self.regs[16] & 3 == 2 { 0 } else { (line + vs) % ph };
            let at = base + ((ply / 8) * wc + plx / 8) * 2;
            let key = at << 3 | (ply & 7);
            if cached.0 != key {
                let entry = word(&self.vram, at);
                let mut row = [Px::default(); 8];
                for (i, r) in row.iter_mut().enumerate() {
                    *r = tile_px(&self.vram, entry, i, ply & 7);
                }
                cached = (key, row);
            }
            *px = cached.1[plx & 7];
        }
        self.vscroll_latch = last as u16;
    }

    /// The window's columns on `line`, if any: the whole line in its vertical range, else its horizontal range.
    fn window_span(&self, line: usize, width: usize) -> Option<(usize, usize)> {
        let (h, v) = (self.regs[17], self.regs[18]);
        let vp = (v & 0x1F) as usize * 8;
        if (v & 0x80 != 0 && line >= vp) || (v & 0x80 == 0 && line < vp) {
            return Some((0, width));
        }
        let hp = ((h & 0x1F) as usize * 16).min(width);
        let span = if h & 0x80 != 0 { (hp, width) } else { (0, hp) };
        (span.0 < span.1).then_some(span)
    }

    fn window(&self, line: usize, from: usize, to: usize, out: &mut [Px]) {
        let h40 = self.h40();
        let base = ((self.regs[3] & if h40 { 0x3C } else { 0x3E }) as usize) << 10;
        let wc = if h40 { 64 } else { 32 };
        for x in from..to {
            let entry = word(&self.vram, base + ((line / 8) * wc + x / 8) * 2);
            out[x] = tile_px(&self.vram, entry, x & 7, line & 7);
        }
    }

    /// The sprites of `line`, as the VDP parses them: the link list from entry 0, at most 64 or 80 entries a frame
    /// and 16 or 20 sprites and 256 or 320 dots a line; a sprite at X 0 masks the rest of the line once a sprite not
    /// at 0 precedes it, or after a line that ended in a dot overflow (Nemesis).
    pub fn sprites(&mut self, line: usize) -> SpriteLine {
        let h40 = self.h40();
        let (max_sprites, max_line, max_dots) = if h40 { (80, 20, 320) } else { (64, 16, 256) };
        let base = ((self.regs[5] & if h40 { 0x7E } else { 0x7F }) as usize) << 9;
        let mut out = SpriteLine { px: [Px::default(); MAX_W], overflow: false, dot_overflow: false, collision: false };
        let width = self.width() as i32;
        let (mut index, mut parsed, mut on_line, mut dots) = (0usize, 0, 0, 0);
        let (mut seen_nonzero, mut masked) = (false, false);
        let prev_overflow = self.dot_overflow_line;
        loop {
            if index >= max_sprites || parsed >= max_sprites {
                break;
            }
            parsed += 1;
            let c = &self.sat_cache[index * 4..index * 4 + 4];
            let y = ((c[0] as i32) << 8 | c[1] as i32) & 0x1FF;
            let size = c[2];
            let link = (c[3] & 0x7F) as usize;
            let (wc, hc) = (((size >> 2) & 3) as usize + 1, (size & 3) as usize + 1);
            let top = y - 128;
            if (top..top + hc as i32 * 8).contains(&(line as i32)) {
                on_line += 1;
                if on_line > max_line {
                    out.overflow = true;
                    break;
                }
                let attr = word(&self.vram, base + index * 8 + 4);
                let xf = word(&self.vram, base + index * 8 + 6) & 0x1FF;
                if xf == 0 {
                    if seen_nonzero || prev_overflow {
                        masked = true;
                    }
                } else {
                    seen_nonzero = true;
                }
                let w = wc * 8;
                let shown = if dots + w > max_dots { max_dots - dots } else { w };
                dots += shown;
                if !masked && xf != 0 {
                    let row = (line as i32 - top) as usize;
                    let vflip = attr & 0x1000 != 0;
                    let hflip = attr & 0x800 != 0;
                    let cell_row = if vflip { hc - 1 - row / 8 } else { row / 8 };
                    for sx in 0..shown {
                        let x = xf as i32 - 128 + sx as i32;
                        if !(0..width).contains(&x) {
                            continue;
                        }
                        let cell_col = if hflip { wc - 1 - sx / 8 } else { sx / 8 };
                        let name = (attr & 0x7FF) as usize + cell_col * hc + cell_row;
                        let p = pattern_px(&self.vram, name as u16, hflip, vflip, sx & 7, row & 7);
                        if p == 0 {
                            continue;
                        }
                        let slot = &mut out.px[x as usize];
                        if slot.opaque() {
                            out.collision = true;
                        } else {
                            *slot = Px { colour: (((attr >> 13) & 3) as u8) << 4 | p, pri: attr & 0x8000 != 0 };
                        }
                    }
                }
                if shown < w {
                    out.overflow = true;
                    out.dot_overflow = true;
                    break;
                }
            }
            if link == 0 {
                break;
            }
            index = link;
        }
        self.dot_overflow_line = out.dot_overflow;
        out
    }

    fn rgb(&self, colour: u8, intensity: u8) -> [u8; 4] {
        let c = Self::cram_word(&self.cram, colour as usize & 0x3F);
        let level = |v: u16| {
            let v = (v & 7) as usize;
            LADDER[match intensity {
                0 => v,
                1 => 2 * v,
                _ => 7 + v,
            }]
        };
        [level(c >> 1), level(c >> 5), level(c >> 9), 255]
    }

    /// A line not drawn still fetches plane B's vertical scroll, which VSRAM past its 40 words reads (argued from
    /// VDPFIFOTesting's VSRAM fills, made in blanking).
    pub fn blank_line(&mut self) {
        self.vscroll_latch = self.vsram_word(1) & 0x3FF;
    }

    /// One line of the picture: the sprite pass always (its flags are the VDP's state), the planes and the composite
    /// only when drawing.
    pub fn render_line(&mut self, line: usize, draw: bool, frame: &mut Frame) {
        let width = self.width();
        let display = self.display() && self.regs[1] & 4 != 0;
        let sprites = if display { self.sprites(line) } else { SpriteLine { px: [Px::default(); MAX_W], overflow: false, dot_overflow: false, collision: false } };
        self.sprite_overflow |= sprites.overflow;
        self.sprite_collision |= sprites.collision;
        if line == 0 {
            frame.width = width;
            frame.height = self.height();
        }
        if !draw || line >= MAX_H {
            return;
        }
        let row = &mut frame.rgba[line * MAX_W * 4..(line + 1) * MAX_W * 4];
        // A line narrower than the frame (the width changed within it) leaves black, never an older frame's pixels.
        for p in row.chunks_mut(4).skip(width) {
            p.copy_from_slice(&[0, 0, 0, 255]);
        }
        let backdrop = self.regs[7] & 0x3F;
        if !display {
            let c = self.rgb(backdrop, 1);
            for p in row.chunks_mut(4).take(width) {
                p.copy_from_slice(&c);
            }
            return;
        }
        let mut a = [Px::default(); MAX_W];
        let mut b = [Px::default(); MAX_W];
        self.plane(line, true, &mut b[..width]);
        self.plane(line, false, &mut a[..width]);
        if let Some((from, to)) = self.window_span(line, width) {
            self.window(line, from, to, &mut a);
        }
        let sh = self.regs[12] & 8 != 0;
        let blank_left = self.regs[0] & 0x20 != 0;
        let mut palette = [[0u8; 4]; 192];
        for (i, c) in palette.iter_mut().enumerate() {
            *c = self.rgb((i & 63) as u8, (i >> 6) as u8);
        }
        for x in 0..width {
            let (s, pa, pb) = (sprites.px[x], a[x], b[x]);
            let order = [(s, true, true), (pa, true, false), (pb, true, false), (s, false, true), (pa, false, false), (pb, false, false)];
            let mut top: Option<(Px, bool)> = None;
            let mut under: Option<Px> = None;
            for &(p, hi, sprite) in &order {
                if p.opaque() && p.pri == hi {
                    if top.is_none() {
                        top = Some((p, sprite));
                    } else if !sprite && under.is_none() {
                        under = Some(p);
                    }
                }
            }
            let mut intensity = 1u8;
            let colour = if !sh {
                top.map_or(backdrop, |(p, _)| p.colour)
            } else {
                // An operator pixel is transparent and so gives no priority (plutiedev: "not transparent pixels in sprites").
                let operator = s.colour == 0x3E || s.colour == 0x3F;
                let high = pa.pri || pb.pri || (s.opaque() && !operator && s.pri);
                intensity = if high { 1 } else { 0 };
                match top {
                    Some((p, true)) if p.colour == 0x3E || p.colour == 0x3F => {
                        // An operator: palette 3's colour 14 brightens what lies under it, 15 darkens (plutiedev).
                        intensity = if p.colour == 0x3E { (intensity + 1).min(2) } else { 0 };
                        under.map_or(backdrop, |u| u.colour)
                    }
                    Some((p, true)) if p.colour & 0xF == 0xE => {
                        intensity = 1;
                        p.colour
                    }
                    Some((p, _)) => p.colour,
                    None => backdrop,
                }
            };
            let c = if blank_left && x < 8 { palette[64 + backdrop as usize] } else { palette[(intensity as usize) << 6 | (colour as usize & 63)] };
            row[x * 4..x * 4 + 4].copy_from_slice(&c);
        }
    }
}
