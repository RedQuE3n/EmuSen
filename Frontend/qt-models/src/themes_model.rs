//! `ThemesModel`, the Themes tab as a QAbstractListModel (EmuSen_Qt.md §3.8): themes.rs's rows of the kind shown, the
//! focused row's detail as properties, Use and Check as calls. A read and a check run off the GUI thread and come back
//! through qt_thread().queue; QML writes no words and no logic.

use crate::clients::{app_settings, store_app_setting};
use crate::help::Screen;
use crate::help_model::shown_screen;
use crate::themes::{self, Checked, Kind, Listing, Row, REFERENCE, SETTING};
use core::pin::Pin;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QByteArray, QHash, QHashPair_i32_QByteArray, QList, QModelIndex, QString, QStringList, QVariant, QVector};
use emusen_platform_client::galaxia::consts::DIR_THEMES;
use emusen_platform_client::Platform;
use serde_json::Value;
use std::path::PathBuf;

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
    }

    extern "RustQt" {
        #[qobject]
        #[base = QAbstractListModel]
        #[qml_element]
        /// The platform library, through which the settings and Galaxia's themes store are read; each write reads them anew.
        #[qproperty(QString, platform_library, cxx_name = "platformLibrary")]
        /// The reference theme's `theme.json` and its folder (`ReferenceTheme`), for its row.
        #[qproperty(QString, reference)]
        #[qproperty(QString, reference_folder, cxx_name = "referenceFolder")]
        /// Whether the tab is shown: the settings and the store are read each time it opens, and not before.
        #[qproperty(bool, open)]
        /// The kind listed, one of `kinds`: the frame's section.
        #[qproperty(QString, kind)]
        #[qproperty(QStringList, kinds, READ, CONSTANT)]
        /// The focused row of the kind listed; the detail is its.
        #[qproperty(i32, focused)]
        /// The id the settings hold, and why the reference is shown in its place, if it is.
        #[qproperty(QString, chosen, READ, NOTIFY)]
        #[qproperty(QString, words, READ, NOTIFY)]
        /// The focused row's detail.
        #[qproperty(QString, name, READ, NOTIFY)]
        #[qproperty(QString, line, READ, NOTIFY)]
        #[qproperty(QString, preview, READ, NOTIFY)]
        #[qproperty(QString, findings, READ, NOTIFY)]
        #[qproperty(bool, usable, READ, NOTIFY)]
        /// The help bar's screen for the focused row (HelpModel's numbers).
        #[qproperty(i32, help, READ, NOTIFY)]
        type ThemesModel = super::ThemesModelRust;
    }

    extern "RustQt" {
        #[qinvokable]
        #[cxx_override]
        fn data(self: &ThemesModel, index: &QModelIndex, role: i32) -> QVariant;

        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &ThemesModel) -> QHash_i32_QByteArray;

        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &ThemesModel, parent: &QModelIndex) -> i32;

        /// The settings and the store read again.
        #[qinvokable]
        fn refresh(self: Pin<&mut ThemesModel>);

        /// The theme of this id chosen: its id stored, and `used` raised with the folder the host loads. Only a row Use is offered for.
        #[qinvokable]
        #[cxx_name = "use"]
        fn use_theme(self: Pin<&mut ThemesModel>, id: &QString);

        /// The theme of this id checked, off the GUI thread.
        #[qinvokable]
        fn check(self: Pin<&mut ThemesModel>, id: &QString);

        /// A declared action on the focused row: Check.
        #[qinvokable]
        fn act(self: Pin<&mut ThemesModel>, key: &QString);

        /// A on the focused row: Use, where it is offered.
        #[qinvokable]
        fn accept(self: Pin<&mut ThemesModel>);

        /// A theme was chosen: the folder the chosen theme's host loads, empty for the reference.
        #[qsignal]
        fn used(self: Pin<&mut ThemesModel>, folder: QString);
    }

    unsafe extern "RustQt" {
        #[inherit]
        #[cxx_name = "beginResetModel"]
        fn begin_reset_model(self: Pin<&mut ThemesModel>);

        #[inherit]
        #[cxx_name = "endResetModel"]
        fn end_reset_model(self: Pin<&mut ThemesModel>);

        #[inherit]
        fn index(self: &ThemesModel, row: i32, column: i32, parent: &QModelIndex) -> QModelIndex;

        #[inherit]
        #[qsignal]
        #[cxx_name = "dataChanged"]
        fn data_changed(self: Pin<&mut ThemesModel>, top_left: &QModelIndex, bottom_right: &QModelIndex, roles: &QVector_i32);
    }

    impl cxx_qt::Threading for ThemesModel {}
    impl cxx_qt::Initialize for ThemesModel {}
}

