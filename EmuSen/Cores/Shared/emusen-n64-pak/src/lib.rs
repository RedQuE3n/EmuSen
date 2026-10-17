//! The N64 Controller Pak's file system as N64brew's "Controller Pak/Filesystem" lays it out: 128 pages of 256 bytes,
//! the ID page with its ID block at four places, an index of pages and its backup, the note table of sixteen 32-byte
//! notes over two pages, then the data pages from page 5, each note a chain through the index.
//!
//! No standard library and no allocation: every function reads an image or edits one in place, so a host's card tool
//! and an emulated console's firmware can share it (EmuSen_Backend.md D14). How a note's id is written as text is the
//! host's; this crate knows the bytes and the console's character set. See Hardware/Shared/EmuSen_MediumFormats.md.

#![no_std]

pub const PAK_LENGTH: usize = 32 * 1024;
pub const PAGE_LENGTH: usize = 256;
pub const PAGES: usize = 128;
/// The first page a note may take.
pub const FIRST_DATA: usize = 5;
pub const NOTES: usize = 16;
pub const NOTE_LENGTH: usize = 32;
/// The data pages, the pak's capacity.
pub const DATA_PAGES: usize = PAGES - FIRST_DATA;

/// The index's entries for the last page of a note, and for a page no note holds.
pub const END_OF_NOTE: u16 = 0x0001;
pub const FREE_PAGE: u16 = 0x0003;
const INDEX_PAGE: usize = 1;
const BACKUP_INDEX_PAGE: usize = 2;
const NOTE_TABLE: usize = 3 * PAGE_LENGTH;
/// Where the ID block is kept: four copies in the ID page.
pub const ID_BLOCKS: [usize; 4] = [0x20, 0x60, 0x80, 0xC0];

fn be16(bytes: &[u8]) -> u16 {
    u16::from_be_bytes([bytes[0], bytes[1]])
}

/// An ID block's two checksums: the sum of its first fourteen big-endian words, and 0xFFF2 less that.
pub fn id_checksums(block: &[u8]) -> (u16, u16) {
    let sum = (0..28).step_by(2).fold(0u16, |s, i| s.wrapping_add(be16(&block[i..])));
    (sum, 0xFFF2u16.wrapping_sub(sum))
}

fn id_valid(image: &[u8], at: usize) -> bool {
    let block = &image[at..at + 32];
    let (sum, inverse) = id_checksums(block);
    be16(&block[0x1C..]) == sum && be16(&block[0x1E..]) == inverse && block[0x19] & 1 != 0
}

/// An index page's checksum: the low byte of the sum of its bytes from 02h to its end, which it keeps at 01h.
pub fn index_sum(page: &[u8]) -> u8 {
    page[2..].iter().fold(0u8, |s, &b| s.wrapping_add(b))
}

fn page(image: &[u8], page: usize) -> &[u8] {
    &image[page * PAGE_LENGTH..(page + 1) * PAGE_LENGTH]
}

/// Whether an image is a formatted pak: the pak's length and any of its ID blocks whole.
pub fn is_formatted(image: &[u8]) -> bool {
    image.len() == PAK_LENGTH && ID_BLOCKS.iter().any(|&at| id_valid(image, at))
}

/// The index page the console's manager trusts: the first copy whose checksum holds; none where neither does, or the
/// image is too short to hold both.
pub fn index_page(image: &[u8]) -> Option<usize> {
    if image.len() < 3 * PAGE_LENGTH {
        return None;
    }
    [INDEX_PAGE, BACKUP_INDEX_PAGE].into_iter().find(|&p| index_sum(page(image, p)) == page(image, p)[1])
}

/// The index's entry for `page`: the next page of its note, `END_OF_NOTE` or `FREE_PAGE`.
pub fn entry(index: &[u8], page: usize) -> u16 {
    be16(&index[2 * page..])
}

/// Note `n`'s 32 bytes in the note table.
pub fn note(image: &[u8], n: usize) -> &[u8] {
    &image[NOTE_TABLE + n * NOTE_LENGTH..NOTE_TABLE + (n + 1) * NOTE_LENGTH]
}

/// A note's first page.
pub fn start_of(note: &[u8]) -> usize {
    be16(&note[6..]) as usize
}

/// Whether a note is in use: its first page is a data page.
pub fn in_use(note: &[u8]) -> bool {
    (FIRST_DATA..PAGES).contains(&start_of(note))
}

