//! The OBC1, from fullsnes ("SNES Cart OBC1"): 8 KiB of SRAM at 6000h-7FFFh in banks 00-3F and 80-BF, with eight
//! registers at 7FF0h-7FF7h giving an OAM-shaped view of a 220h-byte workspace in it. See VenusRT_Native.md §27.

#[derive(Clone, Debug, Default)]
pub struct Obc1 {
    /// 7FF0h-7FF7h as last written.
    pub regs: [u8; 8],
}

impl Obc1 {
    /// Whether an S-CPU address is the chip's: 6000h-7FFFh in banks 00-3F and 80-BF.
    pub fn snes_maps(address: u32) -> bool {
        (address >> 16) as u8 & 0x40 == 0 && matches!(address as u16, 0x6000..=0x7FFF)
    }

    /// The workspace's base in the SRAM: 7C00h, or 7800h when 7FF5h's bit 0 is set.
    fn base(&self) -> usize {
        if self.regs[5] & 1 != 0 { 0x1800 } else { 0x1C00 }
    }

    fn index(&self) -> usize {
        (self.regs[6] & 0x7F) as usize
    }

    /// The SRAM byte a register stands for: 7FF0h-7FF3h the OBJ's four bytes, 7FF4h the byte of its two high bits.
    fn slot(&self, reg: usize) -> usize {
        match reg {
            0..=3 => self.base() + self.index() * 4 + reg,
            _ => self.base() + 0x200 + self.index() / 4,
        }
    }

    pub fn read(&self, offset: u16, sram: &[u8]) -> Option<u8> {
        if sram.is_empty() {
            return None;
        }
        let n = sram.len();
        Some(match offset {
            // fullsnes: a read of 7FF4h returns the whole byte, the two bits not isolated.
            0x1FF0..=0x1FF4 => sram[self.slot((offset - 0x1FF0) as usize) % n],
            0x1FF5..=0x1FF7 => self.regs[(offset - 0x1FF0) as usize],
            _ => sram[offset as usize % n],
        })
    }

    pub fn write(&mut self, offset: u16, v: u8, sram: &mut [u8]) {
        if sram.is_empty() {
            return;
        }
        let n = sram.len();
        match offset {
            0x1FF0..=0x1FF3 => {
                self.regs[(offset - 0x1FF0) as usize] = v;
                sram[self.slot((offset - 0x1FF0) as usize) % n] = v;
            }
            0x1FF4 => {
                self.regs[4] = v;
                let at = self.slot(4) % n;
                let shift = (self.index() & 3) * 2;
                sram[at] = (sram[at] & !(3 << shift)) | ((v & 3) << shift);
            }
            0x1FF5..=0x1FF7 => self.regs[(offset - 0x1FF0) as usize] = v,
            _ => sram[offset as usize % n] = v,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // fullsnes's ports: 7FF0h-7FF3h reach OBJ Index's four bytes, 7FF4h its two bits in the high table.
    #[test]
    fn the_registers_reach_the_workspace() {
        let mut o = Obc1::default();
        let mut sram = vec![0u8; 0x2000];
        o.write(0x1FF5, 0, &mut sram);
        o.write(0x1FF6, 5, &mut sram);
        o.write(0x1FF1, 0x42, &mut sram);
        o.write(0x1FF4, 0x03, &mut sram);
        assert_eq!(sram[0x1C00 + 5 * 4 + 1], 0x42);
        assert_eq!(sram[0x1C00 + 0x200 + 1], 0x03 << 2);
        assert_eq!(o.read(0x1FF1, &sram), Some(0x42));
        assert_eq!(o.read(0x0010, &sram), Some(0));
    }
}
