//! Install, update, roll back, remove and verify against cores zipped by the test and served by its own TLS server, never
//! the internet and never a real core: each check of EmuSen_Cores.md §5.2 refusing, a stopped job and a failed update
//! leaving the installed folder as it was, nothing written outside `/Cores` (§9, C3).

mod common;

use common::*;
use emusen_core_entry::{built_name, library_suffix, sha256_file, Entry, Notice, PREVIOUS, STAGING};
use emusen_core_tool::catalogue;
use emusen_core_tool::install::{self, Note};
use emusen_core_tool::notes::{self, Tested};
use emusen_core_tool::Context;
use emusen_galaxia::archive::ZipWriter;
use serde_json::Value;
use emusen_galaxia::rom_hash::Crc32;
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// An ELF header for this machine, then `body` and enough filler that the download reports progress more than once.
fn library(body: &[u8]) -> Vec<u8> {
    let mut b = vec![0u8; 64];
    b[..4].copy_from_slice(b"\x7fELF");
    b[4] = if cfg!(target_pointer_width = "64") { 2 } else { 1 };
    b[5] = 1;
    let machine: u16 = if cfg!(target_arch = "aarch64") { 183 } else { 62 };
    b[18..20].copy_from_slice(&machine.to_le_bytes());
    b.extend_from_slice(body);
    let mut x = 0x9e37_79b9_7f4a_7c15u64 ^ body.len() as u64;
    b.extend((0..(3u32 << 20)).map(|_| {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        x as u8
    }));
    b
}

/// A zip of the named members, each deflated.
fn zip(dir: &Path, members: &[(&str, &[u8])]) -> Vec<u8> {
    let src = dir.join("members");
    std::fs::create_dir_all(&src).unwrap();
    let at = dir.join("core.zip");
    let _ = std::fs::remove_file(&at);
    let mut w = ZipWriter::create(at.to_str().unwrap()).unwrap();
    for (i, (name, bytes)) in members.iter().enumerate() {
        let file = src.join(format!("m{i}"));
        std::fs::write(&file, bytes).unwrap();
        w.add(name, file.to_str().unwrap(), SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000), false, &|| false).unwrap();
    }
    w.finish().unwrap();
    std::fs::read(at).unwrap()
}

/// The first central directory entry's field at `offset`, overwritten.
fn patch_central(zip: &mut [u8], offset: usize, bytes: &[u8]) {
    let at = zip.windows(4).position(|w| w == [0x50, 0x4b, 0x01, 0x02]).unwrap();
    zip[at + offset..at + offset + bytes.len()].copy_from_slice(bytes);
}

fn crc(bytes: &[u8]) -> u32 {
    let mut c = Crc32::default();
    c.append(bytes);
    c.value()
}

fn core_name(stem: &str) -> String {
    format!("{stem}_libretro.{}", library_suffix())
}

/// The CRC-32 the zip's first central directory entry records, as the buildbot's index gives a core's library; 0 for no zip.
fn member_crc(zip: &[u8]) -> u32 {
    zip.windows(4).position(|w| w == [0x50, 0x4b, 0x01, 0x02]).map_or(0, |at| u32::from_le_bytes(zip[at + 16..at + 20].try_into().unwrap()))
}

/// A buildbot of the test's: the catalogue, and an index whose line for each core is its library's CRC-32 as the zip
/// records it, as the buildbot's is, unless `claim` gives another.
fn buildbot(dir: &Path, cores: &[(&str, &str, Vec<u8>, Option<u32>)]) -> Server {
    let mut infos: Vec<(&str, &str)> = vec![("glonly", GLONLY)];
    let mut index = format!("2026-10-30 0000abcd glonly_libretro.{}.zip\n", library_suffix());
    let mut files = HashMap::new();
    for (stem, date, bytes, claim) in cores {
        infos.push((stem, if *stem == "pcsx2" { LRPS2 } else { BEETLE }));
        index.push_str(&format!("{date} {:08x} {stem}_libretro.{}.zip\n", claim.unwrap_or_else(|| member_crc(bytes)), library_suffix()));
        files.insert(format!("/nightly/linux/x86_64/latest/{stem}_libretro.{}.zip", library_suffix()), bytes.clone());
    }
    files.insert("/assets/frontend/info.zip".into(), info_zip(dir, &infos));
    files.insert("/nightly/linux/x86_64/latest/.index-extended".into(), index.into_bytes());
    serve(files)
}

