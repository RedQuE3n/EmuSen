//! A running game as plain Rust (EmuSen_Qt.md §1.2): a session opened through the frontend API, run on a frame thread
//! of its own, its pictures handed to the picture's feed, paced by the window's swaps. No Qt here; `SessionModel`
//! puts it on Qt.

use crate::input::GameInput;
use crate::picture::{Frame, Handoff, Source};
use emusen_crystal_client::shapes::SessionId;
use emusen_crystal_client::FrameInfo;
use emusen_crystal_client::{frame_info, Open, Runtime};
use emusen_platform_client::audio::GameSound;
use emusen_platform_client::pace::DisplayClock;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

/// The longest a frame thread waits for a swap before it runs on its own clock.
const SWAP_BOUND: Duration = Duration::from_millis(100);

/// What the frame thread reports, on its own thread; the model queues each onto the GUI thread.
#[derive(Clone, Debug, PartialEq)]
pub enum Report {
    Opened { session: u32 },
    FirstPicture { after: Duration },
    /// The picture's display shape changed: its width over its height as shown (`shown_aspect`).
    Shape { aspect: f64 },
    Failed(String),
    Closed,
}

/// What the model asks of the frame thread.
#[derive(Default)]
struct Control {
    stop: AtomicBool,
    paused: AtomicBool,
    /// Stereo frames of sound the player has taken in, by its own count.
    sounded: AtomicU64,
}

/// What the game hears and plays through, each when the window has it: the game's side of the pads and the sound player.
#[derive(Default)]
pub struct GameIo {
    pub pads: Option<GameInput>,
    pub sound: Option<GameSound>,
}

/// What a session opens: a game by its file, or a console with nothing in it (Mistress's `StartConsole`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Opening {
    Game(String),
    Console(String),
}

impl From<&str> for Opening {
    fn from(path: &str) -> Opening {
        Opening::Game(path.to_owned())
    }
}

/// What a failure to open is said after, in Mistress's words: "Failed to load {the game's title}" or "Failed to start
/// {console}" (EmuSen_Qt.md §7.9).
pub fn failing(opening: &Opening) -> String {
    match opening {
        Opening::Game(path) => format!("Failed to load {}", crate::library::title_of(path)),
        Opening::Console(console) => format!("Failed to start {console}"),
    }
}

/// A game running on its frame thread.
pub struct Session {
    control: Arc<Control>,
    thread: Option<JoinHandle<()>>,
}

impl Session {
    /// Opens a game or a console (`opening`) with `engine` (none for the settings' choice) and runs it until closed; `device` is the presenter's
    /// Serenity device, lent to the machine; `io` the pads and the sound player the window has; `clock` Serenity's
    /// display clock, fed the window's swaps (EmuSen_Qt.md §1.8.3); `held` opens it paused, for a question asked first.
    #[allow(clippy::too_many_arguments)]
    pub fn start(
        runtime: Runtime,
        opening: Opening,
        engine: Option<String>,
        device: Option<NonNull<std::ffi::c_void>>,
        handoff: Handoff,
        io: GameIo,
        clock: DisplayClock,
        held: bool,
        report: impl Fn(Report) + Send + 'static,
    ) -> Session {
        let (path, console) = match opening {
            Opening::Game(path) => (Some(path), None),
            Opening::Console(console) => (None, Some(console)),
        };
        let control = Arc::new(Control::default());
        control.paused.store(held, Ordering::Relaxed);
        // The device's handle crosses to the frame thread as a number; the handle locks itself (emusen_platform.h).
        let device = device.map_or(0, |d| d.as_ptr() as usize);
        let thread = std::thread::spawn({
            let control = control.clone();
            move || {
                let device = NonNull::new(device as *mut std::ffi::c_void);
                // A machine whose core draws on a ring of its own is lent one wherever the window has a device (§1.8.7).
                let flags = if device.is_some() { emusen_crystal_client::consts::OPEN_RING } else { 0 };
                let open = Open { flags, path, console, engine, device, ..Open::default() };
                run(&runtime, open, &handoff, io, clock, &control, &report)
            }
        });
        Session { control, thread: Some(thread) }
    }

