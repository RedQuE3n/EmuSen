//! The boards of the NES by iNES mapper number, by name: the console's fact of which board a number means. What one
//! engine builds of each, and how it shows a board's registers, is that engine's (EmuSen_Backend.md §13.4).

/// The boards named, by mapper number.
pub const BOARDS: [(u16, &str); 16] = [
    (0, "NROM"),
    (1, "MMC1"),
    (2, "UxROM"),
    (3, "CNROM"),
    (4, "MMC3"),
    (7, "AxROM"),
    (9, "MMC2"),
    (11, "Color Dreams"),
    (64, "RAMBO-1"),
    (65, "Irem H3001"),
    (66, "GxROM"),
    (67, "Sunsoft-3"),
    (68, "Sunsoft-4"),
    (69, "Sunsoft FME-7"),
    (71, "Camerica BF909x"),
    (79, "NINA-003-006"),
];

/// A mapper's board by name; none for a number not named here.
pub fn name(mapper: i32) -> Option<&'static str> {
    BOARDS.iter().find(|&&(n, _)| n as i32 == mapper).map(|&(_, name)| name)
}
