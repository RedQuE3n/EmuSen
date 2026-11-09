//! `PageLayout`: kit-core's page frame places on Qt, as rectangles a page binds to (EmuSen_Kit.md §7.44).

use core::pin::Pin;

use cxx_qt::CxxQtType;
use cxx_qt_lib::QRectF;
use emusen_kit_core::page::{page_layout, PageGeometry};

use crate::tool::rect;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qrectf.h");
        type QRectF = cxx_qt_lib::QRectF;
    }

    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(f64, width)]
        #[qproperty(f64, height)]
        #[qproperty(f64, aspect)]
        #[qproperty(QRectF, hero, READ, NOTIFY = laid_out)]
        #[qproperty(QRectF, cover, READ, NOTIFY = laid_out)]
        #[qproperty(QRectF, title, READ, NOTIFY = laid_out)]
        #[qproperty(QRectF, facts, READ, NOTIFY = laid_out)]
        #[qproperty(QRectF, actions, READ, NOTIFY = laid_out)]
        #[qproperty(QRectF, strip, READ, NOTIFY = laid_out)]
        #[qproperty(QRectF, content, READ, NOTIFY = laid_out)]
        #[qproperty(i32, rows, READ, NOTIFY = laid_out)]
        #[qproperty(f64, tile_width, READ, CONSTANT)]
        #[qproperty(f64, tile_height, READ, CONSTANT)]
        #[qproperty(f64, tile_gap, READ, CONSTANT)]
        #[qproperty(f64, label_offset, READ, CONSTANT)]
        #[qproperty(f64, line_offset, READ, CONSTANT)]
        #[qproperty(f64, picture_width, READ, CONSTANT)]
        #[qproperty(f64, picture_height, READ, CONSTANT)]
        #[qproperty(f64, picture_gap, READ, CONSTANT)]
        type PageLayout = super::PageLayoutRust;

    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        /// Every place changed together.
        #[qsignal]
        fn laid_out(self: Pin<&mut PageLayout>);
    }

    impl cxx_qt::Initialize for PageLayout {}
}

/// The page frame's places at the screen's size in design units, for a cover of a shape (width per unit of height).
pub struct PageLayoutRust {
    width: f64,
    height: f64,
    aspect: f64,
    hero: QRectF,
    cover: QRectF,
    title: QRectF,
    facts: QRectF,
    actions: QRectF,
    strip: QRectF,
    content: QRectF,
    rows: i32,
    tile_width: f64,
    tile_height: f64,
    tile_gap: f64,
    label_offset: f64,
    line_offset: f64,
    picture_width: f64,
    picture_height: f64,
    picture_gap: f64,
}

impl Default for PageLayoutRust {
    fn default() -> Self {
        let g = PageGeometry::RECORDED;
        Self {
            width: 0.0,
            height: 0.0,
            aspect: 0.75,
            hero: QRectF::default(),
            cover: QRectF::default(),
            title: QRectF::default(),
            facts: QRectF::default(),
            actions: QRectF::default(),
            strip: QRectF::default(),
            content: QRectF::default(),
            rows: 0,
            tile_width: g.tile_width,
            tile_height: g.tile_height,
            tile_gap: g.tile_gap,
            label_offset: g.label_offset,
            line_offset: g.line_offset,
            picture_width: g.picture_width,
            picture_height: g.picture_height,
            picture_gap: g.picture_gap,
        }
    }
}

impl cxx_qt::Initialize for qobject::PageLayout {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut().on_width_changed(|this| this.lay_out()).release();
        self.as_mut().on_height_changed(|this| this.lay_out()).release();
        self.as_mut().on_aspect_changed(|this| this.lay_out()).release();
    }
}

impl qobject::PageLayout {
    fn lay_out(mut self: Pin<&mut Self>) {
        let l = page_layout(self.width, self.height, self.aspect, &PageGeometry::RECORDED);
        let mut r = self.as_mut().rust_mut();
        r.hero = rect(l.hero);
        r.cover = rect(l.cover);
        r.title = rect(l.title);
        r.facts = rect(l.facts);
        r.actions = rect(l.actions);
        r.strip = rect(l.strip);
        r.content = rect(l.content);
        r.rows = l.rows as i32;
        self.laid_out();
    }
}
