//! The standard controllers on SIO0, one byte at a time after their address (psx-spx "Controllers - Communication
//! Sequence", "Standard Controllers", "Configuration Commands", "Vibration/Rumble Control"): the digital pad (SCPH-1080)
//! and the DualShock (SCPH-1200) with its analog mode, configuration mode, LED and motors. Diamond_Pad.md §4 and §7.

/// What a device sends back for one byte, and whether it pulls /ACK after it, with any delay beyond the usual.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reply {
    pub byte: u8,
    pub ack: Option<u32>,
}

impl Reply {
    /// A byte followed by /ACK at the usual delay.
    pub const fn more(byte: u8) -> Reply {
        Reply { byte, ack: Some(0) }
    }

    /// A transfer's last byte: no /ACK after it.
    pub const fn last(byte: u8) -> Reply {
        Reply { byte, ack: None }
    }
}

/// No device driving the data line (psx-spx "Controller ID": FFFFh, pins floating).
pub const HIGH_Z: Reply = Reply::last(0xFF);

/// The read command, "B", and the DualShock's configuration commands' first, "C".
pub const READ: u8 = 0x42;
pub const CONFIG: u8 = 0x43;

/// The report's L3, R3 and the Analog button, which only the DualShock has; the button is not one of the report's bits.
pub const L3: u16 = 1 << 1;
pub const R3: u16 = 1 << 2;
pub const ANALOG_BUTTON: u32 = 1 << 16;

/// The ID bytes: digital, analog (red LED), and configuration mode.
pub const DIGITAL_ID: u8 = 0x41;
pub const ANALOG_ID: u8 = 0x73;
pub const CONFIG_ID: u8 = 0xF3;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DigitalPad {
    /// The switches as the pad report's bits (psx-spx "Standard Controllers"), 1 for pressed.
    pub pressed: u16,
}

impl DigitalPad {
    /// The reply to the byte at `step` of a transfer, 1 being the command byte after the address, and whether the
    /// transfer goes on; the ID's low byte is sent while the command arrives, so it does not depend on it.
    pub fn exchange(&self, step: u8, byte: u8) -> (Reply, bool) {
        let switches = !self.pressed;
        match step {
            1 if byte == READ => (Reply::more(DIGITAL_ID), true),
            1 => (Reply::last(DIGITAL_ID), false),
            2 => (Reply::more(0x5A), true),
            3 => (Reply::more(switches as u8), true),
            4 => (Reply::last((switches >> 8) as u8), false),
            _ => (HIGH_Z, false),
        }
    }

    /// What a 42h read returns after the address, padded with FFh to a multitap's four halfwords.
    pub fn report(&self) -> [u8; 8] {
        let s = !self.pressed;
        [DIGITAL_ID, 0x5A, s as u8, (s >> 8) as u8, 0xFF, 0xFF, 0xFF, 0xFF]
    }
}

/// The DualShock (SCPH-1200). It powers on in digital mode and answers as the digital pad until the Analog button or
/// command 44h switches it (psx-spx "Analog Mode Note").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DualShock {
    pub pressed: u16,
    /// The Analog button as last given, so that a press is seen once.
    pub analog_held: bool,
    /// The sticks as the report sends them: right X, right Y, left X, left Y; 00h left or up, 80h centred.
    pub sticks: [u8; 4],
    pub analog: bool,
    pub config: bool,
    /// Command 44h's Key 3: the Analog button ignored.
    pub locked: bool,
    /// Whether configuration commands were used, which ends the one-motor method and makes the Analog button
    /// replace 5Ah by 00h ("Caution 3").
    pub configured: bool,
    pub zero_5a: bool,
    /// Command 4Dh's mapping of the read command's bytes 3-8 to the motors: 00h the small one, 01h the large, FFh none.
    pub rumble_map: [u8; 6],
    /// The motors: the small one on or off, the large one's speed.
    pub small: bool,
    pub large: u8,
    /// The transfer under way: its command, the bytes it sends, and the values it collects.
    pub command: u8,
    pub out: [u8; 8],
    pub length: u8,
    pub collect: [u8; 6],
}

