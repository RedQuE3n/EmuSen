//! The YM2612's vibrato: the offset the LFO adds to a channel's frequency number, measured on the board (Nephrite_LfoTables.md).

/// The vibrato's steps a cycle: `lfo_step` runs from 0 to 31.
pub const PM_STEPS: u8 = 32;

/// For each PMS and quarter-wave step, the terms summed: bit k adds the frequency number's top seven bits shifted right by k.
pub const PM_TERMS: [[u8; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 1, 1, 1, 1],
    [0, 0, 0, 1, 1, 1, 1, 1],
    [0, 0, 1, 1, 1, 1, 6, 6],
    [0, 0, 1, 1, 1, 1, 6, 1],
    [0, 0, 1, 6, 1, 1, 5, 3],
    [0, 0, 1, 6, 1, 1, 5, 3],
    [0, 0, 1, 6, 1, 1, 5, 3],
];

/// For each PMS and quarter-wave step, how far right the sum of the terms is shifted.
pub const PM_SHIFT: [[u8; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 4, 4, 4, 4],
    [0, 0, 0, 4, 4, 4, 3, 3],
    [0, 0, 4, 4, 3, 3, 2, 2],
    [0, 0, 4, 3, 3, 3, 2, 2],
    [0, 0, 3, 2, 2, 2, 2, 2],
    [0, 0, 2, 1, 1, 1, 1, 1],
    [0, 0, 1, 0, 0, 0, 0, 0],
];

/// The quarter-wave step of an LFO step: steps 0-7 rise through it, 8-15 fall back, 16-31 repeat the two negated.
pub fn quarter(lfo_step: u8) -> usize {
    let s = lfo_step & 15;
    (if s & 8 == 0 { s } else { 15 - s }) as usize
}

/// The offset added at `lfo_step` (0-31) to an 11-bit frequency number at `pms` (0-7), in half frequency-number steps.
pub fn pm_offset(fnum: u16, pms: u8, lfo_step: u8) -> i16 {
    let top = (fnum >> 4) & 0x7F;
    let (pms, q) = ((pms & 7) as usize, quarter(lfo_step));
    let sum: u16 = (0..3).filter(|k| PM_TERMS[pms][q] >> k & 1 != 0).map(|k| top >> k).sum();
    let v = (sum >> PM_SHIFT[pms][q]) as i16;
    if lfo_step & 16 != 0 { -v } else { v }
}

