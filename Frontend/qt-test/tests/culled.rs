// EmuSen_Kit.md §7.10: a row a list has scrolled out of its box is not shown, though the row's box meets the list's by a
// rounding's sliver, as Cores' list at 1280 x 720 scrolls one (scale 0.9: a row of 48.6 above a list at 144). No Qt here.

use emusen_kit_core::audit::{fit, Item, Rect};
use emusen_qt_test::tree::Tree;
use serde_json::{json, Value};

fn node(id: &str, parent: &str, ty: &str, rect: [f64; 4], operable: bool, clip: bool, scrolls: bool) -> Value {
    json!({
        "id": id, "parent": parent, "ty": ty, "cls": ty, "name": "", "x": rect[0], "y": rect[1], "w": rect[2], "h": rect[3],
        "visible": true, "opacity": 1.0, "clip": clip, "hit": operable, "operable": operable, "control": false,
        "focus": false, "popup": false, "menubar": false, "picture": false, "sv": scrolls,
    })
}

fn find(item: &Item, id: u32) -> Option<&Item> {
    if item.id.0 == id {
        return Some(item);
    }
    item.children.iter().find_map(|c| find(c, id))
}

/// The strip's tab above a list whose first row is scrolled up out of it, its foot `sliver` past the list's top.
fn screen(sliver: f64) -> Tree {
    let list_top = 160.0 * 0.9;
    let row = 54.0 * 0.9;
    let nodes = json!([
        node("1", "0", "Root", [0.0, 0.0, 1280.0, 720.0], false, false, false),
        node("2", "1", "TabButton", [64.0, 90.0, 140.0, 54.0], true, false, false),
        node("3", "1", "QQuickListView", [57.6, list_top, 621.9, 388.8], false, true, true),
        node("4", "3", "Walkable", [57.6, list_top - row + sliver, 621.9, row], true, false, false),
        node("5", "3", "Walkable", [57.6, list_top + sliver, 621.9, row], true, false, false),
    ]);
    Tree::from_json(&nodes.to_string()).unwrap()
}

#[test]
fn qt_kit_7_10_a_row_scrolled_out_by_all_but_a_rounding_s_sliver_is_not_shown() {
    for sliver in [0.0, 1e-9, 0.2] {
        let tree = screen(sliver);
        assert!(!find(&tree.root, 3).unwrap().visible, "the row out of the list is not shown ({sliver})");
        assert!(find(&tree.root, 4).unwrap().visible, "the row in the list is shown ({sliver})");
        let faults = fit(&tree.root, Rect::new(0.0, 0.0, 1280.0, 720.0), 1.0, 0.0);
        assert!(faults.is_empty(), "{sliver}: {:?}", faults.iter().map(ToString::to_string).collect::<Vec<_>>());
    }
    // A row half a pixel or more into the list is drawn there, so it is shown.
    assert!(find(&screen(0.6).root, 3).unwrap().visible);
}
