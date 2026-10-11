//! The PlayStation's bus as the R3000A meets it (psx-spx "Memory Map", "I/O Map", "Memory Control"): the segments and
//! their mirrors, RAM, the scratchpad, the BIOS, the expansion regions and the I/O ports; the I-cache and the write
//! queue; every access timed (Diamond_Plan.md §5.2) and the clock owned here; the events of the clock domains and the
//! DMA's moves. Diamond_Native.md §11 records each rule and its source.

use blackmoon_gte::Gte;
use blackmoon_r3000::{Bus, Size};

use crate::devices::{CdromStub, Duart, MdecStub, SioStub, SpuStub, Tty};
use crate::sio0::Sio0;
use crate::dma::{self, Dma, chcr};
use crate::gpu::Gpu;
use crate::icache::{ICache, Lookup};
use crate::irq::{self, Irq};
use crate::memctl::{self, MemControl, Timing};
use crate::timers::{self, Timers};
use crate::video::{Video, edge};

pub const RAM_BYTES: usize = 0x20_0000;
pub const SCRATCH_BYTES: usize = 0x400;
pub const BIOS_BYTES: usize = 0x8_0000;

/// The stall of a RAM load beyond its instruction's cycle, and of an internal I/O register's (ps1-tests'
/// `access-time` on a console, Diamond_Disputes.md D-4).
pub const RAM_STALL: u64 = 4;
pub const IO_STALL: u64 = 2;
/// A store's time to leave the write queue for RAM (dma/chain-looping's console log, Diamond_Disputes.md D-5).
pub const RAM_DRAIN: u64 = 8;
/// The CPU's cycles between two DMA blocks or list nodes, and a node's own overhead [C, Diamond_Disputes.md D-5].
pub const DMA_GAP: u64 = 10;
pub const NODE_OVERHEAD: u64 = 4;
/// A chopped burst's time between windows beyond the CPU's window, which is held to 32 cycles a word of the DMA's
/// window (dma/chopping's console log, D-5) [C].
pub const CHOP_OVERHEAD: u64 = 6;

/// Where an address leads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Target {
    Ram(usize),
    Scratch(usize),
    Io(u32),
    Dev0,
    Dev1,
    Dev8(u32),
    Bios(usize),
    CacheControl,
    None,
}

/// Which delay register times an I/O address, or none for the internal registers.
fn io_device(offset: u32) -> Option<usize> {
    match offset {
        0x800..=0x803 => Some(memctl::DEV5),
        0xC00..=0xFFF => Some(memctl::DEV4),
        _ => None,
    }
}

/// Whether the processor can fetch from an I/O address (ps1-tests' `code-in-io` on a console): the SPU's and the DMA
/// registers answer, the MDEC's and the interrupt controller's raise a bus error, as do the rest [C].
fn io_fetchable(offset: u32) -> bool {
    matches!(offset, 0x080..=0x0FF | 0xC00..=0xFFF)
}

/// The decoded I/O ranges; the gaps between them raise a bus error (psx-spx "Memory Exceptions").
fn io_present(offset: u32) -> bool {
    matches!(offset, 0x000..=0x023 | 0x040..=0x04F | 0x050..=0x05F | 0x060..=0x063 | 0x070..=0x077 | 0x080..=0x0FF | 0x100..=0x12F | 0x800..=0x803 | 0x810..=0x817 | 0x820..=0x827 | 0xC00..=0xFFF)
}

#[derive(Clone)]
pub struct Board {
    /// The clock: CPU cycles since power-on.
    pub now: u64,
    pub next_event: u64,
    pub ram: Vec<u8>,
    pub scratch: Vec<u8>,
    pub bios: Vec<u8>,
    pub memctl: MemControl,
    pub cache: ICache,
    pub irq: Irq,
    pub timers: Timers,
    pub dma: Dma,
    pub video: Video,
    pub gpu: Gpu,
    pub spu: SpuStub,
    pub cdrom: CdromStub,
    pub mdec: MdecStub,
    /// The controller and memory card port, and the serial port's stub.
    pub sio0: Sio0,
    pub sio1: SioStub,
    pub duart: Duart,
    pub tty: Tty,
    pub gte: Gte,
    /// The cycle at which the GTE's command in flight is done; a read of its registers or a command waits for it.
    pub gte_ready: u64,
    /// The POST register at 1F802041h.
    pub post: u8,
    /// When each of the write queue's four entries is drained, oldest first.
    pub(crate) queue: [u64; 4],
    /// Whether the last access met no device.
    fault: bool,
    /// A field ended: the frame is done.
    pub field_done: bool,
    /// The cycle until which a DMA piece holds the bus, and the end of the CPU's latest access to it; the CPU waits
    /// for the DMA only when it needs the bus.
    pub bus_free: u64,
    pub cpu_busy: u64,
    /// Scratch words for a cache refill, so that a miss allocates nothing.
    refill: [u32; 4],
    /// A GPU DMA piece's own clock in sixteenths of a cycle, since the GPU may hold each word back.
    dma_clock: u64,
    /// Whether this field's vblank began, which ends a frame.
    pub(crate) vblank_seen: bool,
}

