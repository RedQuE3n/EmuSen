//! C#'s `EmuSen.Common.SampleQueue`: a core's undrained stereo samples, the oldest pair dropped past the limit.
//!
//! The limit is the host's `AudioSettings.AudioBufferMaxSamples`, set through a core's `_set_audio_limit` export
//! (EmuSen_Settings_Reference.md §4.85.2). Nothing here is in a save state.

use std::collections::VecDeque;

/// C#'s `SampleQueue.DefaultLimit`.
pub const DEFAULT_LIMIT: usize = 128_000;

#[derive(Clone, Debug, PartialEq)]
pub struct SampleQueue {
    samples: VecDeque<i16>,
    limit: usize,
}

impl Default for SampleQueue {
    fn default() -> Self {
        SampleQueue::new(DEFAULT_LIMIT)
    }
}

impl SampleQueue {
    pub fn new(limit: usize) -> Self {
        SampleQueue { samples: VecDeque::new(), limit }
    }

    /// Room for `limit` samples up front, so a full queue never reallocates.
    pub fn with_capacity(limit: usize) -> Self {
        SampleQueue { samples: VecDeque::with_capacity(limit), limit }
    }

    pub fn limit(&self) -> usize {
        self.limit
    }

    /// Takes hold at the next pair, as C#'s does.
    pub fn set_limit(&mut self, limit: usize) {
        self.limit = limit;
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// `Enqueue`: the pair, then the oldest pairs dropped until within the limit.
    #[inline(always)]
    pub fn push_pair(&mut self, left: i16, right: i16) {
        self.samples.push_back(left);
        self.samples.push_back(right);
        while self.samples.len() > self.limit && self.samples.len() >= 2 {
            self.samples.pop_front();
            self.samples.pop_front();
        }
    }

    /// `Drain`: whole pairs, oldest first, at most `max_frames` of them and no more than `out` holds; returns the samples written.
    pub fn drain(&mut self, out: &mut [i16], max_frames: usize) -> usize {
        let mut wanted = max_frames.saturating_mul(2).min(self.samples.len()).min(out.len());
        wanted -= wanted & 1;
        for (slot, sample) in out.iter_mut().zip(self.samples.drain(..wanted)) {
            *slot = sample;
        }
        wanted
    }

    pub fn clear(&mut self) {
        self.samples.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_oldest_pairs_go_first_past_the_limit() {
        let mut q = SampleQueue::new(6);
        for i in 0..5 {
            q.push_pair(i * 2, i * 2 + 1);
        }
        let mut out = [0i16; 16];
        assert_eq!(q.drain(&mut out, 100), 6);
        assert_eq!(out[..6], [4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn an_odd_limit_keeps_whole_pairs() {
        let mut q = SampleQueue::new(5);
        for i in 0..4 {
            q.push_pair(i, -i);
        }
        assert_eq!(q.len(), 4);
        let mut q = SampleQueue::new(1);
        q.push_pair(1, 2);
        assert_eq!(q.len(), 0);
    }

    #[test]
    fn a_lowered_limit_takes_hold_at_the_next_pair() {
        let mut q = SampleQueue::new(100);
        for i in 0..10 {
            q.push_pair(i, i);
        }
        q.set_limit(4);
        assert_eq!(q.len(), 20);
        q.push_pair(10, 10);
        assert_eq!(q.len(), 4);
    }

    #[test]
    fn a_drain_takes_whole_pairs_only() {
        let mut q = SampleQueue::default();
        for i in 0..4 {
            q.push_pair(i, i);
        }
        let mut out = [0i16; 5];
        assert_eq!(q.drain(&mut out, 100), 4);
        assert_eq!(q.drain(&mut out, 0), 0);
        assert_eq!(q.drain(&mut out, 1), 2);
        assert_eq!(q.len(), 2);
    }
}
