//! `Settings`, the Settings screen as a theme reads it, read-only (EmuSen_Qt.md §3.2).

use crate::glue::ffi;
use crate::help::kind_of;
use crate::rows::qobject::ThemeRows;
use crate::view::{Cell, Row};
use core::pin::Pin;
use cxx_qt::casting::Upcast;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QList, QString, QStringList, QUrl};
use emusen_kit_core::settings::{Row as WalkRow, RowKind, SliderRange};
use emusen_qt_models::settings::{Pane, SettingsView};
use emusen_qt_models::settings_shown::Form;
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
        type QList_f64 = cxx_qt_lib::QList<f64>;
        include!("emusen-theme-qt/src/rows.cxxqt.h");
        type ThemeRows = crate::rows::qobject::ThemeRows;
    }

    /// From 1.11: how the boards draw a row's value, `Settings.rows`' `form` (`settings_shown::Form`).
    #[qenum(ThemeSettings)]
    enum SettingsForm {
        Choice,
        Switch,
        Door,
        Fact,
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
        #[qproperty(QList_f64, sample_scales, cxx_name = "sampleScales", READ, NOTIFY = shown)]
        /// From 1.11 (R-20261123-30): each section's key, in `sections`' order, by which a theme picks a section's picture.
        #[qproperty(QStringList, section_keys, cxx_name = "sectionKeys", READ, NOTIFY = shown)]
        /// From 1.11: the screen's title, the page's heading and the About panel's (the focused row's name), the newest screenshot as a file URL for a picture preview, and whether X takes the focused row back to a default.
        /// From 1.12 (R-20261123-39): every level above the page shown, joined as the boards join them; empty on a section.
        #[qproperty(QString, trail_line, cxx_name = "trailLine", READ, NOTIFY = shown)]
        #[qproperty(QString, title, READ, NOTIFY = shown)]
        #[qproperty(QString, heading, READ, NOTIFY = shown)]
        #[qproperty(QString, about_heading, cxx_name = "aboutHeading", READ, NOTIFY = shown)]
        #[qproperty(QString, shot, READ, NOTIFY = shown)]
        #[qproperty(bool, defaultable, READ, NOTIFY = shown)]
        type ThemeSettings = super::ThemeSettingsRust;
    }

    unsafe extern "RustQt" {
        /// What the screen shows changed: one signal for every member, as the host feeds them together.
        #[qsignal]
        fn shown(self: Pin<&mut ThemeSettings>);
    }

    impl cxx_qt::Initialize for ThemeSettings {}
}

/// `Settings.rows`' roles (EmuSen_Qt.md §3.2, `Settings`).
pub const SETTINGS_ROW_ROLES: [&str; 19] = [
    "key", "label", "about", "value", "kind", "isSwitch", "switchOn", "held", "marked", "code", "minimum", "maximum", "step", "amount", "consoleName",
    "system", "games", "engine", "form",
];

/// Everything the singleton shows, as the host reads it from the program's model.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SettingsFacts {
    pub sections: Vec<String>,
    pub section_keys: Vec<String>,
    pub title: String,
    pub heading: String,
    pub about_heading: String,
    pub defaultable: bool,
    pub section: i32,
    pub trail: Vec<String>,
    pub trail_line: String,
    pub page: String,
    pub rows: Vec<Row>,
    pub focused: i32,
    /// The help bar's kind for what is shown (`Help.entries`' kind), -1 where none is the screen's (a bindings page, listening).
    pub help: i32,
    pub pane: Pane,
}

