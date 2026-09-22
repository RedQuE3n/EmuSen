//! The signal processor, ported from Mars's C# plain path, which stays its oracle. See Mars_Native.md §3.

pub const PC_MASK: u32 = 0xFFC;
pub const DATA_MASK: u32 = 0xFFF;
const ELEMENTS: usize = 8;
const ACCUMULATOR_MASK: u64 = 0xFFFF_FFFF_FFFF;
const VECTOR_OPERATION: u32 = 1 << 25;
const TRANSFER_FORMATS: u32 = 12;
const TRANSFER_SCALES: u64 = 0x4444_3344_3210;

/// The processor's scalar state, copied in from C# before a call and out after it. See Mars_Native.md §3.1.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Scalars {
    pub pc: u32,
    pub next_pc: u32,
    pub vco: u16,
    pub vcc: u16,
    pub divide_input: u16,
    pub divide_output: u16,
    pub vce: u8,
    pub halted: u8,
    pub broke: u8,
    pub divide_loaded: u8,
}

/// What the processor runs over: its registers, IMEM and DMEM, and its scalars. See Mars_Native.md §5.2.
pub trait Memory {
    fn gpr(&self, register: usize) -> u32;
    fn set_gpr(&mut self, register: usize, value: u32);
    fn element(&self, register: usize, element: usize) -> u16;
    fn set_element(&mut self, register: usize, element: usize, value: u16);
    fn acc(&self, element: usize) -> u64;
    fn set_acc(&mut self, element: usize, value: u64);
    fn data(&self, address: u32) -> u8;
    fn set_data(&mut self, address: u32, value: u8);
    fn fetch(&self, pc: u32) -> u32;
    fn pc(&self) -> u32;
    fn set_pc(&mut self, value: u32);
    fn next_pc(&self) -> u32;
    fn set_next_pc(&mut self, value: u32);
    fn vco(&self) -> u16;
    fn set_vco(&mut self, value: u16);
    fn vcc(&self) -> u16;
    fn set_vcc(&mut self, value: u16);
    fn vce(&self) -> u8;
    fn set_vce(&mut self, value: u8);
    fn divide_input(&self) -> u16;
    fn set_divide_input(&mut self, value: u16);
    fn divide_output(&self) -> u16;
    fn set_divide_output(&mut self, value: u16);
    fn divide_loaded(&self) -> u8;
    fn set_divide_loaded(&mut self, value: u8);
    fn halted(&self) -> u8;
}

/// Memory C# owns: pinned registers, vectors, accumulator, IMEM and DMEM, and scalars copied across each call.
pub struct Pinned {
    pub s: Scalars,
    gpr: *mut u32,
    vector: *mut u16,
    accumulator: *mut u64,
    imem: *const u8,
    dmem: *mut u8,
}

impl Memory for Pinned {
    #[inline(always)]
    fn gpr(&self, register: usize) -> u32 {
        unsafe { *self.gpr.add(register & 31) }
    }
    #[inline(always)]
    fn set_gpr(&mut self, register: usize, value: u32) {
        unsafe { *self.gpr.add(register & 31) = value }
    }
    #[inline(always)]
    fn element(&self, register: usize, element: usize) -> u16 {
        unsafe { *self.vector.add(((register & 31) << 3) | (element & 7)) }
    }
    #[inline(always)]
    fn set_element(&mut self, register: usize, element: usize, value: u16) {
        unsafe { *self.vector.add(((register & 31) << 3) | (element & 7)) = value }
    }
    #[inline(always)]
    fn acc(&self, element: usize) -> u64 {
        unsafe { *self.accumulator.add(element & 7) }
    }
    #[inline(always)]
    fn set_acc(&mut self, element: usize, value: u64) {
        unsafe { *self.accumulator.add(element & 7) = value }
    }
    #[inline(always)]
    fn data(&self, address: u32) -> u8 {
        unsafe { *self.dmem.add((address & DATA_MASK) as usize) }
    }
    #[inline(always)]
    fn set_data(&mut self, address: u32, value: u8) {
        unsafe { *self.dmem.add((address & DATA_MASK) as usize) = value }
    }
    #[inline(always)]
    fn fetch(&self, pc: u32) -> u32 {
        let at = (pc & PC_MASK) as usize;
        let bytes = unsafe { std::ptr::read_unaligned(self.imem.add(at) as *const [u8; 4]) };
        u32::from_be_bytes(bytes)
    }
    #[inline(always)]
    fn pc(&self) -> u32 {
        self.s.pc
    }
    #[inline(always)]
    fn set_pc(&mut self, value: u32) {
        self.s.pc = value
    }
    #[inline(always)]
    fn next_pc(&self) -> u32 {
        self.s.next_pc
    }
    #[inline(always)]
    fn set_next_pc(&mut self, value: u32) {
        self.s.next_pc = value
    }
    #[inline(always)]
    fn vco(&self) -> u16 {
        self.s.vco
    }
    #[inline(always)]
    fn set_vco(&mut self, value: u16) {
        self.s.vco = value
    }
    #[inline(always)]
    fn vcc(&self) -> u16 {
        self.s.vcc
    }
    #[inline(always)]
    fn set_vcc(&mut self, value: u16) {
        self.s.vcc = value
    }
    #[inline(always)]
    fn vce(&self) -> u8 {
        self.s.vce
    }
    #[inline(always)]
    fn set_vce(&mut self, value: u8) {
        self.s.vce = value
    }
    #[inline(always)]
    fn divide_input(&self) -> u16 {
        self.s.divide_input
    }
    #[inline(always)]
    fn set_divide_input(&mut self, value: u16) {
        self.s.divide_input = value
    }
    #[inline(always)]
    fn divide_output(&self) -> u16 {
        self.s.divide_output
    }
    #[inline(always)]
    fn set_divide_output(&mut self, value: u16) {
        self.s.divide_output = value
    }
    #[inline(always)]
    fn divide_loaded(&self) -> u8 {
        self.s.divide_loaded
    }
    #[inline(always)]
    fn set_divide_loaded(&mut self, value: u8) {
        self.s.divide_loaded = value
    }
    #[inline(always)]
    fn halted(&self) -> u8 {
        self.s.halted
    }
}

