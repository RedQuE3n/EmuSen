//! The Sega CD, the Genesis's attachment: its sub CPU (a second Beryl 68000 at 12.5 MHz on its own bus), the gate
//! array's registers on both CPUs' sides, PRG-RAM, Word RAM, backup RAM and the boot ROM's place in the main CPU's map
//! in mode 2. The Mega-CD Hardware Manual (Sega, 1991) §1-§4 is the source; Nephrite_Native.md §50 is the record, with
//! what mcd-verificator's tests settled where the manual is silent.

pub(in crate::mcd) mod asic;
pub(in crate::mcd) mod cdc;
pub(in crate::mcd) mod cdda;
pub(crate) mod cdd;
pub(crate) mod frontend;
pub(in crate::mcd) mod pcm;
pub(in crate::mcd) mod state;
pub(in crate::mcd) mod word;
#[cfg(test)]
mod discs;
#[cfg(test)]
mod verificator;

use beryl_m68k::{Access, Bus, M68000, Size};

use crate::interface::{Answer, Attachment, AttachmentView, Board, Master, SpaceRef, Steps};

use cdc::{Cdc, dest};
use cdd::{Cdd, SECTOR};
use word::{Lands, WordRam};

pub use frontend::{DISC_SIGNATURE, SaveMemory, disc_offset, fingerprint, is_disc, open_disc, system_area};

/// Sub CPU clocks a second: the 50 MHz crystal over 4.
pub(in crate::mcd) const SUB_HZ: u64 = 12_500_000;
/// The stopwatch's and the timer's count, 30.72 µs (the manual's §3-3, §3-4), in sub CPU clocks.
pub(in crate::mcd) const TICK: u64 = 384;
/// RES0 reads 0 for about 100 ms after power-on or a reset of the peripherals (§3-1).
pub(in crate::mcd) const PERIPHERAL_RESET: u64 = SUB_HZ / 10;

pub(in crate::mcd) const PRG_SIZE: usize = 0x8_0000;
pub(in crate::mcd) const BRAM_SIZE: usize = 0x2000;
pub(in crate::mcd) const PCM_RAM_SIZE: usize = 0x1_0000;
pub(in crate::mcd) const BIOS_SIZE: usize = 0x2_0000;

/// The Sega CD's memories by their space ids, from its range (Nephrite_Plan.md §4.3): the backup RAM inside the unit
/// is the last of `MEMORIES`, and the backup RAM cartridge's, when one is plugged in, follows.
pub(in crate::mcd) const MEMORIES: [(u32, &str, usize); 4] = [(17, "PRGRAM", PRG_SIZE), (18, "WORDRAM", 0x4_0000), (19, "PCMRAM", PCM_RAM_SIZE), (20, "BRAM", BRAM_SIZE)];
pub const BRAM_ID: u32 = 20;
/// The sub CPU's processor id, from the Sega CD's range (Nephrite_Attachments.md Q7).
pub const S68K: u32 = 2;
pub const CARTRAM_ID: u32 = 21;

/// The sub CPU and everything on its bus, and the fingerprint of the disc in its drive (0 for none), which a state
/// records and a load is held to.
pub struct MegaCd {
    pub(in crate::mcd) cpu: M68000,
    pub hw: Sub,
    pub(in crate::mcd) disc_print: u64,
}

/// The gate array's registers, both sides'.
#[derive(Clone, Default)]
pub struct Gate {
    /// SRES held at 0 by the main CPU, SBRQ set, and the reset exception owed when SRES is released.
    pub(in crate::mcd) reset_held: bool,
    pub(in crate::mcd) busreq: bool,
    pub(in crate::mcd) reset_due: bool,
    pub(in crate::mcd) leds: u8,
    /// The sub clock from which RES0 reads 1 again.
    pub(in crate::mcd) res0_at: u64,
    /// WP7-WP0, and BK1-BK0 of `$A12003`.
    pub(in crate::mcd) wp: u8,
    pub(in crate::mcd) bank: u8,
    pub(in crate::mcd) hivect: u16,
    /// The communication flags (the main CPU's, the sub CPU's), commands and statuses.
    pub cfm: u8,
    pub cfs: u8,
    pub(in crate::mcd) cmd: [u16; 8],
    pub(in crate::mcd) sta: [u16; 8],
    /// The 30.72 µs count at which the stopwatch read 0.
    pub(in crate::mcd) stopwatch: u64,
    /// The timer's value, its count, and the sub clock of its next count.
    pub(in crate::mcd) timer: u8,
    pub(in crate::mcd) timer_count: u8,
    pub(in crate::mcd) timer_next: u64,
    /// IEN1-IEN6 in bits 1-6, and the levels requested and not yet taken.
    pub mask: u8,
    pub(in crate::mcd) pending: u8,
    /// The sub clock at which level 2 was last taken: a step of the sub CPU can run past the main CPU's present, and
    /// a request taken after it was still pending then.
    pub(in crate::mcd) taken2: u64,
    pub(in crate::mcd) font_color: u8,
    pub(in crate::mcd) font_bits: u16,
    /// How far the main CPU is through the forced reset's four writes (§4-1).
    pub(in crate::mcd) forced: u8,
}

