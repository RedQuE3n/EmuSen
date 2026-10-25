//! A scrape run on the player's word alone, `Scraper` and the window's half of a run ported (`StartScrape`,
//! `ScrapeArrived`, `FailoverFor`, `ScraperFinished`, `EndRun`): the queue's workers take each due game, identify it as
//! §5.2 orders, keep its text and media, and hand what ScreenScraper left without a cover to OpenEmu's sources; the
//! host drains what happened. See EmuSen_BigPicture.md §17, §17.14, §38, §46 and EmuSen_Backend.md §30.3.

use crate::client::{self, Client, Hashes, MediaOutcome, Wire};
use crate::clock::Stop;
use crate::consoles::{self, Core};
use crate::kinds::{self, Kind};
use crate::openemu;
use crate::quota::{BUSY_WAIT, Gate, Status, TOO_MANY_WAIT};
use crate::redact::redact;
use crate::rules;
use crate::store::{Priority, Queued, Record, State, Stored};
use crate::{Error, Scraper, layout};
use emusen_dotnet::date::TICKS_PER_SECOND;
use emusen_galaxia::dotnet_files::{file_exists, file_info};
use emusen_galaxia::dotnet_path::{self as path, Style};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::thread::JoinHandle;
use std::time::Duration;

pub const MAX_ATTEMPTS: i32 = 5;
pub const RECENT_LIMIT: usize = 200;
const IDLE_POLL: Duration = Duration::from_millis(100);
const STYLE: Style = Style::HOST;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Found,
    Unknown,
    Error,
    Retry,
    Stopped,
    Missing,
}

impl Outcome {
    pub fn word(self) -> &'static str {
        match self {
            Outcome::Found => "found",
            Outcome::Unknown => "unknown",
            Outcome::Error => "error",
            Outcome::Retry => "retry",
            Outcome::Stopped => "stopped",
            Outcome::Missing => "missing",
        }
    }
}

/// What one game's turn came to, `ScrapeResult`.
#[derive(Clone, Debug)]
pub struct Finished {
    pub path: String,
    pub system: String,
    pub outcome: Outcome,
    pub has_cover: bool,
    pub written: Vec<String>,
    pub detail: String,
    pub matched_by: Option<String>,
    pub skipped: bool,
    pub unchanged: i32,
}

impl Finished {
    fn new(item: &Queued, outcome: Outcome, detail: impl Into<String>) -> Finished {
        Finished { path: item.path.clone(), system: item.system.clone(), outcome, has_cover: false, written: Vec::new(), detail: detail.into(), matched_by: None, skipped: false, unchanged: 0 }
    }
}

/// The run's choices, the player's preferences passed with each run.
#[derive(Clone, Debug)]
pub struct Choices {
    pub kinds: Vec<Kind>,
    pub region: String,
    pub language: String,
    pub region_fallback: bool,
    pub threads: i32,
    pub refresh: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Phase {
    Idle,
    Running,
    Done,
    Stopped,
    Cancelled,
}

impl Phase {
    pub fn word(&self) -> &'static str {
        match self {
            Phase::Idle => "idle",
            Phase::Running => "running",
            Phase::Done => "done",
            Phase::Stopped => "stopped",
            Phase::Cancelled => "cancelled",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Row {
    pub serial: u64,
    pub path: String,
    pub system: String,
    pub outcome: Outcome,
    pub kinds: Vec<String>,
    pub detail: String,
    pub skipped: bool,
    pub has_cover: bool,
    pub matched_by: Option<String>,
    pub from_failover: bool,
}

#[derive(Clone, Debug)]
pub struct Activity {
    pub path: String,
    pub system: String,
    pub step: &'static str,
    pub kind: Option<String>,
    pub saved: Option<String>,
}

/// The run as the host drains it, `ScrapeProgress`.
pub struct Progress {
    pub serial: u64,
    pub phase: Phase,
    pub total: i32,
    pub done: i32,
    pub found: i32,
    pub unknown: i32,
    pub failed: i32,
    pub skipped: i32,
    pub retrying: i32,
    pub unchanged: i32,
    pub requests: i32,
    pub last_failure: Option<String>,
    pub why: Option<String>,
    pub failover_asked: i32,
    pub failover_found: i32,
    pub failover_pending: HashSet<String>,
    pub current: Option<Activity>,
    pub last_picture: Option<String>,
    pub started: i64,
    pub ended: Option<i64>,
    pub paused_at: Option<i64>,
    pub paused_for: i64,
    pub recent: VecDeque<Row>,
    /// Whether ScreenScraper's workers still run in this run.
    pub workers_live: bool,
}

impl Default for Progress {
    fn default() -> Progress {
        Progress {
            serial: 0,
            phase: Phase::Idle,
            total: 0,
            done: 0,
            found: 0,
            unknown: 0,
            failed: 0,
            skipped: 0,
            retrying: 0,
            unchanged: 0,
            requests: 0,
            last_failure: None,
            why: None,
            failover_asked: 0,
            failover_found: 0,
            failover_pending: HashSet::new(),
            current: None,
            last_picture: None,
            started: 0,
            ended: None,
            paused_at: None,
            paused_for: 0,
            recent: VecDeque::new(),
            workers_live: false,
        }
    }
}

/// The ScreenScraper workers' shared state for one run.
struct Workers {
    client: Arc<Client>,
    /// Set when the run stops, so that a request left to end on its own stops at its next read.
    cancel: Arc<emusen_net::Progress>,
    choices: Choices,
    stop: Stop,
    held: Mutex<HashSet<String>>,
    live: Mutex<i32>,
    stopped_by_quota: AtomicBool,
    handles: Mutex<Vec<JoinHandle<()>>>,
    signal: Stop,
}

/// The failover's worker and what it was asked this session, `CoverFetcher`.
#[derive(Default)]
pub struct Failover {
    asked: HashSet<String>,
    queue: VecDeque<(String, Core)>,
    waiting: Vec<(String, Core)>,
    pub(crate) downloading: bool,
    running: bool,
}

/// What a running scraper holds between calls.
#[derive(Default)]
pub struct Runs {
    pub progress: Mutex<Progress>,
    pub changed: Condvar,
    workers: Mutex<Option<Arc<Workers>>>,
    pub paused: Mutex<bool>,
    pub unpaused: Condvar,
    pub picks: Mutex<HashMap<i64, crate::answer::Game>>,
    chosen: Mutex<HashMap<String, crate::answer::Game>>,
    pub failover: Mutex<Failover>,
    failover_stop: Arc<Stop>,
    failover_thread: Mutex<Option<JoinHandle<()>>>,
}

/// A handle's address, sent to its own threads; the handle joins them before it is freed.
#[derive(Clone, Copy)]
struct Handle(*const Scraper);
unsafe impl Send for Handle {}

impl Handle {
    fn get(&self) -> &'static Scraper {
        unsafe { &*self.0 }
    }
}

fn io(e: std::io::Error, at: &str) -> String {
    emusen_dotnet::file::io_message(&e, at)
}

enum Turn {
    Done(Finished),
    Io(String),
    Cancelled,
}

impl Scraper {
    pub(crate) fn progress(&self) -> MutexGuard<'_, Progress> {
        self.runs.progress.lock().unwrap_or_else(|p| p.into_inner())
    }

