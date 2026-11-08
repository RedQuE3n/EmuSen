//! Qt base classes declared once for the crate's bridges, as the kit's `base.rs` does (EmuSen_Kit.md §7.6).

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++Qt" {
        include!(<QtCore/QAbstractListModel>);
        #[qobject]
        type QAbstractListModel;
    }
}
