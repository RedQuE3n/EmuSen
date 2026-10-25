//! A decoder of MPEG-2 video and of the MPEG-1 streams the IPU's MP1 mode reads, written from ITU-T H.262 (02/2000)
//! apart from `src/ipu/`: the syntax of §6.2, the variable-length decoding of §7.2 through its own copy of Annex B's
//! tables matched bit by bit, the inverse scan, inverse quantisation, saturation and mismatch control of §7.3 and §7.4
//! (MPEG-1's oddification, §D.9.1, as `src/ipu/` reads it, Sapphire_Disputes.md D-307), the reference IDCT of Annex A
//! as a direct double sum, and the frame prediction of §7.6 for the progressive streams the gate's P- and B-pictures
//! come in. It records, for each macroblock, where each of its symbols lies, so that `driver` can put the IPU's
//! commands at the same places, and the six blocks the IDCT gives. Field prediction, dual prime, concealment vectors
//! and the scalable extensions are not decoded: no stream of the corpus has them.

#![allow(dead_code, clippy::needless_range_loop)]

/// Annex B's tables, copied from the standard's text: Tables B.1, B.9 to B.13 as (code, value); Tables B.2 to B.4 as
/// (code, [quant, forward, backward, pattern, intra]); Tables B.14 and B.15 as (code without its sign, run, level,
/// note), run −1 for End of Block and −2 for Escape, note 3 or 4 for Table B.14's two codes of run 0, level 1.
pub mod annex_b {
    pub const B1: &[(&str, i32)] = &[
        ("1", 1), ("0000 0101 01", 18), ("011", 2), ("0000 0101 00", 19), ("010", 3), ("0000 0100 11", 20),
        ("0011", 4), ("0000 0100 10", 21), ("0010", 5), ("0000 0100 011", 22), ("0001 1", 6), ("0000 0100 010", 23),
        ("0001 0", 7), ("0000 0100 001", 24), ("0000 111", 8), ("0000 0100 000", 25), ("0000 110", 9), ("0000 0011 111", 26),
        ("0000 1011", 10), ("0000 0011 110", 27), ("0000 1010", 11), ("0000 0011 101", 28), ("0000 1001", 12), ("0000 0011 100", 29),
        ("0000 1000", 13), ("0000 0011 011", 30), ("0000 0111", 14), ("0000 0011 010", 31), ("0000 0110", 15), ("0000 0011 001", 32),
        ("0000 0101 11", 16), ("0000 0011 000", 33), ("0000 0101 10", 17), ("0000 0001 000", -1),
    ];
    pub const B2: &[(&str, [u8; 5])] = &[
        ("1", [0, 0, 0, 0, 1]), ("01", [1, 0, 0, 0, 1]),
    ];
    pub const B3: &[(&str, [u8; 5])] = &[
        ("1", [0, 1, 0, 1, 0]), ("01", [0, 0, 0, 1, 0]), ("001", [0, 1, 0, 0, 0]),
        ("0001 1", [0, 0, 0, 0, 1]), ("0001 0", [1, 1, 0, 1, 0]), ("0000 1", [1, 0, 0, 1, 0]),
        ("0000 01", [1, 0, 0, 0, 1]),
    ];
    pub const B4: &[(&str, [u8; 5])] = &[
        ("10", [0, 1, 1, 0, 0]), ("11", [0, 1, 1, 1, 0]), ("010", [0, 0, 1, 0, 0]),
        ("011", [0, 0, 1, 1, 0]), ("0010", [0, 1, 0, 0, 0]), ("0011", [0, 1, 0, 1, 0]),
        ("0001 1", [0, 0, 0, 0, 1]), ("0001 0", [1, 1, 1, 1, 0]), ("0000 11", [1, 1, 0, 1, 0]),
        ("0000 10", [1, 0, 1, 1, 0]), ("0000 01", [1, 0, 0, 0, 1]),
    ];
    pub const B9: &[(&str, i32)] = &[
        ("111", 60), ("0001 1100", 35), ("1101", 4), ("0001 1011", 13), ("1100", 8), ("0001 1010", 49),
        ("1011", 16), ("0001 1001", 21), ("1010", 32), ("0001 1000", 41), ("1001 1", 12), ("0001 0111", 14),
        ("1001 0", 48), ("0001 0110", 50), ("1000 1", 20), ("0001 0101", 22), ("1000 0", 40), ("0001 0100", 42),
        ("0111 1", 28), ("0001 0011", 15), ("0111 0", 44), ("0001 0010", 51), ("0110 1", 52), ("0001 0001", 23),
        ("0110 0", 56), ("0001 0000", 43), ("0101 1", 1), ("0000 1111", 25), ("0101 0", 61), ("0000 1110", 37),
        ("0100 1", 2), ("0000 1101", 26), ("0100 0", 62), ("0000 1100", 38), ("0011 11", 24), ("0000 1011", 29),
        ("0011 10", 36), ("0000 1010", 45), ("0011 01", 3), ("0000 1001", 53), ("0011 00", 63), ("0000 1000", 57),
        ("0010 111", 5), ("0000 0111", 30), ("0010 110", 9), ("0000 0110", 46), ("0010 101", 17), ("0000 0101", 54),
        ("0010 100", 33), ("0000 0100", 58), ("0010 011", 6), ("0000 0011 1", 31), ("0010 010", 10), ("0000 0011 0", 47),
        ("0010 001", 18), ("0000 0010 1", 55), ("0010 000", 34), ("0000 0010 0", 59), ("0001 1111", 7), ("0000 0001 1", 27),
        ("0001 1110", 11), ("0000 0001 0", 39), ("0001 1101", 19), ("0000 0000 1", 0),
    ];
    pub const B10: &[(&str, i32)] = &[
        ("0000 0011 001", -16), ("0000 0011 011", -15), ("0000 0011 101", -14), ("0000 0011 111", -13), ("0000 0100 001", -12), ("0000 0100 011", -11),
        ("0000 0100 11", -10), ("0000 0101 01", -9), ("0000 0101 11", -8), ("0000 0111", -7), ("0000 1001", -6), ("0000 1011", -5),
        ("0000 111", -4), ("0001 1", -3), ("0011", -2), ("011", -1), ("1", 0), ("010", 1),
        ("0010", 2), ("0001 0", 3), ("0000 110", 4), ("0000 1010", 5), ("0000 1000", 6), ("0000 0110", 7),
        ("0000 0101 10", 8), ("0000 0101 00", 9), ("0000 0100 10", 10), ("0000 0100 010", 11), ("0000 0100 000", 12), ("0000 0011 110", 13),
        ("0000 0011 100", 14), ("0000 0011 010", 15), ("0000 0011 000", 16),
    ];
    pub const B11: &[(&str, i32)] = &[
        ("11", -1), ("0", 0), ("10", 1),
    ];
    pub const B12: &[(&str, i32)] = &[
        ("100", 0), ("00", 1), ("01", 2), ("101", 3), ("110", 4), ("1110", 5),
        ("1111 0", 6), ("1111 10", 7), ("1111 110", 8), ("1111 1110", 9), ("1111 1111 0", 10), ("1111 1111 1", 11),
    ];
    pub const B13: &[(&str, i32)] = &[
        ("00", 0), ("01", 1), ("10", 2), ("110", 3), ("1110", 4), ("1111 0", 5),
        ("1111 10", 6), ("1111 110", 7), ("1111 1110", 8), ("1111 1111 0", 9), ("1111 1111 10", 10), ("1111 1111 11", 11),
    ];
    pub const B14: &[(&str, i32, i32, u8)] = &[
        ("10", -1, 0, 0), ("1", 0, 1, 3), ("11", 0, 1, 4), ("011", 1, 1, 0),
        ("0100", 0, 2, 0), ("0101", 2, 1, 0), ("0010 1", 0, 3, 0), ("0011 1", 3, 1, 0),
        ("0011 0", 4, 1, 0), ("0001 10", 1, 2, 0), ("0001 11", 5, 1, 0), ("0001 01", 6, 1, 0),
        ("0001 00", 7, 1, 0), ("0000 110", 0, 4, 0), ("0000 100", 2, 2, 0), ("0000 111", 8, 1, 0),
        ("0000 101", 9, 1, 0), ("0000 01", -2, 0, 0), ("0010 0110", 0, 5, 0), ("0010 0001", 0, 6, 0),
        ("0010 0101", 1, 3, 0), ("0010 0100", 3, 2, 0), ("0010 0111", 10, 1, 0), ("0010 0011", 11, 1, 0),
        ("0010 0010", 12, 1, 0), ("0010 0000", 13, 1, 0), ("0000 0010 10", 0, 7, 0), ("0000 0011 00", 1, 4, 0),
        ("0000 0010 11", 2, 3, 0), ("0000 0011 11", 4, 2, 0), ("0000 0010 01", 5, 2, 0), ("0000 0011 10", 14, 1, 0),
        ("0000 0011 01", 15, 1, 0), ("0000 0010 00", 16, 1, 0), ("0000 0001 1101", 0, 8, 0), ("0000 0001 1000", 0, 9, 0),
        ("0000 0001 0011", 0, 10, 0), ("0000 0001 0000", 0, 11, 0), ("0000 0001 1011", 1, 5, 0), ("0000 0001 0100", 2, 4, 0),
        ("0000 0001 1100", 3, 3, 0), ("0000 0001 0010", 4, 3, 0), ("0000 0001 1110", 6, 2, 0), ("0000 0001 0101", 7, 2, 0),
        ("0000 0001 0001", 8, 2, 0), ("0000 0001 1111", 17, 1, 0), ("0000 0001 1010", 18, 1, 0), ("0000 0001 1001", 19, 1, 0),
        ("0000 0001 0111", 20, 1, 0), ("0000 0001 0110", 21, 1, 0), ("0000 0000 1101 0", 0, 12, 0), ("0000 0000 1100 1", 0, 13, 0),
        ("0000 0000 1100 0", 0, 14, 0), ("0000 0000 1011 1", 0, 15, 0), ("0000 0000 1011 0", 1, 6, 0), ("0000 0000 1010 1", 1, 7, 0),
        ("0000 0000 1010 0", 2, 5, 0), ("0000 0000 1001 1", 3, 4, 0), ("0000 0000 1001 0", 5, 3, 0), ("0000 0000 1000 1", 9, 2, 0),
        ("0000 0000 1000 0", 10, 2, 0), ("0000 0000 1111 1", 22, 1, 0), ("0000 0000 1111 0", 23, 1, 0), ("0000 0000 1110 1", 24, 1, 0),
        ("0000 0000 1110 0", 25, 1, 0), ("0000 0000 1101 1", 26, 1, 0), ("0000 0000 0111 11", 0, 16, 0), ("0000 0000 0111 10", 0, 17, 0),
        ("0000 0000 0111 01", 0, 18, 0), ("0000 0000 0111 00", 0, 19, 0), ("0000 0000 0110 11", 0, 20, 0), ("0000 0000 0110 10", 0, 21, 0),
        ("0000 0000 0110 01", 0, 22, 0), ("0000 0000 0110 00", 0, 23, 0), ("0000 0000 0101 11", 0, 24, 0), ("0000 0000 0101 10", 0, 25, 0),
        ("0000 0000 0101 01", 0, 26, 0), ("0000 0000 0101 00", 0, 27, 0), ("0000 0000 0100 11", 0, 28, 0), ("0000 0000 0100 10", 0, 29, 0),
        ("0000 0000 0100 01", 0, 30, 0), ("0000 0000 0100 00", 0, 31, 0), ("0000 0000 0011 000", 0, 32, 0), ("0000 0000 0010 111", 0, 33, 0),
        ("0000 0000 0010 110", 0, 34, 0), ("0000 0000 0010 101", 0, 35, 0), ("0000 0000 0010 100", 0, 36, 0), ("0000 0000 0010 011", 0, 37, 0),
        ("0000 0000 0010 010", 0, 38, 0), ("0000 0000 0010 001", 0, 39, 0), ("0000 0000 0010 000", 0, 40, 0), ("0000 0000 0011 111", 1, 8, 0),
        ("0000 0000 0011 110", 1, 9, 0), ("0000 0000 0011 101", 1, 10, 0), ("0000 0000 0011 100", 1, 11, 0), ("0000 0000 0011 011", 1, 12, 0),
        ("0000 0000 0011 010", 1, 13, 0), ("0000 0000 0011 001", 1, 14, 0), ("0000 0000 0001 0011", 1, 15, 0), ("0000 0000 0001 0010", 1, 16, 0),
        ("0000 0000 0001 0001", 1, 17, 0), ("0000 0000 0001 0000", 1, 18, 0), ("0000 0000 0001 0100", 6, 3, 0), ("0000 0000 0001 1010", 11, 2, 0),
        ("0000 0000 0001 1001", 12, 2, 0), ("0000 0000 0001 1000", 13, 2, 0), ("0000 0000 0001 0111", 14, 2, 0), ("0000 0000 0001 0110", 15, 2, 0),
        ("0000 0000 0001 0101", 16, 2, 0), ("0000 0000 0001 1111", 27, 1, 0), ("0000 0000 0001 1110", 28, 1, 0), ("0000 0000 0001 1101", 29, 1, 0),
        ("0000 0000 0001 1100", 30, 1, 0), ("0000 0000 0001 1011", 31, 1, 0),
    ];
    pub const B15: &[(&str, i32, i32, u8)] = &[
        ("0110", -1, 0, 0), ("10", 0, 1, 0), ("010", 1, 1, 0), ("110", 0, 2, 0),
        ("0010 1", 2, 1, 0), ("0111", 0, 3, 0), ("0011 1", 3, 1, 0), ("0001 10", 4, 1, 0),
        ("0011 0", 1, 2, 0), ("0001 11", 5, 1, 0), ("0000 110", 6, 1, 0), ("0000 100", 7, 1, 0),
        ("1110 0", 0, 4, 0), ("0000 111", 2, 2, 0), ("0000 101", 8, 1, 0), ("1111 000", 9, 1, 0),
        ("0000 01", -2, 0, 0), ("1110 1", 0, 5, 0), ("0001 01", 0, 6, 0), ("1111 001", 1, 3, 0),
        ("0010 0110", 3, 2, 0), ("1111 010", 10, 1, 0), ("0010 0001", 11, 1, 0), ("0010 0101", 12, 1, 0),
        ("0010 0100", 13, 1, 0), ("0001 00", 0, 7, 0), ("0010 0111", 1, 4, 0), ("1111 1100", 2, 3, 0),
        ("1111 1101", 4, 2, 0), ("0000 0010 0", 5, 2, 0), ("0000 0010 1", 14, 1, 0), ("0000 0011 1", 15, 1, 0),
        ("0000 0011 01", 16, 1, 0), ("1111 011", 0, 8, 0), ("1111 100", 0, 9, 0), ("0010 0011", 0, 10, 0),
        ("0010 0010", 0, 11, 0), ("0010 0000", 1, 5, 0), ("0000 0011 00", 2, 4, 0), ("0000 0001 1100", 3, 3, 0),
        ("0000 0001 0010", 4, 3, 0), ("0000 0001 1110", 6, 2, 0), ("0000 0001 0101", 7, 2, 0), ("0000 0001 0001", 8, 2, 0),
        ("0000 0001 1111", 17, 1, 0), ("0000 0001 1010", 18, 1, 0), ("0000 0001 1001", 19, 1, 0), ("0000 0001 0111", 20, 1, 0),
        ("0000 0001 0110", 21, 1, 0), ("1111 1010", 0, 12, 0), ("1111 1011", 0, 13, 0), ("1111 1110", 0, 14, 0),
        ("1111 1111", 0, 15, 0), ("0000 0000 1011 0", 1, 6, 0), ("0000 0000 1010 1", 1, 7, 0), ("0000 0000 1010 0", 2, 5, 0),
        ("0000 0000 1001 1", 3, 4, 0), ("0000 0000 1001 0", 5, 3, 0), ("0000 0000 1000 1", 9, 2, 0), ("0000 0000 1000 0", 10, 2, 0),
        ("0000 0000 1111 1", 22, 1, 0), ("0000 0000 1111 0", 23, 1, 0), ("0000 0000 1110 1", 24, 1, 0), ("0000 0000 1110 0", 25, 1, 0),
        ("0000 0000 1101 1", 26, 1, 0), ("0000 0000 0111 11", 0, 16, 0), ("0000 0000 0111 10", 0, 17, 0), ("0000 0000 0111 01", 0, 18, 0),
        ("0000 0000 0111 00", 0, 19, 0), ("0000 0000 0110 11", 0, 20, 0), ("0000 0000 0110 10", 0, 21, 0), ("0000 0000 0110 01", 0, 22, 0),
        ("0000 0000 0110 00", 0, 23, 0), ("0000 0000 0101 11", 0, 24, 0), ("0000 0000 0101 10", 0, 25, 0), ("0000 0000 0101 01", 0, 26, 0),
        ("0000 0000 0101 00", 0, 27, 0), ("0000 0000 0100 11", 0, 28, 0), ("0000 0000 0100 10", 0, 29, 0), ("0000 0000 0100 01", 0, 30, 0),
        ("0000 0000 0100 00", 0, 31, 0), ("0000 0000 0011 000", 0, 32, 0), ("0000 0000 0010 111", 0, 33, 0), ("0000 0000 0010 110", 0, 34, 0),
        ("0000 0000 0010 101", 0, 35, 0), ("0000 0000 0010 100", 0, 36, 0), ("0000 0000 0010 011", 0, 37, 0), ("0000 0000 0010 010", 0, 38, 0),
        ("0000 0000 0010 001", 0, 39, 0), ("0000 0000 0010 000", 0, 40, 0), ("0000 0000 0011 111", 1, 8, 0), ("0000 0000 0011 110", 1, 9, 0),
        ("0000 0000 0011 101", 1, 10, 0), ("0000 0000 0011 100", 1, 11, 0), ("0000 0000 0011 011", 1, 12, 0), ("0000 0000 0011 010", 1, 13, 0),
        ("0000 0000 0011 001", 1, 14, 0), ("0000 0000 0001 0011", 1, 15, 0), ("0000 0000 0001 0010", 1, 16, 0), ("0000 0000 0001 0001", 1, 17, 0),
        ("0000 0000 0001 0000", 1, 18, 0), ("0000 0000 0001 0100", 6, 3, 0), ("0000 0000 0001 1010", 11, 2, 0), ("0000 0000 0001 1001", 12, 2, 0),
        ("0000 0000 0001 1000", 13, 2, 0), ("0000 0000 0001 0111", 14, 2, 0), ("0000 0000 0001 0110", 15, 2, 0), ("0000 0000 0001 0101", 16, 2, 0),
        ("0000 0000 0001 1111", 27, 1, 0), ("0000 0000 0001 1110", 28, 1, 0), ("0000 0000 0001 1101", 29, 1, 0), ("0000 0000 0001 1100", 30, 1, 0),
        ("0000 0000 0001 1011", 31, 1, 0),
    ];
}

