# EmuSen — Games Tested

*(Living document — tracks real-game compatibility testing across EmuSen's cores, separate from the CPU/PPU ground-truth test suites in `EmuSen.Validation`. Those prove individual opcodes/registers match hardware in isolation; this tracks what actually happens when a real, complete commercial ROM is played.)*

---

## 1. Rating scale

Five categories, most to least playable:

- **Perfect** — no known issues. Plays exactly as expected, start to finish.
- **Good** — mild issues. Small, cosmetic, or easily-worked-around problems that don't affect actually playing the game.
- **Fair** — moderate issues. Noticeable, real bugs (visual glitches, incorrect behavior in specific scenes/mechanics) that don't block progress but are clearly wrong.
- **Bad** — severe issues. Bugs serious enough to meaningfully interfere with playing normally (frequent crashes, broken core mechanics, major visual corruption).
- **Unplayable** — doesn't boot, crashes immediately/consistently, or is broken badly enough that it can't be played at all.

A game's category reflects its *current* state, not a permanent verdict — entries move up as bugs get fixed (or down if a fix regresses something else). Each entry links to the relevant hardware doc section(s) for the technical detail behind the rating, rather than duplicating it here.

---

## 2. Venus (SNES)

### Fair

- **The Legend of Zelda: A Link to the Past** — real, reproducible bugs found during testing, none of them blocking progress:
  - Stuck overworld subscreen ("yellow bar"): closing an item-get dialog (e.g. the Lamp chest) can leave WRAM `$1D`/`TS` stuck enabling BG1 on the subscreen outdoors, permanently color-math-blending a torch overlay tile into the overworld as a solid yellow band. Root cause and investigation history (including two reverted fix attempts) in `Hardware/Nintendo/Venus - SNES/Venus_PPU.md` §12.1. Still open, unpatched.
  - Missing rain overlay: the overworld storm-intro rain doesn't always render on a fresh save, and on other saves can appear briefly then disappear while walking. Root-caused as far as BG3's raindrop tilemap not being consistently populated/maintained in VRAM. Not yet root-caused further. Details in `Venus_PPU.md` §12.2.
  - (Fixed, no longer an issue as of this testing pass) Picking up the Lamp from a chest showed the "You got the Lamp!" dialog but the item never entered permanent inventory — root-caused to a 65816 CPU emulation bug (opcode `0x87`, `STA [dp]`, wired to the wrong addressing-mode function), fixed and verified against ground-truth CPU test vectors.

### Unplayable

- **Super Metroid** — hangs before reaching the title screen; never gets further than the Nintendo logo boot splash. Root cause not found despite extensive investigation - full writeup, everything ruled out so far, and the current best lead in `Hardware/Nintendo/Venus - SNES/Venus_APU.md` §2.7. Two real, independently-useful bugs were found and fixed while chasing this (general-DMA cycle accounting, `Venus_Memory.md` §3.1; two SPC700 opcode bugs, `Venus_APU.md` §2.4/§2.5) but neither turned out to be the cause - confirmed via direct before/after comparison at the exact hang point.

---

## 3. Notes on methodology

- Testing uses `EmuSen.HeadlessDebug` (scripted, reproducible: save states, `--tap` input sequences, screenshot capture, per-pixel/register diagnostics) plus live interactive play via `EmuSen.RaylibFrontend` for anything that needs real-time feedback (precise movement/navigation, timing-sensitive input) that scripted input can't easily reproduce.
- A bug found and fixed gets *removed* from a game's list of open issues, not just annotated — this document should always reflect current, real behavior, checked against the actual running build, not a historical changelog. (Changelog-style history belongs in commit messages and the hardware docs' own investigation writeups, not here.)
- "No known issues" (Perfect) means no issues found *in what's been tested*, not a formal claim of full-game coverage — most games here have only had specific scenes/mechanics exercised, not a full playthrough.
