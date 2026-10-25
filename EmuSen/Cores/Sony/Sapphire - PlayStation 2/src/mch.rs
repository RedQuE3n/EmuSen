//! The memory controller's serial interface to the RDRAMs (Sapphire_Native.md §35.3; Sapphire_Disputes.md D-342): the
//! BIOS sends a serial command through MCH_RICM (1000F430h; the serial address in bits 16-27, the serial operation in
//! bits 6-9, the device in bits 0-4, bit 31 the start) and reads the answer at MCH_DRD (1000F440h), which it also
//! writes with the data of a serial write [R ps2tek "EE RDRAM initialization", the registers' names and fields]. The
//! answers are those measured on PCSX2 by `programs/s24/mch.s` [M]: the two RDRAMs of the console answer the
//! device-identity read of register 21h in turn, then none until a serial write to it with MCH_DRD's bit 7 clear starts
//! the count again; registers 23h and 24h read 0D0Dh and 0090h; register 40h reads back the device field; every other
//! read and every non-read operation answers zero; a command ends at once.

pub const RICM: u32 = 0x1000_F430;
pub const DRD: u32 = 0x1000_F440;

/// The RDRAMs on the serial chain: two of 128 Mbit for 32 MiB [A, ps2tek "Memory Map"; M PCSX2].
pub const DEVICES: u8 = 2;
/// The device-identity answer, and the two configuration registers' values [M PCSX2, D-342].
pub const IDENTITY: u32 = 0x1F;
pub const CONFIG_A: u32 = 0x0D0D;
pub const CONFIG_B: u32 = 0x0090;

const START: u32 = 1 << 31;
const READ: u32 = 0;
const WRITE: u32 = 1;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Mch {
    ricm: u32,
    drd: u32,
    /// The devices that have answered the identity read since the count began.
    answered: u8,
}

impl Mch {
    fn address(&self) -> u32 {
        self.ricm >> 16 & 0xFFF
    }

    fn operation(&self) -> u32 {
        self.ricm >> 6 & 0xF
    }

    pub fn read(&mut self, paddr: u32) -> u32 {
        if paddr != DRD {
            return 0;
        }
        if self.operation() != READ {
            return 0;
        }
        match self.address() {
            0x21 if self.answered < DEVICES => {
                self.answered += 1;
                IDENTITY
            }
            0x23 => CONFIG_A,
            0x24 => CONFIG_B,
            0x40 => self.ricm & 0x1F,
            _ => 0,
        }
    }

    pub fn write(&mut self, paddr: u32, v: u32) {
        match paddr {
            RICM => {
                if v >> 16 & 0xFFF == 0x21 && v >> 6 & 0xF == WRITE && self.drd & 0x80 == 0 {
                    self.answered = 0;
                }
                self.ricm = v & !START;
            }
            DRD => self.drd = v,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `programs/s24/mch.s`'s table and the DRD answers PCSX2 gave it (measured 2026-10-25).
    #[test]
    fn the_serial_answers_are_the_referees() {
        let table: [(u32, u32, u32); 22] = [
            (0, 0x8021_0000, 0x1F),
            (0, 0x8021_0000, 0x1F),
            (0, 0x8021_0000, 0),
            (0, 0x8021_0040, 0),
            (0, 0x8021_0000, 0x1F),
            (0, 0x8021_0000, 0x1F),
            (0, 0x8021_0000, 0),
            (0x80, 0x8021_0040, 0),
            (0, 0x8021_0000, 0),
            (0, 0x8021_0000, 0),
            (0, 0x8023_0000, 0x0D0D),
            (0, 0x8024_0000, 0x0090),
            (0, 0x8040_0000, 0),
            (0, 0x8040_0015, 0x15),
            (0, 0x8040_001F, 0x1F),
            (0, 0x8040_0080, 0),
            (0, 0x8022_0000, 0),
            (0, 0x8023_0040, 0),
            (0, 0x8023_0080, 0),
            (0xABCD, 0x8000_0000, 0),
            (0, 0x8021_0000, 0),
            (0, 0x0021_0000, 0),
        ];
        let mut m = Mch::default();
        for (n, &(drd, ricm, want)) in table.iter().enumerate() {
            m.write(DRD, drd);
            m.write(RICM, ricm);
            assert_eq!((m.read(RICM), m.read(DRD)), (0, want), "command {n}");
        }
    }
}