use annex_b::*;

/// ISO/IEC 11172-2's D-picture macroblock_type, its one code (H.262 §D.9.6).
pub const D_PICTURE: &[(&str, [u8; 5])] = &[("1", [0, 0, 0, 0, 1])];

/// The default intra matrix, row by row (H.262 §6.3.11); the non-intra default is 16 throughout.
pub const DEFAULT_INTRA: [u8; 64] = [
    8, 16, 19, 22, 26, 27, 29, 34, 16, 16, 22, 24, 27, 29, 34, 37, 19, 22, 26, 27, 29, 34, 34, 38, 22, 22, 26, 27, 29, 34, 37, 40, 22, 26, 27,
    29, 32, 35, 40, 48, 26, 27, 29, 32, 35, 40, 48, 58, 26, 27, 29, 34, 38, 46, 56, 69, 27, 29, 35, 38, 46, 56, 69, 83,
];

/// Figure 7-2 (zigzag) and Figure 7-3 (alternate) as rows of u, by v.
pub const ZIGZAG: [[usize; 8]; 8] = [
    [0, 1, 5, 6, 14, 15, 27, 28],
    [2, 4, 7, 13, 16, 26, 29, 42],
    [3, 8, 12, 17, 25, 30, 41, 43],
    [9, 11, 18, 24, 31, 40, 44, 53],
    [10, 19, 23, 32, 39, 45, 52, 54],
    [20, 22, 33, 38, 46, 51, 55, 60],
    [21, 34, 37, 47, 50, 56, 59, 61],
    [35, 36, 48, 49, 57, 58, 62, 63],
];
pub const ALTERNATE: [[usize; 8]; 8] = [
    [0, 4, 6, 20, 22, 36, 38, 52],
    [1, 5, 7, 21, 23, 37, 39, 53],
    [2, 8, 19, 24, 34, 40, 50, 54],
    [3, 9, 18, 25, 35, 41, 51, 55],
    [10, 17, 26, 30, 42, 46, 56, 60],
    [11, 16, 27, 31, 43, 47, 57, 61],
    [12, 15, 28, 32, 44, 48, 58, 62],
    [13, 14, 29, 33, 45, 49, 59, 63],
];

