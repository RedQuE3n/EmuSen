//! `scrape` in a shell this library makes, over a scraper the test lends as a host does, answered by a recorded-response server on 127.0.0.1 - see EmuSen_LibraryTools.md §8.

use emusen_galaxia::tree::{self, Override};
use emusen_library::Library;
use emusen_platform::dianash::{dianash_free, dianash_new, dianash_output_take, dianash_session_new, dianash_submit};
use emusen_platform::scraping::{Lend, emusen_scraping_lend, opened_its_own};
use emusen_scrape::Scraper;
use rcgen::{BasicConstraints, CertificateParams, CertifiedIssuer, DnType, IsCa, KeyPair};
use rustls::pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};


/// A 1x1 PNG, padded past the scraper's smallest picture after its end, which every media URL answers with.
const PNG: [u8; 67] = [
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
    0x42, 0x60, 0x82,
];

/// The first recorded `jeuInfos` answer of the scraper's own fixtures.
fn recorded() -> String {
    let line = include_str!("../../scrape/tests/fixtures/answers.jsonl").lines().next().unwrap();
    let v = emusen_dotnet::json::parse(line).unwrap();
    v.get("input").and_then(|i| i.as_str()).unwrap().to_string()
}

/// An HTTPS server at every address the scraper asks, answering `jeuInfos` with the recording and any other path with the PNG; the paths it was asked kept.
fn serve() -> (u16, String, Arc<Mutex<Vec<String>>>) {
    let root_key = KeyPair::generate().unwrap();
    let mut root_params = CertificateParams::new(Vec::<String>::new()).unwrap();
    root_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    root_params.distinguished_name.push(DnType::CommonName, "EmuSen Test Root");
    let root = CertifiedIssuer::self_signed(root_params, root_key).unwrap();
    let leaf_key = KeyPair::generate().unwrap();
    let names = ["api.screenscraper.fr", "neoclone.screenscraper.fr", "x", "localhost"].map(String::from).to_vec();
    let leaf = CertificateParams::new(names).unwrap().signed_by(&leaf_key, &root).unwrap();
    let config = Arc::new(
        rustls::ServerConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_safe_default_protocol_versions()
            .unwrap()
            .with_no_client_auth()
            .with_single_cert(vec![leaf.der().clone()], PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(leaf_key.serialize_der())))
            .unwrap(),
    );
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let asked = Arc::new(Mutex::new(Vec::new()));
    let (kept, body) = (asked.clone(), recorded());
    std::thread::spawn(move || {
        for socket in listener.incoming().flatten() {
            let (config, kept, body) = (config.clone(), kept.clone(), body.clone());
            std::thread::spawn(move || answer(socket, config, kept, &body));
        }
    });
    (port, root.pem(), asked)
}

fn answer(socket: TcpStream, config: Arc<rustls::ServerConfig>, asked: Arc<Mutex<Vec<String>>>, body: &str) {
    let Ok(connection) = rustls::ServerConnection::new(config) else { return };
    let mut stream = rustls::StreamOwned::new(connection, socket);
    let mut head = Vec::new();
    let mut byte = [0u8; 1];
    while !head.ends_with(b"\r\n\r\n") {
        match stream.read(&mut byte) {
            Ok(1) => head.push(byte[0]),
            _ => return,
        }
    }
    let path = String::from_utf8_lossy(&head).split(' ').nth(1).unwrap_or("/").to_string();
    asked.lock().unwrap().push(path.clone());
    let (kind, bytes) = if path.contains("jeuInfos.php") { ("application/json", body.as_bytes().to_vec()) } else { ("image/png", [&PNG[..], &[0u8; 4096]].concat()) };
    let _ = stream.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: {kind}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", bytes.len()).as_bytes());
    let _ = stream.write_all(&bytes);
    let _ = stream.flush();
}

