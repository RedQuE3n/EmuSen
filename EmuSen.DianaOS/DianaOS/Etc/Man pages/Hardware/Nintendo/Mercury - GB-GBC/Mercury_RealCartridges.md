# Mercury — what happened when real cartridges were finally run

*Written 2026-08-09. Until this point every one of Mercury's 146 tests ran against
`SyntheticGbRom`, and `Mercury_Gameplan.md` §3 named that as the largest remaining
gap — "a gap in the evidence rather than in the code". This page is what closing
it found.*

---

## 1. What was run, and how

Seven images: Tetris, Super Mario Land, Dr. Mario, Kirby's Dream Land, Link's
Awakening (and its DX prototype, which despite the name carries `$0143 = $00` and
is a monochrome build), and Pokémon Yellow. Boards covered: no-MBC, MBC1, MBC3
with battery, MBC5 with battery. Sizes from 32K to 1 MB. Yellow is `$0143 = $80`,
so it is also the first colour cartridge Phase D has ever seen.

None of them is committed and none of them is required. `CommercialRomLibrary`
discovers whatever is sitting in the gitignored `TestRoms/` directory and
`MercuryCommercialRomTests` passes when that directory is empty — the same shape
`ReferenceDumpTests` already used for the reference dumps, and for the same
reason.

A detail worth stating because it is easy to get backwards: **the ROM library is
never written to.** The images under `TestRoms/` are copies extracted from
archives; `AppSettings.RomDirectory` is read and nothing else.

### 1.1 The invariants, and why these ones

There is no Game Boy reference emulator wired into this project's differential
harness, so none of the checks can be "matches hardware". What is available is
*self-consistency*, and five of the six are that:

| Check | What a failure would mean |
|---|---|
| Boots and renders more than one shade | the PPU never drew what the game asked for |
| Two fresh runs agree exactly | something outside the emulated machine leaks in |
| A save state resumes into the same future | state exists that `StateSerializer` does not walk |
| Fast-forward does not change the machine | `SkipRendering` skips more than pixels |
| The music changes rather than holding one note | the sound driver is not being driven |
| `BatteryRamDisabled` leaves no `.srm` | a documented switch is lying |

The save-state one is the strongest. Every subsystem added in the four phases —
PPU position, colour palettes, VRAM/WRAM banks, HDMA, four channels of APU — is
live state under a real game, and a round trip through `SaveState`/`LoadState`
that lands on a byte-identical frame 120 frames later is a hard thing to pass by
accident. All seven pass it, Yellow included.

## 2. The defect: `BatteryRamDisabled` was not honoured

`CoreOptions.BatteryRamDisabled` carries the comment "honoured by every core".
Venus honours it. Moon honours it. **Mercury did not**, because
`Cartridge.Load` set `_savePath` unconditionally before `LoadSram` ever looked at
the flag.

The symptom is worse than a missing feature, and it is the reason this is the one
finding here that is unambiguously a bug rather than an observation: Pharaoh
prints `--nobattery: the cartridge save is neither read nor written` and then
Mercury read and wrote it anyway. Every run in this investigation that used the
flag was quietly loading whatever the *previous* run had left beside the ROM, and
`SaveSram` fires every 300 frames, so any run past frame 300 left one.

That is exactly the failure recorded for the SuperFX work — a harness run whose
result depends on the run before it. It went unnoticed here for four days because
`SyntheticGbRom` never sets a battery flag in the header, so no synthetic test
could ever have caught it. Three of the seven real cartridges catch it
immediately.

The fix moves `_savePath` assignment inside `LoadSram` and gates it on the flag,
which matches Moon exactly, including the property that a null `_savePath` is
also what disables `SaveSram`.

## 3. What the reference emulator says

The libretro probe (`EmuSen.WiseMan/Reference/`) already had gambatte unpacked, so
Super Mario Land was run in both and compared at frame 1500, deep into its attract
demo:

- **Screen**: 894 of 23040 pixels differ, quantised to the four DMG shades — 3.9%.
  Every differing pixel is on Mario or in the two-tile column beside him. The
  demo is one animation frame out of phase; the background, the window-layer HUD,
  the parallax and the sprite priorities are pixel-identical.
- **WRAM**: 46 of 8192 bytes differ. Reading them back through the disassembler,
  the divergent block at `$C00C-$C01B` is a four-entry shadow-OAM struct — Mario's
  2×2 sprite — and the bytes that differ are the tile numbers. Same cause.

