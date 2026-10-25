//! The GIF (EE User's Manual chapter 7; ps2tek "Graphics Interface (GIF)"), stage S15: the GS's front end, taking GS
//! packets from three paths, PATH1 from VU1's memory on XGKICK, PATH2 from VIF1's DIRECT and DIRECTHL, PATH3 from the
//! DMAC's channel 2 (and the EE's stores) through a FIFO of 16 qwords, and writing them to the GS's host interface in
//! the PACKED, REGLIST and IMAGE formats (`packing`). One path holds the GIF from its packet's first GIFtag to the end
//! of its EOP primitive; between packets the waiting paths are arbitrated PATH1, PATH2, PATH3 (§7.1.2), PATH3 skipped
//! while masked by GIF_MODE's M3R or VIF1's MSKPATH3 (§7.1.4), and in the intermittent mode PATH3's IMAGE data gives
//! way to PATH1 and DIRECT at each slice of 8 qwords (§7.1.3). The GIF runs on its own clock and is brought forward by
//! the board like the VIFs; nothing is polled. Sapphire_Native.md §33.

pub mod packing;

use std::collections::VecDeque;

use crate::clock::BUS;
use crate::gs::HostInterface;
use crate::vu::{Path1, Vu};
use packing::{Flg, Packet, Q_ONE};

/// The GIF's registers and FIFO in the EE's map (EE User's Manual §2.2.1's table; the FIFO mirrored over its 4 KiB
/// as VIF0's is, [A]).
pub const REGS: u32 = 0x1000_3000;
pub const REGS_END: u32 = 0x1000_3100;
pub const FIFO: u32 = 0x1000_6000;
pub const FIFO_END: u32 = 0x1000_7000;

/// PATH3's FIFO in qwords (GIF_STAT.FQC "0-16"; ps2tek "GIF PATH3 Masking").
pub const DEPTH: usize = 16;

/// Register offsets (§7.6).
pub mod reg {
    pub const CTRL: u32 = 0x00;
    pub const MODE: u32 = 0x10;
    pub const STAT: u32 = 0x20;
    pub const TAG0: u32 = 0x40;
    pub const TAG1: u32 = 0x50;
    pub const TAG2: u32 = 0x60;
    pub const TAG3: u32 = 0x70;
    pub const CNT: u32 = 0x80;
    pub const P3CNT: u32 = 0x90;
    pub const P3TAG: u32 = 0xA0;
}

pub mod ctrl {
    pub const RST: u32 = 1;
    pub const PSE: u32 = 1 << 3;
}

pub mod mode {
    pub const M3R: u32 = 1;
    pub const IMT: u32 = 1 << 2;
}

/// GIF_STAT's fields (§7.6).
pub mod stat {
    pub const M3R: u32 = 1;
    pub const M3P: u32 = 1 << 1;
    pub const IMT: u32 = 1 << 2;
    pub const PSE: u32 = 1 << 3;
    pub const IP3: u32 = 1 << 5;
    pub const P3Q: u32 = 1 << 6;
    pub const P2Q: u32 = 1 << 7;
    pub const P1Q: u32 = 1 << 8;
    pub const OPH: u32 = 1 << 9;
    pub const APATH_SHIFT: u32 = 10;
    pub const DIR: u32 = 1 << 12;
    pub const FQC_SHIFT: u32 = 24;
}

/// The three general data transfer paths (§7.1.1), in priority order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Path {
    One = 0,
    Two = 1,
    Three = 2,
}

/// An XGKICK's end estimated as never: a PATH1 packet with no EOP in VU1's memory runs round it for ever.
pub const ENDLESS: u64 = u64::MAX / 4;

/// VU1's memory in qwords, where PATH1's addresses wrap (GIF_CNT's 10-bit VUADDR).
const VU1_QWORDS: u32 = 1024;

