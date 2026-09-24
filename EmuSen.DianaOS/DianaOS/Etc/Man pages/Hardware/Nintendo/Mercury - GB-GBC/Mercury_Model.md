# Mercury_Model — which console a Game Boy game runs on

*Written 2026-09-24.* The user asked for "a dropdown to Mistress that lets you cycle between gameboy and gameboy color".
This page records what was built for that request. That is a core setting both engines honour, and it brings a mode
Mercury never had: a Game Boy Color running a cartridge made for the Game Boy.

The page also records:

- what each choice does to each kind of cartridge;
- where each hardware fact came from;
- how the model interacts with states, names and folders;
- what was measured, and what was left out.

The frontend half, the GB tab's row, is `EmuSen_Settings_Reference.md` §4.47. The defects fixed before this work
began are `Mercury_Native.md` §9.

## 1. The setting

**The setting and where it is stored.**

- **Key:** `Model`, a per-console choice.
- **Values:** `Auto` (the default), `Game Boy` and `Game Boy Color`.
- **Stored:** in `graphics.json` under `Consoles.GB`, and shown as the **Model** row on Graphics Settings' GB tab.
- **Declared in:** `MercuryCore.ModelSettings`, which `CoreCatalog.SettingsFor("GB")` returns, as `MarsCore` declares
  `VideoSettings`.
- **Honoured by both engines**, through `ICoreSettings`: `MercuryCore` and `MercuryRtCore`. Neither frontend nor
  factory resolves the model. The core does, at load.

**When it takes effect.** At the next game load, as the Engine row's choice does (§4.44 of the settings reference).
A change before a game's first frame loads the game again at once, on the chosen console. That is the rule Mars's
Expansion Pak follows (`Mars_Core.md` §7), and it is how Mistress gets the stored choice to the core: Mistress loads
the game, then hands the core its console's settings, before the first frame. A change after the first frame is
stored and waits for the next load.

**How the header and the choice decide.** `MercuryCore.ConsoleFor(model, header)` decides, and the Rust
`Model::console_for` decides the same way:

- **Game Boy** is always a Game Boy.
- **Game Boy Color** is always a Game Boy Color.
- **Auto** is a Game Boy Color when `$0143` is `$80` or `$C0`, which is today's rule, unchanged.

## 2. What each choice does with each cartridge

The machine now has three states, not two. `MemoryBus` carries three read-only flags, fixed at construction:

- **`CgbHardware`:** the console is a Game Boy Color.
- **`Cgb`:** colour mode, meaning a Color running a colour cartridge. It keeps its old meaning, so every colour path in
  the core is unchanged.
- **`DmgCompat`:** a Color running a Game Boy cartridge.

The Rust bus carries the same three flags. VRAM (16 KB) and WRAM (32 KB) follow the hardware, not the mode.

| Cartridge `$0143` | Auto | Game Boy | Game Boy Color |
|---|---|---|---|
| `$00` (Game Boy only) | Game Boy | Game Boy | **Color, compatibility mode** (§3, §4) |
| `$80` (colour-enhanced) | Color | **Game Boy**, the game's monochrome path | Color |
| `$C0` (Color only) | Color | **Game Boy**, the game's own "Color only" behaviour | Color |

**A colour cartridge on the Game Boy** gets exactly what a Game Boy gives it: `A = $01` at `$0100`, DMG registers, 8 KB
banks, no colour registers. A `$80` game reads `A` and takes its monochrome path (`Mercury_Cgb.md` §1). This is what
real hardware does. Pan Docs says the DMG boot ROM never reads `$0143` (`Power_Up_Sequence.md`, lines 30–34).

**A Color-only cartridge on the Game Boy: decided to run it, not to refuse it.** A real DMG boots a `$C0` cartridge.
The DMG boot ROM checks only the logo and the header checksum. The game itself sees `A = $01` and shows whatever it
shows a Game Boy, usually a "this game can only be played on a Game Boy Color" screen. Mercury does the same and adds
nothing. The alternative was a refusal from the core, which would be a message the hardware never gives. It would also
make a Model row that silently does nothing for these games. A game that shows no such screen and then misbehaves is
behaving as it would on the console, which is the point of the choice.

