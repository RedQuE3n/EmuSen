//! Which board a cartridge image is on (Endou_Plan.md §5.6, Endou_Native.md §9.2): the table of boards keyed by the
//! image's CRC-32 first, then Codemasters' own header, then the image's size. No header names a mapper, so nothing
//! here guesses from what a game writes.

use crate::media::Console;

/// The mapping hardware on a cartridge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mapper {
    /// ROM alone, at most 48 KiB, mirrored over $0000-$BFFF.
    Plain,
    /// Sega's mapper (315-5208, 315-5235, 315-5365): registers at $FFFC-$FFFF, cartridge RAM in two 16 KiB banks.
    Sega,
    /// Codemasters' mapper: a register in each 16 KiB slot, written at the slot's addresses.
    Codemasters,
    /// An SG-1000 board with RAM at $8000-$BFFF, mirrored every `ram` bytes.
    SgRam,
    /// Sega's mapper with a 93C46B serial EEPROM in place of RAM, on a few Game Gear boards (Jadeite_Native.md §9.3).
    SegaEeprom,
}

/// Why a board was chosen, for the log and the tests.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Why {
    Table,
    CodemastersHeader,
    Size,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Board {
    pub mapper: Mapper,
    /// Bytes of RAM on the cartridge, 0 for none.
    pub ram: usize,
    /// Whether that RAM keeps its contents with the power off, and so is a battery file.
    pub battery: bool,
    pub why: Why,
}

/// The largest image a board without a mapper holds.
pub const PLAIN_MAX: usize = 48 * 1024;
/// Sega's mapper's cartridge RAM, its two banks at most [R SMS Power!'s "Mappers"].
pub const SEGA_RAM: usize = 32 * 1024;
/// Codemasters' one RAM board, 8 KiB at $A000 [R SMS Power!'s "Mappers"].
pub const CODEMASTERS_RAM: usize = 8 * 1024;

/// One row of the table: an image by its CRC-32, and its board.
pub struct Row {
    pub crc: u32,
    pub mapper: Mapper,
    pub ram: usize,
    pub battery: bool,
}

const fn battery(crc: u32) -> Row {
    Row { crc, mapper: Mapper::Sega, ram: 8 * 1024, battery: true }
}

/// A Game Gear board with the 93C46B, its 128 bytes the battery file.
const fn eeprom(crc: u32) -> Row {
    Row { crc, mapper: Mapper::SegaEeprom, ram: crate::eeprom::BYTES, battery: true }
}

/// The boards the rules below would place wrongly, each with its source in Endou_Native.md §9.2 and §14.6: Sega boards
/// with 8 KiB of battery RAM, the games SMS Power! tags "Memory Backup" in every release No-Intro's DAT lists, betas,
/// demos and re-releases left out; and The Castle's SG-1000 board with 8 KiB at $8000, as the survey saw it written.
pub const TABLE: &[Row] = &[
    battery(0xC7DE_D988), // Golden Axe Warrior (USA, Europe, Brazil)
    battery(0x4865_1325), // Golfamania (Europe, Brazil)
    battery(0xF97E_9875), // Penguin Land (USA, Europe)
    battery(0x2BCD_B8FA), // Doki Doki Penguin Land - Uchuu Daibouken (Japan)
    battery(0x7597_1BEF), // Phantasy Star (Brazil)
    battery(0x747E_83B5), // Phantasy Star (Korea)
    battery(0x00BE_F1D7), // Phantasy Star (USA, Europe) (Rev A)
    battery(0xE4A6_5E79), // Phantasy Star (USA, Europe)
    battery(0x6605_D36A), // Phantasy Star (Japan)
    battery(0x0730_1F83), // Phantasy Star (Japan) (Rev 1)
    battery(0xB52D_60C8), // Ultima IV - Quest of the Avatar (Europe, Brazil)
    battery(0xB33E_2827), // Ys - The Vanished Omens (USA, Europe, Brazil)
    battery(0x3275_9751), // Ys (Japan)
    battery(0x0E33_3B6E), // Miracle Warriors - Seal of The Dark Lord (USA, Europe, Brazil)
    battery(0xB9FD_F6D9), // Haja no Fuuin (Japan)
    battery(0x6953_8469), // Monopoly (USA, Europe)
    battery(0x026D_94A4), // Monopoly (USA, Europe) (Rev A)
    Row { crc: 0x092F_29D6, mapper: Mapper::SgRam, ram: 8 * 1024, battery: false },
    // The Game Gear's EEPROM boards: the released games of Nerdly Pleasures' list (2021-10), by their CRC-32 in No-Intro's
    // Game Gear DAT as libretro-database carries it (2026.10.07), prototypes left out (Jadeite_Native.md §9.3).
    eeprom(0x056C_AE74), // Hyper Pro Yakyuu '92 (Japan)
    eeprom(0x36EB_CD6D), // Majors, The - Pro Baseball (USA)
    eeprom(0x2DA8_E943), // Pro Yakyuu GG League (Japan)
    eeprom(0x578A_8A38), // World Series Baseball '95 (USA)
    eeprom(0x3D8D_0DD6), // World Series Baseball (USA)
    eeprom(0xBB38_CFD7), // World Series Baseball (USA) (Rev A)
];

