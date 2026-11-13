//! The tool screen as plain data (EmuSen_Interface.md §15.2–15.6, §16; EmuSen_Qt.md §1.2.2): what `ToolModel` puts on
//! the kit's `ToolScreen` for any tool, the `Source` each tool implements, and `Tool`, which holds what every tool's
//! screen holds the same way (the section, the filter, the sort, the search, the focus, the marks, the open card and
//! the question). Cores is one source; `jobs` another, the tools that are lists of the platform library's jobs; `scraper`
//! the third, the Scraper's runs; `cheats` the cheat database and the library's games it has files for.
//! No Qt here.

pub mod cheats;
pub mod cores;
pub mod jobs;
pub mod scraper;

use crate::card::Row as CardRow;
use crate::question::{confirm, Question};
use emusen_kit_core::input::UiButton;

/// The colour words take by what they say: the kit's `Tones`, in its order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tone {
    #[default]
    Text,
    Muted,
    Strong,
    Accent,
    Success,
    Attention,
    Error,
}

/// Words and their tone: a header's fact, a pane's sentence, a cell; a cell of a column that gives way has short words
/// too, shown where its whole ones do not fit (EmuSen_Kit.md §7.106).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Words {
    pub words: String,
    pub tone: Tone,
    pub short: Option<String>,
}

impl Words {
    pub fn new(words: impl Into<String>, tone: Tone) -> Words {
        Words { words: words.into(), tone, short: None }
    }

    /// A cell's whole words and the short words that stand for them where they do not fit.
    pub fn giving_way(words: impl Into<String>, short: impl Into<String>) -> Words {
        Words { short: Some(short.into()), ..Words::text(words) }
    }

    pub fn text(words: impl Into<String>) -> Words {
        Words::new(words, Tone::Text)
    }
}

/// A column of the list: its title as a reader hears it, a fill's weight (0 for its words' width), its least width in
/// design units, whether its words stand at its right, and whether it gives way first, its cells' short words shown
/// where their whole ones do not fit (EmuSen_Kit.md §7.106).
#[derive(Clone, Debug, PartialEq)]
pub struct Column {
    pub title: &'static str,
    pub fill: f64,
    pub least: f64,
    pub trailing: bool,
    pub gives_way: bool,
}

/// One row of a source: its key across refreshes, a cell for each column, the sections and filters it is listed under
/// besides All, the words a search finds it by, and its value for each sort after the source's own order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Item {
    pub key: String,
    pub cells: Vec<Words>,
    pub sections: Vec<String>,
    pub filters: Vec<String>,
    pub search: String,
    pub sorts: Vec<String>,
}

/// A mark a row can be given, by its key and the word the plan says it with (`install`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mark {
    pub key: String,
    pub label: String,
}

/// The apply bar: its line and its tone, the share done while a job runs (none when nothing runs), and the space meter.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Bar {
    pub line: Words,
    pub progress: Option<f64>,
    pub meter: f64,
    pub meter_words: String,
}

/// The pane beside the list: the focused row's title, its caption, and its sentences.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Pane {
    pub title: String,
    pub caption: String,
    pub sentences: Vec<Words>,
}

/// A row's one action on A, named in the help bar (`Install…`), by its key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Action {
    pub key: String,
    pub label: String,
}

impl Action {
    pub fn new(key: &str, label: &str) -> Action {
        Action { key: key.to_owned(), label: label.to_owned() }
    }
}

/// What a source's action asks of the screen.
#[derive(Clone, Debug, PartialEq)]
pub enum Effect {
    /// Nothing more than the source's own change, which the next view shows.
    Done,
    /// A question, answered to the source by the key it gave.
    Ask(Question, String),
    /// The row marked with the mark of this key; with `true` the plan is asked at once.
    Mark(String, bool),
    /// The row's mark taken away.
    Unmark,
    /// A page of rows turned to in the open card, under a title.
    Page(String, Vec<CardRow>),
    /// A destination of the host's by its key (a console's settings, the firmware page), which this screen does not hold.
    Host(String),
}

