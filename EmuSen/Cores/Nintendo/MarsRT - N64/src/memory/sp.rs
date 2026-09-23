//! The signal processor's interface and registers, the C# `SpInterface` and `Rsp`.

use crate::memory::bus::MemoryBus;
use crate::memory::dp_threads::site;
use crate::memory::mi::interrupt;
use crate::Skip;
use crate::rsp::decoded::Decoded;
use crate::rsp::{self, DATA_MASK, Memory, PC_MASK, Trace};
use crate::state::{State, StateReader, StateResult, StateWriter};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rsp {
    /// The 48-bit accumulator as three thirds of eight lanes, high, middle and low, as the vector unit keeps it; the state carries it as C#'s eight words (Mars_Native.md §6.10).
    pub accumulator: [[u16; 8]; 3],
    /// Bits 63:48 of the state's words, which no instruction reads; a load keeps them and a whole write clears them, as the words did.
    pub accumulator_top: [u16; 8],
    pub broke: bool,
    pub gpr: [u32; 32],
    pub halted: bool,
    pub next_pc: u32,
    pub pc: u32,
    pub vcc: u16,
    pub vce: u8,
    pub vco: u16,
    /// Register first, element second: a register is one 128-bit lane of eight elements.
    pub vector: [[u16; 8]; 32],
    pub divide_input: u16,
    pub divide_input_loaded: bool,
    pub divide_output: u16,
    /// The vector unit in host vectors rather than element by element, where the host has them; in no state (Mars_Native.md §6.10).
    pub simd: Skip<bool>,
}

impl Default for Rsp {
    fn default() -> Self {
        Rsp {
            accumulator: [[0; 8]; 3],
            accumulator_top: [0; 8],
            broke: false,
            gpr: [0; 32],
            halted: true,
            next_pc: 0,
            pc: 0,
            vcc: 0,
            vce: 0,
            vco: 0,
            vector: [[0; 8]; 32],
            divide_input: 0,
            divide_input_loaded: false,
            divide_output: 0,
            simd: Skip(rsp::simd_default()),
        }
    }
}

