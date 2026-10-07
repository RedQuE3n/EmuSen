//! Nephrite's own state format, "NPHR" (Nephrite_Plan.md §5.7): the magic, the version, the system, the frame count
//! and every writable memory in id order. The layout listing is pinned by a test, there being no other record.

use emusen_native::ffi::{StateMachine, Status, status};
use emusen_native::{StateReader, StateWriter, Truncated};

use crate::machine::Machine;
use crate::media::System;

pub const STATE_MAGIC: u32 = u32::from_le_bytes(*b"NPHR");
pub const STATE_VERSION: i32 = 17;
/// A state of this core made for another of its systems: a Genesis state offered to a 32X machine.
pub const STATUS_OTHER_SYSTEM: i32 = -10;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateError {
    Truncated { at: usize, wanted: usize },
    Foreign(u32),
    Version(i32),
    OtherSystem(u8),
    BufferTooSmall { needed: usize },
}

impl Status for StateError {
    fn status(&self) -> i32 {
        match self {
            StateError::Truncated { .. } => status::TRUNCATED,
            StateError::Foreign(_) => status::FOREIGN,
            StateError::Version(_) => status::VERSION,
            StateError::OtherSystem(_) => STATUS_OTHER_SYSTEM,
            StateError::BufferTooSmall { .. } => status::BUFFER_TOO_SMALL,
        }
    }
}

impl From<Truncated> for StateError {
    fn from(t: Truncated) -> Self {
        StateError::Truncated { at: t.at, wanted: t.wanted }
    }
}

fn system_byte(s: System) -> u8 {
    match s {
        System::Md => 0,
        System::Mcd => 1,
        System::S32x => 2,
    }
}

impl Machine {
    fn write(&self, w: &mut StateWriter) {
        w.u32("Magic", STATE_MAGIC);
        w.i32("Version", STATE_VERSION);
        w.u8("System", system_byte(self.media.system));
        w.i64("Frames", self.frames);
        for m in self.spaces().into_iter().filter(|m| !m.read_only) {
            w.bytes(m.name, self.bytes(m.id).expect("a listed space"));
        }
        self.genesis.write_state(w);
    }
}

impl StateMachine for Machine {
    type Error = StateError;

    fn load_state(&mut self, data: &[u8]) -> Result<(), StateError> {
        let mut r = StateReader::new(data);
        let magic = r.u32().map_err(|_| StateError::Foreign(0))?;
        if magic != STATE_MAGIC {
            return Err(StateError::Foreign(magic));
        }
        let version = r.i32()?;
        if version != STATE_VERSION {
            return Err(StateError::Version(version));
        }
        let system = r.u8()?;
        if system != system_byte(self.media.system) {
            return Err(StateError::OtherSystem(system));
        }
        let frames = r.i64()?;
        let mut loaded = Vec::new();
        for m in self.spaces().into_iter().filter(|m| !m.read_only) {
            let mut bytes = vec![0; self.bytes(m.id).expect("a listed space").len()];
            r.bytes(&mut bytes)?;
            loaded.push((m.id, bytes));
        }
        let registers = self.genesis.read_state(&mut r)?;
        self.frames = frames;
        for (id, bytes) in loaded {
            self.bytes_mut(id).expect("a listed space").copy_from_slice(&bytes);
        }
        self.genesis.apply_state(registers);
        self.refresh_battery_file();
        Ok(())
    }

    fn state_size(&self) -> usize {
        let mut w = StateWriter::counter();
        self.write(&mut w);
        w.len()
    }

    fn save_state(&self, out: &mut [u8]) -> Result<usize, StateError> {
        let needed = self.state_size();
        if out.len() < needed {
            return Err(StateError::BufferTooSmall { needed });
        }
        let mut w = StateWriter::new(out);
        self.write(&mut w);
        Ok(w.len())
    }

