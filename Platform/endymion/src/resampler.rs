//! A phase-continuous linear resampler for interleaved stereo: the C# `LinearResampler`. See EmuSen_Audio_Sync.md §2.

/// Why a ratio was refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RatioError {
    /// Zero or below, which C# refuses with these words.
    NotPositive,
    /// NaN or infinite, which C# takes and never returns from: no step it makes reaches the next input frame.
    NotFinite,
}

impl RatioError {
    pub fn words(self) -> &'static str {
        match self {
            RatioError::NotPositive => "Resample ratio must be positive.",
            RatioError::NotFinite => "Resample ratio must be finite.",
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct LinearResampler {
    /// Where the next output falls between the previous frame and the next unconsumed one.
    frac: f64,
    prev_left: i16,
    prev_right: i16,
    primed: bool,
}

impl LinearResampler {
    pub fn new() -> LinearResampler {
        LinearResampler::default()
    }

    pub fn reset(&mut self) {
        *self = LinearResampler::default();
    }

    /// Output frames per input frame as `ratio`, appended to `out`. The arithmetic is C#'s step for step: a double for the position, half to even for the rounding.
    pub fn resample(&mut self, input: &[i16], ratio: f64, out: &mut Vec<i16>) -> Result<(), RatioError> {
        if ratio <= 0.0 {
            return Err(RatioError::NotPositive);
        }
        if !ratio.is_finite() {
            return Err(RatioError::NotFinite);
        }
        if input.len() < 2 {
            return Ok(());
        }
        let frames = input.len() / 2;
        let mut i = 0;
        // The first call ever has no previous frame to interpolate from.
        if !self.primed {
            self.prev_left = input[0];
            self.prev_right = input[1];
            self.frac = 0.0;
            self.primed = true;
            i = 1;
        }
        let step = 1.0 / ratio;
        out.reserve(((frames as f64 * ratio) as usize).saturating_mul(2).saturating_add(4));
        loop {
            while self.frac >= 1.0 {
                if i >= frames {
                    return Ok(());
                }
                self.prev_left = input[i * 2];
                self.prev_right = input[i * 2 + 1];
                i += 1;
                self.frac -= 1.0;
            }
            if i >= frames {
                return Ok(());
            }
            let (next_left, next_right) = (input[i * 2], input[i * 2 + 1]);
            out.push(lerp(self.prev_left, next_left, self.frac));
            out.push(lerp(self.prev_right, next_right, self.frac));
            self.frac += step;
        }
    }
}

/// `(short)Math.Round(prev + (next - prev) * frac)`: the difference an integer, widened, then rounded half to even.
fn lerp(prev: i16, next: i16, frac: f64) -> i16 {
    (prev as f64 + (next as i32 - prev as i32) as f64 * frac).round_ties_even() as i16
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(resampler: &mut LinearResampler, input: &[i16], ratio: f64) -> Vec<i16> {
        let mut out = Vec::new();
        resampler.resample(input, ratio, &mut out).unwrap();
        out
    }

    #[test]
    fn a_ratio_of_one_passes_frames_through_one_behind() {
        let mut resampler = LinearResampler::new();
        assert_eq!(run(&mut resampler, &[1, -1, 2, -2, 3, -3], 1.0), [1, -1, 2, -2]);
        assert_eq!(run(&mut resampler, &[4, -4], 1.0), [3, -3]);
    }

    #[test]
    fn half_way_rounds_to_even() {
        let mut resampler = LinearResampler::new();
        assert_eq!(run(&mut resampler, &[0, 0, 1, 3, 2, 5], 2.0), [0, 0, 0, 2, 1, 3, 2, 4]);
        assert_eq!(lerp(0, 1, 0.5), 0);
        assert_eq!(lerp(1, 2, 0.5), 2);
        assert_eq!(lerp(-1, -2, 0.5), -2);
        assert_eq!(lerp(i16::MIN, i16::MAX, 0.999999), 32767);
    }

    #[test]
    fn a_ratio_is_refused_as_csharp_refuses_it_and_where_csharp_would_never_return() {
        let mut resampler = LinearResampler::new();
        let mut out = Vec::new();
        assert_eq!(resampler.resample(&[1, 1], 0.0, &mut out), Err(RatioError::NotPositive));
        assert_eq!(resampler.resample(&[1, 1], f64::NEG_INFINITY, &mut out), Err(RatioError::NotPositive));
        assert_eq!(resampler.resample(&[1, 1], f64::NAN, &mut out), Err(RatioError::NotFinite));
        assert_eq!(resampler.resample(&[1, 1], f64::INFINITY, &mut out), Err(RatioError::NotFinite));
        assert_eq!(resampler.resample(&[1], 1.0, &mut out), Ok(()));
        assert!(out.is_empty());
    }

    #[test]
    fn a_reset_forgets_the_previous_frame() {
        let mut resampler = LinearResampler::new();
        run(&mut resampler, &[100, 100, 200, 200], 1.0);
        resampler.reset();
        assert_eq!(run(&mut resampler, &[7, 7, 9, 9], 1.0), [7, 7]);
    }
}
