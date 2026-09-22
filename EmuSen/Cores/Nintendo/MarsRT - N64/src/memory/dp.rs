//! The display processor's command interface, the C# `DpInterface`, and the words a snapshot carries; the threaded half is `dp_threads`.

use crate::Skip;
use crate::memory::bus::{MemoryBus, SP_MEM_SIZE};
use crate::memory::dp_threads::{PAGES, PageMarks, Threads, site};
use crate::memory::mi::interrupt;
use crate::memory::ram::Detached;
use crate::rdp::{Rdp, RdpMemory};
use crate::state::{State, StateError, StateReader, StateResult, StateWriter};
use std::sync::atomic::Ordering::Relaxed;

/// `DpInterface.SnapshotWords`: a snapshot's tail always holds this many words, the unused ones zero.
pub const SNAPSHOT_WORDS: usize = 1 << 15;

/// Dropped before the bus's RDRAM, which `MemoryBus` declares after it, so the drain never outlives the memory it draws into.
#[derive(Default)]
pub struct DpInterface {
    pub processor: Detached<Rdp>,
    pub current: u32,
    pub end: u32,
    pub freeze: bool,
    pub running: bool,
    pub start: u32,
    pub start_valid: bool,
    pub xbus: bool,
    /// The words handed over and not yet run, which only a snapshot carries (`WritePending`); `[SkipInState]` in C#.
    pub pending: Vec<u64>,
    /// `_marks` and `_writeMarks`, all zero unless a drain runs.
    pub marks: Skip<PageMarks>,
    /// The drain and the shadow, while the list runs on a thread of its own.
    pub threads: Skip<Option<Box<Threads>>>,
}

impl Drop for DpInterface {
    fn drop(&mut self) {
        self.threads.take();
    }
}

impl DpInterface {
    /// Everything handed over has run, for a caller holding the machine shared.
    pub fn wait_all(&self) {
        if let Some(t) = self.threads.as_ref() {
            t.wait_all();
        }
    }
}

impl Clone for DpInterface {
    fn clone(&self) -> Self {
        self.wait_all();
        DpInterface {
            processor: self.processor.clone(),
            current: self.current,
            end: self.end,
            freeze: self.freeze,
            running: self.running,
            start: self.start,
            start_valid: self.start_valid,
            xbus: self.xbus,
            pending: self.pending.clone(),
            marks: Skip(PageMarks::default()),
            threads: Skip(None),
        }
    }
}

impl PartialEq for DpInterface {
    fn eq(&self, other: &Self) -> bool {
        self.wait_all();
        other.wait_all();
        self.processor == other.processor
            && (self.current, self.end, self.freeze, self.running, self.start, self.start_valid, self.xbus) == (other.current, other.end, other.freeze, other.running, other.start, other.start_valid, other.xbus)
            && self.pending == other.pending
    }
}

impl Eq for DpInterface {}

impl std::fmt::Debug for DpInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.wait_all();
        f.debug_struct("DpInterface")
            .field("processor", &*self.processor)
            .field("current", &self.current)
            .field("end", &self.end)
            .field("freeze", &self.freeze)
            .field("running", &self.running)
            .field("start", &self.start)
            .field("start_valid", &self.start_valid)
            .field("xbus", &self.xbus)
            .field("pending", &self.pending)
            .field("threaded", &self.threads.is_some())
            .finish()
    }
}

impl State for DpInterface {
    fn write_state(&self, w: &mut StateWriter) {
        w.class("Processor", &*self.processor);
        w.u32("_current", self.current);
        w.u32("_end", self.end);
        w.bool("_freeze", self.freeze);
        w.bool("_running", self.running);
        w.u32("_start", self.start);
        w.bool("_startValid", self.start_valid);
        w.bool("_xbus", self.xbus);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        r.class(&mut *self.processor)?; // Processor
        self.current = r.u32()?; // _current
        self.end = r.u32()?; // _end
        self.freeze = r.bool()?; // _freeze
        self.running = r.bool()?; // _running
        self.start = r.u32()?; // _start
        self.start_valid = r.bool()?; // _startValid
        self.xbus = r.bool()?; // _xbus
        Ok(())
    }
}

impl DpInterface {
    /// `WritePending`: the count, the words, then zeros to `SNAPSHOT_WORDS`.
    pub fn write_pending(&self, w: &mut StateWriter, words: &[u64]) {
        w.group("Dp.Pending", |w| {
            w.i32("Count", words.len() as i32);
            w.u64s("Words", words);
            let padding = SNAPSHOT_WORDS.saturating_sub(words.len());
            w.u64s("Padding", &ZEROS[..padding]);
        });
    }

