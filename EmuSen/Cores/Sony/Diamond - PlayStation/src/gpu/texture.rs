//! Texturing (psx-spx "Texture Bitmaps", "Texture Palettes", "GP0(E2h)", "GPU Texture Caching"): a texel by the page,
//! its depth and the texture window, read through the 2 KiB texture cache and, for 4- and 8-bit pages, the CLUT
//! cache. Both caches keep what they read until they miss or are cleared, so VRAM written behind them is not seen
//! (Diamond_GPU.md §4).

use super::vram::Vram;

/// A primitive's texture: the page in halfwords, its depth (0 4-bit, 1 8-bit, 2 15-bit), the CLUT and the window.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Texture {
    pub page_x: u32,
    pub page_y: u32,
    pub depth: u32,
    pub clut_x: u32,
    pub clut_y: u32,
    /// GP0(E2h)'s masks and offsets, in texels.
    pub mask_u: u8,
    pub mask_v: u8,
    pub off_u: u8,
    pub off_v: u8,
}

impl Texture {
    /// From the page bits of GP0(E1h) or a polygon's page attribute, a CLUT attribute and GP0(E2h); bit 11 is the
    /// page's second megabyte only while GP1(09h) lets the 208-pin chip address one (psx-spx "GP1(09h)").
    pub fn new(page: u32, clut: u32, window: u32, two_megabytes: bool) -> Texture {
        let depth = (page >> 7 & 3).min(2);
        Texture {
            page_x: (page & 0xF) * 64,
            page_y: (page >> 4 & 1) * 256 + if two_megabytes { (page >> 11 & 1) * 512 } else { 0 },
            depth,
            clut_x: (clut & 0x3F) * 16,
            clut_y: clut >> 6 & 0x1FF,
            mask_u: ((window & 0x1F) * 8) as u8,
            mask_v: ((window >> 5 & 0x1F) * 8) as u8,
            off_u: ((window >> 10 & 0x1F) * 8) as u8,
            off_v: ((window >> 15 & 0x1F) * 8) as u8,
        }
    }

    /// psx-spx's texture window: `(t AND NOT (mask*8)) OR ((offset AND mask)*8)`.
    #[inline]
    pub fn window(&self, u: u8, v: u8) -> (u8, u8) {
        (u & !self.mask_u | self.off_u & self.mask_u, v & !self.mask_v | self.off_v & self.mask_v)
    }

    pub fn entries(&self) -> u32 {
        [16, 256, 0][self.depth as usize]
    }
}

/// The texture cache: 256 entries of four halfwords, direct-mapped by VRAM address, each tagged by the address of its
/// first halfword, in blocks of 64 by 64 texels at 4 bits and 32 halfwords by 32 lines at 8 and 15; and the page it
/// was filled for, whose change clears it (Diamond_GPU.md §4, Diamond_Disputes.md D-23).
#[derive(Clone)]
pub struct TextureCache {
    pub tags: [u32; 256],
    pub data: [[u16; 4]; 256],
    pub page: u32,
}

pub const NO_TAG: u32 = u32::MAX;

impl Default for TextureCache {
    fn default() -> TextureCache {
        TextureCache { tags: [NO_TAG; 256], data: [[0; 4]; 256], page: 0 }
    }
}

impl TextureCache {
    pub fn clear(&mut self) {
        self.tags = [NO_TAG; 256];
    }

    /// GP0(E1h)'s page bits as a page is set: another X or Y, or a change between 4 bits and the other depths, which
    /// index the entries otherwise, clears the cache.
    pub fn set_page(&mut self, e1: u32) {
        let page = e1 & 0x81F | ((e1 >> 7 & 3 != 0) as u32) << 5;
        if page != self.page {
            self.page = page;
            self.clear();
        }
    }

    /// The halfword at (x, y) through the cache, and whether it missed: entries are 8 bytes; a 4-bit page indexes them
    /// by x's bits 2-3 and y's bits 0-5, an 8- or 15-bit page by x's bits 2-4 and y's bits 0-4.
    #[inline]
    pub fn read(&mut self, vram: &Vram, x: u32, y: u32, depth: u32) -> (u16, bool) {
        let (x, y) = (x & 1023, y & 511);
        let index = if depth != 0 { (y & 31) << 3 | (x >> 2 & 7) } else { (y & 63) << 2 | (x >> 2 & 3) } as usize;
        let base = x & !3;
        let tag = y << 10 | base;
        let miss = self.tags[index] != tag;
        if miss {
            self.tags[index] = tag;
            for k in 0..4 {
                self.data[index][k] = vram.get(base + k as u32, y);
            }
        }
        (self.data[index][(x & 3) as usize], miss)
    }
}

