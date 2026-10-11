//! VRAM, 1024 by 512 halfwords (psx-spx "VRAM Overview"), held as the little-endian bytes the `VRAM` space shows, and
//! the transfers that bypass the rasteriser: the fill, the VRAM-to-VRAM copy and the CPU's writes and reads, each
//! with psx-spx's masking of its parameters and its wrapping at the edges (Diamond_GPU.md §3).

pub const WIDTH: u32 = 1024;
pub const HEIGHT: u32 = 512;
pub const BYTES: usize = (WIDTH * HEIGHT * 2) as usize;

#[derive(Clone)]
pub struct Vram {
    pub bytes: Vec<u8>,
}

impl Default for Vram {
    fn default() -> Vram {
        Vram { bytes: vec![0; BYTES] }
    }
}

/// The mask bit's two settings, GP0(E6h): bit 15 forced on every write, and pixels with bit 15 left alone.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Mask {
    pub set: u16,
    pub check: bool,
}

impl Mask {
    pub fn from_e6(word: u32) -> Mask {
        Mask { set: ((word & 1) as u16) << 15, check: word & 2 != 0 }
    }
}

/// A transfer's rectangle, its parameters masked as psx-spx's "Masking for COPY Commands parameters" gives.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl Rect {
    pub fn copy_params(position: u32, size: u32) -> Rect {
        Rect { x: position & 0x3FF, y: position >> 16 & 0x1FF, w: ((size & 0xFFFF).wrapping_sub(1) & 0x3FF) + 1, h: ((size >> 16).wrapping_sub(1) & 0x1FF) + 1 }
    }

    /// The fill's: X in steps of 16, its width rounded up to 16, no fill at a zero size.
    pub fn fill_params(position: u32, size: u32) -> Rect {
        Rect { x: position & 0x3F0, y: position >> 16 & 0x1FF, w: ((size & 0x3FF) + 0xF) & !0xF, h: size >> 16 & 0x1FF }
    }

    pub fn pixels(&self) -> u32 {
        self.w * self.h
    }

    /// The `i`th pixel in row order, wrapped at VRAM's edges with no carry between the axes.
    pub fn at(&self, i: u32) -> (u32, u32) {
        ((self.x + i % self.w) & (WIDTH - 1), (self.y + i / self.w) & (HEIGHT - 1))
    }
}

impl Vram {
    #[inline]
    fn index(x: u32, y: u32) -> usize {
        (((y & (HEIGHT - 1)) << 10 | (x & (WIDTH - 1))) as usize) << 1
    }

    #[inline]
    pub fn get(&self, x: u32, y: u32) -> u16 {
        let i = Self::index(x, y);
        u16::from_le_bytes([self.bytes[i], self.bytes[i + 1]])
    }

    #[inline]
    pub fn set(&mut self, x: u32, y: u32, v: u16) {
        let i = Self::index(x, y);
        self.bytes[i..i + 2].copy_from_slice(&v.to_le_bytes());
    }

    /// A write under the mask settings: refused over a masked pixel when checking, bit 15 forced when setting.
    #[inline]
    pub fn put(&mut self, x: u32, y: u32, v: u16, mask: Mask) {
        if mask.check && self.get(x, y) & 0x8000 != 0 {
            return;
        }
        self.set(x, y, v | mask.set);
    }

    /// GP0(02h): the colour's top five bits of each channel with bit 15 clear, no clipping and no mask; the lines of
    /// parity `skip` left alone, as interlaced drawing leaves them (Diamond_Disputes.md D-23).
    pub fn fill(&mut self, r: Rect, colour: u32, skip: Option<i32>) {
        let c = ((colour >> 3 & 0x1F) | (colour >> 11 & 0x1F) << 5 | (colour >> 19 & 0x1F) << 10) as u16;
        for j in 0..r.h {
            if skip == Some(((r.y + j) & 1) as i32) {
                continue;
            }
            for i in 0..r.w {
                self.set(r.x + i, r.y + j, c);
            }
        }
    }

    /// GP0(80h): row by row, each row read whole before it is written (Diamond_GPU.md §3, Diamond_Disputes.md D-23).
    pub fn copy(&mut self, src: Rect, dst: Rect, mask: Mask) {
        let mut row = [0u16; WIDTH as usize];
        for j in 0..src.h {
            for i in 0..src.w {
                row[i as usize] = self.get(src.x + i, src.y + j);
            }
            for i in 0..src.w {
                self.put(dst.x + i, dst.y + j, row[i as usize], mask);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_parameters_are_masked_as_documented() {
        assert_eq!(Rect::copy_params(0x0200_0400, 0), Rect { x: 0, y: 0, w: 1024, h: 512 });
        assert_eq!(Rect::fill_params(0x0001_000F, 0x0001_03F1), Rect { x: 0, y: 1, w: 1024, h: 1 });
        assert_eq!(Rect::fill_params(0, 0x0001_0400).w, 0, "400h is handled as 0");
        let mut v = Vram::default();
        v.fill(Rect { x: 1016, y: 511, w: 16, h: 2 }, 0x00FF_FFFF, None);
        assert_eq!((v.get(1023, 511), v.get(7, 511), v.get(7, 0), v.get(8, 0), v.get(1015, 0)), (0x7FFF, 0x7FFF, 0x7FFF, 0, 0), "wrapped, no carry between axes");
    }

    /// amidog's MOVE, a row of 17 to 47 pixels copied 1 to 4 to the right over itself: every pixel moves as it was.
    #[test]
    fn a_copy_over_itself_to_the_right_moves_each_row_as_it_was() {
        for (w, dx) in [(17, 1), (32, 2), (47, 4)] {
            let mut v = Vram::default();
            for x in 0..64 {
                v.set(16 + x, 16, 0x1000 | (16 + x) as u16);
            }
            v.copy(Rect { x: 16, y: 16, w, h: 1 }, Rect { x: 16 + dx, y: 16, w, h: 1 }, Mask::default());
            for x in 0..w {
                assert_eq!(v.get(16 + dx + x, 16), 0x1000 | (16 + x) as u16, "w {w}, dx {dx}, x {x}");
            }
        }
    }

    #[test]
    fn the_mask_bit_is_set_and_checked() {
        let mut v = Vram::default();
        v.put(0, 0, 0x1234, Mask::from_e6(1));
        assert_eq!(v.get(0, 0), 0x9234);
        v.put(0, 0, 0x0001, Mask::from_e6(2));
        assert_eq!(v.get(0, 0), 0x9234);
        v.put(0, 0, 0x0456, Mask::default());
        assert_eq!(v.get(0, 0), 0x0456, "a masked pixel is overwritten when not checking");
    }
}
