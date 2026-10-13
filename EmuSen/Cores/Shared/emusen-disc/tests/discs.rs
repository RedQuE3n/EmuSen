//! Synthetic discs built here, read back sector for sector: a two-file dump (mode 2 data, then audio with a stored
//! pause), the same as one file with an unstored PREGAP, an `.iso`, a lone raw track, subchannel Q and the TOC, an
//! `.sbi`, and (with chdman named) the same sheets as CHDs. EmuSen_Disc.md §6 lists what each shows, and §9.4 the
//! cases since 1.3: every form read from borrowed bytes as from owned ones, `open`, and the fingerprint; §10.3 the
//! place of a sector, since 1.4.

use std::sync::Arc;

use emusen_disc::cue::Sheet;
use emusen_disc::ecma130::{self, Form, RAW};
use emusen_disc::{Disc, IsoForm, Lone, Msf, Place, Refusal, Sector, open, samples};

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
        let packed = std::fs::read(&chd).unwrap();
        let mut from_chd = Disc::from_image(packed.clone(), IsoForm::Mode1).unwrap();
        same(&mut from_chd, &mut Disc::from_image(&packed[..], IsoForm::Mode1).unwrap(), name);
        same(&mut from_chd, &mut open(&packed[..], Vec::new(), IsoForm::Mode1, Lone::Detect).unwrap(), name);
        assert_eq!(from_chd.toc(), from_sheet.toc(), "{name}");
        let (mut x, mut y) = ([0u8; RAW], [0u8; RAW]);
        for at in 0..from_sheet.lead_out() + 10 {
            assert_eq!(from_chd.read(at, &mut x), from_sheet.read(at, &mut y), "{name} sector {at}");
            assert_eq!(x, y, "{name} sector {at}");
            assert_eq!(from_chd.subq(at), from_sheet.subq(at), "{name} sector {at}");
        }
        // Since 1.4: a sector's place is its stored hunk's first byte in the file. Shown without reading the container's
        // source: with the sixteen bytes there changed the sector no longer reads as it did (the decoder may refuse
        // the hunk, or stop on it: chd 0.3.4 indexes past a slice on some, which is caught here and counts), and
        // every sector whose hunk is stored elsewhere still does; a sector the image does not hold has no place.
        let mut places: Vec<u64> = Vec::new();
        for at in 0..from_sheet.lead_out() + 10 {
            let place = from_chd.place(at);
            assert!(place.is_none_or(|p| p.file.is_none() && stored(from_chd.read(at, &mut x))), "{name} sector {at}: {place:?}");
            places.extend(place.map(|p| p.offset).filter(|o| !places.contains(o)));
        }
        assert!(!places.is_empty(), "{name}: no hunk of its own");
        for &offset in &places {
            let mut changed = packed.clone();
            changed[offset as usize..offset as usize + 16].iter_mut().for_each(|b| *b ^= 0xFF);
            let Ok(mut broken) = Disc::from_image(changed, IsoForm::Mode1) else { continue };
            for at in 0..from_sheet.lead_out() {
                let was = from_chd.read(at, &mut x);
                let now = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| broken.read(at, &mut y))).ok();
                match from_chd.place(at) {
                    Some(p) if p.offset == offset => assert!(Some(was) != now || x != y, "{name} sector {at} reads the same without its hunk's bytes at {offset}"),
                    Some(_) => assert!(Some(was) == now && x == y, "{name} sector {at} changed with another hunk's bytes at {offset}"),
                    None => {}
                }
            }
        }
    }
    std::fs::remove_dir_all(&dir).ok();
}

/// Two discs that must be one: the tracks, the TOC, the lead-in's items, and every sector's kind, bytes and Q from
/// the first pause through the lead-out's start.
fn same<A: AsRef<[u8]>, B: AsRef<[u8]>>(a: &mut Disc<A>, b: &mut Disc<B>, what: &str) {
    assert_eq!(a.tracks(), b.tracks(), "{what}");
    assert_eq!((a.toc(), a.lead_out()), (b.toc(), b.lead_out()), "{what}");
    for pointer in (1..=3).chain([0xA0, 0xA1, 0xA2]) {
        assert_eq!(a.lead_in(pointer), b.lead_in(pointer), "{what} pointer {pointer:02X}");
    }
    let (mut x, mut y) = ([0u8; RAW], [1u8; RAW]);
    for at in 0..a.lead_out() + 10 {
        assert_eq!(a.read(at, &mut x), b.read(at, &mut y), "{what} sector {at}");
        assert_eq!(x, y, "{what} sector {at}");
        assert_eq!(a.subq(at), b.subq(at), "{what} sector {at}");
    }
    assert_eq!(a.fingerprint([0, 4, 16]), b.fingerprint([0, 4, 16]), "{what}");
}

