//! The C# `MemoryBus`'s behaviour: every read and write by region and width, the clock, and the events. See Mars_Memory.md §2.

use crate::bus::{MemoryBus, RDRAM_SIZE_EXPANDED};

/// `MemoryMap`: the physical address map.
pub mod map {
    pub const RDRAM_REGISTERS_BASE: u32 = 0x03F0_0000;
    pub const RDRAM_REGISTERS_SIZE: u32 = 0x0010_0000;
    pub const SP_DMEM_BASE: u32 = 0x0400_0000;
    pub const SP_REGISTERS_BASE: u32 = 0x0404_0000;
    pub const SP_PC_BASE: u32 = 0x0408_0000;
    pub const DP_COMMAND_BASE: u32 = 0x0410_0000;
    pub const MI_BASE: u32 = 0x0430_0000;
    pub const VI_BASE: u32 = 0x0440_0000;
    pub const AI_BASE: u32 = 0x0450_0000;
    pub const PI_BASE: u32 = 0x0460_0000;
    pub const SI_BASE: u32 = 0x0480_0000;
    pub const CART_DOMAIN2_ADDRESS1: u32 = 0x0500_0000;
    pub const CART_DOMAIN2_ADDRESS2: u32 = 0x0800_0000;
    pub const CART_DOMAIN1_ADDRESS2: u32 = 0x1000_0000;
    pub const IS_VIEWER_BASE: u32 = 0x13FF_0000;
    pub const IS_VIEWER_SIZE: u32 = 0x1000;
    pub const PIF_ROM_BASE: u32 = 0x1FC0_0000;
    pub const PIF_RAM_BASE: u32 = 0x1FC0_07C0;
    pub const PIF_RAM_SIZE: u32 = 64;
    pub const SP_MEM_SIZE: u32 = 0x1000;
    pub const SP_MEM_WINDOW: u32 = SP_REGISTERS_BASE - SP_DMEM_BASE;
}

use map::*;

/// `RdramRegisters`: what they read once IPL3 has set RDRAM up, every 64 bytes.
const RDRAM_REGISTERS: [u32; 16] = [0xB419_0010, 0, 0x2B3B_1A0B, 0, 0, 0, 0x101C_0A04, 0, 0, 0, 0, 0, 0, 0, 0, 0];

/// `RepeatRowMask`: the MI's repeat stays inside one 2KB RDRAM row.
const REPEAT_ROW_MASK: u32 = 0x7FF;

#[inline(always)]
pub fn be32(memory: &[u8], offset: u32) -> u32 {
    let at = offset as usize;
    u32::from_be_bytes([memory[at], memory[at + 1], memory[at + 2], memory[at + 3]])
}

#[inline(always)]
pub fn put_be32(memory: &mut [u8], offset: u32, value: u32) {
    let at = offset as usize;
    memory[at..at + 4].copy_from_slice(&value.to_be_bytes());
}

#[inline(always)]
fn in_range(address: u32, start: u32, length: u32) -> bool {
    address >= start && address < start.wrapping_add(length)
}

/// `SignalProcessorMemory`: the two memories repeat every eight kilobytes up to the interface registers; true for IMEM.
#[inline(always)]
fn sp_memory(physical: u32) -> Option<(bool, u32)> {
    let local = physical.wrapping_sub(SP_DMEM_BASE);
    let imem = local % (2 * SP_MEM_SIZE) >= SP_MEM_SIZE;
    if physical >= SP_DMEM_BASE && local < SP_MEM_WINDOW { Some((imem, local % SP_MEM_SIZE)) } else { None }
}

#[inline(always)]
fn save_window(physical: u32) -> bool {
    (CART_DOMAIN2_ADDRESS2..CART_DOMAIN1_ADDRESS2).contains(&physical)
}

/// `CartridgeBus`: every store on the cartridge's bus is kept, the debug port aside.
#[inline(always)]
fn cartridge_bus(physical: u32) -> bool {
    (CART_DOMAIN2_ADDRESS1..PIF_ROM_BASE).contains(&physical) && !in_range(physical, IS_VIEWER_BASE, IS_VIEWER_SIZE)
}

#[inline(always)]
fn cartridge_rom(physical: u32) -> bool {
    physical >= CART_DOMAIN1_ADDRESS2 && cartridge_bus(physical)
}

