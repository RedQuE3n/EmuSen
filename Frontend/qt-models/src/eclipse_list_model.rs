//! `EclipseListModel`: virtualized `QAbstractListModel` with row-scoped diffs and native `QVariant` items.

use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QByteArray, QHash, QHashPair_i32_QByteArray, QModelIndex, QString, QVariant, QVector};
use serde_json::Value;

const ROLE_ITEM: i32 = 0;
const ROLE_KIND: i32 = 1;

/// Parses a JSON value string directly into a Qt `QVariant` without QML JS evaluation.
pub fn json_to_variant(json: &str) -> QVariant {
    qobject::json_to_variant(&QString::from(json))
}

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!(<QtCore/QAbstractListModel>);
        type QAbstractListModel = crate::library_model::qobject::QAbstractListModel;
    }

    unsafe extern "C++" {
        include!("cxx-qt-lib/qbytearray.h");
        type QByteArray = cxx_qt_lib::QByteArray;
        include!("cxx-qt-lib/qhash.h");
        type QHash_i32_QByteArray = cxx_qt_lib::QHash<cxx_qt_lib::QHashPair_i32_QByteArray>;
        include!("cxx-qt-lib/qmodelindex.h");
        type QModelIndex = cxx_qt_lib::QModelIndex;
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
        include!("cxx-qt-lib/qvector.h");
        type QVector_i32 = cxx_qt_lib::QVector<i32>;
        include!("json_variant.h");

        #[cxx_name = "jsonToVariant"]
        fn json_to_variant(json: &QString) -> QVariant;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[base = QAbstractListModel]
        #[qproperty(QString, rows)]
        #[qproperty(i32, count, READ, NOTIFY)]
        type EclipseListModel = super::EclipseListModelRust;

        #[qinvokable]
        #[cxx_override]
        fn data(self: &EclipseListModel, index: &QModelIndex, role: i32) -> QVariant;

        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &EclipseListModel) -> QHash_i32_QByteArray;

        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &EclipseListModel, parent: &QModelIndex) -> i32;
    }

    unsafe extern "RustQt" {
        #[inherit]
        #[cxx_name = "beginResetModel"]
        fn begin_reset_model(self: Pin<&mut EclipseListModel>);

        #[inherit]
        #[cxx_name = "endResetModel"]
        fn end_reset_model(self: Pin<&mut EclipseListModel>);

        #[inherit]
        fn index(self: &EclipseListModel, row: i32, column: i32, parent: &QModelIndex) -> QModelIndex;

        #[inherit]
        #[qsignal]
        #[cxx_name = "dataChanged"]
        fn data_changed(self: Pin<&mut EclipseListModel>, top_left: &QModelIndex, bottom_right: &QModelIndex, roles: &QVector_i32);
    }

    impl cxx_qt::Initialize for EclipseListModel {}
}

#[derive(Default)]
pub struct EclipseListModelRust {
    rows: QString,
    count: i32,
    raw: Vec<Value>,
    kinds: Vec<QVariant>,
    items: Vec<QVariant>,
}

impl cxx_qt::Initialize for qobject::EclipseListModel {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut().on_rows_changed(|m| m.sync_rows()).release();
    }
}

fn kind_key(v: &Value) -> Result<i32, &str> {
    match v.get("kind") {
        Some(Value::Number(n)) => Ok(n.as_i64().unwrap_or(0) as i32),
        Some(Value::String(s)) => Err(s.as_str()),
        _ => Ok(0),
    }
}

fn kind_of(v: &Value) -> QVariant {
    match kind_key(v) {
        Ok(n) => QVariant::from(&n),
        Err(s) => QVariant::from(&QString::from(s)),
    }
}

#[derive(Debug, PartialEq, Eq)]
enum SyncPlan {
    Same,
    DataChanged { first: usize, last: usize },
    Reset,
}

