//! A cartridge on its board (Endou_Native.md §9.3): the ROM, the cartridge's RAM and the mapper's registers, read and
//! written at the Z80's addresses. The slots' ROM offsets are kept worked out, so a read is an index and a mask.

use crate::board::{Board, Mapper};
use crate::eeprom::Eeprom;

const SLOT: usize = 0x4000;

/// The EEPROM board's register and its enable, $FFFC's bit 4 [R Charles MacDonald, SMS Power! forum topic 770, 1999-12-24].
pub const EEPROM_REGISTER: u16 = 0x8000;
pub const EEPROM_ENABLE: u8 = 0x10;

/// The register's bits for the chip's pins: DI on bit 0 [R the same post]; CLK on bit 1 and CS on bit 2, and DO read
/// back on bit 0, DI and DO tied as the data sheet allows (§3.2), are argued (Endou_Disputes.md D-26).
pub const EEPROM_DI: u8 = 1 << 0;
pub const EEPROM_CLK: u8 = 1 << 1;
pub const EEPROM_CS: u8 = 1 << 2;

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
    /// The EEPROM board's chip, its memory `ram`, and the register's last byte; idle on every other board.
    pub eeprom: Eeprom,
    pub eeprom_register: u8,
}

impl Cartridge {
    pub fn new(board: Board, rom: Vec<u8>) -> Cartridge {
        let regs = match board.mapper {
            // The 315-5235's power-on values [R SMS Power!'s "Mappers"]; the older chips' are undefined, taken the same.
            Mapper::Sega | Mapper::SegaEeprom => [0, 0, 1, 2],
            // Banks 0, 1 and 0 [R the same page].
            Mapper::Codemasters => [0, 0, 1, 0],
            _ => [0; 4],
        };
        // A 93C46B's cells read as erased, all ones, before anything is written [A].
        let fill = if board.mapper == Mapper::SegaEeprom { 0xFF } else { 0 };
        let mut c = Cartridge { board, rom, ram: vec![fill; board.ram], regs, slots: [0, SLOT, 2 * SLOT], dirty: false, eeprom: Eeprom::default(), eeprom_register: 0 };
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
            Mapper::Sega | Mapper::SegaEeprom | Mapper::Codemasters => {
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
            Mapper::Sega | Mapper::SegaEeprom => match a {
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
        self.board.mapper == Mapper::Sega && self.regs[0] & 0x08 != 0 && !self.ram.is_empty()
    }

    /// Whether `a` is the EEPROM board's register, enabled.
    pub fn eeprom_selected(&self, a: u16) -> bool {
        self.board.mapper == Mapper::SegaEeprom && a == EEPROM_REGISTER && self.regs[0] & EEPROM_ENABLE != 0
    }

    /// The register read at the Z80's `clock`: the byte last written, bit 0 DO where the chip drives it (D-26).
    pub fn eeprom_read(&self, clock: u64) -> u8 {
        let data = self.eeprom.output(clock).unwrap_or(self.eeprom_register & EEPROM_DI != 0);
        (self.eeprom_register & !EEPROM_DI) | data as u8
    }

    /// The register written at `clock`: the chip's pins set from its bits.
    pub fn eeprom_write(&mut self, v: u8, clock: u64) {
        self.eeprom_register = v;
        if self.eeprom.pins(v & EEPROM_CS != 0, v & EEPROM_CLK != 0, v & EEPROM_DI != 0, &mut self.ram, clock) {
            self.dirty = true;
        }
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
    /// The offset in the cartridge's RAM the byte landed at, for a debugger's watches; none where it landed in no RAM.
    pub fn write(&mut self, a: u16, v: u8) -> Option<usize> {
        let at = match self.board.mapper {
            Mapper::Plain => None,
            Mapper::SgRam if (0x8000..0xC000).contains(&a) && !self.ram.is_empty() => Some((a as usize - 0x8000) % self.ram.len()),
            Mapper::SgRam => None,
            Mapper::Sega | Mapper::SegaEeprom => match a {
                0xFFFC => {
                    self.regs[0] = v;
                    None
                }
                0xFFFD..=0xFFFF => {
                    self.regs[(a - 0xFFFC) as usize] = v;
                    let s = self.shift();
                    self.slots = [1, 2, 3].map(|i| self.bank(self.regs[i].wrapping_add(s) as usize));
                    None
                }
                0x8000..0xC000 if self.sega_ram_in_slot_2() => Some(self.ram_bank() + (a as usize & 0x3FFF) % self.ram_half()),
                0xC000.. if self.ram_over_work_ram() => Some((a as usize & 0x3FFF) % self.ram_half()),
                _ => None,
            },
            Mapper::Codemasters => match a {
                0xA000..0xC000 if self.codemasters_ram() => Some((a as usize & 0x1FFF) % self.ram.len()),
                0..0xC000 => {
                    self.regs[1 + a as usize / SLOT] = v;
                    self.remap();
                    None
                }
                _ => None,
            },
        };
        if let Some(at) = at {
            self.ram[at] = v;
            self.dirty = true;
        }
        at
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

    // Jadeite_Native.md §9.3: the EEPROM board is Sega's mapper with no RAM; its register answers at $8000 only while $FFFC's
    // bit 4 is set, and neither RAM bit maps the chip's bytes anywhere.
    #[test]
    fn the_eeprom_board_s_register_answers_only_while_enabled() {
        let mut c = cart(Mapper::SegaEeprom, 16, crate::eeprom::BYTES);
        assert!(c.ram.iter().all(|&b| b == 0xFF), "erased at first");
        c.write(0xFFFF, 5);
        assert_eq!((c.eeprom_selected(0x8000), c.read(0x8001)), (false, Some(5)));
        c.write(0xFFFC, EEPROM_ENABLE | 0x08);
        assert!(c.eeprom_selected(0x8000) && !c.eeprom_selected(0x8001) && !c.ram_over_work_ram());
        assert_eq!(c.read(0x8001), Some(5), "bit 3 maps no RAM on this board");
        c.eeprom_write(EEPROM_CS | EEPROM_DI, 0);
        assert_eq!(c.eeprom_read(0) & EEPROM_DI, EEPROM_DI, "DO high-Z: bit 0 reads the DI written");
        c.eeprom_write(EEPROM_CS, 0);
        assert_eq!(c.eeprom_read(0), EEPROM_CS);
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
