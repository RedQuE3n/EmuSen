# EmuSen

EmuSen (**Emu**lator **Sen**shi) is a clean-room multi-console emulator written in Rust with a Qt 6 frontend. Every component is organized around small, composable modules separated by narrow C-ABI and read-only view boundaries:

- **Native Cores (`EmuSen/Cores/`):** Eight shipping Rust console cores built on a stable Core ABI v1 (`emusen_core.h`, currently v1.19), backed by three shared hardware-chip libraries (`SilverMillennium-HW`, `Beryl-HW`, `BlackMoon-HW`) and 19 shared core infrastructure crates (`EmuSen/Cores/Shared/`).
- **Platform Services (`Platform/`):** A 36-crate Rust workspace compiled into `libemusen_platform`, covering configuration and storage (`galaxia`), SDL3 audio and RetroPad input (`endymion`), Vulkan shaders and CRT rendering (`serenity`), runtime session/rewind/speed orchestration (`crystal`), library/packages/CHD tools (`library`, `library-tool`), scraping (`scrape`), save media (`savemedia`), cheats (`cheats`), firmware lookup (`firmware`), and the standalone **`dianash`** shell plus **`nano`**.
- **Qt 6 Frontend (`Frontend/`):** A 21-crate Rust + Qt 6 (`cxx-qt`) stack shipping **`emusen-qt`** (the unified six-screen library and game frontend with the `Eclipse` dark theme, reference `EmuSen` theme, and ES-DE theme engine), **`emusen-qt-play`** (direct game and picture runner), and **`kit-gallery`** (component gallery).
- **Build Host (`xtask/`):** A Rust `cargo xtask` driver that builds, stages, sidecar-verifies, and packages cores, platform libraries, and Linux AppImage bundles.

---

## Native Rust Cores

All eight native cores live under `EmuSen/Cores/`, export Core ABI v1 (`emusen_core_*`), and declare their capabilities through build-generated `.core.json` sidecars (`xtask/libraries.tsv`):

