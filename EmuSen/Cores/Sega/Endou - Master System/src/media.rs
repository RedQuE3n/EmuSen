//! The image as stage 1 reads it (Endou_Native.md §2): a copier's 512-byte header set aside, Sega's header found at
//! `$7FF0`, `$3FF0` or `$1FF0` and its checksum compared, the market its region code names, and the console the image
//! is for. Nothing here maps a board; the mappers are stage 2's.

/// The two systems Endou runs (Endou_Plan.md §1.1); the Game Gear is a later system of the same library (§2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Console {
    Sms,
    Sg1000,
}

impl Console {
    pub const ALL: [Console; 2] = [Console::Sms, Console::Sg1000];

    pub fn id(self) -> &'static str {
        match self {
            Console::Sms => "sms",
            Console::Sg1000 => "sg1000",
        }
    }

    /// The console a system id names, as core info lists it; none for another.
    pub fn of_id(id: &str) -> Option<Console> {
        Console::ALL.into_iter().find(|c| c.id() == id)
    }
}

/// The markets of the region setting (Endou_Plan.md §5.8); Europe alone is PAL.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Market {
    Japan,
    Americas,
    Europe,
    Brazil,
}

/// Sega's header: where it was found and what it states (SMS Power!'s "ROM Header"; Sega's Mark III manual, "The
/// cartridge header").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Header {
    pub at: usize,
    pub checksum: u16,
    /// The sum Sega's manual describes, over the range the size code names (Endou_Native.md §2.2).
    pub computed: u16,
    pub product: u32,
    pub version: u8,
    pub region: u8,
    pub size_code: u8,
}

impl Header {
    pub fn checksum_agrees(&self) -> bool {
        self.checksum == self.computed
    }

    /// The market the region code names: Japan for `$3` and `$5`, an export market (the Americas) for `$4`, `$6` and
    /// `$7`; none for any other value.
    pub fn market(&self) -> Option<Market> {
        match self.region {
            0x3 | 0x5 => Some(Market::Japan),
            0x4 | 0x6 | 0x7 => Some(Market::Americas),
            _ => None,
        }
    }
}

/// The bytes the size code names, as SMS Power!'s "ROM Header" tabulates them; none for a code it does not list.
pub fn size_of_code(code: u8) -> Option<usize> {
    let kib = match code {
        0xA => 8,
        0xB => 16,
        0xC => 32,
        0xD => 48,
        0xE => 64,
        0xF => 128,
        0x0 => 256,
        0x1 => 512,
        0x2 => 1024,
        _ => return None,
    };
    Some(kib * 1024)
}

pub const SIGNATURE: &[u8; 8] = b"TMR SEGA";
pub const HEADER_PLACES: [usize; 3] = [0x7FF0, 0x3FF0, 0x1FF0];
pub const COPIER_HEADER: usize = 512;
/// The largest image accepted: 4 MiB, above every board the plan names (Endou_Plan.md §5.6).
pub const MAX_IMAGE: usize = 4 << 20;
/// The largest cartridge of the SG-1000's boards, 48 KiB; an image without Sega's header and no larger is taken as the SG-1000's (Endou_Native.md §2.3).
pub const SG1000_MAX: usize = 48 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    Empty,
    TooLarge(usize),
}

/// What the image is, read once at create.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Media {
    pub console: Console,
    /// Bytes of a copier's header before the image, set aside.
    pub skipped: usize,
    pub header: Option<Header>,
}

impl Media {
    /// The image's facts, or why it is no image at all; its console by the stage-1 rule (Endou_Native.md §2.3).
    pub fn read(image: &[u8]) -> Result<Media, Refusal> {
        Media::read_as(image, None)
    }

    /// The same for an image a host routed to `console` (core ABI 1.15), which the stage-1 rule then does not guess.
    pub fn read_as(image: &[u8], console: Option<Console>) -> Result<Media, Refusal> {
        if image.is_empty() {
            return Err(Refusal::Empty);
        }
        if image.len() > MAX_IMAGE + COPIER_HEADER {
            return Err(Refusal::TooLarge(image.len()));
        }
        let skipped = if image.len() > COPIER_HEADER && image.len() % 0x4000 == COPIER_HEADER { COPIER_HEADER } else { 0 };
        let rom = &image[skipped..];
        let header = HEADER_PLACES.iter().find_map(|&at| header_at(rom, at));
        let guessed = if header.is_none() && rom.len() <= SG1000_MAX { Console::Sg1000 } else { Console::Sms };
        let console = console.unwrap_or(guessed);
        Ok(Media { console, skipped, header })
    }

    /// The market a game names, where its header names one.
    pub fn market(&self) -> Option<Market> {
        self.header.and_then(|h| h.market())
    }
}

fn header_at(rom: &[u8], at: usize) -> Option<Header> {
    let h = rom.get(at..at + 16)?;
    if &h[..8] != SIGNATURE {
        return None;
    }
    let bcd = |b: u8| u32::from(b >> 4) * 10 + u32::from(b & 0xF);
    let size_code = h[15] & 0xF;
    Some(Header {
        at,
        checksum: u16::from_le_bytes([h[10], h[11]]),
        computed: checksum(rom, at, size_code),
        product: bcd(h[12]) + bcd(h[13]) * 100 + u32::from(h[14] >> 4) * 10_000,
        version: h[14] & 0xF,
        region: h[15] >> 4,
        size_code,
    })
}

