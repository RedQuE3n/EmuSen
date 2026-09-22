//! The whole machine, the C# `MarsCore` without its host: its save state (Mars_Native.md §5.1), boot, the frame, and a load's derivations (§5.2).

use std::sync::Arc;

use crate::Skip;
use crate::bus::{MemoryBus, RDRAM_SIZE, RDRAM_SIZE_EXPANDED};
use crate::controller::ControllerPak;
use crate::cop0::{COMPARE, CONFIG, CONFIG_AT_RESET, PROCESSOR_ID, PROCESSOR_ID_REGISTER, STATUS};
use crate::cpu::Cpu;
use crate::dp::SNAPSHOT_WORDS;
use crate::rom::{self, Cic, RomImage};
use crate::save::{SaveChip, save_type};
use crate::state::{State, StateError, StateReader, StateResult, StateWriter};

/// `StateMagic`: "MARS" little-endian.
pub const STATE_MAGIC: u32 = 0x5352_414D;
/// `StateVersion`: a state, written once the RDP's thread has drained.
pub const STATE_VERSION: i32 = 1;
/// `SnapshotVersion`: the same body, then the RDP words not yet run.
pub const SNAPSHOT_VERSION: i32 = 2;
/// `MarsCore.CycleCap`, which `_lastFrameCycles` starts at.
pub const CYCLE_CAP: i64 = 4_100_000;

/// The whole machine, as a `MarsCore` saves it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Machine {
    /// `TotalFrames`.
    pub total_frames: i64,
    /// `_lastFrameCycles`.
    pub last_frame_cycles: i64,
    pub cpu: Cpu,
    pub bus: MemoryBus,
    /// The version of the state last loaded, 1 or 2; 0 before any.
    pub loaded_version: i32,
    pub options: Skip<Options>,
}

/// How the machine runs, none of which changes what it computes.
#[derive(Clone, Copy, Debug)]
pub struct Options {
    /// C#'s `Cpu.SkipIdle`: the idle loop passed in one piece.
    pub idle_skip: bool,
    /// The idle loop runs a running signal processor to its next event at once, as C#'s `RunBlocks` does.
    pub rsp_whole: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options { idle_skip: true, rsp_whole: true }
    }
}

impl Machine {
    /// A machine with 4 MB or 8 MB of RDRAM; any other size is refused, as `LoadState` refuses it.
    pub fn new(rdram_bytes: usize) -> Option<Machine> {
        if rdram_bytes != RDRAM_SIZE && rdram_bytes != RDRAM_SIZE_EXPANDED {
            return None;
        }
        Some(Machine {
            total_frames: 0,
            last_frame_cycles: CYCLE_CAP,
            cpu: Cpu::default(),
            bus: MemoryBus::new(rdram_bytes),
            loaded_version: 0,
            options: Skip(Options::default()),
        })
    }

    pub fn rdram_bytes(&self) -> usize {
        self.bus.rdram.len()
    }

    /// `MarsCore.Write`. A state cannot carry pending RDP words, since only running them would drain them.
    pub fn write_state(&self, w: &mut StateWriter, snapshot: bool) -> StateResult {
        let pending = self.bus.dp.pending.len();
        if pending > SNAPSHOT_WORDS {
            return Err(StateError::PendingOverflow(pending as i32));
        }
        if !snapshot && pending != 0 {
            return Err(StateError::PendingInState(pending));
        }
        w.u32("Magic", STATE_MAGIC);
        w.i32("Version", if snapshot { SNAPSHOT_VERSION } else { STATE_VERSION });
        w.i32("Rdram.Length", self.bus.rdram.len() as i32);
        w.i64("TotalFrames", self.total_frames);
        w.i64("_lastFrameCycles", self.last_frame_cycles);
        w.group("Cpu", |w| self.cpu.write_state(w));
        self.bus.write_state_all(w, snapshot);
        Ok(())
    }

    pub fn state_size(&self, snapshot: bool) -> StateResult<usize> {
        let mut w = StateWriter::counter();
        self.write_state(&mut w, snapshot)?;
        Ok(w.len())
    }

    /// Writes into `out`, which must hold `state_size` bytes; returns the bytes written.
    pub fn save_state(&self, out: &mut [u8], snapshot: bool) -> StateResult<usize> {
        let mut w = StateWriter::new(out);
        self.write_state(&mut w, snapshot)?;
        if w.overflowed() {
            return Err(StateError::BufferTooSmall { needed: w.len() });
        }
        Ok(w.len())
    }

