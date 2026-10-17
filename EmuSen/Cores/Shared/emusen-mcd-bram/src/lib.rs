//! The Sega CD's backup RAM file system, the internal RAM's and a RAM cartridge's alike, as Nephrite_Bram.md states
//! it: a footer naming the format with its free and file counts four times over, a directory growing down from the
//! footer two entries to a block, each block stored in the block code (§6), and files placed up from block 1.
//!
//! No standard library and no allocation: every function reads an image or edits one in place, so a host's card tool
//! and an emulated console's open firmware can share it (EmuSen_Backend.md D14). Where a damaged directory sends an
//! edit outside the volume, the edit is refused with the reason rather than made. See
//! Hardware/Shared/EmuSen_MediumFormats.md.

#![no_std]

pub mod block_code;

pub const BLOCK_LENGTH: usize = 64;
pub const MIN_LENGTH: usize = 0x2000;
pub const MAX_LENGTH: usize = 0x8_0000;
pub const NAME_LENGTH: usize = 11;

const IDENTIFIER: &[u8; 12] = b"SEGA_CD_ROM\0";
const CARTRIDGE: &[u8; 16] = b"RAM_CARTRIDGE___";

/// A volume's length: 8 KiB to 512 KiB, a power of two.
pub fn size_ok(length: usize) -> bool {
    (MIN_LENGTH..=MAX_LENGTH).contains(&length) && length.is_power_of_two()
}

/// The blocks a formatted volume of `length` has for files and directory.
pub fn capacity(length: usize) -> usize {
    length / BLOCK_LENGTH - 3
}

/// What a volume's footer says it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Sega,
    Unformatted,
    /// Another format's name, of 0-9, A-Z and _, in the footer.
    Other,
}

/// §3: RAM_CARTRIDGE___ needs one of its first three longwords whole; then the identifier exactly, or another name of
/// 0-9, A-Z and _. `image` must be at least 32 bytes.
pub fn kind(image: &[u8]) -> Kind {
    let s = image.len();
    if !(0..3).any(|k| image[s - 0x10 + 4 * k..s - 0x10 + 4 * k + 4] == CARTRIDGE[4 * k..4 * k + 4]) {
        return Kind::Unformatted;
    }
    if image[s - 0x20..s - 0x20 + 12] == IDENTIFIER[..] {
        return Kind::Sega;
    }
    if image[s - 0x20..s - 0x20 + 11].iter().all(|&c| c == b'_' || c.is_ascii_digit() || c.is_ascii_uppercase()) {
        Kind::Other
    } else {
        Kind::Unformatted
    }
}

/// The footer's format name, its 11 bytes.
pub fn format_name(image: &[u8]) -> &[u8] {
    &image[image.len() - 0x20..image.len() - 0x20 + 11]
}

fn be16(bytes: &[u8]) -> u16 {
    u16::from_be_bytes([bytes[0], bytes[1]])
}

/// §3: the value three or four of the four copies hold, else 0.
fn count(image: &[u8], at: usize) -> usize {
    let v: [u16; 4] = core::array::from_fn(|k| be16(&image[at + 2 * k..]));
    for c in 0..2 {
        if v.iter().filter(|&&x| x == v[c]).count() >= 3 {
            return v[c] as usize;
        }
    }
    0
}

/// The free blocks and the files the footer counts; none of either where it counts more files than the volume holds.
pub fn counts(image: &[u8]) -> (usize, usize) {
    let s = image.len();
    let (free, files) = (count(image, s - 0x30), count(image, s - 0x28));
    if files > s / 32 { (0, 0) } else { (free, files) }
}

/// Each count written four times, as 16 bits.
pub fn set_counts(image: &mut [u8], free: u16, files: u16) {
    let s = image.len();
    for k in 0..4 {
        image[s - 0x30 + 2 * k..s - 0x30 + 2 * k + 2].copy_from_slice(&free.to_be_bytes());
        image[s - 0x28 + 2 * k..s - 0x28 + 2 * k + 2].copy_from_slice(&files.to_be_bytes());
    }
}

