//! An iNES or NES 2.0 header read without building a board; byte 7's bits 2-3 reading exactly 0b10 is the whole NES 2.0
//! detection (Moon_Memory.md §2).

/// What a header gives, field for field `NesHeader`'s; `trust` and `mirroring` by their C# members' numbers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Header {
    pub mapper: i32,
    pub prg_banks: i32,
    pub chr_banks: i32,
    pub battery: bool,
    pub trainer: bool,
    pub nes20: bool,
    /// Clean 0, Unverifiable 1, Archaic 2.
    pub trust: u8,
    /// Horizontal 0, Vertical 1, FourScreen 4.
    pub mirroring: u8,
}

pub const SIZE: usize = 16;
pub const PRG_BANK: i64 = 0x4000;
pub const TRAINER: i64 = 512;

/// The header of an image of `length` bytes whose first bytes are `head`; the words of the C#'s InvalidDataException
/// where it is no iNES image or claims more PRG than the file holds.
pub fn read(head: &[u8], length: usize) -> Result<Header, String> {
    if length < SIZE || head.len() < SIZE || head[..4] != *b"NES\x1A" {
        return Err("Not an iNES image: missing the \"NES\\x1A\" magic.".into());
    }
    let nes20 = head[7] & 0x0C == 0x08;
    let archaic = !nes20 && (head[7] & 0x0C != 0 || head[12..16].iter().any(|&b| b != 0));
    let (mut prg, mut chr) = (head[4] as i32, head[5] as i32);
    let mut mapper = (head[6] >> 4) as i32;
    if !archaic {
        mapper |= (head[7] & 0xF0) as i32;
    }
    let trust = if archaic { 2 } else if nes20 || head[7] & 0xF0 == 0 { 0 } else { 1 };
    if nes20 {
        mapper |= ((head[8] & 0x0F) as i32) << 8;
        prg |= ((head[9] & 0x0F) as i32) << 8;
        chr |= ((head[9] & 0xF0) as i32) << 4;
    }
    let mirroring = if head[6] & 0x08 != 0 { 4 } else if head[6] & 0x01 != 0 { 1 } else { 0 };
    let trainer = head[6] & 0x04 != 0;
    let offset = SIZE as i64 + if trainer { TRAINER } else { 0 };
    if offset + prg as i64 * PRG_BANK > length as i64 {
        return Err(format!("Header claims {prg} PRG bank(s) ({} bytes) but the file holds {}.", prg as i64 * PRG_BANK, length as i64 - offset));
    }
    Ok(Header { mapper, prg_banks: prg, chr_banks: chr, battery: head[6] & 0x02 != 0, trainer, nes20, trust, mirroring })
}
