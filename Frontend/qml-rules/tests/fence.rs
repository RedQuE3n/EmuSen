// The fence of EmuSen_Kit.md §7.9, as kit-core's: the back end's dev module depends on qml-rules, so its tree names no Qt, no cxx-qt and no EmuSen crate.

const LOCK: &str = include_str!("../Cargo.lock");
const MANIFEST: &str = include_str!("../Cargo.toml");

fn locked_names() -> Vec<&'static str> {
    LOCK.lines().filter_map(|line| line.strip_prefix("name = \"")).filter_map(|rest| rest.strip_suffix('"')).collect()
}

#[test]
fn kit_7_9_the_dependency_tree_names_no_qt_cxx_or_emusen_crate() {
    let names = locked_names();
    assert!(names.contains(&"emusen-qml-rules"), "the lock lists the crate itself");
    for name in names {
        if name == "emusen-qml-rules" {
            continue;
        }
        let lower = name.to_ascii_lowercase();
        assert!(!lower.starts_with("emusen"), "an EmuSen crate in the tree: {name}");
        assert!(!lower.contains("qt") && !lower.contains("cxx"), "a Qt binding in the tree: {name}");
    }
}

#[test]
fn kit_7_9_the_manifest_reaches_into_no_other_component() {
    assert!(!MANIFEST.contains("path ="), "a path dependency reaches into another component");
    assert!(!MANIFEST.contains("workspace = true"), "the crate stands outside the Frontend workspace, lock file and all");
}
