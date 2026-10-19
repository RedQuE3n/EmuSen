//! The run's calls and the reads and changes that ask the server, as `emusen_scrape.h` describes them: `start`,
//! `progress`, the plan, the name search, signing in and out, OpenVGDB's download and removal, and a picture's source.
//! See EmuSen_Backend.md §30.3.

use crate::client::{self, Client, Wire};
use crate::clock::Stop;
use crate::json::{self, number_or_null, quoted, text_or_null};
use crate::kinds;
use crate::quota::Status;
use crate::redact::redact;
use crate::run::{Choices, Phase};
use crate::store::Priority;
use crate::{Error, Scraper, credentials, openemu};
use emusen_dotnet::date::TICKS_PER_SECOND;
use emusen_dotnet::json::Value;

/// `ScrapeCost.Kinds`: the share of §17.9's 39 found answers offering each kind, and the mean size the region rule took.
const COST: [(&str, f64, f64); 11] = [
    ("cover", 33.0 / 39.0, 653.0),
    ("screenshot", 36.0 / 39.0, 5.0),
    ("marquee", 36.0 / 39.0, 90.0),
    ("titlescreen", 36.0 / 39.0, 6.0),
    ("miximage", 39.0 / 39.0, 565.0),
    ("backcover", 33.0 / 39.0, 468.0),
    ("3dbox", 33.0 / 39.0, 351.0),
    ("physicalmedia", 33.0 / 39.0, 449.0),
    ("fanart", 25.0 / 39.0, 397.0),
    ("manual", 27.0 / 39.0, 2254.0),
    ("video", 32.0 / 39.0, 1354.0),
];

fn cost(ty: &str) -> Option<(f64, f64)> {
    COST.iter().find(|(t, _, _)| *t == ty).map(|(_, o, k)| (*o, *k))
}

fn expected_requests(kinds: &[kinds::Kind]) -> f64 {
    1.0 + kinds.iter().map(|k| cost(k.esde_type).map_or(1.0, |c| c.0)).sum::<f64>()
}

fn expected_kb(kinds: &[kinds::Kind]) -> f64 {
    kinds.iter().map(|k| cost(k.esde_type).map_or(0.0, |c| c.0 * c.1)).sum()
}

fn kinds_of(r: &Value) -> Result<Vec<kinds::Kind>, Error> {
    let names = json::strings(r, "kinds")?;
    names.iter().map(|n| kinds::by_type(n).ok_or_else(|| Error::BadRequest(format!("No kind is called {n}.")))).collect()
}

pub(crate) fn choices(r: Option<&Value>, refresh: bool) -> Result<Choices, Error> {
    let empty = Value::Object(Vec::new());
    let c = r.filter(|v| !v.is_null()).unwrap_or(&empty);
    let kinds = if c.get("kinds").is_some() { kinds_of(c)? } else { vec![kinds::COVER, kinds::SCREENSHOT, kinds::MARQUEE, kinds::MIXIMAGE] };
    Ok(Choices {
        kinds,
        region: json::text_or_none(c, "region")?.unwrap_or_else(|| "auto".into()),
        language: json::text_or_none(c, "language")?.unwrap_or_else(|| "en".into()),
        region_fallback: json::flag(c, "region_fallback", true)?,
        threads: json::integer_or_none(c, "threads")?.unwrap_or(1) as i32,
        refresh,
    })
}

impl Scraper {
    /// `emusen_scrape_start`'s request: the games queued, or a status.
    pub fn start_request(&self, request: &str) -> Result<usize, Error> {
        let r = json::parse(request)?;
        let games = crate::reads::games(&r)?;
        let priority = match json::text_or_none(&r, "priority")? {
            Some(p) => Priority::named(&p).ok_or_else(|| Error::BadRequest(format!("No priority is called {p}.")))?,
            None => Priority::Library,
        };
        let refresh = json::flag(&r, "refresh", false)?;
        let resume = json::flag(&r, "resume", false)?;
        let choices = choices(r.get("choices"), refresh && !resume)?;
        self.start(&games, priority, json::flag(&r, "one", false)?, resume, json::integer_or_none(&r, "pick")?, json::flag(&r, "enabled", true)?, choices)
    }

