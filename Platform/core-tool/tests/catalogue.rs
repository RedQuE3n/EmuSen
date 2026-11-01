//! The core tool against a catalogue and index made by the test and served by its own TLS server on 127.0.0.1, with
//! certificates made at test time; never the internet (EmuSen_Cores.md §9, C2).

use emusen_core_entry::{library_suffix, sha256_file, Entry, Notice};
use emusen_core_tool::catalogue::{self, State};
use emusen_core_tool::info::CoreInfo;
use emusen_core_tool::{index, job, Context, Placement, Source};
use emusen_galaxia::archive::ZipWriter;
use emusen_galaxia::json::{self, Value};
use rcgen::{BasicConstraints, CertificateParams, CertifiedIssuer, DnType, IsCa, KeyPair};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const LRPS2: &str = "display_name = \"Sony - PlayStation 2 (LRPS2)\"\ncorename = \"LRPS2\"\nauthors = \"PCSX2 Team|libretro\"\nlicense = \"GPLv3\"\ndisplay_version = \"2.3.1\"\nsystemid = \"playstation2\"\ndatabase = \"Sony - PlayStation 2\"\nsupported_extensions = \"iso|chd|cue\"\nfirmware_count = 2\nfirmware0_desc = \"PlayStation 2 BIOS\"\nfirmware0_path = \"pcsx2/bios/ps2.bin\"\nfirmware0_opt = \"false\"\nfirmware1_path = \"pcsx2/extra.bin\"\nfirmware1_opt = \"true\"\nneeds_fullpath = \"true\"\nrequired_hw_api = \"Vulkan >= 1.1 | OpenGL Core >= 3.3\"\nnotes = \"(!) needs a BIOS|disc images by path\"\n# a comment\n";
const BEETLE: &str = "display_name = \"Sony - PlayStation (Beetle PSX)\"\ncorename = \"Beetle PSX\"\nauthors = \"Mednafen Team\"\nlicense = \"GPLv2\"\ndisplay_version = \"v0.9.44.1\"\nsystemid = \"playstation\"\ndatabase = \"Sony - PlayStation\"\n";
const GLONLY: &str = "display_name = \"Only GL\"\ncorename = \"GL\"\nlicense = \"MIT\"\nsystemid = \"playstation\"\nrequired_hw_api = \"OpenGL Core >= 3.3 | OpenGL ES >= 3.0\"\n";
const STRANGER: &str = "display_name = \"Arcade Thing\"\ncorename = \"AT\"\nlicense = \"MIT\"\nsystemid = \"arcade\"\n";

fn scratch(name: &str) -> PathBuf {
    let base = std::env::var_os("TMPDIR").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    let dir = base.join(format!("emusen-core-tool-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn info_zip(dir: &Path, infos: &[(&str, &str)]) -> Vec<u8> {
    let src = dir.join("infos");
    std::fs::create_dir_all(&src).unwrap();
    let zip = dir.join("info.zip");
    let mut w = ZipWriter::create(zip.to_str().unwrap()).unwrap();
    for (stem, text) in infos {
        let file = src.join(format!("{stem}_libretro.info"));
        std::fs::write(&file, text).unwrap();
        w.add(&format!("{stem}_libretro.info"), file.to_str().unwrap(), std::time::SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000), false, &|| false).unwrap();
    }
    w.finish().unwrap();
    std::fs::read(zip).unwrap()
}

fn index_text() -> String {
    let s = library_suffix();
    format!("2026-10-31 3a224a58 pcsx2_libretro.{s}.zip\n2026-10-29 11112222 mednafen_psx_libretro.{s}.zip\n2026-10-30 0000abcd glonly_libretro.{s}.zip\n2026-10-30 0000abce stranger_libretro.{s}.zip\n2026-10-30 0000abcf noinfo_libretro.{s}.zip\nnot a line\n2026-10-30 zzzzzzzz bad_libretro.{s}.zip\n2026-10-30 12345678 RetroArch.7z\n")
}

struct Server {
    port: u16,
    root: CertificateDer<'static>,
    heads: Arc<Mutex<Vec<String>>>,
}

fn serve(files: HashMap<String, Vec<u8>>) -> Server {
    let root_key = KeyPair::generate().unwrap();
    let mut root_params = CertificateParams::new(Vec::<String>::new()).unwrap();
    root_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    root_params.distinguished_name.push(DnType::CommonName, "EmuSen Test Root");
    let root = CertifiedIssuer::self_signed(root_params, root_key).unwrap();
    let leaf_key = KeyPair::generate().unwrap();
    let leaf = CertificateParams::new(vec!["localhost".to_string(), "buildbot.libretro.com".to_string()]).unwrap().signed_by(&leaf_key, &root).unwrap();
    let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(leaf_key.serialize_der()));
    let config = Arc::new(
        rustls::ServerConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_safe_default_protocol_versions()
            .unwrap()
            .with_no_client_auth()
            .with_single_cert(vec![leaf.der().clone()], key)
            .unwrap(),
    );
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let heads = Arc::new(Mutex::new(Vec::new()));
    let seen = heads.clone();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut tcp) = stream else { continue };
            let mut conn = rustls::ServerConnection::new(config.clone()).unwrap();
            let mut tls = rustls::Stream::new(&mut conn, &mut tcp);
            let mut head = Vec::new();
            let mut byte = [0u8; 1];
            while !head.ends_with(b"\r\n\r\n") {
                match tls.read(&mut byte) {
                    Ok(1) => head.push(byte[0]),
                    _ => break,
                }
            }
            let head = String::from_utf8_lossy(&head).into_owned();
            let path = head.split_whitespace().nth(1).unwrap_or("").to_string();
            seen.lock().unwrap().push(path.clone());
            let reply = match files.get(&path) {
                Some(body) if body.starts_with(b"REDIRECT ") => format!("HTTP/1.1 302 Found\r\nLocation: {}\r\nContent-Length: 0\r\n\r\n", String::from_utf8_lossy(&body[9..])).into_bytes(),
                Some(body) => [format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n", body.len()).into_bytes(), body.clone()].concat(),
                None => b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n".to_vec(),
            };
            let _ = tls.write_all(&reply);
            let _ = tls.flush();
            conn.send_close_notify();
            let _ = conn.complete_io(&mut tcp);
        }
    });
    Server { port, root: root.der().clone(), heads }
}

