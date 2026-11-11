//! A test's excuses (EmuSen_Kit.md §7.100): each finding a test does not count names its kind, the items it is
//! about, the size it holds at and who decided it; an excuse that excused nothing by the time the test ends fails the test.

use crate::mockup::Difference;
use crate::tree::Node;
use emusen_kit_core::audit::{ACCEPTED, Accepted, Exempted, Fault, SMALL_TEXT_EXCEPTIONS};
use emusen_kit_core::input::ElementId;

/// One finding a test does not count.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Excuse {
    /// The finding's kind: kit-core's `Fault::kind` or `Exempted::kind` (`low contrast`, `cut with an ellipsis`), or a
    /// mockup difference's (`Y`, `not shown`).
    pub finding: &'static str,
    /// The objectName of the item it excuses.
    pub item: &'static str,
    /// The other item of a finding about two (an overlap, a limit passed, an edge cut at); empty for a finding about one.
    pub with: &'static str,
    /// Words the item's own words hold; needed where its name is shared by others (a row's `value`).
    pub words: &'static str,
    /// Words the other item's own words hold, as `words` is for the first.
    pub with_words: &'static str,
    /// The window size it holds at; every size when `None`.
    pub size: Option<(i32, i32)>,
    /// Why, and who decided it and when, or `no ruling`.
    pub why: &'static str,
}

/// One item a finding names, as the tree has it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Named {
    pub name: String,
    pub ty: String,
    pub text: Option<String>,
    /// Whether another item of the tree has its name, so that the name alone does not tell which it is.
    pub shared: bool,
}

/// One finding of a held state in the one form an excuse reads: its kind, the items it names and its line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    pub kind: &'static str,
    pub items: Vec<Named>,
    pub line: String,
}

fn named(id: ElementId, nodes: &[Node]) -> Named {
    let Some(n) = nodes.get(id.0 as usize) else { return Named { name: format!("#{}", id.0), ..Named::default() } };
    let shared = !n.name.is_empty() && nodes.iter().filter(|o| o.name == n.name).count() > 1;
    Named { name: n.name.clone(), ty: n.ty.clone(), text: n.text.clone(), shared }
}

impl Finding {
    /// A rule's finding, its items read from the fault itself.
    pub fn gate(fault: &Fault, nodes: &[Node]) -> Finding {
        Finding { kind: fault.kind(), items: fault.items().into_iter().map(|i| named(i, nodes)).collect(), line: format!("gate: {fault}") }
    }

    /// An item a rule's exemption let pass.
    pub fn exempted(exempted: &Exempted, nodes: &[Node]) -> Finding {
        Finding { kind: exempted.kind(), items: vec![named(exempted.item(), nodes)], line: format!("gate: {exempted}") }
    }

    /// A difference from the mockup, about the element of its name, which the mockup names once.
    pub fn mockup(difference: &Difference, nodes: &[Node]) -> Finding {
        let shown = nodes.iter().find(|n| n.name == difference.element && n.visible);
        let item = Named {
            name: difference.element.clone(),
            ty: shown.map(|n| n.ty.clone()).unwrap_or_default(),
            text: shown.and_then(|n| n.text.clone()),
            shared: false,
        };
        Finding { kind: difference.kind, items: vec![item], line: format!("mockup: {difference}") }
    }

    /// The items as a reader finds them in the QML: type and name.
    pub fn who(&self) -> String {
        self.items.iter().map(|n| format!("{} {}", n.ty, n.name)).collect::<Vec<_>>().join(", ")
    }
}

/// What an excuse makes of a finding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// Not the finding it names.
    Other,
    /// Its finding, on an item whose name others share, and the excuse gives no words to tell which.
    Ambiguous,
    /// The finding it names.
    Names,
}

/// Whether one item is the one an excuse names by `name` and `words`.
fn tells(item: &Named, name: &str, words: &str) -> Verdict {
    if item.name != name {
        return Verdict::Other;
    }
    if words.is_empty() {
        return if item.shared { Verdict::Ambiguous } else { Verdict::Names };
    }
    if item.text.as_deref().is_some_and(|t| t.contains(words)) { Verdict::Names } else { Verdict::Other }
}

/// Whether `excuse` names `finding`: the same kind, the same items (both of a pair, either way round, each held to its
/// name and words alike), its words if given.
pub fn names(excuse: &Excuse, finding: &Finding) -> Verdict {
    if excuse.finding != finding.kind {
        return Verdict::Other;
    }
    let both = |a: Verdict, b: Verdict| match (a, b) {
        (Verdict::Names, Verdict::Names) => Verdict::Names,
        (Verdict::Other, _) | (_, Verdict::Other) => Verdict::Other,
        _ => Verdict::Ambiguous,
    };
    match (excuse.with, finding.items.as_slice()) {
        ("", [a]) => tells(a, excuse.item, excuse.words),
        (with, [a, b]) if !with.is_empty() => {
            let forward = both(tells(a, excuse.item, excuse.words), tells(b, with, excuse.with_words));
            let back = both(tells(b, excuse.item, excuse.words), tells(a, with, excuse.with_words));
            match (forward, back) {
                (Verdict::Names, _) | (_, Verdict::Names) => Verdict::Names,
                (Verdict::Ambiguous, _) | (_, Verdict::Ambiguous) => Verdict::Ambiguous,
                _ => Verdict::Other,
            }
        }
        _ => Verdict::Other,
    }
}