/// What `ScreenScraperDeveloper.targets` writes, for a made-up developer.
fn developer_blob() -> Vec<u8> {
    let json = br#"{"devid":"testdev","devpassword":"testdevpw","softname":"EmuSenTest"}"#;
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
fn scrape_runs_the_lent_scraper_and_no_other_is_opened() {
    let Some(sqlite) = emusen_sqlite::for_tests() else { return };
    let base = std::env::var("TMPDIR").map(PathBuf::from).unwrap_or_else(|_| std::env::temp_dir()).join(format!("emusen-platform-scrape-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let (library, home, etc) = (base.join("library"), base.join("home"), base.join("etc"));
    let mut nes = b"NES\x1a\x01\x01\x00\x00".to_vec();
    nes.resize(16 + 16384 + 8192, 0);
    write(&library.join("nes/Third.nes"), &nes);
    write(&library.join("snes/Game.sfc"), &vec![0u8; 0x8000]);
    write(&library.join("snes/Other.sfc"), &vec![1u8; 0x8000]);
    write(&etc.join("appsettings.json"), format!("{{\"RomDirectory\": \"{}\", \"OpenEmuFallback\": false}}", library.display()).as_bytes());
    tree::tree_mut().set_override(Override::Data, Some(home.to_string_lossy().into_owned()));
    tree::tree_mut().set_override(Override::Config, Some(etc.to_string_lossy().into_owned()));
    let (game, other) = (library.join("snes/Game.sfc").to_string_lossy().into_owned(), library.join("snes/Other.sfc").to_string_lossy().into_owned());

    let games_db = home.join("Library/games.db");
    std::fs::create_dir_all(games_db.parent().unwrap()).unwrap();
    let lib: &'static Library = Box::leak(Box::new(Library::open(sqlite.clone(), &games_db.to_string_lossy()).unwrap()));
    lib.change_request(&format!("{{\"op\":\"set_favourite\",\"path\":{game:?},\"favourite\":true}}")).unwrap();
    lib.change_request(&format!("{{\"op\":\"save_edits\",\"path\":{other:?},\"changes\":{{\"nomultiscrape\":\"1\"}}}}")).unwrap();
    let (port, pem, asked) = serve();
    let settings = format!(
        "{{\"roms\":{:?},\"openemu\":false,\"developer_files\":false,\"cores\":[[\"snes\"],[\"nes\"]],\"test\":{{\"connect\":\"127.0.0.1:{port}\",\"roots\":{pem:?}}}}}",
        library.to_string_lossy()
    );
    let media = home.join("Media");
    let scraper = Box::into_raw(Box::new(Scraper::open(sqlite, &media.to_string_lossy(), Some(lib), &developer_blob(), &settings).unwrap()));
    let lend = Lend { size: std::mem::size_of::<Lend>() as u32, scraper, library: lib as *const Library as *mut Library, systems: std::ptr::null(), systems_len: 0, theme_kinds: Some(theme), ctx: std::ptr::null_mut() };
    assert_eq!(unsafe { emusen_scraping_lend(&lend) }, 0);

    let sh = unsafe { dianash_new(std::ptr::null()) };
    assert_eq!(unsafe { dianash_session_new(sh) }, 1);
    let status = run(sh, "scrape status");
    assert!(status.contains("account.signed_in\tfalse") && status.contains("developer\tembedded") && status.contains("run.state\tidle"), "{status}");
    assert!(!status.contains("testdev"), "no credential is printed: {status}");

    let listed = run(sh, "scrape ls --system snes");
    assert!(listed.contains(&format!("{game}\tsnes\tcover=- screenshot=- marquee=- miximage=-")), "{listed}");
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
    assert!(media.is_dir() && !library.join("snes").read_dir().unwrap().flatten().any(|e| e.file_name() != "Game.sfc" && e.file_name() != "Other.sfc"), "the library is only read");

    assert_eq!(unsafe { emusen_scraping_lend(std::ptr::null()) }, 0);
    let refused = run(sh, "scrape status");
    assert!(refused.contains("not open now"), "{refused}");
    assert!(!opened_its_own(), "after a lend this library never opens a scraper of its own");
    unsafe { dianash_free(sh) };
    drop(unsafe { Box::from_raw(scraper) });
    let _ = std::fs::remove_dir_all(&base);
}
