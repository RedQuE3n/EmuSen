//! `WheelLayout`: kit-core's wheels and dial on Qt, read from any list model's group role, as lists a screen's items bind
//! to by index. Nodes are the rows, then the dial's stops: `places` of the screen's own, then one per group. The walk's
//! lists change only when the rows or the width do (`laidOut`); the wheel's turn follows `current` (`turned`), so a
//! screen's KeyNavigation stays put while the wheel turns (EmuSen_Kit.md §7.32).

use core::pin::Pin;

use cxx_qt::CxxQtType;
use cxx_qt_lib::{QList, QString, QVariant};
use emusen_kit_core::wheels::{turn, wheels, WheelGeometry, Wheels};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    unsafe extern "C++" {
        // The model is declared once, in item_model.rs: cxx-qt emits an upcast shim per declaration (EmuSen_Kit.md §7.6).
        include!(<QtCore/QAbstractItemModel>);
        type QAbstractItemModel = crate::item_model::ffi::QAbstractItemModel;
    }

    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(f64, width)]
        #[qproperty(i32, places)]
        #[qproperty(*mut QAbstractItemModel, model)]
        #[qproperty(QString, role)]
        #[qproperty(QString, weight_role)]
        #[qproperty(i32, current)]
        #[qproperty(i32, count, READ, NOTIFY = laid_out)]
        #[qproperty(i32, groups, READ, NOTIFY = laid_out)]
        #[qproperty(i32, reach, READ, NOTIFY = laid_out)]
        #[qproperty(f64, step, READ, NOTIFY = laid_out)]
        #[qproperty(f64, dial_width, READ, NOTIFY = laid_out)]
        #[qproperty(f64, view, READ, NOTIFY = laid_out)]
        #[qproperty(bool, scrolls, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, group_of, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, place_in, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, firsts, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, sizes, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, totals, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, ranks, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, dial_xs, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, left, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, right, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, up, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, down, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, previous, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, next, READ, NOTIFY = laid_out)]
        #[qproperty(i32, wheel, READ, NOTIFY = turned)]
        #[qproperty(bool, dimmed, READ, NOTIFY = turned)]
        #[qproperty(i32, hidden, READ, NOTIFY = turned)]
        #[qproperty(i32, lit, READ, NOTIFY = turned)]
        #[qproperty(f64, camera, READ, NOTIFY = turned)]
        #[qproperty(QVariant, turns, READ, NOTIFY = turned)]
        #[qproperty(QVariant, on_wheel, READ, NOTIFY = turned)]
        #[qproperty(QVariant, shown, READ, NOTIFY = turned)]
        type WheelLayout = super::WheelLayoutRust;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        /// The rows, the width or the places changed: every walk and dial list together.
        #[qsignal]
        fn laid_out(self: Pin<&mut WheelLayout>);

        /// The focus moved: the wheel, its turns and the camera together.
        #[qsignal]
        fn turned(self: Pin<&mut WheelLayout>);

        /// The model's rows read again, for a screen to call when the model resets or changes.
        #[qinvokable]
        fn read(self: Pin<&mut WheelLayout>);
    }

    impl cxx_qt::Initialize for WheelLayout {}
}

/// The inputs, the rows' group keys as last read, the layout kept for the turns, and both as the lists QML reads, each a
/// QList in a QVariant since qmllint cannot name cxx-qt's list aliases; a node is a row or `count` + a dial stop, -1 none.
pub struct WheelLayoutRust {
    width: f64,
    places: i32,
    model: *mut qobject::QAbstractItemModel,
    role: QString,
    weight_role: QString,
    current: i32,
    keys: Vec<String>,
    weights: Vec<f64>,
    laid: Option<Wheels>,
    count: i32,
    groups: i32,
    reach: i32,
    step: f64,
    dial_width: f64,
    view: f64,
    scrolls: bool,
    group_of: QVariant,
    place_in: QVariant,
    firsts: QVariant,
    sizes: QVariant,
    totals: QVariant,
    ranks: QVariant,
    dial_xs: QVariant,
    left: QVariant,
    right: QVariant,
    up: QVariant,
    down: QVariant,
    previous: QVariant,
    next: QVariant,
    wheel: i32,
    dimmed: bool,
    hidden: i32,
    lit: i32,
    camera: f64,
    turns: QVariant,
    on_wheel: QVariant,
    shown: QVariant,
}

