//! The VR4300 interpreter: the step, the fetch, translation, and the integer instructions. C#'s Cpu.cs and Cpu/Opcodes. See Mars_Cpu.md.

use crate::memory::bus::MemoryBus;
use crate::memory::dp_threads::site;
use crate::cpu::cop0::*;
use crate::cpu::{Cpu, Exec, Fault, Raised, code};
use crate::cpu::segments::{self, Mode, Segment};
use crate::cpu::tlb::TlbResult;

/// KSEG0 and KSEG1 as a sign-extended range, where kernel mode never mirrors.
pub const KERNEL_DIRECT_BASE: u64 = 0xFFFF_FFFF_8000_0000;
pub const KERNEL_DIRECT_SIZE: u64 = 0x4000_0000;

pub const MULTIPLY_STALL: i32 = 5;
pub const MULTIPLY_DOUBLE_STALL: i32 = 8;
pub const DIVIDE_STALL: i32 = 37;
pub const DIVIDE_DOUBLE_STALL: i32 = 69;
/// `MultiplyOperandBits`: how wide the multiplier reads its second operand.
const MULTIPLY_OPERAND_BITS: u32 = 35;

#[inline(always)]
pub fn rs(i: u32) -> usize {
    ((i >> 21) & 0x1F) as usize
}
#[inline(always)]
pub fn rt(i: u32) -> usize {
    ((i >> 16) & 0x1F) as usize
}
#[inline(always)]
pub fn rd(i: u32) -> usize {
    ((i >> 11) & 0x1F) as usize
}
#[inline(always)]
pub(crate) fn sa(i: u32) -> u32 {
    (i >> 6) & 0x1F
}
#[inline(always)]
pub(crate) fn immediate(i: u32) -> u64 {
    (i & 0xFFFF) as u64
}
#[inline(always)]
pub(crate) fn signed_immediate(i: u32) -> i64 {
    i as i16 as i64
}

#[inline(always)]
fn overflowed_add(left: i64, right: i64, result: i64) -> bool {
    ((left ^ result) & (right ^ result)) < 0
}

#[inline(always)]
fn overflowed_subtract(left: i64, right: i64, result: i64) -> bool {
    ((left ^ right) & (left ^ result)) < 0
}

/// `IsDoubleword`: what the other two modes may run only with their own addressing bit.
#[inline(always)]
fn is_doubleword(op: u32, instruction: u32) -> bool {
    match op {
        0x00 => matches!(
            instruction & 0x3F,
            0x14 | 0x16 | 0x17 | 0x1C | 0x1D | 0x1E | 0x1F | 0x2C | 0x2D | 0x2E | 0x2F | 0x38 | 0x3A | 0x3B | 0x3C | 0x3E | 0x3F
        ),
        0x18 | 0x19 | 0x1A | 0x1B | 0x2C | 0x2D | 0x34 | 0x37 | 0x3C | 0x3F => true,
        _ => false,
    }
}

impl Cpu {
    /// `new Cpu(bus)`: the program counter at IPL's entry, the mode derived, the timer scheduled.
    pub fn power_on(bus: &MemoryBus) -> Cpu {
        let mut cpu = Cpu { pc: 0xFFFF_FFFF_A400_0040, next_pc: 0xFFFF_FFFF_A400_0044, ..Cpu::default() };
        cpu.refresh_mode();
        cpu.schedule_timer(bus);
        cpu
    }

    /// `Step`: one instruction, or the exception it raised.
    #[inline(always)]
    pub fn step(&mut self, bus: &mut MemoryBus) {
        self.current_pc = self.pc;
        self.in_delay_slot = self.branch_pending;
        if self.step_body(bus).is_err() {
            self.enter_exception();
            bus.tick(1);
        }
    }

    #[inline(always)]
    fn step_body(&mut self, bus: &mut MemoryBus) -> Exec {
        let asserted = bus.mi.asserted();
        if self.run.recheck || asserted != self.run.asserted_seen {
            self.check_interrupts(asserted)?;
        }
        let instruction = self.fetch_instruction(bus)?;
        self.branch_pending = false;
        self.pc = self.next_pc;
        self.next_pc = self.pc.wrapping_add(4);
        self.execute(bus, instruction)?;
        bus.tick(1 + self.extra_cycles as i64);
        self.extra_cycles = 0;
        self.instructions += 1;
        // `ReturnObserver`: the return a `jr ra` began is done once its delay slot has run.
        if self.hooks.return_after_slot && self.in_delay_slot {
            self.hooks.return_after_slot = false;
            self.hooks.note_return();
        }
        if bus.cycles >= self.run.timer_due {
            self.timer_reached(bus);
        }
        self.last_count = bus.count();
        Ok(())
    }

