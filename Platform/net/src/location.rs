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
        let path = if path.is_empty() { "/".to_string() } else { remove_dot_segments(&path.replace('\\', "/")) };
        Ok(Url { scheme, host, port, explicit_port, path: escape(&path), query: escape(query) })
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
        let path = path.replace('\\', "/");
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
            next.path = escape(&remove_dot_segments(&merged));
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

/// What a request line cannot carry, escaped as .NET's `Uri` escapes it: spaces, controls, non-ASCII and `"<>\^`{|}`.
fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for byte in text.bytes() {
        if byte <= b' ' || byte >= 0x7F || matches!(byte, b'"' | b'<' | b'>' | b'\\' | b'^' | b'`' | b'{' | b'|' | b'}') {
            out.push_str(&format!("%{byte:02X}"));
        } else {
            out.push(byte as char);
        }
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
}