/// A note's pages, in order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Chain {
    pages: [u8; DATA_PAGES],
    len: usize,
}

impl Chain {
    pub fn pages(&self) -> &[u8] {
        &self.pages[..self.len]
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

/// A note's pages in order, following the index from `start`; none where the chain leaves the data pages, loops or
/// meets a free page.
pub fn chain(index: &[u8], start: usize) -> Option<Chain> {
    let mut chain = Chain { pages: [0; DATA_PAGES], len: 0 };
    let mut p = start;
    loop {
        if !(FIRST_DATA..PAGES).contains(&p) || chain.pages().contains(&(p as u8)) {
            return None;
        }
        chain.pages[chain.len] = p as u8;
        chain.len += 1;
        let next = entry(index, p);
        if next == END_OF_NOTE {
            return Some(chain);
        }
        if next == FREE_PAGE || (next as usize) < FIRST_DATA {
            return None;
        }
        p = next as usize;
    }
}

/// A note as the table lists it: its place, its pages (one where its chain is broken) and whether the chain is broken.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Save {
    pub note: usize,
    pub pages: usize,
    pub damaged: bool,
}

/// Every note in use on a formatted pak whose index is `index_page`, in table order.
pub fn saves(image: &[u8], index_page: usize) -> impl Iterator<Item = Save> + '_ {
    let index = page(image, index_page);
    (0..NOTES).filter(move |&n| in_use(note(image, n))).map(move |n| {
        let chain = chain(index, start_of(note(image, n)));
        Save { note: n, pages: chain.map_or(1, |c| c.len), damaged: chain.is_none() }
    })
}

/// The data pages the index marks free.
pub fn free_pages(image: &[u8], index_page: usize) -> usize {
    let index = page(image, index_page);
    (FIRST_DATA..PAGES).filter(|&p| entry(index, p) == FREE_PAGE).count()
}

/// The note-name characters N64brew lists: space, digits, letters, the punctuation, then the kana; a code it lists
/// none for reads as a middle dot.
pub fn char_of(code: u8) -> char {
    const PUNCTUATION: &str = "!\"#'*+,-./:=?@";
    const KANA: &str = "。゛゜ァィゥェォッャュョヲンアイウエオカキクケコサシスセソタチツテトナニヌネノハヒフヘホマミムメモヤユヨラリルレロワガギグゲゴザジズゼゾダヂヅデドバビブベボパピプペポ";
    match code {
        0x01 => ' ',
        0x10..=0x19 => (b'0' + code - 0x10) as char,
        0x1A..=0x33 => (b'A' + code - 0x1A) as char,
        0x34..=0x41 => PUNCTUATION.chars().nth((code - 0x34) as usize).unwrap_or('·'),
        0x42..=0x94 => KANA.chars().nth((code - 0x42) as usize).unwrap_or('·'),
        _ => '·',
    }
}

/// Why a copy was refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CopyError {
    FromNotFormatted,
    ToNotFormatted,
    BrokenChain,
    NotesFull,
    NoRoom { needed: usize, free: usize },
}

/// Both copies of the index made `index`, its checksum written.
fn write_index(image: &mut [u8], index: &mut [u8; PAGE_LENGTH]) {
    index[0] = 0;
    index[1] = index_sum(index);
    image[INDEX_PAGE * PAGE_LENGTH..(INDEX_PAGE + 1) * PAGE_LENGTH].copy_from_slice(index);
    image[BACKUP_INDEX_PAGE * PAGE_LENGTH..(BACKUP_INDEX_PAGE + 1) * PAGE_LENGTH].copy_from_slice(index);
}