#[inline(always)]
fn doubled(word: u32) -> u64 {
    ((word as u64) << 32) | word as u64
}

#[inline(always)]
fn lane(word: u32, physical: u32, size: u32) -> u64 {
    match size {
        1 => ((word >> ((3 - (physical & 3)) * 8)) & 0xFF) as u64,
        2 => ((word >> ((2 - (physical & 2)) * 8)) & 0xFFFF) as u64,
        _ => word as u64,
    }
}

/// `WholeWord`: the register shifted to its lane, or a doubleword's upper half.
#[inline(always)]
fn whole_word(physical: u32, value: u64, size: u32) -> u32 {
    if size == 8 {
        (value >> 32) as u32
    } else {
        value.wrapping_shl(8 * (4i32 - size as i32 - (physical & 3) as i32) as u32) as u32
    }
}

impl MemoryBus {
    pub fn read32(&mut self, physical: u32) -> u32 {
        if (physical as usize) < self.rdram.len() {
            return be32(&self.rdram, physical);
        }
        if physical < RDRAM_SIZE_EXPANDED as u32 {
            return 0;
        }
        if in_range(physical, RDRAM_REGISTERS_BASE, RDRAM_REGISTERS_SIZE) {
            return RDRAM_REGISTERS[((physical >> 2) & 0xF) as usize];
        }
        if let Some((imem, offset)) = sp_memory(physical) {
            return be32(if imem { &self.sp_imem[..] } else { &self.sp_dmem[..] }, offset);
        }
        if in_range(physical, IS_VIEWER_BASE, IS_VIEWER_SIZE) {
            return self.is_viewer.read32(physical - IS_VIEWER_BASE);
        }
        if in_range(physical, PIF_RAM_BASE, PIF_RAM_SIZE) {
            return be32(&self.pif_ram, physical - PIF_RAM_BASE);
        }
        if physical >= CART_DOMAIN1_ADDRESS2 {
            return self.read_cart32(physical - CART_DOMAIN1_ADDRESS2);
        }
        if save_window(physical) {
            return self.save.read32(physical - CART_DOMAIN2_ADDRESS2);
        }
        if in_range(physical, SP_REGISTERS_BASE, 0x20) {
            return self.sp_read32(physical - SP_REGISTERS_BASE);
        }
        if in_range(physical, SP_PC_BASE, 0x08) {
            return self.sp.processor.pc;
        }
        if in_range(physical, DP_COMMAND_BASE, 0x20) {
            return self.dp.read32(physical - DP_COMMAND_BASE);
        }
        if in_range(physical, PI_BASE, 0x34) {
            return self.pi_read32(physical - PI_BASE);
        }
        if in_range(physical, MI_BASE, 0x10) {
            return self.mi.read32(physical - MI_BASE);
        }
        if in_range(physical, VI_BASE, 0x38) {
            return self.vi.read32(physical - VI_BASE);
        }
        if in_range(physical, SI_BASE, 0x1C) {
            return self.si_read32(physical - SI_BASE);
        }
        if in_range(physical, AI_BASE, 0x18) {
            return self.ai.read32(physical - AI_BASE);
        }
        self.registers.get(&physical).copied().unwrap_or(0)
    }

