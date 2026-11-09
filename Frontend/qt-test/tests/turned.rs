// EmuSen_Kit.md §3: words run along their item's own axes, so a turned Text is held to its own box, not to its turned
// footprint in the scene: one that fits passes, one that overflows its own box is caught.

use emusen_kit_core::audit::{fit, Fault, Rect};
use emusen_qt_test::Harness;

#[test]
fn qt_7_10_a_turned_text_is_held_to_its_own_box() {
    let dir = std::env::temp_dir().join(format!("emusen-turned-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("Turned.qml");
    let scene = "import QtQuick\nWindow {\n    width: 1280\n    height: 800\n    Text {\n        objectName: \"fits\"\n        x: 200\n        y: 300\n        width: implicitWidth\n        height: implicitHeight\n        rotation: -90\n        text: \"RECENT\"\n        font.pixelSize: 20\n    }\n    Text {\n        objectName: \"overflows\"\n        x: 600\n        y: 300\n        width: 30\n        height: implicitHeight\n        rotation: -90\n        text: \"RECENTLY PLAYED\"\n        font.pixelSize: 20\n    }\n}\n";
    std::fs::write(&file, scene).unwrap();
    let mut harness = Harness::new();
    let mut scene = harness.load(&format!("file://{}", file.display()), 1280, 800);
    let tree = scene.tree();
    let fits = tree.find_named("fits").expect("in the tree");
    let overflows = tree.find_named("overflows").expect("in the tree");
    let faults = fit(&tree.root, Rect::new(0.0, 0.0, 1280.0, 800.0), 0.5, 0.0);
    let runs_past = |index: u32| faults.iter().any(|f| matches!(f, Fault::RunsPastItsBox { item } if item.0 == index));
    assert!(!runs_past(fits), "a turned word that fits its own box is whole: {faults:?}");
    assert!(runs_past(overflows), "a turned word longer than its own box still runs past it: {faults:?}");
    std::fs::remove_dir_all(&dir).unwrap();
}
