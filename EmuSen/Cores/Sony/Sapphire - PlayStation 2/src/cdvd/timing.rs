//! The drive's times in IOP cycles (Sapphire_Native.md §25.4): ps2tek's block formula and seek classes, the contiguous
//! bounds measured on PCSX2 (D-120), and the seek distance taken across a dual-layer disc's radius (D-123).

/// The IOP's clock, ~36,864,000 Hz [R ps2tek "CDVD Reads and Seeks"; D-0]: the drive's unit of time.
pub const IOP_HZ: u64 = 36_864_000;

/// The read speeds of ps2tek's formula: "24 * 153600" for CD reads, "4 * 1382400" for DVD reads, bytes a second.
pub const CD_SPEED: u64 = 24 * 153_600;
pub const DVD_SPEED: u64 = 4 * 1_382_400;

/// ps2tek's seek classes: "~30 ms" for a fast seek, "~100 ms" for a full one, 333 ms to spin a stopped drive up.
pub const FAST_SEEK: u64 = IOP_HZ * 30 / 1000;
pub const FULL_SEEK: u64 = IOP_HZ / 10;
pub const SPIN_UP: u64 = IOP_HZ / 3;
/// Stop's delay, ps2tek's "166 ms", measured on PCSX2 as 166.67 ms (D-120).
pub const STOP: u64 = IOP_HZ / 6;
/// What a command with nothing to wait for takes before its interrupt [C].
pub const PROMPT: u64 = 64;

/// A fast seek is a distance below these, in sectors [R ps2tek: "< 14764 for DVD reads and < 4371 for CD reads"].
pub const DVD_FAST: u32 = 14_764;
pub const CD_FAST: u32 = 4_371;
/// A contiguous move is a distance below these [M PCSX2, D-120: 15 sectors contiguous and 16 a fast seek on a DVD; 7 and
/// 8 on a CD].
pub const DVD_CONTIGUOUS: u32 = 16;
pub const CD_CONTIGUOUS: u32 = 8;

/// ps2tek's "block_timing = (IOP_CLOCK * block_size) / read_speed".
pub const fn block(size: usize, speed: u64) -> u64 {
    IOP_HZ * size as u64 / speed
}

/// Where a logical sector lies across the disc's radius, in sectors from the Data Zone's start: itself on a single
/// layer, and on an Opposite Track Path disc's layer 1 the layer 0 sector it sits over (ECMA-267 §25, D-123).
pub fn radius(lba: u32, layer_break: Option<u32>) -> (u8, u32) {
    match layer_break {
        Some(b) if lba >= b => (1, (2 * b - 1).saturating_sub(lba)),
        _ => (0, lba),
    }
}

/// A seek's time from `from` to `to` (ps2tek "Seeking"): contiguous below the contiguous bound, `block` a sector of
/// distance; fast below the fast bound; full otherwise; a change of layer at least a fast seek (D-123).
pub fn seek(from: u32, to: u32, dvd: bool, layer_break: Option<u32>, block: u64) -> u64 {
    let ((l0, r0), (l1, r1)) = (radius(from, layer_break), radius(to, layer_break));
    let distance = r0.abs_diff(r1);
    let (contiguous, fast) = if dvd { (DVD_CONTIGUOUS, DVD_FAST) } else { (CD_CONTIGUOUS, CD_FAST) };
    let travel = if distance < contiguous {
        block * distance as u64
    } else if distance < fast {
        FAST_SEEK
    } else {
        FULL_SEEK
    };
    if l0 != l1 { travel.max(FAST_SEEK) } else { travel }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_formula_and_the_classes_are_ps2teks() {
        assert_eq!(block(2048, CD_SPEED), 20_480);
        assert_eq!(block(2064, DVD_SPEED), 13_760);
        let b = block(2064, DVD_SPEED);
        assert_eq!(seek(100, 100, true, None, b), 0);
        assert_eq!(seek(100, 115, true, None, b), 15 * b);
        assert_eq!(seek(100, 116, true, None, b), FAST_SEEK);
        assert_eq!(seek(100, 100 + 14_763, true, None, b), FAST_SEEK);
        assert_eq!(seek(100, 100 + 14_764, true, None, b), FULL_SEEK);
        assert_eq!(seek(5000, 4993, false, None, 1), 7);
        assert_eq!(seek(5000, 4992, false, None, 1), FAST_SEEK);
        assert_eq!(seek(0, 4371, false, None, 1), FULL_SEEK);
    }

    #[test]
    fn a_dual_layer_seek_is_measured_across_the_radius() {
        let (b, brk) = (13_760, Some(600_000));
        assert_eq!(radius(599_999, brk), (0, 599_999));
        assert_eq!(radius(600_000, brk), (1, 599_999), "layer 1's first sector sits over layer 0's last");
        assert_eq!(radius(600_010, brk), (1, 599_989));
        assert_eq!(seek(599_999, 600_000, true, brk, b), FAST_SEEK, "no distance, but a change of layer");
        assert_eq!(seek(600_000, 600_008, true, brk, b), 8 * b, "along layer 1");
        assert_eq!(seek(0, 1_199_000, true, brk, b), FAST_SEEK, "the inner end of layer 1 is over sector 999");
        assert_eq!(seek(0, 600_000, true, brk, b), FULL_SEEK);
    }
}
