//! Cores as a tool's source (EmuSen_Interface.md §15; EmuSen_Cores.md §2–§7): the project's cores, libretro's and the
//! player's own in one list, filtered by console and state, marked and applied through the core tool's jobs, approved and
//! made default through the runtime. The behaviour and the words are C#'s `CoresPane`'s; the frame's part is `Tool`'s.

pub mod approval;
pub mod backend;
pub mod plan;
pub mod row;
pub mod words;

use crate::card::{Pane as CardPane, Row as CardRow};
use crate::tools::{Action, Bar, Column, Effect, Item, Mark, Pane, Source, Tone, Words};
use approval::Asking;
use backend::{Backend, Job};
use row::{joined, of_runtime, of_tool, Consoles, Core, Origin};
use serde_json::{json, Value};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};

/// The console's setting naming its engine.
pub const ENGINE_KEY: &str = "Engine";

/// What the host opens for Cores; a destination it has not got is not offered.
#[derive(Clone, Copy, Debug, Default)]
pub struct Host {
    pub options: bool,
    pub firmware: bool,
    pub folder: bool,
}

/// The kinds of job Cores runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Listing,
    Fetch,
    Verify,
    Step,
}

/// The listing with each core's details, the catalogue's fetch and the check of every downloaded core.
pub fn listing_request() -> Value {
    json!({ "op": "core-ls", "show": "all", "detail": true })
}

pub fn fetch_request() -> Value {
    json!({ "op": "core-fetch" })
}

pub fn verify_request() -> Value {
    json!({ "op": "core-verify", "cores": [] })
}

/// The Cores source.
pub struct Cores {
    backend: Box<dyn Backend>,
    host: Host,
    consoles: Consoles,
    order: Vec<String>,
    engines_of: HashMap<String, Vec<String>>,
    firmware_of: HashMap<String, String>,
    have_firmware: RefCell<HashMap<String, Option<&'static str>>>,
    listing: Vec<Core>,
    tool_rows: Vec<Core>,
    rows: Vec<Core>,
    info: HashMap<String, Value>,
    outcomes: HashMap<String, String>,
    notice: Option<String>,
    catalogue: Option<String>,
    catalogue_failure: Option<String>,
    fetched_on_open: bool,
    acted_on: Option<String>,
    job: Option<(Box<dyn Job>, Kind)>,
    steps: VecDeque<plan::Step>,
    step: Option<plan::Step>,
    step_count: usize,
    steps_done: usize,
    checks: HashSet<String>,
    bytes: u64,
    total: u64,
    from: Option<String>,
    installed: Vec<String>,
    progress: Option<(f64, String)>,
}

fn text(e: &Value, name: &str) -> Option<String> {
    e.get(name).and_then(Value::as_str).map(str::to_owned)
}

fn number(e: &Value, name: &str) -> u64 {
    e.get(name).and_then(Value::as_u64).unwrap_or(0)
}

/// The host part of an address: `buildbot.libretro.com` of `https://buildbot.libretro.com/nightly/…`.
fn host_of(url: &str) -> Option<String> {
    let rest = url.split_once("://")?.1;
    let host = rest.split(['/', ':', '?']).next()?;
    (!host.is_empty()).then(|| host.to_owned())
}

impl Cores {
    pub fn new(backend: Box<dyn Backend>, host: Host) -> Cores {
        Cores {
            backend,
            host,
            consoles: Consoles::default(),
            order: Vec::new(),
            engines_of: HashMap::new(),
            firmware_of: HashMap::new(),
            have_firmware: RefCell::new(HashMap::new()),
            listing: Vec::new(),
            tool_rows: Vec::new(),
            rows: Vec::new(),
            info: HashMap::new(),
            outcomes: HashMap::new(),
            notice: None,
            catalogue: None,
            catalogue_failure: None,
            fetched_on_open: false,
            acted_on: None,
            job: None,
            steps: VecDeque::new(),
            step: None,
            step_count: 0,
            steps_done: 0,
            checks: HashSet::new(),
            bytes: 0,
            total: 0,
            from: None,
            installed: Vec::new(),
            progress: None,
        }
    }

    /// The rows as listed, joined and ordered.
    pub fn cores(&self) -> &[Core] {
        &self.rows
    }

    pub fn notice(&self) -> Option<&str> {
        self.notice.as_deref()
    }

    pub fn catalogue(&self) -> Option<&str> {
        self.catalogue.as_deref()
    }

    fn row(&self, key: &str) -> Option<&Core> {
        self.rows.iter().find(|r| r.key == key)
    }

    fn info_of(&self, row: &Core) -> Option<&Value> {
        row.stem.as_ref().and_then(|s| self.info.get(s))
    }

    /// The hash a row is approved by: the runtime's, else the one its downloaded entry records.
    fn hash_of(&self, row: &Core) -> Option<String> {
        row.sha256.clone().or_else(|| self.info_of(row).and_then(|i| text(i, "sha256")))
    }

    fn firmware_listed(&self, row: &Core) -> Vec<Value> {
        self.info_of(row).and_then(|i| i.get("firmware")).and_then(Value::as_array).cloned().unwrap_or_default()
    }

    fn refresh_keeping(&mut self, keep_notice: bool) {
        if self.job.is_some() {
            return;
        }
        if !keep_notice {
            self.notice = None;
        }
        self.backend.rescan();
        self.read_runtime();
        self.listing = self.runtime_rows();
        if self.start(listing_request(), Kind::Listing) {
            self.progress = Some((0.0, "Listing the cores…".to_owned()));
        } else {
            self.rows = self.ordered(self.listing.clone());
        }
    }

