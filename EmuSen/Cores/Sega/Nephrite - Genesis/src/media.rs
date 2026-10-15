//! What an image is, read from its bytes alone: a cartridge (Genesis or 32X) or a Sega CD disc, with the header the
//! Genesis reads (`genesis/header.rs`), and a copier's image unwrapped. Nephrite_Native.md §2.2 says which fields are
//! read at stage 0 and which rules are still to be checked against the pinned documents.

/// The three systems of the core info, by their ids.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum System {
    Md,
    #[cfg(feature = "mcd")]
    Mcd,
    #[cfg(feature = "s32x")]
    S32x,
}

impl System {
    pub const fn id(self) -> &'static str {
        match self {
            System::Md => "md",
            #[cfg(feature = "mcd")]
            System::Mcd => "mcd",
            #[cfg(feature = "s32x")]
            System::S32x => "32x",
        }
    }

    /// A Sega CD disc's machine.
    pub fn is_disc(self) -> bool {
        match self {
            #[cfg(feature = "mcd")]
            System::Mcd => true,
            _ => false,
        }
    }

    /// A 32X cartridge's machine.
    pub fn is_32x(self) -> bool {
        match self {
            #[cfg(feature = "s32x")]
            System::S32x => true,
            _ => false,
        }
    }
}

pub use crate::genesis::header::{Header, Markets, SaveRam};

#[cfg(feature = "mcd")]
use crate::mcd::disc_offset;
#[cfg(feature = "s32x")]
use crate::s32x::is_32x;

/// Without the Sega CD built, no image is a disc.
#[cfg(not(feature = "mcd"))]
fn disc_offset(_: &[u8]) -> Option<usize> {
    None
}

/// A copier's 512-byte header and 16 KiB blocks (Nephrite_Native.md §11.2).
pub const COPIER_HEADER: usize = 0x200;
pub const COPIER_BLOCK: usize = 0x4000;

fn names_sega(image: &[u8]) -> bool {
    image.get(0x100..0x110).is_some_and(|f| f.windows(4).any(|w| w == b"SEGA"))
}

/// A Super Magic Drive block in the cartridge's order: its first half holds the bytes at odd addresses, its second
/// those at even ones.
fn deinterleave(body: &[u8]) -> Vec<u8> {
    let mut out = vec![0u8; body.len()];
    for (o, block) in out.chunks_mut(COPIER_BLOCK).zip(body.chunks(COPIER_BLOCK)) {
        let (odd, even) = block.split_at(block.len() / 2);
        for (i, (&e, &d)) in even.iter().zip(odd).enumerate() {
            (o[2 * i], o[2 * i + 1]) = (e, d);
        }
    }
    out
}

/// The cartridge's bytes in an image a copier wrote: a 512-byte header before whole 16 KiB blocks, interleaved when
/// the header carries the Super Magic Drive's $AA $BB and the result names SEGA at $100; anything else is unchanged.
pub fn cartridge_bytes(image: &[u8]) -> std::borrow::Cow<'_, [u8]> {
    use std::borrow::Cow;
    let n = image.len();
    if n <= COPIER_HEADER || (n - COPIER_HEADER) % COPIER_BLOCK != 0 || disc_offset(image).is_some() || names_sega(image) {
        return Cow::Borrowed(image);
    }
    let body = &image[COPIER_HEADER..];
    if image[8..10] == [0xAA, 0xBB] {
        let rom = deinterleave(body);
        if names_sega(&rom) {
            return Cow::Owned(rom);
        }
    }
    if names_sega(body) { Cow::Borrowed(body) } else { Cow::Borrowed(image) }
}

/// An image as the stub reads it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Media {
    pub system: System,
    /// The header, absent on a cartridge too short to carry one.
    pub header: Option<Header>,
    /// 2048 or 2352 for a disc, 0 for a cartridge.
    pub sector: usize,
}

