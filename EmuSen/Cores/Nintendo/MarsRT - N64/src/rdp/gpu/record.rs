//! A processor at the multiple that walks and hands its rows to a device instead of shading them: C#'s `Rdp.Gpu.cs`, the record
//! layouts `shade.comp`'s. See Mars_Gpu.md §5 and §6.

use super::{Declined, GpuRasteriser};
use crate::rdp::depth::delta_z_encoding;
use crate::rdp::{ATTRIBUTE_S, ATTRIBUTE_Z, ATTRIBUTES, COPY_CYCLE, Color, CombinerSelectors, FILL_CYCLE, ONE_CYCLE, Rdp, Rows, TWO_CYCLE};

#[inline(always)]
fn bit(at: u32, set: bool) -> u32 {
    if set { 1 << at } else { 0 }
}

fn put(record: &mut [u32], at: usize, color: Color) {
    record[at] = color.r as u32;
    record[at + 1] = color.g as u32;
    record[at + 2] = color.b as u32;
    record[at + 3] = color.a as u32;
}

/// A cycle reading the combiner's own previous result is the carry an invocation per pixel cannot see (Mars_Gpu.md §6.2).
fn reads_combined(c: &CombinerSelectors) -> bool {
    c.color_a == 0 || c.color_b == 0 || c.color_d == 0 || c.color_c == 0 || c.color_c == 7 || c.alpha_a == 0 || c.alpha_b == 0 || c.alpha_d == 0
}

fn packed_combiner(c: &CombinerSelectors) -> u32 {
    (c.color_a | (c.color_b << 4) | (c.color_c << 8) | ((c.color_d & 7) << 13) | (c.alpha_a << 16) | (c.alpha_b << 19) | (c.alpha_c << 22) | (c.alpha_d << 25)) as u32
}

impl Rdp {
    /// `RecordForTheDevice`: the primitive walked here, its rows written down for the device instead of shaded.
    pub(crate) fn record_for_the_device(&mut self, gpu: &mut GpuRasteriser, rows: Rows, major_on_left: bool, tile: i32, max_level: i32) {
        let cycle = self.modes.cycle_type;
        // A thirty-two-bit image in copy mode draws nothing on the CPU path either, so there is nothing to decline (Mars_Rdp.md §11).
        if cycle == COPY_CYCLE && self.color_image_size == 3 {
            return;
        }
        if cycle != FILL_CYCLE && cycle != COPY_CYCLE && !self.the_device_shades_this_primitive() {
            gpu.not_shaded(Declined::Carry);
            return;
        }
        gpu.image(self.color_image & !((self.color_image_bytes - 1).max(0) as u32), self.color_image_width, if self.color_image_bytes == 1 { 0 } else { self.color_image_bytes });
        gpu.depth_image(self.depth_image);
        if !gpu.shades() {
            gpu.not_shaded(Declined::Image);
            return;
        }

        if cycle == FILL_CYCLE {
            let (filled, fill) = gpu.primitive();
            fill[0] = 3;
            fill[1] = self.fill_color;
            for y in rows.0..=rows.1 {
                let yu = y as usize;
                if self.span_drawn[yu] {
                    let (left, right) = (self.span_left[yu], self.span_right[yu]);
                    gpu.row(filled, y, left, right, (right - left.max(self.color_image_width) + 1).max(0));
                }
            }
            return;
        }
        self.record_shaded(gpu, rows, major_on_left, tile, max_level);
    }

    /// In the two-cycle mode the second cycle's COMBINED is this pixel's first cycle and is fine; the first cycle's is the pixel before.
    /// The first blend weighing memory alpha carries the previous pixel's depth slope (Mars_Gpu.md §10.1).
    fn the_device_shades_this_primitive(&self) -> bool {
        let m = &self.modes;
        if m.cycle_type == ONE_CYCLE {
            !reads_combined(&m.second_combine_cycle)
        } else {
            !reads_combined(&m.first_combine_cycle) && m.first_blend_cycle.second_alpha != 1 && !m.convert_one
        }
    }