/// Table 7-6, quantiser_scale for q_scale_type 1; type 0 is twice the code.
pub const NON_LINEAR: [i32; 32] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 10, 12, 14, 16, 18, 20, 22, 24, 28, 32, 36, 40, 44, 48, 52, 56, 64, 72, 80, 88, 96, 104, 112];

/// A bit reader over a whole stream, positions counted in bits from its first.
pub struct Bits<'a> {
    pub d: &'a [u8],
    pub pos: usize,
}

impl<'a> Bits<'a> {
    pub fn bit(&self, i: usize) -> u32 {
        self.d.get(i / 8).map_or(0, |b| u32::from(b >> (7 - i % 8) & 1))
    }

    pub fn peek(&self, n: usize) -> u32 {
        (0..n).fold(0, |a, k| a << 1 | self.bit(self.pos + k))
    }

    pub fn read(&mut self, n: usize) -> u32 {
        let v = self.peek(n);
        self.pos += n;
        v
    }

    /// The table's entry whose code the stream's next bits spell, and the code's length; the stream moves past it.
    pub fn vlc<T: Copy>(&mut self, table: &[(&str, T)]) -> Option<(T, usize)> {
        for &(code, v) in table {
            let c: Vec<u32> = code.chars().filter(|c| *c != ' ').map(|c| c.to_digit(2).unwrap()).collect();
            if c.iter().enumerate().all(|(k, &b)| self.bit(self.pos + k) == b) {
                self.pos += c.len();
                return Some((v, c.len()));
            }
        }
        None
    }

