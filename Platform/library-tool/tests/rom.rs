//! T2's tests (EmuSen_LibraryTools.md §7): over a generated library, `rom hash` gives a sheet's tracks as dumped and
//! one comparable line for the same dump in another form (headered, zipped, split, a CHD); `rom check` names each
//! broken file's check and passes the sound ones; `rom dups` groups the same dump and not a package's own files; and
//! the library is left byte for byte as it was. The CHD cases need `EMUSEN_CHDMAN`; without it they say so.

use emusen_disc::ecma130::{self, RAW};
use emusen_galaxia::claims::{Index, Package};
use serde_json::Value;
use emusen_galaxia::rom_hash::{self, Hashes};
use emusen_galaxia::tree::Tree;
use emusen_library_tool::job::{self, Job};
use emusen_library_tool::rom::{self, Member};
use emusen_library_tool::Context;
use std::path::{Path, PathBuf};
use std::time::Duration;

struct Temp(PathBuf);

impl Temp {
    fn new(label: &str) -> Temp {
        let dir = std::env::temp_dir().join(format!("emusen-rom-{label}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Temp(std::fs::canonicalize(&dir).unwrap())
    }

    fn at(&self, rel: &str) -> String {
        self.0.join(rel).to_string_lossy().into_owned()
    }
}

impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn put(at: &str, bytes: &[u8]) {
    std::fs::create_dir_all(Path::new(at).parent().unwrap()).unwrap();
    std::fs::write(at, bytes).unwrap();
}

fn noise(n: usize, seed: u64) -> Vec<u8> {
    let mut x = seed;
    (0..n)
        .map(|_| {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x as u8
        })
        .collect()
}

fn data(sectors: u32) -> Vec<u8> {
    (0..sectors)
        .flat_map(|i| {
            let user: Vec<u8> = (0..2048u32).map(|k| (k * 5 + i) as u8).collect();
            let mut s = [0u8; RAW];
            ecma130::mode2_form1(150 + i, [1, 0, ecma130::submode::DATA, 0], &user, &mut s);
            s
        })
        .collect()
}

/// A stored zip of `files`, with each entry's CRC-32 recorded.
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

const DISC: &str = "FILE \"Disc.bin\" BINARY\n  TRACK 01 MODE2/2352\n    INDEX 01 00:00:00\n  TRACK 02 AUDIO\n    INDEX 00 00:00:20\n    INDEX 01 00:00:22\n";
const SPLIT: &str = "FILE \"Split (Track 1).bin\" BINARY\n  TRACK 01 MODE2/2352\n    INDEX 01 00:00:00\nFILE \"Split (Track 2).bin\" BINARY\n  TRACK 02 AUDIO\n    INDEX 00 00:00:00\n    INDEX 01 00:00:02\n";

/// The library: the same NES game headered and not, the same SNES game plain, with a copier header and zipped, one
/// disc as one file and as two, and a broken file of each kind the checks know.
fn library(root: &str) {
    let body = noise(32 * 1024, 7);
    let mut ines = b"NES\x1A\x02\x01\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec();
    ines.extend(&body);
    put(&format!("{root}/nes/Game.nes"), &ines);
    put(&format!("{root}/nes/Game (Bare).nes"), &body);
    put(&format!("{root}/nes/Empty.nes"), b"");
    let snes = noise(64 * 1024, 11);
    put(&format!("{root}/snes/Game.sfc"), &snes);
    let mut copier = vec![0u8; 512];
    copier.extend(&snes);
    put(&format!("{root}/snes/Game (Copier).smc"), &copier);
    put(&format!("{root}/snes/Game.zip"), &zip(&[("Game.sfc", &snes)]));
    let mut bad = zip(&[("Bad.sfc", &snes)]);
    bad[30 + 7 + 100] ^= 0x01;
    put(&format!("{root}/snes/Bad.zip"), &bad);
    let mut disc = data(20);
    disc.extend(noise(12 * RAW, 3));
    put(&format!("{root}/psx/Disc.cue"), DISC.as_bytes());
    put(&format!("{root}/psx/Disc.bin"), &disc);
    put(&format!("{root}/psx/Split/Split.cue"), SPLIT.as_bytes());
    put(&format!("{root}/psx/Split/Split (Track 1).bin"), &disc[..20 * RAW]);
    put(&format!("{root}/psx/Split/Split (Track 2).bin"), &disc[20 * RAW..]);
    put(&format!("{root}/psx/Missing.cue"), b"FILE \"Gone.bin\" BINARY\n  TRACK 01 MODE2/2352\n    INDEX 01 00:00:00\n");
    put(&format!("{root}/psx/Partial.cue"), b"FILE \"Partial.bin\" BINARY\n  TRACK 01 MODE2/2352\n    INDEX 01 00:00:00\n");
    put(&format!("{root}/psx/Partial.bin"), &disc[..5 * RAW + 100]);
    put(&format!("{root}/psx/Two.m3u"), b"Disc.cue\nAbsent.cue\n");
}

fn context(t: &Temp) -> Context {
    let tree = Tree { root: t.at("root").into(), data_override: Some(t.at("home").into()), config_override: Some(t.at("home/etc/EmuSen").into()), ..Tree::default() };
    let chdman = std::env::var("EMUSEN_CHDMAN").unwrap_or_default();
    let mut ctx = Context::pinned(tree, Some(t.at("library")), chdman.clone());
    ctx.pin = emusen_library_tool::pin::sha256(&chdman).ok();
    ctx
}

fn events(job: &Job) -> Vec<Value> {
    let mut out = Vec::new();
    while !job.finished() {
        if let Some(line) = job.peek(Duration::from_millis(200)) {
            job.take();
            out.push(serde_json::from_str::<Value>(&line).unwrap());
        }
    }
    out
}

fn run(ctx: &Context, request: &str) -> Vec<Value> {
    events(&job::start(request, ctx.clone()).unwrap())
}

fn text<'a>(e: &'a Value, name: &str) -> &'a str {
    e.get(name).and_then(Value::as_str).unwrap_or("")
}

