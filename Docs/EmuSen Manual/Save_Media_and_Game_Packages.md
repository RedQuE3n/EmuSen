# Save Media and Game Packages

EmuSen draws a clear boundary between ephemeral emulation snapshots (**save states** and **rewind**) and persistent console storage (**save media**), and manages ROMs, multi-disc sets, and fan patches cleanly through **game packages** and **CHD compression**. Together, these layers guarantee that rewinding never overwrites shared memory cards, ROM hacks keep isolated saves, and library assets remain verifiable and portable.

For Galaxia directory locations and SQLite store paths see [Settings and Configuration](Settings_and_Configuration.md). For interactive CLI usage of `pkg`, `chd`, `rom`, `memcard`, and `scrape` see [DianaSh and Debugging](DianaSh_and_Debugging.md).

---

## 1. Binary Save States (`emusen-native`)

Native Rust cores serialize machine state using the deterministic binary codec in `EmuSen/Cores/Shared/emusen-native/src/{writer,reader,skip,shell}.rs`.

### 1.1 Writer, Reader, and Field Skipping

- **`StateWriter` (`writer.rs`):** Writes fixed-width little-endian primitives, booleans as single bytes (`0` or `1`), and fixed arrays without redundant length prefixes. Supports three operating modes:
  - `StateWriter::new(buf)` — serializes directly into a caller-supplied slice; if the buffer is too short, sets `overflowed()` while continuing to measure exact total `len()`.
  - `StateWriter::counter()` — computes exact serialized byte size with zero allocation and zero writes (backing `emusen_core_state_size`).
  - `StateWriter::layout()` — emits a line-by-line schema (`offset length type path`) for automated layout inspection and regression diffing.
- **`StateReader` (`reader.rs`):** Parses little-endian streams safely, treating any nonzero byte as `true` for booleans and returning typed `Truncated { at, wanted }` errors on short buffers. After reading the state header's integer version via `set_version(v)`, cores branch cleanly on `reader.before(v)` to skip or default retired fields from older save-state versions without breaking backward compatibility.
- **`Skip<T>` (`skip.rs`):** Zero-overhead wrapper marking host-only or derived fields (render caches, lookup tables, host handles) that are excluded from serialization and ignored in state equality comparisons (`PartialEq` always returns `true`).
- **Common State Shell (`shell.rs`):** Shared pre-validation wrapper rejecting malformed headers with typed `Refusal` variants mapped directly to Core ABI v1 status codes: `Truncated` (`EMUSEN_TRUNCATED`), `Foreign` (`EMUSEN_FOREIGN`), `Version` (`EMUSEN_STATE_VERSION`), and `BufferTooSmall` (`EMUSEN_BUFFER_TOO_SMALL`).

### 1.2 Full States vs Fast Snapshots

Core ABI v1 distinguishes two state kinds (`emusen_state_kind`):

- **`EMUSEN_STATE_FULL` (`0`):** Complete, versioned, persistent save state written to disk (`States/` and indexed in `states.db`) after core worker threads settle.
- **`EMUSEN_STATE_SNAPSHOT` (`1`, `EMUSEN_CAP_SNAPSHOT`):** Lightweight in-memory snapshot optimized for rewind rings and run-ahead.

---

## 2. XOR-Delta Rewind Ring (`Platform/crystal/src/rewind.rs`)

Rewind and the visual **Moments** preview reel run core-agnostically inside `Platform/crystal` on top of `EMUSEN_STATE_SNAPSHOT`:

### 2.1 Delta Encoding and Memory Budgets

- **Whole head + XOR delta chain:** The newest captured snapshot is kept whole in memory; each older snapshot is stored as a compact XOR delta against its successor computed on a dedicated background worker thread so emulation frames never stall on compression.
- **Self-inverting run-length XOR format (`encode` / `apply`):** Emits repeating `[varint identical_run][varint differing_run][diff_xor_bytes]` records using 8-byte fast-skip scans and omitting identical trailing runs. Because XOR is its own inverse, `apply()` reconstructs any prior snapshot in place while checking bounds against state length (`InvalidDataException` fault on corrupt deltas).
- **Zero steady-state allocation:** Once the ring reaches its memory ceiling, evicted delta buffers are recycled back into the encoder pool so steady-state recording performs zero heap allocations.
- **Moments thumbnails (`Thumbnail::from`):** Downscales RGBA frames (respecting `row_repeat` for doubled/interlaced scanlines) via 4× box sampling into compact RGB565 thumbnails displayed in the frontend Moments reel.
- **Default constants:**
  - `DEFAULT_INTERVAL_FRAMES = 4` (one snapshot every 4 frames).
  - `DEFAULT_BUDGET_BYTES = 96 MiB` (XOR delta chain budget).
  - `DEFAULT_THUMBNAIL_BUDGET_BYTES = 32 MiB` (Moments RGB565 thumbnail budget).

### 2.2 Speed Modes and Rewind Immunity

`Platform/crystal/src/speed.rs` pairs rewind with host speed control (`SPEED_NORMAL`, `SPEED_SLOW`, `SPEED_TURBO`, `SPEED_DRAW_NONE`).