/// One state's exemptions sorted by a table of accepted classes: the line of each it accepts, with its ruling, and each
/// it does not, which is a finding (EmuSen_Kit.md §7.102).
pub fn sort(exempted: &[Exempted], table: &'static [Accepted]) -> (Vec<String>, Vec<Exempted>) {
    let mut accepted = Vec::new();
    let mut refused = Vec::new();
    for e in exempted {
        match e.accepted_in(table) {
            Some(a) => accepted.push(format!("accepted by {}: {e}", a.ruling)),
            None => refused.push(e.clone()),
        }
    }
    (accepted, refused)
}

/// A test's excuses, how often each excused a finding, and how often one met its finding on an item it could not tell
/// from others; dropped with one unused, the test fails. It holds the table of accepted classes its exemptions are
/// sorted by, and lists each it accepted.
#[derive(Debug)]
pub struct Excuses {
    list: Vec<(Excuse, usize, usize)>,
    table: &'static [Accepted],
    listed: Vec<String>,
}

impl Default for Excuses {
    fn default() -> Excuses {
        Excuses { list: Vec::new(), table: &ACCEPTED, listed: Vec::new() }
    }
}

impl Excuses {
    pub fn new(list: &[Excuse]) -> Excuses {
        Excuses { list: list.iter().cloned().map(|e| (e, 0, 0)).collect(), table: &ACCEPTED, listed: Vec::new() }
    }

    /// No excuse: every finding counts.
    pub fn none() -> Excuses {
        Excuses::default()
    }

    /// The same, its exemptions sorted by `table` in place of kit-core's `ACCEPTED`.
    pub fn accepting(mut self, table: &'static [Accepted]) -> Excuses {
        self.table = table;
        self
    }

    /// One state's exemptions: those the table accepts are listed under `state`, the others returned as findings.
    pub fn sort(&mut self, state: &str, exempted: &[Exempted]) -> Vec<Exempted> {
        let (accepted, refused) = sort(exempted, self.table);
        self.listed.extend(accepted.into_iter().map(|l| format!("{state}: {l}")));
        refused
    }

    /// Every exemption accepted so far, each with its state and its ruling.
    pub fn listed(&self) -> &[String] {
        &self.listed
    }

    /// Whether one of the excuses names this finding at this size; the first that does is counted used.
    pub fn excuses(&mut self, finding: &Finding, size: (i32, i32)) -> bool {
        for (excuse, used, ambiguous) in &mut self.list {
            if !excuse.size.is_none_or(|s| s == size) {
                continue;
            }
            match names(excuse, finding) {
                Verdict::Names => {
                    *used += 1;
                    return true;
                }
                Verdict::Ambiguous => *ambiguous += 1,
                Verdict::Other => {}
            }
        }
        false
    }

    /// The excuses that excused nothing, each in words.
    pub fn stale(&self) -> Vec<String> {
        self.list
            .iter()
            .filter(|(_, used, _)| *used == 0)
            .map(|(e, _, ambiguous)| {
                let with = if e.with.is_empty() { String::new() } else { format!(" with {}", e.with) };
                let at = e.size.map_or(String::new(), |(w, h)| format!(" at {w}x{h}"));
                let why = if *ambiguous > 0 { ", its item's name shared by others and no words given" } else { "" };
                format!("{} on {}{with}{at}: excused nothing{why}", e.finding, e.item)
            })
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

/// Which of kit-core's `SMALL_TEXT_EXCEPTIONS` a gate saw let pass, over every state it held; dropped with an entry that
/// let nothing pass, the gate fails, as a stale excuse does.
#[derive(Debug, Default)]
pub struct SmallText {
    seen: Vec<&'static str>,
}

impl SmallText {
    pub fn new() -> SmallText {
        SmallText::default()
    }

    /// Notes each named small text among one state's exemptions.
    pub fn saw(&mut self, exempted: &[Exempted], nodes: &[Node]) {
        for e in exempted.iter().filter(|e| matches!(e, Exempted::SmallText { .. })) {
            let name = nodes.get(e.item().0 as usize).map(|n| n.name.as_str()).unwrap_or_default();
            let entry = SMALL_TEXT_EXCEPTIONS.iter().map(|(n, _)| *n).find(|n| *n == name);
            if let Some(entry) = entry.filter(|e| !self.seen.contains(e)) {
                self.seen.push(entry);
            }
        }
    }

    /// The entries that let nothing pass, each in words.
    pub fn stale(&self) -> Vec<String> {
        SMALL_TEXT_EXCEPTIONS
            .iter()
            .filter(|(name, _)| !self.seen.contains(name))
            .map(|(name, _)| format!("SMALL_TEXT_EXCEPTIONS' {name}: let nothing pass"))
            .collect()
    }
}

impl Drop for SmallText {
    fn drop(&mut self) {
        let stale = self.stale();
        if !stale.is_empty() && !std::thread::panicking() {
            panic!("{} stale small-text exceptions:\n{}", stale.len(), stale.join("\n"));
        }
    }
}