/// The sub CPU's side: its clock, its memories and the gate array.
pub struct Sub {
    /// Sub CPU clocks since power-on.
    pub(in crate::mcd) clock: u64,
    /// Sub clocks a master clock, as a fraction.
    ratio: (u64, u64),
    /// The boot ROM, empty when none was given.
    pub(in crate::mcd) bios: Vec<u8>,
    pub prg: Vec<u8>,
    pub(in crate::mcd) word: WordRam,
    pub bram: Vec<u8>,
    pub(in crate::mcd) pcm_ram: Vec<u8>,
    /// The RF5C164.
    pub(in crate::mcd) pcm: pcm::Pcm,
    pub g: Gate,
    /// The sub CPU waits for Word RAM the main CPU holds: its writes owed, done when the RAM is its again.
    pub(in crate::mcd) parked: bool,
    pub(in crate::mcd) owed: Vec<(u32, Size, u16)>,
    /// The drive, the CDC, the sector the drive last read, and the fader's register.
    pub cdd: Cdd,
    pub cdc: Cdc,
    pub(in crate::mcd) raw: Box<[u8; 2352]>,
    pub fader: u16,
    pub(in crate::mcd) cdda: cdda::Cdda,
    /// A backup RAM cartridge in the slot at `$400000`, when one is plugged in.
    pub(in crate::mcd) cart: Option<RamCart>,
    /// The graphics ASIC.
    pub(in crate::mcd) asic: asic::Asic,
    /// The registers' accesses, when a development tool asks for them; never state.
    pub seen: Option<Vec<Seen>>,
}

/// One access at the attachment's register interfaces, as a development tool watches the devices: which side, which
/// register (an offset from `$FF8000`, `$A12000` or, for the PCM chip, `$FF0000`), and the value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Seen {
    pub clock: u64,
    pub what: SeenKind,
    pub reg: u32,
    pub value: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeenKind {
    SubRead,
    SubWrite,
    PcmRead,
    PcmWrite,
    MainRead,
    MainWrite,
    /// The sub CPU took the level in `reg`.
    Taken,
    /// The sub CPU began to wait for Word RAM the main CPU holds, by a read (0) or a write (1).
    Parked,
}

/// A backup RAM cartridge: its RAM on the odd bytes of `$600000`-`$6FFFFF`, its size code on `$400001` (the RAM is
/// 8 KiB shifted left by it), and its write enable on bit 0 of `$700001`, as mcd-verificator's RAM CART reads it.
#[derive(Clone)]
pub(in crate::mcd) struct RamCart {
    pub(in crate::mcd) bytes: Vec<u8>,
    pub(in crate::mcd) writable: bool,
}

impl RamCart {
    /// Sega's 128 KiB cartridge.
    pub(in crate::mcd) const SEGA: usize = 0x2_0000;

    pub(in crate::mcd) fn new(size: usize) -> RamCart {
        RamCart { bytes: vec![0; size], writable: false }
    }

    fn code(&self) -> u8 {
        (self.bytes.len() / 0x2000).trailing_zeros() as u8
    }

    /// A byte of `$400000`-`$7FFFFF`; the even bytes read `$FF`.
    pub(in crate::mcd) fn read8(&self, a: u32) -> u8 {
        if a & 1 == 0 {
            return 0xFF;
        }
        match a {
            0x40_0000..=0x5F_FFFF => self.code(),
            0x60_0000..=0x6F_FFFF => self.bytes[(a as usize - 0x60_0000) >> 1 & (self.bytes.len() - 1)],
            _ => 0xFE | self.writable as u8,
        }
    }

    pub(in crate::mcd) fn write8(&mut self, a: u32, v: u8) {
        if a & 1 == 0 {
            return;
        }
        match a {
            0x60_0000..=0x6F_FFFF if self.writable => {
                let n = self.bytes.len() - 1;
                self.bytes[(a as usize - 0x60_0000) >> 1 & n] = v;
            }
            0x70_0000..=0x7F_FFFF => self.writable = v & 1 != 0,
            _ => {}
        }
    }
}

impl MegaCd {
    /// A Sega CD beside the Genesis `board` describes, at its power-on, with `bios` in its boot ROM and `disc` in its
    /// drive.
    pub(crate) fn new(board: Board, bios: Vec<u8>, mut disc: Option<cdd::Disc>) -> MegaCd {
        let disc_print = disc.as_mut().map_or(0, fingerprint);
        let master_hz = board.master_hz;
        let ratio = (SUB_HZ * master_hz.1, master_hz.0);
        let g = gcd(ratio.0, ratio.1);
        let mut hw = Sub {
            clock: 0,
            ratio: (ratio.0 / g, ratio.1 / g),
            bios,
            prg: vec![0; PRG_SIZE],
            word: WordRam::default(),
            bram: vec![0; BRAM_SIZE],
            pcm_ram: vec![0; PCM_RAM_SIZE],
            pcm: pcm::Pcm::default(),
            // `$A12000` reads `$0002` at power-on: the sub CPU held in reset with its bus requested (§2, the initial values).
            g: Gate { reset_held: true, busreq: true, hivect: 0xFFFF, ..Gate::default() },
            parked: false,
            owed: Vec::new(),
            cdd: Cdd::new(disc, 0),
            cdc: Cdc::default(),
            raw: Box::new([0; 2352]),
            fader: 0,
            cdda: cdda::Cdda::new(master_hz.0, master_hz.1, board.rate),
            cart: None,
            asic: asic::Asic::default(),
            seen: None,
        };
        hw.clock = hw.to_sub(board.start);
        hw.cdd.next = hw.clock + SECTOR[0];
        hw.g.res0_at = hw.clock + PERIPHERAL_RESET;
        hw.word.sub_owns = false;
        MegaCd { cpu: M68000::new(), hw, disc_print }
    }

