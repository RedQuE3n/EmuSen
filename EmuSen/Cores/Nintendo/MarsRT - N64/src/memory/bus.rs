//! The C# `MemoryBus`: every memory, the devices, and the hand-written tail of the state. See Mars_Native.md §5.1.

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::Skip;
use crate::memory::ram::Ram;
use crate::rom::RomImage;

use crate::memory::ai::AiInterface;
use crate::memory::controller::ControllerPak;
use crate::memory::dp::DpInterface;
use crate::memory::isviewer::IsViewer;
use crate::memory::mi::MiInterface;
use crate::memory::pi::PiInterface;
use crate::memory::save::SaveChip;
use crate::memory::si::SiInterface;
use crate::memory::sp::SpInterface;
use crate::state::{State, StateReader, StateResult, StateWriter, boxed};
use crate::vi::Vi;

/// `MemoryBus.RdramSize`.
pub const RDRAM_SIZE: usize = 0x0040_0000;
/// `MemoryBus.RdramSizeExpanded`, with the Expansion Pak.
pub const RDRAM_SIZE_EXPANDED: usize = 0x0080_0000;
/// `MemoryMap.SpMemSize`.
pub const SP_MEM_SIZE: usize = 0x1000;
/// `MemoryMap.RiSelect`, which the C# constructor sets to 0x14.
pub const RI_SELECT: u32 = 0x0470_000C;

/// The SI's transfer rides among the unmodelled registers at addresses no bus reaches (`SiDueLow`, `SiDueHigh`, `SiReadTo`).
pub const SI_DUE_LOW: u32 = 0xFFFF_FF00;
pub const SI_DUE_HIGH: u32 = 0xFFFF_FF04;
pub const SI_READ_TO: u32 = 0xFFFF_FF08;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemoryBus {
    pub ai: AiInterface,
    pub cycles: i64,
    pub dp: DpInterface,
    pub is_viewer: IsViewer,
    pub mi: MiInterface,
    pub pi: PiInterface,
    pub pif_ram: [u8; 64],
    /// `Rdram`: 4 MB, or 8 MB with the Expansion Pak.
    pub rdram: Ram,
    /// `RdramHidden`: the RDP's extra bits, one byte per 16-bit word.
    pub rdram_hidden: Ram,
    pub si: SiInterface,
    pub sp: SpInterface,
    pub sp_dmem: Box<[u8; SP_MEM_SIZE]>,
    pub sp_imem: Box<[u8; SP_MEM_SIZE]>,
    pub vi: Vi,
    pub count_bias: i64,
    /// `_registers`: the stub registers nobody models, written sorted by address; the SI's three keys live in `si`.
    pub registers: BTreeMap<u32, u32>,
    /// `Save`: `[SkipInState]` in C#, and written after the registers.
    pub save: SaveChip,
    /// `_nextEvent`: the earliest cycle the VI, AI or SI has something to do; zero until the first tick asks them.
    pub next_event: Skip<i64>,
    /// `Written`: every write to memory that is not the processor's own direct store, which ends an idle run.
    pub written: Skip<i64>,
    /// `Cart`: the cartridge, shared by every copy of the machine.
    pub cart: Skip<Option<Arc<RomImage>>>,
}

impl MemoryBus {
    pub fn new(rdram_bytes: usize) -> Self {
        MemoryBus {
            ai: AiInterface::default(),
            cycles: 0,
            dp: DpInterface::default(),
            is_viewer: IsViewer::default(),
            mi: MiInterface::default(),
            pi: PiInterface::default(),
            pif_ram: [0; 64],
            rdram: Ram::zeroed(rdram_bytes),
            rdram_hidden: Ram::zeroed(rdram_bytes / 2),
            si: SiInterface::default(),
            sp: SpInterface::default(),
            sp_dmem: boxed(0),
            sp_imem: boxed(0),
            vi: Vi::default(),
            count_bias: 0,
            registers: BTreeMap::from([(RI_SELECT, 0x14)]),
            save: SaveChip::default(),
            next_event: Skip(0),
            written: Skip(0),
            cart: Skip(None),
        }
    }

    /// `WriteStateBody`, then `WritePending` in a snapshot.
    pub fn write_state_all(&self, w: &mut StateWriter, snapshot: bool) {
        w.group("Bus", |w| {
            self.write_state(w);
            self.write_tail(w);
            if snapshot {
                self.dp.write_pending(w, &self.dp.pending_words());
            }
        });
    }