/// The processor over a `Memory`: C#'s pinned arrays for the twin, or MarsRT's own state.
pub struct Rsp<M: Memory> {
    pub m: M,
}

// Tables built once, by the C# arithmetic. See Mars_RspVector.md §10.
static TABLES: std::sync::OnceLock<([u16; 512], [u16; 512])> = std::sync::OnceLock::new();

fn tables() -> &'static ([u16; 512], [u16; 512]) {
    TABLES.get_or_init(|| {
        let mut reciprocal = [0u16; 512];
        reciprocal[0] = 0xFFFF;
        for (i, entry) in reciprocal.iter_mut().enumerate().skip(1) {
            *entry = ((((1u64 << 34) / (i as u64 + 512)) + 1) >> 8) as u16;
        }
        let mut root = [0u16; 512];
        for (i, entry) in root.iter_mut().enumerate() {
            let scale: u64 = if i < 256 { i as u64 + 256 } else { ((i as u64 - 256) << 1) + 512 };
            let mut r: u64 = 1 << 17;
            let mut step: u64 = 512;
            while step != 0 {
                while scale * (r + step) * (r + step) < (1u64 << 44) {
                    r += step;
                }
                step >>= 1;
            }
            *entry = (r >> 1) as u16;
        }
        (reciprocal, root)
    })
}

// C# masks a shift count to five bits, so these do too. See Mars_RspVector.md §10.1.
fn evaluate(value: u32, root: bool) -> u32 {
    if value == 0 {
        return 0x7FFF_FFFF;
    }
    if value == 0xFFFF_8000 {
        return 0xFFFF_0000;
    }
    let adjusted = if value > 0xFFFF_8000 { value - 1 } else { value };
    let negative = (adjusted as i32) < 0;
    let magnitude = if negative { !adjusted } else { adjusted };
    let shift = magnitude.leading_zeros() as i32 + 1;
    let normalised = magnitude.wrapping_shl(shift as u32);
    let (reciprocal, rsq) = tables();
    let result = if root {
        let index = ((normalised >> 24) | (((shift & 1) as u32) << 8)) as usize;
        (0x4000_0000u32 | ((rsq[index] as u32) << 14)).wrapping_shr(((32 - shift) >> 1) as u32)
    } else {
        (0x4000_0000u32 | ((reciprocal[(normalised >> 23) as usize] as u32) << 14)).wrapping_shr((32 - shift) as u32)
    };
    if negative { !result } else { result }
}

#[inline(always)]
fn signed48(value: u64) -> i64 {
    ((value << 16) as i64) >> 16
}

#[inline(always)]
fn clamp_signed(value: i64) -> u16 {
    if value < i16::MIN as i64 {
        0x8000
    } else if value > i16::MAX as i64 {
        0x7FFF
    } else {
        value as u16
    }
}

#[inline(always)]
fn clamp_unsigned(value: i64) -> u16 {
    if value < 0 {
        0
    } else if value > 0x7FFF_FFFF {
        0xFFFF
    } else {
        (value >> 16) as u16
    }
}

#[inline(always)]
fn clamp_low(value: i64) -> u16 {
    let high = value >> 16;
    if high < i16::MIN as i64 {
        0
    } else if high > i16::MAX as i64 {
        0xFFFF
    } else {
        value as u16
    }
}

#[inline(always)]
fn selected(selector: usize, i: usize) -> usize {
    match selector {
        0 | 1 => i,
        2 | 3 => selector - 2 + (i & 6),
        4..=7 => selector - 4 + (i & 4),
        _ => selector - 8,
    }
}

#[inline(always)]
fn immediate(instruction: u32) -> u32 {
    instruction as i16 as i32 as u32
}

#[inline(always)]
fn rs(instruction: u32) -> usize {
    ((instruction >> 21) & 0x1F) as usize
}

#[inline(always)]
fn rt(instruction: u32) -> usize {
    ((instruction >> 16) & 0x1F) as usize
}

#[inline(always)]
fn rd(instruction: u32) -> usize {
    ((instruction >> 11) & 0x1F) as usize
}

#[derive(Clone, Copy)]
enum Comparison {
    Less,
    Equal,
    NotEqual,
    GreaterOrEqual,
}

#[derive(Clone, Copy, PartialEq)]
enum Reciprocal {
    Single,
    Low,
    High,
}

impl Rsp<Pinned> {
    /// # Safety
    /// Every pointer must stay valid, and be touched by no one else while a call into this processor runs:
    /// gpr 32 words, vector 256 halfwords, accumulator 8 doublewords, imem and dmem 4096 bytes each.
    pub unsafe fn new(
        gpr: *mut u32,
        vector: *mut u16,
        accumulator: *mut u64,
        imem: *const u8,
        dmem: *mut u8,
    ) -> Rsp<Pinned> {
        tables();
        Rsp { m: Pinned { s: Scalars::default(), gpr, vector, accumulator, imem, dmem } }
    }
}

impl<M: Memory> Rsp<M> {
    /// The processor over memory the caller lends it for the call.
    #[inline(always)]
    pub fn over(m: M) -> Rsp<M> {
        Rsp { m }
    }

    #[inline(always)]
    fn read(&self, register: usize) -> u32 {
        self.m.gpr(register)
    }

    #[inline(always)]
    fn write(&mut self, register: usize, value: u32) {
        if register != 0 {
            self.m.set_gpr(register, value)
        }
    }

    #[inline(always)]
    fn element(&self, register: usize, element: usize) -> u16 {
        self.m.element(register, element)
    }

    #[inline(always)]
    fn set_element(&mut self, register: usize, element: usize, value: u16) {
        self.m.set_element(register, element, value)
    }

    #[inline(always)]
    fn acc(&self, element: usize) -> u64 {
        self.m.acc(element)
    }

    #[inline(always)]
    fn set_acc(&mut self, element: usize, value: u64) {
        self.m.set_acc(element, value)
    }

