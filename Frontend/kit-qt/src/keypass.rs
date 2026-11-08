//! `KeyPass`: a key a combo box declines on purpose, Up and Down while its list is closed, handed to the item's ancestors as Qt's own
//! propagation would, so a vertical walk passes such a row and the control takes the keys only once chosen (EmuSen_Kit.md §7.24).

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!(<QtQuick/QQuickItem>);
        type QQuickItem = crate::frame::qobject::QQuickItem;
        include!("kit_glue.h");
        /// Offers a key press and release to each ancestor of the item in turn until one accepts it.
        unsafe fn pass_key(item: *mut QQuickItem, key: i32, modifiers: i32) -> bool;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        type KeyPass = super::KeyPassRust;
    }

    unsafe extern "RustQt" {
        /// The key goes to the item's ancestors and never to the item; true always, so the control does not act on it.
        #[qinvokable]
        #[cxx_name = "toAncestors"]
        unsafe fn to_ancestors(self: &KeyPass, item: *mut QQuickItem, key: i32, modifiers: i32) -> bool;
    }
}

#[derive(Default)]
pub struct KeyPassRust;

impl qobject::KeyPass {
    /// # Safety
    /// `item` is the QML item that called, alive for the call.
    pub unsafe fn to_ancestors(&self, item: *mut qobject::QQuickItem, key: i32, modifiers: i32) -> bool {
        if !item.is_null() {
            unsafe { qobject::pass_key(item, key, modifiers) };
        }
        true
    }
}
