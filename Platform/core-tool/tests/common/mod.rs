//! What the core tool's tests share: a catalogue made by the test, served by the test's own TLS server on 127.0.0.1 with
//! certificates made at test time, and a job's events read back (EmuSen_Cores.md §9).
#![allow(dead_code)]

use emusen_core_tool::{job, Context, Placement, Source};
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

pub const LRPS2: &str = "display_name = \"Sony - PlayStation 2 (LRPS2)\"\ncorename = \"LRPS2\"\nauthors = \"PCSX2 Team|libretro\"\nlicense = \"GPLv3\"\ndisplay_version = \"2.3.1\"\nsystemid = \"playstation2\"\ndatabase = \"Sony - PlayStation 2\"\nsupported_extensions = \"iso|chd|cue\"\nfirmware_count = 2\nfirmware0_desc = \"PlayStation 2 BIOS\"\nfirmware0_path = \"pcsx2/bios/ps2.bin\"\nfirmware0_opt = \"false\"\nfirmware1_path = \"pcsx2/extra.bin\"\nfirmware1_opt = \"true\"\nneeds_fullpath = \"true\"\nrequired_hw_api = \"Vulkan >= 1.1 | OpenGL Core >= 3.3\"\nnotes = \"(!) needs a BIOS|disc images by path\"\n# a comment\n";
pub const BEETLE: &str = "display_name = \"Sony - PlayStation (Beetle PSX)\"\ncorename = \"Beetle PSX\"\nauthors = \"Mednafen Team\"\nlicense = \"GPLv2\"\ndisplay_version = \"v0.9.44.1\"\nsystemid = \"playstation\"\ndatabase = \"Sony - PlayStation\"\n";
pub const GLONLY: &str = "display_name = \"Only GL\"\ncorename = \"GL\"\nlicense = \"MIT\"\nsystemid = \"playstation\"\nrequired_hw_api = \"OpenGL Core >= 3.3 | OpenGL ES >= 3.0\"\n";
pub const STRANGER: &str = "display_name = \"Arcade Thing\"\ncorename = \"AT\"\nlicense = \"MIT\"\nsystemid = \"arcade\"\n";

pub fn scratch(name: &str) -> PathBuf {
    let base = std::env::var_os("TMPDIR").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    let dir = base.join(format!("emusen-core-tool-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

pub fn info_zip(dir: &Path, infos: &[(&str, &str)]) -> Vec<u8> {
    let src = dir.join("infos");
    std::fs::create_dir_all(&src).unwrap();
    let zip = dir.join("info.zip");
    let _ = std::fs::remove_file(&zip);
    let mut w = ZipWriter::create(zip.to_str().unwrap()).unwrap();
    for (stem, text) in infos {
        let file = src.join(format!("{stem}_libretro.info"));
        std::fs::write(&file, text).unwrap();
        w.add(&format!("{stem}_libretro.info"), file.to_str().unwrap(), std::time::SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000), false, &|| false).unwrap();
    }
    w.finish().unwrap();
    std::fs::read(zip).unwrap()
}

pub struct Server {
    pub port: u16,
    pub root: CertificateDer<'static>,
    /// The paths asked for, in order.
    pub heads: Arc<Mutex<Vec<String>>>,
}

pub fn serve(files: HashMap<String, Vec<u8>>) -> Server {
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

pub fn source(server: &Server) -> Source {
    Source { base: format!("https://localhost:{}", server.port), platform: "linux/x86_64", test_roots: vec![server.root.clone()], test_connect: None, offline: false }
}

pub fn placement() -> Placement {
    Arc::new(|systemid: &str, databases: &[&str]| {
        let names: Vec<&str> = if databases.is_empty() { vec![systemid] } else { databases.to_vec() };
        names.iter().filter_map(|n| match *n {
            "Sony - PlayStation 2" | "playstation2" => Some("ps2".to_string()),
            "Sony - PlayStation" | "playstation" => Some("psx".to_string()),
            _ => None,
        }).collect()
    })
}

/// Every event line of a job, parsed.
pub fn events(ctx: &Context, request: &str) -> Vec<Value> {
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

pub fn text<'a>(e: &'a Value, k: &str) -> &'a str {
    e.get(k).and_then(Value::as_str).unwrap_or("")
}