    fn touch(&self, p: &mut Progress) {
        p.serial += 1;
        self.runs.changed.notify_all();
    }

    pub fn is_running(&self) -> bool {
        self.progress().phase == Phase::Running
    }

    /// ScreenScraper's id for an ES-DE system, from the systems table.
    pub(crate) fn screenscraper_id(esde: &str) -> Option<i32> {
        consoles::by_esde(esde).and_then(|s| s.screenscraper_system).map(|i| i as i32)
    }

    /// Why ScreenScraper cannot answer a run now, or none when it can.
    pub(crate) fn unusable(&self, enabled: bool) -> Result<Option<String>, Error> {
        if !enabled {
            return Ok(Some("it is switched off".into()));
        }
        if self.developer().is_none() {
            return Ok(Some(crate::credentials::NONE_HERE.into()));
        }
        let (gate, until, why) = {
            let store = self.store();
            self.quota.check(&store, &self.clock)?
        };
        if gate == Gate::Stopped {
            let mut words = why.unwrap_or_default();
            if let Some(u) = until {
                words.push_str(&format!(", until {}", local_day_time(u)));
            }
            return Ok(Some(words));
        }
        Ok(None)
    }

    /// One run over a scope: the queue replaced by it unless resuming, ScreenScraper asked where it can be, the
    /// failover where it has nothing. How many games are queued; zero starts nothing.
    #[allow(clippy::too_many_arguments)]
    pub fn start(&self, games: &[(String, String)], priority: Priority, one: bool, resume: bool, pick: Option<i64>, enabled: bool, choices: Choices) -> Result<usize, Error> {
        if self.is_running() {
            return Err(Error::Busy("A run is already going.".into()));
        }
        let queued = {
            let store = self.store();
            if !resume {
                store.clear_queue()?;
                for (rom, system) in games {
                    if one {
                        store.forget_unfound(rom)?;
                        if let Ok(library) = self.library() {
                            let _ = library.change_request(&format!("{{\"op\":\"forget_cover_lookup\",\"path\":{}}}", emusen_dotnet::json::quoted(rom)));
                        }
                        self.runs.failover.lock().unwrap_or_else(|p| p.into_inner()).asked.remove(rom);
                    }
                    store.enqueue(rom, system, priority)?;
                }
            }
            store.queue()?
        };
        if queued.is_empty() {
            return Ok(0);
        }
        {
            let mut p = self.progress();
            *p = Progress { phase: Phase::Running, total: queued.len() as i32, started: self.clock.now(), serial: p.serial, ..Progress::default() };
            self.touch(&mut p);
        }
        *self.runs.paused.lock().unwrap_or_else(|p| p.into_inner()) = false;
        if let Some(why) = self.unusable(enabled)? {
            let mut p = self.progress();
            p.why = Some(redact(&why));
            drop(p);
            self.failover_for(queued.iter().map(|q| q.path.clone()).collect());
            let mut p = self.progress();
            if p.failover_pending.is_empty() {
                self.end_run(&mut p, Phase::Stopped);
            }
            return Ok(queued.len());
        }
        let developer = self.developer().expect("checked just above");
        let member = Some(crate::credentials::Member::load()).filter(|m| m.is_set());
        let wire = Wire::of(&self.settings());
        let workers = Arc::new(Workers {
            client: Arc::new(Client { wire, developer, member: std::sync::RwLock::new(member) }),
            cancel: Arc::new(emusen_net::Progress::default()),
            choices,
            stop: Stop::default(),
            held: Mutex::new(HashSet::new()),
            live: Mutex::new(0),
            stopped_by_quota: AtomicBool::new(false),
            handles: Mutex::new(Vec::new()),
            signal: Stop::default(),
        });
        if let (Some(pick), true) = (pick, one) {
            if let Some(game) = self.runs.picks.lock().unwrap_or_else(|p| p.into_inner()).get(&pick).cloned() {
                if let Some(first) = queued.first() {
                    self.runs.chosen.lock().unwrap_or_else(|p| p.into_inner()).insert(first.path.clone(), game);
                    self.store().enqueue(&first.path, &first.system, Priority::Shown)?;
                }
            }
        }
        self.progress().workers_live = true;
        *self.runs.workers.lock().unwrap_or_else(|p| p.into_inner()) = Some(workers.clone());
        self.ensure_workers(&workers);
        Ok(queued.len())
    }

