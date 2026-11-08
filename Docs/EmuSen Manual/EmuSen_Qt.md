# EmuSen_Qt — the Qt 6 front end: layers, surfaces, themes and the way there

Design, 2026-11-07, revising the Qt plan of 2026-11-01 (which named LunaP as a toolkit to port; §0.2 says what changed).
**Approved by the tester on 2026-11-07, with every recommendation of §11.** Nothing of it is built in the tree; a
toolchain spike outside the tree proved the binding, the drawn-control path and run-time themes on the desktop and the
handheld, and its figures are in §10. **Marks:** **[D]** decided by the tester, with the date; **[M]** measured, with
the date and how; **[A]** argued; **[decision]** a question whose answer changes scope, an interface, cost or what the
player sees; **[routine]** a question with a recommended answer that changes none of those.

**Reading order.** §0 for the requirements and decisions, §1 for the layers and their surfaces, §2 for the house rules
applied layer by layer, §3 for the theme model (the test of the API is §3.6, four console-style designs as themes), §4 for
ES-DE themes, §5 for what is stable for developers, §6 for adaptive cores, §7 for the migration, §8 tests, §9 packaging and
the handheld, §10 the spike, §11 risks and questions, §12 sources.

---

## 0. What is required, and what is already decided

### 0.1 The requirements (the tester, 2026-11-07, verbatim)

1. "It must be able to render our EmuSen theme perfectly and allow us to go further with design to match interfaces such
   as the steam deck, xbox, playstation, and switch."
2. "It must be able to use ESDE themes, currently we are working on compatibility because all themes arent compatible,
   but I am hoping after our switch, we can work on this."
3. "It must have a stable API/ABI for developers and a sane way for people to make QML themes."
4. "We believe in adaptive styling and code, both front end and back end. We want to be able to add retroarch cores and it
   not take a million years to do so. We also want it so when we add our own cores, it should just be a common every day
   affair rather than hard coding everything."

### 0.2 Decided before this page **[D]**

- **The binding is cxx-qt** (KDAB; 0.10.0 on crates.io, released 2026-08-24, MIT OR Apache-2.0 [M 2026-11-07]). Not Qt
  Bridges, not qmetaobject.
- **QML is layout only; every view model and all logic is Rust.** §1.3 states the rule as a checker enforces it.
- **LunaP is not ported as a toolkit.** Its own parts become a QML module inside EmuSen (§1.4): the design tokens as a
  Qt Quick Controls style, the drawn sky, bodies and Moonrise, the bell sounds, the pad's focus walk across forms, and
  the fit audit. This replaces the 2026-11-01 plan's separate `LunaP-Qt` repository and crate; what that plan mapped
  piece by piece (its §2) stands as the inventory of what is Qt's and what is ours, and §1.4 carries the result.
- **Frameworks first.** Qt Quick's layouts, styles, states and models; libretro's own declarations; proven crates.
  Hand-rolled code that a framework covers is a defect; building our own needs a written reason, and every such reason
  on this page is marked *[own, because …]*.
- **Adaptive.** A core declares and the front end derives (§6). Hand-written data is optional overrides only.
- **Every input reaches every screen [D 2026-11-07].** A pointer (mouse, touchpad or touch), the keyboard and the pad
  each reach every element of every screen in every form, big screen included; none is a second-class way in. A
  big-screen view carries no desktop chrome: no menu bar and no status bar. §1.7 has the design and §8.1 the tests. The
  reason was measured on the C# front end on 2026-11-07, in big picture with its own look on the desktop: Home has no
  pointer handling at all and the grid and Choose a console very little, so a mouse did nothing, and the desktop's status
  bar showed beneath the big-screen library reading "No ROM loaded". Neither is fixed in C#; the Qt front end is built
  without them.
- **Unix-style, modular, auditable.** Each component has one job behind a written, versioned surface; nothing reaches
  past one. §2 shows every boundary.
- **No artwork from any company or show.** Console-style designs are inspired, never copied (§3.6).
- **Platforms.** Linux first: the Fedora desktop and the Legion Go S on SteamOS, the full-speed target; desktops stay
  covered and Android stays open; iOS is out. Distribution as an AppImage and a Flatpak, both (§9); development builds on
  the distribution's Qt.
- **Feature freeze and the shape of the move.** The C# front end (Mistress) keeps running until the Qt one reaches parity
  screen by screen, never big-bang (§7). There is no release before a true 1.0, and the stable surfaces freeze then (§5).

### 0.3 What the 2026-11-01 plan got right and this page keeps

Its layer picture (the runtime behind the frontend API, cxx-qt view models, layout-only QML, a style carrying the
tokens), its pad bridge (Qt 6 has no gamepad module; the pad becomes key events after the frame has had it), its
zero-copy game picture on Serenity's Vulkan device adopted by the scene graph, its two-tier headless tests, its reading of
Kirigami (patterns borrowed, the dependency not taken) and its migration order. Each is restated here only as far as the
surface rules need; the arguments stand there.

---

## 1. The layers and their surfaces

```
 ┌───────────────────────────────────────────────────────────────────────────────────────────────┐
 │ Themes (data + QML, outside the binary)        EmuSen theme (reference) · console-style themes  │
 │   import EmuSen.Theme 1.0          ES-DE themes: XML, drawn by the ES-DE engine, never QML        │
 ├───────────────────────────────────────────────────────────────────────────────────────────────┤
 │ QML views (layout only), the screens of EmuSen_Interface.md: cards, pages, Settings, tools      │
 │   import QtQuick · QtQuick.Controls (style "EmuSen") · EmuSen.Kit · EmuSen.Theme                 │
 ├───────────────────────────────────────────────────────────────────────────────────────────────┤
 │ EmuSen.Kit  (QML module + style + Rust)   tokens · Frame (3 forms) · Pad · Sky, Body, Moonrise   │
 │   emusen-kit-qt (cxx-qt)  over  emusen-kit-core (pure Rust: walk, grammar, tokens, audit, bells)  │
 ├───────────────────────────────────────────────────────────────────────────────────────────────┤
 │ View models  emusen-qt-models (cxx-qt QObjects, QAbstractListModels): Library, Session, Device, │
 │   Cores, Settings, Scraper…  one per screen or role; the ES-DE engine's ElementModel beside them  │
 ├───────────────────────────────────────────────────────────────────────────────────────────────┤
 │ Client bindings  emusen-crystal-client (over emusen_crystal.h 1.x)  ·  emusen-platform-client     │
 │   (over emusen_platform.h, exact match)                                                           │
 ├───────────────────────────────────────────────────────────────────────────────────────────────┤
 │ The Rust back end: the runtime (Crystal), the platform library (Galaxia, Endymion, Serenity,     │
 │   systems, scraping, cheats), cores behind the core ABI v1, libretro cores behind the host         │
 └───────────────────────────────────────────────────────────────────────────────────────────────┘
```

Each layer below names its one job, its surface and the surface's rule (`EmuSen_Surfaces.md` §6.1: **A** stable C ABI,
**B** exact C interface, **R** internal Rust API, and two new ones this page adds, **Q** a versioned QML module and **T** a
versioned theme package, both defined in §5).

### 1.1 The back end, through the frontend API

