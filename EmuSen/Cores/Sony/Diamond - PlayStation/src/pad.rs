//! The digital pad (SCPH-1080) on SIO0, one byte at a time after its address (psx-spx "Controllers - Communication
//! Sequence", "Standard Controllers"): its ID 41h, then 5Ah, then the two switch bytes, 0 for a button pressed. Its
//! design, and what stage 6 adds beside it, is Diamond_Pad.md.

/// What a device sends back for one byte, and whether it pulls /ACK after it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reply {
    pub byte: u8,
    pub ack: bool,
}

/// No device driving the data line (psx-spx "Controller ID": FFFFh, pins floating).
pub const HIGH_Z: Reply = Reply { byte: 0xFF, ack: false };

/// The controller's ID halfword, low byte first: a digital pad, one halfword of switches to follow.
pub const ID: [u8; 2] = [0x41, 0x5A];
/// The read command, "B".
pub const READ: u8 = 0x42;

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
            1 => (Reply { byte: ID[0], ack: byte == READ }, byte == READ),
            2 => (Reply { byte: ID[1], ack: true }, true),
            3 => (Reply { byte: switches as u8, ack: true }, true),
            4 => (Reply { byte: (switches >> 8) as u8, ack: false }, false),
            _ => (HIGH_Z, false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_read_answers_the_id_and_the_switches_low_for_pressed() {
        let pad = DigitalPad { pressed: 1 << 3 | 1 << 14 };
        let replies: Vec<(Reply, bool)> = [READ, 0, 0, 0, 0].iter().enumerate().map(|(i, &b)| pad.exchange(i as u8 + 1, b)).collect();
        assert_eq!(replies.iter().map(|r| r.0.byte).collect::<Vec<_>>(), [0x41, 0x5A, 0xF7, 0xBF, 0xFF]);
        assert_eq!(replies.iter().map(|r| r.0.ack).collect::<Vec<_>>(), [true, true, true, false, false], "no /ACK after the last byte");
        assert_eq!(pad.exchange(1, 0x43), (Reply { byte: 0x41, ack: false }, false), "a digital pad answers no other command");
    }
}
