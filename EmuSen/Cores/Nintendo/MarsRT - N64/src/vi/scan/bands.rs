//! The deferred walk in bands of rows, one on the presenter and the rest on helper threads, each band a walker of its own. See Mars_Native.md §6.11.

use std::cell::UnsafeCell;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::sync::{Arc, OnceLock};
use std::thread::{self, JoinHandle};

use super::walker::{Pass, Walker};
use super::{Job, Rasters, View};

/// `Vi.MinimumBandRows`: a band shorter than this is not worth its hand-over.
const MINIMUM_BAND_ROWS: i32 = 32;
const MAXIMUM_BANDS: usize = 8;

/// `Vi.Bands`: a quarter of the processors, one to four, as C# takes it; `EMUSEN_MARSRT_SCAN_BANDS` sets it for a process, one to eight.
pub fn default_bands() -> usize {
    static BANDS: OnceLock<usize> = OnceLock::new();
    *BANDS.get_or_init(|| {
        if let Some(n) = std::env::var("EMUSEN_MARSRT_SCAN_BANDS").ok().and_then(|v| v.parse::<usize>().ok()) {
            return n.clamp(1, MAXIMUM_BANDS);
        }
        (thread::available_parallelism().map_or(1, |n| n.get()) / 4).clamp(1, 4)
    })
}

const IDLE: u32 = 0;
const SUBMITTED: u32 = 1;
const DONE: u32 = 2;
const STOP: u32 = 3;

/// One band handed to a helper: the pass it reads and the rows and bytes of the raster it writes, alive until the helper's `DONE`.
struct Task {
    pass: *const Pass<'static>,
    chunk: *mut u8,
    len: usize,
    at: usize,
    length: usize,
    from: i32,
    to: i32,
    /// A test's hold: the helper waits here, before its rows, until the flag is set.
    #[cfg(test)]
    hold: Option<Arc<std::sync::atomic::AtomicBool>>,
}

/// A helper's slot: the presenter owns it at `IDLE` and `DONE`, the helper at `SUBMITTED`; each hands it over with a Release.
struct Slot {
    state: AtomicU32,
    task: UnsafeCell<Option<Task>>,
    fault: UnsafeCell<Option<Box<dyn std::any::Any + Send>>>,
}

// SAFETY: `task` and `fault` are touched only by the side `state` names, after an Acquire of the other side's Release; the pointers in a task
// name the presenter's pass and a raster chunk no other band holds, both alive until the presenter has seen `DONE`.
unsafe impl Sync for Slot {}
unsafe impl Send for Slot {}

struct Helper {
    slot: Arc<Slot>,
    thread: Option<JoinHandle<()>>,
}

/// The presenter's helpers, started as a walk first asks for them and stopped with the presenter.
#[derive(Default)]
pub(super) struct Bands {
    helpers: Vec<Helper>,
    /// How many bands the last walk took.
    pub last: usize,
}

/// A band the presenter waits for, for tests: which band, and the flag that releases it.
#[cfg(test)]
pub(super) type Hold = Option<(usize, Arc<std::sync::atomic::AtomicBool>)>;

