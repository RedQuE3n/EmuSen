//! `MarkShapes`: one of the scene's built-in marks as a list model of SVG paths in pixels, one row per part, that
//! `Mark.qml` draws with Qt Quick Shapes as the kit's `Body.qml` draws kit-core's shapes (EmuSen_Qt.md §4.8).

use core::pin::Pin;
use std::fmt::Write;

use cxx_qt::CxxQtType;
use cxx_qt_lib::{QColor, QHash, QHashPair_i32_QByteArray, QModelIndex, QString, QVariant};
use emusen_esde::marks::{MarkId, Part};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!(<QtCore/QAbstractListModel>);
        type QAbstractListModel = emusen_kit_qt::base::ffi::QAbstractListModel;
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
        include!("cxx-qt-lib/qcolor.h");
        type QColor = cxx_qt_lib::QColor;
    }

    #[qenum(MarkShapes)]
    enum MarkRole {
        Outline,
        FillInk,
        StrokeInk,
        StrokeSize,
    }

    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[base = QAbstractListModel]
        #[qml_element]
        #[qproperty(QString, mark)]
        #[qproperty(f64, width)]
        #[qproperty(f64, height)]
        #[qproperty(QColor, color)]
        /// The mark's width over its height, and the share of its height down to the baseline it sits on beside words.
        #[qproperty(f64, aspect, READ, NOTIFY)]
        #[qproperty(f64, baseline, READ, NOTIFY)]
        type MarkShapes = super::MarkShapesRust;
    }

    extern "RustQt" {
        #[qinvokable]
        #[cxx_override]
        fn data(self: &MarkShapes, index: &QModelIndex, role: i32) -> QVariant;
        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &MarkShapes) -> QHash_i32_QByteArray;
        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &MarkShapes, parent: &QModelIndex) -> i32;
    }

    unsafe extern "RustQt" {
        #[inherit]
        #[cxx_name = "beginResetModel"]
        fn begin_reset_model(self: Pin<&mut MarkShapes>);
        #[inherit]
        #[cxx_name = "endResetModel"]
        fn end_reset_model(self: Pin<&mut MarkShapes>);
    }

    impl cxx_qt::Initialize for MarkShapes {}
}

use qobject::MarkRole;

const ROLE_NAMES: [(MarkRole, &str); 4] =
    [(MarkRole::Outline, "outline"), (MarkRole::FillInk, "fillInk"), (MarkRole::StrokeInk, "strokeInk"), (MarkRole::StrokeSize, "strokeSize")];

/// One part as drawn: its rings as one SVG path in pixels (holes filled even-odd), filled or stroked at a width in pixels.
pub struct MarkRow {
    pub outline: String,
    pub fill: bool,
    pub stroke: f64,
}

#[derive(Default)]
pub struct MarkShapesRust {
    /// The mark's name as `MarkId::name` gives it; an unknown name draws nothing.
    mark: QString,
    width: f64,
    height: f64,
    color: QColor,
    aspect: f64,
    baseline: f64,
    rows: Vec<MarkRow>,
}

/// The mark of this name, if the scene knows one.
pub fn mark_named(name: &str) -> Option<MarkId> {
    MarkId::ALL.into_iter().find(|m| m.name() == name)
}

/// A part's rings scaled from the unit box to `width` × `height` pixels, as an SVG path.
pub fn row_of(part: &Part, width: f64, height: f64) -> MarkRow {
    let mut outline = String::new();
    for ring in &part.rings {
        for (i, p) in ring.iter().enumerate() {
            let _ = write!(outline, "{}{:.3} {:.3} ", if i == 0 { "M" } else { "L" }, p[0] * width, p[1] * height);
        }
        if part.closed && !ring.is_empty() {
            outline.push_str("Z ");
        }
    }
    MarkRow { outline, fill: part.fill, stroke: part.stroke * height }
}

impl MarkShapesRust {
    fn rebuild(&mut self) {
        let mark = mark_named(&self.mark.to_string()).map(MarkId::mark);
        (self.aspect, self.baseline) = mark.as_ref().map_or((1.0, 1.0), |m| (m.aspect, m.baseline));
        self.rows = match mark {
            Some(m) if self.width > 0.0 && self.height > 0.0 => m.parts.iter().map(|p| row_of(p, self.width, self.height)).collect(),
            _ => Vec::new(),
        };
    }
}

impl cxx_qt::Initialize for qobject::MarkShapes {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut().on_mark_changed(|this| this.refresh()).release();
        self.as_mut().on_width_changed(|this| this.refresh()).release();
        self.as_mut().on_height_changed(|this| this.refresh()).release();
        self.as_mut().on_color_changed(|this| this.refresh()).release();
    }
}

impl qobject::MarkShapes {
    fn refresh(mut self: Pin<&mut Self>) {
        let was = (self.aspect, self.baseline);
        self.as_mut().begin_reset_model();
        self.as_mut().rust_mut().rebuild();
        self.as_mut().end_reset_model();
        // Announced only when the mark's shape changes, never for a new size, so a width bound to the aspect reads no loop.
        if (self.aspect, self.baseline) != was {
            self.as_mut().aspect_changed();
            self.as_mut().baseline_changed();
        }
    }

    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let Some(row) = self.rows.get(index.row() as usize) else {
            return QVariant::default();
        };
        let clear = QColor::from_rgba(0, 0, 0, 0);
        match (MarkRole { repr: role }) {
            MarkRole::Outline => QVariant::from(&QString::from(row.outline.as_str())),
            MarkRole::FillInk => QVariant::from(if row.fill { &self.color } else { &clear }),
            MarkRole::StrokeInk => QVariant::from(if row.stroke > 0.0 { &self.color } else { &clear }),
            MarkRole::StrokeSize => QVariant::from(&row.stroke),
            _ => QVariant::default(),
        }
    }

    pub fn role_names(&self) -> QHash<QHashPair_i32_QByteArray> {
        emusen_kit_qt::convert::role_names(ROLE_NAMES.map(|(role, name)| (role.repr, name)))
    }

    pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
        self.rows.len() as i32
    }
}