    fn read_runtime(&mut self) {
        self.consoles = Consoles::default();
        self.engines_of.clear();
        self.firmware_of.clear();
        self.have_firmware.borrow_mut().clear();
        let systems = match self.backend.systems() {
            Ok(s) => s,
            Err(e) => {
                self.notice = Some(format!("EmuSen's runtime did not list its systems: {e}"));
                Vec::new()
            }
        };
        let mut listed: Vec<(String, String, i32)> = Vec::new();
        for s in systems.iter().filter(|s| !s.id.is_empty() && !s.console.is_empty()) {
            self.consoles.console_of.insert(s.id.clone(), s.console.clone());
            if self.engines_of.contains_key(&s.console) {
                continue;
            }
            self.engines_of.insert(s.console.clone(), s.engines.clone());
            if !s.display_name.is_empty() {
                self.consoles.name_of.insert(s.console.clone(), s.display_name.clone());
            }
            if let Some(firmware) = s.firmware_name.clone().filter(|f| !f.is_empty()) {
                self.firmware_of.insert(s.console.clone(), firmware);
            }
            let year = if s.release_year > 0 { s.release_year } else { i32::MAX };
            listed.push((s.console.clone(), s.manufacturer.clone(), year));
        }
        // By maker in the table's order of them, then by release within a maker, the table's order breaking a tie.
        let mut makers: Vec<String> = Vec::new();
        for (_, maker, _) in &listed {
            if !makers.contains(maker) {
                makers.push(maker.clone());
            }
        }
        let mut ranked: Vec<(usize, &(String, String, i32))> = listed.iter().enumerate().collect();
        ranked.sort_by_key(|(at, l)| (makers.iter().position(|m| *m == l.1), l.2, *at));
        self.order = ranked.into_iter().map(|(_, l)| l.0.clone()).collect();
    }

    /// The console's engine: its stored setting, else the first engine discovery lists for it.
    fn is_default(&self, console: &str, engine: &str) -> bool {
        let first = self.engines_of.get(console).and_then(|e| e.first()).cloned();
        self.backend.engine(console).or(first).as_deref() == Some(engine)
    }

    fn runtime_rows(&mut self) -> Vec<Core> {
        let cores = match self.backend.cores() {
            Ok(c) => c,
            Err(e) => {
                self.notice = Some(format!("EmuSen's runtime did not list its cores: {e}"));
                return Vec::new();
            }
        };
        let backend = &self.backend;
        let size = |library: &str| backend.size(library);
        cores.iter().filter_map(|c| of_runtime(c, &self.consoles, &|console, engine| self.is_default(console, engine), &size)).collect()
    }

    fn rank(&self, console: &str) -> usize {
        self.order.iter().position(|c| c == console).unwrap_or(self.order.len())
    }

    /// EmuSen_Interface.md §15.3: by console in the strip's order, the console's default first, then what is installed
    /// and can run, then by name.
    fn ordered(&self, mut rows: Vec<Core>) -> Vec<Core> {
        let rank = |r: &Core| r.consoles.iter().map(|c| self.rank(c)).min().unwrap_or(usize::MAX);
        rows.sort_by(|a, b| {
            rank(a)
                .cmp(&rank(b))
                .then(b.is_default.cmp(&a.is_default))
                .then((b.installed() && !b.awaits_approval()).cmp(&(a.installed() && !a.awaits_approval())))
                .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });
        rows
    }

    fn start(&mut self, request: Value, kind: Kind) -> bool {
        match self.backend.start(&request) {
            Ok(job) => {
                self.job = Some((job, kind));
                true
            }
            Err(e) => {
                self.notice = Some(words::refusal(&e));
                false
            }
        }
    }

    fn check_for_updates(&mut self) {
        if self.job.is_some() {
            return;
        }
        self.notice = None;
        self.catalogue_failure = None;
        if self.start(fetch_request(), Kind::Fetch) {
            self.progress = Some((0.0, "Reading libretro's catalogue from buildbot.libretro.com…".to_owned()));
        } else {
            self.catalogue_failure = self.notice.clone();
        }
    }

    /// A home that has never read the catalogue reads it once when Cores is opened (EmuSen_Cores.md §4.2).
    fn read_catalogue_once(&mut self) {
        if self.catalogue.is_some() || self.fetched_on_open {
            return;
        }
        self.fetched_on_open = true;
        self.check_for_updates();
    }

    fn verify(&mut self) {
        if self.job.is_some() {
            return;
        }
        self.notice = None;
        self.outcomes.clear();
        if self.start(verify_request(), Kind::Verify) {
            self.progress = Some((0.0, "Checking the downloaded cores…".to_owned()));
        }
    }

    fn next_step(&mut self) {
        while let Some(step) = self.steps.pop_front() {
            self.checks.clear();
            self.bytes = 0;
            self.total = 0;
            self.from = None;
            let started = self.start(step.request.clone(), Kind::Step);
            let names: Vec<String> = step.rows.iter().filter_map(|k| self.row(k).map(|r| r.name.clone())).collect();
            if started {
                let share = self.steps_done as f64 / self.step_count.max(1) as f64;
                self.progress = Some((share, format!("{} {}…", step.doing, names.join(", "))));
                self.step = Some(step);
                return;
            }
            for key in &step.rows {
                self.outcomes.insert(key.clone(), self.notice.clone().unwrap_or_default());
            }
            self.steps_done += 1;
        }
        self.step = None;
        self.progress = None;
        if !self.installed.is_empty() {
            let after = self.notice.as_deref().map(|n| format!(" {n}")).unwrap_or_default();
            self.notice = Some(format!("Installed {}. Nothing downloaded runs until you approve it.{after}", self.installed.join(", ")));
        }
        self.refresh_keeping(true);
    }