impl State for Rsp {
    fn write_state(&self, w: &mut StateWriter) {
        w.u64s("Accumulator", &rsp::widen(&self.accumulator, &self.accumulator_top));
        w.bool("Broke", self.broke);
        w.u32s("Gpr", &self.gpr[..]);
        w.bool("Halted", self.halted);
        w.u32("NextPc", self.next_pc);
        w.u32("Pc", self.pc);
        w.u16("Vcc", self.vcc);
        w.u8("Vce", self.vce);
        w.u16("Vco", self.vco);
        w.u16s("Vector", self.vector.as_flattened());
        w.u16("_divideInput", self.divide_input);
        w.bool("_divideInputLoaded", self.divide_input_loaded);
        w.u16("_divideOutput", self.divide_output);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        let mut wide = [0u64; 8];
        r.u64s(&mut wide)?; // Accumulator
        (self.accumulator, self.accumulator_top) = rsp::narrow(&wide);
        self.broke = r.bool()?; // Broke
        r.u32s(&mut self.gpr[..])?; // Gpr
        self.halted = r.bool()?; // Halted
        self.next_pc = r.u32()?; // NextPc
        self.pc = r.u32()?; // Pc
        self.vcc = r.u16()?; // Vcc
        self.vce = r.u8()?; // Vce
        self.vco = r.u16()?; // Vco
        r.u16s(self.vector.as_flattened_mut())?; // Vector
        self.divide_input = r.u16()?; // _divideInput
        self.divide_input_loaded = r.bool()?; // _divideInputLoaded
        self.divide_output = r.u16()?; // _divideOutput
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct SpInterface {
    pub processor: Rsp,
    pub dram_address: u32,
    pub interrupt_on_break: bool,
    pub mem_address: u32,
    pub semaphore: bool,
    pub signals: u32,
    pub single_step: bool,
    /// The processor's coverage while `cov rsp` is armed, in no state; while it exists the idle loop steps the processor a cycle at a time (Mars_Native.md §6.5).
    pub trace: Skip<Option<Box<Trace>>>,
    /// IMEM decoded a word at a time, each entry checked against IMEM before it runs; in no state (Mars_Native.md §6.12).
    pub decoded: Skip<Decoded>,
}

impl State for SpInterface {
    fn write_state(&self, w: &mut StateWriter) {
        w.class("Processor", &self.processor);
        w.u32("_dramAddress", self.dram_address);
        w.bool("_interruptOnBreak", self.interrupt_on_break);
        w.u32("_memAddress", self.mem_address);
        w.bool("_semaphore", self.semaphore);
        w.u32("_signals", self.signals);
        w.bool("_singleStep", self.single_step);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        r.class(&mut self.processor)?; // Processor
        self.dram_address = r.u32()?; // _dramAddress
        self.interrupt_on_break = r.bool()?; // _interruptOnBreak
        self.mem_address = r.u32()?; // _memAddress
        self.semaphore = r.bool()?; // _semaphore
        self.signals = r.u32()?; // _signals
        self.single_step = r.bool()?; // _singleStep
        Ok(())
    }
}

pub const STATUS_HALT: u32 = 0x01;
pub const STATUS_BROKE: u32 = 0x02;
pub const STATUS_SINGLE_STEP: u32 = 0x20;
pub const STATUS_INTERRUPT_ON_BREAK: u32 = 0x40;
const SIGNAL_SHIFT: u32 = 7;
const IMEM_SELECT: u32 = 0x1000;
const MEM_SIZE: u32 = 0x1000;

/// The processor's memory, lent by the machine for one call: its own registers, IMEM and DMEM.
pub struct Lent<'a> {
    pub p: &'a mut Rsp,
    pub imem: &'a [u8; 4096],
    pub dmem: &'a mut [u8; 4096],
}

impl Memory for Lent<'_> {
    #[inline(always)]
    fn gpr(&self, register: usize) -> u32 {
        self.p.gpr[register & 31]
    }
    #[inline(always)]
    fn set_gpr(&mut self, register: usize, value: u32) {
        self.p.gpr[register & 31] = value
    }
    #[inline(always)]
    fn element(&self, register: usize, element: usize) -> u16 {
        self.p.vector[register & 31][element & 7]
    }
    #[inline(always)]
    fn set_element(&mut self, register: usize, element: usize, value: u16) {
        self.p.vector[register & 31][element & 7] = value
    }
    #[inline(always)]
    fn acc(&self, element: usize) -> u64 {
        let (a, i) = (&self.p.accumulator, element & 7);
        ((self.p.accumulator_top[i] as u64) << 48) | ((a[0][i] as u64) << 32) | ((a[1][i] as u64) << 16) | a[2][i] as u64
    }
    #[inline(always)]
    fn set_acc(&mut self, element: usize, value: u64) {
        let (a, i) = (&mut self.p.accumulator, element & 7);
        a[0][i] = (value >> 32) as u16;
        a[1][i] = (value >> 16) as u16;
        a[2][i] = value as u16;
        self.p.accumulator_top[i] = (value >> 48) as u16;
    }
    #[inline(always)]
    fn set_acc_low(&mut self, element: usize, value: u16) {
        self.p.accumulator[2][element & 7] = value
    }
    #[inline(always)]
    fn register(&self, register: usize) -> [u16; 8] {
        self.p.vector[register & 31]
    }
    #[inline(always)]
    fn set_register(&mut self, register: usize, value: [u16; 8]) {
        self.p.vector[register & 31] = value
    }
    #[inline(always)]
    fn third(&self, third: usize) -> [u16; 8] {
        self.p.accumulator[third]
    }
    #[inline(always)]
    fn set_third(&mut self, third: usize, value: [u16; 8]) {
        self.p.accumulator[third] = value
    }
    #[inline(always)]
    fn set_thirds(&mut self, value: [[u16; 8]; 3]) {
        self.p.accumulator = value;
        self.p.accumulator_top = [0; 8];
    }
    #[inline(always)]
    fn simd(&self) -> bool {
        *self.p.simd
    }
    #[inline(always)]
    fn data_block(&self, address: usize) -> [u8; 16] {
        self.dmem[address..address + 16].try_into().unwrap()
    }
    #[inline(always)]
    fn set_data_block(&mut self, address: usize, value: [u8; 16]) {
        self.dmem[address..address + 16].copy_from_slice(&value)
    }
    #[inline(always)]
    fn data(&self, address: u32) -> u8 {
        self.dmem[(address & DATA_MASK) as usize]
    }
    #[inline(always)]
    fn set_data(&mut self, address: u32, value: u8) {
        self.dmem[(address & DATA_MASK) as usize] = value
    }
    #[inline(always)]
    fn fetch(&self, pc: u32) -> u32 {
        fetch(self.imem, pc)
    }
    #[inline(always)]
    fn pc(&self) -> u32 {
        self.p.pc
    }
    #[inline(always)]
    fn set_pc(&mut self, value: u32) {
        self.p.pc = value
    }
    #[inline(always)]
    fn next_pc(&self) -> u32 {
        self.p.next_pc
    }
    #[inline(always)]
    fn set_next_pc(&mut self, value: u32) {
        self.p.next_pc = value
    }
    #[inline(always)]
    fn vco(&self) -> u16 {
        self.p.vco
    }
    #[inline(always)]
    fn set_vco(&mut self, value: u16) {
        self.p.vco = value
    }
    #[inline(always)]
    fn vcc(&self) -> u16 {
        self.p.vcc
    }
    #[inline(always)]
    fn set_vcc(&mut self, value: u16) {
        self.p.vcc = value
    }
    #[inline(always)]
    fn vce(&self) -> u8 {
        self.p.vce
    }
    #[inline(always)]
    fn set_vce(&mut self, value: u8) {
        self.p.vce = value
    }
    #[inline(always)]
    fn divide_input(&self) -> u16 {
        self.p.divide_input
    }
    #[inline(always)]
    fn set_divide_input(&mut self, value: u16) {
        self.p.divide_input = value
    }
    #[inline(always)]
    fn divide_output(&self) -> u16 {
        self.p.divide_output
    }
    #[inline(always)]
    fn set_divide_output(&mut self, value: u16) {
        self.p.divide_output = value
    }
    #[inline(always)]
    fn divide_loaded(&self) -> u8 {
        self.p.divide_input_loaded as u8
    }
    #[inline(always)]
    fn set_divide_loaded(&mut self, value: u8) {
        self.p.divide_input_loaded = value != 0
    }
    #[inline(always)]
    fn halted(&self) -> u8 {
        self.p.halted as u8
    }
}

