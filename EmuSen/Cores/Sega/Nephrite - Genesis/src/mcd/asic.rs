//! The graphics ASIC, "rotation and scaling": it traces each line of an image buffer across a stamp map, a dot per
//! step of the line's vector, and writes the stamps' dots it meets into the buffer, cells stacked in columns as the
//! VDP's sprites are. The Mega-CD Hardware Manual §3-8 (`$FF8058`-`$FF8066`, the stamps, the map, the trace vectors,
//! the image buffer and its offset, the operation's time) is the source; Nephrite_Native.md §57 is the record, D-37
//! what the manual leaves open.

/// Sub clocks a line costs before its dots, a dot of the buffer's offset costs within its group of four, and a group
/// of four dots costs (the manual's "graphics operation time": `Vdot × (13 + 2 × (3 and offset) + 9 × ((Hdot + offset
/// − 1) >> 2))`, a clock 80 ns).
pub(in crate::mcd) const LINE_CLOCKS: u64 = 13;
pub(in crate::mcd) const OFFSET_CLOCKS: u64 = 2;
pub(in crate::mcd) const GROUP_CLOCKS: u64 = 9;
/// A refresh of three clocks every 192, and three clocks for each access the sub CPU or the CDC's DMA makes to Word
/// RAM while the ASIC runs.
pub(in crate::mcd) const REFRESH_EVERY: u64 = 192;
pub(in crate::mcd) const STOLEN: u64 = 3;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::mcd) struct Asic {
    /// `$FF8058`'s SMS, STS and RPT (bits 2-0).
    pub(in crate::mcd) size: u8,
    /// `$FF805A`, `$FF805C`, `$FF805E`, `$FF8060`, `$FF8062` as written; `$FF8064` counted down a line at a time.
    pub(in crate::mcd) map: u16,
    pub(in crate::mcd) vcells: u8,
    pub(in crate::mcd) buffer: u16,
    pub(in crate::mcd) offset: u8,
    pub(in crate::mcd) hdots: u16,
    pub(in crate::mcd) vdots: u8,
    /// `$FF8066`, the trace vector table's address in units of 4 bytes, and the line the operation is on.
    pub(in crate::mcd) vector: u16,
    pub(in crate::mcd) line: u32,
    /// GRON: an operation is running; the sub clock its current line ends, and the clocks the refresh owes.
    pub(in crate::mcd) busy: bool,
    pub(in crate::mcd) next: u64,
    pub(in crate::mcd) refresh: u64,
}

impl Default for Asic {
    fn default() -> Asic {
        Asic { size: 0, map: 0, vcells: 0, buffer: 0, offset: 0, hdots: 0, vdots: 0, vector: 0, line: 0, busy: false, next: u64::MAX, refresh: 0 }
    }
}

impl Asic {
    fn screen(&self) -> usize {
        if self.size & 4 != 0 { 4096 } else { 256 }
    }

    fn stamp(&self) -> usize {
        if self.size & 2 != 0 { 32 } else { 16 }
    }

    /// The stamp map's address in Word RAM: `$FF805A`'s bits are A17 down, as many as the map's size leaves.
    fn map_address(&self) -> usize {
        let bytes = (self.screen() / self.stamp()).pow(2) * 2;
        ((self.map as usize) << 2) & 0x3_FFFF & !(bytes - 1)
    }

    /// A register as the sub CPU reads it, by its offset from `$FF8058`.
    pub(in crate::mcd) fn read(&self, off: u32) -> u16 {
        match off {
            0x0 => (self.busy as u16) << 15 | self.size as u16,
            0x2 => self.map,
            0x4 => self.vcells as u16,
            0x6 => self.buffer,
            0x8 => self.offset as u16,
            0xA => self.hdots,
            0xC => self.vdots as u16,
            _ => 0,
        }
    }

    /// A register written; `$FF8066` starts an operation at sub clock `now` when Word RAM is in 2M, the only mode it
    /// runs in. True when it started.
    pub(in crate::mcd) fn write(&mut self, off: u32, v: u16, mask: u16, now: u64, two_m: bool) -> bool {
        let w = |old: u16| (old & !mask) | (v & mask);
        match off {
            0x0 => self.size = w(self.size as u16) as u8 & 7,
            0x2 => self.map = w(self.map) & 0xFFE0,
            0x4 => self.vcells = w(self.vcells as u16) as u8 & 0x1F,
            0x6 => self.buffer = w(self.buffer) & 0xFFF8,
            0x8 => self.offset = w(self.offset as u16) as u8 & 0x3F,
            0xA => self.hdots = w(self.hdots) & 0x1FF,
            0xC => self.vdots = w(self.vdots as u16) as u8,
            0xE => {
                self.vector = w(self.vector) & 0xFFFE;
                if two_m && self.vdots != 0 {
                    (self.busy, self.line, self.refresh) = (true, 0, 0);
                    self.next = now + self.line_clocks();
                    return true;
                }
            }
            _ => {}
        }
        false
    }

