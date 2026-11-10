//! The ES-DE painter (EmuSen_Qt.md §4.8): the QML module `EmuSen.Esde` over the scene of `emusen-esde`. The scene decides
//! every place and time; this crate puts its records on Qt Quick items and decides nothing of its own.

// A read-only property set from Rust: the field changed and its signal raised, only when the value differs.
macro_rules! publish {
    ($m:expr, $field:ident, $signal:ident, $value:expr) => {{
        let value = $value;
        if $m.$field != value {
            $m.as_mut().rust_mut().$field = value;
            $m.as_mut().$signal();
        }
    }};
}

pub mod base;
pub mod help;
pub mod hinted;
pub mod lists;
pub mod marks;
pub mod model;
pub mod painter;
pub mod shelves;
pub mod text;
pub mod values;