    #[inline(always)]
    fn fetch_instruction(&mut self, bus: &mut MemoryBus) -> Exec<u32> {
        let pc = self.pc;
        if pc & 3 != 0 {
            return Err(self.raise(code::ADDRESS_ERROR_LOAD, pc));
        }
        if pc.wrapping_sub(KERNEL_DIRECT_BASE) < KERNEL_DIRECT_SIZE && self.run.mode == Mode::Kernel {
            let physical = (pc as u32) & 0x1FFF_FFFF;
            if (physical as usize) < bus.rdram.len() {
                if bus.dp.read_marked(physical) {
                    bus.dp.wait_read(physical, 4, site::FETCH);
                }
                return Ok(bus.rdram.be32(physical));
            }
            return Ok(bus.read32(physical));
        }
        let access = self.mirrored(pc, 4);
        let physical = self.translate_access(access, pc, false)?;
        Ok(bus.read32(physical))
    }

    /// `Translate`: the access and the address a fault names are the same word.
    #[inline(always)]
    pub fn translate(&mut self, address: u64, store: bool) -> Exec<u32> {
        self.translate_access(address, address, store)
    }

    #[inline(always)]
    pub fn translate_access(&mut self, access: u64, fault: u64, store: bool) -> Exec<u32> {
        if access.wrapping_sub(KERNEL_DIRECT_BASE) < KERNEL_DIRECT_SIZE && self.run.mode == Mode::Kernel {
            return Ok((access as u32) & 0x1FFF_FFFF);
        }
        self.translate_slow(access, fault, store)
    }

    #[inline(never)]
    fn translate_slow(&mut self, access: u64, fault: u64, store: bool) -> Exec<u32> {
        match segments::decode(access, self.run.mode, self.wide_addressing()) {
            Segment::Illegal => Err(self.raise(if store { code::ADDRESS_ERROR_STORE } else { code::ADDRESS_ERROR_LOAD }, fault)),
            Segment::Direct(physical) => Ok(physical),
            Segment::Mapped => match self.tlb.try_translate(access, self.cop0[ENTRY_HI], store) {
                TlbResult::Mapped(physical) => Ok(physical),
                TlbResult::NotWritable => Err(self.raise(code::TLB_MODIFICATION, fault)),
                TlbResult::Invalid => Err(self.raise(if store { code::TLB_STORE } else { code::TLB_LOAD }, fault)),
                TlbResult::Missing => Err(self.raise_with(if store { code::TLB_STORE } else { code::TLB_LOAD }, fault, true, 0)),
            },
        }
    }

    /// `ReverseEndian`: in user mode the bit mirrors every access inside its doubleword.
    #[inline(always)]
    pub fn reverse_endian(&self) -> bool {
        self.run.mode == Mode::User && (self.cop0[STATUS] & STATUS_REVERSE_ENDIAN) != 0
    }

    #[inline(always)]
    fn mirrored(&self, address: u64, size: u32) -> u64 {
        if self.reverse_endian() { (address ^ 7) & !(size as u64 - 1) } else { address }
    }

    #[inline(always)]
    pub fn raise(&mut self, code: u32, address: u64) -> Raised {
        self.raise_with(code, address, false, 0)
    }

    #[inline(always)]
    pub fn raise_with(&mut self, code: u32, address: u64, refill: bool, coprocessor: u32) -> Raised {
        self.run.fault = Fault { code, address, in_delay_slot: self.in_delay_slot, refill, coprocessor };
        Raised
    }

    #[inline(always)]
    pub fn write(&mut self, register: usize, value: u64) {
        if register != 0 {
            self.gpr[register] = value;
        }
    }

    #[inline(always)]
    pub fn write32(&mut self, register: usize, value: u32) {
        self.write(register, value as i32 as i64 as u64);
    }

    #[inline(always)]
    pub fn read(&self, register: usize) -> u64 {
        self.gpr[register]
    }

    /// `Branch`: the slot runs next; a branch to itself marks where the idle loop may be.
    #[inline(always)]
    pub(crate) fn branch(&mut self, target: u64) {
        if target == self.current_pc {
            self.run.idle_at = target;
        }
        self.next_pc = target;
        self.branch_pending = true;
    }