    /// next_start_code() (§5.3): to the next byte, then past zero bytes to a start code prefix; the prefix's position.
    pub fn next_start_code(&mut self) -> Option<usize> {
        self.pos = self.pos.div_ceil(8) * 8;
        while self.pos / 8 + 3 <= self.d.len() {
            let i = self.pos / 8;
            if self.d[i] == 0 && self.d[i + 1] == 0 && self.d[i + 2] == 1 {
                return Some(self.pos);
            }
            self.pos += 8;
        }
        None
    }
}

/// The picture-level parameters the IPU's IPU_CTRL carries.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Ctrl {
    pub intra_dc_precision: u32,
    pub alternate_scan: bool,
    pub intra_vlc_format: bool,
    pub q_scale_type: bool,
    pub mpeg1: bool,
    pub picture_coding_type: u32,
}

/// A motion code's place, its value and its residual's width (0 when none follows).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MotionCode {
    pub pos: usize,
    pub code: i32,
    pub len: usize,
    pub residual: u32,
    pub residual_bits: usize,
}

/// One macroblock as the stream gives it, and where each of its symbols lies.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mb {
    pub address: usize,
    /// Where macroblock_address_increment begins (with any escapes and stuffing before it), its value, and the codes it
    /// took (each escape and stuffing one code).
    pub pos_mbai: usize,
    pub increment: usize,
    pub mbai_codes: Vec<(usize, i32)>,
    pub pos_type: usize,
    pub type_len: usize,
    /// macroblock_type as [quant, forward, backward, pattern, intra].
    pub kind: [u8; 5],
    pub pos_motion_type: Option<usize>,
    pub pos_dct_type: Option<usize>,
    pub dct_type: bool,
    pub pos_qsc: Option<usize>,
    pub qsc: u32,
    pub motion: Vec<MotionCode>,
    /// Where coded_block_pattern begins (non-intra) or the first block (intra), and where the last block ends.
    pub pos_cbp: usize,
    pub cbp: u32,
    pub pos_end: usize,
    /// Whether the DC predictors were reset before this macroblock (a slice's start, a non-intra or skipped one before).
    pub dc_reset: bool,
    /// The six blocks f[y][x] the IDCT gives, saturated to [−256, 255]; zeros for blocks not coded.
    pub blocks: [[i16; 64]; 6],
    /// The six blocks' reconstructed coefficients F[v][u], the IDCT's input.
    pub coefs: [[[i32; 8]; 8]; 6],
    /// What follows the macroblock: a start code's prefix, if one does.
    pub next_start_code: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slice {
    pub pos: usize,
    pub vertical: u32,
    pub qsc: u32,
    pub mbs: Vec<Mb>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    /// A matrix in force from here: loaded from the stream at `pos`, or the default.
    Matrix { pos: Option<usize>, intra: bool, w: [u8; 64] },
    Picture { pos: usize, ctrl: Ctrl, frame_pred_frame_dct: bool },
    Slice(Slice),
}