    fn ensure_workers(&self, w: &Arc<Workers>) {
        let mut handles = w.handles.lock().unwrap_or_else(|p| p.into_inner());
        if w.stop.is_stopped() {
            return;
        }
        let mut live = w.live.lock().unwrap_or_else(|p| p.into_inner());
        if !handles.is_empty() && *live == 0 {
            return;
        }
        let wanted = self.quota.threads(w.choices.threads) as usize;
        while handles.len() < wanted {
            let index = handles.len();
            *live += 1;
            let (me, w2) = (Handle(self as *const Scraper), w.clone());
            handles.push(std::thread::spawn(move || me.get().work(index, &w2)));
        }
    }

    fn wait_while_paused(&self, stop: &Stop) -> bool {
        let mut paused = self.runs.paused.lock().unwrap_or_else(|p| p.into_inner());
        while *paused {
            if stop.is_stopped() {
                return false;
            }
            paused = self.runs.unpaused.wait_timeout(paused, Duration::from_millis(50)).unwrap_or_else(|p| p.into_inner()).0;
        }
        !stop.is_stopped()
    }

    /// Every request waits out a pause first, then the quota's turn: Some(true) to go, Some(false) stopped, None cancelled.
    fn turn(&self, w: &Workers) -> Option<bool> {
        if !self.wait_while_paused(&w.stop) {
            return None;
        }
        self.quota.take_turn_with(&|| self.store(), &self.clock, &w.stop).ok().flatten()
    }

    fn work(&self, index: usize, w: &Arc<Workers>) {
        loop {
            if w.stop.is_stopped() || !self.wait_while_paused(&w.stop) {
                break;
            }
            let gate = {
                let store = self.store();
                self.quota.check(&store, &self.clock).map(|g| g.0).unwrap_or(Gate::Go)
            };
            if gate == Gate::Stopped {
                w.stopped_by_quota.store(true, Ordering::Relaxed);
                break;
            }
            if index >= self.quota.threads(w.choices.threads) as usize {
                break;
            }
            let (item, others_busy) = {
                let mut held = w.held.lock().unwrap_or_else(|p| p.into_inner());
                let item = self.store().next_due(self.clock.now(), &|p| held.contains(p)).ok().flatten();
                if let Some(i) = &item {
                    held.insert(i.path.clone());
                }
                let busy = held.len() > if item.is_none() { 0 } else { 1 };
                (item, busy)
            };
            let Some(item) = item else {
                let (length, earliest) = {
                    let store = self.store();
                    (store.queue_length().unwrap_or(0), store.earliest_retry().ok().flatten())
                };
                if length == 0 && !others_busy {
                    break;
                }
                let now = self.clock.now();
                match earliest {
                    Some(due) if !others_busy && due > now => {
                        if !self.clock.delay(due - now, &w.stop) {
                            break;
                        }
                    }
                    _ => {
                        if w.signal.wait(IDLE_POLL) && w.stop.is_stopped() {
                            break;
                        }
                    }
                }
                continue;
            };
            let turn = self.process(&item, w);
            w.held.lock().unwrap_or_else(|p| p.into_inner()).remove(&item.path);
            match turn {
                Turn::Done(result) => self.arrived(result),
                Turn::Io(words) => {
                    let backoff = backoff(item.attempts);
                    let _ = self.store().retry(&item.path, self.clock.now() + backoff, true);
                    self.arrived(Finished::new(&item, Outcome::Retry, redact(&words)));
                }
                Turn::Cancelled => break,
            }
            self.ensure_workers(w);
        }
        let last = {
            let mut live = w.live.lock().unwrap_or_else(|p| p.into_inner());
            *live -= 1;
            *live == 0
        };
        if last {
            let end = if w.stop.is_stopped() {
                Phase::Cancelled
            } else if w.stopped_by_quota.load(Ordering::Relaxed) {
                Phase::Stopped
            } else {
                Phase::Done
            };
            if !w.stop.is_stopped() {
                self.scraper_finished(end);
            }
        }
    }

    /// `ScrapeArrived`: a game's turn counted, the failover asked where no cover is shown, a stop's reason kept.
    fn arrived(&self, result: Finished) {
        let mut p = self.progress();
        if p.phase != Phase::Running {
            return;
        }
        count(&mut p, &result);
        if result.outcome == Outcome::Stopped {
            p.why = Some(result.detail.clone());
        }
        self.touch(&mut p);
        drop(p);
        if matches!(result.outcome, Outcome::Found | Outcome::Unknown | Outcome::Error) && !result.has_cover {
            self.failover_for(vec![result.path]);
        }
    }

    pub(crate) fn note(&self, activity: Activity) {
        let mut p = self.progress();
        if matches!(activity.step, "looking_up" | "downloading") {
            p.requests += 1;
        }
        if activity.step == "arrived" && activity.kind.as_deref().and_then(kinds::by_type).is_none_or(|k| k.payload == kinds::Payload::Image) {
            p.last_picture = activity.saved.clone();
        }
        p.current = Some(activity);
        self.touch(&mut p);
    }

    /// ScreenScraper's part is over: a stop hands what is left to the failover; the run ends when the failover has answered too.
    fn scraper_finished(&self, end: Phase) {
        let mut p = self.progress();
        p.workers_live = false;
        if p.phase != Phase::Running {
            return;
        }
        set_paused(&mut p, false, self.clock.now());
        *self.runs.paused.lock().unwrap_or_else(|q| q.into_inner()) = false;
        if end == Phase::Stopped {
            let why = {
                let store = self.store();
                self.quota.check(&store, &self.clock).ok().and_then(|g| g.2)
            };
            if why.is_some() {
                p.why = why.map(|w| redact(&w));
            }
            drop(p);
            let left: Vec<String> = self.store().queue().map(|q| q.into_iter().map(|i| i.path).collect()).unwrap_or_default();
            self.failover_for(left);
            p = self.progress();
        }
        if p.failover_pending.is_empty() {
            self.end_run(&mut p, end);
        } else {
            self.touch(&mut p);
        }
    }