impl Default for WheelLayoutRust {
    fn default() -> Self {
        Self {
            width: 0.0,
            places: 0,
            model: core::ptr::null_mut(),
            role: QString::default(),
            weight_role: QString::default(),
            current: -1,
            keys: Vec::new(),
            weights: Vec::new(),
            laid: None,
            count: 0,
            groups: 0,
            reach: 0,
            step: 0.0,
            dial_width: 0.0,
            view: 0.0,
            scrolls: false,
            group_of: QVariant::default(),
            place_in: QVariant::default(),
            firsts: QVariant::default(),
            sizes: QVariant::default(),
            totals: QVariant::default(),
            ranks: QVariant::default(),
            dial_xs: QVariant::default(),
            left: QVariant::default(),
            right: QVariant::default(),
            up: QVariant::default(),
            down: QVariant::default(),
            previous: QVariant::default(),
            next: QVariant::default(),
            wheel: -1,
            dimmed: false,
            hidden: 0,
            lit: -1,
            camera: 0.0,
            turns: QVariant::default(),
            on_wheel: QVariant::default(),
            shown: QVariant::default(),
        }
    }
}

impl cxx_qt::Initialize for qobject::WheelLayout {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut().on_width_changed(|this| this.lay_out()).release();
        self.as_mut().on_places_changed(|this| this.lay_out()).release();
        self.as_mut().on_model_changed(|this| this.read()).release();
        self.as_mut().on_role_changed(|this| this.read()).release();
        self.as_mut().on_weight_role_changed(|this| this.read()).release();
        self.as_mut().on_current_changed(|this| this.turn()).release();
    }
}

fn list<T: cxx_qt_lib::QListElement + Clone>(values: impl Iterator<Item = T>) -> QVariant
where
    QList<T>: cxx_qt_lib::QVariantValue,
{
    QVariant::from(&QList::from(&values.collect::<Vec<T>>()))
}

fn node(n: Option<usize>) -> i32 {
    n.map_or(-1, |i| i as i32)
}

impl qobject::WheelLayout {
    /// Each row's group key and weight, read as text from the roles the screen names; a model without the group's role is
    /// one group, and a weight that is not a number counts nothing.
    pub fn read(mut self: Pin<&mut Self>) {
        // SAFETY: QML hands the pointer of a live model, cleared by the engine when the model is destroyed.
        let model = unsafe { self.model.as_ref() };
        let keys = crate::item_model::role_texts(model, &self.role);
        let weights = crate::item_model::role_texts(model, &self.weight_role).iter().map(|t| t.parse().unwrap_or(0.0)).collect();
        let mut r = self.as_mut().rust_mut();
        r.keys = keys;
        r.weights = weights;
        self.lay_out();
    }

    fn lay_out(mut self: Pin<&mut Self>) {
        let g = WheelGeometry::WHEELWORK;
        let w = wheels(self.width, &self.keys, self.places.max(0) as usize, &g);
        let mut r = self.as_mut().rust_mut();
        r.count = w.group_of.len() as i32;
        r.groups = w.sizes.len() as i32;
        r.reach = g.reach as i32;
        r.step = g.step();
        r.dial_width = w.dial_width;
        r.view = w.view;
        r.scrolls = w.scrolls;
        r.group_of = list(w.group_of.iter().map(|x| *x as i32));
        r.place_in = list(w.place_in.iter().map(|x| *x as i32));
        r.firsts = list(w.firsts.iter().map(|x| *x as i32));
        r.sizes = list(w.sizes.iter().map(|x| *x as i32));
        r.totals = list(w.totals(&r.weights).into_iter());
        r.ranks = list(w.ranks().into_iter().map(|x| x as i32));
        r.dial_xs = list(w.dial_xs.iter().copied());
        r.left = list(w.steps.iter().map(|s| node(s.left)));
        r.right = list(w.steps.iter().map(|s| node(s.right)));
        r.up = list(w.steps.iter().map(|s| node(s.up)));
        r.down = list(w.steps.iter().map(|s| node(s.down)));
        r.previous = list(w.steps.iter().map(|s| node(s.previous)));
        r.next = list(w.steps.iter().map(|s| node(s.next)));
        r.laid = Some(w);
        self.as_mut().laid_out();
        self.turn();
    }

    fn turn(mut self: Pin<&mut Self>) {
        let Some(w) = self.laid.as_ref() else { return };
        let t = turn(w, usize::try_from(self.current).ok(), &WheelGeometry::WHEELWORK);
        let mut r = self.as_mut().rust_mut();
        r.wheel = node(t.wheel);
        r.dimmed = t.dimmed;
        r.hidden = t.hidden as i32;
        r.lit = node(t.lit);
        r.camera = t.camera;
        r.turns = list(t.turns.iter().map(|x| *x as i32));
        r.on_wheel = list(t.on_wheel.iter().copied());
        r.shown = list(t.shown.iter().copied());
        self.turned();
    }
}
