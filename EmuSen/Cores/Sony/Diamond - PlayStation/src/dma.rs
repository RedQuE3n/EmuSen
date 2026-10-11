//! The DMA controller's registers (psx-spx "DMA Channels"): seven channels' MADR, BCR and CHCR, DPCR and DICR, and the
//! decisions a transfer needs (whether it may start, how it is cut into pieces, what it costs). The words themselves
//! are moved by the bus, which owns RAM and the devices (Diamond_Native.md §11.7).

pub const MDEC_IN: usize = 0;
pub const MDEC_OUT: usize = 1;
pub const GPU: usize = 2;
pub const CDROM: usize = 3;
pub const SPU: usize = 4;
pub const PIO: usize = 5;
pub const OTC: usize = 6;

pub mod chcr {
    pub const FROM_RAM: u32 = 1 << 0;
    pub const BACKWARD: u32 = 1 << 1;
    pub const CHOP: u32 = 1 << 8;
    pub const START: u32 = 1 << 24;
    pub const TRIGGER: u32 = 1 << 28;
    /// What a CHCR write keeps: bits 2-7, 11-15, 19, 23, 25-27 and 31 read zero.
    pub const WRITABLE: u32 = 0x7177_0703;
    /// OTC's: only 24, 28 and 30, with bit 1 (backward) always set.
    pub const OTC_WRITABLE: u32 = 0x5100_0000;
}

pub const DPCR_RESET: u32 = 0x0765_4321;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Channel {
    pub madr: u32,
    pub bcr: u32,
    pub chcr: u32,
}

impl Channel {
    pub fn sync(&self) -> u32 {
        self.chcr >> 9 & 3
    }
}

#[derive(Clone, Debug)]
pub struct Dma {
    pub ch: [Channel; 7],
    pub dpcr: u32,
    pub dicr: u32,
    /// The channel moving now and the CPU cycle its next piece is due, if a transfer is under way in pieces.
    pub active: Option<usize>,
    pub due: u64,
    /// Words left of a chopped burst, and whether the event at `due` ends the transfer.
    pub left: u32,
    pub finishing: bool,
}

impl Default for Dma {
    fn default() -> Dma {
        Dma { ch: [Channel { madr: 0, bcr: 0, chcr: 0 }; 7], dpcr: DPCR_RESET, dicr: 0, active: None, due: u64::MAX, left: 0, finishing: false }
    }
}

/// Words moved per CPU cycle for each channel, as sixteenths of a cycle per word (psx-spx "DMA Transfer Rates").
pub fn sixteenths_per_word(channel: usize, cd_delay: u32) -> u64 {
    match channel {
        CDROM if cd_delay & 1 << 8 != 0 => 40 * 16,
        CDROM => 24 * 16,
        SPU => 66,
        PIO => 20 * 16,
        _ => 17,
    }
}

impl Dma {
    pub fn read(&self, offset: u32) -> u32 {
        match offset {
            0x70 => self.dpcr,
            0x74 => self.dicr,
            o if o < 0x70 => {
                let c = &self.ch[(o >> 4) as usize];
                match o & 0xC {
                    0 => c.madr,
                    4 => c.bcr,
                    _ => c.chcr,
                }
            }
            _ => 0,
        }
    }

    /// A register write; returns whether the DICR's master flag rose (I_STAT's DMA edge).
    pub fn write(&mut self, offset: u32, value: u32) -> bool {
        match offset {
            0x70 => self.dpcr = value,
            0x74 => {
                let flags = self.dicr & 0x7F00_0000 & !(value & 0x7F00_0000);
                self.dicr = flags | value & 0x00FF_807F;
                return self.recompute();
            }
            o if o < 0x70 => {
                let n = (o >> 4) as usize;
                let c = &mut self.ch[n];
                match o & 0xC {
                    0 => c.madr = value & 0x00FF_FFFF,
                    4 => c.bcr = value,
                    _ if n == OTC => c.chcr = value & chcr::OTC_WRITABLE | chcr::BACKWARD,
                    _ => c.chcr = value & chcr::WRITABLE,
                }
            }
            _ => {}
        }
        false
    }

    /// DICR's bit 31: the bus error flag, or the master enable with any flag (psx-spx); whether it rose.
    fn recompute(&mut self) -> bool {
        let was = self.dicr & 1 << 31 != 0;
        let on = self.dicr & 1 << 15 != 0 || (self.dicr & 1 << 23 != 0 && self.dicr & 0x7F00_0000 != 0);
        self.dicr = self.dicr & 0x7FFF_FFFF | (on as u32) << 31;
        on && !was
    }

    /// A channel's transfer, or a piece of it with DICR's bit 0-6 set, completed: its flag if enabled, and whether the
    /// master flag rose.
    pub fn flag(&mut self, n: usize) -> bool {
        if self.dicr & 1 << (16 + n) != 0 && self.dicr & 1 << 23 != 0 {
            self.dicr |= 1 << (24 + n);
        }
        self.recompute()
    }

    pub fn enabled(&self, n: usize) -> bool {
        self.dpcr & 8 << (4 * n) != 0
    }

    /// Whether channel `n` may move now: started, enabled in DPCR, and triggered (bit 28) or requested by its device.
    pub fn ready(&self, n: usize, request: bool) -> bool {
        let c = &self.ch[n];
        c.chcr & chcr::START != 0 && self.enabled(n) && (c.chcr & chcr::TRIGGER != 0 || (request && (c.sync() != 0 || n != OTC)))
    }

    /// Of the ready channels, the one DPCR ranks first: the lowest priority number, the higher channel on a tie.
    pub fn choose(&self, ready: [bool; 7]) -> Option<usize> {
        (0..7).filter(|&n| ready[n]).min_by_key(|&n| ((self.dpcr >> (4 * n)) & 7, 7 - n))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn otc_keeps_three_bits_and_reads_backward() {
        let mut d = Dma::default();
        d.write(0x68, 0b01110000_01110111_00000111_00000011);
        assert_eq!(d.read(0x68), 0b01010000_00000000_00000000_00000010, "ps1-tests' testOtcWhichBitsAreHardwiredToZero");
        d.write(0x68, 0);
        assert_eq!(d.read(0x68), 2);
        d.write(0x68, 0b10001110_10001000_11111000_11111100);
        assert_eq!(d.read(0x68), 2);
    }

    #[test]
    fn dicr_flags_acknowledge_and_raise_the_master_flag_once() {
        let mut d = Dma::default();
        assert!(!d.write(0x74, 1 << 23 | 1 << 18));
        assert!(d.flag(2));
        assert!(!d.flag(2), "already up");
        assert_eq!(d.read(0x74) >> 24, 0x84);
        d.write(0x74, 1 << 23 | 1 << 18 | 1 << 26);
        assert_eq!(d.read(0x74) >> 24, 0);
        assert!(!d.flag(3), "channel 3 not enabled");
        assert_eq!(d.read(0x74), 1 << 23 | 1 << 18);
    }

    #[test]
    fn priority_by_dpcr_then_channel() {
        let d = Dma::default();
        assert_eq!(d.choose([true, true, true, false, false, false, true]), Some(0));
        let e = Dma { dpcr: 0x0333_3333, ..Dma::default() };
        assert_eq!(e.choose([true, true, true, false, false, false, false]), Some(2));
    }
}
