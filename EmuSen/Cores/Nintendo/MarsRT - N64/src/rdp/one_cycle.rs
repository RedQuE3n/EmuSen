//! The one-cycle mode, the combiner, the blender, dither, and the colour image they read and write: C#'s `Rdp.OneCycle.cs`.

use super::depth::{delta_z_encoding, store_depth};
use super::tables::{BLEND_QUOTIENTS, COVERAGE_OFFSETS, DITHER_TABLES};
use super::{ATTRIBUTE_S, ATTRIBUTE_T, ATTRIBUTE_W, ATTRIBUTE_Z, ATTRIBUTES, BlendSelectors, Color, CombinerSelectors, Rdp, RdpMemory, Rows};

/// Nine bits whose top two are both set are negative.
#[inline(always)]
fn extend9(value: i32) -> i32 {
    if (value & 0x180) == 0x180 { value | !0x1FF } else { value & 0x1FF }
}

/// The multiplier is negative on its ninth bit alone.
#[inline(always)]
fn signed_multiplier(value: i32) -> i32 {
    value | -(value & 0x100)
}

#[inline(always)]
pub(super) fn clamp9(value: i32) -> i32 {
    match (value >> 7) & 3 {
        2 => 0xFF,
        3 => 0,
        _ => value & 0xFF,
    }
}

#[inline(always)]
fn color_equation(a: i32, b: i32, c: i32, d: i32) -> i32 {
    ((extend9(a) - extend9(b)) * signed_multiplier(c) + (extend9(d) << 8) + 0x80) & 0x1FFFF
}

#[inline(always)]
fn alpha_equation(a: i32, b: i32, c: i32, d: i32) -> i32 {
    (((extend9(a) - extend9(b)) * signed_multiplier(c) + (extend9(d) << 8) + 0x80) >> 8) & 0x1FF
}

/// Rounds up to the next multiple of eight when the dither value is below the colour's low three bits.
#[inline(always)]
fn dithered(value: i32, dither: i32) -> i32 {
    let raised = if value > 247 { 255 } else { (value & 0xF8) + 8 };
    if dither < (value & 7) { raised } else { value }
}

