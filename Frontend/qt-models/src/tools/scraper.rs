//! The Scraper on the tool screen (EmuSen_Qt.md §7.7.F): a row for the whole library and one for each console; A starts
//! a run over the row's games (`scrape_run`, the run's one owner), which asks only for what each game lacks of what the
//! Scraping settings ask for. While it runs the bar follows the service's progress, A on its row pauses and resumes it,
//! and the List card's Stop the scrape cancels it; X lists each game's result. No Qt here.

use super::{Action, Bar, Column, Effect, Item, Mark, Pane, Source, Tone, Words};
use crate::card::{Pane as CardPane, Row as CardRow};
use crate::clients::ClientSource;
use crate::library::LibrarySource;
use crate::question::{confirm, Question};
use crate::scrape_run::{self, Run};
use serde_json::Value;
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;

/// A's actions on a row, and the List card's Stop the scrape and its question.
pub const START: &str = "start";
pub const PAUSE: &str = "pause";
pub const RESUME: &str = "resume";
pub const STOP: &str = "stop";

/// The whole library's row.
pub const ALL: &str = crate::library::ALL;

/// What the run's thread tells: a progress, or its end with what went wrong.
enum Told {
    Progress(Value),
    Ended(Result<(), String>),
}

/// The tool: the clients its runs go through, the rows' consoles, the test seam, and the run going or last ended.
pub struct Scraper {
    source: Arc<ClientSource>,
    consoles: Vec<(String, String)>,
    test: Option<Value>,
    /// The row the run was started from, its progress so far, and while it goes what its thread tells.
    scope: Option<String>,
    run: Option<Run>,
    told: Option<Receiver<Told>>,
    problem: Option<String>,
    acted: Option<String>,
}

impl Scraper {
    /// The tool over `source`, `test` its scraper's test seam where one is given.
    pub fn new(source: Arc<ClientSource>, test: Option<Value>) -> Scraper {
        Scraper { source, consoles: Vec::new(), test, scope: None, run: None, told: None, problem: None, acted: None }
    }

    fn title_words() -> (&'static str, &'static str) {
        crate::home_card::tool_words(crate::home_card::SCRAPER).unwrap_or_default()
    }

    fn name(&self, key: &str) -> String {
        if key == ALL {
            return crate::library::PLACES[0].1.to_owned();
        }
        self.consoles.iter().find(|c| c.0 == key).map(|c| c.1.clone()).unwrap_or_default()
    }

    /// The run over the row of `key` started off the GUI thread: the library's games, the console's alone for a console.
    pub fn start(&mut self, key: &str) {
        if self.running() {
            return;
        }
        let (tx, rx) = channel();
        let (source, test, scope) = (self.source.clone(), self.test.clone(), key.to_owned());
        std::thread::spawn(move || {
            let games = match source.games() {
                Ok(all) => all.into_iter().filter(|g| scope == ALL || g.system.as_deref() == Some(scope.as_str())).collect::<Vec<_>>(),
                Err(why) => return drop(tx.send(Told::Ended(Err(why)))),
            };
            if let Err(why) = scrape_run::start(&source, &games, false, test.as_ref()) {
                return drop(tx.send(Told::Ended(Err(why))));
            }
            let followed = scrape_run::follow(&source, |p| drop(tx.send(Told::Progress(p))));
            let _ = tx.send(Told::Ended(followed));
        });
        self.scope = Some(key.to_owned());
        self.run = Some(Run { state: "running".into(), ..Run::default() });
        self.told = Some(rx);
        self.problem = None;
        self.acted = Some(key.to_owned());
    }

    /// The run paused or resumed.
    pub fn pause(&mut self, paused: bool) {
        if let Err(why) = self.source.scrape_pause(paused) {
            self.problem = Some(why);
        }
    }

    /// The run stopped; what was fetched is kept.
    pub fn stop(&mut self) {
        if let Err(why) = self.source.scrape_cancel() {
            self.problem = Some(why);
        }
    }

