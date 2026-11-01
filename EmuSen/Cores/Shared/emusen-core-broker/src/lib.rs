//! The broker: a v1 core run in a confined process of its own, reached through a proxy library that is itself an
//! ordinary v1 core (EmuSen_LibretroHost.md §3; EmuSen_CoreAPI.md §7.3).

pub mod client;
pub mod proxy;
pub mod runner;
pub mod sandbox;
pub mod wire;
