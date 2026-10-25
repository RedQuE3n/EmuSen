//! The DMAC (EE User's Manual chapter 5; ps2tek "DMA Controller"): ten channels in normal, chain and interleave modes,
//! source and destination chain tags, tag interrupts, the address stack, stall control, the MFIFO, the priority
//! control, the release cycle, D_ENABLEW's hold, INT1 and CPCOND0. A transfer moves in units (a slice, a burst or an
//! interleave block, a tag), each costing its qwords at one a bus cycle and taking effect at its last cycle; between
//! units the channels are arbitrated. The DMAC is woken only when a unit ends, a release period ends or a register
//! is written, never polled. The peripherals behind channels 0 to 7 are reached through `Peripherals`, which the
//! stages that build them implement; the scratchpad channels work end to end. Sapphire_Native.md §20.2 and §20.3.

use crate::clock::BUS;

pub const CHANNELS: usize = 10;
pub const VIF0: usize = 0;
pub const VIF1: usize = 1;
pub const GIF: usize = 2;
pub const FROM_IPU: usize = 3;
pub const TO_IPU: usize = 4;
pub const SIF0: usize = 5;
pub const SIF1: usize = 6;
pub const SIF2: usize = 7;
pub const FROM_SPR: usize = 8;
pub const TO_SPR: usize = 9;

/// Each channel's register block (§2.2.1).
pub const BASES: [u32; CHANNELS] =
    [0x1000_8000, 0x1000_9000, 0x1000_A000, 0x1000_B000, 0x1000_B400, 0x1000_C000, 0x1000_C400, 0x1000_C800, 0x1000_D000, 0x1000_D400];
pub const D_CTRL: u32 = 0x1000_E000;
pub const D_STAT: u32 = 0x1000_E010;
pub const D_PCR: u32 = 0x1000_E020;
pub const D_SQWC: u32 = 0x1000_E030;
pub const D_RBSR: u32 = 0x1000_E040;
pub const D_RBOR: u32 = 0x1000_E050;
pub const D_STADR: u32 = 0x1000_E060;
pub const D_ENABLER: u32 = 0x1000_F520;
pub const D_ENABLEW: u32 = 0x1000_F590;

/// Dn_CHCR's fields (§5.9).
pub mod chcr {
    pub const DIR: u32 = 1;
    pub const MOD_SHIFT: u32 = 2;
    pub const ASP_SHIFT: u32 = 4;
    pub const TTE: u32 = 1 << 6;
    pub const TIE: u32 = 1 << 7;
    pub const STR: u32 = 1 << 8;
    /// The bits a write keeps: DIR, MOD, ASP, TTE, TIE, STR and TAG [A: bits 1 and 9 to 15 read 0].
    pub const BITS: u32 = 0xFFFF_01FD;
}

/// D_CTRL's fields (§5.8).
pub mod ctrl {
    pub const DMAE: u32 = 1;
    pub const RELE: u32 = 1 << 1;
}

/// D_STAT's fields (§5.8).
pub mod stat {
    pub const SIS: u32 = 1 << 13;
    pub const MEIS: u32 = 1 << 14;
    pub const BEIS: u32 = 1 << 15;
    pub const CLEARED: u32 = 0x3FF | SIS | MEIS | BEIS;
    pub const MASKS: u32 = 0x3FF << 16 | 1 << 29 | 1 << 30;
}

/// D_PCR's PCE (§5.8).
pub const PCE: u32 = 1 << 31;
/// D_ENABLEW's and D_ENABLER's CPND (§5.8).
pub const CPND: u32 = 1 << 16;

/// The peripherals behind channels 0 to 7, as a channel sees them: DREQ, a qword handed to the peripheral (memory to
/// peripheral; `tag` marks a DMAtag sent by TTE) and a qword taken from it (peripheral to memory, a destination
/// chain's tags among them). The stages that build the VIFs, the GIF, the IPU and the SIF implement it.
pub trait Peripherals {
    /// How many of `want` qwords the peripheral takes or gives now; 0 holds the channel until `Dmac::wake` is called.
    fn request(&mut self, ch: usize, want: usize) -> usize;
    fn put(&mut self, ch: usize, q: u128, tag: bool);
    fn get(&mut self, ch: usize) -> u128;
    /// Whether a destination chain's tag can be taken now; a peripheral whose tag is narrower than a qword says so.
    fn tag_ready(&mut self, ch: usize) -> bool {
        self.request(ch, 1) > 0
    }
    /// A destination chain's tag, its DMAtag in bits 63..0.
    fn get_tag(&mut self, ch: usize) -> u128 {
        self.get(ch)
    }
}

