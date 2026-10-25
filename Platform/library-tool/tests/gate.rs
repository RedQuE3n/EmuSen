//! Step 3's synthetic gate (EmuSen_GamePackages.md §7, §12.4): generated sheets, made as `emusen-disc`'s own CHD test
//! makes them, round-trip through all three checks; a corrupted CHD fails check 2; a stopped batch resumes and
//! finishes; the pin refuses a wrong binary; and the library is left byte for byte as it was. The chdman runs need
//! `EMUSEN_CHDMAN` naming the pinned binary; without it they are skipped and say so.

use emusen_disc::ecma130::{self, RAW};
use emusen_galaxia::json::{self, Value};
use emusen_galaxia::rom_hash;
use emusen_galaxia::tree::Tree;
use emusen_library_tool::job::{self, Job};
use emusen_library_tool::Context;
use std::path::{Path, PathBuf};
use std::time::Duration;

const DATA: u32 = 40;
const AUDIO: u32 = 30;

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

/// Audio no codec can shrink, so that chdman stores its hunks as they are and a changed byte is a changed sample.
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

fn mode1_iso(sectors: usize) -> Vec<u8> {
    let mut iso: Vec<u8> = (0..sectors * 2048).map(|i| (i * 7 / 3) as u8).collect();
    iso[16 * 2048..16 * 2048 + 6].copy_from_slice(b"\x01CD001");
    iso
}

/// A stored zip of `files`, read back by Galaxia's own reader.
fn zip(files: &[(&str, &[u8])]) -> Vec<u8> {
    let (mut out, mut central) = (Vec::new(), Vec::new());
    for (name, bytes) in files {
        let mut crc = rom_hash::Crc32::default();
        crc.append(bytes);
        let offset = out.len() as u32;
        let head = |sig: u32, central: bool| {
            let mut h = Vec::new();
            h.extend(sig.to_le_bytes());
            if central {
                h.extend(20u16.to_le_bytes());
            }
            h.extend([20u16, 0, 0, 0, 0].iter().flat_map(|v| v.to_le_bytes()));
            h.extend(crc.value().to_le_bytes());
            h.extend((bytes.len() as u32).to_le_bytes());
            h.extend((bytes.len() as u32).to_le_bytes());
            h.extend((name.len() as u16).to_le_bytes());
            h.extend(0u16.to_le_bytes());
            if central {
                h.extend([0u16, 0, 0].iter().flat_map(|v| v.to_le_bytes()));
                h.extend(0u32.to_le_bytes());
                h.extend(offset.to_le_bytes());
            }
            h.extend(name.as_bytes());
            h
        };
        out.extend(head(0x0403_4B50, false));
        out.extend_from_slice(bytes);
        central.extend(head(0x0201_4B50, true));
    }
    let start = out.len() as u32;
    out.extend(&central);
    out.extend(0x0605_4B50u32.to_le_bytes());
    out.extend([0u16, 0, files.len() as u16, files.len() as u16].iter().flat_map(|v| v.to_le_bytes()));
    out.extend((central.len() as u32).to_le_bytes());
    out.extend(start.to_le_bytes());
    out.extend(0u16.to_le_bytes());
    out
}

struct Temp(PathBuf);

