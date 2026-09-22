//! Coverage: eight of each pixel's sixteen sub-samples, from the walker's sub-scanline edges: C#'s `Rdp.Coverage.cs`.

use super::Rdp;

#[inline(always)]
fn left_samples(eighths: i32, samples: i32) -> i32 {
    (0x0F >> (((eighths & 7) + 1) >> 1)) & samples
}

#[inline(always)]
fn right_samples(eighths: i32, samples: i32) -> i32 {
    (0xF0 >> (((eighths & 7) + 1) >> 1)) & samples
}

impl Rdp {
    /// Each sub-scanline owns two samples of a pixel's eight, the pairs offset by one column on alternate lines.
    pub(super) fn row_coverage(&mut self, row: i32, left: i32, right: i32) {
        let (l, r) = (left as usize, right as usize);
        self.coverage[l..=r].fill(0xFF);

        for sub in 0..4 {
            let samples = 0xA >> (sub & 1);
            let shift = (sub - 2) & 4;
            let cleared = !(samples << shift) as u8;
            let k = (row * 4 + sub) as usize;

            if self.edge_invalid[k] {
                for c in &mut self.coverage[l..=r] {
                    *c &= cleared;
                }
                continue;
            }

            let left_edge = self.edge_left[k];
            let right_edge = self.edge_right[k];
            let left_pixel = left_edge >> 3;
            let right_pixel = right_edge >> 3;

            for x in left..=left_pixel {
                self.coverage[x as usize] &= cleared;
            }
            for x in right_pixel..=right {
                self.coverage[x as usize] &= cleared;
            }

            if right_pixel > left_pixel {
                self.coverage[left_pixel as usize] |= (left_samples(left_edge, samples) << shift) as u8;
                self.coverage[right_pixel as usize] |= (right_samples(right_edge, samples) << shift) as u8;
            } else if right_pixel == left_pixel {
                self.coverage[left_pixel as usize] |= ((left_samples(left_edge, samples) & right_samples(right_edge, samples)) << shift) as u8;
            }
        }
    }

    /// What is written back beside the colour: clamped, wrapped, forced full, or memory's.
    #[inline(always)]
    pub(super) fn final_coverage(&self, blend: bool, coverage: i32, memory: i32) -> i32 {
        match self.modes.coverage_destination {
            0 => {
                let sum = if blend { coverage + memory } else { coverage - 1 };
                if (sum & 8) != 0 { 7 } else { sum & 7 }
            }
            1 => (coverage + memory) & 7,
            2 => 7,
            _ => memory,
        }
    }
}
