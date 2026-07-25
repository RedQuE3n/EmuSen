# EmuSen.Frontend — Multi-Core Launcher Game Plan

*(Written to be self-contained: if you're a fresh chat session reading only this file, this section and the next should be enough to orient you before the phased plan starts. Companion docs — `EmuSen.Frontend_Project_Overview.md` and `EmuSen.Frontend_Debugging_Tools_Reference_STUB.md`, both in this same `Man pages` folder — have the deeper technical detail this doc deliberately doesn't repeat.)*

---

## 0. Orientation, if you're new to this

**EmuSen** is a SNES emulator written in C# / .NET 10, two sibling projects on disk: `EmuSen/` (the core CPU/PPU/APU/memory emulation, plus a Raylib-based console frontend used for development/debugging) and `EmuSen.Frontend/` (an Avalonia GUI frontend — this document's subject). Both reference the same core code; `EmuSen.Frontend` currently opens exactly one SNES ROM at a time in one window, with no concept of "other systems" at all.

**The vision, stated by the project owner directly:** evolve `EmuSen.Frontend` from "a window for the SNES core" into something with two blended influences — **RetroArch's** core-agnostic, plug-in-any-system runtime, and **EmulationStation's** user-friendly, themeable, heavily-configurable library-browsing front-end. Not "someday support two systems" — a genuine launcher/shell product, eventually.

**Why this document exists:** this is a big, multi-year-shaped ambition sitting on top of a project that (as of this writing) has exactly one working core (SNES) and hasn't started a second one yet. The explicit agreement reached before this document was written: **don't build speculative abstraction ahead of a second real core proving it's needed.** This plan is sequenced so early phases are useful and low-risk on their own, and the big, launcher-shaped work is *later*, gated on things that don't exist yet (most importantly, an actual second emulated system).

**Current, immediate, already-scoped next step** (independent of everything else in this doc): a coupling cleanup where `ControllerKeyMap`, `GamepadBindingMap`, `GamepadManager` are hardcoded to `SnesButton` instead of being generic over "whatever button enum the active core uses." This was already reviewed in detail in a prior session and is Phase 1's first concrete task below — see `EmuSen.Frontend_Project_Overview.md` §2 and §5 for the full review that led here.

---

## 1. The four-layer breakdown

Four genuinely separate concerns get conflated if you're not careful, and RetroArch and EmulationStation each really only solve *one or two* of them, not all four:

| Layer | What it is | Exists today? |
|---|---|---|
| **1. Core abstraction** | A contract (`ICoreSession`-shaped) that any emulated system's session implements, so the frontend can drive "whichever core is loaded" instead of a concrete `EmulatorSession`. Includes generic input (the `SnesButton` problem). | No — `EmulatorSession` is used as a concrete type everywhere. |
| **2. Content/library** | "Systems" as data (a core + a ROM folder + its own metadata/art), a ROM scanner, a game list, favorites/recently-played. | No — today's whole experience is "File → Open ROM," once, per session. |
| **3. Presentation/theming** | A themeable, skinnable UI for *browsing* the library from layer 2 — this is EmulationStation's actual specialty. `MainWindow` as it exists today would likely become "the thing you land in after picking a game," not the app's first screen. | No. |
| **4. Configuration depth** | Real settings: video/audio/per-system/per-game overrides. Today there's exactly one settings surface (`InputSettingsWindow`, input-rebinding only). | Minimal — input only. |

**Sequencing logic:** Layer 1 is the only one that's *architecturally* blocking — you can't build a "pick your system" browser (layer 2/3) on top of a frontend that only knows how to talk to one concrete core type. Layers 2-4 are genuinely independent of each other in principle, but layer 3 (theming/browsing UI) is much more valuable once layer 2 (an actual library, not just one ROM) exists, so they're sequenced together below.

---

## 2. Phased game plan

Each phase lists: what it delivers, why it's sequenced where it is, and what stays explicitly *undecided* until that phase actually starts (so a future session doesn't accidentally treat a placeholder as a made decision).

### Phase 1 — Core abstraction layer (the RetroArch part, minimal viable version)

**Goal:** make the frontend's dependency on "the SNES core specifically" explicit and swappable, without yet having a second core to prove it against.