## 3. The Color's boot ROM, for a Game Boy cartridge

Mercury starts past the boot ROM (`Mercury_Cpu.md` §5). So what the Color's boot ROM leaves behind for a Game Boy
cartridge is written by `MemoryBus.Reset` and `Cpu.ResetForCompatibility` (Rust: `MemoryBus::reset` and
`Cpu::reset_for_compatibility`). It comes from documented behaviour, not from any emulator's code.

### 3.1 The palette

**The sources.** Pan Docs (`Power_Up_Sequence.md`, "Compatibility palettes", lines 107–145) describes the choice. It
does not print the tables; it points to a disassembly of the Color's boot ROM for them. The tables in
`Ppu/CompatibilityPalettes.cs` and `ppu/compat.rs` are that ROM's data:

- the 79 title checksums;
- the 29 fourth letters;
- the 94 entries of palette row and shuffle flags;
- the 29 rows of three offsets;
- the 30 palettes of four RGB555 colours.

They were read from ISSOtm's disassembly at the commit Pan Docs links, and cross-checked in §6.1. They are data about
the hardware's ROM, typed as numbers. No emulator's code was used.

**The algorithm**, which Pan Docs states and the disassembly confirms:

1. **Licensee.** Only Nintendo's games are coloured. The old licensee `$014B` must be `$01`, or it must be `$33` with
   the new licensee `$0144–$0145` = `"01"`. Any other game gets palette 0.
2. **Checksum.** The 8-bit sum of the sixteen title bytes `$0134–$0143`, the CGB flag included.
3. **Look-up.** The checksum's first match in the table, by index:
   - index 0–64 is the palette number;
   - index 65–78 is ambiguous. The title's fourth letter `$0137` is compared against up to three rows of letters,
     giving number 65 + column, 79 + column or 93. No match means palette 0.
4. **Shuffle.** Each number names a row (e0, e1, e2) and three flag bits:
   - BG = e2;
   - OBJ0 = e0 if bit 0 is set, else e2;
   - OBJ1 = e1 if bit 2 is set, else e0 if bit 1 is set, else e2.

   The disassembly's own comment states the OBJ rules backwards. Its code, TCRF and the MiSTer core's boot ROM agree
   on the rule above.
5. **The default**, palette 0: BG `7FFF 1BEF 6180 0000`, OBJ0 = OBJ1 `7FFF 421F 1CF2 0000`.

**What is written.** Object palettes 0 and 1 (sixteen bytes through OCPD), then background palette 0 (eight bytes
through BCPD), both indices auto-incrementing from 0. So BCPS is left at `$88` and OCPS at `$90`. They read back as
`$C8` and `$D0`, as mooneye's hardware-verified `misc/boot_hwio-C` expects. BG palettes 1–7 and OBJ palettes 2–7 stay
white, which is Mercury's reset.

**Not modelled.** Three things are not modelled:

- **The twelve button combinations** held during the logo, which override the choice. Mercury has no logo to hold them
  through.
- **The Nintendo logo tilemap** the ROM writes to VRAM for checksums `$43` and `$58`. Mercury clears VRAM at reset for
  the DMG too, where the DMG boot ROM also leaves the logo behind (`Mercury_Cpu.md` §5).
- **AGB mode.** It adds one to B.

### 3.2 The registers at hand-off

`A = $11`, `F = $80`, `B` = the title checksum for a Nintendo game (else `$00`), `C = $00`, `DE = $0008`,
`SP = $FFFE`, `PC = $0100`. `HL = $991A` when B is `$43` or `$58`, where the logo map was drawn, and `$007C` otherwise.
These values come from Pan Docs' "CGB (DMG mode)" column and its footnotes.

