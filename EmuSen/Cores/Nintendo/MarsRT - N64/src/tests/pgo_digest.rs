//! build.rs's source digest: a profile goes stale when the crate or a path dependency it is built from changes.

use std::fs;
use std::path::{Path, PathBuf};

include!("../../pgo/digest.rs");

fn tree(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("marsrt-digest-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    for (file, text) in [
        ("core/Cargo.toml", "[dependencies]\nshared = { path = \"../shared\" }\n"),
        ("core/Cargo.lock", "lock"),
        ("core/src/lib.rs", "core"),
        ("core/src/tests/mod.rs", "core tests"),
        ("shared/Cargo.toml", "[package]"),
        ("shared/src/lib.rs", "shared"),
        ("shared/src/tests.rs", "shared tests"),
    ] {
        let path = dir.join(file);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
    dir
}

#[test]
fn the_digest_covers_the_path_dependencies_and_not_the_tests() {
    let dir = tree("deps");
    let core = dir.join("core");
    let base = source_digest(&core);
    let edit = |file: &str, text: &str| fs::write(dir.join(file), text).unwrap();
    edit("shared/src/tests.rs", "changed");
    edit("core/src/tests/mod.rs", "changed");
    assert_eq!(source_digest(&core), base, "a test file moved the digest");
    edit("shared/src/lib.rs", "changed");
    let shared = source_digest(&core);
    assert_ne!(shared, base, "an edit in the dependency left the digest alone");
    edit("core/src/lib.rs", "changed");
    assert_ne!(source_digest(&core), shared);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn the_crate_names_the_state_crate_as_a_path_dependency() {
    let manifest = include_str!("../../Cargo.toml");
    let dependencies = path_dependencies(manifest);
    assert!(dependencies.iter().any(|d| d.ends_with("Shared/emusen-native")), "{dependencies:?}");
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert!(dependencies.iter().all(|d| root.join(d).join("src").join("lib.rs").is_file()), "{dependencies:?}");
}
