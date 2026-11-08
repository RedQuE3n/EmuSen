//! `StarField`: the sky's stars as a list model and its gradient and glow as properties, on the clock `Sky.qml` ticks (EmuSen_Kit.md §7.5).

use core::pin::Pin;
use std::time::Duration;

use cxx_qt::CxxQtType;
use cxx_qt_lib::{QByteArray, QColor, QHash, QHashPair_i32_QByteArray, QModelIndex, QVariant, QVector};
use emusen_kit_core::world::{Timeline, STAR_COLOUR};

use crate::convert::qcolor;
use crate::stars::SkyState;

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
    }

    #[qenum(StarField)]
    enum StarRole {
        StarX,
        StarY,
        StarRadius,
        StarOpacity,
    }

    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[base = QAbstractListModel]
        #[qml_element]
        #[qproperty(i32, seed)]
        #[qproperty(i32, count)]
        #[qproperty(bool, moonrise_sky)]
        #[qproperty(f64, width)]
        #[qproperty(f64, height)]
        #[qproperty(f64, stars_opacity)]
        #[qproperty(f64, glow_hue)]
        #[qproperty(f64, glow_x)]
        #[qproperty(f64, glow_y)]
        #[qproperty(f64, time, READ, NOTIFY)]
        #[qproperty(QColor, star_ink, READ, CONSTANT)]
        #[qproperty(f64, stop0_at, READ, NOTIFY)]
        #[qproperty(QColor, stop0_ink, READ, NOTIFY)]
        #[qproperty(f64, stop1_at, READ, NOTIFY)]
        #[qproperty(QColor, stop1_ink, READ, NOTIFY)]
        #[qproperty(f64, stop2_at, READ, NOTIFY)]
        #[qproperty(QColor, stop2_ink, READ, NOTIFY)]
        #[qproperty(bool, glow_visible, READ, NOTIFY)]
        #[qproperty(f64, glow_center_x, READ, NOTIFY)]
        #[qproperty(f64, glow_center_y, READ, NOTIFY)]
        #[qproperty(f64, glow_radius_x, READ, NOTIFY)]
        #[qproperty(f64, glow_radius_y, READ, NOTIFY)]
        #[qproperty(f64, glow_stop0_at, READ, NOTIFY)]
        #[qproperty(QColor, glow_stop0_ink, READ, NOTIFY)]
        #[qproperty(f64, glow_stop1_at, READ, NOTIFY)]
        #[qproperty(QColor, glow_stop1_ink, READ, NOTIFY)]
        #[qproperty(f64, glow_stop2_at, READ, NOTIFY)]
        #[qproperty(QColor, glow_stop2_ink, READ, NOTIFY)]
        #[qproperty(f64, glow_stop3_at, READ, NOTIFY)]
        #[qproperty(QColor, glow_stop3_ink, READ, NOTIFY)]
        type StarField = super::StarFieldRust;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        /// The host's clock, in seconds: the stars twinkle and the glow eases to it.
        #[qinvokable]
        fn tick(self: Pin<&mut StarField>, seconds: f64);
    }

    extern "RustQt" {
        #[qinvokable]
        #[cxx_override]
        fn data(self: &StarField, index: &QModelIndex, role: i32) -> QVariant;
        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &StarField) -> QHash_i32_QByteArray;
        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &StarField, parent: &QModelIndex) -> i32;
    }

    unsafe extern "RustQt" {
        #[inherit]
        #[cxx_name = "beginResetModel"]
        fn begin_reset_model(self: Pin<&mut StarField>);
        #[inherit]
        #[cxx_name = "endResetModel"]
        fn end_reset_model(self: Pin<&mut StarField>);
        #[inherit]
        fn index(self: &StarField, row: i32, column: i32, parent: &QModelIndex) -> QModelIndex;
        #[inherit]
        #[qsignal]
        #[cxx_name = "dataChanged"]
        fn data_changed(self: Pin<&mut StarField>, top_left: &QModelIndex, bottom_right: &QModelIndex, roles: &QVector_i32);
    }

    impl cxx_qt::Initialize for StarField {}
}

use qobject::StarRole;

