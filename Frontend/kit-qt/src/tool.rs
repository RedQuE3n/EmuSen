//! `ToolLayout`: kit-core's tool screen places on Qt, as rectangles a screen binds to (EmuSen_Kit.md §7.43).

use core::pin::Pin;

use cxx_qt::CxxQtType;
use cxx_qt_lib::QRectF;
use emusen_kit_core::audit::Rect;
use emusen_kit_core::tool::{tool_layout, ToolGeometry};

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
        #[qproperty(QRectF, header, READ, NOTIFY = laid_out)]
        #[qproperty(QRectF, strip, READ, NOTIFY = laid_out)]
        #[qproperty(QRectF, search, READ, NOTIFY = laid_out)]
        #[qproperty(QRectF, list, READ, NOTIFY = laid_out)]
        #[qproperty(QRectF, bar, READ, NOTIFY = laid_out)]
        #[qproperty(QRectF, pane, READ, NOTIFY = laid_out)]
        #[qproperty(i32, rows, READ, NOTIFY = laid_out)]
        #[qproperty(f64, bar_inset, READ, CONSTANT)]
        #[qproperty(f64, pane_padding, READ, CONSTANT)]
        type ToolLayout = super::ToolLayoutRust;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        /// Every place changed together.
        #[qsignal]
        fn laid_out(self: Pin<&mut ToolLayout>);
    }

    impl cxx_qt::Initialize for ToolLayout {}
}

/// A kit-core rectangle as Qt's.
pub fn rect(r: Rect) -> QRectF {
    QRectF::new(r.x, r.y, r.w, r.h)
}

/// The tool screen's places at the screen's size in design units.
pub struct ToolLayoutRust {
    width: f64,
    height: f64,
    header: QRectF,
    strip: QRectF,
    search: QRectF,
    list: QRectF,
    bar: QRectF,
    pane: QRectF,
    rows: i32,
    bar_inset: f64,
    pane_padding: f64,
}

impl Default for ToolLayoutRust {
    fn default() -> Self {
        let g = ToolGeometry::RECORDED;
        Self {
            width: 0.0,
            height: 0.0,
            header: QRectF::default(),
            strip: QRectF::default(),
            search: QRectF::default(),
            list: QRectF::default(),
            bar: QRectF::default(),
            pane: QRectF::default(),
            rows: 0,
            bar_inset: g.bar_inset,
            pane_padding: g.pane_padding,
        }
    }
}

impl cxx_qt::Initialize for qobject::ToolLayout {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut().on_width_changed(|this| this.lay_out()).release();
        self.as_mut().on_height_changed(|this| this.lay_out()).release();
    }
}

impl qobject::ToolLayout {
    fn lay_out(mut self: Pin<&mut Self>) {
        let l = tool_layout(self.width, self.height, &ToolGeometry::RECORDED);
        let mut r = self.as_mut().rust_mut();
        r.header = rect(l.header);
        r.strip = rect(l.strip);
        r.search = rect(l.search);
        r.list = rect(l.list);
        r.bar = rect(l.bar);
        r.pane = rect(l.pane);
        r.rows = l.rows as i32;
        self.laid_out();
    }
}
