//! Where in VRAM a stretch of records has drawn or read, kept coarsely enough to be cheap: blocks of 16 by 16 pixels,
//! 64 across and 32 down, each row of blocks one word. And the tiles of 8 by 8 pixels a span covers, each once, which
//! the records are binned into (Diamond_Enhancements.md §16.4).

use crate::record::Span;

/// A set of 16-by-16 blocks of VRAM.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Blocks([u64; 32]);

/// The bits of `count` units from `first`, in a ring of `size` (at most 64): all of them when the count fills it.
fn ring(first: u32, last: u32, size: u32) -> u64 {
    let full = if size == 64 { u64::MAX } else { (1u64 << size) - 1 };
    let count = last - first + 1;
    if count >= size {
        return full;
    }
    let run = (1u64 << count) - 1;
    let at = first % size;
    (run << at | if at + count > size { run >> (size - at) } else { 0 }) & full
}

impl Blocks {
    pub fn clear(&mut self) {
        self.0 = [0; 32];
    }

    pub fn is_empty(&self) -> bool {
        self.0.iter().all(|&w| w == 0)
    }

    /// The block rows a span covers, each once, and its block columns.
    fn rows(s: Span) -> (u64, u64) {
        (ring(s.y0 >> 4, s.y1 >> 4, 32), ring(s.x0 >> 4, s.x1 >> 4, 64))
    }

    pub fn mark(&mut self, s: Span) {
        let (rows, columns) = Blocks::rows(s);
        for (i, w) in self.0.iter_mut().enumerate() {
            if rows >> i & 1 != 0 {
                *w |= columns;
            }
        }
    }

    pub fn meets(&self, s: Span) -> bool {
        let (rows, columns) = Blocks::rows(s);
        self.0.iter().enumerate().any(|(i, w)| rows >> i & 1 != 0 && w & columns != 0)
    }
}

/// Tiles across and down: 1,024 by 512 pixels in tiles of 8.
pub const TILES_ACROSS: u32 = 128;
pub const TILES_DOWN: u32 = 64;
pub const TILES: usize = (TILES_ACROSS * TILES_DOWN) as usize;

/// Every tile a span covers, each once, as `row * 128 + column`.
pub fn tiles(s: Span, mut each: impl FnMut(u32)) {
    let (y0, y1) = (s.y0 >> 3, s.y1 >> 3);
    let (x0, x1) = (s.x0 >> 3, s.x1 >> 3);
    let rows = if y1 - y0 + 1 >= TILES_DOWN { 0..TILES_DOWN } else { y0..y1 + 1 };
    let columns = if x1 - x0 + 1 >= TILES_ACROSS { 0..TILES_ACROSS } else { x0..x1 + 1 };
    for y in rows {
        for x in columns.clone() {
            each((y % TILES_DOWN) * TILES_ACROSS + x % TILES_ACROSS);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_span_past_an_edge_wraps_and_a_whole_axis_is_every_unit_once() {
        assert_eq!(ring(62, 65, 64), 0b11 | 0b11 << 62);
        assert_eq!(ring(3, 3 + 63, 64), u64::MAX);
        assert_eq!(ring(30, 33, 32), 0b11 | 0b11 << 30);
        let mut b = Blocks::default();
        b.mark(Span { x0: 1016, x1: 1031, y0: 500, y1: 520 });
        assert!(b.meets(Span::sized(0, 0, 8, 8)) && b.meets(Span::sized(1020, 510, 1, 1)) && !b.meets(Span::sized(500, 0, 16, 16)));
        let mut seen = Vec::new();
        tiles(Span { x0: 4, x1: 11, y0: 300, y1: 811 }, |t| seen.push(t));
        assert_eq!(seen.len(), 64 * 2, "512 rows are every tile row once, and two columns");
        seen.sort();
        seen.dedup();
        assert_eq!(seen.len(), 128);
    }
}
