//! The cartridge: the header found and scored, and the LoROM, HiROM and ExHiROM maps with their mirrors and SRAM,
//! from fullsnes's "SNES Memory Map" and "SNES Cartridge ROM Header". See VenusRT_Native.md §12.2.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Map {
    LoRom,
    HiRom,
    ExHiRom,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Region {
    Ntsc,
    Pal,
}

/// The header's fields that the machine uses, and the evidence the choice rested on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Header {
    pub map: Map,
    /// The header's offset in the image, after any copier header.
    pub at: usize,
    pub title: String,
    pub map_mode: u8,
    pub chipset: u8,
    pub rom_size: u8,
    pub sram_size: u8,
    pub country: u8,
    pub score: i32,
}

impl Header {
    pub fn region(&self) -> Region {
        if matches!(self.country, 0x02..=0x0C | 0x11) { Region::Pal } else { Region::Ntsc }
    }

    /// SRAM in bytes: (1 << n) KiB when the chipset says RAM is fitted.
    pub fn sram_bytes(&self) -> usize {
        let has_ram = matches!(self.chipset & 0x0F, 0x01 | 0x02 | 0x04 | 0x05 | 0x09 | 0x0A) || self.chipset == 0x02;
        if !has_ram || self.sram_size == 0 || self.sram_size > 8 { 0 } else { 1024 << self.sram_size }
    }

    pub fn battery(&self) -> bool {
        matches!(self.chipset & 0x0F, 0x02 | 0x05 | 0x06 | 0x09 | 0x0A)
    }
}

/// The header's candidate places, by map: fullsnes's 7FC0h, FFC0h and 40FFC0h.
const CANDIDATES: [(Map, usize); 3] = [(Map::LoRom, 0x7FC0), (Map::HiRom, 0xFFC0), (Map::ExHiRom, 0x40_FFC0)];

/// A plausible first instruction of a reset handler.
fn plausible_start(op: u8) -> bool {
    matches!(op, 0x78 | 0x18 | 0x38 | 0xD8 | 0x9C | 0x4C | 0x5C | 0xC2 | 0xE2 | 0xA9 | 0xA2 | 0xA0 | 0x8D | 0x20 | 0x22 | 0xFB | 0x5B | 0xEA | 0x80 | 0x82)
}

/// Scores one candidate: each field that agrees with the place it was found in adds evidence. The weights are this
/// core's own, set out in VenusRT_Native.md §12.2.
fn score(image: &[u8], map: Map, at: usize) -> Option<(i32, Header)> {
    let h = image.get(at..at + 0x40)?;
    let mode = h[0x15];
    let mut s = 0;
    let mode_fits = match map {
        Map::LoRom => matches!(mode & 0x0F, 0x0 | 0x2 | 0x3),
        Map::HiRom => matches!(mode & 0x0F, 0x1 | 0xA),
        Map::ExHiRom => mode & 0x0F == 0x5,
    };
    if mode_fits && mode & 0xE0 == 0x20 {
        s += 4;
    }
    let complement = u16::from_le_bytes([h[0x1C], h[0x1D]]);
    let checksum = u16::from_le_bytes([h[0x1E], h[0x1F]]);
    if complement ^ checksum == 0xFFFF {
        s += 4;
        if checksum != 0 && checksum != 0xFFFF {
            s += 2;
        }
    }
    let reset = u16::from_le_bytes([h[0x3C], h[0x3D]]);
    if reset >= 0x8000 {
        s += 2;
        let base = match map {
            Map::LoRom => at & !0x7FFF,
            Map::HiRom | Map::ExHiRom => at & !0xFFFF,
        };
        let offset = match map {
            Map::LoRom => base + (reset as usize & 0x7FFF),
            _ => base + reset as usize,
        };
        if image.get(offset).copied().is_some_and(plausible_start) {
            s += 2;
        }
    } else {
        s -= 4;
    }
    if h[..21].iter().all(|&c| (0x20..0x7F).contains(&c)) {
        s += 2;
    }
    if h[0x17] >= 7 && h[0x17] <= 0x0D && (1024usize << h[0x17]) >= image.len() / 2 {
        s += 1;
    }
    if h[0x18] <= 8 {
        s += 1;
    }
    let title = h[..21].iter().map(|&c| if (0x20..0x7F).contains(&c) { c as char } else { '.' }).collect::<String>().trim_end().to_owned();
    Some((s, Header { map, at, title, map_mode: mode, chipset: h[0x16], rom_size: h[0x17], sram_size: h[0x18], country: h[0x19], score: s }))
}

/// The best-scoring candidate by its fields, a tie going to the earlier of LoROM, HiROM and ExHiROM, unless D-4's rule
/// overrules it. ExHiROM is considered only for an image of more than 4 MiB, the only place its header can be.
pub fn find_header(image: &[u8]) -> Option<Header> {
    let all = candidates(image);
    let best = all.iter().fold(None, |best: Option<&Header>, h| if best.is_none_or(|b| h.score > b.score) { Some(h) } else { best })?;
    // D-4: a choice whose reset handler does nothing gives way to a candidate whose handler works.
    let (writes, crashed) = crate::machine::Machine::reset_evidence(image, best.clone(), HANDLER_INSTRUCTIONS);
    if !crashed && writes <= 1 {
        let working = all.iter().filter(|h| h.map != best.map).find(|h| {
            let (w, c) = crate::machine::Machine::reset_evidence(image, (*h).clone(), HANDLER_INSTRUCTIONS);
            !c && w >= 4
        });
        if let Some(other) = working {
            return Some(other.clone());
        }
    }
    Some(best.clone())
}

