//! The PPU: its registers and memory ports, the H/V counter latch, and the picture drawn a line at a time in spans,
//! from fullsnes's PPU sections and anomie's register document. Drawing reads the state and writes only the frame,
//! so a skipped picture leaves the machine as a drawn one does. See VenusRT_Native.md §15.

pub const WIDTH: usize = 256;
pub const HEIGHT: usize = 224;
/// The first dot of the picture: H=22 to 277 are visible (fullsnes, OPHCT).
pub const FIRST_DOT: u16 = 22;

#[derive(Clone, Debug)]
pub struct Ppu {
    pub vram: Box<[u16]>,
    pub cgram: Box<[u16]>,
    pub oam: Box<[u8]>,
    /// $2100-$213F as last written.
    pub regs: Box<[u8]>,
    pub vram_address: u16,
    /// The prefetch register $2139/$213A read.
    pub vram_buffer: u16,
    pub cgram_address: u8,
    pub cgram_second: bool,
    pub cgram_low: u8,
    /// The 9-bit reload value, the priority rotation bit and the 10-bit address the ports step.
    pub oam_reload: u16,
    pub oam_rotation: bool,
    pub oam_address: u16,
    pub oam_low: u8,
    /// BGnHOFS and BGnVOFS, and the byte their two-write mechanism keeps.
    pub hofs: [u16; 4],
    pub vofs: [u16; 4],
    pub bg_old: u8,
    pub ophct: u16,
    pub opvct: u16,
    pub oph_second: bool,
    pub opv_second: bool,
    pub latched: bool,
    /// The two chips' own open-bus latches (anomie's open-bus document).
    pub ppu1_mdr: u8,
    pub ppu2_mdr: u8,
    /// RGBA8888, 256x224; not part of the machine's state.
    pub frame: Box<[u8]>,
    /// How far the current line is drawn, in pixels.
    pub drawn: u16,
    pub skip: bool,
    /// $213E's OBJ range and time overflow flags.
    pub range_over: bool,
    pub time_over: bool,
    /// The current line's sprite pixels, 0 where none: the CGRAM index, and the OAM priority in bits 8-9, with
    /// bit 15 set. Drawing's alone; not part of the machine's state.
    pub obj_line: Box<[u16]>,
    /// The four backgrounds' pixels for the current line, in the sprites' encoding; drawing's alone, like `obj_line`.
    pub bg_line: Box<[[u16; WIDTH]; 4]>,
    /// COLDATA's fixed colour, as red, green and blue.
    pub fixed: [u8; 3],
    /// Mosaic's row within its block and the block's height, taken when a block ends (D-15).
    pub mosaic_row: u8,
    pub mosaic_size: u8,
}

impl Default for Ppu {
    fn default() -> Self {
        let mut frame = vec![0u8; WIDTH * HEIGHT * 4];
        for px in frame.chunks_exact_mut(4) {
            px[3] = 0xFF;
        }
        Ppu {
            vram: vec![0; 0x8000].into(),
            cgram: vec![0; 0x100].into(),
            oam: vec![0; 0x220].into(),
            regs: vec![0; 0x40].into(),
            vram_address: 0,
            vram_buffer: 0,
            cgram_address: 0,
            cgram_second: false,
            cgram_low: 0,
            oam_reload: 0,
            oam_rotation: false,
            oam_address: 0,
            oam_low: 0,
            hofs: [0; 4],
            vofs: [0; 4],
            bg_old: 0,
            ophct: 0,
            opvct: 0,
            oph_second: false,
            opv_second: false,
            latched: false,
            ppu1_mdr: 0,
            ppu2_mdr: 0,
            frame: frame.into(),
            drawn: 0,
            skip: false,
            range_over: false,
            time_over: false,
            obj_line: vec![0; WIDTH].into(),
            bg_line: Box::new([[0; WIDTH]; 4]),
            fixed: [0; 3],
            mosaic_row: 0,
            mosaic_size: 1,
        }
    }
}

/// Where the raster stands when a register is touched.
#[derive(Clone, Copy, Debug, Default)]
pub struct Beam {
    pub line: u16,
    pub dot: u16,
    pub vblank: bool,
    pub field: bool,
    pub pal: bool,
}

/// The layers from the front, as (layer, priority), layer 4 the sprites (fullsnes, "Background Priority Chart").
const OBJ: usize = 4;
const MODE0: [(usize, u16); 12] = [(OBJ, 3), (0, 1), (1, 1), (OBJ, 2), (0, 0), (1, 0), (OBJ, 1), (2, 1), (3, 1), (OBJ, 0), (2, 0), (3, 0)];
const MODE1: [(usize, u16); 10] = [(OBJ, 3), (0, 1), (1, 1), (OBJ, 2), (0, 0), (1, 0), (OBJ, 1), (2, 1), (OBJ, 0), (2, 0)];
const MODE1_BG3_HIGH: [(usize, u16); 10] = [(2, 1), (OBJ, 3), (0, 1), (1, 1), (OBJ, 2), (0, 0), (1, 0), (OBJ, 1), (OBJ, 0), (2, 0)];
/// The modes whose backgrounds are later steps': the sprites alone.
const OBJ_ONLY: [(usize, u16); 4] = [(OBJ, 3), (OBJ, 2), (OBJ, 1), (OBJ, 0)];
/// Modes 2, 3 and 4: two backgrounds, each priority of BG2 behind BG1's.
const MODE2: [(usize, u16); 8] = [(OBJ, 3), (0, 1), (OBJ, 2), (1, 1), (OBJ, 1), (0, 0), (OBJ, 0), (1, 0)];
/// Bits a pixel of each background has, by mode; 0 where the mode has no such background or a later step draws it.
const DEPTHS: [[u8; 4]; 8] = [[2, 2, 2, 2], [4, 4, 2, 0], [4, 4, 0, 0], [8, 4, 0, 0], [8, 2, 0, 0], [0; 4], [0; 4], [0; 4]];
/// A line-buffer entry whose low 8 bits are a direct colour and bits 10-12 its palette bits, not a CGRAM index.
const DIRECT: u16 = 0x4000;