The front end is a client in the sense of `EmuSen_Surfaces.md` §6.2 rule E: it holds the runtime through
`emusen_crystal.h` (version 2.0 today; rule A) and the platform library through `emusen_platform.h` (interface 39; rule
B), and never a runtime type. 2.0 removed the session service's twins (`EmuSen_RustPlatform.md` §47.23): the debugger,
states by path, the log switch and the cheat ops are called through the `emusen_session_v1` table that
`emusen_crystal_session_service` hands out, the one the shell is handed too. That the implementation behind both
headers is Rust changes nothing: the header is the contract. Two client crates carry them: `emusen-crystal-client`, a safe Rust binding generated from `emusen_crystal.h`
(bindgen at build time, the generated module committed and diffed so a header change is a reviewed line), with typed
readers for the JSON shapes the header names (`systems`, `sessions`, `shelves`, `engines`, `cores`, `route`,
`session_info`, the settings schema), the per-frame calls left as thin wrappers over the size-prefixed structs, and the
session's table wrapped once, its failures read with the table's own `last_error`; and
`emusen-platform-client`, the same over `emusen_platform.h`, matched exactly to the library built from the same commit.
Neither crate knows Qt. Everything the front end shows comes through these two: the library and its shelves, a game's
route, sessions and frames, settings, firmware, cards, the pads (Endymion), sounds (Endymion's interface player), the
shader chain and the device (Serenity), the systems table, scraping jobs, cheats and the libretro cores table. **Pull,
never push** holds: notifications are drained from a subscription on a timer or waited on with a timeout, never a callback
into a QObject.

*What this costs and buys.* A C hop between two Rust halves is one indirect call and one JSON parse on the non-frame path;
the frame path crosses structs and buffers. It buys an auditable boundary (two headers, both already baselined in CI),
and both front ends interchangeable while both ship.

#### 1.1.1 `emusen-crystal-client` as built **[M 2026-11-08]**

`Frontend/crystal-client`, a crate outside the Qt workspace with its own lock file, as `emusen-kit-core` is, so that its
fence test reads what its tree names (no Qt crate; no `emusen-*` crate but itself, the shared idioms and its test
fixture; no other path dependency). Its dependencies are `libloading` 0.8, `serde` 1, `serde_json` 1, `thiserror` 2 and
`emusen-c-calls` (below); `bindgen` 0.72 and `emusen-client-fixture` (§1.1.3) are dev-dependencies. It names no runtime crate: the
platform library is reached at run time, by its path, as any front end reaches it.

**The raw layer** is `src/raw.rs`, bindgen 0.72.1's output for `emusen_crystal.h` and the `emusen_session.h` it includes
(allow-listed by file), committed. It is generated for dynamic loading (`dynamic_library_name`, with
`dynamic_link_require_all`, so that every function of the header is resolved when the library is loaded and a missing
one refuses the load with the loader's words naming it), with `derive_default`, `wrap_unsafe_ops` (edition 2024) and
bindgen's layout tests, which hold each struct's size and offsets to the header at compile time. `tests/binding.rs` runs
the same builder and fails on any difference from the committed file; `EMUSEN_BINDGEN_WRITE=1` writes it instead, so a
header change reaches the client as a reviewed diff of `raw.rs`. rustfmt formats the output with the crate's
`rustfmt.toml`, the kit's.

**Loading** (`Api::load`). The library is opened, `emusen_crystal_abi_version` alone is resolved and called, and a major
other than 2, or a minor below the header's, is refused with words that name the path and both versions, before any
other symbol is read (`Error::Refused`); a later minor is taken, since a minor only adds. Then every symbol is resolved.

**One owner for each idiom.** How a client calls an EmuSen C header is `emusen-c-calls`'s (`Frontend/c-calls`), shared
by both client crates *[own, because the headers' calling conventions are theirs and no crate speaks them]*: the
library opened by its path with its version symbol read before any other; text in as a pointer and a length, none
being the absent string; the copy-out, which asks the length, copies, and asks again when the answer grew between the
two calls (its own four tests); a negative return made an `Error::Status` carrying the header's name for it and the
words read at once on the same thread, with the header's ABSENT read as `None` where a call may answer it; and an
answer read into its record, a refusal of serde's being `Error::Shape` with the call's name. The error type is that
crate's, so both clients fail alike; each client gives it only its header's status names.

**Typed readers** (`src/shapes.rs`, serde) for `systems` (`SystemRow`: the table's 24 keys, then `engines`,
`starts_empty`, `core` and `controllers`; a system only an engine names reads with the table's keys at their defaults),
`sessions`, `shelves`, `engines`, `cores` (with the libretro row), `route`, `session_info`, `settings_schema`,
`firmware_needed`, `firmware_overview`, `firmware_status`, `cheat_codec_list`, `cheat_decode`, `state_slots`,
`state_record`, `rewind_reel`, `save_media`, and the events a subscription drains (`Event`, tagged by `kind`, a kind a
later minor adds being `Other`). Keys a later minor adds are skipped; a key the header gives as possibly null is an
`Option`. A firmware request keeps the entry as the runtime wrote it, which `firmware_install` hands back whole.

**The per-frame calls** (`src/frame.rs`) are thin wrappers: `frame` and `rewind_step` fill a caller's
`FrameInfo` (`emusen_crystal_frame_info` itself, its `size` set by `frame_info()`), `picture_copy`, `raw_copy`,
`audio_drain` and `phases` fill a caller's buffer, and the input, rumble and speed calls pass numbers. Nothing on that
path allocates but a failure's words. `picture_lend` and `picture_ring` return the pointer for Serenity to borrow,
dereferenced nowhere here.

**The session service's table** (`src/service.rs`) is wrapped once, as `SessionService`, which keeps its runtime alive
(the table lives as long as the runtime). Its major is checked; each entry is read only after the table's `size` is
seen to reach it, as `EMUSEN_SESSION_HAS` asks *[own, because that is a C macro bindgen does not translate]*, and
through raw pointers, so a provider's shorter table is never read past its end; an entry the table lacks is
`EMUSEN_SVC_NOT_SUPPORTED` with words that name it; and every failure's words are the table's own `last_error`'s. Wrapped
now: `sessions`, `info`, `summary`, `run_control`, `cheat_op`, `cheat_codecs`, `telemetry`, `audio_channels`,
`set_channel_muted`, `state_save`, `state_load`, `state_default_path`, `log_switch`, `log_status`, `events_drain`,
`games`, `open_game`, `open_console`. **Owed**, the debugger's entries, which the shell's commands reach today:
`halt_state`, `spaces`, `space_read`, `space_write`, `resolve_physical`, `processors`, `registers`, `set_register`,
`disassemble`, the `bp_*`, `watch_*`, `framelog_*`, `freeze_*` and `label_*` families, `coverage_op`, `profile_op`,
`counters_op`, `call_stack`, `dma_channels`, `dma_log_op`, `register_flow_op`, `coprocessor_history`, `vectors`,
`tilemap_entry`, `tile_pixels`, `sprites`, `palettes`, `evaluate`, `symbols`, `trace`, `trace_off`, `tile_sheet`,
`palette_swatch` and `audio_peek`. Also owed: typed records for the card manager's answers and the table's cheat,
telemetry and channel answers (read as `serde_json::Value` today), and the config's SQLite lend (none is lent, so the
state records' calls answer `NOT_SUPPORTED` until the front end lends one).

**Threads.** `Runtime` is a shared handle (`Arc`), freed with its last clone; it is `Send` and `Sync` because the header
answers a read on any thread and makes a change from another thread wait for the frame boundary. A `Subscription` is
`Send` and is drained or waited on by its owner, never called back.

**A finding, measured 2026-11-08.** `firmware_status`'s `"need"` is written as a string naming the need (`"file:boot.rom"`
for the test core's boot ROM; `Platform/crystal/src/firmware.rs`), where the header's comment says it is the index of
the first file meeting the same need. The reader follows the library (`need: String`), and the header's comment was
corrected to say what the library writes, a comment only: the ABI test strips comments, so its baseline is unchanged.

**Proof, measured 2026-11-08** against the platform library built from the same commit (§1.1.3): 9 tests pass, the
binding's regeneration, the fence's two, the version check's unit test, and five against the library: the listings read
into their records (the table's NES, the SNES Game Genie encoding and decoding `0xEA` at `0x8000`, an unknown codec
`ABSENT`); a file that is no library, and a library without the API, refused with the loader's words; a player's core
approved by its hash through `core_approve`, routed, opened, run ten frames with the picture and sound copied, its
schema read (the core's `Rate` a choice of `32000` and `48000`, and the `Engine` row), a console setting set and read
back, a state saved and loaded through the table, the session closed and its table then answering `NO_SESSION`, and
the subscription's `opened`, `Rate` and `closed` events pulled; and failures carrying their own words (a file nothing
opens is `NO_ENGINE` with the runtime's words; a cheat op asked of the table with no game is `NO_SESSION` with the
table's "No ROM loaded"). Four faults were seeded and caught: the table's failures read through the runtime's
`last_error` (`a_failure_carries_its_own_words`), a constant edited by hand in `raw.rs` (`binding.rs`), any major
accepted (the unit test), and `need` read as the header's index (the session test).

#### 1.1.2 `emusen-platform-client` as built **[M 2026-11-08]**

`Frontend/platform-client`, beside the crystal client and held to the same fence, with the same dependencies
(`libloading`, `serde`, `serde_json`, `thiserror`, `emusen-c-calls`; dev: `bindgen`, `emusen-client-fixture`). Its raw
layer is bindgen's output for `emusen_platform.h` (with `emusen_scrape_service.h`, which it includes), `emusen_library.h`
and `emusen_games_service.h`, all served by the one library, generated, committed and regenerated by `tests/binding.rs`
as §1.1.1 describes: 268 functions at interface 40, every one resolved at load.

**Matched exactly** (rule B). `Platform::load` reads `emusen_platform_abi_version` alone first and refuses any number but
the one the binding was generated for, older or newer, with words that name the path, both numbers and that the two are
built from the same commit (`Error::Refused`); only then is every symbol resolved.

**Covered, for stage 1** (60 of the 268):
- *The platform's own:* the interface number, `last_error`, the thread's diagnostics, and the lending of a host's
  library. SDL3 and SQLite are the host's, lent by their loader's handle for the process's life; `Lent::open` loads one
  by path or name (`libSDL3.so.0`, `libsqlite3.so.0`) and never closes it, since the platform library holds it.
- *Galaxia:* a directory of the tree, Galaxia's files by number (the settings file and the stores, `file`, since 40),
  a config file's path, and the config models (`AppSettings`, `AudioConfig`,
  `GraphicsConfig`) loaded, saved and made new as the documents Galaxia binds. A model crosses as `serde_json::Value`
  and its fields are not restated, so Galaxia stays the one owner of each model.
- *The library* (`emusen_library.h` 1.3): a handle on games.db over the SQLite lent; scan, progress, cancel, the last
  scan's games (typed), `get` and `change` as the header's JSON, with typed reads for a game's record, the collections
  and the recently played. The `emusen.games` service speaks for the same file to a shell and has no front-end entry:
  a front end keeps its own handle, as that header says.
- *Endymion's pads:* SDL's pads or a simulated set's, started, polled and taken as events (opened or closed, announced
  or not, by key), the open keys, a pad's numbers, texts and axes, the PadButton as the game hears it, the player light,
  the motors and the five settings; simulated pads and sets, pressed and plugged as tests do. Since 2026-11-08 (the pad
  bridge, `EmuSen_Kit.md` §7.15): the family a pad's buttons print (`FAMILY`, interface 41), `show_only` for a test's
  virtual pads, and a simulated pad's numbers and texts set (its SDL type, its name).
- *Endymion's binding files* (since 40): each of the four loaded for the consoles a program knows, saved, and a map's
  defaults, as JSON that stays Endymion's (`EmuSen_Input.md` §5.2); a file that would not read leaves its diagnostic
  in `diagnostics`. `DeviceModel` and `SettingsModel` read and write the bindings through these.
- *Endymion's interface player:* made once SDL is lent (refused with words before), samples remembered and played, the
  device by name and the volume; since 2026-11-08 the bytes still queued, which prove a play.
- *The rule for SQLite* **[D 2026-11-08]**: the Qt front end lends the system's `libsqlite3.so.0` to every call that
  takes one (the library handle, the scraper and, once wrapped, the runtime's states), so the process holds one copy of
  SQLite, as `EmuSen_RustPlatform.md` §47.25's argument requires.
- *The firmware folders* for a setting. What a game or a system needs, and whether the library holds it, is the
  frontend API's (`firmware_needed`, `firmware_status`), as are the systems table and the cheat codecs since interface
  38 (`EmuSen_RustPlatform.md` §47.24).
- *Scraping:* the process's one scraper opened (closing any before it) and closed by its serial, the `emusen.scrape`
  2.0 table wrapped once (each entry read only where the table's size reaches it, its failures read with the table's
  own `last_error`), and the scrape command as data. Requests and answers stay `serde_json::Value` until the Scraper
  screen types them; the theme loader the open may name is not passed yet.

**Owed, by name**, the rest of the header: Galaxia's `init`, `set_override` and `thread_scope` (a test's; the fixture
calls the one it needs), the legacy and bundle paths, `save_path`, `file_read`, `file_write`, the ROM hashes and
tracks, packages, mapped files, migration, `config_read`, `config_write` and `config_delete`, `model_bind`,
`model_format`, `model_load_from`, `model_save_to`, the cheat names and lists, the error log, `suggest`; the library's
`identity` and `abi_version`; the library tool's jobs (`emusen_tool_job_*`); `emusen_patch_package_image`; the disc sets
(`emusen_disc_*`); emusen-systems' libretro cores, save media formats, code, recognition, route, OpenVGDB, header,
disassembly, board name and view; `emusen_firmware_find` and `_locate`; the cheat files and database
(`emusen_cheats_*`); the HTTP client (`emusen_net_download_*`); the medium locks; Endymion's resampler, rate control,
router, player slots, `pad_resolve`, `pad_combine`, `pad_controls_for`, `pads_use`,
`pads_rescan_due`, the simulated pads' texts read, holds and device calls, the game's audio player (`emusen_endymion_audio_*`),
the interface player's `decode`, `decode_take` and `volume`; and the rest of Serenity (sources, the compiler,
reflection, the cache, the lend, the ring, and the chain's other calls); since interface 42 its `serenity` module covers
the device made for an adopting host, its raw handles, a built-in filter's preset, and a chain advanced and drawn in place
(§1.8). Each is wrapped when a
screen needs it: the router and slots with the Device screen, the audio player, rate control and Serenity with the
game picture (§1.8), the library tool and the packages with their windows.

**Proof, measured 2026-11-08** against the library built from the same commit: 12 tests pass (13 at interface 40, below), the binding's
regeneration, the fence's two, the interface check's unit test, seven against the library and one in a binary of its
own. The library matched exactly, and a file that is no library and a library without the interface refused with the
loader's words; Galaxia's config directory inside the scratch tree, `AppSettings` made new, its `RomDirectory` saved and
loaded back; a synthetic NES image (an iNES header and 16 KiB of zeros) scanned as the one `nes` game, a favourite
toggled, a collection made with the game in it, the game started and listed as recently played, and an unknown read
refused with the library's own words; a simulated pad opened unannounced at start, its south button read as held, its
motors driven to 1000 and 2000, a setting set and read, and its unplugging polled as closed; the interface player
refused before SDL is lent; the firmware folders `Unset` with the store among them; the scraper opened on the system
SQLite, read, an unknown read refused as `BAD_REQUEST` with the table's own words, and closed once by its serial; and,
alone in its binary, SDL3 lent on its dummy audio driver (no sound reaches the speakers) and a remembered tenth of a
second of silence played. The scraper with no SQLite lent looks for the C# program's `libe_sqlite3.so`. Three faults were seeded and caught: a newer interface taken (the unit test), the library's
failures read with the platform's `last_error` (the library test) and pads' opened and closed swapped (the pads test).

**Interface 40, measured 2026-11-08.** The binding was regenerated (four functions and ten numbers more) and one test
added: Galaxia's settings file is the config directory's `appsettings.json`, games.db and media.db sit in the library
and media folders of the scratch tree, and an unknown number is refused; a console the program names is given the
default keys; a player's map written with a word in lower case is saved under its `SNES Player 2` key by its name and
read back; a misspelled hotkeys file loads as the defaults with exactly one diagnostic, naming the file, the word and
the nearest action, though the length-query idiom calls twice; the profiles have no map of defaults; and a misspelled
key is not saved, with the words. All 13 pass.

#### 1.1.3 The clients' test world: `emusen-client-fixture` **[M 2026-11-08]**

`Frontend/client-fixture`, a dev-dependency of both client crates and nothing else, with one job: a test world built
from the same commit. `library()` and `test_core()` run cargo once a process on this checkout's `Platform/`
(`-p emusen-platform --lib -p emusen-crystal --example crystal_test_core`, profile `test-fast`, the land gate's for that
workspace, so its build is shared) *[own, because the platform library is a cdylib of another workspace and cargo has no
artifact dependency on stable]*; `EMUSEN_PLATFORM_LIBRARY` and `EMUSEN_TEST_CORE` name prebuilt ones instead. `World`
makes a scratch tree under `TMPDIR` and moves Galaxia's config and data redirects into it with
`emusen_galaxia_set_override`, the one call it makes, so no test touches the player's tree; since the redirects are
the process's, one world is held at a time. A game is the test core's image (`V1TC`, a little-endian length, the
payload), and a player's core is the test core copied unread into a player's cores folder, approved by the test through
the API as a player approves one. Building both took 27 s from a clean target with sccache's cache warm, on 2026-11-08,
and about a second when nothing changed.

### 1.2 View models: `emusen-qt-models`

One job: turn the client crates' answers into Qt's model and property types for QML, and turn QML's actions into client
calls. One `#[cxx_qt::bridge]` QObject per screen or shared role, as `EmuSen_Interface.md` names them: `LibraryModel`
(consoles with games, the places, shelves, a console's games, collections, search), `GameModel` (one game's page),
`SessionModel` (the running game: paused, rewind, states, discs, speed), `DeviceModel` (controllers, clock, battery,
network, output), `CoresModel`, `SettingsModel` (the six sections, derived from the schema, §6.3), `ScraperModel`,
`PackagesModel`, `CheatsModel`, `FirmwareModel`. Rows are `QAbstractListModel` subclasses (`#[base = QAbstractListModel]`,
`data`, `rowCount`, `roleNames` overridden, as the spike's model does, §10) over a keyed `Vec<T>`, refreshed keeping the
selection. Background work (a scan, a download, the frame pacer) reaches the GUI thread through cxx-qt's
`qt_thread().queue(...)`, the one mechanism; nothing else touches a QObject off its thread. Every word a screen shows is a
property or a role from here, so QML carries no strings.

The crate's surface is the `.qmltypes` registry its bridges generate (rule Q in §5.3, module `EmuSen.Models` 1.0); it is
internal to the application until 1.0 and a theme never imports it directly: themes see the curated `EmuSen.Theme` module
(§3.2), which is a view over these models.

#### 1.2.1 `LibraryModel` as built **[M 2026-11-08]**

`Frontend/qt-models`, the crate `emusen-qt-models` (module `EmuSen.Models` 1.0), a member of the Qt workspace. Its
dependencies are the workspace's cxx-qt 0.10 crates, `serde_json`, and the two client crates by path; the kit's harness
and the clients' fixture are dev-dependencies. It names no C header and no runtime crate.

Four parts, each with one job:
- `keyed.rs`, **`KeyedRows<T>`**: rows held by a key that stays the same across refreshes, with one selected. A refresh
  replaces them whole; the selection follows its key, stays at its index clamped to the last row where its row is
  gone, and starts at the first row. *[own, because QAbstractListModel keeps no selection and knows no keys]*. Every
  later model holds its rows in it.
- `library.rs`, **the view as plain Rust**: a `LibrarySource` trait (the systems, the games found afresh, how many are
  favourites, how many have been played) and the rows a source's answers make: the three places (`all`, `favourites`,
  `recent`, with their names and counts), then each system that has games, in the systems table's order, named by the
  table's `display_name` or else the declaration's `name`, with the look's hue and body, the maker, the year, and the
  first engine the runtime lists for it. Nothing per console is written: a system only an engine declares shows by its
  declared name and computed look. A failed refresh keeps the rows and shows the source's words.
- `clients.rs`, **the source over the clients**: the systems and shelves from `emusen-crystal-client`, the games from a
  scan of the library handle whose roots are the player's ROM folder (`AppSettings.RomDirectory`, read through Galaxia's
  model) and whose systems are the shelves' (as `emusen_library.h` says), with the packages and compressed folders
  beside, and the records from games.db (`records` once, `recent` once).
- `library_model.rs`, **the bridge**: `LibraryModel`, a `#[base = QAbstractListModel]` QObject over a `LibraryView`,
  roles `kind`, `key`, `name`, `count`, `hue`, `body`, `maker`, `year` and `engine` (from 0, §10.4); read-only
  properties `selectedIndex`, `selectedKey`, `problem` and `loading`; a writable `platformLibrary`, plain text a window
  binds once; invokables `refresh()` and `select(index)`. Naming the platform library opens the clients
  and refreshes on a thread of its own, and the answer comes back to the GUI thread only through
  `qt_thread().queue`, where the rows are replaced inside `beginResetModel`/`endResetModel` (a whole reset, the simplest
  correct change; finer signals wait for a measured need) and the selection kept by its key. Before a source is named
  the model holds the places alone. Every word it shows is a role or a property; QML holds none.

**What the model is handed, and why as text.** A QML element is made by the engine, so it cannot be handed a Rust
value; the platform library's path is plain text that a window's root binds from its initial properties, and the model
opens its own clients from it. No static carries it (§2). Since interface 40 the model copies no fact of Galaxia's:
games.db is where `emusen_galaxia_file` names it, and the settings are loaded by the name of the file it names for
them, which sits in the config directory itself. The `gamesDatabase` property is gone with the copy.

**Proof, measured 2026-11-08.** Seven tests pass: four of the view with a fake source and no Qt (the places then the
consoles with games in the table's order, the Super Nintendo with none left out; a console only an engine declares,
shown by its declared name; the selection following the Game Boy when a console appears before it, and staying at its
place when the Game Boy's games are gone; a failed refresh keeping the rows and showing the words, cleared by the next);
one of the source over the clients built from the same commit (the scan's roots from `AppSettings`, its systems the
shelves', and a synthetic NES image, an iNES header and 16 KiB of zeros, making the places and the NES's row with the
table's name and hue); and the harness test, in which QML reads the rows' `name`, `key`, `kind` and `count` by role and
the model's properties, then names the real clients through `platformLibrary` (two properties before interface 40) and
sees the NES's row arrive from the background refresh with `loading` false and the selection kept; and the kit's QML
rules over the test's scene. Four faults were seeded and caught: the selection kept by index (the selection test),
consoles with no games shown (three view tests), the rows replaced without the model's reset (the harness test: QML
never saw the fourth row) and two roles' names swapped (the harness test). At interface 40 (2026-11-08) the source's
test also finds games.db where Galaxia names it, inside the scratch tree, and all seven pass again.

### 1.3 QML views: layout only

A QML file may hold anchors, layouts, bindings with arithmetic and ternaries, states, transitions, animations whose
durations read tokens, `forceActiveFocus()` in a handler, and handler lines that call one invokable. It may not hold
`function`, `Qt.callLater`, arrays or objects built in QML, string literals shown to the player, `Component.onCompleted`
beyond focus, or `import QtQml.Models`' `ListModel` with rows. **Enforced** by `qmllint` with a project settings file and
by a repository check (`qml-rules`, a DianaSh command in Rust, §8.4) that fails a commit on `function`, on string literals
outside `objectName`, `font.family` and `source`, and on any import outside the allowed list. The spike's `Main.qml` breaks
the rule in three places on purpose, marked in §10.3, to show what the checker must catch.

### 1.4 `EmuSen.Kit`: the QML module that carries what was LunaP's

Two crates and one QML module. `emusen-kit-core` is pure Rust and names no Qt type: the frame's walk (`Walk`: roles,
`move(±1)`, the shoulders stepping sections, A opening a detail, B climbing and declined at the top), the pad grammar and
its repeat tiers, the token tables in both forms and variants with their contrast floors as tests, the fit-audit rules,
the nine navigation cues and the Moonrise bells as sample synthesis, the sky's star field and glow, the body geometry
(`WorldBody`: a planet with craters, bands, a cap, a ring and moonlets; shards; a gem), the Moonrise timeline, the motion
curves. `emusen-kit-qt` is the cxx-qt binding that puts those decisions on Qt items: `Tokens` (a singleton with one
property per token, `form` and `variant` switching the column), `FrameController`, `PadRouter`, `Sky`, `Body` and
`Moonrise` as painted or scene-graph items, `Bells` (renders through the core, plays through Endymion's interface player),
the drawn glyphs, and the `.qml` files of the module: `Frame` with its three forms, `Card`, `Sheet`, `Question`,
`HelpBar`, `MenuPanel`, `MenuRow`, `WholeRow`, `NormalizedCanvas`, `Keyboard` (the docked pad keyboard *[own, because Qt
Virtual Keyboard is GPL-only in the open edition and docks nowhere]*), `Carousel` over `PathView`, `RatingPicker`,
`DatePicker`. Everything Qt Quick Controls already does (buttons, switches, sliders, combo boxes, tabs, scroll bars,
menus, dialogs, tool tips, `TableView`, `TreeView`, `GridView`, `SplitView`) is Qt's, styled by §1.5.

The module's surface is `import EmuSen.Kit 1.0` and its `.qmltypes` (rule Q). It knows nothing of consoles, games or the
runtime: the bodies draw any host's worlds, the frame walks any host's roles. `cargo deny`-style test: the kit's
dependency tree names no runtime or client crate.

Attached properties (`EmuSen.Pad.button`, `EmuSen.Focus.group`, `EmuSen.Fit.exempt`) need a `QML_ATTACHED` C++ shell
each, since cxx-qt does not declare attached types and QML cannot; this is the trivial glue the decisions allow, and
§10.4 records the same kind of glue the spike needed for a brush. Decided 2026-11-08: cxx-qt is the binding; C++ glue is
written where cxx-qt-lib does not reach, each part with its reason.

### 1.5 The style: tokens as a Qt Quick Controls style named `EmuSen`

A directory of control files, each composing its `QtQuick.Templates` type and reading `Tokens`; Basic is the fallback
for any control not written (`QQuickStyle::setFallbackStyle("Basic")`), chosen with `QQuickStyle::setStyle("EmuSen")`
before the engine loads. The tokens are `EmuSen_Interface.md` §5's, written once in `emusen-kit-core` and exported; the
two forms are two columns (desktop pixels, big-screen design units × the inherited scale); dark and light are two more.
The style is the vehicle for the tokens: a plain `Button` paints in them, so "palette reach" is a property of the style's
files rather than a sweep. A theme may override token *values* (§3.4), never a control's file.

### 1.6 The ES-DE engine

Its own component, `emusen-esde`, §4: a Rust loader and scene over plain data, with a QML painter module `EmuSen.Esde`.
It depends on the kit (for `NormalizedCanvas`, `Carousel`, the glyphs) and on the models' curated view, never on the
runtime or the client crates directly; its input is a theme folder and the same models every theme sees.

### 1.7 Input: the pad, the pointer and the keyboard

Qt 6 has no gamepad module [the 2026-11-01 plan, sources there]. The pad is Endymion's, in Rust already
(`emusen_endymion_pads_*`, the router, the player slots, the pad family), read on a Rust thread and queued onto the GUI
thread with `qt_thread().queue`, coalesced per axis. `PadRouter` (kit) applies the dead zone, the repeat tiers measured
in `EmuSen_BigPicture.md` §14 and the player's A/B swap, then offers each press in order to the active sheet or question,
the focused `Frame`'s walk, and, if declined, the window's focus item as a `QKeyEvent` (directions → arrows, A → Return,
B → Escape, Start → Menu, Select → Select), so stock controls and `ListView` answer a pad with no code of their own. Tests
push `PadEvent`s into the same queue a real pad uses. As built on 2026-11-08 (`EmuSen_Kit.md` §7.15): `PadReader` in
`emusen-qt-endymion` (module `EmuSen.Endymion`) reads the pads, a scene joins its `button` signal to `PadRouter.push`,
and the help bar's family is the family Endymion reports for the pad last used (`EMUSEN_ENDYMION_PADS_FAMILY`, since
Endymion owns pads); `BellPlayer` plays the kit's cues through Endymion's interface player.

**The pointer and the keyboard are equals of the pad (§0.2), not a fallback.** Qt Quick delivers both already: stock
controls, `ListView`, `GridView` and `PathView` take clicks, the wheel, flicks and keys with no code of ours. The kit adds
them only where it draws its own items, and once: every element a `Frame` walks to carries a `TapHandler` and a
`HoverHandler`, declared in the kit's `.qml` and never per screen, so a screen that is walkable is clickable.
- Hovering moves the focus ring to the element, as the walk would; a click activates it, the same view-model call as A.
- The wheel scrolls a list or turns a `Carousel` one step per notch; touch is the pointer's taps, and a swipe on a
  `Carousel` is `PathView`'s own flick.
- The mouse's back button and a right click are B; every glyph in the `HelpBar` is a button that sends its press, so a
  pointer has every action the help bar names.
- The keyboard is the map above (arrows, Return, Escape) plus Tab and Shift+Tab through the walk.
- Desktop chrome belongs to the desktop form. The big-screen and card forms have no menu bar and no status bar; a status
  message there is shown as a notice (§7, stage 2).

### 1.8 The game picture

As the 2026-11-01 plan's §3, unchanged in substance: Serenity's Vulkan device becomes the scene graph's device
(`QQuickGraphicsDevice::fromDeviceObjects`, set before the first show), the chain's output image is wrapped without a copy
(`QSGVulkanTexture::fromNative`) by one item, `EmuSen.Kit.GamePicture`, submission order on one queue plus one image
barrier in `beforeRendering` is the whole dependency, and `frameSwapped` is the display clock Serenity's pacer reads. The
threaded render loop and `QSGRendererInterface::Vulkan` are forced on every platform (Mesa's OpenGL DRI3 deadlock on the
handheld is why). Fallbacks: `QQuickRhiItem` with one copy where device adoption is refused; the readback path where
there is no Vulkan. Its first open question, whether Qt's RHI tolerates a foreign submission on its queue between
frames, is the first day of stage 1 (§7).

#### 1.8.1 Day one: the protocol, and its answer **[M 2026-11-08]**

**The answer is yes: Qt's RHI takes a foreign submission on its queue.** As built:
- *The device.* Serenity's device is made with the extensions Qt's scene graph needs, `VK_KHR_surface`, the platform's
  surface (`VK_KHR_wayland_surface` or `VK_KHR_xcb_surface`, by `QGuiApplication::platformName`) and
  `VK_EXT_swapchain_colorspace` on the instance, `VK_KHR_swapchain` on the device (`emusen_serenity_device_new_for`,
  platform interface 42, `EmuSen_RustPlatform.md` §49). The window is created hidden; before its first show a
  `QVulkanInstance` adopts the instance (`setVkInstance`) and `QQuickGraphicsDevice::fromDeviceObjects` names the
  physical device, the device and the graphics queue (family, index 0), all read through `emusen_serenity_device_raw`
  (`EmuSen_Kit.md` §7.16). `QSG_RHI_BACKEND=vulkan` and `QSG_RENDER_LOOP=threaded` are set before the application.
  Qt's log: "Using imported physical device", "Using imported device", "Using queue family index 0 and queue index 0".
- *The picture.* `emusen_serenity_chain_draw` runs every pass for the item's size in pixels and leaves the last pass's own
  image in `SHADER_READ_ONLY_OPTIMAL`; `GamePicture` wraps that `VkImage` with `QSGVulkanTexture::fromNative`. No copy.
- *The order.* Every chain call that submits is made on Qt's render thread, so the queue has one submitter at a time, as
  Vulkan requires; the chain submits at the item's sync, before Qt's frame submits at its end. The dependency is the
  chain's render passes': their existing one from the colour writes to later shader reads, whose second scope takes in
  Qt's later draw, and one added on 2026-11-08 from earlier vertex and fragment reads to the next colour writes, so that
  Qt's read of the last picture finishes before the chain writes it again. That is the "one image barrier" of the
  2026-11-01 plan, carried by the render pass instead of a separate command. A new size waits for the queue before the
  outputs are remade, since Qt may still be reading them.
- *Where it differs from the plan, and why.* The draw is made at the item's sync (`updatePaintNode`, on the render thread
  with the GUI thread held) rather than in `beforeRendering`: the size the picture is drawn for and the image the node
  must name are known there together, and a draw in `beforeRendering` would need a second call to learn the image after
  the node was made. Both points are on the render thread before Qt's submission, so the order is the same. The fallback
  (`QQuickRhiItem` with one copy) was not needed.

**Evidence.** Measured on 2026-11-08 on the desktop (RX 6800, RADV, Mesa 26.2.4, Qt 6.11.2), in a real window of a
compositor of its own (`kwin_wayland --virtual`, 1920 by 1200, Wayland), since the desk's session was locked and KWin sends
a locked session's windows no frame callbacks (a window there drew one frame and stopped). The validation layer was
loaded on Serenity's instance, which Qt adopted, so it saw Qt's calls as well as the chain's, with synchronisation
validation and `VK_LAYER_SYNCVAL_SHADER_ACCESSES_HEURISTIC=1`. A moving test picture (256 by 224, one frame a swap) ran
for **540 s: 32,389 swaps (59.97 a second), 32,267 frames drawn, no finding**. Two faults were seeded. Without the
render pass's dependency from colour writes to shader reads, the layer reported `READ_AFTER_WRITE` at `vkQueueSubmit`,
naming Qt's `vkCmdDrawIndexed` reading the image after the chain's layout transition: the layer does check across the two
submitters. Without the added dependency (Qt's read before the next write) it reported nothing in 5 s, so that direction
rests on the specification, not on the layer; the dependency stays. The first run's one finding, a swapchain colour space
from an extension the instance lacked, is why `VK_EXT_swapchain_colorspace` is asked for.

#### 1.8.2 `SessionModel` and the feed **[M 2026-11-08]**

`emusen-qt-models` gains three parts, each with one job:
- `picture.rs`, **`PictureFeed`**: the feed `GamePicture` names (`EmuSen_Kit.md` §7.16). A frame thread hands over its
  newest frame by swapping buffers with the one waiting, so neither side copies or allocates per frame; at the item's
  sync the render thread takes it, advances it into a chain of the built-in plain filter (`effect.None.nearest`), made on
  the render thread at the first draw and used only there, and draws in place. Each swap's time is counted, and a frame
  thread can wait for a swap or for its frame to be taken. It counts swaps, frames handed, drawn and dropped (replaced
  before a draw), and swaps repeated (no new frame while running).
- `session.rs`, **`Session`**: a game opened through the frontend API with the presenter's device lent
  (`Open.device`, added to the crystal client), run on a frame thread until dropped: each frame's picture copied and
  handed over, the sound drained, paused and resumed by a flag. Plain Rust, no Qt.
- `session_model.rs`, **`SessionModel`** (`EmuSen.Models` 1.0): writable `platformLibrary`, `feed`, `cores` (a cores
  folder in place of the runtime's default), `engine` (an engine by name in place of the settings' choice) and `game`
  (naming one opens it once the library and the feed are named too, in whatever order QML sets them; none closes it);
  read-only `running`, `paused`, `problem` and `firstPictureMs`; invokables `open`, `pause`, `resume` and `close`. The
  frame thread's reports reach it only through `qt_thread().queue`. It reaches the picture the window adopted by the
  feed's number *[own, because QML hands an element no Rust value, and the device must exist before the window does]*.

The program is `Frontend/qt-play`, `emusen-qt-play`: it makes the device, adopts it, names the feed, the library, the
cores folder and the game as the window's initial properties, and with `--seconds` counts and quits. Its window,
`Play.qml`, is a `SessionModel` and a `Kit.GamePicture`, held to the QML rules and qmllint (`tests/qml.rs`).

#### 1.8.3 Pacing by the display **[M 2026-11-08]**

Serenity's display clock (`emusen-serenity::pace`, platform interface 44, `EmuSen_RustPlatform.md` §51) smooths the
display's period from the swaps' times, which `GamePicture` delivers through the feed's `swapped`; the frame thread keeps
one (`platform-client`'s `pace::DisplayClock`, given to `Session::start`). A game whose interval sits within 1 % of the
period is locked to the display: after each hand-over the frame thread waits until a draw has taken the frame, so the
next one is run a whole frame before the sync that shows it. The first version woke at each swap instead and raced the
sync that follows a swap at once in the threaded loop: on a 10 s run of an NES game, after the first 2 s, 3 frames were
dropped and 1 swap repeated; waiting for the take gave 0 and 0. A game off the display's rate runs on its own clock.

Until 2026-11-08 the clock lived in `qt-models`' `session.rs` and took the frame thread's first look, before any swap, as
a sample, so it locked only after about two seconds (`EmuSen_RustPlatform.md` §51); it moved to Serenity with that fixed, and the copy in
`qt-models` is deleted. The rest of the pacer (`FrameScheduler`'s port: speed, frame skip, the presentation trace) follows
it there.

#### 1.8.4 The gate, the tests, and what is owed **[M 2026-11-08]**

**The picture at 60 frames a second for 60 s** (§9.3), the program in a debug build, the platform library in the
`test-fast` profile, in the virtual compositor: Super Mario Bros. (copied read-only to the probe folder) on Moon from the
published cores folder, 62 s, counted after the first 2 s:

| Round | Swaps | Drawn | Dropped | Repeated | Play to the first picture |
|---|---|---|---|---|---|
| 1 | 3,602 | 3,601 | 0 | 0 | 165 ms |
| 2 | 3,597 | 3,573 | 22 | 23 | 158 ms |
| 3 | 3,601 | 3,600 | 0 | 0 | 172 ms |

Round 2 ran while other builds loaded the machine (the run is at CPU weight 20); rounds 1 and 3 are clean. "Play to the
first picture" is from the window's load, which names the game, to the first frame drawn.

**Tests.** `emusen-qt-models`' `session` (the test core opened with the device lent, its first picture reported, the
feed's draw made as the render thread makes it, at two sizes, in `SHADER_READ_ONLY_OPTIMAL`; 30 swaps; paused, nothing is
handed over; resumed, frames come again; closed; no finding from the layer) and `DisplayClock`'s unit test;
`emusen-platform-client`'s `serenity` (an extension not offered refused by name; the raw handles; a chain refuses to
draw before a frame, draws in place, keeps its image at the same size and remakes it at another; no finding); the
kit's layout test of the feed and its registry. Seeded and caught: the draw naming the wrong layout (`session`), a pause
ignored (`session`), and the dependency from writes to reads removed (the layer, §1.8.1).

**Owed**, each with its reason:
- *The real desk, and the C# figures from the same script.* The desk's session was locked through the work; both need
  the tester there. The C# front end counts no `frameSwapped`; which of its own counters stands for the same count is
  settled when its figures are taken, and they are written next to these.
- *Play to the first picture for a hosted core.* Done 2026-11-08, §9.3.3: Nestopia (the NES's hosted core) through
  `cores get` in a sandbox laid out as a publish, equal to Mistress within the noise.
- *The pad driving the game.* Done 2026-11-08, §1.8.5.
- *A device core's picture.* Done 2026-11-08, §1.8.7.
- *The sound* (done 2026-11-08, §1.8.6); *the picture's content* (done 2026-11-08 by a window grab, §1.8.8); and the chain dropped on the render thread when a session closes (it is now
  dropped with the feed, after the window).

#### 1.8.5 The pad drives the game **[M 2026-11-08]**

One owner of the pads (Endymion) and one reader (`PadReader`'s thread, `EmuSen_Kit.md` §7.15): each tick reads the pads
for the interface and for the running game, the second through each player's bindings (`EmuSen_Kit.md` §7.17). The
game's read crosses to the frame thread by a hand-over (`GamePads`) and is routed there:
- `input.rs`, **`GameInput`** (`emusen-qt-models`): owned by the session's frame thread. At open it reads the session's
  console, ports (one where a core declares none, as Mistress reads them) and axes, asks them of the pads and resets
  Endymion's router (`emusen_endymion_router_*`, now wrapped by the platform client); before each frame it takes a newer
  read, polls the router and tells the core each change in the router's order (`input_connected`, `input_button`,
  `input_axis`), applied at once since the frame thread calls them between frames. The router is the C# front end's
  (`PortRouter`), so both front ends route by one rule.
- `SessionModel.pads` names the reader's hand-over (`PadReader.gamePads`); a session with none named hears no pad.
- While the game runs and is not paused, `PadRouter.chordOnly` keeps the walk from game input and the menu chord reaches
  the program. emusen-qt opens the Pause Card on it (§7.1, step 7); `Play.qml`, emusen-qt-play's window of one game,
  has no card and pauses and resumes on it.

The new dependency, `emusen-qt-models` on `emusen-qt-endymion`, is stated: the session must reach the one reader's
hand-over, and a third crate for one struct would add a boundary with nothing behind it. Owed: the seats the player
arranges and keeps, rumble to the players' pads, the keyboard as a player, a core's ports changing while it runs
(Mistress's `Resize`). The C# front end now asks interface 43's `_choose` (decided 2026-11-08,
`EmuSen_RustPlatform.md` §50), so the binding rule has one owner.

**Proof in the window, measured 2026-11-08** (`qt-play/tests/pads.rs`): `Play.qml` loaded by the harness in a compositor
of the test's own (`emusen_qt_test::compositor`, `kwin_wayland --virtual`), Qt adopting Serenity's device on the RX 6800
before the window is shown (`Harness::load_prepared`), the test core installed in the player's cores folder and approved
as a player would, and an SDL virtual pad named to the reader alone. Before a game the router gives everything; while the
game runs it gives the chord alone; A (East, Endymion's default) held half a second is heard on port 0, read afterwards
from the battery file the test core counts it in; the guide button pauses the game and gives the walk the pad, and again
resumes it; the reader and session say no failure and the validation layer's findings are none. 1.5 s. Seeded and
caught: the window not naming the reader's hand-over (no battery file: the game heard nothing), `chordOnly` never set
(the router gave everything during the game), and the chord not reaching the session (no pause in 10 s).

**The bars are black** (the tester, 2026-11-08: a game never shows white around its picture). On the desk the 4:3
picture stood between two white bars: `Play.qml`'s window set no colour and Qt's default is white, and `emusen-qt`'s
window paints `voidFill`, which the light variant makes light. The bars belong to the game, not the theme, so `Game.qml`
fills its whole box with plain black (`#000000`, no token) behind the picture, for every host and every variant, and
`Play.qml`'s window is black for the moment before the first frame. **Held to:** qt-play's `device.rs`, once the core's
frame is read back, samples the window beside the picture (a fifth of the way from each edge, at the picture's top,
middle and foot) in the dark and the light variant and requires black.

#### 1.8.6 The game's sound **[M 2026-11-08]**

The sound takes the C# front end's path through the platform library, so it has one owner: Endymion's game player
(`emusen_endymion_audio_*`, `AudioPlayer`'s, `EmuSen_Audio_Sync.md` §7) on SDL's playback device, steered by a rate
control of its own (`emusen_endymion_rate_*`) that absorbs the drift between the game's clock and the device's by
resampling. The parts, each with one job:
- `audio.rs`, **`GameSound`** (`emusen-platform-client`): the player and its rate control, made together and freed
  together; `submit` (interleaved stereo at the game's rate, a new rate reopening the device), `reset`, the queued and
  taken frames, the device's rate and the volume. `SoundSettings` carries audio.json's three values and the buffer.
- The session's frame thread (`session.rs`) drains each frame's sound and submits it, where it was drained and dropped
  (§1.8.3); a game that pauses starts its rate control over, as Mistress's `DrainSound` does. `Session::sounded` reports
  the frames the player took in, by the rate control's own count. `GameIo` carries the pads and the player into the
  session together.
- `SessionModel` lends SDL to the library and makes the player at open; where it cannot be made the game plays without
  sound and `problem` says so. `sounding` says whether the open game's sound reaches a player.

**Owed, stated:** audio.json is not yet read, so the player uses `AudioPlayer`'s own defaults, which are audio.json's
(32 kHz, 256 ms, a deviation of 0.005, 4,096 frames); the output device, the volume and mute of `appsettings.json` and
audio.json's `MasterVolume`, `Muted` and `AudioEnabled` wait for the settings screens. The C# clamps audio.json's values
in `AudioConfig.Bounded`, which has no Rust owner yet; reading the file in Qt needs that rule moved into Galaxia's model
first, so that the bounds are not copied.

**Proof, measured 2026-11-08** (SDL's dummy driver, nothing reaches the speakers): `platform-client`'s `sound` (the
player opens at 32 kHz, reopens at a game's 48 kHz and again at 32 kHz, holds a tone's 800 frames, and its rate control
took exactly those); `qt-models`' `session` on the RX 6800 (every handed frame's sound, a sixtieth of the test core's
32 kHz, reached the player, and a paused game played none); `qt-play`'s `pads` (the window's game is `sounding`).
Seeded and caught: the submit skipped (no frames taken, `session`), the device opened at a fixed rate (`sound`), and the
window's player made and dropped (`pads`).

#### 1.8.7 Pictures on the device: a machine's lend and its ring **[M 2026-11-08]**

Every kind of picture the runtime offers now crosses the feed (`picture.rs`), as Mistress's `OfferPicture` offers them
(`EmuSen_Backend.md` §14.12, §33): a frame's `Source` is its pixels (read back by `picture_copy`), the newest picture the
machine took through its lend (`picture_on_device`, `picture_lend`), or a slot of its ring (`picture_in_ring`,
`picture_ring`, the slot and value of the frame). The render thread advances the chain from where the picture lies
(`Chain::advance_lent`, `Chain::advance_ring`, new in `platform-client`), and a picture that will not advance is never
tried again. The session opens with `OPEN_RING` wherever the window has a device.

**The rules that keep a lend and a ring alive while the render thread may read them:**
1. A picture on the device is drawn before the machine runs again: the frame thread waits (stopping when asked) until a
   draw has taken it. A core whose ring slots are all held therefore never waits inside `frame`, and no lend or ring is
   remade (a reset, a state of another size, a ring's maxima growing) under a picture still to be drawn. The cost is
   that a window that does not draw (hidden, minimised) holds such a game still; Mistress instead gives each slot back
   through a one-pixel chain (`SlangRingRelease`) and shows the picture read back, which in Qt would submit from the
   frame thread to the queue Qt's render thread uses, so it is not taken.
2. Before the machine is freed the frame thread withdraws the frame still waiting (`Handoff::withdraw`) and waits out a
   draw under way, which holds the feed's `drawing` lock from taking a frame to its end.

**A named difference from Mistress, accepted for stage 1 (decided 2026-11-08).** While the window does not draw (hidden
or minimised), a game drawing on the device is held still: rule 1 keeps it from running past a picture nobody has drawn.
Mistress keeps such a game running through `SlangRingRelease`; the tester sees this difference listed here until the
Pause Card and the window's own life settle what a hidden window should do.

**The player's cores beside a named installation.** The runtime scans no player's folder when a host names its `cores`
and leaves `player_cores` unset (`emusen_crystal.h`'s config; `Runtime::new`), which suits a test of the installation
alone but would hide every core the player installed from a program that names its installation. `SessionModel` therefore
passes Galaxia's cores folder (`EMUSEN_GALAXIA_DIR_CORES`, the runtime's own default) whenever it names `cores`.

**Proof, measured 2026-11-08** in a real window (`qt-play`'s `device` and `ring`): the program's Play.qml in a
`kwin_wayland --virtual` of the test's own, Qt adopting a validated Serenity device on the RX 6800, the game opened
through `SessionModel` as the player opens one.
- *The lend:* emusen-native's `device_test_core` (built from `emusen-libretro-host`'s examples by `client-fixture`'s
  `cores::device_core`), installed as a player's core and approved, paints its picture on the lent device. Every frame
  drawn (60 and more) came from the lend (`Counts::shown_lent`), none from a ring.
- *The ring:* the libretro test core, laid out as an installed libretro entry (`cores::libretro_entry`: the proxy, the
  libretro host, the core and its `.info` under the player's `Cores/libretro/testcore`, with an installation whose
  `cores.manifest` vouches for the proxy and the host, and `EMUSEN_CORE_RUNNER`), set to draw with Vulkan (`tc_hw`).
  Every frame drawn came from a slot of the ring (`Counts::shown_ring`).
- *Rule 1:* with the window hidden, nothing more is handed over, and the game closes in about 1.3 s (the runner
  stopping); with the frame thread's wait for the draw removed, the close took 9.8 s, its frame waiting on a ring slot
  nobody gave back. The test bounds the close at 5 s.
- The layer found nothing in either run, apart from the window grab's own (§1.8.8).

Seeded and caught: the ring's slot never advanced into the chain (`ring`: no picture within ten seconds); the wait for
a device picture's draw removed (`ring`: the hidden window's close, 9.8 s); `SessionModel` naming its installation
without the player's folder (`ring`: no core for `.tcg`).

#### 1.8.8 The picture's content, read back **[M 2026-11-08]**

"It swapped" does not show the right picture, so both tests read back what the window shows and compare it with what the
core made. The device test core and the libretro test core paint the same picture, a function of the frame's serial
(`client-fixture`'s `cores::painted_pixel`: red the column plus the serial, green twice the row plus forty a life, blue
the column over eight). The feed records the core's serial of the frame it drew last (`Counts::shown_serial`). The test
pauses the game by the chord, so the frame drawn stays the frame shown, grabs the window (`QQuickWindow::grabWindow`,
the harness's `colours_at`) and compares 48 pixels, spread over the picture and mapped to the window at the size the
compositor gave it, with the core's picture of that serial: none may differ by more than one level. Measured
2026-11-08: none differs, for the lend (frame 67 on the first run) and for the ring.

The first run found a mismatch in green only, from row 46 down, growing with the row: the compositor had given the
window less height than asked for (its decoration), so the mapping now reads the window's own size.

**The grab's findings, a named exception.** `grabWindow` under the threaded render loop gives two findings on Qt's own
swapchain: a render pass after the present (`WRITE_AFTER_PRESENT`) and a barrier after the acquire (`WRITE_AFTER_READ`).
The same test without the grab gives none. The tests therefore require no finding up to the grab and after it, and around
the grab allow only those two (`assert_qt_grab_only`), which name `vkQueuePresentKHR` and `vkAcquireNextImageKHR`.

### 1.9 Homes for the front-end logic still in C#

The survey of the remaining C# (2026-11-07) lists front-end logic that needs a Rust owner. Each is given one layer here,
by the one-owner rule, and none goes into the view models unless it is a view model's job.

| Logic (C# lines) | Home | Why there |
|---|---|---|
| The ES-DE theme engine: loader, scene, gamelists, collections, folders (about 4,000, plus 3,900 of tests) | `emusen-esde` (§4.2), a crate of its own, with `EmuSen.Esde` as its painter | one job, no Qt in the loader or scene, testable alone |
| The theme store: `themes.db`, GitHub and GitLab downloads, the browser's records, attribution (about 1,150) | `emusen-themes`, a platform crate over `emusen-net`, the zip reader and SQLite, exposed as the DianaSh command `theme` (`list`, `install`, `update`, `remove`, `check`), which the Themes tab fronts; it serves both theme kinds | the 2026-10-26 rule: every tool a command; one store for both kinds |
| The four binding files: keybindings, hotkeybindings, gamepadbindings, gamepadprofiles (about 800) | Endymion, through the platform library (`emusen_endymion_*`) and Galaxia's models; the front end's `DeviceModel` and `SettingsModel` read and write them through `emusen-platform-client` | Endymion is the one owner of pads; both front ends share the files, so this is stage 0. Done 2026-11-08 at interface 40 (`EmuSen_Input.md` §5.2) |
| Frame pacing: `FrameScheduler`, `DisplayClock`, `PictureSlots`, `PresentationTrace`, `VblankCounter`, `FrameHandOff`, `FramePacer` (about 650) | Serenity (`emusen-serenity::pace`), fed the window's `frameSwapped` stamps by `GamePicture` and read by the emulation thread's frame loop | `EmuSen_Surfaces.md` §6.2 already makes Serenity the owner of the display clock and the pacer; the view models only relay stamps. `DisplayClock` moved 2026-11-08 (interface 44, §1.8.3) |
| The build host: `EmuSen.csproj`, `RustCores.props`, `RustCoresPublish.targets`, `CheckCoreSidecars.proj`, `Platform.targets`, the publish layouts (about 760 lines of MSBuild) | a cargo workspace with an `xtask` crate: `cargo xtask check-sidecars`, `publish`, `appdir`, `qml-rules`, `qmltypes-diff`, each a plain Rust subcommand calling Qt's libexec tools (`qmlimportscanner`, `qmlcachegen`, `qmltyperegistrar`) where needed; MSBuild stays only for the C# until it retires | cxx-qt builds under cargo already, so one toolchain builds everything; CMake would add a second build system for no C++ host, since the front end has none **[A]**; a CMake host becomes worth it only if a C++ consumer of the kit appears (§11 Q8 says none is planned) |

The rest of that survey (the headers' folder, the memcard command, records.db, Pharaoh's probe as a Rust CLI, the shell
program) is not front-end logic and is placed by `EmuSen_Backend.md` §40's successors.

---

## 2. The house rules, layer by layer

The code-design rules (`CONTRIBUTING.md`'s code-design section; restated 2026-11-07) applied to each layer: its one job,
its surface and the surface's rule, what crosses, what never crosses, and how a reader audits it alone.

| Layer | One job | Surface, version, rule | Crosses | Never crosses | Audited by |
|---|---|---|---|---|---|
| Runtime and platform library | run consoles; keep the player's files and devices | `emusen_crystal.h` 1.x (A); `emusen_platform.h` exact (B) | C calls; JSON once; structs and buffers per frame | any Rust type; any callback | the headers' baselines in CI; `CrystalParityTests` goldens |
| `emusen-crystal-client`, `emusen-platform-client` | make the two headers safe Rust | their `pub` API, `cargo public-api` baseline (R until 1.0, then C-like semver, §5) | the headers' functions and JSON as typed records | Qt; any runtime crate as a path dependency | the generated binding diffed; `cargo deny` on the dependency tree |
| `emusen-qt-models` | Qt models and properties over the clients | `EmuSen.Models` 1.0 `.qmltypes` (Q, internal until 1.0) | properties, roles, invokables, signals | a C header; a kit drawing; a QML file | one bridge per screen; a view-model test with a fake client |
| `EmuSen.Kit` (`emusen-kit-core`, `emusen-kit-qt`) | the grammar, tokens, frame, drawn bodies, sounds, audit | `import EmuSen.Kit 1.0` + `.qmltypes` (Q) | plain data in (roles, tokens, bodies), items and attached properties out | consoles, games, the runtime, the client crates | `emusen-kit-core` tests without Qt; the kit's `cargo deny` |
| The `EmuSen` style | paint stock controls in the tokens | a Qt Quick Controls style directory; the tokens' names and types (Q, with the kit) | token reads | app or theme logic | a `grabImage` sweep per control file |
| QML views | arrange | none outward; held to §1.3's rule | bindings to models; invokables; focus | functions, literals, data | `qml-rules`; InterfaceSpec recordings |
| `EmuSen.Theme` | what a theme may see and do | `import EmuSen.Theme 1.0` + `theme.json` (Q and T, §5) | curated models, roles, actions, tokens, cues | `EmuSen.Models` itself; files outside the theme; the network | the import allow-list; the sandboxed engine (§3.5) |
| `emusen-esde` | draw an ES-DE theme from its XML | a crate API (R) and `EmuSen.Esde` (Q); `esde-check` (a DianaSh command) | a theme folder in; an `ElementModel` out | runtime calls; QML logic | the loader's fixtures; the compatibility corpus (§4.4) |
| Themes | look | `theme.json` (T) and QML against `EmuSen.Theme` | roles, actions, tokens, cues | everything else | `theme-check` (§3.7) |

What the rules forbid, named: no reflection into another component (QML reaches objects only through registered
properties and invokables; Rust reaches Qt only through cxx-qt's generated code); no shared mutable statics as wires (the
one `OnceLock` in the spike is a start-time stamp set once in `main`, §10.3, and the design has none: every singleton is a
QML singleton owned by the engine); no new cross-component dependency unsaid (the dependency list of each crate is in its
`Cargo.toml` and in this table; `cargo deny` holds the tree to it); one owner per fact (tokens in `emusen-kit-core`; the
systems table in the platform library; a console's hue and body in the systems table's optional override, §6.3; the pad's
button bits in Endymion; a core's names in its info); and features by form (§11's questions each say whether an answer is
an extension, a module or a component).

---

## 3. The theme model

### 3.1 Two kinds of theme, one set of models

An **EmuSen theme** is a folder of QML and data against the `EmuSen.Theme` API. The reference theme, "EmuSen", is one,
shipped in the binary's resources and rendered exactly to the approved mockups (Choose a console as the courts draft,
Home, the grid: `EmuSen_Interface.md` §24, the canvases of 1280 × 800 design units). An **ES-DE theme** is a folder of
XML in ES-DE's format, drawn by the engine of §4 from the same models. The player chooses one or the other in Settings ▸
Display ▸ Theme, as today; "EmuSen" is the first entry.

What a theme owns is `EmuSen_Interface.md` §7's rule, carried over: the library's screens (Home, Choose a console, a
console's games or the ES-DE system and gamelist views) and their sounds. EmuSen owns everything else: the Home card, the
Pause Card, the List card, the game's page, tools, Settings, questions and the launch screen; **a theme never styles a
card** (§0.7 there) and no theme colour is read into one. A card opens over the theme under the Scrim, in the tokens.

### 3.2 What a theme sees: the `EmuSen.Theme` API **[M 2026-11-08]**

`import EmuSen.Theme 1.0` gives a theme exactly these, each a QML singleton or type, each a view over `emusen-qt-models`
with the fields a theme may read and nothing more. The table is the module as built (`Frontend/theme-qt`, the crate
`emusen-theme-qt`); its registry is committed as `Testing/qmltypes/EmuSen.Theme-1.0.*` and held to this table by hand in
`tests/surface.rs`. The amendments of 2026-11-08 are listed in `Frontend/theme-qt/CHANGELOG-theme-api.md`.

| Name | Kind | What it holds |
|---|---|---|
| `Library` | singleton | `consoles` (`key`, `name`, `displayName`, `maker`, `year`, `count`, `hue`, `body` = the world the kit's `Body` draws, `engineName`, `engineKind` = `emusen`/`libretro`, `engineMark`); `places`, Home's seven destinations in Home's order (`key` = `games`, `scraper`, `packages`, `library`, `cores`, `settings`, `power`; `name`; `kind` = `library`/`tool`/`settings`/`power`; `line`; `attention`; `count`, Games' consoles); `shelves`, All, Favourites, Recent (`key`, `name`, `line`, `count`); `recent`, `favourites` (game rows); `collections` (`key`, `name`, `count`); `gamesOf(key)` for a console's or a shelf's key and `search(words)`, game rows: `key` (the game's file), `title`, `cover`, `coverShape` (the console's width over height), `favourite`, `lastPlayed` (s since 1970, 0 never), `played`, `consoleId`, `consoleHue`, `players`, `year`, `genre`, `rating`, `description`, `video`, `hero`, `lastPlayedWords` |
| `Session` | singleton | `hasGame`, `title`, `consoleId`, `pausedSince`, `played` (seconds), `frame` (the item the game's picture is drawn in, for a `ShaderEffectSource`) |
| `Device` | singleton | `controllers`, `family` (the glyphs' family, as `Frame.padFamily` takes it), `clock`, `battery` (-1 none), `charging`, `network`, `bigScreen`, `form` |
| `Actions` | singleton | `play(key)`, `openPage(key)`, `favourite(key)`, `random(console)`, `openConsole(key)`, `openPlace(key)`, `home()`, `back()`, `stepSection(±1)`, `openHomeCard()`, `openListCard()`, `search()`; each returns whether it was taken |
| `Cues` | singleton | `scroll()`, `systembrowse()`, `select()`, `back()`, `favorite()`, `launch()`, `quicksysselect()`: the seven names of `EmuSen_Interface.md` §23, rung by the host through Endymion at the player's volume |
| `Words` | singleton | `title`, `gamesHeading`, `chooseAConsole`, `gamesPath`, `continuePlaying`, `recent`, `between`, `search`, `emusenTag`, `libretroTag`; `games(n)`, `ago(t)`, `pausedFacts(console, pausedSince)`, `facts(maker, year, count, runsOn)`, `runsOn(engineName, engineKind)`, `statusLine(controllers, clock)`, `splashCaption(n)`: the curated face of `words.rs` |
| `Help` | singleton | `entries(kind)` for `Help.Continue`, `Game`, `Console`, `Place`, `Section`, `Search`: a list of `button`, `label`, `pair` with `actionLines`, which `Frame.help` takes; the curated face of `help.rs` |
| `Tokens` | singleton | every token of the kit (`EmuSen_Interface.md` §5) in the current form and variant, read-only; `Tokens.Dark`/`Light`, `Tokens.Desktop`/`BigScreen`/`Card` to compare `variant` and `form` with; overridden only through `theme.json` (§3.4) |
| `Frame`, `HelpBar`, `Walkable`, `Body`, `Sky`, `Scene`, `Carousel`, `NormalizedCanvas`, `Moonrise` | types (the kit's, by name) | the pieces a theme places; `Frame` carries the walk, so a theme that places roles gets the pad for free; every item the pad and the pointer reach is a `Walkable` |
| `ThemeScreen` | type | the root of a theme's screen: `role` (`ThemeScreen.Home`, `Consoles`, `Games`, `Splash`), `focusRole` (`ThemeScreen.Navigation`, `Content`, `Detail`, `Actions`, `Search`), `focusItems`, and for the splash `length` (ms), `still` and `finished()` |

A theme provides `screens/Home.qml`, `screens/Consoles.qml` and `screens/Games.qml`, each a `ThemeScreen`, and may provide
`screens/Splash.qml`; a theme that omits one gets the reference theme's (Moonrise for the splash). Everything on them is
bound to the singletons above; the pad reaches them through the `Frame` they place or, where a theme arranges freely on
a `NormalizedCanvas`, through `ThemeScreen.focusItems` and `KeyNavigation`. The views are read-only; every change goes through
`Actions`, every sound through `Cues`.

**As built.** One job per part, each in its own file of `Frontend/theme-qt/src`:
- `view.rs`, plain Rust and no Qt: each list's roles, written once, and its rows curated from the library's view
  (`LibraryView`, §1.2.1) a cell for each role, in the role's order. The games' rows come from `library::games_of_at`
  and a search from `listing::shown`, the grid's own rules; nothing about a game or a console is decided here.
- `rows.rs`, **`Rows`**: every list as one read-only `QAbstractListModel`, its roles fixed when its singleton makes it,
  its rows replaced whole under the model's reset, `count` notified. A theme cannot make one (uncreatable) or write one.
- `library.rs`, `session.rs`, `device.rs`, `words.rs`, `help.rs`: the singletons. Each has a Rust-only feed (`show`),
  no invokable that changes it, and no property a theme may write: a writable path to the platform library, as
  `LibraryModel` has, would let a theme load any library, so the host hands the views their data instead.
- `actions.rs`, `cues.rs`: each invokable becomes a value (`Action`, `Cue`) handed to the host's sink or player, a
  Rust trait object; with none set, nothing is taken and nothing rings.
- `qml/`: `ThemeScreen.qml`, `Tokens.qml` and one file a kit type, `Kit.Frame {}` and its kind. The qmldir names these and
  imports nothing, so `import EmuSen.Theme` brings no other kit type: the kit's `Card`, `Question`, `PadRouter` and its
  writable `Tokens` stay out of a theme's reach. `Tokens.qml` reads each kit token into a read-only property, and
  `ThemeScreen`'s walk roles and the tokens' `Variant` and `Form` repeat the kit's enumerations; the surface test reads the
  kit's registry and fails when either side changes, so the kit stays their one owner.
- `glue.rs` and `cpp/theme_glue.cpp` *[own, because cxx-qt makes no QObject from Rust and cxx-qt-lib has no
  `singletonInstance`]*: a list made as its singleton's child and held in C++ ownership, so no engine's collector takes a
  list a call handed to QML; and a singleton found in the engine an object lives in. The host reaches the singletons
  through `emusen_theme_qt::host`, given the theme's engine or any object of it.

The C++ class names carry `Theme` (`ThemeLibrary`, `ThemeWords`, ...), registered under the API's names, because
`EmuSen.Models` and the kit already define `Words` and `Cues` in the same program.

**What the views do not hold yet.** The own look's library view reads no scraper facts and no collections (the ES-DE
view does), so `players`, `year`, `genre`, `rating`, `description` and `video` are empty or 0 and `collections` is
empty; the roles stand so a theme written now reads them when the view does. `hero` falls back to the cover. Not yet in
the module: `CoverTile` (moved into the kit with the reference theme's package), `VideoTile` (stage 4) and `Bells` with
`ladder(count)` and `strike(i)`, which the kit's `Bells` does not offer; each is added by name when the kit has it.

**What the host does.** The host (§3.5) makes the theme's engine, then feeds `Library.show(view, homeLines)` after each
refresh of the library model, `Session.show` and `show_frame` while a game is suspended, `Device.show` on each status
change and clock tick, and sets the `Actions` sink and the `Cues` player once. The module holds no static; a singleton
lives and dies with its engine, so switching themes drops them with it.

**Proof, measured 2026-11-08.** `tests/view.rs`: each list's roles against a hand-written list, and every list over the
real clients built from this commit (`emusen-client-fixture`, one synthetic NES game): the NES's row with the table's
key and name and its engine's kind; Home's seven in order with their kinds and the scraper's line asking for the player;
the shelves' counts; `gamesOf` for a console and a shelf, the cover's shape the console's; an unknown key and a search
with no match empty. `tests/qml.rs`, one harness test: a scene importing `EmuSen.Theme` reads every list through
`required property` delegates of every role (a missing role is a warning, and the scene must stay quiet), `gamesOf`,
`Words`, `Device` and `Session` as fed, `Tokens.accent` equal to the kit's, `ThemeScreen`'s enums, and places each kit type
by its name; `Help.entries` is non-empty for every kind; `Actions` take nothing with no sink and then say which the host
took; `Cues` are silent with no player. `tests/surface.rs`: the committed registry is the built one; each type's
properties and methods equal a hand-written list and nothing else is exported; the qmldir lists the kit's types by name
and imports nothing; `Tokens` mirrors every kit token; `qml-rules` and `qmllint` pass the module and its scene; the
crate's dependencies name no client and no runtime crate, and no source file reaches one (the tree reaches the clients
only through `emusen-qt-models`, which it curates). Three faults were seeded and caught: a game's title and cover cells
swapped (`view.rs`'s test and the harness test), a declined action reported as taken (the harness test), and a token
left out of `Tokens.qml` (the surface test).

**The name `ThemeScreen`.** §3.2 named the type `Screen`, but QtQuick has an attached type of that name (`Screen.width`,
`Screen.pixelDensity`), and which one a theme got depended on the order of its imports. Decided 2026-11-08: the type is
`ThemeScreen`, and its enumerations are read as `ThemeScreen.Home` and the others.

### 3.3 Versioning

The API is the module's version, `EmuSen.Theme 1.0`. A minor adds singletons, roles, actions, cues or tokens and never
removes or renames one; a role or action no longer meant is kept and marked deprecated in the `.qmltypes` for one minor,
then removed at a major. A theme states the API it was written for in `theme.json` (`"api": "1.0"`); a theme written for
a newer major than the build has is refused with words naming both; one written for an older minor loads, since minors
are additive. Before 1.0 (§5.1) the same rules hold, with breaking changes allowed and listed in `CHANGELOG-theme-api.md`.

### 3.4 The package

```
MyTheme/
    theme.json         name, author, licence, api, variants (dark/light), fontSizes, the screens it provides,
                       token overrides per variant, cue files, preview images
    screens/Home.qml   screens/Consoles.qml   screens/Games.qml
    parts/*.qml        the theme's own components (QML against the allowed imports only)
    assets/            pictures, fonts (OFL or the author's), sounds
```

Token overrides are values only (`"tokens": {"dark": {"accent": "#4C9AE8", "radius": 8}}`), checked against the token's
type and the contrast floors by `theme-check` (§3.7); a theme cannot add a token or change what a token is for. The
reference theme is the same package shape, compiled into the resources, and its `theme.json` is the schema's worked
example.

### 3.5 Loading and the sandbox

Each theme runs in a `QQmlEngine` of its own, created by Rust with: an import path list holding only Qt's modules on an
allow-list (`QtQuick`, `QtQuick.Layouts`, `QtQuick.Effects`, `QtQuick.Shapes`, `QtQuick.Controls` in the EmuSen style) and
`EmuSen.Theme`, so `import EmuSen.Models`, `QtQml.XmlListModel`, `Qt.labs.*`, `QtMultimedia` (video is the kit's
`VideoTile`, §11 Q5) and `QtNetwork`-backed types do not resolve; a URL interceptor that resolves `file:` URLs inside the
theme's folder and the cover store only and refuses `http:` and `qrc:` outside the module's own; a network access manager
factory whose manager refuses every request, so `XMLHttpRequest` and remote `Image.source` fail with a diagnostic; the
engine's `offlineStoragePath` unset and `LocalStorage` absent from the allow-list. QML's JavaScript has no file, process or
socket API, so what remains is the engine's own surface. **What the sandbox does not do:** it is not a security boundary
against a hostile theme in the sense of a process sandbox (a theme can spin the CPU or allocate until the process dies);
the theme list EmuSen offers is reviewed, as ES-DE's is, and a theme installed by hand is the player's choice. A theme that
fails to load is reported in the Themes tab with the engine's warnings and the reference theme is shown. **[A]**

Switching themes at run time destroys the engine and creates another; the spike shows the smaller case, one engine and
a `Loader` whose `source` moves between theme files (§10.3).

#### 3.5.1 The host as built **[M 2026-11-08]**

**The crate.** `emusen-theme-host` (`Frontend/theme-host`), the QML module `EmuSen.ThemeHost 1.0` (the program's; a theme's engine
never resolves it). Four parts, one job each: `policy.rs`, the sandbox's rules as plain data with no Qt; `package.rs`, `theme.json`
read through `emusen-theme-manifest` (the schema's one owner, §3.7) into what the host loads, the name and the screens provided;
`sandbox.rs`, the bridge to the C++ shells; `host.rs`, the `ThemeHost` type. Its dependencies are cxx, cxx-qt, cxx-qt-lib,
`emusen-qml-rules` (for `THEME_IMPORTS` and its theme profile, the allow-list's one owner) and the manifest crate; no client, runtime
or model crate (`tests/lint.rs` holds the list).

**`ThemeHost`.** `folder` (a path, a `file:` URL or a `qrc:` folder; empty for none) and `coverStores` are written; `name`, `ready`,
`failed`, `warnings` (one line per finding), `generation` and `liveEngines` are read. Writing `folder` destroys the engine before,
drops `ready` (so every slot lets its screen go), reads `theme.json`, and makes the new engine; a manifest that breaks the schema or
names a newer API is refused with the manifest's words (`theme.json:4 api written for EmuSen.Theme 2.0; this build has 1.0`) and
no engine is made. `provides(role)` says whether the package lists the role (`ThemeHost.Home`, `Consoles`, `Games`, `Splash`, an
enum, so no QML writes a role's name); `place(role, slot)` creates the role's screen in the theme's engine as a child of the slot,
filling it, and returns false, setting `failed`, when it cannot.

**The engine.** One `QQmlEngine` per theme, created and owned by the sandbox: its import paths are Qt's QML folder, `qrc:/qt/qml`
and `qrc:/qt-project.org/imports` only (no application folder, no `QML_IMPORT_PATH`), no plugin path, the URL interceptor and the
network factory installed, `offlineStoragePath` set empty, and its `warnings` signal gathered into the host's log. The sandbox
deletes the screens it made, then the engine, then the shells the engine was handed, in that order; `liveEngines` counts the
engines still alive through a `QPointer` each, so a leak is a number a test reads.

**What a theme's engine may reach**, judged by `Policy::judge` for every URL the engine resolves (QML, script, qmldir, and every
`url` property after resolution):

1. A theme's own QML or script file has its imports held to `THEME_IMPORTS` (`QtQuick`, `QtQuick.Layouts`, `QtQuick.Effects`,
   `QtQuick.Shapes`, `QtQuick.Controls`, `EmuSen.Theme`) and to relative folders, through qml-rules' theme profile; a script's
   `.import` likewise. A file that names another module is refused before it compiles, with the file, the line and the list:
   `screens/Home.qml:2 imports EmuSen.Models, which a theme may not import (allowed: …)`.
2. Some modules never resolve, by any route: the program's own (`EmuSen.Models`, `.App`, `.Play`, `.Endymion`, `.Esde`,
   `.ThemeHost`) and those that reach files, processes, media or the network (`QtMultimedia`, `QtNetwork`, `QtCore`,
   `QtQml.XmlListModel`, `QtQuick.LocalStorage`, `QtQuick.Dialogs`, `QtWebEngine`, `Qt.labs`), refused at their `qmldir` under
   every import root. This is what stops an inline component, `Qt.createQmlObject("import EmuSen.Models; …")`, which no file
   check sees: measured on 2026-11-08 with the program's models registered in the process, the module's qmldir is looked for
   under each of the three roots, each lookup is refused, and the object is not made. The allow-list of rule 1 is the narrower
   rule; rule 2 exists because a module allowed by rule 1 (QtQuick, the kit through `EmuSen.Theme`) imports modules of its own,
   which a theme must still be able to resolve transitively.
3. `file:` URLs inside the theme's folder (symbolic links resolved, `..` taken out), pictures inside a cover store, and Qt's own
   module files in its QML folder; nothing else on disk (`file:///etc/hostname is outside the theme's folder and the cover store`).
4. `qrc:` URLs of Qt's own modules, `EmuSen.Theme`, the kit it re-exports, the EmuSen style's controls and the kit's fonts, and the
   theme's folder when it is a resource (the reference package, R); never another resource of the program (`EmuSen.App`'s screens,
   the ES-DE painter's shader). A `qmldir` is judged by the module it is looked for: Qt's probes for `import EmuSen.Theme 1.0`
   (`EmuSen/Theme.1.0/qmldir`, `EmuSen.1.0/Theme/qmldir`, `EmuSen/Theme.1/qmldir`, `EmuSen.1/Theme/qmldir`, four a screen, measured
   on 2026-11-08) pass and are simply not found, nothing logged; a probe for a module not on the list is still refused and logged.
5. `data:` and `image:` (only the providers registered on the theme's engine, none yet), and an empty URL (an unset `source`); no
   other scheme, so no network:
   `http://example.invalid/cover.png: a theme reaches no network …`.

A refused URL is rewritten into `/emusen-refused/<scheme>/<host><path>`, a folder that never exists, so the engine reports the
file missing synchronously (`QML Image: Cannot open: file:///emusen-refused/http/example.invalid/cover.png`) and the host's log has
the reason. A first version rewrote to an unknown scheme left for the refusing network to decline; that sent qmldir lookups through
the network asynchronously, and a synchronous `Qt.createQmlObject` waiting on them never returned (measured on 2026-11-08: the test
hung past 180 s). The network factory stays as the second wall: every manager it makes refuses every request, logged, so a URL
that reached it by a route the interceptor does not see still fails with a diagnostic.

**The two C++ shells, and why.** `SandboxInterceptor` subclasses `QQmlAbstractUrlInterceptor` and `SandboxNetwork` subclasses
`QQmlNetworkAccessManagerFactory` (its managers subclass `QNetworkAccessManager::createRequest`): cxx-qt declares no subclass of a Qt
C++ interface with virtual functions, so these are the glue §1.4 allows. Each asks Rust: the interceptor calls `policy_reads` and
`policy_judge` through the bridge and holds no rule of its own. Their log is guarded by a mutex, since the type loader and the
picture readers call from their own threads; the policy is plain immutable data and is shared by reference, owned by the host and
dropped after the sandbox. The sandbox (engine ownership and the screens' creation through `QQmlComponent`, which cxx-qt-lib does
not bind) is the third piece of glue; `cpp/sandbox.h` and `cpp/sandbox.cpp` hold all three in 264 lines.

**In the program.** `ThemeHost.qml` (`EmuSen.App`) holds the host and one `Loader` slot per role, each made while the theme provides
the role and placing the screen when loaded; a slot's `themed` is true when its screen is in place and the theme has not failed.
`Main.qml` draws it over the reference's screens, below every card, and hides the reference's Home, Choose a console or grid only
where the theme's slot is `themed`; a role the theme omits, and every role once it fails, is the reference's, drawn as before. A
theme's splash slot is made only while the splash is shown and is drawn in a layer over Moonrise, faded with it: Moonrise keeps the
splash's clock, the skip and the fade (`EmuSen_Interface.md` §22), so a theme's splash cannot hold the start or take a press. Until
`EmuSen.Theme`'s `Screen` gives a splash its `length`, Moonrise's length is the splash's, and Moonrise draws unseen under a theme's
splash. The theme is chosen by `--theme DIR` or `EMUSEN_THEME`, empty for the reference; the cover stores are the media store and the
settings' `ArtworkDirectory` and `EsdeMediaDirectory`. With no theme chosen the program is unchanged.

**Proof.** `theme-host/tests/policy.rs` (5 tests, no Qt): the folder, the cover store and nothing else on disk; no network and no
unknown scheme; Qt's modules resolve and the named ones never; a theme file held to the allow-list; a folder in the resources.
`tests/sandbox.rs` (offscreen, the software backend, `EmuSen.Models` imported by the test scene so its types are registered as in
the program): a stub theme's screen fills its slot and reads a picture from its folder, quietly; an `EmuSen.Models` import, a remote
picture, a file outside, an inline component and a newer API each fail with the words above; switching stub → remote → stub leaves
one engine, and no theme none. `tests/lint.rs`: the committed registry `Testing/qmltypes/EmuSen.ThemeHost-1.0.*` is the built one,
the scene keeps the QML rules, the dependency list. `qt-app/tests/theme.rs`: a theme's splash over Moonrise and gone after it, its
Home in place of the reference's and the omitted Choose a console the reference's, a failing theme reported and the reference
shown, switching twice leaving one engine. The stub themes are plain QtQuick under `theme-host/tests/themes/` until `EmuSen.Theme`
lands.

**Not yet.** The `Screen` type and the splash's `length`, `still` and `finished` (T); the Themes tab and the setting that chooses a
theme (S); a themed Games screen opens no List card until `Actions.openListCard` exists (T, R). What the sandbox does not do is
§3.5's paragraph above, unchanged: a theme can still spin or allocate.

### 3.6 Console-style designs as themes: the test that the API is sufficient

Four designs, each **inspired** by a console's shell (its arrangement and motion, never its logos, glyph sets, fonts or
assets), each written against §3.2 with no addition to it. If any of the four needed a new singleton or role, the API
would be too narrow; none does. **[A]**

| Design | Arrangement (general patterns only) | What it reads | What it places |
|---|---|---|---|
| **Deck-like** | a left column of places and consoles with the focused one expanded; the content a horizontal band of large art tiles with the focused tile's details beneath; a dark surface with a cool accent | `Library.places`, `Library.consoles`, `gamesOf`, `recent`; `Device.controllers`, `battery`, `clock`; `Session` for a Continue tile | `Frame` (navigation + content + detail), `CoverTile`, `HelpBar`; tokens: `accent`, `surfaceRaised` overridden |
| **Xbox-like** | rows of blocks: a large Continue block, a row of recent covers, square tiles for places; blocks slide as the focus moves | `Session`, `recent`, `places`, `collections`; `Device` | `NormalizedCanvas` with `ThemeScreen.focusItems`; `CoverTile`; the walk through `Frame` roles `content` and `actions` |
| **PlayStation-like** | one horizontal strip of console or game icons along the top third, the focused one larger with its name under it; the lower two thirds show the focused item's art and facts; a soft moving background | `consoles` or `gamesOf`, `hue` for the background's tint, `description`, `video` | `Carousel` (over `PathView`), `Sky` with the glow following `hue`, a `VideoTile` where a clip exists |
| **Switch-like** | a horizontal strip of game tiles at mid-height, round places beneath (favourites, recent, search, settings), a light or dark variant; a cue on every move | `gamesOf(All)`, `places`, `Cues`, `Tokens.variant` | `Carousel` horizontal with equal tiles, `WholeRow` for the round places, `HelpBar`; `Cues.scroll` on move, `select` on A |

Each of the four is also a fit-audit and pad-reach case (§8): a theme passes when every operable item is reached from
the first focus and nothing is cut at 1280 × 800, 1920 × 1200 (the handheld's panel), 1920 × 1080, 1280 × 720 and 3840 × 2160. The two or three the tester
picks are built in stage 3 (§7) as the first third-party-shaped themes, and they stay in the repository as the API's
regression tests.

### 3.7 `theme-check`

A DianaSh command in Rust (the 2026-10-26 rule): `theme-check <folder>` reads `theme.json`, holds it to the schema and the
API version, lints the QML against the import allow-list and §1.3's rule, checks token overrides for type and contrast,
loads each screen offscreen on the synthetic library and runs the fit and reach audits (pad and pointer, §8.1) at the
four sizes, and prints one line per finding (`file:line kind words`) and a last line `ok` or `N findings`. The Themes tab runs it before offering
"Use", as the install gate; the compatibility corpus of §4.4 runs the ES-DE equivalent.

#### 3.7.1 The static step as built **[M 2026-11-08]**

Three Qt-free crates, each outside the Frontend workspace with its own lock file and a fence test, since the back end's dev module
links the command and must never pull in Qt. The rules and their kinds are in `man theme-check`.

- **`emusen-theme-manifest`** (`Frontend/theme-check/manifest`) is the one owner of `theme.json`'s schema 1: `parse(text)` gives the
  manifest as plain data (name, author, licence, `api`, `variants`, `fontSizes`, `screens`, token overrides per variant with their
  lines, `cues`, `previews`) and each break as a finding on its line. A schema above 1 or an API newer than the build's `1.0` (a
  newer major, or a newer minor) is refused with words naming both; a path that leaves the package is refused; an unknown key is a
  note that refuses nothing (§5.2). The theme host reads `theme.json` through it, so the schema is not written twice.
- **`emusen-qml-rules`** gained a theme profile: `THEME_IMPORTS` (the six modules of §3.5, matched whole, which the host's engine
  reads too), `check_theme`, §1.3's rules with that list in place of the view list, and quoted relative folder imports allowed,
  and `local_imports`, so the checker can hold each folder inside the package.
- **`emusen-theme-check`** (`Frontend/theme-check`), the command `theme-check <folder>...` in `emusen.dev`: the manifest; the listed
  screens against the files present; the files it names; every `.qml` through the theme profile; each token override against the
  kit's committed registry (a read-only property of `Tokens`, of its declared type), the text scales against kit-core's range; and
  kit-core's 25 contrast pairs a variant, with the overrides applied, through `Colour::contrast`. A pair is measured only when the
  theme overrides one of its tokens. The colour token to property table (`overAccent` for `OnAccent`, and so on) is held to the
  registry both ways by a test.

Proof, 2026-11-08: four fixture packages under `tests/themes` (clean `ok`; a literal in a part; `import QtMultimedia` in Home; a
dark `accent` of #37526F, 2.13:1 on the surface, giving four contrast findings and none in the light variant), each named in the
output; the schema, token and screen cases in temporary packages. Five seeded faults, each caught: the `OnAccent` property misnamed
(the registry test and the contrast fixture); `EmuSen.*` let through the theme profile (qml-rules' theme test); the containment of
a folder import dropped (the screens test); a newer minor accepted (the API test); an overridden back ignored (the contrast
fixture). The step that loads the screens offscreen and runs §8.1's audits is a Qt-side program of its own (C2), run by
`theme-check` as a process.

---

## 4. ES-DE themes: the engine as a component

### 4.1 What exists and what it learned

The C# work (`EmuSen_BigPicture.md` §12 to §43, 2026-09-24 to 2026-10-16) built a loader (`BigPicture/Theme`, 2,523
lines free of Avalonia), a scene (element state as a function of resolved view, data and time, 2,098 lines) and a painter
of LunaP controls, and measured it against ES-DE run as a black box. What it learned, carried over whole:

- **The format** (`THEMES.md`, MIT): `capabilities.xml` mandatory; two views; the parsing order (transitions,
  variables, colour schemes, font sizes, languages, includes, general, variants, aspect ratios); one global variable
  namespace; 12 aspect-ratio names and their vertical forms; `pos`/`size` as fractions, `fontSize` of the height,
  `cornerRadius` of the width; structural errors untheme a system, bad values reset with a warning; zIndex defaults;
  variant triggers one step deep; no format version (the `<formatVersion>` tag was dropped; ES-DE 2.2.0 and later refuse
  the legacy format outright) [read again 2026-11-07].
- **Compatibility, measured.** Of the 66 themes ES-DE's list held on 2026-09-26, 46 loaded with no loader error and 51
  were themed for the five systems surveyed; after Q47 matched ES-DE's lenient rules (§31 there: 15 refused themes all
  load in ES-DE 3.4.1, so the loader was wrong, not the themes), **66 of 66 load**. No listed theme is drawn whole: every
  one sets at least one property the scene does not draw, and 28 of 66 use an element or carousel kind not drawn
  (wheels, `gameselector`, `animation`, video), the counts as of pass 14. Video is the largest unmapped property family.
- **The diagnostics model** (`ThemeDiagnostics`: debug, warning, error with a code, file and line) and the install gate
  that is the loader itself; the capabilities record (variants with triggers, colour schemes, font sizes, languages,
  aspect ratios, transition profiles); the browser over `themes.json` (fields, hosts, licence line from the README, no
  network until the browser opens, `themes.db`); the sound resolver (the theme's file for a cue name, else the built-in,
  `EmuSen_Interface.md` §23.1).
- **The oracle.** ES-DE itself, run locally on the same inputs, its captures kept outside the repository; the pixel
  comparisons hold scene output, not a toolkit's rasteriser.

### 4.2 The component

`emusen-esde`, one crate, three modules with one job each, behind the platform's rules (R):

- `loader`: XML to a resolved element model, variables, capabilities, diagnostics; a direct translation of the C# loader
  with its fixtures re-expressed in Rust against the same synthetic themes; ES-DE's lenient rules of Q47 kept.
- `scene`: each element's rectangle, opacity, offset and text at time *t* for a resolved view, the selection and the
  measured durations (`EmuSen_BigPicture.md` §14); plain records out, no Qt.
- `check`: the `esde-check` DianaSh command: loads a theme for a list of systems, prints diagnostics and a property map
  (set, unknown, unsupported) as TSV, which is the corpus tool of §4.4.

The painter is QML: `EmuSen.Esde.View` is a `NormalizedCanvas` instantiated from an `ElementModel` (a cxx-qt
`QAbstractListModel`, one row per element with its type, name, rectangle and typed properties as roles) through a
`DelegateChooser` by type: `image` → `Image` + `MultiEffect` (tint, saturation, radius), `text` → the kit's text,
`textlist` → `ListView`, `carousel` → `Carousel` (`PathView`, the wheel's path), `grid` → `GridView`, `rating`,
`badges`, `datetime`, `gamelistinfo`, `helpsystem` → `HelpBar`, `clock`, `systemstatus`, `video` → `VideoTile`,
`animation` → `AnimatedImage` for GIF and *[decision]* for Lottie (§11 Q6). Motion is the scene's: QML binds the roles
and animates nothing of its own, so ES-DE's measured durations stay numbers in Rust and the pixel tests compare the scene.

### 4.3 Where compatibility work goes after the switch

The unmapped families, in the order of the themes they unlock: video (12 of Art Book Next's 20 variants play one),
`gameselector`, wheel carousels and reflections, Lottie, `badges` sizing, the carousel's margins and offsets. Each is a
scene or painter change; the loader is done. The ranking is re-measured by the corpus before each step.

### 4.4 The compatibility corpus and its measure

- **The corpus** is ES-DE's own list (`themes.json`, 66 themes on 2026-09-26; `latestStableRelease` 52 on 2026-11-07),
  fetched XML-only as pass 3 did (the five systems' XML was 68.8 MB), kept under `~/.cache/emusen/bigpicture/corpus/`
  outside the repository, refreshed on request. Art Book Next, CarAlt, Artflix and the themes the handheld's player
  installs are the *watched* subset, fetched whole with assets.
- **The measure**, from `esde-check` over the corpus, one TSV row per theme: loads (yes/no), systems themed (of five),
  properties set, unknown, unsupported, and the element kinds used; and for the watched subset, the pixel difference
  against ES-DE's capture per view at 1280 × 800 and 1920 × 1200 (mean absolute error and the count of elements whose
  rectangle differs by over two design pixels). The headline numbers are the three the 2026-09-26 survey set: themes
  loading, themes drawn whole (no unsupported property), themes using no undrawn kind. The targets for the switch: 66 of
  66 load (held), drawn whole rises from 0 as §4.3's families land, and the watched subset's view difference stays under
  the C# painter's on the day of the switch, recorded as the baseline.
- **Change detection.** The corpus TSV of each measurement is committed under `Testing/esde-corpus/<date>.tsv`; a row
  that worsens fails the landing until explained.

### 4.5 `emusen-esde` as built: the loader, the scene and `esde-check` **[M 2026-11-08]**

Stage 4's first half. The crate is `Frontend/esde`, outside the Frontend workspace with its own lock file, as
`qml-rules` is, because the back end's development module links it; `tests/fence.rs` holds its tree to the crate,
quick-xml and memchr, with no Qt, no cxx-qt and no other EmuSen crate. The QML painter is the second half and is not
here.

#### 4.5.1 The three modules

| Module | Job | Files |
|---|---|---|
| `loader` | a theme folder to a `Theme` for one system and one set of choices: capabilities, selection, the parsing order, includes, variables, merging, typing, defaults, bindings, drawing order and every diagnostic with its code, file and line | `xml`, `diagnostics`, `values`, `catalog` (with `catalog.tsv`), `capabilities`, `choices`, `parse`, `view`, `resolved` |
| `scene` | each element's rectangle, rotation, opacity, offset and text at a time on the host's clock, for a resolved view, the selection and ES-DE's measured motion; the move between views; the painter's `Frame` (§4.5.5) | `data`, `motion`, `words`, `grid`, `mapping` (with `mapping.tsv`), `view`, `stage`, `frame`, `paint`, `primary`, `help`, `draw` |
| `marks` | the built-in icons a theme does not supply, as polylines in a unit box (§4.5.5) | `marks` |
| `check` | a theme or a corpus loaded for every selectable variant and each system, measured as TSV | `check` |

- **A translation, file by file.** `loader` follows the C# `BigPicture/Theme` one to one: `ThemeXml` → `xml`,
  `ThemeValueParser` and `ThemeValues` → `values`, `ThemeCapabilities` → `capabilities`, `ThemeChoices` → `choices`,
  `ThemeLoader`'s `ParseRun` → `parse`, `ThemeViewBuilder` → `view`, `ResolvedTheme` → `resolved`. Q47's and Q70/Q71's
  readings (`EmuSen_BigPicture.md` §31, §35) are kept to the character: ES-DE's leading-number FLOAT, C-style
  UNSIGNED_INTEGER and COLOR, the pair split at its first space, `selectable` false only on `0`, `f` or `n`, untrimmed
  paths and includes, an unknown `imageType` hiding the element.
- **The catalogue is data.** The 16 element types and 472 properties (468 until the re-check of §4.5.5) are `catalog.tsv`, one row a property (type,
  literal default, same-as default, computed default, range, pair rule, shape, values, view limit, deferral), read once.
  It was generated from the C# `ThemeCatalog` by a script outside the repository on 2026-11-08 and is checked by the
  C# catalogue tests re-expressed (§4.5.3), the 39 hand-read entries among them.
- **The XML reader.** quick-xml 0.38 (MIT), a streaming reader, with a small owned tree built over it. It was chosen
  over a DOM parser because ES-DE's lenient reading needs a fragment read: when the strict reading fails, a bare `&`
  is escaped, declarations are blanked, the first element is read whole and anything after it is ignored, as the C#
  `ThemeXml` does through `XmlReader` in fragment mode. Whitespace-only text is dropped and line ends normalised, as
  `XDocument` does; lines are counted from the reader's byte offsets.
- **One addition to the C# model.** A diagnostic carries a `subject` (the tag, the element type, or `type.property`)
  for the three unknown codes, so the property map does not parse messages.
- **The scene's record.** `SceneView::frame()` gives one `ElementState` per element in drawing order: why it is
  skipped (the C# `SceneBuilder.Skip` reasons, and "nothing to show" for a text with no words), the box in pixels from
  `pos`, `size` and `origin`, `rotation` and `rotationOrigin`, the opacity (its own, times `scrollFadeIn`'s rise, times
  the held list's metadata fade), the offset (a carousel's or list's position in items, a grid's first row), the time
  since the selection, the `TextScroll` rule a self-scrolling text follows (LunaP's §102.3, with ES-DE's measured
  rates), the grid's focus, and what a text, datetime or clock says in its letter case. `Stage::layers()` gives the two
  views during a pan with their shifts, the help bar, clock and status held still. The clock is microseconds the host
  passes in; nothing reads wall time. Text widths are the painter's: it reports them through `Stage::measured` and the
  scene gives the offset (§4.5.5).
- **What the C# scene did that this one leaves to the host or the painter.** Media lookup (the C# `ISceneMedia`; the
  host fills `Game::media`), the system view's gameselector picks (`GameSelectors`; the system view's game is none
  here), animation frame timing, and the rebuild-on-step lever of `EmuSen_BigPicture.md` §14.8, which has no meaning
  where a frame is a function of state. Badges, help entries, device status and the quick-scroll letters moved into
  the scene with the painter's contract (§4.5.5).
- **What is drawn** is `scene/mapping.tsv`: every catalogue pair once, with its status (§4.5.5). Its `drawn` rows are
  the C# `SceneMapping` as it stood on 2026-11-08, the pairs the painter must draw to match the C# one; `check` counts
  a pair neither drawn nor accepted as unsupported. Each painter stage that draws more extends it.

#### 4.5.2 `check` and `esde-check`

`esde-check` is the fourth command of the `emusen.dev` module (`Platform/emusen-dev-module/src/esde_check.rs`, a
cross-component dependency on `Frontend/esde`, the same shape as `qml-rules`'s), with its page at
`Docs/man/emusen.dev/esde-check.1`. A theme is loaded once for each selectable variant (once when none is offered) and
each of EmuSen's five systems, at 1280 × 800, as pass 3's survey loaded it. The default table is one row a theme
(loads, systems themed, variants, runs, errors, warnings, pairs set, unknown, unsupported, drawn whole, kinds, undrawn
kinds), ending in the three headline numbers; `--diagnostics` and `--properties` print the diagnostics and the property
map. It exits 1 when a theme does not load, which is the landing check §4.4 asks for.

#### 4.5.3 Proof: the C# fixtures, then the C# survey's own corpus

**The fixtures.** Every loader fixture of `ThemeCapabilitiesTests`, `ThemeLoaderTests`, `ThemeErrorTests`,
`ThemeCatalogTests`, `ThemeEsdeRulesTests` and `ThemeValueRulesTests` is re-expressed in `Frontend/esde/tests/` against
the same synthetic themes (`tests/common` is `SyntheticTheme`), theory rows as loops over the same rows: 116 tests, all
passing. Left out are the two install-gate cases, which test the C# downloader, not the loader. The scene's fixtures
are those of `SceneMotionTests`, `GridSceneTests` and `SceneSemanticsTests` that assert state rather than a control's
pixels, 22 tests over the same synthetic library and the same round-number motion. One translation defect was found
by them: the grid's two focus levels were assigned one after the other, so the second read the first's new value; the
C# assigns them as a pair. `A_step_while_moving_starts_from_the_items_current_focus` caught it.

**The September corpus.** `esde-check` was run over the 66 themes' XML that pass 3 fetched on 2026-09-26
(`~/.cache/emusen/bigpicture/theme-survey/xml`) and compared theme by theme with the C# survey's `survey.json` of
2026-09-27. For all 66: the same systems themed, the same errors (0), the same runs, the same undrawn element kinds and
the same count of unsupported pairs. Warnings differ in 22 themes by 1 or 2, each by the theme's own capabilities
warnings, which the C# survey counted once more outside its loads (`ThemeSurveyTool.Analyse_the_fetched_themes`);
checked on SimpleMenu (one bare `&`) and Razor (a bare `&` and `ar_SA`). 66 of 66 load, as in C#. The whole run takes
2.7 s, against the C# survey's 3.6 s.

**Seeded faults**, each applied alone and reverted:

| Fault | Caught by |
|---|---|
| general `<view>`s parsed before `<variables>` | `errors::a_property_made_empty_by_its_variables_is_ignored_with_a_warning` and the parsing-order tests |
| a bare `&` left raw in the lenient reading | `xml::a_bare_ampersand_and_text_outside_are_read_leniently`, `esde_rules::a_bare_ampersand_is_kept_as_text_and_references_are_decoded` |
| the fast tier stepping one item, not four | `scene::a_list_wraps_on_a_tap_stops_at_its_end_when_held_and_jumps_at_the_fast_tier` |
| the metadata fading at the press, not the first repeat | `scene::metadata_fades_out_while_the_list_scrolls_fast_and_back_in_when_it_stops`, `scene::a_held_grid_fades_the_metadata_out_from_the_first_repeat` |

#### 4.5.4 The corpus measured on 2026-11-08

ES-DE's `themes.json` was fetched from its list's repository on 2026-11-08: **71 themes** (the 66 of September and five
new: Atelier, Hyperflix, NXE Menu, PSOne-ish, Unreleased), all on github.com but one on gitlab.com, and
`latestStableRelease` 52. That field is the list's own release number, not a count of themes (it was 51 in September).
Each theme's XML was fetched by a blobless, sparse, depth-one `git clone` from the address the list names, so that
only `*.xml` blobs came down and no API allowance was spent: 25,548 files, 291 MB, every XML file rather than pass 3's
five systems' (68.8 MB), since a file skipped by name could be one an include needs. Art Book Next, Artflix (Revisited)
and CarAlt were fetched whole with their theme files (XML, images, fonts, sounds; no video): 219, 72 and 30 MB. Hooks,
LFS filters and links were off, nothing fetched was run, and each folder keeps its commit in `.corpus-commit`. All of it
is under `~/.cache/emusen/bigpicture/corpus/`, outside the repository.

The measure is `Testing/esde-corpus/2026-11-08.tsv`, taken in 2.6 s:

| | September (C#, 66 themes) | 2026-11-08 (Rust, 71 themes) |
|---|---|---|
| Load for all five systems | 66 | **70** |
| Drawn whole | 0 | 0 |
| Use no undrawn kind | 57 | 61 |

- **Every one of the 66 still loads,** and against the Rust run over September's XML no row worsened; one improved
  (X-Grid sets two fewer unsupported pairs, 9 to 7).
- **NXE Menu does not load:** its carousels set `itemLinearScale` and `itemLinearSpacing`, which the catalogue lacks,
  so each is an unknown property and unthemes every system. Both are documented in ES-DE's current `THEMES.md` (master,
  read 2026-11-08, its "carousel" section: NORMALIZED_PAIR, the scale and the spacing of the items before and after the
  centre); the catalogue was read from `THEMES.md` in September. The C# loader shares the catalogue and would refuse it
  the same way. This is two catalogue rows to add, not a translation defect. Decided 2026-11-08: add them; with the
  re-check of §4.5.5 NXE Menu loads, and all 71 do.
- **The watched subset, fetched whole:** Art Book Next in all 100 runs with no error and no warning, 168 pairs set
  and 5 unsupported (the video playback properties); Artflix (Revisited) and CarAlt load in every run, with 6 and 8
  unsupported pairs, and both use a vertical wheel carousel, which the scene draws only in part. The pixel comparison against ES-DE's captures waits
  for the painter.

#### 4.5.5 The painter's contract: `Stage::frame`, `mapping.tsv`, `marks` and the catalogue re-check **[M 2026-11-08]**

The scene's output for the QML painter, as the painter's design (§3 and §13 there) defines it. Everything is in window
pixels at the scene's time, a pure function of (view, data, size, time, measurements); the painter binds it and
animates nothing.

**The records** (`scene/frame.rs`), produced by `Stage::frame()` (`scene/draw.rs` per view, `scene/stage.rs` for the
two views of a pan):

| Record | Holds | Built from |
|---|---|---|
| `Frame` | `elements: Vec<Placed>` in drawing order (the leaving view's then the current view's during a pan, the overlays unmoved), `primary`, `leaving_primary`, `moving`, `next_change_ms`, `overlay` | `Stage` |
| `Placed` | `id` (FNV-1a of view, kind and name, stable across frames and steps), kind, name, `x y w h`, rotation, rotation origin (fractions), opacity (the element's × the metadata fade × `scrollFadeIn`), `paint` | `SceneView::frame()`'s `ElementState`, its box re-placed by origin once measured |
| `Paint::Picture` | source, fit (Fill, Contain, Cover, Tile), crop position, tile size and alignment, tint (colour, gradient end, direction), saturation, corner radius px, nearest, GIF frame | `paint.rs`, the C# `ImageElements` translated: image, video's still, animation (GIF) |
| `Paint::Text` | words in their case, font, px, colour, alignments, line spacing, wrap, ellipsis, background (grown outward by its padding), `Scroll` | `TextElements` and the clock |
| `Scroll` | a horizontal offset and the distance to the following copy, or a vertical offset and opacity | `TextScroll::loop_offset` and `run_at`, once measured |
| `Paint::Rating` | value, filled and unfilled (`Icon`: the theme's file or the star mark), tint, overlay | the box 5:1 when one axis is 0 |
| `Paint::Badges` | each shown slot's cell, its icon (a file fills the cell, a mark a centred square), the controller and folder link overlays | `SceneBadges` and LunaP's `BadgeStrip.Cells` |
| `Paint::Help` | entries (label in case, `PadButton`, pair, the theme's icon for the button), font, px, colours, background, spacings, scale, filled glyphs | `HelpPrompts`, `help.rs` |
| `Paint::Info`, `Paint::Status` | marks and texts, styles | gamelistinfo's counts (§39 of `EmuSen_BigPicture.md`); the device's radios and battery |
| `Primary` | the element's id, `Rows` (systems, or one system's games), selected, and `Carousel { slots }`, `TextList { first_row, row_h, content_y, selector, colours, marquee }` or `Grid { columns, pitch, margin, content_y, focus, styles }` | `primary.rs`: the C# `PrimaryElements` and `GridElements`, LunaP's `ImageCarousel` and `TextRowList` |
| `Slot` | row, the item box before scale, scale and its origin, turn and its origin, opacity, dim, saturation, tint, reflection | `ImageCarousel.ItemRect`, `GrowthPoint`, `WheelTransform`, `WheelAlignShift`, farthest first |
| `ScrollLetters` | letters, star, px, opacity | the C# `ShowScrollLetters` |

- **A slot's matrix as two transforms.** LunaP placed a carousel item by one matrix: scale about the growth point, the
  wheel's turn about its centre, then the vertical wheel's alignment shift (or, upright, a translation round the
  wheel). A translation after a scale and a turn equals the same scale and turn about origins moved by it, so a slot
  carries the box already moved and the two origins moved with it, which QML's `Scale` and `Rotation` take directly.
- **`measured(id, w, h)`** is the painter's report of what only it can know: a picture's own size, a text's or an
  indicator row's extent, a text list's selected name's width. The scene keeps it with a key of the content it measured
  (FNV-1a of the paint before measurement), so a measurement of the last game's name or cover is never applied to the
  next: after a step the scroll is none and the box is the unmeasured one until the painter reports again.
  `Stage::measured` returns whether the frame changed. It sizes `maxSize` pictures (contained), pictures with one axis
  0 (the other by aspect) or none (their own size), and texts and the help, status and info rows on the axes the theme
  left at 0, placed by `origin`.
- **`moving` and `next_change_ms`.** `moving` is a glide, a hold, a fade, a `scrollFadeIn`, a pan or a measured scroll
  in motion. `SceneView::is_animating` is the first five; `is_moving` keeps its old meaning (any text that may scroll),
  which the scene tests use. `next_change_ms` is the nearest of each measured scroll's wait (`TextScroll::loop_wait`,
  `run_wait`: until the delay, the pause or the fade ends) and a shown clock's next minute, or next second when its
  format shows seconds, from `Data::now`. At rest with no clock and nothing measured to scroll, the frame asks for
  nothing: the condition for §6's zero frames at rest.
- **Inputs added to `Data`, `System` and `Game`:** the device status and the indicators kept, ES-DE's controller type
  (which picks a theme's help icons), the face-button swap, the help context, the quick-scroll setting; a system's
  stars, favourites-first and counts; a game's media by type (the host's lookup), controller, manual, alternative
  emulator, folder link and collection membership.
- **`PadButton`, not the kit's `UiButton`.** The crate depends on nothing of EmuSen's (`tests/fence.rs`), so a help
  entry names its button by place, as THEMES.md's entries do (a, b, x, y in Xbox positions, the pads, shoulders,
  triggers, back, start, stick click); the bridge maps place to the kit's button. One owner each: the places are
  ES-DE's, the kit's functions the kit's.

**`scene/mapping.tsv`** is now one row per (element, property): all 472 catalogue pairs, each once, with a status. The
test `mapping_lists_every_catalogue_pair_once_and_the_dimmed_help_as_accepted` holds it equal to the catalogue.

| Status | Pairs | Meaning |
|---|---|---|
| `drawn` | 347 | the C# `SceneMapping` sets, `Common` expanded to each element that declares it, less the 20 pairs the painter's mapping test found undrawn (§4.8.5, measured 2026-11-08) |
| `accepted` | 9 | the help's eight `*Dimmed` properties (decided 2026-11-08; the design counted six, `THEMES.md` lists eight: `posDimmed`, `originDimmed`, `textColorDimmed`, `iconColorDimmed`, `fontSizeDimmed`, `entrySpacingDimmed`, `iconTextSpacingDimmed`, `opacityDimmed`), and `sound.path`, which the host plays |
| `later:video` | 16 | the video family |
| `later:gradients` | 22 | the `*ColorEnd` and `*GradientType` pairs the C# did not draw |
| `later:item-text` | 19 | carousel and grid item text styles and scrolling, the collections' letter cases, name suffixes |
| `later:picture` | 16 | interpolation, corner radius and brightness where not drawn, flips, mipmap, `scaleFactor`, `gameOverridePath`, `defaultFolderImage` |
| `later:stationary` | 11 | `stationary`, `renderDuringTransitions`, `fadeAbovePrimary` |
| `later:carousel-layout` | 6 | `itemStacking`, `selectedItemOffset`, `itemLinearScale`, `itemLinearSpacing`, `itemAxisRotation`, `itemDiagonalOffset` |
| `later:textlist-selector` | 5 | the selector image, width and offset, the secondary selected background |
| `later:help-layout` | 1 | `entryLayout` |
| `later:gameselector` | 13 | the gameselector's picks (`selection`, `gameCount`, `allowDuplicates`) and the elements that read them (`gameselector`, `gameselectorEntry` of image, video, text, datetime and rating): the scene shows no game in the system view (§4.8.5) |
| `later:animation-timing` | 3 | a GIF's `speed`, `direction` and `iterationCount`: the scene gives the painter no frame, so a GIF stands on its first |
| `later:container` | 2 | a text container's `containerVerticalSnap` (not read) and `containerScrollSpeed` (read, no visible change by 1 s) |
| `later:grid-rows` | 1 | `fractionalRows`: the grid's record carries no row count, so a partial row is drawn either way |
| `later:text-lines` | 1 | datetime's `lineSpacing`: one line at the top of its box is placed the same at every spacing |

`check` counts a pair as `set` when it is drawn or accepted, `unsupported` otherwise; nothing reads `catalog.tsv`'s
`deferred` column for it any more.

**`marks`** (`src/marks.rs`): 30 icons as `Mark { aspect, parts }`, each part rings of points in the unit box (the first
the outline, the rest holes, filled even-odd) filled or stroked at a width in box heights, in the element's colour, the
idiom of kit-core's `Shape`. They are LunaP's drawings translated: the list's star, folder and tick; gamelistinfo's
gamepad, star, funnel and folder (an em high, the baseline 0.85 down); the status's Bluetooth, Wi-Fi, cellular and the
battery at 0 to 4 bars; the nine badges and the folder link; the controllers (unknown, generic pad, NES, SNES, N64).
Arcs are flattened (eight segments a corner, 32 a circle). One difference is named: the unknown controller's "?" was a
font glyph in LunaP and is a stroked hook and dot here, since the crate sets no type.

**The catalogue re-check.** ES-DE's `THEMES.md` (master, fetched from gitlab.com/es-de/emulationstation-de on
2026-11-08) was read element by element against `catalog.tsv`: names, types, literal defaults, ranges and enumerated
values. Six differences, all additions since September:

| Element | Added | From `THEMES.md` |
|---|---|---|
| `carousel` | `itemLinearScale`, `itemLinearSpacing` | NORMALIZED_PAIR, −0.5 to 1 per axis, default `0 0`, horizontal and vertical types only |
| `image`, `animation` | `scaleFactor` | FLOAT 0.1 to 1, default 1; SVG (image) and Lottie (animation) rasterisation scale |
| `text` | `metadata` values `customCollectionNameGrouped`, `customCollectionNameGames` | the custom collection's name in the grouped system, and inside it |

The two new `metadata` values are accepted by the loader and draw nothing yet (a text with nothing to say is skipped):
the scene has no grouped custom collections, which belong to the shelves (the design's piece D2).

**The corpus re-measured**, `Testing/esde-corpus/2026-11-08b.tsv`, the same 71 themes' XML as §4.5.4, in 2.75 s:

| | 2026-11-08 (§4.5.4) | 2026-11-08b |
|---|---|---|
| Load for all five systems | 70 | **71** |
| Drawn whole | 0 | 0 |
| Use no undrawn kind | 61 | 61 |

No row worsened but one, and that one by loading: NXE Menu loads in all five runs with no error (was 20 errors, 2 unknown properties; its unsupported
pairs rise from 10 to 12, the two carousel rows now known and `later`). 30 themes set
fewer unsupported pairs, each by its help's dimmed properties now accepted (CarAlt 8 to 2, Epic Noir Revisited 8 to 2,
Razor 4 to 2). Atelier and Analogue 3D Menu warn less (6,795 to 6,585 and 2,575 to 2,540), each by the two
`customCollectionName` values no longer refused. Nine themes now set at most two unsupported pairs; the pairs most often
unsupported are the video family's (`video.delay` in 55 themes, `pillarboxes` 49), then `carousel.imageInterpolation`
(20) and `image.stationary` (18), the order the painter's design ranks the families in.

**Proof.** 14 tests in `tests/frame.rs` (the mapping and its accepted pairs through `check`, the marks in their box,
the carousel at rest and mid-step, a wheel and a reflecting row, the text list's window and marquee through a
measurement, a `maxSize` picture and a text sized by its words, the help's words, buttons and icons under the swap,
badges, status and rating, the overlays through a pan, the quick-scroll letters, the clock's wake); the crate's 157
tests pass. Seeded faults, each applied alone and reverted:

| Fault | Caught by |
|---|---|
| carousel slots drawn nearest first | `a_carousel_s_selected_slot_grows…`, `a_carousel_mid_step…`, `a_wheel_turns…` |
| a list's measurement kept after its name changes | `a_text_list_keeps_the_selection_mid_window_and_scrolls_its_name_once_measured` |
| accepted pairs counted unsupported | `mapping_lists_every_catalogue_pair_once…`, `a_theme_setting_only_accepted_pairs_counts_none_unsupported` |
| the swap moving the button but not its icon | `the_help_names_the_project_s_actions_on_the_buttons_and_icons_of_the_pad` |
| overlays panned with the view | `the_overlays_hold_still_while_the_views_pan_and_both_primaries_are_drawn` |
| the next change ignoring the scrolls | `a_text_list_keeps_the_selection_mid_window…` |
| `maxSize` contained by the larger ratio | `a_picture_with_max_size_is_contained_once_its_own_size_is_known` |

**Left for later, named:** a GIF's frame at the time (`Picture::frame` is none until the painter reports the file's
frame delays); the system view's gameselector picks (the host's); the grid's and carousel's per-row text styles beyond
the C#'s (`later:item-text`). A frame allocates its records each tick; the bridge's models change rows in place
(design §4), and whether the scene's own allocation shows in a frame's cost is measured with the painter, not assumed.

### 4.6 The library as an ES-DE theme sees it, and its screen **[M 2026-11-08]**

The painter takes plain data from `LibraryModel` (stage 3's curated view is not built); `emusen-qt-models` writes it,
`esde_shelves.rs`, and nothing of the painter reaches into the clients.

**The texts.** One refresh, off the GUI thread as `lists_of` is, writes four texts, published as `LibraryModel`
properties after the refresh that changed them and never on a step:

| Property | Text | From |
|---|---|---|
| `themed` | `LibraryStyle` is `Theme` and `BigPictureTheme` names a folder | the player's settings (`clients::app_settings`) |
| `esdeSettings` | `{"style", "theme", "choices", "interface", "collections", "show_hidden"}`; the choices stored under the theme folder's absolute path less a trailing separator, as Mistress's `ThemeSettingsWindow.Key` keys them | the same |
| `esdeShelves` | each shelf as the gamelist rules list it now (§4.7): the consoles with games, in the systems sorting's order, under the systems table's `esde_*` names, then the collections | the systems table's `esde_*` fields (the `shelves` call reads the same table) |
| `esdeMedia` | `{system: {type: {name: path}}}` | `MediaIndex`, below |
| `esdePresence` | `{shelf: [type]}`: the types some game of the shelf has, each game looked up under its own console, the theme's variant triggers | derived from the index |

Each game carries every field Mistress's `SceneGame` carried: `file`, `name`, `sort_name`, `custom_sort_name`,
`description`, `developer`, `publisher`, `genre`, `players`, `release_date` (the metadata's `yyyy-MM-dd`), `rating`
(0–1 or null), `last_played` (seconds since 1970, 0 for never), `play_count`, `play_time` (seconds), `favourite`,
`completed`, `kid_game`, `broken`, `hidden`, `not_counted`, `hide_metadata`, `controller`, `alt_emulator`, `emulator`
(the alternative emulator's name), `manual` (the index has a manual), `folder`, `folder_path`, `folder_link`, and
`esde_system`, `esde_full_name`, `is_collection` and `face` for the collections (§4.7). The description is left out of
the text and given by `LibraryModel.esdeDescription(file)`, the one field that grows long. The
metadata is the scraper's `metadata` read (EmuSen_Backend.md §30.3), its edit else ScreenScraper's else the default,
chosen there and not again here; the plays are games.db's `records`. While the own look is shown the three library
texts are `[]`, `{}` and `{}` and no game's metadata is read, so the own look's refresh costs what it did.

**`MediaIndex`.** One walk per refresh of each type folder, `<root>/<esde_system>/<folder>/<name>.<ext>`, ES-DE's own
layout, under the media store and then `EsdeMediaDirectory`, the type folders taken from the scraper's `kinds` table; a
key keeps the first file found, so the store wins as it does in the scraper's order. A game's cover is then set from
the scraper's own `media` answer, which the library already reads for every game, so a cover the player placed, a
package's, OpenEmu's or another game's chosen cover is the index's too. The index is kept in the client source with
the scraper's `stamp` of each shelf and the two roots, and an unchanged tree is not walked again. Named differences
from the scraper's `Locate`: a packaged game's own media of a type other than the cover is not indexed; where two
files of one name differ in extension, the index keeps the first by name (`.jpg` before `.png`) where `Locate` tries
`.png` first.

**The screen.** `Screens` gains `Esde` and the property `themed`, which `emusen-qt` binds from `LibraryModel.themed`.
`step_in(themed, shown, ask)` is the whole rule; with the own look it is `step` unchanged. With a theme:

| Shown | Ask | Then |
|---|---|---|
| Home, Choose a console, the grid, its search | the look changed (`Restyled`), as Moonrise gives way | Esde |
| Esde | the look changed back | Home |
| Esde | Start (`home`) | the Home card at its first page |
| Esde | Select (`list`) | the List card, ES-DE's options |
| Esde | B | nothing: the view's own, going back from a gamelist (decided 2026-11-08) |
| Esde, the List card, the Pause Card | play | the game |
| the Home card, the List card | B, Start, or any action | Esde |
| the game | the menu chord | the Pause Card |
| the Pause Card | Back to Library | Esde |
| the game, the Pause Card | the game closed | Esde |

**Measured on 2026-11-08** (`tests/esde.rs`, debug build): the GUI thread's part of a refresh at 9,000 synthetic games
with descriptions, a shelves text over 2 MB (comparing the texts, moving the read in, making the changed texts
QStrings) took 0.65 and 0.67 ms, best of three each, under the 2 ms gate.

**The folder.** A game's `folder_path` is the library's scan answer's `folder` (`emusen_library.h` 1.3, decided
2026-11-08; EmuSen_Backend.md §28.4), Galaxia's `game_folders::of`, so its one owner is unchanged; a game's media is
found by its folder and stem, then its stem. **Owed:** the metadata is one scraper read a game; its cost at a library's
scale is measured with the real library in the hosting piece.

### 4.7 The gamelist rules, `gamelists.rs` **[M 2026-11-08]**

Mistress's `GamelistOptions`, `CollectionShelves` and `ThemedLibrary.Gamelists`/`.Folders` (EmuSen_BigPicture.md §22,
§30) as plain Rust with no view behind them. `esde_shelves::build` applies them to what a refresh read (`Base`) and
writes the shelves' text; a choice of the List card writes it again from the same `Base`, off the GUI thread, with no
read of the library.

| Rule | As built |
|---|---|
| Sort | ES-DE's eleven keys, ascending and descending (System for collections alone, 22 orders); favourites first where the list asks, games with no value of the key after those with one whichever way, then the sort name ascending, then the order given (a stable sort); text compared as .NET's OrdinalIgnoreCase; "1-4" players is four. The default is `BigPictureCollections.DefaultSortOrder` |
| Filters | rating in half stars, developer, publisher, genre, players, favourite, completed, kid game, broken; a value from the list's own games, "Unknown" for none, nothing offered where there is nothing to tell apart; every chosen field holds, any chosen value within one; a folder always passes. The search's words match a name anywhere, over every list |
| Quick selector | "Folders" and "★" where each is sorted on top among other entries, then each first character of the sort name in list order |
| Random | another entry than the selected where there is another, from a number the caller draws |
| Folders | a console shows folders unless flattened (`FlattenedSystems`), once a game of it sits in one; a folder entry holds a kept game, on top unless `FoldersOnTop` is off; entering and leaving keep a path per console; a search or filter that empties the folder walks it up; a folder's link (an edit `folderlink` on the folder's path, the library's `all_edits`) rides on its entry |
| Collections | after the consoles: the grouped custom collections (a folder each, its description naming its games in a seeded random order and showing the first), the discrete ones beside them (grouping `unthemed`, `always`, `never`; `unthemed` by `LibraryModel.esdeThemeFolders`, which the painter sets from the theme), then all games, favorites and last played as `AutoCollections` turns them on; a game excluded from the counter joins none; last played is the library's recent list in its order until a sort is chosen; a custom collection's sortname wins inside it |
| Counts | a gamelistinfo's: the whole list's games and favourites over every folder, how many the search and filters keep, whether a filter is on and a folder open; folders and uncounted games are not counted |
| Each list's own | sort and filter by the list's key (a console, or `collections/<name>` inside a grouped collection); the folder by console |

A shelf of the text is `{"name", "full_name", "theme", "system", "kind" (regular, auto, custom), "collection_id",
"stars", "key", "folder", "sort", "favourites_on_top", "counts", "letters": [[label, index]], "games"}`. A console's
games are whole rows; a collection's are `{"file", "esde_system", "sort_name"}`, the row its console's shelf's, so
each game is in the text once; a folder entry is whole. The List card under a theme reads `esdeListFacts(shelf,
focused)` and acts through `esdeListAct(shelf, action, focused)` with the grid's action keys (Sort steps through the
list's orders, Filter's Favourites is the favourite filter, Jump to and Random answer the entry to select); the
painter calls `esdeEnter(shelf, entry)` and `esdeLeave(shelf)`, and `esdeSearch(words)`.

**Measured on 2026-11-08** (`tests/esde.rs`, debug build): with the collections the GUI thread's part of a refresh at
9,000 games first took 10.6 ms, every game's row written again in all games and favourites and each description
carried; with the collections listing files and the description given on demand, 0.77 ms, under the 2 ms gate.

**Proof.** `tests/gamelists.rs` re-expresses the rule cases of Mistress's `GamelistOptionsTests` (sorts, sort names,
filters, letters, random, names, collection order and grouping, last played, a grouped collection's folder) and adds
folders entered, left and walked up, a link, flatten, folders not on top, a grouped collection entered with its own
sort, and the List card's Sort and Favourites changing the text's order and its facts. Seeded faults, each caught:
missing values sorted first, folders on top inverted, an uncounted game joining the collections, a grouped
collection's sort kept by its shelf, the scan answer's folder emptied.

### 4.8 The painter: `emusen-esde-qt` and the QML module `EmuSen.Esde` **[M 2026-11-08]**

The second half of the engine, as the painter's design laid it out: a crate of its own in the Frontend workspace,
`Frontend/esde-qt`, beside the plain-data `emusen-esde` the way `kit-qt` stands beside `kit-core`. It depends on cxx-qt,
`emusen-esde` (the scene and its marks) and `emusen-kit-qt` (the body face, and from E2 the help bar and the pad
router); `tests/lint.rs` holds its manifest to exactly those and the two EmuSen crates' manifests to no client, runtime
or model crate. The registry is `Testing/qmltypes/EmuSen.Esde-1.0.*`, compared on every build as the kit's is (§5.3).

#### 4.8.1 The primitives

| Type | What it is | Holds |
|---|---|---|
| `Picture` | an `Image` as the texture source (never drawn itself) and one `ShaderEffect` over `shaders/picture.frag`, baked by Qt's `qsb` in `build.rs` into `qrc:/emusen/esde/picture.frag.qsb` | `source` (a URL), `fit` (`Fill`, `Contain`, `Cover`, `Tile`), `alignX/Y`, `cropX/Y`, `tileWidth/Height`, `tileRight/Bottom`, `tint`, `tintEnd`, `gradientVertical`, `saturation`, `dim`, `radius` px, `nearest`, `fadeFrom`, `fadeFalloff`, `sourceScale`; reads back `status`, `ownWidth/Height` |
| `Mark` | `MarkShapes` (a list model in Rust: one row per part of a scene mark, its rings as one SVG path in pixels, filled even-odd or stroked) drawn by one `Shape` a row | `mark` (the `MarkId` name, `star`, `battery3`), `color` |
| `TextBlock` | a `FontLoader` for the theme's file and one for the kit's body face, a `Text` at a fixed line height (`px × lineSpacing`, LunaP's `FontText.LineHeight`), a background `Rectangle` grown by its padding, and a second `Text` for the marquee's following copy | `text`, `fontSource`, `px`, `color`, `hAlign`, `vAlign`, `lineSpacing`, `wrap`, `ellipsis`, the background and its padding, `scrolling`, `scrollVertical`, `scrollX`, `repeatAt`, `scrollY`, `scrollOpacity`; signal `measured(width, height)` |

**The shader, one pass.** The texture coordinate is cropped (`Cover`: the visible fraction of the decoded picture,
placed by `cropX/Y` as LunaP's `FittedImage.Cover` places it) or tiled (`fract` over the box in tiles, the first whole
tile from the left or, with `tileRight`, ending flush with the right edge, as `FittedImage` does). Then, in LunaP's
order (`ImagePixels.Apply`): saturation mixes towards Rec. 601 luma on the stored values; the tint, or its gradient
along the picture's own horizontal or vertical, multiplies every channel, alpha included; `dim` multiplies RGB; a
rounded-rectangle mask of `radius` pixels with a pixel of antialiasing; the reflection's ramp, `fadeFrom` at the top
falling linearly to nothing at `1 / fadeFalloff` of the height (`EmuSen_BigPicture.md` §36.3); and Qt's opacity.
`Contain` draws the effect at the image's painted size, placed by `alignX/Y`, so the mask and the ramp follow the
picture, not its box.

- **Decoding at the drawn size.** `sourceSize` is the box (times `sourceScale` for a slot a transform grows), or the
  tile; Qt keeps the picture's aspect when it decodes, so `Cover` and `Fill` read the decoded aspect. A `nearest`
  picture is decoded at its own size: Qt scales a raster up only for `PreserveAspectFit` and `Crop`, and then smoothly
  (`QImage::scaled` with `SmoothTransformation`), which would blur pixel art before the GPU's nearest filter saw it.
- **Qt hands a `color` uniform over premultiplied.** Measured 2026-11-08: a white picture tinted `80FF8000` read
  (64, 32, 0) over black when the shader multiplied RGB by the tint's alpha again, and reads (128, 64, 0) now. The
  shader multiplies by the colour as given.
- **A resource name in QML.** `qml-rules` counts `fragmentShader` and `vertexShader` with `source` as bindings that
  take a name (decided 2026-11-08: a shader's path is a resource name, not words the player reads).
- **Paths arrive as URLs** (`source`, `fontSource`): QML builds no strings (§1.3), so E2's model hands each file over
  as a `file:` URL.
- **One report per extent.** `TextBlock` raises `measured` from `Text`'s `contentSizeChanged`, which Qt emits once per
  layout that changes either axis; the same words set again report nothing. A face file that does not load, or none
  named, sets the kit's Barlow (decided 2026-11-08: the kit's face is the default). Qt's `FontLoader` keeps one font
  per URL for the process, so a face is loaded once whatever the number of texts.

#### 4.8.2 Proof (E1)

`tests/primitives.rs`, one test on the GPU tier (§8.1: `kwin_wayland --virtual` at 3840 × 2160, so the 1280 × 800
window is not cut by the compositor's own room, measured: at a 1280 × 800 screen the window came up 772 pixels tall;
Vulkan on the RX 6800 with `QSG_RHI_DEBUG_LAYER=1`, which Qt reports as "Graphics API debug/validation layers: 1";
Qt's log required quiet): each primitive drawn over black from pictures the test writes as PPM, and read back.

| Held | Read |
|---|---|
| a white picture tinted `FF8000` | (255, 128, 0) within a level; `80FF8000` reads half of it |
| red, green, blue and (200, 100, 50) at saturation 0 | each its Rec. 601 luma within a level |
| a 192 × 128 picture at a 32 px radius | clear at the corners, opaque at the middles of the top and bottom edges |
| the ramp at 0.5 and falloff 2 on 96 pixels | 0.494 at the top row, nothing at half height |
| a 4 × 4 checker at 8× | hard edges with `nearest`; the same edge blended without it |
| the star mark at 64 px | its colour in the middle, clear at the corner |
| `TextBlock` | one report per new extent, none for the same words; one family for two texts of one file; Barlow when no face is named |

`tests/qmltypes.rs` (rule Q, 3) and `tests/lint.rs` (`qml-rules`, `qmllint`, the fence; 3). Seeded faults, each
applied alone and reverted: Rec. 709 weights for the luma failed the saturation columns; the mask measured on the
box's width alone failed the bottom edge's middle (the first form of the test, on a square picture, could not see it,
which is why the box is 192 × 128); `nearest` ignored failed the pixel-exact strip.

**Beside the C# painter.** Art Book Next's system view in its dark scheme tints each slice of its carousel `ffffffdd`.
The SNES slice (`_inc/systems/artwork-screenshots/snes.png`, 454 × 1080, contained at 336 × 800) drawn by `Picture`
with that tint, against piece B's C# grab and ES-DE 3.4.1's capture of the same state, over the slice's opaque
pixels clear of the logo and the help bar (117,347 pixels): **mean absolute error 0.10 levels against the C#, 0.69
against ES-DE** (the C# painter's own figure against ES-DE over the whole state is 0.84, `Testing/esde-baseline/mae.tsv`).
The sheet is `~/.cache/emusen/tmp/qt-s4/compare/e1-slice-esde-csharp-qt-diff4.png` (ES-DE, C#, Qt, the difference ×4).
The watched states hold no desaturated and no rounded picture, since Art Book Next's dark scheme keeps saturation 1
and its rounded cover is not in the variant measured, so those two are held to LunaP's arithmetic by the numbers
above, not to a grab.

#### 4.8.3 The painter without Qt: `shelves.rs` and `painter.rs` (E2, first part)

What the bridge will put on Qt, written and tested as plain Rust first, so the behaviours below are held without a
Qt application.

- **`shelves.rs`** reads §4.6's texts. `settings` takes `esdeSettings`: the theme folder, the choices under Mistress's
  keys (`Variant`, `ColorScheme`, `FontSize`, `AspectRatio`, `Language`, `Transitions`, as `BigPictureChoices` stores
  them) and the interface's switches (`DisplayClock`, `DisplayHelp`, `ListScrollOverlay`, `QuickSystemSelect`,
  `StartupView`); a text that does not read gives the defaults. `shelves` takes `esdeShelves` and `esdeMedia` into the
  scene's `ThemeSystem` and `Game`s: a collection's entry, carried by its file alone, takes the row its console's shelf
  carries, and its `source` is that console's system; a game's media is looked up by its folder and stem, then its
  stem, under its own console (§4.6); `last_played` (seconds since 1970) becomes a civil date in UTC by Howard
  Hinnant's days-to-civil; `release_date` reads the metadata's `yyyy-MM-dd`. `presence` takes `esdePresence`.
- **`painter.rs`**: `ThemeCache` keeps a resolved theme per (folder, system, choices, media presence), as
  `ThemedLibrary._themes` did, behind a mutex so the data thread fills it, with `forget` for a folder updated on disk;
  `systems` is the data thread's work, shelves and their themes. `Painter` holds the `Stage`, its clock in
  microseconds (`tick(ms)` adds), the latest `Frame`, and the pad's grammar on the view, the C# `ThemedLibrary.Command`'s:
  a direction along the primary element's axis (a list's and a vertical carousel's vertical, a grid's both) presses
  the scene; Left and Right across it change the system in a gamelist when the quick system select names them, as do
  the shoulders or the triggers it names; otherwise the shoulders jump ten and the triggers to the first and last; A
  opens the gamelist or plays the game; B goes back from a gamelist and does nothing at the system view (decided
  2026-11-08); Y is the favourite; Start and Select are the host's menu and options. `install` keeps the view and the
  selection across a refresh.
- **The hold is the scene's.** The router's repeats are not passed on: the scene repeats a held direction on its own
  clock, and only the scene knows the measured consequences of a hold (the metadata fading out from the first repeat,
  the list's fast tier jumping four entries). The router gives the first press and the release.
- **The pointer makes the pad's calls.** `hover(row)` steps to the row as the pad's step does, `tap(row)` is that and A,
  and a wheel notch is one step, back for a notch up.

**Proof**, `tests/painter.rs` (6, on a synthetic theme the test writes, no Qt application): the settings under
Mistress's keys and a text that does not read; a collection's entry and the media by folder then stem, the two dates;
the cache read on the second refresh while the file changed, and read again after `forget`; a carousel step moving at
200 ms, settled after 401, and its frame equal to the static frame of the new state; the router's repeat not a second
step, the scene's hold reaching the fourth game in 960 ms, and nothing moving after the release; hover, tap and the
wheel making the same selection and the same `Played` as the pad, and B at the system view doing nothing. Seeded
faults, each applied alone and reverted: the router's repeats passed on failed the hold test; hover one row off failed
the pointer test; nothing cached failed the cache test.

**Owed to the rest of E2** (the bridge, `View.qml`, the delegates, the GPU-tier proofs and the side-by-sides of the
directions), recorded where the work stands: the description is not read yet (`LibraryModel.esdeDescription(file)`
is a call the bridge makes for the selected game); the system view's random gameselector picks; the cue's sound files.

#### 4.8.4 The bridge and `View.qml` (E2)

**Five objects, one painter.** `EsdeModel` (`src/model.rs`) holds the `Painter` of §4.8.3 behind an `Rc<RefCell<…>>`
on the GUI thread. The four list models (`src/lists.rs`) are given it in QML as a property, `scene: esde`, a
`*mut EsdeModel` property of cxx-qt 0.10 (it compiles across bridges by naming the other bridge's type in an
`extern "C++Qt"` block, without `#[qobject]`); on assignment a list takes its own share of the painter and keeps no
pointer. Each list binds `revision: esde.revision` and refreshes on its change, so no `Connections` is needed.

| Object | Holds | Changes |
|---|---|---|
| `EsdeModel` | writable `settings`, `shelves`, `media`, `presence` (the texts of §4.6), `width`, `height`, `swap`; read-only `themed`, `problem`, `view`, `moving`, `nextChangeMs`, `revision`, the quick-scroll `letters`, `lettersMark`, `lettersPx`, `lettersOpacity`; invokables `tick(ms)`, `pressed(button, repeat, jump)`, `released`, `hover(row)`, `tap(row)`, `wheel(notches)`, `measured(id, w, h)`, `reload()`; signals `played(path)`, `favourite(path)`, `menu`, `options`, `cued(name)` | a text or the size changing starts a load on a thread of its own (the texts read, `painter::systems` through the `ThemeCache`), handed back by `qt_thread().queue`; a load older than the last asked for is dropped |
| `ElementModel` | one row per `Frame.elements`, roles flattened by `src/values.rs`, the `paint` role a `PaintKind` enum (`ElementModel.Picture` … `Grid`) the chooser reads, the primary's layout flattened onto its own row | a reset when the ids, paints or layouts change (a view switch, a system change); otherwise `dataChanged` for each row that differs |
| `SlotsModel` | a carousel's slots (current or `leaving`) with their row's text and picture | a reset when the count or the rows change; `dataChanged` per moved slot |
| `RowsModel` | a list's or grid's rows | replaced under a reset only when the `Rows` or the install changes; a step changes none |
| `PartsModel` | one element's badges, help entries (`button`, `pair`, `label`, `icon`, as `Kit.HelpBar` reads them), info or status items | a reset when the count changes, else `dataChanged` over them |

- **Ids are text.** An element's id is a 64-bit FNV value; QML numbers are doubles, so the `elementId` role and
  `measured`'s first argument carry it as decimal text.
- **Paths become URLs in Rust** (`QUrl::fromLocalFile`), since QML builds no strings (§1.3).
- **A role never shadows an item's own property.** The box is `boxX`, `boxY`, `boxWidth`, `boxHeight`, `turn`,
  `alpha` (qmllint refuses `left` and `top`, final members of `QQuickItem` in Qt 6.11); the help's scale is
  `entryScale`.
- **The help's buttons.** The scene names a help entry by its place (§4.5.5); the kit's `PadGlyph` draws
  `UiButton::control(swap)`, so the bridge hands it the press of that place swapped, `help_buttons` in `painter.rs`,
  from kit-core's own `PadControl::press` table. The crate now depends on `emusen-kit-core` (as the design's §1 lists),
  and the painter's own copy of the button order is gone: it takes `UiButton::ALL`.
- **The router's tiers are not bound** (decided 2026-11-08). The scene repeats a held direction on its own clock
  (§4.8.3), so the router's repeat tiers would only produce repeats the painter ignores.
- **The cue is a signal**, `cued(name)` with the scene's name (`navigate`, `select`, `back`, `quicksysselect`), since
  two equal cues in a row must both sound; mapping a name to the theme's sound file is the host's (owed with H).

**`View.qml`** is a plain `Item` filling its parent: an `EsdeModel` sized by it, an `ElementModel`, and a `Repeater`
whose delegate is a `DelegateChooser` on `paint`. `DelegateChooser` is in `QtQml.Models` since Qt 6.9 (measured on the
desktop's 6.11.2: `QtQml/Models/plugins.qmltypes` exports `QtQml.Models/DelegateChooser 6.9`), so `qml-rules` no longer
refuses `import QtQml.Models` (decided 2026-11-08); a `ListModel` with rows is still a finding by the rows rule, and
`role` joins the bindings that take a name. The delegates, one file each: `PictureElement`, `TextElement`,
`RatingElement`, `BadgesElement`, `InfoElement`, `StatusElement`, `HelpElement` (the kit's `HelpBar`, fitted, unruled,
in the theme's face and colours, filled glyphs; a press of an entry is `pressed(button)`), `CarouselElement`,
`TextListView` and `GridElement`, over `ElementBox` (the box and its turn) and `Icon` (a theme file or a mark). The
text list's file is not `TextListElement`: `qml-rules` reads any line containing `ListElement` as rows.

- **The clock.** A `FrameAnimation` runs while `moving` and calls `tick(frameTime × 1000)`; a repeating `Timer` of
  `nextChangeMs` runs while nothing moves and something will. At rest neither runs and the window swaps no frame.
- **The pad and the keyboard.** `Connections` on `Kit.PadRouter` while the view has focus (`pressed`, `released`);
  `Keys.onPressed` is `PadRouter.pushKey`.
- **The pointer.** Each carousel slot, list row and grid cell carries a `HoverHandler` (`hover(row)` on entering) and a
  `TapHandler` (`tap(row)`); a `WheelHandler` steps one item a notch; a right or back click is B.
- **A carousel slot** is an item filling the element's box, scaled about the growth point and turned about the wheel's
  centre (the scene's origins are in the element's box, so neither needs translating), its `Picture` at the slot's box.
  A reflection is a second `Picture` drawn `flipped` (a new `Picture` property: the crop rectangle read upside down, so
  the ramp of §4.8.1 still starts at the drawn top, the picture's bottom row) and placed under the painted picture.
- **A grid cell's level** is the scene's `focus_of`, evaluated in a binding from `focusFrom`, `fromLevel`, `toLevel`
  and `progress`; scale, opacity, dim and saturation follow from it.
- **Measurement.** A picture reports its own size when it changes, a text each new extent, a list its selected name's.

**Proof.** `tests/view.rs`, one test on the GPU tier (Vulkan on the RX 6800 in `kwin_wayland --virtual` at 3840 × 2160,
the validation layer on, Qt's log required quiet), over a synthetic theme (a text carousel of three systems, a text
list and the selected game's name) with 4,000 games on one shelf, measured on 2026-11-08: the theme loads with no
problem; the system view at rest swaps no frame in 2 s and asks for no wake; a carousel step is moving at 150 ms,
swaps frames while it glides, rests, and its last frame equals, pixel for pixel, the static frame a reload of the
same state draws; A opens the gamelist; the list makes delegates for the rows that fit its box (within one), not for 4,000; a step selects
the next game and its name in the same frame with nothing gliding; a hover selects row 3, a wheel notch down steps to
row 4, a click selects row 3 again and a right click goes back to the system view. Repeater's delegates have no QObject
parent, so the test reads them through the item tree (`Harness::tree`), never by `findChild`. In `tests/painter.rs`
(8): the help's places under the swap, and 4,000 games: install and rows 3.7 ms, a step and a tick 0.11 ms (debug
build). A first run found a defect the plain-Rust tests could not: a list held the painter borrowed while it signalled
its rows, a text delegate measured itself in that signal, and the second borrow aborted; every list now copies what it
needs and releases the painter before any signal, and `measured` publishes from the event loop.

| Seeded fault, applied alone and reverted | Caught by |
|---|---|
| the `FrameAnimation` running always | `assert_still`: 119 frames in 2 s at rest |
| `ElementModel` emitting no `dataChanged` | the metadata still "Game 0000" after the step |
| a list row's hover one row off | the hover selecting row 4 |
| the help's place not swapped | `a_help_entry_s_place_is_the_kit_button_that_draws_there_under_the_swap` |

**Owed, named:** at the GPU tier, the held list's metadata fade and the quick-scroll letters (the harness presses and
releases at once; holding needs a down-only press in `qt-test`), `scrollFadeIn`, the slide with the overlays held
still, a wheel's logo centres (§36.3) and the `(default)` help and status places (§39.6): each is held in the scene's
own tests (`Frontend/esde/tests/scene.rs`, `frame.rs`), and in the painter they are bindings of the same roles. The
side-by-sides of Art Book Next beside piece B's grabs need the oracle's synthetic library and are left for piece C's
rig. The scene allocates its `Frame` each tick and the lists copy what changed; the cost of both is measured with H.

#### 4.8.5 The mapping, the five sizes and the motion on the GPU tier (E3) **[M 2026-11-08]**

Three GPU-tier tests, each in a `kwin_wayland --virtual` of its own on the RX 6800 with the validation layer on, Qt's
log required quiet; what they share is `tests/common/mod.rs` (the compositor, a written theme, BMP and GIF writers so
no image crate is needed, the oracle library read as the three texts of §4.6).

- **`tests/motion.rs`** (green): the slide pans the coming list while the clock keeps its box exactly and the help its
  place (its entries are the coming view's at once, so only its width may change);
  a step fades a `scrollFadeIn` cover in from half (0.45–0.95 at 60 ms, 1 when the motion ends) and the view's opening
  does not; a held Down (the harness's `pad_down`, EmuSen_Kit.md §7.28's router left holding) keeps the developer text
  at 1 at 300 ms, 0 at 800 ms (the first repeat at 500, the fade 149), frames swapping, and 1 again after `pad_up`.
- **`tests/mapping.rs`** (green): the C# `SceneMappingTests` translated, its cases as data
  (`tests/themes/mapping-cases.tsv`, the shared properties made from the catalogue as the C# `Common` made them), the
  C# case library as `esdeShelves` (five consoles, an automatic collection, a console whose sixth game is a folder, a
  custom collection, a console of pads), each case's system and game rotated to the front so the view starts on them.
  Each case is rendered twice through the bridge at 640 × 400 and the two grabs must differ; each case's two themes
  load without an error; every drawn pair has a case, and a case's pair is drawn or later. 347 cases, 254 s.
  The first runs found 47 drawn pairs that painted nothing, and each was settled one way or the other:

| Found | Settled |
|---|---|
| `grid.selectorLayer`: the role compared the layer with `"boxY"`, a rename of `top` that reached a string | fixed in `values.rs` |
| `helpsystem` and `systemstatus` (24 pairs): neither delegate reported its extent, so both stood in a 0 × 0 box and the origin, the background and its paddings had nothing to act on | both delegates report their row's size through `measured`; the status background grows by its paddings |
| `TextBlock`: a vertical container bound the text's height to its own implicit height, a binding loop Qt logged | the height is left to the text (`undefined`) while it scrolls vertically |
| `textlist.indicators`, `collectionIndicators`, `grid.imageCropPos`, `image.tileVerticalAlignment` | the fixtures: the shelves now carry `stars`, a custom collection's games `in_collection` (which `shelves.rs` now reads; qt-models writes none yet, so the tick waits for collection editing), and picture A is four quadrants, so a vertical crop or tile shows |
| the gameselector (13), a GIF's timing (3), `fractionalRows`, `containerVerticalSnap`, `containerScrollSpeed`, datetime `lineSpacing` | not drawn: marked `later:` in `mapping.tsv` with their families (§4.5.5), 367 → 347 drawn |

- **`tests/sizes.rs`** (green, measured 2026-11-08): the three watched themes (`Testing/esde-baseline/states.tsv`), both
  views (the SNES; its first game, the folder being entry 0), at the five `SIZES`, on the oracle's library with the
  systems in the order ES-DE showed them and its counts (every game of a system, those inside folders too: 14 games and
  7 favourites on the SNES, as ES-DE's captures read). Held: nothing drawn but the elements, the help's entries and the
  primary's items inside the window and taking the pointer, a step and A and B answered, the pointer selecting, and
  every element's box ÷ (W, H) equal within 0.001 across the sizes of one aspect ratio. A box sized by its laid-out
  words (a help bar, a status row, an auto-sized text) may differ by a further 4 % of its extent: each run's advance is
  rounded to whole pixels and hinted at its own size, so a 14 px bar is not two thirds of a 21 px one (CarAlt's help
  measured 3.3 % apart at 1280 × 720 and 1920 × 1080), and ES-DE's own is not proportional either (Art Book Next's
  gamelist help 0.2883 W at 1280 × 800, 0.2906 W at 1920 × 1200, 0.8 % apart, on its captures). A size is read only
  after its install has published, since the old frame stands until the new size's load comes back. The defects it
  found, each fixed and proved:

| Found | Fixed |
|---|---|
| A reload or resize installed a new `Stage` with no measurements, and a picture, text or bar whose extent had not changed never reported again, so Art Book Next's date, players and play time, its icons and Artflix's overlay stood at width 0 | `Stage::restart` keeps the measurements while a new view and data are installed (the painter's reload, resize and quick system select use it); a text reports again on new words, size or face, and a picture once it is ready, since the scene keys a measurement by what it shows |
| An auto-sized text was elided in its 0 box, so it measured nothing and was never given a box (the chicken-and-egg of §4.8.5's first run) | an auto-sized text has no ellipsis (`paint.rs`) |
| A vector picture contained or covered was decoded at both of the box's sides, so Qt stretched it to them and the scene measured the box's aspect, not the picture's (Art Book Next's system logo 0.2167 H high at 1280 × 800, 0.35 H at 1920 × 1200; CarAlt's controllers and screenshot) | a contained or covered picture asks only a width (a cover twice its height), so the decoded picture keeps its own aspect |
| The help bar's glyphs were the kit's pixel size and its paddings moved the entries in | the glyph's side is 0.9 of `fontSize` × H (ES-DE's icon measured 30–32 px at a 36.7 px font on 1280 × 800, 45 px at 55 px on 1920 × 1200); `entryRelativeScale` shrinks the icons above 1 and the text below 1, as THEMES.md has it; the background grows outward by its paddings and leaves `pos` and `origin` alone; the kit's `HelpBar.glyphSize` and the glyph's letter growing with its glyph (EmuSen_Kit.md §7.28) |
| A carousel item drawn as text (no picture) had no `slot-` name, so neither the test nor a pointer test could find it | the text item carries the slot's name |
| An item gliding under a still pointer took the selection, and the carousel chased the pointer for ever | hover selects only while the items are at rest; a tap selects at once, glide or not (`painter.rs`) |
| The run of 2026-11-08 at the end of E3 (20 failures): every contained or aspect-sized picture lost its box at a size after the first (Art Book Next's system logo 0.35 H high where 0.2167, its play-time, players and release-date icons and Artflix's wide overlay at width 0, CarAlt's controllers, cover and marquee), because the scene keyed a picture's measurement by its paint and its box, both of which change with the window, and a delegate reports again only when Qt's image changes status or height, which a cached decode may not do | a picture sized by its own aspect (`maxSize`, `size` with one axis 0, no size) keys its measurement by its file alone (`draw.rs`), so the measurement holds across a resize and a reload; text and rows keep the key by paint and box. Proved by `frame.rs`'s `a_pictures_own_size_holds_across_a_resize`, which fails on the old key with the very boxes the run showed |
| Art Book Next's help a see-through strip at every size after the first, its glyphs white on nothing | the kit's `HelpBar` draws no backing when its colour is clear (EmuSen_Kit.md §7.28); every grab is now sampled for alpha below 255 on a 32 × 20 grid and along each help bar |
| "The pointer over an item selected nothing" on Art Book Next's gamelist | the test's: it hovered the second-nearest item to the middle, which after the pad's step was the selected one. It now hovers the two nearest, of which one at most is selected |

  Two were the fixtures': the oracle's media index listed a folder's own media subfolder as Tower Set's picture, and
  its shelves carried no counts.

**The side-by-sides, re-made 2026-11-08** (`~/.cache/emusen/tmp/qt-s4/compare/side-*.png`: ES-DE 3.4.1 | C# | Qt;
`sizes-*.png`: the five sizes of each view). Every difference found, against ES-DE's captures and its published
THEMES.md and USERGUIDE.md, settled or named:

| Seen | Settled |
|---|---|
| The help's words: LAUNCH, SYSTEM on the system view, JUMP, FIRST, LAST and CHOOSE in a gamelist, FAVORITE | ES-DE 3.4.1's entries in its order and words (`help.rs`): the system view MENU, SELECT, SCREENSAVER, CHOOSE; a gamelist OPTIONS, MENU, SELECT, BACK, FAVORITES, SYSTEM, RANDOM. A reads Select on every entry, as the user guide's "A button" and the captures have it. The project's words of `EmuSen_BigPicture.md` §15 stay the C# painter's |
| No SCREENSAVER or RANDOM entry | the help reads the player's *Enable screensaver controls* (`ScreensaverControls`, on by default) and *Random entry button* (`RandomEntryButton`, games by default), and resolves *Quick system select*'s two "or" settings by the gamelist's primary (UG "Quick system select") |
| Wi-Fi, a battery and 80 % drawn where ES-DE drew none | the view shows no indicator until the host reports the device on `EsdeModel.device` (a JSON text, `{"wifi":true,"battery":80}`; an addition to EmuSen.Esde 1.0's registry, copied to `Testing/qmltypes`) |
| CarAlt's title screen, screenshots and marquee tiled into strips | THEMES.md: `tile` "can only be used when `path` has been set", so a game's media is never tiled (`paint.rs`) |
| Art Book Next's Tower Set drawn with a folder where ES-DE draws its folder-link mark | a folder with a link has the `FolderLink` marker (`primary.rs`); the mark is the scene's link, where ES-DE's symbol is an arrow into a bracket: **named, open** |
| The list's names indented on a row with no marker (Brass Lantern), the selection bar beginning at the name, the star larger | ES-DE prefixes the marker to the name: a row with none starts at the margin, the bar covers marker and name, the marker is one `fontSize` square |
| The list's rows 6–8 px lower than ES-DE's | a row's name takes the face's own line height, not the list's `lineSpacing`, which is the rows' pitch |
| Artflix's clock missing | ES-DE draws a clock "in the upper left corner" where a theme has none (UG "Display clock"); the loader adds one with THEMES.md's defaults, as it adds the help and the status |
| Artflix's game count read "12 games available, 6 favorites" | THEMES.md's `gamecount`: "Number of favorites is printed inside brackets", from the shelf's counts: "14 games (7 favorites)" |
| Artflix's title reads SUPER NINTENDO where ES-DE reads NINTENDO SNES (SUPER NINTENDO) | not accepted as a difference: **owed to H**. The systems table's `esde_full_name` is the project's short name, not ES-DE 3.4.1's full name, a data change in `Platform/systems/data` rather than the painter's |
| The D-pad pair drawn as two glyphs (left lit, right lit) where ES-DE draws one cross | the kit has no glyph for an axis: **named, open** |
| No VIEW MEDIA entry in a gamelist | the media viewer is not built; the entry would name a press that does nothing: **named, open** |
| The help labels wider than ES-DE's | the kit's face where ES-DE uses its own (decided, design §12) |
| The clock's time | the wall clock at the grab |

---

## 5. The stable developer surface

### 5.1 Which surfaces, which rules, and when they freeze **[D 2026-11-07 on the timing]**

| Surface | For whom | Rule | Stable from |
|---|---|---|---|
| Core ABI v1 (`emusen_core.h` 1.19) | core authors | A: additive minors, never removed | already (2026-10-03) |
| Frontend API (`emusen_crystal.h` 2.0) | front ends and tools | A | already (2.0 on 2026-11-07, the twins removed) |
| libretro API | libretro core authors | libretro's own; EmuSen implements the host | theirs |
| DianaSh command modules | tool authors | DianaSh's ABIs (`EmuSen_DianaSh.md`) | that page's |
| `EmuSen.Theme` 1.x and `theme.json` | theme authors | **Q** + **T** (below) | **1.0**; before it, breaking changes allowed under the same rules and listed |
| `EmuSen.Kit` 1.x | themes (through `EmuSen.Theme`'s re-export of the types it allows) and the application | Q | 1.0 |
| `emusen-crystal-client`, `emusen-platform-client` | the application; later, Rust tools | R until 1.0, then semver with a `cargo public-api` baseline | 1.0 for the Crystal client; the platform client stays exact-match |
| `EmuSen.Models`, `emusen-esde`'s crate API | the application | R / internal | never promised |

### 5.2 Rule T: a versioned theme package

`theme.json` has a `schema` number (1) and an `api` version; a key is only added; an unknown key is ignored with a
`theme-check` note; a screen file or token name is only added. Deprecation: a key kept and reported for one minor, removed
at a major, the removal listed. Before 1.0 the same, with removals allowed and listed.

### 5.3 Rule Q: a versioned QML module

The module's `qmldir` version is major.minor; the `.qmltypes` file `qmltyperegistrar` writes for it is committed under
`Testing/qmltypes/<module>-<version>.qmltypes` and diffed in CI: a removed or changed type, property, role, signal or
invokable fails the build unless the major moved; an addition bumps the minor. Deprecation is a `@deprecated` note in the
registry for one minor. The same dump is the published reference for theme authors, with a generated page per type.

### 5.4 How third parties build themes and plugins

- **Themes**: a folder as §3.4, written against the published `.qmltypes` and `theme.json` schema, checked with
  `theme-check`, installed by copying into the themes folder or listed in EmuSen's theme list (the browser of
  `EmuSen_BigPicture.md` §25 generalised to both kinds, with the same licence line before a download). A theme's QML is
  the author's, under the licence they choose; the API imposes none.
- **Cores**: a shared library against the core ABI v1 (its conformance kit and sidecar), or a libretro core, which the
  host turns into one (§6).
- **Tools**: DianaSh command modules in Rust, which the front end's Shell window runs; a tool that wants a window fronts
  the command, never replaces it (the 2026-10-26 rule).
- **No plugin into the front end's process other than themes.** A QML plugin with native code, or a Qt plugin, would be a
  second kind of code running inside the application with no surface; the form for such a feature is a DianaSh command
  or a core, and §11 Q8 asks whether that stands. **[A]**

---

## 6. Adaptive cores, end to end

### 6.1 What declares what

| Fact | EmuSen core (core ABI v1 `info` and `machine_info` JSON) | libretro core (through the host) | Front end reads it as |
|---|---|---|---|
| Name, version, licence, authors | `info`: `name`, `version`, `license`, `authors`, `description` | `.info`: `corename`, `display_name`, `license`, `authors`, `version` (`machine.rs`) | the engine line, the Cores window, the approval words |
| Systems it runs | `info.systems[]`: `id`, `name`, `extensions` | `.info`: `systemid`, `database` joined to the systems table; else a system `libretro-<stem>` named by `systemname` | a console on Choose a console; a shelf; a route |
| Extensions | per system | `.info` `supported_extensions`, narrowed by `libretro-extensions.tsv` | which files land on which shelf |
| Controllers | `info.systems[].controllers[]`: `id`, `label`, `ports`, `buttons` (bit, control, label), `axes` | `SET_CONTROLLER_INFO` (the device per port), `SET_INPUT_DESCRIPTORS` (labels per port and id), `GET_INPUT_MAX_USERS`; the RetroPad as the canonical pad | the bindings page and its drawing; the help glyphs; how many players |
| Firmware | `info.systems[].firmware[]` | `.info` `firmware<n>_path/_desc/_opt`, less what the core carries | Setup's Firmware step; the firmware question; the Firmware tool |
| Settings | `settings_schema`: key, label, help, kind, default, scope, category, effect, advanced, restart | `SET_CORE_OPTIONS_V2` (categories, labels, choices, defaults) → the same schema, through the host | Settings ▸ Consoles ▸ <console>, a game's Game Options, the Graphics page |
| Saves and cards | `machine_info` battery files with kinds (`game`, `removable`, `console`) | `GET_MEMORY_DATA`, `SET_MEMORY_MAPS`, named files (E3), the save directory | the Save media pages, the cards' two doors |
| Picture shape | `machine_info` display: width, height, aspect, rows, repeat | `av_info` geometry and aspect | the picture's fit and the console's CRT profile (`ConsoleScreens`) |
| Rumble, presence, discs, states | capability bits and `machine_info` | the environment calls the host maps (`EmuSen_LibretroHost.md` §4) | which rows a card shows |

### 6.2 What is derived, and its defaults

Dropping a core in (an EmuSen core's folder with its sidecar, or a libretro core's folder as `EmuSen_LibretroHost.md`
§1.1), the front end derives, with no code per core:

1. **The console** appears in Choose a console, the grid's sections and Settings ▸ Consoles when it has at least one game
   (`EmuSen_Interface.md` §24.1), named by the systems table's `display_name` where the table knows the system, else by
   the declaration's name. Its **hue** is the table's where set, else a hue from the id's hash, the same on every start
   (§22 there); its **body** is the maker's (Nintendo a planet, Sega shards, Sony a gem) where the table names a maker,
   else the plain planet. Its **cover shape** is the table's `cover_aspect`, else 1.0 (a square card for a game with no
   art); real covers are drawn whole at their own shape regardless (§21).
2. **The engine line** reads the engine's `name` and kind from the cores listing, with the mark `EngineMarks.Of` gives it.
3. **The shelf and the route**: a file lands by extension, with the table's extensions and the declaration's.
4. **Controls**: the bindings page draws the console's controller from the declaration (buttons with labels and bits,
   axes, ports), on the generic controller drawing with the labels positioned by the kit's `NormalizedCanvas`; a console
   whose controller is the RetroPad gets the RetroPad's drawing. *Gap found 2026-11-07:* the libretro host collects
   `SET_INPUT_DESCRIPTORS` but builds the pad with the RetroPad's names (`machine.rs`, `pad(ports, &[])`), so a hosted core's
   own labels ("Cross", "Circle") are not yet shown; a host stage maps them, after which the page needs no change.
5. **Settings pages** are generated from the schema: one row per setting in its category, the control by kind (choice →
   `ComboBox`, boolean → `Switch`, number → `Slider` or `SpinBox` with the schema's range), `advanced` under a disclosure,
   `restart` with its note. The six sections of `EmuSen_Interface.md` §13b stay as they are; the console's page within
   them is schema-driven already in C# and stays so.
6. **Firmware**: a needed file is asked for in Setup and at launch, from the declaration, with the open replacement's
   status (`EmuSen_Cores.md` §3.9), never fetched.
7. **Saves**: the kinds decide the doors; a removable card declared becomes a Cards page.

Defaults are the table's values where the table has a row, else the derivations above. **Overrides** live in one place:
`Platform/systems/data/<system>.json`, the systems table, which is data read by the platform library and the one owner of a
console's display name, maker, year, cover shape, ES-DE names, ScreenScraper id and cheat systems; `libretro-cores.tsv`
and `libretro-extensions.tsv` beside it hold the hosted cores' listing and the named exceptions. A console not in the
table is not second-class: it shows and plays with the derived defaults, and a table row is an improvement someone adds
later, not a prerequisite.

### 6.3 The path from dropping a core in to playing

1. Discovery (the runtime's cores listing, `emusen_crystal_cores`): the folder is read, the sidecar or `.info` parsed, the
   trust standing decided (manifest, approved, needs approval).
2. Approval, where needed, in Cores or at the first launch: the words are the declaration's name, authors, licence and
   hash, and nothing is loaded before "Approve".
3. The systems the declaration runs are joined to the table and to the library's shelves; the front end's `LibraryModel`
   refreshes from `systems` and `shelves` (one listing, kept until `Forget`).
4. Choose a console shows the console; its engine line reads the choice (`Setting(console, "Engine")`, else the default).
5. A press of A on a game routes the file (`route`), opens a session (`open`), and the picture arrives through the ring or
   the copy; the pad reaches the core through Endymion by the canonical controls, with no mapping table of the console's.
6. The first launch's firmware question, if any, comes from the declaration.
7. Settings ▸ Consoles ▸ <console> and the game's Game Options already list the schema.

What a stage adds per core is therefore the core itself and, optionally, a systems-table row. The front end's code does
not change; `RuntimeCatalogEngineTests` (today) and the view-model tests (§8) hold it with a synthetic core whose
declaration the tests write, which is the proof that no console is hard-coded.

---

## 7. The migration: screen by screen beside Mistress

**Form.** The Qt front end is a second program, `emusen-qt` (working name), reading the same Galaxia models and settings
files, so switching costs nothing; listed in the C# front end's Settings as a preview while it lacks screens; the default
once every screen passes its gate; the C# front end deleted in one landing after that. Two toolkits never share a
process. Each stage lands with its screens' recordings matched (§8.2) and its gates passed (§9.3), and re-records nothing
of the C# side. Estimates are weeks of work, one person each unless said.

| # | Stage | Weeks | Gate | Retires |
|---|---|---|---|---|
| 0 | `emusen-kit-core`, `emusen-kit-qt`, the style, the two client crates, `emusen-qt-models`' first models, the pad bridge, the test harness and the spec writer, the AppImage of the program | 6 | the fit audit, pad reach, pointer reach and the style sweep green on the kit's gallery | nothing |
| 1 | **The first vertical slice**: Moonrise, Home, Choose a console and the grid in the reference theme (the mockups), a game through the ring, the Pause Card's first page; the gates measured on both machines | 4 | §9.3's gates; the Home, Choose a console and grid recordings of the C# side matched | nothing yet (preview) |
| 2 | The Pause Card's pages, the Home card, the List card, the question form, notices, the docked keyboard | 2 | recordings | `MainWindow.PauseCard`, `PadMenu`, `OptionListWindow`, the sheets |
| 3 | `EmuSen.Theme` 1.0 and two or three console-style themes (§3.6), `theme-check`, the Themes tab for both kinds | 3 | the four fit sizes and reach on every theme | the C# theme list |
| 4 | The ES-DE engine: loader and scene in Rust, the painter, `esde-check`, the corpus baseline | 6 (two people, loader beside painter) | the watched subset within the C# painter's difference; 66 of 66 load | `BigPicture/` (about 9,600 lines) |
| 5 | Settings (six sections, the controller drawing), a game's page and its five pages, the console pages, Cores and the tools' card form | 5 | recordings | the settings windows, `GameOptionsWindow`, the tools' hosts |
| 6 | The desktop form: library, menus, Preferences, the tools' desktop form, the Shell and `vstop`, the Debugger | 5 | recordings | `MainWindow.*`, `LibraryWindow`, `HotaruWindow`, `VstopWindow` |
| 7 | The Flatpak manifest and the Discover listing (§9.2); Windows and macOS builds; Android as a build | 3 (+2 Android) | starts on each | nothing |
| 8 | Retirement: the C# Mistress deleted, WiseMan's Avalonia tests retired for the spec comparisons and view-model tests, `EmuSen.LunaP` archived, Pegasus decided (§11 Q9) | 2 | the full suite green on Rust alone | Avalonia, SkiaSharp, the .NET runtime from the front end |

About 36 weeks of work; with two people where stages are independent (0 beside 4's loader; 3 beside 5), about 26
calendar weeks. The order puts the tester's first requirement first (the reference theme rendered to the mockups, stage 1),
the third next (the theme API proven by console-style themes, stage 3), the second after it (ES-DE, stage 4), and the
fourth is a property of the models from stage 0 on (§6), held by tests rather than a stage.

### 7.1 Stage 1 as built: the program and its screens (2026-11-08)

**The program, step 1.** `Frontend/qt-app`, the crate `emusen-qt`, its binary `emusen-qt` and the QML module
`EmuSen.App` 1.0 (`Main.qml`, the window; one file a screen). Before the window opens, `start.rs` reads two facts and
hands them to the window's root as its initial properties (`QQmlApplicationEngine::setInitialProperties`), so no static
carries them: the platform library's path (`--platform-library PATH`, else `EMUSEN_PLATFORM_LIBRARY`, else
`libemusen_platform.so` beside the program), and `AppSettings.StartupSplash`, read through Galaxia's model by
`emusen_qt_models::clients::app_settings`, the one function the library view's scan also reads the settings through. So
the program reads the player's own settings file, as Mistress does, and switching costs nothing. Its dependencies are
the kit, the models, `emusen-qt-endymion` and the platform client, all by path in the Qt workspace.

The window holds `LibraryModel` and `PadReader` once, the screens on the big screen's `NormalizedCanvas`, and Moonrise
over them. It has no menu bar and no status bar: the big-screen and card forms have none (§1.7). The Frame gained
`surfaceDrawn` (on by default), off on the night screens, whose sky is behind the frame.

**Moonrise at the start** (`EmuSen_Interface.md` §22, as the C# front end does it): its planets are one per system the
systems table lists with an engine, in the table's order (`library::splash_hues`), read by `LibraryModel` from the
systems before the scan of the games and published as `splashHues` and `splashCaption` ("N CONSOLES", `words.rs`), so
the scan runs beneath the splash. Since 2026-11-08 (§9.3.1, the real path) the splash's clock runs from the window's first
frame and does not wait for the systems: they are read in 2 to 15 ms, and the row of worlds is not due until 1.78 s
(the draft's 1.15 s at the pace of 1.55). It ends at the later of its own end and the library read
(`!loading`), then fades over the Fade token. A pad press, a key or a click skips it to its finished frame, and the
press is the splash's alone: while it shows it holds the focus, so the frame's `Connections` on the router are off.
With the setting off it is never shown. **Words**: `Words`, a QML singleton over `words.rs`, holds the screens' fixed
words, so a QML file names a word and never writes one (§1.3); `words::counted` writes "1,566 games" as the mockups do.

**The bodies** (decided 2026-11-08 by the tester, `EmuSen_Interface.md` §24.6): the splash draws each console's body,
as its tile does. `library::world_of` reads a console's body from the systems table's look, the one owner (its shape,
its three colours where the table gives three, its craters, bands, cap and moons), and writes it as the kit's world line
(`EmuSen_Kit.md` §7.18); `LibraryModel` publishes the lines of the systems with an engine as `splashWorlds`, the window
hands them to `Moonrise.worlds`, and every console's row carries its line as the role `world`. The draft stays the
splash's oracle for everything else.

**Reduced motion and the skip's keys** (measured 2026-11-08). `start.rs` reads `AppSettings.ReducedMotion` beside
`StartupSplash` and hands it to the window as `reducedMotion`, which sets `Kit.Tokens.reducedMotion`; the kit's
`Moonrise` then draws its last frame from the start and holds it for the splash's length, as Mistress's
`Reduced_motion_shows_the_still_frame_for_the_splashs_length` does (`EmuSen_Kit.md` §7.19). A key skips the splash only
when the kit's map makes it a press (§1.7: the arrows, Return and Enter, Escape, and the map's other keys), through
`MoonriseModel.takeKey`; any other key (Tab, Shift, a letter) is kept from the screens behind and skips nothing. A pad
press and a click still skip it. One difference from Mistress is named: the player's "system" choice reads the desktop's
own preference in Mistress (KDE's `AnimationDurationFactor` of 0, or GNOME's `enable-animations` off, read by
`MainWindow.SystemReducesMotion`), which has no Rust owner yet, so here "system" leaves motion on. **Held to**:
`tests/still.rs` (the program under reduced motion, compared with Mistress's finished splash while not yet finished,
then Tab, Shift and A leaving it and Return skipping it) and `tests/start.rs` (the setting's three values). Seeded
faults, each reverted: the frame drawn at the clock's time failed the picture at 126,229 pixels; the hold dropped failed
"held"; every key made a press failed at Tab; "system" read as on failed the settings test.

**Held to** (measured 2026-11-08): `tests/start.rs` (2: the library's path in its order; the splash on unless turned off),
`tests/program.rs` (the window over the real clients on a scratch tree with a synthetic NES image: Moonrise covers the
window, runs once the systems arrive with a caption counting them, a pad press finishes it, it goes once the library is
read, Home holds the focus and the library's selection is unmoved; nothing logged; the scenes keep `qml-rules` and
`qmllint`), and the library view's test of the planets (a system with no engine has none, in the table's order) and the
counted words; since the bodies, the worlds' lines in the table's order from each look, with the NES's palette and
a gem. Seeded faults, each reverted: every system given a planet failed the library test; the palette dropped by
`world_of` failed it; the splash
deaf to the pad failed the program test (the splash never finished); the setting read inverted failed `start.rs`.

**The mockups as numbers, step 2.** `Testing/mockups/Consoles.mockup.spec`, `Home.mockup.spec` and `Grid.mockup.spec`,
written by hand from the three approved canvases (`Courts.dc.html`, `Home.dc.html`, `Grid.dc.html`, 1280 × 800 design
units, approved 2026-11-05), and `emusen-qt-test`'s `mockup` module that reads them and holds a shown state to them
(§8.2) *[own, because the format is EmuSen's own]*. The format, `mockup 1`, is plain text: a `canvas NAME WIDTH HEIGHT`
line, `source` and `note` lines a reader reads and the comparison skips, and one `element` line per named element:
its objectName (quoted where it holds a space), the edges the canvas states (`x`, `y`, `w`, `h`, `right`, `bottom`; one
left out is free), a colour token by kit-core's name (`token=Muted`) or a colour that is no token (`ink=#RRGGBB`), the
paint's `alpha` with the item's opacity, and the words, exact (`text="GAMES"`) or the canvas's example data
(`words=data`, where only words being shown is held). The comparison finds each element by its objectName among the
visible items, divides the scene's rectangle by the design unit, and holds every stated edge within two units, a token's
colour exactly (an ink within a level a channel), an alpha within three levels, and the words.

How the boxes were worked out from the CSS, so a reader can check a line: a text's box is its line box, Barlow's and
Barlow Condensed's normal line height being 1.2 of the size (no half-leading); in a row aligned by baselines the tallest
item's top is the row's top and the others share its baseline (Choose a console's GAMES at 40 with its baseline at 72,
so "Choose a console" at 52); a grid's column is (width − gaps) / columns (the places 372, Home's places 152.57, the
grid's cells 211.2); a centred flex row starts at (1280 − its width) / 2 (six tiles of 176 with gaps of 18 at 67, five
at 164); a focused tile is lifted 6 (PlayStation 2's at 380) and its ring drawn 3 outside it; a body is centred in its
104-unit art box (a planet 58, a shard cluster or gem 42 × 68, the focused gem 60 × 96). The canvases' data is
example data (eleven consoles with their real library counts), so the words that are data are marked and the boxes
that follow from the count of consoles are stated for eleven: the acceptance test of step 4 shows the canvas's
eleven consoles. **One decision overrides the canvases**: the help bar stands flush on the foot, 742 to 800
(`EmuSen_Interface.md` §24.7, decided 2026-11-05), where the Grid canvas drew it at 702 to 760; every file says so in
a `note` line. The canvases' focus is stated too (PlayStation 2 on Choose a console, Continue on Home, the first cover
on the grid), so a state is compared in the canvas's own focus.

**Held to** (`qt-test/tests/mockup.rs`, 2, measured 2026-11-08): the three files read, each element once, the help bar
at 742 in each; and the comparison at 1920 × 1200 (a unit of 1.5 pixels) passing a title, a glass tile at 55% and a
data count where the canvas has them, and failing a title three units low, a tile painting Surface for SurfaceRaised, a
count with no words, an opaque tile for one at 55%, and elements not shown. Three seeded faults, each reverted: a
tolerance of five units failed the comparison test (the title three units low passed); the alpha not checked failed it;
the Grid's help bar written at the canvas's 702 failed the files' test. The canvases cannot be rendered here (their
`support.js` is not kept beside them), so the side-by-side pictures of step 4 draw each `.mockup.spec`'s boxes over the
built screen's grab, beside the C# grab of the same state.

**The screens' models, step 3's first part** (measured 2026-11-08). Everything Home, Choose a console and the grid show
comes from a model's roles; nothing is written per console:

- **The library view** (`library.rs`) reads, besides the systems and the games, which games are favourites and which
  were played (paths, the most recent first; the source's `recent` answered a count before) and **the engine each
  console runs now**, `LibrarySource::engines`: through the runtime's `Setting(console, "Engine")` with session 0 (the
  player's stored choice, else the engine row's default), else the first engine the table lists, named from the cores
  listing by `clients::engine_shown` (a libretro core by its corename, an EmuSen core by its info name less the port's
  "RT", `without_port_mark`, the named exception of `EmuSen_Interface.md` §24.6). This replaces the table's first
  engine, a difference from Mistress named at step 1. A row gained `fullName`, `status` (the count, "Your hearts, every
  console", "Last: " and the game played last or "Nothing played yet"), `facts` ("Nintendo · 1983 · 2 games · runs on
  Moon, an EmuSen core"), `engineName`, `libretro` and `countWords`; `LibraryModel` gained `summary` ("11 consoles ·
  1,566 games"). The words are `words.rs`'s, as Mistress's `ConsolesScreen` words them.
- **`GameModel`**: a list of games (roles `path`, `system`, `title`, `hue`, `favourite`) for the grid's cells, Home's
  shelf and Choose a console's fan. It is set from `LibraryModel.gamesOf(key)`, the games a row leads to (all and a
  console's by title, the favourites by title, the recently played the most recent first) as plain data, a JSON array
  (`games.rs`), so the library is read once and neither model reaches into the other. A title is the file name without
  its extension until scraped names are read. Since 2026-11-08 (§9.3.1, finding 1) every row's list is written once a
  refresh, off the GUI thread (`library::lists_of`, each game's cover, hero, record and console found by its path in a
  map), and `gamesOf` hands the written text back; each row also carries its first four games as the role `fan`, the
  covers Choose a console fans; and `GameModel` parses the text it is given on a thread of its own, taking the rows back
  through `qt_thread().queue` (the newest read wins). What the GUI thread still does in proportion to a list is copy its
  text twice as a grid opens (`QString` to and from UTF-8), about a millisecond for 9,000 games.
- **`HelpModel`**: a screen's help bar (`screen`: Home, Consoles, Grid) with the roles the kit's `HelpBar` reads, from
  `help.rs`: Choose a console's Open (A), Back (B), Favourites (Y, `Search`), Random game (X, `Screensaver`); the grid's
  Play, Game page, Favourite, Consoles; Home's Resume, Game page, Home card, as the mockups label them.
- **`Words`** gained the screens' fixed words (GAMES, Choose a console, GAMES ▸, CONTINUE PLAYING, RECENT, Home's seven
  places, the EMUSEN and LIBRETRO tags). The three list models declare `QAbstractListModel` once, in `library_model.rs`,
  as kit-qt's `base.rs` does, since cxx-qt emits an upcast shim per declaration and a second one clashes at link time.
- **The faces**: the mockups set names in Barlow Condensed Medium and words in Barlow; both are now in the kit's
  `fonts/Fonts.qrc` beside Moonrise's two, the C#'s own files (SIL Open Font License 1.1, the licence already in
  `licenses/`; SHA-256 of BarlowCondensed-Medium.ttf beginning `1da0b65f`, of Barlow-Regular.ttf `54786f6c`).

**Held to** (`qt-models/tests/library.rs`, 8, and `clients.rs`, measured 2026-11-08): the rows' words, facts and engine
from a fake source with an EmuSen and a libretro engine; the games of each row in order, read back whole through the
plain data, and the summary; an engine named by its own info with the port's mark dropped only after a lower-case
letter; each screen's help entries; and, over the real platform library, the NES's engine as the runtime's setting
gives it with facts ending "an EmuSen core". Seeded faults, each reverted: the engine taken from the table's first in
place of the source's answer failed the rows' test; the port's mark dropped from any name ending in "RT" failed the
engine test; the recently played reversed failed the games' test.

**The screens, step 3's second part** (measured 2026-11-08). `Home.qml`, `Consoles.qml` (Choose a console) and
`GameGrid.qml` (the grid; not `Grid`, which is Qt Quick's positioner and was taken in its place) are in `qt-app/qml`, with
`GameCard.qml` (a game without its cover: a card in its console's hue with its title), `CoverFan.qml` (up to four of a
row's games, fanned) and `Faces.qml` (the two Barlow faces). Each lays its content out in canvas units from the window's
corner inside a `Kit.Frame` with the navigation hidden, under its own `Kit.Sky`, and keeps the qml-rules and `qmllint`:

- **What a screen shows is a model's.** Choose a console is one `Repeater` over `LibraryModel`: the places first
  (`LibraryModel.places` counts them), then each console with games as its body; the tiles' boxes and the walk between
  rows are `Kit.PlanetLayout`'s (`EmuSen_Kit.md` §7.19), the body's size `Body.tileWidth`/`tileHeight`, the focused
  console's words its row's `fullName` and `facts`, the fan `GameModel` over the selected row's `fan` (four games). The
  grid's cells are a `GridView` over `GameModel`, its heading the row it opened; the two rows shown move only when the
  focus leaves them, a whole row at a time and at once, as LunaP's `GalleryLayout` keeps its rows (`KeepRowShown`), the
  view's `contentY` set from the first row shown (§9.3.1, finding 2). A cover is decoded at the size it is drawn
  (`RoundPicture`'s `sourceSize`, the canvas unit and the screen's pixel ratio), and released with its delegate. Home's Continue and shelf are
  `LibraryModel.recent` (the recently played as plain data, kept current by the model), its places `Words.homePlaces`,
  its status line `Clock.time` (`clock.rs`, `QTime`'s, ticked by a `Timer` every 15 s). An empty `games` text is no
  games, so a screen never writes a fallback list.
- **The walk** is the frame's: a tile is a `Kit.Walkable`, and `KeyNavigation` names its neighbours (on Choose a console
  from the layout's lists; on Home Continue, the shelf and the places). Each binding reads its `Repeater`'s count, so it
  is read again once the items exist. A walkable is named apart from the painted rectangle the mockup names
  (`tile.N` and `place.N` over `consoleTile.N` and `placeTile.N`, `continue` over `continuePanel`), so the comparison
  finds the paint, and Home's and Choose a console's names do not meet.
- **Switching screens** is `Screens` (`screens.rs`), one rule in Rust: Home's first place opens Choose a console, a row
  chosen there opens its grid, the favourites' action (Y) opens theirs, B goes back one screen and stays on Home. The
  other actions (X's random game, the game page, Home's card, Home's other six places) change nothing until their stage.
  `Main.qml` shows the screen `Screens.shown` names and hands each screen's signal to it in one call. A change is
  applied after the event that asked (`qt_thread().queue`): applied at once, the screen it opened took the same B press
  and went back a second screen, which the seeded fault below shows. The actions are declared as data (§7.19 of
  `EmuSen_Kit.md`): `HelpModel.actionLines` from `help.rs`, whose entries now carry their action keys.

**Held to** (`qt-app/tests/screens.rs`, measured 2026-11-08, beside `program.rs` whose `sandbox` and `wait_for` moved to
`tests/common`): over a sandbox of three synthetic games on two consoles, Home holds the focus on Continue; Down goes to
the places when nothing was played; A on Games opens Choose a console on the selected row; Down goes to the first
planet, Right along the row; A opens that console's grid with its own two games and its name; B goes back to Choose a
console on the same console; Y opens the favourites; B twice goes Home, and nothing is logged. The tree, not the
window's children, finds a `Repeater`'s items. `qt-models/tests/library.rs` holds `screens::step` and each screen's
declared actions read back by the walk's parser. Seeded faults, each reverted: the switch applied at once failed the
screens test at B from the grid; the favourites' action key misspelt failed it at Y. The kit's two are in `EmuSen_Kit.md`
§7.19.

**Step 4, acceptance** (measured 2026-11-08; `qt-app/tests/acceptance.rs`). The program over a sandbox of the canvas's
eleven consoles, one synthetic image each under its own folder with its first extension and ten on the Super Nintendo,
three played and one a favourite through the library's own changes. The fixture's engines shelve five of them (the NES,
the Super Nintendo, the Game Boy, the Game Boy Color and the Nintendo 64); the others have no engine beside the test's
platform library, or no image a scan routes without a real disc. Each screen is shown in the canvas's own focus
(Continue; the last console, standing for PlayStation 2; the Super Nintendo's first cover) and held at the gate's five
sizes: its `.mockup.spec` at 1280 × 800 and 1920 × 1200 (two units, token colours exact), the kit's gate audit (fit, pad
and pointer reach, no chrome, contrast, the palette, the floor) at every size, and nothing logged. Each state is recorded
as a spec and grabbed; `EMUSEN_QT_COMPARE` names the folder. Choose a console's boxes that follow from eleven consoles
(the console tiles, bodies, names, counts, ring and the second orbit) are held by kit-core's layout test against the
canvas's own numbers (`EmuSen_Kit.md` §7.19), not by this state of five.

Fixed on the way, each found by the comparison or the gate: Home's Continue read its game through an invokable, so the
binding never saw the games arrive (it now reads the count first); the grid opened with no current cover; the help bar's
entries carried the style's padding, so the first glyph stood at 80 where the canvases put it at 64 (`HelpBar` now sets
none); a generated card's border and an engine's name and tag were coloured from the console's hue, outside the palette
and under 4.5:1 at 1920 wide (they are now `rule`, `muted` and `text`); the grid's neighbours (the shoulders and the rows
beside it) were not built.

**Named differences from the mockups**, each with its reason, which the test lists in `named_difference` and nowhere else:
the help bar paints Surface opaque where the courts canvas had it at 92% (`HelpBar` is the kit's, one surface for every
screen); the orbit line keeps the canvas's own ink `#8C96DC`, outside the palette; at 1280 × 720 alone the canvas's 17-
and 18-unit words (a place's status, a console's count and engine) fall under the floor, as the canvas scales them by
the height; the focused console's name keeps the canvas's line box of 1 (44 units) where Barlow's is 53, which the gate
reads as cut at the foot though the capitals are whole; an engine's name longer than its tile is elided (the fixture's
cores call themselves "MoonRT (Rust)"); the fan's covers overlap, as a fan does.

**Compared with Mistress's recordings** (`EmuSen.WiseMan/Mistress/Specification/{Home,Consoles,Games}-*.spec`, and their
grabs beside the Qt grabs with the mockup's boxes drawn over them, in `~/.cache/emusen/tmp/qt-s1/compare/*-side.png`).
Where the C# and the mockup part, the program follows the mockup, so these differences are the approved design's: Home's
Continue panel at 80 and 304 high (C# 72 and 296) and its places at 632, 152.57 wide (C# 622, 150); the grid's covers as
208 × 146 cards in five columns from 64 (C# 180 × 240 cover cards in a 996-wide grid from 142); each help bar's entries as
its canvas labels them (the C#'s Home adds Favourite, Search and Home; its grid adds Console, List and Home). What the
program does not draw yet, named: the status line's controller count (the time alone); the crescent beside the wordmark
(a plain body); Continue's console and facts line; the places' status lines on Home and their icons on Choose a console
(plain discs); the engines' mark glyphs and hued tags; covers and favourite hearts (generated cards, until covers are
read); scraped titles (file names, sorted as text, so "Game 10" before "Game 2"); and the grid's heading names its row,
not the C#'s section. **Held to**: the test passes; seeded faults, each reverted: Continue's heading 4 units low failed
the mockup at both sizes; a card's border in its hue failed the gate's palette; the help bar's padding restored failed
the mockup at the first glyph.

**Step 5, the screens brought to the accepted render** (measured on 2026-11-08). The canvases cannot be drawn here (their
`support.js` is not kept and their faces are fetched online), so the oracle is Mistress's own look as the tester approved
it, recorded at 1280 × 800 and 1920 × 1200 with the help bar on the foot (the `courts-built` grabs, and
`EmuSen.WiseMan/Mistress/Specification/{Home,Consoles,Games}-*.spec` for their numbers). Decided 2026-11-08: where that
render and a canvas disagree, the render stands, and each `.mockup.spec` entry it supersedes is rewritten with the reason
in a `note` beside it (the grid's covers whole at their own shapes in 180 × 240 boxes, the tester's decision of
2026-11-04, Interface §21; Home's Continue at 72 and 296 high, RECENT at 376, the places at 622, 150 wide every 167 and
named in title case; the grid's focused words on two lines at 628 and 670).

- **The library is the render's own.** `acceptance.rs` builds Mistress's own-look fixture: the eight Synthetic games on
  the NES, the Super Nintendo and the Game Boy in one folder, their covers at their boxes' shapes in an ES-DE media
  folder named by `EsdeMediaDirectory` (each drawn by Qt Quick as `SyntheticLibrary.Label` draws it: the game's colour,
  a white rim, its console, title and size), each played three hours apart, two hearted, the installed engines as the
  sidecars an installation carries (nothing is loaded from them), and the tools' summaries as Mistress's fixture
  answers them. Home is held after Cobalt Harbor is started and the library read again, as the render's paused game.
- **Covers through the models.** `ClientSource` opens the scraper as a handle for finding pictures alone
  (`sources_alone`, the media store and the player's folders as Mistress gives them) and asks `emusen.scrape`'s `media`
  for each game's cover; `GameRow` carries it, and `GameModel` gives `cover` (a file URL) and `facts` ("Super Nintendo ·
  3 h ago", from games.db's `last_played`). `Cover.qml` draws a picture whole at its own shape on its box's foot with
  the heart, and the gallery's lift (1.06 and 6 units up) and ring; a game with no art keeps the
  generated card. The fan takes the canvas's last slots, so two covers stand at the right, turned 2 and 6 degrees.
- **The rest of the picture.** The faces' weights are named (Barlow Condensed Medium, the wordmark Light), since both
  files share a family and Qt chose the Light; the places' icons, the heart, the engine symbols, the sparkles, the
  crescent and the warning mark are the canvases' own paths, kept as data in `marks.rs` and read through the `Marks`
  singleton, since a QML file writes no words (`Glyph.qml`, `EngineMark.qml` with the project's pixel alien); an engine's name and tag are in its console's hue inside a `Kit.Scene` (`EmuSen_Kit.md` §7.20), its mark
  `engine_mark` (a sky-named core's symbol, else its maker's glyph, libretro's badge); the consoles follow the
  runtime's shelves; the grid's heading is a place's short name ("ALL") and its shoulders wrap; Home's places carry
  their lines (`words::home_lines`: the games, each tool's brief or what it does, the look and the sound from the
  settings, Exit; a tool's attention in amber with its mark; the tools' summaries come in through
  `LibraryModel.toolSummaries`, which nothing fills until each tool's stage), and the status line counts Endymion's pads
  (`Clock.controllers`, "1 controller · 21:42", the time alone with none). A generic pad's
  glyphs are drawn as the Xbox pad's lettered ones, Mistress's rule for these screens; the help bars name the render's
  entries, the shoulders as one.
- **Fixed on the way:** Continue read its game through invokables, so a second read of the same count of games left the
  old one shown; `GameModel.revision` now announces each read. Home's walk named the shelf's first cover by `itemAt`,
  stale after a second read; the cover now names itself as it is made.

**Differences that remain, each for the tester:** Continue shows the game's cover where the render shows the paused
frame and "Paused just now": no session runs in the test, and the frame belongs to `SessionModel` and `GamePicture`,
which Home does not read yet; the status line has no controller in the test, which reads no pad, and its time is the
clock's; the covers' corners are square where the render rounds them a little (a mask through `MultiEffect` hid the
pictures, so it was left out); the help bar's glyph stands 8 units from its word where the render has 10; the planets
and the focused body are the kit's drawing at its sizes.

**Held to** (measured on 2026-11-08): `acceptance.rs` passes, the mockups at 1280 × 800 and 1920 × 1200 and the gate at
its five sizes with nothing logged, and now also holds every grid cover drawn from its picture, each engine named by its
info less the port's mark with its own symbol, and Continue on the game played last after a second read; `program.rs`'s
QML rules and `qmllint`; `qt-models` 15, `kit-qt` 38, `qt-test` 9, `kit-gallery` 4 (the gate), `kit-core` 88; clippy
clean. The side-by-sides (the render left, the program right, nothing drawn over either) and their pixel differences
are `~/.cache/emusen/tmp/qt-s1/compare/*-side.png` and `*-diff.png`. Seeded faults, each reverted and each caught by
the acceptance test: the covers' lookup answering none (every cover read and drawn); Continue bound to the games' count
in place of `revision` (Continue follows the games read again); a sky-named core given its maker's glyph (the engine's
symbol).

**Step 6, the last differences closed** (2026-11-08).

- **Rounded covers** (measured 2026-11-08). The mask through `MultiEffect` drew nothing because the harness's first tier
  renders with Qt Quick's software backend (§8.1), which has no shader effects: on the software backend a `MultiEffect`
  is an empty box, and so is a `ShapePath.fillItem` (both tried in `qml` under `offscreen`; the second filled white). On
  Vulkan both round. The two ways were compared at 1.5 times in a compositor of the test's own: a layer with a mask
  renders the picture into a texture at the item's size in canvas units and is then scaled, so its covers came out
  soft; a `Shape` whose path is a `PathRectangle` with the radius, filled by `fillItem` with the `Image`'s own texture
  (`fillTransform` scaling the texture into the drawn box), draws its edges as curves at the window's resolution and
  samples the picture where it lies, with no texture of its own. `RoundPicture.qml` is the second: a picture whole in
  its box (the covers, on their box's foot) or filling it cut at the centre (Home's top, as Mistress's poster slot
  fills its box), radius `tileRadius` (8, Mistress's `CoverCard` for a big cover). On the software backend it draws the
  plain `Image`, square-cornered: that tier holds layout, focus and words, and the pictures are judged on Vulkan, where
  the acceptance test runs.
- **The help bar's glyph 10 units from its word**, the `IconTextSpacing` Mistress's help bands set (`HelpBar.glyphGap`,
  `EmuSen_Kit.md` §7.20).
- **A game in the program.** `emusen-qt` now plays: a game chosen on Home's shelf or the grid is recorded as started
  (`LibraryModel.started`, games.db's `started`, then the games read again), opened and shown; the menu chord pauses it
  and shows Home over it, Continue resumes it, and its screen gives way to Home when it stops (`Screens`: `Game`, `play`,
  `menu`, `gameRunning`; the rule is `screens::step`). Nothing of the play is copied: the game is `Game.qml` of
  `EmuSen.Play`, the session, the picture, the router's clock and the pad given to the game while it is in front, which
  emusen-qt-play's window now holds too, answering the chord with a pause there; and the device is
  `emusen_qt_play::host` (the device for Qt's platform, the plain preset's feed, the window's adoption as plain
  handles), which both programs start through and the tests use. A new dependency, stated here: `emusen-qt` on
  `emusen-qt-play`, the program on its game. `Main.qml` is shown by the program once its scene graph has adopted the
  device, as emusen-qt-play's window is; without a device the window still opens and plays nothing. The splash keeps the
  focus until it has faded: on the frame the skipping press finished it, Home took the focus while that press was still
  being delivered, which Continue had ignored and the first place did not (`program.rs`, "the press was the splash's
  alone").
- **Continue as Mistress's Home has it** (`HomeScreen.Continue`, `HubLayout.AddTop`, `MainWindow.GamePage`'s `Art`).
  Continue stands only while a game is suspended: its caption, its glass and ring, the planet, the title, and the facts
  "Super Nintendo · Paused 3 minutes ago" (`words::paused_ago`, Mistress's `PausedAgo`; `Clock.paused`, said again on each
  tick, counting from when `gamePaused` turned on), and the paused frame. The frame is the game's own `GamePicture`
  drawn into a `ShaderEffectSource` at the window's resolution, cut at its centre to the box's shape as Mistress's
  poster slot cuts the still, and filled into the same rounded `Shape`; one picture item names the feed, so the frame
  thread sees one item's swaps. The shelf leaves the suspended game out (`GameModel.leaveOut`). With no game suspended
  there is no Continue: the top shows the shelf's focused game (else its first) with no caption, glass or planet, its
  picture the game's hero art, its fan art, else its screenshot, else its title screen, else its cover
  (`library::HERO`, read from the media store for the played games alone), and the focus starts on the shelf, its name
  under it; with no game played, on the first place. The help bar follows the row focused, as `HomeScreen.HintsFor`
  gives it: Resume on Continue, Play on the shelf, Open on a place (`help::Screen::HomeShelf`, `HomePlaces`). A game
  the library does not hold (opened by its file) is named from its file (`LibraryModel.titleOf`), with no console in its
  facts and no planet. A game that starts is shown (`Screens.gameRunning`), so a start from outside the screens shows it
  too; the screens' rule stays in `screens::step`, QML holding no logic (§1.3: no function, no `Qt.callLater`).
- **One controller.** The acceptance test attaches an SDL virtual pad and names it to the reader alone
  (`program.onlyPads`), so the status line reads "1 controller · 21:42" as the render's.

**Acceptance on Vulkan** (measured on 2026-11-08; `qt-app/tests/acceptance.rs`, §8.1's second tier). The program runs as
a player's: in a compositor of the test's own (`kwin_wayland --virtual`, 3840 × 2160), its window adopting Serenity's
device on the RX 6800 through `host`, Qt on Vulkan with the threaded loop, the pads an SDL virtual pad alone. Four
states, each at the five sizes with the mockup at 1280 × 800 and 1920 × 1200: Home with no game suspended (held to the
gate; the render's `HomeNothingLeft`), Choose a console, the grid, and Home over a paused game. That game is a real
session on the test core, Cobalt Harbor's file replaced by the test core's image (the sandbox has no Super Nintendo
core), paused by the guide button on the virtual pad; Continue resumes it, and closed, Home has no Continue again. The
covers' window adopts the same device: a second Vulkan instance in the process faulted in Qt's swapchain release when
the engine was dropped, and each window is hidden before the harness goes, which releases its swapchain while its
adopted instance still lives. *Open:* on a failed assertion the unwinding drops the windows unhidden and the process
faults the same way, leaving its compositor running (its socket names the test's process); a watchdog such as
qt-play's `within` would stop it. **Differences that remain, each for the tester:** the paused game's console and planet
("Super Nintendo ·" and the gold planet in the render), its frame (the test core's dark picture, the render's orange)
and Home's game count (7, Cobalt Harbor's file being the test core's now), all from the test core standing in for a
Super Nintendo core; the clock's own time; the focused shelf cover's name meets the focus ring's foot, as the render
places both, and the gate's contrast sampling at 1280 × 720 reads the ring there (named in `named_difference`).

**Held to** (measured on 2026-11-08): `acceptance.rs` (as above, and the top's picture rounded at its corner, sampled
from the window); `program.rs`, `screens.rs`, `splash.rs`, `still.rs`, `start.rs`; qt-play's `device`, `pads`, `ring`
and `qml`; qt-models' `library` 12 (the hero art's order, the shelf less the suspended game, the paused words, the
screens' rule for the game); clippy clean. Seeded faults, each reverted: the shelf keeping the suspended game (the
models' test), the hero art's order with the screenshot first (the models' test), `RoundPicture` drawing square corners
(the acceptance test's corner sample, at 66,205,110 where the box's corner must show what is behind).

**Step 7, the Pause Card's first page** (2026-11-08; `EmuSen_Interface.md` §2.8, §5 item 4, §18.5, §20.6). Stage 1's
last screen. The oracle is Mistress's own card over a paused game: the recordings
`EmuSen.WiseMan/Mistress/Specification/PauseCard-{1280x800,1920x1200}.spec` and its accepted renders at 1280 × 800 and
1920 × 1080, plus the Leave page's (`PauseCardLeave`).

- **The chord opens it, as Mistress's `OpenPadMenu` does over a game.** Decided 2026-11-08: the chord over the game no
  longer shows Home. `Screens` gained `Pause` and the ask `Library` (`screens::step`: the chord over the game is the
  card; the card resuming is the game; Back to Library is Home with the game suspended; a game that stops under the card
  is Home). Home over a paused game is reached as in Mistress, by Leave ▸ Back to Library, so the player keeps a way
  home while a game runs.
- **Every word from a model, the actions as data.** `pause.rs` (plain Rust) holds the pages: the first page's seven rows
  in Mistress's words (`GameMenuRows`: Resume, Rewind, Save State ◂ slot ▸, Load State ◂ slot ▸, Speed ◂ 100% ▸, Game
  Options ▸, Leave ▸, the lighter rule ending the first section), Leave's page, each row's key, what a choice does
  (`choose`) and what B does (`back`: to the first page on the row that opened the page, else resume). The words come
  from the session's facts, read as the game pauses: `SessionModel.cardFacts` (the runtime's `session_info`,
  `rewind_reel` and `state_slots`, all reads, which the runtime answers on any thread, emusen_crystal.h), worded as
  Mistress words them (`RewindUnavailable`, `RewindHeld`, `SlotValue`). `PauseModel` puts the page on Qt as rows, title
  and breadcrumb; the help bar is `HelpModel`'s `Pause` and `PausePage` (A Select or Choose, B Resume or Back, X
  Screenshot, Start Resume), whose `resume` line is the frame's declared action (`EmuSen_Kit.md` §7.19).
- **The drawing** (`PauseCard.qml`, canvas units): the scrim over the paused frame, which never moves; the panel at the
  left edge at 40% of the width on Surface, its shadow by `MultiEffect` (offset 0,8, blur 24, black at 50%, §5.4) from a
  caster behind it, so the panel's words are never drawn through a layer; the title row (the game's cover at 60 × 80
  where the library has one, the breadcrumb in Muted above the title on a page past the first, the title in Barlow
  Condensed Regular at 32 down to 22, LunaP's card's face); the rows from 64,156 as `Kit.MenuRow`s 54 high with their
  marks and rules (`EmuSen_Kit.md` §7.21); the band, 98 units of Surface with a Border rule under the panel and its
  shadow, and its help bar over both. The help bar paints Surface: painted `transparent` on Vulkan it cleared its strip
  to alpha 0 across the window (the panel and band beneath read 0,0,0,0 in the grab, measured 2026-11-08; on the
  software backend it drew as expected), so it is opaque, and the panel's shadow does not darken the band within it.
- **The slide** (§20.6, LunaP §218): `slide` from 1 to 0 over Enter (200 ms, quadratic out) as the card opens and back
  over Exit (160 ms, quadratic in) as it closes; the panel moves left by its right edge and 40 units times `slide`, the
  band down by its height, the pane fades. A page turn moves nothing. The tokens' durations are nothing under reduced
  motion, so the card comes and goes at once. B or Start closes it at once (the card stops taking the pad), and the
  game resumes only when it says it is `gone`.

**Owed to stage 2, named:** the context pane (the reel's three pictures on Rewind, the slots' pictures and sentence on
Save and Load: the pane is laid out, empty); what Rewind, Save State, Load State, Speed, their Left and Right, Game
Options ▸ and X's screenshot do (declared, answering nothing yet); the console rows (Change Disc, Swap Players); Leave's
Restart Game…, Quit Game… and Quit EmuSen… with their questions; the Guide chords (Up saves, Down loads, R1 a
screenshot); and a card opened again while it slides out taking over that pause. `Play.qml`, emusen-qt-play's window of
one game, keeps pausing and resuming on the chord.

**Compared with Mistress** (measured on 2026-11-08). The Qt recordings `PauseCard-1280x800.spec` and `-1920x1200.spec`
were read by WiseMan's own `SpecDocument.Parse` and written back byte for byte, `InterfaceRules.All` found 0 faults, and
`InterfaceSpec.Compare` against Mistress's recordings found the card's root (`PadCard`, a pane over the window), the scrim
(`AdaptiveCardScrim`), the rows' box (`AdaptiveCardRows`, 64,156 424 × 378) and the list (`PadMenuList`, a list) in place
at both sizes. Its 53 differences at each size are these, each a fact of the toolkit rather than of the card: Avalonia's
wrappers, which Qt has no counterpart for (`AdaptiveLayers`, `AdaptiveContent`, the list's `PART_ScrollViewer`: the card
holds at most eight rows and never scrolls); `AdaptiveCard` at 0,0,512,800, where Avalonia's `MenuPanel` control spans the
window and paints its panel inside; the Qt items the C# tree has no names for; and the walk's 28 edges through the
seven rows, where Mistress records `walk none` because its menu moves its own selection. The grabs beside Mistress's
renders at 1280 × 800 and 1920 × 1080, with nothing drawn over either, are `~/.cache/emusen/tmp/qt-s1/compare/
PauseCard-*-side.png` and `PauseCardLeave-*-side.png`, with their differences as `*-diff.png`: the title within one unit
of width (254 against 255), the marks within a pixel, the rows one pixel lower, the help bar's last entry three units
right.

**Differences that remain, each for the tester:** the Rewind row reads "nothing to go back to yet", where Mistress's
render shows no value, because emusen-qt's session does not yet turn the runtime's rewind on from the player's settings
as Mistress does at a game's start (owed with Rewind's page); the paused frame is the test core's dark picture where the
render's is black; the panel's shadow does not darken the band within the help bar's strip (above); Mistress draws the
section's rule in `#707076`, the card in `Border` (`#707070`); the title's shrink floor is the design's 22 units
(`EmuSen_Interface.md` §5 item 4), where LunaP's card shrinks to 0.55 of 32; and Leave's page holds Back to Library
alone.

**Held to** (measured on 2026-11-08): `acceptance.rs` on Vulkan in its own compositor with the RX 6800, the game a real
session on the test core opened from the program's own path: the chord pausing it under the card, the card in place
with Mistress's seven rows and her words, held at the gate's five sizes with nothing logged and recorded; B resuming the
game only once the card has gone; the chord again, Leave ▸ (a page turn, nothing moved), Back to Library to Home over the
paused game, held as before. A watchdog stops its compositor: `Compositor::watch` (`emusen-qt-test`) makes a panic, or a
run past ten minutes, write its words to the process's own stderr, stop KWin and end the process, since unwinding
faulted in Qt's swapchain release and left the compositor running (step 6's open point); a first failed run showed the
panic's words and left no KWin. `pause.rs` on the software backend: the first page's words and help bar, the slide part
of the way in at 60 ms and in place by 400, a page turn moving nothing, B back to Leave's row, Back to Library, B and
Start resuming once gone, and none of the motion under reduced motion. qt-models' `pause` 4 (the rows and words from the
facts, the facts read as Mistress reads them, the choices and B, the help bar and declared actions) and `library` 12
(the screens' rule with the card); `program.rs`'s QML rules and `qmllint`; the kit's 39 and the gallery's gate 4; clippy
clean. Seeded faults, each reverted: the section's rule on Game Options rather than Speed (the models' `pause` test);
the slide's duration written as 200 ms in place of the token (`pause.rs`, under reduced motion); B on Leave returning
to the first row (`pause.rs`, the row that opened the page).

### 7.2 Stage 2: the Card form, the Home card and the List card (2026-11-08)

`EmuSen_Interface.md` §2.3 and §18.1; the oracle is Mistress's own cards, recorded as `HomeCardPower`, `HomeCardQuit`,
`HomeCardHandheldPower`, `GamesListCard` and `GamesSearch` in `EmuSen.WiseMan/Mistress/Specification/`, and her renders
(`HomeCardPower-plain` of 2026-11-02, `GamesListCard` and `GamesSearch` of the courts build of 2026-11-05). The
`HomeCardDesktop*` states and the themed `BigPicture-Gamelist` and `GamelistFilter` are read for reference: the first
belongs to the desktop form (stage 6) and the other two to the ES-DE engine (stage 4), whose Sort, Filter and Jump To
sheets the List card replaces (§18.1).

**One form for every card.** The Pause Card, the Home card and the List card are the same Card form (§2.8: "shares the
Home card's title row, rows, pane and help bar"). It is built once, in two parts, and the Pause Card's later pages are
built on it too:

- `qt-models/src/card.rs` holds the pages as plain data with no Qt. A `Row` has its key, its words, its marks (steps,
  opens, the lighter rule that ends a section) and its `Pane` (lines with a muted line under each, or a picture with its
  caption). A `Page` has a title, the words above it (the way back, or a question's sentence) and its rows. `question`
  makes a question a page of No then Yes, as Mistress's `Ask` does, so the focus starts on the answer that changes
  nothing. `Choice` says what a row does: turn to a page, go back, close and act, act and stay, or nothing yet. The
  `Pages` trait gives a card's pages, and `Deck` is the stack of pages turned through, so B returns to the row that
  opened a page. `CardModel` (`card_model.rs`) puts any card's pages on Qt: the rows as a list model (`label`, `value`,
  `steps`, `opens`, `sectionEnd`, `key`), and `title`, `breadcrumb`, `current` and the focused row's pane as properties.
  It opens at `start` each time `shown` turns on. A value stepped on the same page changes the rows in place
  (`dataChanged`), so the focused row keeps its delegate; a page turn resets them. Without that, the gate's walk lost
  the List card's rows after Left on Sort: the reset made new delegates under the walk (a seeded fault below).
- `qt-app/qml/MenuCard.qml` draws it: the scrim, the panel at 40% with its shadow, the title row (an optional cover, the
  breadcrumb above the title), the rows as the kit's `MenuRow`s, the pane at 576,156, the 98-unit band with its help bar,
  and the slide (Enter in, Exit out, nothing under reduced motion). Its words are its host's properties, its pane is
  lines or a picture or the host's own items (its default property, for the Pause Card's reel and slots), and its
  signals are `chosen`, `stepped(index, by, held)`, `focused`, `backed`, `acted` and `gone`, each answered in one call.
  **The shadow falls over the help band** beside the panel, as Mistress's does (the tester's note on stage 1's Pause
  Card, 2026-11-08). The band and its bar are painted after the panel, opaque because a transparent bar cleared its
  strip on Vulkan (§7.1 step 7). So the part of the shadow beside the panel is drawn again over them, clipped to the band
  and to 64 units past the panel's edge. Its falloff is matched to Mistress's render. The design's offset of 0,8 is kept,
  with a radius of 48 and black at 90% (`shadowOpacity`, `shadowRadius`). Measured on Vulkan at 1280 × 800 on
  2026-11-08, every 4 pixels from the panel's edge over the band, Mistress's render reads 15, 17, 20, 23, 25 and 26
  against Surface's 27, and Qt reads 16, 19, 21, 23, 24 and 25. Before, at the Pause Card's radius of 24 and 50%, Qt read
  22 at the edge and nothing by 17 pixels.

**The Home card** (`home_card.rs`): Mistress's `HomeRows`, `ToolRows` and `PowerRows` in her words. The first page is
Continue (while a game is suspended), Selected game ▸, Search, Start Console ▸, Tools ▸, Settings ▸ and Power ▸, with
Tools and Power each starting a section. Tools lists the five tools with their summaries as values, then Themes,
Collections, Compressed Discs and Shell. Power has Exit Big Picture, then Quit EmuSen..., whose question says "EmuSen
closes." or "The game closes, then EmuSen."; Yes quits the program. In Steam's Game Mode Power holds Quit alone. Under
Power's rows stands "EmuSen 0.9.1" in Muted at 26 (Mistress's `BigMenuFooter`; within a pixel of her render at 1280 ×
800). The version's owner is still MSBuild's `Directory.Build.props`. `qt-models/build.rs` reads it from there rather
than copying it, and the reading moves with the build host (§1.9). Start Console is offered on both cards wherever a
console of the systems table starts with nothing in it (`starts_empty`, Mistress's `ConsolesThatStartEmpty`, through
`LibraryModel.startsEmpty`).
`start.rs` reads Game Mode as Mistress's `InGameModeSession` does: gamescope among `XDG_CURRENT_DESKTOP`'s names, else
`SteamDeck=1`. Its facts come from `LibraryModel.homeCardFacts` (the suspended game, its paused words, the tools'
summaries, Game Mode).

**Start and the routes, as Mistress's own look has them** (decided 2026-11-08; `OwnScreenPadCommand`,
`EmuSen_BigPicture.md` §51.2, §51.5). Start on the grid or on Choose a console goes Home, and on Home it takes the focus
back to where Home starts (`Screens.homeAgain`). Home's Power place opens the Home card at Power, with Home as the way
back. On the big screen's own look the card's first page and Tools are not reached: Mistress draws that card only over
a theme and on the desktop. They are built and held by the models' test, and have nowhere to show until stage 3 (themes)
and stage 6 (the desktop form). The screen under a card is drawn dimmed, is disabled, and declares itself a `backdrop`,
so the gate does not judge what the scrim covers. Start on Choose a console is declared as an action its help bar does
not name (`help::unnamed`), since the mockup's bar shows four entries.

**The List card and search** (`list_card.rs`, `listing.rs`). Select on the grid opens it, titled with the section and its
count ("All · 8 games") under the breadcrumb "List". Its rows are Search ▸, Sort ◂ Name ▸ (Name, Last played, Most
played, Year, Rating), Filter ▸, Jump to ◂ letter ▸, Random game and Collections ▸, each with Mistress's pane. Filter's
page has Every game and Favourites (not offered in the Favourites section); each section keeps its own choice. The grid's
`GameModel` gained the listing (`section`, `sectionName`, `search`, `focused`, `countLine`, `listFacts`, `listAct`,
`back`). It is applied off the GUI thread with the rows' read, which also finds each letter's first row, so Jump to and
Random game are steps through a few dozen letters or one number. The count reads "1 of 8 games" while narrowed. B
clears a search, then the section's favourites, then goes back a screen (`passedBack`). Search docks the keyboard: the
field at 64,100 1152 × 44 over one row of covers, the keyboard's place at 0,398 304 high, and the help bar's keyboard
entries (Type, Delete, Space, Done, Move to grid, Home), as `GamesScreen.HintsFor` gives them. The kit's docked
`Keyboard` stands in its place (§7.6).

**Compared with Mistress** (measured on 2026-11-08). Every Qt recording (`~/.cache/emusen/tmp/qt-s2/compare/qt/`) was read
by WiseMan's `SpecDocument.Parse` and written back byte for byte, and `InterfaceRules.All` found 0 faults in each.
`InterfaceSpec.Compare` against Mistress's recording of the same state found the card's root (`PadCard`), its scrim, rows
and list in place at both sizes, with these differences, each a fact of the toolkit or named here: Avalonia's wrappers
(`AdaptiveLayers`, `AdaptiveContent`, `PART_ScrollViewer`, `PauseCardPane`); `AdaptiveCard` at 512 wide where Avalonia's
`MenuPanel` spans the window; the Qt items the C# tree has no names for; the walk's edges, where Mistress records `walk
none`; and the screens beneath under their stage 1 names. The side-by-sides (Mistress left, Qt right, nothing drawn
over either) are `~/.cache/emusen/tmp/qt-s2/compare/<State>-<size>-side.png`.

**Differences that remain, each for the tester:**
- Exit Big Picture closes the card and does nothing more: emusen-qt has only the big-screen form until stage 6.
- The Tools page's rows, Settings ▸, Start Console ▸ (§18.3's page), Collections ▸ and the paused frame in Continue's
  pane are declared and do nothing yet. Each opens a screen of a later stage.
- Most played, Year and Rating sort by name, since the library reads no play counts, years or ratings yet. Mistress
  falls back the same way for Rating.
- The Filter page lists no collections of the player's own, since the library does not read them yet.
- The words of the screen under a card read under 4.5:1 through the scrim. Mistress's recordings hold the card alone, so
  the test names this (`cards.rs`' `named`), as it names the grid's 18-unit neighbours at 1280 × 720 that stage 1 named.
- The title stands one or two units higher than Mistress's, as the Pause Card's does (§7.1 step 7).

**Held to** (measured on 2026-11-08). `qt-models/tests/cards.rs` (4): the Home card's pages and words (Power with and
without Game Mode, the question's two sentences, the first page's order and sections, Tools' values and panes, the
attention value); the deck's way back; the listing (the name sort in any case, Last played, Recent's own order,
favourites, search, wrapping sorts, the count line, letters and jumps); the List card's rows, marks, panes and choices; and
the screens' rule for Start and the cards. `qt-app/tests/cards.rs` drives the program on the software backend: Power
opens the card at Power, Quit asks, B returns to its row, No goes back, B closes, Start takes Home's focus back, and
Start on the grid goes Home. Select opens the List card; Sort steps both ways; Jump to moves the grid's focus; Filter ▸
Favourites narrows the grid to "0 of 8 games"; B clears it. Search narrows it to "1 of 8 games"; B clears the search,
closes the keyboard, then goes back. Each card is held at the gate's five sizes with nothing logged and recorded.
`scale.rs` holds, over 9,000 games, Sort and Jump to within 400 ms each and a search's narrowing within 400 ms.
`start.rs` holds Game Mode. The library's actions test, `screens.rs` (which now waits for the grid's off-thread read),
`program.rs`'s QML rules and `qmllint`, and `pause.rs` pass, and clippy is clean. Seeded faults, each reverted:
- B returning to a page's first row: the models' deck test.
- The recently played sorted by name: the listing test.
- The rows reset on a stepped value: the program's cards test, where the gate's walk did not reach the List card's rows.
- Start on the grid left where it was: the program's cards test.

**A narrowed tile is the full grid's tile** (checked on 2026-11-08). The first GamesSearch side-by-side showed the found
game as a generated card, smaller and lower than Mistress's cover. The cause was the fixture, not the search. The program
test's sandbox had no media store, so every tile of its grid, narrowed or not, was a generated card, and it was set
beside Mistress's render of a library with covers. The test now draws the covers of Mistress's own-look fixture into an
ES-DE media folder (`common::draw_covers`, shared with `acceptance.rs`). It holds Granite Choir's tile in the search to the
full grid's: the same picture heard by its title, the same box, and the same colours at 25 points inside it. A tile
whose cover is dropped while searching fails that check (seeded and reverted). The re-made side-by-sides show the
cover at Mistress's place and size; the keyboard's area stays empty until the kit's keyboard lands.

**On Vulkan** (`acceptance.rs`, measured on 2026-11-08 in its own compositor on the RX 6800, 72 s). The program over
Mistress's own-look fixture holds the List card over the grid of eight covers ("All · 8 games"), HomeCardPower and
HomeCardQuit at the gate's five sizes with nothing logged, and checks the shadow over the band (`shadow_over_band`). The
fixture's systems table, read through the runtime with its three engines, marks no console as starting empty, so the
row is absent there; the models' test holds it present where the table says so. The test now reads each card's slide
through its canvas (`cardCanvas`, `homeCardCanvas`, `listCardCanvas`), since every card's root is `PadCard`, as
Mistress names it. The first run, with the Pause Card's shadow (radius 24, black at 50%), failed `shadow_over_band` at
22, 27 and 27 for 1, 17 and 36 pixels out, which is why the shadow was matched as above.

The C# side is unchanged. No C++ was added.

### 7.3 Stage 2: the Pause Card's pages (2026-11-08)

The pages players use most come first: Rewind with its reel, and Save State and Load State with their slots
(`EmuSen_Interface.md` §2.8, §18.5, §20.1). The oracle is Mistress's: `GameMenuRows`, `MainWindow.Rewind`,
`PauseCardPane` and the recordings `PauseCardRewind-*` and `PauseCardSave-*`. Piece B of stage 2 builds the one card
form (`MenuCard`) every card is drawn on; the pages below are models and actions first, and `PauseCard.qml` moves onto
that form (below), its shadow and band the form's.

**The parts, each with one job** (all in `emusen-qt-models` unless named):
- `reel.rs`, **the reel as plain Rust**: rewind kept for every game as Mistress's `WireSession` keeps it (`keep`: a
  snapshot every half second of console time, `interval_for`, the history's budget the runtime's own, a picture 160 wide
  with each snapshot), called by the frame thread right after a game opens; an engine that withholds rewind says why in
  `session_info`, which the card shows, and the game plays on. The marks (the present, then the held moments with a
  picture, newest first), the strip's stops (the present, the moments nearest 1.5, 4, 8 and 12 s back, the oldest), a
  press of Left or Right going stop to stop, held moment by moment, the shoulders striding 5 s and the triggers to either
  end, a scrubbed moment standing in the strip for the nearer stop, and every word ("1.7 s ago", "Stay at Now", the
  sentences). The constants restate Mistress's (`RewindSnapshotsPerSecond`, `ReelPictureWidth`, `StopSeconds`), which
  have no Rust owner while the C# front end ships; when it retires these are their owner.
- `pause.rs`, **the card as plain Rust** (`Card`): the page (First, Leave, Reel), the row focused, the reel while its
  page is open, and the session's facts; its rows, title, breadcrumb, help bar and **context pane** (`Pane`: the reel's
  three newest pictures and what it holds on Rewind, the focused moment over the strip of six on the reel's page, three
  slots around the current one on Save and Load with what A does to it); and what a choice, a step, a shoulder or B
  comes to (`Outcome`: a slot or a speed chosen, a slot saved or loaded, a moment to return to, resume, the library).
  Save closes the card as Mistress's does; Load on an empty slot does nothing; the slots wrap from 1 to 8 and 8 to 1.
- `PauseModel` puts the card on Qt (rows, `pane` as plain data, `helpScreen`) and says what the session must do as
  signals (`slotChosen`, `saveAsked`, `loadAsked`, `rewindAsked`); a step changes the rows' values as a data change and
  a page turn resets them. `PaneModel` reads a pane's plain data into tiles (picture, name, when, ringed) and properties,
  as `GameModel` reads `games`; `CardPane.qml` draws it in canvas units as `PauseCardPane` does (184 × 138 tiles 20
  apart, the moment at 592 × 300 over 88 × 66 tiles, the ring 3 in Accent, an empty slot a dashed Border outline).
- `SessionModel` gains the slot (`slot`, `chooseSlot`), `saveState`, `loadState` and `rewindTo`, each made on a thread
  of its own as Mistress makes them, their words (`notice`, Mistress's "State saved to slot 3", "Rewound") brought back
  through `qt_thread().queue`; a resume asked while one runs waits for it, so the card's B or A never lets a frame run
  before the game holds the state or the moment. The card's facts now carry the reel's held frames, the present, the
  slots with their times worded in the player's time (`QDateTime`, "Sun 8 Nov, 21:42") and the address of each picture.
- `pictures.rs`, **the session's pictures by address** for QML's `Image`: `image://session/<pause>/reel/<frame>` (the
  runtime's `rewind_picture`), `.../now` (the picture on show, its repeated rows written out, Mistress's
  `CurrentPicture`) and `.../file/<path>` (a slot's picture, which the runtime writes beside the state, at the path
  Galaxia names: `emusen_galaxia_save_path` with `SAVE_PICTURE`, now wrapped by `emusen-platform-client`). The pause's
  number in the address keeps a picture of an earlier pause out of the image cache. *C++ part*:
  `cpp/session_pictures.cpp`, a `QQuickImageProvider` registered on the engine that holds the `SessionModel` as its game
  opens [own, because QML takes an image only by address, an image provider is Qt's mechanism for pictures held in
  memory, and cxx-qt subclasses no Qt class but a QObject's].
- `help.rs` gains the first page's bar on Save and Load (Change slot on Left and Right) and the reel's (Go back, Back,
  Moment, Screenshot, no Start entry, as Mistress's reel sets `StartResumes = false`); `pair_of` pairs Left with Right.

**Differences from Mistress, named:** the help bar draws Left and Right as two d-pad glyphs where Mistress draws one
glyph of both; an empty slot's outline is Qt Quick's dash (4, 2) where Mistress's is (4, 3) (QML may build no array,
§1.3); the reel's Hold row is drawn as a plain row, not greyed, until the kit's row has a dimmed form; Load does not
yet ask before loading a state its record ties to another copy of the game (Mistress's `SameGameOrConfirmed`, which
needs the question form of piece C); §2.8's "Load first drops one undo snapshot into rewind" is not done by Mistress
either, and the runtime clears the history on a load; the shoulders and triggers on the reel are the model's and are
not yet delivered by the card's QML (delivered since §7.5); the words of `notice` wait for piece C's notices.

**Held to** (measured 2026-11-08): `qt-models`' `pause` 8 (the rows and words, the facts from a reel and a session, a
choice, a step and B, the slots' pane and sentences, the Rewind row's pictures, the reel's page from its first stop to
28 s back in four presses, a scrubbed moment, a stride, Stay at Now, A's frame, B back to Rewind's row, Mistress's
stops) and `pictures` 1 over the platform library built from the same commit, no window and no device (rewind kept with
the interval for the game's rate, the reel's moments read into the facts, a moment's picture and the picture on show
served, a slot saved with its picture where Galaxia names it, loaded, an empty slot refused, and the game returned to
the moment the card's A names, the history after it dropped); `qt-app`'s `pause_pages` on the software backend (the pane
following the row, the reel's three newest, the three slots and their sentences, Change slot on the bar, Right choosing
slot 2 through the session, the reel's page turned in place with its strip, Right to the next stop, B back to Rewind's
row, nothing logged); `pause`, `program` (the QML rules and `qmllint`) and `screens` as before; `platform-client`'s
Galaxia test (a slot's state and picture named in the scratch tree's Save States). Clippy clean. Seeded faults, each
reverted: the slots not wrapping (`pause`, Left from slot 1), a snapshot a second in place of two
(`pictures`), the pane not following the row (`pause_pages`).

**On Vulkan, over a real session** (measured 2026-11-08; `acceptance.rs`, the RX 6800 in a compositor of the test's own):
the Rewind row half a second into the game, where the reel holds no picture yet and the pane says "Under a second
available", as Mistress's render of the row does; Save State on Slot 1 (Empty); A saving and the game resuming once the
state is written ("State saved to slot 1"); after 2.5 s of play the reel's three newest pictures drawn through the
provider, and the slot's own picture from beside its state; A on Load State ("State loaded from slot 1"); the reel's
page with its moment drawn large, and A returning the game ("Rewound"). Each page is held at the gate's five sizes with
nothing logged (`PauseCardRewind`, `PauseCardSave`, `PauseCardRewindHeld`, `PauseCardSaved`, `PauseCardReel`). The
side-by-sides with Mistress's renders, nothing drawn over either, are `~/.cache/emusen/tmp/qt-s2/compare/
PauseCard{Rewind,Save,Leave}-{1280x800,1920x1080}-side.png` with `-diff.png` beside: the rows, the title, the pane's box,
the three slots, their words and the sentence stand where Mistress's do. **Differences that remain, for the tester:**
the paused frame is the test core's red picture where the render's is black; at 1920 × 1080 the Rewind pane's sentence
wraps to a second line where Mistress's fits one.

**Leave's rows and the card form** (2026-11-08). Piece B's card form (`MenuCard.qml`, `card.rs`) landed beside this piece,
and the Pause Card is now drawn on it: `PauseCard.qml` is a `MenuCard` whose rows, title, breadcrumb, cover, current row
and help screen are `PauseModel`'s, whose signals each call one of its invokables, and whose pane box holds `CardPane`
(the form's `paneDrawn`, added for this, shows the box while the host's own drawing has something to show; counting the
box's children showed it on every row, and counting its visible children never showed it, the children being inside the
hidden box). Leave holds Mistress's four rows (`LeaveRows`): Back to Library, Restart Game..., Quit Game..., Quit
EmuSen.... Each end is asked first on the card form's question page (`card::question`: the question as the title, the
sentence above it, No then Yes, so A straight through changes nothing), in Mistress's words; No and B go back to the row
that asked. Yes on Restart resets the game through the session (`SessionModel.restart`, `emusen_crystal_reset`, made off
the GUI thread as a save is, the resume waiting for it) and closes the card; on Quit Game the session closes and the
screens' rule shows Home; on Quit EmuSen the program's `Screens.quit` ends it, as the Home card's Quit EmuSen does.

**Held to:** `qt-models`' `pause` 9 (Leave's rows, each question's title, sentence and No first, No and B back to the row
that asked, each Yes's outcome); `qt-app`'s `pause` (the four rows, Restart Game's question, No back to its row, Quit
Game's question, B back, Yes quitting the game), `pause_pages` and `cards` (piece B's, unchanged by the form's addition);
`acceptance.rs` on Vulkan, which now also holds Quit Game's question at the gate's five sizes (`PauseCardQuitGame`): its
rows at 64,156 and 64,210, the sentence at 58 and the title at 82, and the help bar's Choose, Back, Screenshot and Resume
are those of Mistress's `PauseCardQuitGame-1280x800.spec` (no accepted render of that page exists to set beside it).
Seeded faults, each reverted: Quit Game's Yes quitting EmuSen (`pause`), B on a question going to the first page
(`qt-app`'s `pause`), the card not carrying Quit Game's Yes to the window (`qt-app`'s `pause`).

**Speed** (2026-11-08). The row steps 25%, 100% and 300% (Mistress's `StepSpeed` over `SlowMotionPercent`,
`NormalPercent` and `TurboPercent`), Left and Right or A, and stays on the card. The choice is `SessionModel.speed`, the
player's for as long as the program runs, as Mistress keeps `_baseSpeedPercent`: given to the running game at once and to
each game opened after (`chooseSpeed`, `emusen_crystal_speed_set` with turbo and slow motion at the row's fastest and
slowest, as `ApplySpeed` gives them), and read back into the card's row. Nothing else is new for it: the runtime's frame
says the interval at that speed and whether its sound is played, so the frame thread's own clock already runs a game off
the display's rate at that interval (§1.8.3), and since this step a frame whose sound is not to be played (turbo,
unthrottled) is drained and dropped rather than handed to the player, as Mistress does.

**Held to:** `pause_pages` (Right on Speed choosing 300 through the session, Left twice 25); `pictures` (the frame's
interval a third at 300% and four times at 25%, within a microsecond of the runtime's rounding, and no sound played at
turbo); `acceptance.rs` on Vulkan (Right giving the running game turbo, read back from the runtime into the card's facts,
and Left the game's own speed again). Seeded faults, each reverted: the card not carrying the choice to the session
(`pause_pages`); the speed's change not announced, which no test catches, since the harness reads a property's value
and not its signal; the card's row reads the speed from the facts, which are read again either way. (Caught since
§7.5, whose harness counts a property's signal.)

**The screenshot** (2026-11-08). X on any page of the card is its declared action `screenshot` (help.rs, every Pause
screen): the card stays and `SessionModel.screenshot` writes the picture on show, its repeated rows written out, as a PNG
in Galaxia's screenshots folder (`EMUSEN_GALAXIA_DIR_SCREENSHOTS`), named as Mistress's `TakeScreenshot` names one (the
game's file name, then the local time to the millisecond, `2026-11-08 21.42.07.115`), off the GUI thread, its notice
"Screenshot saved". *C++ part*: `savePicture` in `cpp/session_pictures.cpp`, Qt's image writer [own, because cxx-qt-lib
wraps no `QImage::save`, and Qt already holds a PNG writer]. **Held to:** `acceptance.rs` on Vulkan (X on the card saves
the screenshot and the card stays in place).

**One owner for a card's rows on Qt.** `row_roles.rs` holds the roles `MenuCard.qml`'s rows read (label, value, steps,
opens, sectionEnd, key, dimmed) and each row's value for them; `CardModel` and `PauseModel` both answer `data` and
`roleNames` through it, where each had held its own copy.

### 7.4 Stage 2, the forms: the question, notices and the docked keyboard (2026-11-08)

Kit pieces the stage's screens use, each an extension of `EmuSen.Kit` reviewed under rule Q (`EmuSen_Kit.md` §7.22 to
§7.25, where each is held to its tests): the **docked keyboard** (`KeyboardModel` and `Keyboard`, kit-core's
`keyboard.rs`), at Mistress's boxes from `GamesSearch-1280x800.spec`; the **question form** (`Question` rebuilt to
Mistress's question card from `CoresPlan-1280x800.spec`, `MenuRow.overRaised`, and the words in `emusen-qt-models`'
`question.rs`, the Resume question's among them); **notices** (`NoticeModel` and `NoticeLayer`, kit-core's `notice.rs`);
the **vertical walk** passing a field and a combo box (`KeyPass`); B offered to the focused control before the host's
`backDeclined`; the router's clock asleep while it has nothing to do; and the night **help band** as glass. The
side-by-sides, Mistress left and the kit right, are in `~/.cache/emusen/tmp/qt-s2/compare/`.

**Named, for the tester:** the question's cover stops at the 58-unit help bar on the foot (742), where Mistress's Cores
window, whose band is 98 high, stops it at 702, so the card is centred 20 units lower; the notice has no shadow on the software tier (it has on the GPU, §7.6); with a combo box's list open, the pad does not reach the frame until it closes (owed to stage 5). The Resume question is asked at a game's start since §7.6.


### 7.5 Stage 2, the Pause Card's remaining rows: Guide chords, Game Options, Change Disc, Swap Players, the reel's shoulders (2026-11-08)

`EmuSen_Interface.md` §18.5, §20.2, §20.3 to §20.9; `EmuSen_BigPicture.md` §48.3, §48.7. The oracle is Mistress's
`GameMenuRows`, `GameOptionsRows`, `ChangeDiscRows`, `PauseCardPane.ShowCover`, `GuideChordReader` and `MainWindow.Pad`,
the recording `PauseCardGameOptions` and her render of it.

**The Guide chords.** The reader and the router's part are the kit's (`EmuSen_Kit.md` §7.26); the program gives them
meaning. `pause::guide_chord` says Guide with Up saves to the slot chosen, with Down loads it, with the right shoulder
takes a screenshot; `SessionModel.guideChord` acts through the session's own `saveState`, `loadState` and `screenshot`,
off the GUI thread, their notices as before. `start.rs` reads `AppSettings.GuideChords` as Mistress's `GuideChordsOn`
does ("On", "Off", or Auto, the default, off under Steam's Game Mode), and `Main.qml` hands it to `Game.qml`, which
binds `PadRouter.guideChords`. While Guide is held the game hears nothing (`PadRouter.guideHeld` bound to
`PadReader.gameMuted`, `EmuSen_Kit.md` §7.26), so the state saved is the one the game was in before Up.
Decided by the tester on 2026-11-08: the Controllers, Display, Sound and Memory Cards pages and the Cheats window wait
for stage 5.

**The first page's console rows** (§48.3). `Change Disc ▸` stands after Speed where the session's drive can change (the
runtime's `discs.can_change`, Mistress's `DriveEngine`); `Swap Players 1 and 2` stands there instead where player 2 has
a pad and no drive row takes the eighth place. The last of them ends Speed's section, so Game Options starts the next.
The drive, the game's file and its system as the library keys it now cross in the card's facts, and whether player 2
is seated is read from the pads' hand-over (`GamePads::seated`).

**Change Disc** (§20.2): the game's own discs (`Basalt Run (Disc 1)` with `Inserted` on the one in, or the game's name
for a single disc), then the console's other games in the library (`LibraryModel.gamesOf` of the facts' system, handed
to `PauseModel.shelf`), then `Open the tray` while a disc is in. A on an own disc is `SessionModel.changeDisc` ("Disc 2
is in"), on another game `insertDisc` ("Amber Coast is in"), on the tray `ejectDisc` ("The disc is out"), each off the
GUI thread with the card closed, as Mistress's rows close it; A on the disc already in only closes the card. The help
reads Put in, Back and Screenshot, Start not offered (`help::Screen::PauseDisc`). The pane is Mistress's `ShowCover`: the
cover at 180 square, the generated card edged in Border where there is none, and beside it, 24 across and 48 down, the
disc's name and the sentence in Muted at 22 ("A opens the tray, swaps the disc and closes it, as the console sees it.",
or "In the console now. B goes back."); the tray's pane is its sentence alone. `Any file…` is the desktop form's
(stage 6).

**Swap Players 1 and 2** closes the card and swaps the first two players' pads in the hand-over from the next read, for
as long as the program runs (`GamePads::swap_players`; the reader seats them swapped). Its notice reads "Players 1 and 2
swapped".

**Game Options** (§18.5): Cheats ▸, Controllers ▸, Display ▸, Sound ▸, and Memory Cards ▸ on a console with a drive,
titled Game Options with the game as the way back, B returning to its row, the bar A Choose, B Back, X Screenshot,
Start Resume, as `PauseCardGameOptions-1280x800.spec` has it. Cheats is drawn opening, as Mistress's `Opens` marks it
and her render draws it (the recording's text omits the mark, which her row draws from `Opens`, not from the text).

**The reel's shoulders.** `MenuCard` hands a key its row did not take to its host (`keyed`); `PauseModel.key` reads it
as a press and `Card::shoulder` takes it as Mistress's reel does: the left shoulder five seconds further back, the
right five nearer, the left trigger the oldest moment, the right the present. Seen in the test: a stride from 4.2 s
went to 9.7 s, not 9.2 s, because 250/60 + 5 lies an ulp above 550/60. Mistress's `Stride` compares the same way, so
the defect is hers too; `reel.rs` now takes two moments within a nanosecond as one time (`SAME_MOMENT`).

**Differences from Mistress, for the tester:**
- Controllers, Display, Sound, Memory Cards and Cheats are declared and do nothing yet. Their pages are pages of
  Settings' one description (§20.3, §20.7 to §20.9) and Cheats is the cheats window; both are stage 5's.
- Her render of Game Options is older than the recording: it shows a Volume row where §20.8 put Sound ▸. The Qt page
  follows the recording. Over the test core, which claims a drive, the page also holds Memory Cards, and the first page
  Change Disc, which Mistress's render (made over a core without one) has not.
- No accepted render or recording of Change Disc exists; the page is held to her code's words and the pane to
  `ShowCover`'s measures.
- The generated card in the cover pane is drawn at hue 0, where Mistress's carries the console's hue.
- Swap Players' notice does not name the pads, where Mistress's reads "Wireless pad: player 1 · …": the pads' names do
  not reach the session's side of the hand-over.

**Held to** (measured on 2026-11-08). kit-core's `input` (the reader), kit-qt's `router` 8, `route`, `qt_keys` and
`qmltypes`; `qt-models`' `pause` 14 (Game Options' rows and way back, Change Disc's rows, outcomes, panes and way back,
a single disc, Swap Players and its room, the chords' meaning, the shoulders and triggers); `qt-app`'s `pause_pages` on
the software backend (the speed's change counted, the shoulders and triggers on the reel, Game Options, Change Disc's
rows, bar and cover pane, Memory Cards) and `start` (the chords' setting); `acceptance.rs` on Vulkan over a real session
(Change Disc over the test core's drive, held at the gate's five sizes as `PauseCardChangeDisc`; Game Options as
`PauseCardGameOptions`; Guide with Up saving to slot 1 through the session with no menu, Guide going down opening
nothing and let go alone opening the card). Clippy clean. Seeded faults, each reverted: Guide alone opening nothing
(kit-qt's `router`); Up loading (`pause`); Swap Players beside the drive's row (`pause`); B on Change Disc to the first
row (`pause`); the card's row passing no key (`pause_pages`, the shoulder); the speed's change unannounced
(`pause_pages`, the count). The side-by-sides, Mistress left and Qt right, are
`~/.cache/emusen/tmp/qt-s2/compare/PauseCard{,GameOptions}-{1280x800,1920x1080}-side.png`; the Change Disc page is
`PauseCardChangeDisc-*.png` alone.

*C++ parts:* none in the program; the harness's spy (`EmuSen_Kit.md` §7.26).

### 7.6 Stage 2, the loose ends: the search's keyboard, the Resume question, notices (2026-11-08)

**The merged tree on Vulkan.** WiseMan d4b23bf05 (stage 2's three branches merged) passed `acceptance.rs` in its own
compositor on the RX 6800 before anything here was built on it: 1 test, 139 s, nothing logged.

**The grid's search on the kit's keyboard** (`GameGrid.qml`; LunaP's `GalleryLayout` and Mistress's `GamesScreen` the
oracle). The placeholder rectangle is gone: `Kit.KeyboardModel` and `Kit.Keyboard` dock at 0,398, wired as qt-test's
`KeyboardScene` wires them. `GameModel.search` stays the words' one owner; the typist writes it and is set from it, so
B on the covers, which clears the search there, empties the keys' text too.
- **Two states of one screen.** `Screens` gains `SearchGrid`: the keyboard docked, the focus on the covers. `Search` is
  the keys' (A types, B erases a letter and on an empty field closes the keyboard, Y Space, X Done keeping the words,
  Start Home); LB or RB moves the focus to the covers while there are any (`Screens.found`, LunaP's `_items.Count`).
  On the covers either shoulder takes it back, A plays, B is the grid's (the search cleared, then Choose a console, as
  `GamesBack` goes) and the help bar is the grid's, B reading **Clear search** while words are searched (`HelpModel`'s
  `GridSearched`, `GamesScreen.HintsFor`'s `searched`). The keyboard's close is the screen's Back (`Search` to `Grid`).
  The shoulders reach these declared actions through kit-core's walk, which now leaves the shoulders to a screen that
  has no sections (`EmuSen_Kit.md` §7.27); the grid's shoulders are declared as `previous` and `next`.
- **No lift under the docked keyboard** (decided 2026-11-08 by the tester). Mistress's geometry (cells at 156, the field
  to 144, the keyboard from 398) leaves 242 units for a 240-unit cover, so the lit cover's lift (rise 6, scale 1.06)
  reached 1.2 units over the field's foot and its ring past the grid's clip (the gate's overlap, 2026-11-08). While the
  keyboard is docked a lit cover keeps its ring and stays in place (`Cover.lifts`); the gate holds `GamesSearchGrid`
  at its five sizes with no overlap.
- **The field is never focused**, as LunaP's `GallerySearch` (read-only, not focusable): its Border edge, never the
  accent ring, the words in TextStrong 16 units in. The keys' ring shows only while they hold the focus.
- **The pointer moves the focus too.** A cover the pointer rests on takes the focus, as every walkable's hover does, and
  a click on the band gives it to the keys; the screen follows (`Screens.keysFocused`), so the help bar and B always
  answer where the focus is. The focus is held to the state by a `Binding`, since a hover's `forceActiveFocus` undoes a
  plain focus binding. Found by the gate on 2026-11-08: after a resize to 3840 × 2160 the pointer, left where a key had
  been clicked, rested on the cover and took the focus while the help bar still said Type.
- **Compared with Mistress** (`GamesSearch-{1280x800,1920x1200}-side.png`, the courts build's render left): the field,
  the cover and its count, the band, every key at Mistress's box and `n` ringed stand where hers do. The erase key's
  glyph is the font's `⌫` where Mistress's is drawn smaller; the help bar's shoulders are two glyphs where Mistress
  draws one (§7.3). At 1920 × 1200 the ring stands on Done, where the gate's walk at 1280 × 800 left it: the harness
  puts back a Qt Quick view's current index after its walk, not a model's (`KeyboardModel.current`).
- **Named, for the tester:** the gate's walk presses the four directions, so the covers reached from the keys only by
  the shoulders, and the keys from the covers, are named in the cards test (the test walks them by the shoulders and
  holds the focus there). Select on the covers opens the List
  card, which closes back to the grid without the keyboard, where Mistress's keeps it docked.

**The Resume question at a game's start** (`EmuSen_Interface.md` §18.3; Mistress's `ChooseResumeAsync` and
`WriteResumeState`). No new call: the runtime's resume state was already wrapped by `emusen-crystal-client`
(`resume_save`, `resume_load`, and `state_slots`, whose slot 0 is the resume state with its time), and the setting is
Galaxia's `ResumeOnLaunch` (`Ask`, `Resume`, `Restart`) read through the platform's settings model.
- `resume.rs` (plain Rust) holds the rules: the setting read; a start is from the beginning with no resume state or
  with Restart, where it was left off with Resume, and the question with Ask; the answers by index (Continue, Start
  again, B). `QuestionModel` puts `question::resume` on Qt for the kit's `Question`.
- `SessionModel.open` opens the game **held** (session.rs's `held`: the frame thread opens it and runs no frame) unless
  the setting is Restart. When the runtime reports it open, slot 0 decides: run it, load the resume state off the GUI
  thread then run it, or raise `resumeAsked`. `answerResume` loads, starts afresh, or on B lets the game go unplayed:
  it is no longer named (`Play.Game.game` is now the session's own `game`), so choosing it again opens it and asks
  again. Before that, B closed the session but left the game named, and the same game chosen again changed nothing.
  A resume the screens ask meanwhile waits, as it waits for a save.
- **Where it was left off is written as the game closes**, on the frame thread once it has stopped (Mistress writes it
  with the emulation thread stopped), only for a game that ran a frame, so B on the question leaves the old state in
  place. A failure to write it is reported as the session's problem.
- `Main.qml` draws the question on a `Kit.Frame` of its own over the held game (`QuestionBack`'s bar, A Choose · B
  Back), and the game gives up the pad while it is asked (`inFront`).

**Notices in the program and their shadow.** The session's words (`State saved to slot 1`, `Rewound`, `Screenshot
saved`, what failed) now reach `Kit.NoticeModel` and a `NoticeLayer` over everything, fit for gameplay; a job's words
are `Ok` or `Err`, the latter shown for attention (`noticeAttention`). The notice draws §5.4's shadow
(`EmuSen_Kit.md` §7.27). **A named difference from Mistress** (decided 2026-11-08 by the tester: the design's shadow is
kept): Mistress's notice, LunaP's pill, has no shadow.

**Home's Continue line and its count** (the Home side-by-side of stage 2). Qt read "Paused just now" where Mistress
reads "● Super Nintendo · Paused just now", and Games 7 where Mistress says 8. Both are the fixture's: acceptance.rs
replaces Cobalt Harbor's `.sfc` with the test core's `.tst` image to play it (the sandbox has no Super Nintendo core),
so the library holds 7 games and none is the suspended game. The Continue line takes its console and its dot from the
game's library row (`Clock.console` from `recent.consoleAt`), and finds none. With a library game suspended it reads
"Super Nintendo · Paused …" (`words::paused_facts`, held by the models' tests). A game opened from outside the library
shows no console, where Mistress names the session's.

**On Vulkan** (`acceptance.rs`, 2026-11-08, its own compositor on the RX 6800): the saved state's notice at the top
right over the gameplay, the game's picture 8 pixels under it darker while it shows than once it has gone; the game
closed and opened again asks the Resume question over the game held with no picture (`firstPictureMs` 0), Continue
focused, held at the gate's five sizes (`ResumeQuestion`); B closes it to Home; opened again it asks again, and
Continue loads where it was left off and the game draws more than 10 frames in 0.5 s, with no "Could not" notice. The
last run passed every step and failed one gate fault: the screenshot's notice, still fading over the Pause Card while
`PauseCardLeave` was held, read 3.96:1 at 1920 × 1080. The test now lets that notice go before the next page is held,
and the run after it passed whole (1 test, 121 s, nothing logged). qt-play's `device`, `pads` and `ring` pass on Vulkan
with `Play.Game.game` the session's own.

**Held to** (measured on 2026-11-08): kit-core 95 tests, qt-test 12, qt-models 37 (`cards`' routes for the search's two
states, the pointer and B; `library`'s help bars and both shoulders declared; `question`'s resume rules), emusen-qt's
software-backend tests (`cards`: keys typed on a keyboard and a key clicked narrow the grid to "1 of 8 games", no ring
on the field, the shoulders to the covers and back, the pointer on a cover and a click on the band, B erasing four
letters then closing, X keeping the words, B clearing them then going to Choose a console; GamesSearch and
GamesSearchGrid at the gate's five sizes; `program`'s QML rules and `qmllint`; the rest unchanged), clippy clean.
Seeded faults, each reverted: the shoulders kept for sections with none (kit-core's `input`); B on the covers back to
the grid (the models' `cards`); Resume asking (the models' `question`); the covers' focus not held by the Binding (the
program's `cards`: the found game not focused). A restored seeded file keeps the copy's older time, so cargo kept the
faulted build until the file was touched again; the runner now touches each file it puts back.

---

## 8. Testing

### 8.1 Two headless tiers

1. **Layout, focus, pad and words**: `QT_QPA_PLATFORM=offscreen` with `QT_QUICK_BACKEND=software`; no GPU; runs anywhere.
   The rules of `EmuSen_InterfaceSpec.md` §4 (fit, pad reach, names a reader hears, palette reach) re-implemented once in
   `emusen-qt-test` over the item tree: `objectName` for names, `mapToScene` rectangles, `Accessible.name` and `role`, the
   words from `Text`, colours by `grabImage` in the software backend. Effects (`MultiEffect`, `ShaderEffect`) need a GPU
   adapter, so the audited states avoid them or they are stubbed under the software backend. Two rules join them for
   §1.7: **pointer reach**, on every recorded state a click at the centre of each element the walk reaches must hit that
   element (nothing covers it) and make the same view-model call as the pad's A there, and the wheel must move every
   list that scrolls; and **no chrome**, in the big-screen and card forms no menu bar or status bar item is visible.
2. **The picture**: `QQuickRenderControl` on Vulkan on the RX 6800, the ring's cases drawn through `GamePicture` and read
   back; one GPU test at a time, the validation layer on. Since 2026-11-08 the program's acceptance runs here too, in a compositor of the
   test's own, so the screens' pictures (rounded corners, the paused frame) are judged as a player sees them (§7.1, step 6).

### 8.2 The mockups and the recordings as oracles

- **The mockups are the oracle for the reference theme.** Each approved canvas (Choose a console, Home, the grid at
  1280 × 800 design units) is held as numbers: every named element's rectangle, colour and words, written into a
  `.mockup.spec` by hand from the canvas once, and the Qt screen's recording compared to it within two design units and
  exact colours for tokens. "Rendered perfectly" is this comparison passing, plus a human look at the picture on both
  machines before a stage lands.
- **The InterfaceSpec recordings** (`EmuSen.WiseMan/Mistress/Specification/`, `spec 1`) are the C# screens' contract. The
  Rust harness writes the same format from the Qt tree; a Qt screen's acceptance is the comparison with the C# recording
  of the screen it replaces at 1280 × 800 and 1920 × 1200 within §7.3's tolerances there. **Recordings are change
  detectors only**: a difference is read and either the Qt side is fixed or the recording is re-made with its reason in
  the stage's section, never tuned to pass. Where a C# recording carries one of §0.2's measured defects (the status bar
  beneath a big-screen view; a screen a pointer cannot use), the Qt side follows §0.2 and the stage's section names the
  difference.
- **Behaviour** (what a press does to a setting, what a scan finds) is tested in Rust view-model tests with a fake client,
  no Qt at all; that is where most of WiseMan's 43,000 lines of Avalonia UI tests belong, and they are not ported line by
  line.

### 8.3 Qt Quick Test

Qt Quick Test's `TestCase` is JavaScript; its `mouseClick`, `keyClick`, `grabImage`, `tryCompare` and `findChild` are the
verbs the harness needs, but logic in QML is against §1.3. The harness is Rust: `emusen-qt-test` exposes the same verbs
through a small `extern "C++Qt"` block (`QTest::keyClick`, `QTest::mouseClick`, `QQuickItem::grabToImage`,
`QQuickWindow::grabWindow`, `qWaitForItemPolished`), plus the pad router's queue. Qt Quick Test itself is used for one
thing: the style's gallery, where a `TestCase` per control file checks `grabImage` against the tokens, since that is
pixel work with no logic. One `QGuiApplication` per process, so one process per test binary, the suite serial.

### 8.4 Checkers that run on every commit

`qml-rules` (§1.3) over every `.qml`; `qmllint` with the project's settings; the `.qmltypes` diff (§5.3); `cargo deny` on
each crate's dependency tree against §2's table; the fit audit, pad reach, pointer reach and no chrome (§8.1) at the four
sizes on every recorded state; `theme-check`
on the themes in the repository; the esde corpus TSV diff (§4.4).

---

## 9. Packaging and the handheld

Kept short by decision **[D 2026-11-07]**: there is no release before a true 1.0, so this is the plan, not a pipeline.

### 9.1 Development builds

On the distribution's Qt: Fedora 44's 6.11.2 (`qt6-qtbase-devel`, `qt6-qtdeclarative-devel`, `qt6-qtsvg-devel`,
`qt6-qtshadertools-devel`; `qmake6` at `/usr/bin/qmake6`, which cxx-qt finds through `QMAKE`). The Qt floor for building is
**6.8** (`QQuickRhiItem` 6.7, `TableView` editing 6.5, `MultiEffect` 6.5); the shipped Qt is the current minor.

### 9.2 Distribution: an AppImage and a Flatpak, from one build **[D 2026-11-07]**

- **AppImage.** The program's release binary, Qt's libraries and the QML modules it imports bundled beside it and found
  by rpath (`$ORIGIN/../lib`) and `qt.conf`, built by `linuxdeploy` with its Qt plugin (which runs `qmlimportscanner` and
  copies the imports) and `appimagetool`; the `wayland` and `xcb` platform plugins both bundled; `QSG_RHI_BACKEND=vulkan`
  set by the program. Built on the oldest glibc the targets need: a bundled desktop Qt carries the build machine's glibc
  floor (the desktop's `libQt6Quick` needs `GLIBC_2.43`, measured 2026-11-07), so the release AppImage is built in a
  container of an older base, not on the desktop. The spike's AppImage (§10.5) is the first test of the route, on the
  handheld, where both machines are at glibc 2.43 today so the floor does not bite.
  **The kit's gallery as an AppImage** (stage 0; `EmuSen_Kit.md` §7.13) is built by `cargo xtask appimage --out DIR`,
  with `PATCHELF` naming the distribution's patchelf. **The tools, pinned** (approved by the tester on 2026-11-08 to be
  downloaded from their official GitHub releases and run on this desktop to package our build, nothing else). Each is
  kept in `~/.cache/emusen/toolchains/appimage/` and checked against its sha256 before every run, from the one list
  `xtask/appimage-tools.tsv`:

  | File | Release | URL | sha256 |
  |---|---|---|---|
  | `linuxdeploy-x86_64.AppImage` | 1-alpha-20251107-1 | https://github.com/linuxdeploy/linuxdeploy/releases/download/1-alpha-20251107-1/linuxdeploy-x86_64.AppImage | `c20cd71e3a4e3b80c3483cef793cda3f4e990aca14014d23c544ca3ce1270b4d` |
  | `linuxdeploy-plugin-qt-x86_64.AppImage` | 1-alpha-20250213-1 | https://github.com/linuxdeploy/linuxdeploy-plugin-qt/releases/download/1-alpha-20250213-1/linuxdeploy-plugin-qt-x86_64.AppImage | `15106be885c1c48a021198e7e1e9a48ce9d02a86dd0a1848f00bdbf3c1c92724` |
  | `appimagetool-x86_64.AppImage` | 1.9.1 | https://github.com/AppImage/appimagetool/releases/download/1.9.1/appimagetool-x86_64.AppImage | `ed4ce84f0d9caff66f50bcca6ff6f35aae54ce8135408b3fa33abfc3cb384eb0` |
  | `runtime-x86_64` | 20251108 | https://github.com/AppImage/type2-runtime/releases/download/20251108/runtime-x86_64 | `2fca8b443c92510f1483a883f60061ad09b46b978b2631c807cd873a47ec260d` |

  The sizes matched the releases' (19,421,688, 12,317,424 and 15,092,216 bytes), and two of the hashes the digests
  GitHub publishes for the assets; the Qt plugin's release of 2025-02-13 publishes none, so its hash is the one computed
  on download (fetched 2026-11-08). The runtime is the type2 runtime appimagetool would otherwise download at every build, unpinned, from the
  moving `continuous` release (it failed with HTTP 504 on 2026-11-08); the xtask passes it with `--runtime-file`, so
  packing needs no network, and its 944,632 bytes and sha256 match the digest GitHub publishes for the asset (fetched
  2026-11-08 through the release-asset API, the download URL answering 504). The fifth pin is patchelf, below.

  **linuxdeploy's own patchelf breaks Fedora 44's libraries** [M 2026-11-08]. linuxdeploy sets each bundled library's
  RUNPATH to `$ORIGIN` with the patchelf it carries, 0.15.0, and that rewrite corrupts libraries whose relocations are
  packed in a `.relr.dyn` section, as Fedora 44 builds them: the first bundle segfaulted before `main`, in a bundled
  library's initializer. Under gdb the crash was in `libudev.so.1` (RUNPATH `$ORIGIN`, differing from the system's copy
  from byte 57); with the system's unpatched copy put in its place the crash moved to the next patched library,
  `libcbor.so.0.13`. linuxdeploy takes another patchelf from `$PATCHELF`, so the build uses the distribution's, 0.18.0
  (Fedora's `patchelf-0.18.0-9.fc43.x86_64`, fetched with `dnf download` from the configured repositories, its signature
  checked with `rpm -K` against the Fedora key on this machine, the binary's sha256 `d241dc20…63f3` pinned as the fourth
  row), and the bundle runs. linuxdeploy's own `strip` is as old and refuses the same sections; Fedora's libraries come
  stripped, so `NO_STRIP=1`.
- **Flatpak.** A manifest on `org.kde.Platform//6.11` (freedesktop 25.08 base; Qt and its plugins from the runtime, the
  payload ours; `org.kde.Sdk//6.11` is 1.2 GB to download and 3.2 GB installed, the `rust-stable` 25.08 extension carries
  Rust 1.98.1, `org.flatpak.Builder` 138 MB [M 2026-11-07, Flathub]). Permissions, each named for its reason: `--device=dri`
  (Vulkan), `--socket=wayland` and `--socket=fallback-x11`, `--socket=pulseaudio` (Endymion's audio), `--device=input`
  (controllers through SDL; the portal has no gamepad path), `--filesystem=host:ro` is **not** asked: the ROM folder, the
  BIOS folder and the themes folder are granted by the player through the file-chooser portal and remembered as
  document-portal paths, which Galaxia records; `--share=network` for ScreenScraper and the theme and core downloads, with
  every request still gated by the player's action. On SteamOS the Flatpak is what Discover installs and updates, which
  is the route a player without a terminal has; the AppImage is the route for a player who wants one folder. Building the
  Flatpak is stage 7 and needs its own go; nothing of its SDK is installed now.
- **Not SteamOS's own Qt.** Measured 2026-11-07 on the handheld (SteamOS 3.9.5, build 20261023.100): `qt6-base`
  6.11.1-1 and `qt6-declarative` 6.11.1-3 in `/usr/lib`, with the `wayland` and `xcb` platform plugins and the Basic,
  Fusion, Material, Universal, Imagine and FluentWinUI3 styles; glibc 2.43; ICU 78.3 (the desktop's Qt links ICU 77).
  It would run the program today with no Qt bundled. It is rejected as the shipped route because the system Qt is an
  image component that any SteamOS update replaces, so the program's behaviour would change under the player with no
  change of EmuSen's; and because a patch difference (6.11.1 there, 6.11.2 here) already has the program built against a
  newer patch than it runs on, which Qt's compatibility promise covers only the other way round. **Recommended: bundle**
  (the AppImage), with the Flatpak for Discover. **[A]**

### 9.3 Performance on the handheld: the gates

Measured on the Legion Go S under gamescope on the charger (`systemd-run --user`, EmuSen not otherwise running) and on
the desktop, two rounds and a third within the noise, against the C# build of the same day with the same script; a Qt
screen replaces its C# counterpart only when every gate is equal or better, "equal" within the rounds' noise:

- start-up to the first usable screen (process start to Home's first frame with the pad accepted; the C# figure: the
  themed view cold about 490 ms after Show, the process's own start unmeasured and the first baseline to record);
- memory: RSS at the library idle for 10 s, and at the Pause Card over a running game;
- frame times scrolling the grid, 300 steps held, median and worst (the C# desktop figures: median 0.3 to 0.5 ms, worst
  6.9 to 10 ms);
- time from Play to the first picture, a hosted core and an EmuSen core;
- the picture at 60 frames a second for 60 s in the real window, dropped frames counted at `frameSwapped`
  (`EmuSen_Backend.md` §38's gate).

The handheld's GPU sits at its 800 MHz floor at the library (`project_handheld_gpu_clock`, 2026-09-28), so the scene graph's
idle cost matters: an unchanged scene is not re-rendered, and the sky's glow animates only while the focus moves.

#### 9.3.1 Stage 1 on the desktop against Mistress **[M 2026-11-08]**

**Method.** Both programs measured by the same script on the same day, one run at a time, the machine otherwise quiet.
Each ran from its own sandbox copy of the published desktop build (`out/linux-x64/Mistress`, published 2026-11-08 07:58)
with a fresh home: Mistress as published, and `emusen-qt` in release (opt-level 2, at `cfc7591b6`) placed beside the
copy's platform library, so both read the same tree, the same settings and the same cores (VenusRT for the Super
Nintendo). The settings were the same for both: the big screen in the own look, Moonrise off (on for the real-path
check), scraping off, the ROM folder the player's library (9,007 games on ten consoles), bound read-only into the
process. Every run started from the same snapshot of the home (the library read once, nothing played). Each run had a
compositor of its own, `kwin_wayland --virtual` at 1920 × 1200 with Xwayland (Mistress is an X11 client), on the RX 6800,
with a window rule making every window fullscreen, and a uinput pad (vendor 1209, product 0001) that both programs read
through SDL. A frame is a buffer commit on the program's window surface in the Wayland protocol log (`WAYLAND_DEBUG`:
emusen-qt's own, Xwayland's for Mistress). "Home shown" is the first of KWin's `ScreenShot2` grabs, taken about every
35 ms from the process's start, in which Games' focus ring stands. The script and every run's log and grabs are in
`~/.cache/emusen/tmp/qt-s1-gates/`. Two rounds each; a third for the play run, where the programs came within each
other's noise.

| Gate | emusen-qt | Mistress | |
|---|---|---|---|
| Start to Home shown | 0.30–0.34 s, 0.30–0.34 s | 0.77–0.81 s, 0.77–0.81 s | Qt better |
| Start to the first pad press answered | 2.17–2.62 s, 2.16–2.61 s | ≤ 0.95 s, ≤ 0.95 s | **Qt fails** |
| RSS, the library idle 10 s | 990, 991 MiB (871 anonymous) | 305, 307 MiB | **Qt fails** |
| RSS, the same with an empty library | 173 MiB | 255 MiB | |
| RSS at the Pause Card over a game | 1,211, 1,212, 1,212 MiB | 370, 382, 379 MiB | **Qt fails** |
| The grid, Down held 24.4 s (300 steps at the kit's 400 + 80 ms) | 1,340, 1,332 frames; median 16.9 ms, worst 69.1, 68.5 ms | 1,266, 1,262 frames; median 16.7 ms, worst 96.8, 96.9 ms | **Qt fails**: the view does not scroll |
| Play to the game's screen (polled grabs, the library gone) | 0.16–0.23 s, 0.23–0.31 s | 0.03–0.09 s, 0–0.03 s | Mistress better |
| Play to the first picture, the program's own count | 101, 104, 107 ms (`emusen-qt-play`, from the load naming the game) | no count on this build | owed |
| 60 s of play, frames presented 2–60 s after Play | 3,174, 3,170, 3,164; over 25 ms 130, 126, 128 | 3,167, 3,166, 3,175; over 25 ms 117, 112, 121 | equal |
| 60 s of play, the program's own counters | `emusen-qt-play`: 3,295, 3,277, 3,291 swaps, 322, 351, 373 dropped | `[fps]` lines, 57 s: 3,425 offered, 3,115, 3,107, 3,117 shown | equal |

**Findings**, numbered for reference:

1. *Choose a console stops the program for seconds at each step, in proportion to the row's games.* The frames stop
   after a press and the focus moves at the end of the stop (Qt's `qt.quick.focus` log beside the protocol log, one run):
   2.35 s for a step onto the NES (4,185 games), 0.6 s onto the Game Boy (1,913), under 0.3 s onto the Game Boy Color
   (443). A press made during a stop is applied after it, so a player's quick presses land one screen late (seen in the
   exploration runs: a Down pressed on the opening screen applied with the next A). The fan's `GameModel` is set from
   `LibraryModel.gamesOf(key)`, every game of the focused row written as JSON and read back on the GUI thread
   (`Consoles.qml`), for a fan that draws four. The start's stop of 2.0 s (frames stop 0.66 s after the start) is the same
   kind and is why the first press is answered only at about 2.6 s; with an empty library the stop is absent. Play's
   frames show the same stops (the presentation is steady only 3.4 to 6.1 s after Play, against 0.4 to 2.4 s for
   Mistress), which fits the games being read again as a game starts (§7.1, step 6); not traced further.
2. *The grid does not follow the focus.* Down held for 24.4 s on All Games moved the selection to "Cosmic Spacehead (E)
   [h1]" (the caption follows), but the view still shows the first two rows with no focused cover
   (`r1-scroll-qt/03-end.png`). `GameGrid.qml`'s `GridView` sets `highlightFollowsCurrentItem: false` and is never
   positioned at the current index. So the gate's frame times are not of scrolling; Mistress's are (its worst 97 ms).
3. *Memory grows with the library:* about 820 MiB of anonymous memory for 9,007 games (173 MiB empty, 990 MiB with
   the library), against 50 MiB for Mistress (255 against 305).
4. *The 60 s picture cannot be judged in this compositor.* Both programs lost 9 to 10% of their frames to the same cause:
   the compositor's frame callbacks came about 54.7 times a second (`emusen-qt-play` without the protocol log: 3,281
   swaps in 60 s, 372 dropped, the same as with it), where §1.8.4's compositor gave 60. The two programs are equal here;
   the gate itself waits for the real desk (measured there in §9.3.4: both pass).
5. *Play to the first picture* has no own count in Mistress on this build: `EMUSEN_PRESENT_TRACE` wrote no file. The
   polled grabs time the library's leaving, not the picture (the game's first frames are black), so the comparison of
   first pictures is owed to the real desk with a count on both sides.
6. *The game picture fills the window.* On the 16:10 screen emusen-qt stretches the Super Nintendo's picture to the full
   width, where Mistress keeps the console's 4:3 with bars at the sides (`r1-play-*/04-playing.png`).

**The real path** (`runs/realpath-qt/`, its grabs `00-` to `13-`): emusen-qt from Moonrise to Home, Choose a console, the
Super Nintendo, ActRaiser (U) [!] and Play; the game ran with its sound into SDL's dummy driver and no problem
reported; Start on the pad reached the title's menu and Down moved its cursor; the chord opened the Pause Card;
Leave ▸, Back to Library went Home, where Continue showed "● Super Nintendo · Paused just now" over the paused frame,
and "Paused 1 minute ago" a minute later. Passed. Seen on the way: Moonrise shows a bare sky for its first two seconds,
until the systems arrive after the start's stop (finding 1); the Rewind row says "nothing to go back to yet" (§7.1,
owed with Rewind's page); Mistress's status line counted "0 controllers" while it read the same pad.

**Not measured here:** the hosted core's Play to the first picture, measured 2026-11-08 in §9.3.3.

**The defects' mechanisms and fixes** **[M 2026-11-08]**, by the findings they answer:

- *Finding 1, the stops on Choose a console, at the start and at Play,* were the fan. `CoverFan`'s `Repeater` took the focused row's
   whole `GameModel` and hid all but four delegates, so a row of 4,185 games made 4,185 `Cover`s (each a `Shape`, an
   `Image` and a generated card) on the GUI thread, and All games, selected at the start, made 9,007 while the program
   started; `gamesOf` also found each game's cover, hero art and record by a linear search, quadratic in the library.
   Proved by `qt-app/tests/scale.rs` on the build before the fix: All games fanned 9,000 delegates for 4. Fixed: every
   row's games are written once a refresh off the GUI thread and each row carries its first four as `fan` (§7.1);
   `GameModel` parses off the GUI thread. The longest stop of the frames in the 3 s after the step onto the NES:
   2,345 ms before, 67 ms after, the compositor's own gaps (`runs/focus-qt`, `runs/fix-focus-qt`). Play's presentation is
   steady 0.40 and 0.61 s after Play, where it was 3.4 to 6.1 s.
- *Finding 2: the grid* now moves its two rows when the focus leaves them, a whole row and at once (§7.1); the grab after 300
   steps shows the focused "Cosmic Spacehead (E) [h1]" on the second row (`runs/fix-r1-scroll-qt/03-end.png`).
- *Finding 3: memory* was the same delegates: offscreen on the software backend over the same sandbox, 1,028 MiB before the fix
   and 181 MiB after; in the gate's run 201 MiB idle (82 anonymous, where 871 were). Covers are decoded at their drawn
   size and released with their delegates; the sandbox has no media store, so that change is not in these figures.
- *Finding 6: the picture's shape.* The session reports the runtime's display aspect (the systems table's shape, else the core's
   declaration), else the frame's own shape with its repeated rows, as `pictureAspect`, and `Game.qml` centres the
   picture at that shape, as Mistress's `ComputeLetterboxRect` does; ActRaiser is drawn 1,600 × 1,200 between black
   bars on the 1,920 × 1,200 screen (`runs/fix-r1-play-qt/04-playing.png`), as Mistress draws it.
- *Finding 1, the first press:* the first attempts fell in the start's stop of finding 1 and were taken late; the driver's rule presses
   only when the window has been still for 0.1 s, and Home now presents frames continuously (its sky's
   `FrameAnimation`, present before the fix from 2.5 s on), so the rule waits its full 5 s. Two runs pressing as Home
   stood measured the answer from the focus log instead: pressed at 0.325 and 0.353 s, the focus on the next place at
   0.345 and 0.369 s. (The continuous frames were two causes, the sky's clock and the game picture's loop, fixed in
   §9.3.2; since then the driver's own rule answers at 0.65 to 0.70 s on its first attempt.)
- *The real path's bare sky on Moonrise* was the start's stop of finding 1 (no frame for 2 s from 0.66 s) after a clock held until the
   systems arrived. The clock now runs from the first frame (§7.1); the systems arrive within 2 to 15 ms (timed over the
   real library), long before the row of worlds at 1.78 s. Grabs at 0.62, 1.17, 2.23, 3.30 and 5.37 s after the start
   show the draft's stars, the moon's outline, the worlds' row, the halo and the sweep in turn (`runs/fix-moonrise-qt/`).

**Re-measured** (`runs/fix-r1-*`, `runs/fix-r2-*`, the same script and sandbox, Mistress's figures of the morning kept):

| Gate | emusen-qt after the fixes | Mistress | |
|---|---|---|---|
| Start to Home shown | 0.25–0.30 s, 0.30–0.34 s | 0.77–0.81 s | Qt better |
| Start to the first press answered (pressed as Home stood, focus log) | 0.345, 0.369 s | ≤ 0.95 s | Qt better |
| RSS, the library idle 10 s | 201, 201 MiB | 305, 307 MiB | Qt better |
| RSS at the Pause Card over a game | 249, 249 MiB | 370, 382, 379 MiB | Qt better |
| The grid, Down held 24.4 s, scrolling | 1,335, 1,331 frames; median 16.6, 17.0 ms; worst 82.6, 70.0 ms | 1,266, 1,262 frames; median 16.7 ms; worst 96.8, 96.9 ms | Qt better |
| 60 s of play, frames presented 2–60 s after Play | 3,169, 3,171; over 25 ms 127, 125 | 3,167, 3,166, 3,175; over 25 ms 117, 112, 121 | equal |

**Held to** (measured 2026-11-08): `qt-app/tests/scale.rs` (9,000 synthetic games as hard links to one image: each step
on Choose a console fans four covers, holds the scene at the same number of items on rows of 9,000, 6,000 and 3,000
games and returns within 400 ms; the grid of 9,000 read, Down 30 times with the focused cover inside the view on the
second row's place, and another row's grid opening at its first row), `qt-app/tests/first_frame.rs` (the clock running
at the first frame, the worlds in before the row is due), `qt-models/tests/library.rs`'s list test (each row's text and
fan equal to `games_of` at 9,000 games), `qt-models/tests/session.rs`'s shape test, the acceptance test's letterbox at
1,280 × 800, and `qt-play`'s read-back over the picture's own item. Seeded faults, each reverted: the fan over the whole
row failed the scale test (9,000 fanned); the view's move dropped failed it (cover 10 below the view); the declared
aspect ignored failed the shape test (1.143 for 1.333); the clock held for the systems failed the first-frame test.

#### 9.3.2 A still screen is not drawn again **[M 2026-11-08]**

§9.3 requires that an unchanged scene is not re-rendered and that the sky's glow animates only while the focus moves
(`EmuSen_Interface.md` §24: Moonrise's stars held still, one glow that follows the focus). §9.3.1's runs found the
opposite: Home presented about 54 frames a second while nothing on it changed.

**Mechanism.** Three causes, each proved before it was fixed:

1. *The sky's clock.* Through the harness (offscreen, the software backend), the window's `frameSwapped` counted over
   2 s once the focus had settled, with every running animation under the window's items listed: Home 124 frames,
   Choose a console 125 to 158, the grid 159 to 160, and on every screen one running animation, the `FrameAnimation`
   of that screen's `Sky` (`homeSky`, `consolesSky`, `gridSky`). It ran whenever the sky was visible and ticked the
   stars' twinkle into all 220 rows, so every tick changed the scene. The standalone Pause Card scene was already still.
2. *A hidden list's highlight.* With the sky's clock stopped, Home still swapped 55 frames in its first 2 s. An item
   tree read every 150 ms showed one item changing: the invisible highlight of the Pause Card's `PadMenuList`, its
   width growing from −80 to 424 over about 1.3 s, which is `ListView`'s default `highlightResizeVelocity` of 400 pixels
   a second. The views set `highlightMoveDuration` to the instant token but left the resize at its default; Qt's own
   highlight animation is private to the view, so it runs while the card is hidden and is in no list of animations.
3. *The game picture's loop.* In the real window (virtual KWin, Vulkan, the gates' `run.sh` with Qt's render-loop log,
   `qt.scenegraph.time.renderloop`) the build with causes 1 and 2 fixed still rendered 54 to 55 frames a second on all
   four screens, one "frame rendered" line for each buffer commit. `GamePicture` asks for the next frame at every
   `frameSwapped` while `live` (`EmuSen_Kit.md` §7.16), and `Game.qml` set `live` whenever the window had a feed, which
   it has from the start whether or not a game runs; so once any frame was drawn, each swap asked for the next, under
   the library with no game, under the Pause Card over a paused one, and behind Home over a suspended one. The harness
   has no feed, which is why only the real window showed it.

**Fixes.** A screen's stars hold still at their twinkle for time 0, as `LunaSky` draws them, and the sky's
`FrameAnimation` runs only while kit-core's glow is not settled (`Sky.glowSettled`, `EmuSen_Kit.md` §7.5); Moonrise's
stars twinkle for the splash's length only, as before. The kit's `MenuPanel`, the style's `Menu` and `ComboBox` and the
Pause Card's list set `highlightResizeDuration` to the instant token beside `highlightMoveDuration`. `Game.qml` keeps the
picture `live` only while the session runs unpaused; paused, its last picture stands. Since no swap then comes while a
game is paused, the frame thread restarts its display clock as the game pauses (`DisplayClock::restart`, the clock as
new, §1.8.3), so the pause is not read as one long period; the first swaps after the resume read the period afresh.

**Re-measured** in the same virtual KWin with the same script (`runs/idle-before-qt`, `pause-before-qt`; after:
`runs/idle-r1-*`, `idle-r2-*`, `pause-after-qt`, `idle-cost-*`), frames presented in 5 s spans 3 s after each arrival:

| | Before (the gates' fixed build, `d7cb1000a`) | After, round 1 | After, round 2 |
|---|---|---|---|
| Home, idle | 55.0 a second (fix-r1/r2 startup: 54.8, 54.3) | 0 | 0 |
| Choose a console, idle | 54.6 | 0 | 0 |
| Choose a console on a planet, idle | 54.8 | 0 | 0 |
| The grid, idle | 54.8 | 0 | 0 |
| The Pause Card over a paused Super Nintendo game | 54.4 | 0 | |
| Start to the first press answered (the driver presses once the window is still 0.1 s) | waits its 5 s | 0.696 s, first attempt | 0.648 s, first attempt |
| The grid, Down held 24.4 s: median, worst interval | 16.6, 17.0 ms; 82.6, 70.0 ms | 17.0 ms; 81.8 ms | 17.0 ms; 74.6 ms |
| The grid: frames presented, intervals over 50 ms | 1,335, 1,331; 42, 45 | 1,296; 58 | 1,290; 59 |

The render-loop counts matched the protocol log's commits frame for frame in every span. The scroll is no slower: one
more round each with the render-loop log (`idle-cost-before-qt`, `idle-cost-after-qt`) gave a frame's work on the GUI
thread (polish, lock, sync, animations) at most 3 ms and the render thread's at most 2 ms both before and after, with the
worst interval 68.4 against 75.5 ms. The after build presents about 40 fewer frames and 15 more intervals over 50 ms; read
as steps where nothing moved (the glow keeps its hue between covers of one console), which before were filled by the
sky's frames; not traced frame by frame.

**Held to.** The harness counts `frameSwapped` and lists running animations (`EmuSen_Kit.md` §7.9); `Scene::assert_still`
waits for the animations to stop (within `SETTLE_MS`) and then requires no frame for 2 s. `qt-app/tests/idle.rs` holds
Home, Choose a console and the grid still after the focus settles, and requires the glow to ease (frames and a running
animation within 100 ms) after Down and Right on Choose a console; `pause.rs` holds the card still once in place;
`acceptance.rs`, on Vulkan over a real session, holds the Pause Card over the paused game and Home over the suspended
one still, and requires the resumed game to be drawn again (over 10 frames in 0.5 s); `kit-qt/tests/stars.rs` holds a
screen's stars to their time-0 twinkle, the splash's moving, and the glow settled after Pan; `platform-client/tests/pace.rs`
holds a restarted clock to no period across a 5 s pause. Seeded faults, each reverted: the sky's clock running whenever
visible failed `idle.rs` (the `FrameAnimation` in `homeSky` running); a screen's stars twinkling on the clock failed the
stars test; `glowSettled` never announced failed `idle.rs`; the Pause Card's resize left at its default failed
`idle.rs` (55 frames in 2 s on Home); the picture `live` under a paused game failed `acceptance.rs` (120 frames in 2 s
at the Pause Card).

**Seen, then changed (2026-11-08).** Each `Frame` (and `Game.qml` while the game is in front, and the splash) ran an 8 ms
`Timer` pumping the pad router while it had the focus, waking the GUI thread 125 times a second at an idle screen. It drew
nothing, but it woke the thread for nothing; the timers now run only while `PadRouter.pending` (an event waiting or a
button held), `EmuSen_Kit.md` §7.25.

#### 9.3.3 Play to the first picture on a hosted core **[M 2026-11-08]**

The gate §1.8.4 left owed: Play to the first picture on a libretro core, emusen-qt against Mistress by the same script.
The tester approved one core download on 2026-11-08. The NES's hosted core is Nestopia (`libretro-cores.tsv`; FCEUmm,
first named for the run, is not a core EmuSen hosts), so the run made three requests to buildbot.libretro.com and
downloaded one core.

**The sandbox.** The published desktop build of 2026-11-08 07:58 (`out/linux-x64/Mistress`: Mistress, the platform
library, the libretro host and proxy, `emusen-core-runner`, `cores.manifest`) was copied without its home into a
folder of its own, with `HOME` and the `XDG_*` folders pointed into the sandbox, and `emusen-qt-play` (release, built at
`d6a6d10d8`) placed beside the copy's platform library. Mistress's ROM folder was an empty folder of the sandbox, so
neither program read the library. Super Mario Bros. was opened in place, read-only, from the player's library
(`NES/World/Super Mario Bros (JU) (PRG 0).nes`); no ROM was copied.

**The core, as the player gets it.** The `cores` command runs only in a shell with the platform library's services and
the runtime's settings service, which today only Mistress's shell provides; `Testing/qt-gates/shell.py` gives the same
shell to the terminal over the copy's own platform library (`dianash.h`, `emusen_crystal.h`), each line of its input
submitted as typed at the prompt. Its lines and answers:

- `cores fetch`: "Fetched libretro's catalogue on 2026-11-08: 260 cores for this platform" (`info.zip` and the Linux
  x86_64 index, 0.88 s).
- `cores get nestopia`: installed build 2026-11-07, SHA-256 `72e2784b…5b0be105`, the same build §4.12 of
  `EmuSen_Cores.md` smoke-tested on 2026-11-01; every check of §5.2 passed (0.15 s); "Not tested by EmuSen".
- `cores approve nestopia`: approved; `cores verify`: `ok nestopia`.
- `cores default NES "Nintendo - NES / Famicom (Nestopia)"`: "NES now opens on Nintendo - NES / Famicom (Nestopia)".
  Both programs open the NES on it from then on; Mistress's log names the engine and its `retro_run`.

The approval is recorded at once, but the entry is read (its proxy loaded and its sidecar written) only by a process
whose executable lies beside `emusen-core-runner`, which the proxy looks for; the terminal's is the Python interpreter,
so the runtime refused the read ("no emusen-core-runner beside /usr/bin/python3.14") and the first program started from
the copy read it at its start. That is the test terminal's limit, not the installation's.

**Method.** `Testing/qt-gates/run.sh`, one run at a time, each in a compositor of its own (`kwin_wayland --virtual` at
1,920 × 1,200 with Xwayland, a D-Bus of its own, every window fullscreen by a rule), on the RX 6800, under a watchdog,
with SDL's dummy sound driver. Each program is given the game at its start: `emusen-qt-play --game`, and Mistress
`--play`, which loads the game when its window opens. KWin's `ScreenShot2` grabs are polled from the program's exec
(`grab.py`, a median 37 to 56 ms apart), and `first.py` reads three times from them: the program's window (the first grab
with more than 1% of the screen lit), the first picture (the first grab with more than 20% of the screen in Super Mario
Bros.'s sky, a colour box both cores' palettes fall in), and the span between them, which is Play to the first picture
on both sides, since each opens the game as its window appears. Each time is late by up to one grab interval.
`emusen-qt-play` also prints its own count, from the window's load, which names the game, to the first frame drawn
(§1.8.4), which may be one of the game's black frames. Three stage 2 builds were compiling beside the runs; the load is
the one-minute average from `uptime` at each run's start, on 16 threads.

| Round | Load | emusen-qt-play: window, picture, between | own count | Mistress: window, picture, between |
|---|---|---|---|---|
| 1 | 7.7, 8.0 | 0.292, 0.902, 0.610 s | 270 ms | 1.013, 1.599, 0.586 s |
| 2 | 8.5, 8.7 | 0.329, 0.885, 0.556 s | 274 ms | 0.810, 1.487, 0.677 s |
| 3 | 8.4, 9.8 | 0.284, 0.846, 0.562 s | 250 ms | 1.127, 1.898, 0.771 s |

The third round was taken because Mistress's two rounds differed by more than a grab interval and the two programs'
spans overlapped. **Play to the first picture is equal within the noise**: 0.56 to 0.61 s for emusen-qt, 0.59 to 0.77 s
for Mistress, the hosted core's start and the game's first black frames on both sides. From the program's exec to the
picture emusen-qt is 0.6 to 1.1 s sooner, which is the start (its window at 0.28 to 0.33 s against 0.81 to 1.13 s for
Mistress, whose library window shows first and gives way to the game screen). The first run of all, before the rounds
(`trial1-qt`), read the newly approved entry at its start: window at 0.93 s, picture at 1.23 s, with a slower grabber
(54 ms).

**It plays.** With the pad of §9.3.1 (`pad.py`: Start, then Right held 3 s), on both programs Start left the title for
World 1-1 and Right walked Mario on and scrolled the level; the two programs' grabs at 11.5 s show the same frame
(`pad-qt/grabs/0295.png`, `pad-mistress/grabs/0115.png`). No pad was left behind after any run. Over the
rounds' 8 s `emusen-qt-play` counted 349 to 358 swaps after its first 2 s, 0 to 12 dropped, at the compositor's own
rate (§9.3.1, finding 4); Mistress's `[fps]` lines read 60.1 offered, 58 to 59 shown.

**Seen.** `emusen-qt-play`'s window shows white for its first 0.2 s and white bars beside the picture; it is the
measuring window of §1.8, not the front end's game screen, whose bars are black (§9.3.1, finding 6).

The grabs and logs of every run are in `~/.cache/emusen/tmp/qt-hosted-grabs/`; the sandbox, its core and its settings
were removed after the runs. The scripts are committed in `Testing/qt-gates/` (`README.md`) so the gates run again
without rebuilding the method; §9.3.1's own scripts, under `~/.cache/emusen/tmp/qt-s1-gates/`, were removed with its
sandbox.

#### 9.3.4 60 seconds of play on the real desk **[M 2026-11-08]**

§9.3.1's finding 4 left the 60-second gate to the real desk, because the virtual compositor called back only about 55
times a second. Measured on the tester's own desktop, with the tester's consent, on the main monitor.

**The display.** DP-3, 2,560 × 1,440 at 165.00 Hz, scale 1.25 (2,048 × 1,152 logical), adaptive sync "Automatic"; a
second monitor (HDMI-A-1, 1,920 × 1,080 at 74.97 Hz) stayed in use beside it. KWin 6.7.5 on the RX 6800.

**Method.** `Testing/qt-gates/run.sh` with its `DESK` option (`README.md`): the session's own compositor, where
`desk.py` loads a KWin script for the run only that sends the program's window to DP-3, makes it fullscreen and
activates it, and reports every window and activation back over D-Bus; the pad is started only once the program holds
the focus; no grabs, since the session's KWin refuses `ScreenShot2` to a client it has not authorised. The sandbox was
a copy of the published desktop build of 2026-11-08 07:58 (`out/linux-x64/Mistress`) without its home, with
`emusen-qt-play` (release, built at `d6a6d10d8`) beside its platform library, `HOME` and the `XDG_*` folders in the
sandbox, the big screen on, the splash and scraping off, and the ROM folder the player's library, bound read-only into
each program by `bwrap`. Both programs played Super Mario Bros. (`NES/World/Super Mario Bros (JU) (PRG 0).nes`, opened
in place) on MoonRT, the NES's default engine (`cores default NES`: "MoonRT (Rust)"), sound into SDL's dummy driver.
Mistress ran once for 30 s first, so the library was read once, and every round started from a snapshot of the home
taken after it. Each run: `emusen-qt-play --seconds 62 --game`, counted by its own counters after its first 2 s; and
Mistress `--play` for 62 s, counted by its `[fps]` lines (one a second from the game's start) less the first two. The
uinput pad pressed Start once, about 3 s after the program took the focus, and was removed at once. Each run in a capped
scope (`CPUWeight=20`); three stage 2 builds were compiling beside the runs; the load is the one-minute average at each
run's start, on 16 threads.

| Round | Load | emusen-qt-play, 2–62 s: swaps, frames shown, dropped, repeated | Mistress, `[fps]` seconds 3 to the end: offered, shown, copied |
|---|---|---|---|
| 1 | 8.4, 5.9 | 9,886 (164.8 a second), 3,605 (60.08), 1, 6,281 | 57 s: 3,427, 3,425, 3,425 (60.09 shown a second; lowest second 58.9) |
| 2 | 6.4, 6.9 | 9,869 (164.5), 3,603 (60.05), 2, 6,266 | 58 s: 3,486, 3,485, 3,485 (60.09; lowest second 59.1) |

**Both programs show every game frame at the desk.** `emusen-qt-play` swapped at the monitor's 165 Hz and drew each of
the core's 60.1 frames a second, 1 and 2 dropped in 60 s; a repeated frame is a swap with no new frame, about 1.7 to
each frame shown at this refresh, so "repeated" counts the refresh's surplus, not a fault. Mistress drew each frame it
was offered once (shown equal to copied), 2 and 1 short of offered; its scheduler named itself "content-paced 60.10 Hz",
"refresh follows the content" for the first seconds and "refresh not measured" after (in the warm-up run, "display-locked
120.78 Hz / 2"), so whether adaptive sync lowered the monitor's rate under Mistress is not known from its side. The 9 to
10% lost in §9.3.1 was the virtual compositor's, as finding 4 read it; on the real desk neither program loses frames, and
the gate is equal, both passing.

**Seen.** Both windows stayed fullscreen on DP-3 for the whole of every run (`desk.log`). The focus moved to the
tester's own windows on the other monitor during three runs (about 23 s into round 1's `emusen-qt-play`, 2.3 s into
round 2's, before its Start, and about 53 and 47 s into Mistress's); neither program's counts changed at those moments,
and Mistress, with "pause in background" on, did not pause, the window being still in front on its own monitor. Whether
round 2's Start reached `emusen-qt-play` with the focus elsewhere is not recorded (no grabs); the counts are of the
picture either way. Steam opened a window each time the pad appeared.

The logs of every run are in `~/.cache/emusen/tmp/qt-desk/runs/`; the sandbox was removed after the runs.

---

## 10. The toolchain spike (outside the tree, `~/.cache/emusen/qt-spike/`)

### 10.1 What it is

A cargo-only cxx-qt 0.10.0 program against Fedora's Qt 6.11.2 (about 470 lines of Rust, 180 of QML and 30 of C++, the
build script included), showing the three things the design leans on:

1. **A list model in Rust**: `Consoles`, a `QAbstractListModel` subclass with twelve roles (the courts mockup's eleven
   consoles as its rows), read by a horizontal `ListView` through `required property` roles.
2. **A drawn item from Rust**: `Planet`, a `QQuickPaintedItem` subclass whose `paint()` is Rust, drawing the mockup's body
   (radial gradient, craters, bands, a polar cap, highlight, moonlets) from six properties; a property change calls
   `update()` through cxx-qt's generated connectors.
3. **A theme loaded from a QML file at run time**: `themes/night.qml` and `themes/day.qml`, `QtObject`s of token
   properties outside the binary, chosen by Rust (`Spike.themeSource`, a `file:` URL) and instantiated by a `Loader`;
   every colour, radius, duration and font family on screen binds to it; **T** cycles the files without a rebuild.

### 10.2 Figures **[M 2026-11-07]**

On the desktop (Ryzen 7 7700X, Fedora 44, Qt 6.11.2, Rust 1.99.0, GCC 16.2.1), every build in a scope of CPU weight 20
with six compilers and no `sccache`:

| Measure | Figure | How |
|---|---|---|
| Cold release build (dependencies fetched already; cxx-qt's generated C++ and Qt's headers dominate) | **61.5 s** | `cargo clean`, then `cargo build --release`, wall time |
| Rebuild after one Rust line changed | **2.1 s** | `touch src/consoles.rs`, the same build |
| Rebuild after a QML file changed (the resource is regenerated) | **2.1 s** | `touch qml/Main.qml` |
| Rebuild after a C++ helper changed | 4.6 s | the grab helper added |
| Binary, stripped, linked dynamically to Qt | **1.55 MB** | `stat` |
| Start to the first frame swapped, real window on Wayland (Plasma), three rounds | **105 to 110 ms** after process start; **131 to 135 ms** wall to exit | `--quit-after-first-frame`, `frameSwapped` → Rust → `exit(0)` |
| The same, `offscreen` platform | 30 to 34 ms; 50 to 57 ms wall | the same |
| The bundle for the handheld (§10.5): Qt's libraries, plugins and QML modules | **93 MB** on disk, of which ICU 37 MB; **36 MB** as a zstd tarball | `du`, `tar --zstd` |

The pictures `logs/grab-night.png` and `logs/grab-day.png` are the same binary with the other theme file: the sky, the
surfaces, the accent, the ring and the help bar all follow the file. The planets are Rust's painting (the Super Nintendo's
bands, the Nintendo 64's cap and two moonlets, the NES's craters); shards and gems are drawn as planets in the spike, which
draws one body kind. The fallback font (Barlow is not installed on the desktop) is wider than Barlow Condensed, so two
console names touch at the mockup's tile width: a fit-audit case the kit's harness would catch (§8.1), and the reason the
fonts travel in the binary.

### 10.3 Where the spike breaks the design's rules on purpose

`Main.qml` has three JavaScript expressions (`toUpperCase()`, string concatenation for the facts line, `Qt.quit()`) that
§1.3's checker must flag, and one `OnceLock` holding the process start time for the first-frame figure, which is
measurement scaffolding and not a design element. They are left in as the checker's first test cases.

### 10.4 Findings

- **cxx-qt 0.10.0's library (`cxx-qt-lib`) has no `QQuickItem`, `QQuickPaintedItem` or `QBrush`.** Subclassing works through
  `unsafe extern "C++Qt" { include!(<QtQuick/QQuickPaintedItem>); #[qobject] type QQuickPaintedItem; }` and
  `#[base = QQuickPaintedItem]`, with `paint` as a `#[cxx_override]` taking `*mut QPainter`, and `cxx_qt::Initialize`
  default-constructs the base. `QPainter` is bound but without brushes or gradients (`fill_rect` with a colour, `stroke_path`
  with a pen, no `fill_path`), so a gradient fill needed a C++ helper of two functions (`spike_paint.cpp`, 20 lines)
  declared in the bridge. The design counts such glue (§1.4); the alternative for the kit's drawn bodies is the scene
  graph (`QSGGeometryNode` with a vertex colour or a `ShaderEffect`), which is also the faster path and is what `Body` and
  `Sky` should use, with painted items kept for one-off drawings such as the controller.
- **QML singletons and elements from Rust** (`#[qml_element]`, `#[qml_singleton]`, `#[qproperty]`, `#[qinvokable]`) need no
  C++ at all; the module is registered by `build.rs`, and the QML files live in the resource system under
  `qrc:/qt/qml/<URI>/`.
- **Two toolchain facts found the hard way.** A `#[qenum]` may not give its variants explicit values ("not supported
  (yet)" in 0.10.0), so model roles start at 0 as cxx-qt's own example has them rather than at `Qt::UserRole`; the
  engine resolves them by name through `roleNames`, so nothing is lost. And GNU `ld.bfd` fails the link with "DSO
  missing from command line" (`libQt6Quick.so` is listed before the generated archive that needs it and `--as-needed`
  drops it; cxx-qt warns that only `mold`, `lld` or `gold` are expected to work): the cure, with no linker installed, is
  five `cargo:rustc-link-arg` lines in `build.rs` naming the Qt libraries again after `--no-as-needed`. A stage 0 item is
  to put `lld` (Fedora's `lld` package) in the toolchain, which also shortens the link. The kit found a second `ld.bfd`
  case on 2026-11-07, when a crate links a cxx-qt library as an rlib and is itself the final link: the whole-archive
  initializer members come after the rlib and the init functions go undefined. `lld` was installed the same day and
  both workarounds were removed; `EmuSen_Kit.md` §7.6 has the record and the build figures under `lld`.
- **Build times** are dominated by cxx-qt's generated C++ and Qt's headers, not by Rust: the incremental figure in §10.2 is
  the one a stage lives with.

### 10.5 The handheld bundle: an AppDir, the AppImage's content

`mkappdir.sh` builds the AppDir by hand with the system's tools, the same shape `linuxdeploy`'s Qt plugin produces: the
binary relinked with `RUNPATH=$ORIGIN/../lib` (`cargo rustc -- -C link-arg`), the libraries `ldd` names less the ones every
Linux has (glibc, the GL vendor stack, X11, Wayland, fontconfig, freetype, harfbuzz, xkbcommon: the AppImage exclude
list's rule) plus `libstdc++` and `libgcc_s`, the `wayland` and `xcb` platform plugins with Wayland's shell and
graphics integrations, three image formats, the QML modules `qmlimportscanner` lists for the program's QML (QtQuick,
Controls Basic and its impls, Templates, Shapes, QtQml and its Models and WorkerScript), `qt.conf` beside the binary
naming `lib`, `plugins` and `qml`, and an `AppRun` that exports `LD_LIBRARY_PATH` to the bundle's `lib` (a `RUNPATH` is
not transitive, so ICU 77, which the handheld lacks, is found only through the path). The themes folder travels at
`usr/share/emusen-qt-spike/themes`, since a theme is a file the program reads at run time. Checked on the desktop: the
binary resolves Qt and ICU from the bundle and draws the same picture from `AppRun` offscreen (`logs/grab-appdir.png`).
The last step, `appimagetool AppDir emusen-qt-spike-x86_64.AppImage`, needs the tool downloaded from its releases, which
this machine's rules did not allow the spike to fetch and run; the AppDir runs as it is, and the tarball is what travels.

### 10.6 Commands for the handheld (SteamOS 3.9.5, glibc 2.43, both platform plugins present)

```
# desktop → handheld
scp -i ~/.ssh/emusen_handheld ~/.cache/emusen/qt-spike/emusen-qt-spike-AppDir.tar.zst deck@10.1.1.205:~/.cache/emusen/
# handheld, in a terminal (desktop mode), nothing of EmuSen running
mkdir -p ~/.cache/emusen/qt-spike && cd ~/.cache/emusen/qt-spike && tar --zstd -xf ../emusen-qt-spike-AppDir.tar.zst
QT_QPA_PLATFORM=offscreen ./AppDir/AppRun --quit-after-first-frame --grab offscreen.png     # no window: libraries, QML, the picture
./AppDir/AppRun --quit-after-first-frame                                                    # a window for one frame: prints the ms
./AppDir/AppRun --theme night --grab handheld-night.png                                     # a window; T swaps the theme, Q quits
# under gamescope (Game Mode), as a non-Steam shortcut or: QT_QPA_PLATFORM=wayland ./AppDir/AppRun
```

What the run should show: `first frame swapped N ms after process start` on each run; `grab …: saved`; and in the window
the eleven tiles with their planets, the focused one ringed, and the help bar on the foot; **T** turns the sky light. The
three figures to record are the offscreen and windowed first-frame times and, under gamescope, whether the window is
shown at the panel's size with the Wayland plugin (the log line `QT_QPA_PLATFORM`, or the `xcb` fallback taken).

### 10.7 The handheld run **[M 2026-11-07]**

The AppDir was run on the Legion Go S (SteamOS 3.9.5, under gamescope, `DISPLAY=:0`) on 2026-11-07, from the tarball,
with nothing of EmuSen running: it starts from the bundle's own Qt and ICU, draws, and quits.

| Measure | Handheld | Desktop (§10.2) |
|---|---|---|
| First frame, `offscreen` | **41 ms** after process start | 30 to 34 ms |
| First frame, a window | **146 ms**, then **125 ms** (two rounds) | 105 to 110 ms |

The grab saved, and showed the day theme although the picture was named for the night: the run passed no `--theme`,
and without one the program takes the first file of the themes folder in name order, which is `day.qml` before
`night.qml`; the desktop runs of §10.2 had named `--theme night` and did not show it. The command above now names the
theme. The lesson is the design's own rule for the real program: a default is a stated choice (the reference theme by
name, stored in Settings), never the order of a folder listing.

**Game Mode shows only what Steam started** (measured 2026-11-08). The kit's gallery as an AppImage, started over ssh
with `DISPLAY=:0` while the handheld was in Game Mode, rendered (its `--grab` saved), but gamescope kept it behind
Steam's interface: `gamescopectl`'s screenshot showed Steam's game page. An on-screen check in Game Mode therefore needs
the program added to Steam as a non-Steam shortcut and started from Steam. The handheld is a Lenovo Legion Go S; its
panel is 1920 × 1200 (a BOE NS080WUM-LX1, 8 inches), not the Steam Deck's 1280 × 800, and that grab is what
`EmuSen_Kit.md` §7.13 reads the grab's timing from.

---

## 11. Risks and open questions

**Risks**, largest first: (1) the one-queue protocol between Serenity's submissions and Qt's RHI (stage 1's first day); (2)
the ES-DE engine's video and Lottie families, where the licence and the decoder are undecided (Q5, Q6); (3) the theme
sandbox is an engine-level boundary, not a process one (§3.5); (4) cxx-qt's gaps (no scene-graph or painted-item bindings
in its library, no attached properties) are each filled with glue, each part with its reason; (5) a QML-layout-only rule erodes without the checker
from day one; (6) Qt's open-source LTS access is short, so a security fix may mean the next minor mid-way; (7) the
estimate carries two unknowns (the picture protocol, the ES-DE painter's effects stack).

**Questions**, each with a recommendation. **Decided 2026-11-07 by the tester: every recommendation below stands as
written, Q1 to Q12, and the design as a whole is approved.** For Q9, Pegasus is abandoned and retires with the C# front
end. The questions are kept as asked, with their recommendations, as the record of what was decided.

1. **[decision] The theme API's screens.** Three (Home, Consoles, Games) owned by a theme, as §3.2; or also the game's page's
   art header? Recommended: the three, with the art header reading the theme's tokens only, since the page is EmuSen's by
   `EmuSen_Interface.md` §7.
2. **[decision] Token overrides by themes.** Allowed (values only, contrast-checked, §3.4) or not at all? Recommended:
   allowed; the console-style designs need their accent and surfaces, and the check keeps the floors.
3. **[decision] The sandbox's strength.** An engine-level sandbox (§3.5) and a reviewed list, or a process per theme with
   its picture shared through the ring? Recommended: engine-level now; a process is a later form if a theme store ever
   admits unreviewed themes.
4. **[decision] The client boundary.** The front end over the two C headers through generated bindings (§1.1), or a direct
   Rust dependency on the runtime crates? Recommended: the headers, for the audit and the interchangeable front ends.
5. **[decision] Video in themes.** Qt Multimedia with an LGPL FFmpeg bundled, or a Rust decoder into a texture?
   Recommended: Qt Multimedia, behind the kit's `VideoTile` so a theme never imports it.
6. **[decision] Lottie animations in ES-DE themes.** Qt's Lottie module is GPL-only in the open edition; a Rust Lottie
   renderer (rlottie bindings or a pure-Rust crate) into a texture is the alternative. Recommended: defer; no watched
   theme uses one (`EmuSen_BigPicture.md` §3), and the corpus will say when it matters.
7. **[decision] Which console-style themes first.** Recommended: Deck-like and Switch-like, the two whose arrangements
   differ most from the reference (a left column; a strip with round places), so they exercise the most of §3.2.
8. **[decision] No native plugins into the front end** (§5.4). Recommended: hold to it; a feature that needs native code is
   a DianaSh command or a core.
9. **[decision] Pegasus**, the notepad and the only F# assembly: retire with the C# front end, or rebuild on the kit?
   Recommended: retire unless a consumer is named.
10. **[decision] The drawn bodies' path**: scene-graph nodes (fast, more code) or painted items (simple, a texture upload
    per change)? Recommended: scene graph for `Sky` and `Body` (they animate), painted items for the controller drawing.
11. **[routine] The program's name**: `emusen-qt` until the C# front end retires, then `EmuSen`. Recommended as stated.
12. **[routine] Licence of the kit and the theme API**: GPL-3 with the application; themes under their authors' licences.
    Recommended as stated, since the kit is no longer a separate toolkit.

---

## 12. Sources (opened 2026-11-07 unless dated)

- cxx-qt: the book's inheritance chapter (https://kdab.github.io/cxx-qt/book/concepts/inheritance.html), the `extern
  "RustQt"` reference (https://kdab.github.io/cxx-qt/book/bridge/extern_rustqt.html), the cargo executable chapter
  (https://kdab.github.io/cxx-qt/book/getting-started/4-cargo-executable.html), the traits page
  (https://kdab.github.io/cxx-qt/book/bridge/traits.html); crates.io (`cxx-qt` 0.10.0, 2026-08-24); docs.rs for
  `cxx-qt-lib` 0.10.0 (`QPainter`, `QPainterPath`, `QPen`, `QColor`, `QCoreApplication`) and `cxx-qt` 0.10.0
  (`Constructor`); the `qml_features` example's `custom_base_class.rs` and `build.rs` at tag v0.10.0.
- Qt 6: the pages the 2026-11-01 plan cites for the scene graph, `QQuickGraphicsDevice`, `QSGVulkanTexture`,
  `QQuickRhiItem`, `QQuickRenderControl`, Qt Quick Test, customising Quick Controls, Qt Virtual Keyboard's licence, the
  release policy; `QQmlEngine` (import paths, URL interceptor, network access manager factory,
  https://doc.qt.io/qt-6/qqmlengine.html).
- ES-DE: `THEMES.md` at master (https://gitlab.com/es-de/emulationstation-de/-/raw/master/THEMES.md); the theme list
  (https://gitlab.com/es-de/themes/themes-list/-/raw/master/themes.json, `latestStableRelease` 52).
- Flathub: `org.kde.Platform` and `org.kde.Sdk` 6.11, `org.freedesktop.Sdk.Extension.rust-stable` 25.08,
  `org.flatpak.Builder` (sizes by `flatpak remote-info`, 2026-11-07).
- Fedora 44: `qt6-qtbase` 6.11.2-3, `qt6-qtdeclarative` 6.11.2-2; glibc 2.43; the GLIBC symbol versions by `objdump -T`.
- SteamOS 3.9.5 (build 20261023.100), read on the handheld 2026-11-07: `qt6-base` 6.11.1-1, `qt6-declarative` 6.11.1-3,
  glibc 2.43, icu 78.3-1, the platform plugins and styles listed in §9.2.
- EmuSen: `EmuSen_Interface.md` §0, §5, §7, §21 to §24; `EmuSen_BigPicture.md` §0, §2, §25, §31; `EmuSen_InterfaceSpec.md`
  §1, §4, §7; `EmuSen_Backend.md` §40; `EmuSen_Surfaces.md` §6; `EmuSen_LibretroHost.md` §1, §4.4, §4.8;
  `emusen_crystal.h` 1.13, `emusen_platform.h` 35, `emusen_core.h` 1.19; `Platform/systems/data/`; the libretro host's
  `machine.rs` and `front.rs`; the approved mockups (Choose a console, Home, the grid, 2026-11-05).