    #[inline(always)]
    pub(crate) fn execute(&mut self, bus: &mut MemoryBus, instruction: u32) -> Exec {
        let op = instruction >> 26;
        if self.run.mode != Mode::Kernel && !self.wide_addressing() && is_doubleword(op, instruction) {
            return Err(self.raise(code::RESERVED_INSTRUCTION, self.current_pc));
        }
        let i = instruction;
        match op {
            0x00 => return self.special(bus, i),
            0x01 => return self.regimm(i),
            0x02 => self.branch(self.jump_target(i)),
            0x03 => {
                self.write(31, self.next_pc);
                let target = self.jump_target(i);
                if self.hooks.calls {
                    self.hooks.note_call(self.current_pc, target);
                }
                self.branch(target);
            }
            0x04 => self.branch_if(self.read(rs(i)) == self.read(rt(i)), i, false, false),
            0x05 => self.branch_if(self.read(rs(i)) != self.read(rt(i)), i, false, false),
            0x06 => self.branch_if(self.read(rs(i)) as i64 <= 0, i, false, false),
            0x07 => self.branch_if(self.read(rs(i)) as i64 > 0, i, false, false),
            0x08 => return self.add_immediate(i, true),
            0x09 => return self.add_immediate(i, false),
            0x0A => self.write(rt(i), ((self.read(rs(i)) as i64) < signed_immediate(i)) as u64),
            0x0B => self.write(rt(i), (self.read(rs(i)) < signed_immediate(i) as u64) as u64),
            0x0C => self.write(rt(i), self.read(rs(i)) & immediate(i)),
            0x0D => self.write(rt(i), self.read(rs(i)) | immediate(i)),
            0x0E => self.write(rt(i), self.read(rs(i)) ^ immediate(i)),
            0x0F => self.write32(rt(i), (immediate(i) << 16) as u32),
            0x10 => return self.execute_cop0(bus, i),
            0x11 => return self.execute_cop1(i),
            0x12 => return self.execute_cop2(i),
            0x14 => self.branch_if(self.read(rs(i)) == self.read(rt(i)), i, true, false),
            0x15 => self.branch_if(self.read(rs(i)) != self.read(rt(i)), i, true, false),
            0x16 => self.branch_if(self.read(rs(i)) as i64 <= 0, i, true, false),
            0x17 => self.branch_if(self.read(rs(i)) as i64 > 0, i, true, false),
            0x18 => return self.add_immediate64(i, true),
            0x19 => return self.add_immediate64(i, false),
            0x1A => return self.load_double_left(bus, i),
            0x1B => return self.load_double_right(bus, i),
            0x20 => return self.load(bus, i, 1, true),
            0x21 => return self.load(bus, i, 2, true),
            0x22 => return self.load_word_left(bus, i),
            0x23 => return self.load(bus, i, 4, true),
            0x24 => return self.load(bus, i, 1, false),
            0x25 => return self.load(bus, i, 2, false),
            0x26 => return self.load_word_right(bus, i),
            0x27 => return self.load(bus, i, 4, false),
            0x2F => return self.cache(i),
            0x31 => return self.load_cop1(bus, i, false),
            0x35 => return self.load_cop1(bus, i, true),
            0x39 => _ = self.store_cop1(bus, i, false)?,
            0x3D => _ = self.store_cop1(bus, i, true)?,
            0x30 => return self.load_linked(bus, i, 4),
            0x34 => return self.load_linked(bus, i, 8),
            0x37 => return self.load(bus, i, 8, false),
            0x28 => _ = self.store(bus, i, 1)?,
            0x29 => _ = self.store(bus, i, 2)?,
            0x2A => _ = self.store_word_left(bus, i)?,
            0x2B => _ = self.store(bus, i, 4)?,
            0x2C => _ = self.store_double_left(bus, i)?,
            0x2D => _ = self.store_double_right(bus, i)?,
            0x2E => _ = self.store_word_right(bus, i)?,
            0x38 => _ = self.store_conditional(bus, i, 4)?,
            0x3C => _ = self.store_conditional(bus, i, 8)?,
            0x3F => _ = self.store(bus, i, 8)?,
            _ => return Err(self.raise(code::RESERVED_INSTRUCTION, self.current_pc)),
        }
        Ok(())
    }

