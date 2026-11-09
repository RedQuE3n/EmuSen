//! `SettingsLayout`: kit-core's settings frame on Qt, its measures and the walk over a page's rows read from any list
//! model's `kind`, `held`, `marked`, `sentence` and a slider's `minimum`, `maximum`, `step` and `amount` roles; `Kit.RowKinds` numbers the kinds (EmuSen_Kit.md §7.40–§7.42).

use core::pin::Pin;

use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use emusen_kit_core::settings::{Row, RowKind, RowOutcome, RowWalk, SETTINGS, SliderRange};
use emusen_kit_core::tokens::{Form, spacing};

use crate::kinds::{self, PadButton};
use crate::qt_keys;

#[cxx_qt::bridge]
pub mod qobject {
    #[qml_element]
    qnamespace!("RowKinds");

    /// What a settings row does: kit-core's `RowKind`, in its order.
    #[qenum]
    #[namespace = "RowKinds"]
    enum RowKindNumber {
        Action,
        Steps,
        Opens,
        Asks,
        Slider,
        Choice,
    }

    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qmodelindex.h");
        type QModelIndex = cxx_qt_lib::QModelIndex;
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
        include!("cxx-qt-lib/qhash.h");
        type QHash_i32_QByteArray = cxx_qt_lib::QHash<cxx_qt_lib::QHashPair_i32_QByteArray>;
        include!("emusen-kit-qt/src/kinds.cxxqt.h");
        #[namespace = "Pad"]
        type PadButton = crate::kinds::ffi::PadButton;
    }

    unsafe extern "C++" {
        // The model is declared once, in item_model.rs: cxx-qt emits an upcast shim per declaration (EmuSen_Kit.md §7.6).
        include!(<QtCore/QAbstractItemModel>);
        type QAbstractItemModel = crate::item_model::ffi::QAbstractItemModel;
    }

    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(*mut QAbstractItemModel, model)]
        #[qproperty(f64, width)]
        #[qproperty(i32, sections)]
        #[qproperty(i32, section)]
        #[qproperty(bool, trail_shown)]
        #[qproperty(f64, side, READ, CONSTANT)]
        #[qproperty(f64, strip_top, READ, CONSTANT)]
        #[qproperty(f64, strip_height, READ, CONSTANT)]
        #[qproperty(f64, strip_gap, READ, CONSTANT)]
        #[qproperty(f64, tab_padding, READ, CONSTANT)]
        #[qproperty(f64, content_top, READ, CONSTANT)]
        #[qproperty(f64, pane_width, READ, CONSTANT)]
        #[qproperty(f64, pane_height, READ, CONSTANT)]
        #[qproperty(f64, pane_inset, READ, CONSTANT)]
        #[qproperty(f64, slot_gap, READ, CONSTANT)]
        #[qproperty(f64, line_gap, READ, CONSTANT)]
        #[qproperty(i32, rows_shown, READ, CONSTANT)]
        #[qproperty(f64, pane_left, READ, NOTIFY = laid_out)]
        #[qproperty(f64, rows_width, READ, NOTIFY = laid_out)]
        #[qproperty(i32, count, READ, NOTIFY = walked)]
        #[qproperty(i32, current, READ, NOTIFY = walked)]
        #[qproperty(i32, first, READ, NOTIFY = walked)]
        #[qproperty(i32, marked, READ, NOTIFY = walked)]
        #[qproperty(QString, sentence, READ, NOTIFY = walked)]
        type SettingsLayout = super::SettingsLayoutRust;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        /// The pane's left edge and the rows' width changed with the canvas's width.
        #[qsignal]
        fn laid_out(self: Pin<&mut SettingsLayout>);
        /// The focus, the window or the mark moved, or the rows were read again.
        #[qsignal]
        fn walked(self: Pin<&mut SettingsLayout>);
        /// Left (-1) or Right (+1) stepped a value or a slider.
        #[qsignal]
        fn stepped(self: Pin<&mut SettingsLayout>, row: i32, by: i32);
        /// A shoulder stepped the strip to a section; never raised when the host sets `section`.
        #[qsignal]
        fn section_chosen(self: Pin<&mut SettingsLayout>, section: i32);
        /// Left or Right moved a slider to an amount, on its grid and inside its range.
        #[qsignal]
        fn slid(self: Pin<&mut SettingsLayout>, row: i32, amount: f64);
        /// A chose a row: the host acts, opens its page, asks, or takes the choice.
        #[qsignal]
        fn chosen(self: Pin<&mut SettingsLayout>, row: i32);

        /// The model's rows read again, the focus kept where it can be; for a host to call when its rows change.
        #[qinvokable]
        fn read(self: Pin<&mut SettingsLayout>);
        /// A press in the rows' grammar; false where it is the frame's or the host's.
        #[qinvokable]
        fn press(self: Pin<&mut SettingsLayout>, button: PadButton) -> bool;
        /// A key the focused row declined, as the press it means.
        #[qinvokable]
        fn key(self: Pin<&mut SettingsLayout>, key: i32, modifiers: i32) -> bool;
        /// The pointer rests on a row.
        #[qinvokable]
        fn focus_row(self: Pin<&mut SettingsLayout>, row: i32);
        /// The wheel moved the window to start at a row.
        #[qinvokable]
        fn scroll_to(self: Pin<&mut SettingsLayout>, first: i32);
    }

    impl cxx_qt::Initialize for SettingsLayout {}
}

