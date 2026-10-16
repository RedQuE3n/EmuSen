//! The commands that remember something between calls in one shell: `search`'s candidates and the snapshots `snapshot`
//! takes and `diff` compares, kept in the call's per-shell slot rather than in the module (§6.3).

use crate::memory::{find, read, side_effects, Space};
use crate::{session, Run};
use dianash_module::{Answer, Call};
use emusen_dotnet::text::to_lower;
use emusen_session_service::words::parse_hex;
use emusen_session_service::Session;
use std::collections::HashMap;

/// A string-keyed table that enumerates as .NET's `Dictionary<string, T>` does: in insertion order, a removed entry's
/// place taken by the next one added.
#[derive(Default)]
pub(crate) struct Ordered<T> {
    slots: Vec<Option<(String, T)>>,
    free: Vec<usize>,
}

impl<T> Ordered<T> {
    pub fn get(&self, key: &str) -> Option<&T> {
        self.slots.iter().flatten().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    pub fn set(&mut self, key: &str, value: T) {
        if let Some(slot) = self.slots.iter_mut().flatten().find(|(k, _)| k == key) {
            slot.1 = value;
            return;
        }
        match self.free.pop() {
            Some(at) => self.slots[at] = Some((key.to_string(), value)),
            None => self.slots.push(Some((key.to_string(), value))),
        }
    }

    pub fn remove(&mut self, key: &str) -> bool {
        match self.slots.iter().position(|s| s.as_ref().is_some_and(|(k, _)| k == key)) {
            Some(at) => {
                self.slots[at] = None;
                self.free.push(at);
                true
            }
            None => false,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &(String, T)> {
        self.slots.iter().flatten()
    }

    pub fn len(&self) -> usize {
        self.iter().count()
    }
}

/// `search`'s one search at a time.
#[derive(Default)]
struct Search {
    space: Option<String>,
    width: i32,
    candidates: Option<Vec<i32>>,
    last: HashMap<i32, i64>,
}

/// What this module keeps for one shell.
#[derive(Default)]
pub(crate) struct State {
    search: Search,
    snapshots: Ordered<(String, Vec<u8>)>,
}

fn state<'a>(call: &'a Call) -> &'a mut State {
    call.slot(State::default)
}

/// `ReadValue`: little-endian, `width` bytes, from a copy of the whole space.
fn value(bytes: &[u8], at: i32, width: i32) -> i64 {
    (0..width).map(|i| i64::from(bytes.get((at + i) as usize).copied().unwrap_or(0)) << (8 * i)).fold(0, |v, b| v | b)
}

/// The whole space, as the candidates' values are read from it.
fn whole(session: &Session, space: &Space) -> Result<Vec<u8>, String> {
    read(session, space, 0, space.size)
}

const NO_SEARCH: &str = "No active search - run 'search <space> <value>' first.";

pub fn search(call: &Call, parts: &[String]) -> Run {
    if parts.len() < 2 {
        return Ok(Answer::ok("Usage: search <space> <value> [<width>] | search refine <value> | search changed|unchanged|increased|decreased | search list [<count>] | search reset"));
    }
    let first = to_lower(&parts[1]);
    let st = &mut state(call).search;
    match first.as_str() {
        "reset" => {
            st.space = None;
            st.candidates = None;
            st.last.clear();
            return Ok(Answer::ok("Search cleared."));
        }
        "list" => {
            let Some(candidates) = st.candidates.as_ref() else { return Ok(Answer::ok(NO_SEARCH)) };
            let count = if parts.len() >= 3 { parse_hex(&parts[2])? } else { 20 };
            let session = session(call)?;
            let space = find(&session, st.space.as_deref().unwrap_or(""))?;
            let bytes = whole(&session, &space)?;
            let shown: Vec<String> = candidates.iter().take(count.max(0) as usize).map(|&a| format!("  0x{a:X} = 0x{:X}", value(&bytes, a, st.width))).collect();
            return Ok(Answer::ok(format!(
                "{} candidate(s) in {} (width {}), showing up to {count}:\n{}",
                candidates.len(),
                st.space.as_deref().unwrap_or(""),
                st.width,
                shown.join("\n")
            )));
        }
        "refine" => {
            if st.candidates.is_none() {
                return Ok(Answer::ok(NO_SEARCH));
            }
            if parts.len() < 3 {
                return Ok(Answer::ok("Usage: search refine <value>"));
            }
            let session = session(call)?;
            let space = find(&session, st.space.as_deref().unwrap_or(""))?;
            let target = i64::from(parse_hex(&parts[2])?) & 0xFFFF_FFFF;
            let bytes = whole(&session, &space)?;
            let width = st.width;
            let kept: Vec<i32> = st.candidates.take().unwrap_or_default().into_iter().filter(|&a| value(&bytes, a, width) == target).collect();
            st.last = kept.iter().map(|&a| (a, value(&bytes, a, width))).collect();
            let count = kept.len();
            st.candidates = Some(kept);
            return Ok(Answer::ok(format!("{count} candidate(s) remain.")));
        }
        "changed" | "unchanged" | "increased" | "decreased" => {
            if st.candidates.is_none() {
                return Ok(Answer::ok(NO_SEARCH));
            }
            let session = session(call)?;
            let space = find(&session, st.space.as_deref().unwrap_or(""))?;
            let bytes = whole(&session, &space)?;
            let width = st.width;
            let last = std::mem::take(&mut st.last);
            let mut kept = Vec::new();
            for a in st.candidates.take().unwrap_or_default() {
                let now = value(&bytes, a, width);
                let Some(&then) = last.get(&a) else { return Err("The given key was not present in the dictionary.".to_string()) };
                let keep = match first.as_str() {
                    "changed" => now != then,
                    "unchanged" => now == then,
                    "increased" => now > then,
                    _ => now < then,
                };
                if keep {
                    kept.push(a);
                }
            }
            st.last = kept.iter().map(|&a| (a, value(&bytes, a, width))).collect();
            let count = kept.len();
            st.candidates = Some(kept);
            return Ok(Answer::ok(format!("{count} candidate(s) remain.")));
        }
        _ => {}
    }
    if parts.len() < 3 {
        return Ok(Answer::ok("Usage: search <space> <value> [<width>]"));
    }
    let session = session(call)?;
    let space = find(&session, &parts[1])?;
    if space.side_effects {
        return Ok(Answer::ok(side_effects(&space, "search")));
    }
    let target = i64::from(parse_hex(&parts[2])?) & 0xFFFF_FFFF;
    let width = if parts.len() >= 4 { parse_hex(&parts[3])? } else { 1 };
    if !matches!(width, 1 | 2 | 4) {
        return Ok(Answer::ok("width must be 1, 2, or 4"));
    }
    let bytes = whole(&session, &space)?;
    let candidates: Vec<i32> = (0..=space.size - width).filter(|&a| value(&bytes, a, width) == target).collect();
    st.last = candidates.iter().map(|&a| (a, target)).collect();
    st.space = Some(space.name.clone());
    st.width = width;
    let count = candidates.len();
    st.candidates = Some(candidates);
    Ok(Answer::ok(format!("{count} candidate(s) found in {} matching 0x{target:X} (width {width}).", space.name)))
}

pub fn snapshot(call: &Call, parts: &[String]) -> Run {
    if parts.len() < 2 {
        return Ok(Answer::ok("Usage: snapshot <space> <name> | snapshot list | snapshot remove <name>"));
    }
    let first = to_lower(&parts[1]);
    let snapshots = &mut state(call).snapshots;
    if first == "list" {
        if snapshots.len() == 0 {
            return Ok(Answer::ok("No saved snapshots."));
        }
        let rows: Vec<String> = snapshots.iter().map(|(name, (space, data))| format!("  {name}: {space} ({} bytes)", data.len())).collect();
        return Ok(Answer::ok(rows.join("\n")));
    }
    if first == "remove" {
        if parts.len() < 3 {
            return Ok(Answer::ok("Usage: snapshot remove <name>"));
        }
        let name = &parts[2];
        return Ok(Answer::ok(if snapshots.remove(name) { format!("Snapshot '{name}' removed.") } else { format!("No snapshot named '{name}'.") }));
    }
    if parts.len() < 3 {
        return Ok(Answer::ok("Usage: snapshot <space> <name>"));
    }
    let session = session(call)?;
    let space = find(&session, &parts[1])?;
    if space.side_effects {
        return Ok(Answer::ok(side_effects(&space, "snapshot")));
    }
    let data = whole(&session, &space)?;
    let name = &parts[2];
    let length = data.len();
    snapshots.set(name, (space.name.clone(), data));
    Ok(Answer::ok(format!("Snapshot '{name}' saved: {}, {length} bytes.", space.name)))
}

pub fn diff(call: &Call, parts: &[String]) -> Run {
    if parts.len() < 2 {
        return Ok(Answer::ok("Usage: diff <name> [<count>]"));
    }
    let name = &parts[1];
    let Some((space_name, saved)) = state(call).snapshots.get(name).map(|(s, d)| (s.clone(), d.clone())) else {
        return Ok(Answer::ok(format!("No snapshot named '{name}'. Try 'snapshot list'.")));
    };
    let session = session(call)?;
    let space = find(&session, &space_name)?;
    let count = if parts.len() >= 3 { parse_hex(&parts[2])? } else { 20 };
    let now = read(&session, &space, 0, saved.len() as i32)?;
    let changes: Vec<(usize, u8, u8)> = saved.iter().enumerate().filter_map(|(a, &old)| (now[a] != old).then_some((a, old, now[a]))).collect();
    if changes.is_empty() {
        return Ok(Answer::ok(format!("No differences from snapshot '{name}'.")));
    }
    let shown: Vec<String> = changes.iter().take(count.max(0) as usize).map(|(a, old, new)| format!("  0x{a:X}: 0x{old:02X} -> 0x{new:02X}")).collect();
    Ok(Answer::ok(format!("{} byte(s) differ from snapshot '{name}' ({space_name}), showing up to {count}:\n{}", changes.len(), shown.join("\n"))))
}
