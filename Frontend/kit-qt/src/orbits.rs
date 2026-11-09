//! `OrbitLayout`: kit-core's orbits on Qt, read from any list model's group role, as lists a screen's items bind to by
//! index (EmuSen_Kit.md §7.30).

use core::pin::Pin;

use cxx_qt::CxxQtType;
use cxx_qt_lib::{QList, QString, QStringList, QVariant};
use emusen_kit_core::orbits::{orbit_layout, OrbitGeometry, Steps};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
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
        #[qproperty(i32, leading)]
        #[qproperty(*mut QAbstractItemModel, model)]
        #[qproperty(QString, role)]
        #[qproperty(QString, label_role)]
        #[qproperty(i32, current)]
        #[qproperty(f64, centre, READ, NOTIFY = laid_out)]
        #[qproperty(i32, orbit_count, READ, NOTIFY = laid_out)]
        #[qproperty(i32, focused_orbit, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, angles, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, orbit_of, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, shown, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, radii_x, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, radii_y, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, orbit_shown, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, sizes, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, turning, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, label_rights, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, label_tops, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, before_xs, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, before_ys, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, after_xs, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, after_ys, READ, NOTIFY = laid_out)]
        #[qproperty(QStringList, sections, READ, NOTIFY = laid_out)]
        #[qproperty(QString, section, READ, NOTIFY = laid_out)]
        #[qproperty(QStringList, labels, READ, NOTIFY = laid_out)]
        type OrbitLayout = super::OrbitLayoutRust;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        /// Every list changed together.
        #[qsignal]
        fn laid_out(self: Pin<&mut OrbitLayout>);

        /// The model's rows read again, for a screen to call when the model resets or changes.
        #[qinvokable]
        fn read(self: Pin<&mut OrbitLayout>);

        /// A leading item's centre across, which the walk goes up to and down from.
        #[qinvokable]
        fn lead(self: Pin<&mut OrbitLayout>, index: i32, centre: f64);

        /// Asks for the focus on the orbit of a section's key (the shoulders' step); `jumped` names the body's walk index.
        #[qinvokable]
        fn jump(self: Pin<&mut OrbitLayout>, key: &QString);

        /// A step from the current item: the target is laid out shown first, since it may wait on an orbit off screen,
        /// then named by `jumped`; a step with nowhere to go does nothing.
        #[qinvokable]
        fn step_left(self: Pin<&mut OrbitLayout>);
        #[qinvokable]
        fn step_right(self: Pin<&mut OrbitLayout>);
        #[qinvokable]
        fn step_up(self: Pin<&mut OrbitLayout>);
        #[qinvokable]
        fn step_down(self: Pin<&mut OrbitLayout>);

        #[qsignal]
        fn jumped(self: Pin<&mut OrbitLayout>, index: i32);
    }

    impl cxx_qt::Initialize for OrbitLayout {}
}

/// The inputs, the rows' group keys as last read, the base kept between layouts, and the layout as the lists QML reads;
/// an ellipsis' point is NaN where none is drawn.
pub struct OrbitLayoutRust {
    width: f64,
    leading: i32,
    model: *mut qobject::QAbstractItemModel,
    role: QString,
    label_role: QString,
    current: i32,
    groups: Vec<String>,
    names: Vec<String>,
    centres: Vec<f64>,
    base: Option<i32>,
    centre: f64,
    orbit_count: i32,
    focused_orbit: i32,
    angles: QVariant,
    orbit_of: QVariant,
    shown: QVariant,
    radii_x: QVariant,
    radii_y: QVariant,
    orbit_shown: QVariant,
    sizes: QVariant,
    turning: QVariant,
    label_rights: QVariant,
    label_tops: QVariant,
    before_xs: QVariant,
    before_ys: QVariant,
    after_xs: QVariant,
    after_ys: QVariant,
    sections: QStringList,
    section: QString,
    labels: QStringList,
    reaches: Vec<usize>,
    steps: Vec<Steps>,
}

impl Default for OrbitLayoutRust {
    fn default() -> Self {
        Self {
            width: 0.0,
            leading: 0,
            model: core::ptr::null_mut(),
            role: QString::default(),
            label_role: QString::default(),
            current: 0,
            groups: Vec::new(),
            names: Vec::new(),
            centres: Vec::new(),
            base: None,
            centre: 0.0,
            orbit_count: 0,
            focused_orbit: -1,
            angles: QVariant::default(),
            orbit_of: QVariant::default(),
            shown: QVariant::default(),
            radii_x: QVariant::default(),
            radii_y: QVariant::default(),
            orbit_shown: QVariant::default(),
            sizes: QVariant::default(),
            turning: QVariant::default(),
            label_rights: QVariant::default(),
            label_tops: QVariant::default(),
            before_xs: QVariant::default(),
            before_ys: QVariant::default(),
            after_xs: QVariant::default(),
            after_ys: QVariant::default(),
            sections: QStringList::default(),
            section: QString::default(),
            labels: QStringList::default(),
            reaches: Vec::new(),
            steps: Vec::new(),
        }
    }
}

impl cxx_qt::Initialize for qobject::OrbitLayout {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut().on_width_changed(|this| this.lay_out()).release();
        self.as_mut().on_leading_changed(|this| this.lay_out()).release();
        self.as_mut().on_current_changed(|this| this.lay_out()).release();
        self.as_mut().on_model_changed(|this| this.read()).release();
        self.as_mut().on_role_changed(|this| this.read()).release();
        self.as_mut().on_label_role_changed(|this| this.read()).release();
    }
}

