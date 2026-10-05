//! Serial EEPROM boards: a two-wire EEPROM (SDA, SCL) on a cartridge address, its boards known by serial since the
//! header only says "EEPROM". The X24C01's protocol (a start, a byte of seven address bits and R/W, then data with an
//! acknowledge after each byte, pages of four on writes) is MacDonald's commented Monster World routines
//! (`eeprom.txt`); the boards are his `gen-eeprom.txt` and `gen-hw.txt` §4.1. Nephrite_Native.md §10.

/// A board: the serial its header carries, the byte its lines are on, and the bits of SDA and SCL there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Board {
    pub serial: &'static str,
    pub address: u32,
    pub sda: u8,
    pub scl: u8,
    pub bytes: usize,
}

/// The boards the documents describe: Monster World III / Wonder Boy in Monster World, and Mega Man: The Wily Wars
/// (Rockman Megaworld), each an X24C01 of 128 bytes with SDA on bit 0 and SCL on bit 1 at `$200001`.
pub const BOARDS: [Board; 3] = [
    Board { serial: "G-4060", address: 0x20_0001, sda: 0, scl: 1, bytes: 128 },
    Board { serial: "T-12046", address: 0x20_0001, sda: 0, scl: 1, bytes: 128 },
    Board { serial: "T-12053", address: 0x20_0001, sda: 0, scl: 1, bytes: 128 },
];

/// The board a header's serial (`GM T-12046 -00`) names, if the documents know it.
pub fn board(serial: &str) -> Option<Board> {
    let s = serial.trim_start_matches("GM").trim();
    BOARDS.iter().copied().find(|b| s.starts_with(b.serial))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Idle,
    /// Receiving the command byte, then data bytes to write.
    Receive,
    /// Sending data bytes to read.
    Send,
}

pub struct Eeprom {
    pub board: Board,
    pub memory: Vec<u8>,
    state: State,
    /// The lines as the console drives them, and SDA as the EEPROM drives it (pulled up when released).
    scl: bool,
    sda_in: bool,
    sda_out: bool,
    shift: u8,
    bits: u8,
    /// Whether the command byte has been taken, the address the next byte goes to, and whether this transfer reads.
    addressed: bool,
    address: u8,
    read: bool,
    /// The ninth clock of a byte: the acknowledge.
    ack: bool,
}

impl Eeprom {
    pub fn new(board: Board) -> Eeprom {
        Eeprom {
            board,
            memory: vec![0xFF; board.bytes],
            state: State::Idle,
            scl: true,
            sda_in: true,
            sda_out: true,
            shift: 0,
            bits: 0,
            addressed: false,
            address: 0,
            read: false,
            ack: false,
        }
    }

    fn mask(&self) -> u8 {
        (self.board.bytes - 1) as u8
    }

    /// SDA as the console reads it: the wire, low if either side pulls it low.
    pub fn sda(&self) -> bool {
        self.sda_in && self.sda_out
    }

    /// The console's write of the lines' byte.
    pub fn write(&mut self, v: u8) {
        let scl = v >> self.board.scl & 1 != 0;
        let sda = v >> self.board.sda & 1 != 0;
        if self.scl && scl && self.sda_in != sda {
            if !sda {
                self.state = State::Receive;
                self.addressed = false;
                self.bits = 0;
                self.ack = false;
                self.sda_out = true;
            } else {
                self.state = State::Idle;
                self.sda_out = true;
            }
            self.sda_in = sda;
            return;
        }
        self.sda_in = sda;
        if !self.scl && scl {
            self.rise();
        } else if self.scl && !scl {
            self.fall();
        }
        self.scl = scl;
    }

    /// SCL rising: a bit is taken from SDA, or on the ninth clock of a read the console's acknowledge.
    fn rise(&mut self) {
        match self.state {
            State::Receive if !self.ack => {
                self.shift = self.shift << 1 | self.sda_in as u8;
                self.bits += 1;
            }
            State::Send if self.ack && self.sda_in => self.state = State::Idle,
            _ => {}
        }
    }

