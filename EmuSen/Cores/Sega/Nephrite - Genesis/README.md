# Nephrite (Sega Genesis / Mega Drive, with the Sega CD and the 32X)

The Genesis core in Rust, crate `nephrite`, on the core ABI v1. The Sega CD and the 32X are attachments of the same
machine, reported as the systems `md`, `mcd` and `32x`. Its CPUs are the Beryl crates in `../Beryl - Shared CPUs/`.

At stage 0 (2026-10-04) it recognises an image, reports its system, region and memories, and shows a blank picture;
nothing is emulated yet. The plan is `Man pages/Hardware/Sega/Nephrite - Genesis/Nephrite_Plan.md`, the build record
`Nephrite_Native.md` beside it.

The crate is named plainly, without the `RT` suffix of the Rust ports (MoonRT, VenusRT), because it has no C#
predecessor to be told from (`EmuSen_Core_Naming_Scheme.md` §3).
