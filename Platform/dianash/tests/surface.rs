//! The shell knows no emulator noun (EmuSen_DianaSh.md §2, §11.4): no identifier `emusen_session.h` declares appears
//! in the crate, and the crate depends on no crate of the emulator's but Galaxia's and the .NET rules split from it.

use std::path::Path;

fn sources(dir: &Path, out: &mut Vec<(String, String)>) {
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs" || e == "h") {
            out.push((path.display().to_string(), std::fs::read_to_string(&path).unwrap()));
        }
    }
}

/// The type names, functions and constants the session service's header declares.
fn session_identifiers(header: &str) -> Vec<String> {
    let mut names = Vec::new();
    for (at, _) in header.match_indices(['e', 'E']) {
        let word: String = header[at..].chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '_').collect();
        let starts_word = at == 0 || !header[..at].chars().next_back().is_some_and(|c| c.is_ascii_alphanumeric() || c == '_');
        if starts_word && (word.starts_with("emusen_session") || word.starts_with("EMUSEN_SESSION") || word.starts_with("EMUSEN_SVC_")) && !names.contains(&word) {
            names.push(word);
        }
    }
    names
}

#[test]
fn the_crate_names_nothing_of_the_session_service() {
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let header = std::fs::read_to_string(crate_dir.join("../../EmuSen.Crystal/include/emusen_session.h")).unwrap();
    let names = session_identifiers(&header);
    assert!(names.len() > 50, "the header's identifiers were not found: {names:?}");
    let mut files = Vec::new();
    sources(&crate_dir.join("src"), &mut files);
    sources(&crate_dir.join("include"), &mut files);
    for (file, text) in files {
        for name in &names {
            assert!(!text.contains(name.as_str()), "{file} names {name}, an identifier of emusen_session.h");
        }
    }
}

#[test]
fn the_crate_depends_on_no_emulator_crate_but_galaxia() {
    let manifest = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")).unwrap();
    for line in manifest.lines().map(str::trim) {
        if let Some((name, _)) = line.split_once(" = ") {
            if name.starts_with("emusen-") {
                assert!(["emusen-galaxia", "emusen-dotnet"].contains(&name), "the shell's crate depends on {name}");
            }
        }
    }
}
