//! The CDC, Sanyo's LC8951, as the Sega CD wires it: its sixteen registers through `$FF8004`-`$FF8007`, its decoder
//! writing each data sector the drive reads into its 16 KiB buffer, and its host transfer to either CPU's host data
//! register or by DMA to PRG-RAM, Word RAM or the PCM chip's memory. Sanyo's LC8950/LC8951 design manual §10-§12 and
//! appendix E, and the Mega-CD Hardware Manual §3-2 and §4-2, are the sources; mcd-verificator's CDC tests settled
//! what they leave open (Nephrite_Native.md §52, D-34).

/// The buffer's size, and where its addresses wrap.
pub const BUFFER: usize = 0x4000;
/// The decoder interrupt stays set for this part of a sector after it is raised, unless STAT3 is read first
/// (mcd-verificator's CDC FLAGS 0x40-0x41: about 40% of the period).
pub const DECI_LENGTH: u64 = 67_000;
/// Sub clocks between the host transfer's words by DMA (argued; D-34).
pub const DMA_WORD: u64 = 16;

/// The device destinations, DD2-DD0.
pub mod dest {
    pub const MAIN: u8 = 2;
    pub const SUB: u8 = 3;
    pub const PCM: u8 = 4;
    pub const PRG: u8 = 5;
    pub const WORD: u8 = 7;
}

/// IFCTRL's bits.
const DOUTEN: u8 = 0x02;
const DECIEN: u8 = 0x20;
const DTEIEN: u8 = 0x40;
/// CTRL0's.
const DECEN: u8 = 0x80;
const WRRQ: u8 = 0x04;
/// CTRL1's SHDREN: the header registers show a mode 2 block's subheader.
const SHDREN: u8 = 0x01;

#[derive(Clone)]
pub struct Cdc {
    pub buffer: Vec<u8>,
    /// The register address, its fifth bit RS: the gate array's own, which reaches no register of the chip.
    pub ar: u8,
    pub ifctrl: u8,
    pub ctrl: [u8; 2],
    /// The byte counter, the data address counter, the write address and the block pointer.
    pub dbc: u16,
    pub dac: u16,
    pub wa: u16,
    pub pt: u16,
    pub head: [u8; 4],
    pub stat: [u8; 4],
    /// The interrupts as set (the chip's register reads them inverted), and the sub clock the decoder's clears itself.
    pub dtei: bool,
    pub deci: bool,
    pub deci_until: u64,
    /// The INT pin as last seen, for its edges.
    pub pin: bool,
    /// The gate array's side: the destination, End of Data Transfer, Data Set Ready, the host data register and the
    /// DMA address in bytes.
    pub dd: u8,
    pub edt: bool,
    pub dsr: bool,
    pub host: u16,
    pub dma: u32,
    /// A transfer under way, and the sub clock of its next DMA word.
    pub busy: bool,
    pub dma_next: u64,
}

impl Default for Cdc {
    fn default() -> Cdc {
        // At power-on the transfer end interrupt reads set (CDC REGS 0x0B reads DBCH's top nibble as 0).
        Cdc { buffer: vec![0; BUFFER], ar: 0, ifctrl: 0, ctrl: [0; 2], dbc: 0, dac: 0, wa: 0, pt: 0, head: [0; 4], stat: [0; 4], dtei: true, deci: false, deci_until: 0, pin: false, dd: 0, edt: false, dsr: false, host: 0, dma: 0, busy: false, dma_next: u64::MAX }
    }
}

impl Cdc {
    /// The INT pin: an interrupt set whose enable is set. True when it has just fallen, which requests level 5.
    pub fn edge(&mut self) -> bool {
        let pin = (self.dtei && self.ifctrl & DTEIEN != 0) || (self.deci && self.ifctrl & DECIEN != 0);
        let fell = pin && !self.pin;
        self.pin = pin;
        fell
    }

    fn reset(&mut self) {
        (self.ifctrl, self.ctrl, self.stat, self.dtei, self.deci, self.busy) = (0, [0; 2], [0; 4], false, false, false);
        self.dma_next = u64::MAX;
    }

    // ---- `$FF8004`-`$FF8007`

    /// `$FF8004` as a word: EDT, DSR, DD; the register address on the sub CPU's side alone.
    pub fn mode(&self, sub: bool) -> u16 {
        (self.edt as u16) << 15 | (self.dsr as u16) << 14 | (self.dd as u16) << 8 | if sub { self.ar as u16 } else { 0 }
    }

    /// A write of the destination ends the transfer's flags and sets its address to 0; the transfer itself goes on.
    pub fn write_mode(&mut self, v: u16, mask: u16) {
        if mask & 0xFF00 != 0 {
            (self.dd, self.dma, self.edt, self.dsr) = ((v >> 8) as u8 & 7, 0, false, false);
        }
        if mask & 0x00FF != 0 {
            self.ar = v as u8 & 0x1F;
        }
    }

