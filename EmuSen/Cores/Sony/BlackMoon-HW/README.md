# BlackMoon Hardware (the shared hardware of the Sony cores)

Not a console: one Rust crate per chip that more than one Sony core can use, so that each can be used alone. The
first are the PlayStation's processors.

- `blackmoon-r3000` — the R3000A, the MIPS I processor of the PlayStation's CPU, and later the PlayStation 2's I/O
  processor (Sapphire).
- `blackmoon-gte` — the GTE, the PlayStation's geometry coprocessor on the R3000A's coprocessor 2.

Named for the Black Moon Clan, the Sony cores' faction, as Beryl Hardware is for the Sega cores' (decided 2026-10-11,
`Diamond_Plan.md` §11 Q10; `EmuSen_Core_Naming_Scheme.md` §4). The interface and each crate's record are in
`Man pages/Hardware/Sony/BlackMoon-HW/`.
