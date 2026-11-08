// The fence: the back end's dev module links theme-check, so its tree names no Qt and no EmuSen crate beyond the three it reads.

const LOCK: &str = include_str!("../Cargo.lock");

#[test]
fn qt_3_7_the_dependency_tree_names_no_qt_and_only_the_schema_the_rules_and_the_kit_core() {
    let names: Vec<&str> = LOCK.lines().filter_map(|l| l.strip_prefix("name = \"")).filter_map(|r| r.strip_suffix('"')).collect();
    let ours = ["emusen-theme-check", "emusen-theme-manifest", "emusen-qml-rules", "emusen-kit-core"];
    for name in &ours {
        assert!(names.contains(name), "the lock lists {name}");
    }
    for name in names {
        let lower = name.to_ascii_lowercase();
        assert!(!lower.starts_with("emusen") || ours.contains(&name), "another EmuSen crate in the tree: {name}");
        assert!(!lower.contains("qt") && !lower.contains("cxx"), "a Qt binding in the tree: {name}");
    }
}