/// Where directory block `block` begins; negative where it would lie below the volume.
fn directory_at(length: usize, block: usize) -> i64 {
    length as i64 - 0x80 - (BLOCK_LENGTH * block) as i64
}

/// §5.4: `files` files are accounted (files + 2) div 2 directory blocks.
pub fn directory_blocks(files: usize) -> usize {
    (files + 2) / 2
}

/// A directory entry as stored: its name's bytes, the protected flag, its first block and its length in blocks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    pub name: [u8; NAME_LENGTH],
    pub protected: bool,
    pub start: usize,
    pub blocks: usize,
    /// Its directory block past correcting, or its blocks outside the files' area.
    pub damaged: bool,
}

/// Entry `index`, numbered from 1; none where its directory block would lie outside the volume.
pub fn entry(image: &[u8], index: usize) -> Option<Entry> {
    let at = directory_at(image.len(), (index - 1) / 2);
    if at < 0 {
        return None;
    }
    let at = at as usize;
    let mut u = [0u8; 32];
    let whole = block_code::decode(&image[at..at + BLOCK_LENGTH], &mut u) >= 0;
    let e = if index % 2 == 1 { &u[16..32] } else { &u[0..16] };
    let (start, blocks) = (be16(&e[12..]) as usize, be16(&e[14..]) as usize);
    let inside = start >= 1 && ((start + blocks) * BLOCK_LENGTH) as i64 <= directory_at(image.len(), 0);
    let mut name = [0u8; NAME_LENGTH];
    name.copy_from_slice(&e[..NAME_LENGTH]);
    Some(Entry { name, protected: e[11] != 0, start, blocks, damaged: !whole || !inside })
}

/// The footer's files, entries 1 to its count; a `None` among them is an entry outside the volume.
pub fn entries(image: &[u8]) -> impl Iterator<Item = Option<Entry>> + '_ {
    (1..=counts(image).1).map(move |i| entry(image, i))
}

fn entry_bytes(e: &Entry, start: i64) -> [u8; 16] {
    let mut b = [0u8; 16];
    b[..NAME_LENGTH].copy_from_slice(&e.name);
    b[11] = if e.protected { 0xFF } else { 0 };
    b[12..14].copy_from_slice(&(start as u16).to_be_bytes());
    b[14..16].copy_from_slice(&(e.blocks as u16).to_be_bytes());
    b
}

/// Directory block `block` of `out` from the two entries it holds, `slot` giving each by its place in the list with
/// its start; a slot past the list holds zeros.
fn write_directory(out: &mut [u8], block: usize, slot: impl Fn(usize) -> Option<(Entry, i64)>) {
    let mut u = [0u8; 32];
    if let Some((e, start)) = slot(2 * block) {
        u[16..].copy_from_slice(&entry_bytes(&e, start));
    }
    if let Some((e, start)) = slot(2 * block + 1) {
        u[..16].copy_from_slice(&entry_bytes(&e, start));
    }
    let at = directory_at(out.len(), block) as usize;
    block_code::encode(&u, &mut out[at..at + BLOCK_LENGTH]);
}

/// Why an edit was refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditError {
    FromNotFormatted,
    ToNotFormatted,
    NotFormatted,
    /// An entry's directory block lies outside the volume, so the directory cannot be read whole.
    DirectoryOutside,
    Damaged,
    NoRoom { needed: usize, free: usize },
    /// A damaged directory places the files after the one deleted outside the volume, so they cannot be moved down.
    MoveOutside,
}

fn formatted(image: &[u8]) -> bool {
    size_ok(image.len()) && kind(image) == Kind::Sega
}

/// Every entry of the footer's count read, or the refusal where one lies outside the volume.
fn whole(image: &[u8]) -> Result<usize, EditError> {
    let files = counts(image).1;
    if entries(image).any(|e| e.is_none()) { Err(EditError::DirectoryOutside) } else { Ok(files) }
}

/// Entry `j` of the list, numbered from 0, with its start.
fn listed(image: &[u8], j: usize) -> (Entry, i64) {
    let e = entry(image, j + 1).expect("read whole");
    (e, e.start as i64)
}