impl Default for DualShock {
    fn default() -> DualShock {
        DualShock {
            pressed: 0,
            analog_held: false,
            sticks: [0x80; 4],
            analog: false,
            config: false,
            locked: false,
            configured: false,
            zero_5a: false,
            rumble_map: [0xFF; 6],
            small: false,
            large: 0,
            command: 0,
            out: [0; 8],
            length: 0,
            collect: [0; 6],
        }
    }
}

impl DualShock {
    /// The host's buttons, the report's bits and bit 16 for the Analog button, which switches the mode on a press
    /// unless locked, stopping and locking the motors ("Caution 2").
    pub fn set_buttons(&mut self, mask: u32) {
        self.pressed = mask as u16;
        let held = mask & ANALOG_BUTTON != 0;
        if held && !self.analog_held && !self.locked {
            self.analog = !self.analog;
            self.small = false;
            self.large = 0;
            self.rumble_map = [0xFF; 6];
            self.zero_5a |= self.configured;
        }
        self.analog_held = held;
    }

    pub fn id(&self) -> u8 {
        if self.config {
            CONFIG_ID
        } else if self.analog {
            ANALOG_ID
        } else {
            DIGITAL_ID
        }
    }

    fn switches(&self, analog: bool) -> u16 {
        let s = !self.pressed;
        if analog { s } else { s | L3 | R3 }
    }

    fn buttons(&self, analog: bool) -> [u8; 6] {
        let s = self.switches(analog);
        [s as u8, (s >> 8) as u8, self.sticks[0], self.sticks[1], self.sticks[2], self.sticks[3]]
    }

    /// What a 42h read returns after the address in normal mode, padded with FFh as the multitap sends it.
    pub fn report(&self) -> [u8; 8] {
        let b = self.buttons(self.analog);
        let second = if self.zero_5a { 0 } else { 0x5A };
        if self.analog { [ANALOG_ID, second, b[0], b[1], b[2], b[3], b[4], b[5]] } else { [DIGITAL_ID, second, b[0], b[1], 0xFF, 0xFF, 0xFF, 0xFF] }
    }

    /// The bytes after the ID for `command`, from the state as the command arrives (psx-spx "Configuration Commands").
    fn begin(&mut self, command: u8) -> bool {
        self.command = command;
        self.configured |= self.config;
        let second = if self.zero_5a { 0 } else { 0x5A };
        let z = [0u8; 6];
        let (body, ok) = if !self.config {
            match command {
                READ | CONFIG => (self.buttons(self.analog), true),
                _ => (z, false),
            }
        } else {
            match command {
                READ => (self.buttons(true), true),
                0x45 => ([0x01, 0x02, self.analog as u8, 0x02, 0x01, 0x00], true),
                0x47 => ([0x00, 0x00, 0x02, 0x00, 0x01, 0x00], true),
                0x4D => (self.rumble_map, true),
                0x40..=0x4F => (z, true),
                _ => (z, false),
            }
        };
        let halfwords = if self.config || self.analog { 3 } else { 1 };
        self.out = [second, body[0], body[1], body[2], body[3], body[4], body[5], 0];
        self.length = 2 + 2 * halfwords;
        self.collect = [0; 6];
        ok
    }

    /// A byte the host sent at a parameter position (step 3 on) takes effect as it arrives, so that a later reply of
    /// the same command can depend on it.
    fn parameter(&mut self, index: usize, byte: u8) {
        self.collect[index] = byte;
        match (self.config, self.command, index) {
            (false, CONFIG, 0) | (true, CONFIG, 0) => self.config = byte == 1,
            (true, 0x44, 0) if byte <= 1 => self.analog = byte == 1,
            (true, 0x44, 1) => self.locked = byte & 3 == 3,
            (true, 0x46, 0) => {
                let v = match byte {
                    0 => [0x01, 0x02, 0x00, 0x0A],
                    1 => [0x01, 0x01, 0x01, 0x14],
                    _ => [0; 4],
                };
                self.out[3..7].copy_from_slice(&v);
            }
            (true, 0x4C, 0) => self.out[4] = [0x04, 0x07].get(byte as usize).copied().unwrap_or(0),
            (true, 0x48, 0) => self.out[5] = (byte <= 1) as u8,
            (true, 0x4D, i) => self.rumble_map[i] = byte,
            (false, READ, _) => self.rumble(),
            _ => {}
        }
    }

