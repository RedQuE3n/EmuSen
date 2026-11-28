//! `Themes`, the read-only installed themes catalog and active selection projection.

use crate::rows::qobject::ThemeRows;
use crate::view::Row;
use core::pin::Pin;
use cxx_qt::casting::Upcast;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QList, QString, QStringList};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
        include!("emusen-theme-qt/src/rows.cxxqt.h");
        type ThemeRows = crate::rows::qobject::ThemeRows;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element = "Themes"]
        #[qml_singleton]
        #[qproperty(QStringList, kinds, READ, NOTIFY = changed)]
        #[qproperty(QString, kind, READ, NOTIFY = changed)]
        #[qproperty(i32, focused, READ, NOTIFY = changed)]
        #[qproperty(QString, chosen, READ, NOTIFY = changed)]
        #[qproperty(QString, words, READ, NOTIFY = changed)]
        #[qproperty(QString, name, READ, NOTIFY = changed)]
        #[qproperty(QString, line, READ, NOTIFY = changed)]
        #[qproperty(QString, preview, READ, NOTIFY = changed)]
        #[qproperty(QString, findings, READ, NOTIFY = changed)]
        #[qproperty(bool, usable, READ, NOTIFY = changed)]
        #[qproperty(i32, help, READ, NOTIFY = changed)]
        #[qproperty(*mut ThemeRows, rows, READ, CONSTANT)]
        type ThemeThemes = super::ThemeThemesRust;
    }

    unsafe extern "RustQt" {
        #[qsignal]
        fn changed(self: Pin<&mut ThemeThemes>);
    }

    impl cxx_qt::Initialize for ThemeThemes {}
}

pub const THEME_ROW_ROLES: [&str; 11] = ["id", "label", "value", "kind", "steps", "active", "usable", "dualFace", "badge", "about", "preview"];

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ThemesFacts {
    pub kinds: Vec<String>,
    pub kind: String,
    pub focused: i32,
    pub chosen: String,
    pub words: String,
    pub name: String,
    pub line: String,
    pub preview: String,
    pub findings: String,
    pub usable: bool,
    pub help: i32,
    pub rows: Vec<Row>,
}

pub struct ThemeThemesRust {
    kinds: QStringList,
    kind: QString,
    focused: i32,
    chosen: QString,
    words: QString,
    name: QString,
    line: QString,
    preview: QString,
    findings: QString,
    usable: bool,
    help: i32,
    rows: *mut ThemeRows,
}

impl Default for ThemeThemesRust {
    fn default() -> Self {
        Self {
            kinds: QStringList::default(),
            kind: QString::default(),
            focused: 0,
            chosen: QString::default(),
            words: QString::default(),
            name: QString::default(),
            line: QString::default(),
            preview: QString::default(),
            findings: QString::default(),
            usable: false,
            help: 0,
            rows: std::ptr::null_mut(),
        }
    }
}

impl cxx_qt::Initialize for qobject::ThemeThemes {
    fn initialize(mut self: Pin<&mut Self>) {
        let list = crate::glue::ffi::new_rows(self.as_mut().upcast_pin());
        if let Some(rows) = unsafe { list.as_mut() } {
            unsafe { Pin::new_unchecked(rows) }.set_roles(&THEME_ROW_ROLES);
        }
        self.as_mut().rust_mut().rows = list;
    }
}

fn to_qstring_list(items: &[String]) -> QStringList {
    let mut list = QList::<QString>::default();
    for item in items {
        list.append(QString::from(item.as_str()));
    }
    QStringList::from(&list)
}

impl qobject::ThemeThemes {
    pub fn show(mut self: Pin<&mut Self>, facts: &ThemesFacts) {
        let mut changed = false;
        let kinds = to_qstring_list(&facts.kinds);
        if self.kinds != kinds {
            self.as_mut().rust_mut().kinds = kinds;
            changed = true;
        }
        macro_rules! set_text {
            ($field:ident, $val:expr) => {{
                let v = QString::from($val.as_str());
                if self.$field != v {
                    self.as_mut().rust_mut().$field = v;
                    changed = true;
                }
            }};
        }
        macro_rules! set_plain {
            ($field:ident, $val:expr) => {{
                let v = $val;
                if self.$field != v {
                    self.as_mut().rust_mut().$field = v;
                    changed = true;
                }
            }};
        }
        set_text!(kind, facts.kind);
        set_plain!(focused, facts.focused);
        set_text!(chosen, facts.chosen);
        set_text!(words, facts.words);
        set_text!(name, facts.name);
        set_text!(line, facts.line);
        set_text!(preview, facts.preview);
        set_text!(findings, facts.findings);
        set_plain!(usable, facts.usable);
        set_plain!(help, facts.help);
        if let Some(list) = unsafe { self.rows.as_mut() } {
            if list.rows() != facts.rows.as_slice() {
                changed = true;
            }
            unsafe { Pin::new_unchecked(list) }.replace(facts.rows.clone());
        }
        if changed {
            self.changed();
        }
    }
}

pub fn facts_of(snap: &emusen_qt_models::themes_model::ThemesSnapshot, url_map: &dyn Fn(String) -> String) -> ThemesFacts {
    use crate::view::Cell;
    let map_url = |u: &String| if u.is_empty() { String::new() } else { url_map(u.clone()) };
    ThemesFacts {
        kinds: snap.kinds.clone(),
        kind: snap.kind.clone(),
        focused: snap.focused,
        chosen: snap.chosen.clone(),
        words: snap.words.clone(),
        name: snap.name.clone(),
        line: snap.line.clone(),
        preview: map_url(&snap.preview),
        findings: snap.findings.clone(),
        usable: snap.usable,
        help: snap.help,
        rows: snap
            .rows
            .iter()
            .map(|r| {
                vec![
                    Cell::Text(r.id.clone()),
                    Cell::Text(r.label.clone()),
                    Cell::Text(r.value.clone()),
                    Cell::Text(r.kind.clone()),
                    Cell::Flag(r.steps),
                    Cell::Flag(r.active),
                    Cell::Flag(r.usable),
                    Cell::Flag(r.dual_face),
                    Cell::Text(r.badge.clone()),
                    Cell::Text(r.about.clone()),
                    Cell::Text(map_url(&r.preview)),
                ]
            })
            .collect(),
    }
}
