//! The game picture's feed (EmuSen_Qt.md §1.8): the newest frame a frame thread handed over, advanced into a Serenity
//! chain and drawn in place on the render thread, where `GamePicture` asks for it; and each swap's time, the display
//! clock the frame thread paces by. The chain is made, used and dropped on the render thread alone, since its calls
//! submit to the queue the scene graph submits to; a preset asked for in its place is built there at the next draw.

use emusen_kit_qt::picture::{Feed, Picture};
use emusen_platform_client::serenity::{Chain, Device};
use std::ffi::c_void;
use std::path::{Path, PathBuf};
use std::ptr::NonNull;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Where a frame's picture lies: its pixels in `Frame::rgba`, the newest picture a machine took through its lend on the
/// device, or a slot of the machine's ring; a lend or a ring is the platform library's, by its address.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Source {
    #[default]
    Pixels,
    Lent(usize),
    Ring { ring: usize, slot: u32, value: u64 },
}

/// One frame: its picture's shape and where it lies; its pixel buffer is kept and refilled, never made per frame.
#[derive(Default)]
pub struct Frame {
    pub rgba: Vec<u8>,
    pub width: i32,
    pub height: i32,
    pub row_repeat: i32,
    pub serial: u64,
    pub source: Source,
}

/// What the render thread counted, read by anyone.
#[derive(Default)]
pub struct Counts {
    /// Swaps seen.
    pub swaps: AtomicU64,
    /// Frames handed over by a frame thread.
    pub handed: AtomicU64,
    /// Frames handed over and drawn.
    pub shown: AtomicU64,
    /// Of those, the frames drawn from a machine's lend and from its ring.
    pub shown_lent: AtomicU64,
    pub shown_ring: AtomicU64,
    /// The core's serial of the frame drawn last, which a read-back of the picture is compared with.
    pub shown_serial: AtomicU64,
    /// Frames handed over that a newer one replaced before any draw.
    pub dropped: AtomicU64,
    /// Swaps while running at which no new frame had come since the last.
    pub repeated: AtomicU64,
    /// The last swap's time, steady-clock nanoseconds.
    pub last_swap: AtomicI64,
}

/// What the frame thread and the render thread share.
struct Shared {
    /// The newest frame handed over and not yet drawn.
    newest: Mutex<(Frame, bool)>,
    /// The swaps and the frames taken by a draw, for a frame thread to wait on, with their one condition.
    swaps: Mutex<u64>,
    taken: Mutex<u64>,
    swapped: Condvar,
    counts: Counts,
    /// Whether a frame thread is running, for `repeated`.
    running: AtomicBool,
    /// Held by a draw from taking the newest frame to its end, so a withdrawal can wait it out.
    drawing: Mutex<()>,
    /// A preset and the uniforms set over it, asked for in place of the chain's, not yet built.
    through: Mutex<Option<Through>>,
    /// The folder built-in filters' presets are written under.
    presets: PathBuf,
    /// Parameter values asked for over the chain's own, set by the next draw without building it again.
    live: Mutex<Vec<(String, f32)>>,
    /// The parameters the chain drawn last declares, by id, for a lever to know whether it is set live.
    declared: Mutex<Vec<String>>,
}

/// A preset document and the uniforms the chain is given over the preset's values.
#[derive(Clone, Debug, Default, PartialEq)]
struct Through {
    preset: Vec<u8>,
    uniforms: Vec<(String, f32)>,
}

/// The render thread's own: the chain and the frame it last advanced from.
struct Render {
    device: Arc<Device>,
    through: Through,
    chain: Option<Chain>,
    /// The chain's parameters and the values it was last given, in its order.
    values: Vec<(String, f32)>,
    current: Frame,
    advanced: bool,
    /// Whether `current` came since it was last advanced into a chain, so a chain built again over it counts no new frame.
    fresh: bool,
    fresh_since_swap: bool,
    problem: Option<String>,
}

/// A feed `GamePicture` names by its address, and the handle a frame thread hands frames to.
pub struct PictureFeed {
    feed: Box<Feed>,
    context: Box<Context>,
}

struct Context {
    shared: Arc<Shared>,
    render: Mutex<Render>,
}

