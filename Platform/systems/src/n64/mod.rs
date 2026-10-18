//! The N64's pack code: its GameShark codec, the Controller Pak's module and its images' byte order.

pub mod cheats;
pub mod disasm;
pub mod pak;

/// The magic word of a big-endian (.z64) image.
pub const MAGIC: u32 = 0x8037_1240;

/// An N64 image in the halfword-swapped .v64 order whatever order it is in, told by its first word; none where it is in
/// that order already, is no N64 image or is a truncated one (Mars_Rom.md §1.1).
pub fn byte_swapped(file: &[u8]) -> Option<Vec<u8>> {
    if file.len() < 4 {
        return None;
    }
    let bit = match u32::from_be_bytes([file[0], file[1], file[2], file[3]]) {
        MAGIC if file.len() % 2 == 0 => 1,
        0x4012_3780 if file.len() % 4 == 0 => 2,
        _ => return None,
    };
    Some((0..file.len()).map(|i| file[i ^ bit]).collect())
}
