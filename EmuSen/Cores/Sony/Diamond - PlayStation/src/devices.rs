//! The devices of later stages, as register stubs that answer what a stage 3 program reads (Diamond_Native.md
//! §11.8): the GPU's GPUSTAT with its ready bits and its command lengths, the SPU's registers with its transfer to
//! sound RAM, the CD-ROM controller's index and status, the MDEC's status, the two serial ports, and the DEV8 debug
//! port's DUART, whose channel A carries the TTY.

use crate::video::Video;

/// VRAM and sound RAM, held by their stubs as the spaces the host reads.
pub const VRAM_BYTES: usize = 0x10_0000;
pub const SPU_RAM_BYTES: usize = 0x8_0000;

/// GPUSTAT after GP1(00h) (psx-spx "GP1(00h) - Reset GPU").
pub const GPUSTAT_RESET: u32 = 0x1480_2000;

#[derive(Clone)]
pub struct GpuStub {
    pub vram: Vec<u8>,
    /// GPUSTAT's stored bits: the attributes, the display mode, display enable, the IRQ and the DMA direction.
    pub stat: u32,
    /// The latched GPUREAD value, the command being received, and the words it still takes.
    pub read: u32,
    pub command: u32,
    pub remaining: u32,
    /// A polyline's words until its terminator, and a CPU-to-VRAM transfer's data words.
    pub polyline: bool,
    pub data_words: u32,
    /// GP0(E2h)..GP0(E5h), which GP1(10h) reads back.
    pub attributes: [u32; 4],
    pub display_start: u32,
    pub version: u32,
}

impl GpuStub {
    /// `version` is GP1(10h)'s answer at index 7: 2 for the 208-pin chip, 0 for the 160-pin one, which has no index 7.
    pub fn new(version: u32) -> GpuStub {
        GpuStub { vram: vec![0; VRAM_BYTES], stat: GPUSTAT_RESET, read: 0, command: 0, remaining: 0, polyline: false, data_words: 0, attributes: [0; 4], display_start: 0, version }
    }

    /// GPUSTAT: always ready for a command, a DMA block and never holding VRAM to read; bit 25 by the DMA direction;
    /// bits 31 and 13 from the video timing.
    pub fn status(&self, video: &Video, now: u64) -> u32 {
        let ready = 1 << 26 | 1 << 28;
        let dreq = match self.stat >> 29 & 3 {
            0 => 0,
            1 | 2 => 1 << 25,
            _ => 0,
        };
        self.stat & !(1 << 31 | 1 << 13 | 7 << 25 | 1 << 28) | ready | dreq | video.status_bits(now)
    }

    /// Whether the GPU requests DMA, by GP1(04h)'s direction.
    pub fn dreq(&self) -> bool {
        self.stat >> 29 & 3 != 0
    }

    /// A GP0 word: commands are counted out by their documented lengths and their effects dropped, but for the
    /// attributes GPUSTAT shows and GP0(1Fh)'s interrupt. Returns whether IRQ1 rose.
    pub fn gp0(&mut self, word: u32) -> bool {
        if self.data_words > 0 {
            self.data_words -= 1;
            return false;
        }
        if self.polyline {
            if word & 0xF000_F000 == 0x5000_5000 {
                self.polyline = false;
            }
            return false;
        }
        if self.remaining > 0 {
            self.remaining -= 1;
            if self.remaining == 0 && self.command >> 5 == 5 {
                let (w, h) = (word & 0x3FF, word >> 16 & 0x1FF);
                let (w, h) = (if w == 0 { 0x400 } else { w }, if h == 0 { 0x200 } else { h });
                self.data_words = (w * h).div_ceil(2);
            }
            if self.remaining == 0 && self.command >> 5 == 2 && self.command & 1 << 3 != 0 {
                self.polyline = true;
            }
            return false;
        }
        let c = word >> 24;
        self.command = c;
        let mut irq = false;
        self.remaining = match c >> 5 {
            1 => {
                let n = if c & 0x08 != 0 { 4 } else { 3 };
                let per = 1 + (c >> 2 & 1);
                n * per + if c & 0x10 != 0 { n - 1 } else { 0 }
            }
            2 => {
                if c & 0x08 != 0 {
                    1 + (c >> 4 & 1)
                } else {
                    2 + (c >> 4 & 1)
                }
            }
            3 => 1 + (c >> 2 & 1) + (c & 0x18 == 0) as u32,
            4 => 3,
            5 | 6 => 2,
            _ => match c {
                0x02 => 2,
                0x1F => {
                    irq = self.stat & 1 << 24 == 0;
                    self.stat |= 1 << 24;
                    0
                }
                0xE1 => {
                    self.stat = self.stat & !0x87FF | word & 0x7FF | (word >> 11 & 1) << 15;
                    0
                }
                0xE2..=0xE5 => {
                    self.attributes[(c - 0xE2) as usize] = word & 0x00FF_FFFF;
                    0
                }
                0xE6 => {
                    self.stat = self.stat & !(3 << 11) | (word & 3) << 11;
                    0
                }
                _ => 0,
            },
        };
        irq
    }

