//! The one place a ScreenScraper address or message is made safe to show or write, `ScrapeRedactor` ported: every
//! credential parameter's value blanked, and every credential's own value blanked wherever it turns up once it has been
//! read. The registry is the process's, as the C#'s is, since the words of any handle may carry any credential. See
//! EmuSen_BigPicture.md §5.7 and §17.7.

use std::sync::Mutex;

pub const BLANK: &str = "***";

static LITERALS: Mutex<Vec<String>> = Mutex::new(Vec::new());

const NAMES: [&str; 4] = ["devid", "devpassword", "ssid", "sspassword"];

/// A credential's own value, blanked wherever it turns up; shorter than four characters it would blank ordinary words.
pub fn register(secret: &str) {
    if secret.chars().count() < 4 {
        return;
    }
    let mut literals = LITERALS.lock().unwrap_or_else(|p| p.into_inner());
    if !literals.iter().any(|l| l == secret) {
        literals.push(secret.to_string());
    }
}

/// What ends a parameter's value: `&`, white space, a quote, `<`, `>`, `\`, a backtick or `)`.
fn ends_value(c: char) -> bool {
    c.is_whitespace() || matches!(c, '&' | '"' | '\'' | '<' | '>' | '\\' | '`' | ')')
}

/// The regex `(?i)(?<![A-Za-z0-9_])(devid|devpassword|ssid|sspassword)=([^&\s"'<>\\`)]*)`, each match's value replaced.
fn parameters(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    let bytes = text.as_bytes();
    'scan: while i < text.len() {
        let preceded = i == 0 || !(bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_');
        if preceded {
            for name in NAMES {
                let end = i + name.len();
                if end < text.len() && bytes[end] == b'=' && text.get(i..end).is_some_and(|w| w.eq_ignore_ascii_case(name)) {
                    out.push_str(&text[i..end]);
                    out.push('=');
                    out.push_str(BLANK);
                    let value = &text[end + 1..];
                    let length = value.find(ends_value).unwrap_or(value.len());
                    i = end + 1 + length;
                    continue 'scan;
                }
            }
        }
        let c = text[i..].chars().next().expect("i is at a character");
        out.push(c);
        i += c.len_utf8();
    }
    out
}

pub fn redact(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    let mut safe = parameters(text);
    for secret in LITERALS.lock().unwrap_or_else(|p| p.into_inner()).iter() {
        safe = safe.replace(secret.as_str(), BLANK);
    }
    safe
}
