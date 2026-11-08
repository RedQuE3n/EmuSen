//! `emusen.scrape`: the `scrape` command over the `emusen.scrape` and `emusen.games` services - see EmuSen_LibraryTools.md §8 and `man scrape`.

use dianash_module::{Answer, Call, Manifest, Module};
use serde_json::Value;
use std::collections::{BTreeSet, HashMap, HashSet};

pub const SERVICE: &str = "emusen.scrape";
pub const MAJOR: u32 = 2;
pub const GAMES: &str = "emusen.games";
pub const GAMES_MAJOR: u32 = 1;

/// `emusen_scrape_service.h`'s table.
#[repr(C)]
pub struct ScrapeServiceV2 {
    pub size: u32,
    pub get: unsafe extern "C" fn(*const u8, usize, *mut u8, usize) -> i64,
    pub change: unsafe extern "C" fn(*const u8, usize, *mut u8, usize) -> i64,
    pub start: unsafe extern "C" fn(*const u8, usize) -> i64,
    pub progress: unsafe extern "C" fn(*const u8, usize, *mut u8, usize) -> i64,
    pub wait: extern "C" fn(u64, u32) -> i64,
    pub pause: extern "C" fn(i32) -> i32,
    pub cancel: extern "C" fn() -> i32,
    pub theme_kinds: unsafe extern "C" fn(*const u8, usize, *mut u8, usize) -> i64,
    pub last_error: unsafe extern "C" fn(*mut u8, usize) -> i64,
}

/// `emusen_games_service.h`'s table.
#[repr(C)]
pub struct GamesServiceV1 {
    pub size: u32,
    pub get: unsafe extern "C" fn(*const u8, usize, *mut u8, usize) -> i64,
    pub games: unsafe extern "C" fn(*const u8, usize, *mut u8, usize) -> i64,
    pub last_error: unsafe extern "C" fn(*mut u8, usize) -> i64,
}

/// The scraper's status for a run already going.
const BUSY: i64 = -1164;

/// A `since` past every result, for a progress read that wants none of them.
const ALL_SEEN: u64 = i64::MAX as u64;

pub const COMMANDS: &[(&str, bool, &str)] = &[(
    "scrape",
    false,
    "  scrape ls [<choice>...] [--kinds <k,...>|theme] [--videos]\n                                the chosen games, one a line: path, system, and where each kind comes from\n  scrape plan [<choice>...] [<option>...]\n                                what a run would ask: games, requests, the day's quota left; nothing is sent\n  scrape run [<choice>...] [<option>...] [--resume]\n                                the plan on standard error, then the run: one line a game as it finishes\n  scrape status                 the quota, the account and the run going, one key a line\n  scrape wait [scrape<n>]       follow a run that went on in the background (ps lists it, kill stops it)\n  choices: --system <s>, --favorites, --collection <name>, --missing <kind>|any, <path>..., or paths on stdin\n  options: --kinds, --videos, --overwrite, --region <r>, --language <l>, --threads <n>",
)];

const USAGE: &str = "Usage: scrape ls|plan|run [--system <s>] [--favorites] [--collection <name>] [--missing <kind>|any] [<path>...] [--kinds <k,...>|theme] [--videos] [--overwrite] [--region <r>] [--language <l>] [--threads <n>] [--resume] | scrape status | scrape wait [scrape<n>]\n  Paths on standard input, one a line (the first tab-separated field), are chosen by hand. See man scrape.";

/// The kinds a run asks when none is named: every picture a theme can draw, so that changing themes never needs a
/// second scrape (decided 2026-11-02 by the tester); videos and manuals only when asked.
pub const DEFAULT_KINDS: [&str; 9] = ["cover", "screenshot", "marquee", "titlescreen", "miximage", "backcover", "3dbox", "physicalmedia", "fanart"];

/// `--missing metadata`: a game with no description, ES-DE's "No metadata".
pub const METADATA: &str = "metadata";

/// ES-DE's eleven kinds, the scraper's names.
pub const KINDS: [&str; 11] = ["cover", "screenshot", "marquee", "titlescreen", "miximage", "backcover", "3dbox", "physicalmedia", "fanart", "manual", "video"];

/// What the scraper is asked through: the service's calls, each request and answer JSON.
pub trait Backend {
    fn get(&self, request: &str) -> Result<String, String>;
    fn start(&self, request: &str) -> Result<i64, (i64, String)>;
    fn progress(&self, since: u64) -> Result<String, String>;
    fn wait(&self, serial: u64, timeout_ms: u32) -> u64;
    fn cancel(&self) -> bool;
    fn library_get(&self, request: &str) -> Result<String, String>;
    fn games(&self) -> Result<String, String>;
    fn theme_kinds(&self, system: &str) -> Result<String, String>;
}

/// The two services' tables as a `Backend`: the scraper's, and the library's games.
pub struct Service {
    pub scrape: *const ScrapeServiceV2,
    pub games: *const GamesServiceV1,
}

fn words(last_error: unsafe extern "C" fn(*mut u8, usize) -> i64) -> String {
    let mut buffer = vec![0u8; 1024];
    let n = unsafe { last_error(buffer.as_mut_ptr(), buffer.len()) };
    String::from_utf8_lossy(&buffer[..(n.max(0) as usize).min(buffer.len())]).into_owned()
}

fn call_table(call: unsafe extern "C" fn(*const u8, usize, *mut u8, usize) -> i64, last_error: unsafe extern "C" fn(*mut u8, usize) -> i64, request: &str) -> Result<String, String> {
    let mut buffer = vec![0u8; 4096];
    loop {
        let n = unsafe { call(request.as_ptr(), request.len(), buffer.as_mut_ptr(), buffer.len()) };
        if n < 0 {
            return Err(words(last_error));
        }
        if n as usize <= buffer.len() {
            return Ok(String::from_utf8_lossy(&buffer[..n as usize]).into_owned());
        }
        buffer.resize(n as usize, 0);
    }
}