| Core | Systems | Crate Directory | Built-in Open Firmware | Hardware Reference |
|---|---|---|---|---|
| **MoonRT** | NES / Famicom (NTSC, PAL, Dendy) | `EmuSen/Cores/Nintendo/MoonRT - NES/` | None required | [Moon_Hardware.md](Docs/Man%20pages/Hardware/Nintendo/Moon%20-%20NES/Moon_Hardware.md), [Moon_PAL.md](Docs/Man%20pages/Hardware/Nintendo/Moon%20-%20NES/Moon_PAL.md) |
| **MercuryRT** | Game Boy / Game Boy Color | `EmuSen/Cores/Nintendo/MercuryRT - GB/` | Post-boot register state | [Mercury_Hardware.md](Docs/Man%20pages/Hardware/Nintendo/Mercury%20-%20GB-GBC/Mercury_Hardware.md), [Mercury_CgbAndModels.md](Docs/Man%20pages/Hardware/Nintendo/Mercury%20-%20GB-GBC/Mercury_CgbAndModels.md) |
| **VenusRT** | Super Nintendo / Super Famicom (SA-1, SuperFX, OBC1, DSP-1/1B/2/4, ST010) | `EmuSen/Cores/Nintendo/VenusRT - SNES/` | Open HLE for DSP-1/1B/2/4 & ST010 | [Venus_Hardware.md](Docs/Man%20pages/Hardware/Nintendo/Venus%20-%20SNES/Venus_Hardware.md), [Venus_Coprocessors.md](Docs/Man%20pages/Hardware/Nintendo/Venus%20-%20SNES/Venus_Coprocessors.md), [Venus_DspHle.md](Docs/Man%20pages/Hardware/Nintendo/Venus%20-%20SNES/Venus_DspHle.md) |
| **MarsRT** | Nintendo 64 (Vulkan compute RDP, VR4300 block JIT) | `EmuSen/Cores/Nintendo/MarsRT - N64/` | Open PIF/CIC boot handoff | [Mars_CpuAndMemory.md](Docs/Man%20pages/Hardware/Nintendo/Mars%20-%20N64/Mars_CpuAndMemory.md), [Mars_Rsp.md](Docs/Man%20pages/Hardware/Nintendo/Mars%20-%20N64/Mars_Rsp.md), [Mars_RdpAndVi.md](Docs/Man%20pages/Hardware/Nintendo/Mars%20-%20N64/Mars_RdpAndVi.md) |
| **Endou** | Master System, Mark III, SG-1000, Game Gear (incl. YM2413 OPLL) | `EmuSen/Cores/Sega/Endou - Master System/` | Open SMS & Game Gear boot ROMs | [Endou_Hardware.md](Docs/Man%20pages/Hardware/Sega/Endou%20-%20Master%20System/Endou_Hardware.md), [Endou_GameGear.md](Docs/Man%20pages/Hardware/Sega/Endou%20-%20Master%20System/Endou_GameGear.md) |
| **Nephrite** | Genesis / Mega Drive, Sega CD / Mega-CD (32X stub) | `EmuSen/Cores/Sega/Nephrite - Genesis/` | Open Sega CD BIOS & BRAM | [Nephrite_Genesis.md](Docs/Man%20pages/Hardware/Sega/Nephrite%20-%20Genesis/Nephrite_Genesis.md), [Nephrite_SegaCD.md](Docs/Man%20pages/Hardware/Sega/Nephrite%20-%20Genesis/Nephrite_SegaCD.md), [Nephrite_32X.md](Docs/Man%20pages/Hardware/Sega/Nephrite%20-%20Genesis/Nephrite_32X.md) |
| **Diamond** | PlayStation (Vulkan 1×–4× compute renderer, precision layer) | `EmuSen/Cores/Sony/Diamond - PlayStation/` | Open BIOS Kernel & shell | [Diamond_System.md](Docs/Man%20pages/Hardware/Sony/Diamond%20-%20PlayStation/Diamond_System.md), [Diamond_VideoAndMedia.md](Docs/Man%20pages/Hardware/Sony/Diamond%20-%20PlayStation/Diamond_VideoAndMedia.md), [Diamond_OpenKernel.md](Docs/Man%20pages/Hardware/Sony/Diamond%20-%20PlayStation/Diamond_OpenKernel.md) |
| **Sapphire** | PlayStation 2 (EE/VU1 JIT, Vulkan compute GS) | `EmuSen/Cores/Sony/Sapphire - PlayStation 2/` | ELF fast-boot (player BIOS optional) | [Sapphire_EeAndVu.md](Docs/Man%20pages/Hardware/Sony/Sapphire%20-%20PlayStation%202/Sapphire_EeAndVu.md), [Sapphire_GsAndVideo.md](Docs/Man%20pages/Hardware/Sony/Sapphire%20-%20PlayStation%202/Sapphire_GsAndVideo.md), [Sapphire_IopAndIo.md](Docs/Man%20pages/Hardware/Sony/Sapphire%20-%20PlayStation%202/Sapphire_IopAndIo.md), [Sapphire_System.md](Docs/Man%20pages/Hardware/Sony/Sapphire%20-%20PlayStation%202/Sapphire_System.md) |

---

## Opt-In Hosted Libretro Cores

On Linux, EmuSen can also host a fixed allowlist of external Libretro cores (`Platform/systems/data/libretro-cores.tsv`) inside a Landlock-confined child process (`emusen-core-runner`) with Vulkan ring handoff. Hosted cores are **never bundled**: they are downloaded only at the player's explicit request and verified against recorded SHA-256 digests.

| Core Stem | Systems | Renderer | Default Engine | Upstream Licence |
|---|---|---|---|---|
| `pcee2` | PlayStation 2 (`ps2`) | Vulkan | Yes (`ps2`) | GPLv3 |
| `flycast` | Dreamcast (`dreamcast`) | Vulkan | — | GPLv2 |
| `snes9x` | SNES (`snes`) | Software | — | Non-commercial |
| `nestopia` | NES (`nes`) | Software | — | GPLv2 |
| `genesis_plus_gx` | Genesis, Sega CD, SMS, SG-1000 (`md`, `mcd`, `sms`, `sg1000`) | Software | — | Non-commercial |
| `picodrive` | 32X (`32x`) | Software | — | MAME |
| `mupen64plus_next` | Nintendo 64 (`n64`) | Vulkan | — | GPLv2 |
| `sameboy` | Game Boy / Game Boy Color (`gb`, `gbc`) | Software | — | MIT |
| `mednafen_psx_hw` | PlayStation (`psx`) | Vulkan | — | GPLv2 |
| `mednafen_saturn` | Sega Saturn (`saturn`) | Software | — | GPLv2 |

