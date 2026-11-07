//! kit-core's plain values as Qt's types, in one place.

use cxx_qt_lib::{QColor, QMarginsF, QPointF, QPolygonF, QVariant, QVector};
use emusen_kit_core::tokens::{Colour, Insets};
use emusen_kit_core::world::Point;

pub fn qcolor(c: Colour) -> QColor {
    QColor::from_rgba_f(f32::from(c.r) / 255.0, f32::from(c.g) / 255.0, f32::from(c.b) / 255.0, f32::from(c.a) / 255.0)
}

pub fn margins(i: Insets) -> QMarginsF {
    QMarginsF::new(i.left, i.top, i.right, i.bottom)
}

/// Points as the QPolygonF a `PathPolyline` takes for its `path`.
pub fn polygon(points: &[Point]) -> QVariant {
    let mut vector = QVector::<QPointF>::default();
    vector.reserve(points.len() as isize);
    for p in points {
        vector.append(QPointF::new(p[0], p[1]));
    }
    QVariant::from(&QPolygonF::from(&vector))
}
