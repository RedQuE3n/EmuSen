//! The variable-length codes of ITU-T H.262 (02/2000) Annex B, as the standard prints them (Tables B.1 to B.4 and B.9
//! to B.15), and the decoder the IPU reads them with: each table is grouped by its codes' leading zeros and looked up
//! by the bits after the first one, so a code costs a count, a shift and an index. The tables were taken from the
//! standard's text and checked to be prefix-free and, where the standard leaves no code unused, complete
//! (`every_table_is_prefix_free_and_as_complete_as_the_standard`). Sapphire_Native.md §34.3.

use std::sync::OnceLock;

/// macroblock_address_increment's escape and ISO/IEC 11172-2's stuffing, as VDEC reports them (EE User's Manual
/// §8.5.3: 00000023h and 00000022h after the length).
pub const ESCAPE: u8 = 35;
pub const STUFFING: u8 = 34;
/// The stuffing code, '0000 0001 111', which only an MPEG-1 stream may carry (H.262 §D.9.2).
const STUFFING_CODE: &str = "0000 0001 111";

/// A DCT coefficient table's End of Block and Escape rows, and Table B.14's two codes for run 0, level 1: Note 3's for
/// the first coefficient of a non-intra block, Note 4's for every other.
const EOB: u8 = 0xFF;
const ESC: u8 = 0xFE;
const FIRST: u8 = 0x80;
const LATER: u8 = 0x40;

