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
use crate::cdvd::{self, Cdvd};
use crate::pcrtc::timing::Timing;
use crate::clock::IOP;
use crate::reports::{Access, Reports, Side};
use crate::sif::{self, IopPorts, Sif};
use crate::sio2::{self, Sio2};
use crate::spu2::{self, Spu2};

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
    /// The I/O registers at 1F400000h-1F4FFFFFh (CDVD), 1F801000h-1F80FFFFh and the SPU2's 1F900000h-1F9007FFh, the rest
    /// of 1F900000h-1F90FFFFh reading zero as D-61's I/O gaps do (the BIOS writes 1F900B60h; Sapphire_Disputes.md D-346).
    Io(u32),
    /// The SIF's registers at 1D000000h, S22's.
    Sif(u32),
    /// 1E000000h-1EFFFFFFh, the place of the DVD player's ROMs, whose images are not taken (Sapphire_Plan.md §5.9), and
    /// the region past RAM's 8 MiB below the SIF: on the bus, reading zero, taking writes without effect (D-346, D-482).
    Absent,
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
        RAM_WINDOW..0x1D00_0000 => Target::Absent,
        0x1D00_0000..0x1D00_0100 => Target::Sif(p),
        0x1E00_0000..0x1F00_0000 => Target::Absent,
        0x1F40_0000..0x1F50_0000 | 0x1F80_1000..0x1F81_0000 | 0x1F90_0000..0x1F91_0000 => Target::Io(p),
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
    /// The SIF, its registers here at 1D000000h and at the EE's 1000F200h, its FIFOs behind channels 2, 9 and 10 (S22).
    pub sif: Sif,
    /// SIO2, its registers at 1F808200h and its FIFOs behind channels 11 and 12 (S25).
    pub sio2: Sio2,
    /// The SPU2, its registers at 1F900000h, its cores behind channels 4 and 7 (S26).
    pub spu2: Spu2,
    /// The CDVD drive, its registers at 1F402000h, its buffer behind channel 3 and its interrupt IRQ 2 (S23, joined
    /// at S24).
    pub cdvd: Cdvd,
    /// The drive's interrupt line as last seen, so that IRQ 2 is raised at its rise.
    cdvd_line: bool,
    /// Accesses no unit answers, for the hardware's reports (S24).
    pub reports: Reports,
    pub blank: Timing,
    /// The bus's clock, in IOP cycles.
    pub now: u64,
    /// The EE cycle up to which the units have been brought, and the next at which one needs attention.
    synced: u64,
    next: u64,
    /// Where a run of instructions stops: the earlier of `next` and the run's end, in EE cycles.
    horizon: u64,
    fault: bool,
    /// Where a processor store completed an export table's magic word in RAM, newest last (S10, `tty`).
    pub export_tables: Vec<u32>,
    /// The processor's writes, its accesses to anything but RAM, the scratchpad and the boot ROM, and the I-cache's
    /// refills: the bus's own counts, which idle skipping reads (Sapphire_Native.md §47), none of them in the state.
    pub writes: u64,
    pub io: u64,
    pub fills: u64,
    /// The physical address a run of the processor stops in front of, the console hook's while it is armed; none at
    /// `u32::MAX`. Not state.
    pub breakpoint: u32,
}

