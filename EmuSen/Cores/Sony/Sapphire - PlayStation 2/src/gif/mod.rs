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
    /// The path holding the GIF has output something since it was granted: data, or a GIFtag with NLOOP above 0.
    output: bool,
    /// BUSDIR as the last `run_to` saw it, kept while nothing has reached the GIF since; not state (Sapphire_Speed.md §61.3).
    still: Option<bool>,
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
            output: false,
            still: None,
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
        self.output = false;
        self.time = at + BUS;
        if path == Path::Three {
            self.ip3 = false;
        }
    }

    /// PATH3's IMAGE data gives way at a slice's end to a request seen at `at`.
    fn interrupt(&mut self, at: u64) {
        self.time = self.time.max(at);
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
        self.output |= !s.tag || self.packet[path as usize].open;
        if s.end {
            self.owner = None;
        }
    }

    /// Runs every step that begins by cycle `t`: arbitration, PATH1's qwords from VU1's memory and PATH3's from the
    /// FIFO. Nothing moves while GIF_CTRL.PSE pauses the GIF or BUSDIR turns the host interface round [A].
    pub fn run_to(&mut self, t: u64, vu1: &Vu, gs: &mut dyn HostInterface) {
        let reversed = gs.reversed();
        if self.still == Some(reversed) && self.resume.is_none_or(|r| t < r) {
            return;
        }
        self.still = Some(reversed);
        self.resume = None;
        if self.pse || reversed {
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
                        let at = self.time;
                        let c = packing::cycles(|i| vu1.mem_q_at((i % VU1_QWORDS) as usize, at), u32::from(addr), VU1_QWORDS);
                        self.p1_end = self.time + c.unwrap_or(u64::from(VU1_QWORDS)) * BUS;
                    }
                }
                Some(Path::One) => {
                    if self.time > t {
                        self.resume = Some(self.time);
                        return;
                    }
                    let w = vu1.mem_q_at(self.vuaddr as usize, self.time);
                    self.vuaddr = (self.vuaddr + 1) % VU1_QWORDS as u16;
                    self.step(Path::One, w, self.time, gs);
                }
                Some(Path::Three) => {
                    let asks = self.kicks.front().map(|k| k.1).or(self.p2.filter(|r| !r.1).map(|r| r.0));
                    if let Some(at) = asks.filter(|_| self.at_slice()) {
                        self.interrupt(at);
                        continue;
                    }
                    let Some(&(w, arrived)) = self.fifo.front() else {
                        // A packet that has output nothing does not hold the GIF while PATH3's data stops (D-350).
                        if !self.output && !self.packet[Path::Three as usize].open {
                            self.owner = None;
                            continue;
                        }
                        return;
                    };
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

    /// Whether `run_to` would find nothing to send: no path waiting, PATH3's FIFO empty, and the GIF free or held by
    /// PATH3 or PATH2 between their qwords (the board's shortcut, Sapphire_Native.md §35.7).
    pub fn quiet(&self) -> bool {
        self.owner != Some(Path::One) && self.kicks.is_empty() && self.p2.is_none() && self.fifo.is_empty() && !self.ip3
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
        self.still = None;
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
                self.interrupt(at);
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
        self.still = None;
        self.m3p = masked;
    }

    /// An XGKICK handed on from VU1 (`Path1Port`): VU1's qword address, its cycle, and the end the VU was told.
    pub fn kick(&mut self, addr: u16, at: u64, told: u64) {
        self.still = None;
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
        self.still = None;
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
        self.still = None;
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
        self.still = None;
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
        // A packet whose EOP is not yet in VU1's memory is completed as PATH1 reads it (D-288); the VU is told one pass
        // over the memory, never a time that cannot come (Sapphire_Disputes.md D-351).
        let end = start + BUS * (1 + packing::cycles(read, u32::from(addr), qwords).unwrap_or(u64::from(qwords)));
        self.kicks.push_back((addr, at, end));
        self.free = end;
        end
    }

    fn set_free(&mut self, at: u64) {
        // Kicks held for a later cycle end where the last was told to (Sapphire_Speed.md §63).
        self.free = self.kicks.back().map_or(at, |k| k.2);
    }

    fn take_kick(&mut self, t: u64) -> Option<(u16, u64, u64)> {
        if self.kicks.front().is_some_and(|k| k.1 <= t) { self.kicks.pop_front() } else { None }
    }

    fn write_state(&self, w: &mut emusen_native::StateWriter) {
        w.u64("Free", self.free);
        write_kicks(w, &self.kicks);
    }

    fn read_state(&mut self, r: &mut emusen_native::StateReader) -> Result<(), emusen_native::Truncated> {
        self.free = r.u64()?;
        read_kicks(r, &mut self.kicks)
    }

    fn fits(&self) -> bool {
        self.kicks.len() <= KICK_SLOTS
    }

    fn holds(&self) -> bool {
        !self.kicks.is_empty()
    }
}

