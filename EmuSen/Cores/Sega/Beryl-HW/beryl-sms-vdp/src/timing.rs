//! A line's and a frame's geometry (Beryl_SmsVdp.md §3): the dots of a line counted from the V counter's step, the
//! H counter at each dot, the V counter on each line, and the dots at which the line's events fall.

/// The chip's clock a line: 342 dots of two clocks each, the master clock over ten a dot [R MacDonald §11: "a scanline
/// consists of 342 pixels"; FluBBa's VDP test readme's "MClock (0->683)"].
pub const LINE_CLOCKS: u64 = 684;
pub const DOTS: u32 = 342;

/// Lines a frame: 262 NTSC, 313 PAL [R MacDonald §11].
pub const NTSC_LINES: u32 = 262;
pub const PAL_LINES: u32 = 313;

/// The H counter's last value before its jump and the value it jumps to: it counts $000-$127, then $1D2-$1FF, 342
/// values, read as their top eight bits [M FluBBa's VDP test's table of H counter reads, which runs $00-$93, $E9-$FF].
pub const H_LAST: u16 = 0x127;
pub const H_JUMP: u16 = 0x1D2;

/// The H counter at dot 0, where the V counter steps and, on the line after the last active one, the frame interrupt
/// is raised. FluBBa's readme puts the frame interrupt, the line interrupt and the V counter's step at HClock -18, -17
/// and -16; its test's timing cases, run on Endou, hold the three to $1E8, $1E9 and $1E8 (Beryl_SmsVdp.md §3.2).
pub const H_AT_STEP: u16 = 0x1E8;

/// Dots after the V counter's step (Beryl_SmsVdp.md §3.2).
/// The line counter steps and the X scroll is taken for the line [R the readme: "XScroll is latched between HClock -18
/// & -17"].
pub const LINE_DOT: u32 = 1;
/// Pixel 0, ten dots after the H counter's zero by the test's collision timing case (Beryl_SmsVdp.md §3.2).
pub const PIXEL_0_DOT: u32 = dot_of(10);
/// The H counter is latched this many of the chip's clocks after TH rises, by the test's H counter table.
pub const LATCH_DELAY: u64 = 2;

/// The position in the H counter's run of 342 values of the 9-bit value `h`, $000 first.
const fn run_index(h: u16) -> u32 {
    if h <= H_LAST { h as u32 } else { (H_LAST as u32 + 1) + (h - H_JUMP) as u32 }
}

/// The dot, from the V counter's step, at which the H counter reads `h`.
pub const fn dot_of(h: u16) -> u32 {
    (run_index(h) + DOTS - run_index(H_AT_STEP)) % DOTS
}

/// The 9-bit H counter at `dot`.
pub fn h_counter(dot: u32) -> u16 {
    let i = (dot + run_index(H_AT_STEP)) % DOTS;
    if i <= H_LAST as u32 { i as u16 } else { H_JUMP + (i - H_LAST as u32 - 1) as u16 }
}

/// The V counter on `line` of a frame with `active` lines: linear, then a jump back so that a frame's values fit eight
/// bits [R MacDonald §11; SMS Power!'s "Scanline Counter"]. NTSC's 240-line display "does not work"; Endou keeps its
/// 262 lines and lets the counter wrap through zero [A].
pub fn v_counter(line: u32, active: u32, pal: bool) -> u8 {
    let v = match (pal, active) {
        (false, 192) if line > 0xDA => line - 6,
        (false, 224) if line > 0xEA => line - 6,
        (true, 192) if line > 0xF2 => line - 57,
        (true, 224 | 240) if line > 0xFF + if active == 224 { 3 } else { 11 } => line - 57,
        _ => line,
    };
    v as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(active: u32, pal: bool) -> Vec<u8> {
        (0..if pal { PAL_LINES } else { NTSC_LINES }).map(|l| v_counter(l, active, pal)).collect()
    }

    /// The sequences MacDonald gives, each value once per line, in order.
    #[test]
    fn the_v_counter_runs_as_the_document_lists_it() {
        let seq = |parts: &[(u32, u32)]| parts.iter().flat_map(|&(a, b)| (a..=b).map(|v| v as u8)).collect::<Vec<_>>();
        assert_eq!(run(192, false), seq(&[(0x00, 0xDA), (0xD5, 0xFF)]));
        assert_eq!(run(224, false), seq(&[(0x00, 0xEA), (0xE5, 0xFF)]));
        assert_eq!(run(240, false), seq(&[(0x00, 0xFF), (0x00, 0x05)]));
        assert_eq!(run(192, true), seq(&[(0x00, 0xF2), (0xBA, 0xFF)]));
        assert_eq!(run(224, true), seq(&[(0x00, 0xFF), (0x00, 0x02), (0xCA, 0xFF)]));
        assert_eq!(run(240, true), seq(&[(0x00, 0xFF), (0x00, 0x0A), (0xD2, 0xFF)]));
    }

    /// FluBBa's test reads the H counter every 229 Z80 cycles, a line and a cycle, and holds the reads to a table that
    /// runs $00-$93 then $E9-$FF; a dot is two-thirds of a cycle, so each value is read on its own dots.
    #[test]
    fn the_h_counter_runs_through_its_jump_once_a_line() {
        let reads: Vec<u8> = (0..DOTS).map(|d| (h_counter(d) >> 1) as u8).collect();
        let mut distinct = reads.clone();
        distinct.dedup();
        let first = distinct.iter().position(|&v| v == 0).unwrap();
        distinct.rotate_left(first);
        let want: Vec<u8> = (0x00..=0x93).chain(0xE9..=0xFF).collect();
        assert_eq!(distinct, want);
        assert_eq!(h_counter(0), H_AT_STEP);
        assert_eq!((h_counter(PIXEL_0_DOT), h_counter(LINE_DOT)), (10, 0x1E9));
        assert_eq!(PIXEL_0_DOT, 34);
    }
}