/// How far a candidate's reset handler is run for D-4's evidence.
const HANDLER_INSTRUCTIONS: u32 = 4000;

/// Every candidate with its fields' score, in LoROM, HiROM, ExHiROM order.
pub fn candidates(image: &[u8]) -> Vec<Header> {
    let mut all = Vec::new();
    for (map, at) in CANDIDATES {
        if map == Map::ExHiRom && image.len() <= 0x40_0000 {
            continue;
        }
        if let Some((_, h)) = score(image, map, at) {
            all.push(h);
        }
    }
    all
}

/// An offset into a ROM of any size, as the chips decode it: the larger power of two first, then the remainder's
/// mirrors (fullsnes, "ROM Size / Checksum Notes").
pub fn mirror(offset: usize, size: usize) -> usize {
    if size == 0 {
        return 0;
    }
    let p = size.next_power_of_two();
    let offset = offset & (p - 1);
    if offset < size {
        offset
    } else {
        let half = p / 2;
        half + mirror(offset - half, size - half)
    }
}

#[derive(Clone, Debug)]
pub struct Cartridge {
    pub rom: Box<[u8]>,
    pub sram: Box<[u8]>,
    pub header: Header,
}

impl Cartridge {
    /// The cartridge under a header already chosen.
    pub fn with_header(rom: &[u8], header: Header) -> Cartridge {
        let sram = vec![0u8; header.sram_bytes()].into_boxed_slice();
        Cartridge { rom: rom.into(), sram, header }
    }

    /// A copier's 512-byte header is dropped by the image's length modulo 1 KiB; an image with no header that scores
    /// is mapped as LoROM, as a homebrew file of no header would be.
    pub fn new(image: &[u8]) -> Option<Cartridge> {
        let rom = if image.len() % 1024 == 512 { &image[512..] } else { image };
        if rom.len() < 0x8000 {
            return None;
        }
        let header = find_header(rom).unwrap_or(Header {
            map: Map::LoRom,
            at: 0x7FC0,
            title: String::new(),
            map_mode: 0x20,
            chipset: 0,
            rom_size: 0,
            sram_size: 0,
            country: 0,
            score: 0,
        });
        let sram = vec![0u8; header.sram_bytes()].into_boxed_slice();
        Some(Cartridge { rom: rom.into(), sram, header })
    }

