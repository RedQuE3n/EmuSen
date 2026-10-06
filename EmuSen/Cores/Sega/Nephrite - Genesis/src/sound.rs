//! The Genesis's sound: the YM2612 (`ym2612.rs`) and the PSG in the VDP (Beryl's SN76489, Sega's variant), each
//! caught up when it is touched and at the frame's end, each resampled to the output rate by its own step synthesiser
//! (emusen-native's `steps`), the two summed and put through the model's output circuit. The mix's levels and the
//! circuits are measured on recordings of consoles (Nephrite_Native.md §28, Nephrite_Disputes.md D-27).

use emusen_native::steps::Steps;
use emusen_native::{StateReader, StateWriter, Truncated};
use beryl_sn76489::{Sn76489, Variant};

use crate::ym2612::Ym2612;

/// Master clocks a PSG clock: the Z80's clock, over 16 inside the chip.
pub const PSG_CLOCK: u64 = 15 * 16;
/// The output rate.
pub const RATE: u32 = 48_000;
/// The output's units for one step of the YM2612's nine-bit output, and the PSG's level against it: a PSG channel at
/// full volume, 8,192 from the chip, is 0.324 of the DAC's whole swing of 512 steps, as seven model 1 consoles have
/// it (Nephrite_Disputes.md D-27).
pub const FM_GAIN: i32 = 16;
pub const PSG_GAIN: (i32, i32) = (81, 250);
/// The YM2612's pins carry each channel for a part of its sixth of the sample, not a held level, so the band has its
/// samples at full strength where a held sample droops by 2.1 dB at 20 kHz. The synthesiser holds each sample, which
/// keeps its images down; these seven taps at the output rate, the centre's and three each side in 24-bit fractions,
/// undo the hold's droop to 0.08 dB up to 20 kHz (D-27).
pub const UNHOLD: [i64; 4] = [18_804_106, -1_231_811, 289_733, -71_367];

/// A model's output circuit as a filter at the output rate, its coefficients in 24-bit fractions: two low-pass
/// sections, each `(b0 + b1 z^-1 + b2 z^-2) / (1 - a1 z^-1 - a2 z^-2)` as `[b0, b1, b2, a1, a2]`, fitted to the
/// analogue ones' magnitudes up to 20 kHz, and the pole of the coupling's first-order high-pass.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Circuit {
    pub sections: [[i64; 5]; 2],
    pub high_pass: i64,
}

const THROUGH: [i64; 5] = [1 << 24, 0, 0, 0, 0];

impl Circuit {
    /// Model 1, boards VA3 to VA6.5: a first-order low-pass at 3,216 Hz and a high-pass at 23 Hz, as seven consoles'
    /// recordings have them (D-27).
    pub const MODEL1: Circuit = Circuit { sections: [[5_015_895, 2_578_393, 162_973, 5_223_622, 3_796_332], THROUGH], high_pass: 16_726_781 };
    /// Model 2, boards VA0 to VA1.8, whose YM3438 is in the main chip: a second-order low-pass at 5,805 Hz with a Q
    /// of 1.16, a first-order one at 4,111 Hz and a high-pass at 16 Hz, the nearest a filter comes to four recordings
    /// of an amplifier that also distorts (D-27).
    pub const MODEL2: Circuit = Circuit {
        sections: [[-266_812, 5_006_049, 1_993_670, 18_750_050, -8_705_742], [6_080_632, 3_145_731, 200_960, 3_932_671, 3_417_222]],
        high_pass: 16_742_115,
    };
    /// No circuit: the mix as it is.
    pub const FLAT: Circuit = Circuit { sections: [THROUGH, THROUGH], high_pass: 1 << 24 };
}

/// One side's place in a circuit: each section's last two inputs and outputs and the high-pass's last input and
/// output, in 256ths of an output unit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Side {
    x: [[i64; 2]; 2],
    y: [[i64; 2]; 2],
    high: [i64; 2],
}

impl Side {
    fn step(&mut self, c: &Circuit, x: i32) -> i16 {
        let mut v = (x as i64) << 8;
        for (k, s) in c.sections.iter().enumerate() {
            let y = (s[0] * v + s[1] * self.x[k][0] + s[2] * self.x[k][1] + s[3] * self.y[k][0] + s[4] * self.y[k][1]) >> 24;
            (self.x[k], self.y[k]) = ([v, self.x[k][0]], [y, self.y[k][0]]);
            v = y;
        }
        let h = (c.high_pass * (self.high[1] + v - self.high[0])) >> 24;
        self.high = [v, h];
        (h >> 8).clamp(-32768, 32767) as i16
    }
}

