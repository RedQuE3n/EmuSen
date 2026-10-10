# Contributing to EmuSen

This page is for someone joining the project: how the repositories fit together, where to start reading, and the
conventions every change is held to. The README says what EmuSen is and how to build it.

## The repositories and branches

- **EmuSen** (this repository). Development happens on `WiseMan`; `main` follows it and is kept at the same commit, so
  a fresh clone of the default branch is current.
- **EmuSen.LunaP**, the interface toolkit, is a separate repository, cloned beside this one on its `openemu-library`
  branch (`LunaP.props` points at it). A change that spans both is merged and pushed to LunaP first, then to EmuSen,
  so EmuSen never points at LunaP code that is not published.
- Work happens on a branch per task and is merged into `WiseMan` with a merge commit once it is reviewed.

## Where to start reading

The manual lives inside the repository, as DianaSh's home folder:

1. `README.md`, for what works and how to build.
2. `EmuSen.DianaSh/DianaSh/Etc/Man pages/README.md`, the index of every page, one line each on what it holds.
3. `EmuSen_Project_Overview_v2.md`, the architecture and the directory map.
4. `EmuSen_Stack.md`, which language and store owns what, and why.
5. `EmuSen_Surfaces.md`, the boundaries between components, measured, and the surfaces they are moving to.
6. Then the page of the component you are working on: a core's folder under `Etc/Man pages/Hardware/`, or the
   component's page in `Usr/Home/Documents/EmuSen Manual/`.

Design pages record decisions with their dates and reasons, and mark what was measured, read from a source, argued
or estimated. Read a component's design page before changing its surface.

## Code

Every change, in C# and in Rust, is held to these:

- **Modular and Unix-like.** One job per component, crate, module or class. Small parts that compose; files, text
  and plain data as interfaces where they fit. A component stands alone where it reasonably can: the shell runs
  without an emulator, a core without a frontend.
- **Defined surfaces.** Every boundary is a written, versioned interface: the core ABI (`EmuSen_CoreAPI.md`), the
  platform library's C interface (`EmuSen_RustPlatform.md` §3), DianaSh's (`EmuSen_DianaSh.md`). Nothing crosses a
  boundary otherwise: no reflection into another component's internals, no shared mutable statics used as wires, no
  new dependency between components without saying so in the change. Types are internal by default, and a surface is
  narrowed rather than widened.
- **Reusable.** What two places need goes into one shared crate or assembly, not into copies: `emusen-native` and
  Beryl-HW for cores, Galaxia for paths and settings, Endymion for input and audio. Each fact has one owner.
- **Plain and correct.** The simplest design that is correct, with clear names. Prove a defect with a test before
  fixing it, and show the test passing after. Measure before optimising.
- **Efficient, never at accuracy's cost.** No allocation per frame on a hot path and no needless copies, measured
  with the bench or the sampler. Speed that costs accuracy is a setting the player opts into, never a default.
- **A feature that would break these** (a second job for a component, a coupling, a bloated surface) takes another
  form: an extension of an existing surface (an optional capability, a minor version), a module or plugin, or a
  component of its own. Say which form, and why, in the proposal.

**Comments are one line in any `.cs` file.** A comment points to where the explanation lives. Design rationale, edge
cases and investigation history go in the manual: a DianaSh command's in its `man` page, everything else in the page
the index lists for that component, created if none exists.

**Moving C# to Rust** happens behind a switch (`EMUSEN_<COMPONENT>_NATIVE`), with the C# kept as the oracle and the
fallback until the Rust has held to it (parity tests, golden outputs). C# is deleted only after that, and by
decision, never as a side effect.

## Accuracy, clean room and firmware

- Cores are written from hardware documentation, test programs and measurement. **No emulator's source code is
  read or copied.** Reference emulators are run as black boxes and their outputs compared; hardware models (FPGA
  cores, transcribed chips) are run, not read, except where a dispute log records a narrow, stated exception. Each
  core's references are cited in the README and in its own pages.
- **Firmware is never shipped, fetched or prompted for.** Every core has an open replacement; a player's own dump is
  optional (`EmuSen_Firmware.md`).
- A disagreement between references is settled by a test program, then by a hardware model, and is recorded in the
  core's disputes log with what was read and measured.

## ROMs and data

- **Commercial ROMs are never committed.** Tests build their own images (`EmuSen.WiseMan`'s `SyntheticRom` and each
  system's builders).
- A player's ROM library is read, never written or deleted, by the program or by any tool or test. Saves, coverage
  maps and caches are local data and gitignored.

## Tests

- Tests run headless through `EmuSen.WiseMan`; nothing opens a real window. If the harness cannot express a check,
  the harness is extended.
- Run the tests in the blast radius of a change: the code it touches, what calls it, what shares state with it. The
  whole suite runs before a release and now and then to catch drift, not after every change.
- A change to what a recorded output prints (DianaSh's golden corpus, a probe baseline) is a deliberate change: the
  golden file is re-recorded in a commit that says why.

## Commits and pages

- A commit message says what changed and why, in plain prose, ending with its last paragraph. A decision is stated
  with its date: "Decided 2026-09-27: F4 opens the Start menu".
- Pages and messages write "the player" for whoever uses the software and "the tester" for whoever tested, reported
  or decided something, and "measured on <date>" for a measurement.
- Manual pages are written in an academic register: what was measured and how, the argument kept apart from the
  mechanism, predictions recorded and marked when they are retired.
- A dependency added or updated is listed in `THIRD_PARTY_NOTICES.md` with its licence, and its licence text added to
  `licenses/` when it is new.

## Licence

EmuSen is GPL-3.0 (`LICENSE`). LunaP is MIT. A contribution is made under the licence of the repository it goes into.
