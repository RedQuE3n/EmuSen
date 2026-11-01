//! The core tool against a catalogue and index made by the test and served by its own TLS server on 127.0.0.1, with
//! certificates made at test time; never the internet (EmuSen_Cores.md §9, C2).

mod common;

use common::*;
use emusen_core_entry::{library_suffix, sha256_file, Entry, Notice};
use emusen_core_tool::catalogue::{self, State};
use emusen_core_tool::info::CoreInfo;
use emusen_core_tool::{index, job, Context, Source};
use emusen_galaxia::json::Value;
use std::collections::HashMap;
use std::path::Path;

fn index_text() -> String {
    let s = library_suffix();
    format!("2026-10-31 3a224a58 pcsx2_libretro.{s}.zip\n2026-10-29 11112222 mednafen_psx_libretro.{s}.zip\n2026-10-30 0000abcd glonly_libretro.{s}.zip\n2026-10-30 0000abce stranger_libretro.{s}.zip\n2026-10-30 0000abcf noinfo_libretro.{s}.zip\nnot a line\n2026-10-30 zzzzzzzz bad_libretro.{s}.zip\n2026-10-30 12345678 RetroArch.7z\n")
}

fn files(dir: &Path) -> HashMap<String, Vec<u8>> {
    let zip = info_zip(dir, &[("pcsx2", LRPS2), ("mednafen_psx", BEETLE), ("glonly", GLONLY), ("stranger", STRANGER), ("unindexed", BEETLE)]);
    HashMap::from([("/assets/frontend/info.zip".to_string(), zip), ("/nightly/linux/x86_64/latest/.index-extended".to_string(), index_text().into_bytes())])
}

fn install(cores: &Path, stem: &str, crc: u32, build: &str) {
    let e = Entry::new(cores, stem).unwrap();
    std::fs::create_dir_all(&e.folder).unwrap();
    std::fs::write(e.core_path(), b"a core").unwrap();
    std::fs::write(e.info_path(), BEETLE).unwrap();
    let n = Notice {
        stem: stem.into(),
        name: "Sony - PlayStation (Beetle PSX)".into(),
        corename: "Beetle PSX".into(),
        authors: vec!["Mednafen Team".into()],
        license: "GPLv2".into(),
        source: format!("https://buildbot.libretro.com/nightly/linux/x86_64/latest/{stem}_libretro.{}.zip", library_suffix()),
        build: build.into(),
        index_crc32: crc,
        sha256: sha256_file(&e.core_path()).unwrap(),
        fetched: "2026-10-29".into(),
        extra: Vec::new(),
    };
    std::fs::write(e.notice_path(), n.to_text().unwrap()).unwrap();
}

#[test]
fn an_info_file_reads_as_libretro_writes_it() {
    let i = CoreInfo::parse("pcsx2", LRPS2);
    assert_eq!(i.name(), "Sony - PlayStation 2 (LRPS2)");
    assert_eq!(i.get("corename"), Some("LRPS2"));
    assert_eq!(i.list("authors"), ["PCSX2 Team", "libretro"]);
    assert!(i.flag("needs_fullpath"));
    let f = i.firmware();
    assert_eq!(f.len(), 2);
    assert_eq!((f[0].desc.as_str(), f[0].path.as_str(), f[0].optional), ("PlayStation 2 BIOS", "pcsx2/bios/ps2.bin", false));
    assert_eq!((f[1].desc.as_str(), f[1].optional), ("pcsx2/extra.bin", true));
    assert_eq!(i.unsupported(), None);
    assert!(CoreInfo::parse("glonly", GLONLY).unsupported().unwrap().contains("OpenGL Core >= 3.3 or OpenGL ES >= 3.0"));
    let odd = CoreInfo::parse("x", "a = unquoted # trailing\nb = \"has # inside\"\nbad line\na = \"second\"\nc = \"open quote\n");
    assert_eq!((odd.get("a"), odd.get("b"), odd.get("c")), (Some("unquoted"), Some("has # inside"), Some("open quote")));
}

#[test]
fn the_index_keeps_this_platforms_core_zips_and_counts_the_rest() {
    let (lines, skipped) = index::parse(&index_text());
    assert_eq!(lines.iter().map(|l| l.stem.as_str()).collect::<Vec<_>>(), ["pcsx2", "mednafen_psx", "glonly", "stranger", "noinfo"]);
    assert_eq!((lines[0].date.as_str(), lines[0].crc32), ("2026-10-31", 0x3a22_4a58));
    assert_eq!(skipped, 3);
}