/// A decoded frame, 4:2:0, at the coded size of whole macroblocks; `shown` is the size the sequence header gives.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    pub w: usize,
    pub h: usize,
    pub shown: (usize, usize),
    pub y: Vec<u8>,
    pub cb: Vec<u8>,
    pub cr: Vec<u8>,
}

impl Frame {
    fn new(w: usize, h: usize, shown: (usize, usize)) -> Frame {
        Frame { w, h, shown, y: vec![0; w * h], cb: vec![0; w * h / 4], cr: vec![0; w * h / 4] }
    }

    fn plane(&self, c: usize) -> (&[u8], usize, usize) {
        match c {
            0 => (&self.y, self.w, self.h),
            1 => (&self.cb, self.w / 2, self.h / 2),
            _ => (&self.cr, self.w / 2, self.h / 2),
        }
    }

    fn plane_mut(&mut self, c: usize) -> (&mut [u8], usize) {
        let w = self.w;
        match c {
            0 => (&mut self.y, w),
            1 => (&mut self.cb, w / 2),
            _ => (&mut self.cr, w / 2),
        }
    }

    /// The shown planes as one buffer, Y then Cb then Cr, as `ffmpeg -pix_fmt yuv420p` writes a frame.
    pub fn bytes(&self) -> Vec<u8> {
        let (w, h) = self.shown;
        let mut v = Vec::new();
        for (c, cw, ch) in [(0, w, h), (1, w / 2, h / 2), (2, w / 2, h / 2)] {
            let (p, pw, _) = self.plane(c);
            for y in 0..ch {
                v.extend_from_slice(&p[y * pw..y * pw + cw]);
            }
        }
        v
    }
}

pub struct Decoded {
    pub events: Vec<Event>,
    /// The frames in display order.
    pub frames: Vec<Frame>,
    /// Where the stream's last start code (sequence_end_code) lies.
    pub end: usize,
    pub mpeg2: bool,
}

/// round() of Annex A, halves away from zero, for f(x, y) of `f` given its double-precision value `d`: within 2^-20 of
/// a half, decided exactly. 8f is Σ F(u,v)·2·C(u)C(v)cos((2x+1)uπ/16)cos((2y+1)vπ/16), and each product of cosines is
/// half the sum of the cosines of the sum and difference of the angles, so 8f gathers into integer multiples of
/// cos(kπ/16) for k = 0 to 7 (with C(0) taken as cos(4π/16)); those being independent over the rationals, f is a half
/// only if all but the first vanish, and otherwise its side of the half is found with the cosines to 100 bits.
fn round(f: &[[i32; 8]; 8], x: usize, y: usize, d: f64) -> i32 {
    if ((d.abs() - d.abs().floor()) - 0.5).abs() > 1.0 / 1_048_576.0 {
        return d.round() as i32;
    }
    let mut n = [0i128; 17];
    for v in 0..8 {
        for u in 0..8 {
            if f[v][u] == 0 {
                continue;
            }
            let ang_u = if u == 0 { 4 } else { (2 * x + 1) * u };
            let ang_v = if v == 0 { 4 } else { (2 * y + 1) * v };
            for m in [ang_u + ang_v, ang_u.abs_diff(ang_v)] {
                let m = m % 32;
                let (m, sign) = match m {
                    0..=8 => (m, 1),
                    9..=16 => (16 - m, -1),
                    17..=24 => (m - 16, -1),
                    _ => (32 - m, 1),
                };
                n[m] += sign * i128::from(f[v][u]);
            }
        }
    }
    if n[1..8].iter().all(|&c| c == 0) {
        let q = (n[0].abs() + 4) / 8;
        return (q * n[0].signum()) as i32;
    }
    // cos(kπ/16) × 2^100, rounded.
    const COS: [i128; 8] = [
        1267650600228229401496703205376,
        1243293049398167358316279903776,
        1171156443926508635938050024219,
        1054012953106854663288574085998,
        896364335596578238699711011639,
        704268939356236033728032079564,
        485108882735005078916833875245,
        247306363802463555846379077689,
    ];
    let v: i128 = (0..8).map(|k| n[k] * COS[k]).sum();
    let q = (v.abs() + (1 << 102)) >> 103;
    (q * v.signum()) as i32
}

/// The saturated mathematical integer-number IDCT of Annex A, as its double sum.
pub fn idct(f: &[[i32; 8]; 8]) -> [i16; 64] {
    let c = |u: usize| if u == 0 { std::f64::consts::FRAC_1_SQRT_2 } else { 1.0 };
    let pi = std::f64::consts::PI;
    let mut out = [0i16; 64];
    for y in 0..8 {
        for x in 0..8 {
            let mut s = 0.0;
            for v in 0..8 {
                for u in 0..8 {
                    if f[v][u] != 0 {
                        s += c(u) * c(v) * f64::from(f[v][u]) * (((2 * x + 1) * u) as f64 * pi / 16.0).cos() * (((2 * y + 1) * v) as f64 * pi / 16.0).cos();
                    }
                }
            }
            out[8 * y + x] = round(f, x, y, s / 4.0).clamp(-256, 255) as i16;
        }
    }
    out
}