pub struct ThemesModelRust {
    platform_library: QString,
    reference: QString,
    reference_folder: QString,
    open: bool,
    kind: QString,
    kinds: QStringList,
    focused: i32,
    chosen: QString,
    words: QString,
    name: QString,
    line: QString,
    preview: QString,
    findings: QString,
    usable: bool,
    help: i32,
    rows: Vec<Row>,
    store: Option<PathBuf>,
    /// The reads asked for, so an older read that finishes late is dropped.
    asked: u64,
}

impl Default for ThemesModelRust {
    fn default() -> Self {
        let kinds: QList<QString> = themes::KINDS.iter().map(|k| QString::from(*k)).collect();
        ThemesModelRust {
            platform_library: QString::default(),
            reference: QString::default(),
            reference_folder: QString::default(),
            open: false,
            kind: QString::from(themes::KINDS[0]),
            kinds: QStringList::from(&kinds),
            focused: 0,
            chosen: QString::from(REFERENCE),
            words: QString::default(),
            name: QString::default(),
            line: QString::default(),
            preview: QString::default(),
            findings: QString::default(),
            usable: false,
            help: shown_screen(Screen::ThemesChecked),
            rows: Vec::new(),
            store: None,
            asked: 0,
        }
    }
}

impl cxx_qt::Initialize for qobject::ThemesModel {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut().on_open_changed(|model| model.read_if_open()).release();
        self.as_mut().on_platform_library_changed(|model| model.read_if_open()).release();
        self.as_mut().on_reference_changed(|model| model.read_if_open()).release();
        self.as_mut().on_reference_folder_changed(|model| model.read_if_open()).release();
        self.as_mut()
            .on_kind_changed(|mut model| {
                model.as_mut().begin_reset_model();
                model.as_mut().end_reset_model();
                model.refocus();
            })
            .release();
        self.on_focused_changed(|model| model.detail()).release();
    }
}

const ID: i32 = 0;
const NAME: i32 = 1;
const KIND: i32 = 2;
const AUTHOR: i32 = 3;
const API: i32 = 4;
const VARIANTS: i32 = 5;
const PREVIEW: i32 = 6;
const CURRENT: i32 = 7;
const CHECKED: i32 = 8;
const USABLE: i32 = 9;
const LABEL: i32 = 10;
const VALUE: i32 = 11;

/// A row's check as the `checked` role says it: empty before one, `checking`, `ok`, or the findings a line each.
fn checked_words(checked: &Checked) -> String {
    match checked {
        Checked::Not => String::new(),
        Checked::Running => "checking".to_owned(),
        Checked::Ok => "ok".to_owned(),
        Checked::Findings(found) => found.join("\n"),
    }
}

/// The listing read through the platform library: its settings and its themes store.
fn read(platform_library: &str, reference: &str, reference_folder: &str) -> Listing {
    let platform = Platform::load(platform_library).ok();
    let settings = platform.as_ref().and_then(|p| app_settings(p).ok()).unwrap_or(Value::Null);
    let store = platform.as_ref().and_then(|p| p.directory(DIR_THEMES).ok().flatten());
    themes::listing(&settings, store.as_deref(), reference, reference_folder)
}

