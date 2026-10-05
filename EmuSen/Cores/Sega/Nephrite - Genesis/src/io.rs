//! The I/O chip at `$A10000`: the version register, the three ports' data, control and serial registers, and the
//! pads on ports A and B with TH's multiplexing and the six-button pad's count. Charles MacDonald's "Sega Genesis
//! hardware notes" §3 is the source; Nephrite_Native.md §9 records the readings.

/// Master clocks after TH's last fall before a six-button pad forgets its count: MacDonald's "about 8192 68000
/// cycles", taken as given.
pub const SIX_BUTTON_TIMEOUT: u64 = 8192 * 7;

/// A pad's buttons as the ABI's mask holds them (`v1.rs`'s table), pressed as 1.
#[derive(Clone, Copy, Debug, Default)]
pub struct Pad {
    pub buttons: u32,
    pub six: bool,
    /// TH as the pad sees it, and the falls since the count was last reset.
    th: bool,
    falls: u8,
    last_fall: u64,
}

impl Pad {
    fn set_th(&mut self, th: bool, clock: u64) {
        if self.th && !th {
            if clock.saturating_sub(self.last_fall) > SIX_BUTTON_TIMEOUT {
                self.falls = 0;
            }
            self.falls = self.falls.saturating_add(1);
            self.last_fall = clock;
        }
        self.th = th;
    }

    /// The six lines below TH, active low: TH high gives C B Right Left Down Up; TH low gives Start A 0 0 Down Up. A
    /// six-button pad's second fall forces the directions low, its next rise gives C B Mode X Y Z, and its third fall
    /// forces them high.
    fn lines(&self, clock: u64) -> u8 {
        let b = self.buttons;
        let bit = |n: u32| (b >> n & 1) as u8;
        let falls = if clock.saturating_sub(self.last_fall) > SIX_BUTTON_TIMEOUT { 0 } else { self.falls };
        let pressed = if self.th {
            if self.six && falls == 2 {
                bit(5) << 5 | bit(4) << 4 | bit(11) << 3 | bit(10) << 2 | bit(9) << 1 | bit(8)
            } else {
                (b & 0x3F) as u8
            }
        } else {
            let low = match (self.six, falls) {
                (true, 2) => 0x0F,
                (true, 3) => 0x00,
                _ => (b & 3) as u8 | 0x0C,
            };
            bit(7) << 5 | bit(6) << 4 | low
        };
        !pressed & 0x3F
    }
}

/// The ports' registers and the pads' TH and count, for the state; the buttons are input, set each frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IoRegs {
    pub data: [u8; 3],
    pub ctrl: [u8; 3],
    pub tx: [u8; 3],
    pub sctrl: [u8; 3],
    pub th: [bool; 2],
    pub falls: [u8; 2],
    pub last_fall: [u64; 2],
}

pub struct Io {
    pub version: u8,
    data: [u8; 3],
    ctrl: [u8; 3],
    tx: [u8; 3],
    sctrl: [u8; 3],
    pub pads: [Pad; 2],
    /// TH as a device drives it, for the external interrupt.
    th_in: [bool; 3],
}

impl Io {
    pub fn regs_state(&self) -> IoRegs {
        IoRegs {
            data: self.data,
            ctrl: self.ctrl,
            tx: self.tx,
            sctrl: self.sctrl,
            th: self.pads.map(|p| p.th),
            falls: self.pads.map(|p| p.falls),
            last_fall: self.pads.map(|p| p.last_fall),
        }
    }

    pub fn set_regs_state(&mut self, s: IoRegs) {
        (self.data, self.ctrl, self.tx, self.sctrl) = (s.data, s.ctrl, s.tx, s.sctrl);
        for (i, p) in self.pads.iter_mut().enumerate() {
            (p.th, p.falls, p.last_fall) = (s.th[i], s.falls[i], s.last_fall[i]);
        }
    }

    /// The version register: overseas, PAL, no expansion unit, and the model's version (TMSS from 1).
    pub fn new(overseas: bool, pal: bool, version: u8) -> Io {
        let v = (overseas as u8) << 7 | (pal as u8) << 6 | 1 << 5 | (version & 0xF);
        let pads = [Pad { th: true, ..Pad::default() }; 2];
        Io { version: v, data: [0x7F; 3], ctrl: [0; 3], tx: [0xFF; 3], sctrl: [0; 3], pads, th_in: [true; 3] }
    }

    /// TH as driven: the data register's bit 6 when the control register makes it an output, pulled high otherwise.
    fn drive(&mut self, port: usize, clock: u64) {
        let th = self.ctrl[port] & 0x40 == 0 || self.data[port] & 0x40 != 0;
        if let Some(p) = self.pads.get_mut(port) {
            p.set_th(th, clock);
        }
    }

