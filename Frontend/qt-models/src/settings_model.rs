//! `SettingsModel`, the Settings screen as a QAbstractListModel (EmuSen_Qt.md §7.7): settings.rs's rows of the section
//! or page shown, with the roles the kit's settings frame reads, and the focused row's pane as properties. A read runs
//! off the GUI thread and comes back through qt_thread().queue; a change is saved through Galaxia's model at once.

use crate::help::Screen;
use crate::help_model::shown_screen;
use crate::settings::{self, Outcome, Preview, SettingsView};
use crate::settings_source::ClientSettings;
use crate::settings::SettingsSource;
use core::pin::Pin;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QByteArray, QHash, QHashPair_i32_QByteArray, QList, QModelIndex, QString, QStringList, QVariant, QVector};

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
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
        include!("cxx-qt-lib/qvector.h");
        type QVector_i32 = cxx_qt_lib::QVector<i32>;
        include!("cxx-qt-lib/qlist.h");
        type QList_f64 = cxx_qt_lib::QList<f64>;
    }

    extern "RustQt" {
        #[qobject]
        #[base = QAbstractListModel]
        #[qml_element]
        /// The platform library, through which the settings are read and saved.
        #[qproperty(QString, platform_library, cxx_name = "platformLibrary")]
        /// Whether Settings is shown: the settings are read each time it opens.
        #[qproperty(bool, open)]
        /// The program's facts a pane names, as LibraryModel.settingsFacts writes them.
        #[qproperty(QString, facts)]
        #[qproperty(QStringList, sections, READ, CONSTANT)]
        /// The section shown, which the strip's shoulders and clicks choose.
        #[qproperty(i32, section)]
        /// The pages open under the section, the section's name first; empty on the section.
        #[qproperty(QStringList, trail, READ, NOTIFY)]
        /// The focused row, which the frame's walk sets; the pane is its.
        #[qproperty(i32, focused)]
        #[qproperty(QStringList, pane_lines, cxx_name = "paneLines", READ, NOTIFY)]
        #[qproperty(QStringList, pane_muted, cxx_name = "paneMuted", READ, NOTIFY)]
        /// The pane's preview: 0 none, 1 the game's picture, 2 a meter, 3 the two variants, 4 the text sizes.
        #[qproperty(i32, preview, READ, NOTIFY)]
        #[qproperty(QString, caption, READ, NOTIFY)]
        #[qproperty(f64, meter, READ, NOTIFY)]
        /// The text sizes' lines, or the two variants' names, and each line's scale.
        #[qproperty(QStringList, samples, READ, NOTIFY)]
        #[qproperty(QList_f64, sample_scales, cxx_name = "sampleScales", READ, NOTIFY)]
        /// What the settings ask of the whole program, once read: the type scale, the light variant, reduced motion.
        #[qproperty(bool, ready, READ, NOTIFY)]
        #[qproperty(f64, text_scale, cxx_name = "textScale", READ, NOTIFY)]
        #[qproperty(bool, light, READ, NOTIFY)]
        #[qproperty(bool, reduced_motion, cxx_name = "reducedMotion", READ, NOTIFY)]
        /// The help bar's screen (HelpModel's numbers).
        #[qproperty(i32, help, READ, NOTIFY)]
        /// Why a read or a save failed, in the platform's words; empty when it did not.
        #[qproperty(QString, problem, READ, NOTIFY)]
        type SettingsModel = super::SettingsModelRust;
    }

    extern "RustQt" {
        #[qinvokable]
        #[cxx_override]
        fn data(self: &SettingsModel, index: &QModelIndex, role: i32) -> QVariant;

        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &SettingsModel) -> QHash_i32_QByteArray;

        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &SettingsModel, parent: &QModelIndex) -> i32;

        /// The settings read again.
        #[qinvokable]
        fn refresh(self: Pin<&mut SettingsModel>);

        /// Left (-1) or Right (+1) on a row.
        #[qinvokable]
        fn step(self: Pin<&mut SettingsModel>, row: i32, by: i32);

        /// A on a row.
        #[qinvokable]
        fn choose(self: Pin<&mut SettingsModel>, row: i32);

        /// B: the page closed, or `left` raised on a section.
        #[qinvokable]
        fn back(self: Pin<&mut SettingsModel>);

        /// A door to another screen chosen, by its name in the description.
        #[qsignal]
        fn door(self: Pin<&mut SettingsModel>, name: QString);

        /// B on a section: Settings is left.
        #[qsignal]
        fn left(self: Pin<&mut SettingsModel>);
    }

    unsafe extern "RustQt" {
        #[inherit]
        fn index(self: &SettingsModel, row: i32, column: i32, parent: &QModelIndex) -> QModelIndex;

        #[inherit]
        #[cxx_name = "beginRemoveRows"]
        fn begin_remove_rows(self: Pin<&mut SettingsModel>, parent: &QModelIndex, first: i32, last: i32);

        #[inherit]
        #[cxx_name = "endRemoveRows"]
        fn end_remove_rows(self: Pin<&mut SettingsModel>);

        #[inherit]
        #[cxx_name = "beginInsertRows"]
        fn begin_insert_rows(self: Pin<&mut SettingsModel>, parent: &QModelIndex, first: i32, last: i32);

        #[inherit]
        #[cxx_name = "endInsertRows"]
        fn end_insert_rows(self: Pin<&mut SettingsModel>);

        #[inherit]
        #[qsignal]
        #[cxx_name = "dataChanged"]
        fn data_changed(self: Pin<&mut SettingsModel>, top_left: &QModelIndex, bottom_right: &QModelIndex, roles: &QVector_i32);
    }

    impl cxx_qt::Threading for SettingsModel {}
    impl cxx_qt::Initialize for SettingsModel {}
}

