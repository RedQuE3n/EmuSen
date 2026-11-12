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
        type QAbstractListModel = emusen_kit_qt::base::ffi::QAbstractListModel;
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
        Clip,
        Page,
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
        // Named for the crate, since qt-models' RowsModel links into the same program; QML knows it as RowsModel.
        #[qml_element = "RowsModel"]
        #[qproperty(*mut EsdeModel, scene)]
        #[qproperty(i32, revision)]
        #[qproperty(bool, leaving)]
        type EsdeRowsModel = super::EsdeRowsModelRust;

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
        fn data(self: &EsdeRowsModel, index: &QModelIndex, role: i32) -> QVariant;
        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &EsdeRowsModel) -> QHash_i32_QByteArray;
        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &EsdeRowsModel, parent: &QModelIndex) -> i32;

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
        #[cxx_name = "beginRemoveRows"]
        fn begin_remove_rows(self: Pin<&mut ElementModel>, parent: &QModelIndex, first: i32, last: i32);
        #[inherit]
        #[cxx_name = "endRemoveRows"]
        fn end_remove_rows(self: Pin<&mut ElementModel>);
        #[inherit]
        #[cxx_name = "beginInsertRows"]
        fn begin_insert_rows(self: Pin<&mut ElementModel>, parent: &QModelIndex, first: i32, last: i32);
        #[inherit]
        #[cxx_name = "endInsertRows"]
        fn end_insert_rows(self: Pin<&mut ElementModel>);
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
        fn begin_reset_model(self: Pin<&mut EsdeRowsModel>);
        #[inherit]
        #[cxx_name = "endResetModel"]
        fn end_reset_model(self: Pin<&mut EsdeRowsModel>);

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
    impl cxx_qt::Initialize for EsdeRowsModel {}
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
        "clip" => PaintKind::Clip,
        "page" => PaintKind::Page,
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

// A list follows its EsdeModel: a new scene takes a share of the painter, and each change of it or of the revision refreshes.
// A macro, since cxx-qt generates each `on_*_changed` as an inherent method of its own type, with no trait to be generic over.
macro_rules! follow_scene {
    ($model:ident) => {
        $model
            .as_mut()
            .on_scene_changed(|mut m| {
                let shared = shared_of(*m.scene());
                m.as_mut().rust_mut().shared = shared;
                m.refresh();
            })
            .release();
        $model.as_mut().on_revision_changed(|m| m.refresh()).release();
    };
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

/// The longest run of keys the two lists share in order, as (old index, new index) pairs: the rows a change keeps.
fn common<K: PartialEq>(old: &[K], new: &[K]) -> Vec<(usize, usize)> {
    let (n, m) = (old.len(), new.len());
    let mut length = vec![0u32; (n + 1) * (m + 1)];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            length[i * (m + 1) + j] =
                if old[i] == new[j] { length[(i + 1) * (m + 1) + j + 1] + 1 } else { length[(i + 1) * (m + 1) + j].max(length[i * (m + 1) + j + 1]) };
        }
    }
    let (mut i, mut j, mut out) = (0, 0, Vec::new());
    while i < n && j < m {
        if old[i] == new[j] {
            out.push((i, j));
            (i, j) = (i + 1, j + 1);
        } else if length[(i + 1) * (m + 1) + j] >= length[i * (m + 1) + j + 1] {
            i += 1;
        } else {
            j += 1;
        }
    }
    out
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
        follow_scene!(self);
    }
}

