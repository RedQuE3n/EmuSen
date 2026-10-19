//! Whether control flow ever reached an address, and the call chain: the C# `CoverageRegistry` (`man cov`) and
//! `CallStackRegistry` (`man bt`, `man profile`).

use crate::breakpoints::CallKind;
use crate::ordered::Ordered;

const ADDRESS_SPACE: usize = 0x100_0000;
const BITMAP_BYTES: usize = ADDRESS_SPACE / 8;
/// A saved map carries this, so a stale file fails loudly.
const MAGIC: [u8; 5] = *b"EMCV\x01";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct EntryPoint {
    /// The kind's number, kept whole from a saved map as .NET keeps an enum's value it has no member for.
    kind: u8,
    entries: i64,
}

#[derive(Default)]
pub struct CoverageRegistry {
    seen: Option<Vec<u8>>,
    instructions: i64,
    mark: Option<Vec<u8>>,
    entry_points: Ordered<i32, EntryPoint>,
    armed: bool,
}

fn bit(map: &[u8], address: i32) -> bool {
    let index = (address & 0xFF_FFFF) as usize;
    map[index >> 3] & (1 << (index & 7)) != 0
}

impl CoverageRegistry {
    pub fn new() -> CoverageRegistry {
        CoverageRegistry::default()
    }

    pub fn is_armed(&self) -> bool {
        self.armed
    }

    pub fn instructions(&self) -> i64 {
        self.instructions
    }

    pub fn has_mark(&self) -> bool {
        self.mark.is_some()
    }

    pub fn entry_point_count(&self) -> usize {
        self.entry_points.len()
    }

    /// The bitmap, two megabytes, made the first time it is armed.
    pub fn arm(&mut self) {
        self.seen.get_or_insert_with(|| vec![0; BITMAP_BYTES]);
        self.armed = true;
    }

    pub fn disarm(&mut self) {
        self.armed = false;
    }

    pub fn clear(&mut self) {
        if let Some(seen) = self.seen.as_mut() {
            seen.fill(0);
        }
        self.instructions = 0;
        self.mark = None;
        self.entry_points.clear();
    }

    /// `Merge`: a bitmap the core recorded itself over this one's first bytes, and the instructions it stands for.
    pub fn merge(&mut self, bitmap: &[u8], instructions: i64) {
        if !self.armed {
            return;
        }
        let seen = self.seen.get_or_insert_with(|| vec![0; BITMAP_BYTES]);
        let n = bitmap.len().min(BITMAP_BYTES);
        for (s, b) in seen[..n].iter_mut().zip(&bitmap[..n]) {
            *s |= b;
        }
        self.instructions += instructions;
    }

    /// From the call stack, for every call and interrupt taken.
    pub fn record_entry_point(&mut self, address: i32, kind: CallKind) {
        if !self.armed {
            return;
        }
        self.entry_points.entry(address & 0xFF_FFFF, || EntryPoint { kind: kind as u8, entries: 0 }).entries += 1;
    }

    pub fn was_executed(&self, address: i32) -> bool {
        self.seen.as_deref().is_some_and(|s| bit(s, address))
    }

    fn was_marked(&self, address: i32) -> bool {
        self.mark.as_deref().is_some_and(|m| bit(m, address))
    }

    pub fn count_executed(&self, address: i32, length: i32) -> i32 {
        (0..length).filter(|&i| self.was_executed(address.wrapping_add(i))).count() as i32
    }

    /// Which addresses of a range ran, at most `limit` of them.
    pub fn executed_addresses(&self, address: i32, length: i32, limit: i32) -> Vec<i32> {
        let mut hits = Vec::new();
        let mut i = 0;
        while i < length && (hits.len() as i32) < limit {
            if self.was_executed(address.wrapping_add(i)) {
                hits.push(address.wrapping_add(i));
            }
            i += 1;
        }
        hits
    }

    /// `Mark`: what has run so far frozen, so `new_since_mark` can answer what ran after it.
    pub fn mark(&mut self) {
        self.mark = Some(match &self.seen {
            None => vec![0; BITMAP_BYTES],
            Some(seen) => seen.clone(),
        });
    }

    pub fn clear_mark(&mut self) {
        self.mark = None;
    }

    pub fn new_since_mark(&self, address: i32, length: i32, limit: i32) -> Vec<i32> {
        let mut fresh = Vec::new();
        if self.mark.is_none() {
            return fresh;
        }
        let mut i = 0;
        while i < length && (fresh.len() as i32) < limit {
            let at = address.wrapping_add(i);
            if self.was_executed(at) && !self.was_marked(at) {
                fresh.push(at);
            }
            i += 1;
        }
        fresh
    }

    pub fn count_new_since_mark(&self, address: i32, length: i32) -> i32 {
        if self.mark.is_none() {
            return 0;
        }
        (0..length).filter(|&i| self.was_executed(address.wrapping_add(i)) && !self.was_marked(address.wrapping_add(i))).count() as i32
    }

    /// Discovered routines, most entered first, then by address.
    pub fn entry_points(&self, limit: i32) -> Vec<(i32, u8, i64)> {
        let mut all: Vec<(i32, u8, i64)> = self.entry_points.iter().map(|(&a, e)| (a, e.kind, e.entries)).collect();
        all.sort_by(|a, b| b.2.cmp(&a.2).then(a.0.cmp(&b.0)));
        all.truncate(limit.max(0) as usize);
        all
    }

    pub fn entry_points_in(&self, address: i32, length: i32) -> Vec<i32> {
        let mut found: Vec<i32> = self.entry_points.keys().copied().filter(|&a| a >= address && a < address.wrapping_add(length)).collect();
        found.sort();
        found
    }

