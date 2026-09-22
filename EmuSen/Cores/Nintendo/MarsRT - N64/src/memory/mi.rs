//! The MIPS interface, the C# `MiInterface`.

use crate::state::{State, StateReader, StateResult, StateWriter};

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct MiInterface {
    pub ebus: bool,
    /// `Mask`, a `MiInterrupt` written as its int32.
    pub mask: i32,
    /// `Pending`, a `MiInterrupt` written as its int32.
    pub pending: i32,
    pub repeat_count: u32,
    pub repeating: bool,
    pub upper: bool,
}

impl State for MiInterface {
    fn write_state(&self, w: &mut StateWriter) {
        w.bool("Ebus", self.ebus);
        w.i32("Mask", self.mask);
        w.i32("Pending", self.pending);
        w.u32("RepeatCount", self.repeat_count);
        w.bool("Repeating", self.repeating);
        w.bool("Upper", self.upper);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        self.ebus = r.bool()?; // Ebus
        self.mask = r.i32()?; // Mask
        self.pending = r.i32()?; // Pending
        self.repeat_count = r.u32()?; // RepeatCount
        self.repeating = r.bool()?; // Repeating
        self.upper = r.bool()?; // Upper
        Ok(())
    }
}

/// `MiInterrupt`: which device is asking for attention.
pub mod interrupt {
    pub const SIGNAL_PROCESSOR: i32 = 1 << 0;
    pub const SERIAL_INTERFACE: i32 = 1 << 1;
    pub const AUDIO_INTERFACE: i32 = 1 << 2;
    pub const VIDEO_INTERFACE: i32 = 1 << 3;
    pub const PERIPHERAL_INTERFACE: i32 = 1 << 4;
    pub const DISPLAY_PROCESSOR: i32 = 1 << 5;
}

/// `RcpVersion`.
pub const RCP_VERSION: u32 = 0x0202_0102;

impl MiInterface {
    /// Level-triggered: asserted while a device is raised and unmasked.
    #[inline(always)]
    pub fn asserted(&self) -> bool {
        (self.pending & self.mask) != 0
    }

    #[inline(always)]
    pub fn raise(&mut self, source: i32) {
        self.pending |= source;
    }

    #[inline(always)]
    pub fn clear(&mut self, source: i32) {
        self.pending &= !source;
    }

    pub fn read32(&self, offset: u32) -> u32 {
        match offset & 0x0C {
            0x00 => {
                self.repeat_count
                    | if self.repeating { 0x080 } else { 0 }
                    | if self.ebus { 0x100 } else { 0 }
                    | if self.upper { 0x200 } else { 0 }
            }
            0x04 => RCP_VERSION,
            0x08 => self.pending as u32,
            _ => self.mask as u32,
        }
    }

    pub fn write32(&mut self, offset: u32, value: u32) {
        if offset & 0x0C == 0x00 {
            self.write_mode(value);
            return;
        }
        if offset & 0x0C != 0x0C {
            return;
        }
        for source in 0..6 {
            let flag = 1 << source;
            if value & (1u32 << (source * 2)) != 0 {
                self.mask &= !flag;
            }
            if value & (1u32 << (source * 2 + 1)) != 0 {
                self.mask |= flag;
            }
        }
    }

    /// A set bit wins over its clear when a write carries both.
    fn write_mode(&mut self, value: u32) {
        self.repeat_count = value & 0x7F;
        if value & 0x080 != 0 {
            self.repeating = false;
        }
        if value & 0x100 != 0 {
            self.repeating = true;
        }
        if value & 0x200 != 0 {
            self.ebus = false;
        }
        if value & 0x400 != 0 {
            self.ebus = true;
        }
        if value & 0x1000 != 0 {
            self.upper = false;
        }
        if value & 0x2000 != 0 {
            self.upper = true;
        }
        if value & 0x800 != 0 {
            self.clear(interrupt::DISPLAY_PROCESSOR);
        }
    }
}