/// The board for an image (its copier header already set aside).
pub fn identify(rom: &[u8], console: Console) -> Board {
    identify_with(rom, console, TABLE)
}

pub fn identify_with(rom: &[u8], _console: Console, table: &[Row]) -> Board {
    let crc = crc32(rom);
    if let Some(r) = table.iter().find(|r| r.crc == crc) {
        return Board { mapper: r.mapper, ram: r.ram, battery: r.battery, why: Why::Table };
    }
    if codemasters_header(rom) {
        return Board { mapper: Mapper::Codemasters, ram: CODEMASTERS_RAM, battery: false, why: Why::CodemastersHeader };
    }
    if rom.len() > PLAIN_MAX {
        return Board { mapper: Mapper::Sega, ram: SEGA_RAM, battery: false, why: Why::Size };
    }
    Board { mapper: Mapper::Plain, ram: 0, battery: false, why: Why::Size }
}

/// Codemasters' header at $7FE0 [R SMS Power!'s "Codemasters Header"]: its checksum, the little-endian words of the
/// banks it names summed with Sega's header at $7FF0-$7FFF left out, and the word after it that sums with it to zero.
pub fn codemasters_header(rom: &[u8]) -> bool {
    let Some(h) = rom.get(0x7FE0..0x7FF0) else { return false };
    let banks = h[0] as usize;
    let stated = u16::from_le_bytes([h[6], h[7]]);
    let complement = u16::from_le_bytes([h[8], h[9]]);
    if banks == 0 || banks * 0x4000 > rom.len() || stated.wrapping_add(complement) != 0 || h[10..16] != [0; 6] {
        return false;
    }
    let sum = rom[..banks * 0x4000].chunks(2).enumerate().filter(|(i, _)| !(0x7FF0 / 2..0x8000 / 2).contains(i)).fold(0u16, |s, (_, w)| s.wrapping_add(u16::from_le_bytes([w[0], w[1]])));
    sum == stated
}

