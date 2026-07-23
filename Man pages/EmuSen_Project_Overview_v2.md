# EmuSen — Project Overview

*(Living document — companion to `EmuSen_Debugging_Tools_Reference` in this same `Man pages` folder, which covers the debug toolchain in detail. This one covers the emulator itself: what's implemented, what's known-incomplete, and what's next.)*

---

## 1. What this is

EmuSen is a SNES emulator written in C# / .NET 10, structured as two sibling projects:

- **`EmuSen/`** — the core (CPU/PPU/APU/memory) plus a Raylib-based console frontend (`Frontend/Program.cs`). This is the primary development/debugging environment — cores get built and verified here first, standalone, before the GUI frontend is the main way of running them.
- **`EmuSen.Frontend/`** — an Avalonia GUI frontend, referencing `EmuSen` via project reference. ROM picker, ~~ad hoc~~ rebindable keyboard+gamepad input, Save/Load State menu items.

**Licensing stance:** the SNESdev wiki (mirrored at both `snes.nesdev.org` and `snesdev.mesen.ca` — maintained by the Mesen/MesenCE team) is the primary hardware-reference source. Mesen/MesenCE's own *documentation* gets read to understand hardware behavior; their source code does not — every implementation here is original. `SourMesen/Mesen2` is archived; `nesdev-org/MesenCE` is the actively maintained continuation and gets cited, not "Mesen2."

**Multi-core intent:** the project is meant to eventually support more than one console. `Cores/Snes/` is already folder-scoped for this, though the C# *namespaces* underneath it haven't caught up yet (still generic names like `EmuSen.Memory` rather than `EmuSen.Cores.Snes.Memory` — a cheap, low-risk rename planned as a drive-by whenever those files are touched anyway, not yet done as its own pass). The debug toolchain (`Debug/IDebugTarget.cs` and everything built on it) was deliberately designed core-agnostic from day one for the same reason — see the debugging tools reference doc.

---

## 1a. Development environment

The machine this project is actually built and run on, kept current here so anything environment-specific (paths, package manager, GPU driver quirks) can be cross-checked against real conditions rather than assumed:

- **OS:** Fedora Linux 44, KDE Plasma Desktop Edition (Wayland)
- **Kernel:** 7.1.4-200.fc44.x86_64
- **KDE Plasma:** 6.7.3 / **KDE Frameworks:** 6.28.0 / **Qt:** 6.11.1
- **CPU:** AMD Ryzen 7 7700X (8-core / 16-thread)
- **RAM:** 32 GiB
- **GPU:** AMD Radeon RX 6800 (discrete) + Ryzen 7700X integrated graphics
- **Motherboard/system manufacturer:** ASUS

Package manager is `dnf` (Fedora), not `apt`/`apt-get` — relevant for any install instructions given for this project (e.g. `ffmpeg` for the frame-recording tool's video encode step — see the debugging tools reference, §3.8 — is `sudo dnf install ffmpeg` here, not the Debian/Ubuntu-style command).

---

## 2. Architecture

**Layering, roughly bottom-to-top:**

- **Hardware simulation** (`Cores/Snes/Cpu`, `Cores/Snes/Apu`, `Cores/Snes/Ppu`, `Cores/Snes/Memory`) — the actual 65816/SPC700/S-DSP/PPU/memory-map implementations. This layer knows nothing about debugging, rendering presentation, or frontends; it just simulates hardware, one register/opcode/pixel at a time.
- **`Common/`** — cross-cutting, not console-specific: `EmulatorSession` (a headless per-frame driver used by the Avalonia frontend), `StateSerializer` (reflective save states), `TeeTextWriter` (console+file logging).
- **`Settings/`** — global configuration, most notably `DebugSettings` (the logging-toggle registry) and input/graphics/audio settings. Intentionally simple global static state for the debug toggles specifically — a conscious tradeoff (see §5) rather than an oversight.
- **The debug toolchain** (`Debug/`, plus `Cores/Snes/Debug/`) — a deliberately separate, core-agnostic layer sitting *beside* the hardware simulation, not inside it. `Debug/IDebugTarget.cs` defines a contract any core can implement; `Cores/Snes/Debug/SnesDebugTarget.cs` is the SNES implementation; `Debug/DebugCommandProcessor.cs` is a Unix-toolchain-style command layer on top of that. See the companion `EmuSen_Debugging_Tools_Reference` doc for the full breakdown.
- **Frontends** (`Frontend/Program.cs` in `EmuSen/`, all of `EmuSen.Frontend/`) — presentation only. The Raylib console build and the Avalonia GUI are two independent entry points into the same core; neither owns emulation logic itself.

**A concrete example of the layering working as intended:** `MemoryBus` (hardware simulation) exposes a tiny, debug-agnostic `IWriteObserver` hook that it calls on every WRAM write, with no idea what's listening. `SnesDebugTarget` (debug toolchain) implements that interface and is the thing that actually knows what a "watchpoint" is. `MemoryBus` could be reused by a completely different debug story (or none at all) without any changes — the coupling only exists in one direction, and it's the debug layer depending on the core, not the reverse. This wasn't always true — a `WatchRegistry` field and a `Cpu` back-reference used to live directly on `MemoryBus` itself, mixing the two layers together, until a later architecture-focused pass (see §4 and §7) untangled it.

**Multi-core intent:** `Cores/Snes/` is already folder-scoped assuming siblings (`Cores/Nes/`, eventually) will exist someday. The C# *namespaces* underneath haven't caught up to the folder structure yet (see §7) — a known, deliberately-deferred cleanup, not an oversight. The debug toolchain's core-agnostic design (`IDebugTarget` et al.) is the one piece of this multi-core intent that's actually been built and used, rather than just planned for.

---

## 3. Directory map

```
EmuSen Project/
├── EmuSen/                                   # Core emulator + Raylib console frontend
│   ├── EmuSen.csproj
│   ├── Common/
│   │   ├── EmulatorSession.cs                # Headless per-frame driver (used by Avalonia frontend)
│   │   ├── StateSerializer.cs                # Reflective save-state serializer
│   │   └── TeeTextWriter.cs                  # Console + file log tee
│   ├── Cores/
│   │   └── Snes/                             # Folder-scoped for multi-core; namespaces underneath
│   │       │                                  #   still generic (EmuSen.Memory, not
│   │       │                                  #   EmuSen.Cores.Snes.Memory) - see §7.
│   │       ├── Cpu/
│   │       │   ├── Core/Cpu.cs               # 65816 execution core
│   │       │   ├── Opcodes/
│   │       │   │   ├── Cpu.AddressModes.cs
│   │       │   │   ├── Cpu.OpcodeTable.cs
│   │       │   │   └── Cpu.Opcodes.cs
│   │       │   └── Disassembler/
│   │       │       └── Snes65816Disassembler.cs  # Separate, read-only mnemonic table - NOT built
│   │       │                                      #   from the execution opcode table (see §4's
│   │       │                                      #   companion doc reference)
│   │       ├── Apu/
│   │       │   ├── Dsp/
│   │       │   │   ├── BrrDecoder.cs
│   │       │   │   ├── DspVoice.cs
│   │       │   │   └── SDsp.cs                # Real audio synthesis; not connected to output
│   │       │   └── Spc700/
│   │       │       ├── Core/Spc700.cs
│   │       │       └── Opcodes/
│   │       │           ├── Spc700.AddressModes.cs
│   │       │           ├── Spc700.OpcodeTable.cs
│   │       │           └── Spc700.Opcodes.cs
│   │       ├── Memory/
│   │       │   ├── Cartridge.cs              # ROM/SRAM load+save, header-driven SRAM sizing
│   │       │   ├── Dma.cs
│   │       │   ├── MathUnit.cs               # Hardware multiply/divide - extracted out of MemoryBus
│   │       │   ├── IWriteObserver.cs         # Debug-agnostic write-observer hook (see §2)
│   │       │   └── MemoryBus.cs              # Address decode/dispatch - the bus's actual job only
│   │       ├── Ppu/
│   │       │   ├── Core/
│   │       │   │   ├── Ppu.cs
│   │       │   │   ├── Ppu.Registers.cs
│   │       │   │   └── Ppu.RegisterTable.cs
│   │       │   └── Renderer/
│   │       │       ├── Renderer.cs
│   │       │       ├── Renderer.Backgrounds.cs   # All 7 BG modes
│   │       │       ├── Renderer.Sprites.cs
│   │       │       ├── Renderer.Mode7.cs         # Matrix transform + EXTBG
│   │       │       ├── Renderer.Scanline.cs      # Compositing/priority order per mode
│   │       │       └── Renderer.Debug.cs         # VRAM sheet, OAM dump, black-tile diagnostic
│   │       ├── Debug/
│   │       │   ├── StateDump.cs              # Pre-toolchain CPU+PPU snapshot formatter
│   │       │   └── SnesDebugTarget.cs        # SNES's IDebugTarget implementation
│   │       └── Input/
│   │           └── Input.cs
│   ├── Debug/                                # Core-agnostic debug toolchain (see companion doc)
│   │   ├── IDebugTarget.cs
│   │   ├── DebugCommandProcessor.cs
│   │   ├── WatchRegistry.cs
│   │   └── DebugTools.cs                     # Older generic helpers (hexdump, tile-ASCII, etc.)
│   ├── Frontend/
│   │   └── Program.cs                        # Raylib console frontend; Main = timing loop,
│   │                                          #   RunHotkeys = all F1-F5/F9/P dispatch (extracted
│   │                                          #   from Main in a later decoupling pass - see §7)
│   └── Settings/
│       ├── AudioSettings.cs
│       ├── DebugSettings.cs                  # Every logging toggle - see companion doc
│       ├── GraphicsSettings.cs
│       └── InputBindings.cs
│
└── EmuSen.Frontend/                           # Avalonia GUI frontend
    ├── EmuSen.Frontend.csproj
    ├── App.axaml / App.axaml.cs
    ├── Input/
    │   ├── ControllerKeyMap.cs
    │   ├── GamepadBindingMap.cs
    │   └── GamepadManager.cs
    ├── Program.cs
    └── Views/
        ├── MainWindow.axaml / .axaml.cs       # ScreenWidth now an instance property tracking
        │                                       #   the renderer's actual FrameWidth (hi-res support)
        └── InputSettingsWindow.axaml / .axaml.cs
```

Not shown: `Saves/*.srm`/`*.state`, `Logs/`, `bin/`, `obj/` — build artifacts and user data, excluded from any packaging.

---

## 4. Current features, by subsystem


### CPU (65816)
- Full 256/256 opcode table, verified against oxyron.de during initial development (one cross-reference typo caught and fixed).
- WAI/STP real halt states.
- **Known gap:** decimal (BCD) mode — SED/CLD correctly toggle the D flag, but ADC/SBC never check it. Rare in practice (few SNES games use decimal mode) but not implemented.
- `LastInstructionPC`/`LastInstructionPB` — the pre-execution PC, exposed for debug tooling (distinct from the live `PC`/`PB`, which advance almost immediately after fetch — see the debugging tools reference, §2, for the bug this fixed).

### APU (SPC700 + S-DSP)
- Full 256/256 SPC700 opcode table.
- Real S-DSP audio synthesis: BRR decoding, pitch resampling (nearest-neighbor, not Gaussian — documented simplification), full ADSR/GAIN envelopes, 8-voice mixing.
- **Known gap:** synthesized samples are never sent to an actual audio output device. The DSP is "correct but silent."

### PPU — background rendering
- **All 7 BG modes implemented**, including correct per-mode bit depth (2/4/8bpp as appropriate) and per-mode compositing/priority order — verified this session against the SNESdev wiki's Backgrounds page priority table for every mode (0 through 6; Mode 7 has its own separate compositing path). Two real bugs found and fixed in this pass: Mode 0's BG4 was missing its priority-bit split entirely, and Modes 2-5 were incorrectly reusing Mode 0/1's compositing order instead of their own (genuinely different) interleave pattern.
- 16x16 tile mode (BGMODE bits 4-7), tilemap 32x32/64x32/32x64/64x64 wraparound (including the tricky 64x64 case), tile bitplane format — all independently verified against the wiki.
- Mosaic, including a real starting-scanline latch (anchors to whichever scanline `$2106` was last written on, matching documented hardware behavior — not just always scanline 0).
- Mode 7: full affine transform (matrix formula verified against two independent sources), plus **EXTBG** (Mode 7's second layer via SETINI bit 6) — implemented this session, not yet tested against a ROM that actually uses it.
- Offset-per-tile (Modes 2/4/6): implemented as a best-effort reproduction of the commonly-documented behavior — the exact sub-tile column-alignment edge cases are something even experienced SNES homebrew developers describe as ambiguous in official documentation, so this is a solid approximation, not a verified-exact implementation.
- Direct Color mode (Modes 3/4's 8bpp BG1).
- Hi-res:
  - **Phase A (pseudo hi-res, SETINI bit 3):** real 512-column output via genuine main/sub screen interleave — not an approximation.
  - **Phase B (true Mode 5/6 hi-res):** real tile-pairing (two adjacent, ordinary tile definitions feeding the main vs. sub composite) rather than a blended approximation.
  - Both phases required real architecture work: the frame buffer, `GetFrameBufferRgba`, and the Avalonia frontend's bitmap all had to become width-aware instead of assuming a fixed 256px frame.
  - **Not done:** interlace's actual doubled-*vertical*-resolution output. The field-parity bit (STAT78) is correctly tracked now, but combining two fields into one taller displayed image was scoped out — deliberately, given genuine interlace saw real gameplay use in only a couple of known titles, and the engineering cost (scanline *count* feeds this project's core timing loop directly, not just buffer dimensions) is meaningfully larger than hi-res was for a much rarer payoff.

### PPU — sprites (OBJ)
- Full OAM decode (size-select, high-table, priority rotation), correct multi-tile VRAM addressing (16-tile-wide grid, 512-byte row stride).
- Real per-scanline evaluation matching hardware's 32-sprite/34-sliver limits, including the reverse-index-order sliver-culling detail.
- OBJ color math restriction (only palettes 4-7 participate).
- **Real bug fixed this session:** sprite Y was being culled too aggressively (`Y >= 224`), discarding sprites that should wrap in from the top of the screen (Y = 240-255, signed -16..-1) instead of rendering their visible portion.

### PPU — other
- Full open-bus emulation (`$4210`/`$4211`/`$4212`'s undriven bits now reflect the last bus value instead of being hardcoded to 0), plus the general "last value on the bus" model for genuinely-unmapped register reads.
- Windowing (masking logic independently verified against 3 sources).
- Color math / CGADSUB, per-layer participation rules.

### Memory / DMA
- LoROM mapping; SRAM size now read from the ROM header (was hardcoded to 2KB for every game) with correct chip-mirroring behavior (modulo addressing instead of a hard size cutoff) — this was the actual fix for Super Metroid's boot-time anti-piracy check, which specifically tests for correct SRAM mirroring.
- WRAM low-bank mirror (`$00-$3F`/`$80-$BF:$0000-$1FFF`).
- `$2180-$2183` (WMDATA/WMADDL/M/H) — a second WRAM access path, previously entirely unimplemented.
- `$2140-$217F` APU port mirroring (previously only the canonical `$2140-$2143` were handled).
- General DMA + HDMA.
- `MemoryBus` decoupled from debug-toolchain plumbing: the hardware multiply/divide unit now lives in its own `MathUnit` class rather than as loose fields/logic on the bus, and `MemoryBus` no longer holds a `WatchRegistry` or a `Cpu` back-reference — it exposes a small `IWriteObserver` hook instead, with `SnesDebugTarget` on the other end. See §2.

### Save states, input, frontends
- Save states via a reflective binary serializer (F5/F9 in the console build; menu items in Avalonia). **Known limitation:** no version header — can break across builds if field layout changes.
- SRAM auto-save (every 300 frames + on exit/ROM switch).
- Rebindable keyboard + gamepad input (Avalonia frontend), Wayland support.
- Console frontend: ROM path via CLI arg; Avalonia frontend: real file-picker.
- Console frontend's `Program.cs` decoupled: `Main` used to be a single ~350-line method containing ROM loading, the timing loop, and all six hotkeys inline. Hotkey dispatch (F1-F5, F9, P) is now its own `RunHotkeys` method, separate from the actual per-scanline emulation loop.

### Debug toolchain
Covered in full in the companion document. Summary: a core-agnostic `IDebugTarget` interface (memory spaces, registers, sprites, palettes, watchpoints, frame counter, disassembly), a `SnesDebugTarget` implementation, and a Unix-toolchain-style command layer (`DebugCommandProcessor`) reachable via the console's F4 prompt. Built specifically so it isn't SNES-only and can eventually back a real GUI debugger.

---

## 5. Known, already-accepted architectural simplifications (not bugs — don't re-flag without new evidence)

- No per-dot H-position timing — scanline granularity throughout (affects OPHCT and a few edge cases).
- Offset-per-tile's exact sub-tile alignment (see above).
- BRR pitch resampling is nearest-neighbor, not Gaussian.
- Mosaic/mode-change/HDMA-timing all operate at scanline granularity, with documented per-feature latching behavior where hardware specifically requires it (mosaic's starting-scanline latch, the immediate-NMI-on-vblank-rising-edge case, etc.) rather than true per-cycle accuracy.
- Frame timer in the Avalonia frontend is a fixed 60fps UI timer, not accumulator-driven — will drift over long sessions.

---

## 6. Open bugs / active investigations

- **Coins and Yoshi not rendering in SMW.** Long investigation this session — ruled out: sprite/tile rendering logic (verified correct against docs and against the exact tile+palette data dumped from a real session), the general DMA transfer mechanism (proven correct via adjacent, working animated-tile transfers), and the DMA source-address computation itself (confirmed varying correctly in the most recent session, not stuck). Currently narrowed to: **whatever's supposed to write real graphics data into the WRAM staging buffer before the DMA copies it out doesn't seem to be doing so** — a targeted watch (`watch add WRAM 8000 1800` via F4) is in place to confirm this directly. Not yet resolved.
- **A stuck HDMA window on the title screen** (freezes at a single pixel) — isolated but never root-caused; windowing was previously disabled globally to work around it, then re-enabled once judged lower-risk than leaving every window-based effect broken everywhere. Worth a dedicated pass now that windowing is back on.

---

## 7. TODO / roadmap

Roughly in order of "cheap and likely valuable" to "bigger, deliberately-deferred":

1. **Continue the coin/Yoshi WRAM investigation** (above) — the immediate active thread.
2. **Get an actual test ROM for Mode 7 EXTBG and offset-per-tile** to confirm those implementations against real content rather than documentation alone (same category of "implemented but unexercised" as tile16/mosaic were before SMW's own logs confirmed them safe).
3. **A verification pass on the new disassembler** (`Snes65816Disassembler`) — built carefully but not given the oxyron.de-level scrutiny the execution opcode table got. See the debugging tools reference, §3.7.
4. **Watchpoints beyond WRAM** — report writes from `Ppu`'s VRAM/CGRAM/OAM paths and the general CPU-bus/SRAM path through `MemoryBus`'s `IWriteObserver` hook the same way WRAM already does.
5. **The drive-by namespace rename** (`EmuSen.Memory` → `EmuSen.Cores.Snes.Memory`, etc.) — cheap, zero-behavior-change, do it opportunistically when touching affected files rather than as its own pass.
6. **The rest of the `MemoryBus` decoupling** — the multiply/divide unit and the debug-toolchain plumbing are out (§2, §4); H/V-IRQ/NMI/vblank state is not. On closer inspection this cluster turned out more entangled than it first looked (the same `_vblankFlag` feeds NMI edge-detection *and* the RDNMI/HVBJOY register reads, and $4200 sets both NMI and IRQ enable in one write) — forcing a clean split risked adding more cross-object coupling than it removed, in genuinely delicate, already-hard-won timing logic. Worth revisiting deliberately, not as a quick follow-on.
7. **The `Renderer`/Raylib split** — `Renderer` still mixes pure pixel computation with Raylib window/texture ownership even in headless mode. Real, but risky enough (core rendering code, many delicate accuracy fixes riding on it) to treat as its own dedicated future pass rather than bundling into a quick cleanup.
8. **Audio output** — connect the already-correct S-DSP synthesis to an actual playback device.
9. **Decimal (BCD) mode** on the 65816 (ADC/SBC currently ignore the D flag).
10. **The stuck HDMA title-screen window bug** — dedicated investigation, now that windowing is confirmed safe to leave on globally.
11. **Breakpoints / single-step / pause-resume** — needs real execution-loop support for pausing mid-frame, which doesn't exist today. A real, separate piece of work, not a quick add.
12. **The Avalonia GUI debug window** — the actual Mesen-style multi-pane debugger, built against `IDebugTarget` once enough of the above exists to make it worthwhile.
13. **A second `IDebugTarget` implementation (NES or otherwise)** — to actually prove out the core-agnostic design rather than just asserting it.
14. **True doubled-resolution interlace output** — explicitly deferred (§4) given its rarity in real games versus its engineering cost; revisit only if a specific ROM actually needs it.
15. **NES core work generally** — the original long-term goal this whole architecture (folder structure, `IDebugTarget`, the eventual namespace cleanup) has been building toward, still not started. Planned to begin only once SNES + the Avalonia frontend are stable — per earlier project discussion, not a new decision.
