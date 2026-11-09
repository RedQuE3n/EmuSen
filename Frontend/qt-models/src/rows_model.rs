//! `RowsModel`: rows given as plain data, a JSON array of objects, put on Qt as a list model whose roles are the objects'
//! keys, so a view model hands any list (a tool's rows, columns, facts, sentences, help, a card's rows) as text and QML
//! reads it by role (EmuSen_Qt.md §1.2.2). The same keys and count change the rows in place, so a focused delegate stays.

use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QByteArray, QHash, QHashPair_i32_QByteArray, QModelIndex, QString, QVariant, QVector};
use serde_json::Value;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!(<QtCore/QAbstractListModel>);
        type QAbstractListModel = crate::library_model::qobject::QAbstractListModel;
    }

    unsafe extern "C++" {
        include!("cxx-qt-lib/qhash.h");
        type QHash_i32_QByteArray = cxx_qt_lib::QHash<cxx_qt_lib::QHashPair_i32_QByteArray>;
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
        include!("cxx-qt-lib/qmodelindex.h");
        type QModelIndex = cxx_qt_lib::QModelIndex;
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qvector.h");
        type QVector_i32 = cxx_qt_lib::QVector<i32>;
    }

    extern "RustQt" {
        #[qobject]
        #[base = QAbstractListModel]
        #[qml_element]
        /// The rows, a JSON array of objects.
        #[qproperty(QString, rows)]
        /// How many rows there are.
        #[qproperty(i32, count, READ, NOTIFY)]
        type RowsModel = super::RowsModelRust;
    }

    extern "RustQt" {
        #[qinvokable]
        #[cxx_override]
        fn data(self: &RowsModel, index: &QModelIndex, role: i32) -> QVariant;

        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &RowsModel) -> QHash_i32_QByteArray;

        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &RowsModel, parent: &QModelIndex) -> i32;
    }

    unsafe extern "RustQt" {
        #[inherit]
        #[cxx_name = "beginResetModel"]
        fn begin_reset_model(self: Pin<&mut RowsModel>);

        #[inherit]
        #[cxx_name = "endResetModel"]
        fn end_reset_model(self: Pin<&mut RowsModel>);

        #[inherit]
        fn index(self: &RowsModel, row: i32, column: i32, parent: &QModelIndex) -> QModelIndex;

        #[inherit]
        #[qsignal]
        #[cxx_name = "dataChanged"]
        fn data_changed(self: Pin<&mut RowsModel>, top_left: &QModelIndex, bottom_right: &QModelIndex, roles: &QVector_i32);
    }

    impl cxx_qt::Initialize for RowsModel {}
}

#[derive(Default)]
pub struct RowsModelRust {
    rows: QString,
    count: i32,
    roles: Vec<String>,
    read: Vec<Value>,
}

impl cxx_qt::Initialize for qobject::RowsModel {
    fn initialize(self: Pin<&mut Self>) {
        self.on_rows_changed(|model| model.read()).release();
    }
}

/// The rows of a JSON array of objects and every key they use, in order; anything else is no rows.
pub fn rows_of(text: &str) -> (Vec<Value>, Vec<String>) {
    let rows: Vec<Value> = serde_json::from_str::<Value>(text).ok().and_then(|v| v.as_array().cloned()).unwrap_or_default();
    let mut roles: Vec<String> = rows.iter().filter_map(Value::as_object).flat_map(|o| o.keys().cloned()).collect();
    roles.sort();
    roles.dedup();
    (rows, roles)
}

fn variant(value: &Value) -> QVariant {
    match value {
        Value::Bool(b) => QVariant::from(b),
        Value::Number(n) if n.is_i64() => QVariant::from(&(n.as_i64().unwrap_or(0) as i32)),
        Value::Number(n) => QVariant::from(&n.as_f64().unwrap_or(0.0)),
        Value::String(s) => QVariant::from(&QString::from(s.as_str())),
        Value::Null => QVariant::default(),
        other => QVariant::from(&QString::from(other.to_string().as_str())),
    }
}

impl qobject::RowsModel {
    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let (Some(row), Some(name)) = (self.read.get(index.row() as usize), self.roles.get(role as usize)) else { return QVariant::default() };
        row.get(name).map(variant).unwrap_or_default()
    }

    pub fn role_names(&self) -> QHash<QHashPair_i32_QByteArray> {
        let mut roles = QHash::<QHashPair_i32_QByteArray>::default();
        for (at, name) in self.roles.iter().enumerate() {
            roles.insert(at as i32, QByteArray::from(name.as_str()));
        }
        roles
    }

    pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
        self.read.len() as i32
    }

    fn read(mut self: Pin<&mut Self>) {
        let (rows, roles) = rows_of(&self.rows.to_string());
        if rows == self.read {
            return;
        }
        if roles == self.roles && rows.len() == self.read.len() {
            let count = rows.len() as i32;
            self.as_mut().rust_mut().read = rows;
            let root = QModelIndex::default();
            let (first, last) = (self.index(0, 0, &root), self.index(count - 1, 0, &root));
            self.as_mut().data_changed(&first, &last, &QVector::<i32>::default());
        } else {
            self.as_mut().begin_reset_model();
            self.as_mut().rust_mut().read = rows;
            self.as_mut().rust_mut().roles = roles;
            self.as_mut().end_reset_model();
        }
        let count = self.read.len() as i32;
        if self.count != count {
            self.as_mut().rust_mut().count = count;
            self.count_changed();
        }
    }
}
