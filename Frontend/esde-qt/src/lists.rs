//! The list models a `View` binds, each reading the `EsdeModel` it is given as `scene` (EmuSen_Qt.md §4.8.4):
//! `ElementModel` (the frame's elements), `SlotsModel` (a carousel's slots), `RowsModel` (a list's or grid's rows) and
//! `PartsModel` (an element's badges, help entries, info or status items). Each refreshes when its `revision` changes,
//! resets only when its rows' identities change, and otherwise signals `dataChanged` for the rows that moved.

use core::pin::Pin;

use cxx_qt::CxxQtType;
use cxx_qt_lib::{QByteArray, QColor, QHash, QHashPair_i32_QByteArray, QList, QModelIndex, QString, QUrl, QVariant};
use emusen_esde::scene::{Frame, Layout, Paint, Placed, Primary, Row, Rows, Slot};

use crate::model::Shared;
use crate::values::{self, V};

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
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qlist.h");
        type QList_i32 = cxx_qt_lib::QList<i32>;
    }

    unsafe extern "C++Qt" {
        include!("emusen-esde-qt/src/model.cxxqt.h");
        type EsdeModel = crate::model::qobject::EsdeModel;
    }

    /// The `paint` role: which delegate draws an element.
    #[qenum(ElementModel)]
    enum PaintKind {
        None,
        Picture,
        Text,
        Rating,
        Badges,
        Help,
        Info,
        Status,
        Carousel,
        TextList,
        Grid,
    }

    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[base = QAbstractListModel]
        #[qml_element]
        #[qproperty(*mut EsdeModel, scene)]
        #[qproperty(i32, revision)]
        type ElementModel = super::ElementModelRust;

        #[qobject]
        #[base = QAbstractListModel]
        #[qml_element]
        #[qproperty(*mut EsdeModel, scene)]
        #[qproperty(i32, revision)]
        #[qproperty(bool, leaving)]
        type SlotsModel = super::SlotsModelRust;

        #[qobject]
        #[base = QAbstractListModel]
        #[qml_element]
        #[qproperty(*mut EsdeModel, scene)]
        #[qproperty(i32, revision)]
        #[qproperty(bool, leaving)]
        type RowsModel = super::RowsModelRust;

        #[qobject]
        #[base = QAbstractListModel]
        #[qml_element]
        #[qproperty(*mut EsdeModel, scene)]
        #[qproperty(i32, revision)]
        #[qproperty(QString, element)]
        type PartsModel = super::PartsModelRust;
    }

    extern "RustQt" {
        #[cxx_override]
        fn data(self: &ElementModel, index: &QModelIndex, role: i32) -> QVariant;
        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &ElementModel) -> QHash_i32_QByteArray;
        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &ElementModel, parent: &QModelIndex) -> i32;

        #[cxx_override]
        fn data(self: &SlotsModel, index: &QModelIndex, role: i32) -> QVariant;
        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &SlotsModel) -> QHash_i32_QByteArray;
        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &SlotsModel, parent: &QModelIndex) -> i32;

        #[cxx_override]
        fn data(self: &RowsModel, index: &QModelIndex, role: i32) -> QVariant;
        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &RowsModel) -> QHash_i32_QByteArray;
        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &RowsModel, parent: &QModelIndex) -> i32;

        #[cxx_override]
        fn data(self: &PartsModel, index: &QModelIndex, role: i32) -> QVariant;
        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &PartsModel) -> QHash_i32_QByteArray;
        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &PartsModel, parent: &QModelIndex) -> i32;
    }

    unsafe extern "RustQt" {
        #[inherit]
        #[cxx_name = "beginResetModel"]
        fn begin_reset_model(self: Pin<&mut ElementModel>);
        #[inherit]
        #[cxx_name = "endResetModel"]
        fn end_reset_model(self: Pin<&mut ElementModel>);
        #[inherit]
        fn index(self: &ElementModel, row: i32, column: i32, parent: &QModelIndex) -> QModelIndex;
        #[inherit]
        #[qsignal]
        #[cxx_name = "dataChanged"]
        fn data_changed(self: Pin<&mut ElementModel>, top_left: &QModelIndex, bottom_right: &QModelIndex, roles: &QList_i32);

        #[inherit]
        #[cxx_name = "beginResetModel"]
        fn begin_reset_model(self: Pin<&mut SlotsModel>);
        #[inherit]
        #[cxx_name = "endResetModel"]
        fn end_reset_model(self: Pin<&mut SlotsModel>);
        #[inherit]
        fn index(self: &SlotsModel, row: i32, column: i32, parent: &QModelIndex) -> QModelIndex;
        #[inherit]
        #[qsignal]
        #[cxx_name = "dataChanged"]
        fn data_changed(self: Pin<&mut SlotsModel>, top_left: &QModelIndex, bottom_right: &QModelIndex, roles: &QList_i32);

        #[inherit]
        #[cxx_name = "beginResetModel"]
        fn begin_reset_model(self: Pin<&mut RowsModel>);
        #[inherit]
        #[cxx_name = "endResetModel"]
        fn end_reset_model(self: Pin<&mut RowsModel>);

        #[inherit]
        #[cxx_name = "beginResetModel"]
        fn begin_reset_model(self: Pin<&mut PartsModel>);
        #[inherit]
        #[cxx_name = "endResetModel"]
        fn end_reset_model(self: Pin<&mut PartsModel>);
        #[inherit]
        fn index(self: &PartsModel, row: i32, column: i32, parent: &QModelIndex) -> QModelIndex;
        #[inherit]
        #[qsignal]
        #[cxx_name = "dataChanged"]
        fn data_changed(self: Pin<&mut PartsModel>, top_left: &QModelIndex, bottom_right: &QModelIndex, roles: &QList_i32);
    }

    impl cxx_qt::Initialize for ElementModel {}
    impl cxx_qt::Initialize for SlotsModel {}
    impl cxx_qt::Initialize for RowsModel {}
    impl cxx_qt::Initialize for PartsModel {}
}

