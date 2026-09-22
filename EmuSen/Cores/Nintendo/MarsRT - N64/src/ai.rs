//! The audio interface's serialized state, the C# `AiInterface`.

use crate::state::{State, StateReader, StateResult, StateWriter};

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct AiInterface {
    /// `<SamplesPlayed>k__BackingField`, the auto-property's backing field.
    pub samples_played: i64,
    pub address: u32,
    pub carry: bool,
    pub dac_rate: u32,
    pub debt: i64,
    pub dma_enabled: bool,
    pub length: u32,
    pub next_address: u32,
    pub next_length: u32,
    pub queued: i32,
}

impl State for AiInterface {
    fn write_state(&self, w: &mut StateWriter) {
        w.i64("<SamplesPlayed>k__BackingField", self.samples_played);
        w.u32("_address", self.address);
        w.bool("_carry", self.carry);
        w.u32("_dacRate", self.dac_rate);
        w.i64("_debt", self.debt);
        w.bool("_dmaEnabled", self.dma_enabled);
        w.u32("_length", self.length);
        w.u32("_nextAddress", self.next_address);
        w.u32("_nextLength", self.next_length);
        w.i32("_queued", self.queued);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        self.samples_played = r.i64()?; // <SamplesPlayed>k__BackingField
        self.address = r.u32()?; // _address
        self.carry = r.bool()?; // _carry
        self.dac_rate = r.u32()?; // _dacRate
        self.debt = r.i64()?; // _debt
        self.dma_enabled = r.bool()?; // _dmaEnabled
        self.length = r.u32()?; // _length
        self.next_address = r.u32()?; // _nextAddress
        self.next_length = r.u32()?; // _nextLength
        self.queued = r.i32()?; // _queued
        Ok(())
    }
}
