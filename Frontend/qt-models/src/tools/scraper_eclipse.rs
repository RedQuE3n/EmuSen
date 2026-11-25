//! Eclipse Scraper (#66) and Scrape Status (#67) state and board JSON projection.

use super::{
    eclipse_aside_line, eclipse_columns, eclipse_console_lookup, eclipse_filter, eclipse_help_entry, eclipse_move_row,
    eclipse_row, eclipse_step_filter, eclipse_step_row_up, eclipse_table_help, eclipse_trail, Effect,
};
use crate::card::{Pane as CardPane, Row as CardRow};
use crate::clients::ClientSource;
use crate::library::LibrarySource;
use crate::question::{confirm, Question};
use crate::scrape_run::{self, Run};
use crate::words::{grouped, now};
use emusen_kit_core::input::UiButton;
use serde_json::{json, Value};
use std::collections::{BTreeSet, HashMap};
use std::path::Path;

const TICKS_PER_SECOND: i64 = 10_000_000;
pub(crate) const APPLY_ECLIPSE: &str = "eclipse-apply";
pub(crate) const SCRAPE_ONE: &str = "eclipse-scrape-one";
const TILE_KINDS: [(&str, &str); 3] = [("cover", "Cover"), ("screenshot", "Screenshot"), ("video", "Video")];

#[derive(Clone, Debug, Default)]
pub(crate) struct MediaTile {
    pub kind: String,
    pub label: String,
    pub present: bool,
    pub source_url: String,
    pub origin: String,
}

#[derive(Clone, Debug)]
pub(crate) struct GameRow {
    pub path: String,
    pub title: String,
    pub system: String,
    pub console: String,
    pub folder_path: String,
    pub have_count: usize,
    pub total_count: usize,
    pub missing: Vec<String>,
    pub last_run: String,
}

#[derive(Default)]
pub(crate) struct EclipseScraper {
    pub enabled: bool,
    pub initialized: bool,
    pub status_open: bool,
    pub status_focus: usize,
    pub active_filter: String,
    pub focus_zone: usize,
    pub focused_filter_idx: usize,
    pub focused_row_key: Option<String>,
    pub focused_toggle_idx: usize,
    pub marked: BTreeSet<String>,
    pub videos: bool,
    pub overwrite: bool,
    pub search: String,
    pub consoles: Vec<(String, String)>,
    pub games: Vec<GameRow>,
    pub account_signed_in: bool,
    pub account_name: String,
    pub quota_requests: u64,
    pub quota_max_per_day: Option<u64>,
    pub latest_progress: Option<Value>,
    pub pending_paths: Vec<String>,
    pub pending_one: Option<String>,
    pub focused_media: Option<(String, Vec<MediaTile>)>,
    pub started_at_ms: i64,
}

pub(crate) fn kind_label(esde_type: &str) -> &'static str {
    match esde_type {
        "cover" => "Cover",
        "screenshot" => "Screenshot",
        "marquee" => "Marquee",
        "titlescreen" => "Title Screen",
        "miximage" => "Miximage",
        "backcover" => "Back Cover",
        "3dbox" => "3D Box",
        "physicalmedia" => "Physical Media",
        "fanart" => "Fan Art",
        "manual" => "Manual",
        "video" => "Video",
        _ => "Media",
    }
}

pub(crate) fn source_words(word: &str) -> &'static str {
    match word {
        "package" => "the game's package",
        "screenscraper" => "ScreenScraper",
        "esde" => "an ES-DE media folder",
        "hand" => "placed by hand",
        "openemu" => "OpenEmu",
        "other_game" => "another game's picture",
        _ => "unknown source",
    }
}

pub(crate) fn outcome_words(word: &str, skipped: bool, empty_found: bool) -> &'static str {
    if skipped {
        return "Skipped";
    }
    match word {
        "found" if empty_found => "Nothing new",
        "found" => "Found",
        "unknown" => "Not known to ScreenScraper",
        "error" => "Failed",
        "retry" => "To retry",
        "stopped" => "Stopped",
        _ => "",
    }
}

fn format_duration(secs: i64) -> String {
    let s = secs.max(0);
    let h = s / 3600;
    let m = (s % 3600) / 60;
    let rem = s % 60;
    if h > 0 { format!("{h}:{m:02}:{rem:02}") } else { format!("{m}:{rem:02}") }
}

fn game_stem(path: &str) -> String {
    Path::new(path).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
}

