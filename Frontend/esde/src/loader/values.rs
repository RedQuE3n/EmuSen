//! THEMES.md's seven property types, and a property's text read into one as ES-DE 3.4.1 reads it
//! (EmuSen_BigPicture.md §12.2, §31.2 and §35.2).

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Component, Path, PathBuf};

use super::catalog::{PairRule, PropertySpec};

/// Two fractions of the parent, which is nearly always the screen; y points down.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pair {
    pub x: f32,
    pub y: f32,
}

impl Pair {
    pub const fn new(x: f32, y: f32) -> Self {
        Pair { x, y }
    }
}

impl fmt::Display for Pair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", number(self.x as f64), number(self.y as f64))
    }
}

/// A number as "0.######" writes it: up to six decimals, no trailing zeros.
pub fn number(value: f64) -> String {
    let text = format!("{value:.6}");
    let text = if text.contains('.') { text.trim_end_matches('0').trim_end_matches('.') } else { &text };
    text.to_owned()
}

/// An RGBA colour as THEMES.md writes it: 6 hex digits are opaque, 8 carry alpha last.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Color(pub u32);

impl Color {
    pub fn r(self) -> u8 {
        (self.0 >> 24) as u8
    }
    pub fn g(self) -> u8 {
        (self.0 >> 16) as u8
    }
    pub fn b(self) -> u8 {
        (self.0 >> 8) as u8
    }
    pub fn a(self) -> u8 {
        self.0 as u8
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:08X}", self.0)
    }
}

/// A file a property names: as written, as resolved, whether it was built from a variable, and whether it exists.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThemePath {
    pub written: String,
    pub absolute: String,
    pub from_variable: bool,
    pub exists: bool,
}

/// One typed, resolved property value.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Pair(Pair),
    Path(ThemePath),
    Bool(bool),
    Color(Color),
    UInt(u32),
    Float(f32),
    Str(String),
    /// A STRING that holds a list: imageType, slots, entries.
    List(Vec<String>),
    /// A PATH keyed by an attribute, such as customBadgeIcon badge="favorite".
    Keyed(BTreeMap<String, ThemePath>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Pair(p) => write!(f, "{p}"),
            Value::Path(p) => write!(f, "{}", p.absolute),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Color(c) => write!(f, "{c}"),
            Value::UInt(u) => write!(f, "{u}"),
            Value::Float(x) => write!(f, "{}", number(*x as f64)),
            Value::Str(s) => write!(f, "{s}"),
            Value::List(items) => write!(f, "{}", items.join(",")),
            Value::Keyed(paths) => write!(f, "{}", paths.iter().map(|(k, p)| format!("{k}={}", p.absolute)).collect::<Vec<_>>().join(",")),
        }
    }
}

/// The documented separators for name lists and value lists: commas, tabs, spaces or line breaks.
pub fn split(text: &str) -> Vec<String> {
    text.split([',', ' ', '\t', '\r', '\n']).filter(|s| !s.is_empty()).map(str::to_owned).collect()
}

/// The image and video imageType shortcut: THEMES.md's "image".
pub const IMAGE_SHORTCUT: [&str; 4] = ["miximage", "screenshot", "titlescreen", "cover"];

pub fn try_bool(text: &str) -> Option<bool> {
    match text.to_lowercase().as_str() {
        "true" | "1" => Some(true),
        "false" | "0" => Some(false),
        _ => None,
    }
}

pub fn try_float(text: &str) -> Option<f32> {
    text.parse::<f32>().ok().filter(|v| v.is_finite())
}

pub fn try_pair(text: &str) -> Option<Pair> {
    let parts: Vec<&str> = text.split([' ', '\t', '\r', '\n']).filter(|s| !s.is_empty()).collect();
    match parts.as_slice() {
        [x, y] => Some(Pair::new(try_float(x)?, try_float(y)?)),
        _ => None,
    }
}

/// ES-DE's BOOLEAN: true when the first character is t, T, y, Y or 1, never an error; and whether it was written plainly.
pub fn esde_bool(text: &str) -> (bool, bool) {
    let exact = matches!(text.to_lowercase().as_str(), "true" | "false" | "1" | "0");
    (matches!(text.chars().next(), Some('t' | 'T' | 'y' | 'Y' | '1')), exact)
}

/// ES-DE's FLOAT: the leading number after any C whitespace, hexadecimal, inf and nan included, else 0.
pub fn esde_float(text: &str) -> (f32, bool) {
    let t = text.as_bytes();
    let start = skip_c_space(t, 0);
    let (end, value, plain) = number_end(text, start);
    let exact = end > start && plain && text[end..].trim().is_empty();
    (if end == start { 0.0 } else { value as f32 }, exact)
}

/// A value ES-DE could not draw at an unbounded property, such as nan or an infinite position, is taken as 0.
pub fn finite(value: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
}