    fn layout(&self) -> String {
        let mut w = StateWriter::layout();
        self.write(&mut w);
        w.into_layout()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::{Media, cartridge};

    fn machine(system: &str) -> Machine {
        let rom = cartridge(system, "U", None);
        Machine::new(&rom, Media::read(&rom))
    }

    #[test]
    fn the_layout_is_pinned() {
        assert_eq!(
            machine("SEGA GENESIS").layout(),
            concat!(
                "0 4 u32 Magic\n4 4 i32 Version\n8 1 u8 System\n9 8 i64 Frames\n17 65536 u8[65536] WRAM\n",
                "65553 8192 u8[8192] Z80RAM\n73745 65536 u8[65536] VRAM\n139281 128 u8[128] CRAM\n",
                "139409 80 u8[80] VSRAM\n139489 32 u32[8] M68000.D\n139521 32 u32[8] M68000.A\n",
                "139553 4 u32 M68000.OtherSp\n139557 2 u16 M68000.Sr\n139559 4 u32 M68000.Pc\n",
                "139563 4 u16[2] M68000.Prefetch\n139567 1 bool M68000.Stopped\n139568 1 bool M68000.Halted\n",
                "139569 1 u8 M68000.LastLevel\n139570 1 bool M68000.TracePending\n139571 4 u32 M68000.TraceResume\n",
                "139575 8 u16[4] Z80.Main\n139583 8 u16[4] Z80.Alternate\n139591 4 u16[2] Z80.Index\n",
                "139595 2 u16 Z80.Sp\n139597 2 u16 Z80.Pc\n139599 1 u8 Z80.I\n139600 1 u8 Z80.R\n",
                "139601 2 u16 Z80.Wz\n139603 1 u8 Z80.Q\n139604 1 bool Z80.P\n139605 1 bool Z80.Iff1\n",
                "139606 1 bool Z80.Iff2\n139607 1 u8 Z80.Im\n139608 1 bool Z80.EiPending\n139609 1 bool Z80.Halted\n",
                "139610 24 u8[24] VdpRegisters\n139634 5 bool[5] VdpLatches\n139639 4 u32 HvLatch\n",
                "139643 1 u8 VdpCode\n139644 2 u16 VdpAddress\n139646 1 u8 VdpLineCounter\n",
                "139647 24 u16[12] VdpFifo\n139671 2 u8[2] VdpFifoPointers\n139673 32 u64[4] VdpFifoReady\n",
                "139705 264 u64[33] VdpLanding\n139969 2 u16 VdpReadBuffer\n139971 32 u64[4] VdpDma\n",
                "140003 16 u64[2] VdpTimes\n140019 320 u8[320] SpriteCache\n140339 3 bool[3] SpriteFlags\n",
                "140342 2 u16 VscrollLatch\n140344 2 u8[2] Field\n140346 320 u8[320] SpriteLineBuffer\n",
                "140666 80 u8[80] LineVsram\n140746 4 u16[2] LineHscroll\n140750 2 u8[2] LineWindow\n",
                "140752 4 u16[2] OpenLine\n140756 640 u8[640] OpenLineSprites\n141396 80 u8[80] OpenLineVsram\n",
                "141476 4 u16[2] OpenLineScroll\n141480 2 u8[2] OpenLineWindow\n141482 4 u16[2] FrameSize\n",
                "141486 3 u8[3] IoData\n",
                "141489 3 u8[3] IoCtrl\n141492 3 u8[3] IoTx\n141495 3 u8[3] IoSctrl\n141498 2 bool[2] PadTh\n",
                "141500 2 u8[2] PadFalls\n141502 16 u64[2] PadLastFall\n141518 1 u8 SramRegister\n",
                "141519 1 bool Mapper\n141520 8 u8[8] MapperPages\n141528 80 u64[10] Clocks\n141608 4 u32 Line\n",
                "141612 2 u16 Z80Bank\n141614 2 u16 OpenBus\n141616 4 bool[4] Lines\n141620 4 u8[4] Tmss\n",
                "141624 256 u8[256] Sound.Part0\n141880 256 u8[256] Sound.Part1\n",
                "142136 6 u8[6] Sound.AddressPartFlags\n142142 4 u16[2] Sound.Counters\n",
                "142146 24 u64[3] Sound.Times\n142170 8 i32[2] Sound.Out\n142178 24 i32[6] Sound.Channels\n",
                "142202 5 u8[5] Sound.Timers\n142207 40 u64[5] Sound.TimersAndCounts\n",
                "142247 1024 u64[128] Sound.Writes\n143271 32 u64[4] Sound.Csm\n",
                "143303 1056 u64[132] Sound.FmOperators\n144359 24 bool[24] Sound.FmKeys\n",
                "144383 16 u64[2] Sound.FmEnvelope\n144399 4 u8[4] Sound.FmLatches\n144403 3 u8[3] Sound.FmLfo\n",
                "144406 31 bool[31] Sound.FmSsgKeys\n144437 48 u16[24] Sound.FmShown\n144485 6 u16[3] Sound.Tone\n",
                "144491 4 u8[4] Sound.Volume\n144495 3 u8[3] Sound.NoiseLatchStereo\n144498 8 u16[4] Sound.Counter\n",
                "144506 4 bool[4] Sound.Bit\n144510 2 u16 Sound.Shift\n144512 16 u64[2] Sound.PsgTimeAndTime\n",
                "144528 80 i32[20] Sound.Lines\n144608 8 i32[2] Sound.Level\n144616 24 u64[3] Sound.Sum\n",
                "144640 1024 u64[128] Sound.Pending\n145664 8 i32[2] Sound.Level\n145672 24 u64[3] Sound.Sum\n",
                "145696 1024 u64[128] Sound.Pending\n146720 88 u64[11] Sound.Circuit\n",
                "146808 160 u64[20] Sound.CircuitSides\n",
            )
        );
    }

    // A state is taken at a frame's end, with the next frame's line 0 already open and the picture's size taken from it; a machine that loads it keeps that size, whatever its own was (Nephrite_Native.md §39).
    #[test]
    fn a_loaded_state_keeps_the_size_line_0_gave_the_frame() {
        let mut a = machine("SEGA GENESIS");
        (a.genesis.hw.vdp.regs[1], a.genesis.hw.vdp.regs[12]) = (0x44, 0x81);
        for _ in 0..3 {
            a.advance();
        }
        // A write at the frame's first slots opens line 0 in H40, and the width is then set to H32 for the frame after.
        a.genesis.hw.vdp.before_change(None);
        a.genesis.hw.vdp.regs[12] = 0;
        assert_eq!((a.genesis.hw.vdp.open, a.genesis.hw.vdp.frame.width, a.genesis.hw.vdp.width()), (Some(0), 320, 256));
        let mut s = vec![0; a.state_size()];
        a.save_state(&mut s).unwrap();
        let mut b = machine("SEGA GENESIS");
        b.load_state(&s).unwrap();
        assert_eq!((b.genesis.hw.vdp.frame.width, b.genesis.hw.vdp.frame.height), (320, a.genesis.hw.vdp.frame.height));
        a.advance();
        b.advance();
        assert_eq!((b.picture.len(), &b.picture), (320 * 224 * 4, &a.picture));
    }

    // An EEPROM board's transfer under way is in the state, beside its bytes (Nephrite_Native.md §39).
    #[test]
    fn a_loaded_state_keeps_an_eeprom_transfer_where_it_stood() {
        let eeprom_cart = || {
            let mut rom = cartridge("SEGA GENESIS", "U", None);
            rom[0x180..0x18E].copy_from_slice(b"GM T-081586-00");
            Machine::new(&rom, Media::read(&rom))
        };
        let mut a = eeprom_cart();
        let e = a.genesis.hw.cart.eeprom.as_mut().expect("the board");
        // SDA on bit 0 of $200001 and SCL on bit 0 of $200000: a start, then three bits of a command clocked in.
        for (sda, scl) in [(1, 1), (0, 1), (0, 0), (1, 0), (1, 1), (0, 0), (0, 1), (1, 0), (1, 1), (1, 0)] {
            e.write(&[(0x20_0001, sda), (0x20_0000, scl)]);
        }
        let taken = e.protocol();
        assert_ne!(taken, crate::eeprom::Eeprom::new(e.board).protocol());
        let mut s = vec![0; a.state_size()];
        a.save_state(&mut s).unwrap();
        let mut b = eeprom_cart();
        b.load_state(&s).unwrap();
        assert_eq!(b.genesis.hw.cart.eeprom.as_ref().unwrap().protocol(), taken);
    }

    #[test]
    fn a_state_round_trips_and_a_refused_one_changes_nothing() {
        let mut a = machine("SEGA GENESIS");
        a.frames = 7;
        a.bytes_mut(2).unwrap()[5] = 0x5A;
        let mut s = vec![0; a.state_size()];
        a.save_state(&mut s).unwrap();
        let mut b = machine("SEGA GENESIS");
        b.load_state(&s).unwrap();
        assert_eq!((b.frames, b.bytes(2).unwrap()[5]), (7, 0x5A));

        let before = b.bytes(2).unwrap().to_vec();
        assert_eq!(b.load_state(&s[..s.len() - 1]).unwrap_err().status(), status::TRUNCATED);
        assert_eq!(b.load_state(b"SNES0000").unwrap_err().status(), status::FOREIGN);
        assert_eq!(b.load_state(&[]).unwrap_err().status(), status::FOREIGN);
        assert_eq!(machine("SEGA 32X").load_state(&s).unwrap_err().status(), STATUS_OTHER_SYSTEM);
        assert_eq!((b.frames, b.bytes(2).unwrap()), (7, &before[..]));
    }
}
