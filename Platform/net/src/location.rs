//! An http or https URL, and a `Location` resolved against one as RFC 3986 §5.2 resolves a reference, which is how
//! .NET's `Uri(Uri, string)` gives a redirect's target.

/// An absolute http or https URL, split as a request needs it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Url {
    /// `http` or `https`, lower case.
    pub scheme: String,
    /// The host as written, lower case, an IPv6 address in its brackets.
    pub host: String,
    /// The port given, or the scheme's.
    pub port: u16,
    /// Whether the port was written.
    explicit_port: bool,
    /// The path, starting with `/`.
    pub path: String,
    /// The query with its `?`, or empty.
    pub query: String,
}

/// Why a text is not a URL a request can go to, as .NET refuses it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// Not an absolute URL.
    Relative,
    /// A scheme other than http and https, named.
    Scheme(String),
}

impl Url {
    pub fn parse(text: &str) -> Result<Url, Refusal> {
        let text = text.trim();
        let Some((scheme, rest)) = split_scheme(text) else { return Err(Refusal::Relative) };
        let scheme = scheme.to_ascii_lowercase();
        if scheme != "http" && scheme != "https" {
            return Err(Refusal::Scheme(scheme));
        }
        let Some(rest) = rest.strip_prefix("//").or_else(|| rest.strip_prefix("\\\\")) else { return Err(Refusal::Relative) };
        let (authority, tail) = rest.split_at(rest.find(['/', '\\', '?', '#']).unwrap_or(rest.len()));
        let (host, port) = split_authority(authority).ok_or(Refusal::Relative)?;
        let (explicit_port, port) = match port {
            Some(p) => (true, p),
            None => (false, if scheme == "https" { 443 } else { 80 }),
        };
        let (path, query, _) = split_reference(tail);
        let path = if path.is_empty() { "/".to_string() } else { canonical_path(path) };
        Ok(Url { scheme, host, port, explicit_port, path, query: escape(query) })
    }

    /// Where a `Location` sends a request from this URL, or a refusal when it names nowhere a request can go.
    pub fn join(&self, reference: &str) -> Result<Url, Refusal> {
        let reference = reference.trim();
        if split_scheme(reference).is_some() {
            return Url::parse(reference);
        }
        if let Some(rest) = reference.strip_prefix("//") {
            return Url::parse(&format!("{}://{rest}", self.scheme));
        }
        let (path, query, _) = split_reference(reference);
        let path = escape(&path.replace('\\', "/"));
        let mut next = self.clone();
        if path.is_empty() {
            if !query.is_empty() || reference.starts_with('?') {
                next.query = escape(query);
            }
        } else {
            let merged = if path.starts_with('/') {
                path
            } else {
                let base = &self.path[..self.path.rfind('/').map_or(0, |i| i + 1)];
                format!("{base}{path}")
            };
            next.path = remove_dot_segments(&merged);
            next.query = escape(query);
        }
        Ok(next)
    }

    /// `host:port` as the request's `Host` header and .NET's connection words give it: the port only when written.
    pub fn authority(&self) -> String {
        if self.explicit_port && self.port != if self.scheme == "https" { 443 } else { 80 } { format!("{}:{}", self.host, self.port) } else { self.host.clone() }
    }

    /// The host without an IPv6 address's brackets.
    pub fn bare_host(&self) -> &str {
        self.host.trim_start_matches('[').trim_end_matches(']')
    }

    /// What the request is sent to, without a fragment.
    pub fn request_text(&self) -> String {
        format!("{}://{}{}{}", self.scheme, self.authority(), self.path, self.query)
    }
}

fn split_scheme(text: &str) -> Option<(&str, &str)> {
    let colon = text.find(':')?;
    let scheme = &text[..colon];
    let mut chars = scheme.chars();
    let first = chars.next()?;
    (first.is_ascii_alphabetic() && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))).then(|| (scheme, &text[colon + 1..]))
}

/// The host, lower case, and the port when one is written; none for an authority with no host or a port that is not one.
fn split_authority(authority: &str) -> Option<(String, Option<u16>)> {
    let authority = authority.rsplit_once('@').map_or(authority, |(_, a)| a);
    let (host, port) = if authority.starts_with('[') {
        let end = authority.find(']')?;
        let rest = &authority[end + 1..];
        (&authority[..=end], if rest.is_empty() { None } else { Some(rest.strip_prefix(':')?) })
    } else {
        match authority.rsplit_once(':') {
            Some((h, p)) => (h, Some(p)),
            None => (authority, None),
        }
    };
    if host.is_empty() {
        return None;
    }
    let port = match port {
        None | Some("") => None,
        Some(p) => Some(p.parse::<u16>().ok()?),
    };
    Some((host.to_ascii_lowercase(), port))
}

