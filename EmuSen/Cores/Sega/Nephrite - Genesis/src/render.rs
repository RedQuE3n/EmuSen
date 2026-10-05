//! The VDP's picture (stage 4, steps 2 to 4), mode 5's lines drawn in spans up to each write that changes them. Mode
//! 5: planes A and B with full, cell and line horizontal scrolling and full or 2-cell vertical scrolling, the window
//! and its bug, sprites through the link list parsed on the line before they show, with their per-line and per-frame
//! limits and masking, priority, shadow/highlight, interlace's double resolution, the eight-colour palette and the
//! CRAM dots. Mode 4, the Master System's. MacDonald's "Sega Genesis VDP documentation" §12-§17 and "Sega Master
//! System VDP documentation" §7-§10 are the sources, with Nemesis's sprite masking and overflow findings (SpritesMind
//! topic 541), plutiedev's shadow/highlight page and TmEE's measured output levels (topic 2188), and what Nuked-MD's
//! board showed where they are silent (Nephrite_Disputes.md D-4 to D-10). Nephrite_Native.md §14 to §16 are the
//! record, with what is argued.

use crate::vdp::Vdp;

/// The output levels of the 15-step ladder a channel can take: shadow uses steps 0-7, normal the even steps,
/// highlight steps 7-14 (TmEE's measurements).
pub const LADDER: [u8; 15] = [0, 29, 52, 70, 87, 101, 116, 130, 144, 158, 172, 187, 206, 228, 255];

/// The widest line and the tallest frame (interlace's double resolution in V30).
pub const MAX_W: usize = 320;
pub const MAX_H: usize = 480;

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

    /// A sprite pixel as the line buffer keeps it between lines: the colour, the priority in bit 7.
    fn packed(self) -> u8 {
        self.colour | (self.pri as u8) << 7
    }

    fn unpack(b: u8) -> Px {
        Px { colour: b & 0x3F, pri: b & 0x80 != 0 }
    }
}

/// What the sprite pass of one line leaves: the sprites' pixels, and whether the line overflowed or collided.
pub struct SpriteLine {
    px: [Px; MAX_W],
    pub overflow: bool,
    pub dot_overflow: bool,
    pub collision: bool,
}

impl SpriteLine {
    fn empty() -> SpriteLine {
        SpriteLine { px: [Px::default(); MAX_W], overflow: false, dot_overflow: false, collision: false }
    }
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

/// The colour index of a pattern's pixel, `rows` 8, or 16 for double resolution where name n is the 64 bytes at n * 64.
fn pattern_px(vram: &[u8], name: u16, hflip: bool, vflip: bool, x: usize, y: usize, rows: usize) -> u8 {
    let (x, y) = (if hflip { 7 - x } else { x }, if vflip { rows - 1 - y } else { y });
    let base = if rows == 16 { (name as usize & 0x3FF) * 64 } else { (name as usize & 0x7FF) * 32 };
    let b = vram[(base + y * 4 + x / 2) & 0xFFFF];
    if x & 1 == 0 { b >> 4 } else { b & 0xF }
}

/// Mode 4's output levels for a 2-bit component, red and green, and blue, as the board's DAC gives them (D-7).
const MODE4_RG: [u8; 4] = [0, 95, 161, 255];
const MODE4_B: [u8; 4] = [0, 109, 161, 255];

/// Where mode 4's byte address `a` lies in VRAM: bits 1-8 one place up, bit 9 at bit 1, an even address the low byte.
pub fn vram4_address(a: usize) -> usize {
    (a & 0x3C00) | (a & 0x1FE) << 1 | (a >> 8) & 2 | (a & 1) ^ 1
}

fn tile_px(vram: &[u8], entry: u16, x: usize, y: usize, rows: usize) -> Px {
    let p = pattern_px(vram, entry, entry & 0x800 != 0, entry & 0x1000 != 0, x, y, rows);
    Px { colour: if p == 0 { 0 } else { (((entry >> 13) & 3) as u8) << 4 | p }, pri: entry & 0x8000 != 0 }
}

impl Vdp {
    pub fn mode4(&self) -> bool {
        self.regs[1] & 4 == 0
    }

