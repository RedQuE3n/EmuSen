//! MarsViTests' angrylion pixels, and the filters' arithmetic against independent formulations. See Mars_Native.md §5.4.

use super::filters::{Pixel, divot, gamma, median, mix, pull, root, runners, step};
use super::*;

const FRAMEBUFFER: usize = 0x0020_0000;
const GAMMA_ON: u32 = 1 << 3;
const DIVOT_ON: u32 = 1 << 4;
const DITHER_FILTER: u32 = 1 << 16;

/// MarsViTests.Noisy: the reference test's frame buffer and coverage, in a 4 MB machine.
fn noisy() -> (Vec<u8>, Vec<u8>) {
    let mut rdram = vec![0u8; 0x40_0000];
    let mut hidden = vec![0u8; 0x20_0000];
    let mut state: u32 = 0x2468_ACE0;
    for i in 0..0x3000 {
        state = state.wrapping_mul(1_103_515_245).wrapping_add(12345);
        rdram[FRAMEBUFFER + i] = (state >> 16) as u8;
    }
    let mut seed: u32 = 0x1B4E_81B4;
    for i in 0..0x1800 {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        hidden[FRAMEBUFFER / 2 + i] = (seed >> 30) as u8;
    }
    (rdram, hidden)
}

struct Regs {
    kind: u32,
    antialias: u32,
    width: u32,
    step_x: u32,
    step_y: u32,
    left: u32,
    columns: u32,
    top: u32,
    rows: u32,
    bias_x: u32,
    bias_y: u32,
    serrate: bool,
    current_line: u32,
    sync: u32,
    origin: u32,
    control: u32,
}

#[allow(clippy::too_many_arguments)]
fn regs(kind: u32, antialias: u32, width: u32, step_x: u32, step_y: u32, left: u32, columns: u32, top: u32, rows: u32, bias_x: u32, bias_y: u32) -> Regs {
    Regs {
        kind,
        antialias,
        width,
        step_x,
        step_y,
        left,
        columns,
        top,
        rows,
        bias_x,
        bias_y,
        serrate: false,
        current_line: 0,
        sync: 525,
        origin: FRAMEBUFFER as u32,
        control: 0,
    }
}

impl Regs {
    fn words(&self) -> [u32; 14] {
        let mut r = [0u32; 14];
        r[0] = (self.kind & 3) | ((self.antialias & 3) << 8) | if self.serrate { 1 << 6 } else { 0 } | self.control;
        r[1] = self.origin;
        r[2] = self.width;
        r[4] = self.current_line;
        r[6] = self.sync;
        r[9] = ((self.left & 0x3FF) << 16) | ((self.left + self.columns) & 0x3FF);
        r[10] = ((self.top & 0x3FF) << 16) | ((self.top + self.rows * 2) & 0x3FF);
        r[12] = ((self.bias_x & 0xFFF) << 16) | (self.step_x & 0xFFF);
        r[13] = ((self.bias_y & 0xFFF) << 16) | (self.step_y & 0xFFF);
        r
    }
    fn serrate(mut self, line: u32) -> Self {
        self.serrate = true;
        self.current_line = line;
        self
    }
    fn sync(mut self, sync: u32) -> Self {
        self.sync = sync;
        self
    }
    fn origin(mut self, origin: u32) -> Self {
        self.origin = origin;
        self
    }
    fn control(mut self, control: u32) -> Self {
        self.control = control;
        self
    }
}

/// `Vi.Frame`: the raster's first `FrameHeight` rows.
fn frame_of(vi: &Vi, out: &Scanout) -> Vec<u8> {
    out.raster()[..RASTER_WIDTH * frame_height(&vi.registers) * 4].to_vec()
}