/// The modulated frequency number in half steps, twelve bits, wrapped: the phase increment's base is `(this << block) >> 2`.
pub fn pm_fnum(fnum: u16, pms: u8, lfo_step: u8) -> u16 {
    ((((fnum & 0x7FF) << 1) as i16).wrapping_add(pm_offset(fnum, pms, lfo_step)) as u16) & 0xFFF
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FNV-1a of every offset as a little-endian i16, PMS then frequency number then step: `mdboard.py lfo`'s hash of the board's.
    const BOARD_HASH: u32 = 0x8F2D_DBC5;

    fn fnv(bytes: impl Iterator<Item = u8>) -> u32 {
        bytes.fold(0x811C_9DC5u32, |h, b| (h ^ b as u32).wrapping_mul(0x0100_0193))
    }

    #[test]
    fn every_step_of_every_frequency_number_and_pms_is_the_boards() {
        let all = (0..8u8).flat_map(|p| (0..2048u16).flat_map(move |f| (0..PM_STEPS).map(move |s| pm_offset(f, p, s))));
        assert_eq!(fnv(all.flat_map(|v| v.to_le_bytes())), BOARD_HASH);
    }

    /// The board's quarter-wave (steps 0-7) for single frequency-number bits and for wider numbers, as `mdboard.py lfo` reads them.
    #[test]
    fn quarter_waves_are_the_boards() {
        let board: &[(u16, u8, [i16; 8])] = &[
            (1024, 1, [0, 0, 0, 0, 4, 4, 4, 4]),
            (1024, 2, [0, 0, 0, 4, 4, 4, 8, 8]),
            (1024, 3, [0, 0, 4, 4, 8, 8, 12, 12]),
            (1024, 4, [0, 0, 4, 8, 8, 8, 12, 16]),
            (1024, 5, [0, 0, 8, 12, 16, 16, 20, 24]),
            (1024, 6, [0, 0, 16, 24, 32, 32, 40, 48]),
            (1024, 7, [0, 0, 32, 48, 64, 64, 80, 96]),
            (16, 7, [0, 0, 0, 0, 1, 1, 1, 1]),
            (15, 7, [0; 8]),
            (2047, 0, [0; 8]),
            (2047, 4, [0, 0, 7, 15, 15, 15, 23, 31]),
            (2047, 7, [0, 0, 63, 94, 127, 127, 158, 190]),
            (1777, 4, [0, 0, 6, 13, 13, 13, 20, 27]),
            (1777, 7, [0, 0, 55, 82, 111, 111, 138, 166]),
            (1900, 4, [0, 0, 7, 14, 14, 14, 22, 29]),
            (1900, 7, [0, 0, 59, 88, 118, 118, 147, 177]),
            (1365, 7, [0, 0, 42, 63, 85, 85, 106, 127]),
            (683, 4, [0, 0, 2, 5, 5, 5, 7, 10]),
        ];
        for &(fnum, pms, q) in board {
            assert_eq!(core::array::from_fn::<i16, 8, _>(|s| pm_offset(fnum, pms, s as u8)), q, "fnum {fnum} pms {pms}");
        }
    }

    #[test]
    fn the_cycle_is_a_quarter_wave_mirrored_then_negated() {
        for (f, p) in [(2047, 7), (1900, 4), (683, 2)] {
            for s in 0..8u8 {
                assert_eq!(pm_offset(f, p, 15 - s), pm_offset(f, p, s));
                assert_eq!(pm_offset(f, p, 16 + s), -pm_offset(f, p, s));
                assert_eq!(pm_offset(f, p, 31 - s), -pm_offset(f, p, s));
            }
        }
    }

    /// On the board 2047 at PMS 4, step 6, is 23 where its bits' own offsets sum to 22: the sum is shifted after it is made.
    #[test]
    fn an_offset_is_not_the_sum_of_its_bits_offsets() {
        assert_eq!(pm_offset(2047, 4, 6), 23);
        assert_eq!((0..11).map(|b| pm_offset(1 << b, 4, 6)).sum::<i16>(), 22);
        assert_eq!(pm_offset(2047, 7, 3), 94);
        assert_eq!((0..11).map(|b| pm_offset(1 << b, 7, 3)).sum::<i16>(), 94);
    }

    #[test]
    fn the_low_four_bits_never_count() {
        for p in 0..8u8 {
            for f in 0..2048u16 {
                for s in 0..PM_STEPS {
                    assert_eq!(pm_offset(f, p, s), pm_offset(f & !15, p, s));
                }
            }
        }
    }

    /// The board's 2047 at PMS 7, steps 2 and 7: a phase increment of 1,952 and 6,016 at block 7, the sum wrapped in twelve bits.
    #[test]
    fn the_modulated_number_wraps_in_twelve_bits() {
        assert_eq!((pm_fnum(2047, 7, 2) as u32) << 7 >> 2, 1952);
        assert_eq!((pm_fnum(2047, 7, 7) as u32) << 7 >> 2, 6016);
        assert_eq!((pm_fnum(2047, 7, 23) as u32) << 7 >> 2, 124928);
        assert_eq!(pm_fnum(1024, 0, 9), 2048);
    }

    /// Phase increments the board's pins decide at blocks 0 to 5 (multiple 1): (fnum, PMS, block, step, increment).
    #[test]
    fn the_block_shifts_the_modulated_number_as_on_the_board() {
        let board = [(2047, 7, 0, 0, 1023), (2047, 7, 0, 3, 23), (2047, 7, 0, 7, 47), (2047, 7, 1, 23, 1952), (1777, 7, 1, 3, 1818),
                     (1900, 4, 3, 7, 7658), (1777, 7, 3, 23, 6776), (1900, 4, 5, 23, 30168), (1777, 7, 5, 23, 27104)];
        for (fnum, pms, block, step, inc) in board {
            assert_eq!((pm_fnum(fnum, pms, step) as u32) << block >> 2, inc, "fnum {fnum} pms {pms} block {block} step {step}");
        }
    }
}
