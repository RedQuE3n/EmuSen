//! The tester's Sega CD discs, found through `EMUSEN_NEPHRITE_DISCS` (a folder of folders, each a `.cue` with its
//! tracks), opened as a host passes them and read by the drive and the CDC with no BIOS: the sheet's tracks, the TOC
//! the drive reports, the system area at 00:02:00, and audio from an audio track. Without the variable they pass
//! unrun (Nephrite_Native.md §53).

use emusen_native::core::Kept;

use crate::mcd::cdd::{Cdd, SPIN_UP, status};

pub const DISCS_VARIABLE: &str = "EMUSEN_NEPHRITE_DISCS";

/// Each disc's name, sheet and track files' bytes.
fn discs() -> Vec<(String, Vec<u8>, Vec<Vec<u8>>)> {
    let Some(root) = std::env::var_os(DISCS_VARIABLE) else {
        eprintln!("{DISCS_VARIABLE} is not set: the discs not run");
        return Vec::new();
    };
    let mut out = Vec::new();
    let mut dirs: Vec<_> = std::fs::read_dir(root).expect("the discs").flatten().map(|e| e.path()).collect();
    dirs.sort();
    for dir in dirs {
        let Some(cue) = std::fs::read_dir(&dir).expect("a disc").flatten().map(|e| e.path()).find(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("cue"))) else { continue };
        let sheet = std::fs::read(&cue).expect("the sheet");
        let parsed = emusen_disc::cue::Sheet::parse(&String::from_utf8_lossy(&sheet)).expect("a sheet");
        let files = parsed.files.iter().map(|f| std::fs::read(dir.join(f)).expect("a track")).collect();
        out.push((dir.file_name().unwrap().to_string_lossy().into_owned(), sheet, files));
    }
    out
}

/// The drive's exchanges until its status carries the report asked for.
fn ask(c: &mut Cdd, nibbles: [u8; 9]) -> [u8; 10] {
    let mut n = [0u8; 10];
    n[..9].copy_from_slice(&nibbles);
    n[9] = !n[..9].iter().fold(0u8, |a, &b| a.wrapping_add(b)) & 15;
    for k in 0..5 {
        c.write(0x0C + 2 * k as u32, (n[2 * k] as u16) << 8 | n[2 * k + 1] as u16, 0xFFFF, 0);
    }
    for _ in 0..200 {
        c.exchange();
        if c.status[1] == nibbles[3] {
            return c.status;
        }
    }
    panic!("no report {:X}", nibbles[3]);
}

#[test]
fn each_disc_opens_and_the_drive_reads_it() {
    for (name, sheet, files) in discs() {
        let mut disc = crate::mcd::open_disc(Kept::from(sheet), files.into_iter().map(Kept::from).collect()).unwrap_or_else(|e| panic!("{name}: {e}"));
        let area = crate::mcd::system_area(&mut disc);
        let media = crate::media::Media::read(&area);
        assert_eq!(media.system, crate::media::System::Mcd, "{name}: SEGADISCSYSTEM at 00:02:00");
        let toc = disc.toc();
        let (first, last, lead_out) = (toc.first, toc.last, toc.lead_out);
        let audio = disc.tracks().iter().filter(|t| t.audio).count();
        assert!(!disc.tracks()[0].audio && audio + 1 == disc.tracks().len(), "{name}: a data track, then audio");
        let mut c = Cdd::new(Some(disc), 0);
        c.set_hock(true);
        for _ in 0..SPIN_UP {
            c.exchange();
        }
        let r = ask(&mut c, [2, 0, 0, 4, 0, 0, 0, 0, 0]);
        assert_eq!(c.status[0], status::READY);
        assert_eq!((r[2] * 10 + r[3], r[4] * 10 + r[5]), (first, last), "{name}: first and last tracks");
        let r = ask(&mut c, [2, 0, 0, 3, 0, 0, 0, 0, 0]);
        let len = emusen_disc::Msf::from_bcd([r[2] << 4 | r[3], r[4] << 4 | r[5], r[6] << 4 | r[7]]).unwrap().sector();
        assert_eq!(len, lead_out, "{name}: the disc's length");
        let t2 = c.disc.as_ref().unwrap().tracks()[1].start;
        let r = ask(&mut c, [2, 0, 0, 5, 0, 2, 0, 0, 0]);
        let at = emusen_disc::Msf::from_bcd([r[2] << 4 | r[3], r[4] << 4 | r[5], (r[6] & 7) << 4 | r[7]]).unwrap().sector();
        assert_eq!(at, t2, "{name}: track 2's start");
        eprintln!("{name}: tracks {first}-{last}, {audio} audio, lead-out {lead_out}, track 2 at {t2}, title {:?}", media.header.map(|h| h.overseas_title).unwrap_or_default());
    }
}
