//! The two-cycle mode, pipelined so each pixel's first cycles run before its predecessor's last: C#'s `Rdp.TwoCycle.cs`.

use super::depth::{delta_z_encoding, store_depth};
use super::one_cycle::clamp9;
use super::tables::COVERAGE_OFFSETS;
use super::{ATTRIBUTE_S, ATTRIBUTE_T, ATTRIBUTE_W, ATTRIBUTE_Z, ATTRIBUTES, Color, CombinerSelectors, Rdp, RdpMemory, Rows};

#[derive(Clone, Copy)]
struct TwoCyclePixel {
    coverage: i32,
    coverage_bit: bool,
    offset: (u8, u8),
    compare_alpha: i32,
    dither_color: i32,
}

/// What a pixel's first cycles need beyond its values, fixed for the primitive or the row.
#[derive(Clone, Copy)]
struct Setup {
    texel_level: i32,
    lod: bool,
    next_row_drawn: bool,
    length: i32,
    tile: i32,
    max_level: i32,
}

fn reads_alpha(c: &CombinerSelectors, texel: i32) -> bool {
    c.alpha_a == texel || c.alpha_b == texel || c.alpha_c == texel || c.alpha_d == texel
}

fn reads_any(c: &CombinerSelectors, texel: i32) -> bool {
    c.color_a == texel || c.color_b == texel || c.color_c == texel || c.color_d == texel || c.color_c == texel + 7 || reads_alpha(c, texel)
}

impl Rdp {
    pub(super) fn draw_two_cycle(&mut self, rows: Rows, major_on_left: bool, tile: i32, max_level: i32, mem: &mut RdpMemory) {
        let primitive_depth = self.modes.primitive_depth;
        let delta_z = if primitive_depth { self.primitive_delta_z } else { self.depth_slope };
        let delta_z_encoded = delta_z_encoding(delta_z);
        if primitive_depth {
            self.depth_correct_dx = 0;
            self.depth_correct_dy = 0;
        }

        let steps = self.pixel_steps(major_on_left);
        let direction = if major_on_left { 1 } else { -1 };

        let (texel_level, lod) = self.two_cycle_texels();
        let (mut dither_color, mut dither_alpha) = (7, 0);
        let mut values = [0i32; ATTRIBUTES];

        for y in rows.0..=rows.1 {
            let yu = y as usize;
            if !self.span_drawn[yu] || self.span_right[yu] < self.span_left[yu] || !self.owns(y) {
                continue;
            }

            let (left, right) = (self.span_left[yu], self.span_right[yu]);
            self.stamp_row(y, left, right, false, true, true);
            self.row_coverage(y, left, right);

            values.copy_from_slice(&self.span_attributes[yu * ATTRIBUTES..yu * ATTRIBUTES + ATTRIBUTES]);
            if primitive_depth {
                values[ATTRIBUTE_Z] = self.primitive_z;
            }

            let clipped = (if major_on_left { left - self.span_major_x[yu] } else { self.span_major_x[yu] - right }) & 0xFFF;
            for c in 0..ATTRIBUTES {
                values[c] = values[c].wrapping_add(steps[c].wrapping_mul(clipped));
            }

            let last = right - left;
            let setup = Setup {
                texel_level,
                lod,
                next_row_drawn: y < rows.1 && self.span_drawn[yu + 1],
                length: last + clipped,
                tile,
                max_level,
            };

            let mut x = if major_on_left { left } else { right };
            let mask = self.coverage[x as usize];
            let mut current = self.first_cycles(&values, &steps, x, y, mask, false, setup, &mut dither_color, &mut dither_alpha);

            for n in 0..=last {
                let mut coverage = current.coverage;
                let z = self.correct_depth((values[ATTRIBUTE_Z] >> 10) & 0x3F_FFFF, current.offset, coverage);

                std::mem::swap(&mut self.texel0, &mut self.texel1);
                self.combine_second_cycle(dither_alpha, &mut coverage);

                let pixel = y.wrapping_mul(self.color_image_width).wrapping_add(x);
                let nothing = RdpMemory::nothing();
                let seen = if self.blind(x, y) { &nothing } else { &*mem };
                let memory_coverage = self.read_memory(pixel, seen);
                let depth_index = (self.depth_image >> 1).wrapping_add(pixel as u32);

                let (mut blend, mut overflow) = (false, false);
                let write = self.compare_depth(depth_index, z, delta_z, delta_z_encoded, memory_coverage, &mut coverage, &mut blend, &mut overflow, seen)
                    && (if self.modes.antialias { coverage != 0 } else { current.coverage_bit });

                if write {
                    let (br, bg, bb) = self.blend_equation(self.modes.first_blend_cycle, self.pixel, false, self.past_shift_a, self.past_shift_b);
                    self.blended = Color { r: br, g: bg, b: bb, a: 0 };
                    self.split.blended = self.split.row_stamp;
                }

                for c in 0..ATTRIBUTES {
                    values[c] = values[c].wrapping_add(steps[c]);
                }
                let next_mask = if n < last { self.coverage[(x + direction) as usize] } else { 0 };
                let next = self.first_cycles(&values, &steps, x + direction, y, next_mask, n == last, setup, &mut dither_color, &mut dither_alpha);

                if write && (!self.modes.alpha_compare || current.compare_alpha >= self.alpha_threshold()) {
                    let (r, g, b) = self.blend(
                        self.modes.second_blend_cycle,
                        self.blended,
                        current.dither_color,
                        blend,
                        overflow,
                        self.blend_shift_a,
                        self.blend_shift_b,
                    );
                    self.write_memory(pixel, r, g, b, blend, coverage, memory_coverage, mem);
                    if self.modes.depth_update {
                        store_depth(depth_index, z, delta_z_encoded, mem);
                    }
                }

                current = next;
                x += direction;
            }
        }
    }

