//! The cartridge on the 68000's bus: ROM up to 4 MiB with its mirrors, save RAM on its declared lanes and range, the
//! `$A130F1` register that maps it over ROM, and the Sega mapper of `$A130F3`-`$A130FF`. Nephrite_Native.md §9.

use crate::eeprom::{self, Eeprom};
use crate::media::SaveRam;

pub struct Cart {
    pub rom: Vec<u8>,
    /// The ROM's address lines: its size rounded up to a power of two, less one, so a small ROM mirrors.
    mask: usize,
    /// Save RAM, battery-backed or not; empty when the header declares none.
    pub sram: Vec<u8>,
    save: Option<SaveRam>,
    /// `$A130F1`: bit 0 maps the save RAM over ROM, bit 1 protects it from writes.
    pub sram_reg: u8,
    /// Save RAM above the ROM's end is mapped without the register.
    sram_always: bool,
    /// The Sega mapper's pages for the eight 512 KiB banks, bank 0 fixed at page 0; absent on a plain board.
    pub banks: Option<[u8; 8]>,
    /// A serial EEPROM board's chip, which takes the place of the save RAM its header declares.
    pub eeprom: Option<Eeprom>,
    /// Sonic & Knuckles' lock-on: the cartridge on top, seen at `$200000`, and the 256 KiB patch ROM that `$A130F1`'s
    /// bit 0 maps over `$300000`-`$3FFFFF` (plutiedev's "Sonic & Knuckles Lock-on").
    pub lockon: Option<Box<Cart>>,
    pub patch: Vec<u8>,
    /// Sonic & Knuckles' board, whose upper 2 MiB is the slot on top: nothing there with no cartridge in it.
    slot_on_top: bool,
    /// The battery's bytes have changed since its file was last made.
    dirty: bool,
    /// ROM patches as a Game Genie makes them on the cartridge port, (address, value, compare or `NO_COMPARE`): every
    /// read of the cartridge at the address answers the value, the save RAM's included; the host's, not in the state.
    pub patches: Vec<(u32, u8, u32)>,
}

/// Sonic & Knuckles' serial, whose cartridge takes another on top.
pub const LOCK_ON_SERIAL: &str = "GM MK-1563";

impl Cart {
    /// The board an image's header describes: the mapper for "SEGA SSF" or an image above 4 MiB, a serial EEPROM for
    /// a serial the documents know.
    pub fn new(rom: Vec<u8>, save: Option<SaveRam>, system_type: &str, serial: &str) -> Cart {
        let mask = rom.len().max(2).next_power_of_two() - 1;
        let checksum = rom.get(0x18E..0x190).map_or(0, |c| u16::from_be_bytes([c[0], c[1]]));
        let eeprom = eeprom::board(serial, checksum).map(Eeprom::new);
        let save = if eeprom.is_some() { None } else { save };
        let sram = save.map_or(Vec::new(), |s| vec![0xFF; s.bytes()]);
        let sram_always = save.is_some_and(|s| s.start as usize >= rom.len());
        let mapper = system_type.starts_with("SEGA SSF") || rom.len() > 0x40_0000;
        Cart { rom, mask, sram, save, sram_reg: 0, sram_always, banks: mapper.then_some([0, 1, 2, 3, 4, 5, 6, 7]), eeprom, lockon: None, patch: Vec::new(), slot_on_top: serial.starts_with(LOCK_ON_SERIAL), dirty: true, patches: Vec::new() }
    }

    /// A cartridge locked on top of this one, with Sonic & Knuckles' patch ROM when given. The cartridge on top keeps
    /// its own save RAM, switched by the register Sonic & Knuckles passes on.
    pub fn lock_on(&mut self, top: Vec<u8>, patch: Vec<u8>) {
        let h = crate::media::Header::read(&top, 0x100).unwrap_or_default();
        let mut c = Cart::new(top, h.save, &h.system_type, &h.serial);
        c.sram_always = false;
        self.lockon = Some(Box::new(c));
        self.patch = if patch.len().is_power_of_two() { patch } else { Vec::new() };
    }

