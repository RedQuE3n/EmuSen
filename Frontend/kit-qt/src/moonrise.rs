//! `MoonriseModel`: kit-core's splash frame as properties and its planets as rows, so `Moonrise.qml` only lays the frame out (EmuSen_Kit.md §7.5).

use core::pin::Pin;

use cxx_qt::CxxQtType;
use cxx_qt_lib::{QByteArray, QColor, QHash, QHashPair_i32_QByteArray, QList, QModelIndex, QString, QVariant, QVector};
use emusen_kit_core::world::{self, Frame, PlanetFrame, Splash, SplashBody, Timeline, WorldBody, WorldShape};

use crate::convert::{polygon, qcolor};

/// Points along each of the shadow's two arcs: under a tenth of a pixel from the circle at a 4K moon.
const SHADOW_STEPS: usize = 64;
use crate::kinds::{self, BodyShape, Ring};

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
        include!("cxx-qt-lib/qvector.h");
        type QVector_i32 = cxx_qt_lib::QVector<i32>;
        include!("cxx-qt-lib/qcolor.h");
        type QColor = cxx_qt_lib::QColor;
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qlist.h");
        type QList_f64 = cxx_qt_lib::QList<f64>;
        include!("emusen-kit-qt/src/kinds.cxxqt.h");
        #[namespace = "Bodies"]
        type BodyShape = crate::kinds::ffi::BodyShape;
        #[namespace = "Rings"]
        type Ring = crate::kinds::ffi::Ring;
    }

    #[qenum(MoonriseModel)]
    enum PlanetRole {
        PlanetX,
        PlanetY,
        PlanetRadius,
        PlanetOpacity,
        GlowRadius,
        GlowInk,
        FlareRadius,
        FlareOpacity,
        FlareInk,
        BoxX,
        BoxY,
        BoxWidth,
        BoxHeight,
        BodyKind,
        Hue,
        Craters,
        Bands,
        Cap,
        RingKind,
        Moons,
        Plain,
        LightInk,
        RimInk,
        FocusX,
        FocusY,
        FocusRadius,
    }

    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[base = QAbstractListModel]
        #[qml_element]
        #[qproperty(f64, width)]
        #[qproperty(f64, height)]
        #[qproperty(QString, title)]
        #[qproperty(QString, caption)]
        #[qproperty(QList_f64, hues)]
        #[qproperty(i32, seed, READ, CONSTANT)]
        #[qproperty(i32, star_count, READ, CONSTANT)]
        #[qproperty(f64, length, READ, CONSTANT)]
        #[qproperty(f64, time, READ, NOTIFY)]
        #[qproperty(f64, draft_time, READ, NOTIFY)]
        #[qproperty(f64, unit, READ, NOTIFY)]
        #[qproperty(bool, finished, READ, NOTIFY)]
        #[qproperty(f64, stars_opacity, READ, NOTIFY)]
        #[qproperty(f64, moon_x, READ, NOTIFY)]
        #[qproperty(f64, moon_y, READ, NOTIFY)]
        #[qproperty(f64, moon_radius, READ, NOTIFY)]
        #[qproperty(f64, halo_opacity, READ, NOTIFY)]
        #[qproperty(f64, halo_inner_radius, READ, NOTIFY)]
        #[qproperty(f64, halo_radius, READ, NOTIFY)]
        #[qproperty(f64, outline_drawn, READ, NOTIFY)]
        #[qproperty(f64, outline_width, READ, NOTIFY)]
        #[qproperty(f64, fill, READ, NOTIFY)]
        #[qproperty(f64, lit_focus_x, READ, NOTIFY)]
        #[qproperty(f64, lit_focus_y, READ, NOTIFY)]
        #[qproperty(f64, lit_focus_radius, READ, NOTIFY)]
        #[qproperty(f64, lit_radius, READ, NOTIFY)]
        #[qproperty(QVariant, shadow_outline, READ, NOTIFY)]
        #[qproperty(bool, sweep_visible, READ, NOTIFY)]
        #[qproperty(f64, sweep_x, READ, NOTIFY)]
        #[qproperty(f64, sweep_y, READ, NOTIFY)]
        #[qproperty(f64, sweep_width, READ, NOTIFY)]
        #[qproperty(f64, sweep_height, READ, NOTIFY)]
        #[qproperty(bool, line_visible, READ, NOTIFY)]
        #[qproperty(f64, line_from_x, READ, NOTIFY)]
        #[qproperty(f64, line_to_x, READ, NOTIFY)]
        #[qproperty(f64, line_y, READ, NOTIFY)]
        #[qproperty(f64, line_width, READ, NOTIFY)]
        #[qproperty(f64, line_opacity, READ, NOTIFY)]
        #[qproperty(bool, words_visible, READ, NOTIFY)]
        #[qproperty(f64, words_opacity, READ, NOTIFY)]
        #[qproperty(f64, title_size, READ, NOTIFY)]
        #[qproperty(f64, title_spacing, READ, NOTIFY)]
        #[qproperty(f64, title_baseline_y, READ, NOTIFY)]
        #[qproperty(f64, caption_size, READ, NOTIFY)]
        #[qproperty(f64, caption_spacing, READ, NOTIFY)]
        #[qproperty(f64, caption_baseline_y, READ, NOTIFY)]
        #[qproperty(QColor, halo_ink, READ, CONSTANT)]
        #[qproperty(QColor, halo_edge_ink, READ, CONSTANT)]
        #[qproperty(QColor, outline_ink, READ, CONSTANT)]
        #[qproperty(QColor, lit_inner_ink, READ, CONSTANT)]
        #[qproperty(QColor, lit_outer_ink, READ, CONSTANT)]
        #[qproperty(QColor, shadow_ink, READ, CONSTANT)]
        #[qproperty(QColor, sweep_ink, READ, CONSTANT)]
        #[qproperty(QColor, line_ink, READ, CONSTANT)]
        #[qproperty(QColor, title_ink, READ, CONSTANT)]
        #[qproperty(QColor, caption_ink, READ, CONSTANT)]
        type MoonriseModel = super::MoonriseModelRust;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        /// A world on the row, in order; the splash is placed again from its seed at the next tick.
        #[qinvokable]
        fn add_body(
            self: Pin<&mut MoonriseModel>,
            name: &QString,
            hue: f64,
            shape: BodyShape,
            craters: f64,
            bands: f64,
            cap: f64,
            ring: Ring,
            moons: i32,
        );
        #[qinvokable]
        fn clear_bodies(self: Pin<&mut MoonriseModel>);
        /// The frame for the host's clock, in seconds from the splash's start.
        #[qinvokable]
        fn tick(self: Pin<&mut MoonriseModel>, seconds: f64);
    }

    extern "RustQt" {
        #[qinvokable]
        #[cxx_override]
        fn data(self: &MoonriseModel, index: &QModelIndex, role: i32) -> QVariant;
        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &MoonriseModel) -> QHash_i32_QByteArray;
        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &MoonriseModel, parent: &QModelIndex) -> i32;
    }

    unsafe extern "RustQt" {
        #[inherit]
        #[cxx_name = "beginResetModel"]
        fn begin_reset_model(self: Pin<&mut MoonriseModel>);
        #[inherit]
        #[cxx_name = "endResetModel"]
        fn end_reset_model(self: Pin<&mut MoonriseModel>);
        #[inherit]
        fn index(self: &MoonriseModel, row: i32, column: i32, parent: &QModelIndex) -> QModelIndex;
        #[inherit]
        #[qsignal]
        #[cxx_name = "dataChanged"]
        fn data_changed(self: Pin<&mut MoonriseModel>, top_left: &QModelIndex, bottom_right: &QModelIndex, roles: &QVector_i32);
    }

    impl cxx_qt::Initialize for MoonriseModel {}
}