The phase difference has a known origin: **Mercury boots roughly four frames ahead
of gambatte**, because neither runs a boot ROM but they hardcode different amounts
of the startup sequence. Any future frame-indexed differential against gambatte
has to align for that before comparing anything.

### 3.1 The per-frame signature did not work here, and why

`--sig` writes a CRC per frame, which is how the SNES work located divergences
rather than measuring them. Against gambatte it was not usable: at the best
alignment only 1.1% of frames agreed, because a whole-WRAM CRC is maximally
sensitive — one byte of a one-frame-late animation counter changes it — while the
two machines were in fact running the same code. **A per-frame CRC needs
per-region granularity to be informative on a machine with 8K of WRAM.** Recorded
as a negative result: the tool is right, the granularity was wrong for this
question.

## 4. Super Mario Land is silent, and it is not the emulator

The finding that took the longest and turned out not to be a bug.

SML produces **zero writes into `$FF10-$FF3F`** across 1500 frames of title screen
and attract demo — not one note, while the other six games all drive their
channels. The chain of evidence:

1. The APU is powered and configured. The boot code at `$01AF` writes
   `NR52 = $80`, `NR51 = $FF`, `NR50 = $77` through `LD (HL-),A`, and all three
   land.
2. Interrupts are delivered: 285 vblanks and 286 STAT interrupts over 300 frames.
3. The game is running — the attract demo plays 1-1 and matches gambatte (§3).
4. Static scanning finds no `LDH ($FF13),A` anywhere in the image; SML drives its
   registers through `LD ($FF00+C),A` like Tetris does. Counting that instruction
   at execute time gives **Tetris 14 sound-register writes in 20 frames and SML
   zero**.
5. Coverage settles it. With Start tapped, code pages `$6Cxx`, `$6Dxx` and `$6Exx`
   execute and the driver runs perfectly — 216 `NR14` triggers, wave RAM loaded,
   all four channels. Without Start, **those pages are never reached at all.**

So the game does not call its music driver on that path. Mercury runs the code it
is given; the code does not ask for sound.

What this does **not** establish is whether a real DMG is also silent there.
Settling that needs reference *audio*, and the blocker is concrete: the probe's
libretro backend answers `--wav is not supported by the libretro backend`. The
Mesen backend has audio capture and no Game Boy. **Until the libretro backend
grows an audio callback there is no way to diff Game Boy sound against anything**,
and that is the single most valuable thing that could be added to this harness
next.

### 4.1 The test that passed for the wrong reason

The first version of the audio invariant asserted "produced non-zero samples".
Tetris passed it while doing nothing but holding a single tone left over from its
boot-time channel setup — 292 writes to `NR51` and not one to a frequency
register. The assertion was measuring that a DAC was enabled, not that music was
playing.

It now asserts that the set of distinct `(pulse1, pulse2, wave, noise)` settings
observed over 900 frames exceeds four, which a stuck note cannot satisfy. This is
worth recording as a method note rather than a bug: **"is there output" is almost
never the invariant worth asserting about an audio subsystem — "does the output
change" is.**

## 5. What did not go wrong

Recorded because negative results are results, and because each of these was a
plausible place for a four-day-old subsystem to fail:

- **No illegal opcode was reached** by any of the seven. Since Mercury throws on
  those (`Mercury_Cpu.md` §6.1), booting at all proves it.
- **No MBC1, MBC3 or MBC5 misbehaviour surfaced**, across 32K to 1 MB.
- **Colour works on a real cartridge.** Pokémon Yellow reaches its Surfing Pikachu
  intro in full colour — BG and OBJ palettes, both VRAM banks, and the water
  animation that HDMA drives.
- **The absent VRAM access blocking cost nothing visible** (`Mercury_Ppu.md` §7).
  This was the simplification most likely to corrupt a real game's graphics, and
  seven games show no sign of it. That is evidence for the trade being right, not
  proof — the condition for revisiting it is unchanged.
- **The window's line counter survives fast-forward.** The bookkeeping deliberately
  left outside the render guard (`Mercury_Ppu.md` §4.2) is now checked against real
  games rather than a synthetic window.

## 6. What is still not covered

- **No CGB-exclusive cartridge.** Yellow is `$80`, colour-enhanced. Nothing has
  exercised double speed, HDMA under real pressure, or `$C0`-only code, because
  the library on this machine is a DMG set.

### 6.1 The header scan, because one image is not what its filename says

