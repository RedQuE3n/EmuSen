//! A cartridge on its board (Endou_Native.md §9.3): the ROM, the cartridge's RAM and the mapper's registers, read and
//! written at the Z80's addresses. The slots' ROM offsets are kept worked out, so a read is an index and a mask.

use crate::board::{Board, Mapper};

const SLOT: usize = 0x4000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cartridge {
    pub board: Board,
    pub rom: Vec<u8>,
    pub ram: Vec<u8>,
    /// The mapper's registers as written: Sega's $FFFC-$FFFF, or Codemasters' three slots in 1-3.
    pub regs: [u8; 4],
    /// Each 16 KiB slot's ROM offset, its bank already taken modulo the ROM's banks.
    pub slots: [usize; 3],
    /// Whether the RAM has been written since the battery file was last saved.
    pub dirty: bool,
}

impl Cartridge {
    pub fn new(board: Board, rom: Vec<u8>) -> Cartridge {
        let regs = match board.mapper {
            // The 315-5235's power-on values [R SMS Power!'s "Mappers"]; the older chips' are undefined, taken the same.
            Mapper::Sega => [0, 0, 1, 2],
            // Banks 0, 1 and 0 [R the same page].
            Mapper::Codemasters => [0, 0, 1, 0],
            _ => [0; 4],
        };
        let mut c = Cartridge { board, rom, ram: vec![0; board.ram], regs, slots: [0, SLOT, 2 * SLOT], dirty: false };
        c.remap();
        c
    }

    fn banks(&self) -> usize {
        self.rom.len().div_ceil(SLOT).max(1)
    }

    /// A bank's offset: a bank number past the ROM's end wraps, as the unused high bits of a power-of-two ROM do; an
    /// image of another size wraps at its own end [A].
    fn bank(&self, n: usize) -> usize {
        n % self.banks() * SLOT
    }

    /// The slots' offsets from the registers. Sega's bank shift is added to every bank when one of $FFFD-$FFFF is
    /// written, so it is applied by `write`, not here [R "Mappers", "ROM bank shifting"].
    fn remap(&mut self) {
        match self.board.mapper {
            Mapper::Sega | Mapper::Codemasters => {
                self.slots = [self.bank(self.regs[1] as usize), self.bank(self.regs[2] as usize), self.bank(self.regs[3] as usize)];
            }
            _ => {}
        }
    }

    /// Sega's shift of bank numbers by $FFFC's low bits: none, $18, $10 or $08.
    fn shift(&self) -> u8 {
        [0x00, 0x18, 0x10, 0x08][(self.regs[0] & 3) as usize]
    }

    fn rom_at(&self, offset: usize) -> u8 {
        if self.rom.is_empty() { 0xFF } else { self.rom[offset % self.rom.len()] }
    }

    /// The byte at `a`, below $C000, or none where the board drives nothing; an empty slot drives nothing anywhere.
    pub fn read(&self, a: u16) -> Option<u8> {
        if self.rom.is_empty() {
            return None;
        }
        let a = a as usize;
        Some(match self.board.mapper {
            Mapper::Plain => self.rom_at(a),
            Mapper::SgRam if a >= 0x8000 && !self.ram.is_empty() => self.ram[(a - 0x8000) % self.ram.len()],
            Mapper::SgRam => self.rom_at(a),
            Mapper::Sega => match a {
                0..0x400 => self.rom_at(a),
                0x8000.. if self.sega_ram_in_slot_2() => self.ram[self.ram_bank() + (a & 0x3FFF) % self.ram_half()],
                _ => self.rom_at(self.slots[a / SLOT] + (a & 0x3FFF)),
            },
            Mapper::Codemasters => match a {
                0xA000.. if self.codemasters_ram() => self.ram[(a & 0x1FFF) % self.ram.len()],
                _ => self.rom_at(self.slots[a / SLOT] + (a & 0x3FFF)),
            },
        })
    }

    fn sega_ram_in_slot_2(&self) -> bool {
        self.regs[0] & 0x08 != 0 && !self.ram.is_empty()
    }

    fn codemasters_ram(&self) -> bool {
        self.regs[2] & 0x80 != 0 && !self.ram.is_empty()
    }