    #[inline(always)]
    fn data(&self, address: u32) -> u8 {
        self.m.data(address)
    }

    #[inline(always)]
    fn set_data(&mut self, address: u32, value: u8) {
        self.m.set_data(address, value)
    }

    #[inline(always)]
    fn fetch(&self, pc: u32) -> u32 {
        self.m.fetch(pc)
    }

    // A COP0 move or a break reaches the rest of the machine, so C# runs it; this side never calls back. See Mars_Native.md §3.2.
    #[inline(always)]
    fn is_event(instruction: u32) -> bool {
        let op = instruction >> 26;
        op == 0x10 || (op == 0 && instruction & 0x3F == 0x0D)
    }

    /// One instruction, as C#'s StepOne, unless it is an event, which is left unrun; returns whether it ran.
    #[inline(always)]
    pub fn step(&mut self) -> bool {
        let instruction = self.fetch(self.m.pc());
        if Self::is_event(instruction) {
            return false;
        }
        self.m.set_pc(self.m.next_pc());
        self.m.set_next_pc((self.m.pc() + 4) & PC_MASK);
        self.execute(instruction);
        true
    }

    /// Up to `budget` instructions while not halted, stopping before an event; returns the steps run.
    pub fn run(&mut self, budget: u64) -> u64 {
        let mut ran = 0;
        while ran < budget && self.m.halted() == 0 {
            if !self.step() {
                break;
            }
            ran += 1;
        }
        ran
    }

    #[inline(always)]
    fn execute(&mut self, instruction: u32) {
        match instruction >> 26 {
            0x00 => self.special(instruction),
            0x01 => self.regimm(instruction),
            0x02 => self.jump(instruction),
            0x03 => {
                self.link();
                self.jump(instruction);
            }
            0x04 => {
                let taken = self.read(rs(instruction)) == self.read(rt(instruction));
                self.branch_if(taken, instruction);
            }
            0x05 => {
                let taken = self.read(rs(instruction)) != self.read(rt(instruction));
                self.branch_if(taken, instruction);
            }
            0x06 => {
                let taken = (self.read(rs(instruction)) as i32) <= 0;
                self.branch_if(taken, instruction);
            }
            0x07 => {
                let taken = (self.read(rs(instruction)) as i32) > 0;
                self.branch_if(taken, instruction);
            }
            0x08 | 0x09 => {
                let value = self.read(rs(instruction)).wrapping_add(immediate(instruction));
                self.write(rt(instruction), value);
            }
            0x0A => {
                let value = ((self.read(rs(instruction)) as i32) < (immediate(instruction) as i32)) as u32;
                self.write(rt(instruction), value);
            }
            0x0B => {
                let value = (self.read(rs(instruction)) < immediate(instruction)) as u32;
                self.write(rt(instruction), value);
            }
            0x0C => {
                let value = self.read(rs(instruction)) & (instruction & 0xFFFF);
                self.write(rt(instruction), value);
            }
            0x0D => {
                let value = self.read(rs(instruction)) | (instruction & 0xFFFF);
                self.write(rt(instruction), value);
            }
            0x0E => {
                let value = self.read(rs(instruction)) ^ (instruction & 0xFFFF);
                self.write(rt(instruction), value);
            }
            0x0F => self.write(rt(instruction), instruction << 16),
            0x12 => self.cop2(instruction),
            0x20 => {
                let value = self.read_data(self.address(instruction), 1) as i8 as i32 as u32;
                self.write(rt(instruction), value);
            }
            0x21 => {
                let value = self.read_data(self.address(instruction), 2) as i16 as i32 as u32;
                self.write(rt(instruction), value);
            }
            0x23 | 0x27 => {
                let value = self.read_data(self.address(instruction), 4);
                self.write(rt(instruction), value);
            }
            0x24 => {
                let value = self.read_data(self.address(instruction), 1);
                self.write(rt(instruction), value);
            }
            0x25 => {
                let value = self.read_data(self.address(instruction), 2);
                self.write(rt(instruction), value);
            }
            0x28 => self.write_data(self.address(instruction), self.read(rt(instruction)), 1),
            0x29 => self.write_data(self.address(instruction), self.read(rt(instruction)), 2),
            0x2B => self.write_data(self.address(instruction), self.read(rt(instruction)), 4),
            0x32 => self.vector_load(instruction),
            0x3A => self.vector_store(instruction),
            _ => {}
        }
    }

    #[inline(always)]
    fn special(&mut self, instruction: u32) {
        let shift = (instruction >> 6) & 0x1F;
        let t = self.read(rt(instruction));
        let s = self.read(rs(instruction));
        let d = rd(instruction);
        match instruction & 0x3F {
            0x00 => self.write(d, t << shift),
            0x02 => self.write(d, t >> shift),
            0x03 => self.write(d, ((t as i32) >> shift) as u32),
            0x04 => self.write(d, t << (s & 0x1F)),
            0x06 => self.write(d, t >> (s & 0x1F)),
            0x07 => self.write(d, ((t as i32) >> (s & 0x1F)) as u32),
            0x08 => self.m.set_next_pc(s & PC_MASK),
            0x09 => {
                let jump = s & PC_MASK;
                let next = self.m.next_pc();
                self.write(d, next);
                self.m.set_next_pc(jump);
            }
            0x20 | 0x21 => self.write(d, s.wrapping_add(t)),
            0x22 | 0x23 => self.write(d, s.wrapping_sub(t)),
            0x24 => self.write(d, s & t),
            0x25 => self.write(d, s | t),
            0x26 => self.write(d, s ^ t),
            0x27 => self.write(d, !(s | t)),
            0x2A => self.write(d, ((s as i32) < (t as i32)) as u32),
            0x2B => self.write(d, (s < t) as u32),
            _ => {}
        }
    }

    #[inline(always)]
    fn regimm(&mut self, instruction: u32) {
        let value = self.read(rs(instruction)) as i32;
        let selector = rt(instruction);
        if selector & 0x10 != 0 {
            self.link();
        }
        let taken = if selector & 1 != 0 { value >= 0 } else { value < 0 };
        self.branch_if(taken, instruction);
    }