    pub fn run(&self) -> Option<&Run> {
        self.run.as_ref()
    }

    /// The row's state: how far its run is, or how it ended.
    fn state(&self, key: &str) -> Words {
        let Some(run) = self.run.as_ref().filter(|_| self.scope.as_deref() == Some(key)) else { return Words::new("", Tone::Muted) };
        match (self.running(), run.paused) {
            (true, true) => Words::new(format!("Paused, {} of {}", run.done, run.total), Tone::Attention),
            (true, false) if run.total > 0 => Words::new(format!("{} of {}", run.done, run.total), Tone::Accent),
            (true, false) => Words::new("Starting", Tone::Accent),
            (false, _) => {
                let (words, tone) = self.outcome();
                Words::new(words, tone)
            }
        }
    }

    /// How the last run ended, said short: Stopped, Failed, Done with failures, or Done.
    fn outcome(&self) -> (&'static str, Tone) {
        match self.run.as_ref() {
            _ if self.problem.is_some() => ("Failed", Tone::Error),
            Some(r) if r.state == "cancelled" || r.state == "stopped" => ("Stopped", Tone::Muted),
            Some(r) if r.why.is_some() => ("Failed", Tone::Error),
            Some(r) if r.failed > 0 => ("Done, with failures", Tone::Attention),
            _ => ("Done", Tone::Success),
        }
    }

    /// A game's name as its result line says it: its file's name less its extension.
    fn game(path: &str) -> String {
        std::path::Path::new(path).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
    }
}

impl Source for Scraper {
    fn title(&self) -> &str {
        Scraper::title_words().0
    }

    fn section_word(&self) -> &str {
        "Section"
    }

    fn columns(&self) -> Vec<Column> {
        vec![
            Column { title: "Console", fill: 1.0, least: 190.0, trailing: false, gives_way: false },
            Column { title: "State", fill: 0.0, least: 0.0, trailing: true, gives_way: false },
        ]
    }

    fn items(&self) -> Vec<Item> {
        std::iter::once(ALL.to_owned())
            .chain(self.consoles.iter().map(|c| c.0.clone()))
            .map(|key| {
                let name = self.name(&key);
                Item { cells: vec![Words::text(&name), self.state(&key)], search: name, key, ..Item::default() }
            })
            .collect()
    }

    /// The row's name and what the tool does; for the run's row the game in hand and its step, the latest result, what
    /// last failed, and how the run ended.
    fn pane(&self, key: Option<&str>) -> Pane {
        let mut sentences = vec![Words::text(Scraper::title_words().1)];
        let title = key.map(|k| self.name(k)).unwrap_or_else(|| self.title().to_owned());
        if let (Some(run), true) = (self.run.as_ref(), key.is_some() && key == self.scope.as_deref()) {
            if let Some((path, step)) = run.current.as_ref().filter(|_| self.running()) {
                sentences.push(Words::new(format!("{}: {step}", Scraper::game(path)), Tone::Accent));
            }
            if let Some((path, words, failed)) = run.results.first() {
                sentences.push(Words::new(format!("{}: {words}", Scraper::game(path)), if *failed { Tone::Error } else { Tone::Text }));
            }
            if let Some(failure) = &run.last_failure {
                sentences.push(Words::new(failure, Tone::Error));
            }
            if let Some(why) = self.problem.as_ref().or(run.why.as_ref()) {
                sentences.push(Words::new(format!("It could not run: {why}"), Tone::Error));
            }
            if run.results.len() > 1 {
                sentences.push(Words::new(format!("X lists all {} lines.", run.results.len()), Tone::Muted));
            }
        }
        Pane { title, caption: String::new(), sentences }
    }

