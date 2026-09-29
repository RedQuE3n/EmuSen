//! C#'s `MemoryBus`: the CPU's address decode, OAM DMA and the per-cycle clock of everything the CPU drives. See Moon_Memory.md §1.

use std::collections::HashMap;

use super::{Board, Controller};
use crate::Skip;
use crate::apu::Apu;
use crate::ppu::Ppu;
use crate::state::{StateReader, StateResult, StateWriter};

pub const RAM_SIZE: usize = 0x0800;
pub const OAM_DMA_REGISTER: u16 = 0x4014;
pub const DOTS_PER_CPU_CYCLE: i32 = 3;

/// Game Genie's table: per patched CPU address, 256 entries of `0x100 | patched` or 0, one per original byte.
pub type RomPatches = HashMap<u16, Box<[u16; 256]>>;

#[derive(Clone, Debug, PartialEq)]
pub struct MemoryBus {
    pub open_bus: u8,
    pub pending_dma_cycles: i32,
    pub ram: [u8; RAM_SIZE],
    pub stolen_cycles: i32,

    pub board: Skip<Board>,
    pub ppu: Skip<Ppu>,
    pub apu: Skip<Apu>,
    pub controller1: Skip<Controller>,
    pub controller2: Skip<Controller>,
    pub mapper_clocks_on_cpu: Skip<bool>,
    pub rom_patches: Skip<Option<Box<RomPatches>>>,
    pub oam_dma_pending: Skip<bool>,
    pub oam_dma_page: Skip<u8>,
    pub last_read_cycle: Skip<i64>,
    pub last_read_address: Skip<u16>,
    pub strobe_latch: Skip<bool>,
    pub strobe_out: Skip<bool>,
    /// C#'s `Cpu.Cycles`, which the CPU stamps here at each cycle's start.
    pub cpu_cycles: Skip<i64>,
}

impl MemoryBus {
    pub fn new(board: Board, ppu: Ppu, apu: Apu) -> Self {
        let clocks = board.mapper.clocks_on_cpu_cycle();
        MemoryBus {
            open_bus: 0,
            pending_dma_cycles: 0,
            ram: [0; RAM_SIZE],
            stolen_cycles: 0,
            board: Skip(board),
            ppu: Skip(ppu),
            apu: Skip(apu),
            controller1: Skip(Controller::default()),
            controller2: Skip(Controller::default()),
            mapper_clocks_on_cpu: Skip(clocks),
            rom_patches: Skip(None),
            oam_dma_pending: Skip(false),
            oam_dma_page: Skip(0),
            last_read_cycle: Skip(-2),
            last_read_address: Skip(0),
            strobe_latch: Skip(false),
            strobe_out: Skip(false),
            cpu_cycles: Skip(0),
        }
    }

    /// One CPU cycle of everything clocked from the CPU; returns the NMI level after the PPU and the IRQ level at the end, in C#'s order.
    #[inline(always)]
    pub fn tick(&mut self) -> (bool, bool) {
        self.ppu.step(DOTS_PER_CPU_CYCLE, &mut self.board);
        let nmi = self.ppu.nmi_output();
        self.apu.step(1);
        if *self.mapper_clocks_on_cpu {
            self.board.mapper.on_cpu_cycle();
        }
        if *self.strobe_latch != *self.strobe_out && !self.apu.is_get_cycle() {
            *self.strobe_out = *self.strobe_latch;
            self.controller1.set_strobe(*self.strobe_out);
            self.controller2.set_strobe(*self.strobe_out);
        }
        (nmi, self.irq_level())
    }

    #[inline(always)]
    pub fn irq_level(&self) -> bool {
        self.apu.irq_asserted() || self.board.mapper.irq_pending()
    }