#[inline(always)]
fn fetch(imem: &[u8; 4096], pc: u32) -> u32 {
    let at = (pc & PC_MASK) as usize;
    u32::from_be_bytes([imem[at], imem[at + 1], imem[at + 2], imem[at + 3]])
}

impl Rsp {
    /// `Start`.
    pub fn start(&mut self, pc: u32) {
        self.pc = pc & PC_MASK;
        self.next_pc = (self.pc + 4) & PC_MASK;
        self.halted = false;
    }
}

impl SpInterface {
    /// `StatusWord`: assembled rather than stored.
    pub fn status_word(&self) -> u32 {
        (if self.processor.halted { STATUS_HALT } else { 0 })
            | (if self.processor.broke { STATUS_BROKE } else { 0 })
            | (if self.single_step { STATUS_SINGLE_STEP } else { 0 })
            | (if self.interrupt_on_break { STATUS_INTERRUPT_ON_BREAK } else { 0 })
            | (self.signals << SIGNAL_SHIFT)
    }
}

/// `Asks`: a pair of bits with both asserted is no request at all.
#[inline]
fn asks(value: u32, clear_bit: u32, set: bool) -> bool {
    ((value >> clear_bit) & 3) == if set { 2 } else { 1 }
}

impl MemoryBus {
    /// One step to the next event, through the decoded table when it is on (Mars_Native.md §6.12).
    #[inline(always)]
    fn rsp_step_core(&mut self) -> bool {
        let (sp, imem, dmem) = (&mut self.sp, &self.sp_imem, &mut self.sp_dmem);
        let mut core = rsp::Rsp::over(Lent { p: &mut sp.processor, imem, dmem });
        if sp.decoded.on { sp.decoded.step(&mut core) } else { core.step() }
    }

    /// `Rsp::run`, through the decoded table when it is on.
    #[inline(always)]
    fn rsp_run_core(&mut self, budget: u64) -> u64 {
        let (sp, imem, dmem) = (&mut self.sp, &self.sp_imem, &mut self.sp_dmem);
        let mut core = rsp::Rsp::over(Lent { p: &mut sp.processor, imem, dmem });
        if sp.decoded.on { sp.decoded.run(&mut core, budget) } else { core.run(budget) }
    }

    pub fn sp_read32(&mut self, offset: u32) -> u32 {
        match offset & 0x1C {
            0x00 => self.sp.mem_address,
            0x04 => self.sp.dram_address,
            0x08 | 0x0C => 0xFF8,
            0x10 => self.sp.status_word(),
            0x14 | 0x18 => 0,
            _ => {
                let held = self.sp.semaphore as u32;
                self.sp.semaphore = true;
                held
            }
        }
    }

    pub fn sp_write32(&mut self, offset: u32, value: u32) {
        match offset & 0x1C {
            0x00 => self.sp.mem_address = value & 0x1FF8,
            0x04 => self.sp.dram_address = value & 0x00FF_FFF8,
            0x08 => self.sp_transfer(value, true),
            0x0C => self.sp_transfer(value, false),
            0x10 => self.sp_write_status(value),
            0x1C => self.sp.semaphore = false,
            _ => {}
        }
    }

