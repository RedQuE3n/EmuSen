//! The speed a session runs at, the C# `SpeedController`: a percentage of the console's rate, the turbo and slow
//! motion it switches to, which frames are drawn and when the sound is played (EmuSen_Rewind_And_FastForward.md §2).

pub const NORMAL_PERCENT: i32 = 100;
pub const UNTHROTTLED_PERCENT: i32 = 0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Speed {
    percent: i32,
    turbo: i32,
    slow: i32,
    max_frame_skip: i32,
}

impl Default for Speed {
    fn default() -> Speed {
        Speed { percent: NORMAL_PERCENT, turbo: 300, slow: 25, max_frame_skip: 9 }
    }
}

impl Speed {
    pub fn percent(&self) -> i32 {
        self.percent
    }

    pub fn set_percent(&mut self, value: i32) {
        self.percent = value.max(0);
    }

    pub fn turbo(&self) -> i32 {
        self.turbo
    }

    pub fn set_turbo(&mut self, value: i32) {
        self.turbo = value.max(NORMAL_PERCENT);
    }

    pub fn slow(&self) -> i32 {
        self.slow
    }

    pub fn set_slow(&mut self, value: i32) {
        self.slow = value.clamp(1, NORMAL_PERCENT);
    }

    /// Nanoseconds between frames, zero for no wait, as `TimeSpan.FromSeconds` truncates to ticks.
    pub fn frame_interval_ns(&self, hz: f64) -> u64 {
        if self.percent == UNTHROTTLED_PERCENT || hz <= 0.0 {
            return 0;
        }
        let ticks = (NORMAL_PERCENT as f64 / (hz * self.percent as f64)) * 10_000_000.0;
        (ticks as i64).max(0) as u64 * 100
    }

    /// One frame in every n drawn, so the rate a player sees stays near the console's.
    pub fn render_every(&self) -> i32 {
        if self.percent == UNTHROTTLED_PERCENT {
            return self.max_frame_skip + 1;
        }
        if self.percent <= NORMAL_PERCENT {
            return 1;
        }
        (self.percent / NORMAL_PERCENT).min(self.max_frame_skip + 1)
    }

    pub fn should_render(&self, frame: i64) -> bool {
        let n = self.render_every() as i64;
        n <= 1 || frame % n == 0
    }

    pub fn should_play_audio(&self) -> bool {
        (50..=200).contains(&self.percent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_speeds_draw_and_play_as_the_c_sharp_controller_does() {
        let mut s = Speed::default();
        assert_eq!((s.render_every(), s.should_play_audio(), s.frame_interval_ns(60.0)), (1, true, 16_666_600));
        s.set_percent(350);
        assert_eq!((s.render_every(), s.should_render(3), s.should_render(4), s.should_play_audio()), (3, true, false, false));
        s.set_percent(0);
        assert_eq!((s.render_every(), s.frame_interval_ns(60.0)), (10, 0));
        s.set_turbo(20);
        s.set_slow(500);
        assert_eq!((s.turbo(), s.slow()), (100, 100));
    }
}
