//! §5.5's rules one by one on a clock the test moves: `ScrapeQuotaTests`' cases, held by the Rust half too. The two
//! halves' paces are compared directly by part 4's replay (EmuSen_Backend.md §30.7).

use emusen_dotnet::date::{TICKS_PER_SECOND, instant};
use emusen_scrape::answer::Quota;
use emusen_scrape::clock::{Clock, Stop};
use emusen_scrape::quota::{Gate, Manager, Status, day_of, start_of_next_day};
use emusen_scrape::store::Store;
use emusen_sqlite::Sqlite;
use std::path::PathBuf;
use std::sync::Arc;

fn sqlite() -> Option<Arc<Sqlite>> {
    let mut candidates: Vec<PathBuf> = std::env::var_os("EMUSEN_SQLITE").map(PathBuf::from).into_iter().collect();
    if let Some(home) = std::env::var_os("HOME") {
        if let Ok(versions) = std::fs::read_dir(PathBuf::from(home).join(".nuget/packages/sqlite")) {
            let mut found: Vec<PathBuf> = versions.flatten().map(|v| v.path().join("runtimes/linux-x64/native/libe_sqlite3.so")).filter(|p| p.exists()).collect();
            found.sort();
            candidates.extend(found.into_iter().rev());
        }
    }
    candidates.into_iter().find_map(|p| Sqlite::open(&p.to_string_lossy()).ok().map(Arc::new))
}

fn store(label: &str) -> Option<Store> {
    let sqlite = sqlite()?;
    let dir = std::env::temp_dir().join(format!("emusen-scrape-quota-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    Some(Store::open(sqlite, &dir.to_string_lossy()).unwrap())
}

fn limits(per_minute: i32, per_day: i32, ko_per_day: i32, today: i32, ko_today: i32, threads: i32) -> Option<Quota> {
    Some(Quota {
        max_threads: Some(threads),
        max_download_kbps: Some(128),
        requests_today: Some(today),
        requests_ko_today: Some(ko_today),
        max_requests_per_minute: Some(per_minute),
        max_requests_per_day: Some(per_day),
        max_requests_ko_per_day: Some(ko_per_day),
    })
}

/// Noon in Paris on a summer day, so the day's end is ten hours on.
fn noon() -> i64 {
    instant("2026-07-14T10:00:00Z").unwrap()
}

#[test]
fn the_pace_turns_threads_and_the_days_two_percent() {
    let Some(store) = store("rules") else {
        eprintln!("no e_sqlite3 found: nothing was run");
        return;
    };
    let clock = Clock::test(noon());
    let quota = Manager::new(&store, &clock).unwrap();
    assert_eq!(quota.interval(), 22_222_222);
    quota.observe(&store, &clock, limits(60, 20000, 2000, 0, 0, 1)).unwrap();
    assert_eq!(quota.interval(), 11_111_111);
    let stop = Stop::default();
    for _ in 0..4 {
        assert_eq!(quota.take_turn(&store, &clock, &stop).unwrap(), Some(true));
    }
    assert_eq!(clock.now() - noon(), 3 * 11_111_111);
    assert_eq!(quota.threads(4), 1);
    quota.observe(&store, &clock, limits(60, 20000, 2000, 0, 0, 2)).unwrap();
    assert_eq!((quota.threads(4), quota.threads(1)), (2, 1));

    quota.observe(&store, &clock, limits(60, 20000, 2000, 19599, 0, 1)).unwrap();
    assert_eq!(quota.check(&store, &clock).unwrap().0, Gate::Go);
    quota.observe(&store, &clock, limits(60, 20000, 2000, 19600, 0, 1)).unwrap();
    let (gate, until, why) = quota.check(&store, &clock).unwrap();
    assert_eq!((gate, until), (Gate::Stopped, Some(instant("2026-07-14T22:00:00Z").unwrap())));
    assert_eq!(why.as_deref(), Some("today's requests are nearly used up (19600 of 20000)"));
}

#[test]
fn a_430_lasts_until_paris_midnight_and_a_restart_and_a_403_until_the_settings_change() {
    let Some(store) = store("stops") else {
        eprintln!("no e_sqlite3 found: nothing was run");
        return;
    };
    let clock = Clock::test(noon());
    let quota = Manager::new(&store, &clock).unwrap();
    quota.answered(&store, &clock, Status::DailyQuota).unwrap();
    let (gate, until, why) = quota.check(&store, &clock).unwrap();
    assert_eq!((gate, until, why.as_deref()), (Gate::Stopped, Some(instant("2026-07-14T22:00:00Z").unwrap()), Some("today's requests are used up (430)")));
    let restarted = Manager::new(&store, &clock).unwrap();
    assert_eq!(restarted.check(&store, &clock).unwrap().0, Gate::Stopped);
    assert_eq!(restarted.take_turn(&store, &clock, &Stop::default()).unwrap(), Some(false));
    let later = Clock::test(instant("2026-07-14T22:00:01Z").unwrap());
    assert_eq!(restarted.check(&store, &later).unwrap().0, Gate::Go);

    restarted.answered(&store, &later, Status::BadCredentials).unwrap();
    assert_eq!(restarted.check(&store, &later).unwrap().2.as_deref(), Some("ScreenScraper refused EmuSen's developer credentials (403)"));
    assert!(restarted.settings_changed());
    assert_eq!(restarted.check(&store, &later).unwrap().0, Gate::Go);
}

#[test]
fn a_429_halves_the_pace_and_waits_a_minute_and_a_401_five() {
    let Some(store) = store("waits") else {
        eprintln!("no e_sqlite3 found: nothing was run");
        return;
    };
    let clock = Clock::test(noon());
    let quota = Manager::new(&store, &clock).unwrap();
    quota.observe(&store, &clock, limits(60, 20000, 2000, 0, 0, 1)).unwrap();
    let before = quota.interval();
    quota.answered(&store, &clock, Status::TooManyRequests).unwrap();
    assert_eq!(quota.interval(), before * 2);
    let (gate, until, _) = quota.check(&store, &clock).unwrap();
    assert_eq!((gate, until), (Gate::Wait, Some(noon() + 60 * TICKS_PER_SECOND)));
    assert_eq!(quota.take_turn(&store, &clock, &Stop::default()).unwrap(), Some(true));
    assert_eq!(clock.drain_delays().iter().sum::<i64>(), 60 * TICKS_PER_SECOND);
    quota.answered(&store, &clock, Status::ServerBusy).unwrap();
    assert_eq!(quota.check(&store, &clock).unwrap().1, Some(clock.now() + 300 * TICKS_PER_SECOND));
}

#[test]
fn pariss_day_turns_at_its_own_midnight_in_summer_and_in_winter() {
    assert_eq!(day_of(instant("2026-07-14T21:59:59Z").unwrap()), "2026-07-14");
    assert_eq!(day_of(instant("2026-07-14T22:00:00Z").unwrap()), "2026-07-15");
    assert_eq!(start_of_next_day(instant("2026-07-14T10:00:00Z").unwrap()), instant("2026-07-14T22:00:00Z").unwrap());
    assert_eq!(start_of_next_day(instant("2026-12-14T10:00:00Z").unwrap()), instant("2026-12-14T23:00:00Z").unwrap());
    assert_eq!(start_of_next_day(instant("2026-03-28T12:00:00Z").unwrap()), instant("2026-03-28T23:00:00Z").unwrap());
    assert_eq!(start_of_next_day(instant("2026-03-29T12:00:00Z").unwrap()), instant("2026-03-29T22:00:00Z").unwrap());
}