    /// `Pc`'s setter: the program counter is a register of its own, a page away.
    pub fn sp_set_pc(&mut self, value: u32) {
        let p = &mut self.sp.processor;
        p.pc = value & PC_MASK;
        p.next_pc = (p.pc + 4) & PC_MASK;
    }

    fn sp_write_status(&mut self, value: u32) {
        if asks(value, 0, false) {
            let pc = self.sp.processor.pc;
            self.sp.processor.start(pc);
        }
        if asks(value, 0, true) {
            self.sp.processor.halted = true;
        }
        if value & 0x004 != 0 {
            self.sp.processor.broke = false;
        }
        if asks(value, 3, false) {
            self.mi.clear(interrupt::SIGNAL_PROCESSOR);
        }
        if asks(value, 3, true) {
            self.mi.raise(interrupt::SIGNAL_PROCESSOR);
        }
        if asks(value, 5, false) {
            self.sp.single_step = false;
        }
        if asks(value, 5, true) {
            self.sp.single_step = true;
        }
        if asks(value, 7, false) {
            self.sp.interrupt_on_break = false;
        }
        if asks(value, 7, true) {
            self.sp.interrupt_on_break = true;
        }
        for signal in 0..8u32 {
            if asks(value, 9 + signal * 2, false) {
                self.sp.signals &= !(1u32 << signal);
            }
            if asks(value, 9 + signal * 2, true) {
                self.sp.signals |= 1u32 << signal;
            }
        }
    }

    /// `SpInterface.Step`: at least one step, then on while cycles remain and nothing halted it; a single step halts after one.
    #[inline(always)]
    pub fn sp_step(&mut self, cycles: i64) {
        if self.sp.processor.halted {
            return;
        }
        if cycles <= 1 && !self.sp.single_step {
            if !self.rsp_step_core() {
                self.rsp_event();
                if self.sp.single_step {
                    self.sp.processor.halted = true;
                }
            }
            return;
        }
        self.sp_step_many(cycles);
    }

    #[inline(never)]
    fn sp_step_many(&mut self, cycles: i64) {
        if self.sp.single_step {
            self.rsp_step_one();
            self.sp.processor.halted = true;
            return;
        }
        let mut left = cycles.max(1) as u64;
        loop {
            let ran = self.rsp_run_core(left);
            left -= ran;
            if left == 0 {
                return;
            }
            self.rsp_event();
            if self.sp.single_step {
                self.sp.processor.halted = true;
                return;
            }
            left -= 1;
            if left == 0 || self.sp.processor.halted {
                return;
            }
        }
    }

    /// `StepOne`: one step for a caller that has tested the halt itself; an event runs here, in the machine.
    #[inline(always)]
    pub fn rsp_step_one(&mut self) {
        if self.sp.trace.is_some() {
            return self.rsp_step_one_traced();
        }
        if !self.rsp_step_core() {
            self.rsp_event();
        }
    }

    /// `sp_step` with every instruction recorded, the events included: C#'s managed step, which the native shortcut yields to while `cov rsp` is armed.
    #[inline(never)]
    pub(crate) fn sp_step_traced(&mut self, cycles: i64) {
        if self.sp.single_step {
            self.rsp_step_one_traced();
            self.sp.processor.halted = true;
            return;
        }
        let mut left = cycles.max(1) as u64;
        loop {
            let ran = self.rsp_run_traced(left);
            left -= ran;
            if left == 0 {
                return;
            }
            self.rsp_event_traced();
            left -= 1;
            if left == 0 || self.sp.processor.halted {
                return;
            }
        }
    }

    #[inline(never)]
    fn rsp_step_one_traced(&mut self) {
        if self.rsp_run_traced(1) == 0 {
            self.rsp_event_traced();
        }
    }

    fn rsp_run_traced(&mut self, budget: u64) -> u64 {
        let (sp, imem, dmem) = (&mut self.sp, &self.sp_imem, &mut self.sp_dmem);
        let trace = sp.trace.as_mut().expect("the trace is armed");
        rsp::Rsp::over(Lent { p: &mut sp.processor, imem, dmem }).run_traced(budget, trace)
    }

    fn rsp_event_traced(&mut self) {
        let pc = self.sp.processor.pc;
        if let Some(trace) = self.sp.trace.as_mut() {
            trace.record(pc);
        }
        self.rsp_event();
    }