use qobject::{EsdeModel, PaintKind};

fn paint_number(kind: &str) -> i32 {
    match kind {
        "picture" => PaintKind::Picture,
        "text" => PaintKind::Text,
        "rating" => PaintKind::Rating,
        "badges" => PaintKind::Badges,
        "help" => PaintKind::Help,
        "info" => PaintKind::Info,
        "status" => PaintKind::Status,
        "carousel" => PaintKind::Carousel,
        "textlist" => PaintKind::TextList,
        "grid" => PaintKind::Grid,
        _ => PaintKind::None,
    }
    .repr
}

/// Role numbers start past Qt's own, at Qt::UserRole.
const USER_ROLE: i32 = 0x100;

fn variant(v: V) -> QVariant {
    match v {
        V::N(n) => QVariant::from(&n),
        V::I(i) => QVariant::from(&i),
        V::B(b) => QVariant::from(&b),
        V::S(s) => QVariant::from(&QString::from(s.as_str())),
        V::U(p) if p.is_empty() => QVariant::from(&QUrl::default()),
        V::U(p) => QVariant::from(&QUrl::from_local_file(&QString::from(p.as_str()))),
        V::C(c) => QVariant::from(&QColor::from_rgba(c.r().into(), c.g().into(), c.b().into(), c.a().into())),
        V::None => QVariant::default(),
    }
}

fn names<'a>(roles: impl Iterator<Item = &'a str>) -> QHash<QHashPair_i32_QByteArray> {
    let mut hash = QHash::<QHashPair_i32_QByteArray>::default();
    for (i, name) in roles.enumerate() {
        hash.insert(USER_ROLE + i as i32, QByteArray::from(name));
    }
    hash
}

fn role_of<'a>(roles: &[&'a str], role: i32) -> Option<&'a str> {
    usize::try_from(role - USER_ROLE).ok().and_then(|i| roles.get(i)).copied()
}

