//! The GS drawn on a Vulkan device (Sapphire_Gs.md): the 4 MiB of local memory kept on the device in the GS's own
//! layout and drawn into by a compute walk (`plan`, `shaders/walk.comp`), with every PSM's addressing, the pixel
//! pipeline and texturing in shaders and in the software rasteriser's integers, so that the memory is the same. The
//! processor keeps a mirror of the memory for what it reads itself (a CLUT's load, a local-to-host transfer, the PCRTC's
//! lines, a state): each page is clean, newer on the processor (written by a transfer, and copied in before the next
//! drawing that follows it) or newer on the device (drawn, and read back before the processor reads it).

pub mod plan;

use std::cell::RefCell;
use std::time::Instant;

use emusen_gpu::{GpuBuffer, GpuDevice, GpuProgram, Lent, Where};

use super::draw::{Environment, Kind, Vertex};
use super::local::{self, Local, Units};
use super::raster;
use super::texture::Clut;
use plan::{Close, Drawn, Op, Planner};

/// The walk's SPIR-V, compiled from `shaders/walk.comp` and checked against it by `SapphireShaderTests`.
const WALK: &[u8] = include_bytes!("../../../shaders/walk.spv");

const PAGE_BYTES: u64 = (plan::PAGE_WORDS * 4) as u64;

/// Measurement switches for the walk, from SAPPHIRE_GS_WALK_DEBUG: 1 skips texturing, 2 everything after coverage.
fn debug_bits() -> u32 {
    std::env::var("SAPPHIRE_GS_WALK_DEBUG").ok().and_then(|v| v.parse().ok()).unwrap_or(0)
}

/// What made the processor wait for the device.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cause {
    /// A CLUT's load, a local-to-host or local-to-local transfer, a host-to-local transfer into drawn pages.
    ClutLoad,
    Download,
    Copy,
    Upload,
    /// A primitive the processor draws itself.
    Fallback,
    /// The PCRTC's lines.
    Display,
    /// A state, a debugger or a test.
    Join,
}

/// What the device path did, cumulatively.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub plan: plan::Counts,
    /// Waits by `Cause`: a count and nanoseconds.
    pub waits: [(u64, u64); 7],
    pub read_back_pages: u64,
    /// Nanoseconds the processor spent submitting.
    pub submit_ns: u64,
}

/// One submission's host buffers: records, environments, entries, tiles, CLUTs and staged pages.
struct Slot {
    buffers: [GpuBuffer; 6],
}

struct Gpu {
    device: GpuDevice,
    /// The walks made so far, by class with exact mode in bit 8.
    walks: Vec<(u32, GpuProgram)>,
    float_controls: bool,
    local: GpuBuffer,
    /// The walk's measurement counts (switch 32): lanes that met a record, pixels covered, pixels textured.
    lanes: GpuBuffer,
    /// The memory as it stood before a snapshot stretch (Sapphire_Gs.md §4.4).
    snapshot: GpuBuffer,
    readback: GpuBuffer,
    slots: [Slot; 2],
    next: usize,
}

impl Gpu {
    fn new(lend: Option<(Lent, u64)>) -> Result<Gpu, String> {
        // SAFETY: a lend's handles are the lender's, live while the core is (EmuSen_Gpu.md §5).
        let device = match lend {
            Some((l, _)) => unsafe { GpuDevice::lent(&l)? },
            None => GpuDevice::try_create(None)?,
        };
        if !device.int64() {
            return Err(format!("{} offers shaders no 64-bit integers", device.name()));
        }
        let float_controls = device.float_controls();
        let local = device.create_buffer((local::WORDS * 4) as u64, Where::Device)?;
        let readback = device.create_buffer((local::WORDS * 4) as u64, Where::Host)?;
        let lanes = device.create_buffer(64, Where::Host)?;
        let snapshot = device.create_buffer((local::WORDS * 4) as u64, Where::Device)?;
        lanes.slice_mut::<u32>().fill(0);
        let sizes = [
            plan::MAX_RECORDS * plan::RECORD_WORDS * 4,
            plan::MAX_ENVS * plan::ENV_WORDS * 4,
            plan::MAX_ENTRIES * 4,
            plan::MAX_TILES * 16,
            plan::MAX_CLUTS * plan::CLUT_WORDS * 4,
            plan::MAX_STAGED_PAGES * plan::PAGE_WORDS * 4,
        ];
        let slot = || -> Result<Slot, String> {
            let b: Vec<GpuBuffer> = sizes.iter().map(|&s| device.create_buffer(s as u64, Where::Host)).collect::<Result<_, _>>()?;
            Ok(Slot { buffers: b.try_into().map_err(|_| "six buffers".to_string())? })
        };
        let slots = [slot()?, slot()?];
        device.submit(|c| c.fill(&local, 0));
        let mut g = Gpu { device, walks: Vec::new(), float_controls, local, lanes, snapshot, readback, slots, next: 0 };
        g.walk_for(plan::CLASS_ALL, false);
        Ok(g)
    }