    /// The ROM byte or SRAM slot a CPU address selects, or None for nothing the cartridge drives.
    #[inline]
    pub fn decode(&self, address: u32) -> Option<Slot> {
        let bank = (address >> 16) as usize;
        let offset = address as usize & 0xFFFF;
        let size = self.rom.len();
        match self.header.map {
            Map::LoRom => {
                if offset >= 0x8000 {
                    return Some(Slot::Rom(mirror(((bank & 0x7F) << 15) | (offset & 0x7FFF), size)));
                }
                if !self.sram.is_empty() && (matches!(bank, 0x70..=0x7D) || bank >= 0xF0) {
                    return Some(Slot::Sram((((bank & 0x0F) << 15) | offset) & (self.sram.len() - 1)));
                }
                // Below $8000 a LoROM cartridge drives only its SRAM (fullsnes; jonasquinn's memtest agrees).
                None
            }
            Map::HiRom | Map::ExHiRom => {
                let upper = if self.header.map == Map::ExHiRom && bank & 0x80 == 0 { 0x40_0000 } else { 0 };
                if bank & 0x40 != 0 && (bank & 0x7F) < 0x7E || bank >= 0xC0 {
                    return Some(Slot::Rom(mirror(upper | ((bank & 0x3F) << 16) | offset, size)));
                }
                if bank & 0x40 == 0 {
                    if offset >= 0x8000 {
                        return Some(Slot::Rom(mirror(upper | ((bank & 0x3F) << 16) | offset, size)));
                    }
                    if !self.sram.is_empty() && (0x6000..0x8000).contains(&offset) && (bank & 0x3F) >= 0x20 {
                        return Some(Slot::Sram((((bank & 0x1F) << 13) | (offset - 0x6000)) & (self.sram.len() - 1)));
                    }
                }
                None
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
    Rom(usize),
    Sram(usize),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image(size: usize, map: Map, mode: u8) -> Vec<u8> {
        let mut rom = vec![0xEAu8; size];
        let at = match map {
            Map::LoRom => 0x7FC0,
            Map::HiRom => 0xFFC0,
            Map::ExHiRom => 0x40_FFC0,
        };
        rom[at..at + 21].copy_from_slice(b"VENUSRT TEST CART    ");
        rom[at + 0x15] = mode;
        rom[at + 0x16] = 0x02;
        rom[at + 0x17] = 0x0A;
        rom[at + 0x18] = 0x03;
        rom[at + 0x1C..at + 0x20].copy_from_slice(&[0x34, 0x12, 0xCB, 0xED]);
        rom[at + 0x3C..at + 0x3E].copy_from_slice(&[0x00, 0x80]);
        let start = match map { Map::LoRom => at & !0x7FFF, _ => at & !0xFFFF };
        rom[start] = 0x78;
        rom
    }

    #[test]
    fn each_map_is_found_where_its_header_is() {
        for (map, mode, size) in [(Map::LoRom, 0x20, 0x10_0000), (Map::HiRom, 0x21, 0x10_0000), (Map::ExHiRom, 0x25, 0x60_0000)] {
            let c = Cartridge::new(&image(size, map, mode)).unwrap();
            assert_eq!(c.header.map, map, "{map:?}");
            assert_eq!(c.header.title, "VENUSRT TEST CART");
            assert_eq!(c.sram.len(), 8192);
            assert!(c.header.battery());
        }
        let mut copier = vec![0u8; 512];
        copier.extend(image(0x8_0000, Map::HiRom, 0x31));
        assert_eq!(Cartridge::new(&copier).unwrap().header.map, Map::HiRom);
    }

    #[test]
    fn the_maps_decode_as_fullsnes_draws_them() {
        let lo = Cartridge::new(&image(0x10_0000, Map::LoRom, 0x20)).unwrap();
        assert_eq!(lo.decode(0x00_8000), Some(Slot::Rom(0)));
        assert_eq!(lo.decode(0x81_FFFF), Some(Slot::Rom(0xFFFF)));
        assert_eq!(lo.decode(0x00_1234), None);
        assert_eq!(lo.decode(0x40_1234), None);
        assert_eq!(lo.decode(0x40_8000), Some(Slot::Rom(0)));
        assert_eq!(lo.decode(0x70_0000), Some(Slot::Sram(0)));
        assert_eq!(lo.decode(0xF0_1FFF), Some(Slot::Sram(0x1FFF)));
        assert_eq!(lo.decode(0x70_2000), Some(Slot::Sram(0)));
        let hi = Cartridge::new(&image(0x10_0000, Map::HiRom, 0x21)).unwrap();
        assert_eq!(hi.decode(0xC0_0000), Some(Slot::Rom(0)));
        assert_eq!(hi.decode(0x40_FFC0), Some(Slot::Rom(0xFFC0)));
        assert_eq!(hi.decode(0x00_FFC0), Some(Slot::Rom(0xFFC0)));
        assert_eq!(hi.decode(0x00_7FFF), None);
        assert_eq!(hi.decode(0x30_6000), Some(Slot::Sram(0)));
        assert_eq!(hi.decode(0xB0_7FFF), Some(Slot::Sram(0x1FFF)));
        let ex = Cartridge::new(&image(0x60_0000, Map::ExHiRom, 0x25)).unwrap();
        assert_eq!(ex.decode(0xC0_0000), Some(Slot::Rom(0)));
        assert_eq!(ex.decode(0x40_0000), Some(Slot::Rom(0x40_0000)));
        assert_eq!(ex.decode(0x00_FFC0), Some(Slot::Rom(0x40_FFC0)));
    }

    // Batman's shape: the only well-formed header at the HiROM place, whose reset leads to an idle loop, and an empty
    // LoROM place whose reset vector leads to a handler that sets the machine up (D-4).
    #[test]
    fn the_reset_handler_decides_when_the_fields_choose_a_handler_that_does_nothing() {
        let mut rom = image(0x10_0000, Map::HiRom, 0x31);
        rom[0x8000..0x8002].copy_from_slice(&[0x80, 0xFE]);
        rom[0xFFFC..0xFFFE].copy_from_slice(&[0x00, 0x80]);
        let handler = [0x78, 0x9C, 0x00, 0x42, 0x9C, 0x0B, 0x42, 0x9C, 0x0C, 0x42, 0x9C, 0x00, 0x21, 0x9C, 0x05, 0x21, 0x80, 0xFE];
        rom[..handler.len()].copy_from_slice(&handler);
        rom[0x7FFC..0x7FFE].copy_from_slice(&[0x00, 0x80]);
        rom[0x7FC0..0x7FFC].fill(0);
        assert_eq!(find_header(&rom).unwrap().map, Map::LoRom);
        // With the HiROM handler doing the same work, the fields' choice stands.
        rom[0x8000..0x8000 + handler.len()].copy_from_slice(&handler);
        assert_eq!(find_header(&rom).unwrap().map, Map::HiRom);
    }

    #[test]
    fn an_odd_size_mirrors_its_smaller_part() {
        let three = 0x30_0000;
        assert_eq!(mirror(0x10_0000, three), 0x10_0000);
        assert_eq!(mirror(0x30_0000, three), 0x20_0000);
        assert_eq!(mirror(0x3F_FFFF, three), 0x2F_FFFF);
        assert_eq!(mirror(0x40_0001, three), 1);
        assert_eq!(mirror(0x2_8000, 0x2_0000), 0x8000);
    }
}
