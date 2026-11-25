# EmuSen Manual

The EmuSen Manual is the live reference for EmuSen's Rust + Qt 6 architecture, player workflows, configuration files, theming engine, C interfaces, and verification tooling.

---

## For Players

| Guide | What It Covers |
|---|---|
| [User_Guide.md](User_Guide.md) | Launching `emusen-qt` and `emusen-qt-play`, navigating the six unified screens, controller/keyboard grammar, save states, rewind & Moments, fast-forward, cheats, firmware, and opt-in Libretro cores |
| [Settings_and_Configuration.md](Settings_and_Configuration.md) | Galaxia directory tree, root discovery, `AtomicFile` durability, JSONC parsing, SQLite stores, and complete field tables for `appsettings.json`, `audio.json`, `graphics.json`, and `consoles.json` |
| [Input_and_Controllers.md](Input_and_Controllers.md) | RetroPad button/axis contract, SDL3 hotplug, per-console bindings, controller profiles, multiplayer seat claiming, dead zones, Guide chords, and rumble |
| [Audio_Video_CRT_and_Shaders.md](Audio_Video_CRT_and_Shaders.md) | SDL3 audio resampling and device selection, Vulkan slang shader chains, SPIR-V caching, frame pacing, and the three-tier physically modelled CRT filter |
| [Save_Media_and_Game_Packages.md](Save_Media_and_Game_Packages.md) | Binary save states, XOR-delta rewind, battery SRAM and removable memory cards, game packages, IPS/BPS/UPS patching, CHD conversion, ROM hashing, and ScreenScraper |

---

## For Builders and Packagers

| Guide | What It Covers |
|---|---|
| [Building_and_Running.md](Building_and_Running.md) | Toolchain prerequisites, building `Platform/`, `Frontend/`, and `EmuSen/Cores/`, `cargo xtask` commands and pins, Cargo profiles, launching binaries, and Linux/SteamOS AppImage packaging |
| [Testing_and_Verification.md](Testing_and_Verification.md) | Blast-radius test selection, parallel CPU vs serialized GPU execution, `emusen.testing` studio, headless Qt 6 testing (`qt-test`, `theme-check`, `qml-rules`), and core conformance gates |

---

## For Theme Authors and Frontend Developers

| Guide | What It Covers |
|---|---|
| [Qt_Frontend_and_Theming.md](Qt_Frontend_and_Theming.md) | 21-crate `Frontend/` workspace and pure-Rust fence, zero-logic QML rules, Theme API v1.20 (`Eclipse` and `EmuSen` themes), `theme-check` auditing, ES-DE theme engine, and `kit-gallery` |

---

## For Core and Platform Developers

| Guide | What It Covers |
|---|---|
| [Overview_and_Architecture.md](Overview_and_Architecture.md) | Four-layer architecture, workspace and crate map, Unix modularity and C-ABI/QML boundaries, Two Faces accuracy contract, and the Core and Component Naming Scheme |
| [Core_ABI_and_Libretro_Host.md](Core_ABI_and_Libretro_Host.md) | Core ABI v1.19 (`emusen_core.h`), `.core.json` sidecars, conformance and ABI checks, `emusen-native`, shared core crates, and Landlock-confined Libretro hosting |
| [Platform_C_Interfaces.md](Platform_C_Interfaces.md) | 36-crate `Platform/` workspace, `emusen_platform` C-ABI conventions, catalog of all nine C headers, declarative `Platform/systems/data/` tables, and `Platform/crystal` sessions |
| [DianaSh_and_Debugging.md](DianaSh_and_Debugging.md) | Standalone `dianash` and `nano`, shell syntax and path sandbox, all live command modules, hardware debugging workflows, and extending DianaSh via `dianash_module.h` |

---

## Companion Documentation

- **Hardware Reference Manuals:** [Docs/Man pages/README.md](../Man%20pages/README.md) — Factual hardware register maps, clocks, chip contracts, open firmware tables, and accuracy boundaries for every native Rust core and shared hardware library.
- **DianaSh Man Pages:** [Docs/man/](../man/) — Plain-text Unix manual pages for every shell builtin, text utility, debugger command, library tool, and developer checker (`man <command>` inside `dianash`).
- **Repository Front Door & Policies:** [README.md](../../README.md) and [CONTRIBUTING.md](../../CONTRIBUTING.md).
