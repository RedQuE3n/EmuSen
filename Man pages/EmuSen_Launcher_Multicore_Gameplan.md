# EmuSen Launcher — Multi-Core Game Plan

*(Written to be self-contained: if you're a fresh chat session reading only this file, this section and the next should be enough to orient you before the phased plan starts. A companion doc, `EmuSen.Frontend_Project_Overview.md`, was originally planned to hold deeper technical detail this doc deliberately doesn't repeat — it and a matching debugging-tools stub were never actually written; treat `EmuSen_Project_Overview_v2.md` and `EmuSen_Debugging_Tools_Reference_v5.md` as the real technical-detail companions instead, keeping in mind neither is scoped around this launcher vision specifically.)*

---

## 0. Orientation, if you're new to this

**EmuSen** is a SNES emulator written in C# / .NET 10, several sibling projects on disk (see `EmuSen_Project_Overview_v2.md` §1 for the current full list): `EmuSen/` (the core CPU/PPU/APU/memory emulation), `EmuSen.RaylibFrontend/` (a console frontend used for development/debugging), `EmuSen.Serenity/` (shared presentation/shader code), and `EmuSen.TestingStudio/` (an Avalonia GUI - this document's subject **used to be** this project, back when it was named `EmuSen.Frontend/`).

**A decision this document's original version explicitly left open has since been made:** `EmuSen.Frontend` was renamed to `EmuSen.TestingStudio` and deliberately scoped as bug-testing tooling for testers (ROM loading, save states, controller rebinding, a configurable log directory + ROM directory + ROM browser) — **not** as the seed of the launcher vision described below. That vision is scoped for a **separate, not-yet-created project**, specifically so bug-testing tooling and a future polished launcher never have to share one codebase, risk breaking each other, or awkwardly carry each other's UI baggage (a menu bar and a Preferences window don't belong in an EmulationStation-style browsing shell, and launcher chrome doesn't belong in a tester's quick ROM-loading tool). This resolves part of old Phase 1c/Phase 2's "explicitly not decided" question below: it will not be built *inside* `EmuSen.TestingStudio`. What's still genuinely open is *how much*, if any, of `EmuSen.TestingStudio`'s existing code (its `ControllerKeyMap`/`GamepadBindingMap`/`GamepadManager`, `EmulatorSession` usage patterns) gets reused/forked into that new project versus written fresh — don't assume an answer to that either.

**The vision, stated by the project owner directly:** a genuine launcher/shell product with two blended influences — **RetroArch's** core-agnostic, plug-in-any-system runtime, and **EmulationStation's** user-friendly, themeable, heavily-configurable library-browsing front-end. Not "someday support two systems" — eventually, pick a system, browse its library, launch a game, all from one polished shell.

**Why this document exists:** this is a big, multi-year-shaped ambition sitting on top of a project that (as of this writing) has exactly one working core (SNES) and hasn't started a second one yet. The explicit agreement reached before this document was originally written: **don't build speculative abstraction ahead of a second real core proving it's needed.** This plan is sequenced so early phases are useful and low-risk on their own, and the big, launcher-shaped work is *later*, gated on things that don't exist yet (most importantly, an actual second emulated system, *and now also* the launcher project itself, which doesn't exist yet either).

**Current, immediate, already-scoped next step** (independent of everything else in this doc): a coupling cleanup where `ControllerKeyMap`, `GamepadBindingMap`, `GamepadManager` (today living in `EmuSen.TestingStudio/Input/`) are hardcoded to `SnesButton` instead of being generic over "whatever button enum the active core uses." This was already reviewed in detail in a prior session and is Phase 1's first concrete task below.

---

## 1. The four-layer breakdown

Four genuinely separate concerns get conflated if you're not careful, and RetroArch and EmulationStation each really only solve *one or two* of them, not all four:

| Layer | What it is | Exists today? |
|---|---|---|
| **1. Core abstraction** | A contract (`ICoreSession`-shaped) that any emulated system's session implements, so the launcher can drive "whichever core is loaded" instead of a concrete `EmulatorSession`. Includes generic input (the `SnesButton` problem). | No — `EmulatorSession` is used as a concrete type everywhere it exists today (`EmuSen.TestingStudio`). |
| **2. Content/library** | "Systems" as data (a core + a ROM folder + its own metadata/art), a ROM scanner, a game list, favorites/recently-played. | No — `EmuSen.TestingStudio`'s whole experience is "File → Open ROM" (or, as of its own log/ROM-directory work, a flat ROM-browser list) once per session, not a persistent library. |
| **3. Presentation/theming** | A themeable, skinnable UI for *browsing* the library from layer 2 — this is EmulationStation's actual specialty. Not a window this project already has - the launcher's gameplay view (whatever plays the role `MainWindow` plays in `EmuSen.TestingStudio` today) would be "the thing you land in after picking a game," not the app's first screen. | No. |
| **4. Configuration depth** | Real settings: video/audio/per-system/per-game overrides. `EmuSen.TestingStudio` has controller rebinding and (as of its own later work) log/ROM-directory preferences, but nothing per-system or per-game, and nothing video/audio. | Minimal, and specific to the testing tool, not this launcher. |

**Sequencing logic:** Layer 1 is the only one that's *architecturally* blocking — you can't build a "pick your system" browser (layer 2/3) on top of a launcher that only knows how to talk to one concrete core type. Layers 2-4 are genuinely independent of each other in principle, but layer 3 (theming/browsing UI) is much more valuable once layer 2 (an actual library, not just one ROM) exists, so they're sequenced together below.

---

## 2. Phased game plan

Each phase lists: what it delivers, why it's sequenced where it is, and what stays explicitly *undecided* until that phase actually starts (so a future session doesn't accidentally treat a placeholder as a made decision).

### Phase 1 — Core abstraction layer (the RetroArch part, minimal viable version)

**Goal:** make "the SNES core specifically" an explicit, swappable dependency rather than a hardcoded one, without yet having a second core to prove it against. Whether this phase's code lands inside `EmuSen.TestingStudio` first (since that's where `EmulatorSession`/`ControllerKeyMap` live today) and gets forked/extracted later, or starts directly in a new project, is an implementation-order choice, not a design one - see §0's still-open question.

