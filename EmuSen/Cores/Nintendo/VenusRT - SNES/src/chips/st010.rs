//! The ST010's open replacement (F1 ROC II): its mailbox and commands as characterised and graded in
//! VenusRT_Native.md §43 and §54, over `dsphle`'s ports. The chip's RAM is the battery file, so every command writes exactly the
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
        0x03 => 31,
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

/// 01h's angle in 256ths of a turn for a, b in 0..32: round(256·atan2(a, b)/2π), half up (VenusRT_Native.md §54.1).
fn atan_n(a: usize, b: usize) -> u16 {
    static TABLE: std::sync::OnceLock<Vec<u8>> = std::sync::OnceLock::new();
    let t = TABLE.get_or_init(|| (0..1024).map(|i| ((i / 32) as f64).atan2((i % 32) as f64) / std::f64::consts::TAU * 256.0).map(|n| (n + 0.5).floor() as u8).collect());
    t[a * 32 + b] as u16
}

/// The angle table as words of 256·n, a by rows, for the independent generator's check and firmwarecheck.
pub fn atan_image() -> Vec<u8> {
    (0..1024).flat_map(|i| (atan_n(i / 32, i % 32) << 8).to_le_bytes()).collect()
}

/// 01h's routine on (x, y) into words 0-3 and 8, and its cycles; None when -32768 makes it never complete (§54.1).
fn bearing(r: &mut [u16], x: u16, y: u16) -> Option<u64> {
    let (sx, sy) = (s16(x), s16(y));
    let q: u16 = match (sx < 0, sy < 0) {
        (false, false) => 0,
        (true, false) => 0xC000,
        (true, true) => 0x8000,
        (false, true) => 0x4000,
    };
    if sx == -32768 || sy == -32768 {
        (r[0], r[1], r[2], r[3]) = (sx.unsigned_abs() as u16, sy.unsigned_abs() as u16, q, y);
        return None;
    }
    let zero = sx == 0 && sy < 0;
    let (mut a, mut b, mut cycles) = match q {
        _ if zero => (0, -sy, 64),
        0 => (sx, sy, 71),
        0xC000 => (sy, -sx, 78),
        0x8000 => (-sx, -sy, 75),
        _ => (-sy, sx, 78),
    };
    while a >= 32 || b >= 32 {
        let halve = |v: i32| if v != 0 && v >> 1 == 0 { 1 } else { v >> 1 };
        let (low, lower) = (a.min(b), (a >> 1).min(b >> 1));
        cycles += if low == 0 { 8 } else if lower == 0 { 10 } else { 7 };
        (a, b) = (halve(a), halve(b));
    }
    let theta = if zero { 0 } else { q.wrapping_add(0x8000).wrapping_add(atan_n(a as usize, b as usize) << 8) };
    (r[0], r[1], r[2], r[3], r[8]) = (a as u16, b as u16, q, y, theta);
    Some(cycles)
}

/// 05h, one driver's step on the words from 60h, and its cycles; None when it never completes (§54.1).
fn drive(r: &mut [u16]) -> Option<u64> {
    let (dx, dy) = (r[0x60].wrapping_sub(r[0x63]), r[0x61].wrapping_sub(r[0x65]));
    (r[0x68], r[0x69]) = (dx, dy);
    let mut cycles = bearing(r, dx, dy)?;
    let h = r[0x66];
    let e = r[8].wrapping_sub(h);
    let turn = (e as i16) >> 8;
    r[0x66] = h.wrapping_add(0x280u16.wrapping_mul(turn.signum() as u16));
    r[0x67] = e;
    cycles += [9, 0, 6][(turn.signum() + 1) as usize];
    let (v, acc, want) = (r[0x6A] as u32, r[0x6B] as u32, r[0x6C] as u32);
    let size = (e as i16).unsigned_abs() as u32;
    let (v, c) = if size >= 0x1000 {
        let fall = (((size as u16 as i16) >> 4) as u16) as u32;
        if v >= fall { (v - fall, 123) } else { (0, 124) }
    } else if v + acc > 0xFFFF {
        (want, 130)
    } else if v + acc >= want {
        (want, 129)
    } else {
        (v + acc, 128)
    };
    r[0x6A] = v as u16;
    cycles += c;
    let k = (r[0x66] >> 8) as usize;
    let step = |s: i32| (2 * (v >> 8) as i32 * (s >> 5)) as u32;
    let (mx, my) = (step(sine(k)), step(sine(k + 64)));
    (r[0], r[1], r[2], r[3]) = (mx as u16, (mx >> 16) as u16, my as u16, (my >> 16) as u16);
    let px = ((r[0x63] as u32) << 16 | r[0x62] as u32).wrapping_sub(mx) & 0x1FFF_FFFF;
    let py = ((r[0x65] as u32) << 16 | r[0x64] as u32).wrapping_sub(my) & 0x1FFF_FFFF;
    (r[0x62], r[0x63], r[0x64], r[0x65]) = (px as u16, (px >> 16) as u16, py as u16, (py >> 16) as u16);
    let (ax, ay) = ((dx as i16).unsigned_abs(), (dy as i16).unsigned_abs());
    let upright = r[0x6D] & 0x8000 != 0;
    let (wx, wy) = if upright { (7, 127) } else { (127, 7) };
    cycles += upright as u64 + 2 * (dx & 0x8000 != 0) as u64;
    if ax <= wx {
        cycles += 5 + 2 * (dy & 0x8000 != 0) as u64;
        if ay <= wy {
            let next = r[0x70];
            cycles += 20 + (next >> 15) as u64;
            (r[0x60], r[0x61], r[0x6D]) = (r[0x6F], next & 0x0FFF, if next & 0x8000 != 0 { 0xFFFF } else { 0 });
            r[0x6E] |= 8;
        }
    }
    Some(cycles)
}

