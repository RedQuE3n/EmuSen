//! The EE's side of the console around its core (Sapphire_Native.md §20.1): the physical map, the DMAC, the INTC, the
//! four timers and the PCRTC's video timing, on one clock of EE cycles (`clock`), and `System`, which runs the core and
//! brings these units to the core's time after each instruction. A unit is brought forward only at the events it
//! schedules (a DMA unit's end, a timer's flag, a blanking edge, a line the PCRTC reads) or when the core reads or writes it; nothing is
//! polled cycle by cycle. The core reaches the registers through `Bus`; the units drive its Int0, Int1 and CPCOND0.
//! From S16 the GS sits on the board too: its privileged registers at 12000000h, its interrupt latched by the INTC at
//! its rising edge. From S20 the PCRTC (`pcrtc`) gives H-blank, V-blank, V-sync and the field in place of S11's
//! stand-in, and reads each displayed line from the GS's memory at its time (Sapphire_Native.md §29.4).
//! From S14 VIF0 and VIF1 take the DMAC's channels 0 and 1, their registers at 10003800h and FIFOs at 10004000h
//! (Sapphire_Native.md §28.5). From S15 the GIF (`gif`) takes channel 2, VIF1's PATH2 and VU1's PATH1, its registers at
//! 10003000h and its FIFO at 10006000h, and drives the GS's host interface (Sapphire_Native.md §33.5).
//! From S27 the IPU takes the DMAC's channels 3 and 4, its registers at 10002000h and FIFOs at 10007000h, its interrupt
//! latched by the INTC as a command ends (Sapphire_Native.md §34.7).

use std::collections::VecDeque;

use crate::clock::{self, Standard};
use crate::dmac::{self, Dmac, Peripherals};
use crate::ee::map::{self, Map, Region};
use crate::ee::{Bus, Ee, Halt};
use crate::gif::{self, Gif};
use crate::gs::{self, Gs, HostInterface};
use crate::intc::{self, Intc};
use crate::iop::Iop;
use crate::ipu::{self, Ipu};
use crate::mch::{self, Mch};
use crate::pcrtc::Pcrtc;
use crate::reports::{Access, Reports, Side};
use crate::pcrtc::timing::Mode;
use crate::sif::{self, EePorts};
use crate::timer::{self, Timer};
use crate::vif::{self, Reach, Vif};
use crate::vu::Vpu;

/// A stand-in for a peripheral the DMAC reaches before its stage exists (none since S27: the SIF's reach `sif` since
/// S22, the VIFs', the GIF's and the IPU's their units since S14, S15 and S27): it takes every qword offered, keeping them when `keep` is set, and gives the qwords queued in
/// `give`, holding its channel when it has none.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StandIn {
    pub taken: Vec<(u128, bool)>,
    pub count: u64,
    pub keep: bool,
    pub give: VecDeque<u128>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StandIns {
    pub ports: [StandIn; 8],
}

impl Peripherals for StandIns {
    fn request(&mut self, ch: usize, want: usize) -> usize {
        let p = &self.ports[ch];
        let to_memory = matches!(ch, dmac::FROM_IPU | dmac::SIF0) || (matches!(ch, dmac::VIF1 | dmac::SIF2) && !p.give.is_empty());
        if to_memory { want.min(p.give.len()) } else { want }
    }

    fn put(&mut self, ch: usize, q: u128, tag: bool) {
        let p = &mut self.ports[ch];
        p.count += 1;
        if p.keep {
            p.taken.push((q, tag));
        }
    }

    fn get(&mut self, ch: usize) -> u128 {
        self.ports[ch].give.pop_front().unwrap_or(0)
    }
}

/// The DMAC's peripherals on the board: VIF0 and VIF1 on channels 0 and 1, the GIF's FIFO on channel 2, the IPU on 3
/// and 4, the stand-ins on the rest.
struct Ports<'a, 'b> {
    vif: &'a mut [Vif; 2],
    ipu: &'a mut Ipu,
    stand: &'a mut StandIns,
    gif: &'a mut GifPort<'b>,
    /// Channel 1's direction as D1_CHCR.DIR gives it: to memory when clear.
    vif1_to_memory: bool,
    /// The cycle the DMAC's step runs at, when the qwords it puts arrive.
    at: u64,
}

