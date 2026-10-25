//! Synthetic DVDs built here, read back sector for sector (EmuSen_Disc.md §14.6): a single-layer image, a dual-layer one
//! whose second volume places the layer break, images that place none, whole Data Frames, the medium chosen at open,
//! every form over borrowed bytes as over owned, and (with chdman named) the same images as DVD CHDs, clean and
//! damaged.

use std::path::PathBuf;
use std::process::Command;

use emusen_disc::dvd::CD_LIMIT;
use emusen_disc::ecma267::{self, FRAME, TrackPath, USER, Zone};
use emusen_disc::{Dvd, DvdSector, IsoForm, Kind, Lone, Medium, open_medium};

/// Sector `lba`'s Main Data: every byte names its sector and offset, so a sector read from elsewhere shows.
fn user(lba: u32) -> [u8; USER] {
    std::array::from_fn(|k| (k as u32 ^ lba.wrapping_mul(0x9E37) ^ (lba >> 8)) as u8)
}

/// A primary volume descriptor whose Volume Space Size is `blocks` (ECMA-119 §8.4, §8.4.8).
fn pvd(blocks: u32) -> [u8; USER] {
    let mut d = [0u8; USER];
    d[..6].copy_from_slice(b"\x01CD001");
    d[80..84].copy_from_slice(&blocks.to_le_bytes());
    d[84..88].copy_from_slice(&blocks.to_be_bytes());
    d
}

/// `sectors` logical sectors of `user`'s data, with a descriptor of `volume` blocks at sector 16 and, where asked, a
/// second at `second`.
fn image(sectors: u32, volume: u32, second: Option<u32>) -> Vec<u8> {
    let mut v: Vec<u8> = (0..sectors).flat_map(user).collect();
    v[16 * USER..17 * USER].copy_from_slice(&pvd(volume));
    if let Some(s) = second {
        v[s as usize * USER..(s as usize + 1) * USER].copy_from_slice(&pvd(sectors - (s - 16)));
    }
    v
}

fn data(lba: u32, img: &[u8]) -> &[u8] {
    &img[lba as usize * USER..(lba as usize + 1) * USER]
}

const LAYER0: u32 = 1024;
const TOTAL: u32 = 1100;

fn dual() -> Vec<u8> {
    image(TOTAL, LAYER0, Some(LAYER0 + 16))
}

#[test]
fn a_single_layer_image_reads_as_ecma_267s_frames() {
    let img = image(300, 300, None);
    let mut d = Dvd::from_image(img.clone()).unwrap();
    assert_eq!((d.sectors(), d.layers(), d.layer_break()), (300, 1, None));
    let mut f = [0u8; FRAME];
    for lba in 0..300 {
        let s = d.read(lba, &mut f);
        assert_eq!(s, DvdSector { layer: 0, psn: 0x03_0000 + lba, zone: Zone::Data, synthesised: true });
        assert_eq!(f[..4], [0x00, 0x03, (lba >> 8) as u8, lba as u8], "§16.1: Layer 0 of a single layer disk, the Data Zone, its number");
        assert!(ecma267::holds(&f), "sector {lba}: IED and EDC");
        assert_eq!(f[6..12], [0; 6], "§16.3: CPR_MAI's default");
        assert_eq!(&f[12..2060], data(lba, &img));
        let mut u = [0u8; USER];
        assert_eq!(d.read_user(lba, &mut u), DvdSector { synthesised: false, ..s });
        assert_eq!(&u[..], data(lba, &img));
    }
    let s = d.read(305, &mut f);
    assert_eq!((s.zone, s.psn, s.synthesised, f[0]), (Zone::LeadOut, 0x03_0000 + 305, true, 0x08), "§28: the Lead-out Zone past the end");
    assert!(f[12..2060].iter().all(|&b| b == 0) && ecma267::holds(&f));
}

