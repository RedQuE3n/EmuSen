//! Serial EEPROM boards: a two-wire EEPROM (SDA, SCL) on cartridge addresses, its boards known by serial since the
//! header only says "EEPROM". The protocol (a start, a command byte, the address in one of three modes, data with an
//! acknowledge after each byte, pages on writes) and the boards are Eke's "Serial EEPROMs in Sega Genesis / Mega Drive
//! cartridges" (version 2, 2010), with MacDonald's commented Monster World routines (`eeprom.txt`) and his
//! `gen-eeprom.txt` for the Sega boards. Nephrite_Native.md §10.1 and §11.4.

/// A line: the cartridge address it is on and its bit there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Line {
    pub address: u32,
    pub bit: u8,
}

/// How the address follows the start: seven bits in the command byte (an X24C01), a word address after it with the
/// command's three device bits as the upper bits (24C01-24C16), or two address bytes after it (24C32 and up).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    One,
    Two,
    Three,
}

/// A board: the serial its header carries (and its checksum where boards share a serial), the lines the console
/// writes SDA and SCL on and reads SDA from, the mode, and the chip's size and page less one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Board {
    pub serial: &'static str,
    pub checksum: Option<u16>,
    pub sda_in: Line,
    pub sda_out: Line,
    pub scl: Line,
    pub mode: Mode,
    pub size_mask: u16,
    pub page_mask: u16,
}

impl Board {
    pub fn bytes(&self) -> usize {
        self.size_mask as usize + 1
    }
}

const fn line(address: u32, bit: u8) -> Line {
    Line { address, bit }
}

const fn sega(serial: &'static str) -> Board {
    Board { serial, checksum: None, sda_in: line(0x20_0001, 0), sda_out: line(0x20_0001, 0), scl: line(0x20_0001, 1), mode: Mode::One, size_mask: 0x7F, page_mask: 3 }
}

const fn acclaim(serial: &'static str, mode: Mode, size_mask: u16, page_mask: u16) -> Board {
    Board { serial, checksum: None, sda_in: line(0x20_0001, 0), sda_out: line(0x20_0001, 0), scl: line(0x20_0000, 0), mode, size_mask, page_mask }
}

const fn codemasters(serial: &'static str, checksum: Option<u16>, mode: Mode, size_mask: u16, page_mask: u16) -> Board {
    Board { serial, checksum, sda_in: line(0x30_0000, 0), sda_out: line(0x38_0001, 7), scl: line(0x30_0000, 1), mode, size_mask, page_mask }
}

const fn ea(serial: &'static str) -> Board {
    Board { serial, checksum: None, sda_in: line(0x20_0001, 7), sda_out: line(0x20_0001, 7), scl: line(0x20_0001, 6), mode: Mode::One, size_mask: 0x7F, page_mask: 3 }
}

/// The boards of Eke's document whose games are in the corpus, by the serials their images carry (the document names
/// the games). Brian Lara Cricket 96's page, which the document leaves open, is the 24C65 datasheet's: a write's six
/// low address bits count, through its cache of 64 bytes.
pub const BOARDS: [Board; 21] = [
    sega("G-4060"),
    sega("T-12046"),
    sega("T-12053"),
    sega("MK-1215"),
    sega("MK-1228"),
    sega("00004076"),
    sega("00001211"),
    Board { serial: "T-081326", checksum: None, sda_in: line(0x20_0001, 0), sda_out: line(0x20_0001, 1), scl: line(0x20_0001, 1), mode: Mode::Two, size_mask: 0xFF, page_mask: 3 },
    acclaim("T-81406", Mode::Two, 0xFF, 3),
    acclaim("T-081276", Mode::Two, 0xFF, 3),
    acclaim("T-081586", Mode::Two, 0x7FF, 7),
    acclaim("T-81576", Mode::Three, 0x1FFF, 7),
    acclaim("T-81476", Mode::Three, 0x1FFF, 7),
    codemasters("T-120106", None, Mode::One, 0x7F, 3),
    codemasters("T-120096", None, Mode::Two, 0x3FF, 0xF),
    codemasters("00000000", Some(0x168B), Mode::Two, 0x3FF, 0xF),
    codemasters("00000000", Some(0x2C41), Mode::Two, 0x7FF, 0xF),
    codemasters("T-120146", None, Mode::Three, 0x1FFF, 0x3F),
    ea("T-50516"),
    ea("T-50396"),
    ea("T-50176"),
];