    pub(crate) fn end_run(&self, p: &mut Progress, end: Phase) {
        set_paused(p, false, self.clock.now());
        p.ended = Some(self.clock.now());
        p.phase = end;
        p.current = None;
        self.touch(p);
    }

    pub fn pause(&self, paused: bool) -> bool {
        let mut p = self.progress();
        if p.phase != Phase::Running || !p.workers_live || p.paused_at.is_some() == paused {
            return false;
        }
        *self.runs.paused.lock().unwrap_or_else(|q| q.into_inner()) = paused;
        self.runs.unpaused.notify_all();
        set_paused(&mut p, paused, self.clock.now());
        self.touch(&mut p);
        true
    }

    /// Stops the run: the workers cancelled and joined, the failover let go of, the run ended Cancelled.
    pub fn cancel(&self) -> bool {
        if !self.is_running() {
            return false;
        }
        self.stop_workers();
        self.stop_failover();
        let mut p = self.progress();
        p.workers_live = false;
        self.end_run(&mut p, Phase::Cancelled);
        true
    }

    /// The member a running run's requests carry from their next one: set at a sign-in, none at a sign-out.
    pub(crate) fn set_run_member(&self, member: Option<crate::credentials::Member>) {
        if let Some(w) = self.runs.workers.lock().unwrap_or_else(|p| p.into_inner()).as_ref() {
            *w.client.member.write().unwrap_or_else(|p| p.into_inner()) = member;
        }
    }

    pub(crate) fn stop_workers(&self) {
        let workers = self.runs.workers.lock().unwrap_or_else(|p| p.into_inner()).take();
        if let Some(w) = workers {
            w.cancel.cancel();
            w.stop.stop();
            w.signal.stop();
            *self.runs.paused.lock().unwrap_or_else(|p| p.into_inner()) = false;
            self.runs.unpaused.notify_all();
            let handles: Vec<JoinHandle<()>> = std::mem::take(&mut *w.handles.lock().unwrap_or_else(|p| p.into_inner()));
            for h in handles {
                let _ = h.join();
            }
        }
    }

    fn process(&self, item: &Queued, w: &Workers) -> Turn {
        match self.process_inner(item, w) {
            Ok(Some(result)) => Turn::Done(result),
            Ok(None) => Turn::Cancelled,
            Err(words) => Turn::Io(words),
        }
    }

    /// The hashes of a file, its SHA-1 too, the run's stop asked every 64 KiB; none when stopped.
    fn hashes(rom: &str, stop: &Stop) -> Result<Option<Hashes>, String> {
        match emusen_galaxia::rom_hash::hashes_with(rom, None, true, &|| stop.is_stopped()) {
            Ok(h) => Ok(Some(Hashes { md5: h.md5, crc32: h.crc32, sha1: h.sha1.unwrap_or_default(), size: h.size })),
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => Ok(None),
            Err(e) => Err(io(e, rom)),
        }
    }

