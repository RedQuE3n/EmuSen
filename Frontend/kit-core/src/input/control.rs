//! A pad's controls by where they are, and which control gives each of the interface's presses (EmuSen_Kit.md §7.12).

use super::button::UiButton;

/// A pad's control by its place, as SDL names them: the face buttons by position (South is the bottom one), the d-pad's
/// four, the shoulders and triggers, the two middle buttons, the sticks' clicks and the guide button.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PadControl {
    South,
    East,
    West,
    North,
    Up,
    Down,
    Left,
    Right,
    LeftShoulder,
    RightShoulder,
    LeftTrigger,
    RightTrigger,
    Start,
    Select,
    LeftStick,
    RightStick,
    Guide,
}

impl PadControl {
    pub const ALL: [PadControl; 17] = [
        PadControl::South,
        PadControl::East,
        PadControl::West,
        PadControl::North,
        PadControl::Up,
        PadControl::Down,
        PadControl::Left,
        PadControl::Right,
        PadControl::LeftShoulder,
        PadControl::RightShoulder,
        PadControl::LeftTrigger,
        PadControl::RightTrigger,
        PadControl::Start,
        PadControl::Select,
        PadControl::LeftStick,
        PadControl::RightStick,
        PadControl::Guide,
    ];

    pub const fn index(self) -> usize {
        self as usize
    }

    pub const fn is_face(self) -> bool {
        matches!(self, PadControl::South | PadControl::East | PadControl::West | PadControl::North)
    }
}

impl UiButton {
    /// The control that gives this press, with the player's swap applied as `swapped` applies it: Accept on South and
    /// Back on East, Search on North and Screensaver on West, each pair traded by the swap; the C# front end's table.
    pub const fn control(self, swap: bool) -> PadControl {
        match self.swapped(swap) {
            UiButton::Up => PadControl::Up,
            UiButton::Down => PadControl::Down,
            UiButton::Left => PadControl::Left,
            UiButton::Right => PadControl::Right,
            UiButton::Accept => PadControl::South,
            UiButton::Back => PadControl::East,
            UiButton::Menu => PadControl::Start,
            UiButton::Options => PadControl::Select,
            UiButton::PageUp => PadControl::LeftShoulder,
            UiButton::PageDown => PadControl::RightShoulder,
            UiButton::First => PadControl::LeftTrigger,
            UiButton::Last => PadControl::RightTrigger,
            UiButton::Guide => PadControl::Guide,
            UiButton::Search => PadControl::North,
            UiButton::Random => PadControl::LeftStick,
            UiButton::Screensaver => PadControl::West,
        }
    }
}