#[test]
fn a_dual_layer_images_second_volume_places_the_layer_break() {
    let img = dual();
    let mut d = Dvd::from_image(img.clone()).unwrap();
    assert_eq!((d.sectors(), d.layers(), d.layer_break(), d.track_path()), (TOTAL, 2, Some(LAYER0), TrackPath::Opposite));
    let mut f = [0u8; FRAME];
    let s = d.read(LAYER0 - 1, &mut f);
    assert_eq!((s.layer, s.psn, f[0]), (0, 0x03_03FF, 0x20), "the last sector of layer 0, b29 set on a dual layer disk");
    let s = d.read(LAYER0, &mut f);
    assert_eq!((s.layer, s.psn, [f[0], f[1], f[2], f[3]]), (1, 0xFC_FC00, [0x21, 0xFC, 0xFC, 0x00]), "§25: layer 0's last, 0303FFh, inverted");
    assert!(ecma267::holds(&f));
    assert_eq!(&f[12..2060], data(LAYER0, &img));
    for lba in 0..TOTAL + 4 {
        let s = d.read(lba, &mut f);
        assert_eq!(s.layer, (lba >= LAYER0) as u8, "sector {lba}");
        assert_eq!(f[0] & 1, s.layer, "sector {lba}: the ID's layer bit");
        assert!(ecma267::holds(&f), "sector {lba}");
    }
    d.set_track_path(TrackPath::Parallel);
    assert_eq!(d.read(LAYER0 + 3, &mut f).psn, 0x03_0003, "PTP numbers layer 1 from (030000) again");
}

#[test]
fn an_image_whose_volumes_place_no_break_is_one_layer() {
    let cases = [
        ("no second descriptor", image(TOTAL, LAYER0, None)),
        ("a volume of 1,000, no multiple of 16", image(TOTAL, 1000, Some(1016))),
        ("the volume is the whole image", image(TOTAL, TOTAL, None)),
        ("a second descriptor past the volume's sector 16", image(TOTAL, LAYER0, Some(LAYER0 + 17))),
    ];
    for (what, img) in cases {
        let d = Dvd::from_image(img).unwrap();
        assert_eq!((d.layers(), d.layer_break()), (1, None), "{what}");
    }
}

#[test]
fn whole_data_frames_are_read_as_stored_and_told_by_their_first_id() {
    let mut synth = Dvd::from_image(dual()).unwrap();
    let mut frames = vec![0u8; TOTAL as usize * FRAME];
    for (lba, f) in frames.chunks_exact_mut(FRAME).enumerate() {
        synth.read(lba as u32, f.try_into().unwrap());
    }
    frames[5 * FRAME + 8] = 0x5A;
    let mut d = Dvd::from_image(&frames[..]).unwrap();
    assert_eq!((d.sectors(), d.layer_break()), (TOTAL, Some(LAYER0)));
    let mut f = [0u8; FRAME];
    for lba in 0..TOTAL {
        assert!(!d.read(lba, &mut f).synthesised);
        assert_eq!(&f[..], &frames[lba as usize * FRAME..(lba as usize + 1) * FRAME], "sector {lba} as stored, CPR_MAI and all");
    }
    let mut cut = frames.clone();
    cut[3] ^= 1;
    assert!(Dvd::from_image(&cut[..]).is_err(), "a first ID other than (030000) is no frame image");
    assert!(Dvd::from_image(&frames[..FRAME * 3 + 5]).is_err(), "nor is a length that is no multiple of 2,064");
}

fn same<B: AsRef<[u8]>, C: AsRef<[u8]>>(a: &mut Dvd<B>, b: &mut Dvd<C>, what: &str) {
    assert_eq!((a.sectors(), a.layer_break()), (b.sectors(), b.layer_break()), "{what}");
    let (mut x, mut y) = ([0u8; FRAME], [0u8; FRAME]);
    for lba in 0..a.sectors() + 3 {
        assert_eq!(a.read(lba, &mut x), b.read(lba, &mut y), "{what} sector {lba}");
        assert_eq!(x, y, "{what} sector {lba}");
    }
    assert_eq!(a.fingerprint([16, 17, 500]), b.fingerprint([16, 17, 500]), "{what}");
}