impl Peripherals for Ports<'_, '_> {
    fn request(&mut self, ch: usize, want: usize) -> usize {
        match ch {
            dmac::VIF0 => self.vif[0].request(want, false, self.gif),
            dmac::VIF1 => self.vif[1].request(want, self.vif1_to_memory, self.gif),
            dmac::GIF => self.gif.gif.request(want),
            dmac::FROM_IPU => self.ipu.request_out(want),
            dmac::TO_IPU => self.ipu.request_in(want),
            _ => self.stand.request(ch, want),
        }
    }

    fn put(&mut self, ch: usize, q: u128, tag: bool) {
        match ch {
            dmac::VIF0 | dmac::VIF1 => self.vif[ch].put(q, tag),
            dmac::GIF => self.gif.gif.put(q, tag, self.at),
            dmac::TO_IPU if !tag => self.ipu.put(q),
            // A tag a chain sends by TTE is not part of the IPU's bit stream (Sapphire_Disputes.md D-325).
            dmac::TO_IPU => {}
            _ => self.stand.put(ch, q, tag),
        }
    }

    fn get(&mut self, ch: usize) -> u128 {
        match ch {
            dmac::VIF1 => self.vif[1].get(self.gif),
            dmac::VIF0 | dmac::GIF => 0,
            dmac::FROM_IPU => self.ipu.get(),
            _ => self.stand.get(ch),
        }
    }
}

/// The GIF as VIF1 reaches it (`vif::Gif`): PATH2, the paths' idle for the FLUSH family with channel 2's STR as PATH3's
/// request, MSKPATH3, and the GS's host interface for the local-to-host data.
struct GifPort<'a> {
    gif: &'a mut Gif,
    gs: &'a mut Gs,
    dma: bool,
}

impl vif::Gif for GifPort<'_> {
    fn path2(&mut self, q: u128, hl: bool, at: u64) -> Option<u64> {
        self.gif.path2(q, hl, at, self.gs)
    }

    fn idle_at(&mut self, path3: bool, at: u64) -> Option<u64> {
        self.gif.idle_at(path3, self.dma, at)
    }

    fn mask_path3(&mut self, masked: bool, _: u64) {
        self.gif.mask_path3(masked);
    }

    fn host_read(&mut self) -> Option<u128> {
        let lo = self.gs.read()?;
        let hi = self.gs.read().unwrap_or(0);
        Some(u128::from(lo) | u128::from(hi) << 64)
    }
}

impl dmac::Memory for Map {
    fn read_q(&mut self, paddr: u32) -> Option<u128> {
        (map::region(paddr) != Region::BusError).then(|| Bus::read(self, paddr, 16))
    }

    fn write_q(&mut self, paddr: u32, q: u128) -> bool {
        let ok = map::region(paddr) != Region::BusError;
        if ok {
            Bus::write(self, paddr, 16, q);
        }
        ok
    }
}

pub struct Board {
    pub map: Map,
    pub dmac: Dmac,
    pub intc: Intc,
    pub timers: [Timer; 4],
    /// The PCRTC: the video timing the timers, the INTC, the GS and the IOP follow, and the picture (from S20).
    pub pcrtc: Pcrtc,
    pub ports: StandIns,
    /// VU0 and VU1, which the core reaches through COP2 and the map at 11000000h (from S12).
    pub vpu: Vpu,
    /// The IOP, a unit of its own with its bus and units, brought to the board's time (Sapphire_Native.md §22).
    pub iop: Iop,
    /// The GS (from S16), its host interface driven by the GIF (from S15).
    pub gs: Gs,
    /// VIF0 and VIF1 (from S14).
    pub vif: [Vif; 2],
    /// The GIF (from S15): channel 2's peripheral, PATH2 behind VIF1 and PATH1 behind VU1's XGKICK.
    pub gif: Gif,
    /// The IPU (from S27), on the DMAC's channels 3 and 4.
    pub ipu: Ipu,
    /// The time every unit has been brought to, in EE cycles.
    pub now: u64,
    /// The memory controller's serial interface to the RDRAMs (S24).
    pub mch: Mch,
    /// The IOP's resets by the EE's write to the SIF's CTRL (S24).
    pub iop_resets: u64,
    /// The EE's bus accesses past its caches by memory, counts only: RDRAM reads, RDRAM writes, boot ROM reads (S24).
    pub traffic: [u64; 3],
    /// Accesses no unit answers on the EE's side, for the hardware's reports (S24).
    pub reports: Reports,
    next: u64,
    /// The PCRTC's next event as last computed, with the H-blank wish it was asked with: still the next while `now` is
    /// before it and nothing has changed the timing (a pure function of both, cached; Sapphire_Native.md §35.7).
    pcrtc_next: Option<(u64, bool)>,
    /// The time whose blanking edges were last taken, so an event rescheduled at that time does not take them again;
    /// the timing's start at 0 is no edge.
    edges_taken: Option<u64>,
}