    /// `ReadState`, less everything it derives or runs after reading.
    pub fn read_state_all(&mut self, r: &mut StateReader, snapshot: bool) -> StateResult {
        self.read_state(r)?;
        self.read_tail(r)?;
        self.dp.pending.clear();
        if snapshot {
            self.dp.read_pending(r)?;
        }
        Ok(())
    }

    /// The registers with the SI's transfer encoded among them, as `WriteStateBody` leaves `_registers`.
    pub fn registers_with_si(&self) -> BTreeMap<u32, u32> {
        let mut registers = self.registers.clone();
        registers.remove(&SI_DUE_LOW);
        registers.remove(&SI_DUE_HIGH);
        registers.remove(&SI_READ_TO);
        if self.si.due != i64::MAX {
            registers.insert(SI_DUE_LOW, self.si.due as u32);
            registers.insert(SI_DUE_HIGH, (self.si.due >> 32) as u32);
        }
        if self.si.pending_read >= 0 {
            registers.insert(SI_READ_TO, self.si.pending_read as u32);
        }
        registers
    }

    fn write_tail(&self, w: &mut StateWriter) {
        let registers = self.registers_with_si();
        w.i32("_registers.Count", registers.len() as i32);
        for (i, (&address, &value)) in registers.iter().enumerate() {
            w.group(&format!("_registers[{i}]"), |w| {
                w.u32("Key", address);
                w.u32("Value", value);
            });
        }

        self.save.write_state(w);

        for (i, port) in self.si.controllers.iter().enumerate() {
            let name = format!("Si.Controllers[{i}].Pak");
            w.bool(&name, port.pak.is_some());
            if let Some(pak) = &port.pak {
                w.structure(&name, pak);
            }
        }
    }

    fn read_tail(&mut self, r: &mut StateReader) -> StateResult {
        self.registers.clear();
        let count = r.i32()?; // _registers.Count
        for _ in 0..count.max(0) {
            let address = r.u32()?;
            let value = r.u32()?;
            self.registers.insert(address, value);
        }

        let low = self.registers.remove(&SI_DUE_LOW);
        let high = self.registers.remove(&SI_DUE_HIGH);
        self.si.due = match (low, high) {
            (Some(low), Some(high)) => (((high as u64) << 32) | low as u64) as i64,
            _ => i64::MAX,
        };
        self.si.pending_read = self.registers.remove(&SI_READ_TO).map_or(-1, i64::from);

        self.save.read_state(r)?; // Save

        for port in &mut self.si.controllers {
            if !r.bool()? {
                port.pak = None;
                continue;
            }
            port.pak.get_or_insert_with(ControllerPak::default).read_state(r)?; // Controllers[i].Pak
        }
        Ok(())
    }
}

impl State for MemoryBus {
    fn write_state(&self, w: &mut StateWriter) {
        w.class("Ai", &self.ai);
        w.i64("Cycles", self.cycles);
        w.class("Dp", &self.dp);
        w.class("IsViewer", &self.is_viewer);
        w.class("Mi", &self.mi);
        w.class("Pi", &self.pi);
        w.bytes("PifRam", &self.pif_ram[..]);
        w.bytes("Rdram", &self.rdram);
        w.bytes("RdramHidden", &self.rdram_hidden);
        w.class("Si", &self.si);
        w.class("Sp", &self.sp);
        w.bytes("SpDmem", &self.sp_dmem[..]);
        w.bytes("SpImem", &self.sp_imem[..]);
        w.class("Vi", &self.vi);
        w.i64("_countBias", self.count_bias);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        r.class(&mut self.ai)?; // Ai
        self.cycles = r.i64()?; // Cycles
        r.class(&mut self.dp)?; // Dp
        r.class(&mut self.is_viewer)?; // IsViewer
        r.class(&mut self.mi)?; // Mi
        r.class(&mut self.pi)?; // Pi
        r.bytes(&mut self.pif_ram[..])?; // PifRam
        r.bytes(&mut self.rdram)?; // Rdram
        r.bytes(&mut self.rdram_hidden)?; // RdramHidden
        r.class(&mut self.si)?; // Si
        r.class(&mut self.sp)?; // Sp
        r.bytes(&mut self.sp_dmem[..])?; // SpDmem
        r.bytes(&mut self.sp_imem[..])?; // SpImem
        r.class(&mut self.vi)?; // Vi
        self.count_bias = r.i64()?; // _countBias
        Ok(())
    }
}