    #[inline(always)]
    pub(crate) fn special(&mut self, bus: &mut MemoryBus, i: u32) -> Exec {
        let _ = bus;
        match i & 0x3F {
            0x00 => self.write32(rd(i), (self.read(rt(i)) as u32) << sa(i)),
            0x02 => self.write32(rd(i), (self.read(rt(i)) as u32) >> sa(i)),
            0x03 => self.shift_right_arithmetic(i, sa(i)),
            0x04 => self.write32(rd(i), (self.read(rt(i)) as u32) << (self.read(rs(i)) & 0x1F)),
            0x06 => self.write32(rd(i), (self.read(rt(i)) as u32) >> (self.read(rs(i)) & 0x1F)),
            0x07 => self.shift_right_arithmetic(i, (self.read(rs(i)) & 0x1F) as u32),
            0x08 => self.jump_register(i, false),
            0x09 => self.jump_register(i, true),
            0x0F => {}
            0x0C => return Err(self.raise(code::SYSCALL, self.current_pc)),
            0x0D => return Err(self.raise(code::BREAKPOINT, self.current_pc)),
            0x10 => self.write(rd(i), self.hi),
            0x11 => self.hi = self.read(rs(i)),
            0x12 => self.write(rd(i), self.lo),
            0x13 => self.lo = self.read(rs(i)),
            0x14 => self.write(rd(i), self.read(rt(i)) << (self.read(rs(i)) & 0x3F)),
            0x16 => self.write(rd(i), self.read(rt(i)) >> (self.read(rs(i)) & 0x3F)),
            0x17 => self.write(rd(i), ((self.read(rt(i)) as i64) >> (self.read(rs(i)) & 0x3F)) as u64),
            0x18 => self.multiply(i, false),
            0x19 => self.multiply(i, true),
            0x1A => self.divide(i, false),
            0x1B => self.divide(i, true),
            0x1C => self.multiply_double(i, false),
            0x1D => self.multiply_double(i, true),
            0x1E => self.divide_double(i, false),
            0x1F => self.divide_double(i, true),
            0x20 => return self.add(i, true),
            0x21 => return self.add(i, false),
            0x22 => return self.subtract(i, true),
            0x23 => return self.subtract(i, false),
            0x24 => self.write(rd(i), self.read(rs(i)) & self.read(rt(i))),
            0x25 => self.write(rd(i), self.read(rs(i)) | self.read(rt(i))),
            0x26 => self.write(rd(i), self.read(rs(i)) ^ self.read(rt(i))),
            0x27 => self.write(rd(i), !(self.read(rs(i)) | self.read(rt(i)))),
            0x2A => self.write(rd(i), ((self.read(rs(i)) as i64) < (self.read(rt(i)) as i64)) as u64),
            0x2B => self.write(rd(i), (self.read(rs(i)) < self.read(rt(i))) as u64),
            0x2C => return self.add64(i, true),
            0x2D => return self.add64(i, false),
            0x2E => return self.subtract64(i, true),
            0x2F => return self.subtract64(i, false),
            0x30..=0x34 | 0x36 => return self.execute_trap(i),
            0x38 => self.write(rd(i), self.read(rt(i)) << sa(i)),
            0x3A => self.write(rd(i), self.read(rt(i)) >> sa(i)),
            0x3B => self.write(rd(i), ((self.read(rt(i)) as i64) >> sa(i)) as u64),
            0x3C => self.write(rd(i), self.read(rt(i)) << (sa(i) + 32)),
            0x3E => self.write(rd(i), self.read(rt(i)) >> (sa(i) + 32)),
            0x3F => self.write(rd(i), ((self.read(rt(i)) as i64) >> (sa(i) + 32)) as u64),
            _ => return Err(self.raise(code::RESERVED_INSTRUCTION, self.current_pc)),
        }
        Ok(())
    }

    pub(crate) fn regimm(&mut self, i: u32) -> Exec {
        let value = self.read(rs(i)) as i64;
        match rt(i) {
            0x00 => self.branch_if(value < 0, i, false, false),
            0x01 => self.branch_if(value >= 0, i, false, false),
            0x02 => self.branch_if(value < 0, i, true, false),
            0x03 => self.branch_if(value >= 0, i, true, false),
            0x08..=0x0C | 0x0E => return self.execute_trap_immediate(i),
            0x10 => self.branch_if(value < 0, i, false, true),
            0x11 => self.branch_if(value >= 0, i, false, true),
            0x12 => self.branch_if(value < 0, i, true, true),
            0x13 => self.branch_if(value >= 0, i, true, true),
            _ => return Err(self.raise(code::RESERVED_INSTRUCTION, self.current_pc)),
        }
        Ok(())
    }

    /// `ShiftRightArithmetic`: the whole register shifted, then truncated.
    #[inline(always)]
    pub(crate) fn shift_right_arithmetic(&mut self, i: u32, amount: u32) {
        self.write32(rd(i), ((self.read(rt(i)) as i64) >> amount) as u32);
    }

    pub(crate) fn add_immediate(&mut self, i: u32, trap: bool) -> Exec {
        let left = self.read(rs(i)) as u32 as i32;
        let right = signed_immediate(i) as i32;
        let result = left.wrapping_add(right);
        if trap && overflowed_add(left as i64, right as i64, result as i64) {
            return Err(self.raise(code::OVERFLOW, self.current_pc));
        }
        self.write32(rt(i), result as u32);
        Ok(())
    }

    pub(crate) fn add_immediate64(&mut self, i: u32, trap: bool) -> Exec {
        let left = self.read(rs(i)) as i64;
        let right = signed_immediate(i);
        let result = left.wrapping_add(right);
        if trap && overflowed_add(left, right, result) {
            return Err(self.raise(code::OVERFLOW, self.current_pc));
        }
        self.write(rt(i), result as u64);
        Ok(())
    }

    pub(crate) fn add(&mut self, i: u32, trap: bool) -> Exec {
        let left = self.read(rs(i)) as u32 as i32;
        let right = self.read(rt(i)) as u32 as i32;
        let result = left.wrapping_add(right);
        if trap && overflowed_add(left as i64, right as i64, result as i64) {
            return Err(self.raise(code::OVERFLOW, self.current_pc));
        }
        self.write32(rd(i), result as u32);
        Ok(())
    }

