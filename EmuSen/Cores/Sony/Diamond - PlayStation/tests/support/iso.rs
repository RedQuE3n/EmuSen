//! A small ISO 9660 volume built for the open kernel's tests (ECMA-119 2nd edition, §8.4 and §9.1): a primary volume
//! descriptor and its terminator, an L path table, the root and one subdirectory, each file in sectors of its own, all
//! as mode 2 form 1 sectors from 00:02:00. No licence text: the disc is put in the drive directly.

/// A file or a directory of the volume: its path from the root, its bytes (none for a directory).
pub enum Entry<'a> {
    File(&'a str, &'a [u8]),
    Dir(&'a str),
}

const SECTOR: usize = 2048;

fn both16(v: u16) -> [u8; 4] {
    let (l, b) = (v.to_le_bytes(), v.to_be_bytes());
    [l[0], l[1], b[0], b[1]]
}

fn both32(v: u32) -> [u8; 8] {
    let (l, b) = (v.to_le_bytes(), v.to_be_bytes());
    [l[0], l[1], l[2], l[3], b[0], b[1], b[2], b[3]]
}

/// A directory record: its extent, its length, a directory or not, and its identifier.
fn record(lba: u32, size: u32, dir: bool, id: &[u8]) -> Vec<u8> {
    let len = 33 + id.len() + (id.len() + 1) % 2;
    let mut r = vec![0u8; len];
    r[0] = len as u8;
    r[2..10].copy_from_slice(&both32(lba));
    r[10..18].copy_from_slice(&both32(size));
    r[25] = if dir { 2 } else { 0 };
    r[28..32].copy_from_slice(&both16(1));
    r[32] = id.len() as u8;
    r[33..33 + id.len()].copy_from_slice(id);
    r
}

/// The volume's user sectors: system area, descriptors at 16 and 17, the path table at 18, then the root, the
/// directories and the files in the order given.
pub fn volume(entries: &[Entry]) -> Vec<[u8; SECTOR]> {
    let dirs: Vec<&str> = entries.iter().filter_map(|e| if let Entry::Dir(n) = e { Some(*n) } else { None }).collect();
    let root = 19u32;
    let mut next = root + 1 + dirs.len() as u32;
    let mut placed: Vec<(String, u32, u32, bool)> = Vec::new();
    for (i, d) in dirs.iter().enumerate() {
        placed.push((d.to_string(), root + 1 + i as u32, SECTOR as u32, true));
    }
    let mut data: Vec<(u32, &[u8])> = Vec::new();
    for e in entries {
        if let Entry::File(n, bytes) = e {
            placed.push((format!("{n};1"), next, bytes.len() as u32, false));
            data.push((next, bytes));
            next += (bytes.len().div_ceil(SECTOR)).max(1) as u32;
        }
    }
    let mut sectors = vec![[0u8; SECTOR]; next as usize + 2];
    let directory = |parent: &str, lba: u32, up: u32| {
        let mut s = [0u8; SECTOR];
        let mut at = 0;
        let mut put = |r: Vec<u8>| {
            s[at..at + r.len()].copy_from_slice(&r);
            at += r.len();
        };
        put(record(lba, SECTOR as u32, true, &[0]));
        put(record(up, SECTOR as u32, true, &[1]));
        let mut children: Vec<&(String, u32, u32, bool)> = placed
            .iter()
            .filter(|p| p.0.rsplit_once('\\').map_or("", |x| x.0) == parent)
            .collect();
        children.sort_by(|a, b| a.0.cmp(&b.0));
        for c in children {
            let name = c.0.rsplit_once('\\').map_or(c.0.as_str(), |x| x.1);
            put(record(c.1, c.2, c.3, name.as_bytes()));
        }
        s
    };
    sectors[root as usize] = directory("", root, root);
    for (i, d) in dirs.iter().enumerate() {
        sectors[root as usize + 1 + i] = directory(d, root + 1 + i as u32, root);
    }
    for (lba, bytes) in data {
        for (k, chunk) in bytes.chunks(SECTOR).enumerate() {
            sectors[lba as usize + k][..chunk.len()].copy_from_slice(chunk);
        }
    }
    let mut table = vec![1u8, 0];
    table.extend_from_slice(&root.to_le_bytes());
    table.extend_from_slice(&1u16.to_le_bytes());
    table.extend_from_slice(&[0, 0]);
    sectors[18][..table.len()].copy_from_slice(&table);
    let p = &mut sectors[16];
    p[0] = 1;
    p[1..6].copy_from_slice(b"CD001");
    p[6] = 1;
    p[80..88].copy_from_slice(&both32(next + 2));
    p[128..132].copy_from_slice(&both16(SECTOR as u16));
    p[132..140].copy_from_slice(&both32(table.len() as u32));
    p[140..144].copy_from_slice(&18u32.to_le_bytes());
    p[156..190].copy_from_slice(&record(root, SECTOR as u32, true, &[0]));
    let t = &mut sectors[17];
    t[0] = 255;
    t[1..6].copy_from_slice(b"CD001");
    t[6] = 1;
    sectors
}

/// The volume as a disc of mode 2 form 1 sectors, as the drive reads a data track.
pub fn disc(entries: &[Entry]) -> emusen_disc::Disc {
    let mut bin = Vec::new();
    for (i, user) in volume(entries).iter().enumerate() {
        let mut s = [0u8; 2352];
        emusen_disc::ecma130::mode2_form1(150 + i as u32, [0, 0, emusen_disc::ecma130::submode::DATA, 0], user, &mut s);
        bin.extend_from_slice(&s);
    }
    emusen_disc::Disc::from_image(bin, emusen_disc::IsoForm::Mode2Form1).unwrap()
}
