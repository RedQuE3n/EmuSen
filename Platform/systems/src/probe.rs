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
        let pvd = self.sector(16);
        if pvd.len() < 190 || pvd[0] != 1 || &pvd[1..6] != b"CD001" {
            return None;
        }
        let (extent, size) = (le32(&pvd[158..]) as i64, le32(&pvd[166..]) as i64);
        let mut block = 0i64;
        while block * (USER as i64) < size {
            let dir = self.sector(extent + block);
            let mut at = 0usize;
            while at + 33 < dir.len() && dir[at] != 0 {
                let length = dir[at + 32] as usize;
                if at + 33 + length > dir.len() {
                    break;
                }
                let entry = crate::media::ascii(&dir[at + 33..at + 33 + length]);
                if equals_ignore_case(entry.split(';').next().unwrap_or(""), name) {
                    let (start, length) = (le32(&dir[at + 2..]) as i64, le32(&dir[at + 10..]) as i64);
                    if length < 0 {
                        return None;
                    }
                    let mut file = Vec::new();
                    let (mut done, mut s) = (0i64, start);
                    while done < length {
                        let part = self.sector(s);
                        if part.is_empty() {
                            return None;
                        }
                        file.extend_from_slice(&part[..(USER as i64).min(length - done) as usize]);
                        done += USER as i64;
                        s += 1;
                    }
                    return Some(file);
                }
                at += dir[at] as usize;
            }
            block += 1;
        }
        None
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
    matches!(system, "md" | "mcd" | "psx")
}

/// The system's recogniser over the head; none for a system without one.
pub fn recognise(system: &str, head: &Head) -> Result<Option<Recognition>, Need> {
    match system {
        "md" | "mcd" => crate::genesis::recognise(system, head),
        "psx" => crate::psx::disc::recognise(head),
        _ => Ok(None),
    }
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
}