pub struct SettingsModelRust {
    platform_library: QString,
    open: bool,
    facts: QString,
    sections: QStringList,
    section: i32,
    trail: QStringList,
    focused: i32,
    pane_lines: QStringList,
    pane_muted: QStringList,
    preview: i32,
    caption: QString,
    meter: f64,
    samples: QStringList,
    sample_scales: QList<f64>,
    ready: bool,
    text_scale: f64,
    light: bool,
    reduced_motion: bool,
    help: i32,
    problem: QString,
    view: SettingsView,
    rows: Vec<settings::Row>,
    /// The reads asked for, so an older read that finishes late is dropped.
    asked: u64,
}

fn list(words: impl Iterator<Item = String>) -> QStringList {
    let mut out = QList::<QString>::default();
    for w in words {
        out.append(QString::from(w.as_str()));
    }
    QStringList::from(&out)
}

impl Default for SettingsModelRust {
    fn default() -> Self {
        let view = SettingsView::new();
        SettingsModelRust {
            platform_library: QString::default(),
            open: false,
            facts: QString::default(),
            sections: list(view.sections().into_iter()),
            section: 0,
            trail: QStringList::default(),
            focused: 0,
            pane_lines: QStringList::default(),
            pane_muted: QStringList::default(),
            preview: 0,
            caption: QString::default(),
            meter: 0.0,
            samples: QStringList::default(),
            sample_scales: QList::default(),
            ready: false,
            text_scale: 1.0,
            light: false,
            reduced_motion: false,
            help: shown_screen(Screen::Settings),
            problem: QString::default(),
            rows: view.rows(),
            view,
            asked: 0,
        }
    }
}

impl cxx_qt::Initialize for qobject::SettingsModel {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut()
            .on_open_changed(|model| {
                if model.open {
                    model.read_if_named();
                }
            })
            .release();
        self.as_mut().on_platform_library_changed(|model| model.read_if_named()).release();
        self.as_mut()
            .on_facts_changed(|mut model| {
                let facts = serde_json::from_str(&model.facts.to_string()).unwrap_or(serde_json::Value::Null);
                model.as_mut().rust_mut().view.facts = facts;
                model.shown(false);
            })
            .release();
        self.as_mut()
            .on_section_changed(|mut model| {
                let section = model.section.max(0) as usize;
                model.as_mut().rust_mut().view.choose_section(section);
                model.shown(true);
            })
            .release();
        self.on_focused_changed(|model| model.pane()).release();
    }
}