fn element_roles() -> impl Iterator<Item = &'static str> {
    values::ELEMENT_ROLES.into_iter().chain(values::GRID_ROLES)
}

// The painter of the EsdeModel a list was given; the list keeps a share of it, not the pointer.
fn shared_of(scene: *mut EsdeModel) -> Option<Shared> {
    // SAFETY: QML hands a live EsdeModel or null; it is read once, here, on the GUI thread.
    unsafe { scene.as_ref() }.map(|m| m.rust().shared.clone())
}

fn primary_of(frame: &Frame, leaving: bool) -> Option<&Primary> {
    if leaving {
        frame.leaving_primary.as_ref()
    } else {
        frame.primary.as_ref()
    }
}

/// The rows a primary shows, kept with their list and the install they came from.
#[derive(Default)]
struct RowCache {
    key: Option<(Rows, u64)>,
    rows: Vec<Row>,
}

impl RowCache {
    // True when the rows were replaced.
    fn update(&mut self, shared: &Shared, leaving: bool) -> bool {
        let p = shared.borrow();
        let key = primary_of(&p.frame, leaving).map(|pr| (pr.rows, p.generation));
        if key == self.key {
            return false;
        }
        self.rows = key.map(|(rows, _)| p.rows(rows)).unwrap_or_default();
        self.key = key;
        true
    }
}

pub struct ElementModelRust {
    scene: *mut EsdeModel,
    revision: i32,
    shared: Option<Shared>,
    shown: Vec<Placed>,
    primaries: [Option<Primary>; 2],
}

impl Default for ElementModelRust {
    fn default() -> Self {
        ElementModelRust { scene: std::ptr::null_mut(), revision: 0, shared: None, shown: Vec::new(), primaries: [None, None] }
    }
}

impl ElementModelRust {
    fn primary_for(&self, p: &Placed) -> Option<&Primary> {
        match p.paint {
            Paint::Primary { leaving } => self.primaries[leaving as usize].as_ref(),
            _ => None,
        }
    }
}

impl cxx_qt::Initialize for qobject::ElementModel {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut()
            .on_scene_changed(|mut m| {
                let shared = shared_of(*m.scene());
                m.as_mut().rust_mut().shared = shared;
                m.refresh();
            })
            .release();
        self.as_mut().on_revision_changed(|m| m.refresh()).release();
    }
}

impl qobject::ElementModel {
    /// The frame's elements: a reset when the list of ids or kinds changes, else `dataChanged` for each row that moved.
    /// The painter is read and released before any signal, since a delegate may call back into it (a measurement).
    fn refresh(mut self: Pin<&mut Self>) {
        let Some(shared) = self.shared.clone() else { return };
        let (elements, primaries) = {
            let p = shared.borrow();
            (p.frame.elements.clone(), [p.frame.primary.clone(), p.frame.leaving_primary.clone()])
        };
        let layout = |x: &Option<Primary>| x.as_ref().map(|x| std::mem::discriminant(&x.layout));
        let same = elements.len() == self.shown.len()
            && elements.iter().zip(&self.shown).all(|(a, b)| a.id == b.id && std::mem::discriminant(&a.paint) == std::mem::discriminant(&b.paint))
            && primaries.iter().zip(&self.primaries).all(|(a, b)| layout(a) == layout(b));
        if !same {
            self.as_mut().begin_reset_model();
            self.as_mut().rust_mut().shown = elements;
            self.as_mut().rust_mut().primaries = primaries;
            self.as_mut().end_reset_model();
            return;
        }
        let primary_moved = [0, 1].map(|i| primaries[i] != self.primaries[i]);
        let moved: Vec<usize> = (0..elements.len())
            .filter(|&i| self.shown[i] != elements[i] || matches!(elements[i].paint, Paint::Primary { leaving } if primary_moved[leaving as usize]))
            .collect();
        self.as_mut().rust_mut().shown = elements;
        self.as_mut().rust_mut().primaries = primaries;
        for i in moved {
            let at = self.index(i as i32, 0, &QModelIndex::default());
            self.as_mut().data_changed(&at, &at, &QList::default());
        }
    }

    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let name = role_of(&values::ELEMENT_ROLES, role).or_else(|| role_of(&values::GRID_ROLES, role - values::ELEMENT_ROLES.len() as i32));
        match (self.shown.get(index.row() as usize), name) {
            (Some(p), Some("paint")) => QVariant::from(&paint_number(values::paint_kind(p, self.primary_for(p)))),
            (Some(p), Some(name)) => variant(values::element(p, self.primary_for(p), name)),
            _ => QVariant::default(),
        }
    }

    pub fn role_names(&self) -> QHash<QHashPair_i32_QByteArray> {
        names(element_roles())
    }

    pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
        self.shown.len() as i32
    }
}

