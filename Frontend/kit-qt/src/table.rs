//! `ColumnFit`: a table's columns fitted to its width from any list model, kit-core's FitsWidth on Qt; the words are
//! measured in the cells' own font (EmuSen_Kit.md §7.39).

use core::pin::Pin;

use cxx_qt::CxxQtType;
use cxx_qt_lib::{QFont, QList, QString, QVariant};
use emusen_kit_core::table::{fit_columns, ColumnWidth, TableMeasures};

use crate::item_model::role_texts;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qfont.h");
        type QFont = cxx_qt_lib::QFont;
        include!("text_width.h");
        fn text_width(font: &QFont, text: &QString) -> f64;
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
        #[qproperty(f64, width)]
        #[qproperty(*mut QAbstractItemModel, rows)]
        #[qproperty(*mut QAbstractItemModel, columns)]
        #[qproperty(QFont, font)]
        #[qproperty(QVariant, xs, READ, NOTIFY = fitted)]
        #[qproperty(QVariant, widths, READ, NOTIFY = fitted)]
        type ColumnFit = super::ColumnFitRust;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        /// The columns' places changed together.
        #[qsignal]
        fn fitted(self: Pin<&mut ColumnFit>);

        /// The rows' words measured again, for a screen to call when the rows or the columns change.
        #[qinvokable]
        fn measure(self: Pin<&mut ColumnFit>);
    }

    impl cxx_qt::Initialize for ColumnFit {}
}

fn list(values: Vec<f64>) -> QVariant {
    QVariant::from(&QList::from(&values))
}

/// A table's width, its rows and its columns (roles `key`, `fill`, `least`), the font its cells are drawn in, and each
/// column's place from the table's left, as lists QML reads by a column's index.
pub struct ColumnFitRust {
    width: f64,
    rows: *mut qobject::QAbstractItemModel,
    columns: *mut qobject::QAbstractItemModel,
    font: QFont,
    measured: Vec<ColumnWidth>,
    xs: QVariant,
    widths: QVariant,
}

impl Default for ColumnFitRust {
    fn default() -> Self {
        Self {
            width: 0.0,
            rows: core::ptr::null_mut(),
            columns: core::ptr::null_mut(),
            font: QFont::default(),
            measured: Vec::new(),
            xs: QVariant::default(),
            widths: QVariant::default(),
        }
    }
}

impl cxx_qt::Initialize for qobject::ColumnFit {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut().on_width_changed(|this| this.fit()).release();
        self.as_mut().on_rows_changed(|this| this.measure()).release();
        self.as_mut().on_columns_changed(|this| this.measure()).release();
        self.as_mut().on_font_changed(|this| this.measure()).release();
    }
}

/// A column's number from a model's text, nothing where it is not one.
fn number(text: Option<&String>) -> f64 {
    text.and_then(|t| t.parse().ok()).filter(|n: &f64| n.is_finite()).unwrap_or(0.0)
}

impl qobject::ColumnFit {
    pub fn measure(mut self: Pin<&mut Self>) {
        // SAFETY: QML hands the pointers of live models, cleared by the engine when a model is destroyed.
        let (rows, columns) = unsafe { (self.rows.as_ref(), self.columns.as_ref()) };
        let keys = role_texts(columns, &QString::from("key"));
        let fills = role_texts(columns, &QString::from("fill"));
        let leasts = role_texts(columns, &QString::from("least"));
        let measured = keys
            .iter()
            .enumerate()
            .map(|(i, key)| {
                let weight = number(fills.get(i));
                if weight > 0.0 {
                    ColumnWidth::Fill { weight, least: number(leasts.get(i)) }
                } else {
                    let words = role_texts(rows, &QString::from(key.as_str()));
                    ColumnWidth::Content(
                        words.iter().map(|w| qobject::text_width(&self.font, &QString::from(w.as_str()))).fold(0.0, f64::max),
                    )
                }
            })
            .collect();
        self.as_mut().rust_mut().measured = measured;
        self.fit();
    }

    fn fit(mut self: Pin<&mut Self>) {
        let placed = fit_columns(self.width, &self.measured, &TableMeasures::BIG_SCREEN);
        let mut r = self.as_mut().rust_mut();
        r.xs = list(placed.iter().map(|c| c.x).collect());
        r.widths = list(placed.iter().map(|c| c.width).collect());
        self.fitted();
    }
}
