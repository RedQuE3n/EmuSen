//! C#'s `Apu`: the frame sequencer, the register file, the non-linear DAC and the output filters. See Moon_APU.md.

pub mod channels;

use std::collections::VecDeque;

use crate::Skip;
use crate::state::{StateReader, StateResult, StateWriter};
use channels::{DmcChannel, LENGTH_COUNTER, NoiseChannel, PulseChannel, TriangleChannel};

pub const CHANNEL_COUNT: usize = 5;
pub const CPU_CLOCK_HZ: f64 = 1789773.0;
const OUTPUT_GAIN: f64 = 80000.0;
pub const MAX_BUFFERED_SAMPLES: usize = 128000;

const FRAME_NONE: i32 = 0;
const FRAME_QUARTER: i32 = 1;
const FRAME_HALF: i32 = 2;
const STEP_COUNT: i32 = 6;
static FOUR_STEP_CYCLES: [i32; 6] = [7457, 14913, 22371, 29828, 29829, 29830];
static FIVE_STEP_CYCLES: [i32; 6] = [7457, 14913, 22371, 29829, 37281, 37282];
static FRAME_TYPES: [i32; 6] = [FRAME_QUARTER, FRAME_HALF, FRAME_QUARTER, FRAME_NONE, FRAME_HALF, FRAME_NONE];

/// The mixer's state, none of it in a save state; a C# load keeps the instance's own (Moon_Native.md §3.1).
#[derive(Clone, Debug, PartialEq)]
pub struct Mixer {
    pub sample_accumulator: f64,
    pub sample_count: i32,
    pub cycles_per_sample: f64,
    pub cycle_fraction: f64,
    pub buffer: VecDeque<i16>,
    /// The host's `AudioSettings.AudioBufferMaxSamples`, sent through `moon_machine_set_audio_limit`.
    pub max_buffered_samples: usize,
    pub hp90: f64,
    pub hp90_prev: f64,
    pub hp440: f64,
    pub hp440_prev: f64,
    pub lp14k: f64,
    pub high_pass90_alpha: f64,
    pub high_pass440_alpha: f64,
    pub low_pass_alpha: f64,
    pub channel_muted: [bool; CHANNEL_COUNT],
}