#[derive(Clone)]
pub struct Sound {
    pub ym: Ym2612,
    pub psg: Sn76489,
    /// The master clock the PSG has reached, on its clock's boundaries.
    pub psg_time: u64,
    fm_steps: Steps,
    psg_steps: Steps,
    fm_out: Vec<[i32; 2]>,
    /// The latest master clock an access or a frame's end has brought the sound to: the Z80 runs up to an
    /// instruction past the 68000's clock, and an access from behind it is taken as made then.
    pub time: u64,
    /// The FM stream's last seven samples at the output rate, and the PSG's last three, waiting beside them.
    fm_line: [[i32; 2]; 7],
    psg_line: [[i32; 2]; 3],
    /// The model's output circuit, and each side's place in it.
    pub circuit: Circuit,
    sides: [Side; 2],
}

impl Sound {
    /// The sound of a board whose master clock is `num / den` Hz: a model 1's discrete YM2612 and circuit, or a model
    /// 2's YM3438 and circuit.
    pub fn new(num: u64, den: u64, model1: bool) -> Sound {
        Sound {
            circuit: if model1 { Circuit::MODEL1 } else { Circuit::MODEL2 },
            sides: [Side::default(); 2],
            ym: Ym2612::new(model1),
            psg: Sn76489::new(Variant::SEGA),
            psg_time: 0,
            fm_steps: Steps::new(num, den, RATE),
            psg_steps: Steps::new(num, den, RATE),
            fm_out: Vec::new(),
            time: 0,
            fm_line: [[0; 2]; 7],
            psg_line: [[0; 2]; 3],
        }
    }

    fn now(&mut self, t: u64) -> u64 {
        self.time = self.time.max(t);
        self.time
    }

    /// The YM2612 up to `t`: each sample's sum over the rest level, held until the next.
    pub fn run_ym(&mut self, t: u64) {
        let (steps, rest) = (&mut self.fm_steps, self.ym.rest());
        self.ym.run(t, |at, out| steps.set(at, [(out[0] - rest) * FM_GAIN, (out[1] - rest) * FM_GAIN]));
    }

    /// The PSG up to the last of its clocks at or before `t`.
    pub fn run_psg(&mut self, t: u64) {
        if t <= self.psg_time {
            return;
        }
        let ticks = ((t - self.psg_time) / PSG_CLOCK) as u32;
        let (start, steps) = (self.psg_time, &mut self.psg_steps);
        self.psg.advance(ticks, |k, out| steps.set(start + k as u64 * PSG_CLOCK, [psg_level(out[0]), psg_level(out[1])]));
        self.psg_time += ticks as u64 * PSG_CLOCK;
    }

    /// A write takes effect at once: a level it changes is the output from `t`, after the PSG's last clock.
    pub fn psg_write(&mut self, v: u8, t: u64) {
        let t = self.now(t);
        self.run_psg(t);
        self.psg.write(v);
        let out = self.psg.output();
        self.psg_steps.set(t, [psg_level(out[0]), psg_level(out[1])]);
    }

    pub fn ym_write(&mut self, port: u16, v: u8, t: u64) {
        let t = self.now(t);
        self.run_ym(t);
        self.ym.write(port, v, t);
    }

    /// The reset line the YM2612 shares with the Z80, changed at `t`.
    pub fn ym_reset(&mut self, held: bool, t: u64) {
        let t = self.now(t);
        self.run_ym(t);
        let (steps, rest) = (&mut self.fm_steps, self.ym.rest());
        self.ym.flush(t, |at, out| steps.set(at, [(out[0] - rest) * FM_GAIN, (out[1] - rest) * FM_GAIN]));
        self.ym.reset_line(held, t);
    }

    pub fn ym_read(&mut self, port: u16, t: u64) -> u8 {
        let t = self.now(t);
        self.run_ym(t);
        self.ym.read(port, t)
    }

    /// Both chips up to `t`, and every finished sample, the two sources summed and put through the circuit.
    pub fn take(&mut self, t: u64, mut out: impl FnMut(i16, i16)) {
        let t = self.now(t);
        self.run_ym(t);
        self.run_psg(t);
        let fm = &mut self.fm_out;
        fm.clear();
        self.fm_steps.take(t, |l, r| fm.push([l, r]));
        let (mut i, circuit, sides, fm_line, psg_line) = (0, self.circuit, &mut self.sides, &mut self.fm_line, &mut self.psg_line);
        self.psg_steps.take(t, |l, r| {
            // The FM stream's held samples have their droop undone, three samples late; the PSG waits with it.
            fm_line.rotate_left(1);
            fm_line[6] = fm[i];
            i += 1;
            let unheld = |side: usize| {
                let f = |k: usize| fm_line[k][side] as i64;
                ((UNHOLD[0] * f(3) + UNHOLD[1] * (f(2) + f(4)) + UNHOLD[2] * (f(1) + f(5)) + UNHOLD[3] * (f(0) + f(6))) >> 24) as i32
            };
            let psg = psg_line[0];
            psg_line.rotate_left(1);
            psg_line[2] = [l, r];
            out(sides[0].step(&circuit, unheld(0) + psg[0]), sides[1].step(&circuit, unheld(1) + psg[1]));
        });
        debug_assert_eq!(i, fm.len(), "both synthesisers finish the same samples");
    }

