//! `PadGlyphMarks`: a pad button's glyph as a list model of kit-core's marks in pixels, one row per `Shape`, that `PadGlyph.qml` draws (EmuSen_Kit.md §7.12).

use core::pin::Pin;

use cxx_qt::CxxQtType;
use cxx_qt_lib::{QHash, QHashPair_i32_QByteArray, QList, QModelIndex, QString, QVariant};
use emusen_kit_core::glyph::{self, Glyph};
use emusen_kit_core::world::Point;
use emusen_kit_core::input::UiButton;

use crate::convert::polygon;
use crate::kinds::{self, Family, GlyphStyle, PadButton};

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
        include!("emusen-kit-qt/src/kinds.cxxqt.h");
        #[namespace = "Families"]
        type Family = crate::kinds::ffi::Family;
        #[namespace = "Pad"]
        type PadButton = crate::kinds::ffi::PadButton;
        #[namespace = "GlyphStyles"]
        type GlyphStyle = crate::kinds::ffi::GlyphStyle;
    }

    #[qenum(PadGlyphMarks)]
    enum MarkRole {
        Outline,
        Filled,
        Stroke,
        CutLetter,
        Holes,
    }

    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[base = QAbstractListModel]
        #[qml_element]
        #[qproperty(Family, family)]
        #[qproperty(PadButton, button)]
        #[qproperty(bool, swap)]
        #[qproperty(f64, size)]
        #[qproperty(QString, letter, READ, NOTIFY)]
        #[qproperty(f64, letter_scale, READ, NOTIFY)]
        #[qproperty(bool, cut, READ, NOTIFY)]
        #[qproperty(QString, name, READ, NOTIFY)]
        /// The outlined set or the filled one (EmuSen_Kit.md §7.28).
        #[qproperty(GlyphStyle, style)]
        #[qproperty(bool, letter_bold, READ, NOTIFY)]
        /// The press drawn with `button` as one glyph where kit-core has one for the two, -1 (the default) for none (EmuSen_Kit.md §7.54).
        #[qproperty(i32, pair)]
        /// Whether the glyph drawn is the pair's one.
        #[qproperty(bool, paired, READ, NOTIFY)]
        type PadGlyphMarks = super::PadGlyphMarksRust;
    }

    extern "RustQt" {
        #[qinvokable]
        #[cxx_override]
        fn data(self: &PadGlyphMarks, index: &QModelIndex, role: i32) -> QVariant;
        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &PadGlyphMarks) -> QHash_i32_QByteArray;
        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &PadGlyphMarks, parent: &QModelIndex) -> i32;
    }

    unsafe extern "RustQt" {
        #[inherit]
        #[cxx_name = "beginResetModel"]
        fn begin_reset_model(self: Pin<&mut PadGlyphMarks>);
        #[inherit]
        #[cxx_name = "endResetModel"]
        fn end_reset_model(self: Pin<&mut PadGlyphMarks>);
    }

    impl cxx_qt::Initialize for PadGlyphMarks {}
}

use qobject::MarkRole;

pub const ROLE_NAMES: [(MarkRole, &str); 5] = [
    (MarkRole::Outline, "outline"),
    (MarkRole::Filled, "filled"),
    (MarkRole::Stroke, "stroke"),
    (MarkRole::CutLetter, "cutLetter"),
    (MarkRole::Holes, "holes"),
];

pub struct PadGlyphMarksRust {
    family: Family,
    button: PadButton,
    swap: bool,
    size: f64,
    letter: QString,
    letter_scale: f64,
    cut: bool,
    name: QString,
    style: GlyphStyle,
    letter_bold: bool,
    pair: i32,
    paired: bool,
    glyph: Glyph,
}

impl Default for PadGlyphMarksRust {
    fn default() -> Self {
        let mut rust = PadGlyphMarksRust {
            family: Family::Generic,
            button: PadButton::Accept,
            swap: false,
            size: 0.0,
            letter: QString::default(),
            letter_scale: 1.0,
            cut: false,
            name: QString::default(),
            style: GlyphStyle::Outlined,
            letter_bold: false,
            pair: -1,
            paired: false,
            glyph: Glyph { marks: Vec::new(), letter: None },
        };
        rust.rebuild();
        rust
    }
}

