# Overview and Architecture

EmuSen is a clean-room multi-console emulation framework built in Rust with a Qt 6 frontend. Its architecture follows Unix modularity: small, single-purpose crates separated by narrow C-ABI and read-only view contracts so hardware emulation, platform services, and user interfaces evolve independently without cross-layer entanglement.

---

## Four-Layer Architecture

```text
+-----------------------------------------------------------------------------+
|  Layer 4: Frontends & Shell                                                 |
|    Frontend/ (emusen-qt, emusen-qt-play, kit-gallery)   Platform/ (dianash) |
+---------------------------+---------------------------------+---------------+
                            | C-ABI (9 Platform headers)      |
                            v                                 v
+-----------------------------------------------------------------------------+
|  Layer 3: Platform Services (Platform/ -> libemusen_platform)               |
|    galaxia (storage/config)  endymion (SDL3 audio/input)  serenity (Vulkan) |
|    crystal (session/rewind)  library / library-tool       scrape / cheats   |
|    savemedia / firmware / systems / dianash modules                         |
+-------------------------------------+---------------------------------------+
                                      | Core ABI v1.19 (emusen_core.h)
                                      v
+-----------------------------------------------------------------------------+
|  Layer 2: Emulation Cores & Shared Hardware (EmuSen/Cores/)                 |
|    8 Native Rust Cores      3 Shared HW Families     19 Shared Core Crates  |
|    5 Libretro Host Crates (emusen-core-runner Landlock sandbox on Linux)    |
+-----------------------------------------------------------------------------+
                                      ^
                                      | Built, staged & packaged by
+-----------------------------------------------------------------------------+
|  Build Host: xtask/ (cargo xtask stage, check-sidecars, prebuilt, appimage) |
+-----------------------------------------------------------------------------+
```

Each layer depends only downward through an explicit contract:

1. **Emulation Cores (`EmuSen/Cores/`):** Self-contained shared libraries exporting Core ABI v1 (`emusen_core.h`). Cores never call back into host code; the host calls into a core and receives frames, audio samples, save media, and debug data as values or borrowed buffers.
2. **Platform Services (`Platform/`):** A single Rust workspace linked into `libemusen_platform` (plus standalone `dianash` and `nano`). Owns the directory tree, JSONC configuration, SQLite stores, SDL3 audio/input, Vulkan slang shader pipeline, runtime orchestration, save media, packages, cheats, scraping, and shell modules.
3. **Frontends (`Frontend/`):** A Rust + Qt 6 (`cxx-qt`) stack that consumes `libemusen_platform` through pure-Rust client crates (`platform-client`, `crystal-client`) and exposes read-only Qt models (`qt-models`) to declarative QML views (`qt-app`, `qt-play`, `theme-qt`, `esde-qt`, `kit-qt`).
4. **Build Host (`xtask/`):** Orchestrates native core builds, `.core.json` sidecar generation and verification, platform staging, prebuilt artifact layout, and Linux AppImage bundling from `xtask/libraries.tsv` and `xtask/appimage-tools.tsv`.

---

## Workspace and Crate Map

### `Platform/` (36 Workspace Crates, Rust 2024 Edition)

`Platform/Cargo.toml` defines a 36-crate workspace exposing nine C headers:

| Header | Subsystem |
|---|---|
| `Platform/include/emusen_platform.h` | Galaxia tree/config, Endymion audio/input, Serenity video/shaders, firmware, cheats, save media |
| `Platform/include/emusen_games_service.h` | Library scanning, collections, game metadata queries |
| `Platform/include/emusen_scrape_service.h` | ScreenScraper background scraping jobs and media index |
| `Platform/crystal/include/emusen_crystal.h` | Core loading, runtime execution, rewind ring, speed controller |
| `Platform/crystal/include/emusen_session.h` | Session inspection, debugger attachment, and trace access |
| `Platform/library/include/emusen_library.h` | Library database and game discovery API |
| `Platform/library-tool/include/emusen_library_tool.h` | Packages (`pkg`), CHD conversion (`chd`), ROM verification (`rom`) |
| `Platform/dianash/include/dianash.h` | Embedded shell interpreter host interface |
| `Platform/dianash/include/dianash_module.h` | External command module ABI for extending `dianash` |

