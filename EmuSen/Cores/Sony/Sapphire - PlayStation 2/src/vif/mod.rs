//! VIF0 and VIF1 (EE User's Manual chapter 6; ps2tek "Vector Interface (VIF)"): each takes a stream of 32-bit words
//! through its FIFO, from the DMAC's channel or from the EE's stores, and runs the VIFcodes in it: the registers it
//! sets, MPG into micro memory, UNPACK into data memory (`unpack`), the MSCAL family starting the VU through
//! `Vpu::start` with VIF1's double buffering and the TOP and ITOP that XTOP and XITOP read, the FLUSH family's waits,
//! DIRECT and DIRECTHL to PATH2 and MSKPATH3, which end at `Gif` (the GIF implements it at S15), the i bit's
//! interrupt and stall, FBRST and the error masks; and VIF1 carries the GS's local-to-host data to the DMAC when FDR
//! reverses it. A VIF runs on its own clock and is brought forward by `run_to` as the VU is: at its events while the
//! DMAC waits on its FIFO, before the DMAC looks at the FIFO, and whenever the EE reaches it. Sapphire_Native.md §28.

pub mod unpack;

use std::collections::VecDeque;

use crate::clock::BUS;
use crate::vu::{Vpu, pipe};
use unpack::{Fill, Next, Unpack};

/// The register blocks and the FIFOs in the EE's map (§2.2.1's table): VIF0's FIFO is mirrored to 10004FF0h.
pub const REGS: [u32; 2] = [0x1000_3800, 0x1000_3C00];
pub const FIFO: [u32; 2] = [0x1000_4000, 0x1000_5000];
pub const FIFO_END: u32 = 0x1000_6000;

/// Register offsets in a block (§2.2.1, §6.5).
pub mod reg {
    pub const STAT: u32 = 0x00;
    pub const FBRST: u32 = 0x10;
    pub const ERR: u32 = 0x20;
    pub const MARK: u32 = 0x30;
    pub const CYCLE: u32 = 0x40;
    pub const MODE: u32 = 0x50;
    pub const NUM: u32 = 0x60;
    pub const MASK: u32 = 0x70;
    pub const CODE: u32 = 0x80;
    pub const ITOPS: u32 = 0x90;
    pub const BASE: u32 = 0xA0;
    pub const OFST: u32 = 0xB0;
    pub const TOPS: u32 = 0xC0;
    pub const ITOP: u32 = 0xD0;
    pub const TOP: u32 = 0xE0;
    pub const R0: u32 = 0x100;
    pub const C0: u32 = 0x140;
}

/// STAT's fields (§6.5, VIFn_STAT).
pub mod stat {
    pub const VPS: u32 = 3;
    pub const VEW: u32 = 1 << 2;
    pub const VGW: u32 = 1 << 3;
    pub const MRK: u32 = 1 << 6;
    pub const DBF: u32 = 1 << 7;
    pub const VSS: u32 = 1 << 8;
    pub const VFS: u32 = 1 << 9;
    pub const VIS: u32 = 1 << 10;
    pub const INT: u32 = 1 << 11;
    pub const ER0: u32 = 1 << 12;
    pub const ER1: u32 = 1 << 13;
    pub const FDR: u32 = 1 << 23;
    pub const FQC_SHIFT: u32 = 24;
    /// The bits STC clears; any of them set stalls the VIF.
    pub const STALLS: u32 = VSS | VFS | VIS | INT | ER0 | ER1;
}

/// FBRST's bits (§6.5).
pub mod fbrst {
    pub const RST: u32 = 1;
    pub const FBK: u32 = 1 << 1;
    pub const STP: u32 = 1 << 2;
    pub const STC: u32 = 1 << 3;
}

/// ERR's bits (§6.5).
pub mod err {
    pub const MII: u32 = 1;
    pub const ME0: u32 = 1 << 1;
    pub const ME1: u32 = 1 << 2;
}

