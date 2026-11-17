//! `MeasuredWalk`: kit-core's walk over rows of their own heights on Qt (EmuSen_Kit.md §7.122).

use core::pin::Pin;

use cxx_qt::CxxQtType;
use emusen_kit_core::settings::MeasuredWalk;

use crate::kinds::{self, PadButton};
use crate::qt_keys;
use crate::settings::{tell, Told};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
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
        /// The window's height and the space between two rows, in the screen's own units; a window of 0 shows every row.
        #[qproperty(f64, window)]
        #[qproperty(f64, gap)]
        #[qproperty(i32, sections)]
        #[qproperty(i32, section)]
        #[qproperty(bool, trail_shown)]
        #[qproperty(i32, count, READ, NOTIFY = walked)]
        #[qproperty(i32, current, READ, NOTIFY = walked)]
        #[qproperty(i32, first, READ, NOTIFY = walked)]
        /// How far the first row shown stands below the first row: the list's scroll.
        #[qproperty(f64, offset, READ, NOTIFY = walked)]
        #[qproperty(i32, marked, READ, NOTIFY = walked)]
        type MeasuredWalk = super::MeasuredWalkRust;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        /// The focus, the window or the mark moved, or the rows were read again.
        #[qsignal]
        fn walked(self: Pin<&mut MeasuredWalk>);
        /// Left (-1) or Right (+1) stepped a value or a slider.
        #[qsignal]
        fn stepped(self: Pin<&mut MeasuredWalk>, row: i32, by: i32);
        /// A shoulder stepped to a section; never raised when the host sets `section`.
        #[qsignal]
        fn section_chosen(self: Pin<&mut MeasuredWalk>, section: i32);
        /// Left or Right moved a slider to an amount, on its grid and inside its range.
        #[qsignal]
        fn slid(self: Pin<&mut MeasuredWalk>, row: i32, amount: f64);
        /// A chose a row.
        #[qsignal]
        fn chosen(self: Pin<&mut MeasuredWalk>, row: i32);

        /// The model's rows read again, the focus kept where it can be.
        #[qinvokable]
        fn read(self: Pin<&mut MeasuredWalk>);
        /// A row's height as drawn, told by its delegate whenever it changes.
        #[qinvokable]
        fn measure(self: Pin<&mut MeasuredWalk>, row: i32, height: f64);
        /// A press in the rows' grammar; false where it is the screen's or the host's.
        #[qinvokable]
        fn press(self: Pin<&mut MeasuredWalk>, button: PadButton) -> bool;
        /// A key as the press it means.
        #[qinvokable]
        fn key(self: Pin<&mut MeasuredWalk>, key: i32, modifiers: i32) -> bool;
        /// The pointer rests on a row.
        #[qinvokable]
        fn focus_row(self: Pin<&mut MeasuredWalk>, row: i32);
        /// The wheel moved the window to start at a row.
        #[qinvokable]
        fn scroll_to(self: Pin<&mut MeasuredWalk>, first: i32);
    }

    impl cxx_qt::Initialize for MeasuredWalk {}
}

pub struct MeasuredWalkRust {
    model: *mut qobject::QAbstractItemModel,
    window: f64,
    gap: f64,
    sections: i32,
    section: i32,
    trail_shown: bool,
    count: i32,
    current: i32,
    first: i32,
    offset: f64,
    marked: i32,
    walk: MeasuredWalk,
}

impl Default for MeasuredWalkRust {
    fn default() -> Self {
        Self {
            model: core::ptr::null_mut(),
            window: 0.0,
            gap: 0.0,
            sections: 0,
            section: 0,
            trail_shown: false,
            count: 0,
            current: 0,
            first: 0,
            offset: 0.0,
            marked: -1,
            walk: MeasuredWalk::new(),
        }
    }
}

