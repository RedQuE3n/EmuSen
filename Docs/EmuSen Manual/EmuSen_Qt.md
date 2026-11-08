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
- *The library* (`emusen_library.h` 1.2): a handle on games.db over the SQLite lent; scan, progress, cancel, the last
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
the interface player's `decode`, `decode_take` and `volume`; and all of Serenity (presets, sources, the compiler,
reflection, the cache, the device, the lend, the chain, the ring and the built-in filters). Each is wrapped when a
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

### 1.9 Homes for the front-end logic still in C#

The survey of the remaining C# (2026-11-07) lists front-end logic that needs a Rust owner. Each is given one layer here,
by the one-owner rule, and none goes into the view models unless it is a view model's job.

| Logic (C# lines) | Home | Why there |
|---|---|---|
| The ES-DE theme engine: loader, scene, gamelists, collections, folders (about 4,000, plus 3,900 of tests) | `emusen-esde` (§4.2), a crate of its own, with `EmuSen.Esde` as its painter | one job, no Qt in the loader or scene, testable alone |
| The theme store: `themes.db`, GitHub and GitLab downloads, the browser's records, attribution (about 1,150) | `emusen-themes`, a platform crate over `emusen-net`, the zip reader and SQLite, exposed as the DianaSh command `theme` (`list`, `install`, `update`, `remove`, `check`), which the Themes tab fronts; it serves both theme kinds | the 2026-10-26 rule: every tool a command; one store for both kinds |
| The four binding files: keybindings, hotkeybindings, gamepadbindings, gamepadprofiles (about 800) | Endymion, through the platform library (`emusen_endymion_*`) and Galaxia's models; the front end's `DeviceModel` and `SettingsModel` read and write them through `emusen-platform-client` | Endymion is the one owner of pads; both front ends share the files, so this is stage 0. Done 2026-11-08 at interface 40 (`EmuSen_Input.md` §5.2) |
| Frame pacing: `FrameScheduler`, `DisplayClock`, `PictureSlots`, `PresentationTrace`, `VblankCounter`, `FrameHandOff`, `FramePacer` (about 650) | Serenity (`emusen-serenity::pace`), fed the window's `frameSwapped` stamps by `GamePicture` and read by the emulation thread's frame loop | `EmuSen_Surfaces.md` §6.2 already makes Serenity the owner of the display clock and the pacer; the view models only relay stamps |
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

### 3.2 What a theme sees: the `EmuSen.Theme` API

`import EmuSen.Theme 1.0` gives a theme exactly these, each a QML singleton or type backed by Rust, each a view over
`emusen-qt-models` with the fields a theme may read and nothing more.

| Name | Kind | What it holds |
|---|---|---|
| `Library` | singleton | `consoles` (a model: `id`, `name`, `displayName`, `maker`, `year`, `count`, `hue`, `body`, `engineName`, `engineKind` = `emusen`/`libretro`, `engineMark`), `places` (All, Favourites, Recent with counts and sublines), `recent`, `favourites`, `collections`, `gamesOf(console)` (a model: `title`, `cover` URL, `coverShape`, `favourite`, `lastPlayed`, `played`, `consoleId`, `consoleHue`, `players`, `year`, `genre`, `rating`, `description`, `video` URL), `search(text)` |
| `Session` | singleton | `hasGame`, `title`, `consoleId`, `pausedSince`, `played`, `frame` (the paused picture as a texture item) |
| `Device` | singleton | `controllers` (count, family for glyphs), `clock`, `battery`, `charging`, `network`, `bigScreen`, `form` |
| `Actions` | singleton | `play(game)`, `openPage(game)`, `favourite(game)`, `random(console)`, `openConsole(id)`, `openPlace(id)`, `home()`, `back()`, `stepSection(±1)`, `openHomeCard()`, `openListCard()`, `search()`; each returns whether it was taken |
| `Cues` | singleton | `scroll`, `systembrowse`, `select`, `back`, `favorite`, `launch`, `quicksysselect`: the seven names of `EmuSen_Interface.md` §23, played through Endymion at the player's volume; a theme may supply its own files for them in `theme.json` |
| `Tokens` | singleton (the kit's) | every token of `EmuSen_Interface.md` §5 in the current form and variant; read-only to a theme, overridden only through `theme.json` (§3.4) |
| `Frame`, `HelpBar`, `Body`, `Sky`, `Carousel`, `CoverTile`, `NormalizedCanvas` | types (the kit's) | the pieces a theme places; `Frame` carries the walk, so a theme that places roles gets the pad for free |
| `Screen` | type | the root of one of the three screens a theme provides; its `role` names which, and its `focusRole` the role the walk enters first |

A theme provides `screens/Home.qml`, `screens/Consoles.qml` and `screens/Games.qml`, each a `Screen`; a theme that omits
one gets the reference theme's. Everything on them is bound to the singletons above; the pad reaches them through the
`Frame` they place or, where a theme arranges freely on a `NormalizedCanvas`, through `Screen.focusItems`, an ordered
list the walk uses. The models are read-only; every change goes through `Actions`.

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

### 3.6 Console-style designs as themes: the test that the API is sufficient

Four designs, each **inspired** by a console's shell (its arrangement and motion, never its logos, glyph sets, fonts or
assets), each written against §3.2 with no addition to it. If any of the four needed a new singleton or role, the API
would be too narrow; none does. **[A]**

| Design | Arrangement (general patterns only) | What it reads | What it places |
|---|---|---|---|
| **Deck-like** | a left column of places and consoles with the focused one expanded; the content a horizontal band of large art tiles with the focused tile's details beneath; a dark surface with a cool accent | `Library.places`, `Library.consoles`, `gamesOf`, `recent`; `Device.controllers`, `battery`, `clock`; `Session` for a Continue tile | `Frame` (navigation + content + detail), `CoverTile`, `HelpBar`; tokens: `accent`, `surfaceRaised` overridden |
| **Xbox-like** | rows of blocks: a large Continue block, a row of recent covers, square tiles for places; blocks slide as the focus moves | `Session`, `recent`, `places`, `collections`; `Device` | `NormalizedCanvas` with `Screen.focusItems`; `CoverTile`; the walk through `Frame` roles `content` and `actions` |
| **PlayStation-like** | one horizontal strip of console or game icons along the top third, the focused one larger with its name under it; the lower two thirds show the focused item's art and facts; a soft moving background | `consoles` or `gamesOf`, `hue` for the background's tint, `description`, `video` | `Carousel` (over `PathView`), `Sky` with the glow following `hue`, a `VideoTile` where a clip exists |
| **Switch-like** | a horizontal strip of game tiles at mid-height, round places beneath (favourites, recent, search, settings), a light or dark variant; a cue on every move | `gamesOf(All)`, `places`, `Cues`, `Tokens.variant` | `Carousel` horizontal with equal tiles, `WholeRow` for the round places, `HelpBar`; `Cues.scroll` on move, `select` on A |

Each of the four is also a fit-audit and pad-reach case (§8): a theme passes when every operable item is reached from
the first focus and nothing is cut at 1280 × 800, 1920 × 1080, 1280 × 720 and 3840 × 2160. The two or three the tester
picks are built in stage 3 (§7) as the first third-party-shaped themes, and they stay in the repository as the API's
regression tests.

### 3.7 `theme-check`

A DianaSh command in Rust (the 2026-10-26 rule): `theme-check <folder>` reads `theme.json`, holds it to the schema and the
API version, lints the QML against the import allow-list and §1.3's rule, checks token overrides for type and contrast,
loads each screen offscreen on the synthetic library and runs the fit and reach audits (pad and pointer, §8.1) at the
four sizes, and prints one line per finding (`file:line kind words`) and a last line `ok` or `N findings`. The Themes tab runs it before offering
"Use", as the install gate; the compatibility corpus of §4.4 runs the ES-DE equivalent.

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
   back; one GPU test at a time, the validation layer on.

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

  The sizes matched the releases' (19,421,688, 12,317,424 and 15,092,216 bytes), and two of the hashes the digests
  GitHub publishes for the assets; the Qt plugin's release of 2025-02-13 publishes none, so its hash is the one computed
  on download (fetched 2026-11-08). The fourth pin is patchelf, below.

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