pub struct Gif {
    /// The cycle the GIF's next step may begin at.
    time: u64,
    /// The path holding the GIF until its packet ends.
    owner: Option<Path>,
    /// The path whose last qword occupies the GIF until `time`.
    last: Option<Path>,
    packet: [Packet; 3],
    q: u32,
    mode: u32,
    pse: bool,
    /// MSKPATH3's mask, VIF1's (GIF_STAT.M3P).
    m3p: bool,
    /// PATH3's IMAGE data interrupted in the intermittent mode, to resume at the next arbitration.
    ip3: bool,
    p3cnt: u32,
    p3tag: u32,
    /// The GIFtag most recently read on any path.
    tag: u128,
    /// PATH1's next qword in VU1's memory, and its packet's end if nothing holds it.
    vuaddr: u16,
    p1_end: u64,
    /// XGKICKs waiting for PATH1: the address, the cycle and the end the VU was told.
    kicks: VecDeque<(u16, u64, u64)>,
    /// PATH2's request while it waits: the cycle of its first refused qword, and whether it is DIRECTHL's.
    p2: Option<(u64, bool)>,
    /// PATH3's FIFO: each qword with the cycle it arrived.
    fifo: VecDeque<(u128, u64)>,
    /// The cycle of the last qwords put and how many were put at it, one DMA unit.
    batch: (u64, usize),
    dmac_waiting: bool,
    wake: bool,
    /// The cycle the next step begins, when the last run stopped only because it lies beyond.
    resume: Option<u64>,
}

impl Default for Gif {
    fn default() -> Self {
        Gif::new()
    }
}

impl Gif {
    pub fn new() -> Gif {
        Gif {
            time: 0,
            owner: None,
            last: None,
            packet: [Packet::default(); 3],
            q: Q_ONE,
            mode: 0,
            pse: false,
            m3p: false,
            ip3: false,
            p3cnt: 0,
            p3tag: 0,
            tag: 0,
            vuaddr: 0,
            p1_end: 0,
            kicks: VecDeque::with_capacity(4),
            p2: None,
            fifo: VecDeque::with_capacity(DEPTH),
            batch: (u64::MAX, 0),
            dmac_waiting: false,
            wake: false,
            resume: None,
        }
    }

    fn masked(&self) -> bool {
        self.mode & mode::M3R != 0 || self.m3p
    }

    /// PATH3 sits at a slice's end of IMAGE data in the intermittent mode, where PATH1 and DIRECT may take over.
    fn at_slice(&self) -> bool {
        let p = &self.packet[2];
        self.mode & mode::IMT != 0 && p.open && p.tag.flg == Flg::Image && p.taken > 0 && p.taken.is_multiple_of(8)
    }

    /// The path the arbitration grants once the GIF is free, and the cycle, no later than `limit` (§7.1.2): the
    /// earliest request decides when, the priority which among those waiting then. DIRECTHL does not overtake an
    /// interrupted PATH3 (§6.4's DIRECTHL page).
    fn choose(&self, limit: u64) -> Option<(Path, u64)> {
        let p1 = self.kicks.front().map(|k| k.1);
        let p2 = self.p2.filter(|r| !(self.ip3 && r.1)).map(|r| r.0);
        let p3 = if self.ip3 {
            Some(self.time)
        } else if self.masked() {
            None
        } else {
            self.fifo.front().map(|f| f.1)
        };
        let first = [p1, p2, p3].into_iter().flatten().min()?;
        let t = self.time.max(first);
        if t > limit {
            return None;
        }
        [(Path::One, p1), (Path::Two, p2), (Path::Three, p3)].into_iter().find(|(_, r)| r.is_some_and(|r| r <= t)).map(|(p, _)| (p, t))
    }

    /// The arbitration's idle cycle, then `path` holds the GIF.
    fn grant(&mut self, path: Path, at: u64) {
        self.owner = Some(path);
        self.time = at + BUS;
        if path == Path::Three {
            self.ip3 = false;
        }
    }