impl cxx_qt::Initialize for qobject::MeasuredWalk {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut().on_model_changed(|this| this.take_rows(false)).release();
        self.as_mut()
            .on_window_changed(|mut this| {
                let window = this.window;
                this.as_mut().rust_mut().walk.set_window(window);
                this.sync();
            })
            .release();
        self.as_mut()
            .on_gap_changed(|mut this| {
                let gap = this.gap;
                this.as_mut().rust_mut().walk.set_gap(gap);
                this.sync();
            })
            .release();
    }
}

impl qobject::MeasuredWalk {
    pub fn read(self: Pin<&mut Self>) {
        self.take_rows(true);
    }

    pub fn measure(mut self: Pin<&mut Self>, row: i32, height: f64) {
        let Ok(row) = usize::try_from(row) else { return };
        self.as_mut().rust_mut().walk.set_height(row, height);
        self.sync();
    }

    pub fn press(mut self: Pin<&mut Self>, button: PadButton) -> bool {
        let Some(ui) = kinds::ui_button(button) else { return false };
        // The sections and the trail as the screen has them now, read at each press.
        let (sections, section, trail) = (usize::try_from(self.sections).unwrap_or(0), usize::try_from(self.section).unwrap_or(0), self.trail_shown);
        let mut me = self.as_mut().rust_mut();
        me.walk.set_sections(sections);
        me.walk.select_section(section);
        me.walk.set_trail(trail);
        let outcome = me.walk.press(ui);
        tell(self, outcome)
    }

    pub fn key(self: Pin<&mut Self>, key: i32, modifiers: i32) -> bool {
        qt_keys::press_of(key, modifiers).is_some_and(|button| self.press(kinds::pad_button(button)))
    }

    pub fn focus_row(mut self: Pin<&mut Self>, row: i32) {
        let moved = usize::try_from(row).is_ok_and(|row| self.as_mut().rust_mut().walk.focus(row));
        if moved {
            self.sync();
        }
    }

    pub fn scroll_to(mut self: Pin<&mut Self>, first: i32) {
        let moved = self.as_mut().rust_mut().walk.scroll_to(usize::try_from(first).unwrap_or(0));
        if moved {
            self.sync();
        }
    }

    fn take_rows(mut self: Pin<&mut Self>, keep: bool) {
        // SAFETY: QML hands the pointer of a live model, cleared by the engine when the model is destroyed.
        let rows = crate::settings::walk_rows(unsafe { self.model.as_ref() });
        self.as_mut().rust_mut().walk.set_rows(rows, keep);
        self.sync();
    }

    /// The walk's section where a shoulder stepped it.
    fn sync_section(self: Pin<&mut Self>) {
        let section = self.walk.section() as i32;
        if self.section != section {
            self.set_section(section);
        }
    }

    /// Mirrors the walk into the read-only properties.
    fn sync(mut self: Pin<&mut Self>) {
        let w = &self.walk;
        let (count, current, first, offset, marked) =
            (w.rows().len() as i32, w.current() as i32, w.first() as i32, w.offset(), w.marked().map_or(-1, |m| m as i32));
        let mut r = self.as_mut().rust_mut();
        r.count = count;
        r.current = current;
        r.first = first;
        r.offset = offset;
        r.marked = marked;
        self.walked();
    }
}

impl Told for qobject::MeasuredWalk {
    fn sync(self: Pin<&mut Self>) {
        qobject::MeasuredWalk::sync(self)
    }
    fn sync_section(self: Pin<&mut Self>) {
        qobject::MeasuredWalk::sync_section(self)
    }
    fn stepped(self: Pin<&mut Self>, row: i32, by: i32) {
        qobject::MeasuredWalk::stepped(self, row, by)
    }
    fn slid(self: Pin<&mut Self>, row: i32, amount: f64) {
        qobject::MeasuredWalk::slid(self, row, amount)
    }
    fn chosen(self: Pin<&mut Self>, row: i32) {
        qobject::MeasuredWalk::chosen(self, row)
    }
    fn section_chosen(self: Pin<&mut Self>, section: i32) {
        qobject::MeasuredWalk::section_chosen(self, section)
    }
}