impl Rdp {
    /// Each row starts from its major edge's values, stepped to the first pixel drawn, and steps once per pixel.
    pub(super) fn draw_one_cycle(&mut self, rows: Rows, major_on_left: bool, tile: i32, max_level: i32, mem: &mut RdpMemory) {
        let primitive_depth = self.modes.primitive_depth;
        let delta_z = if primitive_depth { self.primitive_delta_z } else { self.depth_slope };
        let delta_z_encoded = delta_z_encoding(delta_z);
        if primitive_depth {
            self.depth_correct_dx = 0;
            self.depth_correct_dy = 0;
        }

        let steps = self.pixel_steps(major_on_left);
        let direction = if major_on_left { 1 } else { -1 };

        let (texel0, texel1) = self.combiner_texels();
        let last_cycle = self.modes.second_combine_cycle;
        let lod_fraction = last_cycle.color_c == 13 || last_cycle.alpha_c == 0;
        let lod = self.modes.lod_enabled || lod_fraction;
        let bilinear = self.modes.bilinear_first_cycle;

        let (mut dither_color, mut dither_alpha) = (7, 0);
        let dither = self.modes.dither_table != 0xF;
        let mut values = [0i32; ATTRIBUTES];

        for y in rows.0..=rows.1 {
            let yu = y as usize;
            if !self.span_drawn[yu] || self.span_right[yu] < self.span_left[yu] {
                continue;
            }

            let (left, right) = (self.span_left[yu], self.span_right[yu]);
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
            let length = last + clipped;
            let long_span = length > 7;
            let mid_span = length == 7;
            let next_row_drawn = y < rows.1 && self.span_drawn[yu + 1];
            let mut level = (tile, self.lod_fraction);
            let mut level_ready = false;

            let mut x = if major_on_left { left } else { right };
            for n in 0..=last {
                let (ds, dt, dw) = (steps[ATTRIBUTE_S], steps[ATTRIBUTE_T], steps[ATTRIBUTE_W]);
                let (s0, t0, w0) = (values[ATTRIBUTE_S], values[ATTRIBUTE_T], values[ATTRIBUTE_W]);
                let (s1, t1, w1) = (s0.wrapping_add(ds), t0.wrapping_add(dt), w0.wrapping_add(dw));

                if lod && (texel0 || texel1 || lod_fraction) && !level_ready {
                    level = self.pixel_level_of_detail(
                        s0, t0, w0, ds, dt, dw, y + 1, next_row_drawn, n == last, n == last - 1, long_span, mid_span, tile, max_level,
                    );
                }

                self.lod_fraction = level.1;
                level_ready = false;

                if texel0 || texel1 {
                    let (s, t) = self.texture_coordinates(s0, t0, w0);
                    self.texel0 = self.texel(s, t, level.0, bilinear, false, Color::default());
                }

                if texel1 {
                    let (s, t) = if n == last && long_span && next_row_drawn {
                        let next = (yu + 1) * ATTRIBUTES;
                        self.texture_coordinates(
                            self.span_attributes[next + ATTRIBUTE_S],
                            self.span_attributes[next + ATTRIBUTE_T],
                            self.span_attributes[next + ATTRIBUTE_W],
                        )
                    } else {
                        self.texture_coordinates(s1, t1, w1)
                    };

                    let mut next_tile = tile;
                    if lod && n < last {
                        level = self.pixel_level_of_detail(
                            s1, t1, w1, ds, dt, dw, y + 1, next_row_drawn, n + 1 == last, n + 1 == last - 1, long_span, mid_span, tile, max_level,
                        );
                        next_tile = level.0;
                        level_ready = true;
                    } else if lod {
                        next_tile = self.after_span_tile(s1, t1, w1, ds, dt, dw, y + 1, next_row_drawn, long_span, mid_span, length == 6, tile, max_level);
                    }

                    self.texel1 = self.texel(s, t, next_tile, bilinear, false, Color::default());
                }

                let mask = self.coverage[x as usize];
                let mut coverage = mask.count_ones() as i32;
                let coverage_bit = (mask & 0x80) != 0;
                let offset = COVERAGE_OFFSETS[mask as usize];

                self.shade = self.shaded(&values, offset, coverage);
                let z = self.correct_depth((values[ATTRIBUTE_Z] >> 10) & 0x3F_FFFF, offset, coverage);

                if dither {
                    self.dither(x, y, &mut dither_color, &mut dither_alpha);
                }
                self.combine_second_cycle(dither_alpha, &mut coverage);

                let pixel = y.wrapping_mul(self.color_image_width).wrapping_add(x);
                let memory_coverage = self.read_memory(pixel, mem);
                let depth_index = (self.depth_image >> 1).wrapping_add(pixel as u32);

                let (mut blend, mut overflow) = (false, false);
                if self.compare_depth(depth_index, z, delta_z, delta_z_encoded, memory_coverage, &mut coverage, &mut blend, &mut overflow, mem)
                    && let Some((r, g, b)) = self.blend_one_cycle(dither_color, blend, overflow, coverage, coverage_bit)
                {
                    self.write_memory(pixel, r, g, b, blend, coverage, memory_coverage, mem);
                    if self.modes.depth_update {
                        store_depth(depth_index, z, delta_z_encoded, mem);
                    }
                }

                for c in 0..ATTRIBUTES {
                    values[c] = values[c].wrapping_add(steps[c]);
                }
                x += direction;
            }
        }
    }

    /// Every attribute's step per pixel, signed by the span's direction; depth holds still at the primitive's.
    pub(super) fn pixel_steps(&self, major_on_left: bool) -> [i32; ATTRIBUTES] {
        let direction: i32 = if major_on_left { 1 } else { -1 };
        let mut steps = [0i32; ATTRIBUTES];
        for (step, shade) in steps.iter_mut().zip(self.shade_step) {
            *step = direction.wrapping_mul(shade);
        }
        steps[ATTRIBUTE_Z] = if self.modes.primitive_depth { 0 } else { direction.wrapping_mul(self.depth_step) };
        for c in 0..3 {
            steps[ATTRIBUTE_S + c] = direction.wrapping_mul(self.texture_step[c]);
        }
        steps
    }

