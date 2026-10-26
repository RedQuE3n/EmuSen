//! The IOP's DMA controller (ps2tek "IOP DMA"; psx-spx "DMA Channels" for the seven channels the PlayStation had): 13
//! channels in burst (with chopping), slice, linked-list and chain modes, DPCR and DPCR2's priorities and enables,
//! DICR and DICR2's flags and the master flag that raises the INTC's IRQ 3, DMACEN and DMACINTEN. A transfer moves in
//! units (a burst, a chopped window, a block, a list node, a chain tag with its data), each holding the bus for a cycle
//! a word; the processor waits while it does. OTC (channel 6) works end to end; the others reach their peripherals
//! through `Peripherals`, which the stages that build them implement (SIF on 2, 9 and 10 at S22, CDVD on 3 at S23,
//! SPU2 on 4 and 7 at S26, SIO2 on 11 and 12 since S25). Times are IOP cycles. Sapphire_Native.md §22.5.

pub const CHANNELS: usize = 13;
pub const MDEC_IN: usize = 0;
pub const MDEC_OUT: usize = 1;
pub const SIF2: usize = 2;
pub const CDVD: usize = 3;
pub const SPU: usize = 4;
pub const PIO: usize = 5;
pub const OTC: usize = 6;
pub const SPU2: usize = 7;
pub const DEV9: usize = 8;
pub const SIF0: usize = 9;
pub const SIF1: usize = 10;
pub const SIO2_IN: usize = 11;
pub const SIO2_OUT: usize = 12;

pub const DPCR: u32 = 0x1F80_10F0;
pub const DICR: u32 = 0x1F80_10F4;
pub const DPCR2: u32 = 0x1F80_1570;
pub const DICR2: u32 = 0x1F80_1574;
pub const DMACEN: u32 = 0x1F80_1578;
pub const DMACINTEN: u32 = 0x1F80_157C;
/// DMACINTEN at power-on: channel interrupts on and IRQ 3 passed to the INTC. No document gives it; the player's BIOS
/// never writes the register and waits on SIF1's completion interrupt, so its bit 0 is set at reset [A, observed at S24;
/// Sapphire_Disputes.md D-344].
pub const DMACINTEN_AT_RESET: u32 = 1;

/// Channel `n`'s register block: MADR, BCR, CHCR and TADR at +0, +4, +8 and +C.
pub const fn base(n: usize) -> u32 {
    if n < 7 { 0x1F80_1080 + 0x10 * n as u32 } else { 0x1F80_1500 + 0x10 * (n as u32 - 7) }
}

/// Dn_CHCR's fields.
pub mod chcr {
    pub const DIR: u32 = 1;
    pub const BACK: u32 = 1 << 1;
    /// Chopping in burst mode; the tag's two words first in chain mode.
    pub const CHOP: u32 = 1 << 8;
    pub const MODE_SHIFT: u32 = 9;
    pub const DMA_WINDOW_SHIFT: u32 = 16;
    pub const CPU_WINDOW_SHIFT: u32 = 20;
    pub const STR: u32 = 1 << 24;
    pub const TRIGGER: u32 = 1 << 28;
    pub const PAUSE: u32 = 1 << 29;
    pub const SNOOP: u32 = 1 << 30;
    /// What a write keeps [A: the bits ps2tek names, iLink's 11 and 31 as storage].
    pub const BITS: u32 = 0xF177_0F03;
    /// OTC's: three bits, its step always backward (psx-spx).
    pub const OTC_BITS: u32 = STR | TRIGGER | SNOOP;
}

