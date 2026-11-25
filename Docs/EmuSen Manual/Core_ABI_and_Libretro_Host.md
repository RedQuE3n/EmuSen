# Core ABI and Libretro Host

Every emulator core in EmuSen — whether a first-party Rust core or an opt-in hosted Libretro core — speaks a single stable, pull-only C binary interface: **Core ABI v1**, defined in `EmuSen/Cores/Shared/emusen-native/include/emusen_core.h` (currently **v1.19**). Native cores compile directly against this ABI via `emusen-native`, while hosted Libretro cores run inside Landlock-confined child processes (`emusen-core-runner`) bridged by `emusen-libretro-host` and `emusen-libretro-proxy`.

For the Two Faces accuracy contract and core naming conventions see [Overview and Architecture](Overview_and_Architecture.md). For binary save states and save media see [Save Media and Game Packages](Save_Media_and_Game_Packages.md). For Vulkan shader chains and device-picture rendering see [Audio, Video, CRT, and Shaders](Audio_Video_CRT_and_Shaders.md).

---

## 1. Stable Core ABI v1 (`emusen_core.h`)

A core is a shared library exporting C-calling-convention functions prefixed with `emusen_core_*`, using types `emusen_*` and constants `EMUSEN_*`. The packed ABI version returned by `emusen_core_abi_version()` is `(EMUSEN_CORE_ABI_MAJOR << 16) | EMUSEN_CORE_ABI_MINOR`, currently `(1 << 16) | 19` (**v1.19**).

### 1.1 Universal ABI Contracts

Every export follows seven permanent guarantees for the lifetime of major version 1:

1. **Pull-only control flow:** The host calls the library; the library never calls back into the host. Logs, audio rate changes, geometry updates, battery dirty flags, rumble levels, and disc seek hints are returned as values or drained via `emusen_core_events()`.
2. **Signed status returns:** Negative `int32_t` / `int64_t` values are status codes; zero or positive values indicate success (often a byte length or item count). Passing a null `emusen_machine *` always returns `EMUSEN_NULL` (`-1`). Only `emusen_core_abi_version()` and `emusen_core_capabilities()` return unsigned integers and cannot fail.
3. **Copy-out length-query idiom:** Calls filling caller buffers take `(out, len)` and return the full byte or element count. Passing `out == NULL` queries the required size without writing. No pointer into core memory is ever returned; input pointers remain valid only for the duration of the call, except for borrowed game images and disc media lent under `EMUSEN_CAP_BORROW`.
4. **Size-prefixed structs:** Every struct crossing the boundary begins with `uint32_t size`. Callees inspect only the prefix both sides understand, never modify `size`, and reject structs shorter than their v1.0 minimum with `EMUSEN_BAD_STRUCT` (`-263`). Minor versions only append fields, with `0` representing "absent".
5. **Open enumerations:** Unknown enum values are rejected cleanly with `EMUSEN_NOT_SUPPORTED` (`-256`) or ignored where documented; hosts skip unknown event kinds, capability bits, and JSON descriptor fields.
6. **Thread discipline:** One thread at a time may call instance exports on a given `emusen_machine *` (multiple machines may coexist inside one library). Library-level exports (`abi_version`, `capabilities`, `info`, `settings_schema`, `status_text`) are safe concurrently from any thread.
7. **Errors are values:** No export unwinds, `longjmp`s, raises signals, or calls `exit`. Rust cores compile with `panic = "abort"` after writing a crash log.

### 1.2 Fixed Status Bands

