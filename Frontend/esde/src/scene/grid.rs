//! A grid's arithmetic: columns, rows shown, where a row scrolls to, as ES-DE 3.4.1 lays it out (EmuSen_BigPicture.md
//! §16.5; LunaP's GridGeometry, §104.1 there).

use crate::loader::{Element, Pair};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GridGeometry {
    pub width: f64,
    pub height: f64,
    pub item: (f64, f64),
    pub spacing: (f64, f64),
    pub scale: f64,
    pub scale_inwards: bool,
    pub fractional_rows: bool,
    pub count: usize,
}

/// A pair in pixels, -1 on one axis taking the other axis's pixels, as itemSize and itemSpacing allow.
pub fn pixels(pair: Option<Pair>, w: f64, h: f64) -> Option<(f64, f64)> {
    let p = pair?;
    let (mut x, mut y) = (p.x as f64 * w, p.y as f64 * h);
    if p.x < 0.0 && p.y >= 0.0 {
        x = y;
    }
    if p.y < 0.0 && p.x >= 0.0 {
        y = x;
    }
    Some((x.max(0.0), y.max(0.0)))
}

impl GridGeometry {
    /// A grid element's layout on a screen, for so many items.
    pub fn of(e: &Element, screen_w: f64, screen_h: f64, count: usize) -> Self {
        let size = e.pair("size").unwrap_or_default();
        let item = pixels(e.pair("itemSize"), screen_w, screen_h).unwrap_or((0.15 * screen_w, 0.25 * screen_h));
        let scale = e.float("itemScale").unwrap_or(1.05) as f64;
        let spacing =
            pixels(e.pair("itemSpacing"), screen_w, screen_h).unwrap_or((item.0 * (scale - 1.0) / 2.0, item.1 * (scale - 1.0) / 2.0));
        GridGeometry {
            width: (size.x.max(0.0) as f64) * screen_w,
            height: (size.y.max(0.0) as f64) * screen_h,
            item,
            spacing,
            scale: scale.max(0.01),
            scale_inwards: e.bool("scaleInwards") == Some(true),
            fractional_rows: e.bool("fractionalRows") == Some(true),
            count,
        }
    }

    pub fn margin(&self) -> (f64, f64) {
        if self.scale_inwards {
            return (0.0, 0.0);
        }
        ((self.item.0 * (self.scale - 1.0) / 2.0).max(0.0), (self.item.1 * (self.scale - 1.0) / 2.0).max(0.0))
    }

    pub fn pitch(&self) -> (f64, f64) {
        (self.item.0 + self.spacing.0, self.item.1 + self.spacing.1)
    }

    pub fn columns(&self) -> usize {
        let (pw, _) = self.pitch();
        if pw <= 0.0 {
            return 1;
        }
        (((self.width + self.spacing.0 - 2.0 * self.margin().0) / pw + 1e-9).floor() as i64).max(1) as usize
    }

    pub fn rows(&self) -> usize {
        self.count.div_ceil(self.columns())
    }

    pub fn visible_rows(&self) -> f64 {
        let (_, ph) = self.pitch();
        if ph <= 0.0 {
            return 1.0;
        }
        ((self.height + self.spacing.1 - 2.0 * self.margin().1) / ph).max(0.0)
    }

    pub fn whole_rows(&self) -> usize {
        ((self.visible_rows() + 1e-9).floor() as usize).max(1)
    }

    pub fn shown_rows(&self) -> f64 {
        if self.fractional_rows {
            self.visible_rows().max(1.0)
        } else {
            self.whole_rows() as f64
        }
    }

    pub fn row_of(&self, index: usize) -> usize {
        index / self.columns()
    }

    pub fn column_of(&self, index: usize) -> usize {
        index % self.columns()
    }

    /// The rows scrolled off the top when an item is selected.
    pub fn scroll_for(&self, index: usize) -> f64 {
        (self.row_of(index) as f64 - (self.shown_rows() - 1.0)).max(0.0)
    }

    /// An item's cell in the grid's box at a scroll: x, y, width, height.
    pub fn cell(&self, index: usize, scroll: f64) -> (f64, f64, f64, f64) {
        let (m, p) = (self.margin(), self.pitch());
        (m.0 + self.column_of(index) as f64 * p.0, m.1 + (self.row_of(index) as f64 - scroll) * p.1, self.item.0, self.item.1)
    }
}
