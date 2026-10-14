//! An image's progress as the corpus form watches it (EmuSen_CoreAPI.md §28.10): the thread running an image's cases
//! stamps the time of each frame it runs, and the runner reports an image whose stamp grows old as hung.

use std::cell::RefCell;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Instant;

static EPOCH: OnceLock<Instant> = OnceLock::new();

thread_local! {
    static WATCHED: RefCell<Option<Arc<AtomicU64>>> = const { RefCell::new(None) };
}

/// Milliseconds since the process's first stamp.
pub fn now() -> u64 {
    EPOCH.get_or_init(Instant::now).elapsed().as_millis() as u64
}

/// The stamp this thread's frames write from now on, stamped now; `None` stops.
pub fn watch(stamp: Option<Arc<AtomicU64>>) {
    if let Some(s) = &stamp {
        s.store(now(), Ordering::Relaxed);
    }
    WATCHED.with(|w| *w.borrow_mut() = stamp);
}

/// A frame run: the watched stamp, if any, set to now.
pub fn beat() {
    WATCHED.with(|w| {
        if let Some(s) = &*w.borrow() {
            s.store(now(), Ordering::Relaxed);
        }
    });
}

/// Milliseconds since `stamp` was last written.
pub fn idle(stamp: &AtomicU64) -> u64 {
    now().saturating_sub(stamp.load(Ordering::Relaxed))
}