    // ---- the main CPU's side, mode 2

    /// A word of `$000000`-`$3FFFFF` or `$A12000`-`$A1203F` as the main CPU reads it at master clock `master`; None
    /// for the open bus.
    pub(in crate::mcd) fn main_read16(&mut self, a: u32, master: u64) -> Option<u16> {
        let a = a & 0xFF_FFFE;
        if a >= 0xA1_2000 {
            self.run_to(master);
            let w = self.hw.main_reg(a & 0x3E, master);
            self.hw.see(SeenKind::MainRead, a & 0x3E, w);
            self.hw.cdc_irq();
            return Some(w);
        }
        let h = &self.hw;
        match a {
            0x70 if !h.bios.is_empty() => Some(0xFFFF),
            0x72 if !h.bios.is_empty() => Some(h.g.hivect),
            _ if a < 0x20_0000 && a & 0x2_0000 == 0 => {
                let i = (a & 0x1_FFFF) as usize;
                h.bios.get(i..i + 2).map(|b| u16::from_be_bytes([b[0], b[1]]))
            }
            _ if a < 0x20_0000 => {
                let i = h.prg_window(a)?;
                Some(u16::from_be_bytes([h.prg[i], h.prg[i + 1]]))
            }
            _ => {
                let at = h.word.main(a);
                Some((h.word.read8(at)? as u16) << 8 | h.word.read8(h.word.main(a | 1))? as u16)
            }
        }
    }

    pub(in crate::mcd) fn main_read8(&mut self, a: u32, master: u64) -> Option<u8> {
        if a & 0xFF_FFFF >= 0xA1_2000 {
            self.run_to(master);
            let b = self.hw.main_reg8(a & 0x3F, master);
            self.hw.see(SeenKind::MainRead, a & 0x3F, b as u16);
            self.hw.cdc_irq();
            return Some(b);
        }
        let w = self.main_read16(a & !1, master)?;
        Some(if a & 1 == 0 { (w >> 8) as u8 } else { w as u8 })
    }

    /// A write by the main CPU: `mask` is `$FF00` for a byte at an even address, `$00FF` at an odd one, `$FFFF` for a
    /// word; a byte's value is on both halves of `v`, as the 68000 drives it.
    pub(in crate::mcd) fn main_write(&mut self, a: u32, v: u16, mask: u16, master: u64) {
        let a = a & 0xFF_FFFF;
        if a >= 0xA1_2000 {
            self.run_to(master);
            self.hw.see(SeenKind::MainWrite, a & 0x3E, v & mask);
            self.hw.main_reg_write(a & 0x3E, v, mask, master);
            return;
        }
        let h = &mut self.hw;
        if a < 0x20_0000 {
            if a & 0x2_0000 == 0 {
                return;
            }
            if let Some(i) = h.prg_window(a & !1) {
                write_masked(&mut h.prg[i..i + 2], v, mask);
            }
        } else {
            for (half, m, byte) in [(0u32, 0xFF00u16, (v >> 8) as u8), (1, 0x00FF, v as u8)] {
                if mask & m != 0 {
                    let at = h.word.main((a & !1) | half);
                    h.word.write8(at, byte);
                }
            }
        }
    }
}

/// Whether the Sega CD decodes a main CPU address in mode 2: its boot ROM, PRG-RAM's window and Word RAM below
/// `$400000`, the cartridge slot to `$7FFFFF`, and its registers.
fn decodes(a: u32) -> bool {
    a < 0x80_0000 || (0xA1_2000..=0xA1_203F).contains(&a)
}

impl Attachment for MegaCd {
    const EXPANSION: bool = true;

    fn read8(&mut self, a: u32, at: Master) -> Answer<u8> {
        if !decodes(a) {
            return Answer::Pass;
        }
        // The cartridge slot at `$400000` in mode 2: a backup RAM cartridge, or empty and reading `$FF` (D-32).
        if (0x40_0000..0x80_0000).contains(&a) {
            return Answer::Value(self.hw.cart.as_ref().map_or(0xFF, |c| c.read8(a)));
        }
        self.main_read8(a, at).map_or(Answer::Open, Answer::Value)
    }

    fn read16(&mut self, a: u32, at: Master) -> Answer<u16> {
        if !decodes(a) {
            return Answer::Pass;
        }
        if (0x40_0000..0x80_0000).contains(&a) {
            return Answer::Value(self.hw.cart.as_ref().map_or(0xFFFF, |c| (c.read8(a) as u16) << 8 | c.read8(a | 1) as u16));
        }
        self.main_read16(a, at).map_or(Answer::Open, Answer::Value)
    }

