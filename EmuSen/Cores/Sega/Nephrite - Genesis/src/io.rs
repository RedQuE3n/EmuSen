//! The I/O chip at `$A10000`: the version register, the three ports' data, control and serial registers, and the
//! pads on ports A and B with TH's multiplexing and the six-button pad's count. Charles MacDonald's "Sega Genesis
//! hardware notes" §3 is the source; Nephrite_Native.md §9 records the readings. A port may instead hold Sega's Team
//! Player or, with the other port, Electronic Arts' 4 Way Play, each with four pads: Nephrite_Native.md §44.

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

/// The most pads the two ports hold: a Team Player on each.
pub const PADS: usize = 8;

/// What is plugged into a port.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Plug {
    /// One pad.
    #[default]
    Pad,
    /// Sega's Team Player in its MULTI setting: four pads read as a packet of nibbles.
    TeamPlayer,
    /// Electronic Arts' 4 Way Play, or the Team Player in its EXTRA setting: four pads on port A, chosen through port
    /// B, which it also takes. It is port A's plug; port B's is then not used.
    FourWay,
}

/// A Team Player's place in its packet: TH and TR as the console drives them, and the nibbles asked for since TH fell.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Tap {
    th: bool,
    tr: bool,
    asked: u8,
}

impl Tap {
    /// TH high rests the tap; its fall starts a packet; each change of TR while TH is low asks for the next nibble.
    fn set(&mut self, th: bool, tr: bool) {
        if !th && self.th {
            self.asked = 0;
        } else if !th && tr != self.tr {
            self.asked = self.asked.saturating_add(1);
        }
        (self.th, self.tr) = (th, tr);
    }

    /// TL and D3-D0: at rest `%0011`, which with the packet's start gives the peripheral ID 7; `%1111` when TH has
    /// fallen and nothing is asked; then the packet's nibbles, TL following TR as each is ready. Past the packet's
    /// end the lines are released (argued: no document says what is there).
    fn lines(&self, pads: &[Pad]) -> u8 {
        if self.th {
            return 0x13;
        }
        let nibble = match self.asked {
            0 => 0xF,
            n => packet(pads).get(n as usize - 1).copied().unwrap_or(0xF),
        };
        (self.tr as u8) << 4 | nibble
    }
}

/// A Team Player's packet: two zero nibbles, each port's kind (0 a three-button pad, 1 a six-button), then each pad's
/// nibbles, a pressed button 0: Right Left Down Up, Start A C B and, for six buttons, Mode X Y Z.
fn packet(pads: &[Pad]) -> Vec<u8> {
    let mut p = vec![0, 0];
    p.extend(pads.iter().map(|pad| pad.six as u8));
    for pad in pads {
        let b = !pad.buttons;
        p.extend([(b & 0xF) as u8, (b >> 4 & 0xF) as u8]);
        if pad.six {
            p.push((b >> 8 & 0xF) as u8);
        }
    }
    p
}

/// The ports' registers, the pads' TH and count and the taps' places, for the state; the buttons and the plugs are
/// input, set each frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IoRegs {
    pub data: [u8; 3],
    pub ctrl: [u8; 3],
    pub tx: [u8; 3],
    pub sctrl: [u8; 3],
    pub th: [bool; PADS],
    pub falls: [u8; PADS],
    pub last_fall: [u64; PADS],
    /// Each port's tap: TH, TR and the nibbles asked for.
    pub taps: [u8; 6],
}

pub struct Io {
    pub version: u8,
    data: [u8; 3],
    ctrl: [u8; 3],
    tx: [u8; 3],
    sctrl: [u8; 3],
    /// The pads in the players' order: port A's, then port B's (`first_pad`).
    pub pads: [Pad; PADS],
    pub plugs: [Plug; 2],
    taps: [Tap; 2],
    /// TH as a device drives it, for the external interrupt.
    th_in: [bool; 3],
}

