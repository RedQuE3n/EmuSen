//! .NET's number parsing, as `int.TryParse`, `long.TryParse` and `double.TryParse` read text with the invariant
//! culture's symbols, and as `Convert.ToInt32(text, 16)` reads hexadecimal. See EmuSen_DianaSh.md §13.7 and §13.11.

/// `NumberStyles.Integer`'s white space: tab to carriage return, and the space.
fn number_white(c: char) -> bool {
    matches!(c, '\u{9}'..='\u{D}' | ' ')
}

/// `long.TryParse(text)`: white space around, one sign, ASCII digits, no overflow, and NULs at the very end.
pub fn parse_i64(text: &str) -> Option<i64> {
    let body = text.trim_end_matches('\0').trim_matches(number_white);
    let (negative, digits) = match body.chars().next() {
        Some('-') => (true, &body[1..]),
        Some('+') => (false, &body[1..]),
        _ => (false, body),
    };
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let mut value: i128 = 0;
    for b in digits.bytes() {
        value = value * 10 + i128::from(b - b'0');
        if value > i128::from(i64::MAX) + 1 {
            return None;
        }
    }
    i64::try_from(if negative { -value } else { value }).ok()
}

/// Whether text has an integer's form, white space, a sign and ASCII digits, whatever its size.
fn integer_form(text: &str) -> bool {
    let body = text.trim_end_matches('\0').trim_matches(number_white);
    let digits = body.strip_prefix(['-', '+']).unwrap_or(body);
    !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())
}

/// `int.Parse(text)`, with .NET's words for a refusal.
pub fn int_parse(text: &str) -> Result<i32, String> {
    parse_i32(text).ok_or_else(|| {
        if integer_form(text) { "Value was either too large or too small for an Int32.".to_string() } else { format!("The input string '{text}' was not in a correct format.") }
    })
}

/// `long.Parse(text)`, with .NET's words for a refusal.
pub fn long_parse(text: &str) -> Result<i64, String> {
    parse_i64(text).ok_or_else(|| {
        if integer_form(text) { "Value was either too large or too small for an Int64.".to_string() } else { format!("The input string '{text}' was not in a correct format.") }
    })
}

/// `int.TryParse(text)`.
pub fn parse_i32(text: &str) -> Option<i32> {
    parse_i64(text).and_then(|v| i32::try_from(v).ok())
}

/// `double.TryParse(text)` as .NET reads it under the en-US culture, measured 2026-10-15: `NumberStyles.Float |
/// AllowThousands`, white space around, a sign, digits with `,` between them (never before the first), a fraction, an
/// exponent, `∞` for infinity and `NaN` in any case. The C# read the machine's culture; this reads en-US's on every
/// machine, which is what the C# read where it was recorded (EmuSen_DianaSh.md §13.8).
pub fn parse_f64(text: &str) -> Option<f64> {
    let body = text.trim_matches(number_white);
    let (negative, rest) = match body.chars().next() {
        Some('-') => (true, &body[1..]),
        Some('+') => (false, &body[1..]),
        _ => (false, body),
    };
    if rest == "\u{221E}" {
        return Some(if negative { f64::NEG_INFINITY } else { f64::INFINITY });
    }
    if rest.eq_ignore_ascii_case("NaN") {
        return Some(f64::NAN);
    }
    let mut digits = String::new();
    let mut chars = rest.chars().peekable();
    let mut seen_digit = false;
    while let Some(&c) = chars.peek() {
        match c {
            '0'..='9' => {
                digits.push(c);
                seen_digit = true;
            }
            ',' if seen_digit => {}
            _ => break,
        }
        chars.next();
    }
    if chars.peek() == Some(&'.') {
        chars.next();
        digits.push('.');
        while let Some(&c) = chars.peek() {
            if !c.is_ascii_digit() {
                break;
            }
            digits.push(c);
            seen_digit = true;
            chars.next();
        }
    }
    if !seen_digit {
        return None;
    }
    if matches!(chars.peek(), Some('e' | 'E')) {
        chars.next();
        digits.push('e');
        if let Some(&sign @ ('+' | '-')) = chars.peek() {
            digits.push(sign);
            chars.next();
        }
        let mut exponent = false;
        while let Some(&c) = chars.peek() {
            if !c.is_ascii_digit() {
                break;
            }
            digits.push(c);
            exponent = true;
            chars.next();
        }
        if !exponent {
            return None;
        }
    }
    if chars.next().is_some() {
        return None;
    }
    let value: f64 = digits.parse().ok()?;
    Some(if negative { -value } else { value })
}