impl Service {
    fn table(&self) -> &ScrapeServiceV2 {
        unsafe { &*self.scrape }
    }

    fn library(&self) -> &GamesServiceV1 {
        unsafe { &*self.games }
    }

    fn last_error(&self) -> String {
        words(self.table().last_error)
    }

    fn ask(&self, call: unsafe extern "C" fn(*const u8, usize, *mut u8, usize) -> i64, request: &str) -> Result<String, String> {
        call_table(call, self.table().last_error, request)
    }
}

impl Backend for Service {
    fn get(&self, request: &str) -> Result<String, String> {
        self.ask(self.table().get, request)
    }

    fn start(&self, request: &str) -> Result<i64, (i64, String)> {
        let n = unsafe { (self.table().start)(request.as_ptr(), request.len()) };
        if n < 0 { Err((n, self.last_error())) } else { Ok(n) }
    }

    fn progress(&self, since: u64) -> Result<String, String> {
        self.ask(self.table().progress, &format!("{{\"since\":{since}}}"))
    }

    fn wait(&self, serial: u64, timeout_ms: u32) -> u64 {
        (self.table().wait)(serial, timeout_ms).max(0) as u64
    }

    fn cancel(&self) -> bool {
        (self.table().cancel)() == 1
    }

    fn library_get(&self, request: &str) -> Result<String, String> {
        call_table(self.library().get, self.library().last_error, request)
    }

    fn games(&self) -> Result<String, String> {
        call_table(self.library().games, self.library().last_error, "")
    }

    fn theme_kinds(&self, system: &str) -> Result<String, String> {
        self.ask(self.table().theme_kinds, system)
    }
}

// SAFETY: the services' tables are the platform library's or the host's, callable from any thread (emusen_scrape_service.h, emusen_games_service.h).
unsafe impl Send for Service {}

fn compact(members: Vec<(&str, Value)>) -> String {
    let mut out = String::new();
    out.push_str(&serde_json::to_string(&Value::Object(members.into_iter().map(|(k, v)| (String::from(k), v)).collect())).expect("a value is JSON"));
    out
}

fn text<'a>(e: &'a Value, name: &str) -> &'a str {
    e.get(name).and_then(Value::as_str).unwrap_or("")
}

fn token(e: &Value, name: &str) -> String {
    match e.get(name) {
        Some(Value::Number(t)) => t.to_string(),
        Some(Value::String(s)) => s.clone(),
        Some(Value::Bool(b)) => b.to_string(),
        _ => String::new(),
    }
}

fn number(e: &Value, name: &str) -> f64 {
    match e.get(name) {
        Some(Value::Number(t)) => t.as_f64().unwrap_or(0.0),
        _ => 0.0,
    }
}

fn parse(answer: &str) -> Result<Value, String> {
    serde_json::from_str::<Value>(answer).map_err(|_| "the scraper's answer is not JSON".to_string())
}

/// What the command line asks for, before anything is read.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Line {
    pub sub: String,
    pub systems: Vec<String>,
    pub favorites: bool,
    pub collection: Option<String>,
    pub missing: Option<String>,
    pub paths: Vec<String>,
    pub kinds: Option<Vec<String>>,
    pub theme: bool,
    pub videos: bool,
    pub overwrite: bool,
    pub region: Option<String>,
    pub language: Option<String>,
    pub threads: Option<u32>,
    pub resume: bool,
}

/// A command line as what it asks, or the usage.
pub fn parse_line(args: &[String]) -> Result<Line, String> {
    let mut words = args.iter().skip(1);
    let sub = words.next().map(String::as_str).unwrap_or("");
    if !matches!(sub, "ls" | "plan" | "run" | "status") {
        return Err(USAGE.into());
    }
    let mut line = Line { sub: sub.to_string(), ..Line::default() };
    let value = |words: &mut dyn Iterator<Item = &String>, flag: &str| -> Result<String, String> {
        words.next().filter(|v| !v.starts_with("--")).cloned().ok_or_else(|| format!("scrape: {flag} needs a value\n{USAGE}"))
    };
    while let Some(w) = words.next() {
        if sub == "status" {
            return Err(format!("scrape status takes nothing more\n{USAGE}"));
        }
        match w.as_str() {
            "--system" => line.systems.extend(value(&mut words, w)?.split(',').filter(|s| !s.is_empty()).map(str::to_string)),
            "--favorites" | "--favourites" => line.favorites = true,
            "--collection" => line.collection = Some(value(&mut words, w)?),
            "--missing" => {
                let kind = value(&mut words, w)?;
                if let Some(bad) = kind.split(',').find(|k| !KINDS.contains(k) && (kind.contains(',') || !matches!(*k, "any" | METADATA))) {
                    return Err(format!("scrape: --missing {bad}: not a kind; the kinds are {}, a list of them, any, or {METADATA}", KINDS.join(", ")));
                }
                line.missing = Some(kind);
            }
            "--kinds" => {
                let named = value(&mut words, w)?;
                if named == "theme" {
                    line.theme = true;
                    continue;
                }
                let kinds: Vec<String> = named.split(',').filter(|k| !k.is_empty()).map(str::to_string).collect();
                if let Some(bad) = kinds.iter().find(|k| !KINDS.contains(&k.as_str())) {
                    return Err(format!("scrape: --kinds {bad}: not a kind; the kinds are {}, or theme", KINDS.join(", ")));
                }
                line.kinds = Some(kinds);
            }
            "--videos" => line.videos = true,
            "--overwrite" if sub != "ls" => line.overwrite = true,
            "--region" if sub != "ls" => line.region = Some(value(&mut words, w)?),
            "--language" if sub != "ls" => line.language = Some(value(&mut words, w)?),
            "--threads" if sub != "ls" => line.threads = Some(value(&mut words, w)?.parse().ok().filter(|&n| n > 0).ok_or_else(|| format!("scrape: --threads needs a number above 0\n{USAGE}"))?),
            "--resume" if sub == "run" => line.resume = true,
            "-" => {}
            o if o.starts_with('-') && o.len() > 1 => return Err(format!("scrape: {o} is not an option of scrape {sub}\n{USAGE}")),
            p => line.paths.push(p.to_string()),
        }
    }
    if line.resume && (line.has_choice() || line.kinds.is_some() || line.theme) {
        return Err(format!("scrape run --resume: an interrupted run goes on as it was queued, so it takes no choice or kinds\n{USAGE}"));
    }
    Ok(line)
}

