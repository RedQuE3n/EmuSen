// The kit's fence (EmuSen_Qt.md §1.4, §2): its dependency tree names no Qt binding, no runtime, client or Platform crate.

const LOCK: &str = include_str!("../Cargo.lock");
const MANIFEST: &str = include_str!("../Cargo.toml");

fn locked_names() -> Vec<&'static str> {
    LOCK.lines()
        .filter_map(|line| line.strip_prefix("name = \""))
        .filter_map(|rest| rest.strip_suffix('"'))
        .collect()
}

#[test]
fn qt_1_4_the_dependency_tree_names_no_qt_runtime_client_or_platform_crate() {
    let forbidden = ["qt", "cxx", "qml", "crystal", "galaxia", "endymion", "serenity", "beryl", "dianash", "libretro"];
    let names = locked_names();
    assert!(names.contains(&"emusen-kit-core"), "the lock lists the crate itself");
    for name in names {
        let lower = name.to_ascii_lowercase();
        if name == "emusen-kit-core" {
            continue;
        }
        assert!(!lower.starts_with("emusen"), "an emusen crate in the tree: {name}");
        for word in forbidden {
            assert!(!lower.contains(word), "a forbidden crate in the tree: {name}");
        }
    }
}

#[test]
fn qt_2_the_manifest_has_no_path_dependency() {
    assert!(!MANIFEST.contains("path ="), "a path dependency reaches into another component");
}