/// ES-DE's UNSIGNED_INTEGER: a C reading with the base from the prefix, kept to 32 bits, 0 when none.
pub fn esde_uint(text: &str) -> (u32, bool) {
    let t = text.as_bytes();
    let mut i = skip_c_space(t, 0);
    let mut negative = false;
    if i < t.len() && (t[i] == b'+' || t[i] == b'-') {
        negative = t[i] == b'-';
        i += 1;
    }
    let mut radix = 10u64;
    if i < t.len() && t[i] == b'0' {
        if i + 2 < t.len() && (t[i + 1] == b'x' || t[i + 1] == b'X') && t[i + 2].is_ascii_hexdigit() {
            radix = 16;
            i += 2;
        } else {
            radix = 8;
        }
    }
    let mut value: u64 = 0;
    let mut overflow = false;
    while i < t.len() {
        let Some(d) = digit(t[i]).filter(|&d| (d as u64) < radix) else { break };
        let d = d as u64;
        if value > (u64::MAX - d) / radix {
            overflow = true;
        }
        value = value.wrapping_mul(radix).wrapping_add(d);
        i += 1;
    }
    if overflow {
        value = u64::MAX;
    }
    if negative {
        value = 0u64.wrapping_sub(value);
    }
    let trimmed = text.trim();
    let exact = !trimmed.is_empty()
        && trimmed.bytes().all(|b| b.is_ascii_digit())
        && (trimmed.len() == 1 || !trimmed.starts_with('0'))
        && !overflow
        && value <= u32::MAX as u64;
    (value as u32, exact)
}

/// ES-DE's COLOR: 6 or 8 characters as written, else refused (None); then read as hexadecimal the C way.
pub fn esde_color(text: &str) -> Option<(Color, bool)> {
    let length = text.chars().count();
    if length != 6 && length != 8 {
        return None;
    }
    let t = text.as_bytes();
    let mut i = skip_c_space(t, 0);
    let mut negative = false;
    if i < t.len() && (t[i] == b'+' || t[i] == b'-') {
        negative = t[i] == b'-';
        i += 1;
    }
    if i + 2 < t.len() && t[i] == b'0' && (t[i + 1] == b'x' || t[i + 1] == b'X') && t[i + 2].is_ascii_hexdigit() {
        i += 2;
    }
    let mut value: u32 = 0;
    while i < t.len() && t[i].is_ascii_hexdigit() {
        value = value.wrapping_mul(16).wrapping_add(digit(t[i]).unwrap_or(0));
        i += 1;
    }
    if negative {
        value = 0u32.wrapping_sub(value);
    }
    let color = Color(if length == 6 { (value << 8) | 0xFF } else { value });
    Some((color, text.chars().all(|c| c.is_ascii_hexdigit())))
}

/// ES-DE's NORMALIZED_PAIR: split at the first space, each side read as a FLOAT; None when there is no space.
pub fn esde_pair(text: &str) -> Option<(Pair, bool)> {
    let space = text.find(' ')?;
    let (x, x_exact) = esde_float(&text[..space]);
    let (y, y_exact) = esde_float(&text[space + 1..]);
    Some((Pair::new(x, y), x_exact && y_exact && try_pair(text).is_some()))
}

// C's whitespace, which ES-DE's readings skip.
fn skip_c_space(t: &[u8], mut i: usize) -> usize {
    while i < t.len() && matches!(t[i], b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r') {
        i += 1;
    }
    i
}

fn digit(c: u8) -> Option<u32> {
    match c {
        b'0'..=b'9' => Some((c - b'0') as u32),
        b'a'..=b'z' => Some((c - b'a') as u32 + 10),
        b'A'..=b'Z' => Some((c - b'A') as u32 + 10),
        _ => None,
    }
}

// The end of the longest number at start, as C reads one: decimal with an exponent, hexadecimal with a binary one, inf, infinity, nan.
fn number_end(text: &str, start: usize) -> (usize, f64, bool) {
    let t = text.as_bytes();
    let mut i = start;
    let mut negative = false;
    if i < t.len() && (t[i] == b'+' || t[i] == b'-') {
        negative = t[i] == b'-';
        i += 1;
    }
    let rest = &text[i..];
    let lower = rest.chars().take(8).collect::<String>().to_ascii_lowercase();
    let infinity = if negative { f64::NEG_INFINITY } else { f64::INFINITY };
    if lower.starts_with("infinity") {
        return (i + 8, infinity, false);
    }
    if lower.starts_with("inf") {
        return (i + 3, infinity, false);
    }
    if lower.starts_with("nan") {
        let r = rest.as_bytes();
        if r.len() > 3 && r[3] == b'(' {
            if let Some(close) = rest[4..].find(')').map(|c| c + 4) {
                if rest[4..close].bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_') {
                    return (i + close + 1, f64::NAN, false);
                }
            }
        }
        return (i + 3, f64::NAN, false);
    }
    let r = rest.as_bytes();
    if r.len() > 2
        && r[0] == b'0'
        && (r[1] == b'x' || r[1] == b'X')
        && (r[2].is_ascii_hexdigit() || (r[2] == b'.' && r.len() > 3 && r[3].is_ascii_hexdigit()))
    {
        let (end, value) = hex_end(t, i + 2, negative);
        return (end, value, false);
    }
    let mut digits = 0;
    let mut j = i;
    while j < t.len() && t[j].is_ascii_digit() {
        j += 1;
        digits += 1;
    }
    if j < t.len() && t[j] == b'.' {
        j += 1;
        while j < t.len() && t[j].is_ascii_digit() {
            j += 1;
            digits += 1;
        }
    }
    if digits == 0 {
        return (start, 0.0, false);
    }
    if j < t.len() && (t[j] == b'e' || t[j] == b'E') {
        let mut k = j + 1;
        if k < t.len() && (t[k] == b'+' || t[k] == b'-') {
            k += 1;
        }
        let mut m = k;
        while m < t.len() && t[m].is_ascii_digit() {
            m += 1;
        }
        if m > k {
            j = m;
        }
    }
    (j, text[start..j].parse::<f64>().unwrap_or(0.0), true)
}

