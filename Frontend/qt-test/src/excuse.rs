//! A test's excuses (EmuSen_Kit.md §7.100): each finding a test does not count names the one item it excuses, the finding,
//! the size it holds at and who decided it; an excuse that excused nothing by the time the test ends fails the test.

use crate::tree::Node;

/// One finding a test does not count.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Excuse {
    /// The finding's words as the gate or the mockup writes them: `low contrast`, `too small to read`, `Y`.
    pub finding: &'static str,
    /// The objectName of the one item it excuses.
    pub item: &'static str,
    /// Words the item's own words hold, where its name is shared by others (a row's `value`); empty for any.
    pub words: &'static str,
    /// The window size it holds at; every size when `None`.
    pub size: Option<(i32, i32)>,
    /// Why, and who decided it and when, or `no ruling`.
    pub why: &'static str,
}

/// A test's excuses and how often each excused a finding; dropped with one unused, the test fails.
#[derive(Debug, Default)]
pub struct Excuses {
    list: Vec<(Excuse, usize)>,
}

impl Excuses {
    pub fn new(list: &[Excuse]) -> Excuses {
        Excuses { list: list.iter().cloned().map(|e| (e, 0)).collect() }
    }

    /// No excuse: every finding counts.
    pub fn none() -> Excuses {
        Excuses::default()
    }

    /// Whether one of the excuses names this finding over these items at this size; the first that does is counted used.
    pub fn excuses(&mut self, fault: &str, nodes: &[Node], size: (i32, i32)) -> bool {
        for (excuse, used) in &mut self.list {
            if excuse.size.is_none_or(|s| s == size) && names(fault, excuse.finding, excuse.item, excuse.words, nodes) {
                *used += 1;
                return true;
            }
        }
        false
    }

    /// The excuses that excused nothing, each in words.
    pub fn stale(&self) -> Vec<String> {
        self.list
            .iter()
            .filter(|(_, used)| *used == 0)
            .map(|(e, _)| format!("{} on {}{}: excused nothing", e.finding, e.item, e.size.map_or(String::new(), |(w, h)| format!(" at {w}x{h}"))))
            .collect()
    }
}

impl Drop for Excuses {
    fn drop(&mut self) {
        let stale = self.stale();
        if !stale.is_empty() && !std::thread::panicking() {
            panic!("{} stale excuses:\n{}", stale.len(), stale.join("\n"));
        }
    }
}

/// Whether `fault` is of `finding` and names `item`: as a mockup's finding begins with its element's name, or as the gate's
/// numbers the item right after its words (both numbers of an overlap or of a limit and what passes it).
pub fn names(fault: &str, finding: &str, item: &str, words: &str, nodes: &[Node]) -> bool {
    let Some(at) = fault.find(finding) else { return false };
    if words.is_empty() && fault[..at].ends_with(&format!("{item}: ")) {
        return true;
    }
    let said = |n: &Node| words.is_empty() || n.text.as_deref().is_some_and(|t| t.contains(words));
    let both = finding == "overlap" || finding.starts_with("past its scroller");
    let ids = fault[at + finding.len()..]
        .split_whitespace()
        .map(|w| w.trim_end_matches([':', ',']))
        .filter_map(|w| w.parse::<usize>().ok())
        .take(if both { 2 } else { 1 });
    ids.collect::<Vec<_>>().into_iter().any(|i| nodes.get(i).is_some_and(|n| n.name == item && said(n)))
}
