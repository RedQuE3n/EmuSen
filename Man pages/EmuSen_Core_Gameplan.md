# EmuSen (Core) — Game Plan

*(Written to be self-contained: if you're a fresh chat session reading only this file, §0-1 should orient you before the phased plan starts. Companion docs in this same `Man pages` folder — `EmuSen_Project_Overview.md` and `EmuSen_Debugging_Tools_Reference.md` — have the full technical depth this doc deliberately doesn't repeat; check those for "how does X actually work" questions, this doc is for "what's next and in what order.")*

---

## 0. Orientation, if you're new to this

**EmuSen** is a SNES emulator in C# / .NET 10 — full 65816 CPU, SPC700+S-DSP audio, PPU (all 7 background modes plus Mode 7, sprites, hi-res, mosaic, windowing, color math), memory/DMA. Several sibling projects on disk (see `EmuSen_Project_Overview_v2.md` §1 for the full current list): `EmuSen/` (this doc's subject — the core), `EmuSen.RaylibFrontend/` (the console build used for development), `EmuSen.Presentation/` (shared presentation/shader code), and `EmuSen.TestingStudio/` (an Avalonia GUI purpose-built for bug testers, renamed from `EmuSen.Frontend/`). The separate, later-stage multi-core/launcher ambition (an EmulationStation-style browsing UI) is deliberately **not** being built into `EmuSen.TestingStudio` — that vision is scoped for a distinct, not-yet-created project, so bug-testing tooling and a future polished launcher never have to share one codebase. See `EmuSen_Launcher_Multicore_Gameplan.md` (renamed from `EmuSen.Frontend_Multicore_Gameplan.md`) for that plan.

**Licensing/reference stance:** the SNESdev wiki (mirrored at `snes.nesdev.org` and `snesdev.mesen.ca`, maintained by the Mesen/MesenCE team) is the primary hardware reference. Mesen/MesenCE's *documentation* gets read to understand hardware behavior; their source code never does — everything here is an original implementation. Cite `nesdev-org/MesenCE`, not the archived `SourMesen/Mesen2`.

**Multi-core intent:** `Cores/Nintendo/Venus - SNES/` (SNES) sits alongside reserved sibling folders for every other planned core, per the Sailor Moon-themed naming scheme in `EmuSen_Core_Naming_Scheme.md`. The debug toolchain (`Debug/IDebugTarget.cs` and everything built on it) is already genuinely core-agnostic by design — the one piece of the multi-core intent that's actually been built and proven useful, not just planned. The C# namespaces now match the folder structure (`EmuSen.Cores.Nintendo.Venus.Memory`, not a generic `EmuSen.Memory`) — the rename this section used to flag as deferred is done.