    fn process_inner(&self, item: &Queued, w: &Workers) -> Result<Option<Finished>, String> {
        let rom = item.path.as_str();
        if !file_exists(rom) {
            self.store().dequeue(rom).map_err(|e| e.words())?;
            return Ok(Some(Finished { skipped: true, ..Finished::new(item, Outcome::Missing, "the file is gone") }));
        }
        let Some(system_id) = Scraper::screenscraper_id(&item.system) else {
            self.store().dequeue(rom).map_err(|e| e.words())?;
            return Ok(Some(Finished { skipped: true, ..Finished::new(item, Outcome::Missing, "this game's console has no ScreenScraper system") }));
        };
        let (bytes, modified) = file_info(rom).ok_or_else(|| format!("Could not find file '{rom}'."))?;
        let mut hashes: Option<Hashes> = None;
        let cached = self.store().cached_md5(rom, bytes, modified).map_err(|e| e.words())?;
        let md5 = match cached {
            Some(md5) => md5,
            None => {
                let Some(h) = Scraper::hashes(rom, &w.stop)? else { return Ok(None) };
                let md5 = h.md5.clone();
                self.store().remember_file(rom, bytes, modified, &md5).map_err(|e| e.words())?;
                hashes = Some(h);
                md5
            }
        };
        let settings = self.settings();
        let cores = consoles::cores(&settings.cores);
        let choices = w.choices.clone();
        let hand_cover = self.console_for(rom, &cores).is_some_and(|c| self.art().find(Some(&c), path::file_name_without_extension(STYLE, rom)).is_some());
        let wanted: Vec<Kind> = choices.kinds.iter().copied().filter(|k| !(hand_cover && k.esde_type == kinds::COVER.esde_type)).collect();

        if let Some(picked) = self.runs.chosen.lock().unwrap_or_else(|p| p.into_inner()).remove(rom) {
            return self.keep(item, &md5, bytes, &picked, &choices, &wanted, "name", true, w, &settings);
        }

        let known = self.store().game(&md5, bytes).map_err(|e| e.words())?;
        if let Some(k) = known.as_ref().filter(|k| k.state != State::Error) {
            if k.state == State::Unknown {
                self.store().dequeue(rom).map_err(|e| e.words())?;
                return Ok(Some(Finished { skipped: true, ..Finished::new(item, Outcome::Unknown, "known to be unknown") }));
            }
            let kept = self.follow_rename(&md5, bytes, rom, &settings)?;
            let stored = self.store().media(&md5, bytes).map_err(|e| e.words())?;
            if !choices.refresh && !wanted.iter().any(|kind| k.may_offer(kind.esde_type) && !stored.iter().any(|s| s.ty == kind.esde_type)) {
                self.store().dequeue(rom).map_err(|e| e.words())?;
                return Ok(Some(Finished { has_cover: self.has_cover(&md5, bytes), written: kept, matched_by: k.matched_by.clone(), skipped: true, ..Finished::new(item, Outcome::Found, "already scraped") }));
            }
        }
        let by_name = known.as_ref().is_some_and(|k| k.state == State::Found && k.matched_by.as_deref() == Some("name") && k.game_id.is_some());

        match self.turn(w) {
            None => return Ok(None),
            Some(false) => return Ok(Some(self.stopped(item))),
            Some(true) => {}
        }
        let hashes = match hashes {
            Some(h) => h,
            None => match Scraper::hashes(rom, &w.stop)? {
                Some(h) => h,
                None => return Ok(None),
            },
        };
        self.note(Activity { path: rom.into(), system: item.system.clone(), step: "looking_up", kind: None, saved: None });
        let file_name = path::file_name(STYLE, rom).to_string();
        let Some(mut answer) = self.ask(w, system_id, hashes, &file_name, if by_name { known.as_ref().and_then(|k| k.game_id) } else { None }) else { return Ok(None) };
        let mut matched_by = if by_name { "name" } else { "file" };

        if answer.status == Status::NotFound && bytes as u64 <= openemu::HASH_LIMIT {
            let core_first = self.route(rom, &cores).and_then(|s| consoles::core_of(&cores, &s.id)).and_then(|c| c.systems.first().cloned());
            if let Some(first) = core_first {
                let file = std::fs::read(rom).map_err(|e| io(e, rom))?;
                if let Some(h) = emusen_galaxia::rom_hash::transformed(&file, |f| emusen_systems::openvgdb::bytes(&first, f)) {
                    match self.turn(w) {
                        None => return Ok(None),
                        Some(false) => return Ok(Some(self.stopped(item))),
                        Some(true) => {}
                    }
                    self.note(Activity { path: rom.into(), system: item.system.clone(), step: "looking_up", kind: None, saved: None });
                    let Some(second) = self.ask(w, system_id, Hashes { md5: h.md5, crc32: h.crc32, sha1: h.sha1.unwrap_or_default(), size: h.size }, &file_name, None) else { return Ok(None) };
                    if second.status == Status::Found {
                        matched_by = "transformed";
                    }
                    if second.status != Status::NotFound {
                        answer = second;
                    }
                }
            }
        }

        let now = self.clock.now();
        match answer.status {
            Status::Found => {
                let game = answer.game.expect("a found answer has its game");
                self.keep(item, &md5, bytes, &game, &choices, &wanted, matched_by, choices.refresh, w, &settings)
            }
            Status::NotFound if known.as_ref().is_some_and(|k| k.state == State::Found) => {
                self.store().dequeue(rom).map_err(|e| e.words())?;
                let k = known.expect("matched above");
                let detail = if by_name { "found by name; Find by Name asks again" } else { "kept; ScreenScraper no longer lists this file" };
                Ok(Some(Finished { has_cover: self.has_cover(&md5, bytes), matched_by: k.matched_by.clone(), skipped: true, ..Finished::new(item, Outcome::Found, detail) }))
            }
            Status::NotFound => Ok(Some(self.final_answer(item, State::Unknown, "not in ScreenScraper", &md5, bytes)?)),
            Status::TooManyRequests | Status::ServerBusy => {
                let wait = if answer.status == Status::ServerBusy { BUSY_WAIT } else { TOO_MANY_WAIT };
                self.store().retry(rom, now + wait, false).map_err(|e| e.words())?;
                Ok(Some(Finished::new(item, Outcome::Retry, answer.detail)))
            }
            Status::DailyQuota | Status::DailyKoQuota | Status::BadCredentials | Status::ApiClosed | Status::Blacklisted => Ok(Some(self.stopped(item))),
            _ => {
                if item.attempts + 1 >= MAX_ATTEMPTS || answer.status == Status::BadRequest {
                    return Ok(Some(self.final_answer(item, State::Error, &reason(&answer), &md5, bytes)?));
                }
                self.store().retry(rom, now + backoff(item.attempts), true).map_err(|e| e.words())?;
                Ok(Some(Finished::new(item, Outcome::Retry, reason(&answer))))
            }
        }
    }

    /// One `jeuInfos`, its quota observed and counted; none when the run stopped while it was asked.
    fn ask(&self, w: &Workers, system_id: i32, hashes: Hashes, file_name: &str, game_id: Option<i64>) -> Option<client::JeuInfos> {
        let (client, cancel, name) = (w.client.clone(), w.cancel.clone(), file_name.to_string());
        let answer = crate::clock::detached(&w.stop, move || client.jeu_infos(system_id, &hashes, &name, game_id, &cancel))?;
        let store = self.store();
        let _ = self.quota.observe(&store, &self.clock, answer.quota);
        if matches!(answer.status, Status::Found | Status::NotFound | Status::Malformed | Status::BadRequest) {
            let _ = self.quota.counted(&store, &self.clock, answer.status == Status::NotFound);
        }
        let _ = self.quota.answered(&store, &self.clock, answer.status);
        Some(answer)
    }

    fn stopped(&self, item: &Queued) -> Finished {
        let why = {
            let store = self.store();
            self.quota.check(&store, &self.clock).ok().and_then(|g| g.2)
        };
        Finished::new(item, Outcome::Stopped, why.unwrap_or_else(|| "stopped".into()))
    }

    /// A game's last word: recorded against its hashes with the reason.
    fn final_answer(&self, item: &Queued, state: State, detail: &str, md5: &str, bytes: i64) -> Result<Finished, String> {
        let reason = redact(detail);
        let store = self.store();
        let mut r = Record::new(md5, bytes, state, self.clock.now());
        r.detail = Some(reason.clone());
        store.record(&r).map_err(|e| e.words())?;
        store.dequeue(&item.path).map_err(|e| e.words())?;
        Ok(Finished::new(item, if state == State::Unknown { Outcome::Unknown } else { Outcome::Error }, reason))
    }

