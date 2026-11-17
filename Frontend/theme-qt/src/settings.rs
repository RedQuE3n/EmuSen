//! `Settings`, the Settings screen as a theme reads it (EmuSen_Qt.md §3.2): the sections, the trail, the page's key, the
//! rows and the focused row's pane, a read-only view the host feeds from the program's SettingsModel. A theme changes them
//! only through `Actions`' settings calls.

use crate::glue::ffi;
use crate::help::kind_of;
use crate::rows::qobject::ThemeRows;
use crate::view::{Cell, Row};
use core::pin::Pin;
use cxx_qt::casting::Upcast;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QList, QString, QStringList};
use emusen_kit_core::settings::RowKind;
use emusen_qt_models::settings::{Pane, Preview, SettingsView};
use emusen_qt_models::settings;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
        include!("cxx-qt-lib/qlist.h");
        type QList_i32 = cxx_qt_lib::QList<i32>;
        include!("emusen-theme-qt/src/rows.cxxqt.h");
        type ThemeRows = crate::rows::qobject::ThemeRows;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element = "Settings"]
        #[qml_singleton]
        #[qproperty(QStringList, sections, READ, NOTIFY = shown)]
        #[qproperty(i32, section, READ, NOTIFY = shown)]
        #[qproperty(QStringList, trail, READ, NOTIFY = shown)]
        #[qproperty(QString, page, READ, NOTIFY = shown)]
        #[qproperty(*mut ThemeRows, rows, READ, CONSTANT)]
        #[qproperty(i32, focused, READ, NOTIFY = shown)]
        #[qproperty(i32, count, READ, NOTIFY = shown)]
        #[qproperty(i32, help, READ, NOTIFY = shown)]
        #[qproperty(QString, sentence, READ, NOTIFY = shown)]
        #[qproperty(QStringList, lines, READ, NOTIFY = shown)]
        #[qproperty(QStringList, muted, READ, NOTIFY = shown)]
        #[qproperty(QList_i32, tones, READ, NOTIFY = shown)]
        #[qproperty(i32, preview, READ, NOTIFY = shown)]
        #[qproperty(QString, caption, READ, NOTIFY = shown)]
        #[qproperty(f64, meter, READ, NOTIFY = shown)]
        #[qproperty(QStringList, samples, READ, NOTIFY = shown)]
        type ThemeSettings = super::ThemeSettingsRust;
    }

    unsafe extern "RustQt" {
        /// What the screen shows changed: one signal for every member, as the host feeds them together.
        #[qsignal]
        fn shown(self: Pin<&mut ThemeSettings>);
    }

    impl cxx_qt::Initialize for ThemeSettings {}
}

/// `Settings.rows`' roles: the row's words and kind, a switch as a state (`isSwitch`, `switchOn`), a slider's fields, and a
/// console row's tile (`console`, `system`, `games`, `engine`).
pub const SETTINGS_ROW_ROLES: [&str; 18] = [
    "key", "label", "about", "value", "kind", "isSwitch", "switchOn", "held", "marked", "code", "minimum", "maximum", "step", "amount", "consoleName",
    "system", "games", "engine",
];

/// Everything the singleton shows, as the host reads it from the program's model.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SettingsFacts {
    pub sections: Vec<String>,
    pub section: i32,
    pub trail: Vec<String>,
    pub page: String,
    pub rows: Vec<Row>,
    pub focused: i32,
    /// The help bar's kind for what is shown (`Help.entries`' kind), -1 where none is the screen's (a bindings page, listening).
    pub help: i32,
    pub pane: Pane,
}

/// The facts of a view whose rows were last told as `rows`, with the focus on `focused`.
pub fn facts_of(view: &SettingsView, rows: &[settings::Row], focused: i32) -> SettingsFacts {
    let facets = view.facets();
    let cells = rows
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let facet = facets.get(i).cloned().unwrap_or_default();
            let slider = if r.kind == RowKind::Slider { view.slider(i) } else { None };
            let [minimum, maximum, step, amount] = slider.unwrap_or([0.0, 1.0, 0.0, 0.0]);
            let tile = facet.tile.unwrap_or_default();
            vec![
                Cell::Text(r.id.clone()),
                Cell::Text(r.label.clone()),
                Cell::Text(facet.about),
                Cell::Text(r.value.clone()),
                Cell::Whole(RowKind::ALL.iter().position(|k| *k == r.kind).unwrap_or(0) as i32),
                Cell::Flag(facet.switch.is_some()),
                Cell::Flag(facet.switch.unwrap_or(false)),
                Cell::Flag(r.held),
                Cell::Flag(r.marked),
                Cell::Flag(r.code),
                Cell::Number(minimum),
                Cell::Number(maximum),
                Cell::Number(step),
                Cell::Number(amount),
                Cell::Text(tile.console),
                Cell::Text(tile.system),
                Cell::Whole(tile.games as i32),
                Cell::Text(tile.engine),
            ]
        })
        .collect();
    SettingsFacts {
        sections: view.sections(),
        section: view.section as i32,
        trail: view.trail(),
        page: view.page_key(),
        rows: cells,
        focused,
        help: kind_of(view.help()).map_or(-1, |k| k.repr),
        pane: view.pane(focused.max(0) as usize),
    }
}