/// OBSEL's sizes as (small, large), each (width, height) (fullsnes, OBSEL, with its two undocumented settings).
const OBJ_SIZES: [((u16, u16), (u16, u16)); 8] = [
    ((8, 8), (16, 16)), ((8, 8), (32, 32)), ((8, 8), (64, 64)), ((16, 16), (32, 32)),
    ((16, 16), (64, 64)), ((32, 32), (64, 64)), ((16, 32), (32, 64)), ((16, 32), (32, 32)),
];

impl Ppu {
    pub fn forced_blank(&self) -> bool {
        self.regs[0x00] & 0x80 != 0
    }

    /// VMAIN's address translation: the low 8, 9 or 10 bits rotated left by three.
    fn vram_index(&self) -> usize {
        let a = self.vram_address;
        let translated = match (self.regs[0x15] >> 2) & 3 {
            0 => a,
            1 => (a & 0xFF00) | ((a & 0x001F) << 3) | ((a >> 5) & 7),
            2 => (a & 0xFE00) | ((a & 0x003F) << 3) | ((a >> 6) & 7),
            _ => (a & 0xFC00) | ((a & 0x007F) << 3) | ((a >> 7) & 7),
        };
        (translated & 0x7FFF) as usize
    }

    fn vram_step(&mut self, high: bool) -> bool {
        let stepping = high == (self.regs[0x15] & 0x80 != 0);
        if stepping {
            self.vram_address = self.vram_address.wrapping_add([1, 32, 128, 128][(self.regs[0x15] & 3) as usize]);
        }
        stepping
    }

    /// VRAM takes writes in forced blank and in vertical blank (fullsnes: "accessed only during V-Blank, or Forced Blank").
    fn vram_open(&self, beam: Beam) -> bool {
        self.forced_blank() || beam.vblank
    }

    /// The pixel a dot is, clipped to the picture.
    fn pixel_of(dot: u16) -> u16 {
        dot.saturating_sub(FIRST_DOT).min(WIDTH as u16)
    }

    pub fn write(&mut self, reg: u8, value: u8, beam: Beam) {
        // What the picture read before this write stands for the pixels already passed.
        self.draw_to(beam.line, Self::pixel_of(beam.dot));
        let r = (reg & 0x3F) as usize;
        match r {
            0x00 => {
                // Leaving forced blank on the first line of vertical blank reloads the OAM address too.
                if self.regs[0] & 0x80 != 0 && value & 0x80 == 0 && beam.line == 225 {
                    self.reload_oam();
                }
            }
            0x02 => {
                self.oam_reload = (self.oam_reload & 0x100) | value as u16;
                self.oam_address = self.oam_reload << 1;
            }
            0x03 => {
                self.oam_reload = (self.oam_reload & 0xFF) | ((value as u16 & 1) << 8);
                self.oam_rotation = value & 0x80 != 0;
                self.oam_address = self.oam_reload << 1;
            }
            0x04 => {
                let a = self.oam_address & 0x3FF;
                if a >= 0x200 {
                    self.oam[0x200 + (a as usize & 0x1F)] = value;
                } else if a & 1 == 0 {
                    self.oam_low = value;
                } else {
                    self.oam[a as usize - 1] = self.oam_low;
                    self.oam[a as usize] = value;
                }
                self.oam_address = (a + 1) & 0x3FF;
            }
            0x32 => {
                for (c, bit) in [(0, 0x20), (1, 0x40), (2, 0x80)] {
                    if value & bit != 0 {
                        self.fixed[c] = value & 0x1F;
                    }
                }
            }
            0x0D..=0x14 => {
                let bg = (r - 0x0D) / 2;
                if r & 1 == 1 {
                    self.hofs[bg] = ((value as u16) << 8) | (self.bg_old as u16 & !7) | ((self.hofs[bg] >> 8) & 7);
                } else {
                    self.vofs[bg] = ((value as u16) << 8) | self.bg_old as u16;
                }
                self.bg_old = value;
            }
            0x16 => {
                self.vram_address = (self.vram_address & 0xFF00) | value as u16;
                self.vram_buffer = self.vram[self.vram_index()];
            }
            0x17 => {
                self.vram_address = (self.vram_address & 0x00FF) | (value as u16) << 8;
                self.vram_buffer = self.vram[self.vram_index()];
            }
            0x18 | 0x19 => {
                let high = r == 0x19;
                if self.vram_open(beam) {
                    let i = self.vram_index();
                    let w = self.vram[i];
                    self.vram[i] = if high { (w & 0x00FF) | (value as u16) << 8 } else { (w & 0xFF00) | value as u16 };
                }
                self.vram_step(high);
            }
            0x21 => {
                self.cgram_address = value;
                self.cgram_second = false;
            }
            0x22 => {
                if self.cgram_second {
                    self.cgram[self.cgram_address as usize] = ((value as u16 & 0x7F) << 8) | self.cgram_low as u16;
                    self.cgram_address = self.cgram_address.wrapping_add(1);
                } else {
                    self.cgram_low = value;
                }
                self.cgram_second = !self.cgram_second;
            }
            _ => {}
        }
        self.regs[r] = value;
    }