/// The frame thread's side: hand a frame over, wait for a swap.
#[derive(Clone)]
pub struct Handoff {
    shared: Arc<Shared>,
}

impl PictureFeed {
    /// A feed drawing through `preset` (a document Serenity's preset readers give) on `device`; the presets of the
    /// filters asked for later are written under `presets`.
    pub fn new(device: Arc<Device>, preset: Vec<u8>, presets: PathBuf) -> PictureFeed {
        let shared = Arc::new(Shared {
            newest: Mutex::default(),
            swaps: Mutex::new(0),
            taken: Mutex::new(0),
            swapped: Condvar::new(),
            counts: Counts::default(),
            running: AtomicBool::new(false),
            drawing: Mutex::new(()),
            through: Mutex::new(None),
            presets,
            live: Mutex::default(),
            declared: Mutex::default(),
        });
        let render = Render {
            device,
            through: Through { preset, uniforms: Vec::new() },
            chain: None,
            values: Vec::new(),
            current: Frame::default(),
            advanced: false,
            fresh: false,
            fresh_since_swap: false,
            problem: None,
        };
        let context = Box::new(Context { shared, render: Mutex::new(render) });
        let pointer = std::ptr::from_ref::<Context>(&context).cast_mut().cast::<c_void>();
        let feed = Box::new(Feed { context: pointer, draw, swapped });
        PictureFeed { feed, context }
    }

    /// The number `GamePicture.feed` takes.
    pub fn address(&self) -> u64 {
        self.feed.address()
    }

    pub fn handoff(&self) -> Handoff {
        Handoff { shared: self.context.shared.clone() }
    }

    /// The device the feed draws on, which the window adopted.
    pub fn device(&self) -> Arc<Device> {
        locked(&self.context.render).device.clone()
    }

    /// The hand-off and the device of the feed a `GamePicture.feed` number names: the one way a QML-made model reaches
    /// the picture the window adopted, since QML hands an element no Rust value (EmuSen_Qt.md §1.8.2).
    ///
    /// # Safety
    /// `address` is zero or a live `PictureFeed`'s `address()`.
    pub unsafe fn at(address: u64) -> Option<(Handoff, Arc<Device>)> {
        let feed = unsafe { (address as *const Feed).as_ref() }?;
        let context = unsafe { &*feed.context.cast::<Context>() };
        Some((Handoff { shared: context.shared.clone() }, locked(&context.render).device.clone()))
    }

    pub fn counts(&self) -> &Counts {
        &self.context.shared.counts
    }

    /// The last failure the render thread met, if any.
    pub fn problem(&self) -> Option<String> {
        locked(&self.context.render).problem.clone()
    }
}

impl Handoff {
    /// The frame thread's newest frame: its buffer swapped with the one waiting, so neither side copies or allocates.
    pub fn hand_over(&self, frame: &mut Frame) {
        let mut newest = locked(&self.shared.newest);
        self.shared.counts.handed.fetch_add(1, Ordering::Relaxed);
        if newest.1 {
            self.shared.counts.dropped.fetch_add(1, Ordering::Relaxed);
        }
        std::mem::swap(&mut newest.0, frame);
        newest.1 = true;
    }

    /// The frame waiting for a draw taken back, and any draw under way waited out: after it the render thread holds no
    /// picture of a machine's lend or ring that it has not drawn, so the machine may be freed (EmuSen_Qt.md §1.8.7).
    pub fn withdraw(&self) {
        {
            let mut newest = locked(&self.shared.newest);
            newest.1 = false;
            newest.0.source = Source::Pixels;
            newest.0.width = 0;
        }
        drop(locked(&self.shared.drawing));
    }

    /// Waits for the next swap after `seen`, at most `bound`; the swaps' count.
    pub fn wait_swap(&self, seen: u64, bound: Duration) -> u64 {
        let swaps = locked(&self.shared.swaps);
        let (swaps, _) = self.shared.swapped.wait_timeout_while(swaps, bound, |n| *n <= seen).unwrap_or_else(PoisonError::into_inner);
        *swaps
    }

    /// Waits until a draw has taken more than `seen` frames, at most `bound`; the frames taken.
    pub fn wait_taken(&self, seen: u64, bound: Duration) -> u64 {
        let taken = locked(&self.shared.taken);
        let (taken, _) = self.shared.swapped.wait_timeout_while(taken, bound, |n| *n <= seen).unwrap_or_else(PoisonError::into_inner);
        *taken
    }

