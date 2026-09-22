//! The colour image, the scissor, the fill cycle, and the commands that reach the walker: C#'s `Rdp.Fill.cs`.

use super::{ATTRIBUTE_S, ATTRIBUTE_Z, COPY_CYCLE, FILL_CYCLE, ONE_CYCLE, Rdp, RdpMemory, Rows, TWO_CYCLE, sign_extend};

/// Quarter pixels, as a command carries them.
#[inline(always)]
pub(super) fn quarters(field: u64) -> i32 {
    (field & 0xFFF) as i32
}

#[inline(always)]
fn joined(whole: u64, fraction: u64, shift: u32) -> i32 {
    ((((whole >> shift) & 0xFFFF) << 16) | ((fraction >> shift) & 0xFFFF)) as i32
}

impl Rdp {
    pub(super) fn set_color_image(&mut self, word: u64) {
        self.color_image_format = ((word >> 53) & 7) as i32;
        self.color_image_size = ((word >> 51) & 3) as i32;
        self.color_image_bytes = match self.color_image_size {
            1 => 1,
            2 => 2,
            3 => 4,
            _ => 0,
        };
        self.color_image_width = ((word >> 32) & 0x3FF) as i32 + 1;
        self.color_image = (word as u32) & 0x00FF_FFFF;
    }

    pub(super) fn set_scissor(&mut self, word: u64) {
        self.scissor_left = quarters(word >> 44);
        self.scissor_top = quarters(word >> 32);
        self.scissor_right = quarters(word >> 12);
        self.scissor_bottom = quarters(word);
        self.scissor_field = ((word >> 25) & 1) != 0;
        self.scissor_keep_odd = ((word >> 24) & 1) != 0;
    }

    pub(super) fn fill(&mut self, word: u64, mem: &mut RdpMemory) {
        self.clear_attributes();
        let rows = self.walk_rectangle(word);
        self.draw(rows, true, 0, 0, mem);
    }

    /// A rectangle is a primitive with its major edge on the left and no slope; fill and copy include the bottom row.
    pub(super) fn walk_rectangle(&mut self, word: u64) -> Rows {
        let copy_or_fill = self.modes.cycle_type >= COPY_CYCLE;
        let right = quarters(word >> 44);
        let bottom = quarters(word >> 32) | if copy_or_fill { 3 } else { 0 };
        let left = quarters(word >> 12);
        let top = quarters(word);

        let right_x = ((right >> 2) << 16) | ((right & 3) << 14);
        let left_x = ((left >> 2) << 16) | ((left & 3) << 14);

        self.walk(true, top, bottom, bottom, left_x, right_x, right_x, 0, 0, 0, false)
    }

    pub(super) fn triangle(&mut self, id: u32, mem: &mut RdpMemory) {
        let edges = self.command[0];
        let major_on_left = ((edges >> 55) & 1) != 0;
        self.decode_attributes(id);

        let c1 = self.command[1];
        let c2 = self.command[2];
        let c3 = self.command[3];
        let rows = self.walk(
            major_on_left,
            sign_extend(edges as u32, 14),
            sign_extend((edges >> 16) as u32, 14),
            sign_extend((edges >> 32) as u32, 14),
            sign_extend((c2 >> 32) as u32, 28),
            sign_extend((c3 >> 32) as u32, 28),
            sign_extend((c1 >> 32) as u32, 28),
            sign_extend(c2 as u32, 30),
            sign_extend(c3 as u32, 30),
            sign_extend(c1 as u32, 30),
            (c2 as i32) < 0,
        );
        self.draw(rows, major_on_left, ((edges >> 48) & 7) as i32, ((edges >> 51) & 7) as i32, mem);
    }

