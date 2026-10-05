//! The VDP as stage 3 needs it: the ports and registers, VRAM, CRAM and VSRAM written at once, DMA done at once,
//! the status register, the HV counter by line and an approximate position, and the line-timed interrupts. The
//! FIFO, the slot schedule, DMA's timing and the picture are stage 4's. Charles MacDonald's "Sega Genesis VDP
//! documentation" §4-§7 is the source; Nephrite_Native.md §9 records what is argued.

/// Master clocks a line.
pub const LINE: u64 = 3420;
/// Where in line 224 the vertical interrupt is raised: "roughly at H counter cycle 08h", taken as 16 pixels in.
pub const VINT_OFFSET: u64 = 128;

pub struct Vdp {
    pub regs: [u8; 24],
    pending: bool,
    code: u8,
    address: u16,
    pub vram: Vec<u8>,
    pub cram: Vec<u8>,
    pub vsram: Vec<u8>,
    /// A fill waiting for its data port write.
    fill: bool,
    pub vint_pending: bool,
    pub hint_pending: bool,
    hint_counter: u8,
    pub pal: bool,
    /// The external interrupt, and the HV counter latched by it while register 0's bit 1 is set.
    pub ext_pending: bool,
    hv_latch: Option<u16>,
}

/// A transfer from the 68000's bus the control port has started: the source, the words, and the destination's code.
pub struct Transfer {
    pub source: u32,
    pub words: u32,
}

/// The VDP's registers and latches, for the state; the memories are spaces of their own.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VdpRegs {
    pub regs: [u8; 24],
    pub pending: bool,
    pub code: u8,
    pub address: u16,
    pub fill: bool,
    pub vint_pending: bool,
    pub hint_pending: bool,
    pub hint_counter: u8,
    pub ext_pending: bool,
    pub hv_latch: Option<u16>,
}

impl Vdp {
    pub fn regs_state(&self) -> VdpRegs {
        VdpRegs {
            regs: self.regs,
            pending: self.pending,
            code: self.code,
            address: self.address,
            fill: self.fill,
            vint_pending: self.vint_pending,
            hint_pending: self.hint_pending,
            hint_counter: self.hint_counter,
            ext_pending: self.ext_pending,
            hv_latch: self.hv_latch,
        }
    }

    pub fn set_regs_state(&mut self, s: VdpRegs) {
        self.regs = s.regs;
        self.pending = s.pending;
        self.code = s.code;
        self.address = s.address;
        self.fill = s.fill;
        self.vint_pending = s.vint_pending;
        self.hint_pending = s.hint_pending;
        self.hint_counter = s.hint_counter;
        self.ext_pending = s.ext_pending;
        self.hv_latch = s.hv_latch;
    }

    pub fn new(pal: bool) -> Vdp {
        Vdp {
            regs: [0; 24],
            pending: false,
            code: 0,
            address: 0,
            vram: vec![0; 0x1_0000],
            cram: vec![0; 128],
            vsram: vec![0; 80],
            fill: false,
            vint_pending: false,
            hint_pending: false,
            hint_counter: 0,
            pal,
            ext_pending: false,
            hv_latch: None,
        }
    }

    fn increment(&self) -> u16 {
        self.regs[15] as u16
    }

    /// Lines in a frame, and the line of the vertical interrupt (240 in PAL's V30).
    pub fn lines(&self) -> u32 {
        if self.pal { 313 } else { 262 }
    }
    pub fn vint_line(&self) -> u32 {
        if self.pal && self.regs[1] & 8 != 0 { 240 } else { 224 }
    }

    /// The control port. A register write, or a command's first or second word; a second word with CD5 set and DMA
    /// enabled starts a DMA, which a transfer from the bus returns for the bus to perform.
    pub fn control(&mut self, v: u16) -> Option<Transfer> {
        if self.pending {
            self.pending = false;
            self.address = (self.address & 0x3FFF) | (v & 3) << 14;
            self.code = (self.code & 3) | ((v >> 2) & 0x3C) as u8;
            if self.code & 0x20 != 0 && self.regs[1] & 0x10 != 0 {
                return self.dma();
            }
            return None;
        }
        if v & 0xC000 == 0x8000 {
            let r = ((v >> 8) & 0x1F) as usize;
            if r < 24 {
                self.regs[r] = v as u8;
            }
            return None;
        }
        self.code = (self.code & 0x3C) | (v >> 14) as u8;
        self.address = (self.address & 0xC000) | (v & 0x3FFF);
        self.pending = true;
        None
    }

