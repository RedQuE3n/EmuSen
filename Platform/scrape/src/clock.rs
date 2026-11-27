//! Time for the pacer and the stop rules, `IScrapeClock`: the system's, or the test clock the settings' `test.clock`
//! starts, which a delay advances at once and records, as WiseMan's `FakeScrapeClock` does (EmuSen_Backend.md §30.3).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use emusen_galaxia::clock::TICKS_PER_SECOND;

#[derive(Default)]
struct StopInner {
    stopped: Mutex<bool>,
    signal: Condvar,
    linked: Mutex<Vec<Arc<emusen_net::Progress>>>,
}

/// A wait that a cancel ends at once.
#[derive(Clone, Default)]
pub struct Stop(Arc<StopInner>);

pub struct StopLink {
    inner: Arc<StopInner>,
    progress: Arc<emusen_net::Progress>,
}

impl Drop for StopLink {
    fn drop(&mut self) {
        self.inner.linked.lock().unwrap_or_else(|p| p.into_inner()).retain(|p| !Arc::ptr_eq(p, &self.progress));
    }
}

impl Stop {
    pub fn stop(&self) {
        *self.0.stopped.lock().unwrap_or_else(|p| p.into_inner()) = true;
        for p in self.0.linked.lock().unwrap_or_else(|p| p.into_inner()).iter() {
            p.cancel();
        }
        self.0.signal.notify_all();
    }

    pub fn reset(&self) {
        *self.0.stopped.lock().unwrap_or_else(|p| p.into_inner()) = false;
    }

    pub fn is_stopped(&self) -> bool {
        *self.0.stopped.lock().unwrap_or_else(|p| p.into_inner())
    }

    /// Cancels `progress` when `self` stops; unregisters when dropped.
    pub fn link(&self, progress: &Arc<emusen_net::Progress>) -> StopLink {
        self.0.linked.lock().unwrap_or_else(|p| p.into_inner()).push(progress.clone());
        if self.is_stopped() {
            progress.cancel();
        }
        StopLink { inner: self.0.clone(), progress: progress.clone() }
    }

    /// Waits up to `wait`, or until stopped or `poke`d; true when stopped.
    pub fn wait(&self, wait: Duration) -> bool {
        let guard = self.0.stopped.lock().unwrap_or_else(|p| p.into_inner());
        if *guard {
            return true;
        }
        let (guard, _) = self.0.signal.wait_timeout(guard, wait).unwrap_or_else(|p| p.into_inner());
        *guard
    }

    /// Wakes a wait without stopping it.
    pub fn poke(&self) {
        self.0.signal.notify_all();
    }
}

pub enum Clock {
    System,
    Test { now: Mutex<i64>, delays: Mutex<Vec<i64>> },
}

impl Clock {
    pub fn test(start: i64) -> Clock {
        Clock::Test { now: Mutex::new(start), delays: Mutex::new(Vec::new()) }
    }

    /// UTC ticks from 1970.
    pub fn now(&self) -> i64 {
        match self {
            Clock::System => {
                let since = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
                since.as_secs() as i64 * TICKS_PER_SECOND + since.subsec_nanos() as i64 / 100
            }
            Clock::Test { now, .. } => *now.lock().unwrap_or_else(|p| p.into_inner()),
        }
    }

    /// Waits `ticks`, or until stopped; false when stopped. The test clock advances by the wait and records it.
    pub fn delay(&self, ticks: i64, stop: &Stop) -> bool {
        if stop.is_stopped() {
            return false;
        }
        match self {
            Clock::System => ticks <= 0 || !stop.wait(Duration::from_nanos(ticks as u64 * 100)),
            Clock::Test { now, delays } => {
                delays.lock().unwrap_or_else(|p| p.into_inner()).push(ticks);
                if ticks > 0 {
                    *now.lock().unwrap_or_else(|p| p.into_inner()) += ticks;
                }
                true
            }
        }
    }

    /// The test clock's recorded waits since the last drain; none for the system's.
    pub fn drain_delays(&self) -> Vec<i64> {
        match self {
            Clock::System => Vec::new(),
            Clock::Test { delays, .. } => std::mem::take(&mut *delays.lock().unwrap_or_else(|p| p.into_inner())),
        }
    }

    pub fn is_test(&self) -> bool {
        matches!(self, Clock::Test { .. })
    }
}

/// `f` run on a thread of its own and waited on via `Condvar`, ending at once on completion or `stop`.
pub fn detached<T: Send + 'static>(stop: &Stop, f: impl FnOnce() -> T + Send + 'static) -> Option<T> {
    struct NotifyOnDrop {
        done: Arc<AtomicBool>,
        wake: Stop,
    }
    impl Drop for NotifyOnDrop {
        fn drop(&mut self) {
            self.done.store(true, Ordering::Release);
            let _g = self.wake.0.stopped.lock().unwrap_or_else(|p| p.into_inner());
            self.wake.0.signal.notify_all();
        }
    }
    let slot = Arc::new(Mutex::new(None));
    let done = Arc::new(AtomicBool::new(false));
    let (tx, flag, wake) = (slot.clone(), done.clone(), stop.clone());
    std::thread::spawn(move || {
        let _notify = NotifyOnDrop { done: flag, wake };
        let value = f();
        *tx.lock().unwrap_or_else(|p| p.into_inner()) = Some(value);
    });
    let mut guard = stop.0.stopped.lock().unwrap_or_else(|p| p.into_inner());
    loop {
        if let Some(v) = slot.lock().unwrap_or_else(|p| p.into_inner()).take() {
            return Some(v);
        }
        if *guard || done.load(Ordering::Acquire) {
            return None;
        }
        guard = stop.0.signal.wait(guard).unwrap_or_else(|p| p.into_inner());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detached_wakes_on_completion_and_stop_cancels_linked_progress() {
        let stop = Stop::default();
        assert_eq!(detached(&stop, || 42), Some(42));
        let progress = Arc::new(emusen_net::Progress::default());
        let _link = stop.link(&progress);
        assert!(!progress.is_cancelled());
        let stop_clone = stop.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(10));
            stop_clone.stop();
        });
        let abandoned = detached(&stop, || {
            std::thread::sleep(Duration::from_millis(200));
            1
        });
        assert_eq!(abandoned, None);
        assert!(progress.is_cancelled());
    }
}
