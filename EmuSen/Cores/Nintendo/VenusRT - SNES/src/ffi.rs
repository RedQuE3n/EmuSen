//! VenusRT's C ABI: the common native interface, with no optional group yet. A negative return is a status.
//! See EmuSen_NativeCores.md §3 and VenusRT_Native.md §2.

use emusen_native::abi::{self, File, FrameInfo, NativeCore, Settings};

use crate::machine::{DSP_RATE, Machine};

/// VenusRT's half of the interface version.
pub const CORE_VERSION: u16 = 1;

/// An image shorter than one 32 KiB bank after any copier header.
pub const STATUS_IMAGE_TOO_SHORT: i32 = -9;

/// The sound unit's 64-byte boot ROM, file 1, was not given; the console's firmware is the frontend's to supply.
pub const STATUS_NO_IPL: i32 = -10;

impl NativeCore for Machine {
    const CORE_VERSION: u16 = CORE_VERSION;
    const CAPABILITIES: u64 = 0;
    const ENGINE: &'static str = "VenusRT";

    /// The battery save is file 0, clipped to the cartridge's RAM; file 1 is the SPC700's 64-byte boot ROM, required;
    /// file 2 is a NEC DSP's firmware, 8,192 or 53,248 bytes, for a cartridge whose header names one.
    fn create(image: &[u8], settings: &Settings, files: &[File<'_>]) -> Result<Self, i32> {
        if settings.keys().next().is_some() {
            return Err(abi::status::UNKNOWN_SETTING);
        }
        let mut m = Machine::load_rom(image).map_err(|_| STATUS_IMAGE_TOO_SHORT)?;
        let ipl = match files.iter().find(|f| f.which == 1) {
            Some(f) => <[u8; 64]>::try_from(f.data).map_err(|_| abi::status::BAD_FILE)?,
            None => return Err(STATUS_NO_IPL),
        };
        let pal = m.sys.timing.pal;
        m.sys.apu = crate::apu::smp::Smp::new(Some(ipl), pal);
        for file in files {
            if file.which > 2 || (file.which == 2 && !m.attach_dsp(file.data)) {
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
            6 => self.sys.cart.sram.len() as i64,
            7 => self.sys.apu.ram.len() as i64,
            _ => return Err(abi::status::NO_SUCH_SPACE),
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
                6 => self.sys.cart.sram.get(a).copied().unwrap_or(0),
                7 => self.sys.apu.ram.get(a).copied().unwrap_or(0),
                _ => return Err(abi::status::NO_SUCH_SPACE),
            };
        }
        Ok(())
    }

    /// Past a space's end is dropped; CpuBus and IO are not written by the host yet.
    fn space_write(&mut self, space: u32, address: u32, data: &[u8]) -> Result<(), i32> {
        for (i, &b) in data.iter().enumerate() {
            let a = address as usize + i;
            let slot = match space {
                0 | 1 => return Err(abi::status::READ_ONLY),
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
                6 => self.sys.cart.sram.get_mut(a),
                7 => self.sys.apu.ram.get_mut(a),
                _ => return Err(abi::status::NO_SUCH_SPACE),
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

emusen_native::native_exports!(Machine;);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_refuses_what_it_cannot_take_and_the_spaces_reach_the_machine() {
        let image = crate::machine::tests::rom(&[0x80, 0xFE]);
        assert_eq!(Machine::create(&image[..0x4000], &Settings::default(), &[]).err(), Some(STATUS_IMAGE_TOO_SHORT));
        assert_eq!(Machine::create(&image, &Settings::parse(b"SampleRate=48000").unwrap(), &[]).err(), Some(abi::status::UNKNOWN_SETTING));
        assert_eq!(Machine::create(&image, &Settings::default(), &[]).err(), Some(STATUS_NO_IPL));
        assert_eq!(Machine::create(&image, &Settings::default(), &[File { which: 1, data: &[1] }]).err(), Some(abi::status::BAD_FILE));
        let ipl = crate::apu::smp::tests::idle_ipl();
        assert_eq!(Machine::create(&image, &Settings::default(), &[File { which: 2, data: &[1] }, File { which: 1, data: &ipl }]).err(), Some(abi::status::BAD_FILE));
        let mut m = Machine::create(&image, &Settings::default(), &[File { which: 0, data: &[1, 2] }, File { which: 1, data: &ipl }]).unwrap();
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