    fn dma_length(&self) -> u32 {
        let n = (self.regs[20] as u32) << 8 | self.regs[19] as u32;
        if n == 0 { 0x1_0000 } else { n }
    }

    fn dma(&mut self) -> Option<Transfer> {
        match self.regs[23] >> 6 {
            0 | 1 => {
                let source = ((self.regs[23] & 0x7F) as u32) << 17 | (self.regs[22] as u32) << 9 | (self.regs[21] as u32) << 1;
                let words = self.dma_length();
                self.regs[19] = 0;
                self.regs[20] = 0;
                let end = (source >> 1).wrapping_add(words);
                self.regs[21] = end as u8;
                self.regs[22] = (end >> 8) as u8;
                Some(Transfer { source, words })
            }
            2 => {
                self.fill = true;
                None
            }
            _ => {
                let mut src = (self.regs[22] as u16) << 8 | self.regs[21] as u16;
                for _ in 0..self.dma_length() {
                    let b = self.vram[src as usize];
                    self.vram[self.address as usize] = b;
                    src = src.wrapping_add(1);
                    self.address = self.address.wrapping_add(self.increment());
                }
                self.regs[21] = src as u8;
                self.regs[22] = (src >> 8) as u8;
                self.regs[19] = 0;
                self.regs[20] = 0;
                None
            }
        }
    }

    /// A word through the data port to the memory the code names; VRAM's bytes are swapped at an odd address.
    pub fn data(&mut self, v: u16) {
        self.pending = false;
        if self.fill {
            self.fill = false;
            self.store(v);
            let hi = (v >> 8) as u8;
            for _ in 0..self.dma_length() {
                if self.code & 0xF == 1 {
                    self.vram[self.address as usize ^ 1] = hi;
                }
                self.address = self.address.wrapping_add(self.increment());
            }
            self.regs[19] = 0;
            self.regs[20] = 0;
            return;
        }
        self.store(v);
        self.address = self.address.wrapping_add(self.increment());
    }

    fn store(&mut self, v: u16) {
        let a = self.address as usize;
        match self.code & 0xF {
            1 => {
                let (hi, lo) = ((v >> 8) as u8, v as u8);
                let (x, y) = if a & 1 == 0 { (hi, lo) } else { (lo, hi) };
                self.vram[a & 0xFFFE] = x;
                self.vram[(a & 0xFFFE) | 1] = y;
            }
            3 => {
                self.cram[a & 0x7E] = (v >> 8) as u8;
                self.cram[(a & 0x7E) | 1] = v as u8;
            }
            5 if a & 0x7E < 80 => {
                self.vsram[a & 0x7E] = (v >> 8) as u8;
                self.vsram[(a & 0x7E) | 1] = v as u8;
            }
            _ => {}
        }
    }

    /// A word read through the data port: VRAM, CRAM or VSRAM by the code.
    pub fn read_data(&mut self) -> u16 {
        self.pending = false;
        let a = self.address as usize;
        let v = match self.code & 0xF {
            0 => (self.vram[a & 0xFFFE] as u16) << 8 | self.vram[(a & 0xFFFE) | 1] as u16,
            8 => (self.cram[a & 0x7E] as u16) << 8 | self.cram[(a & 0x7E) | 1] as u16,
            4 if a & 0x7E < 80 => (self.vsram[a & 0x7E] as u16) << 8 | self.vsram[(a & 0x7E) | 1] as u16,
            _ => 0,
        };
        self.address = self.address.wrapping_add(self.increment());
        v
    }

