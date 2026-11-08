//! The client's fence (EmuSen_Qt.md §1.1.1, §2): its tree names no Qt binding and no runtime crate, and its path
//! dependencies are the shared idioms and, as a dev-dependency, the test fixture.

const LOCK: &str = include_str!("../Cargo.lock");
const MANIFEST: &str = include_str!("../Cargo.toml");

#[test]
fn the_dependency_tree_names_no_qt_and_no_runtime_crate() {
    let names: Vec<&str> = LOCK.lines().filter_map(|l| l.strip_prefix("name = \"")).filter_map(|r| r.strip_suffix('"')).collect();
    assert!(names.contains(&"emusen-crystal-client"));
    for name in names.iter().map(|n| n.to_ascii_lowercase()) {
        assert!(!["qt", "cxx", "qml"].iter().any(|w| name.contains(w)), "a Qt crate in the tree: {name}");
        assert!(
            !name.starts_with("emusen") || ["emusen-crystal-client", "emusen-c-calls", "emusen-client-fixture"].contains(&name.as_str()),
            "a runtime crate in the tree: {name}"
        );
    }
}

#[test]
fn the_path_dependencies_are_the_shared_idioms_and_the_test_fixture() {
    let (deps, dev) = MANIFEST.split_once("[dev-dependencies]").expect("dev-dependencies");
    assert_eq!(deps.matches("path =").count(), 1, "a path dependency reaches into another component");
    assert!(deps.contains("emusen-c-calls = { path = \"../c-calls\" }"));
    assert_eq!(dev.matches("path =").count(), 1);
}
