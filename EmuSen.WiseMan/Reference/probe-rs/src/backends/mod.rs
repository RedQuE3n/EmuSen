// The roster: the only module that knows which emulators exist. Everything
// above it - the policy layer, the dump format, the audio encoder - is held to
// that by tests::no_emulator_is_named_in_the_agnostic_layers in main.rs.
// See EmuSen_Debugging_Tools_Reference_v5.md §3.53.
use crate::backend::ProbeBackend;

#[cfg(feature = "libretro")]
mod libretro;

#[cfg(feature = "mesen")]
mod mesen;
#[cfg(feature = "mesen")]
mod mesen_sys;

// Which backends are compiled in is a build-time choice, because linking one is
// not free: the Mesen backend pulls in a 14 MB MesenCore.so, the libretro one
// needs nothing but dlopen. One binary per backend, one policy layer for all.
#[cfg(feature = "mesen")]
pub const DEFAULT: &str = "mesen";
#[cfg(all(feature = "libretro", not(feature = "mesen")))]
pub const DEFAULT: &str = "libretro";

// In roster order, so the "not compiled in" message can say what is.
#[allow(clippy::vec_init_then_push)] // the pushes are cfg-gated; vec![] cannot be
pub fn names() -> Vec<&'static str> {
    #[allow(unused_mut)]
    let mut names = Vec::new();
    #[cfg(feature = "mesen")]
    names.push("mesen");
    #[cfg(feature = "libretro")]
    names.push("libretro");
    names
}

// Help for flags only some backends read. The policy layer prints these without
// knowing what any of them mean, which is why they arrive as finished lines.
#[allow(clippy::vec_init_then_push)]
pub fn usage_lines() -> Vec<&'static str> {
    #[allow(unused_mut)]
    let mut lines = Vec::new();
    #[cfg(feature = "libretro")]
    lines.push("  --core PATH      libretro backend only: the *_libretro.so/.dylib to drive");
    lines
}

// `core` is the opaque --core string; only a backend that wants one reads it.
#[allow(unused_variables)]
pub fn make(name: &str, core: &str) -> Option<Box<dyn ProbeBackend>> {
    #[cfg(feature = "mesen")]
    if name == "mesen" {
        if !mesen::check_abi() {
            return None;
        }
        return Some(Box::new(mesen::MesenBackend::new()));
    }

    #[cfg(feature = "libretro")]
    if name == "libretro" {
        if core.is_empty() {
            println!("[ERROR] --backend libretro needs --core <path to *_libretro.so/.dylib>");
            return None;
        }
        return Some(Box::new(libretro::LibretroBackend::new(core)));
    }

    println!(
        "[ERROR] backend '{name}' is not compiled into this probe (built with: {})",
        names().join(", ")
    );
    None
}
