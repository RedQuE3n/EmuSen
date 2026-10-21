//! The serial EEPROM a few Game Gear boards carry: Microchip's 93C46B, 64 words of 16 bits on a Microwire interface
//! (CS, CLK, DI, DO), from its data sheet [R DS21172E, 1998, SMS Power!'s `93c46doc.zip`] (Jadeite_Native.md §9.3).
//! The chip alone: which bits of which address drive its pins is the board's (`cart`, Endou_Disputes.md D-26). Its
//! memory is the caller's 128 bytes, word n in bytes 2n (low) and 2n + 1, so that it is the board's battery file.

use emusen_native::{StateReader, StateWriter, Truncated};

pub const WORDS: usize = 64;
pub const BYTES: usize = 2 * WORDS;

/// The self-timed cycles at the data sheet's maxima, in Z80 clocks of 3,579,545 Hz [R Table 1-2: TWC 2 ms for ERASE and
/// WRITE, TEC 6 ms for ERAL, TWL 15 ms for WRAL]; the sheet gives no typical value [A].
pub const WRITE_CLOCKS: u64 = 7_159;
pub const ERAL_CLOCKS: u64 = 21_477;
pub const WRAL_CLOCKS: u64 = 53_693;

/// Where the chip is in an instruction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Phase {
    /// CS low: held reset, DO high-Z [R §2.1].
    Standby = 0,
    /// CS high, waiting for a START: CS and DI high at a rising CLK [R §3.1].
    Waiting = 1,
    /// The opcode's two bits and the address's six.
    Command = 2,
    /// WRITE's or WRAL's sixteen data bits.
    Data = 3,
    /// READ: a dummy zero, then each word from D15, the address advancing while CS stays high [R §3.7].
    Reading = 4,
    /// An instruction done: CLK and DI are don't-cares until CS goes low [R §3.1].
    Done = 5,
    /// CS high again after an ERASE or WRITE began: DO low while busy, high when ready [R §2.4].
    Status = 6,
}

