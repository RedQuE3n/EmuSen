//! The help element's entries per view in the project's own words, translated from the C# HelpPrompts (EmuSen_BigPicture.md
//! §3.6, §15, §22, §29, §37): which actions a view offers, the button each sits on, and the theme's icon for it.

use crate::loader::Element;

use super::data::{ControllerType, Data};
use super::frame::{HelpEntry, PadButton};
use super::paint::existing;
use super::words;

// An ES-DE help entry: its THEMES.md name, the action it names here, its button, and its customButtonIcon key with {0} for the face suffix, {1} the middle.
#[derive(Clone)]
struct Prompt {
    entry: &'static str,
    label: &'static str,
    button: PadButton,
    key: String,
}

fn p(entry: &'static str, label: &'static str, button: PadButton, key: &str) -> Prompt {
    Prompt { entry, label, button, key: key.to_owned() }
}

fn system() -> Vec<Prompt> {
    vec![
        p("left/right", "System", PadButton::DpadLeftRight, "dpad_leftright"),
        p("a", "Select", PadButton::South, "button_a_{0}"),
        p("start", "Menu", PadButton::Start, "button_start_{1}"),
    ]
}

fn gamelist() -> Vec<Prompt> {
    vec![
        p("up/down", "Choose", PadButton::DpadUpDown, "dpad_updown"),
        p("left/right", "System", PadButton::DpadLeftRight, "dpad_leftright"),
        p("l", "Jump", PadButton::LeftShoulder, "button_l"),
        p("r", "Jump", PadButton::RightShoulder, "button_r"),
        p("lt", "First", PadButton::LeftTrigger, "button_lt"),
        p("rt", "Last", PadButton::RightTrigger, "button_rt"),
        p("a", "Launch", PadButton::South, "button_a_{0}"),
        p("b", "Back", PadButton::East, "button_b_{0}"),
        p("y", "Favorite", PadButton::North, "button_y_{0}"),
        p("back", "Options", PadButton::Select, "button_back_{1}"),
        p("start", "Menu", PadButton::Start, "button_start_{1}"),
    ]
}

// ES-DE's suffixes for a controller type's face and middle buttons.
fn suffixes(t: ControllerType) -> (&'static str, &'static str) {
    match t {
        ControllerType::PlayStation => ("PS", "PS4"),
        ControllerType::Switch => ("switch", "switch"),
        ControllerType::Xbox => ("XBOX", "XBOX"),
    }
}

// With the swap on, a function's prompt names the face button it moved to, and that button's icon.
fn swapped(mut p: Prompt) -> Prompt {
    let (button, from, to) = match p.button {
        PadButton::South => (PadButton::East, "button_a_", "button_b_"),
        PadButton::East => (PadButton::South, "button_b_", "button_a_"),
        PadButton::North => (PadButton::West, "button_y_", "button_x_"),
        PadButton::West => (PadButton::North, "button_x_", "button_y_"),
        _ => return p,
    };
    p.button = button;
    p.key = p.key.replace(from, to);
    p
}

/// The entries a theme's help element lists, in its order, that the project has an action for in this view; "all" is every one.
pub fn entries(view: &str, e: &Element, data: &Data, folder: bool) -> Vec<HelpEntry> {
    let c = &data.help;
    let mut known = if view == "system" { system() } else { gamelist() };
    if view == "gamelist" {
        for p in &mut known {
            if p.entry == "y" && c.editing {
                p.label = "Collection";
            }
            if p.entry == "a" && folder {
                p.label = "Select";
            }
            if matches!((p.entry, c.quick_select.as_str()), ("l" | "r", "shoulders") | ("lt" | "rt", "triggers")) {
                p.label = "System";
            }
        }
        known.retain(|p| p.entry != "left/right" || c.quick_select == "leftright");
    }
    let last = known.len() - 1;
    if if view == "system" { c.random_systems } else { c.random_games } {
        known.insert(last, p("thumbstickclick", "Random", PadButton::StickClick, "thumbstick_click"));
    }
    if view == "system" && c.screensaver {
        let last = known.len() - 1;
        known.insert(last, p("x", "Screensaver", PadButton::West, "button_x_{0}"));
    }
    let named = e.list("entries");
    let chosen: Vec<Prompt> = if named.is_empty() || named.iter().any(|n| n == "all") {
        known
    } else {
        named.iter().filter_map(|n| known.iter().find(|p| p.entry == n).cloned()).collect()
    };
    let (face, middle) = suffixes(data.controller_type);
    let icons = e.keyed("customButtonIcon");
    let case = e.string("letterCase").or(Some("uppercase"));
    chosen
        .into_iter()
        .map(|p| if data.swap { swapped(p) } else { p })
        .map(|p| {
            let key = p.key.replace("{0}", face).replace("{1}", middle);
            HelpEntry {
                label: words::cased(p.label, case),
                pair: matches!(p.button, PadButton::DpadUpDown | PadButton::DpadLeftRight),
                button: p.button,
                icon: icons.and_then(|m| existing(m.get(&key))),
            }
        })
        .collect()
}