    /// The pixel's four shade channels, corrected for partial coverage.
    #[inline(always)]
    pub(super) fn shaded(&self, values: &[i32; ATTRIBUTES], offset: (u8, u8), coverage: i32) -> Color {
        let (dx, dy) = (&self.shade_correct_dx, &self.shade_correct_dy);
        Color {
            r: Rdp::correct_shade(values[0] >> 14, dx[0], dy[0], offset, coverage),
            g: Rdp::correct_shade(values[1] >> 14, dx[1], dy[1], offset, coverage),
            b: Rdp::correct_shade(values[2] >> 14, dx[2], dy[2], offset, coverage),
            a: Rdp::correct_shade(values[3] >> 14, dx[3], dy[3], offset, coverage),
        }
    }

    /// Rows are counted in fields under an interlaced scissor.
    #[inline(always)]
    pub(super) fn dither(&self, x: i32, y: i32, color: &mut i32, alpha: &mut i32) {
        let index = (((y >> (if self.scissor_field { 1 } else { 0 })) & 3) << 2) | (x & 3);
        let (c, a) = DITHER_TABLES[self.modes.dither_table][index as usize];
        *color = c as i32;
        *alpha = a as i32;
    }

    /// (A - B) × C + D for one cycle's selectors: colour before its shift to nine bits, alpha after.
    #[inline(always)]
    pub(super) fn combiner_equations(&self, c: CombinerSelectors) -> (i32, i32, i32, i32) {
        let a = self.color_a(c.color_a);
        let b = self.color_b(c.color_b);
        let m = self.color_c(c.color_c);
        let d = self.color_a(c.color_d);

        (
            color_equation(a.r, b.r, m.r, d.r),
            color_equation(a.g, b.g, m.g, d.g),
            color_equation(a.b, b.b, m.b, d.b),
            alpha_equation(self.alpha_abd(c.alpha_a), self.alpha_abd(c.alpha_b), self.alpha_c(c.alpha_c), self.alpha_abd(c.alpha_d)),
        )
    }

    /// The one-cycle mode's cycle and the two-cycle mode's last: the pixel's colour and alpha, clamped to eight bits.
    pub(super) fn combine_second_cycle(&mut self, dither_alpha: i32, coverage: &mut i32) {
        let last = self.modes.second_combine_cycle;
        let (red, green, blue, alpha) = self.combiner_equations(last);

        self.combined = Color { r: red >> 8, g: green >> 8, b: blue >> 8, a: alpha };

        let key = self.modes.key_enabled;
        let mut key_alpha = 0;
        if key {
            key_alpha = self.chroma_key(red, green, blue);
            let through = self.color_a(last.color_a);
            self.pixel = Color { r: clamp9(through.r), g: clamp9(through.g), b: clamp9(through.b), a: 0 };
        } else {
            let c = self.combined;
            self.pixel = Color { r: clamp9(c.r), g: clamp9(c.g), b: clamp9(c.b), a: 0 };
        }

        let mut pixel_alpha = clamp9(alpha);
        if pixel_alpha == 0xFF {
            pixel_alpha = 0x100;
        }

        let times_alpha = self.modes.coverage_times_alpha;
        let mut scaled = 0;
        if times_alpha {
            scaled = (pixel_alpha * *coverage + 4) >> 3;
            *coverage = (scaled >> 5) & 0xF;
        }

        if !self.modes.alpha_from_coverage {
            if key {
                pixel_alpha = key_alpha;
            } else {
                pixel_alpha += dither_alpha;
                if (pixel_alpha & 0x100) != 0 {
                    pixel_alpha = 0xFF;
                }
            }
        } else {
            pixel_alpha = if times_alpha { scaled } else { *coverage << 5 };
            if pixel_alpha > 0xFF {
                pixel_alpha = 0xFF;
            }
        }

        self.pixel.a = pixel_alpha;

        self.blender_shade_alpha = self.shade.a + dither_alpha;
        if (self.blender_shade_alpha & 0x100) != 0 {
            self.blender_shade_alpha = 0xFF;
        }
    }

