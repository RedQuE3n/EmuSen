# Platform C Interfaces

The `Platform/` Cargo workspace implements all shared runtime services above the emulator cores — storage, configuration, declarative system tables, input, audio, Vulkan shader rendering, save media, library indexing, scraping, session orchestration, and the DianaSh shell — and links them into one shared library (`emusen_platform`, alongside the standalone `dianash` and `nano` executables).

Frontends and external shell modules communicate with `emusen_platform` strictly through **9 C headers** instead of linking Rust crates directly. This keeps `Platform/` and `Frontend/` cleanly decoupled, preserves narrow Unix contracts, and makes every cross-layer call auditable.

For core-level ABI contracts see [Core ABI and Libretro Host](Core_ABI_and_Libretro_Host.md). For Galaxia storage layouts and JSONC settings see [Settings and Configuration](Settings_and_Configuration.md).

---

## 1. The 36-Crate `Platform/` Workspace

`Platform/Cargo.toml` uses Rust 2024 edition (`resolver = "3"`) and organizes **36 crates** into clear subsystems aggregated by `Platform/platform` into the `emusen_platform` `cdylib`:

| Layer | Crates | Responsibility |
|---|---|---|
| **Core Runtime & Emulation** | `crystal`, `crystal-debug`, `session-service`, `systems`, `firmware`, `cheats`, `savemedia`, `recorder` | Core discovery/routing, session lifecycle, frame loop, rewind ring, speed control, debugging service table, declarative system specs, firmware detection, cheat codecs, save media, and A/V recording. |
| **Storage & Library** | `galaxia`, `sqlite`, `library`, `library-tool`, `patch`, `scrape`, `net`, `suggest`, `tool-job` | Directory layout (`tree.rs`), atomic JSONC config files, SQLite stores, game library scanning, CHD/package/ROM jobs, IPS/BPS/UPS patching, ScreenScraper HTTP client, typo suggestions, and async job runner. |
| **Audio, Video & Input** | `endymion`, `serenity` | SDL3 audio streams, resampler, dynamic rate control, RetroPad input/bindings/seats/hotplug (`endymion`); Vulkan shader compiler, SPIR-V cache, CRT/LCD filters, frame pacing, and Vulkan lend/ring (`serenity`). |
| **Core Management** | `core-entry`, `core-tool` | Core catalogue entries, SHA-256 verification, and atomic staging/install/rollback of hosted Libretro cores. |
| **DianaSh Shell & Modules** | `dianash`, `dianash-module`, `dianash-program`, `dianash-text`, `emusen-debug`, `emusen-trace`, `emusen-cheat`, `emusen-session`, `emusen-cores-module`, `emusen-library-module`, `emusen-memcard-module`, `emusen-scrape-module`, `emusen-dev-module`, `emusen-testing-module` | Standalone shell (`dianash` + `nano`), loadable module ABI, text tools, and all 11 live command modules. |
| **Library Aggregation** | `platform` | Compiles and exports the unified `emusen_platform` shared library. |

### 1.1 Cargo Build Profiles

`Platform/Cargo.toml` defines four Cargo profiles shared by the platform library and native cores:

| Profile | Configuration | Purpose |
|---|---|---|
| `release` | `panic = "abort"`, `opt-level = 3`, `lto = "fat"`, `codegen-units = 1` | Production performance with hard panic-abort guarantee across C boundaries. |
| `dist` | Inherits `release`, `debug = false`, `strip = "debuginfo"` | Release distribution bundles and AppImages. |
| `test-checked` | Inherits `release`, `overflow-checks = true` | CI and nightly test runs so unintended arithmetic wrapping fails immediately. |
| `test-fast` | Inherits `release`, `lto = false`, `codegen-units = 16`, `incremental = true` | Fast incremental rebuilds for local and landing verification. |

---

## 2. Universal C-ABI Conventions

Every Platform C header follows the same pull-only design rules established by `emusen_core.h`:

1. **Pull, never push:** Callers invoke the library and read back results. The library never registers host callbacks; long-running jobs (`emusen_tool_job`, library scans, scraping queues) and runtime notifications (`emusen_crystal_subscription`) are drained via poll/wait calls.
2. **Opaque handles and numeric session IDs:** Stateful objects (`emusen_crystal_runtime`, `emusen_crystal_subscription`, `emusen_library`, `emusen_tool_job`, `emusen_serenity_chain`, `dianash_shell`) are opaque pointers released by matching `*_free` or `*_dispose` exports. Emulation sessions use 32-bit unsigned IDs numbered from `1`, with `0` referring to the most recently opened active session.
3. **Explicit UTF-8 byte slices:** Text crosses as `(const uint8_t *ptr, size_t len)` in UTF-8 bytes and is never assumed NUL-terminated. A null pointer represents an absent string distinct from an empty non-null string.
4. **Length-query copy-out:** Functions producing variable-length strings, JSON, or byte arrays accept `(uint8_t *out, size_t len)` and return the full byte length as `int64_t`. Passing `out == NULL` queries the required size; bytes are written only when `len` holds the entire result. Callers own every output buffer, with two explicit exceptions:
   - Borrowed service function tables (such as `emusen_session_v1`) that remain valid for the lifetime of their parent session.
   - Lent Serenity readback pixel pointers, released back to the pool when the caller returns the readback handle.
