//! `TextLayout`: a theme's words laid out by `emusen-esde-text`, ES-DE's rules (EmuSen_Qt.md §4.9), as a list model of
//! glyphs at whole-pixel pens that `GlyphText.qml` draws; the face is the theme's file, or the kit's body face.

use core::pin::Pin;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use cxx_qt::CxxQtType;
use cxx_qt_lib::{QHash, QHashPair_i32_QByteArray, QModelIndex, QString, QUrl, QVariant};
use emusen_esde_text::{layout, Align, Face, Layout, Spec};

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
        include!("cxx-qt-lib/qurl.h");
        type QUrl = cxx_qt_lib::QUrl;
    }

    #[qenum(TextLayout)]
    enum GlyphRole {
        Glyph,
        Pen,
        LineTop,
    }

    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[base = QAbstractListModel]
        #[qml_element]
        /// The theme's font file; empty, or a file that holds no face, lays out in the kit's body face.
        #[qproperty(QUrl, font)]
        #[qproperty(QString, text)]
        /// THEMES.md's size in pixels, and whether the face is ES-DE's own (the theme named none), drawn at a whole size.
        #[qproperty(f64, px)]
        #[qproperty(bool, own_face)]
        /// The box; 0 leaves that side unbounded.
        #[qproperty(f64, box_width)]
        #[qproperty(f64, box_height)]
        /// THEMES.md's lineSpacing; 0 is the face's own line.
        #[qproperty(f64, line_spacing)]
        #[qproperty(bool, wrap)]
        #[qproperty(bool, ellipsis)]
        /// 0 start, 1 centre, 2 end.
        #[qproperty(i32, align)]
        /// The size the glyphs are drawn at, the pitch, a line's baseline below its top, the widest line and the lines' height.
        #[qproperty(f64, size, READ, NOTIFY)]
        #[qproperty(f64, pitch, READ, NOTIFY)]
        #[qproperty(f64, baseline, READ, NOTIFY)]
        #[qproperty(f64, content_width, READ, NOTIFY)]
        #[qproperty(f64, content_height, READ, NOTIFY)]
        #[qproperty(i32, line_count, READ, NOTIFY)]
        type TextLayout = super::TextLayoutRust;
    }

    extern "RustQt" {
        #[qinvokable]
        #[cxx_override]
        fn data(self: &TextLayout, index: &QModelIndex, role: i32) -> QVariant;
        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &TextLayout) -> QHash_i32_QByteArray;
        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &TextLayout, parent: &QModelIndex) -> i32;
    }

    unsafe extern "RustQt" {
        #[inherit]
        #[cxx_name = "beginResetModel"]
        fn begin_reset_model(self: Pin<&mut TextLayout>);
        #[inherit]
        #[cxx_name = "endResetModel"]
        fn end_reset_model(self: Pin<&mut TextLayout>);
    }

    impl cxx_qt::Initialize for TextLayout {}
}

use qobject::GlyphRole;

const ROLE_NAMES: [(GlyphRole, &str); 3] = [(GlyphRole::Glyph, "glyph"), (GlyphRole::Pen, "pen"), (GlyphRole::LineTop, "lineTop")];

/// One glyph as drawn: its characters, its pen in the box and its line's top, whole pixels.
#[derive(Clone, PartialEq)]
struct Row {
    glyph: String,
    pen: i32,
    top: i32,
}

#[derive(Default)]
pub struct TextLayoutRust {
    font: QUrl,
    text: QString,
    px: f64,
    own_face: bool,
    box_width: f64,
    box_height: f64,
    line_spacing: f64,
    wrap: bool,
    ellipsis: bool,
    align: i32,
    size: f64,
    pitch: f64,
    baseline: f64,
    content_width: f64,
    content_height: f64,
    line_count: i32,
    rows: Vec<Row>,
}

thread_local! {
    // Each face read once, by its file; "" is the kit's body face.
    static FACES: RefCell<HashMap<String, Option<Rc<Face>>>> = RefCell::new(HashMap::new());
}

/// The face of a font file, read once; none when the file holds none.
pub(crate) fn face_of(path: &str) -> Option<Rc<Face>> {
    FACES.with(|faces| {
        faces
            .borrow_mut()
            .entry(path.to_owned())
            .or_insert_with(|| {
                let bytes = if path.is_empty() { Some(emusen_kit_qt::fonts::BODY.to_vec()) } else { std::fs::read(path).ok() };
                bytes.and_then(Face::new).map(Rc::new)
            })
            .clone()
    })
}

impl TextLayoutRust {
    fn laid(&self) -> Layout {
        let path = self.font.to_local_file().map(|p| p.to_string()).unwrap_or_default();
        let face = face_of(&path).or_else(|| face_of("")).expect("the kit's body face is a face");
        let text = self.text.to_string();
        let align = match self.align {
            1 => Align::Center,
            2 => Align::End,
            _ => Align::Start,
        };
        let spec = Spec {
            text: &text,
            px: self.px,
            own_face: self.own_face,
            width: self.box_width,
            height: self.box_height,
            line_spacing: self.line_spacing,
            wrap: self.wrap,
            ellipsis: self.ellipsis,
            align,
        };
        layout(&face, &spec)
    }
}

impl cxx_qt::Initialize for qobject::TextLayout {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut().on_font_changed(|this| this.refresh()).release();
        self.as_mut().on_text_changed(|this| this.refresh()).release();
        self.as_mut().on_px_changed(|this| this.refresh()).release();
        self.as_mut().on_own_face_changed(|this| this.refresh()).release();
        self.as_mut().on_box_width_changed(|this| this.refresh()).release();
        self.as_mut().on_box_height_changed(|this| this.refresh()).release();
        self.as_mut().on_line_spacing_changed(|this| this.refresh()).release();
        self.as_mut().on_wrap_changed(|this| this.refresh()).release();
        self.as_mut().on_ellipsis_changed(|this| this.refresh()).release();
        self.as_mut().on_align_changed(|this| this.refresh()).release();
    }
}

impl qobject::TextLayout {
    fn refresh(mut self: Pin<&mut Self>) {
        if self.px <= 0.0 {
            return;
        }
        let laid = self.laid();
        let rows: Vec<Row> = laid
            .lines
            .iter()
            .flat_map(|l| l.glyphs.iter().map(move |g| Row { glyph: g.text.clone(), pen: l.x + g.x, top: l.y }))
            .filter(|r| !r.glyph.trim().is_empty())
            .collect();
        // The glyphs are made again only when they change, so a binding's settling does not redraw the words.
        if rows != self.rows {
            self.as_mut().begin_reset_model();
            self.as_mut().rust_mut().rows = rows;
            self.as_mut().end_reset_model();
        }
        publish!(self, size, size_changed, laid.size);
        publish!(self, pitch, pitch_changed, laid.pitch);
        publish!(self, baseline, baseline_changed, laid.baseline());
        publish!(self, line_count, line_count_changed, laid.lines.len() as i32);
        publish!(self, content_width, content_width_changed, f64::from(laid.width));
        publish!(self, content_height, content_height_changed, laid.height());
    }

    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let Some(row) = self.rows.get(index.row() as usize) else {
            return QVariant::default();
        };
        match (GlyphRole { repr: role }) {
            GlyphRole::Glyph => QVariant::from(&QString::from(row.glyph.as_str())),
            GlyphRole::Pen => QVariant::from(&row.pen),
            GlyphRole::LineTop => QVariant::from(&row.top),
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
