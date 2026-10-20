//! The sound (Endou_Native.md §12, §16): Beryl's SN76489, clocked at the Z80's clock over 16, and on the consoles that
//! carry one the YM2413, a sample every 72 Z80 clocks, each brought to the present at its writes and at the frame's end;
//! port $F2 chooses which sound; the two are summed and resampled to the output rate by emusen-native's step
//! synthesiser, its two sides kept apart for the Game Gear's stereo register. No analogue stage is modelled: no
//! recording of a Master System's output is in the corpus, so the output is the chips' levels, scaled (§12.3).

use std::fmt;

use beryl_sn76489::Sn76489;
use emusen_native::steps::Steps;
use emusen_native::{StateReader, StateWriter, Truncated};

use crate::model::FmUnit;
use crate::opll::{CLOCKS_PER_SAMPLE, Opll};

/// The output rate.
pub const RATE: u32 = 48_000;

/// Z80 clocks a PSG clock [R SMS Power!'s "SN76489"; `Beryl_SN76489.md` §2].
pub const PSG_DIVIDER: u64 = 16;

/// The output's units for the chip's: a channel at full volume, 8,192 from the chip, is 4,096, so four at full volume
/// fill half of a 16-bit sample (Endou_Native.md §12.3).
pub const GAIN: (i32, i32) = (1, 2);

/// The output's units for the YM2413's DAC sum: a full sine on one channel, 255, is 0.59 of a full-volume PSG tone's
/// fundamental, near the median of the references' (Endou_Native.md §16.6).
pub const FM_GAIN: i32 = 6;

/// The FM unit: the chip, the samples it has made, and port $F2's two bits as written (Endou_Native.md §16.3).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fm {
    pub unit: FmUnit,
    pub chip: Opll,
    pub samples: u64,
    pub control: u8,
    /// The chip's last sample in the output's units, sounding or not.
    pub level: i32,
}

impl Fm {
    pub fn new(unit: FmUnit) -> Fm {
        Fm { unit, chip: Opll::new(), samples: 0, control: 0, level: 0 }
    }

    /// Whether port $F2 sounds the FM, and the PSG [R SMS Power!'s "Audio Control Port"].
    pub fn sounds(&self) -> (bool, bool) {
        let fm = self.control & 1 != 0;
        match self.unit {
            FmUnit::AddOn => (fm, !fm),
            FmUnit::BuiltIn => (fm, self.control & 1 == self.control >> 1 & 1),
        }
    }
}

#[derive(Clone)]
pub struct Sound {
    pub psg: Sn76489,
    /// The PSG's internal clocks run so far.
    pub ticks: u64,
    pub fm: Option<Fm>,
    /// The PSG's output in the output's units.
    psg_level: [i32; 2],
    steps: Steps,
}

/// What sounds at `clock`: the PSG's level and the FM's, each as port $F2 lets it through.
fn emit(steps: &mut Steps, fm: Option<&Fm>, psg_level: [i32; 2], clock: u64) {
    let (fm, psg) = fm.map_or((0, true), |f| {
        let (fm, psg) = f.sounds();
        (if fm { f.level } else { 0 }, psg)
    });
    let p = if psg { psg_level } else { [0; 2] };
    steps.set(clock, [p[0] + fm, p[1] + fm]);
}

impl fmt::Debug for Sound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Sound").field("psg", &self.psg).field("ticks", &self.ticks).field("fm", &self.fm).finish_non_exhaustive()
    }
}

/// The chip's level in the output's units.
pub fn level(chip: i32) -> i32 {
    chip * GAIN.0 / GAIN.1
}

impl Sound {
    /// The sound of a console whose Z80 runs at `num / den` Hz.
    pub fn new(psg: Sn76489, num: u64, den: u64) -> Sound {
        Sound::with_fm(psg, None, num, den)
    }

