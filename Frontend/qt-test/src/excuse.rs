//! A test's excuses (EmuSen_Kit.md §7.100): each finding a test does not count names its kind, the items it is
//! about, the size it holds at and who decided it; an excuse that excused nothing by the time the test ends fails the test.

use crate::mockup::Difference;
use crate::tree::Node;
use emusen_kit_core::audit::{ACCEPTED, Class, Exempted, Fault, SMALL_TEXT_EXCEPTIONS, accepted};
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
    /// The accepted class it stands for, whose row in kit-core's `ACCEPTED` gives its ruling; without that row it excuses
    /// nothing, and an excuse naming no class fails its test (EmuSen_Kit.md §7.105).
    pub class: Option<Class>,
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

/// One state's exemptions sorted by kit-core's `ACCEPTED`: the line of each it accepts, with its ruling, and each it does
/// not, which is a finding (EmuSen_Kit.md §7.102).
pub fn sort(exempted: &[Exempted]) -> (Vec<String>, Vec<Exempted>) {
    sort_refusing(exempted, &[])
}

/// As `sort`, the classes `refusing` names accepted by no row: a test narrows the table this way, never widens it.
pub fn sort_refusing(exempted: &[Exempted], refusing: &[Class]) -> (Vec<String>, Vec<Exempted>) {
    let mut listed = Vec::new();
    let mut refused = Vec::new();
    for e in exempted {
        match e.accepted().filter(|a| !refusing.contains(&a.class)) {
            Some(a) => listed.push(format!("accepted by {}: {e}", a.ruling)),
            None => refused.push(e.clone()),
        }
    }
    (listed, refused)
}

/// A test's excuses, how often each excused a finding, and how often one met its finding on an item it could not tell
/// from others; dropped with one unused, the test fails. It holds the table of accepted classes its exemptions are
/// sorted by, and lists each it accepted.
#[derive(Debug, Default)]
pub struct Excuses {
    list: Vec<(Excuse, usize, usize)>,
    refusing: Vec<Class>,
    listed: Vec<String>,
    gate: Option<&'static str>,
}

/// The folder every gate lists what a ruling accepted in, one file a gate (EmuSen_Kit.md §7.102, §7.105).
pub fn accepted_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".cache/emusen/tmp/accepted")
}

/// Where a gate lists what a ruling accepted, in `dir`: one file a gate, so each gate's list survives the others' runs.
pub fn accepted_list(dir: &std::path::Path, gate: &str) -> std::path::PathBuf {
    dir.join(format!("accepted-{gate}.txt"))
}

/// Writes a gate's list, printing it as well; a list that cannot be written fails the gate.
pub fn write_accepted(dir: &std::path::Path, gate: &str, lines: &[String]) {
    let file = accepted_list(dir, gate);
    let written = std::fs::create_dir_all(dir).and_then(|()| std::fs::write(&file, lines.join("\n")));
    if let Err(e) = written {
        panic!("{gate}'s accepted list not written to {}: {e}", file.display());
    }
    println!("{} accepted, listed in {}:\n{}", lines.len(), file.display(), lines.join("\n"));
}

/// Where a test program's counts of the test-only lines are listed in `dir`.
pub fn tests_only_list(dir: &std::path::Path, program: &str) -> std::path::PathBuf {
    let name: String = program.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '-' }).collect();
    accepted_list(dir, &format!("tests-only-{name}"))
}

/// Adds one test's count of each test-only line (`harness::QT_TESTS_ONLY`) to its test program's list in `dir`, so a gate
/// whose runner shows a passing test's output to no one still leaves, per program, how many and under which ruling
/// (EmuSen_Kit.md §7.105). The list starts afresh in each run; runs of one program's tests at once wait on its lock.
pub fn tally_tests_only(dir: &std::path::Path, program: &str, run: &str, test: &str, seen: &[(&str, &str, usize)]) {
    use std::io::{Read, Seek, Write};
    let file = tests_only_list(dir, program);
    let written = std::fs::create_dir_all(dir).and_then(|()| {
        let mut f = std::fs::File::options().read(true).write(true).create(true).truncate(false).open(&file)?;
        f.lock()?;
        let mut was = String::new();
        f.read_to_string(&mut was)?;
        let head = format!("run {run}");
        let mut tests: Vec<String> = if was.lines().next() == Some(head.as_str()) {
            was.lines().filter(|l| l.starts_with("test ")).map(str::to_owned).collect()
        } else {
            Vec::new()
        };
        tests.extend(seen.iter().map(|(_, text, n)| format!("test {test}: {n} × {text}")));
        let mut totals = Vec::new();
        for (ruling, text, _) in seen {
            let sum: usize = tests.iter().filter_map(|l| l.strip_suffix(&format!(" × {text}"))?.rsplit(": ").next()?.parse::<usize>().ok()).sum();
            totals.push(format!("accepted in tests, {ruling}: {sum} × {text}"));
        }
        let body = format!("{head}\n{}\n{}\n", totals.join("\n"), tests.join("\n"));
        f.set_len(0)?;
        f.rewind()?;
        f.write_all(body.as_bytes())
    });
    if let Err(e) = written {
        panic!("{program}'s test-only count not written to {}: {e}", file.display());
    }
}

