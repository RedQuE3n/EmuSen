//! What an image is, read from its bytes: a PS-EXE by its "PS-X EXE" header, or a disc by the licence text in sector 4
//! of its system area (psx-spx "CDROM File Formats" and "CDROM ISO Volume Descriptors"). Recognition only: nothing is
//! loaded or run (Diamond_Native.md §2.2).

/// A PS-EXE's identifier, and the header the program follows.
pub const EXE_MAGIC: &[u8; 8] = b"PS-X EXE";
pub const EXE_HEADER: usize = 0x800;

/// The licence string's first line, at the start of sector 4's user data.
pub const LICENCE: &[u8; 32] = b"          Licensed  by          ";
pub const LICENCE_SECTOR: usize = 4;

/// A sector's user data in a 2,048-byte image, and a whole sector in a raw 2,352-byte one with its sync pattern.
pub const USER_SECTOR: usize = 2048;
pub const RAW_SECTOR: usize = 2352;
pub const SYNC: [u8; 12] = [0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00];

/// The market an image names: a disc's licence text, a PS-EXE's marker.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Market {
    Japan,
    Americas,
    Europe,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A program: its entry, where it loads and how many bytes follow the header.
    Exe { pc: u32, load: u32, size: u32 },
    /// A disc's first data track, of 2,048-byte user sectors or of raw 2,352-byte ones.
    Disc { raw: bool },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Media {
    pub kind: Kind,
    /// `None` when the image names no market: a homebrew PS-EXE without the marker.
    pub market: Option<Market>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// Neither a PS-EXE nor a disc with the licence text.
    NotPlayStation,
    /// A PS-EXE shorter than its header and the size it states.
    Truncated { needs: usize, has: usize },
}

fn word(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

fn exe_market(header: &[u8]) -> Option<Market> {
    let marker = &header[0x4C..EXE_HEADER];
    let names = |text: &[u8]| marker.windows(text.len()).any(|w| w == text);
    if names(b"for Japan area") {
        Some(Market::Japan)
    } else if names(b"for North America area") {
        Some(Market::Americas)
    } else if names(b"for Europe area") {
        Some(Market::Europe)
    } else {
        None
    }
}

/// The licence's second line names Sony's company for the market (psx-spx "System Area").
fn licence_market(line: &[u8]) -> Option<Market> {
    let rest = line.strip_prefix(b"Sony Computer Entertainment ")?;
    match rest.get(..4)? {
        b"Inc." => Some(Market::Japan),
        b"Amer" => Some(Market::Americas),
        b"Euro" => Some(Market::Europe),
        _ => None,
    }
}

/// Sector 4's user data, from a 2,048-byte image or a raw one of mode 1 or mode 2.
fn licence_sector(image: &[u8]) -> Option<(&[u8], bool)> {
    let user = LICENCE_SECTOR * USER_SECTOR;
    if image.get(user..user + LICENCE.len()) == Some(LICENCE) {
        return Some((&image[user..], false));
    }
    let raw = LICENCE_SECTOR * RAW_SECTOR;
    if image.get(..12) != Some(&SYNC) || image.get(raw..raw + 12) != Some(&SYNC) {
        return None;
    }
    let data = raw + if image.get(raw + 15) == Some(&1) { 16 } else { 24 };
    (image.get(data..data + LICENCE.len()) == Some(LICENCE)).then(|| (&image[data..], true))
}

impl Media {
    pub fn read(image: &[u8]) -> Result<Media, Refusal> {
        if image.starts_with(EXE_MAGIC) {
            if image.len() < EXE_HEADER {
                return Err(Refusal::Truncated { needs: EXE_HEADER, has: image.len() });
            }
            let size = word(image, 0x1C);
            let needs = EXE_HEADER + size as usize;
            if image.len() < needs {
                return Err(Refusal::Truncated { needs, has: image.len() });
            }
            return Ok(Media { kind: Kind::Exe { pc: word(image, 0x10), load: word(image, 0x18), size }, market: exe_market(&image[..EXE_HEADER]) });
        }
        match licence_sector(image) {
            Some((data, raw)) => Ok(Media { kind: Kind::Disc { raw }, market: data.get(0x20..0x46).and_then(licence_market) }),
            None => Err(Refusal::NotPlayStation),
        }
    }
}

/// Test images: a PS-EXE with its header, and a disc's system area as either kind of image.
#[cfg(test)]
pub mod sample {
    use super::*;

    pub fn exe(market: Option<&str>, code: usize) -> Vec<u8> {
        let size = code.div_ceil(USER_SECTOR) * USER_SECTOR;
        let mut b = vec![0u8; EXE_HEADER + size];
        b[..8].copy_from_slice(EXE_MAGIC);
        b[0x10..0x14].copy_from_slice(&0x8001_0000u32.to_le_bytes());
        b[0x18..0x1C].copy_from_slice(&0x8001_0000u32.to_le_bytes());
        b[0x1C..0x20].copy_from_slice(&(size as u32).to_le_bytes());
        b[0x30..0x34].copy_from_slice(&0x801F_FFF0u32.to_le_bytes());
        if let Some(m) = market {
            let text = format!("Sony Computer Entertainment Inc. for {m} area");
            b[0x4C..0x4C + text.len()].copy_from_slice(text.as_bytes());
        }
        b
    }

    pub fn line2(market: Market) -> &'static [u8] {
        match market {
            Market::Japan => b"Sony Computer Entertainment Inc.\n",
            Market::Americas => b"Sony Computer Entertainment Amer  ica ",
            Market::Europe => b"Sony Computer Entertainment Euro pe   ",
        }
    }

    pub fn disc(market: Market, raw: Option<u8>) -> Vec<u8> {
        let sectors = 20;
        let (size, data) = match raw {
            Some(mode) => (RAW_SECTOR, if mode == 1 { 16 } else { 24 }),
            None => (USER_SECTOR, 0),
        };
        let mut b = vec![0u8; sectors * size];
        for s in 0..sectors {
            if let Some(mode) = raw {
                b[s * size..s * size + 12].copy_from_slice(&SYNC);
                b[s * size + 15] = mode;
            }
        }
        let at = LICENCE_SECTOR * size + data;
        b[at..at + 32].copy_from_slice(LICENCE);
        let l2 = line2(market);
        b[at + 32..at + 32 + l2.len()].copy_from_slice(l2);
        b
    }
}

