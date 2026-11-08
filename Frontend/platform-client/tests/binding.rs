//! src/raw.rs is bindgen's output for emusen_platform.h (with the scrape service's table it includes), emusen_library.h
//! and emusen_games_service.h, committed so that a header change is a reviewed diff: this test regenerates it and fails
//! on any difference. `EMUSEN_BINDGEN_WRITE=1` writes it instead.

use std::path::Path;

#[test]
fn the_committed_binding_is_bindgen_s_output_for_the_headers() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let platform = here.join("../../Platform/include");
    let library = here.join("../../Platform/library/include");
    let made = bindgen::Builder::default()
        .header(platform.join("emusen_platform.h").to_str().unwrap())
        .header(library.join("emusen_library.h").to_str().unwrap())
        .header(platform.join("emusen_games_service.h").to_str().unwrap())
        .clang_arg(format!("-I{}", platform.display()))
        .allowlist_file(".*/emusen_(platform|scrape_service|library|games_service)\\.h")
        .dynamic_library_name("PlatformLibrary")
        .dynamic_link_require_all(true)
        .derive_default(true)
        .wrap_unsafe_ops(true)
        .layout_tests(true)
        .generate_comments(false)
        .raw_line("//! bindgen's output for emusen_platform.h and the headers it serves; regenerated and compared by tests/binding.rs, never edited.")
        .raw_line("#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case, dead_code, clippy::all)]")
        .generate()
        .expect("bindgen reads the headers")
        .to_string();
    let at = here.join("src/raw.rs");
    if std::env::var_os("EMUSEN_BINDGEN_WRITE").is_some() {
        std::fs::write(&at, &made).unwrap();
        return;
    }
    let committed = std::fs::read_to_string(&at).unwrap_or_default();
    assert!(
        committed == made,
        "src/raw.rs differs from bindgen's output for the headers: run with EMUSEN_BINDGEN_WRITE=1 and review the diff"
    );
}