#[test]
fn every_form_reads_the_same_from_borrowed_bytes_as_from_owned() {
    let img = dual();
    let mut owned = Dvd::from_image(img.clone()).unwrap();
    same(&mut owned, &mut Dvd::from_image(&img[..]).unwrap(), "borrowed");
    same(&mut owned, &mut Dvd::from_image(std::sync::Arc::<[u8]>::from(img.clone())).unwrap(), "lent");
    same(&mut owned, &mut Dvd::from_user_image(&img[..]).unwrap(), "user sectors");
    for lba in [0, 16, LAYER0, TOTAL - 1] {
        assert_eq!(owned.place(lba).map(|p| (p.file, p.offset)), Some((None, lba as u64 * USER as u64)));
    }
    assert_eq!(owned.place(TOTAL), None, "the lead-out is not stored");
    let a = owned.fingerprint([16, 600]);
    let mut moved = img.clone();
    moved[600 * USER + 7] ^= 1;
    assert_ne!(a, Dvd::from_image(moved).unwrap().fingerprint([16, 600]));
    let mut outside = img;
    outside[601 * USER] ^= 1;
    assert_eq!(a, Dvd::from_image(outside).unwrap().fingerprint([16, 600]));
}

#[test]
fn the_medium_is_chosen_when_an_image_is_opened() {
    let kind = |image: &[u8], user: Kind, lone: Lone| open_medium(image, Vec::new(), IsoForm::Mode2Form1, lone, user).map(|m| m.kind());
    let iso = image(300, 300, None);
    assert_eq!(kind(&iso, Kind::Dvd, Lone::Detect), Ok(Kind::Dvd));
    assert_eq!(kind(&iso, Kind::Cd, Lone::Detect), Ok(Kind::Cd), "2,048-byte sectors are the console's word");
    let mut bare = iso.clone();
    bare[16 * USER] = 0;
    assert_eq!(kind(&bare, Kind::Dvd, Lone::User), Ok(Kind::Dvd), "no volume descriptor, on the console's word");
    assert!(kind(&bare, Kind::Dvd, Lone::Detect).is_err());
    let mut frames = vec![0u8; 20 * FRAME];
    let mut d = Dvd::from_image(iso.clone()).unwrap();
    for (lba, f) in frames.chunks_exact_mut(FRAME).enumerate() {
        d.read(lba as u32, f.try_into().unwrap());
    }
    assert_eq!(kind(&frames, Kind::Cd, Lone::Detect), Ok(Kind::Dvd), "whole frames are a DVD's whatever the console says");
    let mut raw = [0u8; 2352];
    emusen_disc::ecma130::mode2_form1(150, [0, 0, 8, 0], &[0; USER], &mut raw);
    assert_eq!(kind(&raw, Kind::Dvd, Lone::Detect), Ok(Kind::Cd), "a raw track is a compact disc's");
    let sheet = b"FILE \"a.bin\" BINARY\nTRACK 01 MODE2/2352\nINDEX 01 00:00:00\n";
    let m = open_medium(&sheet[..], vec![&raw[..]], IsoForm::Mode2Form1, Lone::Detect, Kind::Dvd).unwrap();
    assert!(matches!(m, Medium::Cd(_)));
    let mut big = vec![0u8; (CD_LIMIT as usize + 1) * USER];
    big[16 * USER..17 * USER].copy_from_slice(&pvd(CD_LIMIT + 1));
    assert_eq!(kind(&big, Kind::Cd, Lone::Detect), Ok(Kind::Dvd), "more sectors than a compact disc's time code names");
    big.truncate(CD_LIMIT as usize * USER);
    assert_eq!(kind(&big, Kind::Cd, Lone::Detect), Ok(Kind::Cd));
}

fn chdman() -> Option<String> {
    let c = std::env::var("EMUSEN_DISC_CHDMAN").ok();
    if c.is_none() {
        eprintln!("skipped: EMUSEN_DISC_CHDMAN names no chdman");
    }
    c
}