    /// A GP1 command, and the video timing it changes, if any.
    pub fn gp1(&mut self, word: u32, video: &mut Video, now: u64) {
        let p = word & 0x00FF_FFFF;
        match word >> 24 & 0x3F {
            0x00 => {
                self.stat = GPUSTAT_RESET;
                self.remaining = 0;
                self.data_words = 0;
                self.polyline = false;
                self.attributes = [0; 4];
                self.display_start = 0;
                video.retime(now, |v| {
                    v.mode = 0;
                    v.x1 = 0x200;
                    v.x2 = 0x200 + 256 * 10;
                    v.y1 = 0x10;
                    v.y2 = 0x10 + 240;
                });
            }
            0x01 => {
                self.remaining = 0;
                self.data_words = 0;
                self.polyline = false;
            }
            0x02 => self.stat &= !(1 << 24),
            0x03 => self.stat = self.stat & !(1 << 23) | (p & 1) << 23,
            0x04 => self.stat = self.stat & !(3 << 29) | (p & 3) << 29,
            0x05 => self.display_start = p & 0x7FFFF,
            0x06 => video.retime(now, |v| {
                v.x1 = p & 0xFFF;
                v.x2 = p >> 12 & 0xFFF;
            }),
            0x07 => video.retime(now, |v| {
                v.y1 = p & 0x3FF;
                v.y2 = p >> 10 & 0x3FF;
            }),
            0x08 => {
                self.stat = self.stat & !(0x7F << 16 | 1 << 14) | (p & 0x3F) << 17 | (p >> 6 & 1) << 16 | (p >> 7 & 1) << 14;
                video.retime(now, |v| v.mode = p & 0xFF);
            }
            0x10..=0x1F => match p & 0xF {
                2..=5 => self.read = self.attributes[(p & 0xF) as usize - 2],
                7 if self.version != 0 => self.read = self.version,
                8 if self.version != 0 => self.read = 0,
                _ => {}
            },
            _ => {}
        }
    }
}

/// The SPU's registers at 1F801C00h..1F801FFFh as halfwords, and sound RAM, written through the transfer address.
#[derive(Clone)]
pub struct SpuStub {
    pub regs: [u16; 512],
    pub ram: Vec<u8>,
    /// The transfer's current address in sound RAM, in bytes.
    pub cursor: u32,
}

const ATTR: usize = 0x1AA / 2;
const STATX: usize = 0x1AE / 2;
const TSA: usize = 0x1A6 / 2;
const FIFO: usize = 0x1A8 / 2;

impl Default for SpuStub {
    fn default() -> SpuStub {
        SpuStub::new()
    }
}

impl SpuStub {
    pub fn new() -> SpuStub {
        SpuStub { regs: [0; 512], ram: vec![0; SPU_RAM_BYTES], cursor: 0 }
    }

    pub fn read16(&self, offset: u32) -> u16 {
        let i = (offset as usize >> 1) & 511;
        match i {
            STATX => self.regs[ATTR] & 0x3F | (self.regs[ATTR] & 0x20) << 2,
            _ if offset >= 0x280 => 0xFFFF,
            _ => self.regs[i],
        }
    }

    pub fn write16(&mut self, offset: u32, value: u16) {
        let i = (offset as usize >> 1) & 511;
        match i {
            STATX => {}
            TSA => {
                self.regs[i] = value;
                self.cursor = value as u32 * 8;
            }
            FIFO => self.store(value),
            _ => self.regs[i] = value,
        }
    }

    /// A halfword into sound RAM at the transfer address, which then advances.
    pub fn store(&mut self, value: u16) {
        let a = (self.cursor & 0x7FFFE) as usize;
        self.ram[a..a + 2].copy_from_slice(&value.to_le_bytes());
        self.cursor = (self.cursor + 2) & 0x7FFFF;
    }

    pub fn load(&mut self) -> u16 {
        let a = (self.cursor & 0x7FFFE) as usize;
        self.cursor = (self.cursor + 2) & 0x7FFFF;
        u16::from_le_bytes([self.ram[a], self.ram[a + 1]])
    }

    /// Whether the SPU requests DMA: ATTR's transfer mode is a DMA write or read.
    pub fn dreq(&self) -> bool {
        self.regs[ATTR] >> 4 & 3 >= 2
    }
}

/// The CD-ROM controller's register bank (psx-spx "CDROM Controller I/O Ports"): its index, empty FIFOs and the
/// interrupt mask and flags; commands get no response before stage 7.
#[derive(Clone, Copy, Debug, Default)]
pub struct CdromStub {
    pub index: u8,
    pub mask: u8,
    pub flags: u8,
}

impl CdromStub {
    pub fn read8(&self, offset: u32) -> u8 {
        match (offset & 3, self.index) {
            (0, _) => 0x18 | self.index,
            (3, 0 | 2) => 0xE0 | self.mask,
            (3, _) => 0xE0 | self.flags,
            _ => 0,
        }
    }

