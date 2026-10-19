//! The chips' spaces beside the eight every cartridge has, and the reads and writes v1's `space_read` and `space_write`
//! hand to them. See VenusRT_Native.md §2 and EmuSen_Backend.md §19.9.

use emusen_native::core::status;

use crate::machine::Machine;

/// The chips' spaces, present with their chip: the GSU's RAM and its own bus, the SA-1's I-RAM, BW-RAM and its own
/// bus, a NEC DSP's data RAM (two bytes a word) and its program (three bytes an opcode, low byte first).
pub const GSURAM: u32 = 8;
pub const GSUBUS: u32 = 9;
pub const SA1IRAM: u32 = 10;
pub const BWRAM: u32 = 11;
pub const SA1BUS: u32 = 12;
pub const DSPRAM: u32 = 13;
pub const DSPPRG: u32 = 14;

impl Machine {
    pub fn chip_space_size(&self, space: u32) -> Option<i64> {
        let c = &self.sys.cart;
        Some(match space {
            GSURAM if c.gsu.is_some() => c.sram.len() as i64,
            GSUBUS if c.gsu.is_some() => 0x80_0000,
            SA1IRAM => c.sa1.as_ref()?.iram.len() as i64,
            BWRAM if c.sa1.is_some() => c.sram.len() as i64,
            SA1BUS if c.sa1.is_some() => 0x100_0000,
            DSPRAM => Some(2 * c.dsp.as_ref()?.0.ram().len() as i64).filter(|&n| n > 0)?,
            DSPPRG => 3 * c.dsp.as_ref()?.0.lle()?.program.len() as i64,
            _ => return None,
        })
    }

    /// A byte of a chip's space, zero past its end; None for a space this cartridge does not have.
    pub fn chip_space_read(&self, space: u32, a: usize) -> Option<u8> {
        let size = self.chip_space_size(space)? as usize;
        if a >= size {
            return Some(0);
        }
        let c = &self.sys.cart;
        Some(match space {
            GSURAM | BWRAM => c.sram[a],
            GSUBUS => crate::chips::gsu::Gsu::peek(a as u32, &c.rom, &c.sram),
            SA1IRAM => c.sa1.as_ref()?.iram[a],
            SA1BUS => c.sa1.as_ref()?.peek(a as u32, &c.rom, &c.sram),
            DSPRAM => c.dsp.as_ref()?.0.ram()[a >> 1].to_le_bytes()[a & 1],
            _ => c.dsp.as_ref()?.0.lle()?.program[a / 3].to_le_bytes()[a % 3],
        })
    }

    /// A store into a chip's RAM; its bus and the DSP's program are read-only.
    pub(crate) fn chip_space_write(&mut self, space: u32, a: usize, value: u8) -> Result<(), i32> {
        let size = self.chip_space_size(space).ok_or(status::NO_SUCH_SPACE)? as usize;
        let c = &mut self.sys.cart;
        match space {
            GSUBUS | SA1BUS | DSPPRG => return Err(status::READ_ONLY),
            _ if a >= size => {}
            GSURAM | BWRAM => c.sram[a] = value,
            SA1IRAM => c.sa1.as_mut().expect("the space's size says so").iram[a] = value,
            _ => {
                let w = &mut c.dsp.as_mut().expect("the space's size says so").0.ram_mut()[a >> 1];
                let mut bytes = w.to_le_bytes();
                bytes[a & 1] = value;
                *w = u16::from_le_bytes(bytes);
            }
        }
        Ok(())
    }

    /// An ST010 or ST011, whose on-chip RAM is the cartridge's battery RAM and stands as SRAM, space 6.
    pub(crate) fn st_ram(&mut self) -> Option<&mut crate::chips::dspengine::DspEngine> {
        self.sys.cart.dsp.as_mut().map(|(d, _)| d).filter(|d| d.st())
    }
}