impl Board {
    pub fn new(standard: Standard) -> Board {
        let mut b = Board {
            map: Map::new(),
            dmac: Dmac::default(),
            intc: Intc::default(),
            timers: Default::default(),
            pcrtc: Pcrtc::new(standard),
            ports: StandIns::default(),
            vpu: Vpu::new(),
            iop: Iop::new(Pcrtc::new(standard).timing),
            gs: Gs::new(),
            vif: [Vif::new(0), Vif::new(1)],
            gif: Gif::new(),
            ipu: Ipu::new(),
            now: 0,
            mch: Mch::default(),
            iop_resets: 0,
            traffic: [0; 3],
            reports: Reports::default(),
            next: 0,
            edges_taken: Some(0),
            pcrtc_next: None,
        };
        b.vpu.units.link.path1 = Box::new(gif::Path1Port::default());
        b.reschedule();
        b
    }

    fn wants_h(&self) -> bool {
        self.timers.iter().any(Timer::wants_h) || self.gs.wants_hsync()
    }

    /// The GS's interrupt, latched into I_STAT at its rising edge (EE User's Manual §3.1); called after the GS's host
    /// interface is driven (the GIF, from S15).
    pub fn sync_gs(&mut self) {
        if self.gs.take_rise() {
            self.intc.raise(intc::source::GS);
        }
    }

    /// The video mode from the next field (the kernel's mode setting, `Pcrtc::set_mode`), the IOP's copy of the timing
    /// told too.
    pub fn set_mode(&mut self, mode: Mode) {
        if self.pcrtc.set_mode(self.now, mode) {
            self.iop.bus.blank = self.pcrtc.timing;
        }
        self.pcrtc_next = None;
        self.reschedule();
    }

    /// The next time any unit needs attention.
    fn reschedule(&mut self) {
        let want = self.wants_h();
        let edge = match self.pcrtc_next {
            Some((e, w)) if w == want && self.now < e => e,
            _ => {
                let e = self.pcrtc.next_event(self.now, want);
                self.pcrtc_next = Some((e, want));
                e
            }
        };
        let mut next = edge.min(self.dmac.next_event());
        for t in &self.timers {
            next = next.min(t.next_event());
        }
        for v in &self.vif {
            next = next.min(v.next_event());
        }
        next = next.min(self.gif.next_event());
        self.next = next.max(self.now);
    }

    fn sync_timer(&mut self, n: usize, t: u64) {
        if self.timers[n].sync(t) != 0 {
            self.intc.raise(intc::source::TIMER0 + n as u32);
        }
    }