use qobject::PlanetRole;

pub const ROLE_NAMES: [(PlanetRole, &str); 26] = [
    (PlanetRole::PlanetX, "planetX"),
    (PlanetRole::PlanetY, "planetY"),
    (PlanetRole::PlanetRadius, "planetRadius"),
    (PlanetRole::PlanetOpacity, "planetOpacity"),
    (PlanetRole::GlowRadius, "glowRadius"),
    (PlanetRole::GlowInk, "glowInk"),
    (PlanetRole::FlareRadius, "flareRadius"),
    (PlanetRole::FlareOpacity, "flareOpacity"),
    (PlanetRole::FlareInk, "flareInk"),
    (PlanetRole::BoxX, "boxX"),
    (PlanetRole::BoxY, "boxY"),
    (PlanetRole::BoxWidth, "boxWidth"),
    (PlanetRole::BoxHeight, "boxHeight"),
    (PlanetRole::BodyKind, "bodyKind"),
    (PlanetRole::Hue, "hue"),
    (PlanetRole::Craters, "craters"),
    (PlanetRole::Bands, "bands"),
    (PlanetRole::Cap, "cap"),
    (PlanetRole::RingKind, "ringKind"),
    (PlanetRole::Moons, "moons"),
    (PlanetRole::Plain, "plain"),
    (PlanetRole::LightInk, "lightInk"),
    (PlanetRole::RimInk, "rimInk"),
    (PlanetRole::FocusX, "focusX"),
    (PlanetRole::FocusY, "focusY"),
    (PlanetRole::FocusRadius, "focusRadius"),
];