/// The decoder's state across a stream.
struct State {
    mpeg2: bool,
    w: usize,
    h: usize,
    shown: (usize, usize),
    intra_w: [u8; 64],
    non_intra_w: [u8; 64],
    ctrl: Ctrl,
    f_code: [[u32; 2]; 2],
    picture_structure: u32,
    frame_pred_frame_dct: bool,
    concealment: bool,
    picture_pos: usize,
    picture_open: bool,
    announced: bool,
    current: Option<Frame>,
    refs: [Option<Frame>; 2],
    held: Option<Frame>,
    frames: Vec<Frame>,
}

/// The matrix from 64 bytes in zigzag order (§7.3.1).
fn matrix(b: &mut Bits) -> [u8; 64] {
    let wz: Vec<u8> = (0..64).map(|_| b.read(8) as u8).collect();
    let mut w = [0u8; 64];
    for v in 0..8 {
        for u in 0..8 {
            w[8 * v + u] = wz[ZIGZAG[v][u]];
        }
    }
    w
}

pub fn decode(data: &[u8]) -> Decoded {
    let mut b = Bits { d: data, pos: 0 };
    let mut events = Vec::new();
    let mut s = State {
        mpeg2: false,
        w: 0,
        h: 0,
        shown: (0, 0),
        intra_w: DEFAULT_INTRA,
        non_intra_w: [16; 64],
        ctrl: Ctrl::default(),
        f_code: [[15; 2]; 2],
        picture_structure: 3,
        frame_pred_frame_dct: true,
        concealment: false,
        picture_pos: 0,
        picture_open: false,
        announced: false,
        current: None,
        refs: [None, None],
        held: None,
        frames: Vec::new(),
    };
    let mut end = data.len() * 8;
    while let Some(at) = b.next_start_code() {
        b.pos += 24;
        let code = b.read(8);
        if matches!(code, 0x00 | 0xB3 | 0xB7 | 0xB8) {
            finish_picture(&mut s);
        }
        match code {
            0xB3 => {
                s.shown = (b.read(12) as usize, b.read(12) as usize);
                s.w = s.shown.0.div_ceil(16) * 16;
                s.h = s.shown.1.div_ceil(16) * 16;
                b.read(4 + 4 + 18 + 1 + 10 + 1);
                for intra in [true, false] {
                    let (pos, w) = if b.read(1) == 1 {
                        (Some(b.pos), matrix(&mut b))
                    } else {
                        (None, if intra { DEFAULT_INTRA } else { [16; 64] })
                    };
                    if intra { s.intra_w = w } else { s.non_intra_w = w }
                    events.push(Event::Matrix { pos, intra, w });
                }
            }
            0xB5 => match b.read(4) {
                1 => {
                    s.mpeg2 = true;
                    b.read(8);
                    if b.read(1) == 0 {
                        s.h = s.shown.1.div_ceil(32) * 32;
                    }
                    assert_eq!(b.read(2), 1, "4:2:0");
                }
                3 => {
                    for intra in [true, false] {
                        if b.read(1) == 1 {
                            let pos = b.pos;
                            let w = matrix(&mut b);
                            if intra { s.intra_w = w } else { s.non_intra_w = w }
                            events.push(Event::Matrix { pos: Some(pos), intra, w });
                        }
                    }
                }
                8 => {
                    for r in 0..2 {
                        for t in 0..2 {
                            s.f_code[r][t] = b.read(4);
                        }
                    }
                    s.ctrl.intra_dc_precision = b.read(2);
                    s.picture_structure = b.read(2);
                    assert_eq!(s.picture_structure, 3, "frame pictures only");
                    b.read(1);
                    s.frame_pred_frame_dct = b.read(1) == 1;
                    s.concealment = b.read(1) == 1;
                    assert!(!s.concealment, "no concealment vectors in the corpus");
                    s.ctrl.q_scale_type = b.read(1) == 1;
                    s.ctrl.intra_vlc_format = b.read(1) == 1;
                    s.ctrl.alternate_scan = b.read(1) == 1;
                }
                _ => {}
            },
            0x00 => {
                b.read(10);
                let kind = b.read(3);
                b.read(16);
                s.ctrl = Ctrl { picture_coding_type: kind, mpeg1: !s.mpeg2, ..Ctrl::default() };
                s.frame_pred_frame_dct = true;
                s.picture_structure = 3;
                s.f_code = [[15; 2]; 2];
                if kind == 2 || kind == 3 {
                    b.read(1);
                    let f = b.read(3);
                    s.f_code[0] = [f, f];
                }
                if kind == 3 {
                    b.read(1);
                    let f = b.read(3);
                    s.f_code[1] = [f, f];
                }
                s.picture_pos = at;
                s.picture_open = true;
                s.announced = false;
                s.current = Some(Frame::new(s.w, s.h, s.shown));
            }
            1..=0xAF => {
                if !s.announced {
                    events.push(Event::Picture { pos: s.picture_pos, ctrl: s.ctrl, frame_pred_frame_dct: s.frame_pred_frame_dct });
                    s.announced = true;
                }
                let slice = decode_slice(&mut b, &mut s, at, code);
                events.push(Event::Slice(slice));
            }
            0xB7 => {
                end = at;
                break;
            }
            _ => {}
        }
    }
    finish_picture(&mut s);
    if let Some(f) = s.held.take() {
        s.frames.push(f);
    }
    Decoded { events, frames: s.frames, end, mpeg2: s.mpeg2 }
}

/// A picture complete: B-pictures go out at once, I- and P-pictures when the next of them arrives (§6.1.1.11).
fn finish_picture(s: &mut State) {
    if !s.picture_open {
        return;
    }
    s.picture_open = false;
    let f = s.current.take().unwrap();
    if s.ctrl.picture_coding_type == 3 {
        s.frames.push(f);
    } else {
        if let Some(h) = s.held.take() {
            s.frames.push(h);
        }
        s.held = Some(f.clone());
        s.refs = [s.refs[1].take(), Some(f)];
    }
}

