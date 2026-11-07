use super::Fault;
use super::tree::{Item, Kind};
use crate::input::WalkForm;
use crate::tokens::Colour;

/// No chrome (EmuSen_Qt.md §8.1): in the big-screen and card forms no menu bar or status bar is visible.
pub fn no_chrome(root: &Item, form: WalkForm) -> Vec<Fault> {
    let mut faults = Vec::new();
    if form == WalkForm::Desktop {
        return faults;
    }
    root.visit(&mut |item, _| {
        if matches!(item.kind, Kind::MenuBar | Kind::StatusBar) {
            faults.push(Fault::Chrome { item: item.id });
        }
    });
    faults
}

/// Names a reader hears (InterfaceSpec §4.3): every operable element has one.
pub fn named(root: &Item) -> Vec<Fault> {
    let mut faults = Vec::new();
    root.visit(&mut |item, _| {
        if item.operable && item.heard.as_deref().is_none_or(|s| s.trim().is_empty()) {
            faults.push(Fault::Unnamed { item: item.id });
        }
    });
    faults
}

/// Palette reach (InterfaceSpec §4.4): every colour an element paints is one the state declares; a theme's scene is exempt.
pub fn palette_reach(root: &Item, declared: &[Colour]) -> Vec<Fault> {
    let mut faults = Vec::new();
    root.visit(&mut |item, ancestors| {
        if item.scene || ancestors.iter().any(|a| a.scene) {
            return;
        }
        for colour in &item.paints {
            if !declared.contains(colour) {
                faults.push(Fault::OutsidePalette { item: item.id, colour: *colour });
            }
        }
    });
    faults
}
