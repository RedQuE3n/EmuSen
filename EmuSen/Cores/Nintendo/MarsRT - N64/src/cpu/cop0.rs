//! Coprocessor zero: its registers, the interrupt check, the timer, exceptions and the TLB instructions. C#'s Cop0Registers.cs and Cpu.Opcodes.Cop0.cs. See Mars_Cop0.md.

use crate::memory::bus::MemoryBus;
use crate::cpu::{Cpu, Exec, code};
use crate::cpu::interp::{rd, rt};
use crate::cpu::segments::Mode;
use crate::cpu::tlb::{ENTRY_LO_GLOBAL, ENTRY_LO_KEPT, Tlb};

pub const INDEX: usize = 0;
pub const RANDOM: usize = 1;
pub const ENTRY_LO0: usize = 2;
pub const ENTRY_LO1: usize = 3;
pub const CONTEXT: usize = 4;
pub const PAGE_MASK: usize = 5;
pub const WIRED: usize = 6;
pub const BAD_VIRTUAL_ADDRESS: usize = 8;
pub const COUNT: usize = 9;
pub const ENTRY_HI: usize = 10;
pub const COMPARE: usize = 11;
pub const STATUS: usize = 12;
pub const CAUSE: usize = 13;
pub const EXCEPTION_PC: usize = 14;
pub const PROCESSOR_ID_REGISTER: usize = 15;
pub const CONFIG: usize = 16;
pub const LINKED_ADDRESS: usize = 17;
pub const XCONTEXT: usize = 20;
pub const PARITY_ERROR: usize = 26;
pub const CACHE_ERROR: usize = 27;
pub const TAG_LO: usize = 28;
pub const TAG_HI: usize = 29;
pub const ERROR_EXCEPTION_PC: usize = 30;

pub const STATUS_INTERRUPT_ENABLE: u64 = 1 << 0;
pub const STATUS_EXCEPTION_LEVEL: u64 = 1 << 1;
pub const STATUS_ERROR_LEVEL: u64 = 1 << 2;
pub const STATUS_USER_EXTENDED: u64 = 1 << 5;
pub const STATUS_SUPERVISOR_EXTENDED: u64 = 1 << 6;
pub const STATUS_KERNEL_EXTENDED: u64 = 1 << 7;
pub const STATUS_BOOTSTRAP_VECTORS: u64 = 1 << 22;
pub const STATUS_REVERSE_ENDIAN: u64 = 1 << 25;
pub const STATUS_FPU_FULL_MODE: u64 = 1 << 26;
pub const STATUS_COP0_USABLE: u64 = 1 << 28;
pub const STATUS_COP1_USABLE: u64 = 1 << 29;
pub const STATUS_COP2_USABLE: u64 = 1 << 30;
const STATUS_MODE_FIELD: u64 = 3 << 3;

pub const CAUSE_BRANCH_DELAY: u64 = 1 << 31;
pub const CAUSE_COPROCESSOR: u64 = 3 << 28;
pub const CAUSE_INTERRUPT_RCP: u64 = 1 << 10;
pub const CAUSE_INTERRUPT_TIMER: u64 = 1 << 15;
const INTERRUPT_SHIFT: u32 = 8;

pub const VECTOR_BASE: u64 = 0xFFFF_FFFF_8000_0000;
pub const VECTOR_BASE_BOOTSTRAP: u64 = 0xFFFF_FFFF_BFC0_0200;

pub const PROCESSOR_ID: u64 = 0x0B22;
pub const CONFIG_WRITABLE: u64 = 0x0F00_800F;
pub const CONFIG_CONSTANT: u64 = 0x7006_6460;
pub const CONFIG_AT_RESET: u64 = 0x7006_E463;
pub const ENTRY_LO_WRITABLE: u64 = 0x3FFF_FFFF;
pub const PAGE_MASK_WRITABLE: u64 = 0x01FF_E000;
pub const ENTRY_HI_WRITABLE: u64 = 0xC000_00FF_FFFF_E0FF;
pub const CONTEXT_WRITABLE: u64 = 0xFFFF_FFFF_FF80_0000;
pub const XCONTEXT_WRITABLE: u64 = 0xFFFF_FFFE_0000_0000;
const CONTEXT_BAD_VPN2: u64 = 0x007F_FFF0;
const XCONTEXT_BAD_VPN2: u64 = 0x7FFF_FFF0;
const XCONTEXT_REGION: u64 = 0x1_8000_0000;

#[inline(always)]
fn is_unused(register: usize) -> bool {
    matches!(register, 7 | 21 | 22 | 23 | 24 | 25 | 31)
}