    pub fn write8(&mut self, offset: u32, value: u8) {
        match (offset & 3, self.index) {
            (0, _) => self.index = value & 3,
            (2, 1) => self.mask = value & 0x1F,
            (3, 1) => self.flags &= !(value & 0x1F),
            _ => {}
        }
    }
}

/// The MDEC's status after a reset (psx-spx "MDEC Status Register"), and its two requests.
#[derive(Clone, Copy, Debug)]
pub struct MdecStub {
    pub status: u32,
}

impl Default for MdecStub {
    fn default() -> MdecStub {
        MdecStub { status: 0x8004_0000 }
    }
}

impl MdecStub {
    pub fn control(&mut self, value: u32) {
        if value & 1 << 31 != 0 {
            self.status = 0x8004_0000;
        }
        self.status = self.status & !(3 << 27) | (value >> 30 & 1) << 28 | (value >> 29 & 1) << 27;
    }
}

/// A serial port with nothing plugged in: a byte sent is answered with FFh and no /ACK.
#[derive(Clone, Copy, Debug, Default)]
pub struct SioStub {
    pub mode: u16,
    pub control: u16,
    pub baud: u16,
    pub rx: Option<u8>,
}

impl SioStub {
    pub fn status(&self) -> u32 {
        1 | 4 | (self.rx.is_some() as u32) << 1
    }

    pub fn send(&mut self, _byte: u8) {
        self.rx = Some(0xFF);
    }

    pub fn receive(&mut self) -> u8 {
        self.rx.take().unwrap_or(0)
    }

    pub fn set_control(&mut self, value: u16) {
        self.control = if value & 1 << 6 != 0 { 0 } else { value & 0x3FAF };
        if value & 1 << 6 != 0 {
            self.mode = 0;
            self.rx = None;
        }
    }
}

/// The TTY's text, kept as the `TTY` space (Diamond_Plan.md §3.3): what the program prints, in order, until full.
pub const TTY_BYTES: usize = 0x1_0000;

#[derive(Clone)]
pub struct Tty {
    pub text: Vec<u8>,
    pub len: usize,
}

impl Default for Tty {
    fn default() -> Tty {
        Tty::new()
    }
}

impl Tty {
    pub fn new() -> Tty {
        Tty { text: vec![0; TTY_BYTES], len: 0 }
    }

    pub fn put(&mut self, byte: u8) {
        if byte != 0 && self.len < TTY_BYTES {
            self.text[self.len] = byte;
            self.len += 1;
        }
    }
}

/// The SCN2681 DUART of the DEV8 debug port at 1F802020h..1F80202Fh (psx-spx "DEV8 Dual Serial Port"), as a debug
/// terminal card would fit it: channel A's transmitter always ready, its bytes the TTY; nothing is ever received.
#[derive(Clone, Copy, Debug, Default)]
pub struct Duart {
    /// Channel A's mode registers, MR1A and MR2A, reached through one address by a pointer.
    pub mr: [u8; 2],
    pub pointer: u8,
}

impl Duart {
    pub fn read8(&mut self, offset: u32) -> u8 {
        match offset & 0xF {
            0 => {
                let v = self.mr[self.pointer as usize];
                self.pointer = 1;
                v
            }
            1 => 0x0C,
            _ => 0,
        }
    }

    /// A write, and the byte it sends to the TTY, if any.
    pub fn write8(&mut self, offset: u32, value: u8) -> Option<u8> {
        match offset & 0xF {
            0 => {
                self.mr[self.pointer as usize] = value;
                self.pointer = 1;
            }
            2 if value >> 4 & 7 == 1 => self.pointer = 0,
            3 => return Some(value),
            _ => {}
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gp0_commands_are_counted_out_by_their_lengths() {
        let mut g = GpuStub::new(2);
        for (words, name) in [(vec![0x2000_0000, 1, 2, 3], "flat triangle"), (vec![0x3C00_0000, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11], "shaded textured quad"), (vec![0x6000_0000, 1, 2], "variable rectangle"), (vec![0x4800_0000, 1, 2, 3, 0x5555_5555], "polyline")] {
            for w in words {
                g.gp0(w);
            }
            g.gp0(0xE100_0605);
            assert_eq!(g.stat & 0x7FF, 0x605, "{name}: the next word is a command");
            g.gp0(0xE100_0000);
        }
        g.gp0(0xA000_0000);
        g.gp0(0);
        g.gp0(0x0002_0003);
        for _ in 0..3 {
            g.gp0(0xE1FF_FFFF);
        }
        assert_eq!(g.stat & 0x7FF, 0, "three data words of a 3x2 transfer");
        assert!(g.gp0(0x1F00_0000));
        assert_ne!(g.stat & 1 << 24, 0);
    }

    #[test]
    fn the_resets_are_the_documented_values() {
        let v = Video::new(false);
        let g = GpuStub::new(2);
        assert_eq!(g.status(&v, 0) & !(1 << 31), 0x1480_2000 | 1 << 26 | 1 << 28);
        assert_eq!(MdecStub::default().status, 0x8004_0000);
        assert_eq!(CdromStub::default().read8(0), 0x18);
    }
}