    /// Word RAM answers the VDP a cycle late, so a transfer from it reads the word before the one addressed (the
    /// Mega-CD Hardware Manual's "precautions in using DMA of the VDP").
    fn transfer16(&mut self, a: u32, at: Master) -> Answer<u16> {
        let a = if (0x20_0000..0x24_0000).contains(&a) { 0x20_0000 | (a.wrapping_sub(2) & 0x3_FFFF) } else { a };
        self.read16(a, at)
    }

    fn write(&mut self, a: u32, v: u16, mask: u16, at: Master) -> bool {
        if !decodes(a) {
            return false;
        }
        if (0x40_0000..0x80_0000).contains(&a) {
            if let Some(c) = self.hw.cart.as_mut()
                && mask & 0x00FF != 0
            {
                c.write8(a | 1, v as u8);
            }
            return true;
        }
        self.main_write(a, v, mask, at);
        true
    }

    /// The sub CPU brought up to the main CPU's master clock `master`, instruction by instruction. Kept out of line,
    /// so that the Genesis's loop stays as small as it was.
    #[inline(never)]
    fn run_to(&mut self, master: Master) {
        let target = self.hw.to_sub(master);
        while self.hw.clock < target {
            let h = &mut self.hw;
            h.events();
            if h.g.reset_due && !h.g.reset_held {
                h.g.reset_due = false;
                self.cpu.reset(&mut SubBus(h));
                continue;
            }
            if h.g.reset_held || h.g.busreq || h.parked || self.cpu.halted {
                h.clock = target;
                h.events();
                break;
            }
            if self.cpu.stopped && h.level() <= ((self.cpu.regs.sr >> 8) & 7) as u8 {
                // STOP waits for a request, which only a count or the main CPU makes: the next count, or the target.
                h.clock = target.min(h.next_event()).max(h.clock + 4);
                continue;
            }
            self.cpu.step(&mut SubBus(h));
        }
    }

    /// The PCM chip's and CD-DA's samples, one stream.
    fn sound(&mut self, to: Master) -> Option<&mut Steps> {
        self.hw.sound_to(to);
        Some(&mut self.hw.cdda.steps)
    }
}

impl AttachmentView for MegaCd {
    fn spaces(&self, out: &mut Vec<SpaceRef>) {
        out.extend(MEMORIES.iter().map(|&(id, name, _)| SpaceRef { id, name, read_only: false }));
        if self.hw.cart.is_some() {
            out.push(SpaceRef { id: CARTRAM_ID, name: "CARTRAM", read_only: false });
        }
    }

    fn bytes(&self, id: u32) -> Option<&[u8]> {
        let s = &self.hw;
        Some(match id {
            17 => &s.prg,
            18 => &s.word.bytes,
            19 => &s.pcm_ram,
            BRAM_ID => &s.bram,
            CARTRAM_ID => &s.cart.as_ref()?.bytes,
            _ => return None,
        })
    }

    fn bytes_mut(&mut self, id: u32) -> Option<&mut [u8]> {
        let s = &mut self.hw;
        Some(match id {
            17 => &mut s.prg,
            18 => &mut s.word.bytes,
            19 => &mut s.pcm_ram,
            BRAM_ID => &mut s.bram,
            CARTRAM_ID => &mut s.cart.as_mut()?.bytes,
            _ => return None,
        })
    }
}

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd(b, a % b) }
}

/// `v`'s bytes under `mask` into the two bytes of `b`, big-endian.
fn write_masked(b: &mut [u8], v: u16, mask: u16) {
    if mask & 0xFF00 != 0 {
        b[0] = (v >> 8) as u8;
    }
    if mask & 0x00FF != 0 {
        b[1] = v as u8;
    }
}

impl Sub {
    #[inline]
    fn see(&mut self, what: SeenKind, reg: u32, value: u16) {
        if let Some(v) = self.seen.as_mut() {
            v.push(Seen { clock: self.clock, what, reg, value });
        }
    }

    /// A master clock as a sub clock.
    pub(in crate::mcd) fn to_sub(&self, master: u64) -> u64 {
        (master as u128 * self.ratio.0 as u128 / self.ratio.1 as u128) as u64
    }

    /// A sub clock as the first master clock at or after it.
    pub(in crate::mcd) fn to_master(&self, sub: u64) -> u64 {
        (sub as u128 * self.ratio.1 as u128).div_ceil(self.ratio.0 as u128) as u64
    }

    /// PRG-RAM's byte behind a main CPU address of the window, when the main CPU may reach it: the sub CPU held in
    /// reset or its bus granted.
    fn prg_window(&self, a: u32) -> Option<usize> {
        (self.g.reset_held || self.g.busreq).then_some((self.g.bank as usize) << 17 | (a & 0x1_FFFF) as usize)
    }

