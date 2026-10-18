//! emusen-net against a server of the test's own on 127.0.0.1, with certificates made at test time; never the internet.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use emusen_net::{Download, Failure, Kind, Progress, Request, State, fetch};
use rcgen::{BasicConstraints, CertificateParams, CertifiedIssuer, DnType, IsCa, KeyPair};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};

struct Certs {
    root: CertificateDer<'static>,
    leaf: (CertificateDer<'static>, Vec<u8>),
    stranger: (CertificateDer<'static>, Vec<u8>),
    misnamed: (CertificateDer<'static>, Vec<u8>),
}

fn certs() -> Certs {
    let root_key = KeyPair::generate().unwrap();
    let mut root_params = CertificateParams::new(Vec::<String>::new()).unwrap();
    root_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    root_params.distinguished_name.push(DnType::CommonName, "EmuSen Test Root");
    let root = CertifiedIssuer::self_signed(root_params, root_key).unwrap();
    let names = vec!["localhost".to_string(), "127.0.0.1".to_string()];
    let leaf_key = KeyPair::generate().unwrap();
    let leaf = CertificateParams::new(names.clone()).unwrap().signed_by(&leaf_key, &root).unwrap();
    let stranger_key = KeyPair::generate().unwrap();
    let stranger = CertificateParams::new(names).unwrap().self_signed(&stranger_key).unwrap();
    let misnamed_key = KeyPair::generate().unwrap();
    let misnamed = CertificateParams::new(vec!["other.test".to_string()]).unwrap().signed_by(&misnamed_key, &root).unwrap();
    Certs {
        root: root.der().clone(),
        leaf: (leaf.der().clone(), leaf_key.serialize_der()),
        stranger: (stranger.der().clone(), stranger_key.serialize_der()),
        misnamed: (misnamed.der().clone(), misnamed_key.serialize_der()),
    }
}

type Handler = Arc<dyn Fn(&str, &mut dyn Write) + Send + Sync>;

/// A TLS server on 127.0.0.1 that hands each request's path to `handler`, closing the connection after it.
struct Server {
    port: u16,
    heads: Arc<Mutex<Vec<String>>>,
    connections: Arc<AtomicUsize>,
}

impl Server {
    fn start(cert: &(CertificateDer<'static>, Vec<u8>), handler: Handler) -> Server {
        let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(cert.1.clone()));
        let config = rustls::ServerConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_safe_default_protocol_versions()
            .unwrap()
            .with_no_client_auth()
            .with_single_cert(vec![cert.0.clone()], key)
            .unwrap();
        let config = Arc::new(config);
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let heads = Arc::new(Mutex::new(Vec::new()));
        let connections = Arc::new(AtomicUsize::new(0));
        let (h, c) = (heads.clone(), connections.clone());
        thread::spawn(move || {
            for socket in listener.incoming().flatten() {
                c.fetch_add(1, Ordering::SeqCst);
                let (config, handler, heads) = (config.clone(), handler.clone(), h.clone());
                thread::spawn(move || serve(socket, config, handler, heads));
            }
        });
        Server { port, heads, connections }
    }

    fn url(&self, path: &str) -> String {
        format!("https://localhost:{}{path}", self.port)
    }
}

fn serve(socket: TcpStream, config: Arc<rustls::ServerConfig>, handler: Handler, heads: Arc<Mutex<Vec<String>>>) {
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
    let head = String::from_utf8_lossy(&head).to_string();
    let path = head.split(' ').nth(1).unwrap_or("/").to_string();
    heads.lock().unwrap().push(head);
    handler(&path, &mut stream);
    let _ = stream.flush();
}

fn request(url: String, roots: &[CertificateDer<'static>], timeout: Duration) -> Request {
    Request { url, timeout, test_roots: roots.to_vec() }
}

fn get(server: &Server, path: &str, certs: &Certs) -> Result<Vec<u8>, Failure> {
    fetch(&request(server.url(path), std::slice::from_ref(&certs.root), Duration::from_secs(10)), &Progress::default())
}

fn reply(text: &'static str) -> Handler {
    Arc::new(move |_, w| {
        let _ = w.write_all(text.as_bytes());
    })
}

#[test]
fn a_body_comes_whole_with_the_request_dotnet_sends_and_its_progress() {
    let c = certs();
    let body: Vec<u8> = (0..300_000u32).map(|i| (i * 7) as u8).collect();
    let served = body.clone();
    let server = Server::start(&c.leaf, Arc::new(move |_, w| {
        let _ = w.write_all(format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n", served.len()).as_bytes());
        let _ = w.write_all(&served);
    }));
    let progress = Progress::default();
    let got = fetch(&request(server.url("/a?b=1"), std::slice::from_ref(&c.root), Duration::from_secs(10)), &progress).unwrap();
    assert_eq!(got, body);
    assert_eq!((progress.received(), progress.total()), (300_000, Some(300_000)));
    // .NET writes `Host`, ureq `host`: header names are case-insensitive (RFC 9110 §5.1), the one difference on the wire.
    assert_eq!(server.heads.lock().unwrap().as_slice(), [format!("GET /a?b=1 HTTP/1.1\r\nhost: localhost:{}\r\n\r\n", server.port)]);
}

#[test]
fn a_relative_redirect_is_followed_and_fifty_is_the_most() {
    let c = certs();
    let server = Server::start(&c.leaf, Arc::new(|path, w| {
        let text = if path == "/start" { "HTTP/1.1 302 Found\r\nLocation: next?x=1\r\nContent-Length: 0\r\n\r\n".to_string() } else { "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nhi".to_string() };
        let _ = w.write_all(text.as_bytes());
    }));
    assert_eq!(get(&server, "/start", &c).unwrap(), b"hi");
    assert!(server.heads.lock().unwrap()[1].starts_with("GET /next?x=1 HTTP/1.1\r\n"));

    let looping = Server::start(&c.leaf, reply("HTTP/1.1 301 Moved Permanently\r\nLocation: /again\r\nContent-Length: 0\r\n\r\n"));
    let failure = get(&looping, "/", &c).unwrap_err();
    assert_eq!(failure.message, "Response status code does not indicate success: 301 (Moved Permanently).");
    assert_eq!(looping.heads.lock().unwrap().len(), 51);
}

#[test]
fn a_redirect_from_https_to_http_is_not_followed() {
    let c = certs();
    let server = Server::start(&c.leaf, reply("HTTP/1.1 302 Found\r\nLocation: http://localhost:1/x\r\nContent-Length: 0\r\n\r\n"));
    let failure = get(&server, "/", &c).unwrap_err();
    assert_eq!((failure.kind, failure.status), (Kind::Status, Some(302)));
    assert_eq!(failure.message, "Response status code does not indicate success: 302 (Found).");
}

#[test]
fn a_status_names_the_reason_the_server_sent() {
    let c = certs();
    for (line, words) in [
        ("404 Not Found", "404 (Not Found)."),
        ("500 Internal Server Error", "500 (Internal Server Error)."),
        ("404 Nope Nope", "404 (Nope Nope)."),
        ("404 ", "404."),
        ("404", "404."),
        ("599 Whatever", "599 (Whatever)."),
    ] {
        let text: &'static str = Box::leak(format!("HTTP/1.1 {line}\r\nContent-Length: 3\r\n\r\nno!").into_boxed_str());
        let server = Server::start(&c.leaf, reply(text));
        let failure = get(&server, "/", &c).unwrap_err();
        assert_eq!(failure.message, format!("Response status code does not indicate success: {words}"), "{line}");
    }
}

#[test]
fn a_body_cut_short_is_refused_unless_its_end_was_the_close() {
    let c = certs();
    let cut = Server::start(&c.leaf, Arc::new(|_, w| {
        let _ = w.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 1000\r\n\r\n");
        let _ = w.write_all(&[b'x'; 500]);
    }));
    assert_eq!(get(&cut, "/", &c).unwrap_err(), Failure::ended());
    let chunked = Server::start(&c.leaf, Arc::new(|_, w| {
        let _ = w.write_all(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n3e8\r\n");
        let _ = w.write_all(&[b'x'; 500]);
    }));
    assert_eq!(get(&chunked, "/", &c).unwrap_err(), Failure::ended());
    let closed = Server::start(&c.leaf, Arc::new(|_, w| {
        let _ = w.write_all(b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n");
        let _ = w.write_all(&[b'x'; 500]);
    }));
    assert_eq!(get(&closed, "/", &c).unwrap().len(), 500);
}

#[test]
fn the_timeout_covers_the_body_and_the_wait_for_an_answer() {
    let c = certs();
    let slow = Server::start(&c.leaf, Arc::new(|_, w| {
        let _ = w.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 1000\r\n\r\n0123456789");
        let _ = w.flush();
        thread::sleep(Duration::from_secs(4));
    }));
    let silent = Server::start(&c.leaf, Arc::new(|_, _| thread::sleep(Duration::from_secs(4))));
    let hops = Server::start(&c.leaf, Arc::new(|path, w| {
        thread::sleep(Duration::from_secs(1));
        let text = if path == "/" { "HTTP/1.1 302 Found\r\nLocation: /next\r\nContent-Length: 0\r\n\r\n" } else { "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nhi" };
        let _ = w.write_all(text.as_bytes());
    }));
    for server in [&slow, &silent, &hops] {
        let started = Instant::now();
        let failure = fetch(&request(server.url("/"), std::slice::from_ref(&c.root), Duration::from_millis(1500)), &Progress::default()).unwrap_err();
        assert_eq!(failure.message, "The request was canceled due to the configured HttpClient.Timeout of 1.5 seconds elapsing.");
        assert!(started.elapsed() < Duration::from_secs(3), "{:?}", started.elapsed());
    }
}

#[test]
fn a_cancel_ends_the_download_at_once_and_its_progress_is_drained() {
    let c = certs();
    let server = Server::start(&c.leaf, Arc::new(|_, w| {
        let _ = w.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 100000\r\n\r\n");
        let _ = w.write_all(&[b'x'; 400]);
        let _ = w.flush();
        thread::sleep(Duration::from_secs(10));
    }));
    let download = Download::start(request(server.url("/"), std::slice::from_ref(&c.root), Duration::from_secs(30)));
    let started = Instant::now();
    while download.progress().0 < 300 {
        assert!(started.elapsed() < Duration::from_secs(10));
        thread::sleep(Duration::from_millis(10));
    }
    let (received, total, state) = download.progress();
    assert_eq!((total, state), (Some(100_000), State::Running));
    assert!(received >= 300);
    let cancelled = Instant::now();
    download.cancel();
    assert_eq!(download.wait(Duration::from_secs(5)), State::Cancelled);
    assert!(cancelled.elapsed() < Duration::from_millis(100));
    assert_eq!(download.with_outcome(|o| o.clone().unwrap_err()), Some(Failure::cancelled()));
}

#[test]
fn a_download_ends_with_its_body() {
    let c = certs();
    let server = Server::start(&c.leaf, reply("HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhello"));
    let download = Download::start(request(server.url("/"), std::slice::from_ref(&c.root), Duration::from_secs(10)));
    assert_eq!(download.wait(Duration::from_secs(10)), State::Done);
    assert_eq!(download.progress(), (5, Some(5), State::Done));
    assert_eq!(download.with_outcome(|o| o.clone().unwrap()), Some(b"hello".to_vec()));
}

#[test]
fn a_certificate_the_system_does_not_trust_is_refused() {
    let c = certs();
    let stranger = Server::start(&c.stranger, reply("HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n"));
    let failure = get(&stranger, "/", &c).unwrap_err();
    assert_eq!(failure.kind, Kind::Tls);
    assert_eq!(failure.message, "The SSL connection could not be established, see inner exception.");
    assert_eq!(failure.inner.as_deref(), Some("The remote certificate is invalid because of errors in the certificate chain: UntrustedRoot"));
    let ours = Server::start(&c.leaf, reply("HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n"));
    let failure = fetch(&request(ours.url("/"), &[], Duration::from_secs(10)), &Progress::default()).unwrap_err();
    assert_eq!(failure.kind, Kind::Tls);
    assert_eq!(failure.inner.as_deref(), Some("The remote certificate is invalid because of errors in the certificate chain: PartialChain"));
    assert!(ours.heads.lock().unwrap().is_empty());
    let misnamed = Server::start(&c.misnamed, reply("HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n"));
    let trusted = get(&misnamed, "/", &c).unwrap_err();
    assert_eq!(trusted.inner.as_deref(), Some("The remote certificate is invalid according to the validation procedure: RemoteCertificateNameMismatch"));
    let untrusted = fetch(&request(misnamed.url("/"), &[], Duration::from_secs(10)), &Progress::default()).unwrap_err();
    assert_eq!(untrusted.inner.as_deref(), Some("The remote certificate is invalid according to the validation procedure: RemoteCertificateNameMismatch, RemoteCertificateChainErrors"));
}

#[test]
fn what_is_not_http_is_named_by_its_first_line() {
    let c = certs();
    let server = Server::start(&c.leaf, reply("SSH-2.0-OpenSSH\r\n\r\n"));
    assert_eq!(get(&server, "/", &c).unwrap_err().message, "Received an invalid status line: 'SSH-2.0-OpenSSH'.");
}

#[test]
fn a_request_no_byte_answers_is_sent_four_times() {
    let c = certs();
    let server = Server::start(&c.leaf, Arc::new(|_, _| {}));
    assert_eq!(get(&server, "/", &c).unwrap_err(), Failure::unanswered());
    assert_eq!(server.connections.load(Ordering::SeqCst), 4);
}

#[test]
fn a_refused_connection_names_its_host_and_port() {
    let port = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
    let failure = fetch(&request(format!("https://127.0.0.1:{port}/"), &[], Duration::from_secs(10)), &Progress::default()).unwrap_err();
    assert_eq!((failure.kind, failure.message), (Kind::Connect, format!("Connection refused (127.0.0.1:{port})")));
}

#[test]
fn a_url_dotnet_refuses_is_refused_in_its_words() {
    let failure = fetch(&request("ftp://example.org/".into(), &[], Duration::from_secs(1)), &Progress::default()).unwrap_err();
    assert_eq!((failure.kind, failure.message.as_str()), (Kind::Scheme, "The 'ftp' scheme is not supported."));
}
