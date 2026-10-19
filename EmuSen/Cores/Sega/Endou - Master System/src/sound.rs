//! The sound (Endou_Native.md §12): Beryl's SN76489, clocked at the Z80's clock over 16, brought to the present at each
//! write and at the frame's end, and resampled to the output rate by emusen-native's step synthesiser, its two sides
//! kept apart for the Game Gear's stereo register. No analogue stage is modelled: no recording of a Master System's
//! output is in the corpus, so the output is the chip's levels, scaled (Endou_Native.md §12.3).

use std::fmt;

use beryl_sn76489::Sn76489;
use emusen_native::steps::Steps;
use emusen_native::{StateReader, StateWriter, Truncated};

/// The output rate.
pub const RATE: u32 = 48_000;

/// Z80 clocks a PSG clock [R SMS Power!'s "SN76489"; `Beryl_SN76489.md` §2].
pub const PSG_DIVIDER: u64 = 16;

/// The output's units for the chip's: a channel at full volume, 8,192 from the chip, is 4,096, so four at full volume
/// fill half of a 16-bit sample (Endou_Native.md §12.3).
pub const GAIN: (i32, i32) = (1, 2);

#[derive(Clone)]
pub struct Sound {
    pub psg: Sn76489,
    /// The PSG's internal clocks run so far.
    pub ticks: u64,
    steps: Steps,
}

impl fmt::Debug for Sound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Sound").field("psg", &self.psg).field("ticks", &self.ticks).finish_non_exhaustive()
    }
}

/// The chip's level in the output's units.
pub fn level(chip: i32) -> i32 {
    chip * GAIN.0 / GAIN.1
}

impl Sound {
    /// The sound of a console whose Z80 runs at `num / den` Hz.
    pub fn new(psg: Sn76489, num: u64, den: u64) -> Sound {
        let mut steps = Steps::new(num, den, RATE);
        let out = psg.output();
        steps.set(0, [level(out[0]), level(out[1])]);
        Sound { psg, ticks: 0, steps }
    }

    /// The PSG up to the last of its clocks at or before the Z80's clock `clock`, each change of its output a step at
    /// the clock it was made.
    pub fn run(&mut self, clock: u64) {
        let now = clock / PSG_DIVIDER;
        while self.ticks < now {
            let n = (now - self.ticks).min(u32::MAX as u64) as u32;
            let (start, steps) = (self.ticks, &mut self.steps);
            self.psg.advance(n, |k, out| steps.set((start + k as u64) * PSG_DIVIDER, [level(out[0]), level(out[1])]));
            self.ticks += n as u64;
        }
    }

    /// A byte at the PSG's port at `clock`: a level it changes is the output from then, after the chip's last clock.
    pub fn write(&mut self, v: u8, clock: u64) {
        self.run(clock);
        self.psg.write(v);
        let out = self.psg.output();
        self.steps.set(clock, [level(out[0]), level(out[1])]);
    }

    /// The Game Gear's stereo register at `clock` (port $06, stage J2).
    pub fn write_stereo(&mut self, v: u8, clock: u64) {
        self.run(clock);
        self.psg.write_stereo(v);
        let out = self.psg.output();
        self.steps.set(clock, [level(out[0]), level(out[1])]);
    }

    /// Every sample finished by `clock`, oldest first.
    pub fn take(&mut self, clock: u64, mut out: impl FnMut(i16, i16)) {
        self.run(clock);
        self.steps.take(clock, |l, r| out(l.clamp(-32768, 32767) as i16, r.clamp(-32768, 32767) as i16));
    }

    /// The samples finished since power-on.
    pub fn emitted(&self) -> u64 {
        self.steps.emitted()
    }

    pub fn write_state(&self, w: &mut StateWriter) {
        self.psg.write_state(w);
        w.u64("PsgTicks", self.ticks);
        self.steps.write_state(w);
    }

