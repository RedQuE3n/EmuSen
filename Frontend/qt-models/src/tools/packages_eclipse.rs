// Eclipse Packages workspace (#22 Library, #23 Layers, #69 Package Page) over library-tool and Galaxia manifests.

use super::{Action, Effect};
use crate::clients::ClientSource;
use crate::library::{title_of, LibrarySource};
use crate::question::{Question, confirm};
use crate::tools::cores::OPEN_FOLDER;
use emusen_kit_core::input::UiButton;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub const SCREEN_LIBRARY: i32 = 5;
pub const SCREEN_LAYERS: i32 = 6;
pub const SCREEN_PAGE: i32 = 7;

pub const CHOOSE_PATCH_FILE: &str = "packages:choosePatchFile";
const ASK_BATCH: &str = "pkg:batch";
const ASK_SINGLE: &str = "pkg:single";
const ASK_REMOVE_PATCH: &str = "pkg:rm-patch";
const ASK_INFO: &str = "pkg:info";

const DISC_EXTENSIONS: [&str; 6] = ["cue", "chd", "iso", "cso", "m3u", "gdi"];
const PATCH_FILTER_LABEL: &str = "Patches (*.ips *.bps *.ups)";
const ALL_FILES_FILTER_LABEL: &str = "All files (*)";