Workspace crate grouping:

| Subsystem | Workspace Crates | Responsibility |
|---|---|---|
| **Storage & Config** | `galaxia`, `sqlite`, `suggest`, `net` | Directory tree (`tree.rs`), `AtomicFile`, JSONC config models (`models.rs`), SQLite stores, string-distance suggestions, HTTP client |
| **Audio, Input & Video** | `endymion`, `serenity`, `recorder` | SDL3 audio resampling/device selection, RetroPad input and profiles, Vulkan slang shaders, SPIR-V cache, CRT filter, frame pacing and capture |
| **Runtime & Systems** | `crystal`, `crystal-debug`, `session-service`, `systems`, `core-entry`, `core-tool` | Session loop, rewind ring, speed controller, declarative system metadata (`Platform/systems/data/`), core catalog and download management |
| **Media, Saves & Library** | `library`, `library-tool`, `savemedia`, `patch`, `firmware`, `cheats`, `scrape`, `tool-job` | Game library scanning, game packages, `chdman` wrapper, memory cards/paks, IPS/BPS/UPS patching, firmware discovery, `.cht` cheats, ScreenScraper, background jobs |
| **C-ABI Aggregation** | `platform` | Builds `libemusen_platform` cdylib re-exporting all platform C entry points |
| **DianaSh Shell & Modules** | `dianash`, `dianash-module`, `dianash-program`, `dianash-text`, `emusen-cheat`, `emusen-cores-module`, `emusen-debug`, `emusen-dev-module`, `emusen-library-module`, `emusen-memcard-module`, `emusen-scrape-module`, `emusen-session`, `emusen-testing-module`, `emusen-trace` | Standalone `dianash` + `nano` binaries, sandboxed shell interpreter, C module ABI, and all live shell command modules |

See [Platform_C_Interfaces.md](Platform_C_Interfaces.md) and [DianaSh_and_Debugging.md](DianaSh_and_Debugging.md) for full details.

### `Frontend/` (21 Crates: 12 Workspace Members + 9 Fenced Pure-Rust Crates)

`Frontend/Cargo.toml` splits the frontend into two strictly enforced rings:

- **Qt 6 Workspace (12 crates, Rust 2021 edition, `cxx-qt` 0.10):** `kit-qt`, `qt-test`, `kit-gallery`, `qt-models`, `qt-endymion`, `qt-app`, `qt-play`, `esde-qt`, `esde-text`, `theme-qt`, `theme-host`, and `theme-check/audit`. Produces three binaries: `emusen-qt`, `emusen-qt-play`, and `kit-gallery`.
- **Fenced Pure-Rust Crates (9 excluded crates with independent lockfiles):** `kit-core`, `qml-rules`, `esde`, `theme-check`, `theme-check/manifest`, `crystal-client`, `platform-client`, `client-fixture`, and `c-calls`. Keeping them outside the Qt workspace guarantees that headless tools (such as `Platform/emusen-dev-module` depending on `qml-rules`) never link or require Qt 6.
- **Themes (`Frontend/themes/`):** Ships `eclipse` (Theme API `1.20`, dark variant, six screens: `home`, `games`, `consoles`, `splash`, `settings`, `game`) and the reference `emusen` theme (Theme API `1.3`, dark/light variants, three screens: `home`, `consoles`, `games`), alongside ES-DE theme support via `esde` and `esde-qt`.

See [Qt_Frontend_and_Theming.md](Qt_Frontend_and_Theming.md) for the frontend and theming architecture.

### `EmuSen/Cores/` (8 Native Cores, 3 HW Families, 19 Shared Crates, 5 Libretro Crates)

