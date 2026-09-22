//! The RDP's lookup tables, built at compile time by the C# builders' arithmetic.

const MAGIC_SQUARE: [u8; 16] = [0, 6, 1, 7, 4, 2, 5, 3, 3, 5, 2, 4, 7, 1, 6, 0];
const BAYER: [u8; 16] = [0, 4, 1, 5, 4, 0, 5, 1, 3, 7, 2, 6, 7, 3, 6, 2];

/// `DitherTables`: sixteen (colour, alpha) dithers per pair of modes, indexed `(rgb << 2) | alpha`.
pub static DITHER_TABLES: [[(u8, u8); 16]; 16] = build_dither_tables();

/// `BlendQuotients`: four divisor bits and eleven dividend bits in, eight quotient bits out.
pub static BLEND_QUOTIENTS: [u8; 0x8000] = build_blend_quotients();

/// `DivideTable`: a normalised w's shift in the low four bits and its reciprocal above.
pub static DIVIDE_TABLE: [i32; 0x8000] = build_divide_table();

/// `FiveToEight`: a five-bit channel widened by repeating its top three bits.
pub static FIVE_TO_EIGHT: [u8; 32] = build_five_to_eight();

/// `Log2`: the index of the highest set bit, zero for zero and one.
pub static LOG2: [u8; 256] = build_log2();

/// `CoverageOffsets`: a mask's first covered sample row and that row's first covered column, as (x, y).
pub static COVERAGE_OFFSETS: [(u8, u8); 256] = build_coverage_offsets();

const fn build_dither_tables() -> [[(u8, u8); 16]; 16] {
    let mut tables = [[(0u8, 0u8); 16]; 16];
    let mut rgb = 0;
    while rgb < 4 {
        let mut alpha_mode = 0;
        while alpha_mode < 4 {
            let mut index = 0;
            while index < 16 {
                let (color, pattern) = match rgb {
                    0 => (MAGIC_SQUARE[index], MAGIC_SQUARE[index] as i32),
                    1 => (BAYER[index], BAYER[index] as i32),
                    2 => (0, MAGIC_SQUARE[index] as i32),
                    _ => (7, BAYER[index] as i32),
                };
                let alpha = match alpha_mode {
                    0 => pattern,
                    1 => !pattern & 7,
                    _ => 0,
                };
                tables[rgb * 4 + alpha_mode][index] = (color, alpha as u8);
                index += 1;
            }
            alpha_mode += 1;
        }
        rgb += 1;
    }
    tables
}

const fn build_blend_quotients() -> [u8; 0x8000] {
    let mut table = [0u8; 0x8000];
    let mut index = 0;
    while index < 0x8000 {
        let divisor = ((index >> 11) & 0xF) as i32;
        let dividend = (index & 0x7FF) as i32;
        let complement = !divisor & 0xF;
        let mut remainder = (complement + (dividend >> 8) + 1) & 7;
        let mut quotient = 0;
        let mut bit = 7;
        while bit >= 0 {
            let incoming = (dividend >> bit) & 1;
            let sum = if ((quotient >> (bit + 1)) & 1) != 0 {
                complement + (remainder << 1) + incoming + 1
            } else {
                divisor + (remainder << 1) + incoming
            };
            remainder = sum & 7;
            if (sum & 0x10) != 0 {
                quotient |= 1 << bit;
            }
            bit -= 1;
        }
        table[index] = quotient as u8;
        index += 1;
    }
    table
}

/// `ReciprocalPoint`: 2^20 over 64 to 128, rounded (no quotient is a tie), the seventh one lower.
pub const fn reciprocal_point(segment: i32) -> i32 {
    if segment == 6 {
        return 0x3A83;
    }
    let d = 64 + segment;
    (2 * 0x10_0000 + d) / (2 * d)
}

const fn build_divide_table() -> [i32; 0x8000] {
    let mut table = [0i32; 0x8000];
    let mut w = 0;
    while w < 0x8000 {
        let mut k = 1;
        while k <= 14 && ((w << k) & 0x8000) == 0 {
            k += 1;
        }
        let shift = k - 1;
        let normalised = (w << shift) & 0x3FFF;
        let fraction = (normalised & 0xFF) << 2;
        let segment = normalised >> 8;
        let point = reciprocal_point(segment);
        let slope = reciprocal_point(segment + 1) - point;
        let reciprocal = (((slope * fraction) >> 10) + point) & 0x7FFF;
        table[w as usize] = shift | (reciprocal << 4);
        w += 1;
    }
    table
}

const fn build_five_to_eight() -> [u8; 32] {
    let mut table = [0u8; 32];
    let mut i = 0;
    while i < 32 {
        table[i] = ((i << 3) | ((i >> 2) & 7)) as u8;
        i += 1;
    }
    table
}

const fn build_log2() -> [u8; 256] {
    let mut table = [0u8; 256];
    let mut i = 2;
    while i < 256 {
        table[i] = (31 - (i as u32).leading_zeros()) as u8;
        i += 1;
    }
    table
}

const fn build_coverage_offsets() -> [(u8, u8); 256] {
    let mut table = [(0u8, 0u8); 256];
    let mut mask = 0;
    while mask < 256 {
        let samples = (mask & 0x5) | ((mask & 0x5A) << 4) | ((mask & 0xA0) << 8);
        let mut row = 0;
        while row < 3 && (samples & (0xF000 >> (row * 4))) == 0 {
            row += 1;
        }
        if (samples & (0xF000 >> (row * 4))) == 0 {
            row = 0;
        }
        let columns = (samples >> (12 - row * 4)) & 0xF;
        let column = if columns == 0 { 0 } else { 3 - (31 - (columns as u32).leading_zeros() as i32) };
        table[mask as usize] = (column as u8, row as u8);
        mask += 1;
    }
    table
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_integer_reciprocal_is_the_rounded_quotient() {
        for segment in 0..=64 {
            let expected = if segment == 6 { 0x3A83 } else { (1_048_576.0 / (64.0 + segment as f64)).round_ties_even() as i32 };
            assert_eq!(reciprocal_point(segment), expected, "segment {segment}");
        }
    }

    #[test]
    fn the_tables_hold_known_entries() {
        assert_eq!(FIVE_TO_EIGHT[31], 0xFF);
        assert_eq!(FIVE_TO_EIGHT[1], 0x08);
        assert_eq!(LOG2[255], 7);
        assert_eq!(LOG2[32], 5);
        assert_eq!(COVERAGE_OFFSETS[0xFF], (0, 0));
        assert_eq!(DITHER_TABLES[0][1], (6, 6));
        assert_eq!(DITHER_TABLES[0xF][5], (7, 0));
        assert_eq!(DIVIDE_TABLE[0] & 0xF, 14);
    }
}
