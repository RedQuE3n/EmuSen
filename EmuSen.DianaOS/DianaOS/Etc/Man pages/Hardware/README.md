# Hardware notes

One page per piece of hardware being abstracted, rather than the long inline `// WHY` comments this project used to accumulate directly in the source. Code comments now stay short — local, easy-to-miss gotchas only (a guard that must not be removed, a magic constant's source) — and the actual reasoning, citations, and "this used to be wrong because..." history live here instead, organized by the chip/subsystem they explain rather than scattered across whichever file happened to need the explanation first.

**Why the change:** as EmuSen picked up more cores, per-file inline comments stopped scaling — the same hardware quirk (e.g. SRAM mirroring) could be relevant to multiple files, and finding "everything we know about the PPU" meant grepping across a dozen files instead of reading one page. Structuring by hardware component instead mirrors how the emulator itself is organized (see `Man pages/EmuSen_Project_Overview_v2.md`'s architecture section) and how real hardware documentation (fullsnes, the SNESdev wiki) is itself organized.

**Organization**: this folder mirrors `EmuSen/Cores/` exactly, one level at a time — `<Manufacturer>/<CoreCodename> - <Console>/`, e.g. `Nintendo/Venus - SNES/`. Within a core's subfolder, pages are named `<CoreCodename>_<Component>.md` — e.g. `Nintendo/Venus - SNES/Venus_Memory.md`. Matches both `EmuSen/Cores/`'s own folder convention and the Sailor-Moon-themed core naming scheme (`Man pages/EmuSen_Core_Naming_Scheme.md`).

**Relationship to the rest of `Man pages/`:**
- `EmuSen_Project_Overview_v2.md` / `EmuSen_Core_Gameplan.md` — project-level status and architecture, not hardware-specific. Unchanged by this reorganization.
- `EmuSen_Debugging_Tools_Reference_v5.md` — the core-agnostic debug toolchain (`WatchRegistry`, `search`, `disasm`, etc.). Also unchanged; these pages are about the hardware *being* debugged, not the tools doing the debugging, though they cross-reference each other where relevant (e.g. this Memory page's observer-hooks section points at the debugging-tools page's `SnesDebugTarget` writeup).
- `EmuSen_Core_Naming_Scheme.md` — unrelated (naming convention, not technical documentation).

**`DebugSettings` vs. the newer `DianaOS/` toolchain**: some older ad hoc logging (`DmaVerboseLogging`, `HvIrqChangeLogging`, etc. — plain static bools in `DebugSettings.cs`) predates `WatchRegistry`/`FrameLogRegistry` and hasn't been migrated. Both exist side by side today; a hardware page that references one of these flags is documenting what's actually in the code, not endorsing the pattern going forward. New investigations should prefer the `DianaOS/` toolchain (`watch`, `framelog`, etc.) where it covers the need - see `EmuSen_Settings_Reference.md` §1.6.

## Nintendo

- [`Venus - SNES/`](Nintendo/Venus%20-%20SNES/) — **complete** (CPU/PPU/APU/Memory)
  - [`Venus_Memory.md`](Nintendo/Venus%20-%20SNES/Venus_Memory.md) — CPU address bus, LoROM cartridge mapping, DMA/HDMA, NMI/IRQ subsystem, hardware math unit.
  - [`Venus_CPU.md`](Nintendo/Venus%20-%20SNES/Venus_CPU.md) — 65816 core: fetch/execute loop, interrupt entry sequences, WAI/STP, addressing modes, opcode table verification status.
  - [`Venus_PPU.md`](Nintendo/Venus%20-%20SNES/Venus_PPU.md) — register dispatch, scroll-latch fix, Mode 7, compositing order, color math, sprite budget, windowing, hi-res, status registers.
  - [`Venus_APU.md`](Nintendo/Venus%20-%20SNES/Venus_APU.md) — SPC700 boot/ports/timers, instruction-set quirks, S-DSP register handling, ADSR/GAIN envelopes, BRR decoding.
- [`Moon - NES/`](Nintendo/Moon%20-%20NES/) — **runs, silent** (CPU/PPU/memory/`ICore`/`IDebugTarget`; no audio synthesis, no PAL)
  - [`Moon_CPU.md`](Nintendo/Moon%20-%20NES/Moon_CPU.md) — 2A03/6502 core: the one-access-per-cycle bus model, addressing modes and their dummy reads, the phantom `B`/`U` flag bits, reset and interrupts, the undocumented opcodes, and validation status.
  - [`Moon_Core.md`](Nintendo/Moon%20-%20NES/Moon_Core.md) — `ICore`: the master clock and the CPU-budget accumulator, the Crystal schedule, scanline granularity, named address spaces, save states.
  - [`Moon_Memory.md`](Nintendo/Moon%20-%20NES/Moon_Memory.md) — CPU address decode, the iNES image, nametable mirroring, the five implemented boards, MMC1's serial register, OAM DMA, controllers, the write-observer seam.
  - [`Moon_PPU.md`](Nintendo/Moon%20-%20NES/Moon_PPU.md) — 2C02: the loopy `v`/`t`/`x`/`w` registers, the `$2007` read buffer, palette holes, background and sprite composition, sprite 0 hit.
  - [`Moon_APU.md`](Nintendo/Moon%20-%20NES/Moon_APU.md) — the register surface, length counters and frame IRQ that are modelled, and the synthesis that is not.
  - [`Moon_Debug.md`](Nintendo/Moon%20-%20NES/Moon_Debug.md) — `IDebugTarget`'s second implementation: memory spaces, registers, tile/tilemap decoders, the disassembler, and what that second implementation proved.
- `Mercury - GB-GBC/` — CPU, memory and cartridge documented; no PPU or APU yet.
- `Jupiter - GBA/`, `Mars - N64/`, `Saturn - Virtual Boy/`, `Uranus - GameCube/`, `Neptune - Wii/`, `Pluto - Wii U/`, `Luna - DS/`, `Artemis - 3DS-New3DS/` — stubs, not started yet.

## Sega

`Jadeite - Game Gear/`, `Nephrite - 32X/`, `Endymion - Master System/`, `Beryl - Genesis/`, `Zoisite - Saturn/`, `Kunzite - Dreamcast/` — stubs, not started yet.

## Sony

`Diamond - PlayStation/`, `Sapphire - PlayStation 2/`, `Rubeus - PlayStation 3/`, `Esmeraude - PSP/`, `Wiseman - PS Vita/` — stubs, not started yet.

## Atari

`Eudial - Atari 2600/`, `Mimete - Atari 5200/`, `Tellu - Atari 7800/`, `Viluy - Atari Lynx/`, `Cyprine & Ptilol - Atari Jaguar/` — stubs, not started yet.

## Microsoft

`CereCere - Xbox/`, `JunJun - Xbox 360/`, `PallaPalla - Xbox One/`, `VesVes - Xbox Series/` — stubs, not started yet.

## NEC

`Tigers Eye - PC Engine/`, `Fish Eye - SuperGrafx/`, `Hawks Eye - PC-FX/` — stubs, not started yet.

---

Every core folder above — started or not — has at least a placeholder `README.md`, so `Man pages/Hardware/` always mirrors `EmuSen/Cores/` folder-for-folder, manufacturer subfolder included, ahead of a core actually starting. Real pages (`<CoreCodename>_CPU.md`, `_PPU.md`, etc.) get added the same way `Venus - SNES/`'s were, once there's a core to document.

(Folder names drop apostrophes the same way the corresponding `EmuSen/Cores/` folder does — e.g. `Hawks Eye`, not `Hawk's Eye` — for the same filesystem-portability reasons. See each stub's own `README.md` for the fully-punctuated display name.)
