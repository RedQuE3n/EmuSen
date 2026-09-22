//! The audio interface's serialized state, the C# `AiInterface`.

use std::collections::VecDeque;

use crate::Skip;
use crate::bus::MemoryBus;
use crate::mi::interrupt;
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
    /// `_debtAt`: where the debt was last brought up to.
    pub debt_at: Skip<i64>,
    /// `Due`: the first cycle a sample is owed.
    pub due: Skip<i64>,
    /// `_samples`: what the frontend has not drained.
    pub samples: Skip<VecDeque<i16>>,
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

pub const STATUS_FULL: u32 = 0x8000_0001;
pub const STATUS_BUSY: u32 = 0x4000_0000;
pub const STATUS_ENABLED: u32 = 0x0200_0000;
pub const STATUS_ALWAYS_SET: u32 = 0x0110_0000;
/// `MaxBufferedSamples`: past this, undrained samples are dropped oldest first, a pair at a time.
pub const MAX_BUFFERED_SAMPLES: usize = 128_000;
pub const DEFAULT_SAMPLE_RATE: i32 = 44_100;
pub const SHORTEST_PERIOD: u32 = 0x200;
pub const PAGE_SIZE: u32 = 0x2000;
const PROCESSOR_CLOCK: i64 = 93_750_000;
const ADDRESS_MASK: u32 = 0x00FF_FFF8;
const LENGTH_MASK: u32 = 0x0003_FFF8;

impl AiInterface {
    #[inline]
    fn period(&self) -> u32 {
        (self.dac_rate + 1).max(SHORTEST_PERIOD)
    }

    /// `SampleRate`: the DAC divides the video clock; before a game sets a rate, the default stands.
    pub fn sample_rate(&self, video_clock: i64) -> i32 {
        if self.dac_rate == 0 { DEFAULT_SAMPLE_RATE } else { (video_clock as f64 / self.period() as f64).round_ties_even() as i32 }
    }

    pub fn read32(&self, offset: u32) -> u32 {
        match offset & 0x1C {
            0x0C => {
                (if self.queued > 1 { STATUS_FULL } else { 0 })
                    | (if self.queued > 0 { STATUS_BUSY } else { 0 })
                    | (if self.dma_enabled { STATUS_ENABLED } else { 0 })
                    | STATUS_ALWAYS_SET
            }
            _ => self.length,
        }
    }

    /// `Drain`: destructive, interleaved left then right, whole pairs only.
    pub fn drain(&mut self, max_frames: usize) -> Vec<i16> {
        let mut wanted = (max_frames.saturating_mul(2)).min(self.samples.len());
        wanted -= wanted & 1;
        self.samples.drain(..wanted).collect()
    }

    /// `DropUndrained`: a loaded state starts the frontend's queue afresh.
    pub fn drop_undrained(&mut self) {
        self.samples.clear();
    }

    fn enqueue(&mut self, left: i16, right: i16) {
        if self.samples.len() + 2 > MAX_BUFFERED_SAMPLES {
            self.samples.pop_front();
            self.samples.pop_front();
        }
        self.samples.push_back(left);
        self.samples.push_back(right);
    }
}

impl MemoryBus {
    pub fn ai_write32(&mut self, offset: u32, value: u32) {
        self.settle();
        self.ai_apply(offset, value);
        self.reschedule();
    }

    fn ai_apply(&mut self, offset: u32, value: u32) {
        match offset & 0x1C {
            0x00 => {
                if self.ai.queued == 0 {
                    self.ai.address = value & ADDRESS_MASK;
                } else if self.ai.queued == 1 {
                    self.ai.next_address = value & ADDRESS_MASK;
                }
            }
            0x04 => self.ai_queue(value & LENGTH_MASK),
            0x08 => self.ai.dma_enabled = value & 1 != 0,
            0x0C => self.mi.clear(interrupt::AUDIO_INTERFACE),
            0x10 => self.ai.dac_rate = value & 0x3FFF,
            _ => {}
        }
    }

    /// `Settle`: an idle tick zeroes the debt, as every idle step once did.
    pub fn ai_settle(&mut self) {
        let now = self.cycles;
        if now == *self.ai.debt_at {
            return;
        }
        if self.ai.queued == 0 || !self.ai.dma_enabled {
            self.ai.debt = 0;
        } else {
            self.ai.debt = self.ai.debt.wrapping_add((now - *self.ai.debt_at).wrapping_mul(self.vi.video_clock()));
        }
        *self.ai.debt_at = now;
    }

    pub fn ai_catch(&mut self) {
        if self.cycles < *self.ai.due {
            return;
        }
        self.ai_settle();
        let period = self.ai.period() as i64 * PROCESSOR_CLOCK;
        while self.ai.debt >= period && self.ai.queued > 0 {
            self.ai.debt -= period;
            self.ai_play();
        }
        self.ai_schedule();
    }

    pub fn ai_schedule(&mut self) {
        if self.ai.queued == 0 || !self.ai.dma_enabled {
            *self.ai.due = i64::MAX;
            return;
        }
        let clock = self.vi.video_clock();
        let remaining = self.ai.period() as i64 * PROCESSOR_CLOCK - self.ai.debt;
        *self.ai.due = if remaining <= 0 { *self.ai.debt_at } else { *self.ai.debt_at + (remaining + clock - 1) / clock };
    }

    /// `Rebase`: a loaded state was settled when it was saved.
    pub fn ai_rebase(&mut self) {
        *self.ai.debt_at = self.cycles;
    }

    fn ai_queue(&mut self, length: u32) {
        if self.ai.queued == 0 {
            self.ai.length = length;
            self.ai.queued = 1;
            self.ai_begin();
        } else if self.ai.queued == 1 {
            self.ai.next_length = length;
            self.ai.queued = 2;
        }
    }

    /// `Begin`: the interrupt marks a buffer beginning, not one ending.
    fn ai_begin(&mut self) {
        self.mi.raise(interrupt::AUDIO_INTERFACE);
        if self.ai.length == 0 {
            self.ai_end();
        }
    }

    fn ai_end(&mut self) {
        if self.ai.queued < 2 {
            self.ai.queued = 0;
            return;
        }
        self.ai.address = self.ai.next_address;
        self.ai.length = self.ai.next_length;
        self.ai.queued = 1;
        self.ai_begin();
    }

    /// `Play`: one stereo sample, big-endian, left first; the carry into the next page lands one fetch late.
    fn ai_play(&mut self) {
        if self.ai.carry {
            self.ai.address = self.ai.address.wrapping_add(PAGE_SIZE) & ADDRESS_MASK;
            self.ai.carry = false;
        }
        let at = self.ai.address as usize;
        let (mut left, mut right) = (0i16, 0i16);
        if at + 3 < self.rdram.len() {
            left = i16::from_be_bytes([self.rdram[at], self.rdram[at + 1]]);
            right = i16::from_be_bytes([self.rdram[at + 2], self.rdram[at + 3]]);
        }
        self.ai.enqueue(left, right);
        self.ai.samples_played += 1;
        self.ai.address = (self.ai.address & !(PAGE_SIZE - 1)) | (self.ai.address.wrapping_add(4) & (PAGE_SIZE - 1));
        if self.ai.address & (PAGE_SIZE - 1) == 0 {
            self.ai.carry = true;
        }
        self.ai.length = self.ai.length.wrapping_sub(4);
        if self.ai.length == 0 {
            self.ai_end();
        }
    }
}
