//! The GPU's video timing, the second clock domain (Diamond_Plan.md §5.1, Diamond_Disputes.md D-0): lines and fields
//! on the console's video clock, the blanks the display ranges make, and the dot clock and hblanks the timers count.
//! Positions are in half video clocks so that a line of 3,412.5 clocks is whole (Diamond_Native.md §11.5).

/// Half video clocks per CPU cycle as a fraction: 2 × 53.693175 MHz (NTSC) or 2 × 53.203425 MHz (PAL) over
/// 33.8688 MHz, reduced.
pub const NTSC_CLOCK: (u64, u64) = (1_431_818, 451_584);
pub const PAL_CLOCK: (u64, u64) = (1_418_758, 451_584);

/// A line in half video clocks, provisional on D-0: 3,412.5 video clocks in NTSC mode, 3,405 in PAL mode.
pub const NTSC_LINE: u64 = 6_825;
pub const PAL_LINE: u64 = 6_810;
pub const NTSC_LINES: u32 = 263;
pub const PAL_LINES: u32 = 314;

/// What happened at an edge.
pub mod edge {
    pub const FIELD: u32 = 1 << 0;
    pub const VBLANK_START: u32 = 1 << 1;
    pub const VBLANK_END: u32 = 1 << 2;
    pub const HBLANK_START: u32 = 1 << 3;
    pub const HBLANK_END: u32 = 1 << 4;
}

#[derive(Clone, Debug)]
pub struct Video {
    pub clock: (u64, u64),
    /// GP1(08h)'s bits, and the display ranges of GP1(06h) and GP1(07h).
    pub mode: u32,
    pub x1: u32,
    pub x2: u32,
    pub y1: u32,
    pub y2: u32,
    /// The current timing segment, since the last mode change: its start in absolute half clocks, the absolute line
    /// it starts on, and the dots and hblank edges counted before it.
    pub(crate) base_h: u64,
    pub(crate) base_line: u64,
    pub(crate) base_dots: u64,
    pub(crate) base_hblanks: u64,
    /// The field being scanned: its first absolute line, its lines, and whether it is the odd field.
    pub(crate) field_line: u64,
    pub(crate) field_lines: u32,
    pub odd: bool,
    /// The edges processed up to this absolute half clock.
    pub(crate) done_h: u64,
}

impl Video {
    pub fn new(pal_clock: bool) -> Video {
        let mut v = Video {
            clock: if pal_clock { PAL_CLOCK } else { NTSC_CLOCK },
            mode: 0,
            x1: 0x200,
            x2: 0x200 + 256 * 10,
            y1: 0x10,
            y2: 0x10 + 240,
            base_h: 0,
            base_line: 0,
            base_dots: 0,
            base_hblanks: 0,
            field_line: 0,
            field_lines: 0,
            odd: false,
            done_h: 0,
        };
        v.field_lines = v.lines_of_field();
        v
    }

    /// The absolute half video clock at CPU cycle `t`, and the first CPU cycle at or after half clock `h`.
    pub fn h_at(&self, t: u64) -> u64 {
        (t as u128 * self.clock.0 as u128 / self.clock.1 as u128) as u64
    }

    pub fn t_at(&self, h: u64) -> u64 {
        (h as u128 * self.clock.1 as u128).div_ceil(self.clock.0 as u128) as u64
    }

    pub fn pal_mode(&self) -> bool {
        self.mode & 1 << 3 != 0
    }

    pub fn interlaced(&self) -> bool {
        self.mode & 1 << 5 != 0
    }

    pub fn line_length(&self) -> u64 {
        if self.pal_mode() { PAL_LINE } else { NTSC_LINE }
    }

    /// 263 or 314 lines; interlaced, 263 and 262 or 313 and 312 by field, averaging 262.5 or 312.5 (D-0).
    fn lines_of_field(&self) -> u32 {
        let full = if !self.pal_mode() { NTSC_LINES } else if self.interlaced() { PAL_LINES - 1 } else { PAL_LINES };
        full - (self.interlaced() && self.odd) as u32
    }

    /// The video clocks of a dot by the horizontal resolution (psx-spx "GPU Timings").
    pub fn dot_divider(&self) -> u64 {
        if self.mode & 1 << 6 != 0 {
            return 7;
        }
        [10, 8, 5, 4][(self.mode & 3) as usize]
    }