**Working style established across this project, worth preserving:** verify hardware claims against real documentation (SNESdev wiki, fullsnes, SnesLab, sometimes SMWCentral for game-specific behavior) before implementing, not from memory alone. State confidence honestly — "verified against docs," "verified against real captured data," and "built carefully but unverified" are different claims and get labeled differently throughout this project's own comments and docs. When something can't be build-tested (common — the environment producing this plan often can't compile/run the project), say so plainly rather than implying more certainty than earned.

---

## 1. Current status summary

**Solid, verified, not under active question:** the CPU/SPC700 opcode tables (both verified 256/256 against oxyron.de), all 7 PPU background modes with correct per-mode bit depth and compositing order (verified this session against the SNESdev wiki's priority tables, with two real bugs found and fixed — Mode 0's missing BG4 priority split, Modes 2-5 using the wrong compositing order), sprite rendering (including a real Y-wraparound bug fixed this session), open-bus emulation, SRAM sizing/mirroring (the actual fix for Super Metroid's anti-piracy boot check), hi-res output (both the pseudo-hi-res interleave and true Mode 5/6 tile-pairing), and a debug toolchain (`IDebugTarget`/`SnesDebugTarget`/`DebugCommandProcessor`/`WatchRegistry`) built specifically to be core-agnostic and reusable.

**Actively in progress, not resolved:** SMW's coins and Yoshi don't render. Long investigation, substantially narrowed. Ruled out with real evidence: the sprite/tile rendering logic itself, the general DMA transfer mechanism, DMA source-address computation, and every write-path bug that could have hidden real writes from the debug watch (direct bank-$7E addressing, the WMDATA `$2180` port, DBR-relative absolute addressing, block-move, and the low-RAM `offset < 0x2000` path — all now correctly observed).

**Two separate WRAM ranges confirmed empty across every write path, watched from true power-on:** `$7E8000-$7E97FF` (the graphics-upload staging buffer DMA Channel 2 actually reads from, thousands of times per session) and `$0D80-$0D9F` (a job table — source pointer + pending-item counts — the real DMA-trigger routine at `$00A300` reads to decide what to upload). Both ranges are read constantly but never written, in every session captured so far. `$00A300` (not `$00A317`, which is just partway through the same routine) was found via a ground-truth CPU trace and confirmed to be a dual-purpose dispatcher — one branch fires a palette/CGRAM upload from bank $00 (ROM-resident, static), the other fires the graphics/VRAM upload from bank $7E (the WRAM staging buffer). Neither branch writes WRAM itself; both just read the job table and fire DMA from whatever's there.

**Current lead:** find whoever populates the `$0D80-$0D9F` job table — that's one level upstream of everything traced so far, and is the thing that actually decides whether Yoshi's slot points at real data. A second power-on-registered watch on this range (alongside the existing `$8000-$97FF` one) is now wired into `Program.cs`, so the next full play session through the Yoshi scene should show whether — and from what PC — this table ever gets written.

**Recently done, worth knowing about even though it's "finished":** a decoupling pass separating the debug toolchain from core emulation classes (`MemoryBus` no longer references `Cpu` or any debug type directly — see `IWriteObserver`/`DebugPcProvider`), extracting the hardware multiply/divide unit into its own `MathUnit` class, and extracting the entire NMI/H-V-IRQ/vblank subsystem into its own `InterruptController` class (done as ONE cohesive extraction after an earlier, more naive split attempt was caught as architecturally wrong mid-review — worth remembering if tempted to split interrupt-adjacent state again). `Frontend/Program.cs`'s ~350-line single `Main` method was also split, pulling all hotkey dispatch into its own `RunHotkeys` method separate from the actual per-scanline timing loop.

---

## 2. Phased game plan

### Phase 1 — Finish the active investigation

- **Continue the coin/Yoshi WRAM trace.** Get an actual log with the WRAM watch registered and read it. This is the single most "in-flight" piece of work — pick this up first if nothing else has changed since this doc was written.

### Phase 2 — Verification passes on recent, unverified-by-build work

- **The 65816 disassembler** (`Snes65816Disassembler`) was built carefully against the standard opcode matrix and spot-checked against two real instructions from the Yoshi investigation, but never given the equivalent scrutiny the execution opcode table got (a dedicated pass against oxyron.de). Worth a real verification pass, not just trusting it.
- **Mode 7 EXTBG and offset-per-tile** (Modes 2/4/6) are implemented against documentation but have never been exercised by an actual ROM that uses them — same "implemented but unproven" category tile16/mosaic were in before SMW's own logs confirmed them safe. Get a test ROM.

### Phase 3 — Remaining architectural decoupling

Two items were identified in an architecture review and deliberately *not* rushed:

- **The `Renderer`/Raylib split.** `Renderer` still mixes pure pixel computation with Raylib window/texture ownership, even in headless mode. Real coupling issue, but this is core rendering code with a lot of delicate, hard-won accuracy work riding on it (the entire Mode 0-7 reconciliation, hi-res, mosaic, etc.) — treat as its own dedicated, carefully-planned pass, not a quick cleanup alongside something else.
- **Watchpoints beyond WRAM.** `MemoryBus`'s `IWriteObserver` hook (used by the debug toolchain's watch mechanism) is only wired into the WRAM write path today. Extending it to `Ppu`'s VRAM/CGRAM/OAM writes and the general CPU-bus/SRAM path is a natural, low-risk, additive follow-up whenever a specific investigation needs to watch one of those.
- ~~The drive-by namespace rename~~ — done: `Cores/Snes/` → `Cores/Nintendo/Venus - SNES/`, matching namespace rename applied throughout.

### Phase 4 — Smaller feature completions

None of these are architecturally risky; they're just not done yet:

- **Audio output** — the S-DSP synthesizes genuinely correct samples (BRR decode, ADSR/GAIN envelopes, 8-voice mixing) but nothing plays them to an actual output device. Silent but correct.
- **Decimal (BCD) mode** on the 65816 — SED/CLD correctly toggle the D flag, but ADC/SBC never check it. Rare in practice (few SNES games use decimal mode), low priority.
- **The stuck HDMA title-screen window bug** — isolated but never root-caused; windowing was previously disabled globally to work around it, then re-enabled once judged lower-risk than leaving every window-based effect broken everywhere. Worth a dedicated investigation now that windowing is confirmed safe to leave on.

### Phase 5 — Debugger infrastructure (foundation for Phase 6/7)

- **Breakpoints / single-step / pause-resume.** Done. `VenusCore.RunFrame()` can now halt mid-frame (`IsHaltedAtBreakpoint`/`HaltedAddress`), preserving all scanline-loop state so the next call resumes the same in-progress frame; `IDebugTarget.Breakpoints` (`BreakpointRegistry`) backs the console's `break add`/`list`/`remove` commands and F4 prompt `step`/`continue`. See `EmuSen_Debugging_Tools_Reference_v5.md` §3.1/§3.3 for the full picture. Not yet wired into `EmuSen.HeadlessDebug` (its `break add` only counts hits, doesn't halt) or the Avalonia GUI (Phase 6, still not started).
- **Headless harness for AI-agent-driven debugging.** Done — `EmuSen.HeadlessDebug`, a console entry point loading a ROM and driving `SnesDebugTarget`/`DebugCommandProcessor` exactly as originally envisioned here, plus scripted input (`--tap`), save-state load/save, screenshot capture, windowed CPU/SPC700 tracing, and generic `DebugSettings` flag control from the command line. This became the primary tool for real investigation work (the LttP color-math/subscreen chase and the Super Metroid boot-hang investigation both ran almost entirely through it) rather than staying a one-off. Full reference: `EmuSen_Debugging_Tools_Reference_v5.md` §3.15.

