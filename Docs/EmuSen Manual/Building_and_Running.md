# Building and Running

This guide covers toolchain prerequisites, compiling EmuSen's three Rust codebases (`EmuSen/Cores/`, `Platform/`, `Frontend/`), staging artifacts with `cargo xtask`, selecting Cargo profiles, running each executable, and building Linux desktop and AppImage packages.

---

## Prerequisites

### Required Toolchain and System Libraries (Linux)

| Dependency | Purpose | Notes |
|---|---|---|
| **Rust (`rustc`, `cargo`)** | Compiles `Platform/` (Rust 2024), `Frontend/`, `xtask/`, and `EmuSen/Cores/` (Rust 2021) | Current stable toolchain with `cargo` |
| **Qt 6 development packages** | Required by `Frontend/` (`cxx-qt` 0.10) | `Qt6Core`, `Qt6Gui`, `Qt6Qml`, `Qt6Quick`, `Qt6QuickControls2`, `Qt6Multimedia`, `Qt6WaylandClient`, `Qt6WaylandCompositor`, and `qmake6` |
| **SDL3** | Audio output and gamepad input (`Platform/endymion`, `Frontend/qt-endymion`) | Development headers and shared library |
| **Vulkan loader & headers** | Vulkan slang shader pipeline (`Platform/serenity`), compute renderers (`emusen-gpu`), and Qt RHI | Plus Vulkan validation layers when running with `EMUSEN_VALIDATE=1` |
| **C / C++ compiler & `pkg-config`** | Native C-ABI headers, `cxx-qt-build`, and system glue | GCC or Clang |

### Optional Tools

- **`podman` and `7z`:** Required only when building the pinned MAME `chdman` binary for `Platform/library-tool`:
  ```bash
  bash Platform/library-tool/build-chdman.sh
  ```
  The script downloads the release archive named in `Platform/library-tool/chdman.pin`, verifies its SHA-256, builds `TOOLS=1 EMULATOR=0` inside a rootless Fedora container with `-static-libstdc++ -static-libgcc` (so it runs cleanly on SteamOS as well as standard desktop Linux), and installs the result at `~/.cache/emusen/toolchains/chdman/linux-x64/chdman`.
