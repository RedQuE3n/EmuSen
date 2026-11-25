# Contributing to EmuSen

EmuSen is a clean-room multi-console emulator written in Rust with a Qt 6 frontend. Contributions are welcome across native cores, platform services, Qt 6 views and themes, DianaSh tools, tests, and documentation.

---

## Branching Model

- **`WiseMan`** is the primary integration branch. Branch feature and fix work from `WiseMan` and merge verified commits back to `WiseMan`.
- **`main`** tracks release snapshots promoted from `WiseMan`.
- Keep pull requests focused on a single concern, rebase or merge cleanly onto `WiseMan`, and never push unverified changes directly to `main`.

---

## Recommended Reading Order

Before modifying code, read the live documentation in this order:

1. **[Docs/EmuSen Manual/README.md](Docs/EmuSen%20Manual/README.md)** — Index of all 12 manuals grouped by audience; start with [Overview_and_Architecture.md](Docs/EmuSen%20Manual/Overview_and_Architecture.md) and [Building_and_Running.md](Docs/EmuSen%20Manual/Building_and_Running.md).
2. **[Docs/Man pages/README.md](Docs/Man%20pages/README.md)** — Hardware specifications, shared chip contracts (`SilverMillennium-HW`, `Beryl-HW`, `BlackMoon-HW`), open firmware ABI tables, and per-core accuracy boundaries.
3. **[Docs/man/](Docs/man/)** — Plain-text Unix manual pages for `dianash` builtins, text utilities, debugger commands, and developer checkers.
4. **[Testing/README.md](Testing/README.md)** and **[Docs/EmuSen Manual/Testing_and_Verification.md](Docs/EmuSen%20Manual/Testing_and_Verification.md)** — Test tiers, blast-radius rules, and verification gates.

---

## Architectural Principles

### Unix Modularity and Narrow Boundaries

EmuSen follows Unix design discipline: small, single-purpose crates communicating across explicit, stable contracts.

- **Core ABI v1 (`EmuSen/Cores/Shared/emusen-native/include/emusen_core.h`):** Native cores export only the C functions defined in `emusen_core.h` (currently v1.19) and describe themselves via build-time `.core.json` sidecars. Additive minor versions never remove or alter existing v1 symbols, enforced by `emusen-core-abi-check` and `emusen-core-conform`.
- **Platform C-ABI (`Platform/*/include/*.h`):** `Platform/` exposes nine C headers (`emusen_platform.h`, `emusen_crystal.h`, `emusen_session.h`, `emusen_library.h`, `emusen_library_tool.h`, `emusen_games_service.h`, `emusen_scrape_service.h`, `dianash.h`, `dianash_module.h`). Preserve caller-owned buffer sizing, UTF-8 contracts, and `last_error` semantics on every change.
- **Zero logic in QML (`Frontend/qml-rules`):** QML files in `Frontend/` are strictly declarative views over read-only Rust/Qt models (`qt-models`, `kit-core`). State transitions, formatting, sorting, and persistence belong in Rust, never in JavaScript blocks inside QML.
- **Fenced pure-Rust crates:** `Frontend/kit-core`, `Frontend/qml-rules`, `Frontend/esde`, `Frontend/theme-check`, `Frontend/theme-check/manifest`, and the `*-client` crates stay outside `Frontend/Cargo.toml`'s workspace with independent lockfiles so headless tools never pull in Qt 6.
- **One owner per fact:** Place console metadata in `Platform/systems/data/`, paths and store filenames in `Platform/galaxia/src/{tree,stores}.rs`, and library build lists in `xtask/libraries.tsv` instead of duplicating tables across crates.

### One-Line Source Comments

Keep source comments terse and high-signal. Prefer a single-line comment stating *why* an invariant, hardware quirk, or contract exists rather than multi-paragraph prose, dated logs, or volatile line-number cross-references.

---

## Core Accuracy and Two Faces Rules

EmuSen enforces a transparent accuracy hierarchy across all native cores (see [Overview_and_Architecture.md](Docs/EmuSen%20Manual/Overview_and_Architecture.md)):

