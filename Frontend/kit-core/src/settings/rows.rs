//! The walk over a settings page's rows (EmuSen_Kit.md §7.40): Up and Down move the focus and stop at the ends, Left and
//! Right step a stepped value or a slider, A chooses, a page of choices keeps its mark on the one chosen, and the shoulders
//! step the strip's sections, wrapping, while no page is open under them. The window
//! of rows shown moves only when the focus leaves it, a row at a time, as LunaP's `KeepRowShown` did.

use super::slider::SliderRange;
use crate::input::UiButton;

/// What a row does, in the order `Kit.RowKinds` numbers them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum RowKind {
    /// A status or an action: A does it.
    #[default]
    Action,
    /// A value between ◂ and ▸: Left and Right step it.
    Steps,
    /// A row that opens a page of its own: A opens it.
    Opens,
    /// A row that asks first: A asks.
    Asks,
    /// A slider: Left and Right step it, the pointer drags it.
    Slider,
    /// One of a page of choices: A chooses it, and the mark moves to it.
    Choice,
}

impl RowKind {
    pub const ALL: [RowKind; 6] = [RowKind::Action, RowKind::Steps, RowKind::Opens, RowKind::Asks, RowKind::Slider, RowKind::Choice];

    /// The kind by its number; an unknown number is an action.
    pub fn from_index(index: i64) -> RowKind {
        usize::try_from(index).ok().and_then(|i| Self::ALL.get(i).copied()).unwrap_or_default()
    }
}

/// One row as the walk sees it: its kind, whether something holds it (shown with why, taking no change), whether it is
/// the marked choice, and a slider's range and amount.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Row {
    pub kind: RowKind,
    pub held: bool,
    pub marked: bool,
    pub range: SliderRange,
    pub amount: f64,
}

impl Row {
    pub fn of(kind: RowKind) -> Row {
        Row { kind, ..Row::default() }
    }
}

/// What a press did.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RowOutcome {
    /// The focus moved to this row.
    Moved(usize),
    /// Left (-1) or Right (+1) stepped this row's value.
    Stepped { row: usize, by: i32 },
    /// Left or Right moved this slider to an amount.
    Slid { row: usize, amount: f64 },
    /// A chose this row: the host acts, opens, asks or takes the choice.
    Chosen(usize),
    /// A shoulder stepped the strip to this section.
    Section(usize),
    /// The walk took the press and nothing changed: an end of the rows, a held row, Left on a row that does not step.
    Kept,
    /// Not the walk's: the frame's or the host's (B, the shoulders, Start).
    Declined,
}

#[derive(Clone, Debug, Default)]
pub struct RowWalk {
    rows: Vec<Row>,
    current: usize,
    first: usize,
    shown: usize,
    sections: usize,
    section: usize,
    trail: bool,
}

impl RowWalk {
    pub fn new(shown: usize) -> RowWalk {
        RowWalk { shown, ..RowWalk::default() }
    }

    /// Gives the walk its rows: the focus stays on the same index where it still exists, else goes to the marked choice,
    /// else the first row; a page of choices opens on its mark.
    pub fn set_rows(&mut self, rows: Vec<Row>, keep: bool) {
        let marked = rows.iter().position(|r| r.marked);
        self.current = match (keep, marked) {
            (true, _) if self.current < rows.len() => self.current,
            (_, Some(m)) => m,
            _ => 0,
        };
        if !keep {
            self.first = 0;
        }
        self.rows = rows;
        self.keep_shown();
    }

    /// How many rows the window holds.
    pub fn set_shown(&mut self, shown: usize) {
        self.shown = shown;
        self.keep_shown();
    }

    /// How many sections the strip holds; the chosen one is kept where it still exists.
    pub fn set_sections(&mut self, count: usize) {
        self.sections = count;
        if self.section >= count {
            self.section = 0;
        }
    }

    /// Chooses a section as the host does, raising nothing.
    pub fn select_section(&mut self, section: usize) -> bool {
        if section >= self.sections {
            return false;
        }
        self.section = section;
        true
    }

    /// Whether pages are open under the section: the strip draws their trail and the shoulders are the host's.
    pub fn set_trail(&mut self, open: bool) {
        self.trail = open;
    }

    pub fn section(&self) -> usize {
        self.section
    }

    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    pub fn current(&self) -> usize {
        self.current
    }

    pub fn first(&self) -> usize {
        self.first
    }

    pub fn marked(&self) -> Option<usize> {
        self.rows.iter().position(|r| r.marked)
    }

    /// The pointer resting on a row focuses it; the window does not move.
    pub fn focus(&mut self, row: usize) -> bool {
        if row >= self.rows.len() || row == self.current {
            return false;
        }
        self.current = row;
        true
    }

    /// The wheel moved the window to start at this row, held so that no empty row shows past the last; the focus stays.
    pub fn scroll_to(&mut self, first: usize) -> bool {
        let first = first.min(self.last_first());
        let moved = first != self.first;
        self.first = first;
        moved
    }

    /// A press in the rows' grammar.
    pub fn press(&mut self, button: UiButton) -> RowOutcome {
        if let UiButton::PageUp | UiButton::PageDown = button {
            if self.trail || self.sections < 2 {
                return RowOutcome::Declined;
            }
            let by = if button == UiButton::PageUp { self.sections - 1 } else { 1 };
            self.section = (self.section + by) % self.sections;
            return RowOutcome::Section(self.section);
        }
        if self.rows.is_empty() {
            return RowOutcome::Declined;
        }
        match button {
            UiButton::Up | UiButton::Down => {
                let to = if button == UiButton::Up { self.current.checked_sub(1) } else { Some(self.current + 1) };
                match to.filter(|&to| to < self.rows.len()) {
                    Some(to) => {
                        self.current = to;
                        self.keep_shown();
                        RowOutcome::Moved(to)
                    }
                    None => RowOutcome::Kept,
                }
            }
            UiButton::Left | UiButton::Right => {
                let row = self.rows[self.current];
                let by = if button == UiButton::Left { -1 } else { 1 };
                match row.kind {
                    _ if row.held => RowOutcome::Kept,
                    RowKind::Steps => RowOutcome::Stepped { row: self.current, by },
                    RowKind::Slider => {
                        let amount = row.range.stepped(row.amount, by);
                        if amount == row.amount {
                            return RowOutcome::Kept;
                        }
                        self.rows[self.current].amount = amount;
                        RowOutcome::Slid { row: self.current, amount }
                    }
                    _ => RowOutcome::Kept,
                }
            }
            UiButton::Accept => {
                let row = self.rows[self.current];
                if row.held {
                    return RowOutcome::Kept;
                }
                if row.kind == RowKind::Choice {
                    for (i, r) in self.rows.iter_mut().enumerate() {
                        r.marked = i == self.current;
                    }
                }
                RowOutcome::Chosen(self.current)
            }
            _ => RowOutcome::Declined,
        }
    }

    fn last_first(&self) -> usize {
        self.rows.len().saturating_sub(self.shown)
    }

    fn keep_shown(&mut self) {
        if self.current < self.first {
            self.first = self.current;
        }
        if self.shown > 0 && self.current >= self.first + self.shown {
            self.first = self.current + 1 - self.shown;
        }
        self.first = self.first.min(self.last_first());
    }
}
