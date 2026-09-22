//! Holds the naming rule the state's field lists rest on: each Rust field is the snake_case of the C# name beside it.

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

#[cfg(test)]
mod tests {
    use super::snake;

    const SOURCES: [(&str, &str); 15] = [
        ("memory/ai.rs", include_str!("memory/ai.rs")),
        ("memory/bus.rs", include_str!("memory/bus.rs")),
        ("memory/controller.rs", include_str!("memory/controller.rs")),
        ("cpu/mod.rs", include_str!("cpu/mod.rs")),
        ("memory/dp.rs", include_str!("memory/dp.rs")),
        ("memory/isviewer.rs", include_str!("memory/isviewer.rs")),
        ("machine.rs", include_str!("machine.rs")),
        ("memory/mi.rs", include_str!("memory/mi.rs")),
        ("memory/pi.rs", include_str!("memory/pi.rs")),
        ("rdp/mod.rs", include_str!("rdp/mod.rs")),
        ("memory/save.rs", include_str!("memory/save.rs")),
        ("memory/si.rs", include_str!("memory/si.rs")),
        ("memory/sp.rs", include_str!("memory/sp.rs")),
        ("cpu/tlb.rs", include_str!("cpu/tlb.rs")),
        ("vi/mod.rs", include_str!("vi/mod.rs")),
    ];

    /// Named on purpose: `SaveChip.Type` is `kind`, `type` being a keyword; a snapshot's words are no C# field.
    const RENAMED: [(&str, &str); 2] = [("Type", "kind"), ("Words", "pending")];

    fn field_after(code: &str, prefix: &str) -> Option<String> {
        let at = code.find(prefix)? + prefix.len();
        Some(code[at..].chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '_').collect())
    }

    fn agrees(cs: &str, rust: &str) -> bool {
        snake(cs) == rust || RENAMED.contains(&(cs, rust))
    }

    #[test]
    fn the_rule_reads_the_csharp_names_as_the_modules_spell_them() {
        assert_eq!(snake("<TextureMemory>k__BackingField"), "texture_memory");
        assert_eq!(snake("_attributeDe"), "attribute_de");
        assert_eq!(snake("EntryLo0"), "entry_lo0");
        assert_eq!(snake("SH"), "sh");
        assert_eq!(snake("ShiftS"), "shift_s");
        assert_eq!(snake("IsViewer"), "is_viewer");
        assert_eq!(snake("_k0"), "k0");
    }

    /// A writer's label and the field it writes, and a reader's field and its comment, name the same C# field.
    #[test]
    fn every_label_and_comment_names_the_field_on_its_line() {
        let (mut writes, mut reads, mut wrong) = (0, 0, Vec::new());
        for (file, source) in SOURCES {
            for (n, line) in source.lines().enumerate() {
                let line = line.trim();
                if line.starts_with("w.") && line.contains("(\"") {
                    let rest = &line[line.find("(\"").unwrap() + 2..];
                    let Some((label, arg)) = rest.split_once("\", ") else { continue };
                    let arg = arg.trim_start_matches('&');
                    let Some(field) = arg.strip_prefix("self.").map(|a| a.trim_end_matches(");").trim_end_matches("[..]")) else { continue };
                    if !field.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                        continue;
                    }
                    writes += 1;
                    if !agrees(label, field) {
                        wrong.push(format!("{file}:{} writes self.{field} as \"{label}\"", n + 1));
                    }
                } else if let Some((code, comment)) = line.split_once("; // ") {
                    if !code.contains("r.") && !code.contains("(r)") && !code.contains("(&mut r") {
                        continue;
                    }
                    let Some(field) = field_after(code, "self.").or_else(|| field_after(code, "machine.")) else { continue };
                    reads += 1;
                    if !agrees(comment, &field) {
                        wrong.push(format!("{file}:{} reads self.{field} under \"// {comment}\"", n + 1));
                    }
                }
            }
        }
        assert!(wrong.is_empty(), "{}", wrong.join("\n"));
        assert!(writes > 200 && reads > 200, "only {writes} writes and {reads} reads were checked");
    }
}
