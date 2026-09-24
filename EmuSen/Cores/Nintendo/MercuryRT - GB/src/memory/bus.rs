//! C#'s `Memory/MemoryBus.cs` and `MemoryBus.Cgb.cs`: everything at the far end of the CPU's pins. See Mercury_Memory.md §3.

use crate::Skip;
use crate::apu::Apu;
use crate::memory::cartridge::Cartridge;
use crate::memory::mappers::Mapper;
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
    /// Colour mode, a Game Boy Color running a cartridge made for it; decided once, as C#'s readonly `Cgb` is (Mercury_Model.md §2).
    pub cgb: Skip<bool>,
    /// The console itself, which stays a Game Boy Color while it runs a Game Boy cartridge.
    pub cgb_hardware: Skip<bool>,
    /// A Game Boy cartridge on a Game Boy Color: the colour registers locked and DMG rendering through colour palettes (Mercury_Model.md §4).
    pub dmg_compat: Skip<bool>,
    pub joypad: Skip<Joypad>,
    /// What the test corpus printed, for the harness to read - see Mercury_Memory.md §10.
    pub serial_log: Skip<Vec<u8>>,
    /// The cartridge and its board, reached through the bus as C#'s `_cart` is; walked by the machine, not here.
    pub cart: Cartridge,
    pub mapper: Mapper,
    /// Game Genie's reads, flattened by the shim from C#'s registry: per address, 0x100 | patched for each original byte a patch replaces.
    pub rom_patches: Skip<Option<Box<std::collections::HashMap<u16, [u16; 256]>>>>,
}

