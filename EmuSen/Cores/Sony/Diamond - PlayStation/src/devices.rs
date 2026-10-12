//! The small devices (Diamond_Native.md §11.8): the serial port SIO1 as a stub that answers what a stage 3 program
//! reads, the DEV8 debug port's DUART, whose channel A carries the TTY, and the TTY itself. SIO0 is `sio0.rs`, the SPU
//! `spu/`, the MDEC `mdec/`.

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