/// The CRC-32 (IEEE 802.3, reflected) of the image, the key public per-game listings use.
pub fn crc32(data: &[u8]) -> u32 {
    let mut c = !0u32;
    for &b in data {
        c ^= b as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 { c >> 1 ^ 0xEDB8_8320 } else { c >> 1 };
        }
    }
    !c
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn codemasters(size: usize) -> Vec<u8> {
        crate::programs::images::with_codemasters_header((0..size).map(|i| (i * 7 + i / 256) as u8).collect())
    }

    #[test]
    fn the_crc_is_the_standard_one() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
        assert_eq!(crc32(b""), 0);
    }

    #[test]
    fn a_board_follows_the_table_then_the_header_then_the_size() {
        let b = |rom: &[u8]| identify(rom, Console::Sms);
        assert_eq!(b(&vec![0; 0x8000]), Board { mapper: Mapper::Plain, ram: 0, battery: false, why: Why::Size });
        assert_eq!(b(&vec![0; PLAIN_MAX]).mapper, Mapper::Plain);
        assert_eq!(b(&vec![0; PLAIN_MAX + 1]), Board { mapper: Mapper::Sega, ram: SEGA_RAM, battery: false, why: Why::Size });
        let cm = codemasters(0x20000);
        assert_eq!(b(&cm), Board { mapper: Mapper::Codemasters, ram: CODEMASTERS_RAM, battery: false, why: Why::CodemastersHeader });
        let mut wrong = cm.clone();
        wrong[0x100] ^= 1;
        assert_eq!(b(&wrong).mapper, Mapper::Sega, "a checksum that disagrees is no Codemasters header");
        let mut sega_byte = cm.clone();
        sega_byte[0x7FF5] ^= 1;
        assert_eq!(b(&sega_byte).mapper, Mapper::Codemasters, "Sega's header is outside the sum");
        let row = [Row { crc: crc32(&cm), mapper: Mapper::Sega, ram: 0x2000, battery: true }];
        assert_eq!(identify_with(&cm, Console::Sms, &row), Board { mapper: Mapper::Sega, ram: 0x2000, battery: true, why: Why::Table });
        assert_eq!(b(&[0x5A]).mapper, Mapper::Plain, "a short image has no Codemasters header to read");
        let crcs: Vec<u32> = TABLE.iter().map(|r| r.crc).collect();
        assert!(crcs.iter().enumerate().all(|(i, c)| !crcs[..i].contains(c)), "one row an image");
        assert!(TABLE.iter().all(|r| r.mapper != Mapper::SgRam || r.ram > 0), "an SG-1000 RAM board has RAM");
    }

    // Jadeite_Native.md §9.3: the six released games with the 93C46B, each an EEPROM board of 128 bytes kept as a battery file.
    #[test]
    fn every_released_eeprom_game_has_its_eeprom_board() {
        let crcs = [0x056C_AE74, 0x36EB_CD6D, 0x2DA8_E943, 0x578A_8A38, 0x3D8D_0DD6, 0xBB38_CFD7u32];
        for crc in crcs {
            let row = TABLE.iter().find(|r| r.crc == crc).unwrap_or_else(|| panic!("{crc:08X} has no row"));
            assert_eq!((row.mapper, row.ram, row.battery), (Mapper::SegaEeprom, 128, true), "{crc:08X}");
        }
        assert_eq!(TABLE.iter().filter(|r| r.mapper == Mapper::SegaEeprom).count(), crcs.len());
    }

    // A seeded fault that dropped a row of the table passed every test (Endou_Native.md §14.10): the releases of the eight games
    // SMS Power! tags "Memory Backup", by their CRC-32 in No-Intro's DAT, each a battery board.
    #[test]
    fn every_release_of_a_memory_backup_game_has_its_battery_board() {
        let releases: [(u32, &str); 17] = [
            (0xC7DE_D988, "Golden Axe Warrior"),
            (0x4865_1325, "Golfamania"),
            (0xF97E_9875, "Penguin Land"),
            (0x2BCD_B8FA, "Doki Doki Penguin Land"),
            (0x7597_1BEF, "Phantasy Star (Brazil)"),
            (0x747E_83B5, "Phantasy Star (Korea)"),
            (0x00BE_F1D7, "Phantasy Star (Rev A)"),
            (0xE4A6_5E79, "Phantasy Star"),
            (0x6605_D36A, "Phantasy Star (Japan)"),
            (0x0730_1F83, "Phantasy Star (Japan) (Rev 1)"),
            (0xB52D_60C8, "Ultima IV"),
            (0xB33E_2827, "Ys - The Vanished Omens"),
            (0x3275_9751, "Ys (Japan)"),
            (0x0E33_3B6E, "Miracle Warriors"),
            (0xB9FD_F6D9, "Haja no Fuuin"),
            (0x6953_8469, "Monopoly"),
            (0x026D_94A4, "Monopoly (Rev A)"),
        ];
        for (crc, name) in releases {
            let row = TABLE.iter().find(|r| r.crc == crc).unwrap_or_else(|| panic!("{name} has no row"));
            assert_eq!((row.mapper, row.ram, row.battery), (Mapper::Sega, 0x2000, true), "{name}");
        }
        assert_eq!(TABLE.iter().filter(|r| r.battery && r.mapper == Mapper::Sega).count(), releases.len(), "no battery row the tag does not name");
    }
}
