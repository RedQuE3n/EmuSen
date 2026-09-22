//! The display processor's list run on a thread of its own: C#'s threaded `DpInterface`, its ring, marks, ranges, boxes and verifier. See Mars_Native.md §5.6.
//!
//! The rule: between a word's publish and the join, the drain owns the processor and may touch the RDRAM bytes the word's marks name; the
//! emulation thread touches such a byte only after an Acquire of `completed` at or past the word that last touches it (`wait`).

use std::cell::Cell;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::Ordering::{AcqRel, Acquire, Relaxed, Release, SeqCst};
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicI64, AtomicU32, AtomicU64, fence};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle, Thread};
use std::time::Instant;

use crate::memory::ram::{Detached, Ram};
use crate::rdp::{self, Rdp, RdpMemory, Step, command_id, command_length};

/// `_ring.Length`: the words handed over and not yet run.
pub const RING: usize = 1 << 16;
/// `RangeCount`, `_idleRanges.Length`, `BoxCount`.
const RANGE_COUNT: usize = 1 << 13;
const IDLE_RANGES: usize = 256;
const BOX_COUNT: usize = 1 << 14;
/// Pages of the largest RDRAM, 8 MB in 4 KB.
pub const PAGES: usize = 2048;
/// `Idle`: an image's mark while its batch is open, everything handed over so far.
pub const IDLE: i64 = i64::MAX;
/// `SnapshotWords`, as `dp::SNAPSHOT_WORDS`.
const SNAPSHOT_WORDS: i64 = crate::memory::dp::SNAPSHOT_WORDS as i64;

/// The wait sites of C#'s `DpInterface`, by number.
pub mod site {
    pub const BUS_READ: usize = 0;
    pub const BUS_WRITE: usize = 1;
    pub const LOAD: usize = 2;
    pub const STORE: usize = 3;
    pub const FETCH: usize = 4;
    pub const BLOCK: usize = 5;
    pub const SI: usize = 6;
    pub const AI: usize = 7;
    pub const VI: usize = 8;
    pub const CHEAT: usize = 9;
    pub const TAKE: usize = 10;
    pub const SP_DMA: usize = 11;
    pub const COUNT: usize = 12;
}

/// `_marks` and `_writeMarks`: the count a writer of each page waits for, and the count a reader waits for; zero when nothing is due.
pub struct Marks {
    pub marks: [AtomicI64; PAGES],
    pub write_marks: [AtomicI64; PAGES],
}

impl Marks {
    fn clear(&self) {
        for m in self.marks.iter().chain(self.write_marks.iter()) {
            m.store(0, Relaxed);
        }
    }
}

/// The marks the fast paths read, all zero while unthreaded; a clone gets its own, since a clone is never threaded.
pub struct PageMarks(pub Arc<Marks>);

impl Default for PageMarks {
    fn default() -> Self {
        PageMarks(Arc::new(Marks { marks: [const { AtomicI64::new(0) }; PAGES], write_marks: [const { AtomicI64::new(0) }; PAGES] }))
    }
}

impl std::ops::Deref for PageMarks {
    type Target = Marks;
    #[inline(always)]
    fn deref(&self) -> &Marks {
        &self.0
    }
}

impl Clone for PageMarks {
    fn clone(&self) -> Self {
        PageMarks::default()
    }
}

impl std::fmt::Debug for PageMarks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PageMarks")
    }
}

/// One of `_ranges`: its bytes, the word that first reaches them, the count that frees them, and whether the processor writes them.
#[derive(Default)]
struct RangeSlot {
    from: AtomicI64,
    to: AtomicI64,
    start: AtomicI64,
    mark: AtomicI64,
    write: AtomicBool,
}

#[derive(Default)]
struct IdleSlot {
    from: AtomicI64,
    to: AtomicI64,
    start: AtomicI64,
}

const EMPTY: i32 = 0;
const RECTANGLE: i32 = 1;
const TRIANGLE: i32 = 2;
const WHOLE_ROWS: i32 = 3;

/// One of `_boxes`, stored field by field so the verifier can read it while the machine writes the next.
#[derive(Default)]
struct BoxSlot {
    end: AtomicI64,
    color: AtomicU32,
    depth: AtomicU32,
    bytes: AtomicI32,
    width: AtomicI32,
    scissor_right: AtomicI32,
    row_first: AtomicI32,
    row_last: AtomicI32,
    kind: AtomicI32,
    first: AtomicU64,
    low: AtomicU64,
    high: AtomicU64,
    middle: AtomicU64,
}

/// C#'s `Box`, read out of its slot.
#[derive(Clone, Copy)]
struct DrawBox {
    end: i64,
    color: u32,
    depth: u32,
    bytes: i32,
    width: i32,
    scissor_right: i32,
    row_first: i32,
    row_last: i32,
    kind: i32,
    first: u64,
    low: u64,
    high: u64,
    middle: u64,
}

impl BoxSlot {
    fn load(&self) -> DrawBox {
        DrawBox {
            end: self.end.load(Relaxed),
            color: self.color.load(Relaxed),
            depth: self.depth.load(Relaxed),
            bytes: self.bytes.load(Relaxed),
            width: self.width.load(Relaxed),
            scissor_right: self.scissor_right.load(Relaxed),
            row_first: self.row_first.load(Relaxed),
            row_last: self.row_last.load(Relaxed),
            kind: self.kind.load(Relaxed),
            first: self.first.load(Relaxed),
            low: self.low.load(Relaxed),
            high: self.high.load(Relaxed),
            middle: self.middle.load(Relaxed),
        }
    }
}

/// A ring range that covered a byte: its bytes, its first word and its count, and whether the processor writes it.
#[derive(Clone, Copy)]
struct Covering {
    from: i64,
    to: i64,
    start: i64,
    mark: i64,
    write: bool,
}

/// The drain's memory of the last range and box that held a byte, which accepts only what the full scan would; a draw's bytes are neighbours.
#[derive(Default)]
pub struct Checked {
    range: Cell<Option<Covering>>,
    draw: Cell<Option<DrawBox>>,
}

/// One processor's thread as the others see it: how far it has come, where and for which pause it stands, whether it sleeps. See Mars_Rdp.md §2.8.
struct Worker {
    completed: AtomicI64,
    standing: AtomicU64,
    standing_at: AtomicI64,
    sleeping: AtomicBool,
    processor: *mut Rdp,
    words: AtomicI64,
}

/// `SpinBarrier`: all arrive before any leaves, a fault lets all through, and a waiter raises a pause point to its word (Mars_Native.md §5.6.6).
struct Barrier {
    parties: u32,
    arrived: AtomicU32,
    generation: AtomicU32,
    passed: AtomicI64,
}

impl Barrier {
    fn arrive(&self, shared: &Shared, word: i64) {
        let generation = self.generation.load(Acquire);
        if self.arrived.fetch_add(1, AcqRel) + 1 == self.parties {
            self.passed.fetch_add(1, Relaxed);
            self.arrived.store(0, Relaxed);
            self.generation.store(generation.wrapping_add(1), Release);
            return;
        }
        let mut spins = 0;
        while self.generation.load(Acquire) == generation && !shared.faulted.load(Acquire) {
            if shared.pause_request.load(Acquire) != 0 {
                raise(&shared.pause_at, word);
            }
            backoff(&mut spins);
        }
    }
}