impl Temp {
    fn new(label: &str) -> Temp {
        let base = std::env::var("TMPDIR").map(PathBuf::from).unwrap_or_else(|_| std::env::temp_dir());
        let p = base.join(format!("emusen-library-tool-{label}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        Temp(std::fs::canonicalize(&p).unwrap())
    }

    fn at(&self, rel: &str) -> PathBuf {
        self.0.join(rel)
    }
}

impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn put(at: PathBuf, bytes: &[u8]) {
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(at, bytes).unwrap();
}

const TWO_FILES: &str = "FILE \"Two (Track 1).bin\" BINARY\n  TRACK 01 MODE2/2352\n    INDEX 01 00:00:00\nFILE \"Two (Track 2).bin\" BINARY\n  TRACK 02 AUDIO\n    INDEX 00 00:00:00\n    INDEX 01 00:00:07\n";
const ONE_FILE: &str = "FILE \"One.bin\" BINARY\nTRACK 01 MODE2/2352\nINDEX 01 00:00:00\nTRACK 02 AUDIO\nPREGAP 00:00:03\nINDEX 01 00:00:40\nTRACK 03 AUDIO\nINDEX 00 00:00:60\nINDEX 01 00:00:62\n";

/// The library: a two-file Redump-style dump with a stored pause, one file of three tracks with an unstored PREGAP, a
/// lone raw track, a disc whose audio no codec shrinks, a zipped sheet, an ISO of the Sega CD's folder, and a file
/// that is no disc.
fn library(root: &Path) {
    let d: Vec<u8> = (0..DATA).flat_map(data_sector).collect();
    let a: Vec<u8> = (0..AUDIO).flat_map(audio_sector).collect();
    put(root.join("psx/Two/Two.cue"), TWO_FILES.as_bytes());
    put(root.join("psx/Two/Two (Track 1).bin"), &d);
    put(root.join("psx/Two/Two (Track 2).bin"), &a);
    put(root.join("psx/One.cue"), ONE_FILE.as_bytes());
    put(root.join("psx/One.bin"), &[d.clone(), a.clone(), a[..20 * RAW].to_vec()].concat());
    put(root.join("psx/Lone (Track 1).bin"), &d);
    put(root.join("psx/Noise/Noise.cue"), b"FILE \"Noise (Track 1).bin\" BINARY\n  TRACK 01 MODE2/2352\n    INDEX 01 00:00:00\nFILE \"Noise (Track 2).bin\" BINARY\n  TRACK 02 AUDIO\n    INDEX 01 00:00:00\n");
    put(root.join("psx/Noise/Noise (Track 1).bin"), &d);
    put(root.join("psx/Noise/Noise (Track 2).bin"), &noise(60, 0x9E37_79B9_7F4A_7C15));
    put(root.join("psx/Zipped.zip"), &zip(&[("Zipped.cue", TWO_FILES.replace("Two", "Zipped").as_bytes()), ("Zipped (Track 1).bin", &d), ("Zipped (Track 2).bin", &a)]));
    put(root.join("segacd/Disc.iso"), &mode1_iso(64));
    put(root.join("psx/readme.txt"), b"not a disc");
}

/// Every file of the library: its path, size, modification time and MD5.
fn listing(root: &Path) -> Vec<(String, u64, std::time::SystemTime, String)> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for e in std::fs::read_dir(&dir).unwrap().flatten() {
            let p = e.path();
            let meta = std::fs::symlink_metadata(&p).unwrap();
            if meta.is_dir() {
                stack.push(p);
            } else {
                out.push((p.to_string_lossy().into_owned(), meta.len(), meta.modified().unwrap(), rom_hash::md5(&p.to_string_lossy()).unwrap()));
            }
        }
    }
    out.sort();
    out
}

fn context(temp: &Temp, chdman: &str, pinned: Option<String>) -> Context {
    let tree = Tree { root: temp.at("root").to_string_lossy().into_owned(), data_override: Some(temp.at("home").to_string_lossy().into_owned()), ..Tree::default() };
    let pin = pinned.or_else(|| rom_hash_sha256(chdman));
    Context { tree, library: Some(temp.at("library").to_string_lossy().into_owned()), chdman: chdman.to_string(), pin, release: "0.289".into() }
}

fn rom_hash_sha256(at: &str) -> Option<String> {
    emusen_library_tool::pin::sha256(at).ok()
}

fn events(job: &Job) -> Vec<Value> {
    let mut out = Vec::new();
    while !job.finished() {
        if let Some(line) = job.peek(Duration::from_millis(200)) {
            job.take();
            out.push(json::parse(&line).unwrap());
        }
    }
    out
}