/// One tool's content: its rows, words and actions. Everything a source returns is plain data; `Tool` asks for it again
/// after each change, so a source holds only its own facts.
pub trait Source {
    /// The header's title and the word the shoulders step by (`Console`).
    fn title(&self) -> &str;
    fn section_word(&self) -> &str;
    fn columns(&self) -> Vec<Column>;
    /// Every row in the source's own order.
    fn items(&self) -> Vec<Item>;
    /// The strip's sections, All first, by key and label; All alone for a tool with none of its own.
    fn sections(&self) -> Vec<(String, String)> {
        vec![("all".to_owned(), "All".to_owned())]
    }
    /// The List card's filters and sorts, by key and label, All and the source's own order first; those alone by default.
    fn filters(&self) -> Vec<(String, String)> {
        vec![("all".to_owned(), "All".to_owned())]
    }
    fn sorts(&self) -> Vec<(String, String)> {
        vec![("own".to_owned(), "Tool's order".to_owned())]
    }
    /// The header's facts; none by default.
    fn facts(&self) -> Vec<Words> {
        Vec::new()
    }
    /// The pane for the focused row, or the overview with none; `section` is the strip's key.
    fn pane(&self, key: Option<&str>) -> Pane;
    /// The bar with nothing marked: a job's progress, the last outcome, or none.
    fn bar(&self) -> Bar;
    /// A row's one action on A; none leaves A idle.
    fn action(&self, key: &str) -> Option<Action>;
    /// The rows of a row's page under X, each where it applies.
    fn page(&self, key: &str, marked: Option<&Mark>) -> Vec<CardRow>;
    /// The List card's own rows after Search, Filter and Sort; none by default.
    fn list_rows(&self) -> Vec<CardRow> {
        Vec::new()
    }
    /// A row's marks; none by default, for a tool whose rows are not marked and planned.
    fn marks_for(&self, _key: &str) -> Vec<Mark> {
        Vec::new()
    }
    /// The bar's line and the plan's question for the marked rows, by key; a tool with no marks has neither to say.
    fn summary(&self, _marked: &[(String, Mark)]) -> String {
        String::new()
    }
    fn plan(&self, _marked: &[(String, Mark)]) -> Question {
        plan_question(self.title(), &[])
    }
    /// The marked rows' jobs started, once the plan is answered Yes; nothing by default.
    fn apply(&mut self, _marked: &[(String, Mark)]) {}
    /// An action by its key, on a row (A, a page's row) or on none (the List card's rows); `section` is the strip's key.
    fn act(&mut self, action: &str, key: Option<&str>, section: &str) -> Effect;
    /// A question the source asked, answered: `yes` for the answer that does the thing.
    fn answer(&mut self, asked: &str, yes: bool);
    /// Read everything anew, as the screen opens.
    fn refresh(&mut self);
    /// A running job's events taken; true when anything changed, so the view is read again.
    fn poll(&mut self) -> bool;
    /// Whether a job runs, so no row offers an action and no plan is made.
    fn running(&self) -> bool;
    /// The row a run or an approval acted on, to be focused when the list is shown afresh; taken once.
    fn acted_on(&mut self) -> Option<String>;
    /// A row of a row's page stepped by Left or Right (or A, by one): the page's rows again, none where it does not step.
    fn step(&mut self, _key: &str, _item: &str, _by: i32) -> Option<Vec<CardRow>> {
        None
    }
    /// A row's action on A while a job runs (Stop on the running job's row); none by default.
    fn running_action(&self, _key: &str) -> Option<Action> {
        None
    }
    /// Whether rows are marked and applied as a plan; a tool whose rows each start at once has no Apply.
    fn plans(&self) -> bool {
        true
    }
    /// Whether a run's end leaves the library changed, so the library is read again.
    fn changes_library(&self) -> bool {
        false
    }
}

/// The cards over the screen: a row's page (X) and the List card (Select).
#[derive(Clone, Debug, PartialEq)]
pub enum Card {
    Page { key: String, title: String, rows: Vec<CardRow> },
    List,
}

/// What a question on the screen answers: the plan, or one the source asked by its key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Asked {
    Plan,
    Source(String),
}