- **1a. Fix the `SnesButton` coupling** (already scoped, ready to execute): genericize `ControllerKeyMap`/`GamepadBindingMap`/`GamepadManager` over the button-enum type. `InputSettingsWindow` itself stays non-generic (Avalonia XAML code-behind + C# generics is real, documented friction, not to be attempted casually) — it should consume a type-erased view instead of touching the button enum type directly. Full review already done; see the Project Overview doc's §2 and §5.
- **1b. Define an `ICoreSession`-shaped interface** that `EmulatorSession` implements — covering whatever `MainWindow` actually needs today (load ROM, run frame, get frame buffer, save/load state, expose an input sink, expose a frame width). Model this the same way `IDebugTarget` was built: a genuine contract, not a speculative kitchen sink — only add what's actually used today, resisting the urge to guess what a hypothetical NES session would need beyond that.
- **1c. `MainWindow` depends on the interface, not `EmulatorSession` concretely.** This is the payoff step — if done right, `MainWindow`'s code barely changes, it just stops hard-coding the concrete session type.

**Explicitly NOT decided yet, don't assume an answer:** whether a second core, when it exists, gets its own frontend *project* (sharing only non-XAML library code) or plugs into this same `EmuSen.Frontend` — see Phase 2's first bullet. Phase 1 is deliberately useful either way.

### Phase 2 — Wait for a second real core, then validate the abstraction

**Goal:** don't guess. Prove Phase 1's abstraction against a real second system before building anything bigger on top of it.

- This phase's *trigger* is "a second core (most likely NES, per the core project's own long-term roadmap) reaches a working, testable state" — not a fixed date. Everything past this point in this document is explicitly gated on that happening.
- **Resolve the open question from Phase 1c**: does the second core plug into `EmuSen.Frontend`, or get its own project? Answer this with real information (how different does the second core's `ICoreSession` implementation actually turn out to be? how much of `MainWindow`/rendering/input genuinely generalizes?) rather than guessing now.
- If `ICoreSession` needs changes to fit the second core, that's expected and fine — the point of this phase is finding out, not being right on the first try.

### Phase 3 — Content/library layer (EmulationStation's data side)

**Goal:** "systems" and "games" become real data, not just "whatever ROM I opened this session."

- A "system" concept: a core + a ROM directory (or directories) + associated metadata/art conventions.
- A ROM scanner producing a game list per system.
- Basic per-game state: favorites, recently-played, last-played timestamp.
- Metadata/box-art sourcing is a real open question (scrape a database? read local `.nfo`-style sidecar files? both?) — don't decide this speculatively inside this document; it's its own research task when this phase starts.

### Phase 4 — Presentation/theming layer (EmulationStation's actual specialty)

**Goal:** a themeable, skinnable browsing UI sitting in front of the library from Phase 3, with `MainWindow` (or its Phase-1-abstracted descendant) becoming "the gameplay view you land in after picking something," not the first screen.

- Needs a real decision on *how* themes are authored/loaded (a defined theme-file format? Avalonia's own styling/resource system reused directly? something closer to EmulationStation's XML theme trees?) — this is a meaningful design decision, not a small implementation detail, and shouldn't be pre-decided here.
- This is the single biggest, most user-visible piece of the whole vision, and the most likely to benefit from being scoped into its *own* sub-phases once it actually starts (a full theming engine is a project in itself).

### Phase 5 — Configuration depth

**Goal:** real settings beyond input rebinding — video scaling/filters, audio, per-system and per-game overrides.

- Lowest architectural risk of the four content phases (2-5), and could in principle happen in parallel with 3/4 rather than strictly after — sequenced last here mainly because it's the least urgent, not because it depends on the others.

---

## 3. What a fresh session should do with this document

1. Read this file's §0 for orientation, then check whether Phase 1 (the `SnesButton` fix + `ICoreSession` interface) is done — check `EmuSen.Frontend_Project_Overview.md`'s own TODO section for current status, since that doc gets updated as work actually lands and this one may lag slightly behind it.
2. If Phase 1 is done and no second core exists yet, **the correct next action is usually "work on the core project, not this one"** — Phase 2 is gated on a second core existing, and there's genuinely nothing productive to do on the launcher vision until then. Check `EmuSen_Project_Overview.md`'s own game-plan companion doc for what's next on that side.
3. Don't start Phase 3+ work speculatively "to make progress" — the whole point of this sequencing is that layers 2-4 built against an unproven Phase-1 abstraction risk being thrown away or reworked once a real second core exposes what that abstraction actually needed to look like.
