//! A list shared by several processors, each shading the rows that are its by count: C#'s `Classify`, stamps, `RecordAliasedRead` and `TakeScratchFrom`. See Mars_Rdp.md §2.8, Mars_Native.md §5.6.6.

use super::depth::read_depth_word;
use super::CombinerSelectors;
use super::{COPY_CYCLE, FILL_RECTANGLE, LOAD_BLOCK, LOAD_PALETTE, LOAD_TILE, ONE_CYCLE, Rdp, RdpMemory, Rows, SET_COLOR_IMAGE, SET_MASK_IMAGE, TEXTURE_RECTANGLE, TEXTURE_RECTANGLE_FLIPPED, TWO_CYCLE};

/// `Rdp.Step`: what the drain does with a gathered command.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    More,
    Ready,
    Leader,
    All,
    AllJoined,
}

/// C#'s `[SkipInState]` split fields: this processor's place, the primitive count, each scratch field's stamp, and the drawn-to extents.
#[derive(Clone, Debug)]
pub struct Split {
    pub worker: i32,
    pub workers: i32,
    pub alone: bool,
    pub primitive_sequence: i64,
    pub row_stamp: i64,
    pub last_shaded: i64,
    pub memory: i64,
    pub past_stored: i64,
    pub texel0: i64,
    pub texel1: i64,
    pub lod: i64,
    pub blended: i64,
    pub shift: i64,
    pub past_shift: i64,
    pub coverage: Box<[i64]>,
    pub primitives: i64,
    pub serialised: i64,
    pub hazard_loads: i64,
    pub aliased_reads: i64,
}

impl Default for Split {
    fn default() -> Self {
        Split {
            worker: 0,
            workers: 1,
            alone: false,
            primitive_sequence: 0,
            row_stamp: 0,
            last_shaded: 0,
            memory: 0,
            past_stored: 0,
            texel0: 0,
            texel1: 0,
            lod: 0,
            blended: 0,
            shift: 0,
            past_shift: 0,
            coverage: vec![0; super::SPAN_ROWS].into_boxed_slice(),
            primitives: 0,
            serialised: 0,
            hazard_loads: 0,
            aliased_reads: 0,
        }
    }
}

fn selects_combined(c: &CombinerSelectors) -> bool {
    c.color_a == 0 || c.color_b == 0 || c.color_d == 0 || c.color_c == 0 || c.color_c == 7 || c.alpha_a == 0 || c.alpha_b == 0 || c.alpha_d == 0
}

impl Rdp {
    /// `Configure`: this processor's index among those sharing the list.
    pub fn configure(&mut self, worker: i32, workers: i32) {
        self.split.worker = worker;
        self.split.workers = workers;
        self.split.alone = false;
    }

    /// `Owns`: a row is this processor's by its number modulo the count, or every row when the primitive is drawn alone.
    #[inline(always)]
    pub(super) fn owns(&self, y: i32) -> bool {
        let s = &*self.split;
        s.workers == 1 || s.alone || y % s.workers == s.worker
    }

    /// The pixel at the width reads the next row's first bytes, which another worker may be writing: read as nothing, since the next row's owner makes that read itself (Mars_Native.md §5.6.6).
    #[inline(always)]
    pub(super) fn blind(&self, x: i32, y: i32) -> bool {
        x >= self.color_image_width && self.split.workers > 1 && !self.split.alone && !self.owns(y + 1)
    }

    /// `Stamp`: the primitive's count over the row, so a later primitive's row is later than any row of an earlier one.
    #[inline(always)]
    pub(super) fn stamp(&self, row: i32) -> i64 {
        (self.split.primitive_sequence << 11) | row as u32 as i64
    }

    /// `Classify`: what every processor decides alike at a command's last word, from its own registers.
    pub(super) fn classify(&mut self, id: u32) -> Step {
        match id {
            0x08..=0x0F | TEXTURE_RECTANGLE | TEXTURE_RECTANGLE_FLIPPED | FILL_RECTANGLE => {
                self.split.primitive_sequence += 1;
                self.split.primitives += 1;
                if self.split.workers == 1 {
                    return Step::Ready;
                }
                let alone = self.serialised(id);
                self.split.alone = alone;
                if alone {
                    self.split.serialised += 1;
                    Step::Leader
                } else {
                    Step::Ready
                }
            }
            SET_COLOR_IMAGE | SET_MASK_IMAGE => {
                if self.split.workers > 1 {
                    Step::All
                } else {
                    Step::Ready
                }
            }
            LOAD_TILE | LOAD_BLOCK | LOAD_PALETTE => {
                if self.split.workers == 1 || !self.load_reaches_drawn(self.command[0], id == LOAD_BLOCK) {
                    return Step::Ready;
                }
                self.split.hazard_loads += 1;
                Step::AllJoined
            }
            _ => Step::Ready,
        }
    }

