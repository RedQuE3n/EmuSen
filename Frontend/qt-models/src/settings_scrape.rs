//! Settings -> Library -> Scraping (account and run) page (board #62, R-20261116-24), wired to emusen.scrape.

use crate::scrape_run::Run;
use crate::settings::{Outcome, Page, Pane, Row, SettingsView};
use crate::settings_controllers::Typed;
use crate::settings_shown::{BoardAside, BoardRow, Form};
use crate::words;
use emusen_kit_core::settings::RowKind;
use serde_json::{json, Value};

/// Live ScreenScraper account, queue, run progress and quota state for Settings -> Library -> Scraping (#62).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScrapeState {
    pub signed_in: bool,
    pub account_name: String,
    pub verified: String,
    pub queued: usize,
    pub run: Run,
    pub requests_today: u64,
    pub max_per_day: Option<u64>,
    pub threads: u64,
    pub gate: String,
    pub status_note: String,
    pub pending_user: Option<String>,
}

const ROWS: [&str; 6] = ["account", "login", "resume", "cancel", "status", "scraper"];

impl SettingsView {
    pub(crate) fn scrape_entries(&self, page: &Page) -> Option<Vec<Value>> {
        if *page != Page::ScrapeRun {
            return None;
        }
        Some(ROWS.iter().map(|k| json!({ "id": format!("scrape:{k}"), "scrape": k })).collect())
    }

    fn scrape_row_spec(&self, key: &str) -> Option<(String, String, String, String)> {
        let s = &self.scrape;
        let queued_value = if s.queued == 0 { "None queued".to_owned() } else { format!("{} queued", words::games(s.queued)) };
        let spec = match key {
            "account" => {
                let value = if s.signed_in { s.account_name.clone() } else { "Not signed in".to_owned() };
                let aside = if !s.signed_in {
                    "Not signed in: runs use EmuSen's developer credentials alone.".to_owned()
                } else if !s.verified.is_empty() {
                    format!("Signed in as {} (checked {}).", s.account_name, s.verified)
                } else {
                    format!("Signed in as {}.", s.account_name)
                };
                ("Member account".to_owned(), "Your ScreenScraper account, for more requests a day and faster ones.".to_owned(), value, aside)
            }
            "login" => {
                let label = if s.signed_in { "Sign out" } else { "Log in" }.to_owned();
                let about = "Your user name and password, checked with one request and kept only if accepted, in a file only you can read.".to_owned();
                let aside = if s.signed_in {
                    "Removes your ScreenScraper credentials from this machine; future runs use EmuSen's developer credentials alone.".to_owned()
                } else {
                    about.clone()
                };
                (label, about, String::new(), aside)
            }
            "resume" => {
                let aside = if s.run.going() {
                    format!("Scraping now: {} of {} done.", s.run.done, s.run.total)
                } else if s.queued > 0 {
                    format!("{} queued from a stopped run. A resumes from where it stopped.", words::games(s.queued))
                } else {
                    "No games are queued from a stopped run.".to_owned()
                };
                ("Resume".to_owned(), "Goes on with the games a stopped run left queued; nothing resumes by itself.".to_owned(), queued_value, aside)
            }
            "cancel" => {
                let aside = if s.run.going() {
                    "Stops the running scrape; what it has not reached stays queued for Resume.".to_owned()
                } else {
                    "No scrape run is going now.".to_owned()
                };
                ("Cancel scraping".to_owned(), "Stops the run once you say so; what it has not reached stays queued for Resume.".to_owned(), String::new(), aside)
            }
            "status" => {
                let run_part = if s.run.going() {
                    format!("Running: {} of {} done.", s.run.done, s.run.total)
                } else if s.queued > 0 {
                    format!("Idle · {} queued.", words::games(s.queued))
                } else {
                    "Idle.".to_owned()
                };
                let quota_part = match s.max_per_day {
                    Some(max) => format!("Today: {} of {} requests.", s.requests_today, max),
                    None => format!("Today: {} requests.", s.requests_today),
                };
                ("Status".to_owned(), "The run's progress, the account's limits and today's requests.".to_owned(), String::new(), format!("{run_part} {quota_part}"))
            }
            "scraper" => {
                let about = "The tool that chooses which games a run fetches for and starts it.".to_owned();
                ("Scraper".to_owned(), about.clone(), String::new(), about)
            }
            _ => return None,
        };
        Some(spec)
    }

    pub(crate) fn scrape_row(&self, entry: &Value) -> Option<Row> {
        let key = entry["scrape"].as_str()?;
        let (label, _, value, _) = self.scrape_row_spec(key)?;
        Some(Row { id: format!("scrape:{key}"), label, value, kind: RowKind::Opens, held: false, marked: false, code: false })
    }

    pub(crate) fn scrape_pane(&self, entry: &Value) -> Option<Pane> {
        let key = entry["scrape"].as_str()?;
        let (_, _, _, aside) = self.scrape_row_spec(key)?;
        let muted = if self.scrape.status_note.is_empty() { Vec::new() } else { vec![self.scrape.status_note.clone()] };
        Some(Pane { sentence: aside, muted, ..Pane::default() })
    }

    pub(crate) fn scrape_step(&mut self, entry: &Value) -> Option<Outcome> {
        entry["scrape"].as_str().map(|_| Outcome::Nothing)
    }

    pub(crate) fn scrape_choose(&mut self, entry: &Value) -> Option<Outcome> {
        let key = entry["scrape"].as_str()?;
        Some(match key {
            "account" => {
                if self.scrape.signed_in {
                    Outcome::ScrapeCheck
                } else {
                    self.scrape.pending_user = None;
                    Outcome::Type { title: "Log in".to_owned(), prompt: "Your ScreenScraper user name.".to_owned(), start: String::new(), typed: Typed::ScrapeUser }
                }
            }
            "login" => {
                if self.scrape.signed_in {
                    Outcome::ScrapeSignOut
                } else {
                    self.scrape.pending_user = None;
                    Outcome::Type { title: "Log in".to_owned(), prompt: "Your ScreenScraper user name.".to_owned(), start: String::new(), typed: Typed::ScrapeUser }
                }
            }
            "resume" => Outcome::ScrapeResume,
            "cancel" => Outcome::ScrapeCancel,
            "status" => Outcome::ScrapeRefresh,
            "scraper" => Outcome::Door("scraper".to_owned()),
            _ => Outcome::Nothing,
        })
    }

    pub(crate) fn scrape_board_row(&self, entry: &Value) -> Option<BoardRow> {
        let key = entry["scrape"].as_str()?;
        let (label, about, value, _) = self.scrape_row_spec(key)?;
        Some(BoardRow { label, about, value, form: Form::Door, planet: -3, subtitle: String::new(), chip: String::new(), chip_attention: false, runs: true })
    }

    pub(crate) fn scrape_board_aside(&self, focused: usize, board_rows: &[BoardRow], pane: &Pane, out: &mut BoardAside) {
        out.trail_line = "Library".to_owned();
        out.heading = "Scraping".to_owned();
        out.page_glow = "#1FA890".to_owned();
        out.about_heading = board_rows.get(focused).map(|r| r.label.clone()).unwrap_or_default();
        out.sentence = pane.sentence.clone();
        out.lines.clear();
        out.muted = pane.muted.clone();
        out.wrap_about = true;
        out.cards_help = true;
        out.box_menu = true;
    }
}
