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
    /// The picture as last presented, RGBA8888, `frame_width` by 224 at the start of the buffer; not part of the
    /// machine's state.
    pub frame: Box<[u8]>,
    /// 256, or 512 for a frame with a hi-res line in it; 224, or 448 for one with an interlaced hi-res line.
    pub frame_width: u16,
    pub frame_height: u16,
    /// The field being drawn, and whether a line of it was interlaced hi-res; the clock's, for drawing.
    pub field: bool,
    pub tall: bool,
    /// The frame being drawn, always 512 wide, a low-res pixel written twice; presented at the start of V-Blank.
    pub canvas: Box<[u8]>,
    /// Whether a line of the frame being drawn was hi-res.
    pub wide: bool,
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
    /// Modes 5 and 6: BG1's and BG2's even half-pixels, the sub screen's; `bg_line` holds the odd ones, the main's.
    pub bg_sub_line: Box<[[u16; WIDTH]; 2]>,
    /// What the compositor did at the pixel before, which decides the next sub half-pixel's math in hi-res (D-19).
    pub before: Mix,
    /// COLDATA's fixed colour, as red, green and blue.
    pub fixed: [u8; 3],
    /// Mode 7's matrix A to D, centre, and scroll as written through the shared byte M7_old (fullsnes, "M7xx").
    pub m7: [u16; 8],
    pub m7_old: u8,
    /// Mosaic's row within its block and the block's height, taken when a block ends (D-15).
    pub mosaic_row: u8,
    pub mosaic_size: u8,
}

impl Default for Ppu {
    fn default() -> Self {
        let mut frame = vec![0u8; 2 * WIDTH * 2 * HEIGHT * 4];
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
            canvas: frame[..2 * WIDTH * HEIGHT * 4].to_vec().into(),
            frame: frame.into(),
            frame_width: WIDTH as u16,
            frame_height: HEIGHT as u16,
            field: false,
            tall: false,
            wide: false,
            drawn: 0,
            skip: false,
            range_over: false,
            time_over: false,
            obj_line: vec![0; WIDTH].into(),
            bg_line: Box::new([[0; WIDTH]; 4]),
            bg_sub_line: Box::new([[0; WIDTH]; 2]),
            before: Mix::default(),
            fixed: [0; 3],
            m7: [0; 8],
            m7_old: 0,
            mosaic_row: 0,
            mosaic_size: 1,
        }
    }
}

