//! Eclipse Library Manager (#68) state, real rom-check/rom-dups/games.db findings, and board projection.

use super::{
    eclipse_aside_line, eclipse_columns, eclipse_console_lookup, eclipse_format_size, eclipse_help_entry,
    eclipse_move_row, eclipse_row, eclipse_step_filter, eclipse_step_row_up, eclipse_trail, Effect,
};
use crate::card::{Pane as CardPane, Row as CardRow};
use crate::clients::ClientSource;
use crate::library::{console_name, LibrarySource};
use crate::question::{confirm, Question};
use crate::settings_shown::planet_of;
use crate::words::{grouped, now, utc};
use emusen_kit_core::input::UiButton;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;
use std::sync::Arc;

pub(crate) const HIDE_ONE: &str = "eclipse-lib-hide-one";
pub(crate) const APPLY_BATCH: &str = "eclipse-lib-apply";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum FindingKind {
    SameDump,
    SameGame,
    Broken,
    Unrecognised,
    HiddenOnly,
}

#[derive(Clone, Debug)]
pub(crate) struct DumpMember {
    pub source: String,
    pub size: u64,
    pub package: Option<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct CheckedFile {
    pub source: String,
    pub system: String,
    pub checks: Vec<(String, String)>,
}

#[derive(Clone, Debug)]
pub(crate) struct LibraryFindingRow {
    pub row_key: String,
    pub kind: FindingKind,
    pub source: String,
    pub full_path: String,
    pub filename: String,
    pub console: String,
    pub finding_text: String,
    pub group_text: String,
    pub size: u64,
    pub hidden: bool,
    pub others: Vec<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum SortBy {
    #[default]
    Finding,
    Name,
    Console,
    Size,
}

impl SortBy {
    pub fn label(self) -> &'static str {
        match self {
            SortBy::Finding => "Finding",
            SortBy::Name => "Name",
            SortBy::Console => "Console",
            SortBy::Size => "Size",
        }
    }

    pub fn step(self, by: i32) -> SortBy {
        const ALL: [SortBy; 4] = [SortBy::Finding, SortBy::Name, SortBy::Console, SortBy::Size];
        let at = ALL.iter().position(|s| *s == self).unwrap_or(0);
        ALL[eclipse_step_filter(ALL.len(), at, by)]
    }
}

pub(crate) fn check_name(check: &str) -> &str {
    match check {
        "size" => "Size",
        "sheet" => "Disc sheet",
        "archive" => "Archive",
        "chd" => "CHD",
        "header" => "Header checksum",
        other => other,
    }
}

#[derive(Default)]
pub(crate) struct EclipseLibrary {
    pub enabled: bool,
    pub client: Option<Arc<ClientSource>>,
    pub library_root: Option<String>,
    pub consoles: HashMap<String, String>,
    pub console_orbs: HashMap<String, (i32, String)>,
    pub claimants: HashMap<String, String>,
    pub hidden_paths: BTreeSet<String>,
    pub dump_groups: Vec<Vec<DumpMember>>,
    pub broken_files: Vec<CheckedFile>,
    pub scraped_files: Vec<ScrapedEntry>,
    pub rows: Vec<LibraryFindingRow>,
    pub total_files: usize,
    pub checked_once: bool,
    pub pending_queue: Vec<String>,
    pub active_filter: String,
    pub focus_zone: usize,
    pub focused_filter_idx: usize,
    pub focused_row_key: Option<String>,
    pub sort_by: SortBy,
    pub search: String,
    pub marked: BTreeMap<String, (String, bool)>,
    pub pending_single: Option<(String, bool)>,
    pub library_changed: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct ScrapedEntry {
    pub path: String,
    pub rel: String,
    pub system: String,
    pub name: String,
    pub md5: String,
    pub bytes: u64,
    pub recognised: bool,
    pub game_id: Option<i64>,
}

const FILTERS: [(&str, &str); 6] = [
    ("all", "All"),
    ("same_dump", "Same Dump"),
    ("same_game", "Same Game"),
    ("broken", "Broken or Suspect"),
    ("unrecognised", "Unrecognised"),
    ("hidden", "Hidden"),
];

fn norm_slash(p: &str) -> String {
    p.replace('\\', "/")
}

fn relative_in(root: &str, full: &str) -> Option<String> {
    let r = Path::new(root);
    let f = Path::new(full);
    let rel = f.strip_prefix(r).ok()?;
    let s = norm_slash(&rel.to_string_lossy());
    (!s.is_empty() && s != ".").then_some(s)
}

fn full_in(root: Option<&str>, source: &str) -> String {
    match root {
        Some(r) => Path::new(r).join(source).to_string_lossy().into_owned(),
        None => source.to_owned(),
    }
}

fn display_path(full_path: &str, library_root: Option<&str>, source: &str) -> String {
    if let Ok(home) = std::env::var("HOME") {
        if !home.is_empty() && full_path.starts_with(&home) {
            let tail = full_path[home.len()..].trim_start_matches('/');
            return norm_slash(&format!("~/{tail}"));
        }
    }
    if let Some(root) = library_root {
        let base = Path::new(root).file_name().and_then(|s| s.to_str()).unwrap_or("Library");
        return norm_slash(&format!("~/{base}/{source}"));
    }
    norm_slash(full_path)
}

fn native_engine_orb(engine: &str) -> Option<(i32, String)> {
    match engine.to_ascii_lowercase().as_str() {
        "moon" | "venus" | "mars" | "mercury" | "endou" | "nephrite" | "diamond" | "sapphire" => {
            Some((planet_of(engine, false, true), engine.to_owned()))
        }
        _ => None,
    }
}

fn compact_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.0} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

impl EclipseLibrary {
    pub fn attach(&mut self, platform_library: &str, cores: &str) {
        if self.client.is_some() || platform_library.is_empty() {
            return;
        }
        self.focus_zone = 1;
        if let Ok(client) = crate::clients::source_for(platform_library, cores) {
            self.library_root = client
                .settings()
                .ok()
                .and_then(|s| s.get("RomDirectory").and_then(Value::as_str).map(str::to_owned));
            self.client = Some(Arc::new(client));
        }
    }