fn plan_sync(old: &[Value], next: &[Value]) -> SyncPlan {
    if old == next {
        return SyncPlan::Same;
    }
    if old.len() == next.len() && !next.is_empty() {
        let mut first_diff = None;
        let mut last_diff = 0usize;
        for (i, next_val) in next.iter().enumerate() {
            if old[i] != *next_val {
                if kind_key(&old[i]) != kind_key(next_val) {
                    return SyncPlan::Reset;
                }
                if first_diff.is_none() {
                    first_diff = Some(i);
                }
                last_diff = i;
            }
        }
        return match first_diff {
            Some(first) => SyncPlan::DataChanged { first, last: last_diff },
            None => SyncPlan::Same,
        };
    }
    SyncPlan::Reset
}

fn value_to_variant(v: &Value) -> QVariant {
    json_to_variant(&v.to_string())
}

impl qobject::EclipseListModel {
    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let row = index.row() as usize;
        match role {
            ROLE_ITEM => self.items.get(row).cloned().unwrap_or_default(),
            ROLE_KIND => self.kinds.get(row).cloned().unwrap_or_default(),
            _ => QVariant::default(),
        }
    }

    pub fn role_names(&self) -> QHash<QHashPair_i32_QByteArray> {
        let mut roles = QHash::<QHashPair_i32_QByteArray>::default();
        roles.insert(ROLE_ITEM, QByteArray::from("item"));
        roles.insert(ROLE_KIND, QByteArray::from("kind"));
        roles
    }

    pub fn row_count(&self, parent: &QModelIndex) -> i32 {
        if parent.is_valid() { 0 } else { self.items.len() as i32 }
    }

    fn sync_rows(mut self: Pin<&mut Self>) {
        let parsed: Vec<Value> = match serde_json::from_str::<Value>(&self.rows.to_string()) {
            Ok(Value::Array(items)) => items,
            _ => Vec::new(),
        };
        match plan_sync(&self.raw, &parsed) {
            SyncPlan::Same => {}
            SyncPlan::DataChanged { first, last } => {
                let mut r = self.as_mut().rust_mut();
                for i in first..=last {
                    if r.raw[i] != parsed[i] {
                        r.kinds[i] = kind_of(&parsed[i]);
                        r.items[i] = value_to_variant(&parsed[i]);
                        r.raw[i] = parsed[i].clone();
                    }
                }
                let root = QModelIndex::default();
                let (top_left, bottom_right) = (self.index(first as i32, 0, &root), self.index(last as i32, 0, &root));
                self.as_mut().data_changed(&top_left, &bottom_right, &QVector::<i32>::default());
            }
            SyncPlan::Reset => {
                let kinds: Vec<QVariant> = parsed.iter().map(kind_of).collect();
                let items: Vec<QVariant> = parsed.iter().map(value_to_variant).collect();
                let next_count = items.len() as i32;
                let count_changed = self.count != next_count;
                self.as_mut().begin_reset_model();
                {
                    let mut r = self.as_mut().rust_mut();
                    r.raw = parsed;
                    r.kinds = kinds;
                    r.items = items;
                    r.count = next_count;
                }
                self.as_mut().end_reset_model();
                if count_changed {
                    self.count_changed();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn equal_length_kind_change_requires_reset_and_same_kind_uses_data_changed() {
        let old = vec![json!({"kind": 0, "text": "a"}), json!({"kind": 1, "text": "b"})];
        let same_kind = vec![json!({"kind": 0, "text": "a2"}), json!({"kind": 1, "text": "b"})];
        assert_eq!(plan_sync(&old, &same_kind), SyncPlan::DataChanged { first: 0, last: 0 });

        let changed_kind = vec![json!({"kind": 1, "text": "a"}), json!({"kind": 1, "text": "b"})];
        assert_eq!(plan_sync(&old, &changed_kind), SyncPlan::Reset);

        let str_to_num = vec![json!({"kind": "chip", "text": "a"}), json!({"kind": 1, "text": "b"})];
        assert_eq!(plan_sync(&old, &str_to_num), SyncPlan::Reset);
    }
}
