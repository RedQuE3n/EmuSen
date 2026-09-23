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