    #[allow(clippy::too_many_arguments)]
    fn keep(&self, item: &Queued, md5: &str, bytes: i64, game: &crate::answer::Game, choices: &Choices, wanted: &[Kind], matched_by: &str, refresh: bool, w: &Workers, settings: &crate::Settings) -> Result<Option<Finished>, String> {
        let rom = item.path.as_str();
        let stem = path::file_name_without_extension(STYLE, rom).to_string();
        let file_name = path::file_name(STYLE, rom).to_string();
        let regions = rules::region_order(&file_name, &choices.region, choices.region_fallback);
        let languages = rules::language_order(&choices.language);
        let offered: Vec<&str> = kinds::ALL.iter().filter(|k| rules::choose_media(&game.media, k, &regions, true).is_some()).map(|k| k.esde_type).collect();
        let mut name_order = regions.clone();
        name_order.push("ss".into());
        let mut r = Record::new(md5, bytes, State::Found, self.clock.now());
        r.game_id = Some(game.id);
        r.rom_id = game.rom_id;
        r.system_id = game.system_id.or(Scraper::screenscraper_id(&item.system));
        r.name = rules::choose_text(&game.names, &name_order, true).map(|t| t.text.clone());
        r.description = rules::choose_text(&game.synopses, &languages, choices.region_fallback).map(|t| t.text.clone());
        r.developer = game.developer.clone();
        r.publisher = game.publisher.clone();
        r.genre = rules::genre(&game.genres, &languages);
        r.players = game.players.clone();
        r.rating = rules::rating(game.note);
        r.release_date = rules::date(rules::choose_text(&game.dates, &regions, true).map(|t| t.text.as_str())).map(rules::date_text);
        r.region = Some(regions[0].clone());
        r.language = Some(languages[0].clone());
        r.matched_by = Some(matched_by.to_string());
        r.offered = Some(offered.join(","));
        r.kinds_known = Some(kinds::ALL.iter().map(|k| k.esde_type).collect::<Vec<_>>().join(","));
        self.store().record(&r).map_err(|e| e.words())?;

        let mut written = Vec::new();
        let mut unchanged = 0;
        let stored = self.store().media(md5, bytes).map_err(|e| e.words())?;
        let root = self.root();
        let folder = settings.folder_of(rom);
        for kind in wanted {
            let kept = stored.iter().find(|s| s.ty == kind.esde_type && file_exists(&path::combine(STYLE, &root, &s.relative_path)));
            if kept.is_some() && !refresh {
                continue;
            }
            let Some(media) = rules::choose_media(&game.media, kind, &regions, choices.region_fallback) else { continue };
            let relative = layout::relative(&item.system, kind.folder, &folder, &format!("{stem}{}", rules::extension(media.format.as_deref(), kind.payload)));
            let mut local_sha1 = None;
            if let Some(k) = kept {
                let at = path::combine(STYLE, &root, &k.relative_path);
                let sha = emusen_galaxia::rom_hash::sha1(&at).map_err(|e| io(e, &at))?;
                if media.sha1.as_deref().is_some_and(|theirs| !theirs.is_empty() && theirs == sha) {
                    unchanged += 1;
                    continue;
                }
                local_sha1 = Some(sha);
            }
            match self.turn(w) {
                None => return Ok(None),
                Some(false) => break,
                Some(true) => {}
            }
            self.note(Activity { path: rom.into(), system: item.system.clone(), step: "downloading", kind: Some(kind.esde_type.into()), saved: None });
            let replace = kept.is_some_and(|k| k.relative_path == relative);
            let target = path::combine(STYLE, &root, &relative);
            let (client, cancel, url, at, payload, size) = (w.client.clone(), w.cancel.clone(), media.url.clone(), target.clone(), kind.payload, media.size);
            let got = match crate::clock::detached(&w.stop, move || client.download(&url, &at, &cancel, payload, local_sha1.as_deref(), replace, size)) {
                Some(Ok(g)) => g,
                _ => return Ok(None),
            };
            if w.stop.is_stopped() {
                return Ok(None);
            }
            self.throttle(&got, &w.stop);
            if got.outcome == MediaOutcome::Unchanged {
                unchanged += 1;
                continue;
            }
            if !matches!(got.outcome, MediaOutcome::Saved | MediaOutcome::AlreadyThere) {
                continue;
            }
            if let Some(k) = kept.filter(|_| !replace) {
                let old = path::combine(STYLE, &root, &k.relative_path);
                std::fs::remove_file(&old).map_err(|e| io(e, &old))?;
            }
            self.note(Activity { path: rom.into(), system: item.system.clone(), step: "arrived", kind: Some(kind.esde_type.into()), saved: got.path.clone() });
            self.store().record_media(md5, bytes, &Stored { ty: kind.esde_type.into(), relative_path: relative.clone(), region: media.region.clone(), sha1: got.sha1.clone() }, self.clock.now()).map_err(|e| e.words())?;
            written.push(relative);
        }
        self.store().dequeue(rom).map_err(|e| e.words())?;
        Ok(Some(Finished { has_cover: self.has_cover(md5, bytes), written, matched_by: Some(matched_by.into()), unchanged, ..Finished::new(item, Outcome::Found, "") }))
    }

    /// The member's `maxdownloadspeed`, in KB/s, held by waiting after a file that came faster.
    fn throttle(&self, got: &client::MediaAnswer, stop: &Stop) {
        let Some(speed) = self.quota.limits().and_then(|l| l.max_download_kbps).filter(|&s| s > 0) else { return };
        if got.outcome != MediaOutcome::Saved {
            return;
        }
        let owed = emusen_dotnet::date::ticks_from_seconds(got.bytes as f64 / (speed as f64 * 1024.0)) - client::ticks(got.took);
        if owed > 0 {
            self.clock.delay(owed, stop);
        }
    }

