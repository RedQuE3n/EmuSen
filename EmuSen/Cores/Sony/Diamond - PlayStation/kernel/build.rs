// The linker script places the image at the BIOS's addresses (Diamond_OpenKernel.md §2).
fn main() {
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("cargo:rustc-link-arg-bins=-T{dir}/kernel.ld");
    println!("cargo:rerun-if-changed=kernel.ld");
}
