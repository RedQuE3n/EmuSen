//! The software rasteriser on worker threads (Sapphire_Native.md §50, Sapphire_Plan.md §5.12's lever 3), exact by
//! construction. The GS's thread (the emulation thread) takes each drawing kick as it does on one thread, with the
//! environment of its context at that kick, and publishes the primitive to a ring that every worker reads in order;
//! a host-to-local transfer's data goes into the same ring, 32 units to a job. Worker `w` of `n` draws the rows of
//! the bands of 32 or 64 rows (the taller page height of the frame and Z formats) whose number modulo `n` is `w`; a
//! primitive whose stores meet between workers, or whose reads meet what it changes, and a transfer's job, go whole to
//! one worker.
//!
//! The local memory is owned by page lanes, the low 24 bits and the high 8 of each page's words (`local::Units`): the
//! GS's thread records, for each, the worker and the job that last changed it and the last job that read it, and gives
//! each job, for each worker running it, the earlier jobs of other workers it must see finished: before a store of a
//! page's words, the last changers of both lanes and the last readers of the lanes it changes; before a read, the
//! lane's last changer. The worker waits for those, and the GS's thread goes on. Every access of the GS's own thread to
//! the memory (a local-to-host or local-to-local transfer, a CLUT's load, the PCRTC's lines, a state, a debugger) waits
//! the same way for the lanes it touches, or for every worker. So each lane of each word sees its reads and changes in
//! the order the one-thread path makes them, whatever the worker count and the host's scheduling, and the memory at any
//! point the machine can observe is the one-thread path's. The registers (CSR, SIGLBLID and the events FINISH, SIGNAL
//! and LABEL raise) are the GS's thread's alone, so they need no barrier.
//!
//! The CLUT's buffer is copied into one of sixteen slots when a primitive uses it after a load, and a slot is reused
//! only once every primitive that read it is drawn. Every hand-over is one of this module's atomics: a job is written
//! into its slot before the count that publishes it is stored with Release, and a worker's count is stored with Release
//! after its last access to a job, the memory and the slot, and loaded with Acquire before the GS's thread, or another
//! worker, touches what it covered. A worker waits only for earlier jobs, so no wait closes a cycle.

use std::cell::{Cell, RefCell, UnsafeCell};
use std::mem::MaybeUninit;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering, fence};
use std::thread::JoinHandle;
use std::time::Instant;

use super::draw::{Environment, Kind, Vertex};
use super::local::{self, Local, Memory, UNITS, Units, address, lanes};
use super::psm::Psm;
use super::raster::{self, Rows};
use super::texture::{Clut, Sampler};
use super::transfer::Transfer;

/// The most workers a GS runs.
pub const MAX: usize = 8;
/// Jobs published and not yet run by every worker, at most.
const RING: u64 = 4096;
/// CLUT buffers in use at once, at most.
const CLUTS: usize = 16;
/// Rounds a worker spins on an empty ring before it parks.
const SPIN: u32 = 2000;
/// A host-to-local transfer's 64-bit units in one job.
const BATCH: usize = 32;

/// The workers' count chosen automatically (measured 2026-10-26, Sapphire_Native.md §50): a quarter of the host's
/// threads, at least 1 (the one-thread path) and at most 4.
pub fn automatic() -> usize {
    std::thread::available_parallelism().map_or(1, |n| n.get() / 4).clamp(1, 4)
}