### Phase 6 — The Avalonia GUI debug window

- Gated on Phase 5 existing (or at least on deciding it's not needed for a first version — a read-only, Mesen-style multi-pane viewer *could* be built against today's query-only `IDebugTarget` without pause/step, if that's judged useful enough on its own).
- `EmuSen.TestingStudio` (the Avalonia GUI, renamed from `EmuSen.Frontend`) currently has *zero* debug-tooling integration — no watch/cheat/disasm/command-prompt UI, only ROM loading, save states, controller rebinding, and (as of the log/ROM-directory + ROM-browser work) session preferences. This phase is specifically about giving *that* project a real debug window, not the separate future EmulationStation-style launcher - a stub reference doc for this was planned (`EmuSen.Frontend_Debugging_Tools_Reference_STUB.md`) but never actually written; treat this bullet as the current honest starting point instead, including an explicitly-unverified question (does `EmulatorSession` expose enough state to construct a debug target the way it already exposes `Bus` for input?).

### Phase 7 — A second `IDebugTarget` implementation, and NES core work generally

- **This is the big one, and it's the thing everything else has been implicitly building toward.** The debug toolchain's core-agnostic design has never been proven against a second implementation — only asserted to be generic.
- Per earlier project discussion, NES core work is planned to begin only once SNES + the Avalonia frontend are stable — not a new decision, a standing one.
- **This phase is also the trigger for the launcher vision's own Phase 2** (see `EmuSen_Launcher_Multicore_Gameplan.md`) — that plan's multi-core abstraction work is explicitly gated on a second real core existing to validate it against, and this is that core. Note that plan now targets a distinct future launcher project, not `EmuSen.TestingStudio`.

---

## 3. What a fresh session should do with this document

1. Read §0-1 for orientation, then check the "actively in progress" item in §1 (the coin/Yoshi investigation) — if it's since been resolved, that section of this doc is stale and should be updated or removed, not trusted as still-current.
2. Work through the phases roughly in order, but Phase 1 (finish the active investigation) and Phase 2 (verification passes) are both low-risk and don't block each other — either is a reasonable place to actually start.
3. **Phase 7 is the one thing that changes everything downstream of it** — once real NES work starts, expect this document (and the frontend's own game plan) to need real revision based on what that work actually reveals, not just checked off as "done per plan."
4. If asked to do architecture work not listed here, check whether it's the kind of thing this project already has a stated policy about (e.g. "verify against real docs before implementing," "don't build debug-toolchain abstraction past what's proven useful," "state confidence honestly") before improvising a new approach.