/// Each hashed line of a run as `name -> sha1 size`.
fn lines(events: &[Value]) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for e in events.iter().filter(|e| text(e, "event") == "file") {
        if let Some(Value::Array(ls)) = e.get("lines") {
            for l in ls {
                let size = match l.get("size") {
                    Some(Value::Number(t)) => t.to_string(),
                    _ => String::new(),
                };
                out.push((text(l, "name").to_string(), format!("{} {size}", text(l, "sha1"))));
            }
        }
    }
    out
}

fn line_of<'a>(lines: &'a [(String, String)], name: &str) -> &'a str {
    lines.iter().find(|(n, _)| n == name).map(|(_, v)| v.as_str()).unwrap_or_else(|| panic!("no line for {name}: {lines:?}"))
}

/// Each checked file's line: `source: ok` or its failed checks by name.
fn checked(events: &[Value]) -> Vec<String> {
    events
        .iter()
        .filter(|e| text(e, "event") == "file")
        .map(|e| {
            let failed: Vec<String> = match e.get("failed") {
                Some(Value::Array(f)) => f.iter().map(|x| format!("{}: {}", text(x, "check"), text(x, "words"))).collect(),
                _ => Vec::new(),
            };
            format!("{}: {}", text(e, "source"), if failed.is_empty() { "ok".into() } else { failed.join("; ") })
        })
        .collect()
}

fn listing(root: &str) -> Vec<(String, u64, String)> {
    let mut out = Vec::new();
    let mut stack = vec![PathBuf::from(root)];
    while let Some(dir) = stack.pop() {
        for e in std::fs::read_dir(&dir).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else {
                out.push((p.to_string_lossy().into_owned(), p.metadata().unwrap().len(), rom_hash::md5(&p).unwrap()));
            }
        }
    }
    out.sort();
    out
}

fn chdman() -> Option<String> {
    let c = std::env::var("EMUSEN_CHDMAN").ok().filter(|c| !c.is_empty());
    if c.is_none() {
        eprintln!("skipped the CHD cases: EMUSEN_CHDMAN names no chdman");
    }
    c
}