    /// The hashes a job dropped from disk revoked through the runtime, since approvals are its calls, never a job's.
    fn revoke_all(&mut self, e: &Value) {
        let hashes: Vec<String> = e.get("revoke").and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_str).map(str::to_owned).collect()).unwrap_or_default();
        for sha in hashes {
            if let Err(why) = self.backend.revoke(&sha) {
                self.notice = Some(format!("The approval of {sha} could not be revoked: {why}"));
            }
        }
    }

    fn take(&mut self, e: &Value, kind: Kind) {
        let stem = text(e, "stem");
        let key = format!("libretro:{}", stem.clone().unwrap_or_default());
        let in_step = self.step.is_some();
        match (text(e, "event").as_deref().unwrap_or_default(), kind) {
            ("core", Kind::Listing) => {
                if let Some(row) = of_tool(e, &self.consoles) {
                    self.tool_rows.push(row);
                }
                if let Some(stem) = stem {
                    self.info.insert(stem, e.clone());
                }
            }
            ("done", _) => {
                if let Some(c) = e.get("catalogue") {
                    self.catalogue = c.as_str().map(str::to_owned);
                }
                if kind == Kind::Listing && self.catalogue.is_none() && self.notice.is_none() && self.catalogue_failure.is_some() {
                    self.notice = Some("No catalogue of libretro's cores yet: Check for Updates reads it from buildbot.libretro.com.".to_owned());
                }
            }
            ("refused", _) if stem.is_none() => {
                let reason = text(e, "reason").unwrap_or_default();
                self.notice = Some(words::refusal(&reason));
                if kind == Kind::Fetch {
                    self.catalogue_failure = Some(reason);
                }
            }
            ("refused", _) => {
                let reason = text(e, "reason").unwrap_or_default();
                self.outcomes.insert(key, format!("refused: {reason}"));
                self.notice = Some(format!("{}: {reason}", stem.unwrap_or_default()));
            }
            ("fetched", _) => {
                self.notice = Some(format!("Read libretro's catalogue on {}: {} cores for this machine.", text(e, "date").unwrap_or_default(), number(e, "cores")));
            }
            ("plan", _) if in_step => self.from = text(e, "from").and_then(|u| host_of(&u)),
            ("progress", _) if in_step => {
                let (bytes, total) = (number(e, "bytes"), number(e, "total"));
                (self.bytes, self.total) = (bytes, total);
                let within = if total > 0 { (bytes as f64 / total as f64).min(1.0) * 0.9 } else { 0.0 };
                let doing = self.step.as_ref().map_or("", |s| s.doing);
                let name = self.rows.iter().find(|r| r.stem == stem).map(|r| r.name.clone()).or(stem).unwrap_or_default();
                let of = if total > 0 { format!(" of {}", words::size(total)) } else { String::new() };
                let share = (self.steps_done as f64 + within) / self.step_count.max(1) as f64;
                self.progress = Some((share, format!("{doing} {name}: {}{of}", words::size(bytes))));
            }
            ("checked", _) if in_step => {
                let check = text(e, "check").unwrap_or_default();
                self.checks.insert(check.clone());
                let doing = self.step.as_ref().map_or("", |s| s.doing);
                let share = (self.steps_done as f64 + 0.95) / self.step_count.max(1) as f64;
                self.progress = Some((share, format!("{doing} {}: checking its {check}", stem.unwrap_or_default())));
            }
            ("installed", _) => {
                let name = self.rows.iter().find(|r| r.stem == stem).map(|r| r.name.clone()).or(stem).unwrap_or_default();
                self.installed.push(name);
                self.revoke_all(e);
            }
            ("removed", _) => {
                self.outcomes.insert(key, "removed".to_owned());
                self.revoke_all(e);
            }
            ("current", _) => {
                self.outcomes.insert(key, "already current".to_owned());
            }
            ("verified", _) => {
                let result = text(e, "result").unwrap_or_default();
                let said = match result.as_str() {
                    "refreshed" => "refreshed".to_owned(),
                    "changed" => "changed: needs approval".to_owned(),
                    other => other.to_owned(),
                };
                self.outcomes.insert(key, said);
                if result == "changed" {
                    self.notice = Some(format!("{}: {}", stem.unwrap_or_default(), text(e, "reason").unwrap_or_default()));
                }
            }
            _ => {}
        }
    }

    /// The step running on a row, none when none is (EmuSen_Interface.md §15.3).
    fn running_on(&self, row: &Core) -> Option<&plan::Step> {
        self.step.as_ref().filter(|s| s.rows.contains(&row.key))
    }

    /// The status column: the step running on the row, a run's outcome for it, else the status words.
    fn status_of(&self, row: &Core) -> Words {
        if let Some(step) = self.running_on(row) {
            return Words::new(format!("{}…", step.doing), Tone::Muted);
        }
        if let Some(said) = self.outcomes.get(&row.key).filter(|s| !s.is_empty()) {
            return Words::new(words::capital(said), if said.starts_with("refused") { Tone::Error } else { Tone::Muted });
        }
        Words::new(words::status(row), words::tone(row))
    }

    fn belongs(row: &Core, filter: &str) -> bool {
        match filter {
            "installed" => row.installed(),
            "available" => row.state == "available",
            "updates" => row.state == "update",
            "approval" => row.awaits_approval(),
            "unsupported" => matches!(row.state.as_str(), "unsupported" | "not_read"),
            _ => true,
        }
    }

    fn make_default(&mut self, key: &str, section: &str) {
        let Some(row) = self.row(key).cloned() else { return };
        let Some(engine) = row.engine.clone() else { return };
        let chosen = section.strip_prefix("console:").filter(|c| row.consoles.iter().any(|r| r == c));
        let consoles: Vec<String> = chosen.map_or_else(|| row.consoles.clone(), |c| vec![c.to_owned()]);
        for console in consoles {
            if let Err(e) = self.backend.set_engine(&console, &engine) {
                self.notice = Some(format!("{} could not be made the {}'s engine: {e}", row.name, self.consoles.name(&console)));
            }
        }
        let mut rows = std::mem::take(&mut self.rows);
        for r in &mut rows {
            r.is_default = r.engine.as_deref().is_some_and(|en| !r.awaits_approval() && r.consoles.iter().any(|c| self.is_default(c, en)));
        }
        self.rows = rows;
    }

    /// The approval's words for a row (EmuSen_Interface.md §15.6).
    fn asking(&self, row: &Core, sha256: String) -> Asking {
        let info = self.info_of(row);
        let file = if row.is_libretro() {
            format!("{}_libretro", row.stem.clone().unwrap_or_default())
        } else {
            row.library.as_deref().and_then(|l| std::path::Path::new(l).file_name()).map(|f| f.to_string_lossy().into_owned()).unwrap_or_else(|| row.name.clone())
        };
        let from = info.and_then(|i| text(i, "from")).and_then(|u| host_of(&u)).or_else(|| row.is_libretro().then(|| "buildbot.libretro.com".to_owned()));
        let when = info.and_then(|i| text(i, "fetched")).map(|f| words::long_day(&f).map_or(f, |d| format!("on {d}")));
        Asking {
            name: row.name.clone(),
            file,
            authors: row.authors.clone(),
            licence: row.licence.clone(),
            consoles: row.console_names.clone(),
            sha256,
            downloaded: row.is_libretro(),
            from,
            when,
            tested: info.and_then(tested),
        }
    }

    /// The firmware of the row's first console as the runtime finds it: present where any file is, else missing or
    /// not needed; none where it cannot say.
    fn have_firmware(&self, row: &Core) -> Option<&'static str> {
        let first = row.consoles.first()?;
        let system = row.systems.iter().find(|s| self.consoles.console_of.get(*s) == Some(first)).or(row.systems.first())?;
        if let Some(known) = self.have_firmware.borrow().get(system) {
            return *known;
        }
        let found: Vec<String> = self.backend.firmware(system).unwrap_or_default().into_iter().map(|f| f.status).collect();
        let have = if found.is_empty() {
            None
        } else if found.iter().any(|s| s == "present") {
            Some("present")
        } else if found.iter().any(|s| s == "missing") {
            Some("missing")
        } else {
            Some("not_needed")
        };
        self.have_firmware.borrow_mut().insert(system.clone(), have);
        have
    }

    /// Why a core that could run its console is not its engine, and what makes it one.
    fn not_the_engine(&self, row: &Core) -> Option<String> {
        if row.is_default || row.awaits_approval() {
            return None;
        }
        let (engine, console) = (row.engine.as_ref()?, row.consoles.first()?);
        let first = self.engines_of.get(console).and_then(|e| e.first()).cloned();
        let current = self.backend.engine(console).or(first)?;
        if &current == engine {
            return None;
        }
        let other = self.rows.iter().find(|r| r.engine.as_ref() == Some(&current)).map_or(current.clone(), |r| r.name.clone());
        let development = if row.development { " while it is under development" } else { "" };
        Some(format!("Not the {}'s engine{development}: {other} is. A makes {} the engine instead.", self.consoles.name(console), row.name))
    }

    fn overview(&self) -> String {
        if let Some(notice) = &self.notice {
            return notice.clone();
        }
        let built = self.rows.iter().filter(|r| r.origin == Origin::Project).count();
        let libretro = self.rows.iter().filter(|r| r.is_libretro()).count();
        let installed = self.rows.iter().filter(|r| r.is_libretro() && r.installed()).count();
        let fetched = self.catalogue.as_ref().map_or("libretro's catalogue has not been read yet".to_owned(), |c| format!("libretro's catalogue was read on {c}"));
        format!("{built} cores ship with EmuSen. {libretro} libretro cores are listed, {installed} of them installed; {fetched}.")
    }

    /// The pane: the core in sentences, nothing raw, each fact once (EmuSen_Interface.md §15.4).
    fn detail(&self, row: &Core) -> Vec<Words> {
        let mut said = Vec::new();
        let info = self.info_of(row);
        if matches!(self.running_on(row).map(|s| s.doing), Some("Installing" | "Updating")) {
            return self.job_lines(row);
        }
        let runs = if row.consoles.is_empty() { "games of no console EmuSen knows".to_owned() } else { format!("{} games", row.console_names.join(" and ")) };
        said.push(Words::text(match row.origin {
            Origin::Libretro => format!("Runs {runs} through EmuSen's libretro host, in a sandbox of its own."),
            Origin::Project => format!("Runs {runs}. Built into EmuSen and updated with it, so there is nothing to install or approve."),
            Origin::Added => format!("Runs {runs}, from a file of your own."),
        }));
        let by = if row.authors.is_empty() { "By authors not said".to_owned() } else { format!("By {}", row.authors.join(", ")) };
        let licence = if row.licence.is_empty() { ", licence not said".to_owned() } else { format!(", under the {}", row.licence) };
        let build = match (row.version.is_empty(), row.is_libretro()) {
            (true, _) => String::new(),
            (false, true) => {
                let last = row.version.split(' ').next_back().unwrap_or_default();
                words::long_day(last).map_or_else(|| format!(" Build {}", row.version), |d| format!(" Nightly build of {d}"))
            }
            (false, false) => format!(" Version {}", row.version),
        };
        let size = row.size.filter(|n| *n > 0).map(|n| format!(", {}", words::size(n))).unwrap_or_default();
        let fetched = info.and_then(|i| text(i, "fetched")).map(|w| format!(", downloaded {w}")).unwrap_or_default();
        let end = if build.len() + size.len() + fetched.len() > 0 { "." } else { "" };
        said.push(Words::text(format!("{by}{licence}.{build}{size}{fetched}{end}")));
        let before = said.len();
        let listed = self.firmware_listed(row);
        for f in &listed {
            let optional = if f.get("optional").and_then(Value::as_bool).unwrap_or(false) { " (optional)" } else { "" };
            let desc = text(f, "desc").unwrap_or_else(|| "firmware".to_owned());
            said.push(Words::text(format!("Needs {} of your own{optional}, which EmuSen never downloads.", words::article(&desc))));
        }
        if let (true, Some(need), Some(kind)) = (listed.is_empty(), row.firmware.as_deref(), row.consoles.first().and_then(|c| self.firmware_of.get(c))) {
            said.push(Words::text(if need == "required" {
                format!("Needs {} of your own, which EmuSen never downloads.", words::article(kind))
            } else {
                format!("Can use {} of your own, which EmuSen never downloads; it runs without one.", words::article(kind))
            }));
        }
        if said.len() > before {
            match self.have_firmware(row) {
                Some("present") => said.push(Words::new("✓ You have one.", Tone::Success)),
                Some("missing") => said.push(Words::new("⚠ None found yet. Firmware, under X, says where it goes.", Tone::Attention)),
                _ => {}
            }
        }
        if row.awaits_approval() {
            said.push(Words::new("⚠ Waiting for your approval. Nothing downloaded runs until you approve it.", Tone::Attention));
        } else if row.trust.as_deref() == Some("needs_refresh") {
            said.push(Words::new("⚠ EmuSen's copies beside it are older than this build: Verify refreshes them.", Tone::Attention));
        } else if matches!(row.state.as_str(), "unsupported" | "not_read") {
            let why = row.reason.as_ref().map(|w| format!(": {w}")).unwrap_or_default();
            said.push(Words::new(format!("Not supported{why}."), Tone::Muted));
        } else if row.is_libretro() && row.installed() && self.hash_of(row).is_some() {
            said.push(Words::new("✓ Approved, held to its hash.", Tone::Success));
        } else if row.is_libretro() && !row.installed() {
            said.push(Words::new("Its SHA-256 is known once it is downloaded; nothing downloaded runs until you approve it.", Tone::Muted));
        }
        if let Some(t) = info.and_then(tested) {
            said.push(Words::new(t, Tone::Muted));
        }
        if row.is_default && !row.consoles.is_empty() {
            let console = &row.console_names[0];
            said.push(Words::text(format!("● The {console}'s engine: every {console} game opens on it unless the game's own options choose another.")));
        } else if row.state == "update" {
            said.push(Words::text("A newer build is offered; the build installed now is kept for Roll Back."));
        } else if let Some(instead) = self.not_the_engine(row) {
            said.push(Words::text(instead));
        }
        if let Some(previous) = info.and_then(|i| i.get("previous")) {
            said.push(Words::text(format!("The build of {} is kept for Roll Back.", text(previous, "build").unwrap_or_default())));
        }
        if row.is_libretro() && row.engine.is_none() && row.installed() && !row.awaits_approval() {
            said.push(Words::new("It becomes a choice for its consoles once EmuSen's runtime lists it.", Tone::Muted));
        }
        if let Some(outcome) = self.outcomes.get(&row.key).filter(|o| !o.is_empty()) {
            let tone = if outcome.starts_with("refused") { Tone::Error } else { Tone::Muted };
            said.push(Words::new(format!("{}.", words::capital(outcome)), tone));
        }
        said
    }

    /// While a core is fetched, the steps in the order the tool checks them, each ticked as its event arrives (§15.4 7).
    fn job_lines(&self, row: &Core) -> Vec<Words> {
        let host = self.from.clone().unwrap_or_else(|| "buildbot.libretro.com".to_owned());
        let last = row.version.split(' ').next_back().unwrap_or_default();
        let day = words::long_day(last).map_or_else(|| "its build".to_owned(), |d| format!("the nightly build of {d}"));
        let total = if self.total > 0 { format!(", {}", words::size(self.total)) } else { String::new() };
        let step = |words: String, done: bool| if done { Words::new(format!("✓ {words}"), Tone::Success) } else { Words::new(format!("· {words}"), Tone::Muted) };
        let transport = self.checks.contains("transport");
        let download = if transport {
            step(format!("Downloaded, {}", words::size(self.total)), true)
        } else {
            let of = if self.total > 0 { format!(" of {}", words::size(self.total)) } else { String::new() };
            Words::text(format!("↓ Download · {}{of}", words::size(self.bytes)))
        };
        vec![
            Words::text(format!("Downloading {day} from {host}{total}.")),
            // A byte arrives only over the checked connection, so the first one says it is made.
            step(format!("Secure connection to {host}"), self.bytes > 0 || transport),
            download,
            step("Checksum, one file of the right name, its size".to_owned(), self.checks.contains("members")),
            step("Built for this machine".to_owned(), self.checks.contains("architecture")),
            step("SHA-256 recorded".to_owned(), self.checks.contains("sha256")),
            Words::text("Then it waits for your approval: nothing downloaded runs until you approve it."),
        ]
    }

    /// The Details page: every fact as the tool holds it (EmuSen_Interface.md §15.5), a row each with its words whole
    /// in the pane.
    fn details(&self, row: &Core) -> Vec<CardRow> {
        let mut facts: Vec<(&str, String)> = Vec::new();
        if let Some(listed) = &row.listed_as {
            facts.push(("Listed by libretro as", listed.clone()));
        }
        if let Some(stem) = &row.stem {
            facts.push(("File stem", stem.clone()));
        }
        if let Some(engine) = &row.engine {
            facts.push(("Engine", engine.clone()));
        }
        if let Some(library) = &row.library {
            facts.push(("Library", library.clone()));
        }
        if let Some(info) = self.info_of(row) {
            if let Some(from) = text(info, "from") {
                facts.push(("From", format!("{from}, index CRC-32 {}", text(info, "index_crc32").unwrap_or_default())));
            }
            if let Some(fetched) = text(info, "fetched") {
                facts.push(("Downloaded", fetched));
            }
            for f in self.firmware_listed(row) {
                let need = if f.get("optional").and_then(Value::as_bool).unwrap_or(false) { "optional" } else { "required" };
                facts.push(("Firmware", format!("{} at {}, {need}", text(&f, "desc").unwrap_or_default(), text(&f, "path").unwrap_or_default())));
            }
            let notes: Vec<String> = info.get("notes").and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_str).map(str::to_owned).collect()).unwrap_or_default();
            if !notes.is_empty() {
                facts.push(("Notes", notes.join(" · ")));
            }
            let asks: Vec<String> = info.get("permissions").and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_str).map(str::to_owned).collect()).unwrap_or_default();
            if !asks.is_empty() {
                facts.push(("Asks for", asks.join(", ")));
            }
        }
        if let Some(sha) = self.hash_of(row) {
            let waiting = if row.awaits_approval() { " · not yet approved" } else { "" };
            facts.push(("SHA-256", format!("{}{waiting}", words::grouped(&sha).replace('\n', " "))));
        }
        if let Some(trust) = &row.trust {
            facts.push(("Trust", trust.clone()));
        }
        facts.into_iter().enumerate().map(|(at, (label, said))| CardRow::new(&format!("detail.{at}"), label).pane(CardPane::said(&said, ""))).collect()
    }
}

