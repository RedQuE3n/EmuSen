//! `impact`: the units a change can affect, one per line with the rule that chose it. The rules, in order: recording
//! (a changed recording selects its owner), always (a test class's own file), declared (Testing/inputs.json's lines),
//! covered (the classes the coverage map saw execute a changed .cs file), graph (the reverse closure over the cargo,
//! MSBuild and host edges, narrowed by the lines where no map answers), and closed: a file no rule claims selects the
//! tier's whole set, never nothing. See EmuSen_Testing.md §3.2 and `man impact`.

use crate::catalogue::Snapshot;
use crate::coverage::Map;
use crate::config::{LineKind, Tier};
use crate::repo::{parent, Repo};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pick {
    pub kind: String,
    pub unit: String,
    pub reason: String,
    pub via: String,
}

impl Pick {
    pub fn line(&self) -> String {
        format!("{}\t{}\t{}\t{}", self.kind, self.unit, self.reason, self.via)
    }

    pub fn id(&self) -> String {
        format!("{}:{}", self.kind, self.unit)
    }

    /// A selection line back into a pick; a line of spaces rather than tabs is read by its first two words.
    pub fn parse(line: &str) -> Option<Pick> {
        let line = line.trim_end();
        if line.is_empty() || line.starts_with('#') {
            return None;
        }
        let parts: Vec<&str> = if line.contains('\t') { line.split('\t').collect() } else { line.split_whitespace().collect() };
        let kind = parts.first()?.to_string();
        if !matches!(kind.as_str(), "cargo" | "dotnet" | "job") {
            return None;
        }
        Some(Pick { kind, unit: parts.get(1)?.to_string(), reason: parts.get(2).unwrap_or(&"given").to_string(), via: parts.get(3..).map(|v| v.join(" ")).unwrap_or_default() })
    }
}

/// The selection, kept in the order rules added it, the first reason for a unit kept.
#[derive(Default)]
pub struct Selection {
    pub picks: Vec<Pick>,
    seen: BTreeSet<String>,
    pub notes: Vec<String>,
}

impl Selection {
    fn add(&mut self, kind: &str, unit: &str, reason: &str, via: &str) {
        let id = format!("{kind}:{unit}");
        if self.seen.insert(id) {
            self.picks.push(Pick { kind: kind.into(), unit: unit.into(), reason: reason.into(), via: via.into() });
        }
    }
}

pub struct Ask<'a> {
    pub files: Vec<String>,
    pub tier: &'a Tier,
    pub failed_last: Vec<String>,
    /// The coverage map, when the store has a fresh one.
    pub map: Option<&'a Map>,
    /// Filter terms given on the command line, each selecting the classes it names.
    pub also: Vec<String>,
    /// The files the change adds: a new .cs file the map never saw selects through the changed files naming its types.
    pub added: BTreeSet<String>,
}

/// The types a C# text declares.
fn declared_types(text: &str) -> Vec<String> {
    regex::Regex::new(r"\b(?:class|struct|record|interface|enum)\s+([A-Z][A-Za-z0-9_]*)").unwrap().captures_iter(text).map(|c| c[1].to_string()).collect()
}

