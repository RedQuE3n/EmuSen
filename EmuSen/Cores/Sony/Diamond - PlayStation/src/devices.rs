//! The devices of later stages, as register stubs that answer what a stage 3 program reads (Diamond_Native.md
//! §11.8): the SPU's registers with its transfer to sound RAM, the MDEC's status, the serial port SIO1, and the DEV8 debug port's DUART, whose channel A carries the TTY. SIO0 is `sio0.rs`.

/// Sound RAM, held by its stub as the space the host reads.
pub const SPU_RAM_BYTES: usize = 0x8_0000;

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
    fn the_resets_are_the_documented_values() {
        assert_eq!(MdecStub::default().status, 0x8004_0000);
    }
}
