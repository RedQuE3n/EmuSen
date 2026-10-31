//! Sound as the core made it, in runs tagged with their rate so that no drain crosses a change; never resampled here
//! (EmuSen_LibretroHost.md §4.3). The last run is kept when empty, so a steady rate allocates nothing per frame.

use std::collections::VecDeque;

/// A run of interleaved stereo samples made at one rate.
struct Run {
    rate: (u64, u64),
    samples: VecDeque<i16>,
}

pub struct Sound {
    runs: VecDeque<Run>,
    /// The rate new samples are made at, exact.
    rate: (u64, u64),
    /// The drop-oldest bound, in samples (two per stereo frame).
    limit: usize,
    buffered: usize,
}

/// The rate rounded to the hertz, as `audio_rate` reports it.
pub fn rounded((num, den): (u64, u64)) -> i32 {
    if den == 0 { 0 } else { ((num + den / 2) / den).min(i32::MAX as u64) as i32 }
}

impl Sound {
    pub fn new(rate: (u64, u64)) -> Sound {
        Sound { runs: VecDeque::new(), rate, limit: usize::MAX, buffered: 0 }
    }

    pub fn set_rate(&mut self, rate: (u64, u64)) {
        self.rate = rate;
    }

    pub fn rate(&self) -> (u64, u64) {
        self.rate
    }

    /// The rate of the next sample a drain returns: the oldest run's, or the current one's when nothing is buffered.
    pub fn next_rate(&self) -> (u64, u64) {
        self.runs.iter().find(|r| !r.samples.is_empty()).map_or(self.rate, |r| r.rate)
    }

    pub fn buffered(&self) -> usize {
        self.buffered
    }

    pub fn set_limit(&mut self, samples: usize) {
        self.limit = samples.max(2) & !1;
        self.trim();
    }

    /// Interleaved stereo samples appended at the current rate.
    pub fn push(&mut self, samples: &[i16]) {
        let whole = samples.len() & !1;
        if whole == 0 {
            return;
        }
        match self.runs.back_mut() {
            Some(run) if run.rate == self.rate || run.samples.is_empty() => {
                run.rate = self.rate;
                run.samples.extend(&samples[..whole]);
            }
            _ => self.runs.push_back(Run { rate: self.rate, samples: samples[..whole].iter().copied().collect() }),
        }
        self.buffered += whole;
        self.trim();
    }

    fn trim(&mut self) {
        while self.buffered > self.limit {
            let excess = self.buffered - self.limit;
            let Some(front) = self.runs.front_mut() else { break };
            let n = excess.min(front.samples.len());
            front.samples.drain(..n);
            self.buffered -= n;
            if front.samples.is_empty() && self.runs.len() > 1 {
                self.runs.pop_front();
            }
        }
    }

    /// Whole stereo frames into `out`, at most `max_frames`, from the oldest run only; the samples written and their rate.
    pub fn drain(&mut self, out: &mut [i16], max_frames: usize) -> (usize, (u64, u64)) {
        let rate = self.next_rate();
        let Some(front) = self.runs.front_mut() else { return (0, rate) };
        let n = front.samples.len().min(max_frames.saturating_mul(2)).min(out.len() & !1);
        for (o, s) in out.iter_mut().zip(front.samples.drain(..n)) {
            *o = s;
        }
        self.buffered -= n;
        if front.samples.is_empty() && self.runs.len() > 1 {
            self.runs.pop_front();
        }
        (n, rate)
    }

    /// The buffered samples copied without draining, across runs; how many are buffered.
    pub fn peek(&self, out: &mut [i16]) -> usize {
        for (o, s) in out.iter_mut().zip(self.runs.iter().flat_map(|r| r.samples.iter())) {
            *o = *s;
        }
        self.buffered
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_drain_stops_at_a_rate_change() {
        let mut s = Sound::new((48_000, 1));
        s.push(&[1, 2, 3, 4]);
        s.set_rate((64_081, 2));
        s.push(&[5, 6]);
        let mut out = [0i16; 8];
        assert_eq!(s.drain(&mut out, 4), (4, (48_000, 1)));
        assert_eq!(&out[..4], &[1, 2, 3, 4]);
        assert_eq!(s.next_rate(), (64_081, 2));
        assert_eq!(s.drain(&mut out, 4), (2, (64_081, 2)));
        assert_eq!(rounded((64_081, 2)), 32_041);
    }

    #[test]
    fn the_limit_drops_the_oldest_whole_frames() {
        let mut s = Sound::new((48_000, 1));
        s.set_limit(4);
        s.push(&[1, 2, 3, 4, 5, 6]);
        let mut out = [0i16; 6];
        assert_eq!(s.drain(&mut out, 3).0, 4);
        assert_eq!(&out[..4], &[3, 4, 5, 6]);
    }

    #[test]
    fn half_a_frame_is_not_taken() {
        let mut s = Sound::new((48_000, 1));
        s.push(&[1, 2, 3]);
        assert_eq!(s.buffered(), 2);
    }
}