/// For each added .cs file the map never saw, the changed files the map has seen that name its types, directly or
/// through other added files; an added file nothing names is absent, and falls to the graph.
fn new_files_through(repo: &Repo, snap: &Snapshot, ask: &Ask) -> BTreeMap<String, BTreeSet<String>> {
    let Some(map) = ask.map else { return BTreeMap::new() };
    let new: Vec<&String> = ask.files.iter().filter(|f| f.ends_with(".cs") && ask.added.contains(*f) && map.classes_of(f).is_none()).collect();
    if new.is_empty() {
        return BTreeMap::new();
    }
    let seen: Vec<&String> = ask.files.iter().filter(|f| f.ends_with(".cs") && !ask.added.contains(*f) && map.classes_of(f).is_some()).collect();
    let paths: Vec<String> = new.iter().chain(seen.iter()).map(|f| (*f).clone()).collect();
    let mut texts = repo.read_many(&snap.rev, &paths).unwrap_or_default();
    for p in &paths {
        if !texts.contains_key(p) {
            if let Ok(t) = std::fs::read_to_string(repo.root.join(p)) {
                texts.insert(p.clone(), t);
            }
        }
    }
    let names = |text: &str, types: &[String]| types.iter().any(|t| regex::Regex::new(&format!(r"\b{}\b", regex::escape(t))).is_ok_and(|re| re.is_match(text)));
    let types: BTreeMap<&String, Vec<String>> = new.iter().map(|n| (*n, texts.get(*n).map(|t| declared_types(t)).unwrap_or_default())).collect();
    let mut through: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    loop {
        let mut changed = false;
        for n in &new {
            let Some(ts) = types.get(n).filter(|t| !t.is_empty()) else { continue };
            let mut sources = through.get(*n).cloned().unwrap_or_default();
            for s in &seen {
                if !sources.contains(*s) && texts.get(*s).is_some_and(|t| names(t, ts)) {
                    sources.insert((*s).clone());
                }
            }
            for (other, their) in through.clone() {
                if &other != *n && texts.get(&other).is_some_and(|t| names(t, ts)) && !their.is_subset(&sources) {
                    sources.extend(their);
                }
            }
            if !sources.is_empty() && through.get(*n) != Some(&sources) {
                through.insert((*n).clone(), sources);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    through
}

/// Expands one select entry of a line: a filter term to the classes it names, `cargo:` to a crate, `refs:` to the test
/// files naming the types a path declares, `covered:` to the classes the map saw execute the files a path names,
/// `lunap` to the toolkit's suite. Whether the entry narrows the project it reaches: a `covered:` entry with no map,
/// or naming no file, does not.
fn expand(repo: &Repo, snap: &Snapshot, tier: &Tier, map: Option<&Map>, entry: &str, reason: &str, via: &str, out: &mut Selection) -> bool {
    if let Some(dir) = entry.strip_prefix("cargo:") {
        out.add("cargo", dir, reason, via);
    } else if let Some(spec) = entry.strip_prefix("refs:") {
        for c in refs(repo, snap, &[spec.to_string()]) {
            if !named_only(tier, &c) {
                out.add("dotnet", &c, reason, &format!("{via} (refs:{spec})"));
            }
        }
    } else if let Some(spec) = entry.strip_prefix("covered:") {
        let Some(m) = map else { return false };
        let files = pathspec_files(snap, spec);
        if files.is_empty() {
            out.notes.push(format!("covered:{spec} names no file ({via})"));
            return false;
        }
        let mut units: BTreeSet<&str> = m.unknown.iter().map(String::as_str).collect();
        for f in &files {
            units.extend(m.classes_of(f).into_iter().flatten().map(String::as_str));
        }
        for u in units {
            if snap.class_of_unit(u).is_some() {
                out.add("dotnet", u, reason, &format!("{via} (covered:{spec})"));
            }
        }
    } else if entry == "lunap" {
        out.add("dotnet", "lunap", reason, via);
    } else if !entry.is_empty() {
        let found = snap.classes_matching(entry);
        if found.is_empty() {
            out.notes.push(format!("term {entry} names no test class ({via})"));
        }
        for c in found {
            if named_only(tier, &c.fqn) && !c.fqn.rsplit('.').next().is_some_and(|short| entry.contains(short)) {
                continue;
            }
            out.add("dotnet", &c.fqn, reason, via);
        }
    }
    true
}

/// The revision's files a git pathspec names: `*` and `?` match any characters, a folder prefix matches below it.
fn pathspec_files(snap: &Snapshot, spec: &str) -> Vec<String> {
    let mut re = String::from("^");
    for c in spec.chars() {
        match c {
            '*' => re.push_str(".*"),
            '?' => re.push('.'),
            other => re.push_str(&regex::escape(&other.to_string())),
        }
    }
    re.push_str("(/.*)?$");
    let Ok(re) = regex::Regex::new(&re) else { return Vec::new() };
    snap.files.iter().filter(|f| re.is_match(f)).cloned().collect()
}

/// Whether a class runs in this tier only when a line names it.
fn named_only(tier: &Tier, fqn: &str) -> bool {
    tier.named_only.iter().any(|n| fqn == n || fqn.ends_with(&format!(".{n}")))
}

/// The test classes of the files naming a type declared under `specs`, directly or through a fixture (blast.sh's
/// `refs:` rule, kept so the two agree while they run side by side).
pub fn refs(repo: &Repo, snap: &Snapshot, specs: &[String]) -> Vec<String> {
    let types = |specs: &[String]| -> Vec<String> {
        let out = repo.git(&{
            let mut a: Vec<&str> = vec!["grep", "-h", "-o", "-E", r"\b(class|struct|record|interface|enum) +[A-Z][A-Za-z0-9_]+", &snap.rev, "--"];
            a.extend(specs.iter().map(String::as_str));
            a
        });
        let mut t: BTreeSet<String> = out.unwrap_or_default().lines().filter_map(|l| l.split_whitespace().nth(1).map(str::to_string)).collect();
        t.remove("App");
        t.remove("Program");
        t.into_iter().collect()
    };
    let projects: Vec<String> = snap.graph.projects.iter().filter(|(_, t)| **t).map(|(p, _)| format!("{}/*.cs", parent(p))).collect();
    let users = |types: &[String]| -> Vec<String> {
        if types.is_empty() {
            return Vec::new();
        }
        repo.grep_files(&snap.rev, &types.join("|"), &projects, true)
    };
    let direct = users(&types(specs));
    let fixtures: Vec<String> = direct.iter().filter(|f| f.contains("/Fixtures/")).cloned().collect();
    let mut files: BTreeSet<String> = direct.into_iter().collect();
    if !fixtures.is_empty() {
        files.extend(users(&types(&fixtures)));
    }
    let mut out = BTreeSet::new();
    for f in files.iter().filter(|f| !f.contains("/Fixtures/")) {
        for c in snap.classes_of_file(f) {
            out.insert(c.fqn.clone());
        }
    }
    out.into_iter().collect()
}

/// The tier's whole set: every unit whose tag the tier does not exclude.
fn whole_tier(snap: &Snapshot, tier: &Tier, via: &str, out: &mut Selection) {
    for r in snap.graph.roots() {
        if r.virtual_root {
            for m in snap.graph.members_of(&r.dir) {
                out.add("cargo", &m.dir, "closed", via);
            }
        } else {
            out.add("cargo", &r.dir, "closed", via);
        }
    }
    for c in snap.classes.values() {
        if !(c.nightly && tier.exclude.iter().any(|e| e == "nightly")) {
            out.add("dotnet", &c.fqn, "closed", via);
        }
    }
    if snap.has_lunap() {
        out.add("dotnet", "lunap", "closed", via);
    }
}

pub fn impact(repo: &Repo, snap: &Snapshot, ask: &Ask) -> Selection {
    let mut out = Selection::default();
    let tier = ask.tier;
    let g = &snap.graph;
    let through = new_files_through(repo, snap, ask);
    for f in &ask.files {
        let lines = snap.inputs.matching(f);
        let recording: Vec<_> = lines.iter().filter(|l| l.kind == LineKind::Recording).collect();
        let folder: Vec<_> = lines.iter().filter(|l| l.kind == LineKind::Folder).collect();
        let none = lines.iter().any(|l| l.kind == LineKind::None);
        if none && recording.is_empty() && folder.is_empty() {
            continue;
        }
        let mut claimed = !lines.is_empty();
        // recording; an owner mapped by case runs only the group the recording belongs to
        for l in &recording {
            for s in &l.select {
                let groups: Vec<(String, String)> = match ask.map {
                    Some(m) if !s.contains(':') && s != "lunap" => snap.classes_matching(s).iter().filter_map(|c| m.group_of(&c.fqn, f).map(|g| (c.fqn.clone(), g))).collect(),
                    _ => Vec::new(),
                };
                if groups.is_empty() {
                    expand(repo, snap, tier, ask.map, s, "recording", f, &mut out);
                } else {
                    for (_, g) in groups {
                        out.add("dotnet", &g, "recording", &format!("{f} (its group)"));
                    }
                }
            }
        }
        if recording.is_empty() && snap.inputs.is_recording(f) && !none {
            whole_tier(snap, tier, &format!("{f} (a recording no line owns)"), &mut out);
            continue;
        }
        // always: a test class's own file
        let own = snap.classes_of_file(f);
        for c in &own {
            out.add("dotnet", &c.fqn, "always", f);
        }
        claimed |= !own.is_empty();
        // covered: the classes that executed the file, plus every class the map has no report for; a new file, the
        // classes that executed the changed files naming it
        let covered: Option<Vec<String>> = match (ask.map.and_then(|m| m.classes_of(f)), through.get(f)) {
            (Some(units), _) => Some(units.clone()),
            (None, Some(sources)) => {
                let mut units: BTreeSet<String> = BTreeSet::new();
                for s in sources {
                    units.extend(ask.map.and_then(|m| m.classes_of(s)).into_iter().flatten().cloned());
                }
                out.notes.push(format!("{f} is new; selected through {}", sources.iter().cloned().collect::<Vec<_>>().join(", ")));
                Some(units.into_iter().collect())
            }
            (None, None) => None,
        };
        if let Some(units) = &covered {
            claimed = true;
            let mut gone = Vec::new();
            for u in units.iter().chain(ask.map.map(|m| m.unknown.iter()).into_iter().flatten()) {
                if snap.class_of_unit(u).is_some() {
                    out.add("dotnet", u, "covered", f);
                } else {
                    gone.push(u.as_str());
                }
            }
            if !gone.is_empty() {
                out.notes.push(format!("the map names {} class(es) the revision no longer has ({}): rebuild it", gone.len(), gone.join(", ")));
            }
        }
        // declared; a covered file keeps only the entries coverage says nothing about (crates, the toolkit's suite)
        let mut narrowed = covered.is_some() || !recording.is_empty();
        for l in &folder {
            for s in &l.select {
                if covered.is_some() && !(s.starts_with("cargo:") || s == "lunap") {
                    continue;
                }
                narrowed |= expand(repo, snap, tier, ask.map, s, "declared", &format!("{f} ({})", l.source), &mut out);
            }
        }
        // graph; a line on another crate the closure reaches adds its units, and such lines narrow only when every
        // crate through which the closure enters C# has one
        if let Some(owner) = g.owner(f) {
            claimed = true;
            let closure = g.reverse_closure(&owner, tier.depth);
            let mut entries_lined: Option<bool> = None;
            for n in &closure {
                if g.is_crate(n) {
                    out.add("cargo", n, "graph", f);
                    let mut lined = n == &owner && narrowed;
                    if n != &owner {
                        for l in snap.inputs.matching(&format!("{n}/")).into_iter().filter(|l| l.kind == LineKind::Folder) {
                            for s in &l.select {
                                lined |= expand(repo, snap, tier, ask.map, s, "declared", &format!("{f} via {n} ({})", l.source), &mut out);
                            }
                        }
                    }
                    if g.enters_host(n) {
                        entries_lined = Some(entries_lined.unwrap_or(true) && lined);
                    }
                }
            }
            narrowed |= entries_lined == Some(true);
            let inside_test = closure.first().is_some_and(|o| g.is_test_project(o));
            if inside_test && own.is_empty() && !narrowed && f.ends_with(".cs") {
                let found = refs(repo, snap, std::slice::from_ref(f));
                narrowed = !found.is_empty();
                for c in found {
                    if !named_only(tier, &c) {
                        out.add("dotnet", &c, "graph", &format!("{f} (refs)"));
                    }
                }
            }
            if !narrowed && !(inside_test && !own.is_empty()) {
                for tp in closure.iter().filter(|n| g.is_test_project(n)) {
                    for c in snap.classes.values().filter(|c| &c.project == tp) {
                        if !(c.nightly && tier.exclude.iter().any(|e| e == "nightly")) && !named_only(tier, &c.fqn) {
                            out.add("dotnet", &c.fqn, "graph", &format!("{f} (no line narrows {tp})"));
                        }
                    }
                }
            }
        }
        if !claimed {
            whole_tier(snap, tier, &format!("{f} (no rule claims it)"), &mut out);
        }
    }
    for u in &ask.failed_last {
        if let Some((kind, unit)) = u.split_once(':') {
            out.add(kind, unit, "always", "failed in this branch's last trial");
        }
    }
    for term in &ask.also {
        expand(repo, snap, tier, ask.map, term, "given", "--also", &mut out);
    }
    if !out.picks.is_empty() {
        for a in &tier.always {
            if let Some((kind, unit)) = a.split_once(':') {
                out.add(kind, unit, "always", &format!("tier {}", tier.name));
            }
        }
    }
    let nightly: BTreeSet<&str> = snap.classes.values().filter(|c| c.nightly).map(|c| c.fqn.as_str()).collect();
    if tier.exclude.iter().any(|e| e == "nightly") {
        out.picks.retain(|p| !(p.kind == "dotnet" && nightly.contains(p.unit.split('#').next().unwrap_or(&p.unit))));
    }
    // A class picked whole runs every group of its cases, so its groups are not picked beside it.
    let whole: BTreeSet<String> = out.picks.iter().filter(|p| p.kind == "dotnet" && !p.unit.contains('#')).map(|p| p.unit.clone()).collect();
    out.picks.retain(|p| !(p.kind == "dotnet" && p.unit.split_once('#').is_some_and(|(c, _)| whole.contains(c))));
    out
}

/// The selection in blast.sh's shape (the dotnet filter, the crates, the toolkit's suite, the files that fell to the
/// closed rule), so land.sh reads either answer the same way.
pub fn blast_format(sel: &Selection, tier: &Tier) -> String {
    let classes: Vec<String> = sel.picks.iter().filter(|p| p.kind == "dotnet" && p.unit != "lunap").map(|p| p.unit.clone()).collect();
    let mut out = String::new();
    if classes.is_empty() {
        out.push_str("dotnet filter: (none)\n");
    } else {
        out.push_str(&format!("dotnet filter: {}\n", crate::adapters::dotnet_filter(&classes, false, tier.exclude.iter().any(|e| e == "nightly"))));
    }
    for p in sel.picks.iter().filter(|p| p.kind == "cargo") {
        out.push_str(&format!("cargo: {}\n", p.unit));
    }
    if sel.picks.iter().any(|p| p.kind == "dotnet" && p.unit == "lunap") {
        out.push_str("lunap: selected\n");
    }
    let closed: BTreeSet<String> = sel.picks.iter().filter(|p| p.reason == "closed").map(|p| p.via.split(" (").next().unwrap_or(&p.via).to_string()).collect();
    for f in closed {
        out.push_str(&format!("closed: {f}\n"));
    }
    out
}

/// What blast.sh printed, as units: its crates, its filter's terms resolved to classes under its own exclusions, and
/// the files it left unmapped.
pub struct Blast {
    pub crates: BTreeSet<String>,
    pub classes: BTreeMap<String, String>,
    pub unmapped: Vec<String>,
}

pub fn read_blast(snap: &Snapshot, text: &str) -> Blast {
    let mut b = Blast { crates: BTreeSet::new(), classes: BTreeMap::new(), unmapped: Vec::new() };
    for l in text.lines() {
        if let Some(c) = l.strip_prefix("cargo: ") {
            b.crates.insert(c.trim().to_string());
        } else if let Some(u) = l.strip_prefix("unmapped: ") {
            b.unmapped.push(u.trim().to_string());
        } else if let Some(f) = l.strip_prefix("dotnet filter: ") {
            if f.trim() == "(none)" {
                continue;
            }
            let inner = f.trim_start_matches('(').split(")&").next().unwrap_or("");
            let fit_out = f.contains("FullyQualifiedName!~WindowFitAuditTests");
            for term in inner.split('|').filter_map(|t| t.strip_prefix("FullyQualifiedName~")) {
                for c in snap.classes_matching(term) {
                    if c.nightly || (fit_out && c.fqn.ends_with(".WindowFitAuditTests")) {
                        continue;
                    }
                    b.classes.entry(c.fqn.clone()).or_insert_with(|| term.to_string());
                }
            }
        }
    }
    b
}

/// Each unit of either answer as agree, impact (impact only) or blast (blast.sh only), and a summary line.
pub fn compare(sel: &Selection, blast: &Blast) -> Vec<String> {
    let mut out = Vec::new();
    let mine: BTreeMap<String, &Pick> = sel.picks.iter().map(|p| (p.id(), p)).collect();
    let mut theirs: BTreeMap<String, String> = blast.crates.iter().map(|c| (format!("cargo:{c}"), "crate".to_string())).collect();
    theirs.extend(blast.classes.iter().map(|(c, t)| (format!("dotnet:{c}"), format!("term {t}"))));
    let (mut agree, mut only_mine, mut only_theirs) = (0, 0, 0);
    for (id, p) in &mine {
        if theirs.contains_key(id) {
            agree += 1;
            out.push(format!("agree\t{}\t{}\t{}\t{}", p.kind, p.unit, p.reason, p.via));
        } else {
            only_mine += 1;
            out.push(format!("impact\t{}\t{}\t{}\t{}", p.kind, p.unit, p.reason, p.via));
        }
    }
    for (id, why) in &theirs {
        if !mine.contains_key(id) {
            only_theirs += 1;
            let (kind, unit) = id.split_once(':').unwrap_or(("", id));
            out.push(format!("blast\t{kind}\t{unit}\t{why}\t-"));
        }
    }
    for u in &blast.unmapped {
        out.push(format!("unmapped\t-\t{u}\tblast.sh\t-"));
    }
    out.push(format!("summary\tagree {agree}, impact only {only_mine}, blast only {only_theirs}, unmapped {}", blast.unmapped.len()));
    out
}
