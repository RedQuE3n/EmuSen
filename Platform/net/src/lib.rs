//! The platform's one HTTP and TLS client (EmuSen_Backend.md SR9 and §16): `ureq` over `rustls`, certificates checked
//! against the system's own store through `rustls-platform-verifier`.
//!
//! A fetch is a GET whose body is held whole, as the C# `CheatDatabaseInstaller.Fetch` holds it with .NET's
//! `HttpClient.GetByteArrayAsync`; it keeps that client's behaviour (its request, its redirects, its one timeout over the
//! whole call, its retries of a request no byte answered, its limit) and its words for every failure, so that a host
//! moved onto this crate tells the player what it told them before. A [`Download`] runs one on a thread of its own,
//! and the host drains its progress, as the frontend API's pull rule asks; nothing calls back.

mod download;
mod failure;
mod fetch;
mod location;
mod tls;
mod wire;

pub use download::{Download, State};
pub use failure::{Failure, Kind};
pub use fetch::{Progress, Request, fetch};

pub use rustls::pki_types::CertificateDer;

/// The certificates of a PEM text, for [`Request::test_roots`]; none when the text holds one that does not read.
pub fn pem_certificates(text: &str) -> Option<Vec<CertificateDer<'static>>> {
    use rustls::pki_types::pem::PemObject;
    CertificateDer::pem_slice_iter(text.as_bytes()).collect::<Result<Vec<_>, _>>().ok()
}