    /// `Serialised`: a live carry across rows, a span that can write past its row, or images whose rows overlap.
    fn serialised(&self, id: u32) -> bool {
        let m = &self.modes;
        let cycle = m.cycle_type;
        if cycle == TWO_CYCLE && (m.first_blend_cycle.second_alpha == 1 || selects_combined(&m.first_combine_cycle)) {
            return true;
        }
        if cycle == ONE_CYCLE && selects_combined(&m.second_combine_cycle) {
            return true;
        }
        let mut right = self.scissor_right;
        if matches!(id, FILL_RECTANGLE | TEXTURE_RECTANGLE | TEXTURE_RECTANGLE_FLIPPED) {
            right = right.min(super::fill::quarters(self.command[0] >> 44));
        }
        let width = self.color_image_width << 2;
        if if cycle >= COPY_CYCLE { right >= width } else { right > width } {
            return true;
        }
        if cycle >= COPY_CYCLE || !(m.depth_compare || m.depth_update) {
            return false;
        }
        let reach = self.reach();
        let color_to = self.color_image as i64 + reach * self.color_image_bytes.max(1) as i64;
        let depth_to = self.depth_image as i64 + reach * 2;
        (self.color_image as i64) < depth_to && (self.depth_image as i64) < color_to
    }

    /// `Reach`: the greatest pixel index a primitive under this scissor can address.
    fn reach(&self) -> i64 {
        let (rows, width) = (((self.scissor_bottom + 3) >> 2) as i64, self.color_image_width as i64);
        (rows * width).max((rows - 1) * width + (self.scissor_right >> 2) as i64 + 3)
    }

    /// `LoadReachesDrawn`, widened: whether a load reads bytes any draw into the current images could write before the next image change, which is a barrier.
    fn load_reaches_drawn(&self, word: u64, block: bool) -> bool {
        let sl = ((word >> 44) & 0xFFF) as i32;
        let tl = ((word >> 32) & 0xFFF) as i32;
        let sh = ((word >> 12) & 0xFFF) as i32;
        let th = (word & 0xFFF) as i32;
        let bits = 4i64 << self.texture_image_size;
        let row_bytes = (self.texture_image_width as i64 * bits + 7) / 8;
        let image = self.texture_image as i64;
        let (from, to) = if block {
            let left = ((sl << 20) >> 20) as i64;
            (image + (tl & 0x3FF) as i64 * row_bytes + left * bits / 8 - 16, image + (tl & 0x3FF) as i64 * row_bytes + (sh as i64 + 1) * bits / 8 + 32)
        } else {
            (image + (tl >> 2) as i64 * row_bytes + (sl >> 2) as i64 * bits / 8 - 16, image + (th >> 2) as i64 * row_bytes + ((sh as i64 >> 2) + 1) * bits / 8 + 32)
        };
        // C# asks only whether a draw since the image was set reached the bytes, so a load before the first draw races the draws after it.
        let bytes = self.color_image_bytes.max(1) as i64;
        let most = 1024 * self.color_image_width as i64 + 1024 + 3;
        let (color, depth) = (self.color_image as i64, self.depth_image as i64);
        (from < color + most * bytes && to > color - 2 * bytes) || (from < depth + most * 2 && to > depth)
    }