| Range | Constants / Ownership | Meaning |
|---|---|---|
| `-1 .. -7` | `EMUSEN_NULL` (`-1`), `EMUSEN_TRUNCATED` (`-2`), `EMUSEN_FOREIGN` (`-3`), `EMUSEN_STATE_VERSION` (`-4`), `EMUSEN_BAD_STRING` (`-5`), `EMUSEN_BUFFER_TOO_SMALL` (`-7`) | Save-state and buffer status codes (`-6`, `-8` reserved). |
| `-9 .. -255` | `EMUSEN_CORE_BAND_FIRST .. LAST` | Core-defined error codes described by `emusen_core_status_text()`. |
| `-256 .. -268` | `EMUSEN_NOT_SUPPORTED` (`-256`), `NO_SUCH_SPACE` (`-257`), `READ_ONLY` (`-258`), `UNKNOWN_SETTING` (`-259`), `BAD_SETTING` (`-260`), `NO_SUCH_PORT` (`-261`), `BAD_FILE` (`-262`), `BAD_STRUCT` (`-263`), `BAD_IMAGE` (`-264`), `LID_CLOSED` (`-265`, v1.1), `WRONG_MEDIA` (`-266`, v1.1), `NO_SUCH_DRIVE` (`-267`, v1.1), `NOT_EXPECTED` (`-268`, v1.17) | Interface-level errors across settings, ports, media drives, and trust checks (`-269 .. -319` unassigned). |

### 1.3 Capability Bits (`EMUSEN_CAP_*`)

A core advertises optional export groups and behaviors via the 64-bit mask returned by `emusen_core_capabilities()`:

| Bit | Constant | Since | Governs |
|---:|---|---|---|
| `0` | `EMUSEN_CAP_RESET` | 1.0 | `emusen_core_reset` (soft/hard reset). |
| `1` | `EMUSEN_CAP_PRESENT` | 1.0 | `emusen_core_present` (explicit host-driven frame presentation). |
| `2` | `EMUSEN_CAP_SNAPSHOT` | 1.0 | Fast state kind `EMUSEN_STATE_SNAPSHOT` (`1`) for rewind and run-ahead. |
| `3` | `EMUSEN_CAP_AXES` | 1.0 | `emusen_core_set_axis` for analog sticks (`-1..1`) and triggers (`0..1`). |
| `4` | `EMUSEN_CAP_AUDIO_PEEK` | 1.0 | Non-destructive audio buffer peek (`emusen_core_audio_peek`). |
| `5` | `EMUSEN_CAP_MUTES` | 1.0 | Per-channel audio muting (`emusen_core_set_mutes`). |
| `6` | `EMUSEN_CAP_SETTINGS` | 1.0 | Creation-time and live settings (`emusen_core_set_settings`). |
| `7` | `EMUSEN_CAP_PHASES` | 1.0 | Per-frame internal phase timings (`emusen_core_phases`). |
| `8` | `EMUSEN_CAP_FRAME_SERIAL` | 1.0 | `frame_info.serial` advances only when displayed image pixels change. |
| `9` | `EMUSEN_CAP_ROW_REPEAT` | 1.0 | `frame_info.row_repeat` may exceed `1` (interlaced/doubled scanlines). |
| `10` | `EMUSEN_CAP_BATTERY_DIRTY` | 1.0 | `emusen_core_battery` flags track SRAM / save-media mutations. |
| `11` | `EMUSEN_CAP_ROM_PATCHES` | 1.0 | Dynamic ROM cheat patches (`emusen_core_set_rom_patches`). |
| `12` | `EMUSEN_CAP_DEBUG` | 1.0 | Core debug exports (breakpoints, watchpoints, write/call/profile/coverage logs, PC, debug frame runner). |
| `13` | `EMUSEN_CAP_DEBUG_STACK` | 1.0 | Call-stack tracking (`emusen_core_debug_set_stack`). |
| `14` | `EMUSEN_CAP_CHEAT_POKES` | 1.0 | Per-frame RAM cheat pokes (`emusen_core_set_cheat_pokes`). |
| `15` | `EMUSEN_CAP_SETTING_NOTES` | 1.0 | Human-readable active setting notes (`emusen_core_setting_notes`). |
| `16` | `EMUSEN_CAP_DEBUG_REGISTERS` | 1.0 | Structured CPU and hardware-unit register reads (`emusen_core_debug_registers`; unit IDs >= `256` since v1.13). |
| `17` | `EMUSEN_CAP_DEBUG_DISASSEMBLE` | 1.0 | Core-native JSON disassembly (`emusen_core_debug_disassemble`). |
| `18` | `EMUSEN_CAP_MEDIA` | 1.1 | Multi-disc drive control (`emusen_core_media_lid`, `emusen_core_media_insert`). |
| `19` | `EMUSEN_CAP_DEBUG_READS` | 1.2 | Read watchpoints/breakpoints (`EMUSEN_RANGE_READ_WATCH`, `EMUSEN_RANGE_READ_BREAK`, `emusen_core_debug_reads`). |
| `20` | `EMUSEN_CAP_BORROW` | 1.3 | Zero-copy borrowing of host-owned ROM and disc buffers until free or disc swap. |
| `21` | `EMUSEN_CAP_DEVICE_PICTURE` | 1.9 | Direct Vulkan rendering onto a host-lent queue (`emusen_device_lend`, `emusen_core_frame_device`). |
| `22` | `EMUSEN_CAP_RAW_PICTURE` | 1.10 | Secondary raw framebuffer inspection (`emusen_core_raw_select`, `raw_info`, `raw_copy`). |
| `23` | `EMUSEN_CAP_TELEMETRY` | 1.13 | Non-mutating hardware telemetry counters (`emusen_core_telemetry`). |
| `24` | `EMUSEN_CAP_PRESENCE` | 1.13 | Hardware controller plug/unplug presence reporting (`emusen_core_set_present`). |
| `25` | `EMUSEN_CAP_DEBUG_TRANSLATE` | 1.13 | Side-effect-free virtual-to-bus address translation (`emusen_core_debug_translate`). |
| `26` | `EMUSEN_CAP_IMAGE_PATH` | 1.16 | Core opens large disc/ROM images via `image_path` on `emusen_create_params` and `emusen_media`. |
| `27` | `EMUSEN_CAP_DEVICE_RING` | 1.16 | Cross-process Vulkan image ring handoff (`emusen_core_ring_lend`, `emusen_core_frame_ring`) for sandboxed cores. |
| `28` | `EMUSEN_CAP_EXPECT` | 1.17 | Pre-startup SHA-256 verification (`emusen_core_expect`) ensuring a proxy serves exact approved library bytes. |