**Where the referee disagrees, and why Pan Docs was followed.** The MiSTer core runs SameBoy's boot ROM. It always
hands off `HL = $007C` (`BootROMs/cgb_boot.asm:886`, `:850`), and it hands off `F = $A0` because of its own menu hook
(`Mercury_Referee.md` §4.5). Pan Docs' HL rule follows from the ROM writing the tilemap through HL. mooneye's
`misc/boot_regs-cgb` checks a non-Nintendo header, so it sees only `B = $00, HL = $007C`, and cannot tell the two rules
apart. `A = $11` is kept: a Game Boy game that tests `A` sees a Color, which is what real hardware shows it.

## 4. A Game Boy cartridge on the Color

### 4.1 How it renders

The DMG pipeline runs, with one change. Every pixel's two-bit shade, taken through BGP, OBP0 or OBP1 as on a Game Boy,
is looked up in the colour palettes instead of the grey ramp:

- the background and window use BG palette 0;
- an object uses OBJ palette 0 or 1, by attribute bit 4.

`Ppu.WriteShade` in C# and `write_shade` in Rust are the one place this happens. Pan Docs (lines 111–112) and the RTL
(`video.v:1166-1176`) agree on it.

The rest of the pipeline is the DMG's, because `Cgb` is false:

- bank-1 attributes are ignored;
- sprites sort by X;
- LCDC bit 0 keeps its DMG meaning.

Mercury's DMG rule for LCDC bit 0 is shade 0. In compatibility mode that becomes BG palette 0's colour 0, which is white
for every compatibility palette but three: palettes `$32`, `$44` and `$51` start with `0000`. The referee records
(`Mercury_Referee.md` §2.12) that both references show BGP's colour 0 instead, on a DMG too. That dispute belongs to
the DMG renderer and was not changed here.

### 4.2 Which colour registers answer

| Register | Mercury in compatibility mode | Why |
|---|---|---|
| KEY0 `$FF4C`, KEY1 `$FF4D` | read `$FF`, writes ignored, **so `STOP` never switches speed** | Pan Docs' "CGB mode only"; RTL `gb.v:225-226`; mooneye reads `$FF` |
| HDMA `$FF51-$FF55`, RP `$FF56` | read `$FF`, writes ignored | RTL `gb.v:223-227`; mooneye `$FF` |
| BCPD/OCPD `$FF69/$FF6B` | read `$FF`, writes ignored | RTL `video.v:545-594`; mooneye `$FF` |
| OPRI `$FF6C`, SVBK `$FF70` | read `$FF`, writes ignored; the WRAM bank stays 1 | RTL `gb.v:222`; mooneye `$FF` |
| **VBK `$FF4F`** | **reachable**: reads `$FE \| bank`, a write selects the CPU's VRAM bank, and the renderer never reads bank 1 | see below |
| **BCPS/OCPS `$FF68/$FF6A`** | **reachable**: the index reads and writes as in colour mode | see below |

**The two unsettled rows: the RTL was followed, and why.** The referee found the RTL and Mesen disagreeing on VBK and
the palette indices (`Mercury_Referee.md` §4.7). The RTL leaves them reachable. Mesen answers `$FF` and drops writes.

mooneye's `misc/boot_hwio-C` passes on CGB, AGB and AGS hardware, and it expects `$FF4F = $FE`, `$FF68 = $C8` and
`$FF6A = $D0` after boot. A register that answered `$FF` could not give those reads. So the hardware-verified reads
side with the RTL, and Mercury follows it. The *writes* are verified by nothing known. They follow the RTL for
consistency, and they are recorded here as the unverified half.

**Not modelled.** Three registers are not modelled in compatibility mode:

- **FF72–FF75**, the undocumented registers. Mercury models them in neither mode, so they read whatever the generic
  I/O array holds.
- **The STAT write quirk.** Pan Docs says a Color in DMG mode does not have it. Mercury emulates it on no model, so
  nothing had to change.
- **OPRI** as a register. It reads `$FF`, and the renderer's priority comes from the mode, not from a latch.

## 5. States, names and folders

**The model is in the state, and a state resumes on its own console.** State version 7 writes one byte after the
version: 1 for a Game Boy Color, 0 for a Game Boy (`MercuryCore.SaveState`, Rust `Machine::write_state`). The layout
of everything after it depends on that byte. VRAM and WRAM are twice and four times as large on the Color. So reading a
Color's state into a Game Boy's machine cannot work.

