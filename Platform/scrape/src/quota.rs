//! §5.5's rules, `ScrapeQuotaManager` ported: limits read from every answer, never more workers than `maxthreads`, a
//! pace of `maxrequestspermin` less 10%, a stop at the day's limit less 2% or on 430 or 431 until midnight in Paris, and
//! the back-offs; a 403, 423 or 426 stops until the settings change (EmuSen_Backend.md §30.1, as fixed). See
//! EmuSen_BigPicture.md §5.5 and §17.4.

use crate::Error;
use crate::answer::Quota;
use crate::clock::{Clock, Stop};
use crate::store::{Day, Store};
use crate::zone::Zone;
use emusen_dotnet::date::{TICKS_PER_DAY, TICKS_PER_SECOND, local_text, ticks_from_seconds};
use std::sync::{Mutex, OnceLock};

/// Assumed until an answer says otherwise: 30 requests a minute.
pub const ASSUMED_PER_MINUTE: i32 = 30;
pub const BUSY_WAIT: i64 = 5 * 60 * TICKS_PER_SECOND;
pub const TOO_MANY_WAIT: i64 = 60 * TICKS_PER_SECOND;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Found,
    NotFound,
    BadRequest,
    ServerBusy,
    BadCredentials,
    ApiClosed,
    Blacklisted,
    TooManyRequests,
    DailyQuota,
    DailyKoQuota,
    Malformed,
    Failed,
}

impl Status {
    pub fn of(code: u16) -> Status {
        match code {
            200 => Status::Found,
            400 => Status::BadRequest,
            401 => Status::ServerBusy,
            403 => Status::BadCredentials,
            404 => Status::NotFound,
            423 => Status::ApiClosed,
            426 => Status::Blacklisted,
            429 => Status::TooManyRequests,
            430 => Status::DailyQuota,
            431 => Status::DailyKoQuota,
            _ => Status::Failed,
        }
    }

    /// The C# enum member's name, as `$"ScreenScraper answered {answer.Status}"` writes it.
    pub fn name(self) -> &'static str {
        match self {
            Status::Found => "Found",
            Status::NotFound => "NotFound",
            Status::BadRequest => "BadRequest",
            Status::ServerBusy => "ServerBusy",
            Status::BadCredentials => "BadCredentials",
            Status::ApiClosed => "ApiClosed",
            Status::Blacklisted => "Blacklisted",
            Status::TooManyRequests => "TooManyRequests",
            Status::DailyQuota => "DailyQuota",
            Status::DailyKoQuota => "DailyKoQuota",
            Status::Malformed => "Malformed",
            Status::Failed => "Failed",
        }
    }
}

/// One look at a turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Turn {
    Go,
    Wait(i64),
    Stopped,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gate {
    Go,
    Wait,
    Stopped,
}

impl Gate {
    pub fn word(self) -> &'static str {
        match self {
            Gate::Go => "go",
            Gate::Wait => "wait",
            Gate::Stopped => "stopped",
        }
    }
}

fn paris() -> &'static Zone {
    static PARIS: OnceLock<Zone> = OnceLock::new();
    PARIS.get_or_init(|| Zone::system("Europe/Paris"))
}

/// ScreenScraper's day, taken as Paris's.
pub fn day_of(t: i64) -> String {
    local_text(paris().local(t), "yyyy-MM-dd")
}

pub fn start_of_next_day(t: i64) -> i64 {
    let local = paris().local(t);
    let midnight = (local.div_euclid(TICKS_PER_DAY) + 1) * TICKS_PER_DAY;
    midnight - paris().offset_of_local(midnight) * TICKS_PER_SECOND
}

struct Inner {
    limits: Option<Quota>,
    slowdown: f64,
    next_slot: i64,
    wait_until: Option<i64>,
    wait_reason: Option<String>,
    session_stop: Option<String>,
    day: Day,
    from_server: bool,
}

/// What the quota allows now, its limits, and today's counts.
#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    pub day: String,
    pub requests: i32,
    pub max_per_day: Option<i32>,
    pub ko: i32,
    pub max_ko_per_day: Option<i32>,
    pub threads: i32,
    pub per_minute: Option<i32>,
    pub interval: i64,
    pub gate: Gate,
    pub until: Option<i64>,
    pub reason: Option<String>,
    pub from_server: bool,
    pub max_kbps: Option<i32>,
}

pub struct Manager {
    inner: Mutex<Inner>,
}