    /// Coverage, shade, dither, texels and the first combiner cycle for a pixel, or for the one past a row's end.
    #[allow(clippy::too_many_arguments)]
    #[inline(always)]
    fn first_cycles(
        &mut self,
        values: &[i32; ATTRIBUTES],
        steps: &[i32; ATTRIBUTES],
        x: i32,
        y: i32,
        mask: u8,
        beyond: bool,
        setup: Setup,
        dither_color: &mut i32,
        dither_alpha: &mut i32,
    ) -> TwoCyclePixel {
        let coverage = mask.count_ones() as i32;
        let offset = COVERAGE_OFFSETS[mask as usize];

        self.shade = self.shaded(values, offset, coverage);

        if self.modes.dither_table != 0xF {
            self.dither(x, y, dither_color, dither_alpha);
        }

        let (s, t, w) = (values[ATTRIBUTE_S], values[ATTRIBUTE_T], values[ATTRIBUTE_W]);
        let (ds, dt, dw) = (steps[ATTRIBUTE_S], steps[ATTRIBUTE_T], steps[ATTRIBUTE_W]);
        let (tile, max_level) = (setup.tile, setup.max_level);
        let bilinear_first = self.modes.bilinear_first_cycle;

        if setup.texel_level <= 1 {
            if beyond && setup.texel_level == 0 && setup.next_row_drawn && setup.length >= 3 {
                let (first, second, fraction) =
                    if setup.lod { self.next_row_level_of_detail(y + 1, ds, dt, dw, tile, max_level) } else { (tile, tile, self.lod_fraction) };
                self.lod_fraction = fraction;
                if setup.lod {
                    self.split.lod = self.split.row_stamp;
                }

                let (cs, ct) = self.texture_coordinates(s, t, w);
                self.texel0 = self.texel(cs, ct, first, bilinear_first, false, Color::default());

                let (rs, rt, rw) = self.row_start(y + 1);
                let (ns, nt) = self.texture_coordinates(rs, rt, rw);
                self.texel1 = self.texel(ns, nt, second, bilinear_first, false, self.texel0);
            } else {
                let (first, second, fraction) = if setup.lod {
                    self.two_cycle_level_of_detail(s, t, w, ds, dt, dw, tile, max_level)
                } else {
                    (tile, (tile + 1) & 7, self.lod_fraction)
                };
                self.lod_fraction = fraction;
                if setup.lod {
                    self.split.lod = self.split.row_stamp;
                }

                let (cs, ct) = self.texture_coordinates(s, t, w);
                self.texel0 = self.texel(cs, ct, first, bilinear_first, false, Color::default());
                self.texel1 = self.texel(cs, ct, second, self.modes.bilinear_second_cycle, self.modes.convert_one, self.texel0);
            }
        } else if setup.texel_level == 2 {
            let (first, _, fraction) =
                if setup.lod { self.two_cycle_level_of_detail(s, t, w, ds, dt, dw, tile, max_level) } else { (tile, 0, self.lod_fraction) };
            self.lod_fraction = fraction;
            if setup.lod {
                self.split.lod = self.split.row_stamp;
            }

            let (cs, ct) = self.texture_coordinates(s, t, w);
            self.texel0 = self.texel(cs, ct, first, bilinear_first, false, Color::default());
        }

        let compare_alpha = self.combine_first_cycle(*dither_alpha, coverage);
        TwoCyclePixel { coverage, coverage_bit: (mask & 0x80) != 0, offset, compare_alpha, dither_color: *dither_color }
    }

    /// The first combiner cycle: the combined colour the second reads, and the alpha the alpha compare tests.
    #[inline(always)]
    fn combine_first_cycle(&mut self, dither_alpha: i32, coverage: i32) -> i32 {
        let m = &self.modes;
        let (red, green, blue, alpha) = self.combiner_equations(m.first_combine_cycle);

        let mut compare_alpha = 0;
        if m.alpha_compare {
            compare_alpha = clamp9(alpha);
            if compare_alpha == 0xFF {
                compare_alpha = 0x100;
            }

            if !m.alpha_from_coverage {
                compare_alpha += dither_alpha;
                if (compare_alpha & 0x100) != 0 {
                    compare_alpha = 0xFF;
                }
            } else {
                compare_alpha = if m.coverage_times_alpha { (compare_alpha * coverage + 4) >> 3 } else { coverage << 5 };
                if compare_alpha > 0xFF {
                    compare_alpha = 0xFF;
                }
            }
        }

        self.combined = Color { r: red >> 8, g: green >> 8, b: blue >> 8, a: alpha };

        self.blender_shade_alpha = self.shade.a + dither_alpha;
        if (self.blender_shade_alpha & 0x100) != 0 {
            self.blender_shade_alpha = 0xFF;
        }

        compare_alpha
    }

    /// Both texels and the second's successor (0), both (1), the first alone (2), or neither (3); and whether a level is measured.
    fn two_cycle_texels(&self) -> (i32, bool) {
        let m = &self.modes;
        let (first, second) = (&m.first_combine_cycle, &m.second_combine_cycle);

        let fraction_first = first.color_c == 13 || first.alpha_c == 0;
        let fraction_second = second.color_c == 13 || second.alpha_c == 0;
        let lod = m.lod_enabled || fraction_first || fraction_second;

        let level = if reads_any(second, 2) || m.alpha_compare && (reads_alpha(first, 1) || reads_alpha(first, 2) || first.alpha_c == 0) {
            0
        } else if reads_any(first, 2) || reads_any(second, 1) {
            1
        } else if reads_any(first, 1) || fraction_first || fraction_second {
            2
        } else {
            3
        };
        (level, lod)
    }
}