/// The workers' count a setting names: a number, or `auto`; 1 is the one-thread path, the reference.
pub fn parse(s: &str) -> Option<usize> {
    if s == "auto" { Some(automatic()) } else { s.parse::<usize>().ok().filter(|&n| (1..=MAX).contains(&n)) }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Split {
    /// Every worker its own bands of 2^shift rows.
    Rows(u32),
    /// One worker the whole job.
    One(u32),
}

#[derive(Clone, Debug)]
enum Job {
    Draw { kind: Kind, env: Environment, v: [Vertex; 3], clut: u32 },
    /// A host-to-local transfer's units from the transfer as it stood before the first.
    Upload { from: Transfer, units: [u64; BATCH], n: u32 },
}

#[derive(Clone, Debug)]
struct Command {
    job: Job,
    split: Split,
    /// For each running worker, the count each other worker must reach first.
    need: [[u64; MAX]; MAX],
}

#[repr(align(64))]
struct Padded<T>(T);

struct Shared {
    ring: Box<[UnsafeCell<MaybeUninit<Command>>]>,
    cluts: Box<[UnsafeCell<Clut>]>,
    published: Padded<AtomicU64>,
    done: [Padded<AtomicU64>; MAX],
    sleeping: [Padded<AtomicBool>; MAX],
    busy: [Padded<AtomicU64>; MAX],
    stalled: [Padded<AtomicU64>; MAX],
    /// Workers held before their next job, by bit: the wait-site tests' lever.
    held: AtomicU64,
    stop: AtomicBool,
}

// SAFETY: the ring's and the CLUT slots' cells are written by the GS's thread only while no worker can read them, as the
// module's comment argues; everything else is atomic.
unsafe impl Sync for Shared {}
unsafe impl Send for Shared {}

/// What the GS's thread waited at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cause {
    /// A ring slot, then a CLUT slot, for a job.
    Ring,
    Clut,
    /// A local-to-host transfer's reads, a local-to-local transfer, a CLUT's load.
    Download,
    Copy,
    ClutLoad,
    /// Every worker, for a state, a debugger or a test.
    Join,
    /// The PCRTC's lines.
    Display,
}

/// What the workers did and the GS's thread waited for, for the bench and the tests.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub primitives: u64,
    pub split: u64,
    pub alone: u64,
    /// Primitives drawn alone because their stores met between workers, or their reads met what they change.
    pub alone_why: [u64; 2],
    pub empty: u64,
    pub uploads: u64,
    pub clut_copies: u64,
    /// The GS's thread's waits by `Cause`, each a count and nanoseconds.
    pub waits: [(u64, u64); 7],
    /// Each worker's nanoseconds running jobs, and waiting within that for another worker.
    pub busy: [u64; MAX],
    pub stalled: [u64; MAX],
}

/// A host-to-local transfer's units not yet published.
struct Batch {
    from: Transfer,
    units: [u64; BATCH],
    n: usize,
}

/// The workers and the GS's thread's bookkeeping, used by that thread alone (so its cells need no locks).
pub struct Workers {
    n: usize,
    shared: Arc<Shared>,
    threads: Vec<JoinHandle<()>>,
    published: Cell<u64>,
    /// By page lane: the worker and the job that last changed it, and the last job that read it.
    owner: Box<[Cell<u8>]>,
    wseq: Box<[Cell<u64>]>,
    rseq: Box<[Cell<u64>]>,
    seen: [Cell<u64>; MAX],
    seen_min: Cell<u64>,
    clut_slot: Cell<usize>,
    clut_use: [Cell<u64>; CLUTS],
    /// The CLUT's buffer changed since it was last copied to a slot.
    clut_dirty: Cell<bool>,
    textures: Cell<Option<(Sampler, Units)>>,
    rotate: Cell<usize>,
    batch: RefCell<Option<Batch>>,
    counts: Cell<Counts>,
}

impl Workers {
    /// `n` workers (2 to MAX) over `local`, which must outlive them, starting from the CLUT buffer `clut`.
    pub fn new(n: usize, local: &Local, clut: &Clut) -> Workers {
        assert!((2..=MAX).contains(&n));
        let shared = Arc::new(Shared {
            ring: (0..RING).map(|_| UnsafeCell::new(MaybeUninit::uninit())).collect(),
            cluts: (0..CLUTS).map(|_| UnsafeCell::new(clut.clone())).collect(),
            published: Padded(AtomicU64::new(0)),
            done: std::array::from_fn(|_| Padded(AtomicU64::new(0))),
            sleeping: std::array::from_fn(|_| Padded(AtomicBool::new(false))),
            busy: std::array::from_fn(|_| Padded(AtomicU64::new(0))),
            stalled: std::array::from_fn(|_| Padded(AtomicU64::new(0))),
            held: AtomicU64::new(0),
            stop: AtomicBool::new(false),
        });
        let threads = (0..n)
            .map(|w| {
                let (s, mut view) = (Arc::clone(&shared), local.view());
                std::thread::Builder::new().name(format!("sapphire-gs-{w}")).spawn(move || run(&s, w, n, &mut view)).expect("a worker thread")
            })
            .collect();
        Workers {
            n,
            shared,
            threads,
            published: Cell::new(0),
            owner: (0..UNITS).map(|_| Cell::new(0)).collect(),
            wseq: (0..UNITS).map(|_| Cell::new(0)).collect(),
            rseq: (0..UNITS).map(|_| Cell::new(0)).collect(),
            seen: std::array::from_fn(|_| Cell::new(0)),
            seen_min: Cell::new(0),
            clut_slot: Cell::new(0),
            clut_use: std::array::from_fn(|_| Cell::new(0)),
            clut_dirty: Cell::new(false),
            textures: Cell::new(None),
            rotate: Cell::new(0),
            batch: RefCell::new(None),
            counts: Cell::new(Counts::default()),
        }
    }