pub struct SettingsLayoutRust {
    model: *mut qobject::QAbstractItemModel,
    width: f64,
    sections: i32,
    section: i32,
    trail_shown: bool,
    side: f64,
    strip_top: f64,
    strip_height: f64,
    strip_gap: f64,
    tab_padding: f64,
    content_top: f64,
    pane_width: f64,
    pane_height: f64,
    pane_inset: f64,
    slot_gap: f64,
    line_gap: f64,
    rows_shown: i32,
    pane_left: f64,
    rows_width: f64,
    count: i32,
    current: i32,
    first: i32,
    marked: i32,
    sentence: QString,
    sentences: Vec<String>,
    walk: RowWalk,
}

impl Default for SettingsLayoutRust {
    fn default() -> Self {
        let g = SETTINGS;
        let shown = g.rows_shown(spacing(Form::BigScreen).row_height);
        Self {
            model: core::ptr::null_mut(),
            width: 0.0,
            sections: 0,
            section: 0,
            trail_shown: false,
            side: g.side,
            strip_top: g.strip_top,
            strip_height: g.strip_height,
            strip_gap: g.strip_gap,
            tab_padding: g.tab_padding,
            content_top: g.content_top,
            pane_width: g.pane_width,
            pane_height: g.pane_height,
            pane_inset: g.pane_inset,
            slot_gap: g.slot_gap,
            line_gap: g.line_gap,
            rows_shown: shown as i32,
            pane_left: 0.0,
            rows_width: 0.0,
            count: 0,
            current: 0,
            first: 0,
            marked: -1,
            sentence: QString::default(),
            sentences: Vec::new(),
            walk: RowWalk::new(shown),
        }
    }
}

impl cxx_qt::Initialize for qobject::SettingsLayout {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut().on_width_changed(|this| this.lay_out()).release();
        self.as_mut().on_model_changed(|this| this.take_rows(false)).release();
        self.as_mut()
            .on_sections_changed(|mut this| {
                let count = usize::try_from(this.sections).unwrap_or(0);
                let wanted = usize::try_from(this.section).unwrap_or(0);
                let mut r = this.as_mut().rust_mut();
                r.walk.set_sections(count);
                r.walk.select_section(wanted);
                this.sync_section();
            })
            .release();
        self.as_mut()
            .on_section_changed(|mut this| {
                let section = usize::try_from(this.section).unwrap_or(usize::MAX);
                this.as_mut().rust_mut().walk.select_section(section);
            })
            .release();
        self.as_mut()
            .on_trail_shown_changed(|mut this| {
                let open = this.trail_shown;
                this.as_mut().rust_mut().walk.set_trail(open);
            })
            .release();
    }
}

/// A role's text as a whole number; a role a model lacks, or words, are 0.
fn number(text: &str) -> i64 {
    text.trim().parse::<f64>().map(|n| n.round() as i64).unwrap_or(0)
}

