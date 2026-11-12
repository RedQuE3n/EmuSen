// EmuSen_Kit.md §7.105: a disabled control's words keep their ink in the tree, so the contrast rule sees them, and kit-core's
// `exempted` names each that falls under it with R-20261111-43's class; no other place exempts them.

use emusen_kit_core::audit::{self, Class, Exempted, Fault};
use emusen_kit_core::input::ElementId;
use emusen_qt_test::Harness;

#[test]
fn kit_7_105_a_disabled_control_s_words_are_named_under_their_ruling_not_dropped() {
    let mut harness = Harness::new();
    let mut scene = harness.load("qrc:/qt/qml/EmuSen/Scenes/qml/HelpBarScene.qml", 1280, 800);
    let mut tree = scene.tree();
    scene.sample_behind(&mut tree, 1280.0, 800.0);
    let back: Vec<u32> = (0..tree.nodes.len() as u32)
        .filter(|&i| {
            let n = &tree.nodes[i as usize];
            n.visible && !n.enabled && n.text.as_deref() == Some("Back")
        })
        .collect();
    assert!(!back.is_empty(), "the declined Back entry's words are in the tree");
    for &i in &back {
        let n = &tree.nodes[i as usize];
        assert!(n.ink.as_deref().is_some_and(|ink| !ink.is_empty()), "a disabled control's words keep their ink: {:?}", n.ink);
        let item = tree.root.find(ElementId(i)).expect("the item");
        assert!(item.legibility.as_ref().is_some_and(|w| w.disabled), "read as a disabled control's words");
    }
    let low: Vec<ElementId> = audit::contrast(&tree.root).iter().filter_map(|f| match f {
        Fault::LowContrast { item, .. } => Some(*item),
        _ => None,
    }).collect();
    assert!(back.iter().all(|i| !low.contains(&ElementId(*i))), "the contrast rule leaves them to exempted: {low:?}");
    let named: Vec<Exempted> = audit::exempted(&tree.root, None, 0.0).into_iter().filter(|e| matches!(e, Exempted::Disabled { .. })).collect();
    assert!(back.iter().any(|i| named.iter().any(|e| e.item() == ElementId(*i))), "Back at 38% is under its contrast and named: {named:?}");
    assert!(named.iter().all(|e| e.class() == Class::DimmedAndMoving && e.accepted().map(|a| a.ruling) == Some("R-20261111-43")));
}