    /// `RecordTiles`: the eight tiles as the shader reads them, four words each (Mars_Gpu.md §7.1).
    fn record_tiles(&self, into: &mut [u32]) {
        for (i, tile) in self.tiles.iter().enumerate() {
            into[i * 4] = (tile.format | (tile.size << 3) | (tile.line << 5) | (tile.memory << 14) | (tile.palette << 23)) as u32;
            into[i * 4 + 1] = bit(0, tile.clamp_s)
                | bit(1, tile.mirror_s)
                | bit(2, tile.clamp_t)
                | bit(3, tile.mirror_t)
                | ((tile.mask_s as u32) << 4)
                | ((tile.shift_s as u32) << 8)
                | ((tile.mask_t as u32) << 12)
                | ((tile.shift_t as u32) << 16);
            into[i * 4 + 2] = (tile.sl | (tile.tl << 12)) as u32;
            into[i * 4 + 3] = (tile.sh | (tile.th << 12)) as u32;
        }
    }

    /// `RecordShaded`: `DrawOneCycle`'s setup, written down instead of run; the layout is `shade.comp`'s (Mars_Gpu.md §6).
    fn record_shaded(&mut self, gpu: &mut GpuRasteriser, rows: Rows, major_on_left: bool, tile: i32, max_level: i32) {
        let m = self.modes;
        let delta_z = if m.primitive_depth { self.primitive_delta_z } else { self.depth_slope };
        if m.primitive_depth {
            self.depth_correct_dx = 0;
            self.depth_correct_dy = 0;
        }
        let direction: i32 = if major_on_left { 1 } else { -1 };
        let (two_cycle, copy) = (m.cycle_type == TWO_CYCLE, m.cycle_type == COPY_CYCLE);
        let mut steps = [0i32; ATTRIBUTES];
        for (step, &shade) in steps.iter_mut().zip(&self.shade_step) {
            *step = direction.wrapping_mul(shade);
        }
        steps[ATTRIBUTE_Z] = if m.primitive_depth { 0 } else { direction.wrapping_mul(self.depth_step) };
        for c in 0..3 {
            steps[ATTRIBUTE_S + c] = direction.wrapping_mul(if copy { self.copy_pixel_step(c) } else { self.texture_step[c] });
        }

        let (texel0, texel1) = self.combiner_texels();
        let reads_lod_fraction = m.second_combine_cycle.color_c == 13 || m.second_combine_cycle.alpha_c == 0;
        let memory = gpu.texture_memory(&self.texture_memory[..], self.multiple.texture_memory_changed);
        let (tile_set, packed) = gpu.tiles(self.multiple.tiles_changed);
        if let Some(packed) = packed {
            let mut words = [0u32; super::TILE_WORDS];
            self.record_tiles(&mut words);
            packed.copy_from_slice(&words);
        }
        self.multiple.texture_memory_changed = false;
        self.multiple.tiles_changed = false;
        let texel_level = if two_cycle { self.two_cycle_texels().0 } else { 3 };

        let (primitive, p) = gpu.primitive();
        p[0] = if copy {
            2
        } else if two_cycle {
            1
        } else {
            0
        };
        p[2] = bit(0, m.key_enabled)
            | bit(1, m.coverage_times_alpha)
            | bit(2, m.alpha_from_coverage)
            | bit(3, m.alpha_compare)
            | bit(4, m.dither_alpha)
            | bit(5, m.antialias)
            | bit(6, m.color_on_coverage)
            | bit(7, m.force_blend)
            | bit(8, m.image_read)
            | bit(9, m.depth_update)
            | bit(10, m.depth_compare)
            | bit(11, m.primitive_depth)
            | bit(12, self.scissor_field)
            | bit(13, major_on_left)
            | bit(14, ((m.rgb_dither << 2) | m.alpha_dither) != 0xF)
            | bit(15, (if two_cycle { m.second_blend_cycle.second_alpha } else { m.first_blend_cycle.second_alpha }) == 1);
        p[3] = (m.rgb_dither | (m.alpha_dither << 2) | (m.depth_mode << 4) | (m.coverage_destination << 6) | (self.color_image_size << 8) | (self.color_image_format << 10)) as u32;
        p[4] = packed_combiner(&m.second_combine_cycle);
        let blend = m.first_blend_cycle;
        p[5] = (blend.first_color | (blend.first_alpha << 2) | (blend.second_color << 4) | (blend.second_alpha << 6)) as u32;

        put(p, 6, self.primitive_color);
        put(p, 10, self.environment_color);
        put(p, 14, self.blend_color);
        put(p, 18, self.fog_color);
        put(p, 22, self.key_center);
        put(p, 26, self.key_scale);
        put(p, 30, self.key_width);
        p[33] = self.k4 as u32;
        p[34] = self.k5 as u32;
        p[35] = self.primitive_lod_fraction as u32;
        p[36] = self.primitive_z as u32;
        p[37] = delta_z as u32;
        p[38] = delta_z_encoding(delta_z) as u32;
        for c in 0..ATTRIBUTES {
            p[40 + c] = steps[c] as u32;
        }
        for c in 0..4 {
            p[48 + c] = self.shade_correct_dx[c] as u32;
            p[52 + c] = self.shade_correct_dy[c] as u32;
        }
        p[56] = self.depth_correct_dx as u32;
        p[57] = self.depth_correct_dy as u32;
        p[39] = bit(0, texel0)
            | bit(1, texel1)
            | bit(2, m.perspective)
            | bit(3, m.sample_four)
            | bit(4, m.palette_enabled)
            | bit(5, m.palette_intensity_alpha)
            | bit(6, m.mid_texel)
            | bit(7, m.bilinear_first_cycle)
            | bit(8, m.detail_enabled)
            | bit(9, m.sharpen_enabled)
            | bit(10, m.lod_enabled)
            | bit(11, m.convert_one)
            | bit(12, reads_lod_fraction)
            | bit(13, m.bilinear_second_cycle);
        p[58] = memory;
        p[59] = tile_set;
        p[60] = tile as u32;
        p[61] = max_level as u32;
        p[62] = self.min_level as u32;
        p[68] = packed_combiner(&m.first_combine_cycle);
        let blend2 = m.second_blend_cycle;
        p[69] = (blend2.first_color | (blend2.first_alpha << 2) | (blend2.second_color << 4) | (blend2.second_alpha << 6)) as u32;
        p[70] = texel_level as u32;
        for c in 0..3 {
            p[72 + c] = (self.attribute_dy[ATTRIBUTE_S + c] & !0x7FFF) as u32;
        }
        p[64] = self.k0 as u32;
        p[65] = self.k1 as u32;
        p[66] = self.k2 as u32;
        p[67] = self.k3 as u32;

        for y in rows.0..=rows.1 {
            let yu = y as usize;
            if !self.span_drawn[yu] || self.span_right[yu] < self.span_left[yu] {
                continue;
            }
            let (left, right) = (self.span_left[yu], self.span_right[yu]);
            let covered = self.covered_past_the_width(y, left, right);
            let Some(row) = gpu.row(primitive, y, left, right, covered) else { continue };

            // The row's values at its first pixel, as DrawOneCycle steps them there from the major edge.
            let clipped = if major_on_left { left - self.span_major_x[yu] } else { self.span_major_x[yu] - right };
            let clipped = if self.multiple.scaled { clipped } else { clipped & 0xFFF };
            // The copy mode at a multiple starts each row from its major edge's values, unstepped (Mars_Gpu.md §11.3).
            let offset = if copy { 0 } else { clipped };
            for c in 0..ATTRIBUTES {
                row[4 + c] = self.span_attributes[yu * ATTRIBUTES + c].wrapping_add(steps[c].wrapping_mul(offset)) as u32;
            }
            let length = (right - left) + clipped;
            let next_row_drawn = y < rows.1 && self.span_drawn[yu + 1];
            row[24] = bit(0, next_row_drawn) | bit(1, length > 7) | bit(2, length == 7) | bit(3, length == 6) | bit(4, length >= 3);
            if next_row_drawn {
                for c in 0..3 {
                    row[21 + c] = self.span_attributes[(yu + 1) * ATTRIBUTES + ATTRIBUTE_S + c] as u32;
                }
            }
            let mut invalid = 0u32;
            for sub in 0..4 {
                let k = yu * 4 + sub;
                row[12 + sub] = self.edge_left[k] as u32;
                row[16 + sub] = self.edge_right[k] as u32;
                if self.edge_invalid[k] {
                    invalid |= 1 << sub;
                }
            }
            row[20] = invalid;
        }
    }

    /// `CoveredPastTheWidth`: a shaded column with no coverage is read and never written, and the scissor's own column is always such a one (Mars_Gpu.md §5.4).
    fn covered_past_the_width(&mut self, y: i32, left: i32, right: i32) -> i32 {
        if right < self.color_image_width {
            return 0;
        }
        self.row_coverage(y, left, right);
        let mut covered = 0;
        for x in left.max(self.color_image_width)..=right {
            if self.coverage[x as usize] != 0 {
                covered += 1;
            }
        }
        covered
    }
}
