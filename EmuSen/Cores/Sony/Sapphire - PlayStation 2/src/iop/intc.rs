//! The IOP's interrupt controller (ps2tek "IOP Interrupts"): 26 sources latched into I_STAT, masked by I_MASK, and the
//! global enable I_CTRL, which a read returns and clears; the processor's interrupt line is I_CTRL AND (I_STAT AND
//! I_MASK). Sapphire_Native.md §22.4.

/// The sources by their bit in I_STAT and I_MASK (ps2tek's table).
pub mod source {
    pub const VBLANK_START: u32 = 0;
    pub const GPU: u32 = 1;
    pub const CDVD: u32 = 2;
    pub const DMA: u32 = 3;
    pub const TIMER0: u32 = 4;
    pub const TIMER1: u32 = 5;
    pub const TIMER2: u32 = 6;
    pub const SIO0: u32 = 7;
    pub const SIO1: u32 = 8;
    pub const SPU2: u32 = 9;
    pub const PIO: u32 = 10;
    pub const VBLANK_END: u32 = 11;
    pub const DVD: u32 = 12;
    pub const PCMCIA: u32 = 13;
    pub const TIMER3: u32 = 14;
    pub const TIMER4: u32 = 15;
    pub const TIMER5: u32 = 16;
    pub const SIO2: u32 = 17;
    pub const USB: u32 = 22;
    pub const FWRE: u32 = 24;
    pub const FDMA: u32 = 25;
    /// Each timer's source, by timer.
    pub const TIMERS: [u32; 6] = [TIMER0, TIMER1, TIMER2, TIMER3, TIMER4, TIMER5];
}

pub const I_STAT: u32 = 0x1F80_1070;
pub const I_MASK: u32 = 0x1F80_1074;
pub const I_CTRL: u32 = 0x1F80_1078;
/// The 26 sources' bits; ps2tek calls 26-31 "unused/garbage", read here as zero [C].
pub const BITS: u32 = 0x03FF_FFFF;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Intc {
    pub stat: u32,
    pub mask: u32,
    pub ctrl: u32,
}

impl Intc {
    /// A source's request: its bit set whether or not it is masked (psx-spx "Interrupts", which ps2tek follows).
    pub fn raise(&mut self, source: u32) {
        self.stat |= 1 << source;
    }

    /// The interrupt line, as Cause's IP2 sees it.
    pub fn line(&self) -> bool {
        self.ctrl & 1 != 0 && self.stat & self.mask != 0
    }

    /// A read; I_CTRL's returns bit 0 and clears it, disabling interrupts (ps2tek).
    pub fn read(&mut self, paddr: u32) -> u32 {
        match paddr {
            I_STAT => self.stat,
            I_MASK => self.mask,
            I_CTRL => std::mem::take(&mut self.ctrl),
            _ => 0,
        }
    }

    /// A write: I_STAT's bits written 0 are acknowledged and those written 1 kept; I_MASK takes the value; I_CTRL's
    /// bit 0 is set or reset.
    pub fn write(&mut self, paddr: u32, v: u32) {
        match paddr {
            I_STAT => self.stat &= v,
            I_MASK => self.mask = v & BITS,
            I_CTRL => self.ctrl = v & 1,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_line_needs_ctrl_stat_and_mask_and_a_ctrl_read_clears_it() {
        let mut i = Intc::default();
        i.raise(source::DMA);
        assert!(!i.line(), "masked");
        i.write(I_MASK, u32::MAX);
        assert_eq!(i.mask, BITS);
        assert!(!i.line(), "I_CTRL clear");
        i.write(I_CTRL, 3);
        assert!(i.line());
        assert_eq!(i.read(I_CTRL), 1);
        assert_eq!(i.read(I_CTRL), 0, "the read cleared it");
        assert!(!i.line());
        i.write(I_CTRL, 1);
        i.write(I_STAT, !(1 << source::DMA));
        assert_eq!(i.stat, 0, "a 0 acknowledges");
        assert!(!i.line());
    }
}