impl IopBus {
    pub fn new(blank: Timing) -> IopBus {
        let mut b = IopBus {
            ram: vec![0; RAM],
            scratch: [0; SCRATCH],
            icache: ICache::default(),
            rom: Vec::new(),
            intc: Intc::default(),
            dmac: Dmac::default(),
            timers: core::array::from_fn(Timer::new),
            ports: StandIns::default(),
            sif: Sif::default(),
            sio2: Sio2::default(),
            spu2: Spu2::new(),
            cdvd: Cdvd::new(None),
            cdvd_line: false,
            reports: Reports::default(),
            blank,
            now: 0,
            synced: 0,
            next: 0,
            horizon: 0,
            fault: false,
            export_tables: Vec::new(),
            writes: 0,
            io: 0,
            fills: 0,
            breakpoint: u32::MAX,
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
        let mut next = self.blank.next_edge(self.synced, self.wants_h()).min(self.dmac.next_event().saturating_mul(IOP)).min(self.sio2.next_event());
        for t in &self.timers {
            next = next.min(t.next_event());
        }
        self.next = next.min(self.spu2.next_event().saturating_mul(IOP)).min(self.cdvd.next_event().saturating_mul(IOP));
    }

    /// Brings every unit to EE cycle `t`, taking the events due on the way in time order: blanking edges, timer
    /// requests and the DMAC's units, whose cycles hold the processor.
    pub fn run_units(&mut self, t: u64) {
        if self.sif.take_wake_iop() | self.spu2.take_wake() {
            self.dmac.wake_from(self.synced.max(t).div_ceil(IOP));
            self.reschedule();
        }
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
            if self.sio2.complete(at) {
                self.intc.raise(sio2::SOURCE);
                self.dmac.wake_from(at.div_ceil(IOP));
            }
            self.sync_spu2(at);
            if self.cdvd.next_event().saturating_mul(IOP) <= at {
                self.cdvd.run(at / IOP);
                self.sync_cdvd();
                self.dmac.wake_from(at.div_ceil(IOP));
            }
            if self.dmac.next_event().saturating_mul(IOP) <= at {
                let IopBus { dmac, ram, ports, sif, sio2, spu2, cdvd, .. } = self;
                let now = at.div_ceil(IOP);
                let mut drive = cdvd::Ports { others: &mut sio2::Ports { others: ports, sio2 }, cdvd, now };
                let mut spu = spu2::Ports { others: &mut drive, spu2, irq: false };
                if dmac.step(at.div_ceil(IOP), &mut RamPort(ram), &mut IopPorts { others: &mut spu, sif }) {
                    self.intc.raise(source::DMA);
                }
                if spu.irq {
                    self.intc.raise(source::SPU2);
                }
                if self.spu2.take_wake() {
                    self.dmac.wake_from(at.div_ceil(IOP));
                }
                self.sync_cdvd();
            }
            self.synced = at;
            self.reschedule();
            debug_assert!(self.next > at, "an event at {at} asked for its own time again");
        }
        self.synced = self.synced.max(t);
        self.sync_spu2(t);
    }

    /// The SPU2 brought to EE cycle `t`, its IRQ raised if one rose; a buffer AutoDMA freed wakes the DMAC.
    fn sync_spu2(&mut self, t: u64) {
        if self.spu2.run(t / IOP) {
            self.intc.raise(source::SPU2);
        }
        if self.spu2.take_wake() {
            self.dmac.wake_from(t.div_ceil(IOP));
            self.reschedule();
        }
    }

    /// The player's hand on the drive's tray at EE cycle `t`, the units brought there first (Sapphire_Plan.md §8, G10).
    pub fn tray(&mut self, open: bool, t: u64) {
        self.run_units(t);
        self.cdvd.tray(open, t / IOP);
        self.sync_cdvd();
        self.reschedule();
    }

    /// IRQ 2 raised at each reason the drive sets in its I_STAT, whatever reasons are still pending (ps2tek "IOP
    /// Interrupts", "CDVD I/O Ports"; D-481).
    fn sync_cdvd(&mut self) {
        if self.cdvd.take_raised() {
            self.intc.raise(source::CDVD);
        }
        self.cdvd_line = self.cdvd.irq();
    }

    /// A read of the drive's byte registers at `p`, a wider access taking the bytes in order [C].
    pub(crate) fn cdvd_read(&mut self, p: u32, size: Size) -> u32 {
        self.run_units(self.now * IOP);
        let v = (0..size.bytes()).fold(0u32, |v, k| v | u32::from(self.cdvd.read8(p - cdvd::BASE + k)) << (8 * k));
        self.reschedule();
        v
    }

    pub(crate) fn cdvd_write(&mut self, p: u32, size: Size, value: u32) {
        self.run_units(self.now * IOP);
        for k in 0..size.bytes() {
            self.cdvd.write8(p - cdvd::BASE + k, (value >> (8 * k)) as u8, self.now);
        }
        self.sync_cdvd();
        self.dmac.wake_from(self.now);
        self.reschedule();
        self.horizon = self.horizon.min(self.next);
    }

