//! The GS's local memory, 4 MiB of 32-bit words, and the one owner of its layout (GS User's Manual chapter 8): a
//! pixel's place for every storage format, from a buffer's base pointer in blocks, its width in units of 64 pixels and
//! the pixel's coordinates. Pages follow each other along the buffer's rows (§8.2); a base pointer inside a page counts
//! on in blocks, so the next page begins 32 blocks on (§8.5); addresses wrap at the memory's end. Within a page the
//! blocks, columns and words fall as §8.3's figures give, precomputed here as one table a page shape. The drawing
//! stages (S17 to S19) and the device path (S44) reach memory only through this module. Sapphire_Native.md §24.2.

use super::psm::{Blocks, Psm, Shape, block_number};

/// The memory's size in words: 4 MiB (§8.1).
pub const WORDS: usize = 1 << 20;
/// Its size in blocks of 64 words, the unit of a base pointer (§8.1).
pub const BLOCKS: u32 = 1 << 14;

/// A pixel's place: the word, and the lowest bit of the pixel's field in it; the field's width is the format's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Address {
    pub word: u32,
    pub shift: u32,
}

/// A word's index within an 8×2 column of 32-bit pixels: §8.3.1's pixel order, two pixels across, then the next two
/// below them; the 16-, 8- and 4-bit columns reuse it.
const fn column_word(x: u32, y: u32) -> u32 {
    (x >> 1) << 2 | y << 1 | (x & 1)
}

/// A pixel's offset within the 32 blocks of a page (block × 64 + column × 16 + word, 11 bits) and the part of the word
/// it takes, the sub-word index above bit 11: the half for 16-bit pixels, the byte for 8-bit and the nibble for 4-bit.
const fn entry(shape: Shape, order: Blocks, x: u32, y: u32) -> u16 {
    let (bw, bh) = shape.block();
    let ch = shape.column_height();
    let (bx, by) = (x / bw, y / bh);
    let (xb, yb) = (x % bw, y % bh);
    let block = block_number(order, bx, by);
    let column = yb / ch;
    let yc = yb % ch;
    let (word, sub) = match shape {
        Shape::Word => (column_word(xb, yc), 0),
        Shape::Half => (column_word(xb & 7, yc), xb >> 3),
        Shape::Byte | Shape::Nibble => {
            // §8.3.3 and §8.3.4: the lower two rows take the even byte or nibble, the upper two the odd, with the
            // word order turned by four pixels in rows 2 and 3 of even columns and rows 0 and 1 of odd ones.
            let turned = (yc >> 1) ^ (column & 1);
            let xc = if turned == 1 { (xb + 4) & 7 } else { xb & 7 };
            (column_word(xc, yc & 1), (xb >> 3) * 2 + (yc >> 1))
        }
    };
    ((block * 64 + column * 16 + word) | (sub << 11)) as u16
}

const fn table<const N: usize>(shape: Shape, order: Blocks) -> [u16; N] {
    let (pw, _) = shape.page();
    let mut t = [0u16; N];
    let mut i = 0;
    while i < N {
        t[i] = entry(shape, order, i as u32 % pw, i as u32 / pw);
        i += 1;
    }
    t
}

static CT32: [u16; 64 * 32] = table(Shape::Word, Blocks::Ct32);
static Z32: [u16; 64 * 32] = table(Shape::Word, Blocks::Z32);
static CT16: [u16; 64 * 64] = table(Shape::Half, Blocks::Ct16);
static CT16S: [u16; 64 * 64] = table(Shape::Half, Blocks::Ct16s);
static Z16: [u16; 64 * 64] = table(Shape::Half, Blocks::Z16);
static Z16S: [u16; 64 * 64] = table(Shape::Half, Blocks::Z16s);
static T8: [u16; 128 * 64] = table(Shape::Byte, Blocks::Ct32);
static T4: [u16; 128 * 128] = table(Shape::Nibble, Blocks::Ct16);

/// A format's page table: a pixel's entry by its place in the page, row by row.
pub fn page_table(psm: Psm) -> &'static [u16] {
    match psm {
        Psm::Ct32 | Psm::Ct24 | Psm::T8h | Psm::T4hl | Psm::T4hh => &CT32,
        Psm::Z32 | Psm::Z24 => &Z32,
        Psm::Ct16 => &CT16,
        Psm::Ct16s => &CT16S,
        Psm::Z16 => &Z16,
        Psm::Z16s => &Z16S,
        Psm::T8 => &T8,
        Psm::T4 => &T4,
    }
}

