//! `SettingsModel`, the Settings screen as a QAbstractListModel (EmuSen_Qt.md §7.7): settings.rs's rows of the section
//! or page shown, with the roles the kit's settings frame reads, and the focused row's pane as properties. A read runs
//! off the GUI thread and comes back through qt_thread().queue; a change is saved through Galaxia's model at once.

use crate::consoles_source::{ClientConsoles, ScriptedConsoles};
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
        include!("emusen-kit-qt/src/kinds.cxxqt.h");
        #[namespace = "Pad"]
        type PadButton = emusen_kit_qt::kinds::ffi::PadButton;
    }

    extern "RustQt" {
        #[qobject]
        #[base = QAbstractListModel]
        #[qml_element]
        /// The platform library, through which the settings are read and saved.
        #[qproperty(QString, platform_library, cxx_name = "platformLibrary")]
        /// The runtime's cores folder, as the library's; empty for the installation's own.
        #[qproperty(QString, cores)]
        /// A file holding the consoles' world in place of the runtime's, as the recordings script theirs; empty for the runtime.
        #[qproperty(QString, world)]
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
        /// The focused row's sentence, or the bindings page's, which has no row (EmuSen_Qt.md §7.7.H).
        #[qproperty(QString, sentence, READ, NOTIFY)]
        #[qproperty(QStringList, pane_lines, cxx_name = "paneLines", READ, NOTIFY)]
        #[qproperty(QStringList, pane_muted, cxx_name = "paneMuted", READ, NOTIFY)]
        /// The tone (Kit.Tones) of the focused row's sentence; `lineTone` gives each further line's.
        #[qproperty(i32, sentence_tone, cxx_name = "sentenceTone", READ, NOTIFY)]
        /// The pane's preview: 0 none, 1 the game's picture, 2 a meter, 3 the two variants, 4 the text sizes.
        #[qproperty(i32, preview, READ, NOTIFY)]
        #[qproperty(QString, caption, READ, NOTIFY)]
        #[qproperty(f64, meter, READ, NOTIFY)]
        /// The text sizes' lines, or the two variants' names; `sampleScale` gives each line's scale.
        #[qproperty(QStringList, samples, READ, NOTIFY)]
        /// What the settings ask of the whole program, once read: the text size (its word, which the kit's tokens scale
        /// by), the light variant, reduced motion.
        #[qproperty(bool, ready, READ, NOTIFY)]
        #[qproperty(QString, text_size, cxx_name = "textSize", READ, NOTIFY)]
        #[qproperty(bool, light, READ, NOTIFY)]
        #[qproperty(bool, reduced_motion, cxx_name = "reducedMotion", READ, NOTIFY)]
        /// And of the pads (EmuSen_Qt.md §7.7.H3): Swap A and B in menus, Controller type's family (Kit.Families; -1 for
        /// Automatic), Controller notices and Rumble; the rest the pads' hand-over is given (`gamePads`).
        #[qproperty(bool, swap_buttons, cxx_name = "swapButtons", READ, NOTIFY)]
        #[qproperty(i32, controller_family, cxx_name = "controllerFamily", READ, NOTIFY)]
        #[qproperty(bool, controller_notices, cxx_name = "controllerNotices", READ, NOTIFY)]
        #[qproperty(bool, rumble, READ, NOTIFY)]
        /// The help bar's screen (HelpModel's numbers).
        #[qproperty(i32, help, READ, NOTIFY)]
        /// The pads' hand-over (`PadReader.gamePads`), whose seats the Players page shows and changes (EmuSen_Qt.md §7.7.H).
        #[qproperty(u64, game_pads, cxx_name = "gamePads")]
        /// The bindings page: whether it is shown, its drawing file, controls, labels' words, focused and held controls,
        /// its strip and caption, and whether a press is listened for or the tester runs.
        #[qproperty(bool, bindings_shown, cxx_name = "bindingsShown", READ, NOTIFY = drawn)]
        #[qproperty(QString, drawing, READ, NOTIFY = drawn)]
        #[qproperty(QStringList, drawing_controls, cxx_name = "drawingControls", READ, NOTIFY = drawn)]
        #[qproperty(QStringList, drawing_texts, cxx_name = "drawingTexts", READ, NOTIFY = drawn)]
        #[qproperty(i32, drawing_focus, cxx_name = "drawingFocus", READ, NOTIFY = drawn)]
        #[qproperty(QStringList, drawing_held, cxx_name = "drawingHeld", READ, NOTIFY = drawn)]
        #[qproperty(QStringList, binding_tabs, cxx_name = "bindingTabs", READ, NOTIFY = drawn)]
        #[qproperty(QString, binding_caption, cxx_name = "bindingCaption", READ, NOTIFY = drawn)]
        #[qproperty(bool, listening, READ, NOTIFY = drawn)]
        #[qproperty(bool, listening_pad, cxx_name = "listeningPad", READ, NOTIFY = drawn)]
        #[qproperty(bool, testing, READ, NOTIFY = drawn)]
        /// The tester: how far each control is pushed (the drawing's knobs), whether the interface's B is held to leave
        /// it, and how long it is held to leave.
        #[qproperty(QVariant, drawing_pushes, cxx_name = "drawingPushes", READ, NOTIFY = drawn)]
        #[qproperty(bool, tester_back, cxx_name = "testerBack", READ, NOTIFY = drawn)]
        #[qproperty(i32, tester_hold_ms, cxx_name = "testerHoldMs", READ, NOTIFY = drawn)]
        /// The Pause Card's Game Options ▸ Controllers rows, as plain data for `PauseModel.controllers` (EmuSen_Interface.md §20.3).
        #[qproperty(QString, card_controllers, cxx_name = "cardControllers", READ, NOTIFY = drawn)]
        /// A page's question waiting for typed words: whether one is, its title and its prompt.
        #[qproperty(bool, typing, READ, NOTIFY = asked)]
        #[qproperty(QString, typing_title, cxx_name = "typingTitle", READ, NOTIFY = asked)]
        #[qproperty(QString, typing_prompt, cxx_name = "typingPrompt", READ, NOTIFY = asked)]
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

        /// The tone (Kit.Tones) of the pane's further line at `index`, the text's past the tones the pane gives.
        #[qinvokable]
        #[cxx_name = "lineTone"]
        fn line_tone(self: &SettingsModel, index: i32) -> i32;

        /// The scale of the sample at `index`, 1 past the samples.
        #[qinvokable]
        #[cxx_name = "sampleScale"]
        fn sample_scale(self: &SettingsModel, index: i32) -> f64;

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

        /// LB (-1) or RB (+1) on a console's page: the console before or after it; false elsewhere.
        #[qinvokable]
        fn shoulder(self: Pin<&mut SettingsModel>, by: i32) -> bool;

        /// A page closed: the row that opened it is focused again.
        #[qsignal]
        fn refocus(self: Pin<&mut SettingsModel>, row: i32);

        /// A folder row's key (`Model Key`) to choose a folder for in the system's dialog, starting at `start`.
        #[qsignal]
        #[cxx_name = "chooseFolder"]
        fn choose_folder(self: Pin<&mut SettingsModel>, key: QString, start: QString);

        /// The folder chosen for a folder row's key, saved at once.
        #[qinvokable]
        #[cxx_name = "setFolder"]
        fn set_folder(self: Pin<&mut SettingsModel>, key: &QString, path: &QString);

        /// A folder to show in the system's file manager, through the program's one action for it.
        #[qsignal]
        #[cxx_name = "openFolder"]
        fn open_folder(self: Pin<&mut SettingsModel>, path: QString);

        /// The bindings page's drawing changed.
        #[qsignal]
        fn drawn(self: Pin<&mut SettingsModel>);

        /// Words to type for a page (a profile's name), starting with `start`; `typing`, `typingTitle` and
        /// `typingPrompt` say the question, and `typed` answers.
        #[qsignal]
        #[cxx_name = "typeWords"]
        fn type_words(self: Pin<&mut SettingsModel>, start: QString);

        /// The question's words changed.
        #[qsignal]
        fn asked(self: Pin<&mut SettingsModel>);

        /// A declared action the page did not take, for the screens (Start, Home).
        #[qsignal]
        fn passed(self: Pin<&mut SettingsModel>, key: QString);

        /// A pad's button while a key is listened for: B lets the listening go.
        #[qinvokable]
        #[cxx_name = "padPressed"]
        fn pad_pressed(self: Pin<&mut SettingsModel>, button: PadButton);

        /// A row of the card's Controllers page stepped (`by` ±1) or chosen (`by` 0).
        #[qinvokable]
        #[cxx_name = "cardAct"]
        fn card_act(self: Pin<&mut SettingsModel>, index: i32, by: i32);

        /// The words typed for the page that asked, or none for a question let go.
        #[qinvokable]
        fn typed(self: Pin<&mut SettingsModel>, words: &QString);

        /// The seats read again from the pads' hand-over, as the pads' reader says they changed.
        #[qinvokable]
        #[cxx_name = "seatsChanged"]
        fn seats_changed(self: Pin<&mut SettingsModel>);

        /// A direction on the bindings page (`dx`, `dy`): the nearest control that way on the drawing.
        #[qinvokable]
        #[cxx_name = "bindingMove"]
        fn binding_move(self: Pin<&mut SettingsModel>, dx: i32, dy: i32);

        /// A control of the bindings page focused by the pointer.
        #[qinvokable]
        #[cxx_name = "bindingFocus"]
        fn binding_focus(self: Pin<&mut SettingsModel>, index: i32);

        /// A on the bindings page: a press listened for.
        #[qinvokable]
        #[cxx_name = "bindingAccept"]
        fn binding_accept(self: Pin<&mut SettingsModel>);

        /// One of the help bar's declared actions on the bindings page or Hotkeys (reset, player, clear, test); one the
        /// page does not take is `passed` on.
        #[qinvokable]
        fn act(self: Pin<&mut SettingsModel>, key: &QString);

        /// A key pressed while a press is listened for, by Qt's number and modifiers: bound, or Escape cancelling;
        /// true when it was taken.
        #[qinvokable]
        #[cxx_name = "heardKey"]
        fn heard_key(self: Pin<&mut SettingsModel>, key: i32, modifiers: i32) -> bool;

        /// A key let go while the tester runs; true when it was taken.
        #[qinvokable]
        #[cxx_name = "releasedKey"]
        fn released_key(self: Pin<&mut SettingsModel>, key: i32, modifiers: i32) -> bool;

        /// A button of the interface down or up on the pads while the tester runs, before the player's swap (`swap`):
        /// whether it is B, held to leave the tester. True for a release, which the router still hears, so that no
        /// button stays held there.
        #[qinvokable]
        #[cxx_name = "testerButton"]
        fn tester_button(self: Pin<&mut SettingsModel>, button: PadButton, down: bool, swap: bool) -> bool;
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
    cores: QString,
    world: QString,
    open: bool,
    facts: QString,
    sections: QStringList,
    section: i32,
    trail: QStringList,
    focused: i32,
    sentence: QString,
    pane_lines: QStringList,
    pane_muted: QStringList,
    sentence_tone: i32,
    line_tones: Vec<i32>,
    preview: i32,
    caption: QString,
    meter: f64,
    samples: QStringList,
    sample_scales: Vec<f64>,
    ready: bool,
    text_size: QString,
    light: bool,
    reduced_motion: bool,
    swap_buttons: bool,
    controller_family: i32,
    controller_notices: bool,
    rumble: bool,
    help: i32,
    problem: QString,
    view: SettingsView,
    rows: Vec<settings::Row>,
    /// The reads asked for, so an older read that finishes late is dropped.
    asked: u64,
    game_pads: u64,
    bindings_shown: bool,
    drawing: QString,
    drawing_controls: QStringList,
    drawing_texts: QStringList,
    drawing_focus: i32,
    drawing_held: QStringList,
    drawing_pushes: QVariant,
    tester_back: bool,
    tester_hold_ms: i32,
    binding_tabs: QStringList,
    binding_caption: QString,
    listening: bool,
    listening_pad: bool,
    testing: bool,
    card_controllers: QString,
    typing: bool,
    typing_title: QString,
    typing_prompt: QString,
    /// The page's question waiting for typed words.
    typed_for: Option<crate::settings_controllers::Typed>,
    /// Each pad's buttons at the last read of the seats, so a press listened for is the one newly down.
    pressed: Vec<(u64, u32)>,
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
            cores: QString::default(),
            world: QString::default(),
            open: false,
            facts: QString::default(),
            sections: list(view.sections().into_iter()),
            section: 0,
            trail: QStringList::default(),
            focused: 0,
            sentence: QString::default(),
            pane_lines: QStringList::default(),
            pane_muted: QStringList::default(),
            sentence_tone: 0,
            line_tones: Vec::new(),
            preview: 0,
            caption: QString::default(),
            meter: 0.0,
            samples: QStringList::default(),
            sample_scales: Vec::new(),
            ready: false,
            text_size: QString::default(),
            light: false,
            reduced_motion: false,
            swap_buttons: false,
            controller_family: -1,
            controller_notices: false,
            rumble: false,
            help: shown_screen(Screen::Settings),
            problem: QString::default(),
            rows: view.rows(),
            view,
            asked: 0,
            game_pads: 0,
            bindings_shown: false,
            drawing: QString::default(),
            drawing_controls: QStringList::default(),
            drawing_texts: QStringList::default(),
            drawing_focus: -1,
            drawing_held: QStringList::default(),
            drawing_pushes: QVariant::from(&QList::<f64>::default()),
            tester_back: false,
            tester_hold_ms: 0,
            binding_tabs: QStringList::default(),
            binding_caption: QString::default(),
            listening: false,
            listening_pad: false,
            testing: false,
            card_controllers: QString::default(),
            typing: false,
            typing_title: QString::default(),
            typing_prompt: QString::default(),
            typed_for: None,
            pressed: Vec::new(),
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
        self.as_mut()
            .on_game_pads_changed(|mut model| {
                model.as_mut().settle_pads();
                model.seats_changed();
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
            MARKED => QVariant::from(&row.marked),
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

    pub fn line_tone(&self, index: i32) -> i32 {
        usize::try_from(index).ok().and_then(|i| self.line_tones.get(i)).copied().unwrap_or(0)
    }

    pub fn sample_scale(&self, index: i32) -> f64 {
        usize::try_from(index).ok().and_then(|i| self.sample_scales.get(i)).copied().unwrap_or(1.0)
    }

    pub fn refresh(mut self: Pin<&mut Self>) {
        let library = self.platform_library.to_string();
        if library.is_empty() {
            return;
        }
        let asked = self.asked.wrapping_add(1);
        self.as_mut().rust_mut().asked = asked;
        // Cleared at the start of each attempt, so the same failure twice is said twice (EmuSen_Qt.md §7.9).
        self.as_mut().said(String::new());
        let thread = self.qt_thread();
        let (cores, world) = (self.cores.to_string(), self.world.to_string());
        std::thread::spawn(move || {
            let read = ClientSettings::open(&library).and_then(|source| {
                let mut read = settings::read(&source)?;
                read.consoles = if world.is_empty() {
                    crate::consoles::read(&ClientConsoles::open(&library, &cores)?)?
                } else {
                    crate::consoles::read(&ScriptedConsoles::open(&world)?)?
                };
                let names: Vec<&str> = read.consoles.consoles.iter().map(|c| c.console.as_str()).collect();
                read.bindings = source.bindings(&names)?;
                Ok((read, source.chooser()))
            });
            let _ = thread.queue(move |mut model| {
                if model.asked != asked {
                    return;
                }
                match read {
                    Ok((read, chooser)) => {
                        model.as_mut().rust_mut().view.read = read;
                        model.as_mut().rust_mut().view.chooser = Some(chooser);
                        model.as_mut().said(String::new());
                    }
                    Err(words) => model.as_mut().said(format!("The settings could not be read: {words}")),
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
        if self.as_mut().rust_mut().view.back_taken() {
            self.shown(false);
            return;
        }
        match self.as_mut().rust_mut().view.back() {
            Some(row) => {
                self.as_mut().shown(true);
                self.refocus(row as i32);
            }
            None => self.left(),
        }
    }

    pub fn set_folder(mut self: Pin<&mut Self>, key: &QString, path: &QString) {
        if path.is_empty() {
            return;
        }
        let outcome = self.as_mut().rust_mut().view.set_folder(&key.to_string(), &path.to_string());
        self.outcome(outcome);
    }

    pub fn shoulder(mut self: Pin<&mut Self>, by: i32) -> bool {
        if self.view.binding_open() {
            let stepped = self.as_mut().rust_mut().view.binding_shoulder(by);
            if stepped {
                self.shown(false);
            }
            return stepped;
        }
        let stepped = self.as_mut().rust_mut().view.shoulder(by);
        if stepped {
            self.shown(true);
        }
        stepped
    }

    fn read_if_named(self: Pin<&mut Self>) {
        if !self.platform_library.is_empty() {
            self.refresh();
        }
    }

    fn outcome(mut self: Pin<&mut Self>, outcome: Outcome) {
        if matches!(outcome, Outcome::Saved(..) | Outcome::Set { .. } | Outcome::NewCard(_)) {
            self.as_mut().said(String::new());
        }
        match outcome {
            Outcome::Nothing => {}
            Outcome::Saved(model, document) => {
                let saved = ClientSettings::open(&self.platform_library.to_string()).and_then(|s| s.save(model, &document));
                self.as_mut().said(saved.err().map(|e| format!("The settings could not be saved: {e}")).unwrap_or_default());
                self.as_mut().applied();
                self.shown(false);
            }
            Outcome::Opened => self.shown(true),
            Outcome::Shown => self.shown(false),
            Outcome::Door(name) => self.door(QString::from(name.as_str())),
            Outcome::OpenFolder(path) => self.open_folder(QString::from(path.as_str())),
            Outcome::ChooseFolder { key, start } => self.choose_folder(QString::from(key.as_str()), QString::from(start.as_str())),
            Outcome::Set { .. } | Outcome::NewCard(_) | Outcome::Cards { .. } if !self.world.is_empty() => self.shown(false),
            Outcome::Cards { op, args } => {
                let (done, words) = match ClientConsoles::open(&self.platform_library.to_string(), &self.cores.to_string()) {
                    Ok(c) => c.cards_change(op, &args),
                    Err(e) => (false, e),
                };
                self.as_mut().rust_mut().view.cards_said(op, done, words);
                self.as_mut().shown(true);
                self.refresh();
            }
            Outcome::Set { console, key, value } => {
                let set = ClientConsoles::open(&self.platform_library.to_string(), &self.cores.to_string()).and_then(|c| c.set(&console, &key, value.as_deref()));
                let failed = set.is_err();
                self.as_mut().said(set.err().map(|e| format!("The console's setting could not be stored: {e}")).unwrap_or_default());
                self.as_mut().shown(false);
                // The engine chosen changes the schema the page draws, and a refusal leaves the stored value as it was.
                if failed || key == crate::consoles::ENGINE_KEY {
                    self.refresh();
                }
            }
            Outcome::NewCard(path) => {
                let made = ClientConsoles::open(&self.platform_library.to_string(), &self.cores.to_string()).and_then(|c| c.new_card(&path));
                self.as_mut().said(made.err().map(|e| format!("The memory card could not be made: {e}")).unwrap_or_default());
                self.refresh();
            }
            Outcome::Reread => self.refresh(),
            Outcome::Bindings(files) => {
                let saved = ClientSettings::open(&self.platform_library.to_string()).and_then(|s| s.save_bindings(&self.view.read.bindings, &files));
                self.as_mut().said(saved.err().unwrap_or_default());
                // A running game hears its pads through the files' bindings: bound again at once (EmuSen_Interface.md §20.3).
                // SAFETY: the address is PadReader.gamePads, whose reader outlives the screens.
                if let Some(pads) = unsafe { emusen_qt_endymion::game::GamePads::at(self.game_pads) } {
                    pads.rebind();
                }
                self.as_mut().shown(false);
                self.refresh();
            }
            Outcome::Seat(ask) => {
                // SAFETY: the address is PadReader.gamePads, whose reader outlives the screens.
                if let Some(pads) = unsafe { emusen_qt_endymion::game::GamePads::at(self.game_pads) } {
                    pads.ask(ask);
                }
            }
            Outcome::Type { title, prompt, start, typed } => {
                {
                    let mut r = self.as_mut().rust_mut();
                    r.typed_for = Some(typed);
                    r.typing = true;
                    r.typing_title = QString::from(title.as_str());
                    r.typing_prompt = QString::from(prompt.as_str());
                }
                self.as_mut().asked();
                self.type_words(QString::from(start.as_str()));
            }
            Outcome::All(all) => {
                for one in all {
                    self.as_mut().outcome(one);
                }
            }
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
        let (size, light, reduced) = (QString::from(self.view.text_size().as_str()), self.view.light(), self.view.reduced_motion());
        if self.text_size != size {
            self.as_mut().rust_mut().text_size = size;
            self.as_mut().text_size_changed();
        }
        if self.light != light {
            self.as_mut().rust_mut().light = light;
            self.as_mut().light_changed();
        }
        if self.reduced_motion != reduced {
            self.as_mut().rust_mut().reduced_motion = reduced;
            self.as_mut().reduced_motion_changed();
        }
        let (swap, notices, rumble) = (self.view.swap_buttons(), self.view.controller_notices(), self.view.rumble());
        let family = self.view.controller_family().map_or(-1, |f| emusen_kit_qt::kinds::family_of(f).repr);
        if self.swap_buttons != swap {
            self.as_mut().rust_mut().swap_buttons = swap;
            self.as_mut().swap_buttons_changed();
        }
        if self.controller_family != family {
            self.as_mut().rust_mut().controller_family = family;
            self.as_mut().controller_family_changed();
        }
        if self.controller_notices != notices {
            self.as_mut().rust_mut().controller_notices = notices;
            self.as_mut().controller_notices_changed();
        }
        if self.rumble != rumble {
            self.as_mut().rust_mut().rumble = rumble;
            self.as_mut().rumble_changed();
        }
        self.as_mut().settle_pads();
        if !self.ready {
            self.as_mut().rust_mut().ready = true;
            self.ready_changed();
        }
    }

    /// The settings the pads act on handed to the pads' reader, once read (Mistress's `SaveApp`: deadzone, stickdpad, mirror).
    fn settle_pads(self: Pin<&mut Self>) {
        let Some(settings) = self.view.pad_settings() else { return };
        // SAFETY: the address is PadReader.gamePads, whose reader outlives the screens.
        if let Some(pads) = unsafe { emusen_qt_endymion::game::GamePads::at(self.game_pads) } {
            pads.settle(settings);
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
        let help = shown_screen(self.view.help());
        if self.help != help {
            self.as_mut().rust_mut().help = help;
            self.as_mut().help_changed();
        }
        self.as_mut().draw();
        self.pane();
    }

    /// The bindings page's drawing from the view.
    fn draw(mut self: Pin<&mut Self>) {
        let drawn = self.view.drawn();
        let shown = drawn.is_some();
        let d = drawn.unwrap_or_default();
        let (listening, listening_pad, testing) = (self.view.listening(), self.view.listening_pad(), self.view.testing());
        let hold = self.view.tester_hold_ms();
        let card = serde_json::Value::Array(self.view.game_rows().iter().map(|g| g.to_json()).collect()).to_string();
        let mut r = self.as_mut().rust_mut();
        r.bindings_shown = shown;
        r.drawing = QString::from(d.drawing.as_str());
        r.drawing_controls = list(d.controls.into_iter());
        r.drawing_texts = list(d.texts.into_iter());
        r.drawing_focus = d.focus;
        r.drawing_held = list(d.held.into_iter());
        r.drawing_pushes = QVariant::from(&d.pushes.iter().copied().collect::<QList<f64>>());
        r.tester_back = r.tester_back && testing;
        r.tester_hold_ms = hold;
        r.binding_tabs = list(d.tabs.into_iter());
        r.binding_caption = QString::from(d.caption.as_str());
        r.listening = listening;
        r.listening_pad = listening_pad;
        r.testing = testing;
        r.card_controllers = QString::from(card.as_str());
        self.drawn();
    }

    pub fn card_act(mut self: Pin<&mut Self>, index: i32, by: i32) {
        let outcome = self.as_mut().rust_mut().view.game_act(index.max(0) as usize, by);
        let opened = matches!(&outcome, Outcome::Door(d) if d == crate::settings_controllers::CARD_BINDINGS);
        if opened {
            self.as_mut().shown(true);
        }
        self.outcome(outcome);
    }

    pub fn typed(mut self: Pin<&mut Self>, words: &QString) {
        let Some(typed) = self.as_mut().rust_mut().typed_for.take() else { return };
        self.as_mut().rust_mut().typing = false;
        self.as_mut().asked();
        let outcome = self.as_mut().rust_mut().view.typed(&typed, &words.to_string());
        self.outcome(outcome);
    }

    pub fn seats_changed(mut self: Pin<&mut Self>) {
        // SAFETY: the address is PadReader.gamePads, whose reader outlives the screens.
        let Some(pads) = (unsafe { emusen_qt_endymion::game::GamePads::at(self.game_pads) }) else { return };
        let seated = crate::settings_controllers::Seated { players: pads.players(), pads: pads.connected(), deadzone: pads.stick_deadzone() };
        let before = std::mem::take(&mut self.as_mut().rust_mut().pressed);
        let now: Vec<(u64, u32)> = seated.pads.iter().map(|p| (p.key, p.buttons)).collect();
        self.as_mut().rust_mut().pressed = now.clone();
        let fresh = now.iter().find_map(|(key, mask)| {
            let was = before.iter().find(|(k, _)| k == key).map_or(0, |(_, m)| *m);
            let down = mask & !was;
            (down != 0).then_some(down)
        });
        let (held, pushes) = (self.view.held_from(&seated), self.view.pushes_from(&seated));
        {
            let mut r = self.as_mut().rust_mut();
            r.view.seats = seated;
            r.view.held = held;
            r.view.pushes = pushes;
        }
        if let (true, Some(down)) = (self.view.listening_pad(), fresh) {
            let outcome = self.as_mut().rust_mut().view.heard_down(down);
            self.outcome(outcome);
            return;
        }
        self.shown(false);
    }

    pub fn binding_move(mut self: Pin<&mut Self>, dx: i32, dy: i32) {
        if self.as_mut().rust_mut().view.binding_move(dx, dy) {
            self.shown(false);
        }
    }

    pub fn binding_focus(mut self: Pin<&mut Self>, index: i32) {
        if self.as_mut().rust_mut().view.binding_focus(index.max(0) as usize) {
            self.shown(false);
        }
    }

    pub fn binding_accept(mut self: Pin<&mut Self>) {
        if self.as_mut().rust_mut().view.binding_accept() {
            // The buttons held as listening starts are not the press listened for.
            self.as_mut().seats_changed();
            self.shown(false);
        }
    }

    pub fn act(mut self: Pin<&mut Self>, key: &QString) {
        let words = key.to_string();
        let outcome = if self.view.binding_open() {
            self.as_mut().rust_mut().view.binding_act(&words)
        } else if words == crate::help::CLEAR {
            let focused = self.focused.max(0) as usize;
            self.as_mut().rust_mut().view.clear_hotkey(focused)
        } else {
            Outcome::Nothing
        };
        if outcome == Outcome::Nothing {
            self.passed(key.clone());
            return;
        }
        self.outcome(outcome);
    }

    pub fn pad_pressed(self: Pin<&mut Self>, button: qobject::PadButton) {
        if emusen_kit_qt::kinds::ui_button(button) == Some(emusen_kit_core::input::UiButton::Back) {
            self.back();
        }
    }

    pub fn heard_key(mut self: Pin<&mut Self>, key: i32, modifiers: i32) -> bool {
        if self.view.testing() {
            return self.tester_key(key, modifiers, true);
        }
        if !self.view.listening() || self.view.listening_pad() {
            return false;
        }
        let outcome = match emusen_qt_endymion::keys::heard(key, modifiers) {
            emusen_qt_endymion::keys::Heard::Nothing => return true,
            emusen_qt_endymion::keys::Heard::Escape => self.as_mut().rust_mut().view.heard_key(None),
            emusen_qt_endymion::keys::Heard::Key(word) => self.as_mut().rust_mut().view.heard_key(Some(word)),
        };
        self.outcome(outcome);
        true
    }

    pub fn released_key(self: Pin<&mut Self>, key: i32, modifiers: i32) -> bool {
        self.view.testing() && self.tester_key(key, modifiers, false)
    }

    /// A key down or up while the tester runs: Escape stops it, any other key lights what it is bound to.
    fn tester_key(mut self: Pin<&mut Self>, key: i32, modifiers: i32, down: bool) -> bool {
        match emusen_qt_endymion::keys::heard(key, modifiers) {
            emusen_qt_endymion::keys::Heard::Nothing => {}
            emusen_qt_endymion::keys::Heard::Escape if down => self.as_mut().back(),
            emusen_qt_endymion::keys::Heard::Escape => {}
            emusen_qt_endymion::keys::Heard::Key(word) => {
                if self.as_mut().rust_mut().view.tester_key(word, down) {
                    self.shown(false);
                }
            }
        }
        true
    }

    pub fn tester_button(mut self: Pin<&mut Self>, button: qobject::PadButton, down: bool, swap: bool) -> bool {
        let back = emusen_kit_qt::kinds::ui_button(button).map(|b| b.swapped(swap)) == Some(emusen_kit_core::input::UiButton::Back);
        let held = back && down && self.view.testing();
        if back && self.tester_back != held {
            self.as_mut().rust_mut().tester_back = held;
            self.as_mut().drawn();
        }
        !down
    }

    /// The focused row's pane.
    fn pane(mut self: Pin<&mut Self>) {
        let pane = self.view.pane(self.focused.max(0) as usize);
        let sentence = QString::from(pane.sentence.as_str());
        if self.sentence != sentence {
            self.as_mut().rust_mut().sentence = sentence;
            self.as_mut().sentence_changed();
        }
        // The kit's Tones are numbered in the order tools::Tone lists them.
        let tones: Vec<i32> = pane.tones.iter().map(|t| *t as i32).collect();
        let sentence_tone = tones.first().copied().unwrap_or(0);
        if self.sentence_tone != sentence_tone {
            self.as_mut().rust_mut().sentence_tone = sentence_tone;
            self.as_mut().sentence_tone_changed();
        }
        // The lines' tones before the lines, so the delegates the new lines make read theirs.
        let line_tones: Vec<i32> = tones.iter().skip(1).copied().collect();
        let lines_changed = self.line_tones != line_tones;
        self.as_mut().rust_mut().line_tones = line_tones;
        let lines = list(pane.lines.into_iter());
        if self.pane_lines != lines || lines_changed {
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
            Preview::Heading => 5,
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
        let scales: Vec<f64> = pane.samples.iter().map(|(_, scale)| *scale).collect();
        if self.samples != samples || self.sample_scales != scales {
            // The scales first, so the delegates the new samples make read theirs.
            self.as_mut().rust_mut().sample_scales = scales;
            self.as_mut().rust_mut().samples = samples;
            self.as_mut().samples_changed();
        }
    }
}
