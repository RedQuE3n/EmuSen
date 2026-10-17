# EmuSen — the backend in Rust: the line, the frontend API, the system packs and the services

*This revision: the first, 2026-10-17. A design; B1 has since been built (§12, 2026-10-17). It answers the tester's goal of 2026-10-16, "to
get the entire backend ported to rust", after which the frontend's move is planned, and the request of 2026-10-17 to
design two things: one frontend API by which every frontend drives the cores, and the system packs and the remaining C#
platform services in Rust. Every claim about the code was read on `WiseMan` at `165e60b4`. Claims are marked as
`EmuSen_GamePackages.md` marks them:*

- **[M]** *measured*: counted on 2026-10-17 with the command named (`wc -l` over `.cs` files outside `bin/` and `obj/`,
  `grep` over the sources), or read from the file and line named.
- **[R]** *read* from another page of this manual, named.
- **[A]** *argued* from the code and the pages, with no measurement behind it; a step's gate or a prediction of §9 says
  how it is to be shown.
- **[C]** *a choice*: a recommendation awaiting the tester (§11).

Companion pages: `EmuSen_Stack.md` §6 (the plan this completes), `EmuSen_Surfaces.md` (§4.1 the three frame loops, §6
the stability rules, §7 steps S0 to S10), `EmuSen_RustPlatform.md` (how each platform component moved, with C# as
failover: §3.9 the switch, §5 the parity rule, §6 the tests, §26 DianaSh in the workspace), `EmuSen_CoreAPI.md` (§3 the
layers, §6 the core ABI v1, §7 discovery and trust, §8 the runtime, §9 the frontend API as first sketched, §10 its gap
list, §12 and §28 the conformance kit), `EmuSen_DianaSh.md` (§9 the session service, §10 the host API, §13 the shell's
migration, §16 Q1 and Q25), `EmuSen_LunaP_Kit.md` (§7 and §8: the frontend's move and its prerequisites P1 to P5),
`EmuSen_GamePackages.md` (a proposal touching the per-game stores) and `Diamond_Shell.md` §1.5 and §11.2 (a launch
that is not a file).

---

## 0. The result in brief

**The line (§2).** The backend is everything that decides what the emulated machine is, does and keeps, and every
fact that two frontends must agree on: core discovery and loading, the systems table and its packs, sessions and their
one frame-end order, input routing to a port and a core bit, speed, rewind, states and their records, cheats, save
media, firmware, the per-console and per-game core settings, the debugger and telemetry, and the player's library of
games with its stores. The frontend is everything that needs a window, a display clock, an audio or input device or a
toolkit, and every preference about presentation: windows and themes, shaders and the presenter, pacing against a
display, the audio device and its rate control, physical bindings, controller-first navigation. Scraping falls on the
backend side as a component of its own, last in the order (§2.4, D13).

**The frontend API (§3).** A stable C ABI, `emusen_crystal.h`, owned by Crystal and versioned as the core ABI is
(major 1, additive minors, a baseline in CI). It offers the runtime (discovery, systems, engines, routing, approvals),
the session's life (open a game or a console, insert, eject, reset, close), **one frame call** that performs the whole
frame-end order and hands over the picture and the sound, input, speed, rewind, states, settings, save media, firmware,
events, and the debugger and telemetry. The session service `emusen.session@1` that DianaSh's modules call becomes
what DianaSh §9.1 already says it is, the session part of this API: every one of its 79 functions has a twin of the same
arguments in `emusen_crystal.h`, and Crystal builds the table from the twins, so no frontend implements it any more. The
library is a second header, `emusen_library.h`, of its own component, because it runs with no session and a session
runs with no library. C# frontends reach both through one binding assembly; the Rust frontend through a binding crate
over the same C ABI. `EmuSen_DianaSh.md` §9.9's frontend service is built on rather than doubled: it keeps the two
window operations, and its other five functions become Crystal's (§3.11). Eight candidate defects read at today's seam,
none yet demonstrated, would be removed by the design, each to be shown by a failing test first (§3.12).

**The inventory (§4).** The C# backend is about 27,000 lines [M]: 11,878 in `EmuSen` (of which 3,946 are the per-core shims
of MarsRT, MercuryRT, MoonRT and VenusRT), 10,797 in `EmuSen.Crystal` (of which 3,117 are the C# provider of the session
service), and about 4,300 of Mistress's library and scraping code with no Avalonia in it. Twenty-eight facts have two or
more owners today (§4.3), among them which console a file is, an engine's name, which engine a game runs on (Mistress
reads a per-game choice that emusen-shell ignores), where a state is kept (Mistress honours the player's state folder
and emusen-shell does not; both read from the code and not yet run), and the frame-end order itself, three times.

**The system packs (§5).** A pack is a data file per system (names, years, extensions, cheat folders, OpenVGDB and
ES-DE names, cover shape, attachments, save media words) and Rust code keyed by the system's id (recognisers, header
readers, cheat codecs, save-media format modules, debugger views, the OpenVGDB byte transform), in one crate,
`emusen-systems`. Core info stays the owner of what an engine does; the pack is the owner of what the console is; every
field that both carry today is given one owner (§5.3).

**The order (§6).** Eleven steps, B0 to B10, each with a gate, a failover and a proof. **The smallest first step that
gives value is B1: the systems table as data, read by a Rust crate**, with the Game Boy and the Nintendo 64 given packs
of their own, about 1,350 lines (*built 2026-10-17 in about 690 of Rust, data and C# facade, §12*); `CoreCatalog`'s hand-written rows and the second record type for a console go. The
step of greatest value is B2: the frontend API, its binding and a C# implementation of it consolidated from today's
three loops, with Pharaoh, emusen-shell and Mistress moved onto it, so that the frontends move once and the Rust Crystal
of B7 is switched in behind the binding as each platform component was.

**Sizes (§7).** About 52,000 lines of Rust with their tests and C layers (41,000 to 62,000), by the platform steps'
measured ratio of Rust to the C# it replaced, against about 17,800 lines of C# ported and about 5,480 deleted
unported. B7, Crystal itself, is the largest at 18,000 to 27,000, and is switched program by program: Pharaoh first.

**Questions (§11).** Twelve standing rules or policies, each its own question, then fourteen design choices.

---

## 1. Where things stand (measured 2026-10-17)

### 1.1 What is already in Rust

| Part | Where | Surface | State |
|---|---|---|---|
| Every core: MarsRT, MercuryRT, MoonRT, VenusRT, Nephrite, Diamond | `EmuSen/Cores/…` | the core ABI v1, now 1.11 (`EmuSen_CoreAPI.md` §6.31) [R] | no C# core remains (`EmuSen_Stack.md` §2.1, revised 2026-10-16) [R] |
| Galaxia, Endymion, Serenity's Vulkan and shader half | `Platform/` | `emusen_platform.h`, interface 12, exact match [M]; *13 since B1 (§12)* | state 2: Rust by default, C# one variable away (`EmuSen_RustPlatform.md` §3.9) [R] |
| DianaSh's shell core, its text module, EmuSen's four command modules, `nano` | `Platform/dianash*`, `Platform/emusen-*` | `dianash.h` and `dianash_module.h`, stable | state 2 since 2026-10-16 (`EmuSen_DianaSh.md` §13.10) [R] |
| The session service from a caller's side | `Platform/session-service` (1,312 lines of Rust [M]) | `emusen_session.h` 1.2, 79 functions [M] | **implemented by the C# host**: `EmuSen.Crystal/Session/`, 3,117 lines of C# [M] |
| The conformance kit, a host of the core ABI | `EmuSen/Cores/Shared/emusen-core-conform` (4,017 lines [M]) | its own loader, `api.rs` and `sidecar.rs` | a second host of the core ABI beside the C# one (§4.3, F15) |

### 1.2 What is still C#

`EmuSen` (11,878 lines [M]) holds the core host (`Cores/Native/`, 4,487), the session and the factory
(`EmulatorSession`, `CoreFactory`, `CoreCatalog` and their small types, 1,536 at `Cores/`), the services (`RewindBuffer`,
`SpeedController`, `FirmwareLibrary`, `FrameScheduler` and others, 1,808 at `Common/`) and the per-core shims (3,946).
`EmuSen.Crystal` (10,797 [M]) holds the debugger's registries and expressions (2,998), the cheat engine (1,268), the
system packs (2,539), the session service's C# provider (3,117), save media (252), telemetry (327), the disc probe (127)
and the frame recorder (169). Mistress keeps its library and scraping: 4,665 lines in `Library/` and `Scraping/` that
never name Avalonia, of which about 4,300 are backend by §2's line [M: `grep -L Avalonia`].

### 1.3 The three loops, again

`EmuSen_Surfaces.md` §4.1 counted three frontends driving cores three ways on 2026-10-10. On `165e60b4` it still holds
[M]: Mistress runs `EmulationLoop` (`MainWindow.axaml.cs:1288`) over an `EmulatorSession`; Hotaru's `GameWindow`
(now a library under emusen-shell, `EmuSen_Frontend_Driver.md` §0) runs its own over an `ICore` from `CoreFactory`
(`GameWindow.axaml.cs:408–440`); Pharaoh runs `FrameRunner` (`FrameRunner.cs:69–104`). Rewind capture asks the engine's
opt-out in Mistress alone (`MainWindow.axaml.cs:1399`; `RewindCapture` is named in no other frontend). Two more
divergences have appeared since the audit, both from code that decides a backend fact in one frontend (§4.3, F3 and F4).
And `EmuSen_Surfaces.md` §7's S1 and S3, decided 2026-10-11, are not built: `EmuSen.csproj:30` still references
Serenity, `EmulatorSession` now names Serenity's `SlangLend` and `RawPicture`, and `GraphicsSettings.` appears on 21 lines
of shipped code [M].

---

## 2. The line

### 2.1 The rule

**A thing is backend if two frontends must give the same answer about it, or a headless client needs it. A thing is
frontend if it needs a window, a display clock, an audio or input device, or a toolkit, or if it is one program's
preference about how something is presented.** [C, SR1]

The rule is a sharpening of `EmuSen_CoreAPI.md` §3's table ("presentation: windows, shaders, the audio device, physical
bindings, pacing against a display, the library and its scraping and themes" for the clients), and it moves one thing
across that table: the library's data (§2.4). The reason is the coexistence the kit plan requires: the C# Mistress keeps
working until the Rust frontend has passed every recorded state and been played (`EmuSen_LunaP_Kit.md` §7.5, K9) [R].
For that whole period two frontends in two languages run against the same player's files. Whatever both read and
write must have one owner below both, or it acquires two writers, two migration lists and two readings of one schema.

### 2.2 Backend: moves to Rust

| Area | Today | Why it is backend |
|---|---|---|
| Core discovery, sidecars, trust, loading, the v1 adapter | `EmuSen/Cores/Native/` | every client opens cores; the kit already needs a host of its own (F15) |
| The systems table and the system packs | `CoreCatalog`, `Crystal/Systems/`, console code in the shims | console facts are agreed by every frontend, the shell and the library scan |
| Sessions: open a game or an empty console, discs, reset, close; the frame-end order | `EmulatorSession`, `CoreFactory`, `CoreEngine`, three loops | the divergences of §1.3 are this |
| Input routing: canonical control to port to core bit, the input latch, which controller a port holds | `InputLatch`, `ControllerPorts`, the packs' `PlayerControllers` | a port's controller is a console setting, not a device |
| Speed (fast-forward, slow motion, frames not drawn) | `SpeedController` | decides what the machine renders; `EmuSen_CoreAPI.md` §8.2 [R] |
| Rewind and its snapshots | `RewindBuffer`, `XorDeltaCodec`, `RewindThumbnail` | capture at the frame boundary, gated by the engine |
| States: slots, resume, the state's record | `SaveLibrary` names (Galaxia), Mistress's slots and `records.db` | F4, F5 |
| Cheats: the registry, application, the codecs, the `.cht` database | `Crystal/Cheats/`, the packs | one registry per session (`EmuSen_CoreAPI.md` §8.2) [R] |
| Save media: placement, modes, locks, the card manager | `MemoryCards`, `SaveMediaManager`, `MediumLock` | the shell's `cards` and Mistress's window must agree |
| Firmware: requirements, the store, the overview | `FirmwareLibrary`, `FirmwareOverview` | asked before a load by every frontend |
| Core settings per console, per engine and per game; the engine chosen | `graphics.json`'s `Consoles`, `games.db`'s `altemulator`, `CoreFactory.StoredSettings` | F3, F11 |
| The debugger, expressions, telemetry, the frame recorder | `Crystal/Debugger/`, `Telemetry/`, `Recording/` | already Crystal's by decision (`EmuSen_DianaSh.md` §9.5) [R] |
| Logs and crash records of a session | `CoreLibrary`'s crash file, `CategorizedLogWriter` | the runtime records core faults (`EmuSen_CoreAPI.md` §10.1) [R] |
| The library: scan, identity and hashes, `games.db`, play time, collections, metadata edits | Mistress's `Library/` | §2.4 |
| Scraping and the media store, `media.db` | Mistress's `Scraping/` | §2.4 |

### 2.3 Frontend: stays C# until the frontend's move

| Area | Today | Why it is frontend |
|---|---|---|
| Windows, menus, sheets, Big Picture, ES-DE themes and `themes.db` | Mistress, LunaP | a toolkit |
| The presenter, shaders, filters, the device picture's chain, the display clock | Serenity (its Vulkan half already Rust) | a display |
| Pacing against the display: `FrameScheduler`, the hand-off hold | `EmuSen.Serenity/FrameScheduler.cs` (moved by B0), `FrameHandOff.cs` | a display clock (`EmuSen_CoreAPI.md` §8.4) [R] |
| The audio device, resampler and rate control; UI sounds | Endymion (already Rust) | an audio device |
| Physical bindings: keys, pads, hotkeys; device to player | `keybindings.json`, `hotkeybindings.json`, `gamepadbindings.json`, `GamepadManager`, `PortRouter` | an input device |
| Window size, filtering, shader parameters and recent shaders | `graphics.json` minus `Consoles` | presentation preference |
| `appsettings.json`'s interface keys (views, scales, status bar, scraping choices, Big Picture) | `AppSettings` | presentation preference; the runtime only reads the folders (§3.10) |
| A game's crop | `games.db`'s `crop_*` | presentation; stored by the library, meant by the frontend |
| The shader pack's build record | `records.db` | presentation; §4.4 splits the file |
| Controller diagrams' drawing, pad navigation, the on-screen keyboard | Mistress `Input/` | a toolkit |

`EmuSen_RustPlatform.md` §7.3 had already drawn the line inside Endymion the same way: the input vocabulary and the
port router's rules are the runtime's, the devices, the resampler and the rate control are a client's [R]. They are
already Rust, so they cross no line here; they are frontend-facing parts of the platform library.

### 2.4 The library and scraping, argued

`EmuSen_CoreAPI.md` §3 put the library with the clients, and the kit plan counts it among the portable code the Rust
frontend ports (`EmuSen_LunaP_Kit.md` §7.2) [R]. This page moves its **data** to the backend, for three reasons:

1. **Coexistence** (§2.1). Mistress and the Rust frontend would otherwise both open `games.db`, which has an append-only
   migration list and refuses a newer file (`EmuSen_Stack.md` §2.3) [R]; two migration lists in two languages for one
   schema is the defect the rule exists to prevent.
2. **The shell needs it.** `core` resolves a path through the library (`man core`), and game packages require that a
   file a package claims opens as that package "from every route, the shell's included" (`EmuSen_GamePackages.md` §3.5,
   Q14) [R]. A claim index known to one frontend is not a rule.
3. **It is already free of the toolkit.** Of the 4,665 lines of `Library/` and `Scraping/`, all but two files never
   name Avalonia [M], so the move is a port, not a separation.

It is **a component of its own**, not part of Crystal: it runs with no session (the library screen), and a session runs
with no library (Pharaoh, the kit). Giving Crystal the library would give it a second job (§11, D3).

**Scraping** passes the same test (both frontends' Big Picture needs it, and it writes `media.db`, which both read), and
it is the one part with network I/O, credentials (`screenscraper-developer.json`, `EmuSen_BigPicture.md` §5.7) and a
quota. It is a component of its own, `emusen-scrape`, and comes last in the order, so that the backend's move is not
held for it; its other answer, leaving it to the frontend's move, is coherent too and is put to the tester as D13.

### 2.5 Two files with two jobs

The line runs through two configuration files, and one owner per fact requires that it run through no file:

- **`graphics.json`** holds the window's size and filtering and the shaders' parameters (frontend), and `Consoles`, each
  console's core settings by key, the engine chosen and the save-media modes (backend)
  (`EmuSen.Galaxia/Models/GraphicsConfig.cs:9–21`) [M]. Today the runtime only reads it (`CoreFactory.StoredSettings`),
  and Mistress holds one loaded copy and saves it whole. Once Crystal writes a setting (`settings_set`, §3.6), a frontend
  saving its stale copy would undo it. **Recommended: `Consoles` moves to a file of the runtime's own,
  `/etc/EmuSen/consoles.json`, by one migration that copies and never overwrites (`EmuSen_Galaxia.md` §3.2), and
  `graphics.json` keeps the rest** [C, SR5, SR12].
- **`appsettings.json`** holds the ROM folder, the state folder, the firmware and cheat-database folders (backend
  reads) and fifty-odd interface preferences (`AppSettings.cs`) [M]. Here no split is needed: the runtime reads its four
  folders through Galaxia on each use and never writes the file, so the file keeps one writer [C, SR5].

---

## 3. The frontend API

### 3.1 Its shape: a C ABI of its own, not an extension of the session service

Three shapes were weighed:

| Shape | For | Against |
|---|---|---|
| (a) Extend `emusen.session@1` with the lifecycle, frames, sound and input | one header | the table is handed by a shell to a module during one call, on the shell's thread (`EmuSen_DianaSh.md` §9.1, §6.4) [R]; a frame call, a picture and a sound ring are not a command's, and a module that could open and run sessions would be a frontend inside the shell |
| (b) A table only, fetched by one export | uniform with the service | a frontend is a program that loads a library; exported functions are what the core ABI and `dianash.h` already give their hosts |
| **(c) Exported functions, `emusen_crystal_*`, with the session service built from them** | the core ABI's and the host API's shape one layer up; one implementation; the service unchanged for modules | two names per session function, which the parity test of §3.9 holds together |

**Recommended: (c)** [C, D1]. The header is `emusen_crystal.h`, owned by Crystal beside `emusen_session.h`; the exports
are `emusen_crystal_*`; types are `emusen_crystal_<name>`; constants `EMUSEN_CRYSTAL_*`. It is exported from the
platform library, `emusen_platform`, as `dianash.h` already is (`EmuSen_RustPlatform.md` §26), so that one copy of
Galaxia's tree and overrides serves the runtime, the shell and the platform in one process (`EmuSen_RustPlatform.md`
§2.2) [R].

### 3.2 Versioning and ownership

As the core ABI (`EmuSen_CoreAPI.md` §4) and the host API (`EmuSen_DianaSh.md` §11) [R]: `emusen_crystal_abi_version()`
returns `(MAJOR << 16) | MINOR`; the major is 1 and is expected never to change; a minor adds functions, appends fields
to size-prefixed structs, adds values to open enumerations and keys to JSON; nothing is removed, a deprecated call keeps
working. A baseline, `Platform/crystal/abi/v1/baseline.txt`, is checked in CI by the core ABI's guard job, the header is
compiled as C99 and C++17, and skew fixtures (a host at minor 0, a far minor with larger structs and canaries) are built
as `EmuSen_CoreAPI.md` §12.5 has for cores [A]. **Stable from its first commit** [C, SR3], for the reason
`EmuSen_DianaSh.md` §16 Q17 gave for the host API: the baseline costs one CI step, and an outside frontend can then use
the API the day it wants to.

Statuses use the platform's band (`EmuSen_RustPlatform.md` §3.5). The session service's codes keep their numbers and
names (−1100 to −1108), so a twin passes its status through unchanged; the frontend API's own begin at −1109
(`EMUSEN_CRYSTAL_NO_ENGINE` −1109, a file no engine opens, with the routing decision's words; `EMUSEN_CRYSTAL_FIRMWARE`
−1110; `EMUSEN_CRYSTAL_BUSY` −1111, a call from the frame thread that would wait for itself), and the library's at −1140
[C, D12].

### 3.3 The rules every call follows

The core ABI's, one layer up (`EmuSen_CoreAPI.md` §2.3, §6.16, §6.17) [R]: pull, never push (no callback into the
client; notifications are drained); handles for objects with a life (the runtime, a subscription) and 32-bit ids for
sessions, as the service already uses; copy out by the length-query idiom; signed returns; UTF-8 as pointer and length;
JSON for lists and descriptors, read once and never on the per-frame path (§2.5 there); size-prefixed structs for the
per-frame calls; no unwinding (the library aborts with the crash log, as the platform library does).

