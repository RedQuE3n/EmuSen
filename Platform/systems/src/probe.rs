//! Routing by content: what a recogniser is shown of a file (its first bytes, asked for by length), the data track's
//! sectors and the ISO 9660 lookup a recogniser reads a file by, and the decision among the systems that take a file's
//! extension (EmuSen_CoreAPI_Extensions.md §8.4). Reading the file is the host's: a recogniser that needs more bytes
//! than it was given says how many, and the host reads them and asks again.

use emusen_dotnet::text::equals_ignore_case;

/// A recogniser's claim: the system, and the game's serial and market where the pack can read them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recognition {
    pub system: String,
    pub serial: Option<String>,
    pub region: Option<String>,
}

/// More of the file is needed: this many bytes from its start.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Need(pub usize);

/// The first bytes of a file as far as the host has read it: `asked` is the length it asked for, `bytes` what it got,
/// shorter where the file is.
#[derive(Clone, Copy, Debug)]
pub struct Head<'a> {
    pub bytes: &'a [u8],
    pub asked: usize,
}

impl<'a> Head<'a> {
    /// The first `length` bytes, or fewer where the file ends; `Need` where the host has not asked for so many.
    pub fn read(&self, length: usize) -> Result<&'a [u8], Need> {
        if length > self.asked {
            return Err(Need(length));
        }
        Ok(&self.bytes[..length.min(self.bytes.len())])
    }
}

pub const SYNC: [u8; 12] = [0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00];
pub const USER: usize = 2048;
pub const RAW: usize = 2352;

/// A data track's first sectors as user data, from an .iso's 2,048-byte sectors or a raw track's 2,352-byte ones.
pub struct DiscSectors<'a> {
    bytes: &'a [u8],
    raw: bool,
}

fn le32(b: &[u8]) -> i32 {
    i32::from_le_bytes([b[0], b[1], b[2], b[3]])
}

impl<'a> DiscSectors<'a> {
    pub fn new(head: &'a [u8]) -> DiscSectors<'a> {
        DiscSectors { bytes: head, raw: head.len() >= RAW && head[..12] == SYNC }
    }

    pub fn is_raw(&self) -> bool {
        self.raw
    }

    /// Sector n's user data, or empty where the head does not reach it or n is before the first.
    pub fn sector(&self, n: i64) -> &'a [u8] {
        if n < 0 {
            return &[];
        }
        let n = n as usize;
        if !self.raw {
            return if (n + 1) * USER <= self.bytes.len() { &self.bytes[n * USER..(n + 1) * USER] } else { &[] };
        }
        let at = n * RAW;
        if at + RAW > self.bytes.len() {
            return &[];
        }
        let data = if self.bytes[at + 15] == 1 { 16 } else { 24 };
        &self.bytes[at + data..at + data + USER]
    }

    /// The bytes of a file in the volume's root directory, by its name without the version, where the head holds
    /// them; none otherwise, a record that runs past its sector ending that sector's scan.
    pub fn root_file(&self, name: &str) -> Option<Vec<u8>> {
        match self.reach(name) {
            Reach::Found(file) => Some(file),
            Reach::Absent | Reach::Needs(_) => None,
        }
    }

    /// The image's bytes up to the end of sector n, in this head's sector size.
    fn end_of(&self, n: i64) -> usize {
        let size = if self.raw { RAW } else { USER };
        usize::try_from(n.saturating_add(1)).unwrap_or(0).saturating_mul(size)
    }

    /// Where a root file stands in this head: its bytes, absent from the volume, or past the head's end, the image's
    /// bytes it needs named (Sapphire_Native.md §3.2).
    pub fn reach(&self, name: &str) -> Reach {
        let pvd = self.sector(16);
        if pvd.len() < 190 || pvd[0] != 1 || &pvd[1..6] != b"CD001" {
            return Reach::Absent;
        }
        let (extent, size) = (le32(&pvd[158..]) as i64, le32(&pvd[166..]) as i64);
        let mut block = 0i64;
        while block * (USER as i64) < size {
            let dir = self.sector(extent + block);
            if dir.is_empty() && extent + block >= 0 {
                return Reach::Needs(self.end_of(extent + block));
            }
            let mut at = 0usize;
            while at + 33 < dir.len() && dir[at] != 0 {
                let length = dir[at + 32] as usize;
                if at + 33 + length > dir.len() {
                    break;
                }
                let entry = crate::media::ascii(&dir[at + 33..at + 33 + length]);
                if equals_ignore_case(entry.split(';').next().unwrap_or(""), name) {
                    let (start, length) = (le32(&dir[at + 2..]) as i64, le32(&dir[at + 10..]) as i64);
                    if length < 0 || start < 0 {
                        return Reach::Absent;
                    }
                    let mut file = Vec::new();
                    let (mut done, mut s) = (0i64, start);
                    while done < length {
                        let part = self.sector(s);
                        if part.is_empty() {
                            return Reach::Needs(self.end_of(start + (length - 1) / USER as i64));
                        }
                        file.extend_from_slice(&part[..(USER as i64).min(length - done) as usize]);
                        done += USER as i64;
                        s += 1;
                    }
                    return Reach::Found(file);
                }
                at += dir[at] as usize;
            }
            block += 1;
        }
        Reach::Absent
    }
}

