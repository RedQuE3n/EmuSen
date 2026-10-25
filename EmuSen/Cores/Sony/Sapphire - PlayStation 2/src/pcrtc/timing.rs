//! The PCRTC's video timing (Sapphire_Native.md §29.3): the H-blank, V-blank and V-sync signals and the field, from the
//! video mode and SMODE2's INT, on the PCRTC's own clock VCK. Times are EE cycles; a signal's edge at VCK tick `v`
//! falls at the first EE cycle at or after it, so no rounding accumulates. A change of mode or of INT takes effect at
//! the next field's start. The timers, the INTC, the GS's CSR and the IOP's timers take their signals from here.

use crate::clock;

/// How a mode scans.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scan {
    /// Interlaced or not by SMODE2's INT; a non-interlaced field has `lines` lines.
    Either { lines: u64 },
    Progressive,
    Interlaced,
}

/// A video mode's geometry in VCK and lines (GS User's Manual §5.1.2's table; the choices are §29.3's).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mode {
    /// VCK ticks per EE cycle, as a fraction (`clock::VCK_54`, `clock::VCK_74`).
    pub vck: (u64, u64),
    /// VCK in one line.
    pub line: u64,
    /// Lines in an interlaced frame (two fields), or in a progressive field.
    pub lines: u64,
    pub scan: Scan,
    /// Lines from a field's start during which V-blank is high.
    pub vblank: u64,
    /// VCK from each line's start during which H-blank is high.
    pub hblank: u64,
    /// Lines from a field's start to its V-sync, where VSINT is raised and CSR's FIELD changes.
    pub vsync: u64,
    /// The picture with neither read circuit on: VCK a pixel, pixels, rasters.
    pub picture: (u64, u32, u32),
    /// The picture's shape on the screen.
    pub aspect: (u32, u32),
}

/// NTSC: 858 pixels of 13.5 MHz a line (4 VCK each), 525 lines a frame, 263 a non-interlaced field.
pub const NTSC: Mode = Mode {
    vck: clock::VCK_54,
    line: 3432,
    lines: 525,
    scan: Scan::Either { lines: 263 },
    vblank: 22,
    hblank: 552,
    vsync: 4,
    picture: (4, 640, 448),
    aspect: (4, 3),
};

/// PAL: 864 pixels of 13.5 MHz a line, 625 lines a frame, 314 a non-interlaced field.
pub const PAL: Mode = Mode {
    vck: clock::VCK_54,
    line: 3456,
    lines: 625,
    scan: Scan::Either { lines: 314 },
    vblank: 26,
    hblank: 576,
    vsync: 4,
    picture: (4, 640, 512),
    aspect: (4, 3),
};

/// DTV 480p: 1,716 VCK a line at 54 MHz, 525 lines a progressive field.
pub const DTV480P: Mode =
    Mode { vck: clock::VCK_54, line: 1716, lines: 525, scan: Scan::Progressive, vblank: 45, hblank: 276, vsync: 4, picture: (2, 640, 480), aspect: (4, 3) };

/// DTV 1080i: 2,200 VCK a line at 74.25 MHz, 1,125 lines a frame.
pub const DTV1080I: Mode = Mode {
    vck: clock::VCK_74,
    line: 2200,
    lines: 1125,
    scan: Scan::Interlaced,
    vblank: 22,
    hblank: 280,
    vsync: 4,
    picture: (1, 1920, 1080),
    aspect: (16, 9),
};

impl Mode {
    pub fn interlaced(&self, int: bool) -> bool {
        match self.scan {
            Scan::Either { .. } => int,
            Scan::Progressive => false,
            Scan::Interlaced => true,
        }
    }

    /// VCK in one field.
    pub fn field(&self, int: bool) -> u64 {
        match (self.scan, self.interlaced(int)) {
            (_, true) => self.line * self.lines / 2,
            (Scan::Either { lines }, false) => self.line * lines,
            _ => self.line * self.lines,
        }
    }

    /// Rasters in a frame: both fields' lines when interlaced, a field's otherwise.
    pub fn rasters(&self, int: bool) -> u32 {
        (if self.interlaced(int) { self.lines } else { self.field(int) / self.line }) as u32
    }

    /// The EE cycle at which VCK tick `vck` falls, counted from a timing's start.
    pub fn cycles(&self, vck: u64) -> u64 {
        (vck as u128 * self.vck.1 as u128).div_ceil(self.vck.0 as u128) as u64
    }

    /// Fields a second, as a fraction.
    pub fn field_rate(&self, int: bool) -> (u64, u64) {
        let (num, den) = (clock::EE_HZ * self.vck.0, self.vck.1 * self.field(int));
        let g = gcd(num, den);
        (num / g, den / g)
    }
}

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd(b, a % b) }
}

