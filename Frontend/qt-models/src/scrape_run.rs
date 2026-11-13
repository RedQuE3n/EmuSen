//! A scrape run as the models hold it (EmuSen_Qt.md §7.8.2, §7.7.F): the start request the Scraping settings make for a
//! set of games, the run followed through the scrape service's progress until it ends, and what its progress says. The
//! game page's Scrape This Game and the Scraper tool both run through it. No Qt here.

use crate::clients::ClientSource;
use crate::library::LibrarySource;
use crate::question::{confirm, Question};
use emusen_platform_client::library::GameFile;
use serde_json::{json, Value};

/// Whether the switch `key` of the player's settings is on; a key the settings lack is off.
fn switch(settings: &Value, key: &str) -> bool {
    settings.as_object().and_then(|all| all.iter().find(|(k, _)| k.eq_ignore_ascii_case(key))).and_then(|(_, v)| v.as_bool()).unwrap_or(false)
}

/// The kinds the Scraping settings ask for, by the media store's type: each kind of the scraper's table (`get kinds`)
/// whose switch `Scrape<folder>` is on, as the settings name every kind by its folder.
pub fn wanted_kinds(settings: &Value, kinds: &Value) -> Vec<String> {
    let of = |k: &Value| Some((k.get("type")?.as_str()?.to_owned(), k.get("folder")?.as_str()?.to_owned()));
    kinds.as_array().into_iter().flatten().filter_map(of).filter(|(_, folder)| switch(settings, &format!("Scrape{folder}"))).map(|(ty, _)| ty).collect()
}

/// The start request for `games` (emusen_scrape_service.h `start`): one game chosen alone, or a run over many that asks
/// only for what each lacks; the kinds, region, language, fallback, workers and refresh the Scraping settings ask for,
/// and ScreenScraper used as its Scraping switch says.
pub fn request(games: &[GameFile], one: bool, settings: &Value, kinds: &Value) -> Value {
    let text = |key: &str, or: &str| settings.get(key).and_then(Value::as_str).filter(|v| !v.is_empty()).unwrap_or(or).to_owned();
    // 0 workers is as many as the account allows, which the scraper holds to its maxthreads.
    let threads = settings.get("ScrapeThreads").and_then(Value::as_i64).filter(|n| *n > 0).unwrap_or(i64::from(i32::MAX));
    let listed: Vec<Value> = games.iter().filter_map(|g| Some(json!({ "path": g.path, "system": g.system.as_deref()? }))).collect();
    json!({
        "games": listed,
        "one": one,
        "refresh": switch(settings, "ScrapeRefresh"),
        "enabled": settings.get("Scraping").and_then(Value::as_bool).unwrap_or(true),
        "choices": {
            "kinds": wanted_kinds(settings, kinds),
            "region": text("ScrapeRegion", "auto"),
            "language": text("ScrapeLanguage", "en"),
            "region_fallback": settings.get("ScrapeRegionFallback").and_then(Value::as_bool).unwrap_or(true),
            "threads": threads,
        },
    })
}

/// The run started over `games` on the scraper of `source`, its store opened, `test` its test seam where one is given;
/// how many games it queued.
pub fn start(source: &ClientSource, games: &[GameFile], one: bool, test: Option<&Value>) -> Result<i64, String> {
    let settings = source.settings()?;
    let kinds = source.scraper().ok_or("the scraper could not be opened")?.get(&json!({ "what": "kinds" })).map_err(|e| e.to_string())?;
    source.scrape_start(&request(games, one, &settings, &kinds), test)
}

/// The run followed until it ends, each progress told, then the scraper put back to finding pictures alone.
pub fn follow(source: &ClientSource, mut tell: impl FnMut(Value)) -> Result<(), String> {
    let mut serial = 0;
    let followed = loop {
        serial = source.scrape_wait(serial, 250).unwrap_or(serial);
        let progress = match source.scrape_progress() {
            Ok(p) => p,
            Err(why) => break Err(why),
        };
        let going = progress["state"] == "running";
        tell(progress);
        if !going {
            break Ok(());
        }
    };
    let ended = source.scrape_end();
    followed.and(ended)
}

