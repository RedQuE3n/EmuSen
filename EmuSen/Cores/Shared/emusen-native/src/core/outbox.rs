//! What a core tells its host without calling it: events and log records, queued and pulled (§6.18, §6.19).
//! A core calls [`emit`], [`log`] or [`detail`] from anywhere in its code; during a machine call they reach that
//! machine's queues, and otherwise the library's log.

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::sync::Mutex;

use super::sys::{Event, event};

/// Events a machine holds before the oldest are dropped.
pub const EVENT_CAPACITY: usize = 256;
/// Log records a queue holds before the oldest are dropped.
pub const LOG_CAPACITY: usize = 1024;
/// The longest record text, in bytes; longer text is cut at a character boundary.
pub const RECORD_TEXT_MAX: usize = 4096;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    Error,
    Warn,
    Info,
    Debug,
}

impl Level {
    pub const fn name(self) -> &'static str {
        match self {
            Level::Error => "error",
            Level::Warn => "warn",
            Level::Info => "info",
            Level::Debug => "debug",
        }
    }
}

/// A bounded queue of formatted records, each one whole line.
#[derive(Default)]
pub struct LogQueue {
    records: VecDeque<String>,
    dropped: u64,
}

impl LogQueue {
    pub fn push(&mut self, level: Level, category: &str, text: &str) {
        let clean = |s: &str| s.replace(['\t', '\r', '\n'], " ");
        let mut text = clean(text);
        if text.len() > RECORD_TEXT_MAX {
            let mut cut = RECORD_TEXT_MAX;
            while !text.is_char_boundary(cut) {
                cut -= 1;
            }
            text.truncate(cut);
        }
        let category = if category.is_empty() { "core".to_owned() } else { clean(category) };
        if self.records.len() >= LOG_CAPACITY {
            self.records.pop_front();
            self.dropped += 1;
        }
        self.records.push_back(format!("{}\t{}\t{}\n", level.name(), category, text));
    }

    pub fn len(&self) -> usize {
        self.records.len() + (self.dropped > 0) as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn dropped_line(n: u64) -> String {
        format!("warn\tlog\t{n} records dropped\n")
    }

    fn take_dropped(&mut self) {
        if self.dropped > 0 {
            let n = std::mem::take(&mut self.dropped);
            self.records.push_front(Self::dropped_line(n));
        }
    }

    /// The bytes all waiting records take.
    pub fn waiting(&self) -> usize {
        self.records.iter().map(String::len).sum::<usize>() + if self.dropped > 0 { Self::dropped_line(self.dropped).len() } else { 0 }
    }

    /// Whole records copied into `out` and removed, oldest first; the bytes copied.
    pub fn drain(&mut self, out: &mut [u8]) -> usize {
        self.take_dropped();
        let mut n = 0;
        while let Some(r) = self.records.front() {
            if n + r.len() > out.len() {
                break;
            }
            out[n..n + r.len()].copy_from_slice(r.as_bytes());
            n += r.len();
            self.records.pop_front();
        }
        n
    }

    pub fn append(&mut self, other: &mut LogQueue) {
        other.take_dropped();
        for r in other.records.drain(..) {
            if self.records.len() >= LOG_CAPACITY {
                self.records.pop_front();
                self.dropped += 1;
            }
            self.records.push_back(r);
        }
    }
}

/// The library's own records: those made outside any machine call, and those a freed or refused machine left.
pub static LIBRARY_LOG: Mutex<LogQueue> = Mutex::new(LogQueue { records: VecDeque::new(), dropped: 0 });

pub fn library_log() -> std::sync::MutexGuard<'static, LogQueue> {
    LIBRARY_LOG.lock().unwrap_or_else(|e| e.into_inner())
}

/// One machine's queues and the detail of its last failure.
#[derive(Default)]
pub struct Inner {
    pub events: VecDeque<Event>,
    pub log: LogQueue,
    /// A `LOG` event is queued for what the log holds.
    pub announced: bool,
    /// The words the core gave for the failure of the call in flight.
    pub detail: Option<String>,
    pub last_error: String,
}

impl Inner {
    pub fn push_event(&mut self, kind: u32, a: i64, b: i64) {
        let e = Event { size: std::mem::size_of::<Event>() as u32, kind, a, b };
        if self.events.len() >= EVENT_CAPACITY {
            self.events.pop_front();
            if !self.events.iter().any(|e| e.kind == event::MACHINE_INFO) {
                self.events.pop_front();
                self.events.push_back(Event { kind: event::MACHINE_INFO, ..e });
            }
        }
        self.events.push_back(e);
    }

    /// At a call's end: a `LOG` event for records not yet announced.
    pub fn announce(&mut self) {
        if !self.announced && !self.log.is_empty() {
            self.announced = true;
            let n = self.log.len() as i64;
            self.push_event(event::LOG, n, 0);
        }
    }
}

pub type Outbox = RefCell<Inner>;

thread_local! {
    static CURRENT: Cell<*const Outbox> = const { Cell::new(std::ptr::null()) };
}

/// Routes [`emit`], [`log`] and [`detail`] to `outbox` until dropped.
pub struct Route {
    previous: *const Outbox,
}

impl Route {
    pub fn to(outbox: &Outbox) -> Route {
        Route { previous: CURRENT.with(|c| c.replace(outbox)) }
    }
}

impl Drop for Route {
    fn drop(&mut self) {
        CURRENT.with(|c| c.set(self.previous));
    }
}

fn with_current<R>(f: impl FnOnce(Option<&Outbox>) -> R) -> R {
    let p = CURRENT.with(Cell::get);
    // SAFETY: a non-null pointer is a live outbox for as long as the `Route` that set it.
    f(unsafe { p.as_ref() })
}

/// An event for the host, queued on the machine in its call; dropped outside any machine call.
pub fn emit(kind: u32, a: i64, b: i64) {
    with_current(|o| {
        if let Some(o) = o {
            o.borrow_mut().push_event(kind, a, b);
        }
    });
}

/// A log record: the machine's in its call, the library's otherwise.
pub fn log(level: Level, category: &str, text: &str) {
    with_current(|o| match o {
        Some(o) => o.borrow_mut().log.push(level, category, text),
        None => library_log().push(level, category, text),
    });
}

/// The words for the failure the call in flight is about to return, for `last_error` and create's error text.
pub fn detail(text: &str) {
    with_current(|o| {
        if let Some(o) = o {
            o.borrow_mut().detail = Some(text.to_owned());
        }
    });
}