    /// The RAM bank slot 2 shows: $FFFC's bit 2 chooses the second 16 KiB; a smaller RAM mirrors.
    fn ram_bank(&self) -> usize {
        if self.regs[0] & 0x04 != 0 && self.ram.len() > SLOT { SLOT } else { 0 }
    }

    fn ram_half(&self) -> usize {
        self.ram.len().min(SLOT)
    }

    /// Whether Sega's mapper puts the cartridge's RAM over $C000-$FFFF ($FFFC bit 4), its first bank [A]; no known
    /// software does [R "Mappers"].
    pub fn ram_over_work_ram(&self) -> bool {
        self.board.mapper == Mapper::Sega && self.regs[0] & 0x10 != 0 && !self.ram.is_empty()
    }

    pub fn read_high(&self, a: u16) -> u8 {
        self.ram[(a as usize & 0x3FFF) % self.ram_half()]
    }

    /// A write anywhere in the address space: the board takes what its registers and RAM decode, and ignores the rest.
    /// Sega's mapper sees $FFFC-$FFFF whatever else is there, since the console's RAM keeps a copy [R `smstech` §2].
    pub fn write(&mut self, a: u16, v: u8) {
        match self.board.mapper {
            Mapper::Plain => {}
            Mapper::SgRam => {
                if (0x8000..0xC000).contains(&a) && !self.ram.is_empty() {
                    let n = self.ram.len();
                    self.ram[(a as usize - 0x8000) % n] = v;
                    self.dirty = true;
                }
            }
            Mapper::Sega => match a {
                0xFFFC => self.regs[0] = v,
                0xFFFD..=0xFFFF => {
                    self.regs[(a - 0xFFFC) as usize] = v;
                    let s = self.shift();
                    self.slots = [1, 2, 3].map(|i| self.bank(self.regs[i].wrapping_add(s) as usize));
                }
                0x8000..0xC000 if self.sega_ram_in_slot_2() => {
                    let at = self.ram_bank() + (a as usize & 0x3FFF) % self.ram_half();
                    self.ram[at] = v;
                    self.dirty = true;
                }
                0xC000.. if self.ram_over_work_ram() => {
                    let at = (a as usize & 0x3FFF) % self.ram_half();
                    self.ram[at] = v;
                    self.dirty = true;
                }
                _ => {}
            },
            Mapper::Codemasters => match a {
                0xA000..0xC000 if self.codemasters_ram() => {
                    let n = self.ram.len();
                    self.ram[(a as usize & 0x1FFF) % n] = v;
                    self.dirty = true;
                }
                0..0xC000 => {
                    self.regs[1 + a as usize / SLOT] = v;
                    self.remap();
                }
                _ => {}
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{CODEMASTERS_RAM, SEGA_RAM, Why};

    /// A ROM of `banks` 16 KiB banks, each byte its bank number and each bank's first byte past the first KiB $Bn.
    fn rom(banks: usize) -> Vec<u8> {
        let mut r: Vec<u8> = (0..banks * SLOT).map(|i| (i / SLOT) as u8).collect();
        for b in 0..banks {
            r[b * SLOT + 0x400] = 0xB0 | b as u8;
        }
        r
    }

    fn cart(mapper: Mapper, banks: usize, ram: usize) -> Cartridge {
        Cartridge::new(Board { mapper, ram, battery: false, why: Why::Size }, rom(banks))
    }

    fn slots(c: &Cartridge) -> [u8; 3] {
        [c.read(0x0401).unwrap(), c.read(0x4001).unwrap(), c.read(0x8001).unwrap()]
    }

    #[test]
    fn sega_s_mapper_banks_each_slot_and_keeps_the_first_kilobyte() {
        let mut c = cart(Mapper::Sega, 32, SEGA_RAM);
        assert_eq!(slots(&c), [0, 1, 2], "the power-on banks");
        c.write(0xFFFD, 5);
        c.write(0xFFFE, 6);
        c.write(0xFFFF, 31);
        assert_eq!(slots(&c), [5, 6, 31]);
        assert_eq!((c.read(0x0000), c.read(0x03FF), c.read(0x0400)), (Some(0), Some(0), Some(0xB5)), "$0000-$03FF stays bank 0");
        c.write(0xFFFF, 33);
        assert_eq!(slots(&c)[2], 1, "a bank past the ROM wraps");
        c.write(0x8000, 0x77);
        assert_eq!(c.read(0x8000), Some(1), "ROM is not written");
    }

    #[test]
    fn sega_s_ram_takes_slot_2_in_either_bank() {
        let mut c = cart(Mapper::Sega, 8, SEGA_RAM);
        c.write(0xFFFC, 0x08);
        c.write(0x8000, 0x11);
        c.write(0xBFFF, 0x22);
        assert_eq!((c.read(0x8000), c.read(0xBFFF)), (Some(0x11), Some(0x22)));
        c.write(0xFFFC, 0x0C);
        assert_eq!(c.read(0x8000), Some(0), "the second bank");
        c.write(0x8000, 0x33);
        assert_eq!((c.ram[0], c.ram[SLOT]), (0x11, 0x33));
        c.write(0xFFFC, 0x00);
        assert_eq!(c.read(0x8001), Some(2), "ROM again; the RAM keeps its bytes");
        assert_eq!(c.ram[0], 0x11);
        assert!(c.dirty);
        let mut small = cart(Mapper::Sega, 8, 0x2000);
        small.write(0xFFFC, 0x08);
        small.write(0x8000, 0x44);
        assert_eq!((small.read(0xA000), small.read(0x8000)), (Some(0x44), Some(0x44)), "8 KiB mirrors through the slot");
    }

    #[test]
    fn sega_s_shift_applies_when_a_bank_is_next_written() {
        let mut c = cart(Mapper::Sega, 32, 0);
        c.write(0xFFFF, 2);
        c.write(0xFFFC, 0x01);
        assert_eq!(slots(&c), [0, 1, 2], "no effect until a bank register is written");
        c.write(0xFFFE, 1);
        assert_eq!(slots(&c), [0x18, 0x19, 0x1A], "all three shifted by $18");
        c.write(0xFFFC, 0x03);
        c.write(0xFFFD, 0);
        assert_eq!(slots(&c), [0x08, 0x09, 0x0A]);
    }

    #[test]
    fn codemasters_mapper_banks_every_slot_by_its_own_addresses() {
        let mut c = cart(Mapper::Codemasters, 16, CODEMASTERS_RAM);
        assert_eq!(slots(&c), [0, 1, 0], "banks 0, 1 and 0");
        c.write(0x0000, 3);
        c.write(0x7FFF, 4);
        c.write(0x8000, 15);
        assert_eq!(slots(&c), [3, 4, 15]);
        assert_eq!(c.read(0x0000), Some(3), "the first kilobyte is not kept");
        c.write(0x4000, 0x85);
        assert_eq!(slots(&c)[1], 5, "the RAM bit is not a bank bit");
        c.write(0xA000, 0x99);
        assert_eq!((c.read(0xA000), c.read(0x8001)), (Some(0x99), Some(15)), "RAM over $A000-$BFFF, ROM under it");
        c.write(0x4000, 0x05);
        assert_eq!(c.read(0xA000), Some(15));
        c.write(0xFFFF, 1);
        assert_eq!(slots(&c), [3, 5, 15], "Sega's registers mean nothing here");
    }

    #[test]
    fn plain_and_sg_1000_boards_mirror_what_they_hold() {
        let p = cart(Mapper::Plain, 2, 0);
        assert_eq!((p.read(0x0000), p.read(0x4000), p.read(0x8000)), (Some(0), Some(1), Some(0)));
        let mut s = Cartridge::new(Board { mapper: Mapper::SgRam, ram: 0x800, battery: false, why: Why::Table }, rom(2));
        s.write(0x8000, 0x5A);
        assert_eq!((s.read(0x8800), s.read(0xBFFF - 0x7FF), s.read(0x4000)), (Some(0x5A), Some(0x5A), Some(1)));
        let mut e = Cartridge::new(Board { mapper: Mapper::Plain, ram: 0, battery: false, why: Why::Size }, vec![0x12; 0x2000]);
        e.write(0xFFFF, 1);
        assert_eq!((e.read(0x2000), e.read(0xBFFF)), (Some(0x12), Some(0x12)));
    }
}