pub struct MoonriseModelRust {
    width: f64,
    height: f64,
    title: QString,
    caption: QString,
    hues: QList<f64>,
    seed: i32,
    star_count: i32,
    length: f64,
    time: f64,
    draft_time: f64,
    unit: f64,
    finished: bool,
    stars_opacity: f64,
    moon_x: f64,
    moon_y: f64,
    moon_radius: f64,
    halo_opacity: f64,
    halo_inner_radius: f64,
    halo_radius: f64,
    outline_drawn: f64,
    outline_width: f64,
    fill: f64,
    lit_focus_x: f64,
    lit_focus_y: f64,
    lit_focus_radius: f64,
    lit_radius: f64,
    /// The shadow clipped to the moon, in the moon's box (from its top left corner).
    shadow_outline: QVariant,
    shadow_points: Vec<world::Point>,
    sweep_visible: bool,
    sweep_x: f64,
    sweep_y: f64,
    sweep_width: f64,
    sweep_height: f64,
    line_visible: bool,
    line_from_x: f64,
    line_to_x: f64,
    line_y: f64,
    line_width: f64,
    line_opacity: f64,
    words_visible: bool,
    words_opacity: f64,
    title_size: f64,
    title_spacing: f64,
    title_baseline_y: f64,
    caption_size: f64,
    caption_spacing: f64,
    caption_baseline_y: f64,
    halo_ink: QColor,
    halo_edge_ink: QColor,
    outline_ink: QColor,
    lit_inner_ink: QColor,
    lit_outer_ink: QColor,
    shadow_ink: QColor,
    sweep_ink: QColor,
    line_ink: QColor,
    title_ink: QColor,
    caption_ink: QColor,
    bodies: Vec<SplashBody>,
    splash: Option<Splash>,
    frame: Option<Frame>,
}

