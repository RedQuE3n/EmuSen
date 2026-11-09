// EmuSen_Kit.md §7.39: a walkable inside a ListView or GridView becomes the view's current item when it takes the focus,
// so the pad moves on from the cell the pointer focused, whether the walkable is the delegate or stands inside it; and
// what a view turns under a pointer that stays where it is takes no focus.

use emusen_kit_core::input::UiButton;
use emusen_qt_test::harness::Landing;
use emusen_qt_test::{Harness, Scene};

fn centre(scene: &mut Scene<'_>, name: &str) -> (f64, f64) {
    let tree = scene.tree();
    let index = tree.find_named(name).unwrap_or_else(|| panic!("{name} in the scene"));
    let n = &tree.nodes[index as usize];
    (n.x + n.w / 2.0, n.y + n.h / 2.0)
}

fn focused(scene: &mut Scene<'_>) -> String {
    let tree = scene.tree();
    scene.focus(&tree).map(|i| tree.nodes[i as usize].name.clone()).unwrap_or_default()
}

/// Hovers `hover`, waits out the strip's turn, presses, waits again, and names where the focus is.
fn hover_then(scene: &mut Scene<'_>, hover: &str, press: UiButton) -> String {
    let (x, y) = centre(scene, hover);
    scene.hover(x, y);
    scene.wait(400);
    assert_eq!(focused(scene), hover, "hovering {hover} focused it, and it kept the focus");
    scene.pad(press);
    scene.wait(400);
    focused(scene)
}

#[test]
fn qt_7_39_a_walkable_focused_by_the_pointer_becomes_its_view_s_current_item() {
    let mut harness = Harness::new();
    let mut scene = harness.load("qrc:/qt/qml/EmuSen/Scenes/qml/GridScene.qml", 1280, 800);
    assert_eq!(focused(&mut scene), "cell.0", "the grid's first cell holds the focus");
    assert_eq!(hover_then(&mut scene, "cell.7", UiButton::Right), "cell.8", "Right from a hovered cell lands beside it");
    assert_eq!(scene.property("grid", "currentIndex"), "8");
    assert_eq!(hover_then(&mut scene, "cell.3", UiButton::Down), "cell.8", "Down from a hovered cell lands under it");
    assert_eq!(hover_then(&mut scene, "cell.9", UiButton::Left), "cell.8", "Left from the last cell lands beside it");
    assert_eq!(hover_then(&mut scene, "item.4", UiButton::Right), "item.5", "a walkable inside a strip's delegate: Right lands beside it");
    assert_eq!(scene.property("strip", "currentIndex"), "5");
    assert_ne!(scene.property("strip", "contentX"), "0", "the strip turned to its current item, under the pointer");
    let at = centre(&mut scene, "item.3");
    assert_eq!(hover_then(&mut scene, "item.3", UiButton::Left), "item.2", "and Left, after the strip turned");
    assert_ne!(centre(&mut scene, "item.2"), at, "the strip turned under the pointer resting where item 3 was");
    for _ in 0..6 {
        scene.pad(UiButton::Right);
    }
    scene.wait(400);
    assert_eq!(focused(&mut scene), "item.8", "Right six times from item 2, through items the strip made as it turned");
    assert_eq!(scene.property("strip", "currentIndex"), "8");
    scene.assert_quiet();
    drop(scene);

    // The walk over the strip, which turns to each item the walk focuses: every press lands where the pad would, and is put back.
    let mut scene = harness.load("qrc:/qt/qml/EmuSen/Scenes/qml/GridScene.qml", 1280, 800);
    let tree = scene.tree();
    let walk = scene.walk(&tree);
    let name = |i: u32| tree.nodes[i as usize].name.clone();
    let items: Vec<u32> = (0..).map_while(|i| tree.find_named(&format!("item.{i}"))).collect();
    assert!(items.len() >= 5, "{} of the strip's items read", items.len());
    for (i, &from) in items.iter().enumerate() {
        let to = walk.iter().find(|s| s.from == from && s.button == UiButton::Right).map(|s| &s.to);
        match (to, items.get(i + 1)) {
            (Some(Landing::At(to)), Some(&next)) => assert_eq!(name(*to), name(next), "Right from {}", name(from)),
            (Some(Landing::Lost), None) => {}
            (to, _) => panic!("Right from {}: {to:?}", name(from)),
        }
    }
    assert_eq!(scene.property("strip", "currentIndex"), "0", "the walk put the strip back");
    assert_eq!(scene.property("strip", "contentX"), "0");
    // Settling waits out the turn, which Qt's `moving` does not report.
    let tree = scene.tree();
    let item = tree.find_named("item.5").expect("item 5");
    scene.focus_item(&tree, item);
    scene.settle(&tree);
    let settled = scene.property("strip", "contentX");
    scene.wait(500);
    assert_eq!(scene.property("strip", "contentX"), settled, "the strip at rest once settled");
    assert_ne!(settled, "0", "and turned to item 5");
    scene.assert_quiet();
}