/// Memory by physical address, as the DMAC reaches it (main memory and what the map places beside it); `None` and
/// `false` are a bus error.
pub trait Memory {
    fn read_q(&mut self, paddr: u32) -> Option<u128>;
    fn write_q(&mut self, paddr: u32, q: u128) -> bool;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Normal,
    SourceChain,
    DestChain,
    Interleave,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Channel {
    pub chcr: u32,
    pub madr: u32,
    pub qwc: u32,
    pub tadr: u32,
    pub asr: [u32; 2],
    pub sadr: u32,
    /// A chain's packet is in progress: its data still to move, or its end not yet acted on.
    open: bool,
    /// The packet in progress is the chain's last (refe, end, ret with nothing pushed).
    ends: bool,
    /// The packet in progress is under stall control: normal mode, refs on a drain, cnts on a source (§5.3.2).
    stall: bool,
    /// After the packet, TADR takes the address after its data (cnt alone; Sapphire_Disputes.md D-35).
    tadr_follows: bool,
    /// The packet's data lies in the MFIFO's ring (cnt, next, call, ret, end on the drain).
    in_ring: bool,
    /// Qwords moved in the current interleave block.
    block: u32,
}

impl Channel {
    pub fn str(&self) -> bool {
        self.chcr & chcr::STR != 0
    }

    fn asp(&self) -> u32 {
        self.chcr >> chcr::ASP_SHIFT & 3
    }

    fn set_asp(&mut self, n: u32) {
        self.chcr = self.chcr & !(3 << chcr::ASP_SHIFT) | n << chcr::ASP_SHIFT;
    }