/// Table B.1, macroblock_address_increment.
const MBAI: &[(&str, u8)] = &[
    ("1", 1), ("0000 0101 01", 18), ("011", 2), ("0000 0101 00", 19),
    ("010", 3), ("0000 0100 11", 20), ("0011", 4), ("0000 0100 10", 21),
    ("0010", 5), ("0000 0100 011", 22), ("0001 1", 6), ("0000 0100 010", 23),
    ("0001 0", 7), ("0000 0100 001", 24), ("0000 111", 8), ("0000 0100 000", 25),
    ("0000 110", 9), ("0000 0011 111", 26), ("0000 1011", 10), ("0000 0011 110", 27),
    ("0000 1010", 11), ("0000 0011 101", 28), ("0000 1001", 12), ("0000 0011 100", 29),
    ("0000 1000", 13), ("0000 0011 011", 30), ("0000 0111", 14), ("0000 0011 010", 31),
    ("0000 0110", 15), ("0000 0011 001", 32), ("0000 0101 11", 16), ("0000 0011 000", 33),
    ("0000 0101 10", 17), ("0000 0001 000", ESCAPE),
];
/// Tables B.2 to B.4, macroblock_type in I-, P- and B-pictures, as `mb`'s bits.
const MB_TYPE_I: &[(&str, u8)] = &[
    ("1", 1), ("01", 17),
];
const MB_TYPE_P: &[(&str, u8)] = &[
    ("1", 10), ("01", 2), ("001", 8), ("0001 1", 1),
    ("0001 0", 26), ("0000 1", 18), ("0000 01", 17),
];
const MB_TYPE_B: &[(&str, u8)] = &[
    ("10", 12), ("11", 14), ("010", 4), ("011", 6),
    ("0010", 8), ("0011", 10), ("0001 1", 1), ("0001 0", 30),
    ("0000 11", 26), ("0000 10", 22), ("0000 01", 17),
];
/// Table B.9, coded_block_pattern.
const CBP: &[(&str, u8)] = &[
    ("111", 60), ("0001 1100", 35), ("1101", 4), ("0001 1011", 13),
    ("1100", 8), ("0001 1010", 49), ("1011", 16), ("0001 1001", 21),
    ("1010", 32), ("0001 1000", 41), ("1001 1", 12), ("0001 0111", 14),
    ("1001 0", 48), ("0001 0110", 50), ("1000 1", 20), ("0001 0101", 22),
    ("1000 0", 40), ("0001 0100", 42), ("0111 1", 28), ("0001 0011", 15),
    ("0111 0", 44), ("0001 0010", 51), ("0110 1", 52), ("0001 0001", 23),
    ("0110 0", 56), ("0001 0000", 43), ("0101 1", 1), ("0000 1111", 25),
    ("0101 0", 61), ("0000 1110", 37), ("0100 1", 2), ("0000 1101", 26),
    ("0100 0", 62), ("0000 1100", 38), ("0011 11", 24), ("0000 1011", 29),
    ("0011 10", 36), ("0000 1010", 45), ("0011 01", 3), ("0000 1001", 53),
    ("0011 00", 63), ("0000 1000", 57), ("0010 111", 5), ("0000 0111", 30),
    ("0010 110", 9), ("0000 0110", 46), ("0010 101", 17), ("0000 0101", 54),
    ("0010 100", 33), ("0000 0100", 58), ("0010 011", 6), ("0000 0011 1", 31),
    ("0010 010", 10), ("0000 0011 0", 47), ("0010 001", 18), ("0000 0010 1", 55),
    ("0010 000", 34), ("0000 0010 0", 59), ("0001 1111", 7), ("0000 0001 1", 27),
    ("0001 1110", 11), ("0000 0001 0", 39), ("0001 1101", 19), ("0000 0000 1", 0),
];
/// Table B.10, motion_code.
const MOTION_CODE: &[(&str, i8)] = &[
    ("0000 0011 001", -16), ("0000 0011 011", -15), ("0000 0011 101", -14), ("0000 0011 111", -13),
    ("0000 0100 001", -12), ("0000 0100 011", -11), ("0000 0100 11", -10), ("0000 0101 01", -9),
    ("0000 0101 11", -8), ("0000 0111", -7), ("0000 1001", -6), ("0000 1011", -5),
    ("0000 111", -4), ("0001 1", -3), ("0011", -2), ("011", -1),
    ("1", 0), ("010", 1), ("0010", 2), ("0001 0", 3),
    ("0000 110", 4), ("0000 1010", 5), ("0000 1000", 6), ("0000 0110", 7),
    ("0000 0101 10", 8), ("0000 0101 00", 9), ("0000 0100 10", 10), ("0000 0100 010", 11),
    ("0000 0100 000", 12), ("0000 0011 110", 13), ("0000 0011 100", 14), ("0000 0011 010", 15),
    ("0000 0011 000", 16),
];
/// Table B.11, dmvector.
const DMVECTOR: &[(&str, i8)] = &[
    ("11", -1), ("0", 0), ("10", 1),
];
/// Tables B.12 and B.13, dct_dc_size_luminance and dct_dc_size_chrominance.
const DC_SIZE_LUMINANCE: &[(&str, u8)] = &[
    ("100", 0), ("00", 1), ("01", 2), ("101", 3),
    ("110", 4), ("1110", 5), ("1111 0", 6), ("1111 10", 7),
    ("1111 110", 8), ("1111 1110", 9), ("1111 1111 0", 10), ("1111 1111 1", 11),
];
const DC_SIZE_CHROMINANCE: &[(&str, u8)] = &[
    ("00", 0), ("01", 1), ("10", 2), ("110", 3),
    ("1110", 4), ("1111 0", 5), ("1111 10", 6), ("1111 110", 7),
    ("1111 1110", 8), ("1111 1111 0", 9), ("1111 1111 10", 10), ("1111 1111 11", 11),
];
/// Table B.14, DCT coefficients Table zero: the code without its sign bit, the run, the level.
const DCT_ZERO: &[(&str, u8, u8)] = &[
    ("10", EOB, 0), ("1", 0 | FIRST, 1), ("11", 0 | LATER, 1),
    ("011", 1, 1), ("0100", 0, 2), ("0101", 2, 1),
    ("0010 1", 0, 3), ("0011 1", 3, 1), ("0011 0", 4, 1),
    ("0001 10", 1, 2), ("0001 11", 5, 1), ("0001 01", 6, 1),
    ("0001 00", 7, 1), ("0000 110", 0, 4), ("0000 100", 2, 2),
    ("0000 111", 8, 1), ("0000 101", 9, 1), ("0000 01", ESC, 0),
    ("0010 0110", 0, 5), ("0010 0001", 0, 6), ("0010 0101", 1, 3),
    ("0010 0100", 3, 2), ("0010 0111", 10, 1), ("0010 0011", 11, 1),
    ("0010 0010", 12, 1), ("0010 0000", 13, 1), ("0000 0010 10", 0, 7),
    ("0000 0011 00", 1, 4), ("0000 0010 11", 2, 3), ("0000 0011 11", 4, 2),
    ("0000 0010 01", 5, 2), ("0000 0011 10", 14, 1), ("0000 0011 01", 15, 1),
    ("0000 0010 00", 16, 1), ("0000 0001 1101", 0, 8), ("0000 0001 1000", 0, 9),
    ("0000 0001 0011", 0, 10), ("0000 0001 0000", 0, 11), ("0000 0001 1011", 1, 5),
    ("0000 0001 0100", 2, 4), ("0000 0001 1100", 3, 3), ("0000 0001 0010", 4, 3),
    ("0000 0001 1110", 6, 2), ("0000 0001 0101", 7, 2), ("0000 0001 0001", 8, 2),
    ("0000 0001 1111", 17, 1), ("0000 0001 1010", 18, 1), ("0000 0001 1001", 19, 1),
    ("0000 0001 0111", 20, 1), ("0000 0001 0110", 21, 1), ("0000 0000 1101 0", 0, 12),
    ("0000 0000 1100 1", 0, 13), ("0000 0000 1100 0", 0, 14), ("0000 0000 1011 1", 0, 15),
    ("0000 0000 1011 0", 1, 6), ("0000 0000 1010 1", 1, 7), ("0000 0000 1010 0", 2, 5),
    ("0000 0000 1001 1", 3, 4), ("0000 0000 1001 0", 5, 3), ("0000 0000 1000 1", 9, 2),
    ("0000 0000 1000 0", 10, 2), ("0000 0000 1111 1", 22, 1), ("0000 0000 1111 0", 23, 1),
    ("0000 0000 1110 1", 24, 1), ("0000 0000 1110 0", 25, 1), ("0000 0000 1101 1", 26, 1),
    ("0000 0000 0111 11", 0, 16), ("0000 0000 0111 10", 0, 17), ("0000 0000 0111 01", 0, 18),
    ("0000 0000 0111 00", 0, 19), ("0000 0000 0110 11", 0, 20), ("0000 0000 0110 10", 0, 21),
    ("0000 0000 0110 01", 0, 22), ("0000 0000 0110 00", 0, 23), ("0000 0000 0101 11", 0, 24),
    ("0000 0000 0101 10", 0, 25), ("0000 0000 0101 01", 0, 26), ("0000 0000 0101 00", 0, 27),
    ("0000 0000 0100 11", 0, 28), ("0000 0000 0100 10", 0, 29), ("0000 0000 0100 01", 0, 30),
    ("0000 0000 0100 00", 0, 31), ("0000 0000 0011 000", 0, 32), ("0000 0000 0010 111", 0, 33),
    ("0000 0000 0010 110", 0, 34), ("0000 0000 0010 101", 0, 35), ("0000 0000 0010 100", 0, 36),
    ("0000 0000 0010 011", 0, 37), ("0000 0000 0010 010", 0, 38), ("0000 0000 0010 001", 0, 39),
    ("0000 0000 0010 000", 0, 40), ("0000 0000 0011 111", 1, 8), ("0000 0000 0011 110", 1, 9),
    ("0000 0000 0011 101", 1, 10), ("0000 0000 0011 100", 1, 11), ("0000 0000 0011 011", 1, 12),
    ("0000 0000 0011 010", 1, 13), ("0000 0000 0011 001", 1, 14), ("0000 0000 0001 0011", 1, 15),
    ("0000 0000 0001 0010", 1, 16), ("0000 0000 0001 0001", 1, 17), ("0000 0000 0001 0000", 1, 18),
    ("0000 0000 0001 0100", 6, 3), ("0000 0000 0001 1010", 11, 2), ("0000 0000 0001 1001", 12, 2),
    ("0000 0000 0001 1000", 13, 2), ("0000 0000 0001 0111", 14, 2), ("0000 0000 0001 0110", 15, 2),
    ("0000 0000 0001 0101", 16, 2), ("0000 0000 0001 1111", 27, 1), ("0000 0000 0001 1110", 28, 1),
    ("0000 0000 0001 1101", 29, 1), ("0000 0000 0001 1100", 30, 1), ("0000 0000 0001 1011", 31, 1),
];
/// Table B.15, DCT coefficients Table one.
const DCT_ONE: &[(&str, u8, u8)] = &[
    ("0110", EOB, 0), ("10", 0, 1), ("010", 1, 1),
    ("110", 0, 2), ("0010 1", 2, 1), ("0111", 0, 3),
    ("0011 1", 3, 1), ("0001 10", 4, 1), ("0011 0", 1, 2),
    ("0001 11", 5, 1), ("0000 110", 6, 1), ("0000 100", 7, 1),
    ("1110 0", 0, 4), ("0000 111", 2, 2), ("0000 101", 8, 1),
    ("1111 000", 9, 1), ("0000 01", ESC, 0), ("1110 1", 0, 5),
    ("0001 01", 0, 6), ("1111 001", 1, 3), ("0010 0110", 3, 2),
    ("1111 010", 10, 1), ("0010 0001", 11, 1), ("0010 0101", 12, 1),
    ("0010 0100", 13, 1), ("0001 00", 0, 7), ("0010 0111", 1, 4),
    ("1111 1100", 2, 3), ("1111 1101", 4, 2), ("0000 0010 0", 5, 2),
    ("0000 0010 1", 14, 1), ("0000 0011 1", 15, 1), ("0000 0011 01", 16, 1),
    ("1111 011", 0, 8), ("1111 100", 0, 9), ("0010 0011", 0, 10),
    ("0010 0010", 0, 11), ("0010 0000", 1, 5), ("0000 0011 00", 2, 4),
    ("0000 0001 1100", 3, 3), ("0000 0001 0010", 4, 3), ("0000 0001 1110", 6, 2),
    ("0000 0001 0101", 7, 2), ("0000 0001 0001", 8, 2), ("0000 0001 1111", 17, 1),
    ("0000 0001 1010", 18, 1), ("0000 0001 1001", 19, 1), ("0000 0001 0111", 20, 1),
    ("0000 0001 0110", 21, 1), ("1111 1010", 0, 12), ("1111 1011", 0, 13),
    ("1111 1110", 0, 14), ("1111 1111", 0, 15), ("0000 0000 1011 0", 1, 6),
    ("0000 0000 1010 1", 1, 7), ("0000 0000 1010 0", 2, 5), ("0000 0000 1001 1", 3, 4),
    ("0000 0000 1001 0", 5, 3), ("0000 0000 1000 1", 9, 2), ("0000 0000 1000 0", 10, 2),
    ("0000 0000 1111 1", 22, 1), ("0000 0000 1111 0", 23, 1), ("0000 0000 1110 1", 24, 1),
    ("0000 0000 1110 0", 25, 1), ("0000 0000 1101 1", 26, 1), ("0000 0000 0111 11", 0, 16),
    ("0000 0000 0111 10", 0, 17), ("0000 0000 0111 01", 0, 18), ("0000 0000 0111 00", 0, 19),
    ("0000 0000 0110 11", 0, 20), ("0000 0000 0110 10", 0, 21), ("0000 0000 0110 01", 0, 22),
    ("0000 0000 0110 00", 0, 23), ("0000 0000 0101 11", 0, 24), ("0000 0000 0101 10", 0, 25),
    ("0000 0000 0101 01", 0, 26), ("0000 0000 0101 00", 0, 27), ("0000 0000 0100 11", 0, 28),
    ("0000 0000 0100 10", 0, 29), ("0000 0000 0100 01", 0, 30), ("0000 0000 0100 00", 0, 31),
    ("0000 0000 0011 000", 0, 32), ("0000 0000 0010 111", 0, 33), ("0000 0000 0010 110", 0, 34),
    ("0000 0000 0010 101", 0, 35), ("0000 0000 0010 100", 0, 36), ("0000 0000 0010 011", 0, 37),
    ("0000 0000 0010 010", 0, 38), ("0000 0000 0010 001", 0, 39), ("0000 0000 0010 000", 0, 40),
    ("0000 0000 0011 111", 1, 8), ("0000 0000 0011 110", 1, 9), ("0000 0000 0011 101", 1, 10),
    ("0000 0000 0011 100", 1, 11), ("0000 0000 0011 011", 1, 12), ("0000 0000 0011 010", 1, 13),
    ("0000 0000 0011 001", 1, 14), ("0000 0000 0001 0011", 1, 15), ("0000 0000 0001 0010", 1, 16),
    ("0000 0000 0001 0001", 1, 17), ("0000 0000 0001 0000", 1, 18), ("0000 0000 0001 0100", 6, 3),
    ("0000 0000 0001 1010", 11, 2), ("0000 0000 0001 1001", 12, 2), ("0000 0000 0001 1000", 13, 2),
    ("0000 0000 0001 0111", 14, 2), ("0000 0000 0001 0110", 15, 2), ("0000 0000 0001 0101", 16, 2),
    ("0000 0000 0001 1111", 27, 1), ("0000 0000 0001 1110", 28, 1), ("0000 0000 0001 1101", 29, 1),
    ("0000 0000 0001 1100", 30, 1), ("0000 0000 0001 1011", 31, 1),
];

