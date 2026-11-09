// EmuSen_Kit.md §7.11: the views the harness settles are the ones Qt types as views; a shadow's vector offset is not a path's.

use emusen_qt_test::Harness;

#[test]
fn qt_7_11_a_shadow_s_offset_is_not_taken_for_a_view_that_never_settles() {
    let dir = std::env::temp_dir().join(format!("emusen-settle-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("Shadow.qml");
    let scene = "import QtQuick\nimport QtQuick.Effects\nWindow {\n    width: 1280\n    height: 800\n    RectangularShadow {\n        objectName: \"shadow\"\n        width: 100\n        height: 100\n        offset.y: 8\n        blur: 24\n    }\n}\n";
    std::fs::write(&file, scene).unwrap();
    let mut harness = Harness::new();
    let mut scene = harness.load(&format!("file://{}", file.display()), 1280, 800);
    let tree = scene.tree();
    let shadow = tree.find_named("shadow").expect("the shadow is in the tree");
    assert!(!tree.nodes[shadow as usize].kept.contains_key("offset"), "a vector is not kept");
    scene.resize(1920, 1200);
    std::fs::remove_dir_all(&dir).unwrap();
}
