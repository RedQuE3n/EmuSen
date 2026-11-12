// The fence of EmuSen_Qt.md §4.2: the loader, the scene and the shelves' schema are plain data, and the back end's dev module depends on the crate, so its tree names no Qt, no cxx-qt and no EmuSen crate.

const LOCK: &str = include_str!("../Cargo.lock");
const MANIFEST: &str = include_str!("../Cargo.toml");

#[test]
fn qt_4_2_the_dependency_tree_is_the_crate_and_its_xml_reader() {
    let names: Vec<&str> =
        LOCK.lines().filter_map(|line| line.strip_prefix("name = \"")).filter_map(|rest| rest.strip_suffix('"')).collect();
    assert!(names.contains(&"emusen-esde"), "the lock lists the crate itself");
    for name in names.iter().filter(|n| **n != "emusen-esde") {
        let lower = name.to_ascii_lowercase();
        assert!(!lower.starts_with("emusen"), "an EmuSen crate in the tree: {name}");
        assert!(!lower.contains("qt") && !lower.contains("cxx"), "a Qt binding in the tree: {name}");
    }
    let known = [
        "emusen-esde", "quick-xml", "memchr", "serde", "serde_core", "serde_derive", "proc-macro2", "quote", "syn", "unicode-ident",
    ];
    let unknown: Vec<&&str> = names.iter().filter(|n| !known.contains(n)).collect();
    assert!(unknown.is_empty(), "the crate, quick-xml and serde with its derive, and nothing else: {unknown:?}");
}

#[test]
fn qt_4_2_the_manifest_reaches_into_no_other_component() {
    assert!(!MANIFEST.contains("path ="), "a path dependency reaches into another component");
    assert!(!MANIFEST.contains("workspace = true"), "the crate stands outside the Frontend workspace, lock file and all");
}
