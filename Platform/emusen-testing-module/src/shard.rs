//! `shard`: a selection's dotnet units dealt to N processes by their measured times, the longest first into the
//! least loaded, a collection's classes kept together unless spread, so a land runs the processes at once and
//! waits for the longest. See EmuSen_Testing.md §3.8 and `man shard`.

use crate::catalogue::Snapshot;
use crate::store::Store;
use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct Shard {
    pub units: Vec<String>,
    pub seconds: f64,
}

/// The seconds a unit last took, from the newest trial that recorded it (a land, or the coverage map's run).
pub fn times(store: &Store) -> BTreeMap<String, f64> {
    store
        .rows("SELECT u.unit, u.seconds FROM unit_run u JOIN (SELECT unit, MAX(trial) AS t FROM unit_run WHERE seconds IS NOT NULL GROUP BY unit) m ON m.unit = u.unit AND m.t = u.trial", &[])
        .unwrap_or_default()
        .into_iter()
        .filter_map(|r| Some((r[0].clone()?.trim_start_matches("dotnet:").to_string(), r[1].as_deref()?.parse().ok()?)))
        .collect()
}

/// Deals (unit, seconds, collection) over `n` shards: items are a unit, or a whole collection when not spread.
pub fn deal(units: &[(String, f64, Option<String>)], n: usize, spread: bool) -> Vec<Shard> {
    let n = n.max(1);
    let mut items: Vec<(Vec<String>, f64)> = Vec::new();
    let mut by_collection: BTreeMap<String, usize> = BTreeMap::new();
    for (unit, secs, collection) in units {
        match collection.as_ref().filter(|_| !spread) {
            Some(c) => {
                let i = *by_collection.entry(c.clone()).or_insert_with(|| {
                    items.push((Vec::new(), 0.0));
                    items.len() - 1
                });
                items[i].0.push(unit.clone());
                items[i].1 += secs;
            }
            None => items.push((vec![unit.clone()], *secs)),
        }
    }
    items.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let mut shards: Vec<Shard> = (0..n).map(|_| Shard::default()).collect();
    for (units, secs) in items {
        let least = (0..n).min_by(|a, b| shards[*a].seconds.partial_cmp(&shards[*b].seconds).unwrap_or(std::cmp::Ordering::Equal)).unwrap_or(0);
        shards[least].units.extend(units);
        shards[least].seconds += secs;
    }
    shards.retain(|s| !s.units.is_empty());
    shards
}

/// What a source file shows touching the machine, which separate processes share: a virtual pad every process's SDL
/// sees, a listener on a port, a fixed path under the home or `out/` that it writes, a lock on a fixed file, the
/// GPU. The patterns and the reasons, in the order tried. FixedRunFolder is not one: it claims the next free slot
/// (`NNN.lock`, FileShare.None) per process, which is what lets processes run side by side.
const MACHINE: &[(&str, &str)] = &[
    ("uinput", r"\[Uinput(Fact|Theory)|\bUinputPad\b|/dev/uinput|SdlVirtualPads|VirtualPad\("),
    ("port", r"new TcpListener\(|HttpListener|Prefixes\.Add|IPEndPoint\(|localhost:\d{4,5}|127\.0\.0\.1:\d{4,5}"),
    ("flock", r"\bflock\b|LockFile\(|LockFileEx"),
    ("gpu", r"\[Gpu(Fact|Theory)|GpuFact|GpuBench|EMUSEN_MARS_GPU|SceneGpu|VK_DRIVER_FILES"),
];
/// The profile folder (the home) is not one: tests read the ROM library and the documents through it; the config,
/// cache and data folders, and a published build, are.
const FIXED_PATH: &str = r"\.config/EmuSen|\.cache/emusen|\.local/share/EmuSen|SpecialFolder\.(ApplicationData|LocalApplicationData)|\bout/linux|\bout/mac|\bout/win";
const WRITES: &str = r"CreateDirectory\(|WriteAll(Text|Bytes|Lines)|File\.Copy\(|File\.Move\(|File\.Delete\(|Directory\.Delete\(|new FileStream\(";

/// The test project's source files that touch machine-wide state, each with its reason, at the snapshot's revision.
pub fn machine_files(repo: &crate::repo::Repo, snap: &Snapshot) -> BTreeMap<String, String> {
    let patterns: Vec<(&str, regex::Regex)> = MACHINE.iter().map(|(why, re)| (*why, regex::Regex::new(re).unwrap())).collect();
    let (fixed, writes) = (regex::Regex::new(FIXED_PATH).unwrap(), regex::Regex::new(WRITES).unwrap());
    let projects: Vec<&str> = snap.graph.projects.iter().filter(|(_, t)| **t).map(|(p, _)| crate::repo::parent(p)).collect();
    let paths: Vec<String> = snap.files.iter().filter(|f| f.ends_with(".cs") && projects.iter().any(|p| crate::repo::is_under(f, p))).cloned().collect();
    let texts = repo.read_many(&snap.rev, &paths).unwrap_or_default();
    let mut out = BTreeMap::new();
    for (path, text) in &texts {
        let why = patterns.iter().find(|(_, re)| re.is_match(text)).map(|(w, _)| w.to_string()).or_else(|| (fixed.is_match(text) && writes.is_match(text)).then(|| "fixed path".to_string()));
        if let Some(w) = why {
            out.insert(path.clone(), w);
        }
    }
    out
}

/// The units that touch machine-wide state, with the reason: by the map, a unit that executed a machine file; without
/// rows in the map, a class whose own sources are one.
pub fn machine_units(snap: &Snapshot, files: &BTreeMap<String, String>, map: Option<&crate::coverage::Map>) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    if let Some(m) = map {
        for (f, why) in files {
            for u in m.classes_of(f).into_iter().flatten() {
                out.entry(u.clone()).or_insert_with(|| format!("{why} ({f})"));
            }
        }
    }
    for c in snap.classes.values() {
        let mapped = map.is_some_and(|m| m.units.iter().any(|u| u == &c.fqn || u.starts_with(&format!("{}#", c.fqn))));
        if !mapped {
            if let Some((f, why)) = c.files.iter().find_map(|f| files.get(f).map(|w| (f, w))) {
                out.entry(c.fqn.clone()).or_insert_with(|| format!("{why} ({f}, by its own sources)"));
            }
        }
    }
    out
}

/// The shards of a selection's dotnet units, with a unit's time from the store (`default_seconds` when it has none):
/// the machine-global units one item, a collection's classes one item only when `keep_collections`.
pub fn plan(snap: &Snapshot, times: &BTreeMap<String, f64>, machine: &BTreeMap<String, String>, units: &[String], n: usize, keep_collections: bool, default_seconds: f64) -> Vec<Shard> {
    let rows: Vec<(String, f64, Option<String>)> = units
        .iter()
        .map(|u| {
            let secs = times.get(u).copied().or_else(|| u.split_once('#').and_then(|(c, _)| times.get(c).map(|t| t / 40.0))).unwrap_or(default_seconds);
            let class = snap.class_of_unit(u);
            let group = match class {
                _ if machine.contains_key(u) => Some("machine".to_string()),
                Some(c) if machine.contains_key(&c.fqn) && !u.contains('#') => Some("machine".to_string()),
                Some(c) if keep_collections => c.collection.clone(),
                _ => None,
            };
            (u.clone(), secs, group)
        })
        .collect();
    deal(&rows, n, false)
}