    fn dots_per_line(&self) -> u64 {
        self.line_length() / (2 * self.dot_divider())
    }

    /// The absolute line and the half clock within it at `h`.
    fn line_at(&self, h: u64) -> (u64, u64) {
        let rel = h.saturating_sub(self.base_h);
        let len = self.line_length();
        (self.base_line + rel / len, rel % len)
    }

    fn line_start(&self, line: u64) -> u64 {
        self.base_h + (line - self.base_line) * self.line_length()
    }

    /// Where in a line hblank begins, if it begins at all: the range X1..X2 is the picture, the rest is blank.
    fn hblank_rise(&self) -> Option<u64> {
        let (a, b, len) = (self.x1 as u64 * 2, (self.x2 as u64 * 2).min(self.line_length()), self.line_length());
        if a >= b {
            None
        } else if b < len {
            Some(b)
        } else if a > 0 {
            Some(0)
        } else {
            None
        }
    }

    fn in_hblank(&self, pos: u64) -> bool {
        pos < self.x1 as u64 * 2 || pos >= self.x2 as u64 * 2
    }

    /// The field line at which vblank begins, if it does.
    fn vblank_rise(&self) -> Option<u32> {
        let (a, b, n) = (self.y1, self.y2.min(self.field_lines), self.field_lines);
        if a >= b {
            None
        } else if b < n {
            Some(b)
        } else if a > 0 {
            Some(0)
        } else {
            None
        }
    }

    /// The absolute line at CPU cycle `t`, and the first absolute line of the field being scanned.
    pub fn position(&self, t: u64) -> (u64, u64) {
        (self.line_at(self.h_at(t)).0, self.field_line)
    }

    pub fn field_lines(&self) -> u32 {
        self.field_lines
    }

    pub fn in_vblank_at(&self, t: u64) -> bool {
        let (line, _) = self.line_at(self.h_at(t));
        let l = (line - self.field_line) as u32;
        l < self.y1 || l >= self.y2
    }

    pub fn in_hblank_at(&self, t: u64) -> bool {
        self.in_hblank(self.line_at(self.h_at(t)).1)
    }

    /// The dot clock's edges since power-on at `t`; a line holds a whole number of them (psx-spx: the fraction is
    /// dropped).
    pub fn dots(&self, t: u64) -> u64 {
        let (line, pos) = self.line_at(self.h_at(t));
        let n = self.dots_per_line();
        self.base_dots + (line - self.base_line) * n + (pos / (2 * self.dot_divider())).min(n)
    }

    /// The first CPU cycle at which `dots` reaches `k`.
    pub fn t_of_dots(&self, k: u64) -> u64 {
        let n = self.dots_per_line();
        let rel = k.saturating_sub(self.base_dots);
        let (lines, rest) = (rel / n, rel % n);
        let h = self.line_start(self.base_line + lines) + rest * 2 * self.dot_divider();
        self.t_at(h)
    }

    /// Hblank's rising edges since power-on at `t`.
    pub fn hblanks(&self, t: u64) -> u64 {
        let Some(rise) = self.hblank_rise() else { return self.base_hblanks };
        let (line, pos) = self.line_at(self.h_at(t));
        self.base_hblanks + (line - self.base_line) + (pos >= rise) as u64
    }

    pub fn t_of_hblanks(&self, k: u64) -> u64 {
        let Some(rise) = self.hblank_rise() else { return u64::MAX };
        let rel = k.saturating_sub(self.base_hblanks);
        if rel == 0 {
            return 0;
        }
        self.t_at(self.line_start(self.base_line + rel - 1) + rise)
    }

    /// A change to the mode or the ranges at `t`: the counts so far are kept and the rest of the field follows the new
    /// timing from the current line.
    pub fn retime(&mut self, t: u64, change: impl FnOnce(&mut Video)) {
        let h = self.h_at(t);
        let (line, pos) = self.line_at(h);
        let dots = self.dots(t) - self.dots_from_line_start(t);
        let hblanks = self.hblanks(t);
        let start = self.line_start(line);
        change(self);
        self.base_h = start;
        self.base_line = line;
        self.base_dots = dots;
        self.base_hblanks = hblanks - self.hblank_rise().is_some_and(|r| pos >= r) as u64;
        self.field_lines = self.lines_of_field().max((line - self.field_line + 1) as u32);
    }

