// EmuSen_Kit.md §7.10: a flickable scrolls over its margins as well as its content, so the audit holds it as a scroller
// when content and margins together are longer than it, as Qt lets it move.

use emusen_qt_test::Harness;

#[test]
fn qt_7_10_a_flickable_whose_margins_make_it_longer_than_itself_scrolls() {
    let dir = std::env::temp_dir().join(format!("emusen-scroller-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("Margins.qml");
    let scene = "import QtQuick\nWindow {\n    width: 1280\n    height: 800\n    Flickable {\n        objectName: \"strip\"\n        width: 1280\n        height: 300\n        flickableDirection: Flickable.HorizontalFlick\n        leftMargin: 64\n        rightMargin: 64\n        contentWidth: 1220\n        contentHeight: 300\n    }\n    Flickable {\n        objectName: \"short\"\n        y: 400\n        width: 1280\n        height: 300\n        flickableDirection: Flickable.HorizontalFlick\n        leftMargin: 64\n        rightMargin: 64\n        contentWidth: 1100\n        contentHeight: 300\n    }\n}\n";
    std::fs::write(&file, scene).unwrap();
    let mut harness = Harness::new();
    let mut scene = harness.load(&format!("file://{}", file.display()), 1280, 800);
    let tree = scene.tree();
    let node = |name: &str| &tree.nodes[tree.find_named(name).expect("in the tree") as usize];
    assert!(node("strip").sh, "1220 and two margins of 64 are longer than 1280");
    assert!(!node("short").sh, "1100 and two margins of 64 are not");
    std::fs::remove_dir_all(&dir).unwrap();
}
