//! C#'s `Channels.cs` and `DmcChannel.cs`: the envelope, both pulses, the triangle, the noise and the DMC. See Moon_APU.md §3.

use crate::Skip;
use crate::state::{State, StateReader, StateResult, StateWriter};

pub static LENGTH_COUNTER: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30,
];
static PULSE_DUTY: [[u8; 8]; 4] = [[0, 1, 0, 0, 0, 0, 0, 0], [0, 1, 1, 0, 0, 0, 0, 0], [0, 1, 1, 1, 1, 0, 0, 0], [1, 0, 0, 1, 1, 1, 1, 1]];
static TRIANGLE_SEQUENCE: [u8; 32] = [15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
static NOISE_PERIOD: [i32; 16] = [4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068];
static DMC_RATE: [i32; 16] = [428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106, 84, 72, 54];

/// A table read whose index a loaded state can set: C#'s exception becomes a fault and zero.
#[inline(always)]
fn table<T: Copy + Default>(t: &[T], i: i32) -> T {
    match t.get(i as usize) {
        Some(&v) => v,
        None => {
            crate::fault(crate::Fault::IndexOutOfRange);
            T::default()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Envelope {
    pub constant_volume: bool,
    pub r#loop: bool,
    pub volume: i32,
    pub decay: i32,
    pub divider: i32,
    pub start: bool,
}

impl Envelope {
    #[inline(always)]
    pub fn restart(&mut self) {
        self.start = true;
    }

    #[inline(always)]
    pub fn output(&self) -> i32 {
        if self.constant_volume { self.volume } else { self.decay }
    }

    pub fn clock(&mut self) {
        if self.start {
            self.start = false;
            self.decay = 15;
            self.divider = self.volume;
            return;
        }
        if self.divider > 0 {
            self.divider -= 1;
            return;
        }
        self.divider = self.volume;
        if self.decay > 0 {
            self.decay -= 1;
        } else if self.r#loop {
            self.decay = 15;
        }
    }
}

impl State for Envelope {
    type Error = crate::state::StateError;
    fn write_state(&self, w: &mut StateWriter) {
        w.bool("ConstantVolume", self.constant_volume);
        w.bool("Loop", self.r#loop);
        w.i32("Volume", self.volume);
        w.i32("_decay", self.decay);
        w.i32("_divider", self.divider);
        w.bool("_start", self.start);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        self.constant_volume = r.bool()?; // ConstantVolume
        self.r#loop = r.bool()?; // Loop
        self.volume = r.i32()?; // Volume
        self.decay = r.i32()?; // _decay
        self.divider = r.i32()?; // _divider
        self.start = r.bool()?; // _start
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct PulseChannel {
    pub duty: i32,
    pub envelope: Envelope,
    pub length_counter: i32,
    pub length_halted: bool,
    pub sweep_enabled: bool,
    pub sweep_negate: bool,
    pub sweep_period: i32,
    pub sweep_shift: i32,
    pub timer_period: i32,
    pub enabled: bool,
    pub ones_complement: bool,
    pub sequence_step: i32,
    pub sweep_divider: i32,
    pub sweep_reload: bool,
    pub timer: i32,
}

impl PulseChannel {
    pub fn new(ones_complement: bool) -> Self {
        PulseChannel { ones_complement, ..Default::default() }
    }

    #[inline(always)]
    pub fn set_enabled(&mut self, value: bool) {
        self.enabled = value;
        if !value {
            self.length_counter = 0;
        }
    }

    pub fn restart_sequencer(&mut self) {
        self.sequence_step = 0;
        self.envelope.restart();
    }

    #[inline(always)]
    fn sweep_target(&self) -> i32 {
        let change = self.timer_period.wrapping_shr(self.sweep_shift as u32);
        if !self.sweep_negate {
            return self.timer_period.wrapping_add(change);
        }
        self.timer_period.wrapping_sub(change).wrapping_sub(if self.ones_complement { 1 } else { 0 })
    }

    #[inline(always)]
    fn muted(&self) -> bool {
        self.timer_period < 8 || self.sweep_target() > 0x7FF
    }

    #[inline(always)]
    pub fn output(&self) -> i32 {
        if self.length_counter == 0 || self.muted() {
            0
        } else {
            let duty = table(&PULSE_DUTY, self.duty);
            table(&duty, self.sequence_step) as i32 * self.envelope.output()
        }
    }

    #[inline(always)]
    pub fn step_timer(&mut self) {
        if self.timer > 0 {
            self.timer -= 1;
            return;
        }
        self.timer = self.timer_period;
        self.sequence_step = (self.sequence_step + 1) & 0x07;
    }

    pub fn clock_length(&mut self) {
        if !self.length_halted && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    pub fn clock_sweep(&mut self) {
        if self.sweep_divider == 0 && self.sweep_enabled && self.sweep_shift > 0 && !self.muted() {
            self.timer_period = self.sweep_target();
        }
        if self.sweep_divider == 0 || self.sweep_reload {
            self.sweep_divider = self.sweep_period;
            self.sweep_reload = false;
        } else {
            self.sweep_divider -= 1;
        }
    }
}

impl State for PulseChannel {
    type Error = crate::state::StateError;
    fn write_state(&self, w: &mut StateWriter) {
        w.i32("Duty", self.duty);
        w.class("Envelope", &self.envelope);
        w.i32("LengthCounter", self.length_counter);
        w.bool("LengthHalted", self.length_halted);
        w.bool("SweepEnabled", self.sweep_enabled);
        w.bool("SweepNegate", self.sweep_negate);
        w.i32("SweepPeriod", self.sweep_period);
        w.i32("SweepShift", self.sweep_shift);
        w.i32("TimerPeriod", self.timer_period);
        w.bool("_enabled", self.enabled);
        w.bool("_onesComplement", self.ones_complement);
        w.i32("_sequenceStep", self.sequence_step);
        w.i32("_sweepDivider", self.sweep_divider);
        w.bool("_sweepReload", self.sweep_reload);
        w.i32("_timer", self.timer);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        self.duty = r.i32()?; // Duty
        r.class(&mut self.envelope)?; // Envelope
        self.length_counter = r.i32()?; // LengthCounter
        self.length_halted = r.bool()?; // LengthHalted
        self.sweep_enabled = r.bool()?; // SweepEnabled
        self.sweep_negate = r.bool()?; // SweepNegate
        self.sweep_period = r.i32()?; // SweepPeriod
        self.sweep_shift = r.i32()?; // SweepShift
        self.timer_period = r.i32()?; // TimerPeriod
        self.enabled = r.bool()?; // _enabled
        self.ones_complement = r.bool()?; // _onesComplement
        self.sequence_step = r.i32()?; // _sequenceStep
        self.sweep_divider = r.i32()?; // _sweepDivider
        self.sweep_reload = r.bool()?; // _sweepReload
        self.timer = r.i32()?; // _timer
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct TriangleChannel {
    pub control_flag: bool,
    pub length_counter: i32,
    pub linear_reload_value: i32,
    pub timer_period: i32,
    pub enabled: bool,
    pub linear_counter: i32,
    pub linear_reload: bool,
    pub sequence_step: i32,
    pub timer: i32,
}

impl TriangleChannel {
    #[inline(always)]
    pub fn set_enabled(&mut self, value: bool) {
        self.enabled = value;
        if !value {
            self.length_counter = 0;
        }
    }

    #[inline(always)]
    pub fn output(&self) -> i32 {
        if self.timer_period < 2 { 0 } else { table(&TRIANGLE_SEQUENCE, self.sequence_step) as i32 }
    }

    #[inline(always)]
    pub fn step_timer(&mut self) {
        if self.length_counter == 0 || self.linear_counter == 0 {
            return;
        }
        if self.timer > 0 {
            self.timer -= 1;
            return;
        }
        self.timer = self.timer_period;
        self.sequence_step = (self.sequence_step + 1) & 0x1F;
    }

    pub fn clock_linear(&mut self) {
        if self.linear_reload {
            self.linear_counter = self.linear_reload_value;
        } else if self.linear_counter > 0 {
            self.linear_counter -= 1;
        }
        if !self.control_flag {
            self.linear_reload = false;
        }
    }

    pub fn clock_length(&mut self) {
        if !self.control_flag && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }
}

impl State for TriangleChannel {
    type Error = crate::state::StateError;
    fn write_state(&self, w: &mut StateWriter) {
        w.bool("ControlFlag", self.control_flag);
        w.i32("LengthCounter", self.length_counter);
        w.i32("LinearReloadValue", self.linear_reload_value);
        w.i32("TimerPeriod", self.timer_period);
        w.bool("_enabled", self.enabled);
        w.i32("_linearCounter", self.linear_counter);
        w.bool("_linearReload", self.linear_reload);
        w.i32("_sequenceStep", self.sequence_step);
        w.i32("_timer", self.timer);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        self.control_flag = r.bool()?; // ControlFlag
        self.length_counter = r.i32()?; // LengthCounter
        self.linear_reload_value = r.i32()?; // LinearReloadValue
        self.timer_period = r.i32()?; // TimerPeriod
        self.enabled = r.bool()?; // _enabled
        self.linear_counter = r.i32()?; // _linearCounter
        self.linear_reload = r.bool()?; // _linearReload
        self.sequence_step = r.i32()?; // _sequenceStep
        self.timer = r.i32()?; // _timer
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NoiseChannel {
    pub envelope: Envelope,
    pub length_counter: i32,
    pub length_halted: bool,
    pub period_index: i32,
    pub short_mode: bool,
    pub enabled: bool,
    pub shift: i32,
    pub timer: i32,
}

impl Default for NoiseChannel {
    fn default() -> Self {
        NoiseChannel { envelope: Envelope::default(), length_counter: 0, length_halted: false, period_index: 0, short_mode: false, enabled: false, shift: 1, timer: 0 }
    }
}

impl NoiseChannel {
    #[inline(always)]
    pub fn set_enabled(&mut self, value: bool) {
        self.enabled = value;
        if !value {
            self.length_counter = 0;
        }
    }

    #[inline(always)]
    pub fn output(&self) -> i32 {
        if self.length_counter == 0 || (self.shift & 0x01) != 0 { 0 } else { self.envelope.output() }
    }

    #[inline(always)]
    pub fn step_timer(&mut self) {
        if self.timer > 0 {
            self.timer -= 1;
            return;
        }
        self.timer = table(&NOISE_PERIOD, self.period_index);
        let tap = if self.short_mode { (self.shift >> 6) & 0x01 } else { (self.shift >> 1) & 0x01 };
        let feedback = (self.shift & 0x01) ^ tap;
        self.shift = (self.shift >> 1) | (feedback << 14);
    }

    pub fn clock_length(&mut self) {
        if !self.length_halted && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }
}

impl State for NoiseChannel {
    type Error = crate::state::StateError;
    fn write_state(&self, w: &mut StateWriter) {
        w.class("Envelope", &self.envelope);
        w.i32("LengthCounter", self.length_counter);
        w.bool("LengthHalted", self.length_halted);
        w.i32("PeriodIndex", self.period_index);
        w.bool("ShortMode", self.short_mode);
        w.bool("_enabled", self.enabled);
        w.i32("_shift", self.shift);
        w.i32("_timer", self.timer);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        r.class(&mut self.envelope)?; // Envelope
        self.length_counter = r.i32()?; // LengthCounter
        self.length_halted = r.bool()?; // LengthHalted
        self.period_index = r.i32()?; // PeriodIndex
        self.short_mode = r.bool()?; // ShortMode
        self.enabled = r.bool()?; // _enabled
        self.shift = r.i32()?; // _shift
        self.timer = r.i32()?; // _timer
        Ok(())
    }
}

/// `DmcChannel`; the fetch through the bus is split so the APU can perform it (Moon_Native.md §2.5).
#[derive(Clone, Debug, PartialEq)]
pub struct DmcChannel {
    pub irq_enabled: bool,
    pub irq_pending: bool,
    pub r#loop: bool,
    pub output_level: i32,
    pub rate_index: i32,
    pub sample_address: i32,
    pub sample_length: i32,
    pub stall_cycles: i32,
    pub bits_remaining: i32,
    pub bytes_remaining: i32,
    pub current_address: i32,
    pub enabled: bool,
    pub shift: i32,
    pub silence: bool,
    pub timer: i32,
    pub sample_buffer: Skip<u8>,
    pub buffer_full: Skip<bool>,
    pub load_delay: Skip<i32>,
}

impl Default for DmcChannel {
    fn default() -> Self {
        DmcChannel {
            irq_enabled: false,
            irq_pending: false,
            r#loop: false,
            output_level: 0,
            rate_index: 0,
            sample_address: 0,
            sample_length: 0,
            stall_cycles: 0,
            bits_remaining: 0,
            bytes_remaining: 0,
            current_address: 0,
            enabled: false,
            shift: 0,
            silence: true,
            timer: 0,
            sample_buffer: Skip(0),
            buffer_full: Skip(false),
            load_delay: Skip(0),
        }
    }
}

impl DmcChannel {
    #[inline(always)]
    pub fn active(&self) -> bool {
        self.bytes_remaining > 0
    }

    /// The reader wants the bus: the buffer is empty, bytes remain, and no load delay is running.
    #[inline(always)]
    pub fn dma_requested(&self) -> bool {
        !*self.buffer_full && ((self.bytes_remaining > 0 && *self.load_delay == 0) || *self.load_delay < 0)
    }

    #[inline(always)]
    pub fn dma_address(&self) -> u16 {
        self.current_address as u16
    }

    /// `SetEnabled`: an empty buffer asks for its first byte three cycles later on a get, two on a put.
    pub fn set_enabled(&mut self, value: bool, on_get_cycle: bool) {
        self.enabled = value;
        self.irq_pending = false;
        if !value {
            *self.load_delay = if self.bytes_remaining > 0 && *self.load_delay == 0 { if on_get_cycle { -3 } else { -2 } } else { 0 };
            self.bytes_remaining = 0;
        } else if self.bytes_remaining == 0 {
            self.restart();
            if !*self.buffer_full && self.bytes_remaining > 0 {
                *self.load_delay = if on_get_cycle { 3 } else { 2 };
            }
        }
    }

    pub fn restart(&mut self) {
        self.current_address = self.sample_address;
        self.bytes_remaining = self.sample_length;
    }

    /// `StepTimer`: the table is the period, so the count reloads one short.
    #[inline(always)]
    pub fn step_timer(&mut self) {
        if *self.load_delay > 0 {
            *self.load_delay -= 1;
        } else if *self.load_delay < 0 {
            *self.load_delay += 1;
        }
        if self.timer > 0 {
            self.timer -= 1;
            return;
        }
        self.timer = table(&DMC_RATE, self.rate_index) - 1;
        self.clock();
    }

    fn clock(&mut self) {
        if !self.silence {
            if (self.shift & 0x01) != 0 {
                if self.output_level <= 125 {
                    self.output_level += 2;
                }
            } else if self.output_level >= 2 {
                self.output_level -= 2;
            }
        }
        self.shift >>= 1;
        self.bits_remaining = self.bits_remaining.wrapping_sub(1);
        if self.bits_remaining > 0 {
            return;
        }
        self.bits_remaining = 8;
        if !*self.buffer_full {
            self.silence = true;
            return;
        }
        self.silence = false;
        self.shift = *self.sample_buffer as i32;
        *self.buffer_full = false;
    }

    /// `CompleteDma`: the get cycle's byte fills the buffer and the sample moves on.
    pub fn complete_dma(&mut self, value: u8) {
        *self.sample_buffer = value;
        *self.buffer_full = true;
        *self.load_delay = 0;
        if self.bytes_remaining == 0 {
            return;
        }
        self.current_address = if self.current_address == 0xFFFF { 0x8000 } else { self.current_address.wrapping_add(1) };
        self.bytes_remaining = self.bytes_remaining.wrapping_sub(1);
        if self.bytes_remaining != 0 {
            return;
        }
        if self.r#loop {
            self.restart();
            return;
        }
        if self.irq_enabled {
            self.irq_pending = true;
        }
        *self.load_delay = -3;
    }

    pub fn reset(&mut self) {
        *self = DmcChannel { sample_address: 0xC000, bits_remaining: 8, ..Default::default() };
    }
}

impl State for DmcChannel {
    type Error = crate::state::StateError;
    fn write_state(&self, w: &mut StateWriter) {
        w.bool("IrqEnabled", self.irq_enabled);
        w.bool("IrqPending", self.irq_pending);
        w.bool("Loop", self.r#loop);
        w.i32("OutputLevel", self.output_level);
        w.i32("RateIndex", self.rate_index);
        w.i32("SampleAddress", self.sample_address);
        w.i32("SampleLength", self.sample_length);
        w.i32("StallCycles", self.stall_cycles);
        w.i32("_bitsRemaining", self.bits_remaining);
        w.i32("_bytesRemaining", self.bytes_remaining);
        w.i32("_currentAddress", self.current_address);
        w.bool("_enabled", self.enabled);
        w.i32("_shift", self.shift);
        w.bool("_silence", self.silence);
        w.i32("_timer", self.timer);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        self.irq_enabled = r.bool()?; // IrqEnabled
        self.irq_pending = r.bool()?; // IrqPending
        self.r#loop = r.bool()?; // Loop
        self.output_level = r.i32()?; // OutputLevel
        self.rate_index = r.i32()?; // RateIndex
        self.sample_address = r.i32()?; // SampleAddress
        self.sample_length = r.i32()?; // SampleLength
        self.stall_cycles = r.i32()?; // StallCycles
        self.bits_remaining = r.i32()?; // _bitsRemaining
        self.bytes_remaining = r.i32()?; // _bytesRemaining
        self.current_address = r.i32()?; // _currentAddress
        self.enabled = r.bool()?; // _enabled
        self.shift = r.i32()?; // _shift
        self.silence = r.bool()?; // _silence
        self.timer = r.i32()?; // _timer
        Ok(())
    }
}
