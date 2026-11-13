//! A recorded-response ScreenScraper on 127.0.0.1, which the scraping test and a front end's tests answer from: every
//! address the scraper asks, `jeuInfos` with the scraper's first recorded answer and any other path with a PNG, over
//! HTTPS under a root made at start (EmuSen_LibraryTools.md §8).

use rcgen::{BasicConstraints, CertificateParams, CertifiedIssuer, DnType, IsCa, KeyPair};
use rustls::pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

/// A 1x1 PNG, padded past the scraper's smallest picture after its end, which every media URL answers with.
const PNG: [u8; 67] = [
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
    0x42, 0x60, 0x82,
];

/// The first recorded `jeuInfos` answer of the scraper's own fixtures.
fn recorded() -> String {
    let line = include_str!("../../../scrape/tests/fixtures/answers.jsonl").lines().next().unwrap();
    let v = serde_json::from_str::<serde_json::Value>(line).unwrap();
    v.get("input").and_then(|i| i.as_str()).unwrap().to_string()
}

/// An HTTPS server at every address the scraper asks, answering `jeuInfos` with the recording and any other path with the PNG; the paths it was asked kept.
pub fn serve() -> (u16, String, Arc<Mutex<Vec<String>>>) {
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
