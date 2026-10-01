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

/// The layers of modes 0 and 1 from the front, as (background, tile priority) (fullsnes, "Background Priority Chart").
const MODE0: [(usize, u16); 8] = [(0, 1), (1, 1), (0, 0), (1, 0), (2, 1), (3, 1), (2, 0), (3, 0)];
const MODE1: [(usize, u16); 6] = [(0, 1), (1, 1), (0, 0), (1, 0), (2, 1), (2, 0)];
const MODE1_BG3_HIGH: [(usize, u16); 6] = [(2, 1), (0, 1), (1, 1), (0, 0), (1, 0), (2, 0)];

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
        self.forced_blank() || beam.vblank || beam.line == 0
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
            0x3E => (self.ppu1_mdr & 0x10) | 0x01,
            0x3F => {
                let v = (if beam.field { 0x80 } else { 0 }) | (if self.latched { 0x40 } else { 0 }) | (self.ppu2_mdr & 0x20) | (if beam.pal { 0x10 } else { 0 }) | 0x03;
                if side_effects {
                    self.latched = false;
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
        let blank = self.forced_blank() || brightness == 0;
        for x in from..to {
            let colour = if blank { 0 } else { self.pixel(x, line) };
            let scale = |c: u16| -> u8 {
                let c = (c as u32 & 31) * (brightness + 1) / 16;
                ((c << 3) | (c >> 2)) as u8
            };
            let at = row + x as usize * 4;
            self.frame[at] = scale(colour);
            self.frame[at + 1] = scale(colour >> 5);
            self.frame[at + 2] = scale(colour >> 10);
        }
    }

    /// The main screen's colour at a pixel: the frontmost opaque background of modes 0 and 1, or the backdrop.
    fn pixel(&self, x: u16, line: u16) -> u16 {
        let mode = self.regs[0x05] & 7;
        let order: &[(usize, u16)] = match mode {
            0 => &MODE0,
            1 if self.regs[0x05] & 0x08 != 0 => &MODE1_BG3_HIGH,
            1 => &MODE1,
            _ => &[],
        };
        let mut cache: [Option<(u16, u16)>; 4] = [None; 4];
        let mut fetched = [false; 4];
        for &(bg, priority) in order {
            if self.regs[0x2C] & (1 << bg) == 0 {
                continue;
            }
            if !fetched[bg] {
                fetched[bg] = true;
                cache[bg] = self.background(bg, mode, x, line);
            }
            if let Some((index, p)) = cache[bg] {
                if p == priority {
                    return self.cgram[index as usize];
                }
            }
        }
        self.cgram[0]
    }

    /// A background's pixel as (CGRAM index, tile priority), or None where it is transparent.
    fn background(&self, bg: usize, mode: u8, x: u16, line: u16) -> Option<(u16, u16)> {
        let big = self.regs[0x05] & (0x10 << bg) != 0;
        let shift = if big { 4 } else { 3 };
        let px = x.wrapping_add(self.hofs[bg]);
        let py = line.wrapping_add(self.vofs[bg]);
        let (tx, ty) = (px >> shift, py >> shift);
        let sc = self.regs[0x07 + bg];
        let screen = match sc & 3 {
            0 => 0,
            1 => (tx >> 5) & 1,
            2 => (ty >> 5) & 1,
            _ => ((tx >> 5) & 1) + 2 * ((ty >> 5) & 1),
        };
        let base = ((sc as u16 >> 2) << 10).wrapping_add(screen << 10);
        let entry = self.vram[(base.wrapping_add(((ty & 31) << 5) | (tx & 31)) & 0x7FFF) as usize];
        let size = 1u16 << shift;
        let mut fx = px & (size - 1);
        let mut fy = py & (size - 1);
        if entry & 0x4000 != 0 {
            fx = size - 1 - fx;
        }
        if entry & 0x8000 != 0 {
            fy = size - 1 - fy;
        }
        let tile = ((entry & 0x3FF) + (fx >> 3) + ((fy >> 3) << 4)) & 0x3FF;
        let deep = mode == 1 && bg < 2;
        let nba = (self.regs[0x0B + bg / 2] >> (4 * (bg & 1))) as u16 & 0x0F;
        let words = if deep { 16 } else { 8 };
        let at = (nba << 12).wrapping_add(tile * words).wrapping_add(fy & 7);
        let bit = 7 - (fx & 7);
        let plane = |word: u16, high: bool| -> u16 { ((if high { word >> 8 } else { word }) >> bit) & 1 };
        let w0 = self.vram[(at & 0x7FFF) as usize];
        let mut colour = plane(w0, false) | plane(w0, true) << 1;
        if deep {
            let w1 = self.vram[(at.wrapping_add(8) & 0x7FFF) as usize];
            colour |= plane(w1, false) << 2 | plane(w1, true) << 3;
        }
        if colour == 0 {
            return None;
        }
        let palette = (entry >> 10) & 7;
        let index = if deep { palette * 16 + colour } else if mode == 0 { bg as u16 * 0x20 + palette * 4 + colour } else { palette * 4 + colour };
        Some((index, (entry >> 13) & 1))
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
        assert!(!p.latched && !p.oph_second);
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
}