    /// Executed bytes, the range's length, and those new since the mark.
    pub fn statistics(&self, address: i32, length: i32) -> (i32, i32, i32) {
        (self.count_executed(address, length), length, self.count_new_since_mark(address, length))
    }

    /// `Export`: the magic, the entry points and the bitmap, for `cov save`.
    pub fn export(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(MAGIC.len() + 4 + self.entry_points.len() * 13 + BITMAP_BYTES);
        out.extend_from_slice(&MAGIC);
        out.extend_from_slice(&(self.entry_points.len() as i32).to_le_bytes());
        for (&address, e) in self.entry_points.iter() {
            out.extend_from_slice(&address.to_le_bytes());
            out.push(e.kind);
            out.extend_from_slice(&e.entries.to_le_bytes());
        }
        match &self.seen {
            Some(seen) => out.extend_from_slice(seen),
            None => out.resize(out.len() + BITMAP_BYTES, 0),
        }
        out
    }

    /// `Import`: a saved map merged into what is recorded; false for bytes that are not one.
    pub fn import(&mut self, data: &[u8]) -> bool {
        if data.len() < MAGIC.len() + 4 || data[..MAGIC.len()] != MAGIC {
            return false;
        }
        let mut at = MAGIC.len();
        let count = i32::from_le_bytes(data[at..at + 4].try_into().unwrap());
        at += 4;
        if count < 0 || (data.len() as i64) < at as i64 + count as i64 * 13 + BITMAP_BYTES as i64 {
            return false;
        }
        for _ in 0..count {
            let address = i32::from_le_bytes(data[at..at + 4].try_into().unwrap());
            let kind = data[at + 4];
            let entries = i64::from_le_bytes(data[at + 5..at + 13].try_into().unwrap());
            at += 13;
            let e = self.entry_points.entry(address, || EntryPoint { kind, entries: 0 });
            e.entries += entries;
        }
        let seen = self.seen.get_or_insert_with(|| vec![0; BITMAP_BYTES]);
        for (s, b) in seen.iter_mut().zip(&data[at..at + BITMAP_BYTES]) {
            *s |= b;
        }
        true
    }
}

/// One frame of the live call stack.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CallFrame {
    pub source: i32,
    pub target: i32,
    pub kind: CallKind,
    pub frame: i64,
}

/// A runaway chain must not grow without bound.
const MAX_DEPTH: usize = 512;

#[derive(Default)]
pub struct CallStackRegistry {
    frames: Vec<CallFrame>,
    exclusive: Ordered<i32, i64>,
    calls: Ordered<i32, i64>,
    profiled: i64,
    unmatched: i64,
    profiling: bool,
}

impl CallStackRegistry {
    pub fn new() -> CallStackRegistry {
        CallStackRegistry::default()
    }

    pub fn is_profiling(&self) -> bool {
        self.profiling
    }

    pub fn profiled_instructions(&self) -> i64 {
        self.profiled
    }

    pub fn unmatched_returns(&self) -> i64 {
        self.unmatched
    }

    pub fn depth(&self) -> i32 {
        self.frames.len() as i32
    }

    pub fn arm_profiler(&mut self) {
        self.profiling = true;
    }

    pub fn disarm_profiler(&mut self) {
        self.profiling = false;
    }

    pub fn clear_profile(&mut self) {
        self.exclusive.clear();
        self.calls.clear();
        self.profiled = 0;
    }

    pub fn reset(&mut self) {
        self.frames.clear();
        self.unmatched = 0;
    }

    pub fn frames(&self) -> &[CallFrame] {
        &self.frames
    }

    /// Innermost first, the order a backtrace prints in.
    pub fn backtrace(&self) -> impl Iterator<Item = &CallFrame> {
        self.frames.iter().rev()
    }

    /// `NotePush`, stamped with `frame`; the target is the coverage's entry point even past the depth cap.
    pub fn note_push(&mut self, source: i32, target: i32, kind: CallKind, frame: i64, coverage: Option<&mut CoverageRegistry>) {
        if let Some(c) = coverage {
            c.record_entry_point(target, kind);
        }
        if self.frames.len() >= MAX_DEPTH {
            return;
        }
        self.frames.push(CallFrame { source, target, kind, frame });
        if self.profiling {
            *self.calls.entry(target, || 0) += 1;
        }
    }

    pub fn note_pop(&mut self) {
        if self.frames.pop().is_none() {
            self.unmatched += 1;
        }
    }

    /// `NoteInstructions`: a run a core counted itself, charged to the routine innermost while it ran.
    pub fn note_instructions(&mut self, owner: i32, instructions: i64) {
        if !self.profiling || instructions <= 0 {
            return;
        }
        self.profiled += instructions;
        *self.exclusive.entry(owner, || 0) += instructions;
    }

    /// Hottest first, with each routine's calls and share; address 0 is outside any recorded call.
    pub fn hottest(&self, limit: i32) -> Vec<(i32, i64, i64, f64)> {
        if self.profiled == 0 {
            return Vec::new();
        }
        let mut rows: Vec<(i32, i64)> = self.exclusive.iter().map(|(&a, &n)| (a, n)).collect();
        rows.sort_by(|a, b| b.1.cmp(&a.1));
        rows.truncate(limit.max(0) as usize);
        rows.into_iter().map(|(a, n)| (a, n, self.calls.get(&a).copied().unwrap_or(0), n as f64 * 100.0 / self.profiled as f64)).collect()
    }
}
