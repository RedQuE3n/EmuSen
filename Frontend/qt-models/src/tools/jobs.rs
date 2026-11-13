//! A tool that is a list of the platform library's jobs, as plain data (`jobs.json`): Library Manager, Compressed Discs
//! and Packages. Each row is a job; A starts it (after a question where it writes a great deal) and, while it runs,
//! stops it; the bar follows its progress and the pane its latest result line and its outcome; X lists every result
//! line. The jobs' events are the library tool's (EmuSen_GamePackages.md §12.3, EmuSen_LibraryTools.md §7.1). No Qt here.

use super::cores::backend::Job;
use super::{Action, Bar, Column, Effect, Item, Mark, Pane, Source, Tone, Words};
use crate::card::{Pane as CardPane, Row as CardRow};
use crate::question::confirm;
use serde_json::Value;
use std::collections::HashMap;

/// The tools in order, each its key (`tool`) and its jobs.
const TOOLS: &str = include_str!("jobs.json");

/// A's actions: a job started, and the running job stopped.
pub const START: &str = "start";
pub const STOP: &str = "stop";

/// One job of a tool: its row's key and name, what it does, the word its progress is said with, the request it starts,
/// the question asked first (none for a job that only reads), and the states whose lines are not kept.
#[derive(Clone, Debug, PartialEq)]
pub struct Spec {
    pub key: String,
    pub name: String,
    pub about: String,
    pub doing: String,
    pub request: Value,
    pub ask: Vec<String>,
    pub quiet: Vec<String>,
}

fn all() -> Value {
    serde_json::from_str(TOOLS).expect("jobs.json is JSON")
}

/// The tools that are lists of jobs, by key, in the file's order.
pub fn tools() -> Vec<String> {
    all().as_array().into_iter().flatten().filter_map(|t| t.get("tool").and_then(Value::as_str).map(str::to_owned)).collect()
}

/// A tool's jobs; none for a key that is no such tool.
pub fn specs(tool: &str) -> Vec<Spec> {
    let words = |v: &Value, k: &str| v.get(k).and_then(Value::as_str).unwrap_or_default().to_owned();
    let list = |v: &Value, k: &str| v.get(k).and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_str).map(str::to_owned).collect()).unwrap_or_default();
    all()
        .as_array()
        .into_iter()
        .flatten()
        .find(|t| t.get("tool").and_then(Value::as_str) == Some(tool))
        .and_then(|t| t.get("jobs"))
        .and_then(Value::as_array)
        .map(|jobs| {
            jobs.iter()
                .map(|j| Spec {
                    key: words(j, "key"),
                    name: words(j, "name"),
                    about: words(j, "about"),
                    doing: words(j, "doing"),
                    request: j.get("request").cloned().unwrap_or(Value::Null),
                    ask: list(j, "ask"),
                    quiet: list(j, "quiet"),
                })
                .collect()
        })
        .unwrap_or_default()
}

/// What a job has said so far: how many items it has, how many have ended, the one it is on, each result line, and
/// its outcome once it has ended.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Run {
    pub total: Option<u64>,
    pub done: u64,
    pub working: String,
    pub lines: Vec<Words>,
    pub outcome: Option<Words>,
}

impl Run {
    /// The share done, while the job's count is known.
    pub fn progress(&self) -> Option<f64> {
        self.total.filter(|t| *t > 0).map(|t| (self.done as f64 / t as f64).min(1.0))
    }

    /// One event taken: `start`'s count and refusals, an item's state, a group, `end`'s counts or error.
    pub fn take(&mut self, event: &Value, quiet: &[String]) {
        let text = |k: &str| event.get(k).and_then(Value::as_str).unwrap_or_default().to_owned();
        let count = |k: &str| event.get(k).and_then(Value::as_u64);
        match text("event").as_str() {
            "start" => {
                let total = ["discs", "packages", "files"].iter().filter_map(|k| count(k)).sum::<u64>() + count("games").unwrap_or(0);
                self.total = Some(total);
                for refused in event.get("refused").and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str) {
                    self.lines.push(Words::new(refused, Tone::Error));
                }
                if !text("chdman_words").is_empty() {
                    self.lines.push(Words::new(format!("chdman: {}", text("chdman_words")), Tone::Attention));
                }
            }
            "end" => self.outcome = Some(outcome(event)),
            "group" => {
                let names: Vec<String> = event.get("members").and_then(Value::as_array).into_iter().flatten().map(|m| m.get("name").and_then(Value::as_str).unwrap_or_default().to_owned()).collect();
                self.lines.push(Words::new(format!("The same dump: {}", names.join(", ")), Tone::Attention));
            }
            _ => {
                let state = text("state");
                let source = [text("source"), text("package")].into_iter().find(|s| !s.is_empty()).unwrap_or_default();
                if state == "working" {
                    self.working = source;
                    return;
                }
                self.done += 1;
                self.working.clear();
                if !quiet.contains(&state) {
                    self.lines.push(line(&source, &state, event));
                }
            }
        }
    }
}

