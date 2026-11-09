//! A console's memory cards as rows, the one description Settings ▸ Consoles ▸ <console> ▸ Memory cards and the Pause
//! Card's Game Options ▸ Memory Cards share (EmuSen_Interface.md §18.5, §18.10; EmuSen_Qt.md §7.7.E). The media, their
//! mode, slots and drawer are the card manager's (`cards_op`'s list, EmuSen_SaveMedia.md §6); the mode and the slots are
//! the schema's rows keyed as the list names them, so nothing here knows a console.

use crate::schema;
use emusen_crystal_client::shapes::SettingRow;
use serde::Deserialize;
use serde_json::Value;

/// One medium of a console as the card manager lists it.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct Medium {
    pub medium: String,
    pub title: String,
    pub noun: String,
    pub extension: String,
    pub kind: String,
    pub mode: String,
    pub mode_key: String,
    pub slots: Vec<Slot>,
    pub cards: Vec<Card>,
    pub next_name: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct Slot {
    pub label: String,
    pub name: String,
    pub path: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct Card {
    pub path: String,
    pub name: String,
    pub label: String,
    pub games_own: bool,
    pub held_by: Option<String>,
}

impl Medium {
    /// The medium as the card manager lists it, for a model that hands it on.
    pub fn to_json(&self) -> Value {
        let slots: Vec<Value> = self.slots.iter().map(|s| serde_json::json!({ "label": s.label, "name": s.name, "path": s.path })).collect();
        let cards: Vec<Value> = self
            .cards
            .iter()
            .map(|c| serde_json::json!({ "path": c.path, "name": c.name, "label": c.label, "games_own": c.games_own, "held_by": c.held_by }))
            .collect();
        serde_json::json!({ "medium": self.medium, "title": self.title, "noun": self.noun, "extension": self.extension, "kind": self.kind,
                            "mode": self.mode, "mode_key": self.mode_key, "slots": slots, "cards": cards, "next_name": self.next_name })
    }

    /// The cards in the drawer: the medium's own files, not a game's.
    pub fn drawer(&self) -> Vec<&Card> {
        self.cards.iter().filter(|c| !c.games_own).collect()
    }

    /// The noun inside a sentence ("card"; an acronym kept).
    pub fn noun_in_text(&self) -> String {
        in_text(&self.noun)
    }

    /// "2 cards": the drawer counted in the medium's noun.
    pub fn counted(&self) -> String {
        let noun = self.noun_in_text();
        crate::words::counted(self.drawer().len(), &noun, &format!("{noun}s"))
    }

    /// The file a new card is made at: the drawer's folder, the first name no card has, the medium's extension.
    pub fn new_card_path(&self) -> Option<String> {
        let folder = self.slots.iter().find_map(|s| std::path::Path::new(&s.path).parent())?;
        Some(folder.join(format!("{}{}", self.next_name, self.extension)).to_string_lossy().into_owned())
    }
}

/// Words inside a sentence, each lowercased but an acronym, as the card manager's own words write a noun.
fn in_text(words: &str) -> String {
    words
        .split(' ')
        .map(|w| if w.chars().count() > 1 && w.chars().all(|c| !c.is_alphabetic() || c.is_uppercase()) { w.to_owned() } else { w.to_lowercase() })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The card manager's list, read; an answer it did not give is no media.
pub fn media(list: &Value) -> Vec<Medium> {
    serde_json::from_value(list.clone()).unwrap_or_default()
}

/// What a card on a drawer's page is, as Mistress's page says it.
pub const DRAWER_CARD: &str = "A card in the drawer.";

/// What a row of the page is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Part {
    /// The medium's mode, its schema row by the list's `mode_key`.
    Mode,
    /// A slot's card, its schema row the one shown with the mode's Named.
    Slot(usize),
    /// The drawer's cards, a page of its own.
    Drawer,
    /// The saves on the cards, a door to the card manager's window.
    Saves,
    /// A new card made in the drawer.
    New,
}

/// A row of the page as both screens draw it.
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    pub id: String,
    pub medium: String,
    pub part: Part,
    pub label: String,
    pub value: String,
    pub steps: bool,
    pub opens: bool,
    pub asks: bool,
    /// A slot whose card the mode places, which the slot's row does not choose.
    pub held: bool,
    /// The schema's key the row steps, where it steps one.
    pub key: Option<String>,
}

/// The rows' schema: the mode's row and each slot's, by the medium's keys.
pub struct Keys<'a> {
    pub mode: Option<&'a SettingRow>,
    pub slots: Vec<&'a SettingRow>,
}

/// The schema's rows of a medium: the mode by its key, then the rows shown with the mode, in order, as its slots.
pub fn keys<'a>(medium: &Medium, schema: &'a [SettingRow]) -> Keys<'a> {
    Keys {
        mode: schema.iter().find(|r| r.key == medium.mode_key),
        slots: schema.iter().filter(|r| r.shown_with.as_ref().is_some_and(|w| w.key == medium.mode_key)).collect(),
    }
}