    fn has_cover(&self, md5: &str, bytes: i64) -> bool {
        let root = self.root();
        self.store().media(md5, bytes).map(|m| m.iter().any(|s| s.ty == kinds::COVER.esde_type && file_exists(&path::combine(STYLE, &root, &s.relative_path)))).unwrap_or(false)
    }

    /// A renamed or moved file takes its media to its new name and folder; a copy beside the old one gets copies.
    fn follow_rename(&self, md5: &str, bytes: i64, rom: &str, settings: &crate::Settings) -> Result<Vec<String>, String> {
        let mut moved = Vec::new();
        let stem = path::file_name_without_extension(STYLE, rom).to_string();
        let root = self.root();
        let media = self.store().media(md5, bytes).map_err(|e| e.words())?;
        for m in media {
            let old = path::combine(STYLE, &root, &m.relative_path);
            let relative = layout::relative(&layout::system_of(&m.relative_path), &layout::kind_folder_of(&m.relative_path), &settings.folder_of(rom), &format!("{stem}{}", path::extension(STYLE, &m.relative_path)));
            let target = path::combine(STYLE, &root, &relative);
            if relative == m.relative_path || !file_exists(&old) || file_exists(&target) {
                continue;
            }
            let old_stem = path::file_name_without_extension(STYLE, &m.relative_path).to_string();
            let old_folder = layout::folder_of(&m.relative_path);
            let still_there = self
                .store()
                .paths_of(md5, bytes)
                .map_err(|e| e.words())?
                .iter()
                .any(|p| p != rom && path::file_name_without_extension(STYLE, p) == old_stem && settings.folder_of(p) == old_folder && file_exists(p));
            if let Some(dir) = path::directory_name(STYLE, &target) {
                std::fs::create_dir_all(&dir).map_err(|e| io(e, &dir))?;
            }
            if still_there {
                std::fs::copy(&old, &target).map_err(|e| io(e, &old))?;
            } else {
                std::fs::rename(&old, &target).map_err(|e| io(e, &old))?;
            }
            self.store().record_media(md5, bytes, &Stored { relative_path: relative.clone(), ..m }, self.clock.now()).map_err(|e| e.words())?;
            moved.push(relative);
        }
        Ok(moved)
    }

    // --- the failover ---

    /// The cover a game shows from any source, `CoverPathFor`.
    fn cover_shown(&self, rom: &str, cores: &[Core]) -> Option<String> {
        let system = self.route(rom, cores)?;
        let game = crate::media::Game { file: rom, folder_path: self.settings().folder_of(rom), folder: false };
        self.locate(&system.esde_system, &game, kinds::COVER.esde_type).map(|(_, p)| p)
    }

    /// `FailoverFor`: the games of this run with no cover shown handed to OpenEmu's sources, each once.
    fn failover_for(&self, roms: Vec<String>) {
        let settings = self.settings();
        if !settings.openemu || self.progress().phase != Phase::Running {
            return;
        }
        let cores = consoles::cores(&settings.cores);
        for rom in roms {
            if self.cover_shown(&rom, &cores).is_some() || self.progress().failover_pending.contains(&rom) {
                continue;
            }
            let Some(core) = self.route(&rom, &cores).and_then(|s| consoles::core_of(&cores, &s.id)).cloned() else { continue };
            if self.ask_for_cover(&rom, core) {
                let mut p = self.progress();
                p.failover_pending.insert(rom);
                p.failover_asked += 1;
                self.touch(&mut p);
            }
        }
    }

    fn ask_for_cover(&self, rom: &str, core: Core) -> bool {
        if core.openvgdb_systems.is_empty() {
            return false;
        }
        if let Ok(library) = self.library() {
            let known = library.get_request(&format!("{{\"what\":\"cover_lookup\",\"path\":{}}}", emusen_dotnet::json::quoted(rom))).unwrap_or_default();
            if known != "null" && !known.is_empty() {
                return false;
            }
        }
        let mut f = self.runs.failover.lock().unwrap_or_else(|p| p.into_inner());
        if f.running {
            if !f.asked.insert(rom.to_string()) {
                return false;
            }
            f.queue.push_back((rom.to_string(), core));
            return true;
        }
        f.waiting.push((rom.to_string(), core));
        if !f.downloading {
            if file_exists(&openemu::default_path()) {
                self.start_failover(&mut f);
            } else {
                f.downloading = true;
                let me = Handle(self as *const Scraper);
                std::thread::spawn(move || me.get().download_for_failover());
            }
        }
        true
    }

    fn start_failover(&self, f: &mut Failover) {
        f.running = true;
        self.runs.failover_stop.reset();
        for (rom, core) in std::mem::take(&mut f.waiting) {
            if f.asked.insert(rom.clone()) {
                f.queue.push_back((rom, core));
            }
        }
        let me = Handle(self as *const Scraper);
        *self.runs.failover_thread.lock().unwrap_or_else(|p| p.into_inner()) = Some(std::thread::spawn(move || me.get().failover_worker()));
    }

    fn download_for_failover(&self) {
        let wire = Wire::of(&self.settings());
        let result = openemu::download(self.sqlite.clone(), &wire, &openemu::default_path(), &emusen_net::Progress::default());
        let mut f = self.runs.failover.lock().unwrap_or_else(|p| p.into_inner());
        f.downloading = false;
        match result {
            Ok(_) => {
                if self.settings().openemu && self.is_running() {
                    self.start_failover(&mut f);
                }
            }
            Err(words) => {
                let waiting = std::mem::take(&mut f.waiting);
                drop(f);
                for (rom, _) in waiting {
                    self.failover_arrived(openemu::Result { rom, outcome: openemu::Outcome::Failed, rom_name: None, saved: None, detail: Some(words.clone()) });
                }
            }
        }
    }