    #[inline(always)]
    fn link(&mut self) {
        let next = self.m.next_pc();
        self.write(31, next);
    }

    #[inline(always)]
    fn jump(&mut self, instruction: u32) {
        self.m.set_next_pc((instruction << 2) & PC_MASK);
    }

    #[inline(always)]
    fn branch_if(&mut self, taken: bool, instruction: u32) {
        if taken {
            self.m.set_next_pc(self.m.pc().wrapping_add(immediate(instruction) << 2) & PC_MASK);
        }
    }

    #[inline(always)]
    fn address(&self, instruction: u32) -> u32 {
        self.read(rs(instruction)).wrapping_add(immediate(instruction)) & DATA_MASK
    }

    #[inline(always)]
    fn read_data(&self, address: u32, size: u32) -> u32 {
        let mut value = 0u32;
        for i in 0..size {
            value = (value << 8) | self.data(address.wrapping_add(i)) as u32;
        }
        value
    }

    #[inline(always)]
    fn write_data(&mut self, address: u32, value: u32, size: u32) {
        for i in 0..size {
            self.set_data(address.wrapping_add(i), (value >> ((size - 1 - i) * 8)) as u8);
        }
    }

    fn vector_byte(&self, register: usize, index: usize) -> u8 {
        let element = self.element(register, index >> 1);
        if index & 1 == 0 { (element >> 8) as u8 } else { element as u8 }
    }

    fn set_vector_byte(&mut self, register: usize, index: usize, value: u8) {
        let element = self.element(register, index >> 1);
        let updated = if index & 1 == 0 { (element & 0x00FF) | ((value as u16) << 8) } else { (element & 0xFF00) | value as u16 };
        self.set_element(register, index >> 1, updated);
    }

    fn cop2(&mut self, instruction: u32) {
        if instruction & VECTOR_OPERATION != 0 {
            self.vector_op(instruction);
            return;
        }
        let register = rd(instruction);
        let element = ((instruction >> 7) & 0xF) as usize;
        match rs(instruction) {
            0x00 => {
                let high = self.vector_byte(register, element) as u16;
                let low = self.vector_byte(register, (element + 1) & 0xF) as u16;
                self.write(rt(instruction), ((high << 8) | low) as i16 as i32 as u32);
            }
            0x02 => {
                let value = match register & 3 {
                    0 => self.m.vco() as i16 as i32 as u32,
                    1 => self.m.vcc() as i16 as i32 as u32,
                    _ => self.m.vce() as u32,
                };
                self.write(rt(instruction), value);
            }
            0x04 => {
                let value = self.read(rt(instruction));
                self.set_vector_byte(register, element, (value >> 8) as u8);
                if element < 15 {
                    self.set_vector_byte(register, element + 1, value as u8);
                }
            }
            0x06 => {
                let value = self.read(rt(instruction));
                match register & 3 {
                    0 => self.m.set_vco(value as u16),
                    1 => self.m.set_vcc(value as u16),
                    _ => self.m.set_vce(value as u8),
                }
            }
            _ => {}
        }
    }

    #[inline(always)]
    fn accumulate(&mut self, element: usize, value: i64) -> i64 {
        let wrapped = (value as u64) & ACCUMULATOR_MASK;
        self.set_acc(element, wrapped);
        signed48(wrapped)
    }

    #[inline(always)]
    fn set_acc_low(&mut self, element: usize, value: u16) {
        let old = self.acc(element);
        self.set_acc(element, (old & !0xFFFFu64) | value as u64);
    }

    fn vector_op(&mut self, instruction: u32) {
        let vt = rt(instruction);
        let vs = rd(instruction);
        let vd = ((instruction >> 6) & 0x1F) as usize;
        let selector = ((instruction >> 21) & 0xF) as usize;

        let mut s = [0u16; ELEMENTS];
        let mut t = [0u16; ELEMENTS];
        let mut d = [0u16; ELEMENTS];
        for i in 0..ELEMENTS {
            s[i] = self.element(vs, i);
            t[i] = self.element(vt, selected(selector, i));
            d[i] = self.element(vd, i);
        }

        match instruction & 0x3F {
            0x00 => self.fraction(&s, &t, &mut d, false, false),
            0x01 => self.fraction(&s, &t, &mut d, true, false),
            0x02 => self.round(&t, &mut d, vs & 1 != 0, true),
            0x03 => self.quarter(&s, &t, &mut d),
            0x04 => self.low(&s, &t, &mut d, false),
            0x05 => self.middle(&s, &t, &mut d, false),
            0x06 => self.normal(&s, &t, &mut d, false),
            0x07 => self.high(&s, &t, &mut d, false),
            0x08 => self.fraction(&s, &t, &mut d, false, true),
            0x09 => self.fraction(&s, &t, &mut d, true, true),
            0x0A => self.round(&t, &mut d, vs & 1 != 0, false),
            0x0B => self.accumulated_quarter(&mut d),
            0x0C => self.low(&s, &t, &mut d, true),
            0x0D => self.middle(&s, &t, &mut d, true),
            0x0E => self.normal(&s, &t, &mut d, true),
            0x0F => self.high(&s, &t, &mut d, true),
            0x10 => self.add(&s, &t, &mut d, false),
            0x11 => self.add(&s, &t, &mut d, true),
            0x13 => self.absolute(&s, &t, &mut d),
            0x14 => self.add_carrying(&s, &t, &mut d),
            0x15 => self.subtract_carrying(&s, &t, &mut d),
            0x1D => self.read_accumulator(selector, &mut d),
            0x20 => self.compare(&s, &t, &mut d, Comparison::Less),
            0x21 => self.compare(&s, &t, &mut d, Comparison::Equal),
            0x22 => self.compare(&s, &t, &mut d, Comparison::NotEqual),
            0x23 => self.compare(&s, &t, &mut d, Comparison::GreaterOrEqual),
            0x24 => self.clip_low(&s, &t, &mut d),
            0x25 => self.clip_high(&s, &t, &mut d),
            0x26 => self.clip_ones(&s, &t, &mut d),
            0x27 => self.merge(&s, &t, &mut d),
            0x28 => self.logic(&s, &t, &mut d, |a, b| a & b),
            0x29 => self.logic(&s, &t, &mut d, |a, b| !(a & b)),
            0x2A => self.logic(&s, &t, &mut d, |a, b| a | b),
            0x2B => self.logic(&s, &t, &mut d, |a, b| !(a | b)),
            0x2C => self.logic(&s, &t, &mut d, |a, b| a ^ b),
            0x2D => self.logic(&s, &t, &mut d, |a, b| !(a ^ b)),
            0x30 => self.reciprocate(self.element(vt, selector & 7), vs & 7, &t, &mut d, Reciprocal::Single, false),
            0x31 => self.reciprocate(self.element(vt, selector & 7), vs & 7, &t, &mut d, Reciprocal::Low, false),
            0x32 => self.reciprocate(self.element(vt, selector & 7), vs & 7, &t, &mut d, Reciprocal::High, false),
            0x33 => self.move_element(vs & 7, &t, &mut d),
            0x34 => self.reciprocate(self.element(vt, selector & 7), vs & 7, &t, &mut d, Reciprocal::Single, true),
            0x35 => self.reciprocate(self.element(vt, selector & 7), vs & 7, &t, &mut d, Reciprocal::Low, true),
            0x36 => self.reciprocate(self.element(vt, selector & 7), vs & 7, &t, &mut d, Reciprocal::High, true),
            0x37 | 0x3F => return,
            _ => self.sum_into_accumulator(&s, &t, &mut d),
        }

        for (i, value) in d.iter().enumerate() {
            self.set_element(vd, i, *value);
        }
    }

