//! `Collections`, the read-only game collections projection.

use crate::rows::qobject::ThemeRows;
use crate::view::Row;
use core::pin::Pin;
use cxx_qt::casting::Upcast;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("emusen-theme-qt/src/rows.cxxqt.h");
        type ThemeRows = crate::rows::qobject::ThemeRows;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element = "Collections"]
        #[qml_singleton]
        #[qproperty(i32, focused, READ, NOTIFY = changed)]
        #[qproperty(i32, total_count, cxx_name = "totalCount", READ, NOTIFY = changed)]
        #[qproperty(QString, footer_right, cxx_name = "footerRight", READ, NOTIFY = changed)]
        #[qproperty(*mut ThemeRows, items, READ, CONSTANT)]
        type ThemeCollections = super::ThemeCollectionsRust;
    }

    unsafe extern "RustQt" {
        #[qsignal]
        fn changed(self: Pin<&mut ThemeCollections>);
    }

    impl cxx_qt::Initialize for ThemeCollections {}
}

pub const COLLECTION_ITEM_ROLES: [&str; 5] = ["id", "name", "count", "kind", "subtitle"];

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CollectionsFacts {
    pub focused: i32,
    pub total_count: i32,
    pub footer_right: String,
    pub items: Vec<Row>,
}

pub struct ThemeCollectionsRust {
    focused: i32,
    total_count: i32,
    footer_right: QString,
    items: *mut ThemeRows,
}

impl Default for ThemeCollectionsRust {
    fn default() -> Self {
        Self {
            focused: 0,
            total_count: 0,
            footer_right: QString::default(),
            items: std::ptr::null_mut(),
        }
    }
}

impl cxx_qt::Initialize for qobject::ThemeCollections {
    fn initialize(mut self: Pin<&mut Self>) {
        let list = crate::glue::ffi::new_rows(self.as_mut().upcast_pin());
        if let Some(rows) = unsafe { list.as_mut() } {
            unsafe { Pin::new_unchecked(rows) }.set_roles(&COLLECTION_ITEM_ROLES);
        }
        self.as_mut().rust_mut().items = list;
    }
}

impl qobject::ThemeCollections {
    pub fn show(mut self: Pin<&mut Self>, facts: &CollectionsFacts) {
        let mut changed = false;
        if self.focused != facts.focused {
            self.as_mut().rust_mut().focused = facts.focused;
            changed = true;
        }
        if self.total_count != facts.total_count {
            self.as_mut().rust_mut().total_count = facts.total_count;
            changed = true;
        }
        let footer = QString::from(facts.footer_right.as_str());
        if self.footer_right != footer {
            self.as_mut().rust_mut().footer_right = footer;
            changed = true;
        }
        if let Some(list) = unsafe { self.items.as_mut() } {
            if list.rows() != facts.items.as_slice() {
                changed = true;
            }
            unsafe { Pin::new_unchecked(list) }.replace(facts.items.clone());
        }
        if changed {
            self.changed();
        }
    }
}

pub fn facts_of(snap: &emusen_qt_models::collections_eclipse::CollectionsSnapshot) -> CollectionsFacts {
    use crate::view::Cell;
    CollectionsFacts {
        focused: snap.focused,
        total_count: snap.total_count,
        footer_right: snap.footer_right.clone(),
        items: snap
            .items
            .iter()
            .map(|item| {
                vec![
                    Cell::Text(item.id.clone()),
                    Cell::Text(item.name.clone()),
                    Cell::Whole(item.count),
                    Cell::Text(item.kind.clone()),
                    Cell::Text(item.subtitle.clone()),
                ]
            })
            .collect(),
    }
}