impl Default for Mixer {
    fn default() -> Self {
        Mixer {
            sample_accumulator: 0.0,
            sample_count: 0,
            cycles_per_sample: CPU_CLOCK_HZ / 44100.0,
            cycle_fraction: 0.0,
            buffer: VecDeque::with_capacity(MAX_BUFFERED_SAMPLES),
            max_buffered_samples: MAX_BUFFERED_SAMPLES,
            hp90: 0.0,
            hp90_prev: 0.0,
            hp440: 0.0,
            hp440_prev: 0.0,
            lp14k: 0.0,
            high_pass90_alpha: 1.0,
            high_pass440_alpha: 1.0,
            low_pass_alpha: 0.0,
            channel_muted: [false; CHANNEL_COUNT],
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Apu {
    pub dmc: DmcChannel,
    pub frame_counter: u8,
    pub frame_irq_pending: bool,
    pub noise: NoiseChannel,
    pub pulse1: PulseChannel,
    pub pulse2: PulseChannel,
    pub registers: [u8; 0x14],
    pub triangle: TriangleChannel,
    pub apu_cycle: bool,
    pub block_frame_counter_tick: i32,
    pub cycle_count: i64,
    pub frame_cycle: i32,
    pub frame_step: i32,
    pub pending_frame_value: i32,
    pub step_mode: bool,
    pub write_delay_counter: i32,

    pub mixer: Skip<Mixer>,
}

impl Default for Apu {
    fn default() -> Self {
        Apu {
            dmc: DmcChannel::default(),
            frame_counter: 0,
            frame_irq_pending: false,
            noise: NoiseChannel::default(),
            pulse1: PulseChannel::new(true),
            pulse2: PulseChannel::new(false),
            registers: [0; 0x14],
            triangle: TriangleChannel::default(),
            apu_cycle: false,
            block_frame_counter_tick: 0,
            cycle_count: 0,
            frame_cycle: 0,
            frame_step: 0,
            pending_frame_value: -1,
            step_mode: false,
            write_delay_counter: 0,
            mixer: Skip(Mixer::default()),
        }
    }
}

fn high_pass_alpha(hz: f64, dt: f64) -> f64 {
    let rc = 1.0 / (2.0 * std::f64::consts::PI * hz);
    rc / (rc + dt)
}

fn low_pass_alpha(hz: f64, dt: f64) -> f64 {
    let rc = 1.0 / (2.0 * std::f64::consts::PI * hz);
    dt / (rc + dt)
}

impl Apu {
    #[inline(always)]
    pub fn irq_inhibited(&self) -> bool {
        (self.frame_counter & 0x40) != 0
    }

    /// `IrqAsserted`: everything the CPU sees as one IRQ line.
    #[inline(always)]
    pub fn irq_asserted(&self) -> bool {
        self.frame_irq_pending || self.dmc.irq_pending
    }

    /// `SetSampleRate`: every coefficient is `+ - * /` and pi, so both runtimes compute the same bits (Moon_Native.md §3.3).
    pub fn set_sample_rate(&mut self, sample_rate: i32) {
        let rate = sample_rate.max(1);
        let m = &mut *self.mixer;
        m.cycles_per_sample = CPU_CLOCK_HZ / rate as f64;
        let dt = 1.0 / rate as f64;
        m.high_pass90_alpha = high_pass_alpha(90.0, dt);
        m.high_pass440_alpha = high_pass_alpha(440.0, dt);
        m.low_pass_alpha = low_pass_alpha(14000.0, dt);
    }

    pub fn reset(&mut self) {
        self.registers.fill(0);
        self.frame_counter = 0;
        self.step_mode = false;
        self.cycle_count = 0;
        self.soft_reset();
        self.dmc.reset();
        self.mixer.buffer.clear();
    }

    /// RESET silences every channel and rewrites `$4017` with the mode it already had - see Moon_APU.md §2.2.
    pub fn soft_reset(&mut self) {
        self.write_enable(0);
        self.frame_irq_pending = false;
        self.frame_cycle = 0;
        self.frame_step = 0;
        self.apu_cycle = false;
        self.block_frame_counter_tick = 0;
        self.pending_frame_value = if self.step_mode { 0x80 } else { 0x00 };
        self.write_delay_counter = 3;
    }

    pub fn write_register(&mut self, address: u16, value: u8) {
        if (0x4000..=0x4013).contains(&address) {
            self.registers[(address - 0x4000) as usize] = value;
        }
        let v = value as i32;
        match address {
            0x4000 => write_pulse_control(&mut self.pulse1, value),
            0x4001 => write_sweep(&mut self.pulse1, value),
            0x4002 => self.pulse1.timer_period = (self.pulse1.timer_period & 0x700) | v,
            0x4003 => write_pulse_high(&mut self.pulse1, value),
            0x4004 => write_pulse_control(&mut self.pulse2, value),
            0x4005 => write_sweep(&mut self.pulse2, value),
            0x4006 => self.pulse2.timer_period = (self.pulse2.timer_period & 0x700) | v,
            0x4007 => write_pulse_high(&mut self.pulse2, value),
            0x4008 => {
                self.triangle.control_flag = (v & 0x80) != 0;
                self.triangle.linear_reload_value = v & 0x7F;
            }
            0x400A => self.triangle.timer_period = (self.triangle.timer_period & 0x700) | v,
            0x400B => {
                self.triangle.timer_period = (self.triangle.timer_period & 0xFF) | ((v & 0x07) << 8);
                if self.triangle.enabled {
                    self.triangle.length_counter = LENGTH_COUNTER[(value >> 3) as usize] as i32;
                }
                self.triangle.linear_reload = true;
            }
            0x400C => {
                self.noise.length_halted = (v & 0x20) != 0;
                self.noise.envelope.r#loop = self.noise.length_halted;
                self.noise.envelope.constant_volume = (v & 0x10) != 0;
                self.noise.envelope.volume = v & 0x0F;
            }
            0x400E => {
                self.noise.short_mode = (v & 0x80) != 0;
                self.noise.period_index = v & 0x0F;
            }
            0x400F => {
                if self.noise.enabled {
                    self.noise.length_counter = LENGTH_COUNTER[(value >> 3) as usize] as i32;
                }
                self.noise.envelope.restart();
            }
            0x4010 => {
                self.dmc.irq_enabled = (v & 0x80) != 0;
                self.dmc.r#loop = (v & 0x40) != 0;
                self.dmc.rate_index = v & 0x0F;
                if !self.dmc.irq_enabled {
                    self.dmc.irq_pending = false;
                }
            }
            0x4011 => self.dmc.output_level = v & 0x7F,
            0x4012 => self.dmc.sample_address = 0xC000 + v * 64,
            0x4013 => self.dmc.sample_length = v * 16 + 1,
            0x4015 => self.write_enable(value),
            0x4017 => {
                self.frame_counter = value;
                if self.irq_inhibited() {
                    self.frame_irq_pending = false;
                }
                self.pending_frame_value = v;
                self.write_delay_counter = if (self.cycle_count & 1) != 0 { 4 } else { 3 };
            }
            _ => {}
        }
    }

    fn write_enable(&mut self, value: u8) {
        self.pulse1.set_enabled((value & 0x01) != 0);
        self.pulse2.set_enabled((value & 0x02) != 0);
        self.triangle.set_enabled((value & 0x04) != 0);
        self.noise.set_enabled((value & 0x08) != 0);
        let get = self.is_get_cycle();
        self.dmc.set_enabled((value & 0x10) != 0, get);
    }

    /// The cycle now being run is a get cycle, the parity the frame counter's $4017 delay uses.
    #[inline(always)]
    pub fn is_get_cycle(&self) -> bool {
        (self.cycle_count & 1) != 0
    }

    #[inline(always)]
    pub fn next_cycle_is_get(&self) -> bool {
        (self.cycle_count & 1) == 0
    }

    /// `ReadStatus`: reading acknowledges the frame IRQ.
    pub fn read_status(&mut self) -> u8 {
        let mut value = 0u8;
        if self.pulse1.length_counter > 0 {
            value |= 0x01;
        }
        if self.pulse2.length_counter > 0 {
            value |= 0x02;
        }
        if self.triangle.length_counter > 0 {
            value |= 0x04;
        }
        if self.noise.length_counter > 0 {
            value |= 0x08;
        }
        if self.dmc.active() {
            value |= 0x10;
        }
        if self.frame_irq_pending {
            value |= 0x40;
        }
        if self.dmc.irq_pending {
            value |= 0x80;
        }
        self.frame_irq_pending = false;
        value
    }

    /// `Step`: every timer by one CPU cycle, `cpu_cycles` times; the DMC's byte is read through `read`, and `$4015` by the APU itself.
    #[inline(always)]
    pub fn step(&mut self, cpu_cycles: i32) {
        for _ in 0..cpu_cycles {
            self.cycle_count = self.cycle_count.wrapping_add(1);
            self.triangle.step_timer();
            self.dmc.step_timer();

            self.apu_cycle = !self.apu_cycle;
            if self.apu_cycle {
                self.pulse1.step_timer();
                self.pulse2.step_timer();
                self.noise.step_timer();
            }

            self.step_frame_counter();

            let mix = self.mix();
            let m = &mut *self.mixer;
            m.sample_accumulator += mix;
            m.sample_count = m.sample_count.wrapping_add(1);
            m.cycle_fraction += 1.0;
            if m.cycle_fraction < m.cycles_per_sample {
                continue;
            }
            m.cycle_fraction -= m.cycles_per_sample;
            m.emit_sample();
        }
    }

    /// The two halves of the DAC, non-linear and summed separately - see Moon_APU.md §4.
    #[inline(always)]
    fn mix(&self) -> f64 {
        let muted = &self.mixer.channel_muted;
        let pulse_sum = (if muted[0] { 0 } else { self.pulse1.output() }) + (if muted[1] { 0 } else { self.pulse2.output() });
        let pulse_out = if pulse_sum == 0 { 0.0 } else { 95.88 / ((8128.0 / pulse_sum as f64) + 100.0) };
        let tnd = ((if muted[2] { 0 } else { self.triangle.output() }) as f64 / 8227.0)
            + ((if muted[3] { 0 } else { self.noise.output() }) as f64 / 12241.0)
            + ((if muted[4] { 0 } else { self.dmc.output_level }) as f64 / 22638.0);
        let tnd_out = if tnd == 0.0 { 0.0 } else { 159.79 / ((1.0 / tnd) + 100.0) };
        pulse_out + tnd_out
    }

    /// Modelled on Mesen's `ApuFrameCounter` - see Moon_APU.md §2.1.
    #[inline(always)]
    fn step_frame_counter(&mut self) {
        self.frame_cycle = self.frame_cycle.wrapping_add(1);
        let steps = if self.step_mode { &FIVE_STEP_CYCLES } else { &FOUR_STEP_CYCLES };
        let due = match steps.get(self.frame_step as usize) {
            Some(&d) => d,
            None => {
                crate::fault(crate::Fault::IndexOutOfRange);
                i32::MAX
            }
        };
        if self.frame_cycle >= due {
            if !self.step_mode && self.frame_step >= 3 && !self.irq_inhibited() {
                self.frame_irq_pending = true;
            }
            let kind = FRAME_TYPES[self.frame_step as usize];
            if kind != FRAME_NONE && self.block_frame_counter_tick == 0 {
                self.clock_quarter_frame();
                if kind == FRAME_HALF {
                    self.clock_half_frame();
                }
                self.block_frame_counter_tick = 2;
            }
            self.frame_step += 1;
            if self.frame_step == STEP_COUNT {
                self.frame_step = 0;
                self.frame_cycle = 0;
            }
        }

        if self.pending_frame_value >= 0 {
            self.write_delay_counter = self.write_delay_counter.wrapping_sub(1);
            if self.write_delay_counter == 0 {
                self.step_mode = (self.pending_frame_value & 0x80) != 0;
                self.pending_frame_value = -1;
                self.frame_step = 0;
                self.frame_cycle = 0;
                if self.step_mode && self.block_frame_counter_tick == 0 {
                    self.clock_quarter_frame();
                    self.clock_half_frame();
                    self.block_frame_counter_tick = 2;
                }
            }
        }

        if self.block_frame_counter_tick > 0 {
            self.block_frame_counter_tick -= 1;
        }
    }

    fn clock_quarter_frame(&mut self) {
        self.pulse1.envelope.clock();
        self.pulse2.envelope.clock();
        self.noise.envelope.clock();
        self.triangle.clock_linear();
    }

    fn clock_half_frame(&mut self) {
        self.pulse1.clock_length();
        self.pulse2.clock_length();
        self.triangle.clock_length();
        self.noise.clock_length();
        self.pulse1.clock_sweep();
        self.pulse2.clock_sweep();
    }

    /// `Drain`: whole stereo pairs, oldest first, into `out`; returns the samples written.
    pub fn drain(&mut self, out: &mut [i16], max_frames: usize) -> usize {
        let buffer = &mut self.mixer.buffer;
        let mut wanted = (max_frames.saturating_mul(2)).min(buffer.len()).min(out.len());
        wanted -= wanted & 1;
        for (slot, sample) in out.iter_mut().zip(buffer.drain(..wanted)) {
            *slot = sample;
        }
        wanted
    }

    pub fn write_state(&self, w: &mut StateWriter) {
        w.class("Dmc", &self.dmc);
        w.u8("FrameCounter", self.frame_counter);
        w.bool("FrameIrqPending", self.frame_irq_pending);
        w.class("Noise", &self.noise);
        w.class("Pulse1", &self.pulse1);
        w.class("Pulse2", &self.pulse2);
        w.bytes("Registers", &self.registers);
        w.class("Triangle", &self.triangle);
        w.bool("_apuCycle", self.apu_cycle);
        w.i32("_blockFrameCounterTick", self.block_frame_counter_tick);
        w.i64("_cycleCount", self.cycle_count);
        w.i32("_frameCycle", self.frame_cycle);
        w.i32("_frameStep", self.frame_step);
        w.i32("_pendingFrameValue", self.pending_frame_value);
        w.bool("_stepMode", self.step_mode);
        w.i32("_writeDelayCounter", self.write_delay_counter);
    }

    pub fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        r.class(&mut self.dmc)?; // Dmc
        self.frame_counter = r.u8()?; // FrameCounter
        self.frame_irq_pending = r.bool()?; // FrameIrqPending
        r.class(&mut self.noise)?; // Noise
        r.class(&mut self.pulse1)?; // Pulse1
        r.class(&mut self.pulse2)?; // Pulse2
        r.bytes(&mut self.registers)?; // Registers
        r.class(&mut self.triangle)?; // Triangle
        self.apu_cycle = r.bool()?; // _apuCycle
        self.block_frame_counter_tick = r.i32()?; // _blockFrameCounterTick
        self.cycle_count = r.i64()?; // _cycleCount
        self.frame_cycle = r.i32()?; // _frameCycle
        self.frame_step = r.i32()?; // _frameStep
        self.pending_frame_value = r.i32()?; // _pendingFrameValue
        self.step_mode = r.bool()?; // _stepMode
        self.write_delay_counter = r.i32()?; // _writeDelayCounter
        Ok(())
    }
}

impl Mixer {
    /// Two high-passes then a low-pass, the console's own output stage.
    #[inline(always)]
    fn filter(&mut self, sample: f64) -> f64 {
        self.hp90 = self.high_pass90_alpha * (self.hp90 + sample - self.hp90_prev);
        self.hp90_prev = sample;
        self.hp440 = self.high_pass440_alpha * (self.hp440 + self.hp90 - self.hp440_prev);
        self.hp440_prev = self.hp90;
        self.lp14k += self.low_pass_alpha * (self.hp440 - self.lp14k);
        self.lp14k
    }

    fn emit_sample(&mut self) {
        let mean = if self.sample_count > 0 { self.sample_accumulator / self.sample_count as f64 } else { 0.0 };
        self.sample_accumulator = 0.0;
        self.sample_count = 0;
        let sample = (self.filter(mean) * OUTPUT_GAIN).clamp(i16::MIN as f64, i16::MAX as f64) as i16;
        self.buffer.push_back(sample);
        self.buffer.push_back(sample);
        // C#'s SampleQueue: trimmed after the pair, until within the limit (EmuSen_Settings_Reference.md §4.85.2).
        while self.buffer.len() > self.max_buffered_samples && self.buffer.len() >= 2 {
            self.buffer.pop_front();
            self.buffer.pop_front();
        }
    }
}

fn write_pulse_control(pulse: &mut PulseChannel, value: u8) {
    let v = value as i32;
    pulse.duty = (v >> 6) & 0x03;
    pulse.length_halted = (v & 0x20) != 0;
    pulse.envelope.r#loop = pulse.length_halted;
    pulse.envelope.constant_volume = (v & 0x10) != 0;
    pulse.envelope.volume = v & 0x0F;
}

fn write_sweep(pulse: &mut PulseChannel, value: u8) {
    let v = value as i32;
    pulse.sweep_enabled = (v & 0x80) != 0;
    pulse.sweep_period = (v >> 4) & 0x07;
    pulse.sweep_negate = (v & 0x08) != 0;
    pulse.sweep_shift = v & 0x07;
    pulse.sweep_reload = true;
}

fn write_pulse_high(pulse: &mut PulseChannel, value: u8) {
    pulse.timer_period = (pulse.timer_period & 0xFF) | (((value & 0x07) as i32) << 8);
    if pulse.enabled {
        pulse.length_counter = LENGTH_COUNTER[(value >> 3) as usize] as i32;
    }
    pulse.restart_sequencer();
}