/// What changes at an edge: H-blank's and V-blank's rise and fall, and the V-sync.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Edges {
    pub h_rise: bool,
    pub h_fall: bool,
    pub v_rise: bool,
    pub v_fall: bool,
    pub vsync: bool,
}

impl Edges {
    pub fn any(&self) -> bool {
        self.h_rise || self.h_fall || self.v_rise || self.v_fall || self.vsync
    }
}

/// One stretch of timing: a mode and INT from `epoch`, the EE cycle of VCK tick 0 and of field 0's start.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Span {
    mode: Mode,
    int: bool,
    epoch: u64,
}

impl Span {
    /// VCK ticks whole by EE cycle `t`; none before the epoch.
    fn ticks(&self, t: u64) -> Option<u64> {
        (t >= self.epoch).then(|| ((t - self.epoch) as u128 * self.mode.vck.0 as u128 / self.mode.vck.1 as u128) as u64)
    }

    /// The EE cycle at which tick `v` falls: the first whose ticks reach it.
    fn time(&self, v: u64) -> u64 {
        self.epoch + (v as u128 * self.mode.vck.1 as u128).div_ceil(self.mode.vck.0 as u128) as u64
    }

    /// The tick that falls at exactly `t`, if one does.
    fn lands(&self, t: u64) -> Option<u64> {
        let v = self.ticks(t)?;
        if t == self.epoch {
            return Some(0);
        }
        (self.ticks(t - 1) != Some(v)).then_some(v)
    }

    fn field(&self) -> u64 {
        self.mode.field(self.int)
    }

    fn edges_at(&self, v: u64) -> Edges {
        let m = &self.mode;
        let q = v % self.field();
        Edges {
            h_rise: v.is_multiple_of(m.line),
            h_fall: v % m.line == m.hblank,
            v_rise: q == 0,
            v_fall: q == m.vblank * m.line,
            vsync: q == m.vsync * m.line,
        }
    }

    /// The first tick after `v` at which a signal changes; H-blank's only when `want_h`.
    fn next_tick(&self, v: u64, want_h: bool) -> u64 {
        let m = &self.mode;
        let f = self.field();
        let base = v - v % f;
        let q = v % f;
        let mut next = base + f;
        for at in [m.vsync * m.line, m.vblank * m.line] {
            if at > q {
                next = next.min(base + at);
            }
        }
        if want_h {
            let lb = v - v % m.line;
            let fall = lb + m.hblank;
            next = next.min(if fall > v { fall } else { lb + m.line });
        }
        next
    }
}

/// The signals' timing, with a change waiting for the next field's start.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Timing {
    now: Span,
    next: Option<Span>,
}

/// A field: its number since the timing's epoch, whether it is odd, and the EE cycle it starts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Field {
    pub number: u64,
    pub odd: bool,
    pub start: u64,
}

impl Timing {
    /// The mode non-interlaced from EE cycle 0, as after a reset (SMODE2 reads 0).
    pub fn new(mode: Mode) -> Timing {
        Timing { now: Span { mode, int: false, epoch: 0 }, next: None }
    }

    fn span(&self, t: u64) -> &Span {
        match &self.next {
            Some(n) if t >= n.epoch => n,
            _ => &self.now,
        }
    }

    /// The mode and INT in force at `t`.
    pub fn mode(&self, t: u64) -> (Mode, bool) {
        let s = self.span(t);
        (s.mode, s.int)
    }

    /// The mode and INT the timing will have once any change waiting has taken effect.
    pub fn target(&self) -> (Mode, bool) {
        let s = self.next.as_ref().unwrap_or(&self.now);
        (s.mode, s.int)
    }

    /// Asks for `mode` with INT `int` from the first field that starts after `t`; nothing if that is already so.
    pub fn change(&mut self, t: u64, mode: Mode, int: bool) {
        self.settle(t);
        if self.target() == (mode, int) {
            return;
        }
        let s = self.now;
        let v = s.ticks(t).unwrap_or(0);
        let start = s.time(v - v % s.field() + s.field());
        self.next = if (s.mode, s.int) == (mode, int) { None } else { Some(Span { mode, int, epoch: start }) };
    }

    /// Takes a change whose field has begun by `t`.
    pub fn settle(&mut self, t: u64) {
        if let Some(n) = self.next
            && t >= n.epoch
        {
            self.now = n;
            self.next = None;
        }
    }

    pub fn h(&self, t: u64) -> bool {
        let s = self.span(t);
        s.ticks(t).is_some_and(|v| v % s.mode.line < s.mode.hblank)
    }

    pub fn v(&self, t: u64) -> bool {
        let s = self.span(t);
        s.ticks(t).is_some_and(|v| v % s.field() < s.mode.vblank * s.mode.line)
    }