    pub fn read_state(&mut self, r: &mut StateReader) -> Result<(), Truncated> {
        self.psg.read_state(r)?;
        self.ticks = r.u64()?;
        self.steps.read_state(r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use beryl_sn76489::{VOLUME, Variant};

    /// The NTSC Z80's clock, the master clock over 15.
    const Z80: (u64, u64) = (53_693_175, 15);

    fn sound(variant: Variant) -> Sound {
        Sound::new(Sn76489::new(variant), Z80.0, Z80.1)
    }

    fn samples(s: &mut Sound, to: u64) -> Vec<[i16; 2]> {
        let mut v = Vec::new();
        s.take(to, |l, r| v.push([l, r]));
        v
    }

    /// The amplitude of the frequency `f` in `x` at the output rate.
    fn amplitude(x: &[f64], f: f64) -> f64 {
        let (mut re, mut im) = (0.0, 0.0);
        for (n, &v) in x.iter().enumerate() {
            let a = 2.0 * std::f64::consts::PI * f * n as f64 / RATE as f64;
            re += v * a.cos();
            im += v * a.sin();
        }
        2.0 * (re * re + im * im).sqrt() / x.len() as f64
    }

    fn centred(v: &[[i16; 2]], side: usize) -> Vec<f64> {
        let mean = v.iter().map(|s| s[side] as f64).sum::<f64>() / v.len() as f64;
        v.iter().map(|s| s[side] as f64 - mean).collect()
    }

    const SECOND: u64 = Z80.0 / Z80.1;

    /// A tone of half-period $0FE is the Z80's clock over 16 × 2 × 254, 440.4 Hz, and its square's fundamental is 4/π
    /// of its half swing.
    #[test]
    fn a_tone_comes_out_at_its_frequency_and_level() {
        let mut s = sound(Variant::SEGA);
        for v in [0x8E, 0x0F, 0x90] {
            s.write(v, 0);
        }
        let out = samples(&mut s, SECOND);
        let x = centred(&out, 0);
        let f = SECOND as f64 / 16.0 / 2.0 / 254.0;
        let at = amplitude(&x, f);
        let want = 4.0 / std::f64::consts::PI * level(VOLUME[0]) as f64 / 2.0;
        assert!((at / want - 1.0).abs() < 0.01, "{at} at {f} Hz, {want} wanted");
        assert!(amplitude(&x, f * 1.05) < at * 0.02);
        assert_eq!(centred(&out, 1), x, "both sides alike off the Game Gear");
    }

    /// Each attenuation step is 2 dB in the output, as in the chip.
    #[test]
    fn the_attenuation_reaches_the_output() {
        let level_at = |volume: u8| {
            let mut s = sound(Variant::SEGA);
            for v in [0x8E, 0x0F, 0x90 | volume] {
                s.write(v, 0);
            }
            let x = centred(&samples(&mut s, SECOND / 2), 0);
            amplitude(&x, SECOND as f64 / 16.0 / 2.0 / 254.0)
        };
        let full = level_at(0);
        for volume in [1u8, 4, 10] {
            let db = 20.0 * (full / level_at(volume)).log10();
            assert!((db - 2.0 * volume as f64).abs() < 0.05, "volume {volume}: {db} dB");
        }
    }

    /// A write's level is the output from its clock, to the output sample: a held tone's step lands half way at the
    /// write's place in the output's time.
    #[test]
    fn a_write_takes_effect_at_its_clock() {
        let mut s = sound(Variant::SEGA);
        s.write(0x81, 0);
        s.write(0x00, 0);
        let at = SECOND / 10;
        s.write(0x90, at);
        let out = samples(&mut s, SECOND / 5);
        // Output sample n is the source at n - 16 samples; the step is at 4,800 samples.
        let mid = 4_800 + emusen_native::steps::HALF;
        assert!(out[mid - 20][0].abs() <= 1 && out[mid + 20][0] == level(VOLUME[0]) as i16);
        assert!((out[mid][0] as i32 - level(VOLUME[0]) / 2).abs() < level(VOLUME[0]) / 20, "{:?}", &out[mid - 2..mid + 3]);
    }

    /// The Game Gear's stereo register sends each channel to its sides, and the output keeps them apart (stage J2).
    #[test]
    fn the_two_sides_are_kept_apart_for_the_game_gear() {
        let mut s = sound(Variant::SEGA);
        for v in [0x81, 0x00, 0x90, 0xA1, 0x00, 0xB6] {
            s.write(v, 0);
        }
        s.write_stereo(0x21, 0);
        let last = *samples(&mut s, SECOND / 10).last().unwrap();
        assert_eq!(last, [level(VOLUME[6]) as i16, level(VOLUME[0]) as i16]);
    }

    /// A second's clocks give the output rate's samples, to the one the synthesiser holds back.
    #[test]
    fn a_seconds_clocks_make_a_seconds_samples() {
        let mut s = sound(Variant::SEGA);
        let n = samples(&mut s, SECOND).len() as i64;
        assert!((n - RATE as i64).abs() <= 1, "{n}");
    }

    /// Four channels at full volume and the noise's every change stay within a 16-bit sample.
    #[test]
    fn four_channels_at_full_volume_stay_within_a_sample() {
        let mut s = sound(Variant::SEGA);
        for v in [0x81, 0x00, 0x90, 0xA1, 0x00, 0xB0, 0xC1, 0x00, 0xD0, 0xE4, 0xF0] {
            s.write(v, 0);
        }
        let out = samples(&mut s, SECOND / 10);
        assert!(out.iter().all(|p| p[0] > -2_000), "{:?}", out.iter().map(|p| p[0]).min());
        let top = out.iter().map(|p| p[0]).max().unwrap();
        assert!(top >= (4 * level(VOLUME[0]) - 1) as i16, "{top}");
    }

    /// A state taken mid-note continues as the sound that saved it.
    #[test]
    fn a_state_continues_the_sound() {
        let mut a = sound(Variant::DISCRETE);
        for v in [0x8E, 0x0F, 0x90, 0xE5, 0xF2] {
            a.write(v, 100);
        }
        samples(&mut a, 77_777);
        let mut bytes = vec![0u8; 4096];
        let mut w = StateWriter::new(&mut bytes);
        a.write_state(&mut w);
        let n = w.len();
        let mut b = sound(Variant::DISCRETE);
        b.read_state(&mut StateReader::new(&bytes[..n])).unwrap();
        a.write(0x94, 90_000);
        b.write(0x94, 90_000);
        assert_eq!(samples(&mut a, 200_000), samples(&mut b, 200_000));
        assert_eq!((a.psg, a.ticks), (b.psg, b.ticks));
    }
}