pub struct StarFieldRust {
    seed: i32,
    count: i32,
    moonrise_sky: bool,
    width: f64,
    height: f64,
    stars_opacity: f64,
    glow_hue: f64,
    glow_x: f64,
    glow_y: f64,
    time: f64,
    star_ink: QColor,
    stop0_at: f64,
    stop0_ink: QColor,
    stop1_at: f64,
    stop1_ink: QColor,
    stop2_at: f64,
    stop2_ink: QColor,
    glow_visible: bool,
    glow_center_x: f64,
    glow_center_y: f64,
    glow_radius_x: f64,
    glow_radius_y: f64,
    glow_stop0_at: f64,
    glow_stop0_ink: QColor,
    glow_stop1_at: f64,
    glow_stop1_ink: QColor,
    glow_stop2_at: f64,
    glow_stop2_ink: QColor,
    glow_stop3_at: f64,
    glow_stop3_ink: QColor,
    state: SkyState,
}

impl Default for StarFieldRust {
    fn default() -> Self {
        let timeline = Timeline::MOONRISE;
        let state = SkyState::new(timeline.seed, timeline.star_count, false);
        let mut rust = StarFieldRust {
            seed: timeline.seed as i32,
            count: timeline.star_count as i32,
            moonrise_sky: false,
            width: 0.0,
            height: 0.0,
            stars_opacity: 1.0,
            glow_hue: -1.0,
            glow_x: 0.5,
            glow_y: 0.5,
            time: 0.0,
            star_ink: qcolor(STAR_COLOUR),
            stop0_at: 0.0,
            stop0_ink: QColor::default(),
            stop1_at: 0.0,
            stop1_ink: QColor::default(),
            stop2_at: 0.0,
            stop2_ink: QColor::default(),
            glow_visible: false,
            glow_center_x: 0.0,
            glow_center_y: 0.0,
            glow_radius_x: 0.0,
            glow_radius_y: 0.0,
            glow_stop0_at: 0.0,
            glow_stop0_ink: QColor::default(),
            glow_stop1_at: 0.0,
            glow_stop1_ink: QColor::default(),
            glow_stop2_at: 0.0,
            glow_stop2_ink: QColor::default(),
            glow_stop3_at: 0.0,
            glow_stop3_ink: QColor::default(),
            state,
        };
        rust.read_stops();
        rust
    }
}

impl StarFieldRust {
    fn now(&self) -> Duration {
        Duration::from_secs_f64(self.time.max(0.0))
    }

    fn read_stops(&mut self) {
        let [s0, s1, s2] = self.state.stops();
        (self.stop0_at, self.stop0_ink) = (s0.0, qcolor(s0.1));
        (self.stop1_at, self.stop1_ink) = (s1.0, qcolor(s1.1));
        (self.stop2_at, self.stop2_ink) = (s2.0, qcolor(s2.1));
    }

    fn read_glow(&mut self) {
        match self.state.glow_at(self.now(), [self.width, self.height]) {
            Some(drawn) => {
                self.glow_visible = true;
                self.glow_center_x = drawn.bounds[0] + drawn.bounds[2] / 2.0;
                self.glow_center_y = drawn.bounds[1] + drawn.bounds[3] / 2.0;
                self.glow_radius_x = drawn.bounds[2] / 2.0;
                self.glow_radius_y = drawn.bounds[3] / 2.0;
                let [g0, g1, g2, g3] = drawn.stops;
                (self.glow_stop0_at, self.glow_stop0_ink) = (g0.0, qcolor(g0.1));
                (self.glow_stop1_at, self.glow_stop1_ink) = (g1.0, qcolor(g1.1));
                (self.glow_stop2_at, self.glow_stop2_ink) = (g2.0, qcolor(g2.1));
                (self.glow_stop3_at, self.glow_stop3_ink) = (g3.0, qcolor(g3.1));
            }
            None => self.glow_visible = false,
        }
    }
}

impl cxx_qt::Initialize for qobject::StarField {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut().on_seed_changed(|this| this.replace_stars()).release();
        self.as_mut().on_count_changed(|this| this.replace_stars()).release();
        self.as_mut().on_moonrise_sky_changed(|this| this.replace_stars()).release();
        self.as_mut().on_width_changed(|this| this.redraw()).release();
        self.as_mut().on_height_changed(|this| this.redraw()).release();
        self.as_mut().on_stars_opacity_changed(|this| this.redraw()).release();
        self.as_mut()
            .on_glow_hue_changed(|mut this| {
                let (hue, now) = (this.glow_hue, this.now());
                this.as_mut().rust_mut().state.set_glow_hue((hue >= 0.0).then_some(hue), now);
                this.announce_glow();
            })
            .release();
        self.as_mut().on_glow_x_changed(|this| this.move_glow()).release();
        self.as_mut().on_glow_y_changed(|this| this.move_glow()).release();
    }
}