    /// The first EE cycle after `t` at which a signal changes, or a waiting change takes effect; H-blank's edges only
    /// when `want_h`.
    pub fn next_edge(&self, t: u64, want_h: bool) -> u64 {
        let s = self.span(t);
        let at = match s.ticks(t) {
            Some(v) => s.time(s.next_tick(v, want_h)),
            None => s.epoch,
        };
        match &self.next {
            Some(n) if n.epoch > t => at.min(n.epoch),
            _ => at,
        }
    }

    /// The edges at exactly `t`.
    pub fn edges(&self, t: u64) -> Edges {
        let s = self.span(t);
        s.lands(t).map(|v| s.edges_at(v)).unwrap_or_default()
    }

    /// The field in progress at `t`.
    pub fn field(&self, t: u64) -> Field {
        let s = self.span(t);
        let n = s.ticks(t).unwrap_or(0) / s.field();
        Field { number: n, odd: n & 1 == 1, start: s.time(n * s.field()) }
    }

    /// The EE cycle at which line `l` of the field in progress at `t` starts, counted from the field's first H-blank
    /// (half a line in, for an interlaced odd field); none past the field's end.
    pub fn line_start(&self, t: u64, l: u64) -> Option<u64> {
        let s = self.span(t);
        let f = s.field();
        let n = s.ticks(t).unwrap_or(0) / f;
        let first = (n * f).div_ceil(s.mode.line) * s.mode.line;
        let v = first + l * s.mode.line;
        (v < (n + 1) * f).then(|| s.time(v))
    }

    /// The raster of line `l` of `field`: both fields' lines interleaved when interlaced, even rasters the even field's.
    pub fn raster(&self, t: u64, field: &Field, l: u64) -> u32 {
        let s = self.span(t);
        (if s.mode.interlaced(s.int) { 2 * l + field.odd as u64 } else { l }) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_four_rates_of_the_manuals_table_follow_from_13_5_mhz() {
        let hz = |m: Mode, int| {
            let (n, d) = m.field_rate(int);
            n as f64 / d as f64
        };
        assert_eq!(NTSC.field_rate(true), (60_000, 1_001));
        assert_eq!(PAL.field_rate(true), (50, 1));
        assert_eq!(format!("{:.2}", hz(NTSC, false)), "59.83");
        assert_eq!(format!("{:.2}", hz(PAL, false)), "49.76");
        assert_eq!(format!("{:.2}", hz(DTV480P, false)), "59.94");
        assert_eq!(format!("{:.2}", hz(DTV1080I, true)), "60.00");
        let line_khz = |m: Mode| clock::EE_HZ as f64 * m.vck.0 as f64 / m.vck.1 as f64 / m.line as f64 / 1000.0;
        for (m, khz) in [(NTSC, "15.734"), (PAL, "15.625"), (DTV480P, "31.469"), (DTV1080I, "33.750")] {
            assert_eq!(format!("{:.3}", line_khz(m)), khz);
        }
    }

    #[test]
    fn an_interlaced_field_is_half_a_frame_and_the_odd_one_starts_half_a_line_in() {
        let mut t = Timing::new(NTSC);
        t.change(0, NTSC, true);
        t.settle(u64::MAX);
        let f0 = t.field(t.now.epoch);
        let one = t.now.time(NTSC.field(true));
        let f1 = t.field(one);
        assert!(!f0.odd && f1.odd && f1.start == one);
        assert_eq!(t.line_start(one, 0), Some(t.now.time(NTSC.field(true) + NTSC.line / 2)));
        let lines = |at| (0..).take_while(|&l| t.line_start(at, l).is_some()).count();
        assert_eq!((lines(f0.start), lines(f1.start)), (263, 262));
        assert_eq!((t.raster(one, &f1, 0), t.raster(f0.start, &f0, 1)), (1, 2));
    }

    #[test]
    fn edges_fall_once_each_and_a_change_waits_for_the_field() {
        let mut t = Timing::new(NTSC);
        let (mut h, mut v, mut s, mut at) = (0, 0, 0, 0);
        let field = t.now.time(NTSC.field(false));
        while at < field {
            at = t.next_edge(at, true);
            let e = t.edges(at);
            assert!(e.any(), "an edge at {at}");
            h += e.h_rise as u32;
            v += e.v_fall as u32;
            s += e.vsync as u32;
        }
        assert_eq!((h, v, s), (263, 1, 1), "lines 1 to 262 and the next field's first");
        t.change(100, NTSC, true);
        assert_eq!(t.mode(field - 1).1, false);
        assert_eq!(t.mode(field + NTSC.field(false)).1, true);
        assert_eq!(t.next_edge(field - 10, false), field);
        assert_eq!((t.field(field + 1).start, t.field(field + 1).number), (field, 0), "the new timing's first field");
    }
}