impl Cpu {
    /// `ComputeMode`: kernel whenever an exception is being handled, whatever the mode field says.
    fn compute_mode(&self) -> Mode {
        let status = self.cop0[STATUS];
        if status & (STATUS_EXCEPTION_LEVEL | STATUS_ERROR_LEVEL) != 0 {
            return Mode::Kernel;
        }
        match (status & STATUS_MODE_FIELD) >> 3 {
            1 => Mode::Supervisor,
            2 => Mode::User,
            _ => Mode::Kernel,
        }
    }

    #[inline]
    pub fn refresh_mode(&mut self) {
        self.run.mode = self.compute_mode();
    }

    #[inline(always)]
    pub fn wide_addressing(&self) -> bool {
        let bit = match self.run.mode {
            Mode::Supervisor => STATUS_SUPERVISOR_EXTENDED,
            Mode::User => STATUS_USER_EXTENDED,
            Mode::Kernel => STATUS_KERNEL_EXTENDED,
        };
        self.cop0[STATUS] & bit != 0
    }

    /// `Cop0Written`: for a caller that wrote COP0 or loaded the processor outside an instruction.
    pub fn cop0_written(&mut self, bus: &MemoryBus) {
        self.run.recheck = true;
        self.run.tlb_generation = self.run.tlb_generation.wrapping_add(1);
        self.refresh_mode();
        self.schedule_timer(bus);
    }

    /// `CheckInterrupts`: level-triggered from the MI, edge-latched from the counter; run only when an input changed.
    #[inline(never)]
    pub fn check_interrupts(&mut self, asserted: bool) -> Exec {
        self.run.recheck = false;
        self.run.asserted_seen = asserted;
        if asserted {
            self.cop0[CAUSE] |= CAUSE_INTERRUPT_RCP;
        } else {
            self.cop0[CAUSE] &= !CAUSE_INTERRUPT_RCP;
        }
        let status = self.cop0[STATUS];
        if status & STATUS_INTERRUPT_ENABLE == 0 {
            return Ok(());
        }
        if status & (STATUS_EXCEPTION_LEVEL | STATUS_ERROR_LEVEL) != 0 {
            return Ok(());
        }
        let pending = (self.cop0[CAUSE] >> INTERRUPT_SHIFT) & (status >> INTERRUPT_SHIFT) & 0xFF;
        if pending != 0 {
            return Err(self.raise(code::INTERRUPT, self.current_pc));
        }
        Ok(())
    }

    /// `ScheduleTimer`: the first cycle the counter reaches Compare, from the count the timer last settled at.
    pub fn schedule_timer(&mut self, bus: &MemoryBus) {
        let now = bus.count();
        let compare = self.cop0[COMPARE] as u32;
        let to_compare = compare.wrapping_sub(self.last_count);
        let owed: i64 = if to_compare == 0 { 1 << 32 } else { to_compare as i64 };
        let remaining = owed - now.wrapping_sub(self.last_count) as i64;
        self.run.timer_due = if remaining <= 0 { bus.cycles } else { 2 * ((bus.cycles >> 1) + remaining) };
        self.run.scheduled_compare = compare;
        self.run.scheduled_bias = now.wrapping_sub((bus.cycles >> 1) as u32);
    }

    /// `TimerReached`: the first successful step to end on or past Compare.
    #[inline(never)]
    pub fn timer_reached(&mut self, bus: &MemoryBus) {
        self.cop0[CAUSE] |= CAUSE_INTERRUPT_TIMER;
        self.run.recheck = true;
        self.last_count = bus.count();
        self.schedule_timer(bus);
    }

    fn read_cop0(&mut self, bus: &MemoryBus, register: usize) -> u64 {
        if is_unused(register) {
            return self.cop0_latch;
        }
        match register {
            COUNT => bus.count() as u64,
            RANDOM => self.read_random(),
            _ => self.cop0[register],
        }
    }

    /// `ReadRandom`: counts down from 31 to Wired and starts again; above 31 through the whole six bits.
    fn read_random(&self) -> u64 {
        let wired = self.cop0[WIRED] & 0x3F;
        let period = if wired <= 31 { 32 - wired } else { 96 - wired };
        let elapsed = (self.instructions.wrapping_sub(self.random_start) as u64) % period;
        31u64.wrapping_sub(elapsed) & 0x3F
    }