    /// The same with the console's FM unit, if it has one.
    pub fn with_fm(psg: Sn76489, fm: Option<FmUnit>, num: u64, den: u64) -> Sound {
        let out = psg.output();
        let mut s = Sound { psg, ticks: 0, fm: fm.map(Fm::new), psg_level: [level(out[0]), level(out[1])], steps: Steps::new(num, den, RATE) };
        s.emit(0);
        s
    }

    fn emit(&mut self, clock: u64) {
        emit(&mut self.steps, self.fm.as_ref(), self.psg_level, clock);
    }

    /// The PSG up to the last of its clocks at or before the Z80's clock `clock`, and the FM to its last sample at or
    /// before it, each change a step at the clock it was made, in the order they were made.
    pub fn run(&mut self, clock: u64) {
        loop {
            let next = self.fm.as_ref().map(|f| (f.samples + 1) * CLOCKS_PER_SAMPLE).filter(|&t| t <= clock);
            self.run_psg(next.unwrap_or(clock));
            let Some(t) = next else { return };
            let f = self.fm.as_mut().expect("an FM unit");
            let (melody, rhythm) = f.chip.sample();
            f.samples += 1;
            f.level = (melody + rhythm) * FM_GAIN;
            self.emit(t);
        }
    }

    fn run_psg(&mut self, clock: u64) {
        let now = clock / PSG_DIVIDER;
        while self.ticks < now {
            let n = (now - self.ticks).min(u32::MAX as u64) as u32;
            let (start, steps, fm, psg_level) = (self.ticks, &mut self.steps, self.fm.as_ref(), &mut self.psg_level);
            self.psg.advance(n, |k, out| {
                *psg_level = [level(out[0]), level(out[1])];
                emit(steps, fm, *psg_level, (start + k as u64) * PSG_DIVIDER);
            });
            self.ticks += n as u64;
        }
    }

    /// A byte at the PSG's port at `clock`: a level it changes is the output from then, after the chip's last clock.
    pub fn write(&mut self, v: u8, clock: u64) {
        self.run(clock);
        self.psg.write(v);
        let out = self.psg.output();
        self.psg_level = [level(out[0]), level(out[1])];
        self.emit(clock);
    }

    /// The Game Gear's stereo register at `clock` (port $06, stage J2).
    pub fn write_stereo(&mut self, v: u8, clock: u64) {
        self.run(clock);
        self.psg.write_stereo(v);
        let out = self.psg.output();
        self.psg_level = [level(out[0]), level(out[1])];
        self.emit(clock);
    }

    /// A write to the YM2413's address (`data` false) or data port at `clock`, after its last sample; none without the
    /// unit.
    pub fn write_fm(&mut self, data: bool, v: u8, clock: u64) {
        self.run(clock);
        if let Some(f) = self.fm.as_mut() {
            if data { f.chip.write_data(v) } else { f.chip.write_address(v) }
        }
    }