*Added 2026-08-09.* §6 asserted "the library on this machine is a DMG set" from the
filenames. Reading `$0143` out of all seven images confirms it, and corrects one
assumption worth writing down:

| Image | `$0143` | Meaning |
|---|---|---|
| Dr. Mario | `$00` | monochrome |
| Kirby's Dream Land | `$44` | **not a flag** — see below |
| Zelda: Link's Awakening | `$00` | monochrome |
| Zelda: Link's Awakening **DX** (Proto, 1998-11-08) | `$00` | **monochrome** |
| Pokémon Yellow | `$80` | colour-enhanced |
| Super Mario Land | `$00` | monochrome |
| Tetris (Rev 1) | `$00` | monochrome |

**The DX proto is not colour coverage.** A shelf holding "Link's Awakening DX" reads
like the CGB gap is half closed; that build is flagged `$00` and Mercury takes its
monochrome path. It is an early proto that had not yet set the flag. Anyone reaching
for colour evidence should not reach for this file.

**Kirby's `$44` is the title, not a flag,** and it is correct that nothing treats it
as one. `$0134`–`$0143` is a sixteen-byte title field on pre-CGB carts, and
`KIRBY DREAM LAND` is exactly sixteen characters — the final `D` is `$44`, sitting
in the byte the CGB later reassigned. `Cartridge.cs` matches `$C0` and `$80`
exactly rather than testing bit 7, so `$44` falls through to monochrome. A
`(b & 0x80)` test would agree here by luck and disagree on a title ending in a
letter above `$7F`; the exact match is the reason this never became a bug.
- **No audio differential**, per §4.
- **Deep gameplay.** Everything here is title screens, attract demos, and a few
  hundred frames past a tapped Start. Nothing has been played.
- **No SGB.** Several of these images are SGB-enhanced and Mercury ignores that
  entirely.

### 6.2 The scan is a test now

*Added 2026-09-06, after a session that re-derived §6.1 from scratch.* The header
scan above was run once, by hand, and written down. A written-down measurement
answers the question it was taken for and nothing later; when "why is Zelda DX in
black and white" was asked again, the answer was re-measured with a throwaway probe
rather than read off a passing test. `MercuryCommercialRomColourTests` is that scan
turned into a standing assertion, so the next person gets it from a test run.

**What it claims** is narrow and in two halves. The mode follows the file's own
`$0143` — `$80` or `$C0` gives `CoreName == "GBC"`, anything else gives `"GB"` —
and the mode reaches the frame buffer: a monochrome cartridge draws no pixel whose
channels differ, a colour one draws at least one that does.

**It reads the byte out of the file itself** rather than asking `Cartridge.Cgb`.
The claim under test is that the header decides, so taking the header from the
thing being tested would make it agree with itself; a `Cartridge` that parsed the
byte wrongly would satisfy a test written that way.

**The colour half looks twice.** A colour cartridge can still be sitting on a
monochrome logo at `BootFrames`, so the test runs on to `PlayFrames` before it will
call a grey frame a failure. That is a tolerance, not a measurement: it exists so
that adding some future colour ROM to `TestRoms` does not produce a failure that
means only "its logo is white".

**Negative controls.** Two mutations, each reverted:

| Mutation | Result |
|---|---|
| `Cartridge.cs` maps `$80` to `CgbSupport.None` | Pokémon Yellow fails on `CoreName`, six monochrome images pass |
| `WriteColorPixel` writes one channel to all three | Yellow fails with "drew only greys through 900 frames", six pass |

The second is the one worth having. Mode selection is already pinned by
`MercuryCgbTests` on synthetic ROMs; that a real colour game's palettes actually
reach the screen was, until now, asserted nowhere — it was a sentence in §5.

**What it still does not cover.** It cannot say the colours are *right*: any single
non-grey pixel satisfies it, and comparing against hardware is §3's job. The
monochrome half assumes the four neutral greys of `Mercury_Ppu.md` §6, so a panel
tint option would have to be handled here rather than silently failing six
cartridges. And `$C0` remains untested, for the reason §6 already gives — the
library on this machine holds 1,915 Game Boy images and 17 of them are colour, none
`$C0`-only.

**Cost**: seven cartridges, up to 900 frames each, about 10 seconds, and it runs
concurrently with the other six commercial-ROM classes. Nothing is required — the
`TestRoms` directory is gitignored and an empty one still passes as one empty case.
