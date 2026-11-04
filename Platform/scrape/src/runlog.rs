//! The scrape log: one line a ScreenScraper request and one when a run's workers end, in `scrape_<yyyyMMdd>.log` beside
//! the error log, so that where a run's time went is read off it rather than guessed. Every line passes the redactor and
//! no line carries an address, an account or a key. See EmuSen_LibraryTools.md §9.10.

use crate::answer::Quota;
use crate::redact::redact;
use std::io::Write;
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub const PREFIX: &str = "scrape_";

/// Where a run's lines go, the folder found once at the run's start; with none, nothing is written.
pub struct Log {
    folder: Option<String>,
    gate: Mutex<()>,
}

impl Log {
    /// The error log's folder, as Galaxia resolves it (the `LogDirectory` setting, else its fallback).
    pub fn beside_error_log() -> Log {
        let root = {
            let tree = emusen_galaxia::tree::tree();
            emusen_galaxia::error_log::root(&tree, None).0
        };
        Log { folder: (!root.trim().is_empty()).then_some(root), gate: Mutex::new(()) }
    }

    pub fn none() -> Log {
        Log { folder: None, gate: Mutex::new(()) }
    }

    /// A day's file in a folder, the day as `yyyyMMdd`.
    pub fn path_for(folder: &str, day: &str) -> String {
        emusen_galaxia::dotnet_path::combine(emusen_galaxia::dotnet_path::Style::HOST, folder, &format!("{PREFIX}{day}.log"))
    }

    /// One line, stamped with the local time and redacted; a log that cannot be written is never a fault of the run.
    pub fn write(&self, line: &str) {
        let Some(folder) = &self.folder else { return };
        let (day, stamp) = local_now();
        let text = format!("{stamp}\t{}\n", redact(&line.replace(['\r', '\n'], " ")));
        let _held = self.gate.lock().unwrap_or_else(|p| p.into_inner());
        if std::fs::create_dir_all(folder).is_err() {
            return;
        }
        if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(Log::path_for(folder, &day)) {
            let _ = file.write_all(text.as_bytes());
        }
    }
}

/// The local day as `yyyyMMdd` and the instant as `yyyy-MM-dd HH:mm:ss.fff`, from the system's clock.
fn local_now() -> (String, String) {
    let since = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
    let seconds = since.as_secs() as i64;
    let local = (seconds + emusen_dotnet::date::local_offset(seconds)) * emusen_dotnet::date::TICKS_PER_SECOND;
    let day = emusen_dotnet::date::local_text(local, "yyyyMMdd");
    let stamp = format!("{}.{:03}", emusen_dotnet::date::local_text(local, "yyyy-MM-dd HH:mm:ss"), since.subsec_millis());
    (day, stamp)
}

fn ms(d: Duration) -> u128 {
    d.as_millis()
}

fn or_dash(v: Option<i32>) -> String {
    v.map_or_else(|| "-".to_string(), |n| n.to_string())
}

/// One request as the log writes it.
pub struct Request<'a> {
    pub game: &'a str,
    pub kind: &'a str,
    pub code: u16,
    pub outcome: &'a str,
    pub bytes: u64,
    pub took: Duration,
    pub worker: usize,
    pub live: i32,
    pub allowed: i32,
    pub pace: Duration,
    pub cap: Duration,
    pub hash: Option<Duration>,
    pub quota: Option<Quota>,
}

/// A request's line: tab-separated `name=value` fields, `-` for what the answer did not say.
pub fn request_line(r: &Request) -> String {
    let q = r.quota;
    format!(
        "request\tgame={}\tkind={}\tstatus={}\toutcome={}\tbytes={}\tms={}\tworker={}\tthreads={}/{}\tpace_ms={}\tcap_ms={}\thash_ms={}\tmaxthreads={}\tmaxrequestspermin={}\tmaxdownloadspeed={}\trequeststoday={}",
        r.game,
        r.kind,
        r.code,
        r.outcome,
        r.bytes,
        ms(r.took),
        r.worker,
        r.live,
        r.allowed,
        ms(r.pace),
        ms(r.cap),
        r.hash.map_or_else(|| "-".to_string(), |h| ms(h).to_string()),
        or_dash(q.and_then(|q| q.max_threads)),
        or_dash(q.and_then(|q| q.max_requests_per_minute)),
        or_dash(q.and_then(|q| q.max_download_kbps)),
        or_dash(q.and_then(|q| q.requests_today)),
    )
}

/// Where one worker's time went, or a run's, summed over its workers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Tally {
    pub games: u64,
    pub requests: u64,
    pub bytes: u64,
    pub workers: u64,
    pub worker: Duration,
    pub pace: Duration,
    pub cap: Duration,
    pub server: Duration,
    pub hash: Duration,
    pub idle: Duration,
}