When a state names the other console, `LoadState` rebuilds the machine as that console before reading. The rebuilt
machine keeps the host's side of the old one:

- the held buttons;
- the channel mutes and the sample queue's limit;
- the write observer;
- the rendering switch;
- the Game Genie table;
- the mixer's rate.

The Rust `Machine::build` carries the same list. Versions 5 and 6 carry no byte, and they were made on the console the
header chose, so that is the console they rebuild as. The rule is Mars's Expansion Pak rule again: a state resumes
with the machine it was made on. The Model row changes only what a *new* load builds.

**The debugger across a rebuild.** MercuryRT's stage 6 (`Mercury_Native.md` §8.5) reached this branch after the
setting was built, and the merge had to decide what a rebuild does to the debugger's side:

- **C#.** `Build` hands each new bus the kept write observer and each new CPU the call-stack observers, so a machine
  rebuilt for a state reports calls, returns and interrupts as the one it replaced did. The call stack itself is not
  reset, as it is not by a state load on one console.
- **Rust.** The rebuilt `Machine` takes the old one's `hooks`, which are in no state, so the debugger's tables and
  logs survive the rebuild as they survive any load.
- **MercuryRT's mirror.** The debugger reads MercuryRT through a C# `MercuryCore` refreshed from its state. That mirror
  now takes the shim's Model at every load, so it is on the machine's console before the first refresh. Before this,
  it was an *Auto* machine until the first refresh rebuilt it from the state's console byte, and a view read in
  between saw the wrong console (§6.4).

**`CoreName` is the console running, not the cartridge.** It is `"GBC"` while the machine is a Color, in colour or
compatibility mode, and `"GB"` otherwise, on both engines. A state from the other console changes it (the shim asks
the machine, `mercury_machine_cgb_hardware`).

**Where CoreName is not used.**

- Save and state folders are keyed by the ROM's path (`SaveLibrary.StatePathFor`; the `.srm` beside the ROM,
  `Mercury_Memory.md` §9). So neither moves when the model changes.
- The library shelf is the cartridge's (§4.46 of the settings reference), not the model's.
- Mistress's per-console settings, pad bindings, screen filter and cheat console used to follow `CoreName`. They now
  follow the catalogue's console, `GB`, for every Game Boy game (§4.47 there). Before this, a colour cartridge's
  `CoreName`, `GBC`, named a console with no tab, so the GB tab's settings never reached colour games. The Model row
  would have inherited that defect.

**The C ABI.** `mercury_machine_new` takes the model as a `u32` in `GbModel`'s order, and refuses any other value
with status −11. `mercury_machine_cgb_hardware` reports the console. The interface version is 3.

## 6. Evidence

### 6.1 The palette table against documented values

- **TCRF's per-game table** (the "Game Boy Color Bootstrap ROM" notes page, oldid 1256473) lists 45 configurations
  under 74 hashes, with fourth letters, as RGB888 converted by round(c × 255 / 31). All 74 rows were fed to
  `CompatibilityPalettes`. **73 agree exactly.** `MercuryModelTests.The_colour_palette_for_a_listed_game_is_the_one_tcrf_documents`
  is 73 cases.
- **The one disagreement is Radar Mission, checksum `$8C`** (confirmed from the library's own header; the ROM was only
  read). TCRF gives it entry `$01`, flags `$00`: palette 13 for all three. The boot ROM's own tables give `$8C` the
  palette number 8, whose byte is `$20`: entry `$00`, flags `$01`. And entry `$01` with flags `$00` belongs to no number
  in the table at all.

  The ROM's data was followed, and the disagreement is a test of its own
  (`Radar_mission_follows_the_boot_roms_table_where_tcrf_disagrees`). Which the hardware shows is unverified, because
  no hardware was run.
- **Pan Docs' default** (palette 0) and the licensee rules have their own cases. So does the three-row letter table:
  `$B3` with B, U or R, `$46` with R, and a letter that matches nothing.
