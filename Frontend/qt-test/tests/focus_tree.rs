// EmuSen_Kit.md §7.9, the focus's tree: the focus's ancestors, its walkable's subtree and every view, each node as the whole tree writes it.

use emusen_qt_test::Harness;

const DOWN: i32 = 0x0100_0015;

#[test]
fn kit_7_9_the_focus_s_tree_holds_the_whole_tree_s_nodes_for_the_focus_its_words_and_the_views() {
    let mut harness = Harness::new();
    let mut scene = harness.load("qrc:/qt/qml/EmuSen/Scenes/qml/LongListScene.qml", 1280, 800);
    for _ in 0..14 {
        scene.key(DOWN, 0);
    }
    let whole = scene.tree();
    let focused = scene.focus_tree();
    assert!(focused.nodes.len() < whole.nodes.len(), "a part of the tree: {} of {}", focused.nodes.len(), whole.nodes.len());
    // Each node it writes is the whole tree's node of that item, every field.
    for node in &focused.nodes {
        let same = whole.nodes.iter().find(|n| n.id == node.id).unwrap_or_else(|| panic!("{} {} is in the window's tree", node.ty, node.name));
        assert_eq!(format!("{node:?}"), format!("{same:?}"), "{} as the whole tree writes it", node.name);
    }
    // The focus and its walkable stand as in the whole tree, with every node under the walkable.
    let (f, w) = (scene.focus(&focused).expect("a focus"), scene.focus(&whole).expect("a focus"));
    assert_eq!(focused.nodes[f as usize].name, "row.14");
    assert_eq!(whole.nodes[w as usize].name, "row.14");
    let under = |tree: &emusen_qt_test::tree::Tree, at: u32| -> Vec<String> {
        let mut ids = vec![tree.nodes[at as usize].id.clone()];
        for n in &tree.nodes[at as usize + 1..] {
            if ids.contains(&n.parent) {
                ids.push(n.id.clone());
            }
        }
        ids
    };
    assert_eq!(under(&focused, f), under(&whole, w), "the walkable's subtree, in the whole tree's order");
    // Every view the settle waits on.
    let views = |tree: &emusen_qt_test::tree::Tree| -> Vec<String> {
        let mut v: Vec<String> = tree.nodes.iter().filter(|n| n.kept.contains_key("contentY") || n.kept.contains_key("offset")).map(|n| n.id.clone()).collect();
        v.sort();
        v
    };
    assert!(!views(&whole).is_empty());
    assert_eq!(views(&focused), views(&whole), "the views");
    scene.assert_quiet();
}