    fn interrupt(&mut self) {
        self.ip3 = true;
        self.p3cnt = u32::from(self.packet[2].loops);
        self.owner = None;
    }

    /// One qword of `path`'s packet at cycle `at`.
    fn step(&mut self, path: Path, w: u128, at: u64, gs: &mut dyn HostInterface) {
        let mut out = Host(gs);
        let s = self.packet[path as usize].take(w, &mut self.q, &mut out);
        if s.tag {
            self.tag = w;
            if path == Path::Three {
                self.p3tag = (w as u32) & 0xFFFF;
            }
        }
        self.time = at + s.cycles * BUS;
        self.last = Some(path);
        if s.end {
            self.owner = None;
        }
    }

    /// Runs every step that begins by cycle `t`: arbitration, PATH1's qwords from VU1's memory and PATH3's from the
    /// FIFO. Nothing moves while GIF_CTRL.PSE pauses the GIF or BUSDIR turns the host interface round [A].
    pub fn run_to(&mut self, t: u64, vu1: &Vu, gs: &mut dyn HostInterface) {
        self.resume = None;
        if self.pse || gs.reversed() {
            return;
        }
        loop {
            match self.owner {
                None => {
                    let Some((path, at)) = self.choose(t) else {
                        self.resume = self.choose(u64::MAX).map(|(_, at)| at);
                        return;
                    };
                    self.grant(path, at);
                    if path == Path::One {
                        let (addr, _, _) = self.kicks.pop_front().expect("a kick");
                        self.vuaddr = addr;
                        let c = packing::cycles(|i| vu1.mem_q((i % VU1_QWORDS) as usize), u32::from(addr), VU1_QWORDS);
                        self.p1_end = c.map_or(ENDLESS, |c| self.time + c * BUS);
                    }
                }
                Some(Path::One) => {
                    if self.time > t {
                        self.resume = Some(self.time);
                        return;
                    }
                    let w = vu1.mem_q(self.vuaddr as usize);
                    self.vuaddr = (self.vuaddr + 1) % VU1_QWORDS as u16;
                    self.step(Path::One, w, self.time, gs);
                }
                Some(Path::Three) => {
                    if self.at_slice() && (!self.kicks.is_empty() || self.p2.is_some_and(|r| !r.1)) {
                        self.interrupt();
                        continue;
                    }
                    let Some(&(w, arrived)) = self.fifo.front() else { return };
                    let at = self.time.max(arrived);
                    if at > t {
                        self.resume = Some(at);
                        return;
                    }
                    self.fifo.pop_front();
                    if self.dmac_waiting {
                        self.dmac_waiting = false;
                        self.wake = true;
                    }
                    self.step(Path::Three, w, at, gs);
                }
                Some(Path::Two) => return,
            }
        }
    }

    /// The cycle the board next needs the GIF: only while the DMAC waits on the FIFO and the GIF can move.
    pub fn next_event(&self) -> u64 {
        if self.dmac_waiting { self.resume.unwrap_or(u64::MAX) } else { u64::MAX }
    }

    /// Whether the DMAC is to be woken: FIFO room freed while it waited.
    pub fn take_wake(&mut self) -> bool {
        std::mem::take(&mut self.wake)
    }

    fn request2(&mut self, at: u64, hl: bool) {
        if self.p2.is_none() {
            self.p2 = Some((at, hl));
        }
    }

    /// A qword of DIRECT's (DIRECTHL's with `hl`) data offered to PATH2 at `at` (`vif::Gif::path2`): the cycle it is
    /// taken, or none while another path holds the GIF, a higher one is waiting, or PATH3's IMAGE data cannot be
    /// interrupted (§7.1.3; DIRECTHL's page).
    pub fn path2(&mut self, w: u128, hl: bool, at: u64, gs: &mut dyn HostInterface) -> Option<u64> {
        self.request2(at, hl);
        if self.pse || gs.reversed() {
            return None;
        }
        match self.owner {
            Some(Path::Two) => {}
            None => match self.choose(self.time.max(at)) {
                Some((Path::Two, t)) => self.grant(Path::Two, t),
                _ => return None,
            },
            Some(Path::Three) if !hl && self.at_slice() => {
                self.interrupt();
                self.grant(Path::Two, self.time.max(at));
            }
            _ => return None,
        }
        let start = self.time.max(at);
        self.p2 = None;
        self.step(Path::Two, w, start, gs);
        Some(start)
    }