impl Line {
    fn has_choice(&self) -> bool {
        !self.systems.is_empty() || self.favorites || self.collection.is_some() || self.missing.is_some() || !self.paths.is_empty()
    }
}

/// One library game as the service lists it.
#[derive(Clone, Debug, PartialEq)]
pub struct Game {
    pub path: String,
    pub system: String,
    pub package: Option<String>,
}

/// A piped list's paths: each line's first tab-separated field, blank lines passed over.
pub fn listed(stdin: &str) -> Vec<String> {
    stdin.lines().map(|l| l.split('\t').next().unwrap_or("").trim_end_matches('\r')).filter(|p| !p.trim().is_empty()).map(str::to_string).collect()
}

/// What the choice needs read from the library: favourites, a collection's members and the excluded games.
pub struct Facts {
    pub favorites: HashSet<String>,
    pub members: Option<HashSet<String>>,
    pub excluded: HashSet<String>,
}

/// The games a line chooses, by hand or by its criteria; the notes it leaves for standard error.
pub fn choose(games: &[Game], line: &Line, by_hand: &[String], resolve: &dyn Fn(&str) -> Option<String>, facts: &Facts) -> (Vec<Game>, Vec<String>) {
    let mut notes = Vec::new();
    let mut chosen: Vec<Game> = if by_hand.is_empty() {
        games.iter().filter(|g| !facts.excluded.contains(&g.path)).cloned().collect()
    } else {
        let mut out = Vec::new();
        let mut seen = HashSet::new();
        for p in by_hand {
            let found = games.iter().find(|g| g.path == *p).or_else(|| resolve(p).and_then(|real| games.iter().find(|g| g.path == real)));
            match found {
                Some(g) if seen.insert(g.path.clone()) => out.push(g.clone()),
                Some(_) => {}
                None => notes.push(format!("scrape: {p}: not a game of the library, passed over")),
            }
        }
        out
    };
    if !line.systems.is_empty() {
        chosen.retain(|g| line.systems.iter().any(|s| *s == g.system));
    }
    if line.favorites {
        chosen.retain(|g| facts.favorites.contains(&g.path));
    }
    if let Some(members) = &facts.members {
        chosen.retain(|g| members.contains(&g.path));
    }
    let left_out = games.iter().filter(|g| facts.excluded.contains(&g.path)).count();
    if by_hand.is_empty() && left_out > 0 {
        notes.push(format!("scrape: {left_out} game(s) marked \"exclude from multi-scraper\" left out; name one to include it"));
    }
    (chosen, notes)
}

/// Every game the service lists, with its ES-DE system; one with none is not a game the scraper can ask about.
pub fn games_of(answer: &str) -> Result<(Vec<Game>, Vec<String>), String> {
    let v = parse(answer)?;
    let roots = match v.get("roots") {
        Some(Value::Array(r)) => r.iter().filter_map(Value::as_str).map(str::to_string).collect(),
        _ => Vec::new(),
    };
    let games = match v.get("games") {
        Some(Value::Array(list)) => list
            .iter()
            .filter_map(|g| {
                let system = g.get("esde").and_then(Value::as_str)?;
                Some(Game { path: text(g, "path").to_string(), system: system.to_string(), package: g.get("package").and_then(Value::as_str).map(str::to_string) })
            })
            .collect(),
        _ => Vec::new(),
    };
    Ok((games, roots))
}

