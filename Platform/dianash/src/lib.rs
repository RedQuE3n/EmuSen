//! DianaSh's shell core: a standalone Unix shell whose commands are builtins, modules and a host's own, reached
//! through the module ABI of `include/dianash_module.h`. It knows no emulator type: what a host offers its commands
//! is a service the shell copies into their calls and never reads. See EmuSen_DianaSh.md §4 to §6 and §10.
//!
//! The language and every builtin print what the C# shell printed, byte for byte, held by the golden corpus
//! (§13.1); where that meant reproducing .NET's own rules they are in `dotnet`.

pub mod accounts;
pub mod ast;
pub mod builtins;
pub mod dotnet;
mod host;
mod interp;
pub mod lexer;
pub mod man;
pub mod parser;
pub mod sandbox;
pub mod shell;
pub mod table;

/// The module ABI's types, owned by the crate a module is written with.
pub use dianash_module::abi;

pub use shell::{Config, Request, Shell, Submitted};

/// The host API's version: a major expected never to change, a minor grown by addition (§11).
pub const HOST_ABI_MAJOR: u32 = 1;
pub const HOST_ABI_MINOR: u32 = 0;
pub const HOST_ABI_VERSION: u32 = (HOST_ABI_MAJOR << 16) | HOST_ABI_MINOR;