    fn fraction(&mut self, s: &[u16; 8], t: &[u16; 8], d: &mut [u16; 8], unsigned: bool, accumulate: bool) {
        for i in 0..ELEMENTS {
            let product = (s[i] as i16 as i64) * (t[i] as i16 as i64);
            let base = if accumulate { signed48(self.acc(i)) } else { 0x8000 };
            let value = self.accumulate(i, base + (product << 1));
            d[i] = if unsigned { clamp_unsigned(value) } else { clamp_signed(value >> 16) };
        }
    }

    fn low(&mut self, s: &[u16; 8], t: &[u16; 8], d: &mut [u16; 8], accumulate: bool) {
        for i in 0..ELEMENTS {
            let product = (s[i] as u32) * (t[i] as u32);
            let base = if accumulate { signed48(self.acc(i)) } else { 0 };
            d[i] = clamp_low(self.accumulate(i, base + (product >> 16) as i64));
        }
    }

    fn middle(&mut self, s: &[u16; 8], t: &[u16; 8], d: &mut [u16; 8], accumulate: bool) {
        for i in 0..ELEMENTS {
            let product = (s[i] as i16 as i64) * (t[i] as i64);
            let base = if accumulate { signed48(self.acc(i)) } else { 0 };
            d[i] = clamp_signed(self.accumulate(i, base + product) >> 16);
        }
    }

    fn normal(&mut self, s: &[u16; 8], t: &[u16; 8], d: &mut [u16; 8], accumulate: bool) {
        for i in 0..ELEMENTS {
            let product = (s[i] as i64) * (t[i] as i16 as i64);
            let base = if accumulate { signed48(self.acc(i)) } else { 0 };
            d[i] = clamp_low(self.accumulate(i, base + product));
        }
    }

    fn high(&mut self, s: &[u16; 8], t: &[u16; 8], d: &mut [u16; 8], accumulate: bool) {
        for i in 0..ELEMENTS {
            let product = (s[i] as i16 as i64) * (t[i] as i16 as i64);
            let base = if accumulate { signed48(self.acc(i)) } else { 0 };
            d[i] = clamp_signed(self.accumulate(i, base + (product << 16)) >> 16);
        }
    }

    fn quarter(&mut self, s: &[u16; 8], t: &[u16; 8], d: &mut [u16; 8]) {
        for i in 0..ELEMENTS {
            let product = (s[i] as i16 as i64) * (t[i] as i16 as i64);
            let value = self.accumulate(i, (product << 16) + if product < 0 { 0x1F_0000 } else { 0 });
            d[i] = clamp_signed(value >> 17) & 0xFFF0;
        }
    }

    #[allow(clippy::needless_range_loop)]
    fn accumulated_quarter(&mut self, d: &mut [u16; 8]) {
        for i in 0..ELEMENTS {
            let mut value = signed48(self.acc(i));
            if value & 0x20_0000 == 0 {
                let upper = value >> 22;
                if upper < 0 {
                    value += 0x20_0000;
                } else if upper > 0 {
                    value -= 0x20_0000;
                }
            }
            d[i] = clamp_signed(self.accumulate(i, value) >> 17) & 0xFFF0;
        }
    }

    fn round(&mut self, t: &[u16; 8], d: &mut [u16; 8], shifted: bool, positive: bool) {
        for i in 0..ELEMENTS {
            let mut value = signed48(self.acc(i));
            if (value >= 0) == positive {
                value += (t[i] as i16 as i64) << if shifted { 16 } else { 0 };
            }
            d[i] = clamp_signed(self.accumulate(i, value) >> 16);
        }
    }

    fn add(&mut self, s: &[u16; 8], t: &[u16; 8], d: &mut [u16; 8], subtract: bool) {
        for i in 0..ELEMENTS {
            let carry = ((self.m.vco() >> i) & 1) as i32;
            let (a, b) = (s[i] as i16 as i32, t[i] as i16 as i32);
            let sum = if subtract { a - b - carry } else { a + b + carry };
            d[i] = clamp_signed(sum as i64);
            self.set_acc_low(i, sum as u16);
        }
        self.m.set_vco(0);
    }