    /// `emusen_scrape_progress`'s answer, the games finished after `since` among its results.
    pub fn progress_request(&self, request: &str) -> Result<String, Error> {
        let since = if request.trim().is_empty() { 0 } else { json::integer_or_none(&json::parse(request)?, "since")?.unwrap_or(0) as u64 };
        let mut out = String::new();
        {
            let p = self.progress();
            let now = self.clock.now();
            out.push_str(&format!("{{\"serial\":{},\"state\":\"{}\",\"paused\":{},\"why\":", p.serial, p.phase.word(), p.paused_at.is_some()));
            text_or_null(&mut out, p.why.as_deref());
            out.push_str(&format!(
                ",\"total\":{},\"done\":{},\"found\":{},\"unknown\":{},\"failed\":{},\"skipped\":{},\"retrying\":{},\"unchanged\":{},\"requests\":{},\"last_failure\":",
                p.total, p.done, p.found, p.unknown, p.failed, p.skipped, p.retrying, p.unchanged, p.requests
            ));
            text_or_null(&mut out, p.last_failure.as_deref());
            out.push_str(&format!(",\"failover\":{{\"asked\":{},\"found\":{},\"pending\":{}}},\"current\":", p.failover_asked, p.failover_found, p.failover_pending.len()));
            match &p.current {
                Some(a) => {
                    out.push_str(&format!("{{\"path\":{},\"system\":{},\"step\":\"{}\",\"kind\":", quoted(&a.path), quoted(&a.system), a.step));
                    text_or_null(&mut out, a.kind.as_deref());
                    out.push_str(",\"saved\":");
                    text_or_null(&mut out, a.saved.as_deref());
                    out.push('}');
                }
                None => out.push_str("null"),
            }
            out.push_str(",\"last_picture\":");
            text_or_null(&mut out, p.last_picture.as_deref());
            out.push_str(",\"started\":");
            text_or_null(&mut out, (p.phase != Phase::Idle).then(|| crate::store::stamp(p.started)).as_deref());
            out.push_str(",\"ended\":");
            text_or_null(&mut out, p.ended.map(crate::store::stamp).as_deref());
            let working = (p.ended.unwrap_or(now) - p.started - p.paused_for - p.paused_at.map_or(0, |since| p.ended.unwrap_or(now) - since)).max(0);
            out.push_str(&format!(",\"paused_ticks\":{},\"working_ticks\":{},\"workers\":{},\"interrupted\":", p.paused_for, if p.phase == Phase::Idle { 0 } else { working }, p.workers_live));
            let interrupted = if p.phase == Phase::Running { 0 } else { self.store().queue_length()? };
            out.push_str(&interrupted.to_string());
            out.push_str(",\"results\":[");
            let mut first = true;
            for row in p.recent.iter().filter(|row| row.serial > since) {
                if !first {
                    out.push(',');
                }
                first = false;
                out.push_str(&format!("{{\"serial\":{},\"path\":{},\"system\":{},\"outcome\":\"{}\",\"kinds\":", row.serial, quoted(&row.path), quoted(&row.system), row.outcome.word()));
                json::strings_into(&mut out, &row.kinds);
                out.push_str(&format!(",\"detail\":{},\"skipped\":{},\"has_cover\":{},\"matched_by\":", quoted(&row.detail), row.skipped, row.has_cover));
                text_or_null(&mut out, row.matched_by.as_deref());
                out.push_str(&format!(",\"from_failover\":{}}}", row.from_failover));
            }
            out.push_str("],\"quota\":");
        }
        crate::reads::quota_json(self, &mut out)?;
        out.push('}');
        Ok(out)
    }

