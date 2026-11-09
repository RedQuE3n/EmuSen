//! `StackMember`: marks the item it is placed in as one member of a stack the kit lays out, whose members may lie over each
//! other in their z order; the harness finds it by its class (EmuSen_Kit.md §7.33). Only the kit's own QML places it:
//! `EmuSen.Theme` does not export it, so no theme can declare a stack of its own.

#[cxx_qt::bridge]
pub mod qobject {
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        type StackMember = super::StackMemberRust;
    }
}

#[derive(Default)]
pub struct StackMemberRust;
