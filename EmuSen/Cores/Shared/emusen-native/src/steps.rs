//! A band-limited step synthesiser: a source whose output is held between changes (a DAC, a square wave) is given as
//! levels at times on the machine's clock, and comes out as stereo samples at the output rate with nothing above the
//! kernel's cutoff. Each change adds a windowed sinc, scaled by the change, to a buffer of differences that is summed
//! into samples once no later change can reach them. EmuSen_NativeCores.md §12.6 is the design and its measurements.
//!
//! The kernel is computed with additions, multiplications and divisions only, so that it is the same on every
//! machine, and its state is a fixed block, so that a loaded state continues as the saved machine would.

use crate::{StateReader, StateWriter, Truncated};

/// Output samples on either side of a change that it reaches.
pub const HALF: usize = 16;
/// Kernel taps, and positions between two output samples that a change is placed at.
pub const TAPS: usize = 2 * HALF;
pub const PHASES: usize = 256;
/// What a kernel's taps sum to: a change of 1 moves the output by 1 once summed.
const ONE_SHIFT: u32 = 15;
const ONE: i64 = 1 << ONE_SHIFT;
/// Differences that may wait to be summed: a change reaches `HALF` samples past the present one.
const PENDING: usize = 2 * TAPS;
/// The cutoff, as a fraction of the output rate (0.45: 21.6 kHz at 48 kHz).
const CUTOFF: f64 = 0.45;

/// sin(x) from its series after reduction to [-π/2, π/2]: the same bits on every machine.
fn sine(x: f64) -> f64 {
    const PI: f64 = std::f64::consts::PI;
    let mut x = x % (2.0 * PI);
    if x > PI {
        x -= 2.0 * PI;
    } else if x < -PI {
        x += 2.0 * PI;
    }
    if x > PI / 2.0 {
        x = PI - x;
    } else if x < -PI / 2.0 {
        x = -PI - x;
    }
    let (x2, mut term, mut sum) = (x * x, x, x);
    for n in 1..12 {
        term *= -x2 / ((2 * n) as f64 * (2 * n + 1) as f64);
        sum += term;
    }
    sum
}

fn cosine(x: f64) -> f64 {
    sine(x + std::f64::consts::PI / 2.0)
}

/// The windowed sinc: the cutoff's impulse response under a Blackman window over `TAPS` samples.
fn impulse(x: f64) -> f64 {
    const PI: f64 = std::f64::consts::PI;
    if x <= -(HALF as f64) || x >= HALF as f64 {
        return 0.0;
    }
    let sinc = if x == 0.0 { 2.0 * CUTOFF } else { sine(2.0 * PI * CUTOFF * x) / (PI * x) };
    let w = (x + HALF as f64) / TAPS as f64;
    sinc * (0.42 - 0.5 * cosine(2.0 * PI * w) + 0.08 * cosine(4.0 * PI * w))
}

/// Each phase's taps: the band-limited step's rise over each output sample, its value at sample n less its value at
/// n - 1, the step being the windowed sinc's integral (Simpson's rule on a grid of `PHASES` points a sample). Taps of
/// the impulse itself would be summed as a discrete integrator, whose gain is (ω/2)/sin(ω/2) of the true one's.
fn kernel() -> Vec<[i32; TAPS]> {
    let step = 1.0 / PHASES as f64;
    // The step at x = j / PHASES - HALF, j = 0 ..= 2 * HALF * PHASES, and 0 before.
    let points = 2 * HALF * PHASES;
    let mut integral = vec![0f64; points + 1];
    for j in 0..points {
        let a = j as f64 * step - HALF as f64;
        integral[j + 1] = integral[j] + step / 6.0 * (impulse(a) + 4.0 * impulse(a + step / 2.0) + impulse(a + step));
    }
    let total = integral[points];
    let at = |x_phases: i64| -> f64 {
        let j = x_phases + (HALF * PHASES) as i64;
        if j <= 0 { 0.0 } else if j as usize >= points { 1.0 } else { integral[j as usize] / total }
    };
    (0..PHASES)
        .map(|ph| {
            let mut f = [0f64; TAPS];
            for (k, v) in f.iter_mut().enumerate() {
                // Tap k is sample i + 1 - HALF + k, at x = k + 1 - HALF - frac from the change.
                let x = (k as i64 + 1 - HALF as i64) * PHASES as i64 - ph as i64;
                *v = at(x) - at(x - PHASES as i64);
            }
            let mut taps = [0i32; TAPS];
            let mut sum = 0i64;
            for k in 0..TAPS {
                taps[k] = (f[k] * ONE as f64).round() as i32;
                sum += taps[k] as i64;
            }
            taps[HALF] += (ONE - sum) as i32;
            taps
        })
        .collect()
}