    pub fn set_paused(&self, paused: bool) {
        self.control.paused.store(paused, Ordering::Relaxed);
    }

    pub fn paused(&self) -> bool {
        self.control.paused.load(Ordering::Relaxed)
    }

    /// Stereo frames of the game's sound the player has taken in so far.
    pub fn sounded(&self) -> u64 {
        self.control.sounded.load(Ordering::Relaxed)
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.control.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn run(
    runtime: &Runtime,
    open: Open,
    handoff: &Handoff,
    io: GameIo,
    mut clock: DisplayClock,
    control: &Control,
    report: &impl Fn(Report),
) {
    let (stop, paused) = (&control.stop, &control.paused);
    let (mut input, mut sound) = (io.pads, io.sound);
    let started = Instant::now();
    let session = match runtime.open(&open) {
        Ok(session) => session,
        Err(e) => return report(Report::Failed(e.to_string())),
    };
    if let Some(Err(words)) = input.as_mut().map(|input| input.start(runtime, session, open.path.as_deref().unwrap_or_default())) {
        let _ = runtime.close(session);
        return report(Report::Failed(words));
    }
    // An engine that withholds rewind says why in session_info, which the card shows; the game plays on without it.
    let _ = crate::reel::keep(runtime, session);
    report(Report::Opened { session });
    let mut info = frame_info();
    let mut frame = Frame::default();
    let mut audio = vec![0i16; 8192];
    let mut seen = handoff.swaps();
    let mut taken = handoff.taken();
    let mut first = true;
    let mut shape = 0.0;
    let mut next = Instant::now();
    handoff.set_running(true);
    let failed = Mutex::new(None);
    let mut was_held = false;
    // Whether the game ran a frame: a game closed before it played leaves where it was left off as it was.
    let mut played = false;
    while !stop.load(Ordering::Relaxed) {
        let held = paused.load(Ordering::Relaxed);
        if let Some(input) = input.as_mut() {
            input.set_owned(!held);
        }
        if held && !was_held {
            // No swap comes while the picture is still, so the display clock starts over rather than read the pause as a period (EmuSen_Qt.md §9.3.2).
            clock.restart();
            // A game that stops playing for a while starts its rate control over, as Mistress's does.
            if let Some(Err(e)) = sound.as_mut().map(GameSound::reset) {
                *failed.lock().unwrap_or_else(PoisonError::into_inner) = Some(e.to_string());
                break;
            }
        }
        was_held = held;
        if held {
            handoff.set_running(false);
            seen = handoff.wait_swap(seen, SWAP_BOUND);
            next = Instant::now();
            continue;
        }
        handoff.set_running(true);
        if let Some(Err(words)) = input.as_mut().map(|input| input.frame(runtime, session)) {
            *failed.lock().unwrap_or_else(PoisonError::into_inner) = Some(words);
            break;
        }
        if let Err(e) = runtime.frame(session, &mut info) {
            *failed.lock().unwrap_or_else(PoisonError::into_inner) = Some(e.to_string());
            break;
        }
        played = true;
        let mut handed = false;
        if info.rendered != 0 && info.width > 0 && info.height > 0 && take_picture(runtime, session, &info, &mut frame) {
            taken = handoff.taken();
            let on_device = frame.source != Source::Pixels;
            handoff.hand_over(&mut frame);
            handed = true;
            if std::mem::take(&mut first) {
                report(Report::FirstPicture { after: started.elapsed() });
            }
            let aspect = shown_aspect(&info);
            if aspect != shape {
                shape = aspect;
                report(Report::Shape { aspect });
            }
            // A picture on the device is drawn before the machine runs again, so a frame never waits on a ring slot
            // nobody takes and no lend or ring is remade under a picture still to be drawn (§1.8.7).
            while on_device && handoff.taken() <= taken && !stop.load(Ordering::Relaxed) {
                handoff.wait_taken(taken, SWAP_BOUND);
            }
        }
        // A speed whose sound is not played (turbo, unthrottled) drains it and drops it, as the runtime's frame says.
        let playing = if info.play_audio != 0 { sound.as_mut() } else { None };
        if let Err(words) = play_sound(runtime, session, &mut audio, playing, &control.sounded) {
            *failed.lock().unwrap_or_else(PoisonError::into_inner) = Some(words);
            break;
        }
        let locks = clock.saw(handoff.swaps(), handoff.last_swap()).and_then(|()| clock.locks(info.frame_interval_ns));
        let locks = match locks {
            Ok(locks) => locks,
            Err(e) => {
                *failed.lock().unwrap_or_else(PoisonError::into_inner) = Some(e.to_string());
                break;
            }
        };
        if locks {
            // Locked to the display: the next frame runs once a draw has taken this one, a whole frame before the
            // sync that shows it, so the two never race (EmuSen_Qt.md §1.8.3).
            if handed {
                taken = handoff.wait_taken(taken, SWAP_BOUND);
            } else {
                seen = handoff.wait_swap(seen, SWAP_BOUND);
            }
            next = Instant::now();
        } else if info.frame_interval_ns > 0 {
            next += Duration::from_nanos(info.frame_interval_ns);
            let now = Instant::now();
            if next > now {
                std::thread::sleep(next - now);
            } else {
                next = now;
            }
        }
    }
    handoff.set_running(false);
    handoff.withdraw();
    drop(input);
    // Where the game was left off, written as it closes with its thread stopped, as Mistress's `WriteResumeState` does.
    let left_off = if played { runtime.resume_save(session).err().map(|e| format!("Could not save where you left off: {e}")) } else { None };
    let _ = runtime.close(session);
    let mut failed = failed.into_inner().unwrap_or_else(PoisonError::into_inner);
    if failed.is_none() {
        failed = left_off;
    }
    match failed {
        Some(words) => report(Report::Failed(words)),
        None => report(Report::Closed),
    }
}

/// The picture's shape as shown, width over height: the runtime's display aspect, which the systems table's shape and
/// the core's own declaration give (EmuSen_Serenity.md §2.9), else the frame's own shape, as Mistress letterboxes it.
pub fn shown_aspect(info: &FrameInfo) -> f64 {
    if info.display_aspect > 0.0 {
        info.display_aspect
    } else if info.width > 0 && info.height > 0 {
        info.width as f64 / (info.height as f64 * info.row_repeat.max(1) as f64)
    } else {
        0.0
    }
}

/// The frame's picture into `frame`: where the machine left it on the device (its ring's slot, or its lend), else its
/// pixels read back; false where none could be had.
fn take_picture(runtime: &Runtime, session: SessionId, info: &FrameInfo, frame: &mut Frame) -> bool {
    let ring = (info.picture_in_ring != 0).then(|| runtime.picture_ring(session)).flatten();
    let lend = (info.picture_on_device != 0).then(|| runtime.picture_lend(session)).flatten();
    frame.source = match (ring, lend) {
        (Some(ring), _) => Source::Ring { ring: ring.as_ptr() as usize, slot: info.ring_slot, value: info.ring_value },
        (None, Some(lend)) => Source::Lent(lend.as_ptr() as usize),
        (None, None) => {
            frame.rgba.resize(info.width as usize * info.height as usize * 4, 0);
            if runtime.picture_copy(session, &mut frame.rgba).is_err() {
                return false;
            }
            Source::Pixels
        }
    };
    (frame.width, frame.height, frame.serial) = (info.width, info.height, info.picture_serial);
    frame.row_repeat = if frame.source == Source::Pixels { info.row_repeat.max(1) } else { 1 };
    true
}

/// The frame's sound drained and handed to the player (EmuSen_Qt.md §1.8.6); with none, drained and dropped.
fn play_sound(runtime: &Runtime, session: SessionId, audio: &mut [i16], mut sound: Option<&mut GameSound>, sounded: &AtomicU64) -> Result<(), String> {
    loop {
        let (n, rate) = runtime.audio_drain(session, audio).map_err(|e| e.to_string())?;
        if n == 0 {
            return Ok(());
        }
        if let Some(sound) = sound.as_deref_mut() {
            sound.submit(&audio[..n * 2], rate).map_err(|e| e.to_string())?;
            sounded.store(sound.taken_frames().map_err(|e| e.to_string())? as u64, Ordering::Relaxed);
        }
    }
}
