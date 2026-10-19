//! The debugger beneath Crystal in Rust, step B7's part (b) (EmuSen_Backend.md §29.2, §29.30): what the C#
//! `EmuSen.Crystal/Debugger`, `CoreDebugTarget`, `CoreDebugBridge` and `UnitExpressionContext` do, over a machine of
//! `emusen-core-host` and the packs of `emusen-systems`, with nothing of sessions, states, settings or files.
//!
//! - [`expr`] is the expression language and [`context`] what it reads; [`conditions`] prepares and evaluates a
//!   breakpoint's condition and logpoint.
//! - [`breakpoints`], [`watches`], [`coverage`] and [`labels`] are the registries.

pub mod breakpoints;
pub mod conditions;
pub mod context;
pub mod coverage;
pub mod expr;
pub mod labels;
mod ordered;
pub mod watches;
