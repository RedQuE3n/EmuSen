//! `PlanetLayout`: kit-core's rows of bodies on Qt, as lists a screen's tiles bind to by index (EmuSen_Kit.md §7.17).

use core::pin::Pin;

use cxx_qt::CxxQtType;
use cxx_qt_lib::{QList, QVariant};
use emusen_kit_core::layout::{planet_layout, PlanetGeometry};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
    }

    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(f64, width)]
        #[qproperty(i32, leading)]
        #[qproperty(i32, bodies)]
        #[qproperty(QVariant, xs, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, ys, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, widths, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, heights, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, left, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, right, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, up, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, down, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, orbits, READ, NOTIFY = laid_out)]
        #[qproperty(f64, lift, READ, CONSTANT)]
        type PlanetLayout = super::PlanetLayoutRust;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        /// Every list changed together.
        #[qsignal]
        fn laid_out(self: Pin<&mut PlanetLayout>);
    }

    impl cxx_qt::Initialize for PlanetLayout {}
}

/// The inputs, and the layout as the lists QML reads, each a QList in a QVariant since qmllint cannot name cxx-qt's
/// list aliases; a neighbour is -1 where there is none.
#[derive(Default)]
pub struct PlanetLayoutRust {
    width: f64,
    leading: i32,
    bodies: i32,
    xs: QVariant,
    ys: QVariant,
    widths: QVariant,
    heights: QVariant,
    left: QVariant,
    right: QVariant,
    up: QVariant,
    down: QVariant,
    orbits: QVariant,
    lift: f64,
}

impl cxx_qt::Initialize for qobject::PlanetLayout {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().lift = PlanetGeometry::COURTS.lift;
        self.as_mut().on_width_changed(|this| this.lay_out()).release();
        self.as_mut().on_leading_changed(|this| this.lay_out()).release();
        self.as_mut().on_bodies_changed(|this| this.lay_out()).release();
    }
}

fn list<T: cxx_qt_lib::QListElement + Clone>(values: impl Iterator<Item = T>) -> QVariant
where
    QList<T>: cxx_qt_lib::QVariantValue,
{
    QVariant::from(&QList::from(&values.collect::<Vec<T>>()))
}

fn index(n: Option<usize>) -> i32 {
    n.map_or(-1, |i| i as i32)
}

impl qobject::PlanetLayout {
    fn lay_out(mut self: Pin<&mut Self>) {
        let l = planet_layout(self.width, self.leading.max(0) as usize, self.bodies.max(0) as usize, &PlanetGeometry::COURTS);
        let mut r = self.as_mut().rust_mut();
        r.xs = list(l.tiles.iter().map(|t| t.x));
        r.ys = list(l.tiles.iter().map(|t| t.y));
        r.widths = list(l.tiles.iter().map(|t| t.width));
        r.heights = list(l.tiles.iter().map(|t| t.height));
        r.left = list(l.tiles.iter().map(|t| index(t.left)));
        r.right = list(l.tiles.iter().map(|t| index(t.right)));
        r.up = list(l.tiles.iter().map(|t| index(t.up)));
        r.down = list(l.tiles.iter().map(|t| index(t.down)));
        r.orbits = list(l.orbits.iter().copied());
        self.laid_out();
    }
}