    pub fn taken(&self) -> u64 {
        *locked(&self.shared.taken)
    }

    pub fn swaps(&self) -> u64 {
        *locked(&self.shared.swaps)
    }

    /// Swaps, frames drawn, frames dropped and swaps repeated, so far.
    pub fn counted(&self) -> [u64; 4] {
        let c = &self.shared.counts;
        [
            c.swaps.load(Ordering::Relaxed),
            c.shown.load(Ordering::Relaxed),
            c.dropped.load(Ordering::Relaxed),
            c.repeated.load(Ordering::Relaxed),
        ]
    }

    /// Frames drawn so far.
    pub fn shown(&self) -> u64 {
        self.shared.counts.shown.load(Ordering::Relaxed)
    }

    pub fn last_swap(&self) -> i64 {
        self.shared.counts.last_swap.load(Ordering::Relaxed)
    }

    /// The picture drawn through `preset` from the next draw, `uniforms` set over the preset's values; nothing changes
    /// when it is what the chain draws through already. A picture from a ring, whose slot is given back once drawn, is
    /// drawn through it from the next frame (EmuSen_Qt.md §1.8.10).
    pub fn draw_through(&self, preset: Vec<u8>, uniforms: Vec<(String, f32)>) {
        *locked(&self.shared.through) = Some(Through { preset, uniforms });
    }

    /// A parameter of the chain drawn set to `value` from the next draw, with no chain built again: false, and nothing
    /// asked, when the chain declares no parameter `id` (EmuSen_Qt.md §1.8.10).
    pub fn set_value(&self, id: &str, value: f32) -> bool {
        if !locked(&self.shared.declared).iter().any(|d| d == id) {
            return false;
        }
        let mut live = locked(&self.shared.live);
        live.retain(|(d, _)| d != id);
        live.push((id.to_owned(), value));
        true
    }

    /// The parameters the chain drawn last declares, by id.
    pub fn declared(&self) -> Vec<String> {
        locked(&self.shared.declared).clone()
    }

    /// The folder the filters' presets are written under.
    pub fn presets(&self) -> &Path {
        &self.shared.presets
    }

    /// Whether a frame thread runs, so that a swap with no new frame counts as a repeat.
    pub fn set_running(&self, running: bool) {
        self.shared.running.store(running, Ordering::Relaxed);
    }
}

/// `Feed::draw`: the newest frame advanced into the chain, the chain made first if it is not, every pass drawn.
extern "C" fn draw(context: *mut c_void, width: i32, height: i32, out: *mut Picture) -> i32 {
    // SAFETY: the context is the feed's own, alive while an item names the feed.
    let context = unsafe { &*context.cast::<Context>() };
    let _drawing = locked(&context.shared.drawing);
    let mut render = locked(&context.render);
    let render = &mut *render;
    {
        let mut newest = locked(&context.shared.newest);
        if newest.1 {
            std::mem::swap(&mut newest.0, &mut render.current);
            newest.1 = false;
            render.advanced = false;
            render.fresh = true;
            *locked(&context.shared.taken) += 1;
            context.shared.swapped.notify_all();
        }
    }
    if render.current.width <= 0 {
        return 0;
    }
    let ring = matches!(render.current.source, Source::Ring { .. });
    if !(ring && render.advanced) {
        if let Some(through) = locked(&context.shared.through).take().filter(|t| *t != render.through) {
            render.through = through;
            render.chain = None;
            render.advanced = false;
        }
    }
    if render.chain.is_none() {
        match build(&render.device, &render.through) {
            Ok((chain, values)) => {
                *locked(&context.shared.declared) = values.iter().map(|(id, _)| id.clone()).collect();
                locked(&context.shared.live).clear();
                render.chain = Some(chain);
                render.values = values;
            }
            Err(e) => {
                render.problem = Some(e.to_string());
                return -1;
            }
        }
    }
    let Some(chain) = render.chain.as_mut() else { return -1 };
    let live = std::mem::take(&mut *locked(&context.shared.live));
    if !live.is_empty() {
        for (id, value) in live {
            if let Some(held) = render.values.iter_mut().find(|(d, _)| *d == id) {
                held.1 = value;
            }
        }
        let values: Vec<f32> = render.values.iter().map(|(_, v)| *v).collect();
        if let Err(e) = chain.set_parameters(&values) {
            render.problem = Some(e.to_string());
        }
    }
    if !render.advanced {
        if let Err(e) = advance(chain, &render.current) {
            // A picture that would not advance is not tried again, so no later draw reads a lend or ring it named.
            render.current.width = 0;
            render.problem = Some(e.to_string());
            return -1;
        }
        render.advanced = true;
        if !std::mem::take(&mut render.fresh) {
            return show(chain, width, height, out, &mut render.problem);
        }
        render.fresh_since_swap = true;
        let counts = &context.shared.counts;
        counts.shown.fetch_add(1, Ordering::Relaxed);
        counts.shown_serial.store(render.current.serial, Ordering::Relaxed);
        match render.current.source {
            Source::Lent(_) => counts.shown_lent.fetch_add(1, Ordering::Relaxed),
            Source::Ring { .. } => counts.shown_ring.fetch_add(1, Ordering::Relaxed),
            Source::Pixels => 0,
        };
    }
    show(chain, width, height, out, &mut render.problem)
}

