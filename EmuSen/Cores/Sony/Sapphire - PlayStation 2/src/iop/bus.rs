//! The IOP's bus, which `blackmoon-r3000` meets past the processor (BlackMoon_R3000.md §2): the IOP's memory map
//! (ps2tek "Memory Map"; psx-spx "Memory Map" where ps2tek says "same as on PSX"), its I-cache and scratchpad, the
//! wait states, and the units on it, the INTC, the DMAC and the six timers, brought to the processor's time at their
//! events. The clock is the bus's, in IOP cycles; the units' events are kept in EE cycles (`clock`).
//! Sapphire_Native.md §22.2 and §22.3.

use blackmoon_r3000::{Bus, Size};

use super::dma::{self, Dmac, StandIns};
use super::icache::{self, ICache, Lookup};
use super::intc::{self, Intc, source};
use super::timer::{self, Timer};
use crate::blank::Blank;
use crate::clock::IOP;

pub const RAM: usize = 2 << 20;
/// RAM's window: 2 MiB mirrored through the first 8 MiB, as the PlayStation's default [A: psx-spx "Memory Mirrors",
/// ps2tek "same as on PSX"; D-61].
pub const RAM_WINDOW: u32 = 8 << 20;
pub const SCRATCH: usize = 1 << 10;
pub const SCRATCH_BASE: u32 = 0x1F80_0000;
pub const ROM_BASE: u32 = 0x1FC0_0000;
pub const ROM_SIZE: u32 = 4 << 20;
pub const SIF_BASE: u32 = 0x1D00_0000;
pub const CACHE_CONTROL: u32 = 0xFFFE_0130;

/// The access times in IOP cycles beyond the instruction's own [A: the PlayStation's, measured on its console,
/// Diamond_Native.md §11.3, since no document gives the IOP's; D-63]: a load from RAM, from an I/O register, from the
/// boot ROM [C: a placeholder until S24 runs firmware]; the scratchpad and a stored word cost nothing beyond it.
pub const RAM_STALL: u64 = 4;
pub const IO_STALL: u64 = 2;
pub const ROM_STALL: u64 = 6;

/// What an address reaches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Ram(u32),
    Scratch(u32),
    /// The I/O registers at 1F400000h-1F4FFFFFh (CDVD), 1F801000h-1F80FFFFh and the SPU2's 1F900000h-1F9007FFh.
    Io(u32),
    /// The SIF's registers at 1D000000h, S22's.
    Sif(u32),
    Rom(u32),
    CacheControl,
    /// KSEG2's other control registers at FFFE0000h-FFFE01FFh, which read zero [C].
    Control,
    None,
}

/// The target of a virtual address: KUSEG's first 512 MiB, KSEG0 and KSEG1 decode to one physical space, the scratchpad
/// in KUSEG and KSEG0 only, KSEG2 holding the cache control; anything else meets no device (psx-spx).
pub fn decode(address: u32) -> Target {
    if address >= 0xC000_0000 {
        return match address {
            CACHE_CONTROL => Target::CacheControl,
            0xFFFE_0000..0xFFFE_0200 => Target::Control,
            _ => Target::None,
        };
    }
    if (0x2000_0000..0x8000_0000).contains(&address) {
        return Target::None;
    }
    let p = address & 0x1FFF_FFFF;
    match p {
        0..RAM_WINDOW => Target::Ram(p & (RAM as u32 - 1)),
        0x1D00_0000..0x1D00_0100 => Target::Sif(p),
        0x1F40_0000..0x1F50_0000 | 0x1F80_1000..0x1F81_0000 | 0x1F90_0000..0x1F90_0800 => Target::Io(p),
        0x1F80_0000..0x1F80_0400 if address < 0xA000_0000 => Target::Scratch(p - SCRATCH_BASE),
        ROM_BASE..0x2000_0000 => Target::Rom(p - ROM_BASE),
        _ => Target::None,
    }
}

fn get(bytes: &[u8], at: usize, size: Size) -> u32 {
    let n = size.bytes() as usize;
    let mut v = [0u8; 4];
    if at + n <= bytes.len() {
        v[..n].copy_from_slice(&bytes[at..at + n]);
    }
    u32::from_le_bytes(v)
}

fn put(bytes: &mut [u8], at: usize, size: Size, value: u32) {
    let n = size.bytes() as usize;
    if at + n <= bytes.len() {
        bytes[at..at + n].copy_from_slice(&value.to_le_bytes()[..n]);
    }
}

/// IOP RAM as the DMAC reaches it.
struct RamPort<'a>(&'a mut [u8]);

impl dma::Memory for RamPort<'_> {
    fn load(&mut self, addr: u32) -> Option<u32> {
        (addr < RAM_WINDOW).then(|| get(self.0, (addr as usize) & (RAM - 1), Size::Word))
    }

    fn store(&mut self, addr: u32, w: u32) -> bool {
        let ok = addr < RAM_WINDOW;
        if ok {
            put(self.0, (addr as usize) & (RAM - 1), Size::Word, w);
        }
        ok
    }
}