    /// The run's progress, paused or going, or how the last one ended.
    fn bar(&self) -> Bar {
        let Some(run) = self.run.as_ref() else { return Bar { line: Words::new("No job has run", Tone::Muted), ..Bar::default() } };
        if self.running() {
            let doing = if run.paused { "Paused" } else { "Scraping" };
            return Bar { line: Words::text(format!("{doing}: {} of {}", run.done, run.total)), progress: Some(run.share()), ..Bar::default() };
        }
        let (words, tone) = self.outcome();
        Bar { line: Words::new(format!("{words}: {} of {}", run.done, run.total), tone), ..Bar::default() }
    }

    fn action(&self, key: &str) -> Option<Action> {
        (key == ALL || self.consoles.iter().any(|c| c.0 == key)).then(|| Action::new(START, "Start"))
    }

    /// A on the run's row pauses it, or resumes it.
    fn running_action(&self, key: &str) -> Option<Action> {
        let run = self.run.as_ref().filter(|_| self.scope.as_deref() == Some(key))?;
        Some(if run.paused { Action::new(RESUME, "Resume") } else { Action::new(PAUSE, "Pause") })
    }

    fn plans(&self) -> bool {
        false
    }

    /// Each game's result of the row's last run, the newest first.
    fn page(&self, key: &str, _marked: Option<&Mark>) -> Vec<CardRow> {
        let Some(run) = self.run.as_ref().filter(|_| self.scope.as_deref() == Some(key)) else { return Vec::new() };
        run.results
            .iter()
            .enumerate()
            .map(|(at, (path, words, _))| {
                let line = format!("{}: {words}", Scraper::game(path));
                CardRow::new(&format!("line{at}"), &line).pane(CardPane::said(&line, ""))
            })
            .collect()
    }

    /// While a run goes, Stop the scrape.
    fn list_rows(&self) -> Vec<CardRow> {
        if self.running() { vec![CardRow::new(STOP, "Stop the scrape")] } else { Vec::new() }
    }

    fn marks_for(&self, _key: &str) -> Vec<Mark> {
        Vec::new()
    }

    fn summary(&self, _marked: &[(String, Mark)]) -> String {
        String::new()
    }

    fn plan(&self, _marked: &[(String, Mark)]) -> Question {
        confirm(self.title(), &[])
    }

    fn apply(&mut self, _marked: &[(String, Mark)]) {}

    fn act(&mut self, action: &str, key: Option<&str>, _section: &str) -> Effect {
        match (action, key) {
            (START, Some(key)) => self.start(key),
            (PAUSE, _) => self.pause(true),
            (RESUME, _) => self.pause(false),
            (STOP, _) if self.running() => return Effect::Ask(scrape_run::stop_question(), STOP.to_owned()),
            _ => {}
        }
        Effect::Done
    }

    fn answer(&mut self, asked: &str, yes: bool) {
        if yes && asked == STOP {
            self.stop();
        }
    }

    /// The consoles read anew from the library's shelves, as the screen opens; a running run is followed on.
    fn refresh(&mut self) {
        let systems = self.source.systems().unwrap_or_default();
        let shelves = self.source.shelves().unwrap_or_default();
        self.consoles = shelves.iter().filter_map(|id| systems.iter().find(|s| &s.id == id)).map(|s| (s.id.clone(), crate::library::console_name(s))).collect();
        self.consoles.dedup();
    }

    fn poll(&mut self) -> bool {
        let Some(told) = self.told.as_ref() else { return false };
        let mut changed = false;
        while let Ok(t) = told.try_recv() {
            changed = true;
            match t {
                Told::Progress(p) => self.run = Some(Run::of(&p)),
                Told::Ended(ended) => {
                    self.told = None;
                    if let Err(why) = ended {
                        self.problem = Some(why);
                    }
                    if let Some(run) = self.run.as_mut().filter(|r| r.going()) {
                        run.state = "stopped".into();
                    }
                    break;
                }
            }
        }
        changed
    }

    fn running(&self) -> bool {
        self.told.is_some()
    }

    fn acted_on(&mut self) -> Option<String> {
        self.acted.take()
    }

    /// A run's end leaves the library with new pictures and facts.
    fn changes_library(&self) -> bool {
        true
    }
}