    /// Brings every unit to time `t`, running the events due on the way; the scratchpad is the core's. With no event
    /// due and the GIF, the VIFs and the VUs idle, only the clocks move and the IOP is brought along, which is what the
    /// whole path would do (Sapphire_Native.md §35.7).
    pub fn run_to(&mut self, t: u64, spram: &mut [u8]) {
        if self.next > t && self.gif.quiet() && self.vif[0].idle_to(t) & self.vif[1].idle_to(t) && self.vpu.idle_to(t) {
            self.now = self.now.max(t);
            self.iop.catch_up(self.now);
            if self.iop.bus.sif.take_wake_ee() {
                self.dmac.wake(self.now);
                if self.dmac.next_event() <= self.now {
                    self.step_dmac(self.now, spram);
                    self.sync_ipu(self.now);
                }
                self.reschedule();
            }
            return;
        }
        while self.next <= t {
            let at = self.next;
            self.now = at;
            self.pcrtc_next = None;
            for n in 0..4 {
                self.sync_timer(n, at);
            }
            let e = if self.edges_taken == Some(at) { Default::default() } else { self.pcrtc.edges(at) };
            if e.any() {
                self.edges_taken = Some(at);
                let (h, v) = (self.pcrtc.timing.h(at), self.pcrtc.timing.v(at));
                for n in 0..4 {
                    if self.timers[n].edges(e.h_rise, e.h_fall, e.v_rise, e.v_fall, h, v) != 0 {
                        self.intc.raise(intc::source::TIMER0 + n as u32);
                    }
                }
                if e.h_rise {
                    self.gs.hsync();
                }
                if e.v_rise {
                    self.intc.raise(intc::source::VB_ON);
                    self.pcrtc.begin_field(at, &self.gs);
                }
                if e.vsync {
                    self.gs.vsync(self.pcrtc.odd());
                }
                self.sync_gs();
                if e.v_fall {
                    self.intc.raise(intc::source::VB_OFF);
                }
            }
            self.pcrtc.read_due(at, &self.gs);
            loop {
                self.run_vifs(at);
                if self.dmac.next_event() > at {
                    break;
                }
                self.iop.run_to(at);
                self.step_dmac(at, spram);
                self.sync_ipu(at);
            }
            self.reschedule();
            debug_assert!(self.next > at, "an event at {at} asked for its own time again");
        }
        self.now = self.now.max(t);
        self.run_vifs(t);
        self.reschedule();
        let free = self.gif.path1_free(self.now);
        self.vpu.units.link.path1.set_free(free);
        self.vpu.catch_up(t);
        let irq = self.vpu.take_interrupts();
        for (unit, source) in [intc::source::VU0, intc::source::VU1].into_iter().enumerate() {
            if irq >> unit & 1 == 1 {
                self.intc.raise(source);
            }
        }
        self.run_gif(t);
        self.reschedule();
        self.iop.catch_up(self.now);
        if self.iop.bus.sif.take_wake_ee() {
            self.dmac.wake(self.now);
            if self.dmac.next_event() <= self.now {
                self.step_dmac(self.now, spram);
            }
            self.reschedule();
        }
    }

    /// `run_to` through each of `times` in turn, as that many calls make it, `each` shown the lines after each time and
    /// stopping the run by returning false; the times run. While no event falls by the last time and the GIF, the VIFs
    /// and the VUs are idle, each call is the IOP brought along and nothing else until the IOP's side wakes the EE's
    /// DMAC, so the checks are made once and the clocks' moves once at the end; from a wake on, each time is `run_to`
    /// itself (Sapphire_Native.md §44.5).
    pub fn run_through(&mut self, times: &[u64], spram: &mut [u8], each: &mut dyn FnMut(usize, (bool, bool, bool)) -> bool) -> usize {
        let Some(&last) = times.last() else { return 0 };
        let quiet = self.next > last && self.gif.quiet() && self.vif[0].idle() && self.vif[1].idle() && self.vpu.idle() && !self.iop.bus.sif.wake_ee_pending();
        let mut from = 0;
        if quiet {
            let lines = self.lines();
            while from < times.len() {
                let t = times[from];
                self.now = self.now.max(t);
                self.iop.catch_up(self.now);
                if self.iop.bus.sif.wake_ee_pending() {
                    break;
                }
                from += 1;
                if !each(from - 1, lines) {
                    self.idle_through(t);
                    return from;
                }
            }
            if from == times.len() {
                self.idle_through(last);
                return from;
            }
            // The wake at `times[from]`, taken as the quick path of `run_to` takes it.
            let t = times[from];
            self.idle_through(t);
            if self.iop.bus.sif.take_wake_ee() {
                self.dmac.wake(self.now);
                if self.dmac.next_event() <= self.now {
                    self.step_dmac(self.now, spram);
                    self.sync_ipu(self.now);
                }
                self.reschedule();
            }
            from += 1;
            if !each(from - 1, self.lines()) {
                return from;
            }
        }
        while from < times.len() {
            self.run_to(times[from], spram);
            from += 1;
            if !each(from - 1, self.lines()) {
                return from;
            }
        }
        from
    }

    /// The clocks' moves `run_to`'s quick path makes at `t` once it has found the units idle.
    fn idle_through(&mut self, t: u64) {
        let idle = self.vif[0].idle_to(t) & self.vif[1].idle_to(t) && self.vpu.idle_to(t);
        debug_assert!(idle, "the units were idle when the run began");
    }