    pub fn save_state_vec(&self, snapshot: bool) -> StateResult<Vec<u8>> {
        let mut out = vec![0; self.state_size(snapshot)?];
        self.save_state(&mut out, snapshot)?;
        Ok(out)
    }

    /// One line per field, `offset length type path`, in the order written; the C# side walks its own to compare.
    pub fn layout(&self, snapshot: bool) -> StateResult<String> {
        let mut w = StateWriter::layout();
        self.write_state(&mut w, snapshot)?;
        Ok(w.into_layout())
    }

    /// `MarsCore.LoadState`: a state of the other known RDRAM size rebuilds the machine to it. A failed load changes nothing.
    pub fn load_state(&mut self, data: &[u8]) -> StateResult {
        let mut r = StateReader::new(data);
        let magic = r.u32()?;
        if magic != STATE_MAGIC {
            return Err(StateError::NotAMarsState(magic));
        }
        let version = r.i32()?;
        if version != STATE_VERSION && version != SNAPSHOT_VERSION {
            return Err(StateError::Version(version));
        }
        let rdram = r.i32()?;
        let mut machine = usize::try_from(rdram).ok().and_then(Machine::new).ok_or(StateError::RdramSize(rdram))?;

        machine.total_frames = r.i64()?; // TotalFrames
        machine.last_frame_cycles = r.i64()?; // _lastFrameCycles
        machine.cpu.read_state(&mut r)?; // Cpu
        machine.bus.read_state_all(&mut r, version == SNAPSHOT_VERSION)?; // Bus
        machine.loaded_version = version;
        machine.options = self.options;
        machine.bus.vi_rebase();
        machine.bus.ai_rebase();

        *self = machine;
        Ok(())
    }
}

/// `Boot`: what the PIF would have done. See Mars_Boot.md §1.
pub mod boot {
    pub const BOOT_CODE_LENGTH: usize = 0x1000;
    pub const MEMORY_SIZE_AT: u32 = 0x318;
    pub const MEMORY_SIZE_AT_6105: u32 = 0x3F0;
    pub const ENTRY_POINT: u64 = 0xFFFF_FFFF_A400_0040;
    pub const STACK_POINTER: u64 = 0xFFFF_FFFF_A400_1FF0;
    pub const IPL2_RETURN_ADDRESS: u64 = 0xFFFF_FFFF_A400_1550;
    pub const IPL2_HEAD: [u32; 8] = [0x3C0DBFC0, 0x8DA807FC, 0x25AD07C0, 0x31080080, 0x5500FFFC, 0x3C0DBFC0, 0x8DA80024, 0x3C0BB000];
}