/// A DCT coefficient code: End of Block, Escape, or a run and a level whose sign bit follows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Coef {
    Eob,
    Escape,
    RunLevel(u8, u8),
}

/// Codes with the same number of leading zeros, indexed by the `k` bits after their first one.
#[derive(Clone, Debug, Default)]
struct Group<V: Copy> {
    k: u32,
    slots: Vec<Option<(u8, V)>>,
}

/// A table of codes up to 16 bits long, read from the next 32 bits of the stream; a code of zeros alone (Table B.12's
/// '00') stands in every group of as many leading zeros or more, which no other code of its table can begin with.
#[derive(Clone, Debug)]
pub struct Table<V: Copy> {
    groups: [Group<V>; 17],
}

impl<V: Copy> Table<V> {
    fn new(entries: impl IntoIterator<Item = (&'static str, V)>) -> Table<V> {
        let codes: Vec<(String, V)> = entries.into_iter().map(|(c, v)| (c.replace(' ', ""), v)).collect();
        let mut groups: [Group<V>; 17] = std::array::from_fn(|_| Group { k: 0, slots: Vec::new() });
        for (c, _) in &codes {
            if let Some(z) = c.find('1') {
                groups[z].k = groups[z].k.max((c.len() - z - 1) as u32);
            }
        }
        for g in &mut groups {
            g.slots = vec![None; 1 << g.k];
        }
        for (c, v) in codes {
            let Some(z) = c.find('1') else {
                for g in &mut groups[c.len()..] {
                    assert!(g.k == 0 && g.slots[0].is_none(), "code {c} is a prefix of another");
                    g.slots[0] = Some((c.len() as u8, v));
                }
                continue;
            };
            let g = &mut groups[z];
            let rest = &c[z + 1..];
            let r = rest.len() as u32;
            let base = if r == 0 { 0 } else { usize::from_str_radix(rest, 2).unwrap() } << (g.k - r);
            for slot in &mut g.slots[base..base + (1 << (g.k - r))] {
                assert!(slot.is_none(), "code {c} is a prefix of another");
                *slot = Some((c.len() as u8, v));
            }
        }
        Table { groups }
    }

    /// The code at the head of `bits` (the stream's next 32 bits, first bit highest): its length and value, or `None`
    /// for bits no code of the table begins.
    #[inline]
    pub fn decode(&self, bits: u32) -> Option<(u8, V)> {
        let z = bits.leading_zeros().min(16);
        let g = &self.groups[z as usize];
        if g.slots.is_empty() {
            return None;
        }
        let rest = if z >= 31 { 0 } else { bits << (z + 1) };
        let idx = if g.k == 0 { 0 } else { (rest >> (32 - g.k)) as usize };
        g.slots[idx]
    }
}

/// Every table the IPU decodes with.
#[derive(Debug)]
pub struct Tables {
    pub mbai: Table<u8>,
    /// Table B.1 with ISO/IEC 11172-2's stuffing code, read when IPU_CTRL.MP1 is set.
    pub mbai_mpeg1: Table<u8>,
    /// Tables B.2, B.3 and B.4, and the D-picture's one code (H.262 §D.9.6), by picture_coding_type 1 to 4: each value
    /// macroblock_quant, _motion_forward, _motion_backward, _pattern and _intra from bit 4 down.
    pub mb_type: [Table<u8>; 4],
    pub cbp: Table<u8>,
    pub motion_code: Table<i8>,
    pub dmvector: Table<i8>,
    pub dc_size: [Table<u8>; 2],
    /// Table B.14 for a non-intra block's first coefficient (Note 3), for every other coefficient, and Table B.15.
    pub dct_first: Table<Coef>,
    pub dct_zero: Table<Coef>,
    pub dct_one: Table<Coef>,
}

/// macroblock_type's bits as `Tables::mb_type` gives them.
pub mod mb {
    pub const INTRA: u8 = 1;
    pub const PATTERN: u8 = 2;
    pub const BACKWARD: u8 = 4;
    pub const FORWARD: u8 = 8;
    pub const QUANT: u8 = 16;
}

/// Table B.14's rows for a non-intra block's first coefficient: all but End of Block and Note 4's code.
fn in_first(r: u8) -> bool {
    r == ESC || (r != EOB && r & LATER == 0)
}

/// Table B.14's rows for every other coefficient: all but Note 3's code.
fn in_later(r: u8) -> bool {
    r == EOB || r == ESC || r & FIRST == 0
}

fn coef(entries: &'static [(&'static str, u8, u8)], keep: impl Fn(u8) -> bool) -> Table<Coef> {
    Table::new(entries.iter().filter(|e| keep(e.1)).map(|&(c, run, level)| {
        let v = match run {
            EOB => Coef::Eob,
            ESC => Coef::Escape,
            r => Coef::RunLevel(r & 0x3F, level),
        };
        (c, v)
    }))
}