    pub(crate) fn subtract(&mut self, i: u32, trap: bool) -> Exec {
        let left = self.read(rs(i)) as u32 as i32;
        let right = self.read(rt(i)) as u32 as i32;
        let result = left.wrapping_sub(right);
        if trap && overflowed_subtract(left as i64, right as i64, result as i64) {
            return Err(self.raise(code::OVERFLOW, self.current_pc));
        }
        self.write32(rd(i), result as u32);
        Ok(())
    }

    pub(crate) fn add64(&mut self, i: u32, trap: bool) -> Exec {
        let left = self.read(rs(i)) as i64;
        let right = self.read(rt(i)) as i64;
        let result = left.wrapping_add(right);
        if trap && overflowed_add(left, right, result) {
            return Err(self.raise(code::OVERFLOW, self.current_pc));
        }
        self.write(rd(i), result as u64);
        Ok(())
    }

    pub(crate) fn subtract64(&mut self, i: u32, trap: bool) -> Exec {
        let left = self.read(rs(i)) as i64;
        let right = self.read(rt(i)) as i64;
        let result = left.wrapping_sub(right);
        if trap && overflowed_subtract(left, right, result) {
            return Err(self.raise(code::OVERFLOW, self.current_pc));
        }
        self.write(rd(i), result as u64);
        Ok(())
    }

    /// `BranchIf`: the link happens whether or not the branch is taken; a likely branch not taken throws its slot away.
    #[inline(always)]
    pub(crate) fn branch_if(&mut self, taken: bool, i: u32, likely: bool, link: bool) {
        if link {
            self.write(31, self.next_pc);
        }
        if taken {
            let target = self.pc.wrapping_add((signed_immediate(i) << 2) as u64);
            if link && self.hooks.calls {
                self.hooks.note_call(self.current_pc, target);
            }
            self.branch(target);
            return;
        }
        if likely {
            self.pc = self.next_pc;
            self.next_pc = self.pc.wrapping_add(4);
            return;
        }
        self.branch_pending = true;
    }

    /// `JumpTarget`: the low 28 bits of the address the delay slot sits at, replaced.
    #[inline(always)]
    pub(crate) fn jump_target(&self, i: u32) -> u64 {
        (self.pc & 0xFFFF_FFFF_F000_0000) | (((i & 0x03FF_FFFF) << 2) as u64)
    }

    #[inline(always)]
    pub(crate) fn jump_register(&mut self, i: u32, link: bool) {
        let target = self.read(rs(i));
        if link {
            self.write(rd(i), self.next_pc);
        }
        // A `jalr` is a call and a `jr` through `ra` a return, the conventions `bt` reads (Mars_Debug.md §2).
        if self.hooks.calls {
            if link {
                self.hooks.note_call(self.current_pc, target);
            } else if rs(i) == 31 {
                self.hooks.return_after_slot = true;
            }
        }
        self.branch(target);
    }

    #[inline(always)]
    pub(crate) fn effective_address(&self, i: u32) -> u64 {
        self.read(rs(i)).wrapping_add(signed_immediate(i) as u64)
    }

    /// `RequireAlignment`: a misaligned access faults before anything is read or written.
    #[inline(always)]
    fn require_alignment(&mut self, address: u64, size: u32, code: u32) -> Exec {
        if address & (size as u64 - 1) != 0 {
            return Err(self.raise(code, address));
        }
        Ok(())
    }

    #[inline(always)]
    pub(crate) fn load(&mut self, bus: &mut MemoryBus, i: u32, size: u32, signed: bool) -> Exec {
        let address = self.effective_address(i);
        self.require_alignment(address, size, code::ADDRESS_ERROR_LOAD)?;
        let physical = self.translate_access(self.mirrored(address, size), address, false)?;
        let raw = if (physical as usize) < bus.rdram.len() {
            if bus.dp.read_marked(physical) {
                bus.dp.wait_read(physical, size, site::LOAD);
            }
            bus.rdram.read(physical, size)
        } else {
            bus.load(physical, size)
        };
        let value = if !signed {
            raw
        } else {
            match size {
                1 => raw as u8 as i8 as i64 as u64,
                2 => raw as u16 as i16 as i64 as u64,
                4 => raw as u32 as i32 as i64 as u64,
                _ => raw,
            }
        };
        self.write(rt(i), value);
        Ok(())
    }

    /// `Store`: an aligned RDRAM store is the bytes named, unless the MI repeats it; returns where it landed, or `THROUGH_BUS`.
    #[inline(always)]
    pub(crate) fn store(&mut self, bus: &mut MemoryBus, i: u32, size: u32) -> Exec<u32> {
        let address = self.effective_address(i);
        self.require_alignment(address, size, code::ADDRESS_ERROR_STORE)?;
        let physical = self.translate_access(self.mirrored(address, size), address, true)?;
        let value = self.read(rt(i));
        if (physical as usize) < bus.rdram.len() && !bus.mi.repeating {
            if bus.dp.write_marked(physical) {
                bus.dp.wait_write(physical, size, site::STORE);
            }
            bus.rdram.write(physical, value, size);
            if self.hooks.writes {
                self.report_store(bus, physical, size);
            }
            return Ok(physical);
        }
        bus.store(physical, value, size);
        if self.hooks.writes {
            self.report_store(bus, physical, size);
        }
        Ok(THROUGH_BUS)
    }