/// XGKICKs waiting in the state, and PATH3's FIFO with room for the EE's stores past the 16 the DMAC fills.
const KICK_SLOTS: usize = 16;
const FIFO_SLOTS: usize = 64;

fn path_code(p: Option<Path>) -> u8 {
    p.map_or(0xFF, |p| p as u8)
}

fn path_of(c: u8) -> Option<Path> {
    [Path::One, Path::Two, Path::Three].get(usize::from(c)).copied()
}

fn write_kicks(w: &mut emusen_native::StateWriter, kicks: &VecDeque<(u16, u64, u64)>) {
    let k = crate::state::slots::<_, KICK_SLOTS>(kicks);
    w.u32("KicksLength", kicks.len() as u32);
    w.u16s("KicksAddr", &k.map(|k| k.0));
    w.u64s("KicksAt", &k.map(|k| k.1));
    w.u64s("KicksEnd", &k.map(|k| k.2));
}

fn read_kicks(r: &mut emusen_native::StateReader, kicks: &mut VecDeque<(u16, u64, u64)>) -> Result<(), emusen_native::Truncated> {
    let len = r.u32()?;
    let mut addr = [0u16; KICK_SLOTS];
    let mut at = [0u64; KICK_SLOTS];
    let mut end = [0u64; KICK_SLOTS];
    r.u16s(&mut addr)?;
    r.u64s(&mut at)?;
    r.u64s(&mut end)?;
    crate::state::refill::<_, KICK_SLOTS>(kicks, len, &std::array::from_fn(|i| (addr[i], at[i], end[i])));
    Ok(())
}

/// The GIF's block of `SPH2` (Sapphire_Native.md §38.2): its time and arbitration, each path's packet, the registers,
/// the XGKICKs waiting, PATH2's request and PATH3's FIFO.
impl emusen_native::State for Gif {
    type Error = emusen_native::Truncated;

    fn write_state(&self, w: &mut emusen_native::StateWriter) {
        w.u64s("Times", &[self.time, self.p1_end, self.batch.0, self.batch.1 as u64]);
        w.bytes("Paths", &[path_code(self.owner), path_code(self.last)]);
        w.structure("Path1", &self.packet[0]);
        w.structure("Path2", &self.packet[1]);
        w.structure("Path3", &self.packet[2]);
        w.u32s("Registers", &[self.q, self.mode, self.p3cnt, self.p3tag]);
        w.u128("Tag", self.tag);
        w.u16("VuAddr", self.vuaddr);
        w.bools("Flags", &[self.pse, self.m3p, self.ip3, self.dmac_waiting, self.wake, self.output]);
        write_kicks(w, &self.kicks);
        let p2 = self.p2.unwrap_or((0, false));
        w.bools("Path2Request", &[self.p2.is_some(), p2.1]);
        w.u64("Path2At", p2.0);
        crate::state::opt_u64(w, "Resume", self.resume);
        let f = crate::state::slots::<_, FIFO_SLOTS>(&self.fifo);
        w.u32("FifoLength", self.fifo.len() as u32);
        w.u128s("FifoQwords", &f.map(|e| e.0));
        w.u64s("FifoAt", &f.map(|e| e.1));
    }

    fn read_state(&mut self, r: &mut emusen_native::StateReader) -> Result<(), emusen_native::Truncated> {
        let mut t = [0u64; 4];
        r.u64s(&mut t)?;
        (self.time, self.p1_end, self.batch) = (t[0], t[1], (t[2], t[3] as usize));
        let mut p = [0u8; 2];
        r.bytes(&mut p)?;
        (self.owner, self.last) = (path_of(p[0]), path_of(p[1]));
        for packet in &mut self.packet {
            packet.read_state(r)?;
        }
        let mut g = [0u32; 4];
        r.u32s(&mut g)?;
        [self.q, self.mode, self.p3cnt, self.p3tag] = g;
        self.tag = r.u128()?;
        self.vuaddr = r.u16()?;
        let mut f = [false; 6];
        r.bools(&mut f)?;
        [self.pse, self.m3p, self.ip3, self.dmac_waiting, self.wake, self.output] = f;
        read_kicks(r, &mut self.kicks)?;
        let mut p2 = [false; 2];
        r.bools(&mut p2)?;
        let at = r.u64()?;
        self.p2 = p2[0].then_some((at, p2[1]));
        self.resume = crate::state::read_opt_u64(r)?;
        self.still = None;
        let len = r.u32()?;
        let mut q = [0u128; FIFO_SLOTS];
        let mut at = [0u64; FIFO_SLOTS];
        r.u128s(&mut q)?;
        r.u64s(&mut at)?;
        crate::state::refill::<_, FIFO_SLOTS>(&mut self.fifo, len, &std::array::from_fn(|i| (q[i], at[i])));
        Ok(())
    }
}

impl Gif {
    /// Whether the GIF's queues are within their slots in the state.
    pub fn fits(&self) -> bool {
        self.kicks.len() <= KICK_SLOTS && self.fifo.len() <= FIFO_SLOTS
    }
}