/// What the project found of the build, one owner for the pane and the approval (EmuSen_Cores.md §4.6).
fn tested(info: &Value) -> Option<String> {
    let t = info.get("tested")?;
    let date = text(t, "date").unwrap_or_default();
    Some(match text(t, "match").as_deref() {
        Some("exact") => format!("This exact build was tested by EmuSen on {date}."),
        Some("other") => format!("An earlier build was tested by EmuSen on {date}."),
        _ => "Not tested by EmuSen.".to_owned(),
    })
}

/// Page rows' and actions' keys.
pub const APPROVE: &str = "approve";
pub const DEFAULT: &str = "default";
pub const OPTIONS: &str = "options";
pub const FIRMWARE: &str = "firmware";
pub const FOLDER: &str = "folder";
pub const VERIFY: &str = "verify";
pub const CHECK: &str = "check";
pub const DETAILS: &str = "details";
const MARK: &str = "mark:";
const UNMARK: &str = "unmark";
const REMOVE_ASK: &str = "remove";

impl Source for Cores {
    fn title(&self) -> &str {
        "Cores"
    }

    fn section_word(&self) -> &str {
        "Console"
    }

    /// Core, Console and Status (EmuSen_Interface.md §15.2); the name's least holds `Mupen64Plus-Next` (190).
    fn columns(&self) -> Vec<Column> {
        vec![
            Column { title: "Core", fill: 1.0, least: 190.0, trailing: false },
            Column { title: "Console", fill: 0.0, least: 0.0, trailing: false },
            Column { title: "Status", fill: 0.0, least: 0.0, trailing: true },
        ]
    }

