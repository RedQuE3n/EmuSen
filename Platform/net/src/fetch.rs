//! One GET, its body held whole, as .NET's `HttpClient.GetByteArrayAsync` makes it with a client of default settings
//! and one timeout (EmuSen_Backend.md §16.1 has each rule and how it was measured).

use std::io::{self, Read};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use rustls::pki_types::CertificateDer;
use ureq::config::AutoHeaderValue;
use ureq::unversioned::resolver::DefaultResolver;
use ureq::unversioned::transport::{ConnectProxyConnector, Connector, TcpConnector};
use ureq::{Agent, Body, Error, http};

use crate::failure::{Failure, Kind, LIMIT};
use crate::location::{Refusal, Url};
use crate::tls::{self, Tls};
use crate::wire::{self, Record, Seen, Watch};

/// What to fetch, and the one timeout over the whole call, redirects and body included.
#[derive(Debug, Clone)]
pub struct Request {
    pub url: String,
    pub timeout: Duration,
    /// Roots trusted beside the system's: a test's own server only. No setting or variable reaches this; a host's code sets it.
    pub test_roots: Vec<CertificateDer<'static>>,
}

/// How far a fetch has come, and whether it was asked to stop: written by the fetch, read by anyone.
#[derive(Debug)]
pub struct Progress {
    received: AtomicU64,
    total: AtomicU64,
    cancelled: AtomicBool,
}

impl Default for Progress {
    fn default() -> Self {
        Progress { received: AtomicU64::new(0), total: AtomicU64::new(u64::MAX), cancelled: AtomicBool::new(false) }
    }
}

impl Progress {
    /// The body's bytes held so far.
    pub fn received(&self) -> u64 {
        self.received.load(Ordering::Relaxed)
    }

    /// The body's length, once an answer gives one.
    pub fn total(&self) -> Option<u64> {
        Some(self.total.load(Ordering::Relaxed)).filter(|&t| t != u64::MAX)
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }
}

/// `HttpClientHandler.MaxAutomaticRedirections`' default.
const REDIRECTS: u32 = 50;
/// How often .NET sends a request again that no byte answered.
const RETRIES: u32 = 3;
const CHUNK: usize = 64 * 1024;
/// The most held ahead of a body that gives its length, so that a length is not an allocation a server chooses.
const RESERVE: u64 = 64 * 1024 * 1024;

/// The body of `request.url`, or why there is none.
pub fn fetch(request: &Request, progress: &Progress) -> Result<Vec<u8>, Failure> {
    let deadline = Instant::now().checked_add(request.timeout);
    let mut url = Url::parse(&request.url).map_err(refused)?;
    let config = tls::config(&request.test_roots).map_err(|e| Failure::tls(e.to_string()))?;
    let record = Record::default();
    let agent = agent(config, record.clone());
    let mut redirects = 0;
    loop {
        let response = send(&agent, &record, &url, request.timeout, deadline, progress)?;
        let status = response.status().as_u16();
        if redirects < REDIRECTS && matches!(status, 300 | 301 | 302 | 303 | 307 | 308) {
            if let Some(next) = target(&response, &url) {
                redirects += 1;
                url = next;
                continue;
            }
        }
        if !(200..300).contains(&status) {
            let line = seen(&record).status_line;
            let reason = match &line {
                Some(line) => wire::reason(line).to_string(),
                None => response.status().canonical_reason().unwrap_or("").to_string(),
            };
            return Err(Failure::status(status, &reason));
        }
        return body(response.into_body(), request.timeout, progress);
    }
}

fn refused(refusal: Refusal) -> Failure {
    match refusal {
        Refusal::Relative => Failure::url("An invalid request URI was provided. Either the request URI must be an absolute URI or BaseAddress must be set."),
        Refusal::Scheme(scheme) => Failure { kind: Kind::Scheme, ..Failure::url(format!("The '{scheme}' scheme is not supported.")) },
    }
}

fn agent(tls: Arc<rustls::ClientConfig>, record: Record) -> Agent {
    let config = Agent::config_builder()
        .http_status_as_error(false)
        .max_redirects(0)
        .user_agent(AutoHeaderValue::None)
        .accept(AutoHeaderValue::None)
        .accept_encoding(AutoHeaderValue::None)
        .build();
    let connector = ().chain(ConnectProxyConnector::default()).chain(TcpConnector::default()).chain(Tls { config: tls }).chain(Watch { record });
    Agent::with_parts(config, connector, DefaultResolver::default())
}

fn seen(record: &Record) -> Seen {
    record.lock().unwrap_or_else(|p| p.into_inner()).clone()
}

/// One request and its answer's head, sent again while no byte answers it, as .NET does.
fn send(agent: &Agent, record: &Record, url: &Url, timeout: Duration, deadline: Option<Instant>, progress: &Progress) -> Result<http::Response<Body>, Failure> {
    let mut retries = 0;
    loop {
        if progress.is_cancelled() {
            return Err(Failure::cancelled());
        }
        let remaining = match deadline {
            Some(d) => Some(d.checked_duration_since(Instant::now()).filter(|r| !r.is_zero()).ok_or_else(|| Failure::timeout(timeout))?),
            None => None,
        };
        *record.lock().unwrap_or_else(|p| p.into_inner()) = Seen::default();
        match agent.get(url.request_text()).config().timeout_global(remaining).build().call() {
            Ok(response) => return Ok(response),
            Err(error) => {
                let seen = seen(record);
                let failure = classify(error, &seen, url, timeout);
                if failure.kind == Kind::Unanswered && !seen.answered && retries < RETRIES {
                    retries += 1;
                    continue;
                }
                return Err(failure);
            }
        }
    }
}

