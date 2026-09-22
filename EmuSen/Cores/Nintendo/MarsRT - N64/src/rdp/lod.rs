//! The level of detail: how far a pixel's coordinates move, the tile that picks and the fraction the combiner reads: C#'s `Rdp.Lod.cs`.

use super::tables::LOG2;
use super::{ATTRIBUTE_S, ATTRIBUTE_T, ATTRIBUTE_W, ATTRIBUTES, Rdp, sign_extend};

/// Movement past fourteen bits keeps its low fifteen with bit 14 set.
#[inline(always)]
fn saturated(movement: i32) -> i32 {
    (movement & 0x7FFF) | if (movement & 0x1C000) != 0 { 0x4000 } else { 0 }
}

/// Seventeen-bit coordinates subtracted, a negative difference taken as its complement.
#[inline(always)]
fn movement(from: i32, to: i32) -> i32 {
    let difference = sign_extend(to as u32, 17) - sign_extend(from as u32, 17);
    if (difference & 0x20000) != 0 { !difference & 0x1FFFF } else { difference }
}

impl Rdp {
    /// The next pixel's coordinates against the one after, or near a span's end the one before, or the next row's.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn pixel_level_of_detail(
        &self,
        s: i32,
        t: i32,
        w: i32,
        ds: i32,
        dt: i32,
        dw: i32,
        next_row: i32,
        next_row_drawn: bool,
        end: bool,
        before_end: bool,
        long_span: bool,
        mid_span: bool,
        tile: i32,
        max_level: i32,
    ) -> (i32, i32) {
        if next_row_drawn && end && long_span {
            let (rs, rt, rw) = self.row_start(next_row);
            return self.level_of_detail(rs, rt, rw, rs.wrapping_add(ds), rt.wrapping_add(dt), rw.wrapping_add(dw), tile, max_level);
        }

        let (ns, nt, nw) = (s.wrapping_add(ds), t.wrapping_add(dt), w.wrapping_add(dw));
        if next_row_drawn && (before_end && long_span || end && mid_span) {
            self.level_of_detail(ns, nt, nw, s.wrapping_sub(ds), t.wrapping_sub(dt), w.wrapping_sub(dw), tile, max_level)
        } else {
            self.level_of_detail(ns, nt, nw, s.wrapping_add(ds << 1), t.wrapping_add(dt << 1), w.wrapping_add(dw << 1), tile, max_level)
        }
    }

    /// The tile of a span's last texel 1, measured from the pixel after the span by the span's length.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn after_span_tile(
        &self,
        s: i32,
        t: i32,
        w: i32,
        ds: i32,
        dt: i32,
        dw: i32,
        next_row: i32,
        next_row_drawn: bool,
        long_span: bool,
        mid_span: bool,
        one_before_mid: bool,
        tile: i32,
        max_level: i32,
    ) -> i32 {
        if next_row_drawn && (long_span || mid_span) {
            let (mut rs, mut rt, mut rw) = self.row_start(next_row);
            if long_span {
                (rs, rt, rw) = (rs.wrapping_add(ds), rt.wrapping_add(dt), rw.wrapping_add(dw));
            }
            return self.level_of_detail(rs, rt, rw, rs.wrapping_add(ds), rt.wrapping_add(dt), rw.wrapping_add(dw), tile, max_level).0;
        }

        let (ns, nt, nw) = (s.wrapping_add(ds), t.wrapping_add(dt), w.wrapping_add(dw));
        if next_row_drawn && one_before_mid {
            self.level_of_detail(ns, nt, nw, s.wrapping_sub(ds), t.wrapping_sub(dt), w.wrapping_sub(dw), tile, max_level).0
        } else {
            self.level_of_detail(ns, nt, nw, s.wrapping_add(ds << 1), t.wrapping_add(dt << 1), w.wrapping_add(dw << 1), tile, max_level).0
        }
    }

    #[inline(always)]
    pub(super) fn row_start(&self, row: i32) -> (i32, i32, i32) {
        let at = row as usize * ATTRIBUTES;
        (self.span_attributes[at + ATTRIBUTE_S], self.span_attributes[at + ATTRIBUTE_T], self.span_attributes[at + ATTRIBUTE_W])
    }

    /// The larger of s's and t's movement, from coordinates that did not overflow the divider, gives the tile and fraction.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn level_of_detail(
        &self,
        near_s: i32,
        near_t: i32,
        near_w: i32,
        far_s: i32,
        far_t: i32,
        far_w: i32,
        tile: i32,
        max_level: i32,
    ) -> (i32, i32) {
        let (ns, nt) = self.divided_coordinates(near_s, near_t, near_w);
        let (fs, ft) = self.divided_coordinates(far_s, far_t, far_w);

        let overflow = ((ns | nt | fs | ft) & 0x60000) != 0;
        let lod = if overflow { 0 } else { saturated(movement(ns, fs).max(movement(nt, ft))) };

        let (mut level, magnify, distant, fraction) = self.level_signals(lod, overflow, max_level);
        if !self.modes.lod_enabled {
            return (tile, fraction);
        }

        if distant {
            level = max_level;
        }
        ((tile + level + if self.modes.detail_enabled && !magnify { 1 } else { 0 }) & 7, fraction)
    }

    /// Saturated, magnified or minified, with the level, whether the pixel is distant, and the fraction each gives.
    fn level_signals(&self, lod: i32, overflow: bool, max_level: i32) -> (i32, bool, bool, i32) {
        let m = &self.modes;
        let plain = !m.sharpen_enabled && !m.detail_enabled;

        if (lod & 0x4000) != 0 || overflow {
            return (0, false, true, 0xFF);
        }

        if lod < 32 {
            let magnified_distant = max_level == 0;
            let magnified = if plain {
                if magnified_distant { 0xFF } else { 0 }
            } else {
                (lod.max(self.min_level) << 3) | if m.sharpen_enabled { 0x100 } else { 0 }
            };
            return (0, true, magnified_distant, magnified);
        }

        let level = LOG2[((lod >> 5) & 0xFF) as usize] as i32;
        let distant = max_level == 0 || (lod & 0x6000) != 0 || level >= max_level;
        (level, false, distant, if plain && distant { 0xFF } else { ((lod << 3) >> level) & 0xFF })
    }

    /// The two-cycle mode measures from the pixel itself, across to the next pixel and down to the next row, for two tiles.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn two_cycle_level_of_detail(&self, s: i32, t: i32, w: i32, ds: i32, dt: i32, dw: i32, tile: i32, max_level: i32) -> (i32, i32, i32) {
        let (cs, ct) = self.divided_coordinates(s, t, w);
        let (xs, xt) = self.divided_coordinates(s.wrapping_add(ds), t.wrapping_add(dt), w.wrapping_add(dw));
        let (ys, yt) = self.divided_coordinates(
            s.wrapping_add(self.down_step(ATTRIBUTE_S)),
            t.wrapping_add(self.down_step(ATTRIBUTE_T)),
            w.wrapping_add(self.down_step(ATTRIBUTE_W)),
        );

        let overflow = ((cs | ct | xs | xt | ys | yt) & 0x60000) != 0;
        let mut lod = 0;
        if !overflow {
            lod = saturated(movement(cs, xs).max(movement(ct, xt)));
            lod = saturated(lod.max(movement(cs, ys).max(movement(ct, yt))));
        }

        let (mut level, magnify, distant, fraction) = self.level_signals(lod, overflow, max_level);
        let m = &self.modes;
        if !m.lod_enabled {
            return (tile, (tile + 1) & 7, fraction);
        }

        if distant {
            level = max_level;
        }
        if !m.detail_enabled {
            let first = (tile + level) & 7;
            return (first, if distant || !m.sharpen_enabled && magnify { first } else { (first + 1) & 7 }, fraction);
        }

        ((tile + level + if magnify { 0 } else { 1 }) & 7, (tile + level + if !distant && !magnify { 2 } else { 1 }) & 7, fraction)
    }

    /// Past a row's end, the next row's first pixel is measured down alone for the fraction and first tile, and across too for the second.
    pub(super) fn next_row_level_of_detail(&self, next_row: i32, ds: i32, dt: i32, dw: i32, tile: i32, max_level: i32) -> (i32, i32, i32) {
        let (rs, rt, rw) = self.row_start(next_row);
        let (cs, ct) = self.divided_coordinates(rs, rt, rw);
        let (ys, yt) = self.divided_coordinates(
            rs.wrapping_add(self.down_step(ATTRIBUTE_S)),
            rt.wrapping_add(self.down_step(ATTRIBUTE_T)),
            rw.wrapping_add(self.down_step(ATTRIBUTE_W)),
        );

        let mut overflow = ((cs | ct | ys | yt) & 0x60000) != 0;
        let mut lod = if overflow { 0 } else { saturated(movement(cs, ys).max(movement(ct, yt))) };

        let (level, magnify, distant, fraction) = self.level_signals(lod, overflow, max_level);
        let m = &self.modes;
        if !m.lod_enabled {
            return (tile, tile, fraction);
        }

        let detail = |magnify: bool| if m.detail_enabled && !magnify { 1 } else { 0 };
        let first = (tile + (if distant { max_level } else { level }) + detail(magnify)) & 7;

        let (xs, xt) = self.divided_coordinates(rs.wrapping_add(ds), rt.wrapping_add(dt), rw.wrapping_add(dw));
        overflow |= ((xs | xt) & 0x60000) != 0;
        if !overflow {
            lod = saturated(lod.max(movement(cs, xs).max(movement(ct, xt))));
        }

        let (level, magnify, distant, _) = self.level_signals(lod, overflow, max_level);
        (first, (tile + (if distant { max_level } else { level }) + detail(magnify)) & 7, fraction)
    }

    /// A texture coordinate's step down one row, with its low fifteen bits cleared.
    #[inline(always)]
    fn down_step(&self, attribute: usize) -> i32 {
        self.attribute_dy[attribute] & !0x7FFF
    }
}