### 1.4 Core Structs, Pixel Formats, and Events

All structs crossing the boundary carry a leading `uint32_t size`:

| Struct | v1.0 / Current Size | Key Fields |
|---|---|---|
| `emusen_file` | `24` / `32` bytes (v1.16) | `which` (`0` = battery save, or firmware/media/named slot), `data`, `len`, and optional relative `name` (v1.16). |
| `emusen_create_params` | `88` / `120` bytes (v1.16) | `host_abi_version`, `image`/`image_len`, `settings`/`settings_len` (`key=value` UTF-8 lines), `files`/`file_count`/`file_size`, `pixel_formats` bitmask, optional `error` buffer, `flags` (`EMUSEN_CREATE_LENT`, v1.3), `device` (`emusen_device_lend *`, v1.9), `system` (v1.11/v1.15), `image_path` (v1.16). |
| `emusen_device_lend` | `64` bytes (v1.9) | Lent Vulkan 1.2+ `instance`, `physical_device`, `device`, `get_instance_proc_addr`, core `queue_family`/`queue_index`, `host_queue_family`, and host `done` timeline `VkSemaphore`. |
| `emusen_device_frame` | `56` bytes (v1.9) | Core-rendered `format`, `VkBuffer`, `offset`, `width`, `height`, `stride`, and core `ready` timeline `VkSemaphore` + `value`. |
| `emusen_ring_image` / `emusen_ring_lend` / `emusen_ring_frame` | v1.16–v1.18 | Cross-process exported 2D Vulkan images (`OPAQUE_FD` / `OPAQUE_WIN32`), ring synchronization handles, and completed slot index for sandboxed runners. |
| `emusen_frame_info` | `56` / `64` bytes (v1.13) | Output dimensions, pixel format, stride, `row_repeat`, frame `serial`, and timing/geometry metadata. |
| `emusen_media` | `48` / `56` bytes (v1.16) | Disc image and companion files passed to `emusen_core_media_insert`, with optional `image_path` (v1.16). |