fn flag(text: &str) -> bool {
    matches!(text.trim(), "true" | "1")
}

impl qobject::SettingsLayout {
    pub fn read(self: Pin<&mut Self>) {
        self.take_rows(true);
    }

    pub fn press(mut self: Pin<&mut Self>, button: PadButton) -> bool {
        let Some(ui) = kinds::ui_button(button) else { return false };
        let outcome = self.as_mut().rust_mut().walk.press(ui);
        match outcome {
            RowOutcome::Moved(_) => self.as_mut().sync(),
            RowOutcome::Stepped { row, by } => self.as_mut().stepped(row as i32, by),
            RowOutcome::Slid { row, amount } => self.as_mut().slid(row as i32, amount),
            RowOutcome::Chosen(row) => {
                self.as_mut().sync();
                self.as_mut().chosen(row as i32);
            }
            RowOutcome::Section(section) => {
                self.as_mut().sync_section();
                self.as_mut().section_chosen(section as i32);
            }
            RowOutcome::Kept => {}
            RowOutcome::Declined => return false,
        }
        true
    }

    pub fn key(self: Pin<&mut Self>, key: i32, modifiers: i32) -> bool {
        match qt_keys::press_of(key, modifiers) {
            Some(button) => self.press(kinds::pad_button(button)),
            None => false,
        }
    }

    pub fn focus_row(mut self: Pin<&mut Self>, row: i32) {
        let Ok(row) = usize::try_from(row) else { return };
        if self.as_mut().rust_mut().walk.focus(row) {
            self.sync();
        }
    }

    pub fn scroll_to(mut self: Pin<&mut Self>, first: i32) {
        if self.as_mut().rust_mut().walk.scroll_to(usize::try_from(first).unwrap_or(0)) {
            self.sync();
        }
    }

    fn take_rows(mut self: Pin<&mut Self>, keep: bool) {
        // SAFETY: QML hands the pointer of a live model, cleared by the engine when the model is destroyed.
        let model = unsafe { self.model.as_ref() };
        let role = |name: &str| crate::item_model::role_texts(model, &QString::from(name));
        let (kinds, held, marked) = (role("kind"), role("held"), role("marked"));
        let (minimum, maximum, step, amount) = (role("minimum"), role("maximum"), role("step"), role("amount"));
        let real = |list: &[String], i: usize, or: f64| list.get(i).and_then(|t| t.trim().parse::<f64>().ok()).unwrap_or(or);
        self.as_mut().rust_mut().sentences = role("sentence");
        let rows = kinds
            .iter()
            .enumerate()
            .map(|(i, kind)| Row {
                kind: RowKind::from_index(number(kind)),
                held: held.get(i).is_some_and(|t| flag(t)),
                marked: marked.get(i).is_some_and(|t| flag(t)),
                range: SliderRange { minimum: real(&minimum, i, 0.0), maximum: real(&maximum, i, 1.0), step: real(&step, i, 0.0) },
                amount: real(&amount, i, 0.0),
            })
            .collect();
        self.as_mut().rust_mut().walk.set_rows(rows, keep);
        self.sync();
    }

    fn lay_out(mut self: Pin<&mut Self>) {
        let width = self.width;
        let mut r = self.as_mut().rust_mut();
        r.pane_left = SETTINGS.pane_left(width);
        r.rows_width = SETTINGS.rows_width(width);
        self.laid_out();
    }

    /// The walk's section, where the host's differs from it: a section stepped, or one the strip no longer holds.
    fn sync_section(self: Pin<&mut Self>) {
        let section = self.walk.section() as i32;
        if self.section != section {
            self.set_section(section);
        }
    }

    /// Mirrors the walk into the read-only properties.
    fn sync(mut self: Pin<&mut Self>) {
        let (count, current, first, marked) = {
            let w = &self.walk;
            (w.rows().len() as i32, w.current() as i32, w.first() as i32, w.marked().map_or(-1, |m| m as i32))
        };
        let sentence = QString::from(self.sentences.get(current as usize).map_or("", String::as_str));
        let mut r = self.as_mut().rust_mut();
        r.count = count;
        r.current = current;
        r.first = first;
        r.marked = marked;
        r.sentence = sentence;
        self.walked();
    }
}