/// A prediction of one plane's block from `r` at vector (vx, vy) in half samples (§7.6.4).
fn predict(r: &Frame, c: usize, x0: usize, y0: usize, n: usize, vx: i32, vy: i32) -> Vec<i32> {
    let (p, w, h) = r.plane(c);
    let (ix, iy) = (vx.div_euclid(2), vy.div_euclid(2));
    let (hx, hy) = (vx - 2 * ix != 0, vy - 2 * iy != 0);
    let at = |x: i32, y: i32| -> i32 {
        assert!(x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h, "a vector outside the reference");
        i32::from(p[y as usize * w + x as usize])
    };
    let mut out = Vec::with_capacity(n * n);
    for y in 0..n as i32 {
        for x in 0..n as i32 {
            let (sx, sy) = (x0 as i32 + x + ix, y0 as i32 + y + iy);
            let v = match (hx, hy) {
                (false, false) => at(sx, sy),
                (false, true) => (at(sx, sy) + at(sx, sy + 1) + 1) / 2,
                (true, false) => (at(sx, sy) + at(sx + 1, sy) + 1) / 2,
                (true, true) => (at(sx, sy) + at(sx + 1, sy) + at(sx, sy + 1) + at(sx + 1, sy + 1) + 2) / 4,
            };
            out.push(v);
        }
    }
    out
}

/// The macroblock's samples: its prediction (none when intra) plus its blocks, saturated (§7.6.7, §7.6.8).
fn reconstruct_mb(s: &mut State, addr: usize, intra: bool, dirs: [bool; 2], mv: [[i32; 2]; 2], blocks: &[[i16; 64]; 6], field: bool) {
    let mbw = s.w / 16;
    let (mx, my) = (addr % mbw, addr / mbw);
    for c in 0..3 {
        let n = if c == 0 { 16 } else { 8 };
        let (x0, y0) = (mx * n, my * n);
        let pred: Vec<i32> = if intra {
            vec![0; n * n]
        } else {
            let mut ps = Vec::new();
            for d in 0..2 {
                if dirs[d] {
                    let r = s.refs[if s.ctrl.picture_coding_type == 3 && d == 0 { 0 } else { 1 }].as_ref().expect("a reference");
                    let (vx, vy) = if c == 0 { (mv[d][0], mv[d][1]) } else { (mv[d][0] / 2, mv[d][1] / 2) };
                    ps.push(predict(r, c, x0, y0, n, vx, vy));
                }
            }
            if ps.len() == 2 { ps[0].iter().zip(&ps[1]).map(|(a, b)| (a + b + 1) / 2).collect() } else { ps.remove(0) }
        };
        let (plane, w) = s.current.as_mut().unwrap().plane_mut(c);
        for y in 0..n {
            for x in 0..n {
                let f = if c == 0 {
                    let (bk, row) = if field { (2 * (y & 1) + x / 8, y / 2) } else { (2 * (y / 8) + x / 8, y % 8) };
                    blocks[bk][8 * row + x % 8]
                } else {
                    blocks[3 + c][8 * y + x]
                };
                plane[(y0 + y) * w + x0 + x] = (pred[n * y + x] + i32::from(f)).clamp(0, 255) as u8;
            }
        }
    }
}

/// One block's coefficients, inverse quantised and transformed (§7.2 to §7.5): F and f.
fn decode_block(b: &mut Bits, s: &State, i: usize, intra: bool, qsc: u32, dc_pred: &mut [i32; 3]) -> ([[i32; 8]; 8], [i16; 64]) {
    let mut qfs = [0i32; 64];
    let mut n = 0;
    let cc = if i < 4 { 0 } else { i - 3 };
    if intra {
        let (size, _) = b.vlc(if cc == 0 { B12 } else { B13 }).expect("dct_dc_size");
        let diff = if size == 0 {
            0
        } else {
            let d = b.read(size as usize) as i32;
            let half = 1 << (size - 1);
            if d >= half { d } else { d + 1 - 2 * half }
        };
        dc_pred[cc] += diff;
        qfs[0] = dc_pred[cc];
        n = 1;
    }
    let one = intra && s.ctrl.intra_vlc_format;
    // A D-picture's blocks hold their DC coefficient alone (H.262 §D.9.6).
    let d_picture = s.ctrl.picture_coding_type == 4;
    loop {
        if d_picture {
            break;
        }
        let table: Vec<(&str, (i32, i32))> = if one {
            B15.iter().map(|e| (e.0, (e.1, e.2))).collect()
        } else {
            B14.iter().filter(|e| if !intra && n == 0 { e.3 != 4 && e.1 != -1 } else { e.3 != 3 }).map(|e| (e.0, (e.1, e.2))).collect()
        };
        let ((run, level), _) = b.vlc(&table).expect("a DCT coefficient code");
        if run == -1 {
            break;
        }
        let (run, signed) = if run == -2 {
            let run = b.read(6) as i32;
            let level = if s.mpeg2 {
                let l = b.read(12) as i32;
                if l >= 2048 { l - 4096 } else { l }
            } else {
                match b.read(8) as i32 {
                    0 => b.read(8) as i32,
                    128 => b.read(8) as i32 - 256,
                    l if l >= 128 => l - 256,
                    l => l,
                }
            };
            (run, level)
        } else {
            (run, if b.read(1) == 1 { -level } else { level })
        };
        n += run as usize;
        assert!(n < 64, "a coefficient past the 64th");
        qfs[n] = signed;
        n += 1;
    }
    let scan = if s.ctrl.alternate_scan { &ALTERNATE } else { &ZIGZAG };
    let qs = if !s.mpeg2 || !s.ctrl.q_scale_type { 2 * qsc as i32 } else { NON_LINEAR[qsc as usize] };
    let w = if intra { &s.intra_w } else { &s.non_intra_w };
    let mut f = [[0i32; 8]; 8];
    let mut sum = 0;
    for v in 0..8 {
        for u in 0..8 {
            let qf = qfs[scan[v][u]];
            let mut x = if intra && u == 0 && v == 0 {
                qf * (8 >> s.ctrl.intra_dc_precision)
            } else if intra {
                (qf * i32::from(w[8 * v + u]) * qs * 2) / 32
            } else {
                ((qf * 2 + qf.signum()) * i32::from(w[8 * v + u]) * qs) / 32
            };
            if !s.mpeg2 && !(intra && u == 0 && v == 0) && x % 2 == 0 && x != 0 {
                x -= x.signum();
            }
            f[v][u] = x.clamp(-2048, 2047);
            sum += f[v][u];
        }
    }
    if s.mpeg2 && sum % 2 == 0 {
        f[7][7] = if f[7][7] % 2 != 0 { f[7][7] - 1 } else { f[7][7] + 1 };
    }
    (f, idct(&f))
}

