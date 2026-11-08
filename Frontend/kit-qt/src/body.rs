//! `BodyShapes`: a world's body as a list model of kit-core's shapes, one row per `Shape`, that `Body.qml` draws with Qt Quick Shapes (EmuSen_Kit.md §7.5).

use core::pin::Pin;

use cxx_qt::CxxQtType;
use cxx_qt_lib::{QByteArray, QHash, QHashPair_i32_QByteArray, QModelIndex, QVariant};
use emusen_kit_core::world;

use crate::convert::{polygon, qcolor};
use crate::kinds::{self, BodyShape, Ring};
use crate::world::{rows, ShapeRow};

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
        include!("emusen-kit-qt/src/kinds.cxxqt.h");
        #[namespace = "Bodies"]
        type BodyShape = crate::kinds::ffi::BodyShape;
        #[namespace = "Rings"]
        type Ring = crate::kinds::ffi::Ring;
    }

    #[qenum(BodyShapes)]
    enum BodyRole {
        Outline,
        FillInk,
        HasGradient,
        GradientX,
        GradientY,
        GradientRadius,
        Stop0At,
        Stop0Ink,
        Stop1At,
        Stop1Ink,
        Stop2At,
        Stop2Ink,
        Stop3At,
        Stop3Ink,
        StrokeInk,
        StrokeSize,
        InkOpacity,
    }

    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[base = QAbstractListModel]
        #[qml_element]
        #[qproperty(BodyShape, shape)]
        #[qproperty(f64, hue)]
        #[qproperty(bool, focused)]
        #[qproperty(f64, craters)]
        #[qproperty(f64, bands)]
        #[qproperty(f64, cap)]
        #[qproperty(Ring, ring)]
        #[qproperty(i32, moons)]
        #[qproperty(f64, unit_width, READ, NOTIFY)]
        #[qproperty(f64, unit_height, READ, NOTIFY)]
        type BodyShapes = super::BodyShapesRust;
    }

    extern "RustQt" {
        #[qinvokable]
        #[cxx_override]
        fn data(self: &BodyShapes, index: &QModelIndex, role: i32) -> QVariant;
        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &BodyShapes) -> QHash_i32_QByteArray;
        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &BodyShapes, parent: &QModelIndex) -> i32;
    }

    unsafe extern "RustQt" {
        #[inherit]
        #[cxx_name = "beginResetModel"]
        fn begin_reset_model(self: Pin<&mut BodyShapes>);
        #[inherit]
        #[cxx_name = "endResetModel"]
        fn end_reset_model(self: Pin<&mut BodyShapes>);
    }

    impl cxx_qt::Initialize for BodyShapes {}
}

use qobject::BodyRole;

pub const ROLE_NAMES: [(BodyRole, &str); 17] = [
    (BodyRole::Outline, "outline"),
    (BodyRole::FillInk, "fillInk"),
    (BodyRole::HasGradient, "hasGradient"),
    (BodyRole::GradientX, "gradientX"),
    (BodyRole::GradientY, "gradientY"),
    (BodyRole::GradientRadius, "gradientRadius"),
    (BodyRole::Stop0At, "stop0At"),
    (BodyRole::Stop0Ink, "stop0Ink"),
    (BodyRole::Stop1At, "stop1At"),
    (BodyRole::Stop1Ink, "stop1Ink"),
    (BodyRole::Stop2At, "stop2At"),
    (BodyRole::Stop2Ink, "stop2Ink"),
    (BodyRole::Stop3At, "stop3At"),
    (BodyRole::Stop3Ink, "stop3Ink"),
    (BodyRole::StrokeInk, "strokeInk"),
    (BodyRole::StrokeSize, "strokeSize"),
    (BodyRole::InkOpacity, "inkOpacity"),
];

pub struct BodyShapesRust {
    shape: BodyShape,
    hue: f64,
    focused: bool,
    craters: f64,
    bands: f64,
    cap: f64,
    ring: Ring,
    moons: i32,
    unit_width: f64,
    unit_height: f64,
    rows: Vec<ShapeRow>,
}