    fn write_cop0(&mut self, bus: &mut MemoryBus, register: usize, value: u64) {
        self.cop0_latch = value;
        self.run.tlb_generation = self.run.tlb_generation.wrapping_add(1);
        if is_unused(register) {
            return;
        }
        self.run.recheck = true;
        if register == COUNT {
            bus.set_count(value as u32);
            self.last_count = bus.count();
            self.schedule_timer(bus);
            return;
        }
        self.cop0[register] = self.mask_cop0_write(register, value);
        if register == STATUS {
            self.refresh_mode();
        }
        if register == WIRED {
            self.random_start = self.instructions;
        }
        if register == COMPARE {
            self.cop0[CAUSE] &= !CAUSE_INTERRUPT_TIMER;
            self.last_count = bus.count();
            self.schedule_timer(bus);
        }
    }

    /// `MaskCop0Write`: narrower than a word, read-only, constant, or half owned by hardware.
    fn mask_cop0_write(&self, register: usize, value: u64) -> u64 {
        match register {
            INDEX => value & 0x8000_003F,
            RANDOM => self.cop0[RANDOM],
            ENTRY_LO0 | ENTRY_LO1 => value & ENTRY_LO_WRITABLE,
            PAGE_MASK => value & PAGE_MASK_WRITABLE,
            ENTRY_HI => value & ENTRY_HI_WRITABLE,
            CONTEXT => (value & CONTEXT_WRITABLE) | (self.cop0[CONTEXT] & !CONTEXT_WRITABLE),
            WIRED => value & 0x3F,
            BAD_VIRTUAL_ADDRESS => self.cop0[BAD_VIRTUAL_ADDRESS],
            STATUS => value & 0xFFF7_FFFF,
            PROCESSOR_ID_REGISTER => PROCESSOR_ID,
            CONFIG => (value & CONFIG_WRITABLE) | CONFIG_CONSTANT,
            LINKED_ADDRESS => value & 0xFFFF_FFFF,
            XCONTEXT => (value & XCONTEXT_WRITABLE) | (self.cop0[XCONTEXT] & !XCONTEXT_WRITABLE),
            PARITY_ERROR => value & 0xFF,
            CACHE_ERROR => 0,
            TAG_LO => value & 0xFFFF_FFFF,
            TAG_HI => 0,
            _ => value,
        }
    }

    /// `RequireCop0`: kernel mode reaches coprocessor zero without permission.
    fn require_cop0(&mut self) -> Exec {
        if self.run.mode == Mode::Kernel || self.cop0[STATUS] & STATUS_COP0_USABLE != 0 {
            return Ok(());
        }
        Err(self.raise(code::COPROCESSOR_UNUSABLE, self.current_pc))
    }

    pub(crate) fn execute_cop0(&mut self, bus: &mut MemoryBus, i: u32) -> Exec {
        self.require_cop0()?;
        let rs = (i >> 21) & 0x1F;
        if rs & 0x10 != 0 {
            match i & 0x3F {
                0x01 => self.read_tlb_entry(),
                0x02 => self.write_tlb_entry((self.cop0[INDEX] & 0x1F) as usize),
                0x06 => {
                    let index = self.read_random() as usize;
                    self.write_tlb_entry(index);
                }
                0x08 => self.probe_tlb(),
                0x18 => self.return_from_exception(),
                0x10 => return Err(self.raise(code::RESERVED_INSTRUCTION, self.current_pc)),
                _ => {}
            }
            return Ok(());
        }
        match rs {
            0x00 => {
                let value = self.read_cop0(bus, rd(i)) as u32;
                self.write32(rt(i), value);
            }
            0x01 => {
                let value = self.read_cop0(bus, rd(i));
                self.write(rt(i), value);
            }
            0x04 | 0x05 => self.write_cop0(bus, rd(i), self.read(rt(i))),
            0x02 | 0x06 | 0x08 => {}
            _ => return Err(self.raise(code::RESERVED_INSTRUCTION, self.current_pc)),
        }
        Ok(())
    }