/// The FIFO's depth in qwords: STAT.FQC's full value, 8 on VIF0 and 16 on VIF1.
pub const DEPTH: [usize; 2] = [8, 16];

/// VIF1 as it reaches the GIF (§6.4's DIRECT, DIRECTHL, MSKPATH3, FLUSH, FLUSHA and MSCALF; chapter 5's reversed
/// channel 1): S15's GIF implements it, `Path2StandIn` until then. Times are EE cycles.
pub trait Gif {
    /// A qword of DIRECT's data (DIRECTHL's with `hl`, which waits out PATH3's IMAGE data) offered to PATH2 at `at`:
    /// the cycle it is taken, or none while PATH2 cannot have the GIF.
    fn path2(&mut self, q: u128, hl: bool, at: u64) -> Option<u64>;
    /// The cycle PATH1 and PATH2 are idle from, `at` or later, and with `path3` also PATH3 asks for nothing; none
    /// while that cannot yet be known.
    fn idle_at(&mut self, path3: bool, at: u64) -> Option<u64>;
    /// MSKPATH3's mask, from the next PATH3 data block.
    fn mask_path3(&mut self, masked: bool, at: u64);
    /// The next qword of the GS's local-to-host data while the host interface is reversed (GS User's Manual §4.2.2).
    fn host_read(&mut self) -> Option<u128>;
}

/// PATH2 until the GIF exists: every qword taken at once and counted (kept with `keep`), the paths always idle, the
/// PATH3 mask recorded.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Path2StandIn {
    pub keep: bool,
    pub taken: Vec<(u128, bool)>,
    pub count: u64,
    pub masked: bool,
}

impl Path2StandIn {
    pub fn take(&mut self, q: u128, hl: bool) {
        self.count += 1;
        if self.keep {
            self.taken.push((q, hl));
        }
    }
}

/// What a VIF reaches while it runs: its VU through the VPU, and the GIF.
pub struct Reach<'a> {
    pub vpu: &'a mut Vpu,
    pub gif: &'a mut dyn Gif,
}

/// What the VIF waits for before a command goes on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Then {
    /// FLUSHE, FLUSH, FLUSHA: nothing more.
    Done,
    /// MSCAL, MSCALF, MSCNT: the VU started at this doubleword (none: TPC).
    Start(Option<u16>),
    /// MPG: its instructions loaded.
    Load,
}

/// The command in progress.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Cmd {
    Idle,
    /// The VU's end (and with `gif`, the paths' idle, `path3` FLUSHA's), then `then`.
    Wait { gif: bool, path3: bool, then: Then },
    /// STMASK, STROW, STCOL: the words still to take into `to` (0 MASK, 1 Row, 2 Col).
    Words { to: u8, got: u8, need: u8, w: [u32; 4] },
    Mpg { addr: u32, left: u32, low: Option<u32> },
    Direct { hl: bool, left: u32, got: u8, w: [u32; 4] },
    Unpack(Unpack),
}

/// One VIF. Registers are kept as the EE reads them.
pub struct Vif {
    pub unit: usize,
    stat: u32,
    pub err: u32,
    pub mark: u32,
    pub cycle: u32,
    pub fill: Fill,
    num: u32,
    pub code: u32,
    pub itops: u32,
    pub itop: u32,
    pub base: u32,
    pub ofst: u32,
    pub tops: u32,
    pub top: u32,
    fifo: VecDeque<(u128, bool)>,
    /// The qword being taken apart, and the next of its words.
    cur: [u32; 4],
    cur_at: usize,
    cmd: Cmd,
    /// A VIFcode fetched and held by the i bit's stall, run when the stall is cancelled.
    held: Option<u32>,
    /// The command in progress carries the i bit (and ERR.MII is clear).
    i_bit: bool,
    /// The last command carried the i bit and was not MARK: the next VIFcode stalls (§6.3.2).
    stall_next: bool,
    /// FBRST.STP written: stop at the next VIFcode.
    stop_next: bool,
    /// The VIF's clock: the cycle of its next step.
    time: u64,
    /// The DMAC found the FIFO full (or, reversed, empty) and holds the channel until woken.
    dmac_waiting: bool,
    wake: bool,
    irq: bool,
    /// Qwords the FIFO has taken, from the DMAC or the EE.
    pub received: u64,
}