impl Default for MoonriseModelRust {
    fn default() -> Self {
        let timeline = Timeline::MOONRISE;
        MoonriseModelRust {
            width: 0.0,
            height: 0.0,
            title: QString::default(),
            caption: QString::default(),
            hues: QList::default(),
            seed: timeline.seed as i32,
            star_count: timeline.star_count as i32,
            length: timeline.length,
            time: 0.0,
            draft_time: 0.0,
            unit: 0.0,
            finished: false,
            stars_opacity: 0.0,
            moon_x: 0.0,
            moon_y: 0.0,
            moon_radius: 0.0,
            halo_opacity: 0.0,
            halo_inner_radius: 0.0,
            halo_radius: 0.0,
            outline_drawn: 0.0,
            outline_width: 0.0,
            fill: 0.0,
            lit_focus_x: 0.0,
            lit_focus_y: 0.0,
            lit_focus_radius: 0.0,
            lit_radius: 0.0,
            shadow_outline: polygon(&[]),
            shadow_points: Vec::new(),
            sweep_visible: false,
            sweep_x: 0.0,
            sweep_y: 0.0,
            sweep_width: 0.0,
            sweep_height: 0.0,
            line_visible: false,
            line_from_x: 0.0,
            line_to_x: 0.0,
            line_y: 0.0,
            line_width: 0.0,
            line_opacity: 0.0,
            words_visible: false,
            words_opacity: 0.0,
            title_size: 0.0,
            title_spacing: 0.0,
            title_baseline_y: 0.0,
            caption_size: 0.0,
            caption_spacing: 0.0,
            caption_baseline_y: 0.0,
            halo_ink: qcolor(world::HALO),
            halo_edge_ink: qcolor(world::HALO_EDGE),
            outline_ink: qcolor(world::OUTLINE),
            lit_inner_ink: qcolor(world::LIT_INNER),
            lit_outer_ink: qcolor(world::LIT_OUTER),
            shadow_ink: qcolor(world::SHADOW),
            sweep_ink: qcolor(world::SWEEP),
            line_ink: qcolor(world::LINE),
            title_ink: qcolor(world::TITLE),
            caption_ink: qcolor(world::CAPTION),
            bodies: Vec::new(),
            splash: None,
            frame: None,
        }
    }
}

impl MoonriseModelRust {
    fn planets(&self) -> &[PlanetFrame] {
        self.frame.as_ref().map_or(&[], |f| f.planets.as_slice())
    }

    fn body_of(&self, planet: &PlanetFrame) -> (f64, WorldBody) {
        let body = &self.bodies[planet.index];
        (body.hue, body.body.unwrap_or(WorldBody::PLAIN))
    }

    /// Reads a frame into the properties; the planets' rows are the caller's.
    fn read(&mut self, frame: &Frame) {
        self.unit = frame.unit;
        self.draft_time = frame.draft_time;
        self.finished = frame.finished;
        self.stars_opacity = frame.stars_opacity;
        let moon = &frame.moon;
        (self.moon_x, self.moon_y, self.moon_radius) = (moon.centre[0], moon.centre[1], moon.radius);
        (self.halo_opacity, self.halo_inner_radius, self.halo_radius) = (moon.halo_opacity, moon.halo_inner_radius, moon.halo_radius);
        (self.outline_drawn, self.outline_width, self.fill) = (moon.outline_drawn, moon.outline_width, moon.fill);
        let (left, top) = (moon.centre[0] - moon.radius, moon.centre[1] - moon.radius);
        (self.lit_focus_x, self.lit_focus_y) = (moon.lit_focus[0] - left, moon.lit_focus[1] - top);
        (self.lit_focus_radius, self.lit_radius) = (moon.lit_focus_radius, moon.lit_radius);
        moon.shadow_outline(SHADOW_STEPS, &mut self.shadow_points);
        for p in &mut self.shadow_points {
            *p = [p[0] - left, p[1] - top];
        }
        self.shadow_outline = polygon(&self.shadow_points);
        self.sweep_visible = frame.sweep.is_some();
        if let Some(sweep) = &frame.sweep {
            (self.sweep_x, self.sweep_y, self.sweep_width, self.sweep_height) =
                (sweep.band[0], sweep.band[1], sweep.band[2], sweep.band[3]);
        }
        self.line_visible = frame.line.is_some();
        if let Some(line) = &frame.line {
            (self.line_from_x, self.line_to_x, self.line_y, self.line_opacity) = (line.from_x, line.to_x, line.y, line.opacity);
            self.line_width = line.width;
        }
        self.words_visible = frame.words.is_some();
        if let Some(words) = &frame.words {
            self.words_opacity = words.opacity;
            (self.title_size, self.title_spacing, self.title_baseline_y) = (words.title_size, words.title_spacing, words.title_baseline_y);
            (self.caption_size, self.caption_spacing, self.caption_baseline_y) =
                (words.caption_size, words.caption_spacing, words.caption_baseline_y);
        }
    }
}

