//! The Pause Card's Game Options ▸ Display and Sound (EmuSen_Qt.md §7.7.H): Settings' own rows for the running game,
//! from the one description. Display is the running console's page (Settings ▸ Consoles ▸ <console>), Sound is the
//! Sound section, each down to the rows the description marks `pause`; a door among them (Shader levers) opens its page
//! under the card's. The card's pages are Settings' pages kept apart from the screen's, so a change is stored, and
//! reaches the game, exactly as Settings stores it.

use crate::settings::{Outcome, Page, SettingsView};
use crate::settings_controllers::GameRow;
use emusen_kit_core::settings::RowKind;
use serde_json::{json, Value};

/// Which of the card's pages Settings' rows are drawn on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardPage {
    Display,
    Sound,
}

impl CardPage {
    pub fn key(self) -> &'static str {
        match self {
            CardPage::Display => "display",
            CardPage::Sound => "sound",
        }
    }

    pub fn of(key: &str) -> Option<CardPage> {
        match key {
            "display" => Some(CardPage::Display),
            "sound" => Some(CardPage::Sound),
            _ => None,
        }
    }
}

/// The card's own place in Settings: the section and pages its rows are read under, the depth its page starts at, and
/// the row a page closed hands the focus back to.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CardPlace {
    page: Option<CardPage>,
    section: usize,
    pages: Vec<(Page, String, usize)>,
    depth: usize,
    focus: usize,
}

/// Settings' rows as the card draws them: the page's own title where a door opened one under it, and the row to focus
/// as such a page opens or closes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CardRows {
    pub page: String,
    /// The title of a page opened under the card's, empty on the card's page itself.
    pub title: String,
    pub focus: usize,
    pub rows: Vec<GameRow>,
}

impl CardRows {
    pub fn to_json(&self) -> Value {
        json!({ "page": self.page, "title": self.title, "focus": self.focus, "rows": self.rows.iter().map(GameRow::to_json).collect::<Vec<_>>() })
    }

    pub fn from_json(text: &str) -> CardRows {
        let v: Value = serde_json::from_str(text).unwrap_or_default();
        CardRows {
            page: v["page"].as_str().unwrap_or_default().to_owned(),
            title: v["title"].as_str().unwrap_or_default().to_owned(),
            focus: v["focus"].as_u64().unwrap_or(0) as usize,
            rows: GameRow::list(&v["rows"]),
        }
    }
}

impl SettingsView {
    /// The card's page opened on Settings' rows for it: Display on `console`'s page, Sound on the Sound section.
    pub fn card_open(&mut self, page: CardPage, console: &str) {
        let section = match page {
            CardPage::Display => self.section_of("consoles"),
            CardPage::Sound => self.section_of("sound"),
        };
        let pages = match page {
            CardPage::Display => vec![(Page::Console(console.to_owned()), String::new(), 0)],
            CardPage::Sound => Vec::new(),
        };
        let depth = pages.len();
        self.card = CardPlace { page: Some(page), section, pages, depth, focus: 0 };
    }

    fn section_of(&self, id: &str) -> usize {
        self.description_at(&["sections"]).as_array().and_then(|s| s.iter().position(|s| s["id"] == id)).unwrap_or(0)
    }

    /// `f` run with the card's section and pages in place of the screen's, which come back after.
    fn in_card<T>(&mut self, f: impl FnOnce(&mut SettingsView) -> T) -> T {
        let section = std::mem::replace(&mut self.section, self.card.section);
        let pages = std::mem::replace(&mut self.pages, std::mem::take(&mut self.card.pages));
        let out = f(self);
        self.card.pages = std::mem::replace(&mut self.pages, pages);
        self.section = section;
        out
    }

    /// The indices among the page's rows that the card shows: on its own page those marked `pause`, under a door all.
    fn card_shown(&self) -> Vec<usize> {
        let entries = self.entries();
        let own = self.pages.len() == self.card.depth;
        (0..entries.len()).filter(|&i| !own || entries[i]["pause"] == true).collect()
    }

    /// The card's rows, each with its pane's words, as `SettingsModel.cardSettings` gives them.
    pub fn card_rows(&mut self) -> CardRows {
        let Some(page) = self.card.page else { return CardRows::default() };
        let focus = self.card.focus;
        self.in_card(|view| {
            let rows = view.rows();
            let rows = view
                .card_shown()
                .into_iter()
                .filter_map(|i| {
                    let row = rows.get(i)?;
                    let pane = view.pane(i);
                    let sentence = std::iter::once(pane.sentence).chain(pane.lines).filter(|l| !l.is_empty()).collect::<Vec<_>>().join(" ");
                    Some(GameRow {
                        key: row.id.clone(),
                        label: row.label.clone(),
                        value: row.value.clone(),
                        steps: matches!(row.kind, RowKind::Steps | RowKind::Slider) && !row.held,
                        opens: row.kind == RowKind::Opens,
                        dimmed: row.held || row.kind == RowKind::Action,
                        sentence,
                        muted: pane.muted.join(" "),
                    })
                })
                .collect();
            let title = if view.pages.len() > view.card.depth { view.pages.last().map(|(_, l, _)| l.clone()).unwrap_or_default() } else { String::new() };
            CardRows { page: page.key().to_owned(), title, focus, rows }
        })
    }

    /// Left or Right (`by` ±1), or A (`by` 0), on the card's row at `index`, as Settings does it on that row.
    pub fn card_settings_act(&mut self, index: usize, by: i32) -> Outcome {
        self.in_card(|view| {
            let Some(&at) = view.card_shown().get(index) else { return Outcome::Nothing };
            let held = view.rows().get(at).is_some_and(|r| r.held);
            match by {
                _ if held => Outcome::Nothing,
                0 => {
                    let outcome = view.choose(at);
                    if outcome == Outcome::Opened {
                        view.card.focus = 0;
                    }
                    outcome
                }
                by => view.step(at, by),
            }
        })
    }

    /// B on the card's page: a page a door opened closed, with the focus on that door, and true; false on its own page.
    pub fn card_back(&mut self) -> bool {
        self.in_card(|view| {
            if view.pages.len() <= view.card.depth {
                return false;
            }
            let Some((_, _, door)) = view.pages.pop() else { return false };
            view.card.focus = view.card_shown().iter().position(|&i| i == door).unwrap_or(0);
            true
        })
    }
}