| Group | Location | Contents |
|---|---|---|
| **Shipping Native Cores (8)** | `EmuSen/Cores/{Nintendo,Sega,Sony}/` | `MoonRT` (NES), `MercuryRT` (GB/GBC), `VenusRT` (SNES), `MarsRT` (N64), `Endou` (SMS/Mark III/SG-1000/GG), `Nephrite` (Genesis/Sega CD; 32X stub), `Diamond` (PS1), `Sapphire` (PS2) |
| **Shared Hardware Families (3)** | `SilverMillennium-HW/`, `Beryl-HW/`, `BlackMoon-HW/` | `silvermillennium-sm83` (1 crate); `beryl-{m68k,z80,sh2,sn76489,sms-vdp,sms-colours,fm-tables,asm,font}` (9 crates, `beryl-sh2` is a stub); `blackmoon-{r3000,gte,spu}` (3 crates) |
| **Shared Core Infrastructure (19)** | `EmuSen/Cores/Shared/` | `emusen-native` (`emusen_core.h` v1.19), `emusen-core-{abi-check,broker,conform,host,schema}`, `emusen-disc`, `emusen-gpu`, `emusen-jit`, `emusen-assembler`, `emusen-mips`, `emusen-mips-planner`, `emusen-6502`, `emusen-goldens`, `emusen-singlestep`, and four `no_std` card/pak crates (`emusen-{psx-card,ps2-card,n64-pak,mcd-bram}`) |
| **Libretro Host (5)** | `EmuSen/Cores/Libretro/` | `emusen-libretro-{host,proxy,sys,games,testcore}` running the 10 opt-in cores from `Platform/systems/data/libretro-cores.tsv` inside Landlock-confined `emusen-core-runner` processes on Linux |

See [Core_ABI_and_Libretro_Host.md](Core_ABI_and_Libretro_Host.md) and [Docs/Man pages/README.md](../Man%20pages/README.md).

---

## Unix Modularity and Defined Boundaries

Five architectural invariants keep the repository modular and testable:

1. **Zero core code runs at discovery time:** Every native core emits a `.core.json` sidecar at build time (`cargo xtask stage --part cores`, validated by `cargo xtask check-sidecars`). Startup catalog enumeration reads sidecars without loading shared libraries.
2. **Additive C-ABI contracts:** Core ABI v1 (`emusen_core.h`, currently v1.19) uses `size`-prefixed structs, open capability bitmasks, and negative integer error bands; `emusen-core-abi-check` blocks breaking changes. `Platform/` applies the same discipline across its nine C headers with caller-owned copy-out buffers and thread-local `last_error` reporting.
3. **One owner per fact:** Console definitions live in `Platform/systems/data/*.json` and `.tsv`; directory paths and database filenames live in `Platform/galaxia/src/{tree,stores}.rs`; core build metadata lives in `xtask/libraries.tsv`. No downstream crate hardcodes duplicate tables.
4. **Declarative QML views:** QML files render properties exposed by `qt-models` and `kit-qt` and emit intents back to Rust. `Frontend/qml-rules` enforces allowed imports and forbids imperative state logic in QML.
5. **Clean-room and zero-bundled-firmware policy:** Every native core is implemented from public documentation and black-box testing. Copyrighted BIOS/DSP dumps are never bundled; systems either ship clean-room open replacements or load optional player images at runtime from `home/Firmware/`.

---

## Accuracy Model and Two Faces (`Cycle` vs `Byte`)

### Accuracy Tiers

Every emulated component states its accuracy tier honestly in its hardware manual and aims as high as real-time playability permits:

| Tier | Name | Basis of Claim |
|---|---|---|
| **1** | **Cycle accurate** | Verified against hardware documentation, console test programs, and black-box referees cycle by cycle |
| **2** | **Byte accurate** | Produces identical architectural results, pixels, and audio samples to Tier 1 with approximated internal timing |
| **3** | **Theory accurate** | Clean-room high-level emulation or open firmware replacement derived from public specifications |
| **4** | **General workaround** | No strict timing claim, but still general across all software—never conditional on a game title, serial, or checksum |

