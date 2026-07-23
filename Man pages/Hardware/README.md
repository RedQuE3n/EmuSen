# Hardware notes

One page per piece of hardware being abstracted, rather than the long inline `// WHY` comments this project used to accumulate directly in the source. Code comments now stay short — local, easy-to-miss gotchas only (a guard that must not be removed, a magic constant's source) — and the actual reasoning, citations, and "this used to be wrong because..." history live here instead, organized by the chip/subsystem they explain rather than scattered across whichever file happened to need the explanation first.

**Why the change:** as EmuSen picked up more cores, per-file inline comments stopped scaling — the same hardware quirk (e.g. SRAM mirroring) could be relevant to multiple files, and finding "everything we know about the PPU" meant grepping across a dozen files instead of reading one page. Structuring by hardware component instead mirrors how the emulator itself is organized (see `Man pages/EmuSen_Project_Overview_v2.md`'s architecture section) and how real hardware documentation (fullsnes, the SNESdev wiki) is itself organized.

**Organization**: one subfolder per core, named `<CoreCodename> - <Console>` — the exact same convention `EmuSen/Cores/Nintendo/` itself uses (e.g. `Venus - SNES/`, and eventually `Moon - NES/`, `Mars - N64/`, etc.), so a doc folder and its corresponding code folder are always named identically. Within a core's subfolder, pages are named `<CoreCodename>_<Component>.md` — e.g. `Venus - SNES/Venus_Memory.md`. Matches the Sailor-Moon-themed core naming scheme (`Man pages/EmuSen_Core_Naming_Scheme.md`).

**Relationship to the rest of `Man pages/`:**
- `EmuSen_Project_Overview_v2.md` / `EmuSen_Core_Gameplan.md` — project-level status and architecture, not hardware-specific. Unchanged by this reorganization.
- `EmuSen_Debugging_Tools_Reference_v5.md` — the core-agnostic debug toolchain (`WatchRegistry`, `search`, `disasm`, etc.). Also unchanged; these pages are about the hardware *being* debugged, not the tools doing the debugging, though they cross-reference each other where relevant (e.g. this Memory page's observer-hooks section points at the debugging-tools page's `SnesDebugTarget` writeup).
- `EmuSen_Core_Naming_Scheme.md` — unrelated (naming convention, not technical documentation).

**`DebugSettings` vs. the newer `Debug/` toolchain**: some older ad hoc logging (`DmaVerboseLogging`, `HvIrqChangeLogging`, etc. — plain static bools in `DebugSettings.cs`) predates `WatchRegistry`/`FrameLogRegistry` and hasn't been migrated. Both exist side by side today; a hardware page that references one of these flags is documenting what's actually in the code, not endorsing the pattern going forward. New investigations should prefer the `Debug/` toolchain (`watch`, `framelog`, etc.) where it covers the need.

## Cores

### [`Venus - SNES/`](Venus%20-%20SNES/) — complete (CPU/PPU/APU/Memory)

- [`Venus_Memory.md`](Venus%20-%20SNES/Venus_Memory.md) — CPU address bus, LoROM cartridge mapping, DMA/HDMA, NMI/IRQ subsystem, hardware math unit.
- [`Venus_CPU.md`](Venus%20-%20SNES/Venus_CPU.md) — 65816 core: fetch/execute loop, interrupt entry sequences, WAI/STP, addressing modes, opcode table verification status.
- [`Venus_PPU.md`](Venus%20-%20SNES/Venus_PPU.md) — register dispatch, scroll-latch fix, Mode 7, compositing order, color math, sprite budget, windowing, hi-res, status registers.
- [`Venus_APU.md`](Venus%20-%20SNES/Venus_APU.md) — SPC700 boot/ports/timers, instruction-set quirks, S-DSP register handling, ADSR/GAIN envelopes, BRR decoding.

### Stub folders — not started yet

Every other core already has a reserved `<CoreCodename> - <Console>/` folder here, one per `EmuSen/Cores/<Manufacturer>/<CoreCodename> - <Console>/`, so the doc tree always mirrors the code tree instead of playing catch-up once a core actually starts. Each stub folder holds a single placeholder `README.md`; real pages (`<CoreCodename>_CPU.md`, `_PPU.md`, etc.) get added the same way `Venus - SNES/`'s were, once there's a core here to document.

**Nintendo** — `Moon - NES/`, `Mercury - GB-GBC/`, `Jupiter - GBA/`, `Mars - N64/`, `Saturn - Virtual Boy/`, `Uranus - GameCube/`, `Neptune - Wii/`, `Pluto - Wii U/`
**Sega** — `Jadeite - Game Gear/`, `Nephrite - 32X/`, `Endymion - Master System/`, `Beryl - Genesis/`, `Zoisite - Saturn/`, `Kunzite - Dreamcast/`
**Sony** — `Diamond - PlayStation/`, `Sapphire - PlayStation 2/`, `Rubeus - PlayStation 3/`, `Esmeraude - PSP/`, `Wiseman - PS Vita/`
**Atari** — `Eudial - Atari 2600/`, `Mimete - Atari 5200/`, `Tellu - Atari 7800/`, `Viluy - Atari Lynx/`, `Cyprine & Ptilol - Atari Jaguar/`
**Microsoft** — `CereCere - Xbox/`, `JunJun - Xbox 360/`, `PallaPalla - Xbox One/`, `VesVes - Xbox Series/`
**NEC** — `Tigers Eye - PC Engine/`, `Fish Eye - SuperGrafx/`, `Hawks Eye - PC-FX/`

(Folder names drop apostrophes the same way the corresponding `EmuSen/Cores/` folder does — e.g. `Hawks Eye`, not `Hawk's Eye` — for the same filesystem-portability reasons. See each stub's own `README.md` for the fully-punctuated display name.)