    /// The walk for a class, made now if it was not; single-precision coordinates need the float controls, and without
    /// them the exact walk draws.
    fn walk_for(&mut self, class: u32, exact: bool) {
        let key = class | u32::from(exact) << 8;
        if self.walks.iter().any(|w| w.0 == key) {
            return;
        }
        let mut constants: Vec<(u32, u32)> = vec![(0, u32::from(exact || !self.float_controls))];
        constants.extend((0..8).map(|b| (b + 1, class >> b & 1)));
        let w = self.device.create_program_specialised(WALK, 8, 16, 2, &constants).expect("the walk compiles for its class");
        self.walks.push((key, w));
    }

    /// A batch submitted, with the pages `back` copied to the readback buffer after it; waited for when `wait`.
    fn submit(&mut self, batch: &plan::Batch, back: &[usize], wait: bool, exact: bool) {
        // Each class's walk, made the first time a stretch of it is submitted (Sapphire_Gs.md §5.5).
        for op in &batch.ops {
            if let Op::Walk { class, .. } = *op {
                self.walk_for(class, exact);
            }
        }
        let s = self.next;
        self.next ^= 1;
        let slot = &self.slots[s];
        let put = |b: &GpuBuffer, words: &[u32]| b.slice_mut::<u32>()[..words.len()].copy_from_slice(words);
        put(&slot.buffers[0], &batch.records);
        put(&slot.buffers[1], &batch.envs);
        put(&slot.buffers[2], &batch.entries);
        slot.buffers[3].slice_mut::<[u32; 4]>()[..batch.tiles.len()].copy_from_slice(&batch.tiles);
        put(&slot.buffers[4], &batch.cluts);
        put(&slot.buffers[5], &batch.staged);
        let walks = &self.walks;
        let (local, lanes, snapshot, readback) = (&self.local, &self.lanes, &self.snapshot, &self.readback);
        let record = |c: &mut emusen_gpu::Commands| {
            let mut bound: Vec<u32> = Vec::with_capacity(8);
            for op in &batch.ops {
                match *op {
                    Op::Walk { first, count, class, ordered } => {
                        let push = [first, debug_bits(), u32::from(ordered), 0];
                        let key = class | u32::from(exact) << 8;
                        let walk = &walks.iter().find(|w| w.0 == key).expect("a walk made for each class").1;
                        if bound.contains(&key) {
                            c.dispatch_again(walk, s, &push, count, 1, 1);
                        } else {
                            let b = &slot.buffers;
                            c.dispatch_in(walk, s, &[local, &b[0], &b[1], &b[2], &b[3], &b[4], lanes, snapshot], &push, count, 1, 1);
                            bound.push(key);
                        }
                    }
                    Op::Snapshot { page, pages } => {
                        let at = u64::from(page) * PAGE_BYTES;
                        c.copy(local, snapshot, u64::from(pages) * PAGE_BYTES, at, at);
                    }
                    Op::Upload { from, to, pages } => {
                        c.copy(&slot.buffers[5], local, u64::from(pages) * PAGE_BYTES, u64::from(from) * PAGE_BYTES, u64::from(to) * PAGE_BYTES);
                    }
                }
            }
            let mut i = 0;
            while i < back.len() {
                let mut j = i + 1;
                while j < back.len() && back[j] == back[j - 1] + 1 {
                    j += 1;
                }
                let at = back[i] as u64 * PAGE_BYTES;
                c.copy(local, readback, (j - i) as u64 * PAGE_BYTES, at, at);
                i = j;
            }
        };
        if wait {
            self.device.submit(record);
            self.device.wait_for_pending();
        } else {
            self.device.submit_pending(record);
        }
    }
}

impl Drop for Gpu {
    fn drop(&mut self) {
        self.device.wait_for_pending();
    }
}

struct Inner {
    plan: Planner,
    gpu: Option<Gpu>,
    /// Lanes the device may have changed since they were last read back.
    newer: Units,
    /// Pages the processor has written that the device has not yet been given, in order, and the same as flags.
    dirty: Vec<usize>,
    dirty_flags: Box<[bool]>,
    /// The pages a host-to-local transfer still writes, kept the processor's across every copy-in until it ends.
    claimed: Option<Units>,
    counts: Counts,
}

