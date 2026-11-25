//! Settings -> Library -> Themes as a Settings page (board #61, R-20261116-24), backed by themes.rs.

use crate::help::Screen;
use crate::settings::{Outcome, Page, Pane, Row, SettingsView};
use crate::settings_shown::{BoardAside, BoardRow, Form};
use crate::themes::{self, Kind, Row as ThemeRow};
use emusen_kit_core::settings::RowKind;
use serde_json::{json, Value};

/// One item on the Themes Settings page: a theme row by index in `listing.rows` or an active ES-DE choice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ThemeItem {
    Theme(usize),
    Choice(usize),
}

impl SettingsView {
    /// Items of the currently selected theme kind (`EmuSen` or `ES-DE`), including active ES-DE choices.
    pub(crate) fn theme_items(&self) -> Vec<ThemeItem> {
        let Some(listing) = self.themes.as_ref() else { return Vec::new() };
        let mut items = Vec::new();
        for (at, row) in listing.rows.iter().enumerate().filter(|(_, r)| r.kind == self.theme_kind) {
            items.push(ThemeItem::Theme(at));
            if let Some(esde) = listing.esde.as_ref().filter(|e| row.kind == Kind::Esde && e.id == row.id) {
                items.extend((0..esde.choices.len()).map(ThemeItem::Choice));
            }
        }
        items
    }

    /// Theme row backing an item (either the theme itself or the active ES-DE theme for a choice).
    fn theme_for(&self, item: ThemeItem) -> Option<&ThemeRow> {
        let listing = self.themes.as_ref()?;
        match item {
            ThemeItem::Theme(at) => listing.rows.get(at),
            ThemeItem::Choice(_) => listing.esde.as_ref().and_then(|e| listing.rows.iter().find(|r| r.id == e.id)),
        }
    }

    pub(crate) fn themes_entries(&self, page: &Page) -> Option<Vec<Value>> {
        if *page != Page::Themes {
            return None;
        }
        let Some(listing) = self.themes.as_ref() else { return Some(Vec::new()) };
        Some(
            self.theme_items()
                .into_iter()
                .enumerate()
                .map(|(i, item)| match item {
                    ThemeItem::Theme(at) => json!({ "id": format!("theme:{}", listing.rows[at].id), "theme_item": i }),
                    ThemeItem::Choice(c) => {
                        let key = listing.esde.as_ref().and_then(|e| e.choices.get(c)).map_or("", |ch| ch.key);
                        json!({ "id": format!("themechoice:{key}"), "theme_item": i })
                    }
                })
                .collect(),
        )
    }

    pub(crate) fn themes_row(&self, entry: &Value) -> Option<Row> {
        let i = entry["theme_item"].as_u64()? as usize;
        let item = *self.theme_items().get(i)?;
        let listing = self.themes.as_ref()?;
        let id = entry["id"].as_str().unwrap_or_default().to_owned();
        match item {
            ThemeItem::Theme(at) => {
                let row = listing.rows.get(at)?;
                Some(Row { id, label: row.name.clone(), value: row.value(), kind: RowKind::Action, held: false, marked: row.current, code: false })
            }
            ThemeItem::Choice(c) => {
                let choice = listing.esde.as_ref()?.choices.get(c)?;
                Some(Row { id, label: choice.label.to_owned(), value: choice.value().to_owned(), kind: RowKind::Steps, held: false, marked: false, code: false })
            }
        }
    }

    pub(crate) fn themes_pane(&self, entry: &Value) -> Option<Pane> {
        let i = entry["theme_item"].as_u64()? as usize;
        let item = *self.theme_items().get(i)?;
        let row = self.theme_for(item)?;
        Some(Pane { sentence: row.findings(), muted: vec![row.line()], ..Pane::default() })
    }

    pub(crate) fn themes_help(&self, focused: usize) -> Option<Screen> {
        if self.page() != Some(&Page::Themes) {
            return None;
        }
        let item = self.theme_items().get(focused).copied();
        Some(match item {
            Some(ThemeItem::Choice(_)) => Screen::ThemesChoice,
            Some(i) if self.theme_for(i).is_some_and(ThemeRow::usable) => Screen::Themes,
            _ => Screen::ThemesChecked,
        })
    }

