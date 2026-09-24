# EmuSen

EmuSen is a multi-console emulator built as a research project. The cores are written from published hardware documentation rather than ported from another emulator. The project started in C#, and the cores are now being rewritten in Rust one at a time. Each C# core stays behind as the reference its Rust port is checked against, frame by frame.

Around the cores there is a frontend, a debugging shell built into the emulator, and a headless harness that most of the testing runs through.

**License:** GPL-3.0 · **Platforms:** Linux, Windows, macOS · **Consoles:** SNES, NES, Game Boy / Game Boy Color, Nintendo 64

## What works today

| Console | Core | Where it stands |
|---|---|---|
| Nintendo 64 | **MarsRT** (Rust), the default; **Mars** (C#) | Super Mario 64, Ocarina of Time, GoldenEye 007 and Donkey Kong 64 run. Up to 4× internal resolution, antialiasing, an optional Vulkan path and a recompiler |
| SNES | **Venus** (C#) | Runs commercial games, including SA-1, SuperFX, NEC DSP and OBC1 cartridges. Several play well; none has a verified playthrough |
| Game Boy / Color | **Mercury** (C#), the default; **MercuryRT** (Rust) | Everything but the boot ROM, the link cable and the Super Game Boy. Passes 94 of the 173 blargg and mooneye hardware test ROMs |
| NES | **Moon** (C#) | CPU, PPU, full APU and sixteen mapper boards. Plays Super Mario Bros. 3 and others, but has had far less play-testing than the SNES core |

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

## Credits

The main hardware references are the [SNESdev wiki](https://snes.nesdev.org/), the NESdev wiki, Pan Docs and the n64brew wiki. [MesenCE](https://github.com/nesdev-org/MesenCE) is consulted as an architecture reference, and the code here is written rather than transcribed. EmuSen is GPL-3.0, so anything built on it stays open.