Per-game hacks are banned across every tier.

### The Two Faces Contract and Live Status

For complex consoles where strict cycle-ordered stepping is too slow for real-time play on handheld hardware, EmuSen defines a **Two Faces** architecture over one shared core body:

| Shared by Both Faces | Different Between Faces |
|---|---|
| Instruction decoding and architectural semantics | Scheduler granularity (every event in cycle order vs batched rounds) |
| Memory map and register semantics | Execution strategy (stepped interpreter vs compiled blocks / transactions) |
| Exact arithmetic (including proven fast paths) | Concurrency and synchronization bookkeeping (excluded from save state) |
| Machine state layout, save-state format, and debugger view | Internal event horizon |

- **Cycle face (reference):** Orders every hardware event by cycle and is never compromised for speed. Graded against hardware documentation, test programs, and black-box referees.
- **Byte face (fast default):** Shares the exact same semantics and save-state format while batching synchronization or compiling blocks. Save states are captured only at quiet synchronization points so states transfer cleanly between faces.

**Live implementation status:** Today, **Sapphire (PlayStation 2)** is the only core implementing the cycle-vs-batched split. In live Sapphire code (`src/board.rs`, `src/blocks.rs`), `board.round == 0` runs the cycle-ordered interpreter reference build, whereas `board.round > 0` runs batched rounds with the EE x86-64/Cranelift block recompiler, VU1 microprogram JIT, ahead-of-board VIF1/VU1/GIF transactions, and the Vulkan compute GS renderer. The other seven native cores (`MoonRT`, `MercuryRT`, `VenusRT`, `MarsRT`, `Endou`, `Nephrite`, `Diamond`) run a single execution path and do not implement a separate Byte face.

---

## Core and Component Naming Scheme

**EmuSen** stands for **Emu**lator **Sen**shi (*Bishoujo Senshi Sailor Moon*). Console cores and shared hardware libraries draw codenames from factions grouped by manufacturer, while hardware register names, disassemblers, and user-facing screen titles always use plain technical names.

The `RT` suffix on the four Nintendo cores (`MoonRT`, `MercuryRT`, `VenusRT`, `MarsRT`) identifies their Rust crate and folder names; newer Rust-native cores (`Endou`, `Nephrite`, `Diamond`, `Sapphire`) carry no suffix.

### Nintendo → Sailor Guardians (`EmuSen/Cores/Nintendo/`)

| Console / Library | Codename | Folder / Crate | Live Status |
|---|---|---|---|
| NES / Famicom | **Moon** (`MoonRT`) | `MoonRT - NES/` (`moonrt`) | Shipping |
| Game Boy / Game Boy Color | **Mercury** (`MercuryRT`) | `MercuryRT - GB/` (`mercuryrt`) | Shipping (DMG + CGB in one core) |
| SNES / Super Famicom | **Venus** (`VenusRT`) | `VenusRT - SNES/` (`venusrt`) | Shipping |
| Nintendo 64 | **Mars** (`MarsRT`) | `MarsRT - N64/` (`marsrt`) | Shipping |
| Game Boy Advance | **Jupiter** | `Jupiter - GBA/` | Reserved |
| Virtual Boy | **Saturn** | `Saturn - Virtual Boy/` | Reserved *(character codename; Sega Saturn is `Zoisite`)* |
| GameCube | **Uranus** | `Uranus - GameCube/` | Reserved |
| Wii | **Neptune** | `Neptune - Wii/` | Reserved |
| Wii U | **Pluto** | `Pluto - Wii U/` | Reserved |
| Nintendo DS | **Luna** | `Luna - DS/` | Reserved |
| Nintendo 3DS / New 3DS | **Artemis** | `Artemis - 3DS-New3DS/` | Reserved |
| Shared Nintendo HW | **SilverMillennium-HW** | `SilverMillennium-HW/` (`silvermillennium-sm83`) | Active (1 crate) |

