// EmuSen_Kit.md §7.36–§7.38: the settings frame walked by the pad through the router's queue and by the pointer, as a
// player does: rows moved and stepped, the window following, the shoulders and a tab choosing a section, A and a click
// choosing, B and a right click going back; a page of choices opened on its mark and the mark moved by A and a click; a
// slider stepped by Left and Right and dragged, a held one left as it is.

use emusen_kit_core::input::UiButton;
use emusen_kit_gallery::SETTINGS_URL;
use emusen_qt_test::tree::Tree;
use emusen_qt_test::{Harness, Scene};

const LAYOUT: &str = "settingsLayout";
const ROW: f64 = 54.0;

fn number(scene: &mut Scene<'_>, object: &str, property: &str) -> f64 {
    let text = scene.property(object, property);
    text.parse().unwrap_or_else(|_| panic!("{object}.{property} is {text:?}"))
}

fn walked(scene: &mut Scene<'_>) -> (f64, f64) {
    (number(scene, LAYOUT, "current"), number(scene, LAYOUT, "first"))
}

fn presses(scene: &mut Scene<'_>, button: UiButton, times: usize) {
    for _ in 0..times {
        scene.pad(button);
    }
}

/// The centre of the visible row whose label is heard as `label`.
fn row_at(tree: &Tree, label: &str) -> (f64, f64) {
    let n = tree.nodes.iter().find(|n| n.visible && n.heard == label && n.h > 40.0).unwrap_or_else(|| panic!("the row {label}"));
    (n.x + n.w / 2.0, n.y + n.h / 2.0)
}

/// The sliders shown, top to bottom.
fn sliders(tree: &Tree) -> Vec<(f64, f64, f64, f64)> {
    let mut found: Vec<_> = tree.nodes.iter().filter(|n| n.visible && n.name == "slider").map(|n| (n.x, n.y, n.w, n.h)).collect();
    found.sort_by(|a, b| a.1.total_cmp(&b.1));
    found
}