    /// One line's clocks, its share of the refresh included.
    fn line_clocks(&mut self) -> u64 {
        let dot = (self.offset & 7) as u64;
        let c = LINE_CLOCKS + OFFSET_CLOCKS * (dot & 3) + GROUP_CLOCKS * ((self.hdots as u64 + dot).saturating_sub(1) >> 2);
        self.refresh += c;
        let owed = self.refresh / REFRESH_EVERY;
        self.refresh %= REFRESH_EVERY;
        c + STOLEN * owed
    }

    /// An access to Word RAM by the sub CPU or the DMA, which delays the operation.
    #[inline]
    pub(in crate::mcd) fn stolen(&mut self) {
        if self.busy {
            self.next += STOLEN;
        }
    }

    /// The line due at its end: drawn into `ram` (Word RAM in 2M order) with `priority`'s rule; true when it was the
    /// last, the operation done and level 1 owed.
    pub(in crate::mcd) fn step(&mut self, ram: &mut [u8], priority: u8) -> bool {
        self.draw(ram, priority);
        self.line += 1;
        self.vdots = self.vdots.wrapping_sub(1);
        if self.vdots == 0 {
            (self.busy, self.next) = (false, u64::MAX);
            return true;
        }
        self.next += self.line_clocks();
        false
    }

    fn word(ram: &[u8], a: usize) -> u16 {
        let a = a & 0x3_FFFE;
        u16::from_be_bytes([ram[a], ram[a + 1]])
    }

    /// The dot of the stamp map at `(x, y)` in its own dots, 0 off the map unless it repeats.
    fn dot(&self, ram: &[u8], x: u32, y: u32) -> u8 {
        let (screen, s) = (self.screen() as u32, self.stamp() as u32);
        let (x, y) = if self.size & 1 != 0 {
            (x & (screen - 1), y & (screen - 1))
        } else if x >= screen || y >= screen {
            return 0;
        } else {
            (x, y)
        };
        let entry = Self::word(ram, self.map_address() + (((y / s) * (screen / s) + x / s) as usize) * 2);
        let number = (entry & if s == 32 { 0x7FC } else { 0x7FF }) as usize;
        if number == 0 {
            return 0;
        }
        // The stamp as shown is its data flipped across, then turned a quarter counter-clockwise RT times: undo both.
        let (mut u, mut v) = (x % s, y % s);
        for _ in 0..(entry >> 13 & 3) {
            (u, v) = (s - 1 - v, u);
        }
        if entry & 0x8000 != 0 {
            u = s - 1 - u;
        }
        let cells = s / 8;
        let a = number * 128 + (((u >> 3) * cells + (v >> 3)) * 32 + (v & 7) * 4 + ((u & 7) >> 1)) as usize;
        let b = ram[a & 0x3_FFFF];
        if u & 1 == 0 { b >> 4 } else { b & 15 }
    }