/// What the machine's thread and the workers all reach. Its raw pointers are the bus's own allocations, and the processors', which outlive the workers (`Threads::stop`).
pub struct Shared {
    ring: Box<[AtomicU64]>,
    issued: AtomicI64,
    workers: Box<[Worker]>,
    barrier: Barrier,
    /// A pause's number while one is asked for, else zero; the workers answer with it where they stand, at the one word `pause_at` all reach.
    pause_request: AtomicU64,
    pause_at: AtomicI64,
    pauses: AtomicU64,
    /// Holds nest: a snapshot sized and then written holds twice, and only the last resume lets the workers go.
    holds: AtomicU32,
    stopping: AtomicBool,
    faulted: AtomicBool,
    fault: Mutex<Option<String>>,
    marks: Arc<Marks>,
    ranges: Box<[RangeSlot]>,
    ranges_appended: AtomicI64,
    idle: Box<[IdleSlot]>,
    idle_count: AtomicI32,
    idle_sequence: AtomicU32,
    boxes: Box<[BoxSlot]>,
    boxes_appended: AtomicI64,
    verifying: AtomicBool,
    rdram: *mut u8,
    rdram_len: usize,
    hidden: *mut u8,
    hidden_len: usize,
    pub drain_words: AtomicI64,
    pub drain_nanos: AtomicI64,
    pub drain_starts: AtomicI64,
}

// SAFETY: the raw pointers name the bus's RDRAM, hidden bits and processor, which the drain touches only under the module's rule.
unsafe impl Send for Shared {}
unsafe impl Sync for Shared {}

impl Shared {
    /// `Completed`: the least of the workers' counts, since a word is done when every processor has run it.
    #[inline(always)]
    fn completed(&self) -> i64 {
        let mut least = self.workers[0].completed.load(Acquire);
        for w in &self.workers[1..] {
            least = least.min(w.completed.load(Acquire));
        }
        least
    }

    /// Words, barriers passed, and aliased reads, by worker, for the counters.
    pub fn worker_words(&self) -> Vec<i64> {
        self.workers.iter().map(|w| w.words.load(Relaxed)).collect()
    }

    pub fn barriers(&self) -> i64 {
        self.barrier.passed.load(Relaxed)
    }

    fn record_fault(&self, message: String) {
        let mut fault = self.fault.lock().unwrap_or_else(|e| e.into_inner());
        if fault.is_none() {
            *fault = Some(message);
        }
        self.faulted.store(true, Release);
    }

    /// `Touched` and `Wrote`: a byte the processor reaches for a word must lie in a page and a range marked for it, and a write in its draw's box.
    #[cfg(test)]
    pub(crate) fn verify(&self, at: usize, word: i64, write: bool) {
        self.verify_with(at, word, write, &Checked::default());
    }

    pub(crate) fn verify_with(&self, at: usize, word: i64, write: bool, checked: &Checked) {
        if at >= self.rdram_len || self.faulted.load(Relaxed) {
            return;
        }
        let page = at >> 12;
        let mark = if write { self.marks.write_marks[page].load(Acquire) } else { self.marks.marks[page].load(Acquire) };
        let what = if write { "wrote" } else { "read" };
        if mark < word {
            self.record_fault(format!("the display processor {what} {at:06X} in a page its interface did not mark before word {word}"));
            return;
        }
        let byte = at as i64;
        let remembered = checked.range.get().is_some_and(|r| r.start <= word && word <= r.mark && byte >= r.from && byte < r.to && (!write || r.write));
        if !remembered {
            match self.covers(byte, word, write) {
                None => {
                    self.record_fault(format!("the display processor {what} {at:06X} outside every range its interface marked for word {word}"));
                    return;
                }
                Some(found) => {
                    if found.is_some() {
                        checked.range.set(found);
                    }
                }
            }
        }
        if write {
            let draw = match checked.draw.get() {
                Some(b) if b.end == word => Some(b),
                _ => self.box_for(word),
            };
            if let Some(b) = draw {
                checked.draw.set(Some(b));
                if !holds(&b, byte, byte + 1) {
                    self.record_fault(format!("the display processor wrote {at:06X} outside the box its interface recorded for the draw ending at word {word}"));
                }
            }
        }
    }

    /// `Covers`: the open batch read whole under its sequence, then the ring newest first until the counts fall below the word; the ring range that covered, if one did.
    fn covers(&self, at: i64, word: i64, write: bool) -> Option<Option<Covering>> {
        loop {
            let sequence = self.idle_sequence.load(Acquire);
            if sequence & 1 != 0 {
                std::hint::spin_loop();
                continue;
            }
            let count = self.idle_count.load(Relaxed);
            let mut covered = count as usize > IDLE_RANGES;
            let mut i = 0;
            while i < count.max(0) as usize && i < IDLE_RANGES && !covered {
                let slot = &self.idle[i];
                covered = slot.start.load(Relaxed) <= word && at >= slot.from.load(Relaxed) && at < slot.to.load(Relaxed);
                i += 1;
            }
            fence(Acquire);
            if self.idle_sequence.load(Relaxed) != sequence {
                continue;
            }
            if covered {
                return Some(None);
            }
            break;
        }

        let appended = self.ranges_appended.load(Acquire);
        let mut i = appended - 1;
        while i >= (appended - RANGE_COUNT as i64).max(0) {
            let slot = &self.ranges[(i as usize) & (RANGE_COUNT - 1)];
            let (from, to, start, mark, writes) = (slot.from.load(Relaxed), slot.to.load(Relaxed), slot.start.load(Relaxed), slot.mark.load(Relaxed), slot.write.load(Relaxed));
            fence(Acquire);
            if self.ranges_appended.load(Relaxed) - i >= RANGE_COUNT as i64 {
                return Some(None);
            }
            if mark < word {
                return None;
            }
            if start <= word && at >= from && at < to && (!write || writes) {
                return Some(Some(Covering { from, to, start, mark, write: writes }));
            }
            i -= 1;
        }
        (appended >= RANGE_COUNT as i64).then_some(None)
    }

    /// `BoxHolds`' box: the draw ending at the word, found by halving, since ends are unique and grow with the index; none is not the boxes' to judge.
    fn box_for(&self, word: i64) -> Option<DrawBox> {
        let appended = self.boxes_appended.load(Acquire);
        let oldest = (appended - BOX_COUNT as i64).max(0);
        let (mut low, mut high) = (oldest, appended);
        while low < high {
            let middle = low + (high - low) / 2;
            if self.boxes[(middle as usize) & (BOX_COUNT - 1)].end.load(Relaxed) <= word {
                low = middle + 1;
            } else {
                high = middle;
            }
        }
        if low == oldest {
            return None;
        }
        let i = low - 1;
        let b = self.boxes[(i as usize) & (BOX_COUNT - 1)].load();
        fence(Acquire);
        if self.boxes_appended.load(Relaxed) - i > BOX_COUNT as i64 || b.end != word {
            return None;
        }
        Some(b)
    }
}

