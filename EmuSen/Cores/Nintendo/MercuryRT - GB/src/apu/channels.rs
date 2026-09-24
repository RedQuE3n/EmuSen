//! C#'s `Apu/Channels.cs`: the envelope, the length counter and the four channels. See Mercury_Apu.md §3.

use crate::state::{State, StateReader, StateResult, StateWriter};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Envelope {
    pub finished: bool,
    pub increasing: bool,
    pub initial_volume: i32,
    pub period: i32,
    pub timer: i32,
    pub volume: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LengthCounter {
    pub counter: i32,
    pub enabled: bool,
    /// Readonly in C#, and still serialized, so a state can change it.
    pub maximum: i32,
}

impl LengthCounter {
    pub fn new(maximum: i32) -> Self {
        LengthCounter { counter: 0, enabled: false, maximum }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PulseChannel {
    pub dac_enabled: bool,
    pub duty: i32,
    pub enabled: bool,
    pub envelope: Envelope,
    pub frequency: i32,
    /// Readonly in C#, and still serialized.
    pub has_sweep: bool,
    pub length: LengthCounter,
    pub step: i32,
    pub sweep_negate: bool,
    pub sweep_period: i32,
    pub sweep_running: bool,
    pub sweep_shadow: i32,
    pub sweep_shift: i32,
    pub sweep_timer: i32,
    pub timer: i32,
}

impl PulseChannel {
    pub fn new(has_sweep: bool) -> Self {
        PulseChannel {
            dac_enabled: false,
            duty: 0,
            enabled: false,
            envelope: Envelope::default(),
            frequency: 0,
            has_sweep,
            length: LengthCounter::new(64),
            step: 0,
            sweep_negate: false,
            sweep_period: 0,
            sweep_running: false,
            sweep_shadow: 0,
            sweep_shift: 0,
            sweep_timer: 0,
            timer: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WaveChannel {
    pub dac_enabled: bool,
    pub enabled: bool,
    pub frequency: i32,
    pub length: LengthCounter,
    pub position: i32,
    pub ram: [u8; 16],
    pub timer: i32,
    pub volume_code: i32,
}

impl Default for WaveChannel {
    fn default() -> Self {
        WaveChannel { dac_enabled: false, enabled: false, frequency: 0, length: LengthCounter::new(256), position: 0, ram: [0; 16], timer: 0, volume_code: 0 }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoiseChannel {
    pub clock_shift: i32,
    pub dac_enabled: bool,
    pub divisor_code: i32,
    pub enabled: bool,
    pub envelope: Envelope,
    pub length: LengthCounter,
    pub lfsr: i32,
    pub short_mode: bool,
    pub timer: i32,
}

impl Default for NoiseChannel {
    fn default() -> Self {
        NoiseChannel {
            clock_shift: 0,
            dac_enabled: false,
            divisor_code: 0,
            enabled: false,
            envelope: Envelope::default(),
            length: LengthCounter::new(64),
            lfsr: 0x7FFF,
            short_mode: false,
            timer: 0,
        }
    }
}

const DUTY_PATTERNS: [[u8; 8]; 4] = [[0, 0, 0, 0, 0, 0, 0, 1], [1, 0, 0, 0, 0, 0, 0, 1], [1, 0, 0, 0, 0, 1, 1, 1], [0, 1, 1, 1, 1, 1, 1, 0]];
const DIVISORS: [i32; 8] = [8, 16, 32, 48, 64, 80, 96, 112];

impl Envelope {
    pub fn load(&mut self, nrx2: u8) {
        self.initial_volume = (nrx2 >> 4) as i32;
        self.increasing = nrx2 & 0x08 != 0;
        self.period = (nrx2 & 0x07) as i32;
    }

    pub fn trigger(&mut self) {
        self.volume = self.initial_volume;
        self.timer = if self.period == 0 { 8 } else { self.period };
        self.finished = false;
    }

    /// A period of zero is off, not "step every tick" - see Mercury_Apu.md §3.1.
    pub fn clock(&mut self) {
        if self.period == 0 || self.finished {
            return;
        }
        self.timer = self.timer.wrapping_sub(1);
        if self.timer > 0 {
            return;
        }
        self.timer = self.period;
        let next = self.volume + if self.increasing { 1 } else { -1 };
        if !(0..=15).contains(&next) {
            self.finished = true;
            return;
        }
        self.volume = next;
    }
}

impl LengthCounter {
    pub fn load(&mut self, value: i32) {
        self.counter = self.maximum.wrapping_sub(value);
    }

    pub fn trigger(&mut self) {
        if self.counter == 0 {
            self.counter = self.maximum;
        }
    }

    /// True on the tick it reaches zero, which is the tick the channel goes quiet.
    pub fn clock(&mut self) -> bool {
        if !self.enabled || self.counter == 0 {
            return false;
        }
        self.counter = self.counter.wrapping_sub(1);
        self.counter == 0
    }
}

impl PulseChannel {
    #[inline(always)]
    pub fn output(&self) -> i32 {
        if self.enabled && self.dac_enabled && DUTY_PATTERNS[self.duty as usize][self.step as usize] != 0 { self.envelope.volume } else { 0 }
    }

    #[inline(always)]
    pub fn step_timer(&mut self) {
        self.timer = self.timer.wrapping_sub(1);
        if self.timer > 0 {
            return;
        }
        self.timer = (2048 - self.frequency) * 4;
        self.step = (self.step + 1) & 0x07;
    }

    pub fn trigger(&mut self) {
        self.enabled = self.dac_enabled;
        self.timer = (2048 - self.frequency) * 4;
        self.envelope.trigger();
        self.length.trigger();
        if !self.has_sweep {
            return;
        }
        self.sweep_shadow = self.frequency;
        self.sweep_timer = if self.sweep_period == 0 { 8 } else { self.sweep_period };
        self.sweep_running = self.sweep_period > 0 || self.sweep_shift > 0;
        if self.sweep_shift > 0 {
            self.compute_sweep(false);
        }
    }

    pub fn clock_sweep(&mut self) {
        if !self.has_sweep || !self.sweep_running {
            return;
        }
        self.sweep_timer = self.sweep_timer.wrapping_sub(1);
        if self.sweep_timer > 0 {
            return;
        }
        self.sweep_timer = if self.sweep_period == 0 { 8 } else { self.sweep_period };
        if self.sweep_period == 0 {
            return;
        }
        self.compute_sweep(true);
    }

    /// Overflowing 11 bits disables the channel outright, and hardware checks again with the new frequency.
    fn compute_sweep(&mut self, apply: bool) {
        let delta = self.sweep_shadow >> self.sweep_shift;
        let next = if self.sweep_negate { self.sweep_shadow - delta } else { self.sweep_shadow + delta };
        if next > 2047 {
            self.enabled = false;
            return;
        }
        if !apply || self.sweep_shift == 0 {
            return;
        }
        self.sweep_shadow = next;
        self.frequency = next;
        let shifted = self.sweep_shadow >> self.sweep_shift;
        let recheck = if self.sweep_negate { self.sweep_shadow - shifted } else { self.sweep_shadow + shifted };
        if recheck > 2047 {
            self.enabled = false;
        }
    }
}

impl WaveChannel {
    #[inline(always)]
    pub fn output(&self) -> i32 {
        if !self.enabled || !self.dac_enabled || self.volume_code == 0 {
            return 0;
        }
        let byte = self.ram[(self.position >> 1) as usize];
        let sample = if self.position & 1 == 0 { byte >> 4 } else { byte & 0x0F } as i32;
        sample >> (self.volume_code - 1)
    }

    #[inline(always)]
    pub fn step_timer(&mut self) {
        self.timer = self.timer.wrapping_sub(1);
        if self.timer > 0 {
            return;
        }
        self.timer = (2048 - self.frequency) * 2;
        self.position = (self.position + 1) & 0x1F;
    }

    pub fn trigger(&mut self) {
        self.enabled = self.dac_enabled;
        self.timer = (2048 - self.frequency) * 2;
        self.position = 0;
        self.length.trigger();
    }
}

impl NoiseChannel {
    #[inline(always)]
    pub fn output(&self) -> i32 {
        if self.enabled && self.dac_enabled && self.lfsr & 0x01 == 0 { self.envelope.volume } else { 0 }
    }

    #[inline(always)]
    pub fn step_timer(&mut self) {
        self.timer = self.timer.wrapping_sub(1);
        if self.timer > 0 {
            return;
        }
        self.timer = DIVISORS[self.divisor_code as usize] << self.clock_shift;
        self.step_lfsr();
    }

    pub fn trigger(&mut self) {
        self.enabled = self.dac_enabled;
        self.timer = DIVISORS[self.divisor_code as usize] << self.clock_shift;
        self.lfsr = 0x7FFF;
        self.envelope.trigger();
        self.length.trigger();
    }

    /// Short mode feeds the same bit into position 6 as well, shortening the period to 127.
    fn step_lfsr(&mut self) {
        let feedback = (self.lfsr ^ (self.lfsr >> 1)) & 0x01;
        self.lfsr = (self.lfsr >> 1) | (feedback << 14);
        if self.short_mode {
            self.lfsr = (self.lfsr & !0x40) | (feedback << 6);
        }
    }
}

impl State for Envelope {
    fn write_state(&self, w: &mut StateWriter) {
        w.bool("Finished", self.finished);
        w.bool("Increasing", self.increasing);
        w.i32("InitialVolume", self.initial_volume);
        w.i32("Period", self.period);
        w.i32("Timer", self.timer);
        w.i32("Volume", self.volume);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        self.finished = r.bool()?; // Finished
        self.increasing = r.bool()?; // Increasing
        self.initial_volume = r.i32()?; // InitialVolume
        self.period = r.i32()?; // Period
        self.timer = r.i32()?; // Timer
        self.volume = r.i32()?; // Volume
        Ok(())
    }
}

impl State for LengthCounter {
    fn write_state(&self, w: &mut StateWriter) {
        w.i32("Counter", self.counter);
        w.bool("Enabled", self.enabled);
        w.i32("Maximum", self.maximum);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        self.counter = r.i32()?; // Counter
        self.enabled = r.bool()?; // Enabled
        self.maximum = r.i32()?; // Maximum
        Ok(())
    }
}

impl State for PulseChannel {
    fn write_state(&self, w: &mut StateWriter) {
        w.bool("DacEnabled", self.dac_enabled);
        w.i32("Duty", self.duty);
        w.bool("Enabled", self.enabled);
        w.class("Envelope", &self.envelope);
        w.i32("Frequency", self.frequency);
        w.bool("HasSweep", self.has_sweep);
        w.class("Length", &self.length);
        w.i32("Step", self.step);
        w.bool("SweepNegate", self.sweep_negate);
        w.i32("SweepPeriod", self.sweep_period);
        w.bool("SweepRunning", self.sweep_running);
        w.i32("SweepShadow", self.sweep_shadow);
        w.i32("SweepShift", self.sweep_shift);
        w.i32("SweepTimer", self.sweep_timer);
        w.i32("Timer", self.timer);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        self.dac_enabled = r.bool()?; // DacEnabled
        self.duty = r.i32()?; // Duty
        self.enabled = r.bool()?; // Enabled
        r.class(&mut self.envelope)?; // Envelope
        self.frequency = r.i32()?; // Frequency
        self.has_sweep = r.bool()?; // HasSweep
        r.class(&mut self.length)?; // Length
        self.step = r.i32()?; // Step
        self.sweep_negate = r.bool()?; // SweepNegate
        self.sweep_period = r.i32()?; // SweepPeriod
        self.sweep_running = r.bool()?; // SweepRunning
        self.sweep_shadow = r.i32()?; // SweepShadow
        self.sweep_shift = r.i32()?; // SweepShift
        self.sweep_timer = r.i32()?; // SweepTimer
        self.timer = r.i32()?; // Timer
        Ok(())
    }
}

impl State for WaveChannel {
    fn write_state(&self, w: &mut StateWriter) {
        w.bool("DacEnabled", self.dac_enabled);
        w.bool("Enabled", self.enabled);
        w.i32("Frequency", self.frequency);
        w.class("Length", &self.length);
        w.i32("Position", self.position);
        w.bytes("Ram", &self.ram);
        w.i32("Timer", self.timer);
        w.i32("VolumeCode", self.volume_code);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        self.dac_enabled = r.bool()?; // DacEnabled
        self.enabled = r.bool()?; // Enabled
        self.frequency = r.i32()?; // Frequency
        r.class(&mut self.length)?; // Length
        self.position = r.i32()?; // Position
        r.bytes(&mut self.ram)?; // Ram
        self.timer = r.i32()?; // Timer
        self.volume_code = r.i32()?; // VolumeCode
        Ok(())
    }
}

impl State for NoiseChannel {
    fn write_state(&self, w: &mut StateWriter) {
        w.i32("ClockShift", self.clock_shift);
        w.bool("DacEnabled", self.dac_enabled);
        w.i32("DivisorCode", self.divisor_code);
        w.bool("Enabled", self.enabled);
        w.class("Envelope", &self.envelope);
        w.class("Length", &self.length);
        w.i32("Lfsr", self.lfsr);
        w.bool("ShortMode", self.short_mode);
        w.i32("Timer", self.timer);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        self.clock_shift = r.i32()?; // ClockShift
        self.dac_enabled = r.bool()?; // DacEnabled
        self.divisor_code = r.i32()?; // DivisorCode
        self.enabled = r.bool()?; // Enabled
        r.class(&mut self.envelope)?; // Envelope
        r.class(&mut self.length)?; // Length
        self.lfsr = r.i32()?; // Lfsr
        self.short_mode = r.bool()?; // ShortMode
        self.timer = r.i32()?; // Timer
        Ok(())
    }
}