/// A redirect's target, where .NET follows it: a `Location` that resolves to http or https, and never from https to http.
fn target(response: &http::Response<Body>, from: &Url) -> Option<Url> {
    let location = response.headers().get(http::header::LOCATION)?;
    let next = from.join(&String::from_utf8_lossy(location.as_bytes())).ok()?;
    (!(from.scheme == "https" && next.scheme == "http")).then_some(next)
}

/// An I/O error's words without Rust's ` (os error N)`, which are the system's `strerror`, as .NET's socket words are.
fn os_words(error: &io::Error) -> String {
    let words = error.to_string();
    match words.rfind(" (os error ") {
        Some(at) => words[..at].to_string(),
        None => words,
    }
}

fn classify(error: Error, seen: &Seen, url: &Url, timeout: Duration) -> Failure {
    let host = url.bare_host();
    match error {
        Error::Timeout(_) => Failure::timeout(timeout),
        Error::Other(cause) => match cause.downcast::<rustls::Error>() {
            Ok(tls) => Failure::tls(tls_words(&tls)),
            Err(cause) => Failure::other(cause.to_string()),
        },
        Error::Tls(words) => Failure::tls(words.to_string()),
        Error::Io(io) if io.kind() == io::ErrorKind::TimedOut => Failure::timeout(timeout),
        Error::Io(io) if !seen.sent => {
            let words = os_words(&io);
            match words.strip_prefix("failed to lookup address information: ") {
                Some(words) if io.raw_os_error().is_none() => Failure::resolve(words, host, url.port),
                _ => Failure::connect(&words, host, url.port),
            }
        }
        Error::HostNotFound => Failure::resolve("Name or service not known", host, url.port),
        Error::ConnectionFailed => Failure::connect("Connection refused", host, url.port),
        Error::Io(_) => Failure::unanswered(),
        Error::Protocol(_) => match &seen.status_line {
            Some(line) if !is_status_line(line) => Failure::invalid_status_line(line),
            _ => Failure::invalid(),
        },
        Error::BadUri(words) => Failure::url(words),
        other => Failure::other(other.to_string()),
    }
}

/// `HTTP/d.d ddd`, then a space or nothing: the line .NET accepts.
fn is_status_line(line: &str) -> bool {
    let b = line.as_bytes();
    b.len() >= 12
        && b.starts_with(b"HTTP/")
        && b[5].is_ascii_digit()
        && b[6] == b'.'
        && b[7].is_ascii_digit()
        && b[8] == b' '
        && b[9..12].iter().all(u8::is_ascii_digit)
        && (b.len() == 12 || b[12] == b' ')
}

/// What .NET's inner `AuthenticationException` says of a certificate rustls refused, as near as its chain status allows.
fn tls_words(error: &rustls::Error) -> String {
    use rustls::CertificateError as C;
    let chain = |status: &str| format!("The remote certificate is invalid because of errors in the certificate chain: {status}");
    match error {
        rustls::Error::InvalidCertificate(C::Expired | C::ExpiredContext { .. } | C::NotValidYet | C::NotValidYetContext { .. }) => chain("NotTimeValid"),
        rustls::Error::InvalidCertificate(C::Revoked) => chain("Revoked"),
        rustls::Error::InvalidCertificate(C::Other(e)) if e.0.is::<tls::Untrusted>() => e.0.to_string(),
        rustls::Error::InvalidCertificate(C::NotValidForName | C::NotValidForNameContext { .. }) => {
            "The remote certificate is invalid according to the validation procedure: RemoteCertificateNameMismatch".to_string()
        }
        other => other.to_string(),
    }
}

fn body(body: Body, timeout: Duration, progress: &Progress) -> Result<Vec<u8>, Failure> {
    let length = body.content_length();
    if let Some(length) = length {
        if length > LIMIT {
            return Err(Failure::limit());
        }
        progress.total.store(length, Ordering::Relaxed);
    }
    let mut reader = body.into_reader();
    let mut bytes = Vec::with_capacity(length.unwrap_or(0).min(RESERVE) as usize);
    let mut chunk = vec![0u8; CHUNK];
    loop {
        if progress.is_cancelled() {
            return Err(Failure::cancelled());
        }
        let n = match reader.read(&mut chunk) {
            Ok(0) => return Ok(bytes),
            Ok(n) => n,
            Err(e) => return Err(body_failure(e, timeout)),
        };
        if bytes.len() as u64 + n as u64 > LIMIT {
            return Err(Failure::limit());
        }
        bytes.extend_from_slice(&chunk[..n]);
        progress.received.store(bytes.len() as u64, Ordering::Relaxed);
    }
}

fn body_failure(error: io::Error, timeout: Duration) -> Failure {
    match Error::from(error) {
        Error::Timeout(_) => Failure::timeout(timeout),
        Error::Io(io) if io.kind() == io::ErrorKind::TimedOut => Failure::timeout(timeout),
        Error::Io(io) if io.kind() == io::ErrorKind::UnexpectedEof => Failure::ended(),
        Error::Io(io) if io.raw_os_error().is_some() => Failure::copy(&os_words(&io)),
        Error::Protocol(_) => Failure::ended(),
        other => Failure::copy(&other.to_string()),
    }
}
