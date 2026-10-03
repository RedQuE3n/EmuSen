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

/// Where a NEC DSP's DR, SR and RAM sit, by fullsnes's "SNES I/O Ports" table for the board the header describes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DspMap {
    /// LoROM of 1 MiB: banks 30-3F, DR at 8000-BFFF, SR at C000-FFFF.
    LoRom30,
    /// LoROM with RAM: banks 20-3F, as LoRom30.
    LoRom20,
    /// LoROM of 2 MiB: banks 60-6F, DR at 0000-3FFF, SR at 4000-7FFF.
    LoRom60,
    /// HiROM: banks 00-1F, DR at 6000-6FFF, SR at 7000-7FFF.
    HiRom,
    /// ST010/ST011: DR at 60-67:0000, SR at 0001, the chip's RAM at 68-6F:0000-0FFF.
    St,
}

#[derive(Clone, Debug)]
pub struct Cartridge {
    pub rom: Box<[u8]>,
    pub sram: Box<[u8]>,
    pub header: Header,
    /// A NEC DSP-n or ST01x, when its firmware was supplied, and where it is mapped.
    pub dsp: Option<(crate::chips::necdsp::NecDsp, DspMap)>,
    /// The SA-1, when the header's chipset names it (3xh); its BW-RAM is `sram`.
    pub sa1: Option<Box<crate::chips::sa1::Sa1>>,
    /// The GSU, when the chipset names it (1xh); its Game Pak RAM is `sram`.
    pub gsu: Option<Box<crate::chips::gsu::Gsu>>,
    /// The OBC1, when a LoROM header's chipset names it (2xh); its 8 KiB are `sram`.
    pub obc1: Option<crate::chips::obc1::Obc1>,
}

impl Cartridge {
    /// The cartridge under a header already chosen.
    pub fn with_header(rom: &[u8], header: Header) -> Cartridge {
        let gsu = Self::gsu_for(&header, rom.len());
        let sram = vec![0u8; if gsu.is_some() { Self::gsu_ram(rom, &header) } else { header.sram_bytes() }].into_boxed_slice();
        let sa1 = Self::sa1_for(&header);
        Cartridge { rom: rom.into(), sram, obc1: Self::obc1_for(&header), header, dsp: None, sa1, gsu }
    }

    /// A GSU for a chipset of 13h-1Ah on map mode 20h (fullsnes, "GSU Cartridge Header"); a ROM over 1 MiB takes the
    /// GSU-2, whose version code is 4, as fullsnes's rule of thumb has it.
    fn gsu_for(header: &Header, size: usize) -> Option<Box<crate::chips::gsu::Gsu>> {
        (matches!(header.chipset, 0x13..=0x1A) && header.map == Map::LoRom).then(|| Box::new(crate::chips::gsu::Gsu::new(size > 0x10_0000)))
    }

    /// An OBC1 for a LoROM chipset of 2xh (fullsnes's chipset table; Metal Combat's is 25h).
    fn obc1_for(header: &Header) -> Option<crate::chips::obc1::Obc1> {
        (header.chipset >> 4 == 2 && header.map == Map::LoRom).then(crate::chips::obc1::Obc1::default)
    }

    /// The Game Pak RAM's size from the extended header's FFBDh, 32 KiB where there is none (Star Fox).
    fn gsu_ram(rom: &[u8], header: &Header) -> usize {
        match rom.get(header.at.wrapping_sub(3)) {
            Some(&n) if (1..=7).contains(&n) => 1024 << n,
            _ => 0x8000,
        }
    }

    /// An SA-1 for a header whose chipset's high nibble is 3 (fullsnes's 32h-35h), on a map mode of 23h.
    fn sa1_for(header: &Header) -> Option<Box<crate::chips::sa1::Sa1>> {
        (header.chipset >> 4 == 3 && header.map_mode & 0x0F == 3).then(|| Box::new(crate::chips::sa1::Sa1::new(header.region() == Region::Pal)))
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
        let gsu = Self::gsu_for(&header, rom.len());
        let sram = vec![0u8; if gsu.is_some() { Self::gsu_ram(rom, &header) } else { header.sram_bytes() }].into_boxed_slice();
        let sa1 = Self::sa1_for(&header);
        Some(Cartridge { rom: rom.into(), sram, obc1: Self::obc1_for(&header), header, dsp: None, sa1, gsu })
    }

    /// Whether the header names a NEC DSP: chipset 03h-05h for a DSP-n, F6h for an ST010 or ST011 (fullsnes).
    pub fn wants_dsp(&self) -> bool {
        matches!(self.header.chipset, 0x03..=0x05 | 0xF6)
    }

    /// Which NEC DSP firmware the cartridge needs, as a file stem and its size, from the header and title against
    /// fullsnes's list of the 23 games ("List of Games using that chips"): ST010 for F1 ROC II / Exhaust Heat II,
    /// ST011 for the other ST01x game; DSP-2, DSP-3 and DSP-4 for their one game each; DSP-1 for Pilotwings, which
    /// fullsnes names as the DSP-1 game with its visible glitch, and DSP-1B, the corrected revision, for the rest.
    pub fn nec_firmware(&self) -> Option<(&'static str, u64)> {
        let title = self.header.title.to_ascii_uppercase();
        let has = |s: &str| title.contains(s);
        match self.header.chipset {
            0xF6 => Some((if has("F1 ROC") || has("EXHAUST") { "st010" } else { "st011" }, 53_248)),
            0x03..=0x05 => Some((
                if has("DUNGEON MASTER") {
                    "dsp2"
                } else if has("GUNDAM") {
                    "dsp3"
                } else if has("TOP GEAR 3000") || has("TG3000") || has("TG 3000") {
                    "dsp4"
                } else if has("PILOTWINGS") {
                    "dsp1"
                } else {
                    "dsp1b"
                },
                8_192,
            )),
            _ => None,
        }
    }