#[test]
fn scanned_frames_match_the_reference() {
    let (rdram, mut hidden) = noisy();
    let mut vi = Vi::default();
    let mut out = Scanout::default();

    let scans: Vec<(Regs, bool)> = vec![
        (regs(2, 3, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0), true),
        (regs(2, 2, 64, 0x2AB, 0x155, 108, 320, 34, 200, 0x80, 0x40), true),
        (regs(3, 2, 32, 0x400, 0x400, 108, 128, 34, 100, 0, 0), true),
        (regs(2, 3, 64, 0x400, 0x400, 108, 256, 34, 110, 0, 0).serrate(1), true),
        (regs(2, 3, 64, 0x400, 0x400, 108, 256, 34, 110, 0, 0).serrate(0), true),
        (regs(2, 3, 64, 0x400, 0x400, 108, 256, 34, 60, 0, 0), true),
        (regs(2, 3, 64, 0x400, 0x400, 108, 256, 34, 60, 0, 0), true),
        (regs(2, 3, 64, 0x400, 0x400, 108, 256, 34, 60, 0, 0), true),
        (regs(0, 3, 64, 0x400, 0x400, 148, 200, 34, 120, 0, 0), true),
        (regs(2, 3, 64, 0x400, 0x400, 40, 640, 34, 120, 0, 0), true),
        (regs(2, 3, 64, 0x400, 0x400, 128, 640, 44, 240, 0, 0).sync(625), true),
        (regs(2, 3, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0).origin(0), false),
        (regs(0, 3, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0), true),
        (regs(0, 3, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0), false),
        (regs(2, 3, 64, 0x400, 0x400, 108, 0, 34, 120, 0, 0), false),
        (regs(2, 3, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0), true),
    ];

    let mut frames = Vec::new();
    for (registers, expected) in &scans {
        vi.registers = registers.words();
        assert_eq!(*expected, scan(&mut vi, &rdram, &hidden, &mut out));
        if *expected {
            frames.push(frame_of(&vi, &out));
        }
    }

    let filtered: Vec<(Regs, i32)> = vec![
        (regs(2, 1, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0), -1),
        (regs(3, 1, 32, 0x400, 0x400, 108, 128, 34, 100, 0, 0), -1),
        (regs(2, 0, 64, 0x2AB, 0x155, 108, 320, 34, 200, 0x80, 0x40), -1),
        (regs(2, 1, 64, 0x400, 0x200, 108, 256, 34, 110, 0, 0), -1),
        (regs(2, 1, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0), 3),
        (regs(2, 1, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0), 0),
        (regs(2, 3, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0).control(DITHER_FILTER), -1),
        (regs(3, 3, 32, 0x400, 0x400, 108, 128, 34, 100, 0, 0).control(DITHER_FILTER), -1),
        (regs(2, 1, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0).control(DIVOT_ON), 3),
        (regs(2, 3, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0).control(GAMMA_ON), -1),
        (regs(2, 1, 64, 0x2AB, 0x155, 108, 256, 34, 120, 0x80, 0x40).control(DITHER_FILTER | DIVOT_ON | GAMMA_ON), -1),
    ];
    for (registers, bits) in &filtered {
        if *bits >= 0 {
            hidden[FRAMEBUFFER / 2..FRAMEBUFFER / 2 + 0x1800].fill(*bits as u8);
        }
        vi.registers = registers.words();
        assert!(scan(&mut vi, &rdram, &hidden, &mut out));
        frames.push(frame_of(&vi, &out));
    }

    #[rustfmt::skip]
    let pixels: &[(usize, usize, usize, u32)] = &[
        (0, 20, 5, 0x80C88007), (0, 100, 30, 0xB848D807), (0, 180, 60, 0x38401807), (0, 250, 100, 0x00000000),
        (1, 60, 12, 0x8396BA07), (1, 140, 44, 0x7BC36A07), (1, 220, 77, 0x62C28207),
        (2, 20, 5, 0x08B02C07), (2, 100, 30, 0xDB584907), (2, 140, 44, 0x00000000),
        (3, 20, 4, 0x68885807), (3, 100, 31, 0x1B019E07), (3, 200, 100, 0x7068F807), (3, 150, 301, 0x00000000),
        (4, 20, 5, 0x68885807), (4, 100, 30, 0xF090F807), (4, 200, 101, 0x7068F807), (4, 150, 300, 0x00000000),
        (5, 20, 5, 0x80C88007), (5, 20, 70, 0x00000000), (5, 250, 30, 0x00000000),
        (6, 20, 70, 0x00000000), (6, 100, 100, 0x00000000),
        (7, 20, 70, 0x00000000), (7, 100, 100, 0x00000000), (7, 250, 30, 0x00000000),
        (8, 60, 20, 0x90187807), (8, 20, 20, 0x00000000), (8, 300, 20, 0x00000000),
        (9, 3, 10, 0x78681007), (9, 20, 10, 0x1810B007), (9, 569, 10, 0x00000000), (9, 600, 10, 0x00000000),
        (10, 20, 5, 0x80C88007), (10, 300, 200, 0x00000007), (10, 100, 280, 0x00000000),
        (11, 20, 5, 0x80C88007), (11, 300, 40, 0x00000000),
        (12, 20, 5, 0x80C88007), (12, 100, 100, 0x00000007), (12, 300, 40, 0x00000000),
        (13, 20, 5, 0x80C88000), (13, 100, 30, 0xB848D805), (13, 180, 60, 0x38401802),
        (13, 240, 100, 0x00000000), (13, 3, 5, 0x00000000), (13, 252, 5, 0x00000000),
        (14, 20, 5, 0x08B02C03), (14, 100, 30, 0xDB584907), (14, 60, 44, 0xD2D70405),
        (15, 60, 12, 0x8396BA05), (15, 140, 44, 0x7DC08502), (15, 220, 77, 0x62C28203), (15, 300, 150, 0x9CAEDE00),
        (16, 20, 5, 0x8A728A04), (16, 100, 30, 0xF090F802), (16, 180, 60, 0x58F0E803), (16, 200, 100, 0xAFB5F800),
        (17, 20, 5, 0x7CCCA003), (17, 100, 30, 0xB848D807), (17, 180, 60, 0x7C607003),
        (18, 20, 5, 0x80C88000), (18, 100, 30, 0xB848D804), (18, 180, 60, 0x38401800),
        (19, 20, 5, 0x7CC48207), (19, 100, 30, 0xB350D007), (19, 180, 60, 0x3C412007), (19, 9, 1, 0x26866607),
        (20, 20, 5, 0x0EAC3207), (20, 100, 30, 0xD5544D07), (20, 60, 44, 0xCCD10C07),
        (21, 20, 5, 0x7C605803), (21, 100, 30, 0x80B09807), (21, 180, 60, 0x7C8C7003),
        (22, 20, 5, 0xB4E2B407), (22, 100, 30, 0xD886EA07), (22, 180, 60, 0x76804E07), (22, 250, 100, 0x00000000),
        (23, 20, 5, 0xC8C8A207), (23, 100, 30, 0xCC9CBA07), (23, 180, 60, 0x8EC8E207),
    ];
    for &(frame, x, y, color) in pixels {
        let at = (y * 640 + x) * 4;
        let p = &frames[frame][at..at + 4];
        let got = u32::from_be_bytes([p[0], p[1], p[2], p[3]]);
        assert_eq!(color, got, "frame {frame} at ({x},{y})");
    }
}

