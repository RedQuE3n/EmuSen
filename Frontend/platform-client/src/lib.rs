//! emusen-platform-client: the platform library's C interface, emusen_platform.h, as safe Rust, matched exactly to the
//! library built from the same commit, over a raw layer bindgen generated from the headers (src/raw.rs). Covered for
//! stage 1: Galaxia's directories, files and models, the library (emusen_library.h), Endymion's pads, router, interface player
//! and binding files, the firmware folders and the scrape service; the rest of the header is owed, by name, in
//! EmuSen_Qt.md §1.1.2. The systems table, firmware status and the cheat codecs are the frontend API's
//! (emusen-crystal-client). Nothing here knows Qt or names a runtime type.

pub mod audio;
pub mod bindings;
mod error;
pub mod firmware;
pub mod galaxia;
pub mod library;
pub mod pace;
mod out;
pub mod pads;
pub mod router;
mod platform;
mod raw;
pub mod scrape;
pub mod serenity;
pub mod sounds;

pub use error::{Error, Result, status_name};
pub use platform::{Lent, Platform};

/// The interface number this binding was generated for.
pub const INTERFACE: u32 = raw::EMUSEN_PLATFORM_ABI_VERSION;
