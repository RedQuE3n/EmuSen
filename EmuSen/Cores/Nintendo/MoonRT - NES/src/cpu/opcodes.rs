//! C#'s `Cpu.OpcodeTable.cs`, `Cpu.AddressModes.cs` and the `Cpu.Opcodes.*.cs` files: the whole decode, dummy reads included. See Moon_CPU.md §3, §6.

use super::*;

impl Cpu {
    #[inline(always)]
    fn consume_implied(&mut self, bus: &mut MemoryBus) {
        self.read(bus, self.pc);
    }

    #[inline(always)]
    fn addr_immediate(&mut self) -> u16 {
        let pc = self.pc;
        self.pc = pc.wrapping_add(1);
        pc
    }

    #[inline(always)]
    fn fetch(&mut self, bus: &mut MemoryBus) -> u8 {
        let pc = self.pc;
        self.pc = pc.wrapping_add(1);
        self.read(bus, pc)
    }

    #[inline(always)]
    fn addr_zero_page(&mut self, bus: &mut MemoryBus) -> u16 {
        self.fetch(bus) as u16
    }

    #[inline(always)]
    fn addr_zero_page_indexed(&mut self, bus: &mut MemoryBus, index: u8) -> u16 {
        let pointer = self.fetch(bus);
        self.read(bus, pointer as u16);
        pointer.wrapping_add(index) as u16
    }

    #[inline(always)]
    fn addr_absolute(&mut self, bus: &mut MemoryBus) -> u16 {
        let lo = self.fetch(bus) as u16;
        let hi = self.fetch(bus) as u16;
        lo | (hi << 8)
    }

    /// `always_fixup` for writes and read-modify-writes, which pay the fixup cycle without a page cross.
    #[inline(always)]
    fn addr_absolute_indexed(&mut self, bus: &mut MemoryBus, index: u8, always_fixup: bool) -> u16 {
        let base = self.addr_absolute(bus);
        let effective = base.wrapping_add(index as u16);
        if always_fixup || (effective & 0xFF00) != (base & 0xFF00) {
            self.read(bus, (base & 0xFF00) | (effective & 0x00FF));
        }
        effective
    }

    #[inline(always)]
    fn addr_indexed_indirect(&mut self, bus: &mut MemoryBus) -> u16 {
        let pointer = self.fetch(bus);
        self.read(bus, pointer as u16);
        let indexed = pointer.wrapping_add(self.x);
        let lo = self.read(bus, indexed as u16) as u16;
        let hi = self.read(bus, indexed.wrapping_add(1) as u16) as u16;
        lo | (hi << 8)
    }

    #[inline(always)]
    fn addr_indirect_indexed(&mut self, bus: &mut MemoryBus, always_fixup: bool) -> u16 {
        let pointer = self.fetch(bus);
        let lo = self.read(bus, pointer as u16) as u16;
        let hi = self.read(bus, pointer.wrapping_add(1) as u16) as u16;
        let base = lo | (hi << 8);
        let effective = base.wrapping_add(self.y as u16);
        if always_fixup || (effective & 0xFF00) != (base & 0xFF00) {
            self.read(bus, (base & 0xFF00) | (effective & 0x00FF));
        }
        effective
    }

    /// The base's high byte, which the unstable stores AND against - see Moon_CPU.md §6.3.
    fn addr_absolute_indexed_unstable(&mut self, bus: &mut MemoryBus, index: u8) -> (u16, u8, bool) {
        let lo = self.fetch(bus) as u16;
        let hi = self.fetch(bus);
        let base = lo | ((hi as u16) << 8);
        let effective = base.wrapping_add(index as u16);
        let crossed = (effective & 0xFF00) != (base & 0xFF00);
        self.read(bus, (base & 0xFF00) | (effective & 0x00FF));
        (effective, hi, crossed)
    }

    fn addr_indirect_indexed_unstable(&mut self, bus: &mut MemoryBus) -> (u16, u8, bool) {
        let pointer = self.fetch(bus);
        let lo = self.read(bus, pointer as u16) as u16;
        let hi = self.read(bus, pointer.wrapping_add(1) as u16);
        let base = lo | ((hi as u16) << 8);
        let effective = base.wrapping_add(self.y as u16);
        let crossed = (effective & 0xFF00) != (base & 0xFF00);
        self.read(bus, (base & 0xFF00) | (effective & 0x00FF));
        (effective, hi, crossed)
    }

    /// The untouched value goes back before the modified one - see Moon_CPU.md §3.3.
    #[inline(always)]
    fn rmw_fetch(&mut self, bus: &mut MemoryBus, address: u16) -> u8 {
        let value = self.read(bus, address);
        self.write(bus, address, value);
        value
    }

    #[inline(always)]
    fn load(&mut self, bus: &mut MemoryBus, address: u16) -> u8 {
        self.read(bus, address)
    }

    // Arithmetic, logic, shifts.

    #[inline(always)]
    fn op_adc(&mut self, operand: u8) {
        let sum = self.a as i32 + operand as i32 + if self.flag(FLAG_C) { 1 } else { 0 };
        let result = sum as u8;
        self.set_flag(FLAG_C, sum > 0xFF);
        self.set_flag(FLAG_V, ((self.a ^ result) & (operand ^ result) & 0x80) != 0);
        self.a = self.set_zero_negative(result);
    }

