//! The Card form's pages as plain data, shared by every card (EmuSen_Interface.md §2.3, §2.8, §18.1; EmuSen_Qt.md §7.1,
//! stage 2): a page's title, the way back above it, its rows with their marks and each row's pane, a question as a page
//! of No and Yes, and the stack of pages a card has turned through. A card's own pages are a `Pages` (home_card.rs,
//! list_card.rs); `CardModel` puts any of them on Qt and `MenuCard.qml` draws them. No Qt here.

use serde_json::Value;

/// What the pane beside the rows shows for a row: lines (each a line and a muted line under it), or a picture with its
/// caption, or nothing. A card whose pane holds more (the rewind reel, the slots) draws its own in `MenuCard`'s slot.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Pane {
    pub lines: Vec<(String, String)>,
    /// A picture's URL, as a model supplies it; empty for none.
    pub picture: String,
    pub caption: String,
}

impl Pane {
    pub fn said(line: &str, muted: &str) -> Pane {
        Pane { lines: vec![(line.to_owned(), muted.to_owned())], ..Pane::default() }
    }
}

/// One row: its key (what choosing it does, `Pages::choose`), its words, how it is drawn, and its pane.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Row {
    pub key: String,
    pub label: String,
    pub value: String,
    /// Left and Right step its value: drawn ◂ value ▸.
    pub steps: bool,
    /// A choice of it turns to a page of its own: drawn ▸.
    pub opens: bool,
    /// The last row of a section, its rule drawn lighter.
    pub section_end: bool,
    pub pane: Pane,
}

impl Row {
    pub fn new(key: &str, label: &str) -> Row {
        Row { key: key.to_owned(), label: label.to_owned(), ..Row::default() }
    }

    pub fn value(self, value: &str) -> Row {
        Row { value: value.to_owned(), ..self }
    }

    pub fn opens(self) -> Row {
        Row { opens: true, ..self }
    }

    pub fn steps(self) -> Row {
        Row { steps: true, ..self }
    }

    pub fn pane(self, pane: Pane) -> Row {
        Row { pane, ..self }
    }
}

/// A page: its title, the words above it (the way back on a page past the first, a question's sentence), its rows.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Page {
    pub title: String,
    pub breadcrumb: String,
    pub rows: Vec<Row>,
    /// A line under the rows (the Home card's Power page: the program's version); empty for none.
    pub footer: String,
}

pub const NO: &str = "no";
pub const YES: &str = "yes";

/// A question as a page, as Mistress's `Ask` turns a row into one: its title the question, its sentence above it, No
/// then Yes, so the focus starts on the answer that changes nothing.
pub fn question(title: &str, sentence: &str) -> Page {
    Page { title: title.to_owned(), breadcrumb: sentence.to_owned(), rows: vec![Row::new(NO, "No"), Row::new(YES, "Yes")], footer: String::new() }
}

/// The section a row ends: the row before the one that starts a new section has its rule drawn lighter.
pub fn end_section_before(rows: &mut [Row], index: usize) {
    if let Some(row) = index.checked_sub(1).and_then(|i| rows.get_mut(i)) {
        row.section_end = true;
    }
}

/// What a choice does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Choice {
    /// The card turns to the page by its key, focused on its first row.
    Turn(String),
    /// The card goes back a page, as B does (a question's No).
    Back,
    /// The card closes and the host carries out the action by its key.
    Act(String),
    /// The host carries out the action by its key and the card stays, its words read again (a step of a value).
    Stay(String),
    /// A row whose action is a later stage's: nothing happens.
    Owed,
}

/// A card's pages: the first page's key, each page made from the card's facts (plain data, `CardModel.facts`), and
/// what choosing or stepping a row does.
pub trait Pages {
    fn first(&self) -> &'static str;
    fn page(&self, key: &str, facts: &Value) -> Page;
    fn choose(&self, page: &str, row: &str) -> Choice;
    /// Left (-1) or Right (+1) on a row: the action the host carries out, if the row steps.
    fn step(&self, page: &str, row: &str, by: i32) -> Option<String> {
        let _ = (page, row, by);
        None
    }
}

/// The pages a card has turned through, each with the row it was on: B goes back to the row that opened a page.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Deck {
    stack: Vec<(String, usize)>,
}

impl Deck {
    /// The card opened at `page`, focused on its first row.
    pub fn open(&mut self, page: &str) {
        self.stack = vec![(page.to_owned(), 0)];
    }

    /// A page turned to from row `from` of the page shown.
    pub fn turn(&mut self, page: &str, from: usize) {
        if let Some(top) = self.stack.last_mut() {
            top.1 = from;
        }
        self.stack.push((page.to_owned(), 0));
    }

    /// B: the page before, on the row that opened the page left; none on the first page, where the card closes.
    pub fn back(&mut self) -> Option<(String, usize)> {
        if self.stack.len() < 2 {
            return None;
        }
        self.stack.pop();
        self.stack.last().cloned()
    }

    /// The page shown, by its key; empty before the card opens.
    pub fn page(&self) -> &str {
        self.stack.last().map_or("", |(key, _)| key.as_str())
    }

    /// How many pages deep the card is: 1 on the page it opened at.
    pub fn depth(&self) -> usize {
        self.stack.len()
    }
}

/// The card's pages by its name, as `CardModel.card` names it.
pub fn pages_of(card: &str) -> Option<Box<dyn Pages>> {
    match card {
        crate::home_card::CARD => Some(Box::new(crate::home_card::HomeCard)),
        crate::list_card::CARD => Some(Box::new(crate::list_card::ListCard)),
        _ => None,
    }
}