/// Pixel (`x`, `y`) of the buffer at base pointer `bp` (in blocks) and width `bw` (in units of 64 pixels) in format
/// `psm`. The pages in a row are the width over the page's width, as a whole number (§8.2; a PSMT8 or PSMT4 buffer of
/// odd width is outside §2.6's rule, Sapphire_Disputes.md D-104).
pub fn address(psm: Psm, bp: u32, bw: u32, x: u32, y: u32) -> Address {
    let (pw, ph) = psm.shape().page();
    let per_row = bw * 64 / pw;
    let page = (y / ph) * per_row + x / pw;
    let e = u32::from(page_table(psm)[((y % ph) * pw + x % pw) as usize]);
    let word = ((bp + page * 32) * 64 + (e & 0x7FF)) & (WORDS as u32 - 1);
    let shift = match psm.shape() {
        Shape::Word => psm.high_shift(),
        Shape::Half => 16 * (e >> 11),
        Shape::Byte => 8 * (e >> 11),
        Shape::Nibble => 4 * (e >> 11),
    };
    Address { word, shift }
}

const fn mask(bits: u32) -> u32 {
    if bits >= 32 { u32::MAX } else { (1 << bits) - 1 }
}

/// The memory's size in pages of 2,048 words (§8.1), the unit the drawing's workers own (Sapphire_Native.md §50).
pub const PAGES: usize = WORDS >> 11;

/// A set of the memory's physical pages.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Pages(pub [u64; PAGES / 64]);

impl Pages {
    pub const ALL: Pages = Pages([u64::MAX; PAGES / 64]);

    pub fn add(&mut self, p: usize) {
        self.0[p >> 6] |= 1 << (p & 63);
    }

    pub fn has(&self, p: usize) -> bool {
        self.0[p >> 6] >> (p & 63) & 1 != 0
    }

    pub fn or(&mut self, o: &Pages) {
        for (a, b) in self.0.iter_mut().zip(o.0) {
            *a |= b;
        }
    }

    pub fn meets(&self, o: &Pages) -> bool {
        self.0.iter().zip(o.0).any(|(a, b)| a & b != 0)
    }

    pub fn is_empty(&self) -> bool {
        self.0.iter().all(|&w| w == 0)
    }

    /// Each page in the set, in order.
    pub fn each(&self) -> impl Iterator<Item = usize> + '_ {
        self.0.iter().enumerate().flat_map(|(i, &w)| {
            let mut w = w;
            std::iter::from_fn(move || {
                (w != 0).then(|| {
                    let b = w.trailing_zeros() as usize;
                    w &= w - 1;
                    i * 64 + b
                })
            })
        })
    }
}

/// The page cells, by row, of the pixels from (`x0`, `y0`) to (`x1`, `y1`) inclusive in a buffer: each cell's row and the
/// physical pages its pixels lie in, which are one, or two when the base pointer lies inside a page. Every pixel of a cell
/// takes the same `page` in `address`, so its word lies in [start, start + 2047] with start that page's first word.
pub fn cells(psm: Psm, bp: u32, bw: u32, (x0, x1): (u32, u32), (y0, y1): (u32, u32), mut f: impl FnMut(u32, usize)) {
    let (pw, ph) = psm.shape().page();
    let per_row = bw * 64 / pw;
    for cy in y0 / ph..=y1 / ph {
        for cx in x0 / pw..=x1 / pw {
            let start = ((bp + (cy * per_row + cx) * 32) * 64) & (WORDS as u32 - 1);
            f(cy, (start >> 11) as usize);
            if bp % 32 != 0 {
                f(cy, ((start >> 11) as usize + 1) % PAGES);
            }
        }
    }
}

/// The pages a rectangle's pixels lie in; a rectangle of more than 1,024 cells is taken as the whole memory.
pub fn pages(psm: Psm, bp: u32, bw: u32, xs: (u32, u32), ys: (u32, u32)) -> Pages {
    let (pw, ph) = psm.shape().page();
    if u64::from(xs.1 / pw - xs.0 / pw + 1) * u64::from(ys.1 / ph - ys.0 / ph + 1) > 1024 {
        return Pages::ALL;
    }
    let mut s = Pages::default();
    cells(psm, bp, bw, xs, ys, |_, p| s.add(p));
    s
}