    /// The status register at `line` and `dot` master clocks into it, the FIFO always empty; reading it ends a command
    /// half written.
    pub fn status(&mut self, line: u32, dot: u64) -> u16 {
        self.pending = false;
        let vblank = line >= self.vint_line() || self.regs[1] & 0x40 == 0;
        let hblank = !(64..LINE - 420).contains(&dot);
        0x3400 | 0x0200 | (self.vint_pending as u16) << 7 | (vblank as u16) << 3 | (hblank as u16) << 2 | self.pal as u16
    }

    /// The V counter from the line, with NTSC's jump from $EA to $E5 and PAL's from $F2 to $D2; the H counter from the
    /// position as a fraction of the line (argued, stage 4's to make exact).
    pub fn hv(&self, line: u32, dot: u64) -> u16 {
        if let Some(l) = self.hv_latch.filter(|_| self.regs[0] & 2 != 0) {
            return l;
        }
        let v = if self.pal { if line > 0xF2 { line - 0x39 } else { line } } else if line > 0xEA { line - 6 } else { line };
        let h = (dot * 0x100 / LINE) as u16;
        ((v & 0xFF) as u16) << 8 | h
    }

    /// TH changed on a port whose interrupt is enabled: the external interrupt, and the HV counter latched.
    pub fn external(&mut self, line: u32, dot: u64) {
        self.ext_pending = true;
        if self.regs[0] & 2 != 0 {
            self.hv_latch = None;
            self.hv_latch = Some(self.hv(line, dot));
        }
    }

    /// The line counter at the start of `line`: decremented through the active lines and line 224, an interrupt where
    /// it expires, reloaded from register 10 then and on every other line.
    pub fn line_start(&mut self, line: u32) {
        if line <= self.vint_line() {
            if self.hint_counter == 0 {
                self.hint_counter = self.regs[10];
                self.hint_pending = true;
            } else {
                self.hint_counter -= 1;
            }
        } else {
            self.hint_counter = self.regs[10];
        }
    }

    /// The level the 68000 is asked for: 6 for a pending vertical interrupt it enabled, else 4 for a line interrupt.
    pub fn level(&self) -> u8 {
        if self.vint_pending && self.regs[1] & 0x20 != 0 {
            6
        } else if self.hint_pending && self.regs[0] & 0x10 != 0 {
            4
        } else if self.ext_pending && self.regs[11] & 8 != 0 {
            2
        } else {
            0
        }
    }

    pub fn acknowledge(&mut self, level: u8) {
        match level {
            6 => self.vint_pending = false,
            4 => self.hint_pending = false,
            2 => self.ext_pending = false,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_and_a_vram_write_through_the_ports() {
        let mut v = Vdp::new(false);
        v.control(0x8F02);
        assert_eq!(v.regs[15], 2);
        v.control(0x4000);
        v.control(0x0000);
        v.data(0x1234);
        v.data(0x5678);
        assert_eq!(&v.vram[..4], &[0x12, 0x34, 0x56, 0x78]);
        v.control(0xC000);
        v.control(0x0000);
        v.data(0x0EEE);
        assert_eq!(&v.cram[..2], &[0x0E, 0xEE]);
    }

    #[test]
    fn a_fill_writes_its_byte_through_vram() {
        let mut v = Vdp::new(false);
        v.control(0x8114);
        v.control(0x8F01);
        v.control(0x9310);
        v.control(0x9400);
        v.control(0x9780);
        v.control(0x4000);
        v.control(0x0080);
        v.data(0xAB00);
        assert!(v.vram[..16].iter().all(|&b| b == 0xAB) && v.vram[16] == 0);
    }

    #[test]
    fn the_line_counter_interrupts_every_n_plus_one_lines() {
        let mut v = Vdp::new(false);
        v.regs[10] = 3;
        let mut fired = Vec::new();
        for line in 0..262 {
            v.line_start(line);
            if std::mem::take(&mut v.hint_pending) {
                fired.push(line);
            }
        }
        let mut again = Vec::new();
        for line in 0..262 {
            v.line_start(line);
            if std::mem::take(&mut v.hint_pending) {
                again.push(line);
            }
        }
        assert_eq!(&again[..3], &[3, 7, 11]);
        assert!(again.iter().all(|&l| l <= 224));
    }
}