    /// `MemoryBus.Read`.
    #[inline(always)]
    pub fn read(&mut self, address: u16) -> u8 {
        let value = if address < 0x2000 {
            self.ram[(address & 0x07FF) as usize]
        } else if address < 0x4000 {
            self.ppu.read_register((address & 0x07) as i32, &mut self.board)
        } else if address == 0x4015 {
            return self.apu.read_status() | (self.open_bus & 0x20);
        } else if address == 0x4016 {
            let clock = !self.held_read(address);
            (self.open_bus & 0xE0) | self.controller1.read(clock)
        } else if address == 0x4017 {
            let clock = !self.held_read(address);
            (self.open_bus & 0xE0) | self.controller2.read(clock)
        } else if address < 0x6000 {
            self.open_bus
        } else {
            let value = self.board.mapper.read_prg(&self.board.cart, address);
            match &*self.rom_patches {
                Some(table) => match table.get(&address) {
                    Some(entries) if entries[value as usize] != 0 => entries[value as usize] as u8,
                    _ => value,
                },
                None => value,
            }
        };
        self.open_bus = value;
        value
    }

    /// `HeldRead`: a pad clocks once for reads of it on consecutive cycles.
    fn held_read(&mut self, address: u16) -> bool {
        let cycle = *self.cpu_cycles;
        let held = address == *self.last_read_address && cycle == self.last_read_cycle.wrapping_add(1);
        *self.last_read_address = address;
        *self.last_read_cycle = cycle;
        held
    }

    /// `DmaPending`.
    #[inline(always)]
    pub fn dma_pending(&self) -> bool {
        *self.oam_dma_pending || self.apu.dmc.dma_requested()
    }

    /// `ConflictWithApuRegister`: a CPU halted on $4000-$401F keeps the 2A03's registers enabled for the fetch's low five bits.
    pub fn conflict_with_apu_register(&mut self, fetch: u16) {
        let register = 0x4000 | (fetch & 0x1F);
        if register == 0x4015 {
            self.apu.read_status();
        }
        if register == 0x4016 || register == 0x4017 {
            *self.last_read_address = register;
            *self.last_read_cycle = *self.cpu_cycles;
        }
    }

    pub fn forget_last_read(&mut self) {
        *self.last_read_cycle = -2;
    }

    /// `MemoryBus.Write`; `cpu_cycles` is what C# stamps from `Cpu.Cycles`. Returns the IRQ level when an OAM DMA set it.
    #[inline(always)]
    pub fn write(&mut self, address: u16, data: u8, cpu_cycles: i64) -> Option<bool> {
        self.open_bus = data;
        if address < 0x2000 {
            self.ram[(address & 0x07FF) as usize] = data;
            return None;
        }
        if address < 0x4000 {
            self.ppu.write_register((address & 0x07) as i32, data, &mut self.board);
            return None;
        }
        if address == OAM_DMA_REGISTER {
            *self.oam_dma_page = data;
            *self.oam_dma_pending = true;
            return None;
        }
        if address == 0x4016 {
            *self.strobe_latch = (data & 0x01) != 0;
            return None;
        }
        if address < 0x4020 {
            self.apu.write_register(address, data);
            return None;
        }
        let board = &mut *self.board;
        board.cart.cpu_cycle = cpu_cycles;
        board.mapper.write_prg(&mut board.cart, address, data);
        None
    }

    #[inline(always)]
    pub fn take_pending_dma_cycles(&mut self) -> i32 {
        std::mem::take(&mut self.pending_dma_cycles)
    }

    #[inline(always)]
    pub fn take_stolen_cycles(&mut self) -> i32 {
        std::mem::take(&mut self.stolen_cycles)
    }

    pub fn reset(&mut self) {
        self.ram.fill(0);
        self.soft_reset();
    }

    /// The RESET line leaves work RAM alone - see Moon_Core.md §6.
    pub fn soft_reset(&mut self) {
        self.pending_dma_cycles = 0;
        *self.oam_dma_pending = false;
        *self.last_read_cycle = -2;
        *self.strobe_latch = false;
        *self.strobe_out = false;
        self.open_bus = 0;
        self.controller1.reset();
        self.controller2.reset();
    }

    pub fn write_state(&self, w: &mut StateWriter) {
        w.u8("OpenBus", self.open_bus);
        w.i32("PendingDmaCycles", self.pending_dma_cycles);
        w.bytes("Ram", &self.ram);
        w.i32("StolenCycles", self.stolen_cycles);
    }

    pub fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        self.open_bus = r.u8()?; // OpenBus
        self.pending_dma_cycles = r.i32()?; // PendingDmaCycles
        r.bytes(&mut self.ram)?; // Ram
        self.stolen_cycles = r.i32()?; // StolenCycles
        Ok(())
    }
}
