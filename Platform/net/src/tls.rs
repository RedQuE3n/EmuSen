//! TLS over rustls for ureq, with the system's store through `rustls-platform-verifier`: ureq's own rustls connector
//! cannot take a verifier, and a test's extra root needs one (EmuSen_Backend.md §16.1).

use std::io::{self, Read, Write};
use std::sync::Arc;

use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::client::verify_server_name;
use rustls::server::ParsedCertificate;
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{CertificateError, ClientConfig, ClientConnection, DigitallySignedStruct, OtherError, SignatureScheme, StreamOwned};
use ureq::Error;
use ureq::unversioned::transport::{Buffers, ConnectionDetails, Connector, LazyBuffers, NextTimeout, Transport, TransportAdapter};

/// The client's TLS settings: the system's roots, and a test's own root beside them when one is given.
pub fn config(test_roots: &[CertificateDer<'static>]) -> Result<Arc<ClientConfig>, rustls::Error> {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let verifier = if test_roots.is_empty() {
        rustls_platform_verifier::Verifier::new(provider.clone())?
    } else {
        rustls_platform_verifier::Verifier::new_with_extra_roots(test_roots.iter().cloned(), provider.clone())?
    };
    let config = ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()?
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(Chains(verifier)))
        .with_no_client_auth();
    Ok(Arc::new(config))
}

/// A chain no store trusts, in the words .NET's `AuthenticationException` gives it, where rustls says only "unknown issuer".
#[derive(Debug)]
pub struct Untrusted(pub &'static str);

impl std::fmt::Display for Untrusted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for Untrusted {}

/// The platform's verifier, an unknown issuer told apart as .NET's chain status tells it (EmuSen_Backend.md §16.2).
#[derive(Debug)]
struct Chains(rustls_platform_verifier::Verifier);

impl ServerCertVerifier for Chains {
    fn verify_server_cert(&self, leaf: &CertificateDer<'_>, intermediates: &[CertificateDer<'_>], name: &ServerName<'_>, ocsp: &[u8], now: UnixTime) -> Result<ServerCertVerified, rustls::Error> {
        match self.0.verify_server_cert(leaf, intermediates, name, ocsp, now) {
            Err(rustls::Error::InvalidCertificate(CertificateError::UnknownIssuer)) => {
                let misnamed = ParsedCertificate::try_from(leaf).is_ok_and(|c| verify_server_name(&c, name).is_err());
                let words = if misnamed {
                    "The remote certificate is invalid according to the validation procedure: RemoteCertificateNameMismatch, RemoteCertificateChainErrors"
                } else if ends_self_issued(leaf, intermediates) {
                    "The remote certificate is invalid because of errors in the certificate chain: UntrustedRoot"
                } else {
                    "The remote certificate is invalid because of errors in the certificate chain: PartialChain"
                };
                Err(rustls::Error::InvalidCertificate(CertificateError::Other(OtherError(Arc::new(Untrusted(words))))))
            }
            other => other,
        }
    }

    fn verify_tls12_signature(&self, message: &[u8], cert: &CertificateDer<'_>, dss: &DigitallySignedStruct) -> Result<HandshakeSignatureValid, rustls::Error> {
        self.0.verify_tls12_signature(message, cert, dss)
    }

    fn verify_tls13_signature(&self, message: &[u8], cert: &CertificateDer<'_>, dss: &DigitallySignedStruct) -> Result<HandshakeSignatureValid, rustls::Error> {
        self.0.verify_tls13_signature(message, cert, dss)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.0.supported_verify_schemes()
    }
}

/// Whether following issuer to subject from the leaf through what the server sent ends at a certificate that issued itself.
fn ends_self_issued(leaf: &CertificateDer<'_>, sent: &[CertificateDer<'_>]) -> bool {
    let mut current = match names(leaf) {
        Some(n) => n,
        None => return false,
    };
    for _ in 0..=sent.len() {
        if current.0 == current.1 {
            return true;
        }
        match sent.iter().filter_map(|c| names(c)).find(|(_, subject)| *subject == current.0) {
            Some(next) => current = next,
            None => return false,
        }
    }
    false
}