fn source(server: &Server) -> Source {
    Source { base: format!("https://localhost:{}", server.port), platform: "linux/x86_64", test_roots: vec![server.root.clone()], test_connect: None, offline: false }
}

fn placement() -> Placement {
    Arc::new(|systemid: &str, databases: &[&str]| {
        let names: Vec<&str> = if databases.is_empty() { vec![systemid] } else { databases.to_vec() };
        names.iter().filter_map(|n| match *n {
            "Sony - PlayStation 2" | "playstation2" => Some("ps2".to_string()),
            "Sony - PlayStation" | "playstation" => Some("psx".to_string()),
            _ => None,
        }).collect()
    })
}

fn files(dir: &Path) -> HashMap<String, Vec<u8>> {
    let zip = info_zip(dir, &[("pcsx2", LRPS2), ("mednafen_psx", BEETLE), ("glonly", GLONLY), ("stranger", STRANGER), ("unindexed", BEETLE)]);
    HashMap::from([("/assets/frontend/info.zip".to_string(), zip), ("/nightly/linux/x86_64/latest/.index-extended".to_string(), index_text().into_bytes())])
}

/// Every event line of a job, parsed.
fn events(ctx: &Context, request: &str) -> Vec<Value> {
    let j = job::start(request, ctx.clone()).unwrap();
    let mut out = Vec::new();
    while !j.finished() {
        if let Some(line) = j.peek(Duration::from_millis(200)) {
            out.push(json::parse(&line).unwrap());
            j.take();
        }
    }
    out
}

fn text<'a>(e: &'a Value, k: &str) -> &'a str {
    e.get(k).and_then(Value::as_str).unwrap_or("")
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
    let ctx = Context { cores: cores.clone(), source: Some(source(&server)), placement: Some(placement()) };

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
    let ctx = Context { cores: cores.clone(), source: Some(source(&server)), placement: Some(placement()) };
    events(&ctx, r#"{"op":"core-fetch"}"#);
    let e = events(&ctx, r#"{"op":"core-info","core":"pcsx2"}"#);
    let d = &e[0];
    assert_eq!(text(d, "from"), format!("https://localhost:{}/nightly/linux/x86_64/latest/pcsx2_libretro.{}.zip", server.port, library_suffix()));
    assert_eq!(text(d, "index_crc32"), "3a224a58");
    assert_eq!(d.get("needs_fullpath"), Some(&Value::Bool(true)));
    let Some(Value::Array(fw)) = d.get("firmware") else { panic!("{d:?}") };
    assert_eq!(fw.len(), 2);
    assert_eq!(d.get("notes"), Some(&Value::Array(vec![Value::string("(!) needs a BIOS"), Value::string("disc images by path")])));

    let missing = events(&ctx, r#"{"op":"core-info","core":"nothere"}"#);
    assert_eq!(text(&missing[0], "event"), "refused");
    assert!(text(&missing[0], "reason").contains("nothere"));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn a_placement_in_the_request_replaces_none_and_without_one_nothing_is_called_unsupported_for_its_system() {
    let dir = scratch("placement");
    let server = serve(files(&dir));
    let ctx = Context { cores: dir.join("Cores"), source: Some(source(&server)), placement: None };
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
    let ctx = Context { cores: cores.clone(), source: Some(source(&server)), placement: None };
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