    pub fn count(&self) -> usize {
        self.n
    }

    /// The CLUT's buffer has been loaded (or a state read): the next primitive to use it takes a fresh slot.
    pub fn clut_changed(&self) {
        self.clut_dirty.set(true);
    }

    /// Jobs published that some worker has not yet run.
    pub fn pending(&self) -> u64 {
        self.flush();
        let p = self.published.get();
        (0..self.n).map(|w| p - self.shared.done[w].0.load(Ordering::Acquire).min(p)).max().unwrap_or(0)
    }

    /// A handle that holds and releases workers from any thread, for the tests.
    pub fn hold(&self) -> Hold {
        Hold(Arc::clone(&self.shared))
    }

    pub fn counts(&self) -> Counts {
        let mut c = self.counts.get();
        for w in 0..self.n {
            c.busy[w] = self.shared.busy[w].0.load(Ordering::Relaxed);
            c.stalled[w] = self.shared.stalled[w].0.load(Ordering::Relaxed);
        }
        c
    }

    fn tally(&self, f: impl FnOnce(&mut Counts)) {
        let mut c = self.counts.get();
        f(&mut c);
        self.counts.set(c);
    }

    fn refresh_min(&self) {
        self.seen_min.set((0..self.n).map(|w| self.seen[w].get()).min().unwrap_or(0));
    }

    /// Waits until worker `w` has run every job up to `target`.
    fn wait(&self, w: usize, target: u64, cause: Cause) {
        let done = &self.shared.done[w].0;
        let mut d = done.load(Ordering::Acquire);
        if d < target {
            let t0 = Instant::now();
            let mut spins = 0u32;
            while d < target {
                spins += 1;
                if spins < 4096 {
                    std::hint::spin_loop();
                } else {
                    std::thread::yield_now();
                }
                d = done.load(Ordering::Acquire);
            }
            let ns = t0.elapsed().as_nanos() as u64;
            self.tally(|c| {
                let e = &mut c.waits[cause as usize];
                e.0 += 1;
                e.1 += ns;
            });
        }
        self.seen[w].set(d);
        self.refresh_min();
    }

    fn wait_all(&self, target: u64, cause: Cause) {
        for w in 0..self.n {
            if self.seen[w].get() < target {
                self.wait(w, target, cause);
            }
        }
    }

    /// Waits for every job: the memory is then the GS's thread's alone until the next one.
    pub fn join(&self) {
        self.flush();
        self.wait_all(self.published.get(), Cause::Join);
    }

    fn settle_unit(&self, u: usize, write: bool, cause: Cause) {
        let o = self.owner[u].get() as usize;
        if self.wseq[u].get() > self.seen[o].get() {
            self.wait(o, self.wseq[u].get(), cause);
        }
        if write && self.rseq[u].get() > self.seen_min.get() {
            self.wait_all(self.rseq[u].get(), cause);
        }
    }

    /// Before the GS's thread reads the lanes `lanes` of word `word`, or writes them (storing the whole word): the
    /// workers changing those lanes, or for a write either lane, and for a write the readers of the lanes it changes,
    /// are waited for up to their last job touching them.
    #[inline]
    pub fn settle(&self, word: u32, lanes: u32, write: bool, cause: Cause) {
        self.flush();
        let p = (word >> 11) as usize;
        for l in 0..2 {
            if lanes >> l & 1 != 0 {
                self.settle_unit(2 * p + l, write, cause);
            } else if write {
                self.settle_unit(2 * p + l, false, cause);
            }
        }
    }

    /// `settle` for the PCRTC's reads of every unit of a set.
    pub fn settle_reads(&self, units: &Units) {
        self.flush();
        for u in units.each() {
            self.settle_unit(u, false, Cause::Display);
        }
    }