    /// Whether the cartridge answers at `a`: not in Sonic & Knuckles' empty slot on top, which reads the open bus.
    pub fn answers(&self, a: u32) -> bool {
        !(self.slot_on_top && self.lockon.is_none() && a >= 0x20_0000)
    }

    /// Whether a battery keeps anything: an EEPROM, battery-backed save RAM, or the cartridge on top's.
    pub fn has_battery(&self) -> bool {
        if let Some(l) = &self.lockon {
            return l.has_battery();
        }
        self.eeprom.is_some() || (self.save.is_some_and(|s| s.battery) && !self.sram.is_empty())
    }

    /// The bytes the battery keeps: the cartridge on top's, the EEPROM's, or the save RAM's.
    pub fn battery(&self) -> &[u8] {
        if let Some(l) = &self.lockon {
            return l.battery();
        }
        self.eeprom.as_ref().map_or(&self.sram, |e| &e.memory)
    }

    pub fn battery_mut(&mut self) -> &mut [u8] {
        if let Some(l) = self.lockon.as_mut() {
            return l.battery_mut();
        }
        self.dirty = true;
        match self.eeprom.as_mut() {
            Some(e) => &mut e.memory,
            None => &mut self.sram,
        }
    }

    /// Where save RAM byte `i` sits in the battery file: its address less the RAM's even start, as Genesis Plus GX files
    /// it (Nephrite_Native.md §33).
    fn file_offset(s: SaveRam, i: usize) -> usize {
        let address = match s.lanes {
            0 => s.start as usize + i,
            3 => (s.start | 1) as usize + 2 * i,
            _ => (s.start & !1) as usize + 2 * i,
        };
        address - (s.start & !1) as usize
    }

    /// The battery from a battery file: Genesis Plus GX's form, the RAM's range by address from its even start in 64 KiB
    /// or more, or the RAM's bytes in order, as BlastEm and Nephrite before 2026-10-06 wrote it; an EEPROM's bytes in order.
    /// A file longer than the RAM is the first, clipped to the RAM.
    pub fn load_battery(&mut self, file: &[u8]) {
        if let Some(l) = self.lockon.as_mut() {
            return l.load_battery(file);
        }
        let by_address = self.save.filter(|_| self.eeprom.is_none() && file.len() > self.sram.len());
        let bytes = self.battery_mut();
        match by_address {
            Some(s) => bytes.iter_mut().enumerate().for_each(|(i, b)| {
                if let Some(&v) = file.get(Self::file_offset(s, i)) {
                    *b = v;
                }
            }),
            None => {
                let n = bytes.len().min(file.len());
                bytes[..n].copy_from_slice(&file[..n]);
            }
        }
    }

    /// The battery file of a save RAM, in Genesis Plus GX's form: 64 KiB, or its range if that is longer, each byte at its
    /// offset from the RAM's even start and `$FF` where the RAM has none; false, and `out` left, for an EEPROM or none.
    pub fn battery_file(&self, out: &mut Vec<u8>) -> bool {
        if let Some(l) = &self.lockon {
            return l.battery_file(out);
        }
        let Some(s) = self.save.filter(|_| self.eeprom.is_none() && !self.sram.is_empty()) else { return false };
        let length = (s.end - (s.start & !1)) as usize + 1;
        out.clear();
        out.resize(length.max(0x1_0000), 0xFF);
        for (i, &b) in self.sram.iter().enumerate() {
            out[Self::file_offset(s, i)] = b;
        }
        true
    }

    /// Whether the battery's bytes have changed since this was last asked.
    pub fn take_dirty(&mut self) -> bool {
        match self.lockon.as_mut() {
            Some(l) => l.take_dirty(),
            None => std::mem::take(&mut self.dirty),
        }
    }