    fn tag_id(&self) -> u32 {
        self.chcr >> 28 & 7
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Work {
    Data(u32),
    Tag,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Unit {
    ch: usize,
    work: Work,
    end: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dmac {
    pub ch: [Channel; CHANNELS],
    pub ctrl: u32,
    pub stat: u32,
    pub pcr: u32,
    /// D_SQWC as written; a transfer uses its fields SQWC (7..0) and TQWC (23..16) (D-40).
    pub sqwc: u32,
    pub rbsr: u32,
    pub rbor: u32,
    pub stadr: u32,
    pub enable: u32,
    unit: Option<Unit>,
    /// The earliest time the next unit may start, after the release period of RCYC bus cycles (§5.2.1).
    release_until: u64,
    /// The time the DMAC next needs attention; `u64::MAX` when idle or held by a condition only a write or a peripheral
    /// can change.
    wake: u64,
    /// The group C channel served last, for the round robin (§5.3.1).
    last_c: usize,
}

impl Default for Dmac {
    fn default() -> Self {
        Dmac {
            ch: Default::default(),
            ctrl: 0,
            stat: 0,
            pcr: 0,
            sqwc: 0,
            rbsr: 0,
            rbor: 0,
            stadr: 0,
            enable: 0,
            unit: None,
            release_until: 0,
            wake: u64::MAX,
            last_c: TO_SPR,
        }
    }
}

/// Group C in the order its round robin visits (§5.3.1).
const GROUP_C: [usize; 8] = [VIF1, GIF, FROM_IPU, TO_IPU, SIF0, SIF1, FROM_SPR, TO_SPR];

/// The source chain's tag IDs (§5.6).
mod id {
    pub const REFE: u32 = 0;
    pub const CNT: u32 = 1;
    pub const NEXT: u32 = 2;
    pub const REF: u32 = 3;
    pub const REFS: u32 = 4;
    pub const CALL: u32 = 5;
    pub const RET: u32 = 6;
    pub const END: u32 = 7;
    /// The destination chain's cnts.
    pub const CNTS: u32 = 0;
}

const ADDR: u32 = 0x7FFF_FFF0;
const SPR_BIT: u32 = 1 << 31;

impl Dmac {
    /// The time the DMAC next needs attention.
    pub fn next_event(&self) -> u64 {
        self.wake
    }

    /// Asks for an arbitration at `now`: a register written, or a peripheral's DREQ asserted.
    pub fn wake(&mut self, now: u64) {
        if self.unit.is_none() {
            self.wake = now.max(self.release_until);
        }
    }

    /// INT1 (§5.4, D_STAT): any channel's CIS with its CIM, SIS with SIM, MEIS with MEIM, or BEIS.
    pub fn int1(&self) -> bool {
        let s = self.stat;
        s & 0x3FF & (s >> 16) != 0 || s & stat::SIS != 0 && s & 1 << 29 != 0 || s & stat::MEIS != 0 && s & 1 << 30 != 0 || s & stat::BEIS != 0
    }

    /// CPCOND0 (§5.8, D_PCR): every channel with CPCn has its CIS set.
    pub fn cpcond0(&self) -> bool {
        !self.pcr & 0x3FF | self.stat & 0x3FF == 0x3FF
    }

    fn rele(&self) -> bool {
        self.ctrl & ctrl::RELE != 0
    }

    fn rcyc(&self) -> u64 {
        8 << (self.ctrl >> 8 & 7).min(5)
    }

    fn mfd(&self) -> Option<usize> {
        match self.ctrl >> 2 & 3 {
            2 => Some(VIF1),
            3 => Some(GIF),
            _ => None,
        }
    }

    fn sts(&self) -> Option<usize> {
        match self.ctrl >> 4 & 3 {
            1 => Some(SIF0),
            2 => Some(FROM_SPR),
            3 => Some(FROM_IPU),
            _ => None,
        }
    }

    fn std(&self) -> Option<usize> {
        match self.ctrl >> 6 & 3 {
            1 => Some(VIF1),
            2 => Some(GIF),
            3 => Some(SIF1),
            _ => None,
        }
    }

    /// Whether channel `c` moves memory to its peripheral (§5.1's direction; VIF1 and SIF2 by CHCR.DIR).
    fn reads_memory(&self, c: usize) -> bool {
        match c {
            VIF0 | GIF | TO_IPU | SIF1 | TO_SPR => true,
            VIF1 | SIF2 => self.ch[c].chcr & chcr::DIR != 0,
            _ => false,
        }
    }

    /// The logical mode a channel runs in: chain only where §5.1 gives the channel a chain, interleave only on the
    /// scratchpad channels; any other choice runs as normal [A].
    fn mode(&self, c: usize) -> Mode {
        match self.ch[c].chcr >> chcr::MOD_SHIFT & 3 {
            1 => match c {
                VIF0 | VIF1 | GIF | TO_IPU | SIF1 | TO_SPR => Mode::SourceChain,
                SIF0 | FROM_SPR => Mode::DestChain,
                _ => Mode::Normal,
            },
            2 if c == FROM_SPR || c == TO_SPR => Mode::Interleave,
            _ => Mode::Normal,
        }
    }

    fn burst(c: usize) -> bool {
        c == FROM_SPR || c == TO_SPR
    }

    /// The MFIFO's ring address of `a` (§5.5, D_RBOR, D_RBSR).
    fn ring(&self, a: u32) -> u32 {
        self.rbor & ADDR | a & self.rbsr & ADDR
    }

    fn ring_drain(&self, c: usize) -> bool {
        self.mfd() == Some(c) && self.mode(c) == Mode::SourceChain
    }

    fn finish(&mut self, c: usize) {
        self.ch[c].chcr &= !chcr::STR;
        self.ch[c].open = false;
        self.stat |= 1 << c;
    }

    fn bus_error(&mut self, c: usize) {
        self.finish(c);
        self.stat |= stat::BEIS;
    }

    /// STR written from 0 to 1: a chain with QWC left resumes the packet CHCR.TAG names (§5.2.3; ps2tek "DMA
    /// Resuming"); otherwise it reads its first tag.
    fn kick(&mut self, c: usize) {
        let mode = self.mode(c);
        let ch = &mut self.ch[c];
        ch.block = 0;
        ch.open = false;
        ch.ends = false;
        ch.stall = mode == Mode::Normal;
        ch.tadr_follows = false;
        ch.in_ring = false;
        if (mode == Mode::SourceChain || mode == Mode::DestChain) && ch.qwc > 0 {
            let tag = ch.tag_id();
            ch.open = true;
            ch.ends = if mode == Mode::SourceChain { tag == id::REFE || tag == id::END } else { tag == id::END };
            ch.stall = if mode == Mode::SourceChain { tag == id::REFS } else { tag == id::CNTS };
            ch.tadr_follows = mode == Mode::SourceChain && tag == id::CNT;
            ch.in_ring = mode == Mode::SourceChain && matches!(tag, id::CNT | id::NEXT | id::CALL | id::RET | id::END);
        }
    }

    /// Drain `c` held by stall control: Dd_MADR.ADDR + 8 qwords > D_STADR.ADDR (§5.3.2, Sapphire_Disputes.md D-34).
    fn stalled(&self, c: usize) -> bool {
        let ch = &self.ch[c];
        self.std() == Some(c) && ch.stall && (ch.madr & ADDR) + 8 * 16 > self.stadr & ADDR
    }

    /// The MFIFO's drain held: its TADR equals fromSPR's MADR (§5.5).
    fn ring_empty(&self, c: usize) -> bool {
        self.ring_drain(c) && self.ch[c].tadr & ADDR == self.ch[FROM_SPR].madr & ADDR
    }

    /// The zero-time steps a channel takes before its next unit (a packet's end, a finished transfer); then the unit
    /// it would run now, or none when it is stopped or held.
    fn next_work(&mut self, c: usize, dev: &mut dyn Peripherals) -> Option<Work> {
        if !self.ch[c].str() {
            return None;
        }
        let mode = self.mode(c);
        match mode {
            Mode::Normal | Mode::Interleave => {
                if self.ch[c].qwc == 0 {
                    self.finish(c);
                    return None;
                }
            }
            Mode::SourceChain | Mode::DestChain => {
                self.end_packet(c);
                if !self.ch[c].str() {
                    return None;
                }
                if !self.ch[c].open {
                    if self.ring_drain(c) {
                        self.ch[c].tadr = self.ring(self.ch[c].tadr);
                        if self.ring_empty(c) {
                            self.stat |= stat::MEIS;
                            return None;
                        }
                    }
                    let spr = c == TO_SPR || c == FROM_SPR;
                    if !spr && mode == Mode::DestChain && !dev.tag_ready(c) {
                        return None;
                    }
                    if !spr && mode == Mode::SourceChain && self.ch[c].chcr & chcr::TTE != 0 && dev.request(c, 1) == 0 {
                        return None;
                    }
                    return Some(Work::Tag);
                }
            }
        }
        if self.stalled(c) {
            self.stat |= stat::SIS;
            return None;
        }
        if self.ring_empty(c) {
            return None;
        }
        let ch = &self.ch[c];
        let mut n = ch.qwc;
        if mode == Mode::Interleave {
            n = n.min((self.sqwc >> 16 & 0xFF).max(1) - ch.block);
        }
        if !Self::burst(c) || self.rele() && mode != Mode::Interleave {
            n = n.min(8 - (ch.madr >> 4 & 7));
        }
        if !Self::burst(c) {
            n = dev.request(c, n as usize) as u32;
            if n == 0 {
                return None;
            }
        }
        Some(Work::Data(n))
    }

    /// Chooses the next unit: group A, then B, then C by round robin, among channels the DMAC may run (§5.3.1, §5.8:
    /// DMAE, PCE and CDEn (D-41), STR; D_ENABLEW's CPND holding all).
    fn arbitrate(&mut self, dev: &mut dyn Peripherals) -> Option<(usize, Work)> {
        if self.ctrl & ctrl::DMAE == 0 || self.enable & CPND != 0 {
            return None;
        }
        let allowed = |d: &Dmac, c: usize| d.pcr & PCE == 0 || d.pcr & 1 << (16 + c) != 0;
        for c in [VIF0, SIF2] {
            if allowed(self, c)
                && let Some(w) = self.next_work(c, dev)
            {
                return Some((c, w));
            }
        }
        let start = GROUP_C.iter().position(|&c| c == self.last_c).unwrap_or(GROUP_C.len() - 1);
        for k in 1..=GROUP_C.len() {
            let c = GROUP_C[(start + k) % GROUP_C.len()];
            if allowed(self, c)
                && let Some(w) = self.next_work(c, dev)
            {
                self.last_c = c;
                return Some((c, w));
            }
        }
        None
    }

    /// Runs the DMAC at `now`: ends the unit due, then starts the next one the arbitration gives.
    pub fn step(&mut self, now: u64, mem: &mut dyn Memory, spr: &mut [u8], dev: &mut dyn Peripherals) {
        if let Some(u) = self.unit
            && u.end <= now
        {
            self.unit = None;
            if self.ch[u.ch].str() && self.ctrl & ctrl::DMAE != 0 {
                self.run(u.ch, u.work, mem, spr, dev);
            }
            if self.rele() {
                self.release_until = u.end + self.rcyc() * BUS;
            }
        }
        if self.unit.is_none() {
            if now < self.release_until {
                self.wake = self.release_until;
                return;
            }
            self.wake = u64::MAX;
            if let Some((c, work)) = self.arbitrate(dev) {
                let q = match work {
                    Work::Data(n) => u64::from(n),
                    Work::Tag => 1,
                };
                let end = now + q * BUS;
                self.unit = Some(Unit { ch: c, work, end });
                self.wake = end;
            }
        }
    }

    fn read_mem(&self, addr: u32, mem: &mut dyn Memory, spr: &[u8]) -> Option<u128> {
        if addr & SPR_BIT != 0 {
            let a = (addr & 0x3FF0) as usize;
            Some(u128::from_le_bytes(spr[a..a + 16].try_into().unwrap()))
        } else {
            mem.read_q(addr & ADDR)
        }
    }

    fn write_mem(&self, addr: u32, q: u128, mem: &mut dyn Memory, spr: &mut [u8]) -> bool {
        if addr & SPR_BIT != 0 {
            let a = (addr & 0x3FF0) as usize;
            spr[a..a + 16].copy_from_slice(&q.to_le_bytes());
            true
        } else {
            mem.write_q(addr & ADDR, q)
        }
    }

    fn spr_read(&mut self, c: usize, spr: &[u8]) -> u128 {
        let a = (self.ch[c].sadr & 0x3FF0) as usize;
        self.ch[c].sadr = (self.ch[c].sadr + 16) & 0x3FF0;
        u128::from_le_bytes(spr[a..a + 16].try_into().unwrap())
    }

    fn spr_write(&mut self, c: usize, q: u128, spr: &mut [u8]) {
        let a = (self.ch[c].sadr & 0x3FF0) as usize;
        self.ch[c].sadr = (self.ch[c].sadr + 16) & 0x3FF0;
        spr[a..a + 16].copy_from_slice(&q.to_le_bytes());
    }

    /// The address a qword of channel `c`'s data moves at, and the one after it, wrapped into the ring for the MFIFO's
    /// source (fromSPR with MFD set) and for the drain's packets that lie in the ring.
    fn data_address(&self, c: usize) -> (u32, u32) {
        let a = self.ch[c].madr;
        let ringed = (c == FROM_SPR && self.mfd().is_some()) || (self.ring_drain(c) && self.ch[c].in_ring);
        if ringed { (self.ring(a), self.ring(a.wrapping_add(16))) } else { (a, a.wrapping_add(16)) }
    }

    /// A unit's effect, at its last bus cycle.
    fn run(&mut self, c: usize, work: Work, mem: &mut dyn Memory, spr: &mut [u8], dev: &mut dyn Peripherals) {
        match work {
            Work::Tag => self.read_tag(c, mem, spr, dev),
            Work::Data(n) => {
                let interleave = self.mode(c) == Mode::Interleave;
                let tqwc = (self.sqwc >> 16 & 0xFF).max(1);
                for _ in 0..n {
                    let (at, next) = self.data_address(c);
                    if self.reads_memory(c) {
                        let Some(q) = self.read_mem(at, mem, spr) else { return self.bus_error(c) };
                        if c == TO_SPR { self.spr_write(c, q, spr) } else { dev.put(c, q, false) }
                    } else {
                        let q = if c == FROM_SPR { self.spr_read(c, spr) } else { dev.get(c) };
                        if !self.write_mem(at, q, mem, spr) {
                            return self.bus_error(c);
                        }
                    }
                    let ch = &mut self.ch[c];
                    ch.madr = ch.madr & SPR_BIT | next & !SPR_BIT;
                    ch.qwc -= 1;
                    if interleave {
                        ch.block += 1;
                        if ch.block == tqwc {
                            ch.block = 0;
                            ch.madr = ch.madr.wrapping_add((self.sqwc & 0xFF) * 16);
                        }
                    }
                }
                if self.sts() == Some(c) && self.ch[c].stall {
                    self.stadr = self.ch[c].madr & ADDR;
                }
                self.settle(c);
            }
        }
    }

    /// A transfer that has moved its last qword ends at once (normal and interleave modes, §5.4); a chain's packet
    /// that has moved its last qword is acted on at once (`end_packet`).
    fn settle(&mut self, c: usize) {
        let mode = self.mode(c);
        if self.ch[c].qwc == 0 && (mode == Mode::Normal || mode == Mode::Interleave) && self.ch[c].str() {
            self.finish(c);
        } else {
            self.end_packet(c);
        }
    }

    /// A chain's packet with no data left: TADR after its data where the tag says so, then the end of the chain, or
    /// the tag interrupt's stop with TADR at the next tag (§5.4: IRQ with TIE).
    fn end_packet(&mut self, c: usize) {
        let ch = &mut self.ch[c];
        if !ch.str() || !ch.open || ch.qwc != 0 {
            return;
        }
        ch.open = false;
        if ch.tadr_follows {
            ch.tadr = ch.madr;
        }
        if ch.ends || ch.chcr & chcr::TIE != 0 && ch.chcr & 1 << 31 != 0 {
            self.finish(c);
        }
    }

    /// Reads a DMAtag: from TADR in a source chain, from the peripheral (fromSPR: the scratchpad at SADR) in a
    /// destination chain, and acts on its ID (§5.2.3, §5.2.4, §5.6); an undefined one ends the transfer (D-37).
    fn read_tag(&mut self, c: usize, mem: &mut dyn Memory, spr: &mut [u8], dev: &mut dyn Peripherals) {
        let source = self.mode(c) == Mode::SourceChain;
        let q = if source {
            let Some(q) = self.read_mem(self.ch[c].tadr, mem, spr) else { return self.bus_error(c) };
            q
        } else if c == FROM_SPR {
            self.spr_read(c, spr)
        } else {
            dev.get_tag(c)
        };
        let tag = q as u64;
        let id = (tag >> 28 & 7) as u32;
        let qwc = (tag & 0xFFFF) as u32;
        let addr = (tag >> 32) as u32 & ADDR | if tag >> 63 != 0 { SPR_BIT } else { 0 };
        let ch = &mut self.ch[c];
        ch.chcr = ch.chcr & 0xFFFF | (tag as u32) & 0xFFFF_0000;
        match tag >> 26 & 3 {
            2 => self.pcr &= !PCE,
            3 => self.pcr |= PCE,
            _ => {}
        }
        if !source {
            if !matches!(id, id::CNTS | id::CNT | id::END) {
                return self.finish(c);
            }
            let ch = &mut self.ch[c];
            ch.madr = if c == FROM_SPR { addr & !SPR_BIT } else { addr };
            ch.qwc = qwc;
            ch.open = true;
            ch.ends = id == id::END;
            ch.stall = id == id::CNTS;
            ch.tadr_follows = false;
            ch.in_ring = false;
            return self.end_packet(c);
        }
        let stack = matches!(c, VIF0 | VIF1 | GIF);
        let defined = match id {
            id::CALL | id::RET => stack,
            _ => true,
        };
        if !defined {
            return self.finish(c);
        }
        let tadr = self.ch[c].tadr;
        if id == id::CALL && self.ch[c].asp() == 2 {
            return self.finish(c);
        }
        if self.ch[c].chcr & chcr::TTE != 0 {
            if c == TO_SPR { self.spr_write(c, q, spr) } else { dev.put(c, q, true) }
        }
        let after = tadr.wrapping_add(16) & (ADDR | SPR_BIT);
        let ch = &mut self.ch[c];
        ch.qwc = qwc;
        ch.open = true;
        ch.ends = false;
        ch.stall = id == id::REFS && matches!(c, VIF1 | GIF | SIF1);
        ch.tadr_follows = false;
        ch.in_ring = !matches!(id, id::REF | id::REFS | id::REFE);
        match id {
            id::REFE => {
                ch.madr = addr;
                ch.tadr = after;
                ch.ends = true;
            }
            id::CNT => {
                ch.madr = after;
                ch.tadr_follows = true;
            }
            id::NEXT => {
                ch.madr = after;
                ch.tadr = addr;
            }
            id::REF | id::REFS => {
                ch.madr = addr;
                ch.tadr = after;
            }
            id::CALL => {
                ch.madr = after;
                let n = ch.asp();
                ch.asr[n as usize] = after.wrapping_add(qwc * 16);
                ch.set_asp(n + 1);
                ch.tadr = addr;
            }
            id::RET => {
                ch.madr = after;
                let n = ch.asp();
                if n > 0 {
                    ch.set_asp(n - 1);
                    ch.tadr = ch.asr[n as usize - 1];
                } else {
                    ch.ends = true;
                }
            }
            _ => {
                ch.madr = after;
                ch.ends = true;
            }
        }
        self.end_packet(c);
    }

    pub fn read(&self, paddr: u32) -> u32 {
        match paddr {
            D_CTRL => self.ctrl,
            D_STAT => self.stat,
            D_PCR => self.pcr,
            D_SQWC => self.sqwc,
            D_RBSR => self.rbsr,
            D_RBOR => self.rbor,
            D_STADR => self.stadr,
            D_ENABLER => self.enable,
            _ => match decode(paddr) {
                Some((c, reg)) => {
                    let ch = &self.ch[c];
                    match reg {
                        0x00 => ch.chcr,
                        0x10 => ch.madr,
                        0x20 => ch.qwc,
                        0x30 => ch.tadr,
                        0x40 => ch.asr[0],
                        0x50 => ch.asr[1],
                        0x80 => ch.sadr,
                        _ => 0,
                    }
                }
                None => 0,
            },
        }
    }

    /// A register write; the DMAC is woken at `now` to arbitrate again.
    pub fn write(&mut self, paddr: u32, v: u32, now: u64) {
        match paddr {
            D_CTRL => self.ctrl = v & 0x7FF,
            D_STAT => self.stat = (self.stat & !(v & stat::CLEARED)) ^ (v & stat::MASKS),
            D_PCR => self.pcr = v & (0x3FF | 0x3FF << 16 | PCE),
            D_SQWC => self.sqwc = v,
            D_RBSR => self.rbsr = v & ADDR,
            D_RBOR => self.rbor = v & ADDR,
            D_STADR => self.stadr = v & ADDR,
            D_ENABLEW => self.enable = v,
            _ => {
                if let Some((c, reg)) = decode(paddr) {
                    let spr_fixed = c == FROM_SPR || c == TO_SPR;
                    let ch = &mut self.ch[c];
                    match reg {
                        0x00 => {
                            let was = ch.str();
                            ch.chcr = if was { ch.chcr & !chcr::STR | v & chcr::STR } else { v & chcr::BITS };
                            if !was && ch.str() {
                                self.kick(c);
                            }
                        }
                        0x10 => ch.madr = v & if spr_fixed { ADDR } else { ADDR | SPR_BIT },
                        0x20 => ch.qwc = v & 0xFFFF,
                        0x30 => ch.tadr = v & (ADDR | SPR_BIT),
                        0x40 => ch.asr[0] = v & (ADDR | SPR_BIT),
                        0x50 => ch.asr[1] = v & (ADDR | SPR_BIT),
                        0x80 => ch.sadr = v & 0x3FF0,
                        _ => {}
                    }
                }
            }
        }
        self.wake(now);
    }
}

/// The channel register at `paddr`: TADR on channels 0, 1, 2, 4, 6 and 9, ASR0 and ASR1 on 0 to 2, SADR on 8 and 9
/// (§5.9's table); any other offset is reserved.
pub fn decode(paddr: u32) -> Option<(usize, u32)> {
    let c = BASES.iter().position(|&b| paddr & !0x3FF == b)?;
    let reg = paddr & 0x3FF;
    let ok = match reg {
        0x00 | 0x10 | 0x20 => true,
        0x30 => matches!(c, VIF0 | VIF1 | GIF | TO_IPU | SIF1 | TO_SPR),
        0x40 | 0x50 => c <= GIF,
        0x80 => c == FROM_SPR || c == TO_SPR,
        _ => false,
    };
    ok.then_some((c, reg))
}