- **1a. Fix the `SnesButton` coupling** (already scoped, ready to execute): genericize `ControllerKeyMap`/`GamepadBindingMap`/`GamepadManager` over the button-enum type. Whatever UI ends up rebinding controls (today `EmuSen.TestingStudio`'s `InputSettingsWindow`) should consume a type-erased view instead of touching the button enum type directly - Avalonia XAML code-behind + C# generics is real, documented friction, not to be attempted casually.
- **1b. Define an `ICoreSession`-shaped interface** that `EmulatorSession` implements — covering whatever a frontend actually needs today (load ROM, run frame, get frame buffer, save/load state, expose an input sink, expose a frame width). Model this the same way `IDebugTarget` was built: a genuine contract, not a speculative kitchen sink — only add what's actually used today, resisting the urge to guess what a hypothetical NES session would need beyond that.
- **1c. Whatever plays `MainWindow`'s role depends on the interface, not `EmulatorSession` concretely.** This is the payoff step.

**Explicitly NOT decided yet, don't assume an answer:** exactly which new project this work lands in, and how much (if any) of `EmuSen.TestingStudio`'s existing input/session code gets reused versus rewritten. Phase 1 is deliberately useful either way.

### Phase 2 — Wait for a second real core, then validate the abstraction

**Goal:** don't guess. Prove Phase 1's abstraction against a real second system before building anything bigger on top of it.

- This phase's *trigger* is "a second core (most likely NES, per the core project's own long-term roadmap) reaches a working, testable state" — not a fixed date. Everything past this point in this document is explicitly gated on that happening.
- If `ICoreSession` needs changes to fit the second core, that's expected and fine — the point of this phase is finding out, not being right on the first try.

### Phase 3 — Content/library layer (EmulationStation's data side)

**Goal:** "systems" and "games" become real data, not just "whatever ROM I opened this session."

- A "system" concept: a core + a ROM directory (or directories) + associated metadata/art conventions.
- A ROM scanner producing a game list per system.
- Basic per-game state: favorites, recently-played, last-played timestamp.
- Metadata/box-art sourcing is a real open question (scrape a database? read local `.nfo`-style sidecar files? both?) — don't decide this speculatively inside this document; it's its own research task when this phase starts.

### Phase 4 — Presentation/theming layer (EmulationStation's actual specialty)

**Goal:** a themeable, skinnable browsing UI sitting in front of the library from Phase 3, with the gameplay view (whatever plays `MainWindow`'s role) becoming "the thing you land in after picking something," not the first screen.

- Needs a real decision on *how* themes are authored/loaded (a defined theme-file format? Avalonia's own styling/resource system reused directly? something closer to EmulationStation's XML theme trees?) — this is a meaningful design decision, not a small implementation detail, and shouldn't be pre-decided here.
- This is the single biggest, most user-visible piece of the whole vision, and the most likely to benefit from being scoped into its *own* sub-phases once it actually starts (a full theming engine is a project in itself).

### Phase 5 — Configuration depth

**Goal:** real settings beyond input rebinding — video scaling/filters, audio, per-system and per-game overrides.

- Lowest architectural risk of the four content phases (2-5), and could in principle happen in parallel with 3/4 rather than strictly after — sequenced last here mainly because it's the least urgent, not because it depends on the others.

---

## 3. What a fresh session should do with this document

1. Read this file's §0 for orientation, and note that **this launcher's target project does not exist yet** — don't assume it's `EmuSen.TestingStudio` just because that's the only Avalonia project on disk today.
2. If a launcher project exists and Phase 1 is done, but no second core exists yet, **the correct next action is usually "work on the core project, not this one"** — Phase 2 is gated on a second core existing, and there's genuinely nothing productive to do on the launcher vision until then. Check `EmuSen_Core_Gameplan.md` for what's next on that side.
3. Don't start Phase 3+ work speculatively "to make progress" — the whole point of this sequencing is that layers 2-4 built against an unproven Phase-1 abstraction risk being thrown away or reworked once a real second core exposes what that abstraction actually needed to look like.
4. Don't build any part of this plan inside `EmuSen.TestingStudio` - that project's whole reason for existing separately is to stay a small, stable, bug-testing tool untouched by this launcher's much larger and more volatile scope.
