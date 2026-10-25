//! The PlayStation 2 memory card: its raw format, pages of 512 bytes each followed by 16 spare bytes holding the error
//! correction (ps2tek "SIO2 PS2 Memcards"), and its file system, a superblock, a file allocation table reached through a
//! double-indirect list and directories of 512-byte entries (ps2tek "SIO2 PS2 Memcard Filesystem"; Ross Ridge,
//! "PlayStation 2 Memory Card File System", public domain). An image is the whole raw card, 8,650,752 bytes on Sony's
//! 8 MiB card, which is what other emulators' `.ps2` card files hold.
//!
//! No standard library and no allocation: every function reads an image or edits one in place, so a host's card tool
//! and an emulated console's open firmware can share it (EmuSen_Backend.md D14). A refusal is a value, never a panic,
//! and leaves the image as it was. See Hardware/Shared/EmuSen_MediumFormats.md §6.

#![no_std]

pub mod ecc;

/// A page's data, its spare bytes, and the two together as the card stores them.
pub const PAGE: usize = 512;
pub const SPARE: usize = 16;
pub const RAW_PAGE: usize = PAGE + SPARE;
/// Sony's 8 MiB card: pages, pages an erase block, pages a cluster (ps2tek "Get Specs"; the superblock's defaults).
pub const PAGES: usize = 16_384;
pub const PAGES_PER_BLOCK: usize = 16;
pub const PAGES_PER_CLUSTER: usize = 2;
pub const CLUSTER: usize = PAGE * PAGES_PER_CLUSTER;
pub const CLUSTERS: usize = PAGES / PAGES_PER_CLUSTER;
pub const CARD_LENGTH: usize = PAGES * RAW_PAGE;
/// A directory entry's length, and the entries a cluster holds.
pub const ENTRY: usize = 512;
pub const ENTRIES_PER_CLUSTER: usize = CLUSTER / ENTRY;
/// A FAT entry or an indirect entry is a word; a cluster holds this many.
pub const WORDS_PER_CLUSTER: usize = CLUSTER / 4;
pub const NAME_LENGTH: usize = 32;

/// The superblock's magic and the version this crate writes (Ridge: 1.2 "indicates full support for bad_block_list").
pub const MAGIC: &[u8; 28] = b"Sony PS2 Memory Card Format ";
pub const VERSION: &[u8; 7] = b"1.2.0.0";
pub const CARD_TYPE: u8 = 2;
/// The card's flags (Ridge "Card Flags"): ECC, bad blocks possible, erased bits zero; Sony's cards carry 52h.
pub const CF_USE_ECC: u8 = 0x01;
pub const CF_BAD_BLOCK: u8 = 0x08;
pub const CF_ERASE_ZEROES: u8 = 0x10;
pub const FLAGS: u8 = 0x52;

/// A FAT entry: the top bit set for a cluster in use, the rest the next cluster; all ones ends a chain.
pub const FAT_USED: u32 = 0x8000_0000;
pub const FAT_END: u32 = 0xFFFF_FFFF;
/// What a free entry holds when this crate frees or formats one [C: Ridge requires only the top bit clear].
pub const FAT_FREE: u32 = 0x7FFF_FFFF;
/// An empty file's first cluster.
pub const NO_CLUSTER: u32 = 0xFFFF_FFFF;

/// A directory entry's mode bits (Ridge "Directory Entry Mode Flags").
pub mod mode {
    pub const READ: u16 = 0x0001;
    pub const WRITE: u16 = 0x0002;
    pub const EXECUTE: u16 = 0x0004;
    pub const PROTECTED: u16 = 0x0008;
    pub const FILE: u16 = 0x0010;
    pub const DIRECTORY: u16 = 0x0020;
    /// "Set when files and directories are created".
    pub const CREATED: u16 = 0x0400;
    pub const PSX: u16 = 0x1000;
    pub const HIDDEN: u16 = 0x2000;
    pub const EXISTS: u16 = 0x8000;
    /// What this crate writes for a new directory and a new file [C].
    pub const NEW_DIRECTORY: u16 = EXISTS | CREATED | DIRECTORY | EXECUTE | WRITE | READ;
    pub const NEW_FILE: u16 = EXISTS | CREATED | FILE | EXECUTE | WRITE | READ;
}

/// Why an image cannot be read as a card, or an edit is refused. `words` gives a host the sentence it may show.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Damage {
    /// The image is not the raw card's length.
    Length,
    /// Page 0 does not begin with the magic: the card is not formatted.
    Unformatted,
    /// A superblock field is outside what the format allows (the geometry, the type, the offsets).
    Superblock,
    /// A page's error correction finds more than one bit wrong.
    Ecc,
    /// The FAT, its indirect list or a chain points outside the card, loops or ends early.
    Chain,
    /// A directory's entries are not as the format lays them out.
    Directory,
}

impl Damage {
    pub fn words(self) -> &'static str {
        match self {
            Damage::Length => "the file is not a PlayStation 2 memory card: a card is 8,650,752 bytes",
            Damage::Unformatted => "the card is not formatted",
            Damage::Superblock => "the card's superblock is damaged",
            Damage::Ecc => "a page of the card is damaged beyond what its error correction repairs",
            Damage::Chain => "the card's allocation table is damaged",
            Damage::Directory => "a directory on the card is damaged",
        }
    }
}