pub struct SlotsModelRust {
    scene: *mut EsdeModel,
    revision: i32,
    leaving: bool,
    shared: Option<Shared>,
    slots: Vec<Slot>,
    rows: RowCache,
}

impl Default for SlotsModelRust {
    fn default() -> Self {
        SlotsModelRust { scene: std::ptr::null_mut(), revision: 0, leaving: false, shared: None, slots: Vec::new(), rows: RowCache::default() }
    }
}

impl cxx_qt::Initialize for qobject::SlotsModel {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut()
            .on_scene_changed(|mut m| {
                let shared = shared_of(*m.scene());
                m.as_mut().rust_mut().shared = shared;
                m.refresh();
            })
            .release();
        self.as_mut().on_revision_changed(|m| m.refresh()).release();
        self.as_mut().on_leaving_changed(|m| m.refresh()).release();
    }
}

impl qobject::SlotsModel {
    /// The carousel's slots: rows added or removed only when the count changes, else `dataChanged` per moved slot.
    fn refresh(mut self: Pin<&mut Self>) {
        let Some(shared) = self.shared.clone() else { return };
        let leaving = self.leaving;
        let rows_changed = self.as_mut().rust_mut().rows.update(&shared, leaving);
        let slots: Vec<Slot> = match primary_of(&shared.borrow().frame, leaving).map(|pr| &pr.layout) {
            Some(Layout::Carousel(c)) => c.slots.clone(),
            _ => Vec::new(),
        };
        if rows_changed || slots.len() != self.slots.len() {
            self.as_mut().begin_reset_model();
            self.as_mut().rust_mut().slots = slots;
            self.as_mut().end_reset_model();
            return;
        }
        let moved: Vec<usize> = (0..slots.len()).filter(|&i| self.slots[i] != slots[i]).collect();
        self.as_mut().rust_mut().slots = slots;
        for i in moved {
            let at = self.index(i as i32, 0, &QModelIndex::default());
            self.as_mut().data_changed(&at, &at, &QList::default());
        }
    }

    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        match (self.slots.get(index.row() as usize), role_of(&values::SLOT_ROLES, role)) {
            (Some(s), Some(name)) => variant(values::slot(s, self.rows.rows.get(s.row), name)),
            _ => QVariant::default(),
        }
    }

    pub fn role_names(&self) -> QHash<QHashPair_i32_QByteArray> {
        names(values::SLOT_ROLES.into_iter())
    }

    pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
        self.slots.len() as i32
    }
}

pub struct RowsModelRust {
    scene: *mut EsdeModel,
    revision: i32,
    leaving: bool,
    shared: Option<Shared>,
    rows: RowCache,
}

impl Default for RowsModelRust {
    fn default() -> Self {
        RowsModelRust { scene: std::ptr::null_mut(), revision: 0, leaving: false, shared: None, rows: RowCache::default() }
    }
}

