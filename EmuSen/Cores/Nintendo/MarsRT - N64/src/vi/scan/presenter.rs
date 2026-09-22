//! The deferred walk's thread: one job at a time, handed over and back by a state word, never shared. See Mars_Native.md §5.6.5.

use std::cell::UnsafeCell;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::thread::{self, JoinHandle};

use super::{Capture, Job, Walker, compose_into};

/// One deferred scan: what it walks, over what, into which raster and frame; everything it touches travels with it.
pub(super) struct Work {
    pub job: Option<Job>,
    pub capture: Capture,
    pub raster: Vec<u8>,
    pub walker: Walker,
    pub frame: Vec<u8>,
    pub rows: usize,
    pub serrate: bool,
    pub repeat_rows: bool,
    pub shown: (u32, u32, u32),
    pub fault: Option<String>,
}

impl Work {
    /// `Walk` over the capture, then `Compose`, as the pool item of `PresentDeferred` runs them.
    fn run(&mut self) {
        if let Some(job) = self.job {
            self.walker.walk(&job, self.capture.view(), &mut self.raster);
        }
        self.shown = compose_into(&self.raster, self.rows, self.serrate, self.repeat_rows, &mut self.frame);
    }
}

const IDLE: u32 = 0;
const SUBMITTED: u32 = 1;
const DONE: u32 = 2;
const STOP: u32 = 3;

/// The job's slot: the machine's thread owns it at `IDLE` and `DONE`, the presenter at `SUBMITTED`; each hands it over with a Release.
struct Slot {
    state: AtomicU32,
    work: UnsafeCell<Option<Box<Work>>>,
}

// SAFETY: `work` is touched only by the side `state` names, after an Acquire of the other side's Release.
unsafe impl Sync for Slot {}

pub(super) struct Presenter {
    slot: Arc<Slot>,
    thread: Option<JoinHandle<()>>,
    out: bool,
}

impl Presenter {
    pub fn start() -> Presenter {
        let slot = Arc::new(Slot { state: AtomicU32::new(IDLE), work: UnsafeCell::new(None) });
        let theirs = slot.clone();
        let thread = thread::Builder::new().name("MarsRT presenter".into()).spawn(move || serve(&theirs)).expect("the presenter thread could not start");
        Presenter { slot, thread: Some(thread), out: false }
    }

    pub fn submit(&mut self, work: Box<Work>) {
        assert!(!self.out, "a deferred walk is already out");
        // SAFETY: the slot is this thread's at IDLE, until the Release below hands it over.
        unsafe { *self.slot.work.get() = Some(work) };
        self.slot.state.store(SUBMITTED, Release);
        self.out = true;
        if let Some(t) = &self.thread {
            t.thread().unpark();
        }
    }

    /// The job out, once finished; none when none is out.
    pub fn take(&mut self) -> Option<Box<Work>> {
        if !self.out {
            return None;
        }
        let mut spins = 0u32;
        while self.slot.state.load(Acquire) != DONE {
            if spins < 64 {
                std::hint::spin_loop();
            } else {
                thread::yield_now();
            }
            spins = spins.saturating_add(1);
        }
        // SAFETY: at DONE the slot is this thread's again, after the Acquire above.
        let work = unsafe { (*self.slot.work.get()).take() };
        self.slot.state.store(IDLE, Relaxed);
        self.out = false;
        work
    }

    pub fn busy(&self) -> bool {
        self.out
    }
}

impl Drop for Presenter {
    fn drop(&mut self) {
        let _ = self.take();
        self.slot.state.store(STOP, Release);
        if let Some(t) = self.thread.take() {
            t.thread().unpark();
            let _ = t.join();
        }
    }
}

fn serve(slot: &Slot) {
    loop {
        match slot.state.load(Acquire) {
            SUBMITTED => {
                // SAFETY: at SUBMITTED the slot is this thread's, after the Acquire above.
                let mut work = unsafe { (*slot.work.get()).take() }.expect("a submitted slot holds its job");
                if let Err(panic) = catch_unwind(AssertUnwindSafe(|| work.run())) {
                    let message = panic.downcast_ref::<String>().cloned().or_else(|| panic.downcast_ref::<&str>().map(|s| s.to_string()));
                    work.fault = Some(message.unwrap_or_default());
                }
                // SAFETY: still this thread's until the Release below.
                unsafe { *slot.work.get() = Some(work) };
                slot.state.store(DONE, Release);
            }
            STOP => return,
            _ => thread::park(),
        }
    }
}