    pub(crate) fn themes_step(&mut self, entry: &Value, by: i32) -> Option<Outcome> {
        let i = entry["theme_item"].as_u64()? as usize;
        match self.theme_items().get(i).copied()? {
            ThemeItem::Choice(c) => Some(Outcome::ThemeStepChoice { choice: c, by }),
            ThemeItem::Theme(_) => Some(Outcome::Nothing),
        }
    }

    pub(crate) fn themes_choose(&mut self, entry: &Value) -> Option<Outcome> {
        let i = entry["theme_item"].as_u64()? as usize;
        match self.theme_items().get(i).copied()? {
            ThemeItem::Choice(c) => Some(Outcome::ThemeStepChoice { choice: c, by: 1 }),
            ThemeItem::Theme(at) => {
                let row = self.themes.as_ref()?.rows.get(at)?;
                Some(if row.usable() { Outcome::UseTheme(row.id.clone()) } else { Outcome::Nothing })
            }
        }
    }

    /// X on Themes checks the focused theme package off the GUI thread.
    pub fn check_focused_theme(&self, focused: usize) -> Outcome {
        if self.page() != Some(&Page::Themes) {
            return Outcome::Nothing;
        }
        let Some(item) = self.theme_items().get(focused).copied() else { return Outcome::Nothing };
        let Some(row) = self.theme_for(item) else { return Outcome::Nothing };
        Outcome::ThemeCheck(row.id.clone())
    }

    /// LB/RB on Themes switches between EmuSen and ES-DE kinds.
    pub(crate) fn themes_shoulder(&mut self, by: i32) -> bool {
        if self.page() != Some(&Page::Themes) {
            return false;
        }
        let next = crate::settings::wrapped(self.theme_kind.index(), by, themes::KINDS.len());
        self.theme_kind = if next == 0 { Kind::EmuSen } else { Kind::Esde };
        true
    }

    /// Pointer click on an inline kind pill selects that theme kind directly.
    pub fn choose_inline_pill(&mut self, index: i32) -> bool {
        if self.page() != Some(&Page::Themes) {
            return false;
        }
        let next = match index {
            0 => Kind::EmuSen,
            1 => Kind::Esde,
            _ => return false,
        };
        if self.theme_kind == next {
            return false;
        }
        self.theme_kind = next;
        true
    }

    pub(crate) fn themes_board_row(&self, row: &Row, entry: &Value) -> Option<BoardRow> {
        let i = entry["theme_item"].as_u64()? as usize;
        let item = *self.theme_items().get(i)?;
        let form = match item {
            ThemeItem::Choice(_) => Form::Choice,
            ThemeItem::Theme(_) => Form::Fact,
        };
        Some(BoardRow {
            label: row.label.clone(),
            about: String::new(),
            value: row.value.clone(),
            form,
            planet: -3,
            subtitle: String::new(),
            chip: String::new(),
            chip_attention: false,
            runs: true,
        })
    }

    pub(crate) fn themes_board_aside(&self, focused: usize, out: &mut BoardAside) {
        out.trail_line = "Library".to_owned();
        out.heading = "Themes".to_owned();
        out.inline_pills = themes::KINDS.iter().map(|k| (*k).to_owned()).collect();
        out.inline_pill = self.theme_kind.index() as i32;
        out.page_glow = "#1FA890".to_owned();
        out.theme_preview = true;
        out.wrap_about = true;
        out.cards_help = false;
        out.box_menu = true;
        out.lines.clear();
        out.muted.clear();
        if let Some(row) = self.theme_items().get(focused).copied().and_then(|item| self.theme_for(item)) {
            out.preview_image = row.preview.clone();
            out.about_heading = row.name.clone();
            out.meta = row.line();
            out.sentence = row.findings();
        } else {
            out.preview_image.clear();
            out.about_heading.clear();
            out.meta.clear();
            out.sentence.clear();
        }
    }
}