/// A sheet's disc over each backing: its files owned, borrowed for the call, and shared as a host's lent bytes are.
fn sheet_over_every_backing(sheet: &str, files: &[Vec<u8>], iso: IsoForm, what: &str) {
    let sheet = Sheet::parse(sheet).unwrap();
    let mut owned = Disc::from_sheet(&sheet, files.to_vec(), iso).unwrap();
    let mut borrowed = Disc::from_sheet(&sheet, files.iter().map(Vec::as_slice).collect(), iso).unwrap();
    let mut shared = Disc::from_sheet(&sheet, files.iter().map(|f| Arc::<[u8]>::from(&f[..])).collect(), iso).unwrap();
    same(&mut owned, &mut borrowed, what);
    same(&mut owned, &mut shared, what);
}

fn iso_image() -> Vec<u8> {
    let mut iso = vec![0u8; 24 * 2048];
    iso[16 * 2048 + 1..16 * 2048 + 6].copy_from_slice(b"CD001");
    for (i, b) in iso.iter_mut().enumerate().skip(17 * 2048) {
        *b = (i * 7 + i / 2048) as u8;
    }
    iso
}

const SBI: [u8; 11] = [b'S', b'B', b'I', 0, 0x00, 0x02, 0x05, 3, 0x00, 0x02, 0x09];

/// The exactness rule at this crate's level: a disc opened from borrowed bytes reads sector for sector as one opened
/// from owned bytes, for every form the crate has.
#[test]
fn every_form_reads_the_same_from_borrowed_bytes_as_from_owned() {
    let (d, a) = files();
    sheet_over_every_backing(TWO_FILES, &[d.clone(), a.clone()], IsoForm::Mode2Form1, "a sheet of two files");
    let three = "FILE \"d.bin\" BINARY\nTRACK 01 MODE2/2352\nINDEX 01 00:00:00\nFILE \"a.bin\" BINARY\nTRACK 02 AUDIO\nFLAGS PRE\nINDEX 00 00:00:00\nINDEX 01 00:00:07\nFILE \"b.bin\" BINARY\nTRACK 03 AUDIO\nPREGAP 00:00:05\nINDEX 01 00:00:00\nINDEX 02 00:00:09\nPOSTGAP 00:00:02\n";
    sheet_over_every_backing(three, &[d.clone(), a.clone(), a[..20 * RAW].to_vec()], IsoForm::Mode2Form1, "a sheet of three files");
    let gap = format!("FILE \"x.bin\" BINARY\nTRACK 01 MODE2/2352\nINDEX 01 00:00:00\nTRACK 02 AUDIO\nPREGAP 00:00:{PAUSE:02}\nINDEX 01 00:00:{DATA:02}\n");
    sheet_over_every_backing(&gap, &[[d.clone(), a.clone()].concat()], IsoForm::Mode2Form1, "one file with an unstored pregap");
    let cooked = "FILE \"i.iso\" BINARY\nTRACK 01 MODE1/2048\nINDEX 01 00:00:00\nFILE \"a.bin\" BINARY\nTRACK 02 AUDIO\nINDEX 01 00:00:00\n";
    for iso in [IsoForm::Mode1, IsoForm::Mode2Form1] {
        sheet_over_every_backing(cooked, &[iso_image(), a.clone()], iso, "a sheet of user sectors and audio");
    }
    let xa: Vec<u8> = d.chunks_exact(RAW).flat_map(|s| s[16..].to_vec()).collect();
    sheet_over_every_backing("FILE \"m.bin\" BINARY\nTRACK 01 MODE2/2336\nINDEX 01 00:00:00\n", &[xa], IsoForm::Mode2Form1, "a sheet of 2,336-byte sectors");

    let user: Vec<u8> = (0..9 * 2048u32).map(|i| (i * 13) as u8).collect();
    let lone: [(&str, Vec<u8>, bool); 4] = [("a lone raw track", d.clone(), false), ("a lone audio track", a.clone(), false), ("an .iso", iso_image(), false), ("user sectors with no volume descriptor", user, true)];
    for (what, image, user) in lone {
        for iso in [IsoForm::Mode1, IsoForm::Mode2Form1] {
            let from = |bytes| if user { Disc::from_user_image(bytes, iso) } else { Disc::from_image(bytes, iso) };
            let mut owned = from(image.clone()).unwrap();
            let borrow = |bytes| if user { Disc::from_user_image(bytes, iso) } else { Disc::from_image(bytes, iso) };
            same(&mut owned, &mut borrow(&image[..]).unwrap(), what);
            let share = |bytes| if user { Disc::from_user_image(bytes, iso) } else { Disc::from_image(bytes, iso) };
            same(&mut owned, &mut share(Arc::<[u8]>::from(&image[..])).unwrap(), what);
            let how = if user { Lone::User } else { Lone::Detect };
            same(&mut owned, &mut open(&image[..], Vec::new(), iso, how).unwrap(), what);
        }
    }

    let mut owned = two_files();
    owned.apply_sbi(&SBI).unwrap();
    let mut borrowed = open(TWO_FILES.as_bytes(), vec![&d[..], &a[..], &SBI[..]], IsoForm::Mode2Form1, Lone::Detect).unwrap();
    assert!(!borrowed.subq(155).valid());
    same(&mut owned, &mut borrowed, "a sheet with its .sbi");
}