#[test]
fn qt_5_a_the_pad_and_the_pointer_walk_the_settings_frame_choose_and_slide() {
    let mut harness = Harness::new();
    let mut scene = harness.load(SETTINGS_URL, 1280, 800);

    // The frame: Up and Down, the window a row at a time, Left and Right on a value, A on a page's row, a held row.
    assert_eq!(number(&mut scene, LAYOUT, "count"), 12.0);
    assert_eq!(walked(&mut scene), (0.0, 0.0));
    presses(&mut scene, UiButton::Up, 1);
    assert_eq!(walked(&mut scene), (0.0, 0.0), "the first row does not wrap");
    presses(&mut scene, UiButton::Down, 9);
    assert_eq!(walked(&mut scene), (9.0, 0.0), "ten rows in the window");
    presses(&mut scene, UiButton::Down, 1);
    assert_eq!(walked(&mut scene), (10.0, 1.0), "the eleventh moves it by one");
    assert_eq!(number(&mut scene, "settingsList", "contentY"), ROW, "the list shows the window");
    presses(&mut scene, UiButton::Up, 10);
    assert_eq!(walked(&mut scene), (0.0, 0.0));
    assert_eq!(number(&mut scene, "settingsList", "contentY"), 0.0);
    scene.pad(UiButton::Right);
    assert_eq!((number(&mut scene, "scene", "steppedRow"), number(&mut scene, "scene", "steppedBy")), (0.0, 1.0));
    scene.pad(UiButton::Left);
    assert_eq!(number(&mut scene, "scene", "steppedBy"), -1.0);
    presses(&mut scene, UiButton::Down, 2);
    assert!(scene.set_property("scene", "steppedRow", "-1"));
    scene.pad(UiButton::Right);
    assert_eq!(number(&mut scene, "scene", "steppedRow"), -1.0, "a page's row does not step");
    scene.pad(UiButton::Accept);
    assert_eq!(number(&mut scene, "scene", "chosenRow"), 2.0, "A opens Shader levers");
    presses(&mut scene, UiButton::Down, 6);
    scene.pad(UiButton::Accept);
    scene.pad(UiButton::Right);
    assert_eq!(number(&mut scene, "scene", "chosenRow"), 2.0, "a held row is not chosen");
    assert_eq!(number(&mut scene, "scene", "steppedRow"), -1.0, "nor stepped");
    assert_eq!(scene.property(LAYOUT, "sentence"), "The splash as the program starts.", "the pane says the focused row's sentence");

    // The shoulders step the sections, wrapping; B at the top is the host's.
    scene.pad(UiButton::PageDown);
    assert_eq!(scene.property("settings", "section"), "2");
    assert_eq!((number(&mut scene, LAYOUT, "count"), number(&mut scene, LAYOUT, "current")), (4.0, 0.0), "Sound's rows, from the first");
    presses(&mut scene, UiButton::PageUp, 3);
    assert_eq!(scene.property("settings", "section"), "5", "LB from Controllers wraps to EmuSen");
    presses(&mut scene, UiButton::PageDown, 2);
    assert_eq!(scene.property("settings", "section"), "1");
    scene.pad(UiButton::Back);
    assert_eq!(scene.property("scene", "backs"), "1", "B is the host's");

    // The pointer: resting on a row focuses it, a click is A there, the wheel moves the window, a tab is a section, a right click is B.
    let tree = scene.tree();
    let shaders = row_at(&tree, "Shaders");
    scene.hover(shaders.0, shaders.1);
    assert_eq!(walked(&mut scene).0, 3.0, "the pointer resting on Shaders");
    let levers = row_at(&tree, "Shader levers");
    scene.click(levers.0, levers.1);
    assert_eq!((number(&mut scene, "scene", "chosenRow"), walked(&mut scene).0), (2.0, 2.0), "a click chose Shader levers");
    scene.wheel(levers.0, levers.1, -3);
    let tree = scene.tree();
    scene.settle(&tree);
    let first = walked(&mut scene).1;
    assert!(first > 0.0, "the wheel moved the window");
    assert_eq!(number(&mut scene, "settingsList", "contentY"), first * ROW, "to a whole row");
    let sound = tree.nodes.iter().find(|n| n.visible && n.text.as_deref() == Some("SOUND")).expect("the SOUND tab");
    let (sx, sy) = (sound.x + sound.w / 2.0, sound.y + sound.h / 2.0);
    scene.click(sx, sy);
    assert_eq!(scene.property("settings", "section"), "2", "a click on SOUND");
    let tree = scene.tree();
    let volume = row_at(&tree, "Volume");
    scene.right_click(volume.0, volume.1);
    assert_eq!(scene.property("scene", "backs"), "2", "a right click is B");

    // The page of choices: opened on the choice in force, A and a click move the mark, the shoulders are the host's. The
    // pointer rests on the pane first, since a row made under a resting pointer takes the focus, as hovering one does.
    scene.hover(1000.0, 600.0);
    assert!(scene.set_property("scene", "shown", "3"));
    assert_eq!((number(&mut scene, LAYOUT, "current"), number(&mut scene, LAYOUT, "marked")), (1.0, 1.0), "on Venus (C#)");
    scene.pad(UiButton::PageDown);
    assert_eq!(scene.property("settings", "section"), "2", "a page's shoulders leave the section");
    scene.pad(UiButton::Up);
    scene.pad(UiButton::Accept);
    assert_eq!((number(&mut scene, "scene", "chosenRow"), number(&mut scene, LAYOUT, "marked")), (0.0, 0.0), "A chose VenusRT");
    let tree = scene.tree();
    let marks: Vec<_> = tree.nodes.iter().filter(|n| n.visible && n.name == "mark").collect();
    assert_eq!(marks.len(), 1, "one mark");
    assert!(marks[0].y > 108.0 && marks[0].y < 108.0 + ROW, "on the first row");
    let snes9x = row_at(&tree, "Snes9x");
    scene.click(snes9x.0, snes9x.1);
    assert_eq!(number(&mut scene, LAYOUT, "marked"), 2.0, "a click chose Snes9x");

    // The sliders: Left and Right step by the row's step, the pointer drags, a held slider stays.
    scene.hover(1000.0, 600.0);
    assert!(scene.set_property("scene", "shown", "4"));
    assert_eq!(walked(&mut scene), (0.0, 0.0));
    presses(&mut scene, UiButton::Right, 2);
    assert_eq!((number(&mut scene, "scene", "slidRow"), number(&mut scene, "scene", "slidAmount")), (0.0, 1.0), "two steps of 0.5");
    scene.pad(UiButton::Left);
    assert_eq!(number(&mut scene, "scene", "slidAmount"), 0.5);
    let tree = scene.tree();
    let shown = sliders(&tree);
    assert_eq!(shown.len(), 4);
    let (x, y, w, h) = shown[1];
    scene.drag((x + w / 2.0, y + h / 2.0), (x + w + 40.0, y + h / 2.0));
    assert_eq!(
        (number(&mut scene, "scene", "slidRow"), number(&mut scene, "scene", "slidAmount")),
        (1.0, 25.0),
        "Right dragged to its end"
    );
    let tree = scene.tree();
    assert!(tree.nodes.iter().any(|n| n.visible && n.text.as_deref() == Some("25.0%") && (n.y - y).abs() < ROW), "its words follow");
    let (x, y, w, h) = shown[3];
    scene.drag((x + w - 10.0, y + h / 2.0), (x, y + h / 2.0));
    assert_eq!(
        (number(&mut scene, "scene", "slidRow"), number(&mut scene, "scene", "slidAmount")),
        (1.0, 25.0),
        "the held Bottom not dragged"
    );
    assert_eq!(walked(&mut scene).0, 3.0, "though the pointer focused it");
    scene.pad(UiButton::Left);
    assert_eq!(number(&mut scene, "scene", "slidRow"), 1.0, "nor stepped");
    scene.assert_quiet();
}