impl qobject::ThemesModel {
    /// The rows of the kind listed.
    fn shown(&self) -> Vec<&Row> {
        let kind = self.kind.to_string();
        self.rows.iter().filter(|r| themes::KINDS[r.kind.index()] == kind).collect()
    }

    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let Some(row) = self.shown().get(index.row() as usize).copied() else { return QVariant::default() };
        let text = |s: &str| QVariant::from(&QString::from(s));
        match role {
            ID => text(&row.id),
            NAME | LABEL => text(&row.name),
            KIND => text(themes::KINDS[row.kind.index()]),
            AUTHOR => text(&row.author),
            API => text(&row.api),
            VARIANTS => text(&row.variants),
            PREVIEW => text(&row.preview),
            CURRENT => QVariant::from(&row.current),
            CHECKED => text(&checked_words(&row.checked)),
            USABLE => QVariant::from(&row.usable()),
            VALUE => text(&row.value()),
            _ => QVariant::default(),
        }
    }

    pub fn role_names(&self) -> QHash<QHashPair_i32_QByteArray> {
        let mut roles = QHash::<QHashPair_i32_QByteArray>::default();
        let names = [
            (ID, "id"),
            (NAME, "name"),
            (KIND, "kind"),
            (AUTHOR, "author"),
            (API, "api"),
            (VARIANTS, "variants"),
            (PREVIEW, "preview"),
            (CURRENT, "current"),
            (CHECKED, "checked"),
            (USABLE, "usable"),
            (LABEL, "label"),
            (VALUE, "value"),
        ];
        for (role, name) in names {
            roles.insert(role, QByteArray::from(name));
        }
        roles
    }

    pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
        self.shown().len() as i32
    }

    fn read_if_open(self: Pin<&mut Self>) {
        if self.open {
            self.refresh();
        }
    }

    pub fn refresh(mut self: Pin<&mut Self>) {
        let asked = self.asked + 1;
        self.as_mut().rust_mut().asked = asked;
        let (library, reference, folder) = (self.platform_library.to_string(), self.reference.to_string(), self.reference_folder.to_string());
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            let listing = read(&library, &reference, &folder);
            let _ = qt.queue(move |model| model.take(asked, listing));
        });
    }

    /// A finished read: its rows, each keeping the check it had while the program runs.
    fn take(mut self: Pin<&mut Self>, asked: u64, mut listing: Listing) {
        if asked != self.asked {
            return;
        }
        for row in &mut listing.rows {
            if let Some(before) = self.rows.iter().find(|r| r.id == row.id && r.kind == row.kind && row.checked == Checked::Not) {
                row.checked = before.checked.clone();
            }
        }
        self.as_mut().begin_reset_model();
        self.as_mut().rust_mut().rows = listing.rows;
        self.as_mut().rust_mut().store = listing.store;
        self.as_mut().end_reset_model();
        self.as_mut().refocus();
        let (chosen, words) = (QString::from(listing.chosen.as_str()), QString::from(listing.words.as_str()));
        if self.chosen != chosen {
            self.as_mut().rust_mut().chosen = chosen;
            self.as_mut().chosen_changed();
        }
        if self.words != words {
            self.as_mut().rust_mut().words = words;
            self.as_mut().words_changed();
        }
        self.detail();
    }

    /// After a reset the first row is focused, and the view's current row follows it.
    fn refocus(mut self: Pin<&mut Self>) {
        self.as_mut().set_focused(0);
        self.detail();
    }

    /// Every row of the kind listed said to have changed, keeping the view's current row.
    fn rows_changed(mut self: Pin<&mut Self>) {
        let count = self.shown().len() as i32;
        if count > 0 {
            let (first, last) = (self.index(0, 0, &QModelIndex::default()), self.index(count - 1, 0, &QModelIndex::default()));
            self.as_mut().data_changed(&first, &last, &QVector::<i32>::default());
        }
        self.detail();
    }

    /// The row of an id, by its place in `rows`.
    fn find(&self, id: &str) -> Option<usize> {
        self.rows.iter().position(|r| r.id == id)
    }

    /// The folder a row's theme is in, inside the store.
    fn folder_of(&self, row: &Row) -> Option<PathBuf> {
        self.store.as_ref().map(|s| s.join(&row.id))
    }

    pub fn use_theme(mut self: Pin<&mut Self>, id: &QString) {
        let id = id.to_string();
        let Some(at) = self.find(&id).filter(|&at| self.rows[at].usable()) else { return };
        let stored = Platform::load(self.platform_library.to_string()).map_err(|e| e.to_string()).and_then(|p| store_app_setting(&p, SETTING, Value::String(id.clone())));
        if let Err(words) = stored {
            self.as_mut().rust_mut().words = QString::from(format!("The theme could not be stored: {words}").as_str());
            self.as_mut().words_changed();
            return;
        }
        let folder = if id == REFERENCE { String::new() } else { self.folder_of(&self.rows[at]).map(|f| f.to_string_lossy().into_owned()).unwrap_or_default() };
        for (i, row) in self.as_mut().rust_mut().rows.iter_mut().enumerate() {
            row.current = i == at;
        }
        self.as_mut().rust_mut().chosen = QString::from(id.as_str());
        self.as_mut().chosen_changed();
        if !self.words.is_empty() {
            self.as_mut().rust_mut().words = QString::default();
            self.as_mut().words_changed();
        }
        self.as_mut().rows_changed();
        self.used(QString::from(folder.as_str()));
    }

    pub fn check(mut self: Pin<&mut Self>, id: &QString) {
        let id = id.to_string();
        let Some(at) = self.find(&id) else { return };
        let row = &self.rows[at];
        let Some(folder) = self.folder_of(row).filter(|_| row.id != REFERENCE && row.checked != Checked::Running) else { return };
        let kind: Kind = row.kind;
        self.as_mut().rust_mut().rows[at].checked = Checked::Running;
        self.as_mut().rows_changed();
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            let checked = themes::check(kind, &folder);
            let _ = qt.queue(move |model| model.checked(id, checked));
        });
    }

    /// A finished check, kept on its row.
    fn checked(mut self: Pin<&mut Self>, id: String, checked: Checked) {
        let Some(at) = self.find(&id) else { return };
        self.as_mut().rust_mut().rows[at].checked = checked;
        self.rows_changed();
    }

    /// The focused row's id.
    fn focused_id(&self) -> Option<QString> {
        self.shown().get(self.focused.max(0) as usize).map(|r| QString::from(r.id.as_str()))
    }

    pub fn act(self: Pin<&mut Self>, key: &QString) {
        if let Some(id) = self.focused_id().filter(|_| key.to_string() == crate::help::CHECK) {
            self.check(&id);
        }
    }

    pub fn accept(self: Pin<&mut Self>) {
        if let Some(id) = self.focused_id() {
            self.use_theme(&id);
        }
    }

    /// The focused row's detail and help bar.
    fn detail(mut self: Pin<&mut Self>) {
        let row = self.shown().get(self.focused.max(0) as usize).map(|r| (*r).clone());
        let (name, line, preview, findings, usable) = match &row {
            Some(r) => (r.name.clone(), r.line(), r.preview.clone(), r.findings(), r.usable()),
            None => Default::default(),
        };
        let help = shown_screen(if usable { Screen::Themes } else { Screen::ThemesChecked });
        let (name, line, preview, findings) = (QString::from(name.as_str()), QString::from(line.as_str()), QString::from(preview.as_str()), QString::from(findings.as_str()));
        if self.name != name {
            self.as_mut().rust_mut().name = name;
            self.as_mut().name_changed();
        }
        if self.line != line {
            self.as_mut().rust_mut().line = line;
            self.as_mut().line_changed();
        }
        if self.preview != preview {
            self.as_mut().rust_mut().preview = preview;
            self.as_mut().preview_changed();
        }
        if self.findings != findings {
            self.as_mut().rust_mut().findings = findings;
            self.as_mut().findings_changed();
        }
        if self.usable != usable {
            self.as_mut().rust_mut().usable = usable;
            self.as_mut().usable_changed();
        }
        if self.help != help {
            self.as_mut().rust_mut().help = help;
            self.as_mut().help_changed();
        }
    }
}
