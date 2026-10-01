//! The machine: the 65816 over the system bus, run a frame at a time by the master clock. The PPU, APU and the
//! S-CPU's other devices come with later stages; their memories are kept for the spaces and the state.

use emusen_native::SampleQueue;

use crate::bus::System;
use crate::cart::Cartridge;
use crate::cpu::{Cpu, Interrupt};
use crate::state::{STATE_MAGIC, STATE_VERSION, StateError, StateReader, StateResult, StateWriter};

pub const SCREEN_WIDTH: usize = 256;
pub const SCREEN_HEIGHT: usize = 224;
pub const FRAME_BYTES: usize = SCREEN_WIDTH * SCREEN_HEIGHT * 4;
/// The S-DSP's output rate, nominal (VenusRT_Plan.md §5.3).
pub const DSP_RATE: i32 = 32_000;

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
    pub cpu: Cpu,
    pub sys: System,
    pub cgram: Box<[u8]>,
    pub oam: Box<[u8]>,
    pub apuram: Box<[u8]>,
    /// B, Y, Select, Start, Up, Down, Left, Right, A, X, L, R from bit 0, per port; not in the state.
    pub pads: [u16; 2],
    pub skip_rendering: bool,
    pub frame_rgba: Box<[u8]>,
    pub samples: SampleQueue,
}

impl Machine {
    /// The cartridge from the image, and the CPU through its reset sequence (fullsnes: H=0, V=0 after /RESET).
    pub fn load_rom(image: &[u8]) -> Result<Machine, ImageTooShort> {
        let cart = Cartridge::new(image).ok_or(ImageTooShort(image.len()))?;
        let mut frame = vec![0u8; FRAME_BYTES];
        for px in frame.chunks_exact_mut(4) {
            px[3] = 0xFF;
        }
        let mut m = Machine {
            cpu: Cpu::default(),
            sys: System::new(cart),
            cgram: vec![0; CGRAM_BYTES].into(),
            oam: vec![0; OAM_BYTES].into(),
            apuram: vec![0; APURAM_BYTES].into(),
            pads: [0; 2],
            skip_rendering: false,
            frame_rgba: frame.into(),
            samples: SampleQueue::default(),
        };
        m.cpu.interrupt(&mut m.sys, Interrupt::Reset);
        Ok(m)
    }

    pub fn total_frames(&self) -> i64 {
        self.sys.timing.frame as i64
    }

    /// To the next frame's first line.
    pub fn run_frame(&mut self) {
        let frame = self.sys.timing.frame;
        while self.sys.timing.frame == frame {
            self.cpu.step(&mut self.sys);
        }
    }

    pub fn vram_bytes(&self) -> Vec<u8> {
        self.sys.vram.iter().flat_map(|w| w.to_le_bytes()).collect()
    }

