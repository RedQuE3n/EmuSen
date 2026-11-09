//! `ViewCurrent`: a walkable that takes the focus inside a `ListView` or `GridView` makes its delegate the view's current item,
//! so the view's keys move on from the item the pointer or a walk focused (EmuSen_Kit.md §7.39).

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!(<QtQuick/QQuickItem>);
        type QQuickItem = crate::frame::qobject::QQuickItem;
        include!("kit_glue.h");
        /// Makes the delegate holding the item its item view's current item, if it stands in one.
        unsafe fn make_current(item: *mut QQuickItem);
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        type ViewCurrent = super::ViewCurrentRust;
    }

    unsafe extern "RustQt" {
        #[qinvokable]
        #[cxx_name = "take"]
        unsafe fn take(self: &ViewCurrent, item: *mut QQuickItem);
    }
}

#[derive(Default)]
pub struct ViewCurrentRust;

impl qobject::ViewCurrent {
    /// # Safety
    /// `item` is the QML item that called, alive for the call.
    pub unsafe fn take(&self, item: *mut qobject::QQuickItem) {
        if !item.is_null() {
            unsafe { qobject::make_current(item) };
        }
    }
}
