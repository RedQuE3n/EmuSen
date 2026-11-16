//! `GroupGridLayout`: kit-core's group grid on Qt, as index lists a screen's items bind to; -1 is none (EmuSen_Kit.md §7.120).

use core::pin::Pin;

use cxx_qt::CxxQtType;
use cxx_qt_lib::{QList, QString, QVariant};
use emusen_kit_core::group_grid::group_grid;

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
        #[qproperty(*mut QAbstractItemModel, model)]
        #[qproperty(QString, group_role)]
        #[qproperty(QString, index_role)]
        #[qproperty(i32, columns)]
        #[qproperty(i32, count, READ, NOTIFY = laid_out)]
        #[qproperty(i32, groups, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, column_of, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, row_of, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, left, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, right, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, up, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, down, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, firsts, READ, NOTIFY = laid_out)]
        #[qproperty(QVariant, rows, READ, NOTIFY = laid_out)]
        type GroupGridLayout = super::GroupGridLayoutRust;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        /// Every list changed together.
        #[qsignal]
        fn laid_out(self: Pin<&mut GroupGridLayout>);

        /// The model's rows read again, for a screen to call when the model resets or changes.
        #[qinvokable]
        fn read(self: Pin<&mut GroupGridLayout>);
    }

    impl cxx_qt::Initialize for GroupGridLayout {}
}

/// The inputs, each row's group and place as last read, and the layout as the lists QML reads, each a QList in a QVariant.
pub struct GroupGridLayoutRust {
    model: *mut qobject::QAbstractItemModel,
    group_role: QString,
    index_role: QString,
    columns: i32,
    places: Vec<(usize, usize)>,
    count: i32,
    groups: i32,
    column_of: QVariant,
    row_of: QVariant,
    left: QVariant,
    right: QVariant,
    up: QVariant,
    down: QVariant,
    firsts: QVariant,
    rows: QVariant,
}

impl Default for GroupGridLayoutRust {
    fn default() -> Self {
        Self {
            model: core::ptr::null_mut(),
            group_role: QString::from("group"),
            index_role: QString::from("groupIndex"),
            columns: 1,
            places: Vec::new(),
            count: 0,
            groups: 0,
            column_of: QVariant::default(),
            row_of: QVariant::default(),
            left: QVariant::default(),
            right: QVariant::default(),
            up: QVariant::default(),
            down: QVariant::default(),
            firsts: QVariant::default(),
            rows: QVariant::default(),
        }
    }
}

impl cxx_qt::Initialize for qobject::GroupGridLayout {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut().on_columns_changed(|this| this.lay_out()).release();
        self.as_mut().on_model_changed(|this| this.read()).release();
        self.as_mut().on_group_role_changed(|this| this.read()).release();
        self.as_mut().on_index_role_changed(|this| this.read()).release();
    }
}

fn list(values: impl Iterator<Item = i32>) -> QVariant {
    QVariant::from(&QList::from(&values.collect::<Vec<i32>>()))
}

fn index(n: Option<usize>) -> i32 {
    n.map_or(-1, |i| i as i32)
}

impl qobject::GroupGridLayout {
    /// Each row's group and place, read as numbers from the roles the screen names; a row without them is group 0, place 0.
    pub fn read(mut self: Pin<&mut Self>) {
        // SAFETY: QML hands the pointer of a live model, cleared by the engine when the model is destroyed.
        let model = unsafe { self.model.as_ref() };
        let number = |texts: Vec<String>| texts.iter().map(|t| t.parse::<usize>().unwrap_or(0)).collect::<Vec<_>>();
        let groups = number(crate::item_model::role_texts(model, &self.group_role));
        let places = number(crate::item_model::role_texts(model, &self.index_role));
        self.as_mut().rust_mut().places = groups.into_iter().zip(places).collect();
        self.lay_out();
    }

    fn lay_out(mut self: Pin<&mut Self>) {
        let grid = group_grid(&self.places, self.columns.max(1) as usize);
        let mut r = self.as_mut().rust_mut();
        r.count = grid.items.len() as i32;
        r.groups = grid.firsts.len() as i32;
        r.column_of = list(grid.items.iter().map(|i| i.column as i32));
        r.row_of = list(grid.items.iter().map(|i| i.row as i32));
        r.left = list(grid.items.iter().map(|i| index(i.left)));
        r.right = list(grid.items.iter().map(|i| index(i.right)));
        r.up = list(grid.items.iter().map(|i| index(i.up)));
        r.down = list(grid.items.iter().map(|i| index(i.down)));
        r.firsts = list(grid.firsts.iter().map(|f| index(*f)));
        r.rows = list(grid.rows.iter().map(|n| *n as i32));
        self.laid_out();
    }
}