/// `Convert.ToInt32(text, 16)`: a `+` or a `0x` (or both, in that order), then hexadecimal digits read as 32 bits
/// unsigned, so that `ffffffff` is −1; nothing else around them. The words are .NET 10's, measured 2026-10-16.
pub fn convert_to_i32_hex(text: &str) -> Result<i32, String> {
    if text.is_empty() {
        return Err("Specified argument was out of the range of valid values. (Parameter 'Index was out of range. Must be non-negative and less than the size of the collection.')".to_string());
    }
    if text.starts_with('-') {
        return Err("String cannot contain a minus sign if the base is not 10.".to_string());
    }
    let mut rest = text.strip_prefix('+').unwrap_or(text);
    if rest.len() >= 2 && rest.starts_with('0') && matches!(rest.as_bytes()[1], b'x' | b'X') {
        rest = &rest[2..];
    }
    let digits = rest.bytes().take_while(u8::is_ascii_hexdigit).count();
    if digits == 0 {
        return Err("Could not find any recognizable digits.".to_string());
    }
    let mut value: u64 = 0;
    for b in rest[..digits].bytes() {
        value = value * 16 + u64::from((b as char).to_digit(16).unwrap_or(0));
        if value > u64::from(u32::MAX) {
            return Err("Value was either too large or too small for a UInt32.".to_string());
        }
    }
    if digits < rest.len() {
        return Err("Additional non-parsable characters are at the end of the string.".to_string());
    }
    Ok(value as u32 as i32)
}

#[cfg(test)]
mod tests {

    // Measured on .NET 10.0.112 on 2026-10-17: after the number and any white space, NULs may follow and nothing else.
    #[test]
    fn trailing_nuls_end_a_number_as_dotnet_reads_it() {
        assert_eq!(parse_i64("5\0"), Some(5));
        assert_eq!(parse_i64("5 \0\0"), Some(5));
        assert_eq!(parse_i64("5\0\0 "), None);
        assert_eq!(parse_i64("\05"), None);
        assert_eq!(parse_i32("-7\0"), Some(-7));
    }
    use super::*;

    #[test]
    fn hexadecimal_reads_as_convert_reads_it() {
        assert_eq!(convert_to_i32_hex("10"), Ok(16));
        assert_eq!(convert_to_i32_hex("+0x10"), Ok(16));
        assert_eq!(convert_to_i32_hex("ffffffff"), Ok(-1));
        assert_eq!(convert_to_i32_hex("80000000"), Ok(i32::MIN));
        assert_eq!(convert_to_i32_hex("00000000ff"), Ok(255));
        assert_eq!(convert_to_i32_hex("100000000"), Err("Value was either too large or too small for a UInt32.".to_string()));
        assert_eq!(convert_to_i32_hex("-0"), Err("String cannot contain a minus sign if the base is not 10.".to_string()));
        assert_eq!(convert_to_i32_hex("g"), Err("Could not find any recognizable digits.".to_string()));
        assert_eq!(convert_to_i32_hex("0x"), Err("Could not find any recognizable digits.".to_string()));
        assert_eq!(convert_to_i32_hex(" 5"), Err("Could not find any recognizable digits.".to_string()));
        assert_eq!(convert_to_i32_hex("1g"), Err("Additional non-parsable characters are at the end of the string.".to_string()));
        assert!(convert_to_i32_hex("").unwrap_err().starts_with("Specified argument was out of the range"));
    }

    #[test]
    fn integers_parse_as_dotnet_parses_them() {
        assert_eq!(parse_i32(" 12 "), Some(12));
        assert_eq!(parse_i32("+7"), Some(7));
        assert_eq!(parse_i32("2147483648"), None);
        assert_eq!(parse_i32("-2147483648"), Some(i32::MIN));
        assert_eq!(parse_i32("1_0"), None);
        assert_eq!(parse_i32(""), None);
        assert_eq!(parse_i32("-"), None);
        assert_eq!(parse_i32("٣"), None);
        assert_eq!(parse_i64("-9223372036854775808"), Some(i64::MIN));
        assert_eq!(parse_i64("9223372036854775808"), None);
    }

    #[test]
    fn doubles_take_thousands_a_fraction_and_an_exponent() {
        assert_eq!(parse_f64("1,000.5"), Some(1000.5));
        assert_eq!(parse_f64(" -2e3 "), Some(-2000.0));
        assert_eq!(parse_f64(".5"), Some(0.5));
        assert_eq!(parse_f64("5."), Some(5.0));
        assert_eq!(parse_f64("-\u{221E}"), Some(f64::NEG_INFINITY));
        assert_eq!(parse_f64("Infinity"), None);
        assert!(parse_f64("-nan").unwrap().is_nan());
        assert_eq!(parse_f64(",5"), None);
        assert_eq!(parse_f64("1,,0"), Some(10.0));
        assert_eq!(parse_f64("1,000.5,"), None);
        assert_eq!(parse_f64("abc"), None);
        assert_eq!(parse_f64("1e"), None);
        assert_eq!(parse_f64("."), None);
        assert_eq!(parse_f64("1 2"), None);
    }
}