    pub fn width(&self) -> usize {
        if self.h40() && !self.mode4() { 320 } else { 256 }
    }

    pub fn height(&self) -> usize {
        self.vint_line() as usize
    }

    /// Rows of the picture a line gives: two in interlace's double resolution, one otherwise.
    fn doubled(&self) -> bool {
        self.interlace == 3 && !self.mode4()
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
        self.line_window = [self.regs[17], self.regs[18]];
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

    /// A plane's vertical scroll for a 2-cell column, 10 bits or 11 in double resolution.
    fn vscroll(&self, column: usize, b: bool) -> u16 {
        let i = (if self.regs[11] & 4 != 0 { column * 2 } else { 0 } + b as usize).min(39);
        let mask = if self.doubled() { 0x7FF } else { 0x3FF };
        ((self.line_vsram[2 * i] as u16) << 8 | self.line_vsram[2 * i + 1] as u16) & mask
    }

    /// One plane's pixels on picture row `y`: A (or B) scrolled, through its name table. `bug` is the screen range
    /// whose name table data the window's bug fetches from the next column.
    fn plane(&mut self, y: usize, b: bool, bug: Option<(usize, usize)>, from: usize, to: usize, out: &mut [Px]) {
        let (rows, row_shift) = if self.doubled() { (16, 4) } else { (8, 3) };
        let (wc, hc) = self.plane_cells();
        let base = if b { ((self.regs[4] & 7) as usize) << 13 } else { ((self.regs[2] & 0x38) as usize) << 10 };
        let (hsa, hsb) = self.line_hscroll;
        let hs = if b { hsb } else { hsa } as usize;
        // The plane's sizes are powers of two, so its wrap is a mask.
        let (pmask, hmask) = (wc * 8 - 1, hc * rows - 1);
        let first_row = self.regs[16] & 3 == 2;
        let fine = hs & 15;
        let (mut column, mut vs) = (usize::MAX, 0);
        let mut cached = (usize::MAX, [Px::default(); 8]);
        for (x, px) in out.iter_mut().enumerate().take(to).skip(from) {
            // The 2-cell column the VDP fetched this pixel in; the partial one at the left reads column 0 (argued).
            let c = if x < fine { 0 } else { (x - fine) / 16 };
            if c != column {
                (column, vs) = (c, self.vscroll(c, b) as usize);
            }
            let shift = if bug.is_some_and(|(f, t)| (f..t).contains(&x)) { 16 } else { 0 };
            let plx = (x + shift).wrapping_sub(hs) & pmask;
            let ply = (y + vs) & hmask;
            let cell_y = if first_row { 0 } else { ply >> row_shift };
            let at = base + (cell_y * wc + plx / 8) * 2;
            let key = at << 4 | (ply & (rows - 1));
            if cached.0 != key {
                let entry = word(&self.vram, at);
                let mut row = [Px::default(); 8];
                for (i, r) in row.iter_mut().enumerate() {
                    *r = tile_px(&self.vram, entry, i, ply & (rows - 1), rows);
                }
                cached = (key, row);
            }
            *px = cached.1[plx & 7];
        }
        let last = vs;
        self.vscroll_latch = last as u16;
    }

    /// The window's columns on `line`, if any: the whole line in its vertical range, else its horizontal range.
    fn window_span(&self, line: usize, width: usize) -> Option<(usize, usize)> {
        let [h, v] = self.line_window;
        let vp = (v & 0x1F) as usize * 8;
        if (v & 0x80 != 0 && line >= vp) || (v & 0x80 == 0 && line < vp) {
            return Some((0, width));
        }
        let hp = ((h & 0x1F) as usize * 16).min(width);
        let span = if h & 0x80 != 0 { (hp, width) } else { (0, hp) };
        (span.0 < span.1).then_some(span)
    }

    /// MacDonald's window bug: with the window on the left and plane A's fine scroll set, the partial 2-cell column
    /// its scroll shows after the window takes its name table data from the column after it (measured on the board,
    /// Nephrite_Disputes.md D-5).
    fn window_bug(&self, line: usize, width: usize) -> Option<(usize, usize)> {
        let [h, v] = self.line_window;
        let hp = (h & 0x1F) as usize * 16;
        let vp = (v & 0x1F) as usize * 8;
        let whole = (v & 0x80 != 0 && line >= vp) || (v & 0x80 == 0 && line < vp);
        (!whole && h & 0x80 == 0 && hp > 0 && hp < width && self.line_hscroll.0 & 0xF != 0).then_some((hp, (hp + (self.line_hscroll.0 & 0xF) as usize).min(width)))
    }

    fn window(&self, y: usize, from: usize, to: usize, out: &mut [Px]) {
        let rows = if self.doubled() { 16 } else { 8 };
        let h40 = self.h40();
        let base = ((self.regs[3] & if h40 { 0x3C } else { 0x3E }) as usize) << 10;
        let wc = if h40 { 64 } else { 32 };
        for x in from..to {
            let entry = word(&self.vram, base + ((y / rows) * wc + x / 8) * 2);
            out[x] = tile_px(&self.vram, entry, x & 7, y % rows, rows);
        }
    }

    /// The sprites of picture row `y`, as the VDP parses them: the link list from entry 0, at most 64 or 80 entries a
    /// frame and 16 or 20 sprites and 256 or 320 dots a line; a sprite at X 0 masks the rest of the line once a sprite
    /// not at 0 precedes it, or after a line that ended in a dot overflow (Nemesis). In double resolution the sprites'
    /// Y has ten bits and their cells sixteen rows.
    pub fn sprites(&mut self, y: usize) -> SpriteLine {
        let h40 = self.h40();
        let doubled = self.doubled();
        let (rows, ymask, yoff) = if doubled { (16, 0x3FF, 256) } else { (8, 0x1FF, 128) };
        let (max_sprites, max_line, max_dots) = if h40 { (80, 20, 320) } else { (64, 16, 256) };
        let base = ((self.regs[5] & if h40 { 0x7E } else { 0x7F }) as usize) << 9;
        let mut out = SpriteLine::empty();
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
            let sy = ((c[0] as i32) << 8 | c[1] as i32) & ymask;
            let size = c[2];
            let link = (c[3] & 0x7F) as usize;
            let (wc, hc) = (((size >> 2) & 3) as usize + 1, (size & 3) as usize + 1);
            let top = sy - yoff;
            if (top..top + (hc * rows) as i32).contains(&(y as i32)) {
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
                    let row = (y as i32 - top) as usize;
                    let vflip = attr & 0x1000 != 0;
                    let hflip = attr & 0x800 != 0;
                    let cell_row = if vflip { hc - 1 - row / rows } else { row / rows };
                    for sx in 0..shown {
                        let x = xf as i32 - 128 + sx as i32;
                        if !(0..width).contains(&x) {
                            continue;
                        }
                        let cell_col = if hflip { wc - 1 - sx / 8 } else { sx / 8 };
                        let name = (attr & 0x7FF) as usize + cell_col * hc + cell_row;
                        let p = pattern_px(&self.vram, name as u16, hflip, vflip, sx & 7, row % rows, rows);
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

    /// The sprites of `line` parsed on the line before it, as the VDP fills its line buffer: nothing when the display
    /// is off at that point (Nemesis, Eke and MacDonald, topics 851 and 740), and the flags set then.
    pub fn parse_sprites(&mut self, line: usize) {
        if self.display() && !self.mode4() && !self.doubled() {
            let s = self.sprites(line);
            self.sprite_overflow |= s.overflow;
            self.sprite_collision |= s.collision;
            for (b, p) in self.sprite_buffer.iter_mut().zip(s.px.iter()) {
                *b = p.packed();
            }
        } else {
            self.sprite_buffer.fill(0);
        }
    }

    /// Mode 4's colour: the entry's red and green fields hold the Master System's six bits, --BBGGRR, shown at the
    /// board's four levels a channel (D-7).
    fn rgb4(&self, colour: u8) -> [u8; 4] {
        let c = Self::cram_word(&self.cram, colour as usize & 0x1F);
        let b = (c >> 1 & 7) | (c >> 5 & 7) << 3;
        [MODE4_RG[(b & 3) as usize], MODE4_RG[(b >> 2 & 3) as usize], MODE4_B[(b >> 4 & 3) as usize], 255]
    }

    fn rgb(&self, colour: u8, intensity: u8) -> [u8; 4] {
        let c = Self::cram_word(&self.cram, colour as usize & 0x3F);
        // Register 0's bit 2 clear keeps only each component's lowest bit (MacDonald §17; measured on the board, D-8).
        let full = self.regs[0] & 4 != 0;
        let level = |v: u16| {
            let v = if full { (v & 7) as usize } else { (v & 1) as usize };
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

    /// The line and pixel the beam is on: a line's pixels run from H $18 to its width past it, through the V
    /// counter's step into the next line's first H values; between them, before the next line's pixel 0 (`false`).
    fn beam(&self, delay: u64) -> (usize, usize, bool) {
        let at = (self.time + delay).saturating_sub(self.cur_line_start).min(crate::vdp::LINE - 1);
        let h = self.timing().h(at) as usize;
        let (step, width) = if self.h40() { (0x14A, 320) } else { (0x10A, 256) };
        let line = self.cur_line as usize;
        if (0x18..step).contains(&h) {
            (line, h - 0x18, true)
        } else if (step..0x18 + width).contains(&h) && line > 0 {
            (line - 1, h - 0x18, true)
        } else {
            (line, 0, false)
        }
    }

    /// Before a write that changes what mode 5's picture reads: the line under the beam drawn up to it, and a CRAM
    /// write's dot of its colour on the beam's pixel (D-6).
    pub fn before_change(&mut self, dot: Option<u16>) {
        self.before_change_after(dot, 0);
    }

    /// The same for a change that shows `delay` master clocks after it is made.
    pub fn before_change_after(&mut self, dot: Option<u16>, delay: u64) {
        if self.mode4() {
            return;
        }
        let (line, x, on) = self.beam(delay);
        if line >= self.height() {
            // Past the last shown line's last pixel: that line is finished with the state before this write.
            if self.open.is_some_and(|o| o < line) {
                self.finish();
            }
            return;
        }
        self.goto(line, x);
        if let Some(v) = dot
            && on
            && self.display()
            && x < self.width()
            && self.open == Some(line)
            && self.span_x == x
        {
            if self.draw {
                for r in 0..self.rows() {
                    let at = (r * MAX_W + x) * 4;
                    let level = |c: u16| LADDER[2 * (c & 7) as usize];
                    self.span_rgba[at..at + 4].copy_from_slice(&[level(v >> 1), level(v >> 5), level(v >> 9), 255]);
                }
            }
            self.span_x = x + 1;
        }
    }

    /// The display enable changed mid-line: blanking begins twelve pixels after a register write would show, and the
    /// planes return at the first 16-pixel boundary from 24 pixels after it (measured in part, D-9).
    pub fn display_change(&mut self, on: bool, delay: u64) {
        if self.mode4() {
            return;
        }
        let (line, x, beam_on) = self.beam(delay);
        if line >= self.height() {
            if self.open.is_some_and(|o| o < line) {
                self.finish();
            }
            return;
        }
        // Off blanks 12 pixels on; on brings the planes back at the first 16-pixel boundary 24 pixels on.
        let x = if !beam_on { x } else if on { (x + 24).div_ceil(16) * 16 } else { x + 12 };
        self.goto(line, x);
    }

    fn rows(&self) -> usize {
        if self.doubled() { 2 } else { 1 }
    }

    /// The open line drawn to `x` on `line`, the line before finished first.
    fn goto(&mut self, line: usize, x: usize) {
        match self.open {
            Some(o) if o == line => {}
            Some(o) if o > line => return,
            Some(_) => {
                self.finish();
                self.open_line(line);
            }
            None => self.open_line(line),
        }
        self.advance(x);
    }

    /// A line opened: its sprite pixels taken, from the line buffer or, in double resolution, parsed for both rows
    /// with the flags and the dot-overflow carry from the field's own row.
    fn open_line(&mut self, line: usize) {
        if line == 0 {
            self.frame.width = self.width();
            self.frame.height = self.height() * self.rows();
            // CRAM written from outside the ports (a state, a memory editor) is seen from the next frame.
            self.palette_dirty = true;
        }
        self.open = Some(line);
        self.span_x = 0;
        self.span_latch.0.copy_from_slice(&self.line_vsram);
        (self.span_latch.1, self.span_latch.2) = (self.line_hscroll, self.line_window);
        if self.doubled() {
            for p in 0..2 {
                let carry = self.dot_overflow_line;
                let s = if self.display() { self.sprites(2 * line + p) } else { SpriteLine::empty() };
                if p != self.odd as usize {
                    self.dot_overflow_line = carry;
                } else {
                    self.sprite_overflow |= s.overflow;
                    self.sprite_collision |= s.collision;
                }
                for (b, px) in self.span_sprites[p * MAX_W..(p + 1) * MAX_W].iter_mut().zip(s.px.iter()) {
                    *b = px.packed();
                }
            }
        } else {
            self.span_sprites[..MAX_W].copy_from_slice(&self.sprite_buffer[..MAX_W]);
        }
    }

    /// The open line's pixels from where it was drawn to up to `x`, from the present state.
    fn advance(&mut self, x: usize) {
        let x = x.min(self.width());
        let Some(line) = self.open else { return };
        if x <= self.span_x {
            return;
        }
        if self.draw {
            let mut buf = std::mem::take(&mut self.span_rgba);
            self.swap_latch();
            for r in 0..self.rows() {
                let y = if self.doubled() { 2 * line + r } else { line };
                self.compose(line, y, r, self.span_x, x, &mut buf[r * MAX_W * 4..(r + 1) * MAX_W * 4]);
            }
            self.swap_latch();
            self.span_rgba = buf;
        }
        self.span_x = x;
    }

    /// The open line's latched scroll and window values exchanged with the present line's.
    fn swap_latch(&mut self) {
        std::mem::swap(&mut self.line_vsram, &mut self.span_latch.0);
        std::mem::swap(&mut self.line_hscroll, &mut self.span_latch.1);
        std::mem::swap(&mut self.line_window, &mut self.span_latch.2);
    }

    /// After a state's load: the open line's pixels so far drawn again from the state.
    pub fn redraw_open_line(&mut self) {
        if self.open.is_some() {
            let x = std::mem::take(&mut self.span_x);
            self.advance(x);
        }
    }

    /// The open line drawn to its end and put into the frame.
    fn finish(&mut self) {
        let Some(line) = self.open else { return };
        let width = self.width();
        self.advance(width);
        if self.draw {
            for r in 0..self.rows() {
                let y = if self.doubled() { 2 * line + r } else { line };
                if y < MAX_H {
                    let row = &mut self.frame.rgba[y * MAX_W * 4..(y + 1) * MAX_W * 4];
                    row[..width * 4].copy_from_slice(&self.span_rgba[r * MAX_W * 4..r * MAX_W * 4 + width * 4]);
                    // A line narrower than the frame (the width changed within it) leaves black, never an older frame's pixels.
                    row[width * 4..].chunks_mut(4).for_each(|p| p.copy_from_slice(&[0, 0, 0, 255]));
                }
            }
        }
        self.open = None;
    }

    /// The V counter's step that ends `line`: mode 5's line drawn to the beam (its last pixels come after the step),
    /// the line before finished, and the next line's sprites parsed (or, after the last line of the frame, line 0's);
    /// mode 4's line drawn whole. Drawing is skipped when the frame is not drawn; the sprite pass runs either way.
    pub fn line_end(&mut self, line: usize) {
        if self.mode4() {
            self.open = None;
            if line < self.height() {
                let mut frame = std::mem::replace(&mut self.frame, Frame { rgba: Vec::new(), width: 0, height: 0 });
                if line == 0 {
                    (frame.width, frame.height) = (self.width(), self.height());
                }
                if self.regs[0] & 4 != 0 {
                    self.render_mode4(line, self.draw, &mut frame);
                } else if self.draw && line < MAX_H {
                    // Mode 5 and mode 4 both off select the TMS9918's modes, which the Genesis shows black (MacDonald's
                    // SMS VDP document, §13).
                    frame.rgba[line * MAX_W * 4..(line + 1) * MAX_W * 4].chunks_mut(4).for_each(|p| p.copy_from_slice(&[0, 0, 0, 255]));
                }
                self.frame = frame;
            } else {
                self.blank_line();
            }
        } else if line < self.height() {
            let step = if self.h40() { 0x14A } else { 0x10A };
            self.goto(line, step - 0x18);
        } else {
            if self.open.is_some_and(|o| o < line) {
                self.finish();
            }
            self.blank_line();
        }
        if line + 1 < self.height() {
            self.parse_sprites(line + 1);
        } else if line as u32 == self.lines() - 1 {
            self.parse_sprites(0);
        }
    }

    /// Pixels `from` to `to` of picture row `y` of `line` (the open line's row `r`) into `out`.
    fn compose(&mut self, line: usize, y: usize, r: usize, from: usize, to: usize, out: &mut [u8]) {
        let width = self.width();
        let backdrop = self.regs[7] & 0x3F;
        if !self.display() {
            let c = self.rgb(backdrop, 1);
            for p in out[from * 4..to * 4].chunks_mut(4) {
                p.copy_from_slice(&c);
            }
            return;
        }
        let mut a = [Px::default(); MAX_W];
        let mut b = [Px::default(); MAX_W];
        self.plane(y, true, None, from, to, &mut b[..width]);
        let bug = self.window_bug(line, width);
        self.plane(y, false, bug, from, to, &mut a[..width]);
        if let Some((wf, wt)) = self.window_span(line, width) {
            self.window(y, wf.max(from), wt.min(to), &mut a);
        }
        let sh = self.regs[12] & 8 != 0;
        let blank_left = self.regs[0] & 0x20 != 0;
        if self.palette_dirty {
            let mut palette = [[0u8; 4]; 192];
            for (i, c) in palette.iter_mut().enumerate() {
                *c = self.rgb((i & 63) as u8, (i >> 6) as u8);
            }
            (self.palette, self.palette_dirty) = (palette, false);
        }
        let palette = self.palette;
        for x in from..to {
            let (s, pa, pb) = (Px::unpack(self.span_sprites[r * MAX_W + x]), a[x], b[x]);
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
            out[x * 4..x * 4 + 4].copy_from_slice(&c);
        }
    }

    /// A byte of mode 4's 16 KiB (measured on the board, Nephrite_Disputes.md D-7).
    fn vram4(&self, a: usize) -> u8 {
        self.vram[vram4_address(a)]
    }

    /// Mode 4, the Master System's: a 32 by 24 name table of little-endian words in planar patterns, scrolled by
    /// registers 8 and 9 with the top two rows' and right eight columns' locks, eight sprites a line from a 64-entry
    /// table, and the left column's mask (MacDonald's SMS VDP §7-§10), read through mode 4's map of VRAM, its colours
    /// CRAM's first 32 entries (D-7).
    fn render_mode4(&mut self, line: usize, draw: bool, frame: &mut Frame) {
        let width = 256;
        let backdrop = 16 | (self.regs[7] & 0xF);
        let mut sprite = [0u8; 256];
        if self.display() {
            let base = ((self.regs[5] & 0x7E) as usize) << 7;
            let tall = self.regs[1] & 2 != 0;
            let h = if tall { 16 } else { 8 };
            let mut found = 0;
            for i in 0..64 {
                let y = self.vram4(base + i) as usize;
                if y == 0xD0 {
                    break;
                }
                let top = y + 1;
                let row = (line + 256 - top) % 256;
                if row >= h {
                    continue;
                }
                found += 1;
                if found > 8 {
                    self.sprite_overflow = true;
                    break;
                }
                let x0 = self.vram4(base + 0x80 + 2 * i) as i32 - if self.regs[0] & 8 != 0 { 8 } else { 0 };
                let mut n = self.vram4(base + 0x81 + 2 * i) as usize | if self.regs[6] & 4 != 0 { 0x100 } else { 0 };
                if tall {
                    n &= !1;
                }
                let a = n * 32 + row * 4;
                for px in 0..8 {
                    let x = x0 + px;
                    if !(0..256).contains(&x) {
                        continue;
                    }
                    let bit = 7 - px as usize;
                    let c = (0..4).fold(0u8, |c, p| c | ((self.vram4(a + p) >> bit) & 1) << p);
                    if c != 0 {
                        if sprite[x as usize] != 0 {
                            self.sprite_collision = true;
                        } else {
                            sprite[x as usize] = 16 | c;
                        }
                    }
                }
            }
        }
        if !draw || line >= MAX_H {
            return;
        }
        let mut palette = [[0u8; 4]; 32];
        for (i, c) in palette.iter_mut().enumerate() {
            *c = self.rgb4(i as u8);
        }
        let row = &mut frame.rgba[line * MAX_W * 4..(line + 1) * MAX_W * 4];
        for p in row.chunks_mut(4).skip(width) {
            p.copy_from_slice(&[0, 0, 0, 255]);
        }
        if !self.display() {
            for p in row.chunks_mut(4).take(width) {
                p.copy_from_slice(&palette[backdrop as usize]);
            }
            return;
        }
        let name_base = ((self.regs[2] & 0x0E) as usize) << 10;
        let hs = if self.regs[0] & 0x40 != 0 && line < 16 { 0 } else { self.regs[8] as usize };
        for x in 0..width {
            let vs = if self.regs[0] & 0x80 != 0 && x >= 192 { 0 } else { self.regs[9] as usize };
            let bx = (x + 256 - hs) % 256;
            let by = (line + vs) % 224;
            let at = name_base + ((by / 8) * 32 + bx / 8) * 2;
            let e = self.vram4(at) as u16 | (self.vram4(at + 1) as u16) << 8;
            let (hflip, vflip) = (e & 0x200 != 0, e & 0x400 != 0);
            let (px, py) = (if hflip { 7 - bx % 8 } else { bx % 8 }, if vflip { 7 - by % 8 } else { by % 8 });
            let a = (e as usize & 0x1FF) * 32 + py * 4;
            let bit = 7 - px;
            let c = (0..4).fold(0u8, |c, p| c | ((self.vram4(a + p) >> bit) & 1) << p);
            let bg = if e & 0x800 != 0 { 16 } else { 0 } | c;
            let pri = e & 0x1000 != 0;
            let s = sprite[x];
            let colour = if s != 0 && !(pri && c != 0) { s } else { bg };
            let colour = if self.regs[0] & 0x20 != 0 && x < 8 { backdrop } else { colour };
            row[x * 4..x * 4 + 4].copy_from_slice(&palette[colour as usize & 31]);
        }
    }
}
