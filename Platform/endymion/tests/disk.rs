//! The disk corpus of the binding files: the files as found on the project's machines and as the C# classes wrote them on
//! 2026-11-08, and variants edited by hand, each beside the file the C# classes wrote back after loading it (`.model`)
//! and, where they reported one, their diagnostic (`.said`). Recorded once from the C# before it stopped reading the
//! files, and never re-recorded to make a change pass. See EmuSen_Input.md §5.2.

use emusen_endymion::bindings::File;
use serde_json::Value;
use std::path::{Path, PathBuf};

const DISK: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/disk");

/// The consoles every file is read for: four the files name and one they never do.
const CONSOLES: [&str; 5] = ["NES", "SNES", "GB", "N64", "PS1"];

fn files(file: File) -> Vec<PathBuf> {
    let folder = Path::new(DISK).join(file.name().trim_end_matches(".json"));
    let mut found: Vec<PathBuf> = std::fs::read_dir(&folder).unwrap().map(|e| e.unwrap().path()).filter(|p| p.extension().is_some_and(|e| e == "json")).collect();
    found.sort();
    assert!(found.len() >= 4, "{} holds too few files", folder.display());
    found
}

fn text(path: &Path) -> String {
    emusen_galaxia::text_file::decode(&std::fs::read(path).unwrap())
}

/// Two documents alike member by member, in order.
fn same(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Object(x), Value::Object(y)) => x.len() == y.len() && x.iter().zip(y).all(|((kx, vx), (ky, vy))| kx == ky && same(vx, vy)),
        (Value::Array(x), Value::Array(y)) => x.len() == y.len() && x.iter().zip(y).all(|(x, y)| same(x, y)),
        _ => a == b,
    }
}

fn json(text: &str) -> Value {
    serde_json::from_str(text).unwrap_or_else(|e| panic!("{text} is not JSON: {e}"))
}

#[test]
fn every_binding_file_reads_to_what_the_csharp_wrote_back() {
    for file in File::ALL {
        for path in files(file) {
            let (model, words) = file.read(Some(&text(&path)), &CONSOLES);
            assert_eq!(words.is_some(), path.with_extension("said").exists(), "{}: {words:?}", path.display());
            let written = file.text(&model).unwrap();
            let want = std::fs::read_to_string(path.with_extension("model")).unwrap();
            assert!(same(&json(&written), &json(&want)), "{} writes back\n{written}\nand the C# wrote\n{want}", path.display());
        }
    }
}

#[test]
fn a_binding_file_read_written_and_read_again_is_the_same_model() {
    for file in File::ALL {
        for path in files(file) {
            let (model, _) = file.read(Some(&text(&path)), &CONSOLES);
            let (again, words) = file.read(Some(&file.text(&model).unwrap()), &CONSOLES);
            assert_eq!(words, None, "{}", path.display());
            assert_eq!(again, model, "{}", path.display());
        }
    }
}

#[test]
fn a_refusal_names_the_word_and_the_nearest_real_one() {
    let refused = |file: File, name: &str| file.read(Some(&text(&Path::new(DISK).join(name))), &CONSOLES).1.unwrap();
    assert_eq!(refused(File::Gamepad, "gamepadbindings/misspelled-value.json"), "$.Up: 'DPadUpp' is not a valid GamepadButton. Did you mean 'DPadUp'?");
    assert_eq!(refused(File::Gamepad, "gamepadbindings/misspelled-key.json"), "$.NES: 'Strat' is not a valid PadButton. Did you mean 'Start'?");
    assert_eq!(refused(File::Hotkeys, "hotkeybindings/unknown-action.json"), "'Pause' is not a valid HotkeyAction.");
}

#[test]
fn no_file_is_the_defaults_and_a_null_document_too() {
    for file in File::ALL {
        let (none, words) = file.read(None, &CONSOLES);
        assert_eq!(words, None);
        assert_eq!(file.read(Some("null"), &CONSOLES), (none.clone(), None), "{}", file.name());
        if let Some(map) = file.default_map() {
            assert!(none.contains(&map[1..map.len() - 1]), "{}: {none} holds no {map}", file.name());
        }
    }
    assert_eq!(File::Profiles.read(None, &CONSOLES).0, "{}");
}
