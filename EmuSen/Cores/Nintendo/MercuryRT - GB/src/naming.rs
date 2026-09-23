//! MarsRT's naming rule, unchanged: each Rust field is the snake_case of the C# name beside it. See Mars_Native.md §5.1.

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

    const SOURCES: [(&str, &str); 8] = [
        ("apu/channels.rs", include_str!("apu/channels.rs")),
        ("apu/mod.rs", include_str!("apu/mod.rs")),
        ("cpu/mod.rs", include_str!("cpu/mod.rs")),
        ("machine.rs", include_str!("machine.rs")),
        ("memory/bus.rs", include_str!("memory/bus.rs")),
        ("memory/cartridge.rs", include_str!("memory/cartridge.rs")),
        ("memory/mappers.rs", include_str!("memory/mappers.rs")),
        ("ppu/mod.rs", include_str!("ppu/mod.rs")),
    ];

    fn field_after(code: &str, prefix: &str) -> Option<String> {
        let at = code.find(prefix)? + prefix.len();
        Some(code[at..].chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '_').collect())
    }

    #[test]
    fn the_rule_reads_the_csharp_names_as_the_modules_spell_them() {
        assert_eq!(snake("<LastInstructionPC>k__BackingField"), "last_instruction_pc");
        assert_eq!(snake("HdmaIsHBlankDriven"), "hdma_is_h_blank_driven");
        assert_eq!(snake("_oamDmaCyclesLeft"), "oam_dma_cycles_left");
        assert_eq!(snake("Obp0"), "obp0");
        assert_eq!(snake("PC"), "pc");
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
                    if snake(label) != field {
                        wrong.push(format!("{file}:{} writes self.{field} as \"{label}\"", n + 1));
                    }
                } else if let Some((code, comment)) = line.split_once("; // ") {
                    if !code.contains("r.") && !code.contains("(r)") && !code.contains("(&mut r") {
                        continue;
                    }
                    let Some(field) = field_after(code, "self.") else { continue };
                    reads += 1;
                    if snake(comment) != field {
                        wrong.push(format!("{file}:{} reads self.{field} under \"// {comment}\"", n + 1));
                    }
                }
            }
        }
        assert!(wrong.is_empty(), "{}", wrong.join("\n"));
        assert!(writes > 130 && reads > 130, "only {writes} writes and {reads} reads were checked");
    }
}