/// The 16-bit sum of every byte the size code's range holds, the header's sixteen left out, as the library's export
/// cartridges were measured to state it (Endou_OpenBoot.md §1.6); the range ends at the image's end where the image is
/// shorter (Endou_Native.md §2.2).
pub fn checksum(rom: &[u8], header_at: usize, size_code: u8) -> u16 {
    let end = size_of_code(size_code).unwrap_or(header_at + 16).min(rom.len());
    let skip = header_at..header_at + 16;
    rom[..end].iter().enumerate().filter(|(i, _)| !skip.contains(i)).fold(0u16, |s, (_, &b)| s.wrapping_add(u16::from(b)))
}

#[cfg(test)]
pub mod sample {
    use super::*;

    /// A cartridge of `size` bytes with Sega's header at `$7FF0`: product 7026, version 1, `region` and the size code
    /// for 32 KiB, its checksum right; a loop at `$0000`.
    pub fn cartridge(size: usize, region: u8) -> Vec<u8> {
        let mut rom = vec![0u8; size];
        rom[..3].copy_from_slice(&[0xF3, 0x18, 0xFE]);
        rom[0x7FF0..0x7FF8].copy_from_slice(SIGNATURE);
        rom[0x7FFC..0x7FFF].copy_from_slice(&[0x26, 0x70, 0x01]);
        rom[0x7FFF] = region << 4 | 0xC;
        let sum = checksum(&rom, 0x7FF0, 0xC);
        rom[0x7FFA..0x7FFC].copy_from_slice(&sum.to_le_bytes());
        rom
    }
}

#[cfg(test)]
mod tests {
    use super::sample::cartridge;
    use super::*;

    #[test]
    fn the_header_is_read_where_sega_put_it() {
        let rom = cartridge(0x8000, 0x4);
        let m = Media::read(&rom).unwrap();
        let h = m.header.unwrap();
        assert_eq!((m.console, m.skipped, h.at, h.product, h.version, h.size_code), (Console::Sms, 0, 0x7FF0, 7026, 1, 0xC));
        assert!(h.checksum_agrees());
        assert_eq!(m.market(), Some(Market::Americas));
        let mut japan = cartridge(0x8000, 0x3);
        japan[0x100] ^= 1;
        let h = Media::read(&japan).unwrap().header.unwrap();
        assert!(!h.checksum_agrees(), "a changed byte changes the sum");
        assert_eq!(h.market(), Some(Market::Japan));
    }

    #[test]
    fn the_checksum_leaves_out_the_header_and_stops_at_the_coded_size() {
        let mut rom = cartridge(0x10000, 0x4);
        let before = Media::read(&rom).unwrap().header.unwrap().computed;
        rom[0x9000] = 0xFF;
        assert_eq!(Media::read(&rom).unwrap().header.unwrap().computed, before, "the size code names 32 KiB");
        rom[0x7FFA] ^= 0xFF;
        rom[0x7FFE] ^= 0x01;
        assert_eq!(Media::read(&rom).unwrap().header.unwrap().computed, before, "the header's bytes are not summed");
        assert_eq!(size_of_code(0x3), None);
    }

    #[test]
    fn a_copier_header_is_set_aside_and_the_console_follows_the_header() {
        let mut copied = vec![0u8; COPIER_HEADER];
        copied.extend(cartridge(0x8000, 0x4));
        let m = Media::read(&copied).unwrap();
        assert_eq!((m.skipped, m.header.map(|h| h.at)), (COPIER_HEADER, Some(0x7FF0)));
        assert!(m.header.unwrap().checksum_agrees());
        assert_eq!(Media::read(&[0xC3; 0x8000]).unwrap().console, Console::Sg1000);
        assert_eq!(Media::read(&[0xC3; SG1000_MAX + 1]).unwrap().console, Console::Sms);
        let mut small = vec![0u8; 0x4000];
        small[0x3FF0..0x3FF8].copy_from_slice(SIGNATURE);
        let m = Media::read(&small).unwrap();
        assert_eq!((m.console, m.header.unwrap().at), (Console::Sms, 0x3FF0));
    }

    // The three images of the library the stage-1 rule takes for the other console, each read as the system it was routed to (Endou_Native.md §2.3).
    #[test]
    fn a_routed_console_is_taken_and_the_rule_guesses_only_without_one() {
        let music = vec![0xC3; 512 * 1024];
        let bios = vec![0xF3; 8 * 1024];
        assert_eq!((Media::read(&music).unwrap().console, Media::read(&bios).unwrap().console), (Console::Sms, Console::Sg1000));
        assert_eq!(Media::read_as(&music, Some(Console::Sg1000)).unwrap().console, Console::Sg1000);
        assert_eq!(Media::read_as(&bios, Some(Console::Sms)).unwrap().console, Console::Sms);
        let headed = cartridge(0x8000, 0x4);
        assert_eq!(Media::read_as(&headed, Some(Console::Sg1000)).unwrap(), Media { console: Console::Sg1000, ..Media::read(&headed).unwrap() });
        assert_eq!(Media::read_as(&[], Some(Console::Sms)), Err(Refusal::Empty));
        assert_eq!(Console::ALL.map(|c| Console::of_id(c.id())), [Some(Console::Sms), Some(Console::Sg1000)]);
        assert_eq!(Console::of_id("gg"), None);
    }

    #[test]
    fn no_bytes_and_too_many_are_refused() {
        assert_eq!(Media::read(&[]), Err(Refusal::Empty));
        assert_eq!(Media::read(&vec![0; MAX_IMAGE + COPIER_HEADER + 1]), Err(Refusal::TooLarge(MAX_IMAGE + COPIER_HEADER + 1)));
        assert!(Media::read(&[0x5A]).is_ok());
    }
}