    /// `MemoryBus.Report`: what the store left, byte by byte in the memory it landed in; a latching window reports its whole word.
    #[inline(never)]
    fn report_store(&mut self, bus: &MemoryBus, physical: u32, size: u32) {
        use crate::ffi::space;
        use crate::memory::bus_access::{latches_whole_words, map, sp_memory};
        let whole = latches_whole_words(physical);
        let (first, count) = if whole && size < 8 { (physical & !3, 4) } else { (physical, size) };
        let pc = self.current_pc;
        for at in first..first.wrapping_add(count) {
            if (at as usize) < bus.rdram.len() {
                self.hooks.note_write(space::RDRAM, at, bus.rdram[at as usize], pc);
            } else if let Some((imem, offset)) = sp_memory(at) {
                let bank = if imem { &bus.sp_imem[..] } else { &bus.sp_dmem[..] };
                self.hooks.note_write(if imem { space::IMEM } else { space::DMEM }, offset, bank[offset as usize], pc);
            } else if at.wrapping_sub(map::PIF_RAM_BASE) < map::PIF_RAM_SIZE {
                let offset = at - map::PIF_RAM_BASE;
                self.hooks.note_write(space::PIF_RAM, offset, bus.pif_ram[offset as usize], pc);
            }
        }
    }

    /// `LoadLinked`: the load arms the link; the address is taken again after the load, as C# takes it.
    pub(crate) fn load_linked(&mut self, bus: &mut MemoryBus, i: u32, size: u32) -> Exec {
        self.load(bus, i, size, size == 4)?;
        self.linked_flag = true;
        let address = self.effective_address(i);
        let physical = self.translate(address, false)?;
        self.cop0[LINKED_ADDRESS] = (physical >> 4) as u64;
        Ok(())
    }

    pub(crate) fn store_conditional(&mut self, bus: &mut MemoryBus, i: u32, size: u32) -> Exec<u32> {
        let landed = if self.linked_flag { self.store(bus, i, size)? } else { NOT_STORED };
        self.write(rt(i), self.linked_flag as u64);
        self.linked_flag = false;
        Ok(landed)
    }

    /// `Cache`: nothing is cached, but the address is still checked.
    pub(crate) fn cache(&mut self, i: u32) -> Exec {
        let address = self.effective_address(i);
        self.require_alignment(address, 4, code::ADDRESS_ERROR_LOAD)?;
        self.translate(address, false)?;
        Ok(())
    }

    pub(crate) fn load_word_left(&mut self, bus: &mut MemoryBus, i: u32) -> Exec {
        let address = self.effective_address(i);
        let access = self.mirrored(address, 1);
        let shift = (access & 3) as u32 * 8;
        let physical = self.translate_access(access & !3, address, false)?;
        let word = bus.read32(physical);
        let kept = if shift == 0 { 0 } else { (self.read(rt(i)) as u32) & ((1u32 << shift) - 1) };
        self.write32(rt(i), (word << shift) | kept);
        Ok(())
    }

    pub(crate) fn load_word_right(&mut self, bus: &mut MemoryBus, i: u32) -> Exec {
        let address = self.effective_address(i);
        let access = self.mirrored(address, 1);
        let shift = (3 - (access & 3) as u32) * 8;
        let physical = self.translate_access(access & !3, address, false)?;
        let word = bus.read32(physical);
        let previous = self.read(rt(i));
        let merged = (word >> shift) | ((previous as u32) & !(0xFFFF_FFFFu32 >> shift));
        if shift == 0 {
            self.write32(rt(i), merged);
        } else {
            self.write(rt(i), (previous & 0xFFFF_FFFF_0000_0000) | merged as u64);
        }
        Ok(())
    }

    pub(crate) fn load_double_left(&mut self, bus: &mut MemoryBus, i: u32) -> Exec {
        let address = self.effective_address(i);
        let access = self.mirrored(address, 1);
        let shift = (access & 7) as u32 * 8;
        let physical = self.translate_access(access & !7, address, false)?;
        let value = bus.read64(physical);
        let kept = if shift == 0 { 0 } else { self.read(rt(i)) & ((1u64 << shift) - 1) };
        self.write(rt(i), (value << shift) | kept);
        Ok(())
    }