    fn sram_index(&self, a: u32) -> Option<usize> {
        let s = self.save?;
        if self.sram.is_empty() || !(self.sram_always || self.sram_reg & 1 != 0) || a < s.start & !1 || a > s.end {
            return None;
        }
        let i = match s.lanes {
            3 if a & 1 == 1 => (a - (s.start | 1)) / 2,
            2 if a & 1 == 0 => (a - (s.start & !1)) / 2,
            0 => a - s.start,
            _ => return None,
        } as usize;
        (i < self.sram.len()).then_some(i)
    }

    fn rom_byte(&self, a: u32) -> u8 {
        let a = match self.banks {
            Some(b) => (b[(a as usize >> 19) & 7] as usize) << 19 | (a as usize & 0x7_FFFF),
            None => a as usize,
        } & self.mask;
        self.rom.get(a).copied().unwrap_or(0xFF)
    }

    /// A byte as the cartridge port gives it: the board's, or a patch's where one stands at the address and its compare,
    /// if any, is the board's byte.
    pub fn read8(&self, a: u32) -> u8 {
        let v = self.board8(a);
        if self.patches.is_empty() {
            return v;
        }
        let at = a & 0x3F_FFFF;
        self.patches.iter().find(|&&(p, _, c)| p == at && (c == u32::MAX || c as u8 == v)).map_or(v, |&(_, value, _)| value)
    }

    fn board8(&self, a: u32) -> u8 {
        if let Some(l) = self.lockon.as_ref().filter(|_| a >= 0x20_0000) {
            if a >= 0x30_0000 && !self.patch.is_empty() && self.sram_reg & 1 != 0 {
                return self.patch[a as usize & (self.patch.len() - 1)];
            }
            return l.read8(a);
        }
        if let Some(v) = self.eeprom.as_ref().and_then(|e| e.read(a)) {
            return v;
        }
        match self.sram_index(a) {
            Some(i) => self.sram[i],
            None => self.rom_byte(a),
        }
    }

    /// A word: a save RAM on one lane answers its byte and leaves the other lane's ROM byte.
    pub fn read16(&self, a: u32) -> u16 {
        (self.read8(a & !1) as u16) << 8 | self.read8(a | 1) as u16
    }

    pub fn write8(&mut self, a: u32, v: u8) {
        if let Some(l) = self.lockon.as_mut().filter(|_| a >= 0x20_0000) {
            return l.write8(a, v);
        }
        if let Some(e) = self.eeprom.as_mut().filter(|e| e.takes(a)) {
            return e.write(&[(a, v)]);
        }
        if self.sram_reg & 2 != 0 {
            return;
        }
        if let Some(i) = self.sram_index(a) {
            self.sram[i] = v;
            self.dirty = true;
        }
    }

    /// A word: an EEPROM whose lines it reaches sees both bytes in the one bus cycle.
    pub fn write16(&mut self, a: u32, v: u16) {
        if let Some(l) = self.lockon.as_mut().filter(|_| a >= 0x20_0000) {
            return l.write16(a, v);
        }
        let bytes = [(a & !1, (v >> 8) as u8), (a | 1, v as u8)];
        if let Some(e) = self.eeprom.as_mut().filter(|e| bytes.iter().any(|&(b, _)| e.takes(b))) {
            let taken: Vec<(u32, u8)> = bytes.into_iter().filter(|&(b, _)| e.takes(b)).collect();
            return e.write(&taken);
        }
        self.write8(bytes[0].0, bytes[0].1);
        self.write8(bytes[1].0, bytes[1].1);
    }

