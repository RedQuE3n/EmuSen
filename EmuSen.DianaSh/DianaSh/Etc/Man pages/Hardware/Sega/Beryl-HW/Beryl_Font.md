# Beryl_Font — the open firmware's own font

*Created 2026-10-19, at Endou's stage 6 (`Endou_OpenBoot.md` §3).* `beryl-font` holds one thing: the 47 glyphs of five
dots by seven that Nephrite's open Sega CD BIOS was drawn with (`Nephrite_OpenBios.md` §8), as `src/font.txt`, and a
reader of them. It became a crate when a second core's firmware needed the same letters: Endou's open boot program
writes its words in them. One owner of the glyphs; each core's `build.rs` makes its own console's tiles from them, the
Genesis's packed four-bit pixels and the Master System's four bitplanes, so nothing in the crate knows a console.

- **The surface**: `SOURCE`, the text; `glyphs()`, each glyph as its character and seven rows of five dots, the
  leftmost dot the row's bit 4. A build dependency only: no core's library links it, as no core's library links the
  assemblers.
- **What it holds** [M 2026-10-19]: the space, the digits, the capitals and `. , - : ! ? / ( ) '`, each once, held by
  the crate's one test.
- **The move changed nothing Nephrite builds**: its open BIOS, built before and after Nephrite's `build.rs` read the
  glyphs from the crate, is the same file, SHA-256 `fb4a18771b3d6ef73565576bbe9eb329751d63d548cc9dced88e18462b2efcb9`
  [M 2026-10-19].
- **Provenance**: drawn for the project for Nephrite's stage 10 (2026-10-15), from no other font; its glyph shapes are
  the plainest five-by-seven forms of the letters.
