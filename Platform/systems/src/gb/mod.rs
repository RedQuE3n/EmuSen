//! The Game Boy family's pack code: its two cheat formats, its cartridge header reader, its disassembler and its views.

pub mod cheats;
pub mod disasm;
pub mod cartridge;
pub mod views;

/// Whether a Game Boy game shelves as the Game Boy Color: a `.gbc`, or a file whose header byte at 0x143 asks for the
/// Color (0x80 enhanced, 0xC0 only), as Mercury reads it. `byte_0x143` is none for a file too short to hold it. The
/// C# `CoreCatalog.IsGameBoyColor` is the twin until B7 (EmuSen_Backend.md §28.2).
pub fn shelved_as_color(extension: &str, byte_0x143: Option<u8>) -> bool {
    emusen_dotnet::text::equals_ignore_case(extension, ".gbc") || matches!(byte_0x143, Some(0x80 | 0xC0))
}

#[cfg(test)]
mod shelf_tests {
    #[test]
    fn a_gbc_or_a_colour_header_shelves_as_the_color() {
        assert!(super::shelved_as_color(".GBC", None));
        assert!(super::shelved_as_color(".gb", Some(0x80)));
        assert!(super::shelved_as_color(".gb", Some(0xC0)));
        assert!(!super::shelved_as_color(".gb", Some(0x00)));
        assert!(!super::shelved_as_color(".gb", None));
    }
}
