//! The display processor's command interface, the C# `DpInterface`, and the words a snapshot carries; the threaded half is `dp_threads`.

use crate::Skip;
use crate::memory::bus::{MemoryBus, SP_MEM_SIZE};
use crate::memory::dp_threads::{PAGES, PageMarks, ScaledStart, Threads, site};
use crate::memory::mi::interrupt;
use crate::memory::ram::{Detached, Ram};
use crate::rdp::gpu::{GpuDevice, GpuRasteriser};
use crate::rdp::{Rdp, RdpMemory};
use crate::state::{State, StateError, StateReader, StateResult, StateWriter};
use std::sync::atomic::Ordering::Relaxed;

/// `DpInterface.SnapshotWords`: a snapshot's tail always holds this many words, the unused ones zero.
pub const SNAPSHOT_WORDS: usize = 1 << 15;

/// The picture drawn at a multiple beside the machine's own: the scale, a shadow of RDRAM and its hidden bits at the scale squared,
/// and the direct paths' processor at the multiple, which nothing in the machine reads. C#'s `_scale`, `_scaledRdram`, `_scaledHidden`
/// and `_scaledProcessor` (Mars_Rdp.md §11, Mars_Native.md §6.4).
pub struct ScaledDrawing {
    pub scale: i32,
    pub rdram: Ram,
    pub hidden: Ram,
    pub processor: Option<Detached<Rdp>>,
    /// The compute device the multiple is shaded on, when one was asked for and one exists, and what the asking got (Mars_Gpu.md §11).
    pub wants_gpu: bool,
    pub gpu: Option<Box<GpuRasteriser>>,
    pub gpu_report: String,
    /// Counts the device's scans and its rebuilds, so a job can tell whether the picture the device holds is still its own (Mars_Gpu.md §14).
    pub scan_outs: u64,
}

impl Default for ScaledDrawing {
    fn default() -> Self {
        ScaledDrawing { scale: 1, rdram: Ram::zeroed(0), hidden: Ram::zeroed(0), processor: None, wants_gpu: false, gpu: None, gpu_report: "off".into(), scan_outs: 0 }
    }
}

impl ScaledDrawing {
    /// `RebuildGpu`: the device path made or dropped to match the scale and the wish; a rasteriser whose memory still fits is kept and emptied, as `ResetScaled` empties it.
    fn rebuild_gpu(&mut self) {
        self.scan_outs += 1;
        if !self.wants_gpu || self.scale <= 1 {
            self.gpu = None;
            self.gpu_report = if self.wants_gpu { "off at one".into() } else { "off".into() };
            return;
        }
        if let Some(mut gpu) = self.gpu.take()
            && gpu.memory_words() as usize == self.rdram.len() / 2
        {
            gpu.clear();
            self.gpu = Some(gpu);
            return;
        }
        match GpuDevice::try_create(None) {
            Err(report) => self.gpu_report = report,
            Ok(device) => {
                // The device's name alone, as C#'s `GpuRasteriser.TryCreate` reports it; `GpuDevice::report` adds the version.
                match GpuRasteriser::try_create(device, self.rdram.len()) {
                    Ok(gpu) => {
                        self.gpu_report = gpu.device_name().to_string();
                        self.gpu = Some(Box::new(gpu));
                    }
                    Err((_, report)) => self.gpu_report = report,
                }
            }
        }
    }

    /// `CanScanOut`: the device holds the memory at the multiple and can walk the picture out of it itself (Mars_Gpu.md §13).
    pub fn can_scan_out(&self) -> bool {
        self.gpu.is_some()
    }

    /// `NewScaled`'s tail: the direct paths' processor made to shade on the device when there is one.
    fn shade_on_device(&mut self) {
        let on = self.gpu.is_some();
        if let Some(p) = self.processor.as_mut() {
            p.shade_on_device(on);
        }
    }