#[inline(always)]
fn sign_extend14(v: u64) -> i32 {
    (((v & 0x3FFF) as i32) << 18) >> 18
}

#[inline(always)]
fn signed(v: u64, bits: u32) -> i64 {
    (((v & ((1u64 << bits) - 1)) as i64) << (64 - bits)) >> (64 - bits)
}

/// A triangle's edges as the walker steps them: each edge's start and step a sub-scanline, the sub-scanlines it spans; none when unsure.
#[derive(Clone, Copy)]
struct Edges {
    xh: i64,
    xm: i64,
    xl: i64,
    sh: i64,
    sm: i64,
    sl: i64,
    top: i64,
    bottom: i64,
    ym: i64,
}

fn edges(b: &DrawBox) -> Option<Edges> {
    let (xl, dl) = (signed(b.low >> 32, 28), signed(b.low, 30));
    let (xh, dh) = (signed(b.high >> 32, 28), signed(b.high, 30));
    let (xm, dm) = (signed(b.middle >> 32, 28), signed(b.middle, 30));
    let (yh, ym) = (sign_extend14(b.first), sign_extend14(b.first >> 16));
    let top = (yh & !3) as i64;
    let bottom = (b.row_last | 3) as i64;
    let span = (bottom - top).max(0);
    let (sh, sm, sl) = ((dh >> 2) & !1, (dm >> 2) & !1, (dl >> 2) & !1);
    if (xh + sh * span).abs() >= (1i64 << 31) {
        return None;
    }
    Some(Edges { xh, xm, xl, sh, sm, sl, top, bottom, ym: ym as i64 })
}

/// `Reach`: the columns a triangle's sub-scanlines can name between two sub-scanlines, from its edges' two ends there.
fn reach(e: &Edges, from: i64, to: i64) -> (i64, i64) {
    let (mut low, mut high) = (i64::MAX, i64::MIN);
    let mut edge = |x: i64, step: i64, first: i64, last: i64| {
        let a = x.wrapping_add(step.wrapping_mul(first)) & !1;
        let b = x.wrapping_add(step.wrapping_mul(last)) & !1;
        low = low.min(a.min(b));
        high = high.max(a.max(b));
    };
    edge(e.xh, e.sh, from - e.top, to - e.top);
    if e.ym >= e.top && e.ym <= e.bottom {
        if from <= e.ym {
            edge(e.xm, e.sm, from - e.top, to.min(e.ym) - e.top);
        }
        if to >= e.ym {
            edge(e.xl, e.sl, from.max(e.ym) - e.ym, to - e.ym);
        }
    } else {
        edge(e.xm, e.sm, from - e.top, to - e.top);
    }
    (low, high)
}

/// `Row`: the columns a box's draw can reach on one row, a pixel of slack each way and the scissor's reach past the row kept.
fn row(b: &DrawBox, row: i64) -> Option<(i64, i64)> {
    if b.kind == EMPTY {
        return None;
    }
    let whole = Some((-3, b.scissor_right as i64 + 3));
    if b.kind == TRIANGLE {
        let (first_row, last_row) = ((b.row_first >> 2) as i64, ((b.row_last - 1) >> 2) as i64);
        if row < first_row || row > last_row {
            return None;
        }
        let Some(e) = edges(b) else { return whole };
        let (low, high) = reach(&e, (row * 4).max(e.top), (row * 4 + 3).min(e.bottom));
        if low < 0 || high >= (1i64 << 27) {
            return whole;
        }
        return Some(((low >> 16) - 4, ((high >> 16) + 5).min(b.scissor_right as i64) + 3));
    }
    if row < b.row_first as i64 || row > b.row_last as i64 {
        return None;
    }
    if b.kind == WHOLE_ROWS {
        return whole;
    }
    let left = (((b.first >> 12) & 0xFFF) as i64 >> 2) - 3;
    let right = ((((b.first >> 44) & 0xFFF) as i64 >> 2) + 2).min(b.scissor_right as i64) + 3;
    Some((left, right))
}

/// `Holds`: whether any byte between two addresses is a pixel the box's draw can reach, a span running on into the next row included.
fn holds(b: &DrawBox, from: i64, to: i64) -> bool {
    within(b, b.color, b.bytes, from, to) || (b.depth != u32::MAX && within(b, b.depth, 2, from, to))
}

fn within(b: &DrawBox, image: u32, bytes: i32, from: i64, to: i64) -> bool {
    let (image, bytes, width) = (image as i64, bytes.max(1) as i64, b.width.max(1) as i64);
    let mut at = from;
    while at < to {
        if at >= image {
            let pixel = (at - image) / bytes;
            let r0 = pixel / width;
            let column = pixel - r0 * width;
            for r in r0 - 1..=r0 + 1 {
                let c = column + (r0 - r) * width;
                if let Some((left, right)) = row(b, r)
                    && c >= left
                    && c <= right
                {
                    return true;
                }
            }
        }
        at += bytes;
    }
    false
}

/// C#'s `Extent`: a batch's colour or depth extent, grown in place; -1 none yet, -2 not listed.
#[derive(Clone, Copy, Default)]
struct Extent {
    index: i32,
    first: i64,
    last: i64,
}

/// How often the machine waited, and how long, by kind and by site. See Mars_Performance.md §28.
#[derive(Clone, Copy, Debug, Default)]
pub struct Counters {
    pub page_waits: i64,
    pub page_wait_nanos: i64,
    pub range_waits: i64,
    pub range_wait_nanos: i64,
    pub joins: i64,
    pub join_nanos: i64,
    pub bystanders: i64,
    pub reads_narrowed: i64,
    pub reads_freed: i64,
    pub waits_per_site: [i64; site::COUNT],
    pub nanos_per_site: [i64; site::COUNT],
}

/// The machine's half: the workers' handles and their processors past the first, the shadow of the processor's registers, the extents, and the counters.
pub struct Threads {
    shared: Arc<Shared>,
    drains: Vec<JoinHandle<()>>,
    threads: Vec<Thread>,
    extra: Vec<Detached<Rdp>>,
    issued: i64,
    shadow_taken: i32,
    shadow_first: u64,
    color_image: u32,
    depth_image: u32,
    texture_image: u32,
    color_width: i32,
    color_bytes: i32,
    texture_width: i32,
    texture_size: i32,
    scissor_top: i32,
    scissor_bottom: i32,
    scissor_right: i32,
    drawn: bool,
    color_extent: Extent,
    depth_extent: Extent,
    scissor_top_raw: i32,
    scissor_bottom_raw: i32,
    shadow_cycle: i32,
    gathered_before_the_ring: bool,
    shadow_depth: bool,
    shadow_depth_update: bool,
    batch_start: i64,
    pub(crate) taking: bool,
    waiting: i32,
    pub counters: Counters,
}