    /// Port $F2 written at `clock`: the Mark III's unit keeps no bit 1 [R SMS Power!'s "Audio Control Port"].
    pub fn write_control(&mut self, v: u8, clock: u64) {
        self.run(clock);
        if let Some(f) = self.fm.as_mut() {
            f.control = v & if f.unit == FmUnit::AddOn { 1 } else { 3 };
            self.emit(clock);
        }
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

    /// The PSG and its clock, the FM unit where the console has one, then the synthesiser.
    pub fn write_state(&self, w: &mut StateWriter) {
        self.psg.write_state(w);
        w.u64("PsgTicks", self.ticks);
        if let Some(f) = &self.fm {
            f.chip.write_state(w);
            w.u64("FmSamples", f.samples);
            w.u8("AudioControl", f.control);
            w.i32("FmLevel", f.level);
        }
        self.steps.write_state(w);
    }

    /// Reads a state; `false` for an FM unit no chip could be in.
    pub fn read_state(&mut self, r: &mut StateReader) -> Result<bool, Truncated> {
        self.psg.read_state(r)?;
        self.ticks = r.u64()?;
        let out = self.psg.output();
        self.psg_level = [level(out[0]), level(out[1])];
        if let Some(f) = self.fm.as_mut() {
            if !f.chip.read_state(r)? {
                return Ok(false);
            }
            f.samples = r.u64()?;
            f.control = r.u8()? & 3;
            f.level = r.i32()?;
        }
        self.steps.read_state(r)?;
        Ok(true)
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

    /// A write's step falls at its own clock, not at the PSG's last clock before it: a write 15 clocks later rises
    /// later, by a fifth of an output sample (stage 5's seeded fault F2 passed the test above).
    #[test]
    fn a_write_s_step_is_at_its_own_clock_within_the_psg_s() {
        let rise = |at: u64| {
            let mut s = sound(Variant::SEGA);
            s.write(0x81, 0);
            s.write(0x00, 0);
            s.write(0x90, at);
            let out = samples(&mut s, SECOND / 5);
            out[4_790..4_830].iter().map(|p| p[0] as i64).sum::<i64>()
        };
        let base = SECOND / 10 / PSG_DIVIDER * PSG_DIVIDER;
        let (early, late) = (rise(base), rise(base + 15));
        // A step later by 15 / 16 × 48,000 / 3,579,545 of a sample takes that much of its height from the rise's sum.
        let want = 15.0 * RATE as f64 / SECOND as f64 * level(VOLUME[0]) as f64;
        assert!(((early - late) as f64 / want - 1.0).abs() < 0.05, "{early} {late}, {want} apart wanted");
    }

    /// The Game Gear's stereo register sends each channel to its sides, and the output keeps them apart as the chip
    /// runs (stage J2): tone 0 sounding on the right alone, tone 1 held on the left alone.
    #[test]
    fn the_two_sides_are_kept_apart_for_the_game_gear() {
        let mut s = sound(Variant::SEGA);
        for v in [0x8E, 0x0F, 0x90, 0xA1, 0x00, 0xB6] {
            s.write(v, 0);
        }
        s.write_stereo(0x21, 0);
        let out = samples(&mut s, SECOND / 10);
        let tail = &out[100..];
        assert!(tail.iter().all(|p| p[0] == level(VOLUME[6]) as i16), "the left holds tone 1");
        let (low, high) = (tail.iter().map(|p| p[1]).min().unwrap(), tail.iter().map(|p| p[1]).max().unwrap());
        assert!(low <= 0 && high >= level(VOLUME[0]) as i16, "the right sounds tone 0: {low} to {high}");
    }

    /// Port $F2's two bits choose what sounds (SMS Power!'s "Audio Control Port"): on the Japanese Master System the FM
    /// with bit 0, the PSG while the two bits are equal; on the Mark III's unit one or the other (Endou_Native.md §16.3).
    #[test]
    fn port_f2_chooses_what_sounds() {
        let level = |unit: FmUnit, control: u8| {
            let mut s = Sound::with_fm(Sn76489::new(Variant::SEGA), Some(unit), Z80.0, Z80.1);
            for v in [0x8E, 0x0F, 0x90] {
                s.write(v, 0);
            }
            for (a, v) in [(0x00, 0x21), (0x01, 0x21), (0x02, 0x3F), (0x04, 0xF0), (0x05, 0xF0), (0x06, 0x0F), (0x07, 0x0F), (0x10, 0xB0), (0x20, 0x19)] {
                s.write_fm(false, a, 0);
                s.write_fm(true, v, 0);
            }
            s.write_control(control, 0);
            let x = centred(&samples(&mut s, SECOND / 2), 0);
            (amplitude(&x, SECOND as f64 / 16.0 / 2.0 / 254.0) > 1000.0, amplitude(&x, 0x1B0 as f64 * SECOND as f64 / 72.0 * 8.0 / 262_144.0) > 500.0)
        };
        assert_eq!([0, 1, 2, 3].map(|c| level(FmUnit::BuiltIn, c)), [(true, false), (false, true), (false, false), (true, true)]);
        assert_eq!([0, 1, 2, 3].map(|c| level(FmUnit::AddOn, c)), [(true, false), (false, true), (true, false), (false, true)]);
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