/// As BRMWRITE places a file (§7.2): entry `index` of `from` copied after `to`'s last file, its blocks as they were
/// stored; `to` is unchanged on a refusal.
pub fn copy(from: &[u8], index: usize, to: &mut [u8]) -> Result<(), EditError> {
    if !formatted(from) {
        return Err(EditError::FromNotFormatted);
    }
    if !formatted(to) {
        return Err(EditError::ToNotFormatted);
    }
    let source = entry(from, index).ok_or(EditError::DirectoryOutside)?;
    if source.damaged {
        return Err(EditError::Damaged);
    }
    let files = whole(to)?;
    let free = counts(to).0;
    let need = source.blocks + directory_blocks(files + 1) - directory_blocks(files);
    let start = if files == 0 { 1 } else { let (last, at) = listed(to, files - 1); at + last.blocks as i64 };
    if need > free || (start + source.blocks as i64) * BLOCK_LENGTH as i64 > directory_at(to.len(), directory_blocks(files + 1) - 1) {
        return Err(EditError::NoRoom { needed: need, free });
    }
    let block = files / 2;
    let slots: [Option<(Entry, i64)>; 2] = core::array::from_fn(|k| {
        let j = 2 * block + k;
        if j < files { Some(listed(to, j)) } else if j == files { Some((source, start)) } else { None }
    });
    let (src, dst, n) = (source.start * BLOCK_LENGTH, start as usize * BLOCK_LENGTH, source.blocks * BLOCK_LENGTH);
    to[dst..dst + n].copy_from_slice(&from[src..src + n]);
    write_directory(to, block, |j| slots[j - 2 * block]);
    set_counts(to, (free - need) as u16, (files + 1) as u16);
    Ok(())
}

/// As BRMDEL (§7.3): entry `index`'s file removed from `image` into `edited`, which must hold `image`'s bytes: the
/// later files moved down over it and their entries one earlier, the vacated blocks left as they were. Every read is
/// of `image`, so a damaged directory moving data over itself reads as it was; `edited` is unchanged on a refusal.
pub fn delete(image: &[u8], index: usize, edited: &mut [u8]) -> Result<(), EditError> {
    assert_eq!(image.len(), edited.len(), "the edit is made on a copy of the image");
    if !formatted(image) {
        return Err(EditError::NotFormatted);
    }
    let files = whole(image)?;
    let k = index - 1;
    let (gone, gone_start) = listed(image, k);
    let (last, last_start) = listed(image, files - 1);
    let end = last_start + last.blocks as i64;
    let block = BLOCK_LENGTH as i64;
    let gone_blocks = gone.blocks as i64;
    let (from, length, to) = ((gone_start + gone_blocks) * block, (end - gone_start - gone_blocks) * block, gone_start * block);
    // The target lies at or below the range, since a file's length is never negative, so the range fitting is enough.
    if from < 0 || length < 0 || from + length > image.len() as i64 {
        return Err(EditError::MoveOutside);
    }
    edited[to as usize..(to + length) as usize].copy_from_slice(&image[from as usize..(from + length) as usize]);
    let after = |j: usize| -> Option<(Entry, i64)> {
        if j + 1 >= files {
            None
        } else if j < k {
            Some(listed(image, j))
        } else {
            let (e, at) = listed(image, j + 1);
            Some((e, at - gone_blocks))
        }
    };
    for b in k / 2..(files - 1).div_ceil(2) {
        write_directory(edited, b, after);
    }
    let (free, _) = counts(image);
    let free = free + gone.blocks + directory_blocks(files) - directory_blocks(files - 1);
    set_counts(edited, free as u16, (files - 1) as u16);
    Ok(())
}