5. **Negative status codes + thread-local `last_error`:** Any return `< 0` is a typed status code. "Absent" statuses (`EMUSEN_PLATFORM_ABSENT`, `EMUSEN_SVC_ABSENT`) are normal answers rather than errors. Detailed failure text is fetched immediately on the same thread via the header's `*_last_error` export.
6. **Upfront version checks:** Clients must call the header's version symbol before resolving or calling any other export.
   - `emusen_platform.h` is an **internal exact-match ABI** (`EMUSEN_PLATFORM_ABI_VERSION = 50`): frontend and platform must come from the same commit.
   - All service and frontend headers use packed `(MAJOR << 16) | MINOR` stability: within a major version, functions, size-prefixed struct fields, open enum values, and JSON keys are strictly additive.
7. **Single SQLite instance:** `emusen_library.h` accepts a caller-lent SQLite handle so every library database connection inside one process shares one SQLite library copy and lock table.
8. **Zero unwinding:** Panics write the configured crash log and abort rather than unwinding across C stack frames.

---

## 3. Complete Catalog of All 9 Live C Headers

| # | Header Path | Version | Stability | Export / Service Role |
|---:|---|---|---|---|
| 1 | `Platform/include/emusen_platform.h` | `50` | Exact build match | Core platform cdylib exports (~268 functions) covering Galaxia, Endymion, Serenity, disc sets, file services, HTTP client, tool jobs, and shell hosting. |
| 2 | `Platform/crystal/include/emusen_crystal.h` | `2.1` | Stable major `2` | Frontend emulation API owned by `Platform/crystal`: runtime, system/engine catalog, content routing, sessions, frame execution, input/rumble, speed/rewind, save states, settings, firmware, save-media cards, cheat codecs, recorder, events, and `emusen_session_v1`. |
| 3 | `Platform/crystal/include/emusen_session.h` | `1.3` | Stable major `1` | `emusen.session` function table handed out by Crystal for debuggers and shell modules: run/pause/step, memory spaces, registers, disassembly, breakpoints/watchpoints, call/write/read logs, coverage, profiler, hardware counters, labels, cheats, and telemetry. |
| 4 | `Platform/library/include/emusen_library.h` | `1.4` | Stable major `1` | Game library database (`games.db`) API (11 exports) using a lent SQLite instance: JSON queries (`get`, `games`), library mutations, background scanning, reclaim jobs, progress polling, and cancellation. |
| 5 | `Platform/library-tool/include/emusen_library_tool.h` | `1.7` | Stable major `1` | `emusen.library` service table (`emusen_library_tool_v1`) passed to DianaSh modules for background jobs (`chd`, `pkg`, `core-*`, `rom-*`). |
| 6 | `Platform/include/emusen_games_service.h` | `1.0` | Stable major `1` | `emusen.games` service table giving DianaSh modules read access to `games.db` and ROM folder scans (including `esde` system tags) without linking `library` or `galaxia`. |
| 7 | `Platform/include/emusen_scrape_service.h` | `2.1` | Stable major `2` | `emusen.scrape` service table for ScreenScraper metadata/media jobs, quota planning, credentials, and cover selection. |
| 8 | `Platform/dianash/include/dianash.h` | `1.1` | Stable major `1` | Host API for creating, configuring, submitting lines to, draining output/requests from, and destroying a DianaSh shell instance. |
| 9 | `Platform/dianash/include/dianash_module.h` | `1.1` | Stable major `1` | Loadable DianaSh command-module ABI (`dianash_module_*`) defining module manifests, argument/stdin reading, answer formatting, and host service lookups. |

### 3.1 Functional Sections of `emusen_platform.h` (ABI v50)

`Platform/include/emusen_platform.h` groups its exports into clear functional blocks:

- **Galaxia:** Directory tree (`tree`), save-library naming, atomic files, game packages, mapped files, config files & models (`appsettings.json`, `audio.json`, `graphics.json`, `consoles.json`), frontend settings rows, host battery level, error log, and "did you mean" suggestions.
- **Library & Patch Tools:** Async tool job runner (`emusen_tool_job_*`), scraping lifecycle (`emusen_scraping_open`), soft IPS/BPS/UPS patching, and disc-set parsing (`emusen-disc`).
- **File & Network Services:** System pack metadata & debugger tables (`emusen-systems`), firmware lookup (`emusen-firmware`), card manager (`emusen-savemedia`), `.cht` cheat lists (`emusen-cheats`), and HTTP client (`emusen-net`).
- **Endymion:** Logic half (`LinearResampler`, `DynamicRateControl`, `PortRouter`, `PlayerSlots`), four JSONC binding files, and device half (SDL3 audio streams, playback devices, UI sounds, gamepads, simulated pads, rumble).
- **Serenity:** Shader presets/sources, GLSL-to-SPIR-V compiler, reflection, SQLite SPIR-V cache, Vulkan device/chain/lend/ring, `DisplayClock` frame pacing, and built-in CRT/LCD filter preset generation.

---

## 4. Declarative System Specifications (`Platform/systems`)

`Platform/systems` centralizes console definitions in declarative data files under `Platform/systems/data/` so frontends, library scanners, scrapers, and cores never hardcode duplicate console tables:

- **15 console JSON files:** `nes.json`, `snes.json`, `gb.json`, `gbc.json`, `n64.json`, `sg1000.json`, `sms.json`, `md.json`, `mcd.json`, `32x.json`, `saturn.json`, `dreamcast.json`, `psx.json`, `ps2.json`, plus `table.json`. Each file declares the system's canonical `id`, display name, manufacturer, release year, media kind, recognized file extensions, `esde` system slug, vibration support flag, save-media geometry, and display classification.
- **`catalog-rows.tsv`:** Canonical display ordering and catalog grouping shared across UI screens and library tools.
- **`libretro-cores.tsv`:** The 10 allowlisted hosted Libretro cores, their system mappings, upstream URLs, licences, host renderers (`software` / `vulkan`), and default-engine overrides.
- **`libretro-extensions.tsv`:** Optional per-core extension additions or exclusions applied on top of upstream Libretro `.info` files.

---

## 5. `Platform/crystal` Runtime and Session Lifecycle

`Platform/crystal` (`emusen_crystal.h` v2.1) orchestrates core discovery, content routing, emulation sessions, and debugger service tables.

### 5.1 Runtime Initialization and Content Routing

1. **Runtime creation:** Frontends construct an `emusen_crystal_runtime` with Galaxia paths, an optional Serenity Vulkan device/ring configuration, and an optional lent `emusen_library` handle.
2. **Zero-load engine discovery:** Crystal scans `.core.json` sidecars via `emusen-core-host` without loading any core binary.
3. **Routing (`route.rs`):** When launching a ROM, disc image, game package, or empty console, Crystal resolves the target system, respects player engine overrides and defaults, checks SHA-256 approvals (`EMUSEN_CRYSTAL_NOT_APPROVED`), and verifies required firmware (`EMUSEN_CRYSTAL_FIRMWARE`) before returning a routing decision or opening the session.

### 5.2 Session Lifecycle, Threading, and API Surface

- **Opening (`open.rs`):** Opening a game or console returns a 32-bit session ID (`>= 1`). Open flags select `OPEN_NO_BATTERY`, `OPEN_RING` (cross-process Vulkan ring), and `OPEN_SINGLE_ROWS`.
- **Single frame thread:** Exactly one thread per session calls `emusen_crystal_frame`. That call executes the complete per-frame sequence — applying queued inputs/axes/settings, stepping the core, capturing rewind snapshots, updating rumble/media events, tracking battery dirtiness, and returning `FrameInfo` plus interleaved PCM samples. Concurrent `frame` calls from a second thread fail with `EMUSEN_CRYSTAL_BUSY`.
- **Thread-safe reads and boundary-applied writes:** Read calls (session info, telemetry, save states, debugger inspection) run on any thread without waiting for the frame loop. Mutating calls from other threads synchronize cleanly at the next frame boundary; mutating calls on the frame thread apply immediately.
- **Per-API groups in `emusen_crystal.h`:**
  - *Runtime & Routing:* systems, engines, routing, approvals.
  - *Session Control:* open, close, reset, disc lid/swap, speed modes (`SPEED_NORMAL`, `SPEED_SLOW`, `SPEED_TURBO`, `SPEED_DRAW_NONE`), rewind reel & Moments, save states, A/V recording (`record_start`, `record_stop`).
  - *Cards & Cheats:* `emusen_crystal_cards_op` (`LIST`, `OPEN`, `NEW`, `COPY`, `MOVE`, `DELETE`, `DUPLICATE`, `RENAME`, `IMPORT`) and `cheat_codec_list` / `cheat_decode` / `cheat_encode`.
  - *Events & Debugger:* `Subscription` drain/wait/unsubscribe and `emusen_crystal_session_service` handing out `emusen_session_v1` with optional policy flags `EMUSEN_CRYSTAL_SERVICE_READ_ONLY` (`1`) and `EMUSEN_CRYSTAL_SERVICE_NO_GAME` (`2`).