    fn dots_from_line_start(&self, t: u64) -> u64 {
        let (_, pos) = self.line_at(self.h_at(t));
        (pos / (2 * self.dot_divider())).min(self.dots_per_line())
    }

    /// The next edge after the last one processed: its CPU cycle. Hblank's edges are included only when `hblank`.
    pub fn next_edge(&self, hblank: bool) -> u64 {
        let h = self.next_edge_h(hblank);
        self.t_at(h)
    }

    fn next_edge_h(&self, hblank: bool) -> u64 {
        let after = self.done_h;
        let len = self.line_length();
        let field_end = self.line_start(self.field_line + self.field_lines as u64);
        let mut best = field_end;
        let mut consider = |h: u64| {
            if h > after && h < best {
                best = h;
            }
        };
        let field_line = |l: u32| self.line_start(self.field_line + l as u64);
        if let Some(l) = self.vblank_rise() {
            consider(field_line(l));
        }
        if self.y1 < self.y2.min(self.field_lines) && self.y1 > 0 {
            consider(field_line(self.y1));
        }
        if hblank {
            let (line, _) = self.line_at(after.max(self.base_h));
            for l in [line, line + 1] {
                let s = self.line_start(l);
                if let Some(r) = self.hblank_rise() {
                    consider(s + r);
                }
                let x1 = self.x1 as u64 * 2;
                if x1 > 0 && x1 < len {
                    consider(s + x1);
                }
            }
        }
        best
    }

    /// Processes the edge at CPU cycle `t` (the one `next_edge` gave) and says what it was.
    pub fn edge(&mut self, t: u64, hblank: bool) -> u32 {
        let h = self.next_edge_h(hblank);
        debug_assert!(self.t_at(h) <= t);
        self.done_h = h;
        let field_end = self.line_start(self.field_line + self.field_lines as u64);
        let mut what = 0;
        if h == field_end {
            self.field_line += self.field_lines as u64;
            self.odd = !self.odd && self.interlaced();
            self.field_lines = self.lines_of_field();
            what |= edge::FIELD;
        }
        let (line, pos) = self.line_at(h);
        let l = (line - self.field_line) as u32;
        if pos == 0 && Some(l) == self.vblank_rise() {
            what |= edge::VBLANK_START;
        }
        if pos == 0 && l == self.y1 && self.y1 < self.y2.min(self.field_lines) {
            what |= edge::VBLANK_END;
        }
        if hblank {
            if Some(pos) == self.hblank_rise() {
                what |= edge::HBLANK_START;
            }
            if pos == self.x1 as u64 * 2 && pos > 0 {
                what |= edge::HBLANK_END;
            }
        }
        what
    }

    /// GPUSTAT's bit 31 and bit 13 (psx-spx "GPU Status Register"): the line's parity in 240-line mode, the field's
    /// interlaced, zero in vblank; the interlace field, always 1 when not interlaced.
    pub fn status_bits(&self, t: u64) -> u32 {
        let (line, _) = self.line_at(self.h_at(t));
        let l = (line - self.field_line) as u32;
        let vblank = l < self.y1 || l >= self.y2;
        let hires = self.interlaced() && self.mode & 1 << 2 != 0;
        let odd = if hires { self.odd } else { line & 1 != 0 };
        let mut bits = 0;
        if odd && !vblank {
            bits |= 1 << 31;
        }
        if !self.interlaced() || self.odd {
            bits |= 1 << 13;
        }
        bits
    }