    fn absolute(&mut self, s: &[u16; 8], t: &[u16; 8], d: &mut [u16; 8]) {
        for i in 0..ELEMENTS {
            let sign = s[i] as i16;
            let value = if sign < 0 { t[i].wrapping_neg() } else if sign == 0 { 0 } else { t[i] };
            d[i] = if sign < 0 && t[i] == 0x8000 { 0x7FFF } else { value };
            self.set_acc_low(i, value);
        }
    }

    fn add_carrying(&mut self, s: &[u16; 8], t: &[u16; 8], d: &mut [u16; 8]) {
        let mut carries = 0u16;
        for i in 0..ELEMENTS {
            let sum = s[i] as i32 + t[i] as i32;
            d[i] = sum as u16;
            self.set_acc_low(i, sum as u16);
            if sum > 0xFFFF {
                carries |= 1 << i;
            }
        }
        self.m.set_vco(carries);
    }

    fn subtract_carrying(&mut self, s: &[u16; 8], t: &[u16; 8], d: &mut [u16; 8]) {
        let mut flags = 0u16;
        for i in 0..ELEMENTS {
            let difference = s[i] as i32 - t[i] as i32;
            d[i] = difference as u16;
            self.set_acc_low(i, difference as u16);
            if difference != 0 {
                flags |= 0x100 << i;
            }
            if difference < 0 {
                flags |= 1 << i;
            }
        }
        self.m.set_vco(flags);
    }

    fn read_accumulator(&mut self, selector: usize, d: &mut [u16; 8]) {
        let shift: i32 = match selector {
            8 => 32,
            9 => 16,
            10 => 0,
            _ => -1,
        };
        for (i, value) in d.iter_mut().enumerate() {
            *value = if shift < 0 { 0 } else { (self.acc(i) >> shift) as u16 };
        }
    }

    fn compare(&mut self, s: &[u16; 8], t: &[u16; 8], d: &mut [u16; 8], comparison: Comparison) {
        let mut flags = 0u16;
        for i in 0..ELEMENTS {
            let equal = s[i] == t[i];
            let carry = (self.m.vco() >> i) & 1 != 0;
            let not_equal = (self.m.vco() >> (8 + i)) & 1 != 0;
            let (a, b) = (s[i] as i16, t[i] as i16);
            let chosen = match comparison {
                Comparison::Less => a < b || (equal && carry && not_equal),
                Comparison::Equal => equal && !not_equal,
                Comparison::NotEqual => !equal || not_equal,
                Comparison::GreaterOrEqual => a > b || (equal && !(carry && not_equal)),
            };
            d[i] = match comparison {
                Comparison::Equal => t[i],
                Comparison::NotEqual => s[i],
                _ => {
                    if chosen {
                        s[i]
                    } else {
                        t[i]
                    }
                }
            };
            self.set_acc_low(i, d[i]);
            if chosen {
                flags |= 1 << i;
            }
        }
        self.m.set_vcc(flags);
        self.m.set_vco(0);
    }

    fn clip_low(&mut self, s: &[u16; 8], t: &[u16; 8], d: &mut [u16; 8]) {
        let mut flags = self.m.vcc() as i32;
        for i in 0..ELEMENTS {
            let signs_differed = (self.m.vco() >> i) & 1 != 0;
            let not_equal = (self.m.vco() >> (8 + i)) & 1 != 0;
            let extension = (self.m.vce() >> i) & 1 != 0;
            let mut less_or_equal = (flags >> i) & 1 != 0;
            let mut greater_or_equal = (flags >> (8 + i)) & 1 != 0;
            if signs_differed {
                let sum = s[i] as i32 + t[i] as i32;
                let zero = sum & 0xFFFF == 0;
                let carried = sum > 0xFFFF;
                if !not_equal {
                    less_or_equal = (zero && !carried) || (extension && (zero || !carried));
                }
                d[i] = if less_or_equal { t[i].wrapping_neg() } else { s[i] };
            } else {
                if !not_equal {
                    greater_or_equal = s[i] >= t[i];
                }
                d[i] = if greater_or_equal { t[i] } else { s[i] };
            }
            self.set_acc_low(i, d[i]);
            flags = (flags & !(0x101 << i)) | if less_or_equal { 1 << i } else { 0 } | if greater_or_equal { 0x100 << i } else { 0 };
        }
        self.m.set_vcc(flags as u16);
        self.m.set_vco(0);
        self.m.set_vce(0);
    }

    fn clip_high(&mut self, s: &[u16; 8], t: &[u16; 8], d: &mut [u16; 8]) {
        let (mut carries, mut compares, mut extensions) = (0u16, 0u16, 0u8);
        for i in 0..ELEMENTS {
            let a = t[i] as i16 as i32;
            let b = s[i] as i16 as i32;
            let signs_differ = (a ^ b) < 0;
            let (less_or_equal, greater_or_equal, extension, not_equal);
            if signs_differ {
                let sum = a + b;
                greater_or_equal = a < 0;
                less_or_equal = sum <= 0;
                extension = sum == -1;
                not_equal = sum != 0 && sum != -1;
                d[i] = if less_or_equal { (-a) as u16 } else { b as u16 };
            } else {
                let difference = b - a;
                less_or_equal = a < 0;
                greater_or_equal = difference >= 0;
                extension = false;
                not_equal = difference != 0;
                d[i] = if greater_or_equal { a as u16 } else { b as u16 };
            }
            self.set_acc_low(i, d[i]);
            carries |= if signs_differ { 1 << i } else { 0 } | if not_equal { 0x100 << i } else { 0 };
            compares |= if less_or_equal { 1 << i } else { 0 } | if greater_or_equal { 0x100 << i } else { 0 };
            extensions |= if extension { 1 << i } else { 0 };
        }
        self.m.set_vco(carries);
        self.m.set_vcc(compares);
        self.m.set_vce(extensions);
    }

