//! The I/O chip (Endou_Native.md §9.5): the I/O control port $3F, and the two joystick ports $DC and $DD with the
//! pads' pins, TH and TR as port $3F sets them, the region they read back, and the Reset button. Active low throughout.

use crate::model::Model;

/// The pad's bits as the descriptor gives them (Endou_Native.md §2.1), pressed as 1.
pub const UP: u8 = 1 << 0;
pub const DOWN: u8 = 1 << 1;
pub const LEFT: u8 = 1 << 2;
pub const RIGHT: u8 = 1 << 3;
pub const BUTTON_1: u8 = 1 << 4;
pub const BUTTON_2: u8 = 1 << 5;
pub const PAUSE: u8 = 1 << 6;
pub const RESET: u8 = 1 << 7;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Io {
    /// Port $3F as last written: TH and TR levels in bits 4-7, their directions (1 input) in bits 0-3. All inputs at
    /// power-on, which software's region checks assume [R SMS Power!'s "Peripheral Ports"].
    pub control: u8,
    /// Each port's pad, pressed as 1; port 0's also carries the console's Pause and Reset buttons.
    pub pads: [u8; 2],
}

impl Default for Io {
    fn default() -> Io {
        Io { control: 0xFF, pads: [0; 2] }
    }
}

impl Io {
    /// A pin's level: an output's level where the model follows port $3F, else the input's.
    fn pin(&self, model: Model, direction: u8, level: u8, input: bool) -> bool {
        if model.io_control() && self.control & direction == 0 { self.control & level != 0 } else { input }
    }

    fn tr(&self, model: Model, port: usize) -> bool {
        let pressed = self.pads[port] & BUTTON_2 != 0;
        if port == 0 { self.pin(model, 0x01, 0x10, !pressed) } else { self.pin(model, 0x04, 0x40, !pressed) }
    }

    /// TH: no pad drives it, so an input reads high. On the Mark III an output reads low whatever its level, as the
    /// Japanese Master System's does, which is what makes software find a Japanese console (Endou_Disputes.md D-3).
    fn th(&self, model: Model, port: usize) -> bool {
        let (direction, level) = if port == 0 { (0x02, 0x20) } else { (0x08, 0x80) };
        let output = self.control & direction == 0;
        match model {
            Model::MarkIII => !output,
            _ => self.pin(model, direction, level, true),
        }
    }

    /// Both ports' TH levels, as the VDP's H counter latch sees them rise (Endou_Native.md §10.4).
    pub fn th_levels(&self, model: Model) -> [bool; 2] {
        [self.th(model, 0), self.th(model, 1)]
    }

    /// Port $DC: port A's directions, TL and TR, then port B's Up and Down [R MacDonald's `smstech` §4; on the SG-1000
    /// the gamepad row of MacDonald's `sc3000h` §5, the same bits].
    pub fn port_dc(&self, model: Model) -> u8 {
        let (a, b) = (self.pads[0], self.pads[1]);
        let mut v = !(a & (UP | DOWN | LEFT | RIGHT | BUTTON_1)) & 0x1F;
        v |= (self.tr(model, 0) as u8) << 5;
        v |= ((b & UP == 0) as u8) << 6;
        v |= ((b & DOWN == 0) as u8) << 7;
        v
    }

    /// Port $DD: port B's Left, Right, TL and TR, the Reset button, the cartridge's CONT (high on the 8-bit consoles),
    /// and the two TH pins [R `smstech` §4]. On the SG-1000 the high four bits are taken as unconnected inputs [A].
    pub fn port_dd(&self, model: Model) -> u8 {
        let b = self.pads[1];
        let mut v = !((b & (LEFT | RIGHT | BUTTON_1)) >> 2) & 0x07;
        v |= (self.tr(model, 1) as u8) << 3;
        if model == Model::Sg1000 {
            return v | 0xF0;
        }
        v |= ((!model.has_reset_button() || self.pads[0] & RESET == 0) as u8) << 4;
        v |= 1 << 5;
        v |= (self.th(model, 0) as u8) << 6;
        v |= (self.th(model, 1) as u8) << 7;
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_pads_read_high_and_presses_low() {
        let mut io = Io::default();
        assert_eq!((io.port_dc(Model::Sms2), io.port_dd(Model::Sms2)), (0xFF, 0xFF));
        io.pads = [UP | BUTTON_2, DOWN | LEFT | BUTTON_1];
        assert_eq!(io.port_dc(Model::Sms2), 0b0101_1110);
        assert_eq!(io.port_dd(Model::Sms2), 0b1111_1010);
        io.pads = [0, RIGHT | BUTTON_2 | UP];
        assert_eq!((io.port_dc(Model::Sms2), io.port_dd(Model::Sms2)), (0b1011_1111, 0b1111_0101));
    }

    /// The region check of SMS Power!'s "Region Detection": TH set as outputs, written high and then low, read back.
    fn japanese(model: Model) -> bool {
        let mut io = Io::default();
        io.control = 0b1111_0101;
        let high = io.port_dd(model) & 0xC0;
        io.control = 0b0101_0101;
        let low = io.port_dd(model) & 0xC0;
        high != 0xC0 || low != 0
    }

    #[test]
    fn the_export_consoles_read_th_back_and_the_mark_iii_does_not() {
        assert!(!japanese(Model::Sms1));
        assert!(!japanese(Model::Sms2));
        assert!(japanese(Model::MarkIII));
        let mut io = Io::default();
        io.control = 0b0000_1110;
        io.pads[0] = BUTTON_2;
        assert_eq!(io.port_dc(Model::Sms2) & 0x20, 0, "TR as an output reads its level");
        io.control = 0b0001_1110;
        assert_eq!(io.port_dc(Model::Sms2) & 0x20, 0x20);
        assert_eq!(io.port_dc(Model::MarkIII) & 0x20, 0, "the Mark III's TR stays the button");
    }

    #[test]
    fn only_the_first_master_system_has_a_reset_button() {
        let mut io = Io::default();
        io.pads[0] = RESET | PAUSE;
        assert_eq!(io.port_dd(Model::Sms1) & 0x10, 0);
        for m in [Model::Sms2, Model::MarkIII] {
            assert_eq!(io.port_dd(m) & 0x10, 0x10, "{m:?}");
        }
        assert_eq!(io.port_dc(Model::Sms1), 0xFF, "Pause is no pad bit");
        assert_eq!(io.port_dd(Model::Sg1000), 0xFF);
    }
}
