//! What a Game Boy image's header says about its board, with the refusal of a type no board implements
//! (Mercury_Memory.md §2, §4). The refusal is the engine's statement more than the console's, and goes to core info
//! with the shims (EmuSen_Backend.md §13.3).

/// Field for field `GbCartridge`'s.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cartridge {
    pub kind: u8,
    pub board: &'static str,
    pub rom_banks: i32,
    pub ram_length: i32,
    pub battery: bool,
    pub timer: bool,
    pub rumble: bool,
    pub cgb_flag: u8,
}

pub const HEADER_LENGTH: usize = 0x0150;
pub const ROM_BANK: usize = 0x4000;

/// Why a header was refused, as the C# threw: an image too short (InvalidDataException) or a type no board implements
/// (NotSupportedException).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    Invalid(String),
    Unsupported(String),
}

/// MBC2 carries 512 half-bytes on the chip itself and reports no size in the header.
pub fn ram_length(code: u8, kind: u8) -> i32 {
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

/// The header of an image of `length` bytes whose first bytes are `head`.
pub fn read(head: &[u8], length: usize) -> Result<Cartridge, Refusal> {
    if length < HEADER_LENGTH || head.len() < HEADER_LENGTH {
        return Err(Refusal::Invalid(format!("Not a Game Boy image: {length} bytes is shorter than the 336-byte header.")));
    }
    let kind = head[0x0147];
    let timer = matches!(kind, 0x0F | 0x10);
    let rumble = matches!(kind, 0x1C | 0x1D | 0x1E | 0x22);
    let board = match kind {
        0x00 | 0x08 | 0x09 => "ROM",
        0x01..=0x03 => "MBC1",
        0x05 | 0x06 => "MBC2",
        0x0F..=0x13 => if timer { "MBC3+RTC" } else { "MBC3" },
        0x19..=0x1E => if rumble { "MBC5+Rumble" } else { "MBC5" },
        _ => return Err(Refusal::Unsupported(format!("Cartridge type ${kind:02X} is not implemented - see Mercury_Memory.md §4."))),
    };
    let battery = matches!(kind, 0x03 | 0x06 | 0x09 | 0x0D | 0x0F | 0x10 | 0x13 | 0x1B | 0x1E | 0x22 | 0xFF);
    Ok(Cartridge { kind, board, rom_banks: (length / ROM_BANK) as i32, ram_length: ram_length(head[0x0149], kind), battery, timer, rumble, cgb_flag: head[0x0143] })
}