/// §4: zeros, then the footer with length / 64 - 3 blocks free, as the BIOS formats a RAM of zeros. `image` must be a
/// size `size_ok` accepts.
pub fn format(image: &mut [u8]) {
    assert!(size_ok(image.len()), "a backup RAM is 8 KiB to 512 KiB, a power of two");
    image.fill(0);
    let s = image.len();
    let f = &mut image[s - 0x40..];
    f[..11].fill(0x5F);
    f[15] = 0x40;
    f[0x20..0x2C].copy_from_slice(IDENTIFIER);
    f[0x2C] = 1;
    f[0x30..0x40].copy_from_slice(CARTRIDGE);
    set_counts(image, capacity(s) as u16, 0);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blank() -> [u8; MIN_LENGTH] {
        let mut ram = [0u8; MIN_LENGTH];
        format(&mut ram);
        ram
    }

    /// A file written as BRMWRITE writes one, after the last, by the crate's own copy from a volume holding it alone.
    fn with_file(ram: &mut [u8], name: &[u8; 11], blocks: usize, fill: u8) {
        let mut one = [0u8; MIN_LENGTH];
        format(&mut one);
        let mut u = [0u8; 32];
        u[16..27].copy_from_slice(name);
        u[28..30].copy_from_slice(&1u16.to_be_bytes());
        u[30..32].copy_from_slice(&(blocks as u16).to_be_bytes());
        let at = directory_at(MIN_LENGTH, 0) as usize;
        block_code::encode(&u, &mut one[at..at + 64]);
        let (free, _) = counts(&one);
        set_counts(&mut one, (free - blocks - 1) as u16, 1);
        one[64..64 + blocks * 64].fill(fill);
        copy(&one, 1, ram).unwrap();
    }

    #[test]
    fn a_formatted_volume_is_the_segas_with_every_block_free() {
        let ram = blank();
        assert_eq!(kind(&ram), Kind::Sega);
        assert_eq!(counts(&ram), (125, 0));
        assert_eq!(entries(&ram).count(), 0);
        let mut cart = [0u8; 0x2_0000];
        format(&mut cart);
        assert_eq!(counts(&cart), (2045, 0));
        assert_eq!(kind(&[0u8; 64]), Kind::Unformatted);
    }

    #[test]
    fn files_are_placed_one_after_another_with_their_directory() {
        let mut ram = blank();
        with_file(&mut ram, b"SONIC______", 3, 0x11);
        with_file(&mut ram, b"LUNAR______", 2, 0x22);
        let a = entry(&ram, 1).unwrap();
        let b = entry(&ram, 2).unwrap();
        assert_eq!((&a.name, a.start, a.blocks, a.damaged), (b"SONIC______", 1, 3, false));
        assert_eq!((&b.name, b.start, b.blocks, b.damaged), (b"LUNAR______", 4, 2, false));
        assert_eq!(counts(&ram), (125 - 3 - 2 - 1, 2), "the second file opens a second directory block");
        assert!(ram[4 * 64..6 * 64].iter().all(|&x| x == 0x22));
    }

    #[test]
    fn a_delete_moves_the_later_files_down() {
        let mut ram = blank();
        with_file(&mut ram, b"SONIC______", 3, 0x11);
        with_file(&mut ram, b"LUNAR______", 2, 0x22);
        with_file(&mut ram, b"POPFUL_MAIL", 1, 0x33);
        let mut edited = ram;
        assert_eq!(delete(&ram, 1, &mut edited), Ok(()));
        let ram = edited;
        let a = entry(&ram, 1).unwrap();
        assert_eq!((&a.name, a.start), (b"LUNAR______", 1));
        assert_eq!(entry(&ram, 2).unwrap().start, 3);
        assert!(ram[64..3 * 64].iter().all(|&x| x == 0x22));
        assert_eq!(counts(&ram), (125 - 2 - 1 - 1, 2));
    }

    #[test]
    fn a_copy_without_room_is_refused_and_changes_nothing() {
        let mut ram = blank();
        with_file(&mut ram, b"BIG________", 120, 1);
        let mut from = blank();
        with_file(&mut from, b"MORE_______", 5, 2);
        let before = ram;
        assert_eq!(copy(&from, 1, &mut ram), Err(EditError::NoRoom { needed: 6, free: 5 }));
        assert_eq!(ram, before);
    }

    #[test]
    fn a_footer_counting_too_many_files_leaves_entries_outside() {
        let mut ram = blank();
        set_counts(&mut ram, 0, 255);
        assert_eq!(entries(&ram).last(), Some(None));
        let mut edited = ram;
        assert_eq!(delete(&ram, 1, &mut edited), Err(EditError::DirectoryOutside));
    }
}
