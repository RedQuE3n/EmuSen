// The one C file, libretro's printf-style log callback (EmuSen_LibretroHost.md §1.2).
fn main() {
    println!("cargo:rerun-if-changed=c/log.c");
    cc::Build::new().file("c/log.c").warnings(true).compile("emusen_libretro_log");
}