    /// A register's value, or None where the CPU's own open bus shows.
    pub fn read(&mut self, reg: u8, beam: Beam, wrio: u8, side_effects: bool) -> Option<u8> {
        let r = reg & 0x3F;
        let v = match r {
            // Write-only registers that show PPU1's latch (anomie's open-bus document).
            0x04..=0x06 | 0x08..=0x0A | 0x14..=0x16 | 0x18..=0x1A | 0x24..=0x26 | 0x28..=0x2A => return Some(self.ppu1_mdr),
            0x37 => {
                if side_effects && wrio & 0x80 != 0 {
                    self.latch(beam);
                }
                return None;
            }
            0x38 => {
                let a = self.oam_address & 0x3FF;
                let v = if a >= 0x200 { self.oam[0x200 + (a as usize & 0x1F)] } else { self.oam[a as usize] };
                if side_effects {
                    self.oam_address = (a + 1) & 0x3FF;
                }
                v
            }
            0x39 | 0x3A => {
                let high = r == 0x3A;
                let v = if high { (self.vram_buffer >> 8) as u8 } else { self.vram_buffer as u8 };
                if side_effects {
                    // The prefetch is taken from the old address before the step (fullsnes).
                    let fetched = self.vram[self.vram_index()];
                    if self.vram_step(high) {
                        self.vram_buffer = fetched;
                    }
                }
                v
            }
            0x3B => {
                let w = self.cgram[self.cgram_address as usize];
                let v = if self.cgram_second { ((w >> 8) as u8 & 0x7F) | (self.ppu2_mdr & 0x80) } else { w as u8 };
                if side_effects {
                    if self.cgram_second {
                        self.cgram_address = self.cgram_address.wrapping_add(1);
                    }
                    self.cgram_second = !self.cgram_second;
                }
                v
            }
            0x3C | 0x3D => {
                let (counter, second) = if r == 0x3C { (self.ophct, self.oph_second) } else { (self.opvct, self.opv_second) };
                let v = if second { ((counter >> 8) as u8 & 1) | (self.ppu2_mdr & 0xFE) } else { counter as u8 };
                if side_effects {
                    if r == 0x3C { self.oph_second = !second } else { self.opv_second = !second }
                }
                v
            }
            0x3E => (if self.time_over { 0x80 } else { 0 }) | (if self.range_over { 0x40 } else { 0 }) | (self.ppu1_mdr & 0x10) | 0x01,
            0x3F => {
                let v = (if beam.field { 0x80 } else { 0 }) | (if self.latched { 0x40 } else { 0 }) | (self.ppu2_mdr & 0x20) | (if beam.pal { 0x10 } else { 0 }) | 0x03;
                if side_effects {
                    // The latch flag resets only while WRIO bit 7 is set (anomie's register document, $213F).
                    if wrio & 0x80 != 0 {
                        self.latched = false;
                    }
                    self.oph_second = false;
                    self.opv_second = false;
                }
                v
            }
            _ => return None,
        };
        if side_effects {
            if matches!(r, 0x38..=0x3A | 0x3E) { self.ppu1_mdr = v } else { self.ppu2_mdr = v }
        }
        Some(v)
    }

    /// The counters into OPHCT and OPVCT, and the latch flag (fullsnes, $2137 and WRIO).
    pub fn latch(&mut self, beam: Beam) {
        self.ophct = beam.dot;
        self.opvct = beam.line;
        self.latched = true;
    }

    fn reload_oam(&mut self) {
        self.oam_address = self.oam_reload << 1;
    }

    /// A line has ended: its picture is finished, and the first line of vertical blank reloads the OAM address.
    pub fn end_line(&mut self, line: u16, next: u16) {
        self.draw_to(line, WIDTH as u16);
        self.drawn = 0;
        if next == 225 && !self.forced_blank() {
            self.reload_oam();
        }
        // The overflow flags clear at the end of V-Blank, but not in forced blank (fullsnes, STAT77).
        if next == 0 && !self.forced_blank() {
            self.range_over = false;
            self.time_over = false;
        }
        if (1..=HEIGHT as u16).contains(&next) {
            self.evaluate(next);
            // The first block starts at the top; a new size waits for the current block's end (fullsnes; D-15).
            self.mosaic_row += 1;
            if next == 1 || self.mosaic_row >= self.mosaic_size {
                self.mosaic_row = 0;
                self.mosaic_size = (self.regs[0x06] >> 4) + 1;
            }
        }
    }