/// Note `n` of `from` copied onto `to`: its pages onto the first free pages, its entry into the first unused note;
/// `to` is unchanged on a refusal.
pub fn copy(from: &[u8], n: usize, to: &mut [u8]) -> Result<(), CopyError> {
    let Some(from_index) = index_page(from).filter(|_| is_formatted(from)) else { return Err(CopyError::FromNotFormatted) };
    let Some(to_index) = index_page(to).filter(|_| is_formatted(to)) else { return Err(CopyError::ToNotFormatted) };
    let source = note(from, n);
    let chain = chain(page(from, from_index), start_of(source)).ok_or(CopyError::BrokenChain)?;
    let slot = (0..NOTES).find(|&k| !in_use(note(to, k))).ok_or(CopyError::NotesFull)?;
    let mut index = [0u8; PAGE_LENGTH];
    index.copy_from_slice(page(to, to_index));
    let mut free = [0usize; DATA_PAGES];
    let mut count = 0;
    for p in FIRST_DATA..PAGES {
        if entry(&index, p) == FREE_PAGE {
            free[count] = p;
            count += 1;
        }
    }
    if count < chain.len {
        return Err(CopyError::NoRoom { needed: chain.len, free: count });
    }
    for (k, &p) in chain.pages().iter().enumerate() {
        let (src, dst) = (p as usize * PAGE_LENGTH, free[k] * PAGE_LENGTH);
        to[dst..dst + PAGE_LENGTH].copy_from_slice(&from[src..src + PAGE_LENGTH]);
        let next = if k == chain.len - 1 { END_OF_NOTE } else { free[k + 1] as u16 };
        index[2 * free[k]..2 * free[k] + 2].copy_from_slice(&next.to_be_bytes());
    }
    write_index(to, &mut index);
    let at = NOTE_TABLE + slot * NOTE_LENGTH;
    to[at..at + NOTE_LENGTH].copy_from_slice(source);
    to[at + 6..at + 8].copy_from_slice(&(free[0] as u16).to_be_bytes());
    Ok(())
}

/// Why a delete was refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeleteError {
    NotFormatted,
}

/// As the console's manager deletes: the note's pages freed in the index, a broken chain's first page alone, and its
/// entry cleared.
pub fn delete(image: &mut [u8], n: usize) -> Result<(), DeleteError> {
    let Some(index_at) = index_page(image).filter(|_| is_formatted(image)) else { return Err(DeleteError::NotFormatted) };
    let mut index = [0u8; PAGE_LENGTH];
    index.copy_from_slice(page(image, index_at));
    let start = start_of(note(image, n));
    let mut only = Chain { pages: [0; DATA_PAGES], len: 1 };
    only.pages[0] = start as u8;
    for &p in chain(&index, start).unwrap_or(only).pages() {
        index[2 * p as usize..2 * p as usize + 2].copy_from_slice(&FREE_PAGE.to_be_bytes());
    }
    write_index(image, &mut index);
    image[NOTE_TABLE + n * NOTE_LENGTH..NOTE_TABLE + (n + 1) * NOTE_LENGTH].fill(0);
    Ok(())
}