impl Board {
    pub fn new(pal_clock: bool, gpu_version: u32, regs: [u32; 9], ram_size: u32, bcc: u32) -> Board {
        let video = Video::new(pal_clock);
        let mut b = Board {
            now: 0,
            next_event: 0,
            ram: vec![0; RAM_BYTES],
            scratch: vec![0; SCRATCH_BYTES],
            bios: vec![0; BIOS_BYTES],
            memctl: MemControl::new(regs, ram_size),
            cache: ICache::new(bcc),
            irq: Irq::default(),
            timers: Timers::new(),
            dma: Dma::default(),
            gpu: Gpu::new(gpu_version, video.clock),
            video,
            spu: SpuStub::new(),
            cdrom: CdromStub::default(),
            mdec: MdecStub::default(),
            sio0: Sio0::new(Default::default()),
            sio1: SioStub::default(),
            duart: Duart::default(),
            tty: Tty::new(),
            gte: Gte::new(),
            gte_ready: 0,
            post: 0,
            queue: [0; 4],
            fault: false,
            field_done: false,
            bus_free: 0,
            cpu_busy: 0,
            refill: [0; 4],
            dma_clock: 0,
            vblank_seen: false,
        };
        b.schedule();
        b
    }

    /// Where `address` leads; KUSEG beyond 512 MiB, KSEG2 but the cache control register, the scratchpad in KSEG1
    /// and every unmapped range lead nowhere.
    fn decode(&self, address: u32) -> Target {
        if address >= 0xC000_0000 {
            return if address == 0xFFFE_0130 { Target::CacheControl } else { Target::None };
        }
        if (0x2000_0000..0x8000_0000).contains(&address) {
            return Target::None;
        }
        let kseg1 = address >= 0xA000_0000;
        let p = address & 0x1FFF_FFFF;
        let (ras0, ras1) = self.memctl.ram_banks();
        if p < ras0 + ras1 {
            return Target::Ram(p as usize & (RAM_BYTES - 1));
        }
        match p {
            0x1F80_0000..=0x1F80_03FF if !kseg1 => Target::Scratch(p as usize & 0x3FF),
            0x1F80_1000..=0x1F80_1FFF if io_present(p & 0xFFF) => Target::Io(p & 0xFFF),
            0x1F80_2000..=0x1F80_3FFF if p - 0x1F80_2000 < self.memctl.window(memctl::DEV8) => Target::Dev8(p & 0x1FFF),
            0x1FA0_0000..=0x1FBF_FFFF if p - 0x1FA0_0000 < self.memctl.window(memctl::DEV1) => Target::Dev1,
            0x1FC0_0000..=0x1FFF_FFFF if p - 0x1FC0_0000 < self.memctl.window(memctl::DEV2) => Target::Bios(p as usize & (BIOS_BYTES - 1)),
            _ if (self.memctl.regs[memctl::DEV0_BASE] & 0x1FFF_FFFF..(self.memctl.regs[memctl::DEV0_BASE] & 0x1FFF_FFFF) + self.memctl.window(memctl::DEV0)).contains(&p) && p < 0x1F80_0000 => Target::Dev0,
            _ => Target::None,
        }
    }

    fn timing(&self, device: usize) -> Timing {
        self.memctl.read_timing(device)
    }

    /// Waits for the write queue to drain, as a read from memory or I/O, or any uncached access, does.
    fn flush_queue(&mut self) {
        let last = self.queue.iter().copied().max().unwrap_or(0);
        if last > self.now {
            self.now = last;
        }
    }

    /// A store through the write queue: it waits only while all four entries are busy.
    fn queue_store(&mut self, drain: u64) {
        let oldest = (0..4).min_by_key(|&i| self.queue[i]).unwrap_or(0);
        if self.queue[oldest] > self.now {
            self.now = self.queue[oldest];
        }
        let start = self.queue.iter().copied().max().unwrap_or(0).max(self.now).max(self.bus_free);
        self.queue[oldest] = start + drain;
    }

    // --- the memories ---

    fn ram_read(&self, at: usize, size: Size) -> u32 {
        let mut v = 0;
        for i in 0..size.bytes() as usize {
            v |= (self.ram[(at + i) & (RAM_BYTES - 1)] as u32) << (8 * i);
        }
        v
    }

    pub fn ram_word(&self, address: u32) -> u32 {
        let a = address as usize & (RAM_BYTES - 4);
        u32::from_le_bytes([self.ram[a], self.ram[a + 1], self.ram[a + 2], self.ram[a + 3]])
    }