impl qobject::ElementModel {
    /// The frame's elements: an element that leaves is a removed row and one that comes an inserted row, so that every
    /// other delegate lives on (a playing clip among them, EmuSen_Qt.md §4.14), then `dataChanged` for each kept row
    /// that moved; a reset only when a primary's layout changes. The painter is read and released before any signal,
    /// since a delegate may call back into it (a measurement).
    fn refresh(mut self: Pin<&mut Self>) {
        let Some(shared) = self.shared.clone() else { return };
        let (elements, primaries) = {
            let p = shared.borrow();
            (p.frame.elements.clone(), [p.frame.primary.clone(), p.frame.leaving_primary.clone()])
        };
        let layout = |x: &Option<Primary>| x.as_ref().map(|x| std::mem::discriminant(&x.layout));
        if !primaries.iter().zip(&self.primaries).all(|(a, b)| layout(a) == layout(b)) {
            self.as_mut().begin_reset_model();
            self.as_mut().rust_mut().shown = elements;
            self.as_mut().rust_mut().primaries = primaries;
            self.as_mut().end_reset_model();
            return;
        }
        let primary_moved = [0, 1].map(|i| primaries[i] != self.primaries[i]);
        self.as_mut().rust_mut().primaries = primaries;
        let key = |p: &Placed| (p.id, std::mem::discriminant(&p.paint));
        if !(elements.len() == self.shown.len() && elements.iter().zip(&self.shown).all(|(a, b)| key(a) == key(b))) {
            let old: Vec<_> = self.shown.iter().map(key).collect();
            let new: Vec<_> = elements.iter().map(key).collect();
            let kept = common(&old, &new);
            let root = QModelIndex::default();
            for i in (0..old.len()).rev().filter(|i| !kept.iter().any(|k| k.0 == *i)) {
                self.as_mut().begin_remove_rows(&root, i as i32, i as i32);
                self.as_mut().rust_mut().shown.remove(i);
                self.as_mut().end_remove_rows();
            }
            for j in (0..new.len()).filter(|j| !kept.iter().any(|k| k.1 == *j)) {
                self.as_mut().begin_insert_rows(&root, j as i32, j as i32);
                self.as_mut().rust_mut().shown.insert(j, elements[j].clone());
                self.as_mut().end_insert_rows();
            }
        }
        let moved: Vec<usize> = (0..elements.len())
            .filter(|&j| self.shown[j] != elements[j] || matches!(elements[j].paint, Paint::Primary { leaving } if primary_moved[leaving as usize]))
            .collect();
        self.as_mut().rust_mut().shown = elements;
        for j in moved {
            let at = self.index(j as i32, 0, &QModelIndex::default());
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
        follow_scene!(self);
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

pub struct EsdeRowsModelRust {
    scene: *mut EsdeModel,
    revision: i32,
    leaving: bool,
    shared: Option<Shared>,
    rows: RowCache,
}

impl Default for EsdeRowsModelRust {
    fn default() -> Self {
        EsdeRowsModelRust { scene: std::ptr::null_mut(), revision: 0, leaving: false, shared: None, rows: RowCache::default() }
    }
}

impl cxx_qt::Initialize for qobject::EsdeRowsModel {
    fn initialize(mut self: Pin<&mut Self>) {
        follow_scene!(self);
        self.as_mut().on_leaving_changed(|m| m.refresh()).release();
    }
}

impl qobject::EsdeRowsModel {
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
        follow_scene!(self);
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

#[cfg(test)]
mod tests {
    use super::common;

    #[test]
    fn a_change_keeps_the_rows_both_lists_share_in_order() {
        assert_eq!(common(&[1, 2, 5], &[1, 2, 3, 5]), [(0, 0), (1, 1), (2, 3)], "a row inserted keeps the rest");
        assert_eq!(common(&[1, 2, 3, 5], &[1, 2, 5]), [(0, 0), (1, 1), (3, 2)], "a row removed keeps the rest");
        assert_eq!(common(&[1, 2, 5], &[1, 3, 5]), [(0, 0), (2, 2)], "a row replaced is one removed and one inserted");
        assert_eq!(common::<u8>(&[], &[1]), []);
        assert_eq!(common(&[3, 1, 2], &[1, 2, 3]), [(1, 0), (2, 1)]);
    }
}