/// `open` as a host's call gives it: the sheet's files from the first companions, an `.sbi` past them, and each
/// refusal with its words.
#[test]
fn an_image_and_its_companions_open_or_are_refused_with_words() {
    let (d, a) = files();
    let sheet = TWO_FILES.as_bytes().to_vec();
    let mut disc = open(sheet.clone(), vec![d.clone(), a.clone()], IsoForm::Mode2Form1, Lone::Detect).unwrap();
    same(&mut two_files(), &mut disc, "a sheet opened");
    assert!(disc.subq(155).valid(), "no .sbi was given");
    assert_eq!(open(sheet.clone(), vec![d.clone()], IsoForm::Mode2Form1, Lone::Detect).err(), Some(Refusal::Missing { needs: 2, has: 1 }));
    assert_eq!(open(sheet.clone(), vec![SBI.to_vec()], IsoForm::Mode2Form1, Lone::Detect).err(), Some(Refusal::Missing { needs: 2, has: 1 }), "an .sbi is no track");
    let short = open(sheet.clone(), vec![d.clone(), a[..3 * RAW].to_vec()], IsoForm::Mode2Form1, Lone::Detect).err().unwrap();
    assert!(matches!(&short, Refusal::Sheet(w) if w.contains("too short")), "{short}");
    let bad = open(b"FILE \"a.bin\" BINARY\nTRACK 01 NONSENSE\n".to_vec(), vec![d.clone()], IsoForm::Mode1, Lone::Detect).err().unwrap();
    assert!(matches!(bad, Refusal::Sheet(_)), "{bad}");
    let unknown = open(vec![1u8; 1000], Vec::new(), IsoForm::Mode1, Lone::Detect).err().unwrap();
    assert!(matches!(&unknown, Refusal::Image(w) if w.contains("1000 bytes")), "{unknown}");
    assert!(open(vec![1u8; 5000], Vec::new(), IsoForm::Mode1, Lone::User).is_ok(), "the console's word that they are user sectors is taken");
    let torn = open(sheet, vec![d, a, b"SBI\0\x00\x02".to_vec()], IsoForm::Mode2Form1, Lone::Detect).err().unwrap();
    assert!(matches!(torn, Refusal::Sbi(_)), "{torn}");
}

/// Whether a sector is one the image holds.
fn stored(kind: Sector) -> bool {
    matches!(kind, Sector::Data { synthesised: false, .. } | Sector::Audio { synthesised: false })
}

