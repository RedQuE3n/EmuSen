//! The wrappers stage 5's screens need, each against the platform library built from this commit in a scratch tree and
//! never the network: the tools' jobs pulled and drained, the cheat database and lists, the packages, the seats,
//! OpenVGDB's bytes and the preset readers (EmuSen_Qt.md §1.1.2, "Stage 5's wrappers").

use emusen_client_fixture::{World, library};
use emusen_platform_client::cheats::CheatsDatabase;
use emusen_platform_client::galaxia::consts::{DIR_CORES, DIR_PACKAGES, MODEL_APP_SETTINGS};
use emusen_platform_client::packages::Roots;
use emusen_platform_client::seats::{Holder, Pad, Seat};
use emusen_platform_client::tools::{Step, ToolJob};
use emusen_platform_client::{Error, Platform, serenity};
use serde_json::{Value, json};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Every event of a job, pulled until it ends, as a front end drains one on a timer.
fn run(p: &Arc<Platform>, request: Value) -> Vec<Value> {
    let job = ToolJob::start(p, &request).unwrap();
    let (mut events, until) = (Vec::new(), Instant::now() + Duration::from_secs(60));
    while !job.drain(&mut events).unwrap() {
        assert!(Instant::now() < until, "{request} did not end: {events:?}");
        if let Step::Event(e) = job.next(50).unwrap() {
            events.push(e);
        }
    }
    assert_eq!(job.next(0).unwrap(), Step::Ended, "an ended job stays ended");
    events
}

fn event(e: &Value) -> &str {
    e["event"].as_str().unwrap_or_default()
}

fn library_at(p: &Platform, folder: &std::path::Path) {
    let mut settings = p.model_new(MODEL_APP_SETTINGS).unwrap();
    settings["RomDirectory"] = json!(folder.to_str().unwrap());
    p.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();
}