    fn clip_ones(&mut self, s: &[u16; 8], t: &[u16; 8], d: &mut [u16; 8]) {
        let mut compares = 0u16;
        for i in 0..ELEMENTS {
            let a = t[i] as i16 as i32;
            let b = s[i] as i16 as i32;
            let (less_or_equal, greater_or_equal);
            if (a ^ b) < 0 {
                greater_or_equal = a < 0;
                less_or_equal = a + b < 0;
                d[i] = if less_or_equal { !a as u16 } else { b as u16 };
            } else {
                less_or_equal = a < 0;
                greater_or_equal = b - a >= 0;
                d[i] = if greater_or_equal { a as u16 } else { b as u16 };
            }
            self.set_acc_low(i, d[i]);
            compares |= if less_or_equal { 1 << i } else { 0 } | if greater_or_equal { 0x100 << i } else { 0 };
        }
        self.m.set_vco(0);
        self.m.set_vcc(compares);
        self.m.set_vce(0);
    }

    fn merge(&mut self, s: &[u16; 8], t: &[u16; 8], d: &mut [u16; 8]) {
        for i in 0..ELEMENTS {
            d[i] = if (self.m.vcc() >> i) & 1 != 0 { s[i] } else { t[i] };
            self.set_acc_low(i, d[i]);
        }
        self.m.set_vco(0);
    }

    fn logic(&mut self, s: &[u16; 8], t: &[u16; 8], d: &mut [u16; 8], operation: impl Fn(i32, i32) -> i32) {
        for i in 0..ELEMENTS {
            d[i] = operation(t[i] as i32, s[i] as i32) as u16;
            self.set_acc_low(i, d[i]);
        }
    }

    fn move_element(&mut self, element: usize, t: &[u16; 8], d: &mut [u16; 8]) {
        for (i, value) in t.iter().enumerate() {
            self.set_acc_low(i, *value);
        }
        d[element] = t[element];
    }

    fn reciprocate(&mut self, input: u16, element: usize, t: &[u16; 8], d: &mut [u16; 8], kind: Reciprocal, root: bool) {
        for (i, value) in t.iter().enumerate() {
            self.set_acc_low(i, *value);
        }
        if kind == Reciprocal::High {
            d[element] = self.m.divide_output();
            self.m.set_divide_input(input);
            self.m.set_divide_loaded(1);
            return;
        }
        let operand = if kind == Reciprocal::Low && self.m.divide_loaded() != 0 {
            ((self.m.divide_input() as u32) << 16) | input as u32
        } else {
            input as i16 as i32 as u32
        };
        let result = evaluate(operand, root);
        d[element] = result as u16;
        self.m.set_divide_output((result >> 16) as u16);
        self.m.set_divide_loaded(0);
    }

    fn sum_into_accumulator(&mut self, s: &[u16; 8], t: &[u16; 8], d: &mut [u16; 8]) {
        for i in 0..ELEMENTS {
            self.set_acc_low(i, s[i].wrapping_add(t[i]));
            d[i] = 0;
        }
    }

    // The offset is seven signed bits, scaled by the format's nibble. See Mars_RspVector.md §4.
    fn transfer_operands(&self, instruction: u32) -> Option<(u32, usize, usize, u32)> {
        let format = (instruction >> 11) & 0x1F;
        if format >= TRANSFER_FORMATS {
            return None;
        }
        let vt = rt(instruction);
        let element = ((instruction >> 7) & 0xF) as usize;
        let offset = ((instruction << 25) as i32) >> 25;
        let scale = ((TRANSFER_SCALES >> (format * 4)) & 0xF) as u32;
        let address = self.read(rs(instruction)).wrapping_add(offset.wrapping_shl(scale) as u32) & DATA_MASK;
        Some((format, vt, element, address))
    }

    fn vector_load(&mut self, instruction: u32) {
        let Some((format, vt, element, address)) = self.transfer_operands(instruction) else { return };
        match format {
            0..=3 => self.load_bytes(vt, element, address, 1 << format),
            4 => self.load_bytes(vt, element, address, 16 - (address & 0xF) as usize),
            5 => self.load_rest(vt, element, address),
            6 => self.load_unpacked(vt, element, address, 8, 1),
            7 => self.load_unpacked(vt, element, address, 7, 1),
            8 => self.load_unpacked(vt, element, address, 7, 2),
            9 => self.load_fraction(vt, element, address),
            11 => self.load_transposed(vt, element, address),
            _ => {}
        }
    }

    fn vector_store(&mut self, instruction: u32) {
        let Some((format, vt, element, address)) = self.transfer_operands(instruction) else { return };
        match format {
            0..=3 => self.store_bytes(vt, element, address, 1 << format),
            4 => self.store_bytes(vt, element, address, 16 - (address & 0xF) as usize),
            5 => self.store_rest(vt, element, address),
            6 => self.store_packed(vt, element, address, 8, 7),
            7 => self.store_packed(vt, element, address, 7, 8),
            8 => self.store_halves(vt, element, address),
            9 => self.store_fraction(vt, element, address),
            10 => self.store_whole(vt, element, address),
            11 => self.store_transposed(vt, element, address),
            _ => {}
        }
    }

    fn load_bytes(&mut self, vt: usize, element: usize, address: u32, count: usize) {
        for i in 0..(16 - element).min(count) {
            let value = self.data(address.wrapping_add(i as u32));
            self.set_vector_byte(vt, element + i, value);
        }
    }

    fn load_rest(&mut self, vt: usize, element: usize, address: u32) {
        let mut i = 16 - (address & 0xF) as usize;
        while i < 16 && element + i <= 15 {
            let value = self.data(address.wrapping_add(i as u32).wrapping_sub(16));
            self.set_vector_byte(vt, element + i, value);
            i += 1;
        }
    }

    fn load_unpacked(&mut self, vt: usize, element: usize, address: u32, shift: u32, stride: i32) {
        let aligned = address & !7;
        let misalignment = (address & 7) as i32;
        for i in 0..ELEMENTS {
            let at = (misalignment - element as i32 + i as i32 * stride) & 0xF;
            let value = (self.data(aligned + at as u32) as u16) << shift;
            self.set_element(vt, i, value);
        }
    }

