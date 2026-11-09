//! `FigureLayout`: kit-core's figures chart on Qt, read from any list model's group role, as lists a screen's items bind to
//! by index; the three pointers follow the items, at `count`, `count + 1` and `count + 2` (EmuSen_Kit.md §7.31).

use core::pin::Pin;

use cxx_qt::CxxQtType;
use cxx_qt_lib::{QList, QModelIndex, QString, QVariant};
use emusen_kit_core::figures::{figure_layout, FigureGeometry, Node};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qmodelindex.h");
        type QModelIndex = cxx_qt_lib::QModelIndex;
        include!("cxx-qt-lib/qhash.h");
        type QHash_i32_QByteArray = cxx_qt_lib::QHash<cxx_qt_lib::QHashPair_i32_QByteArray>;
    }

    unsafe extern "C++Qt" {
        include!(<QtCore/QAbstractItemModel>);
        #[qobject]
        type QAbstractItemModel;
        #[cxx_name = "rowCount"]
        fn row_count(self: &QAbstractItemModel, parent: &QModelIndex) -> i32;
        fn index(self: &QAbstractItemModel, row: i32, column: i32, parent: &QModelIndex) -> QModelIndex;
        fn data(self: &QAbstractItemModel, index: &QModelIndex, role: i32) -> QVariant;
        #[cxx_name = "roleNames"]
        fn role_names(self: &QAbstractItemModel) -> QHash_i32_QByteArray;
    }

    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(f64, width)]
        #[qproperty(*mut QAbstractItemModel, model)]
        #[qproperty(QString, role)]
        #[qproperty(i32, count, READ, NOTIFY = laid_out)]
        #[qproperty(bool, compact, READ, NOTIFY = laid_out)]
        #[qproperty(f64, body, READ, NOTIFY = laid_out)]
        #[qproperty(f64, label_width, READ, NOTIFY = laid_out)]
        #[qproperty(f64, zone, READ, NOTIFY = laid_out)]
        #[qproperty(f64, content_height, READ, NOTIFY = laid_out)]
        #[qproperty(bool, scrolls, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, xs, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, ys, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, raised, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, left, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, right, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, up, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, down, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, prior, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, group_of, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, figure_xs, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, figure_ys, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, firsts, READ, NOTIFY = laid_out)]
        type FigureLayout = super::FigureLayoutRust;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        /// Every list changed together.
        #[qsignal]
        fn laid_out(self: Pin<&mut FigureLayout>);

        /// The model's rows read again, for a screen to call when the model resets or changes.
        #[qinvokable]
        fn read(self: Pin<&mut FigureLayout>);
    }

    impl cxx_qt::Initialize for FigureLayout {}
}

/// The inputs, the rows' group keys as last read, and the layout as the lists QML reads, each a QList in a QVariant since
/// qmllint cannot name cxx-qt's list aliases; a node is an item's row, or `count` + a pointer, and -1 is none.
pub struct FigureLayoutRust {
    width: f64,
    model: *mut qobject::QAbstractItemModel,
    role: QString,
    keys: Vec<String>,
    count: i32,
    compact: bool,
    body: f64,
    label_width: f64,
    zone: f64,
    content_height: f64,
    scrolls: bool,
    xs: QVariant,
    ys: QVariant,
    raised: QVariant,
    left: QVariant,
    right: QVariant,
    up: QVariant,
    down: QVariant,
    prior: QVariant,
    group_of: QVariant,
    figure_xs: QVariant,
    figure_ys: QVariant,
    firsts: QVariant,
}

impl Default for FigureLayoutRust {
    fn default() -> Self {
        Self {
            width: 0.0,
            model: core::ptr::null_mut(),
            role: QString::default(),
            keys: Vec::new(),
            count: 0,
            compact: false,
            body: 0.0,
            label_width: 0.0,
            zone: 0.0,
            content_height: 0.0,
            scrolls: false,
            xs: QVariant::default(),
            ys: QVariant::default(),
            raised: QVariant::default(),
            left: QVariant::default(),
            right: QVariant::default(),
            up: QVariant::default(),
            down: QVariant::default(),
            prior: QVariant::default(),
            group_of: QVariant::default(),
            figure_xs: QVariant::default(),
            figure_ys: QVariant::default(),
            firsts: QVariant::default(),
        }
    }
}

impl cxx_qt::Initialize for qobject::FigureLayout {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut().on_width_changed(|this| this.lay_out()).release();
        self.as_mut().on_model_changed(|this| this.read()).release();
        self.as_mut().on_role_changed(|this| this.read()).release();
    }
}

fn list<T: cxx_qt_lib::QListElement + Clone>(values: impl Iterator<Item = T>) -> QVariant
where
    QList<T>: cxx_qt_lib::QVariantValue,
{
    QVariant::from(&QList::from(&values.collect::<Vec<T>>()))
}

impl qobject::FigureLayout {
    /// Each row's group key, read as text (a number's digits) from the role the screen names; a model without the role is one group.
    pub fn read(mut self: Pin<&mut Self>) {
        let mut keys = Vec::new();
        // SAFETY: QML hands the pointer of a live model, cleared by the engine when the model is destroyed.
        if let Some(model) = unsafe { self.model.as_ref() } {
            let root = QModelIndex::default();
            let wanted = self.role.to_string();
            let role = model.role_names().iter().find(|(_, name)| name.to_string() == wanted).map(|(r, _)| *r);
            keys = (0..model.row_count(&root))
                .map(|row| {
                    let value = role.map(|r| model.data(&model.index(row, 0, &root), r)).unwrap_or_default();
                    value.value::<QString>().map(|s| s.to_string()).unwrap_or_default()
                })
                .collect();
        }
        self.as_mut().rust_mut().keys = keys;
        self.lay_out();
    }

    fn lay_out(mut self: Pin<&mut Self>) {
        let l = figure_layout(self.width, &self.keys, &FigureGeometry::CHART);
        let n = l.items.len();
        let at = |node: Option<Node>| match node {
            Some(Node::Item(i)) => i as i32,
            Some(Node::Pointer(p)) => (n + p) as i32,
            None => -1,
        };
        let nodes: Vec<_> = l.items.iter().chain(l.pointers.iter()).copied().collect();
        let mut r = self.as_mut().rust_mut();
        r.count = n as i32;
        r.compact = l.compact;
        r.body = l.tier.body;
        r.label_width = l.tier.label_width;
        r.zone = l.tier.zone;
        r.content_height = l.height;
        r.scrolls = l.scrolls;
        r.xs = list(nodes.iter().map(|p| p.x));
        r.ys = list(nodes.iter().map(|p| p.y));
        r.raised = list(nodes.iter().map(|p| p.raised));
        r.left = list(nodes.iter().map(|p| at(p.left)));
        r.right = list(nodes.iter().map(|p| at(p.right)));
        r.up = list(nodes.iter().map(|p| at(p.up)));
        r.down = list(nodes.iter().map(|p| at(p.down)));
        r.prior = list(l.prior.iter().map(|p| p.map_or(-1, |i| i as i32)));
        r.group_of = list(l.group_of.iter().map(|g| *g as i32));
        r.figure_xs = list(l.figures.iter().map(|f| f.x));
        r.figure_ys = list(l.figures.iter().map(|f| f.y));
        r.firsts = list(l.figures.iter().map(|f| f.first as i32));
        self.laid_out();
    }
}