    /// The current line traced across the map and written into the image buffer.
    fn draw(&self, ram: &mut [u8], priority: u8) {
        let tv = ((self.vector as usize) << 2) + self.line as usize * 8;
        let (xs, ys) = (Self::word(ram, tv) as u32, Self::word(ram, tv + 2) as u32);
        let (dx, dy) = (Self::word(ram, tv + 4) as i16 as i32 as u32, Self::word(ram, tv + 6) as i16 as i32 as u32);
        // Positions are 13.11 in 24 bits: the start's 13.3 shifted up, each step's 4.11 added.
        let (mut x, mut y) = (xs << 8, ys << 8);
        let start = (self.buffer as usize) << 2;
        let column = (self.vcells as usize + 1) * 32;
        let line = (self.offset >> 3) as usize + self.line as usize;
        for j in 0..self.hdots as usize {
            let p = self.dot(ram, x >> 11 & 0x1FFF, y >> 11 & 0x1FFF);
            (x, y) = (x.wrapping_add(dx) & 0xFF_FFFF, y.wrapping_add(dy) & 0xFF_FFFF);
            let d = (self.offset & 7) as usize + j;
            let a = (start + (d >> 3) * column + (line >> 3) * 32 + (line & 7) * 4 + ((d & 7) >> 1)) & 0x3_FFFF;
            let high = d & 1 == 0;
            let old = if high { ram[a] >> 4 } else { ram[a] & 15 };
            let keep = match priority {
                1 => old != 0,
                2 => p == 0,
                _ => false,
            };
            if !keep {
                ram[a] = if high { (ram[a] & 0x0F) | p << 4 } else { (ram[a] & 0xF0) | p };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Word RAM with stamp 1 a 16 × 16 stamp whose dot (u, v) is `(u + v) % 15 + 1`, in the sprite order of cells.
    fn ram_with_stamp() -> Vec<u8> {
        let mut ram = vec![0u8; 0x4_0000];
        for v in 0..16usize {
            for u in 0..16usize {
                let a = 128 + ((u >> 3) * 2 + (v >> 3)) * 32 + (v & 7) * 4 + ((u & 7) >> 1);
                let p = ((u + v) % 15 + 1) as u8;
                ram[a] |= if u & 1 == 0 { p << 4 } else { p };
            }
        }
        ram
    }

    fn put(ram: &mut [u8], a: usize, w: u16) {
        ram[a..a + 2].copy_from_slice(&w.to_be_bytes());
    }

    /// The buffer's dot (x, y) for a buffer at `start` of `vcells` cells a column.
    fn buffer_dot(ram: &[u8], start: usize, vcells: usize, x: usize, y: usize) -> u8 {
        let a = start + (x >> 3) * vcells * 32 + (y >> 3) * 32 + (y & 7) * 4 + ((x & 7) >> 1);
        if x & 1 == 0 { ram[a] >> 4 } else { ram[a] & 15 }
    }

    /// A 1 × 1 map of 16-dot stamps at `$20000` with stamp 1 at its top left in `entry`'s orientation, traced across
    /// 16 lines of 16 dots from (0, 0) a dot at a time into a buffer at `$30000` of two cells a column.
    fn run(entry: u16) -> (Vec<u8>, Asic) {
        let mut ram = ram_with_stamp();
        put(&mut ram, 0x2_0000, entry);
        for line in 0..16u16 {
            let tv = 0x1_8000 + line as usize * 8;
            put(&mut ram, tv, 0);
            put(&mut ram, tv + 2, line << 3);
            put(&mut ram, tv + 4, 0x0800);
            put(&mut ram, tv + 6, 0);
        }
        let mut a = Asic::default();
        a.write(0x0, 0, 0xFFFF, 0, true);
        a.write(0x2, (0x2_0000 >> 2) as u16, 0xFFFF, 0, true);
        a.write(0x4, 1, 0xFFFF, 0, true);
        a.write(0x6, (0x3_0000 >> 2) as u16, 0xFFFF, 0, true);
        a.write(0xA, 16, 0xFFFF, 0, true);
        a.write(0xC, 16, 0xFFFF, 0, true);
        assert!(a.write(0xE, (0x1_8000 >> 2) as u16, 0xFFFF, 0, true));
        assert_eq!(a.read(0) & 0x8000, 0x8000, "GRON while it runs");
        let mut lines = 0;
        while !a.step(&mut ram, 0) {
            lines += 1;
        }
        assert_eq!((lines, a.vdots, a.read(0) & 0x8000), (15, 0, 0), "a line at a time, Vdot counted down to 0");
        (ram, a)
    }

    #[test]
    fn a_stamp_is_copied_dot_for_dot_at_one_to_one() {
        let (ram, _) = run(1);
        for y in 0..16 {
            for x in 0..16 {
                assert_eq!(buffer_dot(&ram, 0x3_0000, 2, x, y), ((x + y) % 15 + 1) as u8, "({x}, {y})");
            }
        }
    }

    #[test]
    fn rotation_and_flip_follow_the_manuals_figures() {
        // A quarter turn counter-clockwise: the stamp's top row (v = 0) is shown down the left column, bottom to top.
        let (ram, _) = run(1 | 1 << 13);
        for u in 0..16 {
            assert_eq!(buffer_dot(&ram, 0x3_0000, 2, 0, 15 - u), (u % 15 + 1) as u8);
        }
        // Flipped across: dot (x, y) shows stamp dot (15 - x, y).
        let (ram, _) = run(1 | 0x8000);
        assert_eq!(buffer_dot(&ram, 0x3_0000, 2, 0, 0), (15 % 15 + 1) as u8);
        assert_eq!(buffer_dot(&ram, 0x3_0000, 2, 15, 3), (3 % 15 + 1) as u8);
        // A half turn: dot (x, y) shows stamp dot (15 - x, 15 - y).
        let (ram, _) = run(1 | 2 << 13);
        assert_eq!(buffer_dot(&ram, 0x3_0000, 2, 2, 5), ((13 + 10) % 15 + 1) as u8);
    }

    #[test]
    fn stamp_zero_and_off_the_map_are_dot_zero() {
        let (ram, _) = run(0);
        assert!((0..16).all(|x| buffer_dot(&ram, 0x3_0000, 2, x, 0) == 0));
        let mut a = Asic { size: 0, ..Asic::default() };
        let ram = ram_with_stamp();
        assert_eq!(a.dot(&ram, 300, 0), 0, "past a 256-dot screen");
        a.size = 1;
        assert_eq!(a.dot(&ram, 256, 0), a.dot(&ram, 0, 0), "repeated");
    }

    #[test]
    fn an_operation_takes_the_manuals_time() {
        let mut a = Asic { hdots: 256, vdots: 2, ..Asic::default() };
        a.write(0xE, 0x100, 0xFFFF, 1000, true);
        // 13 + 9 × 63 = 580 clocks, and three for the refresh's 192.
        assert_eq!(a.next, 1000 + 580 + 9);
        let mut b = Asic { vdots: 2, ..Asic::default() };
        assert!(!b.write(0xE, 0x100, 0xFFFF, 0, false), "not in 1M");
    }
}