    /// Shade in the eight words after the edges, texture in the next eight, depth in the last two.
    fn decode_attributes(&mut self, id: u32) {
        self.clear_attributes();
        let mut at = 4;
        let cmd = self.command;

        if (id & 4) != 0 {
            for c in 0..4 {
                let shift = 48 - 16 * c as u32;
                self.attribute_value[c] = joined(cmd[at], cmd[at + 2], shift);
                self.attribute_dx[c] = joined(cmd[at + 1], cmd[at + 3], shift);
                self.attribute_de[c] = joined(cmd[at + 4], cmd[at + 6], shift);
                self.attribute_dy[c] = joined(cmd[at + 5], cmd[at + 7], shift);
            }
            at += 8;
        }

        if (id & 2) != 0 {
            for c in 0..3 {
                let shift = 48 - 16 * c as u32;
                self.attribute_value[ATTRIBUTE_S + c] = joined(cmd[at], cmd[at + 2], shift);
                self.attribute_dx[ATTRIBUTE_S + c] = joined(cmd[at + 1], cmd[at + 3], shift);
                self.attribute_de[ATTRIBUTE_S + c] = joined(cmd[at + 4], cmd[at + 6], shift);
                self.attribute_dy[ATTRIBUTE_S + c] = joined(cmd[at + 5], cmd[at + 7], shift);
            }
            at += 8;
        }

        if (id & 1) != 0 {
            self.attribute_value[ATTRIBUTE_Z] = (cmd[at] >> 32) as i32;
            self.attribute_dx[ATTRIBUTE_Z] = cmd[at] as i32;
            self.attribute_de[ATTRIBUTE_Z] = (cmd[at + 1] >> 32) as i32;
            self.attribute_dy[ATTRIBUTE_Z] = cmd[at + 1] as i32;
        }
    }

    pub(super) fn clear_attributes(&mut self) {
        self.attribute_value = [0; 8];
        self.attribute_dx = [0; 8];
        self.attribute_de = [0; 8];
        self.attribute_dy = [0; 8];
    }

    pub(super) fn draw(&mut self, rows: Rows, major_on_left: bool, tile: i32, max_level: i32, mem: &mut RdpMemory) {
        match self.modes.cycle_type {
            FILL_CYCLE => self.fill_spans(rows, mem),
            ONE_CYCLE => self.draw_one_cycle(rows, major_on_left, tile, max_level, mem),
            TWO_CYCLE => self.draw_two_cycle(rows, major_on_left, tile, max_level, mem),
            _ => self.draw_copy(rows, major_on_left, tile, max_level, mem),
        }
        if self.split.workers > 1 && !self.split.alone && self.modes.cycle_type <= TWO_CYCLE {
            self.record_aliased_read(rows, major_on_left, mem);
        }
    }

    /// A four-bit image has nothing to fill.
    fn fill_spans(&mut self, rows: Rows, mem: &mut RdpMemory) {
        let bytes = self.color_image_bytes;
        if bytes == 0 {
            return;
        }
        let image = self.color_image & !(bytes as u32).wrapping_sub(1);
        let width = self.color_image_width;
        let color = self.fill_color;

        for y in rows.0..=rows.1 {
            let yu = y as usize;
            if !self.span_drawn[yu] || !self.owns(y) {
                continue;
            }
            for x in self.span_left[yu]..=self.span_right[yu] {
                let address = image.wrapping_add(y.wrapping_mul(width).wrapping_add(x).wrapping_mul(bytes) as u32);
                fill_pixel(address, bytes, color, mem);
            }
        }
    }
}

/// Each byte takes its address's lane of the fill colour in a big-endian word, and each odd byte's low bit the hidden bits.
#[inline(always)]
fn fill_pixel(address: u32, bytes: i32, color: u32, mem: &mut RdpMemory) {
    for i in 0..bytes as u32 {
        let at = address.wrapping_add(i);
        if at as usize >= mem.len() {
            continue;
        }
        let value = (color >> (24 - 8 * (at & 3))) as u8;
        mem.set(at as usize, value);
        if (at & 1) != 0 {
            mem.set_hidden((at >> 1) as usize, (value & 1) * 3);
        }
    }
}