    /// The register address after an access: up by one but at R0, from 31 back to 0.
    fn advance(&mut self) {
        if self.ar != 0 {
            self.ar = (self.ar + 1) & 0x1F;
        }
    }

    pub fn read_reg(&mut self, now: u64) -> u8 {
        let r = self.ar;
        self.advance();
        if r >= 16 {
            return 0xFF;
        }
        match r {
            0 => 0xFF,
            1 => {
                let idle = !self.busy as u8;
                0x80 | (!self.dtei as u8) << 6 | (!self.deci as u8) << 5 | 0x10 | idle << 3 | 0x04 | idle << 1 | 1
            }
            2 => self.dbc as u8,
            3 => (if self.dtei { 0 } else { 0xF0 }) | (self.dbc >> 8) as u8 & 15,
            4..=7 => self.head[r as usize - 4],
            8 => self.pt as u8,
            9 => (self.pt >> 8) as u8,
            10 => self.wa as u8,
            11 => (self.wa >> 8) as u8,
            12..=14 => self.stat[r as usize - 12],
            _ => {
                // STAT3 clears the decoder interrupt; VALST reads 0 while it is set.
                let valid = self.deci && now < self.deci_until;
                self.deci = false;
                (!valid as u8) << 7 | self.stat[3] & 0x7F
            }
        }
    }

    /// A register written; DTTRG returns true when it starts a transfer.
    pub fn write_reg(&mut self, v: u8, now: u64) {
        let r = self.ar;
        self.advance();
        if r >= 16 {
            return;
        }
        match r {
            1 => {
                self.ifctrl = v;
                if v & DOUTEN == 0 {
                    self.busy = false;
                    self.dma_next = u64::MAX;
                }
            }
            2 => self.dbc = (self.dbc & 0xF00) | v as u16,
            3 => self.dbc = (self.dbc & 0xFF) | (v as u16 & 15) << 8,
            4 => self.dac = (self.dac & 0xFF00) | v as u16,
            5 => self.dac = (self.dac & 0xFF) | (v as u16) << 8,
            6 => self.trigger(now),
            7 => self.dtei = false,
            8 => self.wa = (self.wa & 0xFF00) | v as u16,
            9 => self.wa = (self.wa & 0xFF) | (v as u16) << 8,
            10 => {
                self.ctrl[0] = v;
                if v & DECEN == 0 {
                    self.deci = false;
                }
            }
            11 => self.ctrl[1] = v,
            12 => self.pt = (self.pt & 0xFF00) | v as u16,
            13 => self.pt = (self.pt & 0xFF) | (v as u16) << 8,
            15 => self.reset(),
            _ => {}
        }
    }

    /// DTTRG: a transfer of `dbc + 1` bytes from `dac`, when DOUTEN allows it.
    fn trigger(&mut self, now: u64) {
        if self.ifctrl & DOUTEN == 0 {
            return;
        }
        (self.busy, self.edt, self.dsr) = (true, false, false);
        match self.dd {
            dest::MAIN | dest::SUB => self.load(),
            dest::PCM | dest::PRG | dest::WORD => self.dma_next = now + DMA_WORD,
            _ => self.busy = false,
        }
    }

    fn take(&mut self) -> u8 {
        let b = self.buffer[self.dac as usize & (BUFFER - 1)];
        self.dac = self.dac.wrapping_add(1);
        b
    }

    /// The transfer's last word or byte taken: EDT, and the transfer end interrupt.
    fn finish(&mut self) {
        (self.busy, self.edt, self.dtei) = (false, true, true);
        self.dma_next = u64::MAX;
    }

    /// The next word into the host data register, the counter counting it; the last sets EDT.
    fn load(&mut self) {
        self.host = (self.take() as u16) << 8 | self.take() as u16;
        self.dsr = true;
        // A word a count of two bytes: an even count's last byte is not sent (DMA2's odd lengths).
        if self.dbc < 3 {
            self.dbc = 0xFFF;
            self.finish();
        } else {
            self.dbc -= 2;
        }
    }

    /// `$A12008` or `$FF8008` read or written by `main` (true) or the sub CPU: the side the transfer is for takes the
    /// word, and the next follows; the other side reads the register as it stands.
    pub fn host_access(&mut self, main: bool) -> u16 {
        let w = self.host;
        let mine = self.dd == if main { dest::MAIN } else { dest::SUB };
        if mine && self.dsr {
            if self.busy {
                self.load();
            } else {
                self.dsr = false;
            }
        }
        w
    }

