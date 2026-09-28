//! C#'s `Cartridge`: one iNES or NES 2.0 image. See Moon_Memory.md §2.

use std::sync::Arc;

use crate::Skip;
use crate::state::{StateReader, StateResult, StateWriter};

pub const PRG_BANK_SIZE: i32 = 0x4000;
pub const CHR_BANK_SIZE: i32 = 0x2000;
pub const TRAINER_SIZE: usize = 512;
pub const HEADER_SIZE: usize = 16;

/// C#'s `Mirroring` enum, held as its `int` because a loaded state may carry any value.
pub mod mirroring {
    pub const HORIZONTAL: i32 = 0;
    pub const VERTICAL: i32 = 1;
    pub const SINGLE_SCREEN_LOWER: i32 = 2;
    pub const SINGLE_SCREEN_UPPER: i32 = 3;
    pub const FOUR_SCREEN: i32 = 4;
}

/// Why an image was refused, as C#'s `Cartridge` refuses it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RomError {
    NotInes,
    PrgTruncated,
    UnsupportedMapper(i32),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Cartridge {
    pub prg_rom: Skip<Arc<Vec<u8>>>,
    pub chr: Vec<u8>,
    pub prg_ram: Vec<u8>,
    pub chr_is_ram: Skip<bool>,
    pub mapper_number: Skip<i32>,
    pub has_battery: Skip<bool>,
    pub header_mirroring: Skip<i32>,
    pub cpu_cycle: i64,
}

impl Cartridge {
    /// `Cartridge.Parse`: the header, the PRG, and CHR ROM or 8K of CHR RAM; every board gets 8K of PRG RAM.
    pub fn parse(image: &[u8]) -> Result<Cartridge, RomError> {
        if image.len() < HEADER_SIZE || image[0] != b'N' || image[1] != b'E' || image[2] != b'S' || image[3] != 0x1A {
            return Err(RomError::NotInes);
        }
        let nes20 = (image[7] & 0x0C) == 0x08;
        let archaic = !nes20 && ((image[7] & 0x0C) != 0 || image[12] != 0 || image[13] != 0 || image[14] != 0 || image[15] != 0);
        let mut prg_banks = image[4] as i32;
        let mut chr_banks = image[5] as i32;
        let mut mapper = (image[6] >> 4) as i32;
        if !archaic {
            mapper |= (image[7] & 0xF0) as i32;
        }
        if nes20 {
            mapper |= ((image[8] & 0x0F) as i32) << 8;
            prg_banks |= ((image[9] & 0x0F) as i32) << 8;
            chr_banks |= ((image[9] & 0xF0) as i32) << 4;
        }
        let has_battery = (image[6] & 0x02) != 0;
        let has_trainer = (image[6] & 0x04) != 0;
        let header_mirroring = if (image[6] & 0x08) != 0 {
            mirroring::FOUR_SCREEN
        } else if (image[6] & 0x01) != 0 {
            mirroring::VERTICAL
        } else {
            mirroring::HORIZONTAL
        };

        let mut offset = HEADER_SIZE + if has_trainer { TRAINER_SIZE } else { 0 };
        let prg_length = (prg_banks * PRG_BANK_SIZE) as usize;
        if offset + prg_length > image.len() {
            return Err(RomError::PrgTruncated);
        }
        let prg_rom = image[offset..offset + prg_length].to_vec();
        offset += prg_length;

        let (chr, chr_is_ram) = if chr_banks == 0 {
            (vec![0u8; CHR_BANK_SIZE as usize], true)
        } else {
            let chr_length = (chr_banks * CHR_BANK_SIZE) as usize;
            let mut chr = vec![0u8; chr_length];
            let n = chr_length.min(image.len() - offset);
            chr[..n].copy_from_slice(&image[offset..offset + n]);
            (chr, false)
        };

        Ok(Cartridge {
            prg_rom: Skip(Arc::new(prg_rom)),
            chr,
            prg_ram: vec![0u8; 0x2000],
            chr_is_ram: Skip(chr_is_ram),
            mapper_number: Skip(mapper),
            has_battery: Skip(has_battery),
            header_mirroring: Skip(header_mirroring),
            cpu_cycle: 0,
        })
    }

    #[inline(always)]
    pub fn prg_len(&self) -> i32 {
        self.prg_rom.len() as i32
    }

    #[inline(always)]
    pub fn chr_len(&self) -> i32 {
        self.chr.len() as i32
    }

    /// `PrgBanks`: whole 16K banks.
    #[inline(always)]
    pub fn prg_banks(&self) -> i32 {
        self.prg_len() / PRG_BANK_SIZE
    }

    /// `PrgRom[offset % PrgRom.Length]`, C#'s exceptions recorded as faults.
    #[inline(always)]
    pub fn prg_mod(&self, offset: i32) -> u8 {
        crate::at(&self.prg_rom, crate::rem(offset, self.prg_len()))
    }

    /// `Chr[offset % Chr.Length]`.
    #[inline(always)]
    pub fn chr_mod(&self, offset: i32) -> u8 {
        crate::at(&self.chr, crate::rem(offset, self.chr_len()))
    }

    #[inline(always)]
    pub fn prg_ram_at(&self, address: u16) -> u8 {
        self.prg_ram[(address & 0x1FFF) as usize]
    }

    #[inline(always)]
    pub fn set_prg_ram(&mut self, address: u16, data: u8) {
        self.prg_ram[(address & 0x1FFF) as usize] = data;
    }

    /// `if (ChrIsRam) Chr[offset] = data`, the offset already reduced as the board reduces it.
    #[inline(always)]
    pub fn write_chr_ram(&mut self, offset: i32, data: u8) {
        if *self.chr_is_ram {
            crate::put(&mut self.chr, offset, data);
        }
    }

    pub fn write_state(&self, w: &mut StateWriter) {
        w.bytes("Chr", &self.chr);
        w.i64("CpuCycle", self.cpu_cycle);
        w.bytes("PrgRam", &self.prg_ram);
    }

    pub fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        r.bytes(&mut self.chr)?; // Chr
        self.cpu_cycle = r.i64()?; // CpuCycle
        r.bytes(&mut self.prg_ram) // PrgRam
    }
}