/// A blank pak as N64brew lays one out: the ID block with a zero serial, a controller pak of one bank, at its four
/// places; every data page free; no notes. `image` must be `PAK_LENGTH` long.
pub fn format(image: &mut [u8]) {
    assert_eq!(image.len(), PAK_LENGTH, "a pak is {PAK_LENGTH} bytes");
    image.fill(0);
    let mut id = [0u8; 32];
    id[0x19] = 0x01;
    id[0x1A] = 0x01;
    let (sum, inverse) = id_checksums(&id);
    id[0x1C..0x1E].copy_from_slice(&sum.to_be_bytes());
    id[0x1E..0x20].copy_from_slice(&inverse.to_be_bytes());
    for at in ID_BLOCKS {
        image[at..at + 32].copy_from_slice(&id);
    }
    let mut index = [0u8; PAGE_LENGTH];
    for p in FIRST_DATA..PAGES {
        index[2 * p..2 * p + 2].copy_from_slice(&FREE_PAGE.to_be_bytes());
    }
    write_index(image, &mut index);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blank() -> [u8; PAK_LENGTH] {
        let mut pak = [0u8; PAK_LENGTH];
        format(&mut pak);
        pak
    }

    /// A note written as a game writes one: its entry, its pages chained and filled.
    fn with_note(pak: &mut [u8], slot: usize, game: &[u8; 4], pages: &[usize], fill: u8) {
        let at = index_page(pak).unwrap();
        let mut index = [0u8; PAGE_LENGTH];
        index.copy_from_slice(page(pak, at));
        for (k, &p) in pages.iter().enumerate() {
            let next = if k == pages.len() - 1 { END_OF_NOTE } else { pages[k + 1] as u16 };
            index[2 * p..2 * p + 2].copy_from_slice(&next.to_be_bytes());
            pak[p * PAGE_LENGTH..(p + 1) * PAGE_LENGTH].fill(fill.wrapping_add(k as u8));
        }
        write_index(pak, &mut index);
        let e = NOTE_TABLE + slot * NOTE_LENGTH;
        pak[e..e + 4].copy_from_slice(game);
        pak[e + 4..e + 6].copy_from_slice(b"01");
        pak[e + 6..e + 8].copy_from_slice(&(pages[0] as u16).to_be_bytes());
        pak[e + 0x10..e + 0x13].copy_from_slice(&[0x26, 0x1A, 0x2B]);
    }

    #[test]
    fn a_formatted_pak_has_every_data_page_free() {
        let pak = blank();
        assert!(is_formatted(&pak));
        assert_eq!(index_page(&pak), Some(1));
        assert_eq!(free_pages(&pak, 1), 123);
        assert_eq!(saves(&pak, 1).count(), 0);
    }

    #[test]
    fn the_backup_index_is_trusted_when_the_first_is_damaged() {
        let mut pak = blank();
        pak[PAGE_LENGTH + 1] ^= 0xFF;
        assert_eq!(index_page(&pak), Some(2));
        pak[2 * PAGE_LENGTH + 1] ^= 0xFF;
        assert_eq!(index_page(&pak), None);
    }

    #[test]
    fn a_note_reads_its_pages_and_a_broken_one_is_damaged() {
        let mut pak = blank();
        with_note(&mut pak, 0, b"NKTE", &[5, 6, 9], 1);
        assert_eq!(saves(&pak, 1).next(), Some(Save { note: 0, pages: 3, damaged: false }));
        assert_eq!(free_pages(&pak, 1), 120);
        let mut index = [0u8; PAGE_LENGTH];
        index.copy_from_slice(page(&pak, 1));
        index[12..14].copy_from_slice(&FREE_PAGE.to_be_bytes());
        write_index(&mut pak, &mut index);
        assert_eq!(saves(&pak, 1).next(), Some(Save { note: 0, pages: 1, damaged: true }));
    }

    #[test]
    fn a_copy_takes_the_first_free_pages_and_note() {
        let mut from = blank();
        with_note(&mut from, 3, b"NKTE", &[20, 7], 0x30);
        let mut to = blank();
        with_note(&mut to, 0, b"CZLE", &[5], 0x50);
        assert_eq!(copy(&from, 3, &mut to), Ok(()));
        assert_eq!(start_of(note(&to, 1)), 6);
        assert_eq!(&note(&to, 1)[..4], b"NKTE");
        assert_eq!(chain(page(&to, 1), 6).unwrap().pages(), &[6, 7]);
        assert_eq!(page(&to, 6), page(&from, 20));
        assert_eq!(page(&to, 1), page(&to, 2), "both copies of the index written");
    }

    #[test]
    fn a_full_table_or_pak_refuses_and_changes_nothing() {
        let mut from = blank();
        with_note(&mut from, 0, b"NKTE", &[5, 6], 1);
        let mut to = blank();
        for n in 0..NOTES {
            with_note(&mut to, n, b"AAAA", &[FIRST_DATA + n], 2);
        }
        let before = to;
        assert_eq!(copy(&from, 0, &mut to), Err(CopyError::NotesFull));
        assert_eq!(to, before);
        let mut to = blank();
        let all: [usize; 122] = core::array::from_fn(|k| FIRST_DATA + k);
        with_note(&mut to, 0, b"BBBB", &all, 3);
        assert_eq!(copy(&from, 0, &mut to), Err(CopyError::NoRoom { needed: 2, free: 1 }));
    }

    #[test]
    fn a_delete_frees_the_pages_and_clears_the_entry() {
        let mut pak = blank();
        with_note(&mut pak, 2, b"NKTE", &[8, 9], 1);
        assert_eq!(delete(&mut pak, 2), Ok(()));
        assert_eq!(free_pages(&pak, 1), 123);
        assert!(note(&pak, 2).iter().all(|&b| b == 0));
        assert_eq!(delete(&mut [0u8; PAK_LENGTH], 0), Err(DeleteError::NotFormatted));
    }

    #[test]
    fn the_character_set_reads_as_n64brew_lists_it() {
        assert_eq!([char_of(0x01), char_of(0x10), char_of(0x1A), char_of(0x34), char_of(0x41), char_of(0x42), char_of(0x94), char_of(0x95), char_of(0)], [' ', '0', 'A', '!', '@', '。', 'ポ', '·', '·']);
    }
}