/// Why an edit is refused, beside damage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    Damaged(Damage),
    /// No such file or directory.
    Missing,
    /// The name is empty, too long, or holds a character the format forbids (`?`, `*`, `/`, a control character), or
    /// is `.` or `..`.
    Name,
    /// The name is taken in that directory.
    Exists,
    /// Not enough free clusters.
    Full,
    /// A path's part is a file where a directory is needed, or the reverse.
    Kind,
}

impl From<Damage> for Refusal {
    fn from(d: Damage) -> Refusal {
        Refusal::Damaged(d)
    }
}

impl Refusal {
    pub fn words(self) -> &'static str {
        match self {
            Refusal::Damaged(d) => d.words(),
            Refusal::Missing => "no such file or directory on the card",
            Refusal::Name => "the name is not one the card's file system allows",
            Refusal::Exists => "a file or directory of that name is already there",
            Refusal::Full => "the card has too little free space",
            Refusal::Kind => "a file stands where a directory is needed, or a directory where a file is",
        }
    }
}

fn le16(b: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([b[at], b[at + 1]])
}

fn le32(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

fn put16(b: &mut [u8], at: usize, v: u16) {
    b[at..at + 2].copy_from_slice(&v.to_le_bytes());
}

fn put32(b: &mut [u8], at: usize, v: u32) {
    b[at..at + 4].copy_from_slice(&v.to_le_bytes());
}

/// Page `n`'s data, 512 bytes.
pub fn page(image: &[u8], n: usize) -> &[u8] {
    &image[n * RAW_PAGE..n * RAW_PAGE + PAGE]
}

/// Page `n`'s spare bytes, 16.
pub fn spare(image: &[u8], n: usize) -> &[u8] {
    &image[n * RAW_PAGE + PAGE..(n + 1) * RAW_PAGE]
}

/// Writes page `n`'s data and its error correction in the spare bytes' first twelve, the last four left erased (Ridge
/// "Error Correction Code").
pub fn write_page(image: &mut [u8], n: usize, data: &[u8; PAGE]) {
    let raw = &mut image[n * RAW_PAGE..(n + 1) * RAW_PAGE];
    raw[..PAGE].copy_from_slice(data);
    raw[PAGE..PAGE + 12].copy_from_slice(&ecc::page(data));
    raw[PAGE + 12..].fill(0xFF);
}

/// Whether page `n` is erased: every byte, data and spare, FFh.
pub fn erased(image: &[u8], n: usize) -> bool {
    image[n * RAW_PAGE..(n + 1) * RAW_PAGE].iter().all(|&b| b == 0xFF)
}

/// The superblock's fields this crate reads (Ridge "Superblock").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Superblock {
    pub page_len: u16,
    pub pages_per_cluster: u16,
    pub pages_per_block: u16,
    pub clusters_per_card: u32,
    pub alloc_offset: u32,
    pub alloc_end: u32,
    pub rootdir_cluster: u32,
    pub backup_block1: u32,
    pub backup_block2: u32,
    pub ifc_list: [u32; 32],
    pub bad_block_list: [u32; 32],
    pub card_type: u8,
    pub card_flags: u8,
}

impl Superblock {
    /// The superblock of a formatted image, or why it is not one.
    pub fn read(image: &[u8]) -> Result<Superblock, Damage> {
        if image.len() != CARD_LENGTH {
            return Err(Damage::Length);
        }
        let p = page(image, 0);
        if &p[..28] != MAGIC {
            return Err(Damage::Unformatted);
        }
        let mut sb = Superblock {
            page_len: le16(p, 0x28),
            pages_per_cluster: le16(p, 0x2A),
            pages_per_block: le16(p, 0x2C),
            clusters_per_card: le32(p, 0x30),
            alloc_offset: le32(p, 0x34),
            alloc_end: le32(p, 0x38),
            rootdir_cluster: le32(p, 0x3C),
            backup_block1: le32(p, 0x40),
            backup_block2: le32(p, 0x44),
            ifc_list: [0; 32],
            bad_block_list: [0; 32],
            card_type: p[0x150],
            card_flags: p[0x151],
        };
        for i in 0..32 {
            sb.ifc_list[i] = le32(p, 0x50 + 4 * i);
            sb.bad_block_list[i] = le32(p, 0xD0 + 4 * i);
        }
        let geometry = sb.page_len as usize == PAGE
            && sb.pages_per_cluster as usize == PAGES_PER_CLUSTER
            && sb.pages_per_block as usize == PAGES_PER_BLOCK
            && sb.clusters_per_card as usize == CLUSTERS;
        let offsets = (sb.alloc_offset as usize) < CLUSTERS
            && sb.alloc_offset as u64 + sb.alloc_end as u64 <= CLUSTERS as u64
            && sb.rootdir_cluster == 0
            && sb.ifc_list[0] != 0
            && (sb.ifc_list[0] as usize) < CLUSTERS;
        if !geometry || !offsets || sb.card_type != CARD_TYPE {
            return Err(Damage::Superblock);
        }
        Ok(sb)
    }

