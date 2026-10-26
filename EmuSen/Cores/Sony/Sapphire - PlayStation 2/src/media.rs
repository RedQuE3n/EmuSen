//! What an image is, read from its bytes (Sapphire_Native.md §2.2): an EE program as an ELF (the Tool Interface
//! Standard's ELF 1.2, 32-bit, little-endian, MIPS), or a disc whose ISO 9660 root holds SYSTEM.CNF with a BOOT2 line
//! (Sapphire_Plan.md §4.1). Recognition only; nothing is loaded.

/// The ELF's magic, `7Fh` and "ELF".
pub const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];
/// The ELF header's and a program header's sizes in the 32-bit class.
pub const ELF_HEADER: usize = 52;
pub const PROGRAM_HEADER: usize = 32;
/// `e_machine` for MIPS, `e_type` for an executable, and the type Sony's IOP modules carry.
pub const EM_MIPS: u16 = 8;
pub const ET_EXEC: u16 = 2;
pub const ET_IRX: u16 = 0xFF80;
/// `p_type` of a segment loaded into memory.
pub const PT_LOAD: u32 = 1;
/// `p_type` of the MIPS register information, whose sixth word is the value of `gp` the program expects (the System V
/// ABI's MIPS supplement, "Register Information").
pub const PT_MIPS_REGINFO: u32 = 0x7000_0000;

pub const USER: usize = 2048;
pub const RAW: usize = 2352;
const SYNC: [u8; 12] = [0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00];
const LICENCE: &[u8; 32] = b"          Licensed  by          ";

/// An EE program: where it starts, and its loaded segments as (address, file offset, bytes in the file, bytes in memory).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Elf {
    pub entry: u32,
    pub segments: Vec<(u32, u32, u32, u32)>,
    /// `gp` from the register information, zero without one.
    pub gp: u32,
}

/// A PlayStation 2 disc: whether its sectors are raw, how many there are, the program BOOT2 names, its serial, and
/// whether SYSTEM.CNF's VMODE line says PAL.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Disc {
    pub raw: bool,
    pub sectors: u64,
    pub program: String,
    pub serial: Option<String>,
    pub pal: bool,
}

/// The market a disc was made for, by its serial's third letter, Sony's region letter (SLUS, SCES, SLPM), else by VMODE
/// where it says PAL; none for a serial of no known letter on an NTSC disc (Sapphire_Native.md §49.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiscMarket {
    Japan,
    Americas,
    Europe,
}

impl Disc {
    pub fn market(&self) -> Option<DiscMarket> {
        market_of(self.serial.as_deref(), self.pal)
    }
}

