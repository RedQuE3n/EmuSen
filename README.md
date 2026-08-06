# EmuSen

A multi-system emulator written from scratch in C# / .NET 10, with a Unix-like debugging shell built into it.

EmuSen is written against published hardware documentation rather than by porting an existing emulator. It is a working emulator, but it is a **hobby project in active development** — see [Status](#status) before expecting to play anything start to finish.

**License:** GPL-3.0 · **Platforms:** Linux, Windows, macOS · **Cores:** SNES, NES (in progress)

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
- [Documentation](#documentation)
- [Roadmap](#roadmap)
- [ROMs](#roms)

---

## What exists so far

A cycle-budgeted SNES emulator — 65816 CPU, SPC700 + S-DSP audio, a full PPU, and the cartridge coprocessors — a younger NES core beside it, and the tooling built around making both *debuggable*. That tooling is the part that makes this project unusual:

- **DianaOS**, a genuine bash-alike shell embedded in the emulator, with pipes, variables, redirection, control flow, a coreutils subset, and hardware inspection commands (`mem`, `regs`, `watch`, `disasm`, `coretop`, …).
- **Pharaoh**, a headless, scriptable harness that runs the real core with no window and no human, so bugs can be reproduced deterministically and fixes proven byte-identical across the ROM library.

The emulation core is a pure library with no window, no `Main`, and no frontend knowledge. Everything else — presentation, frontends, debug tooling — sits above it and depends on it one-directionally.

The project is structured for more than one console, and **two cores are now wired end to end**: hand a frontend a `.smc`/`.sfc` and it builds the SNES core, hand it a `.nes` and it builds the NES one, each with its own debug target behind the same shell.

- **Venus (SNES)** — the mature core. Runs real commercial games.
- **Moon (NES)** — CPU, PPU, a full APU and ten mapper boards. Younger and less play-tested than Venus, but no longer a skeleton. See `EmuSen/Cores/Nintendo/Moon - NES/README.md`.
- **Mercury (Game Boy / Game Boy Color)** — started 2026-08-04: the SM83, the bus and five cartridge boards. No PPU, no APU, deliberately not registered with the core factory yet. One core covers both DMG and colour.

Every other console is a reserved, empty folder.

That second wired core is the point of the architecture rather than a bonus: `IDebugTarget` had exactly one implementation for most of this project's life, and `MoonDebugTarget` is the first thing to prove the interface was genuinely core-agnostic rather than SNES-shaped by accident.

---

## Status

**Honest summary: the SNES core runs real commercial games, and several boot correctly and play, but very few have been verified end to end. The NES core is complete enough to run and make sound, and has had far less play-testing.**

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
| Cartridge coprocessors | **SA-1** (second 65C816, Super MMC banking, arithmetic + bit-stream units), **SuperFX / GSU**, **NEC DSP** family (DSP-1/1B/2/3/4, ST010/ST011), **OBC1**. Verification differs per chip — see the note under this table |
| Save states | Working, via a reflective serializer, with a version header — a state from a newer build is rejected rather than misread |
| Rewind / fast-forward | Working, core-agnostic (XOR-delta chain) |
| Input | Rebindable keyboard + gamepad, per-game hotkeys, gamepad hot-plug |
| Timing accuracy | Scanline granularity, not per-dot. A deliberate, documented tradeoff |
| NES core (Moon) | CPU validated against SingleStepTests `nes6502/v1` (2,560,000 cases, final state *and* per-cycle bus traces); PPU renders backgrounds, sprites, sprite 0 and NMI; all five APU channels synthesize (two pulse, triangle, noise, DMC); ten mapper boards including MMC3. Scanline granularity, NTSC only |
| Game Boy core (Mercury) | SM83, interrupts, timer, joypad, five cartridge boards, save states. No PPU or APU yet |
| Consoles beyond those three | Reserved folders only |

**Coprocessors, chip by chip.** The **SA-1** is the most solid — Kirby's Dream Land 3 and Kirby Super Star both play, and its one known gap is character-conversion DMA, which neither game requests. The **SuperFX** runs Yoshi's Island: it boots and plays through the intro, with one open cosmetic defect (a strip of 18 tiles the game never uploads). The **NEC DSP** interpreter covers all seven uPD7725/uPD96050 variants from one implementation, but **it cannot run without a firmware dump, and none is shipped** — a DSP game sitting on a loading screen is a missing dump, not an emulation bug. The **OBC1** is implemented and tested.

**Game compatibility** is tracked properly in [`EmuSen_Games_Tested.md`](EmuSen.DianaOS/DianaOS/Usr/Home/Documents/EmuSen%20Manual/EmuSen_Games_Tested.md), which rates each title Perfect → Unplayable and links to the investigation behind the rating. As of now nothing sits in Unplayable, several titles boot and play with known cosmetic bugs, and no title has a claimed full playthrough. Treat that file as the source of truth, not this table.

Verified builds: **Linux x64 is launch-tested.** Windows and macOS binaries are produced and structurally correct but have not been executed — there is no Windows or Mac on the build machine, and macOS builds are unsigned (Apple Silicon requires an ad-hoc signature before it will run them at all).

---

## The two frontends

Both are Avalonia applications, and neither owns any emulation logic. They share two libraries: **Serenity** presents the game picture, and **LunaP** is everything around it — the palette, the controls, the window scaffolding and a fluent layout surface, so a new screen is a constructor and a refresh method rather than another hand-built window.

**Mistress** — the fuller GUI, scoped as bug-testing tooling rather than a polished launcher. ROM picker and ROM browser, save/load state, rebindable keyboard + gamepad, emulator hotkeys, preferences, per-session file logging, and a menu entry that opens the DianaOS console against the running game.

**Hotaru** — a lighter, console-first frontend. Takes a ROM path on the command line and puts the DianaOS shell on the terminal it was launched from, with the game in its own window.

**Themes** are a drop-in file: a `ResourceDictionary` at `/etc/EmuSen/themes/<name>.axaml` overriding whichever palette keys it cares about. Keys it does not mention keep their built-in value, and applying one repaints every open window live, with no restart.

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
| `EmuSen.LunaP` | The shared Avalonia toolkit: palette and themes, controls, window scaffolding, fluent layout. References Avalonia and `EmuSen.Galaxia` and nothing else |
| `EmuSen.Mistress` | The fuller Avalonia GUI frontend |
| `EmuSen.Hotaru` | The console-first Avalonia frontend |
| `EmuSen.Pharaoh` | The headless scripted harness, and the CLI runner for ground-truth CPU test vectors |
| `EmuSen.WiseMan` | The xUnit test suite |

The SNES core lives under `EmuSen/Cores/Nintendo/Venus - SNES/`, namespaced `EmuSen.Cores.Nintendo.Venus.*`, with reserved sibling folders for every other planned console.

The layering is enforced in practice, not just described. The PPU exposes a small `IWriteObserver` hook and has no idea a watchpoint exists; the debug layer implements that interface and supplies the meaning. And where a layer's boundary actually matters, a test holds it: `LeafAssemblyTests` asserts that Galaxia, Endymion, Serenity and LunaP reference what they are allowed to and never reach back into the core — which is what keeps a future launcher able to browse a library without loading an emulator.

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

2205 tests across 168 files: CPU and PPU hardware behaviour for both cores, APU/DSP, the cartridge coprocessors and their debug exposure, audio sync and drain, save-state round-tripping, the DianaOS shell (lexer, parser, pipes, control flow, multi-line continuation, sandboxing, man pages), and Avalonia UI tests that drive real key events through a headless window.

The UI tests run against real Skia render passes rather than checking flags, because the failure mode that matters there is silent: a control whose style stopped matching renders as *nothing* and throws no error. `UiTest.AssertLaidOut` catches the flat-image case, and `EMUSEN_UI_BASELINE` records a pixel baseline from one commit and compares the next against it — which is how two visual defects were caught during the toolkit migration, one of them a window whose Close button had been sitting off the bottom edge for a long time.

Some of those are **property-based** (CsCheck), asserting laws rather than examples: that the rewind delta codec's `Apply` really is its own inverse, that resampled audio never leaves the range its input spanned, that a fuller audio queue never asks the resampler to speed up, and that `CanDecode` never promises a cheat-code decode that then throws. Each runs hundreds to thousands of generated cases and shrinks any failure to a minimal counterexample.

Beyond unit tests, the project leans on **output-identity digests**: `framesum` and `audiosum` over a fixed window across all 43 ROMs in the local library, so a change to the renderer or the mixer can be shown to alter exactly the games it was meant to and nothing else.

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

1. Finish verifying the games that currently boot but have not been played through.
2. Play-test the NES core against real games the way the SNES core has been.
3. Close out the SuperFX cosmetic defect in Yoshi's Island (18 tiles that never reach VRAM).
4. Character-conversion DMA on the SA-1 — unused by both Kirby titles, but a game that asks for it gets wrong tile data.
5. Real per-Player-2 input bindings (only a mirror-P1 toggle exists today).
6. Carry Mercury (Game Boy) up to a PPU and register it with the core factory.
7. The Mesen-style multi-pane GUI debugger, built on `IDebugTarget` and LunaP.

Explicitly deferred: true doubled-resolution interlace, and per-dot H-position timing.

---

## ROMs

**No ROMs are included, and none will be.** The local test library referenced in the docs is not in this repository and is excluded by `.gitignore`. Supply your own legally-obtained dumps.

---

## Credits and licensing

Written against the [SNESdev wiki](https://snes.nesdev.org/) as the primary hardware reference. [MesenCE](https://github.com/nesdev-org/MesenCE) is consulted as an architecture reference where useful — both projects are GPL-3.0, so there is no compatibility issue, though implementations here are written rather than transcribed.

Licensed **GPL-3.0**, chosen so anything built on this stays open, matching how the project itself was built from openly-published documentation.
