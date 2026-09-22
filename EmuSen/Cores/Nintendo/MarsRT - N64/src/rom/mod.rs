//! A cartridge image normalised to big-endian with its header read, its CIC, and its declared save chip: C#'s `RomImage`, `Cic` and `SaveTypes`. See Mars_Rom.md.

use crate::memory::save::save_type;

pub const MAGIC: u32 = 0x8037_1240;
pub const HEADER_LENGTH: usize = 0x40;
pub const BOOT_CODE_LENGTH: usize = 0xFC0;
pub const MINIMUM_LENGTH: usize = HEADER_LENGTH + BOOT_CODE_LENGTH;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cic {
    Unknown,
    Nus6101,
    Nus6102,
    Nus6103,
    Nus6105,
    Nus6106,
}

/// Why an image was refused, as `RomImage.FromImage` refuses it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RomError {
    TooShort(usize),
    NotAnImage(u32),
    Truncated(usize),
}

#[derive(Clone, Debug)]
pub struct RomImage {
    pub rom: Vec<u8>,
    pub entry_point: u32,
    pub crc1: u32,
    pub crc2: u32,
    pub destination: u8,
    pub version: u8,
    pub is_pal: bool,
    pub cic: Cic,
    pub has_ed64_header: bool,
    /// `SaveType`: what the ED64 convention declares, as an `N64SaveType`.
    pub save_type: i32,
}

fn read_u32(bytes: &[u8], at: usize) -> u32 {
    u32::from_be_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}

impl RomImage {
    /// `FromImage`: the container order is detected from the first word, never the extension.
    pub fn from_image(file: &[u8]) -> Result<RomImage, RomError> {
        if file.len() < 4 {
            return Err(RomError::TooShort(file.len()));
        }
        let rom = match read_u32(file, 0) {
            MAGIC => file.to_vec(),
            0x3780_4012 => {
                if !file.len().is_multiple_of(2) {
                    return Err(RomError::Truncated(file.len()));
                }
                file.as_chunks::<2>().0.iter().flat_map(|p| [p[1], p[0]]).collect()
            }
            0x4012_3780 => {
                if !file.len().is_multiple_of(4) {
                    return Err(RomError::Truncated(file.len()));
                }
                file.as_chunks::<4>().0.iter().flat_map(|p| [p[3], p[2], p[1], p[0]]).collect()
            }
            word => return Err(RomError::NotAnImage(word)),
        };
        if rom.len() < MINIMUM_LENGTH {
            return Err(RomError::TooShort(rom.len()));
        }
        let destination = rom[0x3E];
        let version = rom[0x3F];
        let has_ed64_header = rom[0x3C] == b'E' && rom[0x3D] == b'D';
        let save_type = if !has_ed64_header {
            save_type::UNKNOWN
        } else {
            match version >> 4 {
                0 => save_type::NONE,
                1 => save_type::EEPROM_4K,
                2 => save_type::EEPROM_16K,
                3 => save_type::SRAM_256K,
                4 => save_type::SRAM_BANKED_768K,
                5 => save_type::FLASH_RAM,
                6 => save_type::SRAM_1M,
                _ => save_type::UNKNOWN,
            }
        };
        Ok(RomImage {
            entry_point: read_u32(&rom, 0x08),
            crc1: read_u32(&rom, 0x10),
            crc2: read_u32(&rom, 0x14),
            destination,
            version,
            is_pal: matches!(destination, b'D' | b'F' | b'I' | b'P' | b'S' | b'U' | b'X' | b'Y'),
            cic: identify(&rom),
            has_ed64_header,
            save_type,
            rom,
        })
    }

    /// `SaveTypes.Declared`: the image's own word first, then the table.
    pub fn declared_save_type(&self) -> i32 {
        if self.save_type != save_type::UNKNOWN {
            return self.save_type;
        }
        if EEPROM_16K_TITLES.contains(&(self.crc1, self.crc2)) { save_type::EEPROM_16K } else { save_type::UNKNOWN }
    }
}

/// `Cic.Identify`: every word of IPL3 summed as a 64-bit total.
pub fn identify(rom: &[u8]) -> Cic {
    if rom.len() < MINIMUM_LENGTH {
        return Cic::Unknown;
    }
    let mut sum: u64 = 0;
    for i in (HEADER_LENGTH..MINIMUM_LENGTH).step_by(4) {
        sum = sum.wrapping_add(read_u32(rom, i) as u64);
    }
    match sum {
        0x0000_00D0_027F_DF31 | 0x0000_00CF_FB63_1223 => Cic::Nus6101,
        0x0000_00D0_57C8_5244 | 0x0000_007C_5624_2373 => Cic::Nus6102,
        0x0000_00D6_497E_414B => Cic::Nus6103,
        0x0000_011A_49F6_0E96 => Cic::Nus6105,
        0x0000_00D6_D5BE_5580 => Cic::Nus6106,
        _ => Cic::Unknown,
    }
}

/// `Cic.Seed`: an unknown chip gets the 6102's.
pub fn seed(chip: Cic) -> u8 {
    match chip {
        Cic::Nus6103 => 0x78,
        Cic::Nus6105 => 0x91,
        Cic::Nus6106 => 0x85,
        _ => 0x3F,
    }
}