// Hexadecimal digits, an optional point and more, then an optional p exponent in powers of two.
fn hex_end(t: &[u8], mut i: usize, negative: bool) -> (usize, f64) {
    let mut mantissa = 0f64;
    let mut scale: i64 = 0;
    while i < t.len() && t[i].is_ascii_hexdigit() {
        mantissa = mantissa * 16.0 + digit(t[i]).unwrap_or(0) as f64;
        i += 1;
    }
    if i < t.len() && t[i] == b'.' {
        i += 1;
        while i < t.len() && t[i].is_ascii_hexdigit() {
            mantissa = mantissa * 16.0 + digit(t[i]).unwrap_or(0) as f64;
            scale -= 4;
            i += 1;
        }
    }
    if i < t.len() && (t[i] == b'p' || t[i] == b'P') {
        let mut k = i + 1;
        let mut minus = false;
        if k < t.len() && (t[k] == b'+' || t[k] == b'-') {
            minus = t[k] == b'-';
            k += 1;
        }
        let (mut m, mut exponent) = (k, 0i64);
        while m < t.len() && t[m].is_ascii_digit() {
            exponent = (exponent * 10 + (t[m] - b'0') as i64).min(100_000);
            m += 1;
        }
        if m > k {
            scale += if minus { -exponent } else { exponent };
            i = m;
        }
    }
    let value = mantissa * 2f64.powf(scale as f64);
    (i, if negative { -value } else { value })
}

/// `./` is the file's folder, `~` is home, backslashes are separators, nothing is trimmed, and any other relative path is
/// the working directory's (EmuSen_BigPicture.md §35.2).
pub fn resolve_path(written: &str, file: &str) -> String {
    let text = written.replace('\\', "/");
    let folder = Path::new(file).parent().map(|p| p.to_string_lossy().into_owned()).unwrap_or_else(|| ".".into());
    if text.is_empty() {
        return full_path(&folder);
    }
    if let Some(rest) = text.strip_prefix("./") {
        return full_path(&format!("{folder}/{rest}"));
    }
    if let Some(rest) = text.strip_prefix('~') {
        return full_path(&format!("{}{rest}", std::env::var("HOME").unwrap_or_default()));
    }
    full_path(&text)
}

/// An absolute path with `.` and `..` folded away, as .NET's Path.GetFullPath gives it; no link is followed.
pub fn full_path(path: &str) -> String {
    let base: PathBuf =
        if path.starts_with('/') { PathBuf::from("/") } else { std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")) };
    let mut parts: Vec<String> =
        base.components().filter(|&c| matches!(c, Component::Normal(_))).map(|c| c.as_os_str().to_string_lossy().into_owned()).collect();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            other => parts.push(other.to_owned()),
        }
    }
    let mut out = format!("/{}", parts.join("/"));
    if path.ends_with('/') && out.len() > 1 {
        out.push('/');
    }
    out
}

pub fn clamp_uint(value: u32, min: Option<f32>, max: Option<f32>) -> u32 {
    if let Some(lo) = min.filter(|&lo| (value as f32) < lo) {
        return lo as u32;
    }
    if let Some(hi) = max.filter(|&hi| (value as f32) > hi) {
        return hi as u32;
    }
    value
}

pub fn clamp(mut value: f32, min: Option<f32>, max: Option<f32>) -> f32 {
    if let Some(lo) = min.filter(|&lo| value < lo) {
        value = lo;
    }
    if let Some(hi) = max.filter(|&hi| value > hi) {
        value = hi;
    }
    value
}

/// A pair clamped by its range, with the per-axis rules THEMES.md gives particular pairs.
pub fn clamp_pair(pair: Pair, spec: &PropertySpec) -> Pair {
    let axis = |v: f32, max: Option<f32>| match spec.rule {
        PairRule::ZeroAxisAuto | PairRule::OneAxisOnly if v == 0.0 => v,
        PairRule::MinusOneMatchesOther if v == -1.0 => v,
        _ => clamp(v, spec.min, max),
    };
    if spec.rule == PairRule::ZeroAxisAuto && pair.x == 0.0 && pair.y == 0.0 {
        if let Some(min) = spec.min.filter(|&m| m > 0.0) {
            return Pair::new(min, min);
        }
    }
    Pair::new(axis(pair.x, spec.max), axis(pair.y, spec.max_y.or(spec.max)))
}