/// A composed main-screen pixel and how it was made: the colour shown; whether math was done, with the fixed colour
/// (1) or the sub screen's pixel (2); the main colour before math; and the clip, subtract and half that applied.
#[derive(Clone, Copy, Debug, Default)]
pub struct Mix {
    pub colour: u16,
    pub math: u8,
    pub before: u16,
    pub clip: bool,
    pub subtract: bool,
    pub half: bool,
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
/// Modes 2, 3 and 4: two backgrounds, each priority of BG2 behind BG1's.
const MODE2: [(usize, u16); 8] = [(OBJ, 3), (0, 1), (OBJ, 2), (1, 1), (OBJ, 1), (0, 0), (OBJ, 0), (1, 0)];
/// Bits a pixel of each background has, by mode; 0 where the mode has no such background or a later step draws it.
const DEPTHS: [[u8; 4]; 8] = [[2, 2, 2, 2], [4, 4, 2, 0], [4, 4, 0, 0], [8, 4, 0, 0], [8, 2, 0, 0], [4, 2, 0, 0], [4, 0, 0, 0], [0; 4]];
/// Mode 6: one background.
const MODE6: [(usize, u16); 6] = [(OBJ, 3), (0, 1), (OBJ, 2), (OBJ, 1), (0, 0), (OBJ, 0)];
/// Mode 7, and with EXTBG its BG2 with a priority bit per pixel (anomie, "Mode 7").
const MODE7: [(usize, u16); 5] = [(OBJ, 3), (OBJ, 2), (OBJ, 1), (0, 0), (OBJ, 0)];
const MODE7_EXTBG: [(usize, u16); 7] = [(OBJ, 3), (OBJ, 2), (1, 1), (OBJ, 1), (0, 0), (OBJ, 0), (1, 0)];
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
            0x1B..=0x20 => {
                self.m7[r - 0x1B] = ((value as u16) << 8) | self.m7_old as u16;
                self.m7_old = value;
            }
            0x0D..=0x14 => {
                // $210D and $210E are mode 7's scroll as well, through M7_old (fullsnes, "M7HOVS/M7VOFS Port Notes").
                if r <= 0x0E {
                    self.m7[6 + (r - 0x0D)] = ((value as u16) << 8) | self.m7_old as u16;
                    self.m7_old = value;
                }
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
            // M7A times M7B's high byte, signed (fullsnes, MPYL); the products during mode 7's drawing are not modelled.
            0x34..=0x36 => {
                let product = (self.m7[0] as i16 as i32) * ((self.m7[1] >> 8) as i8 as i32);
                (product >> (8 * (r - 0x34))) as u8
            }
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
            if matches!(r, 0x34..=0x36 | 0x38..=0x3A | 0x3E) { self.ppu1_mdr = v } else { self.ppu2_mdr = v }
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
        if next == 225 && !self.skip {
            self.present();
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

    /// The finished frame becomes the picture: 512 wide if a line of it was hi-res, else every other pixel of the
    /// canvas, which is the 256-wide picture exactly; 448 high, the fields woven, if a line was interlaced hi-res.
    fn present(&mut self) {
        let row = 2 * WIDTH * 4;
        if self.tall {
            // Each field's lines go to its own rows of the 448; a first interlaced frame fills both.
            let both = self.frame_height != 2 * HEIGHT as u16;
            for y in 0..HEIGHT {
                let line = &self.canvas[y * row..(y + 1) * row];
                for half in 0..2 {
                    if both || half == self.field as usize {
                        self.frame[(2 * y + half) * row..(2 * y + half + 1) * row].copy_from_slice(line);
                    }
                }
            }
            self.frame_width = 2 * WIDTH as u16;
            self.frame_height = 2 * HEIGHT as u16;
        } else if self.wide {
            self.frame[..self.canvas.len()].copy_from_slice(&self.canvas);
            self.frame_width = 2 * WIDTH as u16;
            self.frame_height = HEIGHT as u16;
        } else {
            for (out, pair) in self.frame.chunks_exact_mut(4).zip(self.canvas.chunks_exact(8)) {
                out.copy_from_slice(&pair[..4]);
            }
            self.frame_width = WIDTH as u16;
            self.frame_height = HEIGHT as u16;
        }
        self.wide = false;
        self.tall = false;
    }

    /// The presented picture: `frame_width` by `frame_height` pixels of RGBA.
    pub fn picture(&self) -> &[u8] {
        &self.frame[..self.frame_width as usize * self.frame_height as usize * 4]
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
        // SETINI bit 1: the sprites take every other row by the field, and show at half their height (anomie, SETINI).
        let obj_interlace = self.regs[0x33] & 2 != 0;
        let mut range = [0u8; 32];
        let mut n = 0;
        for k in 0..128 {
            let i = (first + k) & 127;
            let (x, _, (w, h)) = self.obj_geometry(i, sizes);
            let row = y_line.wrapping_sub(self.oam[i * 4 + 1] as u16) & 0xFF;
            // An OBJ at X=256 counts as if at 0 for range and time, though it draws off the screen (anomie).
            let xr = if x == -256 { 0 } else { x };
            if row >= h >> obj_interlace as u16 || xr <= -(w as i16) {
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
            if obj_interlace {
                row = 2 * row + self.field as u16;
            }
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
        let row = (line as usize - 1) * 2 * WIDTH * 4;
        let brightness = (self.regs[0] & 0x0F) as u32;
        if self.forced_blank() || brightness == 0 {
            for x in from..to {
                let at = row + x as usize * 8;
                self.canvas[at..at + 3].fill(0);
                self.canvas[at + 4..at + 7].fill(0);
            }
            return;
        }
        self.fill_backgrounds(line, from, to);
        let scale = |c: u16| -> u8 {
            let c = (c as u32 & 31) * (brightness + 1) / 16;
            ((c << 3) | (c >> 2)) as u8
        };
        // Modes 5 and 6 and SETINI's pseudo-hi-res show the sub screen's pixel left of the main's (anomie, "Mode 5").
        let hires = self.hires();
        self.wide |= hires;
        self.tall |= self.half_lines();
        if from == 0 {
            self.before = Mix::default();
        }
        // The masks change only at the windows' edges, so they are found once for each run between them (decided 2026-10-02).
        let mut windows = 0;
        let mut until = from;
        for x in from..to {
            if x == until {
                windows = self.windows_at(x);
                until = self.next_window_edge(x).min(to);
            }
            let mix = self.mix(x, windows);
            let left = if hires { self.sub_half_pixel(x, self.before, windows) } else { mix.colour };
            self.before = mix;
            let at = row + x as usize * 8;
            for (at, colour) in [(at, left), (at + 4, mix.colour)] {
                self.canvas[at] = scale(colour);
                self.canvas[at + 1] = scale(colour >> 5);
                self.canvas[at + 2] = scale(colour >> 10);
            }
        }
    }

    /// Modes 5 and 6 with SETINI's interlace draw the even or the odd half-lines by the field (anomie, "Mode 5").
    fn half_lines(&self) -> bool {
        matches!(self.regs[0x05] & 7, 5 | 6) && self.regs[0x33] & 1 != 0
    }

    fn hires(&self) -> bool {
        matches!(self.regs[0x05] & 7, 5 | 6) || self.regs[0x33] & 0x08 != 0
    }

    /// Each background's pixels from `from` to `to` into its line buffer, for the layers either screen shows;
    /// mosaic then repeats each block's first pixel (fullsnes, MOSAIC; D-15 for the rows).
    fn fill_backgrounds(&mut self, line: u16, from: u16, to: u16) {
        let mode = self.regs[0x05] & 7;
        let depths = DEPTHS[mode as usize];
        let shown = self.regs[0x2C] | self.regs[0x2D];
        let mosaic = self.regs[0x06];
        let size = (mosaic >> 4) as u16 + 1;
        if mode == 7 {
            self.fill_mode7(line, from, to, shown, mosaic, size);
            return;
        }
        for bg in 0..4 {
            if depths[bg] == 0 || shown & (1 << bg) == 0 {
                continue;
            }
            let blocks = mosaic & (1 << bg) != 0;
            let row = if blocks { line - self.mosaic_row as u16 } else { line };
            // In half-lines the picture's line L is rows 2L and 2L+1, the field choosing which.
            // A mosaic block there is two half-lines high at the least, so both fields show the even one (anomie, "Mosaic").
            let row = if self.half_lines() { 2 * row + (self.field && !blocks) as u16 } else { row };
            self.fill_row(bg, mode, row, from, to);
            if blocks && matches!(mode, 5 | 6) {
                // In true hi-res a block's first half-pixel, a sub-screen one, fills both screens' (fullsnes, "Hires Notes").
                for x in from..to {
                    let first = (x - x % size) as usize;
                    self.bg_sub_line[bg][x as usize] = self.bg_sub_line[bg][first];
                    self.bg_line[bg][x as usize] = self.bg_sub_line[bg][first];
                }
            } else if blocks && size > 1 {
                for x in from..to {
                    let first = x - x % size;
                    self.bg_line[bg][x as usize] = self.bg_line[bg][first as usize];
                }
            }
        }
    }

    /// Mode 7's BG1, and with EXTBG its BG2 from the same pixels, for `from` to `to`. Mosaic repeats as for the other
    /// modes, but EXTBG's BG2 takes its vertical blocks from BG1's enable bit and its horizontal ones from its own (anomie).
    fn fill_mode7(&mut self, line: u16, from: u16, to: u16, shown: u8, mosaic: u8, size: u16) {
        let extbg = self.regs[0x33] & 0x40 != 0;
        let direct = self.regs[0x30] & 1 != 0;
        for (bg, vertical, horizontal) in [(0usize, mosaic & 1 != 0, mosaic & 1 != 0), (1, mosaic & 1 != 0, mosaic & 2 != 0)] {
            if shown & (1 << bg) == 0 || (bg == 1 && !extbg) {
                continue;
            }
            let row = if vertical { line - self.mosaic_row as u16 } else { line };
            for x in from..to {
                let sx = if horizontal { x - x % size } else { x };
                let p = self.mode7_pixel(sx, row);
                self.bg_line[bg][x as usize] = match (bg, p) {
                    (_, 0) => 0,
                    (0, p) if direct => 0x8000 | DIRECT | p as u16,
                    (0, p) => 0x8000 | p as u16,
                    (_, p) if p & 0x7F == 0 => 0,
                    (_, p) => 0x8000 | ((p as u16 >> 7) << 8) | (p as u16 & 0x7F),
                };
            }
        }
    }

    /// The 8-bit pixel of mode 7's playing field that screen pixel (sx, sy) shows, 0 if transparent: fullsnes's and
    /// anomie's formula, the origin's products and the line's rounded to 1/4 pixel, the screen flipped by M7SEL, and
    /// the outside of the 1024-pixel field wrapped, transparent or tile 0's.
    fn mode7_pixel(&self, sx: u16, sy: u16) -> u8 {
        let sel = self.regs[0x1A];
        let signed13 = |v: u16| ((v as i32) << 19) >> 19;
        let clip = |v: i32| if v & 0x2000 != 0 { v | !0x3FF } else { v & 0x3FF };
        let (a, b, c, d) = (self.m7[0] as i16 as i32, self.m7[1] as i16 as i32, self.m7[2] as i16 as i32, self.m7[3] as i16 as i32);
        let (cx, cy) = (signed13(self.m7[4]), signed13(self.m7[5]));
        let (ox, oy) = (clip(signed13(self.m7[6]) - cx), clip(signed13(self.m7[7]) - cy));
        let x = (if sel & 1 != 0 { sx ^ 0xFF } else { sx } & 0xFF) as i32;
        let y = (if sel & 2 != 0 { sy ^ 0xFF } else { sy } & 0xFF) as i32;
        let vx = ((a * ox) & !63) + ((b * oy) & !63) + ((b * y) & !63) + (cx << 8) + a * x;
        let vy = ((c * ox) & !63) + ((d * oy) & !63) + ((d * y) & !63) + (cy << 8) + c * x;
        let (px, py) = (vx >> 8, vy >> 8);
        let tile = if (px | py) & !0x3FF != 0 && sel & 0x80 != 0 {
            if sel & 0x40 == 0 {
                return 0;
            }
            0
        } else {
            self.vram[((((py & 0x3FF) >> 3) << 7) | ((px & 0x3FF) >> 3)) as usize] & 0xFF
        };
        (self.vram[((tile << 6) | (((py & 7) as u16) << 3) | (px & 7) as u16) as usize] >> 8) as u8
    }

    /// A background's line buffer from `from` to `to`, a tile row decoded once for each 8 pixels of the background it
    /// covers; a span that starts or ends inside a chunk takes the part of it that falls inside.
    fn fill_row(&mut self, bg: usize, mode: u8, line: u16, from: u16, to: u16) {
        let mut chunk = [0u16; 8];
        let mut x = from;
        while x < to {
            let (hofs, vofs) = self.scroll(bg, mode, x);
            let px = x.wrapping_add(hofs);
            let first = (px & 7) as usize;
            let n = (8 - first).min((to - x) as usize);
            let at = x as usize;
            if matches!(mode, 5 | 6) {
                let mut sub = [0u16; 8];
                self.decode_chunk_hires(bg, mode, px & !7, line.wrapping_add(vofs), &mut chunk, &mut sub);
                self.bg_sub_line[bg][at..at + n].copy_from_slice(&sub[first..first + n]);
            } else {
                self.decode_chunk(bg, mode, px & !7, line.wrapping_add(vofs), &mut chunk);
            }
            self.bg_line[bg][at..at + n].copy_from_slice(&chunk[first..first + n]);
            x += n as u16;
        }
    }

    /// The scroll a background is drawn with at screen X: its registers, or in modes 2 and 4 the offsets BG3's map
    /// holds for the visible tile X is in (anomie's "Mode 2" and "Mode 4"; D-16).
    fn scroll(&self, bg: usize, mode: u8, x: u16) -> (u16, u16) {
        let (mut hofs, mut vofs) = (self.hofs[bg], self.vofs[bg]);
        if !matches!(mode, 2 | 4 | 6) || bg > 1 {
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
        // Modes 5 and 6 take tiles 16 half-pixels wide whatever the size bit says, which is 8 pixels (anomie).
        let across = if matches!(self.regs[0x05] & 7, 5 | 6) { 3 } else { shift };
        let (tx, ty) = (px >> across, py >> shift);
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
        let depth = DEPTHS[mode as usize][bg] as u16;
        let words = self.tile_row(bg, depth, (entry & 0x3FF) + (fx >> 3) + ((fy >> 3) << 4), fy);
        for (i, o) in out.iter_mut().enumerate() {
            let bit = if hflip { i as u16 } else { 7 - i as u16 };
            *o = self.encode(bg, mode, depth, entry, &words, bit);
        }
    }

    /// Modes 5 and 6: the tile is 16 half-pixels wide, two tiles side by side; the even half-pixels are the sub
    /// screen's and the odd ones the main's (anomie, "Mode 5").
    fn decode_chunk_hires(&self, bg: usize, mode: u8, px0: u16, py: u16, main: &mut [u16; 8], sub: &mut [u16; 8]) {
        let size = if self.regs[0x05] & (0x10 << bg) != 0 { 16u16 } else { 8 };
        let entry = self.map_entry(bg, px0, py);
        let hflip = entry & 0x4000 != 0;
        let mut fy = py & (size - 1);
        if entry & 0x8000 != 0 {
            fy = size - 1 - fy;
        }
        let depth = DEPTHS[mode as usize][bg] as u16;
        let tile = (entry & 0x3FF) + ((fy >> 3) << 4);
        let pair = [self.tile_row(bg, depth, tile, fy), self.tile_row(bg, depth, tile + 1, fy)];
        for half in 0..16u16 {
            let column = if hflip { 15 - half } else { half };
            let value = self.encode(bg, mode, depth, entry, &pair[(column >> 3) as usize], 7 - (column & 7));
            if half & 1 == 0 {
                sub[(half >> 1) as usize] = value;
            } else {
                main[(half >> 1) as usize] = value;
            }
        }
    }

    /// The words of one row of a tile's bitplanes, two planes a word.
    fn tile_row(&self, bg: usize, depth: u16, tile: u16, fy: u16) -> [u16; 4] {
        let nba = (self.regs[0x0B + bg / 2] >> (4 * (bg & 1))) as u16 & 0x0F;
        let at = (nba << 12).wrapping_add((tile & 0x3FF).wrapping_mul(4 * depth)).wrapping_add(fy & 7);
        let mut words = [0u16; 4];
        for (plane, w) in words.iter_mut().enumerate().take(depth as usize / 2) {
            *w = self.vram[(at.wrapping_add(8 * plane as u16) & 0x7FFF) as usize];
        }
        words
    }

    /// One pixel of a tile row as a line-buffer entry: 0 if transparent, else its priority and CGRAM index, or its
    /// direct colour.
    fn encode(&self, bg: usize, mode: u8, depth: u16, entry: u16, words: &[u16; 4], bit: u16) -> u16 {
        let mut colour = 0;
        for (plane, w) in words.iter().enumerate().take(depth as usize / 2) {
            colour |= (((w >> bit) & 1) | (((w >> (bit + 8)) & 1) << 1)) << (2 * plane);
        }
        if colour == 0 {
            return 0;
        }
        let palette = (entry >> 10) & 7;
        let high = 0x8000 | (((entry >> 13) & 1) << 8);
        // A 256-colour background's pixel is a colour itself in direct colour mode, with the palette bits (anomie).
        if depth == 8 && self.regs[0x30] & 1 != 0 {
            return high | DIRECT | (palette << 10) | colour;
        }
        high | match depth {
            2 if mode == 0 => bg as u16 * 0x20 + palette * 4 + colour,
            2 => palette * 4 + colour,
            4 => palette * 16 + colour,
            _ => colour,
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
    fn front(&self, enabled: u8, masked: u8, windows: u8, x: u16, sub: bool) -> Option<(u16, usize, u16)> {
        let order: &[(usize, u16)] = match self.regs[0x05] & 7 {
            0 => &MODE0,
            1 if self.regs[0x05] & 0x08 != 0 => &MODE1_BG3_HIGH,
            1 => &MODE1,
            2..=5 => &MODE2,
            6 => &MODE6,
            _ if self.regs[0x33] & 0x40 != 0 => &MODE7_EXTBG,
            _ => &MODE7,
        };
        let halves = sub && matches!(self.regs[0x05] & 7, 5 | 6);
        for &(layer, priority) in order {
            let bit = 1 << layer;
            if enabled & bit == 0 || (masked & bit != 0 && windows & bit != 0) {
                continue;
            }
            let p = if layer == OBJ {
                self.obj_line[x as usize]
            } else if halves {
                self.bg_sub_line[layer][x as usize]
            } else {
                self.bg_line[layer][x as usize]
            };
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
    #[cfg(test)]
    fn compose(&self, x: u16) -> u16 {
        self.mix(x, self.windows_at(x)).colour
    }

    /// The six window masks at `x` as bits: backgrounds 1 to 4, the sprites, the colour window.
    fn windows_at(&self, x: u16) -> u8 {
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
        windows
    }

    fn fixed_colour(&self) -> u16 {
        (self.fixed[0] as u16) | (self.fixed[1] as u16) << 5 | (self.fixed[2] as u16) << 10
    }

    /// Add or subtract by component, halved if asked, saturated.
    fn blend(a: u16, b: u16, subtract: bool, half: bool) -> u16 {
        let mut out = 0;
        for shift in [0, 5, 10] {
            let (m, s) = ((a >> shift) & 31, (b >> shift) & 31);
            let mut c = if subtract { m.saturating_sub(s) } else { m + s };
            if half {
                c >>= 1;
            }
            out |= c.min(31) << shift;
        }
        out
    }

    /// The first X after `x` at which a window's inside begins or ends, 256 if none does.
    fn next_window_edge(&self, x: u16) -> u16 {
        [self.regs[0x26] as u16, self.regs[0x27] as u16 + 1, self.regs[0x28] as u16, self.regs[0x29] as u16 + 1]
            .into_iter()
            .filter(|&e| e > x)
            .min()
            .unwrap_or(WIDTH as u16)
    }

    fn mix(&self, x: u16, windows: u8) -> Mix {
        let (main, layer, index) = self.front(self.regs[0x2C], self.regs[0x2E], windows, x, false).unwrap_or((self.cgram[0], 5, 0));
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
            return Mix { colour: main, before: main, clip, ..Mix::default() };
        }
        let (sub, math, sub_backdrop) = if cgwsel & 2 != 0 {
            match self.front(self.regs[0x2D], self.regs[0x2F], windows, x, true) {
                Some((c, _, _)) => (c, 2, false),
                None => (self.fixed_colour(), 1, true),
            }
        } else {
            (self.fixed_colour(), 1, false)
        };
        let subtract = cgadsub & 0x80 != 0;
        let half = cgadsub & 0x40 != 0 && !clip && !sub_backdrop;
        Mix { colour: Self::blend(main, sub, subtract, half), math, before: main, clip, subtract, half }
    }

    /// The sub screen's half-pixel at `x` on a hi-res line: its front-most pixel over colour 0 (fullsnes, "Hires
    /// Notes"), clipped and mathed as the main pixel before it was (anomie; D-19).
    fn sub_half_pixel(&self, x: u16, before: Mix, windows: u8) -> u16 {
        let raw = self.front(self.regs[0x2D], self.regs[0x2F], windows, x, true).map_or(self.cgram[0], |p| p.0);
        let raw = if before.clip { 0 } else { raw };
        match before.math {
            0 => raw,
            1 => Self::blend(raw, self.fixed_colour(), before.subtract, before.half),
            _ => Self::blend(raw, before.before, before.subtract, before.half),
        }
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
        let at = |p: &Ppu, x: usize| [p.canvas[x * 8], p.canvas[x * 8 + 1], p.canvas[x * 8 + 2]];
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

    // anomie's "Mode 5": a tile is 16 half-pixels wide, the even ones the sub screen's and the odd ones the main's.
    #[test]
    fn a_hi_res_tile_gives_its_even_half_pixels_to_the_sub_screen() {
        let mut p = Ppu::default();
        p.regs[0x05] = 5;
        p.regs[0x0B] = 0x01;
        // BG1's map entry 0 is tile 0; tile 0's row lights half-pixels 0 and 3, tile 1's row half-pixel 8 (its bit 7).
        p.vram[0x1000] = 0b1001_0000;
        p.vram[0x1010] = 0b1000_0000;
        let (mut main, mut sub) = ([0u16; 8], [0u16; 8]);
        p.decode_chunk_hires(0, 5, 0, 0, &mut main, &mut sub);
        assert_eq!((sub.map(|v| v & 0xFF), main.map(|v| v & 0xFF)), ([1, 0, 0, 0, 1, 0, 0, 0], [0, 1, 0, 0, 0, 0, 0, 0]));
        // Flipped, half-pixel h shows column 15-h: columns 0, 3 and 8 land on half-pixels 15, 12 and 7.
        p.vram[0] = 0x4000;
        p.decode_chunk_hires(0, 5, 0, 0, &mut main, &mut sub);
        assert_eq!((sub.map(|v| v & 0xFF), main.map(|v| v & 0xFF)), ([0, 0, 0, 0, 0, 0, 1, 0], [0, 0, 0, 1, 0, 0, 0, 1]));
    }

    // fullsnes: both screens show colour 0 behind them in hi-res; anomie: the sub half-pixel is mathed as the main
    // pixel before it was (D-19).
    #[test]
    fn the_sub_half_pixel_takes_colour_0_and_the_math_of_the_main_pixel_before() {
        let mut p = Ppu::default();
        p.regs[0x05] = 1;
        p.regs[0x33] = 0x08;
        p.regs[0x2C] = 0x01;
        p.regs[0x2D] = 0x02;
        p.cgram[0] = 0x0421;
        p.cgram[1] = 10;
        p.cgram[2] = 6 << 5;
        p.bg_line[0] = [0x8001; WIDTH];
        p.bg_line[1][1] = 0x8002;
        assert!(p.hires());
        let none = Mix::default();
        assert_eq!((p.sub_half_pixel(0, none, 0), p.sub_half_pixel(1, none, 0)), (0x0421, 6 << 5));
        // Main plus sub, halved: the main pixel at 0 adds the fixed colour (the sub screen is clear there), at 1 BG2.
        p.regs[0x30] = 0x02;
        p.regs[0x31] = 0x41;
        p.fixed = [4, 0, 0];
        let (first, second) = (p.mix(0, 0), p.mix(1, 0));
        assert_eq!((first.math, first.colour, second.math, second.colour), (1, 14, 2, 5 | 3 << 5));
        assert_eq!(p.sub_half_pixel(1, first, 0), 4 | 6 << 5);
        // Colour 0 (1, 1, 1) plus the main pixel before math (10, 0, 0), halved.
        assert_eq!(p.sub_half_pixel(2, second, 0), 5);
    }

    // fullsnes's and anomie's mode 7: the identity matrix shows the field as it is, M7SEL flips the screen and
    // chooses the outside, and $2134 is M7A times M7B's high byte through the shared write-twice byte.
    #[test]
    fn mode7_maps_the_screen_through_its_matrix_and_multiplies_on_2134() {
        let mut p = Ppu::default();
        for (r, v) in [(0x1Bu8, 0x00u8), (0x1B, 0x01), (0x1E, 0x00), (0x1E, 0x01)] {
            p.write(r, v, blank());
        }
        assert_eq!((p.m7[0], p.m7[3]), (0x0100, 0x0100));
        // Map entry (1, 0) is tile 2, whose pixel (1, 1) is colour $55; tile 0's pixel (1, 1) is $77.
        p.vram[1] = 2;
        p.vram[(2 << 6) | (1 << 3) | 1] = 0x5500;
        p.vram[(1 << 3) | 1] = 0x7700;
        assert_eq!(p.mode7_pixel(9, 1), 0x55);
        p.regs[0x1A] = 0x01;
        assert_eq!(p.mode7_pixel(255 - 9, 1), 0x55);
        // Scrolled 1016, pixel 9 is 1025, past the field's right edge: wrapped to 1 (tile 0), transparent, or tile 0.
        // The scroll less the centre keeps ten bits and a sign, so 1024 would be 0 (anomie's CLIP).
        p.regs[0x1A] = 0x00;
        for (r, v) in [(0x0Du8, 0xF8u8), (0x0D, 0x03)] {
            p.write(r, v, blank());
        }
        assert_eq!(p.m7[6], 0x03F8);
        assert_eq!(p.mode7_pixel(9, 1), 0x77);
        p.regs[0x1A] = 0x80;
        assert_eq!(p.mode7_pixel(9, 1), 0);
        p.regs[0x1A] = 0xC0;
        assert_eq!(p.mode7_pixel(9, 1), 0x77);
        // -2 ($FFFE) times $FF (-1) is 2.
        for (r, v) in [(0x1Bu8, 0xFEu8), (0x1B, 0xFF), (0x1C, 0x00), (0x1C, 0xFF)] {
            p.write(r, v, blank());
        }
        let product: Vec<u8> = (0x34..=0x36).map(|r| p.read(r, blank(), 0, true).unwrap()).collect();
        assert_eq!(product, [2, 0, 0]);
    }

    // Decided 2026-10-02: the masks are found once per run between window edges; they must equal the per-pixel ones.
    #[test]
    fn window_masks_found_per_run_equal_those_found_per_pixel() {
        let mut p = Ppu::default();
        let mut seed = 0x1234_5678u32;
        for _ in 0..200 {
            for r in [0x23usize, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2A, 0x2B] {
                seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345);
                p.regs[r] = (seed >> 16) as u8;
            }
            let (mut windows, mut until) = (0, 0);
            for x in 0..WIDTH as u16 {
                if x == until {
                    windows = p.windows_at(x);
                    until = p.next_window_edge(x);
                }
                assert_eq!(windows, p.windows_at(x), "x {x} regs {:02X?}", &p.regs[0x23..0x2C]);
            }
        }
    }
}
