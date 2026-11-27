//! Details ▸ Cover Art walked by the pad, as page_cover.rs and page_cover_none.rs do (EmuSen_Qt.md §7.8.5): presses let
//! go as a hand does, the section stepped, Cover Art opened, its cards read, and the page judged at the gate's sizes.
#![allow(dead_code)]

use super::wait_for;
use emusen_kit_core::input::{UiButton, WalkForm};
use emusen_kit_core::tokens::Variant;
use emusen_qt_test::harness::SIZES;
use emusen_qt_test::Scene;
use std::collections::HashSet;

pub fn until(scene: &mut Scene<'_>, what: &str, ms: i32, mut done: impl FnMut(&mut Scene<'_>) -> bool) {
    let mut waited = 0;
    while !done(scene) && waited < ms {
        scene.wait(25);
        waited += 25;
    }
    assert!(done(scene), "{what} within {ms} ms; {}", super::state(scene));
}

pub fn heard(scene: &mut Scene<'_>) -> String {
    let tree = scene.tree();
    scene.focus(&tree).map(|i| tree.nodes[i as usize].heard.clone()).unwrap_or_default()
}

pub fn move_to(scene: &mut Scene<'_>, button: UiButton, words: &str) {
    for _ in 0..12 {
        if heard(scene) == words {
            return;
        }
        press(scene, button);
        scene.wait(80);
    }
    assert_eq!(heard(scene), words, "{button:?} reaches {words}");
}

pub fn reads(scene: &mut Scene<'_>, words: &str) -> bool {
    scene.tree().nodes.iter().any(|n| n.visible && n.text.as_deref() == Some(words))
}

pub fn section(scene: &mut Scene<'_>, index: i32) {
    for _ in 0..5 {
        let now = scene.property("pageModel", "section");
        if now == index.to_string() {
            break;
        }
        press(scene, UiButton::PageDown);
        until(scene, "the section steps", 1000, |s| s.property("pageModel", "section") != now);
    }
    assert_eq!(scene.property("pageModel", "section"), index.to_string());
    scene.wait(200);
}

pub fn open_page(scene: &mut Scene<'_>) {
    press(scene, UiButton::Screensaver);
    wait_for(scene, "screens", "shown", "Page", 2000);
    wait_for(scene, "pageModel", "ready", "true", 10000);
    scene.wait(300);
}

/// A press let go before the next, as a hand does: the router has seen the release.
pub fn press(scene: &mut Scene<'_>, button: UiButton) {
    scene.pad(button);
    until(scene, "the press let go", 1000, |s| !s.pad_held(button));
}

/// Details ▸ Cover art, its cards shown.
pub fn open_cover_art(scene: &mut Scene<'_>) {
    section(scene, 4);
    move_to(scene, UiButton::Down, "Cover Art");
    press(scene, UiButton::Accept);
    wait_for(scene, "pageModel", "coverArt", "true", 2000);
    until(scene, "Cover art's cards", 2000, |s| s.property("coverCards", "activeFocus") == "true" && s.property("coverCards", "count") != "0");
}

/// The words of each card shown, and of the one marked in use.
pub fn cards(scene: &mut Scene<'_>) -> (Vec<String>, Option<String>) {
    let tree = scene.tree();
    let words_of = |card: &str| tree.nodes.iter().find(|n| n.parent == card && n.name == "coverCardWords").and_then(|n| n.text.clone()).unwrap_or_default();
    let shown: Vec<&emusen_qt_test::tree::Node> = tree.nodes.iter().filter(|n| n.visible && n.name == "coverCard").collect();
    let words = shown.iter().map(|n| words_of(&n.parent)).collect();
    let in_use = tree.nodes.iter().find(|n| n.visible && n.name == "coverInUse").map(|n| words_of(&n.parent));
    (words, in_use)
}

/// At each of the gate's window sizes, kit-core's rules find nothing on Cover art (nothing cut off, nothing overlapping,
/// every place reached), and the pad's walk from the focused card reaches every card.
pub fn judged_at_every_size(scene: &mut Scene<'_>) {
    for (w, h) in SIZES {
        scene.resize(w, h);
        scene.wait(150);
        let tree = scene.tree();
        let focus = scene.focus(&tree);
        let edges = scene.record_edges(&tree);
        let faults = scene.audit(&tree, WalkForm::BigScreen, Variant::Dark, focus, &edges, f64::from(w), f64::from(h));
        assert!(faults.is_empty(), "Cover art at {w}x{h}: {faults:?}\n{}", tree.legend());
        let cards: HashSet<u32> = tree.nodes.iter().enumerate().filter(|(_, n)| n.visible && n.name == "coverCard").map(|(i, _)| tree.walkable_of(i as u32)).collect();
        let mut reached: HashSet<u32> = focus.into_iter().collect();
        let mut grew = true;
        while grew {
            grew = false;
            for e in &edges {
                if reached.contains(&e.from.0) && reached.insert(e.to.0) {
                    grew = true;
                }
            }
        }
        assert!(cards.is_subset(&reached), "the pad reaches every card at {w}x{h}");
        assert!(!cards.is_empty());
    }
    let (w, h) = SIZES[0];
    scene.resize(w, h);
    scene.wait(150);
}

