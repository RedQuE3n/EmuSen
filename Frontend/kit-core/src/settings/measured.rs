//! `MeasuredWalk`: the rows' walk of `RowWalk` over rows of their own heights, in a window of a given height, with no
//! geometry of its own (EmuSen_Kit.md §7.122). The window moves only when the focus leaves it, by whole rows.

use super::rows::{Row, RowOutcome, RowWalk};
use crate::input::UiButton;

#[derive(Clone, Debug, Default)]
pub struct MeasuredWalk {
    walk: RowWalk,
    heights: Vec<f64>,
    gap: f64,
    window: f64,
    first: usize,
}

impl MeasuredWalk {
    /// A walk with no rows; its window is unset, so every row is shown until a height is given.
    pub fn new() -> MeasuredWalk {
        MeasuredWalk { walk: RowWalk::new(0), ..MeasuredWalk::default() }
    }

    /// The rows, kept as `RowWalk::set_rows` keeps them; a new list (not `keep`) starts the window at its top.
    pub fn set_rows(&mut self, rows: Vec<Row>, keep: bool) {
        self.walk.set_rows(rows, keep);
        if !keep {
            self.first = 0;
        }
        self.keep_shown();
    }

    /// Each row's height as drawn, in the rows' order; a row with none counts as 0 high.
    pub fn set_heights(&mut self, heights: Vec<f64>) {
        self.heights = heights;
        self.keep_shown();
    }

    /// One row's height as drawn, the list grown where it is shorter.
    pub fn set_height(&mut self, row: usize, height: f64) {
        if self.heights.len() <= row {
            self.heights.resize(row + 1, 0.0);
        }
        self.heights[row] = height.max(0.0);
        self.keep_shown();
    }

    /// The space between two rows.
    pub fn set_gap(&mut self, gap: f64) {
        self.gap = gap.max(0.0);
        self.keep_shown();
    }

    /// The window's height; 0 or less is none, every row shown from the first.
    pub fn set_window(&mut self, height: f64) {
        self.window = height;
        self.keep_shown();
    }

    pub fn set_sections(&mut self, count: usize) {
        self.walk.set_sections(count);
    }

    pub fn select_section(&mut self, section: usize) -> bool {
        self.walk.select_section(section)
    }

    pub fn set_trail(&mut self, open: bool) {
        self.walk.set_trail(open);
    }

    pub fn section(&self) -> usize {
        self.walk.section()
    }

    pub fn rows(&self) -> &[Row] {
        self.walk.rows()
    }

    pub fn current(&self) -> usize {
        self.walk.current()
    }

    pub fn marked(&self) -> Option<usize> {
        self.walk.marked()
    }

    /// The first row the window shows.
    pub fn first(&self) -> usize {
        self.first
    }

    /// How far the first row shown stands below the first row: the window's scroll.
    pub fn offset(&self) -> f64 {
        (0..self.first).map(|i| self.height(i) + self.gap).sum()
    }

    /// The pointer resting on a row focuses it, as `RowWalk::focus`; the window does not move.
    pub fn focus(&mut self, row: usize) -> bool {
        self.walk.focus(row)
    }

    /// The wheel moved the window to start at this row, held so that no empty space shows past the last; the focus stays.
    pub fn scroll_to(&mut self, first: usize) -> bool {
        let first = first.min(self.last_first());
        let moved = first != self.first;
        self.first = first;
        moved
    }

    /// A press in `RowWalk`'s grammar, the window kept around the focus.
    pub fn press(&mut self, button: UiButton) -> RowOutcome {
        let outcome = self.walk.press(button);
        if let RowOutcome::Moved(_) = outcome {
            self.keep_shown();
        }
        outcome
    }

    fn height(&self, row: usize) -> f64 {
        self.heights.get(row).copied().unwrap_or(0.0)
    }

    /// The height rows `from` to `to`, both included, take with their gaps.
    fn span(&self, from: usize, to: usize) -> f64 {
        (from..=to).map(|i| self.height(i)).sum::<f64>() + self.gap * (to - from) as f64
    }

    /// The last first row that leaves no empty space under the last row.
    fn last_first(&self) -> usize {
        let count = self.walk.rows().len();
        if count == 0 || self.window <= 0.0 {
            return 0;
        }
        let mut first = count - 1;
        while first > 0 && self.span(first - 1, count - 1) <= self.window {
            first -= 1;
        }
        first
    }

    fn keep_shown(&mut self) {
        if self.window <= 0.0 {
            self.first = 0;
            return;
        }
        let current = self.walk.current();
        if current < self.first {
            self.first = current;
        }
        while self.first < current && self.span(self.first, current) > self.window {
            self.first += 1;
        }
        self.first = self.first.min(self.last_first());
    }
}
