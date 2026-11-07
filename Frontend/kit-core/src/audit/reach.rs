use std::collections::HashSet;

use super::Fault;
use super::tree::{Item, Kind};
use crate::input::{ElementId, UiButton};

/// One edge of a screen's recorded focus graph: from an element, a press reaches another.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Edge {
    pub from: ElementId,
    pub button: UiButton,
    pub to: ElementId,
}

/// Every element the pad reaches from the focus along the edges, the focus included.
pub fn reached(focus: Option<ElementId>, edges: &[Edge]) -> HashSet<ElementId> {
    let mut seen = HashSet::new();
    let Some(start) = focus else { return seen };
    let mut queue = vec![start];
    while let Some(at) = queue.pop() {
        if !seen.insert(at) {
            continue;
        }
        queue.extend(edges.iter().filter(|e| e.from == at).map(|e| e.to));
    }
    seen
}

/// Pad reachability (InterfaceSpec §4.2): from the focus, the edges reach every operable element not kept out of view.
pub fn pad_reach(root: &Item, focus: Option<ElementId>, edges: &[Edge]) -> Vec<Fault> {
    let seen = reached(focus, edges);
    let mut faults = Vec::new();
    root.visit(&mut |item, _| {
        if item.operable && !item.unseen && !seen.contains(&item.id) {
            faults.push(Fault::Unreachable { item: item.id });
        }
    });
    faults
}

/// Pointer reach (EmuSen_Qt.md §8.1): a click at the centre of each element the walk reaches hits it and calls what A calls, and the wheel moves every list that scrolls.
pub fn pointer_reach(root: &Item, reached: &HashSet<ElementId>) -> Vec<Fault> {
    let mut faults = Vec::new();
    root.visit(&mut |item, _| {
        if reached.contains(&item.id) {
            let (x, y) = item.rect.centre();
            match root.topmost_at(x, y) {
                Some(top) if item.encloses(top.id) => {}
                Some(top) => faults.push(Fault::Covered { item: item.id, by: top.id }),
                None => faults.push(Fault::Covered { item: item.id, by: item.id }),
            }
            if item.click_call != item.accept_call {
                faults.push(Fault::ClickDiffers { item: item.id });
            }
        }
        if item.kind == Kind::Scroller && (item.scroll.vertical || item.scroll.horizontal) && !item.wheel_scrolls {
            faults.push(Fault::WheelDoesNotMove { item: item.id });
        }
    });
    faults
}
