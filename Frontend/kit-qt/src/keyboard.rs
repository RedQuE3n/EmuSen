//! `KeyboardModel`: the docked keyboard's keys as a list model with their boxes and words, its text and caret, and the
//! keys and presses that change them, over kit-core's `Keyboard` (EmuSen_Kit.md §7.22). The kit's `Keyboard` draws it.

use core::pin::Pin;

use cxx_qt::CxxQtType;
use cxx_qt_lib::{QByteArray, QHash, QHashPair_i32_QByteArray, QModelIndex, QString, QVariant};
use emusen_kit_core::input::{intent, InputEvent, Intent, Key, UiButton};
use emusen_kit_core::keyboard::{Keyboard, KeyKind, Layout, Outcome, Special, DOCKED};

use crate::qt_keys;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!(<QtCore/QAbstractListModel>);
        type QAbstractListModel = crate::base::ffi::QAbstractListModel;
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
    }

    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[base = QAbstractListModel]
        #[qml_element]
        #[qproperty(QString, text, READ, NOTIFY)]
        #[qproperty(i32, caret, READ, NOTIFY)]
        #[qproperty(i32, current, READ, NOTIFY)]
        #[qproperty(bool, shifted, READ, NOTIFY)]
        #[qproperty(f64, band_height, READ, NOTIFY)]
        #[qproperty(QString, layouts)]
        type KeyboardModel = super::KeyboardModelRust;
    }

    extern "RustQt" {
        #[qinvokable]
        #[cxx_override]
        fn data(self: &KeyboardModel, index: &QModelIndex, role: i32) -> QVariant;
        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &KeyboardModel) -> QHash_i32_QByteArray;
        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &KeyboardModel, parent: &QModelIndex) -> i32;
    }

    unsafe extern "RustQt" {
        #[inherit]
        #[cxx_name = "beginResetModel"]
        fn begin_reset_model(self: Pin<&mut KeyboardModel>);
        #[inherit]
        #[cxx_name = "endResetModel"]
        fn end_reset_model(self: Pin<&mut KeyboardModel>);
        #[inherit]
        #[cxx_name = "dataChanged"]
        fn data_changed(self: Pin<&mut KeyboardModel>, top_left: &QModelIndex, bottom_right: &QModelIndex);
        #[inherit]
        fn index(self: &KeyboardModel, row: i32, column: i32, parent: &QModelIndex) -> QModelIndex;

        /// The keyboard closed: kept for Done, not kept for B on an empty field.
        #[qsignal]
        fn closed(self: Pin<&mut KeyboardModel>, kept: bool);

        /// A key the keyboard's item declined to its ancestors, or a key typed: the arrows walk the keys, Return types the
        /// current one, Escape is B, Backspace erases, and printable text goes in at the caret; true if used.
        #[qinvokable]
        fn key(self: Pin<&mut KeyboardModel>, key: i32, modifiers: i32, text: &QString) -> bool;

        /// A declared action of the host's frame: `space`, `done` or `erase` (EmuSen_Interface.md §18.1's Y and X).
        #[qinvokable]
        fn act(self: Pin<&mut KeyboardModel>, action: &QString) -> bool;

        /// The pointer rests on a key (its index; -1 when it leaves, which changes nothing): it becomes the current one.
        #[qinvokable]
        fn point(self: Pin<&mut KeyboardModel>, index: i32);

        /// A click on a key: it becomes current and is typed, as A there.
        #[qinvokable]
        fn click(self: Pin<&mut KeyboardModel>, index: i32);

        /// The field's own text and caret, as a physical keyboard typed them, or the host's starting words.
        #[qinvokable]
        #[cxx_name = "setText"]
        fn set_text(self: Pin<&mut KeyboardModel>, text: &QString, caret: i32);
    }

    impl cxx_qt::Initialize for KeyboardModel {}
}

/// A key of the layout shown: its row and column, and its box in design units, x from the band's centre.
type LaidKey = (usize, usize, (f64, f64, f64, f64));