Because per-game battery SRAM belongs solely to the running title while multi-game memory cards and console internal backup RAM hold saves for *other* games too, rewinding or loading a save state never rolls back shared removable cards or console internal memory — preventing accidental loss of progress in unrelated games.

---

## 3. Unified Three-Category Save Media Model

EmuSen classifies persistent console storage into **three categories**, managed uniformly by `Platform/savemedia` and Core ABI v1:

| Category | Examples | Storage Scope | Rewind / State Scope |
|---|---|---|---|
| **1. Per-Game Battery SRAM** | NES/SNES/GB/GBC/MD/SMS cartridge SRAM, N64 EEPROM & FlashRAM | One file per game (or per patched package) | Tied directly to that game |
| **2. Removable Memory Cards & Paks** | PlayStation 1 Memory Card, PlayStation 2 Memory Card, N64 Controller Pak, Sega CD RAM Cartridge | Shared or per-game virtual card files inserted into console slots | Protected from cross-game state rollback |
| **3. Console Internal Memory** | Sega CD internal backup RAM (`BRAM`) | Console-scoped persistent volume | Protected from cross-game state rollback |

### 3.1 Shared `#![no_std]` Card Filesystem Crates

Four zero-allocation, `#![no_std]` crates under `EmuSen/Cores/Shared/` parse and mutate binary card filesystems in place. Because they require neither `std` nor heap allocation, the exact same filesystem implementation is shared by EmuSen's host card manager (`Platform/savemedia`) and open console firmware inside native cores:

| Crate | Hardware Medium | Binary Format Specifications |
|---|---|---|
| `emusen-psx-card` | PlayStation 1 Memory Card | `128 KiB` (`131,072` bytes), `15` save blocks × `8 KiB`, `128`-byte frames, `21`-byte directory names at `0x0A`, Shift-JIS titles (`64` bytes at `0x04`), allocation markers `0x51` / `0x52`. |
| `emusen-ps2-card` | PlayStation 2 Memory Card (`.ps2`) | `8,650,752` bytes (`16,384` raw pages × `512` data + `16` spare ECC bytes), `16` pages/erase block, `2` pages/cluster (`1 KiB`), superblock + double-indirect FAT + `512`-byte directory entries. |
| `emusen-n64-pak` | N64 Controller Pak | `32 KiB` (`128` pages × `256` bytes), redundant ID page + primary/backup page index (`END_OF_NOTE = 0x0001`, `FREE_PAGE = 0x0003`), `16` note entries (`32` bytes each), `123` data pages (`FIRST_DATA = 5`). |
| `emusen-mcd-bram` | Sega CD Internal BRAM & RAM Cart | Power-of-two volumes from `8 KiB` (`0x2000`) to `512 KiB` (`0x80000`), `64`-byte blocks, `11`-byte names, `SEGA_CD_ROM` / `RAM_CARTRIDGE___` footer + block code. |

### 3.2 Card Manager (`Platform/savemedia`) and `memcard` CLI

- **`Platform/savemedia`:** Combines the four format crates with Galaxia's atomic file replacement and per-medium locking so every operation is crash-safe and requires no emulator core to be loaded. Exposed to frontends via `emusen_crystal_cards_op` (`LIST`, `OPEN`, `NEW`, `COPY`, `MOVE`, `DELETE`, `DUPLICATE`, `RENAME`, `IMPORT`).
- **`memcard` CLI (`Platform/emusen-memcard-module`):** Exposes card management directly from DianaSh — listing saves, formatting new blank cards, copying/moving/deleting saves between cards, and importing/exporting individual save blocks. See [Docs/man/emusen.memcard/memcard.1](../man/emusen.memcard/memcard.1).

---

## 4. Game Packages, Patching, and CHD Compression

`Platform/library-tool` executes packaging, non-destructive ROM patching, CHD disc compression, and ROM integrity checks as structured background jobs (`emusen.library` service table, `emusen_library_tool.h` v1.7) shared by both `emusen-qt` and DianaSh (`pkg`, `chd`, `rom`).

### 4.1 Package Layout and Three Storage Modes

A game package is a folder (or `.zip` archive) under Galaxia's `Packages/` directory anchored by a `package.json` manifest recording title, ES-DE system slug, mode, media entries (`where`, relative path, file size, cryptographic hashes), and optional soft patches:

| Mode | Manifest `mode` | How Game Files Are Stored | Key Properties |
|---|---|---|---|
| **Linked** *(default)* | `"linked"` | References the original ROM or CHD in the player's ROM library or `Compressed/` (`where: library`) along with recorded size and hashes. | Zero disk duplication; leaves source ROM folder read-only. If the image is moved inside the library, `pkg check` automatically rediscovers it by size + hash and repairs the link. |
| **Self-Contained** | `"self-contained"` | Copies the game image and every companion file (`.cue`/`.bin` tracks, multi-disc sets) beside `package.json` (`where: package`). | Fully portable folder independent of the source library; never auto-relinks back to the library. |
| **Zipped** | `"self-contained"` in `.zip` | Archives a self-contained package folder into a single `.zip` beside it (`pkg make --self-contained --zip` or `pkg zip`), preserving manifest and saves. | One-file distribution/archival. Linked packages refuse zipping because the archive would hold no game image. |

