// EmuSen_Kit.md §7.11: settle says whether it waited, since theme-audit's sweep reads its focus from the tree it settled by
// only when settle did not wait (man theme-check); a view easing to its current row is waited for, a view at rest is not.

use emusen_qt_test::{Harness, Scene};

const DOWN: i32 = 0x0100_0015;

/// Each row's place in the window, as the tree holds it.
fn rows(scene: &mut Scene<'_>) -> Vec<(String, f64)> {
    let tree = scene.tree();
    tree.nodes.iter().filter(|n| n.name.starts_with("row.")).map(|n| (n.name.clone(), n.y)).collect()
}

#[test]
fn qt_7_11_settle_says_whether_it_waited_for_a_view() {
    let dir = std::env::temp_dir().join(format!("emusen-settled-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("Rows.qml");
    let scene = "import QtQuick\nWindow {\n    width: 1280\n    height: 800\n    ListView {\n        objectName: \"rows\"\n        width: 480\n        height: 300\n        clip: true\n        focus: true\n        keyNavigationEnabled: true\n        highlightMoveDuration: 300\n        highlight: Rectangle { color: \"#334\" }\n        model: 30\n        delegate: Rectangle {\n            required property int index\n            objectName: \"row.\" + index\n            width: 480\n            height: 56\n            color: \"transparent\"\n        }\n    }\n}\n";
    std::fs::write(&file, scene).unwrap();
    let mut harness = Harness::new();
    let mut scene = harness.load(&format!("file://{}", file.display()), 1280, 800);
    scene.rest();
    let still = scene.tree();
    let before = rows(&mut scene);
    assert!(!scene.settle(&still), "a view at rest is not waited for");
    assert_eq!(rows(&mut scene), before, "a tree read after a settle that did not wait is the tree it was given");
    for _ in 0..8 {
        scene.key(DOWN, 0);
    }
    let pressed = scene.tree();
    let eased = rows(&mut scene);
    assert!(scene.settle(&pressed), "a view easing to its current row is waited for");
    let settled = rows(&mut scene);
    assert_ne!(settled, eased, "the rows moved while settle waited, so the tree read before it no longer stands");
    let again = scene.tree();
    assert!(!scene.settle(&again), "once at rest, settle does not wait again");
    std::fs::remove_dir_all(&dir).unwrap();
}