/// U is the Americas, E Europe, and P, A, K and C Japan and the markets whose discs its consoles boot (Asia, Korea, China),
/// as the BIOS entries group them (Sapphire_Native.md §12.3); a PAL disc of another letter is Europe's.
pub fn market_of(serial: Option<&str>, pal: bool) -> Option<DiscMarket> {
    match serial.and_then(|s| s.as_bytes().get(2)).map(u8::to_ascii_uppercase) {
        Some(b'U') => Some(DiscMarket::Americas),
        Some(b'E') => Some(DiscMarket::Europe),
        Some(b'P' | b'A' | b'K' | b'C') => Some(DiscMarket::Japan),
        _ => pal.then_some(DiscMarket::Europe),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Media {
    Elf(Elf),
    Disc(Disc),
    /// No image: the console started with nothing in it, its tray closed and empty (Sapphire_Plan.md §8, G10).
    Empty,
}

/// Why an image is refused, each with the words `v1` gives it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// Neither an ELF nor an ISO 9660 volume.
    Unknown,
    /// An ELF, but not an EE program: what it is instead.
    NotEe(String),
    /// An ELF whose headers or segments run past its end.
    Truncated { needs: u64, has: u64 },
    /// A PlayStation disc, which Diamond runs (Sapphire_Plan.md §4.1).
    PlayStation,
    /// An ISO 9660 volume whose root names no BOOT2 program: what was found.
    NoBoot2(&'static str),
}

fn u16_at(b: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([b[at], b[at + 1]])
}

fn u32_at(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

impl Media {
    pub fn read(image: &[u8]) -> Result<Media, Refusal> {
        if image.starts_with(&ELF_MAGIC) {
            return read_elf(image).map(Media::Elf);
        }
        let sectors = Sectors::new(image);
        Media::read_by(&mut |n| sectors.sector(n).to_vec(), sectors.count(), sectors.raw)
    }

    /// A disc already opened, its 2,048-byte sectors given by `sector`, as a CHD's are (Sapphire_Native.md §49.2).
    pub fn read_by(sector: &mut dyn FnMut(u64) -> Vec<u8>, count: u64, raw: bool) -> Result<Media, Refusal> {
        if volume_by(sector).is_none() {
            return Err(Refusal::Unknown);
        }
        read_disc(sector, count, raw).map(Media::Disc)
    }

    pub fn serial(&self) -> Option<&str> {
        match self {
            Media::Disc(d) => d.serial.as_deref(),
            Media::Elf(_) | Media::Empty => None,
        }
    }
}

fn read_elf(image: &[u8]) -> Result<Elf, Refusal> {
    let has = image.len() as u64;
    if image.len() < ELF_HEADER {
        return Err(Refusal::Truncated { needs: ELF_HEADER as u64, has });
    }
    let not = |what: String| Err(Refusal::NotEe(what));
    match (image[4], image[5]) {
        (1, 1) => {}
        (2, _) => return not("a 64-bit ELF".into()),
        (1, 2) => return not("a big-endian ELF".into()),
        (class, data) => return not(format!("an ELF of class {class} and byte order {data}")),
    }
    let (kind, machine) = (u16_at(image, 16), u16_at(image, 18));
    if machine != EM_MIPS {
        return not(format!("an ELF for machine {machine}, not MIPS"));
    }
    if kind == ET_IRX {
        return not("an IOP module (an .irx), which the IOP's kernel loads, not a program of the EE".into());
    }
    if kind != ET_EXEC {
        return not(format!("an ELF of type {kind:#06x}, not an executable"));
    }
    let (entry, phoff, phentsize, phnum) = (u32_at(image, 24), u32_at(image, 28) as u64, u16_at(image, 42) as u64, u16_at(image, 44) as u64);
    if phnum == 0 || phentsize != PROGRAM_HEADER as u64 {
        return not(format!("an executable with {phnum} program headers of {phentsize} bytes, where a program has one or more of 32"));
    }
    let table = phoff + phnum * phentsize;
    if table > has {
        return Err(Refusal::Truncated { needs: table, has });
    }
    let mut segments = Vec::new();
    let mut gp = 0;
    for i in 0..phnum {
        let h = &image[(phoff + i * phentsize) as usize..];
        if u32_at(h, 0) == PT_MIPS_REGINFO && u32_at(h, 16) >= 24 && u64::from(u32_at(h, 4)) + 24 <= has {
            gp = u32_at(image, u32_at(h, 4) as usize + 20);
        }
        if u32_at(h, 0) != PT_LOAD {
            continue;
        }
        let (offset, address, file, memory) = (u32_at(h, 4), u32_at(h, 8), u32_at(h, 16), u32_at(h, 20));
        let end = offset as u64 + file as u64;
        if end > has {
            return Err(Refusal::Truncated { needs: end, has });
        }
        if file > memory {
            return not(format!("an executable whose segment at {address:#010x} holds more bytes in the file than in memory"));
        }
        segments.push((address, offset, file, memory));
    }
    if segments.is_empty() {
        return not("an executable with no segment to load".into());
    }
    Ok(Elf { entry, segments, gp })
}

/// A line of SYSTEM.CNF's text by its key, its value trimmed.
fn config_line(config: Option<&[u8]>, key: &str) -> Option<String> {
    let text = String::from_utf8_lossy(config?).into_owned();
    text.split('\n').find_map(|l| l.split_once('=').filter(|(k, _)| k.trim().eq_ignore_ascii_case(key)).map(|(_, v)| v.trim().to_string()))
}

/// The program a disc's SYSTEM.CNF names as BOOT2, its sectors' user data given by `sector` (the drive's reader).
pub fn boot2_by(sector: &mut dyn FnMut(u64) -> Vec<u8>) -> Option<String> {
    config_line(root_file_by(sector, "SYSTEM.CNF").as_deref(), "BOOT2")
}

fn read_disc(sector: &mut dyn FnMut(u64) -> Vec<u8>, count: u64, raw: bool) -> Result<Disc, Refusal> {
    let config = root_file_by(sector, "SYSTEM.CNF");
    let line = |key: &str| config_line(config.as_deref(), key);
    if let Some(program) = line("BOOT2") {
        let pal = line("VMODE").is_some_and(|v| v.eq_ignore_ascii_case("PAL"));
        return Ok(Disc { raw, sectors: count, serial: serial_of(&program), program, pal });
    }
    if line("BOOT").is_some() || (config.is_none() && root_file_by(sector, "PSX.EXE").is_some()) || sector(4).starts_with(LICENCE) {
        return Err(Refusal::PlayStation);
    }
    Err(Refusal::NoBoot2(if config.is_some() { "its SYSTEM.CNF has no BOOT2 line" } else { "its root holds no readable SYSTEM.CNF" }))
}

/// A boot line's program as a serial, as the systems pack reads it: "cdrom0:\SLUS_203.12;1" is SLUS-20312.
pub fn serial_of(program: &str) -> Option<String> {
    let cut = program.rfind('\\').max(program.rfind(':')).map_or(0, |i| i + 1);
    let file = program[cut..].split(';').next().unwrap_or("").replace('.', "").replace('_', "-");
    let file = file.trim().to_ascii_uppercase();
    ((6..=12).contains(&file.len()) && file.contains('-')).then_some(file)
}

/// An image's data sectors, 2,048 bytes each: an .iso's, or a raw track's 2,352-byte sectors told by their sync, the data
/// 16 bytes in for mode 1 and 24 for mode 2 (ECMA-130 §14).
pub struct Sectors<'a> {
    bytes: &'a [u8],
    raw: bool,
}

impl<'a> Sectors<'a> {
    pub fn new(bytes: &'a [u8]) -> Sectors<'a> {
        Sectors { bytes, raw: bytes.len() >= RAW && bytes[..12] == SYNC }
    }

    pub fn count(&self) -> u64 {
        (self.bytes.len() / if self.raw { RAW } else { USER }) as u64
    }

    /// Sector n's user data, or empty past the image's end.
    pub fn sector(&self, n: u64) -> &'a [u8] {
        let Ok(n) = usize::try_from(n) else { return &[] };
        if !self.raw {
            return n.checked_mul(USER).and_then(|at| self.bytes.get(at..at + USER)).unwrap_or(&[]);
        }
        let Some(at) = n.checked_mul(RAW).filter(|at| at + RAW <= self.bytes.len()) else { return &[] };
        let data = if self.bytes[at + 15] == 1 { 16 } else { 24 };
        &self.bytes[at + data..at + data + USER]
    }

    /// The primary volume descriptor's root directory, its extent and size (ECMA-119 §8.4), where sector 16 holds one.
    pub fn volume(&self) -> Option<(u64, u64)> {
        volume_by(&mut |n| self.sector(n).to_vec())
    }

    /// A file of the root directory by its name without the version (ECMA-119 §9.1), where the image holds it whole.
    pub fn root_file(&self, name: &str) -> Option<Vec<u8>> {
        root_file_by(&mut |n| self.sector(n).to_vec(), name)
    }
}

/// The root directory's extent and size from the primary volume descriptor, sectors given by `sector`.
fn volume_by(sector: &mut dyn FnMut(u64) -> Vec<u8>) -> Option<(u64, u64)> {
    let pvd = sector(16);
    (pvd.len() >= 190 && pvd[0] == 1 && &pvd[1..6] == b"CD001").then(|| (u32_at(&pvd, 158) as u64, u32_at(&pvd, 166) as u64))
}

/// A root directory's file by its name without the version, sectors given by `sector`, empty past the disc's end.
fn root_file_by(sector: &mut dyn FnMut(u64) -> Vec<u8>, name: &str) -> Option<Vec<u8>> {
    let (extent, size) = volume_by(sector)?;
    for block in 0..size.div_ceil(USER as u64) {
        let dir = sector(extent + block);
        let mut at = 0;
        while at + 33 < dir.len() && dir[at] != 0 {
            let length = dir[at + 32] as usize;
            if at + 33 + length > dir.len() {
                break;
            }
            let entry = &dir[at + 33..at + 33 + length];
            let stem = entry.split(|&c| c == b';').next().unwrap_or(&[]);
            if stem.eq_ignore_ascii_case(name.as_bytes()) {
                let (start, bytes) = (u32_at(&dir, at + 2) as u64, u32_at(&dir, at + 10) as usize);
                let mut file = Vec::with_capacity(bytes.min(1 << 20));
                for s in 0..bytes.div_ceil(USER) as u64 {
                    let part = sector(start + s);
                    if part.is_empty() {
                        return None;
                    }
                    file.extend_from_slice(&part[..USER.min(bytes - file.len())]);
                }
                return Some(file);
            }
            at += dir[at] as usize;
        }
        if dir.is_empty() {
            return None;
        }
    }
    None
}

/// Where the project's programs are loaded and entered.
pub const PROGRAM_ORIGIN: u32 = 0x0010_0000;

/// Where the segment starts in the file: congruent with the origin modulo the segment's alignment, 16, as the ELF
/// format requires; at 84 bytes, a 160 KiB program read its constants 12 bytes astray on PCSX2 (Sapphire_Native.md
/// §16.6).
pub const SEGMENT_OFFSET: u32 = 0x60;

/// An EE program of the project's own as an ELF: the header, one loadable segment of `code` at `origin`, entered there
/// (Sapphire_Referees.md §4.1). The tests' sample and the referees' programs are written by it.
pub fn program(code: &[u32], origin: u32) -> Vec<u8> {
    let mut image = vec![0u8; ELF_HEADER + PROGRAM_HEADER];
    image[..4].copy_from_slice(&ELF_MAGIC);
    image[4..7].copy_from_slice(&[1, 1, 1]);
    image[16..18].copy_from_slice(&ET_EXEC.to_le_bytes());
    image[18..20].copy_from_slice(&EM_MIPS.to_le_bytes());
    image[20..24].copy_from_slice(&1u32.to_le_bytes());
    image[24..28].copy_from_slice(&origin.to_le_bytes());
    image[28..32].copy_from_slice(&(ELF_HEADER as u32).to_le_bytes());
    image[40..42].copy_from_slice(&(ELF_HEADER as u16).to_le_bytes());
    image[42..44].copy_from_slice(&(PROGRAM_HEADER as u16).to_le_bytes());
    image[44..46].copy_from_slice(&1u16.to_le_bytes());
    image.resize(SEGMENT_OFFSET as usize, 0);
    let h = &mut image[ELF_HEADER..];
    let (offset, bytes) = (SEGMENT_OFFSET, code.len() as u32 * 4);
    for (at, v) in [(0, PT_LOAD), (4, offset), (8, origin), (12, origin), (16, bytes), (20, bytes), (24, 5), (28, 16)] {
        h[at..at + 4].copy_from_slice(&v.to_le_bytes());
    }
    image.extend(code.iter().flat_map(|w| w.to_le_bytes()));
    image
}

/// Images the tests build: never a real program or disc's data (Sapphire_Native.md §5).
impl Media {
    /// What a state names its image by, never 0 (Sapphire_Native.md §38.2): FNV-1a over what recognition read, a
    /// program's entry and segments, a disc's framing, sector count, BOOT2 program and serial. It reads nothing more of
    /// the image, so its cost does not grow with the disc.
    pub fn identity(&self) -> u64 {
        let mut h = 0xCBF2_9CE4_8422_2325u64;
        let mut eat = |bytes: &[u8]| bytes.iter().for_each(|&b| h = (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01B3));
        match self {
            Media::Elf(e) => {
                eat(b"ELF");
                eat(&e.entry.to_le_bytes());
                for s in &e.segments {
                    for v in [s.0, s.1, s.2, s.3] {
                        eat(&v.to_le_bytes());
                    }
                }
            }
            Media::Disc(d) => {
                eat(b"DISC");
                eat(&[u8::from(d.raw)]);
                eat(&d.sectors.to_le_bytes());
                eat(d.program.as_bytes());
                eat(&[0]);
                eat(d.serial.as_deref().unwrap_or("").as_bytes());
            }
            Media::Empty => eat(b"EMPTY"),
        }
        h.max(1)
    }
}

#[cfg(test)]
pub mod sample {
    use super::*;

    /// An EE program: the ELF header, one loadable segment at 00100000h, and a branch to itself with its delay slot.
    pub fn elf() -> Vec<u8> {
        program(&[0x1000_FFFF, 0], PROGRAM_ORIGIN)
    }

    /// An ISO 9660 volume of 2,048-byte sectors, its root directory at `root` holding SYSTEM.CNF in the next sector, and
    /// the licence text in sector 4 where asked.
    pub fn volume(root: usize, config: &str, licence: bool) -> Vec<u8> {
        let mut image = vec![0u8; (root + 2) * USER];
        if licence {
            image[4 * USER..4 * USER + 32].copy_from_slice(LICENCE);
        }
        let pvd = &mut image[16 * USER..17 * USER];
        pvd[0] = 1;
        pvd[1..6].copy_from_slice(b"CD001");
        pvd[158..162].copy_from_slice(&(root as u32).to_le_bytes());
        pvd[166..170].copy_from_slice(&(USER as u32).to_le_bytes());
        let name = b"SYSTEM.CNF;1";
        let dir = &mut image[root * USER..(root + 1) * USER];
        dir[0] = (34 + name.len()) as u8;
        dir[2..6].copy_from_slice(&(root as u32 + 1).to_le_bytes());
        dir[10..14].copy_from_slice(&(config.len() as u32).to_le_bytes());
        dir[32] = name.len() as u8;
        dir[33..33 + name.len()].copy_from_slice(name);
        image[(root + 1) * USER..(root + 1) * USER + config.len()].copy_from_slice(config.as_bytes());
        image
    }

    pub const PS2_CNF: &str = "BOOT2 = cdrom0:\\SLUS_209.99;1\r\nVER = 1.00\r\nVMODE = NTSC\r\n";

    /// A PlayStation 2 CD's volume, and a DVD's with its root past the first 600 KiB as a bridge volume places it.
    pub fn cd() -> Vec<u8> {
        volume(20, PS2_CNF, false)
    }

    pub fn dvd() -> Vec<u8> {
        volume(300, PS2_CNF, false)
    }

    /// The same volume as mode 2 form 1 raw sectors: the sync, the mode byte, the subheader, the data.
    pub fn raw(user: &[u8]) -> Vec<u8> {
        user.chunks(USER)
            .flat_map(|data| {
                let mut sector = vec![0u8; RAW];
                sector[..12].copy_from_slice(&SYNC);
                sector[15] = 2;
                sector[24..24 + USER].copy_from_slice(data);
                sector
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::sample::*;
    use super::*;

    #[test]
    fn an_ee_program_is_its_entry_and_segments() {
        let Ok(Media::Elf(elf)) = Media::read(&elf()) else { panic!("not read") };
        assert_eq!((elf.entry, elf.segments.as_slice()), (0x0010_0000, &[(0x0010_0000, SEGMENT_OFFSET, 8, 8)][..]));
    }

    #[test]
    fn an_elf_that_is_no_ee_program_is_refused_with_what_it_is() {
        let with = |at: usize, bytes: &[u8]| {
            let mut e = elf();
            e[at..at + bytes.len()].copy_from_slice(bytes);
            Media::read(&e)
        };
        let words = |r: Result<Media, Refusal>| match r {
            Err(Refusal::NotEe(w)) => w,
            other => panic!("{other:?}"),
        };
        assert_eq!(words(with(4, &[2])), "a 64-bit ELF");
        assert_eq!(words(with(5, &[2])), "a big-endian ELF");
        assert_eq!(words(with(18, &62u16.to_le_bytes())), "an ELF for machine 62, not MIPS");
        assert!(words(with(16, &ET_IRX.to_le_bytes())).starts_with("an IOP module"));
        assert_eq!(words(with(16, &1u16.to_le_bytes())), "an ELF of type 0x0001, not an executable");
        assert_eq!(words(with(ELF_HEADER, &2u32.to_le_bytes())), "an executable with no segment to load");
        let e = elf();
        assert_eq!(Media::read(&e[..e.len() - 1]), Err(Refusal::Truncated { needs: SEGMENT_OFFSET as u64 + 8, has: SEGMENT_OFFSET as u64 + 7 }));
        assert_eq!(Media::read(&e[..60]), Err(Refusal::Truncated { needs: 84, has: 60 }));
        assert_eq!(Media::read(&e[..20]), Err(Refusal::Truncated { needs: 52, has: 20 }));
    }

    #[test]
    fn a_playstation_2_disc_is_its_boot2_program_in_either_sector_form() {
        for (image, raw) in [(cd(), false), (raw(&cd()), true), (dvd(), false)] {
            let Ok(Media::Disc(d)) = Media::read(&image) else { panic!("not read") };
            assert_eq!((d.raw, d.program.as_str(), d.serial.as_deref()), (raw, "cdrom0:\\SLUS_209.99;1", Some("SLUS-20999")));
        }
        assert_eq!(Media::read(&volume(20, "BOOT2 = cdrom0:\\SLPS_250.01;1", true)).map(|m| m.serial().map(str::to_string)), Ok(Some("SLPS-25001".into())), "the licence text does not make it a PlayStation disc");
    }

    #[test]
    fn a_playstation_disc_and_other_images_are_refused_with_words() {
        assert_eq!(Media::read(&volume(20, "BOOT = cdrom:\\SLUS_007.70;1\r\n", true)), Err(Refusal::PlayStation));
        assert_eq!(Media::read(&raw(&volume(20, "BOOT = cdrom:\\SLUS_007.70;1\r\n", true))), Err(Refusal::PlayStation));
        assert_eq!(Media::read(&volume(20, "VMODE = NTSC\r\n", false)), Err(Refusal::NoBoot2("its SYSTEM.CNF has no BOOT2 line")));
        let mut nameless = cd();
        nameless[20 * USER + 33] = b'X';
        assert_eq!(Media::read(&nameless), Err(Refusal::NoBoot2("its root holds no readable SYSTEM.CNF")));
        for junk in [&[][..], &[0u8; 100][..], &[0xA5; 40_000][..]] {
            assert_eq!(Media::read(junk), Err(Refusal::Unknown));
        }
        let cut = cd();
        assert_eq!(Media::read(&cut[..21 * USER]), Err(Refusal::NoBoot2("its root holds no readable SYSTEM.CNF")), "a file past the image's end is not read");
    }
}