fn list<T: cxx_qt_lib::QListElement + Clone>(values: impl Iterator<Item = T>) -> QVariant
where
    QList<T>: cxx_qt_lib::QVariantValue,
{
    QVariant::from(&QList::from(&values.collect::<Vec<T>>()))
}

impl qobject::OrbitLayout {
    /// Each row's group key and label, read as text from the roles the screen names (item_model::role_texts); a model
    /// without the group role holds every row in one group, and an orbit's label is its first row's.
    pub fn read(mut self: Pin<&mut Self>) {
        // SAFETY: QML hands the pointer of a live model, cleared by the engine when the model is destroyed.
        let model = unsafe { self.model.as_ref() };
        let groups = crate::item_model::role_texts(model, &self.role);
        let names = crate::item_model::role_texts(model, &self.label_role);
        let mut r = self.as_mut().rust_mut();
        if r.groups != groups {
            r.base = None;
        }
        r.groups = groups;
        r.names = names;
        self.lay_out();
    }

    pub fn lead(mut self: Pin<&mut Self>, index: i32, centre: f64) {
        let Ok(at) = usize::try_from(index) else { return };
        let mut r = self.as_mut().rust_mut();
        if r.centres.len() <= at {
            r.centres.resize(at + 1, 0.0);
        }
        r.centres[at] = centre;
        self.lay_out();
    }

    pub fn jump(self: Pin<&mut Self>, key: &QString) {
        let at = QList::<QString>::from(&self.sections).iter().position(|k| k == key);
        let target = at.and_then(|o| self.reaches.get(o).copied());
        self.go(target);
    }

    pub fn step_left(self: Pin<&mut Self>) {
        let target = self.steps.get(self.current.max(0) as usize).and_then(|s| s.left);
        self.go(target);
    }

    pub fn step_right(self: Pin<&mut Self>) {
        let target = self.steps.get(self.current.max(0) as usize).and_then(|s| s.right);
        self.go(target);
    }

    pub fn step_up(self: Pin<&mut Self>) {
        let target = self.steps.get(self.current.max(0) as usize).and_then(|s| s.up);
        self.go(target);
    }

    pub fn step_down(self: Pin<&mut Self>) {
        let target = self.steps.get(self.current.max(0) as usize).and_then(|s| s.down);
        self.go(target);
    }

    /// Lays the target out as the focus, so it is shown, then names it.
    fn go(mut self: Pin<&mut Self>, target: Option<usize>) {
        if let Some(t) = target {
            self.as_mut().set_current(t as i32);
            self.as_mut().jumped(t as i32);
        }
    }

    fn lay_out(mut self: Pin<&mut Self>) {
        let leading = self.leading.max(0) as usize;
        let mut centres = self.centres.clone();
        centres.resize(leading, 0.0);
        let focus = self.current.max(0) as usize;
        let l = orbit_layout(self.width, &self.groups, &centres, focus, self.base, &OrbitGeometry::REFERENCE);
        let point = |p: Option<(f64, f64)>, y: bool| p.map_or(f64::NAN, |(px, py)| if y { py } else { px });
        let mut r = self.as_mut().rust_mut();
        r.base = (!l.orbits.is_empty()).then_some(l.base);
        r.centre = r.width / 2.0;
        r.orbit_count = l.orbits.len() as i32;
        r.focused_orbit = focus.checked_sub(leading).and_then(|b| l.worlds.get(b)).map_or(-1, |w| w.orbit as i32);
        r.angles = list(l.worlds.iter().map(|w| w.angle));
        r.orbit_of = list(l.worlds.iter().map(|w| w.orbit as i32));
        r.shown = list(l.worlds.iter().map(|w| w.shown));
        r.radii_x = list(l.orbits.iter().map(|o| o.ellipse.rx));
        r.radii_y = list(l.orbits.iter().map(|o| o.ellipse.ry));
        r.orbit_shown = list(l.orbits.iter().map(|o| o.shown));
        r.sizes = list(l.orbits.iter().map(|o| o.size as i32));
        r.turning = list(l.orbits.iter().map(|o| o.turning));
        r.label_rights = list(l.orbits.iter().map(|o| o.label.0));
        r.label_tops = list(l.orbits.iter().map(|o| o.label.1));
        r.before_xs = list(l.orbits.iter().map(|o| point(o.before, false)));
        r.before_ys = list(l.orbits.iter().map(|o| point(o.before, true)));
        r.after_xs = list(l.orbits.iter().map(|o| point(o.after, false)));
        r.after_ys = list(l.orbits.iter().map(|o| point(o.after, true)));
        let keys: Vec<QString> = l.orbits.iter().map(|o| QString::from(r.groups[o.first].as_str())).collect();
        // On a leading item, the section before the innermost shown orbit, so the next shoulder lands on that orbit.
        let at = match focus.checked_sub(leading).and_then(|b| l.worlds.get(b)) {
            Some(w) => Some(w.orbit),
            None => l.orbits.iter().position(|o| o.shown).map(|o| (o + l.orbits.len() - 1) % l.orbits.len()),
        };
        r.section = at.map(|o| keys[o].clone()).unwrap_or_default();
        r.sections = QStringList::from(&QList::from(&keys));
        let labels: Vec<QString> = l.orbits.iter().map(|o| QString::from(r.names.get(o.first).map_or("", String::as_str))).collect();
        r.labels = QStringList::from(&QList::from(&labels));
        r.reaches = l.orbits.iter().map(|o| o.reach).collect();
        r.steps = l.walk;
        self.laid_out();
    }
}
