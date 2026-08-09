// Binds libretro.h at build time rather than vendoring a hand-written copy of
// the ABI, which is the same rule build-probe.sh already follows for the C++
// backend: the header comes from the distro's retroarch-devel so it cannot
// drift from the real ABI. See EmuSen_Debugging_Tools_Reference_v5.md §3.50.
// Gated on cfg(feature) rather than the CARGO_FEATURE_* environment because
// bindgen is an *optional* build-dependency: under `--features mesen` alone the
// crate is not linked into this script at all, so a runtime check would still
// have to name `bindgen::` and fail to compile. Build scripts do receive the
// feature cfgs - verified, not assumed.
#[cfg(feature = "libretro")]
use std::path::PathBuf;
use std::path::Path;

fn main() {
    #[cfg(feature = "mesen")]
    link_mesen();

    #[cfg(feature = "libretro")]
    generate_libretro_bindings();
}

#[cfg(feature = "libretro")]
fn generate_libretro_bindings() {
    println!("cargo:rerun-if-env-changed=LIBRETRO_INCLUDE_DIR");

    let include_dir = find_include_dir().expect(
        "libretro.h not found. Set LIBRETRO_INCLUDE_DIR, or run \
         `./build-probe.sh libretro-headers` to fetch it without installing anything.",
    );
    let header = include_dir.join("libretro.h");
    println!("cargo:rerun-if-changed={}", header.display());

    // An allowlist keeps the generated module to the ABI this probe actually
    // speaks; libretro.h in full is several thousand lines of unrelated
    // frontend surface.
    let bindings = bindgen::Builder::default()
        .header(header.to_string_lossy())
        .allowlist_type("retro_.*")
        .allowlist_var("RETRO_.*")
        .allowlist_function("retro_.*")
        .derive_default(true)
        .layout_tests(false)
        .generate()
        .expect("bindgen could not parse libretro.h");

    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("libretro_sys.rs");
    bindings.write_to_file(&out).expect("could not write libretro_sys.rs");
}

// The Mesen backend links MesenCore.so directly rather than dlopening it, which
// is what the C++ probe did and is deliberately preserved: the recorded
// DT_NEEDED stays the relative `bin/pgohelperlib.so`, so the probe still has to
// be run from the checkout and still resolves the same library the C++ baseline
// did. Making it position-independent would be a nicer binary and a worse
// experiment - the A/B is only meaningful if both probes load the same core the
// same way. See EmuSen_Debugging_Tools_Reference_v5.md §3.52.
#[cfg(feature = "mesen")]
fn link_mesen() {
    println!("cargo:rerun-if-env-changed=MESEN_CHECKOUT");

    let checkout = std::env::var("MESEN_CHECKOUT").expect(
        "MESEN_CHECKOUT is not set. Build the mesen backend through \
         ./build-probe.sh mesen <checkout>, which builds MesenCore.so first.",
    );
    let library = Path::new(&checkout).join("bin/pgohelperlib.so");
    assert!(
        library.is_file(),
        "{} does not exist - run ./build-probe.sh mesen <checkout> to build it",
        library.display()
    );
    println!("cargo:rerun-if-changed={}", library.display());

    // -l:<relative path> against -L<checkout>: ld finds the file by
    // concatenation and records the name exactly as written, which is how the
    // relative DT_NEEDED survives cargo linking from a different directory.
    println!("cargo:rustc-link-arg=-L{checkout}");
    println!("cargo:rustc-link-arg=-l:bin/pgohelperlib.so");
}

#[cfg(feature = "libretro")]
fn find_include_dir() -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Ok(dir) = std::env::var("LIBRETRO_INCLUDE_DIR") {
        candidates.push(PathBuf::from(dir));
    }

    // Where build-probe.sh unpacks retroarch-devel, so a plain `cargo build`
    // works after a `./build-probe.sh libretro` without any environment.
    let cache = std::env::var("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|_| std::env::var("HOME").map(|h| Path::new(&h).join(".cache")));
    if let Ok(cache) = cache {
        candidates.push(cache.join("emusen/probe/libretro/deps/usr/include/libretro-common"));
    }

    candidates.push(PathBuf::from("/usr/include/libretro-common"));
    candidates.push(PathBuf::from("/usr/include"));

    candidates.into_iter().find(|dir| dir.join("libretro.h").is_file())
}
