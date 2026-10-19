//! Named addresses, the C# `LabelRegistry` (`man label`): one name for an address and one address for a name, names
//! compared without regard to case.

use emusen_dotnet::text::{fold, trim, trim_start};
use std::collections::{BTreeMap, HashMap};

#[derive(Default)]
pub struct LabelRegistry {
    by_address: BTreeMap<i32, String>,
    by_name: HashMap<String, i32>,
    comments: HashMap<i32, String>,
}

impl LabelRegistry {
    pub fn count(&self) -> usize {
        self.by_address.len()
    }

    pub fn add(&mut self, address: i32, name: &str, comment: Option<&str>) {
        if let Some(existing) = self.by_address.get(&address) {
            self.by_name.remove(&fold(existing));
        }
        if let Some(previous) = self.by_name.get(&fold(name)).copied() {
            self.by_address.remove(&previous);
            self.comments.remove(&previous);
        }
        self.by_address.insert(address, name.to_string());
        self.by_name.insert(fold(name), address);
        if let Some(c) = comment.filter(|c| !c.is_empty()) {
            self.comments.insert(address, c.to_string());
        }
    }

    pub fn remove_by_name(&mut self, name: &str) -> bool {
        let Some(address) = self.by_name.remove(&fold(name)) else { return false };
        self.by_address.remove(&address);
        self.comments.remove(&address);
        true
    }

    pub fn remove_by_address(&mut self, address: i32) -> bool {
        match self.by_address.get(&address).cloned() {
            Some(name) => self.remove_by_name(&name),
            None => false,
        }
    }

    pub fn clear(&mut self) {
        self.by_address.clear();
        self.by_name.clear();
        self.comments.clear();
    }

    pub fn name_at(&self, address: i32) -> Option<&str> {
        self.by_address.get(&address).map(String::as_str)
    }

    pub fn address_of(&self, name: &str) -> Option<i32> {
        if self.by_name.is_empty() {
            return None;
        }
        self.by_name.get(&fold(name)).copied()
    }

    pub fn comment_at(&self, address: i32) -> Option<&str> {
        self.comments.get(&address).map(String::as_str)
    }

    /// `Describe`: "$00A3B2 <NmiHandler>" when known, plain "$00A3B2" otherwise.
    pub fn describe(&self, address: i32, digits: usize) -> String {
        match self.name_at(address) {
            Some(name) => format!("${address:0digits$X} <{name}>"),
            None => format!("${address:0digits$X}"),
        }
    }

    /// Every label in address order, with its comment.
    pub fn all(&self) -> impl Iterator<Item = (i32, &str, Option<&str>)> {
        self.by_address.iter().map(|(&a, n)| (a, n.as_str(), self.comment_at(a)))
    }

    /// `LoadFromLines`: one "ADDRESS NAME [comment...]" a line; how many were added and each line's refusal.
    pub fn load_from_lines<'a>(&mut self, lines: impl IntoIterator<Item = &'a str>) -> (usize, Vec<String>) {
        let mut added = 0;
        let mut errors = Vec::new();
        for (index, raw) in lines.into_iter().enumerate() {
            let number = index + 1;
            let line = trim(raw);
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let parts = split3(line);
            if parts.len() < 2 {
                errors.push(format!("line {number}: expected '<address> <name>', got '{line}'"));
                continue;
            }
            let mut text = parts[0].trim_start_matches('$');
            if text.len() >= 2 && text[..2].eq_ignore_ascii_case("0x") {
                text = &text[2..];
            }
            let Some(address) = hex_number(text) else {
                errors.push(format!("line {number}: '{}' is not a hex address", parts[0]));
                continue;
            };
            self.add(address, parts[1], parts.get(2).copied());
            added += 1;
        }
        (added, errors)
    }

    /// `ToLines`: each label as `LoadFromLines` reads it back.
    pub fn to_lines(&self) -> Vec<String> {
        self.all()
            .map(|(a, n, c)| match c {
                None => format!("{a:06X} {n}"),
                Some(c) => format!("{a:06X} {n} {c}"),
            })
            .collect()
    }
}

/// `line.Split(null, 3, RemoveEmptyEntries)`: at most three parts split at white space, the third the rest of the
/// line with its leading white space dropped.
fn split3(line: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut rest = trim_start(line);
    while !rest.is_empty() {
        if parts.len() == 2 {
            parts.push(rest);
            break;
        }
        let end = rest.find(emusen_dotnet::text::is_white_space).unwrap_or(rest.len());
        parts.push(&rest[..end]);
        rest = trim_start(&rest[end..]);
    }
    parts
}

/// `int.TryParse(text, NumberStyles.HexNumber)`: up to eight hex digits, white space around them allowed, the bits as
/// given.
fn hex_number(text: &str) -> Option<i32> {
    let digits = trim(text);
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let significant = digits.trim_start_matches('0');
    if significant.len() > 8 {
        return None;
    }
    Some(u32::from_str_radix(if significant.is_empty() { "0" } else { significant }, 16).ok()? as i32)
}