/// An installation folder with the project's libretro host and, where asked, its proxy, as a build ships them.
fn installation(dir: &Path, adapter: &[u8], broker: bool) -> PathBuf {
    let at = dir.join("Install");
    std::fs::create_dir_all(&at).unwrap();
    std::fs::write(at.join(built_name("emusen_libretro_host")), adapter).unwrap();
    let proxy = at.join(built_name("emusen_libretro_proxy"));
    if broker {
        std::fs::write(proxy, b"the proxy").unwrap();
    } else {
        let _ = std::fs::remove_file(proxy);
    }
    at
}

fn context(dir: &Path, server: &Server) -> Context {
    Context { cores: dir.join("Cores"), source: Some(source(server)), placement: Some(placement()), installation: Some(installation(dir, b"the adapter", true)), notes: Vec::new(), hosted: None }
}

fn fetched(ctx: &Context) {
    let e = events(ctx, r#"{"op":"core-fetch"}"#);
    assert!(e.last().unwrap().get("error").is_none(), "{e:?}");
}

/// Every path under `dir` outside its `Cores` folder, so a test can say nothing was written there.
fn outside_cores(dir: &Path) -> BTreeSet<PathBuf> {
    fn walk(at: &Path, skip: &Path, out: &mut BTreeSet<PathBuf>) {
        for e in std::fs::read_dir(at).unwrap().flatten() {
            let p = e.path();
            if p == skip {
                continue;
            }
            out.insert(p.clone());
            if p.is_dir() {
                walk(&p, skip, out);
            }
        }
    }
    let mut out = BTreeSet::new();
    walk(dir, &dir.join("Cores"), &mut out);
    out
}

fn staging_is_empty(ctx: &Context) -> bool {
    std::fs::read_dir(ctx.cores.join("libretro").join(STAGING)).map_or(true, |mut d| d.next().is_none())
}

fn of<'a>(e: &'a [Value], event: &str) -> Vec<&'a Value> {
    e.iter().filter(|v| text(v, "event") == event).collect()
}

#[cfg(unix)]
fn mode(p: &Path) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(p).unwrap().permissions().mode() & 0o777
}