    /// Whether a cluster (relative to `alloc_offset`) lies in an erase block the superblock lists as bad.
    fn in_bad_block(&self, cluster: u32) -> bool {
        let block = (self.alloc_offset as usize + cluster as usize) * PAGES_PER_CLUSTER / PAGES_PER_BLOCK;
        self.bad_block_list.iter().any(|&b| b != 0xFFFF_FFFF && b as usize == block)
    }
}

/// A cluster's data (absolute cluster number), its two pages' data side by side, each checked by its error correction
/// unless its code bytes are erased, whatever the superblock's flags say [C: Sony's flags, 52h, have Ridge's ECC bit
/// clear]; a single wrong bit is read as corrected, more refuse.
pub fn read_cluster(image: &[u8], absolute: u32, out: &mut [u8; CLUSTER]) -> Result<(), Damage> {
    if absolute as usize >= CLUSTERS {
        return Err(Damage::Chain);
    }
    for i in 0..PAGES_PER_CLUSTER {
        let n = absolute as usize * PAGES_PER_CLUSTER + i;
        let dst = &mut out[i * PAGE..(i + 1) * PAGE];
        dst.copy_from_slice(page(image, n));
        if !erased(image, n) && spare(image, n)[..12].iter().any(|&b| b != 0xFF) && !ecc::correct_page(dst, spare(image, n)) {
            return Err(Damage::Ecc);
        }
    }
    Ok(())
}

fn write_cluster(image: &mut [u8], absolute: u32, data: &[u8; CLUSTER]) {
    for i in 0..PAGES_PER_CLUSTER {
        let mut p = [0u8; PAGE];
        p.copy_from_slice(&data[i * PAGE..(i + 1) * PAGE]);
        write_page(image, absolute as usize * PAGES_PER_CLUSTER + i, &p);
    }
}

/// The word at `index` of the cluster `absolute`.
fn cluster_word(image: &[u8], absolute: u32, index: usize) -> Result<u32, Damage> {
    let mut c = [0u8; CLUSTER];
    read_cluster(image, absolute, &mut c)?;
    Ok(le32(&c, 4 * index))
}

/// Where FAT entry `n` lives: the absolute cluster of the FAT that holds it and the word within it (Ridge "File
/// Allocation Table": the superblock's list names the indirect clusters, which name the FAT's).
fn fat_place(image: &[u8], sb: &Superblock, n: u32) -> Result<(u32, usize), Damage> {
    let n = n as usize;
    let indirect = n / WORDS_PER_CLUSTER;
    let (outer, inner) = (indirect / WORDS_PER_CLUSTER, indirect % WORDS_PER_CLUSTER);
    let ifc = *sb.ifc_list.get(outer).ok_or(Damage::Chain)?;
    let fat = cluster_word(image, ifc, inner)?;
    if fat as usize >= CLUSTERS {
        return Err(Damage::Chain);
    }
    Ok((fat, n % WORDS_PER_CLUSTER))
}

/// Visits the FAT's entries for the allocatable clusters in order, reading each of the FAT's clusters once; `f` returns
/// false to stop. Returns whether it was stopped.
fn scan_fat(image: &[u8], sb: &Superblock, mut f: impl FnMut(u32, u32) -> bool) -> Result<bool, Damage> {
    let mut data = [0u8; CLUSTER];
    let mut n = 0u32;
    while n < sb.alloc_end {
        let (c, first) = fat_place(image, sb, n)?;
        read_cluster(image, c, &mut data)?;
        for w in first..WORDS_PER_CLUSTER {
            if n >= sb.alloc_end {
                break;
            }
            if !f(n, le32(&data, 4 * w)) {
                return Ok(true);
            }
            n += 1;
        }
    }
    Ok(false)
}

/// FAT entry `n`, for the allocatable cluster `n`.
pub fn fat(image: &[u8], sb: &Superblock, n: u32) -> Result<u32, Damage> {
    if n >= sb.alloc_end {
        return Err(Damage::Chain);
    }
    let (c, i) = fat_place(image, sb, n)?;
    cluster_word(image, c, i)
}

fn set_fat(image: &mut [u8], sb: &Superblock, n: u32, v: u32) -> Result<(), Damage> {
    let (c, i) = fat_place(image, sb, n)?;
    let mut data = [0u8; CLUSTER];
    read_cluster(image, c, &mut data)?;
    put32(&mut data, 4 * i, v);
    write_cluster(image, c, &data);
    Ok(())
}

/// The `k`th cluster (relative) of the chain beginning at `first`, following the FAT; at most every allocatable
/// cluster is visited, so a loop is refused.
fn nth_cluster(image: &[u8], sb: &Superblock, first: u32, k: usize) -> Result<u32, Damage> {
    let mut c = first;
    for _ in 0..k {
        let e = fat(image, sb, c)?;
        if e & FAT_USED == 0 || e == FAT_END {
            return Err(Damage::Chain);
        }
        c = e & !FAT_USED;
    }
    if c >= sb.alloc_end {
        return Err(Damage::Chain);
    }
    Ok(c)
}