/// A reference's path, query (with its `?`) and fragment.
fn split_reference(text: &str) -> (&str, &str, &str) {
    let (rest, fragment) = text.split_at(text.find('#').unwrap_or(text.len()));
    let (path, query) = rest.split_at(rest.find('?').unwrap_or(rest.len()));
    (path, query, fragment)
}

/// RFC 3986 §5.2.4.
fn remove_dot_segments(path: &str) -> String {
    let mut out: Vec<&str> = Vec::new();
    let segments: Vec<&str> = path.split('/').collect();
    for (i, segment) in segments.iter().enumerate() {
        let last = i == segments.len() - 1;
        match *segment {
            "." => {
                if last {
                    out.push("");
                }
            }
            ".." => {
                if out.len() > 1 {
                    out.pop();
                }
                if last {
                    out.push("");
                }
            }
            s => out.push(s),
        }
    }
    let joined = out.join("/");
    if joined.starts_with('/') { joined } else { format!("/{joined}") }
}

/// A path as .NET's `Uri` gives it: `\\` as `/`, then escaped as [`escape`] does, then its dot segments removed, so a
/// `%2e` counts as the dot it unescapes to and a `%2F` stays inside its segment.
fn canonical_path(path: &str) -> String {
    remove_dot_segments(&escape(&path.replace('\\', "/")))
}

fn hex_value(byte: u8) -> Option<u8> {
    (byte as char).to_digit(16).map(|d| d as u8)
}

fn unreserved(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~')
}