#[test]
fn fetch_caches_the_catalogue_and_the_listing_joins_it_with_what_is_installed() {
    let dir = scratch("fetch");
    let server = serve(files(&dir));
    let cores = dir.join("Cores");
    let ctx = Context { cores: cores.clone(), source: Some(source(&server)), placement: Some(placement()), installation: None, notes: Vec::new(), hosted: None };

    let before = events(&ctx, r#"{"op":"core-ls"}"#);
    assert_eq!(before.len(), 1);
    assert!(before[0].get("catalogue").unwrap().is_null());

    let fetched = events(&ctx, r#"{"op":"core-fetch"}"#);
    let last = fetched.last().unwrap();
    assert_eq!(text(last, "event"), "done");
    assert!(last.get("error").is_none(), "{last:?}");
    let f = fetched.iter().find(|e| text(e, "event") == "fetched").unwrap();
    assert_eq!(f.get("cores"), Some(&Value::number(5)));
    assert_eq!(f.get("skipped"), Some(&Value::number(3)));
    assert!(catalogue::folder(&cores).join("info.zip").exists());
    assert!(catalogue::folder(&cores).join("index-linux-x86_64").exists());

    install(&cores, "mednafen_psx", 0x1111_2222, "2026-10-29");
    std::fs::create_dir_all(cores.join("libretro/mine")).unwrap();
    let rows = events(&ctx, r#"{"op":"core-ls"}"#);
    let state: HashMap<&str, &str> = rows.iter().filter(|e| text(e, "event") == "core").map(|e| (text(e, "stem"), text(e, "state"))).collect();
    assert_eq!(state, HashMap::from([("pcsx2", "available"), ("mednafen_psx", "installed"), ("glonly", "unsupported"), ("stranger", "unsupported"), ("mine", "added")]));
    let lrps2 = rows.iter().find(|e| text(e, "stem") == "pcsx2").unwrap();
    assert_eq!((text(lrps2, "corename"), text(lrps2, "license"), text(lrps2, "version")), ("LRPS2", "GPLv3", "2.3.1"));
    assert_eq!(lrps2.get("systems"), Some(&Value::Array(vec![Value::string("ps2")])));
    let stranger = rows.iter().find(|e| text(e, "stem") == "stranger").unwrap();
    assert_eq!(text(stranger, "reason"), "no EmuSen console runs this system");
    assert!(text(rows.iter().find(|e| text(e, "stem") == "glonly").unwrap(), "reason").contains("Vulkan only"));

    install(&cores, "mednafen_psx", 0x0bad_0bad, "2026-10-20");
    let updates = events(&ctx, r#"{"op":"core-ls","show":"updates"}"#);
    assert_eq!(updates.iter().filter(|e| text(e, "event") == "core").map(|e| text(e, "stem")).collect::<Vec<_>>(), ["mednafen_psx"]);
    let ps2 = events(&ctx, r#"{"op":"core-ls","console":"ps2"}"#);
    assert_eq!(ps2.iter().filter(|e| text(e, "event") == "core").map(|e| text(e, "stem")).collect::<Vec<_>>(), ["pcsx2"]);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn info_gives_the_detail_and_names_where_the_build_comes_from() {
    let dir = scratch("info");
    let server = serve(files(&dir));
    let cores = dir.join("Cores");
    let ctx = Context { cores: cores.clone(), source: Some(source(&server)), placement: Some(placement()), installation: None, notes: Vec::new(), hosted: None };
    events(&ctx, r#"{"op":"core-fetch"}"#);
    let e = events(&ctx, r#"{"op":"core-info","core":"pcsx2"}"#);
    let d = &e[0];
    assert_eq!(text(d, "from"), format!("https://localhost:{}/nightly/linux/x86_64/latest/pcsx2_libretro.{}.zip", server.port, library_suffix()));
    assert_eq!(text(d, "index_crc32"), "3a224a58");
    assert_eq!(d.get("needs_fullpath"), Some(&Value::Bool(true)));
    let Some(Value::Array(fw)) = d.get("firmware") else { panic!("{d:?}") };
    assert_eq!(fw.len(), 2);
    assert_eq!(d.get("notes"), Some(&Value::Array(vec![Value::string("(!) needs a BIOS"), Value::string("disc images by path")])));

    let detailed = events(&ctx, r#"{"op":"core-ls","detail":true}"#);
    let row = detailed.iter().find(|e| text(e, "stem") == "pcsx2").unwrap();
    assert_eq!(row, d, "core-ls with detail gives core-info's event");
    assert!(events(&ctx, r#"{"op":"core-ls"}"#).iter().all(|e| e.get("firmware").is_none()));

    let missing = events(&ctx, r#"{"op":"core-info","core":"nothere"}"#);
    assert_eq!(text(&missing[0], "event"), "refused");
    assert!(text(&missing[0], "reason").contains("nothere"));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn a_placement_in_the_request_replaces_none_and_without_one_nothing_is_called_unsupported_for_its_system() {
    let dir = scratch("placement");
    let server = serve(files(&dir));
    let ctx = Context { cores: dir.join("Cores"), source: Some(source(&server)), placement: None, installation: None, notes: Vec::new(), hosted: None };
    events(&ctx, r#"{"op":"core-fetch"}"#);
    let plain = events(&ctx, r#"{"op":"core-ls"}"#);
    assert_eq!(text(plain.iter().find(|e| text(e, "stem") == "stranger").unwrap(), "state"), "available");
    let placed = events(&ctx, r#"{"op":"core-ls","systems":{"Sony - PlayStation 2":"ps2","playstation":["psx"]}}"#);
    let row = |s: &str| placed.iter().find(|e| text(e, "stem") == s).unwrap().clone();
    assert_eq!(row("pcsx2").get("systems"), Some(&Value::Array(vec![Value::string("ps2")])));
    assert_eq!(text(&row("mednafen_psx"), "state"), "unsupported");
    assert_eq!(text(&row("stranger"), "state"), "unsupported");
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn a_failed_fetch_keeps_the_cached_catalogue_and_says_why() {
    let dir = scratch("failed");
    let mut f = files(&dir);
    let server = serve(f.clone());
    let cores = dir.join("Cores");
    let ctx = Context { cores: cores.clone(), source: Some(source(&server)), placement: None, installation: None, notes: Vec::new(), hosted: None };
    events(&ctx, r#"{"op":"core-fetch"}"#);
    let cached = std::fs::read(catalogue::folder(&cores).join("info.zip")).unwrap();

    f.insert("/assets/frontend/info.zip".into(), b"not a zip at all".to_vec());
    let broken = Context { source: Some(source(&serve(f.clone()))), ..ctx.clone() };
    let e = events(&broken, r#"{"op":"core-fetch"}"#);
    assert_eq!(text(&e[1], "event"), "refused");
    assert!(text(&e[1], "reason").contains("info.zip does not read"));
    assert_eq!(std::fs::read(catalogue::folder(&cores).join("info.zip")).unwrap(), cached);

    f.insert("/assets/frontend/info.zip".into(), b"REDIRECT https://127.0.0.1:1/info.zip".to_vec());
    let away = Context { source: Some(source(&serve(f.clone()))), ..ctx.clone() };
    let e = events(&away, r#"{"op":"core-fetch"}"#);
    assert!(text(&e[1], "reason").contains("on its own host only"), "{:?}", e[1]);

    let offline = Context { source: Some(Source { offline: true, ..source(&server) }), ..ctx.clone() };
    let e = events(&offline, r#"{"op":"core-fetch"}"#);
    assert!(text(&e[1], "reason").contains("allows no network"));
    let none = Context { source: None, ..ctx };
    assert!(text(&events(&none, r#"{"op":"core-fetch"}"#)[1], "reason").contains("no buildbot listing"));
    assert_eq!(std::fs::read(catalogue::folder(&cores).join("info.zip")).unwrap(), cached);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn only_the_core_tools_requests_are_taken() {
    assert!(job::is_core_op("core-ls") && !job::is_core_op("pkg-ls"));
    for bad in [r#"{"op":"pkg-ls"}"#, r#"{"op":"core-info"}"#, r#"{"op":"core-ls","show":"some"}"#, r#"{"op":"core-ls","systems":3}"#, "not json"] {
        assert!(job::request(bad).is_err(), "{bad}");
    }
    let _ = State::Installed;
}

#[test]
fn only_the_hosted_cores_are_offered_and_an_installed_one_off_the_list_is_not_supported() {
    let dir = scratch("hosted");
    let server = serve(files(&dir));
    let cores = dir.join("Cores");
    let hosted = vec![emusen_core_tool::Hosted { stem: "pcsx2".into(), systems: vec!["ps2".into()] }];
    let ctx = Context { cores: cores.clone(), source: Some(source(&server)), placement: None, installation: None, notes: Vec::new(), hosted: Some(hosted) };
    events(&ctx, r#"{"op":"core-fetch"}"#);
    install(&cores, "mednafen_psx", 0x1111_2222, "2026-10-29");
    let rows = events(&ctx, r#"{"op":"core-ls"}"#);
    let listed: Vec<(&str, &str)> = rows.iter().filter(|e| text(e, "event") == "core").map(|e| (text(e, "stem"), text(e, "state"))).collect();
    assert_eq!(listed, [("mednafen_psx", "unsupported"), ("pcsx2", "available")]);
    let beetle = rows.iter().find(|e| text(e, "stem") == "mednafen_psx").unwrap();
    assert_eq!(text(beetle, "reason"), "mednafen_psx is not one of the libretro cores EmuSen hosts (pcsx2)");
    let lrps2 = rows.iter().find(|e| text(e, "stem") == "pcsx2").unwrap();
    assert_eq!(lrps2.get("systems"), Some(&Value::Array(vec![Value::string("ps2")])));
    let refused = events(&ctx, r#"{"op":"core-get","core":"glonly"}"#);
    assert!(text(&refused[0], "reason").contains("not one of the libretro cores EmuSen hosts"), "{:?}", refused[0]);
    let refused = events(&ctx, r#"{"op":"core-update","core":"mednafen_psx"}"#);
    assert!(text(&refused[0], "reason").contains("not one of the libretro cores EmuSen hosts"), "{:?}", refused[0]);
    std::fs::remove_dir_all(&dir).unwrap();
}
