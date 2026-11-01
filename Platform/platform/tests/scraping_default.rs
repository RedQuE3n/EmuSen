//! `scrape` in a process whose host lent no scraper: one is opened from the settings file, and a later lend replaces it - see EmuSen_LibraryTools.md §8.

use emusen_galaxia::tree::{self, Override};
use emusen_library::Library;
use emusen_platform::dianash::{dianash_free, dianash_new, dianash_output_take, dianash_session_new, dianash_submit};
use emusen_platform::scraping::{Lend, emusen_scraping_lend, opened_its_own};
use emusen_scrape::Scraper;
use emusen_sqlite::Sqlite;
use std::path::PathBuf;
use std::sync::Arc;

/// SQLite as the C# ships it: `EMUSEN_SQLITE`, else the NuGet cache's builds, newest first, the first that loads.
fn shipped_sqlite() -> Option<(PathBuf, Arc<Sqlite>)> {
    let mut candidates: Vec<PathBuf> = std::env::var_os("EMUSEN_SQLITE").map(PathBuf::from).into_iter().collect();
    if let Some(home) = std::env::var_os("HOME") {
        for package in ["sqlitepclraw.lib.e_sqlite3", "sqlite"] {
            if let Ok(versions) = std::fs::read_dir(PathBuf::from(&home).join(".nuget/packages").join(package)) {
                let mut found: Vec<PathBuf> = versions.flatten().map(|v| v.path().join("runtimes/linux-x64/native/libe_sqlite3.so")).filter(|p| p.exists()).collect();
                found.sort();
                candidates.extend(found.into_iter().rev());
            }
        }
    }
    // The NuGet builds leave libm to their host, which a .NET process has loaded and a test binary has not.
    unsafe { libc::dlopen(c"libm.so.6".as_ptr(), libc::RTLD_NOW | libc::RTLD_GLOBAL) };
    candidates.into_iter().find_map(|p| Sqlite::open(&p.to_string_lossy()).ok().map(|s| (p, Arc::new(s))))
}

fn run(sh: *mut emusen_platform::dianash::DianashShell, line: &str) -> String {
    unsafe {
        assert_eq!(dianash_submit(sh, 1, line.as_ptr(), line.len()), 0, "{line}");
        let mut out = vec![0u8; 1 << 16];
        let n = dianash_output_take(sh, out.as_mut_ptr(), out.len());
        String::from_utf8_lossy(&out[..n.max(0) as usize]).into_owned()
    }
}

#[test]
fn a_process_with_no_lend_opens_its_own_scraper_and_a_lend_replaces_it() {
    let Some((shipped, sqlite)) = shipped_sqlite() else {
        eprintln!("skipped: no libe_sqlite3 (EMUSEN_SQLITE or the NuGet cache)");
        return;
    };
    let base = std::env::var("TMPDIR").map(PathBuf::from).unwrap_or_else(|_| std::env::temp_dir()).join(format!("emusen-platform-scrape-own-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let (program, library, home, etc) = (base.join("program"), base.join("library"), base.join("home"), base.join("etc"));
    for d in [&program, &library.join("nes"), &etc] {
        std::fs::create_dir_all(d).unwrap();
    }
    std::os::unix::fs::symlink(&shipped, program.join(emusen_sqlite::SHIPPED_NAME)).unwrap();
    assert!(tree::init(&program.to_string_lossy(), None), "the program's directory is named before the tree is asked for");
    let mut nes = b"NES\x1a\x01\x01\x00\x00".to_vec();
    nes.resize(16 + 16384 + 8192, 0);
    std::fs::write(library.join("nes/Third.nes"), &nes).unwrap();
    std::fs::write(etc.join("appsettings.json"), format!("{{\"RomDirectory\": \"{}\", \"OpenEmuFallback\": false}}", library.display())).unwrap();
    tree::tree_mut().set_override(Override::Data, Some(home.to_string_lossy().into_owned()));
    tree::tree_mut().set_override(Override::Config, Some(etc.to_string_lossy().into_owned()));

    let sh = unsafe { dianash_new(std::ptr::null()) };
    assert_eq!(unsafe { dianash_session_new(sh) }, 1);
    assert!(!opened_its_own(), "nothing is opened before a scrape command asks");
    let plan = run(sh, "scrape plan");
    assert!(plan.contains("games\t1\n"), "{plan}");
    assert!(opened_its_own() && home.join("Media/media.db").is_file() && home.join("Library/games.db").is_file(), "the store and games.db are the tree's");
    let listed = run(sh, "scrape ls");
    assert!(listed.contains(&format!("{}\tnes\tcover=-", library.join("nes/Third.nes").display())), "{listed}");
    assert!(run(sh, "scrape ls --kinds theme").contains("no theme loader is lent here"), "the theme is a host's to read");

    let lib: &'static Library = Box::leak(Box::new(Library::open(sqlite.clone(), ":memory:").unwrap()));
    let store = base.join("lent");
    let scraper = Box::into_raw(Box::new(Scraper::open(sqlite, &store.to_string_lossy(), Some(lib), &[], r#"{"developer_files":false,"openemu":false,"test":{"offline":true}}"#).unwrap()));
    let lend = Lend { size: std::mem::size_of::<Lend>() as u32, scraper, library: lib as *const Library as *mut Library, systems: br#"["nes"]"#.as_ptr(), systems_len: 7, theme_kinds: None, ctx: std::ptr::null_mut() };
    assert_eq!(unsafe { emusen_scraping_lend(&lend) }, 0);
    assert!(!opened_its_own(), "a lend closes the scraper this library opened, so the process holds one");
    assert!(run(sh, "scrape status").contains("developer\tnone"), "the lent scraper answers now");
    assert_eq!(unsafe { emusen_scraping_lend(std::ptr::null()) }, 0);
    unsafe { dianash_free(sh) };
    drop(unsafe { Box::from_raw(scraper) });
    let _ = std::fs::remove_dir_all(&base);
}
