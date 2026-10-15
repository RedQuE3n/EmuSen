//! The boundary's source check (Nephrite_Attachments.md §5.3): what privacy inside one crate cannot hold, read from the
//! source. An attachment's production code names no path of the crate but `crate::interface` and its own tree; the
//! Genesis's and the interface's name no attachment and nothing of the assembly. Tests are the assembly's view of a
//! part and are not read.

const SRC: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src");

/// The assembly's modules, which only the assembly names.
const ASSEMBLY: [&str; 4] = ["machine", "media", "state", "v1"];
const ATTACHMENTS: [&str; 2] = ["mcd", "s32x"];

/// Each production line of a file: its test module, from the `#[cfg(test)]` before `mod … {` on, is left out.
fn production_lines(text: &str) -> Vec<(usize, &str)> {
    let lines: Vec<&str> = text.lines().collect();
    let end = (0..lines.len())
        .find(|&i| lines[i].trim() == "#[cfg(test)]" && lines.get(i + 1).is_some_and(|l| l.trim_start().starts_with("mod ") && l.trim_end().ends_with('{')))
        .unwrap_or(lines.len());
    lines[..end].iter().enumerate().map(|(i, l)| (i + 1, *l)).filter(|(_, l)| !l.trim_start().starts_with("//")).collect()
}

/// The production files of a tree: a file its `mod.rs` declares under `#[cfg(test)]` is a test's.
fn tree_files(tree: &str) -> Vec<(String, String)> {
    let dir = std::path::Path::new(SRC).join(tree);
    let Ok(entries) = std::fs::read_dir(&dir) else { return Vec::new() };
    let root = std::fs::read_to_string(dir.join("mod.rs")).expect("the tree's mod.rs");
    let mut files: Vec<_> = entries.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|e| e == "rs")).collect();
    files.sort();
    files
        .into_iter()
        .filter(|p| {
            let name = p.file_stem().unwrap().to_string_lossy().into_owned();
            !root.contains(&format!("#[cfg(test)]\nmod {name};")) && !root.contains(&format!("#[cfg(test)]\npub(crate) mod {name};"))
        })
        .map(|p| (format!("{tree}/{}", p.file_name().unwrap().to_string_lossy()), std::fs::read_to_string(&p).unwrap()))
        .collect()
}

/// What a line of `tree`'s production code reaches that it may not, by the rules above.
fn reach(tree: &str, file: &str, line: &str) -> Option<String> {
    if line.contains("crate::{") {
        return Some("a grouped path of the crate, which hides what it names".into());
    }
    let named = line.match_indices("crate::").map(|(i, _)| line[i + 7..].split(|c: char| !(c.is_alphanumeric() || c == '_')).next().unwrap_or(""));
    for module in named {
        let allowed = if ATTACHMENTS.contains(&tree) { module == "interface" || module == tree } else { !ATTACHMENTS.contains(&module) && !ASSEMBLY.contains(&module) };
        if !allowed {
            return Some(format!("crate::{module}"));
        }
    }
    if ATTACHMENTS.contains(&tree) && (line.contains("super::super") || (file.ends_with("/mod.rs") && line.contains("super::"))) {
        return Some("a path out of the tree through super".into());
    }
    None
}

#[test]
fn the_attachments_reach_only_the_interface() {
    let mut wrong = Vec::new();
    let mut parts: Vec<(String, String, String)> = ["genesis", "mcd", "s32x"].iter().flat_map(|t| tree_files(t).into_iter().map(|(f, text)| (t.to_string(), f, text))).collect();
    parts.push(("interface".into(), "interface.rs".into(), std::fs::read_to_string(std::path::Path::new(SRC).join("interface.rs")).unwrap()));
    assert!(parts.iter().any(|p| p.0 == "genesis") && parts.iter().any(|p| p.0 == "mcd"), "the trees are read");
    for (tree, file, text) in &parts {
        for (n, line) in production_lines(text) {
            if let Some(what) = reach(tree, file, line) {
                wrong.push(format!("src/{file}:{n}: {what}: {}", line.trim()));
            }
        }
    }
    assert!(wrong.is_empty(), "the boundary crossed:\n{}", wrong.join("\n"));
}

// The check itself: each kind of crossing caught, and what is allowed let through.
#[test]
fn the_source_check_catches_each_crossing() {
    assert_eq!(reach("mcd", "mcd/mod.rs", "use crate::interface::{Answer, Attachment};"), None);
    assert_eq!(reach("mcd", "mcd/state.rs", "use crate::mcd::cdc::Cdc;"), None);
    assert_eq!(reach("mcd", "mcd/cdda.rs", "let r = crate::genesis::sound::RATE;"), Some("crate::genesis".into()));
    assert_eq!(reach("mcd", "mcd/mod.rs", "use crate::machine::Machine;"), Some("crate::machine".into()));
    assert_eq!(reach("mcd", "mcd/mod.rs", "use crate::s32x::S32x;"), Some("crate::s32x".into()));
    assert!(reach("mcd", "mcd/mod.rs", "use crate::{interface, media};").is_some());
    assert!(reach("mcd", "mcd/mod.rs", "use super::genesis::Hw;").is_some());
    assert!(reach("mcd", "mcd/word.rs", "use super::super::genesis;").is_some());
    assert_eq!(reach("mcd", "mcd/word.rs", "use super::Lands;"), None);
    assert_eq!(reach("genesis", "genesis/mod.rs", "use crate::genesis::cart::Cart;"), None);
    assert_eq!(reach("genesis", "genesis/mod.rs", "use crate::interface::Bare;"), None);
    assert_eq!(reach("genesis", "genesis/mod.rs", "use crate::mcd::MegaCd;"), Some("crate::mcd".into()));
    assert_eq!(reach("genesis", "genesis/cart.rs", "crate::media::Header::read"), Some("crate::media".into()));
    assert_eq!(reach("interface", "interface.rs", "use crate::state::StateError;"), Some("crate::state".into()));
    let text = "use crate::interface::Bare;\n// crate::machine in a comment\n#[cfg(test)]\nmod tests {\n    use crate::machine::Machine;\n}\n";
    assert_eq!(production_lines(text), [(1, "use crate::interface::Bare;")]);
}