- **Pixel formats (`emusen_pixel_format`):** `EMUSEN_PIXEL_RGBA8888` (`0`, mandatory host support), `EMUSEN_PIXEL_BGRA8888` (`1`), and `EMUSEN_PIXEL_RGB565` (`2`).
- **State kinds (`emusen_state_kind`):** `EMUSEN_STATE_FULL` (`0`, persistent save state settled across core threads) and `EMUSEN_STATE_SNAPSHOT` (`1`, fast snapshot for rewind/run-ahead).
- **Asynchronous events (`emusen_event_kind`):** `AUDIO_RATE` (`1`), `GEOMETRY` (`2`), `STATE_SIZE` (`3`), `BATTERY` (`4`), `LOG` (`5`), `MACHINE_INFO` (`6`), `OUTPUT` (`7`, v1.17 — rumble motors/lights packed as `(port << 32) | output` with `0..65535` level), and `MEDIA_SEEK` (`8`, v1.4 — disc read-ahead hints).

### 1.5 Export Function Families

- **Library metadata:** `abi_version`, `capabilities`, `info`, `settings_schema`, `status_text`, `expect` (v1.17).
- **Machine lifecycle:** `create`, `free`, `machine_info`, `reset`, `set_settings`, `setting_notes`, `events`, `log_drain`.
- **Execution & A/V:** `run_frame`, `present`, `frame_info`, `frame_copy`, `frame_device`, `ring_lend`, `frame_ring`, `raw_select`/`raw_info`/`raw_copy`, `audio_drain`, `audio_peek`, `set_mutes`, `phases`, `telemetry`.
- **Input & Media:** `set_button`, `set_axis`, `set_present`, `media_lid`, `media_insert`.
- **Persistence & Cheats:** `state_size`, `state_save`, `state_load`, `battery`, `set_cheat_pokes`, `set_rom_patches`.
- **Debugging:** `debug_set`, `debug_set_ranges`, `debug_set_stack`, `debug_run_frame`, `debug_writes`, `debug_reads`, `debug_calls`, `debug_profile`, `debug_coverage`, `debug_counters`, `debug_pc`, `debug_registers`, `debug_disassemble`, `debug_translate`, plus memory-space read/write exports.

---

## 2. Sidecar Discovery and Trust (`emusen-core-host`)

`EmuSen/Cores/Shared/emusen-core-host` discovers, verifies, and loads Core ABI v1 libraries while guaranteeing **zero core code executes at application startup**.

### 2.1 `.core.json` Sidecars

Every staged core carries a `<stem>.core.json` sidecar generated during build staging (`cargo xtask stage cores` / `check-sidecars`). The sidecar stores the core's ABI version, capability bits, `emusen_core_info` JSON (supported systems, extensions, firmware slots, open-firmware availability, save media), and `emusen_core_settings_schema`.

`Platform/crystal` scans `.core.json` files through `emusen-core-host/src/discovery.rs` and `sidecar.rs` to build the console catalog, content router, and settings UI without opening a single `.so`/`.dll`. A core library is mapped into memory only when a player actually opens a game routed to it.

### 2.2 Trust and Integrity Rules (`trust.rs`)

Before loading any core binary, `emusen-core-host` enforces three rules:

1. **Installation manifest (`cores.manifest`):** Shipped cores must match their SHA-256 digest in `cores.manifest` (`<64-hex>  <filename>`).
2. **Player approvals (`cores.approved`):** Downloaded Libretro cores and player-added cores in the config directory require an explicit approved SHA-256 entry in `cores.approved`. On Unix, `read_approvals` refuses `cores.approved` if it is world-writable (`mode & 0o002 != 0`).
3. **Binary size cap (`MAX_LIBRARY_BYTES`):** Any core library larger than **512 MiB** is refused before loading.

### 2.3 Firmware Policy and Open Replacements