impl MemoryBus {
    /// `new MemoryBus(cart, cgbHardware)`: the colour console's banks are twice and four times the size, whatever it runs.
    pub fn new(cart: Cartridge, mapper: Mapper, cgb_hardware: bool) -> Self {
        let (cgb, compat) = (cgb_hardware && cart.is_cgb(), cgb_hardware && !cart.is_cgb());
        let mut ppu = Ppu::default();
        *ppu.compat = compat;
        MemoryBus {
            cart,
            mapper,
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
            ppu,
            speed_switch_armed: false,
            vram: vec![0; VRAM_BANK_SIZE * if cgb_hardware { 2 } else { 1 }],
            vram_bank: 0,
            wram: vec![0; WRAM_BANK_SIZE * if cgb_hardware { 8 } else { 2 }],
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
            cgb_hardware: Skip(cgb_hardware),
            dmg_compat: Skip(compat),
            joypad: Skip(Joypad::default()),
            serial_log: Skip(Vec::new()),
            rom_patches: Skip(None),
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
        if *self.dmg_compat {
            self.ppu.load_compatibility_palettes(crate::ppu::compat::palette_number(&self.cart.rom));
        }
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

    /// The bus's fields in C#'s ordinal order; version 6 no longer writes the cartridge's `_cart` copy (Mercury_Native.md §9.3).
    pub fn write_state(&self, w: &mut StateWriter) {
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

    pub fn read_state(&mut self, r: &mut StateReader) -> StateResult {
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
        self.cart.read_retired_copy(r)?; // _cart
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

const TIMER: u8 = 1 << 2;
const SERIAL: u8 = 1 << 3;

impl MemoryBus {
    #[inline(always)]
    fn vram_offset(&self, address: u16) -> usize {
        (self.vram_bank as usize).wrapping_mul(VRAM_BANK_SIZE).wrapping_add(address as usize - 0x8000)
    }

    /// $C000-$DFFF and its $E000 echo fold to the same 8K; the top half is the banked one.
    #[inline(always)]
    fn wram_offset(&self, address: u16) -> usize {
        let offset = address as usize & 0x1FFF;
        if offset < WRAM_BANK_SIZE { offset } else { (self.wram_bank as usize).wrapping_mul(WRAM_BANK_SIZE).wrapping_add(offset - WRAM_BANK_SIZE) }
    }

    pub fn oam_dma_active(&self) -> bool {
        self.oam_dma_cycles_left > 0
    }

    /// The CPU loses VRAM while the PPU draws from it, and OAM for the scan too and a whole OAM DMA - see Mercury_Ppu.md §7.
    #[inline(always)]
    fn vram_accessible(&self) -> bool {
        !self.ppu.lcd_enabled() || !self.ppu.is_mode(crate::ppu::PpuMode::Drawing)
    }

    #[inline(always)]
    fn oam_accessible(&self) -> bool {
        !self.oam_dma_active() && (!self.ppu.lcd_enabled() || !(self.ppu.is_mode(crate::ppu::PpuMode::OamScan) || self.ppu.is_mode(crate::ppu::PpuMode::Drawing)))
    }

    /// What a DMA charged the CPU, taken once and cleared - see Mercury_Cgb.md §4.1.
    pub fn take_pending_stall(&mut self) -> i32 {
        std::mem::take(&mut self.stall_cycles)
    }

    pub fn read(&mut self, address: u16) -> u8 {
        match address {
            ..0x8000 => {
                let value = self.mapper.read_rom(&self.cart, address);
                if let Some(patches) = &*self.rom_patches
                    && let Some(table) = patches.get(&address)
                    && table[value as usize] & 0x100 != 0
                {
                    return table[value as usize] as u8;
                }
                value
            }
            ..0xA000 => {
                if self.vram_accessible() {
                    self.vram[self.vram_offset(address)]
                } else {
                    0xFF
                }
            }
            ..0xC000 => self.mapper.read_ram(&self.cart, address),
            ..0xFE00 => self.wram[self.wram_offset(address)],
            ..0xFEA0 => {
                if self.oam_accessible() {
                    self.oam[(address - 0xFE00) as usize]
                } else {
                    0xFF
                }
            }
            ..0xFF00 => 0x00,
            ..0xFF80 => self.read_io(address),
            ..=0xFFFE => self.high_ram[(address - 0xFF80) as usize],
            _ => self.interrupt_enable,
        }
    }

    pub fn write(&mut self, address: u16, data: u8) {
        match address {
            ..0x8000 => self.mapper.write_rom(&self.cart, address, data),
            ..0xA000 => {
                if self.vram_accessible() {
                    let offset = self.vram_offset(address);
                    self.vram[offset] = data;
                }
            }
            ..0xC000 => self.mapper.write_ram(&mut self.cart, address, data),
            ..0xFE00 => {
                let offset = self.wram_offset(address);
                self.wram[offset] = data;
            }
            ..0xFEA0 => {
                if self.oam_accessible() {
                    self.oam[(address - 0xFE00) as usize] = data;
                }
            }
            ..0xFF00 => {}
            ..0xFF80 => self.write_io(address, data),
            ..=0xFFFE => self.high_ram[(address - 0xFF80) as usize] = data,
            _ => self.interrupt_enable = data,
        }
    }

    fn read_joypad(&self) -> u8 {
        let select = self.io[0x00];
        let j = &*self.joypad;
        let mut low = 0x0F;
        if select & 0x10 == 0 {
            low &= !((j.right as u8) | (j.left as u8) << 1 | (j.up as u8) << 2 | (j.down as u8) << 3);
        }
        if select & 0x20 == 0 {
            low &= !((j.a as u8) | (j.b as u8) << 1 | (j.select as u8) << 2 | (j.start as u8) << 3);
        }
        0xC0 | (select & 0x30) | (low & 0x0F)
    }

    fn read_io(&self, address: u16) -> u8 {
        match address {
            0xFF00 => self.read_joypad(),
            0xFF01 => self.serial_data,
            0xFF02 => self.serial_control | if *self.cgb { 0x7C } else { 0x7E },
            0xFF04 => (self.div_counter >> 8) as u8,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.tac | 0xF8,
            0xFF0F => self.interrupt_flags | 0xE0,
            0xFF10..=0xFF26 | 0xFF30..=0xFF3F => self.apu.read_register(address),
            0xFF40 => self.ppu.lcdc,
            0xFF41 => self.ppu.read_stat(),
            0xFF42 => self.ppu.scy,
            0xFF43 => self.ppu.scx,
            0xFF44 => self.ppu.ly,
            0xFF45 => self.ppu.lyc,
            0xFF47 => self.ppu.bgp,
            0xFF48 => self.ppu.obp0,
            0xFF49 => self.ppu.obp1,
            0xFF4A => self.ppu.wy,
            0xFF4B => self.ppu.wx,
            _ if *self.cgb => self.read_cgb_io(address),
            _ if *self.dmg_compat => self.read_compat_io(address),
            _ => self.io[(address - 0xFF00) as usize],
        }
    }

    fn write_io(&mut self, address: u16, data: u8) {
        match address {
            0xFF00 => self.io[0x00] = data & 0x30,
            0xFF01 => self.serial_data = data,
            0xFF02 => self.write_serial_control(data),
            0xFF04 => self.div_counter = 0,
            0xFF05 => {
                self.tima = data;
                self.tima_reload_delay = 0;
            }
            0xFF06 => self.tma = data,
            0xFF07 => self.tac = data & 0x07,
            0xFF0F => self.interrupt_flags = data & 0x1F,
            0xFF10..=0xFF26 | 0xFF30..=0xFF3F => self.apu.write_register(address, data),
            0xFF40 => self.ppu.write_lcdc(data, &mut self.interrupt_flags),
            0xFF41 => self.ppu.write_stat(data, &mut self.interrupt_flags),
            0xFF42 => self.ppu.scy = data,
            0xFF43 => self.ppu.scx = data,
            0xFF44 => {}
            0xFF45 => self.ppu.write_lyc(data, &mut self.interrupt_flags),
            0xFF46 => {
                self.io[0x46] = data;
                self.oam_dma_page = data;
                self.oam_dma_index = 0;
                self.oam_dma_cycles_left = 0xA0 * 4;
            }
            0xFF47 => self.ppu.bgp = data,
            0xFF48 => self.ppu.obp0 = data,
            0xFF49 => self.ppu.obp1 = data,
            0xFF4A => self.ppu.wy = data,
            0xFF4B => self.ppu.wx = data,
            _ => {
                if *self.cgb && self.write_cgb_io(address, data) {
                    return;
                }
                if *self.dmg_compat && self.write_compat_io(address, data) {
                    return;
                }
                self.io[(address - 0xFF00) as usize] = data;
            }
        }
    }

    /// A sink, not a link: an internally clocked transfer completes at once and shifts in $FF - see Mercury_Memory.md §10.
    fn write_serial_control(&mut self, data: u8) {
        self.serial_control = data & if *self.cgb { 0x83 } else { 0x81 };
        if self.serial_control & 0x81 != 0x81 {
            return;
        }
        self.serial_log.push(self.serial_data);
        self.serial_data = 0xFF;
        self.serial_control &= 0x7F;
        self.interrupt_flags |= SERIAL;
    }

    fn read_cgb_io(&self, address: u16) -> u8 {
        match address {
            0xFF4D => 0x7E | if self.double_speed { 0x80 } else { 0 } | self.speed_switch_armed as u8,
            0xFF4F => (0xFE | self.vram_bank) as u8,
            0xFF51 => (self.hdma_source >> 8) as u8,
            0xFF52 => (self.hdma_source & 0xF0) as u8,
            0xFF53 => ((self.hdma_destination >> 8) & 0x1F) as u8,
            0xFF54 => (self.hdma_destination & 0xF0) as u8,
            0xFF55 => {
                if self.hdma_blocks_left == 0 {
                    0xFF
                } else {
                    (self.hdma_blocks_left - 1) as u8
                }
            }
            0xFF68 => self.ppu.read_bg_palette_index(),
            0xFF69 => self.ppu.read_bg_palette_data(),
            0xFF6A => self.ppu.read_obj_palette_index(),
            0xFF6B => self.ppu.read_obj_palette_data(),
            0xFF70 => (0xF8 | self.wram_bank) as u8,
            _ => self.io[(address - 0xFF00) as usize],
        }
    }

    fn write_cgb_io(&mut self, address: u16, data: u8) -> bool {
        match address {
            0xFF4D => self.speed_switch_armed = data & 0x01 != 0,
            0xFF4F => self.vram_bank = (data & 0x01) as i32,
            0xFF51 => self.hdma_source = ((data as u16) << 8) | (self.hdma_source & 0xF0),
            0xFF52 => self.hdma_source = (self.hdma_source & 0xFF00) | (data & 0xF0) as u16,
            0xFF53 => self.hdma_destination = (((data & 0x1F) as u16) << 8) | (self.hdma_destination & 0xF0),
            0xFF54 => self.hdma_destination = (self.hdma_destination & 0x1F00) | (data & 0xF0) as u16,
            0xFF55 => self.start_hdma(data),
            0xFF68 => self.ppu.write_bg_palette_index(data),
            0xFF69 => self.ppu.write_bg_palette_data(data),
            0xFF6A => self.ppu.write_obj_palette_index(data),
            0xFF6B => self.ppu.write_obj_palette_data(data),
            0xFF6C => {}
            0xFF70 => self.wram_bank = if data & 0x07 == 0 { 1 } else { (data & 0x07) as i32 },
            _ => return false,
        }
        true
    }

    /// With a Game Boy cartridge the colour registers read $FF and ignore writes, except VRAM bank and the palette indices (Mercury_Model.md §4.2).
    fn read_compat_io(&self, address: u16) -> u8 {
        match address {
            0xFF4C | 0xFF4D | 0xFF56 | 0xFF6C | 0xFF70 | 0xFF51..=0xFF55 | 0xFF69 | 0xFF6B => 0xFF,
            0xFF4F => (0xFE | self.vram_bank) as u8,
            0xFF68 => self.ppu.read_bg_palette_index(),
            0xFF6A => self.ppu.read_obj_palette_index(),
            _ => self.io[(address - 0xFF00) as usize],
        }
    }

    fn write_compat_io(&mut self, address: u16, data: u8) -> bool {
        match address {
            0xFF4F => self.vram_bank = (data & 0x01) as i32,
            0xFF68 => self.ppu.write_bg_palette_index(data),
            0xFF6A => self.ppu.write_obj_palette_index(data),
            0xFF4C | 0xFF4D | 0xFF56 | 0xFF69 | 0xFF6B | 0xFF6C | 0xFF70 | 0xFF51..=0xFF55 => {}
            _ => return false,
        }
        true
    }

    /// STOP performs the switch KEY1 armed; it is not a stop at all on a CGB - see Mercury_Cgb.md §5.
    pub fn stop(&mut self) {
        if !*self.cgb || !self.speed_switch_armed {
            return;
        }
        self.double_speed = !self.double_speed;
        self.speed_switch_armed = false;
        self.base_clock_phase = false;
    }

    fn start_hdma(&mut self, data: u8) {
        if self.hdma_is_h_blank_driven && self.hdma_blocks_left > 0 && data & 0x80 == 0 {
            self.hdma_blocks_left = 0;
            self.hdma_is_h_blank_driven = false;
            return;
        }
        self.hdma_blocks_left = (data & 0x7F) as i32 + 1;
        self.hdma_is_h_blank_driven = data & 0x80 != 0;
        if self.hdma_is_h_blank_driven {
            return;
        }
        while self.hdma_blocks_left > 0 {
            self.transfer_hdma_block();
        }
    }

    fn on_h_blank_started(&mut self) {
        if self.hdma_is_h_blank_driven && self.hdma_blocks_left > 0 {
            self.transfer_hdma_block();
        }
    }

    /// Eight machine cycles a block, half as long in double speed - see Mercury_Cgb.md §4.1.
    fn transfer_hdma_block(&mut self) {
        self.stall_cycles += if self.double_speed { 16 } else { 32 };
        for _ in 0..16 {
            let value = self.read(self.hdma_source);
            let offset = (self.vram_bank as usize).wrapping_mul(VRAM_BANK_SIZE).wrapping_add((self.hdma_destination & 0x1FFF) as usize);
            self.vram[offset] = value;
            self.hdma_source = self.hdma_source.wrapping_add(1);
            self.hdma_destination = (self.hdma_destination.wrapping_add(1)) & 0x1FFF;
        }
        self.hdma_blocks_left -= 1;
        if self.hdma_blocks_left == 0 {
            self.hdma_is_h_blank_driven = false;
        }
    }

    /// The DMA controller drives the bus itself, so the PPU's CPU-side blocking does not apply to it.
    fn read_for_dma(&self, address: u16) -> u8 {
        match address {
            ..0x8000 => self.mapper.read_rom(&self.cart, address),
            ..0xA000 => self.vram[self.vram_offset(address)],
            ..0xC000 => self.mapper.read_ram(&self.cart, address),
            _ => self.wram[self.wram_offset(address & 0xDFFF)],
        }
    }

    /// 160 machine cycles, one OAM byte each - see Mercury_Memory.md §6.
    #[inline(always)]
    fn step_oam_dma(&mut self) {
        if self.oam_dma_cycles_left == 0 {
            return;
        }
        self.oam_dma_cycles_left -= 1;
        if self.oam_dma_cycles_left & 3 != 0 || self.oam_dma_index >= 0xA0 {
            return;
        }
        let address = ((self.oam_dma_page as u16) << 8).wrapping_add(self.oam_dma_index as u16);
        self.oam[self.oam_dma_index as usize] = self.read_for_dma(address);
        self.oam_dma_index += 1;
    }

    #[inline(always)]
    pub fn tick(&mut self, cycles: i32) {
        for _ in 0..cycles {
            self.step_one_cycle();
        }
        // The cartridge's clock has its own crystal: base-clock cycles, half the CPU's in double speed (Mercury_Native.md §9.2).
        self.mapper.tick(&self.cart, if self.double_speed { cycles >> 1 } else { cycles });
    }

    /// TIMA counts falling edges of one selected bit of the DIV counter - see Mercury_Memory.md §5.
    #[inline(always)]
    fn step_one_cycle(&mut self) {
        if self.double_speed {
            self.base_clock_phase = !self.base_clock_phase;
            if self.base_clock_phase {
                self.step_base_clock();
            }
        } else {
            self.step_base_clock();
        }

        self.step_oam_dma();
        self.div_counter = self.div_counter.wrapping_add(1);
        self.apu.on_div_bit(self.div_counter & if self.double_speed { 0x2000 } else { 0x1000 } != 0);

        if self.tima_reload_delay > 0 {
            self.tima_reload_delay -= 1;
            if self.tima_reload_delay == 0 {
                self.tima = self.tma;
                self.interrupt_flags |= TIMER;
            }
        }

        let mask: u16 = match self.tac & 0x03 {
            0 => 1 << 9,
            1 => 1 << 3,
            2 => 1 << 5,
            _ => 1 << 7,
        };
        let edge = self.div_counter & mask != 0 && self.tac & 0x04 != 0;
        if self.last_timer_edge && !edge {
            self.tima = self.tima.wrapping_add(1);
            if self.tima == 0 {
                self.tima_reload_delay = 4;
            }
        }
        self.last_timer_edge = edge;
    }

    #[inline(always)]
    fn step_base_clock(&mut self) {
        if self.ppu.tick(&mut self.interrupt_flags, &self.vram, &self.oam, *self.cgb) {
            self.on_h_blank_started();
        }
        self.apu.tick();
    }
}

impl crate::cpu::CpuBus for MemoryBus {
    #[inline(always)]
    fn read(&mut self, address: u16) -> u8 {
        MemoryBus::read(self, address)
    }

    #[inline(always)]
    fn write(&mut self, address: u16, data: u8) {
        MemoryBus::write(self, address, data)
    }

    #[inline(always)]
    fn tick(&mut self, cycles: i32) {
        MemoryBus::tick(self, cycles)
    }

    fn stop(&mut self) {
        MemoryBus::stop(self)
    }
}
