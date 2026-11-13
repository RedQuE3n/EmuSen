//! `scrape` in a shell this library makes, over the scraper the test opens as a front end does, answered by a recorded-response server on 127.0.0.1 - see EmuSen_LibraryTools.md §8.

use emusen_galaxia::tree::{self, Override};
use emusen_library::Library;
use emusen_platform::dianash::{dianash_free, dianash_new, dianash_output_take, dianash_session_new, dianash_submit};
use emusen_platform::scraping::{Config, TABLE, emusen_scraping_close, emusen_scraping_open, emusen_scraping_service, opened_its_own};
use std::path::{Path, PathBuf};

#[path = "../examples/recorded_scraper/server.rs"]
mod server;
use server::serve;

/// What `ScreenScraperDeveloper.targets` writes, for a made-up developer.
fn developer_blob() -> Vec<u8> {
    let json = br#"{"devid":"dummydev","devpassword":"dummydevpw","softname":"EmuSenTest"}"#;
    let key = [0x5Au8, 0x13, 0xC7];
    let mut blob = b"ESSD".to_vec();
    blob.extend([1, key.len() as u8]);
    blob.extend(key);
    blob.extend(json.iter().enumerate().map(|(i, b)| b ^ key[i % key.len()]));
    blob
}

unsafe extern "C" fn theme(_ctx: *mut std::ffi::c_void, system: *const u8, len: usize, out: *mut u8, out_len: usize) -> i64 {
    let system = unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(system, len)) };
    let answer: &[u8] = if system == "snes" { br#"["miximage","marquee","video"]"# } else { b"[]" };
    if !out.is_null() && answer.len() <= out_len {
        unsafe { std::ptr::copy_nonoverlapping(answer.as_ptr(), out, answer.len()) };
    }
    answer.len() as i64
}

fn run(sh: *mut emusen_platform::dianash::DianashShell, line: &str) -> String {
    unsafe {
        assert_eq!(dianash_submit(sh, 1, line.as_ptr(), line.len()), 0, "{line}");
        let mut out = vec![0u8; 1 << 16];
        let n = dianash_output_take(sh, out.as_mut_ptr(), out.len());
        String::from_utf8_lossy(&out[..n.max(0) as usize]).into_owned()
    }
}

fn write(path: &Path, bytes: &[u8]) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, bytes).unwrap();
}