const FILTERS: [(&str, &str); 6] = [
    ("all", "All games"),
    ("packaged", "Packaged"),
    ("unpackaged", "Not packaged"),
    ("convertible", "Discs to convert"),
    ("patched", "With soft patches"),
    ("needs_check", "Needs a check"),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum JobKind {
    Convert,
    MakeLinked,
    MakeSelfContained,
    Check,
    Zip,
    Migrate,
}

impl JobKind {
    pub fn from_key(key: &str) -> Option<Self> {
        match key {
            "convert" => Some(Self::Convert),
            "make-linked" => Some(Self::MakeLinked),
            "make-self" => Some(Self::MakeSelfContained),
            "check" => Some(Self::Check),
            "zip" => Some(Self::Zip),
            "migrate" => Some(Self::Migrate),
            _ => None,
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Self::Convert => "convert",
            Self::MakeLinked => "make-linked",
            Self::MakeSelfContained => "make-self",
            Self::Check => "check",
            Self::Zip => "zip",
            Self::Migrate => "migrate",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::Convert => "Convert to CHD",
            Self::MakeLinked => "Make linked",
            Self::MakeSelfContained => "Make self-contained",
            Self::Check => "Verify package",
            Self::Zip => "Write portable .zip",
            Self::Migrate => "Copy stores in",
        }
    }

    pub fn short_tag(self) -> &'static str {
        match self {
            Self::Convert => "Convert",
            Self::MakeLinked => "Package",
            Self::MakeSelfContained => "Self-contained",
            Self::Check => "Check",
            Self::Zip => "Zip",
            Self::Migrate => "Stores",
        }
    }

    pub fn changes_library(self) -> bool {
        matches!(self, Self::Convert | Self::MakeLinked | Self::MakeSelfContained | Self::Check)
    }

    pub fn request(self, target: &str) -> Value {
        match self {
            Self::Convert => json!({ "op": "convert", "paths": [target] }),
            Self::MakeLinked => json!({ "op": "pkg-make", "paths": [target], "self_contained": false, "zip": false }),
            Self::MakeSelfContained => json!({ "op": "pkg-make", "paths": [target], "self_contained": true, "zip": false }),
            Self::Check => json!({ "op": "pkg-check", "paths": [target] }),
            Self::Zip => json!({ "op": "pkg-zip", "paths": [target] }),
            Self::Migrate => json!({ "op": "pkg-migrate", "paths": [target] }),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct PatchLayer {
    pub index: usize,
    pub file: String,
    pub name: String,
    pub format: String,
    pub sha1: String,
    pub source_crc32: Option<String>,
    pub target_crc32: Option<String>,
    pub source_sha1: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct PackageEntry {
    pub key: String,
    pub packaged: bool,
    pub pkg_rel: String,
    pub pkg_full_dir: String,
    pub game_rel: String,
    pub game_full_path: String,
    pub title: String,
    pub system: String,
    pub console: String,
    pub core_name: String,
    pub mode: String,
    pub state: String,
    pub state_words: String,
    pub last_run: String,
    pub folder_bytes: u64,
    pub game_bytes: u64,
    pub original_bytes: u64,
    pub chd_bytes: u64,
    pub convertible: bool,
    pub is_chd: bool,
    pub is_disc: bool,
    pub has_zip: bool,
    pub retroarch_compat: bool,
    pub identity: String,
    pub dump_ext: String,
    pub dump_sha1: String,
    pub dump_crc32: String,
    pub patched_size: u64,
    pub patched_crc32: String,
    pub patches: Vec<PatchLayer>,
}

impl PackageEntry {
    pub fn display_bytes(&self) -> u64 {
        if self.packaged { self.folder_bytes.max(self.game_bytes) } else { self.game_bytes }
    }

    pub fn needs_check(&self) -> bool {
        self.packaged && !matches!(self.state.as_str(), "ok" | "verified")
    }

    pub fn can_run(&self, kind: JobKind) -> bool {
        match kind {
            JobKind::Convert => self.convertible,
            JobKind::MakeLinked => !self.packaged,
            JobKind::MakeSelfContained => self.mode != "self-contained",
            JobKind::Check | JobKind::Migrate => self.packaged,
            JobKind::Zip => self.packaged && self.mode == "self-contained",
        }
    }

    pub fn smart_default_job(&self) -> Option<JobKind> {
        if self.convertible {
            Some(JobKind::Convert)
        } else if !self.packaged {
            Some(JobKind::MakeLinked)
        } else if self.needs_check() {
            Some(JobKind::Check)
        } else {
            Some(JobKind::Check)
        }
    }

    pub fn job_target(&self, kind: JobKind) -> String {
        match kind {
            JobKind::Convert => {
                if !self.game_rel.is_empty() { self.game_rel.clone() } else { self.pkg_rel.clone() }
            }
            JobKind::MakeLinked | JobKind::MakeSelfContained if !self.packaged => self.game_rel.clone(),
            _ => {
                if !self.pkg_rel.is_empty() { self.pkg_rel.clone() } else { self.game_rel.clone() }
            }
        }
    }

    pub fn open_folder_target(&self) -> Option<String> {
        if self.packaged && !self.pkg_full_dir.is_empty() {
            return Some(self.pkg_full_dir.clone());
        }
        if !self.game_full_path.is_empty() {
            return Path::new(&self.game_full_path).parent().map(|p| p.to_string_lossy().into_owned());
        }
        None
    }

    fn format_label(&self) -> String {
        if self.convertible {
            return "CUE/BIN disc".to_owned();
        }
        if self.is_chd {
            return "CHD disc".to_owned();
        }
        let ext = if self.dump_ext.is_empty() { "ROM" } else { self.dump_ext.as_str() };
        if self.is_disc { format!("{ext} disc") } else { format!("{ext} cartridge") }
    }

    fn status_label(&self) -> String {
        match self.state.as_str() {
            "ok" | "verified" => "Healthy".to_owned(),
            "changed" => "Changed dump".to_owned(),
            "missing" => "Missing".to_owned(),
            "moved" => "Moved".to_owned(),
            "refused" => "Refused".to_owned(),
            _ if !self.packaged => "Not packaged".to_owned(),
            other => title_case_word(other),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PendingExec {
    pub op_name: String,
    pub entry_key: String,
    pub request: Value,
    pub changes_library: bool,
    pub is_scan: bool,
    pub batch_key: Option<String>,
}

#[derive(Clone, Debug)]
enum ReorderPhase {
    Removing { remaining_index: usize },
    Adding { next_pos: usize },
    RollingClean { remaining_index: usize },
    RollingRestore { next_pos: usize },
}

#[derive(Clone, Debug)]
struct ReorderRun {
    entry_key: String,
    pkg_rel: String,
    low: usize,
    staged_dir: PathBuf,
    desired_paths: Vec<PathBuf>,
    original_paths: Vec<PathBuf>,
    phase: ReorderPhase,
    error_words: Option<String>,
}

#[derive(Clone, Debug)]
struct PendingSingle {
    entry_key: String,
    kind: JobKind,
    target: String,
}

#[derive(Clone, Debug)]
struct PendingRemovePatch {
    entry_key: String,
    pkg_rel: String,
    one_based_index: usize,
    patch_title: String,
}

#[derive(Default)]
pub(crate) struct EclipsePackages {
    pub enabled: bool,
    pub client: Option<Arc<ClientSource>>,
    pub library_root: Option<String>,
    pub packages_root: Option<String>,
    pub compressed_root: Option<String>,
    pub system_names: HashMap<String, String>,
    pub core_names: HashMap<String, String>,
    pub entries: Vec<PackageEntry>,
    pub scan_active: bool,
    pub scan_buffer: Vec<PackageEntry>,
    pub scan_seen_keys: BTreeSet<String>,
    pub disk_free_bytes: u64,
    pub screen: i32,
    pub layers_return_screen: i32,
    pub active_filter: String,
    pub console_scope: String,
    pub search: String,
    pub focus_zone: usize,
    pub focused_filter_idx: usize,
    pub focused_key: Option<String>,
    pub focused_tile_idx: usize,
    pub page_key: Option<String>,
    pub focused_page_action_idx: usize,
    pub layers_key: Option<String>,
    pub layers_zone: usize,
    pub focused_stack_idx: usize,
    pub focused_layers_action_idx: usize,
    pub batch_queue: BTreeMap<String, JobKind>,
    pub pending_execs: Vec<PendingExec>,
    pub active_exec: Option<PendingExec>,
    pending_single: Option<PendingSingle>,
    pending_remove_patch: Option<PendingRemovePatch>,
    pending_patch_target: Option<(String, String)>,
    reorder: Option<ReorderRun>,
    last_error: Option<String>,
    pub notice_text: String,
    pub library_changed: bool,
    pub dialog_folder: String,
}

impl EclipsePackages {
    pub fn set_notice(&mut self, msg: &str) {
        self.notice_text = msg.to_owned();
    }
    pub fn new() -> Self {
        Self {
            screen: SCREEN_LIBRARY,
            layers_return_screen: SCREEN_LIBRARY,
            active_filter: "all".to_owned(),
            console_scope: "All consoles".to_owned(),
            focus_zone: 1,
            layers_zone: 1,
            ..Self::default()
        }
    }

    pub fn screen_code(&self) -> i32 {
        if self.screen == 0 { SCREEN_LIBRARY } else { self.screen }
    }

    pub fn attach(&mut self, platform_library: &str, cores: &str) {
        if self.client.is_some() || platform_library.is_empty() {
            return;
        }
        if let Ok(client) = crate::clients::source_for(platform_library, cores) {
            if let Ok(scan) = client.scan_request() {
                self.library_root = scan.roots.first().cloned();
                self.packages_root = scan.packages;
                self.compressed_root = scan.compressed;
            }
            if self.library_root.is_none() {
                self.library_root = client
                    .settings()
                    .ok()
                    .and_then(|s| s.get("RomDirectory").and_then(Value::as_str).map(str::to_owned));
            }
            self.load_system_metadata(&client);
            self.client = Some(Arc::new(client));
            self.refresh_disk_free();
        }
    }

    fn load_system_metadata(&mut self, client: &ClientSource) {
        let Ok(systems) = client.systems() else { return };
        let engines: HashMap<String, crate::library::Engine> = client.engines(&systems).unwrap_or_default().into_iter().collect();
        for sys in systems {
            self.system_names.insert(sys.id.to_lowercase(), sys.console.clone());
            self.system_names.insert(sys.esde_system.to_lowercase(), sys.console.clone());
            self.system_names.insert(sys.console.to_lowercase(), sys.console.clone());
            if let Some(eng) = engines.get(&sys.id) {
                let shown = if eng.name.is_empty() { &eng.id } else { &eng.name };
                self.core_names.insert(sys.console.to_lowercase(), shown.clone());
                self.core_names.insert(sys.id.to_lowercase(), shown.clone());
                self.core_names.insert(sys.esde_system.to_lowercase(), shown.clone());
            }
        }
    }

    pub fn refresh_disk_free(&mut self) {
        let probe = self
            .packages_root
            .as_deref()
            .or(self.library_root.as_deref())
            .map(Path::new);
        if let Some(p) = probe {
            self.disk_free_bytes = disk_free_bytes(p);
        }
    }

    pub fn queue_scan(&mut self) {
        if self.active_exec.as_ref().is_some_and(|e| e.is_scan)
            || self.pending_execs.iter().any(|e| e.is_scan)
        {
            return;
        }
        self.pending_execs.push(PendingExec {
            op_name: "pkg-ls".to_owned(),
            entry_key: String::new(),
            request: json!({ "op": "pkg-ls", "unpackaged": true }),
            changes_library: false,
            is_scan: true,
            batch_key: None,
        });
    }

    pub fn take_next_request(&mut self) -> Option<PendingExec> {
        if self.active_exec.is_some() {
            return None;
        }
        if let Some(req) = self.next_reorder_exec() {
            self.active_exec = Some(req.clone());
            return Some(req);
        }
        if self.pending_execs.is_empty() {
            return None;
        }
        let next = self.pending_execs.remove(0);
        if next.is_scan {
            self.scan_active = true;
            self.scan_buffer.clear();
            self.scan_seen_keys.clear();
        } else {
            self.notice_text.clear();
        }
        self.active_exec = Some(next.clone());
        Some(next)
    }

    pub fn ingest_event(&mut self, exec: &PendingExec, event: &Value) {
        let kind = event.get("event").and_then(Value::as_str).unwrap_or("");
        if exec.is_scan {
            match kind {
                "package" => {
                    if let Some(entry) = self.entry_from_package_event(event) {
                        if self.scan_seen_keys.insert(entry.key.clone()) {
                            self.scan_buffer.push(entry);
                        }
                    }
                }
                "game" => {
                    if let Some(entry) = self.entry_from_game_event(event) {
                        if self.scan_seen_keys.insert(entry.key.clone()) {
                            self.scan_buffer.push(entry);
                        }
                    }
                }
                "end" => {}
                _ => {}
            }
            return;
        }
        if kind == "end" {
            if let Some(err) = event.get("error").and_then(Value::as_str) {
                self.last_error = Some(err.to_owned());
            }
            return;
        }
        if matches!(kind, "package" | "disc" | "outcome") && !exec.entry_key.is_empty() {
            let state = event.get("state").and_then(Value::as_str).unwrap_or("");
            let words = event.get("words").and_then(Value::as_str).unwrap_or("");
            if state == "failed" || state == "refused" {
                self.last_error = Some(if words.is_empty() { state.to_owned() } else { words.to_owned() });
            }
            if let Some(entry) = self.entries.iter_mut().find(|e| e.key == exec.entry_key) {
                entry.last_run = format_run_outcome(&exec.op_name, state, words);
                if matches!(state, "verified" | "ok" | "moved" | "changed" | "missing" | "refused") {
                    entry.state = if state == "verified" { "ok".to_owned() } else { state.to_owned() };
                    if !words.is_empty() {
                        entry.state_words = words.to_owned();
                    }
                }
            }
        }
    }

    pub fn finish_exec(&mut self, exec: PendingExec, end_error: Option<String>) -> Option<Question> {
        self.active_exec = None;
        if exec.is_scan {
            self.scan_active = false;
            let mut next = std::mem::take(&mut self.scan_buffer);
            for item in &mut next {
                if let Some(prev) = self.entries.iter().find(|p| p.key == item.key) {
                    if !prev.last_run.is_empty() {
                        item.last_run = prev.last_run.clone();
                    }
                }
            }
            sort_entries(&mut next);
            self.entries = next;
            self.refresh_disk_free();
            self.ensure_selections();
            return None;
        }
        let err = end_error.or_else(|| self.last_error.take());
        if let Some(ref mut reorder) = self.reorder {
            let outcome_q = advance_reorder(reorder, err);
            if outcome_q.is_some() || matches!(reorder.phase, ReorderPhase::Adding { next_pos } if next_pos == reorder.desired_paths.len()) {
                let done_run = self.reorder.take().unwrap();
                let _ = std::fs::remove_dir_all(&done_run.staged_dir);
                self.library_changed = true;
                self.queue_scan();
                return outcome_q;
            }
            if reorder.error_words.is_some() && matches!(reorder.phase, ReorderPhase::RollingRestore { next_pos } if next_pos == reorder.original_paths.len()) {
                let done_run = self.reorder.take().unwrap();
                let _ = std::fs::remove_dir_all(&done_run.staged_dir);
                self.queue_scan();
                return Some(notice_question(
                    "Could not reorder patches",
                    &[&done_run.error_words.unwrap_or_else(|| "Patch chain check failed; original order restored.".to_owned())],
                ));
            }
            return None;
        }
        if let Some(ref bk) = exec.batch_key {
            if err.is_none() {
                self.batch_queue.remove(bk);
            }
        }
        if exec.changes_library && err.is_none() {
            self.library_changed = true;
        }
        if !exec.entry_key.is_empty() && err.is_none() {
            self.batch_queue.remove(&exec.entry_key);
        }
        if self.pending_execs.is_empty() {
            self.queue_scan();
        }
        err.map(|msg| notice_question("Package job failed", &[&msg]))
    }

    #[cfg(test)]
    pub fn entries_from_test_scan(&mut self, events: &[Value]) {
        self.scan_buffer.clear();
        self.scan_seen_keys.clear();
        for ev in events {
            let kind = ev.get("event").and_then(Value::as_str).unwrap_or("");
            match kind {
                "package" => {
                    if let Some(e) = self.entry_from_package_event(ev) {
                        if self.scan_seen_keys.insert(e.key.clone()) {
                            self.scan_buffer.push(e);
                        }
                    }
                }
                "game" => {
                    if let Some(e) = self.entry_from_game_event(ev) {
                        if self.scan_seen_keys.insert(e.key.clone()) {
                            self.scan_buffer.push(e);
                        }
                    }
                }
                _ => {}
            }
        }
        let mut next = std::mem::take(&mut self.scan_buffer);
        sort_entries(&mut next);
        self.entries = next;
        self.ensure_selections();
    }

    fn resolve_console_name(&self, raw_console: &str, raw_system: &str) -> String {
        if !raw_console.is_empty() {
            return raw_console.to_owned();
        }
        self.system_names
            .get(&raw_system.to_lowercase())
            .cloned()
            .unwrap_or_else(|| raw_system.to_owned())
    }

    fn resolve_core_name(&self, console: &str, system: &str) -> String {
        self.core_names
            .get(&console.to_lowercase())
            .or_else(|| self.core_names.get(&system.to_lowercase()))
            .cloned()
            .unwrap_or_else(|| "Default core".to_owned())
    }

    fn read_manifest_details(&self, pkg_rel: &str) -> Option<Value> {
        let pkg_root = self.packages_root.as_deref()?;
        let rel = pkg_rel.strip_prefix("Packages/").unwrap_or(pkg_rel);
        let manifest_path = Path::new(pkg_root).join(rel).join("package.json");
        let path_str = manifest_path.to_string_lossy().into_owned();
        let raw = match self.client.as_ref().and_then(|c| c.platform.package_read(&path_str).ok()) {
            Some(text) => text,
            None => std::fs::read_to_string(&manifest_path).ok()?,
        };
        serde_json::from_str(&raw).ok()
    }

    fn entry_from_package_event(&self, ev: &Value) -> Option<PackageEntry> {
        let raw_source = ev.get("package").or_else(|| ev.get("source")).and_then(Value::as_str)?;
        let pkg_rel = raw_source.strip_prefix("Packages/").unwrap_or(raw_source).to_owned();
        let key = format!("pkg:{pkg_rel}");
        let pkg_full_dir = self
            .packages_root
            .as_deref()
            .map(|r| Path::new(r).join(&pkg_rel).to_string_lossy().into_owned())
            .unwrap_or_default();
        let doc = self.read_manifest_details(&pkg_rel);
        let raw_system = ev
            .get("system")
            .and_then(Value::as_str)
            .or_else(|| doc.as_ref().and_then(|d| d.get("system")).and_then(Value::as_str))
            .unwrap_or("");
        let raw_console = ev.get("console").and_then(Value::as_str).unwrap_or("");
        let console = self.resolve_console_name(raw_console, raw_system);
        let core_name = ev
            .get("core")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .unwrap_or_else(|| self.resolve_core_name(&console, raw_system));
        let game_rel = ev
            .get("game")
            .and_then(Value::as_str)
            .or_else(|| doc.as_ref().and_then(|d| d.get("game")).and_then(|g| g.get("path")).and_then(Value::as_str))
            .unwrap_or("")
            .to_owned();
        let game_full_path = self
            .library_root
            .as_deref()
            .filter(|_| !game_rel.is_empty())
            .map(|r| Path::new(r).join(&game_rel).to_string_lossy().into_owned())
            .unwrap_or_else(|| {
                if !pkg_full_dir.is_empty() && !game_rel.is_empty() {
                    Path::new(&pkg_full_dir).join(&game_rel).to_string_lossy().into_owned()
                } else {
                    String::new()
                }
            });
        let title = ev
            .get("title")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .or_else(|| doc.as_ref().and_then(|d| d.get("title")).and_then(Value::as_str).filter(|s| !s.is_empty()))
            .map(str::to_owned)
            .unwrap_or_else(|| title_of(if game_rel.is_empty() { &pkg_rel } else { &game_rel }));
        let mode = ev
            .get("mode")
            .and_then(Value::as_str)
            .or_else(|| doc.as_ref().and_then(|d| d.get("mode")).and_then(Value::as_str))
            .unwrap_or("linked")
            .to_owned();
        let state = ev.get("state").and_then(Value::as_str).unwrap_or("ok").to_owned();
        let state_words = ev.get("words").and_then(Value::as_str).unwrap_or("").to_owned();
        let folder_bytes = ev.get("size").and_then(Value::as_u64).unwrap_or(0);
        let game_bytes = ev.get("game_size").and_then(Value::as_u64).unwrap_or(0);
        let original_bytes = ev.get("original").and_then(Value::as_u64).unwrap_or(0);
        let chd_bytes = ev.get("chd").and_then(Value::as_u64).unwrap_or(0);
        let convertible = ev.get("convertible").and_then(Value::as_bool).unwrap_or(false);
        let container_type = doc
            .as_ref()
            .and_then(|d| d.get("container"))
            .and_then(|c| c.get("type"))
            .and_then(Value::as_str)
            .unwrap_or("");
        let is_chd = container_type == "chd" || path_ext(&game_rel) == "chd" || chd_bytes > 0;
        let has_tracks = doc
            .as_ref()
            .and_then(|d| d.get("dump"))
            .and_then(|d| d.get("tracks"))
            .is_some_and(|t| !t.is_null());
        let is_disc = convertible
            || is_chd
            || has_tracks
            || ev.get("is_disc").and_then(Value::as_bool).unwrap_or(false)
            || is_disc_path(&game_rel);
        let retroarch_compat = ev.get("retroarch").and_then(Value::as_bool).unwrap_or(false);
        let identity = ev
            .get("identity")
            .and_then(Value::as_str)
            .or_else(|| doc.as_ref().and_then(|d| d.get("identity")).and_then(Value::as_str))
            .unwrap_or("original")
            .to_owned();
        let dump_ext = path_ext(if game_rel.is_empty() { &pkg_rel } else { &game_rel }).to_ascii_uppercase();
        let (dump_sha1, dump_crc32) = extract_dump_hashes(doc.as_ref(), ev);
        let (patched_size, patched_crc32) = extract_patched_info(doc.as_ref(), ev);
        let patches = extract_patches(doc.as_ref(), ev);
        let has_zip = ev.get("zip").is_some_and(|z| !z.is_null())
            || (!pkg_full_dir.is_empty() && Path::new(&format!("{pkg_full_dir}.zip")).exists());

        Some(PackageEntry {
            key,
            packaged: true,
            pkg_rel,
            pkg_full_dir,
            game_rel,
            game_full_path,
            title,
            system: raw_system.to_owned(),
            console,
            core_name,
            mode,
            state,
            state_words,
            last_run: ev.get("last_run").and_then(Value::as_str).unwrap_or("Never checked").to_owned(),
            folder_bytes,
            game_bytes,
            original_bytes,
            chd_bytes,
            convertible,
            is_chd,
            is_disc,
            has_zip,
            retroarch_compat,
            identity,
            dump_ext,
            dump_sha1,
            dump_crc32,
            patched_size,
            patched_crc32,
            patches,
        })
    }

    fn entry_from_game_event(&self, ev: &Value) -> Option<PackageEntry> {
        let game_rel = ev.get("source").and_then(Value::as_str)?.to_owned();
        let key = format!("game:{game_rel}");
        let game_full_path = self
            .library_root
            .as_deref()
            .map(|r| Path::new(r).join(&game_rel).to_string_lossy().into_owned())
            .unwrap_or_default();
        let raw_system = ev.get("system").and_then(Value::as_str).unwrap_or("");
        let raw_console = ev.get("console").and_then(Value::as_str).unwrap_or("");
        let console = self.resolve_console_name(raw_console, raw_system);
        let core_name = self.resolve_core_name(&console, raw_system);
        let title = ev
            .get("title")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .unwrap_or_else(|| title_of(&game_rel));
        let game_bytes = ev.get("size").and_then(Value::as_u64).unwrap_or(0);
        let convertible = ev.get("convertible").and_then(Value::as_bool).unwrap_or(false);
        let is_chd = path_ext(&game_rel) == "chd";
        let is_disc = convertible
            || is_chd
            || ev.get("is_disc").and_then(Value::as_bool).unwrap_or(false)
            || is_disc_path(&game_rel);
        let dump_ext = path_ext(&game_rel).to_ascii_uppercase();

        Some(PackageEntry {
            key,
            packaged: false,
            pkg_rel: String::new(),
            pkg_full_dir: String::new(),
            game_rel,
            game_full_path,
            title,
            system: raw_system.to_owned(),
            console,
            core_name,
            mode: "unpackaged".to_owned(),
            state: "unpackaged".to_owned(),
            state_words: String::new(),
            last_run: "Never checked".to_owned(),
            folder_bytes: 0,
            game_bytes,
            original_bytes: 0,
            chd_bytes: 0,
            convertible,
            is_chd,
            is_disc,
            has_zip: false,
            retroarch_compat: false,
            identity: "original".to_owned(),
            dump_ext,
            dump_sha1: String::new(),
            dump_crc32: String::new(),
            patched_size: 0,
            patched_crc32: String::new(),
            patches: Vec::new(),
        })
    }

    pub fn console_choices(&self) -> Vec<String> {
        let mut set = BTreeSet::new();
        for e in &self.entries {
            if !e.console.is_empty() {
                set.insert(e.console.clone());
            }
        }
        let mut out = vec!["All consoles".to_owned()];
        out.extend(set);
        out
    }

    fn matches_filter(entry: &PackageEntry, filter_key: &str) -> bool {
        match filter_key {
            "packaged" => entry.packaged,
            "unpackaged" => !entry.packaged,
            "convertible" => entry.convertible,
            "patched" => entry.packaged && !entry.patches.is_empty(),
            "needs_check" => entry.needs_check(),
            _ => true,
        }
    }

    fn matches_scope_and_search(&self, entry: &PackageEntry) -> bool {
        if self.console_scope != "All consoles" && entry.console != self.console_scope {
            return false;
        }
        let q = self.search.trim().to_lowercase();
        if q.is_empty() {
            return true;
        }
        entry.title.to_lowercase().contains(&q)
            || entry.console.to_lowercase().contains(&q)
            || entry.game_rel.to_lowercase().contains(&q)
            || entry.pkg_rel.to_lowercase().contains(&q)
    }

    pub fn filtered_entries(&self) -> Vec<&PackageEntry> {
        self.entries
            .iter()
            .filter(|e| self.matches_scope_and_search(e) && Self::matches_filter(e, &self.active_filter))
            .collect()
    }

    pub fn focused_entry(&self) -> Option<&PackageEntry> {
        let shown = self.filtered_entries();
        self.focused_key
            .as_deref()
            .and_then(|k| shown.iter().copied().find(|e| e.key == k))
            .or_else(|| shown.first().copied())
    }

    pub fn layers_entries(&self) -> Vec<&PackageEntry> {
        self.entries
            .iter()
            .filter(|e| e.packaged && !e.is_disc && (!e.patches.is_empty() || Some(e.key.as_str()) == self.layers_key.as_deref()))
            .collect()
    }

    pub fn active_layers_entry(&self) -> Option<&PackageEntry> {
        let shown = self.layers_entries();
        self.layers_key
            .as_deref()
            .and_then(|k| shown.iter().copied().find(|e| e.key == k))
            .or_else(|| shown.first().copied())
    }

    pub fn active_page_entry(&self) -> Option<&PackageEntry> {
        self.page_key
            .as_deref()
            .and_then(|k| self.entries.iter().find(|e| e.key == k))
            .or_else(|| self.focused_entry())
    }

    pub fn ensure_selections(&mut self) {
        let consoles = self.console_choices();
        if !consoles.contains(&self.console_scope) {
            self.console_scope = "All consoles".to_owned();
        }
        let shown_keys: Vec<String> = self.filtered_entries().into_iter().map(|e| e.key.clone()).collect();
        if let Some(ref k) = self.focused_key {
            if !shown_keys.contains(k) {
                self.focused_key = shown_keys.first().cloned();
            }
        } else {
            self.focused_key = shown_keys.first().cloned();
        }
        if self.page_key.as_ref().is_some_and(|pk| !self.entries.iter().any(|e| &e.key == pk)) {
            self.page_key = self.focused_key.clone();
        }
        let layers_keys: Vec<String> = self.layers_entries().into_iter().map(|e| e.key.clone()).collect();
        if self.layers_key.as_ref().is_none_or(|lk| !layers_keys.contains(lk)) {
            self.layers_key = layers_keys.first().cloned();
        }
        self.clamp_stack_focus();
        self.clamp_page_focus();
    }

    fn clamp_stack_focus(&mut self) {
        let count = self.active_layers_entry().map_or(0, |e| e.patches.len() + 1);
        if count == 0 {
            self.focused_stack_idx = 0;
        } else {
            self.focused_stack_idx = self.focused_stack_idx.min(count - 1);
        }
    }

    fn clamp_page_focus(&mut self) {
        let len = self.page_actions_for_active().len();
        if len == 0 {
            self.focused_page_action_idx = 0;
        } else {
            self.focused_page_action_idx = self.focused_page_action_idx.min(len - 1);
        }
    }

    pub fn page_actions_for(entry: &PackageEntry) -> Vec<Value> {
        let mut actions = Vec::new();
        if entry.packaged {
            // AUDIT.md:180: omit Make Package on already-packaged games.
            if !entry.is_disc {
                actions.push(json!({
                    "key": "layers",
                    "label": "Soft patches",
                    "value": format!("{}", entry.patches.len()),
                    "opens": true,
                    "steps": false,
                    "enabled": true,
                }));
                for (i, p) in entry.patches.iter().enumerate() {
                    let name = if p.name.is_empty() { &p.file } else { &p.name };
                    actions.push(json!({
                        "key": format!("rm-patch:{i}"),
                        "label": format!("{}. {} ({})", i + 1, name, p.format.to_ascii_uppercase()),
                        "value": "Remove",
                        "opens": false,
                        "steps": false,
                        "enabled": true,
                    }));
                }
                actions.push(json!({
                    "key": "add-patch",
                    "label": "Add a patch",
                    "value": "…",
                    "opens": true,
                    "steps": false,
                    "enabled": true,
                }));
                if !entry.patches.is_empty() {
                    let id_label = if entry.identity == "patched" { "Scrape as Patched" } else { "Scrape as Original" };
                    actions.push(json!({
                        "key": "identity",
                        "label": "Scraped as",
                        "value": id_label,
                        "opens": false,
                        "steps": true,
                        "enabled": true,
                    }));
                }
            }
            if entry.mode == "linked" {
                actions.push(json!({
                    "key": "make-self",
                    "label": "Make self-contained",
                    "value": "Copy game in",
                    "opens": false,
                    "steps": false,
                    "enabled": true,
                }));
            }
            if entry.mode == "self-contained" {
                actions.push(json!({
                    "key": "zip",
                    "label": "Write portable .zip",
                    "value": if entry.has_zip { "Refresh" } else { "Create" },
                    "opens": false,
                    "steps": false,
                    "enabled": true,
                }));
            }
            if entry.convertible {
                actions.push(json!({
                    "key": "convert",
                    "label": "Convert to CHD",
                    "value": "Compress disc",
                    "opens": false,
                    "steps": false,
                    "enabled": true,
                }));
            }
            actions.push(json!({
                "key": "check",
                "label": "Check Package",
                "value": "Verify hashes",
                "opens": false,
                "steps": false,
                "enabled": true,
            }));
            actions.push(json!({
                "key": "migrate",
                "label": "Stores",
                "value": "Copy Saves and Art In",
                "opens": false,
                "steps": false,
                "enabled": true,
            }));
            actions.push(json!({
                "key": "open-folder",
                "label": "Open Folder",
                "value": format!("Packages/{}", entry.pkg_rel),
                "opens": true,
                "steps": false,
                "enabled": entry.open_folder_target().is_some(),
            }));
        } else {
            actions.push(json!({
                "key": "make-linked",
                "label": "Make linked package",
                "value": "Manifest & saves only",
                "opens": false,
                "steps": false,
                "enabled": true,
            }));
            actions.push(json!({
                "key": "make-self",
                "label": "Make self-contained",
                "value": "Copy game into Packages/",
                "opens": false,
                "steps": false,
                "enabled": true,
            }));
            if entry.convertible {
                actions.push(json!({
                    "key": "convert",
                    "label": "Convert to CHD",
                    "value": "Compress disc",
                    "opens": false,
                    "steps": false,
                    "enabled": true,
                }));
            }
            let dir_hint = Path::new(&entry.game_rel)
                .parent()
                .map(|p| format!("~/{}", p.to_string_lossy()))
                .unwrap_or_else(|| "~".to_owned());
            actions.push(json!({
                "key": "open-folder",
                "label": "Open Folder",
                "value": dir_hint,
                "opens": true,
                "steps": false,
                "enabled": entry.open_folder_target().is_some(),
            }));
        }
        actions
    }

    fn page_actions_for_active(&self) -> Vec<Value> {
        self.active_page_entry().map(Self::page_actions_for).unwrap_or_default()
    }

    pub fn choose_filter(&mut self, key: &str) {
        if let Some(idx) = FILTERS.iter().position(|(k, _)| *k == key) {
            self.active_filter = key.to_owned();
            self.focused_filter_idx = idx;
            self.ensure_selections();
        }
    }

    pub fn step_filter(&mut self, by: i32) {
        let cur = FILTERS.iter().position(|(k, _)| *k == self.active_filter).unwrap_or(0);
        let next = (cur as i32 + by).rem_euclid(FILTERS.len() as i32) as usize;
        self.active_filter = FILTERS[next].0.to_owned();
        self.focused_filter_idx = next;
        self.ensure_selections();
    }

    pub fn step_console(&mut self, by: i32) {
        let consoles = self.console_choices();
        let cur = consoles.iter().position(|c| c == &self.console_scope).unwrap_or(0);
        let next = (cur as i32 + by).rem_euclid(consoles.len().max(1) as i32) as usize;
        self.console_scope = consoles[next].clone();
        self.ensure_selections();
    }

    pub fn toggle_smart_mark(&mut self, key: Option<&str>) {
        let target = match key {
            Some(k) => self.entries.iter().find(|e| e.key == k).cloned(),
            None => self.focused_entry().cloned(),
        };
        let Some(entry) = target else { return };
        if self.batch_queue.remove(&entry.key).is_some() {
            return;
        }
        if let Some(kind) = entry.smart_default_job() {
            self.batch_queue.insert(entry.key.clone(), kind);
        }
    }

    pub fn toggle_tile_queue(&mut self, key: &str, tile_key: &str) {
        let Some(entry) = self.entries.iter().find(|e| e.key == key).cloned() else { return };
        if tile_key == "layers" {
            return;
        }
        let Some(kind) = JobKind::from_key(tile_key) else { return };
        if !entry.can_run(kind) {
            return;
        }
        if self.batch_queue.get(&entry.key) == Some(&kind) {
            self.batch_queue.remove(&entry.key);
        } else {
            self.batch_queue.insert(entry.key, kind);
        }
    }

    pub fn review_batch_question(&self) -> Question {
        if self.batch_queue.is_empty() {
            return notice_question("Batch queue", &["No jobs are queued yet. Press Y on a game row or pick a tile on the right to queue jobs."]);
        }
        let lines: Vec<String> = self
            .batch_queue
            .iter()
            .filter_map(|(k, kind)| {
                self.entries.iter().find(|e| &e.key == k).map(|e| format!("{} — {}", kind.title(), e.title))
            })
            .take(8)
            .collect();
        let more = self.batch_queue.len().saturating_sub(lines.len());
        let mut sentences = lines;
        if more > 0 {
            sentences.push(format!("And {more} more queued item(s)."));
        }
        let strs: Vec<&str> = sentences.iter().map(String::as_str).collect();
        confirm(&format!("Run {} queued job(s)?", self.batch_queue.len()), &strs)
    }

    pub fn start_queued_batch(&mut self) {
        let items: Vec<(String, JobKind)> = self.batch_queue.iter().map(|(k, v)| (k.clone(), *v)).collect();
        for (k, kind) in items {
            if let Some(entry) = self.entries.iter().find(|e| e.key == k) {
                let target = entry.job_target(kind);
                self.pending_execs.push(PendingExec {
                    op_name: kind.title().to_owned(),
                    entry_key: k.clone(),
                    request: kind.request(&target),
                    changes_library: kind.changes_library(),
                    is_scan: false,
                    batch_key: Some(k),
                });
            }
        }
    }

    pub fn navigate(&mut self, dir: &str) -> Effect {
        match self.screen_code() {
            SCREEN_LIBRARY => self.navigate_library(dir),
            SCREEN_LAYERS => self.navigate_layers(dir),
            SCREEN_PAGE => self.navigate_page(dir),
            _ => Effect::Done,
        }
    }

    fn navigate_library(&mut self, dir: &str) -> Effect {
        let shown: Vec<String> = self.filtered_entries().into_iter().map(|e| e.key.clone()).collect();
        match dir {
            "up" => match self.focus_zone {
                0 => {
                    self.focused_filter_idx = self.focused_filter_idx.saturating_sub(1);
                    self.active_filter = FILTERS[self.focused_filter_idx].0.to_owned();
                    self.ensure_selections();
                }
                2 => {
                    self.focused_tile_idx = self.focused_tile_idx.saturating_sub(2);
                }
                _ => {
                    if let Some(cur) = self.focused_key.as_deref().and_then(|k| shown.iter().position(|s| s == k)) {
                        if cur > 0 {
                            self.focused_key = Some(shown[cur - 1].clone());
                        }
                    }
                }
            },
            "down" => match self.focus_zone {
                0 => {
                    self.focused_filter_idx = (self.focused_filter_idx + 1).min(FILTERS.len() - 1);
                    self.active_filter = FILTERS[self.focused_filter_idx].0.to_owned();
                    self.ensure_selections();
                }
                2 => {
                    self.focused_tile_idx = (self.focused_tile_idx + 2).min(6);
                }
                _ => {
                    if let Some(cur) = self.focused_key.as_deref().and_then(|k| shown.iter().position(|s| s == k)) {
                        if cur + 1 < shown.len() {
                            self.focused_key = Some(shown[cur + 1].clone());
                        }
                    } else if let Some(first) = shown.first() {
                        self.focused_key = Some(first.clone());
                    }
                }
            },
            "left" => {
                if self.focus_zone == 2 && self.focused_tile_idx % 2 == 1 {
                    self.focused_tile_idx -= 1;
                } else if self.focus_zone == 2 {
                    self.focus_zone = 1;
                } else if self.focus_zone == 1 {
                    self.focus_zone = 0;
                }
            }
            "right" => {
                if self.focus_zone == 0 {
                    self.focus_zone = 1;
                } else if self.focus_zone == 1 {
                    self.focus_zone = 2;
                    self.focused_tile_idx = 0;
                } else if self.focus_zone == 2 && self.focused_tile_idx % 2 == 1 {
                    // Stay on right edge.
                } else if self.focus_zone == 2 && self.focused_tile_idx > 0 && self.focused_tile_idx < 6 {
                    self.focused_tile_idx += 1;
                }
            }
            "prevFilter" => self.step_filter(-1),
            "nextFilter" => self.step_filter(1),
            "prevConsole" => self.step_console(-1),
            "nextConsole" => self.step_console(1),
            "accept" => {
                if self.focus_zone == 2 && self.focused_tile_idx > 0 {
                    return self.activate_library_tile(self.focused_tile_idx - 1);
                }
                if let Some(entry) = self.focused_entry().cloned() {
                    self.open_page(&entry.key);
                }
            }
            "back" => {
                if self.focus_zone != 1 {
                    self.focus_zone = 1;
                }
            }
            _ => {}
        }
        Effect::Done
    }

    fn library_tile_keys(&self) -> [&'static str; 6] {
        let is_disc = self.focused_entry().is_some_and(|e| e.is_disc);
        ["convert", "make-linked", "make-self", "check", "zip", if is_disc { "migrate" } else { "layers" }]
    }

    fn activate_library_tile(&mut self, tile_idx: usize) -> Effect {
        let Some(entry) = self.focused_entry().cloned() else { return Effect::Done };
        let keys = self.library_tile_keys();
        let Some(tkey) = keys.get(tile_idx).copied() else { return Effect::Done };
        if tkey == "layers" {
            self.open_layers(&entry.key, SCREEN_LIBRARY);
            return Effect::Done;
        }
        self.toggle_tile_queue(&entry.key, tkey);
        Effect::Done
    }

    fn navigate_layers(&mut self, dir: &str) -> Effect {
        let stack_len = self.active_layers_entry().map_or(0, |e| e.patches.len() + 1);
        match dir {
            "up" => {
                if self.layers_zone == 2 {
                    self.focused_layers_action_idx = self.focused_layers_action_idx.saturating_sub(1);
                } else {
                    self.focused_stack_idx = self.focused_stack_idx.saturating_sub(1);
                }
            }
            "down" => {
                if self.layers_zone == 2 {
                    self.focused_layers_action_idx = (self.focused_layers_action_idx + 1).min(3);
                } else if stack_len > 0 {
                    self.focused_stack_idx = (self.focused_stack_idx + 1).min(stack_len - 1);
                }
            }
            "left" => {
                if self.layers_zone == 2 && self.focused_layers_action_idx == 2 {
                    return self.toggle_layers_identity();
                }
                self.layers_zone = 1;
            }
            "right" => {
                if self.layers_zone == 2 && self.focused_layers_action_idx == 2 {
                    return self.toggle_layers_identity();
                }
                self.layers_zone = 2;
            }
            "prevConsole" => {
                self.step_layers_package(-1);
            }
            "nextConsole" => {
                self.step_layers_package(1);
            }
            "prevFilter" => {
                return self.start_reorder_focused(-1);
            }
            "nextFilter" => {
                return self.start_reorder_focused(1);
            }
            "accept" => {
                if self.layers_zone == 2 {
                    return self.activate_layers_action(self.focused_layers_action_idx);
                }
                return self.verify_layers_package();
            }
            "back" => {
                self.screen = self.layers_return_screen;
            }
            _ => {}
        }
        Effect::Done
    }

    fn navigate_page(&mut self, dir: &str) -> Effect {
        let actions = self.page_actions_for_active();
        match dir {
            "up" => {
                self.focused_page_action_idx = self.focused_page_action_idx.saturating_sub(1);
            }
            "down" => {
                if !actions.is_empty() {
                    self.focused_page_action_idx = (self.focused_page_action_idx + 1).min(actions.len() - 1);
                }
            }
            "left" | "right" => {
                if actions.get(self.focused_page_action_idx).and_then(|a| a.get("key")).and_then(Value::as_str) == Some("identity") {
                    return self.toggle_page_identity();
                }
            }
            "accept" => {
                if let Some(key) = actions.get(self.focused_page_action_idx).and_then(|a| a.get("key")).and_then(Value::as_str) {
                    return self.activate_page_action(key);
                }
            }
            "back" => {
                self.screen = SCREEN_LIBRARY;
            }
            _ => {}
        }
        Effect::Done
    }

    pub fn act(&mut self, action: &str, key: Option<&str>) -> Effect {
        match action {
            "filter" => {
                if let Some(k) = key {
                    self.choose_filter(k);
                }
                Effect::Done
            }
            "step-console" => {
                self.step_console(1);
                Effect::Done
            }
            "focus-row" => {
                if let Some(k) = key {
                    self.focused_key = Some(k.to_owned());
                    self.focus_zone = 1;
                }
                Effect::Done
            }
            "open-page" => {
                let target = key
                    .map(str::to_owned)
                    .or_else(|| self.focused_entry().map(|e| e.key.clone()));
                if let Some(k) = target {
                    self.open_page(&k);
                }
                Effect::Done
            }
            "open-layers" => {
                let target = key
                    .map(str::to_owned)
                    .or_else(|| self.active_page_entry().map(|e| e.key.clone()))
                    .or_else(|| self.focused_entry().map(|e| e.key.clone()));
                if let Some(k) = target {
                    let ret = if self.screen_code() == SCREEN_PAGE { SCREEN_PAGE } else { SCREEN_LIBRARY };
                    self.open_layers(&k, ret);
                }
                Effect::Done
            }
            "toggle-tile" => {
                if let Some(tkey) = key {
                    if tkey == "layers" {
                        if let Some(entry) = self.focused_entry().cloned() {
                            self.open_layers(&entry.key, SCREEN_LIBRARY);
                        }
                        return Effect::Done;
                    }
                    if let Some(entry) = self.focused_entry().cloned() {
                        self.toggle_tile_queue(&entry.key, tkey);
                    }
                }
                Effect::Done
            }
            "toggle-mark" => {
                self.toggle_smart_mark(key);
                Effect::Done
            }
            "review-batch" => Effect::Ask(self.review_batch_question(), ASK_BATCH.to_owned()),
            "focus-layers-pkg" => {
                if let Some(k) = key {
                    self.layers_key = Some(k.to_owned());
                    self.focused_stack_idx = 0;
                    self.layers_zone = 1;
                }
                Effect::Done
            }
            "focus-stack" => {
                if let Some(idx) = key.and_then(|s| s.parse::<usize>().ok()) {
                    self.focused_stack_idx = idx;
                    self.layers_zone = 1;
                    self.clamp_stack_focus();
                }
                Effect::Done
            }
            "layers-action" => {
                match key {
                    Some("add-patch") => self.begin_add_patch_from_layers(),
                    Some("remove-patch") => self.ask_remove_focused_patch(),
                    Some("identity") => self.toggle_layers_identity(),
                    Some("verify") => self.verify_layers_package(),
                    Some("move-up") => self.start_reorder_focused(1),
                    Some("move-down") => self.start_reorder_focused(-1),
                    _ => Effect::Done,
                }
            }
            "focus-page-action" => {
                if let Some(idx) = key.and_then(|s| s.parse::<usize>().ok()) {
                    self.focused_page_action_idx = idx;
                    self.clamp_page_focus();
                }
                Effect::Done
            }
            "page-action" => {
                let act_key = key.map(str::to_owned).or_else(|| {
                    self.page_actions_for_active()
                        .get(self.focused_page_action_idx)
                        .and_then(|a| a.get("key"))
                        .and_then(Value::as_str)
                        .map(str::to_owned)
                });
                if let Some(k) = act_key {
                    return self.activate_page_action(&k);
                }
                Effect::Done
            }
            _ => Effect::Done,
        }
    }

    pub fn action_key(&mut self, key: &str) -> Effect {
        match key {
            "back" => self.navigate("back"),
            "prevFilter" => self.navigate("prevFilter"),
            "nextFilter" => self.navigate("nextFilter"),
            "prevConsole" => self.navigate("prevConsole"),
            "nextConsole" => self.navigate("nextConsole"),
            "mark" => {
                if self.screen_code() == SCREEN_LIBRARY {
                    self.toggle_smart_mark(None);
                    Effect::Done
                } else if self.screen_code() == SCREEN_LAYERS {
                    self.ask_remove_focused_patch()
                } else {
                    Effect::Done
                }
            }
            "more" => {
                if self.screen_code() == SCREEN_LIBRARY {
                    if let Some(entry) = self.focused_entry().cloned() {
                        if let Some(kind) = entry.smart_default_job() {
                            self.toggle_tile_queue(&entry.key, kind.key());
                        }
                    }
                    Effect::Done
                } else if self.screen_code() == SCREEN_LAYERS {
                    self.begin_add_patch_from_layers()
                } else {
                    Effect::Done
                }
            }
            "apply" => {
                if self.screen_code() == SCREEN_LIBRARY {
                    Effect::Ask(self.review_batch_question(), ASK_BATCH.to_owned())
                } else if self.screen_code() == SCREEN_PAGE {
                    if !self.batch_queue.is_empty() {
                        Effect::Ask(self.review_batch_question(), ASK_BATCH.to_owned())
                    } else {
                        self.navigate("accept")
                    }
                } else {
                    Effect::Done
                }
            }
            _ => Effect::Done,
        }
    }

    fn open_page(&mut self, entry_key: &str) {
        self.focused_key = Some(entry_key.to_owned());
        self.page_key = Some(entry_key.to_owned());
        self.focused_page_action_idx = 0;
        self.screen = SCREEN_PAGE;
    }

    fn open_layers(&mut self, entry_key: &str, return_screen: i32) {
        self.layers_return_screen = return_screen;
        if self.entries.iter().any(|e| e.key == entry_key && e.packaged && !e.is_disc) {
            self.layers_key = Some(entry_key.to_owned());
        } else {
            self.layers_key = self.layers_entries().first().map(|e| e.key.clone());
        }
        self.focused_stack_idx = 0;
        self.layers_zone = 1;
        self.screen = SCREEN_LAYERS;
    }

    fn step_layers_package(&mut self, by: i32) {
        let keys: Vec<String> = self.layers_entries().into_iter().map(|e| e.key.clone()).collect();
        if keys.is_empty() {
            return;
        }
        let cur = self.layers_key.as_deref().and_then(|k| keys.iter().position(|x| x == k)).unwrap_or(0);
        let next = (cur as i32 + by).rem_euclid(keys.len() as i32) as usize;
        self.layers_key = Some(keys[next].clone());
        self.focused_stack_idx = 0;
    }

    fn focused_patch_in_layers(&self) -> Option<(PackageEntry, PatchLayer)> {
        let entry = self.active_layers_entry()?.clone();
        if entry.patches.is_empty() || self.focused_stack_idx >= entry.patches.len() {
            return None;
        }
        let manifest_idx = entry.patches.len() - 1 - self.focused_stack_idx;
        let patch = entry.patches.get(manifest_idx)?.clone();
        Some((entry, patch))
    }

    fn begin_add_patch_from_layers(&mut self) -> Effect {
        let Some(entry) = self.active_layers_entry().cloned() else { return Effect::Done };
        if !entry.packaged || entry.is_disc {
            return Effect::Done;
        }
        self.begin_add_patch_for(&entry)
    }

    fn begin_add_patch_for(&mut self, entry: &PackageEntry) -> Effect {
        self.pending_patch_target = Some((entry.key.clone(), entry.pkg_rel.clone()));
        self.dialog_folder = entry
            .pkg_full_dir
            .clone()
            .chars()
            .collect::<String>();
        if self.dialog_folder.is_empty() {
            self.dialog_folder = self.library_root.clone().unwrap_or_default();
        }
        Effect::Host(CHOOSE_PATCH_FILE.to_owned())
    }

    pub fn supply_patch_file(&mut self, file_path: &str) {
        let Some((entry_key, pkg_rel)) = self.pending_patch_target.take() else { return };
        if file_path.is_empty() {
            return;
        }
        self.pending_execs.push(PendingExec {
            op_name: "Add patch".to_owned(),
            entry_key,
            request: json!({ "op": "pkg-patch-add", "paths": [pkg_rel], "patch": file_path }),
            changes_library: true,
            is_scan: false,
            batch_key: None,
        });
    }

    pub fn cancel_patch_file(&mut self) {
        self.pending_patch_target = None;
    }

    fn confirm_remove_patch(&mut self, entry: &PackageEntry, patch: &PatchLayer, recheck_note: bool) -> Effect {
        let label = if patch.name.is_empty() { patch.file.clone() } else { patch.name.clone() };
        self.pending_remove_patch = Some(PendingRemovePatch {
            entry_key: entry.key.clone(),
            pkg_rel: entry.pkg_rel.clone(),
            one_based_index: patch.index,
            patch_title: label.clone(),
        });
        let lead = format!("Remove {label} from {}?", entry.title);
        let q = if recheck_note {
            confirm("Remove soft patch?", &[&lead, "The remaining patches will be re-checked."])
        } else {
            confirm("Remove soft patch?", &[&lead])
        };
        Effect::Ask(q, ASK_REMOVE_PATCH.to_owned())
    }

    fn ask_remove_focused_patch(&mut self) -> Effect {
        let Some((entry, patch)) = self.focused_patch_in_layers() else {
            return Effect::Done;
        };
        self.confirm_remove_patch(&entry, &patch, true)
    }

    fn toggle_layers_identity(&mut self) -> Effect {
        let Some(entry) = self.active_layers_entry().cloned() else { return Effect::Done };
        self.enqueue_identity(&entry);
        Effect::Done
    }

    fn toggle_page_identity(&mut self) -> Effect {
        let Some(entry) = self.active_page_entry().cloned() else { return Effect::Done };
        self.enqueue_identity(&entry);
        Effect::Done
    }

    fn enqueue_identity(&mut self, entry: &PackageEntry) {
        if !entry.packaged || entry.patches.is_empty() {
            return;
        }
        let next = if entry.identity == "patched" { "original" } else { "patched" };
        self.pending_execs.push(PendingExec {
            op_name: "Set identity".to_owned(),
            entry_key: entry.key.clone(),
            request: json!({ "op": "pkg-identity", "paths": [entry.pkg_rel], "identity": next }),
            changes_library: true,
            is_scan: false,
            batch_key: None,
        });
    }

    fn verify_layers_package(&mut self) -> Effect {
        let Some(entry) = self.active_layers_entry().cloned() else { return Effect::Done };
        let target = entry.job_target(JobKind::Check);
        self.pending_execs.push(PendingExec {
            op_name: JobKind::Check.title().to_owned(),
            entry_key: entry.key,
            request: JobKind::Check.request(&target),
            changes_library: true,
            is_scan: false,
            batch_key: None,
        });
        Effect::Done
    }

    fn activate_layers_action(&mut self, idx: usize) -> Effect {
        match idx {
            0 => self.begin_add_patch_from_layers(),
            1 => self.ask_remove_focused_patch(),
            2 => self.toggle_layers_identity(),
            3 => self.verify_layers_package(),
            _ => Effect::Done,
        }
    }

    fn start_reorder_focused(&mut self, delta: i32) -> Effect {
        if self.reorder.is_some() {
            return Effect::Done;
        }
        let Some((entry, patch)) = self.focused_patch_in_layers() else { return Effect::Done };
        let n = entry.patches.len();
        if n < 2 {
            return Effect::Done;
        }
        let cur_idx = patch.index - 1;
        let next_idx = if delta > 0 {
            if cur_idx + 1 >= n { return Effect::Done; }
            cur_idx + 1
        } else {
            if cur_idx == 0 { return Effect::Done; }
            cur_idx - 1
        };
        let low = cur_idx.min(next_idx);
        let Ok(staged_dir) = create_staging_dir() else {
            return Effect::Ask(notice_question("Could not reorder patches", &["Temporary folder could not be created."]), ASK_INFO.to_owned());
        };
        let mut original_paths = Vec::new();
        for p in &entry.patches[low..] {
            let src = Path::new(&entry.pkg_full_dir).join(&p.file);
            let preferred = if p.name.is_empty() { &p.file } else { &p.name };
            let dest = unique_staged_path(&staged_dir, preferred);
            if std::fs::copy(&src, &dest).is_err() {
                let _ = std::fs::remove_dir_all(&staged_dir);
                return Effect::Ask(
                    notice_question("Could not reorder patches", &[&format!("Could not read patch file {}.", p.file)]),
                    ASK_INFO.to_owned(),
                );
            }
            original_paths.push(dest);
        }
        let mut desired_paths = original_paths.clone();
        desired_paths.swap(0, 1);
        let new_manifest_idx = next_idx;
        self.focused_stack_idx = n - 1 - new_manifest_idx;
        self.reorder = Some(ReorderRun {
            entry_key: entry.key,
            pkg_rel: entry.pkg_rel,
            low,
            staged_dir,
            desired_paths,
            original_paths,
            phase: ReorderPhase::Removing { remaining_index: n },
            error_words: None,
        });
        Effect::Done
    }

    fn next_reorder_exec(&mut self) -> Option<PendingExec> {
        let run = self.reorder.as_ref()?;
        match &run.phase {
            ReorderPhase::Removing { remaining_index } | ReorderPhase::RollingClean { remaining_index } => {
                if *remaining_index <= run.low {
                    return None;
                }
                Some(PendingExec {
                    op_name: "Reorder remove".to_owned(),
                    entry_key: run.entry_key.clone(),
                    request: json!({ "op": "pkg-patch-rm", "paths": [run.pkg_rel], "index": *remaining_index }),
                    changes_library: true,
                    is_scan: false,
                    batch_key: None,
                })
            }
            ReorderPhase::Adding { next_pos } => {
                let path = run.desired_paths.get(*next_pos)?;
                Some(PendingExec {
                    op_name: "Reorder add".to_owned(),
                    entry_key: run.entry_key.clone(),
                    request: json!({ "op": "pkg-patch-add", "paths": [run.pkg_rel], "patch": path.to_string_lossy() }),
                    changes_library: true,
                    is_scan: false,
                    batch_key: None,
                })
            }
            ReorderPhase::RollingRestore { next_pos } => {
                let path = run.original_paths.get(*next_pos)?;
                Some(PendingExec {
                    op_name: "Rollback add".to_owned(),
                    entry_key: run.entry_key.clone(),
                    request: json!({ "op": "pkg-patch-add", "paths": [run.pkg_rel], "patch": path.to_string_lossy() }),
                    changes_library: true,
                    is_scan: false,
                    batch_key: None,
                })
            }
        }
    }

    fn activate_page_action(&mut self, action_key: &str) -> Effect {
        let Some(entry) = self.active_page_entry().cloned() else { return Effect::Done };
        if action_key == "layers" {
            self.open_layers(&entry.key, SCREEN_PAGE);
            return Effect::Done;
        }
        if action_key == "add-patch" {
            return self.begin_add_patch_for(&entry);
        }
        if action_key == "identity" {
            self.enqueue_identity(&entry);
            return Effect::Done;
        }
        if action_key == "open-folder" {
            if let Some(dir) = entry.open_folder_target() {
                return Effect::Host(format!("{OPEN_FOLDER}{dir}"));
            }
            return Effect::Done;
        }
        if let Some(idx_str) = action_key.strip_prefix("rm-patch:") {
            if let Ok(i) = idx_str.parse::<usize>() {
                if let Some(patch) = entry.patches.get(i).cloned() {
                    return self.confirm_remove_patch(&entry, &patch, false);
                }
            }
            return Effect::Done;
        }
        let Some(kind) = JobKind::from_key(action_key) else { return Effect::Done };
        if !entry.can_run(kind) {
            return Effect::Done;
        }
        let target = entry.job_target(kind);
        if matches!(kind, JobKind::Check | JobKind::Migrate) {
            self.pending_execs.push(PendingExec {
                op_name: kind.title().to_owned(),
                entry_key: entry.key,
                request: kind.request(&target),
                changes_library: kind.changes_library(),
                is_scan: false,
                batch_key: None,
            });
            return Effect::Done;
        }
        self.pending_single = Some(PendingSingle {
            entry_key: entry.key,
            kind,
            target,
        });
        Effect::Ask(
            confirm(&format!("{}?", kind.title()), &[&format!("Run {} for {}?", kind.title(), entry.title)]),
            ASK_SINGLE.to_owned(),
        )
    }

    pub fn answer(&mut self, asked: &str, yes: bool) {
        if !yes {
            self.pending_single = None;
            self.pending_remove_patch = None;
            return;
        }
        match asked {
            ASK_BATCH => self.start_queued_batch(),
            ASK_SINGLE => {
                if let Some(s) = self.pending_single.take() {
                    self.pending_execs.push(PendingExec {
                        op_name: s.kind.title().to_owned(),
                        entry_key: s.entry_key,
                        request: s.kind.request(&s.target),
                        changes_library: s.kind.changes_library(),
                        is_scan: false,
                        batch_key: None,
                    });
                }
            }
            ASK_REMOVE_PATCH => {
                if let Some(rm) = self.pending_remove_patch.take() {
                    self.pending_execs.push(PendingExec {
                        op_name: format!("Remove {}", rm.patch_title),
                        entry_key: rm.entry_key,
                        request: json!({ "op": "pkg-patch-rm", "paths": [rm.pkg_rel], "index": rm.one_based_index }),
                        changes_library: true,
                        is_scan: false,
                        batch_key: None,
                    });
                }
            }
            _ => {}
        }
    }

    pub fn action(&self) -> Option<Action> {
        Some(Action::new("open-page", "Open"))
    }

    pub fn dialog_name_filters(&self) -> Vec<String> {
        vec![PATCH_FILTER_LABEL.to_owned(), ALL_FILES_FILTER_LABEL.to_owned()]
    }

    pub fn projection(&self, running: bool) -> Value {
        match self.screen_code() {
            SCREEN_LAYERS => self.layers_projection(running),
            SCREEN_PAGE => self.page_projection(running),
            _ => self.library_projection(running),
        }
    }

    fn storage_metrics(&self) -> (u64, u64, u64, usize) {
        let mut pkg_bytes = 0u64;
        let mut pkg_count = 0usize;
        let mut rom_bytes = 0u64;
        for e in &self.entries {
            if e.packaged {
                pkg_count += 1;
                pkg_bytes = pkg_bytes.saturating_add(e.folder_bytes);
                if e.mode == "linked" {
                    rom_bytes = rom_bytes.saturating_add(e.game_bytes);
                }
            } else {
                rom_bytes = rom_bytes.saturating_add(e.game_bytes);
                if e.is_chd {
                    pkg_count += 1;
                    pkg_bytes = pkg_bytes.saturating_add(e.game_bytes);
                }
            }
        }
        let free = self.disk_free_bytes;
        (pkg_bytes, rom_bytes, free, pkg_count)
    }

    fn library_projection(&self, running: bool) -> Value {
        let (pkg_bytes, rom_bytes, free_bytes, pkg_count) = self.storage_metrics();
        let total = pkg_bytes.saturating_add(rom_bytes).saturating_add(free_bytes);
        let (seg_pkg, seg_rom, seg_free) = storage_ratios(pkg_bytes, rom_bytes, free_bytes, total);

        let filters: Vec<Value> = FILTERS
            .iter()
            .map(|(k, label)| {
                let count = self
                    .entries
                    .iter()
                    .filter(|e| self.matches_scope_and_search(e) && Self::matches_filter(e, k))
                    .count();
                json!({
                    "key": k,
                    "label": label,
                    "count": format!("{count}"),
                    "selected": *k == self.active_filter,
                })
            })
            .collect();

        let shown = self.filtered_entries();
        let focused = self.focused_entry().cloned();
        let focused_key = focused.as_ref().map(|e| e.key.clone()).unwrap_or_default();
        let focused_idx = shown.iter().position(|e| e.key == focused_key).unwrap_or(0);

        let rows: Vec<Value> = shown
            .iter()
            .enumerate()
            .map(|(idx, e)| {
                let queued = self.batch_queue.get(&e.key).copied();
                let (badge_text, badge_kind) = if let Some(kind) = queued {
                    (format!("{} · Queued", kind.short_tag()), 1i32)
                } else if e.needs_check() {
                    (e.status_label(), 2i32)
                } else if e.convertible {
                    ("Can convert".to_owned(), 0i32)
                } else if e.packaged {
                    ("Healthy".to_owned(), 0i32)
                } else {
                    ("Unpackaged".to_owned(), 0i32)
                };
                let patch_note = if !e.patches.is_empty() {
                    let noun = if e.patches.len() == 1 { "patch" } else { "patches" };
                    format!(" · {} {noun}", e.patches.len())
                } else {
                    String::new()
                };
                let subtitle = if e.packaged {
                    let mode_label = if e.mode == "self-contained" { "Self-contained" } else { "Linked package" };
                    let chd_note = if e.is_chd { " · Converted to CHD" } else { "" };
                    format!("{} · {mode_label}{patch_note}{chd_note}", e.console)
                } else if e.convertible {
                    format!("{} · Disc image · Convertible to CHD", e.console)
                } else {
                    format!("{} · ROM in library · Not packaged", e.console)
                };
                json!({
                    "key": e.key,
                    "index": idx,
                    "title": e.title,
                    "subtitle": subtitle,
                    "badgeText": badge_text,
                    "badgeKind": badge_kind,
                    "sizeText": format_bytes(e.display_bytes()),
                    "focused": e.key == focused_key,
                })
            })
            .collect();

        let inspector = match focused {
            Some(ref e) => {
                let mode_text = if !e.packaged {
                    "Not packaged"
                } else if e.mode == "self-contained" {
                    "Self-contained"
                } else {
                    "Linked"
                };
                let kind_word = if !e.packaged {
                    "ROM in library"
                } else if e.mode == "self-contained" {
                    "Self-contained"
                } else {
                    "Linked package"
                };
                let patch_text = if e.is_disc {
                    "Cartridge only".to_owned()
                } else if e.patches.is_empty() {
                    "None".to_owned()
                } else {
                    let fmts: BTreeSet<String> = e.patches.iter().map(|p| p.format.to_ascii_uppercase()).collect();
                    let fmt_str = fmts.into_iter().collect::<Vec<_>>().join(", ");
                    let noun = if e.patches.len() == 1 { "layer" } else { "layers" };
                    format!("{} {noun} ({fmt_str})", e.patches.len())
                };
                let id_text = if !e.packaged || e.patches.is_empty() {
                    "Original dump"
                } else if e.identity == "patched" {
                    "Patched"
                } else {
                    "Original dump"
                };
                let space_text = if e.packaged {
                    format_bytes(e.display_bytes())
                } else {
                    format!("{} in ~", format_bytes(e.game_bytes))
                };
                let kv = vec![
                    json!({ "k": "Package", "v": mode_text, "warn": false }),
                    json!({ "k": "Status", "v": e.status_label(), "warn": e.needs_check() }),
                    json!({ "k": "Soft patches", "v": patch_text, "warn": false }),
                    json!({ "k": "Identity", "v": id_text, "warn": false }),
                    json!({ "k": "Space used", "v": space_text, "warn": false }),
                    json!({ "k": "Format", "v": e.format_label(), "warn": false }),
                ];
                let queued_kind = self.batch_queue.get(&e.key).copied();
                let tile_keys = self.library_tile_keys();
                let tiles: Vec<Value> = tile_keys
                    .iter()
                    .enumerate()
                    .map(|(i, tk)| {
                        if *tk == "layers" {
                            return json!({
                                "key": "layers",
                                "title": "Soft patches",
                                "sub": if e.packaged { "Inspect or edit patch chain" } else { "Make package to add patches" },
                                "enabled": e.packaged && !e.is_disc,
                                "queued": false,
                                "queuedTag": "",
                                "focused": self.focus_zone == 2 && self.focused_tile_idx == i + 1,
                            });
                        }
                        let kind = JobKind::from_key(tk).unwrap();
                        let sub = match kind {
                            JobKind::Convert => if e.convertible { "Convert disc image in place" } else { "For CUE/BIN or ISO discs" },
                            JobKind::MakeLinked => if !e.packaged { "Store saves & patches only" } else { "Already packaged" },
                            JobKind::MakeSelfContained => if e.mode != "self-contained" { "Copy game into Packages/" } else { "Already self-contained" },
                            JobKind::Check => if e.packaged { "Hash files against manifest" } else { "Package first to verify" },
                            JobKind::Zip => if e.mode == "self-contained" { "Bundle for another host" } else { "Needs self-contained" },
                            JobKind::Migrate => if e.packaged { "Saves, states, art & cheats" } else { "Package first to copy" },
                        };
                        let active = queued_kind == Some(kind);
                        json!({
                            "key": tk,
                            "title": kind.title(),
                            "sub": sub,
                            "enabled": e.can_run(kind),
                            "queued": active,
                            "queuedTag": if active { "QUEUED" } else { "" },
                            "focused": self.focus_zone == 2 && self.focused_tile_idx == i + 1,
                        })
                    })
                    .collect();
                json!({
                    "hasEntry": true,
                    "eyebrow": format!("{} · {kind_word}", e.console),
                    "title": e.title,
                    "path": if e.game_rel.is_empty() { format!("Packages/{}", e.pkg_rel) } else { e.game_rel.clone() },
                    "kv": kv,
                    "tiles": tiles,
                })
            }
            None => json!({
                "hasEntry": false,
                "eyebrow": "PACKAGES",
                "title": "No games in view",
                "path": "",
                "kv": [],
                "tiles": [],
            }),
        };

        let filter_label = FILTERS
            .iter()
            .find(|(k, _)| *k == self.active_filter)
            .map_or("ALL GAMES", |(_, l)| *l)
            .to_ascii_uppercase();
        let batch_n = self.batch_queue.len();
        let batch_summary = if batch_n == 0 {
            "Mark rows with Y or queue jobs on the right".to_owned()
        } else {
            let mut by_kind: BTreeMap<&str, usize> = BTreeMap::new();
            for kind in self.batch_queue.values() {
                *by_kind.entry(kind.short_tag()).or_default() += 1;
            }
            by_kind
                .into_iter()
                .map(|(tag, c)| format!("{c} {tag}"))
                .collect::<Vec<_>>()
                .join(" · ")
        };

        let pos_text = if shown.is_empty() {
            "0 of 0".to_owned()
        } else {
            format!("{} of {}", focused_idx + 1, shown.len())
        };

        let footer_help = json!([
            { "button": UiButton::Accept.index(), "pair": -1, "label": "Open package", "menu": false, "options": false },
            { "button": UiButton::Search.index(), "pair": -1, "label": "Mark for batch", "menu": false, "options": false },
            { "button": UiButton::Screensaver.index(), "pair": -1, "label": "Queue job", "menu": false, "options": false },
            { "button": UiButton::PageUp.index(), "pair": UiButton::PageDown.index(), "label": "Filter", "menu": false, "options": false },
            { "button": UiButton::First.index(), "pair": UiButton::Last.index(), "label": "Console", "menu": false, "options": false },
            { "button": UiButton::Menu.index(), "pair": -1, "label": "Review queue", "menu": true, "options": false },
            { "button": UiButton::Options.index(), "pair": -1, "label": "Search", "menu": false, "options": true },
            { "button": UiButton::Back.index(), "pair": -1, "label": "Back", "menu": false, "options": false },
        ]);

        let mut out = json!({
            "screen": SCREEN_LIBRARY,
            "busy": running,
            "noticeText": self.notice_text,
            "trailTools": "Tools",
            "trailSep": "/",
            "trailPackages": "Packages",
            "percentSuffix": "%",
            "storagePkgAmount": format_bytes(pkg_bytes),
            "storagePkgLabel": format!("in {pkg_count} packages & CHDs"),
            "storageRomAmount": format_bytes(rom_bytes),
            "storageRomLabel": "library (~)",
            "storageFreeAmount": format_bytes(free_bytes),
            "storageFreeLabel": "free",
            "storageNote": "Library files in ~ are read in place, never modified",
            "segPkgRatio": seg_pkg,
            "segRomRatio": seg_rom,
            "segFreeRatio": seg_free,
            "filterEyebrow": "Filter · LB / RB",
            "filters": filters,
            "consoleScope": self.console_scope,
            "consoleHint": "LT / RT",
            "batchTitle": "Batch queue",
            "batchChip": format!("{batch_n} marked"),
            "batchSummary": batch_summary,
            "batchBtnKey": "MENU",
            "batchBtnText": "Review & run jobs",
            "batchCount": batch_n,
        });
        if let Value::Object(ref mut m) = out {
            m.insert("centerEyebrow".to_owned(), json!(format!("{filter_label} · {}", shown.len())));
            m.insert("sortEyebrow".to_owned(), json!("Sorted by console, then title"));
            m.insert("rows".to_owned(), json!(rows));
            m.insert("focusedIndex".to_owned(), json!(if shown.is_empty() { -1i32 } else { focused_idx as i32 }));
            m.insert("emptyText".to_owned(), json!("No packages or games match this filter"));
            m.insert("focusZone".to_owned(), json!(self.focus_zone));
            m.insert("ctaFocused".to_owned(), json!(self.focus_zone == 2 && self.focused_tile_idx == 0));
            m.insert("ctaKey".to_owned(), json!("A"));
            m.insert("ctaLabel".to_owned(), json!("Open package page"));
            m.insert("ctaHint".to_owned(), json!("Manifest, tracks & stores ›"));
            m.insert("actionsTitle".to_owned(), json!("Local package & CHD jobs"));
            m.insert("actionsHint".to_owned(), json!("Press X to queue or toggle"));
            m.insert("inspector".to_owned(), inspector);
            m.insert("positionText".to_owned(), json!(pos_text));
            m.insert("footerHelp".to_owned(), footer_help);
        }
        out
    }

    fn layers_projection(&self, running: bool) -> Value {
        let rail_entries = self.layers_entries();
        let active = self.active_layers_entry().cloned();
        let active_key = active.as_ref().map(|e| e.key.clone()).unwrap_or_default();

        let rail: Vec<Value> = rail_entries
            .iter()
            .map(|e| {
                let n = e.patches.len();
                let noun = if n == 1 { "layer" } else { "layers" };
                let mode_word = if e.mode == "self-contained" { "Self-contained" } else { "Linked" };
                json!({
                    "key": e.key,
                    "title": e.title,
                    "countText": format!("{n} {noun}"),
                    "meta": format!("{} · {} · {mode_word}", e.console, e.core_name),
                    "selected": e.key == active_key,
                })
            })
            .collect();

        let (stack_json, inspector_json, header_json, pos_text) = match active {
            Some(ref e) => {
                let n = e.patches.len();
                let total_stack = n + 1;
                let clamped_stack = self.focused_stack_idx.min(total_stack - 1);
                let mut cards = Vec::new();
                for vis_idx in 0..n {
                    let manifest_idx = n - 1 - vis_idx;
                    let p = &e.patches[manifest_idx];
                    let patch_num = manifest_idx + 1;
                    let is_top = vis_idx == 0;
                    let pill = if is_top { format!("#{patch_num} · Top") } else { format!("#{patch_num}") };
                    let role = if is_top { "Active output layer" } else if patch_num == 1 { "First on base dump" } else { "Intermediate" };
                    let link_text = if patch_num > 1 {
                        format!("Applies over Patch #{} output", patch_num - 1)
                    } else if !e.dump_crc32.is_empty() {
                        format!("Applies over base dump (CRC-32 {})", e.dump_crc32)
                    } else {
                        "Applies over base cartridge dump".to_owned()
                    };
                    let name = if p.name.is_empty() { &p.file } else { &p.name };
                    cards.push(json!({
                        "stackIndex": vis_idx,
                        "pill": pill,
                        "name": name,
                        "meta": format!("{} patch · Stored as {}", p.format.to_ascii_uppercase(), p.file),
                        "role": role,
                        "linkText": link_text,
                        "focused": self.layers_zone == 1 && clamped_stack == vis_idx,
                    }));
                }

                let focused_patch = (clamped_stack < n).then(|| &e.patches[n - 1 - clamped_stack]);
                let detail = match focused_patch {
                    Some(p) => {
                        let name = if p.name.is_empty() { &p.file } else { &p.name };
                        let input_val = p
                            .source_crc32
                            .as_deref()
                            .map(|c| format!("CRC-32 {c}"))
                            .or_else(|| p.source_sha1.as_deref().map(short_hex))
                            .unwrap_or_else(|| "Matched base".to_owned());
                        let naming = if e.retroarch_compat { "RetroArch compatible" } else { "Distinct filenames" };
                        json!({
                            "eyebrow": format!("Focused Patch · #{} of {n}", p.index),
                            "title": name,
                            "kv": [
                                { "k": "Format", "v": format!("{} patch", p.format.to_ascii_uppercase()), "ok": false },
                                { "k": "Patch SHA-1", "v": if p.sha1.is_empty() { "—".to_owned() } else { short_hex(&p.sha1) }, "ok": false },
                                { "k": "Input target", "v": input_val, "ok": false },
                                { "k": "Package naming", "v": naming, "ok": e.retroarch_compat },
                            ],
                        })
                    }
                    None => json!({
                        "eyebrow": "Base Dump · Read-only",
                        "title": e.title,
                        "kv": [
                            { "k": "Format", "v": e.format_label(), "ok": false },
                            { "k": "Dump SHA-1", "v": if e.dump_sha1.is_empty() { "—".to_owned() } else { short_hex(&e.dump_sha1) }, "ok": false },
                            { "k": "Dump CRC-32", "v": if e.dump_crc32.is_empty() { "—".to_owned() } else { e.dump_crc32.clone() }, "ok": false },
                            { "k": "Mode", "v": if e.mode == "self-contained" { "Self-contained" } else { "Linked" }, "ok": e.state == "ok" },
                        ],
                    }),
                };

                let out_label = if n == 0 {
                    format!("No soft patches yet — {} launches base dump", e.core_name)
                } else {
                    format!("Resulting image ready for {}", e.core_name)
                };
                let out_meta = if n == 0 {
                    format_bytes(e.game_bytes)
                } else if !e.patched_crc32.is_empty() {
                    format!("{} · Patched CRC-32: {}", format_bytes(e.patched_size.max(e.game_bytes)), e.patched_crc32)
                } else {
                    format_bytes(e.patched_size.max(e.game_bytes))
                };
                let base_ext = if e.dump_ext.is_empty() { "ROM" } else { e.dump_ext.as_str() };
                let base_check = if e.state == "changed" || e.state == "missing" {
                    e.status_label()
                } else {
                    "SHA-1 ✓ untouched".to_owned()
                };
                let header = json!({
                    "hasEntry": true,
                    "eyebrow": format!("{} · CORE: {} · APPLIED BOTTOM TO TOP", e.console.to_ascii_uppercase(), e.core_name.to_ascii_uppercase()),
                    "title": e.title,
                    "outputLabel": out_label,
                    "outputMeta": out_meta,
                    "hasPatches": n > 0,
                    "baseFocused": self.layers_zone == 1 && clamped_stack == n,
                    "baseIndex": n,
                    "basePath": if e.game_rel.is_empty() { e.pkg_rel.clone() } else { e.game_rel.clone() },
                    "baseBadge": format!("{base_ext} · {}", format_bytes(e.game_bytes)),
                    "baseCheck": base_check,
                    "identityText": if e.identity == "patched" { "Patched" } else { "Original" },
                    "canRemove": clamped_stack < n && n > 0,
                });
                (json!(cards), detail, header, format!("{} of {total_stack}", clamped_stack + 1))
            }
            None => (
                json!([]),
                json!({
                    "eyebrow": "Soft Patches",
                    "title": "No cartridge packages",
                    "kv": [],
                }),
                json!({
                    "hasEntry": false,
                    "eyebrow": "CARTRIDGE PACKAGES · APPLIED BOTTOM TO TOP",
                    "title": "No patched cartridge packages",
                    "outputLabel": "Open any cartridge package to attach .ips, .bps, or .ups soft patches",
                    "outputMeta": "",
                    "hasPatches": false,
                    "baseFocused": false,
                    "baseIndex": 0,
                    "basePath": "",
                    "baseBadge": "",
                    "baseCheck": "",
                    "identityText": "Original",
                    "canRemove": false,
                }),
                "0 of 0".to_owned(),
            ),
        };

        let layers_actions = [
            json!({ "key": "add-patch", "label": "Add a patch file…", "chip": "X", "steps": false, "stepVal": "", "focused": self.layers_zone == 2 && self.focused_layers_action_idx == 0 }),
            json!({ "key": "remove-patch", "label": "Remove focused patch…", "chip": "Y", "steps": false, "stepVal": "", "focused": self.layers_zone == 2 && self.focused_layers_action_idx == 1 }),
            json!({ "key": "identity", "label": "Scraped as", "chip": "", "steps": true, "stepVal": header_json["identityText"], "focused": self.layers_zone == 2 && self.focused_layers_action_idx == 2 }),
            json!({ "key": "verify", "label": "Verify package & patches", "chip": "A", "steps": false, "stepVal": "", "focused": self.layers_zone == 2 && self.focused_layers_action_idx == 3 }),
        ];

        json!({
            "screen": SCREEN_LAYERS,
            "busy": running,
            "noticeText": self.notice_text,
            "trailTools": "Tools",
            "trailSep": "/",
            "trailPackages": "Packages",
            "trailLayers": "Patch Layers",
            "percentSuffix": "%",
            "layersRailTitle": format!("Patched packages · {}", rail_entries.len()),
            "layersRailHint": "LT / RT",
            "rail": rail,
            "header": header_json,
            "layersSubtitle": "Soft patches (.bps, .ips, .ups) apply in memory on launch. Your cartridge dump is never written.",
            "stack": stack_json,
            "baseCardTitle": "Base cartridge dump (read-only)",
            "layersFootnote": "Reordering re-evaluates checksums against the target image before saving.",
            "detail": inspector_json,
            "actionsCardEyebrow": "Local Patch & Package Actions",
            "layersActions": layers_actions,
            "stepLeftGlyph": "◀",
            "stepRightGlyph": "▶",
            "layersNoteText": "Patches never modify the file in ~. Remove or reorder them anytime; EmuSen rolls back immediately if a chain check fails.",
            "positionText": pos_text,
            "footerHelp": [
                { "button": UiButton::Accept.index(), "pair": -1, "label": "Verify chain", "menu": false, "options": false },
                { "button": UiButton::PageUp.index(), "pair": UiButton::PageDown.index(), "label": "Move patch down / up", "menu": false, "options": false },
                { "button": UiButton::Screensaver.index(), "pair": -1, "label": "Add patch file…", "menu": false, "options": false },
                { "button": UiButton::Search.index(), "pair": -1, "label": "Remove patch…", "menu": false, "options": false },
                { "button": UiButton::First.index(), "pair": UiButton::Last.index(), "label": "Game", "menu": false, "options": false },
                { "button": UiButton::Back.index(), "pair": -1, "label": "Back", "menu": false, "options": false },
            ],
        })
    }

    fn page_projection(&self, running: bool) -> Value {
        let Some(entry) = self.active_page_entry().cloned() else {
            return json!({
                "screen": SCREEN_PAGE,
                "busy": running,
                "noticeText": self.notice_text,
                "trailTools": "Tools",
                "trailSep": "/",
                "trailPackages": "Packages",
                "trailPage": "",
                "percentSuffix": "%",
                "pageTitle": "No package selected",
                "pageSubtitle": "",
                "facts": [],
                "actions": [],
                "positionText": "0 of 0",
                "footerHelp": [
                    { "button": UiButton::Back.index(), "pair": -1, "label": "Back", "menu": false, "options": false },
                ],
            });
        };

        let mode_val = if !entry.packaged {
            "Not packaged"
        } else if entry.mode == "self-contained" {
            "Self-contained"
        } else {
            "Linked"
        };
        let subtitle = if entry.packaged {
            format!(
                "Packages/{}, {} package — {}. Last run: {}.",
                entry.pkg_rel,
                entry.mode,
                entry.status_label().to_ascii_lowercase(),
                entry.last_run
            )
        } else {
            format!("~/{}, ROM in library — ready to package.", entry.game_rel)
        };

        let size_val = if entry.original_bytes > 0 && entry.chd_bytes > 0 {
            format!("{} ({} original)", format_bytes(entry.chd_bytes), format_bytes(entry.original_bytes))
        } else {
            format_bytes(entry.display_bytes())
        };

        let facts = [
            json!({ "label": "Package", "value": mode_val, "attention": false }),
            json!({ "label": "Game", "value": entry.title, "attention": false }),
            json!({ "label": "State", "value": entry.status_label(), "attention": entry.needs_check() }),
            json!({ "label": "Last run", "value": entry.last_run, "attention": false }),
            json!({ "label": "Size", "value": size_val, "attention": false }),
            json!({ "label": "Format", "value": entry.format_label(), "attention": false }),
        ];

        let raw_actions = Self::page_actions_for(&entry);
        let actions_len = raw_actions.len();
        let focus_idx = self.focused_page_action_idx.min(actions_len.saturating_sub(1));
        let actions: Vec<Value> = raw_actions
            .into_iter()
            .enumerate()
            .map(|(i, mut obj)| {
                if let Some(m) = obj.as_object_mut() {
                    m.insert("focused".to_owned(), Value::Bool(i == focus_idx));
                    m.insert("index".to_owned(), Value::from(i));
                }
                obj
            })
            .collect();
        let pos_text = if actions.is_empty() {
            "0 of 0".to_owned()
        } else {
            format!("{} of {}", focus_idx + 1, actions.len())
        };

        json!({
            "screen": SCREEN_PAGE,
            "busy": running,
            "noticeText": self.notice_text,
            "trailTools": "Tools",
            "trailSep": "/",
            "trailPackages": "Packages",
            "trailPage": entry.title,
            "percentSuffix": "%",
            "pageTitle": entry.title,
            "pageSubtitle": subtitle,
            "facts": facts,
            "actions": actions,
            "focusedActionIndex": if actions_len == 0 { -1i32 } else { focus_idx as i32 },
            "positionText": pos_text,
            "footerHelp": [
                { "button": UiButton::Accept.index(), "pair": -1, "label": "Choose", "menu": false, "options": false },
                { "button": UiButton::Back.index(), "pair": -1, "label": "Back", "menu": false, "options": false },
                { "button": UiButton::Menu.index(), "pair": -1, "label": "Apply…", "menu": true, "options": false },
            ],
        })
    }
}

fn advance_reorder(run: &mut ReorderRun, err: Option<String>) -> Option<Question> {
    match &mut run.phase {
        ReorderPhase::Removing { remaining_index } => {
            if let Some(w) = err {
                return Some(notice_question("Could not reorder patches", &[&w]));
            }
            if *remaining_index > run.low + 1 {
                *remaining_index -= 1;
            } else {
                run.phase = ReorderPhase::Adding { next_pos: 0 };
            }
            None
        }
        ReorderPhase::Adding { next_pos } => {
            if let Some(w) = err {
                run.error_words = Some(w);
                let added_count = *next_pos;
                if added_count > 0 {
                    run.phase = ReorderPhase::RollingClean { remaining_index: run.low + added_count };
                } else {
                    run.phase = ReorderPhase::RollingRestore { next_pos: 0 };
                }
                return None;
            }
            *next_pos += 1;
            None
        }
        ReorderPhase::RollingClean { remaining_index } => {
            if *remaining_index > run.low + 1 {
                *remaining_index -= 1;
            } else {
                run.phase = ReorderPhase::RollingRestore { next_pos: 0 };
            }
            None
        }
        ReorderPhase::RollingRestore { next_pos } => {
            *next_pos += 1;
            None
        }
    }
}

fn notice_question(title: &str, sentences: &[&str]) -> Question {
    Question {
        title: title.to_owned(),
        sentences: sentences.iter().map(|s| (*s).to_owned()).collect(),
        caption: String::new(),
        choices: vec!["OK".to_owned()],
        back: Some(0),
        picture: String::new(),
    }
}

fn create_staging_dir() -> std::io::Result<PathBuf> {
    let base = std::env::temp_dir().join(format!("emusen-reorder-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base)?;
    Ok(base)
}

fn unique_staged_path(dir: &Path, filename: &str) -> PathBuf {
    let clean = Path::new(filename).file_name().unwrap_or_default().to_string_lossy().into_owned();
    let candidate = dir.join(&clean);
    if !candidate.exists() {
        return candidate;
    }
    let stem = Path::new(&clean).file_stem().unwrap_or_default().to_string_lossy().into_owned();
    let ext = Path::new(&clean).extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
    for k in 2..1000 {
        let next = dir.join(format!("{stem} ({k}){ext}"));
        if !next.exists() {
            return next;
        }
    }
    dir.join(format!("{clean}.tmp"))
}

fn extract_dump_hashes(doc: Option<&Value>, ev: &Value) -> (String, String) {
    let file_obj = doc
        .and_then(|d| d.get("dump"))
        .and_then(|d| d.get("file"))
        .or_else(|| doc.and_then(|d| d.get("dump")).and_then(|d| d.get("sheet")));
    let sha1 = file_obj
        .and_then(|f| f.get("sha1"))
        .and_then(Value::as_str)
        .or_else(|| ev.get("dump_sha1").and_then(Value::as_str))
        .unwrap_or("")
        .to_owned();
    let crc32 = file_obj
        .and_then(|f| f.get("crc32"))
        .and_then(Value::as_str)
        .or_else(|| ev.get("dump_crc32").and_then(Value::as_str))
        .unwrap_or("")
        .to_ascii_uppercase();
    (sha1, crc32)
}

fn extract_patched_info(doc: Option<&Value>, ev: &Value) -> (u64, String) {
    let patched = doc.and_then(|d| d.get("patched"));
    let size = patched
        .and_then(|p| p.get("size"))
        .and_then(Value::as_u64)
        .or_else(|| ev.get("patched_size").and_then(Value::as_u64))
        .unwrap_or(0);
    let crc32 = patched
        .and_then(|p| p.get("crc32"))
        .and_then(Value::as_str)
        .or_else(|| ev.get("patched_crc32").and_then(Value::as_str))
        .unwrap_or("")
        .to_ascii_uppercase();
    (size, crc32)
}

fn parse_patch_object(index: usize, p: &Value, default_to_file: bool) -> PatchLayer {
    let file = p.get("file").and_then(Value::as_str).unwrap_or("").to_owned();
    let name = p
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or(if default_to_file { &file } else { "" })
        .to_owned();
    PatchLayer {
        index,
        format: p.get("format").and_then(Value::as_str).map(str::to_owned).unwrap_or_else(|| path_ext(&file)),
        file,
        name,
        sha1: p.get("sha1").and_then(Value::as_str).unwrap_or("").to_owned(),
        source_crc32: p.get("source_crc32").and_then(Value::as_str).map(|s| s.to_ascii_uppercase()),
        target_crc32: p.get("target_crc32").and_then(Value::as_str).map(|s| s.to_ascii_uppercase()),
        source_sha1: p.get("source_sha1").and_then(Value::as_str).map(str::to_owned),
    }
}

fn extract_patches(doc: Option<&Value>, ev: &Value) -> Vec<PatchLayer> {
    if let Some(arr) = doc.and_then(|d| d.get("patches")).and_then(Value::as_array) {
        return arr.iter().enumerate().map(|(i, p)| parse_patch_object(i + 1, p, false)).collect();
    }
    ev.get("patches")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
        .map(|(i, p)| match p.as_str() {
            Some(s) => PatchLayer {
                index: i + 1,
                file: s.to_owned(),
                name: s.to_owned(),
                format: path_ext(s),
                ..PatchLayer::default()
            },
            None => parse_patch_object(i + 1, p, true),
        })
        .collect()
}

fn sort_entries(entries: &mut [PackageEntry]) {
    entries.sort_by(|a, b| {
        a.console
            .to_lowercase()
            .cmp(&b.console.to_lowercase())
            .then_with(|| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
            .then_with(|| a.key.cmp(&b.key))
    });
}

fn path_ext(path: &str) -> String {
    Path::new(path)
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default()
}

fn is_disc_path(path: &str) -> bool {
    let ext = path_ext(path);
    DISC_EXTENSIONS.contains(&ext.as_str())
}

fn title_case_word(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => format!("{}{}", c.to_uppercase(), chars.as_str()),
        None => String::new(),
    }
}

fn short_hex(hex: &str) -> String {
    if hex.len() <= 14 {
        return hex.to_owned();
    }
    format!("{}…{}", &hex[..8], &hex[hex.len() - 4..])
}

fn format_run_outcome(op: &str, state: &str, words: &str) -> String {
    if words.is_empty() {
        format!("{op}: {}", title_case_word(state))
    } else {
        format!("{op}: {} — {words}", title_case_word(state))
    }
}

pub(crate) fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1000.0;
    const MB: f64 = 1_000_000.0;
    const GB: f64 = 1_000_000_000.0;
    let b = bytes as f64;
    if b >= GB {
        format!("{:.1} GB", b / GB)
    } else if b >= MB {
        format!("{:.1} MB", b / MB)
    } else if b >= KB {
        format!("{:.0} KB", b / KB)
    } else {
        format!("{bytes} B")
    }
}

fn storage_ratios(pkg: u64, rom: u64, free: u64, total: u64) -> (f64, f64, f64) {
    if total == 0 {
        return (0.0, 0.0, 1.0);
    }
    let min_vis = 0.015;
    let mut r_pkg = if pkg > 0 { (pkg as f64 / total as f64).max(min_vis) } else { 0.0 };
    let mut r_rom = if rom > 0 { (rom as f64 / total as f64).max(min_vis) } else { 0.0 };
    let mut r_free = if free > 0 { (free as f64 / total as f64).max(min_vis) } else { 0.0 };
    let sum = r_pkg + r_rom + r_free;
    if sum > 0.0 {
        r_pkg /= sum;
        r_rom /= sum;
        r_free /= sum;
    }
    (r_pkg, r_rom, r_free)
}

#[cfg(unix)]
fn disk_free_bytes(path: &Path) -> u64 {
    let mut probe = path;
    while !probe.exists() {
        let Some(parent) = probe.parent() else { return 0 };
        probe = parent;
    }
    let Ok(cstr) = std::ffi::CString::new(probe.as_os_str().to_string_lossy().as_bytes()) else {
        return 0;
    };
    let mut vfs: libc::statvfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statvfs(cstr.as_ptr(), &mut vfs) } == 0 {
        (vfs.f_bavail as u64).saturating_mul(vfs.f_frsize as u64)
    } else {
        0
    }
}

#[cfg(not(unix))]
fn disk_free_bytes(_path: &Path) -> u64 {
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_events() -> Vec<Value> {
        vec![
            json!({
                "event": "package",
                "source": "snes/Super Metroid",
                "title": "Super Metroid",
                "system": "snes",
                "console": "SNES",
                "core": "Snes9x",
                "mode": "linked",
                "state": "ok",
                "size": 8_200_000,
                "game_size": 4_194_304,
                "patches": [
                    { "file": "Super Metroid.ips", "name": "Super Metroid Redux v1.8.ips", "format": "ips", "sha1": "a3c194f01122e21b" },
                    { "file": "Super Metroid.ips1", "name": "MSU Audio Hook.ips", "format": "ips", "sha1": "7f08b912cc4409d1" }
                ],
                "retroarch": true,
                "identity": "patched"
            }),
            json!({
                "event": "package",
                "source": "saturn/Panzer Dragoon II",
                "title": "Panzer Dragoon II Zwei",
                "system": "saturn",
                "console": "Saturn",
                "core": "Beetle Saturn",
                "mode": "linked",
                "state": "ok",
                "size": 694_000_000,
                "game_size": 694_000_000,
                "convertible": true
            }),
            json!({
                "event": "game",
                "source": "gba/Metroid Fusion.gba",
                "title": "Metroid Fusion",
                "system": "gba",
                "console": "GBA",
                "size": 8_388_608,
                "convertible": false
            }),
            json!({
                "event": "package",
                "source": "psx/Castlevania",
                "title": "Castlevania: Symphony of the Night",
                "system": "psx",
                "console": "PS1",
                "mode": "self-contained",
                "state": "changed",
                "size": 412_000_000,
                "game_size": 412_000_000,
                "is_disc": true
            }),
        ]
    }

    #[test]
    fn filters_and_cartridge_layers_exclude_discs() {
        let mut p = EclipsePackages::new();
        p.entries_from_test_scan(&sample_events());

        assert_eq!(p.filtered_entries().len(), 4);
        p.choose_filter("packaged");
        assert_eq!(p.filtered_entries().len(), 3);
        p.choose_filter("unpackaged");
        assert_eq!(p.filtered_entries().len(), 1);
        p.choose_filter("convertible");
        assert_eq!(p.filtered_entries().len(), 1);
        p.choose_filter("patched");
        assert_eq!(p.filtered_entries().len(), 1);
        p.choose_filter("needs_check");
        assert_eq!(p.filtered_entries().len(), 1);

        let layers = p.layers_entries();
        assert_eq!(layers.len(), 1);
        assert_eq!(layers[0].title, "Super Metroid");
        assert!(!layers.iter().any(|e| e.is_disc));
    }

    #[test]
    fn packaged_page_omits_make_package_audit_180() {
        let mut p = EclipsePackages::new();
        p.entries_from_test_scan(&sample_events());

        let pkg = p.entries.iter().find(|e| e.title == "Super Metroid").unwrap();
        let pkg_actions = EclipsePackages::page_actions_for(pkg);
        let labels: Vec<&str> = pkg_actions.iter().filter_map(|a| a["label"].as_str()).collect();
        assert!(!labels.iter().any(|l| l.contains("Make Package") || l.contains("Make linked")));
        assert!(labels.contains(&"Soft patches"));
        assert!(labels.contains(&"Check Package"));
        assert!(labels.contains(&"Stores"));

        let raw = p.entries.iter().find(|e| e.title == "Metroid Fusion").unwrap();
        let raw_actions = EclipsePackages::page_actions_for(raw);
        let raw_labels: Vec<&str> = raw_actions.iter().filter_map(|a| a["label"].as_str()).collect();
        assert!(raw_labels.contains(&"Make linked package"));
        assert!(raw_labels.contains(&"Make self-contained"));
    }

    #[test]
    fn batch_queue_tracks_marks_and_generates_requests() {
        let mut p = EclipsePackages::new();
        p.entries_from_test_scan(&sample_events());
        p.toggle_smart_mark(Some("pkg:saturn/Panzer Dragoon II"));
        p.toggle_smart_mark(Some("gba/Metroid Fusion.gba"));
        assert_eq!(p.batch_queue.len(), 1);
        p.toggle_smart_mark(Some("game:gba/Metroid Fusion.gba"));
        assert_eq!(p.batch_queue.len(), 2);

        let q = p.review_batch_question();
        assert!(q.title.contains("2 queued"));
        p.start_queued_batch();
        assert_eq!(p.pending_execs.len(), 2);
    }

    #[test]
    fn page_navigation_back_pops_to_library() {
        let mut p = EclipsePackages::new();
        p.entries_from_test_scan(&sample_events());
        let _ = p.act("open-page", Some("pkg:snes/Super Metroid"));
        assert_eq!(p.screen_code(), SCREEN_PAGE);
        let _ = p.act("open-layers", None);
        assert_eq!(p.screen_code(), SCREEN_LAYERS);
        let _ = p.action_key("back");
        assert_eq!(p.screen_code(), SCREEN_PAGE);
        let _ = p.action_key("back");
        assert_eq!(p.screen_code(), SCREEN_LIBRARY);
    }

    #[test]
    fn pkg_ls_package_key_and_game_event_parse() {
        let mut p = EclipsePackages::new();
        p.queue_scan();
        let exec = p.take_next_request().unwrap();
        p.ingest_event(&exec, &json!({
            "event": "package",
            "package": "Super Nintendo/Super Metroid.sfc",
            "state": "ok",
            "mode": "linked",
            "console": "Super Nintendo",
            "size": 1024,
            "game": "snes/Super Metroid.sfc",
            "patches": []
        }));
        p.ingest_event(&exec, &json!({
            "event": "game",
            "source": "gba/Metroid Fusion.gba",
            "system": "Game Boy Advance",
            "state": "unpackaged",
            "size": 32768
        }));
        p.finish_exec(exec, None);
        assert_eq!(p.entries.len(), 2);
        assert_eq!(p.entries[0].key, "game:gba/Metroid Fusion.gba");
        assert_eq!(p.entries[1].key, "pkg:Super Nintendo/Super Metroid.sfc");
    }
}
