//! The deferred walk's thread: one job at a time, handed over and back by a state word, never shared. See Mars_Native.md §5.6.5.

use std::cell::UnsafeCell;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::thread::{self, JoinHandle};
use std::time::Instant;

use super::bands::Bands;
use super::{Capture, Job, Rasters, SharedPictures, Walker, compose_from};

/// One deferred scan: what it walks, over what, into which raster and frame; everything it touches travels with it.
pub(super) struct Work {
    pub job: Option<Job>,
    pub capture: Capture,
    pub rasters: Rasters,
    pub walker: Walker,
    pub frame: Vec<u8>,
    pub rows: usize,
    pub serrate: bool,
    pub repeat_rows: bool,
    pub average: i32,
    /// The device's part: whether the raster is the device's while it averages, what the last walk showed, and the device's pictures to wait for (Mars_Gpu.md §14, §15).
    pub device_raster: bool,
    pub shown_from_device: bool,
    pub pictures: SharedPictures,
    pub shown: (u32, u32, u32),
    pub fault: Option<String>,
    /// How long the presenter took over it, walk and composition.
    pub nanos: i64,
    /// How many bands the walk is split into (Mars_Native.md §6.11).
    pub bands: usize,
    /// How many it was split into, zero when nothing was walked here.
    pub walked_in: usize,
    #[cfg(test)]
    pub hold: super::Hold,
}

impl Work {
    /// `Walk` over the capture, or the device's picture, then `Compose`, as the pool item of `PresentDeferred` runs them.
    fn run(&mut self, pool: &mut Bands) {
        if let Some(job) = self.job {
            let scaled = (job.scale() > 1).then(|| self.capture.scaled_view());
            #[cfg(test)]
            let hold = self.hold.clone();
            #[cfg(not(test))]
            let hold = ();
            pool.last = 0;
            self.shown_from_device = super::walk(&mut self.walker, Some((pool, self.bands, hold)), &mut self.rasters, &job, self.capture.view(), scaled, self.capture.device_scanned, self.device_raster, self.pictures.as_deref());
        }
        self.walked_in = pool.last;
        self.shown = compose_from(&mut self.rasters, self.rows, self.serrate, self.repeat_rows, self.average, self.shown_from_device, self.pictures.as_deref(), &mut self.frame);
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
    /// The joins that found the job unfinished, and how long they waited for it.
    pub waits: i64,
    pub waited_nanos: i64,
}

impl Presenter {
    pub fn start() -> Presenter {
        let slot = Arc::new(Slot { state: AtomicU32::new(IDLE), work: UnsafeCell::new(None) });
        let theirs = slot.clone();
        let thread = thread::Builder::new().name("MarsRT presenter".into()).spawn(move || serve(&theirs)).expect("the presenter thread could not start");
        Presenter { slot, thread: Some(thread), out: false, waits: 0, waited_nanos: 0 }
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
        if self.slot.state.load(Acquire) != DONE {
            let started = Instant::now();
            let mut spins = 0u32;
            while self.slot.state.load(Acquire) != DONE {
                if spins < 64 {
                    std::hint::spin_loop();
                } else {
                    thread::yield_now();
                }
                spins = spins.saturating_add(1);
            }
            self.waits += 1;
            self.waited_nanos += started.elapsed().as_nanos() as i64;
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
    let mut pool = Bands::default();
    loop {
        match slot.state.load(Acquire) {
            SUBMITTED => {
                // SAFETY: at SUBMITTED the slot is this thread's, after the Acquire above.
                let mut work = unsafe { (*slot.work.get()).take() }.expect("a submitted slot holds its job");
                let started = Instant::now();
                if let Err(panic) = catch_unwind(AssertUnwindSafe(|| work.run(&mut pool))) {
                    let message = panic.downcast_ref::<String>().cloned().or_else(|| panic.downcast_ref::<&str>().map(|s| s.to_string()));
                    work.fault = Some(message.unwrap_or_default());
                }
                work.nanos = started.elapsed().as_nanos() as i64;
                // SAFETY: still this thread's until the Release below.
                unsafe { *slot.work.get() = Some(work) };
                slot.state.store(DONE, Release);
            }
            STOP => return,
            _ => thread::park(),
        }
    }
}