impl Phase {
    fn of(b: u8) -> Phase {
        [Phase::Standby, Phase::Waiting, Phase::Command, Phase::Data, Phase::Reading, Phase::Done, Phase::Status][(b as usize).min(6)]
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Eeprom {
    pub cs: bool,
    pub clk: bool,
    pub di: bool,
    pub phase: Phase,
    /// Bits taken in the current field, or the word's bits sent while reading.
    pub bits: u8,
    pub shift: u32,
    pub address: u8,
    /// WRITE (1) or WRAL (2) awaiting its data.
    pub op: u8,
    /// EWEN given since power-on and no EWDS since: the chip powers up disabled [R §3.3, §3.6].
    pub enabled: bool,
    /// The Z80 clock at which the self-timed cycle in progress ends.
    pub busy_until: u64,
    /// Whether an ERASE or WRITE began since the last START, so that CS's next rise shows the status.
    pub programmed: bool,
    /// DO: driven low or high, or high-Z (none).
    pub out: Option<bool>,
}

impl Default for Eeprom {
    fn default() -> Eeprom {
        Eeprom { cs: false, clk: false, di: false, phase: Phase::Standby, bits: 0, shift: 0, address: 0, op: 0, enabled: false, busy_until: 0, programmed: false, out: None }
    }
}

const WRITE: u8 = 1;
const WRAL: u8 = 2;

fn word(mem: &[u8], a: u8) -> u16 {
    u16::from_le_bytes([mem[2 * a as usize], mem[2 * a as usize + 1]])
}

fn set_word(mem: &mut [u8], a: u8, w: u16) {
    mem[2 * a as usize..2 * a as usize + 2].copy_from_slice(&w.to_le_bytes());
}

impl Eeprom {
    /// DO at the Z80's `clock`: the status while one is shown, else the level driven, else high-Z.
    pub fn output(&self, clock: u64) -> Option<bool> {
        if self.phase == Phase::Status { Some(clock >= self.busy_until) } else { self.out }
    }

    /// The three input pins set at `clock`, acting on CS's edges and CLK's rising edge; `true` where `mem` changed.
    pub fn pins(&mut self, cs: bool, clk: bool, di: bool, mem: &mut [u8], clock: u64) -> bool {
        let (was_cs, was_clk) = (self.cs, self.clk);
        (self.cs, self.clk, self.di) = (cs, clk, di);
        if !cs {
            // DO goes high-Z on CS's fall; a cycle in progress finishes regardless [R §2.1, §3.0].
            self.phase = Phase::Standby;
            self.out = None;
            return false;
        }
        if !was_cs {
            self.phase = if self.programmed { Phase::Status } else { Phase::Waiting };
            return false;
        }
        if !clk || was_clk {
            return false;
        }
        match self.phase {
            // A START while busy is not taken: the chip finishes its cycle first [A].
            Phase::Waiting | Phase::Status if di && clock >= self.busy_until => {
                (self.phase, self.bits, self.shift, self.out, self.programmed) = (Phase::Command, 0, 0, None, false);
                false
            }
            Phase::Command => {
                self.shift = self.shift << 1 | di as u32;
                self.bits += 1;
                if self.bits == 8 { self.decode(mem, clock) } else { false }
            }
            Phase::Data => {
                self.shift = self.shift << 1 | di as u32;
                self.bits += 1;
                if self.bits < 16 {
                    return false;
                }
                self.phase = Phase::Done;
                let (w, a) = (self.shift as u16, self.address);
                match self.op {
                    WRITE => self.program(clock, WRITE_CLOCKS, || set_word(mem, a, w)),
                    _ => self.program(clock, WRAL_CLOCKS, || (0..WORDS as u8).for_each(|a| set_word(mem, a, w))),
                }
            }
            Phase::Reading => {
                if self.bits == 16 {
                    self.address = (self.address + 1) % WORDS as u8;
                    self.bits = 0;
                }
                self.out = Some(word(mem, self.address) >> (15 - self.bits) & 1 != 0);
                self.bits += 1;
                false
            }
            _ => false,
        }
    }

    /// The opcode and address taken in [R Table 2-1].
    fn decode(&mut self, mem: &mut [u8], clock: u64) -> bool {
        let (opcode, a) = ((self.shift >> 6) as u8, self.shift as u8 & 0x3F);
        self.phase = Phase::Done;
        match opcode {
            0b10 => {
                (self.phase, self.address, self.bits, self.out) = (Phase::Reading, a, 0, Some(false));
                false
            }
            0b01 => {
                (self.phase, self.address, self.op, self.bits, self.shift) = (Phase::Data, a, WRITE, 0, 0);
                false
            }
            0b11 => self.program(clock, WRITE_CLOCKS, || set_word(mem, a, 0xFFFF)),
            _ => match a >> 4 {
                0b11 => {
                    self.enabled = true;
                    false
                }
                0b00 => {
                    self.enabled = false;
                    false
                }
                0b10 => self.program(clock, ERAL_CLOCKS, || mem[..BYTES].fill(0xFF)),
                _ => {
                    (self.phase, self.op, self.bits, self.shift) = (Phase::Data, WRAL, 0, 0);
                    false
                }
            },
        }
    }

    /// A self-timed cycle of `clocks` from `clock`, where EWEN allows one.
    fn program(&mut self, clock: u64, clocks: u64, change: impl FnOnce()) -> bool {
        if !self.enabled {
            return false;
        }
        change();
        self.busy_until = clock + clocks;
        self.programmed = true;
        true
    }

    pub fn write_state(&self, w: &mut StateWriter) {
        let out = self.out.map_or(2, |o| o as u8);
        w.bytes("EepromPins", &[self.cs as u8, self.clk as u8, self.di as u8, self.phase as u8, self.bits, self.address, self.op, self.enabled as u8, self.programmed as u8, out]);
        w.u32("EepromShift", self.shift);
        w.u64("EepromBusyUntil", self.busy_until);
    }

    pub fn read_state(&mut self, r: &mut StateReader) -> Result<(), Truncated> {
        let mut b = [0u8; 10];
        r.bytes(&mut b)?;
        (self.cs, self.clk, self.di, self.phase) = (b[0] != 0, b[1] != 0, b[2] != 0, Phase::of(b[3]));
        (self.bits, self.address, self.op, self.enabled, self.programmed) = (b[4].min(16), b[5] % WORDS as u8, b[6], b[7] != 0, b[8] != 0);
        self.out = match b[9] {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        };
        self.shift = r.u32()?;
        self.busy_until = r.u64()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A master driving the pins as the data sheet's figures do, one rising edge a bit, the clock advancing 100 a call.
    struct Master {
        chip: Eeprom,
        mem: Vec<u8>,
        clock: u64,
    }

    impl Master {
        fn new() -> Master {
            Master { chip: Eeprom::default(), mem: vec![0xFF; BYTES], clock: 0 }
        }

        fn set(&mut self, cs: bool, clk: bool, di: bool) {
            self.clock += 100;
            self.chip.pins(cs, clk, di, &mut self.mem, self.clock);
        }

        fn bit(&mut self, di: bool) -> Option<bool> {
            self.set(true, false, di);
            self.set(true, true, di);
            self.chip.output(self.clock)
        }

        /// An instruction: CS low then high, the START bit, then `n` bits of `bits` from the highest.
        fn send(&mut self, bits: u32, n: u32) {
            self.set(false, false, false);
            self.set(true, false, false);
            self.bit(true);
            for i in (0..n).rev() {
                self.bit(bits >> i & 1 != 0);
            }
        }

        fn read(&mut self, a: u8, words: usize) -> Vec<u16> {
            self.send(0b10 << 6 | a as u32, 8);
            assert_eq!(self.chip.output(self.clock), Some(false), "the dummy zero");
            (0..words).map(|_| (0..16).fold(0u16, |w, _| w << 1 | self.bit(false).unwrap() as u16)).collect()
        }

        /// CS low then high, and the clocks until DO reads ready.
        fn wait_ready(&mut self) -> u64 {
            self.set(false, false, false);
            self.set(true, false, false);
            let start = self.clock;
            while self.chip.output(self.clock) == Some(false) {
                self.clock += 1;
            }
            assert_eq!(self.chip.output(self.clock), Some(true));
            self.clock - start
        }
    }

    #[test]
    fn the_chip_powers_up_disabled_and_ewen_allows_writing() {
        let mut m = Master::new();
        m.send(0b01 << 22 | 5 << 16 | 0x1234, 24);
        assert_eq!(m.read(5, 1), [0xFFFF], "a WRITE before EWEN changes nothing");
        m.send(0b0011 << 4, 8);
        m.send(0b01 << 22 | 5 << 16 | 0x1234, 24);
        assert!(m.wait_ready() > 0);
        assert_eq!((m.read(5, 1), &m.mem[10..12]), (vec![0x1234], &[0x34u8, 0x12][..]), "word 5 in bytes 10 and 11, low first");
        m.send(0b0000 << 4, 8);
        m.send(0b01 << 22 | 5 << 16 | 0xBEEF, 24);
        assert_eq!(m.read(5, 1), [0x1234], "EWDS protects again");
    }

    #[test]
    fn the_status_is_busy_for_the_self_timed_cycle() {
        let mut m = Master::new();
        m.send(0b0011 << 4, 8);
        m.send(0b01 << 22 | 9 << 16 | 0x0F0F, 24);
        let end = m.chip.busy_until;
        let took = m.wait_ready();
        assert!((WRITE_CLOCKS - 300..=WRITE_CLOCKS).contains(&took), "{took}");
        assert_eq!(m.clock, end);
        m.send(0b0010 << 4, 8);
        assert!((ERAL_CLOCKS - 300..=ERAL_CLOCKS).contains(&m.wait_ready()));
        assert!(m.mem.iter().all(|&b| b == 0xFF), "ERAL erases every word");
        m.send(0b0001 << 20 | 0xA55A, 24);
        assert!((WRAL_CLOCKS - 300..=WRAL_CLOCKS).contains(&m.wait_ready()));
        assert_eq!(m.read(0, 64), vec![0xA55A; 64]);
        m.set(false, false, false);
        assert_eq!(m.chip.output(m.clock), None, "high-Z with CS low");
    }

    #[test]
    fn erase_sets_one_word_and_read_runs_on_through_the_array() {
        let mut m = Master::new();
        m.send(0b0011 << 4, 8);
        for a in 0..64u32 {
            m.send(0b01 << 22 | a << 16 | (a * 0x101), 24);
            m.wait_ready();
        }
        m.send(0b11 << 6 | 7, 8);
        m.wait_ready();
        let all = m.read(62, 4);
        assert_eq!(all, [62 * 0x101, 63 * 0x101, 0, 0x101], "sequential reads wrap at the array's end");
        assert_eq!(m.read(6, 3), [6 * 0x101, 0xFFFF, 8 * 0x101], "ERASE leaves the word all ones");
    }

    #[test]
    fn nothing_starts_without_cs_and_di_high_at_a_rising_clock() {
        let mut m = Master::new();
        m.chip.enabled = true;
        m.set(true, false, false);
        for _ in 0..20 {
            m.bit(false);
        }
        assert_eq!(m.chip.phase, Phase::Waiting, "zeros before a START are ignored");
        m.set(false, true, true);
        m.set(false, false, true);
        assert_eq!(m.chip.phase, Phase::Standby, "CLK is a don't-care with CS low");
    }

    #[test]
    fn a_state_round_trips_mid_instruction() {
        let mut m = Master::new();
        m.send(0b0011 << 4, 8);
        m.send(0b01 << 6 | 3, 8);
        let mut bytes = vec![0u8; 64];
        let mut w = StateWriter::new(&mut bytes);
        m.chip.write_state(&mut w);
        let n = w.len();
        let mut chip = Eeprom::default();
        chip.read_state(&mut StateReader::new(&bytes[..n])).unwrap();
        assert_eq!(chip, m.chip);
    }
}
