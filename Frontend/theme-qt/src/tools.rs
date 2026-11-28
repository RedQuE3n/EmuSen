//! `Tools`, the read-only Tools suite directory projection.

use {
    crate::rows::qobject::ThemeRows,
    crate::view::Row,
    core::pin::Pin,
    cxx_qt::{casting::Upcast, CxxQtType},
    cxx_qt_lib::QString,
};

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
        #[qml_element = "Tools"]
        #[qml_singleton]
        #[qproperty(i32, count, READ, NOTIFY = changed)]
        #[qproperty(i32, focused, READ, NOTIFY = changed)]
        #[qproperty(QString, active_key, cxx_name = "activeKey", READ, NOTIFY = changed)]
        #[qproperty(bool, busy, READ, NOTIFY = changed)]
        #[qproperty(QString, summary, READ, NOTIFY = changed)]
        #[qproperty(*mut ThemeRows, entries, READ, CONSTANT)]
        type ThemeTools = super::ThemeToolsRust;
    }

    unsafe extern "RustQt" {
        #[qsignal]
        fn changed(self: Pin<&mut ThemeTools>);
    }

    impl cxx_qt::Initialize for ThemeTools {}
}

pub const TOOL_ENTRY_ROLES: [&str; 8] = ["index", "key", "name", "line", "badge", "icon", "attention", "running"];

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ToolsFacts {
    pub count: i32,
    pub focused: i32,
    pub active_key: String,
    pub busy: bool,
    pub summary: String,
    pub entries: Vec<Row>,
}

pub struct ThemeToolsRust {
    count: i32,
    focused: i32,
    active_key: QString,
    busy: bool,
    summary: QString,
    entries: *mut ThemeRows,
}

impl Default for ThemeToolsRust {
    fn default() -> Self {
        Self {
            count: 0,
            focused: 0,
            active_key: QString::default(),
            busy: false,
            summary: QString::default(),
            entries: std::ptr::null_mut(),
        }
    }
}

impl cxx_qt::Initialize for qobject::ThemeTools {
    fn initialize(mut self: Pin<&mut Self>) {
        let list = crate::glue::ffi::new_rows(self.as_mut().upcast_pin());
        if let Some(rows) = unsafe { list.as_mut() } {
            unsafe { Pin::new_unchecked(rows) }.set_roles(&TOOL_ENTRY_ROLES);
        }
        self.as_mut().rust_mut().entries = list;
    }
}

impl qobject::ThemeTools {
    pub fn show(mut self: Pin<&mut Self>, facts: &ToolsFacts) {
        let mut changed = false;
        if self.count != facts.count {
            self.as_mut().rust_mut().count = facts.count;
            changed = true;
        }
        if self.focused != facts.focused {
            self.as_mut().rust_mut().focused = facts.focused;
            changed = true;
        }
        let active_key = QString::from(facts.active_key.as_str());
        if self.active_key != active_key {
            self.as_mut().rust_mut().active_key = active_key;
            changed = true;
        }
        if self.busy != facts.busy {
            self.as_mut().rust_mut().busy = facts.busy;
            changed = true;
        }
        let summary = QString::from(facts.summary.as_str());
        if self.summary != summary {
            self.as_mut().rust_mut().summary = summary;
            changed = true;
        }
        if let Some(list) = unsafe { self.entries.as_mut() } {
            if list.rows() != facts.entries.as_slice() {
                changed = true;
            }
            unsafe { Pin::new_unchecked(list) }.replace(facts.entries.clone());
        }
        if changed {
            self.changed();
        }
    }
}

pub fn facts_of(snap: &emusen_qt_models::tools_hub::ToolsSnapshot) -> ToolsFacts {
    use crate::view::Cell;
    ToolsFacts {
        count: snap.count,
        focused: snap.focused,
        active_key: snap.active_key.clone(),
        busy: snap.busy,
        summary: snap.summary.clone(),
        entries: snap
            .entries
            .iter()
            .map(|e| {
                vec![
                    Cell::Whole(e.index),
                    Cell::Text(e.key.clone()),
                    Cell::Text(e.name.clone()),
                    Cell::Text(e.line.clone()),
                    Cell::Text(e.badge.clone()),
                    Cell::Text(e.icon.clone()),
                    Cell::Flag(e.attention),
                    Cell::Flag(e.running),
                ]
            })
            .collect(),
    }
}
