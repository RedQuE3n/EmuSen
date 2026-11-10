//! Where a test finds the checkout's published program: the main checkout, which a worktree shares by git's common
//! directory, holds `out/` (Testing/qt-gates/sandbox.sh and Testing/handheld/pack.sh read the same tree).

use std::path::PathBuf;
use std::process::Command;

/// The main checkout's root, the parent of git's common directory, from the main checkout and from every worktree.
pub fn root() -> PathBuf {
    let out = Command::new("git")
        .args(["rev-parse", "--path-format=absolute", "--git-common-dir"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("git runs");
    assert!(out.status.success(), "not in a git checkout: {}", String::from_utf8_lossy(&out.stderr));
    let common = PathBuf::from(String::from_utf8(out.stdout).expect("a UTF-8 path").trim());
    common.parent().expect("the common directory has a parent").to_path_buf()
}

/// The published installation's folder of engines, `out/linux-x64/Mistress/lib/EmuSen` of the main checkout.
pub fn installed_engines() -> PathBuf {
    root().join("out/linux-x64/Mistress/lib/EmuSen")
}