    #[inline(always)]
    fn op_sbc(&mut self, operand: u8) {
        self.op_adc(!operand);
    }

    #[inline(always)]
    fn compare(&mut self, register: u8, operand: u8) {
        self.set_flag(FLAG_C, register >= operand);
        self.set_zero_negative(register.wrapping_sub(operand));
    }

    fn op_inc(&mut self, bus: &mut MemoryBus, address: u16) {
        let value = self.rmw_fetch(bus, address);
        let result = self.set_zero_negative(value.wrapping_add(1));
        self.write(bus, address, result);
    }

    fn op_dec(&mut self, bus: &mut MemoryBus, address: u16) {
        let value = self.rmw_fetch(bus, address);
        let result = self.set_zero_negative(value.wrapping_sub(1));
        self.write(bus, address, result);
    }

    #[inline(always)]
    fn op_and(&mut self, operand: u8) {
        self.a = self.set_zero_negative(self.a & operand);
    }

    #[inline(always)]
    fn op_ora(&mut self, operand: u8) {
        self.a = self.set_zero_negative(self.a | operand);
    }

    #[inline(always)]
    fn op_eor(&mut self, operand: u8) {
        self.a = self.set_zero_negative(self.a ^ operand);
    }

    fn op_bit(&mut self, operand: u8) {
        self.set_flag(FLAG_Z, (self.a & operand) == 0);
        self.set_flag(FLAG_N, (operand & 0x80) != 0);
        self.set_flag(FLAG_V, (operand & 0x40) != 0);
    }

    #[inline(always)]
    fn shift_left(&mut self, value: u8) -> u8 {
        self.set_flag(FLAG_C, (value & 0x80) != 0);
        self.set_zero_negative(value << 1)
    }

    #[inline(always)]
    fn shift_right(&mut self, value: u8) -> u8 {
        self.set_flag(FLAG_C, (value & 0x01) != 0);
        self.set_zero_negative(value >> 1)
    }

    #[inline(always)]
    fn rotate_left(&mut self, value: u8) -> u8 {
        let carry_in = if self.flag(FLAG_C) { 0x01 } else { 0x00 };
        self.set_flag(FLAG_C, (value & 0x80) != 0);
        self.set_zero_negative((value << 1) | carry_in)
    }

    #[inline(always)]
    fn rotate_right(&mut self, value: u8) -> u8 {
        let carry_in = if self.flag(FLAG_C) { 0x80 } else { 0x00 };
        self.set_flag(FLAG_C, (value & 0x01) != 0);
        self.set_zero_negative((value >> 1) | carry_in)
    }

    fn op_asl(&mut self, bus: &mut MemoryBus, address: u16) {
        let v = self.rmw_fetch(bus, address);
        let r = self.shift_left(v);
        self.write(bus, address, r);
    }

    fn op_lsr(&mut self, bus: &mut MemoryBus, address: u16) {
        let v = self.rmw_fetch(bus, address);
        let r = self.shift_right(v);
        self.write(bus, address, r);
    }

    fn op_rol(&mut self, bus: &mut MemoryBus, address: u16) {
        let v = self.rmw_fetch(bus, address);
        let r = self.rotate_left(v);
        self.write(bus, address, r);
    }

    fn op_ror(&mut self, bus: &mut MemoryBus, address: u16) {
        let v = self.rmw_fetch(bus, address);
        let r = self.rotate_right(v);
        self.write(bus, address, r);
    }

    // Branches, flags, stack, system.

    fn branch(&mut self, bus: &mut MemoryBus, taken: bool) {
        let offset = self.fetch(bus) as i8;
        if !taken {
            return;
        }
        self.suppress_just_arrived_irq();
        self.read(bus, self.pc);
        let target = self.pc.wrapping_add(offset as i16 as u16);
        if (target & 0xFF00) != (self.pc & 0xFF00) {
            self.read(bus, (self.pc & 0xFF00) | (target & 0x00FF));
        }
        self.pc = target;
    }

    fn set_flag_opcode(&mut self, bus: &mut MemoryBus, flag: u8, value: bool) {
        self.consume_implied(bus);
        self.set_flag(flag, value);
    }

    /// CLI and SEI land their I after the interrupt poll.
    fn set_interrupt_disable(&mut self, bus: &mut MemoryBus, value: bool) {
        self.consume_implied(bus);
        self.delayed_i = value;
        self.has_delayed_i = true;
    }

    fn op_pha(&mut self, bus: &mut MemoryBus) {
        self.consume_implied(bus);
        self.push(bus, self.a);
    }

    fn op_php(&mut self, bus: &mut MemoryBus) {
        self.consume_implied(bus);
        self.push(bus, self.p | 0x30);
    }

    fn op_pla(&mut self, bus: &mut MemoryBus) {
        self.consume_implied(bus);
        let v = self.pull_with_dummy(bus);
        self.a = self.set_zero_negative(v);
    }

    fn op_plp(&mut self, bus: &mut MemoryBus) {
        self.consume_implied(bus);
        let pulled = self.pull_with_dummy(bus);
        let previous_i = self.flag(FLAG_I);
        self.delayed_i = (pulled & FLAG_I) != 0;
        self.has_delayed_i = true;
        self.p = (pulled | FLAG_U) & !FLAG_B;
        self.set_flag(FLAG_I, previous_i);
    }

