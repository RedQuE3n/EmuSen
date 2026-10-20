//! The Master System VDP family's colours as RGBA (Beryl_SmsVdp.md §10): a `--BBGGRR` CRAM entry on the 315-5246 and on
//! the 315-5124, whose first blue level differs (Endou_Disputes.md D-1), and the TMS9918A's sixteen colour codes (D-11).
//! The chips' outputs, which their console shows; the VDP crate composes indices into them and never colours.

/// A two-bit component's level on the 315-5246, four equal steps (Endou_Disputes.md D-1).
pub const LEVELS: [u8; 4] = [0, 85, 170, 255];

/// The 315-5124's blue, its first level boosted to #68, read from SMS Power!'s "Palette", not measured (D-1, decided
/// 2026-10-19).
pub const BLUE_5124: [u8; 4] = [0, 0x68, 170, 255];

const fn colours(blue: [u8; 4]) -> [[u8; 4]; 64] {
    let mut t = [[0u8; 4]; 64];
    let mut i = 0;
    while i < 64 {
        t[i] = [LEVELS[i & 3], LEVELS[(i >> 2) & 3], blue[(i >> 4) & 3], 255];
        i += 1;
    }
    t
}

/// The 64 colours of a `--BBGGRR` entry as RGBA: the 315-5246's (the Master System II), and the 315-5124's (the Mark III
/// and the first Master System).
pub const SMS_COLOURS: [[u8; 4]; 64] = colours(LEVELS);
pub const SMS1_COLOURS: [[u8; 4]; 64] = colours(BLUE_5124);

/// The TMS9918A's 16 colours: TI's colour difference levels (TMS9918A data manual, Table 2-3) as RGB, each difference
/// taken from 0.47, rounded to the nearest of 255 steps, held by a test (Endou_Disputes.md D-11).
pub const TMS_COLOURS: [[u8; 4]; 16] = [
    [0x00, 0x00, 0x00, 255],
    [0x00, 0x00, 0x00, 255],
    [0x21, 0xC8, 0x42, 255],
    [0x5E, 0xDC, 0x78, 255],
    [0x54, 0x55, 0xED, 255],
    [0x7D, 0x76, 0xFC, 255],
    [0xD4, 0x52, 0x4D, 255],
    [0x42, 0xEC, 0xF5, 255],
    [0xFC, 0x55, 0x54, 255],
    [0xFF, 0x79, 0x78, 255],
    [0xD4, 0xC1, 0x54, 255],
    [0xE6, 0xCE, 0x80, 255],
    [0x21, 0xB0, 0x3B, 255],
    [0xC9, 0x5B, 0xBA, 255],
    [0xCC, 0xCC, 0xCC, 255],
    [0xFF, 0xFF, 0xFF, 255],
];

#[cfg(test)]
mod tests {
    use super::*;

    /// TI's Table 2-3, the TMS9928A's luminance and colour difference levels in volts for codes 1-15.
    const TI_LEVELS: [(f64, f64, f64); 15] = [
        (0.00, 0.47, 0.47),
        (0.53, 0.07, 0.20),
        (0.67, 0.17, 0.27),
        (0.40, 0.40, 1.00),
        (0.53, 0.43, 0.93),
        (0.47, 0.83, 0.30),
        (0.73, 0.00, 0.70),
        (0.53, 0.93, 0.27),
        (0.67, 0.93, 0.27),
        (0.73, 0.57, 0.07),
        (0.80, 0.57, 0.17),
        (0.47, 0.13, 0.23),
        (0.53, 0.73, 0.67),
        (0.80, 0.47, 0.47),
        (1.00, 0.47, 0.47),
    ];

    #[test]
    fn the_tms9918a_s_colours_are_ti_s_levels_as_rgb() {
        for (code, &(y, ry, by)) in TI_LEVELS.iter().enumerate() {
            let (r, b) = (y + ry - 0.47, y + by - 0.47);
            let g = (y - 0.299 * r - 0.114 * b) / 0.587;
            let q = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
            assert_eq!(TMS_COLOURS[code + 1], [q(r), q(g), q(b), 255], "code {}", code + 1);
        }
        assert_eq!(TMS_COLOURS[0], TMS_COLOURS[1], "transparent over no external video is black");
    }

    #[test]
    fn the_315_5124_boosts_only_its_first_blue_level() {
        let differ: Vec<usize> = (0..64).filter(|&i| SMS1_COLOURS[i] != SMS_COLOURS[i]).collect();
        assert_eq!(differ, (16..32).collect::<Vec<_>>());
        assert_eq!((SMS1_COLOURS[0x10], SMS_COLOURS[0x10]), ([0, 0, 0x68, 255], [0, 0, 0x55, 255]));
    }
}
