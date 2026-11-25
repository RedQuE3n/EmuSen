//! Eclipse Cores board (#10) projection, filtering, and immediate subpage (#72-#74) helpers.

use super::row::{Core, Origin};
use super::{plan, words, Cores, APPROVE, DEFAULT, OPEN_FOLDER};
use crate::card::{Pane as CardPane, Row as CardRow};
use crate::firmware::Have;
use crate::library::without_port_mark;
use crate::settings_shown::planet_of;
use crate::tools::{Effect, Mark};
use crate::words::{counted, games};
use emusen_kit_core::input::UiButton;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;

pub const ECLIPSE_UPDATE: &str = "eclipse-update";
pub const ECLIPSE_INSTALL: &str = "eclipse-install";
pub const ECLIPSE_REMOVE: &str = "eclipse-remove";
pub const ECLIPSE_PAGE: &str = "eclipse-page";

/// Active filter pill on the `#10` Eclipse Cores board.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Filter {
    #[default]
    Ready,
    Updates,
    Firmware,
    Approval,
}

impl Filter {
    pub fn key(self) -> &'static str {
        match self {
            Filter::Ready => "ready",
            Filter::Updates => "updates",
            Filter::Firmware => "firmware",
            Filter::Approval => "approval",
        }
    }

    pub fn from_key(key: &str) -> Self {
        match key {
            "updates" => Filter::Updates,
            "firmware" => Filter::Firmware,
            "approval" => Filter::Approval,
            _ => Filter::Ready,
        }
    }
}

/// Interactive state held for Eclipse Cores (`#10`).
#[derive(Clone, Debug)]
pub struct EclipseState {
    pub enabled: bool,
    pub showing_remaining: bool,
    pub filter: Filter,
    pub focus_zone: String,
    pub focused_top_index: usize,
    pub focused_console: Option<String>,
    pub focused_core: Option<String>,
    pub console_counts: HashMap<String, usize>,
}

impl Default for EclipseState {
    fn default() -> Self {
        Self {
            enabled: false,
            showing_remaining: false,
            filter: Filter::Ready,
            focus_zone: "grid".to_owned(),
            focused_top_index: 0,
            focused_console: None,
            focused_core: None,
            console_counts: HashMap::new(),
        }
    }
}

/// Plain core name for Eclipse surfaces, stripping port marks on EmuSen's own cores.
pub fn plain_name(row: &Core) -> String {
    if row.origin == Origin::Project {
        without_port_mark(&row.name).to_owned()
    } else {
        row.name.clone()
    }
}

/// Parses optional `consoleCounts` JSON from either `{"psx": 88}` or `library.settingsFacts("")`.
pub fn parse_console_counts(raw: &str) -> HashMap<String, usize> {
    let Ok(val) = serde_json::from_str::<Value>(raw) else { return HashMap::new() };
    let obj = val.get("systems").and_then(Value::as_object).or_else(|| val.as_object());
    let Some(map) = obj else { return HashMap::new() };
    map.iter().filter_map(|(k, v)| v.as_u64().map(|n| (k.clone(), n as usize))).collect()
}

/// Builds a single-core immediate confirmation question using plan sentences.
pub fn confirm_single(row: &Core, mark_key: &str) -> crate::question::Question {
    let mut named = row.clone();
    named.name = plain_name(row);
    let mark = Mark { key: mark_key.to_owned(), label: mark_key.to_owned() };
    let pairs = [(&named, &mark)];
    crate::tools::plan_question(&plan::title(&pairs), &plan::sentences(&pairs))
}

impl Cores {
    /// Whether `row` is the active default engine on `console`.
    pub(super) fn is_default_on(&self, console: &str, row: &Core) -> bool {
        row.engine.as_deref().is_some_and(|e| !row.awaits_approval() && row.consoles.iter().any(|c| c == console) && self.is_default(console, e))
    }

    fn planet_for(&self, row: &Core) -> i32 {
        match row.origin {
            Origin::Project => planet_of(&plain_name(row), false, true),
            Origin::Libretro => -1,
            Origin::Added => -3,
        }
    }

    fn count_for(&self, console: &str) -> Option<usize> {
        if self.eclipse.console_counts.is_empty() {
            return None;
        }
        if let Some(n) = self.eclipse.console_counts.get(console) {
            return Some(*n);
        }
        let mut total = 0usize;
        let mut found = false;
        for (sys, c) in &self.consoles.console_of {
            if c == console {
                if let Some(n) = self.eclipse.console_counts.get(sys) {
                    total += *n;
                    found = true;
                }
            }
        }
        found.then_some(total)
    }

    fn have_firmware_on(&self, console: &str, row: &Core) -> Option<Result<Have, String>> {
        let system = row.systems.iter().find(|s| self.consoles.console_of.get(*s).map(String::as_str) == Some(console)).or_else(|| row.systems.first())?;
        if let Some(known) = self.have_firmware.borrow().get(system) {
            return Some(known.clone());
        }
        let have = self.backend.firmware(system).map(|files| crate::firmware::have(&files));
        self.have_firmware.borrow_mut().insert(system.clone(), have.clone());
        Some(have)
    }

