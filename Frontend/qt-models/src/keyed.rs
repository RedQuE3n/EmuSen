//! Rows held by key with one selected: replaced whole on a refresh, the selection kept by its key, or kept near its old
//! place when its row is gone (EmuSen_Qt.md §1.2.1).

/// A row known by a key that stays the same across refreshes.
pub trait Keyed {
    fn key(&self) -> &str;
}

/// The rows and the selected one.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct KeyedRows<T> {
    rows: Vec<T>,
    selected: Option<usize>,
}

impl<T: Keyed> KeyedRows<T> {
    /// The rows in order.
    pub fn rows(&self) -> &[T] {
        &self.rows
    }

    /// The selected row's index, none with no rows.
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    /// The selected row's key.
    pub fn selected_key(&self) -> Option<&str> {
        self.selected.and_then(|i| self.rows.get(i)).map(Keyed::key)
    }

    /// Selects the row at `index`, when there is one; true when it did.
    pub fn select(&mut self, index: usize) -> bool {
        let ok = index < self.rows.len();
        if ok {
            self.selected = Some(index);
        }
        ok
    }

    /// The rows replaced: the selection follows its key; where the key is gone it stays at its index, clamped to the
    /// last row; with no selection before, the first row is selected.
    pub fn replace(&mut self, rows: Vec<T>) -> Vec<T> {
        let key = self.selected_key().map(str::to_owned);
        let before = self.selected;
        let old = std::mem::replace(&mut self.rows, rows);
        self.selected = match (key, before) {
            _ if self.rows.is_empty() => None,
            (Some(key), Some(at)) => Some(self.rows.iter().position(|r| r.key() == key).unwrap_or(at.min(self.rows.len() - 1))),
            _ => Some(0),
        };
        old
    }
}
