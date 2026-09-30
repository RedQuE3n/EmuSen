//! The machine. At stage 0 it holds the console's memories, a blank picture and the frame count, and nothing runs.

use emusen_native::SampleQueue;

use crate::state::{STATE_MAGIC, STATE_VERSION, StateError, StateReader, StateResult, StateWriter};

pub const SCREEN_WIDTH: usize = 256;
pub const SCREEN_HEIGHT: usize = 224;
pub const FRAME_BYTES: usize = SCREEN_WIDTH * SCREEN_HEIGHT * 4;
/// The S-DSP's output rate, nominal (VenusRT_Plan.md §5.3).
pub const DSP_RATE: i32 = 32_000;

pub const WRAM_BYTES: usize = 0x20000;
pub const VRAM_BYTES: usize = 0x10000;
pub const CGRAM_BYTES: usize = 0x200;
pub const OAM_BYTES: usize = 0x220;
pub const APURAM_BYTES: usize = 0x10000;

/// The memory spaces by id, named as C# Venus's debug target names them (VenusRT_Plan.md §4.7).
pub const SPACE_NAMES: [&str; 8] = ["CpuBus", "IO", "WRAM", "VRAM", "CGRAM", "OAM", "SRAM", "APURAM"];

/// An image shorter than one 32 KiB bank, after any copier header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageTooShort(pub usize);

#[derive(Clone)]
pub struct Machine {
    pub rom: Box<[u8]>,
    pub total_frames: i64,
    pub wram: Box<[u8]>,
    pub vram: Box<[u8]>,
    pub cgram: Box<[u8]>,
    pub oam: Box<[u8]>,
    pub apuram: Box<[u8]>,
    pub sram: Box<[u8]>,
    /// B, Y, Select, Start, Up, Down, Left, Right, A, X, L, R from bit 0, per port; not in the state.
    pub pads: [u16; 2],
    pub skip_rendering: bool,
    pub frame_rgba: Box<[u8]>,
    pub samples: SampleQueue,
}

impl Machine {
    /// A copier's 512-byte header is dropped, as the file's length modulo 1 KiB shows it.
    pub fn load_rom(image: &[u8]) -> Result<Machine, ImageTooShort> {
        let rom = if image.len() % 1024 == 512 { &image[512..] } else { image };
        if rom.len() < 0x8000 {
            return Err(ImageTooShort(rom.len()));
        }
        let mut frame = vec![0u8; FRAME_BYTES];
        for px in frame.chunks_exact_mut(4) {
            px[3] = 0xFF;
        }
        Ok(Machine {
            rom: rom.into(),
            total_frames: 0,
            wram: vec![0; WRAM_BYTES].into(),
            vram: vec![0; VRAM_BYTES].into(),
            cgram: vec![0; CGRAM_BYTES].into(),
            oam: vec![0; OAM_BYTES].into(),
            apuram: vec![0; APURAM_BYTES].into(),
            sram: Box::default(),
            pads: [0; 2],
            skip_rendering: false,
            frame_rgba: frame.into(),
            samples: SampleQueue::default(),
        })
    }

    pub fn run_frame(&mut self) {
        self.total_frames += 1;
    }

    pub fn space(&self, space: u32) -> Option<&[u8]> {
        Some(match space {
            2 => &self.wram,
            3 => &self.vram,
            4 => &self.cgram,
            5 => &self.oam,
            6 => &self.sram,
            7 => &self.apuram,
            _ => return None,
        })
    }

    pub fn space_mut(&mut self, space: u32) -> Option<&mut [u8]> {
        Some(match space {
            2 => &mut self.wram,
            3 => &mut self.vram,
            4 => &mut self.cgram,
            5 => &mut self.oam,
            6 => &mut self.sram,
            7 => &mut self.apuram,
            _ => return None,
        })
    }