    /// Whether the GS's thread may read the lanes of a word now without waiting: the verifier's question.
    pub fn settled(&self, word: u32, lanes: u32) -> bool {
        let p = (word >> 11) as usize;
        self.batch.borrow().is_none()
            && (0..2).filter(|l| lanes >> l & 1 != 0).all(|l| {
                let u = 2 * p + l;
                self.shared.done[self.owner[u].get() as usize].0.load(Ordering::Acquire) >= self.wseq[u].get()
            })
    }

    fn texture_units(&self, t: &Sampler) -> Units {
        if let Some((s, p)) = self.textures.get()
            && s == *t
        {
            return p;
        }
        let p = t.units();
        self.textures.set(Some((*t, p)));
        p
    }

    /// The CLUT slot a primitive reads: the current one, or the buffer copied into the next once its readers are done.
    fn clut_for(&self, clut: &Clut) -> usize {
        let slot = self.clut_slot.get();
        if self.clut_dirty.replace(false) {
            // SAFETY: the slot is only read by workers; comparing reads it as they do.
            let same = unsafe { (*self.shared.cluts[slot].get()).entries == clut.entries };
            if !same {
                let next = (slot + 1) % CLUTS;
                if self.clut_use[next].get() > self.seen_min.get() {
                    self.wait_all(self.clut_use[next].get(), Cause::Clut);
                }
                // SAFETY: every primitive that read slot `next` is drawn, and no later one names it yet.
                unsafe { *self.shared.cluts[next].get() = clut.clone() };
                self.clut_slot.set(next);
                self.tally(|c| c.clut_copies += 1);
                return next;
            }
        }
        slot
    }

    /// Worker `w` is to see worker `o` past job `t` before it runs: unless `o` is `w`, or the GS's thread has already
    /// seen it there (its Acquire, then the publication's Release, order it before the worker).
    fn need(&self, need: &mut [[u64; MAX]; MAX], w: usize, o: usize, t: u64) {
        if o != w && t > self.seen[o].get() {
            need[w][o] = need[w][o].max(t);
        }
    }

    /// The worker that runs a job alone: the one still changing a lane it touches, else the next in turn.
    fn choose(&self, changes: &Units, reads: &Units) -> usize {
        for u in changes.pages().each().chain(reads.each()) {
            let o = self.owner[u].get() as usize;
            if self.wseq[u].get() > self.seen[o].get() {
                return o;
            }
        }
        let r = (self.rotate.get() + 1) % self.n;
        self.rotate.set(r);
        r
    }

    /// Publishes a job whose workers change the lanes `sets` and read `reads`, with the dependencies the module's
    /// comment gives.
    fn publish(&self, job: Job, split: Split, sets: &[Units; MAX], reads: &Units) {
        let n = self.n;
        let alone = match split {
            Split::One(k) => Some(k as usize),
            Split::Rows(_) => None,
        };
        let mut need = [[0u64; MAX]; MAX];
        for (w, set) in sets[..n].iter().enumerate() {
            for u in set.pages().each() {
                self.need(&mut need, w, self.owner[u].get() as usize, self.wseq[u].get());
            }
            for u in set.each() {
                if self.rseq[u].get() > self.seen_min.get() {
                    for o in 0..n {
                        self.need(&mut need, w, o, self.rseq[u].get());
                    }
                }
            }
        }
        for u in reads.each() {
            for w in alone.map_or(0..n, |k| k..k + 1) {
                self.need(&mut need, w, self.owner[u].get() as usize, self.wseq[u].get());
            }
        }
        let s = self.published.get() + 1;
        if s > RING && s - RING > self.seen_min.get() {
            self.wait_all(s - RING, Cause::Ring);
        }
        // SAFETY: every worker has run job s - RING, the last to use this slot.
        unsafe { (*self.shared.ring[(s % RING) as usize].get()).write(Command { job, split, need }) };
        self.published.set(s);
        self.shared.published.0.store(s, Ordering::Release);
        fence(Ordering::SeqCst);
        for (w, t) in self.threads.iter().enumerate() {
            if self.shared.sleeping[w].0.load(Ordering::Relaxed) {
                t.thread().unpark();
            }
        }
        for (w, set) in sets[..n].iter().enumerate() {
            for u in set.each() {
                self.owner[u].set(w as u8);
                self.wseq[u].set(s);
            }
        }
        for u in reads.each() {
            self.rseq[u].set(s);
        }
    }