impl Threads {
    /// `Threaded = true` and `Workers`: the shadow taken from the processor (`RefreshShadow`), the others copied from it (`CopyStateFrom`), and a thread each.
    pub fn start(processor: &mut Detached<Rdp>, rdram: &Ram, hidden: &Ram, marks: &Arc<Marks>, verify: bool, workers: usize) -> Threads {
        marks.clear();
        let n = workers.clamp(1, 8);
        processor.configure(0, n as i32);
        let extra: Vec<Detached<Rdp>> = (1..n)
            .map(|i| {
                let mut p = Detached::new((**processor).clone());
                p.configure(i as i32, n as i32);
                p
            })
            .collect();
        let pointers: Vec<*mut Rdp> = std::iter::once(processor.as_ptr()).chain(extra.iter().map(|p| p.as_ptr())).collect();
        let shared = Arc::new(Shared {
            ring: (0..RING).map(|_| AtomicU64::new(0)).collect(),
            issued: AtomicI64::new(0),
            workers: pointers
                .iter()
                .map(|&processor| Worker { completed: AtomicI64::new(0), standing: AtomicU64::new(0), standing_at: AtomicI64::new(-1), sleeping: AtomicBool::new(false), processor, words: AtomicI64::new(0) })
                .collect(),
            barrier: Barrier { parties: n as u32, arrived: AtomicU32::new(0), generation: AtomicU32::new(0), passed: AtomicI64::new(0) },
            pause_request: AtomicU64::new(0),
            pause_at: AtomicI64::new(-1),
            pauses: AtomicU64::new(0),
            holds: AtomicU32::new(0),
            stopping: AtomicBool::new(false),
            faulted: AtomicBool::new(false),
            fault: Mutex::new(None),
            marks: marks.clone(),
            ranges: (0..RANGE_COUNT).map(|_| RangeSlot::default()).collect(),
            ranges_appended: AtomicI64::new(0),
            idle: (0..IDLE_RANGES).map(|_| IdleSlot::default()).collect(),
            idle_count: AtomicI32::new(0),
            idle_sequence: AtomicU32::new(0),
            boxes: (0..BOX_COUNT).map(|_| BoxSlot::default()).collect(),
            boxes_appended: AtomicI64::new(0),
            verifying: AtomicBool::new(verify),
            rdram: rdram.as_ptr(),
            rdram_len: rdram.len(),
            hidden: hidden.as_ptr(),
            hidden_len: hidden.len(),
            drain_words: AtomicI64::new(0),
            drain_nanos: AtomicI64::new(0),
            drain_starts: AtomicI64::new(0),
        });
        let drains: Vec<JoinHandle<()>> = (0..n)
            .map(|i| {
                let theirs = shared.clone();
                thread::Builder::new().name(format!("MarsRT RDP {i}")).spawn(move || work(theirs, i)).expect("a worker thread could not start")
            })
            .collect();
        let threads = drains.iter().map(|d| d.thread().clone()).collect();
        let mut threads = Threads {
            shared,
            drains,
            threads,
            extra,
            issued: 0,
            shadow_taken: 0,
            shadow_first: 0,
            color_image: 0,
            depth_image: 0,
            texture_image: 0,
            color_width: 0,
            color_bytes: 0,
            texture_width: 0,
            texture_size: 0,
            scissor_top: 0,
            scissor_bottom: 0,
            scissor_right: 0,
            drawn: false,
            color_extent: Extent { index: -1, ..Default::default() },
            depth_extent: Extent { index: -1, ..Default::default() },
            scissor_top_raw: 0,
            scissor_bottom_raw: 0,
            shadow_cycle: 0,
            gathered_before_the_ring: false,
            shadow_depth: false,
            shadow_depth_update: false,
            batch_start: 0,
            taking: false,
            waiting: 0,
            counters: Counters::default(),
        };
        threads.refresh_shadow(processor);
        threads
    }

    /// `RefreshShadow`: the gathering, the images and the bounds as the processor holds them.
    fn refresh_shadow(&mut self, p: &Rdp) {
        self.shadow_taken = p.taken;
        self.shadow_first = p.command[0];
        self.gathered_before_the_ring = self.shadow_taken > 0;
        self.color_image = p.color_image;
        self.color_width = p.color_image_width;
        self.color_bytes = p.color_image_bytes.max(1);
        self.depth_image = p.depth_image;
        self.texture_image = p.texture_image;
        self.texture_width = p.texture_image_width;
        self.texture_size = p.texture_image_size;
        self.scissor_top = p.scissor_top >> 2;
        self.scissor_bottom = (p.scissor_bottom + 3) >> 2;
        self.scissor_right = (p.scissor_right + 3) >> 2;
        self.drawn = false;
        self.scissor_top_raw = p.scissor_top;
        self.scissor_bottom_raw = p.scissor_bottom;
        let modes = p.other_modes;
        self.shadow_cycle = ((modes >> 52) & 3) as i32;
        self.shadow_depth = ((modes >> 4) & 3) != 0;
        self.shadow_depth_update = ((modes >> 5) & 1) != 0;
        self.color_extent.index = -1;
        self.depth_extent.index = -1;
    }

    pub fn set_verifying(&self, on: bool) {
        self.shared.verifying.store(on, Relaxed);
    }

    pub fn shared(&self) -> &Shared {
        &self.shared
    }

    /// `Pending`: the words handed over that the drain has not run.
    pub fn pending(&self) -> i64 {
        self.issued - self.shared.completed()
    }

    pub fn issued(&self) -> i64 {
        self.issued
    }

    /// A batch begins: what `TakeOntoThread` sets before its first word.
    pub fn begin_batch(&mut self) {
        self.batch_start = self.issued;
        self.drawn = false;
        self.color_extent.index = -1;
        self.depth_extent.index = -1;
        self.taking = true;
    }

    /// `Publish`: the word into the ring, marked for by the shadow, then published; true when it completed a full sync.
    pub fn publish(&mut self, word: u64) -> bool {
        let tail = self.issued;
        if tail - self.shared.completed() >= RING as i64 {
            self.make_room(tail);
        }
        self.shared.ring[(tail as usize) & (RING - 1)].store(word, Relaxed);
        let tail = tail + 1;
        let sync = self.shadow(word, tail);
        self.issued = tail;
        self.shared.issued.store(tail, Release);
        for (w, t) in self.shared.workers.iter().zip(&self.threads) {
            if w.sleeping.load(Relaxed) {
                t.unpark();
            }
        }
        sync
    }

    /// `EndBatch`: the idle marks downgraded to the batch's count and appended as ranges, or one range of everything.
    pub fn end_batch(&mut self) {
        self.taking = false;
        let tail = self.issued;
        let count = self.shared.idle_count.load(Relaxed);
        if count as usize > IDLE_RANGES {
            self.append(0, self.shared.rdram_len as i64, self.batch_start, tail, true);
        } else {
            for i in 0..count as usize {
                let slot = &self.shared.idle[i];
                let (from, to, start) = (slot.from.load(Relaxed), slot.to.load(Relaxed), slot.start.load(Relaxed));
                self.mark(from, to - from, tail, true, true);
                self.append(from, to, start, tail, true);
            }
        }
        self.idle_edit(|s| s.idle_count.store(0, Relaxed));
        self.kick_surely();
    }