/// Stopping a run: what was fetched is kept.
pub fn stop_question() -> Question {
    let mut q = confirm("Stop the scrape?", &["What was fetched before it stops is kept."]);
    q.choices[1] = "Stop".to_owned();
    q
}

/// The media store's own type names, which the scraper's kinds table holds no words for.
fn kind(k: &Value) -> String {
    k.as_str().unwrap_or("").to_owned()
}

/// The step of the game in hand, as its progress says it.
pub fn step_words(current: &Value) -> String {
    match current["step"].as_str() {
        Some("downloading") => format!("Fetching the {}", kind(&current["kind"])),
        Some("arrived") => format!("Fetched the {}", kind(&current["kind"])),
        Some(_) => "Looking it up".into(),
        None => "Starting".into(),
    }
}

/// How a game came out, by its result in the progress.
pub fn result_words(result: &Value) -> String {
    match (result["outcome"].as_str().unwrap_or(""), result["kinds"].as_array()) {
        ("found", Some(kinds)) if !kinds.is_empty() => format!("Found: {}", kinds.iter().map(kind).collect::<Vec<_>>().join(", ")),
        ("found", _) => "Found, nothing new".into(),
        ("unknown", _) => "Not found".into(),
        ("stopped", _) => "Stopped".into(),
        _ => result["detail"].as_str().filter(|d| !d.is_empty()).map_or_else(|| "It failed".into(), |d| format!("It failed: {d}")),
    }
}

/// Whether a result is a failure: neither found, not found nor stopped.
pub fn failed(result: &Value) -> bool {
    !matches!(result["outcome"].as_str(), Some("found" | "unknown" | "stopped"))
}

/// What a scrape's progress (emusen.scrape `progress`, EmuSen_Backend.md §30.3) says of one game's run: whether it is
/// still going, and its words: the step on the game while it goes, how it came out once it has ended.
pub fn run_words(progress: &Value, game: &str) -> (bool, String) {
    match progress["state"].as_str().unwrap_or("idle") {
        "running" => (true, step_words(&progress["current"])),
        "cancelled" | "stopped" => (false, "Stopped".into()),
        _ => {
            let result = progress["results"].as_array().and_then(|all| all.iter().rev().find(|r| r["path"] == game));
            let words = match result {
                Some(r) => result_words(r),
                None => progress["why"].as_str().map_or_else(|| "Done".into(), |why| format!("It could not run: {why}")),
            };
            (false, words)
        }
    }
}

/// A run over many games as its progress says it: its state, whether it is paused, how many of how many have ended and
/// how many failed, the game in hand and its step, why it could not run, and each game's result, newest first.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Run {
    pub state: String,
    pub paused: bool,
    pub total: u64,
    pub done: u64,
    pub failed: u64,
    pub current: Option<(String, String)>,
    pub why: Option<String>,
    pub last_failure: Option<String>,
    pub results: Vec<(String, String, bool)>,
}

impl Run {
    pub fn of(progress: &Value) -> Run {
        let count = |k: &str| progress[k].as_u64().unwrap_or(0);
        let text = |v: &Value| v.as_str().filter(|w| !w.is_empty()).map(str::to_owned);
        let current = &progress["current"];
        Run {
            state: progress["state"].as_str().unwrap_or("idle").to_owned(),
            paused: progress["paused"].as_bool().unwrap_or(false),
            total: count("total"),
            done: count("done"),
            failed: count("failed"),
            current: current["path"].as_str().map(|p| (p.to_owned(), step_words(current))),
            why: text(&progress["why"]),
            last_failure: text(&progress["last_failure"]),
            results: progress["results"].as_array().into_iter().flatten().map(|r| (r["path"].as_str().unwrap_or("").to_owned(), result_words(r), failed(r))).collect(),
        }
    }

    pub fn going(&self) -> bool {
        self.state == "running"
    }

    /// The share done, while the run's count is known.
    pub fn share(&self) -> f64 {
        if self.total == 0 { 0.0 } else { (self.done as f64 / self.total as f64).min(1.0) }
    }
}
