//! C#'s `Memory/Cartridge.cs`: one image and the board its header describes. See Mercury_Memory.md §2.

use crate::Skip;
use crate::memory::mappers::Mapper;
use crate::state::{StateReader, StateResult, StateWriter};

pub const ROM_BANK_SIZE: usize = 0x4000;
pub const RAM_BANK_SIZE: usize = 0x2000;
pub const HEADER_START: usize = 0x0134;
pub const HEADER_CHECKSUM_ADDRESS: usize = 0x014D;

/// What the `$0143` flag says about colour support.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CgbSupport {
    #[default]
    None,
    Enhanced,
    Required,
}

/// Why an image is refused, as C#'s `Cartridge.FromImage` refuses it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RomError {
    TooShort(usize),
    UnsupportedType(u8),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Cartridge {
    pub rom: Skip<Vec<u8>>,
    pub ram: Vec<u8>,
    pub title: Skip<String>,
    pub cartridge_type: Skip<u8>,
    pub cgb: Skip<CgbSupport>,
    pub super_game_boy: Skip<bool>,
    pub has_battery: Skip<bool>,
    pub has_timer: Skip<bool>,
    pub has_rumble: Skip<bool>,
    pub header_checksum_valid: Skip<bool>,
    /// C#'s `_savePath`: null until a load or a battery save names it, then a string (Mercury_Native.md §6.1, D3).
    pub save_path: Option<String>,
}

impl Cartridge {
    /// `Cartridge.FromImage`, with the board it builds; the save path is the host's, as `LoadSram` sets it.
    pub fn from_image(image: Vec<u8>, save_path: Option<String>) -> Result<(Cartridge, Mapper), RomError> {
        if image.len() < 0x0150 {
            return Err(RomError::TooShort(image.len()));
        }
        let kind = image[0x0147];
        let cgb = match image[0x0143] {
            0xC0 => CgbSupport::Required,
            0x80 => CgbSupport::Enhanced,
            _ => CgbSupport::None,
        };
        let mapper = Mapper::for_type(kind).ok_or(RomError::UnsupportedType(kind))?;
        let cart = Cartridge {
            title: Skip(read_title(&image, cgb != CgbSupport::None)),
            cartridge_type: Skip(kind),
            cgb: Skip(cgb),
            super_game_boy: Skip(image[0x0146] == 0x03),
            has_battery: Skip(matches!(kind, 0x03 | 0x06 | 0x09 | 0x0D | 0x0F | 0x10 | 0x13 | 0x1B | 0x1E | 0x22 | 0xFF)),
            has_timer: Skip(matches!(kind, 0x0F | 0x10)),
            has_rumble: Skip(matches!(kind, 0x1C | 0x1D | 0x1E | 0x22)),
            ram: vec![0; ram_size_for(image[0x0149], kind)],
            header_checksum_valid: Skip(header_checksum(&image) == image[HEADER_CHECKSUM_ADDRESS]),
            save_path,
            rom: Skip(image),
        };
        Ok((cart, mapper))
    }

    pub fn rom_banks(&self) -> usize {
        self.rom.len() / ROM_BANK_SIZE
    }

    pub fn is_cgb(&self) -> bool {
        *self.cgb != CgbSupport::None
    }

    pub fn write_state(&self, w: &mut StateWriter) {
        w.bytes("Ram", &self.ram);
        w.string("_savePath", self.save_path.as_deref());
    }

    pub fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        r.bytes(&mut self.ram)?; // Ram
        self.save_path = Some(r.string()?); // _savePath
        Ok(())
    }

    /// A class-typed `_cart` field elsewhere in the state: the present flag, then the same two fields.
    pub fn write_as_field(&self, w: &mut StateWriter, name: &str) {
        w.group_class(name, |w| self.write_state(w));
    }

    pub fn read_as_field(&mut self, r: &mut StateReader) -> StateResult {
        if r.bool()? {
            self.read_state(r)?;
        }
        Ok(())
    }
}

/// Trailing NULs and, on a colour cart, the manufacturer and CGB bytes are not part of the name.
fn read_title(image: &[u8], cgb: bool) -> String {
    let length = if cgb { 11 } else { 16 };
    let mut text = String::new();
    for &b in &image[HEADER_START..HEADER_START + length] {
        if b == 0 {
            break;
        }
        text.push(if (0x20..0x7F).contains(&b) { b as char } else { ' ' });
    }
    text.trim_end().to_string()
}

/// `x = x - byte - 1` across `$0134-$014C`.
pub fn header_checksum(image: &[u8]) -> u8 {
    image[HEADER_START..=0x014C].iter().fold(0u8, |sum, &b| sum.wrapping_sub(b).wrapping_sub(1))
}

/// MBC2 carries 512 half-bytes on the chip and reports no size in the header.
fn ram_size_for(code: u8, kind: u8) -> usize {
    if matches!(kind, 0x05 | 0x06) {
        return 512;
    }
    match code {
        0x02 => 8 * 1024,
        0x03 => 32 * 1024,
        0x04 => 128 * 1024,
        0x05 => 64 * 1024,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image(kind: u8, ram_code: u8, cgb: u8) -> Vec<u8> {
        let mut rom = vec![0u8; 0x8000];
        rom[0x0134..0x013B].copy_from_slice(b"WISEMAN");
        rom[0x0143] = cgb;
        rom[0x0147] = kind;
        rom[0x0149] = ram_code;
        rom[HEADER_CHECKSUM_ADDRESS] = header_checksum(&rom);
        rom
    }

    #[test]
    fn the_header_decides_the_board_the_ram_and_the_colour_mode() {
        let (cart, mapper) = Cartridge::from_image(image(0x03, 0x02, 0x00), None).unwrap();
        assert!(matches!(mapper, Mapper::Mbc1(_)));
        assert_eq!((cart.ram.len(), *cart.has_battery, cart.is_cgb(), cart.title.as_str()), (8192, true, false, "WISEMAN"));
        assert!(*cart.header_checksum_valid);
        let (cart, mapper) = Cartridge::from_image(image(0x06, 0x03, 0xC0), None).unwrap();
        assert!(matches!(mapper, Mapper::Mbc2(_)));
        assert_eq!((cart.ram.len(), *cart.cgb), (512, CgbSupport::Required));
        let (cart, mapper) = Cartridge::from_image(image(0x10, 0x03, 0x80), None).unwrap();
        assert!(matches!(mapper, Mapper::Mbc3(_)));
        assert_eq!((cart.ram.len(), *cart.has_timer, *cart.cgb), (32768, true, CgbSupport::Enhanced));
        let (cart, mapper) = Cartridge::from_image(image(0x1E, 0x04, 0x00), None).unwrap();
        assert!(matches!(mapper, Mapper::Mbc5(_)));
        assert_eq!((cart.ram.len(), *cart.has_rumble, *cart.has_battery), (131072, true, true));
        assert!(matches!(Cartridge::from_image(image(0x09, 0x05, 0x00), None).unwrap().1, Mapper::NoMbc(_)));
    }

    #[test]
    fn what_csharp_refuses_is_refused() {
        assert_eq!(Cartridge::from_image(vec![0; 0x14F], None).err(), Some(RomError::TooShort(0x14F)));
        assert_eq!(Cartridge::from_image(image(0x22, 0, 0), None).err(), Some(RomError::UnsupportedType(0x22)));
        assert_eq!(Cartridge::from_image(image(0x04, 0, 0), None).err(), Some(RomError::UnsupportedType(0x04)));
    }
}
