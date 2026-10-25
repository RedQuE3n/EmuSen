//! The multitap's frame (Sapphire_Native.md §30.6; Sapphire_Disputes.md D-228): the peripheral byte 21h (ps2tek's
//! "Known peripheral bytes"), answered on both the controller port and the card port of the slot it plugs into, since it
//! plugs into both (psx-spx "PS2 multitap"). No public document gives its commands; what it answers is the one
//! witness's, PCSX2's, measured 2026-10-25: 12h and 13h report four slots, 21h and 22h acknowledge. Which slot 21h and
//! 22h select for the pads and the cards behind it is a choice [C]: the byte after the command's 00h, 0 to 3, any other
//! value selecting none.

pub const PERIPHERAL: u8 = 0x21;
pub const SLOTS: usize = 4;

/// What the tap does with a command: the slot it selects for pads or cards, if any.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Select {
    None,
    Pads(Option<usize>),
    Cards(Option<usize>),
}

/// Answers a whole command (`cmd[0]` the peripheral byte 21h) into `reply`, as long as `cmd`; whether the tap answered
/// and what it selects.
pub fn exchange(cmd: &[u8], reply: &mut [u8]) -> (bool, Select) {
    reply.fill(0xFF);
    let c = cmd.get(1).copied().unwrap_or(0);
    let slot = cmd.get(3).map(|&s| s as usize).filter(|&s| s < SLOTS);
    let (frame, select): (&[u8], _) = match c {
        0x12 | 0x13 => (&[0xFF, 0x80, 0x5A, 0x04, 0x00, 0x5A], Select::None),
        0x21 => (&[0xFF, 0x80, 0x5A, 0x00, 0x00, 0x00, 0x5A], Select::Pads(slot)),
        0x22 => (&[0xFF, 0x80, 0x5A, 0x00, 0x00, 0x00, 0x5A], Select::Cards(slot)),
        _ => return (false, Select::None),
    };
    for (r, &f) in reply.iter_mut().zip(frame) {
        *r = f;
    }
    (true, select)
}