    /// `RecordAliasedRead`: the last row's pixel past the width reads the next row's first bytes, which that row's owner reads here, in raster order's time.
    pub(super) fn record_aliased_read(&mut self, rows: Rows, major_on_left: bool, mem: &RdpMemory) {
        let mut y = rows.1;
        while y >= rows.0 && (!self.span_drawn[y as usize] || self.span_right[y as usize] < self.span_left[y as usize]) {
            y -= 1;
        }
        if y < rows.0 {
            return;
        }
        let yu = y as usize;
        let x = if major_on_left { self.span_right[yu] } else { self.span_left[yu] };
        if x < self.color_image_width || self.owns(y) || !self.owns(y + 1) {
            return;
        }

        self.split.aliased_reads += 1;
        let stamp = self.stamp(y + 1);
        let pixel = y.wrapping_mul(self.color_image_width).wrapping_add(x);
        self.read_memory(pixel, mem);
        self.split.memory = stamp;

        let m = &self.modes;
        let delta_z_encoded = super::depth::delta_z_encoding(if m.primitive_depth { self.primitive_delta_z } else { self.depth_slope });
        let shifts = (if m.cycle_type == TWO_CYCLE { m.second_blend_cycle.second_alpha } else { m.first_blend_cycle.second_alpha }) == 1;
        if !m.depth_compare {
            self.past_stored_encoded = 0xF;
            if shifts {
                self.blend_shift_a = 0;
                self.blend_shift_b = if delta_z_encoded < 0xB { 4 } else { 0xF - delta_z_encoded };
                self.split.shift = stamp;
            }
        } else {
            let (stored, hidden) = read_depth_word((self.depth_image >> 1).wrapping_add(pixel as u32), mem);
            let stored_encoded = ((stored & 3) << 2) | hidden;
            self.past_stored_encoded = stored_encoded;
            if shifts {
                self.blend_shift_a = (delta_z_encoded - stored_encoded).clamp(0, 4);
                self.blend_shift_b = (stored_encoded - delta_z_encoded).clamp(0, 4);
                self.split.shift = stamp;
            }
        }
        self.split.past_stored = stamp;
    }

    /// `TakeScratchFrom`: this processor's scratch replaced by the other's wherever the other wrote it later in raster order.
    pub fn take_scratch_from(&mut self, other: &Rdp) {
        let (mine, theirs) = (&mut *self.split, &*other.split);
        if theirs.last_shaded > mine.last_shaded {
            mine.last_shaded = theirs.last_shaded;
            (self.combined, self.pixel, self.shade, self.blender_shade_alpha) = (other.combined, other.pixel, other.shade, other.blender_shade_alpha);
        }
        if theirs.memory > mine.memory {
            (mine.memory, self.memory) = (theirs.memory, other.memory);
        }
        if theirs.past_stored > mine.past_stored {
            (mine.past_stored, self.past_stored_encoded) = (theirs.past_stored, other.past_stored_encoded);
        }
        if theirs.texel0 > mine.texel0 {
            (mine.texel0, self.texel0) = (theirs.texel0, other.texel0);
        }
        if theirs.texel1 > mine.texel1 {
            (mine.texel1, self.texel1) = (theirs.texel1, other.texel1);
        }
        if theirs.lod > mine.lod {
            (mine.lod, self.lod_fraction) = (theirs.lod, other.lod_fraction);
        }
        if theirs.blended > mine.blended {
            (mine.blended, self.blended) = (theirs.blended, other.blended);
        }
        if theirs.shift > mine.shift {
            (mine.shift, self.blend_shift_a, self.blend_shift_b) = (theirs.shift, other.blend_shift_a, other.blend_shift_b);
        }
        if theirs.past_shift > mine.past_shift {
            (mine.past_shift, self.past_shift_a, self.past_shift_b) = (theirs.past_shift, other.past_shift_a, other.past_shift_b);
        }
        for x in 0..self.coverage.len() {
            if theirs.coverage[x] > mine.coverage[x] {
                mine.coverage[x] = theirs.coverage[x];
                self.coverage[x] = other.coverage[x];
            }
        }
    }

    /// `CopyStateFrom`: everything the state holds, the decoded modes, the stamps and the extents, so a joining processor stands where the leader stands.
    pub fn copy_state_from(&mut self, other: &Rdp) {
        let (worker, workers) = (self.split.worker, self.split.workers);
        let multiple = *self.multiple;
        self.clone_from(other);
        self.configure(worker, workers);
        // The walker's scratch is read at the console's width, and widened again after for a processor at a multiple (Mars_Rdp.md §11).
        *self.multiple = multiple;
        if multiple.scaled {
            self.widen(multiple.scale);
        }
    }
}

impl Rdp {
    /// The stamps a shaded row writes at its start, for the fields every pixel writes; the coverage's beside it when the list is shared.
    #[inline(always)]
    pub(super) fn stamp_row(&mut self, y: i32, left: i32, right: i32, lod: bool, texel0: bool, texel1: bool) {
        let stamp = self.stamp(y);
        let s = &mut *self.split;
        s.row_stamp = stamp;
        s.last_shaded = stamp;
        s.memory = stamp;
        s.past_stored = stamp;
        if lod {
            s.lod = stamp;
        }
        if texel0 {
            s.texel0 = stamp;
        }
        if texel1 {
            s.texel1 = stamp;
        }
        if s.workers > 1 {
            s.coverage[left as usize..=right as usize].fill(stamp);
        }
    }
}