/// An item's result line: where, its state, and the tool's words or the checks it failed.
fn line(source: &str, state: &str, event: &Value) -> Words {
    let failed: Vec<String> = event
        .get("failed")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|f| format!("{}: {}", f.get("check").and_then(Value::as_str).unwrap_or_default(), f.get("words").and_then(Value::as_str).unwrap_or_default()))
        .collect();
    let words = event.get("words").and_then(Value::as_str).filter(|w| !w.is_empty()).map(str::to_owned).or_else(|| (!failed.is_empty()).then(|| failed.join("; ")));
    let tone = match state {
        "failed" => Tone::Error,
        "stopped" => Tone::Muted,
        _ => Tone::Text,
    };
    match words {
        Some(w) => Words::new(format!("{source}: {state}, {w}"), tone),
        None => Words::new(format!("{source}: {state}"), tone),
    }
}

/// `end`'s words: its error, or its counts and seconds; a failure or a stop among them makes it a warning.
fn outcome(event: &Value) -> Words {
    if let Some(error) = event.get("error").and_then(Value::as_str) {
        return Words::new(error, Tone::Error);
    }
    let counts: Vec<(String, u64)> = event.get("counts").and_then(Value::as_object).into_iter().flatten().filter_map(|(k, v)| Some((k.clone(), v.as_u64()?))).collect();
    let said = if counts.is_empty() { "Nothing found".to_owned() } else { counts.iter().map(|(k, n)| format!("{n} {k}")).collect::<Vec<_>>().join(", ") };
    let seconds = event.get("seconds").and_then(Value::as_f64).unwrap_or(0.0);
    let tone = if counts.iter().any(|(k, _)| k == "failed" || k == "stopped") { Tone::Attention } else { Tone::Success };
    Words::new(format!("{said}; {seconds} s."), tone)
}

/// Starts a job from its request.
pub type Starter = Box<dyn Fn(&Value) -> Result<Box<dyn Job>, String>>;

/// A tool of jobs: its title, what it does, its jobs, how they start, the running one and every job's last run.
pub struct Jobs {
    title: String,
    about: String,
    specs: Vec<Spec>,
    starter: Starter,
    running: Option<(String, Box<dyn Job>)>,
    runs: HashMap<String, Run>,
    /// The job last started, whose outcome the bar says; and the same, taken once to focus its row.
    last: Option<String>,
    acted: Option<String>,
}

impl Jobs {
    /// The tool of `key` (its title and sentence the Home card's), its jobs started by `starter`; none for no such tool.
    pub fn new(key: &str, starter: Starter) -> Option<Jobs> {
        let specs = specs(key);
        let (title, about) = crate::home_card::tool_words(key)?;
        (!specs.is_empty()).then(|| Jobs { title: title.to_owned(), about: about.to_owned(), specs, starter, running: None, runs: HashMap::new(), last: None, acted: None })
    }

    fn spec(&self, key: &str) -> Option<&Spec> {
        self.specs.iter().find(|s| s.key == key)
    }

    pub fn run(&self, key: &str) -> Option<&Run> {
        self.runs.get(key)
    }

    /// The job of `key` started, its last run forgotten; a refusal is its outcome.
    pub fn start(&mut self, key: &str) {
        let Some(spec) = self.spec(key).cloned() else { return };
        if self.running.is_some() {
            return;
        }
        let mut run = Run::default();
        match (self.starter)(&spec.request) {
            Ok(job) => self.running = Some((key.to_owned(), job)),
            Err(words) => run.outcome = Some(Words::new(words, Tone::Error)),
        }
        self.runs.insert(key.to_owned(), run);
        self.last = Some(key.to_owned());
        self.acted = Some(key.to_owned());
    }

    /// The running job asked to stop; it ends with what it did.
    pub fn stop(&mut self) {
        if let Some((key, job)) = self.running.as_mut() {
            if let Err(words) = job.stop() {
                let key = key.clone();
                self.runs.entry(key).or_default().lines.push(Words::new(words, Tone::Error));
            }
        }
    }

    fn state(&self, key: &str) -> Words {
        let running = self.running.as_ref().is_some_and(|(k, _)| k == key);
        match (running, self.runs.get(key)) {
            (true, Some(run)) => match (run.total, run.done) {
                (Some(total), done) => Words::new(format!("{done} of {total}"), Tone::Accent),
                (None, _) => Words::new("Starting", Tone::Accent),
            },
            (false, Some(Run { outcome: Some(o), .. })) => Words::new(if o.tone == Tone::Error { "Failed" } else if o.tone == Tone::Attention { "Done, with failures" } else { "Done" }, o.tone),
            _ => Words::new("", Tone::Muted),
        }
    }
}

impl Source for Jobs {
    fn title(&self) -> &str {
        &self.title
    }

    fn section_word(&self) -> &str {
        "Section"
    }

