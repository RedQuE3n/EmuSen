//! Time for the pacer and the stop rules, `IScrapeClock`: the system's, or the test clock the settings' `test.clock`
//! starts, which a delay advances at once and records, as WiseMan's `FakeScrapeClock` does (EmuSen_Backend.md §30.3).

use std::sync::{Condvar, Mutex};
use std::time::Duration;

use emusen_dotnet::date::TICKS_PER_SECOND;

/// A wait that a cancel ends at once.
#[derive(Default)]
pub struct Stop {
    stopped: Mutex<bool>,
    signal: Condvar,
}

impl Stop {
    pub fn stop(&self) {
        *self.stopped.lock().unwrap_or_else(|p| p.into_inner()) = true;
        self.signal.notify_all();
    }

    pub fn reset(&self) {
        *self.stopped.lock().unwrap_or_else(|p| p.into_inner()) = false;
    }

    pub fn is_stopped(&self) -> bool {
        *self.stopped.lock().unwrap_or_else(|p| p.into_inner())
    }

    /// Waits up to `wait`, or until stopped or `poke`d; true when stopped.
    pub fn wait(&self, wait: Duration) -> bool {
        let guard = self.stopped.lock().unwrap_or_else(|p| p.into_inner());
        if *guard {
            return true;
        }
        let (guard, _) = self.signal.wait_timeout(guard, wait).unwrap_or_else(|p| p.into_inner());
        *guard
    }

    /// Wakes a wait without stopping it.
    pub fn poke(&self) {
        self.signal.notify_all();
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

/// `f` run on a thread of its own and waited for, the wait ending at once when `stop` is: the answer, or none when
/// stopped. A request in flight is then left to end on its own by its timeout, its cancel flag set by the caller.
pub fn detached<T: Send + 'static>(stop: &Stop, f: impl FnOnce() -> T + Send + 'static) -> Option<T> {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(f());
    });
    loop {
        match rx.recv_timeout(Duration::from_millis(20)) {
            Ok(v) => return Some(v),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) if !stop.is_stopped() => continue,
            Err(_) => return None,
        }
    }
}
