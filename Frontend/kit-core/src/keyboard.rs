//! The docked keyboard as data and rules: its layouts as lines of keys, the box of every key, the walk over the keys, and
//! what a key, B and Done do to the text (EmuSen_Kit.md §7.22). It names no Qt type; the host draws the keys it returns.

/// A key that does something other than type its own characters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Special {
    Erase,
    Space,
    Shift,
    /// Shows the next layout; the key is labelled with that layout's name.
    Next,
    Done,
}

/// One key: what it types or does, and how many key widths it takes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeySpec {
    pub key: KeyKind,
    pub width: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyKind {
    Chars(String),
    Special(Special),
}

/// A layout: its name and its rows of keys.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layout {
    pub name: String,
    pub rows: Vec<Vec<KeySpec>>,
}

/// The search keyboard's letters, as LunaP's `KeyboardLayout.Search` laid them out and Mistress's search shows them.
pub const LETTERS: &str = "Letters
1 2 3 4 5 6 7 8 9 0
q w e r t y u i o p
a s d f g h j k l '
Shift:2 z x c v b n m , . - Erase:2
Next:2 Space:5 Done:2";

/// Its symbols, LunaP's `KeyboardLayout.Symbols`.
pub const SYMBOLS: &str = "Symbols
! @ # $ % & * ( ) ?
+ = / : ; \" < > _ ~
[ ] { } | ^ ` € £ ¥
Shift:2 , . - ' Erase:2
Next:2 Space:5 Done:2";

impl Layout {
    /// Reads a layout from lines: the name, then one row a line, keys apart by spaces, a width after a colon
    /// (`Space:5`); `Erase`, `Space`, `Shift`, `Next` and `Done` name the special keys. None for no name or no rows.
    pub fn parse(text: &str) -> Option<Layout> {
        let mut lines = text.lines().map(str::trim).filter(|l| !l.is_empty());
        let name = lines.next()?.to_string();
        let rows: Vec<Vec<KeySpec>> = lines.map(|line| line.split_whitespace().map(key_of).collect()).collect();
        if rows.is_empty() {
            return None;
        }
        Some(Layout { name, rows })
    }

    fn row_units(&self, row: usize) -> u32 {
        self.rows[row].iter().map(|k| k.width).sum()
    }

    /// Where a key's middle falls, in key widths from its row's middle.
    fn middle(&self, row: usize, column: usize) -> f64 {
        let before: u32 = self.rows[row][..column].iter().map(|k| k.width).sum();
        f64::from(before) + f64::from(self.rows[row][column].width) / 2.0 - f64::from(self.row_units(row)) / 2.0
    }
}

fn key_of(word: &str) -> KeySpec {
    let (name, width) = match word.rsplit_once(':') {
        Some((name, width)) if !name.is_empty() => (name, width.parse().unwrap_or(1).max(1)),
        _ => (word, 1),
    };
    let key = match name {
        "Erase" => KeyKind::Special(Special::Erase),
        "Space" => KeyKind::Special(Special::Space),
        "Shift" => KeyKind::Special(Special::Shift),
        "Next" => KeyKind::Special(Special::Next),
        "Done" => KeyKind::Special(Special::Done),
        chars => KeyKind::Chars(chars.to_string()),
    };
    KeySpec { key, width }
}

/// The docked band's measures in design units, as Mistress's docked keyboard draws them (EmuSen_Interface.md §18.1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Measures {
    pub key_width: f64,
    pub key_height: f64,
    pub gap: f64,
    pub padding: f64,
    /// The rule along the band's top, inside its height; the keys stand the padding under it.
    pub rule: f64,
}

pub const DOCKED: Measures = Measures { key_width: 72.0, key_height: 48.0, gap: 8.0, padding: 16.0, rule: 1.0 };

impl Measures {
    /// A key's box as (x from the band's centre, y from its top, width, height).
    pub fn key_box(&self, layout: &Layout, row: usize, column: usize) -> (f64, f64, f64, f64) {
        let unit = self.key_width + self.gap;
        let row_width = f64::from(layout.row_units(row)) * unit - self.gap;
        let before: u32 = layout.rows[row][..column].iter().map(|k| k.width).sum();
        let width = f64::from(layout.rows[row][column].width) * unit - self.gap;
        let y = self.rule + self.padding + row as f64 * (self.key_height + self.gap);
        (-row_width / 2.0 + f64::from(before) * unit, y, width, self.key_height)
    }

    /// The band's height for a layout: its rows and the padding above and below, the rule within the top's (Mistress's
    /// band is 304 high for five rows, its keys 15 above its foot).
    pub fn height(&self, layout: &Layout) -> f64 {
        let rows = layout.rows.len() as f64;
        2.0 * self.padding + rows * self.key_height + (rows - 1.0).max(0.0) * self.gap
    }
}

/// What a press did, for the host to answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Nothing changed.
    Nothing,
    /// The current key moved, the layout changed or the shift turned.
    Moved,
    /// The text changed.
    Typed,
    /// The keyboard closed: kept for Done, not kept for B on an empty field.
    Closed { kept: bool },
}

/// A docked keyboard's state: its layouts, the current key, the shift, and the text with its caret (in characters).
#[derive(Clone, Debug)]
pub struct Keyboard {
    layouts: Vec<Layout>,
    layout: usize,
    row: usize,
    column: usize,
    shifted: bool,
    text: Vec<char>,
    caret: usize,
}

impl Keyboard {
    /// A keyboard over its layouts, the first shown first, the current key the first of the first row, as LunaP's opens.
    /// Panics on no layouts, a host's mistake rather than a player's.
    pub fn new(layouts: Vec<Layout>) -> Keyboard {
        assert!(!layouts.is_empty(), "a keyboard needs a layout");
        Keyboard { layouts, layout: 0, row: 0, column: 0, shifted: false, text: Vec::new(), caret: 0 }
    }

    /// The search keyboard: letters, then symbols.
    pub fn search() -> Keyboard {
        Keyboard::new([LETTERS, SYMBOLS].iter().filter_map(|t| Layout::parse(t)).collect())
    }

    pub fn layout(&self) -> &Layout {
        &self.layouts[self.layout]
    }

    pub fn current(&self) -> (usize, usize) {
        (self.row, self.column)
    }

    pub fn shifted(&self) -> bool {
        self.shifted
    }

    pub fn text(&self) -> String {
        self.text.iter().collect()
    }

    pub fn caret(&self) -> usize {
        self.caret
    }

    /// The words on a key: its characters, upper-cased while shifted; a special key's own name, `⌫` for Erase, and the
    /// next layout's name for Next.
    pub fn label(&self, row: usize, column: usize) -> String {
        match &self.layout().rows[row][column].key {
            KeyKind::Chars(chars) if self.shifted => chars.to_uppercase(),
            KeyKind::Chars(chars) => chars.clone(),
            KeyKind::Special(Special::Erase) => "⌫".to_string(),
            KeyKind::Special(Special::Space) => "Space".to_string(),
            KeyKind::Special(Special::Shift) => "Shift".to_string(),
            KeyKind::Special(Special::Done) => "Done".to_string(),
            KeyKind::Special(Special::Next) => self.layouts[(self.layout + 1) % self.layouts.len()].name.clone(),
        }
    }

    /// Replaces the text and puts the caret in it, as a field typed into by a physical keyboard reports it.
    pub fn set_text(&mut self, text: &str, caret: usize) {
        self.text = text.chars().collect();
        self.caret = caret.min(self.text.len());
    }

    /// Moves the current key: across a row wrapping at its ends; up or down to the key whose middle is nearest the
    /// current one's, the first of two as near, stopping at the top and the foot.
    pub fn move_by(&mut self, columns: i32, rows: i32) -> Outcome {
        let before = (self.row, self.column);
        if rows != 0 {
            let middle = self.layout().middle(self.row, self.column);
            let last = self.layout().rows.len() as i32 - 1;
            self.row = (self.row as i32 + rows).clamp(0, last) as usize;
            let row = self.row;
            let layout = self.layout();
            self.column = (0..layout.rows[row].len())
                .min_by(|&a, &b| (layout.middle(row, a) - middle).abs().total_cmp(&(layout.middle(row, b) - middle).abs()))
                .unwrap_or(0);
        }
        if columns != 0 {
            let count = self.layout().rows[self.row].len() as i32;
            self.column = (self.column as i32 + columns).rem_euclid(count) as usize;
        }
        if (self.row, self.column) == before { Outcome::Nothing } else { Outcome::Moved }
    }

    /// Makes a key the current one, as a pointer resting on it does.
    pub fn point(&mut self, row: usize, column: usize) -> Outcome {
        if row >= self.layout().rows.len() || column >= self.layout().rows[row].len() || (row, column) == (self.row, self.column) {
            return Outcome::Nothing;
        }
        self.row = row;
        self.column = column;
        Outcome::Moved
    }

    /// A on the current key.
    pub fn press(&mut self) -> Outcome {
        let key = self.layout().rows[self.row][self.column].key.clone();
        self.type_key(&key)
    }

    /// What a key does: characters go in at the caret (upper-cased and the shift let go while shifted), the special keys
    /// their own work.
    pub fn type_key(&mut self, key: &KeyKind) -> Outcome {
        match key {
            KeyKind::Chars(chars) => {
                let typed = if self.shifted { chars.to_uppercase() } else { chars.clone() };
                self.insert(&typed);
                self.shifted = false;
                Outcome::Typed
            }
            KeyKind::Special(Special::Space) => {
                self.insert(" ");
                Outcome::Typed
            }
            KeyKind::Special(Special::Erase) => {
                if self.erase() { Outcome::Typed } else { Outcome::Nothing }
            }
            KeyKind::Special(Special::Shift) => {
                self.shifted = !self.shifted;
                Outcome::Moved
            }
            KeyKind::Special(Special::Next) => {
                self.layout = (self.layout + 1) % self.layouts.len();
                self.row = self.row.min(self.layout().rows.len() - 1);
                self.column = self.column.min(self.layout().rows[self.row].len() - 1);
                Outcome::Moved
            }
            KeyKind::Special(Special::Done) => Outcome::Closed { kept: true },
        }
    }

    /// B: the character before the caret is erased; with nothing to erase, the keyboard closes (EmuSen_Interface.md §18.1).
    pub fn back(&mut self) -> Outcome {
        if self.erase() { Outcome::Typed } else { Outcome::Closed { kept: false } }
    }

    fn insert(&mut self, chars: &str) {
        for (i, c) in chars.chars().enumerate() {
            self.text.insert(self.caret + i, c);
        }
        self.caret += chars.chars().count();
    }

    fn erase(&mut self) -> bool {
        if self.caret == 0 {
            return false;
        }
        self.caret -= 1;
        self.text.remove(self.caret);
        true
    }
}
