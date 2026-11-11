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
        /// Qt's private signals, heard and never sent from here: a model's reset begins and ends (`ResetGate`).
        #[qsignal]
        #[cxx_name = "modelAboutToBeReset"]
        pub(self) fn model_about_to_be_reset(self: Pin<&mut QAbstractItemModel>);
        #[qsignal]
        #[cxx_name = "modelReset"]
        pub(self) fn model_reset(self: Pin<&mut QAbstractItemModel>);
    }
}

use cxx_qt_lib::{QModelIndex, QString};

/// Each row's value of the role named `role`, read as text (a number's digits); a model without the role gives every row
/// the empty text, and no model gives no rows.
pub fn role_texts(model: Option<&ffi::QAbstractItemModel>, role: &QString) -> Vec<String> {
    let Some(model) = model else { return Vec::new() };
    let root = QModelIndex::default();
    let wanted = role.to_string();
    let role = model.role_names().iter().find(|(_, name)| name.to_string() == wanted).map(|(r, _)| *r);
    (0..model.row_count(&root))
        .map(|row| {
            let value = role.map(|r| model.data(&model.index(row, 0, &root), r)).unwrap_or_default();
            value.value::<QString>().map(|s| s.to_string()).unwrap_or_default()
        })
        .collect()
}