    /// The cycle PATH1 and PATH2 are idle from, `at` or later, and with `path3` also no transfer request from PATH3
    /// (FLUSH, FLUSHA, MSCALF, §6.4's pages); `dma` is channel 2 running. None while that cannot yet be known.
    pub fn idle_at(&self, path3: bool, dma: bool, at: u64) -> Option<u64> {
        if matches!(self.owner, Some(Path::One | Path::Two)) || !self.kicks.is_empty() {
            return None;
        }
        if path3 && (self.owner == Some(Path::Three) || self.ip3 || !self.fifo.is_empty() || dma) {
            return None;
        }
        let busy = match self.last {
            Some(Path::Three) => path3,
            Some(_) => true,
            None => false,
        };
        Some(if busy { at.max(self.time) } else { at })
    }

    /// MSKPATH3 (§6.4): the mask the arbitration reads from the next PATH3 packet.
    pub fn mask_path3(&mut self, masked: bool) {
        self.m3p = masked;
    }

    /// An XGKICK handed on from VU1 (`Path1Port`): VU1's qword address, its cycle, and the end the VU was told.
    pub fn kick(&mut self, addr: u16, at: u64, told: u64) {
        self.kicks.push_back((addr, at, told));
    }

    /// When a PATH1 transfer kicked now could start, for the VU's estimate (`Path1Port::set_free`): behind the kicks
    /// waiting, the packet PATH1 is sending, or the primitive another path is sending.
    pub fn path1_free(&self, now: u64) -> u64 {
        if let Some(&(_, _, told)) = self.kicks.back() {
            return told;
        }
        match self.owner {
            Some(Path::One) => self.p1_end,
            Some(p) => self.time.max(now) + BUS * self.packet[p as usize].remaining(),
            None => self.time,
        }
    }

    /// The DMAC's request on channel 2: room in the FIFO, which PATH3's mask does not change (ps2tek "GIF PATH3
    /// Masking").
    pub fn request(&mut self, want: usize) -> usize {
        let room = DEPTH.saturating_sub(self.fifo.len());
        if room == 0 {
            self.dmac_waiting = true;
        }
        want.min(room)
    }

    /// A qword into the FIFO at cycle `at`, from the DMAC or the EE's store; a DMAtag sent by TTE is not GIF data and
    /// is dropped (D-283). The qwords of one DMA unit, handed over together at its end, are taken to have arrived one a
    /// bus cycle up to it, as the DMAC moved them (§33.4).
    pub fn put(&mut self, w: u128, tag: bool, at: u64) {
        if tag {
            return;
        }
        self.batch = if self.batch.0 == at { (at, self.batch.1 + 1) } else { (at, 1) };
        self.fifo.push_back((w, at));
        for (i, e) in self.fifo.iter_mut().rev().take(self.batch.1).enumerate() {
            e.1 = at.saturating_sub(BUS * i as u64);
        }
    }

    /// GIF_CTRL.RST (§7.6): the paths' packets, the FIFO, the queued kicks, PATH3's interruption and the registers
    /// back to their initial values; MSKPATH3's mask stays, the VIF's to cancel (§7.1.4, D-284).
    fn reset(&mut self) {
        let (m3p, time) = (self.m3p, self.time);
        let mut kicks = std::mem::take(&mut self.kicks);
        let mut fifo = std::mem::take(&mut self.fifo);
        kicks.clear();
        fifo.clear();
        *self = Gif { m3p, time, kicks, fifo, wake: true, ..Gif::new() };
    }

