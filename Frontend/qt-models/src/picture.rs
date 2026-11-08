//! The game picture's feed (EmuSen_Qt.md §1.8): the newest frame a frame thread handed over, advanced into a Serenity
//! chain and drawn in place on the render thread, where `GamePicture` asks for it; and each swap's time, the display
//! clock the frame thread paces by. The chain is made, used and dropped on the render thread alone, since its calls
//! submit to the queue the scene graph submits to.

use emusen_kit_qt::picture::{Feed, Picture};
use emusen_platform_client::serenity::{Chain, Device};
use std::ffi::c_void;
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
}

/// The render thread's own: the chain and the frame it last advanced from.
struct Render {
    device: Arc<Device>,
    preset: Vec<u8>,
    chain: Option<Chain>,
    current: Frame,
    advanced: bool,
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
    /// A feed drawing through `preset` (a document Serenity's preset readers give) on `device`.
    pub fn new(device: Arc<Device>, preset: Vec<u8>) -> PictureFeed {
        let shared = Arc::new(Shared {
            newest: Mutex::default(),
            swaps: Mutex::new(0),
            taken: Mutex::new(0),
            swapped: Condvar::new(),
            counts: Counts::default(),
            running: AtomicBool::new(false),
            drawing: Mutex::new(()),
        });
        let render =
            Render { device, preset, chain: None, current: Frame::default(), advanced: false, fresh_since_swap: false, problem: None };
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
            *locked(&context.shared.taken) += 1;
            context.shared.swapped.notify_all();
        }
    }
    if render.current.width <= 0 {
        return 0;
    }
    if render.chain.is_none() {
        match Chain::new(&render.device, &render.preset) {
            Ok(chain) => render.chain = Some(chain),
            Err(e) => {
                render.problem = Some(e.to_string());
                return -1;
            }
        }
    }
    let Some(chain) = render.chain.as_mut() else { return -1 };
    if !render.advanced {
        if let Err(e) = advance(chain, &render.current) {
            // A picture that would not advance is not tried again, so no later draw reads a lend or ring it named.
            render.current.width = 0;
            render.problem = Some(e.to_string());
            return -1;
        }
        render.advanced = true;
        render.fresh_since_swap = true;
        context.shared.counts.shown.fetch_add(1, Ordering::Relaxed);
    }
    match chain.draw(width, height) {
        Ok(drawn) => {
            // SAFETY: GamePicture passes its own picture to fill.
            unsafe {
                *out = Picture { image: drawn.image, width: drawn.width, height: drawn.height, format: drawn.format, layout: drawn.layout }
            };
            1
        }
        Err(e) => {
            render.problem = Some(e.to_string());
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