impl cxx_qt::Initialize for qobject::RowsModel {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut()
            .on_scene_changed(|mut m| {
                let shared = shared_of(*m.scene());
                m.as_mut().rust_mut().shared = shared;
                m.refresh();
            })
            .release();
        self.as_mut().on_revision_changed(|m| m.refresh()).release();
        self.as_mut().on_leaving_changed(|m| m.refresh()).release();
    }
}

impl qobject::RowsModel {
    /// A list's rows, replaced whole under a reset when the list or the install changes; a step changes none of them.
    fn refresh(mut self: Pin<&mut Self>) {
        let Some(shared) = self.shared.clone() else { return };
        let leaving = self.leaving;
        let mut fresh = RowCache { key: self.rows.key, rows: Vec::new() };
        if fresh.update(&shared, leaving) {
            self.as_mut().begin_reset_model();
            self.as_mut().rust_mut().rows = fresh;
            self.as_mut().end_reset_model();
        }
    }

    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        match (self.rows.rows.get(index.row() as usize), role_of(&values::ROW_ROLES, role)) {
            (Some(r), Some(name)) => variant(values::row(r, name)),
            _ => QVariant::default(),
        }
    }

    pub fn role_names(&self) -> QHash<QHashPair_i32_QByteArray> {
        names(values::ROW_ROLES.into_iter())
    }

    pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
        self.rows.rows.len() as i32
    }
}

pub struct PartsModelRust {
    scene: *mut EsdeModel,
    revision: i32,
    element: QString,
    shared: Option<Shared>,
    paint: Option<Paint>,
    swap: bool,
}

impl Default for PartsModelRust {
    fn default() -> Self {
        PartsModelRust { scene: std::ptr::null_mut(), revision: 0, element: QString::default(), shared: None, paint: None, swap: false }
    }
}

impl cxx_qt::Initialize for qobject::PartsModel {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut()
            .on_scene_changed(|mut m| {
                let shared = shared_of(*m.scene());
                m.as_mut().rust_mut().shared = shared;
                m.refresh();
            })
            .release();
        self.as_mut().on_revision_changed(|m| m.refresh()).release();
        self.as_mut().on_element_changed(|m| m.refresh()).release();
    }
}

impl qobject::PartsModel {
    /// One element's parts: a reset when their count changes, else `dataChanged` over them when the paint changed.
    fn refresh(mut self: Pin<&mut Self>) {
        let Some(shared) = self.shared.clone() else { return };
        let id = self.element.to_string().parse::<u64>().ok();
        let (paint, swap) = {
            let p = shared.borrow();
            (id.and_then(|id| p.frame.elements.iter().find(|e| e.id == id)).map(|e| e.paint.clone()), p.swap)
        };
        let paint = paint.as_ref();
        if paint == self.paint.as_ref() && swap == self.swap {
            return;
        }
        let count = |q: Option<&Paint>| q.map_or(0, values::part_count);
        let (before, after) = (count(self.paint.as_ref()), count(paint));
        if before != after {
            self.as_mut().begin_reset_model();
            self.as_mut().rust_mut().paint = paint.cloned();
            self.as_mut().rust_mut().swap = swap;
            self.as_mut().end_reset_model();
        } else {
            self.as_mut().rust_mut().paint = paint.cloned();
            self.as_mut().rust_mut().swap = swap;
            if after > 0 {
                let (top, bottom) = (self.index(0, 0, &QModelIndex::default()), self.index(after as i32 - 1, 0, &QModelIndex::default()));
                self.as_mut().data_changed(&top, &bottom, &QList::default());
            }
        }
    }

    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        match (&self.paint, role_of(&values::PART_ROLES, role)) {
            (Some(p), Some(name)) => variant(values::part(p, index.row() as usize, name, self.swap)),
            _ => QVariant::default(),
        }
    }

    pub fn role_names(&self) -> QHash<QHashPair_i32_QByteArray> {
        names(values::PART_ROLES.into_iter())
    }

    pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
        self.paint.as_ref().map_or(0, values::part_count) as i32
    }
}