    /// The DMAC at `at`, woken first if the IOP's side has changed a SIF FIFO; SIF0, SIF1 and SIF2 reach the SIF.
    fn step_dmac(&mut self, at: u64, spram: &mut [u8]) {
        if self.iop.bus.sif.take_wake_ee() {
            self.dmac.wake(at);
        }
        let sif2_from_memory = self.dmac.ch[dmac::SIF2].chcr & dmac::chcr::DIR != 0;
        let vif1_to_memory = self.dmac.ch[dmac::VIF1].chcr & dmac::chcr::DIR == 0;
        let dma = self.dmac.ch[dmac::GIF].str();
        let Board { dmac, map, vif, ports, gif, gs, iop, ipu, .. } = self;
        let mut gif = GifPort { gif, gs, dma };
        let mut vifs = Ports { vif, ipu, stand: ports, gif: &mut gif, vif1_to_memory, at };
        let mut ports = EePorts { others: &mut vifs, sif: &mut iop.bus.sif, sif2_from_memory };
        dmac.step(at, map, spram, &mut ports);
    }

    /// The IPU's interrupt at a command's end, and the DMAC woken for a FIFO the IPU has filled or drained.
    fn sync_ipu(&mut self, at: u64) {
        if self.ipu.take_irq() {
            self.intc.raise(intc::source::IPU);
        }
        if self.ipu.take_wake() {
            self.dmac.wake(at.max(self.now));
        }
    }

    /// Brings the GIF to cycle `t`: the XGKICKs VU1 made handed to it, then its paths run, the DMAC woken for FIFO room
    /// freed, and the GS's interrupt latched.
    fn run_gif(&mut self, t: u64) {
        while let Some((addr, at, told)) = self.vpu.units.link.path1.take_kick() {
            self.gif.kick(addr, at, told);
        }
        self.gif.run_to(t, &self.vpu.units.vu1, &mut self.gs);
        if self.gif.take_wake() {
            self.dmac.wake(t.max(self.now));
        }
        self.sync_gs();
    }

    /// Brings the GIF, then VIF0 and VIF1 to cycle `t` (each before its VU), waking the DMAC for a FIFO they freed or
    /// filled and raising their interrupts.
    fn run_vifs(&mut self, t: u64) {
        self.run_gif(t);
        let dma = self.dmac.ch[dmac::GIF].str();
        let Board { vif, vpu, gif, gs, dmac, intc: unit_intc, now, .. } = self;
        let mut gif = GifPort { gif, gs, dma };
        for (n, v) in vif.iter_mut().enumerate() {
            v.run_to(t, &mut Reach { vpu, gif: &mut gif });
            if v.take_wake() {
                dmac.wake(t.max(*now));
            }
            if v.take_irq() {
                unit_intc.raise(intc::source::VIF0 + n as u32);
            }
        }
        if gif.gif.take_wake() {
            dmac.wake(t.max(*now));
        }
        self.sync_gs();
    }

    /// INT0, INT1 and CPCOND0 as the core sees them now: levels, sampled before each instruction (Sapphire_Disputes.md D-39).
    pub fn lines(&self) -> (bool, bool, bool) {
        (self.intc.int0(), self.dmac.int1(), self.dmac.cpcond0())
    }

    fn read_register(&mut self, paddr: u32) -> u32 {
        self.reports.touch(Side::Ee, paddr, Access::Read, None, self.now);
        if let Some((n, reg)) = timer::decode(paddr) {
            self.sync_timer(n, self.now);
            return self.timers[n].read(reg);
        }
        if let Some(offset) = sif::ee_register(paddr) {
            return self.iop.bus.sif.ee_read(offset);
        }
        match paddr {
            intc::I_STAT | intc::I_MASK => self.intc.read(paddr),
            mch::RICM | mch::DRD => self.mch.read(paddr),
            _ if Dmac::answers(paddr) => self.dmac.read(paddr),
            _ => {
                self.reports.note(Side::Ee, paddr, Access::Read, None, self.now);
                0
            }
        }
    }

    fn write_register(&mut self, paddr: u32, v: u32) {
        self.reports.touch(Side::Ee, paddr, Access::Write, Some(v), self.now);
        if let Some((n, reg)) = timer::decode(paddr) {
            self.sync_timer(n, self.now);
            let (h, v_) = (self.pcrtc.timing.h(self.now), self.pcrtc.timing.v(self.now));
            self.timers[n].write(reg, v, h, v_);
        } else if let Some(offset) = sif::ee_register(paddr) {
            self.iop.run_to(self.now);
            match self.iop.bus.sif_write_from_ee(offset, v) {
                Some(true) => {
                    self.iop.reset();
                    self.iop.held = true;
                    self.iop_resets += 1;
                }
                Some(false) => self.iop.held = self.iop.bus.rom.is_empty(),
                None => {}
            }
        } else {
            match paddr {
                intc::I_STAT | intc::I_MASK => self.intc.write(paddr, v),
                mch::RICM | mch::DRD => self.mch.write(paddr, v),
                _ if Dmac::answers(paddr) => self.dmac.write(paddr, v, self.now),
                _ => self.reports.note(Side::Ee, paddr, Access::Write, Some(v), self.now),
            }
        }
        self.reschedule();
    }
}