impl cxx_qt::Initialize for qobject::MoonriseModel {
    /// `hues` is the declarative way to a row of plain worlds, one per hue; `addBody` is the host's full form.
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut()
            .on_hues_changed(|mut this| {
                let bodies: Vec<SplashBody> =
                    this.hues.iter().map(|hue| SplashBody { name: String::new(), hue: *hue, body: None }).collect();
                let mut rust = this.as_mut().rust_mut();
                rust.bodies = bodies;
                rust.splash = None;
            })
            .release();
        // A resize lays the frame out again at the time shown, so a finished splash follows its window.
        self.as_mut().on_width_changed(|this| this.relayout()).release();
        self.as_mut().on_height_changed(|this| this.relayout()).release();
    }
}

impl qobject::MoonriseModel {
    fn relayout(self: Pin<&mut Self>) {
        let time = self.time;
        if self.frame.is_some() {
            self.tick(time);
        }
    }
}

impl qobject::MoonriseModel {
    pub fn add_body(
        mut self: Pin<&mut Self>,
        name: &QString,
        hue: f64,
        shape: BodyShape,
        craters: f64,
        bands: f64,
        cap: f64,
        ring: Ring,
        moons: i32,
    ) {
        let shape = kinds::world_shape(shape);
        let moons = u8::try_from(moons.clamp(0, 2)).unwrap_or(0);
        let body = crate::world::body(shape, craters, bands, cap, kinds::world_ring(ring), moons);
        let plain = shape == WorldShape::Planet && body == WorldBody::PLAIN;
        let mut rust = self.as_mut().rust_mut();
        rust.bodies.push(SplashBody { name: name.to_string(), hue, body: (!plain).then_some(body) });
        rust.splash = None;
    }

    pub fn clear_bodies(mut self: Pin<&mut Self>) {
        let mut rust = self.as_mut().rust_mut();
        rust.bodies.clear();
        rust.splash = None;
    }

    pub fn tick(mut self: Pin<&mut Self>, seconds: f64) {
        let before = self.planets().len();
        {
            let mut rust = self.as_mut().rust_mut();
            if rust.splash.is_none() {
                rust.splash = Some(Splash::new(Timeline::MOONRISE, rust.bodies.clone()));
            }
            let (width, height) = (rust.width, rust.height);
            let frame = rust.splash.as_ref().map(|s| s.frame(seconds, width, height));
            rust.time = seconds;
            if let Some(frame) = &frame {
                rust.read(frame);
            }
            rust.frame = frame;
        }
        let after = self.planets().len();
        if after != before {
            self.as_mut().begin_reset_model();
            self.as_mut().end_reset_model();
        } else if after > 0 {
            let (first, last) = (self.index(0, 0, &QModelIndex::default()), self.index(after as i32 - 1, 0, &QModelIndex::default()));
            self.as_mut().data_changed(&first, &last, &QVector::<i32>::default());
        }
        self.announce();
    }