impl Vif {
    pub fn new(unit: usize) -> Vif {
        Vif {
            unit,
            stat: 0,
            err: 0,
            mark: 0,
            cycle: 0,
            fill: Fill::default(),
            num: 0,
            code: 0,
            itops: 0,
            itop: 0,
            base: 0,
            ofst: 0,
            tops: 0,
            top: 0,
            fifo: VecDeque::with_capacity(DEPTH[1]),
            cur: [0; 4],
            cur_at: 4,
            cmd: Cmd::Idle,
            held: None,
            i_bit: false,
            stall_next: false,
            stop_next: false,
            time: 0,
            dmac_waiting: false,
            wake: false,
            irq: false,
            received: 0,
        }
    }

    fn reversed(&self) -> bool {
        self.stat & stat::FDR != 0
    }

    fn stalled(&self) -> bool {
        self.stat & stat::STALLS & !stat::INT != 0
    }

    fn has_word(&self) -> bool {
        self.cur_at < 4 || !self.fifo.is_empty()
    }

    /// The next word of the stream: of the qword in hand, or of the FIFO's next qword (a DMAtag sent by TTE gives only
    /// its upper 64 bits, §6.3.1's figure). Taking a qword frees a slot, which wakes a DMAC that waits for one.
    fn word(&mut self) -> Option<u32> {
        if self.cur_at == 4 {
            let (q, tag) = self.fifo.pop_front()?;
            self.cur = [q as u32, (q >> 32) as u32, (q >> 64) as u32, (q >> 96) as u32];
            self.cur_at = if tag { 2 } else { 0 };
            if self.dmac_waiting {
                self.dmac_waiting = false;
                self.wake = true;
            }
        }
        let w = self.cur[self.cur_at];
        self.cur_at += 1;
        Some(w)
    }

    /// The `i`th word of the stream past the next one `word` gives, without taking it; 0 where none has arrived.
    fn peek(&self, i: usize) -> u32 {
        let left = 4 - self.cur_at;
        if i < left {
            return self.cur[self.cur_at + i];
        }
        let mut i = i - left;
        for &(q, tag) in &self.fifo {
            let from = if tag { 2 } else { 0 };
            if i < 4 - from {
                return (q >> (32 * (from + i))) as u32;
            }
            i -= 4 - from;
        }
        0
    }

    /// The memory size of this VIF's VU in qwords (VU User's Manual §1.1.5), where UNPACK's addresses wrap
    /// (D-181, the console's `stcycl` text).
    fn data_qwords(&self) -> u32 {
        if self.unit == 0 { 256 } else { 1024 }
    }

    /// The interrupt raised since the last call (INT_VIF0 or INT_VIF1).
    pub fn take_irq(&mut self) -> bool {
        std::mem::take(&mut self.irq)
    }

    /// Whether the DMAC is to be woken: a FIFO slot freed, or reversed data arrived, while it waited.
    pub fn take_wake(&mut self) -> bool {
        std::mem::take(&mut self.wake)
    }

    /// The cycle the VIF next needs the board: only while the DMAC waits on its FIFO and the VIF can take from it.
    pub fn next_event(&self) -> u64 {
        let runs = !self.stalled() && !self.reversed() && self.has_word() && !matches!(self.cmd, Cmd::Wait { .. }) && self.held.is_none();
        if self.dmac_waiting && runs { self.time } else { u64::MAX }
    }