impl Tally {
    pub fn add(&mut self, o: &Tally) {
        self.games += o.games;
        self.requests += o.requests;
        self.bytes += o.bytes;
        self.workers += o.workers;
        self.worker += o.worker;
        self.pace += o.pace;
        self.cap += o.cap;
        self.server += o.server;
        self.hash += o.hash;
        self.idle += o.idle;
    }

    /// The workers' time not spent waiting, asking or hashing: the run's own work between requests.
    pub fn own(&self) -> Duration {
        self.worker.saturating_sub(self.pace + self.cap + self.server + self.hash + self.idle)
    }
}

/// The run's last line: its counts, its wall time, and the workers' time by where it went.
pub fn summary_line(end: &str, t: &Tally, wall: Duration) -> String {
    format!(
        "run\tend={end}\tgames={}\trequests={}\tbytes={}\twall_ms={}\tworkers={}\tworker_ms={}\tpace_ms={}\tcap_ms={}\tserver_ms={}\thash_ms={}\tidle_ms={}\town_ms={}",
        t.games,
        t.requests,
        t.bytes,
        ms(wall),
        t.workers,
        ms(t.worker),
        ms(t.pace),
        ms(t.cap),
        ms(t.server),
        ms(t.hash),
        ms(t.idle),
        ms(t.own()),
    )
}

/// One worker's account as it goes: its tally, and the waits and hashing owed to the line of its next request.
pub struct Lane {
    pub index: usize,
    started: Instant,
    pub tally: Tally,
    pub pace: Duration,
    pub cap: Duration,
    pub hash: Option<Duration>,
}

impl Lane {
    pub fn new(index: usize) -> Lane {
        Lane { index, started: Instant::now(), tally: Tally { workers: 1, ..Tally::default() }, pace: Duration::ZERO, cap: Duration::ZERO, hash: None }
    }

    pub fn hashed(&mut self, took: Duration) {
        self.tally.hash += took;
        self.hash = Some(self.hash.unwrap_or_default() + took);
    }

    /// A request answered: counted, and what it owed handed to its line and cleared.
    pub fn sent(&mut self, bytes: u64, took: Duration) -> (Duration, Duration, Option<Duration>) {
        self.tally.requests += 1;
        self.tally.bytes += bytes;
        self.tally.server += took;
        let owed = (self.pace, self.cap, self.hash);
        (self.pace, self.cap, self.hash) = (Duration::ZERO, Duration::ZERO, None);
        owed
    }

    /// The worker's whole account, at its end.
    pub fn ended(mut self) -> Tally {
        self.tally.worker = self.started.elapsed();
        self.tally
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_request_line_names_every_field_and_dashes_what_the_answer_left_out() {
        let r = Request {
            game: "A Game (USA).iso",
            kind: "info",
            code: 200,
            outcome: "found",
            bytes: 2048,
            took: Duration::from_millis(321),
            worker: 2,
            live: 3,
            allowed: 4,
            pace: Duration::from_millis(1500),
            cap: Duration::ZERO,
            hash: Some(Duration::from_millis(40)),
            quota: Some(Quota { max_threads: Some(4), max_download_kbps: Some(512), requests_today: Some(17), requests_ko_today: None, max_requests_per_minute: Some(160), max_requests_per_day: None, max_requests_ko_per_day: None }),
        };
        assert_eq!(
            request_line(&r),
            "request\tgame=A Game (USA).iso\tkind=info\tstatus=200\toutcome=found\tbytes=2048\tms=321\tworker=2\tthreads=3/4\tpace_ms=1500\tcap_ms=0\thash_ms=40\tmaxthreads=4\tmaxrequestspermin=160\tmaxdownloadspeed=512\trequeststoday=17"
        );
        let media = request_line(&Request { kind: "cover", hash: None, quota: None, ..r });
        assert!(media.ends_with("hash_ms=-\tmaxthreads=-\tmaxrequestspermin=-\tmaxdownloadspeed=-\trequeststoday=-"), "{media}");
    }

    #[test]
    fn a_lane_hands_its_waits_to_the_next_line_once_and_its_own_time_is_what_is_left() {
        let mut lane = Lane::new(0);
        lane.pace += Duration::from_millis(10);
        lane.tally.pace += Duration::from_millis(10);
        lane.hashed(Duration::from_millis(5));
        assert_eq!(lane.sent(100, Duration::from_millis(30)), (Duration::from_millis(10), Duration::ZERO, Some(Duration::from_millis(5))));
        assert_eq!(lane.sent(50, Duration::from_millis(20)), (Duration::ZERO, Duration::ZERO, None));
        let mut t = lane.ended();
        t.worker = Duration::from_millis(100);
        assert_eq!((t.requests, t.bytes, t.own()), (2, 150, Duration::from_millis(35)));
        assert!(summary_line("done", &t, Duration::from_millis(120)).starts_with("run\tend=done\tgames=0\trequests=2\tbytes=150\twall_ms=120\tworkers=1\tworker_ms=100\tpace_ms=10\t"));
    }
}
