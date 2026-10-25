//! The EE's side of the console around its core (Sapphire_Native.md §20.1): the physical map, the DMAC, the INTC, the
//! four timers and the blanking stand-in, on one clock of EE cycles (`clock`), and `System`, which runs the core and
//! brings these units to the core's time after each instruction. A unit is brought forward only at the events it
//! schedules (a DMA unit's end, a timer's flag, a blanking edge) or when the core reads or writes it; nothing is
//! polled cycle by cycle. The core reaches the registers through `Bus`; the units drive its Int0, Int1 and CPCOND0.

use std::collections::VecDeque;

use crate::blank::Blank;
use crate::clock::{self, Standard};
use crate::dmac::{self, Dmac, Peripherals};
use crate::ee::map::{self, Map, Region};
use crate::ee::{Bus, Ee, Halt};
use crate::intc::{self, Intc};
use crate::iop::Iop;
use crate::timer::{self, Timer};
use crate::vu::Vpu;

/// A stand-in for a peripheral the DMAC reaches before its stage exists (VIF0, VIF1, GIF, IPU, SIF): it takes every
/// qword offered, keeping them when `keep` is set, and gives the qwords queued in `give`, holding its channel when it
/// has none.
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
    pub blank: Blank,
    pub ports: StandIns,
    /// VU0 and VU1, which the core reaches through COP2 and the map at 11000000h (from S12).
    pub vpu: Vpu,
    /// The IOP, a unit of its own with its bus and units, brought to the board's time (Sapphire_Native.md §22).
    pub iop: Iop,
    /// The time every unit has been brought to, in EE cycles.
    pub now: u64,
    next: u64,
}

impl Board {
    pub fn new(standard: Standard) -> Board {
        let mut b = Board {
            map: Map::new(),
            dmac: Dmac::default(),
            intc: Intc::default(),
            timers: Default::default(),
            blank: Blank::new(standard),
            ports: StandIns::default(),
            vpu: Vpu::new(),
            iop: Iop::new(Blank::new(standard)),
            now: 0,
            next: 0,
        };
        b.reschedule();
        b
    }

    fn wants_h(&self) -> bool {
        self.timers.iter().any(Timer::wants_h)
    }

    /// The next time any unit needs attention.
    fn reschedule(&mut self) {
        let mut next = self.blank.next_edge(self.now, self.wants_h()).min(self.dmac.next_event());
        for t in &self.timers {
            next = next.min(t.next_event());
        }
        self.next = next.max(self.now);
    }

    fn sync_timer(&mut self, n: usize, t: u64) {
        if self.timers[n].sync(t) != 0 {
            self.intc.raise(intc::source::TIMER0 + n as u32);
        }
    }

    /// Brings every unit to time `t`, running the events due on the way; the scratchpad is the core's.
    pub fn run_to(&mut self, t: u64, spram: &mut [u8]) {
        while self.next <= t {
            let at = self.next;
            self.now = at;
            for n in 0..4 {
                self.sync_timer(n, at);
            }
            let e = self.blank.edges(at);
            if e.h_rise || e.h_fall || e.v_rise || e.v_fall {
                let (h, v) = (self.blank.h(at), self.blank.v(at));
                for n in 0..4 {
                    if self.timers[n].edges(e.h_rise, e.h_fall, e.v_rise, e.v_fall, h, v) != 0 {
                        self.intc.raise(intc::source::TIMER0 + n as u32);
                    }
                }
                if e.v_rise {
                    self.intc.raise(intc::source::VB_ON);
                }
                if e.v_fall {
                    self.intc.raise(intc::source::VB_OFF);
                }
            }
            if self.dmac.next_event() <= at {
                self.dmac.step(at, &mut self.map, spram, &mut self.ports);
            }
            self.reschedule();
            debug_assert!(self.next > at, "an event at {at} asked for its own time again");
        }
        self.now = self.now.max(t);
        self.vpu.run_to(t);
        self.iop.run_to(self.now);
    }

    /// INT0, INT1 and CPCOND0 as the core sees them now: levels, sampled before each instruction (Sapphire_Disputes.md D-39).
    pub fn lines(&self) -> (bool, bool, bool) {
        (self.intc.int0(), self.dmac.int1(), self.dmac.cpcond0())
    }

    fn read_register(&mut self, paddr: u32) -> u32 {
        if let Some((n, reg)) = timer::decode(paddr) {
            self.sync_timer(n, self.now);
            return self.timers[n].read(reg);
        }
        match paddr {
            intc::I_STAT | intc::I_MASK => self.intc.read(paddr),
            _ => self.dmac.read(paddr),
        }
    }

    fn write_register(&mut self, paddr: u32, v: u32) {
        if let Some((n, reg)) = timer::decode(paddr) {
            self.sync_timer(n, self.now);
            let (h, v_) = (self.blank.h(self.now), self.blank.v(self.now));
            self.timers[n].write(reg, v, h, v_);
        } else {
            match paddr {
                intc::I_STAT | intc::I_MASK => self.intc.write(paddr, v),
                _ => self.dmac.write(paddr, v, self.now),
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
            return self.vpu.read(paddr, bytes);
        }
        if IOP_RAM.contains(&paddr) {
            let a = (paddr - IOP_RAM.start) as usize;
            let mut v = [0u8; 16];
            v[..bytes].copy_from_slice(&self.iop.bus.ram[a..a + bytes]);
            return u128::from_le_bytes(v);
        }
        if (0x1000_0000..0x1001_0000).contains(&paddr) {
            if is_unit_register(paddr & !0xF) {
                let w = self.read_register(paddr & !0xF);
                return u128::from(w >> (8 * (paddr & 3))) & if bytes >= 4 { 0xFFFF_FFFF } else { (1 << (8 * bytes)) - 1 };
            }
            return 0;
        }
        self.map.read(paddr, bytes)
    }

    fn write(&mut self, paddr: u32, bytes: usize, value: u128) {
        if VU_MEMORY.contains(&paddr) {
            return self.vpu.write(paddr, bytes, value);
        }
        if IOP_RAM.contains(&paddr) {
            let a = (paddr - IOP_RAM.start) as usize;
            self.iop.bus.ram[a..a + bytes].copy_from_slice(&value.to_le_bytes()[..bytes]);
            return;
        }
        if (0x1000_0000..0x1001_0000).contains(&paddr) {
            if is_unit_register(paddr & !0xF) && bytes >= 4 {
                self.write_register(paddr & !0xF, value as u32);
            }
            return;
        }
        self.map.write(paddr, bytes, value)
    }

    fn error(&mut self, paddr: u32) -> bool {
        self.map.error(paddr)
    }

    fn vpu(&mut self) -> Option<&mut Vpu> {
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
