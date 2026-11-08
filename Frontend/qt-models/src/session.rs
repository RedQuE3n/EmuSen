//! A running game as plain Rust (EmuSen_Qt.md §1.2): a session opened through the frontend API, run on a frame thread
//! of its own, its pictures handed to the picture's feed, paced by the window's swaps. No Qt here; `SessionModel`
//! puts it on Qt.

use crate::picture::{Frame, Handoff};
use emusen_crystal_client::{frame_info, Open, Runtime};
use std::ptr::NonNull;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

/// How far the game's interval may sit from the display's for one frame to run at each swap (EmuSen_Qt.md §1.8.3).
const LOCK_SHARE: f64 = 0.01;
/// The longest a frame thread waits for a swap before it runs on its own clock.
const SWAP_BOUND: Duration = Duration::from_millis(100);

/// The display's period, read from the swaps' times.
#[derive(Default)]
pub struct DisplayClock {
    last: Option<(u64, i64)>,
    period_ns: f64,
}

impl DisplayClock {
    /// A swap's count and time seen; the period is smoothed over the swaps between two looks.
    pub fn saw(&mut self, swaps: u64, at_ns: i64) {
        if let Some((n, t)) = self.last {
            if swaps > n && at_ns > t {
                let one = (at_ns - t) as f64 / (swaps - n) as f64;
                self.period_ns = if self.period_ns == 0.0 { one } else { self.period_ns * 0.9 + one * 0.1 };
            }
        }
        self.last = Some((swaps, at_ns));
    }

    pub fn period_ns(&self) -> f64 {
        self.period_ns
    }

    /// Whether a game whose frames come every `interval_ns` runs one frame a swap.
    pub fn locks(&self, interval_ns: u64) -> bool {
        self.period_ns > 0.0 && interval_ns > 0 && ((interval_ns as f64 - self.period_ns).abs() / self.period_ns) <= LOCK_SHARE
    }
}

/// What the frame thread reports, on its own thread; the model queues each onto the GUI thread.
#[derive(Clone, Debug, PartialEq)]
pub enum Report {
    Opened { session: u32 },
    FirstPicture { after: Duration },
    Failed(String),
    Closed,
}

/// What the model asks of the frame thread.
#[derive(Default)]
struct Control {
    stop: AtomicBool,
    paused: AtomicBool,
}

/// A game running on its frame thread.
pub struct Session {
    control: Arc<Control>,
    thread: Option<JoinHandle<()>>,
}

impl Session {
    /// Opens `path` with `engine` (none for the settings' choice) and runs it until closed; `device` is the presenter's
    /// Serenity device, lent to the machine.
    pub fn start(
        runtime: Runtime,
        path: String,
        engine: Option<String>,
        device: Option<NonNull<std::ffi::c_void>>,
        handoff: Handoff,
        report: impl Fn(Report) + Send + 'static,
    ) -> Session {
        let control = Arc::new(Control::default());
        // The device's handle crosses to the frame thread as a number; the handle locks itself (emusen_platform.h).
        let device = device.map_or(0, |d| d.as_ptr() as usize);
        let thread = std::thread::spawn({
            let control = control.clone();
            move || run(&runtime, path, engine, NonNull::new(device as *mut std::ffi::c_void), &handoff, &control, &report)
        });
        Session { control, thread: Some(thread) }
    }

    pub fn set_paused(&self, paused: bool) {
        self.control.paused.store(paused, Ordering::Relaxed);
    }

    pub fn paused(&self) -> bool {
        self.control.paused.load(Ordering::Relaxed)
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
    path: String,
    engine: Option<String>,
    device: Option<NonNull<std::ffi::c_void>>,
    handoff: &Handoff,
    control: &Control,
    report: &impl Fn(Report),
) {
    let (stop, paused) = (&control.stop, &control.paused);
    let started = Instant::now();
    let open = Open { path: Some(path), engine, device, ..Open::default() };
    let session = match runtime.open(&open) {
        Ok(session) => session,
        Err(e) => return report(Report::Failed(e.to_string())),
    };
    report(Report::Opened { session });
    let mut info = frame_info();
    let mut frame = Frame::default();
    let mut audio = vec![0i16; 8192];
    let mut clock = DisplayClock::default();
    let mut seen = handoff.swaps();
    let mut taken = handoff.taken();
    let mut first = true;
    let mut next = Instant::now();
    handoff.set_running(true);
    let failed = Mutex::new(None);
    while !stop.load(Ordering::Relaxed) {
        if paused.load(Ordering::Relaxed) {
            handoff.set_running(false);
            seen = handoff.wait_swap(seen, SWAP_BOUND);
            next = Instant::now();
            continue;
        }
        handoff.set_running(true);
        if let Err(e) = runtime.frame(session, &mut info) {
            *failed.lock().unwrap_or_else(PoisonError::into_inner) = Some(e.to_string());
            break;
        }
        let mut handed = false;
        if info.rendered != 0 && info.width > 0 && info.height > 0 {
            frame.rgba.resize(info.width as usize * info.height as usize * 4, 0);
            if runtime.picture_copy(session, &mut frame.rgba).is_ok() {
                (frame.width, frame.height, frame.row_repeat, frame.serial) =
                    (info.width, info.height, info.row_repeat.max(1), info.picture_serial);
                taken = handoff.taken();
                handoff.hand_over(&mut frame);
                handed = true;
                if std::mem::take(&mut first) {
                    report(Report::FirstPicture { after: started.elapsed() });
                }
            }
        }
        // The sound is drained and dropped until the game's player is wrapped (EmuSen_Qt.md §1.8.3).
        while matches!(runtime.audio_drain(session, &mut audio), Ok((n, _)) if n > 0) {}
        clock.saw(handoff.swaps(), handoff.last_swap());
        if clock.locks(info.frame_interval_ns) {
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
    let _ = runtime.close(session);
    match failed.into_inner().unwrap_or_else(PoisonError::into_inner) {
        Some(words) => report(Report::Failed(words)),
        None => report(Report::Closed),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_game_locks_to_the_display_only_within_a_hundredth_of_its_period() {
        let mut clock = DisplayClock::default();
        assert!(!clock.locks(16_666_667), "no swap seen yet");
        for n in 0..=10u64 {
            clock.saw(n, n as i64 * 16_666_667);
        }
        assert!((clock.period_ns() - 16_666_667.0).abs() < 1.0);
        assert!(clock.locks(16_639_276), "the NES's 60.0988 Hz on a 60 Hz display");
        assert!(!clock.locks(20_000_000), "a 50 Hz game runs on its own clock");
        clock.saw(12, 12 * 16_666_667);
        assert!((clock.period_ns() - 16_666_667.0).abs() < 1.0, "two swaps between looks are one period each");
    }
}