/// The page's rows: Settings' as the mode, the slots, the drawer, the saves and a new card (§18.10); in a game the slots
/// first, then the mode and the drawer (Mistress's `GameCardRows`). `value` reads a key as the console holds it.
pub fn rows(media: &[Medium], schema: &[SettingRow], value: &dyn Fn(&str) -> String, game: bool) -> Vec<Row> {
    let mut out = Vec::new();
    for m in media {
        let keys = keys(m, schema);
        let row = |part: Part, id: String, label: String| Row { id, medium: m.medium.clone(), part, label, value: String::new(), steps: false, opens: false, asks: false, held: false, key: None };
        let mode = keys.mode.map(|r| Row {
            value: schema::shown(r, &value(&r.key)),
            steps: true,
            key: Some(r.key.clone()),
            ..row(Part::Mode, format!("mode:{}", m.medium), "Mode".to_owned())
        });
        let slots: Vec<Row> = m
            .slots
            .iter()
            .enumerate()
            .map(|(at, slot)| {
                let key = keys.slots.get(at);
                let placed = key.is_some_and(|r| schema::applies(r, value));
                Row {
                    value: slot.name.clone(),
                    steps: placed,
                    held: !placed,
                    key: key.map(|r| r.key.clone()),
                    ..row(Part::Slot(at), format!("slot:{}:{at}", m.medium), slot.label.clone())
                }
            })
            .collect();
        let drawer = Row { value: m.counted(), opens: true, ..row(Part::Drawer, format!("cards:{}", m.medium), "Cards".to_owned()) };
        if game {
            out.extend(slots);
            out.extend(mode);
            out.push(drawer);
        } else {
            out.extend(mode);
            out.extend(slots);
            out.push(drawer);
            out.push(Row { opens: true, ..row(Part::Saves, format!("saves:{}", m.medium), "Saves on the cards".to_owned()) });
            out.push(Row { value: "…".to_owned(), asks: true, ..row(Part::New, format!("new:{}", m.medium), format!("New {}", m.noun_in_text())) });
        }
    }
    out
}

/// The pane of a row: the mode's words, a slot's card, or what the row does.
pub fn pane(media: &[Medium], row: &Row, schema: &[SettingRow], value: &dyn Fn(&str) -> String, console: &str, game: bool) -> crate::firmware::Words {
    let Some(m) = media.iter().find(|m| m.medium == row.medium) else { return Default::default() };
    let said = |line: String, muted: Option<String>| crate::firmware::Words { lines: vec![line], muted: muted.into_iter().collect() };
    match row.part {
        Part::Mode => mode_pane(m, schema, value, console),
        Part::Slot(_) if game => said(format!("{} is in {}.", row.value, row.label), Some("Left and Right choose another card; it goes in when the game next starts.".to_owned())),
        Part::Slot(at) => said(keys(m, schema).slots.get(at).and_then(|r| r.hint.clone()).unwrap_or_else(|| "The card in this slot when a game starts.".to_owned()), None),
        Part::Drawer => said(format!("Every {} in the drawer, and the saves on each.", m.noun_in_text()), None),
        Part::Saves => said(format!("Each {}'s saves, copied, moved or deleted.", m.noun_in_text()), None),
        Part::New => said(format!("A blank {} in the drawer, named for you.", m.noun_in_text()), None),
    }
}

/// The mode's words: what Shared means, and the drawer counted (Mistress's `CardsPane`).
pub fn mode_pane(m: &Medium, schema: &[SettingRow], value: &dyn Fn(&str) -> String, console: &str) -> crate::firmware::Words {
    let mode = keys(m, schema).mode.map_or_else(|| m.mode.clone(), |r| schema::shown(r, &value(&r.key)));
    let noun = m.noun_in_text();
    crate::firmware::Words {
        lines: vec![format!("{mode}: every {console} game sees the same {noun}s when they are shared, as the console did; per game gives each game its own.")],
        muted: vec![format!("{} in the drawer", m.counted())],
    }
}

/// The drawer's page: a row a card, and which game holds it now.
pub fn drawer_rows(m: &Medium) -> Vec<Row> {
    m.drawer()
        .into_iter()
        .map(|c| Row {
            id: format!("card:{}", c.path),
            medium: m.medium.clone(),
            part: Part::Drawer,
            label: c.label.clone(),
            value: c.held_by.as_ref().map(|h| format!("in {h}")).unwrap_or_default(),
            steps: false,
            opens: false,
            asks: false,
            held: false,
            key: None,
        })
        .collect()
}

/// Left or Right on a row that steps: its key and the value one step on.
pub fn step(row: &Row, schema: &[SettingRow], value: &dyn Fn(&str) -> String, by: i32) -> Option<(String, String)> {
    if !row.steps {
        return None;
    }
    let key = row.key.as_ref()?;
    let r = schema.iter().find(|r| &r.key == key)?;
    schema::stepped(r, &value(key), by).map(|v| (key.clone(), v))
}