#[test]
fn a_narrower_frame_buffer_after_a_wider_one_scans_as_it_would_alone() {
    let (rdram, hidden) = noisy();
    let wide = regs(2, 0, 128, 0x400, 0x400, 108, 256, 34, 120, 0, 0).control(DITHER_FILTER | DIVOT_ON).words();
    let narrow = regs(2, 0, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0).control(DITHER_FILTER | DIVOT_ON).words();

    let (mut twice, mut twice_out) = (Vi::default(), Scanout::default());
    twice.registers = wide;
    assert!(scan(&mut twice, &rdram, &hidden, &mut twice_out));
    twice.registers = narrow;
    assert!(scan(&mut twice, &rdram, &hidden, &mut twice_out));

    let (mut once, mut once_out) = (Vi::default(), Scanout::default());
    once.registers = narrow;
    assert!(scan(&mut once, &rdram, &hidden, &mut once_out));
    assert_eq!(frame_of(&once, &once_out), frame_of(&twice, &twice_out));
}

/// The composition: the raster's rows with the coverage made opaque, sent once or twice, and the row repeat that travels with them.
#[test]
fn the_frame_is_the_rasters_rows_opaque_and_repeated_only_when_asked() {
    let (rdram, hidden) = noisy();
    for (serrate, pal, repeat_rows) in [(false, false, false), (false, false, true), (true, false, false), (true, false, true), (false, true, false)] {
        let mut registers = regs(2, 1, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0);
        if pal {
            registers = registers.sync(625);
        }
        if serrate {
            registers = registers.serrate(1);
        }
        let mut vi = Vi { registers: registers.words(), ..Vi::default() };
        let mut out = Scanout { repeat_rows, ..Scanout::default() };
        assert!(scan(&mut vi, &rdram, &hidden, &mut out));

        let rows = if pal { 576 } else { 480 } >> if serrate { 0 } else { 1 };
        let repeat = if serrate || !repeat_rows { 1 } else { 2 };
        assert_eq!((out.width, out.height), (640, (rows * repeat) as u32));
        assert_eq!(out.row_repeat, if serrate || repeat_rows { 1 } else { 2 });
        assert_eq!(out.frame.len(), 640 * rows * repeat * 4);
        for row in 0..rows * repeat {
            let from = &out.raster()[(row / repeat) * 2560..(row / repeat + 1) * 2560];
            let to = &out.frame[row * 2560..(row + 1) * 2560];
            for (a, b) in from.as_chunks::<4>().0.iter().zip(to.as_chunks::<4>().0) {
                assert_eq!([a[0], a[1], a[2], 0xFF], [b[0], b[1], b[2], b[3]]);
            }
        }
    }
}