    /// Everything timed on the sub side up to the present, in the order it falls: the graphics ASIC's lines (level 1 at
    /// the last), the timer's counts (level 3 at 0), the drive's exchanges and the sectors they read, the decoder
    /// interrupt clearing itself, the CDC's DMA.
    pub(in crate::mcd) fn events(&mut self) {
        loop {
            let (timer, deci, dma) = self.times();
            let t = timer.min(self.cdd.next).min(deci).min(dma).min(self.asic.next);
            if t > self.clock {
                return;
            }
            if t == self.asic.next {
                if self.asic.step(&mut self.word.bytes, self.word.priority) {
                    self.raise(1);
                }
            } else if t == timer {
                if self.g.timer_count == 0 {
                    self.raise(3);
                    self.g.timer_count = self.g.timer;
                } else {
                    self.g.timer_count -= 1;
                }
                self.g.timer_next += TICK;
            } else if t == self.cdd.next {
                self.exchange(t);
            } else if t == deci {
                self.cdc.clear_deci(t);
            } else {
                self.dma_step();
            }
            self.cdc_irq();
        }
    }

    /// The timer's next count, the decoder interrupt's end and the next DMA unit, each `u64::MAX` when none is due.
    fn times(&self) -> (u64, u64, u64) {
        let timer = if self.g.timer != 0 { self.g.timer_next } else { u64::MAX };
        let deci = if self.cdc.deci { self.cdc.deci_until } else { u64::MAX };
        let dma = if self.cdc.busy { self.cdc.dma_next } else { u64::MAX };
        (timer, deci, dma)
    }

    /// The sub clock of the next event that can make a request.
    fn next_event(&self) -> u64 {
        let (timer, deci, dma) = self.times();
        timer.min(self.cdd.next).min(deci).min(dma).min(self.asic.next)
    }

    /// The drive's exchange at `t`: level 4, and the sector it read handed to the CDC.
    fn exchange(&mut self, t: u64) {
        let tick = self.cdd.exchange();
        if tick.int4 {
            self.raise(4);
        }
        let read = match (tick.sector, self.cdd.disc.as_mut()) {
            (Some(at), Some(d)) => Some(d.read(at, &mut self.raw)),
            _ => None,
        };
        let data = read.is_some_and(|k| !k.audio());
        self.cdc.sector(data.then_some(&*self.raw), t);
        let period = SECTOR[self.cdd.phase as usize];
        let (start, end) = (self.to_master(t), self.to_master(t + period));
        let volume = (self.fader as i32 >> 4 & 0x7FF).min(cdda::FULL);
        let sounding = read.is_some_and(|k| k.audio()) && self.cdd.status[0] == cdd::status::PLAYING;
        self.cdda.sector(sounding.then_some(&*self.raw), start, end - start, volume, (self.fader >> 2) as u8 & 3);
        self.cdd.next += period;
        self.cdd.phase = (self.cdd.phase + 1) % 3;
    }

    /// The CDC's INT pin falling: level 5.
    fn cdc_irq(&mut self) {
        if self.cdc.edge() {
            self.raise(5);
        }
    }

    /// One unit of the CDC's DMA, held while its destination is the main CPU's.
    fn dma_step(&mut self) {
        let a = self.cdc.dma as usize;
        let free = match self.cdc.dd {
            dest::PRG => !(self.g.reset_held || self.g.busreq),
            dest::WORD => self.word.one_m || self.word.sub_owns,
            _ => true,
        };
        if !free {
            self.cdc.dma_wait();
            return;
        }
        let (b0, b1) = self.cdc.dma_unit();
        match (self.cdc.dd, b1) {
            (dest::PCM, _) => {
                self.pcm.run(self.clock, &self.pcm_ram);
                self.pcm_ram[self.pcm.window(a >> 1)] = b0;
            }
            (dest::PRG, Some(b1)) => (self.prg[a & 0x7_FFFE], self.prg[(a & 0x7_FFFE) + 1]) = (b0, b1),
            (_, Some(b1)) => {
                self.asic.stolen();
                let i = if self.word.one_m { word::bank_byte(self.word.sub_bank(), a & 0x1_FFFE) } else { a & 0x3_FFFE };
                (self.word.bytes[i], self.word.bytes[i + 1]) = (b0, b1);
            }
            _ => {}
        }
        self.cdc.dma_done(if b1.is_some() { 2 } else { 1 });
    }

    /// A request at `level`, made only while its enable is set.
    pub(in crate::mcd) fn raise(&mut self, level: u8) {
        if self.g.mask & 1 << level != 0 {
            self.g.pending |= 1 << level;
        }
    }

    /// The highest level requested and enabled.
    pub(in crate::mcd) fn level(&self) -> u8 {
        let p = self.g.pending & self.g.mask & 0x7E;
        if p == 0 { 0 } else { 7 - p.leading_zeros() as u8 }
    }

    /// The main CPU's IFL2: a level 2 request, which reads 1 until the sub CPU takes it; a request made while one is
    /// pending is lost (mcd-verificator's IRQ TEST, Nephrite_Native.md §50).
    fn main_int2(&mut self, now: u64) {
        if !self.int2_pending(now) {
            self.raise(2);
        }
    }

    /// Whether a level 2 request stands at sub clock `now`, as the main CPU sees it.
    fn int2_pending(&self, now: u64) -> bool {
        self.g.pending & 4 != 0 || self.g.taken2 > now
    }