impl Media {
    pub fn read(image: &[u8]) -> Media {
        #[cfg(feature = "mcd")]
        if let Some(o) = disc_offset(image) {
            return Media { system: System::Mcd, header: Header::read(image, o + 0x100), sector: if o == 0 { 2048 } else { 2352 } };
        }
        let header = Header::read(image, 0x100);
        #[cfg(feature = "s32x")]
        if header.as_ref().is_some_and(|h| is_32x(&h.system_type)) {
            return Media { system: System::S32x, header, sector: 0 };
        }
        Media { system: System::Md, header, sector: 0 }
    }

    pub fn pal(&self) -> bool {
        self.header.as_ref().is_some_and(|h| h.markets.pal())
    }

    /// The region letter a Sega CD BIOS is chosen by: U for the Americas, then J, then E (Nephrite_Plan.md §9, Q6).
    pub fn bios_region(&self) -> char {
        let m = self.header.as_ref().map(|h| h.markets).unwrap_or_default();
        if m.americas || (!m.japan && !m.europe) {
            'U'
        } else if m.japan {
            'J'
        } else {
            'E'
        }
    }

    /// Battery-backed bytes the cartridge declares, zero for none; a disc's internal backup RAM is the machine's.
    pub fn battery_bytes(&self) -> usize {
        match (&self.header, self.system) {
            #[cfg(feature = "mcd")]
            (_, System::Mcd) => 0,
            (Some(Header { save: Some(s), .. }), _) if s.battery => s.bytes(),
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn cartridge(system: &str, region: &str, save: Option<[u8; 12]>) -> Vec<u8> {
        let mut rom = vec![0u8; 0x8000];
        let pad = |s: &str, n: usize| format!("{s:<n$}").into_bytes();
        rom[0x100..0x110].copy_from_slice(&pad(system, 16));
        rom[0x120..0x150].copy_from_slice(&pad("NEPHRITE TEST", 48));
        rom[0x150..0x180].copy_from_slice(&pad("NEPHRITE TEST", 48));
        rom[0x180..0x18E].copy_from_slice(&pad("GM 00000000-00", 14));
        rom[0x190..0x1A0].copy_from_slice(&pad("J6", 16));
        if let Some(s) = save {
            rom[0x1B0..0x1BC].copy_from_slice(&s);
        }
        rom[0x1F0..0x1F3].copy_from_slice(&pad(region, 3));
        rom
    }

    #[test]
    fn the_region_field_reads_both_forms() {
        assert_eq!(Markets::parse(b"JUE"), Markets { japan: true, americas: true, europe: true });
        assert_eq!(Markets::parse(b"U  "), Markets { japan: false, americas: true, europe: false });
        assert_eq!(Markets::parse(b"8  "), Markets { japan: false, americas: false, europe: true });
        assert_eq!(Markets::parse(b"5  "), Markets { japan: true, americas: true, europe: false });
        assert!(Markets::parse(b"E  ").pal());
        assert!(!Markets::parse(b"JE ").pal());
        assert!(!Markets::parse(b"   ").pal());
        for word in [&b"EUROPE"[..], b"Europe", b"EUR", b"eur"] {
            assert_eq!(Markets::parse(word), Markets { japan: false, americas: false, europe: true }, "the word, not E and U");
        }
        assert_eq!(Markets::parse(b"EU "), Markets { japan: false, americas: true, europe: true });
    }

    #[cfg(all(feature = "mcd", feature = "s32x"))]
    #[test]
    fn a_cartridge_is_told_from_a_32x_cartridge_and_a_disc() {
        use crate::mcd::DISC_SIGNATURE;
        assert_eq!(Media::read(&cartridge("SEGA GENESIS", "U", None)).system, System::Md);
        assert_eq!(Media::read(&cartridge("SEGA 32X", "U", None)).system, System::S32x);
        let mut iso = vec![0u8; 0x800];
        iso[..14].copy_from_slice(DISC_SIGNATURE);
        iso[0x1F0] = b'J';
        let m = Media::read(&iso);
        assert_eq!((m.system, m.sector, m.bios_region()), (System::Mcd, 2048, 'J'));
        let mut raw = vec![0u8; 2352];
        raw[16..30].copy_from_slice(DISC_SIGNATURE);
        assert_eq!(Media::read(&raw).sector, 2352);
        assert_eq!(Media::read(&[0u8; 16]).header, None);
    }

    #[test]
    fn a_copier_header_is_stripped_and_its_blocks_deinterleaved() {
        let rom: Vec<u8> = cartridge("SEGA GENESIS", "U", None).iter().enumerate().map(|(i, &b)| if (0x100..0x200).contains(&i) { b } else { i as u8 ^ (i >> 8) as u8 }).collect();
        let mut header = vec![0u8; COPIER_HEADER];
        (header[0], header[1], header[8], header[9]) = ((rom.len() / COPIER_BLOCK) as u8, 3, 0xAA, 0xBB);
        let smd: Vec<u8> = rom.chunks(COPIER_BLOCK).flat_map(|b| b.iter().skip(1).step_by(2).chain(b.iter().step_by(2)).copied().collect::<Vec<_>>()).collect();
        assert_eq!(&*cartridge_bytes(&[header.clone(), smd].concat()), &rom[..]);
        assert_eq!(&*cartridge_bytes(&[vec![0u8; COPIER_HEADER], rom.clone()].concat()), &rom[..]);
        assert_eq!(&*cartridge_bytes(&rom), &rom[..]);
        let unknown = [header, vec![0x5Au8; rom.len()]].concat();
        assert_eq!(&*cartridge_bytes(&unknown), &unknown[..]);
    }

    // Madden NFL 98's, PGA Tour Golf's and HardBall III's serials with nothing declared, then a header's own word and a serial not listed.
    #[test]
    fn a_cartridge_known_to_omit_its_save_ram_is_given_the_standard_board() {
        let with = |serial: &str, save: Option<[u8; 12]>| {
            let mut rom = cartridge("SEGA GENESIS", "U", save);
            rom[0x180..0x18E].copy_from_slice(format!("{serial:<14}").as_bytes());
            Media::read(&rom)
        };
        for serial in ["GM T-172196-00", "GM T-50086 -01", "GM ACLD012 -00", "GM ACLD008 -00"] {
            let m = with(serial, None);
            assert_eq!(m.header.as_ref().unwrap().save, Some(SaveRam { battery: true, lanes: 3, start: 0x20_0001, end: 0x20_FFFF }), "{serial}");
            assert_eq!(m.battery_bytes(), 0x8000, "{serial}");
        }
        let mut declared = [0u8; 12];
        declared[..4].copy_from_slice(&[b'R', b'A', 0xF8, 0x20]);
        declared[4..8].copy_from_slice(&0x20_0001u32.to_be_bytes());
        declared[8..12].copy_from_slice(&0x20_3FFFu32.to_be_bytes());
        assert_eq!(with("GM T-172196-00", Some(declared)).battery_bytes(), 0x2000, "the header's word first");
        assert_eq!(with("GM T-50706 -00", None).header.unwrap().save, None, "a serial not listed");
        assert_eq!(with("GM 00000000-00", None).battery_bytes(), 0);
    }

    #[test]
    fn declared_save_ram_is_sized_by_its_lanes() {
        let ra = |kind: u8, start: u32, end: u32| {
            let mut b = [0u8; 12];
            b[..2].copy_from_slice(b"RA");
            b[2] = kind;
            b[3] = 0x20;
            b[4..8].copy_from_slice(&start.to_be_bytes());
            b[8..12].copy_from_slice(&end.to_be_bytes());
            b
        };
        assert_eq!(Media::read(&cartridge("SEGA GENESIS", "U", Some(ra(0xF8, 0x200001, 0x203FFF)))).battery_bytes(), 8192);
        assert_eq!(Media::read(&cartridge("SEGA GENESIS", "U", Some(ra(0xE0, 0x200000, 0x20FFFF)))).battery_bytes(), 65536);
        assert_eq!(Media::read(&cartridge("SEGA GENESIS", "U", Some(ra(0xB8, 0x200001, 0x203FFF)))).battery_bytes(), 0);
        assert_eq!(Media::read(&cartridge("SEGA GENESIS", "U", None)).battery_bytes(), 0);
    }
}

#[cfg(test)]
pub use tests::cartridge;
