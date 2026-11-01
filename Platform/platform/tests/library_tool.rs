//! The library tool through this library's two surfaces (EmuSen_GamePackages.md §12.3): a shell this library makes is
//! handed the `emusen.library` service, so that `chd` runs with no help from the host, reading the library from the
//! settings and writing only under the tree's `Compressed` and `Packages`; and the C calls refuse what is not a request.
//! The conversion needs `EMUSEN_CHDMAN` naming the pinned chdman; without it only `chd ls` and the refusals run.

use emusen_disc::ecma130::{self, RAW};
use emusen_galaxia::tree::{self, Override};
use emusen_platform::dianash::{dianash_free, dianash_new, dianash_output_take, dianash_session_new, dianash_submit};
use emusen_platform::library_tool::{emusen_tool_job_free, emusen_tool_job_next, emusen_tool_job_start, LibraryJob};
use std::path::PathBuf;

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

/// The tree is the process's: a test that moves it holds this.
static TREE: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn run(sh: *mut emusen_platform::dianash::DianashShell, line: &str) -> String {
    unsafe {
        assert_eq!(dianash_submit(sh, 1, line.as_ptr(), line.len()), 0, "{line}");
        let mut out = vec![0u8; 1 << 16];
        let n = dianash_output_take(sh, out.as_mut_ptr(), out.len());
        String::from_utf8_lossy(&out[..n.max(0) as usize]).into_owned()
    }
}