    fn op_brk(&mut self, bus: &mut MemoryBus) {
        self.fetch(bus);
        self.push(bus, (self.pc >> 8) as u8);
        self.push(bus, self.pc as u8);
        self.push(bus, self.p | 0x30);
        self.set_flag(FLAG_I, true);
        self.pc = self.read_vector(bus, IRQ_VECTOR);
    }

    fn op_rti(&mut self, bus: &mut MemoryBus) {
        self.consume_implied(bus);
        let pulled = self.pull_with_dummy(bus);
        self.p = (pulled | FLAG_U) & !FLAG_B;
        let lo = self.pull(bus) as u16;
        let hi = self.pull(bus) as u16;
        self.pc = lo | (hi << 8);
    }

    fn op_jsr(&mut self, bus: &mut MemoryBus) {
        let lo = self.fetch(bus) as u16;
        self.read(bus, 0x0100 | self.s as u16);
        self.push(bus, (self.pc >> 8) as u8);
        self.push(bus, self.pc as u8);
        let hi = self.read(bus, self.pc) as u16;
        self.pc = lo | (hi << 8);
    }

    fn op_rts(&mut self, bus: &mut MemoryBus) {
        self.consume_implied(bus);
        let lo = self.pull_with_dummy(bus) as u16;
        let hi = self.pull(bus) as u16;
        self.pc = lo | (hi << 8);
        self.read(bus, self.pc);
        self.pc = self.pc.wrapping_add(1);
    }

    fn op_jmp_indirect(&mut self, bus: &mut MemoryBus) {
        let pointer = self.addr_absolute(bus);
        let lo = self.read(bus, pointer) as u16;
        let hi = self.read(bus, (pointer & 0xFF00) | (pointer.wrapping_add(1) & 0x00FF)) as u16;
        self.pc = lo | (hi << 8);
    }

    fn op_jam(&mut self, bus: &mut MemoryBus) {
        self.read(bus, self.pc);
        self.read(bus, self.pc);
        self.pc = self.pc.wrapping_sub(1);
        self.jammed = true;
    }

    // The undocumented opcodes.

    fn op_slo(&mut self, bus: &mut MemoryBus, address: u16) {
        let v = self.rmw_fetch(bus, address);
        let value = self.shift_left(v);
        self.write(bus, address, value);
        self.op_ora(value);
    }

    fn op_rla(&mut self, bus: &mut MemoryBus, address: u16) {
        let v = self.rmw_fetch(bus, address);
        let value = self.rotate_left(v);
        self.write(bus, address, value);
        self.op_and(value);
    }

    fn op_sre(&mut self, bus: &mut MemoryBus, address: u16) {
        let v = self.rmw_fetch(bus, address);
        let value = self.shift_right(v);
        self.write(bus, address, value);
        self.op_eor(value);
    }

    fn op_rra(&mut self, bus: &mut MemoryBus, address: u16) {
        let v = self.rmw_fetch(bus, address);
        let value = self.rotate_right(v);
        self.write(bus, address, value);
        self.op_adc(value);
    }

    fn op_dcp(&mut self, bus: &mut MemoryBus, address: u16) {
        let value = self.rmw_fetch(bus, address).wrapping_sub(1);
        self.write(bus, address, value);
        self.compare(self.a, value);
    }

    fn op_isc(&mut self, bus: &mut MemoryBus, address: u16) {
        let value = self.rmw_fetch(bus, address).wrapping_add(1);
        self.write(bus, address, value);
        self.op_sbc(value);
    }

    fn op_sax(&mut self, bus: &mut MemoryBus, address: u16) {
        self.write(bus, address, self.a & self.x);
    }

    fn op_lax(&mut self, operand: u8) {
        self.a = self.set_zero_negative(operand);
        self.x = self.a;
    }

    fn op_anc(&mut self, operand: u8) {
        self.op_and(operand);
        let n = self.flag(FLAG_N);
        self.set_flag(FLAG_C, n);
    }

    fn op_alr(&mut self, operand: u8) {
        self.op_and(operand);
        self.a = self.shift_right(self.a);
    }

    fn op_arr(&mut self, operand: u8) {
        let masked = self.a & operand;
        let result = (masked >> 1) | if self.flag(FLAG_C) { 0x80 } else { 0x00 };
        self.set_zero_negative(result);
        self.set_flag(FLAG_C, (result & 0x40) != 0);
        self.set_flag(FLAG_V, (((result >> 6) ^ (result >> 5)) & 0x01) != 0);
        self.a = result;
    }

    fn op_sbx(&mut self, operand: u8) {
        let masked = self.a & self.x;
        self.set_flag(FLAG_C, masked >= operand);
        self.x = self.set_zero_negative(masked.wrapping_sub(operand));
    }

    fn op_ane(&mut self, operand: u8) {
        self.a = self.set_zero_negative((self.a | UNSTABLE_MAGIC) & self.x & operand);
    }

    fn op_lxa(&mut self, operand: u8) {
        self.a = self.set_zero_negative((self.a | UNSTABLE_MAGIC) & operand);
        self.x = self.a;
    }

    fn op_las(&mut self, operand: u8) {
        self.s = self.set_zero_negative(operand & self.s);
        self.a = self.s;
        self.x = self.s;
    }