/// The device path, reached through `&self` from the GS's thread alone, as the workers' bookkeeping is.
pub struct Device {
    inner: RefCell<Inner>,
}

impl Device {
    /// On the lent device when there is one, else a device of its own; `plan_only` plans and draws nothing, to measure
    /// the processor's part alone (its memory is then wrong).
    pub fn new(lend: Option<(Lent, u64)>, plan_only: bool) -> Result<Device, String> {
        let gpu = if plan_only { None } else { Some(Gpu::new(lend)?) };
        let mut plan = Planner::new();
        plan.exact_feedback = std::env::var_os("SAPPHIRE_GS_EXACT").is_some();
        plan.ordered_mode = std::env::var_os("SAPPHIRE_GS_ORDERED").is_some();
        Ok(Device {
            inner: RefCell::new(Inner {
                plan,
                gpu,
                newer: Units::default(),
                dirty: (0..local::PAGES).collect(),
                dirty_flags: vec![true; local::PAGES].into_boxed_slice(),
                claimed: None,
                counts: Counts::default(),
            }),
        })
    }

    pub fn report(&self) -> String {
        self.inner.borrow().gpu.as_ref().map_or_else(|| "plan only".to_string(), |g| g.device.report())
    }

    /// The walk's measurement counts since the device was made, with switch 32: lanes that met a record, pixels covered,
    /// pixels textured; each pair of words a 64-bit count. Everything is drawn first.
    pub fn lane_counts(&self, local: &Local) -> [u64; 3] {
        self.join(local);
        let i = self.inner.borrow();
        let Some(g) = &i.gpu else { return [0; 3] };
        let w = g.lanes.slice::<u32>();
        std::array::from_fn(|k| u64::from(w[2 * k]) | u64::from(w[2 * k + 1]) << 32)
    }

    pub fn counts(&self) -> Counts {
        let i = self.inner.borrow();
        Counts { plan: i.plan.counts, ..i.counts }
    }

    /// A drawing kick that covers nothing, counted.
    pub fn empty(&self) {
        self.inner.borrow_mut().plan.counts.empty += 1;
    }

    /// Whether records that read where they draw are drawn by the processor in the reference's order (Sapphire_Gs.md §4.4).
    pub fn set_exact(&self, exact: bool) {
        self.inner.borrow_mut().plan.exact_feedback = exact;
    }

    /// The CLUT's buffer changed (a load, or a state read).
    pub fn clut_changed(&self) {
        self.inner.borrow_mut().plan.clut_changed();
    }

    /// A drawing kick: recorded for the device, or drawn here once what it touches is read back.
    #[allow(clippy::too_many_arguments)]
    pub fn draw(&self, regs: &[u64], kind: Kind, env: &Environment, v: &[Vertex; 3], clut: &Clut, local: &mut Local) {
        let mut i = self.inner.borrow_mut();
        if i.plan.full() {
            i.submit(Close::Full, local, false);
        }
        i.stage(local);
        match i.plan.draw(regs, kind, env, v, &clut.entries) {
            Drawn::Empty => {}
            Drawn::Recorded(writes) => i.newer.or(&writes),
            Drawn::Fallback(touched, writes) => {
                let mut t = touched;
                t.or(&writes.pages());
                if t.meets(&i.newer) {
                    i.sync(local, Cause::Fallback);
                }
                raster::primitive(kind, env, local, clut, v, raster::Rows::ALL);
                for u in writes.each() {
                    i.mark(u / 2);
                }
            }
        }
    }

    /// Before the processor writes a word: a page the device has drawn is read back first, and the page is the processor's.
    #[inline]
    pub fn before_write(&self, word: u32, local: &Local, cause: Cause) {
        let p = (word >> 11) as usize;
        let mut i = self.inner.borrow_mut();
        if i.dirty_flags[p] {
            return;
        }
        if i.newer.has(2 * p) || i.newer.has(2 * p + 1) {
            i.sync(local, cause);
        }
        i.mark(p);
    }

    /// Pages the processor is about to write over many accesses (a host-to-local transfer): those the device drew are read
    /// back first, and all are the processor's until the next drawing copies them in.
    pub fn claim(&self, units: &Units, local: &Local) {
        let mut i = self.inner.borrow_mut();
        i.claimed = Some(units.pages());
        i.reclaim(local);
    }

    /// After a drawing, the claimed pages are the processor's again: read back if the drawing changed them.
    pub fn reclaim(&self, local: &Local) {
        self.inner.borrow_mut().reclaim(local);
    }