    /// A device on `port` driving TH (a light gun; a pad drives none): whether the change interrupts, TH being an
    /// input there with the port's interrupt bit, control bit 7, set (MacDonald's VDP document §4).
    pub fn device_th(&mut self, port: usize, th: bool) -> bool {
        let input = self.ctrl[port] & 0x40 == 0;
        let changed = std::mem::replace(&mut self.th_in[port], th) != th;
        changed && input && self.ctrl[port] & 0x80 != 0
    }

    /// A byte of `$A10000`-`$A1001F`, the even and odd address of each register alike.
    pub fn read(&self, a: u32, clock: u64) -> u8 {
        let reg = (a >> 1) & 0xF;
        match reg {
            0 => self.version,
            1..=3 => {
                let p = reg as usize - 1;
                let outputs = self.ctrl[p] & 0x7F;
                let th = self.ctrl[p] & 0x40 == 0 || self.data[p] & 0x40 != 0;
                let inputs = match self.pads.get(p) {
                    Some(pad) => pad.lines(clock) | (th as u8) << 6,
                    None => 0x7F,
                };
                (self.data[p] & (outputs | 0x80)) | (inputs & !outputs & 0x7F)
            }
            4..=6 => self.ctrl[reg as usize - 4],
            7 | 10 | 13 => self.tx[(reg as usize - 7) / 3],
            8 | 11 | 14 => 0,
            9 | 12 | 15 => self.sctrl[(reg as usize - 9) / 3],
            _ => 0,
        }
    }

    pub fn write(&mut self, a: u32, v: u8, clock: u64) {
        let reg = (a >> 1) & 0xF;
        match reg {
            1..=3 => {
                self.data[reg as usize - 1] = v;
                self.drive(reg as usize - 1, clock);
            }
            4..=6 => {
                self.ctrl[reg as usize - 4] = v;
                self.drive(reg as usize - 4, clock);
            }
            7 | 10 | 13 => self.tx[(reg as usize - 7) / 3] = v,
            9 | 12 | 15 => self.sctrl[(reg as usize - 9) / 3] = v & 0xF0,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const UP: u32 = 1;
    const RIGHT: u32 = 8;
    const B: u32 = 1 << 4;
    const A: u32 = 1 << 6;
    const START: u32 = 1 << 7;
    const Z: u32 = 1 << 8;
    const MODE: u32 = 1 << 11;

    #[test]
    fn the_registers_read_as_macdonald_lists_them_at_power_on() {
        let io = Io::new(true, false, 0);
        let got: Vec<u8> = (0..16).map(|r| io.read(0xA1_0001 + 2 * r, 0)).collect();
        assert_eq!(got, [0xA0, 0x7F, 0x7F, 0x7F, 0, 0, 0, 0xFF, 0, 0, 0xFF, 0, 0, 0xFF, 0, 0]);
    }

    #[test]
    fn a_three_button_pad_multiplexes_on_th() {
        let mut io = Io::new(true, false, 0);
        io.pads[0].buttons = UP | RIGHT | B | A | START;
        io.write(0xA1_0009, 0x40, 0);
        io.write(0xA1_0003, 0x40, 0);
        assert_eq!(io.read(0xA1_0003, 0), 0b0110_0110, "TH high: C B R L D U with B, Right, Up low");
        io.write(0xA1_0003, 0x00, 0);
        assert_eq!(io.read(0xA1_0003, 0), 0b0000_0010, "TH low: Start and A low, then 0 0 D U with Up low");
        io.write(0xA1_0003, 0x80, 0);
        assert_eq!(io.read(0xA1_0003, 0) & 0x80, 0x80, "bit 7 latches what was written");
    }

    #[test]
    fn a_six_button_pad_answers_its_extra_buttons_and_times_out() {
        let mut io = Io::new(true, false, 0);
        io.pads[0] = Pad { buttons: Z | MODE, six: true, ..io.pads[0] };
        io.write(0xA1_0009, 0x40, 0);
        let mut seq = Vec::new();
        for (i, th) in [0x40, 0x00, 0x40, 0x00, 0x40, 0x00, 0x40].into_iter().enumerate() {
            io.write(0xA1_0003, th, 100 * i as u64);
            seq.push(io.read(0xA1_0003, 100 * i as u64) & 0x7F);
        }
        assert_eq!(seq, [0x7F, 0x33, 0x7F, 0x30, 0x76, 0x3F, 0x7F]);
        io.write(0xA1_0003, 0x00, 700 + SIX_BUTTON_TIMEOUT + 1);
        assert_eq!(io.read(0xA1_0003, 700 + SIX_BUTTON_TIMEOUT + 1) & 0x0F, 0x03, "a new count after the timeout");
    }
}