/// The master flag's inputs and DICR's fields (psx-spx; ps2tek).
pub mod dicr {
    pub const BUS_ERROR: u32 = 1 << 15;
    pub const MASTER_ENABLE: u32 = 1 << 23;
    pub const MASTER: u32 = 1 << 31;
    /// The per-block enables 0-6, the bus error flag, the masks 16-22 and the master enable.
    pub const WRITABLE: u32 = 0x00FF_807F;
    pub const FLAGS: u32 = 0x7F00_0000;
    /// DICR2: the tag enables that can be set (channels 4, 9 and 10), the masks 16-21 and the flags 24-29.
    pub const WRITABLE2: u32 = 0x003F_0610;
    pub const FLAGS2: u32 = 0x3F00_0000;
}

/// The cycles between two slice blocks or list nodes, the bus let go for the processor's next access [C].
pub const GAP: u64 = 1;
/// The cycles a word costs on any channel [A: psx-spx's 1 clk/word for OTC, MDEC and the GPU; the IOP's channels'
/// own rates are in no public document].
pub const WORD: u64 = 1;
/// The first address no transfer reaches: 8 MiB, the RAM window psx-spx's mirrors fill [A].
pub const LIMIT: u32 = 0x80_0000;

/// What the DMAC reaches past its own registers: a channel's request for up to `want` words in the direction given
/// (DREQ; 0 holds it), a word handed to the peripheral from memory, and a word taken from it to memory, a chain's
/// tags among them.
pub trait Peripherals {
    fn request(&mut self, ch: usize, want: u32, to_memory: bool) -> u32;
    fn put(&mut self, ch: usize, w: u32);
    fn get(&mut self, ch: usize) -> u32;
}

/// IOP RAM by word address, `None` and `false` at an address past the RAM window.
pub trait Memory {
    fn load(&mut self, addr: u32) -> Option<u32>;
    fn store(&mut self, addr: u32, w: u32) -> bool;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Channel {
    pub madr: u32,
    pub bcr: u32,
    pub chcr: u32,
    pub tadr: u32,
    /// Whether the transfer has begun, and its working address and words left in the burst (MADR and BCR being left
    /// as written in burst mode without chopping).
    begun: bool,
    addr: u32,
    left: u32,
    /// In chain mode: a tag has been read and its words are still moving (`addr`, `left`), and its first word.
    in_packet: bool,
    tag: u32,
    /// In chain mode: the packet's words, as its tag gives them.
    size: u32,
}

impl Channel {
    pub fn mode(&self) -> u32 {
        self.chcr >> chcr::MODE_SHIFT & 3
    }

    /// The mode channel `n` runs in: SIF0 and SIF1 always in chain mode, which they "use exclusively" (ps2tek "IOP
    /// DMA"), whatever CHCR's field says; the BIOS starts SIF1 with mode 1 and bit 8, and PCSX2 then reads its tags from
    /// the FIFO as a chain's (Sapphire_Disputes.md D-343).
    pub fn mode_of(&self, n: usize) -> u32 {
        if n == SIF0 || n == SIF1 { 3 } else { self.mode() }
    }