    /// The IOP's own units at the processor's reset (Sapphire_Native.md §35.2, D-340): the INTC, the DMAC, the six
    /// timers and the cache control; RAM, the SIF and the devices beside the IOP keep their state.
    pub fn reset_units(&mut self) {
        let t = self.now * IOP;
        self.intc = Intc::default();
        self.dmac = Dmac::default();
        self.dmac.wake_from(self.now);
        self.timers = core::array::from_fn(|n| Timer::reset_at(n, t));
        self.icache = ICache::default();
        self.reschedule();
    }

    /// A read of the SPU2's registers at `p`, by halfwords (Sapphire_Native.md §32.3).
    fn spu2_read(&mut self, p: u32, size: Size) -> u32 {
        self.run_units(self.now * IOP);
        let o = p - spu2::BASE;
        let v = match size {
            Size::Word => self.spu2.read16(o & !3) as u32 | (self.spu2.read16((o & !3) + 2) as u32) << 16,
            _ => (self.spu2.read16(o & !1) as u32) >> (8 * (o & 1)),
        };
        self.reschedule();
        v
    }

    /// A write to the SPU2's registers: a word as two halfwords, low first; a byte at an even address as a halfword's,
    /// at an odd one ignored [A, the PlayStation's, Diamond_SPU.md §2].
    fn spu2_write(&mut self, p: u32, size: Size, value: u32) {
        self.run_units(self.now * IOP);
        let o = p - spu2::BASE;
        let rose = match size {
            Size::Word => self.spu2.write16(o & !3, value as u16) | self.spu2.write16((o & !3) + 2, (value >> 16) as u16),
            _ if size == Size::Byte && o & 1 != 0 => false,
            _ => self.spu2.write16(o & !1, value as u16),
        };
        if rose {
            self.intc.raise(source::SPU2);
        }
        self.reschedule();
        self.horizon = self.horizon.min(self.next);
    }

    /// Before an instruction: the units brought to the clock, and the processor held while the DMAC has the bus.
    pub fn service(&mut self) {
        let t = self.now * IOP;
        if self.next > t && !self.sif.wake_iop_pending() && !self.spu2.wake_pending() && !self.spu2.due(self.now) && self.dmac.hold <= self.now {
            // `run_units` with nothing due moves only its sync time (Sapphire_Scheduler.md §54.3).
            self.synced = self.synced.max(t);
            return;
        }
        loop {
            self.run_units(self.now * IOP);
            if self.dmac.hold <= self.now {
                return;
            }
            self.now = self.dmac.hold;
        }
    }