    /// The drain woken if it may sleep: the fence pairs with its own before it parks, so one of the two sees the other.
    fn kick_surely(&self) {
        fence(SeqCst);
        for (w, t) in self.shared.workers.iter().zip(&self.threads) {
            if w.sleeping.load(Relaxed) {
                t.unpark();
            }
        }
    }

    fn wake_all(&self) {
        for t in &self.threads {
            t.unpark();
        }
    }

    /// `AliasedReads`: reads past a row's end made for another processor's row, by whichever processor made them; while the workers idle.
    pub fn aliased_reads(&self) -> i64 {
        self.wait_all();
        // SAFETY: every worker has run everything, so no processor is being written.
        self.shared.workers.iter().map(|w| unsafe { (&*w.processor).split.aliased_reads }).sum()
    }

    /// A test's look at one worker's processor while the workers idle.
    #[cfg(test)]
    pub fn processor(&self, i: usize) -> &Rdp {
        self.wait_all();
        // SAFETY: every worker has run everything, so no processor is being written.
        unsafe { &*self.shared.workers[i].processor }
    }

    /// How many processors share the list: the leader, which is the machine's, and the ones owned here.
    pub fn workers(&self) -> usize {
        1 + self.extra.len()
    }

    fn make_room(&self, tail: i64) {
        self.kick_surely();
        let mut spins = 0;
        while tail - self.shared.completed() >= RING as i64 {
            self.check_fault();
            backoff(&mut spins);
        }
    }

    /// `Shadow`: the interface gathers as the processor does, and remembers the images and the scissor a draw will reach.
    fn shadow(&mut self, word: u64, after: i64) -> bool {
        if self.shadow_taken == 0 {
            self.shadow_first = word;
        }
        self.shadow_taken += 1;
        let id = command_id(self.shadow_first);
        if self.shadow_taken < command_length(id) {
            return false;
        }
        self.shadow_taken = 0;
        let first = self.shadow_first;
        let straddled = self.gathered_before_the_ring;
        self.gathered_before_the_ring = false;

        match id {
            0x08..=0x0F | rdp::TEXTURE_RECTANGLE | rdp::TEXTURE_RECTANGLE_FLIPPED | rdp::FILL_RECTANGLE => self.mark_draw(id, first, after, straddled),
            rdp::SET_OTHER_MODES => {
                self.shadow_cycle = ((first >> 52) & 3) as i32;
                self.shadow_depth = ((first >> 4) & 3) != 0;
                self.shadow_depth_update = ((first >> 5) & 1) != 0;
            }
            rdp::SYNC_FULL => return true,
            rdp::SET_COLOR_IMAGE => {
                self.drawn = false;
                self.color_extent.index = -1;
                self.color_bytes = match (first >> 51) & 3 {
                    1 => 1,
                    2 => 2,
                    3 => 4,
                    _ => 1,
                };
                self.color_width = ((first >> 32) & 0x3FF) as i32 + 1;
                self.color_image = first as u32 & 0x00FF_FFFF;
            }
            rdp::SET_MASK_IMAGE => {
                self.drawn = false;
                self.depth_extent.index = -1;
                self.depth_image = first as u32 & 0x00FF_FFFF;
            }
            rdp::SET_SCISSOR => {
                self.drawn = false;
                self.color_extent.index = -1;
                self.depth_extent.index = -1;
                self.scissor_top_raw = ((first >> 32) & 0xFFF) as i32;
                self.scissor_bottom_raw = (first & 0xFFF) as i32;
                self.scissor_top = ((first >> 32) & 0xFFF) as i32 >> 2;
                self.scissor_bottom = ((first & 0xFFF) as i32 + 3) >> 2;
                self.scissor_right = (((first >> 12) & 0xFFF) as i32 + 3) >> 2;
            }
            rdp::SET_TEXTURE_IMAGE => {
                self.texture_size = ((first >> 51) & 3) as i32;
                self.texture_width = ((first >> 32) & 0x3FF) as i32 + 1;
                self.texture_image = first as u32 & 0x00FF_FFFF;
            }
            rdp::LOAD_TILE | rdp::LOAD_BLOCK | rdp::LOAD_PALETTE => self.mark_load(first, id == rdp::LOAD_BLOCK, after),
            _ => {}
        }
        false
    }

    /// `MarkDraw`: the rows the walker can shade, by its own limits, and the depth image only in a mode that reads or writes it.
    fn mark_draw(&mut self, id: u32, first: u64, start: i64, straddled: bool) {
        self.drawn = true;
        let triangle = (0x08..=0x0F).contains(&id);
        let (upper, lower) = if triangle {
            let (yh, yl) = (sign_extend14(first), sign_extend14(first >> 32));
            let upper = if yh & 0x2000 != 0 {
                self.scissor_top_raw
            } else if yh & 0x1000 != 0 {
                yh
            } else {
                yh.max(self.scissor_top_raw)
            };
            let lower = if yl & 0x2000 != 0 {
                yl
            } else if yl & 0x1000 != 0 {
                self.scissor_bottom_raw
            } else {
                yl.min(self.scissor_bottom_raw)
            };
            (upper, lower)
        } else {
            let top = (first & 0xFFF) as i32;
            let bottom = ((first >> 32) & 0xFFF) as i32 | if self.shadow_cycle >= 2 { 3 } else { 0 };
            (top.max(self.scissor_top_raw), bottom.min(self.scissor_bottom_raw))
        };

        if lower <= upper {
            self.append_box(EMPTY, first, 0, -1, start);
            return;
        }
        let kind = if straddled {
            WHOLE_ROWS
        } else if triangle {
            TRIANGLE
        } else {
            RECTANGLE
        };
        self.append_box(kind, first, upper, lower, start);

        let (width, first_row, last_row) = (self.color_width as i64, (upper >> 2) as i64, ((lower - 1) >> 2) as i64);
        let from = (first_row * width - 2).max(0);
        let to = ((last_row + 1) * width).max(last_row * width + self.scissor_right as i64 + 3);
        let (color, bytes, depth) = (self.color_image, self.color_bytes, self.depth_image);
        self.grow(false, color, bytes, from, to, start);
        if self.shadow_cycle < 2 && self.shadow_depth {
            self.grow(true, depth, 2, from, to, start);
        }
    }

    /// `AppendBox`: written whole before its count moves.
    fn append_box(&mut self, kind: i32, first: u64, upper: i32, lower: i32, end: i64) {
        let depth = self.shadow_cycle < 2 && self.shadow_depth_update;
        let appended = self.shared.boxes_appended.load(Relaxed);
        let slot = &self.shared.boxes[(appended as usize) & (BOX_COUNT - 1)];
        fence(Release);
        slot.end.store(end, Relaxed);
        slot.kind.store(kind, Relaxed);
        slot.color.store(self.color_image, Relaxed);
        slot.depth.store(if depth { self.depth_image } else { u32::MAX }, Relaxed);
        slot.bytes.store(self.color_bytes, Relaxed);
        slot.width.store(self.color_width, Relaxed);
        slot.scissor_right.store(self.scissor_right, Relaxed);
        slot.first.store(first, Relaxed);
        if kind == TRIANGLE {
            let length = command_length(command_id(first)) as i64;
            let word = |k: i64| self.shared.ring[((end - length + k) as usize) & (RING - 1)].load(Relaxed);
            slot.low.store(word(1), Relaxed);
            slot.high.store(word(2), Relaxed);
            slot.middle.store(word(3), Relaxed);
            slot.row_first.store(upper, Relaxed);
            slot.row_last.store(lower, Relaxed);
        } else {
            slot.row_first.store(upper >> 2, Relaxed);
            slot.row_last.store((lower - 1) >> 2, Relaxed);
        }
        self.shared.boxes_appended.store(appended + 1, Release);
    }