    /// `$A130F1`-`$A130FF`, odd bytes: the save RAM's register, then the mapper's pages.
    pub fn register(&mut self, a: u32, v: u8) {
        if let Some(l) = self.lockon.as_mut() {
            l.register(a, v);
        }
        match a & 0xFF {
            0xF1 => self.sram_reg = v & 3,
            r @ 0xF3..=0xFF if r & 1 == 1 => {
                if let Some(b) = self.banks.as_mut() {
                    b[((r - 0xF1) / 2) as usize] = v & 0x3F;
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rom(n: usize) -> Vec<u8> {
        (0..n).map(|i| (i >> 8) as u8 ^ i as u8).collect()
    }

    #[test]
    fn a_small_rom_mirrors_and_a_short_one_reads_ff_past_its_end() {
        let c = Cart::new(rom(0x8_0000), None, "SEGA GENESIS", "");
        assert_eq!(c.read8(0x8_1234), c.read8(0x1234));
        let c = Cart::new(rom(0x30_0000), None, "SEGA GENESIS", "");
        assert_eq!(c.read8(0x38_0000), 0xFF);
    }

    #[test]
    fn odd_lane_save_ram_above_the_rom_is_always_mapped() {
        let save = SaveRam { battery: true, lanes: 3, start: 0x20_0001, end: 0x20_3FFF };
        let mut c = Cart::new(rom(0x10_0000), Some(save), "SEGA GENESIS", "");
        assert_eq!(c.sram.len(), 0x2000);
        c.write8(0x20_0001, 0x5A);
        c.write8(0x20_0003, 0xA5);
        c.write8(0x20_0002, 0x11);
        assert_eq!((c.sram[0], c.sram[1]), (0x5A, 0xA5));
        assert_eq!(c.read16(0x20_0002) & 0xFF, 0xA5, "the odd byte is the RAM's");
        c.register(0xA1_30F1, 2);
        c.write8(0x20_0001, 0);
        assert_eq!(c.sram[0], 0x5A, "bit 1 protects it");
    }

    #[test]
    fn save_ram_over_rom_needs_the_register() {
        let save = SaveRam { battery: true, lanes: 3, start: 0x20_0001, end: 0x20_3FFF };
        let mut c = Cart::new(rom(0x30_0000), Some(save), "SEGA GENESIS", "");
        let under = c.read8(0x20_0001);
        c.write8(0x20_0001, !under);
        assert_eq!(c.read8(0x20_0001), under, "unmapped: ROM");
        c.register(0xA1_30F1, 1);
        c.write8(0x20_0001, 0x42);
        assert_eq!(c.read8(0x20_0001), 0x42);
        c.register(0xA1_30F1, 0);
        assert_eq!(c.read8(0x20_0001), under);
    }

    #[test]
    fn the_mapper_pages_512_kib_banks_but_not_the_first() {
        let r = rom(0x50_0000);
        let mut c = Cart::new(r.clone(), None, "SEGA SSF", "");
        assert_eq!(c.read8(0x08_0010), r[0x08_0010]);
        c.register(0xA1_30FF, 9);
        assert_eq!(c.read8(0x38_0010), r[0x48_0010]);
        c.register(0xA1_30F1, 0);
        assert_eq!(c.read8(0x10), r[0x10]);
    }

    #[test]
    fn a_cartridge_on_top_shows_above_2_mib_and_the_register_swaps_in_the_patch() {
        let sk = vec![0x11u8; 0x20_0000];
        let top: Vec<u8> = (0..0x10_0000).map(|i| (i >> 12) as u8).collect();
        let patch = vec![0x77u8; 0x4_0000];
        let mut c = Cart::new(sk, None, "SEGA GENESIS", "GM MK-1563 -00");
        c.lock_on(top.clone(), patch);
        assert_eq!(c.read8(0x1F_FFFF), 0x11);
        assert_eq!(c.read8(0x20_5000), top[0x5000], "a 1 MiB cartridge mirrors into the upper 2 MiB");
        assert_eq!(c.read8(0x30_5000), top[0x5000]);
        c.register(0xA1_30F1, 1);
        assert_eq!(c.read8(0x30_5000), 0x77);
        assert_eq!(c.read8(0x20_5000), top[0x5000]);
    }
}
