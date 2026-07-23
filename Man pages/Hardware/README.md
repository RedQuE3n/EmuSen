# Hardware notes

One page per piece of hardware being abstracted, rather than the long inline `// WHY` comments this project used to accumulate directly in the source. Code comments now stay short — local, easy-to-miss gotchas only (a guard that must not be removed, a magic constant's source) — and the actual reasoning, citations, and "this used to be wrong because..." history live here instead, organized by the chip/subsystem they explain rather than scattered across whichever file happened to need the explanation first.

**Why the change:** as EmuSen picked up more cores, per-file inline comments stopped scaling — the same hardware quirk (e.g. SRAM mirroring) could be relevant to multiple files, and finding "everything we know about the PPU" meant grepping across a dozen files instead of reading one page. Structuring by hardware component instead mirrors how the emulator itself is organized (see `Man pages/EmuSen_Project_Overview_v2.md`'s architecture section) and how real hardware documentation (fullsnes, the SNESdev wiki) is itself organized.

**Naming**: `<CoreCodename>_<Component>.md` — e.g. `Venus_Memory.md`, `Venus_CPU.md`. Matches the Sailor-Moon-themed core naming scheme (`Man pages/EmuSen_Core_Naming_Scheme.md`), so a page's filename says both which core and which hardware piece without needing "SNES" spelled out redundantly.

**Relationship to the rest of `Man pages/`:**
- `EmuSen_Project_Overview_v2.md` / `EmuSen_Core_Gameplan.md` — project-level status and architecture, not hardware-specific. Unchanged by this reorganization.
- `EmuSen_Debugging_Tools_Reference_v5.md` — the core-agnostic debug toolchain (`WatchRegistry`, `search`, `disasm`, etc.). Also unchanged; these pages are about the hardware *being* debugged, not the tools doing the debugging, though they cross-reference each other where relevant (e.g. this Memory page's observer-hooks section points at the debugging-tools page's `SnesDebugTarget` writeup).
- `EmuSen_Core_Naming_Scheme.md` — unrelated (naming convention, not technical documentation).

**`DebugSettings` vs. the newer `Debug/` toolchain**: some older ad hoc logging (`DmaVerboseLogging`, `HvIrqChangeLogging`, etc. — plain static bools in `DebugSettings.cs`) predates `WatchRegistry`/`FrameLogRegistry` and hasn't been migrated. Both exist side by side today; a hardware page that references one of these flags is documenting what's actually in the code, not endorsing the pattern going forward. New investigations should prefer the `Debug/` toolchain (`watch`, `framelog`, etc.) where it covers the need.

## Pages

- [`Venus_Memory.md`](Venus_Memory.md) — CPU address bus, LoROM cartridge mapping, DMA/HDMA, NMI/IRQ subsystem, hardware math unit.
- More to follow, one hardware component at a time (CPU, PPU, APU next).