    pub fn set_ram_word(&mut self, address: u32, value: u32) {
        let a = address as usize & (RAM_BYTES - 4);
        self.ram[a..a + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn bytes_read(mem: &[u8], at: usize, size: Size) -> u32 {
        let mask = mem.len() - 1;
        (0..size.bytes() as usize).fold(0, |v, i| v | (mem[(at + i) & mask] as u32) << (8 * i))
    }

    fn bytes_write(mem: &mut [u8], at: usize, size: Size, value: u32) {
        let mask = mem.len() - 1;
        for i in 0..size.bytes() as usize {
            mem[(at + i) & mask] = (value >> (8 * i)) as u8;
        }
    }

    // --- I/O ---

    /// The internal registers' 32-bit view at `offset & !3`, brought to the present first.
    fn io_read32(&mut self, offset: u32) -> u32 {
        let o = offset & !3;
        match o {
            0x000..=0x020 => self.memctl.read((o / 4) as usize),
            0x050 => self.sio1.receive() as u32,
            0x054 => self.sio1.status(),
            0x058 => self.sio1.mode as u32 | (self.sio1.control as u32) << 16,
            0x05C => (self.sio1.baud as u32) << 16,
            0x060 => self.memctl.ram_size,
            0x070 => self.irq.stat,
            0x074 => self.irq.mask,
            0x080..=0x0FF => self.dma.read(o - 0x80),
            0x100..=0x12F => {
                let n = ((o - 0x100) >> 4) as usize;
                if self.timers.sync(n, &self.video, self.now) {
                    self.irq.raise(irq::TIMER0 << n);
                }
                match o & 0xF {
                    0 => self.timers.t[n].count,
                    4 => self.timers.read_mode(n),
                    8 => self.timers.t[n].target,
                    _ => 0,
                }
            }
            0x810 => {
                let (v, t) = self.gpu.gpuread(&self.video, self.now);
                self.now = t;
                self.next_event = self.next_event.min(self.gpu.next_event());
                self.wake_gpu_dma();
                v
            }
            0x814 => self.gpu.status(&self.video, self.now),
            0x820 => 0,
            0x824 => self.mdec.status,
            _ => 0,
        }
    }

    fn io_write32(&mut self, offset: u32, value: u32) {
        let o = offset & !3;
        match o {
            0x000..=0x020 => self.memctl.write((o / 4) as usize, value),
            0x050 => self.sio1.send(value as u8),
            0x058 => self.sio1.mode = value as u16 & 0xFF,
            0x05C => self.sio1.baud = (value >> 16) as u16,
            0x060 => self.memctl.ram_size = value,
            0x070 => self.irq.acknowledge(value),
            0x074 => self.irq.set_mask(value),
            0x080..=0x0FF => {
                let reg = if o >= 0xF8 { return } else { o - 0x80 };
                if self.dma.write(reg, value) {
                    self.irq.raise(irq::DMA);
                }
                self.dma_kick();
            }
            0x100..=0x12F => {
                let n = ((o - 0x100) >> 4) as usize;
                if self.timers.sync(n, &self.video, self.now) {
                    self.irq.raise(irq::TIMER0 << n);
                }
                match o & 0xF {
                    0 => self.timers.write_count(n, value),
                    4 => {
                        let blank = match n {
                            0 => self.video.in_hblank_at(self.now),
                            _ => self.video.in_vblank_at(self.now),
                        };
                        let watched = self.wants_hblank();
                        self.timers.write_mode(n, &self.video, self.now, value, blank);
                        if !watched && self.wants_hblank() {
                            self.video.skip_hblanks(self.now);
                        }
                    }
                    8 => self.timers.write_target(n, value),
                    _ => {}
                }
            }
            0x810 => {
                self.now = self.gpu.gp0(value, self.now, &self.video);
                self.take_gpu_irq();
                self.wake_gpu_dma();
            }
            0x814 => {
                self.sync_timers();
                self.gpu.gp1(value, &mut self.video, self.now);
                self.timers.rebase(&self.video, self.now);
                self.dma_kick();
                self.wake_gpu_dma();
            }
            0x820 => {}
            0x824 => {
                self.mdec.control(value);
                self.dma_kick();
            }
            _ => {}
        }
        self.schedule();
    }

    /// A halfword register's write at its own address (the serial ports' halfwords, the timers', the SPU's).
    fn io_write16(&mut self, offset: u32, value: u32) {
        match offset & !1 {
            0x05A => self.sio1.set_control(value as u16),
            0x05E => self.sio1.baud = value as u16,
            o @ 0xC00..=0xFFF => {
                self.spu.write16(o - 0xC00, value as u16);
                self.dma_kick();
            }
            o => self.io_write32(o & !3, value),
        }
    }

    fn io_read(&mut self, offset: u32, size: Size) -> u32 {
        match offset {
            0x040..=0x04F => {
                self.sync_sio0();
                self.sio0.read(offset, size.bytes(), self.now)
            }
            0x800..=0x803 => {
                let b = self.cdrom.read8(offset) as u32;
                (0..size.bytes()).fold(0, |v, i| v | b << (8 * i))
            }
            0xC00..=0xFFF => {
                let at = (offset - 0xC00) & !1;
                let lo = self.spu.read16(at) as u32;
                let v = if size.bytes() > 2 { lo | (self.spu.read16(at + 2) as u32) << 16 } else { lo };
                v >> (8 * (offset & 1))
            }
            0x05A => self.sio1.control as u32,
            0x05E => self.sio1.baud as u32,
            _ => self.io_read32(offset) >> (8 * (offset & 3)),
        }
    }

    fn io_write(&mut self, offset: u32, size: Size, value: u32) {
        match offset {
            0x040..=0x04F => {
                self.sync_sio0();
                if self.sio0.write(offset, size.bytes(), value, self.now) {
                    self.irq.raise(irq::SIO0);
                }
                self.schedule();
            }
            0x800..=0x803 => {
                for i in 0..size.bytes() {
                    self.cdrom.write8(offset, (value >> (8 * i)) as u8);
                }
            }
            0xC00..=0xFFF => {
                self.io_write16(offset & !1, value & 0xFFFF);
                if size.bytes() > 2 {
                    self.io_write16((offset & !1) + 2, value >> 16);
                }
            }
            _ if size == Size::Half || matches!(offset & 0xFFE, 0x05A | 0x05E) => self.io_write16(offset, value),
            _ => self.io_write32(offset, value),
        }
    }

    // --- the clock's events ---

    /// SIO0 brought to the present, its interrupt raised if it rose.
    fn sync_sio0(&mut self) {
        if self.sio0.run(self.now) {
            self.irq.raise(irq::SIO0);
        }
    }

    /// Every timer brought to the present.
    fn sync_timers(&mut self) {
        for n in 0..3 {
            if self.timers.sync(n, &self.video, self.now) {
                self.irq.raise(irq::TIMER0 << n);
            }
        }
    }

    fn wants_hblank(&self) -> bool {
        self.timers.t[0].mode & timers::mode::SYNC != 0
    }

    /// The next cycle at which anything is due: a video edge, a timer's target or FFFFh, a DMA piece.
    pub fn schedule(&mut self) {
        let mut next = self.video.next_edge(self.wants_hblank()).min(self.dma.due).min(self.gpu.next_event()).min(self.sio0.next_event());
        for n in 0..3 {
            next = next.min(self.timers.next_hit(n, &self.video));
        }
        self.next_event = next;
    }

    /// Runs whatever is due at the present.
    pub fn events(&mut self) {
        while self.now >= self.next_event {
            let hb = self.wants_hblank();
            let edge_at = self.video.next_edge(hb);
            let mut due = [u64::MAX; 3];
            for (n, d) in due.iter_mut().enumerate() {
                *d = self.timers.next_hit(n, &self.video);
            }
            let gpu_at = self.gpu.next_event();
            let sio_at = self.sio0.next_event();
            let first = edge_at.min(self.dma.due).min(due[0]).min(due[1]).min(due[2]).min(gpu_at).min(sio_at);
            if first > self.now {
                self.next_event = first;
                return;
            }
            for (n, &d) in due.iter().enumerate() {
                if d == first && self.timers.sync(n, &self.video, d) {
                    self.irq.raise(irq::TIMER0 << n);
                }
            }
            if sio_at == first && self.sio0.run(first) {
                self.irq.raise(irq::SIO0);
            }
            if gpu_at == first {
                self.gpu.run(first, &self.video);
                self.take_gpu_irq();
                self.wake_gpu_dma();
            }
            if edge_at == first {
                self.gpu.sync(edge_at, &self.video);
                self.take_gpu_irq();
                for n in 0..2 {
                    if self.timers.sync(n, &self.video, edge_at) {
                        self.irq.raise(irq::TIMER0 << n);
                    }
                }
                let what = self.video.edge(edge_at, hb);
                if what & edge::FIELD != 0 && !std::mem::take(&mut self.vblank_seen) {
                    self.field_done = true;
                }
                if what & edge::VBLANK_START != 0 {
                    self.field_done = true;
                    self.vblank_seen = what & edge::FIELD == 0;
                    self.irq.raise(irq::VBLANK);
                    self.timers.blank(1, true);
                }
                if what & edge::VBLANK_END != 0 {
                    self.timers.blank(1, false);
                }
                if what & edge::HBLANK_START != 0 {
                    self.timers.blank(0, true);
                }
                if what & edge::HBLANK_END != 0 {
                    self.timers.blank(0, false);
                }
            }
            if self.dma.due == first {
                self.dma.due = u64::MAX;
                self.dma_event(first);
            }
            self.schedule();
        }
    }

    // --- DMA ---

    fn dma_request(&self, n: usize) -> bool {
        match n {
            dma::MDEC_IN => self.mdec.status & 1 << 28 != 0,
            dma::MDEC_OUT => self.mdec.status & 1 << 27 != 0,
            dma::GPU => self.gpu.dma_direction != 0,
            dma::CDROM => true,
            dma::SPU => self.spu.dreq(),
            dma::PIO => true,
            _ => false,
        }
    }

    /// Starts the first-ranked ready channel if none is moving.
    fn dma_kick(&mut self) {
        if self.dma.active.is_some() {
            return;
        }
        let mut ready = [false; 7];
        for (n, r) in ready.iter_mut().enumerate() {
            *r = self.dma.ready(n, self.dma_request(n));
        }
        if let Some(n) = self.dma.choose(ready) {
            self.dma.ch[n].chcr &= !chcr::TRIGGER;
            self.dma.active = Some(n);
            self.dma.left = 0;
            self.dma_piece(self.now);
            self.schedule();
        }
    }

    fn cost(&self, n: usize, words: u64) -> u64 {
        (words * dma::sixteenths_per_word(n, self.memctl.regs[memctl::DEV5])).div_ceil(16)
    }

    /// A piece's time: its words at the channel's rate, or longer when the GPU held them back.
    fn piece_cost(&self, n: usize, words: u64, start: u64) -> u64 {
        let cost = self.cost(n, words);
        if n == dma::GPU { cost.max(self.dma_clock.div_ceil(16).saturating_sub(start)) } else { cost }
    }

    /// GP0(1Fh)'s interrupt, once the GPU reached it.
    fn take_gpu_irq(&mut self) {
        if self.gpu.take_irq() {
            self.irq.raise(irq::GPU);
        }
    }

    /// A GPU transfer waiting for the GPU's request looks again once the GPU's state changed.
    fn wake_gpu_dma(&mut self) {
        if self.dma.active == Some(dma::GPU) && self.dma.due == u64::MAX {
            self.dma.due = self.now;
            self.schedule();
        }
    }

    /// A word to the device of channel `n`.
    fn send_to_device(&mut self, n: usize, word: u32) {
        match n {
            dma::GPU => {
                let at = self.dma_clock.div_ceil(16);
                let taken = self.gpu.gp0(word, at, &self.video);
                self.dma_clock = if taken > at { taken * 16 } else { self.dma_clock } + dma::sixteenths_per_word(dma::GPU, 0);
                self.take_gpu_irq();
            }
            dma::SPU => {
                self.spu.store(word as u16);
                self.spu.store((word >> 16) as u16);
            }
            _ => {}
        }
    }

    fn receive_from_device(&mut self, n: usize) -> u32 {
        match n {
            dma::GPU => {
                let at = self.dma_clock.div_ceil(16);
                let (w, t) = self.gpu.gpuread(&self.video, at);
                self.dma_clock = if t > at { t * 16 } else { self.dma_clock } + dma::sixteenths_per_word(dma::GPU, 0);
                self.next_event = self.next_event.min(self.gpu.next_event());
                w
            }
            dma::SPU => self.spu.load() as u32 | (self.spu.load() as u32) << 16,
            _ => 0,
        }
    }

    /// The CPU is about to use the bus: the DMA pieces due before now run first, and the CPU waits while one holds
    /// the bus (psx-spx "CPU Operation during DMA": the cache, the scratchpad and the coprocessors need no bus).
    fn take_bus(&mut self) {
        loop {
            while self.dma.due <= self.now {
                let at = self.dma.due;
                self.dma.due = u64::MAX;
                self.dma_event(at);
            }
            if self.bus_free > self.now {
                self.now = self.bus_free;
                continue;
            }
            break;
        }
    }

    /// The DMA's event at `at`: a transfer's end, or its next piece.
    fn dma_event(&mut self, at: u64) {
        let Some(n) = self.dma.active else { return };
        if !self.dma.finishing {
            self.dma_piece(at);
            return;
        }
        self.dma.finishing = false;
        self.dma.ch[n].chcr &= !(chcr::START | chcr::TRIGGER);
        self.dma.active = None;
        self.dma.left = 0;
        if self.dma.flag(n) {
            self.irq.raise(irq::DMA);
        }
        self.dma_kick();
    }

    /// The active channel's next piece from cycle `at`: a whole burst, a chopped window, a block, or a list node,
    /// holding the bus for its time; its end, or the next piece, is the DMA's next event.
    fn dma_piece(&mut self, at: u64) {
        let Some(n) = self.dma.active else { return };
        let c = self.dma.ch[n];
        if c.chcr & chcr::START == 0 || !self.dma.enabled(n) {
            self.dma.active = None;
            return;
        }
        if n == dma::GPU && c.sync() != 0 {
            self.gpu.run(at, &self.video);
            if !self.gpu.dreq(at) {
                self.dma.due = self.gpu.dreq_change();
                return;
            }
        }
        let start = at.max(self.bus_free);
        self.dma_clock = start * 16;
        let step: u32 = if c.chcr & chcr::BACKWARD != 0 { 0u32.wrapping_sub(4) } else { 4 };
        let from_ram = c.chcr & chcr::FROM_RAM != 0 && n != dma::OTC;
        let finished;
        let words: u64;
        let mut overhead = 0;
        match c.sync() {
            2 if n != dma::OTC => {
                let header = self.ram_word(c.madr);
                let count = header >> 24;
                self.dma_clock += dma::sixteenths_per_word(n, 0);
                let mut a = c.madr;
                for _ in 0..count {
                    a = a.wrapping_add(4) & 0x00FF_FFFC;
                    let w = self.ram_word(a);
                    self.send_to_device(n, w);
                }
                words = 1 + count as u64;
                overhead = NODE_OVERHEAD;
                self.dma.ch[n].madr = header & 0x00FF_FFFF;
                finished = header & 0x0080_0000 != 0;
            }
            1 if n != dma::OTC => {
                let size = match c.bcr & 0xFFFF {
                    0 => 0x1_0000,
                    s => s,
                };
                let mut a = c.madr;
                for _ in 0..size {
                    self.transfer_word(n, a, from_ram);
                    a = a.wrapping_add(step) & 0x00FF_FFFF;
                }
                words = size as u64;
                self.dma.ch[n].madr = a;
                let blocks = (c.bcr >> 16).wrapping_sub(1) & 0xFFFF;
                self.dma.ch[n].bcr = c.bcr & 0xFFFF | blocks << 16;
                finished = blocks == 0;
            }
            _ => {
                let total = match c.bcr & 0xFFFF {
                    0 => 0x1_0000,
                    s => s,
                };
                let chopped = c.chcr & chcr::CHOP != 0 && n != dma::OTC;
                if self.dma.left == 0 {
                    self.dma.left = total;
                }
                let piece = if chopped { (1u32 << (c.chcr >> 16 & 7)).min(self.dma.left) } else { self.dma.left };
                let mut a = if chopped { c.madr } else { c.madr.wrapping_add(step.wrapping_mul(total - self.dma.left)) & 0x00FF_FFFF };
                for i in 0..piece {
                    if n == dma::OTC {
                        let last = self.dma.left - i == 1;
                        self.set_ram_word(a, if last { 0x00FF_FFFF } else { a.wrapping_sub(4) & 0x001F_FFFC });
                    } else {
                        self.transfer_word(n, a, from_ram);
                    }
                    a = a.wrapping_add(step) & 0x00FF_FFFF;
                }
                self.dma.left -= piece;
                words = piece as u64;
                if chopped {
                    self.dma.ch[n].madr = a;
                    self.dma.ch[n].bcr = c.bcr & !0xFFFF | self.dma.left & 0xFFFF;
                }
                finished = self.dma.left == 0;
                if !finished {
                    let end = self.hold_bus(at, self.piece_cost(n, words, start));
                    let cpu_window = (1u64 << (c.chcr >> 20 & 7)).min(32 << (c.chcr >> 16 & 7));
                    self.dma.due = end + cpu_window + CHOP_OVERHEAD;
                    return;
                }
            }
        }
        let end = self.hold_bus(at, self.piece_cost(n, words, start) + overhead);
        if finished {
            self.dma.finishing = true;
            self.dma.due = end;
        } else {
            if self.dma.dicr & 1 << n != 0 && self.dma.flag(n) {
                self.irq.raise(irq::DMA);
            }
            self.dma.due = end + DMA_GAP;
        }
    }

    /// The DMA holds the bus for `cycles` from `at` or once it is free; the stores queued for memory wait with it,
    /// and so does the CPU when it has used the bus since the piece began, the piece being run late (Diamond_Native.md
    /// §11.7). Returns the cycle it lets go.
    fn hold_bus(&mut self, at: u64, cycles: u64) -> u64 {
        let start = at.max(self.bus_free);
        self.bus_free = start + cycles;
        if self.cpu_busy > start {
            self.now += cycles;
            self.cpu_busy += cycles;
        }
        for q in &mut self.queue {
            if *q > start {
                *q += cycles;
            }
        }
        self.bus_free
    }

    fn transfer_word(&mut self, n: usize, address: u32, from_ram: bool) {
        if from_ram {
            let w = self.ram_word(address);
            self.send_to_device(n, w);
        } else {
            let w = self.receive_from_device(n);
            self.set_ram_word(address, w);
        }
    }

    /// The fetch's cost and word from memory, uncached, at `target`.
    fn fetch_uncached(&mut self, target: Target) -> u32 {
        match target {
            Target::Ram(a) => {
                self.now += 1 + RAM_STALL;
                self.ram_read(a, Size::Word)
            }
            Target::Bios(a) => {
                self.now += self.timing(memctl::DEV2).cycles(4) as u64;
                Self::bytes_read(&self.bios, a, Size::Word)
            }
            Target::Io(o) if io_fetchable(o) => {
                self.now += 1 + IO_STALL;
                self.io_read(o, Size::Word)
            }
            Target::Dev0 | Target::Dev1 | Target::Dev8(_) => {
                let d = match target {
                    Target::Dev0 => memctl::DEV0,
                    Target::Dev1 => memctl::DEV1,
                    _ => memctl::DEV8,
                };
                self.now += self.timing(d).cycles(4) as u64;
                u32::MAX
            }
            _ => {
                self.now += 1;
                self.fault = true;
                0
            }
        }
    }

    fn read_at(&mut self, target: Target, size: Size) -> u32 {
        match target {
            Target::Ram(a) => {
                self.now += RAM_STALL;
                self.ram_read(a, size)
            }
            Target::Scratch(a) => Self::bytes_read(&self.scratch, a, size),
            Target::Bios(a) => {
                self.now += self.timing(memctl::DEV2).cycles(size.bytes()) as u64 - 1;
                Self::bytes_read(&self.bios, a, size)
            }
            Target::Io(o) => {
                self.now += match io_device(o) {
                    Some(d) => self.timing(d).cycles(size.bytes()) as u64 - 1,
                    None => IO_STALL,
                };
                self.io_read(o, size)
            }
            Target::Dev8(o) => {
                self.now += self.timing(memctl::DEV8).cycles(size.bytes()) as u64 - 1;
                match o {
                    0x20..=0x2F => {
                        let b = self.duart.read8(o - 0x20) as u32;
                        (0..size.bytes()).fold(0, |v, i| v | b << (8 * i))
                    }
                    0x41 => self.post as u32,
                    _ => u32::MAX,
                }
            }
            Target::Dev0 | Target::Dev1 => {
                let d = if target == Target::Dev0 { memctl::DEV0 } else { memctl::DEV1 };
                self.now += self.timing(d).cycles(size.bytes()) as u64 - 1;
                u32::MAX
            }
            Target::CacheControl => self.cache.bcc,
            Target::None => {
                self.fault = true;
                0
            }
        }
    }

    fn write_at(&mut self, address: u32, size: Size, value: u32, queued: bool) {
        let target = self.decode(address);
        match target {
            Target::Ram(a) => {
                if queued {
                    self.queue_store(RAM_DRAIN);
                } else {
                    self.now += RAM_STALL;
                }
                Self::bytes_write(&mut self.ram, a, size, value);
            }
            Target::Scratch(a) => Self::bytes_write(&mut self.scratch, a, size, value),
            Target::Bios(_) => {}
            Target::Io(o) => {
                let cost = match io_device(o) {
                    Some(d) => self.memctl.write_timing(d).cycles(size.bytes()) as u64 - 1,
                    None => IO_STALL,
                };
                if queued { self.queue_store(cost) } else { self.now += cost }
                self.io_write(o, size, value);
            }
            Target::Dev8(o) => {
                let cost = self.memctl.write_timing(memctl::DEV8).cycles(size.bytes()) as u64 - 1;
                if queued { self.queue_store(cost) } else { self.now += cost }
                match o {
                    0x20..=0x2F => {
                        if let Some(b) = self.duart.write8(o - 0x20, value as u8) {
                            self.tty.put(b);
                        }
                    }
                    0x41 => self.post = value as u8,
                    _ => {}
                }
            }
            Target::Dev0 | Target::Dev1 => {
                let d = if target == Target::Dev0 { memctl::DEV0 } else { memctl::DEV1 };
                let cost = self.memctl.write_timing(d).cycles(size.bytes()) as u64 - 1;
                if queued { self.queue_store(cost) } else { self.now += cost }
            }
            Target::CacheControl => self.cache.bcc = value & crate::icache::bcc::WRITABLE,
            Target::None => self.fault = true,
        }
    }
}

impl Board {
    /// The interlock: every access to coprocessor 2 holds the processor until the command in flight is done, reads and
    /// commands by psx-spx's "GTE Load Delay Slots", writes by choice (BlackMoon_GTE.md D-13).
    fn wait_for_gte(&mut self) {
        self.now = self.now.max(self.gte_ready);
    }
}

impl Bus for Board {
    fn fetch(&mut self, address: u32) -> u32 {
        let target = self.decode(address);
        let cached = address < 0xA000_0000 && self.cache.enabled() && matches!(target, Target::Ram(_) | Target::Bios(_));
        if !cached {
            self.take_bus();
            if address >= 0xA000_0000 {
                self.flush_queue();
            }
            let w = self.fetch_uncached(target);
            self.cpu_busy = self.now;
            return w;
        }
        let physical = address & 0x1FFF_FFFF;
        match self.cache.lookup(physical) {
            Lookup::Hit(w) => {
                self.now += 1;
                w
            }
            Lookup::Miss { first, count } => {
                self.take_bus();
                let line = physical & !0xF;
                for i in 0..count {
                    let a = line + 4 * (first + i);
                    self.refill[i as usize] = match self.decode(a | 0x8000_0000) {
                        Target::Ram(r) => self.ram_read(r, Size::Word),
                        Target::Bios(b) => Self::bytes_read(&self.bios, b, Size::Word),
                        _ => 0,
                    };
                }
                let per_word = match target {
                    Target::Ram(_) => 1,
                    _ => self.timing(memctl::DEV2).cycles(4) as u64,
                };
                let first_cost = match target {
                    Target::Ram(_) => 1 + RAM_STALL,
                    _ => per_word,
                };
                self.now += first_cost + per_word * (count as u64 - 1);
                self.cpu_busy = self.now;
                let words = self.refill;
                self.cache.fill(physical, first, &words[..count as usize]);
                self.refill[((physical >> 2 & 3) - first) as usize]
            }
        }
    }

    fn read(&mut self, address: u32, size: Size) -> u32 {
        let target = self.decode(address);
        if matches!(target, Target::Scratch(_) | Target::CacheControl) {
            return self.read_at(target, size);
        }
        self.take_bus();
        self.flush_queue();
        let v = self.read_at(target, size);
        self.cpu_busy = self.now;
        v
    }

    fn write(&mut self, address: u32, size: Size, value: u32) {
        let queued = address < 0xA000_0000;
        if !queued {
            self.take_bus();
            self.flush_queue();
        }
        self.write_at(address, size, value, queued);
        if !queued {
            self.cpu_busy = self.now;
        }
    }

    fn isolated_read(&mut self, address: u32, _size: Size) -> u32 {
        self.cache.isolated_read(address)
    }

    fn isolated_write(&mut self, address: u32, _size: Size, value: u32) {
        self.cache.isolated_write(address, value);
    }

    fn idle(&mut self, cycles: u32) {
        self.now += cycles as u64;
    }

    fn cycles(&mut self) -> u64 {
        self.now
    }

    fn interrupt(&mut self) -> bool {
        self.irq.line()
    }

    fn cop2_data(&mut self, register: u8) -> u32 {
        self.wait_for_gte();
        self.gte.data(register)
    }

    fn set_cop2_data(&mut self, register: u8, value: u32) {
        self.wait_for_gte();
        self.gte.set_data(register, value);
    }

    fn cop2_control(&mut self, register: u8) -> u32 {
        self.wait_for_gte();
        self.gte.control(register)
    }

    fn set_cop2_control(&mut self, register: u8, value: u32) {
        self.wait_for_gte();
        self.gte.set_control(register, value);
    }

    fn cop2_command(&mut self, command: u32) {
        self.wait_for_gte();
        self.gte_ready = self.now + self.gte.command(command) as u64;
    }

    fn bus_error(&mut self) -> bool {
        std::mem::take(&mut self.fault)
    }
}

/// The kernel's memory control values, or the power-on ones the kernel starts from: every delay at its maximum (psx-spx
/// "Memory Control"), the sizes and bases as the kernel leaves them [C].
pub fn power_on_memctl() -> [u32; 9] {
    let mut r = memctl::KERNEL_VALUES;
    for v in &mut r[memctl::DEV0..=memctl::DEV8] {
        *v |= 0xFF;
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::icache::KERNEL_BCC;

    fn board() -> Board {
        Board::new(false, 2, memctl::KERNEL_VALUES, memctl::KERNEL_RAM_SIZE, KERNEL_BCC)
    }

    #[test]
    fn the_map_mirrors_ram_and_faults_where_nothing_answers() {
        let mut b = board();
        b.write(0x8000_1234, Size::Word, 0xCAFE_F00D);
        assert_eq!(b.read(0x0020_1234, Size::Word), 0xCAFE_F00D, "2 MiB mirrored in the first 8 MiB");
        assert_eq!(b.read(0xA060_1234, Size::Word), 0xCAFE_F00D);
        assert!(!b.bus_error());
        b.read(0x0080_0000, Size::Word);
        assert!(b.bus_error(), "past the 8 MiB bank");
        b.write(0x1F80_0010, Size::Half, 0xBEEF);
        assert_eq!(b.read(0x9F80_0010, Size::Half), 0xBEEF);
        b.read(0xBF80_0010, Size::Half);
        assert!(b.bus_error(), "no scratchpad in KSEG1");
        b.read(0x1F80_1024, Size::Word);
        assert!(b.bus_error(), "a gap in the I/O ports");
        b.fetch(0x1F80_0000);
        assert!(b.bus_error(), "the scratchpad is not executable");
        b.fetch(0x1F80_1820);
        assert!(b.bus_error(), "nor is the MDEC");
        b.fetch(0x1F80_10F0);
        assert!(!b.bus_error(), "the DMA registers are");
        b.read(0x4000_0000, Size::Word);
        assert!(b.bus_error());
        assert_eq!(b.read(0xFFFE_0130, Size::Word), KERNEL_BCC);
    }

    #[test]
    fn a_load_costs_its_regions_time() {
        let mut b = board();
        let cost = |b: &mut Board, a: u32, s: Size| {
            let t = b.now;
            b.read(a, s);
            b.now - t + 1
        };
        assert_eq!(cost(&mut b, 0x8000_0000, Size::Byte), 5);
        assert_eq!(cost(&mut b, 0x1F80_0000, Size::Word), 1);
        assert_eq!(cost(&mut b, 0xBFC0_0000, Size::Word), 25);
        assert_eq!(cost(&mut b, 0x1F80_1070, Size::Word), 3);
        assert_eq!(cost(&mut b, 0x1F80_1DAA, Size::Word), 39);
        assert_eq!(cost(&mut b, 0x1F80_1800, Size::Half), 14);
        assert_eq!(cost(&mut b, 0xFFFE_0130, Size::Word), 1);
    }

    #[test]
    fn otc_builds_the_ordering_table_backwards() {
        let mut b = board();
        b.write(0x1F80_10F0, Size::Word, 0x0F65_4321);
        b.write(0x1F80_10E0, Size::Word, 0x0001_000C);
        b.write(0x1F80_10E4, Size::Word, 4);
        let t = b.now;
        b.write(0x1F80_10E8, Size::Word, 0x1100_0002);
        assert_eq!([b.ram_word(0x1000C), b.ram_word(0x10008), b.ram_word(0x10004), b.ram_word(0x10000)], [0x10008, 0x10004, 0x10000, 0x00FF_FFFF]);
        assert_eq!(b.read(0x1F80_10E8, Size::Word), 2, "done before the CPU reads it");
        assert!(b.now - t >= 4);
    }

    #[test]
    fn a_gpu_list_runs_node_by_node_and_flags_its_end() {
        let mut b = board();
        b.write(0x1F80_10F0, Size::Word, 0x0765_4B21);
        b.write(0x1F80_10F4, Size::Word, 1 << 23 | 1 << 18);
        b.set_ram_word(0x100, 0x0100_0200);
        b.set_ram_word(0x104, 0xE100_0123);
        b.set_ram_word(0x200, 0x00FF_FFFF);
        b.write(0x1F80_1814, Size::Word, 0x0400_0002);
        b.write(0x1F80_10A0, Size::Word, 0x100);
        b.write(0x1F80_10A8, Size::Word, 0x0100_0401);
        assert_ne!(b.dma.ch[2].chcr & chcr::START, 0, "the second node waits for its turn");
        while b.dma.ch[2].chcr & chcr::START != 0 {
            b.now = b.next_event;
            b.events();
        }
        assert_eq!(b.gpu.env[0] & 0x7FF, 0x123);
        assert_ne!(b.irq.stat & irq::DMA, 0);
        assert_eq!(b.dma.ch[2].madr, 0x00FF_FFFF);
    }

    #[test]
    fn vblank_interrupts_once_a_field() {
        let mut b = board();
        let mut fields = 0;
        let mut vblanks = 0;
        while fields < 3 {
            b.now = b.next_event;
            b.events();
            if std::mem::take(&mut b.field_done) {
                fields += 1;
            }
            if b.irq.stat & irq::VBLANK != 0 {
                vblanks += 1;
                b.irq.acknowledge(!irq::VBLANK);
            }
        }
        assert_eq!(vblanks, 3);
    }
}