    /// The motors from a normal-mode read's bytes 3-8: by command 4Dh's mapping once configuration commands were
    /// used, else the one-motor method's xx in 40h-7Fh with yy's bit 0 (psx-spx "Old Method").
    fn rumble(&mut self) {
        let c = self.collect;
        if self.configured || self.rumble_map != [0xFF; 6] {
            for (i, &m) in self.rumble_map.iter().enumerate() {
                match m {
                    0 => self.small = c[i] & 1 != 0,
                    1 => self.large = c[i],
                    _ => {}
                }
            }
        } else {
            self.small = c[0] & 0xC0 == 0x40 && c[1] & 1 != 0;
        }
    }

    pub fn exchange(&mut self, step: u8, byte: u8) -> (Reply, bool) {
        match step {
            1 => {
                let id = self.id();
                if self.begin(byte) { (Reply::more(id), true) } else { (Reply::last(id), false) }
            }
            s if s <= self.length => {
                if s >= 3 {
                    self.parameter(s as usize - 3, byte);
                }
                let v = self.out[s as usize - 2];
                if s == self.length { (Reply::last(v), false) } else { (Reply::more(v), true) }
            }
            _ => (HIGH_Z, false),
        }
    }
}

/// The controllers' ids, as the core's descriptors name them.
pub const PAD: &str = "psx.pad";
pub const DUALSHOCK: &str = "psx.dualshock";

/// A controller of the kind an id names, as it powers on.
pub fn controller(kind: &str) -> Option<Controller> {
    match kind {
        PAD => Some(Controller::Digital(DigitalPad::default())),
        DUALSHOCK => Some(Controller::DualShock(DualShock::default())),
        _ => None,
    }
}

/// What a port's controller connector holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Controller {
    Digital(DigitalPad),
    DualShock(DualShock),
}

impl Controller {
    pub fn exchange(&mut self, step: u8, byte: u8) -> (Reply, bool) {
        match self {
            Controller::Digital(p) => p.exchange(step, byte),
            Controller::DualShock(p) => p.exchange(step, byte),
        }
    }

    pub fn report(&self) -> [u8; 8] {
        match self {
            Controller::Digital(p) => p.report(),
            Controller::DualShock(p) => p.report(),
        }
    }

    /// The host's mask in the descriptor's bits.
    pub fn set_buttons(&mut self, mask: u32) {
        match self {
            Controller::Digital(p) => p.pressed = mask as u16 & !(L3 | R3),
            Controller::DualShock(p) => p.set_buttons(mask),
        }
    }

    pub fn buttons(&self) -> u32 {
        match self {
            Controller::Digital(p) => p.pressed as u32,
            Controller::DualShock(p) => p.pressed as u32 | if p.analog_held { ANALOG_BUTTON } else { 0 },
        }
    }

