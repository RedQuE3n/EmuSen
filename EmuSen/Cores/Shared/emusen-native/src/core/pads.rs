//! The pad tables the cores share: the PlayStation family's sixteen-bit report and its sticks, and the rows-to-buttons step.

use super::{Axis, AxisControl, Button, Control};

/// A pad bit, its canonical control and its label.
pub type Row = (u32, Control, &'static str);

/// The PlayStation pad's bits (psx-spx "Standard Controllers"), pressed as 1, in the order a bindings screen lists
/// them; L3 and R3 are the DualShock's alone. Diamond and Sapphire both report it.
pub const PLAYSTATION: [Row; 16] = [
    (4, Control::Up, "Up"),
    (6, Control::Down, "Down"),
    (7, Control::Left, "Left"),
    (5, Control::Right, "Right"),
    (12, Control::X, "Triangle"),
    (13, Control::A, "Circle"),
    (14, Control::B, "Cross"),
    (15, Control::Y, "Square"),
    (10, Control::L, "L1"),
    (11, Control::R, "R1"),
    (8, Control::L2, "L2"),
    (9, Control::R2, "R2"),
    (0, Control::Select, "Select"),
    (3, Control::Start, "Start"),
    (1, Control::L3, "L3"),
    (2, Control::R3, "R3"),
];

/// The buttons a table's rows describe.
pub fn buttons(rows: &[Row]) -> Vec<Button> {
    rows.iter().map(|&(bit, c, l)| Button { bit, control: Some(c), label: l.into() }).collect()
}

/// The DualShock's two sticks as axes 0 to 3: left X and Y, right X and Y.
pub fn playstation_sticks() -> Vec<AxisControl> {
    let stick = |axis, control, label: &str| AxisControl { axis, control: Some(control), trigger: false, label: label.into() };
    vec![stick(0, Axis::LeftX, "Left Stick"), stick(1, Axis::LeftY, "Left Stick"), stick(2, Axis::RightX, "Right Stick"), stick(3, Axis::RightY, "Right Stick")]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_playstation_report_keeps_its_bits_and_controls() {
        let text: Vec<String> = PLAYSTATION.iter().map(|&(bit, c, l)| format!("{bit}:{c:?}:{l}")).collect();
        assert_eq!(
            text.join(" "),
            "4:Up:Up 6:Down:Down 7:Left:Left 5:Right:Right 12:X:Triangle 13:A:Circle 14:B:Cross 15:Y:Square 10:L:L1 11:R:R1 8:L2:L2 9:R2:R2 0:Select:Select 3:Start:Start 1:L3:L3 2:R3:R3"
        );
        let mut bits: Vec<u32> = PLAYSTATION.iter().map(|r| r.0).collect();
        bits.sort_unstable();
        assert_eq!(bits, (0..16).collect::<Vec<_>>());
        assert_eq!(buttons(&PLAYSTATION[..14]).len(), 14);
        assert_eq!(playstation_sticks().iter().map(|a| a.axis).collect::<Vec<_>>(), [0, 1, 2, 3]);
    }
}