/// The facts a line's choice needs, read from the library through the service.
pub fn facts(backend: &dyn Backend, line: &Line, by_hand: bool) -> Result<Facts, String> {
    let mut favorites = HashSet::new();
    if line.favorites {
        if let Value::Object(rows) = parse(&backend.library_get(r#"{"what":"records"}"#)?)? {
            favorites = rows.into_iter().filter(|(_, r)| r.get("favourite") == Some(&Value::Bool(true))).map(|(k, _)| k).collect();
        }
    }
    let mut members = None;
    if let Some(name) = &line.collection {
        let Value::Array(all) = parse(&backend.library_get(r#"{"what":"collections"}"#)?)? else { return Err("the library's collections are not a list".into()) };
        let Some(id) = all.iter().find(|c| text(c, "name").eq_ignore_ascii_case(name)).map(|c| token(c, "id")) else {
            let names: Vec<&str> = all.iter().map(|c| text(c, "name")).collect();
            return Err(format!("no collection is called {name}; there are: {}", if names.is_empty() { "none".to_string() } else { names.join(", ") }));
        };
        let Value::Array(paths) = parse(&backend.library_get(&format!("{{\"what\":\"members\",\"id\":{id}}}"))?)? else { return Err("the collection's members are not a list".into()) };
        members = Some(paths.iter().filter_map(Value::as_str).map(str::to_string).collect());
    }
    let mut excluded = HashSet::new();
    if !by_hand {
        if let Value::Object(rows) = parse(&backend.library_get(r#"{"what":"all_edits"}"#)?)? {
            excluded = rows.into_iter().filter(|(_, e)| text(e, "nomultiscrape") == "1").map(|(k, _)| k).collect();
        }
    }
    Ok(Facts { favorites, members, excluded })
}

/// The kinds a line asks, per system where it named the theme; video only with --videos; the notes for standard error.
pub fn kinds(backend: &dyn Backend, line: &Line, systems: &BTreeSet<String>) -> Result<(HashMap<String, Vec<String>>, Vec<String>), String> {
    let mut notes = Vec::new();
    let mut per = HashMap::new();
    for system in systems {
        let mut ks: Vec<String> = if line.theme {
            let Value::Array(drawn) = parse(&backend.theme_kinds(system)?)? else { return Err(format!("the theme's kinds for {system} are not a list")) };
            drawn.iter().filter_map(Value::as_str).filter(|k| KINDS.contains(k)).map(str::to_string).collect()
        } else {
            line.kinds.clone().unwrap_or_else(|| DEFAULT_KINDS.iter().map(|k| k.to_string()).collect())
        };
        if line.videos && !ks.iter().any(|k| k == "video") {
            ks.push("video".into());
        } else if !line.videos && ks.iter().any(|k| k == "video") {
            ks.retain(|k| k != "video");
            if !notes.iter().any(|n: &String| n.contains("videos are off")) {
                notes.push("scrape: videos are off by default and were left out; --videos asks for them".to_string());
            }
        }
        per.insert(system.clone(), ks);
    }
    Ok((per, notes))
}

/// The kinds one run asks: each system's in the order they first appear.
pub fn union(per: &HashMap<String, Vec<String>>, systems: &BTreeSet<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for s in systems {
        for k in per.get(s).into_iter().flatten() {
            if !out.contains(k) {
                out.push(k.clone());
            }
        }
    }
    out
}

/// Where a game's picture of one kind comes from: the scraper's source word, or none.
pub fn source(backend: &dyn Backend, game: &Game, kind: &str) -> Result<Option<String>, String> {
    let request = compact(vec![("what", Value::String(String::from("media"))), ("path", Value::String(String::from(&game.path))), ("system", Value::String(String::from(&game.system))), ("type", Value::String(String::from(kind))), ("folder", Value::Bool(false))]);
    let v = parse(&backend.get(&request)?)?;
    Ok(if v.is_null() { None } else { Some(text(&v, "source").to_string()) })
}

/// A `scrape ls` line: path, system, and each kind with its source or `-`, tab-separated.
pub fn ls_line(game: &Game, sources: &[(String, Option<String>)]) -> String {
    let cells: Vec<String> = sources.iter().map(|(k, s)| format!("{k}={}", s.as_deref().unwrap_or("-"))).collect();
    format!("{}\t{}\t{}\n", game.path, game.system, cells.join(" "))
}

/// The plan request for the chosen games.
pub fn plan_request(chosen: &[Game], kinds: &[String], overwrite: bool) -> String {
    compact(vec![
        ("what", Value::String(String::from("plan"))),
        ("games", Value::Array(chosen.iter().map(|g| Value::String(String::from(&g.path))).collect())),
        ("one", Value::Bool(false)),
        ("refresh", Value::Bool(overwrite)),
        ("kinds", Value::Array(kinds.iter().map(|s| Value::String(String::from(s))).collect())),
        ("enabled", Value::Bool(true)),
    ])
}

/// The plan as `key<TAB>value` lines for standard output, and its sentence for standard error.
pub fn plan_lines(plan: &Value, kinds: &[String]) -> (String, String) {
    let games = number(plan, "games") as u64;
    let complete = games.saturating_sub(number(plan, "not_yet_asked") as u64 + number(plan, "asked_again") as u64);
    let minutes = (number(plan, "time_ticks") / 600_000_000.0).ceil() as u64;
    let left = token(plan, "requests_left_today");
    let mut out = String::new();
    for (k, v) in [
        ("games", games.to_string()),
        ("not_yet_asked", token(plan, "not_yet_asked")),
        ("asked_again", token(plan, "asked_again")),
        ("complete", complete.to_string()),
        ("max_requests", token(plan, "max_requests")),
        ("expected_requests", format!("{:.0}", number(plan, "expected_requests"))),
        ("expected_mb", format!("{:.1}", number(plan, "expected_mb"))),
        ("requests_left_today", if left.is_empty() { "unknown".into() } else { left.clone() }),
        ("minutes", minutes.to_string()),
        ("usable", token(plan, "usable")),
        ("why_not", text(plan, "why_not").to_string()),
        ("kinds", kinds.join(",")),
    ] {
        out.push_str(&format!("{k}\t{v}\n"));
    }
    let mut said = format!("scrape: {games} game(s), about {:.0} request(s) (at most {}), {complete} already complete", number(plan, "expected_requests"), token(plan, "max_requests"));
    if !left.is_empty() {
        said.push_str(&format!("; {left} request(s) left today"));
    }
    said.push_str(&format!("; about {minutes} min"));
    if plan.get("usable") == Some(&Value::Bool(false)) {
        said.push_str(&format!(". ScreenScraper cannot be used now: {}", text(plan, "why_not")));
    }
    (out, said + ".\n")
}

/// The start request for the chosen games: a wide run, as Mistress's Scrape Games makes it.
pub fn start_request(chosen: &[Game], line: &Line, kinds: &[String]) -> String {
    if line.resume {
        return compact(vec![("games", Value::Array(Vec::new())), ("resume", Value::Bool(true)), ("enabled", Value::Bool(true))]);
    }
    let mut choices = vec![("kinds", Value::Array(kinds.iter().map(|s| Value::String(String::from(s))).collect()))];
    if let Some(r) = &line.region {
        choices.push(("region", Value::String(String::from(r))));
    }
    if let Some(l) = &line.language {
        choices.push(("language", Value::String(String::from(l))));
    }
    if let Some(t) = line.threads {
        choices.push(("threads", Value::from(t as u64)));
    }
    let priority = if line.systems.len() == 1 { "console" } else { "library" };
    compact(vec![
        ("games", Value::Array(chosen.iter().map(|g| Value::Object(vec![(String::from("path"), Value::String(String::from(&g.path))), (String::from("system"), Value::String(String::from(&g.system)))].into_iter().collect())).collect())),
        ("priority", Value::String(String::from(priority))),
        ("one", Value::Bool(false)),
        ("resume", Value::Bool(false)),
        ("refresh", Value::Bool(line.overwrite)),
        ("enabled", Value::Bool(true)),
        ("choices", Value::Object(choices.into_iter().map(|(k, v)| (String::from(k), v)).collect())),
    ])
}

/// A finished game as its line: outcome, path, the kinds saved, and what the scraper said.
pub fn result_line(r: &Value) -> String {
    let kinds = match r.get("kinds") {
        Some(Value::Array(k)) => k.iter().filter_map(Value::as_str).collect::<Vec<_>>().join(","),
        _ => String::new(),
    };
    let detail = text(r, "detail");
    let from = if r.get("from_failover") == Some(&Value::Bool(true)) { "\topenemu" } else { "" };
    format!("{}\t{}\t{}{from}{}\n", text(r, "outcome"), text(r, "path"), if kinds.is_empty() { "-" } else { &kinds }, if detail.is_empty() { String::new() } else { format!("\t{detail}") })
}

/// The run's end as its sentence for standard error.
pub fn end_line(p: &Value) -> String {
    let mut said = format!(
        "scrape: {} of {} game(s) {}: {} found, {} unknown, {} failed, {} skipped, {} unchanged; {} request(s)",
        token(p, "done"),
        token(p, "total"),
        text(p, "state"),
        token(p, "found"),
        token(p, "unknown"),
        token(p, "failed"),
        token(p, "skipped"),
        token(p, "unchanged"),
        token(p, "requests")
    );
    if let Some(f) = p.get("failover").filter(|f| number(f, "asked") > 0.0) {
        said.push_str(&format!("; OpenEmu's sources asked for {}, {} found", token(f, "asked"), token(f, "found")));
    }
    let why = text(p, "why");
    if !why.is_empty() {
        said.push_str(&format!(". {why}"));
    }
    said + ".\n"
}

/// `scrape status`'s lines: the quota, the account (never its name or password), the developer's origin and the run.
pub fn status_lines(quota: &Value, account: &Value, developer: &Value, progress: &Value) -> String {
    let mut out = String::new();
    if let Value::Object(members) = quota {
        for (k, v) in members {
            if !matches!(v, Value::Object(_) | Value::Array(_)) {
                out.push_str(&format!("quota.{}\t{}\n", k, if v.is_null() { "-".to_string() } else { token(quota, k) }));
            }
        }
    }
    out.push_str(&format!("account.signed_in\t{}\n", token(account, "signed_in")));
    out.push_str(&format!("account.verified\t{}\n", account.get("verified").and_then(Value::as_str).unwrap_or("-")));
    out.push_str(&format!("developer\t{}\n", developer.get("origin").and_then(Value::as_str).unwrap_or("none")));
    out.push_str(&format!("run.state\t{}\n", text(progress, "state")));
    for k in ["total", "done", "found", "unknown", "failed", "skipped", "requests", "interrupted"] {
        out.push_str(&format!("run.{k}\t{}\n", token(progress, k)));
    }
    out
}

/// One run this shell is following: its number and the last result it printed.
struct Running {
    id: u32,
    since: u64,
    done: String,
    total: String,
    current: String,
    stopping: bool,
}

#[derive(Default)]
struct Jobs {
    next: u32,
    running: Option<Running>,
}

enum Followed {
    Ended(i32),
    Detached,
}

/// Prints a run's results as they finish, until it ends or the line is cancelled, which its budget also does.
fn follow(call: &Call, backend: &dyn Backend, r: &mut Running) -> Followed {
    loop {
        if call.cancelled() {
            return Followed::Detached;
        }
        let p = match backend.progress(r.since).map_err(|w| w.to_string()).and_then(|a| parse(&a)) {
            Ok(p) => p,
            Err(words) => {
                call.err(&format!("scrape: {words}\n"));
                return Followed::Ended(1);
            }
        };
        if let Some(Value::Array(results)) = p.get("results") {
            for x in results {
                call.out(&result_line(x));
                r.since = r.since.max(number(x, "serial") as u64);
            }
        }
        (r.done, r.total) = (token(&p, "done"), token(&p, "total"));
        r.current = p.get("current").map(|c| text(c, "path").to_string()).unwrap_or_default();
        match text(&p, "state") {
            "running" => {
                backend.wait(number(&p, "serial") as u64, 100);
            }
            "idle" => return Followed::Ended(0),
            _ => {
                call.err(&end_line(&p));
                return Followed::Ended(if text(&p, "state") == "done" && number(&p, "failed") == 0.0 { 0 } else { 1 });
            }
        }
    }
}

fn follow_or_detach(call: &Call, backend: &dyn Backend, jobs: &mut Jobs) -> i32 {
    let Some(r) = jobs.running.as_mut() else { return 0 };
    match follow(call, backend, r) {
        Followed::Ended(status) => {
            jobs.running = None;
            status
        }
        Followed::Detached => {
            let id = r.id;
            call.err(&format!("scrape{id} goes on in the background: scrape wait scrape{id} follows it, ps lists it, kill scrape{id} stops it.\n"));
            0
        }
    }
}

/// What the game choice and kinds of a line come to, with the notes for standard error.
fn chosen(call: &Call, backend: &dyn Backend, line: &Line) -> Result<(Vec<Game>, HashMap<String, Vec<String>>, Vec<String>), String> {
    let mut by_hand = line.paths.clone();
    if let Some(stdin) = call.stdin() {
        by_hand.extend(listed(&stdin));
    }
    choice(backend, line, &by_hand, &|p| call.resolve(p).ok())
}

/// The games a line and its paths named by hand choose, each system's kinds, and the notes: the one choice, the shell's and a window's.
pub fn choice(backend: &dyn Backend, line: &Line, by_hand: &[String], resolve: &dyn Fn(&str) -> Option<String>) -> Result<(Vec<Game>, HashMap<String, Vec<String>>, Vec<String>), String> {
    let (games, roots) = games_of(&backend.games()?)?;
    let facts = facts(backend, line, !by_hand.is_empty())?;
    let resolve = |p: &str| -> Option<String> {
        resolve(p).filter(|r| games.iter().any(|g| g.path == *r)).or_else(|| roots.first().map(|root| format!("{}/{}", root.trim_end_matches('/'), p.trim_start_matches('/'))))
    };
    let (mut picked, mut notes) = choose(&games, line, by_hand, &resolve, &facts);
    let systems: BTreeSet<String> = picked.iter().map(|g| g.system.clone()).collect();
    let (per, kind_notes) = kinds(backend, line, &systems)?;
    notes.extend(kind_notes);
    if let Some(missing) = &line.missing {
        let mut kept = Vec::new();
        for g in picked {
            if lacks(backend, &g, missing, &per)? {
                kept.push(g);
            }
        }
        picked = kept;
    }
    Ok((picked, per, notes))
}

/// Whether a game lacks what `--missing` names: one of the run's kinds for `any`, a description for `metadata`, otherwise every kind listed.
fn lacks(backend: &dyn Backend, g: &Game, missing: &str, per: &HashMap<String, Vec<String>>) -> Result<bool, String> {
    if missing == METADATA {
        let request = compact(vec![("what", Value::String(String::from("metadata"))), ("path", Value::String(String::from(&g.path))), ("scraped_names", Value::Bool(false))]);
        let m = parse(&backend.get(&request)?)?;
        return Ok(m.get("description").map_or(true, |d| text(d, "value").trim().is_empty()));
    }
    if missing == "any" {
        for k in per.get(&g.system).into_iter().flatten() {
            if source(backend, g, k)?.is_none() {
                return Ok(true);
            }
        }
        return Ok(false);
    }
    for k in missing.split(',') {
        if source(backend, g, k)?.is_some() {
            return Ok(false);
        }
    }
    Ok(true)
}

fn ls(call: &Call, backend: &dyn Backend, line: &Line) -> Result<i32, String> {
    let (picked, per, notes) = chosen(call, backend, line)?;
    for n in notes {
        call.err(&format!("{n}\n"));
    }
    for g in &picked {
        if call.cancelled() {
            call.err("scrape ls: stopped\n");
            return Ok(1);
        }
        let mut sources = Vec::new();
        for k in per.get(&g.system).into_iter().flatten() {
            sources.push((k.clone(), source(backend, g, k)?));
        }
        call.out(&ls_line(g, &sources));
    }
    Ok(0)
}

fn plan(call: &Call, backend: &dyn Backend, line: &Line) -> Result<(Vec<Game>, Vec<String>, Value), String> {
    let (picked, per, notes) = chosen(call, backend, line)?;
    for n in notes {
        call.err(&format!("{n}\n"));
    }
    let systems: BTreeSet<String> = picked.iter().map(|g| g.system.clone()).collect();
    let kinds = union(&per, &systems);
    let kinds = if kinds.is_empty() { line.kinds.clone().unwrap_or_else(|| DEFAULT_KINDS.iter().map(|k| k.to_string()).collect()) } else { kinds };
    let p = parse(&backend.get(&plan_request(&picked, &kinds, line.overwrite))?)?;
    Ok((picked, kinds, p))
}

fn run(call: &Call, backend: &dyn Backend, jobs: &mut Jobs, line: &Line) -> Result<i32, String> {
    if jobs.running.is_some() {
        return Err("a run this shell started is still going: scrape wait follows it, kill stops it".into());
    }
    let (picked, kinds) = if line.resume {
        (Vec::new(), Vec::new())
    } else {
        let (picked, kinds, p) = plan(call, backend, line)?;
        call.err(&plan_lines(&p, &kinds).1);
        if picked.is_empty() {
            call.err("scrape: no game is chosen, so nothing is asked.\n");
            return Ok(1);
        }
        (picked, kinds)
    };
    let since = number(&parse(&backend.progress(ALL_SEEN)?)?, "serial") as u64;
    match backend.start(&start_request(&picked, line, &kinds)) {
        Ok(0) => {
            call.err(if line.resume { "scrape: no interrupted run is queued.\n" } else { "scrape: nothing was queued.\n" });
            return Ok(1);
        }
        Ok(_) => {}
        Err((BUSY, _)) => return Err("a run is already going, started in EmuSen or another shell; scrape status shows it".into()),
        Err((_, words)) => return Err(words),
    }
    jobs.next += 1;
    jobs.running = Some(Running { id: jobs.next, since, done: String::new(), total: String::new(), current: String::new(), stopping: false });
    Ok(follow_or_detach(call, backend, jobs))
}

fn status(call: &Call, backend: &dyn Backend) -> Result<i32, String> {
    let quota = parse(&backend.get(r#"{"what":"quota"}"#)?)?;
    let account = parse(&backend.get(r#"{"what":"account"}"#)?)?;
    let developer = parse(&backend.get(r#"{"what":"developer"}"#)?)?;
    let progress = parse(&backend.progress(ALL_SEEN)?)?;
    call.out(&status_lines(&quota, &account, &developer, &progress));
    Ok(0)
}

/// A request's whole number, read as its token so that a serial past 2^53 is not rounded; 0 where there is none.
fn whole(r: &Value, name: &str) -> u64 {
    token(r, name).parse().unwrap_or(0)
}

fn object(members: Vec<(&str, Value)>) -> Value {
    Value::Object(members.into_iter().map(|(k, v)| (String::from(k), v)).collect())
}

fn strings(list: &[String]) -> Value {
    Value::Array(list.iter().map(|s| Value::String(String::from(s))).collect())
}

fn notes_said(notes: Vec<String>) -> Value {
    Value::Array(notes.iter().map(|n| Value::String(String::from(n.strip_prefix("scrape: ").unwrap_or(n)))).collect())
}

/// The command as data, for a window that fronts it: `{"args": [...], "paths": [...], "sources": bool}` with the words
/// after `scrape` (ls, plan, run, status, progress, wait, cancel, collections), answered in JSON; a failure is a status and words.
/// See EmuSen_LibraryTools.md §9.
pub fn ask(backend: &dyn Backend, request: &str) -> Result<String, (i64, String)> {
    const BAD: i64 = -1163;
    let r = serde_json::from_str::<Value>(request).map_err(|e| (BAD, format!("the request is not JSON: {e:?}")))?;
    let words: Vec<String> = match r.get("args") {
        Some(Value::Array(a)) => a.iter().filter_map(Value::as_str).map(str::to_string).collect(),
        _ => return Err((BAD, "the request names no args".into())),
    };
    let paths: Vec<String> = match r.get("paths") {
        Some(Value::Array(a)) => a.iter().filter_map(Value::as_str).map(str::to_string).collect(),
        _ => Vec::new(),
    };
    let bad = |w: String| (BAD, w);
    let answer = match words.first().map(String::as_str).unwrap_or("") {
        "progress" => {
            let p = parse(&backend.progress(whole(&r, "since").min(ALL_SEEN)).map_err(bad)?).map_err(bad)?;
            let said = if text(&p, "state") == "running" { Value::Null } else { let e = end_line(&p); Value::String(String::from(e.trim_end().strip_prefix("scrape: ").unwrap_or(e.trim_end()))) };
            object(vec![("progress", p), ("said", said)])
        }
        "collections" => parse(&backend.library_get(r#"{"what":"collections"}"#).map_err(bad)?).map_err(bad)?,
        "wait" => object(vec![("serial", Value::from(backend.wait(whole(&r, "serial"), whole(&r, "timeout_ms").min(250) as u32)))]),
        "cancel" => object(vec![("cancelled", Value::Bool(backend.cancel()))]),
        "status" => object(vec![
            ("quota", parse(&backend.get(r#"{"what":"quota"}"#).map_err(bad)?).map_err(bad)?),
            ("account", parse(&backend.get(r#"{"what":"account"}"#).map_err(bad)?).map_err(bad)?),
            ("developer", parse(&backend.get(r#"{"what":"developer"}"#).map_err(bad)?).map_err(bad)?),
            ("progress", parse(&backend.progress(ALL_SEEN).map_err(bad)?).map_err(bad)?),
        ]),
        _ => {
            let mut argv = vec!["scrape".to_string()];
            argv.extend(words);
            let line = parse_line(&argv).map_err(bad)?;
            if line.sub == "status" {
                return Err((BAD, "status takes nothing more".into()));
            }
            let (picked, per, notes) = if line.resume { (Vec::new(), HashMap::new(), Vec::new()) } else { choice(backend, &line, &paths, &|_| None).map_err(bad)? };
            let systems: BTreeSet<String> = picked.iter().map(|g| g.system.clone()).collect();
            let kinds = union(&per, &systems);
            let kinds = if kinds.is_empty() { line.kinds.clone().unwrap_or_else(|| DEFAULT_KINDS.iter().map(|k| k.to_string()).collect()) } else { kinds };
            match line.sub.as_str() {
                "ls" => {
                    let with_sources = r.get("sources") != Some(&Value::Bool(false));
                    let mut games = Vec::new();
                    for g in &picked {
                        let mut fields = vec![("path", Value::String(String::from(&g.path))), ("system", Value::String(String::from(&g.system))), ("package", g.package.as_deref().map_or(Value::Null, |s| Value::String(String::from(s))))];
                        if with_sources {
                            let mut media = Vec::new();
                            for k in per.get(&g.system).into_iter().flatten() {
                                media.push((String::from(k), source(backend, g, k).map_err(bad)?.map_or(Value::Null, |s| Value::String(String::from(&s)))));
                            }
                            fields.push(("media", Value::Object(media.into_iter().collect())));
                        }
                        games.push(object(fields));
                    }
                    let per_value = Value::Object(systems.iter().map(|s| (String::from(s), strings(per.get(s).map(Vec::as_slice).unwrap_or(&[])))).collect());
                    object(vec![("games", Value::Array(games)), ("kinds", per_value), ("notes", notes_said(notes))])
                }
                "plan" => {
                    let plan = parse(&backend.get(&plan_request(&picked, &kinds, line.overwrite)).map_err(bad)?).map_err(bad)?;
                    let said = plan_lines(&plan, &kinds).1;
                    object(vec![("plan", plan), ("kinds", strings(&kinds)), ("said", Value::String(String::from(said.trim_end().strip_prefix("scrape: ").unwrap_or(said.trim_end())))), ("notes", notes_said(notes))])
                }
                _ => {
                    if picked.is_empty() && !line.resume {
                        return Err((BAD, "no game is chosen, so nothing is asked".into()));
                    }
                    let since = number(&parse(&backend.progress(ALL_SEEN).map_err(bad)?).map_err(bad)?, "serial") as u64;
                    let queued = match backend.start(&start_request(&picked, &line, &kinds)) {
                        Ok(n) => n,
                        Err((BUSY, _)) => return Err((BUSY, "a run is already going, started in EmuSen, a shell or another window; it is shown here as it goes".into())),
                        Err(e) => return Err(e),
                    };
                    object(vec![("queued", Value::from(queued.max(0) as u64)), ("since", Value::from(since)), ("kinds", strings(&kinds)), ("notes", notes_said(notes))])
                }
            }
        }
    };
    let mut out = String::new();
    out.push_str(&serde_json::to_string(&answer).expect("a value is JSON"));
    Ok(out)
}

fn wait(call: &Call, backend: &dyn Backend, jobs: &mut Jobs) -> Answer {
    let args = call.args();
    let ok = match (&jobs.running, args.get(2)) {
        (Some(_), None) => true,
        (Some(r), Some(id)) => id.strip_prefix("scrape").and_then(|n| n.parse::<u32>().ok()) == Some(r.id),
        (None, _) => false,
    };
    if !ok {
        return Answer::fail(format!("scrape wait: no such scrape job{}", args.get(2).map(|a| format!(" {a}")).unwrap_or_default()));
    }
    Answer::with(String::new(), follow_or_detach(call, backend, jobs))
}

fn jobs_of<'a>(call: &'a Call) -> std::sync::MutexGuard<'a, Jobs> {
    call.slot(Jobs::default)
}

fn backend_of(call: &Call) -> Option<Service> {
    Some(Service { scrape: call.service(SERVICE, MAJOR)? as *const ScrapeServiceV2, games: call.service(GAMES, GAMES_MAJOR)? as *const GamesServiceV1 })
}

pub struct ScrapeCommands;

pub static MODULE: ScrapeCommands = ScrapeCommands;

impl Module for ScrapeCommands {
    fn manifest(&self) -> &'static str {
        static MANIFEST: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        MANIFEST.get_or_init(|| Manifest { id: "emusen.scrape", version: env!("CARGO_PKG_VERSION"), services: &[(SERVICE, MAJOR, 0, true), (GAMES, GAMES_MAJOR, 0, true)], commands: COMMANDS, aliases: &[], jobs: &["scrape"], settings: &[], approvals: false }.json())
    }

    fn run(&self, call: &Call) -> i32 {
        if call.command() != "scrape" {
            return call.answer(Answer::fault(format!("emusen.scrape has no command '{}'", call.command())));
        }
        let Some(backend) = backend_of(call) else { return call.answer(Answer::fail("scrape: needs emusen.scrape and emusen.games, which this program does not provide")) };
        let args = call.args();
        if args.get(1).map(String::as_str) == Some("wait") {
            return call.answer(wait(call, &backend, &mut jobs_of(call)));
        }
        let line = match parse_line(&args) {
            Ok(l) => l,
            Err(usage) => return call.answer(Answer::fail(usage)),
        };
        let done = match line.sub.as_str() {
            "ls" => ls(call, &backend, &line),
            "plan" => plan(call, &backend, &line).map(|(_, kinds, p)| {
                let (out, said) = plan_lines(&p, &kinds);
                call.out(&out);
                call.err(&said);
                0
            }),
            "run" => run(call, &backend, &mut jobs_of(call), &line),
            _ => status(call, &backend),
        };
        match done {
            Ok(status) => call.answer(Answer::with(String::new(), status)),
            Err(words) => call.answer(Answer::fail(format!("scrape: {words}"))),
        }
    }

    fn jobs(&self, call: &Call) {
        let Some(backend) = backend_of(call) else { return };
        let mut jobs = jobs_of(call);
        let Some(r) = jobs.running.as_mut() else { return };
        let p = backend.progress(ALL_SEEN).ok().and_then(|a| parse(&a).ok());
        let state = p.as_ref().map(|p| text(p, "state").to_string()).unwrap_or_default();
        if let Some(p) = &p {
            (r.done, r.total) = (token(p, "done"), token(p, "total"));
            r.current = p.get("current").map(|c| text(c, "path").to_string()).unwrap_or_default();
        }
        let progress = if state != "running" { "ended".to_string() } else if r.stopping { "stopping".to_string() } else { format!("{}/{} games", r.done, r.total) };
        call.out(&format!("  scrape{:<4} run         {progress}{}\n", r.id, if r.current.is_empty() || state != "running" { String::new() } else { format!(", {}", r.current) }));
    }

    fn kill(&self, call: &Call, id: &str) -> Option<i32> {
        let n = id.strip_prefix("scrape")?.parse::<u32>().ok()?;
        let backend = backend_of(call)?;
        let mut jobs = jobs_of(call);
        match jobs.running.as_mut().filter(|r| r.id == n) {
            Some(r) => {
                backend.cancel();
                r.stopping = true;
                call.out(&format!("scrape{n} is stopping: the games not reached are not asked; scrape wait scrape{n} shows how far it got.\n"));
                Some(0)
            }
            None => {
                call.err(&format!("kill: no scrape job scrape{n} found.\n"));
                Some(1)
            }
        }
    }
}

#[cfg(test)]
mod tests;
