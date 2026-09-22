//! The signal processor's interface and registers, the C# `SpInterface` and `Rsp`.

use crate::state::{State, StateReader, StateResult, StateWriter};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rsp {
    pub accumulator: [u64; 8],
    pub broke: bool,
    pub gpr: [u32; 32],
    pub halted: bool,
    pub next_pc: u32,
    pub pc: u32,
    pub vcc: u16,
    pub vce: u8,
    pub vco: u16,
    pub vector: [u16; 256],
    pub divide_input: u16,
    pub divide_input_loaded: bool,
    pub divide_output: u16,
}

impl Default for Rsp {
    fn default() -> Self {
        Rsp {
            accumulator: [0; 8],
            broke: false,
            gpr: [0; 32],
            halted: false,
            next_pc: 0,
            pc: 0,
            vcc: 0,
            vce: 0,
            vco: 0,
            vector: [0; 256],
            divide_input: 0,
            divide_input_loaded: false,
            divide_output: 0,
        }
    }
}

impl State for Rsp {
    fn write_state(&self, w: &mut StateWriter) {
        w.u64s("Accumulator", &self.accumulator[..]);
        w.bool("Broke", self.broke);
        w.u32s("Gpr", &self.gpr[..]);
        w.bool("Halted", self.halted);
        w.u32("NextPc", self.next_pc);
        w.u32("Pc", self.pc);
        w.u16("Vcc", self.vcc);
        w.u8("Vce", self.vce);
        w.u16("Vco", self.vco);
        w.u16s("Vector", &self.vector[..]);
        w.u16("_divideInput", self.divide_input);
        w.bool("_divideInputLoaded", self.divide_input_loaded);
        w.u16("_divideOutput", self.divide_output);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        r.u64s(&mut self.accumulator[..])?; // Accumulator
        self.broke = r.bool()?; // Broke
        r.u32s(&mut self.gpr[..])?; // Gpr
        self.halted = r.bool()?; // Halted
        self.next_pc = r.u32()?; // NextPc
        self.pc = r.u32()?; // Pc
        self.vcc = r.u16()?; // Vcc
        self.vce = r.u8()?; // Vce
        self.vco = r.u16()?; // Vco
        r.u16s(&mut self.vector[..])?; // Vector
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
