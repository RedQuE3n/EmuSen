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

/// Palette reach (InterfaceSpec §4.4): every colour an element paints is one the state declares, at any alpha, since a
/// token painted through a colour's alpha is the token as it is through an item's opacity (the night help band's glass,
/// EmuSen_Kit.md §7.25); a theme's scene is exempt.
pub fn palette_reach(root: &Item, declared: &[Colour]) -> Vec<Fault> {
    let mut faults = Vec::new();
    root.visit(&mut |item, ancestors| {
        if item.scene || ancestors.iter().any(|a| a.scene) {
            return;
        }
        for colour in &item.paints {
            if !declared.iter().any(|d| (d.r, d.g, d.b) == (colour.r, colour.g, colour.b) && (d.a == colour.a || d.a == 255)) {
                faults.push(Fault::OutsidePalette { item: item.id, colour: *colour });
            }
        }
    });
    faults
}

/// The share of what is behind words that may fall under the bar (a star, a rule's edge); the rest must pass.
pub const CONTRAST_SPARE: f64 = 0.1;
/// Large type in WCAG 2.1's sense: 18 points, or 14 points bold, as pixels.
pub const LARGE_TEXT: f64 = 24.0;
pub const LARGE_BOLD_TEXT: f64 = 18.66;

/// Contrast (WCAG 2.1, 1.4.3): words drawn over what is behind them reach 4.5:1, or 3:1 for large type, over all but the spare share.
pub fn contrast(root: &Item) -> Vec<Fault> {
    let mut faults = Vec::new();
    root.visit(&mut |item, _| {
        let Some(words) = &item.legibility else { return };
        if words.behind.is_empty() || words.ink.a == 0 {
            return;
        }
        let mut ratios: Vec<f64> = words
            .behind
            .iter()
            .map(|back| {
                let back = Colour::rgb(back.r, back.g, back.b);
                words.ink.over(back).contrast(back)
            })
            .collect();
        ratios.sort_by(f64::total_cmp);
        let ratio = ratios[((ratios.len() as f64) * CONTRAST_SPARE).floor() as usize];
        let large = words.size >= LARGE_TEXT || (words.bold && words.size >= LARGE_BOLD_TEXT);
        let needed = if large { 3.0 } else { 4.5 };
        if ratio < needed {
            faults.push(Fault::LowContrast { item: item.id, ratio, needed });
        }
    });
    faults
}