const LABEL: i32 = 0;
const VALUE: i32 = 1;
const SENTENCE: i32 = 2;
const KIND: i32 = 3;
const HELD: i32 = 4;
const MARKED: i32 = 5;
const KEY: i32 = 6;

impl qobject::SettingsModel {
    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let Some(row) = self.rows.get(index.row() as usize) else { return QVariant::default() };
        let text = |s: &str| QVariant::from(&QString::from(s));
        match role {
            LABEL => text(&row.label),
            VALUE => text(&row.value),
            SENTENCE => text(&self.view.pane(index.row() as usize).sentence),
            KIND => QVariant::from(&(emusen_kit_core::settings::RowKind::ALL.iter().position(|k| *k == row.kind).unwrap_or(0) as i32)),
            HELD => QVariant::from(&row.held),
            MARKED => QVariant::from(&false),
            KEY => text(&row.id),
            _ => QVariant::default(),
        }
    }

    pub fn role_names(&self) -> QHash<QHashPair_i32_QByteArray> {
        let mut roles = QHash::<QHashPair_i32_QByteArray>::default();
        for (role, name) in [(LABEL, "label"), (VALUE, "value"), (SENTENCE, "sentence"), (KIND, "kind"), (HELD, "held"), (MARKED, "marked"), (KEY, "key")] {
            roles.insert(role, QByteArray::from(name));
        }
        roles
    }

    pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
        self.rows.len() as i32
    }

    pub fn refresh(mut self: Pin<&mut Self>) {
        let library = self.platform_library.to_string();
        if library.is_empty() {
            return;
        }
        let asked = self.asked.wrapping_add(1);
        self.as_mut().rust_mut().asked = asked;
        let thread = self.qt_thread();
        std::thread::spawn(move || {
            let read = ClientSettings::open(&library).and_then(|source| settings::read(&source));
            let _ = thread.queue(move |mut model| {
                if model.asked != asked {
                    return;
                }
                match read {
                    Ok(read) => {
                        model.as_mut().rust_mut().view.read = read;
                        model.as_mut().said(String::new());
                    }
                    Err(words) => model.as_mut().said(words),
                }
                model.as_mut().applied();
                model.shown(false);
            });
        });
    }

    pub fn step(mut self: Pin<&mut Self>, row: i32, by: i32) {
        let outcome = self.as_mut().rust_mut().view.step(row.max(0) as usize, by);
        self.outcome(outcome);
    }

    pub fn choose(mut self: Pin<&mut Self>, row: i32) {
        let outcome = self.as_mut().rust_mut().view.choose(row.max(0) as usize);
        self.outcome(outcome);
    }

    pub fn back(mut self: Pin<&mut Self>) {
        if self.as_mut().rust_mut().view.back() {
            self.shown(true);
        } else {
            self.left();
        }
    }

    fn read_if_named(self: Pin<&mut Self>) {
        if !self.platform_library.is_empty() {
            self.refresh();
        }
    }

    fn outcome(mut self: Pin<&mut Self>, outcome: Outcome) {
        match outcome {
            Outcome::Nothing => {}
            Outcome::Saved(model, document) => {
                let saved = ClientSettings::open(&self.platform_library.to_string()).and_then(|s| s.save(model, &document));
                self.as_mut().said(saved.err().unwrap_or_default());
                self.as_mut().applied();
                self.shown(false);
            }
            Outcome::Opened => self.shown(true),
            Outcome::Door(name) => self.door(QString::from(name.as_str())),
        }
    }

    fn said(mut self: Pin<&mut Self>, words: String) {
        let words = QString::from(words.as_str());
        if self.problem != words {
            self.as_mut().rust_mut().problem = words;
            self.problem_changed();
        }
    }

    /// What the settings ask of the whole program, from the values read or changed.
    fn applied(mut self: Pin<&mut Self>) {
        let (scale, light, reduced) = (self.view.text_scale(), self.view.light(), self.view.reduced_motion());
        if self.text_scale != scale {
            self.as_mut().rust_mut().text_scale = scale;
            self.as_mut().text_scale_changed();
        }
        if self.light != light {
            self.as_mut().rust_mut().light = light;
            self.as_mut().light_changed();
        }
        if self.reduced_motion != reduced {
            self.as_mut().rust_mut().reduced_motion = reduced;
            self.as_mut().reduced_motion_changed();
        }
        if !self.ready {
            self.as_mut().rust_mut().ready = true;
            self.ready_changed();
        }
    }

    /// The rows read again from the view: in place where the same rows stand, so a stepped value keeps its delegate,
    /// else reset with the focus on the first row.
    fn shown(mut self: Pin<&mut Self>, turned: bool) {
        let rows = self.view.rows();
        let same = !turned && rows.len() == self.rows.len() && rows.iter().zip(&self.rows).all(|(a, b)| a.id == b.id);
        if same {
            self.as_mut().rust_mut().rows = rows;
            let count = self.rows.len() as i32;
            if count > 0 {
                let (first, last) = (self.index(0, 0, &QModelIndex::default()), self.index(count - 1, 0, &QModelIndex::default()));
                self.as_mut().data_changed(&first, &last, &QVector::<i32>::default());
            }
        } else {
            // The rows go and the new ones come as a removal and an insertion, not a reset: a reset cancels a delegate
            // the view is still making past the new count, which Qt logs as "index out range" (EmuSen_Qt.md §7.7).
            let root = QModelIndex::default();
            let (old, new) = (self.rows.len() as i32, rows.len() as i32);
            if old > 0 {
                self.as_mut().begin_remove_rows(&root, 0, old - 1);
                self.as_mut().rust_mut().rows.clear();
                self.as_mut().end_remove_rows();
            }
            if new > 0 {
                self.as_mut().begin_insert_rows(&root, 0, new - 1);
                self.as_mut().rust_mut().rows = rows;
                self.as_mut().end_insert_rows();
            }
            if self.focused != 0 {
                self.as_mut().rust_mut().focused = 0;
                self.as_mut().focused_changed();
            }
        }
        let trail = list(self.view.trail().into_iter());
        if self.trail != trail {
            self.as_mut().rust_mut().trail = trail;
            self.as_mut().trail_changed();
        }
        let help = shown_screen(if self.view.page_open() { Screen::SettingsPage } else { Screen::Settings });
        if self.help != help {
            self.as_mut().rust_mut().help = help;
            self.as_mut().help_changed();
        }
        self.pane();
    }

    /// The focused row's pane.
    fn pane(mut self: Pin<&mut Self>) {
        let pane = self.view.pane(self.focused.max(0) as usize);
        let lines = list(pane.lines.into_iter());
        if self.pane_lines != lines {
            self.as_mut().rust_mut().pane_lines = lines;
            self.as_mut().pane_lines_changed();
        }
        let muted = list(pane.muted.into_iter());
        if self.pane_muted != muted {
            self.as_mut().rust_mut().pane_muted = muted;
            self.as_mut().pane_muted_changed();
        }
        let preview = match pane.preview {
            Preview::None => 0,
            Preview::Picture => 1,
            Preview::Meter => 2,
            Preview::Variants => 3,
            Preview::Sizes => 4,
        };
        if self.preview != preview {
            self.as_mut().rust_mut().preview = preview;
            self.as_mut().preview_changed();
        }
        let caption = QString::from(pane.caption.as_str());
        if self.caption != caption {
            self.as_mut().rust_mut().caption = caption;
            self.as_mut().caption_changed();
        }
        if self.meter != pane.meter {
            self.as_mut().rust_mut().meter = pane.meter;
            self.as_mut().meter_changed();
        }
        let samples = list(pane.samples.iter().map(|(s, _)| s.clone()));
        let mut scales = QList::<f64>::default();
        for (_, scale) in &pane.samples {
            scales.append(*scale);
        }
        if self.samples != samples || self.sample_scales != scales {
            self.as_mut().rust_mut().samples = samples;
            self.as_mut().rust_mut().sample_scales = scales;
            self.as_mut().samples_changed();
            self.as_mut().sample_scales_changed();
        }
    }
}
