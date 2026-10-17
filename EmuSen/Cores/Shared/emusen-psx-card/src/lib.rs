//! The PlayStation memory card's file system as psx-spx's "Memory Card Data Format" lays it out: a directory block of
//! a header frame and fifteen directory frames, each with its allocation state, its next block and an XOR checksum,
//! then fifteen blocks of saves, each save's first block beginning with its title frame.
//!
//! No standard library and no allocation: every function reads an image or edits one in place, so a host's card tool
//! and an emulated console's open firmware can share it (EmuSen_Backend.md D14). What a save is called and how its
//! title reads as text are the host's; this crate knows the bytes. See Hardware/Shared/EmuSen_MediumFormats.md.

#![no_std]

pub const CARD_LENGTH: usize = 128 * 1024;
pub const BLOCK_LENGTH: usize = 8 * 1024;
pub const FRAME_LENGTH: usize = 128;
/// The blocks a save may take: every block but the directory's.
pub const BLOCKS: usize = 15;
/// A directory frame's name field, at 0Ah.
pub const NAME_AT: usize = 0x0A;
pub const NAME_LENGTH: usize = 21;
/// The title frame's Shift-JIS title, at 04h.
pub const TITLE_AT: usize = 0x04;
pub const TITLE_LENGTH: usize = 64;

/// A directory frame's allocation states.
pub const FIRST: u32 = 0x51;
pub const MIDDLE: u32 = 0x52;
pub const LAST: u32 = 0x53;
pub const FREE: u32 = 0xA0;
pub const DELETED_FIRST: u32 = 0xA1;
pub const DELETED_MIDDLE: u32 = 0xA2;
pub const DELETED_LAST: u32 = 0xA3;
/// A last block's next.
pub const NO_NEXT: u16 = 0xFFFF;

fn frame(image: &[u8], frame: usize) -> &[u8] {
    &image[frame * FRAME_LENGTH..(frame + 1) * FRAME_LENGTH]
}

fn le32(bytes: &[u8]) -> u32 {
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

/// A frame's checksum: the XOR of its first 127 bytes, which its last byte holds.
pub fn checksum(frame: &[u8]) -> u8 {
    frame[..0x7F].iter().fold(0, |x, b| x ^ b)
}

/// The allocation state of directory entry `entry`, 0 to 14.
pub fn state(image: &[u8], entry: usize) -> u32 {
    le32(frame(image, entry + 1))
}

/// The block after `entry`'s, or `NO_NEXT`.
pub fn next(image: &[u8], entry: usize) -> u16 {
    let f = frame(image, entry + 1);
    u16::from_le_bytes([f[8], f[9]])
}

/// Whether a state leaves its block free, a deleted save's included.
pub fn is_free(state: u32) -> bool {
    matches!(state, FREE | DELETED_FIRST | DELETED_MIDDLE | DELETED_LAST)
}

/// Whether an image is a formatted card: the card's length, "MC" and a header frame whose checksum holds.
pub fn is_formatted(image: &[u8]) -> bool {
    image.len() == CARD_LENGTH && image[0] == b'M' && image[1] == b'C' && checksum(frame(image, 0)) == image[0x7F]
}

/// A save's blocks, in order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Chain {
    blocks: [u8; BLOCKS],
    len: usize,
}