    /// `ReadPending`: the C# skips the padding unread, and so does this.
    pub fn read_pending(&mut self, r: &mut StateReader) -> StateResult {
        let count = r.i32()?;
        if count < 0 || count as usize > SNAPSHOT_WORDS {
            return Err(StateError::PendingOverflow(count));
        }
        self.pending = vec![0; count as usize];
        r.u64s(&mut self.pending)?;
        r.skip((SNAPSHOT_WORDS - count as usize) * 8)
    }

    /// The words a snapshot carries: a load's words not yet replayed, then the ring's words not yet run.
    pub fn pending_words(&self) -> Vec<u64> {
        let mut words = self.pending.clone();
        if let Some(t) = self.threads.as_ref() {
            words.extend(t.pending_words());
        }
        words
    }

    /// Words pending in all, for a state's refusal and a snapshot's bound.
    pub fn pending_count(&self) -> usize {
        self.pending.len() + self.threads.as_ref().map_or(0, |t| t.pending().max(0) as usize)
    }

    /// `_writeMarks[page] != 0`: a reader of this byte may have to wait. One load and a compare on every fast path.
    #[inline(always)]
    pub fn read_marked(&self, physical: u32) -> bool {
        self.marks.0.write_marks[(physical >> 12) as usize & (PAGES - 1)].load(Relaxed) != 0
    }

    /// `_marks[page] != 0`: a writer of this byte may have to wait.
    #[inline(always)]
    pub fn write_marked(&self, physical: u32) -> bool {
        self.marks.0.marks[(physical >> 12) as usize & (PAGES - 1)].load(Relaxed) != 0
    }

    /// `WaitForRead` over the access's bytes, at a site; the caller has seen the page marked.
    #[cold]
    #[inline(never)]
    pub fn wait_read(&mut self, physical: u32, bytes: u32, site: usize) {
        if let Some(t) = self.threads.as_mut() {
            t.wait_for(physical, bytes, site, false);
        }
    }

    /// `WaitFor` over the access's bytes, at a site; the caller has seen the page marked.
    #[cold]
    #[inline(never)]
    pub fn wait_write(&mut self, physical: u32, bytes: u32, site: usize) {
        if let Some(t) = self.threads.as_mut() {
            t.wait_for(physical, bytes, site, true);
        }
    }

    /// `WaitForReadRange`: every marked page the range reaches.
    #[inline]
    pub fn wait_read_range(&mut self, from: u32, count: u32, site: usize) {
        if let Some(t) = self.threads.as_mut() {
            t.wait_range(from as i64, count as i64, site, false);
        }
    }

    /// `WaitForRange`.
    #[inline]
    pub fn wait_write_range(&mut self, from: u32, count: u32, site: usize) {
        if let Some(t) = self.threads.as_mut() {
            t.wait_range(from as i64, count as i64, site, true);
        }
    }

    /// `Join`: everything handed over has run and the marks are clear; the processor is the machine's again.
    pub fn join(&mut self) {
        if let Some(t) = self.threads.as_mut() {
            t.join();
        }
    }
}

static ZEROS: [u64; SNAPSHOT_WORDS] = [0; SNAPSHOT_WORDS];

pub const STATUS_XBUS: u32 = 0x001;
pub const STATUS_FREEZE: u32 = 0x002;
pub const STATUS_START_GCLK: u32 = 0x008;
pub const STATUS_PIPE_BUSY: u32 = 0x020;
pub const STATUS_BUFFER_READY: u32 = 0x080;
pub const STATUS_START_VALID: u32 = 0x400;
pub const ADDRESS_MASK: u32 = 0x00FF_FFF8;

impl DpInterface {
    /// `StatusWord`: the buffer is never full, because every word handed over has already been taken.
    pub fn status_word(&self) -> u32 {
        (if self.xbus { STATUS_XBUS } else { 0 })
            | (if self.freeze { STATUS_FREEZE } else { 0 })
            | (if self.running { STATUS_START_GCLK | STATUS_PIPE_BUSY } else { 0 })
            | STATUS_BUFFER_READY
            | if self.start_valid { STATUS_START_VALID } else { 0 }
    }

    pub fn read32(&self, offset: u32) -> u32 {
        match offset & 0x1C {
            0x00 => self.start,
            0x04 => self.end,
            0x08 => self.current,
            0x0C => self.status_word(),
            _ => 0,
        }
    }
}

