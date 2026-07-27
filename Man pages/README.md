# Man pages — index

This folder is EmuSen's documentation set: project status, architecture, the debug toolchain, settings, and per-hardware reference notes. This page exists purely to help you find the right doc without opening all of them - it doesn't duplicate their content.

## Top-level docs

- **`EmuSen_Project_Overview_v2.md`** — the architecture reference: current directory/namespace map, what's built vs. planned, the full TODO/roadmap. Start here for "how is this project structured" and "what's left to do."
- **`EmuSen_Core_Gameplan.md`** — self-contained orientation + a phased plan for the current core (Venus/SNES) specifically: what's verified, what's actively being investigated, phase-by-phase next steps. Start here for "what should I work on next."
- **`EmuSen_Core_Naming_Scheme.md`** — pure naming/organizational convention: what a future core is called (the Sailor Moon codename scheme) and where its folder lives. Not technical documentation.
- **`EmuSen_Debugging_Tools_Reference_v5.md`** — every debug tool: console hotkeys, `DebugSettings` logging flags, and the full DianaOS shell/command reference (the reusable, core-agnostic debug toolchain). The largest doc here - organized top-down from "things you press a key for" to "the underlying toolchain."
- **`EmuSen_Frontend_Driver.md`** — `EmuSen.Hotaru`'s actual launch sequence and per-frame driver: `Main`/`App`/`GameWindow`, the emulation thread, the always-live DianaOS console reader thread, and every hotkey's dispatch mechanism.
- **`EmuSen_Settings_Reference.md`** — the three static config classes (`DebugSettings`, `AudioSettings`, `GraphicsSettings`): every flag, its default, and the historical investigation context behind why it exists.
- **`EmuSen_Games_Tested.md`** — real-game compatibility tracking (as opposed to the CPU/PPU ground-truth test suites in `EmuSen.Tomoe`, which test opcodes/registers in isolation). Always reflects current, real behavior - a bug fix removes an entry rather than just annotating it.
- **`EmuSen_Launcher_Multicore_Gameplan.md`** — a separate, later-stage plan for a distinct future multi-core/launcher project (not `EmuSen.Mistress9`, which stays scoped to bug-testing tooling). Gated on a second real core existing.
- **`Commands.txt`** — flat scratch notes: literal shell commands for common branch/build/run sequences. Not prose documentation.

## Subfolders

- **`Hardware/`** — per-console technical reference, one folder per `EmuSen/Cores/<Manufacturer>/<Codename> - <Console>/`, mirroring that tree exactly. See `Hardware/README.md` for its own index; only `Nintendo/Venus - SNES/` has real content today (CPU/PPU/APU/Memory), everything else is a placeholder stub for a not-yet-started core.
- **`Old/`** — superseded major versions of docs that have since been replaced by a newer numbered version (e.g. `EmuSen_Debugging_Tools_Reference.md` through `_v4.md`, `EmuSen_Project_Overview.md`). Kept for history only. **Nothing here is current** - if you're linking to a companion doc from anywhere in this project, always use the current top-level filename (the one with the highest version suffix, or no suffix if there's only one version), never a bare/old name that happens to still resolve to something in `Old/`.

## The "This revision / Previous revision" convention

Several docs above (notably `EmuSen_Debugging_Tools_Reference_v5.md` and `EmuSen_Frontend_Driver.md`) open with an italicized paragraph chaining "This revision: ... Previous revision: ... " notes - a lightweight, inline changelog covering what's changed recently, kept short enough to skim before reading the rest of the doc. Not every doc uses this (`EmuSen_Core_Gameplan.md`, `Commands.txt`, and others don't) - it's a convention available when a doc's own history is worth surfacing inline, not a requirement. When one of these preambles grows long enough to stop being skimmable, trim the older entries down to a short summary line each and point to `git log` for full detail, rather than letting it grow unbounded - full reasoning/diffs for anything old enough to be compressed already lives in commit history.