pub struct KeyboardModelRust {
    text: QString,
    caret: i32,
    current: i32,
    shifted: bool,
    band_height: f64,
    layouts: QString,
    keyboard: Keyboard,
    /// The name of the layout the keys were laid out for.
    laid: String,
    keys: Vec<LaidKey>,
}

impl Default for KeyboardModelRust {
    fn default() -> Self {
        let mut rust = KeyboardModelRust {
            text: QString::default(),
            caret: 0,
            current: 0,
            shifted: false,
            band_height: 0.0,
            layouts: QString::default(),
            keyboard: Keyboard::search(),
            laid: String::new(),
            keys: Vec::new(),
        };
        rust.lay_out();
        rust
    }
}

impl KeyboardModelRust {
    fn lay_out(&mut self) {
        let layout = self.keyboard.layout();
        self.keys = (0..layout.rows.len())
            .flat_map(|row| (0..layout.rows[row].len()).map(move |column| (row, column)))
            .map(|(row, column)| (row, column, DOCKED.key_box(layout, row, column)))
            .collect();
        self.band_height = DOCKED.height(layout);
        self.laid = layout.name.clone();
    }
}

const LABEL: i32 = 0;
const X: i32 = 1;
const Y: i32 = 2;
const WIDTH: i32 = 3;
const HEIGHT: i32 = 4;

/// Qt::Key_Backspace, the one key the kit's map has no name for that the keyboard answers.
const KEY_BACKSPACE: i32 = 0x0100_0003;

impl cxx_qt::Initialize for qobject::KeyboardModel {
    fn initialize(self: Pin<&mut Self>) {
        self.on_layouts_changed(|mut model| {
            let lines = model.layouts.to_string();
            let layouts: Vec<Layout> = lines.split("\n\n").filter_map(Layout::parse).collect();
            if !layouts.is_empty() {
                let text = model.keyboard.text();
                let caret = model.keyboard.caret();
                model.as_mut().rust_mut().keyboard = Keyboard::new(layouts);
                model.as_mut().rust_mut().keyboard.set_text(&text, caret);
                model.as_mut().relayout();
            }
        })
        .release();
    }
}