/// What `Uri` does to the text of a path or query (measured on .NET 10 on 2026-10-18, EmuSen_Backend.md §30.5): an
/// escaped unreserved character unescaped; an escaped UTF-8 sequence that decodes written again in upper case; any other
/// escape kept as written; a `%` that starts no escape written `%25`; spaces, controls, non-ASCII and `"<>\\^`{|}`
/// escaped in upper case; everything else as it is.
fn escape(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let escaped = |at: usize| -> Option<u8> {
        if bytes.get(at) != Some(&b'%') {
            return None;
        }
        Some(hex_value(*bytes.get(at + 1)?)? * 16 + hex_value(*bytes.get(at + 2)?)?)
    };
    let mut i = 0;
    while i < bytes.len() {
        let byte = bytes[i];
        if byte == b'%' {
            let Some(value) = escaped(i) else {
                out.push_str("%25");
                i += 1;
                continue;
            };
            if unreserved(value) {
                out.push(value as char);
                i += 3;
                continue;
            }
            let length = match value {
                0xC2..=0xDF => 2,
                0xE0..=0xEF => 3,
                0xF0..=0xF4 => 4,
                _ => 0,
            };
            let sequence: Option<Vec<u8>> = (length > 0).then(|| (0..length).map(|k| escaped(i + 3 * k)).collect::<Option<Vec<u8>>>()).flatten();
            match sequence.filter(|s| std::str::from_utf8(s).is_ok()) {
                Some(sequence) => {
                    for b in &sequence {
                        out.push_str(&format!("%{b:02X}"));
                    }
                    i += 3 * sequence.len();
                }
                None => {
                    out.push_str(&text[i..i + 3]);
                    i += 3;
                }
            }
            continue;
        }
        if byte <= b' ' || byte >= 0x7F || matches!(byte, b'"' | b'<' | b'>' | b'\\' | b'^' | b'`' | b'{' | b'|' | b'}') {
            out.push_str(&format!("%{byte:02X}"));
        } else {
            out.push(byte as char);
        }
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn joined(base: &str, reference: &str) -> String {
        Url::parse(base).unwrap().join(reference).unwrap().request_text()
    }

    #[test]
    fn rfc_3986s_normal_examples_resolve_as_written() {
        let base = "http://a/b/c/d;p?q";
        for (reference, expected) in [
            ("g", "http://a/b/c/g"),
            ("./g", "http://a/b/c/g"),
            ("g/", "http://a/b/c/g/"),
            ("/g", "http://a/g"),
            ("//g", "http://g/"),
            ("?y", "http://a/b/c/d;p?y"),
            ("g?y", "http://a/b/c/g?y"),
            ("#s", "http://a/b/c/d;p?q"),
            ("g#s", "http://a/b/c/g"),
            (";x", "http://a/b/c/;x"),
            ("", "http://a/b/c/d;p?q"),
            (".", "http://a/b/c/"),
            ("./", "http://a/b/c/"),
            ("..", "http://a/b/"),
            ("../g", "http://a/b/g"),
            ("../..", "http://a/"),
            ("../../g", "http://a/g"),
            ("../../../g", "http://a/g"),
            ("/./g", "http://a/g"),
            ("g/../h", "http://a/b/c/h"),
        ] {
            assert_eq!(joined(base, reference), expected, "{reference}");
        }
    }

    #[test]
    fn a_url_keeps_its_written_port_and_refuses_what_is_not_http() {
        let url = Url::parse("HTTPS://LocalHost:8443/x?y#z").unwrap();
        assert_eq!((url.scheme.as_str(), url.host.as_str(), url.port, url.authority()), ("https", "localhost", 8443, "localhost:8443".to_string()));
        assert_eq!(url.request_text(), "https://localhost:8443/x?y");
        assert_eq!(Url::parse("https://example.org").unwrap().request_text(), "https://example.org/");
        assert_eq!(Url::parse("https://[::1]:9/").unwrap().bare_host(), "::1");
        assert_eq!(Url::parse("ftp://example.org/"), Err(Refusal::Scheme("ftp".into())));
        assert_eq!(Url::parse("/cheats.zip"), Err(Refusal::Relative));
        assert_eq!(Url::parse("https://a/b").unwrap().join("ftp://c/"), Err(Refusal::Scheme("ftp".into())));
    }

    #[test]
    fn what_a_request_line_cannot_carry_is_escaped() {
        assert_eq!(joined("https://a/b", "/c d/é"), "https://a/c%20d/%C3%A9");
    }

    /// Each address's `PathAndQuery` and request line as .NET 10 sent them, measured on 2026-10-18.
    #[test]
    fn a_given_address_is_sent_as_dotnets_uri_writes_it() {
        for (given, sent) in [
            ("/a?media=box-2D(us)", "/a?media=box-2D(us)"),
            ("/a?x=%41%2f%2F%7e%7E%20", "/a?x=A%2f%2F~~%20"),
            ("/a?x=a b", "/a?x=a%20b"),
            ("/a?x=é", "/a?x=%C3%A9"),
            ("/a?x=|{}[]^`\"<>", "/a?x=%7C%7B%7D[]%5E%60%22%3C%3E"),
            ("/a?x='!*;,:@$", "/a?x='!*;,:@$"),
            ("/a//b/./c/../d", "/a//b/d"),
            ("/a/%2e%2E/b", "/b"),
            ("/a?x=+%2B", "/a?x=+%2B"),
            ("/a?x=1#frag", "/a?x=1"),
            ("/a?x=1?y=2&&z", "/a?x=1?y=2&&z"),
            ("/a\\b?x=\\", "/a/b?x=%5C"),
            ("/a?x=%zz%", "/a?x=%25zz%25"),
            ("/a?x=%", "/a?x=%25"),
            ("/a?x=%c3%a9", "/a?x=%C3%A9"),
            ("/%41b", "/Ab"),
            ("/a b/c", "/a%20b/c"),
            ("/a?x=%25", "/a?x=%25"),
            ("/a?x=%00", "/a?x=%00"),
            ("/ä?é", "/%C3%A4?%C3%A9"),
            ("/a?", "/a?"),
            ("/a?x=1\r\n2", "/a?x=1%0D%0A2"),
            ("/a?x=%FF", "/a?x=%FF"),
            ("/a?x=%C3", "/a?x=%C3"),
            ("/a?x=%E2%82", "/a?x=%E2%82"),
            ("/a?x=\u{1}", "/a?x=%01"),
            ("/a?x=\u{7f}", "/a?x=%7F"),
            ("/a?x=%2e%2E.", "/a?x=..."),
            ("/a/%2e/b", "/a/b"),
            ("/%7E%7e/", "/~~/"),
            ("/a?x=%5c%5C", "/a?x=%5c%5C"),
            ("/a?x=%61%5A%30%2D%5F", "/a?x=aZ0-_"),
            ("/a?x=%e2%82%ac", "/a?x=%E2%82%AC"),
            ("/a?x=%C3%A9%FF", "/a?x=%C3%A9%FF"),
            ("/a?x=%c3x", "/a?x=%c3x"),
            ("/a/b%2Fc/../d", "/a/d"),
            ("/a?x=a%2", "/a?x=a%252"),
            ("/a?x=%G0", "/a?x=%25G0"),
            ("/a?x=\u{20ac}%E2%82%AC", "/a?x=%E2%82%AC%E2%82%AC"),
            ("/a?x=%ED%A0%80", "/a?x=%ED%A0%80"),
            ("/a?x=%F0%9F%98%80", "/a?x=%F0%9F%98%80"),
            ("/a?x=%C0%AF", "/a?x=%C0%AF"),
            ("/a//?x=//", "/a//?x=//"),
            ("/a/./?x=./", "/a/?x=./"),
            ("/A/b?X=Y", "/A/b?X=Y"),
        ] {
            let url = Url::parse(&format!("http://127.0.0.1:9{given}")).unwrap();
            assert_eq!(format!("{}{}", url.path, url.query), sent, "{given:?}");
        }
    }
}