pub struct IopBus {
    pub ram: Vec<u8>,
    pub scratch: [u8; SCRATCH],
    pub icache: ICache,
    /// The boot ROM, empty until firmware is given (S24); it reads as zero.
    pub rom: Vec<u8>,
    pub intc: Intc,
    pub dmac: Dmac,
    pub timers: [Timer; 6],
    /// The peripherals' stand-ins behind the DMAC's channels until their stages.
    pub ports: StandIns,
    pub blank: Blank,
    /// The bus's clock, in IOP cycles.
    pub now: u64,
    /// The EE cycle up to which the units have been brought, and the next at which one needs attention.
    synced: u64,
    next: u64,
    /// Where a run of instructions stops: the earlier of `next` and the run's end, in EE cycles.
    horizon: u64,
    fault: bool,
}

impl IopBus {
    pub fn new(blank: Blank) -> IopBus {
        let mut b = IopBus {
            ram: vec![0; RAM],
            scratch: [0; SCRATCH],
            icache: ICache::default(),
            rom: Vec::new(),
            intc: Intc::default(),
            dmac: Dmac::default(),
            timers: core::array::from_fn(Timer::new),
            ports: StandIns::default(),
            blank,
            now: 0,
            synced: 0,
            next: 0,
            horizon: 0,
            fault: false,
        };
        b.reschedule();
        b
    }

    pub fn load(&mut self, at: u32, words: &[u32]) {
        for (i, w) in words.iter().enumerate() {
            put(&mut self.ram, at as usize + 4 * i, Size::Word, *w);
        }
    }

    pub fn word(&self, at: u32) -> u32 {
        get(&self.ram, at as usize & (RAM - 1), Size::Word)
    }

    fn wants_h(&self) -> bool {
        self.timers.iter().any(Timer::wants_h)
    }

    fn reschedule(&mut self) {
        let mut next = self.blank.next_edge(self.synced, self.wants_h()).min(self.dmac.next_event().saturating_mul(IOP));
        for t in &self.timers {
            next = next.min(t.next_event());
        }
        self.next = next;
    }

    /// Brings every unit to EE cycle `t`, taking the events due on the way in time order: blanking edges, timer
    /// requests and the DMAC's units, whose cycles hold the processor.
    pub fn run_units(&mut self, t: u64) {
        while self.next <= t {
            let at = self.next;
            let edge = self.blank.next_edge(self.synced, self.wants_h());
            for n in 0..6 {
                if self.timers[n].sync(at) {
                    self.intc.raise(source::TIMERS[n]);
                }
            }
            if edge == at {
                let e = self.blank.edges(at);
                let (h, v) = (self.blank.h(at), self.blank.v(at));
                for n in 0..6 {
                    if self.timers[n].edges(e, h, v) {
                        self.intc.raise(source::TIMERS[n]);
                    }
                }
                if e.v_rise {
                    self.intc.raise(source::VBLANK_START);
                }
                if e.v_fall {
                    self.intc.raise(source::VBLANK_END);
                }
            }
            if self.dmac.next_event().saturating_mul(IOP) <= at {
                let IopBus { dmac, ram, ports, .. } = self;
                if dmac.step(at.div_ceil(IOP), &mut RamPort(ram), ports) {
                    self.intc.raise(source::DMA);
                }
            }
            self.synced = at;
            self.reschedule();
            debug_assert!(self.next > at, "an event at {at} asked for its own time again");
        }
        self.synced = self.synced.max(t);
    }

    /// Before an instruction: the units brought to the clock, and the processor held while the DMAC has the bus.
    pub fn service(&mut self) {
        loop {
            self.run_units(self.now * IOP);
            if self.dmac.hold <= self.now {
                return;
            }
            self.now = self.dmac.hold;
        }
    }

    /// The bus's next event, in EE cycles.
    pub fn next_event(&self) -> u64 {
        self.next
    }

    pub fn set_horizon(&mut self, t: u64) {
        self.horizon = t.min(self.next);
    }

    fn sync_timer(&mut self, n: usize, t: u64) {
        if self.timers[n].sync(t) {
            self.intc.raise(source::TIMERS[n]);
        }
    }

    /// An I/O register's read, the units brought to the clock first; registers of later stages read zero [C].
    fn read_io(&mut self, p: u32) -> u32 {
        let t = self.now * IOP;
        self.run_units(t);
        let v = if let Some((n, reg)) = timer::decode(p) {
            self.sync_timer(n, t);
            self.timers[n].read(reg)
        } else if dma::is_register(p) {
            self.dmac.read(p)
        } else {
            match p {
                intc::I_STAT | intc::I_MASK | intc::I_CTRL => self.intc.read(p),
                _ => 0,
            }
        };
        self.reschedule();
        v
    }