    pub fn write_state(&self, w: &mut StateWriter) {
        self.ym.write_state(w);
        self.psg.write_state(w);
        w.u64s("PsgTimeAndTime", &[self.psg_time, self.time]);
        let lines: Vec<i32> = self.fm_line.iter().chain(&self.psg_line).flatten().copied().collect();
        w.i32s("Lines", &lines);
        self.fm_steps.write_state(w);
        self.psg_steps.write_state(w);
        let c = &self.circuit;
        let circuit: Vec<u64> = c.sections.iter().flatten().chain([&c.high_pass]).map(|&v| v as u64).collect();
        w.u64s("Circuit", &circuit);
        let sides: Vec<u64> = self.sides.iter().flat_map(|s| s.x.iter().chain(&s.y).flatten().chain(&s.high).copied().collect::<Vec<_>>()).map(|v| v as u64).collect();
        w.u64s("CircuitSides", &sides);
    }

    pub fn read_state(&mut self, r: &mut StateReader) -> Result<(), Truncated> {
        self.ym.read_state(r)?;
        self.psg.read_state(r)?;
        let mut t = [0u64; 2];
        r.u64s(&mut t)?;
        (self.psg_time, self.time) = (t[0], t[1]);
        let mut lines = [0i32; 20];
        r.i32s(&mut lines)?;
        for (k, pair) in lines.chunks(2).enumerate() {
            *if k < 7 { &mut self.fm_line[k] } else { &mut self.psg_line[k - 7] } = [pair[0], pair[1]];
        }
        self.fm_steps.read_state(r)?;
        self.psg_steps.read_state(r)?;
        let mut c = [0u64; 11];
        r.u64s(&mut c)?;
        let c = c.map(|v| v as i64);
        self.circuit = Circuit { sections: [0, 5].map(|k| std::array::from_fn(|i| c[k + i])), high_pass: c[10] };
        let mut v = [0u64; 20];
        r.u64s(&mut v)?;
        let v = v.map(|x| x as i64);
        let pair = |k: usize| [v[k], v[k + 1]];
        self.sides = [0, 10].map(|k| Side { x: [pair(k), pair(k + 2)], y: [pair(k + 4), pair(k + 6)], high: pair(k + 8) });
        Ok(())
    }
}

/// The PSG's level in the output's units.
fn psg_level(chip: i32) -> i32 {
    chip * PSG_GAIN.0 / PSG_GAIN.1
}

#[cfg(test)]
mod tests {
    use super::*;

    const NTSC: (u64, u64) = (4_725_000_000, 88);

    /// The frequency of the strongest of a few candidates, by their amplitudes in `x` at 48 kHz.
    fn amplitude(x: &[f64], f: f64) -> f64 {
        let (mut re, mut im) = (0.0, 0.0);
        for (n, &v) in x.iter().enumerate() {
            let a = 2.0 * std::f64::consts::PI * f * n as f64 / RATE as f64;
            re += v * a.cos();
            im += v * a.sin();
        }
        2.0 * (re * re + im * im).sqrt() / x.len() as f64
    }

    /// A PSG tone of half-period $0FE is the Z80's clock over 16 × 2 × 254, 440.4 Hz on NTSC, in the output.
    #[test]
    fn a_psg_tone_comes_out_at_its_frequency() {
        let mut s = Sound::new(NTSC.0, NTSC.1, true);
        for v in [0x8E, 0x0F, 0x90] {
            s.psg_write(v, 0);
        }
        let mut left = Vec::new();
        let second = NTSC.0 / NTSC.1;
        s.take(second, |l, _| left.push(l as f64));
        let mean = left.iter().sum::<f64>() / left.len() as f64;
        let x: Vec<f64> = left.iter().map(|v| v - mean).collect();
        let f = 3_579_545.454_5 / 16.0 / 2.0 / 254.0;
        let at = amplitude(&x, f);
        assert!(at > psg_level(8192) as f64 * 0.6, "{at} at {f} Hz");
        assert!(amplitude(&x, f * 1.05) < at * 0.05);
    }