    /// The transfer that claimed pages has ended.
    pub fn release(&self) {
        self.inner.borrow_mut().claimed = None;
    }

    /// Before the processor reads the lanes `lanes` of a word.
    #[inline]
    pub fn before_read(&self, word: u32, lanes: u32, local: &Local, cause: Cause) {
        let p = (word >> 11) as usize;
        let i = self.inner.borrow();
        if (lanes & 1 != 0 && i.newer.has(2 * p)) || (lanes & 2 != 0 && i.newer.has(2 * p + 1)) {
            drop(i);
            self.inner.borrow_mut().sync(local, cause);
        }
    }

    /// Before the processor reads any of `units`.
    pub fn settle(&self, units: &Units, local: &Local, cause: Cause) {
        let meets = units.meets(&self.inner.borrow().newer);
        if meets {
            self.inner.borrow_mut().sync(local, cause);
        }
    }

    /// Everything drawn and read back: the mirror is the memory.
    pub fn join(&self, local: &Local) {
        self.inner.borrow_mut().sync(local, Cause::Join);
    }

    /// What is recorded handed to the device without waiting, so that it draws while the machine runs on.
    pub fn flush(&self, local: &Local) {
        let mut i = self.inner.borrow_mut();
        i.stage(local);
        i.submit(Close::Full, local, false);
    }

    /// The mirror was replaced (a state read): the device's memory is taken again from it whole.
    pub fn loaded(&self) {
        let mut i = self.inner.borrow_mut();
        if let Some(g) = &i.gpu {
            g.device.wait_for_pending();
        }
        i.plan.take(Close::Sync).clear();
        i.plan.clut_changed();
        i.newer = Units::default();
        i.claimed = None;
        i.dirty = (0..local::PAGES).collect();
        i.dirty_flags.fill(true);
    }
}

impl Inner {
    fn reclaim(&mut self, local: &Local) {
        let Some(c) = self.claimed else { return };
        // Whole pages are copied in, so a page with either lane drawn is read back first.
        if c.meets(&self.newer) {
            self.sync(local, Cause::Upload);
        }
        let mut last = usize::MAX;
        for u in c.each() {
            if u / 2 != last {
                last = u / 2;
                self.mark(last);
            }
        }
    }

    fn mark(&mut self, p: usize) {
        if !self.dirty_flags[p] {
            self.dirty_flags[p] = true;
            self.dirty.push(p);
        }
    }

    /// The pages the processor wrote, given to the device at this point of the order.
    fn stage(&mut self, local: &Local) {
        if self.dirty.is_empty() {
            return;
        }
        if self.plan.full() {
            self.submit(Close::Full, local, false);
        }
        let mut pages = std::mem::take(&mut self.dirty);
        pages.sort_unstable();
        self.plan.upload(&pages, local.words());
        for &p in &pages {
            self.dirty_flags[p] = false;
        }
        pages.clear();
        self.dirty = pages;
        if let Some(c) = self.claimed {
            let mut last = usize::MAX;
            for u in c.each() {
                if u / 2 != last {
                    last = u / 2;
                    self.mark(last);
                }
            }
        }
    }

    fn submit(&mut self, why: Close, local: &Local, back: bool) {
        let t0 = Instant::now();
        let exact = self.plan.exact_feedback || std::env::var_os("SAPPHIRE_GS_EXACT_TEXTURE").is_some();
        let batch = self.plan.take(why);
        let pages: Vec<usize> = if back {
            let mut p: Vec<usize> = self.newer.each().map(|u| u / 2).collect();
            p.dedup();
            p
        } else {
            Vec::new()
        };
        if let Some(g) = self.gpu.as_mut() {
            if !batch.is_empty() || !pages.is_empty() {
                g.submit(batch, &pages, back, exact);
            }
            if back {
                let src = g.readback.slice::<u32>();
                for &p in &pages {
                    let r = p * plan::PAGE_WORDS..(p + 1) * plan::PAGE_WORDS;
                    local.store(r.start, &src[r]);
                }
            }
        }
        batch.clear();
        if back {
            self.counts.read_back_pages += pages.len() as u64;
            self.newer = Units::default();
        }
        self.counts.submit_ns += t0.elapsed().as_nanos() as u64;
    }

    /// Everything recorded drawn, and every page the device drew read back into the mirror.
    fn sync(&mut self, local: &Local, cause: Cause) {
        let t0 = Instant::now();
        self.submit(Close::Sync, local, true);
        let w = &mut self.counts.waits[cause as usize];
        w.0 += 1;
        w.1 += t0.elapsed().as_nanos() as u64;
    }
}
