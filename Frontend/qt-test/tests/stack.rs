// EmuSen_Kit.md §7.33: the harness records a turned item's corners and a kit stack's members, so two turned words whose
// scene boxes cross but whose shapes do not are apart, and words inside a StackMember's item are read as that stack's.

use emusen_kit_core::audit::{Fault, Item, Rect, fit};
use emusen_qt_test::Harness;

fn named<'a>(item: &'a Item, name: &str) -> Option<&'a Item> {
    if item.name == name {
        return Some(item);
    }
    item.children.iter().find_map(|c| named(c, name))
}

#[test]
fn kit_7_33_turned_shapes_and_a_stack_are_recorded() {
    let dir = std::env::temp_dir().join(format!("emusen-stack-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("Stack.qml");
    let bar = |name: &str, x: i32, y: i32| {
        format!(
            "    Text {{\n        objectName: \"{name}\"\n        x: {x}\n        y: {y}\n        width: 300\n        height: 24\n        rotation: 45\n        text: \"A TURNED LINE OF WORDS\"\n        font.pixelSize: 16\n    }}\n"
        )
    };
    let scene = format!(
        "import QtQuick\nimport EmuSen.Kit as Kit\nWindow {{\n    width: 1280\n    height: 800\n{}{}    Item {{\n        objectName: \"member\"\n        x: 800\n        y: 300\n        width: 200\n        height: 160\n        Kit.StackMember {{}}\n        Text {{\n            objectName: \"caption\"\n            text: \"CAPTION\"\n        }}\n    }}\n}}\n",
        bar("first", 200, 300),
        bar("second", 240, 260)
    );
    std::fs::write(&file, scene).unwrap();
    let mut harness = Harness::new();
    let mut scene = harness.load(&format!("file://{}", file.display()), 1280, 800);
    let tree = scene.tree();
    let (first, second) = (named(&tree.root, "first").unwrap(), named(&tree.root, "second").unwrap());
    assert!(first.corners.is_some(), "a turned item's corners are recorded");
    assert!(first.rect.overlap(&second.rect).w > 50.0, "their scene boxes cross");
    let faults = fit(&tree.root, Rect::new(0.0, 0.0, 1280.0, 800.0), 0.5, 0.0);
    assert!(!faults.iter().any(|f| matches!(f, Fault::Overlap { .. })), "their shapes are apart: {faults:?}");
    let (member, caption) = (named(&tree.root, "member").unwrap(), named(&tree.root, "caption").unwrap());
    let (m, c) = (member.stacking.expect("the member is a stack's"), caption.stacking.expect("its words are the stack's"));
    assert!(m.is_member && !c.is_member && m.member == c.member && m.stack == c.stack);
    std::fs::remove_dir_all(&dir).unwrap();
}