- **The MiSTer core's boot ROM** is SameBoy's reimplementation of the same selection. The research that fed this page
  checked it against the disassembly's tables: all 94 numbers and 12 button combinations resolve to identical palettes.
  That is agreement between two readings of one ROM, and it is weak evidence. It is recorded, not relied on.

### 6.2 Both engines, every choice

The following cases are all in `MercuryModelTests`, and each is a lock-step run: C# against Rust, state, picture and
sound, every frame.

- **Nine combinations.** Each cartridge kind (`$00`, `$80`, `$C0`) on each choice. A probe program records the
  hand-off registers and the colour registers, tries a speed switch and a WRAM bank, and draws the four shades. The
  assertions are per console:
  - the DMG's registers, greys and 8 KB VRAM;
  - the Color's colour-mode registers and its speed switch;
  - compatibility mode's registers, the §4.2 table (mooneye's values), no speed switch, WRAM bank 1, and palette 0's
    colours on screen.
- **A Nintendo header's hand-off.** Checksums `$70`, `$58` and `$43`: B, HL, and colour RAM holding the chosen palettes.
- **All 94 palette numbers.** Each selected by a title and run on both engines.
- **States across consoles, four ways.** A Game Boy game made on the Color and loaded on Auto, and the reverse; a `$80`
  game made on the Game Boy and loaded on Auto; a `$C0` game made on Auto and loaded on the Game Boy. Each was
  identical after the load and ran 60 frames into the same state as the machine that made it. A button held before
  the load was still held after the rebuild, on both engines.
- **The setting itself** on both engines: read, refused, reloaded before the first frame, and deferred after it.

The crate has two tests of its own: the registers and palettes for a Nintendo header on the Color, and the rebuild on
load with the loader's held buttons kept.

### 6.3 The corpus on each console

**Prediction, stated before the run.** Under *Auto* nothing moves: the committed baseline is untouched.

- mooneye's `misc/boot_regs-cgb` fails under *Auto*, as in the baseline. That ROM's header is a DMG header with
  licensee `ZZ`, verified on a CGB. It passes under *Game Boy Color*, because §3.2's hand-off is exactly what it
  checks: `A=$11 F=$80 B=$00 C=$00 D=$00 E=$08 H=$00 L=$7C`.
- Its sibling `misc/boot_hwio-C` still fails under *Game Boy Color*. It also checks the unmapped I/O registers and the
  audio registers' read masks. Mercury answers those from its generic I/O array on every model, which is the referee's
  §2.1 candidate.
- Both engines stay identical on every ROM under every choice.

**Results.**

- **Lock-step.** Every corpus ROM ran on both engines under *Game Boy* and under *Game Boy Color*
  (`The_hardware_corpus_runs_identically_on_each_console`). They were identical in serial every frame and in state
  every 30 frames and at the end: 34,940 frames under *Game Boy* and 32,116 under *Game Boy Color*. Under *Auto* the
  existing `MercuryRtCorpusTests` ran the same comparison, 31,986 frames.
- **`misc/boot_regs-cgb`.** It fails under *Auto* and passes under *Game Boy Color*
  (`Mooneyes_cgb_boot_register_rom_passes_on_the_color_and_fails_on_the_game_boy`), as predicted. It is the one
  hardware-verified check of the hand-off this page builds, and it now passes.
- **Transcripts.** The C# transcript was taken under each of the three choices. Under *Auto* it matches the recorded
  baseline in all 173 verdicts and verdict frames, so **no baseline moves**. The verdicts that move under the forced
  choices are listed in §6.6.
- **After the stage-6 merge** the Mercury filters were run again on the merged tree: 528 of 528, the lock-step corpus
  under each forced choice included.

### 6.4 The debugger

Two tests in `MercuryModelTests`, written after stage 6 was merged, each run first against the code as the merge left
it:

- **`The_debugger_shows_the_console_the_game_runs_on_on_both_engines`.** A Game Boy game on *Auto* and on *Game Boy
  Color*, and a `$80` game on *Game Boy*. The prediction was that the two forced cases would fail and *Auto* would pass.
  They did, in two layers:
  - first at the mirror: MercuryRT's mirror was on the header's console, not the machine's;
  - with the mirror fixed, the *Game Boy Color* case still failed, because the palette list showed greys where the
    screen showed palette 15's colours.

  With both fixes, palette 0 is the four colours the probe draws on screen (`FFFFFF FF8484 943939 000000` for
  checksum `$70`), and the two engines' lists are equal.
- **`A_rebuild_for_a_state_keeps_the_debuggers_call_stack_on_both_engines`.** A state made on one console, loaded on
  the other, and then a `CALL` that does not return. Both engines' call stacks hold the one frame afterwards.

### 6.5 Mistress

`MercuryModelSettingTests`:

- **The GB tab's Model row.** Its three choices, Auto selected, and a choice stored and reported. The window was
  rendered to PNG before and after the change and looked at: the row sits under Engine with its hint, and it reads
  `Game Boy Color` after the change.
- **A game loaded in Mistress runs on the console the tab names, on either engine.** Two cases: a Game Boy game with
  `Game Boy Color`, and a `$80` game with `Game Boy`. The second is the case the `_activeConsole` change exists for.

### 6.6 What moves under the forced choices

The C# transcript under each forced choice, against the committed baseline (which is *Auto*), verdict and verdict
frame for each of the 173 ROMs.

**Under *Game Boy Color*.** The prediction was written down before the run:

- `misc/boot_regs-cgb` would move from fail to pass;
- the ROMs with a colour header would not move, since *Auto* already runs them on the Color;
- the DMG-only mooneye ROMs that check the DMG's boot state would fail where they now pass;
- the timing ROMs might move, in a direction not predicted;
- nothing else would move.

Two verdicts moved:

| ROM | *Auto* | *Game Boy Color* |
|---|---|---|
| `mooneye/misc/boot_regs-cgb` | fail | **pass** |
| `mooneye/acceptance/boot_regs-dmgABC` | pass | **fail** |

Both are the hand-off, seen from each console's side: each ROM checks the registers its own console hands off. The
third prediction held only for `boot_regs-dmgABC`, because `boot_div-dmgABCmgb` and `boot_hwio-dmgABCmgb` already fail
under *Auto*. No timing ROM moved. Compatibility mode keeps the DMG's single speed and its sprite and rendering paths,
so nothing else was expected to.

**Under *Game Boy*.** No verdict moved. Ten verdict frames did, all in blargg's `cpu_instrs`: the full ROM and its
individual tests `03` to `11`. Each still passes, 1.1 to 2.0 times later (`cpu_instrs` itself at frame 3,194 instead
of 1,783). These ROMs carry a colour header. On the Color they switch to double speed and finish sooner. On a Game Boy
the switch does not exist, so they run at single speed. Nothing else in the corpus moved.

### 6.7 Real games on the Color

`Real_games_run_identically_on_the_other_console` runs the four games of the parity set for 1,500 frames with button
presses, on both engines in lock-step, under each forced choice. The games are copies in scratch, not the library.

- Under both forced choices, all four were identical in state every 10 frames, and in picture and sound every frame.
- The three Game Boy games take palettes 40 (Kirby's Dream Land), 5 (Tetris) and 15 (Link's Awakening). Pokémon Yellow
  is a `$80` game, so under *Game Boy Color* it runs in colour mode.
- The last frames were rendered to PNG and looked at. Kirby's Dream Land has a lavender sky and a pink Kirby, and Link's
  Awakening has red bricks on its naming screen. Both are the colours these games are known to show on the Color. This
  is a visual check against common knowledge, not a measurement against hardware.

### 6.8 Mutants

Twenty-three hand-made mutants. Each was applied to a copy of the merged tree, and then run against:

- the crate's tests, when the mutant was in Rust;
- the WiseMan filters `MercuryModel`, `MercuryDefectTests.D3_version` and `MercuryRtStateTests`, with
  `MercuryModelSettingTests` added for the shim and Mistress.

The unmutated copy passed first: the crate, the build, and 157 of 157 tests.

