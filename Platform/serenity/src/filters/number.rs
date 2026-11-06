//! Numbers written into the built-in filters' GLSL and presets as .NET writes them: a single by "R", the shortest text
//! that reads back to it, and a double by a custom pattern such as "0.0#######", which .NET first takes to 15
//! significant digits and then rounds half up at the last decimal place. See EmuSen_Serenity.md §3.13.

/// The decimal digits of a value and where its point sits: `0.0123` is digits `123` at scale -1.
struct Digits {
    digits: Vec<u8>,
    scale: i32,
}

/// Rust's scientific text of a non-negative value split into its digits and its scale.
fn split(scientific: &str) -> Digits {
    let (mantissa, exponent) = scientific.split_once('e').expect("scientific text has an exponent");
    let mut digits: Vec<u8> = mantissa.bytes().filter(u8::is_ascii_digit).map(|b| b - b'0').collect();
    while digits.last() == Some(&0) {
        digits.pop();
    }
    let scale = exponent.parse::<i32>().expect("an exponent is a number") + 1;
    if digits.is_empty() { Digits { digits, scale: 0 } } else { Digits { digits, scale } }
}

/// .NET's `double.ToString` with a custom pattern of one required decimal and up to `most` in all, such as
/// "0.0#######" for 8: the value taken to 15 significant digits, then rounded half up at the last place kept.
pub fn fixed(value: f64, most: usize) -> String {
    let negative = value.is_sign_negative();
    let mut d = split(&format!("{:.14e}", value.abs()));
    let position = d.scale + most as i32;
    if position < d.digits.len() as i32 {
        if position < 0 {
            d.digits.clear();
        } else {
            let up = d.digits[position as usize] >= 5;
            d.digits.truncate(position as usize);
            if up {
                while d.digits.last() == Some(&9) {
                    d.digits.pop();
                }
                match d.digits.last_mut() {
                    Some(last) => *last += 1,
                    None => {
                        d.digits.push(1);
                        d.scale += 1;
                    }
                }
            }
            while d.digits.last() == Some(&0) {
                d.digits.pop();
            }
        }
        if d.digits.is_empty() {
            d.scale = 0;
        }
    }
    let mut text = String::new();
    if negative {
        text.push('-');
    }
    if d.scale <= 0 {
        text.push('0');
    } else {
        for i in 0..d.scale as usize {
            text.push((b'0' + d.digits.get(i).copied().unwrap_or(0)) as char);
        }
    }
    text.push('.');
    let mut decimals = String::new();
    for _ in d.scale..0 {
        decimals.push('0');
    }
    for &digit in d.digits.iter().skip(d.scale.max(0) as usize) {
        decimals.push((b'0' + digit) as char);
    }
    let kept = decimals.trim_end_matches('0');
    text.push_str(if kept.is_empty() { "0" } else { kept });
    text
}

/// .NET's `float.ToString("R")`: the shortest digits that read back to the value, in plain notation unless the point
/// would sit more than seven places or the digits' count out, or more than three places in.
pub fn single(value: f32) -> String {
    if value.is_nan() {
        return "NaN".to_string();
    }
    if value.is_infinite() {
        return if value < 0.0 { "-∞" } else { "∞" }.to_string();
    }
    let sign = if value.is_sign_negative() { "-" } else { "" };
    if value == 0.0 {
        return format!("{sign}0");
    }
    let d = split(&format!("{:e}", value.abs()));
    let digits: String = d.digits.iter().map(|b| (b'0' + b) as char).collect();
    let most = d.digits.len().max(7) as i32;
    if d.scale > most || d.scale < -3 {
        let exponent = d.scale - 1;
        let rest = if digits.len() > 1 { format!(".{}", &digits[1..]) } else { String::new() };
        return format!("{sign}{}{rest}E{}{:02}", &digits[..1], if exponent < 0 { '-' } else { '+' }, exponent.abs());
    }
    if d.scale <= 0 {
        return format!("{sign}0.{}{digits}", "0".repeat((-d.scale) as usize));
    }
    let scale = d.scale as usize;
    if digits.len() <= scale {
        format!("{sign}{digits}{}", "0".repeat(scale - digits.len()))
    } else {
        format!("{sign}{}.{}", &digits[..scale], &digits[scale..])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_double_is_written_as_the_custom_pattern_writes_it() {
        assert_eq!(fixed(0.0, 8), "0.0");
        assert_eq!(fixed(3.0, 8), "3.0");
        assert_eq!(fixed(0.5, 8), "0.5");
        assert_eq!(fixed(1.0 / 3.0, 8), "0.33333333");
        assert_eq!(fixed(2.0 / 3.0, 8), "0.66666667");
        assert_eq!(fixed(0.999_999_999, 8), "1.0");
        assert_eq!(fixed(-0.25, 8), "-0.25");
        assert_eq!(fixed(123.456, 8), "123.456");
        assert_eq!(fixed(0.01 / 176.0, 8), "0.00005682");
        assert_eq!(fixed(1e-12, 8), "0.0");
        assert_eq!(fixed(0.5 / 255.0, 6), "0.001961");
        assert_eq!(fixed(1500.0, 6), "1500.0");
    }

    #[test]
    fn a_single_is_written_as_r_writes_it() {
        assert_eq!(single(1.0), "1");
        assert_eq!(single(0.031), "0.031");
        assert_eq!(single(-8.0 - 1e6), "-1000008");
        assert_eq!(single(0.031 + 1e6), "1000000");
        assert_eq!(single(0.5), "0.5");
        assert_eq!(single(0.0001), "0.0001");
        assert_eq!(single(0.00001), "1E-05");
        assert_eq!(single(1e7), "1E+07");
        assert_eq!(single(12345678.0), "12345678");
        assert_eq!(single(-0.0), "-0");
        assert_eq!(single(1.0 / 3.0), "0.33333334");
    }
}