/// The EE's registers the units answer at (§2.2.1): the timers, the DMAC and the INTC; other registers read zero.
fn is_unit_register(paddr: u32) -> bool {
    paddr < 0x1000_2000 || (0x1000_8000..0x1000_F000).contains(&paddr) || (0x1000_F000..0x1000_F600).contains(&paddr)
}

/// The VU memories in the EE's map (EE User's Manual §2.1).
const VU_MEMORY: std::ops::Range<u32> = 0x1100_0000..0x1101_0000;

/// The EE's window onto IOP RAM, 1C000000h to 1C1FFFFFh [R ps2tek "Memory Map"].
const IOP_RAM: std::ops::Range<u32> = 0x1C00_0000..0x1C20_0000;

impl Bus for Board {
    fn read(&mut self, paddr: u32, bytes: usize) -> u128 {
        if VU_MEMORY.contains(&paddr) {
            self.run_vifs(self.now);
            return self.vpu.read(paddr, bytes);
        }
        if (vif::REGS[0]..vif::FIFO_END).contains(&paddr) {
            self.run_vifs(self.now);
            if paddr < vif::FIFO[0] {
                let w = self.vif[((paddr - vif::REGS[0]) >> 10) as usize].read(paddr & 0x3F0);
                return u128::from(w >> (8 * (paddr & 3))) & if bytes >= 4 { 0xFFFF_FFFF } else { (1 << (8 * bytes)) - 1 };
            }
            if paddr >= vif::FIFO[1] && bytes == 16 {
                let dma = self.dmac.ch[dmac::GIF].str();
                let mut gif = GifPort { gif: &mut self.gif, gs: &mut self.gs, dma };
                let q = self.vif[1].get(&mut gif);
                if self.vif[1].take_wake() {
                    self.dmac.wake(self.now);
                }
                self.reschedule();
                return q;
            }
            return 0;
        }
        if (ipu::CMD..ipu::REGS_END).contains(&paddr) || (ipu::OUT_FIFO..ipu::FIFO_END).contains(&paddr) {
            let v = self.ipu.read(paddr, bytes);
            self.sync_ipu(self.now);
            self.reschedule();
            return v;
        }
        if IOP_RAM.contains(&paddr) {
            self.reports.touch(Side::Ee, paddr, Access::Read, None, self.now);
            let a = (paddr - IOP_RAM.start) as usize;
            let mut v = [0u8; 16];
            v[..bytes].copy_from_slice(&self.iop.bus.ram[a..a + bytes]);
            return u128::from_le_bytes(v);
        }
        if (gif::REGS..gif::REGS_END).contains(&paddr) || (gif::FIFO..gif::FIFO_END).contains(&paddr) {
            self.run_gif(self.now);
            if paddr >= gif::FIFO {
                return 0;
            }
            let w = self.gif.read(paddr & 0xF0, self.now, self.gs.reversed());
            return u128::from(w >> (8 * (paddr & 3))) & if bytes >= 4 { 0xFFFF_FFFF } else { (1 << (8 * bytes)) - 1 };
        }
        if (gs::privileged::BASE..gs::privileged::END).contains(&paddr) {
            self.reports.touch(Side::Ee, paddr, Access::Read, None, self.now);
            self.run_gif(self.now);
            return u128::from(self.gs.read_privileged(paddr, bytes));
        }
        if (0x1000_0000..0x1001_0000).contains(&paddr) {
            if is_unit_register(paddr & !0xF) {
                let w = self.read_register(paddr & !0xF);
                return u128::from(w >> (8 * (paddr & 3))) & if bytes >= 4 { 0xFFFF_FFFF } else { (1 << (8 * bytes)) - 1 };
            }
            self.reports.note(Side::Ee, paddr, Access::Read, None, self.now);
            return 0;
        }
        match map::region(paddr) {
            Region::Rdram => self.traffic[0] += 1,
            Region::BootRom => self.traffic[2] += 1,
            _ => self.reports.note(Side::Ee, paddr, Access::Read, None, self.now),
        }
        self.map.read(paddr, bytes)
    }