/// The board a header's serial (`GM T-12046 -00`, `MK 00001211-00`) and checksum name, if the document knows it.
pub fn board(serial: &str, checksum: u16) -> Option<Board> {
    let s = serial.get(2..).unwrap_or("").trim_start_matches([' ', '_']);
    BOARDS.iter().copied().find(|b| s.starts_with(b.serial) && b.checksum.is_none_or(|c| c == checksum))
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
    /// Bytes taken since the start, the address the next byte goes to, the command's upper address bits (mode 2),
    /// and whether this transfer reads.
    taken: u8,
    address: u16,
    upper: u16,
    read: bool,
    /// The ninth clock of a byte: the acknowledge.
    ack: bool,
}

impl Eeprom {
    pub fn new(board: Board) -> Eeprom {
        Eeprom {
            board,
            memory: vec![0xFF; board.bytes()],
            state: State::Idle,
            scl: true,
            sda_in: true,
            sda_out: true,
            shift: 0,
            bits: 0,
            taken: 0,
            address: 0,
            upper: 0,
            read: false,
            ack: false,
        }
    }

    /// Where a transfer stands, for the state: the step, the lines, the byte being shifted, its bits, the bytes taken,
    /// the address, the command's upper bits and the read and acknowledge flags (Nephrite_Native.md §39).
    pub fn protocol(&self) -> [u16; 11] {
        let state = match self.state {
            State::Idle => 0,
            State::Receive => 1,
            State::Send => 2,
        };
        [state, self.scl as u16, self.sda_in as u16, self.sda_out as u16, self.shift as u16, self.bits as u16, self.taken as u16, self.address, self.upper, self.read as u16, self.ack as u16]
    }

    /// A board at rest, its lines released: where a power-on leaves it.
    pub const IDLE: [u16; 11] = [0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0];

    pub fn set_protocol(&mut self, p: [u16; 11]) {
        self.state = match p[0] {
            1 => State::Receive,
            2 => State::Send,
            _ => State::Idle,
        };
        (self.scl, self.sda_in, self.sda_out) = (p[1] != 0, p[2] != 0, p[3] != 0);
        (self.shift, self.bits, self.taken) = (p[4] as u8, p[5] as u8, p[6] as u8);
        (self.address, self.upper, self.read, self.ack) = (p[7], p[8], p[9] != 0, p[10] != 0);
    }

    /// SDA as the console reads it: the wire, low if either side pulls it low.
    pub fn sda(&self) -> bool {
        self.sda_in && self.sda_out
    }

    /// Whether a write to `a` reaches a line.
    pub fn takes(&self, a: u32) -> bool {
        a == self.board.sda_in.address || a == self.board.scl.address
    }

    /// The byte a read of `a` returns, if SDA is read there: SDA on its bit, the others low.
    pub fn read(&self, a: u32) -> Option<u8> {
        (a == self.board.sda_out.address).then(|| (self.sda() as u8) << self.board.sda_out.bit)
    }

    /// The console's writes of one bus cycle: both lines change together, so a word write that sets SCL and SDA on
    /// two bytes is one change of the lines.
    pub fn write(&mut self, writes: &[(u32, u8)]) {
        let (mut sda, mut scl) = (self.sda_in, self.scl);
        for &(a, v) in writes {
            if a == self.board.sda_in.address {
                sda = v >> self.board.sda_in.bit & 1 != 0;
            }
            if a == self.board.scl.address {
                scl = v >> self.board.scl.bit & 1 != 0;
            }
        }
        self.lines(sda, scl);
    }