    /// Which texels the second cycle's selectors read.
    fn combiner_texels(&self) -> (bool, bool) {
        let c = self.modes.second_combine_cycle;
        let reads = |texel: i32| {
            c.color_a == texel
                || c.color_b == texel
                || c.color_d == texel
                || c.color_c == texel
                || c.color_c == texel + 7
                || c.alpha_a == texel
                || c.alpha_b == texel
                || c.alpha_c == texel
                || c.alpha_d == texel
        };
        (reads(1), reads(2))
    }

    /// Inputs A and D, which select alike.
    #[inline(always)]
    pub(super) fn color_a(&self, selector: i32) -> Color {
        match selector {
            0 => self.combined,
            1 => self.texel0,
            2 => self.texel1,
            3 => self.primitive_color,
            4 => self.shade,
            5 => self.environment_color,
            6 => Color::broadcast(0x100),
            _ => Color::default(),
        }
    }

    #[inline(always)]
    fn color_b(&self, selector: i32) -> Color {
        match selector {
            0 => self.combined,
            1 => self.texel0,
            2 => self.texel1,
            3 => self.primitive_color,
            4 => self.shade,
            5 => self.environment_color,
            6 => self.key_center,
            7 => Color::broadcast(self.k4),
            _ => Color::default(),
        }
    }

    #[inline(always)]
    fn color_c(&self, selector: i32) -> Color {
        match selector {
            0 => self.combined,
            1 => self.texel0,
            2 => self.texel1,
            3 => self.primitive_color,
            4 => self.shade,
            5 => self.environment_color,
            6 => self.key_scale,
            7 => Color::broadcast(self.combined.a),
            8 => Color::broadcast(self.texel0.a),
            9 => Color::broadcast(self.texel1.a),
            10 => Color::broadcast(self.primitive_color.a),
            11 => Color::broadcast(self.shade.a),
            12 => Color::broadcast(self.environment_color.a),
            13 => Color::broadcast(self.lod_fraction),
            14 => Color::broadcast(self.primitive_lod_fraction),
            15 => Color::broadcast(self.k5),
            _ => Color::default(),
        }
    }

    #[inline(always)]
    fn alpha_abd(&self, selector: i32) -> i32 {
        match selector {
            0 => self.combined.a,
            1 => self.texel0.a,
            2 => self.texel1.a,
            3 => self.primitive_color.a,
            4 => self.shade.a,
            5 => self.environment_color.a,
            6 => 0x100,
            _ => 0,
        }
    }

    #[inline(always)]
    fn alpha_c(&self, selector: i32) -> i32 {
        match selector {
            0 => self.lod_fraction,
            1 => self.texel0.a,
            2 => self.texel1.a,
            3 => self.primitive_color.a,
            4 => self.shade.a,
            5 => self.environment_color.a,
            6 => self.primitive_lod_fraction,
            _ => 0,
        }
    }

    /// Alpha compare, the coverage test, and then the blend; None when the pixel is not written.
    #[inline(always)]
    fn blend_one_cycle(&self, dither_color: i32, blend: bool, overflow: bool, coverage: i32, coverage_bit: bool) -> Option<(i32, i32, i32)> {
        let m = &self.modes;
        if m.alpha_compare && self.pixel.a < self.alpha_threshold() {
            return None;
        }
        if if m.antialias { coverage == 0 } else { !coverage_bit } {
            return None;
        }
        Some(self.blend(m.first_blend_cycle, self.pixel, dither_color, blend, overflow, self.blend_shift_a, self.blend_shift_b))
    }

