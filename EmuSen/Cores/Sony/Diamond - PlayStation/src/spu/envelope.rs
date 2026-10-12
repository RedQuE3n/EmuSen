//! The envelope step that ADSR and the volume sweeps share, as psx-spx's "Envelope Operation depending on
//! Shift/Step/Mode/Direction" gives it (Diamond_SPU.md §4).

/// A rate: the shift (0..31, fast to slow), the step (0..3, "+7..+4" or "-8..-5"), and the direction, mode and phase.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rate {
    pub shift: u8,
    pub step: u8,
    pub decreasing: bool,
    pub exponential: bool,
    pub negative: bool,
}

impl Rate {
    /// A volume register's sweep (psx-spx "Sweep Volume Mode"): mode bit 14, direction 13, phase 12, shift 6-2, step 1-0.
    pub fn sweep(reg: u16) -> Rate {
        Rate { shift: (reg >> 2 & 0x1F) as u8, step: (reg & 3) as u8, decreasing: reg & 1 << 13 != 0, exponential: reg & 1 << 14 != 0, negative: reg & 1 << 12 != 0 }
    }
}

/// An envelope's level and the counter that paces its steps.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Envelope {
    pub level: i16,
    pub counter: u16,
}

impl Envelope {
    /// One 44,100 Hz cycle at `rate`: the counter advances, and when its bit 15 sets the level takes a step and the
    /// counter starts again (the reset is Diamond_SPU.md §4's choice).
    pub fn tick(&mut self, rate: Rate) {
        let shift = rate.shift as i32;
        let mut step = 7 - rate.step as i32;
        if rate.decreasing != rate.negative {
            step = !step;
        }
        step <<= (11 - shift).max(0);
        let mut increment = 0x8000u32 >> (shift - 11).max(0);
        let level = self.level as i32;
        if rate.exponential && !rate.decreasing && level > 0x6000 {
            if shift < 10 {
                step >>= 2;
            } else if shift >= 11 {
                increment >>= 2;
            } else {
                step >>= 1;
                increment >>= 1;
            }
        } else if rate.exponential && rate.decreasing {
            step = step * level >> 15;
        }
        if (rate.step as i32 | shift << 2) != 0x7F {
            increment = increment.max(1);
        }
        let counter = self.counter as u32 + increment;
        if counter & 0x8000 == 0 {
            self.counter = counter as u16;
            return;
        }
        self.counter = 0;
        let next = level + step;
        self.level = if !rate.decreasing {
            next.clamp(-0x8000, 0x7FFF)
        } else if rate.negative {
            next.clamp(-0x8000, 0)
        } else {
            next.clamp(0, 0x7FFF)
        } as i16;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rate(shift: u8, step: u8, decreasing: bool, exponential: bool) -> Rate {
        Rate { shift, step, decreasing, exponential, negative: false }
    }

    /// Samples until the level reaches `to` from `from`, at most `cap`.
    fn time(from: i16, to: i16, r: Rate, cap: usize) -> Option<usize> {
        let mut e = Envelope { level: from, counter: 0 };
        (1..=cap).find(|_| {
            e.tick(r);
            e.level == to
        })
    }

    #[test]
    fn a_fast_linear_attack_steps_every_sample_by_its_step_shifted() {
        let mut e = Envelope::default();
        e.tick(rate(0, 0, false, false));
        assert_eq!(e.level, 7 << 11);
        e.tick(rate(0, 0, false, false));
        assert_eq!(e.level, 14 << 11);
        let mut e = Envelope::default();
        e.tick(rate(11, 3, false, false));
        assert_eq!(e.level, 4, "shift 11 is a step of +4 each sample");
        assert_eq!(time(0, 0x7FFF, rate(0, 0, false, false), 10), Some(3), "and saturates at 7FFFh");
    }

    #[test]
    fn slow_shifts_wait_between_steps() {
        let r = rate(13, 0, false, false);
        let mut e = Envelope::default();
        let steps: Vec<i16> = (0..8).map(|_| {
            e.tick(r);
            e.level
        }).collect();
        assert_eq!(steps, [0, 0, 0, 7, 7, 7, 7, 14], "a step every four samples at shift 13");
        assert_eq!(time(0, 7, rate(26, 0, false, false), 40_000), Some(32_768), "shift 26 counts by one");
        assert_eq!(time(0, 7, rate(30, 0, false, false), 40_000), Some(32_768), "and so do 27 to 31, psx-spx's 76h behaving as 6Ah");
        assert_eq!(time(0, 7, rate(31, 3, false, false), 70_000), None, "rate 7Fh never steps");
    }

    #[test]
    fn the_exponential_increase_slows_above_6000h() {
        let mut e = Envelope { level: 0x6001, counter: 0 };
        e.tick(rate(4, 0, false, true));
        assert_eq!(e.level, 0x6001 + (7 << 7 >> 2));
        let mut e = Envelope { level: 0x6000, counter: 0 };
        e.tick(rate(4, 0, false, true));
        assert_eq!(e.level, 0x6000 + (7 << 7), "at 6000h itself the step is whole");
        assert_eq!(time(0x6001, 0x6001 + 7, rate(12, 0, false, true), 20), Some(8), "shift 12's counter quartered");
        assert_eq!(time(0x6001, 0x6001 + 7, rate(10, 0, false, true), 20), Some(2), "shift 10's step and counter halved");
    }

    #[test]
    fn the_exponential_decrease_scales_its_step_by_the_level_and_stops_at_zero() {
        let mut e = Envelope { level: 0x4000, counter: 0 };
        e.tick(rate(0, 0, true, true));
        assert_eq!(e.level as i32, 0x4000 + ((-8 << 11) * 0x4000 >> 15));
        assert_eq!(time(0x100, 0, rate(0, 0, true, false), 10), Some(1));
        let mut e = Envelope { level: 1, counter: 0 };
        e.tick(rate(0, 0, true, true));
        assert_eq!(e.level, 0, "a step that rounds to -1 still reaches zero");
    }

    #[test]
    fn an_inverted_phase_decreases_toward_minus_8000h_when_increasing() {
        let r = Rate { negative: true, ..rate(0, 0, false, false) };
        let mut e = Envelope { level: 100, counter: 0 };
        e.tick(r);
        assert_eq!(e.level, 100 - (8 << 11));
        assert_eq!(time(0, -0x8000, r, 10), Some(2));
        let d = Rate { negative: true, ..rate(0, 0, true, false) };
        let mut e = Envelope { level: -0x1000, counter: 0 };
        e.tick(d);
        assert_eq!(e.level, 0, "decreasing with the phase inverted rises to zero");
    }
}
