//! The storage formats (GS User's Manual §2.3 to §2.6, the PSM codes of §7.1's BITBLTBUF, and chapter 8's units): each
//! format's code, the bits a pixel takes in transfer data (§4.3), and the geometry of its pages, blocks and columns.
//! `local` owns the address arithmetic built on these. Sapphire_Native.md §24.2.

/// A pixel storage format, by the names of §7.1.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Psm {
    Ct32,
    Ct24,
    Ct16,
    Ct16s,
    T8,
    T4,
    T8h,
    T4hl,
    T4hh,
    Z32,
    Z24,
    Z16,
    Z16s,
}

/// Every format, in the order of BITBLTBUF's table.
pub const ALL: [Psm; 13] = [Psm::Ct32, Psm::Ct24, Psm::Ct16, Psm::Ct16s, Psm::T8, Psm::T4, Psm::T8h, Psm::T4hl, Psm::T4hh, Psm::Z32, Psm::Z24, Psm::Z16, Psm::Z16s];

/// The page, block and column shapes one family of formats shares (§8.1, §8.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    /// PSMCT32, PSMCT24, PSMZ32, PSMZ24 and the H formats: page 64×32, block 8×8, column 8×2.
    Word,
    /// The 16-bit formats: page 64×64, block 16×8, column 16×2.
    Half,
    /// PSMT8: page 128×64, block 16×16, column 16×4.
    Byte,
    /// PSMT4: page 128×128, block 32×16, column 32×4.
    Nibble,
}

impl Shape {
    /// The page's width and height in pixels.
    pub const fn page(self) -> (u32, u32) {
        match self {
            Shape::Word => (64, 32),
            Shape::Half => (64, 64),
            Shape::Byte => (128, 64),
            Shape::Nibble => (128, 128),
        }
    }

    /// The block's width and height in pixels.
    pub const fn block(self) -> (u32, u32) {
        match self {
            Shape::Word => (8, 8),
            Shape::Half => (16, 8),
            Shape::Byte => (16, 16),
            Shape::Nibble => (32, 16),
        }
    }

    /// The column's height in pixels; its width is the block's.
    pub const fn column_height(self) -> u32 {
        match self {
            Shape::Word | Shape::Half => 2,
            Shape::Byte | Shape::Nibble => 4,
        }
    }
}

/// Which block order a format's pages use (§8.3's figures).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Blocks {
    Ct32,
    Z32,
    Ct16,
    Ct16s,
    Z16,
    Z16s,
}

impl Psm {
    pub const fn from_code(code: u32) -> Option<Psm> {
        Some(match code & 0x3F {
            0x00 => Psm::Ct32,
            0x01 => Psm::Ct24,
            0x02 => Psm::Ct16,
            0x0A => Psm::Ct16s,
            0x13 => Psm::T8,
            0x14 => Psm::T4,
            0x1B => Psm::T8h,
            0x24 => Psm::T4hl,
            0x2C => Psm::T4hh,
            0x30 => Psm::Z32,
            0x31 => Psm::Z24,
            0x32 => Psm::Z16,
            0x3A => Psm::Z16s,
            _ => return None,
        })
    }

    pub const fn code(self) -> u32 {
        match self {
            Psm::Ct32 => 0x00,
            Psm::Ct24 => 0x01,
            Psm::Ct16 => 0x02,
            Psm::Ct16s => 0x0A,
            Psm::T8 => 0x13,
            Psm::T4 => 0x14,
            Psm::T8h => 0x1B,
            Psm::T4hl => 0x24,
            Psm::T4hh => 0x2C,
            Psm::Z32 => 0x30,
            Psm::Z24 => 0x31,
            Psm::Z16 => 0x32,
            Psm::Z16s => 0x3A,
        }
    }

    /// The bits one pixel takes in transfer data (§4.3), which is also the bits it holds in memory.
    pub const fn bits(self) -> u32 {
        match self {
            Psm::Ct32 | Psm::Z32 => 32,
            Psm::Ct24 | Psm::Z24 => 24,
            Psm::Ct16 | Psm::Ct16s | Psm::Z16 | Psm::Z16s => 16,
            Psm::T8 | Psm::T8h => 8,
            Psm::T4 | Psm::T4hl | Psm::T4hh => 4,
        }
    }

    pub const fn shape(self) -> Shape {
        match self {
            Psm::Ct32 | Psm::Ct24 | Psm::Z32 | Psm::Z24 | Psm::T8h | Psm::T4hl | Psm::T4hh => Shape::Word,
            Psm::Ct16 | Psm::Ct16s | Psm::Z16 | Psm::Z16s => Shape::Half,
            Psm::T8 => Shape::Byte,
            Psm::T4 => Shape::Nibble,
        }
    }

    pub const fn blocks(self) -> Blocks {
        match self {
            Psm::Ct32 | Psm::Ct24 | Psm::T8h | Psm::T4hl | Psm::T4hh | Psm::T8 => Blocks::Ct32,
            Psm::Z32 | Psm::Z24 => Blocks::Z32,
            Psm::Ct16 | Psm::T4 => Blocks::Ct16,
            Psm::Ct16s => Blocks::Ct16s,
            Psm::Z16 => Blocks::Z16,
            Psm::Z16s => Blocks::Z16s,
        }
    }

    /// For the formats that keep part of a word in the 32-bit layout (§2.4.5, §2.6.3): the lowest bit they hold.
    pub const fn high_shift(self) -> u32 {
        match self {
            Psm::T8h | Psm::T4hl => 24,
            Psm::T4hh => 28,
            _ => 0,
        }
    }
}

/// A block's number from its place in the page, by §8.3's figures read as the interleaving of the block's coordinates'
/// bits (the figures themselves are transcribed in the tests).
pub const fn block_number(order: Blocks, bx: u32, by: u32) -> u32 {
    match order {
        Blocks::Ct32 => (bx & 1) | (by & 1) << 1 | (bx & 2) << 1 | (by & 2) << 2 | (bx & 4) << 2,
        Blocks::Z32 => block_number(Blocks::Ct32, bx, by) ^ 0x18,
        Blocks::Ct16 => (by & 1) | (bx & 1) << 1 | (by & 2) << 1 | (bx & 2) << 2 | (by & 4) << 2,
        Blocks::Ct16s => (by & 1) | (bx & 1) << 1 | (by & 4) | (by & 2) << 2 | (bx & 2) << 3,
        Blocks::Z16 => block_number(Blocks::Ct16, bx, by) ^ 0x18,
        Blocks::Z16s => block_number(Blocks::Ct16s, bx, by) ^ 0x18,
    }
}
