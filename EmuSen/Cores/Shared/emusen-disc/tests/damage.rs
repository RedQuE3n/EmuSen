//! Damaged CHDs (EmuSen_Disc.md §13): the step 3 gate's two synthetic discs made into CHDs by chdman, then four bytes
//! changed at each stored hunk's first byte and 5 and 200 bytes into it, each read whole through this crate in a child
//! process, so that a panic is counted as one rather than ending the run. Opt in with `EMUSEN_DISC_CHDMAN`.

use emusen_disc::ecma130::{self, RAW};
use emusen_disc::{Disc, IsoForm, Sector};
use std::path::{Path, PathBuf};
use std::process::Command;

const CASE: &str = "EMUSEN_DISC_DAMAGE_CASE";
const DATA: u32 = 40;
const AUDIO: u32 = 30;
const NOISE: u32 = 60;
const TWO: &str = "FILE \"Two (Track 1).bin\" BINARY\n  TRACK 01 MODE2/2352\n    INDEX 01 00:00:00\nFILE \"Two (Track 2).bin\" BINARY\n  TRACK 02 AUDIO\n    INDEX 00 00:00:00\n    INDEX 01 00:00:07\n";
const NOISY: &str = "FILE \"Noise (Track 1).bin\" BINARY\n  TRACK 01 MODE2/2352\n    INDEX 01 00:00:00\nFILE \"Noise (Track 2).bin\" BINARY\n  TRACK 02 AUDIO\n    INDEX 01 00:00:00\n";
/// Where in a stored hunk the four bytes are changed, as EmuSen_GamePackages.md §12.2 measured.
const INTO: [u64; 3] = [0, 5, 200];

fn data_sector(i: u32) -> [u8; RAW] {
    let user: Vec<u8> = (0..2048u32).map(|k| (k * 3 + i * 11) as u8).collect();
    let mut s = [0u8; RAW];
    ecma130::mode2_form1(150 + i, [1, 0, ecma130::submode::DATA, 0], &user, &mut s);
    s
}

fn audio_sector(i: u32) -> [u8; RAW] {
    let mut s = [0u8; RAW];
    for k in 0..588 {
        let v = (i * 1000 + k) as i16;
        s[4 * k as usize..4 * k as usize + 2].copy_from_slice(&v.to_le_bytes());
        s[4 * k as usize + 2..4 * k as usize + 4].copy_from_slice(&(-v).to_le_bytes());
    }
    s
}

/// Audio no codec can shrink.
fn noise(sectors: u32, seed: u64) -> Vec<u8> {
    let mut x = seed;
    (0..sectors as usize * RAW)
        .map(|_| {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x as u8
        })
        .collect()
}

/// FNV-1a over every sector's kind and bytes.
fn fnv(h: &mut u64, bytes: &[u8]) {
    for b in bytes {
        *h = (*h ^ *b as u64).wrapping_mul(0x0000_0100_0000_01B3);
    }
}

/// What reading the whole disc gave: its digest, the stored sectors that came back synthesised, and the refusals.
struct Read {
    digest: u64,
    lost: u32,
    refusal: Option<String>,
}

fn read_whole(bytes: Vec<u8>) -> Read {
    let mut disc = match Disc::from_image(bytes, IsoForm::Mode1) {
        Ok(d) => d,
        Err(e) => return Read { digest: 0, lost: 0, refusal: Some(format!("not opened: {e}")) },
    };
    let (mut h, mut lost, mut raw) = (0xCBF2_9CE4_8422_2325u64, 0u32, [0u8; RAW]);
    for at in 0..disc.lead_out() {
        let stored = disc.place(at).is_some();
        let kind = disc.read(at, &mut raw);
        let synthesised = matches!(kind, Sector::Data { synthesised: true, .. } | Sector::Audio { synthesised: true });
        if stored && synthesised {
            lost += 1;
        }
        fnv(&mut h, &[synthesised as u8]);
        fnv(&mut h, &raw);
    }
    Read { digest: h, lost, refusal: disc.damage().map(str::to_string) }
}

/// The child's half: one damaged file read whole, the result on one line.
fn child(path: &str) {
    let r = read_whole(std::fs::read(path).unwrap());
    println!("RESULT {:016x} {} {}", r.digest, r.lost, r.refusal.as_deref().unwrap_or("-"));
}