/// Every sector's place on a disc `make` opens from `files`: a sector the image holds names the file and the byte
/// where it begins, shown by changing that byte, which changes what the sector reads as, and the byte before it,
/// which does not; a sector the image does not hold names nothing. `lone` is whether the one file is the image.
fn every_sector_is_placed(what: &str, files: &[Vec<u8>], lone: bool, make: &dyn Fn(Vec<Vec<u8>>) -> Option<Disc>) {
    let mut disc = make(files.to_vec()).unwrap();
    let (mut raw, mut other) = ([0u8; RAW], [0u8; RAW]);
    let mut placed = 0;
    for at in 0..disc.lead_out() + 10 {
        let kind = disc.read(at, &mut raw);
        let Some(Place { file, offset }) = disc.place(at) else {
            assert!(!stored(kind), "{what}: sector {at} is read from the image and has no place");
            continue;
        };
        assert!(stored(kind), "{what}: sector {at} has a place and is not in the image");
        assert_eq!(file.is_none(), lone, "{what}: sector {at}'s file");
        let (index, offset) = (file.unwrap_or(0), offset as usize);
        let reads_after = |change: usize, other: &mut [u8; RAW]| {
            let mut changed = files.to_vec();
            changed[index][change] ^= 0xFF;
            make(changed).map(|mut d| d.read(at, other))
        };
        assert!(reads_after(offset, &mut other) != Some(kind) || other != raw, "{what}: sector {at} does not begin at byte {offset} of file {index}");
        if offset > 0 && at > 0 && disc.place(at - 1).is_some() {
            assert!(reads_after(offset - 1, &mut other) == Some(kind) && other == raw, "{what}: sector {at} begins before byte {offset} of file {index}");
        }
        placed += 1;
    }
    assert!(placed > 0, "{what}");
}

/// Since 1.4: which file and byte hold a sector, for every form the crate reads from files.
#[test]
fn a_sector_is_placed_in_the_file_and_at_the_byte_that_hold_it() {
    let (d, a) = files();
    let sheet = |text: &'static str, iso: IsoForm| move |files: Vec<Vec<u8>>| Disc::from_sheet(&Sheet::parse(text).unwrap(), files, iso).ok();
    every_sector_is_placed("a sheet of two files", &[d.clone(), a.clone()], false, &sheet(TWO_FILES, IsoForm::Mode2Form1));
    let three = "FILE \"d.bin\" BINARY\nTRACK 01 MODE2/2352\nINDEX 01 00:00:00\nFILE \"a.bin\" BINARY\nTRACK 02 AUDIO\nFLAGS PRE\nINDEX 00 00:00:00\nINDEX 01 00:00:07\nFILE \"b.bin\" BINARY\nTRACK 03 AUDIO\nPREGAP 00:00:05\nINDEX 01 00:00:00\nINDEX 02 00:00:09\nPOSTGAP 00:00:02\n";
    every_sector_is_placed("a sheet of three files", &[d.clone(), a.clone(), a[..20 * RAW].to_vec()], false, &sheet(three, IsoForm::Mode2Form1));
    let gap = "FILE \"x.bin\" BINARY\nTRACK 01 MODE2/2352\nINDEX 01 00:00:00\nTRACK 02 AUDIO\nPREGAP 00:00:07\nINDEX 01 00:00:40\n";
    every_sector_is_placed("one file with an unstored pregap", &[[d.clone(), a.clone()].concat()], false, &sheet(gap, IsoForm::Mode2Form1));
    let cooked = "FILE \"i.iso\" BINARY\nTRACK 01 MODE1/2048\nINDEX 01 00:00:00\nFILE \"a.bin\" BINARY\nTRACK 02 AUDIO\nINDEX 01 00:00:00\n";
    every_sector_is_placed("a sheet of user sectors and audio", &[iso_image(), a.clone()], false, &sheet(cooked, IsoForm::Mode2Form1));
    let xa: Vec<u8> = d.chunks_exact(RAW).flat_map(|s| s[16..].to_vec()).collect();
    every_sector_is_placed("a sheet of 2,336-byte sectors", &[xa], false, &sheet("FILE \"m.bin\" BINARY\nTRACK 01 MODE2/2336\nINDEX 01 00:00:00\n", IsoForm::Mode2Form1));
    let lone = |files: Vec<Vec<u8>>| Disc::from_image(files.into_iter().next().unwrap(), IsoForm::Mode2Form1).ok();
    every_sector_is_placed("a lone raw track", &[d.clone()], true, &lone);
    every_sector_is_placed("a lone audio track", &[a.clone()], true, &lone);
    every_sector_is_placed("an .iso", &[iso_image()], true, &lone);
    let user: Vec<u8> = (0..9 * 2048u32).map(|i| (i * 13) as u8).collect();
    every_sector_is_placed("user sectors with no volume descriptor", &[user], true, &|files: Vec<Vec<u8>>| Disc::from_user_image(files.into_iter().next().unwrap(), IsoForm::Mode1).ok());

    // The numbers themselves, on the sheet of two files: the data track's sector n at byte 2,352 n of the first file,
    // the audio track's from its stored pause on in the second, nothing before 00:02:00 or in the lead-out.
    let disc = two_files();
    assert_eq!(disc.place(149), None);
    assert_eq!(disc.place(150 + 7), Some(Place { file: Some(0), offset: 7 * RAW as u64 }));
    assert_eq!(disc.place(150 + DATA), Some(Place { file: Some(1), offset: 0 }));
    assert_eq!(disc.place(150 + DATA + AUDIO - 1), Some(Place { file: Some(1), offset: (AUDIO as u64 - 1) * RAW as u64 }));
    assert_eq!(disc.place(disc.lead_out()), None);
    let borrowed = open(TWO_FILES.as_bytes(), vec![&d[..], &a[..]], IsoForm::Mode2Form1, Lone::Detect).unwrap();
    assert!((0..disc.lead_out() + 10).all(|at| borrowed.place(at) == disc.place(at)), "a place is the same over any backing");
}