    pub fn reload_metadata(&mut self) {
        if self.active_filter.is_empty() {
            self.active_filter = "all".to_owned();
        }
        let Some(client) = self.client.as_ref() else { return };
        self.consoles = eclipse_console_lookup(client);
        self.console_orbs.clear();
        for sys in client.systems().unwrap_or_default() {
            if let Some(orb) = sys.engines.first().and_then(|e| native_engine_orb(e)) {
                self.console_orbs.insert(console_name(&sys), orb);
            }
        }
        let games = client.games().unwrap_or_default();
        if self.total_files == 0 {
            self.total_files = games.len();
        }
        let mut system_for_rel: HashMap<String, String> = HashMap::new();
        if let Some(root) = &self.library_root {
            for g in &games {
                if let (Some(rel), Some(sys)) = (relative_in(root, &g.path), g.system.as_deref()) {
                    let name = self.consoles.get(sys).cloned().unwrap_or_else(|| sys.to_owned());
                    system_for_rel.insert(rel, name);
                }
            }
        }
        for (rel, name) in system_for_rel {
            self.consoles.entry(rel).or_insert(name);
        }

        let edits = client.library.get(&json!({ "what": "all_edits" })).unwrap_or(Value::Null);
        let mut hidden = BTreeSet::new();
        if let Some(obj) = edits.as_object() {
            for (path, fields) in obj {
                if crate::edits::flag(fields, crate::edits::HIDDEN) {
                    hidden.insert(path.clone());
                }
            }
        }
        self.hidden_paths = hidden;

        self.claimants.clear();
        if let Ok(scan) = client.scan_request() {
            if let Some(pkg_dir) = scan.packages.as_deref() {
                let roots = emusen_platform_client::packages::Roots {
                    packages: pkg_dir,
                    library: scan.roots.first().map(String::as_str),
                    compressed: scan.compressed.as_deref().unwrap_or(""),
                };
                if let Ok(index) = client.platform.package_index(roots) {
                    for pkg in index.get("packages").and_then(Value::as_array).into_iter().flatten() {
                        let folder = pkg.get("folder").and_then(Value::as_str).unwrap_or("");
                        let stem = Path::new(folder).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| folder.to_owned());
                        for claim in pkg.get("claims").and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str) {
                            self.claimants.insert(claim.to_owned(), stem.clone());
                        }
                    }
                }
            }
        }