impl Manager {
    pub fn new(store: &Store, clock: &Clock) -> Result<Manager, Error> {
        let day = store.day(&day_of(clock.now()))?;
        Ok(Manager {
            inner: Mutex::new(Inner { limits: None, slowdown: 1.0, next_slot: i64::MIN, wait_until: None, wait_reason: None, session_stop: None, day, from_server: false }),
        })
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|p| p.into_inner())
    }

    pub fn limits(&self) -> Option<Quota> {
        self.lock().limits
    }

    /// Never more than the member's `maxthreads`, and one until an answer has said.
    pub fn threads(&self, wanted: i32) -> i32 {
        let max = self.lock().limits.and_then(|l| l.max_threads).unwrap_or(1);
        wanted.max(1).min(max).max(1)
    }

    fn interval_of(inner: &Inner) -> i64 {
        let per_minute = inner.limits.and_then(|l| l.max_requests_per_minute).filter(|&m| m > 0).unwrap_or(ASSUMED_PER_MINUTE);
        ticks_from_seconds(60.0 / (per_minute as f64 * 0.9) * inner.slowdown)
    }

    pub fn interval(&self) -> i64 {
        Manager::interval_of(&self.lock())
    }

    fn roll_day(inner: &mut Inner, store: &Store, now: i64) -> Result<(), Error> {
        let today = day_of(now);
        if inner.day.day == today {
            return Ok(());
        }
        let next = store.day(&today)?;
        let (max, max_ko) = (next.max_per_day.or(inner.day.max_per_day), next.max_ko_per_day.or(inner.day.max_ko_per_day));
        inner.day = Day { max_per_day: max, max_ko_per_day: max_ko, ..next };
        Ok(())
    }

    fn over_daily_limit(day: &Day) -> Option<String> {
        if let Some(max) = day.max_per_day.filter(|&m| m > 0) {
            if day.requests as f64 >= (max as f64 * 0.98).floor() {
                return Some(format!("today's requests are nearly used up ({} of {max})", day.requests));
            }
        }
        if let Some(max) = day.max_ko_per_day.filter(|&m| m > 0) {
            if day.ko as f64 >= (max as f64 * 0.98).floor() {
                return Some(format!("today's allowance of unrecognised games is nearly used up ({} of {max})", day.ko));
            }
        }
        None
    }

    fn stop_for_the_day(inner: &mut Inner, store: &Store, until: i64, why: &str) -> Result<(), Error> {
        inner.day.stopped_until = Some(until);
        inner.day.stop_reason = Some(why.to_string());
        store.save_day(&inner.day)
    }

    /// What the quota allows at this moment, without waiting: the gate, until when, and why.
    pub fn check(&self, store: &Store, clock: &Clock) -> Result<(Gate, Option<i64>, Option<String>), Error> {
        let mut inner = self.lock();
        let now = clock.now();
        Manager::roll_day(&mut inner, store, now)?;
        if let Some(stop) = &inner.session_stop {
            return Ok((Gate::Stopped, None, Some(stop.clone())));
        }
        if let Some(stopped) = inner.day.stopped_until.filter(|&s| now < s) {
            return Ok((Gate::Stopped, Some(stopped), inner.day.stop_reason.clone()));
        }
        if let Some(why) = Manager::over_daily_limit(&inner.day) {
            let until = start_of_next_day(now);
            Manager::stop_for_the_day(&mut inner, store, until, &why)?;
            return Ok((Gate::Stopped, Some(until), Some(why)));
        }
        if let Some(w) = inner.wait_until.filter(|&w| now < w) {
            return Ok((Gate::Wait, Some(w), inner.wait_reason.clone()));
        }
        Ok((Gate::Go, None, None))
    }

    /// One look at the next request's turn: go now (the slot taken), wait so long, or stopped.
    pub fn try_turn(&self, store: &Store, clock: &Clock) -> Result<Turn, Error> {
        let (gate, until, _) = self.check(store, clock)?;
        if gate == Gate::Stopped {
            return Ok(Turn::Stopped);
        }
        let mut inner = self.lock();
        let now = clock.now();
        let mut ready = if gate == Gate::Wait { until.unwrap_or(now) } else { now };
        if inner.next_slot > ready {
            ready = inner.next_slot;
        }
        let wait = ready - now;
        if wait <= 0 {
            inner.next_slot = now + Manager::interval_of(&inner);
            return Ok(Turn::Go);
        }
        Ok(Turn::Wait(wait))
    }

    /// Waits for the next request's turn, the store taken only to look: Some(true) to go, Some(false) stopped, None
    /// when the run was.
    pub fn take_turn_with<'a>(&self, store: &dyn Fn() -> std::sync::MutexGuard<'a, Store>, clock: &Clock, stop: &Stop) -> Result<Option<bool>, Error> {
        loop {
            if stop.is_stopped() {
                return Ok(None);
            }
            let step = {
                let guard = store();
                self.try_turn(&guard, clock)?
            };
            match step {
                Turn::Go => return Ok(Some(true)),
                Turn::Stopped => return Ok(Some(false)),
                Turn::Wait(wait) => {
                    if !clock.delay(wait, stop) {
                        return Ok(None);
                    }
                }
            }
        }
    }

    /// As [`Manager::take_turn_with`], over a store the caller holds alone.
    pub fn take_turn(&self, store: &Store, clock: &Clock, stop: &Stop) -> Result<Option<bool>, Error> {
        loop {
            if stop.is_stopped() {
                return Ok(None);
            }
            match self.try_turn(store, clock)? {
                Turn::Go => return Ok(Some(true)),
                Turn::Stopped => return Ok(Some(false)),
                Turn::Wait(wait) => {
                    if !clock.delay(wait, stop) {
                        return Ok(None);
                    }
                }
            }
        }
    }

    /// The limits and the server's counts, from any answer that carried them.
    pub fn observe(&self, store: &Store, clock: &Clock, quota: Option<Quota>) -> Result<bool, Error> {
        let Some(quota) = quota else { return Ok(false) };
        let mut inner = self.lock();
        inner.limits = Some(quota);
        Manager::roll_day(&mut inner, store, clock.now())?;
        let d = &mut inner.day;
        d.requests = quota.requests_today.unwrap_or(d.requests);
        d.ko = quota.requests_ko_today.unwrap_or(d.ko);
        d.max_per_day = quota.max_requests_per_day.or(d.max_per_day);
        d.max_ko_per_day = quota.max_requests_ko_per_day.or(d.max_ko_per_day);
        store.save_day(&inner.day)?;
        inner.from_server = true;
        Ok(true)
    }

    /// Counted here as well, for a day whose answers carry no counts.
    pub fn counted(&self, store: &Store, clock: &Clock, unrecognised: bool) -> Result<(), Error> {
        let mut inner = self.lock();
        Manager::roll_day(&mut inner, store, clock.now())?;
        inner.day.requests += 1;
        if unrecognised {
            inner.day.ko += 1;
        }
        store.save_day(&inner.day)
    }

    /// What an answer's status does to the quota; true when it changed anything.
    pub fn answered(&self, store: &Store, clock: &Clock, status: Status) -> Result<bool, Error> {
        let mut inner = self.lock();
        let now = clock.now();
        match status {
            Status::TooManyRequests => {
                inner.slowdown *= 2.0;
                inner.wait_until = Some(now + TOO_MANY_WAIT);
                inner.wait_reason = Some("ScreenScraper asked for fewer requests (429); the pace is halved".into());
            }
            Status::ServerBusy => {
                inner.wait_until = Some(now + BUSY_WAIT);
                inner.wait_reason = Some("ScreenScraper is closed to non-members while it is busy (401)".into());
            }
            Status::DailyQuota => Manager::stop_for_the_day(&mut inner, store, start_of_next_day(now), "today's requests are used up (430)")?,
            Status::DailyKoQuota => Manager::stop_for_the_day(&mut inner, store, start_of_next_day(now), "today's allowance of unrecognised games is used up (431)")?,
            Status::BadCredentials => inner.session_stop = Some("ScreenScraper refused EmuSen's developer credentials (403)".into()),
            Status::ApiClosed => inner.session_stop = Some("ScreenScraper's API is closed (423)".into()),
            Status::Blacklisted => inner.session_stop = Some("ScreenScraper has blocked this version of the software (426); a newer build is needed".into()),
            _ => return Ok(false),
        }
        Ok(true)
    }

    /// The settings changed: a stop for the credentials or the API is lifted, and nothing else.
    pub fn settings_changed(&self) -> bool {
        self.lock().session_stop.take().is_some()
    }

    pub fn snapshot(&self, store: &Store, clock: &Clock) -> Result<Snapshot, Error> {
        let (gate, until, reason) = self.check(store, clock)?;
        let inner = self.lock();
        Ok(Snapshot {
            day: inner.day.day.clone(),
            requests: inner.day.requests,
            max_per_day: inner.day.max_per_day,
            ko: inner.day.ko,
            max_ko_per_day: inner.day.max_ko_per_day,
            threads: inner.limits.and_then(|l| l.max_threads).unwrap_or(1),
            per_minute: inner.limits.and_then(|l| l.max_requests_per_minute),
            interval: Manager::interval_of(&inner),
            gate,
            until,
            reason,
            from_server: inner.from_server,
            max_kbps: inner.limits.and_then(|l| l.max_download_kbps),
        })
    }
}