    fn load_fraction(&mut self, vt: usize, element: usize, address: u32) {
        let aligned = address & !7;
        let misalignment = (address & 7) as i32;
        let e = element as i32;
        let offsets = [e, 4 - e, 8 - e, 12 - e, 8 - e, 12 - e, -e, 4 - e];
        let mut unpacked = [0u16; ELEMENTS];
        for i in 0..ELEMENTS {
            let at = (misalignment + offsets[i]) & 0xF;
            unpacked[i] = (self.data(aligned + at as u32) as u16) << 7;
        }
        for b in element..element + 8usize.min(16 - element) {
            let value = if b & 1 == 0 { (unpacked[b >> 1] >> 8) as u8 } else { unpacked[b >> 1] as u8 };
            self.set_vector_byte(vt, b, value);
        }
    }

    fn load_transposed(&mut self, vt: usize, element: usize, address: u32) {
        let group = vt & !7;
        let aligned = address & !7;
        let rotation = (address & 8) as usize;
        for i in 0..ELEMENTS {
            let register = group + (((element >> 1) + i) & 7);
            let at = rotation + element + i * 2;
            let high = self.data(aligned + (at & 0xF) as u32) as u16;
            let low = self.data(aligned + ((at + 1) & 0xF) as u32) as u16;
            self.set_element(register, i, (high << 8) | low);
        }
    }

    fn store_bytes(&mut self, vt: usize, element: usize, address: u32, count: usize) {
        for i in 0..count {
            let value = self.vector_byte(vt, (element + i) & 0xF);
            self.set_data(address.wrapping_add(i as u32), value);
        }
    }

    fn store_rest(&mut self, vt: usize, element: usize, address: u32) {
        for i in (16 - (address & 0xF) as usize)..16 {
            let value = self.vector_byte(vt, (element + i) & 0xF);
            self.set_data(address.wrapping_add(i as u32).wrapping_sub(16), value);
        }
    }

    fn store_packed(&mut self, vt: usize, element: usize, address: u32, lower_shift: u32, upper_shift: u32) {
        for i in 0..ELEMENTS {
            let index = element + i;
            let shift = if index & 8 == 0 { lower_shift } else { upper_shift };
            let value = (self.element(vt, index & 7) >> shift) as u8;
            self.set_data(address.wrapping_add(i as u32), value);
        }
    }

    fn store_halves(&mut self, vt: usize, element: usize, address: u32) {
        let aligned = address & !7;
        let misalignment = (address & 7) as usize;
        for i in 0..ELEMENTS {
            let index = element + i * 2;
            let value = ((self.vector_byte(vt, index & 0xF) as i32) << 8) | self.vector_byte(vt, (index + 1) & 0xF) as i32;
            self.set_data(aligned + ((misalignment + i * 2) & 0xF) as u32, (value >> 7) as u8);
        }
    }

    fn store_fraction(&mut self, vt: usize, element: usize, address: u32) {
        let aligned = address & !7;
        let misalignment = (address & 7) as usize;
        let first: i32 = match element {
            0 => 0,
            1 => 6,
            4 => 1,
            5 => 7,
            8 => 4,
            11 => 3,
            12 => 5,
            15 => 0,
            _ => -1,
        };
        for i in 0..4 {
            let value = if first < 0 {
                0
            } else {
                let f = first as usize;
                (self.element(vt, (f & 4) | ((f + i) & 3)) >> 7) as u8
            };
            self.set_data(aligned + ((misalignment + i * 4) & 0xF) as u32, value);
        }
    }

    fn store_whole(&mut self, vt: usize, element: usize, address: u32) {
        let aligned = address & !7;
        let misalignment = (address & 7) as usize;
        for i in 0..16 {
            let value = self.vector_byte(vt, (element + i) & 0xF);
            self.set_data(aligned + ((misalignment + i) & 0xF) as u32, value);
        }
    }

    fn store_transposed(&mut self, vt: usize, element: usize, address: u32) {
        let group = vt & !7;
        let aligned = address & !7;
        for i in 0..16usize {
            let register = group + ((((i >> 1) as i32) - (aligned >> 1) as i32 + (element >> 1) as i32) & 7) as usize;
            let value = self.vector_byte(register, (i + aligned as usize) & 0xF);
            self.set_data(aligned + (address.wrapping_add(i as u32) & 0xF), value);
        }
    }
}

// The C ABI C# calls. See Mars_Native.md §3.

/// # Safety
/// See `Rsp::new`; the returned pointer is freed by `mars_rsp_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_rsp_new(
    gpr: *mut u32,
    vector: *mut u16,
    accumulator: *mut u64,
    imem: *const u8,
    dmem: *mut u8,
) -> *mut Rsp<Pinned> {
    Box::into_raw(Box::new(unsafe { Rsp::new(gpr, vector, accumulator, imem, dmem) }))
}

/// # Safety
/// `rsp` came from `mars_rsp_new` and is not used again.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_rsp_free(rsp: *mut Rsp<Pinned>) {
    if !rsp.is_null() {
        drop(unsafe { Box::from_raw(rsp) });
    }
}

/// The scalars C# copies in before a call and out after it.
///
/// # Safety
/// `rsp` came from `mars_rsp_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_rsp_scalars(rsp: *mut Rsp<Pinned>) -> *mut Scalars {
    unsafe { &mut (*rsp).m.s }
}

/// One instruction unless it is an event; returns 1 if it ran and 0 if C# must run it.
///
/// # Safety
/// `rsp` came from `mars_rsp_new`; the processor is not halted.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_rsp_step(rsp: *mut Rsp<Pinned>) -> u32 {
    unsafe { (*rsp).step() as u32 }
}

/// Up to `budget` instructions, stopping at a halt or before an event; returns the steps run.
///
/// # Safety
/// `rsp` came from `mars_rsp_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_rsp_run(rsp: *mut Rsp<Pinned>, budget: u64) -> u64 {
    unsafe { (*rsp).run(budget) }
}

/// The reciprocal unit alone, for a test that compares every input with C#'s.
#[unsafe(no_mangle)]
pub extern "C" fn mars_rsp_reciprocal(value: u32, root: u32) -> u32 {
    evaluate(value, root != 0)
}