/// The synthesiser for one stereo output: its clock's rate over the output's, the levels it holds, the differences
/// not yet summed, and the samples finished since the last `take`.
#[derive(Clone)]
pub struct Steps {
    /// Output positions (in `PHASES` a sample) an input clock: `rate * den * PHASES / num`, as the fraction itself.
    scale: (u128, u128),
    kernel: Vec<[i32; TAPS]>,
    /// The levels held now, and the sum of every difference before `first`.
    level: [i32; 2],
    sum: [i64; 2],
    /// The first output sample not yet summed, and the differences from it on, sample `n` at `n % PENDING`.
    first: u64,
    diff: [[i64; 2]; PENDING],
    /// Finished samples, empty after every `take`: not part of a state, which is taken between frames.
    done: Vec<[i32; 2]>,
}

impl Steps {
    /// A synthesiser for a clock of `num / den` Hz resampled to `rate` Hz.
    pub fn new(num: u64, den: u64, rate: u32) -> Steps {
        Steps {
            scale: (rate as u128 * den as u128 * PHASES as u128, num as u128),
            kernel: kernel(),
            level: [0; 2],
            sum: [0; 2],
            first: 0,
            diff: [[0; 2]; PENDING],
            done: Vec::new(),
        }
    }

    /// Clock `t` as an output position, `HALF` samples late so that no change reaches before sample 0.
    fn position(&self, t: u64) -> u128 {
        t as u128 * self.scale.0 / self.scale.1 + (HALF * PHASES) as u128
    }

    /// Sums every sample no change at or after `t` can reach.
    fn finish(&mut self, t: u64) {
        let last = (self.position(t) / PHASES as u128) as u64 + 1;
        while self.first + (HALF as u64) < last {
            let d = std::mem::take(&mut self.diff[self.first as usize % PENDING]);
            self.sum[0] += d[0];
            self.sum[1] += d[1];
            self.done.push([(self.sum[0] >> ONE_SHIFT) as i32, (self.sum[1] >> ONE_SHIFT) as i32]);
            self.first += 1;
        }
    }

    /// The source holds `level` from clock `t` on; `t` never goes back.
    #[inline]
    pub fn set(&mut self, t: u64, level: [i32; 2]) {
        let delta = [level[0] - self.level[0], level[1] - self.level[1]];
        if delta == [0, 0] {
            return;
        }
        self.finish(t);
        self.level = level;
        let p = self.position(t);
        let (i, ph) = ((p / PHASES as u128) as u64, (p % PHASES as u128) as usize);
        let taps = &self.kernel[ph];
        for (k, &tap) in taps.iter().enumerate() {
            let n = (i + 1 + k as u64).wrapping_sub(HALF as u64);
            if n < self.first || n > i + HALF as u64 {
                continue;
            }
            let d = &mut self.diff[n as usize % PENDING];
            d[0] += delta[0] as i64 * tap as i64;
            d[1] += delta[1] as i64 * tap as i64;
        }
    }

    /// Every sample no change at or after `t` can reach, oldest first, handed to `out` and forgotten.
    pub fn take(&mut self, t: u64, mut out: impl FnMut(i32, i32)) {
        self.finish(t);
        for [l, r] in self.done.drain(..) {
            out(l, r);
        }
    }

    /// The samples finished so far.
    pub fn emitted(&self) -> u64 {
        self.first
    }

    pub fn write_state(&self, w: &mut StateWriter) {
        w.i32s("Level", &self.level);
        w.u64s("Sum", &[self.sum[0] as u64, self.sum[1] as u64, self.first]);
        let pending: Vec<u64> = (0..PENDING as u64).flat_map(|k| self.diff[((self.first + k) % PENDING as u64) as usize]).map(|v| v as u64).collect();
        w.u64s("Pending", &pending);
    }

