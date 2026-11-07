//! Qt base classes declared once for every bridge that subclasses them: cxx-qt emits one upcast shim per declaration, so a second declaration clashes at link time (EmuSen_Kit.md §7.6).

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++Qt" {
        include!(<QtCore/QAbstractListModel>);
        #[qobject]
        type QAbstractListModel;
    }
}
