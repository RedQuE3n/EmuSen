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
}

/// Sonic & Knuckles' serial, whose cartridge takes another on top.
pub const LOCK_ON_SERIAL: &str = "GM MK-1563";

impl Cart {
    /// The board an image's header describes: the mapper for "SEGA SSF" or an image above 4 MiB, a serial EEPROM for
    /// a serial the documents know.
    pub fn new(rom: Vec<u8>, save: Option<SaveRam>, system_type: &str, serial: &str) -> Cart {
        let mask = rom.len().max(2).next_power_of_two() - 1;
        let eeprom = eeprom::board(serial).map(Eeprom::new);
        let save = if eeprom.is_some() { None } else { save };
        let sram = save.map_or(Vec::new(), |s| vec![0xFF; s.bytes()]);
        let sram_always = save.is_some_and(|s| s.start as usize >= rom.len());
        let mapper = system_type.starts_with("SEGA SSF") || rom.len() > 0x40_0000;
        Cart { rom, mask, sram, save, sram_reg: 0, sram_always, banks: mapper.then_some([0, 1, 2, 3, 4, 5, 6, 7]), eeprom, lockon: None, patch: Vec::new(), slot_on_top: serial.starts_with(LOCK_ON_SERIAL) }
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
        match self.eeprom.as_mut() {
            Some(e) => &mut e.memory,
            None => &mut self.sram,
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

    pub fn read8(&self, a: u32) -> u8 {
        if let Some(l) = self.lockon.as_ref().filter(|_| a >= 0x20_0000) {
            if a >= 0x30_0000 && !self.patch.is_empty() && self.sram_reg & 1 != 0 {
                return self.patch[a as usize & (self.patch.len() - 1)];
            }
            return l.read8(a);
        }
        if let Some(e) = self.eeprom.as_ref().filter(|e| e.board.address == a) {
            return (e.sda() as u8) << e.board.sda;
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
        if let Some(e) = self.eeprom.as_mut().filter(|e| e.board.address == a) {
            return e.write(v);
        }
        if self.sram_reg & 2 != 0 {
            return;
        }
        if let Some(i) = self.sram_index(a) {
            self.sram[i] = v;
        }
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