    fn write_state(&self, w: &mut StateWriter) {
        let c = &self.cpu;
        let t = &self.sys.timing;
        w.u32("Magic", STATE_MAGIC);
        w.i32("Version", STATE_VERSION);
        w.group("Cpu", |w| {
            for (n, v) in [("A", c.a), ("X", c.x), ("Y", c.y), ("S", c.s), ("D", c.d), ("PC", c.pc)] {
                w.u16(n, v);
            }
            w.u8("DBR", c.dbr);
            w.u8("PBR", c.pbr);
            w.u8("P", c.p);
            w.bool("E", c.e);
            w.bool("Waiting", c.waiting);
            w.bool("Stopped", c.stopped);
        });
        w.group("Timing", |w| {
            w.u64("Clock", t.clock);
            w.u16("Line", t.line);
            w.u16("LineClock", t.line_clock);
            w.bool("Field", t.field);
            w.u64("Frame", t.frame);
            w.bool("Refreshed", t.refreshed);
            w.bool("VBlank", t.vblank);
            w.bool("NmiFlag", t.nmi_flag);
        });
        w.group("Bus", |w| {
            w.u8("Mdr", self.sys.mdr);
            w.u32("WramAddress", self.sys.wram_address);
            w.bool("FastRom", self.sys.fast_rom);
            w.bytes("Io", &self.sys.io);
            w.u16("VramAddress", self.sys.vram_address);
            w.u8("Vmain", self.sys.vmain);
        });
        w.bytes("Wram", &self.sys.wram);
        w.u16s("Vram", &self.sys.vram);
        w.bytes("Cgram", &self.cgram);
        w.bytes("Oam", &self.oam);
        w.bytes("ApuRam", &self.apuram);
        w.bytes("Sram", &self.sys.cart.sram);
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
        let c = &mut self.cpu;
        for v in [&mut c.a, &mut c.x, &mut c.y, &mut c.s, &mut c.d, &mut c.pc] {
            *v = r.u16()?;
        }
        c.dbr = r.u8()?;
        c.pbr = r.u8()?;
        c.p = r.u8()?;
        c.e = r.bool()?;
        c.waiting = r.bool()?;
        c.stopped = r.bool()?;
        let t = &mut self.sys.timing;
        t.clock = r.u64()?;
        t.line = r.u16()?;
        t.line_clock = r.u16()?;
        t.field = r.bool()?;
        t.frame = r.u64()?;
        t.refreshed = r.bool()?;
        t.vblank = r.bool()?;
        t.nmi_flag = r.bool()?;
        self.sys.mdr = r.u8()?;
        self.sys.wram_address = r.u32()? & 0x1FFFF;
        self.sys.fast_rom = r.bool()?;
        r.bytes(&mut self.sys.io)?;
        self.sys.vram_address = r.u16()?;
        self.sys.vmain = r.u8()?;
        r.bytes(&mut self.sys.wram)?;
        r.u16s(&mut self.sys.vram)?;
        r.bytes(&mut self.cgram)?;
        r.bytes(&mut self.oam)?;
        r.bytes(&mut self.apuram)?;
        r.bytes(&mut self.sys.cart.sram)?;
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
pub(crate) mod tests {
    use super::*;
    use crate::state::VENUS_MAGIC;

    /// A LoROM image whose reset runs `program` at $00:8000.
    pub fn rom(program: &[u8]) -> Vec<u8> {
        let mut image = vec![0xEAu8; 0x8000];
        image[..program.len()].copy_from_slice(program);
        image[0x7FC0..0x7FD5].copy_from_slice(b"VENUSRT MACHINE TEST ");
        image[0x7FD5] = 0x20;
        image[0x7FDC..0x7FE0].copy_from_slice(&[0xFF, 0xFF, 0x00, 0x00]);
        image[0x7FFC] = 0x00;
        image[0x7FFD] = 0x80;
        image
    }

    fn save(m: &Machine) -> Vec<u8> {
        let mut out = vec![0u8; m.state_size()];
        assert_eq!(m.save_state(&mut out), Ok(out.len()));
        out
    }

    #[test]
    fn a_copier_header_is_dropped_and_a_short_image_refused() {
        let mut copier = vec![0u8; 512];
        copier.extend(rom(&[]));
        assert_eq!(Machine::load_rom(&copier).unwrap().sys.cart.rom.len(), 0x8000);
        assert_eq!(Machine::load_rom(&[0; 0x7FFF]).err(), Some(ImageTooShort(0x7FFF)));
    }

    #[test]
    fn reset_takes_the_vector_and_a_frame_is_the_master_clock_s() {
        let mut m = Machine::load_rom(&rom(&[0x80, 0xFE])).unwrap();
        assert_eq!((m.cpu.pbr, m.cpu.pc, m.cpu.e), (0, 0x8000, true));
        m.run_frame();
        m.run_frame();
        assert_eq!(m.total_frames(), 2);
        assert!(m.sys.timing.clock >= 2 * 262 * 1364 - 4);
    }

    // Version 2 carries the CPU, the clock and the bus; the listing is its record (VenusRT_Plan.md §5.6).
    #[test]
    fn the_version_2_layout_is_pinned() {
        let m = Machine::load_rom(&rom(&[])).unwrap();
        let layout = m.layout();
        assert!(layout.starts_with("0 4 u32 Magic\n4 4 i32 Version\n8 2 u16 Cpu.A\n"), "{layout}");
        assert!(layout.contains(" u64 Timing.Clock\n") && layout.contains(" u8[1024] Bus.Io\n") && layout.contains(" u16[32768] Vram\n"), "{layout}");
        assert_eq!(layout.lines().count(), 34, "{layout}");
        assert_eq!(m.state_size(), 264_283);
        assert_eq!(&save(&m)[..4], b"VNRT");
    }

    #[test]
    fn a_state_round_trips_and_a_refused_one_changes_nothing() {
        let mut m = Machine::load_rom(&rom(&[0xEE, 0x00, 0x10, 0x80, 0xFB])).unwrap();
        m.run_frame();
        let state = save(&m);
        let mut back = Machine::load_rom(&rom(&[0xEE, 0x00, 0x10, 0x80, 0xFB])).unwrap();
        back.load_state(&state).unwrap();
        assert_eq!(save(&back), state);
        back.run_frame();
        m.run_frame();
        assert_eq!(save(&back), save(&m));
        let mut venus = state.clone();
        venus[..4].copy_from_slice(&VENUS_MAGIC.to_le_bytes());
        assert_eq!(back.load_state(&venus), Err(StateError::Foreign(VENUS_MAGIC)));
        assert!(matches!(back.load_state(&state[..state.len() - 1]), Err(StateError::Truncated { .. })));
    }
}
