//! Watches on memory, the frame log and frozen bytes: the C# `WatchRegistry` (`man watch`), `FrameLogRegistry`
//! (`man framelog`) and `FreezeRegistry` (`man freeze`).

use crate::ordered::Ordered;
use emusen_dotnet::text::equals_ignore_case;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};

/// The emulator's one logging switch, `DebugLogging.MasterEnabled`: whether a watch's hit is said as well as stored.
static LOGGING: AtomicBool = AtomicBool::new(false);

pub fn set_logging(on: bool) {
    LOGGING.store(on, Ordering::Relaxed);
}

pub fn logging() -> bool {
    LOGGING.load(Ordering::Relaxed)
}

/// GDB's three: a store, a load, or either; numbered as the session service writes them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WatchKind {
    Write = 0,
    Read = 1,
    Both = 2,
}

impl WatchKind {
    pub fn from_number(n: u32) -> Option<WatchKind> {
        Some(match n {
            0 => WatchKind::Write,
            1 => WatchKind::Read,
            2 => WatchKind::Both,
            _ => return None,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WatchEvent {
    pub sequence: i64,
    pub kind: WatchKind,
    pub address: i32,
    pub value: u8,
    pub context: String,
}

const MAX_EVENTS: usize = 500;

#[derive(Clone, Debug)]
pub struct Watch {
    pub id: i32,
    pub space: String,
    pub address: i32,
    pub length: i32,
    pub kind: WatchKind,
    events: VecDeque<WatchEvent>,
    sites: Ordered<String, i64>,
    total: i64,
}

#[derive(Default)]
pub struct WatchRegistry {
    watches: Vec<Watch>,
    next_id: i32,
    next_sequence: i64,
    /// What a hit says while logging is on, for the runtime's log.
    pub said: Vec<String>,
}

impl WatchRegistry {
    pub fn new() -> WatchRegistry {
        WatchRegistry { watches: Vec::new(), next_id: 1, next_sequence: 1, said: Vec::new() }
    }

    /// `HasWatches`: false exactly when a store or a load recorded would do nothing.
    pub fn has_watches(&self) -> bool {
        !self.watches.is_empty()
    }

    pub fn add(&mut self, space: &str, address: i32, length: i32, kind: WatchKind) -> i32 {
        let id = self.next_id;
        self.next_id += 1;
        self.watches.push(Watch { id, space: space.to_string(), address, length, kind, events: VecDeque::new(), sites: Ordered::new(), total: 0 });
        id
    }

    pub fn remove(&mut self, id: i32) -> bool {
        let before = self.watches.len();
        self.watches.retain(|w| w.id != id);
        self.watches.len() < before
    }

    pub fn watches(&self) -> &[Watch] {
        &self.watches
    }

    /// The last `count` events of a watch, oldest first.
    pub fn events(&self, id: i32, count: i32) -> Vec<WatchEvent> {
        let Some(w) = self.watches.iter().find(|w| w.id == id) else { return Vec::new() };
        let skip = w.events.len().saturating_sub(count.max(0) as usize);
        w.events.iter().skip(skip).cloned().collect()
    }

    /// Every site's count, most first, never losing one to the ring's eviction.
    pub fn sites(&self, id: i32) -> Vec<(String, i64)> {
        let Some(w) = self.watches.iter().find(|w| w.id == id) else { return Vec::new() };
        let mut sites: Vec<(String, i64)> = w.sites.iter().map(|(k, &v)| (k.clone(), v)).collect();
        sites.sort_by(|a, b| b.1.cmp(&a.1));
        sites
    }

    /// The events recorded in all and those still held.
    pub fn counts(&self, id: i32) -> (i64, usize) {
        self.watches.iter().find(|w| w.id == id).map_or((0, 0), |w| (w.total, w.events.len()))
    }

    pub fn clear_events(&mut self, id: i32) {
        if let Some(w) = self.watches.iter_mut().find(|w| w.id == id) {
            w.events.clear();
            w.sites.clear();
            w.total = 0;
        }
    }

    /// `RecordWrite` and `RecordRead`: `context` is made only for an access a watch matches.
    pub fn record(&mut self, kind: WatchKind, space: &str, address: i32, value: u8, context: impl Fn() -> String) {
        for w in &mut self.watches {
            if w.kind != WatchKind::Both && w.kind != kind {
                continue;
            }
            if !equals_ignore_case(&w.space, space) || address < w.address || address >= w.address.wrapping_add(w.length) {
                continue;
            }
            let context = context();
            let sequence = self.next_sequence;
            self.next_sequence += 1;
            w.events.push_back(WatchEvent { sequence, kind, address, value, context: context.clone() });
            if w.events.len() > MAX_EVENTS {
                w.events.pop_front();
            }
            *w.sites.entry(context.clone(), || 0) += 1;
            w.total += 1;
            if logging() {
                let tag = if kind == WatchKind::Write { "W" } else { "R" };
                self.said.push(format!("[WATCH #{}] {tag} {space}:0x{address:X} = 0x{value:02X} ({context})", w.id));
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sample {
    pub frame: i64,
    pub value: i64,
}

const MAX_SAMPLES: usize = 3600;

#[derive(Clone, Debug)]
pub struct FrameLogEntry {
    pub id: i32,
    pub space: String,
    pub address: i32,
    pub width: i32,
    samples: VecDeque<Sample>,
}

#[derive(Default)]
pub struct FrameLogRegistry {
    entries: Vec<FrameLogEntry>,
    next_id: i32,
}

impl FrameLogRegistry {
    pub fn new() -> FrameLogRegistry {
        FrameLogRegistry { entries: Vec::new(), next_id: 1 }
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn add(&mut self, space: &str, address: i32, width: i32) -> i32 {
        let id = self.next_id;
        self.next_id += 1;
        self.entries.push(FrameLogEntry { id, space: space.to_string(), address, width, samples: VecDeque::new() });
        id
    }

    pub fn remove(&mut self, id: i32) -> bool {
        let before = self.entries.len();
        self.entries.retain(|e| e.id != id);
        self.entries.len() < before
    }

    pub fn entries(&self) -> &[FrameLogEntry] {
        &self.entries
    }

    pub fn samples(&self, id: i32, count: i32) -> Vec<Sample> {
        let Some(e) = self.entries.iter().find(|e| e.id == id) else { return Vec::new() };
        let skip = e.samples.len().saturating_sub(count.max(0) as usize);
        e.samples.iter().skip(skip).copied().collect()
    }

    pub fn clear_samples(&mut self, id: i32) {
        if let Some(e) = self.entries.iter_mut().find(|e| e.id == id) {
            e.samples.clear();
        }
    }

    /// `RecordFrame`: each entry's value read this frame.
    pub fn record_frame(&mut self, frame: i64, mut read: impl FnMut(&str, i32, i32) -> i64) {
        for e in &mut self.entries {
            let value = read(&e.space, e.address, e.width);
            if e.samples.len() == MAX_SAMPLES {
                e.samples.pop_front();
            }
            e.samples.push_back(Sample { frame, value });
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frozen {
    pub id: i32,
    pub space: String,
    pub address: i32,
    pub value: u8,
    pub blocked: i64,
}

#[derive(Default)]
pub struct FreezeRegistry {
    frozen: Vec<Frozen>,
    next_id: i32,
}

impl FreezeRegistry {
    pub fn new() -> FreezeRegistry {
        FreezeRegistry { frozen: Vec::new(), next_id: 1 }
    }

    pub fn count(&self) -> usize {
        self.frozen.len()
    }

    /// A byte pinned, or a pinned byte's value changed.
    pub fn add(&mut self, space: &str, address: i32, value: u8) -> i32 {
        if let Some(f) = self.frozen.iter_mut().find(|f| f.address == address && equals_ignore_case(&f.space, space)) {
            f.value = value;
            return f.id;
        }
        let id = self.next_id;
        self.next_id += 1;
        self.frozen.push(Frozen { id, space: space.to_string(), address, value, blocked: 0 });
        id
    }

    pub fn remove(&mut self, id: i32) -> bool {
        let before = self.frozen.len();
        self.frozen.retain(|f| f.id != id);
        self.frozen.len() < before
    }

    pub fn clear(&mut self) {
        self.frozen.clear();
    }

    pub fn all(&self) -> &[Frozen] {
        &self.frozen
    }

    /// `Enforce`: each pinned byte found changed is written back, once a frame.
    pub fn enforce(&mut self, mut read: impl FnMut(&str, i32) -> u8, mut write: impl FnMut(&str, i32, u8)) {
        for f in &mut self.frozen {
            if read(&f.space, f.address) == f.value {
                continue;
            }
            f.blocked += 1;
            write(&f.space, f.address, f.value);
        }
    }
}