    /// The CPU cycles of a non-interlaced field in the console's own mode, as a rational frame rate (cycles per
    /// second, cycles per field).
    pub fn field_rate(&self) -> (u64, u64) {
        let lines = if self.pal_mode() { PAL_LINES } else { NTSC_LINES } as u64;
        let (num, den) = self.clock;
        // half clocks per second over half clocks per field
        let per_second = 33_868_800 * num / den;
        let g = gcd(per_second, self.line_length() * lines);
        (per_second / g, self.line_length() * lines / g)
    }
}

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd(b, a % b) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(v: &mut Video, until: u64, hblank: bool) -> Vec<(u64, u32)> {
        let mut out = Vec::new();
        loop {
            let t = v.next_edge(hblank);
            if t > until {
                return out;
            }
            out.push((t, v.edge(t, hblank)));
        }
    }

    #[test]
    fn the_rates_are_d0s_provisional_ones() {
        let hz = |(n, d): (u64, u64)| n as f64 / d as f64;
        assert!((hz(Video::new(false).field_rate()) - 59.8261).abs() < 1e-4, "{}", hz(Video::new(false).field_rate()));
        let mut pal = Video::new(true);
        pal.mode = 1 << 3;
        assert!((hz(pal.field_rate()) - 49.7614).abs() < 1e-4, "{}", hz(pal.field_rate()));
    }

    /// Interlaced, two fields hold 525 or 625 lines, as psx-spx's 262.5 and 312.5 and the referees have it (D-0).
    #[test]
    fn interlaced_fields_alternate_about_262_5_and_312_5_lines() {
        for (pal, lines) in [(false, [263, 262]), (true, [313, 312])] {
            let mut v = Video::new(pal);
            v.mode = 1 << 5 | (pal as u32) << 3;
            let edges = run(&mut v, 3_000_000, true);
            let fields: Vec<u64> = edges.iter().filter(|e| e.1 & edge::FIELD != 0).map(|e| e.0).collect();
            let counts: Vec<u64> = fields.windows(2).map(|w| edges.iter().filter(|e| e.0 >= w[0] && e.0 < w[1] && e.1 & edge::HBLANK_START != 0).count() as u64).collect();
            assert!(counts.len() >= 2 && counts.iter().all(|&c| lines.contains(&(c as u32))) && counts[0] != counts[1], "{pal}: {counts:?}");
        }
    }

    #[test]
    fn a_field_has_its_lines_vblank_and_hblanks() {
        let mut v = Video::new(false);
        let edges = run(&mut v, 2_000_000, true);
        let fields: Vec<u64> = edges.iter().filter(|e| e.1 & edge::FIELD != 0).map(|e| e.0).collect();
        assert_eq!(fields.len(), 3);
        let period = fields[1] - fields[0];
        assert!((566_121..=566_122).contains(&period), "{period}");
        let vstarts = edges.iter().filter(|e| e.1 & edge::VBLANK_START != 0).count();
        assert_eq!(vstarts, 3);
        let hb = edges.iter().filter(|e| e.0 >= fields[0] && e.0 < fields[1] && e.1 & edge::HBLANK_START != 0).count();
        assert_eq!(hb, 263);
        assert_eq!(v.hblanks(fields[1]) - v.hblanks(fields[0]), 263);
        assert_eq!(v.dots(fields[1]) - v.dots(fields[0]), 263 * 341, "256-dot mode: floor(3412.5 / 10) a line");
        assert_eq!(v.dots(v.t_of_dots(1000)), 1000);
        assert_eq!(v.dots(v.t_of_dots(1000) - 1), 999);
        assert_eq!(v.hblanks(v.t_of_hblanks(10)), 10);
        assert_eq!(v.hblanks(v.t_of_hblanks(10) - 1), 9);
    }

    #[test]
    fn a_mode_change_keeps_the_counts() {
        let mut v = Video::new(false);
        let t = 100_000;
        let (d, h) = (v.dots(t), v.hblanks(t));
        let line_start = d - v.dots_from_line_start(t);
        v.retime(t, |v| v.mode = 1);
        assert_eq!(v.dots(t) - v.dots_from_line_start(t), line_start, "the count at the line's start is kept");
        assert_eq!(v.hblanks(t), h);
        let mut w = Video::new(false);
        let late = w.t_at(w.line_start(3) + 6000);
        let before = w.hblanks(late);
        w.retime(late, |v| v.y2 = 200);
        assert_eq!(w.hblanks(late), before, "an edge already passed in the line is kept");
        let a = v.t_of_dots(v.dots(t) + 10 * 426);
        assert!(v.hblanks(a) - h == 10 || v.hblanks(a) - h == 11, "426 dots a line at 320 pixels");
    }
}
