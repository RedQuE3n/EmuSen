// The linker script places the program at 80010000h as a PS-EXE's body (Diamond_Shell.md §1.3).
fn main() {
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("cargo:rustc-link-arg-bins=-T{dir}/shell.ld");
    println!("cargo:rerun-if-changed=shell.ld");
}
