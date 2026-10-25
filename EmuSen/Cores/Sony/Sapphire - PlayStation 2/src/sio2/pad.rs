//! The DualShock 2 as a state machine on its documented protocol (Sapphire_Native.md §30.4): digital, analogue and
//! pressure modes, configuration mode and its commands, the reply bytes chosen by command 4Fh, and both motors (psx-spx
//! "Controllers - Configuration Commands" to "Controllers - Analog Buttons (Dualshock2)"; ps2tek "SIO2 Controller
//! Commands", "SIO2 Standard Controllers"). A command is answered whole: each reply byte depends only on the bytes before
//! it, so answering the command at once is the same as answering it byte by byte.

/// The report's bits (psx-spx "Standard Controllers"), as the frontend's buttons arrive (`v1::BUTTONS`): 1 = pressed.
pub mod bit {
    pub const SELECT: u16 = 1 << 0;
    pub const L3: u16 = 1 << 1;
    pub const R3: u16 = 1 << 2;
    pub const START: u16 = 1 << 3;
    pub const UP: u16 = 1 << 4;
    pub const RIGHT: u16 = 1 << 5;
    pub const DOWN: u16 = 1 << 6;
    pub const LEFT: u16 = 1 << 7;
    pub const L2: u16 = 1 << 8;
    pub const R2: u16 = 1 << 9;
    pub const L1: u16 = 1 << 10;
    pub const R1: u16 = 1 << 11;
    pub const TRIANGLE: u16 = 1 << 12;
    pub const CIRCLE: u16 = 1 << 13;
    pub const CROSS: u16 = 1 << 14;
    pub const SQUARE: u16 = 1 << 15;
    /// The pressure bytes' order, reply bytes 10 to 21 (psx-spx's 4Fh table; ps2tek's "Input Format").
    pub const PRESSURE: [u16; 12] = [RIGHT, LEFT, UP, DOWN, TRIANGLE, CIRCLE, CROSS, SQUARE, L1, R1, L2, R2];
}

/// What the player holds: the buttons pressed, the sticks (00h left or up, 80h centred, FFh right or down) in the
/// reply's order, right X, right Y, left X, left Y (psx-spx; Sapphire_Disputes.md D-227), and each pressure button's
/// force, 00h released to FFh.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Input {
    pub buttons: u16,
    pub sticks: [u8; 4],
    pub pressure: [u8; 12],
}

impl Default for Input {
    fn default() -> Input {
        Input { buttons: 0, sticks: [0x80; 4], pressure: [0; 12] }
    }
}

impl Input {
    /// From the frontend's digital buttons and its sticks: the plan's rule for pressure until the ABI carries analogue
    /// buttons, FFh for a pressed button and 00h for a released one (Sapphire_Plan.md §4.2).
    pub fn from_digital(buttons: u16, sticks: [u8; 4]) -> Input {
        let mut pressure = [0u8; 12];
        for (p, &b) in pressure.iter_mut().zip(bit::PRESSURE.iter()) {
            *p = if buttons & b != 0 { 0xFF } else { 0 };
        }
        Input { buttons, sticks, pressure }
    }
}

/// A stick's byte from the frontend's axis, -1.0 (left or up) to 1.0 (right or down; `PadAxis` is down-positive): 00h to
/// FFh with 0.0 at 80h (psx-spx "Standard Controllers").
pub fn stick(value: f64) -> u8 {
    ((value.clamp(-1.0, 1.0) + 1.0) * 127.5).round() as u8
}