        let found = client.scraper().and_then(|s| s.get(&json!({ "what": "found" })).ok()).unwrap_or(Value::Null);
        let mut scraped = Vec::new();
        if let (Some(root), Some(obj)) = (self.library_root.as_deref(), found.as_object()) {
            for (path, rec) in obj {
                let Some(rel) = relative_in(root, path) else { continue };
                if !Path::new(path).exists() {
                    continue;
                }
                let state = rec.get("state").and_then(Value::as_str).unwrap_or("");
                let recognised = state == "found";
                let game_id = rec.get("game_id").and_then(Value::as_i64);
                let name = rec.get("name").and_then(Value::as_str).unwrap_or("").to_owned();
                let md5 = rec.get("md5").and_then(Value::as_str).unwrap_or("").to_owned();
                let bytes = rec.get("bytes").and_then(Value::as_u64).unwrap_or(0);
                let system = rel.split('/').next().unwrap_or("").to_owned();
                scraped.push(ScrapedEntry { path: path.clone(), rel, system, name, md5, bytes, recognised, game_id });
            }
        }
        self.scraped_files = scraped;
        self.rebuild_rows();
    }

    pub fn ingest_event(&mut self, job_key: &str, event: &Value) {
        let kind = event.get("event").and_then(Value::as_str).unwrap_or("");
        match (job_key, kind) {
            ("check", "start") => {
                self.broken_files.clear();
                if let Some(files) = event.get("files").and_then(Value::as_u64) {
                    self.total_files = files as usize;
                }
            }
            ("check", "file") => {
                if event.get("state").and_then(Value::as_str) == Some("failed") {
                    let source = event.get("source").and_then(Value::as_str).unwrap_or("").to_owned();
                    let sys = event.get("system").and_then(Value::as_str).unwrap_or("").to_owned();
                    let checks = event
                        .get("failed")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                        .map(|f| (f.get("check").and_then(Value::as_str).unwrap_or("").to_owned(), f.get("words").and_then(Value::as_str).unwrap_or("").to_owned()))
                        .collect();
                    if !source.is_empty() {
                        self.broken_files.push(CheckedFile { source, system: sys, checks });
                    }
                }
            }
            ("check", "end") => {
                self.checked_once = true;
                self.rebuild_rows();
            }
            ("dups", "start") => self.dump_groups.clear(),
            ("dups", "group") => {
                let members: Vec<DumpMember> = event
                    .get("members")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(|m| {
                        let source = m.get("name").and_then(Value::as_str)?.to_owned();
                        if source.is_empty() {
                            return None;
                        }
                        let size = m.get("size").and_then(Value::as_u64).unwrap_or(0);
                        let package = m.get("package").and_then(Value::as_str).map(str::to_owned);
                        Some(DumpMember { source, size, package })
                    })
                    .collect();
                if !members.is_empty() {
                    self.dump_groups.push(members);
                }
            }
            ("dups", "end") => {
                self.checked_once = true;
                self.rebuild_rows();
            }
            _ => {}
        }
    }

    fn console_for(&self, rel: &str, sys_id: &str) -> String {
        if !sys_id.is_empty() {
            if let Some(c) = self.consoles.get(sys_id) {
                return c.clone();
            }
        }
        if let Some(c) = self.consoles.get(rel) {
            return c.clone();
        }
        let folder = rel.split('/').next().unwrap_or("");
        self.consoles.get(folder).cloned().unwrap_or_else(|| folder.to_owned())
    }

    pub fn rebuild_rows(&mut self) {
        let root = self.library_root.as_deref();
        let mut out = Vec::new();
        let mut dump_of: HashMap<String, usize> = HashMap::new();
        let mut with_finding: BTreeSet<String> = BTreeSet::new();

        let mut dump_group = 0usize;
        for group in &self.dump_groups {
            let mut seen_key = BTreeSet::new();
            let mut unique: Vec<&DumpMember> = Vec::new();
            for m in group {
                let k = m.package.clone().unwrap_or_else(|| format!("file:{}", m.source));
                if seen_key.insert(k) {
                    unique.push(m);
                }
            }
            if unique.len() < 2 {
                continue;
            }
            dump_group += 1;
            for m in group {
                dump_of.insert(m.source.clone(), dump_group);
            }
            for m in &unique {
                let full = full_in(root, &m.source);
                let filename = Path::new(&m.source).file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| m.source.clone());
                let others: Vec<String> = unique.iter().filter(|o| o.source != m.source).map(|o| o.source.clone()).collect();
                let finding_text = if others.len() == 1 {
                    "Same dump as 1 other file".to_owned()
                } else {
                    format!("Same dump as {} other files", others.len())
                };
                with_finding.insert(full.clone());
                out.push(LibraryFindingRow {
                    row_key: format!("dump:{dump_group}:{}", m.source),
                    kind: FindingKind::SameDump,
                    source: m.source.clone(),
                    full_path: full.clone(),
                    filename,
                    console: self.console_for(&m.source, ""),
                    finding_text,
                    group_text: format!("Dump {dump_group}"),
                    size: m.size,
                    hidden: self.hidden_paths.contains(&full),
                    others,
                });
            }
        }

        let mut by_game: BTreeMap<i64, Vec<&ScrapedEntry>> = BTreeMap::new();
        for s in &self.scraped_files {
            if s.recognised {
                if let Some(id) = s.game_id {
                    by_game.entry(id).or_default().push(s);
                }
            }
        }
        let mut game_group = 0usize;
        for (_gid, mut entries) in by_game {
            entries.sort_by(|a, b| a.rel.cmp(&b.rel));
            let mut seen = BTreeSet::new();
            let mut files: Vec<&ScrapedEntry> = Vec::new();
            for e in entries {
                let k = self.claimants.get(&e.path).map(|p| format!("pkg:{p}")).unwrap_or_else(|| format!("file:{}", e.rel));
                if seen.insert(k) {
                    files.push(e);
                }
            }
            let distinct_dumps: BTreeSet<String> = files
                .iter()
                .map(|s| {
                    self.claimants
                        .get(&s.path)
                        .map(|p| format!("pkg:{p}"))
                        .or_else(|| dump_of.get(&s.rel).map(|d| format!("dump:{d}")))
                        .unwrap_or_else(|| format!("{}:{}", s.md5, s.bytes))
                })
                .collect();
            if distinct_dumps.len() < 2 {
                continue;
            }
            game_group += 1;
            for s in &files {
                let filename = Path::new(&s.rel).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| s.rel.clone());
                let others: Vec<String> = files.iter().filter(|o| o.rel != s.rel).map(|o| o.rel.clone()).collect();
                let prefix = if others.len() == 1 {
                    "Same game as 1 other file".to_owned()
                } else {
                    format!("Same game as {} other files", others.len())
                };
                let finding_text = if s.name.is_empty() { prefix } else { format!("{prefix} ({})", s.name) };
                with_finding.insert(s.path.clone());
                out.push(LibraryFindingRow {
                    row_key: format!("game:{game_group}:{}", s.rel),
                    kind: FindingKind::SameGame,
                    source: s.rel.clone(),
                    full_path: s.path.clone(),
                    filename,
                    console: self.console_for(&s.rel, &s.system),
                    finding_text,
                    group_text: format!("Game {game_group}"),
                    size: s.bytes,
                    hidden: self.hidden_paths.contains(&s.path),
                    others,
                });
            }
        }

        for b in &self.broken_files {
            let full = full_in(root, &b.source);
            let filename = Path::new(&b.source).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| b.source.clone());
            let size = std::fs::metadata(&full).map(|m| m.len()).unwrap_or(0);
            let finding_text = if let Some((chk, words)) = b.checks.first() {
                format!("{}: {words}", check_name(chk))
            } else {
                "A check failed".to_owned()
            };
            with_finding.insert(full.clone());
            out.push(LibraryFindingRow {
                row_key: format!("broken:{}", b.source),
                kind: FindingKind::Broken,
                source: b.source.clone(),
                full_path: full.clone(),
                filename,
                console: self.console_for(&b.source, &b.system),
                finding_text,
                group_text: String::new(),
                size,
                hidden: self.hidden_paths.contains(&full),
                others: Vec::new(),
            });
        }

        let mut unrec: Vec<&ScrapedEntry> = self.scraped_files.iter().filter(|s| !s.recognised).collect();
        unrec.sort_by(|a, b| a.rel.cmp(&b.rel));
        for s in unrec {
            let filename = Path::new(&s.rel).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| s.rel.clone());
            with_finding.insert(s.path.clone());
            out.push(LibraryFindingRow {
                row_key: format!("unrec:{}", s.rel),
                kind: FindingKind::Unrecognised,
                source: s.rel.clone(),
                full_path: s.path.clone(),
                filename,
                console: self.console_for(&s.rel, &s.system),
                finding_text: "Not recognised by ScreenScraper (a hint, not a fault)".to_owned(),
                group_text: String::new(),
                size: s.bytes,
                hidden: self.hidden_paths.contains(&s.path),
                others: Vec::new(),
            });
        }

        for hidden_path in &self.hidden_paths {
            if with_finding.contains(hidden_path) {
                continue;
            }
            let rel = root.and_then(|r| relative_in(r, hidden_path)).unwrap_or_else(|| hidden_path.clone());
            let filename = Path::new(&rel).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| rel.clone());
            let size = std::fs::metadata(hidden_path).map(|m| m.len()).unwrap_or(0);
            out.push(LibraryFindingRow {
                row_key: format!("hidden:{rel}"),
                kind: FindingKind::HiddenOnly,
                source: rel.clone(),
                full_path: hidden_path.clone(),
                filename,
                console: self.console_for(&rel, ""),
                finding_text: "Hidden from the library".to_owned(),
                group_text: String::new(),
                size,
                hidden: true,
                others: Vec::new(),
            });
        }

        match self.sort_by {
            SortBy::Finding => {}
            SortBy::Name => out.sort_by(|a, b| a.filename.to_lowercase().cmp(&b.filename.to_lowercase())),
            SortBy::Console => out.sort_by(|a, b| a.console.to_lowercase().cmp(&b.console.to_lowercase()).then(a.filename.cmp(&b.filename))),
            SortBy::Size => out.sort_by(|a, b| b.size.cmp(&a.size)),
        }
        self.rows = out;
        self.ensure_focus();
    }

    pub fn visible_rows(&self) -> Vec<&LibraryFindingRow> {
        let q = self.search.trim().to_lowercase();
        self.rows
            .iter()
            .filter(|r| {
                let filter_ok = match self.active_filter.as_str() {
                    "same_dump" => r.kind == FindingKind::SameDump,
                    "same_game" => r.kind == FindingKind::SameGame,
                    "broken" => r.kind == FindingKind::Broken,
                    "unrecognised" => r.kind == FindingKind::Unrecognised,
                    "hidden" => r.hidden,
                    _ => r.kind != FindingKind::HiddenOnly,
                };
                filter_ok && (q.is_empty() || r.filename.to_lowercase().contains(&q) || r.console.to_lowercase().contains(&q) || r.finding_text.to_lowercase().contains(&q))
            })
            .collect()
    }

    pub fn focused_row(&self) -> Option<&LibraryFindingRow> {
        let vis = self.visible_rows();
        self.focused_row_key.as_ref().and_then(|k| vis.iter().copied().find(|r| &r.row_key == k)).or_else(|| vis.first().copied())
    }

    pub fn ensure_focus(&mut self) {
        let vis = self.visible_rows();
        self.focused_row_key = self
            .focused_row_key
            .as_ref()
            .and_then(|k| vis.iter().find(|r| &r.row_key == k).map(|r| r.row_key.clone()))
            .or_else(|| vis.first().map(|r| r.row_key.clone()));
    }

    pub fn choose_filter(&mut self, key: &str) {
        if let Some(at) = FILTERS.iter().position(|(k, _)| *k == key) {
            self.active_filter = key.to_owned();
            self.focused_filter_idx = at;
            self.focused_row_key = None;
            self.ensure_focus();
        }
    }

    pub fn step_filter(&mut self, delta: i32) {
        let next = eclipse_step_filter(FILTERS.len(), self.focused_filter_idx, delta);
        self.choose_filter(FILTERS[next].0);
    }

    pub fn toggle_mark(&mut self, row: &LibraryFindingRow) {
        if self.marked.remove(&row.full_path).is_none() {
            self.marked.insert(row.full_path.clone(), (row.filename.clone(), !row.hidden));
        }
    }

    pub fn ask_hide_single(&mut self, row: &LibraryFindingRow) -> Effect {
        let hide = !row.hidden;
        self.pending_single = Some((row.full_path.clone(), hide));
        let title = if hide {
            format!("Hide {} from the library?", row.filename)
        } else {
            format!("Show {} in the library again?", row.filename)
        };
        let mut q = confirm(&title, &["Only games.db changes: no file in the library is written, moved or removed. A hidden game is shown again from here, or by Settings ▸ Library ▸ Hidden games."]);
        q.choices[1] = if hide { "Hide" } else { "Show again" }.to_owned();
        Effect::Ask(q, HIDE_ONE.to_owned())
    }

    pub fn apply_edits(&mut self, changes: &[(String, bool)]) {
        let Some(client) = self.client.as_ref() else { return };
        let at = utc(now());
        for (path, hide) in changes {
            let val = if *hide { json!(crate::edits::YES) } else { Value::Null };
            let _ = client.library.change(&json!({
                "op": "save_edits",
                "path": path,
                "changes": { crate::edits::HIDDEN: val },
                "at": at,
            }));
        }
        self.library_changed = true;
        self.reload_metadata();
    }

    pub fn navigate(&mut self, dir: &str) -> Effect {
        let vis_keys: Vec<String> = self.visible_rows().into_iter().map(|r| r.row_key.clone()).collect();
        match dir {
            "up" if self.focus_zone == 1 => {
                eclipse_step_row_up(&vis_keys, &mut self.focused_row_key, &mut self.focus_zone);
            }
            "down" => {
                if self.focus_zone == 0 {
                    if !vis_keys.is_empty() {
                        self.focus_zone = 1;
                    }
                } else if self.focus_zone == 1 {
                    let idx = self.focused_row_key.as_ref().and_then(|k| vis_keys.iter().position(|x| x == k)).unwrap_or(0);
                    self.focused_row_key = vis_keys.get(eclipse_move_row(vis_keys.len(), idx, 1)).cloned();
                }
            }
            "left" if self.focus_zone == 0 => self.focused_filter_idx = eclipse_step_filter(FILTERS.len(), self.focused_filter_idx, -1),
            "right" if self.focus_zone == 0 => self.focused_filter_idx = eclipse_step_filter(FILTERS.len(), self.focused_filter_idx, 1),
            "accept" => {
                if self.focus_zone == 0 {
                    self.choose_filter(FILTERS[self.focused_filter_idx].0);
                    self.focus_zone = 1;
                } else if let Some(row) = self.focused_row().cloned() {
                    return self.ask_hide_single(&row);
                }
            }
            "prevFilter" => self.step_filter(-1),
            "nextFilter" => self.step_filter(1),
            _ => {}
        }
        Effect::Done
    }

    pub fn row_page(&self, key: &str) -> Vec<CardRow> {
        let Some(row) = self.rows.iter().find(|r| r.row_key == key) else { return Vec::new() };
        let mark_label = if self.marked.contains_key(&row.full_path) {
            "Unmark"
        } else if row.hidden {
            "Mark to show again"
        } else {
            "Mark to hide"
        };
        let hide_label = if row.hidden { "Show in library again…" } else { "Hide from library…" };
        let mut out = vec![
            CardRow::new("toggle-mark", mark_label).pane(CardPane::said(mark_label, "Marked files change in games.db together when you choose Apply.")),
            CardRow::new("hide-one", hide_label).pane(CardPane::said("Only games.db changes: no file in the library is written, moved or removed.", "")),
            CardRow::new("detail.console", "Console").value(&row.console).pane(CardPane::said(&row.console, "")),
            CardRow::new("detail.path", "File").value(&row.source).pane(CardPane::said(&row.source, "")),
            CardRow::new("detail.finding", "Finding").value(&row.finding_text).pane(CardPane::said(&row.finding_text, "")),
            CardRow::new("detail.size", "Size").value(&eclipse_format_size(row.size)).pane(CardPane::said(&eclipse_format_size(row.size), "")),
        ];
        if !row.others.is_empty() {
            let rel = row.others.join(", ");
            out.push(CardRow::new("detail.related", "Related files").value(&rel).pane(CardPane::said(&rel, "")));
        }
        out
    }

    pub fn list_page(&self) -> Vec<CardRow> {
        let mut out = vec![
            CardRow::new(super::SEARCH, "Search"),
            CardRow::new("sort", "Sort").value(self.sort_by.label()).steps(),
            CardRow::new("run-check", "Check every dump"),
            CardRow::new("run-dups", "Find duplicates"),
            CardRow::new("mark-dups", "Mark duplicates to hide"),
            CardRow::new("mark-broken", "Mark broken or suspect to hide"),
            CardRow::new("clear-marks", "Clear marks"),
        ];
        if !self.search.is_empty() {
            out.insert(1, CardRow::new("clear-search", "Clear search").value(&self.search));
        }
        out
    }

    pub fn apply_question(&self) -> Effect {
        if self.marked.is_empty() {
            return Effect::Ask(
                Question {
                    title: "Nothing is marked".into(),
                    sentences: vec!["Press X (More) or SELECT (List) to mark files to hide or show again, or press A on a file to hide it now.".into()],
                    caption: String::new(),
                    choices: vec!["Close".into()],
                    back: Some(0),
                    picture: String::new(),
                },
                "noop".to_owned(),
            );
        }
        let hide_names: Vec<String> = self.marked.values().filter(|(_, h)| *h).map(|(n, _)| n.clone()).collect();
        let show_names: Vec<String> = self.marked.values().filter(|(_, h)| !*h).map(|(n, _)| n.clone()).collect();
        let mut lines = Vec::new();
        if !hide_names.is_empty() {
            lines.push(format!("Hide ({}): {}", hide_names.len(), hide_names.join(", ")));
        }
        if !show_names.is_empty() {
            lines.push(format!("Show again ({}): {}", show_names.len(), show_names.join(", ")));
        }
        lines.push("Only games.db changes: no file in the library is written, moved or removed.".to_owned());
        let mut q = confirm("Apply Library Manager changes?", &lines.iter().map(String::as_str).collect::<Vec<_>>());
        q.choices[1] = "Apply".to_owned();
        Effect::Ask(q, APPLY_BATCH.to_owned())
    }

    pub fn projection(&self, running: bool) -> Value {
        let vis = self.visible_rows();
        let focused_idx = self.focused_row_key.as_ref().and_then(|k| vis.iter().position(|r| &r.row_key == k)).unwrap_or(0);
        let n_all = self.rows.iter().filter(|r| r.kind != FindingKind::HiddenOnly).count();
        let n_dump = self.rows.iter().filter(|r| r.kind == FindingKind::SameDump).count();
        let n_game = self.rows.iter().filter(|r| r.kind == FindingKind::SameGame).count();
        let n_broken = self.rows.iter().filter(|r| r.kind == FindingKind::Broken).count();
        let n_unrec = self.rows.iter().filter(|r| r.kind == FindingKind::Unrecognised).count();
        let n_hidden = self.rows.iter().filter(|r| r.hidden).count();
        let n_dup = n_dump + n_game;
        let checked_words = if running { "checking…" } else if self.checked_once { "checked today" } else { "not checked yet" };
        let subtitle = format!(
            "{} {} · ⚠ {} {} · {} {} · {} hidden · {checked_words}",
            grouped(self.total_files),
            if self.total_files == 1 { "file" } else { "files" },
            grouped(n_broken),
            if n_broken == 1 { "suspect dump" } else { "suspect dumps" },
            grouped(n_dup),
            if n_dup == 1 { "duplicate" } else { "duplicates" },
            grouped(self.hidden_paths.len()),
        );
        let counts = [n_all, n_dump, n_game, n_broken, n_unrec, n_hidden];
        let filters: Vec<Value> = FILTERS
            .iter()
            .zip(counts)
            .map(|((k, label), count)| {
                json!({
                    "key": k,
                    "label": label,
                    "countText": grouped(count),
                    "active": self.active_filter == *k,
                    "objectName": format!("filterPill.{k}"),
                    "labelObjectName": format!("filterWords.{k}"),
                    "countObjectName": format!("filterCount.{k}"),
                })
            })
            .collect();
        let (columns, col_widths) = eclipse_columns(&[("Game", 0.0, 1.3), ("Console", 0.0, 0.8), ("Finding", 0.0, 1.9), ("Group", 0.0, 0.5), ("Size", 0.0, 0.45), ("State", 0.0, 0.45)]);
        let rows: Vec<Value> = vis
            .iter()
            .enumerate()
            .map(|(i, r)| {
                let state = match self.marked.get(&r.full_path) {
                    Some((_, true)) => "to hide",
                    Some((_, false)) => "to show",
                    None if r.hidden => "hidden",
                    None => "",
                };
                let (chip_text, chip_tone) = match self.marked.get(&r.full_path) {
                    Some((_, true)) => ("To hide", 1),
                    Some((_, false)) => ("To show", 2),
                    None if r.hidden => ("Hidden", 0),
                    None => match r.kind {
                        FindingKind::Broken => ("Suspect", 1),
                        FindingKind::SameDump => ("Duplicate", 2),
                        FindingKind::SameGame => ("Region", 2),
                        FindingKind::Unrecognised | FindingKind::HiddenOnly => ("", 0),
                    },
                };
                let (main_finding, sub_finding) = match r.kind {
                    FindingKind::Unrecognised => ("Not recognised by ScreenScraper".to_owned(), "Informational hint · plays normally".to_owned()),
                    FindingKind::Broken => (r.finding_text.clone(), "Integrity check failed".to_owned()),
                    FindingKind::SameDump => (
                        r.finding_text.clone(),
                        if r.others.is_empty() { String::new() } else { format!("Matches {}", r.others.join(", ")) },
                    ),
                    FindingKind::SameGame => (
                        r.finding_text.clone(),
                        if r.others.is_empty() { String::new() } else { format!("Also {}", r.others.join(", ")) },
                    ),
                    FindingKind::HiddenOnly => (r.finding_text.clone(), "Tracked in games.db only".to_owned()),
                };
                let group_cell = if r.group_text.is_empty() { "—".to_owned() } else { r.group_text.clone() };
                let is_focused = self.focus_zone == 1 && i == focused_idx;
                let mut val = eclipse_row(
                    i,
                    &r.row_key,
                    is_focused,
                    vec![r.filename.clone(), r.console.clone(), main_finding, group_cell, compact_size(r.size), state.to_owned()],
                );
                val["cellMono"] = json!([true, false, false, true, true, false]);
                val["cellSubs"] = json!(["", "", sub_finding, "", "", ""]);
                val["statusChip"] = json!({ "text": chip_text, "toneCode": chip_tone });
                val
            })
            .collect();

        let aside = if let Some(r) = vis.get(focused_idx).copied() {
            let rel_display = display_path(&r.full_path, self.library_root.as_deref(), &r.source);
            let (eyebrow, eyebrow_tone, diag) = match r.kind {
                FindingKind::Broken => (
                    "Broken or Suspect · Dump Check",
                    1,
                    format!("{}. Verify the source archive or re-convert from the original dump.", r.finding_text),
                ),
                FindingKind::SameDump => (
                    "Duplicate Dump · Exact Match",
                    2,
                    format!("{}. Byte-identical content detected against another file in the library.", r.finding_text),
                ),
                FindingKind::SameGame => (
                    "Duplicate Game · Alternate Build",
                    2,
                    format!("{}. Matched the same game identifier across multiple dumps.", r.finding_text),
                ),
                FindingKind::Unrecognised => (
                    "Unrecognised · Metadata Hint",
                    0,
                    "No ScreenScraper record matched this file's signature. Plays normally without issue.".to_owned(),
                ),
                FindingKind::HiddenOnly => (
                    "Hidden · games.db State",
                    0,
                    "Hidden from the library views in games.db only. File remains untouched on disk.".to_owned(),
                ),
            };
            let (planet_idx, console_display) = match self.console_orbs.get(&r.console) {
                Some((p, eng)) => (*p, format!("{} ({eng})", r.console)),
                None => (-3, r.console.clone()),
            };
            let mut facts = vec![
                json!({ "label": "Console", "value": console_display, "planet": planet_idx, "mono": false }),
                json!({ "label": "File size", "value": format!("{} ({} B)", compact_size(r.size), grouped(r.size as usize)), "planet": -3, "mono": true }),
            ];
            if !r.others.is_empty() {
                facts.push(json!({ "label": "Related", "value": r.others.join(", "), "planet": -3, "mono": true }));
            }
            facts.push(json!({
                "label": "Visibility",
                "value": if r.hidden { "Hidden in library (games.db)" } else { "Shown in library (games.db)" },
                "planet": -3,
                "mono": false,
            }));
            let lines = vec![eclipse_aside_line(0, &diag, false)];
            json!({
                "structured": true,
                "eyebrow": eyebrow,
                "eyebrowToneCode": eyebrow_tone,
                "title": r.filename,
                "subtitle": rel_display,
                "subtitleMuted": false,
                "diagObjectName": "asideLine.0",
                "diagnostic": diag,
                "facts": facts,
                "contractHeading": "Read-only library guarantee",
                "contractBody": "Hiding an entry updates games.db only. No file in your library folder is ever written, moved, or deleted by Library Manager.",
                "tiles": [],
                "lines": lines,
            })
        } else {
            let msg = if running { "Checking files in the library…" } else { "No findings under this filter." };
            json!({
                "structured": false,
                "title": "Library Manager",
                "subtitle": msg,
                "subtitleMuted": true,
                "tiles": [],
                "lines": [eclipse_aside_line(0, "Only games.db changes: no file in the library is written, moved or removed.", true)],
            })
        };

        let mark_bar_text = if self.marked.is_empty() {
            format!("Nothing marked · Findings: same dump {n_dump}, same game {n_game}, broken or suspect {n_broken}, unrecognised {n_unrec}.")
        } else {
            let hide_n = self.marked.values().filter(|(_, h)| *h).count();
            let show_n = self.marked.len() - hide_n;
            let mut parts = Vec::new();
            if hide_n > 0 {
                parts.push(format!("{hide_n} to hide"));
            }
            if show_n > 0 {
                parts.push(format!("{show_n} to show again"));
            }
            format!("{} · Only games.db changes on Apply.", parts.join(", "))
        };

        let a_label = if vis.get(focused_idx).is_some_and(|r| r.hidden) { "Show…" } else { "Hide…" };
        let mut help = Vec::new();
        eclipse_help_entry(&mut help, UiButton::Accept, None, a_label);
        eclipse_help_entry(&mut help, UiButton::Screensaver, None, "Mark");
        eclipse_help_entry(&mut help, UiButton::PageUp, Some(UiButton::PageDown), "Filter");
        eclipse_help_entry(&mut help, UiButton::Options, None, "Sort & jobs");
        eclipse_help_entry(&mut help, UiButton::Menu, None, "Apply marked…");
        eclipse_help_entry(&mut help, UiButton::Back, None, "Back");
        let counter = if vis.is_empty() { "0 of 0".to_owned() } else { format!("{} of {}", grouped(focused_idx + 1), grouped(vis.len())) };
        json!({
            "mode": "library",
            "statusOpen": false,
            "percentSuffix": "%",
            "trail": eclipse_trail(&["Tools", "Library Manager"]),
            "title": "Library Manager",
            "subtitle": subtitle,
            "filters": filters,
            "columns": columns,
            "colWidths": col_widths,
            "rows": rows,
            "emptyMessage": "No findings match this filter.",
            "focusedIndex": if vis.is_empty() { -1 } else { focused_idx as i32 },
            "focusedZone": self.focus_zone,
            "focusedFilterIndex": self.focused_filter_idx,
            "focusedToggleIndex": 0,
            "focusedAsideKey": self.focused_row_key.clone().unwrap_or_default(),
            "aside": aside,
            "markBarText": mark_bar_text,
            "toggles": [],
            "footerHelp": help,
            "actionLines": "more Screensaver\nlist Options\napply Menu\nprevFilter PageUp\nnextFilter PageDown\n",
            "counter": counter,
            "asideObjectName": "libraryAside",
            "asideTitleObjectName": "asideHeading",
            "asideSubObjectName": "asidePath",
            "markBarObjectName": "libraryMarkBar",
            "markBarTextObjectName": "libraryMarkSummary",
            "footerObjectName": "libraryFooter",
            "helpObjectName": "libraryHelp",
            "counterObjectName": "libraryCounter",
        })
    }
}