    fn announce(mut self: Pin<&mut Self>) {
        self.as_mut().time_changed();
        self.as_mut().draft_time_changed();
        self.as_mut().unit_changed();
        self.as_mut().finished_changed();
        self.as_mut().stars_opacity_changed();
        self.as_mut().moon_x_changed();
        self.as_mut().moon_y_changed();
        self.as_mut().moon_radius_changed();
        self.as_mut().halo_opacity_changed();
        self.as_mut().halo_inner_radius_changed();
        self.as_mut().halo_radius_changed();
        self.as_mut().outline_drawn_changed();
        self.as_mut().outline_width_changed();
        self.as_mut().fill_changed();
        self.as_mut().lit_focus_x_changed();
        self.as_mut().lit_focus_y_changed();
        self.as_mut().lit_focus_radius_changed();
        self.as_mut().lit_radius_changed();
        self.as_mut().shadow_outline_changed();
        self.as_mut().sweep_visible_changed();
        self.as_mut().sweep_x_changed();
        self.as_mut().sweep_y_changed();
        self.as_mut().sweep_width_changed();
        self.as_mut().sweep_height_changed();
        self.as_mut().line_visible_changed();
        self.as_mut().line_from_x_changed();
        self.as_mut().line_to_x_changed();
        self.as_mut().line_y_changed();
        self.as_mut().line_width_changed();
        self.as_mut().line_opacity_changed();
        self.as_mut().words_visible_changed();
        self.as_mut().words_opacity_changed();
        self.as_mut().title_size_changed();
        self.as_mut().title_spacing_changed();
        self.as_mut().title_baseline_y_changed();
        self.as_mut().caption_size_changed();
        self.as_mut().caption_spacing_changed();
        self.as_mut().caption_baseline_y_changed();
    }

    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let Some(planet) = self.planets().get(index.row() as usize) else {
            return QVariant::default();
        };
        let (hue, body) = self.body_of(planet);
        match (PlanetRole { repr: role }) {
            PlanetRole::PlanetX => QVariant::from(&planet.centre[0]),
            PlanetRole::PlanetY => QVariant::from(&planet.centre[1]),
            PlanetRole::PlanetRadius => QVariant::from(&planet.radius),
            PlanetRole::PlanetOpacity => QVariant::from(&planet.opacity),
            PlanetRole::GlowRadius => QVariant::from(&planet.glow_radius),
            PlanetRole::GlowInk => QVariant::from(&qcolor(planet.glow_colour)),
            PlanetRole::FlareRadius => QVariant::from(&planet.flare_radius),
            PlanetRole::FlareOpacity => QVariant::from(&planet.flare_opacity),
            PlanetRole::FlareInk => QVariant::from(&qcolor(planet.flare_colour)),
            PlanetRole::BoxX => QVariant::from(&planet.body_box[0]),
            PlanetRole::BoxY => QVariant::from(&planet.body_box[1]),
            PlanetRole::BoxWidth => QVariant::from(&planet.body_box[2]),
            PlanetRole::BoxHeight => QVariant::from(&planet.body_box[3]),
            PlanetRole::BodyKind => QVariant::from(&kinds::body_shape(body.shape).repr),
            PlanetRole::Hue => QVariant::from(&hue),
            PlanetRole::Craters => QVariant::from(&body.craters),
            PlanetRole::Bands => QVariant::from(&body.bands),
            PlanetRole::Cap => QVariant::from(&body.cap),
            PlanetRole::RingKind => QVariant::from(&kinds::ring_of(body.ring).repr),
            PlanetRole::Moons => QVariant::from(&i32::from(body.moons)),
            PlanetRole::Plain => QVariant::from(&planet.plain),
            PlanetRole::LightInk => QVariant::from(&qcolor(planet.light)),
            PlanetRole::RimInk => QVariant::from(&qcolor(planet.rim)),
            PlanetRole::FocusX => QVariant::from(&(planet.focus[0] - planet.centre[0] + planet.radius)),
            PlanetRole::FocusY => QVariant::from(&(planet.focus[1] - planet.centre[1] + planet.radius)),
            PlanetRole::FocusRadius => QVariant::from(&planet.focus_radius),
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
        self.planets().len() as i32
    }
}