impl Bands {
    /// `Vi.Walk`'s bands: the rows split into `count` runs, the first walked here and each other on a helper, and every one finished
    /// before this returns, a panic included; one band is the walk of `Walker::walk` itself.
    #[allow(clippy::too_many_arguments)]
    pub fn walk(&mut self, walker: &mut Walker, count: usize, job: &Job, view: View, scaled: Option<View>, rasters: &mut Rasters, #[cfg(test)] hold: Hold) {
        let pass = Pass::new(job, view, scaled);
        let rows = pass.picture.rows;
        let bands = (rows / MINIMUM_BAND_ROWS).clamp(1, count.clamp(1, MAXIMUM_BANDS) as i32) as usize;
        self.last = bands;
        if bands == 1 {
            walker.walk(job, view, scaled, rasters);
            return;
        }
        rasters.output_scale = job.scale();
        let raster = rasters.at(job.scale());
        let length = raster.len();
        while self.helpers.len() < bands - 1 {
            self.helpers.push(Helper::start(self.helpers.len() + 1));
        }

        // Band b is rows [rows·b/n, rows·(b+1)/n), and its chunk starts at its first row's line, where the band before it stops writing.
        let first_row = |b: usize| (rows as i64 * b as i64 / bands as i64) as i32;
        let starts: Vec<usize> = (0..=bands).map(|b| if b == 0 { 0 } else if b == bands { length } else { (pass.line(first_row(b)) * 4).clamp(0, length as i64) as usize }).collect();
        let mut rest = raster;
        let mut chunks = Vec::with_capacity(bands);
        for b in 0..bands {
            let (chunk, tail) = rest.split_at_mut(starts[b + 1] - starts[b]);
            chunks.push(chunk);
            rest = tail;
        }
        let mut chunks = chunks.into_iter();
        let own = chunks.next().expect("a first band");

        // SAFETY: the pass outlives every task, since each helper is waited for below before this frame returns or unwinds.
        let erased = (&pass as *const Pass).cast::<Pass<'static>>();
        for (b, chunk) in chunks.enumerate() {
            let b = b + 1;
            let task = Task {
                pass: erased,
                chunk: chunk.as_mut_ptr(),
                len: chunk.len(),
                at: starts[b],
                length,
                from: first_row(b),
                to: first_row(b + 1),
                #[cfg(test)]
                hold: hold.as_ref().filter(|(band, _)| *band == b).map(|(_, flag)| flag.clone()),
            };
            self.helpers[b - 1].submit(task);
        }
        let mine = catch_unwind(AssertUnwindSafe(|| walker.band(&pass, own, 0, length, 0, first_row(1))));
        let mut fault = mine.err();
        for helper in &mut self.helpers[..bands - 1] {
            if let Some(panic) = helper.wait() {
                fault.get_or_insert(panic);
            }
        }
        if let Some(panic) = fault {
            resume_unwind(panic);
        }
    }
}

impl Helper {
    fn start(index: usize) -> Helper {
        let slot = Arc::new(Slot { state: AtomicU32::new(IDLE), task: UnsafeCell::new(None), fault: UnsafeCell::new(None) });
        let theirs = slot.clone();
        let thread = thread::Builder::new().name(format!("MarsRT scan band {index}")).spawn(move || serve(&theirs)).expect("a scan band's thread could not start");
        Helper { slot, thread: Some(thread) }
    }

    fn submit(&mut self, task: Task) {
        // SAFETY: the slot is the presenter's at IDLE, until the Release below hands it over.
        unsafe { *self.slot.task.get() = Some(task) };
        self.slot.state.store(SUBMITTED, Release);
        if let Some(t) = &self.thread {
            t.thread().unpark();
        }
    }

    /// The band finished: its panic, if it raised one.
    fn wait(&mut self) -> Option<Box<dyn std::any::Any + Send>> {
        let mut spins = 0u32;
        while self.slot.state.load(Acquire) != DONE {
            if spins < 64 {
                std::hint::spin_loop();
            } else {
                thread::yield_now();
            }
            spins = spins.saturating_add(1);
        }
        // SAFETY: at DONE the slot is the presenter's again, after the Acquire above.
        let fault = unsafe { (*self.slot.fault.get()).take() };
        self.slot.state.store(IDLE, Relaxed);
        fault
    }
}

impl Drop for Helper {
    fn drop(&mut self) {
        self.slot.state.store(STOP, Release);
        if let Some(t) = self.thread.take() {
            t.thread().unpark();
            let _ = t.join();
        }
    }
}

fn serve(slot: &Slot) {
    let mut walker = Walker::default();
    loop {
        match slot.state.load(Acquire) {
            SUBMITTED => {
                // SAFETY: at SUBMITTED the slot is this thread's, after the Acquire above.
                let task = unsafe { (*slot.task.get()).take() }.expect("a submitted band holds its task");
                #[cfg(test)]
                if let Some(flag) = &task.hold {
                    while !flag.load(Acquire) {
                        thread::yield_now();
                    }
                }
                let ran = catch_unwind(AssertUnwindSafe(|| {
                    // SAFETY: the presenter keeps the pass alive and this chunk unshared until it has seen DONE below.
                    let (pass, chunk) = unsafe { (&*task.pass, std::slice::from_raw_parts_mut(task.chunk, task.len)) };
                    walker.band(pass, chunk, task.at, task.length, task.from, task.to);
                }));
                // The test's flag goes before the slot is handed back, so nothing of the task outlives the band.
                #[cfg(test)]
                drop(task);
                // SAFETY: still this thread's until the Release below.
                unsafe { *slot.fault.get() = ran.err() };
                slot.state.store(DONE, Release);
            }
            STOP => return,
            _ => thread::park(),
        }
    }
}
