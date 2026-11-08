//! `emusen-qt-test`: the headless harness of EmuSen_Qt.md §8.1 (tier 1) and §8.3, recorded in EmuSen_Kit.md §7.9.
//! Offscreen with the software backend, the item tree read into kit-core's audit, QTest's verbs through one C++ block, PadEvents through the router's queue.

// The one unsafe verb takes the engine QML owns; the contract is cxx's. An audit takes the gate's eight facts at once rather than a struct of its own.
#![allow(clippy::missing_safety_doc, clippy::too_many_arguments)]

pub mod compositor;
pub mod ffi;
pub mod harness;
pub mod lint;
pub mod spec;
pub mod tree;
pub mod virtual_pad;

pub use harness::{Harness, Scene};
pub use tree::Tree;