    /// SCL falling: after eight bits the EEPROM acknowledges and takes the byte; on a read it puts out the next bit.
    fn fall(&mut self) {
        match self.state {
            State::Receive => {
                if self.ack {
                    self.ack = false;
                    self.sda_out = true;
                    if self.read {
                        self.state = State::Send;
                        self.bits = 0;
                        self.shift = self.memory[self.address as usize];
                        self.put_bit();
                    }
                } else if self.bits == 8 {
                    self.take(self.shift);
                    self.bits = 0;
                    self.ack = true;
                    self.sda_out = false;
                }
            }
            State::Send => {
                if self.ack {
                    self.ack = false;
                    self.address = (self.address + 1) & self.mask();
                    self.shift = self.memory[self.address as usize];
                    self.bits = 0;
                    self.put_bit();
                } else if self.bits == 8 {
                    self.ack = true;
                    self.sda_out = true;
                } else {
                    self.put_bit();
                }
            }
            State::Idle => {}
        }
    }

    fn put_bit(&mut self) {
        self.sda_out = self.shift & 0x80 != 0;
        self.shift <<= 1;
        self.bits += 1;
    }

    /// A received byte: first the command, seven address bits and R/W; then data written in pages of four.
    fn take(&mut self, b: u8) {
        if !self.addressed {
            self.addressed = true;
            self.address = (b >> 1) & self.mask();
            self.read = b & 1 != 0;
            return;
        }
        if !self.read {
            self.memory[self.address as usize] = b;
            let page = self.address & !3;
            self.address = page | ((self.address + 1) & 3);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The console's side, bit by bit, as MacDonald's routines drive it.
    struct Master(Eeprom, u8);

    impl Master {
        fn lines(&mut self, sda: bool, scl: bool) {
            self.1 = (sda as u8) | (scl as u8) << 1;
            self.0.write(self.1);
        }
        fn start(&mut self) {
            self.lines(true, true);
            self.lines(false, true);
            self.lines(false, false);
        }
        fn stop(&mut self) {
            self.lines(false, false);
            self.lines(false, true);
            self.lines(true, true);
        }
        fn send(&mut self, b: u8) -> bool {
            for i in (0..8).rev() {
                let bit = b >> i & 1 != 0;
                self.lines(bit, false);
                self.lines(bit, true);
            }
            self.lines(true, false);
            self.lines(true, true);
            let ack = !self.0.sda();
            self.lines(true, false);
            ack
        }
        fn receive(&mut self, more: bool) -> u8 {
            let mut v = 0;
            self.lines(true, false);
            for _ in 0..8 {
                self.lines(true, true);
                v = v << 1 | self.0.sda() as u8;
                self.lines(true, false);
            }
            self.lines(!more, false);
            self.lines(!more, true);
            self.lines(!more, false);
            v
        }
    }

    #[test]
    fn a_page_written_reads_back_and_each_byte_is_acknowledged() {
        let mut m = Master(Eeprom::new(BOARDS[0]), 3);
        m.start();
        assert!(m.send(0x10 << 1), "the command is acknowledged");
        for b in [0xDE, 0xAD, 0xBE, 0xEF] {
            assert!(m.send(b));
        }
        m.stop();
        assert_eq!(&m.0.memory[0x10..0x14], &[0xDE, 0xAD, 0xBE, 0xEF]);
        m.start();
        assert!(m.send(0x11 << 1 | 1));
        assert_eq!((m.receive(true), m.receive(true), m.receive(false)), (0xAD, 0xBE, 0xEF));
        m.stop();
    }

    #[test]
    fn a_page_write_wraps_within_its_four_bytes() {
        let mut m = Master(Eeprom::new(BOARDS[0]), 3);
        m.start();
        m.send(0x22 << 1);
        for b in [1, 2, 3] {
            m.send(b);
        }
        m.stop();
        assert_eq!(&m.0.memory[0x20..0x24], &[3, 0xFF, 1, 2]);
    }

    #[test]
    fn boards_are_known_by_the_serial() {
        assert_eq!(board("GM T-12046 -00").map(|b| b.bytes), Some(128));
        assert_eq!(board("GM G-4060  -00").map(|b| b.address), Some(0x20_0001));
        assert!(board("GM T-081326 01").is_none(), "NBA Jam's wiring is not in the documents");
    }
}
