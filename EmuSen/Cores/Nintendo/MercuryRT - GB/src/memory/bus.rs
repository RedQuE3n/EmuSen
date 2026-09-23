//! C#'s `Memory/MemoryBus.cs` and `MemoryBus.Cgb.cs`: everything at the far end of the CPU's pins. See Mercury_Memory.md §3.

use crate::Skip;
use crate::apu::Apu;
use crate::memory::cartridge::Cartridge;
use crate::ppu::Ppu;
use crate::state::{StateReader, StateResult, StateWriter};

pub const WRAM_BANK_SIZE: usize = 0x1000;
pub const VRAM_BANK_SIZE: usize = 0x2000;

/// The eight buttons behind `$FF00`; not in the state, so held buttons survive a load.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Joypad {
    pub right: bool,
    pub left: bool,
    pub up: bool,
    pub down: bool,
    pub a: bool,
    pub b: bool,
    pub select: bool,
    pub start: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MemoryBus {
    pub apu: Apu,
    pub double_speed: bool,
    pub hdma_blocks_left: i32,
    pub hdma_destination: u16,
    pub hdma_is_h_blank_driven: bool,
    pub hdma_source: u16,
    pub high_ram: [u8; 0x7F],
    pub interrupt_enable: u8,
    pub interrupt_flags: u8,
    pub io: [u8; 0x80],
    pub oam: [u8; 0xA0],
    pub ppu: Ppu,
    pub speed_switch_armed: bool,
    pub vram: Vec<u8>,
    pub vram_bank: i32,
    pub wram: Vec<u8>,
    pub wram_bank: i32,
    pub base_clock_phase: bool,
    pub div_counter: u16,
    pub last_timer_edge: bool,
    pub oam_dma_cycles_left: i32,
    pub oam_dma_index: i32,
    pub oam_dma_page: u8,
    pub serial_control: u8,
    pub serial_data: u8,
    pub stall_cycles: i32,
    pub tac: u8,
    pub tima: u8,
    pub tima_reload_delay: i32,
    pub tma: u8,
    /// Decided once from the header, as C#'s readonly `Cgb` is.
    pub cgb: Skip<bool>,
    pub joypad: Skip<Joypad>,
    /// What the test corpus printed, for the harness to read - see Mercury_Memory.md §10.
    pub serial_log: Skip<Vec<u8>>,
}

impl MemoryBus {
    /// `new MemoryBus(cart)`: the colour console's banks are twice and four times the size.
    pub fn new(cgb: bool) -> Self {
        MemoryBus {
            apu: Apu::default(),
            double_speed: false,
            hdma_blocks_left: 0,
            hdma_destination: 0,
            hdma_is_h_blank_driven: false,
            hdma_source: 0,
            high_ram: [0; 0x7F],
            interrupt_enable: 0,
            interrupt_flags: 0,
            io: [0; 0x80],
            oam: [0; 0xA0],
            ppu: Ppu::default(),
            speed_switch_armed: false,
            vram: vec![0; VRAM_BANK_SIZE * if cgb { 2 } else { 1 }],
            vram_bank: 0,
            wram: vec![0; WRAM_BANK_SIZE * if cgb { 8 } else { 2 }],
            wram_bank: 1,
            base_clock_phase: false,
            div_counter: 0,
            last_timer_edge: false,
            oam_dma_cycles_left: 0,
            oam_dma_index: 0,
            oam_dma_page: 0,
            serial_control: 0,
            serial_data: 0,
            stall_cycles: 0,
            tac: 0,
            tima: 0,
            tima_reload_delay: 0,
            tma: 0,
            cgb: Skip(cgb),
            joypad: Skip(Joypad::default()),
            serial_log: Skip(Vec::new()),
        }
    }

    /// `MemoryBus.Reset`, with `ResetCgb`, the PPU's and the APU's.
    pub fn reset(&mut self) {
        self.vram.fill(0);
        self.wram.fill(0);
        self.oam = [0; 0xA0];
        self.high_ram = [0; 0x7F];
        self.io = [0; 0x80];
        self.interrupt_enable = 0;
        self.interrupt_flags = 0;
        self.serial_data = 0;
        self.serial_control = 0;
        self.serial_log.clear();
        self.oam_dma_page = 0;
        self.oam_dma_cycles_left = 0;
        self.oam_dma_index = 0;
        self.stall_cycles = 0;
        self.div_counter = 0;
        self.tima = 0;
        self.tma = 0;
        self.tac = 0;
        self.last_timer_edge = false;
        self.tima_reload_delay = 0;
        self.io[0x00] = 0x30;
        self.reset_cgb();
        self.ppu.reset();
        self.apu.reset();
    }

