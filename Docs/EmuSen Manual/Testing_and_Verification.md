# Testing and Verification

EmuSen verifies its Rust emulator cores, platform library, Qt 6 frontend, QML themes, ES-DE compatibility layer, and Wayland display pipeline through a multi-layer automated verification stack. Every layer emphasizes deterministic reproducibility, fast blast-radius test selection, and clean-room hardware accuracy without committing proprietary ROMs or firmware.

For building binaries and Cargo profiles see [Building and Running](Building_and_Running.md). For `qml-rules` and `theme-audit` details see [Qt Frontend and Theming](Qt_Frontend_and_Theming.md). For Core ABI contracts see [Core ABI and Libretro Host](Core_ABI_and_Libretro_Host.md).

---

## 1. Testing Policy and Execution Rules

1. **Blast-radius test selection:** Rather than running every crate and test suite after a localized edit, EmuSen maps changed files through its dependency and input graph (`Testing/inputs.json` and `emusen.testing`'s `impact` command) to execute only the test units reachable from the change.
2. **Parallel non-GPU vs serialized GPU execution:**
   - Pure-Rust crate tests, static analyzers (`qml-rules`, `theme-check --static`, `esde-check`, `emusen-core-abi-check`), and offscreen software-rendered `emusen-qt-test` suites run **in parallel** across independent crates and CPU threads.
   - GPU-bound Vulkan tests, shader-chain tests, `theme-audit --gpu`, and Wayland compositor gates (`Testing/qt-gates/`) run **serialized** (one GPU/compositor owner at a time) so render nodes, Vulkan swapchains, and virtual compositors never contend or skew timing numbers.
3. **Clean-room and synthetic ROM policy:** Commercial ROMs and proprietary console BIOS dumps are **never** committed to the repository and are never required for `fast`, `land`, or `nightly` test tiers. All automated repository suites rely strictly on clean-room synthetic test ROMs, open-source hardware test suites, JSON single-step CPU vectors, and the synthetic Libretro core (`emusen-libretro-testcore`). Commercial game regression tests live strictly in the opt-in local `games` tier (`needs: ["roms"]`).

---

## 2. The `emusen.testing` Studio (`Platform/emusen-testing-module`)

`Platform/emusen-testing-module` exposes EmuSen's testing studio as composable DianaSh commands whose line-oriented output pipes naturally into `grep`, `sort`, and `wc`:

| Command | Mode | Key Flags & Purpose |
|---|---|---|
| `units` | Read-only | `units [--kind cargo] [--tier T] [--crates] [--key UNIT] [--profile P] [--rev REV] [--repo DIR]` — prints the catalog of test units, one per line. |
| `impact` | Read-only | `impact [--base REV] [--head REV] [--tier T] [--depth N] [--also TERMS] [--map-age DAYS] [--format blast] [--explain FILE] [--repo DIR]` — computes test units affected by a diff with the matching rule and file. |
| `coverage` | Read/Write | `coverage [FILE...] \| run --bin DIR [--jobs N] [--rev REV] \| load [--merge] [FILE]` — queries and maintains the file-to-test execution coverage map. |
| `shard` | Read-only | `shard [--shards N] [--head REV] [FILE] \| --machine` — partitions test units across `N` workers balanced by measured historical runtimes. |
| `trial` | Read/Write | `trial [--tier T] [--from FILE] [--no-reuse] [--dry-run] [--on RUNNER] \| verdict SHA \| ls \| show N \| quarantine ...` — runs a selection, records results, checks commit verdicts, and manages flaky-test quarantine. |
| `results` | Read-only | `results [--last N] [--slowest N] [--flaky] [--misses] [--selected N] [--quarantine] [unit U \| test T] [--tier T] [--since D] \| export` — queries the studio database for performance, flakiness, and history. |
| `ingest` | Write | `ingest FILE... --runner R --tree SHA [--trial N] [--tier T] \| --backfill [DIR]` — imports external TRX/JUnit logs or historical landing logs into the store. |
| `queue` | Write | `queue \| ask CLASS [--gb N] [--as WHO]` — inspects active runner slots and requests capacity for heavy test jobs. |

### 2.1 Typical Studio Workflow

```sh
# Inspect which test units a branch change affects and why
impact --base main --tier land
impact --explain Frontend/theme-qt/src/help.rs

# Run a dry-run or full land trial, then check the verdict and slowest units
trial --tier land --dry-run
trial --tier land
results --last 10
results --slowest 15
```

### 2.2 Committed Configuration (`Testing/`) vs Local Store

Committed studio configuration lives under `Testing/`:

- **`Testing/tiers.json`:** Defines the **five verification tiers**:

| Tier | Selection | Exclusions | Cargo Profile | Build Reuse | Purpose |
|---|---|---|---|---|---|
| `fast` | `impact` (`depth: 1`) | `nightly`, `slow`, `hardware` | `test-fast` | Yes (`7` days) | Rapid inner-loop developer feedback (10-minute budget). |
| `land` | `impact` (full graph) | `nightly`, `slow`, `hardware` | `test-fast` | Yes (`7` days) | Pre-merge verification gate (20-minute budget). |
| `nightly` | `all` | `hardware` | `test-checked` | No | Full overflow-checked nightly sweep (`02:00`). |
| `games` | `tag:slow` | — | — | No | Local opt-in commercial ROM regression runs (`needs: ["roms"]`). |
| `hardware` | `tag:hardware` | — | — | No | Attended multi-device testing (`handheld`, `mac`, `laptop`, `desktop`). |

- **`Testing/inputs.json`:** Declares non-build-graph couplings: folder rules narrowing reached test suites, recording file extensions (`.spec`, `.golden`, `.frame`, `.framesum`) with their owning tests, and documentation ignore patterns.
- **`Testing/schema/001-studio.sql`:** Forward-only SQLite migration (`PRAGMA user_version = 1`) defining `runner`, `unit`, `build`, `trial`, and result/quarantine tables.

Local runtime state stays strictly outside the Git tree:

- SQLite store: `~/.local/share/EmuSen/testing/studio.db`
- Trial artifacts & logs: `~/.cache/emusen/testing/`
- Local runner roster: `~/.config/EmuSen/testing/runners.json`

---

## 3. Qt 6 Offscreen, Theme, ES-DE, and Compositor Verification

The Qt 6 frontend is verified across four layers from fast static checks up to full Wayland/Vulkan compositor runs.

### 3.1 Static Linting and Theme Audits

- **`qml-rules` (`Frontend/qml-rules`):** Enforces the 7 QML zero-logic rules (`function`, `callLater`, `literal`, `words`, `onCompleted`, `rows`, `import`) across application views and theme packages (`qml-rules [--fixture] <path>...`).
- **`theme-check` & `theme-audit` (`Frontend/theme-check/`):** Validates `theme.json` manifests, token contrast, asset paths, and renders every declared screen offscreen through `ThemeHost` across 5 viewport sizes, variants, reduced motion, and all 15 synthetic library states (`default`, `nogame`, `empty`, `one-console`, `one-game`, `many`, `nocover`, `norecent`, `long`, `pressed`, `disabled`, `loud`, `splash-1s`, `splash-3s`, `splash-5s`).

### 3.2 Headless Qt Quick Harness and Spec Recordings (`Frontend/qt-test`)

`Frontend/qt-test` runs QML scenes offscreen on Qt's software backend without a display server:

| Module (`qt-test/src/`) | Responsibility |
|---|---|
| `harness.rs`, `tree.rs` | Boots QML scenes (`Harness`, `Scene`), injects `QTest` keyboard/pointer input, and captures the live Qt Quick item tree (`Tree`) for `kit-core`'s fit, reach, contrast, and naming audits. |
| `virtual_pad.rs` | Injects synthetic `PadEvent` presses directly into `PadRouter`'s queue. |
| `spec.rs`, `mockup.rs` | Writes deterministic plain-text `spec 1` screen recordings (`sheet`, `menu`, `overlay`, `scene` headers, two-decimal geometry, focus landings, palette, and item trees) and diffs them against reference mockups. |
| `lint.rs`, `qml_surface.rs`, `worded.rs` | Checks live QML surfaces against `qml-rules` and verifies every visible string originates from Rust. |
| `compositor.rs` | Launches a scoped Wayland compositor (`WAYLAND_DISPLAY` + `QT_QPA_PLATFORM=wayland`) with a mandatory watchdog (`watch(seconds)`) that kills the compositor immediately on panic or timeout so tests never leak orphan compositors or fault on swapchain teardown. |

### 3.3 ES-DE Theme Corpus (`Testing/esde-corpus/`)

`esde-check` (provided by `Platform/emusen-dev-module`) evaluates real ES-DE themes across systems and variants and emits per-theme TSV rows tracking:

- `loads`, `systems`, `variants`, `runs`, `errors`, `warnings`, `unknown`, `unsupported`, `whole`, `kinds`, `undrawn`
- Visual regression metrics against reference renders at `1280×720` and `1920×1080`: `mae_system_1280`, `mae_gamelist_1280`, `mae_system_1920`, `mae_gamelist_1920`, `rects_off`, `mae_viewer_1280`, `mae_viewer_1920`, `viewer_off`.

Committed TSV snapshots and exception tables under `Testing/esde-corpus/` (and `rulings/`) catch any regression in ES-DE theme parsing or visual accuracy before landing.

### 3.4 Wayland / KWin Compositor Gates (`Testing/qt-gates/`)

`Testing/qt-gates/` contains self-contained Python and shell scripts that measure real end-to-end startup latency, first-frame time, focus responsiveness, and resident memory (`VmRSS` / `RssAnon`) inside an isolated KWin Wayland virtual compositor (`1920×1200` on a private D-Bus session) without touching the player's home directory:

| Script | Purpose |
|---|---|
| `run.sh` | Starts an isolated KWin Wayland compositor under a watchdog, launches the target binary fullscreen, collects polled grabs (`grab.py`) and optional RSS logs (`rss.py`), and cleans up all child processes on exit. |
| `pad.py` | Feeds scripted controller steps (`wait`, `settle`, `tap`, `hold`) over a named FIFO (`EMUSEN_GATE_PAD`) into an in-process SDL virtual gamepad compiled only when `emusen-qt` enables its `gate-pad` feature (never creating host `uinput` devices). |
| `first.py`, `press.py` | Computes exec-to-window latency, exec-to-first-game-picture latency, and button-press-to-visual-focus-ring latency from timestamped `ScreenShot2` grabs. |
| `sandbox.sh`, `gates.sh`, `figures.py` | Sets up an isolated `bwrap` sandbox (with read-only ROM mount and private saves) and executes repeatable `warm`, `start`, `idle`, and `pause` gate rounds (including handheld profiles under `Testing/handheld/`). |
| `shell.py` | Drives a staged platform library's DianaSh shell from stdin to run `cores fetch`, `get`, `approve`, and `default` inside test sandboxes. |

---

## 4. Core Verification and Hardware Accuracy Suites

Native and hosted cores pass five specialized verification layers under `EmuSen/Cores/Shared/`:

1. **ABI baseline guard (`emusen-core-abi-check`):** Validates `emusen_core.h` and Rust FFI declarations against `EmuSen/Cores/Shared/emusen-native/abi/v1/baseline.txt` so no v1 export, enum constant, status code, or minimum struct size ever breaks backward compatibility.
2. **Black-box ABI conformance (`emusen-core-conform`):** Dynamically loads every staged native core (and the Libretro proxy paired with `emusen-libretro-testcore`) and tests every advertised capability bit, null/bad-struct status code, length-query buffer call, deterministic save/load state round-trip, media lid/insert error path, Vulkan lend/ring contract, and `.core.json` sidecar parity.
3. **Deterministic golden traces (`emusen-goldens`):** Drives cores with reproducible input streams (`DotNetRandom`, `Driven`, `Trace`) and compares frame, audio, and state MD5 digests against checked-in golden traces.
4. **Single-step CPU suites (`emusen-singlestep`):** Loads per-opcode JSON test files (initial CPU registers + sparse RAM pairs → expected final registers + RAM), grades every instruction test, and reports exact `first_mismatch` diffs across 6502, 65816, SM83, Z80, 68000, and MIPS CPU implementations.
5. **Black-box hardware referees:** Individual cores run clean-room hardware test ROMs (CPU timing, PPU/VDP, DMA, audio, and memory-card suites) and cross-check Byte-face fast paths directly against Cycle-face reference implementations as described in [Overview and Architecture](Overview_and_Architecture.md).

---

## Related Documentation

- [Overview and Architecture](Overview_and_Architecture.md) — Two Faces accuracy contract and repository layout.
- [Building and Running](Building_and_Running.md) — `cargo xtask`, `test-fast`, and `test-checked` profiles.
- [Qt Frontend and Theming](Qt_Frontend_and_Theming.md) — `qml-rules`, `theme-check`, `theme-audit`, and `kit-gallery --grab`.
- [Core ABI and Libretro Host](Core_ABI_and_Libretro_Host.md) — Core ABI v1 stability rules and `emusen-core-conform`.
- [DianaSh and Debugging](DianaSh_and_Debugging.md) — Running `emusen.testing` and `emusen.dev` commands in DianaSh.
- [Testing/README.md](../../Testing/README.md) — Quick reference to committed files under `Testing/`.