    fn columns(&self) -> Vec<Column> {
        vec![
            Column { title: "Job", fill: 1.0, least: 190.0, trailing: false, gives_way: false },
            Column { title: "State", fill: 0.0, least: 0.0, trailing: true, gives_way: false },
        ]
    }

    fn items(&self) -> Vec<Item> {
        self.specs
            .iter()
            .map(|s| Item { key: s.key.clone(), cells: vec![Words::text(&s.name), self.state(&s.key)], search: s.name.clone(), ..Item::default() })
            .collect()
    }

    /// The job's name, what it does, then while it runs the item it is on, and the latest result line and the outcome.
    fn pane(&self, key: Option<&str>) -> Pane {
        let Some(spec) = key.and_then(|k| self.spec(k)) else { return Pane { title: self.title.clone(), caption: String::new(), sentences: vec![Words::text(&self.about)] } };
        let mut sentences = vec![Words::text(&spec.about)];
        if let Some(run) = self.runs.get(&spec.key) {
            if !run.working.is_empty() {
                sentences.push(Words::new(format!("{} {}", spec.doing, run.working), Tone::Accent));
            }
            if let Some(last) = run.lines.last() {
                sentences.push(last.clone());
            }
            if let Some(outcome) = &run.outcome {
                sentences.push(outcome.clone());
            }
            if run.lines.len() > 1 {
                sentences.push(Words::new(format!("X lists all {} lines.", run.lines.len()), Tone::Muted));
            }
        }
        Pane { title: spec.name.clone(), caption: String::new(), sentences }
    }

    /// The running job's progress, or the last job's outcome.
    fn bar(&self) -> Bar {
        if let Some((key, _)) = &self.running {
            let run = self.runs.get(key).cloned().unwrap_or_default();
            let doing = self.spec(key).map(|s| s.doing.clone()).unwrap_or_default();
            let line = match run.total {
                Some(total) => format!("{doing}: {} of {total}", run.done),
                None => format!("{doing}…"),
            };
            return Bar { line: Words::text(line), progress: Some(run.progress().unwrap_or(0.0)), ..Bar::default() };
        }
        let last = self.last.as_ref().and_then(|k| self.runs.get(k)).and_then(|r| r.outcome.clone());
        Bar { line: last.unwrap_or_else(|| Words::new("No job has run", Tone::Muted)), ..Bar::default() }
    }

    fn action(&self, key: &str) -> Option<Action> {
        self.spec(key).map(|s| Action::new(START, if s.ask.is_empty() { "Start" } else { "Start…" }))
    }

    fn running_action(&self, key: &str) -> Option<Action> {
        self.running.as_ref().filter(|(k, _)| k == key).map(|_| Action::new(STOP, "Stop"))
    }

    fn plans(&self) -> bool {
        false
    }

    /// Every result line of the job's last run, its outcome last.
    fn page(&self, key: &str, _marked: Option<&Mark>) -> Vec<CardRow> {
        let Some(run) = self.runs.get(key) else { return Vec::new() };
        run.lines
            .iter()
            .chain(&run.outcome)
            .enumerate()
            .map(|(at, l)| CardRow::new(&format!("line{at}"), &l.words).pane(CardPane::said(&l.words, "")))
            .collect()
    }

    fn act(&mut self, action: &str, key: Option<&str>, _section: &str) -> Effect {
        let Some(spec) = key.and_then(|k| self.spec(k)).cloned() else { return Effect::Done };
        match action {
            STOP => self.stop(),
            START if !spec.ask.is_empty() => {
                let words: Vec<&str> = spec.ask.iter().map(String::as_str).collect();
                return Effect::Ask(confirm(&format!("{}?", spec.name), &words), spec.key);
            }
            START => self.start(&spec.key),
            _ => {}
        }
        Effect::Done
    }

    fn answer(&mut self, asked: &str, yes: bool) {
        if yes {
            self.start(asked);
        }
    }

    /// Nothing is read until a job runs; a running job is followed on.
    fn refresh(&mut self) {}

    fn poll(&mut self) -> bool {
        let Some((key, mut job)) = self.running.take() else { return false };
        let quiet = self.spec(&key).map(|s| s.quiet.clone()).unwrap_or_default();
        let mut events = Vec::new();
        let ended = job.drain(&mut events);
        let run = self.runs.entry(key.clone()).or_default();
        for event in &events {
            run.take(event, &quiet);
        }
        match ended {
            Ok(false) => self.running = Some((key, job)),
            Ok(true) => {
                if run.outcome.is_none() {
                    run.outcome = Some(Words::new("Ended without a word", Tone::Error));
                }
            }
            Err(words) => run.outcome = Some(Words::new(words, Tone::Error)),
        }
        !events.is_empty() || self.running.is_none()
    }

    fn running(&self) -> bool {
        self.running.is_some()
    }

    fn acted_on(&mut self) -> Option<String> {
        self.acted.take()
    }
}
