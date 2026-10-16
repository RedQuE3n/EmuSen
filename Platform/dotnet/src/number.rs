//! .NET's number parsing, as `int.TryParse`, `long.TryParse` and `double.TryParse` read text with the invariant
//! culture's symbols. See EmuSen_DianaSh.md §13.7.

/// `NumberStyles.Integer`'s white space: tab to carriage return, and the space.
fn number_white(c: char) -> bool {
    matches!(c, '\u{9}'..='\u{D}' | ' ')
}

/// `long.TryParse(text)`: white space around, one sign, ASCII digits, no overflow.
pub fn parse_i64(text: &str) -> Option<i64> {
    let body = text.trim_matches(number_white);
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

#[cfg(test)]
mod tests {
    use super::*;

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