1. **No per-game hacks:** Never branch emulator behavior on a game title, serial number, header string, or ROM checksum. Every fix or optimization must model general hardware behavior or an explicitly documented accuracy tier that applies uniformly to all software.
2. **Two Faces (`Cycle` vs `Byte`):** Where a core offers a faster execution path alongside its cycle-ordered reference (currently implemented in Sapphire via `board.round == 0` vs batched execution), both faces share one semantic implementation and one save-state format. Never bend the cycle-ordered reference for speed; verify the fast face against both the reference and independent hardware test programs.
3. **Document accuracy boundaries honestly:** Any intentional approximation, open HLE limitation, or unbuilt attachment stub (such as Nephrite's 32X stub) must be stated clearly in the core's manual under `Docs/Man pages/Hardware/`.

---

## Clean-Room, ROM, and Firmware Rules

- **Clean-room sources only:** Implement hardware from public documentation, hardware measurements, test ROMs, and black-box observation. Do not copy code from incompatible licences or proprietary SDKs.
- **No commercial ROMs in the repository:** Never commit commercial game images, assets, or copyrighted captures. Automated tests must rely exclusively on synthetic, procedurally generated, or openly licensed test programs.
- **Zero bundled firmware:** Never commit copyrighted BIOS, boot ROM, or DSP firmware dumps. Systems requiring boot code must either ship a clean-room open replacement or load optional player-supplied dumps at runtime from `home/Firmware/`.

---

## Testing and Verification

EmuSen uses blast-radius selection and strict concurrency discipline so verification stays fast and deterministic:

- **Blast-radius selection:** Run the test suites covering every crate and boundary touched by your change. Consult `Testing/inputs.json`, `Testing/tiers.json`, and the `emusen.testing` DianaSh module (`units`, `impact`, `trial`) to determine the required test scope.
- **Parallel non-GPU tests, serialized GPU tests:** CPU-only unit tests, core conformance checks, `qml-rules`, `esde-check`, and headless software QML checks may run in parallel. Tests that acquire a Vulkan GPU device or compositor session (`*_gpu` tests, Vulkan compute core tests, `emusen-qt-play` gates) must run one at a time to avoid GPU/compositor contention.
- **Cargo profiles:** Use `--profile test-fast` (`lto = false`, `codegen-units = 16`, incremental) for fast iteration on `Platform/`, and `--profile test-checked` (`overflow-checks = true`) before landing changes touching arithmetic or memory indexing.

### Pre-Landing Verification Checklist

- **Core changes (`EmuSen/Cores/`):** Run the affected core's `cargo test --release`, `emusen-core-abi-check`, `emusen-core-conform`, and `cargo xtask check-sidecars`.
- **Diamond OpenKernel doc changes:** Keep `Docs/Man pages/Hardware/Sony/Diamond - PlayStation/Diamond_OpenKernel.md` at its exact path with its verbatim call table and verify with `cargo test --test page` in `EmuSen/Cores/Sony/Diamond - PlayStation/kernel-harness`.
- **Platform changes (`Platform/`):** Run affected crate tests (`cargo test --manifest-path Platform/Cargo.toml --profile test-fast -p <crate>`) and rebuild `emusen-platform` / `dianash`.
- **Frontend & QML changes (`Frontend/`):** Run `qml-rules`, `theme-check`, pure-Rust crate tests (`kit-core`, `esde`), and affected `qt-test` / `qt-app` suites; if `Docs/man/emusen.dev/theme-check.1` changes, run `cargo test --manifest-path Frontend/Cargo.toml -p emusen-theme-audit`.

---

## Commits, Documentation, and Licences

- **Factual commit messages:** Write concise, present-tense messages stating *what* changed and *why*. Never claim tests were run if they were not executed.
- **Documentation style:** Keep all manuals under `Docs/` present-tense, factual, and human-scannable. Document how the codebase works today—never append chronological diaries, dates, raw debugging transcripts, or claims about removed components. Maintain one authoritative owner per topic and use valid relative links.
- **Man page formatting:** Pages under `Docs/man/` use plain-text Unix layout (`NAME`, `SYNOPSIS`, `DESCRIPTION`, `OPTIONS`, `EXAMPLES`, `EXIT STATUS`, `SEE ALSO`), LF-only line endings (`Docs/man/.gitattributes`), 4-space body indentation, and globally unique case-insensitive filename stems.
- **Licences and third-party notices:** All original EmuSen code is licensed under `GPL-3.0-or-later` ([LICENSE](LICENSE)). Whenever a new external crate, tool, or shader dependency is added, update [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) and place its upstream licence text in [`licenses/`](licenses/).