    /// A register written by the EE: GIF_CTRL and GIF_MODE; the rest are read-only.
    pub fn write(&mut self, off: u32, v: u32) {
        match off {
            reg::CTRL => {
                if v & ctrl::RST != 0 {
                    self.reset();
                }
                self.pse = v & ctrl::PSE != 0;
            }
            reg::MODE => self.mode = v & (mode::M3R | mode::IMT),
            _ => {}
        }
    }

    /// A register read at cycle `now`; `reversed` is BUSDIR's direction. The registers §7.6 marks readable only while
    /// paused are answered at any time, and the write-only ones read 0 [A].
    pub fn read(&self, off: u32, now: u64, reversed: bool) -> u32 {
        match off {
            reg::STAT => self.status(now, reversed),
            reg::TAG0 => self.tag as u32,
            reg::TAG1 => (self.tag >> 32) as u32,
            reg::TAG2 => (self.tag >> 64) as u32,
            reg::TAG3 => (self.tag >> 96) as u32,
            reg::CNT => {
                let p = self.owner.or(self.last).unwrap_or(Path::Three);
                self.packet[p as usize].cnt(if p == Path::One { self.vuaddr } else { 0 })
            }
            reg::P3CNT => self.p3cnt,
            reg::P3TAG => self.p3tag,
            _ => 0,
        }
    }

    /// GIF_STAT at cycle `now` (§7.6).
    pub fn status(&self, now: u64, reversed: bool) -> u32 {
        let mut s = 0;
        if self.mode & mode::M3R != 0 {
            s |= stat::M3R;
        }
        if self.m3p {
            s |= stat::M3P;
        }
        if self.mode & mode::IMT != 0 {
            s |= stat::IMT;
        }
        if self.pse {
            s |= stat::PSE;
        }
        if self.ip3 {
            s |= stat::IP3;
        }
        if self.owner != Some(Path::Three) && (self.ip3 || !self.fifo.is_empty()) {
            s |= stat::P3Q;
        }
        if self.owner != Some(Path::Two) && self.p2.is_some() {
            s |= stat::P2Q;
        }
        if !self.kicks.is_empty() {
            s |= stat::P1Q;
        }
        if let Some(p) = self.owner.or(self.last.filter(|_| self.time > now)) {
            s |= stat::OPH | (p as u32 + 1) << stat::APATH_SHIFT;
        }
        if reversed {
            s |= stat::DIR;
        }
        s | (self.fifo.len() as u32) << stat::FQC_SHIFT
    }
}

/// The GS's host interface as the packing's output.
struct Host<'a>(&'a mut dyn HostInterface);

impl packing::Out for Host<'_> {
    fn write(&mut self, r: u8, data: u64) {
        self.0.write(r, data);
    }
}

/// PATH1 as VU1 reaches it (`vu::Path1`): each XGKICK kept for the board to hand to the GIF, and the end the VU is
/// told estimated from the packet in VU1's memory and the GIF's state as the board last gave it (§33.4).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Path1Port {
    free: u64,
    kicks: VecDeque<(u16, u64, u64)>,
}

impl Path1 for Path1Port {
    fn kick(&mut self, data: &[u8], addr: u16, at: u64) -> u64 {
        let start = at.max(self.free);
        let qwords = (data.len() / 16) as u32;
        let read = |i: u32| {
            let a = (i % qwords) as usize * 16;
            u128::from_le_bytes(data[a..a + 16].try_into().unwrap())
        };
        let end = packing::cycles(read, u32::from(addr), qwords).map_or(ENDLESS, |c| start + BUS * (1 + c));
        self.kicks.push_back((addr, at, end));
        self.free = end;
        end
    }

    fn set_free(&mut self, at: u64) {
        self.free = at;
    }

    fn take_kick(&mut self) -> Option<(u16, u64, u64)> {
        self.kicks.pop_front()
    }
}
