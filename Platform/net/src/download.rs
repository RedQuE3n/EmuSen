//! A fetch on a thread of its own, which the host drains: how far it has come, its end, and a cancel that ends it at
//! once for the host however long the socket takes to notice (EmuSen_Backend.md §16.1).

use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::Duration;

use crate::failure::{Failure, Kind};
use crate::fetch::{Progress, Request, fetch};

/// Where a download stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Running = 0,
    Done = 1,
    Failed = 2,
    Cancelled = 3,
}

type Outcome = Result<Vec<u8>, Failure>;

#[derive(Default)]
struct Shared {
    progress: Progress,
    outcome: Mutex<Option<Outcome>>,
    ended: Condvar,
}

impl Shared {
    fn lock(&self) -> MutexGuard<'_, Option<Outcome>> {
        self.outcome.lock().unwrap_or_else(|p| p.into_inner())
    }

    /// The first outcome stands: a cancel's, or the fetch's.
    fn end(&self, outcome: Outcome) {
        let mut held = self.lock();
        if held.is_none() {
            *held = Some(outcome);
            self.ended.notify_all();
        }
    }
}

/// One download. Dropping it leaves a fetch still running to end on its own and its body to be dropped.
pub struct Download {
    shared: Arc<Shared>,
}

fn state(outcome: &Option<Outcome>) -> State {
    match outcome {
        None => State::Running,
        Some(Ok(_)) => State::Done,
        Some(Err(f)) if f.kind == Kind::Cancelled => State::Cancelled,
        Some(Err(_)) => State::Failed,
    }
}

impl Download {
    pub fn start(request: Request) -> Download {
        let shared = Arc::new(Shared::default());
        let worker = shared.clone();
        let spawned = std::thread::Builder::new().name("emusen-net download".into()).spawn(move || {
            let outcome = fetch(&request, &worker.progress);
            worker.end(outcome);
        });
        if let Err(e) = spawned {
            shared.end(Err(Failure::other(format!("no thread for the download: {e}"))));
        }
        Download { shared }
    }

    /// The body's bytes so far, its length once known, and where the download stands.
    pub fn progress(&self) -> (u64, Option<u64>, State) {
        let progress = &self.shared.progress;
        (progress.received(), progress.total(), state(&self.shared.lock()))
    }

    /// Ends the download for the host now; the fetch stops at its next read.
    pub fn cancel(&self) {
        self.shared.progress.cancel();
        self.shared.end(Err(Failure::cancelled()));
    }

    /// Waits up to `timeout` for the end, and says where the download stands.
    pub fn wait(&self, timeout: Duration) -> State {
        let held = self.shared.lock();
        let (held, _) = self.shared.ended.wait_timeout_while(held, timeout, |o| o.is_none()).unwrap_or_else(|p| p.into_inner());
        state(&held)
    }

    /// Runs `read` over the outcome once there is one.
    pub fn with_outcome<R>(&self, read: impl FnOnce(&Result<Vec<u8>, Failure>) -> R) -> Option<R> {
        self.shared.lock().as_ref().map(read)
    }
}

impl Drop for Download {
    fn drop(&mut self) {
        self.shared.progress.cancel();
    }
}