    fn items(&self) -> Vec<Item> {
        self.rows
            .iter()
            .map(|r| {
                let status = self.status_of(r);
                Item {
                    key: r.key.clone(),
                    sections: r.consoles.iter().map(|c| format!("console:{c}")).collect(),
                    filters: ["installed", "available", "updates", "approval", "unsupported"].iter().filter(|f| Self::belongs(r, f)).map(|f| (*f).to_owned()).collect(),
                    search: [Some(r.name.clone()), r.listed_as.clone(), r.stem.clone(), r.engine.clone(), Some(r.consoles.join(" ")), Some(r.licence.clone())]
                        .into_iter()
                        .flatten()
                        .collect::<Vec<_>>()
                        .join(" "),
                    sorts: vec![r.name.clone(), status.words.clone()],
                    cells: vec![Words::text(r.name.clone()), Words::text(words::consoles(r)), status],
                }
            })
            .collect()
    }

    fn sections(&self) -> Vec<(String, String)> {
        let mut consoles: Vec<&String> = Vec::new();
        for c in self.rows.iter().flat_map(|r| &r.consoles) {
            if !consoles.contains(&c) {
                consoles.push(c);
            }
        }
        consoles.sort_by(|a, b| self.rank(a).cmp(&self.rank(b)).then(a.to_lowercase().cmp(&b.to_lowercase())));
        let mut sections = vec![("all".to_owned(), "All".to_owned())];
        sections.extend(consoles.into_iter().map(|c| (format!("console:{c}"), self.consoles.name(c))));
        sections
    }