impl Chain {
    pub fn blocks(&self) -> &[u8] {
        &self.blocks[..self.len]
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

/// The blocks of the save whose first directory entry is `first`; none where the chain leaves the card, loops, meets
/// a block not in the middle or at the end of a save, or does not end at a last block.
pub fn chain(image: &[u8], first: usize) -> Option<Chain> {
    let mut chain = Chain { blocks: [0; BLOCKS], len: 1 };
    chain.blocks[0] = first as u8;
    let mut at = first;
    loop {
        let next = next(image, at);
        if next == NO_NEXT {
            break;
        }
        let n = next as usize;
        if n >= BLOCKS || chain.blocks().contains(&(n as u8)) || !matches!(state(image, n), MIDDLE | LAST) {
            return None;
        }
        chain.blocks[chain.len] = n as u8;
        chain.len += 1;
        at = n;
    }
    if chain.len == 1 {
        return Some(chain);
    }
    if state(image, chain.blocks[chain.len - 1] as usize) != LAST {
        return None;
    }
    if chain.blocks[1..chain.len - 1].iter().any(|&b| state(image, b as usize) != MIDDLE) {
        return None;
    }
    Some(chain)
}

/// A save as the directory lists it: its first entry, its blocks (one where its chain is broken) and whether its
/// chain or its first frame's checksum is broken.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Save {
    pub entry: usize,
    pub blocks: usize,
    pub damaged: bool,
}

/// Every save on a formatted card, in directory order.
pub fn saves(image: &[u8]) -> impl Iterator<Item = Save> + '_ {
    (0..BLOCKS).filter(move |&e| state(image, e) == FIRST).map(move |e| {
        let chain = chain(image, e);
        let f = frame(image, e + 1);
        Save { entry: e, blocks: chain.map_or(1, |c| c.len), damaged: chain.is_none() || checksum(f) != f[0x7F] }
    })
}

/// The blocks a formatted card has free, deleted saves' included.
pub fn free_blocks(image: &[u8]) -> usize {
    (0..BLOCKS).filter(|&e| is_free(state(image, e))).count()
}

/// Entry `entry`'s name: its field's bytes before the first NUL.
pub fn name(image: &[u8], entry: usize) -> &[u8] {
    let field = &frame(image, entry + 1)[NAME_AT..NAME_AT + NAME_LENGTH];
    let end = field.iter().position(|&b| b == 0).unwrap_or(NAME_LENGTH);
    &field[..end]
}

/// The title of the save whose first block is `entry`'s: the title frame's text before its first NUL; none where the
/// frame does not begin "SC" or its text is empty.
pub fn title(image: &[u8], entry: usize) -> Option<&[u8]> {
    let at = (entry + 1) * BLOCK_LENGTH;
    let frame = &image[at..at + FRAME_LENGTH];
    if frame[0] != b'S' || frame[1] != b'C' {
        return None;
    }
    let text = &frame[TITLE_AT..TITLE_AT + TITLE_LENGTH];
    let end = text.iter().position(|&b| b == 0).unwrap_or(TITLE_LENGTH);
    (end > 0).then(|| &text[..end])
}

/// Why a copy was refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CopyError {
    FromNotFormatted,
    ToNotFormatted,
    BrokenChain,
    NoRoom { needed: usize, free: usize },
}

/// The save whose first entry is `first` on `from` copied onto `to`'s first free blocks, its directory frames written
/// afresh and its blocks as they were; `to` is unchanged on a refusal.
pub fn copy(from: &[u8], first: usize, to: &mut [u8]) -> Result<(), CopyError> {
    if !is_formatted(from) {
        return Err(CopyError::FromNotFormatted);
    }
    if !is_formatted(to) {
        return Err(CopyError::ToNotFormatted);
    }
    let chain = chain(from, first).ok_or(CopyError::BrokenChain)?;
    let mut free = [0usize; BLOCKS];
    let mut count = 0;
    for e in 0..BLOCKS {
        if is_free(state(to, e)) {
            free[count] = e;
            count += 1;
        }
    }
    if count < chain.len {
        return Err(CopyError::NoRoom { needed: chain.len, free: count });
    }
    let source = frame(from, first + 1);
    for (k, &block) in chain.blocks().iter().enumerate() {
        let at = (free[k] + 1) * FRAME_LENGTH;
        let entry = &mut to[at..at + FRAME_LENGTH];
        entry.fill(0);
        let state = if k == 0 {
            FIRST
        } else if k == chain.len - 1 {
            LAST
        } else {
            MIDDLE
        };
        entry[..4].copy_from_slice(&state.to_le_bytes());
        if k == 0 {
            entry[4..8].copy_from_slice(&((chain.len * BLOCK_LENGTH) as u32).to_le_bytes());
            entry[NAME_AT..NAME_AT + NAME_LENGTH].copy_from_slice(&source[NAME_AT..NAME_AT + NAME_LENGTH]);
        }
        let next = if k == chain.len - 1 { NO_NEXT } else { free[k + 1] as u16 };
        entry[8..10].copy_from_slice(&next.to_le_bytes());
        entry[0x7F] = checksum(entry);
        let (src, dst) = ((block as usize + 1) * BLOCK_LENGTH, (free[k] + 1) * BLOCK_LENGTH);
        to[dst..dst + BLOCK_LENGTH].copy_from_slice(&from[src..src + BLOCK_LENGTH]);
    }
    Ok(())
}