    /// The DMA's next unit from the buffer: a word, or a byte for the PCM chip, with the bus address it goes to; None
    /// when none is due by `now`.
    pub fn dma_due(&self, now: u64) -> bool {
        self.busy && self.dma_next <= now
    }

    /// The bytes of the next DMA unit, the counter counting them; the caller writes them and then calls `dma_done`.
    pub fn dma_unit(&mut self) -> (u8, Option<u8>) {
        if self.dd == dest::PCM {
            (self.take(), None)
        } else {
            (self.take(), Some(self.take()))
        }
    }

    /// The unit written: the address moved on, the counter down, the next unit timed or the transfer ended.
    pub fn dma_done(&mut self, bytes: u32) {
        self.dma += if self.dd == dest::PCM { 2 * bytes } else { bytes };
        let n = if self.dd == dest::PCM { 1 } else { 2 };
        if self.dbc < 2 * n - 1 {
            self.dbc = 0xFFF;
            self.finish();
        } else {
            self.dbc -= n;
            self.dma_next += DMA_WORD;
        }
    }

    /// A DMA unit that cannot be written now: tried again a word's time later.
    pub fn dma_wait(&mut self) {
        self.dma_next += DMA_WORD;
    }

    /// A sector's period begins: with the decoder on, the decoder interrupt; and a data sector `raw` from the drive
    /// written into the buffer when WRRQ asks, the header and block pointer taken from it.
    pub fn sector(&mut self, raw: Option<&[u8; 2352]>, now: u64) {
        if self.ctrl[0] & DECEN == 0 {
            return;
        }
        if let Some(raw) = raw {
            // The header registers and the status follow every block decoded; with WRRQ clear (Sanyo's
            // "monitor-only mode", §12.5.3) nothing is written to the buffer and the pointers stay.
            let at = if self.ctrl[1] & SHDREN != 0 { 16 } else { 12 };
            self.head.copy_from_slice(&raw[at..at + 4]);
            let mode2 = raw[15] == 2;
            // CRCOK, and STAT2's mode bit; no errors to report from an image.
            self.stat = [0x80, 0, (mode2 as u8) << 3, 0];
            if self.ctrl[0] & WRRQ != 0 {
                for (i, &b) in raw.iter().enumerate() {
                    self.buffer[(self.wa as usize + i) & (BUFFER - 1)] = b;
                }
                self.pt = self.wa.wrapping_add(12) & (BUFFER as u16 - 1);
                self.wa = self.wa.wrapping_add(2352) & (BUFFER as u16 - 1);
            }
        }
        self.deci = true;
        self.deci_until = now + DECI_LENGTH;
    }

    /// The decoder interrupt clearing itself part-way through the sector.
    pub fn clear_deci(&mut self, now: u64) {
        if self.deci && now >= self.deci_until {
            self.deci = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_wrrq_clear_the_header_is_decoded_and_nothing_written() {
        let mut c = Cdc { ctrl: [DECEN, 0], wa: 0x100, ..Cdc::default() };
        let mut raw = [0x55u8; 2352];
        raw[12..20].copy_from_slice(&[0, 2, 0, 1, 9, 8, 7, 6]);
        c.sector(Some(&raw), 0);
        assert_eq!((c.head, c.wa, c.buffer[0x100], c.deci), ([0, 2, 0, 1], 0x100, 0, true), "Sanyo's monitor-only mode");
        c.ctrl[1] = SHDREN;
        c.sector(Some(&raw), 0);
        assert_eq!(c.head, [9, 8, 7, 6], "the subheader");
    }

    #[test]
    fn the_address_counts_up_from_one_and_stops_at_zero() {
        let mut c = Cdc { ar: 1, ..Cdc::default() };
        for i in 2..=31 {
            c.read_reg(0);
            assert_eq!(c.ar, i);
        }
        c.read_reg(0);
        assert_eq!(c.ar, 0);
        c.read_reg(0);
        assert_eq!(c.ar, 0);
    }

    #[test]
    fn a_host_transfer_raises_dsr_then_edt() {
        let mut c = Cdc { ifctrl: DOUTEN | DTEIEN, dd: dest::MAIN, dbc: 7, dtei: false, ..Cdc::default() };
        c.buffer[..8].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        c.ar = 6;
        c.write_reg(0, 0);
        assert!(c.dsr && !c.edt);
        assert_eq!(c.host_access(true), 0x0102);
        assert_eq!(c.host_access(true), 0x0304);
        assert!(c.dsr && !c.edt, "two words left");
        assert_eq!(c.host_access(true), 0x0506);
        assert!(c.dsr && c.edt && c.dtei, "the last word loaded");
        assert!(c.edge(), "the transfer end interrupt falls");
        assert_eq!(c.host_access(true), 0x0708);
        assert!(!c.dsr && c.edt);
    }
}