    /// The EE's write to a SIF register at `offset`, which may raise the IOP's interrupt (Sapphire_Disputes.md D-141);
    /// the IOP's reset line when the write changes it, CTRL's bit 19 asserting it while set (D-340).
    pub fn sif_write_from_ee(&mut self, offset: u32, v: u32) -> Option<bool> {
        if self.sif.ee_write(offset, v) {
            self.intc.raise(sif::IOP_SOURCE);
        }
        let line = v & sif::CTRL_IOP_RESET != 0;
        (offset == sif::reg::CTRL && line != self.sif.reset_line).then(|| {
            self.sif.reset_line = line;
            line
        })
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
        } else if sio2::is_register(p) {
            self.sio2.read(p)
        } else {
            match p {
                intc::I_STAT | intc::I_MASK | intc::I_CTRL => self.intc.read(p),
                _ => {
                    self.reports.note(Side::Iop, p, Access::Read, None, t);
                    0
                }
            }
        };
        self.reschedule();
        v
    }

    /// An I/O register's write, the units brought to the clock first; a run of instructions stops after it, since it
    /// may change what is scheduled. A halfword or byte written to a DMA channel's register changes only its own lanes
    /// (the BIOS writes BCR's upper half alone; Sapphire_Disputes.md D-347).
    fn write_io(&mut self, p: u32, v: u32) {
        let t = self.now * IOP;
        self.run_units(t);
        if let Some((n, reg)) = timer::decode(p) {
            self.sync_timer(n, t);
            let (h, vb) = (self.blank.h(t), self.blank.v(t));
            self.timers[n].write(reg, v, h, vb);
        } else if dma::is_register(p) {
            // A channel started on an idle DMAC begins at this write, not at the last transfer's end (D-361).
            self.dmac.wake_from(self.now);
            if self.dmac.write(p, v) {
                self.intc.raise(source::DMA);
            }
        } else if let intc::I_STAT | intc::I_MASK | intc::I_CTRL = p {
            self.intc.write(p, v);
        } else if sio2::is_register(p) {
            self.sio2.write(p, v, t);
        } else {
            self.reports.note(Side::Iop, p, Access::Write, Some(v), t);
        }
        self.reschedule();
        self.horizon = self.horizon.min(self.next);
    }

    /// A data read's value and cost at `target`.
    fn read_at(&mut self, target: Target, size: Size) -> u32 {
        if let Target::Io(p) | Target::Sif(p) = target {
            self.reports.touch(Side::Iop, p, Access::Read, None, self.now * IOP);
        }
        match target {
            Target::Ram(a) => {
                self.now += RAM_STALL;
                get(&self.ram, a as usize, size)
            }
            Target::Scratch(a) => get(&self.scratch, a as usize, size),
            Target::Io(p) if (spu2::BASE..spu2::END).contains(&p) => {
                self.now += IO_STALL;
                self.spu2_read(p, size)
            }
            Target::Io(p) if (cdvd::BASE..cdvd::END).contains(&p) => {
                self.now += IO_STALL;
                self.cdvd_read(p, size)
            }
            Target::Io(p) => {
                self.now += IO_STALL;
                let w = self.read_io(p & !3);
                w >> (8 * (p & 3))
            }
            Target::Sif(p) => {
                self.now += IO_STALL;
                self.sif.iop_read(p & 0xF0) >> (8 * (p & 3))
            }
            Target::Control | Target::Absent => {
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

    /// An access no device answers, noted by its virtual address (`Target::None`).
    fn unmapped(&mut self, address: u32, access: Access, value: Option<u32>) {
        self.reports.note(Side::Iop, address, access, value, self.now * IOP);
        self.fault = true;
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
                    self.unmapped(address, Access::Fetch, None);
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
                self.fills += 1;
                words[((physical >> 2 & 3) - first) as usize]
            }
        }
    }

    /// A cached hit answered from the word decoded at its refill (Sapphire_Native.md §53); every other fetch as `fetch`.
    #[inline(always)]
    fn fetch_decoded(&mut self, address: u32) -> (u32, emusen_mips::Op) {
        let p = address & 0x1FFF_FFFF;
        let segment = address < 0x2000_0000 || (0x8000_0000..0xA000_0000).contains(&address);
        if segment
            && (p < RAM_WINDOW || p >= ROM_BASE)
            && self.icache.enabled()
            && let Some(hit) = self.icache.hit(p)
        {
            self.now += 1;
            return hit;
        }
        let w = self.fetch(address);
        (w, emusen_mips::decode(w))
    }

    fn read(&mut self, address: u32, size: Size) -> u32 {
        let target = decode(address);
        if !matches!(target, Target::Ram(_) | Target::Scratch(_) | Target::Rom(_)) {
            self.io += 1;
        }
        if target == Target::None {
            self.unmapped(address, Access::Read, None);
            return 0;
        }
        if target == Target::Control {
            self.reports.note(Side::Iop, address, Access::Read, None, self.now * IOP);
        }
        self.read_at(target, size)
    }

    fn write(&mut self, address: u32, size: Size, value: u32) {
        let target = decode(address);
        self.writes += 1;
        if let Target::Io(p) | Target::Sif(p) = target {
            self.reports.touch(Side::Iop, p, Access::Write, Some(value), self.now * IOP);
        }
        match target {
            Target::Ram(a) => {
                put(&mut self.ram, a as usize, size, value);
                let at = a & !3;
                if get(&self.ram, at as usize, Size::Word) == super::tty::EXPORT_MAGIC && self.export_tables.last() != Some(&at) {
                    self.export_tables.retain(|&t| t != at);
                    if self.export_tables.len() == super::tty::TABLES {
                        self.export_tables.remove(0);
                    }
                    self.export_tables.push(at);
                }
            }
            Target::Scratch(a) => put(&mut self.scratch, a as usize, size, value),
            Target::Io(p) if (spu2::BASE..spu2::END).contains(&p) => self.spu2_write(p, size, value),
            Target::Io(p) if (cdvd::BASE..cdvd::END).contains(&p) => self.cdvd_write(p, size, value),
            Target::Io(p) if size != Size::Word && dma::is_register(p & !3) => {
                let shift = 8 * (p & 3);
                let mask = if size == Size::Byte { 0xFF } else { 0xFFFF } << shift;
                let merged = self.dmac.read(p & !3) & !mask | value << shift & mask;
                self.write_io(p & !3, merged)
            }
            Target::Io(p) => self.write_io(p & !3, value << (8 * (p & 3))),
            Target::Sif(p) => self.sif.iop_write(p & 0xF0, value << (8 * (p & 3))),
            Target::Rom(_) | Target::Absent => {}
            Target::Control => self.reports.note(Side::Iop, address, Access::Write, Some(value), self.now * IOP),
            Target::CacheControl => self.icache.bcc = value & icache::bcc::WRITABLE,
            Target::None => self.unmapped(address, Access::Write, Some(value)),
        }
    }

    fn isolated_read(&mut self, address: u32, _size: Size) -> u32 {
        self.io += 1;
        self.icache.isolated_read(address)
    }

    fn isolated_write(&mut self, address: u32, _size: Size, value: u32) {
        self.writes += 1;
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

    #[inline(always)]
    fn stop_before(&mut self, pc: u32) -> bool {
        pc & 0x1FFF_FFFF == self.breakpoint
    }
}

impl IopBus {
    /// The bus's own part of the IOP's block in `SPH2` (Sapphire_Native.md §38.2): RAM, the cache, the scratchpad, the
    /// INTC, the DMAC, the timers, the timing it follows and its clocks. The SIF, the drive, SIO2 and the SPU2 are
    /// blocks of their own; the boot ROM is firmware and never in a state; the stand-ins and reports keep nothing a
    /// program sees.
    pub fn write_own(&self, w: &mut emusen_native::StateWriter) {
        w.bytes("Ram", &self.ram);
        w.u32("CacheControl", self.icache.bcc);
        w.u32s("CacheTags", &self.icache.tags);
        w.u32s("CacheWords", &self.icache.words);
        w.bytes("Scratchpad", &self.scratch);
        w.u32s("Intc", &[self.intc.stat, self.intc.mask, self.intc.ctrl]);
        w.structure("Dmac", &self.dmac);
        w.structures("Timers", &self.timers);
        w.structure("Blank", &self.blank);
        w.u64s("Clocks", &[self.now, self.synced, self.next, self.horizon]);
        w.bools("Lines", &[self.cdvd_line, self.fault]);
    }

    pub fn read_own(&mut self, r: &mut emusen_native::StateReader) -> Result<(), emusen_native::Truncated> {
        use emusen_native::State;
        r.bytes(&mut self.ram)?;
        self.icache.bcc = r.u32()?;
        r.u32s(&mut self.icache.tags)?;
        r.u32s(&mut self.icache.words)?;
        self.icache.redecode();
        r.bytes(&mut self.scratch)?;
        let mut i = [0u32; 3];
        r.u32s(&mut i)?;
        (self.intc.stat, self.intc.mask, self.intc.ctrl) = (i[0], i[1], i[2]);
        self.dmac.read_state(r)?;
        r.structures(&mut self.timers)?;
        self.blank.read_state(r)?;
        let mut c = [0u64; 4];
        r.u64s(&mut c)?;
        [self.now, self.synced, self.next, self.horizon] = c;
        let mut l = [false; 2];
        r.bools(&mut l)?;
        [self.cdvd_line, self.fault] = l;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// D-481 through the bus: with the tray's opening still pending in the drive's I_STAT, a command's end sets I_STAT
    /// bit 2 of the IOP's INTC again.
    #[test]
    fn the_drives_command_end_reaches_the_intc_with_the_opening_unacknowledged() {
        let mut b = IopBus::new(crate::pcrtc::Pcrtc::new(crate::clock::NTSC).timing);
        b.tray(true, 0);
        assert!(b.intc.stat & 1 << source::CDVD != 0);
        b.intc.stat = 0;
        b.tray(false, 0);
        let t = 4 * cdvd::timing::SPIN_UP * IOP;
        b.run_units(t);
        b.now = t / IOP;
        b.cdvd_write(cdvd::BASE + 4, Size::Byte, 0);
        b.run_units(t + 100_000 * IOP);
        assert!(b.intc.stat & 1 << source::CDVD != 0, "IRQ 2 raised at the command's end");
    }
}