impl Default for BodyShapesRust {
    fn default() -> Self {
        let (unit_width, unit_height) = world::unit_box(world::WorldShape::Planet);
        let mut rust = BodyShapesRust {
            shape: BodyShape::Planet,
            hue: 0.0,
            focused: false,
            craters: 0.0,
            bands: 0.0,
            cap: 0.0,
            ring: Ring::NoRing,
            moons: 0,
            unit_width,
            unit_height,
            rows: Vec::new(),
        };
        rust.rebuild();
        rust
    }
}

impl BodyShapesRust {
    fn body(&self) -> world::WorldBody {
        let moons = u8::try_from(self.moons.clamp(0, 2)).unwrap_or(0);
        crate::world::body(kinds::world_shape(self.shape), self.craters, self.bands, self.cap, kinds::world_ring(self.ring), moons)
    }

    fn rebuild(&mut self) {
        let body = self.body();
        (self.unit_width, self.unit_height) = world::unit_box(body.shape);
        self.rows = rows(&body, self.hue, self.focused);
    }
}

impl cxx_qt::Initialize for qobject::BodyShapes {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut().on_shape_changed(|this| this.refresh()).release();
        self.as_mut().on_hue_changed(|this| this.refresh()).release();
        self.as_mut().on_focused_changed(|this| this.refresh()).release();
        self.as_mut().on_craters_changed(|this| this.refresh()).release();
        self.as_mut().on_bands_changed(|this| this.refresh()).release();
        self.as_mut().on_cap_changed(|this| this.refresh()).release();
        self.as_mut().on_ring_changed(|this| this.refresh()).release();
        self.as_mut().on_moons_changed(|this| this.refresh()).release();
    }
}

impl qobject::BodyShapes {
    fn refresh(mut self: Pin<&mut Self>) {
        self.as_mut().begin_reset_model();
        self.as_mut().rust_mut().rebuild();
        self.as_mut().end_reset_model();
        self.as_mut().unit_width_changed();
        self.as_mut().unit_height_changed();
    }

    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let Some(row) = self.rows.get(index.row() as usize) else {
            return QVariant::default();
        };
        let stop = |i: usize| row.gradient.as_ref().map(|g| g.stops[i]).unwrap_or((0.0, crate::world::CLEAR));
        match (BodyRole { repr: role }) {
            BodyRole::Outline => polygon(&row.points),
            BodyRole::FillInk => QVariant::from(&qcolor(row.fill)),
            BodyRole::HasGradient => QVariant::from(&row.gradient.is_some()),
            BodyRole::GradientX => QVariant::from(&row.gradient.as_ref().map_or(0.0, |g| g.centre[0])),
            BodyRole::GradientY => QVariant::from(&row.gradient.as_ref().map_or(0.0, |g| g.centre[1])),
            BodyRole::GradientRadius => QVariant::from(&row.gradient.as_ref().map_or(0.0, |g| g.radius)),
            BodyRole::Stop0At => QVariant::from(&stop(0).0),
            BodyRole::Stop0Ink => QVariant::from(&qcolor(stop(0).1)),
            BodyRole::Stop1At => QVariant::from(&stop(1).0),
            BodyRole::Stop1Ink => QVariant::from(&qcolor(stop(1).1)),
            BodyRole::Stop2At => QVariant::from(&stop(2).0),
            BodyRole::Stop2Ink => QVariant::from(&qcolor(stop(2).1)),
            BodyRole::Stop3At => QVariant::from(&stop(3).0),
            BodyRole::Stop3Ink => QVariant::from(&qcolor(stop(3).1)),
            BodyRole::StrokeInk => QVariant::from(&qcolor(row.stroke)),
            BodyRole::StrokeSize => QVariant::from(&row.stroke_width),
            BodyRole::InkOpacity => QVariant::from(&row.opacity),
            _ => QVariant::default(),
        }
    }

    pub fn role_names(&self) -> QHash<QHashPair_i32_QByteArray> {
        let mut roles = QHash::default();
        for (role, name) in ROLE_NAMES {
            roles.insert(role.repr, QByteArray::from(name));
        }
        roles
    }

    pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
        self.rows.len() as i32
    }
}