    /// The forced reset (§4-1): the sub side's peripherals and registers to their power-on values.
    fn forced_reset(&mut self) {
        let now = self.clock;
        let g = &mut self.g;
        (g.stopwatch, g.timer, g.timer_count, g.mask, g.pending) = (now / TICK, 0, 0, 0, 0);
        (g.font_color, g.font_bits, g.leds) = (0, 0, 0);
        g.res0_at = now + PERIPHERAL_RESET;
        self.cdd.hock = false;
        self.fader = 0;
        self.asic = asic::Asic::default();
    }

    /// The Word RAM the sub CPU waited for is its own again: its owed writes done, and on it goes.
    fn unpark(&mut self) {
        if !self.parked {
            return;
        }
        let owed = std::mem::take(&mut self.owed);
        self.parked = false;
        for (a, size, v) in owed {
            self.sub_write(a, size, v);
            if self.parked {
                return;
            }
        }
    }

    // ---- the main CPU's registers, `$A12000`-`$A1203F`

    fn stopwatch(&self, sub_now: u64) -> u16 {
        ((sub_now / TICK).wrapping_sub(self.g.stopwatch) & 0xFFF) as u16
    }

    fn main_reg(&mut self, off: u32, master: u64) -> u16 {
        if off == 0x08 {
            return self.cdc.host_access(true);
        }
        let g = &self.g;
        match off {
            0x04 => self.cdc.mode(false),
            0x00 => ((g.mask & 4 != 0) as u16) << 15 | (self.int2_pending(self.to_sub(master)) as u16) << 8 | (g.busreq as u16) << 1 | !g.reset_held as u16,
            0x02 => (g.wp as u16) << 8 | (g.bank as u16) << 6 | self.word.mode_bits() as u16,
            0x06 => g.hivect,
            0x0C => self.stopwatch(self.to_sub(master)),
            0x0E => (g.cfm as u16) << 8 | g.cfs as u16,
            0x10..=0x1E => g.cmd[(off as usize - 0x10) / 2],
            0x20..=0x2E => g.sta[(off as usize - 0x20) / 2],
            _ => 0,
        }
    }

    fn main_reg8(&mut self, off: u32, master: u64) -> u8 {
        let w = self.main_reg(off & !1, master);
        if off & 1 == 0 { (w >> 8) as u8 } else { w as u8 }
    }

    fn main_reg_write(&mut self, off: u32, v: u16, mask: u16, master: u64) {
        let forced = std::mem::take(&mut self.g.forced);
        match off {
            0x00 => {
                if mask & 0xFF00 != 0 && v & 0x0100 != 0 {
                    self.main_int2(self.to_sub(master));
                }
                if mask & 0x00FF != 0 {
                    let step = [3, 2, 0];
                    if mask == 0x00FF && (1..=3).contains(&forced) && v as u8 == step[forced as usize - 1] {
                        self.g.forced = forced + 1;
                        if self.g.forced == 4 {
                            self.g.forced = 0;
                            self.forced_reset();
                        }
                    }
                    let (held, busreq) = (v & 1 == 0, v & 2 != 0);
                    if self.g.reset_held && !held {
                        self.g.reset_due = true;
                    }
                    (self.g.reset_held, self.g.busreq) = (held, busreq);
                    self.clock = self.clock.max(self.to_sub(master));
                }
            }
            0x02 => {
                if mask == 0xFFFF && v == 0xFF00 {
                    self.g.forced = 1;
                }
                if mask & 0xFF00 != 0 {
                    self.g.wp = (v >> 8) as u8;
                }
                if mask & 0x00FF != 0 {
                    self.g.bank = (v >> 6) as u8 & 3;
                    self.word.main_dmna(v & 2 != 0);
                    if self.word.sub_owns && !self.word.one_m {
                        self.unpark();
                    }
                }
            }
            0x06 => self.g.hivect = (self.g.hivect & !mask) | (v & mask),
            0x08 => {
                self.cdc.host_access(true);
                self.cdc_irq();
            }
            0x0E => self.g.cfm = if mask == 0x00FF { v as u8 } else { (v >> 8) as u8 },
            0x10..=0x1E => {
                let r = &mut self.g.cmd[(off as usize - 0x10) / 2];
                *r = (*r & !mask) | (v & mask);
            }
            _ => {}
        }
    }

    // ---- the sub CPU's map

    fn font_data(&self, k: u32) -> u16 {
        let (fb, fc) = (self.g.font_bits, self.g.font_color);
        (0..4).fold(0, |w, j| {
            let bit = fb >> (15 - 4 * k - j) & 1 != 0;
            let c = if bit { fc >> 4 } else { fc & 15 } as u16;
            w | c << (12 - 4 * j)
        })
    }