/// As the console's manager deletes: each of the save's directory frames marked deleted, its blocks kept, so the save
/// can be brought back; a broken chain's first frame alone.
pub fn delete(image: &mut [u8], first: usize) {
    let mut one = Chain { blocks: [0; BLOCKS], len: 1 };
    one.blocks[0] = first as u8;
    let chain = chain(image, first).unwrap_or(one);
    for &e in chain.blocks() {
        let at = (e as usize + 1) * FRAME_LENGTH;
        let entry = &mut image[at..at + FRAME_LENGTH];
        let deleted = match le32(entry) {
            FIRST => DELETED_FIRST,
            MIDDLE => DELETED_MIDDLE,
            _ => DELETED_LAST,
        };
        entry[..4].copy_from_slice(&deleted.to_le_bytes());
        entry[0x7F] = checksum(entry);
    }
}

/// A formatted card, as psx-spx lists block 0's: the header, fifteen free entries, an empty broken-sector list, FFh
/// to frame 62, the header again at 63; every other byte zero. `image` must be `CARD_LENGTH` long.
pub fn format(image: &mut [u8]) {
    assert_eq!(image.len(), CARD_LENGTH, "a card is {CARD_LENGTH} bytes");
    image.fill(0);
    for f in 0..64 {
        let frame = &mut image[f * FRAME_LENGTH..(f + 1) * FRAME_LENGTH];
        match f {
            0 | 63 => frame[..2].copy_from_slice(b"MC"),
            1..=BLOCKS => {
                frame[..4].copy_from_slice(&FREE.to_le_bytes());
                frame[8..10].copy_from_slice(&NO_NEXT.to_le_bytes());
            }
            16..=35 => frame[..4].fill(0xFF),
            _ => {
                frame.fill(0xFF);
                continue;
            }
        }
        frame[0x7F] = checksum(frame);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blank() -> [u8; CARD_LENGTH] {
        let mut card = [0u8; CARD_LENGTH];
        format(&mut card);
        card
    }

    /// A save of `blocks` written by hand as a game's would be, its title frame and fill.
    fn with_save(card: &mut [u8], name: &[u8], blocks: &[usize], fill: u8) {
        for (k, &b) in blocks.iter().enumerate() {
            let at = (b + 1) * FRAME_LENGTH;
            let e = &mut card[at..at + FRAME_LENGTH];
            e.fill(0);
            let state = if k == 0 { FIRST } else if k == blocks.len() - 1 { LAST } else { MIDDLE };
            e[..4].copy_from_slice(&state.to_le_bytes());
            if k == 0 {
                e[4..8].copy_from_slice(&((blocks.len() * BLOCK_LENGTH) as u32).to_le_bytes());
                e[NAME_AT..NAME_AT + name.len()].copy_from_slice(name);
            }
            let next = if k == blocks.len() - 1 { NO_NEXT } else { blocks[k + 1] as u16 };
            e[8..10].copy_from_slice(&next.to_le_bytes());
            e[0x7F] = checksum(e);
            let d = (b + 1) * BLOCK_LENGTH;
            card[d..d + BLOCK_LENGTH].fill(fill.wrapping_add(k as u8));
            if k == 0 {
                card[d..d + 2].copy_from_slice(b"SC");
                card[d + TITLE_AT..d + TITLE_AT + 5].copy_from_slice(b"Amber");
                card[d + TITLE_AT + 5] = 0;
            }
        }
    }

    #[test]
    fn a_formatted_card_has_fifteen_free_blocks_and_no_saves() {
        let card = blank();
        assert!(is_formatted(&card));
        assert_eq!(free_blocks(&card), 15);
        assert_eq!(saves(&card).count(), 0);
        assert_eq!(&card[63 * FRAME_LENGTH..63 * FRAME_LENGTH + 2], b"MC");
        assert!(card[36 * FRAME_LENGTH..63 * FRAME_LENGTH].iter().all(|&b| b == 0xFF));
        assert!(!is_formatted(&card[..CARD_LENGTH - 1]));
    }

    #[test]
    fn a_save_reads_with_its_name_title_and_blocks() {
        let mut card = blank();
        with_save(&mut card, b"BASLUS-00001AMBER", &[0, 3, 4], 0x10);
        let all: [Save; 1] = [saves(&card).next().unwrap()];
        assert_eq!(all, [Save { entry: 0, blocks: 3, damaged: false }]);
        assert_eq!(name(&card, 0), b"BASLUS-00001AMBER");
        assert_eq!(title(&card, 0), Some(&b"Amber"[..]));
        assert_eq!(free_blocks(&card), 12);
    }

    #[test]
    fn a_broken_chain_or_checksum_is_damaged() {
        let mut card = blank();
        with_save(&mut card, b"BASLUS-00001A", &[0, 1], 1);
        card[2 * FRAME_LENGTH..2 * FRAME_LENGTH + 4].copy_from_slice(&FREE.to_le_bytes());
        assert_eq!(saves(&card).next(), Some(Save { entry: 0, blocks: 1, damaged: true }));
        let mut card = blank();
        with_save(&mut card, b"BASLUS-00001A", &[0], 1);
        card[FRAME_LENGTH + 0x7F] ^= 1;
        assert!(saves(&card).next().unwrap().damaged);
    }

    #[test]
    fn a_copy_takes_the_first_free_blocks_and_keeps_the_data() {
        let mut from = blank();
        with_save(&mut from, b"BASLUS-00002B", &[5, 9], 0x40);
        let mut to = blank();
        with_save(&mut to, b"BASLUS-00003C", &[0], 0x70);
        assert_eq!(copy(&from, 5, &mut to), Ok(()));
        let listed: [Save; 2] = [saves(&to).next().unwrap(), saves(&to).nth(1).unwrap()];
        assert_eq!(listed[1], Save { entry: 1, blocks: 2, damaged: false });
        assert_eq!(name(&to, 1), b"BASLUS-00002B");
        assert_eq!(&to[2 * BLOCK_LENGTH..3 * BLOCK_LENGTH], &from[6 * BLOCK_LENGTH..7 * BLOCK_LENGTH]);
        assert_eq!(&to[3 * BLOCK_LENGTH..4 * BLOCK_LENGTH], &from[10 * BLOCK_LENGTH..11 * BLOCK_LENGTH]);
    }

    #[test]
    fn a_copy_refused_leaves_the_card_as_it_was() {
        let mut from = blank();
        with_save(&mut from, b"BASLUS-00002B", &[0, 1, 2], 0x40);
        let mut to = blank();
        for e in 0..13 {
            with_save(&mut to, b"X", &[e], e as u8);
        }
        let before = to;
        assert_eq!(copy(&from, 0, &mut to), Err(CopyError::NoRoom { needed: 3, free: 2 }));
        assert_eq!(to, before);
        assert_eq!(copy(&[0; 4], 0, &mut to), Err(CopyError::FromNotFormatted));
    }

    #[test]
    fn a_delete_marks_every_frame_and_keeps_the_blocks() {
        let mut card = blank();
        with_save(&mut card, b"BASLUS-00002B", &[2, 3, 4], 0x40);
        let data: [u8; 3 * BLOCK_LENGTH] = card[3 * BLOCK_LENGTH..6 * BLOCK_LENGTH].try_into().unwrap();
        delete(&mut card, 2);
        assert_eq!((state(&card, 2), state(&card, 3), state(&card, 4)), (DELETED_FIRST, DELETED_MIDDLE, DELETED_LAST));
        assert_eq!(saves(&card).count(), 0);
        assert_eq!(free_blocks(&card), 15);
        assert_eq!(&card[3 * BLOCK_LENGTH..6 * BLOCK_LENGTH], &data[..]);
        assert!((3..6).all(|f| checksum(frame(&card, f)) == card[f * FRAME_LENGTH + 0x7F]));
    }
}
