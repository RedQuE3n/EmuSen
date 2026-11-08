//! The client's fence (EmuSen_Qt.md §1.1.1, §2): its tree names no Qt binding and no runtime crate, and its only path
//! dependency is the test fixture, a dev-dependency.

const LOCK: &str = include_str!("../Cargo.lock");
const MANIFEST: &str = include_str!("../Cargo.toml");

#[test]
fn the_dependency_tree_names_no_qt_and_no_runtime_crate() {
    let names: Vec<&str> = LOCK.lines().filter_map(|l| l.strip_prefix("name = \"")).filter_map(|r| r.strip_suffix('"')).collect();
    assert!(names.contains(&"emusen-crystal-client"));
    for name in names.iter().map(|n| n.to_ascii_lowercase()) {
        assert!(!["qt", "cxx", "qml"].iter().any(|w| name.contains(w)), "a Qt crate in the tree: {name}");
        assert!(
            !name.starts_with("emusen") || ["emusen-crystal-client", "emusen-client-fixture"].contains(&name.as_str()),
            "a runtime crate in the tree: {name}"
        );
    }
}

#[test]
fn the_only_path_dependency_is_the_test_fixture() {
    let (deps, dev) = MANIFEST.split_once("[dev-dependencies]").expect("dev-dependencies");
    assert!(!deps.contains("path ="), "a path dependency reaches into another component");
    assert_eq!(dev.matches("path =").count(), 1);
}