EmuSen ships **zero proprietary console BIOS or firmware files**. Cores declare optional or required firmware hashes in `emusen_core_info`, and `Platform/firmware` matches player-supplied files in Galaxia's `Firmware/` directory by content hash rather than filename. Where open clean-room replacements exist — such as **Diamond OpenKernel** (PlayStation), **Nephrite OpenBIOS** (Sega CD), and built-in **VenusRT** DSP/SPC support — cores run out of the box without proprietary dumps while still letting players supply their own firmware files optionally.

---

## 3. ABI Guard and Conformance Testing

Two dedicated crates under `EmuSen/Cores/Shared/` enforce ABI compatibility continuously:

- **`emusen-core-abi-check`:** Parses `emusen_core.h` and compares every export signature, minimum struct size, constant, and status band against `EmuSen/Cores/Shared/emusen-native/abi/v1/baseline.txt`. CI rejects any change that alters or removes an existing v1 fact; only additive minor extensions pass. It also cross-checks Rust FFI declarations (`rust_check.rs`) against the C header.
- **`emusen-core-conform`:** Black-box conformance suite that dynamically loads any Core ABI v1 library (native core or hosted proxy) and exercises its full advertised contract: null-machine status checks, undersized-struct rejections, length-query copy-out handling, deterministic full and snapshot state round-trips, media lid/insert refusals, Vulkan lend/ring contracts, capability consistency, and `.core.json` sidecar parity.

---

## 4. Writing Native Rust Cores (`emusen-native` and Shared Crates)

### 4.1 `emusen-native` and `core_exports!`

`EmuSen/Cores/Shared/emusen-native` removes hand-written `extern "C"` glue from Rust cores:

- A core implements the safe Rust `Core` trait (`src/core/mod.rs`), returning typed `Result<_, i32>` status values.
- Invoking `core_exports!(Machine; reset, settings, debug, ...)` generates every required and optional `#[no_mangle] extern "C"` export using static dispatch, length-query copy-out helpers, and panic-abort safety.
- `core_exports!` verifies at compile time that the optional export groups named in the macro match the exact `EMUSEN_CAP_*` bits returned by `capabilities()`.
- `emusen-native` also provides the deterministic binary save-state codec (`writer.rs`, `reader.rs`, `skip.rs`, `shell.rs`) documented in [Save Media and Game Packages](Save_Media_and_Game_Packages.md).

### 4.2 Shipping Native Cores and Shared Infrastructure

Eight shipping Rust cores are staged by `xtask/libraries.tsv`:

| Library | Core | Systems | Crate Path |
|---|---|---|---|
| `moonrt` | **MoonRT** | NES | `EmuSen/Cores/Nintendo/MoonRT - NES` |
| `mercuryrt` | **MercuryRT** | Game Boy, Game Boy Color | `EmuSen/Cores/Nintendo/MercuryRT - GB` |
| `venusrt` | **VenusRT** | SNES | `EmuSen/Cores/Nintendo/VenusRT - SNES` |
| `marsrt` | **MarsRT** | Nintendo 64 | `EmuSen/Cores/Nintendo/MarsRT - N64` |
| `endou` | **Endou** | Master System, SG-1000 | `EmuSen/Cores/Sega/Endou - Master System` |
| `nephrite` | **Nephrite** | Genesis, Sega CD, 32X | `EmuSen/Cores/Sega/Nephrite - Genesis` |
| `diamond` | **Diamond** | PlayStation | `EmuSen/Cores/Sony/Diamond - PlayStation/v1` |
| `sapphire` | **Sapphire** | PlayStation 2 | `EmuSen/Cores/Sony/Sapphire - PlayStation 2` |

Cores share **3 hardware-family libraries** (`Nintendo/SilverMillennium-HW`, `Sega/Beryl-HW`, `Sony/BlackMoon-HW`) and **19 shared crates** under `EmuSen/Cores/Shared/`:

- **ABI & Hosting:** `emusen-native`, `emusen-core-host`, `emusen-core-schema`, `emusen-core-broker`, `emusen-core-conform`, `emusen-core-abi-check`.
- **CPU, GPU, Disc & JIT Blocks:** `emusen-6502`, `emusen-mips`, `emusen-mips-planner`, `emusen-jit`, `emusen-gpu`, `emusen-disc`, `emusen-assembler`.
- **Save-Media Filesystems (`#![no_std]`):** `emusen-psx-card`, `emusen-ps2-card`, `emusen-n64-pak`, `emusen-mcd-bram`.
- **Verification Harnesses:** `emusen-goldens`, `emusen-singlestep`.

---

## 5. Inbound Libretro Host and Landlock Confinement

EmuSen hosts selected upstream Libretro cores behind the exact same Core ABI v1 interface so Crystal, Serenity, Endymion, and the Qt frontend handle native and hosted cores identically.

### 5.1 Four-Component Out-of-Process Pipeline

Hosted Libretro cores never execute inside the main `emusen-qt` process. Five crates under `EmuSen/Cores/Libretro/` collaborate with `EmuSen/Cores/Shared/emusen-core-broker`:

```text
Platform/crystal (emusen_core.h v1)
   │
   ▼  loads in-process
emusen-libretro-proxy  (Core ABI v1 shim; broker client; enforces EMUSEN_CAP_EXPECT)
   │
   │  IPC socket + shared memory + Vulkan external-memory ring
   ▼
emusen-core-runner     (Linux child process confined by Landlock ABI v5 + no_new_privs)
   │
   ▼  loads inside sandbox
emusen-libretro-host   (translates Core ABI v1 ↔ retro_* callbacks via emusen-libretro-sys)
   │
   ▼
<core>_libretro.so     (opt-in downloaded upstream Libretro binary, SHA-256 verified)
```

- **`emusen-libretro-proxy`:** In-process Core ABI v1 library loaded by `Platform/crystal`. Calls `emusen_core_expect` (v1.17) before launching so the runner verifies the exact SHA-256 of the target Libretro binary.
- **`emusen-core-broker` / `emusen-core-runner`:** Spawns the isolated runner process, marshals Core ABI v1 calls over a binary wire protocol (`wire.rs`), and isolates crashes cleanly so a fault inside a third-party core terminates only the runner process.
- **`emusen-libretro-host` & `emusen-libretro-sys`:** Translates EmuSen's pull-only ABI into Libretro's `retro_*` callback model (`audio.rs`, `video.rs`, `vulkan.rs`, `input.rs`, `memory.rs`, `state.rs`, `options.rs`, `files.rs`).
- **`emusen-libretro-testcore` & `emusen-libretro-games`:** Synthetic Libretro test core and test fixtures used exclusively for automated verification.

### 5.2 Linux Landlock Sandbox Policy (`sandbox.rs`)

Before `emusen-core-runner` opens any third-party `.so` or game file, `emusen-core-broker/src/sandbox.rs` applies mandatory Linux confinement:

- **`no_new_privs` + Landlock ABI v5 (`AccessFs` + `AccessNet`):** Denies all network access and restricts filesystem access strictly to explicit path lists passed on the command line: `--read-exec` (core and system libraries), `--read` (game image, firmware, core data), `--read-write` (session scratch/saves where needed), and `--device` (GPU render nodes under `/dev/dri` with `ReadFile | WriteFile | IoctlDev | ReadDir` so Vulkan drivers can enumerate render nodes).
- **Private `/dev/shm` (`--private-shm`):** When required by a core that maps named POSIX shared memory, the runner unshares user and mount namespaces (`CLONE_NEWUSER | CLONE_NEWNS`) while single-threaded, maps only its own UID/GID, and mounts a private `mode=0700` `tmpfs` over `/dev/shm`.
- **Fail-closed policy:** If the kernel lacks Landlock support or any path fails confinement, the runner aborts with an error and **never** falls back to running unconfined.

### 5.3 Cross-Process Vulkan Ring Handoff (`EMUSEN_CAP_DEVICE_RING`)

