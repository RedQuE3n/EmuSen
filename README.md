# EmuSen

EmuSen is an emulation framework in the spirit of RetroArch: emulator cores sit behind one stable, versioned
interface, and frontends, tools and the shell talk to every core the same way. Accuracy comes first. A core is
graded against the console's own test programs and against other implementations run side by side, and anything that
trades accuracy for speed is a setting the player turns on, never the default.

EmuSen started as a personal project in C#. The cores have been moving to Rust one at a time since September 2026,
and new cores are written in Rust from the start. Around them are our own frontends (Mistress, with a Big Picture
mode that reads ES-DE themes, and the lighter Hotaru), a Unix-style shell called DianaSh, and a headless harness that
nearly all of the testing runs through.

**Licence:** GPL-3.0 · **Platforms:** Linux, Windows, macOS · **Consoles:** NES, SNES, Game Boy and Game Boy Color,
Nintendo 64, Sega Genesis / Mega Drive, Sega CD, Sony PlayStation

## Download

The current release is **[0.9.1](https://github.com/RedQuE3n/EmuSen/releases/tag/0.9.1)**, a pre-release for Linux
x64 (`EmuSen-0.9.1-linux-x64.tar.gz`) and Windows x64 (`EmuSen-0.9.1-win-x64.zip`), with `SHA256SUMS.txt` beside
them. Unpack the whole folder and run `bin/EmuSen` on Linux, or `bin\EmuSen.cmd` on Windows. Both builds are
self-contained. 0.9.0 was withdrawn: it showed a black PlayStation screen on handhelds with an older Mesa, and its
Windows build lacked the platform library. 0.9.1 replaces it with both fixed. There is no macOS build yet.

## The consoles

| Console | Engines | Where it stands |
|---|---|---|
| Nintendo 64 | **MarsRT** (Rust); the C# core it replaced was retired on 2026-10-16 | Super Mario 64, Ocarina of Time, GoldenEye 007, Donkey Kong 64 and Perfect Dark run, along with many others that haven't had full testing. Up to 4× internal resolution, antialiasing, an optional Vulkan renderer for the display processor, and a Cranelift recompiler. Donkey Kong 64 runs at full speed at 3× on a Legion Go S |
| SNES | **VenusRT** (Rust); the C# core it replaced was retired on 2026-10-16 | VenusRT is a new SNES core written from scratch, the default since 2026-10-05 and the only SNES engine since 2026-10-16. It runs SA-1, SuperFX, NEC DSP and OBC1 cartridges; Cx4, S-DD1, SPC7110 and the other special chips are not supported. Several games play well; none has a verified playthrough |
| Game Boy / Color | **MercuryRT** (Rust); the C# core it replaced was retired on 2026-10-16 | Everything but the boot ROM, the link cable and the Super Game Boy. 126 of the 173 blargg and mooneye hardware test ROMs pass under the Auto model |
| NES | **MoonRT** (Rust); the C# core it replaced was retired on 2026-10-16 | CPU, PPU, full APU and sixteen mapper boards. 127 of AccuracyCoin's 144 tests pass. Plays Super Mario Bros. 3 and others, with far less play-testing than the SNES |
| Sega Genesis / Mega Drive, Sega CD | **Nephrite** (Rust) | The Genesis is offered to players since 2026-10-07. Commercial cartridges run with battery and EEPROM saves, cheat codes, the three- and six-button pads, Sega's Team Player and Electronic Arts' 4 Way Play for four players, and Codemasters' J-Cart. The Sega CD is offered since 2026-10-16, on Nephrite's own open BIOS, with its backup RAM in the Memory Cards window. None has a verified playthrough. The 32X is in development and not offered yet |
| Sony PlayStation | **Diamond** (Rust) | New since the last release, offered to players since 2026-10-11. Boots discs on its own open BIOS. Crash Bandicoot, Spyro the Dragon, Ape Escape, Rayman, Tekken 3, Silent Hill, Castlevania: Symphony of the Night and Mega Man Legends have been played into their games; none has a verified playthrough. Optional enhancements: precise geometry, internal resolution up to 4×, true colour and texture filtering |

Every other console is an empty, reserved folder. A per-game list for the C# SNES and NES cores (the SNES's kept as history since Venus (C#) was retired) is in
[`EmuSen_Games_Tested.md`](EmuSen.DianaSh/DianaSh/Usr/Home/Documents/EmuSen%20Manual/EmuSen_Games_Tested.md).

Where a console has two engines, the choice is the Engine row of its tab in Graphics Settings, and the C# engine also
runs wherever the Rust library is missing. The plan for each port, and why the cores are moving to Rust while
the rest stays in C# for now, is in
[`EmuSen_Stack.md`](EmuSen.DianaSh/DianaSh/Usr/Home/Documents/EmuSen%20Manual/EmuSen_Stack.md).

### How the cores are written

The three newest cores, VenusRT, Nephrite and Diamond, are clean-room work. They are written from hardware
documents, manufacturers' manuals and test programs. Other emulators and FPGA cores are run as black boxes, to
compare memory, pictures and sound, and their source is never opened. Where the documents are silent or disagree, the
question goes into a disputes log with what was read and what was measured to settle it:
[`VenusRT_Disputes.md`](EmuSen.DianaSh/DianaSh/Etc/Man%20pages/Hardware/Nintendo/Venus%20-%20SNES/VenusRT_Disputes.md),
[`Nephrite_Disputes.md`](EmuSen.DianaSh/DianaSh/Etc/Man%20pages/Hardware/Sega/Nephrite%20-%20Genesis/Nephrite_Disputes.md)
and [`Diamond_Disputes.md`](EmuSen.DianaSh/DianaSh/Etc/Man%20pages/Hardware/Sony/Diamond%20-%20PlayStation/Diamond_Disputes.md).
The shared CPU crates under Beryl-HW (the Sega cores) and BlackMoon-HW (the Sony cores) follow the same rules.

The older cores were held to a looser rule, and their pages say so. MarsRT, MercuryRT and MoonRT are ports of our own
C# cores, checked against them frame by frame. Those C# cores were written from documents and graded by test ROMs,
but Mars's pages record where another implementation was read to understand a mechanism
([`Mars_References.md`](EmuSen.DianaSh/DianaSh/Etc/Man%20pages/Hardware/Nintendo/Mars%20-%20N64/Mars_References.md)),
and the Game Boy and NES cores were refereed against MiSTer FPGA cores whose RTL was read for single disputed rules.
Venus, the first SNES core, took parts from Mesen early in its development. That is why it was replaced rather
than repaired: VenusRT was written by people who do not read Venus, and Venus (C#) was removed from the source on
2026-10-16 ([`VenusRT_Plan.md`](EmuSen.DianaSh/DianaSh/Etc/Man%20pages/Hardware/Nintendo/Venus%20-%20SNES/VenusRT_Plan.md) §1.1,
[`VenusRT_Native.md`](EmuSen.DianaSh/DianaSh/Etc/Man%20pages/Hardware/Nintendo/Venus%20-%20SNES/VenusRT_Native.md) §68).

### Firmware

EmuSen never ships, downloads or asks for firmware. Where a console runs code its maker put inside the machine or the
cartridge (a boot ROM, a BIOS, a coprocessor's program), the core carries its own replacement written from public
documentation of what that code does, so the games it supports start with no firmware folder. The PlayStation boots on
Diamond's open kernel, the Sega CD on Nephrite's open BIOS, and the SNES's sound boot program and most of its DSP
chips have open replacements in VenusRT. A player who has dumped their own firmware can put it in `home/Firmware`, and
the core uses it instead as the exact path; it is optional, and nothing prompts for it. The Firmware page in Settings
shows, for each system, what is running and what the open replacement changes. Two gaps remain: SNES games on the
DSP-3 and ST011 chips need the player's own copy of the chip's program, and the Sega CD's open BIOS does not yet save
to a backup RAM cartridge. The policy is
[`EmuSen_Firmware.md`](EmuSen.DianaSh/DianaSh/Usr/Home/Documents/EmuSen%20Manual/EmuSen_Firmware.md).

## What changed in 0.9.1

- **Fixes.** The handheld's black PlayStation screen is gone. It was a deadlock in older Mesa drivers, now avoided by
  asking Mesa to keep the window's drawable, and the Vulkan picture path is on by default again. The Windows build
  ships the Rust platform library, with the Windows and macOS defects CI found fixed, among them cue sheets on file
  systems that ignore case.
- **The Sega CD** is offered to players, on Nephrite's open BIOS. It saves to backup RAM, shown in the Memory Cards
  window, and the drive fix ends the long waits some games had while loading. The CRT and Lottes filters are offered
  for it.
- **MarsRT** runs on the core ABI v1, with Controller Paks as save media in the Memory Cards window.
- **MercuryRT** reached stage 9: HDMA held during HALT, and the slowdown from its hardware audit found and fixed.
- **DianaSh** runs its Rust shell by default, with the C# shell as an automatic failover. It is not yet tested on
  Windows or macOS.
- **Windows that fit** at 1280x800 and 1920x1200 again.

## What changed in 0.9.0, since 0.5a

0.9.0, on 2026-10-15, was the first release since 0.5a on 2026-08-02, which was an SNES emulator with a debugging
shell. Between them:

- **New consoles.** The Game Boy and Game Boy Color (Mercury), the Nintendo 64 (Mars), the Genesis (Nephrite) and the
  PlayStation (Diamond), and the NES core (Moon) brought up to sixteen boards and a full APU.
- **The Rust cores.** MarsRT, a port of the N64 core with its threaded display processor, a Cranelift recompiler and
  a Vulkan path, is the N64's default. VenusRT, a clean rewrite of the SNES core with its own sound boot program and
  DSP replacements, is the SNES default. MercuryRT reached parity with the C# Game Boy core (stage 8), and an audit of
  the FPGA referee's findings, in eleven steps, fixed rules in both engines and took the hardware test corpus from 94
  to 126 passes. MoonRT runs the NES on either engine, with its CPU, PPU and APU audited against NES_MiSTer.
- **Diamond**, the PlayStation core: the R3000A and GTE in the shared BlackMoon crates, a cycle-faithful software GPU,
  the SPU, the MDEC, the CD-ROM controller with CD-DA and XA audio, pads, the multitap and memory cards. It boots on its
  own open BIOS and debugs like every other core. Its enhancements run on a Vulkan renderer of its own: precise
  geometry with crack fill, internal resolution from 1× to 4×, true colour and texture filtering. Memory cards can be
  per game, shared between games, or named cards.
- **Nephrite**, the Genesis core, with the 68000, Z80 and SN76489 in the shared Beryl crates, a YM2612 fitted to
  measurements of real consoles, multitaps, the J-Cart, lock-on and EEPROM boards. The Sega CD's hardware is built
  behind an attachment interface and is in development; the 32X is a stub.
- **A stable core ABI.** Version 1 of `emusen_core.h`, a C interface with additive minor versions, now at 1.9: discs
  and the lid, discs streamed from where they lie, read watchpoints, removable and console save files, a seek the
  host can read ahead of, and a picture handed over on the host's Vulkan device. A conformance kit (cases C1 to C20,
  with a corpus form) checks any core against it, and a guard in CI fails any change that breaks version 1. Cores are
  found by a sidecar file beside the library, so starting a frontend runs no core code.
- **Shared crates.** `emusen-native` (the Rust side of the ABI), `emusen-disc` (cue sheets, CHD, ISO and lone BIN
  images, read from any backing), `emusen-gpu` (the Vulkan compute device MarsRT and Diamond draw on) and
  `emusen-mips`.
- **Saves.** One save-media model for every console: a game's own battery files, removable media such as memory cards
  and the Sega CD's backup RAM cartridge, and memory the console keeps for every game. A Memory Cards window manages the
  slots, and a state load or a rewind leaves a shared card as it stands.
- **Rewind** takes two snapshots a second, and opens a strip of pictures to pick a moment from.
- **The picture.** On Linux the game's picture is drawn on one Vulkan device and handed to the compositor without a
  readback, and the built-in filters are slang presets matched to their old SkSL shaders. A modelled CRT
  filter comes in Accurate, Balanced and Performance tiers, with each console's own signal (the NES's composite among
  them), PAL as a standard of its own, and per-console overscan and screen shape. RetroArch's slang presets can be
  downloaded on request and chosen per console.
- **Mistress.** A library laid out like OpenEmu's, with the Game Boy Color on its own shelf; resume where a game was
  left; collections; a Big Picture mode that loads ES-DE themes and scrapes from ScreenScraper; full controller
  navigation, including on SteamOS's Game Mode; up to four players on their own pads; pads recognised when plugged in
  after start or replugged; sync to the display's refresh; and a controller bindings window that draws each console's
  pad. The published program is now called EmuSen.
- **DianaSh**, the shell (renamed from DianaOS), on its way to being a standalone Unix shell whose commands are
  separate modules. Hotaru became its `hotaru` command, run from the `emusen-shell` program.
- **The platform in Rust.** Path handling (Galaxia), input and audio (Endymion) and the slang shader runtime
  (Serenity) have Rust implementations in one platform library, used by default where it is present, with the C#
  behind them.
- **The LunaP kit.** The first gates of moving the interface toolkit to Rust: Mistress's interface recorded as a
  specification (212 recordings, identical run to run), and three of the kit's assumptions checked on the desktop.

## Work in progress

These are being built now and are not finished:

- **The Sega CD** is offered to players on Nephrite's open BIOS
  ([`Nephrite_OpenBios.md`](EmuSen.DianaSh/DianaSh/Etc/Man%20pages/Hardware/Sega/Nephrite%20-%20Genesis/Nephrite_OpenBios.md)).
  Still open: Sonic CD's opening film is black on the open BIOS, the open BIOS does not save to a backup RAM cartridge,
  and a Sega CD `.chd` is recognised only in a folder named `segacd`.
- **The 32X** is a stub behind the same attachment interface.
- **MercuryRT** is the Game Boy's only engine since Mercury (C#) was retired on 2026-10-16, graded by its own goldens
  and the blargg and mooneye test ROMs.
- **MoonRT** is the NES's only engine since Moon (C#) was retired on 2026-10-16, graded by its own goldens, the NES
  test-ROM corpus and AccuracyCoin. The CRT filter's composite NES path, which read Moon (C#)'s raw picture, waits on
  the core interface's raw picture output.
- **VenusRT** is the SNES's only engine since Venus (C#) was retired on 2026-10-16, graded by its own goldens and the
  SNES test-ROM corpus. (This line said until then that Venus (C#) was kept until VenusRT had had four weeks as the
  default with no open regression; that gate was proposed by `VenusRT_Plan.md` and never decided.) A battery run on
  the handheld is still owed.
- **Diamond** does not read `.pbp` images yet and has no rumble.
- **The picture on the Vulkan device** is used on Linux; Windows and macOS still draw it the older way, with a
  readback.
- **DianaSh** runs its Rust shell by default, with the C# shell as its failover until the Rust one has been run on
  Windows and macOS. More of its commands are still to move into separate modules.
- **The LunaP kit** has its hardware checks (Android, iOS, the Mac and the handheld outside gamescope) and its toolkit
  prototype (K2) ahead of it.

## Mistress, the frontend

Mistress is the main app. Its library is laid out like OpenEmu's: a sidebar of consoles, with the games as a grid of
covers or a list. It is built to be driven from a controller as well as a keyboard. In SteamOS's Game Mode, settings
windows open as sheets inside the main window, and every control, cheats and an on-screen keyboard included, can be
reached with the pad.

**Big Picture** is the full-screen mode for a television or a handheld. It reads ES-DE's theme format, so ES-DE themes
load as they are, and downloads the Art Book Next theme when the player asks. Game information and media come from
ScreenScraper. Plan and record:
[`EmuSen_BigPicture.md`](EmuSen.DianaSh/DianaSh/Usr/Home/Documents/EmuSen%20Manual/EmuSen_BigPicture.md).

Other things it does:

- save states, and rewind as a strip of pictures to pick a moment from
- per-console graphics settings: the engine, the N64's resolution, the Game Boy model, the PlayStation's
  enhancements, screen filters, overscan and screen shape
- a Shaders window with the built-in filters next to RetroArch's slang presets, which are downloaded only on request
- per-console controller bindings, hotkeys and cheats, and a cheat database downloaded on request
- a Firmware page and a Memory Cards window

**Hotaru** is the lighter frontend: the `hotaru` command of the EmuSen shell program, `emusen-shell`. It opens a ROM's
game window while the shell keeps its prompt in the terminal it was started from, or in a window of its own when there
is no terminal. `emusen-shell game.sfc` starts it on a ROM.

Both are built on **LunaP**, a small Avalonia toolkit that lives in its own repository,
[RedQuE3n/EmuSen.LunaP](https://github.com/RedQuE3n/EmuSen.LunaP). Its move to a Rust toolkit of its own is planned in
[`EmuSen_LunaP_Kit.md`](EmuSen.DianaSh/DianaSh/Usr/Home/Documents/EmuSen%20Manual/EmuSen_LunaP_Kit.md).

## The framework

- **The core ABI**, `emusen_core.h`, is the one way into a core: version 1 with minor versions that only add. A core
  describes itself (its systems, settings, controllers, memory spaces, save files and firmware), and a frontend builds
  its menus from that description. Reference:
  [`EmuSen_CoreAPI.md`](EmuSen.DianaSh/DianaSh/Usr/Home/Documents/EmuSen%20Manual/EmuSen_CoreAPI.md).
- **The conformance kit**, `emusen-core-conform`, runs a core library through every rule of the ABI on a game image or
  a whole corpus, and writes the sidecar a frontend discovers the core by.
- **The shared crates** under `EmuSen/Cores/Shared/`: `emusen-native`, `emusen-disc`, `emusen-gpu` and `emusen-mips`
  ([disc](EmuSen.DianaSh/DianaSh/Etc/Man%20pages/Hardware/Shared/EmuSen_Disc.md),
  [GPU](EmuSen.DianaSh/DianaSh/Etc/Man%20pages/Hardware/Shared/EmuSen_Gpu.md),
  [MIPS](EmuSen.DianaSh/DianaSh/Etc/Man%20pages/Hardware/Shared/EmuSen_Mips.md)).
- **Save media**: [`EmuSen_SaveMedia.md`](EmuSen.DianaSh/DianaSh/Usr/Home/Documents/EmuSen%20Manual/EmuSen_SaveMedia.md).
- **The platform library**, `emusen_platform`, under `Platform/`: the parts above the cores moving to Rust
  ([`EmuSen_RustPlatform.md`](EmuSen.DianaSh/DianaSh/Usr/Home/Documents/EmuSen%20Manual/EmuSen_RustPlatform.md)).

## Debugging: DianaSh and Pharaoh

DianaSh is a Unix-style shell inside the emulator. It has pipes, variables, redirection, loops and a small set of
coreutils, and commands for the hardware (`mem`, `regs`, `watch`, `disasm`, `bp`, `cov`, `coretop`). They work the same
way on every core, coprocessors included, because every core answers the same debug interface. Its commands' manual
pages are real `man` entries. Where it is going:
[`EmuSen_DianaSh.md`](EmuSen.DianaSh/DianaSh/Usr/Home/Documents/EmuSen%20Manual/EmuSen_DianaSh.md).

```sh
watch add VRAM 3B8 4 write
mem VRAM 0 100 | head -4
for id in 1 2 3; do watch log $id 20; done > /tmp/watches.txt
```

Pharaoh runs the same core with no window, driven by a script of those commands plus frame stepping, input and
screenshots. That's how most bugs here get reproduced, and how a change is shown to leave every other game's picture
and sound untouched. The test suite, EmuSen.WiseMan, runs headless the same way.

```sh
dotnet run --project EmuSen.Pharaoh -- game.smc 3000 --commands script.txt --out run.log
```

## Building

You need the **.NET 10 SDK** and **Rust** (`cargo`). Without cargo the build warns and builds no core:
every console's engine is a Rust library, so the frontends start and no game runs.

The build takes LunaP from a checkout next to this one, on its `openemu-library` branch, so clone both side by side:

```sh
git clone https://github.com/RedQuE3n/EmuSen.git
git clone -b openemu-library https://github.com/RedQuE3n/EmuSen.LunaP.git
cd EmuSen
dotnet build

dotnet run --project EmuSen.Mistress                      # the main frontend
dotnet run --project EmuSen.Shell -- /path/to/game.sfc    # the shell program, running `hotaru` on the ROM
dotnet test EmuSen.WiseMan/EmuSen.WiseMan.csproj          # the test suite, all headless
```

To publish a self-contained build for the machine you're on:

```sh
dotnet publish EmuSen.Mistress/EmuSen.Mistress.csproj -c Release -r linux-x64 \
    --self-contained true -p:DebugType=none \
    -p:ErrorOnDuplicatePublishOutputFiles=false -o out/linux-x64
```

The output folder is the program's whole tree: the launcher at `bin/EmuSen` (`bin\EmuSen.cmd` on Windows), the
program and its libraries under `lib/EmuSen/` (the Windows program is `lib\EmuSen\EmuSen.exe`), and the licences
beside them. Ship the whole folder. The shell program
publishes the same way, from `EmuSen.Shell/EmuSen.Shell.csproj`, with its launcher at `bin/emusen-shell`.

For another platform (`win-x64`, `osx-x64`, `osx-arm64`) the build doesn't cross-compile the Rust libraries. Take them
from the artefacts of the [Rust cores workflow](.github/workflows/rust-cores.yml) and point the publish at them with
`-p:EmuSenNativePrebuilt=/path/to/native`, where `/path/to/native/<rid>/` holds each library of
`EmuSen/Cores/RustCores.props` with its `.core.json` sidecar. A missing library is a warning: the console falls back
to its C# core, or is absent if it has none. The same folder takes `emusen_platform`, which is optional while every
part of it still has its C# behind it. More in the
[settings reference](EmuSen.DianaSh/DianaSh/Usr/Home/Documents/EmuSen%20Manual/EmuSen_Settings_Reference.md).

macOS builds are unsigned. On Apple Silicon, sign one yourself before it will run:

```sh
xattr -dr com.apple.quarantine EmuSen.app
codesign --force --deep --sign - EmuSen.app
```

### Platforms and requirements

We develop and test on Linux, on a desktop and on a Lenovo Legion Go S running SteamOS. Windows and macOS builds come
from the same tree, and CI runs the Rust cores' tests and their comparisons with the C# cores on both, but they get
far less play.

A 64-bit x86 processor (or Apple Silicon) is needed. The N64's and the PlayStation's graphics-card renderers,
RetroArch's shader presets and the Linux picture path use Vulkan. Without a Vulkan device the games still run: the
cores draw on the processor, the picture is drawn with Skia, and what needs a graphics card (the PlayStation's
enhancements, the N64's Vulkan renderer, RetroArch's presets) is not offered.

## Documentation

The code keeps its comments to a line; the explanations live in the docs, which can be read on GitHub or from inside
the emulator with `man` and `cat`.

- [CONTRIBUTING.md](CONTRIBUTING.md) is for anyone joining: the repositories and branches, a reading order through the
  manual, and the conventions every change is held to.
- [The docs index](EmuSen.DianaSh/DianaSh/Etc/Man%20pages/README.md) is the place to start.
- [`EmuSen Manual/`](EmuSen.DianaSh/DianaSh/Usr/Home/Documents/EmuSen%20Manual/) has the project-level docs: the
  architecture, the core ABI, the debugging tools reference, the settings reference, the stack, saves, firmware, the
  picture and the games list.
- [`Man pages/Hardware/`](EmuSen.DianaSh/DianaSh/Etc/Man%20pages/Hardware/) has each console's pages, from the plan
  written before the first line of code to the build record, the disputes and the write-ups of bugs found and fixed.

## The names

EmuSen is **Emu**lator **Sen**shi, after *Sailor Moon*. Each core is named for a character: Nintendo's consoles get
the Sailor Guardians (Venus for the SNES, Moon for the NES, Mercury for the Game Boy, Mars for the N64) and other
manufacturers get the villains (Nephrite for the Genesis, Diamond for the PlayStation). A Rust port adds `RT` to the
name. The full list is in
[`EmuSen_Core_Naming_Scheme.md`](EmuSen.DianaSh/DianaSh/Usr/Home/Documents/EmuSen%20Manual/EmuSen_Core_Naming_Scheme.md).

## ROMs

No ROMs, disc images or firmware are included, and none will be. Games have to come from legally obtained dumps.

## Credits and references

EmuSen's code is written for EmuSen, with the one exception described under
[How the cores are written](#how-the-cores-are-written): parts of Venus (C#) came from Mesen. The projects below were
read, run or measured against while writing it, and are credited here whether or not their licence asks for it. Each
console's pages say which reference settled which question.

**Libraries shipped in EmuSen's builds** (Avalonia, SkiaSharp, SDL 3, SQLite, Silk.NET, Cranelift, the Rust crates of
every core and the rest) are listed with their copyright holders and licences in
[`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md), with the licence texts in [`licenses/`](licenses/). Every
published build carries both.

### Hardware documentation

- [SNESdev wiki](https://snes.nesdev.org/), Martin Korth's fullsnes, and anomie's SNES documents
- [NESdev wiki](https://www.nesdev.org/wiki/)
- [Pan Docs](https://gbdev.io/pandocs/) and the gbdev community, and Gekkio's *Game Boy: Complete Technical Reference*
- [n64brew wiki](https://n64brew.dev/)
- the Sega and PlayStation documents listed under their cores below

### Reference emulators

Kept outside the repository, and read or run for comparison:

- [Mesen 2](https://github.com/SourMesen/Mesen2) and [MesenCE](https://github.com/nesdev-org/MesenCE) (GPL-3.0): the
  SNES and NES references, and an architecture reference. EmuSen's probe dumps Mesen's state to diff against; for
  VenusRT it is run as a black box only.
- [Mupen64Plus](https://github.com/mupen64plus/mupen64plus-core) (GPL-2.0), [parallel-n64](https://github.com/libretro/parallel-n64)
  (GPL-2.0), [Project64](https://github.com/project64/project64) (GPL-2.0) and [RetroArch](https://github.com/libretro/RetroArch)
  (GPL-3.0): N64 references; the probe runs libretro cores.
- [angrylion-rdp-plus](https://github.com/ata4/angrylion-rdp-plus) (MAME licence), in
  [Themaister's fork](https://github.com/Themaister/parallel-rdp), and [parallel-rdp](https://github.com/Themaister/parallel-rdp)
  (MIT): the N64's display processor is graded against them.
- ares, bsnes, Snes9x, SameBoy, Gambatte, mGBA and CEN64, cited in the docs where they settle a question.
- [MiSTer-devel](https://github.com/mister-devel) FPGA cores, as referees where emulators disagree:
  [SNES_MiSTer](https://github.com/MiSTer-devel/SNES_MiSTer), [N64_MiSTer](https://github.com/MiSTer-devel/N64_MiSTer),
  [Gameboy_MiSTer](https://github.com/MiSTer-devel/Gameboy_MiSTer) and [NES_MiSTer](https://github.com/MiSTer-devel/NES_MiSTer) (GPL).
- The Genesis's and the PlayStation's references, run and never read, are listed under their cores below.

### Test suites

None is included in this repository; the tests read them from a local copy when one is configured.

- blargg's test ROMs (Shay Green), for the NES, the Game Boy and the SNES's SPC700 and S-DSP
- [mooneye-test-suite](https://github.com/Gekkio/mooneye-test-suite) (Joonas Javanainen)
- the [nes-test-roms](https://github.com/christopherpow/nes-test-roms) collection
- AccuracyCoin by 100thCoin (Chris Siebert), 144 scored NES tests in one image
- [SingleStepTests ProcessorTests](https://github.com/SingleStepTests/ProcessorTests) (Tom Harte and contributors),
  among them the 65816, SPC700 and R3000 suites
- [n64-systemtest](https://github.com/lemmy-64/n64-systemtest) by lemmy-64 (MIT). Mars's RSP reciprocal tables are
  generated from the formulas its test file documents, which it ported from ares.
- PeterLemon's SNES and PlayStation test ROMs
- [gilyon's snes-tests](https://github.com/gilyon/snes-tests) (MIT): every 65C816 and SPC700 opcode
- [undisbeliever's snes-test-roms](https://github.com/undisbeliever/snes-test-roms) (zlib, MIT for the INIDISP set)
- absindx's SA-1 tests, and the anomie- and byuu-era SNES timing tests of the higan collection

### References for the Genesis core

Nephrite is the Sega Genesis / Mega Drive core, and Beryl-HW holds the chips the Sega cores share: the 68000, the
Z80 and the SN76489. Both were written from hardware documents and from measurements. No emulator's source code was
used, in whole or in part, and no firmware was read or disassembled. Where the documents were silent or disagreed,
the question went to a test program and then to a model of the chips run as a black box, whose outputs were compared
and whose logic was not read.

Every such question is an entry in a disputes log, with the documents read, the measurement taken and every file
opened: [`Nephrite_Disputes.md`](EmuSen.DianaSh/DianaSh/Etc/Man%20pages/Hardware/Sega/Nephrite%20-%20Genesis/Nephrite_Disputes.md) for the console and
[`Beryl_M68k.md`](EmuSen.DianaSh/DianaSh/Etc/Man%20pages/Hardware/Sega/Beryl-HW/Beryl_M68k.md) §6 for the 68000. The logs also record the few times excluded material was
displayed, and what was done about each. The rules themselves are in
[`Nephrite_Plan.md`](EmuSen.DianaSh/DianaSh/Etc/Man%20pages/Hardware/Sega/Nephrite%20-%20Genesis/Nephrite_Plan.md) §1.3, the full source list with where each document is thin in its §2,
and the build record in [`Nephrite_Native.md`](EmuSen.DianaSh/DianaSh/Etc/Man%20pages/Hardware/Sega/Nephrite%20-%20Genesis/Nephrite_Native.md).

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
  [`Nephrite_OperatorTables.md`](EmuSen.DianaSh/DianaSh/Etc/Man%20pages/Hardware/Sega/Nephrite%20-%20Genesis/Nephrite_OperatorTables.md) gives the formulas and how the tables were later
  checked entry by entry.
- Maxim and contributors, [*SN76489*](https://www.smspower.org/Development/SN76489), SMS Power!: the PSG's latch,
  counters, noise and volume steps, which `beryl-sn76489` was written from
  ([`Beryl_SN76489.md`](EmuSen.DianaSh/DianaSh/Etc/Man%20pages/Hardware/Sega/Beryl-HW/Beryl_SN76489.md)).
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
  run as a test program, reads Nephrite's Team Player as the bulletin lays its results out. Sega Retro's
  [J-Cart](https://segaretro.org/J-Cart) page for which cartridges carry Codemasters' two extra ports; how they are
  read was measured from the games themselves, Micro Machines 2's own J-Cart test among them.
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
  sequence, and its output circuit, ladder effect and PSG level were fitted to and graded against them; the set's
  Sega CDs gave the Sega CD's PCM filter and its PCM and CD-DA levels against the Genesis.
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
  disagree, [`Beryl_M68k.md`](EmuSen.DianaSh/DianaSh/Etc/Man%20pages/Hardware/Sega/Beryl-HW/Beryl_M68k.md) §6 settles each class.
- Frank Cringle's ZEXDOC and ZEXALL, for the Z80.
- krikzz, [mcd-verificator](https://github.com/krikzz/MEGA-PRO) V1.02 (GPL-3.0): the Sega CD's hardware tests,
  run in the place of the Sega CD's boot ROM with no BIOS. Nephrite passes all eighteen. Its source says what each
  test expects; krikzz's notes and the logs it recorded of a console's drive (not those recorded under a BIOS) gave
  the drive's protocol and timing.
- Artemio Urbina, the 240p Test Suite for the Sega CD, version 1.32 (GPL-2.0): a whole disc, data track, pause and
  audio tracks, for the drive.

**The Sega CD**

- Sega, *Mega-CD Hardware Manual* (version 1.0, 1991), as hosted by
  [Sega Retro](https://segaretro.org/Mega-CD_official_documentation): the map, the gate array's registers, PRG-RAM,
  Word RAM, the interrupts, the timers, the CDC's wiring, the graphics ASIC and the fader, each use cited by section.
  A developer document that circulated without Sega's release.
- Sega, *Mega-CD BIOS Manual*, *Mega-CD Disc Format Specifications*, *Mega-CD Software Development Manual*, the
  *Sega-CD Technical Bulletins* and the *Sega-CD Software Standards*, from the same collection: the BIOS's call
  interface, the boot, the disc's system area, the main CPU's jump table and the memory maps, which Nephrite's open
  Sega CD BIOS is written from
  ([`Nephrite_OpenBios.md`](EmuSen.DianaSh/DianaSh/Etc/Man%20pages/Hardware/Sega/Nephrite%20-%20Genesis/Nephrite_OpenBios.md)).
  No BIOS's code, listing, disassembly, trace or log was read by its writer; a player's BIOS was run only as a black
  box, its games compared after the hand-over.
- The 240p Test Suite's Sega CD include file (`cdbios.asm`, Luke Usher's, in the suite's GPL-2.0 sources), screened
  before it was read: the BIOS's function numbers and the sub CPU's entry addresses, which the Mega-CD BIOS Manual
  leaves to Sega's include files. Each number is confirmed by a program's call on the open BIOS
  ([`Nephrite_Disputes.md`](EmuSen.DianaSh/DianaSh/Etc/Man%20pages/Hardware/Sega/Nephrite%20-%20Genesis/Nephrite_Disputes.md) D-41). The suite's start-up and system
  program sources, for what a program expects of the calls.
- [*bram-filesystem*](https://github.com/1hedu/emuTOS-MCD/blob/HEAD/docs/bram-filesystem.md), in the emuTOS-MCD
  project's documentation: the backup RAM volume's footer and its layout on the odd bytes, measured on a console's
  volume (its author first took the footer from an emulator's formatting function, as the page says).
- Plutiedev, [Using the Z80](https://plutiedev.com/using-the-z80): the Z80's reset and bus, as the open BIOS leaves
  them.
- Sanyo, *LC8950 Real-Time Error Correction and Host Interface Processor Design Manual*, revised for the LC8951,
  as hosted by Sega Retro: the CDC's registers, decoder (its monitor-only mode among them) and host transfer.
- Sanyo, *LC7883* data sheet, as hosted by Sega Retro: CD-DA's digital filter, de-emphasis and attenuator.
- Sega, *Mega-CD Hardware Manual*'s "PCM Sound Source" section, and Ricoh's *RF5C68A* data sheet (Console5): the
  RF5C164's registers, address counters, wave format and mixing.
- Microsoft's RIFF WAVE format, as commonly documented, for a cue sheet's audio tracks in `emusen-disc`.
- Plutiedev, [Sub CPU in mode 1](https://plutiedev.com/subcpu-in-mode1): the version register's bit 5 reading 0 with
  the Sega CD attached; and its page on the Mega-CD's LEDs.
- A drive probe written for the purpose (`tools/cddprobe/` in the crate), assembled with GNU binutils for m68k, run as
  the Sega CD's boot ROM in Nephrite and in the reference emulators below to record what the drive reports.

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

### References for the PlayStation core

Diamond is the PlayStation core, and BlackMoon-HW holds the Sony cores' shared processors: the R3000A and the GTE.
They were written under the same rules as Nephrite, made stricter in one place: the FPGA referee is run only as a
black box, in a dispute step too. No emulator's source, no PGXP or other enhancement code, no firmware and no
listing or trace of firmware was read, and OpenBIOS is not used, since its own documentation says it was written from a
disassembly of a retail BIOS. The open BIOS kernel was written under a further rule that keeps its writers away from
anything quoting BIOS code. The rules are in
[`Diamond_Plan.md`](EmuSen.DianaSh/DianaSh/Etc/Man%20pages/Hardware/Sony/Diamond%20-%20PlayStation/Diamond_Plan.md) §1.3, its
sources and where they are thin in its §2, what was pinned in
[`Diamond_References.md`](EmuSen.DianaSh/DianaSh/Etc/Man%20pages/Hardware/Sony/Diamond%20-%20PlayStation/Diamond_References.md),
and the build record in
[`Diamond_Native.md`](EmuSen.DianaSh/DianaSh/Etc/Man%20pages/Hardware/Sony/Diamond%20-%20PlayStation/Diamond_Native.md).

**Documents**

- [psx-spx](https://github.com/psx-spx/psx-spx.github.io), Martin Korth's (nocash) PlayStation specifications, in the maintained
  conversion's `stable` branch (pinned), with nocash's single-file edition. Its maintainers say it draws on Sony's
  confidential documents; its "BIOS Patches" chapter and the CD controller's microcontroller chapters were excluded.
- IDT, *R30xx Family Software Reference Manual* and *MIPS Microprocessor Family Software Reference Manual* (1996), and
  LSI Logic, *CW33000 MIPS Embedded Processor User's Manual* (1992): MIPS I, coprocessor 0, delay slots,
  exceptions and cache isolation.
- Ecma International, ECMA-130 (CD-ROM) and ECMA-119 (ISO 9660).
- Sony's CXD2510Q, CXD1199AQ and CXD2545Q data sheets, and Sony's service manuals for the SCPH-5500, SCPH-7500,
  SCPH-9000 and SCPH-100 consoles, for the video clocks.

**Test programs**

- [ps1-tests](https://github.com/JaCzekanski/ps1-tests) by Jakub Czekański (MIT), with their console logs and VRAM
  pictures, and the timing programs among them rewritten to record in RAM
  ([`Diamond_Records.md`](EmuSen.DianaSh/DianaSh/Etc/Man%20pages/Hardware/Sony/Diamond%20-%20PlayStation/Diamond_Records.md)).
- amidog's `psxtest_cpu`, `psxtest_cpx`, `psxtest_gte` and `psxtest_gpu`, used locally.
- Peter Lemon's PlayStation programs and pictures.
- SingleStepTests' `r3000` suite; its data grade `blackmoon-r3000`, and its generator was not read.
- The PSn00bSDK examples, read as sources of what a program expects.

**Referees, run and never read**

- [DuckStation](https://github.com/stenzek/duckstation) and [PCSX-Redux](https://github.com/grumpycoders/pcsx-redux),
  release binaries driven headless, and Beetle PSX, PCSX-ReARMed and SwanStation as libretro cores through the probe
  ([`Diamond_Referees.md`](EmuSen.DianaSh/DianaSh/Etc/Man%20pages/Hardware/Sony/Diamond%20-%20PlayStation/Diamond_Referees.md)).
- [PSX_MiSTer](https://github.com/MiSTer-devel/PSX_MiSTer) and Laxer3a's PS-GPU and PS-GTE, pinned to be simulated
  as black boxes with GHDL; not yet run
  ([`Diamond_FPGA_Referees.md`](EmuSen.DianaSh/DianaSh/Etc/Man%20pages/Hardware/Sony/Diamond%20-%20PlayStation/Diamond_FPGA_Referees.md)).

### The other cores' sources

Listed with their own pages:
[`VenusRT_Plan.md`](EmuSen.DianaSh/DianaSh/Etc/Man%20pages/Hardware/Nintendo/Venus%20-%20SNES/VenusRT_Plan.md) §2 for the SNES,
with [`VenusRT_DspHle.md`](EmuSen.DianaSh/DianaSh/Etc/Man%20pages/Hardware/Nintendo/Venus%20-%20SNES/VenusRT_DspHle.md) for the DSP replacements and
[`Venus_Referee.md`](EmuSen.DianaSh/DianaSh/Etc/Man%20pages/Hardware/Nintendo/Venus%20-%20SNES/Venus_Referee.md) for the FPGA referee;
[`Mars_References.md`](EmuSen.DianaSh/DianaSh/Etc/Man%20pages/Hardware/Nintendo/Mars%20-%20N64/Mars_References.md) for the Nintendo 64;
[`Mercury_Referee.md`](EmuSen.DianaSh/DianaSh/Etc/Man%20pages/Hardware/Nintendo/Mercury%20-%20GB-GBC/Mercury_Referee.md) and
[`Mercury_Native.md`](EmuSen.DianaSh/DianaSh/Etc/Man%20pages/Hardware/Nintendo/Mercury%20-%20GB-GBC/Mercury_Native.md) §13 for the Game Boy;
[`Moon_Native.md`](EmuSen.DianaSh/DianaSh/Etc/Man%20pages/Hardware/Nintendo/Moon%20-%20NES/Moon_Native.md) §3.7 for the NES's audit against NES_MiSTer;
[`EmuSen_CRT.md`](EmuSen.DianaSh/DianaSh/Usr/Home/Documents/EmuSen%20Manual/EmuSen_CRT.md) §10 for the modelled CRT filter; and
[`EmuSen_LunaP_Kit.md`](EmuSen.DianaSh/DianaSh/Usr/Home/Documents/EmuSen%20Manual/EmuSen_LunaP_Kit.md) §12 for the toolkit survey behind the LunaP kit.

### Disc images: CHD and chdman

- [MAME](https://www.mamedev.org/)'s CHD (Compressed Hunks of Data) format, read through the `chd` crate (Ronny Chan,
  BSD-3-Clause), whose code is not read. Its CD layout was measured from the output of MAME's chdman 0.289; chdman's
  source file is BSD-3-Clause (copyright Aaron Giles), and MAME as a whole is GPL-2.0-or-later
  ([The MAME License](https://docs.mamedev.org/license.html)). chdman is documented in MAME's own
  [chdman page](https://docs.mamedev.org/tools/chdman.html) (MAME Documentation 0.289).
- [MAME](https://www.mamedev.org/)'s **chdman**, shipped with EmuSen since 2026-10-25 and run by its CHD tool (`chd`
  in DianaSh) to write and verify CHD files. It is built unmodified from MAME 0.289's source (tag `mame0289`), following
  MAME's "Compiling MAME", and pinned by SHA-256; MAME's source is not read. chdman's source file is BSD-3-Clause
  (copyright Aaron Giles) and MAME as a whole GPL-2.0-or-later ([The MAME License](https://docs.mamedev.org/license.html)).
  Its manual is MAME's [chdman page](https://docs.mamedev.org/tools/chdman.html); its licences, the libraries inside it
  and the written offer of its source are in [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md) §1.7
  (`EmuSen_GamePackages.md` §4.9, §12).

### Screen filters

The built-in filters are EmuSen's own shaders; the values they use came from these:

- CRT: Timothy Lottes' crt-lottes, public domain, via libretro's slang-shaders
- CRT (the modelled one): no shader's code; its constants are from SMPTE 170M, ITU-R BT.470 and BT.1886, the consoles'
  clocks as the nesdev, SNESdev and n64brew wikis document them, Markus Kuhn's measured P22 decay, DisplayMate's
  measurements of a Sony PVM-20L5, Sony's PVM brochure, and macro photographs by Selçuk Oral and Planemad on
  Wikimedia Commons, which its mask and scanline proportions were fitted to. `EmuSen_CRT.md` cites each. Its signal
  model was refereed by running patchy-ntsc and crt-beans beside it.
- Game Boy: palettes measured by [SameBoy](https://github.com/LIJI32/SameBoy) (Lior Halphon, MIT), the panel response of
  Harlequin's dot-matrix shader (Harlequin and Matt Akins, GPL-3.0), and SameBoy's MonoLCD shadow
- Game Boy Color and Advance: colour matrices measured by Pokefan531 (public domain), stripes after fishku's
  authentic_gbc (CC0), and the stripe layout of [mGBA](https://github.com/mgba-emu/mgba)'s agb001 (MPL-2.0)

### Design

- [OpenEmu](https://openemu.org/): Mistress's library is modelled on its layout
- [EmulationStation](https://github.com/Aloshi/EmulationStation) and [ES-DE](https://es-de.org/) (MIT): Mistress's pad
  controls follow their button layout, and Big Picture reads ES-DE's documented theme format. ES-DE's own
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
Nintendo 64 are trademarks of Nintendo. Sega, Genesis, Mega Drive, Sega CD, Mega-CD and 32X are trademarks of Sega.
Sony and PlayStation are trademarks of Sony Interactive Entertainment. Sailor Moon and its characters are the creation
of Naoko Takeuchi. Steam and SteamOS are trademarks of Valve, and Legion Go is a trademark of Lenovo. EmuSen is not
affiliated with or endorsed by any of them.

EmuSen is licensed under the GPL-3.0 ([`LICENSE`](LICENSE)), so anything built on it stays open.