/// The 4 MiB, one allocation reached a word at a time: while the drawing's workers run, each word is touched only
/// through a raw pointer, never through a reference to the whole memory (Sapphire_Native.md §50). A view shares the
/// owner's allocation and frees nothing.
pub struct Local {
    ptr: std::ptr::NonNull<u32>,
    owned: bool,
}

// The allocation is the owner's alone; views are made only by the workers, which the owner outlives (gs::workers).
unsafe impl Send for Local {}

impl Default for Local {
    fn default() -> Local {
        let b: Box<[u32]> = vec![0; WORDS].into_boxed_slice();
        let ptr = std::ptr::NonNull::new(Box::into_raw(b) as *mut u32).expect("an allocation");
        Local { ptr, owned: true }
    }
}

impl Drop for Local {
    fn drop(&mut self) {
        if self.owned {
            // SAFETY: made by `default` from a boxed slice of WORDS words, and freed once, by its owner.
            drop(unsafe { Box::from_raw(std::ptr::slice_from_raw_parts_mut(self.ptr.as_ptr(), WORDS)) });
        }
    }
}

impl Clone for Local {
    fn clone(&self) -> Local {
        let c = Local::default();
        // SAFETY: both allocations hold WORDS words and are distinct.
        unsafe { std::ptr::copy_nonoverlapping(self.ptr.as_ptr(), c.ptr.as_ptr(), WORDS) };
        c
    }
}

impl PartialEq for Local {
    fn eq(&self, o: &Local) -> bool {
        self.words() == o.words()
    }
}

impl Eq for Local {}

impl Local {
    /// A view of the same memory for a worker; the owner must outlive it.
    pub(super) fn view(&self) -> Local {
        Local { ptr: self.ptr, owned: false }
    }

    /// Word `i`, which must be below WORDS.
    #[inline]
    pub fn word(&self, i: usize) -> u32 {
        assert!(i < WORDS);
        // SAFETY: in bounds; one word, read through the raw pointer.
        unsafe { self.ptr.as_ptr().add(i).read() }
    }

    #[inline]
    pub fn set_word(&mut self, i: usize, v: u32) {
        assert!(i < WORDS);
        // SAFETY: in bounds; one word, written through the raw pointer.
        unsafe { self.ptr.as_ptr().add(i).write(v) }
    }

    /// The pixel's value, its format's bits.
    #[inline]
    pub fn read(&self, psm: Psm, bp: u32, bw: u32, x: u32, y: u32) -> u32 {
        let a = address(psm, bp, bw, x, y);
        // SAFETY: `address` masks the word below WORDS.
        (unsafe { self.ptr.as_ptr().add(a.word as usize).read() } >> a.shift) & mask(psm.bits())
    }

    /// Writes the pixel's bits, leaving the rest of its word (PSMCT24's upper byte, the other half, byte or nibble).
    #[inline]
    pub fn write(&mut self, psm: Psm, bp: u32, bw: u32, x: u32, y: u32, v: u32) {
        let a = address(psm, bp, bw, x, y);
        let m = mask(psm.bits()) << a.shift;
        // SAFETY: `address` masks the word below WORDS.
        unsafe {
            let w = self.ptr.as_ptr().add(a.word as usize);
            w.write((w.read() & !m) | ((v << a.shift) & m));
        }
    }

    /// The whole memory, for a state, a debugger or a test: only while no worker draws (`Gs::local` joins them first).
    pub fn words(&self) -> &[u32] {
        // SAFETY: WORDS words from one allocation; the owner's callers hold no worker running.
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), WORDS) }
    }

    pub(super) fn words_mut(&mut self) -> &mut [u32] {
        // SAFETY: as `words`, and `&mut self` is the owner's.
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), WORDS) }
    }

    /// The memory as bytes, little-endian, as a debugger or the state shows it.
    pub fn bytes(&self) -> Vec<u8> {
        self.words().iter().flat_map(|w| w.to_le_bytes()).collect()
    }
}

impl std::fmt::Debug for Local {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Local({} words)", WORDS)
    }
}