    /// A stick axis, `axis` as the descriptor numbers them (left X, left Y, right X, right Y), `value` −1 to 1 right
    /// and down positive.
    pub fn set_axis(&mut self, axis: u32, value: f64) {
        if let Controller::DualShock(p) = self {
            let v = (127.5 + 127.5 * value.clamp(-1.0, 1.0)).round() as u8;
            match axis {
                0 => p.sticks[2] = v,
                1 => p.sticks[3] = v,
                2 => p.sticks[0] = v,
                3 => p.sticks[1] = v,
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn transfer(c: &mut Controller, bytes: &[u8]) -> Vec<(u8, bool)> {
        let mut out = Vec::new();
        for (i, &b) in bytes.iter().enumerate() {
            let (r, more) = c.exchange(i as u8 + 1, b);
            out.push((r.byte, r.ack.is_some()));
            if !more {
                break;
            }
        }
        out
    }

    fn bytes(t: &[(u8, bool)]) -> Vec<u8> {
        t.iter().map(|b| b.0).collect()
    }

    #[test]
    fn a_read_answers_the_id_and_the_switches_low_for_pressed() {
        let mut pad = Controller::Digital(DigitalPad { pressed: 1 << 3 | 1 << 14 });
        let t = transfer(&mut pad, &[READ, 0, 0, 0, 0]);
        assert_eq!(bytes(&t), [0x41, 0x5A, 0xF7, 0xBF]);
        assert_eq!(t.iter().map(|b| b.1).collect::<Vec<_>>(), [true, true, true, false], "no /ACK after the last byte");
        assert_eq!(transfer(&mut pad, &[0x43, 0]), [(0x41, false)], "a digital pad answers no other command");
    }

    #[test]
    fn the_dualshock_answers_as_a_digital_pad_until_the_analog_button() {
        let mut c = Controller::DualShock(DualShock::default());
        c.set_buttons(L3 as u32 | 1 << 3);
        assert_eq!(bytes(&transfer(&mut c, &[READ, 0, 0, 0, 0])), [0x41, 0x5A, 0xF7, 0xFF], "L3 reads released in digital mode");
        c.set_buttons(ANALOG_BUTTON);
        c.set_buttons(0);
        c.set_axis(0, -1.0);
        c.set_axis(3, 1.0);
        assert_eq!(bytes(&transfer(&mut c, &[READ, 0, 0, 0, 0, 0, 0, 0, 0])), [0x73, 0x5A, 0xFF, 0xFF, 0x80, 0xFF, 0x00, 0x80]);
    }

    #[test]
    fn configuration_mode_sets_the_led_reads_it_and_maps_the_motors() {
        let mut c = Controller::DualShock(DualShock::default());
        assert_eq!(bytes(&transfer(&mut c, &[CONFIG, 0, 1, 0])), [0x41, 0x5A, 0xFF, 0xFF], "entered from normal mode");
        assert_eq!(bytes(&transfer(&mut c, &[0x44, 0, 1, 3, 0, 0, 0, 0])), [0xF3, 0x5A, 0, 0, 0, 0, 0, 0]);
        assert_eq!(bytes(&transfer(&mut c, &[0x45, 0, 0, 0, 0, 0, 0, 0])), [0xF3, 0x5A, 1, 2, 1, 2, 1, 0], "type 1, the LED on");
        assert_eq!(bytes(&transfer(&mut c, &[0x46, 0, 1, 0, 0, 0, 0, 0])), [0xF3, 0x5A, 0, 0, 1, 1, 1, 0x14]);
        assert_eq!(bytes(&transfer(&mut c, &[0x4D, 0, 0, 1, 0xFF, 0xFF, 0xFF, 0xFF])), [0xF3, 0x5A, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF], "the old mapping");
        assert_eq!(bytes(&transfer(&mut c, &[CONFIG, 0, 0, 0, 0, 0, 0, 0])), [0xF3, 0x5A, 0, 0, 0, 0, 0, 0]);
        let t = transfer(&mut c, &[READ, 0, 1, 0xC0, 0, 0, 0, 0]);
        assert_eq!((t[0].0, t.len(), t[7].1), (0x73, 8, false), "analog, nine bytes with the address, the last without /ACK");
        let Controller::DualShock(p) = c else { unreachable!() };
        assert_eq!((p.small, p.large, p.locked), (true, 0xC0, true));
        c.set_buttons(ANALOG_BUTTON);
        assert_eq!(c.report()[0], 0x73, "locked: the Analog button ignored");
    }

    #[test]
    fn the_one_motor_method_and_the_analog_buttons_reset() {
        let mut p = DualShock::default();
        let mut c = Controller::DualShock(p);
        transfer(&mut c, &[READ, 0, 0x40, 0x01, 0]);
        let Controller::DualShock(q) = c else { unreachable!() };
        assert!(q.small, "xx 40h-7Fh and yy bit 0");
        p = q;
        p.configured = true;
        p.set_buttons(ANALOG_BUTTON);
        assert_eq!((p.analog, p.small, p.report()[1]), (true, false, 0x00), "the motors stopped and 5Ah replaced by 00h");
    }
}