impl qobject::KeyboardModel {
    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let Some(&(row, column, (x, y, w, h))) = self.keys.get(index.row() as usize) else { return QVariant::default() };
        match role {
            LABEL => QVariant::from(&QString::from(self.keyboard.label(row, column).as_str())),
            X => QVariant::from(&x),
            Y => QVariant::from(&y),
            WIDTH => QVariant::from(&w),
            HEIGHT => QVariant::from(&h),
            _ => QVariant::default(),
        }
    }

    pub fn role_names(&self) -> QHash<QHashPair_i32_QByteArray> {
        let mut roles = QHash::<QHashPair_i32_QByteArray>::default();
        for (role, name) in [(LABEL, "label"), (X, "keyX"), (Y, "keyY"), (WIDTH, "keyWidth"), (HEIGHT, "keyHeight")] {
            roles.insert(role, QByteArray::from(name));
        }
        roles
    }

    pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
        self.keys.len() as i32
    }

    pub fn key(mut self: Pin<&mut Self>, key: i32, modifiers: i32, text: &QString) -> bool {
        if key == KEY_BACKSPACE {
            let outcome = self.as_mut().rust_mut().keyboard.type_key(&KeyKind::Special(Special::Erase));
            return self.answer(outcome);
        }
        let (kit_key, shift) = qt_keys::kit_key(key, modifiers);
        if kit_key != Key::Other {
            return match intent(InputEvent::Key { key: kit_key, shift }) {
                Some(Intent::Press(button)) => self.press(button),
                _ => false,
            };
        }
        let typed = text.to_string();
        if typed.is_empty() || typed.chars().any(char::is_control) {
            return false;
        }
        let outcome = self.as_mut().rust_mut().keyboard.type_key(&KeyKind::Chars(typed));
        self.answer(outcome)
    }

    fn press(mut self: Pin<&mut Self>, button: UiButton) -> bool {
        let outcome = match button {
            UiButton::Up => self.as_mut().rust_mut().keyboard.move_by(0, -1),
            UiButton::Down => self.as_mut().rust_mut().keyboard.move_by(0, 1),
            UiButton::Left => self.as_mut().rust_mut().keyboard.move_by(-1, 0),
            UiButton::Right => self.as_mut().rust_mut().keyboard.move_by(1, 0),
            UiButton::Accept => self.as_mut().rust_mut().keyboard.press(),
            UiButton::Back => self.as_mut().rust_mut().keyboard.back(),
            _ => return false,
        };
        self.answer(outcome);
        true
    }

    pub fn act(mut self: Pin<&mut Self>, action: &QString) -> bool {
        let special = match action.to_string().as_str() {
            "space" => Special::Space,
            "done" => Special::Done,
            "erase" => Special::Erase,
            _ => return false,
        };
        let outcome = self.as_mut().rust_mut().keyboard.type_key(&KeyKind::Special(special));
        self.answer(outcome);
        true
    }

    pub fn point(mut self: Pin<&mut Self>, index: i32) {
        let Some(&(row, column, _)) = usize::try_from(index).ok().and_then(|at| self.keys.get(at)) else { return };
        let outcome = self.as_mut().rust_mut().keyboard.point(row, column);
        self.answer(outcome);
    }

    pub fn click(mut self: Pin<&mut Self>, index: i32) {
        self.as_mut().point(index);
        let outcome = self.as_mut().rust_mut().keyboard.press();
        self.answer(outcome);
    }

    pub fn set_text(mut self: Pin<&mut Self>, text: &QString, caret: i32) {
        self.as_mut().rust_mut().keyboard.set_text(&text.to_string(), caret.max(0) as usize);
        self.answer(Outcome::Typed);
    }

    /// Publishes what a press changed and says whether anything did.
    fn answer(mut self: Pin<&mut Self>, outcome: Outcome) -> bool {
        match outcome {
            Outcome::Nothing => return false,
            Outcome::Closed { kept } => self.as_mut().closed(kept),
            Outcome::Moved | Outcome::Typed => {}
        }
        self.as_mut().publish();
        true
    }

    /// The keys laid out afresh for the layout shown, then the properties.
    fn relayout(mut self: Pin<&mut Self>) {
        self.as_mut().begin_reset_model();
        self.as_mut().rust_mut().lay_out();
        self.as_mut().end_reset_model();
        self.as_mut().band_height_changed();
        self.as_mut().publish();
    }

    /// Each property that changed announced; a new layout laid out, a turned shift relabelling every key.
    fn publish(mut self: Pin<&mut Self>) {
        if self.keyboard.layout().name != self.laid {
            return self.relayout();
        }
        let text = QString::from(self.keyboard.text().as_str());
        if text != self.text {
            self.as_mut().rust_mut().text = text;
            self.as_mut().text_changed();
        }
        let caret = self.keyboard.caret() as i32;
        if caret != self.caret {
            self.as_mut().rust_mut().caret = caret;
            self.as_mut().caret_changed();
        }
        let (row, column) = self.keyboard.current();
        let current = self.keys.iter().position(|&(r, c, _)| (r, c) == (row, column)).unwrap_or(0) as i32;
        if current != self.current {
            self.as_mut().rust_mut().current = current;
            self.as_mut().current_changed();
        }
        if self.keyboard.shifted() != self.shifted {
            let shifted = self.keyboard.shifted();
            self.as_mut().rust_mut().shifted = shifted;
            self.as_mut().shifted_changed();
            let last = self.keys.len() as i32 - 1;
            let root = QModelIndex::default();
            let (top, bottom) = (self.index(0, 0, &root), self.index(last, 0, &root));
            self.as_mut().data_changed(&top, &bottom);
        }
    }
}