    /// Runs every step that begins by cycle `t`.
    pub fn run_to(&mut self, t: u64, r: &mut Reach) {
        if self.reversed() {
            self.time = self.time.max(t);
            let had = self.fifo.len();
            while self.fifo.len() < DEPTH[1] {
                let Some(q) = r.gif.host_read() else { break };
                self.fifo.push_back((q, false));
            }
            if self.fifo.len() > had && self.dmac_waiting {
                self.dmac_waiting = false;
                self.wake = true;
            }
            return;
        }
        while self.time <= t {
            match self.step(t, r) {
                Some(cost) => self.time += cost,
                None => {
                    if !matches!(self.cmd, Cmd::Wait { .. }) {
                        self.time = self.time.max(t);
                    }
                    return;
                }
            }
        }
    }

    /// The command in progress finished: the i bit's interrupt, and its stall of the next VIFcode but after MARK.
    fn complete(&mut self) {
        let cmd = self.code >> 24 & 0x7F;
        self.cmd = Cmd::Idle;
        if self.i_bit {
            self.i_bit = false;
            self.stat |= stat::INT;
            self.irq = true;
            self.stall_next = cmd != 0x07;
        }
    }

    /// One step at `self.time`: its cost in EE cycles, zero for a step that takes no time, none when blocked. `t` is
    /// how far the caller has brought the machine, for the waits on the VU.
    fn step(&mut self, t: u64, r: &mut Reach) -> Option<u64> {
        if self.stalled() {
            return None;
        }
        match std::mem::replace(&mut self.cmd, Cmd::Idle) {
            Cmd::Idle => self.fetch(r),
            Cmd::Wait { gif, path3, then } => self.wait(t, gif, path3, then, r),
            Cmd::Words { to, mut got, need, mut w } => {
                while got < need {
                    let Some(x) = self.word() else {
                        self.cmd = Cmd::Words { to, got, need, w };
                        return None;
                    };
                    w[got as usize] = x;
                    got += 1;
                }
                match to {
                    0 => self.fill.mask = w[0],
                    1 => self.fill.row = w,
                    _ => self.fill.col = w,
                }
                self.complete();
                Some(0)
            }
            Cmd::Mpg { addr, left, low } => {
                let Some(lo) = low.or_else(|| self.word()) else {
                    self.cmd = Cmd::Mpg { addr, left, low };
                    return None;
                };
                let Some(hi) = self.word() else {
                    self.cmd = Cmd::Mpg { addr, left, low: Some(lo) };
                    return None;
                };
                let micro = &mut if self.unit == 0 { &mut r.vpu.units.vu0 } else { &mut r.vpu.units.vu1 }.micro;
                let at = (addr as usize * 8) % micro.len();
                micro[at..at + 4].copy_from_slice(&lo.to_le_bytes());
                micro[at + 4..at + 8].copy_from_slice(&hi.to_le_bytes());
                self.num = left - 1;
                if left == 1 { self.complete() } else { self.cmd = Cmd::Mpg { addr: addr + 1, left: left - 1, low: None } }
                Some(BUS)
            }
            Cmd::Direct { hl, left, mut got, mut w } => {
                while got < 4 {
                    let Some(x) = self.word() else {
                        self.cmd = Cmd::Direct { hl, left, got, w };
                        return None;
                    };
                    w[got as usize] = x;
                    got += 1;
                }
                let q = w.iter().rev().fold(0u128, |a, &x| a << 32 | u128::from(x));
                let Some(taken) = r.gif.path2(q, hl, self.time) else {
                    self.cmd = Cmd::Direct { hl, left, got, w };
                    self.stat |= stat::VGW;
                    return None;
                };
                self.stat &= !stat::VGW;
                self.num = (left - 1) & 0xFFFF;
                if left == 1 { self.complete() } else { self.cmd = Cmd::Direct { hl, left: left - 1, got: 0, w } }
                Some(BUS + taken.saturating_sub(self.time))
            }
            Cmd::Unpack(mut u) => loop {
                let mut fill = self.fill;
                let next = u.next(&mut fill, &|i| self.peek(i));
                self.fill = fill;
                match next {
                    Next::Word => {
                        let Some(x) = self.word() else {
                            self.cmd = Cmd::Unpack(u);
                            return None;
                        };
                        u.take(x);
                    }
                    Next::Write(at, fields, enable) => {
                        self.store(r, at, fields, enable);
                        self.num = self.num_bits(u.num - u.done);
                        if u.finished() { self.complete() } else { self.cmd = Cmd::Unpack(u) }
                        return Some(BUS);
                    }
                }
            },
        }
    }