    fn filters(&self) -> Vec<(String, String)> {
        [("all", "All"), ("installed", "Installed"), ("available", "Available"), ("updates", "Updates"), ("approval", "To approve"), ("unsupported", "Not supported")]
            .iter()
            .map(|(k, l)| ((*k).to_owned(), (*l).to_owned()))
            .collect()
    }

    fn sorts(&self) -> Vec<(String, String)> {
        [("console", "Console"), ("name", "Name"), ("status", "Status")].iter().map(|(k, l)| ((*k).to_owned(), (*l).to_owned())).collect()
    }

    /// The counts that need the player first, then the catalogue's date (EmuSen_Interface.md §15.2).
    fn facts(&self) -> Vec<Words> {
        let installed = self.rows.iter().filter(|r| r.installed()).count();
        let approve = self.rows.iter().filter(|r| r.awaits_approval()).count();
        let updates = self.rows.iter().filter(|r| r.state == "update").count();
        let mut facts = vec![Words::new(format!("{} cores", self.rows.len()), Tone::Muted), Words::new(format!("{installed} installed"), Tone::Muted)];
        if approve > 0 {
            facts.push(Words::new(format!("⚠ {approve} to approve"), Tone::Attention));
        }
        if updates > 0 {
            facts.push(Words::new(format!("↑ {updates} {}", if updates == 1 { "update" } else { "updates" }), Tone::Muted));
        }
        let catalogue = match (&self.catalogue, &self.catalogue_failure) {
            (Some(c), _) => format!("catalogue read {}", words::day(c)),
            (None, Some(why)) => format!("catalogue could not be read: {}", why.trim_end_matches('.')),
            (None, None) => "catalogue not read yet".to_owned(),
        };
        facts.push(Words::new(catalogue, Tone::Muted));
        facts
    }

