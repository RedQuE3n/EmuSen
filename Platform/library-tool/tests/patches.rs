//! Step 6's gate for `pkg patch` (EmuSen_GamePackages.md §7, §15): over a generated library, patches made by the test's
//! own encoders are added to a package, checked against the image they will be given, named as RetroArch finds them
//! when the list allows it, applied at load to reproduce the targets the test built, and removed; mismatched sources
//! and a patch made for the headerless form are refused with words; and the library is left byte for byte as it was.

#[path = "../../patch/tests/encoders/mod.rs"]
mod encoders;

use emusen_galaxia::json::{self, Value};
use emusen_galaxia::package;
use emusen_galaxia::rom_hash;
use emusen_galaxia::tree::Tree;
use emusen_library_tool::job;
use emusen_library_tool::Context;
use std::path::{Path, PathBuf};
use std::time::Duration;

struct Temp(PathBuf);

impl Temp {
    fn new(label: &str) -> Temp {
        let dir = std::env::temp_dir().join(format!("emusen-patches-{label}-{}", std::process::id()));
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

/// The one package event of a request, or the start's refusal.
fn run(ctx: &Context, request: &str) -> Value {
    let j = job::start(request, ctx.clone()).unwrap();
    let mut events = Vec::new();
    while !j.finished() {
        if let Some(line) = j.peek(Duration::from_millis(200)) {
            j.take();
            events.push(json::parse(&line).unwrap());
        }
    }
    let package = events.iter().find(|e| e.get("event").and_then(Value::as_str) == Some("package")).cloned();
    package.unwrap_or_else(|| events.into_iter().next().unwrap())
}

fn words(e: &Value) -> String {
    let mut l = String::new();
    json::write_compact(&mut l, e);
    l
}

fn state(e: &Value) -> &str {
    e.get("state").and_then(Value::as_str).unwrap_or("absent")
}

fn context(t: &Temp) -> Context {
    let tree = Tree { root: t.at("root"), data_override: Some(t.at("home")), config_override: Some(t.at("home/etc/EmuSen")), ..Tree::default() };
    Context::pinned(tree, Some(t.at("library")), String::new())
}

fn add(ctx: &Context, package: &str, file: &str) -> Value {
    run(ctx, &format!(r#"{{"op":"pkg-patch-add","paths":["{package}"],"patch":"{file}"}}"#))
}

fn files(folder: &str) -> Vec<String> {
    let mut f: Vec<String> = std::fs::read_dir(folder).unwrap().flatten().filter(|e| e.path().is_file()).map(|e| e.file_name().to_string_lossy().into_owned()).collect();
    f.sort();
    f
}

#[test]
fn patches_are_added_applied_named_and_removed_and_the_library_is_left_as_it_was() {
    let t = Temp::new("gate");
    let ctx = context(&t);
    let gamma = encoders::image(0x10000, 41);
    put(&t.at("library/snes/Gamma.sfc"), &gamma);
    let mut alpha = b"NES\x1a\x02\x01\0\0\0\0\0\0\0\0\0\0".to_vec();
    alpha.extend(encoders::image(0x8000 + 0x2000, 43));
    put(&t.at("library/nes/Alpha.nes"), &alpha);
    put(&t.at("library/psx/Disc.cue"), b"FILE \"Disc.bin\" BINARY\n  TRACK 01 MODE2/2352\n    INDEX 01 00:00:00\n");
    put(&t.at("library/psx/Disc.bin"), &vec![0u8; 2352 * 4]);
    let before = listing(&t.at("library"));
    assert_eq!(state(&run(&ctx, r#"{"op":"pkg-make","paths":["/"]}"#)), "made");

    // Targets built first, then each patch by the test's own encoder.
    let mut t1 = gamma.clone();
    t1[0x100..0x180].fill(0x42);
    t1.extend(encoders::image(4000, 47));
    let mut t2 = t1.clone();
    t2[0x2000] ^= 0xFF;
    t2[0x9000..0x9010].copy_from_slice(b"translated text!");
    put(&t.at("patches/Redesign.bps"), &encoders::bps(&gamma, &t1, b""));
    put(&t.at("patches/fix.ips"), &encoders::ips(&t1, &t2, true));
    let given = listing(&t.at("patches"));
    let folder = t.at("home/Packages/SNES/Gamma.sfc");

    let one = add(&ctx, "SNES/Gamma.sfc", &t.at("patches/Redesign.bps"));
    assert_eq!(state(&one), "added", "{}", words(&one));
    assert_eq!(one.get("patch").and_then(Value::as_str), Some("Gamma.bps"), "one BPS takes RetroArch's name: {}", words(&one));
    assert_eq!(one.get("retroarch"), Some(&Value::Bool(true)));
    let doc = package::read(&format!("{folder}/package.json")).unwrap().unwrap();
    assert_eq!(doc.get("patched").unwrap().get("sha1").unwrap().as_str(), Some(rom_hash::sha1_of(&t1).as_str()));
    assert_eq!(doc.get("dump").unwrap().get("file").unwrap().get("sha1").unwrap().as_str(), Some(rom_hash::sha1_of(&gamma).as_str()), "the dump's identity is kept apart");
    assert_eq!(emusen_patch::package::image_at(&folder, &gamma).unwrap(), Some(t1.clone()), "the image a core is given is the target the test built");

    let two = add(&ctx, "SNES/Gamma.sfc", &t.at("patches/fix.ips"));
    assert_eq!(state(&two), "added", "{}", words(&two));
    assert_eq!(two.get("retroarch"), Some(&Value::Bool(false)), "a BPS then an IPS is no RetroArch list");
    assert_eq!(files(&folder), ["Redesign.bps", "fix.ips", "package.json"], "each keeps its own name");
    assert_eq!(emusen_patch::package::image_at(&folder, &gamma).unwrap(), Some(t2.clone()));
    assert_eq!(state(&run(&ctx, r#"{"op":"pkg-check","paths":["SNES"]}"#)), "verified");

    let refused = run(&ctx, r#"{"op":"pkg-patch-rm","paths":["SNES/Gamma.sfc"],"index":1}"#);
    assert_eq!(state(&refused), "failed", "{}", words(&refused));
    assert!(words(&refused).contains("the rest no longer apply") && words(&refused).contains(&rom_hash::sha1_of(&t1)), "the IPS is pinned to the image the BPS made: {}", words(&refused));
    let removed = run(&ctx, r#"{"op":"pkg-patch-rm","paths":["SNES/Gamma.sfc"],"index":2}"#);
    assert_eq!(state(&removed), "removed", "{}", words(&removed));
    assert_eq!(files(&folder), ["Gamma.bps", "package.json"], "the copy is removed and the one left takes RetroArch's name again");
    assert_eq!(emusen_patch::package::image_at(&folder, &gamma).unwrap(), Some(t1.clone()));

    // A patch for another image, and one for the NES game's headerless form, refused with words.
    let other = encoders::image(0x10000, 99);
    put(&t.at("patches/other.ups"), &encoders::ups(&other, &t2));
    let wrong = add(&ctx, "SNES/Gamma.sfc", &t.at("patches/other.ups"));
    assert_eq!(state(&wrong), "failed");
    let w = words(&wrong);
    assert!(w.contains(&format!("{:08x}", encoders::crc32(&other))) && w.contains(&format!("{:08x}", encoders::crc32(&t1))), "the patch's source and the image the list makes are named: {w}");
    let headerless = &alpha[16..];
    let mut hacked = headerless.to_vec();
    hacked[0x50] ^= 1;
    put(&t.at("patches/headerless.bps"), &encoders::bps(headerless, &hacked, b""));
    let form = add(&ctx, "NES/Alpha.nes", &t.at("patches/headerless.bps"));
    assert_eq!(state(&form), "failed");
    assert!(words(&form).contains("comparable form") && words(&form).contains("copier header"), "{}", words(&form));
    put(&t.at("patches/disc.ips"), b"PATCH\0\0\0\0\x01\x01EOF");
    assert!(words(&add(&ctx, "PlayStation/Disc.cue", &t.at("patches/disc.ips"))).contains("disc patches are left"));

    // A patch file changed in the package is caught by check and at load.
    std::fs::write(format!("{folder}/Gamma.bps"), b"BPS1 not the one added").unwrap();
    let checked = run(&ctx, r#"{"op":"pkg-check","paths":["SNES"]}"#);
    assert_eq!(state(&checked), "changed");
    assert!(words(&checked).contains("Gamma.bps is not the patch that was added"), "{}", words(&checked));
    assert!(emusen_patch::package::image_at(&folder, &gamma).unwrap_err().contains("is not the patch that was added"));

    assert_eq!(listing(&t.at("library")), before, "the library is left as it was");
    assert_eq!(listing(&t.at("patches")).into_iter().filter(|f| given.iter().any(|g| g.0 == f.0)).collect::<Vec<_>>(), given, "the files given are only read");
}
