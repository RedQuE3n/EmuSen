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
