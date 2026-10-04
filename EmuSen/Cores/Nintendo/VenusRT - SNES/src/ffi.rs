//! VenusRT's C ABI: the common native interface, with no optional group yet. A negative return is a status.
//! See EmuSen_NativeCores.md §3 and VenusRT_Native.md §2.

use emusen_native::abi::{self, File, FrameInfo, NativeCore, Settings};

use crate::machine::{DSP_RATE, Machine};

/// VenusRT's half of the interface version.
pub const CORE_VERSION: u16 = 1;

/// An image shorter than one 32 KiB bank after any copier header.
pub const STATUS_IMAGE_TOO_SHORT: i32 = -9;

impl NativeCore for Machine {
    const CORE_VERSION: u16 = CORE_VERSION;
    const CAPABILITIES: u64 = 0;
    const ENGINE: &'static str = "VenusRT";

    /// The battery save is file 0, clipped to the cartridge's RAM; file 2 is a NEC DSP's firmware, 8,192 or 53,248
    /// bytes, for a cartridge whose header names one, or its program with file 3 its data (6,144 and 2,048, or 49,152
    /// and 4,096). There is no file 1: the SPC700's boot program is VenusRT's own (D-38).
    fn create(image: &[u8], settings: &Settings, files: &[File<'_>]) -> Result<Self, i32> {
        if settings.keys().next().is_some() {
            return Err(abi::status::UNKNOWN_SETTING);
        }
        let mut m = Machine::load_rom(image).map_err(|_| STATUS_IMAGE_TOO_SHORT)?;
        // File 2 is the DSP's firmware whole, or its program with file 3 its data.
        let program = files.iter().find(|f| f.which == 2).map(|f| f.data);
        let data = files.iter().find(|f| f.which == 3).map(|f| f.data);
        let firmware: Option<Vec<u8>> = match (program, data) {
            (Some(p), Some(d)) => Some([p, d].concat()),
            (Some(p), None) => Some(p.to_vec()),
            (None, Some(_)) => return Err(abi::status::BAD_FILE),
            (None, None) => None,
        };
        match firmware {
            Some(f) if !m.attach_dsp(&f) => return Err(abi::status::BAD_FILE),
            Some(_) => {}
            None if m.sys.cart.wants_dsp() => {
                m.attach_replacement();
            }
            None => {}
        }
        for file in files {
            if file.which == 1 || file.which > 3 {
                return Err(abi::status::BAD_FILE);
            }
            if file.which != 0 {
                continue;
            }
            let sram = &mut m.sys.cart.sram;
            let n = sram.len().min(file.data.len());
            sram[..n].copy_from_slice(&file.data[..n]);
        }
        Ok(m)
    }

    fn advance(&mut self, _detail: &mut u64) -> Result<(), i32> {
        self.run_frame();
        Ok(())
    }

    fn set_options(&mut self, flags: u32) {
        self.sys.ppu.skip = flags & 1 != 0;
    }

    fn frame_count(&self) -> i64 {
        self.total_frames()
    }

    fn frame_info(&self) -> FrameInfo {
        // 256 or 512 wide and 224 or 448 high by what the frame held; the size is the frame's, read after it.
        let picture = self.sys.ppu.picture();
        FrameInfo { width: self.sys.ppu.frame_width as i32, height: self.sys.ppu.frame_height as i32, row_repeat: 1, flags: 0, serial: self.total_frames(), bytes: picture.len() as i64 }
    }

    fn frame(&self) -> &[u8] {
        self.sys.ppu.picture()
    }

    /// Bit n for `PadButton` n: B, Y, Select, Start, Up, Down, Left, Right, A, X, L, R; ports 0 and 1.
    fn set_buttons(&mut self, port: u32, mask: u32, changed: u32) -> Result<(), i32> {
        if let Some(pad) = self.pads.get_mut(port as usize) {
            *pad = (*pad & !(changed as u16)) | (mask as u16 & changed as u16 & 0x0FFF);
        }
        Ok(())
    }

    fn audio_rate(&self) -> i32 {
        DSP_RATE
    }

    fn audio_buffered(&self) -> usize {
        self.samples.len()
    }

    fn drain_audio(&mut self, out: &mut [i16], max_frames: usize) -> usize {
        self.samples.drain(out, max_frames)
    }

    fn set_audio_limit(&mut self, samples: usize) {
        self.samples.set_limit(samples);
    }

    fn space_size(&self, space: u32) -> Result<i64, i32> {
        Ok(match space {
            0 => 0x100_0000,
            1 => self.sys.io.len() as i64,
            2 => self.sys.wram.len() as i64,
            3 => 2 * self.sys.ppu.vram.len() as i64,
            4 => 2 * self.sys.ppu.cgram.len() as i64,
            5 => self.sys.ppu.oam.len() as i64,
            6 => match self.sys.cart.dsp.as_ref().filter(|(d, _)| d.st()) {
                Some((dsp, _)) => 2 * dsp.ram().len() as i64,
                None => self.sys.cart.sram.len() as i64,
            },
            7 => self.sys.apu.ram.len() as i64,
            _ => return self.chip_space_size(space).ok_or(abi::status::NO_SUCH_SPACE),
        })
    }

    /// CpuBus is read as the CPU would see it, but with no side effect and open bus as the MDR stands; past a
    /// space's end reads zero.
    fn space_read(&mut self, space: u32, address: u32, out: &mut [u8]) -> Result<(), i32> {
        for (i, b) in out.iter_mut().enumerate() {
            let a = address as usize + i;
            *b = match space {
                0 => self.sys.read_value(a as u32 & 0xFF_FFFF, false).unwrap_or(self.sys.mdr),
                1 => self.sys.io.get(a).copied().unwrap_or(0),
                2 => self.sys.wram.get(a).copied().unwrap_or(0),
                3 => self.sys.ppu.vram.get(a / 2).map_or(0, |w| w.to_le_bytes()[a & 1]),
                4 => self.sys.ppu.cgram.get(a / 2).map_or(0, |w| w.to_le_bytes()[a & 1]),
                5 => self.sys.ppu.oam.get(a).copied().unwrap_or(0),
                6 => match self.st_ram() {
                    Some(dsp) if a < 2 * dsp.ram().len() => dsp.host_read(crate::chips::necdsp::Port::Ram(a), false),
                    Some(_) => 0,
                    None => self.sys.cart.sram.get(a).copied().unwrap_or(0),
                },
                7 => self.sys.apu.ram.get(a).copied().unwrap_or(0),
                _ => self.chip_space_read(space, a).ok_or(abi::status::NO_SUCH_SPACE)?,
            };
        }
        Ok(())
    }

    /// Past a space's end is dropped; CpuBus lands where the bus would take a write in memory (`System::poke`), and
    /// IO is not written from outside.
    fn space_write(&mut self, space: u32, address: u32, data: &[u8]) -> Result<(), i32> {
        for (i, &b) in data.iter().enumerate() {
            let a = address as usize + i;
            let slot = match space {
                0 => {
                    self.sys.poke(a as u32 & 0xFF_FFFF, b);
                    continue;
                }
                1 => return Err(abi::status::READ_ONLY),
                2 => self.sys.wram.get_mut(a),
                3 | 4 => {
                    let words = if space == 3 { &mut self.sys.ppu.vram } else { &mut self.sys.ppu.cgram };
                    if let Some(w) = words.get_mut(a / 2) {
                        let mut bytes = w.to_le_bytes();
                        bytes[a & 1] = b;
                        *w = u16::from_le_bytes(bytes);
                    }
                    continue;
                }
                5 => self.sys.ppu.oam.get_mut(a),
                6 => {
                    if let Some(dsp) = self.st_ram() {
                        if a < 2 * dsp.ram().len() {
                            dsp.host_write(crate::chips::necdsp::Port::Ram(a), b);
                        }
                        continue;
                    }
                    self.sys.cart.sram.get_mut(a)
                }
                7 => self.sys.apu.ram.get_mut(a),
                _ => {
                    self.chip_space_write(space, a, b)?;
                    continue;
                }
            };
            if let Some(slot) = slot {
                *slot = b;
            }
        }
        Ok(())
    }

    /// The cartridge's RAM when the header says it has a battery; changes are not tracked yet.
    fn battery(&self, which: u32) -> Result<(&[u8], u32), i32> {
        if which != 0 {
            return Err(abi::status::BAD_FILE);
        }
        let cart = &self.sys.cart;
        Ok((if cart.header.battery() { &cart.sram } else { &[] }, 0))
    }
}

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
    fn chip_space_write(&mut self, space: u32, a: usize, value: u8) -> Result<(), i32> {
        let size = self.chip_space_size(space).ok_or(abi::status::NO_SUCH_SPACE)? as usize;
        let c = &mut self.sys.cart;
        match space {
            GSUBUS | SA1BUS | DSPPRG => return Err(abi::status::READ_ONLY),
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
    fn st_ram(&mut self) -> Option<&mut crate::chips::dspengine::DspEngine> {
        self.sys.cart.dsp.as_mut().map(|(d, _)| d).filter(|d| d.st())
    }
}

emusen_native::native_exports!(Machine;);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_refuses_what_it_cannot_take_and_the_spaces_reach_the_machine() {
        let image = crate::machine::tests::rom(&[0x80, 0xFE]);
        assert_eq!(Machine::create(&image[..0x4000], &Settings::default(), &[]).err(), Some(STATUS_IMAGE_TOO_SHORT));
        assert_eq!(Machine::create(&image, &Settings::parse(b"SampleRate=48000").unwrap(), &[]).err(), Some(abi::status::UNKNOWN_SETTING));
        assert_eq!(Machine::create(&image, &Settings::default(), &[File { which: 1, data: &[0; 64] }]).err(), Some(abi::status::BAD_FILE));
        assert_eq!(Machine::create(&image, &Settings::default(), &[File { which: 2, data: &[1] }]).err(), Some(abi::status::BAD_FILE));
        let mut m = Machine::create(&image, &Settings::default(), &[File { which: 0, data: &[1, 2] }]).unwrap();
        assert_eq!(m.sys.apu.cpu.pc, 0xFFC0);
        assert_eq!(m.space_size(0), Ok(0x100_0000));
        assert_eq!(m.space_size(6), Ok(0));
        assert_eq!(m.space_size(8), Err(abi::status::NO_SUCH_SPACE));
        m.space_write(2, 0x1234, &[7]).unwrap();
        let mut b = [0u8; 2];
        m.space_read(0, 0x7E_1234, &mut b[..1]).unwrap();
        m.space_read(0, 0x00_8000, &mut b[1..]).unwrap();
        assert_eq!(b, [7, 0x80]);
        m.space_write(3, 0xFFFF, &[9]).unwrap();
        m.space_read(3, 0xFFFF, &mut b[..1]).unwrap();
        assert_eq!(b[0], 9);
        m.set_buttons(1, 0xFFFF, 0xFFFF).unwrap();
        assert_eq!(m.pads, [0, 0x0FFF]);
    }
}
