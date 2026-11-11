//! The help element's entries per view in ES-DE 3.4.1's order and words (EmuSen_Qt.md §4.8.5): which actions a view
//! offers, the button each sits on, and the theme's icon for it.

use crate::loader::{Element, Pair};

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

// ES-DE 3.4.1's entries in its order and words, as its captures show them (EmuSen_Qt.md §4.8.5): the system view's.
fn system() -> Vec<Prompt> {
    vec![
        p("start", "Menu", PadButton::Start, "button_start_{1}"),
        p("a", "Select", PadButton::South, "button_a_{0}"),
        p("left/right", "Choose", PadButton::DpadLeftRight, "dpad_leftright"),
    ]
}

// The gamelist's; X opens the media viewer (viewer.rs), on a folder too, where it opens nothing.
fn gamelist() -> Vec<Prompt> {
    vec![
        p("back", "Options", PadButton::Select, "button_back_{1}"),
        p("start", "Menu", PadButton::Start, "button_start_{1}"),
        p("a", "Select", PadButton::South, "button_a_{0}"),
        p("b", "Back", PadButton::East, "button_b_{0}"),
        p("x", "View media", PadButton::West, "button_x_{0}"),
        p("y", "Favorites", PadButton::North, "button_y_{0}"),
    ]
}

// The quick system select's pair in this gamelist: the two "or" settings give left and right to a list or a vertical
// carousel and the shoulders or triggers to a grid or a horizontal carousel (UG "Quick system select").
fn quick_select(data: &Data) -> Option<Prompt> {
    let sideways = data.system().theme.view("gamelist").primary().is_some_and(|p| match p.kind() {
        "grid" => true,
        "carousel" => !p.string("type").is_some_and(|t| t.starts_with("vertical")),
        _ => false,
    });
    let pair = match (data.help.quick_select.as_str(), sideways) {
        ("leftright", _) | ("leftrightshoulders" | "leftrighttriggers", false) => ("left/right", PadButton::DpadLeftRight, "dpad_leftright"),
        ("shoulders", _) | ("leftrightshoulders", true) => ("lr", PadButton::LeftShoulder, "button_lr"),
        ("triggers", _) | ("leftrighttriggers", true) => ("ltrt", PadButton::LeftTrigger, "button_ltrt"),
        _ => return None,
    };
    Some(p(pair.0, "System", pair.1, pair.2))
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
pub fn entries(view: &str, e: &Element, data: &Data) -> Vec<HelpEntry> {
    let c = &data.help;
    let mut known = if view == "system" { system() } else { gamelist() };
    if view == "gamelist" {
        for p in known.iter_mut().filter(|p| p.entry == "y" && c.editing) {
            p.label = "Collection";
        }
        known.extend(quick_select(data));
    }
    if view == "system" && c.screensaver {
        known.insert(2, p("x", "Screensaver", PadButton::West, "button_x_{0}"));
    }
    if if view == "system" { c.random_systems } else { c.random_games } {
        known.push(p("thumbstickclick", "Random", PadButton::StickClick, "thumbstick_click"));
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

// The strip's numbers as ES-DE 3.4.1 takes them from a theme, in f32 as it computes them (EmuSen_Qt.md §4.12.4): es-core
// HelpComponent.cpp (the constructor, applyTheme, updateGrid), Font.h getSmallFont, Font.cpp getFromTheme and Font::Font.

/// The font's size in pixels: a theme's fontSize × the screen's height, clamped to 0.001–1.5 heights; without one, the
/// default share (the loader's, 0.035) × the shorter side; then clamped to 3 px – 1.5 heights, as the font is made.
pub fn font_px(e: &Element, w: f64, h: f64) -> f64 {
    let (w, h) = (w as f32, h as f32);
    let share = e.float("fontSize").unwrap_or(0.035);
    let size = if e.explicit.contains_key("fontSize") { (h * share).clamp(h * 0.001, h * 1.5) } else { share * w.min(h) };
    f64::from(size.clamp(3.0, h * 1.5))
}

/// A spacing in pixels: the theme's share clamped to 0–0.04, or the default, times the screen's width.
pub fn spacing_px(e: &Element, property: &str, default: f32, w: f64) -> f64 {
    let share = e.float(property).map_or(default, |s| s.clamp(0.0, 0.04));
    f64::from(share * w as f32)
}

/// entryRelativeScale, clamped to 0.2–3.
pub fn scale(e: &Element) -> f64 {
    f64::from(e.float("entryRelativeScale").map_or(1.0, |s| s.clamp(0.2, 3.0)))
}

/// The strip's drawn top left from its grid's size (the entries' sum with its trailing spacing, and the row's height):
/// the place pos × the screen, or ES-DE's default (0.012, 0.9515) of it, with x moved by the spacing × origin; that place
/// rounded, plus −origin × size rounded on its own.
pub fn top_left(pos: Option<Pair>, origin: Pair, entry_spacing: f64, size: (f64, f64), w: f64, h: f64) -> (f64, f64) {
    let (w, h) = (w as f32, h as f32);
    let (px, py) = pos.map_or((w * 0.012, h * 0.9515), |p| (p.x * w, p.y * h));
    let x = (px + entry_spacing as f32 * origin.x).round() + (-origin.x * size.0 as f32).round();
    let y = py.round() + (-origin.y * size.1 as f32).round();
    (f64::from(x), f64::from(y))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pair(x: f32, y: f32) -> Pair {
        Pair { x, y }
    }

    // Art Book Next's system strip at 1280 × 800: 584.33 wide with its trailing 20 px, 33.75 high, pos 0.5 0.9375, origin 0.5 1.
    #[test]
    fn the_strip_is_placed_by_two_roundings_and_the_spacing_s_share_of_the_origin() {
        assert_eq!(top_left(Some(pair(0.5, 0.9375)), pair(0.5, 1.0), 20.0, (584.33, 33.75), 1280.0, 800.0), (358.0, 716.0));
        // Artflix's SELECT at 20 px: 74.34 seen, 10.6624 trailing, origin 0.5 0 at 0.5 0.96.
        assert_eq!(top_left(Some(pair(0.5, 0.96)), pair(0.5, 0.0), 10.6624, (85.0024, 17.5), 1280.0, 800.0), (602.0, 768.0));
        assert_eq!(top_left(None, pair(0.0, 0.0), 10.0, (100.0, 20.0), 1280.0, 800.0), (15.0, 761.0));
    }
}