    /// The Z80's write after the 68000's clock, then the frame's end at that earlier clock: both synthesisers finish
    /// the same samples, the frame ending where the write was.
    #[test]
    fn an_access_ahead_of_the_frames_end_moves_the_end_with_it() {
        let mut s = Sound::new(NTSC.0, NTSC.1, true);
        s.ym_write(0, 0x2B, 50_000);
        s.psg_write(0x90, 60_000);
        let mut n = 0;
        s.take(40_000, |_, _| n += 1);
        let mut more = 0;
        s.take(100_000, |_, _| more += 1);
        assert_eq!((n > 0, s.time), (true, 100_000));
        assert!(more > 0);
    }

    /// The DAC at 254 and a PSG channel at full volume, before any circuit: the DAC's pulses come to its level over
    /// the rest (to within the synthesiser's ripple) and the PSG's 0.324 of the DAC's swing is added, on either chip.
    #[test]
    fn the_dac_and_the_psg_add() {
        for model1 in [true, false] {
            let mut s = Sound::new(NTSC.0, NTSC.1, model1);
            s.circuit = Circuit::FLAT;
            for (a, v) in [(0x2B, 0x80), (0x2A, 0xFF)] {
                s.ym_write(0, a, 0);
                s.ym_write(1, v, 0);
            }
            s.psg_write(0x81, 0);
            s.psg_write(0x00, 0);
            s.psg_write(0x90, 0);
            let mut last = (0, 0);
            s.take(200_000, |l, r| last = (l, r));
            let want = 254 * FM_GAIN + psg_level(8192);
            assert_eq!(psg_level(8192), 2654);
            assert!((last.0 as i32 - want).abs() <= 16 && last.0 == last.1, "{last:?} for {want}, model 1 {model1}");
        }
    }

    /// Each circuit's digital filter against the analogue one it stands for, by a sine through one side: the model
    /// 1's first-order low-pass at 3,216 Hz and high-pass at 23 Hz; the model 2's second-order low-pass at 5,805 Hz
    /// with a Q of 1.16, first-order one at 4,111 Hz and high-pass at 16 Hz; each to a tenth of a decibel up to 8 kHz
    /// and three tenths above.
    #[test]
    fn the_circuits_follow_their_analogue_responses() {
        let level = |c: &Circuit, f: f64| {
            let mut side = Side::default();
            let x: Vec<f64> = (0..48_000).map(|n| side.step(c, (8000.0 * (2.0 * std::f64::consts::PI * f * n as f64 / RATE as f64).sin()) as i32) as f64).collect();
            20.0 * (amplitude(&x[24_000..], f) / 8000.0).log10()
        };
        for f in [50.0, 100.0, 1000.0, 3216.0, 8000.0, 16_000.0, 20_000.0] {
            let one = -10.0 * (1.0 + (f / 3216.0f64).powi(2)).log10() - 10.0 * (1.0 + (23.0 / f).powi(2)).log10();
            let w = f / 5805.0f64;
            let two = -10.0 * ((1.0 - w * w).powi(2) + (w / 1.16).powi(2)).log10() - 10.0 * (1.0 + (f / 4111.0f64).powi(2)).log10() - 10.0 * (1.0 + (16.0 / f).powi(2)).log10();
            let tolerance = if f > 8000.0 { 0.3 } else { 0.1 };
            assert!((level(&Circuit::MODEL1, f) - one).abs() < tolerance, "model 1 at {f}: {} for {one}", level(&Circuit::MODEL1, f));
            assert!((level(&Circuit::MODEL2, f) - two).abs() < tolerance, "model 2 at {f}: {} for {two}", level(&Circuit::MODEL2, f));
        }
        assert!(level(&Circuit::FLAT, 1000.0).abs() < 0.01);
    }

    /// The seven taps undo the droop of a sample held for one of the YM2612's 53,267 Hz samples, to a tenth of a
    /// decibel up to 20 kHz.
    #[test]
    fn the_unhold_taps_undo_a_held_samples_droop() {
        let fs = 53_693_175.0 / 7.0 / 144.0;
        for f in [100.0f64, 1000.0, 5000.0, 10_000.0, 15_000.0, 20_000.0] {
            let w = 2.0 * std::f64::consts::PI * f / RATE as f64;
            let taps = (UNHOLD[0] as f64 + 2.0 * (1..4).map(|k| UNHOLD[k] as f64 * (k as f64 * w).cos()).sum::<f64>()) / (1 << 24) as f64;
            let x = std::f64::consts::PI * f / fs;
            let droop = x.sin() / x;
            assert!((20.0 * (taps * droop).log10()).abs() < 0.1, "{f} Hz: {}", 20.0 * (taps * droop).log10());
        }
    }
}