    /// `LastHolding`: the last pending draw whose box holds the eight bytes about an address; zero when none, `IDLE` when the ring cannot say.
    fn last_holding(&self, physical: i64, completed: i64) -> i64 {
        let from = physical & !7;
        let to = from + 8;
        let appended = self.shared.boxes_appended.load(Relaxed);
        let mut i = appended - 1;
        while i >= 0 {
            if appended - i > BOX_COUNT as i64 {
                return IDLE;
            }
            let b = self.shared.boxes[(i as usize) & (BOX_COUNT - 1)].load();
            if b.end <= completed {
                return 0;
            }
            if holds(&b, from, to) {
                return b.end;
            }
            i -= 1;
        }
        0
    }

    /// The open batch's ranges changed under the sequence the verifier reads them by.
    fn idle_edit(&self, edit: impl FnOnce(&Shared)) {
        let s = &*self.shared;
        let sequence = s.idle_sequence.load(Relaxed);
        s.idle_sequence.store(sequence.wrapping_add(1), Relaxed);
        fence(Release);
        edit(s);
        s.idle_sequence.store(sequence.wrapping_add(2), Release);
    }

    /// `Grow`: an extent's idle range grown in place; one that could not be listed stays whole-memory.
    fn grow(&mut self, depth: bool, image: u32, bytes: i32, mut from: i64, mut to: i64, start: i64) {
        let extent = if depth { self.depth_extent } else { self.color_extent };
        if extent.index == -2 {
            return;
        }
        if extent.index >= 0 && from >= extent.first && to <= extent.last {
            return;
        }
        if extent.index >= 0 {
            from = from.min(extent.first);
            to = to.max(extent.last);
        }
        let address = image as i64 + from * bytes as i64;
        let count = (to - from) * bytes as i64;
        let mut grown = Extent { index: extent.index, first: from, last: to };
        if extent.index < 0 {
            self.mark_idle(address, count, start);
            let n = self.shared.idle_count.load(Relaxed);
            grown.index = if n as usize <= IDLE_RANGES { n - 1 } else { -2 };
        } else {
            self.mark(address, count, IDLE, true, false);
            let index = extent.index as usize;
            self.idle_edit(|s| {
                s.idle[index].from.store(address, Relaxed);
                s.idle[index].to.store(address + count, Relaxed);
            });
        }
        if depth {
            self.depth_extent = grown;
        } else {
            self.color_extent = grown;
        }
    }

    /// `MarkLoad`: the rows a load reads from the texture image, each widened by the sixteen-byte window it reads through.
    fn mark_load(&mut self, word: u64, block: bool, after: i64) {
        let sl = ((word >> 44) & 0xFFF) as i32;
        let tl = ((word >> 32) & 0xFFF) as i32;
        let sh = ((word >> 12) & 0xFFF) as i32;
        let th = (word & 0xFFF) as i32;
        let bits = 4i64 << self.texture_size;
        let row_bytes = (self.texture_width as i64 * bits + 7) / 8;
        let image = self.texture_image as i64;
        let (from, to) = if block {
            let left = ((sl << 20) >> 20) as i64;
            (image + (tl & 0x3FF) as i64 * row_bytes + left * bits / 8 - 16, image + (tl & 0x3FF) as i64 * row_bytes + (sh as i64 + 1) * bits / 8 + 32)
        } else {
            (image + (tl >> 2) as i64 * row_bytes + (sl >> 2) as i64 * bits / 8 - 16, image + (th >> 2) as i64 * row_bytes + ((sh as i64 >> 2) + 1) * bits / 8 + 32)
        };
        let to = to.max(from + 1);
        self.mark(from, to - from, after, false, false);
        self.append(from, to, after, after, false);
    }

    /// `MarkIdle`: marked idle until the batch ends, and listed; a batch changing images more often than the list holds keeps them idle.
    fn mark_idle(&mut self, from: i64, count: i64, start: i64) {
        self.mark(from, count, IDLE, true, false);
        self.idle_edit(|s| {
            let n = s.idle_count.load(Relaxed);
            if (n as usize) < IDLE_RANGES {
                let slot = &s.idle[n as usize];
                slot.from.store(from, Relaxed);
                slot.to.store(from + count, Relaxed);
                slot.start.store(start, Relaxed);
                s.idle_count.store(n + 1, Relaxed);
            } else {
                s.idle_count.store(IDLE_RANGES as i32 + 1, Relaxed);
            }
        });
    }

    /// `Mark`: a later mark is never smaller, except at a batch's end, which writes its count over the idle marks it made.
    fn mark(&self, from: i64, count: i64, after: i64, write: bool, downgrade: bool) {
        let length = self.shared.rdram_len as i64;
        let first = from.clamp(0, length) >> 12;
        let last = ((from + count).clamp(0, length) - 1) >> 12;
        let marks = &self.shared.marks;
        let mut page = first;
        while page <= last {
            let p = page as usize;
            let m = &marks.marks[p];
            m.store(if downgrade { after } else { m.load(Relaxed).max(after) }, Relaxed);
            if write {
                let m = &marks.write_marks[p];
                m.store(if downgrade { after } else { m.load(Relaxed).max(after) }, Relaxed);
            }
            page += 1;
        }
    }

    /// `Append`: the range written whole before its count moves.
    fn append(&self, from: i64, to: i64, start: i64, mark: i64, write: bool) {
        let length = self.shared.rdram_len as i64;
        let appended = self.shared.ranges_appended.load(Relaxed);
        let slot = &self.shared.ranges[(appended as usize) & (RANGE_COUNT - 1)];
        fence(Release);
        slot.from.store(from.clamp(0, length), Relaxed);
        slot.to.store(to.clamp(0, length), Relaxed);
        slot.start.store(start, Relaxed);
        slot.mark.store(mark, Relaxed);
        slot.write.store(write, Relaxed);
        self.shared.ranges_appended.store(appended + 1, Release);
    }

    /// `Reaches`: whether any range still pending overlaps the bytes, for a writer any range and for a reader one the processor writes.
    fn reaches(&self, from: i64, to: i64, completed: i64, write: bool) -> bool {
        let s = &*self.shared;
        let count = s.idle_count.load(Relaxed);
        if count as usize > IDLE_RANGES {
            return true;
        }
        for slot in &s.idle[..count as usize] {
            if from < slot.to.load(Relaxed) && to > slot.from.load(Relaxed) {
                return true;
            }
        }
        let appended = s.ranges_appended.load(Relaxed);
        let (newest, oldest) = (appended - 1, (appended - RANGE_COUNT as i64).max(0));
        let mut i = newest;
        while i >= oldest {
            let slot = &s.ranges[(i as usize) & (RANGE_COUNT - 1)];
            if slot.mark.load(Relaxed) <= completed {
                return false;
            }
            if from < slot.to.load(Relaxed) && to > slot.from.load(Relaxed) && (write || slot.write.load(Relaxed)) {
                return true;
            }
            i -= 1;
        }
        newest - oldest + 1 >= RANGE_COUNT as i64
    }