    fn failover_worker(&self) {
        let stop = self.runs.failover_stop.clone();
        let wire = Wire::of(&self.settings());
        let artwork = self.openemu_root();
        let mut db: Option<openemu::Database> = None;
        loop {
            if stop.is_stopped() {
                break;
            }
            let next = self.runs.failover.lock().unwrap_or_else(|p| p.into_inner()).queue.pop_front();
            let Some((rom, core)) = next else {
                if stop.wait(Duration::from_millis(20)) {
                    break;
                }
                continue;
            };
            if db.is_none() {
                db = openemu::Database::open(self.sqlite.clone(), &openemu::default_path());
            }
            let result = match &db {
                None => openemu::Result { rom: rom.clone(), outcome: openemu::Outcome::Failed, rom_name: None, saved: None, detail: Some("OpenVGDB is not downloaded".into()) },
                Some(database) => match openemu::look_up(database, &wire, &rom, &core, &artwork, &stop) {
                    Ok(r) => r,
                    Err(None) => break,
                    Err(Some(words)) => openemu::Result { rom: rom.clone(), outcome: openemu::Outcome::Failed, rom_name: None, saved: None, detail: Some(words) },
                },
            };
            if result.outcome != openemu::Outcome::Failed {
                if let Ok(library) = self.library() {
                    let mut request = format!("{{\"op\":\"record_cover_lookup\",\"path\":{},\"outcome\":\"{}\",\"rom_name\":", emusen_dotnet::json::quoted(&rom), result.outcome.word());
                    request.push_str(&result.rom_name.as_deref().map_or("null".to_string(), emusen_dotnet::json::quoted));
                    request.push('}');
                    let _ = library.change_request(&request);
                }
            }
            self.failover_arrived(result);
        }
    }

    fn failover_arrived(&self, result: openemu::Result) {
        let mut p = self.progress();
        if !p.failover_pending.remove(&result.rom) {
            return;
        }
        if result.saved.is_some() {
            p.failover_found += 1;
            if let Some(row) = p.recent.iter_mut().find(|r| r.path == result.rom) {
                row.from_failover = true;
            }
        }
        if p.failover_pending.is_empty() && !p.workers_live && p.phase == Phase::Running {
            let end = if p.why.is_none() { Phase::Done } else { Phase::Stopped };
            self.end_run(&mut p, end);
        } else {
            self.touch(&mut p);
        }
    }

    pub(crate) fn stop_failover(&self) {
        self.runs.failover_stop.stop();
        if let Some(h) = self.runs.failover_thread.lock().unwrap_or_else(|p| p.into_inner()).take() {
            let _ = h.join();
        }
        let mut f = self.runs.failover.lock().unwrap_or_else(|p| p.into_inner());
        f.running = false;
        f.queue.clear();
        f.asked.clear();
    }
}

fn backoff(attempts: i32) -> i64 {
    (1i64 << attempts.min(6)).min(60) * 60 * TICKS_PER_SECOND
}

/// An answer's status in the player's words, with what the server said.
fn reason(answer: &client::JeuInfos) -> String {
    let what = match answer.status {
        Status::BadRequest => "ScreenScraper refused the request".to_string(),
        Status::Malformed => "ScreenScraper's answer could not be read".to_string(),
        Status::Failed => "ScreenScraper could not be reached".to_string(),
        s => format!("ScreenScraper answered {}", s.name()),
    };
    if answer.detail.is_empty() { what } else { format!("{what}: {}", answer.detail) }
}

/// `ScrapeProgress.Count`.
fn count(p: &mut Progress, r: &Finished) {
    match r.outcome {
        Outcome::Found | Outcome::Unknown | Outcome::Error | Outcome::Missing => {
            p.done += 1;
            p.unchanged += r.unchanged;
            if r.skipped {
                p.skipped += 1;
            } else if r.outcome == Outcome::Found {
                p.found += 1;
            } else if r.outcome == Outcome::Unknown {
                p.unknown += 1;
            } else if r.outcome == Outcome::Error {
                p.failed += 1;
                p.last_failure = Some(redact(&r.detail));
            }
            if p.current.as_ref().is_some_and(|c| c.path == r.path) {
                p.current = None;
            }
            let serial = p.serial + 1;
            let kinds = r.written.iter().map(|w| {
                let folder = layout::kind_folder_of(w);
                kinds::ALL.iter().find(|k| k.folder == folder).map(|k| k.esde_type.to_string()).unwrap_or(folder)
            });
            p.recent.push_front(Row {
                serial,
                path: r.path.clone(),
                system: r.system.clone(),
                outcome: r.outcome,
                kinds: kinds.collect(),
                detail: redact(&r.detail),
                skipped: r.skipped,
                has_cover: r.has_cover,
                matched_by: r.matched_by.clone(),
                from_failover: false,
            });
            p.recent.truncate(RECENT_LIMIT);
        }
        Outcome::Retry => p.retrying += 1,
        Outcome::Stopped => {}
    }
}

fn set_paused(p: &mut Progress, paused: bool, now: i64) {
    if paused && p.paused_at.is_none() {
        p.paused_at = Some(now);
    } else if !paused {
        if let Some(since) = p.paused_at.take() {
            p.paused_for += now - since;
        }
    }
}

/// An instant as `ToLocalTime():ddd HH:mm` writes it under the invariant culture.
fn local_day_time(utc: i64) -> String {
    let seconds = utc.div_euclid(TICKS_PER_SECOND);
    let local = utc + crate::changes::local_offset(seconds) * TICKS_PER_SECOND;
    let days = local.div_euclid(emusen_dotnet::date::TICKS_PER_DAY);
    let names = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    format!("{} {}", names[emusen_dotnet::date::weekday(days) as usize], emusen_dotnet::date::local_text(local, "HH:mm"))
}