    #[inline(always)]
    pub(super) fn alpha_threshold(&self) -> i32 {
        if self.modes.dither_alpha { 0 } else { self.blend_color.a }
    }

    /// Blend, pass the first input through, or take the second, then dither; source is what input 0 reads.
    #[allow(clippy::too_many_arguments)]
    #[inline(always)]
    pub(super) fn blend(
        &self,
        selectors: BlendSelectors,
        source: Color,
        dither_color: i32,
        blend: bool,
        overflow: bool,
        shift_a: i32,
        shift_b: i32,
    ) -> (i32, i32, i32) {
        let m = &self.modes;
        let (mut r, mut g, mut b) = if !m.color_on_coverage || overflow {
            let opaque = selectors.first_alpha == 0 && selectors.second_alpha == 0 && self.pixel.a >= 0xFF;
            if !blend || opaque {
                self.blend_input(selectors.first_color, source)
            } else {
                self.blend_equation(selectors, source, !m.force_blend, shift_a, shift_b)
            }
        } else {
            self.blend_input(selectors.second_color, source)
        };

        if m.rgb_dither != 3 {
            let two = m.rgb_dither == 2;
            r = dithered(r, if two { dither_color & 7 } else { dither_color });
            g = dithered(g, if two { (dither_color >> 3) & 7 } else { dither_color });
            b = dithered(b, if two { (dither_color >> 6) & 7 } else { dither_color });
        }
        (r, g, b)
    }

    #[inline(always)]
    fn blend_input(&self, selector: i32, source: Color) -> (i32, i32, i32) {
        let c = match selector {
            0 => source,
            1 => self.memory,
            2 => self.blend_color,
            _ => self.fog_color,
        };
        (c.r, c.g, c.b)
    }

    /// Weighted by the two alphas in eighths, then divided by their sum unless told not to.
    #[inline(always)]
    pub(super) fn blend_equation(&self, selectors: BlendSelectors, source: Color, divide: bool, shift_a: i32, shift_b: i32) -> (i32, i32, i32) {
        let first_alpha = match selectors.first_alpha {
            0 => self.pixel.a,
            1 => self.fog_color.a,
            2 => self.blender_shade_alpha,
            _ => 0,
        };
        let second_alpha = match selectors.second_alpha {
            0 => !first_alpha & 0xFF,
            1 => self.memory.a,
            2 => 0xFF,
            _ => 0,
        };

        let mut first = first_alpha >> 3;
        let mut second = second_alpha >> 3;
        if selectors.second_alpha == 1 {
            first = (first >> shift_a) & 0x3C;
            second = (second >> shift_b) | 3;
        }

        let one = self.blend_input(selectors.first_color, source);
        let two = self.blend_input(selectors.second_color, source);
        let weight = second + 1;

        let sum_r = one.0 * first + two.0 * weight;
        let sum_g = one.1 * first + two.1 * weight;
        let sum_b = one.2 * first + two.2 * weight;

        if !divide {
            return ((sum_r >> 5) & 0xFF, (sum_g >> 5) & 0xFF, (sum_b >> 5) & 0xFF);
        }

        let divisor = ((first & !3) + (second & !3) + 4) << 9;
        let q = |sum: i32| BLEND_QUOTIENTS[(divisor | ((sum >> 2) & 0x7FF)) as usize] as i32;
        (q(sum_r), q(sum_g), q(sum_b))
    }