impl MemoryBus {
    pub fn dp_write32(&mut self, offset: u32, value: u32) {
        match offset & 0x1C {
            0x00 => {
                if !self.dp.start_valid {
                    self.dp.start = value & ADDRESS_MASK;
                    self.dp.start_valid = true;
                }
            }
            0x04 => {
                self.dp.end = value & ADDRESS_MASK;
                if self.dp.start_valid {
                    self.dp.current = self.dp.start;
                    self.dp.start_valid = false;
                }
                if !self.dp.freeze {
                    self.dp.running = true;
                }
                self.dp_take();
            }
            0x0C => {
                if value & 3 == 1 {
                    self.dp.xbus = false;
                }
                if value & 3 == 2 {
                    self.dp.xbus = true;
                }
                if value & 0xC == 4 {
                    self.dp.freeze = false;
                    self.dp_take();
                }
                if value & 0xC == 8 {
                    self.dp.freeze = true;
                }
            }
            _ => {}
        }
    }

    /// `Take`: everything up to end at once; an address compare, so a list may run past DMEM's end.
    fn dp_take(&mut self) {
        *self.written += 1;
        if self.dp.threads.is_some() {
            self.dp_take_onto_thread();
            return;
        }
        while !self.dp.freeze && self.dp.current < self.dp.end {
            let word = if self.dp.xbus { self.dp_read_dmem(self.dp.current) } else { self.read64(self.dp.current) };
            self.dp.current = self.dp.current.wrapping_add(8);
            if self.rdp_accept(word) {
                self.dp_full_sync();
            }
        }
    }

    /// `TakeOntoThread`: the same words read now, each marked for then published, the full sync answered at the word that completes it.
    fn dp_take_onto_thread(&mut self) {
        if let Some(t) = self.dp.threads.as_mut() {
            t.begin_batch();
        }
        while !self.dp.freeze && self.dp.current < self.dp.end {
            let word = if self.dp.xbus { self.dp_read_dmem(self.dp.current) } else { self.read64(self.dp.current) };
            self.dp.current = self.dp.current.wrapping_add(8);
            if self.dp.threads.as_mut().is_some_and(|t| t.publish(word)) {
                self.dp_full_sync();
            }
        }
        if let Some(t) = self.dp.threads.as_mut() {
            t.end_batch();
        }
    }

    fn dp_full_sync(&mut self) {
        self.dp.running = false;
        self.mi.raise(interrupt::DISPLAY_PROCESSOR);
    }

    fn dp_read_dmem(&self, address: u32) -> u64 {
        let mut word = 0u64;
        for i in 0..8u32 {
            word = (word << 8) | self.sp_dmem[(address.wrapping_add(i) & (SP_MEM_SIZE as u32 - 1)) as usize] as u64;
        }
        word
    }

    /// One word to the display processor on this thread, over the memories it draws into; true on a full sync.
    #[inline]
    pub fn rdp_accept(&mut self, word: u64) -> bool {
        debug_assert!(self.dp.threads.is_none(), "the processor is the drain's while it runs");
        let mut memory = RdpMemory::new(&mut self.rdram, &mut self.rdram_hidden);
        self.dp.processor.accept(word, &mut memory)
    }

    /// `ReadPending`'s replay: a snapshot's words, run here with no sync raised, as C# runs them.
    pub fn dp_replay_pending(&mut self) {
        let pending = std::mem::take(&mut self.dp.pending);
        for word in pending {
            self.rdp_accept(word);
        }
    }

    /// The list moved onto a drain, or back; a start takes the shadow from the processor (`Threaded`'s setter).
    pub fn dp_set_threaded(&mut self, on: bool, verify: bool) {
        match (on, self.dp.threads.is_some()) {
            (true, false) => {
                debug_assert!(self.dp.pending.is_empty(), "a load's words run before a drain starts");
                let threads = Threads::start(&mut self.dp.processor, &self.rdram, &self.rdram_hidden, &self.dp.marks.0, verify);
                *self.dp.threads = Some(Box::new(threads));
            }
            (false, true) => {
                if let Some(mut t) = self.dp.threads.take() {
                    t.stop();
                }
            }
            _ => {
                if let Some(t) = self.dp.threads.as_ref() {
                    t.set_verifying(verify);
                }
            }
        }
    }

    /// A cheat's read of RDRAM, which waits as C#'s `ReadForCheat` does; past the end reads zero.
    pub fn cheat_read8(&mut self, address: u32) -> u8 {
        if address as usize >= self.rdram.len() {
            return 0;
        }
        if self.dp.read_marked(address) {
            self.dp.wait_read(address, 1, site::CHEAT);
        }
        self.rdram[address as usize]
    }

    /// A cheat's write, which waits as C#'s `WriteForCheat` does; past the end is dropped.
    pub fn cheat_write8(&mut self, address: u32, value: u8) {
        if address as usize >= self.rdram.len() {
            return;
        }
        if self.dp.write_marked(address) {
            self.dp.wait_write(address, 1, site::CHEAT);
        }
        self.rdram[address as usize] = value;
        *self.written += 1;
    }
}