    /// `WaitFor` and `WaitForRead` over the access's own bytes, where C# tests the first alone (Mars_Rdp.md §2.6.3's suspicion).
    pub fn wait_for(&mut self, physical: u32, bytes: u32, site: usize, write: bool) {
        let page = (physical >> 12) as usize & (PAGES - 1);
        let marks = if write { &self.shared.marks.marks } else { &self.shared.marks.write_marks };
        let mark = marks[page].load(Relaxed);
        if mark != 0 {
            self.wait(physical as i64, physical as i64 + bytes as i64, page, mark, site, write);
        }
    }

    /// `WaitForRange` and `WaitForReadRange`: every marked page of the range, each for its own bytes.
    pub fn wait_range(&mut self, from: i64, count: i64, site: usize, write: bool) {
        let length = self.shared.rdram_len as i64;
        let first = from.clamp(0, length) >> 12;
        let last = ((from + count).clamp(0, length) - 1) >> 12;
        let started = Instant::now();
        self.waiting += 1;
        let mut waited = false;
        let mut page = first;
        while page <= last {
            let p = page as usize;
            let mark = if write { &self.shared.marks.marks[p] } else { &self.shared.marks.write_marks[p] }.load(Relaxed);
            if mark != 0 {
                let (page_from, page_to) = (from.max(page << 12), (from + count).min((page + 1) << 12));
                waited |= self.wait(page_from, page_to, p, mark, site, write);
            }
            page += 1;
        }
        self.waiting -= 1;
        if waited {
            self.counters.range_waits += 1;
            self.counters.range_wait_nanos += started.elapsed().as_nanos() as i64;
        }
    }

    /// `Wait`: a writer waits for every range on its page, a reader for the ones the processor writes; a bystander waits for neither.
    fn wait(&mut self, from: i64, to: i64, page: usize, mark: i64, mut site: usize, write: bool) -> bool {
        let completed = self.shared.completed();
        if mark != IDLE && completed >= mark {
            self.clear(page, mark);
            return false;
        }
        if !self.reaches(from, to, completed, write) {
            self.counters.bystanders += 1;
            return false;
        }

        // A small read waits only for the last pending draw whose box holds it; C# narrows a range by its first eight bytes too, which this does not.
        let mut goal = if mark == IDLE { self.issued } else { mark };
        if !write && to - from <= 8 {
            let holding = self.last_holding(from, completed);
            if holding != IDLE && holding < goal {
                self.counters.reads_narrowed += 1;
                if holding <= completed {
                    self.counters.reads_freed += 1;
                    return false;
                }
                goal = holding;
            }
        }

        if self.taking && site == site::BUS_READ {
            site = site::TAKE;
        }
        let started = Instant::now();
        self.wait_until(goal);
        if mark != IDLE && goal >= mark {
            self.clear(page, mark);
        }
        let took = started.elapsed().as_nanos() as i64;
        self.counters.waits_per_site[site] += 1;
        self.counters.nanos_per_site[site] += took;
        if self.waiting == 0 {
            self.counters.page_waits += 1;
            self.counters.page_wait_nanos += took;
        }
        true
    }

    /// `Clear`: a mark no greater than the one waited for has been run; an idle one, or a later load's, stands.
    fn clear(&self, page: usize, mark: i64) {
        let marks = &self.shared.marks;
        if marks.marks[page].load(Relaxed) <= mark {
            marks.marks[page].store(0, Relaxed);
        }
        if marks.write_marks[page].load(Relaxed) <= mark {
            marks.write_marks[page].store(0, Relaxed);
        }
    }

    /// `WaitUntil`: the drain kicked if it is short, then waited for with an Acquire, so what it drew is this thread's to read.
    pub fn wait_until(&self, words: i64) {
        if self.shared.completed() < words {
            self.kick_surely();
            let mut spins = 0;
            while self.shared.completed() < words {
                self.check_fault();
                backoff(&mut spins);
            }
        }
        self.check_fault();
    }

    /// `Join`: everything handed over has run, the leader holds each scratch field as raster order left it, and the marks and ranges are forgotten.
    pub fn join(&mut self) {
        let started = Instant::now();
        let waited = self.shared.completed() < self.issued;
        self.wait_until(self.issued);
        assemble(&self.shared);
        self.shared.marks.clear();
        self.shared.ranges_appended.store(0, Release);
        if waited {
            self.counters.joins += 1;
            self.counters.join_nanos += started.elapsed().as_nanos() as i64;
        }
    }

    /// Everything handed over has run and the leader assembled, for a caller holding the machine shared; the marks are cleared by the next wait.
    pub fn wait_all(&self) {
        self.wait_until(self.issued);
        assemble(&self.shared);
    }

    /// `Hold`: the backlog brought within a snapshot's tail, then every worker stood at one word.
    pub fn hold(&self) -> u64 {
        if self.pending() > SNAPSHOT_WORDS {
            self.wait_until(self.issued - SNAPSHOT_WORDS);
        }
        self.pause()
    }

    /// `Pause`: answered when every worker stands, for this request, at a pause point that holds still; the Acquires make what they drew this thread's.
    fn pause(&self) -> u64 {
        if self.shared.holds.fetch_add(1, Relaxed) > 0 {
            return self.shared.pauses.load(Relaxed);
        }
        let number = self.shared.pauses.fetch_add(1, Relaxed) + 1;
        self.shared.pause_request.store(number, SeqCst);
        self.wake_all();
        let mut spins = 0;
        loop {
            self.check_fault();
            let at = self.shared.pause_at.load(Acquire);
            if at >= 0 && self.shared.workers.iter().all(|w| w.standing.load(Acquire) == number && w.standing_at.load(Relaxed) == at) && self.shared.pause_at.load(Acquire) == at {
                break;
            }
            backoff(&mut spins);
        }
        assemble(&self.shared);
        number
    }

    pub fn resume(&self) {
        if self.shared.holds.fetch_sub(1, Relaxed) == 1 {
            release(&self.shared, &self.threads);
        }
    }