/// Where a root file stands in a head (`DiscSectors::reach`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reach {
    Found(Vec<u8>),
    Absent,
    /// The file or its directory lies past the head: this many bytes of the image hold it.
    Needs(usize),
}

/// How far into an image a recogniser reads for a root file: 32 MiB, past where a DVD's bridge volume puts its
/// directory and first files (Sapphire_Native.md §3.2).
pub const ROOT_FILE_LIMIT: usize = 32 << 20;

/// A root file read through the head from `first` bytes on, asking for as much of the image as the volume places the
/// directory and the file in, up to `limit` bytes; none where the volume or the file is absent, the image ends before
/// it, or it lies past the limit.
pub fn root_file_through(head: &Head, first: usize, name: &str, limit: usize) -> Result<Option<Vec<u8>>, Need> {
    let mut length = first;
    loop {
        let bytes = head.read(length)?;
        match DiscSectors::new(bytes).reach(name) {
            Reach::Found(file) => return Ok(Some(file)),
            Reach::Needs(n) if n > length && n <= limit && bytes.len() >= length => length = n,
            Reach::Absent | Reach::Needs(_) => return Ok(None),
        }
    }
}

/// A system as routing names it: its id, name and ES-DE folder name.
#[derive(Clone, Copy, Debug)]
pub struct Candidate<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub esde_system: &'a str,
}

/// The decision: a system's id, the recognition that made it where one did, or none with words naming the candidates.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Routed {
    pub system: Option<String>,
    pub recognition: Option<Recognition>,
    pub reason: Option<String>,
}

/// Among candidates, each a system that takes the file's extension: the only one; else the one whose recogniser
/// claimed the file, each claim by the candidate's place; else the one its folder is named for; else none, with words.
pub fn route(file_name: &str, folder: Option<&str>, candidates: &[Candidate], claims: &[(usize, Recognition)]) -> Routed {
    let chosen = |c: &Candidate, r: Option<Recognition>| Routed { system: Some(c.id.to_string()), recognition: r, reason: None };
    match candidates {
        [only] => return chosen(only, None),
        [] => return Routed { system: None, recognition: None, reason: Some(format!("no system takes {file_name}")) },
        _ => {}
    }
    if let [(at, claim)] = claims {
        return chosen(&candidates[*at], Some(claim.clone()));
    }
    let named: Vec<usize> = (0..candidates.len()).filter(|&i| folder.is_some_and(|f| equals_ignore_case(candidates[i].esde_system, f))).collect();
    if let [one] = named[..] {
        return chosen(&candidates[one], claims.iter().find(|(at, _)| *at == one).map(|(_, r)| r.clone()));
    }
    let which = candidates.iter().map(|c| c.name).collect::<Vec<_>>().join(" or ");
    let folders = candidates.iter().map(|c| c.esde_system).collect::<Vec<_>>().join(" or ");
    let content = if claims.is_empty() { "no console recognises its content" } else { "more than one console recognises its content" };
    Routed {
        system: None,
        recognition: None,
        reason: Some(format!("{file_name} could be {which}: {content}, and its folder is named for none; move it into a folder named {folders}")),
    }
}

/// Whether a system has a recogniser, which only consoles sharing an extension with another need.
pub fn recognises(system: &str) -> bool {
    matches!(system, "md" | "mcd" | "32x" | "psx" | "ps2")
}

