//! The bytes OpenVGDB hashed for each system's games: the image without the header a dump may carry, or in the order
//! its database was made from (EmuSen_Settings_Reference.md §4.39). A system with no rule hashes the file as it is.

/// The bytes of `file` OpenVGDB hashed for `system`; none where they are the file's own.
pub fn bytes(system: &str, file: &[u8]) -> Option<Vec<u8>> {
    match system {
        // The iNES header, which OpenVGDB's NES row skips as its 16-byte header.
        "nes" if file.len() > 16 && file[..4] == *b"NES\x1A" => Some(file[16..].to_vec()),
        // The 512-byte copier header some dumps carry, as No-Intro leaves it out.
        "snes" if file.len() % 1024 == 512 => Some(file[512..].to_vec()),
        "n64" => crate::n64::byte_swapped(file),
        _ => None,
    }
}

/// Whether a system has a rule at all, which the C# entries state as their own delegate.
pub fn transforms(system: &str) -> bool {
    matches!(system, "nes" | "snes" | "n64")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_rule_strips_or_reorders_and_other_files_are_their_own() {
        let mut nes = b"NES\x1A".to_vec();
        nes.extend([0u8; 12]);
        nes.push(7);
        assert_eq!(bytes("nes", &nes), Some(vec![7]));
        assert_eq!(bytes("nes", &nes[..16]), None);
        assert_eq!(bytes("snes", &[1u8; 1536]).map(|b| b.len()), Some(1024));
        assert_eq!(bytes("snes", &[1u8; 1024]), None);
        assert_eq!(bytes("n64", &[0x80, 0x37, 0x12, 0x40]), Some(vec![0x37, 0x80, 0x40, 0x12]));
        assert_eq!(bytes("n64", &[0x40, 0x12, 0x37, 0x80]), Some(vec![0x37, 0x80, 0x40, 0x12]));
        assert_eq!(bytes("n64", &[0x37, 0x80, 0x40, 0x12]), None);
        assert_eq!(bytes("psx", &[1, 2, 3]), None);
    }
}