fn effective_wanted(source: &ClientSource, videos: bool, overwrite: bool) -> Vec<String> {
    let mut settings = source.settings().unwrap_or(Value::Null);
    if let Some(obj) = settings.as_object_mut() {
        obj.insert("ScrapeVideos".to_owned(), json!(videos));
        obj.insert("ScrapeRefresh".to_owned(), json!(overwrite));
    }
    let kinds = source.scraper().and_then(|s| s.get(&json!({ "what": "kinds" })).ok()).unwrap_or(Value::Null);
    let mut wanted = scrape_run::wanted_kinds(&settings, &kinds);
    if wanted.is_empty() {
        wanted = ["cover", "screenshot", "marquee", "titlescreen", "miximage", "manual"].into_iter().map(str::to_owned).collect();
        if videos {
            wanted.push("video".to_owned());
        }
    }
    wanted
}

impl EclipseScraper {
    pub fn refresh(&mut self, source: &ClientSource, run: Option<&Run>) {
        if !self.initialized {
            let settings = source.settings().unwrap_or(Value::Null);
            self.videos = settings.get("ScrapeVideos").and_then(Value::as_bool).unwrap_or(false);
            self.overwrite = settings.get("ScrapeRefresh").and_then(Value::as_bool).unwrap_or(false);
            if self.active_filter.is_empty() {
                self.active_filter = super::scraper::ALL.to_owned();
            }
            self.focus_zone = 1;
            self.initialized = true;
        }
        let wanted = effective_wanted(source, self.videos, self.overwrite);
        let sys_names = eclipse_console_lookup(source);
        let games = source.games().unwrap_or_default();
        let scraper = source.scraper();
        let outcomes = scraper.and_then(|s| s.get(&json!({ "what": "outcomes" })).ok()).unwrap_or(Value::Null);
        let found = scraper.and_then(|s| s.get(&json!({ "what": "found" })).ok()).unwrap_or(Value::Null);
        let account = scraper.and_then(|s| s.get(&json!({ "what": "account" })).ok()).unwrap_or(Value::Null);
        let quota = scraper.and_then(|s| s.get(&json!({ "what": "quota" })).ok()).unwrap_or(Value::Null);

        self.account_signed_in = account.get("signed_in").and_then(Value::as_bool).unwrap_or(false);
        self.account_name = account.get("name").and_then(Value::as_str).unwrap_or_default().to_owned();
        self.quota_requests = quota.get("requests").and_then(Value::as_u64).unwrap_or(0);
        self.quota_max_per_day = quota.get("max_per_day").and_then(Value::as_u64);

        let mut run_outcomes: HashMap<String, String> = HashMap::new();
        if let Some(p) = &self.latest_progress {
            for r in p.get("results").and_then(Value::as_array).into_iter().flatten() {
                if let Some(path) = r.get("path").and_then(Value::as_str) {
                    let out = r.get("outcome").and_then(Value::as_str).unwrap_or("");
                    let skipped = r.get("skipped").and_then(Value::as_bool).unwrap_or(false);
                    let empty = r.get("kinds").and_then(Value::as_array).is_some_and(Vec::is_empty);
                    run_outcomes.insert(path.to_owned(), outcome_words(out, skipped, empty).to_owned());
                }
            }
        } else if let Some(r) = run {
            for (path, words, _) in &r.results {
                run_outcomes.insert(path.clone(), words.clone());
            }
        }

        let mut rows = Vec::new();
        let mut seen_consoles = BTreeSet::new();
        for g in games {
            let Some(sys) = g.system.clone() else { continue };
            let console = sys_names.get(&sys).cloned().unwrap_or_else(|| sys.clone());
            seen_consoles.insert(sys.clone());
            let present: Vec<String> = scraper
                .and_then(|s| {
                    s.get(&json!({
                        "what": "present",
                        "system": sys,
                        "games": [{ "path": g.path, "folder_path": g.folder, "folder": false }]
                    }))
                    .ok()
                })
                .and_then(|v| v.as_array().cloned())
                .map(|arr| arr.into_iter().filter_map(|x| x.as_str().map(str::to_owned)).collect())
                .unwrap_or_default();
            let have_count = wanted.iter().filter(|k| present.iter().any(|p| p == *k)).count();
            let missing: Vec<String> = wanted.iter().filter(|k| !present.iter().any(|p| p == *k)).map(|k| kind_label(k).to_owned()).collect();
            let db_state = outcomes.get(&g.path).and_then(|o| o.get("state")).and_then(Value::as_str).unwrap_or("");
            let last_run = run_outcomes.get(&g.path).cloned().unwrap_or_else(|| outcome_words(db_state, false, false).to_owned());
            let scraped_title = found.get(&g.path).and_then(|rec| rec.get("name")).and_then(Value::as_str).filter(|n| !n.is_empty());
            let title = scraped_title.map(str::to_owned).unwrap_or_else(|| game_stem(&g.path));
            rows.push(GameRow { path: g.path, title, system: sys, console, folder_path: g.folder, have_count, total_count: wanted.len(), missing, last_run });
        }
        let mut consoles: Vec<(String, String)> = seen_consoles
            .into_iter()
            .map(|id| {
                let label = sys_names.get(&id).cloned().unwrap_or_else(|| id.clone());
                (id, label)
            })
            .collect();
        consoles.sort_by(|a, b| a.1.to_lowercase().cmp(&b.1.to_lowercase()));
        rows.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()).then_with(|| a.path.cmp(&b.path)));
        self.consoles = consoles;
        self.games = rows;
        self.focused_media = None;
        self.ensure_focus(source);
    }

    fn filter_keys(&self) -> Vec<String> {
        std::iter::once(super::scraper::ALL.to_owned()).chain(self.consoles.iter().map(|c| c.0.clone())).collect()
    }

    pub fn visible_games(&self) -> Vec<&GameRow> {
        let q = self.search.trim().to_lowercase();
        self.games
            .iter()
            .filter(|g| (self.active_filter == super::scraper::ALL || g.system == self.active_filter) && (q.is_empty() || g.title.to_lowercase().contains(&q) || g.console.to_lowercase().contains(&q)))
            .collect()
    }

    pub fn focused_game(&self) -> Option<&GameRow> {
        let vis = self.visible_games();
        if vis.is_empty() {
            return None;
        }
        self.focused_row_key.as_ref().and_then(|k| vis.iter().copied().find(|g| &g.path == k)).or_else(|| vis.first().copied())
    }

    pub fn ensure_focus(&mut self, source: &ClientSource) {
        let vis = self.visible_games();
        let next = self.focused_row_key.as_ref().and_then(|k| vis.iter().find(|g| &g.path == k).map(|g| g.path.clone())).or_else(|| vis.first().map(|g| g.path.clone()));
        self.focused_row_key = next;
        self.sync_focused_media(source);
    }

    pub fn sync_focused_media(&mut self, source: &ClientSource) {
        let Some(g) = self.focused_game().cloned() else {
            self.focused_media = None;
            return;
        };
        if self.focused_media.as_ref().is_some_and(|(p, _)| p == &g.path) {
            return;
        }
        let scraper = source.scraper();
        let mut tiles = Vec::with_capacity(3);
        for (kind, label) in TILE_KINDS {
            let info = scraper.and_then(|s| {
                s.get(&json!({ "what": "media", "path": g.path, "system": g.system, "folder_path": g.folder_path, "folder": false, "type": kind, "choices": true })).ok()
            });
            let (present, source_url, origin) = match info.filter(|v| !v.is_null()) {
                Some(v) => {
                    let path = v.get("path").and_then(Value::as_str).unwrap_or_default();
                    let src = v.get("source").and_then(Value::as_str).unwrap_or_default();
                    let url = if kind != "video" && !path.is_empty() { format!("file://{path}") } else { String::new() };
                    (true, url, source_words(src).to_owned())
                }
                None => (false, String::new(), String::new()),
            };
            tiles.push(MediaTile { kind: kind.to_owned(), label: label.to_owned(), present, source_url, origin });
        }
        self.focused_media = Some((g.path, tiles));
    }

    pub fn toggle_mark(&mut self, path: &str) {
        if !self.marked.remove(path) {
            self.marked.insert(path.to_owned());
        }
    }

    pub fn choose_filter(&mut self, source: &ClientSource, key: &str) {
        let keys = self.filter_keys();
        if let Some(at) = keys.iter().position(|k| k == key) {
            self.active_filter = key.to_owned();
            self.focused_filter_idx = at;
            self.focused_row_key = None;
            self.ensure_focus(source);
        }
    }

    pub fn step_filter(&mut self, source: &ClientSource, delta: i32) {
        let keys = self.filter_keys();
        let next = eclipse_step_filter(keys.len(), self.focused_filter_idx, delta);
        if let Some(key) = keys.get(next).cloned() {
            self.choose_filter(source, &key);
        }
    }

    pub fn navigate(&mut self, source: &ClientSource, running: bool, dir: &str) -> Effect {
        if self.status_open {
            match dir {
                "left" => self.status_focus = eclipse_step_filter(3, self.status_focus, -1),
                "right" => self.status_focus = eclipse_step_filter(3, self.status_focus, 1),
                "accept" => return self.activate_status(running),
                "back" => self.status_open = false,
                _ => {}
            }
            return Effect::Done;
        }
        let filters_len = self.filter_keys().len();
        let vis_keys: Vec<String> = self.visible_games().into_iter().map(|g| g.path.clone()).collect();
        match dir {
            "up" => {
                if self.focus_zone == 2 {
                    self.focus_zone = if vis_keys.is_empty() { 0 } else { 1 };
                } else if self.focus_zone == 1 && eclipse_step_row_up(&vis_keys, &mut self.focused_row_key, &mut self.focus_zone) {
                    self.sync_focused_media(source);
                }
            }
            "down" => {
                if self.focus_zone == 0 {
                    self.focus_zone = if vis_keys.is_empty() { 2 } else { 1 };
                } else if self.focus_zone == 1 {
                    let idx = self.focused_row_key.as_ref().and_then(|k| vis_keys.iter().position(|x| x == k)).unwrap_or(0);
                    if idx + 1 >= vis_keys.len() {
                        self.focus_zone = 2;
                    } else {
                        self.focused_row_key = vis_keys.get(eclipse_move_row(vis_keys.len(), idx, 1)).cloned();
                        self.sync_focused_media(source);
                    }
                }
            }
            "left" => {
                if self.focus_zone == 0 {
                    self.focused_filter_idx = eclipse_step_filter(filters_len, self.focused_filter_idx, -1);
                } else if self.focus_zone == 2 {
                    self.focused_toggle_idx = eclipse_step_filter(2, self.focused_toggle_idx, -1);
                }
            }
            "right" => {
                if self.focus_zone == 0 {
                    self.focused_filter_idx = eclipse_step_filter(filters_len, self.focused_filter_idx, 1);
                } else if self.focus_zone == 2 {
                    self.focused_toggle_idx = eclipse_step_filter(2, self.focused_toggle_idx, 1);
                }
            }
            "accept" => match self.focus_zone {
                0 => {
                    let keys = self.filter_keys();
                    if let Some(k) = keys.get(self.focused_filter_idx).cloned() {
                        self.choose_filter(source, &k);
                        self.focus_zone = 1;
                    }
                }
                1 => {
                    if let Some(k) = self.focused_row_key.clone() {
                        self.toggle_mark(&k);
                    }
                }
                _ => {
                    if self.focused_toggle_idx == 0 {
                        self.videos = !self.videos;
                    } else {
                        self.overwrite = !self.overwrite;
                    }
                    self.refresh(source, None);
                }
            },
            "prevFilter" => self.step_filter(source, -1),
            "nextFilter" => self.step_filter(source, 1),
            _ => {}
        }
        Effect::Done
    }

    pub fn activate_status(&mut self, running: bool) -> Effect {
        match self.status_focus {
            0 if running => Effect::Done,
            1 if running => Effect::Ask(scrape_run::stop_question(), super::scraper::STOP.to_owned()),
            2 => {
                self.status_open = false;
                Effect::Done
            }
            _ => Effect::Done,
        }
    }

    pub fn row_page(&self, key: &str, source: &ClientSource) -> Vec<CardRow> {
        let Some(g) = self.games.iter().find(|r| r.path == key) else { return Vec::new() };
        let mark_label = if self.marked.contains(&g.path) { "Unmark for scraping" } else { "Mark to scrape" };
        let missing_words = if g.missing.is_empty() { "None".to_owned() } else { g.missing.join(", ") };
        let mut rows = vec![
            CardRow::new("toggle-mark", mark_label).pane(CardPane::said(mark_label, "Marked games run together when you choose Apply.")),
            CardRow::new(SCRAPE_ONE, "Scrape this game…").pane(CardPane::said("Scrape this game now.", "Nothing is sent to ScreenScraper until you confirm.")),
            CardRow::new("detail.console", "Console").value(&g.console).pane(CardPane::said(&g.console, "")),
            CardRow::new("detail.media", "Media").value(&format!("{} of {}", g.have_count, g.total_count)).pane(CardPane::said(&format!("{} of {}", g.have_count, g.total_count), "")),
            CardRow::new("detail.missing", "Missing").value(&missing_words).pane(CardPane::said(&missing_words, "")),
        ];
        if !g.last_run.is_empty() {
            rows.push(CardRow::new("detail.last", "Last run").value(&g.last_run).pane(CardPane::said(&g.last_run, "")));
        }
        if let Some((path, tiles)) = &self.focused_media {
            if path == &g.path {
                for t in tiles {
                    let val = if t.present { t.origin.clone() } else { "missing".to_owned() };
                    rows.push(CardRow::new(&format!("detail.tile.{}", t.kind), &t.label).value(&val).pane(CardPane::said(&val, "")));
                }
            }
        }
        let _ = source;
        rows
    }

    pub fn list_page(&self, running: bool) -> Vec<CardRow> {
        let mut rows = vec![
            CardRow::new(super::SEARCH, "Search"),
            CardRow::new("mark-missing", "Mark games without art"),
            CardRow::new("mark-all", "Mark all shown"),
            CardRow::new("clear-marks", "Clear marks"),
            CardRow::new("toggle-videos", "Videos").value(if self.videos { "On" } else { "Off" }).steps(),
            CardRow::new("toggle-overwrite", "Overwrite").value(if self.overwrite { "On" } else { "Off" }).steps(),
        ];
        if !self.search.is_empty() {
            rows.insert(1, CardRow::new("clear-search", "Clear search").value(&self.search));
        }
        if running || self.latest_progress.is_some() {
            rows.push(CardRow::new("open-status", "View scrape status"));
        }
        rows
    }

    pub fn apply_question(&mut self, running: bool) -> Effect {
        if running {
            self.status_open = true;
            return Effect::Done;
        }
        let paths: Vec<String> = if !self.marked.is_empty() {
            self.marked.iter().cloned().collect()
        } else {
            self.visible_games().into_iter().filter(|g| !g.missing.is_empty()).map(|g| g.path.clone()).collect()
        };
        if paths.is_empty() {
            return Effect::Ask(Question { title: "Nothing to scrape".into(), sentences: vec!["Every game shown already has its media.".into()], caption: String::new(), choices: vec!["Close".into()], back: Some(0), picture: String::new() }, "noop".to_owned());
        }
        let count_words = crate::words::games(paths.len());
        let mut notes = vec![
            if self.videos { "Videos: On.".to_owned() } else { "Videos: Off.".to_owned() },
            if self.overwrite { "Overwrite: existing media is replaced.".to_owned() } else { "Overwrite: Off — only missing media is fetched.".to_owned() },
            "Nothing is sent to ScreenScraper until Apply.".to_owned(),
        ];
        if self.marked.is_empty() {
            notes.insert(0, "No games marked; applying to games without art in this view.".to_owned());
        }
        let mut q = confirm(&format!("Scrape {count_words}?"), &notes.iter().map(String::as_str).collect::<Vec<_>>());
        q.choices[1] = "Apply".to_owned();
        self.pending_paths = paths;
        Effect::Ask(q, APPLY_ECLIPSE.to_owned())
    }

    pub fn ask_scrape_one(&mut self, key: &str) -> Effect {
        let title = self.games.iter().find(|g| g.path == key).map(|g| g.title.clone()).unwrap_or_else(|| game_stem(key));
        self.pending_one = Some(key.to_owned());
        let mut q = confirm(&format!("Scrape {title}?"), &["Nothing is sent to ScreenScraper until Apply."]);
        q.choices[1] = "Apply".to_owned();
        Effect::Ask(q, SCRAPE_ONE.to_owned())
    }

    pub fn projection(&self, running: bool, run: Option<&Run>, problem: Option<&str>) -> Value {
        if self.status_open {
            return self.status_projection(running, run, problem);
        }
        let vis = self.visible_games();
        let focused_idx = self.focused_row_key.as_ref().and_then(|k| vis.iter().position(|g| &g.path == k)).unwrap_or(0);
        let without_art = self.games.iter().filter(|g| !g.missing.is_empty()).count();
        let subtitle = if without_art > 0 {
            format!("{} · {} without art", crate::words::games(self.games.len()), grouped(without_art))
        } else {
            format!("{} · Every game has art", crate::words::games(self.games.len()))
        };
        let mut filters = vec![eclipse_filter(super::scraper::ALL, "All", self.active_filter == super::scraper::ALL)];
        for (id, label) in &self.consoles {
            filters.push(eclipse_filter(id, label, self.active_filter == *id));
        }
        let (columns, col_widths) = eclipse_columns(&[("", 36.0, 0.0), ("Game", 0.0, 1.4), ("Console", 0.0, 1.1), ("Media", 0.0, 0.6), ("Missing", 0.0, 1.3), ("Last Run", 0.0, 1.3)]);
        let rows: Vec<Value> = vis
            .iter()
            .enumerate()
            .map(|(i, g)| {
                let mark = if self.marked.contains(&g.path) { "☑" } else { "☐" }.to_owned();
                let media = format!("{} of {}", g.have_count, g.total_count);
                let missing = if g.missing.is_empty() {
                    String::new()
                } else if g.have_count == 0 && g.total_count > 0 {
                    "Everything".to_owned()
                } else {
                    g.missing.join(", ")
                };
                let is_focused = self.focus_zone == 1 && i == focused_idx;
                eclipse_row(i, &g.path, is_focused, vec![mark, g.title.clone(), g.console.clone(), media, missing, g.last_run.clone()])
            })
            .collect();

        let aside = if let Some(g) = vis.get(focused_idx).copied() {
            let tiles_vec = self.focused_media.as_ref().filter(|(p, _)| p == &g.path).map(|(_, t)| t.clone()).unwrap_or_default();
            let tiles: Vec<Value> = tiles_vec
                .iter()
                .enumerate()
                .map(|(idx, t)| {
                    json!({
                        "label": if t.present { t.label.clone() } else { format!("{}: missing", t.label) },
                        "present": t.present,
                        "source": t.source_url,
                        "objectName": format!("mediaTile.{idx}"),
                        "boxObjectName": format!("mediaBox.{idx}"),
                        "labelObjectName": format!("mediaLabel.{idx}"),
                    })
                })
                .collect();
            let mut lines = Vec::new();
            for t in &tiles_vec {
                if t.present {
                    let i = lines.len();
                    lines.push(eclipse_aside_line(i, &format!("{}: {}", t.label, t.origin), false));
                }
            }
            let last_line = if g.last_run.is_empty() { "Last run: Not scraped yet".to_owned() } else { format!("Last run: {}", g.last_run) };
            let i = lines.len();
            lines.push(eclipse_aside_line(i, &last_line, true));
            json!({
                "title": g.title,
                "subtitle": format!("Console: {}", g.console),
                "subtitleMuted": false,
                "tiles": tiles,
                "lines": lines,
            })
        } else {
            json!({ "title": "No games", "subtitle": "", "subtitleMuted": true, "tiles": [], "lines": [] })
        };

        let mark_prefix = match self.marked.len() {
            0 => "Nothing marked".to_owned(),
            n => format!("{} marked to scrape", crate::words::games(n)),
        };
        let mark_bar_text = format!("{mark_prefix} · Nothing is sent to ScreenScraper until Apply.");
        let toggles = vec![
            json!({ "key": "videos", "label": format!("Videos · {}", if self.videos { "On" } else { "Off" }), "on": self.videos, "focused": self.focus_zone == 2 && self.focused_toggle_idx == 0, "objectName": "markToggle.0", "labelObjectName": "markToggleWords.0" }),
            json!({ "key": "overwrite", "label": format!("Overwrite · {}", if self.overwrite { "On" } else { "Off" }), "on": self.overwrite, "focused": self.focus_zone == 2 && self.focused_toggle_idx == 1, "objectName": "markToggle.1", "labelObjectName": "markToggleWords.1" }),
        ];

        let help = eclipse_table_help("Mark", "Console");
        let counter = if vis.is_empty() { "0 of 0".to_owned() } else { format!("{} of {}", grouped(focused_idx + 1), grouped(vis.len())) };
        json!({
            "mode": "scraper",
            "statusOpen": false,
            "percentSuffix": "%",
            "trail": eclipse_trail(&["Tools", "Scraper"]),
            "title": "Scraper",
            "subtitle": subtitle,
            "filters": filters,
            "columns": columns,
            "colWidths": col_widths,
            "rows": rows,
            "emptyMessage": "No games match this filter.",
            "focusedIndex": if vis.is_empty() { -1 } else { focused_idx as i32 },
            "focusedZone": self.focus_zone,
            "focusedFilterIndex": self.focused_filter_idx,
            "focusedToggleIndex": self.focused_toggle_idx,
            "focusedAsideKey": self.focused_row_key.clone().unwrap_or_default(),
            "aside": aside,
            "markBarText": mark_bar_text,
            "toggles": toggles,
            "footerHelp": help,
            "actionLines": "more Screensaver\nlist Options\napply Menu\nprevFilter PageUp\nnextFilter PageDown\n",
            "counter": counter,
            "asideObjectName": "scraperAside",
            "asideTitleObjectName": "asideHeading",
            "asideSubObjectName": "asideConsole",
            "markBarObjectName": "scraperMarkBar",
            "markBarTextObjectName": "scraperMarkSummary",
            "footerObjectName": "scraperFooter",
            "helpObjectName": "scraperHelp",
            "counterObjectName": "scraperCounter",
        })
    }

    fn status_projection(&self, running: bool, run: Option<&Run>, problem: Option<&str>) -> Value {
        let p = self.latest_progress.as_ref();
        let state = p.and_then(|v| v.get("state")).and_then(Value::as_str).or_else(|| run.map(|r| r.state.as_str())).unwrap_or("idle");
        let paused = p.and_then(|v| v.get("paused")).and_then(Value::as_bool).or_else(|| run.map(|r| r.paused)).unwrap_or(false);
        let total = p.and_then(|v| v.get("total")).and_then(Value::as_u64).or_else(|| run.map(|r| r.total)).unwrap_or(0);
        let done = p.and_then(|v| v.get("done")).and_then(Value::as_u64).or_else(|| run.map(|r| r.done)).unwrap_or(0);
        let found = p.and_then(|v| v.get("found")).and_then(Value::as_u64).unwrap_or(0);
        let unknown = p.and_then(|v| v.get("unknown")).and_then(Value::as_u64).unwrap_or(0);
        let failed = p.and_then(|v| v.get("failed")).and_then(Value::as_u64).or_else(|| run.map(|r| r.failed)).unwrap_or(0);
        let skipped = p.and_then(|v| v.get("skipped")).and_then(Value::as_u64).unwrap_or(0);
        let openemu = p.and_then(|v| v.get("failover")).and_then(|f| f.get("found")).and_then(Value::as_u64).unwrap_or(0);
        let working_ticks = p.and_then(|v| v.get("working_ticks")).and_then(Value::as_i64).unwrap_or(0);
        let elapsed_secs = if working_ticks > 0 { working_ticks / TICKS_PER_SECOND } else if self.started_at_ms > 0 { (now() - self.started_at_ms).max(0) } else { 0 };
        let why = problem.map(str::to_owned).or_else(|| p.and_then(|v| v.get("why")).and_then(Value::as_str).filter(|s| !s.is_empty()).map(str::to_owned)).or_else(|| run.and_then(|r| r.why.clone()));

        let games_label = if total == 1 { "game" } else { "games" };
        let heading = match (running, paused, state) {
            (true, true, _) => format!("Paused · {} of {} {games_label}", grouped(done as usize), grouped(total as usize)),
            (true, false, _) => format!("Scraping {} of {} {games_label}", grouped(done as usize), grouped(total as usize)),
            (false, _, "cancelled" | "stopped") => format!("Stopped · {} of {} {games_label}", grouped(done as usize), grouped(total as usize)),
            (false, _, _) if why.is_some() => format!("Failed · {} of {} {games_label}", grouped(done as usize), grouped(total as usize)),
            _ => format!("Scraped {} of {} {games_label}", grouped(done as usize), grouped(total as usize)),
        };
        let fraction = if total == 0 { 0.0 } else { (done as f64 / total as f64).clamp(0.0, 1.0) };

        let cur = p.and_then(|v| v.get("current")).filter(|c| !c.is_null());
        let sys_name = |id: &str| self.consoles.iter().find(|c| c.0 == id).map(|c| c.1.clone()).unwrap_or_else(|| id.to_owned());
        let (cur_title, cur_stage) = if running {
            if paused {
                let title = cur.and_then(|c| c.get("path").and_then(Value::as_str)).map(|path| {
                    let sys = cur.and_then(|c| c.get("system").and_then(Value::as_str)).unwrap_or("");
                    format!("{} · {}", game_stem(path), sys_name(sys))
                }).unwrap_or_else(|| "Scrape paused".to_owned());
                (title, "Paused by player".to_owned())
            } else if let Some(c) = cur {
                let path = c.get("path").and_then(Value::as_str).unwrap_or("");
                let sys = c.get("system").and_then(Value::as_str).unwrap_or("");
                let step = c.get("step").and_then(Value::as_str).unwrap_or("");
                let kind = c.get("kind").and_then(Value::as_str).unwrap_or("media");
                let stage = match step {
                    "downloading" => format!("Downloading its {kind}"),
                    "arrived" => format!("Saved its {kind}"),
                    _ => "Looking up on ScreenScraper".to_owned(),
                };
                (format!("{} · {}", game_stem(path), sys_name(sys)), stage)
            } else {
                ("Preparing scrape".to_owned(), "Opening ScreenScraper session".to_owned())
            }
        } else if let Some(w) = &why {
            ("Scrape stopped".to_owned(), w.clone())
        } else if state == "cancelled" || state == "stopped" {
            ("Scrape stopped".to_owned(), "Stopped; what was fetched is kept.".to_owned())
        } else {
            ("Scrape finished".to_owned(), "All selected games processed.".to_owned())
        };

        let pace_line = if running && done > 0 && total > done && elapsed_secs > 0 {
            let left = (elapsed_secs * (total - done) as i64) / done as i64;
            format!("Elapsed {} · about {} left at this run's pace", format_duration(elapsed_secs), format_duration(left))
        } else {
            format!("Elapsed {}", format_duration(elapsed_secs))
        };
        let counts_line = format!("Found {found} · Not found {unknown} · Failed {failed} · Skipped {skipped} · Filled by OpenEmu {openemu}");
        let account_line = if self.account_signed_in {
            format!("Signed in as {}: member credentials active.", self.account_name)
        } else {
            "No member account: EmuSen's developer credentials alone, with their limits.".to_owned()
        };
        let quota_req = p.and_then(|v| v.get("quota")).and_then(|q| q.get("requests")).and_then(Value::as_u64).unwrap_or(self.quota_requests);
        let quota_max = p.and_then(|v| v.get("quota")).and_then(|q| q.get("max_per_day")).and_then(Value::as_u64).or(self.quota_max_per_day);
        let quota_line = match quota_max {
            Some(max) => format!("{quota_req} of {max} requests used today · {} left", max.saturating_sub(quota_req)),
            None => format!("{quota_req} requests used today"),
        };

        let recent: Vec<String> = p
            .and_then(|v| v.get("results"))
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .take(3)
            .map(|r| {
                let title = game_stem(r.get("path").and_then(Value::as_str).unwrap_or(""));
                let console = sys_name(r.get("system").and_then(Value::as_str).unwrap_or(""));
                let out = r.get("outcome").and_then(Value::as_str).unwrap_or("");
                let skipped = r.get("skipped").and_then(Value::as_bool).unwrap_or(false);
                let kinds: Vec<&str> = r.get("kinds").and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str).collect();
                let detail = r.get("detail").and_then(Value::as_str).unwrap_or("");
                let desc = if skipped {
                    "skipped".to_owned()
                } else {
                    match out {
                        "found" if !kinds.is_empty() => format!("found · {}", kinds.join(", ")),
                        "found" => "nothing new".to_owned(),
                        "unknown" => "not found".to_owned(),
                        "retry" => "to retry".to_owned(),
                        "stopped" => "stopped".to_owned(),
                        _ if detail.is_empty() => "failed".to_owned(),
                        _ => format!("failed: {detail}"),
                    }
                };
                format!("{title} ({console}): {desc}")
            })
            .collect();

        let actions = vec![
            json!({ "key": "pause", "label": if paused { "Resume" } else { "Pause" }, "focused": self.status_focus == 0, "enabled": running, "objectName": "statusAction.0", "glyphObjectName": "statusActionGlyph.0", "labelObjectName": "statusActionWords.0" }),
            json!({ "key": "cancel", "label": "Cancel...", "focused": self.status_focus == 1, "enabled": running, "objectName": "statusAction.1", "glyphObjectName": "statusActionGlyph.1", "labelObjectName": "statusActionWords.1" }),
            json!({ "key": "hide", "label": "Hide", "focused": self.status_focus == 2, "enabled": true, "objectName": "statusAction.2", "glyphObjectName": "statusActionGlyph.2", "labelObjectName": "statusActionWords.2" }),
        ];

        let mut help = Vec::new();
        eclipse_help_entry(&mut help, UiButton::Left, Some(UiButton::Right), "Move");
        eclipse_help_entry(&mut help, UiButton::Accept, None, "Choose");
        eclipse_help_entry(&mut help, UiButton::Back, None, "Back");

        json!({
            "mode": "scrapeStatus",
            "statusOpen": true,
            "percentSuffix": "%",
            "trail": eclipse_trail(&["Tools", "Scraper", "Status"]),
            "footerObjectName": "statusFooter",
            "helpObjectName": "statusHelp",
            "counterObjectName": "statusCounter",
            "title": heading,
            "done": grouped(done as usize),
            "ringSubtitle": format!("of {} {games_label}", grouped(total as usize)),
            "fraction": fraction,
            "currentTitle": cur_title,
            "currentStage": cur_stage,
            "paceLine": pace_line,
            "countsLine": counts_line,
            "accountLine": account_line,
            "quotaLine": quota_line,
            "recentLines": recent,
            "actions": actions,
            "focusedActionIndex": self.status_focus,
            "footerHelp": help,
            "actionLines": "",
            "counter": format!("{} of 3", self.status_focus + 1),
        })
    }
}
