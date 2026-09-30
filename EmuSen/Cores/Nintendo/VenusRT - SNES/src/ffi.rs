//! VenusRT's C ABI: the common native interface, with no optional group yet. A negative return is a status.
//! See EmuSen_NativeCores.md §3 and VenusRT_Native.md §2.

use emusen_native::abi::{self, File, FrameInfo, NativeCore, Settings};

use crate::machine::{DSP_RATE, FRAME_BYTES, Machine, SCREEN_HEIGHT, SCREEN_WIDTH};

/// VenusRT's half of the interface version.
pub const CORE_VERSION: u16 = 1;

/// An image shorter than one 32 KiB bank after any copier header.
pub const STATUS_IMAGE_TOO_SHORT: i32 = -9;

impl NativeCore for Machine {
    const CORE_VERSION: u16 = CORE_VERSION;
    const CAPABILITIES: u64 = 0;
    const ENGINE: &'static str = "VenusRT";

    /// The battery save is file 0, clipped to the cartridge's RAM; the console's firmware files, from 1, come with stage 5.
    fn create(image: &[u8], settings: &Settings, files: &[File<'_>]) -> Result<Self, i32> {
        if settings.keys().next().is_some() {
            return Err(abi::status::UNKNOWN_SETTING);
        }
        let mut m = Machine::load_rom(image).map_err(|_| STATUS_IMAGE_TOO_SHORT)?;
        for file in files {
            if file.which != 0 {
                return Err(abi::status::BAD_FILE);
            }
            let n = m.sram.len().min(file.data.len());
            m.sram[..n].copy_from_slice(&file.data[..n]);
        }
        Ok(m)
    }

    fn advance(&mut self, _detail: &mut u64) -> Result<(), i32> {
        self.run_frame();
        Ok(())
    }

    fn set_options(&mut self, flags: u32) {
        self.skip_rendering = flags & 1 != 0;
    }

    fn frame_count(&self) -> i64 {
        self.total_frames
    }

    fn frame_info(&self) -> FrameInfo {
        FrameInfo { width: SCREEN_WIDTH as i32, height: SCREEN_HEIGHT as i32, row_repeat: 1, flags: 0, serial: self.total_frames, bytes: FRAME_BYTES as i64 }
    }

    fn frame(&self) -> &[u8] {
        &self.frame_rgba
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
        self.space(space).map(|s| s.len() as i64).ok_or(abi::status::NO_SUCH_SPACE)
    }

    /// Past a space's end reads zero.
    fn space_read(&mut self, space: u32, address: u32, out: &mut [u8]) -> Result<(), i32> {
        let s = self.space(space).ok_or(abi::status::NO_SUCH_SPACE)?;
        for (i, b) in out.iter_mut().enumerate() {
            *b = s.get(address as usize + i).copied().unwrap_or(0);
        }
        Ok(())
    }

    /// Past a space's end is dropped.
    fn space_write(&mut self, space: u32, address: u32, data: &[u8]) -> Result<(), i32> {
        let s = self.space_mut(space).ok_or(abi::status::NO_SUCH_SPACE)?;
        for (i, &b) in data.iter().enumerate() {
            if let Some(slot) = s.get_mut(address as usize + i) {
                *slot = b;
            }
        }
        Ok(())
    }

    fn battery(&self, which: u32) -> Result<(&[u8], u32), i32> {
        if which != 0 {
            return Err(abi::status::BAD_FILE);
        }
        Ok((&self.sram, 0))
    }
}

emusen_native::native_exports!(Machine;);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_refuses_what_stage_0_cannot_take() {
        let image = vec![0u8; 0x8000];
        assert_eq!(Machine::create(&image[..0x4000], &Settings::default(), &[]).err(), Some(STATUS_IMAGE_TOO_SHORT));
        assert_eq!(Machine::create(&image, &Settings::parse(b"SampleRate=48000").unwrap(), &[]).err(), Some(abi::status::UNKNOWN_SETTING));
        assert_eq!(Machine::create(&image, &Settings::default(), &[File { which: 1, data: &[1] }]).err(), Some(abi::status::BAD_FILE));
        let mut m = Machine::create(&image, &Settings::default(), &[File { which: 0, data: &[1, 2] }]).unwrap();
        assert_eq!(m.space_size(0), Err(abi::status::NO_SUCH_SPACE));
        assert_eq!(m.space_size(6), Ok(0));
        m.space_write(3, 0xFFFF, &[7, 8]).unwrap();
        let mut two = [9u8; 2];
        m.space_read(3, 0xFFFF, &mut two).unwrap();
        assert_eq!(two, [7, 0]);
        m.set_buttons(1, 0xFFFF, 0xFFFF).unwrap();
        m.set_buttons(2, 0xFFFF, 0xFFFF).unwrap();
        assert_eq!(m.pads, [0, 0x0FFF]);
    }
}