/// A frame buffer at zero shows the raster as it stands, which a fresh machine holds black.
#[test]
fn no_frame_buffer_still_composes_the_raster() {
    let (rdram, hidden) = noisy();
    let mut vi = Vi::default();
    let mut out = Scanout::default();
    assert!(!scan(&mut vi, &rdram, &hidden, &mut out));
    assert_eq!((out.width, out.height, out.row_repeat), (640, 240, 2));
    assert!(out.frame.as_chunks::<4>().0.iter().all(|p| *p == [0, 0, 0, 0xFF]));
}

/// The FPGA's construction of the runners-up, over every configuration of a pixel and six neighbours (Mars_VideoFilter.md §5).
#[test]
fn the_runners_up_are_the_fpgas_sorted_and_clamped_pair() {
    for centre in 0..7 {
        for code in 0..8usize.pow(6) {
            let mut values = [centre; 7];
            let mut full = 1;
            let mut six = [centre; 6];
            for (n, slot) in six.iter_mut().enumerate() {
                let v = (code >> (3 * n)) & 7;
                if v < 7 {
                    *slot = v as i32;
                    values[full] = v as i32;
                    full += 1;
                }
            }
            six.sort_unstable();
            assert_eq!(runners(&values[..full]), (centre.min(six[1]), centre.max(six[4])), "{:?}", &values[..full]);
        }
    }
}

#[test]
fn the_median_is_the_middle_value_and_divot_leaves_whole_runs() {
    for c in 0..6 {
        for l in 0..6 {
            for r in 0..6 {
                let mut sorted = [c, l, r];
                sorted.sort_unstable();
                assert_eq!(median(c, l, r), sorted[1], "{c} {l} {r}");
            }
        }
    }
    let p = |v: i32, coverage: i32| Pixel { red: v, green: v + 1, blue: v + 2, coverage };
    assert_eq!(divot(p(9, 7), p(1, 7), p(2, 7)), p(9, 7));
    assert_eq!(divot(p(9, 6), p(1, 7), p(2, 7)), Pixel { red: 2, green: 3, blue: 4, coverage: 6 });
}

