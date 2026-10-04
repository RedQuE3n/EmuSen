//! The ST010's open replacement (F1 ROC II): its mailbox and commands as characterised and graded in
//! VenusRT_Native.md §43, over `dsphle`'s ports. The chip's RAM is the battery file, so every command writes exactly the
//! words the chip's program writes, its working words included.

use super::dsphle::{DspHle, Next};

/// The mailbox's poll: the cycle of one poll of word 10h, every 3 cycles from it.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Mail {
    pub(super) poll: u64,
}

/// Cycles from the poll that sees the busy bit to its clearing, for the commands whose time is a constant (§43).
fn work(command: u8) -> u64 {
    match command {
        0x00 => 15,
        0x01 => 136,
        0x03 => 31,
        0x05 => 272,
        0x06 => 24,
        0x07 => 4987,
        0x08 => 44,
        _ => 24,
    }
}

fn s16(v: u16) -> i32 {
    v as i16 as i32
}

/// The µPD96050's product K·L·2 as 32 bits (fullsnes), and its high word.
fn product(k: i32, l: i32) -> u32 {
    (k * l).wrapping_mul(2) as u32
}

fn high(k: i32, l: i32) -> i32 {
    (product(k, l) as i32) >> 16
}

/// The sine of the closest member of §43.1's family, 256 a turn: round(32,767·sin) by quadrant, a named loss.
fn sine(i: usize) -> i32 {
    static TABLE: std::sync::OnceLock<[i16; 65]> = std::sync::OnceLock::new();
    let quarter = TABLE.get_or_init(|| std::array::from_fn(|q| (32767.0 * super::dsphle::quarter_sine(q) + 0.5).floor().min(32767.0) as i16));
    let half = i % 128;
    let m = quarter[if half > 64 { 128 - half } else { half }] as i32;
    if i % 256 >= 128 { -m } else { m }
}

/// The quarter-wave sine words, for the independent generator's check.
pub fn sine_image() -> Vec<u8> {
    (0..256).flat_map(|i| (sine(i) as i16).to_le_bytes()).collect()
}

impl DspHle {
    /// The start word was read: the mailbox's poll begins.
    pub(super) fn st_begin(&mut self) {
        self.mail.poll = self.cycles + 2;
    }

    /// Bit 7 of byte 21h set: the command runs from the first poll at or after now.
    pub(super) fn st_command(&mut self) {
        self.command = self.ram[0x10] as u8;
        let at = self.cycles.max(self.mail.poll);
        let poll = self.mail.poll + (at - self.mail.poll).div_ceil(3) * 3;
        let base = self.st_base();
        let w = match base {
            0x02 => {
                let n = self.ram[0x12] as u64;
                if (2..=15).contains(&n) { 41 + 10 * n + 6 * n * (n - 1) + 15 * self.st_sort(false) as u64 } else if n >= 16 { 28 } else { 24 }
            }
            0x04 => {
                let (x, y) = (s16(self.ram[0]), s16(self.ram[1]));
                43 + 4 * ((x.unsigned_abs() & 0xFFFF) <= (y.unsigned_abs() & 0xFFFF)) as u64 + 2 * (x < 0) as u64 + 2 * (y < 0) as u64
            }
            c => work(c),
        };
        self.next = Next::Done;
        self.due = poll + w;
    }

    fn st_base(&self) -> u8 {
        match self.command & 15 {
            c @ 9..=15 => c - 8,
            c => c,
        }
    }

    /// 02h's sort: the exchanges a fixed-pass bubble sort makes, and with `apply` the sort made in the RAM.
    fn st_sort(&mut self, apply: bool) -> u32 {
        let n = self.ram[0x12] as usize;
        let mut keys: Vec<u16> = self.ram[0x20..0x20 + n].to_vec();
        let mut drivers: Vec<u16> = self.ram[0x40..0x40 + n].to_vec();
        let mut exchanges = 0;
        for pass in 0..n - 1 {
            for j in 0..n - 1 - pass {
                if keys[j] < keys[j + 1] {
                    keys.swap(j, j + 1);
                    drivers.swap(j, j + 1);
                    exchanges += 1;
                }
            }
        }
        if apply {
            self.ram[0x20..0x20 + n].copy_from_slice(&keys);
            self.ram[0x40..0x40 + n].copy_from_slice(&drivers);
        }
        exchanges
    }

