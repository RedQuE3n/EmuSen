// The main checkout's root, found from wherever the crate is built (a worktree included): it holds git's directory and
// the Frontend, and is the checkout this crate's own common directory names.
#[test]
fn the_main_checkout_is_found_from_a_worktree_too() {
    let root = emusen_qt_test::checkout::root();
    assert!(root.join(".git").is_dir(), "{} holds git's own directory", root.display());
    assert!(root.join("Frontend").is_dir(), "{} holds the Frontend", root.display());
    assert!(emusen_qt_test::checkout::installed_engines().starts_with(&root));
}
