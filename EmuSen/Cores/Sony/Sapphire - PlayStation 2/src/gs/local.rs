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
    (block * 64 + column * 16 + word | sub << 11) as u16
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

/// The 4 MiB.
#[derive(Clone, PartialEq, Eq)]
pub struct Local {
    pub words: Vec<u32>,
}

impl Default for Local {
    fn default() -> Local {
        Local { words: vec![0; WORDS] }
    }
}

impl Local {
    /// The pixel's value, its format's bits.
    pub fn read(&self, psm: Psm, bp: u32, bw: u32, x: u32, y: u32) -> u32 {
        let a = address(psm, bp, bw, x, y);
        (self.words[a.word as usize] >> a.shift) & mask(psm.bits())
    }

    /// Writes the pixel's bits, leaving the rest of its word (PSMCT24's upper byte, the other half, byte or nibble).
    pub fn write(&mut self, psm: Psm, bp: u32, bw: u32, x: u32, y: u32, v: u32) {
        let a = address(psm, bp, bw, x, y);
        let m = mask(psm.bits()) << a.shift;
        let w = &mut self.words[a.word as usize];
        *w = (*w & !m) | ((v << a.shift) & m);
    }

    /// The memory as bytes, little-endian, as a debugger or the state shows it.
    pub fn bytes(&self) -> Vec<u8> {
        self.words.iter().flat_map(|w| w.to_le_bytes()).collect()
    }
}

impl std::fmt::Debug for Local {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Local({} words)", self.words.len())
    }
}
