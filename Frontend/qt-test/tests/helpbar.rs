// EmuSen_Kit.md §7.28: the help bar outside a frame raises its presses itself, draws the filled set in a theme's colours, hugs
// its entries in a box, and draws a theme's own picture in a glyph's place; the grab is saved for the side-by-side.

use emusen_qt_test::Harness;

#[test]
fn kit_7_28_a_help_bar_outside_a_frame_in_the_filled_set() {
    let mut harness = Harness::new();
    let mut scene = harness.load("qrc:/qt/qml/EmuSen/Scenes/qml/HelpBarScene.qml", 1280, 800);
    let tree = scene.tree();
    let node = |name: &str| tree.find_named(name).map(|i| &tree.nodes[i as usize]).unwrap_or_else(|| panic!("{name} in the tree"));

    // A click on an entry is its press, raised once, with no controller to offer it to.
    let play = node("help-Play");
    scene.click(play.x + play.w / 2.0, play.y + play.h / 2.0);
    assert_eq!((scene.property("scene", "pressedButton"), scene.property("scene", "pressedCount")), ("4".into(), "1".into()));

    // The box hugs its entries: the tallest of them and the paddings.
    let (bar, glyph) = (node("ownHelp"), node("glyph-Play"));
    assert!((bar.h - (glyph.h + 16.0)).abs() <= 1.0, "a fitted bar {} high round a glyph {} high and 8 above and below", bar.h, glyph.h);
    let first = node("help-Play");
    assert!((first.x - bar.x - 16.0).abs() <= 1.0, "the first entry 16 in from the box's edge, at {}", first.x - bar.x);

    // A theme's own picture is drawn in the glyph's place, and the glyph is not.
    let (icon, hidden) = (node("icon-Menu"), node("glyph-Menu"));
    assert!(icon.visible && !hidden.visible, "the picture shown, the glyph hidden");
    let colour = scene.colours_at(&[(icon.x + icon.w / 2.0, icon.y + icon.h / 2.0)])[0].expect("a colour under the picture");
    assert!(colour.r > 200 && colour.g < 100 && colour.b < 100, "the picture's red drawn: {colour:?}");

    // Every filled glyph of the four families shows its cut: two colours or more inside its disc.
    let glyphs: Vec<u32> = (0..tree.nodes.len() as u32)
        .filter(|i| {
            let n = &tree.nodes[*i as usize];
            n.visible && n.name.starts_with("glyph-") && n.w > 0.0
        })
        .collect();
    assert!(glyphs.len() >= 4 * 10, "{} glyphs on the four bars", glyphs.len());
    let uncut: Vec<String> = scene
        .ink_inside(&tree, &glyphs)
        .into_iter()
        .filter(|(_, inked)| *inked < 2)
        .map(|(i, inked)| format!("{} ({inked})", tree.nodes[i as usize].heard))
        .collect();
    assert!(uncut.is_empty(), "filled glyphs with nothing cut out: {}", uncut.join(", "));
    let path = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".cache/emusen/tmp/qt-test/helpbar-filled.png");
    assert!(scene.grab(&path.display().to_string()), "the grab saved");
    scene.assert_quiet();
}