    /// Waits until the run's serial passes `serial` or the timeout ends; the serial then.
    pub fn wait(&self, serial: u64, timeout_ms: u32) -> u64 {
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms as u64);
        let mut p = self.progress();
        while p.serial <= serial {
            let left = deadline.saturating_duration_since(std::time::Instant::now());
            if left.is_zero() {
                break;
            }
            p = self.runs.changed.wait_timeout(p, left).unwrap_or_else(|q| q.into_inner()).0;
        }
        p.serial
    }

    /// `ScrapePlan`'s numbers for a scope, before anything is started.
    pub(crate) fn plan(&self, r: &Value) -> Result<String, Error> {
        let paths = json::strings(r, "games")?;
        let one = json::flag(r, "one", false)?;
        let refresh = json::flag(r, "refresh", false)?;
        let kinds = kinds_of(r)?;
        let why = self.unusable(json::flag(r, "enabled", true)?)?;
        let (outcomes, found) = {
            let store = self.store();
            (store.outcomes_by_path()?, store.found_by_path()?)
        };
        let not_asked = if one { paths.len() } else { paths.iter().filter(|p| !outcomes.iter().any(|(o, _, _)| o == *p)).count() };
        let (mut again, mut again_kb, mut again_requests) = (0usize, 0.0f64, 0.0f64);
        if !one {
            for g in &paths {
                let Some((_, rec)) = found.iter().find(|(p, _)| p == g) else { continue };
                let stored = self.store().media(&rec.md5, rec.bytes)?;
                let lacking: Vec<kinds::Kind> = kinds.iter().copied().filter(|k| rec.may_offer(k.esde_type) && !stored.iter().any(|s| s.ty == k.esde_type)).collect();
                if !refresh && lacking.is_empty() {
                    continue;
                }
                again += 1;
                again_kb += expected_kb(&lacking);
                again_requests += expected_requests(&lacking);
            }
        }
        let q = {
            let store = self.store();
            self.quota.snapshot(&store, &self.clock)?
        };
        let left = q.max_per_day.map(|m| (m - q.requests).max(0));
        let kb = not_asked as f64 * expected_kb(&kinds) + again_kb;
        let kbps = q.max_kbps;
        let asking = (not_asked + again) as i64;
        let time = asking * emusen_dotnet::date::ticks_from_seconds(3.9) + emusen_dotnet::date::ticks_from_seconds(kb / kbps.unwrap_or(128).max(1) as f64);
        let mut out = format!(
            "{{\"games\":{},\"not_yet_asked\":{not_asked},\"asked_again\":{again},\"max_requests\":{},\"requests_left_today\":",
            paths.len(),
            asking * (1 + kinds.len() as i64)
        );
        number_or_null(&mut out, left.map(i64::from));
        out.push_str(&format!(",\"time_ticks\":{time},\"usable\":{},\"why_not\":", why.is_none()));
        text_or_null(&mut out, why.as_deref());
        let expected = not_asked as f64 * expected_requests(&kinds) + again_requests;
        out.push_str(&format!(",\"expected_requests\":{},\"expected_mb\":{},\"refresh\":{refresh}}}", fmt(expected), fmt(kb / 1024.0)));
        Ok(out)
    }

    fn client(&self, member: Option<credentials::Member>) -> Option<Client> {
        let developer = self.developer()?;
        Some(Client { wire: Wire::of(&self.settings()), developer, member: std::sync::RwLock::new(member) })
    }

    /// Log In or Check: one `ssuserInfos` with the account; kept, with the time it was accepted, only when accepted.
    pub(crate) fn sign_in(&self, member: credentials::Member) -> Result<String, Error> {
        let (result, found, said) = if !member.is_set() {
            ("incomplete", None, String::new())
        } else {
            match self.client(Some(member.clone())) {
                None => ("no_developer", None, String::new()),
                Some(client) => {
                    let (status, body) = client.user_infos(&emusen_net::Progress::default());
                    client::sign_in_result(status, &body)
                }
            }
        };
        if let Some(m) = &found {
            let store = self.store();
            let _ = self.quota.observe(&store, &self.clock, Some(m.quota));
        }
        if result == "signed_in" {
            let mut kept = member.clone();
            kept.verified = Some(self.clock.now());
            kept.save().map_err(|e| Error::Io(emusen_dotnet::file::io_message(&e, &credentials::Member::path())))?;
            self.set_run_member(Some(kept));
        }
        let mut out = format!("{{\"result\":\"{result}\",\"name\":");
        text_or_null(&mut out, found.as_ref().map(|m| m.name.as_str()));
        out.push_str(",\"level\":");
        text_or_null(&mut out, found.as_ref().and_then(|m| m.level.as_deref()));
        out.push_str(",\"quota\":");
        match found.as_ref().map(|m| m.quota) {
            Some(q) => {
                out.push('[');
                for (i, v) in [q.max_threads, q.max_download_kbps, q.requests_today, q.requests_ko_today, q.max_requests_per_minute, q.max_requests_per_day, q.max_requests_ko_per_day].iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    number_or_null(&mut out, v.map(i64::from));
                }
                out.push(']');
            }
            None => out.push_str("null"),
        }
        out.push_str(&format!(",\"said\":{}}}", quoted(&said)));
        Ok(out)
    }

    /// One `jeuRecherche` for the text the player typed, paced and counted as every request is.
    pub(crate) fn search(&self, r: &Value) -> Result<String, Error> {
        let rom = json::text(r, "path")?;
        let system = json::text(r, "system")?;
        let name = json::text(r, "text")?.trim_matches(char::is_whitespace).to_string();
        let answer = |status: &str, detail: &str| format!("{{\"status\":\"{status}\",\"games\":[],\"detail\":{}}}", quoted(detail));
        let _ = rom;
        if name.is_empty() {
            return Ok(answer("bad_request", "Type a name to search for."));
        }
        if let Some(why) = self.unusable(json::flag(r, "enabled", true)?)? {
            return Ok(answer("failed", &format!("ScreenScraper cannot be used: {why}.")));
        }
        let Some(system_id) = Scraper::screenscraper_id(&system) else { return Ok(answer("bad_request", "This game's console has no ScreenScraper system.")) };
        let Some(client) = self.client(Some(credentials::Member::load()).filter(|m| m.is_set())) else { return Ok(answer("failed", credentials::NONE_HERE)) };
        let stop = Stop::default();
        if self.quota.take_turn_with(&|| self.store(), &self.clock, &stop)? != Some(true) {
            let why = self.unusable(true)?.unwrap_or_else(|| "the quota stopped it".into());
            return Ok(answer("daily_quota", &why));
        }
        let found = client.search(system_id, &name, &emusen_net::Progress::default());
        {
            let store = self.store();
            let _ = self.quota.observe(&store, &self.clock, found.quota);
            if matches!(found.status, Status::Found | Status::NotFound | Status::Malformed) {
                let _ = self.quota.counted(&store, &self.clock, found.status == Status::NotFound);
            }
            let _ = self.quota.answered(&store, &self.clock, found.status);
        }
        let mut picks = self.runs.picks.lock().unwrap_or_else(|p| p.into_inner());
        picks.clear();
        let mut out = format!("{{\"status\":\"{}\",\"games\":[", found.status.name());
        for (i, g) in found.games.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            picks.insert(i as i64, g.clone());
            let pairs = |items: &[crate::answer::Localised]| -> String {
                format!("[{}]", items.iter().map(|l| format!("[{},{}]", quoted(&l.key), quoted(&l.text))).collect::<Vec<_>>().join(","))
            };
            out.push_str(&format!("{{\"pick\":{i},\"id\":{},\"names\":{},\"dates\":{},\"publisher\":", g.id, pairs(&g.names), pairs(&g.dates)));
            text_or_null(&mut out, g.publisher.as_deref());
            out.push_str(",\"developer\":");
            text_or_null(&mut out, g.developer.as_deref());
            out.push_str(",\"system_id\":");
            number_or_null(&mut out, g.system_id.map(i64::from));
            out.push_str(",\"kinds\":");
            let offered: Vec<&str> = kinds::ALL.iter().filter(|k| crate::rules::choose_media(&g.media, k, &["wor".into()], true).is_some()).map(|k| k.esde_type).collect();
            json::strings_into(&mut out, offered);
            out.push('}');
        }
        out.push_str(&format!("],\"detail\":{}}}", quoted(&redact(&found.detail))));
        Ok(out)
    }

    /// OpenVGDB's newest release fetched now, the player's Download; the tag, or why not.
    pub(crate) fn openvgdb_download(&self) -> Result<String, Error> {
        if self.is_running() || self.runs.failover.lock().unwrap_or_else(|p| p.into_inner()).downloading {
            return Err(Error::Busy("A run is using OpenEmu's fallback; download when it has finished.".into()));
        }
        match openemu::download(self.sqlite.clone(), &Wire::of(&self.settings()), &openemu::default_path(), &emusen_net::Progress::default()) {
            Ok(tag) => Ok(format!("{{\"tag\":{}}}", quoted(&tag))),
            Err(words) => Ok(format!("{{\"failed\":{}}}", quoted(&redact(&words)))),
        }
    }

    /// The player's Remove: the database deleted, its worker let go of first; the bytes freed.
    pub(crate) fn openvgdb_remove(&self) -> Result<String, Error> {
        if self.is_running() || self.runs.failover.lock().unwrap_or_else(|p| p.into_inner()).downloading {
            return Err(Error::Busy("OpenVGDB is in use by a run; remove it when the run has finished.".into()));
        }
        self.stop_failover();
        let at = openemu::default_path();
        let bytes = std::fs::metadata(&at).map(|m| m.len()).ok();
        if bytes.is_some() {
            std::fs::remove_file(&at).map_err(|e| Error::Io(emusen_dotnet::file::io_message(&e, &at)))?;
        }
        let _ = std::fs::remove_file(format!("{at}.part"));
        let mut out = String::from("{\"bytes\":");
        number_or_null(&mut out, bytes.map(|b| b as i64));
        out.push('}');
        Ok(out)
    }
}

/// A double as .NET's shortest round-trip text writes it.
fn fmt(d: f64) -> String {
    emusen_dotnet::json::format_f64(d).unwrap_or_else(|| "0".into())
}

pub(crate) const _TICKS: i64 = TICKS_PER_SECOND;