    fn observe(&self) {
        for w in 0..self.n {
            self.seen[w].set(self.shared.done[w].0.load(Ordering::Acquire));
        }
        self.refresh_min();
    }

    /// A drawing kick: the primitive's page lanes found and the primitive published with the jobs it depends on.
    pub fn draw(&self, kind: Kind, env: Environment, v: [Vertex; 3], clut: &Clut) {
        self.flush();
        let Some((x0, x1, y0, y1)) = raster::bounds(kind, &env, &v) else {
            self.tally(|c| c.empty += 1);
            return;
        };
        let n = self.n;
        self.observe();
        let fpsm = env.frame.psm.expect("bounds holds a frame format");
        let zpsm = env.depth.psm.filter(|_| env.test >> 16 & 1 != 0);
        let height = |p: Psm| p.shape().page().1;
        let shift = height(fpsm).max(zpsm.map_or(0, height)).trailing_zeros();
        // The lanes each worker changes (the frame's, and Z's unless ZMSK), and those every drawing worker may read
        // without changing (the texture's, and Z's under ZMSK).
        let mut sets = [Units::default(); MAX];
        let mut reads = match &env.texture {
            Some(t) => self.texture_units(t),
            None => Units::default(),
        };
        let (xs, ys) = ((x0 as u32, x1 as u32), (y0 as u32, y1 as u32));
        for (psm, bp, changes) in [(Some(fpsm), env.frame.bp, true), (zpsm, env.depth.bp, !env.depth.masked)] {
            let Some(psm) = psm else { continue };
            let ph = height(psm);
            local::cells(psm, bp, env.frame.bw, xs, ys, |cy, p| {
                if changes {
                    sets[((cy * ph) >> shift) as usize % n].add_page(p, lanes(psm));
                } else {
                    reads.add_page(p, lanes(psm));
                }
            });
        }
        // A worker stores whole words, so two workers may not share a page; a reader may share a page with a store
        // that leaves the lane it reads.
        let (mut stored, mut all) = (Units::default(), Units::default());
        let mut overlap = false;
        for s in &sets[..n] {
            let p = s.pages();
            overlap |= stored.meets(&p);
            stored.or(&p);
            all.or(s);
        }
        let reads_own = reads.meets(&all);
        let split = if !overlap && !reads_own {
            Split::Rows(shift)
        } else {
            let k = self.choose(&all, &reads);
            sets = [Units::default(); MAX];
            sets[k] = all;
            self.tally(|c| c.alone_why[usize::from(reads_own)] += 1);
            Split::One(k as u32)
        };
        let slot = self.clut_for(clut);
        self.publish(Job::Draw { kind, env, v, clut: slot as u32 }, split, &sets, &reads);
        self.clut_use[slot].set(self.published.get());
        self.tally(|c| {
            c.primitives += 1;
            if matches!(split, Split::One(_)) {
                c.alone += 1;
            } else {
                c.split += 1;
            }
        });
    }

    /// A host-to-local transfer's unit, `from` being the transfer before it: kept for a worker to write, 32 to a job.
    pub fn upload(&self, from: &Transfer, unit: u64) {
        let full = {
            let mut b = self.batch.borrow_mut();
            let b = b.get_or_insert_with(|| Batch { from: from.clone(), units: [0; BATCH], n: 0 });
            b.units[b.n] = unit;
            b.n += 1;
            b.n == BATCH
        };
        if full {
            self.flush();
        }
    }

    /// The units kept, published as one job for one worker, which changes the lanes of the transfer's destination; before
    /// anything else the GS does, and before a transfer starts or ends.
    pub fn flush(&self) {
        let Some(b) = self.batch.borrow_mut().take() else { return };
        let Some((buf, xs, ys)) = b.from.destination() else { return };
        self.observe();
        let all = local::units(buf.psm, buf.bp, buf.bw, xs, ys);
        let k = self.choose(&all, &Units::default());
        let mut sets = [Units::default(); MAX];
        sets[k] = all;
        self.publish(Job::Upload { from: b.from, units: b.units, n: b.n as u32 }, Split::One(k as u32), &sets, &Units::default());
        self.tally(|c| c.uploads += 1);
    }
}