/// A certificate's issuer and subject, as their DER bytes (RFC 5280 §4.1).
fn names<'a>(cert: &'a CertificateDer<'_>) -> Option<(&'a [u8], &'a [u8])> {
    let (_, certificate, _) = tlv(cert.as_ref())?;
    let (_, mut tbs, _) = tlv(certificate)?;
    let mut fields = Vec::with_capacity(6);
    while fields.len() < 6 && !tbs.is_empty() {
        let (tag, _, rest) = tlv(tbs)?;
        let whole = &tbs[..tbs.len() - rest.len()];
        if !(fields.is_empty() && tag == 0xA0) {
            fields.push(whole);
        }
        tbs = rest;
    }
    // serialNumber, signature, issuer, validity, subject.
    Some((*fields.get(2)?, *fields.get(4)?))
}

/// One DER element: its tag, its contents and what follows it.
fn tlv(der: &[u8]) -> Option<(u8, &[u8], &[u8])> {
    let (&tag, rest) = der.split_first()?;
    let (&first, rest) = rest.split_first()?;
    let (length, rest) = if first < 0x80 {
        (first as usize, rest)
    } else {
        let count = (first & 0x7F) as usize;
        if count == 0 || count > 4 || rest.len() < count {
            return None;
        }
        (rest[..count].iter().fold(0usize, |n, &b| (n << 8) | b as usize), &rest[count..])
    };
    (rest.len() >= length).then(|| (tag, &rest[..length], &rest[length..]))
}

/// Wraps an https connection in TLS; an http one passes as it came.
#[derive(Debug)]
pub struct Tls {
    pub config: Arc<ClientConfig>,
}

/// A handshake's failure: a TLS refusal as rustls words it, anything else as ureq had it.
fn handshake_error(error: io::Error) -> Error {
    if error.get_ref().is_some_and(|e| e.is::<rustls::Error>()) {
        Error::Other(error.into_inner().expect("checked above"))
    } else {
        Error::from(error)
    }
}

impl<In: Transport> Connector<In> for Tls {
    type Out = Box<dyn Transport>;

    fn connect(&self, details: &ConnectionDetails, chained: Option<In>) -> Result<Option<Self::Out>, Error> {
        let Some(transport) = chained else { return Ok(None) };
        if !details.needs_tls() || transport.is_tls() {
            return Ok(Some(transport.boxed()));
        }
        let host = details.uri.host().unwrap_or("").trim_start_matches('[').trim_end_matches(']');
        let name = ServerName::try_from(host.to_string()).map_err(|_| Error::Tls("the host is not a name TLS can check"))?;
        let mut connection = ClientConnection::new(self.config.clone(), name).map_err(|e| Error::Other(Box::new(e)))?;
        let mut socket = TransportAdapter::new(transport.boxed());
        socket.set_timeout(details.timeout);
        connection.complete_io(&mut socket).map_err(handshake_error)?;
        let buffers = LazyBuffers::new(details.config.input_buffer_size(), details.config.output_buffer_size());
        Ok(Some(Box::new(TlsTransport { buffers, stream: StreamOwned::new(connection, socket) })))
    }
}

pub struct TlsTransport {
    buffers: LazyBuffers,
    stream: StreamOwned<ClientConnection, TransportAdapter>,
}

impl std::fmt::Debug for TlsTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TlsTransport")
    }
}

impl Transport for TlsTransport {
    fn buffers(&mut self) -> &mut dyn Buffers {
        &mut self.buffers
    }

    fn transmit_output(&mut self, amount: usize, timeout: NextTimeout) -> Result<(), Error> {
        self.stream.get_mut().set_timeout(timeout);
        let output = &self.buffers.output()[..amount];
        self.stream.write_all(output).map_err(handshake_error)
    }

    fn await_input(&mut self, timeout: NextTimeout) -> Result<bool, Error> {
        self.stream.get_mut().set_timeout(timeout);
        let input = self.buffers.input_append_buf();
        let amount = self.stream.read(input).map_err(handshake_error)?;
        self.buffers.input_appended(amount);
        Ok(amount > 0)
    }

    fn is_open(&mut self) -> bool {
        self.stream.get_mut().get_mut().is_open()
    }

    fn is_tls(&self) -> bool {
        true
    }
}