    pub fn write32(&mut self, physical: u32, value: u32) {
        *self.written += 1;
        if (physical as usize) < self.rdram.len() {
            put_be32(&mut self.rdram, physical, value);
            return;
        }
        if physical < RDRAM_SIZE_EXPANDED as u32 {
            return;
        }
        if in_range(physical, RDRAM_REGISTERS_BASE, RDRAM_REGISTERS_SIZE) {
            return;
        }
        if let Some((imem, offset)) = sp_memory(physical) {
            put_be32(if imem { &mut self.sp_imem[..] } else { &mut self.sp_dmem[..] }, offset, value);
            return;
        }
        if in_range(physical, IS_VIEWER_BASE, IS_VIEWER_SIZE) {
            self.is_viewer.write32(physical - IS_VIEWER_BASE, value);
            return;
        }
        if in_range(physical, PIF_RAM_BASE, PIF_RAM_SIZE) {
            put_be32(&mut self.pif_ram, physical - PIF_RAM_BASE, value);
            return;
        }
        if physical >= CART_DOMAIN1_ADDRESS2 {
            return;
        }
        if save_window(physical) {
            self.save.write32(physical - CART_DOMAIN2_ADDRESS2, value);
            return;
        }
        if in_range(physical, SP_PC_BASE, 0x08) {
            self.sp_set_pc(value);
            return;
        }
        if in_range(physical, SP_REGISTERS_BASE, 0x20) {
            self.sp_write32(physical - SP_REGISTERS_BASE, value);
            return;
        }
        if in_range(physical, DP_COMMAND_BASE, 0x20) {
            self.dp_write32(physical - DP_COMMAND_BASE, value);
            return;
        }
        if in_range(physical, PI_BASE, 0x34) {
            self.pi_write32(physical - PI_BASE, value);
            return;
        }
        if in_range(physical, MI_BASE, 0x10) {
            self.mi.write32(physical - MI_BASE, value);
            return;
        }
        if in_range(physical, VI_BASE, 0x38) {
            self.vi_write32(physical - VI_BASE, value);
            return;
        }
        if in_range(physical, SI_BASE, 0x1C) {
            self.si_write32(physical - SI_BASE, value);
            return;
        }
        if in_range(physical, AI_BASE, 0x18) {
            self.ai_write32(physical - AI_BASE, value);
            return;
        }
        self.registers.insert(physical, value);
    }

    pub fn read8(&mut self, physical: u32) -> u8 {
        let word = self.read32(physical & !3);
        (word >> ((3 - (physical & 3)) * 8)) as u8
    }

    pub fn write8(&mut self, physical: u32, value: u8) {
        let aligned = physical & !3;
        let shift = (3 - (physical & 3)) * 8;
        let word = self.read32(aligned);
        self.write32(aligned, (word & !(0xFFu32 << shift)) | ((value as u32) << shift));
    }

    pub fn read16(&mut self, physical: u32) -> u16 {
        let word = self.read32(physical & !3);
        (word >> ((2 - (physical & 2)) * 8)) as u16
    }

    pub fn write16(&mut self, physical: u32, value: u16) {
        let aligned = physical & !3;
        let shift = (2 - (physical & 2)) * 8;
        let word = self.read32(aligned);
        self.write32(aligned, (word & !(0xFFFFu32 << shift)) | ((value as u32) << shift));
    }

    pub fn read64(&mut self, physical: u32) -> u64 {
        let high = self.read32(physical) as u64;
        (high << 32) | self.read32(physical.wrapping_add(4)) as u64
    }

    pub fn write64(&mut self, physical: u32, value: u64) {
        self.write32(physical, (value >> 32) as u32);
        self.write32(physical.wrapping_add(4), value as u32);
    }

    /// `LatchesWholeWords`: the signal processor's memories, PIF RAM and the save chip take a whole word whatever size is named.
    fn latches_whole_words(physical: u32) -> bool {
        sp_memory(physical).is_some() || in_range(physical, PIF_RAM_BASE, PIF_RAM_SIZE) || save_window(physical)
    }

    /// `Store`: the processor's own stores. Nothing watches them here, so `Report` never runs.
    pub fn store(&mut self, physical: u32, value: u64, size: u32) {
        if cartridge_bus(physical) {
            self.pi_cartridge_store(whole_word(physical, value, size));
        }
        if physical < RDRAM_REGISTERS_BASE + RDRAM_REGISTERS_SIZE && self.mi.repeating {
            self.mi.repeating = false;
            if physical < RDRAM_REGISTERS_BASE {
                let pattern = if size == 8 { value } else { doubled(whole_word(physical, value, size)) };
                self.repeat(physical, pattern);
                return;
            }
        }
        if Self::latches_whole_words(physical) {
            self.write32(if size == 8 { physical } else { physical & !3 }, whole_word(physical, value, size));
            return;
        }
        match size {
            1 => self.write8(physical, value as u8),
            2 => self.write16(physical, value as u16),
            4 => self.write32(physical, value as u32),
            _ => self.write64(physical, value),
        }
    }

