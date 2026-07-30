# EmuSen — Games Tested

*(Living document — tracks real-game compatibility testing across EmuSen's cores, separate from the CPU/PPU ground-truth test suites in `EmuSen.Tomoe`. Those prove individual opcodes/registers match hardware in isolation; this tracks what actually happens when a real, complete commercial ROM is played.)*

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

- **Super Mario World** — coins and Yoshi don't render. Long investigation, substantially narrowed to whatever's supposed to write real graphics data into the `$7E8000-$7E97FF` WRAM staging buffer before DMA copies it out not doing so, and further to finding whoever populates the `$0D80-$0D9F` job table that decides what gets uploaded - see `EmuSen_Core_Gameplan.md` §1 for the investigation detail. **Status since last actively worked is unconfirmed** - later sessions moved on to other games (ALTTP, Super Metroid, below) without a recorded resolution; see `EmuSen_Project_Overview_v2.md` §7 item 1 before assuming this is still the open, in-flight bug. Everything else about the game (movement, level layout, most rendering) plays normally, hence Fair rather than a lower category.

- **Super Mario All-Stars** — reported symptom: no controller response once past each embedded game's title screen, except Super Mario World. Root-caused, **not an EmuSen bug**: SMAS's classic NES-style sub-games (SMB1, SMB2/Lost Levels, SMB3) read Controller 2 for input, not Controller 1 - a real, independently-documented quirk of the actual cartridge (other emulators, e.g. PocketSNES, have hit the same thing; native SMW correctly reads Controller 1). Confirmed directly against this core: on the Select Game menu, every Player 1 button except D-pad scrolling did nothing, while a single Player 1-equivalent press sent as Player 2 (`EmuSen.Pharaoh --tap2`) immediately advanced to File Select. Workaround now built in: a mirror-Player-1-to-Player-2 toggle (`EmuSen.Hotaru`: F7 hotkey, a per-`GameWindow` flag; `EmuSen.Mistress`: checkbox in Controller Settings), off by default so it doesn't affect real two-player sessions in other games. Not yet verified all the way into actual SMB1 gameplay with the mirror on (headless testing got to File Select but not further before this was written) - the menu-level finding is strong evidence, not a full confirmation.
  - A separate, independently-real bug found during the same investigation: `Input.WriteStrobe()` seeded the manual `$4016`/`$4017` shift registers from the previous frame's auto-joypad latch instead of the live button state, contradicting its own doc comment. Fixed, but not confirmed to be related to this game's symptom specifically.

- **The Legend of Zelda: A Link to the Past** — real, reproducible bugs found during testing, none of them blocking progress:
  - Stuck overworld subscreen ("yellow bar"): closing an item-get dialog (e.g. the Lamp chest) can leave WRAM `$1D`/`TS` stuck enabling BG1 on the subscreen outdoors, permanently color-math-blending a torch overlay tile into the overworld as a solid yellow band. Root cause and investigation history (including two reverted fix attempts) in `Hardware/Nintendo/Venus - SNES/Venus_PPU.md` §12.1. Still open, unpatched.
  - Missing rain overlay: the overworld storm-intro rain doesn't always render on a fresh save, and on other saves can appear briefly then disappear while walking. Root-caused as far as BG3's raindrop tilemap not being consistently populated/maintained in VRAM. Not yet root-caused further. Details in `Venus_PPU.md` §12.2.
  - (Fixed, no longer an issue as of this testing pass) Picking up the Lamp from a chest showed the "You got the Lamp!" dialog but the item never entered permanent inventory — root-caused to a 65816 CPU emulation bug (opcode `0x87`, `STA [dp]`, wired to the wrong addressing-mode function), fixed and verified against ground-truth CPU test vectors.

- **Super Metroid** — **was Unplayable (boot hang), now boots.** The hang before the title screen is fixed: the IPL ROM was stamped into APU RAM rather than overlaid read-only above it, so the game's own audio-engine upload into `$FFC0-$FFFF` overwrote the boot code mid-execution. Full root-cause writeup in `Hardware/Nintendo/Venus - SNES/Venus_APU.md` §2.7, hardware model in §1.1. Verified headless over 60 seconds: Nintendo logo → Ceres cinematic → title screen → attract-mode demo, with the SPC700 running Super Metroid's own audio engine at `$1500` instead of sitting in the IPL. **Actual gameplay is not yet verified** — nothing past the attract demo has been exercised, so this Fair rating is provisional and based only on the boot sequence rendering correctly; it needs a real play session (and an audio check) before it can be trusted or moved up.

- **Donkey Kong Country** — **was Unplayable (black screen, never booted), now boots.** Not a coprocessor game despite the look: cartridge type `$02` (ROM + RAM + battery), and Rare's pre-rendered SGI artwork is ordinary compressed tiles/sprites needing no special silicon — only a big ROM, hence **HiROM**, which this core did not implement at all. Full writeup in `Hardware/Nintendo/Venus - SNES/Venus_Memory.md` §2.1a. Verified headless: Rare logo → "Nintendo Presents" → DK/boombox intro cutscene → title screen, all rendering correctly. **Gameplay not yet verified** — nothing past the title has been exercised, so this Fair rating is provisional and needs a real play session. 
- **Donkey Kong Country 3** — **also fixed by the same HiROM work**, same map mode and cartridge type. Verified headless as far as its title screen. Gameplay not yet verified.

- **Donkey Kong Country 2** — boots and runs (HiROM mapping works), but stops on Nintendo's *"This game pack is not designed for your SUPER FAMICOM or SUPER NES"* lockout screen. **This is correct emulation, not a bug.** The local `DKC2.smc` has header country byte `$02` (Europe/PAL) where DKC and DKC3 are `$01` (NTSC); the game reads `STAT78` (`$213F`) bit 4 to identify the console, EmuSen reports NTSC (which is accurate — it only models an NTSC machine), and a PAL cartridge in an NTSC console shows exactly this screen on real hardware too. Two ways forward: use an NTSC dump, or add PAL console support (`STAT78` bit 4, 312 scanlines, 50Hz). The latter is now much more tractable than it was — frame pacing already flows from `ICore.FrameRateHz` rather than a hardcoded 60 (`Venus_CPU.md` §8.5b), so a PAL core would just report its own rate.

### Unplayable

- _(none currently)_

---

## 3. Notes on methodology

- Testing uses `EmuSen.Pharaoh` (scripted, reproducible: save states, `--tap` input sequences, screenshot capture, per-pixel/register diagnostics) plus live interactive play via `EmuSen.Hotaru` for anything that needs real-time feedback (precise movement/navigation, timing-sensitive input) that scripted input can't easily reproduce.
- A bug found and fixed gets *removed* from a game's list of open issues, not just annotated — this document should always reflect current, real behavior, checked against the actual running build, not a historical changelog. (Changelog-style history belongs in commit messages and the hardware docs' own investigation writeups, not here.)
- "No known issues" (Perfect) means no issues found *in what's been tested*, not a formal claim of full-game coverage — most games here have only had specific scenes/mechanics exercised, not a full playthrough.