    fn write_state(&self, w: &mut StateWriter) {
        w.u32("Magic", STATE_MAGIC);
        w.i32("Version", STATE_VERSION);
        w.i64("TotalFrames", self.total_frames);
        w.bytes("Wram", &self.wram);
        w.bytes("Vram", &self.vram);
        w.bytes("Cgram", &self.cgram);
        w.bytes("Oam", &self.oam);
        w.bytes("ApuRam", &self.apuram);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        let magic = r.u32()?;
        if magic != STATE_MAGIC {
            return Err(StateError::Foreign(magic));
        }
        let version = r.i32()?;
        if version != STATE_VERSION {
            return Err(StateError::Version(version));
        }
        r.set_version(version);
        self.total_frames = r.i64()?;
        for memory in [&mut self.wram, &mut self.vram, &mut self.cgram, &mut self.oam, &mut self.apuram] {
            r.bytes(memory)?;
        }
        Ok(())
    }

    /// A failed load changes nothing.
    pub fn load_state(&mut self, data: &[u8]) -> StateResult {
        let mut next = self.clone();
        next.read_state(&mut StateReader::new(data))?;
        *self = next;
        Ok(())
    }

    pub fn state_size(&self) -> usize {
        let mut w = StateWriter::counter();
        self.write_state(&mut w);
        w.len()
    }

    pub fn save_state(&self, out: &mut [u8]) -> StateResult<usize> {
        let mut w = StateWriter::new(out);
        self.write_state(&mut w);
        if w.overflowed() {
            return Err(StateError::BufferTooSmall { needed: w.len() });
        }
        Ok(w.len())
    }

    pub fn layout(&self) -> String {
        let mut w = StateWriter::layout();
        self.write_state(&mut w);
        w.into_layout()
    }
}

impl emusen_native::ffi::StateMachine for Machine {
    type Error = StateError;
    fn load_state(&mut self, data: &[u8]) -> StateResult {
        Machine::load_state(self, data)
    }
    fn state_size(&self) -> usize {
        Machine::state_size(self)
    }
    fn save_state(&self, out: &mut [u8]) -> StateResult<usize> {
        Machine::save_state(self, out)
    }
    fn layout(&self) -> String {
        Machine::layout(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::VENUS_MAGIC;

    fn save(m: &Machine) -> Vec<u8> {
        let mut out = vec![0u8; m.state_size()];
        assert_eq!(m.save_state(&mut out), Ok(out.len()));
        out
    }

    #[test]
    fn a_copier_header_is_dropped_and_a_short_image_refused() {
        assert_eq!(Machine::load_rom(&vec![0xAA; 0x8000 + 512]).unwrap().rom.len(), 0x8000);
        assert_eq!(Machine::load_rom(&[0; 0x7FFF]).err(), Some(ImageTooShort(0x7FFF)));
    }

    // Without a C# oracle the listing is the record of what version 1 means (VenusRT_Plan.md §5.6).
    #[test]
    fn the_version_1_layout_is_pinned() {
        let m = Machine::load_rom(&[0; 0x8000]).unwrap();
        assert_eq!(m.layout(), "0 4 u32 Magic\n4 4 i32 Version\n8 8 i64 TotalFrames\n16 131072 u8[131072] Wram\n131088 65536 u8[65536] Vram\n196624 512 u8[512] Cgram\n197136 544 u8[544] Oam\n197680 65536 u8[65536] ApuRam\n");
        assert_eq!(m.state_size(), 263216);
        assert_eq!(&save(&m)[..4], b"VNRT");
    }

    #[test]
    fn a_state_round_trips_and_a_refused_one_changes_nothing() {
        let mut m = Machine::load_rom(&[0; 0x8000]).unwrap();
        m.run_frame();
        m.vram[0x1234] = 0x56;
        m.apuram[0xFFFF] = 0x78;
        let state = save(&m);
        let mut back = Machine::load_rom(&[0; 0x8000]).unwrap();
        back.load_state(&state).unwrap();
        assert_eq!(save(&back), state);
        assert_eq!((back.total_frames, back.vram[0x1234]), (1, 0x56));

        let mut venus = state.clone();
        venus[..4].copy_from_slice(&VENUS_MAGIC.to_le_bytes());
        assert_eq!(&venus[..4], b"SNES");
        assert_eq!(back.load_state(&venus), Err(StateError::Foreign(VENUS_MAGIC)));
        let mut later = state.clone();
        later[4] = 2;
        assert_eq!(back.load_state(&later), Err(StateError::Version(2)));
        assert!(matches!(back.load_state(&state[..state.len() - 1]), Err(StateError::Truncated { .. })));
        assert_eq!(save(&back), state);
    }
}
