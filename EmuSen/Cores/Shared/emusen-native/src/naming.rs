//! The rule a core's field lists rest on: each Rust field is the snake_case of the C# name written beside it.

/// The C# name as the Rust field's: the backing field's property, no leading underscore, snake_case.
pub fn snake(cs: &str) -> String {
    let name = cs.strip_prefix('<').and_then(|s| s.split_once(">k__BackingField")).map_or(cs, |(n, _)| n);
    let chars: Vec<char> = name.trim_start_matches('_').chars().collect();
    let mut out = String::new();
    for (i, &c) in chars.iter().enumerate() {
        if !c.is_ascii_uppercase() {
            out.push(c);
            continue;
        }
        let next_lower = chars.get(i + 1).is_some_and(|n| n.is_ascii_lowercase());
        if i > 0 && (chars[i - 1].is_ascii_lowercase() || chars[i - 1].is_ascii_digit() || (chars[i - 1].is_ascii_uppercase() && next_lower)) {
            out.push('_');
        }
        out.push(c.to_ascii_lowercase());
    }
    out
}

/// What a core's scan of its own sources found.
#[derive(Debug, Default)]
pub struct Labels {
    pub writes: usize,
    pub reads: usize,
    pub wrong: Vec<String>,
}

/// The field after `prefix` in `code`, a raw identifier's `r#` dropped.
fn field_after(code: &str, prefix: &str) -> Option<String> {
    let at = code.find(prefix)? + prefix.len();
    let rest = code[at..].strip_prefix("r#").unwrap_or(&code[at..]);
    Some(rest.chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '_').collect())
}

/// Every `w.<type>("Label", self.field)` and every `self.field = r.<type>()?; // Label` in `sources`, checked against the rule.
///
/// `renamed` lists the pairs a core names differently on purpose; `owners` are the prefixes a read's field follows, first match wins.
pub fn check(sources: &[(&str, &str)], renamed: &[(&str, &str)], owners: &[&str]) -> Labels {
    let agrees = |cs: &str, rust: &str| snake(cs) == rust || renamed.contains(&(cs, rust));
    let mut found = Labels::default();
    for (file, source) in sources {
        for (n, line) in source.lines().enumerate() {
            let line = line.trim();
            if line.starts_with("w.") && line.contains("(\"") {
                let rest = &line[line.find("(\"").unwrap() + 2..];
                let Some((label, arg)) = rest.split_once("\", ") else { continue };
                let arg = arg.trim_start_matches('&');
                let Some(field) = arg.strip_prefix("self.").map(|a| a.trim_start_matches("r#").trim_end_matches(");").trim_end_matches("[..]")) else { continue };
                if !field.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                    continue;
                }
                found.writes += 1;
                if !agrees(label, field) {
                    found.wrong.push(format!("{file}:{} writes self.{field} as \"{label}\"", n + 1));
                }
            } else if let Some((code, comment)) = line.split_once("; // ") {
                if !code.contains("r.") && !code.contains("(r)") && !code.contains("(&mut r") {
                    continue;
                }
                let Some(field) = owners.iter().find_map(|o| field_after(code, o)) else { continue };
                found.reads += 1;
                if !agrees(comment, &field) {
                    found.wrong.push(format!("{file}:{} reads self.{field} under \"// {comment}\"", n + 1));
                }
            }
        }
    }
    found
}