impl Drop for Workers {
    fn drop(&mut self) {
        self.flush();
        self.shared.stop.store(true, Ordering::SeqCst);
        for t in &self.threads {
            t.thread().unpark();
        }
        for t in self.threads.drain(..) {
            let _ = t.join();
        }
    }
}

/// A worker: every job in order, its own rows of each, its count stored after each.
fn run(shared: &Shared, w: usize, n: usize, local: &mut Local) {
    let mut next = 1u64;
    loop {
        let mut spins = 0u32;
        let end = loop {
            let end = shared.published.0.load(Ordering::Acquire);
            if end >= next {
                break end;
            }
            if shared.stop.load(Ordering::Acquire) {
                return;
            }
            spins += 1;
            if spins < SPIN {
                std::hint::spin_loop();
                continue;
            }
            shared.sleeping[w].0.store(true, Ordering::Relaxed);
            fence(Ordering::SeqCst);
            if shared.published.0.load(Ordering::Relaxed) < next && !shared.stop.load(Ordering::Relaxed) {
                std::thread::park();
            }
            shared.sleeping[w].0.store(false, Ordering::Relaxed);
            spins = 0;
        };
        let t0 = Instant::now();
        while next <= end {
            while shared.held.load(Ordering::Acquire) >> w & 1 != 0 {
                if shared.stop.load(Ordering::Acquire) {
                    return;
                }
                std::thread::yield_now();
            }
            // SAFETY: published with Release before `end` was loaded with Acquire; not rewritten until this worker's
            // count passes it.
            let c = unsafe { (*shared.ring[(next % RING) as usize].get()).assume_init_ref() };
            let rows = match c.split {
                Split::Rows(shift) => Some(Rows { shift, n: n as u32, w: w as u32 }),
                Split::One(k) => (k as usize == w).then_some(Rows::ALL),
            };
            if let Some(rows) = rows {
                for (o, &t) in c.need[w][..n].iter().enumerate() {
                    if t > 0 && shared.done[o].0.load(Ordering::Acquire) < t {
                        let s0 = Instant::now();
                        let mut spins = 0u32;
                        while shared.done[o].0.load(Ordering::Acquire) < t {
                            spins += 1;
                            if spins < 4096 {
                                std::hint::spin_loop();
                            } else {
                                std::thread::yield_now();
                            }
                        }
                        shared.stalled[w].0.fetch_add(s0.elapsed().as_nanos() as u64, Ordering::Relaxed);
                    }
                }
                match &c.job {
                    Job::Draw { kind, env, v, clut } => {
                        // SAFETY: the slot is not rewritten until every primitive naming it is drawn.
                        let clut = unsafe { &*shared.cluts[*clut as usize].get() };
                        raster::primitive(*kind, env, local, clut, v, rows);
                    }
                    Job::Upload { from, units, n } => {
                        let mut t = from.clone();
                        for &u in &units[..*n as usize] {
                            t.hwreg(u, local);
                        }
                    }
                }
            }
            shared.done[w].0.store(next, Ordering::Release);
            next += 1;
        }
        shared.busy[w].0.fetch_add(t0.elapsed().as_nanos() as u64, Ordering::Relaxed);
    }
}

/// Holds the workers whose bits are set before their next job, and releases the others.
pub struct Hold(Arc<Shared>);

impl Hold {
    pub fn set(&self, mask: u64) {
        self.0.held.store(mask, Ordering::Release);
    }
}

impl Drop for Hold {
    fn drop(&mut self) {
        self.set(0);
    }
}

/// The local memory as the GS's own thread reaches it: each access first waits for the workers that touch its lanes.
pub struct Mem<'a> {
    pub local: &'a mut Local,
    pub workers: Option<&'a Workers>,
    pub cause: Cause,
}

impl Memory for Mem<'_> {
    #[inline]
    fn read(&mut self, psm: Psm, bp: u32, bw: u32, x: u32, y: u32) -> u32 {
        if let Some(w) = self.workers {
            w.settle(address(psm, bp, bw, x, y).word, lanes(psm), false, self.cause);
        }
        self.local.read(psm, bp, bw, x, y)
    }

    #[inline]
    fn write(&mut self, psm: Psm, bp: u32, bw: u32, x: u32, y: u32, v: u32) {
        if let Some(w) = self.workers {
            w.settle(address(psm, bp, bw, x, y).word, lanes(psm), true, self.cause);
        }
        self.local.write(psm, bp, bw, x, y, v);
    }
}
