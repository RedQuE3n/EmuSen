//! Synthetic discs built here, read back sector for sector: a two-file dump (mode 2 data, then audio with a stored
//! pause), the same as one file with an unstored PREGAP, an `.iso`, a lone raw track, subchannel Q and the TOC, an
//! `.sbi`, and (with chdman named) the same sheets as CHDs. EmuSen_Disc.md §6 lists what each shows.

use emusen_disc::cue::Sheet;
use emusen_disc::ecma130::{self, Form, RAW};
use emusen_disc::{Disc, IsoForm, Msf, Sector, samples};

const DATA: u32 = 40;
const AUDIO: u32 = 30;
const PAUSE: u32 = 7;

/// Data sector `i`: a mode 2 form 1 sector at 00:02:00 + i whose user data names it.
fn data_sector(i: u32) -> [u8; RAW] {
    let user: Vec<u8> = (0..2048u32).map(|k| (k * 3 + i * 11) as u8).collect();
    let mut s = [0u8; RAW];
    ecma130::mode2_form1(150 + i, [1, 0, ecma130::submode::DATA, 0], &user, &mut s);
    s
}

/// Audio sector `i` of the second track's file: frame k holds (i·1000 + k, −(i·1000 + k)).
fn audio_sector(i: u32) -> [u8; RAW] {
    let mut s = [0u8; RAW];
    for k in 0..588 {
        let v = (i * 1000 + k) as i16;
        s[4 * k as usize..4 * k as usize + 2].copy_from_slice(&v.to_le_bytes());
        s[4 * k as usize + 2..4 * k as usize + 4].copy_from_slice(&(-v).to_le_bytes());
    }
    s
}

fn files() -> (Vec<u8>, Vec<u8>) {
    ((0..DATA).flat_map(data_sector).collect(), (0..AUDIO).flat_map(audio_sector).collect())
}

const TWO_FILES: &str = "FILE \"d.bin\" BINARY\n  TRACK 01 MODE2/2352\n    INDEX 01 00:00:00\nFILE \"a.bin\" BINARY\n  TRACK 02 AUDIO\n    INDEX 00 00:00:00\n    INDEX 01 00:00:07\n";

fn two_files() -> Disc {
    let (d, a) = files();
    Disc::from_sheet(&Sheet::parse(TWO_FILES).unwrap(), vec![d, a], IsoForm::Mode2Form1).unwrap()
}

#[test]
fn a_two_file_dump_is_placed_and_read_sector_for_sector() {
    let mut disc = two_files();
    let t = disc.tracks().to_vec();
    assert_eq!((t[0].pregap, t[0].start, t[0].end), (0, 150, 150 + DATA));
    assert_eq!((t[1].pregap, t[1].start, t[1].end), (150 + DATA, 150 + DATA + PAUSE, 150 + DATA + AUDIO));
    assert_eq!(disc.lead_out(), 150 + DATA + AUDIO);
    let mut raw = [0u8; RAW];
    for i in 0..DATA {
        assert_eq!(disc.read(150 + i, &mut raw), Sector::Data { mode: 2, synthesised: false });
        assert_eq!(raw, data_sector(i), "data sector {i}");
    }
    for i in 0..AUDIO {
        assert_eq!(disc.read(150 + DATA + i, &mut raw), Sector::Audio { synthesised: false });
        assert_eq!(raw, audio_sector(i), "audio sector {i}");
    }
    let s: Vec<(i16, i16)> = samples(&raw).take(2).collect();
    assert_eq!(s, [(29_000, -29_000), (29_001, -29_001)]);
    assert_eq!(disc.read(100, &mut raw), Sector::Data { mode: 2, synthesised: true }, "the first pause, not in the file");
    assert!(ecma130::edc_holds(&raw) && ecma130::form(&raw) == Form::Form1 && raw[12..15] == Msf::of(100).bcd());
    assert_eq!(disc.read(disc.lead_out() + 5, &mut raw), Sector::Audio { synthesised: true }, "the lead-out continues the last track's kind");
}