    /// The sprites of picture line `line`, chosen during the line before it as anomie's "SPRITES" describes: the
    /// first 32 in range from the first sprite, then up to 34 tiles loaded from the last of them back, setting
    /// $213E's flags. The tiles are decoded a row at a time into the line's sprite pixels unless the picture is skipped.
    fn evaluate(&mut self, line: u16) {
        if !self.skip {
            self.obj_line.fill(0);
        }
        if self.forced_blank() {
            return;
        }
        let obsel = self.regs[0x01];
        let sizes = OBJ_SIZES[(obsel >> 5) as usize];
        // Priority rotation takes the first sprite from the internal word address (anomie; D-12 for its limits).
        let first = if self.oam_rotation { ((self.oam_address >> 2) & 0x7F) as usize } else { 0 };
        let y_line = line - 1;
        let mut range = [0u8; 32];
        let mut n = 0;
        for k in 0..128 {
            let i = (first + k) & 127;
            let (x, _, (w, h)) = self.obj_geometry(i, sizes);
            let row = y_line.wrapping_sub(self.oam[i * 4 + 1] as u16) & 0xFF;
            // An OBJ at X=256 counts as if at 0 for range and time, though it draws off the screen (anomie).
            let xr = if x == -256 { 0 } else { x };
            if row >= h || xr <= -(w as i16) {
                continue;
            }
            if n == 32 {
                self.range_over = true;
                break;
            }
            range[n] = i as u8;
            n += 1;
        }
        let mut tiles = 0;
        'load: for &i in range[..n].iter().rev() {
            let i = i as usize;
            let (x, attr, (w, _)) = self.obj_geometry(i, sizes);
            let xr = if x == -256 { 0 } else { x };
            let mut row = y_line.wrapping_sub(self.oam[i * 4 + 1] as u16) & 0xFF;
            if attr & 0x80 != 0 {
                // A rectangular sprite flips as two square ones (anomie).
                row = (row / w) * w + (w - 1 - row % w);
            }
            let columns = w / 8;
            for c in 0..columns {
                let tx = xr + 8 * c as i16;
                if tx <= -8 || tx >= 256 {
                    continue;
                }
                if tiles == 34 {
                    self.time_over = true;
                    break 'load;
                }
                tiles += 1;
                if !self.skip {
                    let column = if attr & 0x40 != 0 { columns - 1 - c } else { c };
                    self.draw_obj_tile(i, attr, column, row, x + 8 * c as i16);
                }
            }
        }
    }

    /// An OBJ's X (signed, 9 bits), its attribute byte and its (width, height).
    fn obj_geometry(&self, i: usize, sizes: ((u16, u16), (u16, u16))) -> (i16, u8, (u16, u16)) {
        let high = self.oam[0x200 + i / 4] >> ((i & 3) * 2);
        let x = self.oam[i * 4] as i16 | if high & 1 != 0 { -256 } else { 0 };
        (x, self.oam[i * 4 + 3], if high & 2 != 0 { sizes.1 } else { sizes.0 })
    }

    /// One 8-pixel row of an OBJ's tile, decoded once, into the line's sprite pixels at screen X `at`; a sprite loaded
    /// later is one nearer the first, so it covers what is there.
    fn draw_obj_tile(&mut self, i: usize, attr: u8, column: u16, row: u16, at: i16) {
        let obsel = self.regs[0x01] as u16;
        let c = self.oam[i * 4 + 2] as u16;
        // The tile table is 16 by 16 and wraps in each direction (anomie, "Character table in VRAM").
        let tile = ((((c >> 4) + row / 8) & 0x0F) << 4) | (((c & 0x0F) + column) & 0x0F);
        let name = if attr & 1 != 0 { (((obsel >> 3) & 3) + 1) << 12 } else { 0 };
        let word = (((obsel & 7) << 13) + (tile << 4) + name + (row & 7)) & 0x7FFF;
        let (w0, w1) = (self.vram[word as usize], self.vram[((word + 8) & 0x7FFF) as usize]);
        let palette = 128 + (((attr >> 1) & 7) as u16) * 16;
        let priority = ((attr >> 4) & 3) as u16;
        for p in 0..8u16 {
            let x = at + p as i16;
            if !(0..256).contains(&x) {
                continue;
            }
            let bit = if attr & 0x40 != 0 { p } else { 7 - p };
            let colour = ((w0 >> bit) & 1) | ((w0 >> (bit + 7)) & 2) | (((w1 >> bit) & 1) << 2) | (((w1 >> (bit + 8)) & 1) << 3);
            if colour != 0 {
                self.obj_line[x as usize] = 0x8000 | (priority << 8) | (palette + colour);
            }
        }
    }

    /// Draws the pixels of `line` not yet drawn, up to `to`, from the registers as they stand.
    pub fn draw_to(&mut self, line: u16, to: u16) {
        let from = self.drawn;
        if to <= from {
            return;
        }
        self.drawn = to;
        if self.skip || !(1..=HEIGHT as u16).contains(&line) {
            return;
        }
        let row = (line as usize - 1) * WIDTH * 4;
        let brightness = (self.regs[0] & 0x0F) as u32;
        if self.forced_blank() || brightness == 0 {
            for x in from..to {
                let at = row + x as usize * 4;
                self.frame[at..at + 3].fill(0);
            }
            return;
        }
        self.fill_backgrounds(line, from, to);
        let scale = |c: u16| -> u8 {
            let c = (c as u32 & 31) * (brightness + 1) / 16;
            ((c << 3) | (c >> 2)) as u8
        };
        for x in from..to {
            let colour = self.compose(x);
            let at = row + x as usize * 4;
            self.frame[at] = scale(colour);
            self.frame[at + 1] = scale(colour >> 5);
            self.frame[at + 2] = scale(colour >> 10);
        }
    }

    /// Each background's pixels from `from` to `to` into its line buffer, for the layers either screen shows;
    /// mosaic then repeats each block's first pixel (fullsnes, MOSAIC; D-15 for the rows).
    fn fill_backgrounds(&mut self, line: u16, from: u16, to: u16) {
        let mode = self.regs[0x05] & 7;
        let depths = DEPTHS[mode as usize];
        let shown = self.regs[0x2C] | self.regs[0x2D];
        let mosaic = self.regs[0x06];
        let size = (mosaic >> 4) as u16 + 1;
        for bg in 0..4 {
            if depths[bg] == 0 || shown & (1 << bg) == 0 {
                continue;
            }
            let blocks = mosaic & (1 << bg) != 0;
            let row = if blocks { line - self.mosaic_row as u16 } else { line };
            self.fill_row(bg, mode, row, from, to);
            if blocks && size > 1 {
                for x in from..to {
                    let first = x - x % size;
                    self.bg_line[bg][x as usize] = self.bg_line[bg][first as usize];
                }
            }
        }
    }

    /// A background's line buffer from `from` to `to`, a tile row decoded once for each 8 pixels of the background it
    /// covers; a span that starts or ends inside a chunk takes the part of it that falls inside.
    fn fill_row(&mut self, bg: usize, mode: u8, line: u16, from: u16, to: u16) {
        let mut chunk = [0u16; 8];
        let mut x = from;
        while x < to {
            let (hofs, vofs) = self.scroll(bg, mode, x);
            let px = x.wrapping_add(hofs);
            self.decode_chunk(bg, mode, px & !7, line.wrapping_add(vofs), &mut chunk);
            let first = (px & 7) as usize;
            let n = (8 - first).min((to - x) as usize);
            let at = x as usize;
            self.bg_line[bg][at..at + n].copy_from_slice(&chunk[first..first + n]);
            x += n as u16;
        }
    }

    /// The scroll a background is drawn with at screen X: its registers, or in modes 2 and 4 the offsets BG3's map
    /// holds for the visible tile X is in (anomie's "Mode 2" and "Mode 4"; D-16).
    fn scroll(&self, bg: usize, mode: u8, x: u16) -> (u16, u16) {
        let (mut hofs, mut vofs) = (self.hofs[bg], self.vofs[bg]);
        if !matches!(mode, 2 | 4) || bg > 1 {
            return (hofs, vofs);
        }
        let tile = (x + (hofs & 7)) >> 3;
        if tile == 0 {
            return (hofs, vofs);
        }
        let column = ((tile - 1) << 3).wrapping_add(self.hofs[2] & !7);
        let valid = 0x2000 << bg;
        let first = self.map_entry(2, column, self.vofs[2]);
        let (h, v) = if mode == 4 {
            if first & 0x8000 != 0 { (0, first) } else { (first, 0) }
        } else {
            (first, self.map_entry(2, column, self.vofs[2].wrapping_add(8)))
        };
        if h & valid != 0 {
            hofs = (h & 0x03F8) | (hofs & 7);
        }
        if v & valid != 0 {
            vofs = v & 0x03FF;
        }
        (hofs, vofs)
    }

    /// A background's map entry for the point (px, py) of its plane (anomie, "Tile Maps and Character Maps").
    fn map_entry(&self, bg: usize, px: u16, py: u16) -> u16 {
        let shift = if self.regs[0x05] & (0x10 << bg) != 0 { 4 } else { 3 };
        let (tx, ty) = (px >> shift, py >> shift);
        let sc = self.regs[0x07 + bg];
        let screen = match sc & 3 {
            0 => 0,
            1 => (tx >> 5) & 1,
            2 => (ty >> 5) & 1,
            _ => ((tx >> 5) & 1) + 2 * ((ty >> 5) & 1),
        };
        let base = ((sc as u16 >> 2) << 10).wrapping_add(screen << 10);
        self.vram[(base.wrapping_add(((ty & 31) << 5) | (tx & 31)) & 0x7FFF) as usize]
    }

    /// The eight pixels of the background at (px0..px0+8, py), px0 a multiple of 8, in screen order and the line
    /// buffer's encoding: the map entry is read and the tile row's words fetched once for all eight.
    fn decode_chunk(&self, bg: usize, mode: u8, px0: u16, py: u16, out: &mut [u16; 8]) {
        let size = if self.regs[0x05] & (0x10 << bg) != 0 { 16u16 } else { 8 };
        let entry = self.map_entry(bg, px0, py);
        let hflip = entry & 0x4000 != 0;
        let mut fx = px0 & (size - 1);
        let mut fy = py & (size - 1);
        if hflip {
            fx = size - 1 - fx;
        }
        if entry & 0x8000 != 0 {
            fy = size - 1 - fy;
        }
        let tile = ((entry & 0x3FF) + (fx >> 3) + ((fy >> 3) << 4)) & 0x3FF;
        let depth = DEPTHS[mode as usize][bg] as u16;
        let nba = (self.regs[0x0B + bg / 2] >> (4 * (bg & 1))) as u16 & 0x0F;
        let at = (nba << 12).wrapping_add(tile.wrapping_mul(4 * depth)).wrapping_add(fy & 7);
        let mut words = [0u16; 4];
        for (plane, w) in words.iter_mut().enumerate().take(depth as usize / 2) {
            *w = self.vram[(at.wrapping_add(8 * plane as u16) & 0x7FFF) as usize];
        }
        let palette = (entry >> 10) & 7;
        let high = 0x8000 | (((entry >> 13) & 1) << 8);
        // A 256-colour background's pixel is a colour itself in direct colour mode, with the palette bits (anomie).
        let direct = depth == 8 && self.regs[0x30] & 1 != 0;
        let base = match depth {
            2 if mode == 0 => bg as u16 * 0x20 + palette * 4,
            2 => palette * 4,
            4 => palette * 16,
            _ => 0,
        };
        for (i, o) in out.iter_mut().enumerate() {
            let bit = if hflip { i as u16 } else { 7 - i as u16 };
            let mut colour = 0;
            for (plane, w) in words.iter().enumerate().take(depth as usize / 2) {
                colour |= (((w >> bit) & 1) | (((w >> (bit + 8)) & 1) << 1)) << (2 * plane);
            }
            *o = if colour == 0 {
                0
            } else if direct {
                high | DIRECT | (palette << 10) | colour
            } else {
                high | (base + colour)
            };
        }
    }

    /// Whether a layer's window mask covers `x`: each window inside or outside, then the two combined (fullsnes, "SNES
    /// PPU Window"). `sel` is the layer's four bits of W12SEL, W34SEL or WOBJSEL, `logic` its two of WBGLOG or WOBJLOG.
    fn window(&self, sel: u8, logic: u8, x: u16) -> bool {
        let inside = |l: u8, r: u8| (l as u16..=r as u16).contains(&x);
        let w1 = inside(self.regs[0x26], self.regs[0x27]) != (sel & 1 != 0);
        let w2 = inside(self.regs[0x28], self.regs[0x29]) != (sel & 4 != 0);
        match (sel & 2 != 0, sel & 8 != 0) {
            (false, false) => false,
            (true, false) => w1,
            (false, true) => w2,
            (true, true) => match logic & 3 {
                0 => w1 | w2,
                1 => w1 & w2,
                2 => w1 ^ w2,
                _ => !(w1 ^ w2),
            },
        }
    }

    /// The front-most pixel of the layers `enabled` shows and `masked` does not hide inside its window, as (colour,
    /// layer, CGRAM index), the layer 0-3 a background and 4 the sprites; None where all are transparent.
    fn front(&self, enabled: u8, masked: u8, windows: u8, x: u16) -> Option<(u16, usize, u16)> {
        let order: &[(usize, u16)] = match self.regs[0x05] & 7 {
            0 => &MODE0,
            1 if self.regs[0x05] & 0x08 != 0 => &MODE1_BG3_HIGH,
            1 => &MODE1,
            2..=4 => &MODE2,
            _ => &OBJ_ONLY,
        };
        for &(layer, priority) in order {
            let bit = 1 << layer;
            if enabled & bit == 0 || (masked & bit != 0 && windows & bit != 0) {
                continue;
            }
            let p = if layer == OBJ { self.obj_line[x as usize] } else { self.bg_line[layer][x as usize] };
            if p != 0 && (p >> 8) & 3 == priority {
                let colour = if layer != OBJ && p & DIRECT != 0 {
                    // BBGGGRRR and the palette's bgr make Red=RRRr0, Green=GGGg0, Blue=BBb00 (anomie, "Direct Color Mode").
                    let (c, bgr) = (p & 0xFF, (p >> 10) & 7);
                    ((c & 7) << 2 | (bgr & 1) << 1) | ((c >> 3 & 7) << 2 | (bgr >> 1 & 1) << 1) << 5 | ((c >> 6) << 3 | (bgr >> 2) << 2) << 10
                } else {
                    self.cgram[(p & 0xFF) as usize]
                };
                return Some((colour, layer, p & 0xFF));
            }
        }
        None
    }

    /// One pixel of the picture: the main screen's front-most pixel, clipped to black and mathed with the sub screen
    /// or the fixed colour as the colour window and CGWSEL and CGADSUB say (fullsnes, "SNES PPU Color-Math"; anomie's
    /// "RENDERING THE SCREEN"; D-13).
    fn compose(&self, x: u16) -> u16 {
        let mut windows = 0u8;
        for (layer, (sel, logic)) in [
            (self.regs[0x23], self.regs[0x2A]),
            (self.regs[0x23] >> 4, self.regs[0x2A] >> 2),
            (self.regs[0x24], self.regs[0x2A] >> 4),
            (self.regs[0x24] >> 4, self.regs[0x2A] >> 6),
            (self.regs[0x25], self.regs[0x2B]),
            (self.regs[0x25] >> 4, self.regs[0x2B] >> 2),
        ]
        .into_iter()
        .enumerate()
        {
            if self.window(sel & 0x0F, logic, x) {
                windows |= 1 << layer;
            }
        }
        let (main, layer, index) = self.front(self.regs[0x2C], self.regs[0x2E], windows, x).unwrap_or((self.cgram[0], 5, 0));
        let cgwsel = self.regs[0x30];
        let cgadsub = self.regs[0x31];
        let colour_window = windows & 0x20 != 0;
        let region = |v: u8| match v & 3 {
            0 => false,
            1 => !colour_window,
            2 => colour_window,
            _ => true,
        };
        let clip = region(cgwsel >> 6);
        let main = if clip { 0 } else { main };
        // Only sprites with palettes 4 to 7 take part (fullsnes, CGADSUB).
        let takes_part = if layer == OBJ { index >= 0xC0 && cgadsub & 0x10 != 0 } else { cgadsub & (1 << layer) != 0 };
        if region(cgwsel >> 4) || !takes_part {
            return main;
        }
        let fixed = (self.fixed[0] as u16) | (self.fixed[1] as u16) << 5 | (self.fixed[2] as u16) << 10;
        let (sub, sub_backdrop) = if cgwsel & 2 != 0 {
            match self.front(self.regs[0x2D], self.regs[0x2F], windows, x) {
                Some((c, _, _)) => (c, false),
                None => (fixed, true),
            }
        } else {
            (fixed, false)
        };
        let half = cgadsub & 0x40 != 0 && !clip && !sub_backdrop;
        let mut out = 0;
        for shift in [0, 5, 10] {
            let (m, s) = ((main >> shift) & 31, (sub >> shift) & 31);
            let mut c = if cgadsub & 0x80 != 0 { m.saturating_sub(s) } else { m + s };
            if half {
                c >>= 1;
            }
            out |= c.min(31) << shift;
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blank() -> Beam {
        Beam { line: 230, dot: 0, vblank: true, ..Beam::default() }
    }

    // fullsnes: 8, 9 and 10 bits rotated left by three; the prefetch taken before the step; a write leaves it alone.
    #[test]
    fn the_vram_port_translates_steps_and_prefetches_as_documented() {
        let mut p = Ppu::default();
        for (i, w) in p.vram.iter_mut().enumerate() {
            *w = i as u16;
        }
        for (mode, address, want) in [(0x00u8, 0x1234u16, 0x1234usize), (0x04, 0x12E5, 0x122F), (0x08, 0x13C5, 0x122F), (0x0C, 0x1385, 0x102F)] {
            p.regs[0x15] = mode;
            p.vram_address = address;
            assert_eq!(p.vram_index(), want, "VMAIN {mode:02X}");
        }
        p.write(0x15, 0x80, blank());
        p.write(0x16, 0x00, blank());
        p.write(0x17, 0x10, blank());
        let reads: Vec<u8> = (0..3).flat_map(|_| [p.read(0x39, blank(), 0, true).unwrap(), p.read(0x3A, blank(), 0, true).unwrap()]).collect();
        assert_eq!(reads, [0x00, 0x10, 0x00, 0x10, 0x01, 0x10]);
        let buffer = p.vram_buffer;
        p.write(0x18, 0xAA, blank());
        p.write(0x19, 0xBB, blank());
        assert_eq!((p.vram[0x1003], p.vram_buffer), (0xBBAA, buffer));
    }

    #[test]
    fn oam_and_cgram_latch_their_low_bytes_and_the_counters_latch_on_2137() {
        let mut p = Ppu::default();
        p.write(0x02, 0x05, blank());
        p.write(0x04, 0x11, blank());
        assert_eq!(p.oam[0x0A], 0);
        p.write(0x04, 0x22, blank());
        assert_eq!((p.oam[0x0A], p.oam[0x0B], p.oam_address), (0x11, 0x22, 0x0C));
        p.write(0x03, 0x01, blank());
        p.write(0x04, 0x77, blank());
        assert_eq!(p.oam[0x20A], 0x77);
        p.end_line(224, 225);
        assert_eq!(p.oam_address, 0x20A);
        p.write(0x21, 0x10, blank());
        p.write(0x22, 0xFF, blank());
        p.write(0x22, 0xFF, blank());
        assert_eq!((p.cgram[0x10], p.cgram_address), (0x7FFF, 0x11));
        let beam = Beam { line: 0x105, dot: 0x123, ..Beam::default() };
        assert_eq!(p.read(0x37, beam, 0x00, true), None);
        assert!(!p.latched);
        p.read(0x37, beam, 0x80, true);
        assert_eq!(p.read(0x3F, beam, 0, false).unwrap() & 0x40, 0x40);
        let h = [p.read(0x3C, beam, 0, true).unwrap(), p.read(0x3C, beam, 0, true).unwrap() & 1];
        let v = [p.read(0x3D, beam, 0, true).unwrap(), p.read(0x3D, beam, 0, true).unwrap() & 1];
        assert_eq!((h, v), ([0x23, 1], [0x05, 1]));
        p.read(0x3F, beam, 0, true);
        assert!(p.latched && !p.oph_second);
        p.read(0x3F, beam, 0x80, true);
        assert!(!p.latched);
    }

    // One 2bpp tile in mode 0 on BG1, its second pixel colour 3, with the backdrop behind and a mid-line change.
    #[test]
    fn a_line_is_drawn_in_spans_from_the_registers_as_they_stood() {
        let mut p = Ppu::default();
        p.vram[0x1000] = 0x4040;
        p.cgram[0] = 0x001F;
        p.cgram[3] = 0x7C00;
        for (r, v) in [(0x00u8, 0x0Fu8), (0x05, 0x00), (0x07, 0x04), (0x0B, 0x01), (0x2C, 0x01), (0x0E, 0xFF), (0x0E, 0xFF)] {
            p.write(r, v, blank());
        }
        p.vram[0x0400] = 0x0000;
        let at = |p: &Ppu, x: usize| [p.frame[x * 4], p.frame[x * 4 + 1], p.frame[x * 4 + 2]];
        p.write(0x2C, 0x00, Beam { line: 1, dot: FIRST_DOT + 100, ..Beam::default() });
        p.end_line(1, 2);
        assert_eq!(at(&p, 0), [0xFF, 0, 0]);
        assert_eq!(at(&p, 1), [0, 0, 0xFF]);
        assert_eq!(at(&p, 9), [0, 0, 0xFF]);
        assert_eq!(at(&p, 105), [0xFF, 0, 0]);
        let mut skipped = Ppu::default();
        skipped.skip = true;
        skipped.write(0x00, 0x0F, blank());
        skipped.end_line(1, 2);
        assert!(skipped.frame.chunks_exact(4).all(|px| px[..3] == [0, 0, 0]));
    }

    // anomie's limits: 32 sprites in range, 34 tiles loaded from the last of them back, flags kept when skipped.
    #[test]
    fn sprites_overflow_at_32_in_range_and_34_tiles_and_the_first_wins() {
        for skip in [false, true] {
            let mut p = Ppu::default();
            p.skip = skip;
            p.regs[0x2C] = 0x10;
            for i in 0..128 {
                p.oam[i * 4 + 1] = 200;
            }
            for i in 0..33 {
                p.oam[i * 4] = (i * 7) as u8;
                p.oam[i * 4 + 1] = 9;
            }
            p.end_line(9, 10);
            assert!(p.range_over && !p.time_over, "skip {skip}");
            for i in 0..18 {
                p.oam[i * 4] = (i * 14) as u8;
            }
            for i in 18..33 {
                p.oam[i * 4 + 1] = 200;
            }
            p.regs[0x01] = 0x00;
            p.oam[0x200..0x205].copy_from_slice(&[0xAA; 5]);
            p.range_over = false;
            p.end_line(9, 10);
            assert!(p.time_over && !p.range_over, "skip {skip}");
        }
        let mut p = Ppu::default();
        p.regs[0x2C] = 0x10;
        for i in 0..128 {
            p.oam[i * 4 + 1] = 200;
        }
        p.vram[0] = 0x00FF;
        p.cgram[0x81] = 0x001F;
        p.cgram[0x91] = 0x03E0;
        p.oam[0..4].copy_from_slice(&[10, 0, 0, 0x00]);
        p.oam[4..8].copy_from_slice(&[12, 0, 0, 0x02]);
        p.end_line(0, 1);
        assert_eq!((p.obj_line[9], p.obj_line[10], p.obj_line[13], p.obj_line[18]), (0, 0x8081, 0x8081, 0x8091));
        p.oam_rotation = true;
        p.oam_address = 4;
        p.end_line(0, 1);
        assert_eq!((p.obj_line[10], p.obj_line[13]), (0x8081, 0x8091));
    }

    // fullsnes and anomie: windows per layer, the colour window's clip and prevent, add and subtract, halving
    // except when clipped or on the sub backdrop, sprites only with palettes 4 to 7.
    #[test]
    fn windows_and_colour_math_compose_from_the_line_buffers() {
        let mut p = Ppu::default();
        p.regs[0x05] = 1;
        p.regs[0x2C] = 0x11;
        p.cgram[0] = 0x0000;
        p.cgram[1] = 10 | 20 << 5 | 30 << 10;
        p.cgram[0xC1] = 0x001F;
        p.cgram[0x81] = 0x03E0;
        p.bg_line[0] = [0x8001; WIDTH];
        p.write(0x32, 0xE0 | 4, blank());
        p.write(0x31, 0x01, blank());
        assert_eq!(p.compose(0), 14 | 24 << 5 | 31 << 10);
        p.write(0x31, 0x41, blank());
        assert_eq!(p.compose(0), 7 | 12 << 5 | 17 << 10);
        p.write(0x31, 0x81, blank());
        assert_eq!(p.compose(0), 6 | 16 << 5 | 26 << 10);
        // Window 1 over x 10..=20 hides BG1 there, so the backdrop shows, unmathed.
        p.write(0x31, 0x00, blank());
        p.write(0x26, 10, blank());
        p.write(0x27, 20, blank());
        p.write(0x23, 0x02, blank());
        p.write(0x2E, 0x01, blank());
        assert_eq!((p.compose(9), p.compose(10), p.compose(20), p.compose(21)), (p.cgram[1], 0, 0, p.cgram[1]));
        // The colour window clips to black inside it, and the sub backdrop, not halved, is what math adds there.
        p.write(0x2E, 0x00, blank());
        p.write(0x25, 0x20, blank());
        p.write(0x30, 0xC2, blank());
        p.write(0x31, 0x41, blank());
        assert_eq!(p.compose(15), 4 | 4 << 5 | 4 << 10);
        // A sprite with palette 4 to 7 takes part; one with palette 0 to 3 does not.
        p.write(0x30, 0x00, blank());
        p.write(0x31, 0x10, blank());
        p.obj_line[30] = 0x8000 | 3 << 8 | 0xC1;
        p.obj_line[31] = 0x8000 | 3 << 8 | 0x81;
        assert_eq!((p.compose(30), p.compose(31)), (31 | 4 << 5 | 4 << 10, 0x03E0));
    }

    // fullsnes: each block of the mosaic shows its upper-left pixel, the first block at the top-left of the picture.
    #[test]
    fn mosaic_repeats_the_blocks_first_pixel_and_first_line() {
        let mut p = Ppu::default();
        p.regs[0x2C] = 0x01;
        p.regs[0x0B] = 0x01;
        // Tile 0 of BG1's characters at $1000: row r's pixel x has colour 1 only where x == r.
        for r in 0..8 {
            p.vram[0x1000 + r] = 0x80 >> r;
        }
        p.cgram[1] = 0x7FFF;
        let lit = |p: &mut Ppu, line: u16| -> Vec<u16> {
            p.fill_backgrounds(line, 0, 16);
            (0..16).filter(|&x| p.bg_line[0][x] != 0).map(|x| x as u16).collect()
        };
        p.end_line(0, 1);
        assert_eq!(lit(&mut p, 2), [2, 10]);
        p.regs[0x06] = 0x31;
        p.end_line(0, 1);
        // Blocks of four: line 1 is the block's first line, whose only lit pixel is x=1, not a block's first.
        assert_eq!(lit(&mut p, 1), Vec::<u16>::new());
        p.end_line(1, 2);
        p.end_line(2, 3);
        p.end_line(3, 4);
        assert_eq!(p.mosaic_row, 3);
        assert_eq!(lit(&mut p, 4), Vec::<u16>::new());
        p.end_line(4, 5);
        // Line 5 starts the second block: its row lights x=5 and x=13, neither a block's first pixel; scrolled by one
        // the lit pixel is x=4, the block's first, and fills the block.
        assert_eq!(p.mosaic_row, 0);
        p.hofs[0] = 1;
        assert_eq!(lit(&mut p, 5), [4, 5, 6, 7, 12, 13, 14, 15]);
    }

    // anomie's "Mode 2", as D-16 reads it: visible tile T takes its scroll from BG3's visible tile T-1.
    #[test]
    fn offset_per_tile_takes_each_visible_tiles_scroll_from_bg3() {
        let mut p = Ppu::default();
        p.regs[0x05] = 2;
        p.regs[0x09] = 0x10;
        p.hofs[0] = 3;
        p.vofs[0] = 5;
        assert_eq!(p.scroll(0, 2, 0), (3, 5));
        assert_eq!(p.scroll(0, 2, 5), (3, 5));
        // BG3's map at $1000: column 0 holds tile 1's offsets, H in row 0 and V in row 1.
        p.vram[0x1000] = 0x2000 | 0x48;
        p.vram[0x1020] = 0x2000 | 0x30;
        p.vram[0x1001] = 0x4000 | 0x80;
        assert_eq!(p.scroll(0, 2, 5), (0x48 | 3, 0x30));
        assert_eq!(p.scroll(0, 2, 12), (0x48 | 3, 0x30));
        // Tile 2's entry is marked for BG2 only, so BG1 keeps its registers there.
        assert_eq!(p.scroll(0, 2, 13), (3, 5));
        assert_eq!(p.scroll(1, 2, 8), (0, 0));
        assert_eq!(p.scroll(1, 2, 16), (0x80, 0));
        // Mode 4 reads one entry: bit 15 makes it the vertical offset.
        p.vram[0x1000] = 0x8000 | 0x2000 | 0x48;
        assert_eq!(p.scroll(0, 4, 5), (3, 0x48));
    }
}