    fn step(&self, n: usize) -> u32 {
        if n == OTC || self.chcr & chcr::BACK != 0 { 4u32.wrapping_neg() } else { 4 }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dmac {
    pub ch: [Channel; CHANNELS],
    pub dpcr: u32,
    pub dicr: u32,
    pub dpcr2: u32,
    pub dicr2: u32,
    pub dmacen: u32,
    pub dmacinten: u32,
    master: bool,
    /// The processor waits until this cycle: the end of the unit holding the bus.
    pub hold: u64,
    /// The next unit may begin from this cycle (after a gap or a chopping window).
    free: u64,
    /// A transfer that ends with the unit holding the bus: its channel and the cycle.
    finish: Option<(usize, u64)>,
    /// Nothing to start until a register is written or a peripheral wakes it.
    idle: bool,
}

impl Default for Dmac {
    fn default() -> Dmac {
        Dmac {
            ch: [Channel::default(); CHANNELS],
            dpcr: 0x0777_7777,
            dicr: 0,
            dpcr2: 0,
            dicr2: 0,
            dmacen: 0,
            dmacinten: DMACINTEN_AT_RESET,
            master: false,
            hold: 0,
            free: 0,
            finish: None,
            idle: true,
        }
    }
}

impl Dmac {
    fn priority_word(&self, n: usize) -> u32 {
        if n < 7 { self.dpcr >> (4 * n) } else { self.dpcr2 >> (4 * (n - 7)) }
    }

    fn enabled(&self, n: usize) -> bool {
        self.priority_word(n) & 8 != 0
    }

    /// The cycle the DMAC next needs attention.
    pub fn next_event(&self) -> u64 {
        let f = self.finish.map_or(u64::MAX, |(_, t)| t);
        if self.idle { f } else { f.min(self.free) }
    }

    /// A peripheral's request changed: arbitrate again.
    pub fn wake(&mut self) {
        self.idle = false;
    }

    /// A peripheral's request changed at cycle `now`: arbitrate again, not before it.
    pub fn wake_from(&mut self, now: u64) {
        if self.idle {
            self.free = self.free.max(now);
        }
        self.idle = false;
    }

    /// The master flag (psx-spx's formula with DICR2's flags, ps2tek): a bus error, or the master enable with any flag,
    /// the flags counting only while DMACINTEN's bit 0 is set.
    fn master_now(&self) -> bool {
        let flags = self.dicr & dicr::FLAGS != 0 || self.dicr2 & dicr::FLAGS2 != 0;
        self.dicr & dicr::BUS_ERROR != 0 || (self.dmacinten & 1 != 0 && self.dicr & dicr::MASTER_ENABLE != 0 && flags)
    }

    /// Recomputes the master flag; whether IRQ 3 is raised, on its rise unless DMACINTEN's bit 1 holds it.
    fn update(&mut self) -> bool {
        let m = self.master_now();
        let rose = m && !self.master;
        self.master = m;
        rose && self.dmacinten & 2 == 0
    }

    /// Channel `n`'s flag, set with its mask whatever DICR's master enable (D-360), or by a tag (`forced`).
    fn flag(&mut self, n: usize, forced: bool) -> bool {
        let (mask, flag) = if n < 7 { (1 << (16 + n), 1 << (24 + n)) } else { (1 << (16 + n - 7), 1 << (24 + n - 7)) };
        let reg = if n < 7 { self.dicr } else { self.dicr2 };
        if forced || reg & mask != 0 {
            if n < 7 { self.dicr |= flag } else { self.dicr2 |= flag }
        }
        self.update()
    }

    /// Whether channel `n` can run a unit: started, enabled, not paused, and triggered or requested; a slice's request
    /// must cover its whole block.
    fn ready(&self, n: usize, dev: &mut dyn Peripherals) -> bool {
        let c = &self.ch[n];
        if c.chcr & chcr::STR == 0 || !self.enabled(n) || self.dmacen & 1 == 0 {
            return false;
        }
        if c.mode_of(n) == 0 && c.chcr & (chcr::TRIGGER | chcr::PAUSE) == chcr::TRIGGER | chcr::PAUSE {
            return false;
        }
        let need = match c.mode_of(n) {
            1 => block(c.bcr),
            3 if !(c.begun && c.in_packet) => chain_tag_words(c).max(1),
            _ => 1,
        };
        c.chcr & chcr::TRIGGER != 0 || (n != OTC && dev.request(n, need, c.chcr & chcr::DIR == 0) >= need)
    }

    /// The ready channel with the best priority, DPCR's and DPCR2's three bits compared on one scale, the higher
    /// channel first on a tie (ps2tek).
    fn pick(&self, dev: &mut dyn Peripherals) -> Option<usize> {
        (0..CHANNELS).rev().filter(|&n| self.ready(n, dev)).min_by_key(|&n| self.priority_word(n) & 7)
    }

    /// Runs what is due by cycle `now`; whether IRQ 3 is raised.
    pub fn step(&mut self, now: u64, mem: &mut dyn Memory, dev: &mut dyn Peripherals) -> bool {
        let mut irq = false;
        if let Some((n, t)) = self.finish
            && t <= now
        {
            self.finish = None;
            irq |= self.complete(n);
        }
        if self.idle || now < self.free || self.finish.is_some() {
            return irq;
        }
        let Some(n) = self.pick(dev) else {
            self.idle = true;
            return irq;
        };
        irq |= self.unit(n, now, mem, dev);
        irq
    }

    /// The end of channel `n`'s transfer: START cleared, its flag; whether IRQ 3 is raised.
    fn complete(&mut self, n: usize) -> bool {
        let c = &mut self.ch[n];
        c.chcr &= !chcr::STR;
        c.begun = false;
        self.idle = false;
        self.flag(n, false)
    }

    /// A bus error: the transfer ends with DICR's bit 15 and no completion flag [A]; whether IRQ 3 is raised.
    fn bus_error(&mut self, n: usize) -> bool {
        let c = &mut self.ch[n];
        c.chcr &= !chcr::STR;
        c.begun = false;
        self.dicr |= dicr::BUS_ERROR;
        self.update()
    }

    /// One word between memory at `addr` and channel `n`'s peripheral (OTC's list entry generated); false on a bus error.
    fn word(&mut self, n: usize, addr: u32, last: bool, mem: &mut dyn Memory, dev: &mut dyn Peripherals) -> bool {
        let addr = addr & 0x00FF_FFFC;
        if addr >= LIMIT {
            return false;
        }
        if self.ch[n].chcr & chcr::DIR != 0 {
            let Some(w) = mem.load(addr) else { return false };
            dev.put(n, w);
            true
        } else {
            let w = if n == OTC { if last { 0x00FF_FFFF } else { addr.wrapping_sub(4) & 0x00FF_FFFF } } else { dev.get(n) };
            mem.store(addr, w)
        }
    }

    /// Holds the bus for `words` from `now`, then lets the next unit begin after `after` more cycles; the unit's end.
    fn hold_for(&mut self, now: u64, words: u64, after: u64) -> u64 {
        let end = now + words * WORD;
        self.hold = end;
        self.free = end + after;
        end
    }

    /// One unit of channel `n` at `now`, its data moved and its registers updated at once (the processor waits until
    /// its end); whether IRQ 3 is raised.
    fn unit(&mut self, n: usize, now: u64, mem: &mut dyn Memory, dev: &mut dyn Peripherals) -> bool {
        let mut c = self.ch[n];
        if !c.begun {
            c.begun = true;
            c.chcr &= !chcr::TRIGGER;
            c.addr = c.madr;
            c.left = block(c.bcr);
            c.in_packet = false;
        }
        let step = c.step(n);
        let device = n != OTC && c.chcr & chcr::TRIGGER == 0;
        let mut irq = false;
        match c.mode_of(n) {
            0 => {
                let chop = c.chcr & chcr::CHOP != 0;
                let window = if chop { 1 << (c.chcr >> chcr::DMA_WINDOW_SHIFT & 7) } else { c.left };
                let mut want = window.min(c.left);
                if device {
                    want = want.min(dev.request(n, want, c.chcr & chcr::DIR == 0));
                }
                self.ch[n] = c;
                for i in 0..want {
                    let last = c.left - i == 1;
                    if !self.word(n, c.addr.wrapping_add(step.wrapping_mul(i)), last, mem, dev) {
                        return self.bus_error(n);
                    }
                }
                c.addr = c.addr.wrapping_add(step.wrapping_mul(want)) & 0x00FF_FFFF;
                c.left -= want;
                if chop {
                    c.madr = c.addr;
                    c.bcr = c.bcr & !0xFFFF | c.left & 0xFFFF;
                }
                let cpu = if chop { 1 << (c.chcr >> chcr::CPU_WINDOW_SHIFT & 7) } else { 0 };
                let end = self.hold_for(now, u64::from(want), cpu);
                if c.left == 0 {
                    self.finish = Some((n, end));
                }
            }
            1 => {
                let want = block(c.bcr);
                self.ch[n] = c;
                for i in 0..want {
                    if !self.word(n, c.madr.wrapping_add(step.wrapping_mul(i)), false, mem, dev) {
                        return self.bus_error(n);
                    }
                }
                c.madr = c.madr.wrapping_add(step.wrapping_mul(want)) & 0x00FF_FFFF;
                let end = self.hold_for(now, u64::from(want), GAP);
                let blocks = (c.bcr >> 16).wrapping_sub(1) & 0xFFFF;
                c.bcr = c.bcr & 0xFFFF | blocks << 16;
                if blocks == 0 {
                    self.finish = Some((n, end));
                } else if n < 7 && self.dicr & 1 << n != 0 {
                    irq |= self.flag(n, false);
                }
            }
            2 => {
                self.ch[n] = c;
                let a = c.madr & 0x00FF_FFFC;
                let Some(header) = (a < LIMIT).then(|| mem.load(a)).flatten() else { return self.bus_error(n) };
                let words = header >> 24;
                for i in 0..words {
                    if !self.word(n, a.wrapping_add(4 * (i + 1)), false, mem, dev) {
                        return self.bus_error(n);
                    }
                }
                c.madr = header & 0x00FF_FFFF;
                let end = self.hold_for(now, u64::from(words) + 1, GAP);
                if c.madr & 0x80_0000 != 0 {
                    self.finish = Some((n, end));
                } else if n < 7 && self.dicr & 1 << n != 0 {
                    irq |= self.flag(n, false);
                }
            }
            _ => {
                self.ch[n] = c;
                let from_memory = c.chcr & chcr::DIR != 0;
                let extra = if c.chcr & chcr::CHOP != 0 { 2 } else { 0 };
                let mut cost = 0;
                if !c.in_packet {
                    let need = chain_tag_words(&c);
                    if need > 0 && dev.request(n, need, !from_memory) < need {
                        self.idle = true;
                        return irq;
                    }
                    let words = if from_memory { 2 + extra } else { 4 };
                    let mut tag = [0u32; 4];
                    let t = c.tadr & 0x00FF_FFFC;
                    for (i, w) in tag.iter_mut().enumerate().take(words) {
                        *w = if from_memory {
                            let a = t + 4 * i as u32;
                            match (a < LIMIT).then(|| mem.load(a)).flatten() {
                                Some(v) => v,
                                None => return self.bus_error(n),
                            }
                        } else {
                            dev.get(n)
                        };
                    }
                    if from_memory {
                        for &w in &tag[2..2 + extra] {
                            dev.put(n, w);
                        }
                        c.tadr = (t + 4 * (2 + extra) as u32) & 0x00FF_FFFF;
                    }
                    c.in_packet = true;
                    c.tag = tag[0];
                    c.addr = tag[0] & 0x00FF_FFFC;
                    c.left = tag[1] & 0x00FF_FFFF;
                    c.size = c.left;
                    c.madr = c.addr;
                    cost += words as u64;
                }
                let words = if c.left == 0 { 0 } else { dev.request(n, c.left, !from_memory) };
                self.ch[n] = c;
                for i in 0..words {
                    if !self.word(n, c.addr.wrapping_add(4 * i), false, mem, dev) {
                        return self.bus_error(n);
                    }
                }
                c.addr = c.addr.wrapping_add(4 * words) & 0x00FF_FFFF;
                c.left -= words;
                c.madr = c.addr;
                cost += u64::from(words);
                if cost == 0 {
                    self.ch[n] = c;
                    self.idle = true;
                    return irq;
                }
                let end = self.hold_for(now, cost, GAP);
                if c.left == 0 {
                    c.in_packet = false;
                    if n == SIF0 && from_memory {
                        for _ in 0..sif0_padding(c.size) {
                            dev.put(n, 0);
                        }
                    }
                    if c.tag & 1 << 30 != 0 && self.dicr2 & 1 << n != 0 {
                        irq |= self.flag(n, true);
                    }
                    if c.tag & 1 << 31 != 0 {
                        self.finish = Some((n, end));
                    }
                }
            }
        }
        self.ch[n] = c;
        irq
    }

    pub fn read(&self, paddr: u32) -> u32 {
        match paddr {
            DPCR => self.dpcr,
            DICR => self.dicr | if self.master { dicr::MASTER } else { 0 },
            DPCR2 => self.dpcr2,
            DICR2 => self.dicr2,
            DMACEN => self.dmacen,
            DMACINTEN => self.dmacinten,
            _ => match decode(paddr) {
                Some((n, 0x0)) => self.ch[n].madr,
                Some((n, 0x4)) => self.ch[n].bcr,
                Some((n, 0x8)) => self.ch[n].chcr,
                Some((n, _)) => self.ch[n].tadr,
                None => 0,
            },
        }
    }

    /// A register write; whether IRQ 3 is raised (DICR's master flag rising, bit 15 written).
    pub fn write(&mut self, paddr: u32, v: u32) -> bool {
        self.idle = false;
        match paddr {
            DPCR => self.dpcr = v,
            DICR => {
                self.dicr = self.dicr & dicr::FLAGS & !v | v & dicr::WRITABLE;
                return self.update();
            }
            DPCR2 => self.dpcr2 = v,
            DICR2 => {
                self.dicr2 = self.dicr2 & dicr::FLAGS2 & !v | v & dicr::WRITABLE2;
                return self.update();
            }
            DMACEN => self.dmacen = v & 1,
            DMACINTEN => {
                self.dmacinten = v & 3;
                return self.update();
            }
            _ => match decode(paddr) {
                Some((n, 0x0)) => self.ch[n].madr = v & 0x00FF_FFFF,
                Some((n, 0x4)) => self.ch[n].bcr = v,
                Some((n, 0x8)) => {
                    let c = &mut self.ch[n];
                    c.chcr = if n == OTC { v & chcr::OTC_BITS | chcr::BACK } else { v & chcr::BITS };
                    if c.chcr & chcr::STR == 0 {
                        c.begun = false;
                        c.in_packet = false;
                    }
                }
                Some((n, _)) => self.ch[n].tadr = v & 0x00FF_FFFF,
                None => {}
            },
        }
        false
    }
}

/// The words a chain's tag takes from or gives to the peripheral: to memory a whole quadword of four, its first two
/// the tag, whatever CHCR's bit 8; from memory the two after the tag with bit 8 (ps2tek "IOP DMA"; Sapphire_Disputes.md
/// D-68, D-143).
fn chain_tag_words(c: &Channel) -> u32 {
    let extra = if c.chcr & chcr::CHOP != 0 { 2 } else { 0 };
    if c.chcr & chcr::DIR != 0 { extra } else { 4 }
}

/// The zero words SIF0 adds after a packet of `words` so that its data fills whole quadwords, which the EE's channel
/// takes as its own tag counts them (EE Overview §2.6, §3.1.2): the BIOS sends packets of six words for two quadwords.
/// PCSX2 fills them with earlier words of the packet; MADR moves by the words read alone (Sapphire_Disputes.md D-345).
fn sif0_padding(words: u32) -> u32 {
    words.next_multiple_of(4) - words
}

/// A burst's words or a slice's block: BCR's low half, 0 meaning 10000h (psx-spx).
fn block(bcr: u32) -> u32 {
    match bcr & 0xFFFF {
        0 => 0x1_0000,
        w => w,
    }
}

/// The channel and register offset at `paddr`.
pub fn decode(paddr: u32) -> Option<(usize, u32)> {
    let n = match paddr {
        0x1F80_1080..0x1F80_10F0 => (paddr - 0x1F80_1080) / 0x10,
        0x1F80_1500..0x1F80_1560 => 7 + (paddr - 0x1F80_1500) / 0x10,
        _ => return None,
    };
    Some((n as usize, paddr & 0xC))
}

/// Whether `paddr` is one of the DMAC's registers.
pub fn is_register(paddr: u32) -> bool {
    decode(paddr).is_some() || matches!(paddr, DPCR | DICR | DPCR2 | DICR2 | DMACEN | DMACINTEN)
}

/// A stand-in for a peripheral the DMAC reaches before its stage exists: it takes every word offered (keeping them when
/// `keep` is set) and gives the words queued in `give`, holding its channel when it has none.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StandIn {
    pub taken: Vec<u32>,
    pub count: u64,
    pub keep: bool,
    pub give: std::collections::VecDeque<u32>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StandIns {
    pub ports: [StandIn; CHANNELS],
}

impl Peripherals for StandIns {
    /// From memory a stand-in always takes; to memory it gives what it holds.
    fn request(&mut self, ch: usize, want: u32, to_memory: bool) -> u32 {
        if to_memory { want.min(self.ports[ch].give.len() as u32) } else { want }
    }

    fn put(&mut self, ch: usize, w: u32) {
        let p = &mut self.ports[ch];
        p.count += 1;
        if p.keep {
            p.taken.push(w);
        }
    }

    fn get(&mut self, ch: usize) -> u32 {
        self.ports[ch].give.pop_front().unwrap_or(0)
    }
}

/// The IOP DMAC's block of `SPH2` (Sapphire_Native.md §38.2): each channel's registers and its transfer in progress,
/// the common registers, the bus's hold and the next unit's start, the transfer ending, and whether it idles.
impl emusen_native::State for Dmac {
    type Error = emusen_native::Truncated;

    fn write_state(&self, w: &mut emusen_native::StateWriter) {
        let regs: [[u32; 8]; CHANNELS] = std::array::from_fn(|n| {
            let c = &self.ch[n];
            [c.madr, c.bcr, c.chcr, c.tadr, c.addr, c.left, c.tag, c.size]
        });
        w.u32s("Channels", regs.as_flattened());
        w.bytes("ChannelBits", &self.ch.map(|c| u8::from(c.begun) | u8::from(c.in_packet) << 1));
        w.u32s("Registers", &[self.dpcr, self.dicr, self.dpcr2, self.dicr2, self.dmacen, self.dmacinten]);
        w.bools("Flags", &[self.master, self.idle, self.finish.is_some()]);
        let (ch, at) = self.finish.unwrap_or((0, 0));
        w.u64s("Times", &[self.hold, self.free, ch as u64, at]);
    }

    fn read_state(&mut self, r: &mut emusen_native::StateReader) -> Result<(), emusen_native::Truncated> {
        let mut regs = [0u32; CHANNELS * 8];
        r.u32s(&mut regs)?;
        let mut bits = [0u8; CHANNELS];
        r.bytes(&mut bits)?;
        for (n, c) in self.ch.iter_mut().enumerate() {
            let g = &regs[8 * n..8 * n + 8];
            (c.madr, c.bcr, c.chcr, c.tadr, c.addr, c.left, c.tag, c.size) = (g[0], g[1], g[2], g[3], g[4], g[5], g[6], g[7]);
            (c.begun, c.in_packet) = (bits[n] & 1 != 0, bits[n] & 2 != 0);
        }
        let mut g = [0u32; 6];
        r.u32s(&mut g)?;
        [self.dpcr, self.dicr, self.dpcr2, self.dicr2, self.dmacen, self.dmacinten] = g;
        let mut f = [false; 3];
        r.bools(&mut f)?;
        let mut t = [0u64; 4];
        r.u64s(&mut t)?;
        (self.master, self.idle, self.hold, self.free) = (f[0], f[1], t[0], t[1]);
        self.finish = (f[2] && (t[2] as usize) < CHANNELS).then_some((t[2] as usize, t[3]));
        Ok(())
    }
}