    /// Memory colour and, with image reads on, the coverage stored beside it.
    #[inline(always)]
    pub(super) fn read_memory(&mut self, pixel: i32, mem: &RdpMemory) -> i32 {
        self.memory = Color { r: 0, g: 0, b: 0, a: 0xE0 };

        match self.color_image_size {
            1 => {
                let at = self.color_image.wrapping_add(pixel as u32) as usize;
                let value = if at < mem.len() { mem.get(at) as i32 } else { 0 };
                self.memory = Color { r: value, g: value, b: value, a: 0xE0 };
                7
            }
            2 => {
                let word = (self.color_image >> 1).wrapping_add(pixel as u32);
                let at = word.wrapping_mul(2);
                let valid = (at.wrapping_add(1) as usize) < mem.len();
                let value = if valid { ((mem.get(at as usize) as i32) << 8) | mem.get(at as usize + 1) as i32 } else { 0 };
                let hidden = if valid { mem.get_hidden(word as usize) as i32 } else { 0 };

                self.memory = if self.color_image_format == 0 {
                    Color { r: (value >> 8) & 0xF8, g: (value & 0x7C0) >> 3, b: (value & 0x3E) << 2, a: 0 }
                } else {
                    Color { r: value >> 8, g: value >> 8, b: value >> 8, a: 0 }
                };

                if !self.modes.image_read {
                    self.memory.a = 0xE0;
                    return 7;
                }

                let stored = if self.color_image_format == 0 { ((value & 1) << 2) | hidden } else { (value >> 5) & 7 };
                self.memory.a = stored << 5;
                stored
            }
            3 => {
                let at = (self.color_image >> 2).wrapping_add(pixel as u32).wrapping_mul(4);
                let value = if (at.wrapping_add(3) as usize) < mem.len() {
                    mem.be32(at as usize)
                } else {
                    0
                };
                let image_read = self.modes.image_read;
                self.memory = Color {
                    r: (value >> 24) as i32,
                    g: ((value >> 16) & 0xFF) as i32,
                    b: ((value >> 8) & 0xFF) as i32,
                    a: if image_read { (value & 0xE0) as i32 } else { 0xE0 },
                };
                if image_read { ((value >> 5) & 7) as i32 } else { 7 }
            }
            _ => 7,
        }
    }

    /// A four-bit image takes a zero byte; an eight-bit one alternates red and green by address.
    #[allow(clippy::too_many_arguments)]
    #[inline(always)]
    pub(super) fn write_memory(&self, pixel: i32, r: i32, g: i32, b: i32, blend: bool, coverage: i32, memory_coverage: i32, mem: &mut RdpMemory) {
        let len = mem.len();

        match self.color_image_size {
            0 => {
                let at = self.color_image.wrapping_add(pixel as u32) as usize;
                if at < len {
                    mem.set(at, 0);
                }
            }
            1 => {
                let at = self.color_image.wrapping_add(pixel as u32);
                if at as usize >= len {
                    return;
                }
                let odd = (at & 1) != 0;
                let value = if odd { g } else { r };
                mem.set(at as usize, value as u8);
                if odd {
                    mem.set_hidden((at >> 1) as usize, ((value & 1) * 3) as u8);
                }
            }
            2 => {
                let word = (self.color_image >> 1).wrapping_add(pixel as u32);
                let at = word.wrapping_mul(2);
                if at.wrapping_add(1) as usize >= len {
                    return;
                }

                let mut stored = self.final_coverage(blend, coverage, memory_coverage);
                let color = if self.color_image_format == 0 {
                    ((r & !7) << 8) | ((g & !7) << 3) | ((b & !7) >> 2)
                } else {
                    let color = (r << 8) | (stored << 5);
                    stored = 0;
                    color
                };

                let value = (color | (stored >> 2)) & 0xFFFF;
                mem.set(at as usize, (value >> 8) as u8);
                mem.set(at as usize + 1, value as u8);
                mem.set_hidden(word as usize, (stored & 3) as u8);
            }
            _ => {
                let index = (self.color_image >> 2).wrapping_add(pixel as u32);
                let at = index.wrapping_mul(4);
                if at.wrapping_add(3) as usize >= len {
                    return;
                }

                let stored = self.final_coverage(blend, coverage, memory_coverage);
                let a = at as usize;
                mem.set(a, r as u8);
                mem.set(a + 1, g as u8);
                mem.set(a + 2, b as u8);
                mem.set(a + 3, (stored << 5) as u8);
                let h = index.wrapping_mul(2) as usize;
                mem.set_hidden(h, ((g & 1) * 3) as u8);
                mem.set_hidden(h + 1, 0);
            }
        }
    }
}