#[test]
fn scrape_runs_the_scraper_the_front_end_opened_and_no_other_is_opened() {
    let Some(sqlite) = emusen_sqlite::for_tests() else { return };
    let shipped = std::fs::read_to_string("/proc/self/maps").unwrap().lines().filter_map(|l| l.split_whitespace().nth(5)).find(|p| p.ends_with("libe_sqlite3.so")).map(PathBuf::from).unwrap();
    let base = std::env::var("TMPDIR").map(PathBuf::from).unwrap_or_else(|_| std::env::temp_dir()).join(format!("emusen-platform-scrape-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let (program, library, home, etc) = (base.join("program"), base.join("library"), base.join("home"), base.join("etc"));
    std::fs::create_dir_all(&program).unwrap();
    std::os::unix::fs::symlink(&shipped, program.join(emusen_sqlite::SHIPPED_NAME)).unwrap();
    assert!(tree::init(&program, None), "the program's directory is named before the tree is asked for");
    let mut nes = b"NES\x1a\x01\x01\x00\x00".to_vec();
    nes.resize(16 + 16384 + 8192, 0);
    write(&library.join("nes/Third.nes"), &nes);
    write(&library.join("snes/Game.sfc"), &vec![0u8; 0x8000]);
    write(&library.join("snes/Other.sfc"), &vec![1u8; 0x8000]);
    write(&etc.join("appsettings.json"), format!("{{\"RomDirectory\": \"{}\", \"OpenEmuFallback\": false}}", library.display()).as_bytes());
    tree::tree_mut().set_override(Override::Data, Some(home.to_path_buf()));
    tree::tree_mut().set_override(Override::Config, Some(etc.to_path_buf()));
    let (game, other) = (library.join("snes/Game.sfc").to_string_lossy().into_owned(), library.join("snes/Other.sfc").to_string_lossy().into_owned());

    let games_db = base.join("named/games.db");
    std::fs::create_dir_all(games_db.parent().unwrap()).unwrap();
    let lib = Library::open(sqlite, &games_db.to_string_lossy()).unwrap();
    lib.change_request(&format!("{{\"op\":\"set_favourite\",\"path\":{game:?},\"favourite\":true}}")).unwrap();
    lib.change_request(&format!("{{\"op\":\"save_edits\",\"path\":{other:?},\"changes\":{{\"nomultiscrape\":\"1\"}}}}")).unwrap();
    let (port, pem, asked) = serve();
    let settings = format!(
        "{{\"roms\":{:?},\"openemu\":false,\"developer_files\":false,\"cores\":[[\"snes\"],[\"nes\"]],\"test\":{{\"connect\":\"127.0.0.1:{port}\",\"roots\":{pem:?}}}}}",
        library.to_string_lossy()
    );
    let (media, games_db, blob) = (home.join("Media").to_string_lossy().into_owned(), games_db.to_string_lossy().into_owned(), developer_blob());
    let config = Config {
        size: std::mem::size_of::<Config>() as u32,
        root: media.as_ptr(),
        root_len: media.len(),
        games_db: games_db.as_ptr(),
        games_db_len: games_db.len(),
        sqlite: std::ptr::null_mut(),
        embedded: blob.as_ptr(),
        embedded_len: blob.len(),
        settings: settings.as_ptr(),
        settings_len: settings.len(),
        systems: std::ptr::null(),
        systems_len: 0,
        theme_kinds: Some(theme),
        ctx: std::ptr::null_mut(),
    };
    let opened = unsafe { emusen_scraping_open(&config) };
    assert!(opened > 0, "the scraper opens: {opened}");
    let serial = opened as u64;
    assert!(std::ptr::eq(emusen_scraping_service(), &TABLE), "a front end is given the shells' own table");

    let sh = unsafe { dianash_new(std::ptr::null()) };
    assert_eq!(unsafe { dianash_session_new(sh) }, 1);
    let status = run(sh, "scrape status");
    assert!(status.contains("account.signed_in\tfalse") && status.contains("developer\tembedded") && status.contains("run.state\tidle"), "{status}");
    assert!(!status.contains("dummydev"), "no credential is printed: {status}");

    let listed = run(sh, "scrape ls --system snes");
    assert!(listed.contains(&format!("{game}\tsnes\tcover=- screenshot=- marquee=- titlescreen=- miximage=- backcover=- 3dbox=- physicalmedia=- fanart=-")), "{listed}");
    assert!(!listed.contains(&format!("{other}\t")) && listed.contains("1 game(s) marked \"exclude from multi-scraper\" left out"), "{listed}");
    let themed = run(sh, "scrape ls --system snes --kinds theme");
    assert!(themed.contains(&format!("{game}\tsnes\tmiximage=- marquee=-\n")) && themed.contains("videos are off by default"), "{themed}");
    assert!(run(sh, "scrape plan --favorites").contains("games\t1\n"));
    let plan = run(sh, "scrape plan");
    assert!(plan.contains("games\t2\n") && plan.contains("usable\ttrue\n") && plan.contains("scrape: 2 game(s), about"), "{plan}");
    assert!(run(sh, &format!("echo {other} | scrape plan")).contains("games\t1\n"), "a game named by hand is chosen though excluded");
    assert!(asked.lock().unwrap().is_empty(), "nothing is sent before a run");

    let ran = run(sh, "scrape run --system snes --kinds cover");
    assert!(ran.contains(&format!("found\t{game}\tcover")), "{ran}");
    assert!(ran.contains("scrape: 1 of 1 game(s) done: 1 found"), "{ran}");
    assert!(asked.lock().unwrap().iter().any(|p| p.contains("jeuInfos.php")), "the run asked the recorded server");
    let after = run(sh, "scrape ls --system snes --kinds cover");
    assert!(after.contains(&format!("{game}\tsnes\tcover=screenscraper")), "{after}");
    assert!(!run(sh, "scrape ls --system snes --missing cover").contains(&game), "a game with its cover is not missing it");
    assert!(Path::new(&media).is_dir() && !library.join("snes").read_dir().unwrap().flatten().any(|e| e.file_name() != "Game.sfc" && e.file_name() != "Other.sfc"), "the library is only read");

    let third = library.join("nes/Third.nes").to_string_lossy().into_owned();
    lib.change_request(&format!("{{\"op\":\"set_favourite\",\"path\":{third:?},\"favourite\":true}}")).unwrap();
    assert!(run(sh, "scrape plan --favorites").contains("games\t2\n"), "the shell reads games.db as the front end's own connection wrote it");
    assert_eq!(emusen_scraping_close(serial + 1), 0, "a close names the scraper it means");
    assert_eq!(emusen_scraping_close(serial), 1);
    assert_eq!(emusen_scraping_close(serial), 0, "a scraper closes once");
    let refused = run(sh, "scrape status");
    assert!(refused.contains("not open now"), "{refused}");
    assert!(!opened_its_own(), "after a front end's open this library never opens a scraper of its own");
    unsafe { dianash_free(sh) };
    let _ = std::fs::remove_dir_all(&base);
}