**Threads.** A session has one *frame thread*, the thread that calls `frame` (§3.4). Reads are answered on any thread
from the session's latest snapshot, never waiting. A change called from another thread is queued and applied at the
next frame boundary, the call waiting for it, as the session service's changes already are (`emusen_session.h`,
"Reads and changes"); a change called **on the frame thread between frames applies at once**, which is how today's
`RequestOnEmulationThread` works (`MainWindow.axaml.cs:728`) and which removes the deadlock a waiting call on that thread
would cause. A paused client parks its frame thread in `emusen_crystal_wait(rt, s, timeout_ms)`, which returns when a
change is queued or run control asks to run: a blocking wait with a timeout, which `EmuSen_CoreAPI.md` §9.2 already
allows because it is not a callback [R]. `open` may be called on any thread, as Mistress's early load under the
launch screen already loads off the UI thread (`MainWindow.Resume.cs:57–65`) [M]; the frame thread is whichever thread
first calls `frame`, and `close` from another thread waits for the frame in progress.

### 3.4 The frame call, and where pacing stays

`EmuSen_CoreAPI.md` §8.4 decided on 2026-10-01 that "the runtime owns the loop and the emulation thread" and that pacing
is "a strategy the client supplies", through "a pacer seam, a call it makes before each frame". In a C ABI that seam is a
callback into the client, which the pull rule forbids. **Recommended instead: the client owns the thread and the pacer;
the runtime owns everything between two pacer decisions, in one call** [C, SR6]:

```c
typedef struct emusen_crystal_frame {
    uint32_t size;
    uint32_t ran;              /* 0 while paused or halted: nothing advanced */
    uint32_t halted;           /* a breakpoint or a step stopped the frame */
    uint32_t rendered;         /* the picture is new; 0 on a frame speed skipped */
    uint64_t picture_serial;   /* the core's frame serial; unchanged means the same picture (§6.6 of the core ABI) */
    int32_t  width, height, row_repeat, display_lines;
    double   display_aspect;
    uint32_t picture_on_device;/* the picture lies on the lent device (§6.29 of the core ABI) */
    uint32_t audio_frames;     /* ready to drain */
    uint32_t audio_rate;       /* the samples' rate, which travels with them (§6.7 of the core ABI) */
    uint32_t play_audio;       /* 0 when speed says the sound is not played */
    uint32_t speed_percent;    /* what the pacer reads; 0 unthrottled */
    uint64_t total_frames;
    double   frame_rate_hz;
} emusen_crystal_frame;

int32_t emusen_crystal_frame(emusen_crystal_runtime *rt, uint32_t session, emusen_crystal_frame *out);
int64_t emusen_crystal_picture_copy(emusen_crystal_runtime *rt, uint32_t session, uint8_t *out, size_t len);
int64_t emusen_crystal_audio_drain(emusen_crystal_runtime *rt, uint32_t session, int16_t *out, size_t frames);
```

`frame` applies the queued changes and the input latch, advances the machine unless paused or halted, then runs the
frame-end order once, in the order `EmuSen_CoreAPI.md` §8.2 gives it [R]: the frame log, freezes and cheat pokes, the
battery period, rewind capture where the engine allows it, the debugger's snapshots (`RefreshProviders`), events. A
held rewind is `emusen_crystal_rewind_step`, called by the client in place of `frame`. `picture_copy` passes straight
through to the core's `frame_copy` into the client's buffer, one copy as today; the raw picture (§6.30 of the core ABI)
and the device picture pass through the same way. What stays in the client is what reads the display: when to call
`frame` (`FrameScheduler`), the hand-off hold, the audio device and its ratio.

The reasons: the pull rule holds with no exception; the steps the three loops disagree on are all inside the call, so
the divergence is closed by construction; and Mistress's loop already separates the two, since everything it does
between `scheduler.Next` and `scheduler.Completed` that is not the presenter is the frame-end order
(`MainWindow.axaml.cs:1343–1457`) [M]. A runtime-owned thread fed with the display's timing as data, which would keep
the 2026-10-01 wording, is left open as a later minor if §9's P2 fails.

### 3.5 What it offers

The groups, with the function names a header would carry; each JSON shape is written in the header's comment, as
`emusen_session.h` writes its own.