#[test]
fn the_library_tools_jobs_are_pulled_to_their_end() {
    let world = World::new("tool-jobs");
    let p = Platform::load(library()).unwrap();
    let roms = world.dir("Roms");
    std::fs::create_dir_all(roms.join("segacd")).unwrap();
    std::fs::write(roms.join("segacd/Disc.iso"), vec![0u8; 2048 * 16]).unwrap();
    std::fs::create_dir_all(roms.join("nes")).unwrap();
    for name in ["a.nes", "b.nes"] {
        std::fs::write(roms.join("nes").join(name), b"NES\x1a the same dump").unwrap();
    }
    library_at(&p, &roms);

    // The disc batch's listing, the packages' and a ROM's check, each from its start to its end.
    let discs = run(&p, json!({ "op": "ls" }));
    assert_eq!((event(&discs[0]), event(discs.last().unwrap())), ("start", "end"), "{discs:?}");
    assert!(discs.iter().any(|e| e.to_string().contains("Disc.iso")), "{discs:?}");
    let packages = run(&p, json!({ "op": "pkg-ls", "paths": [] }));
    assert_eq!(event(packages.last().unwrap()), "end", "{packages:?}");
    let checked = run(&p, json!({ "op": "rom-check", "paths": ["nes/a.nes"] }));
    assert_eq!(event(checked.last().unwrap()), "end", "{checked:?}");
    assert!(checked.iter().any(|e| e.to_string().contains("a.nes")), "{checked:?}");
    let dups = run(&p, json!({ "op": "rom-dups" }));
    assert!(dups.iter().any(|e| e.to_string().contains("a.nes") && e.to_string().contains("b.nes")), "the two dumps are one: {dups:?}");

    // A request that is none is refused with the library's words; a stopped job still ends.
    let Err(Error::Status { words, .. }) = ToolJob::start(&p, &json!({ "op": "no-such-op" })) else { panic!("an unknown op is refused") };
    assert!(words.contains("is not convert, check, ls"), "{words}");
    let job = ToolJob::start(&p, &json!({ "op": "rom-dups" })).unwrap();
    job.stop().unwrap();
    let mut events = Vec::new();
    let until = Instant::now() + Duration::from_secs(30);
    while !job.drain(&mut events).unwrap() {
        assert!(Instant::now() < until);
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn the_core_tools_jobs_list_verify_and_fetch_on_loopback_only() {
    let world = World::new("core-jobs");
    let p = Platform::load(library()).unwrap();
    assert!(p.directory(DIR_CORES).unwrap().unwrap().starts_with(&world.root));

    let listed = run(&p, json!({ "op": "core-ls" }));
    assert!(listed[0]["catalogue"].is_null(), "nothing fetched yet: {listed:?}");
    let verified = run(&p, json!({ "op": "core-verify" }));
    assert!(!verified.is_empty(), "{verified:?}");
    // The buildbot stood in for by a closed loopback port: the fetch is refused with words and nothing leaves the machine.
    let roots = "-----BEGIN CERTIFICATE-----\nAAAA\n-----END CERTIFICATE-----\n";
    let fetched = run(&p, json!({ "op": "core-fetch", "test": { "connect": "127.0.0.1:1", "roots": roots } }));
    assert!(fetched.iter().any(|e| event(e) == "refused" && !e["reason"].as_str().unwrap_or_default().is_empty()), "{fetched:?}");
}

#[test]
fn the_cheat_database_is_asked_and_pruned_and_the_lists_named() {
    let world = World::new("cheats");
    let p = Platform::load(library()).unwrap();
    let db = world.dir("Cheats");
    for (system, game) in [("Nintendo - Super Nintendo Entertainment System", "Mario Paint"), ("Sega - Mega Drive - Genesis", "Sonic")] {
        std::fs::create_dir_all(db.join(system)).unwrap();
        std::fs::write(db.join(system).join(format!("{game}.cht")), "cheats = 0\n").unwrap();
    }
    let database = CheatsDatabase::open(&p, db.to_str().unwrap()).unwrap();
    let entries = database.entries().unwrap();
    assert_eq!(entries.len(), 2, "{entries:?}");
    assert_eq!(database.query(&json!({ "op": "systems" })).unwrap().as_array().unwrap().len(), 2);
    let found = database.query(&json!({ "op": "find", "text": "sonic", "limit": 10 })).unwrap();
    assert_eq!(entries[found[0].as_u64().unwrap() as usize].1, "Sonic");
    assert!(matches!(database.query(&json!({ "op": "no-such" })), Err(Error::Status { .. })));
    let plan = database.prune_plan(&json!({ "keep": ["Sega - Mega Drive - Genesis"] })).unwrap();
    assert_eq!(plan["removing"].as_array().unwrap().len(), 1, "{plan}");
    let removing = json!({ "directory": db.to_str().unwrap(), "removing": [{ "system": plan["removing"][0]["system"], "path": plan["removing"][0]["path"] }] });
    let applied = p.cheats_prune_apply(&removing).unwrap();
    assert_eq!(applied["removed"], 1, "{plan} {applied}");
    assert!(
        !db.join("Nintendo - Super Nintendo Entertainment System").exists() && db.join("Sega - Mega Drive - Genesis/Sonic.cht").exists()
    );

    assert_eq!(p.cheat_names().unwrap(), Vec::<String>::new(), "no list saved yet");
    assert!(p.cheat_name_valid("Infinite lives").unwrap());
    assert!(!p.cheat_name_valid("a/b").unwrap());
    assert_eq!(p.cheat_game_list(Some("SNES"), "/games/Mario Paint.sfc").unwrap(), Some(("Mario Paint".into(), "cheats/SNES".into())));
    assert_eq!(p.cheat_game_list(None, "").unwrap(), None);
}

#[test]
fn a_package_is_written_read_indexed_and_resolved() {
    let world = World::new("packages");
    let p = Platform::load(library()).unwrap();
    let (packages, roms, compressed) = (p.directory(DIR_PACKAGES).unwrap().unwrap(), world.dir("Roms"), world.dir("Compressed"));
    std::fs::create_dir_all(roms.join("NES")).unwrap();
    let dump = b"a game, no header";
    std::fs::write(roms.join("NES/g.nes"), dump).unwrap();
    let manifest = format!(
        r#"{{"format":"emusen-package","version":"1.3","system":"nes","mode":"linked","game":{{"role":"image","where":"library","path":"NES/g.nes","size":{}}},"dump":{{"file":{{"crc32":"{:08x}"}}}},"x-kept":[1.0]}}"#,
        dump.len(),
        crc32(dump)
    );
    let manifest = manifest.as_str();
    let file = packages.join("NES/g.nes/package.json");
    p.package_write(file.to_str().unwrap(), manifest).unwrap();
    assert_eq!(p.package_read(file.to_str().unwrap()).unwrap(), manifest, "every member in its place, the unknown one kept");
    assert_eq!(std::fs::read_to_string(&file).unwrap(), p.package_format(manifest).unwrap());
    let newer = r#"{"format":"emusen-package","version":"2.0"}"#;
    assert!(matches!(p.package_format(newer), Err(Error::Status { name: "EMUSEN_PLATFORM_NEWER_FILE", .. })));

    let roots = Roots { packages: packages.to_str().unwrap(), library: roms.to_str(), compressed: compressed.to_str().unwrap() };
    let index = p.package_index(roots).unwrap();
    let game = roms.join("NES/g.nes");
    assert_eq!(index["packages"][0]["game"], json!(game.to_str().unwrap()), "{index}");
    let resolved = p.package_resolve(file.parent().unwrap().to_str().unwrap(), roots).unwrap();
    assert_eq!((resolved["game"].as_str(), resolved["moved"].as_bool()), (game.to_str(), Some(false)), "{resolved}");
    // The game moved: found by its size and hashes, its manifest rewritten, and the move said once.
    let moved = roms.join("NES/h.nes");
    std::fs::rename(&game, &moved).unwrap();
    let found = p.package_resolve(moved.to_str().unwrap(), roots).unwrap();
    assert_eq!((found["game"].as_str(), found["moved"].as_bool()), (moved.to_str(), Some(true)), "{found}");
    assert!(p.package_read(file.to_str().unwrap()).unwrap().contains("NES/h.nes"));
}

/// CRC-32 (IEEE), as a manifest records a dump's.
fn crc32(bytes: &[u8]) -> u32 {
    !bytes.iter().fold(!0u32, |c, b| (0..8).fold(c ^ u32::from(*b), |c, _| if c & 1 == 1 { (c >> 1) ^ 0xEDB8_8320 } else { c >> 1 }))
}

#[test]
fn a_pad_takes_its_kept_seat_and_moves_trade_seats() {
    let _world = World::new("seats");
    let p = Platform::load(library()).unwrap();
    let pad = |guid: &str| Pad { guid: guid.into(), path: Some(format!("/dev/input/{guid}")) };
    let mut seats: [Seat; 8] = Default::default();
    assert_eq!(p.seat_for(&seats, &pad("a")).unwrap(), 1, "the lowest free seat");
    seats[0] = Seat::Open(pad("a"));
    seats[2] = Seat::Closed(pad("c"));
    assert_eq!(p.seat_for(&seats, &pad("c")).unwrap(), 3, "a pad come back takes its kept seat");
    assert_eq!(p.seat_for(&seats, &pad("d")).unwrap(), 2);
    assert_eq!(p.seat_highest(&seats).unwrap(), 3);
    assert!(p.seat_forget(&seats, 3).unwrap() && !p.seat_forget(&seats, 2).unwrap());
    let moved = p.seat_move(&seats, true, 1, 3).unwrap().expect("the seats changed");
    assert_eq!(&moved[..3], &[Holder::Seat(2), Holder::Empty, Holder::Mover], "player 1 and the kept seat traded");
    assert_eq!(p.seat_move(&seats, true, 1, 1).unwrap(), None, "a move to its own seat changes nothing");
    assert!(matches!(p.seat_move(&seats, true, 1, 9), Err(Error::Status { .. })));
}

#[test]
fn openvgdb_hashes_a_dump_without_its_header_and_presets_are_read_and_merged() {
    let world = World::new("openvgdb-presets");
    let p = Platform::load(library()).unwrap();
    let mut nes = b"NES\x1a".to_vec();
    nes.extend([0u8; 12]);
    nes.push(7);
    assert_eq!(p.openvgdb_bytes("nes", &nes).unwrap(), Some(vec![7]));
    assert_eq!(p.openvgdb_bytes("psx", &nes).unwrap(), None, "the file's own bytes");

    let preset = world.root.join("plain.slangp");
    std::fs::write(&preset, "shaders = 1\nshader0 = plain.slang\nfilter_linear0 = true\n").unwrap();
    let read = serenity::preset_load(&p, preset.to_str().unwrap()).unwrap();
    assert_eq!((read["passes"].as_array().map(Vec::len), read["passes"][0]["linear"].as_bool()), (Some(1), Some(true)), "{read}");
    let missing = serenity::preset_load(&p, world.root.join("none.slangp").to_str().unwrap()).unwrap();
    assert_eq!(missing["fail"], "not_found", "a failed read is a document: {missing}");
    let one = |initial: f32| json!({ "id": "glow", "description": "Glow", "initial": initial.to_bits(), "minimum": 0f32.to_bits(), "maximum": 1f32.to_bits(), "step": 0.1f32.to_bits() });
    let merged = serenity::parameters_merge(&p, &json!({ "sources": [[one(0.5)], [one(0.25)]], "overrides": [] })).unwrap();
    assert_eq!(merged.as_array().unwrap().len(), 1, "the first declaration wins: {merged}");
    assert_eq!(merged[0]["initial"], json!(0.5f32.to_bits()));
    assert!(matches!(serenity::parameters_merge(&p, &json!([])), Err(Error::Status { .. })));
}