/// One motion vector's component (§7.6.3.1), updating its predictor.
fn motion_component(b: &mut Bits, s: &State, r_s: usize, t: usize, pmv: &mut i32, rec: &mut Vec<MotionCode>) -> i32 {
    let pos = b.pos;
    let (code, len) = b.vlc(B10).expect("motion_code");
    let f_code = s.f_code[r_s][t];
    let r_size = f_code as usize - 1;
    let f = 1 << r_size;
    let (residual, residual_bits) = if f != 1 && code != 0 { (b.read(r_size), r_size) } else { (0, 0) };
    rec.push(MotionCode { pos, code, len, residual, residual_bits });
    let delta = if f == 1 || code == 0 {
        code
    } else {
        let d = (code.abs() - 1) * f + residual as i32 + 1;
        if code < 0 { -d } else { d }
    };
    let (low, high, range) = (-16 * f, 16 * f - 1, 32 * f);
    let mut v = *pmv + delta;
    if v < low {
        v += range;
    }
    if v > high {
        v -= range;
    }
    *pmv = v;
    v
}

fn decode_slice(b: &mut Bits, s: &mut State, at: usize, vertical: u32) -> Slice {
    let mbw = s.w / 16;
    let mut qsc = b.read(5);
    let slice_qsc = qsc;
    while b.peek(1) == 1 {
        b.read(9);
    }
    b.read(1);
    let reset = 128 << s.ctrl.intra_dc_precision;
    let mut dc_pred = [reset; 3];
    let mut dc_reset = true;
    let mut pmv = [[0i32; 2]; 2];
    let mut prev_dirs = [false; 2];
    let mut prev_mv = [[0i32; 2]; 2];
    let mut address = (vertical as usize - 1) * mbw;
    let mut first = true;
    let mut mbs = Vec::new();
    let kind = s.ctrl.picture_coding_type;
    loop {
        let pos_mbai = b.pos;
        let mut increment = 0usize;
        let mut mbai_codes = Vec::new();
        loop {
            if !s.mpeg2 && b.peek(11) == 0b000_0000_1111 {
                mbai_codes.push((11, -3));
                b.read(11);
                continue;
            }
            let (v, len) = b.vlc(B1).expect("macroblock_address_increment");
            mbai_codes.push((len, v));
            if v == -1 {
                increment += 33;
                continue;
            }
            increment += v as usize;
            break;
        }
        if first {
            address += increment - 1;
        } else {
            for _ in 1..increment {
                address += 1;
                dc_pred = [reset; 3];
                dc_reset = true;
                let (dirs, mv) = if kind == 2 {
                    pmv = [[0; 2]; 2];
                    ([true, false], [[0; 2]; 2])
                } else {
                    (prev_dirs, prev_mv)
                };
                reconstruct_mb(s, address, false, dirs, mv, &[[0; 64]; 6], false);
            }
            address += 1;
        }
        first = false;
        let pos_type = b.pos;
        let table = match kind {
            1 => B2,
            2 => B3,
            3 => B4,
            _ => D_PICTURE,
        };
        let (k, type_len) = b.vlc(table).expect("macroblock_type");
        let [quant, fwd, bwd, pattern, intra] = k.map(|v| v == 1);
        let mut pos_motion_type = None;
        if (fwd || bwd) && !s.frame_pred_frame_dct {
            pos_motion_type = Some(b.pos);
            assert_eq!(b.read(2), 2, "frame-based prediction only");
        }
        let mut pos_dct_type = None;
        let mut dct_type = false;
        if !s.frame_pred_frame_dct && (intra || pattern) {
            pos_dct_type = Some(b.pos);
            dct_type = b.read(1) == 1;
        }
        let mut pos_qsc = None;
        if quant {
            pos_qsc = Some(b.pos);
            qsc = b.read(5);
        }
        let mut motion = Vec::new();
        let mut mv = [[0i32; 2]; 2];
        for (d, on) in [fwd, bwd].into_iter().enumerate() {
            if on {
                for t in 0..2 {
                    mv[d][t] = motion_component(b, s, d, t, &mut pmv[d][t], &mut motion);
                }
            }
        }
        let pos_cbp = b.pos;
        let cbp = if pattern { b.vlc(B9).expect("coded_block_pattern").0 as u32 } else if intra { 63 } else { 0 };
        if !intra && !dc_reset {
            dc_pred = [reset; 3];
            dc_reset = true;
        }
        let mut blocks = [[0i16; 64]; 6];
        let mut coefs = [[[0i32; 8]; 8]; 6];
        let this_reset = dc_reset;
        for i in 0..6 {
            if cbp >> (5 - i) & 1 == 1 {
                (coefs[i], blocks[i]) = decode_block(b, s, i, intra, qsc, &mut dc_pred);
            }
        }
        if kind == 4 {
            assert_eq!(b.read(1), 1, "end_of_macroblock");
        }
        let pos_end = b.pos;
        if intra {
            dc_reset = false;
            pmv = [[0; 2]; 2];
        } else {
            dc_pred = [reset; 3];
            dc_reset = true;
        }
        let dirs = if kind == 2 && !intra { [true, false] } else { [fwd, bwd] };
        if kind == 2 && !intra && !fwd {
            pmv = [[0; 2]; 2];
        }
        reconstruct_mb(s, address, intra, dirs, mv, &blocks, dct_type);
        prev_dirs = dirs;
        prev_mv = mv;
        let at_end = b.peek(23) == 0;
        let mut look = Bits { d: b.d, pos: b.pos };
        let next_start_code = if at_end { look.next_start_code() } else { None };
        mbs.push(Mb {
            address,
            pos_mbai,
            increment,
            mbai_codes,
            pos_type,
            type_len,
            kind: k,
            pos_motion_type,
            pos_dct_type,
            dct_type,
            pos_qsc,
            qsc,
            motion,
            pos_cbp,
            cbp,
            pos_end,
            dc_reset: this_reset,
            blocks,
            coefs,
            next_start_code,
        });
        if at_end {
            break;
        }
    }
    Slice { pos: at, vertical, qsc: slice_qsc, mbs }
}
