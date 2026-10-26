//! The software rasteriser on worker threads (Sapphire_Native.md §50, Sapphire_Plan.md §5.12's lever 3), exact by
//! construction. The GS's thread (the emulation thread) takes each drawing kick as it does on one thread, with the
//! environment of its context at that kick, and publishes the primitive to a ring that every worker reads in order.
//! Worker `w` of `n` draws the rows of the bands of 32 or 64 rows (the taller page height of the frame and Z formats)
//! whose number modulo `n` is `w`, or, when the pages of the primitive's own pixels are not disjoint between the
//! workers or meet the pages its texture reads, the whole primitive alone.
//!
//! The local memory is owned page by page (8 KiB, 512 pages): the GS's thread records, for each page, the worker and
//! the primitive that last wrote it and the last primitive that read it as a texture, and before it publishes a
//! primitive it waits for every earlier primitive that touches the same pages from another worker (a write after a
//! write or a read, a read after a write). Every access of the GS's own thread to the memory (the transfers, the CLUT's
//! loads, the PCRTC's lines, a state, a debugger) waits the same way for the pages it touches, or for every worker. So
//! each word sees its reads and writes in the order the one-thread path makes them, whatever the worker count and the
//! host's scheduling, and the memory after any point the machine can observe is the one-thread path's. The registers
//! (CSR, SIGLBLID and the events FINISH, SIGNAL and LABEL raise) are the GS's thread's alone, so they need no barrier.
//!
//! The CLUT's buffer is copied into one of sixteen slots when a primitive uses it after a load, and a slot is reused
//! only once every primitive that read it is drawn. Every hand-over is one of this module's atomics: a primitive is
//! written into its slot before the count that publishes it is stored with Release, and a worker's count is stored with
//! Release after its last access to a primitive, the memory and the slot, and loaded with Acquire before the GS's
//! thread touches what it covered.

use std::cell::{Cell, UnsafeCell};
use std::mem::MaybeUninit;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering, fence};
use std::thread::JoinHandle;
use std::time::Instant;

use super::draw::{Environment, Kind, Vertex};
use super::local::{self, Local, PAGES, Pages, address};
use super::psm::Psm;
use super::raster::{self, Rows};
use super::texture::{Clut, Sampler};

/// The most workers a GS runs.
pub const MAX: usize = 8;
/// Primitives published and not yet drawn by every worker, at most.
const RING: u64 = 4096;
/// CLUT buffers in use at once, at most.
const CLUTS: usize = 16;
/// Rounds a worker spins on an empty ring before it parks.
const SPIN: u32 = 2000;

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
    /// One worker the whole primitive.
    One(u32),
}

#[derive(Clone, Copy, Debug)]
struct Command {
    kind: Kind,
    env: Environment,
    v: [Vertex; 3],
    clut: u32,
    split: Split,
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
    /// Workers held before their next primitive, by bit: the wait-site tests' lever.
    held: AtomicU64,
    stop: AtomicBool,
}

// SAFETY: the ring's and the CLUT slots' cells are written by the GS's thread only while no worker can read them, as the
// module's comment argues; everything else is atomic.
unsafe impl Sync for Shared {}
unsafe impl Send for Shared {}

/// What the GS's thread waited for, and how long, for the bench and the tests.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub primitives: u64,
    pub split: u64,
    pub alone: u64,
    pub empty: u64,
    pub clut_copies: u64,
    /// Waits before a primitive (its pages), for a ring slot, for a CLUT slot, at a read or write of the GS's thread,
    /// and joins of every worker: each a count and nanoseconds.
    pub waits: [(u64, u64); 5],
    /// Each worker's nanoseconds drawing.
    pub busy: [u64; MAX],
}

#[derive(Clone, Copy)]
enum Cause {
    Draw,
    Ring,
    Clut,
    Access,
    Join,
}