#[test]
fn subchannel_q_counts_the_pause_down_and_the_lead_out_up() {
    let disc = two_files();
    let q = disc.subq(143);
    assert!(q.valid());
    assert_eq!(q.getloc(), [0x01, 0x00, 0x00, 0x00, 0x06, 0x00, 0x01, 0x68], "the console's getloc log: 00:01:68 is 00:00:06 before index 01");
    assert_eq!(disc.subq(161).getloc(), [0x01, 0x01, 0x00, 0x00, 0x11, 0x00, 0x02, 0x11]);
    assert_eq!(disc.subq(150 + DATA).getloc()[..5], [0x02, 0x00, 0x00, 0x00, 0x06]);
    assert_eq!(disc.subq(150 + DATA + PAUSE).getloc()[..5], [0x02, 0x01, 0x00, 0x00, 0x00]);
    let lo = disc.lead_out();
    assert_eq!(disc.subq(lo + 3).getloc()[..5], [0xAA, 0x01, 0x00, 0x00, 0x03]);
    assert_eq!(disc.subq(150).0[0], 0x41, "data, q-mode 1");
    assert_eq!(disc.subq(150 + DATA + 1).0[0], 0x01, "audio");
}

#[test]
fn the_toc_and_the_lead_in() {
    let disc = two_files();
    let toc = disc.toc();
    assert_eq!((toc.first, toc.last, toc.disc_type, toc.lead_out), (1, 2, 0x20, 150 + DATA + AUDIO));
    assert_eq!(toc.tracks, vec![(1, 4, 150), (2, 0, 150 + DATA + PAUSE)]);
    assert_eq!(disc.lead_in(0xA2).unwrap().0[7..10], Msf::of(150 + DATA + AUDIO).bcd());
    assert_eq!(disc.lead_in(0x02).unwrap().0[7..10], Msf::of(150 + DATA + PAUSE).bcd());
    assert!(disc.lead_in(0x03).is_none());
}

/// psx-spx's rule for PREGAP: the gap is not stored, and every later address moves by it.
#[test]
fn one_file_with_an_unstored_pregap() {
    let (d, a) = files();
    let one = [d, a.clone()].concat();
    let sheet = format!("FILE \"x.bin\" BINARY\nTRACK 01 MODE2/2352\nINDEX 01 00:00:00\nTRACK 02 AUDIO\nPREGAP 00:00:{PAUSE:02}\nINDEX 01 00:00:{DATA:02}\n");
    let mut disc = Disc::from_sheet(&Sheet::parse(&sheet).unwrap(), vec![one], IsoForm::Mode2Form1).unwrap();
    let t = disc.tracks()[1].clone();
    assert_eq!((t.pregap, t.start), (150 + DATA, 150 + DATA + PAUSE));
    let mut raw = [0u8; RAW];
    assert_eq!(disc.read(150 + DATA + 2, &mut raw), Sector::Audio { synthesised: true });
    assert_eq!(disc.read(150 + DATA + PAUSE, &mut raw), Sector::Audio { synthesised: false });
    assert_eq!(raw, audio_sector(0));
    assert_eq!(disc.lead_out(), 150 + DATA + PAUSE + AUDIO);
}

#[test]
fn an_iso_is_framed_as_the_console_asks_and_meets_ecma_130() {
    let mut iso = vec![0u8; 20 * 2048];
    iso[16 * 2048 + 1..16 * 2048 + 6].copy_from_slice(b"CD001");
    for (i, b) in iso.iter_mut().enumerate().skip(17 * 2048) {
        *b = (i * 5) as u8;
    }
    let mut raw = [0u8; RAW];
    let mut xa = Disc::from_image(iso.clone(), IsoForm::Mode2Form1).unwrap();
    for n in 0..20u32 {
        assert_eq!(xa.read(150 + n, &mut raw), Sector::Data { mode: 2, synthesised: false });
        assert!(ecma130::edc_holds(&raw));
        let mut zero_header = raw;
        zero_header[12..16].fill(0);
        assert!(ecma130::parity_holds(&zero_header));
        assert_eq!(&raw[24..2072], &iso[n as usize * 2048..(n as usize + 1) * 2048]);
        assert_eq!(raw[18], ecma130::submode::DATA);
    }
    let mut m1 = Disc::from_image(iso.clone(), IsoForm::Mode1).unwrap();
    m1.read(150 + 17, &mut raw);
    assert!(ecma130::edc_holds(&raw) && ecma130::parity_holds(&raw) && raw[15] == 1);
    assert_eq!(&raw[16..2064], &iso[17 * 2048..18 * 2048]);
    assert_eq!((xa.toc().disc_type, m1.toc().disc_type), (0x20, 0));
}