    fn pane(&self, key: Option<&str>) -> Pane {
        let Some(row) = key.and_then(|k| self.row(k)) else {
            return Pane { sentences: vec![Words::text(self.overview())], ..Pane::default() };
        };
        let source = match row.origin {
            Origin::Project => "Ships with EmuSen",
            Origin::Added => "Added by you",
            Origin::Libretro => "From libretro's buildbot",
        };
        let listed = row.listed_as.as_ref().map(|l| format!(" · listed there as “{l}”")).unwrap_or_default();
        Pane { title: row.name.clone(), caption: format!("{source} · {}{listed}", words::consoles(row)), sentences: self.detail(row) }
    }

    fn bar(&self) -> Bar {
        let (line, progress) = match &self.progress {
            Some((share, said)) => (Words::text(said.clone()), Some(*share)),
            None => (Words::text(self.notice.clone().unwrap_or_default()), None),
        };
        let (meter, meter_words) = match self.backend.space().filter(|s| s.1 > 0) {
            Some((free, total)) => {
                let cores: u64 = self.rows.iter().filter(|r| r.installed() && r.origin != Origin::Project).map(|r| r.size.unwrap_or(0)).sum();
                (1.0 - free as f64 / total as f64, format!("/Cores {} · {} free", words::size(cores), words::size(free)))
            }
            None => (0.0, String::new()),
        };
        Bar { line, progress, meter, meter_words }
    }

    /// The row's one action on the big screen (EmuSen_Interface.md §15.5).
    fn action(&self, key: &str) -> Option<Action> {
        let row = self.row(key)?;
        let engine = row.engine.is_some() && !row.consoles.is_empty();
        if row.awaits_approval() && self.hash_of(row).is_some() {
            Some(Action::new(APPROVE, "Approve…"))
        } else if row.is_libretro() && row.state == "available" {
            Some(Action::new(plan::INSTALL, "Install…"))
        } else if row.is_libretro() && row.state == "update" {
            Some(Action::new(plan::UPDATE, "Update…"))
        } else if engine && !row.is_default {
            Some(Action::new(DEFAULT, "Make Default"))
        } else if engine && self.host.options {
            Some(Action::new(OPTIONS, "Options"))
        } else {
            None
        }
    }

    /// The row's page under X: everything else it offers, each where it applies (EmuSen_Interface.md §15.5).
    fn page(&self, key: &str, marked: Option<&Mark>) -> Vec<CardRow> {
        let Some(row) = self.row(key) else { return Vec::new() };
        let mut rows = Vec::new();
        for mark in plan::marks_for(row) {
            rows.push(if marked == Some(&mark) {
                CardRow::new(UNMARK, &format!("Unmark ({})", mark.label))
            } else {
                CardRow::new(&format!("{MARK}{}", mark.key), &format!("Mark to {}", mark.label))
            });
        }
        let engine = row.engine.is_some() && !row.consoles.is_empty();
        let installed_libretro = row.is_libretro() && row.installed();
        if engine && self.host.options {
            rows.push(CardRow::new(OPTIONS, "Options").opens());
        }
        if !self.firmware_listed(row).is_empty() && self.host.firmware {
            rows.push(CardRow::new(FIRMWARE, "Firmware").opens());
        }
        if self.hash_of(row).is_some() && (row.awaits_approval() || installed_libretro) {
            rows.push(CardRow::new(APPROVE, "Approve…"));
        }
        if installed_libretro {
            rows.push(CardRow::new(VERIFY, "Verify"));
            rows.push(CardRow::new(REMOVE_ASK, "Remove…"));
        }
        if row.installed() && self.host.folder {
            rows.push(CardRow::new(FOLDER, "Open Folder"));
        }
        rows.push(CardRow::new(DETAILS, "Details").opens());
        rows
    }