/// 07h's scale of raster line n: K/(m·n + d) rounded half up, the one sequence the exact search of VenusRT_Native.md §57 found.
fn perspective(n: usize) -> i32 {
    const K: i32 = 39421;
    let q = 5 * n as i32 + 44;
    (2 * K + q) / (2 * q)
}

/// 07h's 176 line scales as words, for the independent generator's check and firmwarecheck.
pub fn perspective_image() -> Vec<u8> {
    (0..176).flat_map(|n| (perspective(n) as u16).to_le_bytes()).collect()
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
            0x01 | 0x05 => {
                let mut copy = self.ram[..0x80].to_vec();
                let (x, y) = (copy[0], copy[1]);
                let spent = if base == 0x01 { bearing(&mut copy, x, y) } else { drive(&mut copy) };
                match spent {
                    Some(cycles) => cycles,
                    None => {
                        // The chip never clears its busy bit; the words it wrote are kept (VenusRT_Native.md §54.1).
                        self.ram[..0x80].copy_from_slice(&copy);
                        self.next = Next::Done;
                        self.due = u64::MAX;
                        return;
                    }
                }
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
            0x01 => {
                let (x, y) = (r[0], r[1]);
                bearing(r, x, y);
            }
            0x05 => {
                drive(r);
            }
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
                    let l = perspective(n);
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
    fn the_st010_perspective_equals_the_independent_generator() {
        let script = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../../EmuSen.WiseMan/Reference/analysis/st010_tables.py");
        let out = std::env::temp_dir().join(format!("venusrt-st010-perspective-{}.bin", std::process::id()));
        match std::process::Command::new("python3").arg(script).arg("--perspective").arg(&out).output() {
            Ok(o) if o.status.success() => {
                let theirs = std::fs::read(&out).unwrap();
                let _ = std::fs::remove_file(&out);
                assert_eq!(theirs, super::perspective_image());
            }
            _ => eprintln!("python3 or st010_tables.py unavailable, not run"),
        }
    }

    #[test]
    fn the_st010_angle_equals_the_independent_generator() {
        let script = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../../EmuSen.WiseMan/Reference/analysis/st010_tables.py");
        let out = std::env::temp_dir().join(format!("venusrt-st010-atan-{}.bin", std::process::id()));
        match std::process::Command::new("python3").arg(script).arg("--atan").arg(&out).output() {
            Ok(o) if o.status.success() => {
                let theirs = std::fs::read(&out).unwrap();
                let _ = std::fs::remove_file(&out);
                assert_eq!(theirs, super::atan_image());
            }
            _ => eprintln!("python3 or st010_tables.py unavailable, not run"),
        }
    }

    // VenusRT_Native.md §54.1's formulas with no image: 01h's angle of (-X, -Y) by quadrant, and a driver whose heading
    // already points at its waypoint moving along it at its wanted speed, then passing a gate and taking the next.
    #[test]
    fn the_st010_angle_and_driver_follow_their_formulas() {
        let mut h = Hle::new(Program::St010);
        st_ready(&mut h, &mut Host::steady()).unwrap();
        for (x, y, theta) in [(1u16, 0u16, 0xC000u16), (0, 1, 0x8000), (0xFFFF, 0, 0x4000), (0, 0xFFFF, 0), (5, 5, 0xA000), (0xFFFB, 0xFFFB, 0x2000)] {
            let m = mailbox(&mut h, &mut Host::steady(), 0x01, &[(0, x), (1, y)]);
            assert_eq!(m.ram[8], theta, "({x:04X}, {y:04X})");
        }
        let set = [(0x60, 0x0800), (0x61, 0x0400), (0x62, 0), (0x63, 0x0800), (0x64, 0), (0x65, 0x0800), (0x66, 0), (0x6A, 0x0300), (0x6B, 0x0100), (0x6C, 0x0300), (0x6D, 0), (0x6E, 0), (0x6F, 0x0123), (0x70, 0x8456)];
        let m = mailbox(&mut h, &mut Host::steady(), 0x05, &set);
        // Due ahead (bearing 0, no turn), speed kept, a step of 2·3·⌊32,767/32⌋ toward -Y; the gate not reached.
        assert_eq!((m.ram[8], m.ram[0x66], m.ram[0x6A], m.ram[0x68], m.ram[0x69]), (0, 0, 0x0300, 0, 0xFC00));
        assert_eq!((m.ram[0x63], m.ram[0x64], m.ram[0x65]), (0x0800, (0x0800_0000u32 - 6 * 1023) as u16, 0x07FF));
        let m = mailbox(&mut h, &mut Host::steady(), 0x05, &[(0x61, 0x07FC), (0x65, 0x0800)]);
        assert_eq!((m.ram[0x60], m.ram[0x61], m.ram[0x6D], m.ram[0x6E]), (0x0123, 0x0456, 0xFFFF, 8));
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
