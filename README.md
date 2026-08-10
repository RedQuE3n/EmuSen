# EmuSen

A multi-system emulator written from scratch in C# / .NET 10, with a Unix-like debugging shell built into it.

EmuSen is written against published hardware documentation rather than by porting an existing emulator. It is a working emulator, but it is a **hobby project in active development** — see [Status](#status) before expecting to play anything start to finish.

**License:** GPL-3.0 · **Platforms:** Linux, Windows, macOS · **Cores:** SNES, NES, Game Boy / Color

The emulator itself is C#. Two development tools beside it are not, and deliberately so: the reference probe is Rust and the dump comparator is Python. See [Development tooling](#development-tooling).

---

## Table of contents

- [What exists so far](#what-exists-so-far)
- [Status](#status)
- [The two frontends](#the-two-frontends)
- [DianaOS — the built-in debug shell](#dianaos--the-built-in-debug-shell)
- [Pharaoh — the headless harness](#pharaoh--the-headless-harness)
- [Architecture](#architecture)
- [Why the names?](#why-the-names)
- [Building and running](#building-and-running)
- [Tests](#tests)
- [Development tooling](#development-tooling)
- [Documentation](#documentation)
- [Roadmap](#roadmap)
- [ROMs](#roms)

---

## What exists so far

A cycle-budgeted SNES emulator — 65816 CPU, SPC700 + S-DSP audio, a full PPU, and the cartridge coprocessors — a younger NES core beside it, and the tooling built around making both *debuggable*. That tooling is the part that makes this project unusual:

- **DianaOS**, a genuine bash-alike shell embedded in the emulator, with pipes, variables, redirection, control flow, a coreutils subset, and hardware inspection commands (`mem`, `regs`, `watch`, `disasm`, `coretop`, …).
- **Pharaoh**, a headless, scriptable harness that runs the real core with no window and no human, so bugs can be reproduced deterministically and fixes proven byte-identical across the ROM library.

The emulation core is a pure library with no window, no `Main`, and no frontend knowledge. Everything else — presentation, frontends, debug tooling — sits above it and depends on it one-directionally.

The project is structured for more than one console, and **three cores are now wired end to end**: hand a frontend a `.smc`/`.sfc` and it builds the SNES core, a `.nes` and it builds the NES one, a `.gb`/`.gbc` and it builds the Game Boy one — each with its own debug target behind the same shell.

- **Venus (SNES)** — the mature core. Runs real commercial games, cartridge coprocessors included.
- **Moon (NES)** — CPU, PPU, a full APU and ten mapper boards. Younger and less play-tested than Venus, but no longer a skeleton. See `EmuSen/Cores/Nintendo/Moon - NES/README.md`.
- **Mercury (Game Boy / Game Boy Color)** — built in four phases between 4 and 9 August 2026 and now feature-complete: SM83, a cycle-granular bus, the full PPU, all four APU channels, colour, five cartridge boards, save states and a debug target. One core covers both DMG and CGB, and it is registered with the core factory.

Every other console is a reserved, empty folder.

The extra cores are the point of the architecture rather than a bonus: `IDebugTarget` had exactly one implementation for most of this project's life and now has three. `MoonDebugTarget` was the one that proved the interface was genuinely core-agnostic rather than SNES-shaped by accident; `MercuryDebugTarget` cost a fraction of it, which is the more useful result — the second implementation tests an abstraction, the third one just uses it. None of Mercury's four days went on a shell, a harness, a watch system or a frontend.

---

## Status

**Honest summary: the SNES core runs real commercial games, and several boot correctly and play, but very few have been verified end to end. The NES core is complete enough to run and make sound, and has had far less play-testing. The Game Boy core is feature-complete and the best instrumented of the three, but its timing claims are argued rather than demonstrated until the hardware test corpus is actually run against it.**

The rows down to *Timing accuracy* describe the SNES core; the other cores follow.

| Area | State |
|---|---|
| 65816 CPU | Full 256/256 opcode table, including decimal (BCD) mode and its documented `ADC`/`SBC` flag divergence |
| SPC700 + S-DSP | Full 256/256 opcodes, BRR decode, 4-tap Gaussian resampling, ADSR/GAIN, 8-voice mixing |
| Audio output | Working — SDL out, with a resampler and dynamic rate control to stop drift |
| PPU backgrounds | All 7 modes, 16×16 tiles, mosaic, Mode 7 + EXTBG, offset-per-tile, direct colour, pseudo and true hi-res |
| PPU sprites | Full OAM decode, real per-scanline 32-sprite / 34-sliver limits |
| Colour math / windowing | Implemented, including the fixed-colour and half-math hardware quirks |
| Memory mapping | LoROM and HiROM, general DMA + HDMA, header-driven SRAM sizing |
| Region / PAL | Real, not faked — region comes off the cartridge country byte and drives 312-scanline / 50 Hz timing and the `STAT78` bit together |
| Cartridge coprocessors | **SA-1** (second 65C816, Super MMC banking, arithmetic + bit-stream units), **SuperFX / GSU**, **NEC DSP** family (DSP-1/1B/2/3/4, ST010/ST011), **OBC1**. Verification differs per chip — see the note under this table |
| Save states | Working, via a reflective serializer, with a version header — a state from a newer build is rejected rather than misread |
| Rewind / fast-forward | Working, core-agnostic (XOR-delta chain) |
| Input | Rebindable keyboard + gamepad, per-game hotkeys, gamepad hot-plug |
| Timing accuracy | Scanline granularity, not per-dot. A deliberate, documented tradeoff |
| NES core (Moon) | CPU validated against SingleStepTests `nes6502/v1` (2,560,000 cases, final state *and* per-cycle bus traces); PPU renders backgrounds, sprites, sprite 0 and NMI; all five APU channels synthesize (two pulse, triangle, noise, DMC); ten mapper boards including MMC3. Scanline granularity, NTSC only |
| Game Boy core (Mercury) | SM83, interrupts, timer, joypad, a cycle-granular bus with access blocking, 160-cycle OAM DMA, HDMA stall, the full PPU, all four APU channels, CGB colour, five cartridge boards, a serial sink and save states. 226 headless tests. Per-scanline renderer under a per-cycle clock; no boot ROM, no real link, no Super Game Boy |
| Consoles beyond those three | Reserved folders only |

**Coprocessors, chip by chip.** The **SA-1** is the most solid — Kirby's Dream Land 3 and Kirby Super Star both play, and its one known gap is character-conversion DMA, which neither game requests. The **SuperFX / GSU** runs Yoshi's Island through its intro, title screen, file menu and into 1-1, and diffs to zero against the reference probe on instruction behaviour; what is left there is cycle cost, plus one open cosmetic defect — a strip of BG3 tiles in the intro whose VRAM upload has not been located. Six real GSU bugs were found and fixed getting that far, each written up in `Venus_SuperFX.md`. The **NEC DSP** interpreter covers all seven uPD7725/uPD96050 variants from one implementation, but **it cannot run without a firmware dump, and none is shipped** — a DSP game sitting on a loading screen is a missing dump, not an emulation bug. The **OBC1** is implemented and tested.

**Game compatibility** is tracked properly in [`EmuSen_Games_Tested.md`](EmuSen.DianaOS/DianaOS/Usr/Home/Documents/EmuSen%20Manual/EmuSen_Games_Tested.md), which rates each title Perfect → Unplayable and links to the investigation behind the rating. As of now nothing sits in Unplayable, several titles boot and play with known cosmetic bugs, and no title has a claimed full playthrough. Treat that file as the source of truth over this table — with the caveat that it *lags* actual core state, because a fix that moves a title up is not always written back the same day.

Verified builds: **Linux x64 is launch-tested.** Windows and macOS binaries are produced and structurally correct but have not been executed — there is no Windows or Mac on the build machine, and macOS builds are unsigned (Apple Silicon requires an ad-hoc signature before it will run them at all).

---

## The two frontends

Both are Avalonia applications, and neither owns any emulation logic. They share two libraries: **Serenity** presents the game picture, and **LunaP** is everything around it — the palette, the controls, the window scaffolding and a fluent layout surface, so a new screen is a constructor and a refresh method rather than another hand-built window.

**Mistress** — the fuller GUI, scoped as bug-testing tooling rather than a polished launcher. ROM picker and ROM browser, save/load state, rebindable keyboard + gamepad, emulator hotkeys, preferences, per-session file logging, and a menu entry that opens the DianaOS console against the running game.

**Hotaru** — a lighter, console-first frontend. Takes a ROM path on the command line and puts the DianaOS shell on the terminal it was launched from, with the game in its own window.

**Themes** are a drop-in file at `/etc/EmuSen/themes/<name>.<ext>`, written either as an `.axaml` `ResourceDictionary` or as `.css`, both spelling the same thing — overrides of whichever palette keys the theme cares about. Keys it does not mention keep their built-in value, and applying one repaints every open window live, with no restart.

A polished, EmulationStation-style launcher is explicitly *not* either of these; it is planned as a separate project, and LunaP exists partly so that project starts with a widget set rather than a blank page.

---

## DianaOS — the built-in debug shell

Reachable from Hotaru's terminal or Mistress's `Settings → DianaOS Console…`. It is a real shell, not a command prompt with a fixed verb list:

```sh
# what changed in VRAM while that tile was wrong?
watch add VRAM 3B8 4 write
watch summary 3

# is the tilemap sane, or is the tile data?
mem VRAM 0 100 | head -4
vramsheet /tmp/sheet.bmp

# ordinary shell things work
for id in 1 2 3; do watch log $id 20; done > /tmp/watches.txt
regs | grep -i vram
```

It has quoting, `$VAR` and `$(...)` expansion, pipes, `>`/`>>`/`<` redirection to real files, `;`/`&&`/`||`, `if`/`for`/`while`, history recall, `man` pages for every command, a coreutils subset (`ls`, `cd`, `grep`, `awk`, `sed`, `nano`, `find`, `xxd`, …), and a live `coretop` hardware dashboard. The filesystem it exposes is a sandboxed, Unix-shaped tree walled to the project directory.

Cartridge coprocessors are first-class here too, not a blind spot: `regs` reports the chip's own register file, its RAM and address space are ordinary memory spaces (`GSURAM`, `GSUBUS`, `SA1IRAM`, `BWRAM`, `SA1BUS`), `disasm` decodes each chip's own instruction set rather than assuming the 65816, `bp sa1` breaks on the second CPU, and `cov` records which code actually executed over a whole run — the answer to "does the game even reach this feature", which a breakpoint that never fires cannot give.

Crucially, `IDebugTarget` — the interface all of this talks to — is **core-agnostic**. It knows about memory spaces, registers, sprites, palettes, breakpoints and coverage, not about the SNES.

---

## Pharaoh — the headless harness

Runs the real core with no window, no audio device, and no human, driven by a script of the same commands the interactive shell accepts. This is how most debugging in this project actually happens.

```sh
dotnet run --project EmuSen.Pharaoh -- game.smc 3000 \
    --loadstate boss.state --commands script.txt --out run.log
```

A script interleaves frame-stepping, input, screenshots and debug commands freely:

```
frames 200
tap Start 8
hold Left
frames 520
release Left
screenshot /tmp/at_the_door.bmp
watch summary 3
framesum 600
```

Notable verbs: `waitstable` / `waitchange` / `waitvalue` (advance until the screen or a memory address settles), `contactsheet` (a grid of thumbnails over time), `autoshot` (capture every visually distinct state), `layers` (isolate BG1/BG2/BG3/OBJ), `vramsheet`, `paletteswatch`, `spriteoverlay`, `rewind`, `perf`, and `framesum` / `audiosum` — output-identity digests that let a renderer or audio change be proven pixel- and sample-identical across the whole ROM library, rather than eyeballed on one frame.

---

## Architecture

Layered bottom-to-top; each layer depends only on the ones below it.

| Project | Role |
|---|---|
| `EmuSen.Galaxia` | Every config file on disk — paths, atomic JSON persistence, the agnostic models. A leaf: no dependencies at all |
| `EmuSen` | The emulation core: CPU, PPU, APU, memory, save states, audio resampling. A pure library — no `Main`, no window |
| `EmuSen.DianaOS` | The shell, `IDebugTarget`, and every debug command. Core-agnostic |
| `EmuSen.Cauldron` | Small realtime-provider abstractions the debug layer polls |
| `EmuSen.Serenity` | Shared presentation: the Avalonia/Skia `GameFrameControl`, shader pipeline, graphics settings |
| `EmuSen.Endymion` | The SDL3 device layer: audio out, gamepad polling, pad bindings. Serenity's counterpart on the sound-and-input side |
| `EmuSen.LunaP` | The shared Avalonia toolkit: palette and themes, controls, window scaffolding, fluent layout, dashboards. References Avalonia, `EmuSen.Galaxia` and `EmuSen.Cauldron`, and nothing else |
| `EmuSen.Mistress` | The fuller Avalonia GUI frontend |
| `EmuSen.Hotaru` | The console-first Avalonia frontend |
| `EmuSen.Pharaoh` | The headless scripted harness, and the CLI runner for ground-truth CPU test vectors |
| `EmuSen.WiseMan` | The xUnit test suite |

The SNES core lives under `EmuSen/Cores/Nintendo/Venus - SNES/`, namespaced `EmuSen.Cores.Nintendo.Venus.*`, with reserved sibling folders for every other planned console.

The layering is enforced in practice, not just described. The PPU exposes a small `IWriteObserver` hook and has no idea a watchpoint exists; the debug layer implements that interface and supplies the meaning. And where a layer's boundary actually matters, a test holds it: `LeafAssemblyTests` asserts that Galaxia, Endymion, Serenity and LunaP reference what they are allowed to and never reach back into the core — which is what keeps a future launcher able to browse a library without loading an emulator.

**LunaP now has a consumer outside this repository.** It is published as a NuGet package, together with the two dependency-free leaves it names, for [EmuSen.Pegasus](https://github.com/RedQuE3n/EmuSen.Pegasus) — a collaborative notepad this project is developed through, which lived here until it needed nothing of EmuSen but the toolkit. That makes the layering rule a property of the artifact rather than a comment: a package cannot reach up into a core at all. It also means LunaP's public surface can be broken by a rename that no build in this repository will catch. `EmuSen_LunaP.md` §17 has the details.

A count worth reading as a design outcome rather than trivia: the solution has shrunk twice this year. `EmuSen.Crystal` and `EmuSen.Nehellania` were folded into the cores and into Endymion, because a project boundary has to buy somebody separability they are actually using.

---

## Why the names?

**EmuSen** = **Emu**lator **Sen**shi, after *Bishoujo Senshi Sailor Moon*. Every core is named for a character, grouped by manufacturer — Nintendo, as the project's origin point, gets the heroes; everyone else gets a villain faction.

| Manufacturer | Faction | Examples |
|---|---|---|
| Nintendo | Sailor Guardians | SNES = **Venus**, NES = **Moon**, GBA = **Jupiter** |
| Sega | Dark Kingdom | Genesis = **Beryl**, Dreamcast = **Kunzite** |
| Sony | Black Moon Clan | PS1 = **Diamond**, PS2 = **Sapphire** |
| Atari | Death Busters | 2600 = **Eudial**, Jaguar = **Cyprine & Ptilol** |
| Microsoft / NEC | Dead Moon Circus | Xbox = **CereCere**, PC Engine = **Tiger's Eye** |

Virtual Boy is **Saturn** — Guardian of Death and Destruction — which is a joke, not a coincidence.

Codenames govern folders and namespaces only. Classes and log output still say `Snes65816Disassembler` and `"SNES"`, because that is what the hardware is actually called. Full mapping in [`EmuSen_Core_Naming_Scheme.md`](EmuSen.DianaOS/DianaOS/Usr/Home/Documents/EmuSen%20Manual/EmuSen_Core_Naming_Scheme.md).

---

## Building and running

Requires the **.NET 10 SDK**. Nothing else — SDL3 ships with the build.

```sh
git clone https://github.com/RedQuE3n/EmuSen-Project.git
cd EmuSen-Project
dotnet build

# the fuller GUI
dotnet run --project EmuSen.Mistress

# the console-first frontend, ROM on the command line
dotnet run --project EmuSen.Hotaru -- /path/to/game.smc
```

### Publishing a standalone build

Per-RID, self-contained, no .NET install needed on the target machine:

```sh
dotnet publish EmuSen.Mistress/EmuSen.Mistress.csproj -c Release -r linux-x64 \
    --self-contained true -p:DebugType=none \
    -p:ErrorOnDuplicatePublishOutputFiles=false -o out/linux-x64
```

Swap `linux-x64` for `win-x64`, `osx-x64` or `osx-arm64`.

Two things worth knowing, both documented in [`EmuSen_Settings_Reference.md`](EmuSen.DianaOS/DianaOS/Usr/Home/Documents/EmuSen%20Manual/EmuSen_Settings_Reference.md) §4.9:

- **Ship the whole output folder.** The executable needs `libSDL3` and `DianaOSRoot/` beside it.
- **`-p:PublishSingleFile=true` is untested here.** It used to be an outright trap — Silk.NET could not resolve the SDL2 the single-file host extracted — and the SDL3 bindings no longer have that fault, but nothing has verified the rest of the stack (Avalonia, SkiaSharp) under single-file publishing. The folder recipe above is the supported one.

macOS builds cross-compiled from Linux are unsigned. Apple Silicon refuses to execute an unsigned arm64 binary outright, so the user must sign it themselves:

```sh
xattr -dr com.apple.quarantine Mistress.app
codesign --force --deep --sign - Mistress.app
```

---

## Tests

```sh
dotnet test EmuSen.WiseMan/EmuSen.WiseMan.csproj
```

2726 tests across 202 files: CPU and PPU hardware behaviour for all three cores, APU/DSP, the cartridge coprocessors and their debug exposure, audio sync and drain, save-state round-tripping, the DianaOS shell (lexer, parser, pipes, control flow, multi-line continuation, sandboxing, man pages), and Avalonia UI tests that drive real key events through a headless window.

Everything runs headless. No window is ever opened on anyone's screen, including for the UI tests — if the harness cannot express something, the harness gets extended rather than a real window launched.

The UI tests run against real Skia render passes rather than checking flags, because the failure mode that matters there is silent: a control whose style stopped matching renders as *nothing* and throws no error. `UiTest.AssertLaidOut` catches the flat-image case, and `EMUSEN_UI_BASELINE` records a pixel baseline from one commit and compares the next against it — which is how two visual defects were caught during the toolkit migration, one of them a window whose Close button had been sitting off the bottom edge for a long time.

Some of those are **property-based** (CsCheck), asserting laws rather than examples: that the rewind delta codec's `Apply` really is its own inverse, that resampled audio never leaves the range its input spanned, that a fuller audio queue never asks the resampler to speed up, and that `CanDecode` never promises a cheat-code decode that then throws. Each runs hundreds to thousands of generated cases and shrinks any failure to a minimal counterexample.

Beyond unit tests, the project leans on **output-identity digests**: `framesum` and `audiosum` over a fixed window across the whole local ROM library (48 titles at present), so a change to the renderer or the mixer can be shown to alter exactly the games it was meant to and nothing else.

The Game Boy core is graded differently on purpose. Its oracle is **not another emulator** — the Game Boy has something the SNES does not, a mature corpus of hardware tests written against real silicon that grade themselves and report the verdict down the link port. Mercury has the serial sink that captures those verdicts and the harness that reads them; actually running the corpus is the largest single piece of outstanding work on the core.

---

## Development tooling

The emulator is C#. Two tools beside it are not, and the boundaries are argued rather than incidental — see [`EmuSen_Stack.md`](EmuSen.DianaOS/DianaOS/Usr/Home/Documents/EmuSen%20Manual/EmuSen_Stack.md) §4 before proposing that anything move languages.

- **The reference probe is Rust** (`EmuSen.WiseMan/Reference/probe-rs/`). It is a policy layer plus a backend per emulator — including a libretro backend that drives any core — so it names no emulator in its own vocabulary. Built with `build-probe.sh <backend>`. Its job is to take ground-truth dumps of a running machine that EmuSen's own output can be diffed against; the GSU work above is what it is for.
- **The comparator is Python** (`EmuSen.WiseMan/Reference/analysis/`). Dump database, signature store, palette and audio analysis, `gsudiff.py`. Offline analysis, deliberately not in-process test code.
- **SQLite** backs the two databases that were already relational — the dump set and the game catalogue.

Config stays JSON and coverage maps stay bitsets; both are reasoned exceptions rather than oversights, documented in the same place.

---

## Documentation

This project documents heavily, and deliberately keeps rationale *out* of code comments and *in* man pages. Code comments are one line and point at a section.

- `EmuSen.DianaOS/DianaOS/Etc/Man pages/Hardware/` — per-console hardware notes. The SNES set (`Venus - SNES/`) covers CPU, PPU, APU, memory and each cartridge coprocessor, including full root-cause writeups for real bugs found and fixed — and, where a lead turned out to be wrong, the measurement that retired it, so the same ground does not get walked twice.
- `EmuSen.DianaOS/DianaOS/Usr/Home/Documents/EmuSen Manual/` — project-level docs: overview, debugging tools reference, save states, audio sync, rewind/fast-forward, settings, games tested, roadmap, the shared-toolkit reference, and the per-subsystem writeups (`EmuSen_Cauldron.md`, `EmuSen_Galaxia.md`, `EmuSen_LunaP.md`, …).
- `EmuSen.DianaOS/DianaOS/Etc/Man pages/README.md` — the index to all of the above. Start there rather than guessing a filename.

These are readable on GitHub, and also from inside the emulator via DianaOS's own `man` and `cat`.

---

## Roadmap

Near-term, roughly in order:

1. **Run the Game Boy hardware test corpus through Mercury's serial sink.** The sink and the harness exist; until the corpus actually runs, Mercury's timing claims are argued rather than demonstrated. Biggest single item on the board.
2. Finish verifying the games that currently boot but have not been played through, and write the results back into `EmuSen_Games_Tested.md`, which lags the cores.
3. Play-test the NES core against real games the way the SNES core has been.
4. Locate the missing VRAM upload behind the Yoshi's Island intro strip — the question is now narrow and concrete, and `Venus_SuperFX.md` §10 records exactly which measurement comes next.
5. Close the SuperFX gap that is left after instruction behaviour: cycle cost.
6. Character-conversion DMA on the SA-1 — unused by both Kirby titles, but a game that asks for it gets wrong tile data.
7. Real per-Player-2 input bindings (only a mirror-P1 toggle exists today).
8. **Make it run acceptably on a low-end x86-64 laptop.** An open effort since 2026-08-03, and a different question from raw throughput — `mainComposite` is the target, and the duplicated main/sub decode is the next win. Four other optimizations were measured and rejected; `Venus_PPU.md` §13.1 records them so they are not retried.
9. The Mesen-style multi-pane GUI debugger, built on `IDebugTarget` and LunaP.

Explicitly deferred: true doubled-resolution interlace, and per-dot H-position timing.

Consoles beyond the three are a naming and layout commitment, not a schedule.

---

## ROMs

**No ROMs are included, and none will be.** The local test library referenced in the docs is not in this repository and is excluded by `.gitignore`. Supply your own legally-obtained dumps.

---

## Credits and licensing

Written against the [SNESdev wiki](https://snes.nesdev.org/) as the primary hardware reference. [MesenCE](https://github.com/nesdev-org/MesenCE) is consulted as an architecture reference where useful — both projects are GPL-3.0, so there is no compatibility issue, though implementations here are written rather than transcribed.

Licensed **GPL-3.0**, chosen so anything built on this stays open, matching how the project itself was built from openly-published documentation.
