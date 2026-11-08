//! src/raw.rs is bindgen's output for emusen_crystal.h and emusen_session.h, committed so that a header change is a
//! reviewed diff: this test regenerates it and fails on any difference. `EMUSEN_BINDGEN_WRITE=1` writes it instead.

use std::path::Path;

#[test]
fn the_committed_binding_is_bindgen_s_output_for_the_header() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let include = here.join("../../Platform/crystal/include");
    let made = bindgen::Builder::default()
        .header(include.join("emusen_crystal.h").to_str().unwrap())
        .clang_arg(format!("-I{}", include.display()))
        .allowlist_file(".*/emusen_(crystal|session)\\.h")
        .dynamic_library_name("CrystalLibrary")
        .dynamic_link_require_all(true)
        .derive_default(true)
        .wrap_unsafe_ops(true)
        .layout_tests(true)
        .generate_comments(false)
        .raw_line("//! bindgen's output for emusen_crystal.h; regenerated and compared by tests/binding.rs, never edited.")
        .raw_line("#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case, dead_code, clippy::all)]")
        .generate()
        .expect("bindgen reads the header")
        .to_string();
    let at = here.join("src/raw.rs");
    if std::env::var_os("EMUSEN_BINDGEN_WRITE").is_some() {
        std::fs::write(&at, &made).unwrap();
        return;
    }
    let committed = std::fs::read_to_string(&at).unwrap_or_default();
    assert!(
        committed == made,
        "src/raw.rs differs from bindgen's output for the header: run with EMUSEN_BINDGEN_WRITE=1 and review the diff"
    );
}