impl PadGlyphMarksRust {
    /// kit-core's glyph for the control that gives the press, with the player's swap, in the style's set; named as it is lettered.
    fn rebuild(&mut self) {
        let style = kinds::glyph_style(self.style);
        let family = glyph::lettered(kinds::pad_family(self.family), style);
        let control = kinds::ui_button(self.button).unwrap_or(UiButton::Guide).control(self.swap);
        let second = kinds::ui_button(PadButton { repr: self.pair }).map(|b| b.control(self.swap));
        let paired = second.and_then(|b| glyph::paired(control, b, style).map(|g| (g, b)));
        self.glyph = paired.as_ref().map_or_else(|| glyph::styled(family, control, style), |(g, _)| g.clone());
        self.paired = paired.is_some();
        let letter = self.glyph.letter;
        self.letter = QString::from(letter.map_or("", |l| l.text));
        self.letter_scale = letter.map_or(1.0, |l| l.scale);
        self.letter_bold = letter.is_some_and(|l| l.bold);
        self.cut = self.glyph.marks.iter().any(|m| m.cut);
        self.name = QString::from(&match paired {
            Some((_, b)) => format!("{} and {}", glyph::name(family, control), glyph::name(family, b)),
            None => glyph::name(family, control).to_owned(),
        });
    }
}

impl cxx_qt::Initialize for qobject::PadGlyphMarks {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut().on_family_changed(|this| this.refresh()).release();
        self.as_mut().on_button_changed(|this| this.refresh()).release();
        self.as_mut().on_swap_changed(|this| this.refresh()).release();
        self.as_mut().on_size_changed(|this| this.resize()).release();
        self.as_mut().on_style_changed(|this| this.refresh()).release();
        self.as_mut().on_pair_changed(|this| this.refresh()).release();
    }
}

impl qobject::PadGlyphMarks {
    // Only the outlines scale with the size; the letter and its scale do not, so their signals stay quiet (EmuSen_Kit.md §7.28).
    fn resize(mut self: Pin<&mut Self>) {
        self.as_mut().begin_reset_model();
        self.as_mut().end_reset_model();
    }

    fn refresh(mut self: Pin<&mut Self>) {
        self.as_mut().begin_reset_model();
        self.as_mut().rust_mut().rebuild();
        self.as_mut().end_reset_model();
        self.as_mut().letter_changed();
        self.as_mut().letter_scale_changed();
        self.as_mut().cut_changed();
        self.as_mut().name_changed();
        self.as_mut().letter_bold_changed();
        self.as_mut().paired_changed();
    }

    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let Some(mark) = self.glyph.marks.get(index.row() as usize) else {
            return QVariant::default();
        };
        match (MarkRole { repr: role }) {
            MarkRole::Outline => {
                let points: Vec<[f64; 2]> = mark.points.iter().map(|p| [p[0] * self.size, p[1] * self.size]).collect();
                polygon(&points)
            }
            MarkRole::Filled => QVariant::from(&mark.filled),
            MarkRole::Stroke => QVariant::from(&(mark.stroke * self.size)),
            MarkRole::CutLetter => QVariant::from(&if mark.cut { self.letter.clone() } else { QString::default() }),
            MarkRole::Holes => {
                let mut holes = QList::<QVariant>::default();
                for hole in &mark.holes {
                    let points: Vec<Point> = hole.iter().map(|p| [p[0] * self.size, p[1] * self.size]).collect();
                    holes.append(polygon(&points));
                }
                QVariant::from(&holes)
            }
            _ => QVariant::default(),
        }
    }

    pub fn role_names(&self) -> QHash<QHashPair_i32_QByteArray> {
        crate::convert::role_names(ROLE_NAMES.map(|(role, name)| (role.repr, name)))
    }

    pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
        self.glyph.marks.len() as i32
    }
}
