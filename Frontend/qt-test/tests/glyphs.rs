// EmuSen_Kit.md §7.12: every press's glyph in every family draws something inside its ring, and the grab is saved for a human look.

use emusen_qt_test::Harness;

#[test]
fn kit_7_12_every_glyph_of_every_family_draws_something() {
    let mut harness = Harness::new();
    let mut scene = harness.load("qrc:/qt/qml/EmuSen/Scenes/qml/GlyphScene.qml", 1280, 800);
    let tree = scene.tree();
    let glyphs: Vec<u32> = (0..tree.nodes.len() as u32).filter(|i| tree.nodes[*i as usize].name == "glyph").collect();
    assert_eq!(glyphs.len(), 4 * 16, "sixteen presses in four families");
    let names: Vec<String> = glyphs.iter().map(|i| tree.nodes[*i as usize].heard.clone()).collect();
    assert!(names.iter().all(|n| !n.is_empty()), "every glyph has a name a reader hears");
    assert_eq!(
        (names[4].as_str(), names[16 + 4].as_str(), names[32 + 4].as_str(), names[48 + 4].as_str()),
        ("South button", "A", "Cross", "B"),
        "Accept on each family's South"
    );
    let empty: Vec<String> = scene
        .ink_inside(&tree, &glyphs)
        .into_iter()
        .filter(|(_, inked)| *inked < 2)
        .map(|(i, inked)| format!("{} ({inked} inked)", tree.nodes[i as usize].heard))
        .collect();
    assert!(empty.is_empty(), "glyphs that draw nothing inside their ring: {}", empty.join(", "));
    let path = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".cache/emusen/tmp/qt-test/glyphs.png");
    assert!(scene.grab(&path.display().to_string()), "the grab saved");
    scene.assert_quiet();
}