/// The system's recogniser over the head; none for a system without one.
pub fn recognise(system: &str, head: &Head) -> Result<Option<Recognition>, Need> {
    match system {
        "md" | "mcd" | "32x" => crate::genesis::recognise(system, head),
        "psx" => crate::psx::disc::recognise(head),
        "ps2" => crate::ps2::disc::recognise(head),
        _ => Ok(None),
    }
}

/// Each candidate's claim that has a recogniser, by its place, reading the file through `read` as each recogniser asks
/// for more; a read that fails ends the asking, keeping what was claimed before it, as the C#'s one `catch` round the
/// loop does (`SystemRouting.Recognised`).
pub fn claims(candidates: &[&str], read: &mut dyn FnMut(usize) -> Result<Vec<u8>, String>) -> Vec<(usize, Recognition)> {
    let mut claims = Vec::new();
    let (mut head, mut cached) = (Vec::new(), 0usize);
    for (place, system) in candidates.iter().enumerate() {
        if !recognises(system) {
            continue;
        }
        let mut asked = 0usize;
        loop {
            let shown: &[u8] = if asked == 0 { &[] } else { &head[..head.len().min(asked)] };
            match recognise(system, &Head { bytes: shown, asked }) {
                Ok(Some(claim)) => {
                    claims.push((place, claim));
                    break;
                }
                Ok(None) => break,
                Err(Need(need)) => {
                    if need <= asked {
                        return claims;
                    }
                    if need > cached {
                        match read(need) {
                            Ok(bytes) => head = bytes,
                            Err(_) => return claims,
                        }
                        cached = need;
                    }
                    asked = need;
                }
            }
        }
    }
    claims
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_recogniser_asks_for_what_it_reads_and_answers_on_less_where_the_file_ends() {
        let iso = [0u8; 100];
        assert_eq!(recognise("psx", &Head { bytes: &iso, asked: 0 }), Err(Need(12 * 1024)));
        assert_eq!(recognise("psx", &Head { bytes: &iso, asked: 12 * 1024 }), Ok(None));
        assert_eq!(recognise("md", &Head { bytes: &iso, asked: 0x8010 }).unwrap().map(|r| r.system), Some("md".into()));
        let mut sega = [0u8; 0x40];
        sega[..14].copy_from_slice(b"SEGADISCSYSTEM");
        assert_eq!(recognise("mcd", &Head { bytes: &sega, asked: 0x40 }).unwrap().map(|r| r.system), Some("mcd".into()));
        assert_eq!(recognise("nes", &Head { bytes: &[], asked: 0 }), Ok(None));
        let mut mars = [0u8; 0x200];
        mars[0x104..0x107].copy_from_slice(b"32X");
        let claimant = |image: &[u8]| ["md", "32x"].into_iter().filter(|s| recognise(s, &Head { bytes: image, asked: 0x8010 }).unwrap().is_some()).collect::<Vec<_>>();
        assert_eq!(recognise("32x", &Head { bytes: &mars, asked: 0 }), Err(Need(0x8010)));
        assert_eq!((claimant(&mars), claimant(&mars[..0x1FF])), (vec!["32x"], vec!["md"]), "a header needs its 256 bytes");
        let mut disc = mars;
        disc[..12].copy_from_slice(&SYNC);
        assert!(claimant(&disc).is_empty(), "a disc is neither's");
    }

    #[test]
    fn routing_takes_the_one_claim_then_the_folder_then_says_why_not() {
        let c = [Candidate { id: "mcd", name: "Sega CD / Mega-CD", esde_system: "segacd" }, Candidate { id: "psx", name: "Sony PlayStation", esde_system: "psx" }];
        let claim = |id: &str| Recognition { system: id.into(), serial: None, region: None };
        assert_eq!(route("a.iso", None, &c, &[(1, claim("psx"))]).system.as_deref(), Some("psx"));
        assert_eq!(route("a.iso", Some("SEGACD"), &c, &[]).system.as_deref(), Some("mcd"));
        assert_eq!(
            route("a.iso", Some("roms"), &c, &[]).reason.as_deref(),
            Some("a.iso could be Sega CD / Mega-CD or Sony PlayStation: no console recognises its content, and its folder is named for none; move it into a folder named segacd or psx")
        );
        assert_eq!(route("a.x", None, &[], &[]).reason.as_deref(), Some("no system takes a.x"));
    }

    // .sms and .sg are each one system's, chosen at once whatever the folder; .bin stays the Genesis's, the 32X's and the PlayStation's (Endou_Plan.md §4.1, EmuSen_Backend.md §32).
    #[test]
    fn a_master_system_or_sg1000_image_routes_to_its_one_system() {
        let table = crate::table().unwrap();
        let candidates = |file: &str| -> Vec<Candidate> {
            let ext = &file[file.rfind('.').unwrap()..];
            table.iter().filter(|s| s.extensions.iter().any(|e| e == ext)).map(|s| Candidate { id: &s.id, name: &s.name, esde_system: &s.esde_system }).collect()
        };
        for (file, system) in [("Alex Kidd.sms", "sms"), ("Flicky.sg", "sg1000")] {
            for folder in [None, Some("mastersystem"), Some("sg-1000"), Some("genesis")] {
                let routed = route(file, folder, &candidates(file), &[]);
                assert_eq!((routed.system.as_deref(), routed.recognition, routed.reason), (Some(system), None, None), "{file} in {folder:?}");
            }
        }
        assert_eq!(candidates("game.bin").iter().map(|c| c.id).collect::<Vec<_>>(), ["md", "32x", "psx", "ps2"]);
        assert!(candidates("game.gg").is_empty(), "the Game Gear is not a system before J3");
    }

    // A cartridge's .bin routes by its header's system name, whatever its folder: a 32X's to the 32X, a Genesis's to the Genesis (EmuSen_Backend.md §32).
    #[test]
    fn a_32x_cartridge_named_bin_routes_to_the_32x_by_its_header() {
        let table = crate::table().unwrap();
        let candidates: Vec<Candidate> = table.iter().filter(|s| s.extensions.iter().any(|e| e == ".bin")).map(|s| Candidate { id: &s.id, name: &s.name, esde_system: &s.esde_system }).collect();
        let ids: Vec<&str> = candidates.iter().map(|c| c.id).collect();
        let cartridge = |system: &[u8]| {
            let mut image = vec![0u8; 0x20000];
            image[0x100..0x110].copy_from_slice(&[system, &b"                "[system.len()..]].concat());
            image
        };
        for (system, expected) in [(&b"SEGA 32X"[..], "32x"), (b"SEGA GENESIS", "md"), (b"SEGA MEGA DRIVE", "md")] {
            let image = cartridge(system);
            for folder in [None, Some("genesis"), Some("sega32x"), Some("roms")] {
                let claims = claims(&ids, &mut |n| Ok(image[..n.min(image.len())].to_vec()));
                let routed = route("game.bin", folder, &candidates, &claims);
                assert_eq!(routed.system.as_deref(), Some(expected), "{} in {folder:?}", String::from_utf8_lossy(system));
            }
        }
    }

    /// An ISO 9660 volume of 2,048-byte sectors: the licence text in sector 4 where asked, the primary volume descriptor
    /// at 16, the root directory at `root` holding SYSTEM.CNF in the sector after it (ECMA-119 §8.4, §9.1).
    pub(crate) fn volume(root: usize, config: &str, licence: bool) -> Vec<u8> {
        let mut image = vec![0u8; (root + 2) * USER];
        if licence {
            image[4 * USER..4 * USER + 32].copy_from_slice(b"          Licensed  by          ");
        }
        let pvd = &mut image[16 * USER..17 * USER];
        pvd[0] = 1;
        pvd[1..6].copy_from_slice(b"CD001");
        pvd[158..162].copy_from_slice(&(root as u32).to_le_bytes());
        pvd[166..170].copy_from_slice(&(USER as u32).to_le_bytes());
        let name = b"SYSTEM.CNF;1";
        let dir = &mut image[root * USER..(root + 1) * USER];
        dir[0] = (33 + name.len() + 1) as u8;
        dir[2..6].copy_from_slice(&(root as u32 + 1).to_le_bytes());
        dir[10..14].copy_from_slice(&(config.len() as u32).to_le_bytes());
        dir[32] = name.len() as u8;
        dir[33..33 + name.len()].copy_from_slice(name);
        image[(root + 1) * USER..(root + 1) * USER + config.len()].copy_from_slice(config.as_bytes());
        image
    }

    /// The same volume as a raw track of mode 2 form 1 sectors: the sync, the mode byte, the subheader, then the data.
    pub(crate) fn raw(user: &[u8]) -> Vec<u8> {
        user.chunks(USER).flat_map(|data| {
            let mut sector = vec![0u8; RAW];
            sector[..12].copy_from_slice(&SYNC);
            sector[15] = 2;
            sector[24..24 + USER].copy_from_slice(data);
            sector
        }).collect()
    }

    // A PlayStation disc goes to the PlayStation and a PlayStation 2 CD or DVD to the PlayStation 2, by SYSTEM.CNF's BOOT and BOOT2, whatever the folder (Sapphire_Native.md §3.2).
    #[test]
    fn a_playstation_disc_routes_to_the_playstation_and_a_playstation_2_disc_to_the_playstation_2() {
        let table = crate::table().unwrap();
        let candidates: Vec<Candidate> = table.iter().filter(|s| s.extensions.iter().any(|e| e == ".iso")).map(|s| Candidate { id: &s.id, name: &s.name, esde_system: &s.esde_system }).collect();
        let ids: Vec<&str> = candidates.iter().map(|c| c.id).collect();
        assert_eq!(ids, ["mcd", "psx", "ps2"]);
        let ps1 = volume(20, "BOOT = cdrom:\\SLUS_007.70;1\r\nTCB = 4\r\n", true);
        let ps2_cd = volume(20, "BOOT2 = cdrom0:\\SLUS_203.12;1\r\nVER = 1.00\r\nVMODE = NTSC\r\n", false);
        let ps2_dvd = volume(300, "BOOT2 = cdrom0:\\SCES_500.00;1\r\nVER = 1.00\r\nVMODE = PAL\r\n", false);
        let licensed_ps2 = volume(20, "BOOT2 = cdrom0:\\SLPS_250.01;1\r\n", true);
        for (label, image, system, serial) in [
            ("ps1 iso", ps1.clone(), "psx", "SLUS-00770"),
            ("ps1 raw", raw(&ps1), "psx", "SLUS-00770"),
            ("ps2 cd iso", ps2_cd.clone(), "ps2", "SLUS-20312"),
            ("ps2 cd raw", raw(&ps2_cd), "ps2", "SLUS-20312"),
            ("ps2 dvd", ps2_dvd.clone(), "ps2", "SCES-50000"),
            ("ps2 with the licence text", licensed_ps2.clone(), "ps2", "SLPS-25001"),
        ] {
            let claims = claims(&ids, &mut |n| Ok(image[..n.min(image.len())].to_vec()));
            assert_eq!(claims.len(), 1, "{label}: one console claims it");
            for folder in [None, Some("psx"), Some("ps2"), Some("segacd"), Some("roms")] {
                let routed = route("game.iso", folder, &candidates, &claims);
                assert_eq!((routed.system.as_deref(), routed.recognition.and_then(|r| r.serial)), (Some(system), Some(serial.to_string())), "{label} in {folder:?}");
            }
        }
        let mut asked = Vec::new();
        let _ = claims(&["ps2"], &mut |n| { asked.push(n); Ok(ps2_dvd[..n.min(ps2_dvd.len())].to_vec()) });
        assert_eq!(asked, [40 * 1024, 301 * USER, 302 * USER], "the DVD's directory and file are asked for where the volume places them");
    }

    #[test]
    fn a_root_file_past_the_limit_or_the_images_end_is_none() {
        let far = volume(300, "BOOT2 = cdrom0:\\SLUS_203.12;1", false);
        let head = |asked: usize| Head { bytes: &far[..asked.min(far.len())], asked };
        assert_eq!(root_file_through(&head(far.len()), 40 * 1024, "SYSTEM.CNF", 100 * USER), Ok(None), "past the limit");
        assert_eq!(root_file_through(&head(far.len()), 40 * 1024, "SYSTEM.CNF", ROOT_FILE_LIMIT).unwrap().map(|f| f.len()), Some(29));
        let cut = Head { bytes: &far[..300 * USER + 100], asked: 1 << 20 };
        assert_eq!(root_file_through(&cut, 40 * 1024, "SYSTEM.CNF", ROOT_FILE_LIMIT), Ok(None), "the image ends before the file");
        assert_eq!(root_file_through(&head(40 * 1024), 40 * 1024, "SYSTEM.CNF", ROOT_FILE_LIMIT), Err(Need(301 * USER)));
        assert_eq!(DiscSectors::new(&far).root_file("SYSTEM.CNF").map(|f| f.len()), Some(29));
        assert_eq!(DiscSectors::new(&far[..300 * USER]).reach("SYSTEM.CNF"), Reach::Needs(301 * USER));
    }
}
