//! What every EmuSen command module says and reads alike, as the C# `DebugCommandHelpers` did: the words for no
//! session, how an address is read, and how one is described with its label. See EmuSen_DianaSh.md §13.11.

use crate::{abi::status, space::Endian, Record, Refusal, Session};

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

/// Shared formatting and numeric flags across debug, trace, and cheat commands.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CommonFlags {
    pub width: Option<i32>,
    pub endian: Option<Endian>,
    pub signed: Option<bool>,
    pub float: bool,
    pub decimal: Option<bool>,
    pub aligned: Option<bool>,
    pub json: bool,
}

fn try_common_flag(args: &[String], i: usize, flags: &mut CommonFlags) -> Result<Option<usize>, String> {
    let arg = &args[i];
    match arg.as_str() {
        "-w" | "--width" => {
            let next = args.get(i + 1).ok_or_else(|| format!("{arg} requires a width (1, 2, or 4)."))?;
            flags.width = Some(parse_width(next)?);
            Ok(Some(i + 2))
        }
        _ if arg.starts_with("--width=") => {
            flags.width = Some(parse_width(&arg["--width=".len()..])?);
            Ok(Some(i + 1))
        }
        "--le" => {
            flags.endian = Some(Endian::Little);
            Ok(Some(i + 1))
        }
        "--be" => {
            flags.endian = Some(Endian::Big);
            Ok(Some(i + 1))
        }
        "--signed" => {
            flags.signed = Some(true);
            Ok(Some(i + 1))
        }
        "--unsigned" => {
            flags.signed = Some(false);
            Ok(Some(i + 1))
        }
        "--float" => {
            flags.float = true;
            Ok(Some(i + 1))
        }
        "--dec" => {
            flags.decimal = Some(true);
            Ok(Some(i + 1))
        }
        "--hex" => {
            flags.decimal = Some(false);
            Ok(Some(i + 1))
        }
        "--aligned" => {
            flags.aligned = Some(true);
            Ok(Some(i + 1))
        }
        "--unaligned" => {
            flags.aligned = Some(false);
            Ok(Some(i + 1))
        }
        "--json" => {
            flags.json = true;
            Ok(Some(i + 1))
        }
        _ => Ok(None),
    }
}

fn validate_common_flags(flags: &CommonFlags) -> Result<(), String> {
    if flags.float && flags.width.is_some_and(|w| w != 4) {
        return Err("--float requires width 4 (32-bit IEEE-754 float).".to_string());
    }
    Ok(())
}

/// Extracts common value/output flags from `args`, leaving positional tokens and caller-specific flags in order.
pub fn split_common_flags(args: &[String]) -> Result<(Vec<String>, CommonFlags), String> {
    let mut flags = CommonFlags::default();
    let mut rest = Vec::with_capacity(args.len());
    let mut i = 0;
    while i < args.len() {
        if let Some(next) = try_common_flag(args, i, &mut flags)? {
            i = next;
        } else {
            rest.push(args[i].clone());
            i += 1;
        }
    }
    validate_common_flags(&flags)?;
    Ok((rest, flags))
}

/// Parses common flags (plus any `extra` flag tokens) before trailing prose starts once `required_pos` tokens exist.
pub fn split_flags_before_tail(args: &[String], required_pos: usize, extra: &[&str]) -> Result<(Vec<String>, Vec<String>, Vec<String>, CommonFlags), String> {
    let mut flags = CommonFlags::default();
    let mut pos = Vec::new();
    let mut found_extra = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if pos.len() >= required_pos && arg == "--" {
            let tail = args[i + 1..].to_vec();
            validate_common_flags(&flags)?;
            return Ok((pos, found_extra, tail, flags));
        }
        if let Some(next) = try_common_flag(args, i, &mut flags)? {
            i = next;
        } else if extra.contains(&arg.as_str()) {
            found_extra.push(arg.clone());
            i += 1;
        } else if pos.len() < required_pos {
            pos.push(arg.clone());
            i += 1;
        } else {
            let tail = args[i..].to_vec();
            validate_common_flags(&flags)?;
            return Ok((pos, found_extra, tail, flags));
        }
    }
    validate_common_flags(&flags)?;
    Ok((pos, found_extra, Vec::new(), flags))
}

fn parse_width(text: &str) -> Result<i32, String> {
    match text.trim().parse::<i32>() {
        Ok(w @ (1 | 2 | 4)) => Ok(w),
        _ => Err(format!("width must be 1, 2, or 4, got '{text}'.")),
    }
}

/// Parses a signed or unsigned decimal integer.
pub fn parse_dec(text: &str) -> Result<i64, String> {
    text.trim().parse::<i64>().map_err(|e| format!("'{text}' is not a decimal number: {e}."))
}

/// Parses a numeric value according to `width`, `decimal`, and `float`, returning raw `width`-byte bits.
pub fn parse_value_bits(text: &str, width: i32, decimal: bool, float: bool) -> Result<u32, String> {
    if float {
        if width != 4 {
            return Err("float values require width 4.".to_string());
        }
        let v = text.trim().parse::<f32>().map_err(|e| format!("'{text}' is not a float: {e}."))?;
        if v.is_nan() {
            return Err(format!("'{text}' is NaN."));
        }
        return Ok(v.to_bits());
    }
    let mask = match width {
        1 => 0xFFu32,
        2 => 0xFFFFu32,
        4 => 0xFFFF_FFFFu32,
        _ => return Err("width must be 1, 2, or 4.".to_string()),
    };
    if decimal {
        let v = parse_dec(text)?;
        let (min, max) = match width {
            1 => (-128i64, 255i64),
            2 => (-32768i64, 65535i64),
            _ => (i64::from(i32::MIN), i64::from(u32::MAX)),
        };
        if !(min..=max).contains(&v) {
            return Err(format!("'{text}' is out of range for {width}-byte value ({min}..={max})."));
        }
        return Ok((v as u32) & mask);
    }
    Ok((parse_hex(text)? as u32) & mask)
}

/// Encodes `value` into `width` bytes (1, 2, or 4) in the selected byte order.
pub fn encode_bytes(value: u32, width: i32, big_endian: bool) -> Vec<u8> {
    let mut out = Vec::with_capacity(width as usize);
    for i in 0..width {
        let shift = if big_endian { 8 * (width - 1 - i) } else { 8 * i };
        out.push(((value >> shift) & 0xFF) as u8);
    }
    out
}

/// Hex digit width appropriate for addresses in a space of `size` bytes.
pub fn hex_digits(space_size: i32) -> usize {
    if space_size > 0x100_0000 { 8 } else if space_size > 0x1_0000 { 6 } else { 4 }
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

    #[test]
    fn common_flags_and_value_encoding_round_trip() {
        let args: Vec<String> = ["WRAM", "100", "-50", "-w", "2", "--be", "--dec", "--json"].into_iter().map(str::to_string).collect();
        let (rest, flags) = split_common_flags(&args).unwrap();
        assert_eq!(rest, vec!["WRAM", "100", "-50"]);
        assert_eq!(flags.width, Some(2));
        assert_eq!(flags.endian, Some(Endian::Big));
        assert_eq!(flags.decimal, Some(true));
        assert!(flags.json);
        let bits = parse_value_bits("-50", 2, true, false).unwrap();
        assert_eq!(encode_bytes(bits, 2, true), vec![0xFF, 0xCE]);
        let fbits = parse_value_bits("1.5", 4, true, true).unwrap();
        assert_eq!(encode_bytes(fbits, 4, false), 1.5f32.to_le_bytes());
    }
}