    fn reset_cgb(&mut self) {
        self.vram_bank = 0;
        self.wram_bank = 1;
        self.double_speed = false;
        self.speed_switch_armed = false;
        self.base_clock_phase = false;
        self.hdma_source = 0;
        self.hdma_destination = 0;
        self.hdma_blocks_left = 0;
        self.hdma_is_h_blank_driven = false;
    }

    /// The bus's fields in C#'s ordinal order, the cartridge's `_cart` copy among them.
    pub fn write_state(&self, w: &mut StateWriter, cart: &Cartridge) {
        w.class("Apu", &self.apu);
        w.bool("DoubleSpeed", self.double_speed);
        w.i32("HdmaBlocksLeft", self.hdma_blocks_left);
        w.u16("HdmaDestination", self.hdma_destination);
        w.bool("HdmaIsHBlankDriven", self.hdma_is_h_blank_driven);
        w.u16("HdmaSource", self.hdma_source);
        w.bytes("HighRam", &self.high_ram);
        w.u8("InterruptEnable", self.interrupt_enable);
        w.u8("InterruptFlags", self.interrupt_flags);
        w.bytes("Io", &self.io);
        w.bytes("Oam", &self.oam);
        w.class("Ppu", &self.ppu);
        w.bool("SpeedSwitchArmed", self.speed_switch_armed);
        w.bytes("Vram", &self.vram);
        w.i32("VramBank", self.vram_bank);
        w.bytes("Wram", &self.wram);
        w.i32("WramBank", self.wram_bank);
        w.bool("_baseClockPhase", self.base_clock_phase);
        cart.write_as_field(w, "_cart");
        w.u16("_divCounter", self.div_counter);
        w.bool("_lastTimerEdge", self.last_timer_edge);
        w.i32("_oamDmaCyclesLeft", self.oam_dma_cycles_left);
        w.i32("_oamDmaIndex", self.oam_dma_index);
        w.u8("_oamDmaPage", self.oam_dma_page);
        w.u8("_serialControl", self.serial_control);
        w.u8("_serialData", self.serial_data);
        w.i32("_stallCycles", self.stall_cycles);
        w.u8("_tac", self.tac);
        w.u8("_tima", self.tima);
        w.i32("_timaReloadDelay", self.tima_reload_delay);
        w.u8("_tma", self.tma);
    }

    pub fn read_state(&mut self, r: &mut StateReader, cart: &mut Cartridge) -> StateResult {
        r.class(&mut self.apu)?; // Apu
        self.double_speed = r.bool()?; // DoubleSpeed
        self.hdma_blocks_left = r.i32()?; // HdmaBlocksLeft
        self.hdma_destination = r.u16()?; // HdmaDestination
        self.hdma_is_h_blank_driven = r.bool()?; // HdmaIsHBlankDriven
        self.hdma_source = r.u16()?; // HdmaSource
        r.bytes(&mut self.high_ram)?; // HighRam
        self.interrupt_enable = r.u8()?; // InterruptEnable
        self.interrupt_flags = r.u8()?; // InterruptFlags
        r.bytes(&mut self.io)?; // Io
        r.bytes(&mut self.oam)?; // Oam
        r.class(&mut self.ppu)?; // Ppu
        self.speed_switch_armed = r.bool()?; // SpeedSwitchArmed
        r.bytes(&mut self.vram)?; // Vram
        self.vram_bank = r.i32()?; // VramBank
        r.bytes(&mut self.wram)?; // Wram
        self.wram_bank = r.i32()?; // WramBank
        self.base_clock_phase = r.bool()?; // _baseClockPhase
        cart.read_as_field(r)?; // _cart
        self.div_counter = r.u16()?; // _divCounter
        self.last_timer_edge = r.bool()?; // _lastTimerEdge
        self.oam_dma_cycles_left = r.i32()?; // _oamDmaCyclesLeft
        self.oam_dma_index = r.i32()?; // _oamDmaIndex
        self.oam_dma_page = r.u8()?; // _oamDmaPage
        self.serial_control = r.u8()?; // _serialControl
        self.serial_data = r.u8()?; // _serialData
        self.stall_cycles = r.i32()?; // _stallCycles
        self.tac = r.u8()?; // _tac
        self.tima = r.u8()?; // _tima
        self.tima_reload_delay = r.i32()?; // _timaReloadDelay
        self.tma = r.u8()?; // _tma
        Ok(())
    }
}