    /// NUM keeps 8 bits on VIF0, as §6.5 draws it, and more on VIF1 (D-185).
    fn num_bits(&self, n: u32) -> u32 {
        if self.unit == 0 { n & 0xFF } else { n & 0xFFFF }
    }

    /// One UNPACK write into data memory, the fields `enable` names, with the VU brought to the write's cycle first.
    fn store(&mut self, r: &mut Reach, at: u32, fields: [u32; 4], enable: u32) {
        r.vpu.run_unit(self.unit, self.time);
        let a = (at % self.data_qwords()) as usize;
        let vu = if self.unit == 0 { &mut r.vpu.units.vu0 } else { &mut r.vpu.units.vu1 };
        let old = vu.mem_q(a);
        let mut q = 0u128;
        for (f, &v) in fields.iter().enumerate().rev() {
            let keep = if enable >> f & 1 == 1 { v } else { (old >> (32 * f)) as u32 };
            q = q << 32 | u128::from(keep);
        }
        vu.set_mem_q(a, q);
    }

    /// Fetches and decodes the next VIFcode (§6.3.2's CMD table).
    fn fetch(&mut self, r: &mut Reach) -> Option<u64> {
        if self.stop_next {
            self.stop_next = false;
            self.stat |= stat::VSS;
            return None;
        }
        let w = match self.held.take() {
            Some(w) => w,
            None => {
                let w = self.word()?;
                self.code = w;
                if self.stall_next {
                    self.stall_next = false;
                    self.held = Some(w);
                    self.stat |= stat::VIS;
                    return None;
                }
                w
            }
        };
        self.i_bit = w >> 31 == 1 && self.err & err::MII == 0;
        let cmd = w >> 24 & 0x7F;
        let imm = w & 0xFFFF;
        let num = w >> 16 & 0xFF;
        let vif1 = self.unit == 1;
        match cmd {
            0x00 => {}
            0x01 => self.cycle = imm,
            0x02 if vif1 => {
                self.ofst = imm & 0x3FF;
                self.stat &= !stat::DBF;
                self.tops = self.base;
            }
            0x03 if vif1 => self.base = imm & 0x3FF,
            0x04 => self.itops = imm & 0x3FF,
            0x05 => self.fill.mode = imm & 3,
            0x06 if vif1 => r.gif.mask_path3(imm & 0x8000 != 0, self.time),
            0x07 => {
                self.mark = imm;
                self.stat |= stat::MRK;
            }
            0x10 => self.cmd = Cmd::Wait { gif: false, path3: false, then: Then::Done },
            0x11 if vif1 => self.cmd = Cmd::Wait { gif: true, path3: false, then: Then::Done },
            0x13 if vif1 => self.cmd = Cmd::Wait { gif: true, path3: true, then: Then::Done },
            0x14 => self.cmd = Cmd::Wait { gif: false, path3: false, then: Then::Start(Some(imm as u16)) },
            0x15 => self.cmd = Cmd::Wait { gif: true, path3: false, then: Then::Start(Some(imm as u16)) },
            0x17 => self.cmd = Cmd::Wait { gif: false, path3: false, then: Then::Start(None) },
            0x20 => self.cmd = Cmd::Words { to: 0, got: 0, need: 1, w: [0; 4] },
            0x30 => self.cmd = Cmd::Words { to: 1, got: 0, need: 4, w: [0; 4] },
            0x31 => self.cmd = Cmd::Words { to: 2, got: 0, need: 4, w: [0; 4] },
            0x4A => {
                self.num = num;
                self.cmd = Cmd::Wait { gif: false, path3: false, then: Then::Load };
            }
            0x50 | 0x51 if vif1 => {
                let left = if imm == 0 { 0x1_0000 } else { imm };
                self.num = imm;
                self.cmd = Cmd::Direct { hl: cmd == 0x51, left, got: 0, w: [0; 4] };
            }
            0x60..=0x7F if unpack::Format::of(cmd).is_some() => {
                let tops = vif1.then_some(self.tops);
                let u = Unpack::new(w, self.cycle, tops).expect("a defined format");
                self.num = self.num_bits(u.num);
                self.cmd = Cmd::Unpack(u);
            }
            _ => {
                if self.err & err::ME1 == 0 {
                    self.stat |= stat::ER1;
                    self.irq = true;
                    self.i_bit = false;
                    return None;
                }
            }
        }
        if self.cmd == Cmd::Idle {
            self.complete();
        }
        Some(BUS)
    }

