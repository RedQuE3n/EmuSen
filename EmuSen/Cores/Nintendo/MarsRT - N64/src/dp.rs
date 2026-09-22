//! The display processor's command interface, the C# `DpInterface`, and the words a snapshot carries.

use crate::Skip;
use crate::bus::{MemoryBus, SP_MEM_SIZE};
use crate::mi::interrupt;
use crate::rdp::{Rdp, RdpMemory};
use crate::state::{State, StateError, StateReader, StateResult, StateWriter};

/// `DpInterface.SnapshotWords`: a snapshot's tail always holds this many words, the unused ones zero.
pub const SNAPSHOT_WORDS: usize = 1 << 15;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct DpInterface {
    pub processor: Rdp,
    pub current: u32,
    pub end: u32,
    pub freeze: bool,
    pub running: bool,
    pub start: u32,
    pub start_valid: bool,
    pub xbus: bool,
    /// The words handed over and not yet run, which only a snapshot carries (`WritePending`); `[SkipInState]` in C#.
    pub pending: Vec<u64>,
    pub framer: Skip<Framer>,
}

impl State for DpInterface {
    fn write_state(&self, w: &mut StateWriter) {
        w.class("Processor", &self.processor);
        w.u32("_current", self.current);
        w.u32("_end", self.end);
        w.bool("_freeze", self.freeze);
        w.bool("_running", self.running);
        w.u32("_start", self.start);
        w.bool("_startValid", self.start_valid);
        w.bool("_xbus", self.xbus);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        r.class(&mut self.processor)?; // Processor
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
    pub fn write_pending(&self, w: &mut StateWriter) {
        w.group("Dp.Pending", |w| {
            w.i32("Count", self.pending.len() as i32);
            w.u64s("Words", &self.pending);
            let padding = SNAPSHOT_WORDS.saturating_sub(self.pending.len());
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

    /// `Take`: everything up to end at once, single-threaded; an address compare, so a list may run past DMEM's end.
    fn dp_take(&mut self) {
        *self.written += 1;
        while !self.dp.freeze && self.dp.current < self.dp.end {
            let word = if self.dp.xbus { self.dp_read_dmem(self.dp.current) } else { self.read64(self.dp.current) };
            self.dp.current = self.dp.current.wrapping_add(8);
            let full = self.rdp_accept(word);
            let framed = self.dp.framer.on && self.dp.framer.frame(word);
            if full || framed {
                self.dp.running = false;
                self.mi.raise(interrupt::DISPLAY_PROCESSOR);
            }
        }
    }

    fn dp_read_dmem(&self, address: u32) -> u64 {
        let mut word = 0u64;
        for i in 0..8u32 {
            word = (word << 8) | self.sp_dmem[(address.wrapping_add(i) & (SP_MEM_SIZE as u32 - 1)) as usize] as u64;
        }
        word
    }

    /// One word to the display processor, over the memories it draws into; true on a full sync.
    #[inline]
    pub fn rdp_accept(&mut self, word: u64) -> bool {
        let mut memory = RdpMemory { rdram: &mut self.rdram, hidden: &mut self.rdram_hidden };
        self.dp.processor.accept(word, &mut memory)
    }

    /// `ReadPending`'s replay: a snapshot's words, run with no sync raised, as C# runs them.
    pub fn dp_replay_pending(&mut self) {
        let p = &self.dp.processor;
        let framer = &mut self.dp.framer;
        framer.taken = p.taken;
        framer.first = p.command[0];
        framer.color = (p.color_image, p.color_image_width as u32, p.color_image_size as u32);
        framer.depth = p.depth_image;
        framer.scissor_bottom = p.scissor_bottom as u32;
        let pending = std::mem::take(&mut self.dp.pending);
        for word in pending {
            self.rdp_accept(word);
            if self.dp.framer.on {
                self.dp.framer.frame(word);
            }
        }
    }
}

/// A measurement aid while the RDP is a stub: frames the words into commands as `Rdp.Gather` does, answers a full sync,
/// and records the RDRAM an image command could draw into. Off by default; see Mars_Native.md §5.2.
#[derive(Clone, Debug, Default)]
pub struct Framer {
    pub on: bool,
    pub taken: i32,
    pub first: u64,
    pub color: (u32, u32, u32),
    pub depth: u32,
    pub scissor_bottom: u32,
    /// The byte ranges primitives were drawn into since the last `take_regions`, as `(start, end)`.
    pub regions: Vec<(u32, u32)>,
}

impl Framer {
    /// `Rdp.Length`: a triangle's words by its coefficients, a textured rectangle's two, one otherwise.
    fn length(id: u32) -> i32 {
        match id {
            0x08..=0x0F => 4 + if id & 4 != 0 { 8 } else { 0 } + if id & 2 != 0 { 8 } else { 0 } + if id & 1 != 0 { 2 } else { 0 },
            0x24 | 0x25 => 2,
            _ => 1,
        }
    }

    /// A range merged into the one it overlaps, so the list stays as short as the images drawn.
    fn note(&mut self, start: u32, end: u32) {
        if let Some(r) = self.regions.iter_mut().find(|r| start <= r.1 && end >= r.0) {
            *r = (r.0.min(start), r.1.max(end));
        } else {
            self.regions.push((start, end));
        }
    }

    /// One word; true when it completed a full sync.
    pub fn frame(&mut self, word: u64) -> bool {
        if self.taken == 0 {
            self.first = word;
        }
        self.taken += 1;
        let id = ((self.first >> 56) & 0x3F) as u32;
        if self.taken < Self::length(id) {
            return false;
        }
        self.taken = 0;
        let w = self.first;
        match id {
            0x3F => self.color = ((w & 0x03FF_FFFF) as u32, (((w >> 32) & 0x3FF) + 1) as u32, ((w >> 51) & 3) as u32),
            0x3E => self.depth = (w & 0x03FF_FFFF) as u32,
            0x2D => self.scissor_bottom = (w & 0xFFF) as u32,
            0x08..=0x0F | 0x24 | 0x25 | 0x36 => {
                let (address, width, size) = self.color;
                let rows = (self.scissor_bottom >> 2) + 1;
                let bytes = (width * rows) << size >> 1;
                self.note(address, address.saturating_add(bytes.max(1)));
                self.note(self.depth, self.depth.saturating_add(width * rows * 2));
            }
            _ => {}
        }
        id == 0x29
    }
}