    fn write(&mut self, paddr: u32, bytes: usize, value: u128) {
        if VU_MEMORY.contains(&paddr) {
            self.run_vifs(self.now);
            return self.vpu.write(paddr, bytes, value);
        }
        if (vif::REGS[0]..vif::FIFO_END).contains(&paddr) {
            self.run_vifs(self.now);
            if paddr < vif::FIFO[0] {
                if bytes >= 4 {
                    self.vif[((paddr - vif::REGS[0]) >> 10) as usize].write(paddr & 0x3F0, value as u32);
                }
            } else if bytes == 16 {
                self.vif[usize::from(paddr >= vif::FIFO[1])].put(value, false);
            }
            for v in &mut self.vif {
                if v.take_wake() {
                    self.dmac.wake(self.now);
                }
            }
            self.reschedule();
            return;
        }
        if (ipu::CMD..ipu::REGS_END).contains(&paddr) || (ipu::OUT_FIFO..ipu::FIFO_END).contains(&paddr) {
            self.ipu.write(paddr, bytes, value);
            self.sync_ipu(self.now);
            self.reschedule();
            return;
        }
        if IOP_RAM.contains(&paddr) {
            self.reports.touch(Side::Ee, paddr, Access::Write, None, self.now);
            let a = (paddr - IOP_RAM.start) as usize;
            self.iop.bus.ram[a..a + bytes].copy_from_slice(&value.to_le_bytes()[..bytes]);
            return;
        }
        if (gif::REGS..gif::REGS_END).contains(&paddr) || (gif::FIFO..gif::FIFO_END).contains(&paddr) {
            self.run_gif(self.now);
            if paddr >= gif::FIFO {
                if bytes == 16 {
                    self.gif.put(value, false, self.now);
                }
            } else if bytes >= 4 {
                self.gif.write(paddr & 0xF0, value as u32);
            }
            self.run_gif(self.now);
            self.reschedule();
            return;
        }
        if (gs::privileged::BASE..gs::privileged::END).contains(&paddr) {
            self.reports.touch(Side::Ee, paddr, Access::Write, Some(value as u32), self.now);
            self.run_gif(self.now);
            self.gs.write_privileged(paddr, bytes, value as u64);
            if self.pcrtc.written(self.now, &self.gs) {
                self.iop.bus.blank = self.pcrtc.timing;
            }
            self.pcrtc_next = None;
            self.run_gif(self.now);
            self.reschedule();
            return;
        }
        if (0x1000_0000..0x1001_0000).contains(&paddr) {
            if is_unit_register(paddr & !0xF) && bytes >= 4 {
                self.write_register(paddr & !0xF, value as u32);
            } else {
                self.reports.note(Side::Ee, paddr, Access::Write, Some(value as u32), self.now);
            }
            return;
        }
        if map::region(paddr) != Region::Rdram {
            self.reports.note(Side::Ee, paddr, Access::Write, Some(value as u32), self.now);
        } else {
            self.traffic[1] += 1;
        }
        self.map.write(paddr, bytes, value)
    }

    fn error(&mut self, paddr: u32) -> bool {
        self.map.error(paddr) || (gs::privileged::END..gs::privileged::BUS_ERROR_END).contains(&paddr)
    }

    fn vpu(&mut self) -> Option<&mut Vpu> {
        self.run_vifs(self.now);
        Some(&mut self.vpu)
    }
}

/// The core and the board on one clock.
pub struct System {
    pub ee: Ee,
    pub board: Board,
}

impl System {
    /// A program's machine entered at `pc` as `Ee::new` enters it, NTSC timing.
    pub fn new(pc: u32) -> System {
        System { ee: Ee::new(pc), board: Board::new(clock::NTSC) }
    }

    /// One instruction, then the board brought to the core's time and the core's lines set from it.
    pub fn step(&mut self) -> Result<(), Halt> {
        let r = self.ee.step(&mut self.board);
        let t = self.ee.timing.cycles();
        self.board.run_to(t, &mut self.ee.spram[..]);
        let (i0, i1, c0) = self.board.lines();
        self.ee.int0 = i0;
        self.ee.int1 = i1;
        self.ee.cpcond0 = c0;
        r
    }