    pub fn read_state(&mut self, r: &mut StateReader) -> Result<(), Truncated> {
        let mut level = [0i32; 2];
        r.i32s(&mut level)?;
        let mut s = [0u64; 3];
        r.u64s(&mut s)?;
        let mut p = [0u64; 2 * PENDING];
        r.u64s(&mut p)?;
        self.level = level;
        self.sum = [s[0] as i64, s[1] as i64];
        self.first = s[2];
        for k in 0..PENDING {
            self.diff[((self.first + k as u64) % PENDING as u64) as usize] = [p[2 * k] as i64, p[2 * k + 1] as i64];
        }
        self.done.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A clock of 1,000 Hz into 100 Hz: ten clocks a sample.
    fn ten() -> Steps {
        Steps::new(1000, 1, 100)
    }

    fn run(s: &mut Steps, to: u64) -> Vec<i32> {
        let mut v = Vec::new();
        s.take(to, |l, _| v.push(l));
        v
    }

    #[test]
    fn every_phase_sums_to_one_and_the_kernel_is_its_own_mirror() {
        let k = kernel();
        for (ph, taps) in k.iter().enumerate() {
            assert_eq!(taps.iter().map(|&t| t as i64).sum::<i64>(), ONE, "phase {ph}");
        }
        // At phase 0 the step is at a sample, so the rise into sample HALF-1+m mirrors the rise into HALF-m.
        for m in 1..HALF {
            assert!((k[0][HALF - 1 + m] - k[0][HALF - m]).abs() <= 1, "phase 0, {m} from the change: {:?}", k[0]);
        }
    }

    #[test]
    fn a_step_settles_on_its_level_and_is_half_way_at_its_time() {
        let mut s = ten();
        s.set(1000, [1000, -1000]);
        let v = run(&mut s, 3000);
        // Sample n here is the source at n - HALF.
        assert_eq!(v.len(), 300 + 1);
        let at = 100 + HALF;
        assert!(v[..at - HALF].iter().all(|&x| x.abs() <= 1), "before the step: {:?}", &v[at - HALF - 10..at - HALF]);
        assert!(v[at + HALF..].iter().all(|&x| x == 1000), "after the step: {:?}", &v[at + HALF..at + HALF + 10]);
        assert!((v[at] - 500).abs() <= 2, "at the step: {}", v[at]);
    }

    #[test]
    fn a_square_wave_keeps_its_harmonics_below_the_cutoff_and_aliases_none_from_above() {
        // A square of `half` clocks high and `half` low on a 960 kHz clock, into 48 kHz: the amplitude at each frequency.
        let amplitudes = |half: u64, at: &[f64]| -> Vec<f64> {
            let mut s = Steps::new(960_000, 1, 48_000);
            let mut out = Vec::new();
            for n in 0..960_000u64 {
                s.set(n, if (n / half).is_multiple_of(2) { [8192, 8192] } else { [-8192, -8192] });
            }
            s.take(960_000, |l, _| out.push(l as f64));
            at.iter()
                .map(|&f| {
                    let (mut re, mut im) = (0.0, 0.0);
                    for (n, &x) in out.iter().enumerate() {
                        let a = 2.0 * std::f64::consts::PI * f * n as f64 / 48_000.0;
                        re += x * cosine(a);
                        im += x * sine(a);
                    }
                    2.0 * (re * re + im * im).sqrt() / out.len() as f64
                })
                .collect()
        };
        let ideal = |h: f64| 8192.0 * 4.0 / std::f64::consts::PI / h;
        // 1,714 Hz: the fundamental and the third and fifth harmonics in the passband, each its own size.
        let f0 = 960_000.0 / 560.0;
        let a = amplitudes(280, &[f0, 3.0 * f0, 5.0 * f0]);
        for (h, got) in [(1.0, a[0]), (3.0, a[1]), (5.0, a[2])] {
            assert!((got / ideal(h) - 1.0).abs() < 0.01, "harmonic {h}: {got} against {}", ideal(h));
        }
        // 6,857 Hz: its fifth harmonic, 34,286 Hz, would alias to 13,714 Hz, twice the fundamental, where a square has
        // nothing of its own.
        let f0 = 960_000.0 / 140.0;
        let a = amplitudes(70, &[f0, 2.0 * f0]);
        assert!((a[0] / ideal(1.0) - 1.0).abs() < 0.01, "fundamental {}", a[0]);
        assert!(a[1] < a[0] * 1e-3, "an alias at {}: {}", 2.0 * f0, a[1]);
    }

    #[test]
    fn a_state_saved_between_steps_continues_alike() {
        let mut a = ten();
        a.set(1005, [300, 7]);
        let mut x = run(&mut a, 1100);
        let mut w = StateWriter::counter();
        a.write_state(&mut w);
        let mut bytes = vec![0u8; w.len()];
        a.write_state(&mut StateWriter::new(&mut bytes));
        let mut b = ten();
        b.read_state(&mut StateReader::new(&bytes)).unwrap();
        for s in [&mut a, &mut b] {
            s.set(1113, [-40, 7]);
            s.set(1150, [0, 0]);
        }
        let (ya, yb) = (run(&mut a, 2000), run(&mut b, 2000));
        assert_eq!(ya, yb);
        x.extend(ya);
        assert_eq!(*x.last().unwrap(), 0);
    }

    #[test]
    fn the_sine_is_the_series_to_the_last_digit_that_matters() {
        for i in -40..40 {
            let x = i as f64 * 0.37;
            assert!((sine(x) - x.sin()).abs() < 1e-12, "{x}");
        }
    }
}