/// The clusters of the chain beginning at `first`, counted; a loop, a free entry inside it or a step outside refuse.
pub fn chain_len(image: &[u8], sb: &Superblock, first: u32) -> Result<usize, Damage> {
    if first == NO_CLUSTER {
        return Ok(0);
    }
    let mut c = first;
    for n in 1..=sb.alloc_end as usize {
        if c >= sb.alloc_end {
            return Err(Damage::Chain);
        }
        let e = fat(image, sb, c)?;
        if e & FAT_USED == 0 {
            return Err(Damage::Chain);
        }
        if e == FAT_END {
            return Ok(n);
        }
        c = e & !FAT_USED;
    }
    Err(Damage::Chain)
}

/// A date and time as the card keeps it (Ridge "Time of Day"), Japan's time zone.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Time {
    pub sec: u8,
    pub min: u8,
    pub hour: u8,
    pub day: u8,
    pub month: u8,
    pub year: u16,
}

impl Time {
    fn read(b: &[u8]) -> Time {
        Time { sec: b[1], min: b[2], hour: b[3], day: b[4], month: b[5], year: le16(b, 6) }
    }

    fn write(&self, b: &mut [u8]) {
        b[0] = 0;
        b[1] = self.sec;
        b[2] = self.min;
        b[3] = self.hour;
        b[4] = self.day;
        b[5] = self.month;
        put16(b, 6, self.year);
    }
}

/// A directory entry (Ridge "Directory Entry").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    pub mode: u16,
    /// A file's bytes, or a directory's entries.
    pub length: u32,
    pub created: Time,
    /// The first cluster, relative to `alloc_offset`; `NO_CLUSTER` for an empty file.
    pub cluster: u32,
    pub dir_entry: u32,
    pub modified: Time,
    pub attr: u32,
    name: [u8; NAME_LENGTH],
}

impl Entry {
    fn read(b: &[u8]) -> Entry {
        let mut name = [0u8; NAME_LENGTH];
        name.copy_from_slice(&b[0x40..0x40 + NAME_LENGTH]);
        Entry {
            mode: le16(b, 0),
            length: le32(b, 4),
            created: Time::read(&b[8..16]),
            cluster: le32(b, 0x10),
            dir_entry: le32(b, 0x14),
            modified: Time::read(&b[0x18..0x20]),
            attr: le32(b, 0x20),
            name,
        }
    }

    fn write(&self, b: &mut [u8]) {
        b.fill(0);
        put16(b, 0, self.mode);
        put32(b, 4, self.length);
        self.created.write(&mut b[8..16]);
        put32(b, 0x10, self.cluster);
        put32(b, 0x14, self.dir_entry);
        self.modified.write(&mut b[0x18..0x20]);
        put32(b, 0x20, self.attr);
        b[0x40..0x40 + NAME_LENGTH].copy_from_slice(&self.name);
    }

    fn new(mode: u16, name: &[u8], cluster: u32, length: u32, time: Time) -> Entry {
        let mut n = [0u8; NAME_LENGTH];
        n[..name.len()].copy_from_slice(name);
        Entry { mode, length, created: time, cluster, dir_entry: 0, modified: time, attr: 0, name: n }
    }

    /// The name: its field's bytes before the first NUL.
    pub fn name(&self) -> &[u8] {
        let end = self.name.iter().position(|&b| b == 0).unwrap_or(NAME_LENGTH);
        &self.name[..end]
    }

    pub fn exists(&self) -> bool {
        self.mode & mode::EXISTS != 0
    }

    pub fn is_dir(&self) -> bool {
        self.mode & mode::DIRECTORY != 0
    }

    pub fn is_file(&self) -> bool {
        self.mode & mode::FILE != 0
    }
}

/// A formatted card read through its superblock.
#[derive(Clone, Copy)]
pub struct Card<'a> {
    pub image: &'a [u8],
    pub sb: Superblock,
}

/// A directory: its first cluster (relative) and its entries' count.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Dir {
    pub cluster: u32,
    pub entries: u32,
}

/// The root's first entry, which stands for the root itself.
fn root(image: &[u8], sb: &Superblock) -> Result<Dir, Damage> {
    let e = entry_at(image, sb, Dir { cluster: sb.rootdir_cluster, entries: 1 }, 0)?;
    if !e.exists() || !e.is_dir() {
        return Err(Damage::Directory);
    }
    checked(image, sb, Dir { cluster: sb.rootdir_cluster, entries: e.length })
}

/// A directory whose count of entries its chain of clusters can hold, at least "." and "..".
fn checked(image: &[u8], sb: &Superblock, d: Dir) -> Result<Dir, Damage> {
    if d.entries < 2 || d.entries as usize > sb.alloc_end as usize * ENTRIES_PER_CLUSTER {
        return Err(Damage::Directory);
    }
    if chain_len(image, sb, d.cluster)? < (d.entries as usize).div_ceil(ENTRIES_PER_CLUSTER) {
        return Err(Damage::Directory);
    }
    Ok(d)
}