- **AppImage toolchain:** Required only for `cargo xtask appimage`. See [Linux Desktop and AppImage Packaging](#linux-desktop-and-appimage-packaging).

---

## Building with `cargo xtask`

`xtask/` is EmuSen's repository-root build host. It reads `xtask/libraries.tsv` and `xtask/appimage-tools.tsv` so local builds and CI produce identical staged outputs.

### `cargo xtask` Subcommands

```text
cargo xtask stage --part cores|platform --out DIR [--profile release|dist]
                  [--target-root DIR] [--no-reuse] [--no-pgo]
                  [--cache DIR] [--chdman FILE]

cargo xtask check-sidecars FOLDER

cargo xtask prebuilt --rid RID --out DIR [--from DIR]

PATCHELF=FILE cargo xtask appimage --out DIR [--tools DIR]
```

| Subcommand | What It Does |
|---|---|
| `stage --part cores` | Builds all eight native cores plus Linux Libretro host components in manifest order, generates `.core.json` sidecars, computes SHA-256 `cores.manifest` files, and writes `DIR/out` (development, includes test libraries) and `DIR/publish` (shipping files only). |
| `stage --part platform` | Builds `emusen-platform` (`libemusen_platform.so`) and `dianash` (`dianash`, `nano`, `nano.dianash.json`) from `Platform/`, stages `--chdman` (or the cached pinned binary if present), and lays out `DIR/out` and `DIR/publish`. |
| `check-sidecars FOLDER` | Verifies that every `.core.json` sidecar in `FOLDER` matches its sibling shared library; exits non-zero on any missing library or mismatched metadata. |
| `prebuilt --rid RID` | Stages prebuilt CI libraries from `FROM/RID/` into `DIR` for packaging a target runtime identifier (`RID`). |
| `appimage` | Verifies every tool pinned in `xtask/appimage-tools.tsv` against its SHA-256, compiles `kit-gallery` in release mode, and bundles `DIR/kit-gallery-x86_64.AppImage`. |

### Staged Output Layout

`cargo xtask stage` writes two sibling directories under `--out DIR`:

- **`DIR/out/`:** Development layout containing every built library, its `.core.json` sidecar, helper executables (`emusen-core-runner`, `dianash/nano`), development test libraries (`emusen_libretro_testcore`), and `cores.manifest`.
- **`DIR/publish/`:** Release layout filtering out entries marked `development` in `xtask/libraries.tsv` and writing a matching shipping `cores.manifest`.

### Key `stage` Options

- `--profile release|dist`: Selects the Cargo profile (`release` default; `dist` strips debuginfo for publishing).
- `--target-root DIR`: Overrides the Cargo target directory (defaults to `<repo>/target`).
- `--cache DIR` / `--no-reuse`: Reuses previously built native libraries cached under `~/.cache/emusen/cores` on local non-Windows machines; automatically disabled when `CI=true` or when `--no-reuse` is passed.
- `--no-pgo`: Disables profile-guided optimization passes during core staging.
- `--chdman FILE`: Explicit path to a `chdman` binary for `stage --part platform`.

### `xtask/libraries.tsv`

`xtask/libraries.tsv` is the authoritative tab-separated manifest of native libraries and helper programs built by `xtask`:

| Name | Role | ABI v1 Sidecar | Ships | Platform |
|---|---|---|---|---|
| `diamond`, `endou`, `marsrt`, `mercuryrt`, `moonrt`, `nephrite`, `sapphire`, `venusrt` | `core` | Yes (`1`) | `ships` | `any` |
| `emusen_libretro_proxy`, `emusen_libretro_host` | `part` | — | `ships` | `linux` |
| `emusen-core-runner` | `program` | — | `ships` | `linux` |
| `emusen_libretro_testcore` | `test` | — | `development` | `linux` |
| `emusen_platform` | `platform` | — | `ships` | `any` |

---

## Building Individual Workspaces Directly

When iterating on one subsystem, invoke Cargo directly against its manifest:

```bash
# 1. Platform workspace (libemusen_platform, dianash, nano)
cargo build --manifest-path Platform/Cargo.toml --profile test-fast -p emusen-platform -p dianash
cargo test --manifest-path Platform/Cargo.toml --profile test-fast -p emusen-galaxia

# 2. Qt 6 Frontend workspace (emusen-qt, emusen-qt-play, kit-gallery)
cargo build --manifest-path Frontend/Cargo.toml --release \
    --bin emusen-qt --bin emusen-qt-play --bin kit-gallery
cargo test --manifest-path Frontend/Cargo.toml -p emusen-theme-audit

# 3. Fenced pure-Rust Frontend crates (outside Frontend/Cargo.toml workspace)
cargo test --manifest-path Frontend/kit-core/Cargo.toml
cargo test --manifest-path Frontend/qml-rules/Cargo.toml
cargo test --manifest-path Frontend/esde/Cargo.toml

# 4. Individual native core (example: MoonRT)
cargo build --manifest-path "EmuSen/Cores/Nintendo/MoonRT - NES/Cargo.toml" --release
cargo test --manifest-path "EmuSen/Cores/Nintendo/MoonRT - NES/Cargo.toml" --release
```

### Cargo Profiles

`Platform/Cargo.toml` defines four profiles tailored for C-ABI safety, CI verification, and local iteration:

| Profile | Flags | Intended Use |
|---|---|---|
| `release` | `panic = "abort"`, `opt-level = 3`, `lto = "fat"`, `codegen-units = 1` | Shipping C-ABI libraries; panics abort after logging so nothing unwinds across FFI |
| `dist` | Inherits `release`, `debug = false`, `strip = "debuginfo"` | Stripped release artifacts for distribution |
| `test-checked` | Inherits `release`, `overflow-checks = true` | CI and pre-land verification to catch unintended integer wrap |
| `test-fast` | Inherits `release`, `lto = false`, `codegen-units = 16`, `incremental = true` | Fast incremental local builds and tests |

`Frontend/Cargo.toml` configures `[profile.release]` with `opt-level = 2` for balanced compile time and UI responsiveness.

---

## Running EmuSen Binaries

### `emusen-qt` (Main Frontend)

```bash
Frontend/target/release/emusen-qt \
    --platform-library out/stage-platform/out/libemusen_platform.so \
    [--theme Frontend/themes/eclipse]
```

| Flag / Environment Variable | Effect |
|---|---|
| `--platform-library PATH` / `EMUSEN_PLATFORM_LIBRARY` | Path to `libemusen_platform.so` (defaults to `libemusen_platform.so` beside the binary) |
| `--theme DIR` / `EMUSEN_THEME` | Theme directory override (otherwise uses `EmuSenTheme` from `appsettings.json` or falls back to the reference theme) |
| `EMUSEN_GPU_DEVICE` | Select a specific Vulkan GPU device name/index for Serenity and compute cores |
| `QSG_RHI_BACKEND` | Qt scene graph backend (defaults to `vulkan` if unset) |
| `QSG_RENDER_LOOP` | Forced to `threaded` by `emusen-qt` and `emusen-qt-play` |

### `emusen-qt-play` (Direct Game & Picture Runner)

```bash
EMUSEN_PLATFORM_LIBRARY=out/stage-platform/out/libemusen_platform.so \
EMUSEN_CORES=out/stage-cores/out \
Frontend/target/release/emusen-qt-play --game /path/to/rom.sfc [--seconds 10]
```

- Without `--game`, renders a moving Vulkan test picture for `--seconds` (default `10`) and prints frame pacing counters.
- Set `EMUSEN_ENGINE=<name>` to select a specific core engine, and `EMUSEN_VALIDATE=1` to enable Vulkan validation layer reporting.

### `kit-gallery` (Component Gallery)

```bash
Frontend/target/release/kit-gallery
```

Launches the standalone `EmuSen.Kit` visual preview and interaction gallery for inspecting kit controls, tokens, and focus navigation outside a full game library.

### `dianash` and `nano` (Standalone Shell)

```bash
cargo run --manifest-path Platform/Cargo.toml --release -p dianash --bin dianash
```

Standalone `dianash` links the core shell builtins together with `dianash.text`, `emusen.dev`, and `emusen.testing`, and spawns the sibling `nano` binary when editing files inside the sandboxed Galaxia tree.

---

## Linux Desktop and AppImage Packaging

### Pin-Verified AppImage Build (`cargo xtask appimage`)

`xtask/appimage-tools.tsv` pins five packaging tools by exact release URL and SHA-256 checksum:

| Tool | Pinned Release | Source |
|---|---|---|
| `linuxdeploy-x86_64.AppImage` | `1-alpha-20251107-1` | `linuxdeploy/linuxdeploy` |
| `linuxdeploy-plugin-qt-x86_64.AppImage` | `1-alpha-20250213-1` | `linuxdeploy/linuxdeploy-plugin-qt` |
| `appimagetool-x86_64.AppImage` | `1.9.1` | `AppImage/appimagetool` |
| `runtime-x86_64` | `20251108` | `AppImage/type2-runtime` |
| `patchelf` | `0.18.0-9.fc43` | Distribution package named by `$PATCHELF` |

Place the four downloaded AppImage tools in `~/.cache/emusen/toolchains/appimage/` (or pass `--tools DIR`), point `PATCHELF` to the verified system `patchelf`, and run:

```bash
PATCHELF=/usr/bin/patchelf cargo xtask appimage --out out/appimage
```

`xtask/src/appimage.rs` checks every SHA-256 before running `linuxdeploy` with:

- `QMAKE=/usr/bin/qmake6`
- `EXTRA_PLATFORM_PLUGINS="libqwayland.so;libqoffscreen.so"` and `EXTRA_QT_MODULES=waylandcompositor`
- `NO_STRIP=1` (preserving `.relr.dyn` relocation sections on modern glibc/Fedora libraries)
- Offline `--runtime-file` passed to `appimagetool --no-appstream` so packaging makes no network requests

Currently `cargo xtask appimage` bundles **`kit-gallery-x86_64.AppImage`** as the reference Qt 6 AppImage bundle; the same pinned toolchain and Qt plugin rules apply when packaging `emusen-qt` alongside staged `out/stage-platform/publish/` and `out/stage-cores/publish/` artifacts.

### Wayland and SteamOS Handheld Notes

- **Wayland & gamescope:** Both `emusen-qt` and AppImage bundles include the Qt Wayland platform plugin (`libqwayland.so`) and render via Vulkan (`QSG_RHI_BACKEND=vulkan`, threaded render loop).
- **Steam Game Mode detection:** `emusen-qt` inspects `XDG_CURRENT_DESKTOP` for `gamescope` (or `SteamDeck=1` when no desktop is set) so `GuideChords: "Auto"` yields Guide shortcuts cleanly to Steam's overlay.
- **Portable root & `chdman`:** Published trees carry a `.dianashroot` marker file beside their `home/` folder for portable discovery (see [Settings_and_Configuration.md](Settings_and_Configuration.md)), and `Platform/library-tool/build-chdman.sh` statically links `libstdc++` and `libgcc` so `chdman` runs without host C++ ABI mismatches on SteamOS.
