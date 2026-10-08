//! Absorbs clock drift by resampling, never by discarding: the C# `DynamicRateControl`. See EmuSen_Audio_Sync.md §1 and §3.

use crate::clamp;
use crate::resampler::{LinearResampler, RatioError};

#[derive(Clone, Debug)]
pub struct DynamicRateControl {
    resampler: LinearResampler,
    shedding: bool,
    /// Where the output queue is steered to sit, in frames.
    pub target_queued_frames: i32,
    /// The largest departure of the ratio from the nominal one.
    pub max_deviation: f64,
    /// Above the target times this, input is shed until it drains below the target times the exit factor.
    pub shedding_entry_factor: f64,
    pub shedding_exit_factor: f64,
    /// The ratio the drift is centred on.
    pub nominal_ratio: f64,
    pub last_ratio: f64,
    pub shedding_events: i32,
    pub total_input_frames: i64,
    pub total_output_frames: i64,
}

impl DynamicRateControl {
    pub fn new(target_queued_frames: i32) -> DynamicRateControl {
        DynamicRateControl {
            resampler: LinearResampler::new(),
            shedding: false,
            target_queued_frames,
            max_deviation: 0.005,
            shedding_entry_factor: 3.0,
            shedding_exit_factor: 2.0,
            nominal_ratio: 1.0,
            last_ratio: 1.0,
            shedding_events: 0,
            total_input_frames: 0,
            total_output_frames: 0,
        }
    }

    pub fn is_shedding(&self) -> bool {
        self.shedding
    }

    /// The ratio the queue's fill calls for.
    pub fn compute_ratio(&self, queued_frames: i32) -> f64 {
        if self.target_queued_frames <= 0 {
            return self.nominal_ratio;
        }
        let target = self.target_queued_frames as f64;
        let delta = (queued_frames as f64 - target) / target;
        self.nominal_ratio * (1.0 - clamp(delta, -1.0, 1.0) * self.max_deviation)
    }

    /// What to hand the output device, appended to `out`; nothing while shedding.
    pub fn process(&mut self, input: &[i16], queued_frames: i32, out: &mut Vec<i16>) -> Result<(), RatioError> {
        self.last_ratio = self.compute_ratio(queued_frames);
        self.total_input_frames = self.total_input_frames.wrapping_add((input.len() / 2) as i64);
        let queued = queued_frames as f64;
        if self.shedding {
            if queued > self.target_queued_frames as f64 * self.shedding_exit_factor {
                return Ok(());
            }
            self.shedding = false;
            self.resampler.reset();
        } else if queued > self.target_queued_frames as f64 * self.shedding_entry_factor {
            self.shedding = true;
            self.shedding_events = self.shedding_events.wrapping_add(1);
            self.resampler.reset();
            return Ok(());
        }
        if input.len() < 2 {
            return Ok(());
        }
        let before = out.len();
        self.resampler.resample(input, self.last_ratio, out)?;
        self.total_output_frames = self.total_output_frames.wrapping_add(((out.len() - before) / 2) as i64);
        Ok(())
    }

    /// On any discontinuity.
    pub fn reset(&mut self) {
        self.resampler.reset();
        self.shedding = false;
        self.last_ratio = self.nominal_ratio;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ratio_leans_against_the_queue_within_its_bound() {
        let control = DynamicRateControl::new(1000);
        assert_eq!(control.compute_ratio(1000), 1.0);
        assert_eq!(control.compute_ratio(2000), 0.995);
        assert_eq!(control.compute_ratio(9000), 0.995);
        assert_eq!(control.compute_ratio(0), 1.005);
        assert_eq!(DynamicRateControl::new(0).compute_ratio(5), 1.0);
    }

    #[test]
    fn a_queue_far_over_its_target_is_shed_until_it_drains() {
        let mut control = DynamicRateControl::new(100);
        let input = [1i16; 64];
        let mut out = Vec::new();
        control.process(&input, 301, &mut out).unwrap();
        assert!(out.is_empty() && control.is_shedding() && control.shedding_events == 1);
        control.process(&input, 201, &mut out).unwrap();
        assert!(out.is_empty() && control.is_shedding());
        control.process(&input, 200, &mut out).unwrap();
        assert!(!control.is_shedding() && !out.is_empty());
        assert_eq!(control.total_input_frames, 96);
        assert_eq!(control.total_output_frames, (out.len() / 2) as i64);
    }

    #[test]
    fn a_reset_returns_to_the_nominal_ratio() {
        let mut control = DynamicRateControl::new(100);
        control.nominal_ratio = 1.25;
        control.process(&[0; 8], 50, &mut Vec::new()).unwrap();
        assert_ne!(control.last_ratio, 1.25);
        control.reset();
        assert_eq!(control.last_ratio, 1.25);
    }
}