    /// The FLUSH family's and the MSCAL family's waits, and MPG's: the VU ready, neither running nor stopped (VU User's
    /// Manual §1.3.3: "From the point of view of the VIF, there is no difference between the Run state and the Stop
    /// state"), then the paths idle where asked.
    fn wait(&mut self, t: u64, gif: bool, path3: bool, then: Then, r: &mut Reach) -> Option<u64> {
        let unit = self.unit;
        if !r.vpu.ready(unit, self.time) && !r.vpu.ready(unit, t.max(self.time)) {
            self.cmd = Cmd::Wait { gif, path3, then };
            self.stat |= stat::VEW;
            return None;
        }
        self.time = self.time.max(r.vpu.last(unit).map_or(0, |e| e + 1));
        self.stat &= !stat::VEW;
        if gif {
            match r.gif.idle_at(path3, self.time) {
                Some(at) => self.time = self.time.max(at),
                None => {
                    self.cmd = Cmd::Wait { gif, path3, then };
                    self.stat |= stat::VGW;
                    return None;
                }
            }
            self.stat &= !stat::VGW;
        }
        match then {
            Then::Done => self.complete(),
            Then::Load => {
                let left = if self.num == 0 { 256 } else { self.num };
                self.cmd = Cmd::Mpg { addr: self.code & 0xFFFF, left, low: None };
                self.num = self.num_bits(left);
            }
            Then::Start(addr) => {
                let vu = if unit == 0 { &r.vpu.units.vu0 } else { &r.vpu.units.vu1 };
                let addr = addr.unwrap_or(vu.tpc);
                self.itop = self.itops;
                if unit == 1 {
                    self.top = self.tops;
                    self.stat ^= stat::DBF;
                    let other = if self.stat & stat::DBF != 0 { self.base + self.ofst } else { self.base };
                    self.tops = other & 0x3FF;
                    r.vpu.units.link.top[1] = self.top as u16;
                }
                r.vpu.units.link.itop[unit] = self.itop as u16;
                r.vpu.start(unit, addr, self.time + pipe::START);
                self.complete();
            }
        }
        Some(0)
    }

    /// STAT as the EE reads it: VPS from the command in progress (D-186), FQC from the FIFO.
    pub fn status(&self) -> u32 {
        let vps = match &self.cmd {
            Cmd::Idle if self.held.is_some() || self.stat & stat::ER1 != 0 => 2,
            Cmd::Idle => 0,
            Cmd::Wait { .. } => 2,
            _ if self.has_word() => 3,
            _ => 1,
        };
        let fqc = (self.fifo.len().min(DEPTH[self.unit]) as u32) << stat::FQC_SHIFT;
        self.stat & !stat::VPS | vps | fqc
    }