    /// `Load`: the processor's own loads, the only reads that see what a store left on the cartridge bus.
    pub fn load(&mut self, physical: u32, size: u32) -> u64 {
        if cartridge_rom(physical) && size < 8 {
            let word = self.cartridge_word(physical, size);
            return lane(word, physical, size);
        }
        match size {
            1 => self.read8(physical) as u64,
            2 => self.read16(physical) as u64,
            4 => self.read32(physical) as u64,
            _ => self.read64(physical),
        }
    }

    fn cartridge_word(&mut self, physical: u32, size: u32) -> u32 {
        if let Some(stored) = self.pi_take_stored() {
            return stored;
        }
        let aligned = physical & !3;
        let word = self.read32(aligned);
        if size < 4 && (physical & 2) != 0 { (word << 16) | (self.read32(aligned.wrapping_add(4)) >> 16) } else { word }
    }

    fn repeat(&mut self, physical: u32, pattern: u64) {
        let doubleword = physical & !7;
        let length = self.mi.repeat_count + 1;
        let mut at = physical & 7;
        while at < length {
            let address = (doubleword & !REPEAT_ROW_MASK) | (doubleword.wrapping_add(at) & REPEAT_ROW_MASK);
            self.write8(address, (pattern >> (8 * (7 - (address & 7)))) as u8);
            at += 1;
        }
    }

    /// `CartridgeDmaRead8`: a transfer reaches the save chip by a path of its own.
    pub fn cartridge_dma_read8(&mut self, physical: u32) -> u8 {
        if save_window(physical) { self.save.dma_read8(physical - CART_DOMAIN2_ADDRESS2) } else { self.read8(physical) }
    }

    pub fn cartridge_dma_write8(&mut self, physical: u32, value: u8) {
        if save_window(physical) {
            self.save.dma_write8(physical - CART_DOMAIN2_ADDRESS2, value);
        } else {
            self.write8(physical, value);
        }
    }

    /// `ReadCart32`: past the end of the cartridge is zero.
    pub fn read_cart32(&self, offset: u32) -> u32 {
        match self.cart.as_ref() {
            Some(rom) if (offset as u64 + 3) < rom.rom.len() as u64 => be32(&rom.rom, offset),
            _ => 0,
        }
    }

    /// `Tick`: the RSP runs in step with the processor; the VI, AI and SI act only when one is due.
    #[inline(always)]
    pub fn tick(&mut self, cycles: i64) {
        self.cycles += cycles;
        if !self.sp.processor.halted {
            self.sp_step(cycles);
        }
        if self.cycles >= *self.next_event {
            self.run_events();
        }
    }

    /// `RunEvents`: the VI's half lines before the AI's samples, then the SI.
    pub fn run_events(&mut self) {
        self.vi_catch();
        self.ai_catch();
        self.si_catch();
        *self.next_event = (*self.vi.due).min(*self.ai.due).min(self.si.due);
    }

    /// `Sooner`: a device just due earlier than anything scheduled.
    pub fn sooner(&mut self, due: i64) {
        *self.next_event = (*self.next_event).min(due);
    }

    /// `Settle`: both clocks brought up to now at the old rate.
    pub fn settle(&mut self) {
        self.vi_settle();
        self.ai_settle();
    }

    pub fn reschedule(&mut self) {
        self.vi_schedule();
        self.ai_schedule();
        *self.next_event = (*self.vi.due).min(*self.ai.due).min(self.si.due);
    }

    /// `Count`: half the CPU clock, off the one counter.
    #[inline(always)]
    pub fn count(&self) -> u32 {
        (self.cycles >> 1).wrapping_add(self.count_bias) as u32
    }

    pub fn set_count(&mut self, value: u32) {
        self.count_bias = (value as i64).wrapping_sub(self.cycles >> 1);
    }

    /// `ReadRspControl`: the eight interface registers, then the eight the display processor owns.
    pub fn read_rsp_control(&mut self, register: usize) -> u32 {
        if register < 8 {
            self.sp_read32((register as u32) << 2)
        } else {
            self.read32(DP_COMMAND_BASE + (((register - 8) as u32) << 2))
        }
    }

    pub fn write_rsp_control(&mut self, register: usize, value: u32) {
        if register < 8 {
            self.sp_write32((register as u32) << 2, value);
        } else {
            self.write32(DP_COMMAND_BASE + (((register - 8) as u32) << 2), value);
        }
    }
}
