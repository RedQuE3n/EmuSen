# EmuSen

EmuSen is a multi-console emulator built as a personal project. The goal was to create an emulator framework that is handheld friendly, user friendly, feature-rich, multi-platform, and saves the player the need to install multiple applications to accomplish a beautiful library. The cores are written from published hardware documentation rather than ported from another emulator. The project started in C#, and the cores are now being rewritten in Rust one at a time. Each C# core stays behind as the reference its Rust port is checked against, frame by frame.

Around the cores there is a frontend, a debugging shell built into the emulator, and a headless harness that most of the testing runs through.

**License:** GPL-3.0 · **Platforms:** Linux, Windows, macOS · **Consoles:** SNES, NES, Game Boy / Game Boy Color, Nintendo 64, Sega Genesis / Mega Drive

## What works today

| Console | Core | Where it stands |
|---|---|---|
| Nintendo 64 | **MarsRT** (Rust), the default; **Mars** (C#) | Super Mario 64, Ocarina of Time, GoldenEye 007 and Donkey Kong 64 run. Up to 4× internal resolution, antialiasing, an optional Vulkan path and a recompiler |
| SNES | **Venus** (C#) | Runs commercial games, including SA-1, SuperFX, NEC DSP and OBC1 cartridges. Several play well; none has a verified playthrough |
| Game Boy / Color | **Mercury** (C#), the default; **MercuryRT** (Rust) | Everything but the boot ROM, the link cable and the Super Game Boy. Passes 94 of the 173 blargg and mooneye hardware test ROMs |
| NES | **Moon** (C#) | CPU, PPU, full APU and sixteen mapper boards. Plays Super Mario Bros. 3 and others, but has had far less play-testing than the SNES core |
| Sega Genesis / Mega Drive | **Nephrite** (Rust) | Offered to players since October 2026. Runs commercial cartridges with battery and EEPROM saves, cheat codes and the three- and six-button pads; none has a verified playthrough. The Sega CD and the 32X are not offered yet |

Every other console is an empty, reserved folder. Per-game results are in [the games list](EmuSen.DianaOS/DianaOS/Usr/Home/Documents/EmuSen%20Manual/EmuSen_Games_Tested.md).

### The Rust ports

MarsRT was the first port and is the most finished. It produces the same machine state, picture and sound as the C# core after every frame, and it has everything the C# core has: the threaded display processor, the resolution multiple, antialiasing, the GPU path, debugger hooks and rewind, plus a Cranelift recompiler. It became the default N64 engine after it was played on a Lenovo Legion Go S running SteamOS, where Donkey Kong 64 plays at full speed at 3× internal resolution. The C# core still runs wherever the Rust library is missing. The full story is in [`Mars_Native.md`](EmuSen.DianaOS/DianaOS/Etc/Man%20pages/Hardware/Nintendo/Mars%20-%20N64/Mars_Native.md).

MercuryRT, the Game Boy port, has the whole machine, the debugger and builds for every platform, and CI checks that its sound, picture and save states come out bit for bit the same on Linux, Windows and macOS. It can be selected in the Game Boy's graphics settings and becomes the default after a session of play on the handheld. Porting it turned up three bugs in the C# Mercury, which are now fixed in both engines. Unlike MarsRT it isn't much faster than the C# core, because the cost is in the cycle-by-cycle design rather than the language. Still ahead: running the rest of the Game Boy tests through both engines, recording reference traces, and moving the C# core to a legacy branch. See [`Mercury_Native.md`](EmuSen.DianaOS/DianaOS/Etc/Man%20pages/Hardware/Nintendo/Mercury%20-%20GB-GBC/Mercury_Native.md).

MoonRT, the NES port, is planned and paused ([`Moon_Native.md`](EmuSen.DianaOS/DianaOS/Etc/Man%20pages/Hardware/Nintendo/Moon%20-%20NES/Moon_Native.md)), and Venus will follow. The reasoning for moving the cores to Rust, and for keeping everything else in C#, is in [`EmuSen_Stack.md`](EmuSen.DianaOS/DianaOS/Usr/Home/Documents/EmuSen%20Manual/EmuSen_Stack.md).

### Checking the cores against other emulators and hardware

Two kinds of reference sit outside the project. A probe written in Rust dumps the state of a running reference emulator, such as Mesen or any libretro core, so EmuSen's output can be diffed against it. MiSTer FPGA cores act as a referee where the emulators disagree; the SNES and Game Boy passes are written up in [`Venus_Referee.md`](EmuSen.DianaOS/DianaOS/Etc/Man%20pages/Hardware/Nintendo/Venus%20-%20SNES/Venus_Referee.md) and [`Mercury_Referee.md`](EmuSen.DianaOS/DianaOS/Etc/Man%20pages/Hardware/Nintendo/Mercury%20-%20GB-GBC/Mercury_Referee.md). The N64's display processor is compared with angrylion's renderer, and the Game Boy runs the blargg and mooneye hardware test ROMs.

## Mistress, the frontend

Mistress is the main app. Its library is laid out like OpenEmu's: a sidebar of consoles, with Game Boy and Game Boy Color on separate shelves, and the games shown as a grid of covers or as a list. It is built to work from a controller as well as a keyboard. On SteamOS's Game Mode, settings windows open as sheets inside the main window, and every control, including cheats and an on-screen keyboard, can be reached with the pad.

Other things it does:

- save states, and rewind: four snapshots a second, shown as a strip of pictures to pick a moment from
- per-console graphics settings, including the N64 engine and resolution, and the Game Boy model (Auto, Game Boy or Game Boy Color)
- a Shaders window with the built-in screen filters (a CRT shader and the handheld LCDs) next to RetroArch's slang presets, which are downloaded only on request
- per-console controller bindings, hotkeys and cheats

**Hotaru** is a lighter frontend that takes a ROM on the command line and puts the debug shell in the terminal it was started from.

Both frontends are built on **LunaP**, a small Avalonia toolkit that lives in its own repository, [RedQuE3n/EmuSen.LunaP](https://github.com/RedQuE3n/EmuSen.LunaP).

## Debugging: DianaOS and Pharaoh

DianaOS is a Unix-style shell inside the emulator. It has pipes, variables, redirection, loops and a small set of coreutils, and commands for the hardware (`mem`, `regs`, `watch`, `disasm`, `bp`, `cov`, `coretop`). They work the same way on every core, coprocessors included.

```sh
watch add VRAM 3B8 4 write
mem VRAM 0 100 | head -4
for id in 1 2 3; do watch log $id 20; done > /tmp/watches.txt
```

Pharaoh runs the same core with no window, driven by a script of those commands plus frame stepping, input and screenshots. That's how most bugs here get reproduced, and how a change is shown to leave every other game's picture and sound untouched.

```sh
dotnet run --project EmuSen.Pharaoh -- game.smc 3000 --commands script.txt --out run.log
```

## Building

You need the **.NET 10 SDK**. **Rust** (`cargo`) is optional: without it the build warns and the N64 and Game Boy run on their C# cores.

The build takes LunaP from a checkout next to this one, on its `openemu-library` branch, so clone both side by side:

```sh
git clone https://github.com/RedQuE3n/EmuSen.git
git clone -b openemu-library https://github.com/RedQuE3n/EmuSen.LunaP.git
cd EmuSen
dotnet build

dotnet run --project EmuSen.Mistress                      # the main frontend
dotnet run --project EmuSen.Hotaru -- /path/to/game.sfc   # the console-first one
dotnet test EmuSen.WiseMan/EmuSen.WiseMan.csproj          # the test suite, all headless
```

To publish a self-contained build for the machine you're on:

```sh
dotnet publish EmuSen.Mistress/EmuSen.Mistress.csproj -c Release -r linux-x64 \
    --self-contained true -p:DebugType=none \
    -p:ErrorOnDuplicatePublishOutputFiles=false -o out/linux-x64
```

Ship the whole output folder. For another platform (`win-x64`, `osx-x64`, `osx-arm64`) the build doesn't cross-compile the Rust libraries, so take them from the [Rust cores workflow](.github/workflows/rust-cores.yml)'s artifacts and point the publish at them with `-p:EmuSenNativePrebuilt=/path/to/native`, where `/path/to/native/<rid>/` holds `marsrt` and `mercuryrt`. If a library is missing, the publish warns and that console falls back to its C# core. More in the [settings reference](EmuSen.DianaOS/DianaOS/Usr/Home/Documents/EmuSen%20Manual/EmuSen_Settings_Reference.md).

macOS builds are unsigned. On Apple Silicon, sign one yourself before it will run:

```sh
xattr -dr com.apple.quarantine Mistress.app
codesign --force --deep --sign - Mistress.app
```

### Platforms

Linux is where EmuSen is developed and played, on a desktop and on a Legion Go S. Whenever the Rust cores change, GitHub Actions builds the Rust libraries for Linux, Windows and macOS (Intel and Apple Silicon), and runs the tests that compare the Rust cores with the C# ones on Linux, Windows and Apple Silicon. Nobody has played on a Windows or macOS build yet.

## Documentation

The code keeps its comments to a line; the explanations live in the docs, which are readable on GitHub or from inside the emulator with `man` and `cat`.

- [The docs index](EmuSen.DianaOS/DianaOS/Etc/Man%20pages/README.md) is the place to start.
- [`EmuSen Manual/`](EmuSen.DianaOS/DianaOS/Usr/Home/Documents/EmuSen%20Manual/) has the project-level docs: the architecture overview, the debugging tools reference, the settings reference, the stack, the games list.
- [`Man pages/Hardware/`](EmuSen.DianaOS/DianaOS/Etc/Man%20pages/Hardware/) has notes for each console, including write-ups of the bugs found and fixed.

## The names

EmuSen is **Emu**lator **Sen**shi, after *Sailor Moon*. Each core is named for a character: Nintendo's consoles get the Sailor Guardians (Venus for the SNES, Moon for the NES, Mercury for the Game Boy, Mars for the N64) and other manufacturers get the villains. A Rust port adds `RT` to the name. The full list is in [`EmuSen_Core_Naming_Scheme.md`](EmuSen.DianaOS/DianaOS/Usr/Home/Documents/EmuSen%20Manual/EmuSen_Core_Naming_Scheme.md).

## ROMs

No ROMs are included, and none will be. Games have to come from legally obtained dumps.

## Credits and references

EmuSen's code is written for EmuSen. The projects below were read, run or measured against while writing it, and
are credited here whether or not their licence asks for it. Where one was used as a reference, no code was copied
from it: each console's docs say which reference settled which question.

**Libraries shipped in EmuSen's builds** (Avalonia, SkiaSharp, SDL 3, SQLite, Silk.NET, Cranelift and the rest) are
listed with their copyright holders and licences in [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md), with the
licence texts in [`licenses/`](licenses/). Every published build carries both.

### Hardware documentation

- [SNESdev wiki](https://snes.nesdev.org/), Martin Korth's fullsnes, and anomie's SNES documents
- [NESdev wiki](https://www.nesdev.org/wiki/)
- [Pan Docs](https://gbdev.io/pandocs/) and the gbdev community
- [n64brew wiki](https://n64brew.dev/)

### Reference emulators

Kept outside the repository, and read or run for comparison:

- [Mesen 2](https://github.com/SourMesen/Mesen2) and [MesenCE](https://github.com/nesdev-org/MesenCE) (GPL-3.0): the
  SNES and NES references, and an architecture reference. EmuSen's probe dumps Mesen's state to diff against.
- [Mupen64Plus](https://github.com/mupen64plus/mupen64plus-core) (GPL-2.0), [parallel-n64](https://github.com/libretro/parallel-n64)
  (GPL-2.0), [Project64](https://github.com/project64/project64) (GPL-2.0) and [RetroArch](https://github.com/libretro/RetroArch)
  (GPL-3.0): N64 references; the probe runs libretro cores.
- [angrylion-rdp-plus](https://github.com/ata4/angrylion-rdp-plus) (MAME licence), in
  [Themaister's fork](https://github.com/Themaister/parallel-rdp), and [parallel-rdp](https://github.com/Themaister/parallel-rdp)
  (MIT): the N64's display processor is graded against them.
- ares, bsnes, Snes9x, SameBoy, Gambatte, mGBA and CEN64, cited in the docs where they settle a question.
- [MiSTer-devel](https://github.com/mister-devel) FPGA cores, as referees where emulators disagree: [SNES_MiSTer](https://github.com/MiSTer-devel/SNES_MiSTer),
  [N64_MiSTer](https://github.com/MiSTer-devel/N64_MiSTer) and [Gameboy_MiSTer](https://github.com/MiSTer-devel/Gameboy_MiSTer) (GPL).

### Test suites

None is included in this repository; the tests read them from a local copy when one is configured.

- blargg's test ROMs (Shay Green)
- [mooneye-test-suite](https://github.com/Gekkio/mooneye-test-suite) (Joonas Javanainen)
- the [nes-test-roms](https://github.com/christopherpow/nes-test-roms) collection
- [SingleStepTests ProcessorTests](https://github.com/SingleStepTests/ProcessorTests) (Tom Harte and contributors)
- [n64-systemtest](https://github.com/lemmy-64/n64-systemtest) by lemmy-64 (MIT). Mars's RSP reciprocal tables are
  generated from the formulas its test file documents, which it ported from ares.
- PeterLemon's SNES test ROMs

### References for the Genesis core

Nephrite is the Sega Genesis / Mega Drive core, and Beryl-HW holds the chips the Sega cores share: the 68000, the
Z80 and the SN76489. Both were written from hardware documents and from measurements. No emulator's source code was
used, in whole or in part, and no firmware was read or disassembled. Where the documents were silent or disagreed,
the question went to a test program and then to a model of the chips run as a black box, whose outputs were compared
and whose logic was not read.

Every such question is an entry in a disputes log, with the documents read, the measurement taken and every file
opened: [`Nephrite_Disputes.md`](EmuSen.DianaOS/DianaOS/Etc/Man%20pages/Hardware/Sega/Nephrite%20-%20Genesis/Nephrite_Disputes.md) for the console and
[`Beryl_M68k.md`](EmuSen.DianaOS/DianaOS/Etc/Man%20pages/Hardware/Sega/Beryl-HW/Beryl_M68k.md) §6 for the 68000. The logs also record the few times excluded material was
displayed, and what was done about each. The rules themselves are in
[`Nephrite_Plan.md`](EmuSen.DianaOS/DianaOS/Etc/Man%20pages/Hardware/Sega/Nephrite%20-%20Genesis/Nephrite_Plan.md) §1.3, the full source list with where each document is thin in its §2,
and the build record in [`Nephrite_Native.md`](EmuSen.DianaOS/DianaOS/Etc/Man%20pages/Hardware/Sega/Nephrite%20-%20Genesis/Nephrite_Native.md).

**Sega's manuals**

- Sega, *Genesis Technical Overview* (version 1.00) and *Genesis Technical Bulletins*, as hosted by
  [Sega Retro](https://segaretro.org/Mega_Drive_official_documentation). Used for the hardware as Sega stated it,
  the Z80's control and the precautions for its bus among them, each use cited by manual and section. These are developer documents that circulated without Sega's release.

**Processor manuals and timing**

- Motorola, [*M68000 Family Programmer's Reference Manual*](https://cache.nxp.com/docs/en/reference-manual/M68000PRM.pdf)
  (M68000PM/AD) and [*M68000 8-/16-/32-Bit Microprocessors User's Manual*](https://cache.nxp.com/docs/en/reference-manual/MC68000UM.pdf)
  (M68000UM/AD): the 68000's instructions, exceptions and bus cycles.
- *Yacht.txt* ("Yet Another Cycle Hunting Table"), from Nemesis's documentation folder,
  [archived](https://web.archive.org/web/20250216033108id_/http://nemesis.hacking-cult.org/MegaDrive/Documentation/Yacht.txt):
  the order of each instruction's bus cycles.
- Zilog, *Z80 CPU User Manual* (UM0080), and Sean Young,
  [*The Undocumented Z80 Documented*](http://www.z80.info/zip/z80-documented.pdf), version 0.91 (2005): the Z80's
  instructions, timing and undocumented opcodes and flags, with boo_boo and Vladimir Kladov's published notes on
  MEMPTR.

**The sound chips**

- Yamaha, *YM2608 (OPNA) Application Manual*, the *YM2608 data sheet* and the *YM3438 (OPN2C) data sheet*, in the
  scans of [archive.org's `yamaha-chip-jpn`](https://archive.org/details/yamaha-chip-jpn), with an English
  translation of the application manual. No YM2612 manual is public; the YM2608's is the nearest, and gave the FM
  registers' meanings, the LFO's speeds and depths, SSG-EG, the timers and CSM.
- Matthew Gambrell and Olli Niemitalo,
  [*OPLx decapsulated*](https://docs.google.com/document/d/18IGx18NQY_Q1PJVZ-bHywao9bhsDoAqoIn1rIm42nwo) (2008):
  its two printed tables, the YM3812's log-sine and exponent ROMs, are the operator tables Nephrite computes with.
  [`Nephrite_OperatorTables.md`](EmuSen.DianaOS/DianaOS/Etc/Man%20pages/Hardware/Sega/Nephrite%20-%20Genesis/Nephrite_OperatorTables.md) gives the formulas and how the tables were later
  checked entry by entry.
- Maxim and contributors, [*SN76489*](https://www.smspower.org/Development/SN76489), SMS Power!: the PSG's latch,
  counters, noise and volume steps, which `beryl-sn76489` was written from
  ([`Beryl_SN76489.md`](EmuSen.DianaOS/DianaOS/Etc/Man%20pages/Hardware/Sega/Beryl-HW/Beryl_SN76489.md)).
- Nemesis and others, ["New Documentation: An authoritative reference on the YM2612"](https://gendev.spritesmind.net/forum/viewtopic.php?t=386),
  SpritesMind forum topic 386 (2008 to 2017). Used as prose: its authors' findings and their own tables on the
  envelope, the phase generator, SSG-EG, CSM, the status register, the test register and the DAC. Program code
  quoted in its posts was excluded.
- Kabuto (TiTAN), [*SEGA Mega Drive / Genesis hardware notes*](https://docs.google.com/document/d/e/2PACX-1vQ6CEUtCM3U6KKFD3i4lbMaF8muGQ5dez27OcpZOWRP2GgJviklr9rLIQut-LdoGfU4InazlIcbCvlG/pub),
  version 1.5 (2017): measurements of the DAC's levels and of the two models' output circuits, which the ladder
  effect and the output stage were held to.

**The console**

- Charles MacDonald, *Sega Genesis hardware notes* (`gen-hw.txt`, version 0.8, 2001), *Sega Genesis VDP documentation*
  (`genvdp.txt`, version 1.5f, 2000), his H and V counter tables (`m5hvc.txt`) and his EEPROM notes, from a
  [mirror of his site](http://dreamjam.co.uk/emuviews/txt/): the memory map, the I/O ports, the Z80's bus, the VDP's
  registers and the EEPROM boards.
- Rick McTeague, *Sega Genesis Hardware Internals* (`genhw.txt`, revised 1993), hosted on the same site: how a Game
  Genie sits on the cartridge's bus.
- Charles MacDonald, *Sega Genesis I/O Chip and Peripherals* (2007), with Plutiedev's
  [Sega multitap](https://plutiedev.com/sega-multitap), [EA multitap](https://plutiedev.com/ea-multitap) and
  [Peripheral ID](https://plutiedev.com/peripheral-id) pages, and Sega's *Genesis Technical Bulletin #16* (1993): the
  Team Player's packet and the 4 Way Play's selection of its four pads. Sega's own I/O check program for the adapter,
  run as a test program, reads Nephrite's Team Player as the bulletin lays its results out.
- [Plutiedev](https://plutiedev.com/): registers, DMA, controllers and multitaps, the SSF2 mapper, lock-on, TMSS, the
  cartridge header and save RAM.
- Eke-Eke, *Serial EEPROMs in Sega Genesis / Mega Drive cartridges*, version 2 (2010), and d0nut and Felipe XnaK,
  [*The complete documentation about Genesis ROM format*](https://www.zophar.net/documents/genesis/genesis-rom-format.html),
  version 1.1 (1998): the EEPROM boards' wiring and the copier formats.
- Nemesis's research on the VDP's FIFO, DMA and sprite masking, in SpritesMind topics and the pages archived from his
  site, and the [MegaDrive Wiki](https://md.railgun.works/): access slots and the bus's refresh.

**Recordings of consoles**

- Artemio Urbina and contributors, [MDFourier](https://junkerhq.net/MDFourier/), "MegaDrive/Sega Genesis and Sega CD
  recordings 2020-06-14": 21 consoles playing the 240p Test Suite's MDFourier sequence. Nephrite plays the same
  sequence, and its output circuit, ladder effect and PSG level were fitted to and graded against them.
- Nemesis's recording of his CSM test on a PAL model 1, the one FM test of his with a recording.

**Test programs**

None is included in this repository.

- Artemio Urbina, [240p Test Suite](https://artemiourbina.itch.io/240p-test-suite) for the Mega Drive, version 1.32
  (GPL-2.0): pictures that stand still, compared between Nephrite and the references, and the MDFourier sequence.
- Nemesis's test programs: VDPFIFOTesting (the VDP port access test, whose 122 tests Nephrite passes), the sprite
  masking and overflow test, the CRAM flicker test, the status-register and HV counter programs and six YM2612
  tests, from [Exodus's technical documentation](https://techdocs.exodusemulator.com/Console/SegaMegaDrive/Software.html)
  and the archive of his site. Their sources say what a console does.
- flamewing's BCD verifier (GPL-3.0), r57shell's opcode sizes test (MIT), Charles MacDonald's illegal-instruction,
  memory, V counter and window tests, TiTAN's Overdrive demos and the programs gathered in Genesis Plus GX's
  `md_test` archive (MacDonald, TmEE, Paul Lee).
- [SingleStepTests](https://github.com/SingleStepTests): the [`m68000`](https://github.com/SingleStepTests/m68000)
  and [`z80`](https://github.com/SingleStepTests/z80) suites (MIT) and Tom Harte's
  [`680x0`](https://github.com/SingleStepTests/680x0). Their data grade `beryl-m68k` and `beryl-z80` case by case,
  registers, memory and bus cycles; the programs that generated them were not read. Where the two 68000 suites
  disagree, [`Beryl_M68k.md`](EmuSen.DianaOS/DianaOS/Etc/Man%20pages/Hardware/Sega/Beryl-HW/Beryl_M68k.md) §6 settles each class.
- Frank Cringle's ZEXDOC and ZEXALL, for the Z80.

**Cheat codes**

- [*Sega Genesis Game Genie Conversion Method*](https://www.segakore.fr/media/segakore/outils/md_gg_codec/md_gg_conv_method.txt),
  a text after Merlyn LeRoy's postings, and Sega Retro's pages
  [Game Genie (Mega Drive)](https://segaretro.org/Game_Genie_(Mega_Drive)) and
  [Action Replay (Mega Drive)](https://segaretro.org/Action_Replay_(Mega_Drive)): the two code forms. No decoder
  program was read.

**Referees, run and not read**

- [Nuked-MD](https://github.com/nukeykt/Nuked-MD) by nukeykt, in its FPGA form in
  [MegaDrive_MiSTer](https://github.com/MiSTer-devel/MegaDrive_MiSTer) (GPL-2.0): a Genesis board transcribed from
  the chips' die images. It was compiled with Verilator and run as a whole board, with programs written for the
  purpose, and its pins, bus and picture were compared with Nephrite's.
- [fx68k](https://github.com/ijor/fx68k) by Jorge Cwik, from
  [MegaCD_MiSTer](https://github.com/MiSTer-devel/MegaCD_MiSTer) (GPL-3.0): a 68000 derived from the processor's
  microcode, run the same way beside Nuked-MD's 68000 to referee the two single-step suites.

Their logic was not read and nothing was transcribed from them. The lines that were opened to wire the benches
(port lists and instance names) are listed in the logs. A model is not a console: where a rule rests on the board
alone, its entry says so.

**Reference emulators, run and never read**

- [Genesis Plus GX](https://github.com/ekeeke/Genesis-Plus-GX) by Eke-Eke,
  [PicoDrive](https://github.com/notaz/picodrive) by notaz and contributors,
  [BlastEm](https://www.retrodev.com/blastem/) by Michael Pavone and
  [ClownMDEmu](https://github.com/Clownacy/clownmdemu) by Clownacy, as libretro cores from the buildbot. EmuSen's
  probe runs them headlessly and dumps their memory, picture and sound for comparison with Nephrite's. Their source
  was not opened.

**The other cores' sources** are listed with their own pages:
[`VenusRT_Plan.md`](EmuSen.DianaOS/DianaOS/Etc/Man%20pages/Hardware/Nintendo/Venus%20-%20SNES/VenusRT_Plan.md) §2 for the SNES port,
[`Mars_References.md`](EmuSen.DianaOS/DianaOS/Etc/Man%20pages/Hardware/Nintendo/Mars%20-%20N64/Mars_References.md) for the Nintendo 64, and
[`EmuSen_CRT.md`](EmuSen.DianaOS/DianaOS/Usr/Home/Documents/EmuSen%20Manual/EmuSen_CRT.md) §10 for the modelled CRT filter.

### Screen filters

The built-in filters are EmuSen's own shaders; the values they use came from these:

- CRT: Timothy Lottes' crt-lottes, public domain, via libretro's slang-shaders
- CRT (the modelled one): no shader's code; its constants are from SMPTE 170M, ITU-R BT.470 and BT.1886, the consoles'
  clocks as the nesdev, SNESdev and n64brew wikis document them, Markus Kuhn's measured P22 decay, DisplayMate's
  measurements of a Sony PVM-20L5, Sony's PVM brochure, and macro photographs by Selçuk Oral and Planemad on
  Wikimedia Commons, which its mask and scanline proportions were fitted to. `EmuSen_CRT.md` cites each.
- Game Boy: palettes measured by [SameBoy](https://github.com/LIJI32/SameBoy) (Lior Halphon, MIT), the panel response of
  Harlequin's dot-matrix shader (Harlequin and Matt Akins, GPL-3.0), and SameBoy's MonoLCD shadow
- Game Boy Color and Advance: colour matrices measured by Pokefan531 (public domain), stripes after fishku's
  authentic_gbc (CC0), and the stripe layout of [mGBA](https://github.com/mgba-emu/mgba)'s agb001 (MPL-2.0)

### Design

- [OpenEmu](https://openemu.org/): Mistress's library is modelled on its layout
- [EmulationStation](https://github.com/Aloshi/EmulationStation) and [ES-DE](https://es-de.org/) (MIT): Mistress's pad
  controls follow their button layout, and big-picture mode reads ES-DE's documented theme format. ES-DE's own
  behaviour was measured to match its timing; its code was not read.

### Downloaded only when the player asks

None of these is included in the repository or a build:

- game information and media from [ScreenScraper](https://www.screenscraper.fr/)
- the [Art Book Next](https://github.com/anthonycaccese/art-book-next-es-de) theme by Anthony Caccese (CC BY-NC-SA 2.0)
- RetroArch's [slang shaders](https://github.com/libretro/slang-shaders) and the
  [libretro-database](https://github.com/libretro/libretro-database) cheat codes
- [OpenVGDB](https://github.com/OpenVGDB/OpenVGDB) and [libretro-thumbnails](https://thumbnails.libretro.com/) box art

Each keeps its own licence and terms; see section 2 of [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md).

### Trademarks

Nintendo, Super Nintendo Entertainment System, Nintendo Entertainment System, Game Boy, Game Boy Color and
Nintendo 64 are trademarks of Nintendo. Sega, Genesis and Mega Drive are trademarks of Sega. Sailor Moon and its characters are the creation of Naoko Takeuchi. Steam and
SteamOS are trademarks of Valve, and Legion Go is a trademark of Lenovo. EmuSen is not affiliated with or endorsed by
any of them.

EmuSen is licensed under the GPL-3.0 ([`LICENSE`](LICENSE)), so anything built on it stays open.