/// `Boot.HandOff`: the cartridge's boot code in DMEM, IPL2's head in IMEM, and the registers the boot code is entitled to.
pub fn hand_off(bus: &mut MemoryBus, cpu: &mut Cpu, rom: Arc<RomImage>) {
    let n = boot::BOOT_CODE_LENGTH.min(rom.rom.len());
    bus.sp_dmem[..n].copy_from_slice(&rom.rom[..n]);
    for (i, word) in boot::IPL2_HEAD.iter().enumerate() {
        bus.sp_imem[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    let size_at = if rom.cic == Cic::Nus6105 { boot::MEMORY_SIZE_AT_6105 } else { boot::MEMORY_SIZE_AT } as usize;
    let size = bus.rdram.len() as u32;
    bus.rdram[size_at..size_at + 4].copy_from_slice(&size.to_be_bytes());

    cpu.pc = boot::ENTRY_POINT;
    cpu.next_pc = boot::ENTRY_POINT + 4;
    cpu.gpr[11] = boot::ENTRY_POINT;
    cpu.gpr[29] = boot::STACK_POINTER;
    cpu.gpr[31] = boot::IPL2_RETURN_ADDRESS;
    cpu.gpr[20] = if rom.is_pal { 0 } else { 1 };
    cpu.gpr[22] = rom::seed(rom.cic) as u64;
    cpu.cop0[STATUS] = 0x3400_0000;
    cpu.cop0[COMPARE] = 0xFFFF_FFFF;
    cpu.cop0[PROCESSOR_ID_REGISTER] = PROCESSOR_ID;
    cpu.cop0[CONFIG] = CONFIG_AT_RESET;
    *bus.cart = Some(rom);
    cpu.cop0_written(bus);
}

impl Machine {
    /// The bus and processor as built, then `Boot.HandOff`; what the corpus runs, and what `load_rom` starts from.
    pub fn boot(rom: Arc<RomImage>, expansion_pak: bool) -> Machine {
        let mut bus = MemoryBus::new(if expansion_pak { RDRAM_SIZE_EXPANDED } else { RDRAM_SIZE });
        bus.sp.processor.halted = true;
        bus.si.controllers[0].present = true;
        let mut cpu = Cpu::power_on(&bus);
        hand_off(&mut bus, &mut cpu, rom);
        Machine { total_frames: 0, last_frame_cycles: CYCLE_CAP, cpu, bus, loaded_version: 0, options: Skip(Options::default()) }
    }

    /// `MarsCore.LoadRom` less the host: boot, then `LoadSaves` from the files the host read, or none.
    pub fn load_rom(rom: Arc<RomImage>, expansion_pak: bool, saved: Option<Vec<u8>>, pak: Option<Vec<u8>>) -> Machine {
        let mut machine = Machine::boot(rom.clone(), expansion_pak);
        let mut kind = rom.declared_save_type();
        if kind == save_type::UNKNOWN
            && let Some(saved) = &saved
        {
            kind = SaveChip::from_save_length(saved.len());
        }
        machine.bus.save = SaveChip::with_saved(kind, saved.map(Arc::new));
        machine.bus.si.controllers[0].pak = Some(ControllerPak::new(pak.as_deref()));
        machine
    }

    /// `Settle`, which a C# save runs before it writes.
    pub fn settle(&mut self) {
        self.bus.settle();
    }

    /// `RunFrame` without rendering or the debugger: to the VI's next field, or the cap; `RunQuietly` with the idle loop.
    pub fn run_frame(&mut self) {
        let start = self.bus.cycles;
        let fields = self.bus.vi.fields;
        let cap_at = start + CYCLE_CAP;
        let Options { idle_skip, rsp_whole } = *self.options;
        let (cpu, bus) = (&mut self.cpu, &mut self.bus);
        while bus.vi.fields == fields && bus.cycles < cap_at {
            if idle_skip && cpu.pc == cpu.run.idle_at && cpu.try_idle(bus, cap_at, rsp_whole) {
                continue;
            }
            cpu.step(bus);
        }
        self.last_frame_cycles = self.bus.cycles - start;
        self.total_frames += 1;
    }

    /// The interpreter alone, as the corpus runs it: `Cpu.Step`, the given number of times.
    pub fn run_steps(&mut self, steps: u64) {
        let (cpu, bus) = (&mut self.cpu, &mut self.bus);
        for _ in 0..steps {
            cpu.step(bus);
        }
    }

    /// `MarsCore.LoadState`: the state read, then everything C# derives after reading it. See Mars_Native.md §5.2.
    pub fn restore_state(&mut self, data: &[u8]) -> StateResult {
        let cart = self.bus.cart.clone();
        let saved = self.bus.save.saved.clone();
        self.load_state(data)?;
        self.bus.cart = cart;
        self.bus.save.saved = saved;
        self.derive_after_load();
        Ok(())
    }

    /// What `MemoryBus.ReadState` and `LoadState` run after the fields: marks, drops, rebases, replays, `Cop0Written`.
    fn derive_after_load(&mut self) {
        let bus = &mut self.bus;
        *bus.written += 1;
        bus.save.set_dirty(true);
        for port in &mut bus.si.controllers {
            if let Some(pak) = &mut port.pak {
                pak.dirty = true;
            }
        }
        bus.ai.drop_undrained();
        bus.vi_rebase();
        bus.ai_rebase();
        bus.reschedule();
        // C#'s Rdp.Refresh ran in the RDP's own read_state.
        bus.dp_replay_pending();
        self.cpu.run.idle_at = u64::MAX;
        self.cpu.cop0_written(&self.bus);
    }

    /// `SetButton`'s `Press`: one mask of the joybus's sixteen button bits.
    pub fn press(&mut self, port: usize, mask: u16, pressed: bool) {
        if let Some(c) = self.bus.si.controllers.get_mut(port) {
            c.buttons = if pressed { c.buttons | mask } else { c.buttons & !mask };
        }
    }

    /// `SetAxis`'s stick: 0 is X and 1 is Y, already scaled and turned as C# turns them.
    pub fn set_stick(&mut self, port: usize, axis: u32, value: i8) {
        if let Some(c) = self.bus.si.controllers.get_mut(port) {
            if axis == 0 {
                c.stick_x = value;
            } else {
                c.stick_y = value;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::ControllerPak;
    use crate::save::SaveChip;

    fn busy() -> Machine {
        let mut m = Machine::new(RDRAM_SIZE_EXPANDED).unwrap();
        m.total_frames = 1234;
        m.cpu.gpr[5] = 0xDEAD_BEEF;
        m.cpu.tlb.entries[31].page_mask = 0x1FFE000;
        m.bus.rdram[0x7F_FFFF] = 0x5A;
        m.bus.dp.processor.tiles[7].th = -9;
        m.bus.si.due = 0x1_2345_6789;
        m.bus.si.pending_read = 0x1000;
        m.bus.registers.insert(0x0450_0010, 7);
        m.bus.save = SaveChip::new(crate::save::save_type::FLASH_RAM);
        m.bus.si.controllers[2].pak = Some(ControllerPak::default());
        m
    }

    #[test]
    fn a_state_reads_back_into_the_machine_that_wrote_it() {
        for snapshot in [false, true] {
            let mut m = busy();
            if snapshot {
                m.bus.dp.pending = vec![1, 2, 3];
            }
            let bytes = m.save_state_vec(snapshot).unwrap();
            assert_eq!(bytes.len(), m.state_size(snapshot).unwrap());

            let mut loaded = Machine::new(RDRAM_SIZE).unwrap();
            loaded.load_state(&bytes).unwrap();
            m.loaded_version = if snapshot { SNAPSHOT_VERSION } else { STATE_VERSION };
            assert_eq!(loaded, m);
            assert_eq!(loaded.save_state_vec(snapshot).unwrap(), bytes);
        }
    }

    #[test]
    fn the_layout_covers_every_byte_once_in_order() {
        let m = busy();
        let layout = m.layout(true).unwrap();
        let mut next = 0usize;
        for line in layout.lines() {
            let mut parts = line.split(' ');
            let offset: usize = parts.next().unwrap().parse().unwrap();
            let length: usize = parts.next().unwrap().parse().unwrap();
            assert_eq!(offset, next, "{line}");
            next += length;
        }
        assert_eq!(next, m.state_size(true).unwrap());
    }

    /// No Mars state has one, since every class field is built with its owner; C# then reads none of its fields.
    #[test]
    fn a_class_whose_flag_is_clear_has_no_fields_in_the_stream() {
        let mut m = busy();
        m.bus.is_viewer.memory[9] = 0x77;
        let bytes = m.save_state_vec(false).unwrap();
        let layout = m.layout(false).unwrap();
        let find = |path: &str| -> (usize, usize) {
            let line = layout.lines().find(|l| l.ends_with(&format!(" {path}"))).unwrap();
            let mut parts = line.split(' ');
            (parts.next().unwrap().parse().unwrap(), parts.next().unwrap().parse().unwrap())
        };
        let (flag, _) = find("Bus.IsViewer");
        let (memory, length) = find("Bus.IsViewer._memory");
        let mut cut = bytes[..memory].to_vec();
        cut[flag] = 0;
        cut.extend_from_slice(&bytes[memory + length..]);

        let mut loaded = Machine::new(RDRAM_SIZE).unwrap();
        loaded.load_state(&cut).unwrap();
        m.bus.is_viewer = Default::default();
        m.loaded_version = STATE_VERSION;
        assert_eq!(loaded, m);
    }

    #[test]
    fn a_failed_load_leaves_the_machine_as_it_was() {
        let mut m = busy();
        let bytes = m.save_state_vec(false).unwrap();
        let before = m.clone();
        assert!(matches!(m.load_state(&bytes[..bytes.len() - 1]), Err(StateError::Truncated { .. })));
        assert_eq!(m, before);
        let mut wrong = bytes.clone();
        wrong[8] = 3;
        assert_eq!(m.load_state(&wrong), Err(StateError::RdramSize(i32::from_le_bytes([3, 0, 0x80, 0]))));
    }

    #[test]
    fn a_state_refuses_pending_words_and_a_snapshot_carries_them() {
        let mut m = busy();
        m.bus.dp.pending = vec![9];
        assert_eq!(m.state_size(false), Err(StateError::PendingInState(1)));
        assert_eq!(m.state_size(true).unwrap(), {
            m.bus.dp.pending.clear();
            m.state_size(false).unwrap() + 4 + 8 * SNAPSHOT_WORDS
        });
    }
}