    /// A register read (§6.5); reserved offsets and VIF1's own registers on VIF0 read 0.
    pub fn read(&self, off: u32) -> u32 {
        let vif1 = self.unit == 1;
        match off {
            reg::STAT => self.status(),
            reg::ERR => self.err,
            reg::MARK => self.mark,
            reg::CYCLE => self.cycle,
            reg::MODE => self.fill.mode,
            reg::NUM => self.num,
            reg::MASK => self.fill.mask,
            reg::CODE => self.code,
            reg::ITOPS => self.itops,
            reg::ITOP => self.itop,
            reg::BASE if vif1 => self.base,
            reg::OFST if vif1 => self.ofst,
            reg::TOPS if vif1 => self.tops,
            reg::TOP if vif1 => self.top,
            0x100..=0x130 if off & 0xF == 0 => self.fill.row[(off as usize - 0x100) >> 4],
            0x140..=0x170 if off & 0xF == 0 => self.fill.col[(off as usize - 0x140) >> 4],
            _ => 0,
        }
    }

    /// A register write (§6.5): MARK, which clears MRK; FBRST; ERR; and VIF1_STAT's FDR. The rest are read-only.
    pub fn write(&mut self, off: u32, v: u32) {
        match off {
            reg::MARK => {
                self.mark = v & 0xFFFF;
                self.stat &= !stat::MRK;
            }
            reg::ERR => self.err = v & 7,
            reg::STAT if self.unit == 1 => self.stat = self.stat & !stat::FDR | v & stat::FDR,
            reg::FBRST => self.fbrst(v),
            _ => {}
        }
    }

    /// FBRST (§6.5): RST empties the FIFO, drops the VIFcode in progress and clears STAT, NUM, CODE and ERR, keeping the
    /// registers a program sets (D-187); FBK stalls at once, STP after the VIFcode in process, STC cancels a stall and
    /// clears its flags.
    fn fbrst(&mut self, v: u32) {
        if v & fbrst::RST != 0 {
            self.fifo.clear();
            self.cur_at = 4;
            self.cmd = Cmd::Idle;
            self.held = None;
            self.i_bit = false;
            self.stall_next = false;
            self.stop_next = false;
            self.stat &= stat::DBF | stat::FDR;
            self.num = 0;
            self.code = 0;
            self.err = 0;
            self.wake = true;
        }
        if v & fbrst::FBK != 0 {
            self.stat |= stat::VFS;
        }
        if v & fbrst::STP != 0 {
            if self.cmd == Cmd::Idle && self.held.is_none() {
                self.stat |= stat::VSS;
            } else {
                self.stop_next = true;
            }
        }
        if v & fbrst::STC != 0 {
            self.stat &= !stat::STALLS;
            self.stop_next = false;
        }
    }

    /// The DMAC's request on this VIF's channel (`dmac::Peripherals::request`): room in the FIFO when it runs forward,
    /// data when it is reversed, nothing when the DMAC's direction is not the FIFO's; refilled from the GS first.
    pub fn request(&mut self, want: usize, to_memory: bool, gif: &mut dyn Gif) -> usize {
        let n = if to_memory != self.reversed() {
            0
        } else if to_memory {
            while self.fifo.len() < DEPTH[1] {
                let Some(q) = gif.host_read() else { break };
                self.fifo.push_back((q, false));
            }
            want.min(self.fifo.len())
        } else {
            want.min(DEPTH[self.unit].saturating_sub(self.fifo.len()))
        };
        if n == 0 {
            self.dmac_waiting = true;
        }
        n
    }

    /// A qword from the DMAC (a DMAtag with `tag`) or the EE's store into the FIFO; lost while reversed.
    pub fn put(&mut self, q: u128, tag: bool) {
        if !self.reversed() {
            self.fifo.push_back((q, tag));
            self.received += 1;
        }
    }

    /// A qword taken from the reversed FIFO by the DMAC or by the EE's load; 0 when there is none.
    pub fn get(&mut self, gif: &mut dyn Gif) -> u128 {
        if self.reversed()
            && self.fifo.is_empty()
            && let Some(q) = gif.host_read()
        {
            self.fifo.push_back((q, false));
        }
        if self.reversed() { self.fifo.pop_front().map_or(0, |(q, _)| q) } else { 0 }
    }
}