    fn core_needs_firmware_missing(&self, console: &str, row: &Core) -> bool {
        let declares = !self.firmware_listed(row).is_empty() || row.firmware.is_some();
        if !declares {
            return false;
        }
        matches!(self.have_firmware_on(console, row), Some(Ok(Have::Missing) | Err(_)))
    }

    fn installed_for(&self, console: &str) -> Vec<&Core> {
        let mut list: Vec<&Core> = self.rows.iter().filter(|r| r.installed() && r.consoles.iter().any(|c| c == console)).collect();
        list.sort_by(|a, b| {
            self.is_default_on(console, b)
                .cmp(&self.is_default_on(console, a))
                .then(a.awaits_approval().cmp(&b.awaits_approval()))
                .then((b.origin == Origin::Project).cmp(&(a.origin == Origin::Project)))
                .then(plain_name(a).to_lowercase().cmp(&plain_name(b).to_lowercase()))
        });
        list
    }

    fn available_for(&self, console: &str) -> Vec<&Core> {
        let mut list: Vec<&Core> = self.rows.iter().filter(|r| !r.installed() && r.is_libretro() && r.state == "available" && r.consoles.iter().any(|c| c == console)).collect();
        let rank = |r: &Core| {
            let key = r.engine.as_deref().unwrap_or(&r.name);
            self.engines_of.get(console).and_then(|list| list.iter().position(|x| x == key)).unwrap_or(usize::MAX)
        };
        list.sort_by(|a, b| rank(a).cmp(&rank(b)).then(a.name.to_lowercase().cmp(&b.name.to_lowercase())));
        list
    }

    /// Breadcrumb string (`"Cores · <Console>"`) for Eclipse core subpages `#72`-`#74`.
    pub fn eclipse_breadcrumb(&self, key: &str) -> Option<String> {
        if !self.eclipse.enabled {
            return None;
        }
        let row = self.row(key)?;
        let console = self
            .eclipse
            .focused_console
            .as_deref()
            .filter(|c| row.consoles.iter().any(|rc| rc == *c))
            .or_else(|| row.consoles.first().map(String::as_str));
        Some(console.map_or_else(|| "Cores".to_owned(), |c| format!("Cores · {}", self.consoles.name(c))))
    }

    /// Label and value split for `#73` Details aside when `paneDrawn` renders it natively.
    pub fn eclipse_detail_pane(rows: &[CardRow], at: usize) -> Option<(String, String)> {
        let row = rows.get(at)?;
        if !row.key.starts_with("detail.") {
            return None;
        }
        let value = row.pane.lines.first().map(|l| l.0.clone()).unwrap_or_default();
        Some((row.label.clone(), value))
    }

    /// Aside summary pane shared across `#72` Core Page rows.
    pub(super) fn eclipse_core_pane(&self, row: &Core) -> CardPane {
        let runs = if row.consoles.is_empty() { "games of no console EmuSen knows".to_owned() } else { format!("{} games", row.console_names.join(" and ")) };
        let lead = match row.origin {
            Origin::Libretro => format!("Runs {runs} through EmuSen's libretro host, in a sandbox of its own."),
            Origin::Project => format!("Runs {runs}. Built into EmuSen and updated with it, so there is nothing to install or approve."),
            Origin::Added => format!("Runs {runs}, from a file of your own."),
        };
        let mut fw_note = String::new();
        let listed = self.firmware_listed(row);
        if let Some(first) = listed.first() {
            let opt = if first.get("optional").and_then(Value::as_bool).unwrap_or(false) { " (optional)" } else { "" };
            let desc = first.get("desc").and_then(Value::as_str).unwrap_or("firmware");
            fw_note = format!("Needs {} of your own{opt}, which EmuSen never downloads.", words::article(desc));
        } else if let (Some(need), Some(kind)) = (row.firmware.as_deref(), row.consoles.first().and_then(|c| self.firmware_of.get(c))) {
            fw_note = if need == "required" {
                format!("Needs {} of your own, which EmuSen never downloads.", words::article(kind))
            } else {
                format!("Can use {} of your own, which EmuSen never downloads; it runs without one.", words::article(kind))
            };
        }
        let (status, status_ok) = if row.awaits_approval() {
            ("⚠ Waiting for your approval. Nothing downloaded runs until you approve it.".to_owned(), false)
        } else if row.trust.as_deref() == Some("needs_refresh") {
            ("⚠ EmuSen's copies beside it are older than this build: Verify refreshes them.".to_owned(), false)
        } else if !fw_note.is_empty() {
            let console = self.eclipse.focused_console.as_deref().or_else(|| row.consoles.first().map(String::as_str)).unwrap_or_default();
            match self.have_firmware_on(console, row) {
                Some(Ok(Have::Found)) => ("✓ You have one.".to_owned(), true),
                Some(Ok(Have::Missing)) => ("⚠ None found yet. Firmware says where it goes.".to_owned(), false),
                Some(Err(why)) => (format!("⚠ Could not check firmware: {why}."), false),
                _ => (String::new(), false),
            }
        } else if row.is_libretro() && row.installed() && self.hash_of(row).is_some() {
            ("✓ Approved, held to its hash.".to_owned(), true)
        } else if row.origin == Origin::Project {
            ("✓ Built into EmuSen, up to date.".to_owned(), true)
        } else {
            (String::new(), false)
        };
        if fw_note.is_empty() {
            let mut extra = Vec::new();
            let by = if row.authors.is_empty() { "By authors not said".to_owned() } else { format!("By {}", row.authors.join(", ")) };
            let licence = if row.licence.is_empty() { ", licence not said".to_owned() } else { format!(", under the {}", row.licence) };
            extra.push(format!("{by}{licence}."));
            if row.state == "update" {
                extra.push("A newer build is offered; the build installed now is kept for Roll Back.".to_owned());
            }
            CardPane { lines: vec![(lead, extra.join(" "))], status, status_ok, ..CardPane::default() }
        } else {
            CardPane { lines: vec![(lead, fw_note)], status, status_ok, ..CardPane::default() }
        }
    }