#[cfg(test)]
mod tests {
    use super::sample::*;
    use super::*;

    #[test]
    fn a_ps_exe_is_read_from_its_header_with_its_market() {
        let m = Media::read(&exe(Some("North America"), 100)).unwrap();
        assert_eq!(m, Media { kind: Kind::Exe { pc: 0x8001_0000, load: 0x8001_0000, size: 0x800 }, market: Some(Market::Americas) });
        assert_eq!(Media::read(&exe(Some("Japan"), 1)).unwrap().market, Some(Market::Japan));
        assert_eq!(Media::read(&exe(Some("Europe"), 1)).unwrap().market, Some(Market::Europe));
        assert_eq!(Media::read(&exe(None, 1)).unwrap().market, None);
    }

    #[test]
    fn a_truncated_ps_exe_is_refused_with_what_it_needs() {
        let e = exe(None, 0x1000);
        assert_eq!(Media::read(&e[..e.len() - 1]), Err(Refusal::Truncated { needs: 0x1800, has: 0x17FF }));
        assert_eq!(Media::read(&e[..0x10]), Err(Refusal::Truncated { needs: 0x800, has: 0x10 }));
        assert!(Media::read(&[e.clone(), vec![0; 7]].concat()).is_ok(), "bytes past the stated size are padding");
    }

    #[test]
    fn a_disc_is_read_from_its_licence_in_every_image_form() {
        for market in [Market::Japan, Market::Americas, Market::Europe] {
            assert_eq!(Media::read(&disc(market, None)).unwrap(), Media { kind: Kind::Disc { raw: false }, market: Some(market) });
            assert_eq!(Media::read(&disc(market, Some(2))).unwrap(), Media { kind: Kind::Disc { raw: true }, market: Some(market) });
            assert_eq!(Media::read(&disc(market, Some(1))).unwrap(), Media { kind: Kind::Disc { raw: true }, market: Some(market) });
        }
    }

    #[test]
    fn anything_else_is_refused() {
        assert_eq!(Media::read(&[]), Err(Refusal::NotPlayStation));
        assert_eq!(Media::read(&[0x5A; 0x10000]), Err(Refusal::NotPlayStation));
        let mut no_licence = disc(Market::Japan, None);
        no_licence[LICENCE_SECTOR * USER_SECTOR] = b'X';
        assert_eq!(Media::read(&no_licence), Err(Refusal::NotPlayStation));
        assert_eq!(Media::read(&disc(Market::Japan, Some(2))[..LICENCE_SECTOR * RAW_SECTOR + 30]), Err(Refusal::NotPlayStation));
    }
}
