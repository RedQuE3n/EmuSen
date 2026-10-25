//! Step 5's gate in Rust (EmuSen_GamePackages.md §7, §14): over a generated library, `pkg make` makes a linked
//! package for every game and a second run makes none; `pkg migrate` copies a game's saves, states and cheat list in
//! and overwrites nothing; `pkg ls` and `pkg check` find a moved file again by its hashes and catch a changed one and
//! a missing one; and the library is left byte for byte as it was. The C# twin and the runtimes are WiseMan's.

use emusen_galaxia::claims;
use emusen_galaxia::json::{self, Value};
use emusen_galaxia::package;
use emusen_galaxia::rom_hash;
use emusen_galaxia::tree::Tree;
use emusen_library_tool::job::{self, Job};
use emusen_library_tool::Context;
use std::path::{Path, PathBuf};
use std::time::Duration;

struct Temp(PathBuf);

impl Temp {
    fn new(label: &str) -> Temp {
        let dir = std::env::temp_dir().join(format!("emusen-packages-{label}-{}", std::process::id()));
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

/// Every file under `root`: its path, size, modification time and MD5.
fn listing(root: &str) -> Vec<(String, u64, std::time::SystemTime, String)> {
    let mut out = Vec::new();
    let mut stack = vec![PathBuf::from(root)];
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

fn summary(events: &[Value]) -> String {
    events.iter().map(|e| { let mut l = String::new(); json::write_compact(&mut l, e); l }).collect::<Vec<_>>().join("\n")
}

/// The state of the package event naming `what` as its source or its package.
fn state<'a>(events: &'a [Value], what: &str) -> &'a str {
    events
        .iter()
        .filter(|e| e.get("event").and_then(Value::as_str) == Some("package"))
        .find(|e| e.get("source").and_then(Value::as_str) == Some(what) || e.get("package").and_then(Value::as_str) == Some(what))
        .and_then(|e| e.get("state"))
        .and_then(Value::as_str)
        .unwrap_or("absent")
}

fn nes(fill: u8) -> Vec<u8> {
    let mut b = b"NES\x1a\x02\x01\0\0\0\0\0\0\0\0\0\0".to_vec();
    b.extend(std::iter::repeat_n(fill, 0x8000 + 0x2000));
    b
}

const SHEET: &str = "FILE \"Disc (Track 1).bin\" BINARY\n  TRACK 01 MODE2/2352\n    INDEX 01 00:00:00\nFILE \"Disc (Track 2).bin\" BINARY\n  TRACK 02 AUDIO\n    INDEX 01 00:00:00\n";

fn context(t: &Temp) -> Context {
    let tree = Tree { root: t.at("root"), data_override: Some(t.at("home")), config_override: Some(t.at("home/etc/EmuSen")), ..Tree::default() };
    let mut ctx = Context::pinned(tree, Some(t.at("library")), String::new());
    ctx.state_folder = Some(t.at("library/states-in-the-library"));
    ctx
}

#[test]
fn packages_are_made_migrated_listed_and_checked_and_the_library_is_left_as_it_was() {
    let t = Temp::new("gate");
    let ctx = context(&t);
    put(&t.at("library/nes/Alpha.nes"), &nes(1));
    put(&t.at("library/nes/USA/Beta (U).nes"), &nes(2));
    put(&t.at("library/snes/Gamma.sfc"), &vec![3u8; 0x8000]);
    put(&t.at("library/psx/Disc/Disc.cue"), SHEET.as_bytes());
    put(&t.at("library/psx/Disc/Disc (Track 1).bin"), &vec![0u8; 2352 * 4]);
    put(&t.at("library/psx/Disc/Disc (Track 2).bin"), &vec![7u8; 2352 * 2]);
    put(&t.at("library/nes/Alpha.srm"), b"a save beside the ROM, as an earlier build kept it");
    put(&t.at("library/states-in-the-library/Alpha.state"), b"state 1");
    put(&t.at("library/states-in-the-library/Alpha.slot2.state"), b"state 2");
    put(&t.at("library/states-in-the-library/Alpha.png"), b"picture");
    put(&t.at("library/states-in-the-library/Alpha Two.state"), b"another game's");
    put(&t.at("home/Saves/NES/Alpha.srm"), b"the save");
    put(&t.at("home/Saves/NES/Alpha Two.srm"), b"another game's save");
    put(&t.at("home/etc/EmuSen/cheats/NES/Alpha.json"), br#"{"cheats":[]}"#);
    let before = listing(&t.at("library"));

    let made = run(&ctx, r#"{"op":"pkg-make","paths":["/"]}"#);
    let s = summary(&made);
    for source in ["nes/Alpha.nes", "nes/USA/Beta (U).nes", "snes/Gamma.sfc", "psx/Disc/Disc.cue"] {
        assert_eq!(state(&made, source), "made", "{source}: {s}");
    }
    assert_eq!(made.iter().filter(|e| e.get("event").and_then(Value::as_str) == Some("package")).count(), 4, "{s}");
    let alpha = t.at("home/Packages/NES/Alpha.nes");
    let doc = package::read(&format!("{alpha}/package.json")).unwrap().unwrap();
    assert_eq!(doc.get("game").unwrap().get("path").unwrap().as_str(), Some("nes/Alpha.nes"));
    assert_eq!(doc.get("dump").unwrap().get("file").unwrap().get("md5").unwrap().as_str(), Some(rom_hash::md5(&t.at("library/nes/Alpha.nes")).unwrap().as_str()));
    assert!(doc.get("dump").unwrap().get("transformed").unwrap().get("sha1").is_some(), "the iNES header is the NES's transform");
    let disc = package::read(&t.at("home/Packages/PlayStation/Disc/Disc.cue/package.json")).unwrap().unwrap();
    let Some(Value::Array(companions)) = disc.get("companions") else { panic!("{s}") };
    assert_eq!(companions.len(), 2);
    assert_eq!(run(&ctx, r#"{"op":"pkg-make","paths":["nes"]}"#).iter().filter(|e| e.get("state").and_then(Value::as_str) == Some("already")).count(), 2);

    let migrated = run(&ctx, r#"{"op":"pkg-migrate","paths":["NES/Alpha.nes"]}"#);
    assert_eq!(state(&migrated, "NES/Alpha.nes"), "migrated", "{}", summary(&migrated));
    let read = |rel: &str| std::fs::read(format!("{alpha}/{rel}")).ok();
    assert_eq!(read("saves/Alpha.srm").as_deref(), Some(&b"the save"[..]), "the console's folder wins over the earlier place beside the ROM");
    assert_eq!(read("states/Alpha.state").as_deref(), Some(&b"state 1"[..]));
    assert_eq!(read("states/Alpha.slot2.state").as_deref(), Some(&b"state 2"[..]));
    assert_eq!(read("states/Alpha.png").as_deref(), Some(&b"picture"[..]));
    assert_eq!(read("cheats.json").as_deref(), Some(&br#"{"cheats":[]}"#[..]));
    assert!(read("states/Alpha Two.state").is_none() && read("saves/Alpha Two.srm").is_none(), "another game's files stay out");
    std::fs::write(format!("{alpha}/saves/Alpha.srm"), b"played since").unwrap();
    let again = run(&ctx, r#"{"op":"pkg-migrate","paths":["NES/Alpha.nes"]}"#);
    assert_eq!(state(&again, "NES/Alpha.nes"), "nothing", "{}", summary(&again));
    assert_eq!(read("saves/Alpha.srm").as_deref(), Some(&b"played since"[..]), "nothing is overwritten");
    assert_eq!(listing(&t.at("library")), before, "making and migrating left the library as it was");

    let checked = run(&ctx, r#"{"op":"pkg-check","paths":[]}"#);
    assert!(checked.iter().filter(|e| e.get("event").and_then(Value::as_str) == Some("package")).all(|e| e.get("state").and_then(Value::as_str) == Some("verified")), "{}", summary(&checked));

    // Moved: found again by size and hashes; ls says so and writes nothing, check rewrites the manifest.
    std::fs::create_dir_all(t.at("library/nes/Europe")).unwrap();
    std::fs::rename(t.at("library/nes/USA/Beta (U).nes"), t.at("library/nes/Europe/Beta (E).nes")).unwrap();
    let listed = run(&ctx, r#"{"op":"pkg-ls","paths":[],"console":"NES"}"#);
    assert_eq!(state(&listed, "NES/USA/Beta (U).nes"), "moved", "{}", summary(&listed));
    assert!(summary(&listed).contains(r#""to":"nes/Europe/Beta (E).nes""#), "{}", summary(&listed));
    assert_eq!(listed.iter().filter(|e| e.get("event").and_then(Value::as_str) == Some("package")).count(), 2, "ls NES lists the NES's two");
    let beta = t.at("home/Packages/NES/USA/Beta (U).nes/package.json");
    assert_eq!(package::read(&beta).unwrap().unwrap().get("game").unwrap().get("path").unwrap().as_str(), Some("nes/USA/Beta (U).nes"));
    let checked = run(&ctx, r#"{"op":"pkg-check","paths":["NES/USA"]}"#);
    assert_eq!(state(&checked, "NES/USA/Beta (U).nes"), "moved", "{}", summary(&checked));
    assert_eq!(package::read(&beta).unwrap().unwrap().get("game").unwrap().get("path").unwrap().as_str(), Some("nes/Europe/Beta (E).nes"));
    let index = claims::index(&ctx.places().roots());
    assert!(index.claimant(&t.at("library/nes/Europe/Beta (E).nes")).is_some());

    // Changed and missing: a track's byte, a cartridge's byte, and a file gone.
    std::fs::write(t.at("library/psx/Disc/Disc (Track 2).bin"), vec![8u8; 2352 * 2]).unwrap();
    std::fs::write(t.at("library/snes/Gamma.sfc"), vec![4u8; 0x8000]).unwrap();
    // A change within the second the package recorded is seen by check, which hashes; ls hashes only a file whose size or time moved.
    let later = std::time::SystemTime::now() + Duration::from_secs(3600);
    std::fs::File::options().write(true).open(t.at("library/snes/Gamma.sfc")).unwrap().set_modified(later).unwrap();
    std::fs::remove_file(t.at("library/nes/Alpha.nes")).unwrap();
    let checked = run(&ctx, r#"{"op":"pkg-check","paths":[]}"#);
    let s = summary(&checked);
    assert_eq!(state(&checked, "PlayStation/Disc/Disc.cue"), "changed", "{s}");
    assert!(s.contains("track 2 (Disc (Track 2).bin) is not the one recorded"), "{s}");
    assert_eq!(state(&checked, "SNES/Gamma.sfc"), "changed", "{s}");
    assert!(s.contains("snes/Gamma.sfc has SHA-1"), "{s}");
    assert_eq!(state(&checked, "NES/Alpha.nes"), "missing", "{s}");
    let listed = run(&ctx, r#"{"op":"pkg-ls","paths":[]}"#);
    assert_eq!(state(&listed, "SNES/Gamma.sfc"), "changed", "{}", summary(&listed));
    assert_eq!(state(&listed, "NES/Alpha.nes"), "missing", "{}", summary(&listed));
}

#[test]
fn a_path_that_names_nothing_is_refused_and_nothing_is_made() {
    let t = Temp::new("refused");
    let ctx = context(&t);
    put(&t.at("library/nes/Alpha.nes"), &nes(1));
    put(&t.at("library/readme.txt"), b"not a game");
    let made = run(&ctx, r#"{"op":"pkg-make","paths":["gone","readme.txt","../x"]}"#);
    let s = summary(&made);
    assert!(s.contains("gone: no such file or folder in the library") && s.contains("readme.txt: no system of this build takes its kind of file"), "{s}");
    assert!(s.contains("no .. part"), "{s}");
    assert!(!Path::new(&t.at("home/Packages")).exists(), "{s}");
    let checked = run(&ctx, r#"{"op":"pkg-check","paths":["SNES"]}"#);
    assert!(summary(&checked).contains("SNES: no package there"), "{}", summary(&checked));
}