    fn lines(&mut self, sda: bool, scl: bool) {
        if self.scl && scl && self.sda_in != sda {
            if !sda {
                self.state = State::Receive;
                self.taken = 0;
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
                    self.address = (self.address + 1) & self.board.size_mask;
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

    /// A received byte: the command, then the address bytes its mode takes on a write, then data written within a
    /// page. A read takes no address bytes and goes on from the address counter, which a write's address set.
    fn take(&mut self, b: u8) {
        let (mask, n) = (self.board.size_mask, self.taken);
        self.taken = n.saturating_add(1);
        let b16 = b as u16;
        match (self.board.mode, n) {
            (_, 0) => {
                self.read = b & 1 != 0;
                match self.board.mode {
                    Mode::One => self.address = (b16 >> 1) & mask,
                    Mode::Two => self.upper = (b16 >> 1) & 7,
                    Mode::Three => {}
                }
            }
            (Mode::Two, 1) => self.address = (self.upper << 8 | b16) & mask,
            (Mode::Three, 1) => self.address = (b16 << 8) & mask,
            (Mode::Three, 2) => self.address = (self.address | b16) & mask,
            _ if !self.read => {
                self.memory[self.address as usize] = b;
                let page = self.board.page_mask;
                self.address = (self.address & !page) | ((self.address + 1) & page);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The console's side, bit by bit, as MacDonald's routines drive it, on the board's own lines.
    struct Master(Eeprom);

    impl Master {
        fn new(serial: &str, checksum: u16) -> Master {
            Master(Eeprom::new(board(serial, checksum).expect("a board")))
        }
        fn lines(&mut self, sda: bool, scl: bool) {
            let b = self.0.board;
            let byte = |l: Line, on: bool| (l.address, (on as u8) << l.bit);
            if b.sda_in.address == b.scl.address {
                self.0.write(&[(b.scl.address, byte(b.sda_in, sda).1 | byte(b.scl, scl).1)]);
            } else {
                self.0.write(&[byte(b.sda_in, sda), byte(b.scl, scl)]);
            }
        }
        fn sda(&self) -> bool {
            self.0.read(self.0.board.sda_out.address).unwrap() >> self.0.board.sda_out.bit & 1 != 0
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
            let ack = !self.sda();
            self.lines(true, false);
            ack
        }
        fn receive(&mut self, more: bool) -> u8 {
            let mut v = 0;
            self.lines(true, false);
            for _ in 0..8 {
                self.lines(true, true);
                v = v << 1 | self.sda() as u8;
                self.lines(true, false);
            }
            self.lines(!more, false);
            self.lines(!more, true);
            self.lines(!more, false);
            v
        }
    }

    // A state taken in the middle of a transfer carries where it stood: a board given the bytes alone answers otherwise (Nephrite_Native.md §39).
    #[test]
    fn a_transfer_taken_into_another_board_goes_on_as_it_would_have() {
        let mut m = Master::new("GM T-081586-00", 0);
        for (i, b) in m.0.memory.iter_mut().enumerate() {
            *b = (i * 7) as u8;
        }
        m.start();
        assert!(m.send(0xA0 | 1 << 1));
        assert!(m.send(0x40));
        m.start();
        assert!(m.send(0xA1 | 1 << 1));
        assert_eq!(m.receive(true), (0x140 * 7) as u8);
        let mut taken = Master(Eeprom::new(m.0.board));
        taken.0.memory.copy_from_slice(&m.0.memory);
        taken.0.set_protocol(m.0.protocol());
        let mut bytes_alone = Master(Eeprom::new(m.0.board));
        bytes_alone.0.memory.copy_from_slice(&m.0.memory);
        let next = |m: &mut Master| (m.receive(true), m.receive(false));
        let expected = ((0x141 * 7) as u8, (0x142 * 7) as u8);
        assert_eq!(next(&mut m), expected);
        assert_eq!(next(&mut taken), expected);
        assert_eq!(taken.0.protocol(), m.0.protocol());
        assert_ne!(next(&mut bytes_alone), expected);
    }

    #[test]
    fn a_page_written_reads_back_and_each_byte_is_acknowledged() {
        let mut m = Master::new("GM G-4060  -00", 0);
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
        let mut m = Master::new("GM G-4060  -00", 0);
        m.start();
        m.send(0x22 << 1);
        for b in [1, 2, 3] {
            m.send(b);
        }
        m.stop();
        assert_eq!(&m.0.memory[0x20..0x24], &[3, 0xFF, 1, 2]);
    }

    #[test]
    fn mode_two_takes_a_word_address_and_the_command_bits_above_it() {
        let mut m = Master::new("GM T-081586-00", 0);
        assert_eq!((m.0.board.bytes(), m.0.board.scl.address, m.0.board.sda_in.address), (2048, 0x20_0000, 0x20_0001));
        m.start();
        assert!(m.send(0xA0 | 5 << 1));
        assert!(m.send(0x3E));
        for b in [7, 8, 9] {
            m.send(b);
        }
        m.stop();
        assert_eq!(&m.0.memory[0x53E..0x541], &[7, 8, 0xFF], "the page of eight wraps at $540");
        assert_eq!(m.0.memory[0x538], 9);
        m.start();
        m.send(0xA0 | 5 << 1);
        m.send(0x3E);
        m.start();
        assert!(m.send(0xA1));
        assert_eq!((m.receive(true), m.receive(false)), (7, 8), "a random read: the address written, then a read");
        m.stop();
    }

    #[test]
    fn mode_three_takes_two_address_bytes_on_the_codemasters_lines() {
        let mut m = Master::new("GM T-120146-50", 0);
        assert_eq!((m.0.board.bytes(), m.0.board.sda_out), (8192, Line { address: 0x38_0001, bit: 7 }));
        m.start();
        m.send(0xA0);
        m.send(0x13);
        m.send(0x45);
        m.send(0x5A);
        m.stop();
        assert_eq!(m.0.memory[0x1345], 0x5A);
        m.start();
        m.send(0xA0);
        m.send(0x13);
        m.send(0x45);
        m.start();
        m.send(0xA1);
        assert_eq!(m.receive(false), 0x5A);
        m.stop();
    }

    #[test]
    fn a_word_write_changes_both_lines_at_once() {
        let mut e = Eeprom::new(board("GM T-81406 -00", 0).unwrap());
        e.write(&[(0x20_0000, 1), (0x20_0001, 1)]);
        e.write(&[(0x20_0000, 0), (0x20_0001, 0)]);
        assert_eq!(e.state, State::Idle, "SDA falling as SCL falls is no start");
        e.write(&[(0x20_0000, 1), (0x20_0001, 1)]);
        e.write(&[(0x20_0001, 0)]);
        assert_eq!(e.state, State::Receive, "SDA falling with SCL held high is");
    }

    #[test]
    fn boards_are_known_by_the_serial_and_where_they_share_one_the_checksum() {
        assert_eq!(board("GM T-12046 -00", 0).map(|b| b.bytes()), Some(128));
        assert_eq!(board("MK 00001211-00", 0).map(|b| b.scl), Some(Line { address: 0x20_0001, bit: 1 }));
        assert_eq!(board("GM_00004076-00", 0).map(|b| b.mode), Some(Mode::One));
        assert_eq!(board("GM T-50396 -00", 0).map(|b| b.sda_in.bit), Some(7));
        assert_eq!(board("GM 00000000-00", 0x168B).map(|b| b.bytes()), Some(1024));
        assert!(board("GM 00000000-00", 0x165E).is_none(), "Micro Machines, the first, has no EEPROM");
        assert!(board("GM 00054503-00", 0).is_none(), "Putter Golf's wiring is in no document");
    }
}
