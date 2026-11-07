//! The cores' golden traces in their tests, one place for the format, the generator and the driver (EmuSen_Backend.md §40.10).

pub mod dotnet_random;
pub mod drive;
pub mod md5;
pub mod trace;

pub use dotnet_random::DotNetRandom;
pub use drive::Driven;
pub use trace::Trace;
