//! What every EmuSen command module says and reads alike, as the C# `DebugCommandHelpers` did: the words for no
//! session, how an address is read, and how one is described with its label. See EmuSen_DianaSh.md §13.11.

use crate::{abi::status, Record, Refusal, Session};

/// `RequireTarget`'s words, which the service gives for no session and a module gives for no service.
pub const NO_SESSION: &str = "No ROM loaded - this command needs an active debug target.";

/// The refusal a module answers with when the host provides no session service at all.
pub fn no_session() -> Refusal {
    Refusal::new(status::NO_SESSION, NO_SESSION)
}

/// `ParseHex`: trimmed, an optional `0x` or `$`, then hexadecimal digits read as 32 bits, so that `ffffffff` is −1.
pub fn parse_hex(text: &str) -> Result<i32, String> {
    let s = text.trim();
    let digits = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")).or_else(|| s.strip_prefix('$')).unwrap_or(s);
    u32::from_str_radix(digits, 16).map(|v| v as i32).map_err(|e| format!("'{text}' is not a hexadecimal number: {e}."))
}

/// `ParseRange`: `<start>-<end>`, or a lone address as a one-byte range; reversed ends are put in order.
pub fn parse_range(text: &str) -> Result<(i32, i32), String> {
    let s = text.trim();
    let split = s.char_indices().skip(1).find(|&(_, c)| c == '-').map(|(i, _)| i);
    let Some(split) = split else {
        let only = parse_hex(s)?;
        return Ok((only, only));
    };
    let start = parse_hex(&s[..split])?;
    let end = parse_hex(&s[split + 1..])?;
    Ok(if end < start { (end, start) } else { (start, end) })
}

/// A 32-bit address in hexadecimal, at least `digits` wide, as .NET's `X` format writes an `int`.
pub fn hex(value: i32, digits: usize) -> String {
    format!("{value:0digits$X}")
}

/// The label at an address, with its comment: none where the session keeps no labels or has none there.
pub fn label_at(session: &Session, address: i32) -> Option<(String, Option<String>)> {
    session.label_at(address as u32).ok().map(|v| (v.text("name"), v.optional_text("comment")))
}

/// `LabelRegistry.Describe`, or the plain address where the session keeps no labels: `$00A3B2 <NmiHandler>`.
pub fn describe(session: &Session, address: i32) -> String {
    match label_at(session, address) {
        Some((name, _)) => format!("${} <{name}>", hex(address, 6)),
        None => format!("${}", hex(address, 6)),
    }
}

/// `TryResolveFrom(Logs/<CoreName>, file)`: a bare name from the core's log folder, a rooted one as the shell takes
/// it, through the call's `resolve`; the refusal in `<command>: '<file>' is outside the project sandbox (<root>)`.
pub fn in_logs(resolve: impl Fn(&str) -> Result<String, String>, core: &str, file: &str, command: &str) -> Result<String, String> {
    // A drive's path on Windows is taken as it is, as Path.GetFullPath(file, base) takes it, and so refused above the root.
    let drive = cfg!(windows) && file.len() >= 2 && file.as_bytes()[1] == b':' && file.as_bytes()[0].is_ascii_alphabetic();
    let shell_path = if drive || file.starts_with('/') || file.starts_with('\\') { file.to_string() } else { format!("/Logs/{core}/{file}") };
    resolve(&shell_path).map_err(|_| format!("{command}: '{file}' is outside the project sandbox ({})", resolve("/").unwrap_or_default()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addresses_read_as_parse_hex_reads_them() {
        assert_eq!(parse_hex(" $8000 "), Ok(0x8000));
        assert_eq!(parse_hex("0X10"), Ok(16));
        assert_eq!(parse_hex("ffffffff"), Ok(-1));
        assert_eq!(parse_hex("zz"), Err("'zz' is not a hexadecimal number: invalid digit found in string.".to_string()));
        assert_eq!(parse_range("8010-8000"), Ok((0x8000, 0x8010)));
        assert_eq!(parse_range("$10"), Ok((16, 16)));
        assert_eq!(parse_range("-1"), Err("'-1' is not a hexadecimal number: invalid digit found in string.".to_string()));
        assert_eq!(hex(-5, 6), "FFFFFFFB");
        assert_eq!(hex(0x8000, 6), "008000");
    }
}
