//! What an image is, read from its bytes alone: a cartridge (Genesis or 32X) or a Sega CD disc, and the header fields
//! the stub reports. The layout is the cartridge header of Nephrite_Plan.md §2.3; Nephrite_Native.md §2.2 says which
//! fields are read at stage 0 and which rules are still to be checked against the pinned documents.

/// The three systems of the core info, by their ids.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum System {
    Md,
    Mcd,
    S32x,
}

impl System {
    pub const fn id(self) -> &'static str {
        match self {
            System::Md => "md",
            System::Mcd => "mcd",
            System::S32x => "32x",
        }
    }
}

/// The header's region field as the three markets it allows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Markets {
    pub japan: bool,
    pub americas: bool,
    pub europe: bool,
}

impl Markets {
    /// PAL only when Europe is the one market allowed; an empty field reads as NTSC.
    pub fn pal(self) -> bool {
        self.europe && !self.japan && !self.americas
    }

    /// The old form (`J`, `U`, `E` in any order) or the new single hex digit (bit 0 Japan, bit 2 Americas, bit 3
    /// Europe). A field that begins `EUR` spells the word and is Europe alone, its `U` no code: Superman's and Another
    /// World's say `EUROPE` and The Smurfs 2's `Europe`, and the first and last stop on an American console
    /// (Nephrite_Native.md §47.4).
    pub fn parse(field: &[u8]) -> Markets {
        if field.len() >= 3 && field[..3].eq_ignore_ascii_case(b"EUR") {
            return Markets { japan: false, americas: false, europe: true };
        }
        let text: Vec<u8> = field.iter().copied().filter(|b| !b.is_ascii_whitespace() && *b != 0).collect();
        if text.len() == 1 && text[0].is_ascii_hexdigit() && !b"EJU".contains(&text[0].to_ascii_uppercase()) {
            let v = (text[0] as char).to_digit(16).unwrap_or(0);
            return Markets { japan: v & 1 != 0, americas: v & 4 != 0, europe: v & 8 != 0 };
        }
        let has = |c: u8| text.iter().any(|b| b.to_ascii_uppercase() == c);
        Markets { japan: has(b'J'), americas: has(b'U'), europe: has(b'E') }
    }
}

/// The cartridge's declared save RAM: its battery, which byte lanes it occupies, and its bus range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SaveRam {
    pub battery: bool,
    /// 0 both lanes, 2 even addresses only, 3 odd addresses only.
    pub lanes: u8,
    pub start: u32,
    pub end: u32,
}

impl SaveRam {
    /// Bytes the RAM holds: the range, halved when it occupies one lane; zero for a range that runs backwards.
    pub fn bytes(&self) -> usize {
        if self.end < self.start {
            return 0;
        }
        let span = (self.end - self.start) as usize + 1;
        if self.lanes == 0 { span } else { span.div_ceil(2) }
    }
}

/// Cartridges that keep their saves in a RAM their header does not declare, by serial (Nephrite_Native.md §46.2,
/// §47.5). Each writes its saves' odd bytes above its ROM, and Sega Retro gives each a battery save. In the table's
/// order: Madden NFL 98, College Football USA 96, FIFA Soccer 97, the same program under a pirate's serial, NHL 96,
/// NHL 98, PGA Tour Golf, Starflight, Buck Rogers, HardBall III, Summer Challenge, Winter Challenge and Test Drive II.
/// The header's word is taken first where it has one.
pub const UNDECLARED_SAVE: [&str; 13] = ["T-172196", "T-172046", "T-172156", "T-183457", "T-172036", "T-172176", "T-50086", "T-50216", "T-50286", "ACLD012", "ACLD013", "ACLD007", "ACLD008"];

/// The board such a cartridge is given: the one plutiedev's "Saving progress with SRAM" describes, 32 KiB on the odd
/// bytes of `$200001`-`$20FFFF` with a battery. A board with a smaller chip would repeat within the range.
pub fn undeclared_save(serial: &str) -> Option<SaveRam> {
    let s = serial.get(2..).unwrap_or("").trim_start_matches([' ', '_']);
    UNDECLARED_SAVE.iter().any(|known| s.starts_with(known)).then_some(SaveRam { battery: true, lanes: 3, start: 0x20_0001, end: 0x20_FFFF })
}

/// The fields of the 256-byte header at $100 that the stub reads.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Header {
    pub system_type: String,
    pub domestic_title: String,
    pub overseas_title: String,
    pub serial: String,
    pub checksum: u16,
    pub devices: String,
    pub markets: Markets,
    pub save: Option<SaveRam>,
}

fn text(b: &[u8]) -> String {
    let s: String = b.iter().map(|&c| if (0x20..0x7F).contains(&c) { c as char } else { ' ' }).collect();
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn long(b: &[u8], at: usize) -> u32 {
    u32::from_be_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

impl Header {
    /// The header whose first byte (the cartridge's $100) is at `h`; None when the image is too short to hold it.
    pub fn read(image: &[u8], h: usize) -> Option<Header> {
        let b = image.get(h..h + 0x100)?;
        let save = (&b[0xB0..0xB2] == b"RA").then(|| SaveRam {
            battery: b[0xB2] & 0x40 != 0,
            lanes: (b[0xB2] >> 3) & 3,
            start: long(b, 0xB4),
            end: long(b, 0xB8),
        });
        let serial = text(&b[0x80..0x8E]);
        let save = save.or_else(|| undeclared_save(&serial));
        Some(Header {
            system_type: text(&b[0x00..0x10]),
            domestic_title: text(&b[0x20..0x50]),
            overseas_title: text(&b[0x50..0x80]),
            serial,
            checksum: u16::from_be_bytes([b[0x8E], b[0x8F]]),
            devices: text(&b[0x90..0xA0]),
            markets: Markets::parse(&b[0xF0..0xF3]),
            save,
        })
    }

    /// Whether the system field names the 32X.
    pub fn is_32x(&self) -> bool {
        self.system_type.contains("32X")
    }
}

/// Where a disc image's system area begins: the user data of its first sector, at 0 in a 2048-byte-sector image
/// and after the 16 bytes of sync and sector header in a raw 2352-byte one.
pub const DISC_SIGNATURE: &[u8; 14] = b"SEGADISCSYSTEM";

pub fn disc_offset(image: &[u8]) -> Option<usize> {
    [0usize, 16].into_iter().find(|&o| image.get(o..o + DISC_SIGNATURE.len()) == Some(DISC_SIGNATURE.as_slice()))
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
        if let Some(o) = disc_offset(image) {
            return Media { system: System::Mcd, header: Header::read(image, o + 0x100), sector: if o == 0 { 2048 } else { 2352 } };
        }
        let header = Header::read(image, 0x100);
        let system = if header.as_ref().is_some_and(Header::is_32x) { System::S32x } else { System::Md };
        Media { system, header, sector: 0 }
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

    #[test]
    fn a_cartridge_is_told_from_a_32x_cartridge_and_a_disc() {
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