    /// A test's way to end a hold from another thread.
    #[cfg(test)]
    pub fn resumer(&self) -> impl FnOnce() + Send + 'static {
        let (shared, threads) = (self.shared.clone(), self.threads.clone());
        move || {
            if shared.holds.fetch_sub(1, Relaxed) == 1 {
                release(&shared, &threads);
            }
        }
    }

    /// `WritePending`'s words: those handed over and not run, read while the workers stand.
    pub fn pending_words(&self) -> Vec<u64> {
        let (completed, issued) = (self.shared.completed(), self.issued);
        (completed..issued).map(|i| self.shared.ring[(i as usize) & (RING - 1)].load(Relaxed)).collect()
    }

    /// `Rethrow`: a fault on a worker, or one the verifier found, is this thread's to raise.
    pub fn check_fault(&self) {
        if self.shared.faulted.load(Acquire) {
            let fault = self.shared.fault.lock().unwrap_or_else(|e| e.into_inner()).clone();
            panic!("the display processor's thread failed: {}", fault.unwrap_or_default());
        }
    }

    /// The workers joined, assembled and ended, and the marks left clear; after this the leader and the memories are the machine's alone, and it draws alone.
    pub fn stop(&mut self) {
        if !self.shared.faulted.load(Acquire) {
            self.wait_until(self.issued);
            assemble(&self.shared);
        }
        self.end();
    }

    fn end(&mut self) {
        self.shared.stopping.store(true, SeqCst);
        self.wake_all();
        for drain in self.drains.drain(..) {
            let _ = drain.join();
        }
        self.shared.marks.clear();
        // SAFETY: every worker has ended, so the leader is the machine's again.
        unsafe { (*self.shared.workers[0].processor).configure(0, 1) };
    }
}

impl Drop for Threads {
    fn drop(&mut self) {
        if !self.drains.is_empty() {
            self.end();
        }
    }
}

/// `Resume`: the request withdrawn and the pause point forgotten, then every worker woken.
fn release(shared: &Shared, threads: &[Thread]) {
    shared.pause_request.store(0, SeqCst);
    shared.pause_at.store(-1, SeqCst);
    for t in threads {
        t.unpark();
    }
}

/// `Assemble`: the leader's scratch replaced by whichever processor wrote each field last in raster order.
fn assemble(shared: &Shared) {
    if shared.workers.len() < 2 {
        return;
    }
    // SAFETY: every caller has every worker standing, waiting at a barrier, or past every word, so no processor is being written.
    unsafe {
        let leader = shared.workers[0].processor;
        for w in &shared.workers[1..] {
            (*leader).take_scratch_from(&*w.processor);
        }
    }
}

/// `SpinWait.SpinOnce(-1)`: spins at first, then yields, and never sleeps.
#[inline]
fn backoff(spins: &mut u32) {
    if *spins < 64 {
        for _ in 0..(1u32 << (*spins / 8).min(6)) {
            std::hint::spin_loop();
        }
    } else {
        thread::yield_now();
    }
    *spins = spins.saturating_add(1);
}

/// A worker's thread; a panic is recorded, and its count set past every word so no one waits for it, as C#'s catch does.
fn work(shared: Arc<Shared>, index: usize) {
    let run = catch_unwind(AssertUnwindSafe(|| run_worker(&shared, index)));
    if let Err(panic) = run {
        let message = panic.downcast_ref::<String>().cloned().or_else(|| panic.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();
        shared.record_fault(format!("a panic on worker {index}: {message}"));
        shared.workers[index].completed.store(i64::MAX >> 1, Release);
    }
}

/// `RunWorker`: every word in order, each command as its step asks, standing when asked and sleeping when there is nothing.
fn run_worker(shared: &Shared, index: usize) {
    let me = &shared.workers[index];
    let processor = me.processor;
    let checked = Checked::default();
    let mut completed = me.completed.load(Relaxed);
    let (mut from, mut started) = (completed, Instant::now());
    loop {
        if shared.stopping.load(Acquire) {
            return;
        }
        if shared.pause_request.load(Acquire) != 0 && stand(shared, index, completed) {
            continue;
        }
        if completed == shared.issued.load(Acquire) {
            me.words.fetch_add(completed - from, Relaxed);
            if index == 0 {
                shared.drain_words.fetch_add(completed - from, Relaxed);
                shared.drain_nanos.fetch_add(started.elapsed().as_nanos() as i64, Relaxed);
            }
            sleep(shared, index, completed);
            if index == 0 {
                shared.drain_starts.fetch_add(1, Relaxed);
            }
            (from, started) = (completed, Instant::now());
            continue;
        }

        let word = shared.ring[(completed as usize) & (RING - 1)].load(Relaxed);
        let check = if shared.verifying.load(Relaxed) { Some((shared, completed + 1, &checked)) } else { None };
        // SAFETY: published words are the workers' to run until the machine's Acquire of every count passes them (the module's rule); each worker has its own processor, the leader's reached by the others only at barriers.
        unsafe {
            let mut memory = RdpMemory::shared((shared.rdram, shared.rdram_len), (shared.hidden, shared.hidden_len), check);
            match (*processor).gather(word) {
                Step::More => {}
                Step::Ready => {
                    (*processor).execute_gathered(&mut memory);
                }
                Step::Leader => {
                    shared.barrier.arrive(shared, completed + 1);
                    if index == 0 {
                        assemble(shared);
                        (*processor).execute_gathered(&mut memory);
                    }
                    shared.barrier.arrive(shared, completed + 1);
                }
                Step::All => {
                    shared.barrier.arrive(shared, completed + 1);
                    (*processor).execute_gathered(&mut memory);
                }
                Step::AllJoined => {
                    shared.barrier.arrive(shared, completed + 1);
                    (*processor).execute_gathered(&mut memory);
                    shared.barrier.arrive(shared, completed + 1);
                }
            }
        }
        completed += 1;
        me.completed.store(completed, Release);
    }
}

/// `Sleep`: a moment's spinning for more, then parked until kicked; the flag and the fence keep a kick from passing unseen.
fn sleep(shared: &Shared, index: usize, completed: i64) {
    for _ in 0..400 {
        if shared.issued.load(Acquire) != completed || shared.pause_request.load(Acquire) != 0 || shared.stopping.load(Acquire) {
            return;
        }
        for _ in 0..50 {
            std::hint::spin_loop();
        }
    }
    let me = &shared.workers[index];
    me.sleeping.store(true, SeqCst);
    if shared.issued.load(SeqCst) == completed && shared.pause_request.load(SeqCst) == 0 && !shared.stopping.load(SeqCst) {
        thread::park();
    }
    me.sleeping.store(false, SeqCst);
}

/// The pause point raised to at least `to`; returns where it stands.
fn raise(pause_at: &AtomicI64, to: i64) -> i64 {
    let mut at = pause_at.load(Acquire);
    while at < to {
        match pause_at.compare_exchange_weak(at, to, AcqRel, Acquire) {
            Ok(_) => at = to,
            Err(now) => at = now,
        }
    }
    at
}

/// `Stand`: the pause point raised to this worker's place; short of it, run on (false); at it, stand and answer until the request or the point moves (true).
fn stand(shared: &Shared, index: usize, completed: i64) -> bool {
    let me = &shared.workers[index];
    let mut spins = 0;
    loop {
        let request = shared.pause_request.load(Acquire);
        if request == 0 || shared.stopping.load(Acquire) {
            return true;
        }
        let at = raise(&shared.pause_at, completed);
        if at > completed {
            return false;
        }
        me.standing_at.store(completed, Relaxed);
        me.standing.store(request, Release);
        backoff(&mut spins);
    }
}