    /// `EnterException`: everything an exception does to the machine before the handler's first instruction.
    pub fn enter_exception(&mut self) {
        let raised = self.run.fault;
        self.run.exceptions += 1;
        // `InterruptObserver`: an interrupt taken, which `runto irq` waits for (Mars_Native.md §6.5).
        if raised.code == code::INTERRUPT && self.hooks.interrupts {
            self.hooks.interrupt_taken = true;
        }
        self.run.tlb_generation = self.run.tlb_generation.wrapping_add(1);
        let already_handling = self.cop0[STATUS] & STATUS_EXCEPTION_LEVEL != 0;
        let extended = self.wide_addressing();
        if !already_handling {
            self.cop0[EXCEPTION_PC] = if raised.in_delay_slot { self.current_pc.wrapping_sub(4) } else { self.current_pc };
            if raised.in_delay_slot {
                self.cop0[CAUSE] |= CAUSE_BRANCH_DELAY;
            } else {
                self.cop0[CAUSE] &= !CAUSE_BRANCH_DELAY;
            }
        }
        self.cop0[CAUSE] = (self.cop0[CAUSE] & !0x7C) | ((raised.code as u64) << 2);
        self.cop0[CAUSE] = (self.cop0[CAUSE] & !CAUSE_COPROCESSOR) | ((raised.coprocessor as u64) << 28);
        if matches!(raised.code, code::ADDRESS_ERROR_LOAD | code::ADDRESS_ERROR_STORE | code::TLB_LOAD | code::TLB_STORE | code::TLB_MODIFICATION) {
            self.record_faulting_address(raised.address);
        }
        self.cop0[STATUS] |= STATUS_EXCEPTION_LEVEL;
        self.refresh_mode();
        self.linked_flag = false;
        self.run.recheck = true;
        self.pc = self.vector_for(raised.refill, already_handling, extended);
        self.next_pc = self.pc.wrapping_add(4);
        self.branch_pending = false;
        self.in_delay_slot = false;
    }

    fn return_from_exception(&mut self) {
        if self.cop0[STATUS] & STATUS_ERROR_LEVEL != 0 {
            self.pc = self.cop0[ERROR_EXCEPTION_PC];
            self.cop0[STATUS] &= !STATUS_ERROR_LEVEL;
        } else {
            self.pc = self.cop0[EXCEPTION_PC];
            self.cop0[STATUS] &= !STATUS_EXCEPTION_LEVEL;
        }
        self.linked_flag = false;
        self.refresh_mode();
        self.run.recheck = true;
        self.next_pc = self.pc.wrapping_add(4);
        self.branch_pending = false;
    }

    /// `VectorFor`: a refill has its own vector only on the way in from ordinary execution.
    fn vector_for(&self, refill: bool, already_handling: bool, extended: bool) -> u64 {
        let start = if self.cop0[STATUS] & STATUS_BOOTSTRAP_VECTORS != 0 { VECTOR_BASE_BOOTSTRAP } else { VECTOR_BASE };
        if !refill || already_handling {
            return start + 0x180;
        }
        start + if extended { 0x080 } else { 0x000 }
    }

    fn read_tlb_entry(&mut self) {
        self.run.tlb_generation = self.run.tlb_generation.wrapping_add(1);
        let entry = self.tlb.entries[(self.cop0[INDEX] & 0x1F) as usize];
        self.cop0[PAGE_MASK] = entry.page_mask;
        self.cop0[ENTRY_HI] = entry.entry_hi;
        self.cop0[ENTRY_LO0] = entry.entry_lo0;
        self.cop0[ENTRY_LO1] = entry.entry_lo1;
    }

    /// `WriteTlbEntry`: an entry keeps less than the registers hold, and one global flag for both halves.
    fn write_tlb_entry(&mut self, index: usize) {
        self.run.tlb_generation = self.run.tlb_generation.wrapping_add(1);
        let page_mask = Tlb::paired_page_mask(self.cop0[PAGE_MASK]);
        let global = self.cop0[ENTRY_LO0] & self.cop0[ENTRY_LO1] & ENTRY_LO_GLOBAL;
        let entry = &mut self.tlb.entries[index & 0x1F];
        entry.page_mask = page_mask;
        entry.entry_hi = self.cop0[ENTRY_HI] & ENTRY_HI_WRITABLE & !page_mask;
        entry.entry_lo0 = (self.cop0[ENTRY_LO0] & ENTRY_LO_KEPT) | global;
        entry.entry_lo1 = (self.cop0[ENTRY_LO1] & ENTRY_LO_KEPT) | global;
    }

    fn probe_tlb(&mut self) {
        let found = self.tlb.probe(self.cop0[ENTRY_HI]);
        self.cop0[INDEX] = if found < 0 { 0x8000_0000 } else { found as u64 };
    }

    /// `RecordFaultingAddress`: the four registers a fault fills in, all from the one address.
    fn record_faulting_address(&mut self, address: u64) {
        self.cop0[BAD_VIRTUAL_ADDRESS] = address;
        self.cop0[ENTRY_HI] = (self.cop0[ENTRY_HI] & 0xFF) | (address & ENTRY_HI_WRITABLE & !0xFF);
        self.cop0[CONTEXT] = (self.cop0[CONTEXT] & CONTEXT_WRITABLE) | ((address >> 9) & CONTEXT_BAD_VPN2);
        self.cop0[XCONTEXT] = (self.cop0[XCONTEXT] & XCONTEXT_WRITABLE) | ((address >> 9) & XCONTEXT_BAD_VPN2) | ((address >> 31) & XCONTEXT_REGION);
    }
}
