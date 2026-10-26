//! Step 7's gate in Rust (EmuSen_GamePackages.md §7, §19): `pkg make --self-contained` copies a game and its tracks into
//! its package and the manifest names the copies and their sources, a linked package is made self-contained in place,
//! `--zip` writes the package as one zip, `pkg check` passes on the folder and on the zip, the library is left byte for
//! byte as it was, and a package copied whole to a stand-in machine (a fresh tree in a temporary folder) opens there,
//! as its folder copied with its times and as its zip; `pkg identity` sets the scraper's identity. The runtimes'
//! half (saves, states and their records, settings, art and patches read on the other machine) is WiseMan's.

use emusen_galaxia::claims::{self, Places};
use emusen_galaxia::json::{self, Value};
use emusen_galaxia::tree::Tree;
use emusen_galaxia::{package, package_zip, rom_hash};
use emusen_library_tool::job::{self, Job};
use emusen_library_tool::Context;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

struct Temp(PathBuf);

impl Temp {
    fn new(label: &str) -> Temp {
        let dir = std::env::temp_dir().join(format!("emusen-self-contained-{label}-{}", std::process::id()));
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

fn files(root: &str) -> Vec<(String, u64, SystemTime, String)> {
    let mut out = Vec::new();
    let mut stack = vec![PathBuf::from(root)];
    while let Some(dir) = stack.pop() {
        for e in std::fs::read_dir(&dir).unwrap().flatten() {
            let p = e.path();
            let meta = std::fs::symlink_metadata(&p).unwrap();
            if meta.is_dir() {
                stack.push(p);
            } else {
                let rel = p.strip_prefix(root).unwrap().to_string_lossy().into_owned();
                out.push((rel, meta.len(), meta.modified().unwrap(), rom_hash::md5(&p.to_string_lossy()).unwrap()));
            }
        }
    }
    out.sort();
    out
}

/// A folder copied as `cp -rp` copies it: every file with its modification time.
fn copy_with_times(from: &str, to: &str) {
    for (rel, _, modified, _) in files(from) {
        let target = format!("{to}/{rel}");
        std::fs::create_dir_all(Path::new(&target).parent().unwrap()).unwrap();
        std::fs::copy(format!("{from}/{rel}"), &target).unwrap();
        std::fs::File::options().write(true).open(&target).unwrap().set_modified(modified).unwrap();
    }
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

fn event<'a>(events: &'a [Value], what: &str) -> Option<&'a Value> {
    events
        .iter()
        .filter(|e| e.get("event").and_then(Value::as_str) == Some("package"))
        .find(|e| e.get("source").and_then(Value::as_str) == Some(what) || e.get("package").and_then(Value::as_str) == Some(what))
}

fn state<'a>(events: &'a [Value], what: &str) -> &'a str {
    event(events, what).and_then(|e| e.get("state")).and_then(Value::as_str).unwrap_or("absent")
}

fn nes(fill: u8) -> Vec<u8> {
    let mut b = b"NES\x1a\x02\x01\0\0\0\0\0\0\0\0\0\0".to_vec();
    b.extend((0..0x8000 + 0x2000).map(|i: u32| fill.wrapping_add((i % 13) as u8)));
    b
}

const SHEET: &str = "FILE \"Disc (Track 1).bin\" BINARY\n  TRACK 01 MODE2/2352\n    INDEX 01 00:00:00\nFILE \"Disc (Track 2).bin\" BINARY\n  TRACK 02 AUDIO\n    INDEX 01 00:00:00\n";

fn context(t: &Temp, machine: &str) -> Context {
    let tree = Tree { root: t.at(&format!("{machine}/root")), data_override: Some(t.at(&format!("{machine}/home"))), config_override: Some(t.at(&format!("{machine}/home/etc/EmuSen"))), ..Tree::default() };
    Context::pinned(tree, Some(t.at(&format!("{machine}/library"))), String::new())
}

/// An IPS patch of one record, written here from its description.
fn ips(offset: u32, bytes: &[u8]) -> Vec<u8> {
    let mut b = b"PATCH".to_vec();
    b.extend([(offset >> 16) as u8, (offset >> 8) as u8, offset as u8, (bytes.len() >> 8) as u8, bytes.len() as u8]);
    b.extend(bytes);
    b.extend(b"EOF");
    b
}

