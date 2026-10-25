//! `emusen-patch`: soft patches applied to an image, a component of its own (EmuSen_GamePackages.md §2.3). The three
//! formats are pure functions, bytes in and bytes out, with no file, clock or state; each is written from its published
//! description, cited in its module, and allocates nothing but the image it returns. A patch that carries the source's
//! checksum (UPS, BPS) is checked against the image before a byte is written, and refused with words naming both;
//! IPS carries none, so a package's list pins it to the image's SHA-1 when it is added (`apply_list`). `package`
//! composes the formats with Galaxia's manifest, which is the one place this crate reads files.

pub mod bps;
pub mod ips;
pub mod package;
pub mod ups;

use emusen_galaxia::rom_hash::Crc32;

/// The largest image a patch may make: the largest cartridge the project runs, an N64 image of 64 MiB (§5.4).
pub const MAX_IMAGE: u64 = 64 << 20;

/// A patch file's format, by the marker it begins with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Ips,
    Ups,
    Bps,
}

impl Format {
    /// The manifest's spelling, which is also the file's extension.
    pub fn name(self) -> &'static str {
        match self {
            Format::Ips => "ips",
            Format::Ups => "ups",
            Format::Bps => "bps",
        }
    }

    /// The format a manifest names.
    pub fn named(name: &str) -> Option<Format> {
        match name {
            "ips" => Some(Format::Ips),
            "ups" => Some(Format::Ups),
            "bps" => Some(Format::Bps),
            _ => None,
        }
    }

    /// The format a patch's first bytes mark: `PATCH`, `UPS1` or `BPS1`.
    pub fn of(patch: &[u8]) -> Option<Format> {
        if patch.starts_with(b"PATCH") {
            Some(Format::Ips)
        } else if patch.starts_with(b"UPS1") {
            Some(Format::Ups)
        } else if patch.starts_with(b"BPS1") {
            Some(Format::Bps)
        } else {
            None
        }
    }

    fn upper(self) -> &'static str {
        match self {
            Format::Ips => "IPS",
            Format::Ups => "UPS",
            Format::Bps => "BPS",
        }
    }
}

/// Why a patch was not applied. Every refusal is words for the player; none leaves a partial image behind.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// No marker of the three formats.
    NotAPatch,
    /// A patch whose own structure or checksum is wrong.
    Damaged(String),
    /// The image is not the one the patch was made for: what the patch records, and what the image is.
    WrongSource { format: Format, expected_crc32: u32, expected_size: u64, found_crc32: u32, found_size: u64 },
    /// The image is the patch's target already, so applying it again would be a mistake.
    AlreadyTarget { format: Format, crc32: u32 },
    /// The image the patch would make is larger than `MAX_IMAGE`.
    TooLarge(u64),
    /// The patch was applied and its result is not the target it records.
    WrongTarget { format: Format, expected_crc32: u32, found_crc32: u32 },
}

impl Refusal {
    pub fn words(&self) -> String {
        match self {
            Refusal::NotAPatch => "it is not an IPS, UPS or BPS patch: it begins with none of PATCH, UPS1 or BPS1".to_string(),
            Refusal::Damaged(words) => words.clone(),
            Refusal::WrongSource { format, expected_crc32, expected_size, found_crc32, found_size } => format!(
                "the {} patch was made for an image of {expected_size} bytes with CRC-32 {expected_crc32:08x}, and this image has {found_size} bytes with CRC-32 {found_crc32:08x}",
                format.upper()
            ),
            Refusal::AlreadyTarget { format, crc32 } => format!("this image is already the {} patch's target (CRC-32 {crc32:08x}): it has been patched before", format.upper()),
            Refusal::TooLarge(size) => format!("the patched image would be {size} bytes, more than the {} a patched image may be", MAX_IMAGE),
            Refusal::WrongTarget { format, expected_crc32, found_crc32 } => {
                format!("the {} patch made an image with CRC-32 {found_crc32:08x}, and it records its target as {expected_crc32:08x}", format.upper())
            }
        }
    }
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.words())
    }
}

/// What a patch says of itself before it is applied: its format, and where it records them, the sizes and CRC-32s of
/// its source and target. IPS records none.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Info {
    pub format: Format,
    pub source_size: Option<u64>,
    pub target_size: Option<u64>,
    pub source_crc32: Option<u32>,
    pub target_crc32: Option<u32>,
}

/// The patch read and its structure and own checksum checked, with no image.
pub fn info(patch: &[u8]) -> Result<Info, Refusal> {
    match Format::of(patch) {
        Some(Format::Ips) => ips::scan(patch).map(|_| Info { format: Format::Ips, source_size: None, target_size: None, source_crc32: None, target_crc32: None }),
        Some(Format::Ups) => ups::header(patch).map(|h| h.info(Format::Ups)),
        Some(Format::Bps) => bps::header(patch).map(|h| h.info(Format::Bps)),
        None => Err(Refusal::NotAPatch),
    }
}

/// The patch applied to `source`, by the format its marker names.
pub fn apply(source: &[u8], patch: &[u8]) -> Result<Vec<u8>, Refusal> {
    match Format::of(patch) {
        Some(Format::Ips) => ips::apply(source, patch),
        Some(Format::Ups) => ups::apply(source, patch),
        Some(Format::Bps) => bps::apply(source, patch),
        None => Err(Refusal::NotAPatch),
    }
}