    /// Runs until `done` returns true or `limit` instructions have run; the instructions run.
    pub fn run(&mut self, limit: u64, done: &mut dyn FnMut(&mut System) -> bool) -> Result<u64, Halt> {
        for n in 0..limit {
            if done(self) {
                return Ok(n);
            }
            self.step()?;
        }
        Ok(limit)
    }

    /// The bytes at `paddr` as the core sees them (a valid data cache line over memory).
    pub fn peek(&mut self, paddr: u32, bytes: usize) -> u128 {
        self.ee.peek(&mut self.board, paddr, bytes)
    }
}

/// The core and the board as `SPH2` holds them, every block in the plan's order (Sapphire_Plan.md §4.7,
/// Sapphire_Native.md §38.2): the EE with COP0, COP1, its issue model, the TLB and the caches; RDRAM; the scratchpad;
/// the VUs with their memories and micro modes; VIF0, VIF1, the GIF; the GS with its 4 MiB; the PCRTC; the DMAC, the
/// INTC and the timers; the IOP with its RAM, cache, scratchpad, DMAC and timers; the SIF; the drive; SIO2; the SPU2
/// with its RAM; the IPU; the memory controller; and the scheduler. The boot ROM is firmware and never in a state; the
/// stand-ins, the reports and the traffic counts keep nothing a program sees; the PCRTC's next event is a cache.
impl System {
    pub fn write_state(&self, w: &mut emusen_native::StateWriter) {
        let b = &self.board;
        w.structure("Ee", &self.ee);
        w.bytes("Rdram", &b.map.rdram);
        w.bytes("Scratchpad", &self.ee.spram[..]);
        w.structure("Vpu", &b.vpu);
        w.structure("Vif0", &b.vif[0]);
        w.structure("Vif1", &b.vif[1]);
        w.structure("Gif", &b.gif);
        w.structure("Gs", &b.gs);
        w.structure("Pcrtc", &b.pcrtc);
        w.structure("Dmac", &b.dmac);
        w.u32s("Intc", &[b.intc.stat, b.intc.mask]);
        w.structures("Timers", &b.timers);
        w.group("Iop", |w| b.iop.write_state(w));
        w.structure("Sif", &b.iop.bus.sif);
        w.structure("Cdvd", &b.iop.bus.cdvd);
        w.structure("Sio2", &b.iop.bus.sio2);
        w.structure("Spu2", &b.iop.bus.spu2);
        w.structure("Ipu", &b.ipu);
        w.structure("Mch", &b.mch);
        w.u64s("Scheduler", &[b.now, b.next]);
        crate::state::opt_u64(w, "EdgesTaken", b.edges_taken);
    }

    pub fn read_state(&mut self, r: &mut emusen_native::StateReader) -> Result<(), emusen_native::Truncated> {
        use emusen_native::State;
        let b = &mut self.board;
        self.ee.read_state(r)?;
        r.bytes(&mut b.map.rdram)?;
        r.bytes(&mut self.ee.spram[..])?;
        b.vpu.read_state(r)?;
        r.structures(&mut b.vif)?;
        b.gif.read_state(r)?;
        b.gs.read_state(r)?;
        b.pcrtc.read_state(r)?;
        b.dmac.read_state(r)?;
        let mut i = [0u32; 2];
        r.u32s(&mut i)?;
        (b.intc.stat, b.intc.mask) = (i[0], i[1]);
        r.structures(&mut b.timers)?;
        b.iop.read_state(r)?;
        b.iop.bus.sif.read_state(r)?;
        b.iop.bus.cdvd.read_state(r)?;
        b.iop.bus.sio2.read_state(r)?;
        b.iop.bus.spu2.read_state(r)?;
        b.ipu.read_state(r)?;
        b.mch.read_state(r)?;
        let mut s = [0u64; 2];
        r.u64s(&mut s)?;
        [b.now, b.next] = s;
        b.edges_taken = crate::state::read_opt_u64(r)?;
        b.pcrtc_next = None;
        Ok(())
    }

    /// Whether every queue is within its slots in the state, so that it can be saved at this instant.
    pub fn fits(&self) -> bool {
        let b = &self.board;
        let io = &b.iop.bus;
        b.vpu.fits() && b.vif.iter().all(Vif::fits) && b.gif.fits() && b.gs.fits() && io.sif.fits() && io.cdvd.fits() && io.sio2.fits() && b.ipu.fits()
    }
}
