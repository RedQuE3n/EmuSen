//! The controllers a console's engine declares, as Settings ▸ Controllers draws and binds them (EmuSen_Qt.md §7.7.H, its
//! design note on drawings as data): the drawing that stands for a controller id, a data file of `data/controllers` named by the id;
//! and the controls the drawing's regions are named by, the canonical controls by the core ABI's own names and each
//! stick the console reads by its axes' names. A controller with no file is drawn as the kit's generic controller.

use emusen_crystal_client::shapes::Controllers;
use emusen_native::core::desc::{Axis, Control};

include!(concat!(env!("OUT_DIR"), "/drawings.rs"));

/// A controller's drawing file by its declared id, following a file that is the same drawing as another's
/// (`{"same_as": "<id>"}`); none for an id with no file.
pub fn drawing(id: &str) -> Option<&'static str> {
    let text = DRAWINGS.iter().find(|(d, _)| *d == id).map(|(_, t)| *t)?;
    let alias = serde_json::from_str::<serde_json::Value>(text).ok().and_then(|v| v["same_as"].as_str().map(str::to_owned));
    match alias {
        Some(other) => DRAWINGS.iter().find(|(d, _)| *d == other).map(|(_, t)| *t),
        None => Some(text),
    }
}

/// The drawing of the controller a console's player holds: its controller's own file, else the file of that controller on
/// one of the console's systems (`<system>.<controller>`, as a hosted core's RetroPad on the Saturn is `saturn.retropad`,
/// the tester's R-20261111-113); none, the kit's generic drawing, where neither exists.
pub fn drawing_on(console: &crate::consoles::Console, player: u32) -> Option<&'static str> {
    let id = controller_of(&console.controllers, player)?;
    drawing(id).or_else(|| console.systems.iter().find_map(|system| drawing(&format!("{system}.{id}"))))
}

/// A canonical control's name by its number in the core ABI (`enum emusen_control`).
pub fn control_name(number: u32) -> Option<&'static str> {
    Control::NAMES.get(number as usize).copied()
}

fn axis_name(number: u32) -> Option<&'static str> {
    Axis::NAMES.get(number as usize).copied()
}

/// The controls a console's pad has, as the drawing's regions are named: its buttons, then up, down, left and right of
/// each stick it reads (`<axis>-` the way to the axis's low end), as Endymion's `PadControls.For` orders them.
pub fn controls(c: &Controllers) -> Vec<String> {
    let mut out: Vec<String> = c.buttons.iter().filter_map(|b| control_name(*b)).map(str::to_owned).collect();
    for (x, y) in [("LeftX", "LeftY"), ("RightX", "RightY")] {
        for axis in [y, x] {
            if c.axes.iter().filter_map(|a| axis_name(*a)).any(|a| a == axis) {
                out.push(format!("{axis}-"));
                out.push(format!("{axis}+"));
            }
        }
    }
    out
}

/// A canonical control's number by its name.
pub fn control_number(name: &str) -> Option<u32> {
    Control::NAMES.iter().position(|n| *n == name).map(|i| i as u32)
}

/// How far a stick's direction (`<axis>-` or `<axis>+`) is pushed on a pad's axes, 0 to 1, by the core ABI's axis
/// numbers; 0 for a control that is not a stick's direction.
pub fn stick_push(control: &str, axes: &[f64]) -> f64 {
    let Some((axis, toward)) = stick_way(control) else { return 0.0 };
    (f64::from(toward) * axes.get(axis as usize).copied().unwrap_or(0.0)).clamp(0.0, 1.0)
}

/// Whether a stick's direction is held: pushed past `threshold` of its travel, the stick deadzone past which Endymion
/// hears a stick as a direction.
pub fn stick_held(control: &str, axes: &[f64], threshold: f64) -> bool {
    stick_push(control, axes) > threshold
}

/// A stick's direction (`<axis>-` or `<axis>+`) as the core's axis number and its way, -1 toward the low end.
pub fn stick_way(control: &str) -> Option<(u32, i32)> {
    let (axis, toward) = control.split_at(control.len().saturating_sub(1));
    let at = Axis::NAMES.iter().position(|n| *n == axis)? as u32;
    match toward {
        "-" => Some((at, -1)),
        "+" => Some((at, 1)),
        _ => None,
    }
}

/// Whether a control is a button, bound to a gamepad's button; a stick's direction is read from the pad's own stick.
pub fn is_button(control: &str) -> bool {
    Control::NAMES.contains(&control)
}

/// The controller a player's port holds, as the console's settings stand: the player's, else the first port's.
pub fn controller_of(c: &Controllers, player: u32) -> Option<&str> {
    let at = player.saturating_sub(1) as usize;
    c.players.get(at).and_then(|p| p.as_deref()).or_else(|| c.players.first().and_then(|p| p.as_deref()))
}

/// How many players the console's ports hold as its settings stand, an adapter's included; one at least.
pub fn players(c: &Controllers) -> u32 {
    (c.players.iter().filter(|p| p.is_some()).count() as u32).max(1)
}
