//! A primitive's two edges walked in quarter-pixel sub-scanlines into one span per row: C#'s `Rdp.Walker.cs`.

use super::depth::normalize_delta_z;
use super::{ATTRIBUTE_S, ATTRIBUTE_Z, ATTRIBUTES, COPY_CYCLE, Color, Rdp, Rows, sign_extend};

/// The edges compared for crossing at quarter-pixel precision, as signed twelve-bit whole parts.
#[inline(always)]
fn quarter_pixel(x: i32) -> i32 {
    (x ^ (1 << 27)) & (0x3FFF << 14)
}

impl Rdp {
    /// The rows it returns are the only ones whose spans it wrote.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn walk(
        &mut self,
        major_on_left: bool,
        yh: i32,
        ym: i32,
        yl: i32,
        xh: i32,
        xm: i32,
        xl: i32,
        dxhdy: i32,
        dxmdy: i32,
        dxldy: i32,
        major_slope_negative: bool,
    ) -> Rows {
        self.combined = Color::default();
        self.prepare_attributes();

        let lean_offset = major_slope_negative == major_on_left;
        let sample_sub = if lean_offset { 3 } else { 0 };
        let mut offsets = [0i32; ATTRIBUTES];
        let mut running = [0i32; ATTRIBUTES];
        for c in 0..ATTRIBUTES {
            let along_edge = self.attribute_de[c] & !0x1FF;
            let down = self.attribute_dy[c] & !0x1FF;
            offsets[c] = if lean_offset { along_edge.wrapping_sub(along_edge >> 2).wrapping_sub(down).wrapping_add(down >> 2) } else { 0 };
            running[c] = self.attribute_value[c];
        }

        let upper = self.upper_limit(yh);
        let lower = self.lower_limit(yl);
        let first = upper >> 2;
        let last = lower >> 2;
        if first > last {
            return (first, last);
        }

        self.span_drawn[first as usize..=last as usize].fill(false);

        let far = lower | 3;
        let close = upper & !3;

        let mut major = xh & !1;
        let mut minor = xm & !1;
        let major_step = (dxhdy >> 2) & !1;
        let mut minor_step = (dxmdy >> 2) & !1;

        let copy = self.modes.cycle_type == COPY_CYCLE;
        let mut per_half_pixel = [0i32; ATTRIBUTES];
        if !copy {
            for (half, dx) in per_half_pixel.iter_mut().zip(self.attribute_dx) {
                *half = (dx >> 8) & !1;
            }
        }

        let (mut left, mut right) = (0, 0);
        let (mut over, mut under, mut outside) = (true, true, true);

        for k in (yh & !3)..=far {
            if k == ym {
                minor = xl & !1;
                minor_step = (dxldy >> 2) & !1;
            }

            if k >= close {
                if (k & 3) == 0 {
                    left = 0xFFF;
                    right = 0;
                    over = true;
                    under = true;
                    outside = true;
                }

                let left_edge = if major_on_left { major } else { minor };
                let right_edge = if major_on_left { minor } else { major };

                let (left_at, left_under, left_over) = self.clip_edge(left_edge);
                let (right_at, right_under, right_over) = self.clip_edge(right_edge);
                over &= left_over && right_over;
                under &= left_under && right_under;

                let crossed = quarter_pixel(right_edge) < quarter_pixel(left_edge);
                let invalid = k < upper || k >= lower || crossed;
                outside &= invalid;

                let ku = k as usize;
                self.edge_left[ku] = left_at & 0x1FFF;
                self.edge_right[ku] = right_at & 0x1FFF;
                self.edge_invalid[ku] = invalid;

                if !invalid {
                    left = left.min(left_at >> 3);
                    right = right.max(right_at >> 3);
                }

                if (k & 3) == sample_sub {
                    let row = (k >> 2) as usize;
                    self.span_major_x[row] = sign_extend((major >> 16) as u32, 12);
                    let fraction = (major >> 8) & 0xFF;
                    let at = row * ATTRIBUTES;
                    for c in 0..ATTRIBUTES {
                        self.span_attributes[at + c] =
                            ((running[c] & !0x1FF).wrapping_add(offsets[c]).wrapping_sub(fraction.wrapping_mul(per_half_pixel[c]))) & !0x3FF;
                    }
                }

                if (k & 3) == 3 {
                    let row = k >> 2;
                    let ru = row as usize;
                    self.span_drawn[ru] = !outside && !over && !under && self.field_keeps(row);
                    self.span_left[ru] = left;
                    self.span_right[ru] = right;
                }
            }

            if (k & 3) == 3 {
                for (value, de) in running.iter_mut().zip(self.attribute_de) {
                    *value = value.wrapping_add(de);
                }
            }

            major = major.wrapping_add(major_step);
            minor = minor.wrapping_add(minor_step);
        }

        (first, last)
    }

    /// A sixteen-fraction-bit x taken to eighth pixels, moved onto the scissor's left edge if under it, then its right if at or past it.
    #[inline(always)]
    fn clip_edge(&self, x: i32) -> (i32, bool, bool) {
        let sticky = if ((x >> 1) & 0x1FFF) != 0 { 1 } else { 0 };
        let clip_left = self.scissor_left * 2;
        let clip_right = self.scissor_right * 2;

        let at = ((x >> 13) & 0x1FFE) | sticky;
        let under = (x & 0x0800_0000) != 0 || (at < clip_left && (x & 0x0400_0000) == 0);

        let at = if under { clip_left } else { ((x >> 13) & 0x3FFE) | sticky };
        let over = (at & 0x2000) != 0 || (at & 0x1FFF) >= clip_right;

        (if over { clip_right } else { at }, under, over)
    }

    /// Negative tops start at the scissor and tops past the last row stand; otherwise the lower of the two wins.
    fn upper_limit(&self, yh: i32) -> i32 {
        if (yh & 0x2000) != 0 {
            self.scissor_top
        } else if (yh & 0x1000) != 0 {
            yh
        } else {
            yh.max(self.scissor_top)
        }
    }

    fn lower_limit(&self, yl: i32) -> i32 {
        if (yl & 0x2000) != 0 {
            yl
        } else if (yl & 0x1000) != 0 {
            self.scissor_bottom
        } else {
            yl.min(self.scissor_bottom)
        }
    }

    /// The derived steps, with the depth slope summed from the two derivatives' magnitudes.
    fn prepare_attributes(&mut self) {
        for c in 0..4 {
            self.shade_step[c] = self.attribute_dx[c] & !0x1F;
            self.shade_correct_dx[c] = sign_extend((self.shade_step[c] >> 14) as u32, 13);
            self.shade_correct_dy[c] = sign_extend((self.attribute_dy[c] >> 14) as u32, 13);
        }

        self.depth_step = self.attribute_dx[ATTRIBUTE_Z];
        self.depth_correct_dx = sign_extend((self.depth_step >> 10) as u32, 22);
        self.depth_correct_dy = sign_extend((self.attribute_dy[ATTRIBUTE_Z] >> 10) as u32, 22);

        for c in 0..3 {
            self.texture_step[c] = self.attribute_dx[ATTRIBUTE_S + c] & !0x1F;
        }

        let down = (self.attribute_dy[ATTRIBUTE_Z] >> 16) & 0xFFFF;
        let across = (self.attribute_dx[ATTRIBUTE_Z] >> 16) & 0xFFFF;
        let magnitude = (if (down & 0x8000) != 0 { !down & 0x7FFF } else { down }) + (if (across & 0x8000) != 0 { !across & 0x7FFF } else { across });
        self.depth_slope = normalize_delta_z(magnitude & 0xFFFF) & 0xFFFF;
    }

    #[inline(always)]
    fn field_keeps(&self, row: i32) -> bool {
        !self.scissor_field || ((row & 1) == 1) == self.scissor_keep_odd
    }
}