| Group | Functions | Replaces |
|---|---|---|
| **Library-level** | `abi_version`, `version_text`, `last_error`, `new(config)`, `free` | `CoreFactory`'s statics |
| **Discovery and systems** | `systems` (the table, each with its engines and its pack's facts), `engines`, `cores` (each library's trust, sidecar and report), `core_approve(hash)`, `rescan`, `route(path)` (the system, the engine and the decision's words) | `CoreCatalog`'s 46 members, `CoreDiscovery`, `SystemRouting` |
| **Session life** | `open(params)` (a path, or a system with `starts_empty`; an engine or the layered choice; the device lend, the core ABI's own `emusen_device_lend`; flags such as no battery), `close`, `reset`, `info` (engine, system, console, game identity, ports and each port's controller, capabilities as flags, drives and discs, run state, the engine's, firmware's and media's notices) | `EmulatorSession`, `CoreFactory.Load`, `LoadConsole`, `CoreBundle`; the launch record of `Diamond_Shell.md` §1.5 is `open`'s parameters |
| **Media** | `media_insert(path)`, `media_eject`, `media_change(index)`, `media_lid(open)` | `CoreEngine.InsertDisc`, `EjectDisc`, `ChangeDisc`, `SetLid` |
| **The frame** | `frame`, `picture_copy`, `picture_raw`, `audio_drain`, `wait` (§3.3, §3.4); `frame_phases`, the core's phase timings for a status line | the three loops; `IFrameProfiler` |
| **Run control** | the service's `run_control`, `halt_state` | `IsHaltedAtBreakpoint`, `HostAction.Resume`, `Step` |
| **Input** | `input_set(port, control, value)` with the core ABI's canonical control numbers, `input_connected(port, on)` | `SetButton`, `SetAxis`, `SetControllerConnected` |
| **Speed** | `speed_set(percent, turbo_percent, slow_percent)` | `SpeedController` held by each frontend |
| **Rewind** | `rewind_set(enabled, interval, budget)`, `rewind_step(count)`, `rewind_reel` (moments with frame numbers), `rewind_picture(index)`, `rewind_to(index)` | `RewindBuffer` held by each frontend |
| **States** | `state_slots` (slot, path, exists, time, picture, record), `state_save(slot or path)`, `state_load`, `resume_save`, `resume_load`, `state_record(path)`; the refusal of a state from another game or console | Mistress's slots, `MainWindow.Resume.cs`, `MainWindow.Identity.cs`, `FileRecords`; Hotaru's single path |
| **Settings** | `settings_schema(system, engine)`, `settings_get(scope, key)`, `settings_set(scope, key, value)` with `scope` = console, engine or game; applied to a running session between frames | `CoreCatalog.SettingsFor`, `ICoreSettings`, `ApplyConsoleSettings`, `CoreFactory.StoredSettings`, `EngineChosen` |
| **Cheats** | the service's `cheat_op` and `cheat_codecs` | `CheatRegistry` handed in by each frontend |
| **Save media** | `media_slots` (a session's), `cards_op(op, json)` (list, open, copy, move, delete, format; runtime-level, no session needed) | `MemoryCards`, `SaveMediaManager` |
| **Firmware** | `firmware_needed(path or system, engine)`, `firmware_find`, `firmware_install(request, bytes)`, `firmware_overview` | `EmulatorSession.MissingFirmwareFor`, `FirmwareLibrary`, `FirmwareOverview` |
| **Debugger and telemetry** | every function of `emusen_session.h` 1.2 as a twin (§3.9), and `session_service(flags)`, which returns Crystal's own `emusen_session_v1` table, read-only or not, for a host to hand to `dianash_set_services` | `IDebugTarget` through `session.DebugTarget`, the C# provider |
| **Events** | `subscribe(session or 0, kinds)`, `events_drain(sub, out, len)`, `events_wait(sub, timeout)`, `unsubscribe` | `HostAction`; the shell's scheduler |
| **Logs** | events of kind `log` carry a core's drained lines; the crash record is written by the runtime | `CategorizedLogWriter` per frontend, `CoreLibrary`'s crash file |

**Events** carry, as JSON lines with a kind and a session: a session opened or closed (so a game opened from the shell
inside Mistress reaches Mistress's window), run, paused, halted and stepped, state saved and loaded, cheat changed,
battery written, media changed, a notice, settings changed, rewind moved, and a core's log line. Each subscriber has its
own bounded queue, as `EmuSen_CoreAPI.md` §8.3 decided [R]; a full queue drops its oldest line and says so in the next
drain.

**Not offered**, and why: anything that draws, plays or reads a device (§2.3); `ICore`, a core's handle or the engine
SPI (`EmuSen_CoreAPI.md` §9.4) [R]; the library and scraping, which have their own header (§3.7).

### 3.6 Settings, layered

`settings_get(scope, key)` answers game over engine over console over the schema's default, as `EmuSen_CoreAPI.md` §8.2
decided [R]. The engine chosen is one such key. Today Mistress layers a game's engine from `games.db`'s `altemulator`
over `graphics.json` (`MainWindow.axaml.cs:377–381`, `MainWindow.GameOptions.cs:36`) and emusen-shell reads
`graphics.json` alone (`CoreFactory.ConfiguredEngine`, `HotaruSession.cs`) [M]; after B2 both ask `settings_get`. Where
a game's own values live is a storage policy with three candidates on record, put to the tester as SR10: the file per
game that `EmuSen_CoreAPI.md` §8.5 adopted, `/etc/EmuSen/games/<game id>.json`; the `games.db` field ES-DE's metadata
names; or a package's `settings.json` (`EmuSen_GamePackages.md` Q12, undecided) [R].

### 3.7 The library's header

`emusen_library.h`, exports `emusen_library_*`, stable on the same policy, owned by the library component:
`scan(roots)` with progress drained, `games(filter)`, `game(id)`, `identity(path)` (the system as routed and the dump's
hashes, from Galaxia's `RomHash` extended as `EmuSen_GamePackages.md` §2.2 proposes), the player's records (favourite,
collections, play time started and ended, metadata edits, a crop stored for the frontend), recent games, and, when game
packages are decided, the claim index. The scraper's header, `emusen_scrape.h`, is planned with B9: a job started,
progress drained, cancelled, and the media of a game by ES-DE's type names. The library calls `emusen-systems` for
routing and Galaxia for identity, both crates below it, and never Crystal.

### 3.8 How DianaSh's session service relates to it

**The session service is the frontend API's session subset, presented as a table** [C, SR4]. This is
`EmuSen_DianaSh.md` §9.1 made literal: there "the session part of Crystal's frontend API … written once as C and offered
to two kinds of client" [R]. Concretely:

- each of the 79 functions of `emusen_session_v1` has an exported twin `emusen_crystal_<name>(rt, session, …)` with the
  same arguments after the context, the same statuses and the same JSON;
- Crystal fills the table from the twins (`ctx` is the runtime), and `emusen_crystal_session_service` hands it out;
- a test reads both headers and fails when a function is in one and not the other, or their arguments differ, the
  parity test of `EmuSen_CoreAPI.md` §8.6 made mechanical;
- the service keeps its own header, version and baseline, because modules depend on it at 1.2, and gains functions only
  as the frontend API gains their twins.

What changes for the hosts: a frontend no longer **provides** the service, it **passes on** Crystal's. The C# provider
(`EmuSen.Crystal/Session/`, 3,117 lines [M]) retires at B7. Two of DianaSh's open items fall out of this: `HostAction`
is retired by events and `open` (`EmuSen_DianaSh.md` Q5, decided [R]), and Q25's `games` and `open_game`, proposed for
the frontend service `emusen.frontend@1`, belong in the session service at 1.3 instead, since Crystal opens sessions
itself and §9.9 there says those two calls retire "when Crystal opens sessions itself" [R]. The frontend service keeps
only what a frontend alone can do: opening windows and activating the game window [C, D10].

### 3.9 The bindings

- **C#: one assembly, `EmuSen.Crystal.Client`,** with two layers as `EmuSen_CoreAPI.md` §9.2 drew them [R]: a contract
  layer of C-shaped static functions, and a binding of disposable classes, exceptions built from statuses, a
  `ChannelReader` over a subscription, and records parsed from the JSON. Underneath, the contract layer dispatches to one
  of two implementations by `EMUSEN_CRYSTAL_NATIVE` through Galaxia's `PlatformSwitch`: the C# one of B2 (§6) or the
  library's exports. Frontends reference this assembly and not `EmuSen` or `EmuSen.Crystal`; a reference test pins it
  [C, SR2].
- **Rust: a binding crate, `emusen-crystal-client`, over the same C ABI,** not the crates' Rust API [C, D2]. The kit plan
  requires that the Rust frontend be "its first client, not a fourth copy of the loop" (`EmuSen_LunaP_Kit.md` §8, P1)
  [R]; holding it to the C ABI is what makes it the same client as Mistress, with the same surface, statuses and threads.
  Linked statically, an `extern "C"` call costs a call; the crate gives safe types, `Result`, iterators over drained
  events, and a picture copied into a buffer the frontend owns. The Rust API of the crates remains rule R of
  `EmuSen_Surfaces.md` §6.1, internal to the runtime's own crates.

### 3.10 How each frontend moves onto it

| Client | Today | On the API | Removed from it |
|---|---|---|---|
| **Pharaoh**, first | `CoreFactory.Create`, `FrameRunner`, a `RewindBuffer` of its own, `--commands` verbs over `ICore` and `IDebugTarget` | `open` with no battery, `frame` in its runner, the service table for its debug verbs, its unknown verbs submitted to the shell as now | `FrameRunner`'s frame-end work, its `ICore` and `IDebugTarget` holdings |
| **emusen-shell** (Hotaru's game window) | `HotaruSession.Build` over `CoreFactory`; `GameWindow`'s loop; `HostAction` synthesised by the shell facade | `open` from the launch prompt and `core`; `frame` in the window's loop beside its pacer; events for resume, step and a swap; Crystal's service table handed to the shell | its rewind, speed, state path and `HostAction` handling |
| **Mistress** | `EmulatorSession`, `CoreFactory`, `CoreCatalog` (26 of its members named, from 34 of Mistress's files [M]), the loop, slots, resume, records, rewind | `open`, `frame`, `rewind_*`, `state_*`, `settings_*`, `systems`, `firmware_*`, `cards_op`; its window code unchanged | `EmulationLoop`'s frame-end order, `RunCoreRequests`, the state records, `ApplyConsoleSettings`, the engine's layering |
| **The Rust frontend** | — | the binding crate from its first screen (K6, Big Picture) | nothing to remove: it is built on the API (`EmuSen_LunaP_Kit.md` §7.1) |

Today Mistress names 118 distinct public types of the backend's assemblies, Hotaru 41 and emusen-shell 17 (a word match
of every type the six assemblies declare over each frontend's sources, comments excluded) [M]; after its move a frontend
names the binding's types and the presentation and device layers' alone.

The order, Pharaoh then emusen-shell then Mistress, is `EmuSen_CoreAPI.md` §10.2's D2 [R]: the headless client first,
because its frame and audio digests (`framesum`, `audiosum`) are an exact oracle, then the smaller window, then the
largest.

### 3.11 The frontend service, `emusen.frontend@1`: built on, not doubled

`EmuSen_DianaSh.md` §9.9 designed a second, small table that a frontend provides to the shell's modules: `open_window`,
`activate_game_window`, `games` and `open_game`, and `pause`, `resume` and `is_paused` [R]. It is not built: its one
mention in code is a refusal's words, "pausing is the frontend's (emusen.frontend@1)" (`SessionService.Machine.cs:82`)
[M]. This design keeps it and narrows it, so that each operation has one surface:

| §9.9's function | Where it goes | Why |
|---|---|---|
| `open_window(kind)`, `activate_game_window` | **kept in `emusen.frontend@1`**, provided by the frontend | only a frontend has windows; Crystal never does (§2.3) |
| `games`, `open_game` | the session service at 1.3, served by Crystal, as twins of `systems` and `open` (§3.8) | §9.9 says they retire "when Crystal opens sessions itself"; from B2 it does |
| `pause`, `resume`, `is_paused` | the session service's existing `run_control` (`RUN_PAUSE`, `RUN_RESUME`) and `info`'s run state, served by Crystal | §9.9 says they retire "into run control when Crystal owns the loop"; with the frame call Crystal owns the frame-end order and the pause state, and the client's thread parks in `wait` (§3.3, §3.4) |

So the frontend service is built at B2 with two functions and nothing in it duplicates Crystal [C, D10]; the C#
provider's refusal of `RUN_PAUSE` ends at B2, when the managed implementation serves it.

### 3.12 Defects the design would remove: candidates to prove

Reading what the three frontends call at the seam (the same measurement as §3.10's counts) turned up eight places where
today's arrangement looks wrong. **None has been demonstrated**: each was read from the code and is a candidate. The step
that removes it first shows it with a test that fails, as the project's evidence rule requires, and only then makes the
change; where the test shows no defect after all, it records that instead.

| # | Candidate, read from the code | What removes it, and when |
|---|---|---|
| C1 | Mistress's console `pause` and `resume` are not read-only, so they are queued and drained on the emulation thread after `RunFrame` (`EmulationControlCommands.cs:24,47`; `DianaShConsoleWindow.cs:144–154`); `PauseEmulation` then sets status text off the UI thread, and while paused the parked loop drains requests but not the console, so a typed `resume` may wait for something else to resume the game (`MainWindow.axaml.cs:1247–1253, 1322–1329`) | `run_control` served by Crystal; a change from another thread is queued, and `wait` wakes the parked frame thread for it (§3.3, §3.11). B2 |
| C2 | Mistress never reads `IsHaltedAtBreakpoint` (`grep`: 0 in its sources [M]), so a breakpoint set from its console may not stop the game: the loop hands over the frame and the next tick resumes it | `frame` returns `halted`, and while halted it advances nothing until run control says so (§3.4). B2 |
| C3 | Hotaru's feed window polls `GetFrameBufferRgba` from the UI thread every 33 ms while the emulation thread calls it too; the method writes the engine's shared scratch frame and shape outside the machine's lock (`CoreEngine.cs:549–561` [M]; `GameWindow.axaml.cs:635`) | the picture is copied into the caller's buffer from the session's latest frame, a read answered from a snapshot (§3.3); no shared scratch is exposed. B2 for the managed implementation, B7 for Crystal |
| C4 | Hotaru never returns a lent picture (no `ReturnFrameBuffer` in its sources [M]), so the lending pool never refills and each frame allocates | `picture_copy` writes into a buffer the client owns; nothing is lent across the API but the device picture, whose lend is the core ABI's and is settled by the runtime. B2 |
| C5 | Mistress's *Apply Cheats* while paused calls `ApplyCheats()` on the UI thread as the pause signal is reset, possibly beside a frame still finishing (`MainWindow.axaml.cs:573–582`); memory-safe at the ABI's lock, but the poke may interleave with frame-end work | cheats are applied inside the frame-end order only; `cheat_op`'s changes land at the boundary (§3.4). B2 |
| C6 | Hotaru sets the pad manager's bindings from the emulation thread during a swap while the UI thread's timer polls the same manager (`GameWindow.axaml.cs:364–365`) | not the backend's, since physical input is the frontend's (§2.3); removed because a swap becomes `close` and `open` called from the UI thread, after which the window rebinds on its own thread. B2, emusen-shell's move |
| C7 | The post-load sequences still differ: Mistress's console `state load` clears rewind and does not reset rate control; Hotaru's does neither, and its rewind capture is ungated (`MainWindow.axaml.cs:623–628`; `HotaruSession.cs:58`; `GameWindow.axaml.cs:451`) | one `state_load`, which clears the rewind chain and emits a `state loaded` event on which a client resets its audio rate control; capture gated inside `frame`. B2 |
| C8 | Telemetry polled from the UI thread (`coretop` every 250 ms) may wait up to a frame for the machine's lock, which the frame holds throughout (`CoreMachine.cs:188–190`) | reads answered from the session's per-frame snapshot, never the machine (§3.3). B7; B2's managed implementation keeps today's behaviour, so the test is written at B2 and expected to pass at B7 |

**Process-wide statics used as wires** between the frontends and the backend, found by `grep` for their assignments
[M], and their end:

| Static | Today | End |
|---|---|---|
| `GraphicsSettings.SyncToDisplay` | written on the UI thread by the preferences window beside `GraphicsConfig.SyncToDisplay`, read each frame on the emulation thread (`PreferencesWindow.cs:276`; `MainWindow.axaml.cs:1342`; `GameWindow.axaml.cs:415`) | a parameter of the client's pacer, which stays frontend (B0, `EmuSen_Surfaces.md` S3). *Done 2026-10-17: `GameFrameControl.SyncToDisplay`, a volatile field the frontend sets and its pacer reads (Surfaces §7.1)* |
| `GraphicsSettings.Window*`, `AudioSettings.*` | loaded once, read by both frontends | the models, as parameters (B0). *Done 2026-10-17 (Surfaces §7.1)* |
| `Console.SetOut` | every core log line and the frame-rate line reach the per-game log through the process's `Console.Out` (Mistress `MainWindow.axaml.cs:1557,1571`; `HotaruSession.cs:46`; Shell `Program.cs:64`) | a session's log lines are `log` events the client files; the runtime writes no process-wide stream (B7; B2's managed implementation keeps one writer per session) |
| `DebugSettings.MasterLoggingEnabled` | forwards to Crystal's `DebugLogging.MasterEnabled` (`DebugSettingsWindow.cs:34`; `DebugSettings.cs:7–11`) | the service's `log_switch` and `log_status`, served by Crystal (B2) |
| `ConfigDiagnostics.Sink`, `ErrorLog.Redactor`, `LunaSettings.*` | each program's setup | stay: each is the program's own, set once at start, and crosses no backend surface |
| `PresentationTrace`, `GameFrameControl.RenderFps`, `DevicePictureAllowed`, `DeviceLendAllowed` | presentation statics | stay frontend (Serenity) |

Two further reaches past a surface were found [M]: windows that call a DianaSh command's statics
(`CheatCommand.FormatWrite` and `PrefersExplicitCodec` from `ActiveCheatsWindow.axaml.cs:445,493`; `CoreCommand.Refusal`
from emusen-shell's `Program.cs:236,244`), and library code that imports the shell's command namespace (`ArtworkIndex.cs:7`,
`CoverFetcher.cs:10`). A frontend's formatting of a cheat is its own, and the refusal of a path is `route`'s answer, so
both go with B2; the library's imports go with B8.

---

## 4. The inventory

### 4.1 Every C# backend service

Sizes are lines by `wc -l` at `165e60b4` [M]. Callers are the non-test projects whose sources name the type
(`grep -rlw`), with WiseMan's files in brackets [M]. "Target" is the Rust crate, or why it stays.

| Service | C# | Lines | Callers | Target |
|---|---|---:|---|---|
| The v1 host: library, machine, interface, descriptors, discovery, input latch, state layout | `Cores/Native/Core{Library,Machine,Interface,Descriptors,Discovery}`, `InputLatch`, `StateLayout` | 1,735 | EmuSen | `emusen-core-host`, shared with the kit (B6) |
| The debug bridge and generic debug target | `CoreDebugBridge`, `CoreDebugTarget` | 505 | EmuSen [WiseMan 4] | `emusen-crystal`, `debug` (B7) |
| The session over an engine | `CoreEngine` | 848 | EmuSen 9 files, Mistress 3, Hotaru 1 [45] | `emusen-crystal` (B7) |
| The pre-stable host | `Native{CoreLibrary,Interface,Machine,RtCore,DebugBridge}` | 1,096 | EmuSen and the shims | **deleted, not ported** (B4) |
| The ports' adapter | `PortEngine` | 303 | the MoonRT and MercuryRT shims | deleted with them (B4) |
| The per-core shims | `MarsRT/Shim` 2,068, `MercuryRT/Shim` 1,114, `MoonRT/Shim` 604, `VenusRT/Shim` 67, MarsRT's PGO driver 89 | 3,942 | `CoreFactory` | deleted (B4); their console code to the packs (B3); the PGO driver stays as tooling |
| The session's front and the factory | `EmulatorSession` 174, `CoreFactory` 241 | 415 | Mistress 5 and 4, Hotaru 2, Pharaoh 2 [30, 50] | the frontend API (B2, B7) |
| The systems table's god object | `CoreCatalog` | 447 | Mistress 34 files, Hotaru 2, Shell 2, EmuSen 9 [67] | split: facts to `emusen-systems` (B1, B3), engines to the host, settings rows to the schema (B7) |
| Small session types | `ControllerPorts`, `ConsoleSession`, `CoreOptions`, `CoreHooks`, `ICore`, `CoreCapabilities`, `IDevicePicture`, `ICheatRegistryHost`, `FrameBufferLending` | 385 | EmuSen; `ICore` also Mistress 3, Hotaru 2, Shell 1, Pharaoh 5 | the engine SPI disappears; ports and capabilities become `info`'s JSON (B7) |
| Rewind | `RewindBuffer` 302, `RewindThumbnail` 81, `XorDeltaCodec` 91 | 474 | Mistress, Hotaru, Pharaoh 1 each [11] | `emusen-crystal`, `rewind` (B7) |
| Speed | `SpeedController` | 81 | Mistress 2, Hotaru 1 [8] | `emusen-crystal` (B7) |
| Battery | `BatterySave` | 76 | EmuSen 8 [10] | `emusen-crystal`; names already Galaxia's (`saves.rs`) |
| Save-media placement | `MemoryCards` | 263 | EmuSen 3, Mistress 3 [10] | `emusen-crystal`, `media` (B7) |
| The card manager | `SaveMediaManager`, `SaveMedium` | 252 | Crystal, DianaSh, Mistress 2 [2] | `emusen-savemedia` (B5) |
| Firmware | `FirmwareLibrary` 125, `FirmwareRequest` 30, `FirmwareOverview` 108 | 263 | EmuSen 4, Mistress 1 [12] | `emusen-firmware` (B5) |
| Cheats: the registry and codec contracts | `CheatRegistry`, `ICheatCodeCodec`, `DelegateCheatCodec` | 530 | EmuSen 13, Crystal 4, DianaSh 1, Mistress 4 [30] | `emusen-cheats` (registry in B7) |
| Cheats: the database and the files | `CheatDatabase`, `…Installer`, `…Pruner`, `CheatImport`, `ChtFile`, `CheatWrite` | 738 | Crystal 3, DianaSh 1, Mistress 1 [8] | `emusen-cheats` (B5) |
| The debugger's registries and expressions | `Crystal/Debugger/` (ten registries, `ExpressionEvaluator`, `BreakpointExpressions`, `DebugExpressions`, the contexts, `IDebugTarget`) | 2,998 | EmuSen, DianaSh, Serenity, the frontends | `emusen-crystal`, `debug` (B7) |
| Telemetry | `Crystal/Telemetry/` | 327 | Crystal, Serenity's `coretop` | `emusen-crystal`; `coretop` reads it through the API |
| The frame recorder | `FrameRecorder` | 169 | Crystal, Hotaru 2, Pharaoh 1 | `emusen-crystal` (B7) |
| The system packs | `Crystal/Systems/` | 2,539 | EmuSen 3, Crystal [7] | `emusen-systems` (B1 data, B3 code) |
| The disc probe | `Crystal/Media/` | 127 | `SystemRouting` | `emusen-systems` (B3), over `emusen-disc` |
| The session service's provider | `Crystal/Session/` | 3,117 | DianaSh 2 [3] | **replaced**: Crystal builds the table (§3.8, B7) |
| Logging | `CategorizedLogWriter` 190, `TeeTextWriter` 26 | 216 | Mistress, Hotaru, Shell | the session's log events (B7); a frontend's own log stays its own |
| Settings statics | `AudioSettings` 53, `DebugSettings` 13 | 66 | EmuSen, Mistress, Hotaru, Shell, Pharaoh | `AudioSettings` deleted by `EmuSen_Surfaces.md` S3 (B0, built 2026-10-17); `DebugSettings` is not, S3 leaving it for `EmuSen_DianaSh.md` (Surfaces §6.3 item 5) |
| The library | Mistress `Library/`: `RomLibrary`, `GameRecords` (`games.db`), `FileRecords` (`records.db`), `PlayTime`, `RecentCache`, `GameMetadata`, `MetadataDraft`, `OpenVgdb`, `ArtworkIndex`, `CoverArtCache`, `CoverFetcher`, `MediaLibrary`, `GameFolders` | 1,889 | Mistress [WiseMan's `Mistress/`] | `emusen-library` (B8); `records.db`'s state rows to Crystal (B7) |
| Scraping and the media store | Mistress `Scraping/` | 2,802 | Mistress [12 for `MediaStore`] | `emusen-scrape` (B9); `RomHashes` folds into Galaxia's `RomHash` (B8) |

**What stays C#, and why.** `FrameScheduler` (129) is pacing and moves to Serenity under `EmuSen_Surfaces.md` S1 (B0).
The imaging and WAV writers (227) and `SqliteCatalogue` (148) serve Pharaoh's tools alone and go with Pharaoh's own
retirement or port. `StateSerializer` (189) is named by no shipped file, only by two WiseMan files, and
`CheatRomPatcher` (16) and `TeeTextWriter` (26) by none at all [M]; all three are dead and are deleted in B4, not
ported. Inside `CoreCatalog`, `ReferenceEngineByConsole` is an empty dictionary and `EngineByConsole` is never written
(`CoreCatalog.cs:387, 405–407`) [M], so the engine-choice rows around them are reachable only through discovery; they go
with B7. *Corrected by B1 (§12.4): this said "B1 and B7", but they are engine rows, not console facts.* Mistress's `ShaderCatalog`,
`SlangPackDownload` and `ThemeRecords` (`themes.db`) are frontend.

### 4.2 The totals

| | Lines |
|---|---:|
| C# backend in `EmuSen` | 11,878 |
| C# backend in `EmuSen.Crystal` | 10,797 |
| Mistress's library and scraping, backend by §2.4 | about 4,300 (of 4,665; the frontend files above excluded) |
| **The C# backend** | **about 27,000** |
| of which deleted unported (shims, pre-stable host, `PortEngine`, the three dead classes) | about 5,480 |
| of which replaced by something already designed (the service's provider) | 3,117 |
| of which frontend after all (`FrameScheduler`, Pharaoh's writers and catalogue) | about 500 |
| **Ported** | **about 17,800, plus the C# implementation of B2 (about 2,500) which B7 also replaces** |

### 4.3 Facts with two owners, and their one owner afterwards

Deliberate pairs, documented and held by a parity test with a date on which they end (the platform's C# halves, the
session header's three copies, the save library's names in both halves), are left out [M].

| # | Fact | Owners today [M] | Owner afterwards |
|---|---|---|---|
| F1 | Which console a file is, and its extensions | the packs for NES, SNES, the three Genesis systems and the PlayStation; `CoreCatalog`'s hand rows for the Game Boy and the N64 (`CoreCatalog.cs:28–40`); `CoreFactory`'s switch on `.smc .sfc .nes .gb .gbc .z64 .n64 .v64` (`CoreFactory.cs:33–41`); `BatterySave.cs:27` and `CoreCatalog.cs:198` testing `.gbc`; each core's info; Pharaoh's `LibraryIndexer.cs:42` | the pack for what the console's games come as; core info for what an engine receives; Crystal's router for the decision (§5.3) |
| F2 | An engine's name | `CoreCatalog`'s constants `"MarsRT (Rust)"`, `"MercuryRT (Rust)"`, `"MoonRT (Rust)"`, `"VenusRT (Rust)"` (`CoreCatalog.cs:358–389`); `RustCores.props`' fallbacks; each core's `display_name` | core info, read from the sidecar |
| F3 | Which engine a game runs on | Mistress: `games.db`'s `altemulator` over `graphics.json` (`MainWindow.axaml.cs:377–381`); emusen-shell: `graphics.json` alone (`CoreFactory.ConfiguredEngine`, `HotaruSession.cs`). A game set to another engine in Mistress opens on the console's engine in emusen-shell [A, read from the code and not run] | Crystal's `settings_get(game, "Engine")` (§3.6) |
| F4 | Where a state is kept | Mistress: `SaveLibrary.StatePathFor(key, slot, _appSettings.StateDirectory)` (`MainWindow.axaml.cs:641–644`); emusen-shell: `SaveLibrary.StatePathFor(path)`, the player's state folder not passed (`HotaruSession.cs:37`); the service's `state_default_path`, whatever each host hands it. With a state folder set, a state saved in one program is not found by the other [A, from the code] | Crystal's states (§3.5) over Galaxia's names |
| F5 | A state's record | `records.db` (Mistress's `FileRecords`); Galaxia's `StateRecord` sidecar, read for import (`EmuSen_RustPlatform.md` §9 Q17) [R] | Crystal's states; the sidecar's import ends with the files it reads |
| F6 | The frame-end order, and whether rewind may capture | Mistress, Hotaru and Pharaoh each (§1.3) | `emusen_crystal_frame` |
| F7 | A console's cheat codecs | `SystemPacks`; `CoreFactory.CheatCodecsFor`'s dispatch by extension (`CoreFactory.cs:220–237`); `CoreFactory.Bundle`'s per-shim switch; the Game Boy's codecs in `Crystal/Systems/GameBoy/` with no pack; the N64's in the MarsRT shim | the pack |
| F8 | How a processor's code is disassembled | MIPS twice: `emusen-mips`'s `disasm.rs` (Diamond's) and `Vr4300Disassembler.cs` (412 lines, the MarsRT shim); the 6502 in `Crystal/Systems/Nes/`; the SM83 in the MercuryRT shim | one per processor in Rust: `emusen-mips` for both MIPS processors, the others in their packs (D6) |
| F9 | The PlayStation's firmware files | Diamond's core info (`Diamond - PlayStation/src/v1.rs:77`); `PsxSystem.Bios` (`PsxSystem.cs:81–86`), named by nothing | core info; the pack's copy deleted |
| F10 | An audio or graphics setting | `GraphicsSettings` against `GraphicsConfig`, `AudioSettings` against `AudioConfig`, by hand (`EmuSen_Surfaces.md` §4.3) [R]; 21 lines still read `GraphicsSettings.` | the models; the statics deleted (B0) |
| F11 | Each console's core settings, once the runtime writes them | `graphics.json`'s `Consoles`, saved whole by Mistress's loaded copy | the runtime's `consoles.json` (§2.5) |
| F12 | A dump's hashes | Galaxia's `RomHash`; Mistress's hand-written CRC-32 and SHA-1 (`Scraping/RomHashes.cs`); Pharaoh's CRC-32 (`SignatureWriter.cs:57`); two hand-written `Crc32` classes in all | Galaxia's `RomHash`, extended (`EmuSen_GamePackages.md` §2.2) [R] |
| F13 | The record of a console | `CoreDescriptor` and `SystemEntry`, both in Crystal, converted by `CoreCatalog.FromSystem` (`CoreCatalog.cs:14–16`) | the pack's data |
| F14 | How a frontend saves or loads a state for the shell | each host hands `StateCommand` its own delegates (`HotaruSession.cs`, Mistress's `DianaShConsoleWindow`), and Mistress has its own path besides (`EmuSen_DianaSh.md` §16 Q25) [R] | the service's `state_save` and `state_load`, served by Crystal |
| F15 | How a core library is loaded and called | the C# host (`Cores/Native/`) and the kit's `api.rs`, `sidecar.rs` and `lender.rs` (704 lines of Rust) | `emusen-core-host`, used by Crystal and the kit (B6) |
| F16 | A console's first release year | the packs' `ReleaseYear` (`NesSystem.cs:22` and the others); `CoreCatalog.cs:29,40`; Big Picture's `SystemsOrder.cs:21` (`FirstRelease`) | the pack |
| F17 | A console's ScreenScraper id, and ES-DE names used as keys | `Scraper.cs:32–35` (`SystemIds`, a console table outside the packs); `CoreCatalog`'s `EsdeNames` (`:127`); `SystemsOrder.cs` | the pack (`screenscraper_system`, `esde_system`) |
| F18 | Tables keyed by console name | `CoreCatalog`'s `CatalogKeys` (`:47`), `EsdeNames`, `ButtonsByConsole` (`:213`), `SettingsByConsole` (`:297`), `SystemIdsByConsole` (`:392`); `ControllerPorts.cs:39–43`; the save folders of `BatterySave.cs:4`; Mistress's controller layouts (`ControllerDiagrams.cs:17–24`) | the pack for console facts (with its save folder's name), core info for controls and ports; a diagram is drawn by the frontend from the controller's id |
| F19 | Controller ids (`md.pad3`, `psx.dualshock` and the rest) | the packs; `ControllerDiagrams.cs:12,15`; Nephrite's and Diamond's info (`Nephrite/src/v1.rs:65–67`, `Diamond/src/pad.rs:252`) | core info; the pack's player rule refers to them by id |
| F20 | Two consoles' settings schemas | the shims (`N64Console.cs`'s `VideoSettings`, `MercuryRtCore.cs:59`) and the cores (`MarsRT/src/ffi/v1.rs:114–118`, `MercuryRT/src/ffi/v1.rs:30,74`) | core info's schema (B4) |
| F21 | A save medium's file system | the PlayStation card in `PsxCardFormat.cs` (195 lines) and in Diamond's open kernel (`kernel/src/bu.rs`, 695); the Sega CD's backup RAM in `SegaCdBramFormat.cs` (290) and in Nephrite (`src/mcd/openbram.rs`, 436). *Corrected by B3 (§13.1): Nephrite's file system is its open BIOS's `src/mcd/bios/bram.s`, 1,016 lines of 68000 assembly; `openbram.rs` is the Rust test that runs it.* Two jobs, a host's card tool and an emulated console's firmware, over one on-disk layout | the pack's format module, written at B3 as a crate with no standard library that an open firmware may also build on; whether a core adopts it is the core's own decision (D14); until then a round trip holds the two equal |
| F22 | A crash file's name | four writers, three patterns (`Mistress/CrashLog.cs:27`, `Cores/Native/CoreLibrary.cs:117`, `NativeCoreLibrary.cs`, `Galaxia/Native/PlatformLibrary.cs:53`), none of them pruned (`error_log.rs` matches `emusen_*.log`) | one rule named by Galaxia, `EmuSen_Surfaces.md` S8; Crystal writes a session's crash records through it |
| F23 | The sidecar's schema | written by the kit's `sidecar.rs:13–32` at build and by `CoreDiscovery.cs:28–45` for tests; read as untyped JSON (`CoreDescriptors.cs`) | `emusen-core-host` (B6), for the kit and Crystal alike |
| F24 | The library of games | two databases, `catalogue.db` (Galaxia's `ICatalogue` with `SqliteCatalogue`, read by Pharaoh's indexer alone) and `games.db` (Mistress's `GameRecords`), and two records named `RomEntry` | `games.db` through `emusen-library` (B8); `ICatalogue` and `catalogue.db` retire with Pharaoh's indexer (D3) |
| F25 | The shell's host glue | `SyncSchedulersToSessions`, identical in Mistress's `DianaShConsoleWindow.cs:111` and Hotaru's `GameWindow` | none: shell sessions live in the Rust shell and Crystal's boundary does the waiting (`EmuSen_DianaSh.md` §17.1, item 4) [R] |
| F26 | A display's shape | Serenity's `DisplayShape` constants named by three shims; `PsxSystem.DisplayAspect`; each core's machine info | machine info (B0, B4) |
| F27 | A memory space's name | the packs' constants (`SnesSystem`'s `CpuBus` and `WRAM`, `PsxSystem.Ram`); the cores (VenusRT names its spaces twice, `v1.rs:13` and `machine.rs:19`) | core info's spaces; the pack states only which space a console's cheats address (`cheat_space`), held to core info by a test |
| F28 | A save medium's file suffix (`.mcr`, `.brm`, `.crm`, `.mpk`) | the packs or `N64Console.cs`, and each core's info | core info's slot (§6.27, §6.28 of the core ABI) [R]; the pack's medium keeps its words and its format |

### 4.4 Two stores with two jobs

`records.db` holds each save state's record and the shader pack's build (`EmuSen_Stack.md` §2.3) [R]: one backend fact
and one frontend fact. **Recommended: the state records move to Crystal's own database, `states.db`, by one migration
that copies the rows; `records.db` keeps the shader pack's build for Mistress** [C, D8]. `graphics.json` is §2.5.


### 4.5 Toolkit types inside backend code

The line of §2 is crossed in four places today, each read from the code [M], and each settled by the step named:

- **`EmuSen → Serenity`**: ten files of `EmuSen` name Serenity's types (`SlangLend` ten times, `RawPicture` four,
  `DisplayShape` three), so the core host takes Avalonia, Skia and Vulkan with it. B0 cuts it; the API carries the core
  ABI's own lend and raw-picture structs instead.
- **`EmuSen.Endymion` takes Avalonia for one file**, `Input/DefaultPadKeyMap.cs:2`, which maps a `PadControl` to an
  Avalonia `Key`. It is the frontend's physical keyboard map and stays on the frontend's side of the line; it is noted
  because the key and hotkey binding files persist Avalonia's `Key` names and the pad binding file SDL3-CS's
  `GamepadButton` names (`ControllerKeyBindings.cs:38`, `HotkeyBindingMap.cs:100`, `GamepadBindings.cs:68`), which the
  Rust frontend will have to read (§10, R11).
- **In the library**: `CoverArtCache.cs:5–6` holds Avalonia bitmaps and posts to its dispatcher, and `MediaSources`
  implements `ISceneMedia`, an interface of Big Picture's scene (`SceneData.cs:108`). At B8 the file cache and the media
  lookup move; the decoded bitmaps and the scene's adapter stay with the frontend.
- **`PngFile.cs`** is `EmuSen`'s only use of SkiaSharp; it serves Pharaoh's and the frontends' screenshots, and stays with
  them.

---

## 5. The system packs in Rust

### 5.1 What a pack is today

`SystemPack` (`Crystal/Systems/SystemPack.cs:41`) is a record of a `SystemEntry` and five optional parts: two codec
factories, a function from settings to each player's controller, a debugger-views object and a list of save media [M].
Six packs exist (NES, SNES, Mega Drive, Mega-CD, 32X, PlayStation); the Game Boy and the N64 have codecs and a format
(`N64PakFormat`) but no pack, so their facts are `CoreCatalog`'s (F1, F7). A pack mixes two kinds of thing: **facts**,
which are data (a name, a year, a list of extensions, a cover's shape, ES-DE's names), and **behaviour**, which is code
(recognise a disc by its licence sector, decode a GameShark code, read a memory card's directory, decode a tile).

### 5.2 How a pack is described: data file and code

**Recommended: each system has a data file, and its code is a module of one crate, `emusen-systems`, keyed by the same
id** [C, D6].

- **The data file**, `Platform/systems/data/<id>.json`: `id`, `name`, `console`, `manufacturer`, `release_year`,
  `extensions` (what the console's games come as, including those the runtime expands, such as `.m3u`), `cheat_systems`,
  `openvgdb_systems`, `cover_aspect`, `esde_system`, `esde_full_name`, `self_delimiting_images`, `attached_to`,
  `screenscraper_system`, `save_folder`, `cheat_space`, `media` (each save medium's id, title and noun), and the player-controller defaults by setting key.
  *As built by B1 (§12.1), the data files carry the first fourteen of these, the part with a C# answer today; the rest
  arrive with the code that reads them at B3.* It is JSON because it is read once at start and is the kind of text a person reviews (`EmuSen_Stack.md` §4.1) [R]. It is
  embedded in the library when built, so there is no installed file to go missing, and printed by `systems` (§3.5) for
  a person or a script.
- **The code**, one module per console family (`nes`, `snes`, `genesis`, `psx`, `n64`, `gb`), each implementing what it
  has of one trait: `recognise(probe) -> Option<Recognition>`, `openvgdb_bytes`, `cheat_codecs`, `media_format(id)`,
  `debug_views`, a header reader where the console has one (`NesHeader`, `GbCartridge`, `N64ImageOrder`). A system
  with a data file and no module is a console with facts and no behaviour, which is what a pack-less system is today
  ("shown by its engine's own name and has no codecs", `SystemPack.cs:44`) [M].
- **One crate, not one per pack.** Six modules of 100 to 500 lines each (`Crystal/Systems/<id>/`, 117 to 535 lines [M])
  do not each buy a separability anyone uses (`EmuSen_Multicore.md` §9.2's pruning test) [R]; a module boundary inside
  one crate keeps them apart, and the crate is one dependency for each of its hosts.

**A data file from outside is not read in version 1** [C, D6]. A core an outside author writes names its systems in core
info with a name and its extensions (`EmuSen_CoreAPI.md` §6.3) [R], which is enough for the library to shelve it; a pack
is the project's statement about a console, and a file the runtime would accept from anywhere is a surface that needs
its own trust rule.

### 5.3 How a pack relates to the cores' own info

The rule: **a fact true of the console whichever engine runs it is the pack's; a fact about what this library does is
core info's** [C, D6]. Applied to every field that both carry or could carry:

| Field | Owner | Today, and the second copy that goes |
|---|---|---|
| The extensions an engine receives | core info | the PlayStation's pack keeps `CoreExtensions` beside `Extensions`, differing by `.m3u` (`PsxSystem.cs:38`); it goes, the runtime reading core info |
| The extensions the console's games come as | pack | — |
| Firmware files, their names, sizes and hashes | core info (§6.3 of the core ABI) | `PsxSystem.Bios`, unused (F9), goes |
| `starts_empty` | core info's system entry (§6.31 of the core ABI) | a fact of the system *as this engine implements it*: Nephrite's Sega CD does not claim it yet though the console starts empty [R] |
| Attachments (`attached_to`: the Sega CD's host is the Genesis) | pack | core info's systems name only themselves |
| The display's shape | machine info, per frame | `PsxSystem.DisplayAspect`, named by nothing outside its file [M], goes |
| Save media: a slot's kind, file and port | core info (§6.27, §6.28 of the core ABI) | — |
| Save media: the medium's words and its format module | pack | — |
| Controllers and their controls | core info | the per-console tables of `EmuSen_Surfaces.md` §4.8 retire with `CoreCatalog` |
| Which controller a port holds, from settings | pack (`PlayerControllers`) | — |
| Settings, their schema and accuracy cost | core info's schema | `CoreCatalog.SettingsFor`'s rows go |
| Recognising a disc or cartridge by its content | pack | — |
| Cheat formats | pack (`EmuSen_CoreAPI.md` §8.2: "the codecs … keyed by system id") [R] | F7's other copies go |

Sidecars stay what discovery reads without loading a core (`EmuSen_CoreAPI.md` §7.1) [R]; the packs are read at start
too, so a frontend lists consoles, engines and games without running any core code, as P6 of that page requires.

### 5.4 How a pack is shared between hosts

| Host | How it reaches the packs |
|---|---|
| Crystal in Rust (B7) | the crate's Rust API, linked |
| The library (B8) | the crate's Rust API, for routing during a scan |
| The C# runtime, until B7's state 3 | the platform library's interface, matched exactly as `emusen_platform.h` is (`EmuSen_RustPlatform.md` §3.2) [R]: `emusen_systems_table` (JSON), `_route`, `_recognise`, `_cheat_decode`, `_media_read` and `_media_edit`, `_debug_view`; facades keep `SystemPacks`' C# names. *As built by B3 (§13), the media's are `emusen_systems_media`, `_media_read`, `_media_copy`, `_media_delete` and `_media_format`, an edit being one call per operation rather than one call with an operation's number* |
| Frontends, C# or Rust | never directly: `emusen_crystal_systems` and `route`, and the library's `identity` |
| DianaSh's modules | never directly: the session service's `cheat_codecs`, `cheat_op` (decode) and the card manager's functions as the service gains them |
| The kit | not at all: it tests a core through the core ABI alone (`EmuSen_CoreAPI.md` §12.2) [R] |

---

## 6. The order of steps

Each step leaves every program working, has a gate that fails before it and passes after, keeps the C# as failover
behind a switch until the tester agrees to state 3, and says how it is proven. The switch states are
`EmuSen_RustPlatform.md` §3.9's [R]. Only blast-radius tests run for a step (`EmuSen_RustPlatform.md` §6.7) [R].

| Step | What | Gate and proof | Failover |
|---|---|---|---|
| **B0** | `EmuSen_Surfaces.md` S1 and S3, decided 2026-10-11 and not built: `FrameScheduler` to Serenity, the display shapes out of Serenity (*corrected 2026-10-17: not to the systems table, which §5.3 and F26 rule out; to Crystal's assembly until the shims retire at B4, `EmuSen_Surfaces.md` §7.1*), `EmuSen → Serenity` cut (the device lend crossing as the core ABI's `emusen_device_lend`, the raw picture as a struct of its own); `GraphicsSettings` and `AudioSettings` deleted, their values parameters; `CoreOptions.BatteryRamDisabled` a create option. The kit plan's P5 [R]. *Built 2026-10-17; the gates as measured are `EmuSen_Surfaces.md` §7.1* | the S1 and S3 gates of `EmuSen_Surfaces.md` §7: `LeafAssemblyTests` pins `EmuSen` to `{ Crystal, Galaxia }`; Pharaoh's output carries no Avalonia assembly; `GraphicsSettings.` on 0 lines; the pacing and settings tests unchanged | git: no behaviour changes, so no switch |
| **B1** ★ | **The systems table as data.** `emusen-systems` with nine data files (NES, SNES, Mega Drive, Mega-CD, 32X, PlayStation, Game Boy, Game Boy Color, N64), the table and nothing else; `SystemPacks` builds each `SystemEntry` from it, keeping its C# recognisers and codecs; `CoreCatalog`'s Game Boy and N64 rows become packs; `CoreDescriptor` is derived and never written by hand | `CoreRegistrationEquivalenceTests` and `CoreRegistrationGolden.txt` unchanged (`EmuSen_CoreAPI.md` §25) [R]; a parity class holding each C# literal entry equal to its data file field by field, run before the literals are deleted; the crate's own tests | `EMUSEN_SYSTEMS_NATIVE=0`, the C# literals, until state 3 |
| **B2** | **The frontend API v1 and its C# implementation.** `emusen_crystal.h` and its baseline; `EmuSen.Crystal.Client`; a managed implementation in `EmuSen` written from `EmulatorSession`, `CoreFactory` and the three loops' frame-end order, which is then written once; Pharaoh, emusen-shell and Mistress moved onto the binding in that order; `HostAction` synthesised from events for the C# shell; the session service 1.3 with `games`, `open_game` and `open_console` | one test per row of `EmuSen_Surfaces.md` §4.1's divergence table, run through every client (rewind gated by the engine in each; one post-load sequence); F3 and F4, and the candidates C1 to C7 of §3.12, each shown by a test that fails first or recorded as no defect; Pharaoh's `framesum` and `audiosum` equal before and after on the bench games; the shell's golden corpus; the shape check that only C-expressible types appear in the contract layer (`EmuSen_CoreAPI.md` §9.3) [R]; the header and the binding agreeing by a test | each frontend's old path stays until its own move is merged; afterwards git, since the implementation is the same C# moved |
| **B3** | **Pack code in Rust**: the recognisers and routing, header readers, cheat codecs, the card formats (`PsxCardFormat`, `N64PakFormat`, `SegaCdBramFormat`), debugger views, the OpenVGDB transform; the console code of the shims moved in (`N64GameSharkCodec`, `MoonBoards`, `GbCartridge`, the SM83 disassembler) and the VR4300's onto `emusen-mips` | a parity class per pack on generated inputs: synthetic discs and cartridges for the recognisers, every code of the cheat tests both ways, cards written by each side read by the other and edited to the same bytes, and cards written by Diamond's and Nephrite's open firmware read by the pack's modules (F21), every tile and tilemap entry of the debug-view tests; seeded faults shown to be caught (`EmuSen_RustPlatform.md` §6.5) [R]; the golden corpus's `cheat`, `cards`, `tile` and `dis` files | `EMUSEN_SYSTEMS_NATIVE=0` |
| **B4** | **The per-core shims retired.** What each adds above the ABI goes into its core's own exports, as optional capabilities of the core ABI v1 where one exists (`DEBUG_REGISTERS`, `EmuSen_DianaSh.md` Q28 [R]) and a minor where none does; MarsRT's private exports (`marsrt_core_counters`, `_cpu_registers`, `_gpu_report` and the rest, `MarsRtCore.cs:21–32` [M]) become telemetry or debug capabilities; the shims, `PortEngine` and the pre-stable host deleted; the three dead classes deleted | each core's goldens and corpus baseline unchanged (`EmuSen_Stack.md` §2.1, 2026-10-16) [R]; the kit's corpus on each core; the debugger's golden corpus on each core unchanged, or each difference recorded and decided; `grep` for a type under `EmuSen.Cores.Nintendo.` in shipped code: 0 | per core: its shim stays until its core's exports pass, reached through `CoreFactory` as now |
| **B5** | **The file services in Rust**: `emusen-firmware` (the store, the folders' order, the overview), `emusen-savemedia` (the card manager over B3's formats and Galaxia's `MediumLock`), `emusen-cheats`' files and database (`.cht`, the list files, the database's install, prune and import) | a parity class per service over generated trees (firmware whole and split, cards of each medium held and free, `.cht` files of the cheat tests); the golden corpus's `cards`, `cheat db` and `firmware` cases | `EMUSEN_SERVICES_NATIVE=0` |
| **B6** | **The core host in Rust, shared with the kit**: `emusen-core-host` in `EmuSen/Cores/Shared/` (discovery from sidecars, trust and the manifest, the checks of `EmuSen_CoreAPI.md` §7.2, the v1 calls, the input latch, the state layout), which the kit's `api.rs`, `sidecar.rs` and `lender.rs` fold into | the kit's corpus on every core gives the same report through the new crate; discovery's listing as JSON equal from the C# and the Rust over the published tree and over trees with a missing, altered and unapproved library | the kit keeps its own loader until its corpus passes; the C# host is untouched by this step |
| **B7** | **Crystal in Rust**: `emusen-crystal` over B1 to B6: sessions and the frame call, input, speed, rewind, states and `states.db`, settings and `consoles.json`, save-media placement, cheat application, the debugger's registries, expressions and bridge, telemetry, the recorder, the session service's table; exported as `emusen_crystal_*` from the platform library. Built in four parts, each graded before the next: (a) life, frame, input, speed, rewind, states, settings, media; (b) the debugger; (c) the service table; (d) the per-program switch | the parity class runs scripted sessions through the binding on both implementations and compares frame and audio digests, state bytes, files written, events drained and every service function's JSON; the debugger's golden corpus with Crystal's table; then state 2 per program: **Pharaoh first**, then emusen-shell, then Mistress, each after a full blast-radius run on Linux and the three systems' CI | `EMUSEN_CRYSTAL_NATIVE=0`, per process, the B2 implementation |
| **B8** | **The library in Rust**: `emusen-library` and `emusen_library.h`; the scan, identity over Galaxia's extended `RomHash`, `games.db` with its migrations, play time, collections and edits; Mistress's `Library/` behind the binding | a parity class: a generated library scanned by both into separate databases, compared row for row; every `games.db` version of the migration tests opened by both; the SQLite rule of `EmuSen_RustPlatform.md` §2.8 kept, one copy of SQLite per process (the C# lends `e_sqlite3` as Q25 there decided) [R] | `EMUSEN_LIBRARY_NATIVE=0` |
| **B9** | **Scraping in Rust**, if D13 says so: `emusen-scrape`, the ScreenScraper client, its quota, the media store and `media.db` | recorded ScreenScraper answers replayed to both, the requests compared byte for byte with the credentials redacted (`ScrapeRedactor`), and the stores compared; no live request in a test | `EMUSEN_SCRAPE_NATIVE=0` |
| **B10** | **State 3, one component at a time**: goldens recorded from the C# where a step's parity class does not already leave a record (`EmuSen_RustPlatform.md` §6.6) [R], then the C# deleted at the tester's word after a release played | per component, the tester's word | none, by definition |

★ The smallest first step that gives value.

**Why B1 first.** It is small (§7), it touches nothing a player sees, and it is the one fact every later step reads:
the managed implementation of B2 answers `systems` from it, B3 hangs the packs' code on it, the host of B6 joins core
info to it, and the library of B8 shelves by it. It removes F13, F16 and F17 and the Game Boy's and N64's half of F1 on its own.

**Why B2 before the rest of the Rust.** The frontends must move once, onto the surface that stays. If the Rust Crystal
came first, the failover would be each frontend keeping its old loop beside the new calls, three copies of the switch.
With B2 the switch is in one place, the binding, as each platform component's was in its facade, and the C#
implementation is both the failover and the oracle the parity class of B7 grades against. It is written mostly by
moving code that exists three times into one place; what it adds is the contract layer, the binding and the tests (§7).

**Why B4 before B6.** A Rust host cannot run per-core C#. The shims' reason to exist was to reproduce their C# oracles'
exact behaviour (`EmuSen_CoreAPI.md` §2.4) [R], and the oracles were deleted on 2026-10-16. What remains of them is
behaviour without an ABI home, and each piece must find one in its core or be retired.

**What may run beside what.** B0 and B1 are independent. B3, B4 and B5 may run beside one another once B1 is in. B6 may
start once B4's first core is through. B8 needs only B1 and B2; B9 needs B8.

---

## 7. Sizes

**The method.** Rust lines are estimated as the C# a step replaces times a ratio measured on the platform steps, where
the Rust crate with its in-crate tests and its C layer was counted against the C# it replaced as measured when each step
began [M]: Endymion 3,890 over 1,445 (2.7); Serenity's half 6,559 over 2,225 (2.9); Galaxia 4,603 over 1,603 (2.9),
an overstatement since Galaxia's crate has grown with disc files since (`EmuSen_Galaxia.md` §3.8). Those steps
reproduced .NET's text and number rules byte for byte (`EmuSen_RustPlatform.md` §5.2) [R], which Crystal's parity
through its surface does not require, so **2.5 is taken as central, with 2.0 and 3.0 as the range** [A]. C# facades are
taken at the platform steps' 250 to 300 a component. New C# and C, where there is no C# to replace, is estimated from
what it consolidates. Each step reports its real figure beside this one, as the platform steps did.

| Step | C# replaced or written | Rust, central (range) | Other new | C# removed at state 3 |
|---|---:|---:|---:|---:|
| B0 | about 300 changed | — | — | about 150 net |
| B1 | about 450 of entries | 1,100 (900–1,400); *built: 438, §12.2* | 320 of data; 200 of C# facade; *built: 144 and 103* | 450 |
| B2 | 3,600 of EmulatorSession, CoreFactory and the three loops' frame-end code | — | header and baseline 1,100; C# contract and binding 1,900; managed implementation 2,500; tests 2,000 | about 1,600 from the frontends at once |
| B3 | about 2,950 | 7,400 (5,900–8,900) | 300 of C# facade | 2,950 |
| B4 | about 5,480 deleted unported | 1,500–3,000 in the cores' own exports | 300 in the generic C# adapter | 5,480 |
| B5 | 1,250 | 3,100 (2,500–3,750) | 300 of C# facade | 1,250 |
| B6 | 1,735 in C#, 704 in the kit | 3,600 (3,000–4,300), net of what the kit brings | — | (with B7) |
| B7 | about 7,800, and B2's 2,500 | 23,000 (18,000–27,000), with the API's C layer and the service's table | — | about 13,400, with the service's provider and B2's implementation |
| B8 | about 1,900 | 4,700 (3,800–5,700) | 300 of binding | 1,900 |
| B9 | about 2,750 | 6,900 (5,500–8,250) | 300 of binding | 2,750 |
| **Total** | | **about 52,000 (41,000–62,000)** | | **about 30,000** |

The Rust total is between three and four times the platform steps' 14,600 (`EmuSen_LunaP_Kit.md` §0.3) [R] and about the size of the
kit itself (35,000 to 55,000, the same page). It is smaller than the C# it replaces would suggest at the ratio, because
about 5,480 lines are deleted unported and 3,117 are replaced by the table Crystal builds.

---

## 8. Alternatives weighed and not chosen

- **Extending the session service into the frontend API** (§3.1 (a)): a module that could open and run sessions would
  be a frontend inside the shell, and the frame's hot path is not a command's.
- **The Rust frontend on the crates' Rust API** (§3.9): it would be a client with a surface no other client has; the
  first Rust client is then the one that can drift from the C ABI unnoticed.
- **The Rust Crystal before the API** (§6, B2): three switches in three frontends instead of one in the binding.
- **The runtime owning the thread, with pacing as a callback** (§3.4): forbidden by the pull rule; with pacing as data
  it is a later minor if measured to be needed.
- **One crate per pack** (§5.2): six crates of 100 to 500 lines buy no separability anyone uses.
- **The library inside Crystal** (§2.4): a second job for the runtime, and a session would carry a database it does not
  need.
- **Leaving `graphics.json` whole** (§2.5): two writers of one file once the runtime writes settings.

---

## 9. Predictions to be retired

- **P1.** The C# implementation of B2 is no larger than the C# it consolidates: `EmulatorSession`, `CoreFactory`, the
  three loops' frame-end code and the frontends' state, rewind and settings plumbing, about 3,600 lines by §7's count.
  Retired at B2 by counting.
- **P2.** The frame call adds under 2 µs a frame on the desktop over today's `RunFrame` through the C# binding, and
  leaves `pacebench`'s frame-time distribution on the bench games unchanged within its run-to-run spread. Retired at B2
  and again at B7. If it fails at B7, a runtime-owned thread fed the display's timing as data is designed as a minor.
- **P3.** Every behaviour a shim adds above the ABI fits an optional capability the core ABI v1 already defines, except
  MarsRT's counters, which need a telemetry minor. Retired at B4.
- **P4.** The debugger's registries in Rust cost no more per breakpoint check than in C#, measured as `EmuSen_DianaSh.md`
  §13.12 measured conditions. Retired at B7(b).
- **P5.** No `games.db` row differs between the two scans of B8 except in columns that record a time. Retired at B8.
- **P6.** B1's data files reproduce every field of every C# entry with no field the C# computes rather than states,
  except `OpenVgdbBytes`, which is code (B3). Retired at B1. *Retired 2026-10-17 (§12.3), with one correction:
  `Recognise` is code as well, and both are kept from the literal.*

---

## 10. Risks

| # | Risk | Mitigation |
|---|---|---|
| R1 | **B7 is the largest single step in the project's plan after the kit**, and a runtime cannot be half switched within one process | its four parts are graded in order on Rust tests and the parity class; state 2 is per program, Pharaoh first, so one program at a time carries the risk |
| R2 | **Behaviour in the shims with no ABI home** (B4): Moon's and Mercury's refusals and battery rules, MarsRT's private exports | P3; a core ABI minor where none fits, proposed to the tester as an extension before it is built |
| R3 | **Two implementations to keep in step** from B2 to B7's state 3: a change to behaviour must land in both | the parity class fails on a change in one; the window is kept short by doing B3 to B6 beside each other |
| R4 | **SQLite twice in one process** when the library or states move while C# still opens the same files | `EmuSen_RustPlatform.md` §2.8's rule, one copy per process, kept by lending `e_sqlite3` as Q25 there decided; a database moves whole, with one owner at a time [R] |
| R5 | **Settings clobbered** by a frontend saving a stale `graphics.json` | §2.5's split before B7 writes any setting |
| R6 | **Network I/O in Rust** (the cheat database's install at B5, scraping at B9) brings an HTTP and TLS dependency on three systems | stated as its own question (SR9); B5's install may stay C# until it is decided |
| R7 | **Threads at the seam**: a change called from the frame thread that waited for the frame boundary would wait for itself | §3.3's rule: on the frame thread between frames a change applies at once; elsewhere it waits; a test for each |
| R8 | **The service's behaviour shifts when its provider changes** at B7(c), under modules that rely on version 1.2 | the golden corpus on every core with Crystal's table, both ways |
| R9 | **Windows and macOS**: the Rust shell has not run on Windows (`EmuSen_DianaSh.md` §13.10) [R], and Crystal's state 2 needs all three | the CI jobs of `EmuSen_RustPlatform.md` §27–§29 for every step's crates and WiseMan classes, with the Mac mini for the device picture [R] |
| R11 | **The binding files speak a toolkit's names** (§4.5): the Rust frontend must read Avalonia's `Key` and SDL3-CS's `GamepadButton` spellings or migrate the files | not the backend's to change; recorded for the frontend's move, where a migration by copying to canonical names is the expected answer |
| R10 | **Scope**: the backend's move delays the frontend's, which the tester has said comes after (2026-10-16) | B8 and B9 can follow K6 rather than precede it, if the tester prefers the frontend sooner; the API (B2) is what K6 needs, and it comes second |

---

## 11. Questions for the tester

The standing rules and policies this design proposes are asked first, **each on its own**, so that no rule is recorded
as decided by an answer given to another. The design choices follow, each with a recommendation. Nothing in this page is
decided by it.

**Decided 2026-10-17 by the tester.** Each standing rule SR1 to SR12 was asked and answered on its own, every one as
recommended (SR10: the file per game). The design choices D1 to D14 were then accepted together, each as recommended.
The order begins with B0 and B1, which are independent (§6). `EmuSen_CoreAPI.md` §8.4 is revised by SR6 and §2.4's
exception for shims is closed by SR8 once B4 is through. Since the same day, Windows and macOS are paused while Linux is
finished (`EmuSen_RustPlatform.md` §31), so a gate here written for the three systems runs on Linux until the port.

### 11.1 Standing rules and policies

- **SR1. The line as a rule.** Should "a thing is backend if two frontends must give the same answer about it or a
  headless client needs it; frontend if it needs a window, a display clock, an audio or input device or a toolkit, or is
  one program's presentation preference" (§2.1) become the project's rule for placing new work? *Recommended: yes*, with
  the tables of §2.2 and §2.3 as its record.
- **SR2. One way to drive a core.** Once a frontend has moved onto the API (B2), may it no longer reference `EmuSen` or
  `EmuSen.Crystal`, hold an engine or `ICore`, or name a runtime type, pinned by a reference test? *Recommended: yes*,
  extending `EmuSen_Surfaces.md` S4's test from console internals to the whole runtime.
- **SR3. The frontend API is stable from its first commit**, under the core ABI's policy with a baseline in CI (§3.2)?
  *Recommended: yes*, as Q17 of `EmuSen_DianaSh.md` decided for the host API.
- **SR4. Every session-service function has a frontend-API twin**, and the service is built by Crystal from the twins,
  held by a test in both directions (§3.8)? *Recommended: yes*.
- **SR5. One writer per settings file.** The runtime never writes `appsettings.json`; a frontend never writes the
  runtime's settings file (§2.5)? *Recommended: yes*.
- **SR6. Pacing stays the client's.** The client owns the frame thread and the pacer; the frame call performs the whole
  frame-end order (§3.4). This revises `EmuSen_CoreAPI.md` §8.4's "the runtime owns the loop and the emulation thread"
  of 2026-10-01. *Recommended: yes*, because the pull rule forbids the seam that page describes.
- **SR7. C# as failover for every backend step**, through the three states, deleted only after goldens and the
  tester's word, as `EmuSen_RustPlatform.md` Q3 decided for the platform, and with state 2 reached per program
  (Pharaoh, then emusen-shell, then Mistress) for Crystal (§6)? *Recommended: yes*.
- **SR8. No per-core code above the ABI.** With every C# oracle deleted, should `EmuSen_CoreAPI.md` §2.4's exception
  for shims be closed, so that no per-core C# or Rust survives in the host after B4? *Recommended: yes*.
- **SR9. A new dependency: an HTTP and TLS client in the platform library**, for the cheat database's download and
  scraping (§10, R6). *Recommended: one crate for the whole platform, chosen at B5 by a measured comparison of two
  candidates on the three systems, its licence recorded in `THIRD_PARTY_NOTICES.md`; until decided, downloads stay C#.*
- **SR10. Where a game's own core settings live** (§3.6): the file per game `EmuSen_CoreAPI.md` §8.5 adopted
  (`/etc/EmuSen/games/<game id>.json`), `games.db`'s `altemulator`, or a package's `settings.json`? *Recommended: the
  file per game, migrated once from `games.db`'s engine field by copying, with a package's file read in its place when
  `EmuSen_GamePackages.md` Q12 is decided for it; the badge an ES-DE theme shows reads the runtime's answer.*
- **SR11. Configuration stays JSON, program-written records SQLite**, for the runtime's new files (`consoles.json`,
  `states.db`), as `EmuSen_Stack.md` §4.1 and §2.3 hold today? *Recommended: yes*.
- **SR12. `graphics.json` split**: each console's core settings to `/etc/EmuSen/consoles.json`, by a migration that
  copies and never overwrites (§2.5). A file-format change, with its own parity case. *Recommended: yes, before B7
  writes any setting.*

### 11.2 Design choices

- **D1. The API's shape** (§3.1): exported functions `emusen_crystal_*` with the session service built from them, or
  the service extended, or a table? *Recommended: exported functions.*
- **D2. The Rust frontend's binding** (§3.9): a binding crate over the C ABI, or the crates' Rust API? *Recommended: the
  C ABI.*
- **D3. The library as backend, a component of its own** (§2.4)? *Recommended: yes*, with its own header.
- **D4. Crystal's crates in the `Platform/` workspace, exported from `emusen_platform`** (§3.1), and the core host in
  `EmuSen/Cores/Shared/` beside the kit (B6)? *Recommended: yes to both.*
- **D5. B2 before the Rust Crystal**: the API with a C# implementation consolidated from the three loops, the frontends
  moved once (§6)? *Recommended: yes*, and the binding in its own assembly, `EmuSen.Crystal.Client`.
- **D6. Packs as embedded data files and code in one crate, `emusen-systems`, with no outside data file in version 1**,
  and the field ownership of §5.3? *Recommended: yes.*
- **D7. The order B0 to B10, B1 first** (§6)? *Recommended: yes.*
- **D8. `records.db` split**: the state records to Crystal's `states.db`, the shader pack's build kept (§4.4)?
  *Recommended: yes.*
- **D9. `starts_empty` stays core info's**, though it is a fact of the system, because an engine may not implement it
  (§5.3)? *Recommended: yes*, as §6.31 of the core ABI already reads.
- **D10. DianaSh's Q25**: `games`, `open_game` and `open_console` in the session service at 1.3, served by Crystal,
  rather than in `emusen.frontend@1`, and that service's `pause`, `resume` and `is_paused` into run control, so that it
  keeps the two window operations alone (§3.8, §3.11)? *Recommended: yes.*
- **D11. B4's per-core work**: each shim's additions as its core's own exports, MarsRT's counters as a telemetry
  capability by a core ABI minor proposed before it is built (§6, P3)? *Recommended: yes.*
- **D12. Status bands** (§3.2): the service's codes kept and the API's from −1109, the library's from −1140?
  *Recommended: yes.*
- **D13. Scraping** (§2.4, B9): backend, last, as its own component; or left to the frontend's move? *Recommended:
  backend, last*, because both frontends' Big Picture read what it writes.
- **D14. A save medium's file system, twice** (F21): the pack's format modules written at B3 as crates without the
  standard library, which Diamond's and Nephrite's open firmware may build on, each core deciding for itself, and a round
  trip holding the two equal until then? *Recommended: yes*: one layout, two jobs, and the reuse rule asks for one owner.

---

## 12. What was built, 2026-10-17: B1, the systems table as data

*Measured on 2026-10-17 on branch `b1-systems`, from `WiseMan` at `69974969`. The step of §6's B1 row, with the
decisions of §11 that bear on it: D6 (one crate, embedded data files, no data file from outside), D9 (`starts_empty`
stays core info's), SR7 (the C# as failover through `EMUSEN_SYSTEMS_NATIVE`), SR11 (JSON) and D4 (in `Platform/`,
exported from `emusen_platform`).*

### 12.1 The artefacts

- **The crate `emusen-systems`** (`Platform/systems/`), an rlib depending on `emusen-dotnet` alone, for its JSON reader
  and .NET's way of writing a double. Nine data files, `data/<id>.json`: `nes`, `snes`, `md`, `mcd`, `32x`, `psx`,
  `gb`, `gbc` and `n64`, embedded with `include_str!`, so no file is installed and none from outside is read. The crate
  reads each file into a `System`, refusing a file with a key it does not know, a key missing or given twice, a value
  of the wrong kind, a blank word, an empty or repeated list, an extension that is not a dot and lower-case letters, or
  an id other than the one the file is named for; it refuses a table with an id or a console twice, or an attachment
  whose host is not a console of its own in the table. `table()`, `find(id)` and `table_json()` answer from one
  reading made on first use. The packs' code (recognisers, codecs, card formats, debugger views, the OpenVGDB byte
  transform) is not in it: that is B3.
- **The fields** are `SystemEntry`'s data fields and one more: `id`, `name`, `console`, `manufacturer`, `release_year`,
  `extensions`, `cheat_systems`, `openvgdb_systems`, `cover_aspect`, `esde_system`, `esde_full_name`,
  `self_delimiting_images`, `attached_to` and `screenscraper_system`. Each is a fact of the console on every engine, so
  the pack's by §5.3. Not among them, by the same table: the extensions an engine receives, firmware, `starts_empty`
  (D9), controllers, settings, save media's slots, and **the display's shape, which §5.3 gives to machine info**.
  §5.2's other fields (`save_folder`, `cheat_space`, `media`, the player-controller defaults) arrive with the code that
  reads them, at B3, so that no field enters the table before something holds it to a C# answer.
- **The C interface, minor addition, interface 13.** One export, `emusen_systems_table(out, len)`, the table as one
  compact JSON array in the table's order with each object's keys in a fixed order, by the length-query idiom
  (`Platform/platform/src/systems.rs`). The number went from 12 to 13 in `emusen_platform.h`, `lib.rs` and
  `PlatformLibrary.InterfaceVersion` together, as `EmuSen_RustPlatform.md` §3.2's exact rule requires (§30 there).
- **The facade, `SystemTable`** (`EmuSen.Crystal/Systems/SystemTable.cs`), behind `EMUSEN_SYSTEMS_NATIVE` through
  Galaxia's `PlatformSwitch`, in state 2 of `EmuSen_RustPlatform.md` §3.9: the table unless the variable is `0`; a
  library absent, of another interface, without the export or giving text the facade cannot read leaves the literals,
  with one line in the log. `SystemTable.Entry(literal)` is the table's row for the literal's id with the literal's
  two code parts, `OpenVgdbBytes` and `Recognise`, or the literal itself. Each pack's public entry (`NesSystem.Entry`,
  `GenesisSystems.MegaCd` and the rest) is now that call over the pack's `Literal`, so every reader of an entry, and
  every comparison by reference, reads one object, and the C# recognisers and codecs are kept.
- **The literals stay**, as the failover and the parity class's oracle, listed in `SystemPacks.Literals` in the table's
  order. `SystemEntry` gained `ScreenScraperSystem`, an optional last parameter, and each literal its value.
- **The Game Boy, the Game Boy Color and the N64 are packs**: `GameBoySystems` (two entries, the Game Boy's two codecs
  on both) and `N64System` (its entry and the Controller Pak, which `SaveMediumFormats` had listed beside the packs as
  "a console with no system pack yet"; that list is gone). The N64's codecs stay in the MarsRT shim until B3, so its
  pack has none. They are appended to `SystemPacks.All`, so every lookup that took the first match keeps its answer.
- **`CoreDescriptor` is derived.** `CoreDescriptor.Of(displayName, systems)` makes a core's row from the entries of the
  systems it runs: the first system's console and facts, every system's extensions, cheat folders and OpenVGDB names.
  `CoreCatalog`'s four hand rows became four `CatalogRow`s, each a display name per shelf and the systems' entries;
  Mercury's row is the `gb` and `gbc` entries with two shelves. The shelves' labels and ES-DE names, the hand table
  `EsdeNames`, `SystemIdsByConsole` and the `.gbc` test in `IsGameBoyColor` are read from the rows' entries. Shipped
  code builds a `CoreDescriptor` nowhere else; two WiseMan classes build small ones as fixtures, which is a test's
  business.
- **Two tables outside the packs went**: Mistress's `Scraper.SystemIds` is built from the packs'
  `ScreenScraperSystem` by `esde_system`, and Big Picture's `SystemsOrder.FirstRelease` from their `ReleaseYear`.

Of §4.3's facts, this removes **F13** (the console's record is the pack's entry, the descriptor derived from it),
**F16** (the year: the pack's alone), **F17** (the ScreenScraper id and the ES-DE names: the pack's alone) and the
Game Boy's and the N64's half of **F1** (`CoreCatalog`'s hand rows). F1's other owners, `CoreFactory`'s switch on
extensions and `BatterySave.cs:27`, are dispatch to the shims and a save folder, and go with B4 and F18.

### 12.2 Sizes, against §7

| | §7's estimate | Built |
|---|---:|---:|
| Rust, with in-crate tests and the C layer | 1,100 (900–1,400) | 438 (`systems` 391 with 112 of tests, its manifest 14, the C layer 33) |
| Data | 320 | 144 (nine files of 16 lines) |
| C# facade | 200 | 103 |
| C# packs added (Game Boy, N64) | — | 76 |
| The parity class | — | 126 |

§7 estimated the Rust as the C# entries times the platform steps' ratio, as though the entries were ported as code. They
were not: the facts became data, and the Rust is a reader and a validator of that data. The ratio does not apply to a
step whose substance moves into data files, and B3's estimate, which ports code, is not affected by this.

### 12.3 The gates

| Gate | Result |
|---|---|
| `cargo test --release -p emusen-systems -p emusen-platform` | 9 pass in the crate; 26 in the library (one new, the table through the C layer, and the header-against-exports test now reading `systems.rs`); 4 in `dianash_abi` |
| `cargo build --workspace --release` in `Platform/`; `dotnet build EmuSen.sln` | clean |
| `emusen_platform.h` as C99 and C++17, `-Wall -Wextra -pedantic -Werror` | clean |
| `CoreRegistrationEquivalenceTests` and `CoreRegistrationGolden.txt` | 3 pass, the golden not re-recorded and unchanged |
| `SystemTableParityTests` | 7 pass |
| The blast radius, one run each with `EMUSEN_SYSTEMS_NATIVE` unset and `0` | 670 pass, 670 pass |

The blast radius was the parity class, the registration golden, the catalog's and the factory's classes
(`CoreDiscoveryTests`, `CoreCommandTests`, `CoreFactoryCheatWiringTests`, `ConsoleStartTests`, `ControllerPortTests`),
the library's (`LibraryScreenTests`, `RomLibraryTests`, `PlayStationLibraryTests`, `LibraryArtworkAndMediaTests`,
`GameMetadataTests`), the packs' and routing's (`SnesSystemPackTests`, `GenesisCheatFormatsTests`,
`PsxCheatFormatsTests`, `DiamondTests`, `DiamondPlayersTests`, `NephriteTests`, `NephritePlayersTests`,
`SegaCdRoutingTests`, `MercuryModelTests`, `MercuryRtFallbackTests`, `MarsRtFrontendTests`, `MoonRtFrontendTests`), the
interface's (`GalaxiaParityTests`, `EndymionParityTests`, the three halves' export tests, `SessionHeaderTests`, the
three switch classes, `LeafAssemblyTests`), the cheats' and cards' (`CheatDatabasePrunerTests`,
`CheatPruneCommandTests`, `CheatCodecAdapterTests`, `MoonCheatTests`, `SaveMediaTests`, `SaveMediaManagerTests`,
`N64PakFormatTests`), Mistress's readers of the two moved tables (`ScraperTests`, `ScrapeExtrasTests`,
`ThemedSwitchesTests`) and `GoldenCorpusTests`. No DianaSh output was expected to change and none did.

**The parity class** reads the library's table whichever the switch chose, and holds: every literal equal to its data
file in each of `SystemEntry`'s sixteen properties (a list by its sequence, a number exactly, the two code parts by
reference to the literal's); the table, the literals and the packs naming the same nine systems in one order; every
data property of `SystemEntry` having a key and every key of each row a property, in the header's order (the schema
check of `EmuSen_RustPlatform.md` §6.4); each pack holding the entry the switch chose; the header declaring the one
export. One case holds the comparison itself to seeing a change in any one of the sixteen fields, since a comparison
that saw nothing would pass the rest.

**Seeded faults** (`EmuSen_RustPlatform.md` §6.5), each made in a data file, built, run and reverted:

| Fault | Parity class | Registration golden | The crate's tests |
|---|---|---|---|
| `snes.json`'s `cover_aspect` 0.73 to 0.74 | caught (2 cases) | caught (`[built]` and `[off]`, one line each) | caught (2 cases) |
| `psx.json`'s `screenscraper_system` 57 to 58 | caught (2 cases) | not caught | not caught |

The second is the reason for the parity class: the golden records what `CoreCatalog` and `CoreFactory` answer, and
nothing there reads a ScreenScraper id, so a wrong one would have reached the scraper unseen.

**The fallback, seen by accident.** The first run had the library at interface 13 and `PlatformLibrary` still at 12.
The library was refused ("speaks interface 13, this build 12"), the parity class failed on `Ready` with those words,
and the registration golden passed on the literals: state 2's fallback behaves as §3.9 of `EmuSen_RustPlatform.md`
says, and the C# entries alone reproduce the golden.

### 12.4 What the design page had wrong, corrected in place

- **§7's B1 sizes** were the ratio applied to data (§12.2). Corrected in §7 by a note.
- **§9's P6** named `OpenVgdbBytes` as the one field the C# computes rather than states; `Recognise` is code too, and
  both are kept from the literal. P6 is otherwise retired: every data field of every C# entry is reproduced by its data
  file, as the parity class shows, and no entry states a field the C# computes. Corrected in §9.
- **§4.1** said `ReferenceEngineByConsole` and `EngineByConsole` "go with B1 and B7". They are engine-choice rows, not
  console facts, so B1 by its own row does not reach them; they go with B7's settings schema. Corrected in §4.1.
- **§5.2's field list** is the whole pack; B1 carries the part that has a C# answer today (§12.1). Noted in §5.2.
- **An inconsistency left to B0**: §6's B0 row moves "the display shapes to the systems table", while §5.3 and F26 give
  the display's shape to machine info. B1 followed §5.3 and put no display shape in the data.

Every other figure this step relied on was checked against the code at `69974969` and held: `CoreCatalog.cs:28–41`
for the hand rows (the page says 28–40), `:14–16` for `FromSystem`, `:127` for `EsdeNames`, `Scraper.cs:32–35`,
`SystemsOrder.cs:21`, `NesSystem.cs:22`, `CoreFactory.cs:33–41`, `BatterySave.cs:27`, `CoreCatalog.cs:198`, the six
packs of §5.1 and the 2,539 lines of `Crystal/Systems/`.

### 12.5 Left for later steps

- State 3, the literals deleted, is the tester's word after goldens (§6, B10); until then a change to a console fact is
  made in its data file and its literal together, and the parity class fails on one alone.
- No program yet reads the table without the C#: `emusen_crystal_systems` (§3.5) is B2's, and the Rust Crystal B7's.
- `CatalogKeys`, `ButtonsByConsole`, `SettingsByConsole` and `AxesByConsole` (F18) remain `CoreCatalog`'s; controls and
  settings are core info's (§5.3) and go with B4 and B7.

---

## 13. What was built, 2026-10-17: B3, the packs' code in Rust

*Measured on 2026-10-17 on branch `b3-packcode`, from `WiseMan` at `358ae2b0`. The step of §6's B3 row, built in the
four parts that row's size asked for, each graded and committed before the next: (a) the card formats, (b) the cheat
codecs, (c) the recognisers, routing, header readers and the OpenVGDB transform, (d) the debugger views and the
disassemblers. The decisions that bear on it: D6 (data and code in `emusen-systems`), D14 (each medium's file system a
crate with no standard library), SR7 (the C# as failover behind `EMUSEN_SYSTEMS_NATIVE`) and SR8.*

### 13.1 Part (a): the card formats

- **Three crates with no standard library and no allocation**, in `EmuSen/Cores/Shared/`: `emusen-psx-card`,
  `emusen-n64-pak` and `emusen-mcd-bram` (`Hardware/Shared/EmuSen_MediumFormats.md`). Each edits an image in place and
  names a save by its place in the directory; a refusal leaves the image as it was. Shared by the cores' tree because an
  open firmware may build on them (D14); CI's shared-crates job tests every crate in that folder already.
- **Their words in `emusen-systems`**: a family module each (`psx::card`, `n64::pak`, `genesis::bram`) giving a save
  its id, title and game and a refusal its C# words, and `media`, which dispatches by medium id and writes a listing as
  JSON. A title's code page 932 and NFKC are tables generated from .NET 10.0.112 (`psx/cp932.rs`, 1,590 lines, and
  `psx::shift_jis`), complete for what code page 932 decodes to and for nothing else; no dependency was added.
- **The data files gained `media`**, each medium's id, title and noun, the part of a medium §5.3 gives the pack; its
  extension and its module stay the C# literal's (F28: the suffix is core info's). The table writes `media` last and the
  parity class reads it as one of `SystemTable.PackKeys`, the keys a row holds beyond `SystemEntry`'s.
- **Interface 14**, an additive minor: `emusen_systems_media` (each module's length, capacity and unit),
  `_media_read`, `_media_copy`, `_media_delete` and `_media_format`, and a status, `EMUSEN_PLATFORM_REFUSED` (−1034), for
  an operation the rules refuse, an answer with words. A damaged directory that reaches outside the image is
  `BAD_ARGUMENT`, which the facade throws as the `ArgumentOutOfRangeException` the C# threw.
- **The facade**: `SystemCode` (`Crystal/Systems/SystemCode.cs`) under the systems table's switch, which now refuses a
  library without every export the code takes, so the table and the code are chosen together and announced once. The
  three C# classes keep their names and forward (`PsxCardFormat.Instance` and the rest); each has a `Managed` twin, the
  C# alone, the failover and the parity class's oracle. A save name that cannot cross as UTF-8 stays on the C#.
- **One defect, shown and fixed in both halves**: the C# Sega CD module wrote a name back as `Encoding.ASCII` read it,
  so a copy, and a delete that rewrote the entries after the one deleted, turned a byte past 7Fh into `?` (shown by
  `SegaCdBramFormatTests.A_name_outside_ascii_is_copied_and_kept_byte_for_byte` failing on the C#: byte C1h written as
  3Fh). Its entry now keeps the stored bytes.
- **Two behaviours found and kept**: a backup RAM whose footer counts 255 or 256 files, and a delete over later entries
  that lie outside the volume, threw `ArgumentOutOfRangeException` in the C#; both halves still do (the parity class has
  a case), and whether a card tool should list such a volume is a question for the tester.
- **F21's round trips**: Diamond's open kernel formats a card and writes a file that `emusen-psx-card` reads
  (`kernel-harness/tests/card.rs`); Nephrite's open BIOS writes three files that `emusen-mcd-bram` reads, reads back
  two the crate copies in, and deletes byte for byte as the crate does (`src/mcd/openbram.rs`). Both are
  dev-dependencies of the cores' test rigs alone. Nephrite's backup RAM code is 68000 assembly (`bios/bram.s`), so its
  firmware cannot build on a Rust crate as it stands: for it D14's round trip is the lasting form, not a stage before
  adoption.

| Part (a) | Lines |
|---|---:|
| C# replaced (the three modules at `358ae2b0`) | 718 |
| Rust, the three crates with their tests | 1,272 |
| Rust, `emusen-systems`' media and family modules, hand-written | 610 |
| Rust, the generated code page 932 tables | 1,590 |
| Rust, the C layer and the table's `media` | about 200 |
| C# facade (`SystemCode` and the forwarding in the three classes and `SystemTable`) | about 230 |
| The parity class `SaveMediaParityTests` | 422 |

| Gate | Result |
|---|---|
| `cargo test --release` in each new crate | 6, 7 and 7 pass |
| `cargo test --release -p emusen-systems -p emusen-platform` | 15, 27 and 4 pass |
| Diamond's `kernel-harness` `card` tests; Nephrite's `openbram` tests | 4 pass; 10 pass |
| `emusen_platform.h` as C99 and C++17, `-Wall -Wextra -pedantic -Werror` | clean |
| `SaveMediaParityTests` | 8 pass: 300 generated cards, 300 paks and 400 backup RAMs read, and each save copied and deleted, byte for byte or refused alike; noise images and wrong lengths; every lead and trail of code page 932 through a title; edits by each half read by the other |
| The blast radius, with `EMUSEN_SYSTEMS_NATIVE` unset and `0` | 270 pass, 270 pass |

The blast radius was the two parity classes, the three format modules' classes, `SaveMediaManagerTests`,
`SaveMediaTests`, `DiamondShellTests`, `NephriteMediaTests`, `MemoryCardsOfferTests`, Mistress's Memory Cards window
cases, `GoldenCorpusTests` (its `memcard` file unchanged), `GalaxiaParityTests`, `EndymionParityTests`,
`SessionHeaderTests`, `CoreRegistrationEquivalenceTests` and `LeafAssemblyTests`.

**Seeded faults**, each made, built, run and reverted:

| Fault | Caught by |
|---|---|
| the card's checksum over 126 bytes | the parity class (generated cards); not the crate's own tests |
| an unmapped code page 932 pair read as `?` | the parity class (3 cases) |
| NFKC without its composition | the parity class (3 cases) |
| a note's id without its extension's dot | the parity class (2 cases) |
| the backup RAM's directory blocks as (n + 1) / 2 | the parity class (2 cases), `SegaCdBramFormatTests` (2), the crate's tests (1) |
| the facade dropping `damaged` from a listing | the parity class (3 cases) |

### 13.2 Part (b): the cheat codecs

- **Eleven codecs in `emusen-systems`**, a family module each (`nes::cheats`, `snes::cheats`, `genesis::cheats`,
  `psx::cheats`, `gb::cheats`, `n64::cheats`) and `cheats`, which names each codec by an id (`nes.raw`, `md.gg`,
  `n64.gs` and the rest), pairs them per system as the packs do (detected, then explicit) and answers a code with all
  four of `ICheatCodeCodec`'s answers at once: whether the code is the codec's, its one byte, its compare byte and its
  writes, each refusal with the C# codec's words. The two Game Genie encoders (the SNES's and the Genesis's) are ported
  too, though only tests call them.
- **The N64's codec joined its pack** (F7): `N64GameSharkCodec` moved from the MarsRT shim to `Crystal/Systems/N64/`
  as the Rust codec's failover, `N64System` gained `GameShark()` and its pack the codec, and the shim's
  `MarsCheatCodecs` became a name for the pack's. `CoreFactory.CheatCodecsFor`'s dispatch by extension and the MarsRT
  bundle's own codec gave way to the pack of the console the core runs, so every console's codecs are its pack's.
- **The data files gained `cheat_space`**, the space a console's RAM cheats write into (F27), read by the codecs and held
  by the parity class to each pack's C# codec's `SpaceName`. F27 asks for it to be held to core info by a test as well;
  that needs a machine's info, which only a loaded game gives, and is left to B7, when the runtime first reads the
  field. Until then the cores' own cheat tests apply codes through those space names.
- **No new export beyond interface 14's minor**: `emusen_systems_cheat_codecs`, `emusen_systems_cheat_decode` (one
  code's answers as JSON) and `emusen_systems_cheat_encode`. The facade is `NativeCheatCodec`, which each pack's codec
  factory returns unless `EMUSEN_SYSTEMS_NATIVE=0`, fronting the C# codec (`Managed`) for a null or a code that cannot
  cross; it keeps the last code's answers, since a caller asks `CanDecode` and then decodes the same code. Each pack
  keeps its C# codecs as `Managed…` factories, listed in `SystemPacks.ManagedCodecs` for the parity class.
- **Two of .NET's rules joined `emusen-dotnet`**, measured on .NET 10.0.112: `char.ToUpperInvariant` sends the long s to
  S and keeps the dotless i (`upper_invariant`), which decides whether `ſſſſ-ſſſſ` is a Genesis Game Genie code; and
  `char.IsLetterOrDigit` now answers from .NET's own 398 ranges above ASCII, where it had approximated with Rust's
  alphabetic and numeric classes, which count superscripts and Roman numerals .NET does not. The shell's lexer, which
  also reads it, keeps its golden corpus unchanged.

| Part (b) | Lines |
|---|---:|
| C# replaced (the seven codec files of `Crystal/Systems/` and the shim's `N64GameSharkCodec`) | 848 |
| Rust, the codecs | 1,106 |
| Rust, `emusen-dotnet`'s two rules with the generated ranges | about 90 |
| Rust, the C layer | about 60 |
| C# facade (`NativeCheatCodec`, `SystemCode`'s part, the packs' factories) | about 200 |
| The parity class `CheatCodecParityTests` | 187 |

| Gate | Result |
|---|---|
| `cargo test --release -p emusen-dotnet -p emusen-systems -p emusen-platform` | 26, 16, 27 and 4 pass |
| `CheatCodecParityTests` | 5 pass: every codec's name, kind, space and pair; 936 codes taken from every cheat test and from the golden corpus's `cheat` file, and 6,008 generated codes, each through all eleven codecs and all four answers, words included; the encoders on 3,000 generated patches and every Game Genie code of the tests |
| The blast radius, with `EMUSEN_SYSTEMS_NATIVE` unset and `0` | 540 pass, 540 pass |

The blast radius was the two parity classes and the table's, every codec and cheat class (`CheatCodecAdapterTests`,
`NesGameGenieCodecTests`, `MoonCheatTests`, `MarsCheatTests`, `GenesisCheatFormatsTests`, `SnesSystemPackTests`,
`PsxCheatFormatsTests`, `CodecProperties`), the cheat commands' and database's classes, `CoreFactoryCheatWiringTests`,
Mistress's `ActiveCheatsWindowTests`, `CheatDatabaseWindowTests` and `ConsoleContextTests`, the cores' classes that
apply cheats (`MercuryRtCoreTests`, `DiamondTests`, `NephriteTests`, `MarsRtFrontendTests`, `MoonRtFrontendTests`),
`CoreRegistrationEquivalenceTests`, `SessionServiceParityTests`, `GalaxiaParityTests` and `GoldenCorpusTests`, its
`cheat` file unchanged.

**Seeded faults**, each made, built, run and reverted:

| Fault | Caught by |
|---|---|
| the NES's eight-letter value bit 30 for 31 | the parity class (2 cases) |
| a SNES Action Replay write given a repeat count of 1, where the C# left 0 | the parity class (2 cases) |
| the Genesis's codes upper-cased as ASCII alone | the parity class (generated codes) |
| the PlayStation's letter-or-digit test as Rust's alphanumeric | the parity class (generated codes) |
| the Xploder's key 7 adding where it subtracts | the parity class (2 cases) |
| the N64 repeater without its step's alignment check | the parity class (2 cases) |
| the Game Boy compare byte rotated left | the parity class (2 cases) |
| the facade dropping a slide's value step | the parity class (2 cases), `PsxCheatFormatsTests` (1) |

The codecs' own WiseMan classes caught none of the faults in the Rust, because they call the C# decoders' static
methods directly; only a codec handed out by a pack goes through the facade. They remain the C#'s tests, and the parity
class is the Rust's.