/// One patch of a list: its name for the words, the format the list records, its bytes, and for IPS the SHA-1 of the
/// image it was added for, which is the image at its place in the list.
#[derive(Clone, Copy, Debug)]
pub struct Step<'a> {
    pub name: &'a str,
    pub format: Format,
    pub bytes: &'a [u8],
    pub source_sha1: Option<&'a str>,
}

/// Each patch applied in order to the image the one before made, the first to `image`; words naming the patch that
/// refused. An IPS patch with a recorded source SHA-1 is refused unless the image it is given has it.
pub fn apply_list(image: &[u8], steps: &[Step]) -> Result<Vec<u8>, String> {
    let mut current: Option<Vec<u8>> = None;
    for step in steps {
        let input = current.as_deref().unwrap_or(image);
        let marked = Format::of(step.bytes);
        if marked != Some(step.format) {
            let what = marked.map_or("no patch at all", |f| f.upper());
            return Err(format!("{}: the package records it as {}, and the file is {what}", step.name, step.format.upper()));
        }
        if let Some(want) = step.source_sha1 {
            let found = emusen_galaxia::rom_hash::sha1_of(input);
            if !found.eq_ignore_ascii_case(want) {
                return Err(format!("{}: the IPS patch was added for an image with SHA-1 {}, and this image has SHA-1 {found}", step.name, want.to_ascii_lowercase()));
            }
        }
        current = Some(apply(input, step.bytes).map_err(|r| format!("{}: {}", step.name, r.words()))?);
    }
    Ok(current.unwrap_or_else(|| image.to_vec()))
}

/// CRC-32 of bytes, RomHash's.
pub(crate) fn crc32(bytes: &[u8]) -> u32 {
    let mut c = Crc32::default();
    c.append(bytes);
    c.value()
}

pub(crate) fn damaged(words: impl Into<String>) -> Refusal {
    Refusal::Damaged(words.into())
}

/// A little-endian 32-bit value at `at`.
pub(crate) fn le32(p: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([p[at], p[at + 1], p[at + 2], p[at + 3]])
}

/// The variable-length number UPS and BPS share, decoded as both specifications give it: seven bits a byte, low first,
/// the top bit marking the last byte, and one added at each continuation so that no number has two spellings. A
/// number that runs to `end` or past 64 bits is a damaged patch.
pub(crate) fn number(p: &[u8], at: &mut usize, end: usize) -> Result<u64, Refusal> {
    let overflow = || damaged("a number in the patch is larger than 64 bits");
    let (mut data, mut shift) = (0u64, 1u64);
    loop {
        if *at >= end {
            return Err(damaged("a number runs into the patch's checksums"));
        }
        let x = p[*at];
        *at += 1;
        data = ((x & 0x7f) as u64).checked_mul(shift).and_then(|v| data.checked_add(v)).ok_or_else(overflow)?;
        if x & 0x80 != 0 {
            return Ok(data);
        }
        shift = shift.checked_mul(128).ok_or_else(overflow)?;
        data = data.checked_add(shift).ok_or_else(overflow)?;
    }
}

/// The header both checksummed formats begin with, and the three checksums of their footer.
pub(crate) struct Checked {
    pub source_size: u64,
    pub target_size: u64,
    pub source_crc32: u32,
    pub target_crc32: u32,
    /// Where the body begins, and where the footer does.
    pub body: usize,
    pub footer: usize,
}

impl Checked {
    fn info(&self, format: Format) -> Info {
        Info { format, source_size: Some(self.source_size), target_size: Some(self.target_size), source_crc32: Some(self.source_crc32), target_crc32: Some(self.target_crc32) }
    }

    /// The image checked against the source the footer records, before anything is written.
    pub fn source(&self, format: Format, source: &[u8]) -> Result<(), Refusal> {
        let found = crc32(source);
        if source.len() as u64 == self.source_size && found == self.source_crc32 {
            return Ok(());
        }
        if source.len() as u64 == self.target_size && found == self.target_crc32 {
            return Err(Refusal::AlreadyTarget { format, crc32: found });
        }
        Err(Refusal::WrongSource { format, expected_crc32: self.source_crc32, expected_size: self.source_size, found_crc32: found, found_size: source.len() as u64 })
    }

    /// The result checked against the target the footer records.
    pub fn target(&self, format: Format, target: &[u8]) -> Result<(), Refusal> {
        let found = crc32(target);
        if found == self.target_crc32 { Ok(()) } else { Err(Refusal::WrongTarget { format, expected_crc32: self.target_crc32, found_crc32: found }) }
    }
}

/// The marker, the patch's own CRC-32 over every byte but its last four, and the footer's two others, read
/// little-endian as vidua's notes on BPS give them; the body begins after the marker.
pub(crate) fn footer(patch: &[u8], marker: &[u8], least: usize) -> Result<(usize, u32, u32), Refusal> {
    if !patch.starts_with(marker) {
        return Err(Refusal::NotAPatch);
    }
    if patch.len() < marker.len() + least + 12 {
        return Err(damaged(format!("it is {} bytes, too short to hold its header and its three checksums", patch.len())));
    }
    let end = patch.len() - 12;
    let recorded = le32(patch, patch.len() - 4);
    let own = crc32(&patch[..patch.len() - 4]);
    if own != recorded {
        return Err(damaged(format!("the patch file is damaged: its bytes have CRC-32 {own:08x}, and it records {recorded:08x}")));
    }
    Ok((end, le32(patch, end), le32(patch, end + 4)))
}
