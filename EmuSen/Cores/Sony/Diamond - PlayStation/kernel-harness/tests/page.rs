//! Diamond_OpenKernel.md's call table against the one the kernel's table.rs prints (§5).

use diamond_kernel_harness::calls;

#[test]
fn the_page_s_call_table_is_the_one_the_source_prints() {
    let page = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../../../../EmuSen.DianaSh/DianaSh/Etc/Man pages/Hardware/Sony/Diamond - PlayStation/Diamond_OpenKernel.md",
    );
    let text = std::fs::read_to_string(page).expect("Diamond_OpenKernel.md");
    let table = calls::markdown(&calls::parse(&calls::source()));
    assert!(
        text.contains(&table),
        "the page's §5 table is stale; print it with `cargo run --release --bin diamond-kernel -- calls`"
    );
}