#[test]
fn get_checks_everything_installs_by_one_rename_and_writes_nothing_outside_cores() {
    let dir = scratch("get");
    let core = library(b"build one");
    let server = buildbot(&dir, &[("pcsx2", "2026-10-31", zip(&dir, &[(&core_name("pcsx2"), &core)]), None)]);
    let mut ctx = context(&dir, &server);
    let sha = {
        std::fs::write(dir.join("probe"), &core).unwrap();
        sha256_file(&dir.join("probe")).unwrap()
    };
    ctx.notes = vec![Tested { stem: "pcsx2".into(), date: "2026-10-31".into(), sha256: sha.clone(), kit: "pass".into(), note: "plays".into() }];
    fetched(&ctx);
    let before = outside_cores(&dir);

    let e = events(&ctx, r#"{"op":"core-get","core":"pcsx2"}"#);
    let checks: Vec<&str> = of(&e, "checked").iter().map(|v| text(v, "check")).collect();
    assert_eq!(checks, ["transport", "crc32", "shape", "size", "members", "architecture", "sha256"]);
    assert!(of(&e, "progress").len() >= 2, "{e:?}");
    assert_eq!(text(of(&e, "plan")[0], "build"), "2026-10-31");
    let done = of(&e, "installed")[0];
    assert_eq!(text(done, "sha256"), sha);
    assert_eq!(done.get("tested").and_then(|t| t.get("match")).and_then(Value::as_str), Some("exact"));
    assert!(e.last().unwrap().get("error").is_none());

    let entry = Entry::new(&ctx.cores, "pcsx2").unwrap();
    let notice = entry.read_notice().unwrap();
    assert_eq!((notice.sha256.as_str(), notice.corename.as_str(), notice.build.as_str()), (sha.as_str(), "LRPS2", "2026-10-31"));
    assert_eq!(entry.verify().unwrap(), sha);
    assert_eq!(std::fs::read_to_string(entry.info_path()).unwrap(), LRPS2);
    assert_eq!(std::fs::read(entry.adapter_path()).unwrap(), b"the adapter");
    assert_eq!(std::fs::read(entry.broker_path()).unwrap(), b"the proxy");
    assert_eq!(std::fs::read_to_string(entry.target_path()).unwrap(), Entry::target_text());
    #[cfg(unix)]
    {
        assert_eq!(mode(&entry.folder), 0o755);
        assert_eq!(mode(&entry.core_path()), 0o644);
        assert_eq!(mode(&entry.notice_path()), 0o644);
    }
    assert!(staging_is_empty(&ctx));
    assert_eq!(outside_cores(&dir), before);

    let again = events(&ctx, r#"{"op":"core-get","core":"pcsx2"}"#);
    assert!(text(of(&again, "refused")[0], "reason").contains("is installed"));
    let row = events(&ctx, r#"{"op":"core-info","core":"pcsx2"}"#);
    assert_eq!(text(&row[0], "state"), "installed");
    assert_eq!(text(&row[0], "sha256"), sha);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn every_check_refuses_and_leaves_no_folder_behind() {
    let dir = scratch("refuse");
    let name = core_name("pcsx2");
    let good = library(b"good");
    let mut wrong_machine = good.clone();
    wrong_machine[18] ^= 0x55;
    let mut link = zip(&dir, &[(&name, &good)]);
    patch_central(&mut link, 5, &[3]);
    patch_central(&mut link, 38, &(0o120777u32 << 16).to_le_bytes());
    let mut declared_huge = zip(&dir, &[(&name, &good)]);
    patch_central(&mut declared_huge, 24, &(600u32 << 20).to_le_bytes());
    let mut inflates_past = zip(&dir, &[(&name, &good)]);
    patch_central(&mut inflates_past, 24, &1000u32.to_le_bytes());
    let mut damaged_crc = zip(&dir, &[(&name, &good)]);
    patch_central(&mut damaged_crc, 16, &0xdead_beefu32.to_le_bytes());
    let whole = zip(&dir, &[(&name, &good)]);
    let cases: Vec<(&str, Vec<u8>, Option<u32>, &str)> = vec![
        ("the index's CRC-32", zip(&dir, &[(&name, &good)]), Some(0x1234_5678), "CRC-32"),
        ("an index giving the whole zip's CRC-32", whole.clone(), Some(crc(&whole)), "CRC-32"),
        ("two members", zip(&dir, &[(&name, &good), ("readme.txt", b"hello")]), None, "holds 2 members"),
        ("a member climbing out", zip(&dir, &[(&format!("../{name}"), &good)]), None, "member is"),
        ("a member in a folder", zip(&dir, &[(&format!("sub/{name}"), &good)]), None, "member is"),
        ("an absolute member", zip(&dir, &[(&format!("/tmp/{name}"), &good)]), None, "member is"),
        ("another core's member", zip(&dir, &[(&core_name("other"), &good)]), None, "member is"),
        ("a link", link, None, "is a link"),
        ("a declared size over the bound", declared_huge, None, "declares"),
        ("bytes inflating past the declared size", inflates_past, None, "damaged"),
        ("a member's CRC-32", damaged_crc, None, "damaged"),
        ("another machine's library", zip(&dir, &[(&name, &wrong_machine)]), None, "not a library for this machine"),
        ("not a zip", b"PK not really".to_vec(), None, "does not read"),
    ];
    for (what, bytes, claim, words) in cases {
        let server = buildbot(&dir, &[("pcsx2", "2026-10-31", bytes, claim)]);
        let ctx = context(&dir, &server);
        let _ = std::fs::remove_dir_all(&ctx.cores);
        fetched(&ctx);
        let e = events(&ctx, r#"{"op":"core-get","core":"pcsx2"}"#);
        let refused = of(&e, "refused");
        assert_eq!(refused.len(), 1, "{what}: {e:?}");
        assert!(text(refused[0], "reason").contains(words), "{what}: {}", text(refused[0], "reason"));
        assert!(of(&e, "installed").is_empty(), "{what}");
        assert!(!ctx.cores.join("libretro/pcsx2").exists(), "{what} left a folder");
        assert!(staging_is_empty(&ctx), "{what} left staging");
    }

    let server = buildbot(&dir, &[("pcsx2", "2026-10-31", b"REDIRECT https://127.0.0.1:1/core.zip".to_vec(), Some(1))]);
    let ctx = context(&dir, &server);
    fetched(&ctx);
    let e = events(&ctx, r#"{"op":"core-get","core":"pcsx2"}"#);
    assert!(text(of(&e, "refused")[0], "reason").contains("on its own host only"), "{e:?}");

    let ctx = Context { installation: Some(dir.join("Empty")), ..ctx };
    std::fs::create_dir_all(dir.join("Empty")).unwrap();
    let asked = server.heads.lock().unwrap().len();
    let e = events(&ctx, r#"{"op":"core-get","cores":["pcsx2","glonly"]}"#);
    let reasons: Vec<&str> = of(&e, "refused").iter().map(|v| text(v, "reason")).collect();
    assert!(reasons[0].contains("libemusen_libretro_host") && reasons[0].contains("could not run"), "{reasons:?}");
    assert!(reasons[1].contains("not supported"), "{reasons:?}");
    assert_eq!(server.heads.lock().unwrap().len(), asked, "a refused plan downloaded");
    assert_eq!(text(e.last().unwrap(), "error"), "2 of 2 refused");
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn a_stopped_update_leaves_the_installed_build_as_it_was() {
    let dir = scratch("stop");
    let name = core_name("pcsx2");
    let server = buildbot(&dir, &[("pcsx2", "2026-10-30", zip(&dir, &[(&name, &library(b"one"))]), None)]);
    let ctx = context(&dir, &server);
    fetched(&ctx);
    assert!(of(&events(&ctx, r#"{"op":"core-get","core":"pcsx2"}"#), "installed").len() == 1);
    let entry = Entry::new(&ctx.cores, "pcsx2").unwrap();
    let was = entry.read_notice().unwrap();

    let newer = buildbot(&dir, &[("pcsx2", "2026-10-31", zip(&dir, &[(&name, &library(b"two"))]), None)]);
    let ctx = Context { source: Some(source(&newer)), ..ctx };
    fetched(&ctx);
    let cat = catalogue::load(&ctx).unwrap();
    let asked = std::cell::Cell::new(0);
    let stop = || {
        asked.set(asked.get() + 1);
        asked.get() > 3
    };
    let r = install::update(&ctx, &cat, "pcsx2", &stop, &mut |_: Note| {});
    assert_eq!(r.unwrap_err(), "stopped");
    assert_eq!(entry.read_notice().unwrap(), was);
    assert_eq!(entry.verify().unwrap(), was.sha256);
    assert!(!ctx.cores.join("libretro").join(PREVIOUS).join("pcsx2").exists());
    assert!(staging_is_empty(&ctx));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn update_keeps_one_previous_build_falls_back_on_failure_and_rollback_swaps() {
    let dir = scratch("update");
    let name = core_name("pcsx2");
    let one = buildbot(&dir, &[("pcsx2", "2026-10-29", zip(&dir, &[(&name, &library(b"one"))]), None)]);
    let mut ctx = context(&dir, &one);
    fetched(&ctx);
    events(&ctx, r#"{"op":"core-get","core":"pcsx2"}"#);
    let entry = Entry::new(&ctx.cores, "pcsx2").unwrap();
    let prev = Entry::at(ctx.cores.join("libretro").join(PREVIOUS).join("pcsx2"), "pcsx2").unwrap();
    let h1 = entry.read_notice().unwrap().sha256;

    let current = events(&ctx, r#"{"op":"core-update","core":"pcsx2"}"#);
    assert_eq!(of(&current, "current").len(), 1, "{current:?}");
    assert!(events(&ctx, r#"{"op":"core-update"}"#).iter().all(|v| text(v, "event") != "installed"));

    let two = buildbot(&dir, &[("pcsx2", "2026-10-30", zip(&dir, &[(&name, &library(b"two"))]), None)]);
    ctx.source = Some(source(&two));
    fetched(&ctx);
    let e = events(&ctx, r#"{"op":"core-update"}"#);
    let i = of(&e, "installed")[0];
    let h2 = text(i, "sha256").to_string();
    assert_eq!((text(i, "previous"), text(i, "build")), (h1.as_str(), "2026-10-30"));
    assert_eq!(i.get("revoke"), Some(&Value::Array(Vec::new())));
    assert_eq!(entry.read_notice().unwrap().sha256, h2);
    assert_eq!(prev.read_notice().unwrap().sha256, h1);

    let three_zip = zip(&dir, &[(&name, &library(b"three"))]);
    let three = buildbot(&dir, &[("pcsx2", "2026-10-31", three_zip, None)]);
    ctx.source = Some(source(&three));
    fetched(&ctx);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let kept = ctx.cores.join("libretro").join(PREVIOUS);
        std::fs::set_permissions(&kept, std::fs::Permissions::from_mode(0o555)).unwrap();
        let failed = events(&ctx, r#"{"op":"core-update","core":"pcsx2"}"#);
        std::fs::set_permissions(&kept, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert_eq!(of(&failed, "refused").len(), 1, "{failed:?}");
        assert_eq!(entry.read_notice().unwrap().sha256, h2, "a failed update did not fall back");
        assert_eq!(prev.read_notice().unwrap().sha256, h1, "a failed update lost the kept build");
        assert!(staging_is_empty(&ctx));
    }
    let e = events(&ctx, r#"{"op":"core-update","core":"pcsx2"}"#);
    let i = of(&e, "installed")[0];
    let h3 = text(i, "sha256").to_string();
    assert_eq!(i.get("revoke"), Some(&Value::Array(vec![Value::String(String::from(&h1))])));
    assert_eq!(prev.read_notice().unwrap().sha256, h2);
    let info = events(&ctx, r#"{"op":"core-info","core":"pcsx2"}"#);
    assert_eq!(info[0].get("previous").and_then(|p| p.get("sha256")).and_then(Value::as_str), Some(h2.as_str()));

    let back = events(&ctx, r#"{"op":"core-rollback","core":"pcsx2"}"#);
    let r = of(&back, "restored")[0];
    assert_eq!((text(r, "sha256"), text(r, "previous")), (h2.as_str(), h3.as_str()));
    assert_eq!(entry.verify().unwrap(), h2);
    events(&ctx, r#"{"op":"core-rollback","core":"pcsx2"}"#);
    assert_eq!(entry.verify().unwrap(), h3);

    std::fs::write(prev.core_path(), b"changed after it was kept").unwrap();
    let refused = events(&ctx, r#"{"op":"core-rollback","core":"pcsx2"}"#);
    assert!(text(of(&refused, "refused")[0], "reason").contains("not put back"));
    assert_eq!(entry.verify().unwrap(), h3);
    std::fs::remove_dir_all(prev.folder.clone()).unwrap();
    assert!(text(of(&events(&ctx, r#"{"op":"core-rollback","core":"pcsx2"}"#), "refused")[0], "reason").contains("no earlier build"));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn remove_takes_the_folder_and_its_kept_build_and_never_the_players_own() {
    let dir = scratch("rm");
    let name = core_name("pcsx2");
    let one = buildbot(&dir, &[("pcsx2", "2026-10-29", zip(&dir, &[(&name, &library(b"one"))]), None)]);
    let mut ctx = context(&dir, &one);
    fetched(&ctx);
    events(&ctx, r#"{"op":"core-get","core":"pcsx2"}"#);
    let two = buildbot(&dir, &[("pcsx2", "2026-10-30", zip(&dir, &[(&name, &library(b"two"))]), None)]);
    ctx.source = Some(source(&two));
    fetched(&ctx);
    let i = events(&ctx, r#"{"op":"core-update"}"#);
    let (h2, h1) = (text(of(&i, "installed")[0], "sha256").to_string(), text(of(&i, "installed")[0], "previous").to_string());
    let saves = dir.join("Home/Saves/pcsx2/game.srm");
    std::fs::create_dir_all(saves.parent().unwrap()).unwrap();
    std::fs::write(&saves, b"a save").unwrap();
    let mine = ctx.cores.join("libretro/mine");
    std::fs::create_dir_all(&mine).unwrap();
    std::fs::write(mine.join(core_name("mine")), b"the player's").unwrap();

    let e = events(&ctx, r#"{"op":"core-rm","cores":["pcsx2","mine"]}"#);
    let removed = of(&e, "removed")[0];
    assert_eq!(removed.get("revoke"), Some(&Value::Array(vec![Value::String(String::from(&h2)), Value::String(String::from(&h1))])));
    assert!(text(of(&e, "refused")[0], "reason").contains("Cores did not make"));
    assert!(!ctx.cores.join("libretro/pcsx2").exists());
    assert!(!ctx.cores.join("libretro").join(PREVIOUS).join("pcsx2").exists());
    assert_eq!(std::fs::read(mine.join(core_name("mine"))).unwrap(), b"the player's");
    assert_eq!(std::fs::read(&saves).unwrap(), b"a save");
    assert!(text(of(&events(&ctx, r#"{"op":"core-rm","core":"pcsx2"}"#), "refused")[0], "reason").contains("not installed"));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn verify_refreshes_the_project_files_and_never_clears_a_changed_core() {
    let dir = scratch("verify");
    let server = buildbot(&dir, &[("pcsx2", "2026-10-31", zip(&dir, &[(&core_name("pcsx2"), &library(b"one"))]), None)]);
    let ctx = context(&dir, &server);
    fetched(&ctx);
    events(&ctx, r#"{"op":"core-get","core":"pcsx2"}"#);
    std::fs::create_dir_all(ctx.cores.join("libretro/mine")).unwrap();
    let entry = Entry::new(&ctx.cores, "pcsx2").unwrap();
    let result = |e: &[Value], stem: &str| of(e, "verified").into_iter().find(|v| text(v, "stem") == stem).map(|v| text(v, "result").to_string());

    let e = events(&ctx, r#"{"op":"core-verify"}"#);
    assert_eq!((result(&e, "pcsx2").as_deref(), result(&e, "mine").as_deref()), (Some("ok"), Some("added")));

    installation(&dir, b"the adapter of a newer EmuSen", true);
    let e = events(&ctx, r#"{"op":"core-verify","core":"pcsx2"}"#);
    let v = of(&e, "verified")[0];
    assert_eq!(text(v, "result"), "refreshed");
    assert_eq!(v.get("files"), Some(&Value::Array(vec![Value::String(String::from(format!("emusen-libretro-host.{}", library_suffix())))])));
    assert_eq!(std::fs::read(entry.adapter_path()).unwrap(), b"the adapter of a newer EmuSen");
    assert_eq!(result(&events(&ctx, r#"{"op":"core-verify","core":"pcsx2"}"#), "pcsx2").as_deref(), Some("ok"));

    std::fs::write(entry.target_path(), "something else\n").unwrap();
    let e = events(&ctx, r#"{"op":"core-verify","core":"pcsx2"}"#);
    assert_eq!(of(&e, "verified")[0].get("files"), Some(&Value::Array(vec![Value::String(String::from("broker.target"))])));
    assert_eq!(std::fs::read_to_string(entry.target_path()).unwrap(), Entry::target_text());

    installation(&dir, b"the adapter of a newer EmuSen", false);
    let e = events(&ctx, r#"{"op":"core-verify","core":"pcsx2"}"#);
    assert!(text(of(&e, "refused")[0], "reason").contains("libemusen_libretro_proxy"), "{e:?}");

    let sha = entry.read_notice().unwrap().sha256;
    std::fs::write(entry.core_path(), b"something else").unwrap();
    std::fs::write(entry.adapter_path(), b"stale").unwrap();
    let e = events(&ctx, r#"{"op":"core-verify","core":"pcsx2"}"#);
    let v = of(&e, "verified")[0];
    assert_eq!(text(v, "result"), "changed");
    assert!(text(v, "reason").contains("needs approval again"));
    assert_eq!(std::fs::read(entry.core_path()).unwrap(), b"something else");
    assert_eq!(std::fs::read(entry.adapter_path()).unwrap(), b"stale", "a changed core's folder was refreshed");
    assert_eq!(entry.read_notice().unwrap().sha256, sha);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn a_second_changing_job_is_refused_while_one_holds_the_lock() {
    let dir = scratch("lock");
    let server = buildbot(&dir, &[("pcsx2", "2026-10-31", zip(&dir, &[(&core_name("pcsx2"), &library(b"one"))]), None)]);
    let ctx = context(&dir, &server);
    fetched(&ctx);
    let held = install::lock(&ctx).unwrap();
    let e = events(&ctx, r#"{"op":"core-get","core":"pcsx2"}"#);
    assert!(text(of(&e, "refused")[0], "reason").contains("another Cores job"));
    drop(held);
    assert_eq!(of(&events(&ctx, r#"{"op":"core-get","core":"pcsx2"}"#), "installed").len(), 1);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn the_tested_builds_are_read_and_matched_by_hash() {
    let a = "a".repeat(64);
    let b = "b".repeat(64);
    let list = notes::parse(&format!("# header\npcsx2\t2026-10-01\t{a}\tpass\tplays\npcsx2\t2026-10-20\t{b}\tpass\tlater\nbad line\nx\t2026-10-01\tnothex\tpass\tskip\n"));
    assert_eq!(list.len(), 2);
    assert_eq!(notes::standing(&list, "pcsx2", Some(&a)).0, notes::Match::Exact);
    let (m, t) = notes::standing(&list, "pcsx2", Some(&"c".repeat(64)));
    assert_eq!((m, t.map(|t| t.date.as_str())), (notes::Match::Other, Some("2026-10-20")));
    assert_eq!(notes::standing(&list, "other", None).0, notes::Match::None);
    assert!(notes::parse(notes::SHIPPED).iter().all(|t| t.sha256.len() == 64));
    let _ = Notice::parse;
}

#[test]
fn a_request_refused_outright_writes_nothing() {
    let dir = scratch("nothing");
    let server = buildbot(&dir, &[("pcsx2", "2026-10-31", zip(&dir, &[(&core_name("pcsx2"), &library(b"one"))]), None)]);
    let ctx = context(&dir, &server);
    for request in [r#"{"op":"core-get","core":"pcsx2"}"#, r#"{"op":"core-rm","core":"pcsx2"}"#, r#"{"op":"core-rollback","core":"pcsx2"}"#, r#"{"op":"core-verify"}"#, r#"{"op":"core-update"}"#] {
        let e = events(&ctx, request);
        assert!(e.last().is_some_and(|d| text(d, "event") == "done"), "{request}");
        assert!(!ctx.cores.exists(), "{request} wrote {}", ctx.cores.display());
    }
    let e = events(&ctx, r#"{"op":"core-get","core":"pcsx2"}"#);
    assert!(text(of(&e, "refused")[0], "reason").contains("never fetched"));
    std::fs::remove_dir_all(&dir).unwrap();
}
