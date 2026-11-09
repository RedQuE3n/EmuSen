//! `Rows`, every list a theme reads as one read-only QAbstractListModel: its roles fixed when it is made, its rows
//! replaced whole by the host's feed (EmuSen_Qt.md §3.2). A theme cannot make one or write to one.

use crate::view::{Cell, Row};
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QByteArray, QHash, QHashPair_i32_QByteArray, QModelIndex, QString, QVariant};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        // The models' own declaration: cxx-qt emits an upcast shim per declaration, one in the program (EmuSen_Kit.md §7.6).
        include!(<QtCore/QAbstractListModel>);
        type QAbstractListModel = emusen_qt_models::library_model::qobject::QAbstractListModel;
        include!("cxx-qt-lib/qhash.h");
        type QHash_i32_QByteArray = cxx_qt_lib::QHash<cxx_qt_lib::QHashPair_i32_QByteArray>;
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
        include!("cxx-qt-lib/qmodelindex.h");
        type QModelIndex = cxx_qt_lib::QModelIndex;
        include!("cxx-qt-lib/qvector.h");
        type QVector_i32 = cxx_qt_lib::QVector<i32>;
    }

    extern "RustQt" {
        #[qobject]
        #[base = QAbstractListModel]
        #[qml_element = "Rows"]
        #[qml_uncreatable]
        #[qproperty(i32, count, READ, NOTIFY)]
        type ThemeRows = super::ThemeRowsRust;
    }

    extern "RustQt" {
        #[qinvokable]
        #[cxx_override]
        fn data(self: &ThemeRows, index: &QModelIndex, role: i32) -> QVariant;

        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &ThemeRows) -> QHash_i32_QByteArray;

        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &ThemeRows, parent: &QModelIndex) -> i32;
    }

    unsafe extern "RustQt" {
        #[inherit]
        #[cxx_name = "beginResetModel"]
        fn begin_reset_model(self: Pin<&mut ThemeRows>);

        #[inherit]
        #[cxx_name = "endResetModel"]
        fn end_reset_model(self: Pin<&mut ThemeRows>);

        #[inherit]
        fn index(self: &ThemeRows, row: i32, column: i32, parent: &QModelIndex) -> QModelIndex;

        #[inherit]
        #[qsignal]
        #[cxx_name = "dataChanged"]
        fn data_changed(self: Pin<&mut ThemeRows>, top_left: &QModelIndex, bottom_right: &QModelIndex, roles: &QVector_i32);
    }
}

/// The list's roles and rows.
#[derive(Default)]
pub struct ThemeRowsRust {
    count: i32,
    roles: &'static [&'static str],
    rows: Vec<Row>,
}

impl qobject::ThemeRows {
    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let Some(cell) = self.rows.get(index.row() as usize).and_then(|r| r.get(role as usize)) else {
            return QVariant::default();
        };
        match cell {
            Cell::Text(t) => QVariant::from(&QString::from(t.as_str())),
            Cell::Number(n) => QVariant::from(n),
            Cell::Whole(n) => QVariant::from(n),
            Cell::Flag(b) => QVariant::from(b),
        }
    }

    pub fn role_names(&self) -> QHash<QHashPair_i32_QByteArray> {
        let mut roles = QHash::<QHashPair_i32_QByteArray>::default();
        for (i, name) in self.roles.iter().enumerate() {
            roles.insert(i as i32, QByteArray::from(*name));
        }
        roles
    }

    pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
        self.rows.len() as i32
    }

    /// The list's role names, in their order.
    pub fn roles(&self) -> &'static [&'static str] {
        self.roles
    }

    /// The rows as fed.
    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    /// The roles, set once by the singleton that owns the list.
    pub fn set_roles(mut self: Pin<&mut Self>, roles: &'static [&'static str]) {
        self.as_mut().rust_mut().roles = roles;
    }

    /// The rows replaced: the same rows by key keep their delegates and change their cells in place, so a theme's focus
    /// and its items stay; other rows replace the list whole, as the library view replaces its own (EmuSen_Qt.md §1.2.1).
    pub fn replace(mut self: Pin<&mut Self>, rows: Vec<Row>) {
        debug_assert!(rows.iter().all(|r| r.len() == self.roles.len()), "a row with a cell for each role");
        if rows == self.rows {
            return;
        }
        let same = !rows.is_empty() && rows.len() == self.rows.len() && rows.iter().zip(&self.rows).all(|(a, b)| a.first() == b.first());
        if same {
            let last = rows.len() as i32 - 1;
            self.as_mut().rust_mut().rows = rows;
            let (first, last) = (self.index(0, 0, &QModelIndex::default()), self.index(last, 0, &QModelIndex::default()));
            self.as_mut().data_changed(&first, &last, &cxx_qt_lib::QVector::<i32>::default());
            return;
        }
        self.as_mut().begin_reset_model();
        let count = rows.len() as i32;
        self.as_mut().rust_mut().rows = rows;
        self.as_mut().end_reset_model();
        if self.count != count {
            self.as_mut().rust_mut().count = count;
            self.as_mut().count_changed();
        }
    }
}
