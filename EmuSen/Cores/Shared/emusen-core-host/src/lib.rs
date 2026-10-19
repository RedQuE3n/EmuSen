//! The core host: what any program that runs a core on the core ABI v1 needs, and nothing a session adds above it.
//!
//! - [`discovery`] lists engines from their sidecars and opens one under [`trust`]'s rules (EmuSen_CoreAPI.md §7).
//! - [`library`] loads a library once for the process and checks it; [`exports`] is the typed table it calls through.
//! - [`machine`] is one owned machine with a method per v1 call, allocating nothing per frame.
//! - [`latch`] holds input from any thread and hands it to the machine on its own; [`layout`] reads a state by field.
//! - [`sidecar`], [`descriptors`] and [`sha256`] are the sidecar's format, the fields of core info a host reads, and
//!   the hash both record. [`listing`] writes what discovery found as the frontend API's `cores` and `engines` shape it.
//!
//! The frame-end order, settings layers, cheats, rewind and states' files are the runtime's. See EmuSen_Backend.md §27.

pub mod descriptors;
pub mod discovery;
pub mod exports;
pub mod latch;
pub mod layout;
pub mod library;
pub mod listing;
pub mod machine;
pub mod sha256;
pub mod sidecar;
pub mod trust;

pub use discovery::{Discovered, Discovery};
pub use latch::InputLatch;
pub use layout::StateLayout;
pub use library::Library;
pub use machine::{Create, Machine, Refusal};
pub use trust::Trust;
