# EmuSen

A multi-system emulator written from scratch in C# / .NET 10, with a Unix-like debugging shell built into it.

EmuSen is written against published hardware documentation rather than by porting an existing emulator. It is a working emulator, but it is a **hobby project in active development** — see [Status](#status) before expecting to play anything start to finish.

**License:** GPL-3.0 · **Platforms:** Linux, Windows, macOS · **Cores:** SNES

---

## Table of contents

- [What this is](#what-this-is)
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

A cycle-budgeted SNES emulator — 65816 CPU, SPC700 + S-DSP audio, and a full PPU — plus the tooling built around making it *debuggable*. That tooling is the part that makes this project unusual:

- **DianaOS**, a genuine bash-alike shell embedded in the emulator, with pipes, variables, redirection, control flow, a coreutils subset, and hardware inspection commands (`mem`, `regs`, `watch`, `disasm`, `coretop`, …).
- **Pharaoh**, a headless, scriptable harness that runs the real core with no window and no human, so bugs can be reproduced deterministically and fixes proven byte-identical across the ROM library.

The emulation core is a pure library with no window, no `Main`, and no frontend knowledge. Everything else — presentation, frontends, debug tooling — sits above it and depends on it one-directionally.

The project is structured for more than one console. Only the SNES core exists today; every other core is a reserved, empty folder.

---

## Status

**Honest summary: the SNES core runs real commercial games, and several boot correctly and play, but very few have been verified end to end.**

| Area | State |
|---|---|
| 65816 CPU | Full 256/256 opcode table. **Gap:** decimal (BCD) mode — `SED`/`CLD` toggle the flag but `ADC`/`SBC` ignore it |
| SPC700 + S-DSP | Full 256/256 opcodes, BRR decode, 4-tap Gaussian resampling, ADSR/GAIN, 8-voice mixing |
| Audio output | Working — SDL out, with a resampler and dynamic rate control to stop drift |
| PPU backgrounds | All 7 modes, 16×16 tiles, mosaic, Mode 7 + EXTBG, offset-per-tile, direct colour, pseudo and true hi-res |
| PPU sprites | Full OAM decode, real per-scanline 32-sprite / 34-sliver limits |
| Colour math / windowing | Implemented, including the fixed-colour and half-math hardware quirks |
| Memory mapping | LoROM and HiROM, general DMA + HDMA, header-driven SRAM sizing |
| Save states | Working, via a reflective serializer. **Gap:** no version header, so states can break across builds |
| Rewind / fast-forward | Working, core-agnostic (XOR-delta chain) |
| Input | Rebindable keyboard + gamepad, per-game hotkeys, gamepad hot-plug |
| Timing accuracy | Scanline granularity, not per-dot. A deliberate, documented tradeoff |
| Cores other than SNES | None. Reserved folders only |

**Game compatibility** is tracked properly in [`EmuSen_Games_Tested.md`](EmuSen.DianaOS/DianaOS/Usr/Home/Documents/EmuSen%20Manual/EmuSen_Games_Tested.md), which rates each title Perfect → Unplayable and links to the investigation behind the rating. As of now nothing sits in Unplayable, several titles boot and play with known cosmetic bugs, and no title has a claimed full playthrough. Treat that file as the source of truth, not this table.

Verified builds: **Linux x64 is launch-tested.** Windows and macOS binaries are produced and structurally correct but have not been executed — there is no Windows or Mac on the build machine, and macOS builds are unsigned (Apple Silicon requires an ad-hoc signature before it will run them at all).

---

## The two frontends

Both are Avalonia applications sharing one presentation layer, and neither owns any emulation logic.

**Mistress** — the fuller GUI, scoped as bug-testing tooling rather than a polished launcher. ROM picker and ROM browser, save/load state, rebindable keyboard + gamepad, emulator hotkeys, preferences, per-session file logging, and a menu entry that opens the DianaOS console against the running game.

**Hotaru** — a lighter, console-first frontend. Takes a ROM path on the command line and puts the DianaOS shell on the terminal it was launched from, with the game in its own window.

A polished, EmulationStation-style launcher is explicitly *not* either of these; it is planned as a separate project.

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

Crucially, `IDebugTarget` — the interface all of this talks to — is **core-agnostic**. It knows about memory spaces, registers, sprites, palettes and breakpoints, not about the SNES.

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
| `EmuSen.Nehellania` | Shared device I/O: SDL3 audio output, gamepad polling, pad bindings |
| `EmuSen.Mistress` | The fuller Avalonia GUI frontend |
| `EmuSen.Hotaru` | The console-first Avalonia frontend |
| `EmuSen.Pharaoh` | The headless scripted harness |
| `EmuSen.Tomoe` | CLI runner for ground-truth CPU/hardware test vectors |
| `EmuSen.WiseMan` | The xUnit test suite |

The SNES core lives under `EmuSen/Cores/Nintendo/Venus - SNES/`, namespaced `EmuSen.Cores.Nintendo.Venus.*`, with reserved sibling folders for every other planned console.

The layering is enforced in practice, not just described: the PPU exposes a small `IWriteObserver` hook and has no idea a watchpoint exists; the debug layer implements that interface and supplies the meaning.

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

631 tests across 56 files: CPU and PPU hardware behaviour, APU/DSP, audio sync and drain, save-state round-tripping, the DianaOS shell (lexer, parser, pipes, control flow, multi-line continuation, sandboxing, man pages), and Avalonia UI tests that drive real key events through a headless window.

Some of those are **property-based** (CsCheck), asserting laws rather than examples: that the rewind delta codec's `Apply` really is its own inverse, that resampled audio never leaves the range its input spanned, that a fuller audio queue never asks the resampler to speed up, and that `CanDecode` never promises a cheat-code decode that then throws. Each runs hundreds to thousands of generated cases and shrinks any failure to a minimal counterexample.

Beyond unit tests, the project leans on **output-identity digests**: `framesum` and `audiosum` over a fixed window across all 37 ROMs in the local library, so a change to the renderer or the mixer can be shown to alter exactly the games it was meant to and nothing else.

---

## Documentation

This project documents heavily, and deliberately keeps rationale *out* of code comments and *in* man pages. Code comments are one line and point at a section.

- `EmuSen.DianaOS/DianaOS/Etc/Man pages/Hardware/` — per-console hardware notes. The SNES set (`Venus - SNES/`) covers CPU, PPU, APU and memory, including full root-cause writeups for real bugs found and fixed.
- `EmuSen.DianaOS/DianaOS/Usr/Home/Documents/EmuSen Manual/` — project-level docs: overview, debugging tools reference, save states, audio sync, rewind/fast-forward, settings, games tested, roadmap.

These are readable on GitHub, and also from inside the emulator via DianaOS's own `man` and `cat`.

---

## Roadmap

Near-term, roughly in order:

1. Finish verifying the games that currently boot but have not been played through.
2. Decimal (BCD) mode on the 65816.
3. A save-state version header.
4. Real per-Player-2 input bindings (only a mirror-P1 toggle exists today).
5. The Mesen-style multi-pane GUI debugger, built on `IDebugTarget`.
6. A second core — NES (**Moon**) — which is the long-term goal the whole architecture has been built toward.

Explicitly deferred: true doubled-resolution interlace, and per-dot H-position timing.

---

## ROMs

**No ROMs are included, and none will be.** The local test library referenced in the docs is not in this repository and is excluded by `.gitignore`. Supply your own legally-obtained dumps.

---

## Credits and licensing

Written against the [SNESdev wiki](https://snes.nesdev.org/) as the primary hardware reference. [MesenCE](https://github.com/nesdev-org/MesenCE) is consulted as an architecture reference where useful — both projects are GPL-3.0, so there is no compatibility issue, though implementations here are written rather than transcribed.

Licensed **GPL-3.0**, chosen so anything built on this stays open, matching how the project itself was built from openly-published documentation.