#[test]
fn gamma_is_twice_the_integer_square_root() {
    for i in 0..0x4000 {
        let expected = (i as f64).sqrt().floor() as i32;
        assert_eq!(root(i), expected, "{i}");
    }
}

#[test]
fn the_mix_rounds_by_half_a_step_and_keeps_the_near_coverage() {
    let near = Pixel { red: 10, green: 200, blue: 0, coverage: 3 };
    let far = Pixel { red: 20, green: 100, blue: 255, coverage: 7 };
    for fraction in 0..32 {
        let got = mix(near, far, fraction);
        let expect = |n: i32, f: i32| ((((f - n) * fraction + 16) >> 5) + n) & 0xFF;
        assert_eq!(got.coverage, 3);
        assert_eq!((got.red, got.green, got.blue), (expect(10, 20), expect(200, 100), expect(0, 255)));
    }
}

/// A walker's stamps wrap after 2^31 new lines; a scan after the wrap must not read either cache's samples from before it.
#[test]
fn a_scan_after_the_stamp_wraps_reads_no_sample_from_before_it() {
    let registers = regs(2, 3, 320, 0x400, 0x400, 108, 320, 34, 1, 0, 0).words();
    let fill = |seed: u32| {
        let (mut rdram, hidden) = noisy();
        let mut state = seed.wrapping_mul(0x9E37_79B9);
        for byte in &mut rdram[FRAMEBUFFER..FRAMEBUFFER + 0x4_0000] {
            state = state.wrapping_mul(1_103_515_245).wrapping_add(12345);
            *byte = (state >> 16) as u8;
        }
        (rdram, hidden)
    };

    let (first, hidden) = fill(1);
    let (second, _) = fill(2);
    let mut wrapped = Vi { registers, ..Vi::default() };
    let mut wrapped_out = Scanout::default();
    assert!(scan(&mut wrapped, &first, &hidden, &mut wrapped_out));
    wrapped_out.walker.set_stamp(i32::MAX - 1);
    assert!(scan(&mut wrapped, &second, &hidden, &mut wrapped_out));

    let mut fresh = Vi { registers, ..Vi::default() };
    let mut fresh_out = Scanout::default();
    assert!(scan(&mut fresh, &second, &hidden, &mut fresh_out));
    assert!(wrapped_out.raster() == fresh_out.raster());
}

/// Gamma reads the entry six bits above the channel: twice the root of the channel times 64 (Mars_VideoPasses.md §3).
#[test]
fn gamma_reads_the_entry_six_bits_above_the_channel() {
    for c in 0..256 {
        let p = gamma(Pixel { red: c, green: 255 - c, blue: c, coverage: 5 });
        let root_of = |v: i32| 2 * ((v * 64) as f64).sqrt().floor() as i32;
        assert_eq!((p.red, p.green, p.blue, p.coverage), (root_of(c), root_of(255 - c), root_of(c), 5), "{c}");
    }
}

/// The dither filter compares the top five bits and nothing below them (Mars_VideoPasses.md §1).
#[test]
fn the_dither_step_compares_five_bits() {
    for c in 0..256 {
        for n in 0..256 {
            assert_eq!(step(c, n), ((n >> 3) - (c >> 3)).signum(), "{c} {n}");
        }
    }
}