    /// Rows for `#72` Core Page in Eclipse mode with immediate confirmation actions and `Open Folder`.
    pub(super) fn eclipse_page(&self, row: &Core) -> Vec<CardRow> {
        let mut rows = Vec::new();
        let pane = self.eclipse_core_pane(row);
        if row.is_libretro() && row.state == "update" {
            rows.push(CardRow::new(ECLIPSE_UPDATE, "Update…").pane(pane.clone()));
        } else if row.is_libretro() && row.state == "available" {
            rows.push(CardRow::new(ECLIPSE_INSTALL, "Install…").pane(pane.clone()));
        }
        let engine = row.engine.is_some() && !row.consoles.is_empty();
        let installed_libretro = row.is_libretro() && row.installed();
        if engine {
            rows.push(CardRow::new(super::OPTIONS, "Options").opens().pane(pane.clone()));
        }
        if (!self.firmware_listed(row).is_empty() || row.firmware.is_some()) && self.firmware_files(row).map_or(true, |f| !f.is_empty()) {
            rows.push(CardRow::new(super::FIRMWARE, "Firmware").opens().pane(pane.clone()));
        }
        if self.hash_of(row).is_some() && row.awaits_approval() {
            rows.push(CardRow::new(APPROVE, "Approve…").pane(pane.clone()));
        }
        if installed_libretro {
            rows.push(CardRow::new(super::VERIFY, "Verify").pane(pane.clone()));
        }
        if installed_libretro && self.kept_build(row).is_some() {
            rows.push(CardRow::new(super::ROLL_BACK, "Roll Back…").pane(pane.clone()));
        }
        if installed_libretro {
            rows.push(CardRow::new(ECLIPSE_REMOVE, "Remove…").pane(pane.clone()));
        }
        if row.installed() {
            if let Some(dir) = row.library.as_deref().and_then(|l| Path::new(l).parent()).map(|p| p.to_string_lossy().into_owned()).filter(|s| !s.is_empty()) {
                rows.push(CardRow::new(&format!("{OPEN_FOLDER}{dir}"), "Open Folder").pane(pane.clone()));
            }
        }
        rows.push(CardRow::new(super::DETAILS, "Details").opens().pane(pane));
        rows
    }