fn create_dvd(chdman: &str, iso: &[u8], name: &str, extra: &[&str]) -> Vec<u8> {
    let dir = std::env::temp_dir().join(format!("emusen-disc-dvd-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let (src, chd): (PathBuf, PathBuf) = (dir.join(format!("{name}.iso")), dir.join(format!("{name}.chd")));
    std::fs::write(&src, iso).unwrap();
    let run = Command::new(chdman).args(["createdvd", "-f", "-np", "2", "-i"]).arg(&src).arg("-o").arg(&chd).args(extra).output().unwrap();
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    let bytes = std::fs::read(&chd).unwrap();
    let _ = std::fs::remove_file(&src);
    let _ = std::fs::remove_file(&chd);
    bytes
}

/// The single- and dual-layer images made into DVD CHDs by chdman (`createdvd`, its default codecs and an
/// uncompressed one): every sector equal to the image's, the layer break found through the CHD, and places in its own
/// hunks. Opt in with `EMUSEN_DISC_CHDMAN`.
#[test]
fn a_dvd_chd_reads_as_its_image() {
    let Some(chdman) = chdman() else { return };
    for (name, img, extra) in [("single", image(300, 300, None), &[][..]), ("dual", dual(), &[][..]), ("plain", dual(), &["-c", "none"][..])] {
        let packed = create_dvd(&chdman, &img, name, extra);
        let mut from_chd = Dvd::from_image(&packed[..]).unwrap();
        same(&mut from_chd, &mut Dvd::from_image(&img[..]).unwrap(), name);
        assert!(from_chd.damage().is_none(), "{name}");
        let m = open_medium(&packed[..], Vec::new(), IsoForm::Mode2Form1, Lone::Detect, Kind::Cd).unwrap();
        assert_eq!(m.kind(), Kind::Dvd, "{name}: a DVD CHD by its unit and metadata");
        assert!(from_chd.place(0).is_some_and(|p| p.file.is_none() && (p.offset as usize) < packed.len()), "{name}");
        assert!(emusen_disc::Disc::from_image(&packed[..], IsoForm::Mode1).is_err(), "{name}: no compact disc");
    }
}

/// Four bytes changed at each stored hunk's first byte of a DVD CHD: read whole, every one refused in words, none read
/// silently otherwise and none a panic (the checks of EmuSen_Disc.md §13 on a DVD's codecs).
#[test]
fn a_damaged_dvd_chd_is_refused_in_words() {
    let Some(chdman) = chdman() else { return };
    let mut noisy = image(600, 600, None);
    let mut x = 0x1234_5678u32;
    for b in noisy[40 * USER..300 * USER].iter_mut() {
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        *b = x as u8;
    }
    let packed = create_dvd(&chdman, &noisy, "noisy", &[]);
    let clean = Dvd::from_image(&packed[..]).unwrap();
    let mut places: Vec<u64> = (0..600).filter_map(|l| clean.place(l)).map(|p| p.offset).collect();
    places.sort();
    places.dedup();
    assert!(places.len() > 10, "{} stored hunks", places.len());
    let mut outcomes = std::collections::BTreeMap::new();
    for &at in &places {
        let mut bad = packed.clone();
        for b in &mut bad[at as usize..at as usize + 4] {
            *b ^= 0x5A;
        }
        let read = std::panic::catch_unwind(|| {
            let mut f = [0u8; FRAME];
            let Ok(mut d) = Dvd::from_image(&bad[..]) else { return Some("refused at open".to_string()) };
            let mut wrong = 0;
            for lba in 0..600 {
                d.read(lba, &mut f);
                if d.damage().is_none() && f[12..2060] != noisy[lba as usize * USER..(lba as usize + 1) * USER] {
                    wrong += 1;
                }
            }
            if wrong > 0 { None } else { d.damage().map(str::to_string) }
        });
        let words = read.unwrap_or_else(|_| panic!("the hunk at byte {at} panicked"));
        assert!(words.as_deref().is_some_and(|w| w.contains("hunk") || w.contains("refused")), "the hunk at byte {at} was read silently wrong");
        let w = words.unwrap();
        *outcomes.entry(if w.contains("CRC") { "the CRC" } else if w.contains("refused") { "at open" } else { "the decoder" }).or_insert(0) += 1;
    }
    eprintln!("{} stored hunks damaged, refused by: {outcomes:?}", places.len());
}