    /// A register read; `low` when the access takes the low byte, which a read of the CDC's register data needs.
    fn sub_reg(&mut self, off: u32, low: bool) -> u16 {
        match off {
            0x06 if low => return self.cdc.read_reg(self.clock) as u16,
            0x08 => return self.cdc.host_access(false),
            _ => {}
        }
        let g = &self.g;
        match off {
            0x04 => self.cdc.mode(true),
            0x0A => (self.cdc.dma >> 3) as u16,
            0x34 => self.fader & 0x7FFC,
            0x36..=0x4A => self.cdd.read(off - 0x36, self.clock),
            0x00 => (g.leds as u16) << 8 | (self.clock >= g.res0_at) as u16,
            0x02 => (g.wp as u16) << 8 | (self.word.priority as u16) << 3 | self.word.mode_bits() as u16,
            0x0C => self.stopwatch(self.clock),
            0x0E => (g.cfm as u16) << 8 | g.cfs as u16,
            0x10..=0x1E => g.cmd[(off as usize - 0x10) / 2],
            0x20..=0x2E => g.sta[(off as usize - 0x20) / 2],
            0x30 => g.timer as u16,
            0x32 => g.mask as u16,
            0x4C => g.font_color as u16,
            0x4E => g.font_bits,
            0x50..=0x56 => self.font_data((off - 0x50) / 2),
            0x58..=0x66 => self.asic.read(off - 0x58),
            _ => 0,
        }
    }

    /// A sub CPU write to its registers; `v` carries a byte on both halves, which the registers that take only words
    /// take whole.
    fn sub_reg_write(&mut self, off: u32, v: u16, mask: u16) {
        match off {
            0x00 => {
                if mask & 0xFF00 != 0 {
                    self.g.leds = (v >> 8) as u8 & 3;
                }
                if mask & 0x00FF != 0 && v & 1 == 0 {
                    self.g.res0_at = self.clock + PERIPHERAL_RESET;
                }
            }
            0x02 => {
                self.word.priority = (v >> 3) as u8 & 3;
                self.word.sub_mode(v & 4 != 0, v & 1 != 0);
            }
            0x04 => self.cdc.write_mode(v, mask),
            0x06 => {
                if mask & 0x00FF != 0 {
                    self.cdc.write_reg(v as u8, self.clock);
                }
            }
            0x08 => {
                self.cdc.host_access(false);
            }
            0x0A => self.cdc.dma = (v as u32) << 3,
            0x0C => self.g.stopwatch = self.clock / TICK,
            0x0E => self.g.cfs = v as u8,
            0x34 => self.fader = v,
            0x36..=0x4A => self.cdd.write(off - 0x36, v, mask, self.clock),
            0x20..=0x2E => {
                let r = &mut self.g.sta[(off as usize - 0x20) / 2];
                *r = (*r & !mask) | (v & mask);
            }
            0x30 => {
                // A write starts the count again from its value, at a fresh 30.72 µs (mcd-verificator's REG 8030).
                (self.g.timer, self.g.timer_count, self.g.timer_next) = (v as u8, v as u8, self.clock + TICK);
            }
            0x32 => {
                if mask & 0x00FF != 0 {
                    self.g.mask = v as u8 & 0x7E;
                    self.g.pending &= self.g.mask;
                }
            }
            0x4C => self.g.font_color = v as u8,
            0x4E => self.g.font_bits = (self.g.font_bits & !mask) | (v & mask),
            0x58..=0x66 => {
                let two_m = !self.word.one_m;
                self.asic.write(off - 0x58, v, mask, self.clock, two_m);
            }
            _ => {}
        }
    }

    fn sub_read(&mut self, a: u32, size: Size) -> u16 {
        let a = a & 0xFF_FFFF;
        let even = a & !1;
        let word = match a {
            0x00_0000..=0x07_FFFF => u16::from_be_bytes([self.prg[even as usize], self.prg[even as usize + 1]]),
            0x08_0000..=0x0D_FFFF => {
                self.asic.stolen();
                let (hi, lo) = (self.word.sub(even), self.word.sub(even | 1));
                if hi == Lands::Elsewhere {
                    self.parked = true;
                    self.see(SeenKind::Parked, 0, 0);
                }
                (self.word.read8(hi).unwrap_or(0) as u16) << 8 | self.word.read8(lo).unwrap_or(0) as u16
            }
            0xFE_0000..=0xFE_FFFF => self.bram[(a as usize & 0x3FFF) >> 1] as u16,
            0xFF_0000..=0xFF_7FFF => {
                let b = self.pcm_read((a & 0x3FFF) >> 1);
                if (a & 0x3FFF) >> 1 < 0x1000 {
                    self.see(SeenKind::PcmRead, (a & 0x3FFF) >> 1, b as u16);
                }
                b as u16
            }
            0xFF_8000..=0xFF_FFFF => {
                let w = self.sub_reg(a & 0x1FE, size == Size::Word || a & 1 == 1);
                self.see(SeenKind::SubRead, a & 0x1FF, w);
                self.cdc_irq();
                w
            }
            _ => 0,
        };
        match size {
            Size::Word => word,
            Size::Byte if a & 1 == 0 => word >> 8,
            Size::Byte => word & 0xFF,
        }
    }

