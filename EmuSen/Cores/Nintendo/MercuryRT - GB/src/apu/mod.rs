//! C#'s `Apu/Apu.cs`: four channels into a two-channel mixer. See Mercury_Apu.md.

pub mod channels;

use crate::Skip;
use crate::apu::channels::{NoiseChannel, PulseChannel, WaveChannel};
use crate::state::{State, StateReader, StateResult, StateWriter};

pub const CHANNEL_COUNT: usize = 4;
pub const REGISTER_BASE: u16 = 0xFF10;
pub const REGISTER_COUNT: usize = 0x17;

/// The mixer's host-side state: none of it is in a save state, so a load keeps what the instance had (Mercury_Native.md §3.1).
#[derive(Clone, Debug, PartialEq)]
pub struct Mixer {
    pub cycles_per_sample: f64,
    pub cycle_fraction: f64,
    pub left_capacitor: f64,
    pub right_capacitor: f64,
    pub charge_factor: f64,
    pub channel_muted: [bool; CHANNEL_COUNT],
    pub buffer: std::collections::VecDeque<i16>,
    pub max_buffered_samples: usize,
}

impl Default for Mixer {
    fn default() -> Self {
        Mixer {
            cycles_per_sample: 4_194_304.0 / 44_100.0,
            cycle_fraction: 0.0,
            left_capacitor: 0.0,
            right_capacitor: 0.0,
            charge_factor: 0.0,
            channel_muted: [false; CHANNEL_COUNT],
            buffer: std::collections::VecDeque::new(),
            max_buffered_samples: 128_000,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Apu {
    pub last_sequencer_bit: bool,
    pub left_volume: i32,
    pub noise: NoiseChannel,
    pub panning: u8,
    pub powered_on: bool,
    pub pulse1: PulseChannel,
    pub pulse2: PulseChannel,
    pub registers: [u8; REGISTER_COUNT],
    pub right_volume: i32,
    pub sequencer_step: i32,
    pub wave: WaveChannel,
    pub mixer: Skip<Mixer>,
}

impl Default for Apu {
    fn default() -> Self {
        Apu {
            last_sequencer_bit: false,
            left_volume: 0,
            noise: NoiseChannel::default(),
            panning: 0,
            powered_on: false,
            pulse1: PulseChannel::new(true),
            pulse2: PulseChannel::new(false),
            registers: [0; REGISTER_COUNT],
            right_volume: 0,
            sequencer_step: 0,
            wave: WaveChannel::default(),
            mixer: Skip(Mixer::default()),
        }
    }
}

impl Apu {
    /// What the DMG boot ROM leaves behind: powered up, both volumes at 7, everything panned to both.
    pub fn reset(&mut self) {
        self.registers = [0; REGISTER_COUNT];
        self.wave.ram = [0; 16];
        self.power_off();
        self.powered_on = true;
        self.left_volume = 7;
        self.right_volume = 7;
        self.panning = 0xF3;
        self.registers[0x24 - 0x10] = 0x77;
        self.registers[0x25 - 0x10] = 0xF3;
        self.registers[0x26 - 0x10] = 0xF1;
        self.sequencer_step = 0;
        self.last_sequencer_bit = false;
        self.mixer.cycle_fraction = 0.0;
        self.mixer.left_capacitor = 0.0;
        self.mixer.right_capacitor = 0.0;
        self.mixer.buffer.clear();
    }

    /// Wave RAM survives a power cycle; nothing else does - see Mercury_Apu.md §4.
    pub fn power_off(&mut self) {
        self.powered_on = false;
        self.registers = [0; REGISTER_COUNT];
        self.left_volume = 0;
        self.right_volume = 0;
        self.panning = 0;
        for pulse in [&mut self.pulse1, &mut self.pulse2] {
            pulse.enabled = false;
            pulse.dac_enabled = false;
            pulse.frequency = 0;
            pulse.duty = 0;
            pulse.length.enabled = false;
            pulse.sweep_running = false;
        }
        self.wave.enabled = false;
        self.wave.dac_enabled = false;
        self.wave.frequency = 0;
        self.wave.volume_code = 0;
        self.wave.length.enabled = false;
        self.noise.enabled = false;
        self.noise.dac_enabled = false;
        self.noise.length.enabled = false;
        self.noise.clock_shift = 0;
        self.noise.divisor_code = 0;
        self.noise.short_mode = false;
    }
}

const WAVE_RAM_BASE: u16 = 0xFF30;
/// What a real console's output capacitor does to the DAC's standing offset - see Mercury_Apu.md §5.2.
pub const HIGH_PASS_SEED: f64 = 0.999958;

impl Apu {
    /// C#'s `SetSampleRate`, integer division included (Mercury_Native.md §6.1, D1); the charge factor comes from C#'s `Math.Pow`.
    pub fn set_sample_rate(&mut self, cycles_per_sample: f64, charge_factor: f64) {
        self.mixer.cycles_per_sample = cycles_per_sample;
        self.mixer.charge_factor = charge_factor;
    }

    /// One T-cycle at the base clock; double speed does not reach here - see Mercury_Apu.md §2.1.
    #[inline(always)]
    pub fn tick(&mut self) {
        if self.powered_on {
            self.pulse1.step_timer();
            self.pulse2.step_timer();
            self.wave.step_timer();
            self.noise.step_timer();
        }
        self.mixer.cycle_fraction += 1.0;
        if self.mixer.cycle_fraction < self.mixer.cycles_per_sample {
            return;
        }
        self.mixer.cycle_fraction -= self.mixer.cycles_per_sample;
        self.emit_sample();
    }

    /// Driven by a falling edge of a DIV bit rather than its own divider - see Mercury_Apu.md §2.
    #[inline(always)]
    pub fn on_div_bit(&mut self, bit: bool) {
        if self.last_sequencer_bit && !bit {
            self.step_sequencer();
        }
        self.last_sequencer_bit = bit;
    }

    fn step_sequencer(&mut self) {
        if !self.powered_on {
            return;
        }
        match self.sequencer_step {
            0 | 4 => self.clock_lengths(),
            2 | 6 => {
                self.clock_lengths();
                self.pulse1.clock_sweep();
            }
            7 => {
                self.pulse1.envelope.clock();
                self.pulse2.envelope.clock();
                self.noise.envelope.clock();
            }
            _ => {}
        }
        self.sequencer_step = (self.sequencer_step + 1) & 0x07;
    }

    fn clock_lengths(&mut self) {
        if self.pulse1.length.clock() {
            self.pulse1.enabled = false;
        }
        if self.pulse2.length.clock() {
            self.pulse2.enabled = false;
        }
        if self.wave.length.clock() {
            self.wave.enabled = false;
        }
        if self.noise.length.clock() {
            self.noise.enabled = false;
        }
    }

    pub fn read_register(&self, address: u16) -> u8 {
        if address >= WAVE_RAM_BASE {
            return self.wave.ram[(address - WAVE_RAM_BASE) as usize];
        }
        let index = address.wrapping_sub(REGISTER_BASE) as usize;
        if index >= REGISTER_COUNT {
            return 0xFF;
        }
        if address == 0xFF26 {
            return (if self.powered_on { 0x80 } else { 0 })
                | self.pulse1.enabled as u8
                | (self.pulse2.enabled as u8) << 1
                | (self.wave.enabled as u8) << 2
                | (self.noise.enabled as u8) << 3
                | 0x70;
        }
        self.registers[index] | read_mask(address)
    }

    pub fn write_register(&mut self, address: u16, value: u8) {
        if address >= WAVE_RAM_BASE {
            self.wave.ram[(address - WAVE_RAM_BASE) as usize] = value;
            return;
        }
        let index = address.wrapping_sub(REGISTER_BASE) as usize;
        if index >= REGISTER_COUNT {
            return;
        }
        if !self.powered_on && address != 0xFF26 {
            return;
        }
        self.registers[index] = value;
        match address {
            0xFF10 => {
                self.pulse1.sweep_period = ((value >> 4) & 0x07) as i32;
                self.pulse1.sweep_negate = value & 0x08 != 0;
                self.pulse1.sweep_shift = (value & 0x07) as i32;
            }
            0xFF11 => write_duty_length(&mut self.pulse1, value),
            0xFF12 => write_envelope(&mut self.pulse1, value),
            0xFF13 => self.pulse1.frequency = (self.pulse1.frequency & 0x0700) | value as i32,
            0xFF14 => write_pulse_control(&mut self.pulse1, value),
            0xFF16 => write_duty_length(&mut self.pulse2, value),
            0xFF17 => write_envelope(&mut self.pulse2, value),
            0xFF18 => self.pulse2.frequency = (self.pulse2.frequency & 0x0700) | value as i32,
            0xFF19 => write_pulse_control(&mut self.pulse2, value),
            0xFF1A => {
                self.wave.dac_enabled = value & 0x80 != 0;
                if !self.wave.dac_enabled {
                    self.wave.enabled = false;
                }
            }
            0xFF1B => self.wave.length.load(value as i32),
            0xFF1C => self.wave.volume_code = ((value >> 5) & 0x03) as i32,
            0xFF1D => self.wave.frequency = (self.wave.frequency & 0x0700) | value as i32,
            0xFF1E => {
                self.wave.frequency = (self.wave.frequency & 0x00FF) | (((value & 0x07) as i32) << 8);
                self.wave.length.enabled = value & 0x40 != 0;
                if value & 0x80 != 0 {
                    self.wave.trigger();
                }
            }
            0xFF20 => self.noise.length.load((value & 0x3F) as i32),
            0xFF21 => {
                self.noise.envelope.load(value);
                self.noise.dac_enabled = value & 0xF8 != 0;
                if !self.noise.dac_enabled {
                    self.noise.enabled = false;
                }
            }
            0xFF22 => {
                self.noise.clock_shift = (value >> 4) as i32;
                self.noise.short_mode = value & 0x08 != 0;
                self.noise.divisor_code = (value & 0x07) as i32;
            }
            0xFF23 => {
                self.noise.length.enabled = value & 0x40 != 0;
                if value & 0x80 != 0 {
                    self.noise.trigger();
                }
            }
            0xFF24 => {
                self.left_volume = ((value >> 4) & 0x07) as i32;
                self.right_volume = (value & 0x07) as i32;
            }
            0xFF25 => self.panning = value,
            0xFF26 => {
                let on = value & 0x80 != 0;
                if on == self.powered_on {
                    return;
                }
                if on {
                    self.powered_on = true;
                    self.sequencer_step = 0;
                } else {
                    self.power_off();
                }
            }
            _ => {}
        }
    }

    fn emit_sample(&mut self) {
        let (mut left, mut right) = (0.0f64, 0.0f64);
        for channel in 0..CHANNEL_COUNT {
            let value = self.dac_output(channel);
            if self.panning & (0x10 << channel) != 0 {
                left += value;
            }
            if self.panning & (0x01 << channel) != 0 {
                right += value;
            }
        }
        left = left / CHANNEL_COUNT as f64 * ((self.left_volume + 1) as f64 / 8.0);
        right = right / CHANNEL_COUNT as f64 * ((self.right_volume + 1) as f64 / 8.0);

        let m = &mut *self.mixer;
        let l = to_sample(high_pass(left, &mut m.left_capacitor, m.charge_factor));
        let r = to_sample(high_pass(right, &mut m.right_capacitor, m.charge_factor));
        if m.buffer.len() + 2 > m.max_buffered_samples {
            m.buffer.pop_front();
            m.buffer.pop_front();
        }
        m.buffer.push_back(l);
        m.buffer.push_back(r);
    }

    /// A silent channel with a live DAC still sits at the bottom of the swing - see Mercury_Apu.md §5.1.
    #[inline(always)]
    fn dac_output(&self, channel: usize) -> f64 {
        if !self.powered_on || self.mixer.channel_muted[channel] {
            return 0.0;
        }
        let (dac, output) = match channel {
            0 => (self.pulse1.dac_enabled, self.pulse1.output()),
            1 => (self.pulse2.dac_enabled, self.pulse2.output()),
            2 => (self.wave.dac_enabled, self.wave.output()),
            _ => (self.noise.dac_enabled, self.noise.output()),
        };
        if dac { (output as f64 / 7.5) - 1.0 } else { 0.0 }
    }

    /// `Drain`: whole stereo pairs, oldest first, at most `max_frames` of them.
    pub fn drain(&mut self, out: &mut [i16], max_frames: usize) -> usize {
        let buffer = &mut self.mixer.buffer;
        let mut wanted = (max_frames.saturating_mul(2)).min(buffer.len()).min(out.len());
        wanted -= wanted & 1;
        for (dst, src) in out.iter_mut().zip(buffer.drain(..wanted)) {
            *dst = src;
        }
        wanted
    }
}

/// Every bit a game cannot read back reads as 1.
fn read_mask(address: u16) -> u8 {
    match address {
        0xFF10 => 0x80,
        0xFF11 | 0xFF16 => 0x3F,
        0xFF13 | 0xFF18 | 0xFF1B | 0xFF1D | 0xFF20 => 0xFF,
        0xFF14 | 0xFF19 | 0xFF1E | 0xFF23 => 0xBF,
        0xFF1A => 0x7F,
        0xFF1C => 0x9F,
        0xFF15 | 0xFF1F => 0xFF,
        _ => 0x00,
    }
}

fn write_duty_length(pulse: &mut PulseChannel, value: u8) {
    pulse.duty = (value >> 6) as i32;
    pulse.length.load((value & 0x3F) as i32);
}

/// Clearing the top five bits of NRx2 kills the DAC, and a dead DAC disables the channel.
fn write_envelope(pulse: &mut PulseChannel, value: u8) {
    pulse.envelope.load(value);
    pulse.dac_enabled = value & 0xF8 != 0;
    if !pulse.dac_enabled {
        pulse.enabled = false;
    }
}

fn write_pulse_control(pulse: &mut PulseChannel, value: u8) {
    pulse.frequency = (pulse.frequency & 0x00FF) | (((value & 0x07) as i32) << 8);
    pulse.length.enabled = value & 0x40 != 0;
    if value & 0x80 != 0 {
        pulse.trigger();
    }
}

#[inline(always)]
fn high_pass(input: f64, capacitor: &mut f64, charge_factor: f64) -> f64 {
    let output = input - *capacitor;
    *capacitor = input - (output * charge_factor);
    output
}

/// `(short)Math.Clamp(value * short.MaxValue, short.MinValue, short.MaxValue)`: clamp, then truncate toward zero.
#[inline(always)]
fn to_sample(value: f64) -> i16 {
    (value * 32767.0).clamp(-32768.0, 32767.0) as i16
}

impl State for Apu {
    fn write_state(&self, w: &mut StateWriter) {
        w.bool("LastSequencerBit", self.last_sequencer_bit);
        w.i32("LeftVolume", self.left_volume);
        w.class("Noise", &self.noise);
        w.u8("Panning", self.panning);
        w.bool("PoweredOn", self.powered_on);
        w.class("Pulse1", &self.pulse1);
        w.class("Pulse2", &self.pulse2);
        w.bytes("Registers", &self.registers);
        w.i32("RightVolume", self.right_volume);
        w.i32("SequencerStep", self.sequencer_step);
        w.class("Wave", &self.wave);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        self.last_sequencer_bit = r.bool()?; // LastSequencerBit
        self.left_volume = r.i32()?; // LeftVolume
        r.class(&mut self.noise)?; // Noise
        self.panning = r.u8()?; // Panning
        self.powered_on = r.bool()?; // PoweredOn
        r.class(&mut self.pulse1)?; // Pulse1
        r.class(&mut self.pulse2)?; // Pulse2
        r.bytes(&mut self.registers)?; // Registers
        self.right_volume = r.i32()?; // RightVolume
        self.sequencer_step = r.i32()?; // SequencerStep
        r.class(&mut self.wave)?; // Wave
        Ok(())
    }
}