/// The tables, built once.
pub fn tables() -> &'static Tables {
    static T: OnceLock<Tables> = OnceLock::new();
    T.get_or_init(|| Tables {
        mbai: Table::new(MBAI.iter().copied()),
        mbai_mpeg1: Table::new(MBAI.iter().copied().chain([(STUFFING_CODE, STUFFING)])),
        mb_type: [
            Table::new(MB_TYPE_I.iter().copied()),
            Table::new(MB_TYPE_P.iter().copied()),
            Table::new(MB_TYPE_B.iter().copied()),
            Table::new([("1", mb::INTRA)]),
        ],
        cbp: Table::new(CBP.iter().copied()),
        motion_code: Table::new(MOTION_CODE.iter().copied()),
        dmvector: Table::new(DMVECTOR.iter().copied()),
        dc_size: [Table::new(DC_SIZE_LUMINANCE.iter().copied()), Table::new(DC_SIZE_CHROMINANCE.iter().copied())],
        dct_first: coef(DCT_ZERO, in_first),
        dct_zero: coef(DCT_ZERO, in_later),
        dct_one: coef(DCT_ONE, |_| true),
    })
}

/// The tables as text, for the check that they are prefix-free and complete.
pub fn all_codes() -> Vec<(&'static str, Vec<String>)> {
    let strip = |c: &str| c.replace(' ', "");
    let signed = |c: &str, r: u8| if r == EOB || r == ESC { vec![strip(c)] } else { vec![strip(c) + "0", strip(c) + "1"] };
    vec![
        ("B.1", MBAI.iter().map(|e| strip(e.0)).collect()),
        ("B.2", MB_TYPE_I.iter().map(|e| strip(e.0)).collect()),
        ("B.3", MB_TYPE_P.iter().map(|e| strip(e.0)).collect()),
        ("B.4", MB_TYPE_B.iter().map(|e| strip(e.0)).collect()),
        ("B.9", CBP.iter().map(|e| strip(e.0)).collect()),
        ("B.10", MOTION_CODE.iter().map(|e| strip(e.0)).collect()),
        ("B.11", DMVECTOR.iter().map(|e| strip(e.0)).collect()),
        ("B.12", DC_SIZE_LUMINANCE.iter().map(|e| strip(e.0)).collect()),
        ("B.13", DC_SIZE_CHROMINANCE.iter().map(|e| strip(e.0)).collect()),
        ("B.14 first", DCT_ZERO.iter().filter(|e| in_first(e.1)).flat_map(|e| signed(e.0, e.1)).collect()),
        ("B.14", DCT_ZERO.iter().filter(|e| in_later(e.1)).flat_map(|e| signed(e.0, e.1)).collect()),
        ("B.15", DCT_ONE.iter().flat_map(|e| signed(e.0, e.1)).collect()),
    ]
}