fn chdman_cd(chdman: &str, dir: &Path, sheet: &str, files: &[(&str, Vec<u8>)], name: &str) -> PathBuf {
    for (file, bytes) in files {
        std::fs::write(dir.join(file), bytes).unwrap();
    }
    std::fs::write(dir.join(format!("{name}.cue")), sheet).unwrap();
    let chd = dir.join(format!("{name}.chd"));
    let run = Command::new(chdman).args(["createcd", "-f", "-np", "2", "-i"]).arg(dir.join(format!("{name}.cue"))).arg("-o").arg(&chd).output().unwrap();
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    chd
}

#[derive(Debug, PartialEq)]
enum Outcome {
    Panic,
    Refused,
    Silent,
    Whole,
}

#[test]
fn every_damaged_hunk_is_refused_without_a_panic() {
    if let Ok(path) = std::env::var(CASE) {
        return child(&path);
    }
    let Ok(chdman) = std::env::var("EMUSEN_DISC_CHDMAN") else {
        eprintln!("skipped: EMUSEN_DISC_CHDMAN names no chdman");
        return;
    };
    let base = std::env::var("TMPDIR").map(PathBuf::from).unwrap_or_else(|_| std::env::temp_dir());
    let dir = base.join(format!("emusen-disc-damage-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let d: Vec<u8> = (0..DATA).flat_map(data_sector).collect();
    let a: Vec<u8> = (0..AUDIO).flat_map(audio_sector).collect();
    let discs = [
        chdman_cd(&chdman, &dir, TWO, &[("Two (Track 1).bin", d.clone()), ("Two (Track 2).bin", a)], "Two"),
        chdman_cd(&chdman, &dir, NOISY, &[("Noise (Track 1).bin", d), ("Noise (Track 2).bin", noise(NOISE, 0x9E37_79B9_7F4A_7C15))], "Noise"),
    ];
    let me = std::env::current_exe().unwrap();
    let (mut lines, mut tally) = (Vec::new(), Vec::new());
    for chd in &discs {
        let clean = std::fs::read(chd).unwrap();
        let whole = read_whole(clean.clone());
        assert!(whole.refusal.is_none() && whole.lost == 0, "{}: the clean CHD does not read whole: {:?}", chd.display(), whole.refusal);
        let disc = Disc::from_image(clean.clone(), IsoForm::Mode1).unwrap();
        let mut hunks: Vec<u64> = Vec::new();
        for at in 0..disc.lead_out() {
            hunks.extend(disc.place(at).map(|p| p.offset).filter(|o| !hunks.contains(o)));
        }
        let damaged = dir.join("damaged.chd");
        for (n, &offset) in hunks.iter().enumerate() {
            for into in INTO {
                let mut bytes = clean.clone();
                let o = (offset + into) as usize;
                bytes[o..o + 4].iter_mut().for_each(|b| *b ^= 0x5A);
                std::fs::write(&damaged, &bytes).unwrap();
                let run = Command::new(&me).args(["every_damaged_hunk_is_refused_without_a_panic", "--exact", "--nocapture", "--test-threads=1"]).env(CASE, &damaged).output().unwrap();
                let out = String::from_utf8_lossy(&run.stdout);
                let result = out.lines().find_map(|l| l.split_once("RESULT ").map(|(_, r)| r));
                let (outcome, words) = match result {
                    None => (Outcome::Panic, String::from_utf8_lossy(&run.stderr).lines().find(|l| !l.starts_with("thread") && !l.is_empty()).unwrap_or("").to_string()),
                    Some(r) => {
                        let mut f = r.splitn(3, ' ');
                        let (digest, lost, words) = (f.next().unwrap(), f.next().unwrap().parse::<u32>().unwrap(), f.next().unwrap_or("-").to_string());
                        let same = digest == format!("{:016x}", whole.digest);
                        match (words != "-", same, lost) {
                            (true, _, _) => (Outcome::Refused, words),
                            (false, true, 0) => (Outcome::Whole, words),
                            _ => (Outcome::Silent, format!("{lost} stored sectors synthesised, digest {}", if same { "same" } else { "differs" })),
                        }
                    }
                };
                lines.push(format!("{} hunk {n} at {offset}+{into}: {outcome:?} {words}", chd.file_name().unwrap().to_string_lossy()));
                tally.push(outcome);
            }
        }
    }
    std::fs::remove_dir_all(&dir).ok();
    let count = |o: Outcome| tally.iter().filter(|t| **t == o).count();
    let summary = format!("{} corruptions: {} panics, {} refused, {} silent, {} whole", tally.len(), count(Outcome::Panic), count(Outcome::Refused), count(Outcome::Silent), count(Outcome::Whole));
    println!("{}\n{summary}", lines.join("\n"));
    assert!(tally.iter().all(|t| *t == Outcome::Refused || *t == Outcome::Whole), "{summary}");
}
