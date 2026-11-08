//! A card's rows as QML reads them, once for every card model (`CardModel`, `PauseModel`): the roles `MenuCard.qml`'s
//! rows name, and a row's value for each (EmuSen_Qt.md §7.3).

use cxx_qt_lib::{QByteArray, QHash, QHashPair_i32_QByteArray, QString, QVariant};

/// One row as a card shows it.
pub struct Shown<'a> {
    pub key: &'a str,
    pub label: &'a str,
    pub value: &'a str,
    pub steps: bool,
    pub opens: bool,
    pub section_end: bool,
    /// Greyed: a hint, chosen to no effect.
    pub dimmed: bool,
}

const LABEL: i32 = 0;
const VALUE: i32 = 1;
const STEPS: i32 = 2;
const OPENS: i32 = 3;
const SECTION_END: i32 = 4;
const KEY: i32 = 5;
const DIMMED: i32 = 6;

/// A row's value for a role; none for a row past the end or a role no card names.
pub fn data(row: Option<Shown<'_>>, role: i32) -> QVariant {
    let Some(row) = row else { return QVariant::default() };
    match role {
        LABEL => QVariant::from(&QString::from(row.label)),
        VALUE => QVariant::from(&QString::from(row.value)),
        STEPS => QVariant::from(&row.steps),
        OPENS => QVariant::from(&row.opens),
        SECTION_END => QVariant::from(&row.section_end),
        KEY => QVariant::from(&QString::from(row.key)),
        DIMMED => QVariant::from(&row.dimmed),
        _ => QVariant::default(),
    }
}

/// The roles by name.
pub fn role_names() -> QHash<QHashPair_i32_QByteArray> {
    let mut roles = QHash::<QHashPair_i32_QByteArray>::default();
    let names = [(LABEL, "label"), (VALUE, "value"), (STEPS, "steps"), (OPENS, "opens"), (SECTION_END, "sectionEnd"), (KEY, "key"), (DIMMED, "dimmed")];
    for (role, name) in names {
        roles.insert(role, QByteArray::from(name));
    }
    roles
}
