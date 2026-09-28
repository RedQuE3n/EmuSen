//! C#'s `MemoryBus`: the CPU's address decode, OAM DMA and the per-cycle clock of everything the CPU drives. See Moon_Memory.md §1.

use std::collections::HashMap;

use super::{Board, Controller};
use crate::Skip;
use crate::apu::Apu;
use crate::ppu::Ppu;
use crate::state::{StateReader, StateResult, StateWriter};

pub const RAM_SIZE: usize = 0x0800;
pub const OAM_DMA_REGISTER: u16 = 0x4014;
pub const OAM_DMA_CYCLES: i32 = 513;
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
}

/// `MemoryBus.Read` for everything but `$4015`, over the parts the APU's own step does not hold (Moon_Native.md §2.5).
#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn decode_read(
    address: u16,
    ram: &mut [u8; RAM_SIZE],
    open_bus: &mut u8,
    ppu: &mut Ppu,
    board: &mut Board,
    controller1: &mut Controller,
    controller2: &mut Controller,
    patches: &Option<Box<RomPatches>>,
) -> u8 {
    let value = if address < 0x2000 {
        ram[(address & 0x07FF) as usize]
    } else if address < 0x4000 {
        ppu.read_register((address & 0x07) as i32, board)
    } else if address == 0x4016 {
        (*open_bus & 0xE0) | controller1.read()
    } else if address == 0x4017 {
        (*open_bus & 0xE0) | controller2.read()
    } else if address < 0x4020 {
        *open_bus
    } else {
        let value = board.mapper.read_prg(&board.cart, address);
        match patches {
            Some(table) => match table.get(&address) {
                Some(entries) if entries[value as usize] != 0 => entries[value as usize] as u8,
                _ => value,
            },
            None => value,
        }
    };
    *open_bus = value;
    value
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
        }
    }

    /// One CPU cycle of everything clocked from the CPU; returns the NMI level after the PPU and the IRQ level at the end, in C#'s order.
    #[inline(always)]
    pub fn tick(&mut self) -> (bool, bool) {
        self.ppu.step(DOTS_PER_CPU_CYCLE, &mut self.board);
        let nmi = self.ppu.nmi_output();

        let (ram, open_bus, ppu, board, c1, c2, patches) =
            (&mut self.ram, &mut self.open_bus, &mut *self.ppu, &mut *self.board, &mut *self.controller1, &mut *self.controller2, &*self.rom_patches);
        let mut read = |a: u16| decode_read(a, ram, open_bus, ppu, board, c1, c2, patches);
        self.apu.step(1, &mut read);
        if *self.mapper_clocks_on_cpu {
            self.board.mapper.on_cpu_cycle();
        }
        if self.apu.dmc.stall_cycles > 0 {
            let stolen = self.apu.dmc.stall_cycles;
            self.apu.dmc.stall_cycles = 0;
            self.stolen_cycles = self.stolen_cycles.wrapping_add(stolen);
            let (ram, open_bus, ppu, board, c1, c2, patches) =
                (&mut self.ram, &mut self.open_bus, &mut *self.ppu, &mut *self.board, &mut *self.controller1, &mut *self.controller2, &*self.rom_patches);
            let mut read = |a: u16| decode_read(a, ram, open_bus, ppu, board, c1, c2, patches);
            self.apu.step(stolen, &mut read);
        }
        (nmi, self.irq_level())
    }

    #[inline(always)]
    pub fn irq_level(&self) -> bool {
        self.apu.irq_asserted() || self.board.mapper.irq_pending()
    }

    #[inline(always)]
    pub fn read(&mut self, address: u16) -> u8 {
        if address == 0x4015 {
            let value = self.apu.read_status();
            self.open_bus = value;
            return value;
        }
        decode_read(
            address,
            &mut self.ram,
            &mut self.open_bus,
            &mut self.ppu,
            &mut self.board,
            &mut self.controller1,
            &mut self.controller2,
            &self.rom_patches,
        )
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
            return Some(self.run_oam_dma(data));
        }
        if address == 0x4016 {
            let strobe = (data & 0x01) != 0;
            self.controller1.set_strobe(strobe);
            self.controller2.set_strobe(strobe);
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

    /// The copy happens at once through the normal decode; the APU lives through its cycles, the PPU does not (Moon_Native.md §6.1).
    fn run_oam_dma(&mut self, page: u8) -> bool {
        let source = (page as u16) << 8;
        for i in 0..256u16 {
            let value = self.read(source.wrapping_add(i));
            let slot = self.ppu.oam_address.wrapping_add(i as u8);
            self.ppu.oam[slot as usize] = value;
        }
        self.pending_dma_cycles = self.pending_dma_cycles.wrapping_add(OAM_DMA_CYCLES);
        let (ram, open_bus, ppu, board, c1, c2, patches) =
            (&mut self.ram, &mut self.open_bus, &mut *self.ppu, &mut *self.board, &mut *self.controller1, &mut *self.controller2, &*self.rom_patches);
        let mut read = |a: u16| decode_read(a, ram, open_bus, ppu, board, c1, c2, patches);
        self.apu.step(OAM_DMA_CYCLES, &mut read);
        self.irq_level()
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
