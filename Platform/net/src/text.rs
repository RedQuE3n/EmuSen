//! A body read as text as .NET's `HttpContent.ReadAsStringAsync` reads it: the `Content-Type`'s charset when it names
//! one, its preamble taken off only when the bytes start with it; with none, a UTF-8, UTF-32 or UTF-16 byte-order mark
//! read and taken off, else UTF-8; a sequence that does not decode replaced by U+FFFD. Measured on .NET 10 on
//! 2026-10-18 (EmuSen_Backend.md §30.5); the charsets are those .NET has with no code-page provider registered.

use crate::failure::Failure;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Encoding {
    Utf8,
    Ascii,
    Latin1,
    Utf16Le,
    Utf16Be,
    Utf32Le,
    Utf32Be,
}

impl Encoding {
    fn named(name: &str) -> Option<Encoding> {
        Some(match name.trim().to_ascii_lowercase().as_str() {
            "utf-8" | "utf8" | "unicode-1-1-utf-8" | "unicode-2-0-utf-8" | "x-unicode20utf8" => Encoding::Utf8,
            "us-ascii" | "ascii" | "us" | "ansi_x3.4-1968" | "ansi_x3.4-1986" | "cp367" | "csascii" | "ibm367" | "iso-ir-6" | "iso646-us" | "iso_646.irv:1991" => Encoding::Ascii,
            "iso-8859-1" | "latin1" | "l1" | "cp819" | "csisolatin1" | "ibm819" | "iso-ir-100" | "iso8859-1" | "iso_8859-1" | "iso_8859-1:1987" => Encoding::Latin1,
            "utf-16" | "utf-16le" | "unicode" | "ucs-2" | "iso-10646-ucs-2" | "csunicode" => Encoding::Utf16Le,
            "utf-16be" | "unicodefffe" => Encoding::Utf16Be,
            "utf-32" | "utf-32le" => Encoding::Utf32Le,
            "utf-32be" => Encoding::Utf32Be,
            _ => return None,
        })
    }

    fn preamble(self) -> &'static [u8] {
        match self {
            Encoding::Utf8 => &[0xEF, 0xBB, 0xBF],
            Encoding::Utf16Le => &[0xFF, 0xFE],
            Encoding::Utf16Be => &[0xFE, 0xFF],
            Encoding::Utf32Le => &[0xFF, 0xFE, 0x00, 0x00],
            Encoding::Utf32Be => &[0x00, 0x00, 0xFE, 0xFF],
            Encoding::Ascii | Encoding::Latin1 => &[],
        }
    }

    fn decode(self, bytes: &[u8]) -> String {
        match self {
            Encoding::Utf8 => String::from_utf8_lossy(bytes).into_owned(),
            Encoding::Ascii => bytes.iter().map(|&b| if b < 0x80 { b as char } else { '?' }).collect(),
            Encoding::Latin1 => bytes.iter().map(|&b| b as char).collect(),
            Encoding::Utf16Le | Encoding::Utf16Be => {
                let units: Vec<u16> =
                    bytes.chunks_exact(2).map(|c| if self == Encoding::Utf16Le { u16::from_le_bytes([c[0], c[1]]) } else { u16::from_be_bytes([c[0], c[1]]) }).collect();
                let mut text = String::from_utf16_lossy(&units);
                if bytes.len() % 2 == 1 {
                    text.push('\u{FFFD}');
                }
                text
            }
            Encoding::Utf32Le | Encoding::Utf32Be => {
                let mut text: String = bytes.chunks_exact(4).map(|c| {
                    let word = if self == Encoding::Utf32Le { u32::from_le_bytes([c[0], c[1], c[2], c[3]]) } else { u32::from_be_bytes([c[0], c[1], c[2], c[3]]) };
                    char::from_u32(word).unwrap_or('\u{FFFD}')
                }).collect();
                if bytes.len() % 4 != 0 {
                    text.push('\u{FFFD}');
                }
                text
            }
        }
    }
}

/// The charset a `Content-Type` names, its quotes taken off; none where it names none.
fn charset(content_type: &str) -> Option<String> {
    content_type.split(';').skip(1).find_map(|parameter| {
        let (name, value) = parameter.split_once('=')?;
        name.trim().eq_ignore_ascii_case("charset").then(|| value.trim().trim_matches('"').to_string())
    })
}

/// The body as `ReadAsStringAsync` gives it, or .NET's refusal of a charset it does not have.
pub fn text(content_type: Option<&str>, body: &[u8]) -> Result<String, Failure> {
    if body.is_empty() {
        return Ok(String::new());
    }
    match content_type.and_then(charset).filter(|c| !c.is_empty()) {
        Some(name) => {
            let Some(encoding) = Encoding::named(&name) else {
                return Err(Failure::other("The character set provided in ContentType is invalid. Cannot read content as string using an invalid character set."));
            };
            let body = body.strip_prefix(encoding.preamble()).filter(|_| !encoding.preamble().is_empty()).unwrap_or(body);
            Ok(encoding.decode(body))
        }
        None => {
            for encoding in [Encoding::Utf8, Encoding::Utf32Le, Encoding::Utf16Le, Encoding::Utf16Be, Encoding::Utf32Be] {
                if let Some(rest) = body.strip_prefix(encoding.preamble()) {
                    return Ok(encoding.decode(rest));
                }
            }
            Ok(Encoding::Utf8.decode(body))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::text;

    fn units(s: &str) -> Vec<u32> {
        s.chars().map(|c| c as u32).collect()
    }

    /// The cases measured against .NET 10's `ReadAsStringAsync` on 2026-10-18.
    #[test]
    fn what_dotnet_read_from_each_body() {
        assert_eq!(text(Some("text/html"), "Erreur : trouvée !".as_bytes()).unwrap(), "Erreur : trouvée !");
        assert_eq!(text(None, &[0xEF, 0xBB, 0xBF, b'h', b'i']).unwrap(), "hi");
        assert_eq!(text(None, &[0xFF, 0xFE, b'h', 0, b'i', 0]).unwrap(), "hi");
        assert_eq!(units(&text(Some("text/plain; charset=iso-8859-1"), &[b'v', 0xE9]).unwrap()), [0x76, 0xE9]);
        assert_eq!(units(&text(Some("application/json"), &[b'a', 0xC3, b'b', 0xFF]).unwrap()), [0x61, 0xFFFD, 0x62, 0xFFFD]);
        assert_eq!(text(Some("text/plain; charset=\"utf-8\""), b"q").unwrap(), "q");
        assert_eq!(
            text(Some("text/plain; charset=klingon"), b"k").unwrap_err().message,
            "The character set provided in ContentType is invalid. Cannot read content as string using an invalid character set."
        );
        assert_eq!(units(&text(Some("text/plain; charset=iso-8859-1"), &[0xEF, 0xBB, 0xBF, b'x']).unwrap()), [0xEF, 0xBB, 0xBF, 0x78]);
        assert_eq!(text(None, &[]).unwrap(), "");
    }
}