/// The chain for a preset, its uniforms set over the preset's values where its passes declare them, and the values
/// it was given.
fn build(device: &Arc<Device>, through: &Through) -> emusen_platform_client::Result<(Chain, Vec<(String, f32)>)> {
    let mut chain = Chain::new(device, &through.preset)?;
    let mut parameters = chain.parameters()?;
    if parameters.iter().any(|(id, _)| through.uniforms.iter().any(|(u, _)| u == id)) {
        for (id, value) in &mut parameters {
            if let Some((_, given)) = through.uniforms.iter().find(|(u, _)| u == id) {
                *value = *given;
            }
        }
        let values: Vec<f32> = parameters.iter().map(|(_, v)| *v).collect();
        chain.set_parameters(&values)?;
    }
    Ok((chain, parameters))
}

/// Every pass drawn for a view of `width` by `height`, the picture handed to GamePicture.
fn show(chain: &mut Chain, width: i32, height: i32, out: *mut Picture, problem: &mut Option<String>) -> i32 {
    match chain.draw(width, height) {
        Ok(drawn) => {
            // SAFETY: GamePicture passes its own picture to fill.
            unsafe {
                *out = Picture { image: drawn.image, width: drawn.width, height: drawn.height, format: drawn.format, layout: drawn.layout }
            };
            1
        }
        Err(e) => {
            *problem = Some(e.to_string());
            -1
        }
    }
}

/// A frame into the chain from where its picture lies.
fn advance(chain: &mut Chain, frame: &Frame) -> emusen_platform_client::Result<()> {
    match frame.source {
        Source::Pixels => chain.advance(&frame.rgba, frame.width, frame.height, frame.row_repeat),
        // SAFETY: the frame thread names a lend or ring of its machine and frees the machine only after `withdraw`.
        Source::Lent(lend) => NonNull::new(lend as *mut c_void).map_or(Ok(()), |lend| unsafe { chain.advance_lent(lend) }.map(drop)),
        Source::Ring { ring, slot, value } => NonNull::new(ring as *mut c_void)
            .map_or(Ok(()), |ring| unsafe { chain.advance_ring(ring, slot, value, frame.width, frame.height) }),
    }
}

/// `Feed::swapped`: the display clock's tick, counted, and a frame thread waiting on it woken.
extern "C" fn swapped(context: *mut c_void, nanoseconds: i64) {
    // SAFETY: as in `draw`.
    let context = unsafe { &*context.cast::<Context>() };
    let shared = &context.shared;
    shared.counts.swaps.fetch_add(1, Ordering::Relaxed);
    shared.counts.last_swap.store(nanoseconds, Ordering::Relaxed);
    {
        let mut render = locked(&context.render);
        if shared.running.load(Ordering::Relaxed) && !std::mem::take(&mut render.fresh_since_swap) {
            shared.counts.repeated.fetch_add(1, Ordering::Relaxed);
        }
    }
    *locked(&shared.swaps) += 1;
    shared.swapped.notify_all();
}
