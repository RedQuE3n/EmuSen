//! `PaneModel`, a card's context pane as a QAbstractListModel (EmuSen_Qt.md §7.3): the pane a page model describes as
//! plain data (pause.rs's `Pane`), its tiles as rows and its large picture and sentence as properties, so the pane's
//! view writes no words and no logic.

use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QByteArray, QHash, QHashPair_i32_QByteArray, QModelIndex, QString, QVariant};
use serde_json::Value;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!(<QtCore/QAbstractListModel>);
        type QAbstractListModel = crate::library_model::qobject::QAbstractListModel;
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
    }

    extern "RustQt" {
        #[qobject]
        #[base = QAbstractListModel]
        #[qml_element]
        /// The pane as plain data, a page model's `pane`.
        #[qproperty(QString, pane)]
        /// What it shows: none, reel, moment or slots.
        #[qproperty(QString, kind, READ, NOTIFY)]
        /// Whether there is anything to show; an empty pane hides with its surface.
        #[qproperty(bool, shown, READ, NOTIFY)]
        /// The reel's page: a large picture over a strip of small ones.
        #[qproperty(bool, moment, READ, NOTIFY)]
        /// The slots: a tile without a picture is drawn as an empty slot.
        #[qproperty(bool, slots, READ, NOTIFY)]
        /// The large picture's address, empty for none.
        #[qproperty(QString, big, READ, NOTIFY)]
        #[qproperty(QString, sentence, READ, NOTIFY)]
        /// What a tile without a picture says inside its outline.
        #[qproperty(QString, blank, READ, NOTIFY)]
        /// A disc's cover beside its name (`title`) and the sentence; a cover-less disc draws the generated card.
        #[qproperty(bool, cover, READ, NOTIFY)]
        #[qproperty(QString, title, READ, NOTIFY)]
        type PaneModel = super::PaneModelRust;
    }

    extern "RustQt" {
        #[qinvokable]
        #[cxx_override]
        fn data(self: &PaneModel, index: &QModelIndex, role: i32) -> QVariant;

        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &PaneModel) -> QHash_i32_QByteArray;

        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &PaneModel, parent: &QModelIndex) -> i32;
    }

    unsafe extern "RustQt" {
        #[inherit]
        #[cxx_name = "beginResetModel"]
        fn begin_reset_model(self: Pin<&mut PaneModel>);

        #[inherit]
        #[cxx_name = "endResetModel"]
        fn end_reset_model(self: Pin<&mut PaneModel>);
    }

    impl cxx_qt::Initialize for PaneModel {}
}

#[derive(Default)]
pub struct PaneModelRust {
    pane: QString,
    kind: QString,
    shown: bool,
    moment: bool,
    slots: bool,
    big: QString,
    sentence: QString,
    blank: QString,
    cover: bool,
    title: QString,
    tiles: Vec<[String; 3]>,
    ringed: Vec<bool>,
}

impl cxx_qt::Initialize for qobject::PaneModel {
    fn initialize(self: Pin<&mut Self>) {
        self.on_pane_changed(|model| model.read()).release();
    }
}

const PICTURE: i32 = 0;
const NAME: i32 = 1;
const WHEN: i32 = 2;
const RINGED: i32 = 3;

impl qobject::PaneModel {
    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let at = index.row() as usize;
        let (Some(tile), Some(ringed)) = (self.tiles.get(at), self.ringed.get(at)) else { return QVariant::default() };
        match role {
            PICTURE | NAME | WHEN => QVariant::from(&QString::from(tile[role as usize].as_str())),
            RINGED => QVariant::from(ringed),
            _ => QVariant::default(),
        }
    }

    pub fn role_names(&self) -> QHash<QHashPair_i32_QByteArray> {
        let mut roles = QHash::<QHashPair_i32_QByteArray>::default();
        for (role, name) in [(PICTURE, "picture"), (NAME, "name"), (WHEN, "when"), (RINGED, "ringed")] {
            roles.insert(role, QByteArray::from(name));
        }
        roles
    }

    pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
        self.tiles.len() as i32
    }

    /// The pane's plain data read into rows and properties.
    fn read(mut self: Pin<&mut Self>) {
        let v: Value = serde_json::from_str(&self.pane.to_string()).unwrap_or(Value::Null);
        let words = |v: &Value| v.as_str().unwrap_or_default().to_string();
        let list = v["tiles"].as_array().cloned().unwrap_or_default();
        let tiles: Vec<[String; 3]> = list.iter().map(|t| [words(&t["picture"]), words(&t["name"]), words(&t["when"])]).collect();
        let ringed: Vec<bool> = list.iter().map(|t| t["ringed"].as_bool().unwrap_or(false)).collect();
        if tiles != self.tiles || ringed != self.ringed {
            self.as_mut().begin_reset_model();
            self.as_mut().rust_mut().tiles = tiles;
            self.as_mut().rust_mut().ringed = ringed;
            self.as_mut().end_reset_model();
        }
        let (kind, big, sentence) = (QString::from(&words(&v["kind"])), QString::from(&words(&v["big"])), QString::from(&words(&v["sentence"])));
        let (shown, moment) = (!kind.is_empty() && kind != QString::from("none"), kind == QString::from("moment"));
        let slots = kind == QString::from("slots");
        let cover = kind == QString::from("cover");
        if self.slots != slots {
            self.as_mut().rust_mut().slots = slots;
            self.as_mut().slots_changed();
        }
        if self.kind != kind {
            self.as_mut().rust_mut().kind = kind;
            self.as_mut().kind_changed();
        }
        if self.shown != shown {
            self.as_mut().rust_mut().shown = shown;
            self.as_mut().shown_changed();
        }
        if self.moment != moment {
            self.as_mut().rust_mut().moment = moment;
            self.as_mut().moment_changed();
        }
        if self.big != big {
            self.as_mut().rust_mut().big = big;
            self.as_mut().big_changed();
        }
        if self.sentence != sentence {
            self.as_mut().rust_mut().sentence = sentence;
            self.as_mut().sentence_changed();
        }
        if self.cover != cover {
            self.as_mut().rust_mut().cover = cover;
            self.as_mut().cover_changed();
        }
        let title = QString::from(&words(&v["title"]));
        if self.title != title {
            self.as_mut().rust_mut().title = title;
            self.as_mut().title_changed();
        }
        let blank = QString::from(&words(&v["blank"]));
        if self.blank != blank {
            self.as_mut().rust_mut().blank = blank;
            self.blank_changed();
        }
    }
}