### 4.2 Non-Destructive Patching (`Platform/patch`) and Isolated Saves

`Platform/patch` applies standard ROM patches cleanly in memory at launch without ever modifying the source ROM on disk. Patch formats are auto-detected by header magic:

| Format | Extension | Header Magic |
|---|---|---|
| **IPS** | `.ips` | `PATCH` |
| **BPS** | `.bps` | `BPS1` |
| **UPS** | `.ups` | `UPS1` |

When patches are attached via `pkg patch add` / `pkg patch rm`:

- Patches apply in manifest order onto the base ROM bytes at launch.
- The patched package receives an **isolated save identity** (separate battery SRAM, save states, and screenshots) so playing a fan translation, randomizer, or ROM hack never overwrites the base game's saves.
- `pkg identity <package> original|patched` controls whether the scraper queries metadata using the original unpatched ROM hash or the patched hash.

### 4.3 CHD Disc Compression (`chd` and Pinned `chdman`)

Disc games (PlayStation, PlayStation 2, Sega CD, Saturn, Dreamcast) can be compressed losslessly to **CHD** via `Platform/library-tool/src/convert.rs`:

- Invokes EmuSen's pinned MAME `chdman` binary verified against its SHA-256 pin (`pin.rs`).
- Writes CHDs by default into Galaxia's `Compressed/` directory mirroring the library's relative path (leaving `AppSettings.RomDirectory` strictly read-only), or to a custom `ChdOutputDirectory` (`--output <folder>`) or per-batch `--beside` destination.
- Verifies every produced CHD three ways: `chdman verify` plus two structural validation checks through EmuSen's own `emusen-disc` reader, then creates a linked package automatically.

### 4.4 `chd`, `pkg`, and `rom` Command Reference

| Command | Subcommands | Purpose |
|---|---|---|
| `chd` | `ls [<path>...]`, `convert <path>... [--fast] [-j N] [--output DIR \| --beside]`, `check [<path>...]`, `wait [chd<n>]` | Disc-to-CHD conversion and verification over library-relative paths (`/` for all). |
| `pkg` | `make`, `ls`, `check`, `patch add/rm`, `zip`, `migrate`, `identity <pkg> original\|patched`, `wait [pkg<n>]` | Create linked/self-contained/zipped packages, attach patches, check hashes, and migrate saves/states. |
| `rom` | `hash [-r] [--comparable] <path>...`, `check [-r] <path>...`, `dups [<console>]`, `wait [rom<n>]` | Compute CRC-32/MD5/SHA-1 hashes (including normalized `--comparable` disc hashes), validate sheets/archives/CHDs, and find duplicate dumps. |

---

## 5. Library Integrity and Scraping (`Platform/scrape`)

`Platform/scrape` (`emusen_scrape_service.h` v2.1) runs as an independent component beside `Platform/library` (`games.db`) and `Platform/crystal`, owning `media.db` and Galaxia's `Media/` tree:

- **Player-initiated queue & dry-run planning:** Scraping never runs silently in the background. Before downloading, the planner (`what: "plan"`) reports total games, unqueried vs refresh counts, expected HTTP requests, estimated megabytes, remaining daily ScreenScraper quota, and estimated minutes.
- **Providers and media kinds:** Queries **ScreenScraper** using configured region (`ScrapeRegion`), language (`ScrapeLanguage`), and region fallback (`ScrapeRegionFallback`), with optional **OpenEmu** cover fallback (`OpenEmuFallback`). Fetches covers (`ScrapeCovers`), screenshots (`ScrapeScreenshots`), marquees (`ScrapeMarquees`), title screens (`ScrapeTitleScreens`), and miximages (`ScrapeMiximages`), and lets players promote any local screenshot to a game's cover art.
- **Credential redaction (`redact.rs`):** Every error string, HTTP diagnostic, and run log passes through redaction so ScreenScraper credentials and developer keys never leak to logs or shell output.
- **CLI access (`scrape`):** `Platform/emusen-scrape-module` exposes planning, queuing, status checks, and cover management directly from DianaSh. See [Docs/man/emusen.scrape/scrape.1](../man/emusen.scrape/scrape.1).

---

## Related Documentation

- [Settings and Configuration](Settings_and_Configuration.md) — Galaxia directory tree (`Saves/`, `States/`, `Packages/`, `Compressed/`, `Media/`) and SQLite databases.
- [Core ABI and Libretro Host](Core_ABI_and_Libretro_Host.md) — `EMUSEN_CAP_SNAPSHOT`, `EMUSEN_CAP_BATTERY_DIRTY`, and `EMUSEN_CAP_MEDIA`.
- [DianaSh and Debugging](DianaSh_and_Debugging.md) — Running `pkg`, `chd`, `rom`, `memcard`, and `scrape` in DianaSh.
- [User Guide](User_Guide.md) — Managing save states, Moments rewind, and packages inside `emusen-qt`.