/// The reply bytes command 4Fh can enable, 18 of them (psx-spx), and the masks the modes reset it to.
pub const ALL: u32 = 0x3_FFFF;
pub const DIGITAL_MASK: u32 = 0x03;
pub const ANALOG_MASK: u32 = 0x3F;
/// The mode byte in configuration mode.
pub const CONFIG: u8 = 0xF3;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DualShock2 {
    pub input: Input,
    pub analog: bool,
    /// The Analog button ignored (44h's key 3).
    pub locked: bool,
    pub config: bool,
    /// Which reply bytes 42h sends in analogue mode (4Fh).
    pub mask: u32,
    /// What the six bytes after 42h's header drive (4Dh): 00h the small motor's bit 0, 01h the large motor, FFh nothing.
    pub rumble: [u8; 6],
    /// 40h's twelve values, 02h each at power-on.
    pub attributes: [u8; 12],
    /// The motors: the small one on or off (00h, FFh), the large one's speed.
    pub small: u8,
    pub large: u8,
    /// Configuration mode has been entered: the one-motor method of the first analogue pads stops working.
    pub configured: bool,
}

impl Default for DualShock2 {
    fn default() -> DualShock2 {
        DualShock2 {
            input: Input::default(),
            analog: false,
            locked: false,
            config: false,
            mask: DIGITAL_MASK,
            rumble: [0xFF; 6],
            attributes: [2; 12],
            small: 0,
            large: 0,
            configured: false,
        }
    }
}

impl DualShock2 {
    /// The Analog button pressed: the mode toggled unless locked, the reply bytes reset and the motors stopped (psx-spx
    /// "Analog Mode Note", Caution 2).
    pub fn press_analog(&mut self) {
        if self.locked {
            return;
        }
        self.analog = !self.analog;
        self.mask = if self.analog { ANALOG_MASK } else { DIGITAL_MASK };
        self.small = 0;
        self.large = 0;
    }

    /// The bytes after the header that `mask` selects, in order, padded to whole halfwords.
    fn data(&self, mask: u32, out: &mut [u8; 18]) -> usize {
        let b = !self.input.buttons;
        let mut n = 0;
        for i in 0..18 {
            if mask >> i & 1 == 0 {
                continue;
            }
            out[n] = match i {
                0 => b as u8,
                1 => (b >> 8) as u8,
                2..=5 => self.input.sticks[i - 2],
                _ => self.input.pressure[i - 6],
            };
            n += 1;
        }
        if n % 2 == 1 {
            out[n] = 0;
            n += 1;
        }
        n
    }

    /// The mode byte outside configuration mode: the mode in the high nibble, the halfwords that follow in the low.
    pub fn mode(&self) -> u8 {
        if !self.analog {
            return 0x41;
        }
        let mut d = [0u8; 18];
        0x70 | (self.data(self.mask, &mut d) / 2) as u8
    }

    /// The motors driven by 42h's bytes 3 to 8: through 4Dh's map, or, before configuration mode was ever entered, the
    /// one-motor method (psx-spx "Old Method").
    fn drive(&mut self, cmd: &[u8]) {
        let byte = |i: usize| cmd.get(3 + i).copied().unwrap_or(0);
        if !self.configured && self.rumble == [0xFF; 6] {
            self.small = if byte(0) & 0xC0 == 0x40 && byte(1) & 1 == 1 { 0xFF } else { 0 };
            return;
        }
        for (i, &m) in self.rumble.iter().enumerate() {
            match m {
                0x00 => self.small = if byte(i) & 1 != 0 { 0xFF } else { 0 },
                0x01 => self.large = byte(i),
                _ => {}
            }
        }
    }

