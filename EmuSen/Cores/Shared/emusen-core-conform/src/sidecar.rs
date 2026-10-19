//! A library's sidecar, `<library>.core.json`, written by loading the library once, which is the build step's work.
//! The format is the core host's, read and written there (EmuSen_Backend.md §27.4).

pub use emusen_core_host::sidecar::write;
