//! The INTC (EE User's Manual chapter 3; ps2tek "EE Interrupt Controller"): fifteen sources latched into I_STAT at
//! their edge, masked by I_MASK, and INT0 to the EE core while any bit of both is set. Sapphire_Native.md §20.4.

/// The sources by their bit in I_STAT and I_MASK (§3.1's table).
pub mod source {
    pub const GS: u32 = 0;
    pub const SBUS: u32 = 1;
    pub const VB_ON: u32 = 2;
    pub const VB_OFF: u32 = 3;
    pub const VIF0: u32 = 4;
    pub const VIF1: u32 = 5;
    pub const VU0: u32 = 6;
    pub const VU1: u32 = 7;
    pub const IPU: u32 = 8;
    pub const TIMER0: u32 = 9;
    pub const SFIFO: u32 = 13;
    pub const VU0WD: u32 = 14;
}

pub const I_STAT: u32 = 0x1000_F000;
pub const I_MASK: u32 = 0x1000_F010;
const BITS: u32 = 0x7FFF;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Intc {
    pub stat: u32,
    pub mask: u32,
}

impl Intc {
    /// A source's effective edge: its bit set, which a further edge cannot change until it is cleared (§3.2).
    pub fn raise(&mut self, source: u32) {
        self.stat |= 1 << source;
    }

    /// INT0: I_STAT AND I_MASK not zero (§3.1).
    pub fn int0(&self) -> bool {
        self.stat & self.mask != 0
    }

    pub fn read(&self, paddr: u32) -> u32 {
        match paddr {
            I_STAT => self.stat,
            I_MASK => self.mask,
            _ => 0,
        }
    }

    /// I_STAT's bits are cleared by writing 1, I_MASK's reversed by writing 1 (§3.2).
    pub fn write(&mut self, paddr: u32, v: u32) {
        match paddr {
            I_STAT => self.stat &= !(v & BITS),
            I_MASK => self.mask ^= v & BITS,
            _ => {}
        }
    }
}