    pub(crate) fn load_double_right(&mut self, bus: &mut MemoryBus, i: u32) -> Exec {
        let address = self.effective_address(i);
        let access = self.mirrored(address, 1);
        let shift = (7 - (access & 7) as u32) * 8;
        let physical = self.translate_access(access & !7, address, false)?;
        let value = bus.read64(physical);
        let kept = self.read(rt(i)) & !(u64::MAX >> shift);
        self.write(rt(i), (value >> shift) | kept);
        Ok(())
    }

    pub(crate) fn store_word_left(&mut self, bus: &mut MemoryBus, i: u32) -> Exec<u32> {
        let address = self.effective_address(i);
        let access = self.mirrored(address, 1);
        let shift = (access & 3) as u32 * 8;
        let physical = self.translate_access(access & !3, address, true)?;
        let kept = bus.read32(physical) & !(0xFFFF_FFFFu32 >> shift);
        bus.write32(physical, kept | ((self.read(rt(i)) as u32) >> shift));
        Ok(landed(bus, physical))
    }

    pub(crate) fn store_word_right(&mut self, bus: &mut MemoryBus, i: u32) -> Exec<u32> {
        let address = self.effective_address(i);
        let access = self.mirrored(address, 1);
        let shift = (3 - (access & 3) as u32) * 8;
        let physical = self.translate_access(access & !3, address, true)?;
        let kept = if shift == 0 { 0 } else { bus.read32(physical) & ((1u32 << shift) - 1) };
        bus.write32(physical, kept | ((self.read(rt(i)) as u32) << shift));
        Ok(landed(bus, physical))
    }

    pub(crate) fn store_double_left(&mut self, bus: &mut MemoryBus, i: u32) -> Exec<u32> {
        let address = self.effective_address(i);
        let access = self.mirrored(address, 1);
        let shift = (access & 7) as u32 * 8;
        let physical = self.translate_access(access & !7, address, true)?;
        let kept = bus.read64(physical) & !(u64::MAX >> shift);
        bus.write64(physical, kept | (self.read(rt(i)) >> shift));
        Ok(landed(bus, physical))
    }

    pub(crate) fn store_double_right(&mut self, bus: &mut MemoryBus, i: u32) -> Exec<u32> {
        let address = self.effective_address(i);
        let access = self.mirrored(address, 1);
        let shift = (7 - (access & 7) as u32) * 8;
        let physical = self.translate_access(access & !7, address, true)?;
        let kept = if shift == 0 { 0 } else { bus.read64(physical) & ((1u64 << shift) - 1) };
        bus.write64(physical, kept | (self.read(rt(i)) << shift));
        Ok(landed(bus, physical))
    }

    pub(crate) fn multiply(&mut self, i: u32, unsigned: bool) {
        let result: i64 = if unsigned {
            ((self.read(rs(i)) as u32 as u64) * (self.read(rt(i)) as u32 as u64)) as i64
        } else {
            (self.read(rs(i)) as i64).wrapping_mul(sign_extend_operand(self.read(rt(i))))
        };
        self.lo = result as i32 as i64 as u64;
        self.hi = (result >> 32) as i32 as i64 as u64;
        self.extra_cycles = MULTIPLY_STALL;
    }

    pub(crate) fn multiply_double(&mut self, i: u32, unsigned: bool) {
        let left = self.read(rs(i));
        let right = self.read(rt(i));
        if unsigned {
            let product = left as u128 * right as u128;
            self.lo = product as u64;
            self.hi = (product >> 64) as u64;
        } else {
            let product = (left as i64 as i128) * (right as i64 as i128);
            self.lo = product as u64;
            self.hi = (product >> 64) as u64;
        }
        self.extra_cycles = MULTIPLY_DOUBLE_STALL;
    }

    /// Division by zero and the one overflowing case have defined results rather than an exception.
    pub(crate) fn divide(&mut self, i: u32, unsigned: bool) {
        self.extra_cycles = DIVIDE_STALL;
        if unsigned {
            let left = self.read(rs(i)) as u32;
            let right = self.read(rt(i)) as u32;
            if right == 0 {
                self.lo = u64::MAX;
                self.hi = left as i32 as i64 as u64;
                return;
            }
            self.lo = (left / right) as i32 as i64 as u64;
            self.hi = (left % right) as i32 as i64 as u64;
            return;
        }
        let left = self.read(rs(i)) as u32 as i32;
        let right = self.read(rt(i)) as u32 as i32;
        if right == 0 {
            self.lo = if left < 0 { 1 } else { u64::MAX };
            self.hi = left as i64 as u64;
            return;
        }
        if left == i32::MIN && right == -1 {
            self.lo = i32::MIN as i64 as u64;
            self.hi = 0;
            return;
        }
        self.lo = (left / right) as i64 as u64;
        self.hi = (left % right) as i64 as u64;
    }