    /// The command's results written and the busy bit cleared; the poll resumes.
    pub(super) fn st_done(&mut self) {
        let base = self.st_base();
        if base == 0x02 {
            let n = self.ram[0x12];
            if (2..=15).contains(&n) {
                self.st_sort(true);
                self.ram[0] = 0;
            } else {
                self.ram[0] = n;
            }
        }
        let r = &mut self.ram;
        let put32 = |r: &mut [u16], at: usize, v: u32| {
            r[at] = v as u16;
            r[at + 1] = (v >> 16) as u16;
        };
        match base {
            0x02 => {}
            0x03 => {
                let (x, y, m) = (s16(r[0]), s16(r[1]), s16(r[2]));
                put32(r, 8, product(x, m));
                put32(r, 10, product(y, m));
            }
            0x04 => {
                let (ax, ay) = (s16(r[0]).unsigned_abs() as u16, s16(r[1]).unsigned_abs() as u16);
                let (mx, mn) = if ax <= ay { (ay, ax) } else { (ax, ay) };
                // The alpha-max-plus-beta-min constants, 2cos(π/8)/(1 + cos(π/8)) raised and 2sin(π/8)/(1 + cos(π/8)) rounded.
                let (c, s) = (std::f64::consts::FRAC_PI_8.cos(), std::f64::consts::FRAC_PI_8.sin());
                let (alpha, beta) = ((32768.0 * 2.0 * c / (1.0 + c)).ceil() as i32, (32768.0 * 2.0 * s / (1.0 + c)).round() as i32);
                let sum = (product(s16(mx), alpha) as i32 as i64) + (product(s16(mn), beta) as i32 as i64);
                (r[0], r[1], r[8]) = (mx, mn, ((sum + 0x8000) >> 16) as u16);
            }
            0x06 => put32(r, 8, product(s16(r[0]), s16(r[1]))),
            0x07 => {
                let i = (r[0] >> 8) as usize;
                let (s, c) = (sine(i), sine(i + 64));
                (r[0], r[1], r[2]) = (i as u16, s as u16, c as u16);
                for n in 0..176usize {
                    // The perspective scale of line n, round(7,885 / (n + 8.8)) (§43.1).
                    let l = ((2 * 78_850 + 10 * n + 88) / (2 * (10 * n + 88))) as i32;
                    r[0x78 + n] = high(l, c) as u16;
                    r[0x128 + n] = high(l, s) as u16;
                    r[0x1D8 + n] = high(l, -s) as u16;
                    r[0x288 + n] = high(l, c) as u16;
                }
            }
            0x08 => {
                let (x, y, i) = (s16(r[0]), s16(r[1]), (r[2] >> 8) as usize);
                let (s, c) = (sine(i), sine(i + 64));
                r[8] = (high(x, c) + high(y, s)) as u16;
                r[9] = (high(y, c) - high(x, s)) as u16;
            }
            _ => {}
        }
        self.ram[0x10] = 0;
        self.mail.poll = self.cycles + 2;
    }
}

#[cfg(test)]
mod tests {
    use super::super::dsphle::Program;
    use super::super::dsporacle::{Hle, Host, mailbox, st_ready};

    // fullsnes's multiplier, K·L·2, through 06h and 03h; 02h's descending sort with its drivers; 00h clears the
    // mailbox alone. No image.
    // VenusRT_DspHle.md §5.2, rule 3: the sine equals an independent generator's, written from the record's formula.
    #[test]
    fn the_st010_sine_equals_the_independent_generator() {
        let script = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../../EmuSen.WiseMan/Reference/analysis/st010_tables.py");
        let out = std::env::temp_dir().join(format!("venusrt-st010-sine-{}.bin", std::process::id()));
        match std::process::Command::new("python3").arg(script).arg(&out).output() {
            Ok(o) if o.status.success() => {
                let theirs = std::fs::read(&out).unwrap();
                let _ = std::fs::remove_file(&out);
                assert_eq!(theirs, super::sine_image());
            }
            _ => eprintln!("python3 or st010_tables.py unavailable, not run"),
        }
    }

    #[test]
    fn the_st010_formulas_answer_through_the_mailbox() {
        let mut h = Hle::new(Program::St010);
        st_ready(&mut h, &mut Host::steady()).unwrap();
        let m = mailbox(&mut h, &mut Host::steady(), 0x06, &[(0, 3), (1, 5)]);
        assert_eq!((m.ram[8], m.ram[9]), (30, 0));
        let m = mailbox(&mut h, &mut Host::steady(), 0x03, &[(0, 0xFFFF), (1, 2), (2, 7)]);
        assert_eq!(&m.ram[8..12], &[0xFFF2, 0xFFFF, 28, 0]);
        let m = mailbox(&mut h, &mut Host::steady(), 0x02, &[(0x12, 3), (0x20, 1), (0x21, 3), (0x22, 2), (0x40, 10), (0x41, 30), (0x42, 20)]);
        assert_eq!((&m.ram[0x20..0x23], &m.ram[0x40..0x43]), (&[3, 2, 1][..], &[30, 20, 10][..]));
        let before = m.ram.clone();
        let m = mailbox(&mut h, &mut Host::steady(), 0x00, &[]);
        assert!(m.latency.is_some() && m.ram == before);
    }
}