    /// An I/O register's write, the units brought to the clock first; a run of instructions stops after it, since it
    /// may change what is scheduled.
    fn write_io(&mut self, p: u32, v: u32) {
        let t = self.now * IOP;
        self.run_units(t);
        if let Some((n, reg)) = timer::decode(p) {
            self.sync_timer(n, t);
            let (h, vb) = (self.blank.h(t), self.blank.v(t));
            self.timers[n].write(reg, v, h, vb);
        } else if dma::is_register(p) {
            if self.dmac.write(p, v) {
                self.intc.raise(source::DMA);
            }
        } else if let intc::I_STAT | intc::I_MASK | intc::I_CTRL = p {
            self.intc.write(p, v);
        }
        self.reschedule();
        self.horizon = self.horizon.min(self.next);
    }

    /// A data read's value and cost at `target`.
    fn read_at(&mut self, target: Target, size: Size) -> u32 {
        match target {
            Target::Ram(a) => {
                self.now += RAM_STALL;
                get(&self.ram, a as usize, size)
            }
            Target::Scratch(a) => get(&self.scratch, a as usize, size),
            Target::Io(p) => {
                self.now += IO_STALL;
                let w = self.read_io(p & !3);
                w >> (8 * (p & 3))
            }
            Target::Sif(_) | Target::Control => {
                self.now += IO_STALL;
                0
            }
            Target::Rom(a) => {
                self.now += ROM_STALL;
                get(&self.rom, a as usize, size)
            }
            Target::CacheControl => self.icache.bcc,
            Target::None => {
                self.fault = true;
                0
            }
        }
    }
}

impl Bus for IopBus {
    fn fetch(&mut self, address: u32) -> u32 {
        let target = decode(address);
        let cached = address < 0xA000_0000 && self.icache.enabled() && matches!(target, Target::Ram(_) | Target::Rom(_));
        if !cached {
            self.now += 1;
            return match target {
                Target::Ram(_) | Target::Rom(_) => self.read_at(target, Size::Word),
                _ => {
                    self.fault = true;
                    0
                }
            };
        }
        let physical = address & 0x1FFF_FFFF;
        match self.icache.lookup(physical) {
            Lookup::Hit(w) => {
                self.now += 1;
                w
            }
            Lookup::Miss { first, count } => {
                let line = physical & !0xF;
                let mut words = [0u32; 4];
                for (i, w) in words.iter_mut().enumerate().take(count as usize) {
                    *w = match decode(line + 4 * (first + i as u32)) {
                        Target::Ram(a) => get(&self.ram, a as usize, Size::Word),
                        Target::Rom(a) => get(&self.rom, a as usize, Size::Word),
                        _ => 0,
                    };
                }
                let per = if matches!(target, Target::Ram(_)) { 1 } else { 1 + ROM_STALL };
                self.now += 1 + if matches!(target, Target::Ram(_)) { RAM_STALL } else { ROM_STALL } + per * u64::from(count - 1);
                self.icache.fill(physical, first, &words[..count as usize]);
                words[((physical >> 2 & 3) - first) as usize]
            }
        }
    }

    fn read(&mut self, address: u32, size: Size) -> u32 {
        let target = decode(address);
        self.read_at(target, size)
    }

    fn write(&mut self, address: u32, size: Size, value: u32) {
        match decode(address) {
            Target::Ram(a) => put(&mut self.ram, a as usize, size, value),
            Target::Scratch(a) => put(&mut self.scratch, a as usize, size, value),
            Target::Io(p) => self.write_io(p & !3, value << (8 * (p & 3))),
            Target::Sif(_) | Target::Rom(_) | Target::Control => {}
            Target::CacheControl => self.icache.bcc = value & icache::bcc::WRITABLE,
            Target::None => self.fault = true,
        }
    }

    fn isolated_read(&mut self, address: u32, _size: Size) -> u32 {
        self.icache.isolated_read(address)
    }

    fn isolated_write(&mut self, address: u32, _size: Size, value: u32) {
        self.icache.isolated_write(address, value);
    }

    fn idle(&mut self, cycles: u32) {
        self.now += u64::from(cycles);
    }

    fn cycles(&mut self) -> u64 {
        self.now
    }

    fn interrupt(&mut self) -> bool {
        self.intc.line()
    }

    fn cop2_data(&mut self, _register: u8) -> u32 {
        0
    }

    fn set_cop2_data(&mut self, _register: u8, _value: u32) {}

    fn cop2_control(&mut self, _register: u8) -> u32 {
        0
    }

    fn set_cop2_control(&mut self, _register: u8, _value: u32) {}

    fn cop2_command(&mut self, _command: u32) {}

    fn bus_error(&mut self) -> bool {
        std::mem::take(&mut self.fault)
    }

    fn running(&mut self) -> bool {
        self.now * IOP < self.horizon
    }
}
