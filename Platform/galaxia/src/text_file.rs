//! A text file a person may have written: decoded by its byte-order mark, UTF-8 without one, and what is not valid
//! read as U+FFFD. Ours because no crate the platform uses reads UTF-32's marks (EmuSen_RustPlatform.md §47.9).

/// A file's bytes as text: a mark chooses UTF-8, UTF-16 or UTF-32 and is dropped.
pub fn decode(bytes: &[u8]) -> String {
    fn utf16(bytes: &[u8], unit: fn([u8; 2]) -> u16) -> String {
        let (pairs, odd) = bytes.as_chunks::<2>();
        let mut text: String = char::decode_utf16(pairs.iter().map(|pair| unit(*pair))).map(|c| c.unwrap_or('\u{FFFD}')).collect();
        if !odd.is_empty() {
            text.push('\u{FFFD}');
        }
        text
    }
    fn utf32(bytes: &[u8], unit: fn([u8; 4]) -> u32) -> String {
        bytes.chunks(4).map(|four| <[u8; 4]>::try_from(four).ok().and_then(|four| char::from_u32(unit(four))).unwrap_or('\u{FFFD}')).collect()
    }
    match bytes {
        [0xFE, 0xFF, rest @ ..] => utf16(rest, u16::from_be_bytes),
        [0xFF, 0xFE, 0, 0, rest @ ..] => utf32(rest, u32::from_le_bytes),
        [0xFF, 0xFE, rest @ ..] => utf16(rest, u16::from_le_bytes),
        [0xEF, 0xBB, 0xBF, rest @ ..] => String::from_utf8_lossy(rest).into_owned(),
        [0, 0, 0xFE, 0xFF, rest @ ..] => utf32(rest, u32::from_be_bytes),
        _ => String::from_utf8_lossy(bytes).into_owned(),
    }
}

/// The text of the file at `path`, decoded as [`decode`] decodes it; the system's words, after the path, when it will not read.
pub fn read(path: &str) -> Result<String, String> {
    std::fs::read(path).map(|bytes| decode(&bytes)).map_err(|e| format!("{path}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::decode;

    #[test]
    fn every_mark_chooses_its_encoding_and_what_is_not_valid_is_a_replacement() {
        let text = "{\"a\": \"é😀\"}";
        assert_eq!(decode(text.as_bytes()), text);
        assert_eq!(decode(&[&[0xEF, 0xBB, 0xBF][..], text.as_bytes()].concat()), text);
        let le: Vec<u8> = text.encode_utf16().flat_map(u16::to_le_bytes).collect();
        let be: Vec<u8> = text.encode_utf16().flat_map(u16::to_be_bytes).collect();
        assert_eq!(decode(&[&[0xFF, 0xFE][..], &le].concat()), text);
        assert_eq!(decode(&[&[0xFE, 0xFF][..], &be].concat()), text);
        let le32: Vec<u8> = text.chars().flat_map(|c| (c as u32).to_le_bytes()).collect();
        let be32: Vec<u8> = text.chars().flat_map(|c| (c as u32).to_be_bytes()).collect();
        assert_eq!(decode(&[&[0xFF, 0xFE, 0, 0][..], &le32].concat()), text);
        assert_eq!(decode(&[&[0, 0, 0xFE, 0xFF][..], &be32].concat()), text);
        assert_eq!(decode(b"R\xffms"), "R\u{FFFD}ms");
        assert_eq!(decode(&[0xFF, 0xFE, b'a', 0, b'b']), "a\u{FFFD}");
        assert_eq!(decode(b"\xFF\xFE\x00\xD8x\0"), "\u{FFFD}x");
        assert_eq!(decode(b""), "");
    }
}