    /// The board's DSP map: the ST01x's when its firmware is the µPD96050's or the chipset says so, then HiROM, then
    /// LoROM by size and RAM.
    pub fn dsp_map(&self, st: bool) -> DspMap {
        if st || self.header.chipset == 0xF6 {
            DspMap::St
        } else if self.header.map != Map::LoRom {
            DspMap::HiRom
        } else if self.rom.len() > 0x10_0000 {
            DspMap::LoRom60
        } else if !self.sram.is_empty() {
            DspMap::LoRom20
        } else {
            DspMap::LoRom30
        }
    }

    /// The DSP's port a CPU address selects, if the cartridge has one there.
    #[inline]
    pub fn dsp_port(&self, address: u32) -> Option<crate::chips::necdsp::Port> {
        use crate::chips::necdsp::Port;
        let (_, map) = self.dsp.as_ref()?;
        let bank = (address >> 16) as usize & 0x7F;
        let offset = address as usize & 0xFFFF;
        match map {
            DspMap::LoRom30 | DspMap::LoRom20 => {
                let first = if *map == DspMap::LoRom30 { 0x30 } else { 0x20 };
                ((first..=0x3F).contains(&bank) && offset >= 0x8000).then_some(if offset < 0xC000 { Port::Dr } else { Port::Sr })
            }
            DspMap::LoRom60 => ((0x60..=0x6F).contains(&bank) && offset < 0x8000).then_some(if offset < 0x4000 { Port::Dr } else { Port::Sr }),
            DspMap::HiRom => (bank < 0x20 && (0x6000..0x8000).contains(&offset)).then_some(if offset < 0x7000 { Port::Dr } else { Port::Sr }),
            DspMap::St => match bank {
                0x60..=0x67 if offset < 0x1000 => Some(if offset & 1 == 0 { Port::Dr } else { Port::Sr }),
                0x68..=0x6F if offset < 0x1000 => Some(Port::Ram(offset)),
                _ => None,
            },
        }
    }

    /// The ROM byte or SRAM slot a CPU address selects, or None for nothing the cartridge drives.
    #[inline]
    pub fn decode(&self, address: u32) -> Option<Slot> {
        // An SA-1 board decodes its own space (chips/sa1.rs); nothing else on it answers the S-CPU.
        if self.sa1.is_some() || self.gsu.is_some() {
            return None;
        }
        if self.dsp.is_some() {
            if let Some(port) = self.dsp_port(address) {
                return Some(Slot::Dsp(port));
            }
        }
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
    Dsp(crate::chips::necdsp::Port),
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

    // fullsnes's "SNES I/O Ports" for the DSP boards: LoROM 1 MiB at 30-3F, HiROM at 00-1F:6000-7FFF, the ST01x at 60-6F.
    #[test]
    fn a_dsp_board_maps_dr_and_sr_where_fullsnes_puts_them() {
        use crate::chips::necdsp::{NecDsp, Port};
        let mut lo = image(0x8_0000, Map::LoRom, 0x20);
        lo[0x7FC0 + 0x16] = 0x03;
        let mut c = Cartridge::new(&lo).unwrap();
        assert!(c.wants_dsp());
        let dsp = NecDsp::from_firmware(&vec![0u8; 8192]).unwrap();
        c.dsp = Some((dsp.clone(), c.dsp_map(false)));
        assert_eq!(c.dsp.as_ref().unwrap().1, DspMap::LoRom30);
        assert_eq!((c.decode(0x30_8000), c.decode(0xBF_C000), c.decode(0x00_8000)), (Some(Slot::Dsp(Port::Dr)), Some(Slot::Dsp(Port::Sr)), Some(Slot::Rom(0))));
        let mut hi = Cartridge::new(&image(0x10_0000, Map::HiRom, 0x21)).unwrap();
        hi.dsp = Some((dsp, hi.dsp_map(false)));
        assert_eq!((hi.decode(0x00_6000), hi.decode(0x9F_7FFF), hi.decode(0x20_6000)), (Some(Slot::Dsp(Port::Dr)), Some(Slot::Dsp(Port::Sr)), Some(Slot::Sram(0))));
        let st = NecDsp::from_firmware(&vec![0u8; 53248]).unwrap();
        let mut f1 = Cartridge::new(&lo).unwrap();
        f1.dsp = Some((st, f1.dsp_map(true)));
        assert_eq!((f1.decode(0x60_0000), f1.decode(0x60_0001), f1.decode(0x68_0FFF)), (Some(Slot::Dsp(Port::Dr)), Some(Slot::Dsp(Port::Sr)), Some(Slot::Dsp(Port::Ram(0xFFF)))));
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