impl qobject::StarField {
    pub fn tick(mut self: Pin<&mut Self>, seconds: f64) {
        self.as_mut().rust_mut().time = seconds;
        self.as_mut().time_changed();
        self.redraw();
    }

    fn replace_stars(mut self: Pin<&mut Self>) {
        let (seed, count, moonrise) = (self.seed.max(0) as u32, self.count.max(0) as usize, self.moonrise_sky);
        self.as_mut().begin_reset_model();
        {
            let mut rust = self.as_mut().rust_mut();
            rust.state = SkyState::new(seed, count, moonrise);
            rust.read_stops();
            let (time, width, height, opacity) = (rust.time, rust.width, rust.height, rust.stars_opacity);
            rust.state.tick(time, [width, height], opacity);
        }
        self.as_mut().end_reset_model();
        self.as_mut().stop0_at_changed();
        self.as_mut().stop0_ink_changed();
        self.as_mut().stop1_at_changed();
        self.as_mut().stop1_ink_changed();
        self.as_mut().stop2_at_changed();
        self.as_mut().stop2_ink_changed();
        self.announce_glow();
    }

    fn move_glow(mut self: Pin<&mut Self>) {
        let (x, y, now) = (self.glow_x, self.glow_y, self.now());
        self.as_mut().rust_mut().state.set_glow_centre([x, y], now);
        self.announce_glow();
    }

    /// The stars' points for the time, size and opacity, then every row's roles and the glow announced.
    fn redraw(mut self: Pin<&mut Self>) {
        // No stars are drawn at no opacity, so the rows come and go with it, and the view is told the model is new (EmuSen_Kit.md §7.10).
        let resets = self.count > 0 && (self.stars_opacity > 0.0) == self.state.points.is_empty();
        if resets {
            self.as_mut().begin_reset_model();
        }
        let count = {
            let mut rust = self.as_mut().rust_mut();
            let (time, width, height, opacity) = (rust.time, rust.width, rust.height, rust.stars_opacity);
            rust.state.tick(time, [width, height], opacity);
            rust.state.points.len() as i32
        };
        if resets {
            self.as_mut().end_reset_model();
        } else if count > 0 {
            let (first, last) = (self.index(0, 0, &QModelIndex::default()), self.index(count - 1, 0, &QModelIndex::default()));
            self.as_mut().data_changed(&first, &last, &QVector::<i32>::default());
        }
        self.announce_glow();
    }

    fn announce_glow(mut self: Pin<&mut Self>) {
        let shown = self.glow_visible;
        self.as_mut().rust_mut().read_glow();
        if self.glow_visible != shown {
            self.as_mut().glow_visible_changed();
        }
        if !self.glow_visible {
            return;
        }
        self.as_mut().glow_center_x_changed();
        self.as_mut().glow_center_y_changed();
        self.as_mut().glow_radius_x_changed();
        self.as_mut().glow_radius_y_changed();
        self.as_mut().glow_stop0_at_changed();
        self.as_mut().glow_stop0_ink_changed();
        self.as_mut().glow_stop1_at_changed();
        self.as_mut().glow_stop1_ink_changed();
        self.as_mut().glow_stop2_at_changed();
        self.as_mut().glow_stop2_ink_changed();
        self.as_mut().glow_stop3_at_changed();
        self.as_mut().glow_stop3_ink_changed();
    }

    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let Some(point) = self.state.points.get(index.row() as usize) else {
            return QVariant::default();
        };
        match (StarRole { repr: role }) {
            StarRole::StarX => QVariant::from(&point.x),
            StarRole::StarY => QVariant::from(&point.y),
            StarRole::StarRadius => QVariant::from(&point.radius),
            StarRole::StarOpacity => QVariant::from(&point.opacity),
            _ => QVariant::default(),
        }
    }

    pub fn role_names(&self) -> QHash<QHashPair_i32_QByteArray> {
        let mut roles = QHash::default();
        for (role, name) in [
            (StarRole::StarX, "starX"),
            (StarRole::StarY, "starY"),
            (StarRole::StarRadius, "starRadius"),
            (StarRole::StarOpacity, "starOpacity"),
        ] {
            roles.insert(role.repr, QByteArray::from(name));
        }
        roles
    }

    pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
        self.state.points.len() as i32
    }
}
