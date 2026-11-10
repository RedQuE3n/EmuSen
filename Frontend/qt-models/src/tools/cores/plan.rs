//! The marks a core is offered, the summary and plan they make, and the jobs that carry them out (EmuSen_Cores.md §2.1,
//! §6.2), as C#'s `CoresPlan`.

use super::row::Core;
use super::words::size;
use crate::tools::Mark;
use serde_json::{json, Value};

pub const INSTALL: &str = "install";
pub const UPDATE: &str = "update";
pub const REMOVE: &str = "remove";

fn mark(key: &str) -> Mark {
    Mark { key: key.to_owned(), label: key.to_owned() }
}

/// A project core is offered nothing, being the installation's; a core the player put in their folder is never removed
/// by Cores.
pub fn marks_for(row: &Core) -> Vec<Mark> {
    if !row.is_libretro() {
        return Vec::new();
    }
    match row.state.as_str() {
        "available" => vec![mark(INSTALL)],
        "update" => vec![mark(UPDATE), mark(REMOVE)],
        "installed" => vec![mark(REMOVE)],
        _ => Vec::new(),
    }
}

/// The mark Update All gives every core with an update.
pub fn update_mark() -> Mark {
    mark(UPDATE)
}

/// Every core offered an update, in the list's order: the rows Update All plans (EmuSen_Interface.md §15.5).
pub fn with_updates(rows: &[Core]) -> Vec<&Core> {
    rows.iter().filter(|r| marks_for(r).iter().any(|m| m.key == UPDATE)).collect()
}

/// Update All's title: one core by its name, as a plan of one update is titled, several by their count.
pub fn update_all_title(rows: &[&Core]) -> String {
    match rows {
        [row] => format!("Update {}?", row.name),
        _ => format!("Update {} cores?", rows.len()),
    }
}

/// The marked rows, each with its mark.
pub type Marked<'a> = [(&'a Core, &'a Mark)];

fn of<'a>(marked: &'a Marked<'a>, key: &str) -> Vec<&'a Core> {
    marked.iter().filter(|(_, m)| m.key == key).map(|(r, _)| *r).collect()
}

fn sized(row: &Core) -> String {
    match row.size {
        Some(n) if n > 0 => format!("{} ({})", row.name, size(n)),
        _ => row.name.clone(),
    }
}

fn names(rows: &[&Core]) -> String {
    rows.iter().map(|r| r.name.as_str()).collect::<Vec<_>>().join(", ")
}

/// The apply bar's line: the count per mark and the bytes to download.
pub fn summary(marked: &Marked<'_>) -> String {
    let parts: Vec<String> = [INSTALL, UPDATE, REMOVE].iter().filter_map(|k| Some(of(marked, k).len()).filter(|n| *n > 0).map(|n| format!("{k} {n}"))).collect();
    let bytes: u64 = of(marked, INSTALL).iter().chain(of(marked, UPDATE).iter()).map(|r| r.size.unwrap_or(0)).sum();
    let download = if bytes > 0 { format!(", about {} to download", size(bytes)) } else { String::new() };
    format!("{} marked: {}{download}", marked.len(), parts.join(", "))
}

/// The plan's title: one marked core by its action and name, several by their count (EmuSen_Interface.md §15.6).
pub fn title(marked: &Marked<'_>) -> String {
    match marked {
        [(row, mark)] => format!("{} {}?", super::words::capital(&mark.label), row.name),
        _ => format!("Apply {} changes?", marked.len()),
    }
}

/// What will happen, before anything runs, a sentence each.
pub fn sentences(marked: &Marked<'_>) -> Vec<String> {
    let (install, update, remove) = (of(marked, INSTALL), of(marked, UPDATE), of(marked, REMOVE));
    let mut lines = Vec::new();
    if !install.is_empty() {
        let list: Vec<String> = install.iter().map(|r| sized(r)).collect();
        let noun = if install.len() == 1 { "core" } else { "cores" };
        lines.push(format!("Download and install {} {noun} from buildbot.libretro.com: {}.", install.len(), list.join(", ")));
    }
    if !update.is_empty() {
        lines.push(format!("Update {}; the build installed now is kept for Roll Back.", names(&update)));
    }
    if install.len() + update.len() > 0 {
        lines.push("Each core is checked before it is kept, and nothing downloaded runs until you approve it.".to_owned());
    }
    if !remove.is_empty() {
        lines.push(format!("Remove {}. Your saves and their settings are kept.", names(&remove)));
    }
    lines
}

/// One job of a confirmed plan: its request, what it is doing in words, and the rows it works on, by key.
#[derive(Clone, Debug, PartialEq)]
pub struct Step {
    pub request: Value,
    pub doing: &'static str,
    pub rows: Vec<String>,
}

/// The jobs in the order they run: installs, updates, then removals, one job each.
pub fn steps(marked: &Marked<'_>) -> Vec<Step> {
    [(INSTALL, "core-get", "Installing"), (UPDATE, "core-update", "Updating"), (REMOVE, "core-rm", "Removing")]
        .iter()
        .filter_map(|(mark, op, doing)| {
            let rows = of(marked, mark);
            (!rows.is_empty()).then(|| Step {
                request: json!({ "op": op, "cores": rows.iter().map(|r| r.stem.clone().unwrap_or_default()).collect::<Vec<_>>() }),
                doing,
                rows: rows.iter().map(|r| r.key.clone()).collect(),
            })
        })
        .collect()
}