    fn sub_write(&mut self, a: u32, size: Size, v: u16) {
        let a = a & 0xFF_FFFF;
        let (v, mask) = match size {
            Size::Word => (v, 0xFFFF),
            Size::Byte => ((v & 0xFF) << 8 | (v & 0xFF), if a & 1 == 0 { 0xFF00 } else { 0x00FF }),
        };
        let even = a & !1;
        match a {
            0x00_0000..=0x07_FFFF => {
                // WP7-WP0 protect `$000000` up to `WP × $200` from the sub CPU (§3-1).
                if even >= self.g.wp as u32 * 0x200 {
                    write_masked(&mut self.prg[even as usize..even as usize + 2], v, mask);
                }
            }
            0x08_0000..=0x0D_FFFF => {
                self.asic.stolen();
                if self.parked || self.word.sub(even) == Lands::Elsewhere {
                    if !self.parked {
                        self.see(SeenKind::Parked, 1, 0);
                    }
                    self.parked = true;
                    self.owed.push((a, size, v));
                    return;
                }
                if mask & 0xFF00 != 0 {
                    self.word.write8(self.word.sub(even), (v >> 8) as u8);
                }
                if mask & 0x00FF != 0 {
                    self.word.write8(self.word.sub(even | 1), v as u8);
                }
            }
            0xFE_0000..=0xFE_FFFF => {
                if mask & 0x00FF != 0 {
                    self.bram[(a as usize & 0x3FFF) >> 1] = v as u8;
                }
            }
            0xFF_0000..=0xFF_7FFF => {
                if mask & 0x00FF != 0 {
                    if (a & 0x3FFF) >> 1 < 0x1000 {
                        self.see(SeenKind::PcmWrite, (a & 0x3FFF) >> 1, v & 0xFF);
                    }
                    self.pcm_write((a & 0x3FFF) >> 1, v as u8);
                }
            }
            0xFF_8000..=0xFF_FFFF => {
                self.see(SeenKind::SubWrite, a & 0x1FF, v & mask);
                self.sub_reg_write(a & 0x1FE, v, mask);
                self.cdc_irq();
            }
            _ => {}
        }
    }

    // ---- the RF5C164 as the sub CPU reaches it

    /// The PCM chip's register or RAM at its own address `r` (`$FF0000` + 2r + 1): its registers, and the wave RAM's
    /// window of 4 KiB from `$1000` in the bank WB chose (the PCM manual's §4-1). The chip is brought up to the present
    /// first.
    fn pcm_read(&mut self, r: u32) -> u8 {
        self.pcm.run(self.clock, &self.pcm_ram);
        match r {
            0x1000..=0x1FFF => self.pcm_ram[self.pcm.window(r as usize)],
            _ => self.pcm.read(r),
        }
    }

    fn pcm_write(&mut self, r: u32, v: u8) {
        self.pcm.run(self.clock, &self.pcm_ram);
        match r {
            0x1000..=0x1FFF => self.pcm_ram[self.pcm.window(r as usize)] = v,
            _ => self.pcm.write(r, v),
        }
    }

    /// The attachment's sound up to master clock `master`: the PCM chip's samples made, and with CD-DA's handed to
    /// the mix's synthesiser.
    pub(in crate::mcd) fn sound_to(&mut self, master: u64) {
        let now = self.to_sub(master).min(self.clock);
        self.pcm.run(now, &self.pcm_ram);
        let (num, den) = self.ratio;
        for e in self.pcm.out.iter_mut() {
            e.0 = (e.0 as u128 * den as u128).div_ceil(num as u128) as u64;
        }
        self.cdda.feed(master, &mut self.pcm.out);
    }
}

/// The sub CPU's bus: four clocks an access.
pub(in crate::mcd) struct SubBus<'a>(pub &'a mut Sub);

impl Bus for SubBus<'_> {
    fn read(&mut self, a: Access) -> u16 {
        self.0.clock += 4;
        self.0.sub_read(a.address, a.size)
    }

    fn write(&mut self, a: Access, v: u16) {
        self.0.clock += 4;
        self.0.sub_write(a.address, a.size, v);
    }

    fn idle(&mut self, clocks: u32) {
        self.0.clock += clocks as u64;
    }

    fn interrupt_level(&mut self) -> u8 {
        self.0.events();
        self.0.level()
    }

    /// Every level is autovectored, and taking it ends its request.
    fn acknowledge(&mut self, level: u8) -> Option<u8> {
        self.0.clock += 4;
        self.0.g.pending &= !(1 << level);
        self.0.see(SeenKind::Taken, level as u32, 0);
        if level == 2 {
            self.0.g.taken2 = self.0.clock;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A transfer from Word RAM reads the word before the one addressed, which a source set a word later corrects; the main CPU reads it in time (Nephrite_Native.md §57).
    #[test]
    fn a_transfer_from_word_ram_reads_a_word_late() {
        let board = Board { master_hz: (4_725_000_000, 88), pal: false, start: 0, rate: 48_000 };
        let mut m = MegaCd::new(board, vec![0u8; BIOS_SIZE], None);
        m.hw.word.bytes[..4].copy_from_slice(&[1, 2, 3, 4]);
        assert_eq!((m.transfer16(0x20_0002, 0), m.transfer16(0x20_0004, 0)), (Answer::Value(0x0102), Answer::Value(0x0304)));
        assert_eq!(m.read16(0x20_0002, 0), Answer::Value(0x0304), "the CPU reads it in time");
        assert_eq!((m.read16(0xA1_3000, 0), m.write(0xC0_0000, 0, 0xFFFF, 0)), (Answer::Pass, false), "outside its areas");
    }
}