/// How many pads `plug` holds.
pub fn pads_of(plug: Plug) -> usize {
    if plug == Plug::Pad { 1 } else { 4 }
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
            taps: [self.taps[0].th as u8, self.taps[0].tr as u8, self.taps[0].asked, self.taps[1].th as u8, self.taps[1].tr as u8, self.taps[1].asked],
        }
    }

    pub fn set_regs_state(&mut self, s: IoRegs) {
        (self.data, self.ctrl, self.tx, self.sctrl) = (s.data, s.ctrl, s.tx, s.sctrl);
        for (i, p) in self.pads.iter_mut().enumerate() {
            (p.th, p.falls, p.last_fall) = (s.th[i], s.falls[i], s.last_fall[i]);
        }
        for (i, t) in self.taps.iter_mut().enumerate() {
            *t = Tap { th: s.taps[3 * i] != 0, tr: s.taps[3 * i + 1] != 0, asked: s.taps[3 * i + 2] };
        }
    }

    /// The first of port `port`'s pads among `pads`: port B's follow port A's.
    pub fn first_pad(&self, port: usize) -> usize {
        if port == 0 { 0 } else { pads_of(self.plugs[0]) }
    }

    /// How many pads are plugged in: the players the two ports hold.
    pub fn pad_count(&self) -> usize {
        match self.plugs[0] {
            Plug::FourWay => 4,
            a => pads_of(a) + pads_of(if self.plugs[1] == Plug::FourWay { Plug::Pad } else { self.plugs[1] }),
        }
    }

    /// Port B's plug as it is used: the 4 Way Play is port A's, and takes port B.
    fn plug(&self, port: usize) -> Plug {
        match (port, self.plugs[0], self.plugs[1]) {
            (0, a, _) => a,
            (_, Plug::FourWay, _) => Plug::FourWay,
            (_, _, Plug::FourWay) => Plug::Pad,
            (_, _, b) => b,
        }
    }

    /// A line of `port` as the console drives it: the data register's bit where the control register makes it an
    /// output, pulled high otherwise.
    fn driven(&self, port: usize, bit: u8) -> bool {
        self.ctrl[port] & bit == 0 || self.data[port] & bit != 0
    }

    /// The version register: overseas, PAL, no expansion unit, and the model's version (TMSS from 1).
    pub fn new(overseas: bool, pal: bool, version: u8) -> Io {
        let v = (overseas as u8) << 7 | (pal as u8) << 6 | 1 << 5 | (version & 0xF);
        let pads = [Pad { th: true, ..Pad::default() }; PADS];
        let taps = [Tap { th: true, tr: true, asked: 0 }; 2];
        Io { version: v, data: [0x7F; 3], ctrl: [0; 3], tx: [0xFF; 3], sctrl: [0; 3], pads, plugs: [Plug::Pad; 2], taps, th_in: [true; 3] }
    }

    /// TH and TR as driven reach what the port holds: a pad's TH, a Team Player's two lines, or, on port A of a 4 Way
    /// Play, the TH all four pads share.
    fn drive(&mut self, port: usize, clock: u64) {
        if port > 1 {
            return;
        }
        let (th, tr) = (self.driven(port, 0x40), self.driven(port, 0x20));
        let first = self.first_pad(port);
        match self.plug(port) {
            Plug::Pad => self.pads[first].set_th(th, clock),
            Plug::TeamPlayer => self.taps[port].set(th, tr),
            Plug::FourWay if port == 0 => self.pads[..4].iter_mut().for_each(|p| p.set_th(th, clock)),
            Plug::FourWay => {}
        }
    }

    /// The six lines below TH as what `port` holds gives them. A 4 Way Play gives on port A the pad that port B's
    /// lines 6 to 4 choose, 0 to 3; 7 asks for its mark, D1 and D0 low; port B itself gives nothing.
    fn device(&self, port: usize, clock: u64) -> u8 {
        let first = self.first_pad(port);
        match self.plug(port) {
            Plug::Pad => self.pads[first].lines(clock),
            Plug::TeamPlayer => self.taps[port].lines(&self.pads[first..first + 4]) | (self.driven(port, 0x20) as u8) << 5,
            Plug::FourWay if port == 0 => {
                let chosen = [0x40u8, 0x20, 0x10].iter().fold(0, |n, &bit| n << 1 | self.driven(1, bit) as usize);
                match chosen {
                    0..=3 => self.pads[chosen].lines(clock),
                    7 => 0x3C,
                    _ => 0x3F,
                }
            }
            Plug::FourWay => 0x3F,
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
                let inputs = if p < 2 { self.device(p, clock) | (self.driven(p, 0x40) as u8) << 6 } else { 0x7F };
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

    const C: u32 = 1 << 5;
    const X: u32 = 1 << 10;

    /// Port `port`'s data and control addresses.
    fn regs(port: u32) -> (u32, u32) {
        (0xA1_0003 + 2 * port, 0xA1_0009 + 2 * port)
    }

    // Plutiedev's "Peripheral ID": TH high then low, each read's D3|D2 and D1|D0; the Team Player's is 7, a pad's 13.
    fn peripheral_id(io: &mut Io, port: u32) -> u8 {
        let (data, ctrl) = regs(port);
        io.write(ctrl, 0x40, 0);
        io.write(data, 0x40, 0);
        let high = io.read(data, 0);
        io.write(data, 0x00, 0);
        let low = io.read(data, 0);
        io.write(data, 0x40, 0);
        let pair = |v: u8| ((v & 0xC != 0) as u8) << 1 | (v & 3 != 0) as u8;
        pair(high) << 2 | pair(low)
    }

    /// A Team Player's packet read as plutiedev's "Reading the Sega multitap" has it: `$60`, `$20`, then TR toggled for
    /// each nibble with TL checked against it; the two checks' nibbles first.
    fn read_team_player(io: &mut Io, port: u32, nibbles: usize) -> Vec<u8> {
        let (data, ctrl) = regs(port);
        io.write(ctrl, 0x60, 0);
        io.write(data, 0x60, 0);
        let mut got = vec![io.read(data, 0) & 0xF];
        io.write(data, 0x20, 0);
        got.push(io.read(data, 0) & 0xF);
        let mut tr = 0x20;
        for _ in 0..nibbles {
            tr ^= 0x20;
            io.write(data, tr, 0);
            let v = io.read(data, 0);
            assert_eq!(v & 0x10, tr >> 1, "TL follows TR when the nibble is ready");
            got.push(v & 0xF);
        }
        io.write(data, 0x60, 0);
        got
    }

    #[test]
    fn a_team_player_answers_its_id_and_its_packet() {
        let mut io = Io::new(true, false, 0);
        assert_eq!(peripheral_id(&mut io, 0), 0xD, "a pad");
        io.plugs[0] = Plug::TeamPlayer;
        assert_eq!(peripheral_id(&mut io, 0), 7);
        io.pads[0].buttons = UP | B;
        io.pads[1].buttons = RIGHT | START;
        io.pads[2] = Pad { buttons: A | X | MODE, six: true, ..io.pads[2] };
        io.pads[3].buttons = C;
        let got = read_team_player(&mut io, 0, 6 + 2 + 2 + 3 + 2 + 2);
        assert_eq!(got[..2], [0x3, 0xF], "the two checks");
        assert_eq!(got[2..8], [0, 0, 0, 0, 1, 0], "two zeroes, then each port's kind");
        assert_eq!(got[8..10], [0xE, 0xE], "pad 1: Up; B");
        assert_eq!(got[10..12], [0x7, 0x7], "pad 2: Right; Start");
        assert_eq!(got[12..15], [0xF, 0xB, 0x3], "pad 3: none; A; Mode and X");
        assert_eq!(got[15..17], [0xF, 0xD], "pad 4: none; C");
        assert_eq!(got[17..], [0xF, 0xF], "past the packet");
        assert_eq!(read_team_player(&mut io, 0, 6)[2..], [0, 0, 0, 0, 1, 0], "a new packet after TH rises");
    }

    #[test]
    fn a_team_player_on_port_b_holds_the_pads_after_port_as() {
        let mut io = Io::new(true, false, 0);
        io.plugs[1] = Plug::TeamPlayer;
        io.pads[0].buttons = START;
        io.pads[1].buttons = UP;
        io.pads[4].buttons = RIGHT;
        assert_eq!((io.pad_count(), io.first_pad(1)), (5, 1));
        io.write(0xA1_0009, 0x40, 0);
        io.write(0xA1_0003, 0x00, 0);
        assert_eq!(io.read(0xA1_0003, 0) & 0x20, 0, "port A's pad is pad 1: Start");
        let got = read_team_player(&mut io, 1, 6 + 8);
        assert_eq!((got[8], got[14]), (0xE, 0x7), "the tap's first pad is pad 2, its fourth pad 5");
        io.plugs[0] = Plug::TeamPlayer;
        assert_eq!((io.pad_count(), io.first_pad(1)), (8, 4));
    }

    // MacDonald's "EA 4-Way Play" and plutiedev's "EA multitap": port B's $0C to $3C choose the pad port A reads, $7C its mark.
    #[test]
    fn a_4_way_play_gives_port_a_the_pad_port_b_chooses() {
        let mut io = Io::new(true, false, 0);
        io.plugs[0] = Plug::FourWay;
        assert_eq!(io.pad_count(), 4);
        for (i, b) in [UP, RIGHT | B, A, START | C].into_iter().enumerate() {
            io.pads[i].buttons = b;
        }
        io.write(0xA1_0009, 0x40, 0);
        io.write(0xA1_000B, 0x7F, 0);
        io.write(0xA1_0003, 0x40, 0);
        io.write(0xA1_0005, 0x0C, 0);
        let first = io.read(0xA1_0003, 0) & 3;
        io.write(0xA1_0005, 0x7C, 0);
        assert_eq!((first != 0, io.read(0xA1_0003, 0) & 3), (true, 0), "the detection: something, then nothing");
        let mut read = Vec::new();
        for select in [0x0C, 0x1C, 0x2C, 0x3C] {
            io.write(0xA1_0005, select, 0);
            io.write(0xA1_0003, 0x40, 0);
            let high = io.read(0xA1_0003, 0) & 0x3F;
            io.write(0xA1_0003, 0x00, 0);
            read.push(!(high | (io.read(0xA1_0003, 0) & 0x30) << 2) as u32 & 0xFF);
        }
        assert_eq!(read, [UP, RIGHT | B, A, START | C], "S A C B R L D U of each pad");
        assert_eq!(io.read(0xA1_0005, 0) & 0x7F, 0x3C, "port B reads back what it drives");
        io.plugs[0] = Plug::Pad;
        io.write(0xA1_0005, 0x7C, 0);
        io.write(0xA1_0003, 0x40, 0);
        assert_eq!(io.read(0xA1_0003, 0) & 3, 2, "without the adapter port A is its own pad, here with Up held");
    }

    // The four pads of a 4 Way Play share TH, so a six-button pad counts the falls made while another is chosen.
    #[test]
    fn the_pads_of_a_4_way_play_share_th() {
        let mut io = Io::new(true, false, 0);
        io.plugs[0] = Plug::FourWay;
        io.pads[2] = Pad { buttons: Z | MODE, six: true, ..io.pads[2] };
        io.write(0xA1_0009, 0x40, 0);
        io.write(0xA1_000B, 0x7F, 0);
        io.write(0xA1_0005, 0x0C, 0);
        for (i, th) in [0x40, 0x00, 0x40, 0x00, 0x40].into_iter().enumerate() {
            io.write(0xA1_0003, th, 100 * i as u64);
        }
        io.write(0xA1_0005, 0x2C, 450);
        assert_eq!(io.read(0xA1_0003, 450) & 0x0F, 0x06, "pad 3, chosen after two falls it was not chosen for, gives Mode X Y Z");
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