### 5.3 Session Service Statuses (`emusen_session.h` v1.3)

`emusen_crystal.h` includes `emusen_session.h` so both APIs share one unified status band:

| Status Code | Constant | Meaning |
|---:|---|---|
| `-1` | `EMUSEN_SVC_NULL` | Null table, context, or required pointer. |
| `-5` | `EMUSEN_SVC_BAD_STRING` | Non-UTF-8 input text. |
| `-1100` | `EMUSEN_SVC_NO_SESSION` | No open session or unknown session ID. |
| `-1101` | `EMUSEN_SVC_NO_SUCH_SPACE` | Unknown memory space name or ID. |
| `-1102` | `EMUSEN_SVC_READ_ONLY` | Mutation attempted in a read-only context or space. |
| `-1103` | `EMUSEN_SVC_NOT_SUPPORTED` | Unsupported operation or missing core capability. |
| `-1104` | `EMUSEN_SVC_BAD_ARGUMENT` | Invalid number, name, or JSON argument. |
| `-1105` | `EMUSEN_SVC_NO_SUCH_ID` | Unknown breakpoint, watch, register, or label ID. |
| `-1106` | `EMUSEN_SVC_PARSE` | Expression or cheat code parse failure. |
| `-1107` | `EMUSEN_SVC_ABSENT` | Requested item does not exist (normal answer). |
| `-1108` | `EMUSEN_SVC_FAULT` | Internal runtime failure (details in `last_error`). |
| `-1109 ..` | `EMUSEN_CRYSTAL_*` | Crystal-specific errors (`NO_ENGINE`, `FIRMWARE`, `NOT_APPROVED`, `BUSY`, `BAD_STRUCT`, `IO`, `STATE_REFUSED`). |

---

## 6. Safe Frontend Bindings (`c-calls`, `platform-client`, `crystal-client`)

No C pointer or `unsafe` FFI call ever appears inside `qt-models`, `theme-qt`, or QML. Three standalone, non-Qt crates under `Frontend/` encapsulate the entire C boundary:

- **`Frontend/c-calls`:** Implements the universal call protocol once for every header:
  1. `open(path, version_symbol)` opens `libemusen_platform.so` via `libloading` and reads its packed ABI version before resolving any other symbol.
  2. `check(n, names, last_error)` maps negative returns into typed `Error::Status { status, name, words }` errors while reading thread-local `last_error` text immediately.
  3. Copy-out helpers run the two-pass length-query idiom (treating header-specific `ABSENT` codes as `Ok(None)`) and deserialize JSON answers into strongly typed Rust structs (`Error::Shape` on mismatch).
- **`Frontend/platform-client`:** Exposes ergonomic RAII Rust wrappers for every section of `emusen_platform.h`, `emusen_library.h`, and service tables (`platform`, `galaxia`, `audio`, `sounds`, `pads`, `bindings`, `seats`, `router`, `serenity`, `pace`, `library`, `packages`, `scrape`, `cheats`, `firmware`, `systems`, `tools`, `net`).
- **`Frontend/crystal-client`:** Exposes typed Rust wrappers over `emusen_crystal.h` and `emusen_session.h` (`Api`, `Runtime`, `Subscription`, `SessionService`, `FrameInfo`, `RawInfo`, `consts`, and typed JSON `shapes`).

Because these three crates stay fenced outside the Qt workspace, they build and test directly against `emusen_platform` and `Frontend/client-fixture` with zero Qt dependency.

---

## Related Documentation

- [Overview and Architecture](Overview_and_Architecture.md) — 4-layer system architecture and crate map.
- [Core ABI and Libretro Host](Core_ABI_and_Libretro_Host.md) — Core ABI v1 (`emusen_core.h`) beneath `Platform/crystal`.
- [Qt Frontend and Theming](Qt_Frontend_and_Theming.md) — How `qt-models` consumes `platform-client` and `crystal-client`.
- [DianaSh and Debugging](DianaSh_and_Debugging.md) — Writing DianaSh modules against `dianash_module.h` and `emusen_session.h`.
- [Settings and Configuration](Settings_and_Configuration.md) — Galaxia directory layout, JSONC configs, and SQLite databases.
