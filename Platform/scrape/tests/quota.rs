//! §5.5's rules one by one on a clock the test moves: `ScrapeQuotaTests`' cases, held by the Rust half too. The two
//! halves' paces are compared directly by part 4's replay (EmuSen_Backend.md §30.7).

use emusen_galaxia::clock::{TICKS_PER_SECOND, instant};
use emusen_scrape::answer::Quota;
use emusen_scrape::clock::{Clock, Stop};
use emusen_scrape::quota::{Gate, Manager, Status, day_of, start_of_next_day};
use emusen_scrape::store::Store;
use emusen_sqlite::Sqlite;
use std::sync::Arc;

fn sqlite() -> Option<Arc<Sqlite>> {
    emusen_sqlite::for_tests()
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

// The day's unrecognised games stop two percent short of their limit too, as the requests do (the C# `ScrapeQuotaTests`' case, held here since its deletion).
#[test]
fn the_days_unrecognised_games_stop_two_percent_short_of_their_limit() {
    let Some(store) = store("ko") else {
        return;
    };
    let clock = Clock::test(noon());
    let quota = Manager::new(&store, &clock).unwrap();
    quota.observe(&store, &clock, limits(60, 20000, 2000, 0, 1959, 1)).unwrap();
    assert_eq!(quota.check(&store, &clock).unwrap().0, Gate::Go);
    quota.observe(&store, &clock, limits(60, 20000, 2000, 0, 1960, 1)).unwrap();
    assert_eq!(quota.check(&store, &clock).unwrap().0, Gate::Stopped);
}

// Counts are kept for the day in media.db, and a server's count replaces them.
#[test]
fn counts_are_kept_for_the_day_and_a_servers_count_replaces_them() {
    let Some(store) = store("counts") else {
        return;
    };
    let clock = Clock::test(noon());
    let quota = Manager::new(&store, &clock).unwrap();
    quota.counted(&store, &clock, false).unwrap();
    quota.counted(&store, &clock, true).unwrap();
    let s = quota.snapshot(&store, &clock).unwrap();
    assert_eq!((s.requests, s.ko), (2, 1));
    assert_eq!(store.day(&day_of(clock.now())).unwrap().requests, 2);
    quota.observe(&store, &clock, limits(60, 20000, 2000, 40, 7, 1)).unwrap();
    let s = quota.snapshot(&store, &clock).unwrap();
    assert_eq!((s.requests, s.ko), (40, 7));
}

// A 431, the day's unrecognised games used up, stops until Paris's midnight and outlives a restart, as a 430 does.
#[test]
fn a_431_lasts_until_paris_midnight_and_a_restart() {
    let Some(store) = store("stops431") else {
        return;
    };
    let clock = Clock::test(noon());
    let quota = Manager::new(&store, &clock).unwrap();
    quota.answered(&store, &clock, Status::DailyKoQuota).unwrap();
    let (gate, until, why) = quota.check(&store, &clock).unwrap();
    assert_eq!((gate, until), (Gate::Stopped, Some(instant("2026-07-14T22:00:00Z").unwrap())));
    assert!(why.as_deref().is_some_and(|w| w.contains("431")), "{why:?}");
    let restarted = Manager::new(&store, &clock).unwrap();
    assert_eq!(restarted.check(&store, &clock).unwrap().0, Gate::Stopped);
    let later = Clock::test(instant("2026-07-14T22:00:01Z").unwrap());
    assert_eq!(restarted.check(&store, &later).unwrap().0, Gate::Go);
}

// Each code ScreenScraper documents has its own meaning, and any other is a failure (the C# `ScreenScraperClientTests`' case, held here since its deletion).
#[test]
fn each_documented_code_has_its_own_meaning() {
    let cases = [
        (200, Status::Found), (400, Status::BadRequest), (401, Status::ServerBusy), (403, Status::BadCredentials), (404, Status::NotFound), (423, Status::ApiClosed),
        (426, Status::Blacklisted), (429, Status::TooManyRequests), (430, Status::DailyQuota), (431, Status::DailyKoQuota), (500, Status::Failed),
    ];
    for (code, status) in cases {
        assert_eq!(Status::of(code), status, "{code}");
    }
}
