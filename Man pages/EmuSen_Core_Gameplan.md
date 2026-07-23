# EmuSen (Core) — Game Plan

*(Written to be self-contained: if you're a fresh chat session reading only this file, §0-1 should orient you before the phased plan starts. Companion docs in this same `Man pages` folder — `EmuSen_Project_Overview.md` and `EmuSen_Debugging_Tools_Reference.md` — have the full technical depth this doc deliberately doesn't repeat; check those for "how does X actually work" questions, this doc is for "what's next and in what order.")*

---

## 0. Orientation, if you're new to this

**EmuSen** is a SNES emulator in C# / .NET 10 — full 65816 CPU, SPC700+S-DSP audio, PPU (all 7 background modes plus Mode 7, sprites, hi-res, mosaic, windowing, color math), memory/DMA. Two sibling projects: `EmuSen/` (this doc's subject — the core, plus a Raylib console frontend used for development) and `EmuSen.Frontend/` (an Avalonia GUI; see its own game plan doc for that side's separate, later-stage multi-core/launcher ambitions).

**Licensing/reference stance:** the SNESdev wiki (mirrored at `snes.nesdev.org` and `snesdev.mesen.ca`, maintained by the Mesen/MesenCE team) is the primary hardware reference. Mesen/MesenCE's *documentation* gets read to understand hardware behavior; their source code never does — everything here is an original implementation. Cite `nesdev-org/MesenCE`, not the archived `SourMesen/Mesen2`.

**Multi-core intent:** `Cores/Nintendo/Venus/` (SNES) sits alongside reserved sibling folders for every other planned core, per the Sailor Moon-themed naming scheme in `EmuSen_Core_Naming_Scheme.md`. The debug toolchain (`Debug/IDebugTarget.cs` and everything built on it) is already genuinely core-agnostic by design — the one piece of the multi-core intent that's actually been built and proven useful, not just planned. The C# namespaces now match the folder structure (`EmuSen.Cores.Nintendo.Venus.Memory`, not a generic `EmuSen.Memory`) — the rename this section used to flag as deferred is done.

**Working style established across this project, worth preserving:** verify hardware claims against real documentation (SNESdev wiki, fullsnes, SnesLab, sometimes SMWCentral for game-specific behavior) before implementing, not from memory alone. State confidence honestly — "verified against docs," "verified against real captured data," and "built carefully but unverified" are different claims and get labeled differently throughout this project's own comments and docs. When something can't be build-tested (common — the environment producing this plan often can't compile/run the project), say so plainly rather than implying more certainty than earned.

---

## 1. Current status summary

**Solid, verified, not under active question:** the CPU/SPC700 opcode tables (both verified 256/256 against oxyron.de), all 7 PPU background modes with correct per-mode bit depth and compositing order (verified this session against the SNESdev wiki's priority tables, with two real bugs found and fixed — Mode 0's missing BG4 priority split, Modes 2-5 using the wrong compositing order), sprite rendering (including a real Y-wraparound bug fixed this session), open-bus emulation, SRAM sizing/mirroring (the actual fix for Super Metroid's anti-piracy boot check), hi-res output (both the pseudo-hi-res interleave and true Mode 5/6 tile-pairing), and a debug toolchain (`IDebugTarget`/`SnesDebugTarget`/`DebugCommandProcessor`/`WatchRegistry`) built specifically to be core-agnostic and reusable.

**Actively in progress, not resolved:** SMW's coins and Yoshi don't render. Long investigation, meaningfully narrowed but not solved — ruled out: the sprite/tile rendering logic itself (checked against docs and against real dumped tile+palette data), the general DMA transfer mechanism (proven correct via adjacent, working animated-tile transfers), and the DMA source-address computation (confirmed varying correctly in the most recent session, not stuck the way it first appeared). Currently narrowed to: whatever's supposed to write real graphics data into the WRAM staging buffer before the DMA copies it out doesn't seem to be doing so. A targeted watch (`watch add WRAM 8000 1800` via the console build's F4 debug prompt) is the way to gather the next piece of evidence — this hasn't been done yet as of this writing.

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
- ~~The drive-by namespace rename~~ — done: `Cores/Snes/` → `Cores/Nintendo/Venus/`, matching namespace rename applied throughout.

### Phase 4 — Smaller feature completions

None of these are architecturally risky; they're just not done yet:

- **Audio output** — the S-DSP synthesizes genuinely correct samples (BRR decode, ADSR/GAIN envelopes, 8-voice mixing) but nothing plays them to an actual output device. Silent but correct.
- **Decimal (BCD) mode** on the 65816 — SED/CLD correctly toggle the D flag, but ADC/SBC never check it. Rare in practice (few SNES games use decimal mode), low priority.
- **The stuck HDMA title-screen window bug** — isolated but never root-caused; windowing was previously disabled globally to work around it, then re-enabled once judged lower-risk than leaving every window-based effect broken everywhere. Worth a dedicated investigation now that windowing is confirmed safe to leave on.

### Phase 5 — Debugger infrastructure (foundation for Phase 6/7)

- **Breakpoints / single-step / pause-resume.** Doesn't exist today — the execution loop runs a whole frame at a time with no mid-frame pause capability. This is real, separate engineering work, not a quick add, and is a prerequisite for a proper interactive debugger (as opposed to today's one-shot query commands).

### Phase 6 — The Avalonia GUI debug window

- Gated on Phase 5 existing (or at least on deciding it's not needed for a first version — a read-only, Mesen-style multi-pane viewer *could* be built against today's query-only `IDebugTarget` without pause/step, if that's judged useful enough on its own).
- The Avalonia frontend currently has *zero* debug-tooling integration — see `EmuSen.Frontend_Debugging_Tools_Reference_STUB.md` for the honest starting point on that side, including an explicitly-unverified question (does `EmulatorSession` expose enough state to construct a debug target the way it already exposes `Bus` for input?).

### Phase 7 — A second `IDebugTarget` implementation, and NES core work generally

- **This is the big one, and it's the thing everything else has been implicitly building toward.** The debug toolchain's core-agnostic design has never been proven against a second implementation — only asserted to be generic.
- Per earlier project discussion, NES core work is planned to begin only once SNES + the Avalonia frontend are stable — not a new decision, a standing one.
- **This phase is also the trigger for `EmuSen.Frontend`'s own Phase 2** (see that project's game plan doc) — the frontend's multi-core abstraction work is explicitly gated on a second real core existing to validate it against, and this is that core.

---

## 3. What a fresh session should do with this document

1. Read §0-1 for orientation, then check the "actively in progress" item in §1 (the coin/Yoshi investigation) — if it's since been resolved, that section of this doc is stale and should be updated or removed, not trusted as still-current.
2. Work through the phases roughly in order, but Phase 1 (finish the active investigation) and Phase 2 (verification passes) are both low-risk and don't block each other — either is a reasonable place to actually start.
3. **Phase 7 is the one thing that changes everything downstream of it** — once real NES work starts, expect this document (and the frontend's own game plan) to need real revision based on what that work actually reveals, not just checked off as "done per plan."
4. If asked to do architecture work not listed here, check whether it's the kind of thing this project already has a stated policy about (e.g. "verify against real docs before implementing," "don't build debug-toolchain abstraction past what's proven useful," "state confidence honestly") before improvising a new approach.