const LUT: [u8; 32] = [
    0x4, 0x7, 0xA, 0x7, 0xE, 0x5, 0xE, 0x1, 0xC, 0xF, 0x8, 0xF, 0x6, 0x3, 0x6, 0x9, 0x4, 0x1, 0xA, 0x7, 0xE, 0x5, 0xE, 0x1, 0xC, 0x9, 0x8, 0x5, 0x6,
    0x3, 0xC, 0x9,
];

/// `Cic.Respond`: the 6105's reply, a nibble at a time from the high one.
pub fn respond(challenge: &[u8], response: &mut [u8]) {
    let mut key = 0xBi32;
    let mut mode = 0i32;
    for (i, &both) in challenge.iter().enumerate() {
        let high = step((both >> 4) as i32, &mut key, &mut mode);
        let low = step((both & 0xF) as i32, &mut key, &mut mode);
        response[i] = ((high << 4) | low) as u8;
    }
}

fn step(nibble: i32, key: &mut i32, mode: &mut i32) -> i32 {
    let result = (*key + 5 * nibble) & 0xF;
    let sign = result >> 3;
    let magnitude = (if sign == 1 { !result } else { result }) & 0x7;
    *key = LUT[((*mode << 4) | result) as usize] as i32;
    let mut next = if magnitude % 3 == 1 { sign } else { 1 - sign };
    if *mode == 1 && (result == 0x1 || result == 0x9) {
        next = 1;
    }
    if *mode == 1 && (result == 0xB || result == 0xE) {
        next = 0;
    }
    *mode = next;
    result
}

/// `SaveTypes.Titles`: the rows mupen64plus and Project64 both give as 16 Kbit EEPROM, by the header's two checksums.
const EEPROM_16K_TITLES: [(u32, u32); 54] = [
    (0xB695_1A94, 0x63C8_49AF),
    (0xA553_3106, 0xB9F2_5E5B),
    (0x514B_6900, 0xB4B1_9881),
    (0x155B_7CDF, 0xF0DA_7325),
    (0xC917_6D39, 0xEA47_79D1),
    (0xC2E9_AA9A, 0x475D_70AA),
    (0xF800_9DB0, 0x6B29_1823),
    (0x373F_5889, 0x9A6C_A80A),
    (0x30C7_AC50, 0x7704_072D),
    (0x83F3_931E, 0xCB72_223D),
    (0xDFE6_1153, 0xD761_18E6),
    (0x0795_01B9, 0xAB02_32AB),
    (0x17C5_4A61, 0x4A83_F2E7),
    (0x1193_6D8C, 0x6F2C_4B43),
    (0x053C_89A7, 0xA506_4302),
    (0x0DD4_ABAB, 0xB5A2_A91E),
    (0xEC58_EABF, 0xAD7C_7169),
    (0xB630_6E99, 0xB63E_D2B2),
    (0xA827_5140, 0xB9B0_56E8),
    (0x202A_8EE4, 0x83F8_8B89),
    (0x861C_3519, 0xF609_1CE5),
    (0xAF75_4F7B, 0x1DD1_7381),
    (0x0786_1842, 0xA12E_BC9F),
    (0xF9D4_11E3, 0x7CB2_9BC0),
    (0xEE4A_0E33, 0x8FD5_88C9),
    (0xC49A_DCA2, 0xF150_1B62),
    (0x0C58_1C7A, 0x3D6E_20E4),
    (0xBCB1_F89F, 0x0607_52A2),
    (0x77DA_3B8D, 0x162B_0D7C),
    (0x0D93_BA11, 0x6838_68A6),
    (0x4603_9FB4, 0x0337_822C),
    (0x1739_EFBA, 0xD0B4_3A68),
    (0xC567_4160, 0x0F5F_453C),
    (0x0B0A_B4CD, 0x7B15_8937),
    (0x7C38_29D9, 0x6E82_47CE),
    (0x839F_3AD5, 0x406D_15FA),
    (0x5001_CF4F, 0xF30C_B3BD),
    (0x3A6C_42B5, 0x1ACA_DA1B),
    (0x147E_0EDB, 0x36C5_B12C),
    (0xF468_118C, 0xE32E_E44E),
    (0xCFE2_CB31, 0x4D6B_1E1D),
    (0xE4B0_8007, 0xA602_FF33),
    (0x9674_7EB4, 0x104B_B243),
    (0xDDF4_60CC, 0x3CA6_34C0),
    (0x41F2_B98F, 0xB458_B466),
    (0xFEE9_7010, 0x4E94_A9A0),
    (0x2500_267E, 0x2A7E_C3CE),
    (0x272B_690F, 0xAD0A_7A77),
    (0x53ED_2DC4, 0x0625_8002),
    (0x61F5_B152, 0x0461_22AB),
    (0x72F7_0398, 0x6556_A98B),
    (0x2DCF_CA60, 0x8354_B147),
    (0xD3F9_7D49, 0x6924_135B),
    (0x2337_D8E8, 0x6B8E_7CEC),
];
