//! A slider row's range and its step by the pad (EmuSen_Kit.md §7.38): Left and Right move the amount one step, held
//! within the range and on the grid of steps from its minimum, so a value read back is the one a drag snaps to.

/// The steps a pad takes across a range whose step is not given: a hundredth of it.
pub const STEPS_WITHOUT_A_STEP: f64 = 100.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SliderRange {
    pub minimum: f64,
    pub maximum: f64,
    /// 0 for none: the pad then takes a hundredth of the range.
    pub step: f64,
}

impl Default for SliderRange {
    fn default() -> Self {
        SliderRange { minimum: 0.0, maximum: 1.0, step: 0.0 }
    }
}

impl SliderRange {
    /// The pad's step.
    pub fn pad_step(&self) -> f64 {
        if self.step > 0.0 { self.step } else { (self.maximum - self.minimum).abs() / STEPS_WITHOUT_A_STEP }
    }

    /// The amount `by` steps from `amount`, on the grid and inside the range.
    pub fn stepped(&self, amount: f64, by: i32) -> f64 {
        let (low, high) = if self.minimum <= self.maximum { (self.minimum, self.maximum) } else { (self.maximum, self.minimum) };
        let step = self.pad_step();
        if step <= 0.0 || !amount.is_finite() {
            return amount.clamp(low, high);
        }
        let at = ((amount - low) / step).round() + f64::from(by);
        let last = ((high - low) / step).floor();
        let snapped = low + at.clamp(0.0, last) * step;
        snapped.clamp(low, high)
    }
}