### Sega → Dark Kingdom (`EmuSen/Cores/Sega/`)

| Console / Library | Codename | Folder / Crate | Live Status |
|---|---|---|---|
| Master System / Mark III / SG-1000 | **Endou** | `Endou - Master System/` (`endou`) | Shipping |
| Game Gear | **Jadeite** | Module inside `endou` (`src/gg.rs`, `src/jadeite.rs`); `Jadeite - Game Gear/` notes the module | Shipping inside `endou` |
| Genesis / Mega Drive, Sega CD, 32X | **Nephrite** | `Nephrite - Genesis/` (`nephrite`) | Shipping (Genesis + Sega CD; 32X is an unbuilt stub) |
| Sega Saturn | **Zoisite** | `Zoisite - Saturn/` | Reserved |
| Dreamcast | **Kunzite** | `Kunzite - Dreamcast/` | Reserved |
| Shared Sega HW | **Beryl-HW** | `Beryl-HW/` (`beryl-{m68k,z80,sh2,sn76489,sms-vdp,sms-colours,fm-tables,asm,font}`) | Active (9 crates; `beryl-sh2` is a stub) |

### Sony → Black Moon Clan (`EmuSen/Cores/Sony/`)

| Console / Library | Codename | Folder / Crate | Live Status |
|---|---|---|---|
| PlayStation | **Diamond** | `Diamond - PlayStation/` (`diamond`) | Shipping |
| PlayStation 2 | **Sapphire** | `Sapphire - PlayStation 2/` (`sapphire`) | Shipping |
| PlayStation 3 | **Rubeus** | `Rubeus - PlayStation 3/` | Reserved |
| PlayStation Portable | **Esmeraude** | `Esmeraude - PSP/` | Reserved |
| PlayStation Vita | **Wiseman** | `Wiseman - PS Vita/` | Reserved |
| Shared Sony HW | **BlackMoon-HW** | `BlackMoon-HW/` (`blackmoon-{r3000,gte,spu}`) | Active (3 crates) |

### Reserved Factions (Atari, NEC, Microsoft)

| Manufacturer | Faction | Reserved Codenames & Folders |
|---|---|---|
| **Atari** | Death Busters (Witches 5) | `Eudial - Atari 2600`, `Mimete - Atari 5200`, `Tellu - Atari 7800`, `Viluy - Atari Lynx`, `Cyprine & Ptilol - Atari Jaguar` |
| **NEC** | Dead Moon Circus (Amazon Trio) | `Tigers Eye - PC Engine` (*Tiger's Eye*), `Hawks Eye - PC-FX` (*Hawk's Eye*), `Fish Eye - SuperGrafx` (apostrophes omitted in folder paths) |
| **Microsoft** | Dead Moon Circus (Amazoness Quartet) | `CereCere` (Xbox), `JunJun` (Xbox 360), `PallaPalla` (Xbox One), `VesVes` (Xbox Series X\|S) |

### Shared Placement Rule and Platform Components

- **Chip placement:** Chips specific to one manufacturer's lineage live in `SilverMillennium-HW`, `Beryl-HW`, or `BlackMoon-HW`. Cross-manufacturer chips and generic infrastructure (`emusen-mips`, `emusen-6502`, `emusen-disc`, `emusen-gpu`, `emusen-jit`) live in `EmuSen/Cores/Shared/`. Board-specific code stays inside its core crate.
- **Platform infrastructure names:** Developer-facing platform components retain their established codenames—**Galaxia** (storage and config), **Endymion** (audio and input), **Serenity** (Vulkan video and shaders), **Crystal** (runtime sessions), and **DianaSh** (the command shell). User-facing screens and menus always use plain descriptive names (`Home`, `Consoles`, `Games`, `Game`, `Settings`, `Cores`, `Packages`, `Scraper`, `Library`, `Display`, `Controllers`, `Cheats`).
