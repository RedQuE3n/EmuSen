//! The interrupt controller (psx-spx "Interrupts"): I_STAT, set on a source's rising edge and acknowledged by writing
//! 0, and I_MASK; their meeting drives the R3000A's IP2.

pub const VBLANK: u32 = 1 << 0;
pub const GPU: u32 = 1 << 1;
pub const CDROM: u32 = 1 << 2;
pub const DMA: u32 = 1 << 3;
pub const TIMER0: u32 = 1 << 4;
pub const SIO0: u32 = 1 << 7;
pub const SIO1: u32 = 1 << 8;
pub const SPU: u32 = 1 << 9;
pub const LIGHTPEN: u32 = 1 << 10;

/// The eleven sources; bits 11-15 read zero.
const SOURCES: u32 = 0x7FF;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Irq {
    pub stat: u32,
    pub mask: u32,
}

impl Irq {
    /// A source's rising edge.
    pub fn raise(&mut self, bits: u32) {
        self.stat |= bits & SOURCES;
    }

    /// Writing I_STAT: a 0 acknowledges, a 1 leaves the bit.
    pub fn acknowledge(&mut self, value: u32) {
        self.stat &= value;
    }

    pub fn set_mask(&mut self, value: u32) {
        self.mask = value & SOURCES;
    }

    pub fn line(&self) -> bool {
        self.stat & self.mask != 0
    }
}
