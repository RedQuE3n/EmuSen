// The overlap rule on turned shapes and on a stack a kit layout declares (EmuSen_Kit.md §7.33), on hand-built trees.

use emusen_kit_core::audit::{Fault, Item, Kind, Rect, Stacking, Text, fit, smallest_capitals};
use emusen_kit_core::input::ElementId;

const PANEL: Rect = Rect::new(0.0, 0.0, 1280.0, 800.0);

/// A bar `length` by `thick` about (cx, cy), turned 45°, as its corners and its scene box.
fn bar(id: u32, cx: f64, cy: f64, length: f64, thick: f64) -> Item {
    let (u, n) = ((0.5_f64.sqrt(), 0.5_f64.sqrt()), (0.5_f64.sqrt(), -0.5_f64.sqrt()));
    let corner = |a: f64, b: f64| (cx + u.0 * a + n.0 * b, cy + u.1 * a + n.1 * b);
    let (l, t) = (length / 2.0, thick / 2.0);
    let corners = [corner(-l, -t), corner(l, -t), corner(l, t), corner(-l, t)];
    let (xs, ys): (Vec<f64>, Vec<f64>) = corners.iter().copied().unzip();
    let lo = |v: &[f64]| v.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = |v: &[f64]| v.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    Item::new(id, Kind::Control, Rect::new(lo(&xs), lo(&ys), hi(&xs) - lo(&xs), hi(&ys) - lo(&ys))).operable().turned(corners)
}

fn overlaps(root: &Item) -> Vec<Fault> {
    fit(root, PANEL, 1.0, smallest_capitals()).into_iter().filter(|f| matches!(f, Fault::Overlap { .. })).collect()
}

fn screen(items: Vec<Item>) -> Item {
    items.into_iter().fold(Item::new(1, Kind::Panel, PANEL), Item::child)
}

#[test]
fn turned_boxes_overlap_by_their_shapes_not_their_scene_boxes() {
    let a = bar(10, 300.0, 300.0, 200.0, 20.0);
    let apart = bar(11, 300.0 + 30.0 * 0.5_f64.sqrt(), 300.0 - 30.0 * 0.5_f64.sqrt(), 200.0, 20.0);
    assert!(a.rect.overlap(&apart.rect).w > 100.0, "their scene boxes cross");
    assert_eq!(overlaps(&screen(vec![a.clone(), apart])), vec![], "their shapes lie 10 apart");
    let touching = bar(12, 300.0 + 15.0 * 0.5_f64.sqrt(), 300.0 - 15.0 * 0.5_f64.sqrt(), 200.0, 20.0);
    assert_eq!(overlaps(&screen(vec![a, touching])), vec![Fault::Overlap { a: ElementId(10), b: ElementId(12) }], "5 of 20 shared");
}

/// A cover of a stack at `x`, 200 wide and 160 tall, at a layer, with its caption 140 wide along its foot from `x + dx`.
fn cover(id: u32, x: f64, dx: f64, layer: f64, focused: bool) -> Item {
    let at = |is_member| Stacking { stack: 7, member: u64::from(id), layer, focused, is_member };
    let caption = Item::new(id + 1, Kind::Words, Rect::new(x + dx, 230.0, 140.0, 24.0))
        .words(Text { cap_height: 14.0, width: 120.0, height: 24.0, ellipsis: false, data: true, own: None })
        .stacked(at(false));
    Item::new(id, Kind::Control, Rect::new(x, 100.0, 200.0, 160.0)).operable().stacked(at(true)).child(caption)
}

#[test]
fn a_stack_lies_over_itself_in_its_order_and_over_nothing_else() {
    // The apex at the top layer, focused; its neighbours under it on both sides, their captions clear of it.
    let stacked = screen(vec![cover(20, 400.0, 10.0, 10.0, true), cover(30, 560.0, 50.0, 9.0, false), cover(40, 240.0, 10.0, 9.0, false)]);
    assert_eq!(overlaps(&stacked), vec![], "covers over covers in their order");

    // Seeded: a neighbour's caption reaches under the apex.
    let mut hidden = cover(30, 560.0, 50.0, 9.0, false);
    hidden.children[0].rect = Rect::new(500.0, 230.0, 140.0, 24.0);
    let faults = overlaps(&screen(vec![cover(20, 400.0, 10.0, 10.0, true), hidden]));
    assert!(faults.contains(&Fault::Overlap { a: ElementId(31), b: ElementId(20) }), "a caption under its neighbour: {faults:?}");

    // Seeded: a stack's cover over a label outside the stack.
    let label = Item::new(50, Kind::Words, Rect::new(700.0, 240.0, 120.0, 24.0)).words(Text {
        cap_height: 14.0,
        width: 100.0,
        height: 24.0,
        ellipsis: false,
        data: true,
        own: None,
    });
    let faults = overlaps(&screen(vec![cover(30, 560.0, 50.0, 9.0, false), label]));
    assert!(faults.contains(&Fault::Overlap { a: ElementId(30), b: ElementId(50) }), "a cover over a neighbouring label: {faults:?}");

    // The focused cover under another hides its ring.
    let faults = overlaps(&screen(vec![cover(20, 400.0, 10.0, 8.0, true), cover(30, 560.0, 50.0, 9.0, false)]));
    assert!(faults.contains(&Fault::Overlap { a: ElementId(20), b: ElementId(30) }), "a focused member under another: {faults:?}");

    // Two stacks are strangers: their covers may not lie over each other.
    let mut other = cover(30, 560.0, 50.0, 9.0, false);
    other.stacking.as_mut().unwrap().stack = 8;
    let faults = overlaps(&screen(vec![cover(20, 400.0, 10.0, 10.0, true), other]));
    assert!(faults.contains(&Fault::Overlap { a: ElementId(20), b: ElementId(30) }), "another stack's cover: {faults:?}");
}