#[test]
fn chd_runs_in_a_shell_this_library_makes_over_the_settings_library() {
    let _tree = TREE.lock().unwrap_or_else(|p| p.into_inner());
    let base = std::env::var("TMPDIR").map(PathBuf::from).unwrap_or_else(|_| std::env::temp_dir()).join(format!("emusen-platform-chd-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let (library, home, etc) = (base.join("library"), base.join("home"), base.join("etc"));
    std::fs::create_dir_all(library.join("psx")).unwrap();
    std::fs::create_dir_all(&etc).unwrap();
    std::fs::write(library.join("psx/Game (Track 1).bin"), data(30)).unwrap();
    std::fs::write(etc.join("appsettings.json"), format!("{{\"RomDirectory\": \"{}\"}}", library.display())).unwrap();
    tree::tree_mut().set_override(Override::Data, Some(home.to_string_lossy().into_owned()));
    tree::tree_mut().set_override(Override::Config, Some(etc.to_string_lossy().into_owned()));
    let sh = unsafe { dianash_new(std::ptr::null()) };
    assert!(!sh.is_null());
    assert_eq!(unsafe { dianash_session_new(sh) }, 1);
    let listed = run(sh, "chd ls psx");
    assert!(listed.contains("not converted psx/Game (Track 1).bin"), "{listed}");
    if let Ok(chdman) = std::env::var("EMUSEN_CHDMAN") {
        assert!(!chdman.is_empty());
        let converted = run(sh, "chd convert psx");
        assert!(converted.contains("[1/1] psx/Game (Track 1).bin: converted"), "{converted}");
        assert!(home.join("Compressed/psx/Game (Track 1).chd").is_file());
        assert!(home.join("Packages/PlayStation/Game (Track 1).chd/package.json").is_file());
        let checked = run(sh, "chd check");
        assert!(checked.contains("[1/1] psx/Game (Track 1).bin: verified, three checks passed"), "{checked}");
        assert!(run(sh, "chd ls").contains("converted     psx/Game (Track 1).bin"));
    } else {
        eprintln!("skipped the conversion: EMUSEN_CHDMAN names no chdman");
    }
    let usage = run(sh, "chd frobnicate");
    assert!(usage.contains("Usage: chd ls"), "{usage}");
    unsafe { dianash_free(sh) };
    let names: Vec<String> = std::fs::read_dir(library.join("psx")).unwrap().flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
    assert_eq!(names, ["Game (Track 1).bin"], "the library was written");
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn a_request_that_is_not_one_is_refused_and_a_job_ends_with_zero() {
    let mut job: *mut LibraryJob = std::ptr::null_mut();
    let bad = br#"{"op": "erase"}"#;
    assert_eq!(unsafe { emusen_tool_job_start(bad.as_ptr(), bad.len(), &mut job) }, -1025);
    let ls = br#"{"op": "ls", "paths": ["nowhere"]}"#;
    assert_eq!(unsafe { emusen_tool_job_start(ls.as_ptr(), ls.len(), &mut job) }, 0);
    let mut events = Vec::new();
    let mut out = vec![0u8; 4096];
    loop {
        let n = unsafe { emusen_tool_job_next(job, 1000, out.as_mut_ptr(), out.len()) };
        if n == 0 {
            break;
        }
        if n > 0 {
            events.push(String::from_utf8_lossy(&out[..n as usize]).into_owned());
        }
    }
    assert!(events.last().unwrap().contains("\"event\":\"end\""), "{events:?}");
    assert_eq!(unsafe { emusen_tool_job_free(job) }, 0);
}

/// A job longer than the line's budget goes on in the background: `ps` lists it as `chd1` until it ends, and what it
/// made is there afterwards. The budget is made 1 ms here so that any job outlives it.
#[test]
fn a_job_past_the_budget_goes_on_in_the_background_and_ps_lists_it() {
    let Ok(chdman) = std::env::var("EMUSEN_CHDMAN") else {
        eprintln!("skipped: EMUSEN_CHDMAN names no chdman");
        return;
    };
    assert!(!chdman.is_empty());
    let _tree = TREE.lock().unwrap_or_else(|p| p.into_inner());
    let base = std::env::var("TMPDIR").map(PathBuf::from).unwrap_or_else(|_| std::env::temp_dir()).join(format!("emusen-platform-chd-bg-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let (library, home, etc) = (base.join("library"), base.join("home"), base.join("etc"));
    std::fs::create_dir_all(library.join("psx")).unwrap();
    std::fs::create_dir_all(&etc).unwrap();
    std::fs::write(library.join("psx/Slow (Track 1).bin"), data(2000)).unwrap();
    std::fs::write(etc.join("appsettings.json"), format!("{{\"RomDirectory\": \"{}\"}}", library.display())).unwrap();
    tree::tree_mut().set_override(Override::Data, Some(home.to_string_lossy().into_owned()));
    tree::tree_mut().set_override(Override::Config, Some(etc.to_string_lossy().into_owned()));
    let config = emusen_dianash::abi::dianash_config { size: std::mem::size_of::<emusen_dianash::abi::dianash_config>() as u32, flags: emusen_dianash::abi::CONFIG_SESSIONS, root: emusen_dianash::abi::dianash_bytes::ABSENT, budget_ms: 1 };
    let sh = unsafe { dianash_new(&config) };
    assert_eq!(unsafe { dianash_session_new(sh) }, 1);
    let said = run(sh, "chd convert psx");
    assert!(said.contains("chd1 goes on in the background"), "{said}");
    assert!(run(sh, "ps").contains("chd1"));
    let mut waited = 0;
    while run(sh, "ps").contains("chd1") {
        std::thread::sleep(std::time::Duration::from_millis(100));
        waited += 1;
        assert!(waited < 600, "the job did not end");
    }
    assert!(home.join("Packages/PlayStation/Slow (Track 1).chd/package.json").is_file());
    unsafe { dianash_free(sh) };
    let _ = std::fs::remove_dir_all(&base);
}

/// `rom dups` and the pipe the shell can run instead, `rom hash -r --comparable / | sort | uniq -D -w 40`, give the
/// same lines; `rom check` passes a sound game. The words the module prints to standard error come in the output too.
#[test]
fn rom_dups_is_the_pipe_of_rom_hash_sort_and_uniq_in_a_shell_this_library_makes() {
    let _tree = TREE.lock().unwrap_or_else(|p| p.into_inner());
    let base = std::env::var("TMPDIR").map(PathBuf::from).unwrap_or_else(|_| std::env::temp_dir()).join(format!("emusen-platform-rom-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let (library, home, etc) = (base.join("library"), base.join("home"), base.join("etc"));
    std::fs::create_dir_all(library.join("nes")).unwrap();
    std::fs::create_dir_all(library.join("snes")).unwrap();
    std::fs::create_dir_all(&etc).unwrap();
    let body: Vec<u8> = (0..16384u32).map(|i| (i * 31 / 7) as u8).collect();
    let mut headered = b"NES\x1A\x01\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec();
    headered.extend(&body);
    std::fs::write(library.join("nes/A.nes"), &headered).unwrap();
    std::fs::write(library.join("nes/B.nes"), &body).unwrap();
    std::fs::write(library.join("snes/C.sfc"), vec![7u8; 4096]).unwrap();
    std::fs::write(etc.join("appsettings.json"), format!("{{\"RomDirectory\": \"{}\"}}", library.display())).unwrap();
    tree::tree_mut().set_override(Override::Data, Some(home.to_string_lossy().into_owned()));
    tree::tree_mut().set_override(Override::Config, Some(etc.to_string_lossy().into_owned()));
    let sh = unsafe { dianash_new(std::ptr::null()) };
    assert_eq!(unsafe { dianash_session_new(sh) }, 1);
    let dups = run(sh, "rom dups");
    let groups: Vec<&str> = dups.lines().filter(|l| !l.starts_with("rom dups:")).collect();
    assert_eq!(groups.len(), 2, "{dups}");
    assert!(groups[0].ends_with(" nes/A.nes") && groups[1].ends_with(" nes/B.nes"), "{dups}");
    let piped = run(sh, "rom hash -r --comparable / | sort | uniq -D -w 40");
    assert_eq!(piped.lines().collect::<Vec<_>>(), groups, "{piped}");
    let checked = run(sh, "rom check -r nes");
    assert!(checked.contains("nes/A.nes: ok") && checked.contains("rom check: 2 ok"), "{checked}");
    assert!(run(sh, "rom frobnicate").contains("Usage: rom hash"));
    unsafe { dianash_free(sh) };
    let _ = std::fs::remove_dir_all(&base);
}

/// A catalogue as `cores fetch` caches it, written by the test: two cores of the hosted list offered, and one of them installed.
fn seed_cores(cores: &std::path::Path) {
    use emusen_core_entry::{library_suffix, sha256_file, Entry, Notice};
    let cat = cores.join("libretro/.catalogue");
    std::fs::create_dir_all(&cat).unwrap();
    let info = cat.join("pcee2_libretro.info");
    std::fs::write(&info, "display_name = \"Sony - PlayStation 2 (LRPS2)\"\ncorename = \"LRPS2\"\nlicense = \"GPLv3\"\ndisplay_version = \"2.3.1\"\nfirmware_count = 1\nfirmware0_desc = \"PlayStation 2 BIOS\"\nfirmware0_path = \"pcsx2/bios/ps2.bin\"\n").unwrap();
    let mut zip = emusen_galaxia::archive::ZipWriter::create(cat.join("info.zip").to_str().unwrap()).unwrap();
    zip.add("pcee2_libretro.info", info.to_str().unwrap(), std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_700_000_000), false, &|| false).unwrap();
    zip.finish().unwrap();
    std::fs::remove_file(&info).unwrap();
    let s = library_suffix();
    std::fs::write(cat.join("index-linux-x86_64"), format!("2026-10-31 3a224a58 pcee2_libretro.{s}.zip\n2026-10-30 11112222 flycast_libretro.{s}.zip\n")).unwrap();
    std::fs::write(cat.join("fetched"), "2026-10-31\n").unwrap();
    let e = Entry::new(cores, "flycast").unwrap();
    std::fs::create_dir_all(&e.folder).unwrap();
    std::fs::write(e.core_path(), b"a core").unwrap();
    let n = Notice {
        stem: "flycast".into(),
        name: "Flycast".into(),
        corename: "Flycast".into(),
        authors: vec!["Someone".into()],
        license: "MIT".into(),
        source: format!("https://buildbot.libretro.com/nightly/linux/x86_64/latest/flycast_libretro.{s}.zip"),
        build: "2026-10-29".into(),
        index_crc32: 0x1111_2222,
        sha256: sha256_file(&e.core_path()).unwrap(),
        fetched: "2026-10-29".into(),
        extra: Vec::new(),
    };
    std::fs::write(e.notice_path(), n.to_text().unwrap()).unwrap();
}

#[test]
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn cores_runs_in_a_shell_this_library_makes_over_the_cached_catalogue() {
    let _tree = TREE.lock().unwrap_or_else(|p| p.into_inner());
    let base = std::env::var("TMPDIR").map(PathBuf::from).unwrap_or_else(|_| std::env::temp_dir()).join(format!("emusen-platform-cores-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let (home, etc) = (base.join("home"), base.join("etc"));
    std::fs::create_dir_all(&etc).unwrap();
    tree::tree_mut().set_override(Override::Data, Some(home.to_string_lossy().into_owned()));
    tree::tree_mut().set_override(Override::Config, Some(etc.to_string_lossy().into_owned()));
    let cores = PathBuf::from(tree::tree().directory(tree::Directory::Cores).unwrap());
    seed_cores(&cores);
    let sh = unsafe { dianash_new(std::ptr::null()) };
    assert_eq!(unsafe { dianash_session_new(sh) }, 1);
    let listed = run(sh, "cores ls");
    let lines: Vec<Vec<&str>> = listed.lines().map(|l| l.split_whitespace().collect()).collect();
    assert!(lines.contains(&vec!["available", "ps2", "pcee2", "libretro", "2.3.1", "-", "GPLv3"]), "{listed}");
    assert!(lines.iter().any(|l| l[..3] == ["update", "dreamcast,naomi,atomiswave", "flycast"]), "{listed}");
    let updates = run(sh, "cores ls --updates");
    assert_eq!(updates.lines().map(|l| l.split_whitespace().nth(2).unwrap_or("")).collect::<Vec<_>>(), ["flycast"], "{updates}");
    assert_eq!(run(sh, "cores ls | grep -c libretro").trim(), "2");
    let info = run(sh, "cores info pcee2");
    assert!(info.contains("corename: LRPS2\n") && info.contains("firmware: PlayStation 2 BIOS (pcsx2/bios/ps2.bin, required)"), "{info}");
    assert!(info.contains("from: https://buildbot.libretro.com/nightly/linux/x86_64/latest/pcee2_libretro.so.zip"), "{info}");
    assert!(run(sh, "cores info nothere").contains("cores: no core nothere"));
    assert!(run(sh, "cores frobnicate").contains("Usage: cores ls"));
    unsafe { dianash_free(sh) };
    let _ = std::fs::remove_dir_all(&base);
}