/// The pane's preview by the number `Settings.preview` gives it: 0 none, 1 picture, 2 meter, 3 variants, 4 sizes, 5 heading.
fn preview_number(preview: Preview) -> i32 {
    match preview {
        Preview::None => 0,
        Preview::Picture => 1,
        Preview::Meter => 2,
        Preview::Variants => 3,
        Preview::Sizes => 4,
        Preview::Heading => 5,
    }
}

fn list(words: &[String]) -> QStringList {
    let mut out = QList::<QString>::default();
    words.iter().for_each(|w| out.append(QString::from(w.as_str())));
    QStringList::from(&out)
}

pub struct ThemeSettingsRust {
    sections: QStringList,
    section: i32,
    trail: QStringList,
    page: QString,
    rows: *mut ThemeRows,
    focused: i32,
    count: i32,
    help: i32,
    sentence: QString,
    lines: QStringList,
    muted: QStringList,
    tones: QList<i32>,
    preview: i32,
    caption: QString,
    meter: f64,
    samples: QStringList,
    last: SettingsFacts,
}

impl Default for ThemeSettingsRust {
    fn default() -> Self {
        ThemeSettingsRust {
            sections: QStringList::default(),
            section: 0,
            trail: QStringList::default(),
            page: QString::default(),
            rows: std::ptr::null_mut(),
            focused: 0,
            count: 0,
            help: -1,
            sentence: QString::default(),
            lines: QStringList::default(),
            muted: QStringList::default(),
            tones: QList::default(),
            preview: 0,
            caption: QString::default(),
            meter: 0.0,
            samples: QStringList::default(),
            last: SettingsFacts { help: -1, ..SettingsFacts::default() },
        }
    }
}

impl cxx_qt::Initialize for qobject::ThemeSettings {
    fn initialize(mut self: Pin<&mut Self>) {
        let list = ffi::new_rows(self.as_mut().upcast_pin());
        // Safety: the list was just made, a child of this singleton.
        if let Some(rows) = unsafe { list.as_mut() } {
            unsafe { Pin::new_unchecked(rows) }.set_roles(&SETTINGS_ROW_ROLES);
        }
        self.as_mut().rust_mut().rows = list;
    }
}

impl qobject::ThemeSettings {
    /// The host's feed; nothing is told where nothing changed.
    pub fn show(mut self: Pin<&mut Self>, facts: &SettingsFacts) {
        if self.last == *facts {
            return;
        }
        // Safety: the list is the singleton's child, made in `initialize`.
        if let Some(rows) = unsafe { self.rows.as_mut() } {
            unsafe { Pin::new_unchecked(rows) }.replace(facts.rows.clone());
        }
        let pane = &facts.pane;
        let mut tones = QList::<i32>::default();
        pane.tones.iter().for_each(|t| tones.append(*t as i32));
        let mut me = self.as_mut().rust_mut();
        me.sections = list(&facts.sections);
        me.section = facts.section;
        me.trail = list(&facts.trail);
        me.page = QString::from(facts.page.as_str());
        me.focused = facts.focused;
        me.count = facts.rows.len() as i32;
        me.help = facts.help;
        me.sentence = QString::from(pane.sentence.as_str());
        me.lines = list(&pane.lines);
        me.muted = list(&pane.muted);
        me.tones = tones;
        me.preview = preview_number(pane.preview);
        me.caption = QString::from(pane.caption.as_str());
        me.meter = pane.meter;
        me.samples = list(&pane.samples.iter().map(|(s, _)| s.clone()).collect::<Vec<_>>());
        me.last = facts.clone();
        drop(me);
        self.shown();
    }

    /// The facts last shown, for the host's tests.
    pub fn shown_facts(&self) -> &SettingsFacts {
        &self.last
    }
}