    fn gpu_pointer(&mut self) -> *mut GpuRasteriser {
        self.gpu.as_mut().map_or(std::ptr::null_mut(), |g| &mut **g as *mut GpuRasteriser)
    }
}

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
    /// The drawing at a multiple, `[SkipInState]` in C#: the picture's, never the machine's.
    pub multiple: Skip<ScaledDrawing>,
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
            multiple: Skip(ScaledDrawing::default()),
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
            .field("scale", &self.multiple.scale)
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
        self.marks.write_marks[(physical >> 12) as usize & (PAGES - 1)].load(Relaxed) != 0
    }

    /// `_marks[page] != 0`: a writer of this byte may have to wait.
    #[inline(always)]
    pub fn write_marked(&self, physical: u32) -> bool {
        self.marks.marks[(physical >> 12) as usize & (PAGES - 1)].load(Relaxed) != 0
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

    /// `WaitForReadRange` as C# waits, for every pending draw on the range's marked pages, which a capture at a multiple keeps (Mars_Native.md §6.14).
    #[inline]
    pub fn wait_read_range_whole(&mut self, from: u32, count: u32, site: usize) {
        if let Some(t) = self.threads.as_mut() {
            t.wait_range_whole(from as i64, count as i64, site);
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

    /// `Scale`: the multiple the picture is drawn at beside the machine's own, one when it is not.
    #[inline(always)]
    pub fn scale(&self) -> i32 {
        self.multiple.scale
    }

    /// `ScaledDrawn`: a draw has reached the scaled memory since it was last emptied, which is when the scan-out may show it (Mars_Rdp.md §11).
    pub fn scaled_drawn(&self) -> bool {
        let m = &self.multiple.0;
        if m.scale <= 1 {
            return false;
        }
        if let Some(t) = self.threads.as_ref() {
            return t.scaled_drawn();
        }
        m.processor.as_ref().is_some_and(|p| p.drew())
    }

    /// `GpuReport`: what the device setting actually got, which is a sentence when it got nothing.
    pub fn gpu_report(&self) -> &str {
        &self.multiple.gpu_report
    }

    /// `ScanOut`: the VI's walk over the device's own memory, once the drawing is finished, submitted without waiting (Mars_Gpu.md §14).
    pub fn scan_out(&mut self, scan: &crate::rdp::gpu::ScanParameters) {
        self.join();
        let m = &mut self.multiple.0;
        if let Some(gpu) = m.gpu.as_mut() {
            gpu.scan_out(scan);
            m.scan_outs += 1;
        }
    }

    /// `ScanIntoRaster`: the VI's clears and walk into the raster the device keeps while it averages, submitted without waiting (Mars_Gpu.md §15).
    #[allow(clippy::too_many_arguments)]
    pub fn scan_into_raster(&mut self, width: usize, height: usize, side: usize, seed: &[u8], clear: bool, spans: &[u32], walk: bool, scan: &crate::rdp::gpu::ScanParameters) {
        self.join();
        let m = &mut self.multiple.0;
        if let Some(gpu) = m.gpu.as_mut() {
            gpu.scan_into_raster(width, height, side, seed, clear, spans, walk, scan);
            m.scan_outs += 1;
        }
    }

    /// `ReadBackScaled`: what the device holds becomes the shadow's bytes, which is the one place a walk on the processor reads them (Mars_Gpu.md §11.2).
    pub fn read_back_scaled(&mut self, from: u32, count: u32) {
        if self.multiple.gpu.is_none() || count == 0 {
            return;
        }
        self.join();
        let m = &mut self.multiple.0;
        if let Some(gpu) = m.gpu.as_mut() {
            gpu.read(from as usize, count as usize, &mut m.rdram, &mut m.hidden);
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

    /// One word to the display processor on this thread, over the memories it draws into, and then to the processor at the multiple; true on a full sync.
    #[inline]
    pub fn rdp_accept(&mut self, word: u64) -> bool {
        debug_assert!(self.dp.threads.is_none(), "the processor is the drain's while it runs");
        let sync = {
            let mut memory = RdpMemory::new(&mut self.rdram, &mut self.rdram_hidden);
            self.dp.processor.accept(word, &mut memory)
        };
        let scaled = &mut self.dp.multiple.0;
        let gpu = scaled.gpu_pointer();
        if let Some(p) = scaled.processor.as_mut() {
            let mut memory = RdpMemory::scaled(&mut scaled.rdram, &mut scaled.hidden, &self.rdram).with_gpu(gpu);
            p.accept(word, &mut memory);
        }
        sync
    }

    /// `Gpu`'s setter: the device asked for or not; the drain, if one runs, is stopped first and restarted by the caller.
    pub fn dp_set_gpu(&mut self, on: bool) {
        if on == self.dp.multiple.wants_gpu {
            return;
        }
        if let Some(mut t) = self.dp.threads.take() {
            t.stop();
        }
        let scaled = &mut self.dp.multiple.0;
        scaled.wants_gpu = on;
        scaled.rebuild_gpu();
        if scaled.scale > 1 {
            scaled.processor = Some(Detached::new(Rdp::new_scaled(&self.dp.processor, scaled.scale)));
            scaled.shade_on_device();
        }
    }

    /// `Scale`'s setter: a shadow at the multiple squared and a processor at the multiple, or none at one; the drain, if one runs,
    /// is stopped first and restarted by the caller. A change empties the shadow, as a state read does (`ResetScaled`).
    pub fn dp_set_scale(&mut self, scale: i32) {
        let scale = scale.clamp(1, 8);
        if scale == self.dp.multiple.scale {
            return;
        }
        if let Some(mut t) = self.dp.threads.take() {
            t.stop();
        }
        let area = (scale * scale) as usize;
        let scaled = &mut self.dp.multiple.0;
        scaled.scale = scale;
        scaled.rdram = Ram::zeroed(if scale > 1 { self.rdram.len() * area } else { 0 });
        scaled.hidden = Ram::zeroed(if scale > 1 { self.rdram_hidden.len() * area } else { 0 });
        scaled.rebuild_gpu();
        scaled.processor = (scale > 1).then(|| Detached::new(Rdp::new_scaled(&self.dp.processor, scale)));
        scaled.shade_on_device();
    }

    /// `ReadPending`'s replay: a snapshot's words, run here with no sync raised, as C# runs them.
    pub fn dp_replay_pending(&mut self) {
        let pending = std::mem::take(&mut self.dp.pending);
        for word in pending {
            self.rdp_accept(word);
        }
    }

    /// The list moved onto workers, or back; a start takes the shadow from the processor (`Threaded`'s and `Workers`' setters).
    pub fn dp_set_threaded(&mut self, on: bool, verify: bool, workers: usize) {
        if on && self.dp.threads.as_ref().is_some_and(|t| t.workers() != workers.clamp(1, 8)) {
            self.dp_set_threaded(false, verify, workers);
        }
        match (on, self.dp.threads.is_some()) {
            (true, false) => {
                debug_assert!(self.dp.pending.is_empty(), "a load's words run before a drain starts");
                let scaled = &mut self.dp.multiple.0;
                let gpu = scaled.gpu_pointer();
                let at_multiple = scaled.processor.as_mut().map(|p| ScaledStart { processor: p, rdram: &scaled.rdram, hidden: &scaled.hidden, scale: scaled.scale, gpu });
                let threads = Threads::start(&mut self.dp.processor, &self.rdram, &self.rdram_hidden, &self.dp.marks.0.0, verify, workers, at_multiple);
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
