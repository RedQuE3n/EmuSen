//! The Game Gear's system ports $00-$05 in GG mode (Jadeite_Native.md §9.2): Start and the market at $00, and the EXT
//! connector's parallel and serial registers at $01-$05 as a unit with nothing linked answers them [R Sega's Game Gear
//! hardware notes, "System control port"; MacDonald's `smstech` §3]. Port $06, the stereo register, is the PSG's.
//! Gear-to-Gear is out (Endou_Plan.md §2.4): no byte is ever received, and the lines no cable drives read high.

use emusen_native::{StateReader, StateWriter, Truncated};

/// Port $05's bits [R the notes, "I/O port 05H"]: the transmit buffer full, receive and transmit enabled.
pub const TXFL: u8 = 1 << 0;
pub const TON: u8 = 1 << 4;
pub const RON: u8 = 1 << 5;

/// The serial link's rates by port $05's bits 7-6, in bits a second [R the same].
const BAUD: [u64; 4] = [4800, 2400, 1200, 300];

/// The Z80's clock, the master clock over 15 [R the notes, "CPU clock": 3.579545 MHz].
const Z80_HZ: u64 = 53_693_175 / 15;

/// The registers software writes; the rest of what the ports read is the unlinked connector's.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ports {
    /// Port $01's output levels for the pins port $02 makes outputs.
    pub data: u8,
    /// Port $02: NINT and the seven pins' directions, 1 an input.
    pub direction: u8,
    /// Port $03: the byte to send.
    pub transmit: u8,
    /// Port $05's writable bits 7-3: the rate, RON, TON and INT.
    pub serial: u8,
    /// The Z80 clock until which the byte last sent is still going out (TXFL).
    pub sending_until: u64,
}

impl Default for Ports {
    /// Power-on: every pin an input, NINT off, the serial port off at 4,800 bits a second [R the notes' ☆ marks].
    fn default() -> Ports {
        Ports { data: 0, direction: 0xFF, transmit: 0, serial: 0, sending_until: 0 }
    }
}

impl Ports {
    /// A read of port `port` ($00-$06) at the Z80's `clock`, with Start held or not, on a Japanese unit or not. The
    /// unlinked unit's power-on reads are "C0 7F FF 00 FF 00 FF" [R `smstech` §3].
    pub fn read(&self, port: u8, start: bool, japanese: bool, clock: u64) -> u8 {
        match port {
            // STT, NJAP and NNTS, the low five bits 0; every Game Gear is NTSC [R the notes, "I/O port 00H"].
            0 => (!start as u8) << 7 | (!japanese as u8) << 6,
            // An input pin no cable drives reads 1, an output its level; TON and RON take PC4 and PC5 over, idle high [A].
            1 => {
                let inputs = self.direction & 0x7F;
                let mut v = (self.data & !inputs & 0x7F) | inputs;
                if self.serial & TON != 0 {
                    v |= 1 << 4;
                }
                if self.serial & RON != 0 {
                    v |= 1 << 5;
                }
                v
            }
            2 => self.direction,
            3 => self.transmit,
            // Nothing is ever received.
            4 => 0xFF,
            // RXRD and FRER stay clear with no line driven low [A]; TXFL while a byte goes out.
            5 => self.serial | (clock < self.sending_until) as u8,
            _ => 0xFF,
        }
    }

    /// A write of `v` to port `port` ($00-$05) at `clock`. A byte written to $03 with TON set goes out over ten bit
    /// times, a start bit, eight and a stop bit [R SMS Power!'s "Gear to Gear Cable", "Serial"], at the rate's clocks
    /// (Jadeite_Native.md §9.2).
    pub fn write(&mut self, port: u8, v: u8, clock: u64) {
        match port {
            1 => self.data = v,
            2 => self.direction = v,
            3 => {
                self.transmit = v;
                if self.serial & TON != 0 {
                    self.sending_until = clock + 10 * Z80_HZ / BAUD[(self.serial >> 6) as usize];
                }
            }
            5 => self.serial = v & 0xF8,
            _ => {}
        }
    }

    pub fn write_state(&self, w: &mut StateWriter) {
        w.bytes("GgPorts", &[self.data, self.direction, self.transmit, self.serial]);
        w.u64("GgSendingUntil", self.sending_until);
    }

    pub fn read_state(&mut self, r: &mut StateReader) -> Result<(), Truncated> {
        let mut b = [0u8; 4];
        r.bytes(&mut b)?;
        [self.data, self.direction, self.transmit, self.serial] = b;
        self.serial &= 0xF8;
        self.sending_until = r.u64()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unlinked_unit_reads_macdonald_s_power_on_values() {
        let p = Ports::default();
        let reads: Vec<u8> = (0..7).map(|port| p.read(port, false, false, 0)).collect();
        assert_eq!(reads, [0xC0, 0x7F, 0xFF, 0x00, 0xFF, 0x00, 0xFF]);
        assert_eq!((p.read(0, true, false, 0), p.read(0, false, true, 0), p.read(0, true, true, 0)), (0x40, 0x80, 0x00));
    }

    #[test]
    fn the_parallel_pins_read_their_levels_as_outputs_and_high_as_inputs() {
        let mut p = Ports::default();
        p.write(1, 0x00, 0);
        p.write(2, 0x80 | 0x0F, 0);
        assert_eq!(p.read(1, false, false, 0), 0x0F, "PC4-PC6 outputs at 0, PC0-PC3 inputs high, bit 7 0");
        p.write(1, 0xFF, 0);
        assert_eq!(p.read(1, false, false, 0), 0x7F);
        p.write(1, 0x00, 0);
        p.write(5, TON | RON, 0);
        assert_eq!(p.read(1, false, false, 0), 0x3F, "TON and RON take PC4 and PC5");
    }

    #[test]
    fn a_byte_sent_holds_txfl_for_ten_bit_times_and_nothing_arrives() {
        let mut p = Ports::default();
        p.write(3, 0x5A, 1000);
        assert_eq!((p.read(3, false, false, 1000), p.read(5, false, false, 1000)), (0x5A, 0x00), "TON off: nothing goes out");
        p.write(5, 0xC0 | TON | 0x07, 1000);
        assert_eq!(p.read(5, false, false, 1000), 0xC0 | TON, "the status bits are not written");
        p.write(3, 0xA5, 2000);
        let byte = 10 * Z80_HZ / 300;
        assert_eq!((p.read(5, false, false, 2000 + byte - 1) & TXFL, p.read(5, false, false, 2000 + byte) & TXFL), (TXFL, 0));
        assert_eq!(p.read(4, false, false, 2000 + byte), 0xFF);
    }
}