/// The CLUT cache: the palette last loaded, where from, and how many of its entries (16 or 256).
#[derive(Clone)]
pub struct ClutCache {
    pub colours: [u16; 256],
    pub x: u32,
    pub y: u32,
    pub loaded: u32,
}

impl Default for ClutCache {
    fn default() -> ClutCache {
        ClutCache { colours: [0; 256], x: 0, y: 0, loaded: 0 }
    }
}

impl ClutCache {
    pub fn clear(&mut self) {
        self.loaded = 0;
    }

    /// Before a 4- or 8-bit primitive: reloaded when its position differs or it holds fewer entries than the depth
    /// needs; the entries loaded, for the drawing time.
    pub fn prepare(&mut self, vram: &Vram, t: &Texture) -> u32 {
        let need = t.entries();
        if need == 0 || (self.loaded >= need && self.x == t.clut_x && self.y == t.clut_y) {
            return 0;
        }
        for i in 0..need {
            self.colours[i as usize] = vram.get(t.clut_x + i, t.clut_y);
        }
        (self.x, self.y, self.loaded) = (t.clut_x, t.clut_y, need);
        need
    }
}

/// The texel at (u, v) with the cache's misses counted: 0000h is transparent, bit 15 the semi-transparency flag; a
/// page in the second megabyte, which no retail console fits, reads as 0000h ("textures disappearing").
#[inline]
pub fn texel(vram: &Vram, cache: &mut TextureCache, clut: &ClutCache, t: &Texture, u: u8, v: u8, misses: &mut u32) -> u16 {
    let (u, v) = t.window(u, v);
    let (u, v) = (u as u32, v as u32);
    let y = t.page_y + v;
    if y >= 512 {
        return 0;
    }
    let (half, miss) = match t.depth {
        0 => cache.read(vram, t.page_x + (u >> 2), y, 0),
        1 => cache.read(vram, t.page_x + (u >> 1), y, 1),
        _ => cache.read(vram, t.page_x + u, y, 2),
    };
    *misses += miss as u32;
    match t.depth {
        0 => clut.colours[(half >> ((u & 3) * 4) & 0xF) as usize],
        1 => clut.colours[(half >> ((u & 1) * 8) & 0xFF) as usize],
        _ => half,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_window_replaces_the_masked_bits() {
        let t = Texture::new(0, 0, 0b00010_00001_00011_00011, false);
        assert_eq!((t.mask_u, t.mask_v, t.off_u, t.off_v), (24, 24, 8, 16));
        assert_eq!(t.window(0xFF, 0x00), (0xEF, 0x10));
    }

    #[test]
    fn the_caches_keep_what_they_read() {
        let mut vram = Vram::default();
        vram.set(64, 0, 0x4321);
        vram.set(0, 7, 0x7C00);
        let mut cache = TextureCache::default();
        let mut clut = ClutCache::default();
        let t = Texture::new(1, 7 << 6, 0, false);
        assert_eq!(clut.prepare(&vram, &t), 16);
        assert_eq!(clut.prepare(&vram, &t), 0, "the same palette stays");
        let mut misses = 0;
        assert_eq!(texel(&vram, &mut cache, &clut, &t, 0, 0, &mut misses), 0x0000, "index 1 of the palette is 0");
        assert_eq!(texel(&vram, &mut cache, &clut, &t, 3, 0, &mut misses), 0x0000);
        assert_eq!(misses, 1, "one entry holds four halfwords");
        vram.set(64, 0, 0);
        let t15 = Texture::new(1 | 2 << 7, 0, 0, false);
        assert_eq!(texel(&vram, &mut cache, &clut, &t15, 0, 0, &mut misses), 0x4321, "stale until cleared");
        cache.clear();
        assert_eq!(texel(&vram, &mut cache, &clut, &t15, 0, 0, &mut misses), 0);
        assert_eq!(clut.colours[0], 0x7C00);
        let high = Texture::new(1 | 2 << 7 | 1 << 11, 0, 0, true);
        assert_eq!(texel(&vram, &mut cache, &clut, &high, 0, 0, &mut misses), 0, "the absent second megabyte");
        assert_eq!(Texture::new(1 << 11, 0, 0, false).page_y, 0, "wrapped while GP1(09h) forbids it");
    }
}