/// The FPGA's pull, `mid + ((penmin + penmax - 2 mid) * (7 - c) + 4) >> 3`, over unscaled values, where the rounding shows (Mars_VideoFilter.md §5).
#[test]
fn the_pull_is_the_fpgas_rounded_by_half_a_step() {
    for centre in 0..24 {
        for a in 0..24 {
            for b in 0..24 {
                let mut six = [centre, a, b, centre, centre, centre];
                six.sort_unstable();
                let (low, high) = (centre.min(six[1]), centre.max(six[4]));
                for missing in 0..8 {
                    let expected = centre + (((low + high - 2 * centre) * missing + 4) >> 3);
                    assert_eq!(pull(&[centre, a, b], centre, missing), expected, "{centre} {a} {b} {missing}");
                }
            }
        }
    }
}

/// C# halves the vertical start less its offset by truncating division, so one half line above the offset is the offset.
#[test]
fn a_picture_one_half_line_above_the_offset_starts_at_it() {
    let (rdram, hidden) = noisy();
    let scan_at = |top: u32| {
        let mut vi = Vi { registers: regs(2, 2, 64, 0x400, 0x2AB, 108, 256, top, 100, 0, 0).words(), ..Vi::default() };
        let mut out = Scanout::default();
        assert!(scan(&mut vi, &rdram, &hidden, &mut out));
        out.raster().to_vec()
    };
    assert!(scan_at(33) == scan_at(34));
    assert!(scan_at(32) != scan_at(34));
}

/// Eight columns at the left and seven at the right carry no signal, and a dark column keeps the coverage beneath it (Mars_Video.md §2.5).
#[test]
fn the_guard_columns_are_dark_and_keep_their_coverage() {
    let (rdram, hidden) = noisy();
    let mut vi = Vi { registers: regs(2, 3, 64, 0x400, 0x400, 40, 640, 34, 120, 0, 0).words(), ..Vi::default() };
    let mut out = Scanout::default();
    assert!(scan(&mut vi, &rdram, &hidden, &mut out));
    vi.registers = regs(2, 3, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0).words();
    assert!(scan(&mut vi, &rdram, &hidden, &mut out));

    let raster = out.raster();
    let pixel = |x: usize, y: usize| &raster[(y * 640 + x) * 4..(y * 640 + x) * 4 + 4];
    let lit = |x: usize| (0..120).any(|y| pixel(x, y)[..3] != [0, 0, 0]);
    for x in (0..8).chain(249..256) {
        assert!(!lit(x), "column {x}");
        assert!((0..120).all(|y| pixel(x, y)[3] == 7), "column {x} lost its coverage");
    }
    assert!(lit(8) && lit(248));
}


/// A picture that shrinks leaves its old lines held a field more; they expire while the next scans repeat, and the deferred picture must darken them too.
#[test]
fn a_deferred_scan_repeating_after_a_held_line_expired_shows_the_darkened_picture() {
    let (rdram, hidden) = noisy();
    let mut deferred = crate::memory::bus::MemoryBus::new(rdram.len());
    deferred.rdram[..].copy_from_slice(&rdram);
    deferred.rdram_hidden[..].copy_from_slice(&hidden);
    let mut at_once = deferred.clone();
    let (mut now, mut later) = (Scanout::default(), Scanout::default());
    let tall = regs(2, 3, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0).words();
    let short = regs(2, 3, 64, 0x400, 0x400, 108, 256, 34, 60, 0, 0).words();
    let mut previous: Option<Vec<u8>> = None;
    let mut changed = 0;
    for (step, registers) in [tall, tall, tall, short, short, short, short].into_iter().enumerate() {
        at_once.vi.registers = registers;
        deferred.vi.registers = registers;
        present_now(&mut at_once, &mut now);
        present_deferred(&mut deferred, &mut later);
        if let Some(picture) = &previous {
            assert!(later.frame == *picture, "step {step}: the deferred picture is not the immediate picture of the step before");
        }
        if previous.as_ref().is_some_and(|p| *p != now.frame) {
            changed += 1;
        }
        previous = Some(now.frame.clone());
    }
    assert!(later.repeated_scans >= 2, "no scan repeated, so the case was not reached");
    assert!(changed >= 1, "the expiry changed no picture, so the case was not reached");
}
