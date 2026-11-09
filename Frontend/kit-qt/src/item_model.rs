//! Any list model a layout reads by role, declared once: cxx-qt emits one upcast shim per declaration, so a second declaration clashes at link time (EmuSen_Kit.md §7.6).

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
        include!("cxx-qt-lib/qmodelindex.h");
        type QModelIndex = cxx_qt_lib::QModelIndex;
        include!("cxx-qt-lib/qhash.h");
        type QHash_i32_QByteArray = cxx_qt_lib::QHash<cxx_qt_lib::QHashPair_i32_QByteArray>;
    }

    unsafe extern "C++Qt" {
        include!(<QtCore/QAbstractItemModel>);
        #[qobject]
        type QAbstractItemModel;
        #[cxx_name = "rowCount"]
        fn row_count(self: &QAbstractItemModel, parent: &QModelIndex) -> i32;
        fn index(self: &QAbstractItemModel, row: i32, column: i32, parent: &QModelIndex) -> QModelIndex;
        fn data(self: &QAbstractItemModel, index: &QModelIndex, role: i32) -> QVariant;
        #[cxx_name = "roleNames"]
        fn role_names(self: &QAbstractItemModel) -> QHash_i32_QByteArray;
    }
}