/// The facts of a view whose rows were last told as `rows`; an error where those are not the view's rows.
pub fn facts_of(view: &SettingsView, rows: &[settings::Row], focused: i32) -> Result<SettingsFacts, String> {
    let facets = view.facets();
    if facets.len() != rows.len() {
        return Err(format!("{} rows told and {} in the view", rows.len(), facets.len()));
    }
    let (range, unset) = (SliderRange::default(), WalkRow::default());
    let cells = rows
        .iter()
        .zip(facets)
        .enumerate()
        .map(|(i, (r, facet))| {
            let slider = if r.kind == RowKind::Slider { view.slider(i) } else { None };
            let [minimum, maximum, step, amount] = slider.unwrap_or([range.minimum, range.maximum, range.step, unset.amount]);
            let tile = facet.tile.unwrap_or_default();
            vec![
                Cell::Text(r.id.clone()),
                Cell::Text(r.label.clone()),
                Cell::Text(facet.about),
                Cell::Text(r.value.clone()),
                Cell::Whole(r.kind.index() as i32),
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
                Cell::Whole(form_of(Form::of(r, facet.switch)).repr),
            ]
        })
        .collect();
    Ok(SettingsFacts {
        sections: view.sections(),
        section_keys: view.section_keys(),
        title: view.title(),
        heading: view.heading(),
        about_heading: usize::try_from(focused).ok().and_then(|f| rows.get(f)).map(|r| r.label.clone()).unwrap_or_default(),
        defaultable: usize::try_from(focused).is_ok_and(|f| view.defaultable(f)),
        section: view.section as i32,
        trail: view.trail(),
        trail_line: view.trail_line(),
        page: view.page_key(),
        rows: cells,
        focused,
        help: kind_of(view.help()).map_or(-1, |k| k.repr),
        pane: view.pane(focused.max(0) as usize),
    })
}

/// A form as the theme's enum gives it.
fn form_of(form: Form) -> qobject::SettingsForm {
    match form {
        Form::Choice => qobject::SettingsForm::Choice,
        Form::Switch => qobject::SettingsForm::Switch,
        Form::Door => qobject::SettingsForm::Door,
        Form::Fact => qobject::SettingsForm::Fact,
    }
}

fn list(words: &[String]) -> QStringList {
    let mut out = QList::<QString>::default();
    words.iter().for_each(|w| out.append(QString::from(w.as_str())));
    QStringList::from(&out)
}

pub struct ThemeSettingsRust {
    sections: QStringList,
    section_keys: QStringList,
    title: QString,
    heading: QString,
    about_heading: QString,
    shot: QString,
    defaultable: bool,
    section: i32,
    trail: QStringList,
    trail_line: QString,
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
    sample_scales: QList<f64>,
    last: SettingsFacts,
}

impl Default for ThemeSettingsRust {
    fn default() -> Self {
        ThemeSettingsRust {
            sections: QStringList::default(),
            section_keys: QStringList::default(),
            title: QString::default(),
            heading: QString::default(),
            about_heading: QString::default(),
            shot: QString::default(),
            defaultable: false,
            section: 0,
            trail: QStringList::default(),
            trail_line: QString::default(),
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
            sample_scales: QList::default(),
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
        me.section_keys = list(&facts.section_keys);
        me.title = QString::from(facts.title.as_str());
        me.heading = QString::from(facts.heading.as_str());
        me.about_heading = QString::from(facts.about_heading.as_str());
        me.shot = if pane.shot.is_empty() { QString::default() } else { QUrl::from_local_file(&QString::from(pane.shot.as_str())).to_qstring() };
        me.defaultable = facts.defaultable;
        me.section = facts.section;
        me.trail = list(&facts.trail);
        me.trail_line = QString::from(facts.trail_line.as_str());
        me.page = QString::from(facts.page.as_str());
        me.focused = facts.focused;
        me.count = facts.rows.len() as i32;
        me.help = facts.help;
        me.sentence = QString::from(pane.sentence.as_str());
        me.lines = list(&pane.lines);
        me.muted = list(&pane.muted);
        me.tones = tones;
        me.preview = pane.preview.number();
        me.caption = QString::from(pane.caption.as_str());
        me.meter = pane.meter;
        me.samples = list(&pane.samples.iter().map(|(s, _)| s.clone()).collect::<Vec<_>>());
        let mut scales = QList::<f64>::default();
        pane.samples.iter().for_each(|(_, scale)| scales.append(*scale));
        me.sample_scales = scales;
        me.last = facts.clone();
        drop(me);
        self.shown();
    }

    /// The facts last shown, for the host's tests.
    pub fn shown_facts(&self) -> &SettingsFacts {
        &self.last
    }
}