#[test]
fn a_self_contained_package_and_its_zip_open_on_another_machine_and_the_library_is_left_as_it_was() {
    let t = Temp::new("gate");
    let a = context(&t, "a");
    let old = SystemTime::UNIX_EPOCH + Duration::from_nanos(1_700_000_000_123_456_700);
    put(&t.at("a/library/nes/Alpha.nes"), &nes(1));
    std::fs::File::options().write(true).open(t.at("a/library/nes/Alpha.nes")).unwrap().set_modified(old).unwrap();
    put(&t.at("a/library/snes/Gamma.sfc"), &vec![3u8; 0x8000]);
    put(&t.at("a/library/psx/Disc/Disc.cue"), SHEET.as_bytes());
    put(&t.at("a/library/psx/Disc/Disc (Track 1).bin"), &vec![0u8; 2352 * 4]);
    put(&t.at("a/library/psx/Disc/Disc (Track 2).bin"), &vec![7u8; 2352 * 2]);
    let before = files(&t.at("a/library"));

    // A new self-contained package, zipped; a disc's sheet and tracks copied in at the places the sheet names them.
    let made = run(&a, r#"{"op":"pkg-make","paths":["nes/Alpha.nes","psx"],"self_contained":true,"zip":true}"#);
    let s = summary(&made);
    assert_eq!(state(&made, "nes/Alpha.nes"), "made", "{s}");
    let alpha = t.at("a/home/Packages/NES/Alpha.nes");
    let e = event(&made, "nes/Alpha.nes").unwrap();
    assert_eq!(e.get("mode").and_then(Value::as_str), Some("self-contained"), "{s}");
    assert_eq!(e.get("zip").and_then(|z| z.get("path")).and_then(Value::as_str), Some("NES/Alpha.nes.zip"), "{s}");
    assert_eq!(std::fs::read(format!("{alpha}/Alpha.nes")).unwrap(), nes(1));
    assert_eq!(std::fs::metadata(format!("{alpha}/Alpha.nes")).unwrap().modified().unwrap(), old, "the copy keeps its original's time");
    let doc = package::read(&format!("{alpha}/package.json")).unwrap().unwrap();
    assert_eq!(doc.get("mode").and_then(Value::as_str), Some("self-contained"));
    assert_eq!((doc.get("game").unwrap().get("where").unwrap().as_str(), doc.get("game").unwrap().get("path").unwrap().as_str()), (Some("package"), Some("Alpha.nes")));
    let Some(Value::Array(sources)) = doc.get("sources") else { panic!("no sources: {s}") };
    assert_eq!(sources[0].get("path").and_then(Value::as_str), Some("nes/Alpha.nes"));
    assert_eq!(sources[0].get("sha1").and_then(Value::as_str), rom_hash::hashes_of(&nes(1)).sha1.as_deref());
    assert_eq!(state(&made, "psx/Disc/Disc.cue"), "made", "{s}");
    let disc = t.at("a/home/Packages/PlayStation/Disc/Disc.cue");
    assert_eq!(std::fs::read(format!("{disc}/Disc (Track 2).bin")).unwrap(), vec![7u8; 2352 * 2]);
    let disc_doc = package::read(&format!("{disc}/package.json")).unwrap().unwrap();
    let Some(Value::Array(tracks)) = disc_doc.get("companions") else { panic!() };
    assert!(tracks.iter().all(|c| c.get("where").and_then(Value::as_str) == Some("package")), "{s}");

    // A linked package made self-contained in place; a zip that is there is never written over.
    assert_eq!(state(&run(&a, r#"{"op":"pkg-make","paths":["snes"]}"#), "snes/Gamma.sfc"), "made");
    let gamma = t.at("a/home/Packages/SNES/Gamma.sfc");
    put(&format!("{gamma}/saves/Gamma.srm"), b"played linked");
    let contained = run(&a, r#"{"op":"pkg-make","paths":["snes"],"self_contained":true}"#);
    assert_eq!(state(&contained, "snes/Gamma.sfc"), "contained", "{}", summary(&contained));
    assert_eq!(std::fs::read(format!("{gamma}/saves/Gamma.srm")).unwrap(), b"played linked", "its saves stay where they are");
    assert_eq!(state(&run(&a, r#"{"op":"pkg-make","paths":["snes"],"self_contained":true}"#), "snes/Gamma.sfc"), "already");
    assert_eq!(state(&run(&a, r#"{"op":"pkg-make","paths":["snes"],"zip":true}"#), "snes/Gamma.sfc"), "zipped");
    let again = run(&a, r#"{"op":"pkg-make","paths":["snes"],"zip":true}"#);
    assert_eq!(state(&again, "snes/Gamma.sfc"), "failed");
    assert!(summary(&again).contains("already there, and it was left as it is"), "{}", summary(&again));

    // One game, one save: the library file opens the package's copy; the scan's index claims it.
    let roots = a.places();
    let opened = claims::resolve(&roots.roots(), &t.at("a/library/nes/Alpha.nes")).unwrap();
    assert_eq!((opened.game, opened.package.as_deref()), (format!("{alpha}/Alpha.nes"), Some(alpha.as_str())));

    // A soft patch and the identity the scraper sends.
    put(&t.at("a/hack.ips"), &ips(0x20, &[0x42, 0x43]));
    let refused = run(&a, r#"{"op":"pkg-identity","paths":["NES/Alpha.nes"],"identity":"patched"}"#);
    assert_eq!(state(&refused, "NES/Alpha.nes"), "failed", "{}", summary(&refused));
    let added = run(&a, &format!(r#"{{"op":"pkg-patch-add","paths":["NES/Alpha.nes"],"patch":"{}"}}"#, t.at("a/hack.ips")));
    assert_eq!(state(&added, "NES/Alpha.nes"), "added", "{}", summary(&added));
    assert_eq!(state(&run(&a, r#"{"op":"pkg-identity","paths":["NES/Alpha.nes"],"identity":"patched"}"#), "NES/Alpha.nes"), "set");
    assert!(package::scrapes_as_patched(&package::read(&format!("{alpha}/package.json")).unwrap().unwrap()));
    assert_eq!(state(&run(&a, r#"{"op":"pkg-identity","paths":["NES/Alpha.nes"],"identity":"patched"}"#), "NES/Alpha.nes"), "already");

    // The package played: a save, a state and its record beside it, settings and a cover; then zipped again.
    put(&format!("{alpha}/saves/Alpha.patched-0.srm"), b"a save");
    put(&format!("{alpha}/states/Alpha.slot1.state"), b"a state");
    put(&format!("{alpha}/states/Alpha.slot1.json"), b"{\"StateModified\":1}");
    put(&format!("{alpha}/settings.json"), b"{\n  \"Crop\": \"1 2 3 4\"\n}");
    put(&format!("{alpha}/media/covers/Alpha.png"), b"a cover");
    std::fs::remove_file(format!("{alpha}.zip")).unwrap();
    assert_eq!(state(&run(&a, r#"{"op":"pkg-make","paths":["nes/Alpha.nes"],"zip":true}"#), "nes/Alpha.nes"), "zipped");

    // pkg check passes on both forms, the zip by its path relative to Packages; the library is as it was.
    let checked = run(&a, r#"{"op":"pkg-check","paths":["NES/Alpha.nes","NES/Alpha.nes.zip","SNES/Gamma.sfc","PlayStation"]}"#);
    let s = summary(&checked);
    let verified = checked.iter().filter(|e| e.get("event").and_then(Value::as_str) == Some("package") && e.get("state").and_then(Value::as_str) == Some("verified")).count();
    assert_eq!(verified, 4, "{s}");
    assert!(s.contains(r#""zip":true"#), "{s}");
    assert!(!std::fs::read_dir(t.at("a/home/Packages")).unwrap().flatten().any(|e| e.file_name().to_string_lossy().starts_with(".check")), "the check's folder is removed");
    assert_eq!(files(&t.at("a/library")), before, "making, containing, zipping and checking left the library as it was");

    // The stand-in machine: a fresh tree with a library of its own that lacks the game.
    let b = context(&t, "b");
    std::fs::create_dir_all(t.at("b/library")).unwrap();
    let carried = t.at("b/home/Packages/NES/Alpha.nes");
    copy_with_times(&alpha, &carried);
    let opened = claims::resolve(&b.places().roots(), &carried).unwrap();
    assert_eq!((opened.game, opened.package.as_deref()), (format!("{carried}/Alpha.nes"), Some(carried.as_str())));
    assert_eq!(files(&carried), files(&alpha), "every file and time came across");
    let checked = run(&b, r#"{"op":"pkg-check","paths":["NES"]}"#);
    assert_eq!(state(&checked, "NES/Alpha.nes"), "verified", "{}", summary(&checked));

    // The zip on a third machine: opened by its path, unpacked once beside it with every file and time, then the folder.
    let c = context(&t, "c");
    std::fs::create_dir_all(t.at("c/library")).unwrap();
    let zip = t.at("c/home/Packages/NES/Alpha.nes.zip");
    put(&zip, &std::fs::read(format!("{alpha}.zip")).unwrap());
    let places: Places = c.places();
    let opened = claims::resolve(&places.roots(), &zip).unwrap();
    let unpacked = t.at("c/home/Packages/NES/Alpha.nes");
    assert_eq!((opened.game, opened.package.as_deref()), (format!("{unpacked}/Alpha.nes"), Some(unpacked.as_str())));
    let ticks = |l: Vec<(String, u64, SystemTime, String)>| l.into_iter().map(|(p, n, t, h)| (p, n, emusen_galaxia::clock::ticks_of(t), h)).collect::<Vec<_>>();
    assert_eq!(ticks(files(&unpacked)), ticks(files(&alpha)), "every file and its time to the tick, as a state's record is stamped, came out of the zip");
    put(&format!("{unpacked}/saves/Alpha.patched-0.srm"), b"played on the third machine");
    assert_eq!(claims::resolve(&places.roots(), &zip).unwrap().package.as_deref(), Some(unpacked.as_str()));
    assert_eq!(std::fs::read(format!("{unpacked}/saves/Alpha.patched-0.srm")).unwrap(), b"played on the third machine", "a zip is never unpacked over its folder");
    let checked = run(&c, r#"{"op":"pkg-check","paths":["NES"]}"#);
    assert_eq!(state(&checked, "NES/Alpha.nes"), "verified", "{}", summary(&checked));
    assert!(package_zip::manifest_in(&zip).unwrap().is_some());
}