    /// Answers a whole command (`cmd[0]` the peripheral byte 01h) into `reply`, as long as `cmd`; bytes past what the pad
    /// sends read 00h [C].
    pub fn exchange(&mut self, cmd: &[u8], reply: &mut [u8]) {
        reply.fill(0);
        let c = cmd.get(1).copied().unwrap_or(0);
        let arg = |i: usize| cmd.get(i).copied().unwrap_or(0);
        let mut body = [0u8; 18];
        let (mode, n) = if self.config {
            let mut six = [0u8; 6];
            match c {
                0x40 => {
                    let i = arg(3) as usize;
                    six[2] = self.attributes.get(i).copied().unwrap_or(0xFF);
                    if i < 12 {
                        self.attributes[i] = arg(4) & 3;
                    }
                }
                // The reply bytes 4Fh can enable, a constant (psx-spx), in analogue mode only, its last byte 5Ah
                // (ps2tek; Sapphire_Disputes.md D-225).
                0x41 => {
                    if self.analog {
                        six = [ALL as u8, (ALL >> 8) as u8, (ALL >> 16) as u8, 0, 0, 0x5A];
                    }
                }
                0x42 => {
                    self.drive(cmd);
                    let mut d = [0u8; 18];
                    self.data(ANALOG_MASK, &mut d);
                    six.copy_from_slice(&d[..6]);
                }
                0x43 => {
                    if arg(3) == 0 {
                        self.config = false;
                    }
                }
                0x44 => match arg(3) {
                    l @ (0 | 1) => {
                        self.analog = l == 1;
                        self.mask = if self.analog { ANALOG_MASK } else { DIGITAL_MASK };
                        self.locked = arg(4) & 3 == 3;
                    }
                    _ => six[2] = 0xFF,
                },
                0x45 => six = [0x03, 0x02, u8::from(self.analog), 0x02, 0x01, 0x00],
                0x46 => match arg(3) {
                    0 => six = [0, 0, 0x01, 0x02, 0x00, 0x0A],
                    1 => six = [0, 0, 0x01, 0x01, 0x01, 0x14],
                    _ => {}
                },
                0x47 => six = [0, 0, 0x02, 0x00, 0x01, 0x00],
                0x48 => {
                    if arg(3) < 2 {
                        six[4] = 0x01;
                    }
                }
                0x4C => match arg(3) {
                    0 => six[3] = 0x04,
                    1 => six[3] = 0x07,
                    _ => {}
                },
                0x4D => {
                    six = self.rumble;
                    for i in 0..6 {
                        self.rumble[i] = arg(3 + i);
                    }
                }
                0x4F => {
                    let v = (0..6).fold(0u64, |v, i| v | u64::from(arg(3 + i)) << (8 * i));
                    if v >> 18 & 0x3F_FFFF == 0 {
                        self.mask = (v as u32) & ALL;
                    }
                }
                _ => {}
            }
            body[..6].copy_from_slice(&six);
            (CONFIG, 6)
        } else {
            match c {
                0x42 => {
                    self.drive(cmd);
                    let mask = if self.analog { self.mask } else { DIGITAL_MASK };
                    let n = self.data(mask, &mut body);
                    (self.mode(), n)
                }
                0x43 => {
                    let mode = self.mode();
                    let mask = if self.analog { self.mask & ANALOG_MASK } else { DIGITAL_MASK };
                    let n = self.data(mask, &mut body);
                    if arg(3) == 1 {
                        self.config = true;
                        self.configured = true;
                    }
                    (mode, n)
                }
                _ => {
                    reply.fill(0xFF);
                    return;
                }
            }
        };
        let header = [0xFF, mode, 0x5A];
        for (i, r) in reply.iter_mut().enumerate() {
            *r = if i < 3 { header[i] } else if i - 3 < n { body[i - 3] } else { 0 };
        }
    }
}

impl DualShock2 {
    /// The pad's block of `SPH2` (Sapphire_Native.md §38.2): its modes, masks, rumble map and motors. The buttons and
    /// sticks are the frontend's, held as they stand across a load.
    pub fn write_state(&self, w: &mut emusen_native::StateWriter) {
        w.bools("Modes", &[self.analog, self.locked, self.config, self.configured]);
        w.u32("Mask", self.mask);
        w.bytes("Rumble", &self.rumble);
        w.bytes("Attributes", &self.attributes);
        w.bytes("Motors", &[self.small, self.large]);
    }

    pub fn read_state(&mut self, r: &mut emusen_native::StateReader) -> Result<(), emusen_native::Truncated> {
        let mut m = [false; 4];
        r.bools(&mut m)?;
        [self.analog, self.locked, self.config, self.configured] = m;
        self.mask = r.u32()?;
        r.bytes(&mut self.rumble)?;
        r.bytes(&mut self.attributes)?;
        let mut motors = [0u8; 2];
        r.bytes(&mut motors)?;
        [self.small, self.large] = motors;
        Ok(())
    }
}