/// The fingerprint a state is held to: the value Diamond's states have recorded since DMND 9, moved only by the
/// tracks, the lead-out and the sectors named, and so reading nothing else of the disc.
#[test]
fn the_fingerprint_is_the_tracks_the_lead_out_and_the_sectors_named() {
    let diamond = || std::iter::once(4).chain(16..24);
    let (d, a) = files();
    let mut disc = two_files();
    let print = disc.fingerprint(diamond());
    let mut h = 0xCBF2_9CE4_8422_2325u64;
    let mut eat = |bytes: &[u8]| bytes.iter().for_each(|&b| h = (h ^ b as u64).wrapping_mul(0x0100_0000_01B3));
    for (start, audio) in [(150u32, 0u8), (150 + DATA + PAUSE, 1)] {
        eat(&start.to_le_bytes());
        eat(&[audio]);
    }
    eat(&(150 + DATA + AUDIO).to_le_bytes());
    for i in diamond() {
        eat(&data_sector(i));
    }
    assert_eq!(print, h, "FNV-1a as medium.rs computed it before 1.3");
    assert_eq!(print, 0x1AE5_19C0_EA40_70CD, "pinned: a state made before 1.3 names its disc by this");

    let mut elsewhere = d.clone();
    for (i, b) in elsewhere.iter_mut().enumerate() {
        let sector = (i / RAW) as u32;
        if sector != 4 && !(16..24).contains(&sector) {
            *b ^= 0xA5;
        }
    }
    let sheet = Sheet::parse(TWO_FILES).unwrap();
    let silent = vec![0u8; a.len()];
    let mut other = Disc::from_sheet(&sheet, vec![&elsewhere[..], &silent[..]], IsoForm::Mode2Form1).unwrap();
    assert_eq!(other.fingerprint(diamond()), print, "no byte outside the named sectors is read into it");
    let mut one = d.clone();
    one[20 * RAW + 1000] ^= 1;
    assert_ne!(Disc::from_sheet(&sheet, vec![&one[..], &a[..]], IsoForm::Mode2Form1).unwrap().fingerprint(diamond()), print);
    assert_ne!(Disc::from_sheet(&sheet, vec![&d[..], &a[..(AUDIO as usize - 1) * RAW]], IsoForm::Mode2Form1).unwrap().fingerprint(diamond()), print, "the lead-out moved");
    assert_ne!(disc.fingerprint([4]), print);
    assert_ne!(Disc::<&[u8]>::from_image(&[0u8; RAW][..], IsoForm::Mode1).unwrap().fingerprint([]), 0, "never the empty drive's 0");
}