    pub(crate) fn divide_double(&mut self, i: u32, unsigned: bool) {
        self.extra_cycles = DIVIDE_DOUBLE_STALL;
        if unsigned {
            let left = self.read(rs(i));
            let right = self.read(rt(i));
            if right == 0 {
                self.lo = u64::MAX;
                self.hi = left;
                return;
            }
            self.lo = left / right;
            self.hi = left % right;
            return;
        }
        let left = self.read(rs(i)) as i64;
        let right = self.read(rt(i)) as i64;
        if right == 0 {
            self.lo = if left < 0 { 1 } else { u64::MAX };
            self.hi = left as u64;
            return;
        }
        if left == i64::MIN && right == -1 {
            self.lo = i64::MIN as u64;
            self.hi = 0;
            return;
        }
        self.lo = (left / right) as u64;
        self.hi = (left % right) as u64;
    }

    pub(crate) fn execute_trap(&mut self, i: u32) -> Exec {
        let left = self.read(rs(i));
        let right = self.read(rt(i));
        let condition = match i & 0x3F {
            0x30 => left as i64 >= right as i64,
            0x31 => left >= right,
            0x32 => (left as i64) < right as i64,
            0x33 => left < right,
            0x34 => left == right,
            _ => left != right,
        };
        self.trap_if(condition)
    }

    /// The immediate sign-extends to sixty-four bits even where the comparison is unsigned.
    pub(crate) fn execute_trap_immediate(&mut self, i: u32) -> Exec {
        let left = self.read(rs(i));
        let right = signed_immediate(i);
        let condition = match rt(i) {
            0x08 => left as i64 >= right,
            0x09 => left >= right as u64,
            0x0A => (left as i64) < right,
            0x0B => left < right as u64,
            0x0C => left == right as u64,
            _ => left != right as u64,
        };
        self.trap_if(condition)
    }

    #[inline(always)]
    fn trap_if(&mut self, condition: bool) -> Exec {
        if condition {
            return Err(self.raise(code::TRAP, self.current_pc));
        }
        Ok(())
    }

    /// `ExecuteCop2`: one latch behind a usability bit.
    pub(crate) fn execute_cop2(&mut self, i: u32) -> Exec {
        if self.cop0[STATUS] & STATUS_COP2_USABLE == 0 {
            return Err(self.raise_with(code::COPROCESSOR_UNUSABLE, self.current_pc, false, 2));
        }
        match rs(i) {
            0x00 => self.write32(rt(i), self.cop2_latch as u32),
            0x01 => self.write(rt(i), self.cop2_latch),
            0x02 => self.write32(rt(i), 0),
            0x06 => {}
            0x04 | 0x05 => self.cop2_latch = self.read(rt(i)),
            _ => return Err(self.raise_with(code::RESERVED_INSTRUCTION, self.current_pc, false, 2)),
        }
        Ok(())
    }

    /// `LoadCop1`.
    pub(crate) fn load_cop1(&mut self, bus: &mut MemoryBus, i: u32, wide: bool) -> Exec {
        self.require_cop1()?;
        let size = if wide { 8 } else { 4 };
        let address = self.effective_address(i);
        self.require_alignment(address, size, code::ADDRESS_ERROR_LOAD)?;
        let physical = self.translate_access(self.mirrored(address, size), address, false)?;
        if wide {
            let value = bus.read64(physical);
            self.write_fpu_wide(rt(i), value);
        } else {
            let value = bus.read32(physical);
            self.write_fpu_word(rt(i), value);
        }
        Ok(())
    }

    /// `StoreCop1`: through the bus, never the direct path.
    pub(crate) fn store_cop1(&mut self, bus: &mut MemoryBus, i: u32, wide: bool) -> Exec<u32> {
        self.require_cop1()?;
        let size = if wide { 8 } else { 4 };
        let address = self.effective_address(i);
        self.require_alignment(address, size, code::ADDRESS_ERROR_STORE)?;
        let physical = self.translate_access(self.mirrored(address, size), address, true)?;
        if wide {
            bus.write64(physical, self.read_fpu_wide(rt(i)));
        } else {
            bus.write32(physical, self.read_fpu_word(rt(i)));
        }
        Ok(landed(bus, physical))
    }
}

/// What a store reports when it went through the bus's own `Store` (a device, the cartridge, the MI's repeat), which a block leaves after.
pub const THROUGH_BUS: u32 = u32::MAX;
/// What a store conditional reports when its link had gone and nothing was written.
pub const NOT_STORED: u32 = u32::MAX - 1;

/// Where a store through `write32` landed: RDRAM's own bytes, or somewhere a device may answer.
#[inline(always)]
fn landed(bus: &MemoryBus, physical: u32) -> u32 {
    if (physical as usize) < bus.rdram.len() { physical } else { THROUGH_BUS }
}

#[inline(always)]
pub(crate) fn sign_extend_operand(value: u64) -> i64 {
    ((value << (64 - MULTIPLY_OPERAND_BITS)) as i64) >> (64 - MULTIPLY_OPERAND_BITS)
}