Native Rust cores remain the default engine on every system they ship for except PlayStation 2, where `pcee2` is the default engine while Sapphire matures. Players can switch engines per system or per game in the **Cores** screen or with `cores(1)` in DianaSh.

---

## Clean-Room and Firmware Policy

- **Clean-room implementation:** Every native core is implemented from public hardware documentation, test programs, and black-box observation. Commercial ROMs, copyrighted BIOS dumps, and proprietary SDKs are never committed to this repository or used as source material.
- **Zero bundled firmware:** EmuSen ships no copyrighted console BIOS, boot ROM, or coprocessor firmware images. Systems that require boot code or DSP programs run out of the box using EmuSen's clean-room open replacements (Diamond's Open BIOS Kernel, Nephrite's open Sega CD BIOS, Endou's open SMS/Game Gear boot ROMs, VenusRT's open DSP HLE, MarsRT's PIF/CIC handoff, and Sapphire's ELF fast-boot).
- **Optional player firmware:** Players who own authentic console firmware dumps can place them in `home/Firmware/` (or configure `FirmwareDirectory` in `appsettings.json`). `Platform/firmware` detects images by canonical name, known alternate names, split parts, or MD5 content hash, and lets players choose between open replacements and their own dumps.

---

## Key Features

- **Unified Six-Screen Interface:** Full controller and keyboard navigation across `Splash`, `Home`, `Consoles`, `Games`, `Game`, and `Settings`, themed via Theme API v1.20 (`Eclipse` dark theme, reference `EmuSen` theme, and ES-DE theme support).
- **Save States, Rewind & Moments:** Versioned binary save states (`emusen-native`), background XOR-delta rewind ring (`Platform/crystal`), thumbnail Moments reel, and configurable turbo/slow-motion speed control.
- **Modelled CRT & Vulkan Shaders:** Zero-readback Vulkan device-picture handoff (`Platform/serenity`), SPIR-V cached slang shader chains, and a physically modelled CRT filter in three tiers (`Accurate`, `Balanced`, `Performance`) with per-console display profiles.
- **Unified Save Media & Game Packages:** Battery SRAM, removable memory cards/paks (PS1, PS2, N64 Controller Pak, Sega CD BRAM), IPS/BPS/UPS patching with isolated saves, CHD conversion, and ScreenScraper metadata support.
- **Input & Multiplayer:** Unified RetroPad mapping (`Platform/endymion`), SDL3 hotplug, per-console profiles, dynamic player seat claiming, Guide chords (automatically yielding to Steam Game Mode under gamescope), and rumble.
- **Integrated Shell & Debugging (`dianash`):** Sandboxed Unix-style shell with 10+ command modules for memory inspection, disassembly, breakpoints, watchpoints, cheat management, ROM verification, scraping, core catalog control, and test orchestration.

---

## Repository Layout

| Directory | Role |
|---|---|
| `EmuSen/Cores/` | Eight shipping native Rust cores, three shared hardware families (`SilverMillennium-HW`, `Beryl-HW`, `BlackMoon-HW`), 19 shared core crates, and five Libretro host crates |
| `Platform/` | 36-crate Rust workspace building `libemusen_platform`, standalone `dianash`, and `nano`, plus nine C headers |
| `Frontend/` | 12-crate Qt 6 workspace (`emusen-qt`, `emusen-qt-play`, `kit-gallery`), nine fenced pure-Rust crates (`kit-core`, `qml-rules`, `esde`, `theme-check`, clients), and built-in themes (`eclipse`, `emusen`) |
| `xtask/` | Rust build host (`cargo xtask`), `libraries.tsv`, and `appimage-tools.tsv` |
| `Testing/` | Test tiers (`tiers.json`), blast-radius inputs (`inputs.json`), SQLite schema, and ES-DE theme regression corpus |
| `Docs/` | The EmuSen Manual (`Docs/EmuSen Manual/`), hardware reference manuals (`Docs/Man pages/`), and DianaSh man pages (`Docs/man/`) |