    fn top_filter_keys(&self) -> Vec<&'static str> {
        let ready: Vec<&String> = self.order.iter().filter(|c| !self.installed_for(c).is_empty()).collect();
        let updates = self.rows.iter().filter(|r| r.state == "update").count();
        let fw = ready.iter().filter(|c| self.installed_for(c).iter().any(|r| self.core_needs_firmware_missing(c, r))).count();
        let approve = self.rows.iter().filter(|r| r.awaits_approval()).count();
        let mut keys = vec!["ready"];
        if updates > 0 {
            keys.push("updates");
        }
        if fw > 0 {
            keys.push("firmware");
        }
        if approve > 0 {
            keys.push("approval");
        }
        keys
    }

    fn visible_consoles(&self) -> Vec<String> {
        let ready: Vec<String> = self.order.iter().filter(|c| !self.installed_for(c).is_empty()).cloned().collect();
        if self.eclipse.showing_remaining {
            return self.order.iter().filter(|c| self.installed_for(c).is_empty()).cloned().collect();
        }
        match self.eclipse.filter {
            Filter::Ready => ready,
            Filter::Updates => ready.into_iter().filter(|c| self.installed_for(c).iter().any(|r| r.state == "update")).collect(),
            Filter::Firmware => ready.into_iter().filter(|c| self.installed_for(c).iter().any(|r| self.core_needs_firmware_missing(c, r))).collect(),
            Filter::Approval => ready.into_iter().filter(|c| self.installed_for(c).iter().any(|r| r.awaits_approval() || r.trust.as_deref() == Some("needs_refresh"))).collect(),
        }
    }

    fn aside_keys_for(&self, console: &str) -> Vec<String> {
        let mut keys: Vec<String> = self.installed_for(console).into_iter().map(|r| r.key.clone()).collect();
        keys.extend(self.available_for(console).into_iter().map(|r| r.key.clone()));
        keys
    }

    /// Handles D-pad navigation (`up`, `down`, `left`, `right`, `accept`) and action shortcuts in Eclipse `#10`.
    pub fn eclipse_navigate(&mut self, dir: &str) -> Effect {
        let top_keys = self.top_filter_keys();
        let top_len = top_keys.len() + 1;
        let visible = self.visible_consoles();
        let grid_idx = self
            .eclipse
            .focused_console
            .as_ref()
            .and_then(|fc| visible.iter().position(|c| c == fc))
            .unwrap_or(0);
        let current_console = visible.get(grid_idx).cloned().unwrap_or_default();
        let aside_keys = if current_console.is_empty() { Vec::new() } else { self.aside_keys_for(&current_console) };
        let aside_idx = self
            .eclipse
            .focused_core
            .as_ref()
            .and_then(|fk| aside_keys.iter().position(|k| k == fk))
            .unwrap_or(0);

        match (self.eclipse.focus_zone.as_str(), dir) {
            ("top", "left") => {
                self.eclipse.focused_top_index = self.eclipse.focused_top_index.saturating_sub(1);
            }
            ("top", "right") => {
                if self.eclipse.focused_top_index + 1 < top_len {
                    self.eclipse.focused_top_index += 1;
                }
            }
            ("top", "down") if !visible.is_empty() => {
                let target = self.eclipse.focused_top_index.min(visible.len() - 1).min(2);
                self.eclipse.focus_zone = "grid".to_owned();
                self.eclipse.focused_console = Some(visible[target].clone());
                self.eclipse.focused_core = None;
            }
            ("top", "accept") => {
                if let Some(&key) = top_keys.get(self.eclipse.focused_top_index) {
                    self.eclipse.showing_remaining = false;
                    self.eclipse.filter = Filter::from_key(key);
                    self.eclipse.focused_console = None;
                    self.eclipse.focused_core = None;
                } else {
                    self.eclipse.showing_remaining = !self.eclipse.showing_remaining;
                    self.eclipse.filter = Filter::Ready;
                    self.eclipse.focused_console = None;
                    self.eclipse.focused_core = None;
                }
            }
            ("grid", "up") => {
                if grid_idx >= 3 {
                    self.eclipse.focused_console = Some(visible[grid_idx - 3].clone());
                    self.eclipse.focused_core = None;
                } else {
                    self.eclipse.focus_zone = "top".to_owned();
                    self.eclipse.focused_top_index = grid_idx.min(top_keys.len());
                }
            }
            ("grid", "down") if !visible.is_empty() => {
                let next = if grid_idx + 3 < visible.len() {
                    Some(grid_idx + 3)
                } else if grid_idx / 3 < (visible.len() - 1) / 3 {
                    Some(visible.len() - 1)
                } else {
                    None
                };
                if let Some(i) = next {
                    self.eclipse.focused_console = Some(visible[i].clone());
                    self.eclipse.focused_core = None;
                }
            }
            ("grid", "left") if grid_idx % 3 > 0 => {
                self.eclipse.focused_console = Some(visible[grid_idx - 1].clone());
                self.eclipse.focused_core = None;
            }
            ("grid", "right") => {
                if grid_idx % 3 < 2 && grid_idx + 1 < visible.len() {
                    self.eclipse.focused_console = Some(visible[grid_idx + 1].clone());
                    self.eclipse.focused_core = None;
                } else if !aside_keys.is_empty() {
                    self.eclipse.focus_zone = "aside".to_owned();
                    self.eclipse.focused_core = Some(aside_keys[0].clone());
                }
            }
            ("grid", "accept") if !aside_keys.is_empty() => {
                self.eclipse.focus_zone = "aside".to_owned();
                self.eclipse.focused_core = Some(aside_keys[0].clone());
            }
            ("aside", "up") if aside_idx > 0 => {
                self.eclipse.focused_core = Some(aside_keys[aside_idx - 1].clone());
            }
            ("aside", "down") if aside_idx + 1 < aside_keys.len() => {
                self.eclipse.focused_core = Some(aside_keys[aside_idx + 1].clone());
            }
            ("aside", "left") => {
                self.eclipse.focus_zone = "grid".to_owned();
            }
            ("aside", "accept") if !aside_keys.is_empty() => {
                let key = aside_keys[aside_idx].clone();
                return self.eclipse_action("primary", Some(&key));
            }
            _ => {}
        }
        Effect::Done
    }

    /// Executes an Eclipse `#10` action immediately (confirmations for Install/Update/Remove/Approve/UpdateAll).
    pub fn eclipse_action(&mut self, action: &str, key: Option<&str>) -> Effect {
        let visible = self.visible_consoles();
        let console = self
            .eclipse
            .focused_console
            .as_ref()
            .filter(|fc| visible.contains(fc))
            .cloned()
            .or_else(|| visible.first().cloned())
            .unwrap_or_default();
        let target_key = key
            .filter(|k| !k.is_empty())
            .map(str::to_owned)
            .or_else(|| (self.eclipse.focus_zone == "aside").then(|| self.eclipse.focused_core.clone().or_else(|| self.aside_keys_for(&console).into_iter().next())).flatten())
            .or_else(|| (action == ECLIPSE_PAGE).then(|| self.aside_keys_for(&console).into_iter().next()).flatten());
        let row = target_key.as_deref().and_then(|k| self.row(k)).cloned();
        if self.job.is_some() && matches!(action, "primary" | ECLIPSE_UPDATE | ECLIPSE_INSTALL | ECLIPSE_REMOVE | ECLIPSE_PAGE | APPROVE | super::UPDATE_ALL) {
            return Effect::Done;
        }
        match (action, row) {
            ("primary", Some(r)) => {
                if r.awaits_approval() && self.hash_of(&r).is_some() {
                    self.eclipse_action(APPROVE, Some(&r.key))
                } else if r.is_libretro() && r.state == "update" {
                    self.eclipse_action(ECLIPSE_UPDATE, Some(&r.key))
                } else if r.is_libretro() && r.state == "available" {
                    self.eclipse_action(ECLIPSE_INSTALL, Some(&r.key))
                } else {
                    Effect::Page(plain_name(&r), self.eclipse_page(&r))
                }
            }
            (ECLIPSE_UPDATE, Some(r)) if r.is_libretro() && r.state == "update" => Effect::Ask(confirm_single(&r, plan::UPDATE), format!("{ECLIPSE_UPDATE}:{}", r.key)),
            (ECLIPSE_INSTALL, Some(r)) if r.is_libretro() && r.state == "available" => Effect::Ask(confirm_single(&r, plan::INSTALL), format!("{ECLIPSE_INSTALL}:{}", r.key)),
            (ECLIPSE_REMOVE, Some(r)) if r.is_libretro() => Effect::Ask(confirm_single(&r, plan::REMOVE), format!("{ECLIPSE_REMOVE}:{}", r.key)),
            (ECLIPSE_PAGE, Some(r)) => Effect::Page(plain_name(&r), self.eclipse_page(&r)),
            (DEFAULT, Some(r)) if r.engine.is_some() && !r.awaits_approval() && !self.is_default_on(&console, &r) => {
                self.make_default(&r.key, &format!("console:{console}"));
                Effect::Done
            }
            (APPROVE, Some(r)) => match self.hash_of(&r) {
                Some(sha) => Effect::Ask(super::approval::question(&self.asking(&r, sha)), format!("{APPROVE}:{}", r.key)),
                None => {
                    self.notice = Some(format!("{} cannot be approved yet: its SHA-256 is known once it is downloaded.", plain_name(&r)));
                    Effect::Done
                }
            },
            (super::UPDATE_ALL, _) => self.update_all_question().map_or(Effect::Done, |q| Effect::Ask(q, super::UPDATE_ALL.to_owned())),
            _ => Effect::Done,
        }
    }

    fn status_pill(&self) -> Value {
        if let Some((_, words)) = &self.progress {
            return json!({ "visible": true, "text": words, "error": false });
        }
        if let Some(why) = &self.catalogue_failure {
            return json!({ "visible": true, "text": words::refusal(why), "error": true });
        }
        if let Some(notice) = &self.notice {
            let err = notice.contains("could not") || notice.contains("refused") || notice.contains("did not");
            if err {
                return json!({ "visible": true, "text": notice, "error": true });
            }
        }
        json!({ "visible": false, "text": "", "error": false })
    }

    /// Serialized `#10` Eclipse view JSON consumed declaratively by `CoresEclipse.qml`.
    pub fn eclipse_projection(&self) -> Value {
        let current_year = words::current_year();
        let ready_consoles: Vec<&String> = self.order.iter().filter(|c| !self.installed_for(c).is_empty()).collect();
        let remaining_consoles: Vec<&String> = self.order.iter().filter(|c| self.installed_for(c).is_empty()).collect();
        let ready_count = ready_consoles.len();
        let remaining_count = remaining_consoles.len();
        let updates_count = self.rows.iter().filter(|r| r.state == "update").count();
        let approve_count = self.rows.iter().filter(|r| r.awaits_approval()).count();
        let firmware_count = ready_consoles.iter().filter(|c| self.installed_for(c).iter().any(|r| self.core_needs_firmware_missing(c, r))).count();

        let mut filters = vec![json!({
            "key": "ready",
            "label": counted(ready_count, "console ready", "consoles ready"),
            "attention": false,
            "active": !self.eclipse.showing_remaining && self.eclipse.filter == Filter::Ready
        })];
        if updates_count > 0 {
            filters.push(json!({
                "key": "updates",
                "label": counted(updates_count, "newer build", "newer builds"),
                "attention": true,
                "active": !self.eclipse.showing_remaining && self.eclipse.filter == Filter::Updates
            }));
        }
        if firmware_count > 0 {
            filters.push(json!({
                "key": "firmware",
                "label": if firmware_count == 1 { "1 needs your firmware".to_owned() } else { format!("{firmware_count} need your firmware") },
                "attention": false,
                "active": !self.eclipse.showing_remaining && self.eclipse.filter == Filter::Firmware
            }));
        }
        if approve_count > 0 {
            filters.push(json!({
                "key": "approval",
                "label": format!("{approve_count} to approve"),
                "attention": true,
                "active": !self.eclipse.showing_remaining && self.eclipse.filter == Filter::Approval
            }));
        }

        let visible_consoles = self.visible_consoles();

        let cards: Vec<Value> = visible_consoles
            .iter()
            .map(|c| {
                let installed = self.installed_for(c);
                let available = self.available_for(c);
                let count_words = self.count_for(c).map(games);
                let maker = self.maker_of.get(c).map(String::as_str).unwrap_or_default();
                let maker_line = match (maker.is_empty(), count_words) {
                    (false, Some(g)) => format!("{maker} · {g}"),
                    (false, None) => maker.to_owned(),
                    (true, Some(g)) => g,
                    (true, None) => String::new(),
                };
                if let Some(primary) = installed.first().copied() {
                    let has_approve = installed.iter().any(|r| r.awaits_approval() || r.trust.as_deref() == Some("needs_refresh"));
                    let has_update = installed.iter().any(|r| r.state == "update");
                    let has_fw = installed.iter().any(|r| self.core_needs_firmware_missing(c, r));
                    let (flag, flag_rose) = if has_approve {
                        ("To approve", true)
                    } else if has_update {
                        ("Newer build", true)
                    } else if has_fw {
                        ("Firmware", false)
                    } else {
                        ("", false)
                    };
                    let core_sub = match primary.origin {
                        Origin::Project => "EmuSen's own core",
                        Origin::Libretro => "Hosted core",
                        Origin::Added => "Added by you",
                    };
                    let others = &installed[1..];
                    let other_checkerboard = others.iter().any(|o| o.is_libretro());
                    let other_line = match others {
                        [] => "The only core installed".to_owned(),
                        [o] if o.awaits_approval() => format!("and {}, awaiting your approval", plain_name(o)),
                        [o] if o.state == "update" => format!("and {}, which has a newer build", plain_name(o)),
                        [o] if self.core_needs_firmware_missing(c, o) => {
                            if o.is_libretro() { "and 1 hosted core that needs your firmware".to_owned() } else { format!("and {} that needs your firmware", plain_name(o)) }
                        }
                        [o] => if o.is_libretro() { "and 1 hosted core, not in use".to_owned() } else { format!("and {}, not in use", plain_name(o)) },
                        many => {
                            if many.iter().any(|o| o.state == "update") {
                                format!("and {} other cores, with a newer build", many.len())
                            } else {
                                format!("and {} other cores, not in use", many.len())
                            }
                        }
                    };
                    json!({
                        "console": c,
                        "name": self.consoles.name(c),
                        "makerLine": maker_line,
                        "flag": flag,
                        "flagRose": flag_rose,
                        "planet": self.planet_for(primary),
                        "coreName": plain_name(primary),
                        "coreSubtitle": core_sub,
                        "otherCheckerboard": other_checkerboard,
                        "otherLine": other_line,
                    })
                } else {
                    let names: Vec<String> = available.iter().take(2).map(|r| r.name.clone()).collect();
                    let sub = if available.is_empty() {
                        "No hosted cores listed".to_owned()
                    } else {
                        counted(available.len(), "hosted core to get", "hosted cores to get")
                    };
                    json!({
                        "console": c,
                        "name": self.consoles.name(c),
                        "makerLine": maker_line,
                        "flag": "",
                        "flagRose": false,
                        "planet": -2,
                        "coreName": "No core installed",
                        "coreSubtitle": sub,
                        "otherCheckerboard": !available.is_empty(),
                        "otherLine": names.join(", "),
                    })
                }
            })
            .collect();

        let focused_console = self
            .eclipse
            .focused_console
            .as_ref()
            .filter(|fc| visible_consoles.iter().any(|c| c == *fc))
            .cloned()
            .or_else(|| visible_consoles.first().cloned());
        let focused_console_index = focused_console
            .as_deref()
            .and_then(|fc| visible_consoles.iter().position(|c| c == fc))
            .map_or(-1, |i| i as i32);

        let (aside, aside_keys, focused_item_meta) = if let Some(ref c) = focused_console {
            let installed = self.installed_for(c);
            let available = self.available_for(c);
            let maker = self.maker_of.get(c).map(String::as_str).unwrap_or_default();
            let count_words = self.count_for(c).map(games);
            let eyebrow = match (maker.is_empty(), count_words) {
                (false, Some(g)) => format!("{maker} · {g}"),
                (false, None) => maker.to_owned(),
                (true, Some(g)) => g,
                (true, None) => String::new(),
            };
            let installed_json: Vec<Value> = installed
                .iter()
                .map(|r| {
                    let is_def = self.is_default_on(c, r);
                    let approves = r.awaits_approval();
                    let has_update = r.state == "update";
                    let running_words = self.running_on(r).map(|_| self.progress.as_ref().map(|p| p.1.clone()).unwrap_or_else(|| "Working…".to_owned()));
                    let sub = if let Some(w) = running_words {
                        w
                    } else if approves {
                        match r.origin {
                            Origin::Added => "Added by you · awaiting your approval".to_owned(),
                            _ => "Hosted core · awaiting your approval".to_owned(),
                        }
                    } else if r.trust.as_deref() == Some("needs_refresh") {
                        "Hosted core · needs refresh".to_owned()
                    } else {
                        match r.origin {
                            Origin::Project => "EmuSen's own core · up to date".to_owned(),
                            Origin::Libretro => "Hosted core from libretro".to_owned(),
                            Origin::Added => "Added by you".to_owned(),
                        }
                    };
                    let (badge, badge_rose) = if is_def {
                        ("Plays this console", false)
                    } else if approves {
                        ("To approve", true)
                    } else {
                        ("", false)
                    };
                    let you_have = r.build.as_deref().and_then(|b| words::short_day_in(b, current_year)).or_else(|| r.build.clone()).unwrap_or_default();
                    let newer = r.offered.as_deref().and_then(|o| words::short_day_in(o, current_year)).or_else(|| r.offered.clone()).unwrap_or_default();
                    let can_default = r.engine.is_some() && !approves && !is_def && !r.consoles.is_empty();
                    let can_remove = r.is_libretro();
                    let can_approve = approves && self.hash_of(r).is_some();
                    json!({
                        "key": r.key,
                        "name": plain_name(r),
                        "planet": self.planet_for(r),
                        "subtitle": sub,
                        "badge": badge,
                        "badgeRose": badge_rose,
                        "hasUpdate": has_update,
                        "youHaveDate": you_have,
                        "newerDate": newer,
                        "canApprove": can_approve,
                        "canUpdate": has_update && r.is_libretro(),
                        "canDefault": can_default,
                        "canRemove": can_remove,
                    })
                })
                .collect();
            let available_json: Vec<Value> = available
                .iter()
                .map(|r| {
                    let desc = r.description.clone().filter(|d| !d.is_empty()).unwrap_or_else(|| "Hosted core from libretro".to_owned());
                    let size = r.size.filter(|&n| n > 0).map(words::size).unwrap_or_default();
                    json!({
                        "key": r.key,
                        "name": r.name,
                        "description": desc,
                        "size": size,
                    })
                })
                .collect();
            let mut keys: Vec<String> = installed.iter().map(|r| r.key.clone()).collect();
            keys.extend(available.iter().map(|r| r.key.clone()));
            let cname = self.consoles.name(c);
            let inst_part = counted(installed.len(), "core here", "cores here");
            let summary = if available.is_empty() {
                format!("{cname} · {inst_part}")
            } else {
                format!("{cname} · {inst_part}, {} more to get", available.len())
            };
            let avail_heading = if available.is_empty() {
                String::new()
            } else {
                format!("More from libretro · {} for this console", available.len())
            };
            let note = if available.is_empty() && !installed.iter().any(|r| r.is_libretro()) {
                String::new()
            } else {
                "A core you get is downloaded from libretro, kept apart in a process of its own, and asks for your say-so before its first game.".to_owned()
            };
            let selected_idx = self
                .eclipse
                .focused_core
                .as_deref()
                .and_then(|fk| keys.iter().position(|k| k == fk))
                .unwrap_or(0);
            let meta = if selected_idx < installed.len() {
                let r = installed[selected_idx];
                Some((
                    false,
                    r.awaits_approval() && self.hash_of(r).is_some(),
                    r.state == "update" && r.is_libretro(),
                    r.engine.is_some() && !r.awaits_approval() && !self.is_default_on(c, r) && !r.consoles.is_empty(),
                    r.is_libretro(),
                ))
            } else if selected_idx < keys.len() {
                Some((true, false, false, false, false))
            } else {
                None
            };
            (
                json!({
                    "console": c,
                    "eyebrow": eyebrow,
                    "heading": cname,
                    "installed": installed_json,
                    "availableHeading": avail_heading,
                    "available": available_json,
                    "note": note,
                    "footerSummary": summary,
                }),
                keys,
                meta,
            )
        } else {
            (
                json!({
                    "console": "",
                    "eyebrow": "",
                    "heading": if self.eclipse.showing_remaining { "No remaining consoles" } else { "No consoles match" },
                    "installed": [],
                    "availableHeading": "",
                    "available": [],
                    "note": "",
                    "footerSummary": "",
                }),
                Vec::new(),
                None,
            )
        };

        let focused_aside_index = if self.eclipse.focus_zone == "aside" && !aside_keys.is_empty() {
            self.eclipse.focused_core.as_deref().and_then(|fk| aside_keys.iter().position(|k| k == fk)).unwrap_or(0) as i32
        } else {
            -1
        };
        let focused_aside_key = if focused_aside_index >= 0 {
            aside_keys.get(focused_aside_index as usize).cloned().unwrap_or_default()
        } else {
            aside_keys.first().cloned().unwrap_or_default()
        };

        let mut footer_help: Vec<Value> = Vec::new();
        let push_help = |list: &mut Vec<Value>, btn: UiButton, label: &str| {
            list.push(json!({ "button": btn.index() as i32, "label": label, "pair": -1 }));
        };
        match (self.eclipse.focus_zone.as_str(), focused_item_meta) {
            ("aside", Some((is_avail, can_approve, can_update, can_default, can_remove))) => {
                if is_avail {
                    push_help(&mut footer_help, UiButton::Accept, "Install");
                } else if can_approve {
                    push_help(&mut footer_help, UiButton::Accept, "Approve");
                } else if can_update {
                    push_help(&mut footer_help, UiButton::Accept, "Update");
                } else {
                    push_help(&mut footer_help, UiButton::Accept, "Core page");
                }
                if can_default {
                    push_help(&mut footer_help, UiButton::Screensaver, "Use for this console");
                }
                if can_remove {
                    push_help(&mut footer_help, UiButton::Search, "Remove");
                }
            }
            ("top", _) => {
                push_help(&mut footer_help, UiButton::Accept, "Select");
                push_help(&mut footer_help, UiButton::Options, if self.eclipse.showing_remaining { "Ready consoles" } else { "More consoles" });
            }
            _ => {
                push_help(&mut footer_help, UiButton::Accept, "Open console");
                push_help(&mut footer_help, UiButton::Options, if self.eclipse.showing_remaining { "Ready consoles" } else { "More consoles" });
            }
        }
        if updates_count > 0 {
            push_help(&mut footer_help, UiButton::Menu, &format!("Update all · {updates_count}"));
        }
        push_help(&mut footer_help, UiButton::Back, "Back");

        let view_label = if self.eclipse.showing_remaining {
            format!("Show {}", counted(ready_count, "ready console", "ready consoles"))
        } else {
            format!("Get cores for {} more {}", crate::words::grouped(remaining_count), if remaining_count == 1 { "console" } else { "consoles" })
        };

        let focus_zone_code = match self.eclipse.focus_zone.as_str() {
            "top" => 0,
            "grid" => 1,
            _ => 2,
        };
        json!({
            "readyCount": ready_count,
            "remainingCount": remaining_count,
            "showingRemaining": self.eclipse.showing_remaining,
            "viewLabel": view_label,
            "filters": filters,
            "cards": cards,
            "focusZone": self.eclipse.focus_zone,
            "focusZoneCode": focus_zone_code,
            "focusedTopIndex": self.eclipse.focused_top_index.min(filters.len()),
            "focusedConsole": focused_console.unwrap_or_default(),
            "focusedConsoleIndex": focused_console_index,
            "focusedAsideIndex": focused_aside_index,
            "focusedAsideKey": focused_aside_key,
            "asideKeys": aside_keys,
            "aside": aside,
            "updateAllCount": updates_count,
            "footerHelp": footer_help,
            "statusPill": self.status_pill(),
            "actionLines": "toggleView Options\nupdateAll Menu\nuseDefault Screensaver\nremoveCore Search\ncorePage Random\n",
            "labels": {
                "tools": "Tools",
                "slash": "/",
                "cores": "Cores",
                "installedHeading": "On this machine",
                "emptyInstalled": "No core installed for this console yet.",
                "youHavePrefix": "You have ",
                "newerPrefix": "Newer ",
                "updatePill": "Update",
                "defaultPill": "Use for this console",
                "removePill": "Remove",
                "pagePill": "Core page",
            },
        })
    }

    pub fn eclipse_action_key(&mut self, key: &str) -> Effect {
        match key {
            "toggleView" => {
                self.eclipse_toggle_view();
                Effect::Done
            }
            "updateAll" => self.eclipse_action(super::UPDATE_ALL, None),
            "useDefault" => self.eclipse_action(DEFAULT, None),
            "removeCore" => self.eclipse_action(ECLIPSE_REMOVE, None),
            "corePage" => self.eclipse_action(ECLIPSE_PAGE, None),
            _ => Effect::Done,
        }
    }

    pub fn eclipse_nav_code(&mut self, code: i32) -> Effect {
        match code {
            0 => self.eclipse_navigate("up"),
            1 => self.eclipse_navigate("down"),
            2 => self.eclipse_navigate("left"),
            3 => self.eclipse_navigate("right"),
            4 => self.eclipse_navigate("accept"),
            _ => Effect::Done,
        }
    }

    pub fn eclipse_act_code(&mut self, code: i32, key: Option<&str>) -> Effect {
        match code {
            0 => self.eclipse_action(ECLIPSE_PAGE, key),
            1 => self.eclipse_action(ECLIPSE_UPDATE, key),
            2 => self.eclipse_action(DEFAULT, key),
            3 => self.eclipse_action(ECLIPSE_REMOVE, key),
            4 => self.eclipse_action(ECLIPSE_INSTALL, key),
            _ => Effect::Done,
        }
    }

    pub fn eclipse_zone_code(&mut self, code: i32) {
        self.eclipse.focus_zone = match code {
            0 => "top",
            1 => "grid",
            _ => "aside",
        }
        .to_owned();
    }

    pub(super) fn eclipse_toggle_view(&mut self) {
        self.eclipse.showing_remaining = !self.eclipse.showing_remaining;
        self.eclipse.filter = Filter::Ready;
        self.eclipse.focus_zone = "grid".to_owned();
        self.eclipse.focused_console = None;
        self.eclipse.focused_core = None;
    }
}
