// The schema's fence: the theme host and theme-check (which the back end's dev module links) both read it, so its tree names no Qt and no other EmuSen crate.

const LOCK: &str = include_str!("../Cargo.lock");
const MANIFEST: &str = include_str!("../Cargo.toml");

#[test]
fn qt_3_4_the_dependency_tree_names_no_qt_cxx_or_other_emusen_crate() {
    let names: Vec<&str> = LOCK.lines().filter_map(|l| l.strip_prefix("name = \"")).filter_map(|r| r.strip_suffix('"')).collect();
    assert!(names.contains(&"emusen-theme-manifest"), "the lock lists the crate itself");
    for name in names.into_iter().filter(|n| *n != "emusen-theme-manifest") {
        let lower = name.to_ascii_lowercase();
        assert!(!lower.starts_with("emusen"), "an EmuSen crate in the tree: {name}");
        assert!(!lower.contains("qt") && !lower.contains("cxx"), "a Qt binding in the tree: {name}");
    }
    assert!(!MANIFEST.contains("path =") && !MANIFEST.contains("workspace = true"), "the crate stands alone, lock file and all");
}