/// A help bar's entry: the button, its words, and the action it invokes where the frame does not answer it itself.
pub type Entry = (UiButton, String, Option<&'static str>);

/// The screen's declared actions: X the row's page, Select the List card, Start the plan.
pub const MORE: &str = "more";
pub const LIST: &str = "list";
pub const APPLY: &str = "apply";

/// The frame's own words: the search box's hint and the apply control's label.
pub const SEARCH_HINT: &str = "Search";
pub const APPLY_LABEL: &str = "Apply…";

/// The List card's own rows.
pub const SEARCH: &str = "search";
pub const FILTER: &str = "filter";
pub const SORT: &str = "sort";

/// A tool's screen: its source and what the screen holds of its own.
pub struct Tool<S> {
    pub source: S,
    section: usize,
    filter: usize,
    sort: usize,
    search: String,
    focused: Option<String>,
    marks: Vec<(String, Mark)>,
    /// The cards open, the last on top, each with its focused row.
    cards: Vec<(Card, usize)>,
    asked: Option<(Question, Asked)>,
    /// The search box takes words from the docked keyboard, as the List card's Search asks.
    searching: bool,
    /// While searching, whether the keys hold the focus rather than the rows.
    on_keys: bool,
    /// A plan was applied and its jobs run: the marks go when they end.
    applied: bool,
}

impl<S: Source> Tool<S> {
    pub fn new(source: S) -> Tool<S> {
        Tool { source, section: 0, filter: 0, sort: 0, search: String::new(), focused: None, marks: Vec::new(), cards: Vec::new(), asked: None, searching: false, on_keys: false, applied: false }
    }

    /// The rows shown: the section's, passing the filter and the search, in the sort's order.
    pub fn shown(&self) -> Vec<Item> {
        let section = self.source.sections().get(self.section).map(|s| s.0.clone()).unwrap_or_default();
        let filter = self.source.filters().get(self.filter).map(|f| f.0.clone()).unwrap_or_default();
        let words = self.search.trim().to_lowercase();
        let mut items: Vec<Item> = self
            .source
            .items()
            .into_iter()
            .filter(|i| self.section == 0 || i.sections.contains(&section))
            .filter(|i| self.filter == 0 || i.filters.contains(&filter))
            .filter(|i| words.is_empty() || i.search.to_lowercase().contains(&words))
            .collect();
        if self.sort > 0 {
            let at = self.sort - 1;
            items.sort_by(|a, b| a.sorts.get(at).map(|s| s.to_lowercase()).cmp(&b.sorts.get(at).map(|s| s.to_lowercase())));
        }
        items
    }

    /// The rows as drawn: a marked row says its mark's word in its last column, in Accent, while nothing runs.
    pub fn rows(&self) -> Vec<Item> {
        let running = self.source.running();
        self.shown()
            .into_iter()
            .map(|mut item| {
                if let (false, Some(mark), Some(last)) = (running, self.mark_of(&item.key), item.cells.last_mut()) {
                    *last = Words::new(mark.label.clone(), Tone::Accent);
                }
                item
            })
            .collect()
    }

    /// The focused row's index among those shown; the first where the focused row is not shown.
    pub fn focus(&self) -> Option<usize> {
        let shown = self.shown();
        if shown.is_empty() {
            return None;
        }
        Some(self.focused.as_ref().and_then(|k| shown.iter().position(|i| &i.key == k)).unwrap_or(0))
    }

    pub fn focused_key(&self) -> Option<String> {
        self.focus().and_then(|at| self.shown().get(at).map(|i| i.key.clone()))
    }

    pub fn set_focus(&mut self, index: usize) {
        if let Some(item) = self.shown().get(index) {
            self.focused = Some(item.key.clone());
        }
    }

    pub fn section(&self) -> usize {
        self.section
    }

    pub fn section_key(&self) -> String {
        self.source.sections().get(self.section).map(|s| s.0.clone()).unwrap_or_default()
    }

    /// The strip's section chosen by its label, as the frame's shoulders and a click choose it.
    pub fn choose_section(&mut self, label: &str) {
        if let Some(at) = self.source.sections().iter().position(|s| s.1 == label) {
            self.section = at;
        }
    }

    pub fn search(&self) -> &str {
        &self.search
    }

    pub fn set_search(&mut self, words: &str) {
        self.search = words.to_owned();
    }

    pub fn mark_of(&self, key: &str) -> Option<&Mark> {
        self.marks.iter().find(|(k, _)| k == key).map(|(_, m)| m)
    }

    pub fn marked(&self) -> &[(String, Mark)] {
        &self.marks
    }

    /// The row marked with the source's mark of this key; one of the row's own marks only, and never while a job runs.
    pub fn set_mark(&mut self, key: &str, mark: Option<&str>) -> bool {
        if self.source.running() {
            return false;
        }
        self.marks.retain(|(k, _)| k != key);
        let Some(mark) = mark else { return true };
        let Some(found) = self.source.marks_for(key).into_iter().find(|m| m.key == mark) else { return false };
        self.marks.push((key.to_owned(), found));
        true
    }

    /// The header's facts, the pane and the bar as drawn.
    pub fn pane(&self) -> Pane {
        self.source.pane(self.focused_key().as_deref())
    }

    pub fn bar(&self) -> Bar {
        let mut bar = self.source.bar();
        if bar.progress.is_none() && !self.marks.is_empty() {
            bar.line = Words::text(self.source.summary(&self.marks));
        } else if bar.progress.is_none() && bar.line.words.is_empty() {
            bar.line = Words::text("Nothing marked");
        }
        bar
    }

    /// Apply is offered while something is marked and nothing runs.
    pub fn can_apply(&self) -> bool {
        !self.marks.is_empty() && !self.source.running()
    }

    /// The focused row's action on A, none while a job runs.
    pub fn action(&self) -> Option<Action> {
        if self.source.running() {
            return self.focused_key().and_then(|k| self.source.running_action(&k));
        }
        self.focused_key().and_then(|k| self.source.action(&k))
    }

    /// The help bar: A by the row's action, B, the shoulders, then X, Select and Start where they mean something now.
    pub fn help(&self) -> Vec<Entry> {
        let running = self.source.running();
        let mut entries = Vec::new();
        if let Some(action) = self.action() {
            entries.push((UiButton::Accept, action.label, None));
        }
        entries.push((UiButton::Back, "Back".to_owned(), None));
        entries.push((UiButton::PageUp, self.source.section_word().to_owned(), None));
        if !running && self.focus().is_some() {
            entries.push((UiButton::Screensaver, "More".to_owned(), Some(MORE)));
        }
        entries.push((UiButton::Options, "List".to_owned(), Some(LIST)));
        if !running && self.source.plans() {
            entries.push((UiButton::Menu, APPLY_LABEL.to_owned(), Some(APPLY)));
        }
        entries
    }

    /// The screen's actions as the kit's frame reads them, `key Button` a line, whether or not the bar names them now; while
    /// a search's rows hold the focus, the shoulders are the search's, back to the keys.
    pub fn action_lines(&self) -> String {
        let shoulders = [(crate::help::PREVIOUS, UiButton::PageUp), (crate::help::NEXT, UiButton::PageDown)];
        let search = if self.searching && !self.on_keys { &shoulders[..] } else { &[] };
        [(MORE, UiButton::Screensaver), (LIST, UiButton::Options), (APPLY, UiButton::Menu)].iter().chain(search).map(|(k, b)| format!("{k} {b:?}\n")).collect()
    }

    /// A on a row: its action, which may ask, mark or turn to the host.
    pub fn activate(&mut self, index: usize) -> Option<String> {
        self.set_focus(index);
        let action = self.action()?;
        let key = self.focused_key()?;
        let section = self.section_key();
        let effect = self.source.act(&action.key, Some(&key), &section);
        self.take(effect, Some(&key))
    }

    /// X on a row: its page, over the screen.
    pub fn more(&mut self) {
        let Some(key) = self.focused_key() else { return };
        if self.source.running() {
            return;
        }
        let title = self.shown().into_iter().find(|i| i.key == key).and_then(|i| i.cells.first().map(|c| c.words.clone())).unwrap_or_default();
        let rows = self.source.page(&key, self.mark_of(&key));
        self.cards = vec![(Card::Page { key, title, rows }, 0)];
    }

    /// Select: the List card.
    pub fn list(&mut self) {
        self.cards = vec![(Card::List, 0)];
    }

    /// Start: the plan, asked while something is marked.
    pub fn apply(&mut self) {
        if self.can_apply() {
            self.asked = Some((self.source.plan(&self.marks), Asked::Plan));
        }
    }

    pub fn question(&self) -> Option<&Question> {
        self.asked.as_ref().map(|(q, _)| q)
    }

    /// The question answered by the choice's index; -1 (B with no answer) or the No answer changes nothing.
    pub fn answer(&mut self, choice: i32) {
        let Some((question, asked)) = self.asked.take() else { return };
        let yes = choice >= 0 && choice as usize + 1 == question.choices.len() && question.back != Some(choice as usize);
        match (asked, yes) {
            (Asked::Plan, true) => {
                let marked = self.marks.clone();
                self.applied = true;
                self.source.apply(&marked);
                self.end_run();
            }
            (Asked::Plan, false) => {}
            (Asked::Source(key), yes) => {
                self.source.answer(&key, yes);
                self.end_run();
            }
        }
    }

    /// The card on top, its rows and its focused row.
    pub fn card(&self) -> Option<(String, String, Vec<CardRow>, usize)> {
        let (card, at) = self.cards.last()?;
        let breadcrumb = if self.cards.len() > 1 { self.cards[self.cards.len() - 2].0.title(self) } else { self.source.title().to_owned() };
        Some((card.title(self), breadcrumb, self.card_rows(card), *at))
    }

    fn card_rows(&self, card: &Card) -> Vec<CardRow> {
        match card {
            Card::Page { rows, .. } => rows.clone(),
            Card::List => {
                let value = |list: Vec<(String, String)>, at: usize| list.get(at).map(|s| s.1.clone()).unwrap_or_default();
                let mut rows = vec![
                    CardRow::new(SEARCH, "Search"),
                    CardRow::new(FILTER, "Filter").value(&value(self.source.filters(), self.filter)).steps(),
                    CardRow { section_end: true, ..CardRow::new(SORT, "Sort").value(&value(self.source.sorts(), self.sort)).steps() },
                ];
                rows.extend(self.source.list_rows());
                rows
            }
        }
    }

    pub fn card_focus(&mut self, index: usize) {
        if let Some((_, at)) = self.cards.last_mut() {
            *at = index;
        }
    }

    /// Left or Right on a card's row: the List card's filter and sort step through theirs.
    pub fn card_step(&mut self, index: usize, by: i32) {
        let Some(row) = self.card().and_then(|c| c.2.get(index).cloned()) else { return };
        if let Some((Card::Page { key, rows, .. }, _)) = self.cards.last_mut() {
            if let Some(again) = self.source.step(key, &row.key, by) {
                *rows = again;
            }
            return;
        }
        let step = |at: usize, n: usize| (at as i32 + by).rem_euclid(n.max(1) as i32) as usize;
        match row.key.as_str() {
            FILTER => self.filter = step(self.filter, self.source.filters().len()),
            SORT => self.sort = step(self.sort, self.source.sorts().len()),
            _ => {}
        }
    }

    /// A on a card's row; a card closes on what it does unless it turns a page.
    pub fn card_choose(&mut self, index: usize) -> Option<String> {
        let (card, rows) = match self.cards.last() {
            Some((card, _)) => (card.clone(), self.card_rows(card)),
            None => return None,
        };
        let row = rows.get(index)?.clone();
        self.card_focus(index);
        match (&card, row.key.as_str()) {
            (Card::List, FILTER | SORT) => {
                self.card_step(index, 1);
                None
            }
            (Card::List, SEARCH) => {
                self.cards.clear();
                self.searching = true;
                self.on_keys = true;
                None
            }
            (Card::List, action) => {
                self.cards.clear();
                let section = self.section_key();
                let effect = self.source.act(action, None, &section);
                self.take(effect, None)
            }
            (Card::Page { key, .. }, action) => {
                let key = key.clone();
                if let Some(again) = self.source.step(&key, action, 1) {
                    if let Some((Card::Page { rows, .. }, _)) = self.cards.last_mut() {
                        *rows = again;
                    }
                    return None;
                }
                let section = self.section_key();
                let effect = self.source.act(action, Some(&key), &section);
                if !matches!(effect, Effect::Page(..)) {
                    self.cards.clear();
                }
                self.take(effect, Some(&key))
            }
        }
    }

    /// Whether the search box takes words from the docked keyboard.
    pub fn searching(&self) -> bool {
        self.searching
    }

    /// While searching, whether the keys hold the focus.
    pub fn search_keys(&self) -> bool {
        self.searching && self.on_keys
    }

    /// A shoulder's action while searching: the focus between the keys and the rows, by the search's one rule
    /// (`screens::search_shoulder`); false when it is no such move.
    pub fn shoulder(&mut self, key: &str) -> bool {
        match crate::screens::search_shoulder(self.on_keys, key).filter(|_| self.searching) {
            Some(keys) => {
                self.on_keys = keys;
                true
            }
            None => false,
        }
    }

    /// While searching, the focus moved to the keys or the rows by the pointer.
    pub fn focus_keys(&mut self, keys: bool) {
        if self.searching {
            self.on_keys = keys;
        }
    }

    /// The docked keyboard closed, its words kept.
    pub fn end_search(&mut self) {
        self.searching = false;
    }

    /// B on a card: its last page, or the card closed.
    pub fn card_back(&mut self) {
        self.cards.pop();
    }

    pub fn card_open(&self) -> bool {
        !self.cards.is_empty()
    }

    /// Read anew as the screen opens: the cards and the question closed, the marks kept.
    pub fn refresh(&mut self) {
        self.cards.clear();
        self.asked = None;
        self.searching = false;
        self.source.refresh();
    }

    /// The source's jobs taken; a run's end clears the marks, and the row it acted on is focused again.
    pub fn poll(&mut self) -> bool {
        let changed = self.source.poll();
        self.end_run();
        changed
    }

    fn end_run(&mut self) {
        if self.applied && !self.source.running() {
            self.applied = false;
            self.marks.clear();
        }
        if let Some(key) = self.source.acted_on() {
            if self.shown().iter().any(|i| i.key == key) {
                self.focused = Some(key);
            }
        }
    }

    fn take(&mut self, effect: Effect, key: Option<&str>) -> Option<String> {
        match effect {
            Effect::Done => None,
            Effect::Ask(question, asked) => {
                self.asked = Some((question, Asked::Source(asked)));
                None
            }
            Effect::Mark(mark, ask) => {
                if let Some(key) = key {
                    if self.set_mark(key, Some(&mark)) && ask {
                        self.apply();
                    }
                }
                None
            }
            Effect::Unmark => {
                if let Some(key) = key {
                    self.set_mark(key, None);
                }
                None
            }
            Effect::Page(title, rows) => {
                self.cards.push((Card::Page { key: key.unwrap_or_default().to_owned(), title, rows }, 0));
                None
            }
            Effect::Host(destination) => Some(destination),
        }
    }
}

impl Card {
    fn title<S: Source>(&self, tool: &Tool<S>) -> String {
        match self {
            Card::Page { title, .. } => title.clone(),
            Card::List => tool.source.title().to_owned(),
        }
    }
}

/// The plan's question as every tool asks it: No first and focused, Yes; B answers No (EmuSen_Interface.md §15.6).
pub fn plan_question(title: &str, sentences: &[String]) -> Question {
    let words: Vec<&str> = sentences.iter().map(String::as_str).collect();
    confirm(title, &words)
}

/// A source chosen while the program runs, as `ToolModel` holds one.
impl Source for Box<dyn Source> {
    fn title(&self) -> &str {
        (**self).title()
    }
    fn section_word(&self) -> &str {
        (**self).section_word()
    }
    fn columns(&self) -> Vec<Column> {
        (**self).columns()
    }
    fn items(&self) -> Vec<Item> {
        (**self).items()
    }
    fn sections(&self) -> Vec<(String, String)> {
        (**self).sections()
    }
    fn filters(&self) -> Vec<(String, String)> {
        (**self).filters()
    }
    fn sorts(&self) -> Vec<(String, String)> {
        (**self).sorts()
    }
    fn facts(&self) -> Vec<Words> {
        (**self).facts()
    }
    fn pane(&self, key: Option<&str>) -> Pane {
        (**self).pane(key)
    }
    fn bar(&self) -> Bar {
        (**self).bar()
    }
    fn action(&self, key: &str) -> Option<Action> {
        (**self).action(key)
    }
    fn page(&self, key: &str, marked: Option<&Mark>) -> Vec<CardRow> {
        (**self).page(key, marked)
    }
    fn list_rows(&self) -> Vec<CardRow> {
        (**self).list_rows()
    }
    fn marks_for(&self, key: &str) -> Vec<Mark> {
        (**self).marks_for(key)
    }
    fn summary(&self, marked: &[(String, Mark)]) -> String {
        (**self).summary(marked)
    }
    fn plan(&self, marked: &[(String, Mark)]) -> Question {
        (**self).plan(marked)
    }
    fn apply(&mut self, marked: &[(String, Mark)]) {
        (**self).apply(marked)
    }
    fn act(&mut self, action: &str, key: Option<&str>, section: &str) -> Effect {
        (**self).act(action, key, section)
    }
    fn answer(&mut self, asked: &str, yes: bool) {
        (**self).answer(asked, yes)
    }
    fn refresh(&mut self) {
        (**self).refresh()
    }
    fn poll(&mut self) -> bool {
        (**self).poll()
    }
    fn running(&self) -> bool {
        (**self).running()
    }
    fn acted_on(&mut self) -> Option<String> {
        (**self).acted_on()
    }
    fn step(&mut self, key: &str, item: &str, by: i32) -> Option<Vec<CardRow>> {
        (**self).step(key, item, by)
    }
    fn running_action(&self, key: &str) -> Option<Action> {
        (**self).running_action(key)
    }
    fn plans(&self) -> bool {
        (**self).plans()
    }
    fn changes_library(&self) -> bool {
        (**self).changes_library()
    }
}
