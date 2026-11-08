//! The pads read on a thread of their own every tick, each tick's changes handed to a sink in one piece (EmuSen_Kit.md §7.15).
//! The sink is the bridge's `qt_thread().queue`; the thread stops when the reader drops or the sink refuses.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use emusen_platform_client::pads::Pads;

use crate::reading::{Reading, Tick};

/// How often the pads are read: Mistress's pad timer's 16 ms (EmuSen_Input.md §8.12).
pub const TICK: Duration = Duration::from_millis(16);

/// What the thread hands its sink: a tick's changes, or words when opening or a poll failed (each failure once).
pub type Delivery = std::result::Result<Tick, String>;

struct Shared {
    stop: AtomicBool,
    /// SDL's pads to show alone from the next poll, when changed; `Some(None)` shows every pad again.
    only: Mutex<Option<Option<Vec<u32>>>>,
}

/// The reading thread; dropping it stops and joins the thread.
pub struct Reader {
    shared: Arc<Shared>,
    thread: Option<JoinHandle<()>>,
}

impl Reader {
    /// Opens the pads on the new thread, shows `only` of SDL's pads (all for none), then reads them every tick until
    /// stopped; `deliver` returns false to stop it.
    pub fn spawn<O, D>(open: O, only: Option<Vec<u32>>, mut deliver: D) -> Reader
    where
        O: FnOnce() -> std::result::Result<Pads, String> + Send + 'static,
        D: FnMut(Delivery) -> bool + Send + 'static,
    {
        let shared = Arc::new(Shared { stop: AtomicBool::new(false), only: Mutex::new(only.is_some().then_some(only)) });
        let thread_shared = Arc::clone(&shared);
        let thread = std::thread::Builder::new().name("emusen-pads".into()).spawn(move || {
            let pads = match open() {
                Ok(pads) => pads,
                Err(words) => {
                    deliver(Err(words));
                    return;
                }
            };
            let mut reading = Reading::default();
            let mut started = false;
            let mut failed: Option<String> = None;
            while !thread_shared.stop.load(Ordering::Acquire) {
                let only = thread_shared.only.lock().map(|mut only| only.take()).unwrap_or(None);
                let tick = only.map_or(Ok(()), |ids| pads.show_only(ids.as_deref())).and_then(|()| {
                    if started {
                        reading.poll(&pads)
                    } else {
                        reading.start(&pads)
                    }
                });
                let delivery = match tick {
                    Ok(tick) => {
                        started = true;
                        failed = None;
                        (!tick.is_empty()).then_some(Ok(tick))
                    }
                    Err(e) => {
                        let words = e.to_string();
                        (failed.replace(words.clone()).as_ref() != Some(&words)).then_some(Err(words))
                    }
                };
                if delivery.is_some_and(|d| !deliver(d)) {
                    break;
                }
                std::thread::sleep(TICK);
            }
        });
        Reader { shared, thread: thread.ok() }
    }

    /// SDL's pads of these ids alone from the next poll, or every pad for none; a test's, as Endymion's `show_only` is.
    pub fn show_only(&self, ids: Option<Vec<u32>>) {
        if let Ok(mut only) = self.shared.only.lock() {
            *only = Some(ids);
        }
    }
}

impl Drop for Reader {
    fn drop(&mut self) {
        self.shared.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