impl Excuses {
    /// Fails, naming each, on an excuse that names no class of the table.
    pub fn new(list: &[Excuse]) -> Excuses {
        let classless: Vec<String> = list.iter().filter(|e| e.class.is_none()).map(|e| format!("{} on {}", e.finding, e.item)).collect();
        assert!(classless.is_empty(), "{} excuses name no class of kit-core's table:\n{}", classless.len(), classless.join("\n"));
        Excuses { list: list.iter().cloned().map(|e| (e, 0, 0)).collect(), refusing: Vec::new(), listed: Vec::new(), gate: None }
    }

    /// No excuse: every finding counts.
    pub fn none() -> Excuses {
        Excuses::default()
    }

    /// The same, its accepted lines written to `gate`'s list in `accepted_dir` when it is dropped.
    pub fn listing(mut self, gate: &'static str) -> Excuses {
        self.gate = Some(gate);
        self
    }

    /// The same, accepting none of kit-core's classes: every exemption is a finding.
    pub fn refusing_all(mut self) -> Excuses {
        self.refusing = ACCEPTED.iter().map(|a| a.class).collect();
        self
    }

    /// The same, accepting no class of `classes`; the table is narrowed, never widened.
    pub fn refusing(mut self, classes: &[Class]) -> Excuses {
        self.refusing = classes.to_vec();
        self
    }

    /// One state's exemptions: those the table accepts are listed under `state`, the others returned as findings.
    pub fn sort(&mut self, state: &str, exempted: &[Exempted]) -> Vec<Exempted> {
        let (accepted, refused) = sort_refusing(exempted, &self.refusing);
        self.listed.extend(accepted.into_iter().map(|l| format!("{state}: {l}")));
        refused
    }

    /// Every exemption accepted so far, each with its state and its ruling.
    pub fn listed(&self) -> &[String] {
        &self.listed
    }

    /// Whether one of the excuses names this finding at this size; the first that does is counted used and listed with its
    /// class's ruling. An excuse whose class no row accepts excuses nothing.
    pub fn excuses(&mut self, finding: &Finding, size: (i32, i32)) -> bool {
        for (excuse, used, ambiguous) in &mut self.list {
            if !excuse.size.is_none_or(|s| s == size) {
                continue;
            }
            let Some(row) = excuse.class.and_then(accepted).filter(|a| !self.refusing.contains(&a.class)) else { continue };
            match names(excuse, finding) {
                Verdict::Names => {
                    *used += 1;
                    self.listed.push(format!("{}x{}: accepted by {}: {}", size.0, size.1, row.ruling, finding.line));
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
        if std::thread::panicking() {
            return;
        }
        let stale = self.stale();
        if !stale.is_empty() {
            panic!("{} stale excuses:\n{}", stale.len(), stale.join("\n"));
        }
        if let Some(gate) = self.gate {
            write_accepted(&accepted_dir(), gate, &self.listed);
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
            let entry = SMALL_TEXT_EXCEPTIONS.iter().map(|(_, n, _)| *n).find(|n| *n == name);
            if let Some(entry) = entry.filter(|e| !self.seen.contains(e)) {
                self.seen.push(entry);
            }
        }
    }

    /// The entries that let nothing pass, each in words.
    pub fn stale(&self) -> Vec<String> {
        SMALL_TEXT_EXCEPTIONS
            .iter()
            .filter(|(_, name, _)| !self.seen.contains(name))
            .map(|(_, name, _)| format!("SMALL_TEXT_EXCEPTIONS' {name}: let nothing pass"))
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
