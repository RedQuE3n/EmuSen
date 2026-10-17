//! Memory card images built from psx-spx's "Memory Card Data Format" for the shell's tests: a formatted card, and saves
//! with their title frames, icons and data, so that what the shell reads and writes is judged against the layout itself.

pub const CARD: usize = 128 * 1024;
pub const FRAME: usize = 128;
pub const BLOCK: usize = 8192;

fn checksum(card: &mut [u8], frame: usize) {
    let at = frame * FRAME;
    card[at + 127] = card[at..at + 127].iter().fold(0, |x, &b| x ^ b);
}

/// A card as a format leaves it: the header frame, fifteen free directory frames, an empty broken sector list, the
/// unused frames FFh and the write test frame.
pub fn formatted() -> Vec<u8> {
    let mut c = vec![0u8; CARD];
    c[..2].copy_from_slice(b"MC");
    checksum(&mut c, 0);
    for f in 1..16 {
        c[f * FRAME] = 0xA0;
        c[f * FRAME + 8..f * FRAME + 10].copy_from_slice(&[0xFF, 0xFF]);
        checksum(&mut c, f);
    }
    for f in 16..36 {
        c[f * FRAME..f * FRAME + 4].copy_from_slice(&[0xFF; 4]);
        checksum(&mut c, f);
    }
    for f in 36..63 {
        c[f * FRAME..(f + 1) * FRAME].fill(0xFF);
    }
    let header = c[..FRAME].to_vec();
    c[63 * FRAME..64 * FRAME].copy_from_slice(&header);
    c
}

/// The directory's first free blocks, in order.
fn free(c: &[u8], n: usize) -> Vec<usize> {
    (1..16).filter(|&b| (0xA0..=0xA3).contains(&c[b * FRAME])).take(n).collect()
}

/// `card` with a save of `blocks` blocks named `name`: its title frame with `title` (Shift-JIS or ASCII bytes) and an
/// icon of `icons` frames (0 for a title frame whose flag marks it damaged), the rest of its blocks filled with `fill`.
pub fn with_save(mut c: Vec<u8>, name: &str, title: &[u8], icons: u8, blocks: usize, fill: u8) -> Vec<u8> {
    let chain = free(&c, blocks);
    assert_eq!(chain.len(), blocks, "room for {name}");
    for (i, &b) in chain.iter().enumerate() {
        let d = b * FRAME;
        c[d..d + FRAME].fill(0);
        c[d] = if i == 0 { 0x51 } else if i + 1 == blocks { 0x53 } else { 0x52 };
        if i == 0 {
            c[d + 4..d + 8].copy_from_slice(&((blocks * BLOCK) as u32).to_le_bytes());
            c[d + 10..d + 10 + name.len()].copy_from_slice(name.as_bytes());
        }
        let next: u16 = chain.get(i + 1).map_or(0xFFFF, |&n| (n - 1) as u16);
        c[d + 8..d + 10].copy_from_slice(&next.to_le_bytes());
        checksum(&mut c, b);
        c[b * BLOCK..(b + 1) * BLOCK].fill(fill);
    }
    let t = chain[0] * BLOCK;
    c[t..t + FRAME].fill(0);
    c[t..t + 2].copy_from_slice(b"SC");
    c[t + 2] = if icons == 0 { 0x00 } else { 0x10 + icons };
    c[t + 3] = blocks as u8;
    c[t + 4..t + 4 + title.len()].copy_from_slice(title);
    for i in 0..16u16 {
        let colour: u16 = if i == 0 { 0 } else { 0x8000 | i * 0x0842 };
        c[t + 0x60 + 2 * i as usize..t + 0x62 + 2 * i as usize].copy_from_slice(&colour.to_le_bytes());
    }
    for f in 0..icons as usize {
        let at = t + FRAME * (f + 1);
        for k in 0..FRAME {
            c[at + k] = ((k + f) % 16) as u8 * 0x11;
        }
    }
    c
}

/// The directory frame of the save named `name`, or None.
pub fn entry(c: &[u8], name: &str) -> Option<usize> {
    (1..16).find(|&b| matches!(c[b * FRAME], 0x51 | 0xA1) && &c[b * FRAME + 10..b * FRAME + 10 + name.len()] == name.as_bytes() && c[b * FRAME + 10 + name.len()] == 0)
}

/// The blocks of the save that starts at directory frame `first`, by its chain.
pub fn chain(c: &[u8], first: usize) -> Vec<usize> {
    let mut out = vec![first];
    let mut b = first;
    loop {
        let next = u16::from_le_bytes([c[b * FRAME + 8], c[b * FRAME + 9]]);
        if next == 0xFFFF || out.len() > 15 {
            return out;
        }
        b = next as usize + 1;
        out.push(b);
    }
}