/// Entry `i` of directory `d`.
fn entry_at(image: &[u8], sb: &Superblock, d: Dir, i: u32) -> Result<Entry, Damage> {
    if i >= d.entries {
        return Err(Damage::Directory);
    }
    let c = nth_cluster(image, sb, d.cluster, i as usize / ENTRIES_PER_CLUSTER)?;
    let mut data = [0u8; CLUSTER];
    read_cluster(image, sb.alloc_offset + c, &mut data)?;
    let at = (i as usize % ENTRIES_PER_CLUSTER) * ENTRY;
    Ok(Entry::read(&data[at..at + ENTRY]))
}

fn set_entry(image: &mut [u8], sb: &Superblock, d: Dir, i: u32, e: &Entry) -> Result<(), Damage> {
    let c = nth_cluster(image, sb, d.cluster, i as usize / ENTRIES_PER_CLUSTER)?;
    let mut data = [0u8; CLUSTER];
    read_cluster(image, sb.alloc_offset + c, &mut data)?;
    let at = (i as usize % ENTRIES_PER_CLUSTER) * ENTRY;
    e.write(&mut data[at..at + ENTRY]);
    write_cluster(image, sb.alloc_offset + c, &data);
    Ok(())
}

impl<'a> Card<'a> {
    /// A formatted card, or why the image is not one; the root directory is read once to prove it.
    pub fn open(image: &'a [u8]) -> Result<Card<'a>, Damage> {
        let sb = Superblock::read(image)?;
        root(image, &sb)?;
        Ok(Card { image, sb })
    }

    pub fn root(&self) -> Result<Dir, Damage> {
        root(self.image, &self.sb)
    }

    /// Entry `i` of `d`; entries 0 and 1 are "." and "..".
    pub fn entry(&self, d: Dir, i: u32) -> Result<Entry, Damage> {
        entry_at(self.image, &self.sb, d, i)
    }

    /// The directory an entry names.
    pub fn dir(&self, e: &Entry) -> Result<Dir, Damage> {
        if !e.is_dir() {
            return Err(Damage::Directory);
        }
        let first = entry_at(self.image, &self.sb, Dir { cluster: e.cluster, entries: 1 }, 0)?;
        if !first.exists() {
            return Err(Damage::Directory);
        }
        checked(self.image, &self.sb, Dir { cluster: e.cluster, entries: e.length })
    }

    /// The entry named `name` in `d` and its index.
    pub fn find(&self, d: Dir, name: &[u8]) -> Result<(u32, Entry), Refusal> {
        for i in 2..d.entries {
            let e = self.entry(d, i)?;
            if e.exists() && e.name() == name {
                return Ok((i, e));
            }
        }
        Err(Refusal::Missing)
    }

    /// Reads a file's bytes from `offset` into `out`; the bytes read.
    pub fn read(&self, e: &Entry, offset: usize, out: &mut [u8]) -> Result<usize, Damage> {
        let len = e.length as usize;
        if offset >= len || out.is_empty() {
            return Ok(0);
        }
        let n = out.len().min(len - offset);
        let mut done = 0;
        let mut data = [0u8; CLUSTER];
        while done < n {
            let at = offset + done;
            let c = nth_cluster(self.image, &self.sb, e.cluster, at / CLUSTER)?;
            read_cluster(self.image, self.sb.alloc_offset + c, &mut data)?;
            let from = at % CLUSTER;
            let k = (CLUSTER - from).min(n - done);
            out[done..done + k].copy_from_slice(&data[from..from + k]);
            done += k;
        }
        Ok(n)
    }

    /// The clusters a file or directory takes, a directory's files and subdirectories with it.
    pub fn clusters_of(&self, e: &Entry) -> Result<usize, Damage> {
        self.clusters_deep(e, 0)
    }

    fn clusters_deep(&self, e: &Entry, depth: usize) -> Result<usize, Damage> {
        if depth > 8 {
            return Err(Damage::Directory);
        }
        if e.is_file() {
            let n = chain_len(self.image, &self.sb, e.cluster)?;
            if n < (e.length as usize).div_ceil(CLUSTER) {
                return Err(Damage::Chain);
            }
            return Ok(n);
        }
        let d = self.dir(e)?;
        let mut n = chain_len(self.image, &self.sb, d.cluster)?;
        if n < (d.entries as usize).div_ceil(ENTRIES_PER_CLUSTER) {
            return Err(Damage::Chain);
        }
        for i in 2..d.entries {
            let c = self.entry(d, i)?;
            if c.exists() {
                n += self.clusters_deep(&c, depth + 1)?;
            }
        }
        Ok(n)
    }

    /// The clusters a player can still use: Sony's drivers round the allocatable clusters down to a thousand (Ridge
    /// "Reserved Clusters"), so a new card, its root directory taking one, has 7,999 of its 8,135 [A: Ridge's rule read
    /// as a limit on the clusters in use; Hardware/Shared/EmuSen_MediumFormats.md §6.2].
    pub fn free_clusters(&self) -> Result<usize, Damage> {
        let mut free = 0usize;
        scan_fat(self.image, &self.sb, |n, e| {
            if e & FAT_USED == 0 && !self.sb.in_bad_block(n) {
                free += 1;
            }
            true
        })?;
        let usable = (self.sb.alloc_end as usize / 1000) * 1000;
        let used = self.sb.alloc_end as usize - free;
        Ok(usable.saturating_sub(used))
    }
}

/// The top level's saves, a directory each, in directory order: the root's entries past "." and "..", existing ones.
/// The first damage found ends the listing.
pub fn saves<'a>(card: &'a Card<'a>) -> impl Iterator<Item = Result<(u32, Entry), Damage>> + 'a {
    let d = card.root();
    let n = match d {
        Ok(d) => d.entries,
        Err(_) => 3,
    };
    let mut failed = false;
    (2..n).map_while(move |i| {
        if failed {
            return None;
        }
        let r = d.and_then(|d| card.entry(d, i));
        failed = r.is_err();
        Some(r.map(|e| (i, e)))
    })
    .filter(|r| !matches!(r, Ok((_, e)) if !e.exists()))
}

/// Whether `name` is one the format allows (Ridge: no "?", "*", "/" and no control character; at most 31 bytes, the
/// field keeping its NUL).
pub fn valid_name(name: &[u8]) -> bool {
    !name.is_empty() && name.len() < NAME_LENGTH && name != b"." && name != b".." && name.iter().all(|&b| b >= 0x20 && b != 0x7F && !matches!(b, b'?' | b'*' | b'/'))
}

/// Formats an image as Sony's 8 MiB card (Ridge "Standard Memory Card Layout"): the superblock in page 0, the indirect
/// FAT cluster 8, the FAT in clusters 9 to 40, the root directory at the first allocatable cluster, 41, with its "."
/// and "..", every other page erased, the last two erase blocks the backups. `time` is the root's creation time.
pub fn format(image: &mut [u8], time: Time) -> Result<(), Damage> {
    if image.len() != CARD_LENGTH {
        return Err(Damage::Length);
    }
    image.fill(0xFF);
    const IFC: u32 = 8;
    const FAT_CLUSTERS: u32 = (CLUSTERS / WORDS_PER_CLUSTER) as u32;
    const ALLOC_OFFSET: u32 = IFC + 1 + FAT_CLUSTERS;
    let backup1 = (PAGES / PAGES_PER_BLOCK - 1) as u32;
    let backup2 = backup1 - 1;
    let alloc_end = (backup2 as usize * PAGES_PER_BLOCK / PAGES_PER_CLUSTER) as u32 - ALLOC_OFFSET;
    let mut p = [0xFFu8; PAGE];
    p[..0x154].fill(0);
    p[..28].copy_from_slice(MAGIC);
    p[0x1C..0x1C + VERSION.len()].copy_from_slice(VERSION);
    put16(&mut p, 0x28, PAGE as u16);
    put16(&mut p, 0x2A, PAGES_PER_CLUSTER as u16);
    put16(&mut p, 0x2C, PAGES_PER_BLOCK as u16);
    put16(&mut p, 0x2E, 0xFF00);
    put32(&mut p, 0x30, CLUSTERS as u32);
    put32(&mut p, 0x34, ALLOC_OFFSET);
    put32(&mut p, 0x38, alloc_end);
    put32(&mut p, 0x3C, 0);
    put32(&mut p, 0x40, backup1);
    put32(&mut p, 0x44, backup2);
    put32(&mut p, 0x50, IFC);
    for i in 0..32 {
        put32(&mut p, 0xD0 + 4 * i, 0xFFFF_FFFF);
    }
    p[0x150] = CARD_TYPE;
    p[0x151] = FLAGS;
    write_page(image, 0, &p);
    let mut c = [0xFFu8; CLUSTER];
    for i in 0..FAT_CLUSTERS {
        put32(&mut c, 4 * i as usize, IFC + 1 + i);
    }
    write_cluster(image, IFC, &c);
    for i in 0..FAT_CLUSTERS {
        let mut f = [0u8; CLUSTER];
        for w in 0..WORDS_PER_CLUSTER {
            let n = i as usize * WORDS_PER_CLUSTER + w;
            put32(&mut f, 4 * w, if n == 0 { FAT_END } else { FAT_FREE });
        }
        write_cluster(image, IFC + 1 + i, &f);
    }
    let mut d = [0u8; CLUSTER];
    Entry::new(mode::NEW_DIRECTORY, b".", 0, 2, time).write(&mut d[..ENTRY]);
    Entry::new(mode::NEW_DIRECTORY | mode::HIDDEN, b"..", 0, 0, time).write(&mut d[ENTRY..]);
    write_cluster(image, ALLOC_OFFSET, &d);
    Ok(())
}

/// Allocates one free cluster, its FAT entry ending a chain; the lowest free one outside a bad block.
fn allocate(image: &mut [u8], sb: &Superblock) -> Result<u32, Refusal> {
    let mut found = None;
    scan_fat(image, sb, |n, e| {
        let free = e & FAT_USED == 0 && !sb.in_bad_block(n);
        if free {
            found = Some(n);
        }
        !free
    })?;
    let n = found.ok_or(Refusal::Full)?;
    set_fat(image, sb, n, FAT_END)?;
    Ok(n)
}

/// Frees a chain's clusters.
fn free_chain(image: &mut [u8], sb: &Superblock, first: u32) -> Result<(), Damage> {
    let n = chain_len(image, sb, first)?;
    let mut c = first;
    for _ in 0..n {
        let e = fat(image, sb, c)?;
        set_fat(image, sb, c, FAT_FREE)?;
        c = e & !FAT_USED;
    }
    Ok(())
}

/// Whether `want` clusters can be allocated: within the player's rounded allowance (`Card::free_clusters`), which never
/// exceeds what the FAT has free.
fn count_free(image: &[u8], sb: &Superblock, want: usize) -> Result<bool, Damage> {
    Ok(Card { image, sb: *sb }.free_clusters()? >= want)
}

/// Follows `path`'s parts ("/"-separated, from the root) to a directory, and the entry that names it (the root's own
/// for the root) with its index and its parent.
fn walk(image: &[u8], sb: &Superblock, path: &[u8]) -> Result<(Dir, Option<(Dir, u32)>), Refusal> {
    let card = Card { image, sb: *sb };
    let mut d = card.root()?;
    let mut place = None;
    for part in path.split(|&b| b == b'/').filter(|p| !p.is_empty()) {
        let (i, e) = card.find(d, part)?;
        if !e.is_dir() {
            return Err(Refusal::Kind);
        }
        place = Some((d, i));
        d = card.dir(&e)?;
    }
    Ok((d, place))
}

/// A free entry's index in `d`, the directory grown by a cluster when it has none, and the directory as it now stands.
fn free_slot(image: &mut [u8], sb: &Superblock, d: Dir, place: Option<(Dir, u32)>) -> Result<(u32, Dir), Refusal> {
    let card = Card { image, sb: *sb };
    for i in 2..d.entries {
        if !card.entry(d, i)?.exists() {
            return Ok((i, d));
        }
    }
    let i = d.entries;
    if (i as usize).is_multiple_of(ENTRIES_PER_CLUSTER) {
        let last = nth_cluster(image, sb, d.cluster, i as usize / ENTRIES_PER_CLUSTER - 1)?;
        let c = allocate(image, sb)?;
        set_fat(image, sb, last, FAT_USED | c)?;
        write_cluster(image, sb.alloc_offset + c, &[0u8; CLUSTER]);
    }
    let grown = Dir { cluster: d.cluster, entries: i + 1 };
    // The directory's length is kept in its own entry: the root's first entry, or the parent's entry naming it.
    let (holder, at) = place.unwrap_or((Dir { cluster: d.cluster, entries: 1 }, 0));
    let mut e = entry_at(image, sb, holder, at)?;
    e.length = grown.entries;
    set_entry(image, sb, holder, at, &e)?;
    Ok((i, grown))
}

/// Makes directory `name` in the directory at `parent` (a path from the root), its "." naming the parent's first
/// cluster and its entry there, as Ridge lays it out.
pub fn mkdir(image: &mut [u8], parent: &[u8], name: &[u8], time: Time) -> Result<(), Refusal> {
    let sb = Superblock::read(image)?;
    if !valid_name(name) {
        return Err(Refusal::Name);
    }
    let (d, place) = walk(image, &sb, parent)?;
    if (Card { image, sb }).find(d, name).is_ok() {
        return Err(Refusal::Exists);
    }
    if !count_free(image, &sb, 1 + usize::from((d.entries as usize).is_multiple_of(ENTRIES_PER_CLUSTER)))? {
        return Err(Refusal::Full);
    }
    let (i, d) = free_slot(image, &sb, d, place)?;
    let c = allocate(image, &sb)?;
    let mut data = [0u8; CLUSTER];
    let mut dot = Entry::new(mode::NEW_DIRECTORY, b".", d.cluster, 0, time);
    dot.dir_entry = i;
    dot.write(&mut data[..ENTRY]);
    Entry::new(mode::NEW_DIRECTORY | mode::HIDDEN, b"..", 0, 0, time).write(&mut data[ENTRY..]);
    write_cluster(image, sb.alloc_offset + c, &data);
    set_entry(image, &sb, d, i, &Entry::new(mode::NEW_DIRECTORY, name, c, 2, time))?;
    Ok(())
}

/// Writes file `name` in the directory at `dir` with `data`, refused if the name is taken; all or nothing.
pub fn create(image: &mut [u8], dir: &[u8], name: &[u8], data: &[u8], time: Time) -> Result<(), Refusal> {
    let sb = Superblock::read(image)?;
    if !valid_name(name) {
        return Err(Refusal::Name);
    }
    let (d, place) = walk(image, &sb, dir)?;
    if (Card { image, sb }).find(d, name).is_ok() {
        return Err(Refusal::Exists);
    }
    let need = data.len().div_ceil(CLUSTER);
    let grow = usize::from((d.entries as usize).is_multiple_of(ENTRIES_PER_CLUSTER));
    if !count_free(image, &sb, need + grow)? {
        return Err(Refusal::Full);
    }
    let (i, d) = free_slot(image, &sb, d, place)?;
    let mut first = NO_CLUSTER;
    let mut prev = None;
    for chunk in data.chunks(CLUSTER) {
        let c = allocate(image, &sb)?;
        let mut buf = [0u8; CLUSTER];
        buf[..chunk.len()].copy_from_slice(chunk);
        write_cluster(image, sb.alloc_offset + c, &buf);
        match prev {
            None => first = c,
            Some(p) => set_fat(image, &sb, p, FAT_USED | c)?,
        }
        prev = Some(c);
    }
    set_entry(image, &sb, d, i, &Entry::new(mode::NEW_FILE, name, first, data.len() as u32, time))?;
    Ok(())
}

/// Deletes the file or directory at `path`, a directory with everything in it: its entry's EXISTS bit cleared and its
/// clusters freed.
pub fn delete(image: &mut [u8], path: &[u8]) -> Result<(), Refusal> {
    let sb = Superblock::read(image)?;
    let split = path.iter().rposition(|&b| b == b'/');
    let (parent, name) = match split {
        Some(k) => (&path[..k], &path[k + 1..]),
        None => (&path[..0], path),
    };
    let (d, _) = walk(image, &sb, parent)?;
    let (i, e) = Card { image, sb }.find(d, name)?;
    remove(image, &sb, d, i, e, 0)
}

fn remove(image: &mut [u8], sb: &Superblock, d: Dir, i: u32, e: Entry, depth: usize) -> Result<(), Refusal> {
    if depth > 8 {
        return Err(Refusal::Damaged(Damage::Directory));
    }
    if e.is_dir() {
        let sub = Card { image, sb: *sb }.dir(&e)?;
        for k in 2..sub.entries {
            let c = entry_at(image, sb, sub, k)?;
            if c.exists() {
                remove(image, sb, sub, k, c, depth + 1)?;
            }
        }
        free_chain(image, sb, e.cluster)?;
    } else if e.cluster != NO_CLUSTER {
        free_chain(image, sb, e.cluster)?;
    }
    let mut gone = e;
    gone.mode &= !mode::EXISTS;
    set_entry(image, sb, d, i, &gone)?;
    Ok(())
}

/// Copies the top-level save `name` (a directory of files) from card `from` to card `to`, refused if `to` has one of
/// that name or too little room; `to` is left as it was on a refusal. The files keep their times and modes.
pub fn copy_save(from: &[u8], name: &[u8], to: &mut [u8]) -> Result<(), Refusal> {
    let src = Card::open(from)?;
    let (_, e) = src.find(src.root()?, name)?;
    if !e.is_dir() {
        return Err(Refusal::Kind);
    }
    let dst = Card::open(to)?;
    if dst.find(dst.root()?, name).is_ok() {
        return Err(Refusal::Exists);
    }
    let need = src.clusters_of(&e)? + 1;
    // Every cluster of the save is read once first, so that a damaged one refuses before `to` is touched.
    let d = src.dir(&e)?;
    let mut probe = [0u8; CLUSTER];
    for i in 2..d.entries {
        let f = src.entry(d, i)?;
        if f.exists() && f.is_file() {
            let mut at = 0;
            while at < f.length as usize {
                src.read(&f, at, &mut probe)?;
                at += CLUSTER;
            }
        }
    }
    if !count_free(to, &dst.sb, need + 1)? {
        return Err(Refusal::Full);
    }
    mkdir(to, b"", name, e.created)?;
    let mut path = [0u8; NAME_LENGTH];
    path[..name.len()].copy_from_slice(name);
    let path = &path[..name.len()];
    for i in 2..d.entries {
        let f = src.entry(d, i)?;
        if !f.exists() || !f.is_file() {
            continue;
        }
        copy_file(&src, &f, to, path)?;
    }
    let sb = Superblock::read(to)?;
    let (root, _) = walk(to, &sb, b"")?;
    let (k, mut made) = Card { image: to, sb }.find(root, name)?;
    made.mode = e.mode;
    made.modified = e.modified;
    made.attr = e.attr;
    set_entry(to, &sb, root, k, &made)?;
    Ok(())
}

/// A file copied cluster by cluster, so that no buffer of its size is needed.
fn copy_file(src: &Card<'_>, f: &Entry, to: &mut [u8], dir: &[u8]) -> Result<(), Refusal> {
    create(to, dir, f.name(), &[], f.created)?;
    let sb = Superblock::read(to)?;
    let (d, _) = walk(to, &sb, dir)?;
    let (i, mut made) = Card { image: to, sb }.find(d, f.name())?;
    let mut prev: Option<u32> = None;
    let mut buf = [0u8; CLUSTER];
    let mut at = 0;
    while at < f.length as usize {
        buf.fill(0);
        src.read(f, at, &mut buf)?;
        let c = allocate(to, &sb)?;
        write_cluster(to, sb.alloc_offset + c, &buf);
        match prev {
            None => made.cluster = c,
            Some(p) => set_fat(to, &sb, p, FAT_USED | c)?,
        }
        prev = Some(c);
        at += CLUSTER;
    }
    made.length = f.length;
    made.mode = f.mode;
    made.modified = f.modified;
    made.attr = f.attr;
    set_entry(to, &sb, d, i, &made)?;
    Ok(())
}

#[cfg(test)]
mod tests;