---

## Quick Start: Build and Run

### Prerequisites

- **Rust toolchain** (supporting Rust 2024 edition for `Platform/` and 2021 edition for `Frontend/` and cores)
- **Qt 6** development libraries (`Qt6Quick`, `Qt6Qml`, `Qt6Gui`, `Qt6Multimedia`, `Qt6Wayland*`, `qmake6`)
- **SDL3** and **Vulkan** loader/headers (plus Vulkan validation layers for `EMUSEN_VALIDATE=1`)
- **Optional:** `podman` and `7z` to build the pinned `chdman` binary via `Platform/library-tool/build-chdman.sh`

### 1. Stage Native Cores and Platform Libraries

From the repository root, run `cargo xtask` to compile and lay out the shared libraries, `.core.json` sidecars, `dianash`, and `nano`:

```bash
# Build and stage all 8 native cores + Libretro host components (Linux)
cargo xtask stage --part cores --out out/stage-cores

# Build and stage libemusen_platform, dianash, and nano
cargo xtask stage --part platform --out out/stage-platform

# Verify every staged .core.json sidecar matches its shared library
cargo xtask check-sidecars out/stage-cores/out
```

### 2. Build and Run the Qt 6 Frontend

```bash
# Build the main frontend, direct game runner, and component gallery
cargo build --manifest-path Frontend/Cargo.toml --release \
    --bin emusen-qt --bin emusen-qt-play --bin kit-gallery

# Launch the main EmuSen frontend
Frontend/target/release/emusen-qt \
    --platform-library out/stage-platform/out/libemusen_platform.so

# Launch a single ROM directly in emusen-qt-play
EMUSEN_PLATFORM_LIBRARY=out/stage-platform/out/libemusen_platform.so \
EMUSEN_CORES=out/stage-cores/out \
Frontend/target/release/emusen-qt-play --game /path/to/game.sfc

# Launch the standalone component gallery
Frontend/target/release/kit-gallery
```

### 3. Run the Standalone DianaSh Shell

```bash
# Build and launch standalone dianash (includes text, dev, and testing modules)
cargo run --manifest-path Platform/Cargo.toml --release -p dianash --bin dianash
```

Inside `dianash`, run `help`, `man <command>` (for example `man hier`, `man cores`, `man bp`), or `nano <path>` to explore the sandboxed tree.

For Cargo profiles (`release`, `dist`, `test-checked`, `test-fast`), Linux desktop and AppImage bundling, and environment variables, see [Building_and_Running.md](Docs/EmuSen%20Manual/Building_and_Running.md).

---

## Documentation

| Documentation Set | Contents | Entry Point |
|---|---|---|
| **EmuSen Manual** | User guide, build/packaging, settings reference, architecture, Qt 6 theming, Core ABI v1, Platform C headers, audio/video/CRT, input, save media, DianaSh, and testing | [Docs/EmuSen Manual/README.md](Docs/EmuSen%20Manual/README.md) |
| **Hardware Reference Manuals** | Factual register maps, clocks, chip contracts, open firmware tables, and accuracy boundaries for every native Rust core and shared hardware library | [Docs/Man pages/README.md](Docs/Man%20pages/README.md) |
| **DianaSh Man Pages** | Plain-text Unix manual pages for every shell builtin, text utility, debugger command, library tool, and developer checker (`man <command>` inside `dianash`) | [Docs/man/](Docs/man/) |
| **Contributing Guide** | Branching model, Unix modularity rules, Two Faces accuracy contract, clean-room rules, and testing workflow | [CONTRIBUTING.md](CONTRIBUTING.md) |

---

## Licence

EmuSen is free software licensed under the **GNU General Public License, version 3 or later** (`GPL-3.0-or-later`). See [LICENSE](LICENSE) for the full text.

Third-party crates, libraries, and optional tools distributed with or fetched by EmuSen are listed with their copyrights and licences in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md), with complete upstream licence texts stored in [`licenses/`](licenses/).