pub struct Workers {
    n: usize,
    shared: Arc<Shared>,
    threads: Vec<JoinHandle<()>>,
    published: u64,
    owner: Box<[u8; PAGES]>,
    wseq: Box<[u64; PAGES]>,
    rseq: Box<[u64; PAGES]>,
    seen: [Cell<u64>; MAX],
    seen_min: Cell<u64>,
    clut_slot: usize,
    clut_use: [u64; CLUTS],
    /// The CLUT's buffer changed since it was last copied to a slot.
    pub(super) clut_dirty: bool,
    textures: Option<(Sampler, Pages)>,
    rotate: usize,
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
            published: 0,
            owner: Box::new([0; PAGES]),
            wseq: Box::new([0; PAGES]),
            rseq: Box::new([0; PAGES]),
            seen: std::array::from_fn(|_| Cell::new(0)),
            seen_min: Cell::new(0),
            clut_slot: 0,
            clut_use: [0; CLUTS],
            clut_dirty: false,
            textures: None,
            rotate: 0,
            counts: Cell::new(Counts::default()),
        }
    }

    pub fn count(&self) -> usize {
        self.n
    }

    /// Primitives published that some worker has not yet drawn.
    pub fn pending(&self) -> u64 {
        (0..self.n).map(|w| self.published - self.shared.done[w].0.load(Ordering::Acquire).min(self.published)).max().unwrap_or(0)
    }

    /// A handle that holds and releases workers from any thread, for the tests.
    pub fn hold(&self) -> Hold {
        Hold(Arc::clone(&self.shared))
    }

    pub fn counts(&self) -> Counts {
        let mut c = self.counts.get();
        for w in 0..self.n {
            c.busy[w] = self.shared.busy[w].0.load(Ordering::Relaxed);
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

    /// Waits until worker `w` has drawn every primitive up to `target`.
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

    /// Waits for every primitive published: the memory is then the GS's thread's alone until the next one.
    pub fn join(&self) {
        self.wait_all(self.published, Cause::Join);
    }

    /// Before the GS's thread reads (or writes) the word `word`: the workers that write its page (or read it) are waited
    /// for up to their last primitive touching it.
    #[inline]
    pub fn settle(&self, word: u32, write: bool) {
        let p = (word >> 11) as usize;
        let o = self.owner[p] as usize;
        if self.wseq[p] > self.seen[o].get() {
            self.wait(o, self.wseq[p], Cause::Access);
        }
        if write && self.rseq[p] > self.seen_min.get() {
            self.wait_all(self.rseq[p], Cause::Access);
        }
    }

    /// `settle` for every page of a set.
    pub fn settle_pages(&self, pages: &Pages, write: bool) {
        for p in pages.each() {
            self.settle((p as u32) << 11, write);
        }
    }

    /// Whether the GS's thread may read (or write) the word now without waiting: the verifier's question.
    pub fn settled(&self, word: u32, write: bool) -> bool {
        let p = (word >> 11) as usize;
        let done = |w: usize| self.shared.done[w].0.load(Ordering::Acquire);
        done(self.owner[p] as usize) >= self.wseq[p] && (!write || (0..self.n).all(|w| done(w) >= self.rseq[p]))
    }

    fn texture_pages(&mut self, t: &Sampler) -> Pages {
        if let Some((s, p)) = &self.textures
            && s == t
        {
            return *p;
        }
        let p = t.pages();
        self.textures = Some((*t, p));
        p
    }

    /// The CLUT slot a primitive reads: the current one, or the buffer copied into the next once its readers are done.
    fn clut_for(&mut self, clut: &Clut) -> usize {
        if self.clut_dirty {
            self.clut_dirty = false;
            // SAFETY: the slot is only read by workers; comparing reads it as they do.
            let same = unsafe { (*self.shared.cluts[self.clut_slot].get()).entries == clut.entries };
            if !same {
                let next = (self.clut_slot + 1) % CLUTS;
                if self.clut_use[next] > self.seen_min.get() {
                    self.wait_all(self.clut_use[next], Cause::Clut);
                }
                // SAFETY: every primitive that read slot `next` is drawn, and no later one names it yet.
                unsafe { *self.shared.cluts[next].get() = clut.clone() };
                self.clut_slot = next;
                self.tally(|c| c.clut_copies += 1);
            }
        }
        self.clut_slot
    }

    /// Before worker `w` writes page `p`: the other worker that wrote it last, and every reader of it, are waited for.
    fn before_write(&self, p: usize, w: usize) {
        let o = self.owner[p] as usize;
        if o != w && self.wseq[p] > self.seen[o].get() {
            self.wait(o, self.wseq[p], Cause::Draw);
        }
        if self.rseq[p] > self.seen_min.get() {
            self.wait_all(self.rseq[p], Cause::Draw);
        }
    }

    /// Before page `p` is read as a texture by every worker, or by `alone` only: its last writer is waited for, unless
    /// that writer is the only reader.
    fn before_read(&self, p: usize, alone: Option<usize>) {
        let o = self.owner[p] as usize;
        if alone != Some(o) && self.wseq[p] > self.seen[o].get() {
            self.wait(o, self.wseq[p], Cause::Draw);
        }
    }

    /// The worker that draws a primitive alone: the one still writing a page it touches, else the next in turn.
    fn choose(&mut self, pages: &Pages, tex: &Pages) -> usize {
        for p in pages.each().chain(tex.each()) {
            let o = self.owner[p] as usize;
            if self.wseq[p] > self.seen[o].get() {
                return o;
            }
        }
        self.rotate = (self.rotate + 1) % self.n;
        self.rotate
    }

    /// A drawing kick: the primitive's pages found, the earlier primitives it depends on waited for, and the primitive
    /// published for the workers.
    pub fn draw(&mut self, kind: Kind, env: Environment, v: [Vertex; 3], clut: &Clut) {
        let Some((x0, x1, y0, y1)) = raster::bounds(kind, &env, &v) else {
            self.tally(|c| c.empty += 1);
            return;
        };
        let n = self.n;
        let fpsm = env.frame.psm.expect("bounds holds a frame format");
        let zpsm = env.depth.psm.filter(|_| env.test >> 16 & 1 != 0);
        let height = |p: Psm| p.shape().page().1;
        let shift = height(fpsm).max(zpsm.map_or(0, height)).trailing_zeros();
        let mut sets = [Pages::default(); MAX];
        let (xs, ys) = ((x0 as u32, x1 as u32), (y0 as u32, y1 as u32));
        for (psm, bp) in [(Some(fpsm), env.frame.bp), (zpsm, env.depth.bp)] {
            let Some(psm) = psm else { continue };
            let ph = height(psm);
            local::cells(psm, bp, env.frame.bw, xs, ys, |cy, p| sets[((cy * ph) >> shift) as usize % n].add(p));
        }
        let mut all = Pages::default();
        let mut overlap = false;
        for s in &sets[..n] {
            overlap |= all.meets(s);
            all.or(s);
        }
        let tex = match &env.texture {
            Some(t) => self.texture_pages(t),
            None => Pages::default(),
        };
        let (split, alone) = if !overlap && !tex.meets(&all) {
            (Split::Rows(shift), None)
        } else {
            let k = self.choose(&all, &tex);
            sets = [Pages::default(); MAX];
            sets[k] = all;
            (Split::One(k as u32), Some(k))
        };
        for (w, s) in sets[..n].iter().enumerate() {
            for p in s.each() {
                self.before_write(p, w);
            }
        }
        for p in tex.each() {
            self.before_read(p, alone);
        }
        let slot = self.clut_for(clut);
        let s = self.published + 1;
        if s > RING && s - RING > self.seen_min.get() {
            self.wait_all(s - RING, Cause::Ring);
        }
        // SAFETY: every worker has drawn primitive s - RING, the last to use this slot.
        unsafe { (*self.shared.ring[(s % RING) as usize].get()).write(Command { kind, env, v, clut: slot as u32, split }) };
        self.published = s;
        self.shared.published.0.store(s, Ordering::Release);
        fence(Ordering::SeqCst);
        for (w, t) in self.threads.iter().enumerate() {
            if self.shared.sleeping[w].0.load(Ordering::Relaxed) {
                t.thread().unpark();
            }
        }
        for (w, set) in sets[..n].iter().enumerate() {
            for p in set.each() {
                self.owner[p] = w as u8;
                self.wseq[p] = s;
            }
        }
        for p in tex.each() {
            self.rseq[p] = s;
        }
        self.clut_use[slot] = s;
        self.tally(|c| {
            c.primitives += 1;
            if alone.is_some() {
                c.alone += 1;
            } else {
                c.split += 1;
            }
        });
    }
}

impl Drop for Workers {
    fn drop(&mut self) {
        self.shared.stop.store(true, Ordering::SeqCst);
        for t in &self.threads {
            t.thread().unpark();
        }
        for t in self.threads.drain(..) {
            let _ = t.join();
        }
    }
}

/// A worker: every primitive in order, its own rows of each, its count stored after each.
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
            // SAFETY: the slot is not rewritten until every primitive naming it is drawn.
            let clut = unsafe { &*shared.cluts[c.clut as usize].get() };
            let rows = match c.split {
                Split::Rows(shift) => Some(Rows { shift, n: n as u32, w: w as u32 }),
                Split::One(k) => (k as usize == w).then_some(Rows::ALL),
            };
            if let Some(rows) = rows {
                raster::primitive(c.kind, &c.env, local, clut, &c.v, rows);
            }
            shared.done[w].0.store(next, Ordering::Release);
            next += 1;
        }
        shared.busy[w].0.fetch_add(t0.elapsed().as_nanos() as u64, Ordering::Relaxed);
    }
}

/// Holds the workers whose bits are set before their next primitive, and releases the others.
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

/// The local memory as the GS's own thread reaches it: each access first waits for the workers that touch its page.
pub struct Mem<'a> {
    pub local: &'a mut Local,
    pub workers: Option<&'a Workers>,
}

impl Mem<'_> {
    #[inline]
    pub fn read(&mut self, psm: Psm, bp: u32, bw: u32, x: u32, y: u32) -> u32 {
        if let Some(w) = self.workers {
            w.settle(address(psm, bp, bw, x, y).word, false);
        }
        self.local.read(psm, bp, bw, x, y)
    }

    #[inline]
    pub fn write(&mut self, psm: Psm, bp: u32, bw: u32, x: u32, y: u32, v: u32) {
        if let Some(w) = self.workers {
            w.settle(address(psm, bp, bw, x, y).word, true);
        }
        self.local.write(psm, bp, bw, x, y, v);
    }
}