fn run(ctx: &Context, request: &str) -> Vec<Value> {
    events(&job::start(request, ctx.clone()).unwrap())
}

fn state_of<'a>(events: &'a [Value], source: &str) -> &'a str {
    events.iter().filter(|e| e.get("event").and_then(Value::as_str) == Some("disc") && e.get("source").and_then(Value::as_str) == Some(source)).next_back().and_then(|e| e.get("state")).and_then(Value::as_str).unwrap_or("absent")
}

fn summary(events: &[Value]) -> String {
    events.iter().map(|e| { let mut l = String::new(); json::write_compact(&mut l, e); l }).collect::<Vec<_>>().join("\n")
}

fn chdman() -> Option<String> {
    let c = std::env::var("EMUSEN_CHDMAN").ok();
    if c.is_none() {
        eprintln!("skipped: EMUSEN_CHDMAN names no chdman");
    }
    c
}

const SOURCES: [&str; 6] = ["psx/Lone (Track 1).bin", "psx/Noise/Noise.cue", "psx/One.cue", "psx/Two/Two.cue", "psx/Zipped.zip", "segacd/Disc.iso"];

#[test]
fn generated_sheets_round_trip_through_all_three_checks_and_the_library_is_unchanged() {
    let Some(chdman) = chdman() else { return };
    let temp = Temp::new("round");
    library(&temp.at("library"));
    let before = listing(&temp.at("library"));
    let ctx = context(&temp, &chdman, None);
    let got = run(&ctx, r#"{"op": "convert", "paths": [], "threads": 2}"#);
    for source in SOURCES {
        let event = got.iter().rev().find(|e| e.get("source").and_then(Value::as_str) == Some(source)).unwrap_or_else(|| panic!("{source}: {}", summary(&got)));
        assert_eq!(event.get("state").and_then(Value::as_str), Some("converted"), "{source}: {}", summary(&got));
        let checks = event.get("checks").unwrap();
        for c in ["chdman", "tracks", "toc"] {
            assert_eq!(checks.get(c).and_then(Value::as_str), Some("passed"), "{source} {c}");
        }
    }
    assert_eq!(state_of(&got, "psx/readme.txt"), "absent");
    assert_eq!(listing(&temp.at("library")), before, "the library changed");
    let manifest = std::fs::read_to_string(temp.at("home/Packages/PlayStation/Two/Two.chd/package.json")).unwrap();
    let doc = emusen_galaxia::package::parse(&manifest).unwrap();
    assert_eq!(doc.get("game").and_then(|g| g.get("path")).and_then(Value::as_str), Some("psx/Two/Two.chd"));
    assert_eq!(doc.get("dump").and_then(|d| d.get("form")).and_then(Value::as_str), Some("cue-bin"));
    assert!(temp.at("home/Packages/PlayStation/Two/Two.chd/Two.cue").is_file());
    assert!(temp.at("home/Packages/Sega CD/Disc.chd/package.json").is_file());
    let log = got.iter().find(|e| e.get("event").and_then(Value::as_str) == Some("start")).and_then(|e| e.get("log")).and_then(Value::as_str).unwrap().to_string();
    assert!(std::fs::read_to_string(&log).unwrap().lines().filter(|l| l.contains("\tconvert\t")).count() >= SOURCES.len(), "{log}");
    let listed = run(&ctx, r#"{"op": "ls"}"#);
    let two = listed.iter().find(|e| e.get("source").and_then(Value::as_str) == Some("psx/Two/Two.cue")).unwrap();
    let whole = ["psx/Two/Two.cue", "psx/Two/Two (Track 1).bin", "psx/Two/Two (Track 2).bin"].iter().map(|f| std::fs::metadata(temp.at("library").join(f)).unwrap().len()).sum::<u64>();
    assert_eq!(two.get("size").and_then(|s| if let Value::Number(n) = s { n.parse::<u64>().ok() } else { None }), Some(whole), "ls gives a sheet's disc its whole size");
    let checked = run(&ctx, r#"{"op": "check"}"#);
    for source in SOURCES {
        assert_eq!(state_of(&listed, source), "converted", "{source}");
        assert_eq!(state_of(&checked, source), "verified", "{source}");
    }
    let again = run(&ctx, r#"{"op": "convert"}"#);
    assert!(SOURCES.iter().all(|s| state_of(&again, s) == "already"), "{}", summary(&again));
    let fast = Temp::new("fast");
    library(&fast.at("library"));
    let fctx = context(&fast, &chdman, None);
    let got = run(&fctx, r#"{"op": "convert", "paths": ["psx/Two"], "fast": true}"#);
    assert_eq!(state_of(&got, "psx/Two/Two.cue"), "converted", "cdzs reads through emusen-disc: {}", summary(&got));
    assert!(std::fs::read_to_string(fast.at("home/Packages/PlayStation/Two/Two.chd/package.json")).unwrap().contains("cdzs"));
    assert_eq!(listing(&temp.at("library")), before, "the library changed");
}

fn check_event(ctx: &Context, rel: &str, source: &str) -> Value {
    let checked = run(ctx, &format!(r#"{{"op": "check", "paths": ["{rel}"]}}"#));
    checked.iter().rev().find(|e| e.get("source").and_then(Value::as_str) == Some(source)).cloned().unwrap_or_else(|| panic!("{}", summary(&checked)))
}

fn check_word(event: &Value, check: &str) -> String {
    event.get("checks").and_then(|c| c.get(check)).and_then(Value::as_str).unwrap_or("").to_string()
}

/// A CHD that is whole, so that chdman's verify passes it and its SHA-1 is the package's, but whose second track
/// differs from the dump by one sample: only check 2 can see it.
#[test]
fn a_corrupted_chd_fails_check_two() {
    let Some(chdman) = chdman() else { return };
    let temp = Temp::new("corrupt");
    library(&temp.at("library"));
    let ctx = context(&temp, &chdman, None);
    let got = run(&ctx, r#"{"op": "convert", "paths": ["psx/Two"]}"#);
    assert_eq!(state_of(&got, "psx/Two/Two.cue"), "converted", "{}", summary(&got));
    let other = temp.at("other");
    std::fs::create_dir_all(&other).unwrap();
    for name in ["Two.cue", "Two (Track 1).bin", "Two (Track 2).bin"] {
        std::fs::copy(temp.at(&format!("library/psx/Two/{name}")), other.join(name)).unwrap();
    }
    let mut audio = std::fs::read(other.join("Two (Track 2).bin")).unwrap();
    audio[12 * RAW + 401] ^= 0x01;
    std::fs::write(other.join("Two (Track 2).bin"), &audio).unwrap();
    let tool = emusen_library_tool::chdman::Chdman::open(&chdman, rom_hash_sha256(&chdman).as_deref(), "0.289").unwrap();
    let chd = temp.at("home/Compressed/psx/Two/Two.chd");
    std::fs::remove_file(&chd).unwrap();
    tool.create_cd(&other.join("Two.cue").to_string_lossy(), &chd.to_string_lossy(), None, 2, &|| false).unwrap();
    let manifest = temp.at("home/Packages/PlayStation/Two/Two.chd/package.json").to_string_lossy().into_owned();
    let mut doc = emusen_galaxia::package::read(&manifest).unwrap().unwrap();
    let seen = emusen_library_tool::convert::seen(&chd.to_string_lossy()).unwrap();
    let sha1 = tool.info(&chd.to_string_lossy()).unwrap().sha1;
    if let Value::Object(members) = &mut doc {
        for (name, value) in members.iter_mut() {
            let Value::Object(inner) = value else { continue };
            for (field, v) in inner.iter_mut() {
                match (name.text.as_str(), field.text.as_str()) {
                    ("game", "size") => *v = Value::number(seen.size),
                    ("game", "seen") => *v = emusen_library_tool::convert::obj(vec![("size", Value::number(seen.size)), ("mtime", Value::string(&emusen_library_tool::convert::utc(seen.modified)))]),
                    ("container", "sha1") => *v = Value::string(&sha1),
                    _ => {}
                }
            }
        }
    }
    emusen_galaxia::package::write(&manifest, &doc).unwrap().unwrap();
    let event = check_event(&ctx, "psx/Two", "psx/Two/Two.cue");
    assert_eq!(event.get("state").and_then(Value::as_str), Some("failed"));
    assert_eq!((check_word(&event, "chdman").as_str(), check_word(&event, "toc").as_str()), ("passed", "passed"));
    assert!(check_word(&event, "tracks").starts_with("failed: track 2 rebuilt from the CHD has SHA-1"), "{}", check_word(&event, "tracks"));
}

/// A whole CHD of the same tracks laid out with a longer unstored pause: its tracks' bytes are the dump's, so only the
/// table of contents, check 3, can see it, and check 2 is not run over a disc whose tracks do not start where the
/// original's did.
#[test]
fn a_chd_of_another_layout_fails_check_three() {
    let Some(chdman) = chdman() else { return };
    let temp = Temp::new("layout");
    library(&temp.at("library"));
    let ctx = context(&temp, &chdman, None);
    let got = run(&ctx, r#"{"op": "convert", "paths": ["psx/One.cue"]}"#);
    assert_eq!(state_of(&got, "psx/One.cue"), "converted", "{}", summary(&got));
    let other = temp.at("other");
    std::fs::create_dir_all(&other).unwrap();
    std::fs::copy(temp.at("library/psx/One.bin"), other.join("One.bin")).unwrap();
    std::fs::write(other.join("One.cue"), ONE_FILE.replace("PREGAP 00:00:03", "PREGAP 00:00:05")).unwrap();
    let tool = emusen_library_tool::chdman::Chdman::open(&chdman, rom_hash_sha256(&chdman).as_deref(), "0.289").unwrap();
    let chd = temp.at("home/Compressed/psx/One.chd");
    std::fs::remove_file(&chd).unwrap();
    tool.create_cd(&other.join("One.cue").to_string_lossy(), &chd.to_string_lossy(), None, 2, &|| false).unwrap();
    let event = check_event(&ctx, "psx/One.cue", "psx/One.cue");
    assert_eq!(event.get("state").and_then(Value::as_str), Some("failed"));
    assert!(check_word(&event, "toc").starts_with("failed: track 2 differs"), "{}", check_word(&event, "toc"));
    assert_eq!(check_word(&event, "tracks"), "failed: not run, since the table of contents differs");
}

/// Bytes changed at a stored hunk's start: chdman's verify fails it, and EmuSen's reader is not run over it, since a
/// damaged map can still stop it at open (EmuSen_Disc.md §13.3), so the disc fails and the program goes on.
#[test]
fn a_damaged_chd_fails_check_one_and_is_not_read() {
    let Some(chdman) = chdman() else { return };
    let temp = Temp::new("damaged");
    library(&temp.at("library"));
    let ctx = context(&temp, &chdman, None);
    let got = run(&ctx, r#"{"op": "convert", "paths": ["psx/Two"]}"#);
    assert_eq!(state_of(&got, "psx/Two/Two.cue"), "converted", "{}", summary(&got));
    let chd = temp.at("home/Compressed/psx/Two/Two.chd");
    let mut bytes = std::fs::read(&chd).unwrap();
    let first = emusen_disc::Disc::from_image(bytes.clone(), emusen_disc::IsoForm::Mode1).unwrap().place(150).unwrap().offset as usize;
    bytes[first..first + 4].iter_mut().for_each(|b| *b ^= 0x5A);
    std::fs::write(&chd, &bytes).unwrap();
    let event = check_event(&ctx, "psx/Two", "psx/Two/Two.cue");
    assert_eq!(event.get("state").and_then(Value::as_str), Some("failed"));
    assert!(check_word(&event, "chdman").starts_with("failed: chdman verify failed"), "{}", check_word(&event, "chdman"));
    assert_eq!(check_word(&event, "tracks"), "failed: not run, since chdman found the CHD damaged");
}

#[test]
fn a_stopped_batch_resumes_and_finishes_leaving_no_partial() {
    let Some(chdman) = chdman() else { return };
    let temp = Temp::new("resume");
    library(&temp.at("library"));
    let ctx = context(&temp, &chdman, None);
    let stale = temp.at("home/Compressed/psx/Old.chd.partial-1");
    put(stale.clone(), b"left by a process that stopped");
    let old = std::time::SystemTime::now() - Duration::from_secs(2 * 3600);
    std::fs::File::options().write(true).open(&stale).unwrap().set_modified(old).unwrap();
    let job = job::start(r#"{"op": "convert", "threads": 1}"#, ctx.clone()).unwrap();
    let mut seen = Vec::new();
    while !job.finished() {
        if let Some(line) = job.peek(Duration::from_millis(100)) {
            job.take();
            let e = json::parse(&line).unwrap();
            if e.get("state").and_then(Value::as_str) == Some("converted") {
                job.stop();
            }
            seen.push(e);
        }
    }
    let converted: Vec<&str> = SOURCES.iter().copied().filter(|s| state_of(&seen, s) == "converted").collect();
    assert_eq!(converted.len(), 1, "{}", summary(&seen));
    assert!(!stale.exists(), "a partial an hour old is removed");
    let leftovers = |dir: &Path| -> usize {
        let mut n = 0;
        let mut stack = vec![dir.to_path_buf()];
        while let Some(d) = stack.pop() {
            for e in std::fs::read_dir(&d).into_iter().flatten().flatten() {
                let name = e.file_name().to_string_lossy().into_owned();
                if name.contains(".partial-") || name.starts_with(".scratch-") {
                    n += 1;
                }
                if e.path().is_dir() {
                    stack.push(e.path());
                }
            }
        }
        n
    };
    assert_eq!(leftovers(&temp.at("home/Compressed")), 0);
    let rest = run(&ctx, r#"{"op": "convert"}"#);
    for s in SOURCES {
        assert_eq!(state_of(&rest, s), if converted.contains(&s) { "already" } else { "converted" }, "{s}: {}", summary(&rest));
    }
}

#[test]
fn the_pin_refuses_a_wrong_binary_and_nothing_is_written() {
    let temp = Temp::new("pin");
    library(&temp.at("library"));
    let fake = temp.at("chdman");
    put(fake.clone(), b"#!/bin/sh\necho not chdman\n");
    let fake = fake.to_string_lossy().into_owned();
    let pinned = "55abb0d257ba692ff14c4bd20073f1717ac54f362e2bdc56ee3e637783c87e0f".to_string();
    let ctx = context(&temp, &fake, Some(pinned.clone()));
    let got = run(&ctx, r#"{"op": "convert"}"#);
    let end = got.last().unwrap();
    let words = end.get("error").and_then(Value::as_str).unwrap();
    let actual = emusen_library_tool::pin::sha256(&fake).unwrap();
    assert!(words.contains(&actual) && words.contains(&pinned) && words.contains("nothing was converted"), "{words}");
    assert!(!temp.at("home/Compressed").exists() && !temp.at("home/Packages").exists());
}

#[test]
fn a_library_that_holds_the_output_is_refused_before_anything_is_written() {
    let temp = Temp::new("overlap");
    library(&temp.at("home"));
    let mut ctx = context(&temp, "/bin/true", Some("0".repeat(64)));
    ctx.library = Some(temp.at("home").to_string_lossy().into_owned());
    let got = run(&ctx, r#"{"op": "convert"}"#);
    let words = got.last().unwrap().get("error").and_then(Value::as_str).unwrap().to_string();
    assert!(words.contains("overlap"), "{words}");
    assert!(!temp.at("home/Compressed").exists());
}

