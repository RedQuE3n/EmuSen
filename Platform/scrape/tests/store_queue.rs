//! media.db's queue, the C# `MediaStoreTests`' rules held here since the C# store was deleted (EmuSen_Backend.md §40.5):
//! the order by priority then arrival, a game shown moving up, a held game passed over, a retry not due before its time,
//! and the queue kept when the store is closed.

use emusen_scrape::store::{Priority, Store};
use std::path::PathBuf;

const NOW: i64 = 1_790_000_000 * 10_000_000;
const MINUTE: i64 = 60 * 10_000_000;

fn dir(name: &str) -> PathBuf {
    let d = std::env::var("TMPDIR").map(PathBuf::from).unwrap_or_else(|_| std::env::temp_dir()).join(format!("emusen-scrape-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

#[test]
fn the_queue_goes_by_priority_then_arrival_and_a_game_shown_moves_up() {
    let Some(sqlite) = emusen_sqlite::for_tests() else { return };
    let d = dir("queue");
    let store = Store::open(sqlite, &d.to_string_lossy()).unwrap();
    store.enqueue("/r/a.sfc", "snes", Priority::Library).unwrap();
    store.enqueue("/r/b.sfc", "snes", Priority::Library).unwrap();
    store.enqueue("/r/c.sfc", "snes", Priority::Console).unwrap();
    store.enqueue("/r/b.sfc", "snes", Priority::Shown).unwrap();
    store.enqueue("/r/c.sfc", "snes", Priority::Library).unwrap();
    let order: Vec<String> = store.queue().unwrap().into_iter().map(|q| q.path).collect();
    assert_eq!(order, ["/r/b.sfc", "/r/c.sfc", "/r/a.sfc"]);
    assert_eq!(store.next_due(NOW, &|p| p == "/r/b.sfc").unwrap().unwrap().path, "/r/c.sfc");
    drop(store);
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn a_game_waiting_to_be_retried_is_not_due_until_its_time_and_the_queue_outlives_the_store_being_closed() {
    let Some(sqlite) = emusen_sqlite::for_tests() else { return };
    let d = dir("retry");
    {
        let store = Store::open(sqlite.clone(), &d.to_string_lossy()).unwrap();
        store.enqueue("/r/a.sfc", "snes", Priority::Shown).unwrap();
        store.enqueue("/r/b.sfc", "nes", Priority::Library).unwrap();
        store.retry("/r/a.sfc", NOW + 5 * MINUTE, true).unwrap();
    }
    let reopened = Store::open(sqlite, &d.to_string_lossy()).unwrap();
    assert_eq!(reopened.next_due(NOW, &|_| false).unwrap().unwrap().path, "/r/b.sfc");
    let later = reopened.next_due(NOW + 5 * MINUTE, &|p| p == "/r/b.sfc").unwrap().unwrap();
    assert_eq!((later.path.as_str(), later.attempts), ("/r/a.sfc", 1));
    reopened.dequeue("/r/b.sfc").unwrap();
    assert_eq!(reopened.queue_length().unwrap(), 1);
    drop(reopened);
    let _ = std::fs::remove_dir_all(&d);
}
