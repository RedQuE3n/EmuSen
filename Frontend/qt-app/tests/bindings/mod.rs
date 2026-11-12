//! What the controllers' tests share (EmuSen_Qt.md §7.7.H): Settings opened over a scripted world with the virtual pad
//! read, and the drawn words read from the tree.
#![allow(dead_code)]

use crate::common::wait_for;
use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_qt_test::Scene;

/// The gate's findings Settings names for the tester (`settings.rs`): the help bar's words under the floor at 1280 × 720,
/// and only those; a label of the drawing's, or any other word, stays a finding at every size.
pub fn named(fault: &str, tree: &emusen_qt_test::tree::Tree, size: (i32, i32)) -> bool {
    let items: Vec<usize> = fault.rsplit(": ").next().unwrap_or("").split(|c: char| !c.is_ascii_digit()).filter_map(|n| n.parse().ok()).collect();
    let in_help_bar = |i: &usize| {
        let mut at = tree.nodes.get(*i);
        while let Some(n) = at {
            if n.name == "helpBar" {
                return true;
            }
            at = tree.nodes.iter().find(|p| p.id == n.parent);
        }
        false
    };
    size == (1280, 720) && fault.contains("too small to read") && !items.is_empty() && items.iter().all(in_help_bar)
}

pub fn sentence(scene: &mut Scene<'_>) -> String {
    scene.property("settingsSentence", "text")
}

/// The visible words of the items named `name` under the item named `under`, top to bottom.
pub fn words_under(scene: &mut Scene<'_>, under: &str, name: &str) -> Vec<String> {
    let tree = scene.tree();
    let Some(top) = tree.nodes.iter().find(|n| n.name == under).map(|n| n.id.clone()) else { return Vec::new() };
    let inside = |id: &str| {
        let mut at = id.to_owned();
        while let Some(n) = tree.nodes.iter().find(|n| n.id == at) {
            if n.id == top {
                return true;
            }
            at = n.parent.clone();
        }
        false
    };
    let mut found: Vec<(f64, f64, String)> = tree
        .nodes
        .iter()
        .filter(|n| n.visible && (name.is_empty() || n.name == name) && inside(&n.id))
        .filter_map(|n| n.text.clone().filter(|t| !t.is_empty()).map(|t| (n.y, n.x, t)))
        .collect();
    found.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));
    found.into_iter().map(|(_, _, t)| t).collect()
}

/// The pane's lines under its sentence, as drawn.
pub fn lines(scene: &mut Scene<'_>) -> String {
    words_under(scene, "settingsPaneLines", "SettingsLine").join(" | ")
}

/// The bindings page's strip, as drawn.
pub fn tabs(scene: &mut Scene<'_>) -> String {
    words_under(scene, "SettingsContentTabs", "").join(", ")
}

/// Settings opened on Controllers from Home's place, the virtual pad read and seated.
pub fn open_settings(scene: &mut Scene<'_>, world: &str, pad: u32) {
    open_settings_reading(scene, world, &pad.to_string());
}

/// Settings opened so, the pads of these SDL ids read (one connected at the start, the rest to come).
pub fn open_settings_reading(scene: &mut Scene<'_>, world: &str, pads: &str) {
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("settingsModel", "world", world));
    assert!(scene.set_property("program", "onlyPads", pads));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(scene, "library", "loading", "false", 10000);
    wait_for(scene, "settingsModel", "ready", "true", 5000);
    wait_for(scene, "pads", "connected", "1", 5000);
    for _ in 0..5 {
        scene.pad(UiButton::Right);
    }
    scene.pad(UiButton::Accept);
    wait_for(scene, "screens", "shown", "Settings", 2000);
}