/// A lone track's `.bin` with no sheet, as two-thirds of the tester's library is: one track, its mode from its header.
#[test]
fn a_lone_raw_track_is_a_disc_of_one_track() {
    let (d, a) = files();
    let mut disc = Disc::from_image(d, IsoForm::Mode2Form1).unwrap();
    assert_eq!((disc.tracks().len(), disc.tracks()[0].start, disc.lead_out()), (1, 150, 150 + DATA));
    let mut raw = [0u8; RAW];
    disc.read(155, &mut raw);
    assert_eq!(raw, data_sector(5));
    let audio = Disc::from_image(a, IsoForm::Mode2Form1).unwrap();
    assert!(audio.tracks()[0].audio);
    assert!(Disc::from_image(vec![1; 1000], IsoForm::Mode1).is_err());
}

#[test]
fn an_sbi_marks_its_sectors_q_bad() {
    let mut disc = two_files();
    let mut sbi = b"SBI\0".to_vec();
    sbi.extend_from_slice(&[0x00, 0x02, 0x05, 3, 0x00, 0x02, 0x09]);
    disc.apply_sbi(&sbi).unwrap();
    let q = disc.subq(155);
    assert!(!q.valid());
    assert_eq!(q.0[7..10], [0x00, 0x02, 0x09]);
    assert!(disc.subq(154).valid() && disc.subq(156).valid());
}

/// The same sheets made into CHDs by chdman, run as a tool: every sector read from the CHD equals the sheet's. Opt in
/// with `EMUSEN_DISC_CHDMAN` naming the chdman binary.
#[test]
fn a_chd_reads_as_its_sheet() {
    let Ok(chdman) = std::env::var("EMUSEN_DISC_CHDMAN") else {
        eprintln!("skipped: EMUSEN_DISC_CHDMAN names no chdman");
        return;
    };
    let dir = std::env::temp_dir().join(format!("emusen-disc-chd-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let (d, a) = files();
    std::fs::write(dir.join("d.bin"), &d).unwrap();
    std::fs::write(dir.join("a.bin"), &a).unwrap();
    let mut iso = vec![0u8; 12 * 2048];
    iso[16 * 2048 - 4 * 2048..].iter_mut().enumerate().for_each(|(i, b)| *b = i as u8);
    std::fs::write(dir.join("i.iso"), &iso).unwrap();
    let sheets = [
        ("two.cue", TWO_FILES.to_owned(), vec![d.clone(), a.clone()]),
        ("gap.cue", format!("FILE \"d.bin\" BINARY\nTRACK 01 MODE2/2352\nINDEX 01 00:00:00\nFILE \"a.bin\" BINARY\nTRACK 02 AUDIO\nPREGAP 00:00:03\nINDEX 01 00:00:00\n"), vec![d, a]),
        ("iso.cue", "FILE \"i.iso\" BINARY\nTRACK 01 MODE1/2048\nINDEX 01 00:00:00\n".to_owned(), vec![iso]),
    ];
    for (name, text, bytes) in sheets {
        std::fs::write(dir.join(name), &text).unwrap();
        let chd = dir.join(name.replace(".cue", ".chd"));
        let run = std::process::Command::new(&chdman).args(["createcd", "-f", "-i"]).arg(dir.join(name)).arg("-o").arg(&chd).output().unwrap();
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
        let mut from_sheet = Disc::from_sheet(&Sheet::parse(&text).unwrap(), bytes, IsoForm::Mode1).unwrap();
        let mut from_chd = Disc::from_image(std::fs::read(&chd).unwrap(), IsoForm::Mode1).unwrap();
        assert_eq!(from_chd.toc(), from_sheet.toc(), "{name}");
        let (mut x, mut y) = ([0u8; RAW], [0u8; RAW]);
        for at in 0..from_sheet.lead_out() + 10 {
            assert_eq!(from_chd.read(at, &mut x), from_sheet.read(at, &mut y), "{name} sector {at}");
            assert_eq!(x, y, "{name} sector {at}");
            assert_eq!(from_chd.subq(at), from_sheet.subq(at), "{name} sector {at}");
        }
    }
    std::fs::remove_dir_all(&dir).ok();
}