| # | Mutant | Caught by |
|---|---|---|
| P1 | C#: compatibility mode renders grey | 6 tests: the lock-step picture, the hand-offs, the real games |
| P2 | Rust: compatibility mode renders grey | 7 tests, the same |
| P3 | C#: OBJ0 ignores its shuffle flag | 60 tests, TCRF's rows among them |
| P4 | Rust: OBJ1 tests the e0 flag before the e1 flag | **not caught, equivalent** (below) |
| P5 | C#: *Auto* ignores the header | 10 tests, the version-5 states among them |
| P6 | C#: VBK reads `$FF` in compatibility mode | 5 tests: mooneye's value in the probe |
| P7 | Rust: BCPS reads `$FF` in compatibility mode | 5 tests |
| P8 | C#: B is never the title checksum | 5 tests |
| P9 | Rust: HL ignores checksum `$58` | 2 tests |
| P10 | C#: no rebuild for another console's state | 6 tests |
| P11 | Rust: the rebuild drops held buttons | the crate and 4 tests |
| P12 | C#: the rebuild drops held buttons | 4 tests |
| P13 | C#: `CoreName` by the header | 8 tests |
| P14 | shim: no reload before the first frame | 3 tests, Mistress's among them |
| P15 | C#: KEY1 writable in compatibility mode | 5 tests |
| P16 | Mistress keys settings by `CoreName` again | 2 tests: the `$80` game on *Game Boy*, both engines |
| P17 | C#: the compatibility palette never loaded | 9 tests |
| P18 | Rust: the state's console byte not read | the crate and 31 tests |
| P19 | Rust: the rebuild drops the debugger's hooks | **the crate only**, after a test was added (below) |
| P20 | C#: the rebuild loses the call observer | 2 tests: the call stack across a rebuild |
| P21 | shim: the mirror ignores the model | 2 tests: the debugger's console |
| P22 | C#: the debugger shows greys in compatibility mode | 1 test |
| P23 | C#: the debugger's OBP1 through object palette 0 | 1 test, as strengthened (below) |

**P4 is equivalent on the hardware's data.** The OBJ1 rule tests two flag bits in order: `$80` for e1, then `$40` for
e0. Swapping the order changes the answer only when both bits are set. None of the 94 entries of the boot ROM's table
has both set; the highest is `$BC`. No input can tell the mutant apart, so it is recorded, not tested.

**P19 needed a crate test.** It survived the first run, as predicted. The shim pushes every table to the hooks before
each run and drains the logs after, so no WiseMan test can see hooks that a rebuild dropped. The property still
matters to anything that drives the crate directly. So `a_state_from_the_other_console_rebuilds_the_machine_and_keeps_its_hooks`
arms a breakpoint, loads a state made on the Color and expects the stop. It fails under the mutant.

**P23's test was strengthened before the run.** The debugger test first compared only palette 0 against the screen.
Both engines' lists come from the same C# view, so they agree with each other even when that view is wrong. By that
reasoning the first test could not see a wrong object palette. So before the mutants ran, the test was changed to check
all three lists against the boot ROM's table, through the registers. That P23 would have survived the first test is
reasoned, not run.

## 7. What is not done

- Only Mistress hands a core its settings (`ApplyConsoleSettings`). Pharaoh, Hotaru and Serenity apply no
  `ICoreSettings` at all, so they run every Game Boy game on *Auto*. The setting is the core's, so giving them a
  switch is a frontend change, and it was not asked for.
- The debugger's palette list shows compatibility mode's colours (§6.4). Its tile sheet still decodes tiles in greys
  on every model, as it always has on a Game Boy, because a tile has no palette until something draws it. Its
  register list shows the DMG's registers, not the Color's locked ones.
- The boot-time button combinations, the logo in VRAM and AGB mode (§3.1).
- The LCDC bit 0 dispute (§4.1) and the unverified writes to VBK and the palette indices (§4.2).
- No game was compared against a real Color. The evidence for the colours is the boot ROM's own tables, TCRF's list
  (§6.1) and a look at four games (§6.7).
