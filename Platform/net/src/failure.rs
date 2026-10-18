//! Why a fetch gave no body, in the words .NET's `HttpClient` gives (measured on .NET 10.0.112 on 2026-10-18; the
//! table is EmuSen_Backend.md §16.2).

use std::fmt;
use std::time::Duration;

/// What kind of failure it was: the host chooses the exception it throws by it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A final answer whose status is not 2xx, a redirect not followed included.
    Status,
    /// The whole call outlasted its timeout.
    Timeout,
    /// The body ended before its length or its last chunk.
    Ended,
    /// The connection ended before any byte of an answer, after the request and its three retries.
    Unanswered,
    /// The TLS handshake failed: a certificate refused, or the server's TLS refused.
    Tls,
    /// The connection could not be made.
    Connect,
    /// The host's name could not be resolved.
    Resolve,
    /// What came back is not an HTTP answer.
    Invalid,
    /// A body larger than .NET's buffer holds.
    Limit,
    /// The host cancelled the download.
    Cancelled,
    /// A URL that is not absolute.
    Url,
    /// A scheme other than http and https.
    Scheme,
    /// Anything else that went wrong reading the answer.
    Other,
}

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Kind::Status => "status",
            Kind::Timeout => "timeout",
            Kind::Ended => "ended",
            Kind::Unanswered => "unanswered",
            Kind::Tls => "tls",
            Kind::Connect => "connect",
            Kind::Resolve => "resolve",
            Kind::Invalid => "invalid",
            Kind::Limit => "limit",
            Kind::Cancelled => "cancelled",
            Kind::Url => "url",
            Kind::Scheme => "scheme",
            Kind::Other => "other",
        }
    }
}

/// A failure: its kind, the words a player is shown, the words of the cause .NET would carry inside them, and the status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    pub kind: Kind,
    pub message: String,
    pub inner: Option<String>,
    pub status: Option<u16>,
}

/// The most a body may hold: `HttpClient.MaxResponseContentBufferSize`'s default, `int.MaxValue`.
pub const LIMIT: u64 = i32::MAX as u64;

const ENDED: &str = "The response ended prematurely.";

impl Failure {
    fn new(kind: Kind, message: impl Into<String>, inner: Option<String>) -> Failure {
        Failure { kind, message: message.into(), inner, status: None }
    }

    /// `EnsureSuccessStatusCode`'s words: the reason the server sent, and none when it sent none.
    pub fn status(code: u16, reason: &str) -> Failure {
        let message = if reason.is_empty() {
            format!("Response status code does not indicate success: {code}.")
        } else {
            format!("Response status code does not indicate success: {code} ({reason}).")
        };
        Failure { status: Some(code), ..Failure::new(Kind::Status, message, None) }
    }

    pub fn timeout(timeout: Duration) -> Failure {
        let message = format!("The request was canceled due to the configured HttpClient.Timeout of {} seconds elapsing.", seconds(timeout));
        Failure::new(Kind::Timeout, message, Some("The operation was canceled.".into()))
    }

    pub fn ended() -> Failure {
        Failure::new(Kind::Ended, "Error while copying content to a stream.", Some(ENDED.into()))
    }

    pub fn unanswered() -> Failure {
        Failure::new(Kind::Unanswered, "An error occurred while sending the request.", Some(ENDED.into()))
    }

    pub fn tls(inner: String) -> Failure {
        Failure::new(Kind::Tls, "The SSL connection could not be established, see inner exception.", Some(inner))
    }

    /// A socket's words with the place .NET appends, `host:port`.
    pub fn connect(words: &str, host: &str, port: u16) -> Failure {
        Failure::new(Kind::Connect, format!("{words} ({host}:{port})"), Some(words.to_string()))
    }

    pub fn resolve(words: &str, host: &str, port: u16) -> Failure {
        Failure::new(Kind::Resolve, format!("{words} ({host}:{port})"), Some(words.to_string()))
    }

    pub fn invalid_status_line(line: &str) -> Failure {
        Failure::new(Kind::Invalid, format!("Received an invalid status line: '{line}'."), None)
    }

    pub fn invalid() -> Failure {
        Failure::new(Kind::Invalid, "The server returned an invalid or unrecognized response.", None)
    }

    pub fn limit() -> Failure {
        Failure::new(Kind::Limit, format!("Cannot write more bytes to the buffer than the configured maximum buffer size: {LIMIT}."), None)
    }

    pub fn cancelled() -> Failure {
        Failure::new(Kind::Cancelled, "The operation was canceled.", None)
    }

    pub fn url(message: impl Into<String>) -> Failure {
        Failure::new(Kind::Url, message, None)
    }

    /// A read of the body that failed for a reason other than its end, with the socket's words inside.
    pub fn copy(words: &str) -> Failure {
        Failure::new(Kind::Other, "Error while copying content to a stream.", Some(format!("Unable to read data from the transport connection: {words}.")))
    }

    pub fn other(message: impl Into<String>) -> Failure {
        Failure::new(Kind::Other, message, None)
    }
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Failure {}

/// A timeout's seconds as .NET writes `TimeSpan.TotalSeconds`: the shortest text that reads back as the same double.
fn seconds(timeout: Duration) -> String {
    let s = timeout.as_secs_f64();
    if s.fract() == 0.0 { format!("{}", s as u64) } else { format!("{s}") }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_status_words_carry_the_reason_only_when_one_was_sent() {
        assert_eq!(Failure::status(404, "Not Found").message, "Response status code does not indicate success: 404 (Not Found).");
        assert_eq!(Failure::status(404, "").message, "Response status code does not indicate success: 404.");
    }

    #[test]
    fn a_timeout_names_its_seconds_as_dotnet_writes_them() {
        assert_eq!(Failure::timeout(Duration::from_secs(300)).message, "The request was canceled due to the configured HttpClient.Timeout of 300 seconds elapsing.");
        assert_eq!(Failure::timeout(Duration::from_millis(1500)).message, "The request was canceled due to the configured HttpClient.Timeout of 1.5 seconds elapsing.");
    }
}