#[test]
fn rom_hash_gives_the_dump_and_one_comparable_line_for_each_form_of_it() {
    let temp = Temp::new("hash");
    library(&temp.at("library"));
    let before = listing(&temp.at("library"));
    let ctx = context(&temp);
    let plain = lines(&run(&ctx, r#"{"op": "rom-hash", "paths": ["nes", "snes", "psx/Disc.cue"], "recursive": true}"#));
    let tracks = rom_hash::tracks(&temp.at("library/psx/Disc.cue"), &|| false).unwrap();
    assert_eq!(line_of(&plain, "psx/Disc.cue#01"), format!("{} {}", tracks[0].hashes.sha1.clone().unwrap(), tracks[0].hashes.size));
    assert_eq!(line_of(&plain, "psx/Disc.cue#02"), format!("{} {}", tracks[1].hashes.sha1.clone().unwrap(), 12 * RAW));
    assert_ne!(line_of(&plain, "nes/Game.nes"), line_of(&plain, "nes/Game (Bare).nes"));
    assert_ne!(line_of(&plain, "snes/Game.sfc"), line_of(&plain, "snes/Game.zip"));
    let form = lines(&run(&ctx, r#"{"op": "rom-hash", "paths": ["/"], "recursive": true, "comparable": true}"#));
    assert_eq!(line_of(&form, "nes/Game.nes"), line_of(&form, "nes/Game (Bare).nes"));
    assert_eq!(line_of(&form, "snes/Game.sfc"), line_of(&form, "snes/Game (Copier).smc"));
    assert_eq!(line_of(&form, "snes/Game.sfc"), line_of(&form, "snes/Game.zip"));
    assert_eq!(line_of(&form, "psx/Disc.cue"), line_of(&form, "psx/Split/Split.cue"));
    assert_eq!(line_of(&form, "psx/Disc.cue").split(' ').nth(1), Some(&*(32 * RAW).to_string()), "the program area, 32 sectors");
    assert!(!form.iter().any(|(n, _)| n.ends_with(".m3u") || n.contains('#')), "{form:?}");
    let refused = run(&ctx, r#"{"op": "rom-hash", "paths": ["nes", "nowhere"]}"#);
    let said: Vec<&str> = match refused[0].get("refused") {
        Some(Value::Array(r)) => r.iter().filter_map(Value::as_str).collect(),
        _ => Vec::new(),
    };
    assert_eq!(said, ["nes: a folder; -r takes what is under it", "nowhere: no such file or folder in the library"]);
    if let Some(chdman) = chdman() {
        let chd = temp.at("library/psx/Disc.chd");
        let made = std::process::Command::new(&chdman).args(["createcd", "-i", &temp.at("library/psx/Disc.cue"), "-o", &chd]).output().unwrap();
        assert!(made.status.success(), "{}", String::from_utf8_lossy(&made.stderr));
        let form = lines(&run(&ctx, r#"{"op": "rom-hash", "paths": ["psx"], "recursive": true, "comparable": true}"#));
        assert_eq!(line_of(&form, "psx/Disc.chd"), line_of(&form, "psx/Disc.cue"));
        std::fs::remove_file(&chd).unwrap();
    }
    assert_eq!(listing(&temp.at("library")), before, "the library was written");
}

#[test]
fn rom_check_names_each_broken_files_check_and_passes_the_sound_ones() {
    let temp = Temp::new("check");
    library(&temp.at("library"));
    let before = listing(&temp.at("library"));
    let ctx = context(&temp);
    let got = run(&ctx, r#"{"op": "rom-check", "paths": ["/"], "recursive": true}"#);
    let lines = checked(&got);
    let want = [
        "nes/Empty.nes: size: Empty.nes is empty",
        "nes/Game (Bare).nes: ok",
        "nes/Game.nes: ok",
        "psx/Disc.cue: ok",
        "psx/Missing.cue: sheet: it names Gone.bin, which is not there",
        "psx/Partial.cue: sheet: track 01 is 11,860 bytes, not a whole number of 2,352-byte sectors",
        "psx/Split/Split.cue: ok",
        "psx/Two.m3u: sheet: it lists Absent.cue, which is not there",
        "snes/Game (Copier).smc: ok",
        "snes/Game.sfc: ok",
        "snes/Game.zip: ok",
    ];
    for w in want {
        assert!(lines.iter().any(|l| l == w), "no line {w:?} in {lines:#?}");
    }
    let bad = lines.iter().find(|l| l.starts_with("snes/Bad.zip: archive: Bad.sfc is damaged")).unwrap_or_else(|| panic!("{lines:#?}"));
    assert!(bad.contains("CRC-32"), "{bad}");
    let end = got.last().unwrap();
    assert_eq!(text(end, "event"), "end");
    if let Some(chdman) = chdman() {
        let chd = temp.at("library/psx/Broken.chd");
        let made = std::process::Command::new(&chdman).args(["createcd", "-i", &temp.at("library/psx/Disc.cue"), "-o", &chd]).output().unwrap();
        assert!(made.status.success());
        let sound = checked(&run(&ctx, r#"{"op": "rom-check", "paths": ["psx/Broken.chd"]}"#));
        assert_eq!(sound, ["psx/Broken.chd: ok"]);
        let mut bytes = std::fs::read(&chd).unwrap();
        let first = emusen_disc::Disc::from_image(bytes.clone(), emusen_disc::IsoForm::Mode1).unwrap().place(160).unwrap().offset as usize;
        bytes[first..first + 4].iter_mut().for_each(|b| *b ^= 0x5A);
        std::fs::write(&chd, &bytes).unwrap();
        let broken = checked(&run(&ctx, r#"{"op": "rom-check", "paths": ["psx/Broken.chd"]}"#));
        assert!(broken[0].starts_with("psx/Broken.chd: chd: chdman verify failed"), "{broken:?}");
        let f = rom::File { rel: "psx/Broken.chd".into(), path: chd.clone(), system: emusen_systems::find("psx") };
        let read = rom::check(&f, None, &|| false);
        assert!(read.len() == 1 && read[0].check == "chd" && read[0].words.contains("damaged"), "{read:?}");
        std::fs::remove_file(&chd).unwrap();
    }
    assert_eq!(listing(&temp.at("library")), before, "the library was written");
}

fn member(rel: &str, sha1: &str) -> Member {
    Member { rel: rel.into(), path: format!("/lib/{rel}"), hashes: Hashes { md5: "m".into(), crc32: "c".into(), sha1: Some(sha1.into()), size: 4 } }
}

#[test]
fn rom_dups_groups_the_same_dump_and_a_packages_own_files_are_one_game() {
    let temp = Temp::new("dups");
    library(&temp.at("library"));
    let ctx = context(&temp);
    let got = run(&ctx, r#"{"op": "rom-dups", "console": null}"#);
    let groups: Vec<Vec<String>> = got
        .iter()
        .filter(|e| text(e, "event") == "group")
        .map(|e| match e.get("members") {
            Some(Value::Array(m)) => m.iter().map(|x| text(x, "name").to_string()).collect(),
            _ => Vec::new(),
        })
        .collect();
    assert_eq!(
        groups,
        [vec!["nes/Game (Bare).nes", "nes/Game.nes"], vec!["psx/Disc.cue", "psx/Split/Split.cue"], vec!["snes/Game (Copier).smc", "snes/Game.sfc", "snes/Game.zip"]],
        "{got:?}"
    );
    let snes = run(&ctx, r#"{"op": "rom-dups", "console": "SNES"}"#);
    assert_eq!(snes.iter().filter(|e| text(e, "event") == "group").count(), 1);
    let none = run(&ctx, r#"{"op": "rom-dups", "console": "Atari 9000"}"#);
    assert_eq!(text(none.last().unwrap(), "error"), "Atari 9000 is no console of this build");
    let package = Package { folder: "/pk/Disc".into(), system: "psx".into(), game: "/lib/a.chd".into(), claims: vec!["/lib/a.chd".into(), "/lib/a.cue".into()] };
    let index = Index { folders: vec!["/pk/Disc".into()], packages: vec![package] };
    assert!(rom::groups(vec![member("a.chd", "1"), member("a.cue", "1")], &index).is_empty(), "a package and the file it claims are one game");
    let three = rom::groups(vec![member("a.chd", "1"), member("a.cue", "1"), member("b.cue", "1"), member("c.cue", "2")], &index);
    assert_eq!(three.len(), 1);
    assert_eq!(three[0].iter().map(|m| m.rel.as_str()).collect::<Vec<_>>(), ["a.chd", "a.cue", "b.cue"]);
}
