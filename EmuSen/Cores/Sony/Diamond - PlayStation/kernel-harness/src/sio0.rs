//! SIO0 for the harness: DATA, STAT and CTRL, and a memory card in slot 1 answering psx-spx's "Memory Card Read/Write
//! Commands" byte for byte, its /ACK raising IRQ7 a fixed number of the rig's accesses after each byte but the last.
//! It models the protocol, not the timing (Diamond_OpenKernel_Phase2.md §25.2).

pub const DATA: u32 = 0x1F80_1040;
pub const STAT: u32 = 0x1F80_1044;
pub const CTRL: u32 = 0x1F80_104A;
/// The rig's accesses from a byte's write to its /ACK: past the kernel's settling wait, within its shortest timeout.
pub const ACK_DELAY: u64 = 400;

/// 128 KiB, its FLAG byte, and where a command has reached.
pub struct Card {
    pub data: Vec<u8>,
    pub flag: u8,
    step: usize,
    command: u8,
    sector: u32,
    sum: u8,
    last: u8,
}

impl Card {
    /// A card of zeros, its FLAG 08h as on insertion ("FLAG Byte").
    pub fn blank() -> Card {
        Card { data: vec![0; 0x2_0000], flag: 0x08, step: 0, command: 0, sector: 0, sum: 0, last: 0 }
    }

    pub fn frame(&self, sector: u32) -> &[u8] {
        &self.data[sector as usize * 128..sector as usize * 128 + 128]
    }

    fn deselect(&mut self) {
        self.step = 0;
    }

    /// The reply to one byte, and whether /ACK follows it.
    fn exchange(&mut self, byte: u8) -> (u8, bool) {
        let step = self.step;
        self.step += 1;
        let reply = match (step, self.command) {
            (0, _) if byte == 0x81 => return (0xFF, true),
            (0, _) => {
                self.step = 0;
                return (0xFF, false);
            }
            (1, _) => {
                self.command = byte;
                if byte != 0x52 && byte != 0x57 {
                    return (self.flag, false);
                }
                return (self.flag, true);
            }
            (2, _) => 0x5A,
            (3, _) => 0x5D,
            (4, _) => {
                self.sector = (byte as u32) << 8;
                0x00
            }
            (5, _) => {
                self.sector |= byte as u32;
                self.sum = (self.sector >> 8) as u8 ^ byte;
                (self.sector >> 8) as u8
            }
            (6, 0x52) => 0x5C,
            (7, 0x52) => 0x5D,
            (8, 0x52) => (self.sector >> 8) as u8,
            (9, 0x52) => self.sector as u8,
            (10..=137, 0x52) => {
                let v = self.data[(self.sector as usize & 0x3FF) * 128 + step - 10];
                self.sum ^= v;
                v
            }
            (138, 0x52) => self.sum,
            (139, 0x52) => return (0x47, false),
            (6..=133, 0x57) => {
                self.data[(self.sector as usize & 0x3FF) * 128 + step - 6] = byte;
                self.sum ^= byte;
                self.last
            }
            (134, 0x57) => {
                self.flag &= !0x08;
                self.last
            }
            (135, 0x57) => 0x5C,
            (136, 0x57) => 0x5D,
            (137, 0x57) => return (0x47, false),
            _ => return (0xFF, false),
        };
        self.last = byte;
        (reply, true)
    }
}

/// The port's registers and slot 1's card; slot 2 and both controllers absent.
#[derive(Default)]
pub struct Sio0 {
    pub card: Option<Card>,
    ctrl: u32,
    rx: u8,
    rx_ready: bool,
    pub ack_at: Option<u64>,
}

impl Sio0 {
    pub fn write_data(&mut self, byte: u8, clock: u64) {
        let selected = self.ctrl & 2 != 0;
        let (reply, ack) = match (&mut self.card, selected, self.ctrl >> 13 & 1) {
            (Some(card), true, 0) => card.exchange(byte),
            _ => (0xFF, false),
        };
        self.rx = reply;
        self.rx_ready = true;
        self.ack_at = ack.then_some(clock + ACK_DELAY);
    }

    pub fn read_data(&mut self) -> u32 {
        self.rx_ready = false;
        self.rx as u32
    }

    pub fn stat(&self) -> u32 {
        0x05 | (self.rx_ready as u32) << 1
    }

    pub fn write_ctrl(&mut self, value: u32) {
        if value & 2 == 0 || value & 0x40 != 0 {
            if let Some(c) = &mut self.card {
                c.deselect();
            }
            self.ack_at = None;
        }
        self.ctrl = value & !0x50;
    }
}