    /// `NativeRunToEvent`: at most the budget, stopping after an event; a single step the event asked for halts, as a tick's would.
    pub fn rsp_run_to_event(&mut self, budget: i64) -> i64 {
        debug_assert!(self.sp.trace.is_none(), "a whole run is not taken while the processor's coverage is armed");
        if self.sp.processor.halted || budget <= 0 {
            return 0;
        }
        let mut ran = self.rsp_run_core(budget as u64) as i64;
        if ran < budget && !self.sp.processor.halted {
            self.rsp_event();
            if self.sp.single_step {
                self.sp.processor.halted = true;
            }
            ran += 1;
        }
        ran
    }

    /// The processor's steps up to its next event, the event left unrun; for measurement (Mars_Native.md §6.12).
    pub fn rsp_run_pure(&mut self, budget: u64) -> u64 {
        if self.sp.processor.halted { 0 } else { self.rsp_run_core(budget) }
    }

    /// `StepManaged` for the two instructions that reach the machine: a COP0 move and a break.
    pub fn rsp_event(&mut self) {
        let instruction = fetch(&self.sp_imem, self.sp.processor.pc);
        let p = &mut self.sp.processor;
        p.pc = p.next_pc;
        p.next_pc = (p.pc + 4) & PC_MASK;
        let op = instruction >> 26;
        if op == 0x10 {
            let register = (((instruction >> 11) & 0x1F) & 0x0F) as usize;
            let rt = ((instruction >> 16) & 0x1F) as usize;
            match (instruction >> 21) & 0x1F {
                0x00 => {
                    let value = self.read_rsp_control(register);
                    if rt != 0 {
                        self.sp.processor.gpr[rt] = value;
                    }
                }
                0x04 => {
                    let value = self.sp.processor.gpr[rt];
                    self.write_rsp_control(register, value);
                }
                _ => {}
            }
        } else if op == 0 && instruction & 0x3F == 0x0D {
            if !self.sp.processor.broke && self.sp.interrupt_on_break {
                self.mi.raise(interrupt::SIGNAL_PROCESSOR);
            }
            self.sp.processor.halted = true;
            self.sp.processor.broke = true;
        } else {
            unreachable!("the processor handed back {instruction:08X}, which is no event");
        }
    }

    /// `Transfer`: length encoded one short, and the row count and skip make it rectangular.
    fn sp_transfer(&mut self, encoded: u32, to_sp: bool) {
        let length = ((encoded & 0xFFF) | 7) + 1;
        let rows = ((encoded >> 12) & 0xFF) + 1;
        let skip = (encoded >> 20) & 0xFFF;
        let imem = self.sp.mem_address & IMEM_SELECT != 0;
        let mut bank_offset = self.sp.mem_address & 0xFF8;
        for _ in 0..rows {
            let dram = self.sp.dram_address;
            if to_sp {
                self.dp.wait_read_range(dram, length, site::SP_DMA);
            } else {
                self.dp.wait_write_range(dram, length, site::SP_DMA);
            }
            if dram as u64 + length as u64 <= self.rdram.len() as u64 {
                self.sp_copy_row(imem, bank_offset, length, to_sp);
            } else {
                for i in 0..length {
                    let sp_offset = ((bank_offset + i) % MEM_SIZE) as usize;
                    let dram_address = dram.wrapping_add(i);
                    if to_sp {
                        let byte = self.read8(dram_address);
                        if imem { self.sp_imem[sp_offset] = byte } else { self.sp_dmem[sp_offset] = byte }
                    } else {
                        let byte = if imem { self.sp_imem[sp_offset] } else { self.sp_dmem[sp_offset] };
                        self.write8(dram_address, byte);
                    }
                }
            }
            bank_offset = (bank_offset + length) % MEM_SIZE;
            self.sp.dram_address = dram.wrapping_add(length).wrapping_add(skip);
        }
        self.sp.mem_address = (self.sp.mem_address & IMEM_SELECT) | bank_offset;
    }

    /// `CopyRow`: a row inside RDRAM, in two runs where the bank wraps.
    fn sp_copy_row(&mut self, imem: bool, bank_offset: u32, length: u32, to_sp: bool) {
        let first = length.min(MEM_SIZE - bank_offset) as usize;
        let length = length as usize;
        let at = self.sp.dram_address as usize;
        let offset = bank_offset as usize;
        let bank = if imem { &mut self.sp_imem } else { &mut self.sp_dmem };
        let dram = &mut self.rdram[at..at + length];
        if to_sp {
            bank[offset..offset + first].copy_from_slice(&dram[..first]);
            bank[..length - first].copy_from_slice(&dram[first..]);
        } else {
            dram[..first].copy_from_slice(&bank[offset..offset + first]);
            dram[first..].copy_from_slice(&bank[..length - first]);
            *self.written += length as i64;
        }
    }
}