    fn list_rows(&self) -> Vec<CardRow> {
        vec![CardRow::new(CHECK, "Check for Updates"), CardRow::new(VERIFY, "Verify")]
    }

    fn marks_for(&self, key: &str) -> Vec<Mark> {
        self.row(key).map(plan::marks_for).unwrap_or_default()
    }

    fn summary(&self, marked: &[(String, Mark)]) -> String {
        let pairs: Vec<(&Core, &Mark)> = marked.iter().filter_map(|(k, m)| Some((self.row(k)?, m))).collect();
        plan::summary(&pairs)
    }

    fn plan(&self, marked: &[(String, Mark)]) -> crate::question::Question {
        let pairs: Vec<(&Core, &Mark)> = marked.iter().filter_map(|(k, m)| Some((self.row(k)?, m))).collect();
        crate::tools::plan_question(&plan::title(&pairs), &plan::sentences(&pairs))
    }

    fn apply(&mut self, marked: &[(String, Mark)]) {
        let pairs: Vec<(&Core, &Mark)> = marked.iter().filter_map(|(k, m)| Some((self.row(k)?, m))).collect();
        let steps = plan::steps(&pairs);
        self.outcomes.clear();
        self.installed.clear();
        self.notice = None;
        self.acted_on = marked.first().map(|(k, _)| k.clone());
        self.step_count = steps.len();
        self.steps = steps.into();
        self.steps_done = 0;
        self.next_step();
    }

    fn act(&mut self, action: &str, key: Option<&str>, section: &str) -> Effect {
        let row = key.and_then(|k| self.row(k)).cloned();
        match (action, row) {
            (CHECK, _) => {
                self.check_for_updates();
                Effect::Done
            }
            (VERIFY, _) => {
                self.verify();
                Effect::Done
            }
            (APPROVE, Some(row)) => match self.hash_of(&row) {
                Some(sha) => Effect::Ask(approval::question(&self.asking(&row, sha)), format!("{APPROVE}:{}", row.key)),
                None => {
                    self.notice = Some(format!("{} cannot be approved yet: its SHA-256 is known once it is downloaded.", row.name));
                    Effect::Done
                }
            },
            (plan::INSTALL | plan::UPDATE | REMOVE_ASK, Some(_)) => Effect::Mark(action.to_owned(), true),
            (UNMARK, Some(_)) => Effect::Unmark,
            (DEFAULT, Some(row)) => {
                self.make_default(&row.key, section);
                Effect::Done
            }
            (DETAILS, Some(row)) => Effect::Page(format!("{} · details", row.name), self.details(&row)),
            (OPTIONS | FIRMWARE | FOLDER, Some(_)) => Effect::Host(action.to_owned()),
            (other, Some(_)) if other.starts_with(MARK) => Effect::Mark(other[MARK.len()..].to_owned(), false),
            _ => Effect::Done,
        }
    }

    fn answer(&mut self, asked: &str, yes: bool) {
        let Some(key) = asked.strip_prefix(&format!("{APPROVE}:")) else { return };
        let Some(row) = self.row(key).cloned() else { return };
        let Some(sha) = self.hash_of(&row).filter(|_| yes) else { return };
        match self.backend.approve(&sha) {
            Ok(()) => {
                self.notice = Some(format!("Approved {} (SHA-256 {sha}).", row.name));
                self.outcomes.remove(&row.key);
                self.acted_on = Some(row.key.clone());
                self.refresh_keeping(true);
            }
            Err(e) => self.notice = Some(format!("{} could not be approved: {e}", row.name)),
        }
    }

    fn refresh(&mut self) {
        self.refresh_keeping(false);
    }

    fn poll(&mut self) -> bool {
        let Some((mut job, kind)) = self.job.take() else { return false };
        let mut events = Vec::new();
        let ended = match job.drain(&mut events) {
            Ok(ended) => ended,
            Err(e) => {
                self.notice = Some(words::refusal(&e));
                if kind == Kind::Fetch {
                    self.catalogue_failure = Some(e);
                }
                true
            }
        };
        for e in &events {
            self.take(e, kind);
        }
        if !ended {
            self.job = Some((job, kind));
            return !events.is_empty();
        }
        drop(job);
        match kind {
            Kind::Listing => {
                self.progress = None;
                let tool = std::mem::take(&mut self.tool_rows);
                self.rows = self.ordered(joined(&self.listing, &tool));
                self.read_catalogue_once();
            }
            Kind::Fetch | Kind::Verify => {
                self.progress = None;
                self.refresh_keeping(true);
            }
            Kind::Step => {
                self.steps_done += 1;
                self.next_step();
            }
        }
        true
    }

    fn running(&self) -> bool {
        self.job.is_some()
    }

    fn acted_on(&mut self) -> Option<String> {
        if self.job.is_some() {
            return None;
        }
        self.acted_on.take()
    }
}