Hardware-accelerated hosted cores (`vulkan` renderer) render directly on the GPU inside the sandboxed runner without CPU pixel readback:

1. `Platform/serenity` allocates an external-memory Vulkan image ring (`Platform/serenity/src/ring.rs`) and passes exported file descriptors across the broker socket via `emusen_core_ring_lend`.
2. Each frame, the runner renders directly into the next ring slot on the GPU and reports completion via `emusen_core_frame_ring`.
3. Serenity samples the completed Vulkan image directly as the input texture of the display/CRT shader chain.

### 5.4 Opt-In Download, Staging, and Allowlisted Cores

EmuSen **never ships** third-party Libretro binaries (several carry non-commercial or MAME licences). Players opt in per core via the Qt Cores view or DianaSh's `cores` command (`Platform/core-tool`).

`Platform/core-tool/src/install.rs` stages every download, update, and rollback atomically inside `/Cores/libretro/.staging/` before renaming into place:

- Enforces the **512 MiB** size limit (`CORE_LIMIT`) on the archive, declared zip entry, and inflated binary, plus a **64 KiB** limit (`INFO_LIMIT`) on `.info` metadata.
- Writes each installed binary's SHA-256 and build metadata into `NOTICE`, retains the previous build for one-step `rollback`, revokes stale hashes from `cores.approved` on upgrade/removal, and supports `verify` (`Ok`, `Refreshed`, `Changed`, `Added`).
- `Platform/systems/data/libretro-extensions.tsv` supports per-core extension overrides on top of upstream `.info` `supported_extensions` (currently empty, meaning every core uses its `.info` list unmodified).

The **10 allowlisted cores** in `Platform/systems/data/libretro-cores.tsv` are:

| Core Stem | Systems | Upstream Source | Licence | Host Renderer | Default Engine? |
|---|---|---|---|---|---|
| `pcee2` | `ps2` | `github.com/libretro/ps2` | GPLv3 | `vulkan` | Yes (`ps2` while Sapphire is in development) |
| `flycast` | `dreamcast` *(+ `naomi`, `atomiswave`)* | `github.com/flyinghead/flycast` | GPLv2 | `vulkan` | Hosted-only system |
| `snes9x` | `snes` | `github.com/libretro/snes9x` | Non-commercial | `software` | No (VenusRT default) |
| `nestopia` | `nes` | `github.com/libretro/nestopia` | GPLv2 | `software` | No (MoonRT default) |
| `genesis_plus_gx` | `md`, `mcd`, `sms`, `sg1000` | `github.com/libretro/Genesis-Plus-GX` | Non-commercial | `software` | No (Nephrite / Endou default) |
| `picodrive` | `32x` | `github.com/libretro/picodrive` | MAME | `software` | No (Nephrite default) |
| `mupen64plus_next` | `n64` | `github.com/libretro/mupen64plus-libretro-nx` | GPLv2 | `vulkan` | No (MarsRT default) |
| `sameboy` | `gb`, `gbc` | `github.com/libretro/SameBoy` | MIT | `software` | No (MercuryRT default) |
| `mednafen_psx_hw` | `psx` | `github.com/libretro/beetle-psx-libretro` | GPLv2 | `vulkan` | No (Diamond default) |
| `mednafen_saturn` | `saturn` | `github.com/libretro/beetle-saturn-libretro` | GPLv2 | `software` | Hosted-only system |

---

## Related Documentation

- [Overview and Architecture](Overview_and_Architecture.md) — Two Faces accuracy contract and core naming scheme.
- [Platform C Interfaces](Platform_C_Interfaces.md) — How `Platform/crystal` exposes cores to frontends.
- [Audio, Video, CRT, and Shaders](Audio_Video_CRT_and_Shaders.md) — Vulkan device-picture lend and cross-process ring handoff.
- [Save Media and Game Packages](Save_Media_and_Game_Packages.md) — `emusen-native` binary save-state codec and save media slots.
- [Testing and Verification](Testing_and_Verification.md) — Running `emusen-core-conform`, `emusen-core-abi-check`, and golden suites.
