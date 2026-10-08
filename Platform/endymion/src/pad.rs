//! The generic pad's vocabulary and the rules between its controls and what a core takes: Galaxia's `PadButton`,
//! `PadAxis`, `PadControl` and `PadControls`. See EmuSen_Input.md §7.

use crate::clamp;

/// `PadButton`, in libretro's RetroPad order; its number is its value.
pub const BUTTONS: usize = 16;
pub const L2: u32 = 12;
pub const R2: u32 = 13;
/// `PadButton.R3`, the last button; a control above it is a stick direction.
pub const R3: u32 = 15;

/// `PadAxis`: LeftX, LeftY, RightX, RightY, LeftTrigger, RightTrigger.
pub const AXES: usize = 6;
pub const LEFT_X: u32 = 0;
pub const LEFT_Y: u32 = 1;
pub const RIGHT_X: u32 = 2;
pub const RIGHT_Y: u32 = 3;
pub const LEFT_TRIGGER: u32 = 4;
pub const RIGHT_TRIGGER: u32 = 5;

/// `PadControl`: the sixteen buttons under their own numbers, then up, down, left and right of each stick.
pub const CONTROLS: usize = 24;
pub const LEFT_STICK_UP: u32 = 16;
pub const LEFT_STICK_DOWN: u32 = 17;
pub const LEFT_STICK_LEFT: u32 = 18;
pub const LEFT_STICK_RIGHT: u32 = 19;
pub const RIGHT_STICK_UP: u32 = 20;
pub const RIGHT_STICK_DOWN: u32 = 21;
pub const RIGHT_STICK_LEFT: u32 = 22;
pub const RIGHT_STICK_RIGHT: u32 = 23;

/// `PadControls.IsButton`: the button a control is, when it is one.
pub fn as_button(control: u32) -> Option<u32> {
    (control <= R3).then_some(control)
}

/// `PadControls.Directions`: the two controls that push a stick's axis, negative first; none for a trigger.
pub fn directions(axis: u32) -> Option<(u32, u32)> {
    match axis {
        LEFT_X => Some((LEFT_STICK_LEFT, LEFT_STICK_RIGHT)),
        LEFT_Y => Some((LEFT_STICK_UP, LEFT_STICK_DOWN)),
        RIGHT_X => Some((RIGHT_STICK_LEFT, RIGHT_STICK_RIGHT)),
        RIGHT_Y => Some((RIGHT_STICK_UP, RIGHT_STICK_DOWN)),
        _ => None,
    }
}

/// `PadControls.TriggerButton`: the button whose press stands in for a trigger's full travel.
pub fn trigger_button(axis: u32) -> Option<u32> {
    match axis {
        LEFT_TRIGGER => Some(L2),
        RIGHT_TRIGGER => Some(R2),
        _ => None,
    }
}

/// `PadControls.For`: a console's bindable controls, its buttons and then up, down, left and right of each stick it reads.
pub fn controls_for(buttons: &[u32], axes: &[u32]) -> Vec<u32> {
    let mut controls: Vec<u32> = buttons.to_vec();
    for (x, y) in [(LEFT_X, LEFT_Y), (RIGHT_X, RIGHT_Y)] {
        if axes.contains(&y) {
            let (negative, positive) = directions(y).expect("a stick's axis has directions");
            controls.extend([negative, positive]);
        }
        if axes.contains(&x) {
            let (negative, positive) = directions(x).expect("a stick's axis has directions");
            controls.extend([negative, positive]);
        }
    }
    controls
}

/// `PadControls.Combine`: one direction held pushes the axis all the way; both held cancel, leaving the stick's own reading.
pub fn combine(analog: f64, negative: bool, positive: bool) -> f64 {
    if negative != positive {
        return if positive { 1.0 } else { -1.0 };
    }
    clamp(analog, -1.0, 1.0)
}

/// `PadControls.Resolve`: one axis from the pad's reading and the controls standing in for it.
pub fn resolve(axis: u32, analog: f64, held: impl Fn(u32) -> bool) -> f64 {
    if let Some((negative, positive)) = directions(axis) {
        return combine(analog, held(negative), held(positive));
    }
    if trigger_button(axis).is_some_and(&held) {
        return 1.0;
    }
    clamp(analog, 0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stick_takes_its_keys_and_a_trigger_its_button() {
        assert_eq!(resolve(LEFT_X, 0.3, |c| c == LEFT_STICK_RIGHT), 1.0);
        assert_eq!(resolve(LEFT_Y, 0.3, |c| c == LEFT_STICK_UP), -1.0);
        assert_eq!(resolve(LEFT_X, 0.3, |c| c == LEFT_STICK_LEFT || c == LEFT_STICK_RIGHT), 0.3);
        assert_eq!(resolve(RIGHT_X, 7.0, |_| false), 1.0);
        assert_eq!(resolve(LEFT_TRIGGER, 0.2, |c| c == L2), 1.0);
        assert_eq!(resolve(RIGHT_TRIGGER, -0.5, |_| false), 0.0);
        assert!(resolve(RIGHT_TRIGGER, f64::NAN, |_| false).is_nan());
        assert_eq!(resolve(9, 2.0, |_| true), 1.0);
        assert_eq!(combine(-0.0, false, false).to_bits(), (-0.0f64).to_bits());
    }

    #[test]
    fn a_consoles_controls_are_its_buttons_then_each_stick_vertical_first() {
        assert_eq!(controls_for(&[0, 8], &[LEFT_X, LEFT_Y, RIGHT_TRIGGER]), [0, 8, 16, 17, 18, 19]);
        assert_eq!(controls_for(&[], &[RIGHT_X]), [22, 23]);
        assert_eq!(controls_for(&[3], &[]), [3]);
        assert_eq!(as_button(15), Some(15));
        assert_eq!(as_button(16), None);
    }
}