    fn unstable_store(&mut self, bus: &mut MemoryBus, value: u8, effective: u16, base_high: u8, crossed: bool) {
        let stored = value & base_high.wrapping_add(1);
        let address = if crossed { ((stored as u16) << 8) | (effective & 0x00FF) } else { effective };
        self.write(bus, address, stored);
    }

    pub(super) fn dispatch(&mut self, bus: &mut MemoryBus, opcode: u8) {
        let (x, y) = (self.x, self.y);
        match opcode {
            0x00 => self.op_brk(bus),
            0x01 => { let a = self.addr_indexed_indirect(bus); let v = self.load(bus, a); self.op_ora(v) }
            0x02 => self.op_jam(bus),
            0x03 => { let a = self.addr_indexed_indirect(bus); self.op_slo(bus, a) }
            0x04 => { let a = self.addr_zero_page(bus); self.load(bus, a); }
            0x05 => { let a = self.addr_zero_page(bus); let v = self.load(bus, a); self.op_ora(v) }
            0x06 => { let a = self.addr_zero_page(bus); self.op_asl(bus, a) }
            0x07 => { let a = self.addr_zero_page(bus); self.op_slo(bus, a) }
            0x08 => self.op_php(bus),
            0x09 => { let a = self.addr_immediate(); let v = self.load(bus, a); self.op_ora(v) }
            0x0A => { self.consume_implied(bus); self.a = self.shift_left(self.a) }
            0x0B => { let a = self.addr_immediate(); let v = self.load(bus, a); self.op_anc(v) }
            0x0C => { let a = self.addr_absolute(bus); self.load(bus, a); }
            0x0D => { let a = self.addr_absolute(bus); let v = self.load(bus, a); self.op_ora(v) }
            0x0E => { let a = self.addr_absolute(bus); self.op_asl(bus, a) }
            0x0F => { let a = self.addr_absolute(bus); self.op_slo(bus, a) }

            0x10 => self.branch(bus, !self.flag(FLAG_N)),
            0x11 => { let a = self.addr_indirect_indexed(bus, false); let v = self.load(bus, a); self.op_ora(v) }
            0x12 => self.op_jam(bus),
            0x13 => { let a = self.addr_indirect_indexed(bus, true); self.op_slo(bus, a) }
            0x14 => { let a = self.addr_zero_page_indexed(bus, x); self.load(bus, a); }
            0x15 => { let a = self.addr_zero_page_indexed(bus, x); let v = self.load(bus, a); self.op_ora(v) }
            0x16 => { let a = self.addr_zero_page_indexed(bus, x); self.op_asl(bus, a) }
            0x17 => { let a = self.addr_zero_page_indexed(bus, x); self.op_slo(bus, a) }
            0x18 => self.set_flag_opcode(bus, FLAG_C, false),
            0x19 => { let a = self.addr_absolute_indexed(bus, y, false); let v = self.load(bus, a); self.op_ora(v) }
            0x1A => self.consume_implied(bus),
            0x1B => { let a = self.addr_absolute_indexed(bus, y, true); self.op_slo(bus, a) }
            0x1C => { let a = self.addr_absolute_indexed(bus, x, false); self.load(bus, a); }
            0x1D => { let a = self.addr_absolute_indexed(bus, x, false); let v = self.load(bus, a); self.op_ora(v) }
            0x1E => { let a = self.addr_absolute_indexed(bus, x, true); self.op_asl(bus, a) }
            0x1F => { let a = self.addr_absolute_indexed(bus, x, true); self.op_slo(bus, a) }

            0x20 => self.op_jsr(bus),
            0x21 => { let a = self.addr_indexed_indirect(bus); let v = self.load(bus, a); self.op_and(v) }
            0x22 => self.op_jam(bus),
            0x23 => { let a = self.addr_indexed_indirect(bus); self.op_rla(bus, a) }
            0x24 => { let a = self.addr_zero_page(bus); let v = self.load(bus, a); self.op_bit(v) }
            0x25 => { let a = self.addr_zero_page(bus); let v = self.load(bus, a); self.op_and(v) }
            0x26 => { let a = self.addr_zero_page(bus); self.op_rol(bus, a) }
            0x27 => { let a = self.addr_zero_page(bus); self.op_rla(bus, a) }
            0x28 => self.op_plp(bus),
            0x29 => { let a = self.addr_immediate(); let v = self.load(bus, a); self.op_and(v) }
            0x2A => { self.consume_implied(bus); self.a = self.rotate_left(self.a) }
            0x2B => { let a = self.addr_immediate(); let v = self.load(bus, a); self.op_anc(v) }
            0x2C => { let a = self.addr_absolute(bus); let v = self.load(bus, a); self.op_bit(v) }
            0x2D => { let a = self.addr_absolute(bus); let v = self.load(bus, a); self.op_and(v) }
            0x2E => { let a = self.addr_absolute(bus); self.op_rol(bus, a) }
            0x2F => { let a = self.addr_absolute(bus); self.op_rla(bus, a) }

            0x30 => self.branch(bus, self.flag(FLAG_N)),
            0x31 => { let a = self.addr_indirect_indexed(bus, false); let v = self.load(bus, a); self.op_and(v) }
            0x32 => self.op_jam(bus),
            0x33 => { let a = self.addr_indirect_indexed(bus, true); self.op_rla(bus, a) }
            0x34 => { let a = self.addr_zero_page_indexed(bus, x); self.load(bus, a); }
            0x35 => { let a = self.addr_zero_page_indexed(bus, x); let v = self.load(bus, a); self.op_and(v) }
            0x36 => { let a = self.addr_zero_page_indexed(bus, x); self.op_rol(bus, a) }
            0x37 => { let a = self.addr_zero_page_indexed(bus, x); self.op_rla(bus, a) }
            0x38 => self.set_flag_opcode(bus, FLAG_C, true),
            0x39 => { let a = self.addr_absolute_indexed(bus, y, false); let v = self.load(bus, a); self.op_and(v) }
            0x3A => self.consume_implied(bus),
            0x3B => { let a = self.addr_absolute_indexed(bus, y, true); self.op_rla(bus, a) }
            0x3C => { let a = self.addr_absolute_indexed(bus, x, false); self.load(bus, a); }
            0x3D => { let a = self.addr_absolute_indexed(bus, x, false); let v = self.load(bus, a); self.op_and(v) }
            0x3E => { let a = self.addr_absolute_indexed(bus, x, true); self.op_rol(bus, a) }
            0x3F => { let a = self.addr_absolute_indexed(bus, x, true); self.op_rla(bus, a) }

            0x40 => self.op_rti(bus),
            0x41 => { let a = self.addr_indexed_indirect(bus); let v = self.load(bus, a); self.op_eor(v) }
            0x42 => self.op_jam(bus),
            0x43 => { let a = self.addr_indexed_indirect(bus); self.op_sre(bus, a) }
            0x44 => { let a = self.addr_zero_page(bus); self.load(bus, a); }
            0x45 => { let a = self.addr_zero_page(bus); let v = self.load(bus, a); self.op_eor(v) }
            0x46 => { let a = self.addr_zero_page(bus); self.op_lsr(bus, a) }
            0x47 => { let a = self.addr_zero_page(bus); self.op_sre(bus, a) }
            0x48 => self.op_pha(bus),
            0x49 => { let a = self.addr_immediate(); let v = self.load(bus, a); self.op_eor(v) }
            0x4A => { self.consume_implied(bus); self.a = self.shift_right(self.a) }
            0x4B => { let a = self.addr_immediate(); let v = self.load(bus, a); self.op_alr(v) }
            0x4C => self.pc = self.addr_absolute(bus),
            0x4D => { let a = self.addr_absolute(bus); let v = self.load(bus, a); self.op_eor(v) }
            0x4E => { let a = self.addr_absolute(bus); self.op_lsr(bus, a) }
            0x4F => { let a = self.addr_absolute(bus); self.op_sre(bus, a) }

            0x50 => self.branch(bus, !self.flag(FLAG_V)),
            0x51 => { let a = self.addr_indirect_indexed(bus, false); let v = self.load(bus, a); self.op_eor(v) }
            0x52 => self.op_jam(bus),
            0x53 => { let a = self.addr_indirect_indexed(bus, true); self.op_sre(bus, a) }
            0x54 => { let a = self.addr_zero_page_indexed(bus, x); self.load(bus, a); }
            0x55 => { let a = self.addr_zero_page_indexed(bus, x); let v = self.load(bus, a); self.op_eor(v) }
            0x56 => { let a = self.addr_zero_page_indexed(bus, x); self.op_lsr(bus, a) }
            0x57 => { let a = self.addr_zero_page_indexed(bus, x); self.op_sre(bus, a) }
            0x58 => self.set_interrupt_disable(bus, false),
            0x59 => { let a = self.addr_absolute_indexed(bus, y, false); let v = self.load(bus, a); self.op_eor(v) }
            0x5A => self.consume_implied(bus),
            0x5B => { let a = self.addr_absolute_indexed(bus, y, true); self.op_sre(bus, a) }
            0x5C => { let a = self.addr_absolute_indexed(bus, x, false); self.load(bus, a); }
            0x5D => { let a = self.addr_absolute_indexed(bus, x, false); let v = self.load(bus, a); self.op_eor(v) }
            0x5E => { let a = self.addr_absolute_indexed(bus, x, true); self.op_lsr(bus, a) }
            0x5F => { let a = self.addr_absolute_indexed(bus, x, true); self.op_sre(bus, a) }

            0x60 => self.op_rts(bus),
            0x61 => { let a = self.addr_indexed_indirect(bus); let v = self.load(bus, a); self.op_adc(v) }
            0x62 => self.op_jam(bus),
            0x63 => { let a = self.addr_indexed_indirect(bus); self.op_rra(bus, a) }
            0x64 => { let a = self.addr_zero_page(bus); self.load(bus, a); }
            0x65 => { let a = self.addr_zero_page(bus); let v = self.load(bus, a); self.op_adc(v) }
            0x66 => { let a = self.addr_zero_page(bus); self.op_ror(bus, a) }
            0x67 => { let a = self.addr_zero_page(bus); self.op_rra(bus, a) }
            0x68 => self.op_pla(bus),
            0x69 => { let a = self.addr_immediate(); let v = self.load(bus, a); self.op_adc(v) }
            0x6A => { self.consume_implied(bus); self.a = self.rotate_right(self.a) }
            0x6B => { let a = self.addr_immediate(); let v = self.load(bus, a); self.op_arr(v) }
            0x6C => self.op_jmp_indirect(bus),
            0x6D => { let a = self.addr_absolute(bus); let v = self.load(bus, a); self.op_adc(v) }
            0x6E => { let a = self.addr_absolute(bus); self.op_ror(bus, a) }
            0x6F => { let a = self.addr_absolute(bus); self.op_rra(bus, a) }

            0x70 => self.branch(bus, self.flag(FLAG_V)),
            0x71 => { let a = self.addr_indirect_indexed(bus, false); let v = self.load(bus, a); self.op_adc(v) }
            0x72 => self.op_jam(bus),
            0x73 => { let a = self.addr_indirect_indexed(bus, true); self.op_rra(bus, a) }
            0x74 => { let a = self.addr_zero_page_indexed(bus, x); self.load(bus, a); }
            0x75 => { let a = self.addr_zero_page_indexed(bus, x); let v = self.load(bus, a); self.op_adc(v) }
            0x76 => { let a = self.addr_zero_page_indexed(bus, x); self.op_ror(bus, a) }
            0x77 => { let a = self.addr_zero_page_indexed(bus, x); self.op_rra(bus, a) }
            0x78 => self.set_interrupt_disable(bus, true),
            0x79 => { let a = self.addr_absolute_indexed(bus, y, false); let v = self.load(bus, a); self.op_adc(v) }
            0x7A => self.consume_implied(bus),
            0x7B => { let a = self.addr_absolute_indexed(bus, y, true); self.op_rra(bus, a) }
            0x7C => { let a = self.addr_absolute_indexed(bus, x, false); self.load(bus, a); }
            0x7D => { let a = self.addr_absolute_indexed(bus, x, false); let v = self.load(bus, a); self.op_adc(v) }
            0x7E => { let a = self.addr_absolute_indexed(bus, x, true); self.op_ror(bus, a) }
            0x7F => { let a = self.addr_absolute_indexed(bus, x, true); self.op_rra(bus, a) }

            0x80 | 0x82 | 0x89 | 0xC2 | 0xE2 => { let a = self.addr_immediate(); self.load(bus, a); }
            0x81 => { let a = self.addr_indexed_indirect(bus); self.write(bus, a, self.a) }
            0x83 => { let a = self.addr_indexed_indirect(bus); self.op_sax(bus, a) }
            0x84 => { let a = self.addr_zero_page(bus); self.write(bus, a, self.y) }
            0x85 => { let a = self.addr_zero_page(bus); self.write(bus, a, self.a) }
            0x86 => { let a = self.addr_zero_page(bus); self.write(bus, a, self.x) }
            0x87 => { let a = self.addr_zero_page(bus); self.op_sax(bus, a) }
            0x88 => { self.consume_implied(bus); self.y = self.set_zero_negative(self.y.wrapping_sub(1)) }
            0x8A => { self.consume_implied(bus); self.a = self.set_zero_negative(self.x) }
            0x8B => { let a = self.addr_immediate(); let v = self.load(bus, a); self.op_ane(v) }
            0x8C => { let a = self.addr_absolute(bus); self.write(bus, a, self.y) }
            0x8D => { let a = self.addr_absolute(bus); self.write(bus, a, self.a) }
            0x8E => { let a = self.addr_absolute(bus); self.write(bus, a, self.x) }
            0x8F => { let a = self.addr_absolute(bus); self.op_sax(bus, a) }

            0x90 => self.branch(bus, !self.flag(FLAG_C)),
            0x91 => { let a = self.addr_indirect_indexed(bus, true); self.write(bus, a, self.a) }
            0x92 => self.op_jam(bus),
            0x93 => { let (e, h, c) = self.addr_indirect_indexed_unstable(bus); self.unstable_store(bus, self.a & self.x, e, h, c) }
            0x94 => { let a = self.addr_zero_page_indexed(bus, x); self.write(bus, a, self.y) }
            0x95 => { let a = self.addr_zero_page_indexed(bus, x); self.write(bus, a, self.a) }
            0x96 => { let a = self.addr_zero_page_indexed(bus, y); self.write(bus, a, self.x) }
            0x97 => { let a = self.addr_zero_page_indexed(bus, y); self.op_sax(bus, a) }
            0x98 => { self.consume_implied(bus); self.a = self.set_zero_negative(self.y) }
            0x99 => { let a = self.addr_absolute_indexed(bus, y, true); self.write(bus, a, self.a) }
            0x9A => { self.consume_implied(bus); self.s = self.x }
            0x9B => {
                let (e, h, c) = self.addr_absolute_indexed_unstable(bus, y);
                self.s = self.a & self.x;
                self.unstable_store(bus, self.s, e, h, c)
            }
            0x9C => { let (e, h, c) = self.addr_absolute_indexed_unstable(bus, x); self.unstable_store(bus, self.y, e, h, c) }
            0x9D => { let a = self.addr_absolute_indexed(bus, x, true); self.write(bus, a, self.a) }
            0x9E => { let (e, h, c) = self.addr_absolute_indexed_unstable(bus, y); self.unstable_store(bus, self.x, e, h, c) }
            0x9F => { let (e, h, c) = self.addr_absolute_indexed_unstable(bus, y); self.unstable_store(bus, self.a & self.x, e, h, c) }

            0xA0 => { let a = self.addr_immediate(); let v = self.load(bus, a); self.y = self.set_zero_negative(v) }
            0xA1 => { let a = self.addr_indexed_indirect(bus); let v = self.load(bus, a); self.a = self.set_zero_negative(v) }
            0xA2 => { let a = self.addr_immediate(); let v = self.load(bus, a); self.x = self.set_zero_negative(v) }
            0xA3 => { let a = self.addr_indexed_indirect(bus); let v = self.load(bus, a); self.op_lax(v) }
            0xA4 => { let a = self.addr_zero_page(bus); let v = self.load(bus, a); self.y = self.set_zero_negative(v) }
            0xA5 => { let a = self.addr_zero_page(bus); let v = self.load(bus, a); self.a = self.set_zero_negative(v) }
            0xA6 => { let a = self.addr_zero_page(bus); let v = self.load(bus, a); self.x = self.set_zero_negative(v) }
            0xA7 => { let a = self.addr_zero_page(bus); let v = self.load(bus, a); self.op_lax(v) }
            0xA8 => { self.consume_implied(bus); self.y = self.set_zero_negative(self.a) }
            0xA9 => { let a = self.addr_immediate(); let v = self.load(bus, a); self.a = self.set_zero_negative(v) }
            0xAA => { self.consume_implied(bus); self.x = self.set_zero_negative(self.a) }
            0xAB => { let a = self.addr_immediate(); let v = self.load(bus, a); self.op_lxa(v) }
            0xAC => { let a = self.addr_absolute(bus); let v = self.load(bus, a); self.y = self.set_zero_negative(v) }
            0xAD => { let a = self.addr_absolute(bus); let v = self.load(bus, a); self.a = self.set_zero_negative(v) }
            0xAE => { let a = self.addr_absolute(bus); let v = self.load(bus, a); self.x = self.set_zero_negative(v) }
            0xAF => { let a = self.addr_absolute(bus); let v = self.load(bus, a); self.op_lax(v) }

            0xB0 => self.branch(bus, self.flag(FLAG_C)),
            0xB1 => { let a = self.addr_indirect_indexed(bus, false); let v = self.load(bus, a); self.a = self.set_zero_negative(v) }
            0xB2 => self.op_jam(bus),
            0xB3 => { let a = self.addr_indirect_indexed(bus, false); let v = self.load(bus, a); self.op_lax(v) }
            0xB4 => { let a = self.addr_zero_page_indexed(bus, x); let v = self.load(bus, a); self.y = self.set_zero_negative(v) }
            0xB5 => { let a = self.addr_zero_page_indexed(bus, x); let v = self.load(bus, a); self.a = self.set_zero_negative(v) }
            0xB6 => { let a = self.addr_zero_page_indexed(bus, y); let v = self.load(bus, a); self.x = self.set_zero_negative(v) }
            0xB7 => { let a = self.addr_zero_page_indexed(bus, y); let v = self.load(bus, a); self.op_lax(v) }
            0xB8 => self.set_flag_opcode(bus, FLAG_V, false),
            0xB9 => { let a = self.addr_absolute_indexed(bus, y, false); let v = self.load(bus, a); self.a = self.set_zero_negative(v) }
            0xBA => { self.consume_implied(bus); self.x = self.set_zero_negative(self.s) }
            0xBB => { let a = self.addr_absolute_indexed(bus, y, false); let v = self.load(bus, a); self.op_las(v) }
            0xBC => { let a = self.addr_absolute_indexed(bus, x, false); let v = self.load(bus, a); self.y = self.set_zero_negative(v) }
            0xBD => { let a = self.addr_absolute_indexed(bus, x, false); let v = self.load(bus, a); self.a = self.set_zero_negative(v) }
            0xBE => { let a = self.addr_absolute_indexed(bus, y, false); let v = self.load(bus, a); self.x = self.set_zero_negative(v) }
            0xBF => { let a = self.addr_absolute_indexed(bus, y, false); let v = self.load(bus, a); self.op_lax(v) }

            0xC0 => { let a = self.addr_immediate(); let v = self.load(bus, a); self.compare(self.y, v) }
            0xC1 => { let a = self.addr_indexed_indirect(bus); let v = self.load(bus, a); self.compare(self.a, v) }
            0xC3 => { let a = self.addr_indexed_indirect(bus); self.op_dcp(bus, a) }
            0xC4 => { let a = self.addr_zero_page(bus); let v = self.load(bus, a); self.compare(self.y, v) }
            0xC5 => { let a = self.addr_zero_page(bus); let v = self.load(bus, a); self.compare(self.a, v) }
            0xC6 => { let a = self.addr_zero_page(bus); self.op_dec(bus, a) }
            0xC7 => { let a = self.addr_zero_page(bus); self.op_dcp(bus, a) }
            0xC8 => { self.consume_implied(bus); self.y = self.set_zero_negative(self.y.wrapping_add(1)) }
            0xC9 => { let a = self.addr_immediate(); let v = self.load(bus, a); self.compare(self.a, v) }
            0xCA => { self.consume_implied(bus); self.x = self.set_zero_negative(self.x.wrapping_sub(1)) }
            0xCB => { let a = self.addr_immediate(); let v = self.load(bus, a); self.op_sbx(v) }
            0xCC => { let a = self.addr_absolute(bus); let v = self.load(bus, a); self.compare(self.y, v) }
            0xCD => { let a = self.addr_absolute(bus); let v = self.load(bus, a); self.compare(self.a, v) }
            0xCE => { let a = self.addr_absolute(bus); self.op_dec(bus, a) }
            0xCF => { let a = self.addr_absolute(bus); self.op_dcp(bus, a) }

            0xD0 => self.branch(bus, !self.flag(FLAG_Z)),
            0xD1 => { let a = self.addr_indirect_indexed(bus, false); let v = self.load(bus, a); self.compare(self.a, v) }
            0xD2 => self.op_jam(bus),
            0xD3 => { let a = self.addr_indirect_indexed(bus, true); self.op_dcp(bus, a) }
            0xD4 => { let a = self.addr_zero_page_indexed(bus, x); self.load(bus, a); }
            0xD5 => { let a = self.addr_zero_page_indexed(bus, x); let v = self.load(bus, a); self.compare(self.a, v) }
            0xD6 => { let a = self.addr_zero_page_indexed(bus, x); self.op_dec(bus, a) }
            0xD7 => { let a = self.addr_zero_page_indexed(bus, x); self.op_dcp(bus, a) }
            0xD8 => self.set_flag_opcode(bus, FLAG_D, false),
            0xD9 => { let a = self.addr_absolute_indexed(bus, y, false); let v = self.load(bus, a); self.compare(self.a, v) }
            0xDA => self.consume_implied(bus),
            0xDB => { let a = self.addr_absolute_indexed(bus, y, true); self.op_dcp(bus, a) }
            0xDC => { let a = self.addr_absolute_indexed(bus, x, false); self.load(bus, a); }
            0xDD => { let a = self.addr_absolute_indexed(bus, x, false); let v = self.load(bus, a); self.compare(self.a, v) }
            0xDE => { let a = self.addr_absolute_indexed(bus, x, true); self.op_dec(bus, a) }
            0xDF => { let a = self.addr_absolute_indexed(bus, x, true); self.op_dcp(bus, a) }

            0xE0 => { let a = self.addr_immediate(); let v = self.load(bus, a); self.compare(self.x, v) }
            0xE1 => { let a = self.addr_indexed_indirect(bus); let v = self.load(bus, a); self.op_sbc(v) }
            0xE3 => { let a = self.addr_indexed_indirect(bus); self.op_isc(bus, a) }
            0xE4 => { let a = self.addr_zero_page(bus); let v = self.load(bus, a); self.compare(self.x, v) }
            0xE5 => { let a = self.addr_zero_page(bus); let v = self.load(bus, a); self.op_sbc(v) }
            0xE6 => { let a = self.addr_zero_page(bus); self.op_inc(bus, a) }
            0xE7 => { let a = self.addr_zero_page(bus); self.op_isc(bus, a) }
            0xE8 => { self.consume_implied(bus); self.x = self.set_zero_negative(self.x.wrapping_add(1)) }
            0xE9 | 0xEB => { let a = self.addr_immediate(); let v = self.load(bus, a); self.op_sbc(v) }
            0xEA => self.consume_implied(bus),
            0xEC => { let a = self.addr_absolute(bus); let v = self.load(bus, a); self.compare(self.x, v) }
            0xED => { let a = self.addr_absolute(bus); let v = self.load(bus, a); self.op_sbc(v) }
            0xEE => { let a = self.addr_absolute(bus); self.op_inc(bus, a) }
            0xEF => { let a = self.addr_absolute(bus); self.op_isc(bus, a) }

            0xF0 => self.branch(bus, self.flag(FLAG_Z)),
            0xF1 => { let a = self.addr_indirect_indexed(bus, false); let v = self.load(bus, a); self.op_sbc(v) }
            0xF2 => self.op_jam(bus),
            0xF3 => { let a = self.addr_indirect_indexed(bus, true); self.op_isc(bus, a) }
            0xF4 => { let a = self.addr_zero_page_indexed(bus, x); self.load(bus, a); }
            0xF5 => { let a = self.addr_zero_page_indexed(bus, x); let v = self.load(bus, a); self.op_sbc(v) }
            0xF6 => { let a = self.addr_zero_page_indexed(bus, x); self.op_inc(bus, a) }
            0xF7 => { let a = self.addr_zero_page_indexed(bus, x); self.op_isc(bus, a) }
            0xF8 => self.set_flag_opcode(bus, FLAG_D, true),
            0xF9 => { let a = self.addr_absolute_indexed(bus, y, false); let v = self.load(bus, a); self.op_sbc(v) }
            0xFA => self.consume_implied(bus),
            0xFB => { let a = self.addr_absolute_indexed(bus, y, true); self.op_isc(bus, a) }
            0xFC => { let a = self.addr_absolute_indexed(bus, x, false); self.load(bus, a); }
            0xFD => { let a = self.addr_absolute_indexed(bus, x, false); let v = self.load(bus, a); self.op_sbc(v) }
            0xFE => { let a = self.addr_absolute_indexed(bus, x, true); self.op_inc(bus, a) }
            0xFF => { let a = self.addr_absolute_indexed(bus, x, true); self.op_isc(bus, a) }
        }
    }
}
