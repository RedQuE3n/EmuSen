# EmuSen — the backend in Rust: the line, the frontend API, the system packs and the services

*This revision: the first, 2026-10-17. A design; B1, B3 and B5 have since been built (§12, §13, §15, 2026-10-17), B2's first three parts (§14, 2026-10-17), and the cheat database's download on SR9's client (§16, 2026-10-18). It answers the tester's goal of 2026-10-16, "to
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
| **B5** | **The file services in Rust**: `emusen-firmware` (the store, the folders' order, the overview), `emusen-savemedia` (the card manager over B3's formats and Galaxia's `MediumLock`), `emusen-cheats`' files and database (`.cht`, the list files, the database's install, prune and import) | a parity class per service over generated trees (firmware whole and split, cards of each medium held and free, `.cht` files of the cheat tests); the golden corpus's `cards`, `cheat db` and `firmware` cases. *Built 2026-10-17, §15; the download on SR9's client built 2026-10-18, §16* | `EMUSEN_SERVICES_NATIVE=0` |
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
| B3 | about 2,950 | 7,400 (5,900–8,900); *parts (a) to (c) built: about 4,000 for about 1,860 replaced, §13.5* | 300 of C# facade; *built: about 630* | 2,950 |
| B4 | about 5,480 deleted unported | 1,500–3,000 in the cores' own exports | 300 in the generic C# adapter | 5,480 |
| B5 | 1,250 | 3,100 (2,500–3,750); *built: 2,497 for about 1,020 replaced, §15.5* | 300 of C# facade; *built: about 730* | 1,250 |
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
| R6 | **Network I/O in Rust** (the cheat database's install at B5, scraping at B9) brings an HTTP and TLS dependency on three systems | stated as its own question (SR9); B5's install may stay C# until it is decided. *SR9 decided 2026-10-18; the client is `emusen-net`, §16, and its portable check's need of a C toolchain for `ring` is §16.6* |
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
  *Decided 2026-10-18 by the tester, on B5's measurement (§15.4): `ureq` 3 with `rustls` is the platform's one HTTP and
  TLS client, and certificates are checked against each system's own store through `rustls-platform-verifier`, as
  .NET's `HttpClient` checks them today, not against a bundled `webpki-roots`.*
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

*On 2026-10-17, after part (b), the branch was rebased onto `WiseMan` at `628bc821`, which carries B0. B0 left the
platform library's interface at 13, so 14 stayed this step's; the rebase met one conflict, in
`EmuSen_RustPlatform.md`, where B0 had taken §31 (Windows and macOS paused) and this step's interface moved to §32.
Parts (a) and (b)'s blast radius was run again on the rebased tree: 647 pass with the switch unset and 647 with it
`0`.*

### 13.3 Part (c): the recognisers, routing, header readers and the OpenVGDB rules

- **Routing by content in `emusen-systems`**: `probe` holds what a recogniser is shown (a head of the file, asked for
  by length: `Head`, and `Need` where a recogniser must read more), the data track's sectors and the ISO 9660 lookup
  (`DiscSectors`, `Crystal/Media/DiscSectors.cs` ported; `emusen-disc` deliberately has no file system, so the lookup
  stays a pack's), and `route`, the decision among the systems that take an extension. The recognisers are the
  Genesis family's (`genesis::recognise`, a cartridge against a disc, the Sega CD's system name) and the
  PlayStation's (`psx::disc`: the licence text, its market and SYSTEM.CNF's serial).
- **The file is still read by the C#.** A recogniser that needs more than it was given answers with the length it
  reads, and the facade reads that much through the probe and asks again (`emusen_systems_recognise`), so the reads
  are the C# recognisers' reads, length for length, which the parity class checks: no callback crosses the C layer
  (`EmuSen_RustPlatform.md` §3.7), and `MediaProbe` stays Galaxia's archives, sheets and lists. The cache of decisions
  and the folder's name stay `SystemRouting`'s; the decision itself is the library's, with the C# kept as
  `SystemRouting.DecideManaged`.
- **The header readers**: `nes::header` (`NesHeader`) and `gb::cartridge` (`GbCartridge`, moved from the MercuryRT
  shim to `Crystal/Systems/GameBoy/` as its failover). Each reads within an image's first 4 KiB and its length, so a
  ROM is not copied to be read. `GbCartridge`'s refusal of a type no board implements is MercuryRT's statement more
  than the console's; it is kept as the C# made it and belongs with core info when the shim goes (B4).
- **The OpenVGDB rules** (`openvgdb`): the NES's header, the SNES's copier header, the N64's byte order
  (`n64::byte_swapped`, `N64ImageOrder` ported); a file another system hashes as it is crosses back as `ABSENT`, so it
  is not copied.
- **Interface 14 gained** `emusen_systems_code` (which systems have each kind of code), `_recognise`, `_route`,
  `_openvgdb` and `_header`. Each pack's entry now carries the library's delegates where the switch chose the table,
  and a recogniser only where its literal has one; `SystemTableParityTests` holds that.
- **Four defects, shown and fixed in both halves.** The C# disc lookup threw on a malformed volume or directory,
  which routing's catch does not take, so one crafted disc in a library would have stopped a scan: a directory
  record running past its sector (`ArgumentOutOfRangeException`), a root directory before sector 0 (the same), a file
  of negative length (`OverflowException`), and a root directory claiming close to 2 GiB, whose `int` loop overflowed
  and never ended. Shown on 2026-10-17 by `DiscProbeTests`, all four failing on the C# (the last by its five-second
  bound); now a record past its sector ends that sector's scan, a sector before the first is none, a negative length
  is no file, the loop counts in `long`, and a file's sectors are found before its bytes are allocated. The
  PlayStation recogniser's market line read past an absent sector 4 in the same way, and reads as empty instead.

| Part (c) | Lines |
|---|---:|
| C# replaced (the disc probe's sectors and lookup, the decision, two header readers, the byte order, the recognisers, the OpenVGDB rules) | about 290 |
| Rust, `emusen-systems` | 458 |
| Rust, the C layer | about 250 |
| C# facade (`SystemCode`'s part, the entries' delegates, `SystemRouting`, the two readers) | about 200 |
| The parity class `SystemCodeParityTests` and the defects' `DiscProbeTests` | 213 and 84 |

| Gate | Result |
|---|---|
| `cargo test --release -p emusen-systems -p emusen-platform` | 19, 27 and 4 pass |
| `emusen_platform.h` as C99 and C++17 | clean |
| `SystemCodeParityTests` | 6 pass: each system's code where its literal has; every recogniser on about 480 generated images (the synthetic PlayStation discs of each market and mode as `.iso` and raw tracks, nine SYSTEM.CNF variants, Sega CD discs and raw sectors, Genesis cartridges, each cut at every length a read straddles and with noise written over parts, and noise), the answer and the lengths read alike; 4,000 routing decisions over random candidates, claims and folders; every OpenVGDB rule on generated files; 6,000 NES headers and every Game Boy cartridge type at five lengths |
| `DiscProbeTests` | 4 pass, after failing on the C# before its fix |
| The blast radius, with `EMUSEN_SYSTEMS_NATIVE` unset and `0` | 518 pass, 518 pass |

The blast radius was the three parity classes of this step and the table's, `DiscProbeTests`,
`SegaCdRoutingTests`, the Diamond classes that route and boot discs (`DiamondTests`, `DiamondPlayersTests`,
`DiamondLaunchTimeTests`), `NephriteTests`, the Mercury classes that read a cartridge (`MercuryCartridgeTests`,
`MercuryRtCoreTests`, `MercuryModelTests`, `MercuryRtFallbackTests`, `MercuryDebugTargetTests`),
`MoonRtFrontendTests`, `SnesSystemPackTests`, `CoreRegistrationEquivalenceTests`, `CoreDiscoveryTests`,
`ConsoleStartTests`, `CoreFactoryCheatWiringTests`, the library's and the scraper's classes that hash or route
(`PlayStationLibraryTests`, `LibraryScreenTests`, `RomLibraryTests`, `LibraryArtworkAndMediaTests`,
`GameMetadataTests`, `OnlineCoverTests`, `OnlineCoverWindowTests`, `ScrapeDiscImageTests`, `ScrapeExtrasTests`,
`ScraperTests`, `ScrapeWindowTests`), `MainWindowLogDirectoryTests` and `GoldenCorpusTests`.

**Seeded faults**, each made, built, run and reverted:

| Fault | Caught by |
|---|---|
| a serial's underscore left in place of the dash | the parity class, `DiamondPlayersTests` |
| the PlayStation recogniser reading 128 KiB where the C# read 160 | the parity class (the lengths read) |
| a mode 2 Sega CD sector's data taken at 16 | the parity class |
| a folder's name compared with its case | the parity class (routing) |
| the SNES copier header taken from files of a whole number of KiB | the run's end: the library panicked on a file shorter than 512 bytes and the test host aborted |
| an NES header with a clean byte 7 called unverifiable | the parity class (headers) |
| MBC3's timer on type 0Fh alone | the parity class (headers) |
| a raw mode 1 sector's data taken at 24 | the parity class (recognisers) |
| the facade dropping a recognition's market | the parity class (2 cases), `DiscProbeTests` (4), `DiamondPlayersTests` |

### 13.4 Where the step stopped, and why: part (d) waits on three questions

*Decided 2026-10-17 by the tester, each as recommended: the views read memory in rounds, with a whole-space snapshot
for the two tile sheets of spaces with no side effects, behind a reader trait B7 implements; `MoonBoards` is split, its
board names to the NES pack and the rest to MoonRT's own exports at B4; `emusen-mips` gains MIPS III with the spelling a
field of `Dialect`, so neither golden changes; and a Sega CD backup RAM whose footer counts 255 or 256 files is listed
as damaged, its readable saves shown and writes refused, rather than thrown on.*

Part (d), the debugger views and the disassemblers with the shims' remaining console code, was not built. Reading it
before building showed three design questions the page does not answer, and the rule for this step is to stop at the
last graded part and ask rather than choose:

1. **How a view reaches memory.** Every view reads the engine's spaces through `IDebugMemorySpace.Read`, one byte at a
   time, and some spaces may disturb state when read (`HasSideEffects`). The C layer takes no callback (§3.7 of
   `EmuSen_RustPlatform.md`), so a view in Rust must either be handed whole spaces (a snapshot, which reads bytes the
   C# view never read, the SNES tile sheet's 64 KiB of VRAM or an IO space's registers among them) or ask in rounds
   for the bytes it reads, as the recognisers ask for a file's head (which keeps the C# reads, at the cost of a
   protocol whose read order the C# interleaves byte by byte for the NES's tile sheet). The page's `_debug_view` names
   neither. *Recommended:* rounds for the views that read registers or small regions (palettes, a tile, a tilemap
   entry, the DMA channels) and a snapshot for the two tile sheets, the snapshot allowed only of a space that states no
   side effects; and since Crystal in Rust (B7) will read a core's memory through the core ABI directly, the views'
   Rust takes a reader trait that both the facade's protocol and B7 implement.
2. **`MoonBoards` is MoonRT's, not the NES's.** The page lists it among the shims' console code for the packs. It is
   the table of the sixteen boards MoonRT builds, each with the paths of its registers *in MoonRT's state*
   (`Mapper._control`) and a mirroring rule that reads them; Pharaoh and the battery rule read it for which mappers
   MoonRT implements. Only a mapper's number and its board's name are facts of the console; the rest is what one engine
   does, which §5.3 gives to core info. *Recommended:* the board names move to the NES pack as data; the paths, the
   mirroring rule and the list of mappers implemented go with B4 into MoonRT's own debug registers capability and its
   info.
3. **The VR4300's disassembler onto `emusen-mips` (F8)** needs a choice of spelling. `emusen-mips` decodes MIPS I as the
   R3000 does and writes Appendix A's text (register 30 as `fp`, no folded forms), which Diamond's `disasm` output and
   its golden corpus hold; the MarsRT shim's `Vr4300Disassembler` decodes MIPS III with the FPU and writes
   `Mars_Disassembler.md`'s text (`s8`, `beqz` and `b` folded from `beq`, `.word` for what it does not decode, its own
   hex). One crate for both processors either carries two spellings, a dialect's option, or makes one console's
   debugger output change. *Recommended:* `emusen-mips` gains MIPS III's decode and the VR4300's coprocessor tables, and
   the spelling becomes a field of `Dialect` with the R3000's and the VR4300's each kept, so neither golden changes;
   decided by the tester before it is built.

The SM83 and 6502 disassemblers, the NES and SNES views' pure parts (vectors, symbols, a summary line, the B bus's
names) and `Nes6502Disassembler` raise none of these and wait only on the same part.

### 13.5 Part (d), 2026-10-17: the debugger views, the disassemblers and the damaged backup RAM

*Built on branch `b3-packcode-d` from `WiseMan` at `8b09514b`, on the tester's four answers recorded at the head of
§13.4.*

- **The views read memory through a reader** (`emusen-systems`' `debug::Memory`): a byte, a batch of bytes whose
  addresses the view knew before it read any of them, and a run from a space's start. B7's Crystal will implement it
  over a core's memory; the C# facade's implementation is `debug::Tape`, the bytes the host has read so far for one
  request in the order asked. The first read past them is asked for with the rest of its batch, and every read after it
  answers 0 and is not asked for, since its address may hang on a byte not yet read; so the host reads exactly the
  bytes the C# view read, in its order, a batch to a round. The one view whose reads hang on what it reads is the SNES's
  DMA table, which reads a channel's line counter only for an enabled HDMA channel; it takes up to 25 rounds, the rest
  one or two. The two tile sheets read a space whole where it states no side effects, the NES's interleaved byte by
  byte where it does (the SNES's sheet already read its 64 KiB in order).
- **The views** are `nes::views` and `snes::views`, each of `ISystemDebugViews`' members the C# implements: palettes,
  the swatch, the tile sheet, the tilemap entry and its stride, a tile's pixels (the refusal of a depth with the C#'s
  words and parameter), the vectors, and the SNES's DMA channels, the B bus's names, the symbols and the summary line.
  The facade is `NativeDebugViews`, which each pack's views are unless `EMUSEN_SYSTEMS_NATIVE=0`, fronting the C#
  views; the MoonRT shim's debug target now reads the NES pack's views rather than the C# class.
- **The disassemblers**: `nes::disasm` (the 2A03), `gb::disasm` (the SM83, moved with its C# from the MercuryRT shim)
  and the VR4300's in `emusen-mips` (`vr4300.rs`, MIPS III in the VR4300's spelling, `Spelling` now a field of
  `Dialect`; `EmuSen_Mips.md` §6), reached through `n64::disasm`, its C# with `MarsOperands` moved from the MarsRT shim
  to `Crystal/Systems/N64/`, where the shim's RSP disassembler still reads them. Each C# method forwards, its C#
  kept as a `…Managed` twin. `blackmoon-gte` and `blackmoon-r3000` name the R3000's spelling where they build a
  `Dialect`; nothing they do changed.
- **`MoonBoards` split**: the sixteen boards' names by mapper number are the NES pack's (`nes::boards`,
  `NesSystem.BoardName`), and `MoonBoards` takes its names from there; its register paths, mirroring rules and the list
  of mappers MoonRT implements stay in the shim for B4, as decided.
- **A backup RAM whose footer counts 255 or 256 files** is listed as damaged in both halves: its readable entries (254
  of an 8 KiB volume's) with `Problem` saying why, a copy onto it and a delete from it refused, a save copied off it
  allowed, nothing thrown. Shown by `SaveMediaParityTests.A_directory_reaching_outside_the_volume_is_listed_as_damaged_in_both`
  failing first (`ArgumentOutOfRangeException`). The delete over later entries outside the volume (§13.1) still throws
  as it did; it was not among the questions.
- **Interface 15**, since 14 was merged: `emusen_systems_disassemble`, `emusen_systems_board_name` and
  `emusen_systems_view` (`Platform/platform/src/systems_debug.rs`). B2, which may take 15, rebases to the next number.

| Part (d) | Lines |
|---|---:|
| C# replaced (the two views 401, the three disassemblers 741, the board names) | about 1,160 |
| Rust, `emusen-systems`' views, disassemblers, boards and reader | 795 |
| Rust, `emusen-mips`' VR4300 table | 410 |
| Rust, the C layer | 249 |
| C# facade (`NativeDebugViews`, `SystemCode`'s part, the disassemblers' forwarding) | about 270 |
| The parity class `DebugCodeParityTests` | 240 |

| Gate | Result |
|---|---|
| `cargo test --release` in `emusen-mips`, `blackmoon-gte`, `blackmoon-r3000`, Diamond's `kernel-harness` | 11, 10, 38 and every harness test pass |
| `cargo test --release -p emusen-systems -p emusen-platform` | 19, 29 and 4 pass |
| `emusen_platform.h` as C99 and C++17 | clean |
| `DebugCodeParityTests` | 6 pass: every 6502 and SM83 opcode at six addresses and 21 operand pairs, every SM83 CB suffix, the references of each at seven operands; about 298,000 VR4300 words (every primary opcode with every function and register pattern the decode reads, and 200,000 random); every mapper number from −2 to 4,099; the NES's views on 8 memory shapes, every tilemap entry of a 4 KiB nametable space and every tile of an 8 KiB CHR at two depths; the SNES's on 6 shapes, every tilemap entry of 64 KiB of VRAM, every tile at four depths, every B bus register, 400 register sets for the summary and seven symbols; each answer and the reads it made compared, in order, or as a set for a sheet read whole |
| The golden corpus's `tile`, `tilemap` and `disasm` files | unchanged (`GoldenCorpusTests` passes with neither re-recorded) |
| The blast radius, with `EMUSEN_SYSTEMS_NATIVE` unset and `0` | 1,047 pass, 1,047 pass |

The blast radius was the parity classes of all four parts and the table's, the card classes (`SegaCdBramFormatTests`,
`SaveMediaManagerTests`, `NephriteMediaTests`), every class that reads a view, a disassembly or a reference
(`CoreDebugViewTests`, `TilePixelDecodeTests`, `StaticReferenceClassificationTests`, `SetRegVectorsAddrTests`,
`ConditionBreakAndDmaTests`, `CoreAdapterTests`, `SnesSystemPackTests`, `VenusRtDebugTests`, `VenusRtDebugViewTests`,
`MoonDebugTargetContract`, `MoonRtFrontendTests`, `MercuryDebugTargetTests`, `MercuryAuditTests`, `MercuryModelTests`,
`MercuryRtCoreTests`, `MarsDisassemblerTests`, `MarsDebugTests`, `MarsRtTests`, `MarsRtFrontendTests`,
`DiamondDebugTests`, `DiamondTests`, `NephriteDebugTests`), `DisasmWidthHintTests`, `SessionServiceParityTests`,
`CoreRegistrationEquivalenceTests`, `LeafAssemblyTests` and `GoldenCorpusTests`.

**Seeded faults**, each made, built, run and reverted:

| Fault | Caught by |
|---|---|
| the 6502's branch counted from the third byte | the parity class, `MoonDebugTargetContract` |
| the SM83's positive displacement without its plus | the parity class |
| the VR4300's register 30 written `fp` | the parity class, `MarsDisassemblerTests` (5) |
| the VR4300's jump region taken from the jump, not its delay slot | the parity class, `MarsDisassemblerTests` (3) |
| the NES's backdrop mirror tested with mask 03h | none: an equivalent mutant, since the entries it adds to the mirror map to themselves; recorded, and replaced by the next |
| the NES's backdrop mirrored to the wrong entry | the parity class |
| the SNES's DMA table reading every line counter | the parity class (the reads) |
| the tape reading on past its first gap | the parity class (the reads), `VenusRtDebugViewTests` |
| MMC3 named `MMC-3` | the parity class |
| the facade dropping a refusal's parameter | the parity class |
| an overcounted backup RAM deleted from | the parity class (3 cases) |

### 13.6 Sizes, against §7, and what is left

| | §7's estimate | Built, parts (a) to (d) |
|---|---:|---:|
| C# replaced | about 2,950 | about 3,020 (718, 848, about 290 and about 1,160) |
| Rust, hand-written, with in-crate tests and the C layer | 7,400 (5,900–8,900) | about 5,500 |
| Rust, generated from .NET (code page 932, the letter-or-digit ranges) | — | 1,645 |
| C# facade | 300 | about 900 |
| The parity classes | — | 1,062, with `DiscProbeTests` 84 |

The Rust ran at about 1.8 times the C# it replaced, against §7's 2.5: the format crates came in larger (their in-place,
allocation-free surface and their own tests), the codecs, routing, views and disassemblers smaller, being ports of
tables and of short functions whose .NET text rules were few. The facade is three times §7's figure because each part
has its own call shapes and the views their rounds; it is deleted whole at state 3.

**Left for later:**

- State 3, the C# formats, codecs, recognisers, readers, views and disassemblers deleted, at the tester's word after
  goldens (§6, B10). Until then a change to a module is made in both halves, and the parity classes fail on one alone.
- B4: `MoonBoards`' register paths, mirroring rules and implemented mappers into MoonRT's own exports; `GbCartridge`'s
  refusal of a type no board implements into MercuryRT's info; the RSP's disassembler with the MarsRT shim.
- F27's test of `cheat_space` against a core's machine info (§13.2), with B7.
- The delete over a backup RAM whose later entries lie outside it, which still throws (§13.1).
- `PsxSystem.Bios` and `DisplayAspect`, which §5.3 says go, are untouched: neither is code this step moved.

---

## 14. What was built, 2026-10-17: B2, the frontend API v1 and its C# implementation

*Measured on 2026-10-17 on branch `b2-frontend-api`, from `WiseMan` at `628bc821` (B0 and B1 in). The step of §6's B2 row,
built in parts with a review between the second and the third: part 1 is the header, its baseline and checks, the
binding and the C# implementation with their own tests (§14.1 to §14.7); part 2 is Pharaoh moved onto the binding
(§14.8). emusen-shell with Hotaru, Mistress and the session service's 1.3 come after the review. The decisions that bear
on it: SR2 to SR7, D1, D5, D10 and D12 (§11).*

### 14.1 The artefacts

- **The header, `EmuSen.Crystal/include/emusen_crystal.h`**, beside `emusen_session.h`, which it includes, so that the
  service's statuses and enumerations are the API's too and a twin passes its status through. 132 exports: 54 of the
  API's own and the 78 twins of the session service's 79 functions (the 79th, `last_error`, is the library's own, which
  takes the runtime as its first argument and may be given none). Three size-prefixed structs (`emusen_crystal_config`, 8
  bytes; `emusen_crystal_open_params`, 64; `emusen_crystal_frame_info`, 104), two opaque handles (the runtime, a
  subscription). It compiles as C99 and C++17 with `-Wall -Wextra -Werror -pedantic`.
- **The Rust side, the crate `emusen-crystal`** (`Platform/crystal/`): the header's numbers and structs as `abi`, the
  baseline `abi/v1/baseline.txt` (198 facts: 33 constants, 132 functions, 3 structs and their 32 fields), and the test
  `tests/crystal_abi.rs`, run by the core ABI guard job as a step of its own (`The frontend API's surface`) with the base
  branch's baseline, as DianaSh's surfaces are (`EmuSen_DianaSh.md` §11.4). It holds: every constant equal; the statuses
  in −1109 to −1139; every session-service function's twin present, in the table's order, with the same return and the
  same parameters by type **and by name** after the runtime, and the header's twins exactly the table's functions; the
  layouts the C compiler gives equal to the Rust ones; the header as C++17; the baseline neither losing nor gaining a
  fact unrecorded; and every `emusen_crystal_` export of the platform's crates declared by the header with its signature.
  At 1.0 the platform library exports none: the C# implementation serves the API until B7, so that check is a subset
  check now and an equality from B7.
- **The binding, `EmuSen.Crystal.Client`**, one assembly, two layers as `EmuSen_CoreAPI.md` §9.2 drew them. The
  contract layer (`EmuSen.Crystal.Client.Contract`) is C-shaped: `CrystalExports`, the exports' function pointers in the
  header's order; `Crystal`, one static function per export forwarding through that table; `CrystalAbi` and
  `CrystalSessionAbi`, the two headers' numbers; and the three structs. The binding (`EmuSen.Crystal.Client`) gives
  `CrystalRuntime`, `CrystalSession` and `CrystalSubscription`, which dispose their handles, `CrystalException` built
  from a status and `last_error`, copy-out helpers, and a `ChannelReader` over a subscription. It references `EmuSen`,
  for the C# implementation, and Galaxia, for the platform library's loader and `PadButton`/`PadAxis`.
- **The switch.** The contract layer fills its table once, by export name, from the platform library when
  `EMUSEN_CRYSTAL_NATIVE` is `1` and the library has every export of major 1, and otherwise from the C# implementation,
  which answers export names as a library does (`ManagedCrystal.Export(name)`). This is `EmuSen_RustPlatform.md` §3.9's
  state 1 (the C# the default, the library asked for), since the library has no export to offer before B7; asked for
  and absent, the fallback says so once in the log. State 2 is B7's.
- **The C# implementation, `EmuSen/Crystal/`** (namespace `EmuSen.Crystal.Managed`, in the `EmuSen` assembly because it
  makes engines through `CoreFactory`): `ManagedCrystal`, every export as an `[UnmanagedCallersOnly]` entry that catches
  what it throws and maps it to a status (`CrystalException`'s own, the service's `SessionException`'s, a core's
  refusal as `FAULT`, I/O as `EMUSEN_CRYSTAL_IO`); `ManagedRuntime`, the sessions by id and the current one, the
  systems, engines and routes, settings, firmware, save media and cards; `ManagedSession`, one session with its engine,
  its debugger's provider, speed, rewind, states and the frame-end order; `ManagedEvents`, the subscriptions.
- **Changes to the session service's C# provider**, Crystal's (`EmuSen.Crystal/Session/`): `SessionServiceOptions`
  gained the id a session is listed and answered by (`Id`, 1 by default, as every shell's provider keeps), a gate held
  around every change (`Gate`, an `IChangeGate`), and a run-control hook (`RunControl`) given the provider's own answer
  to call; `SessionService.Of(table)` answers the provider whose table it is, from a registry of the tables it made;
  and `DebugTarget` and `Options` are readable, for a shell given only the table (part 2). No function of the table
  changed shape, and `SessionHeaderTests` and `SessionServiceParityTests` pass unchanged.
- **Changes to the engine** so that a frame allocates nothing (§14.5): `CoreEngine` made its frame-end readers and
  writers once, gained `CopyPicture(Span<byte>)`, `DrainAudio(Span<short>, out rate)`, `BufferedAudioFrames`,
  `LoggedLines` and a virtual `Reset()` (MoonRT's shim's own reset overrides it); `CoreMachine.DrainEvents` fills a kept
  array and returns a span, `CopyFrame` and `DrainAudio` take spans, and `DrainLog` allocates nothing when the core logged
  nothing; `CoreDebugBridge.Armed` walks its registries by index.

### 14.2 The API as built, against §3's sketch

| §3.5's group | As built | Differs from the sketch, and why |
|---|---|---|
| Library | `abi_version`, `version_text`, `last_error(rt, …)`, `new(config)`, `free` | `last_error` takes the runtime, which may be null, because it is also the service's `last_error` twin |
| Discovery and systems | `systems`, `engines`, `cores`, `rescan`, `route(path)` | `core_approve` is not in 1.0: the C# discovery has no approvals to grant (trust is the sidecar's hash), so a function would be one the implementation cannot honour; it is a later minor with the trust manifest |
| Session life | `open(params)`, `close`, `reset`, `session_info` | the service's `info` twin answers exactly the provider's JSON, as SR4 asks, and the frontend's description is `session_info` of its own; `reset` is the console's button, Mistress's reload stays a frontend's choice |
| Media | `disc_insert`, `disc_eject`, `disc_change`, `disc_lid` | named for discs, so that `save_media` and `cards_op` are not read as the same group |
| The frame | `frame`, `picture_copy`, `audio_drain(…, rate)`, `wait`, `phase_names`, `phases` | the struct is `emusen_crystal_frame_info`, since C refuses a type and a function of one name (the core ABI's rule of `EmuSen_CoreAPI.md` §5.1); it gained `frame_interval_ns` (the speed's one owner hands the pacer its interval), `paused`, `halted_processor` and `halted_address`; the phases are names once as JSON and values per frame into a buffer, so a status line costs no JSON; `picture_raw` and the device picture are 1.1's, with Mistress (§14.7) |
| Run control | the twins `run_control`, `halt_state` | `RUN_PAUSE` is served (D10) |
| Input | `input_button`, `input_axis`, `input_connected` | two calls, because the core ABI has two enumerations, `emusen_control` and `emusen_axis` |
| Speed | `speed_set(percent, turbo, slow, flags)`, `speed_mode(mode)` | the held turbo or slow motion is a mode, so its policy is the runtime's; `EMUSEN_CRYSTAL_SPEED_DRAW_NONE` is §2.2's "frames not drawn", Pharaoh's `ff` |
| Rewind | `rewind_set`, `rewind_step`, `rewind_reel`, `rewind_picture(frame)`, `rewind_to(frame)` | a moment is named by its frame, as the reel lists it |
| States | twins `state_save`, `state_load`, `state_default_path`; `state_slots`, `state_slot_save`, `state_slot_load`, `resume_save`, `resume_load` | `state_record` and `states.db` (D8) are 1.1's, with Mistress, so that the copy of `records.db`'s rows is made once, when its last writer moves |
| Settings | `settings_schema`, `settings_get`, `settings_set` with `SCOPE_CONSOLE` and `SCOPE_GAME` | scope 1 is kept for an engine's own layer, which has no storage today |
| Cheats | the twins `cheat_op`, `cheat_codecs` | — |
| Save media | `save_media`, `cards_op` with nine ops | the ops are `SaveMediaManager`'s |
| Firmware | `firmware_needed`, `firmware_install(request, source path)`, `firmware_overview` | `firmware_find` folded into the overview |
| Debugger | `session_service(rt, session, flags)` and the twins | the session is named, since a session's table serves it |
| Events | `subscribe`, `subscription_drain`, `subscription_wait`, `unsubscribe` | renamed from `events_*` because `events_drain` is the service's twin; kinds `session`, `run`, `state`, `rewind`, `settings`, `disc`, `log` |
| — | `record_start`, `record_stop` | the frame recorder is backend (§2.2) and Pharaoh's `record` verb needs it |

**Statuses.** The service's −1100 to −1108 under their names; then `EMUSEN_CRYSTAL_NO_ENGINE` −1109, `_FIRMWARE`
−1110, `_BUSY` −1111 (a frame called while another thread is inside one), `_BAD_STRUCT` −1112 and `_IO` −1113.

**The naming rule found by building it.** Types are `emusen_crystal_<noun>` where no export is
`emusen_crystal_<noun>`: the first draft named the frame's struct `emusen_crystal_frame`, and a C compiler refused it,
the core ABI's lesson again.

### 14.3 Threads, run control and the pull rule

A session has a gate: a lock held by a frame from start to end and by every change, entered by a change from another
thread after the frame in progress and before the next, which waits for any change already waiting (a count of
waiters the frame thread spins on before it takes the lock). So a change from another thread lands at a frame
boundary and its caller waits, and a change on the frame thread between frames, the lock being its own and reentrant,
applies at once: §3.3's rule with no queue of closures. A change from another thread wakes a frame thread parked in
`emusen_crystal_wait`. The gate is handed to the session's provider (`SessionServiceOptions.Gate`), so a module's change
through the session's table lands at a boundary too. Reads take no gate, except `picture_copy`, which waits for the
frame in progress so that the picture is whole (C3).

The runtime calls nothing of the client's: no callback crosses the API, events are drained, and the two blocking calls
are waits with a timeout. Run control: `PAUSE` holds the session, `RESUME` lets it go and lets a halt go; a step arms
the provider's step and lets one frame call run though the session is paused or halted; a run-to arms its breakpoint and
lets the session run. A halt is a frame that did not complete: `ran` 0, `halted` 1, the debugger's snapshots refreshed,
and nothing advances until run control resumes it.

### 14.4 The frame-end order, written once

From Mistress's `EmulationLoop`, Hotaru's `GameWindow` loop and Pharaoh's `FrameRunner`; `ManagedSession.Advance`:

1. Rendering decided by speed: drawn unless `DRAW_NONE` or the speed's frame skip says not, and always while recording.
2. The engine's frame: the input held sent as it starts, the cheats' patch table refreshed, the machine advanced, and the
   engine's own frame-end work (frame logs, cheat pokes, freezes, the battery's period), then `present`.
3. A halt stops here: snapshots refreshed, a `run` event `halted`, `ran` 0.
4. The debugger's snapshots refreshed (`RefreshProviders`).
5. Rewind's capture, where rewind is on **and the engine allows it** (`EngineFeatures.RewindCapture`), with the
   picture attached when the reel keeps pictures.
6. The recorder's picture, when recording.
7. The core's new log lines as a `log` event.

What stays the client's is what reads a display or a device: when to call `frame`, the hand-off hold, the audio device
and its ratio, and a `state` event's reset of rate control.

### 14.5 The frame call allocates nothing

Measured on the calling thread with `GC.GetAllocatedBytesForCurrentThread` over 1,000 frames of the v1 test core,
after 200 to warm (`ManagedCrystalTests.The_frame_call_allocates_nothing`):

| | Before this step | After part 1 | After the review's change |
|---|---:|---:|---:|
| `CoreEngine.RunFrame`, bytes a frame | 144 (an event array, two boxed enumerators in `Armed`, three delegates) | **0** | **0** |
| `picture_copy` into the client's buffer | (Hotaru's `GetFrameBufferRgba` with no return: a new picture a frame, C4) | **0** | **0** |
| `audio_drain` into the client's buffer | (`DequeueAudioSamples`: a new array a frame) | **0** | **0** |
| `frame` through the binding and the C# implementation | — | 944, all of it the debugger's snapshots | **0** |
| `RefreshProviders` alone | 952 | 952 | **0** |

Part 1 left the debugger's snapshots allocating: every loop publishes, each frame, a register list, a palette list
and a hardware-load list, and part 1 proposed taking them only while a debugger reads. The tester declined that on
2026-10-17, as a change of what a reader on another thread sees, and decided instead that the snapshots go into a
preallocated, reused ring buffer with no change to what a debugger sees. That is what was built.

**The mechanism.** `SnapshotRing<TRecord>` (`EmuSen.Crystal/Telemetry/SnapshotRing.cs`) holds four records made when
the target is made. Once a frame the emulation thread fills the record after the newest in place, under that record's
own lock, and then publishes it as the newest. A reader on any thread clones the newest record under the same lock, and
the clone is shared by every reader until the next frame (`Newest`), so several providers read one frame's clone. The
writer can only meet a reader's lock by coming round the ring to the record that reader is still cloning, three frames
later. The frame side writes raw values only: register words, phase times, the bytes of a state or a space. Everything
that allocates (the register list with its names, the decoded palettes, the channel descriptions) is built on the
reader's side from the clone. A provider that fills a run of values of its own is a `RingProvider` (the generic
`CoreDebugTarget`'s registers, hardware load and palettes, and MarsRT's three register sets). A target whose providers
all read one record is a `SnapshotRing` with a `RingView` for each provider: MoonRT and MercuryRT save the machine's
state into the record once a frame (they had allocated a new state array a frame), with the few values not in the
state beside it: the channels' mutes, the frame's time, and for MercuryRT the registers its views read through the bus.

**The palettes.** A system pack's palette view decodes the memory spaces it is handed, and which spaces it reads is
the pack's business. `ViewSpaceCopy` (`EmuSen.Crystal/Debugger/ViewSpaceCopy.cs`) learns them once for each machine by
running the view over spaces that note each read. Each frame it copies those spaces into the record, and the reader
hands the view spaces over the copy. A space the view did not read when learnt is passed live. For a view whose reads
depend on the machine's state, a mode register for instance, that space would be read at the reader's time, not the
frame's. No pack in the tree does that today: the SNES pack's sixteen palettes over the copy match the live decode
(`VenusRtDebugViewTests.Palette_snapshots_match_the_live_decode_and_allocate_nothing`).

**Two allocations found by measuring, not by reading.** After the ring was in place, the generic target still cost
56 bytes a frame. A probe of each provider in turn found two C# closures: `FillPalettes` and the `DebugCpus` getter
each declared a local at method scope that a lambda captured. The compiler allocates the closure's object when the
method is entered, even on the paths that create no lambda. Moving the code that makes the lambdas into methods of
their own (`LearnPalettes`, `MakeCpus`) brought both to zero.

**Measured** on 2026-10-17, steady state after 200 frames to warm, over 1,000 refreshes. These are 0 bytes for:

- the generic target on the v1 test core and on VenusRT with the SNES pack;
- MoonRT, MercuryRT and MarsRT's own targets (`ManagedCrystalTests.The_shims_debugger_snapshots_allocate_nothing`).

The first refresh after a target is made still allocates once, when it builds its processor list. The work has moved,
not vanished: a reader now clones a frame's record once for every frame it reads, which for MoonRT and MercuryRT is the
size of a state. That cost falls on the debugger's thread, never on the frame.

### 14.6 The gates of part 1

| Gate | Result |
|---|---|
| `cargo test --release -p emusen-crystal` | 6 pass |
| Seeded faults against it, each reverted: two twin parameters' names swapped; a twin's `uint32_t id` widened to `uint64_t`; a twin dropped; two fields of `emusen_crystal_frame_info` swapped | caught by the twin test; by the twin test and the baseline; by both; by the layout test |
| The same widened parameter against the C# | caught by three of `CrystalHeaderTests`: the binding's table, the contract layer, the C# implementation's entries |
| `CrystalHeaderTests` (the header against the binding and the C# implementation: 132 exports, every number of both headers in both mirrors, the struct layouts by a C compiler against both C# mirrors, C++17) | 7 pass |
| `ContractShapeTests` (§9.3's shape check: the contract layer's public types are static classes or plain structs; every function, constant and field an integer, a double, a pointer to one or to a contract struct, or an unmanaged function pointer of such; no property, event or generic; the check seeing `string`, an array, a delegate, a list, a span and `bool`) | 3 pass |
| `ManagedCrystalTests` and `ManagedCrystalGroupTests` (the session's life and its refusals; the frame against the engine run beside it, picture for picture and sample for sample; allocation; input; speed; rewind; states in the player's folder; settings in layers reaching a running session; firmware, save media and cards; systems, engines and a route; events and their overflow; run control; a resume waking a parked frame thread; a breakpoint's halt; a twin and the table answering alike and the read-only table refusing; a frame struct longer and shorter than 1.0's; two threads' frames refused, not run together) | 19 pass |
| `DivergenceRowTests` (§14.6.1) and `FrontendDefectTests` (§14.6.2) | 11 pass, 1 skipped by design |

#### 14.6.1 The divergence rows, re-measured

`EmuSen_Surfaces.md` §4.1's table, re-read at `628bc821`; its line numbers and its `Hotaru/Program.cs` had drifted, so
each row is restated with today's places. One test per row, on the API, which every client that drives a core through
it shares; a client that has not moved still diverges until its move.

| Row | Today | On the API |
|---|---|---|
| 1. Rewind capture and the engine's opt-out | Mistress gated (`MainWindow.axaml.cs:1402`); Hotaru (`GameWindow.axaml.cs:451`) and Pharaoh (`FrameRunner.cs:96`) not. No engine withholds today, so the test attaches one that does | gated inside `frame` |
| 2. After a state load | Mistress's hotkey clears rewind and resets rate control, its console clears rewind alone; Hotaru's hotkey does both, its console neither (`HotaruSession.cs:58`); Pharaoh clears rewind and refreshes the snapshots | one sequence for every way of loading (the binding, the session's table as a shell's `state load` calls it, a slot, the resume state): rewind cleared, snapshots refreshed, a `state` event on which a client resets rate control |
| 3. What the shell asks of the host | Mistress handled `SwitchSession`, Hotaru five actions, Pharaoh none | run control's requests are `run` events any client drains |
| 4. A breakpoint's halt | Mistress never reads it; Hotaru and Pharaoh do | `frame` holds the session halted |
| 5. The hand-over | Mistress: rows, aspect, lines, serial; Hotaru: width and height | `emusen_crystal_frame_info` carries all of them to every client |
| 6. A picture's rows | Mistress's screenshots expand repeated rows, Hotaru's and Pharaoh's do not | the picture comes as shown unless the client opens with `SINGLE_ROWS`; a state's picture is always the shown one |
| 7. *New:* a console's stored settings | applied after the load by Mistress alone (`MainWindow.axaml.cs:498–508`); `CoreFactory` gives a v1 engine its console's values at create, but MarsRT's shim none, so emusen-shell and Pharaoh ran an N64 game on the shim's defaults whatever the player stored [read from the code] | `open` applies every setting the engine takes as the layers give it, game over console |

#### 14.6.2 F3, F4 and the candidates of §3.12

| # | Status at the end of part 1 |
|---|---|
| F3 | The API reads a game's engine from its own file over its console's (`FrontendDefectTests.F3_…`). The old divergence (Mistress's `games.db` choice, emusen-shell's `graphics.json`) ends when both open through the API and Mistress's `altemulator` is copied once into the game files (SR10), both with Mistress's move |
| F4 | **Shown failing**: with a state folder set, Hotaru's `HotaruSession.Build` names the state `Home/Saves/Save States/Kept.state` and the API `States/Kept.state`. The test is committed skipped with that record and passes once emusen-shell opens through the API (part 3). On the API every client is answered in the player's folder, read at each use |
| C1 | On the API: a `RESUME` from another thread wakes the frame thread parked in `wait` (`ManagedCrystalGroupTests`). Mistress's own failing case comes with its move |
| C2 | On the API: a breakpoint holds the session until run control resumes it. Mistress's own case with its move |
| C3 | On the API, `picture_copy` waits for the frame in progress and the engine's scratch frame is used only under the session's gate. Hotaru's feed window's race with its move |
| C4 | **Shown**: Hotaru's hand-over pattern (`GetFrameBufferRgba`, never returned) allocates a picture a frame, 100 frames at least 100 pictures' bytes; `picture_copy` into the client's buffer allocates nothing (`FrontendDefectTests.C4_…`) |
| C5 | On the API a cheat's change takes the gate, so it lands between frames and never beside frame-end work. Mistress's own case with its move |
| C6 | Not the API's (physical input is the frontend's, §2.3); with emusen-shell's move |
| C7 | On the API: one post-load sequence (row 2). Hotaru's console case with its move |

### 14.7 Left for later, and proposed

- **1.1, with Mistress's move**: the device picture (the open parameters' `emusen_device_lend`, the frame's
  `picture_on_device`, and the take the client makes; the take was decided the runtime's on 2026-10-18, §14.12), the raw picture, `state_record` and `states.db` with the one
  copy of `records.db`'s state rows (D8), and the copy of `games.db`'s engine choices into the game files (SR10). The
  split of `graphics.json` (SR12) comes before Mistress stops writing `Consoles`, so that the runtime's writes are not
  undone by a stale copy (R5); until then `settings_set` writes `graphics.json`, which no moved client and Mistress do
  not write at once in parts 1 and 2.
- **`core_approve`**, with the trust manifest. **The engine's own settings layer** (scope 1), with storage for it.
- **The debugger's snapshots.** Part 1 proposed taking them only while a debugger reads them. Decided 2026-10-17 by the
  tester: declined, and the snapshots go into a preallocated, reused ring instead (§14.5).
- **Where a game's settings file is named.** Part 1 named it `games/<the game's file name>.json`. Decided 2026-10-17
  by the tester, before 1.0 froze: **`/etc/EmuSen/games/<system>/<file stem>.json`**. `<system>` is the systems
  table's id, the one the saves' per-console folders go by, so two consoles' games of one name keep two files
  (`ManagedCrystalGroupTests.Two_consoles_games_of_one_name_keep_two_settings_files`). It is not `<game id>`, the name
  `EmuSen_CoreAPI.md` §8.5 had written, because a session runs with no library and so with no game id. The system is
  the one routing decides for the file (`ManagedRuntime.SystemOfFile`), which needs no library either. A file that runs
  as no system of the table has no file of its own: reads skip that layer and a write at the game's scope is refused
  with `NOT_SUPPORTED`.

### 14.8 Part 2: Pharaoh on the API

Pharaoh, first by §3.10's order, now drives its cores through the binding alone. Its project references
`EmuSen.Crystal.Client`, the shell and Galaxia, and no longer `EmuSen` or `EmuSen.Crystal`.

- **Every mode moved.** The classic loop and `--commands` run `FrameRunner` over a `CrystalSession`: held buttons are
  `input_button`, a frame is the frame call, the picture is `picture_copy` into a buffer kept between frames, the sound
  `audio_drain`; after a halt, the next frame asks run control to resume, as a script's next verb always resumed the open
  frame. The verbs map one to one: `savestate`/`loadstate` are the state twins (whose load is the one sequence of
  §14.6.1, row 2), `rewind` is `rewind_set`, `rewind_step` and `rewind_reel`, `ff` is `SPEED_DRAW_NONE`, `record` is
  `record_start` and `record_stop`, `perf` reads `phase_names` once and `phases` each frame, `vramsheet` and
  `paletteswatch` are the tile sheet's and the swatch's twins, `spriteoverlay` the sprites' twin, `audiodump`
  `audio_peek`, `tapuntil` and `waitchange` `space_read`. `--probe` reads the NES's spaces through `space_read` and the
  cartridge's header through `route`; `--testroms` opens each ROM with no battery, reads PRG RAM through `space_read` and
  resets through `reset`, a build with no NES engine an error and a board the engine refuses a skip, as before.
- **The shell is handed the session's table.** `DianaShInterpreter.CreateForService(table)` is the host API's new entry:
  the shell's Rust modules call the table, and its C# host commands reach the provider behind it (`SessionService.Of`)
  for its target, codecs and trace switch. A shell does not dispose a provider it was handed; the runtime does, with
  the session (`NativeShell`'s providers it made itself are the ones it ends).
- **What served Pharaoh alone moved into it**: `BmpFile`, `FrameHash`, `ContactSheet`, `SpriteOverlay` (now given a
  rectangle, not a debugger type) and `WavFile` to `EmuSen.Pharaoh/Imaging` and `Audio`, and `SqliteCatalogue` to
  `Reference`, as §4.1's "what stays C#" placed them; WiseMan's users follow the namespace. `PngFile` stays in `EmuSen`,
  which Mistress, Hotaru and the recorder use.
- **`--flag`** names are Pharaoh's own table, its one entry today the log switch, instead of reflection over
  `DebugSettings`.
- **The API gained, within 1.0**: `session_info`'s `"picture"` (width, height, row repeat, for a screenshot taken
  between frames) and `route`'s `"cartridge"` (the NES pack's reading of the iNES header: board, header trust, PRG and
  CHR bytes, battery), which the probe and the library indexer read in place of the shim's `Header`.
- **`--index-library`** reads the board from `route` and marks a NES image playable when it opens on the API, where it
  asked the MoonRT shim's board list before: the same answer for every board the shim lists, read through the engine
  instead of beside it, at the cost of one open a NES file.

**Behaviour that changes, each by the API's rule and recorded here:** a session's close writes the battery save (no
change under `--nobattery`, which the bench runs use); a game whose required firmware is missing is refused with
`EMUSEN_CRYSTAL_FIRMWARE` before it loads; the shell's `cheat` is given the catalog's cheat systems, as Mistress's and
Hotaru's are; a step of rewind drains and drops the sound the step skipped; recording draws every frame while it
records; and an N64 game runs on the console's stored settings (§14.6.1, row 7). `perf`'s wall time now includes the
frame-end order, which it did not.

| Gate | Result |
|---|---|
| `framesum 600` then `audiosum 600` with `--nobattery`, the old Pharaoh (`WiseMan` at `8b09514b`, Release) against the new, the same core libraries beside both | **equal on all seven**: Super Mario Bros. 3 (NES), Donkey Kong Country 2 (SNES), Tetris 2 (Game Boy), 10-Pin Bowling (Game Boy Color), Super Mario 64 (N64), Sonic the Hedgehog 2 (Mega Drive), Ape Escape (PlayStation, a `.cue`) |
| `FrontendReferenceTests.Pharaoh_names_no_runtime_type` (SR2): no `EmuSen` or `EmuSen.Crystal` among Pharaoh's referenced assemblies, and no type reference in its metadata resolving to either, so a type reached through a transitive reference fails too; held to seeing Hotaru's `ICore` | pass |
| `FrameRunnerTests` (on the API, the synthetic SNES ROM), `HeadlessDebugOptionsTests`, `PharaohDependencyTests`, `CpuThrottleTests`, `MoonTestRomTests`, `MoonPpuDefectTests`, the imaging and WAV tests, `CatalogueTests`, and every class of `EmuSen.WiseMan.Crystal`, with a shell handed a session's table on the Rust shell and on the C# one | 186 pass, 1 skipped (F4's record); `EmuSen.WiseMan.DianaSh` and `.Shell`, the golden corpus among them, 1,034 pass |

Each other frontend's old path stays until its own move: Hotaru, emusen-shell and Mistress are untouched.

### 14.9 Sizes against §7, and two predictions

| | §7's estimate | Built in parts 1 and 2 |
|---|---:|---:|
| Header and baseline | 1,100 | 697 (the header 496, the baseline 201) |
| Rust checks (no Rust runtime before B7) | — | 588 (`abi` and `lib` 118, the test 470) |
| C# contract layer and binding | 1,900 | 1,067, of which 561 generated from the header (the table, the functions, the numbers) |
| C# implementation | 2,500 | 3,271, of which 982 generated (the export map and the 78 twins' entries); 2,289 written |
| Changes to the provider and the engine | — | about 200 |
| Tests | 2,000 | about 1,300 of C# in 9 classes, and the Rust test above |
| Pharaoh | — | 759 lines in, 503 out over 17 files, the imaging, WAV and catalogue files moved whole |

**P1** (the C# implementation no larger than what it consolidates, about 3,600) holds so far by its written part,
2,289; it is retired by counting when the last frontend has moved and its frame-end code is deleted. **P2** (the frame
call under 2 µs a frame over `RunFrame`): measured on 2026-10-17 in Release on the v1 test core, the best of five rounds
of 20,000 frames, the frame call and the drain through the binding cost 1.793 µs a frame against 1.676 µs for
`RunFrame`, the debugger's refresh and the drain called on the engine directly: **0.117 µs** added, the function
pointers' two transitions and the gate. `pacebench`'s distribution on the bench games, P2's second half, is for the
moves of the windowed frontends, which pace; Pharaoh does not.

### 14.10 For the review

- **The API's shape** is §14.2's table and the header. Its first commit is stable by SR3, so a change after the review is
  a minor, not an edit: anything the review would rather see differently in 1.0 is best said before emusen-shell and
  Mistress are built on it.
- **Four choices made in building it** that the review may want to settle: the info twin kept verbatim and the
  frontend's description a function of its own (§14.2); the names `disc_*`, `subscription_*`, `state_slot_*` and the
  two input calls; the game's settings file named for the game's file (§14.7); and 1.1, not 1.0, for the device
  picture, the raw picture and the state records, so that each arrives tested with the frontend that needs it.
- **One proposal**: the debugger's snapshots only while a debugger reads them (§14.5).
- **The review's answer**, 2026-10-17: 1.0 approved, with two changes made before it froze. The game's settings
  file is `games/<system>/<file stem>.json`, and the debugger's snapshots go into a reused ring, the proposal declined
  (§14.5, §14.7).
- **What comes next**, after the review: emusen-shell with Hotaru's session and window (C3, C4's frontend half, C6,
  C7's console case, F4's skipped test made to pass, `HostAction` synthesised from events), then Mistress (1.1, C1, C2
  and C5's own cases, F3's copy from `games.db`, SR12), and the session service's 1.3 with `games`, `open_game` and
  `open_console` (D10).

### 14.11 Part 3: emusen-shell and Hotaru's game window on the API

**What moved.** Hotaru's session and its window now drive the core through `EmuSen.Crystal.Client` alone:

- **`HotaruSession`** opens the game, or an empty console, with the runtime's `open`. The engine is the one the
  settings' layers choose; the sound buffer is `audio.json`'s.
- **The window's loop** calls `frame` beside its own pacer. It copies the picture into buffers it owns
  (`PictureSlots`, `EmuSen_Serenity.md` §2.11) and drains the sound into a buffer of its own.
- **Input** goes through the API's input calls. Turbo is `speed_mode`; a held rewind is `rewind_step`.
- **F5, F9 and `state`** use the API's states, in the player's folder (F4). F6 is `record_start` and `record_stop`,
  F1 is `summary`, and the first watches are `watch_add`.
- **A breakpoint** leaves the session halted. The window's prompt runs until a resume, and leaving the prompt with a
  blank line or `feed` asks `run_control(RESUME)`.

**emusen-shell.** The program holds one runtime for the process. Its launch prompt and its shell window make their
shell with `CreateWithoutSession`, and `core` reads its vocabulary from the API's `systems` (below).

**Removed from the frontends:**

- their `RewindBuffer`, `SpeedController`, `FrameRecorder` and state path;
- every `ICore`, `CoreFactory`, `CoreCatalog`, `IDebugTarget` and `CoreDescriptor`.

Two helpers they used moved:

- `CategorizedLogWriter`, which three frontends use and which needs only the BCL, moved from `EmuSen.Common` to
  `EmuSen.Galaxia.Library`.
- Hotaru's `FrameImageWriter` encodes its PNG with SkiaSharp itself. The runtime keeps `PngFile` for its recorder,
  on the other side of the API.

`FrontendReferenceTests` now holds Hotaru and emusen-shell to SR2 as it holds Pharaoh. Its check that it sees a
violator reads Mistress, which is not yet moved.

**The surfaces this needed.** Under SR2, five members of DianaSh's and Serenity's C# surfaces could not be called,
because their signatures named a runtime type. The tester decided on 2026-10-17 to add members that name only
DianaSh's, Serenity's or the BCL's types, and to give SR2 no exemption. The additions:

- in DianaSh, `HostCommand`, `CoreChoice`, `CreateForService` with sessions, `CreateWithoutSession` and
  `CoretopCommand(Action)` (`EmuSen_DianaSh.md` §13.14);
- in Serenity, `PictureSlots` and `CoretopWindow.Over(ISessionReadout)` (`EmuSen_Serenity.md` §2.11).

Both are recorded in `EmuSen_Surfaces.md` §6.2. The old signatures that took `CoreDescriptor` or
`Action<IDebugTarget>` had no caller left and were removed.

The coretop window reads through the frontend's binding, not a service table reached with `SessionService.Of`, because
a table's pointer outlives a swap. Decided 2026-10-17 by the tester (`EmuSen_Serenity.md` §2.11).

**`HostAction` from events** (`EmuSen_DianaSh.md` Q5). Over a session's table, `resume` and `step` reach the session's
run control on both shells: the Rust shell's modules call it, and the C# facade asks it for its commands. The facade
then reports the action because the session's run events raised it (`EmuSen_DianaSh.md` §13.14,
`ShellOverSessionTests`). The window's loop does not pass the session along: the session itself holds or runs.

**The `core` vocabulary, 1.1.** The runtime alone knows the names `core <name>` accepts:

- the catalog's codenames and console names;
- each discovered console's and engine's name;
- each core's display name and extensions.

The tester decided on 2026-10-17 that these ride along on each `systems` entry as an additive 1.1 field, `core`,
filled from the registry. They are the engine's facts, not the pack's (§5.3), so they stay out of `emusen-systems`'
data files. The decision named `names` and `display_name`. The field also carries `console` and `extensions`, accepted by the
tester the same day, because
`core`'s refusals print them ("no core in this build starts the …", "… - expected: .smc, .sfc"), and Mercury's one
core spans two systems whose extensions differ.

`HotaruOnTheApiTests.The_core_vocabulary_read_from_the_API_is_the_registry_s` holds every name of the registry equal
on the API in all three facts. The golden corpus holds the words.

A system that only an engine's info names, with no pack, now gets a `core` entry of its id and its engines' names.
This lets `core test …` swap the test core's games, which the registry never offered. The version constant moves to
1.1 with part 4's exports, in the same unmerged step.

**Behaviour that changed:**

- In a running game, `core <name> --console` opens the console as a new session. It no longer starts it in place only
  when the running engine is the console's.
- A launch of an empty console is said as "the <console> with nothing in it".

**The gates of part 3:**

| Gate | Result |
|---|---|
| F4 | The test committed skipped in part 1 now runs and passes. With a state folder set, Hotaru's state is `<folder>/Kept.state`, the API's path for every client. |
| C3 | **Not shown failing.** The race is in the code: `CoreEngine.GetFrameBufferRgba` writes `_frame` and `_shape` outside the machine's lock, from the UI thread's feed poll and the emulation thread at once. The test core's picture is one row of eight pixels, a single small copy, so no torn picture was produced to show it. On the API, the feed copies into an array of its own through `picture_copy`. A copy taken on another thread while frames run is always one whole picture (`C3_a_picture_copied_beside_the_frames_is_always_whole`). |
| C4, the frontend's half | The window's pictures are presented from at most four buffers in 100 frames, where Hotaru's hand-over made one a frame (part 1's test shows that one). |
| C6 | **Held by a test of the window itself**, run headless over the test core: after a `core` swap is typed, no session is opened or closed until the UI thread runs, and then one is opened and the old one closed. The pads are rebound in the same call. The old path's defect was read from the code (`GameWindow.axaml.cs:364–365` before this step), not shown by a test, since that window is gone. |
| C7, Hotaru's console case | **Shown failing** on the old wiring: `state load` through `StateCommand` over the engine left the rewind history from before the load. On the API, the same command clears it, and the window's events reset the audio rate control once. |
| SR2 | Hotaru and emusen-shell name no type of `EmuSen` or `EmuSen.Crystal`. |
| Shells | `ShellOverSessionTests` pass on the Rust shell and, with `EMUSEN_DIANASH_NATIVE=0`, on the C# shell. The golden corpus, the shell's tests, the Crystal classes and the Hotaru, Shell, Serenity and Mistress tests pass. |

**Left as it was:**

- The window still gives Endymion's `AudioPlayer.Submit` an array of exactly the samples drained, one a frame, as
  before. Endymion's C# rate control copies the samples whatever it is given.
- Passing a span would need an Endymion overload, and it would save nothing until that copy goes.

---

## 15. What was built, 2026-10-17: B5, the file services in Rust

*Measured on 2026-10-17 on branch `b5-services`, from `WiseMan` at `53117c71`. The step of §6's B5 row, built in the
three parts that row names, each graded and committed before the next: (a) the firmware store, (b) the card manager,
(c) the cheat files and the database. The decisions that bear on it: SR5 (one writer per settings file: none of the
three writes `appsettings.json`, and the firmware store reads `FirmwareDirectory` from the C#'s load and passes it
down), SR7 (the C# as failover, behind one switch, `EMUSEN_SERVICES_NATIVE`, in state 2 of `EmuSen_RustPlatform.md`
§3.9), SR9 (the HTTP client, §15.4), SR11 (JSON for every request; no new store) and D4 (the crates in `Platform/`,
exported from `emusen_platform`).*

**The switch.** `ServiceLibrary` in Crystal holds it: the library unless the variable is `0`; a library absent,
refused or without one of the services' exports leaves all three on the C#, said once. Each service keeps its C# as a
`Managed` twin (`FirmwareLibrary.Managed`, `SaveMediaManager.Managed`, `ChtFile.Managed`, `CheatDatabase.Managed`,
`CheatDatabasePruner.ManagedPlan` and `ManagedApply`, `CheatDatabaseInstaller.ManagedInstall`), which is the failover
and the parity classes' oracle; a call whose text cannot cross (a NUL in a path, half a surrogate pair) stays on the
C#, which refuses it as it always did. Every public type and signature the C# services had is unchanged, so their
callers, the frontend API's managed implementation among them, compile and behave as before.


### 14.12 Part 4a: the API's 1.1, and the settings the runtime now owns

Part 4 moves Mistress, and Mistress needs more from the API than 1.0 gives. Decided 2026-10-18 by the tester: the
work is committed in two pieces, 4a with the API's 1.1 and the Mistress pieces that SR12 forces at once, and 4b with
the move. This section is 4a.

**1.1's additions.** `EMUSEN_CRYSTAL_MINOR` is 1. The baseline gained 18 lines, each tagged `@1.1`. A grown struct
keeps its 1.0 line as the least size a callee accepts, the core ABI's rule (`EmuSen_CoreAPI.md` §4.2). The crate's
baseline test now applies that rule, and a field may only be appended at or past the recorded size. Each addition
and its owner:

| Addition | What it is | Owner |
|---|---|---|
| `open_params.device` (offset 64; 72 bytes, 64 still taken) | the presenter's Vulkan device, the platform library's `emusen_serenity_device` | the frontend gives it; the runtime lends it |
| `frame_info.picture_on_device` (offset 104; 112 bytes, 104 still taken) | the frame's picture was taken on the lent device | the runtime |
| `picture_lend` | the current machine's lend, for the presenter to borrow (`SlangLend.Borrow`) | the runtime makes and frees it |
| `raw_select`, `raw_copy`, `emusen_crystal_raw_info` | the picture before the core's palette, into the client's buffer | the core, through the runtime |
| `state_record`, `state_forget` | a state's record in `states.db` | the runtime |
| `STATE_REFUSED` (−1114) | a load refused: another console's record, a version outside `loads_from`, or the core's own refusal in its words | the runtime and the core |
| `game_setting_get`, `game_setting_set` | a game's own file of settings with no session, the library's view | the runtime |
| `cheat_op` with session 0 | the ten ops that need no game | the runtime's provider with no target |
| `systems[].core` | the names `core <name>` knows, the display name, console and extensions (part 3) | the engine's facts, from the registry |
| `session_info.state` | `{"format","version","loads_from"}` | machine info |

**The device lend is the runtime's** (decided 2026-10-18 by the tester). This supersedes §14.7's take by the client.
`open_params` carries the presenter's device. The runtime makes each machine's lend from it (`PlatformLend`) through
the platform library's own C interface (`emusen_serenity_lend_new`, `_take`, `_settle`, `_free`). The core takes on
the emulation thread inside `frame`, as `CoreMachine` took before, and the lend is settled before the machine is freed.
No callback reaches the client, so the pull rule holds, and the Rust Crystal can do the same.

Why a client's take would not do: a lend serves one machine, whose timeline `done` starts again at 1, and a machine
can be made inside a call that is not `frame`. A client that skipped a take, on a rewind step or an undrawn frame,
would leave the core waiting on `done`. Serenity gained `SlangLend.Borrow`, a lend read and not freed, and
`SlangVulkan.DeviceHandle`.

`ApiDeviceLendTests`, on the RX 6800, holds the next drawn picture taken and no frame call stalled after each of:
- the open;
- a reset, which keeps the machine and its lend;
- held rewind's steps;
- four undrawn frames;
- a session opened again, Mistress's power cycle, which gets a new lend.

An in-place reset was left out of the test when it was written: **resetting a Diamond machine running the triangle
PS-EXE killed the host process** on any path, lent or not. The host called a null `emusen_core_reset` for a core that
did not claim RESET. That was fixed in Diamond's own work on 2026-10-18. Diamond now claims RESET, and the reset is in the test. No v1 engine makes a machine again inside a state load of another
size, so "a state of another size" has no case among the v1 engines today. MarsRT's rebuild is its own shim's, and
MarsRT takes no lend.

**States: `states.db` and the refusals** (D8). The runtime keeps each state's record in `states.db` (`StateRecords`).
The record has records.db's columns less `source`, and is stamped with the state file's size and time. Its first open
copies every state row of Mistress's records.db, none over a row already there, once, and marks the copy in a table
of its own. `state_save` writes the record: the session's console, the running engine's name, the state version, the
runtime's build, the time, the file's name, and the game's hash and size, which are read on a worker as the session
opens.

`state_load` refuses with `STATE_REFUSED` a state whose record names another console. It refuses one of the same
engine whose version is outside machine info's `loads_from`, naming the range. Otherwise it gives the core's own
refusal, an `InvalidDataException` from the core's status, in its words.

The case that started this: Mistress compared the record's version with the running core's, so a PAL NES state
(version 7) offered to an NTSC game (version 6) was refused as "saved by a newer build". The NTSC machine's
`loads_from` is 3 to 7, so the runtime passes the state to MoonRT, which refuses it with "a state saved on another
console", as Moon_PAL.md §7 recommends:
- `StateRefusalTests` holds the API's answer.
- `MistressStateRefusalTests` drives Mistress's window, saving with one copy of a game and loading into the other. It
  was **shown failing** on today's Mistress on 2026-10-18 ("That state was saved by a newer build (0.9.1+…") and
  passes with 4b's move.

**SR12: consoles.json.** Each console's core settings are the runtime's own file, `/etc/EmuSen/consoles.json`
(`ConsolesConfig` in Galaxia, which owns the file's name). Its first load, with no file there, copies graphics.json's
`Consoles` map whole. It never copies over a file that exists. The whole map is copied because which keys are a core's
depends on the engines installed at the time; a key no engine reads is inert there.

The runtime's every reader moved to it: `CoreFactory`'s stored settings and media, `FirmwareOverview`,
`settings_get`, `settings_set` and the save media. graphics.json keeps Mistress's own keys: the shader, the picture's
shape, overscan and crop, and the flag that a card's import was offered.

**What SR12 forced in Mistress at once.** A commit in which Mistress wrote a file the runtime no longer reads would be
a broken commit. So:
- The graphics window sorts its rows. The keys the console's schema, engine row and card rows name are the core's:
  read from consoles.json and written through `settings_set`, which checks them against the schema and applies them
  to a running session. Every other key stays in graphics.json, and Reset forgets both.
- `ApplyConsoleSettings`, the crash log's settings line, the cards, the controller rows and the empty console's
  engine read the runtime's stored values (`CoreSettings.StoredFor`).
- The engine a game opens on is the runtime's route.
- The metadata editor's "Alternative emulator" is written to the game's own file through `game_setting_set` as well
  as to games.db, where ES-DE's field lives.

Mistress reads consoles.json through Galaxia's model and never writes it: one writer, SR5. Mistress takes a
reference to the binding for this. The rest of its move is 4b.

**SR10's copy.** The runtime's first start copies each game's `altemulator` from games.db to its file of settings as
`Engine`. It skips an engine the console's schema does not offer, never writes over a value already there, and does
this once, marked by `games/.engine-choices-copied`. The library's badge reads the runtime's answer in 4b.

**A reset the core does not claim.** Since the host refuses a reset a core does not claim (merged 2026-10-18: the host
had called a null `emusen_core_reset`, which killed the process on Diamond's), the API's `reset` answers that refusal
as `NOT_SUPPORTED` with its words, a status like any other, and the session runs on (`StateRefusalTests`, on
MercuryRT). Mistress's Reset pressing the console's own reset where the core claims it is 4b's.

**The cheat database with no game.** The ten game-free ops that `cheat_op` answers on session 0 are named once, as
`SessionService.GameFreeCheatOps`. The provider answers them with no target, and the runtime asks the same set
before it refuses any other op with `NO_SESSION`. `emusen_session.h`'s `cheat_op` says so.

**Gates of 4a:**
- `ManagedCrystal11Tests` (eight):
  - consoles.json's one copy;
  - a record written, read and forgotten;
  - D8's one copy;
  - another console's state refused by its record;
  - a game's setting with no session;
  - SR10's one copy;
  - the raw picture;
  - every op on session 0, the refused ones each.
- `StateRefusalTests`, `ApiDeviceLendTests`, the header and shape tests at 1.1, and the crate's baseline with 18
  lines at `@1.1`.
- The Crystal, Mistress, Diamond, Nephrite and settings tests: the fixtures that wrote a core's key to graphics.json
  now write consoles.json.

### 15.1 Part (a): the firmware store, `emusen-firmware`

- **The crate** (`Platform/firmware/`) is `FirmwareLibrary` and the facts of `FirmwareOverview`'s rows: the folders'
  order (the player's `FirmwareDirectory` when its value is usable, then `home/Firmware`, the same folder searched
  once), an image found whole by its canonical name and then its alternates, or as a split form taken whole from one
  folder, size the only test (`EmuSen_Firmware.md` §2.1 to §2.3); where an image is without reading it, and the first
  folder holding a file of its name and another size, which is what an overview row says; and the install into
  `home/Firmware` under the canonical name. The chosen folder is only read.
- **The folder setting's rules are Galaxia's**, so their Rust half went into Galaxia's crate as `setting_folders`
  (blank is unset, `~` and `~/` are the home, a relative path or a file is refused, a folder that does not exist is
  accepted and said), with `SettingFolders.Checked`'s words; `emusen-firmware` calls it. The C# `SettingFolders` is
  unchanged: the two halves are a deliberate pair until state 3, held by the parity class. The line said of a refused
  or absent value is said once and again only when the value changes, by each half's own memory of it; the library
  queues it for the host's diagnostics, which report it through `ConfigDiagnostics` as the C# did.
- **The overview's enumeration stays C#.** Which systems a build runs and which engine each will run are
  `CoreCatalog`'s and discovery's (B6, B7); for each item it names, `FirmwareOverview` asks `FirmwareLibrary.Locate`,
  which is the library's. The question before a launch (`MissingFrom`) and `IsInstalled` ask `Locate` too, which opens
  and measures each file and reads none; `Find`, which the cores' hand-over calls, reads the bytes.
- **The C layer** is four exports taking one JSON request each (`emusen_firmware_folders`, `_find`, `_locate`,
  `_install`, `EmuSen_RustPlatform.md` §35); `_find` answers with an envelope carrying the parts' bytes, its first
  buffer the request's size, so a whole image costs one call and one read.
- **One defect, shown and fixed in both halves.** A firmware file given as a symbolic link was measured by the link's
  own length, the length of the path it holds, while the target's bytes were read: a linked image of the right size
  was passed over, and a link whose path happened to be as long as the image passed off a file of another size as the
  image, which §2.1 of the firmware page exists to prevent. The parity class found it on its first run (its generated
  trees hold links), and `FirmwareLibraryTests.An_image_given_as_a_link_is_measured_by_its_target` and
  `A_link_whose_own_length_is_the_size_does_not_pass_off_another_size` showed it, both failing on the C#. A file is now
  measured as reading it finds it, a link by its final target, and a link to nothing is neither the image nor a file
  of the wrong size.

| Gate | Result |
|---|---|
| `cargo test --release -p emusen-firmware -p emusen-galaxia` | 5 pass; 70 pass, 4 of them `setting_folders`' |
| `FirmwareParityTests` | 4 pass: 27 setting values of every kind, the folders and every line said compared; every request of 150 generated trees (three folders of right-sized, wrong-sized, empty, linked, dangling and unreadable files and folders named as files, each request with alternates and split forms of every size, including none and a negative one), found with its bytes and located alike; installs from right, short, unreadable and absent sources into new, nested and blocked stores; the public calls through the facade |
| The blast radius, with `EMUSEN_SERVICES_NATIVE` unset and `0` | 218 pass of 219, 218 of 219: the one failure each way is `VenusGoldenTests.No_corpus_verdict_is_worse_than_the_csharp_baseline`, a golden trace missing from the tree, which fails the same way on `WiseMan` without this step (187 traces for a baseline of 295) |

The blast radius was the parity class, `FirmwareLibraryTests`, `FirmwareFolderTests`, `FirmwareOverviewTests`,
`NephriteFirmwareTests`, `FirmwarePageTests`, `SettingFoldersTests`, `EsdeSettingsMenusTests`, the NEC DSP's firmware
classes, `DiamondLaunchTimeTests`, `VenusRtCoreAbiTests`, `VenusGoldenTests`, and, for the interface's number,
`GalaxiaParityTests`, `EndymionParityTests`, `SystemTableParityTests`, `SessionHeaderTests`, `NativeSwitchTests` and
`LeafAssemblyTests`.

**Seeded faults**, each made, built, run and reverted:

| Fault | Caught by |
|---|---|
| the alternates searched before the canonical name | the parity class (generated trees) |
| a split form held to the request's size when it gives none | the parity class (generated trees) |
| folders compared with a trailing separator, so `home/Firmware/` was searched twice | the parity class (the settings) |
| the wrong-size scan over every name rather than the canonical one | the parity class (generated trees) |
| a refused value reported every time it is read | the parity class (the settings' lines) |
| the facade reading every part from the envelope's first | the parity class (generated trees and the public calls) |

### 15.2 Part (b): the card manager, `emusen-savemedia`, and Galaxia's medium lock

- **The crate** (`Platform/savemedia/`) is `SaveMediaManager` (`EmuSen_SaveMedia.md` §6): a card opened and listed,
  a save copied, moved (copied, read back, then deleted), deleted and imported all or none, and a card made,
  duplicated and renamed, each change one atomic write (Galaxia's `atomic`) under the manager's own lock, every
  outcome in the C#'s words. It reads the media through B3's format modules in `emusen-systems` and their words from
  the systems table. Which medium a file is, by its extension, is not the pack's (F28) and stays the C#'s: the host
  names each file's medium in the request.
- **The lock moved into the library whole.** `MediumLock` keeps a process-wide table of who holds which card, which
  is how a refusal names the game; a card manager in Rust that could not read the C#'s table would have said "another
  program" of the game in the same window. So the lock gained a Rust half in Galaxia's crate, `medium_lock`: the same
  `flock` on `<file>.lock` that .NET's `FileShare.None` takes on Unix (measured: a second exclusive open is refused,
  and Python's `flock` from another process is refused with `EAGAIN`), the holder's words and the process id written
  into it, the file removed with the last handle, shared handles for the program's own machines. The C# `MediumLock`
  forwards to it under **Galaxia's** switch, since the class is Galaxia's; its exports are among `GalaxiaNative`'s.
  The games' placement (`MemoryCards`, B7) and the manager therefore hold one table in the process, and a lock taken
  by either half refuses the other as another program's would. The manager's Rust is used where the services' switch,
  the packs' code (`EMUSEN_SYSTEMS_NATIVE`) and Galaxia's lock all chose the library.
- **One behaviour reproduced on purpose**: a rename onto a folder of the new name is refused with the `IOException`
  .NET's `File.Move` throws, which links and then unlinks, and so meets an existing name of any kind. Found by the
  parity class's first run (the Rust had renamed over the empty folder).
- **Measured and kept**: .NET's refusal of an exclusive open does not remove the holder's lock file, so a third
  program still finds it.

| Gate | Result |
|---|---|
| `cargo test --release -p emusen-savemedia -p emusen-galaxia` | 3 pass; 72 pass, 2 of them `medium_lock`'s |
| `SaveMediaManagerParityTests` | 4 pass: 60 generated trees of every medium's cards (random cards, paks and backup RAMs from B3's generators, empty, blank and all-zero files, a file that will not open, a folder named as a card, a file of no medium, cards in a folder not yet made), each a script of 30 operations run on a copy by each half, about half of them with a card held by a game of the program, shared or not, or by another program's lock, every outcome's words and every file compared after every operation (of the 1,800, 174 changed a card, 1,403 were refused, 216 of those because a card was held, and the rest opened a card or threw alike); an import that exactly fills the card and one a block over; what a card that will not read reports; the two locks' sequences and their refusing each other |
| The blast radius, with `EMUSEN_SERVICES_NATIVE` unset and `0` | 238 pass, 238 pass |
| The lock's consumers with `EMUSEN_GALAXIA_NATIVE=0` | 133 pass |

The blast radius was the parity class and part (a)'s, `SaveMediaParityTests`, `SaveMediaManagerTests`,
`SaveMediaTests`, `SegaCdBramFormatTests`, `N64PakFormatTests`, `DiamondCardsTests`, `DiamondShellTests`,
`MemoryCardsOfferTests`, `NephriteMediaTests`, the Memory Cards window's fit audit, `GoldenCorpusTests` (its `memcard`
file unchanged), `GalaxiaParityTests`, `SystemTableParityTests` and `LeafAssemblyTests`.

**Seeded faults**, each made, built, run and reverted:

| Fault | Caught by |
|---|---|
| an import refused when its saves exactly fill the card | not by the scripts, which never met the case; then by the case written for it |
| a medium's words with their acronyms lower-cased | the parity class (scripts) |
| a save's place counted from 0 | the parity class (scripts) |
| the holder not asked before the manager's lock is taken | the parity class (scripts) |
| a move deleting without its read-back | none: an equivalent mutant here, since the copy the read-back checks was just written under the lock; recorded |
| a move that does not ask who holds the first card | the parity class (scripts) |
| a shared take joining an unshared holder | the parity class (the locks) |
| the facade dropping a save's damaged flag | the parity class (scripts) |
| a rename onto a folder not refused first | the parity class (scripts) |

### 15.3 Part (c): the cheat files and the database, `emusen-cheats`

- **The crate** (`Platform/cheats/`) is `ChtFile` (`.cht` read and written; a code decoded by emusen-systems' codecs
  named by id, the whole code first and then each `+` part, a patch told apart as `cheat add` tells it), `CheatDatabase`
  (the tree indexed as .NET's `Directory.EnumerateFiles(..., AllDirectories)` gives it: a folder's files in the
  system's order, its folders visited after in turn, links to folders followed, any error leaving the index empty;
  then sorted by system and game without regard to case, ties in that order; searched by game), `CheatDatabasePruner`
  (the plan with its four refusals, each folder measured as `FileInfo.Length` measures a link, by the link; the apply
  inside the tree only, never through a link, and `Directory.Delete`'s rule of trying every entry and giving the first
  failure after) and `CheatDatabaseInstaller.Install` (each entry written as `ExtractToFile` writes it: its Unix mode on
  a file it makes and not on one it overwrites, an existing file truncated only once locked, its time read as local
  time with the standard offset where the time falls twice or never, all measured on .NET 10.0.112 on 2026-10-17; no
  CRC check, as .NET makes none).
- **What stays C#.** The registry a session applies (`CheatRegistry`, B7) and so `CheatImport`, which reads a file with
  `ChtFile.Parse` and adds what it read; the download (§15.4). **The list files** (`cheats/<game>.json`) were Galaxia's
  in Rust already (`EmuSen_RustPlatform.md` §20), so nothing of theirs moved.
- **The archive.** A zip is read here only where it is plainly one of stored and deflated entries; a Zip64 or spanned
  archive, an encrypted entry, another method, a name holding a NUL or a directory that is not one is left to the C#,
  which reads every form .NET does and refuses the rest as before, nothing having been written. Inflate is `flate2`
  over `zlib-rs`, already in the workspace through `emusen-disc`'s CHD reader; `libc` gives `mktime` and `localtime_r`.
  On Windows and macOS, paused (`EmuSen_RustPlatform.md` §31), the local time waits for the port and reads as UTC.
- **A database is a handle** in the C layer, scanned once and never changed, so any thread may ask it; the C# index
  holds it for its life and is freed with it. `Games` and `Find` cross as places in the entries' list, so a system of
  thousands of files is not sent twice.
- **Two defects, shown and fixed in both halves.** A `.cht` file whose count is far past its entries
  (`cheats = 2147483647`) walked every number below the count, reading nothing for each, which held `cheat db load`
  and Mistress's import for minutes; shown by `ChtFileTests.A_count_far_past_the_entries_reads_them_without_walking_every_number`
  failing its five-second bound on the C#. Now only the numbers some field is named for are visited, and the rest are
  counted as skipped, which is what the walk counted. And an install into a folder written with a trailing separator,
  as a player may type `CheatDatabaseDirectory`, skipped every entry as one reaching outside the folder, since the
  folder's full path kept its separator and the test added another; shown by
  `CheatDatabaseTests.An_update_into_a_folder_written_with_a_trailing_separator_installs_every_cheat` failing on the C#
  (0 installed, 2 skipped). The folder's path is now taken without its trailing separator.
- **A rule of .NET's, reproduced where it was missing**: `int.TryParse` and `long.TryParse` accept NULs at the very end
  of a number, after any white space (measured: `"5\0"` and `"5 \0"` read as 5, `"5\0\0 "` and `"\05"` do not), which
  a `.cht` value may carry. `emusen-dotnet`'s reading did not; a failing test showed it, and the shell's reading, which
  shares it, gains the rule too, with its golden corpus unchanged.

| Gate | Result |
|---|---|
| `cargo test --release -p emusen-cheats -p emusen-dotnet -p emusen-platform` | 8, 27 and 29 pass, the platform's header test reading the three new C layers |
| `emusen_platform.h` as C99 and C++17, `-Wall -Wextra -pedantic -Werror` | clean |
| `CheatServicesParityTests` | 4 pass: every `.cht` text the cheat tests hold and 1,500 generated ones (entries in any order and case, codes from B3's corpus of the cheat tests' codes and generated ones, explicit writes with every field and malformed numbers, comments, quotes, CRLF, counts past and short of the entries), each read with every system's pair of codecs and with none, about 15,000 reads; 2,000 generated cheat lists written; 80 generated trees indexed, every system's games with five filters and twelve texts at four limits searched, planned against five keep-sets and pruned, the trees compared after; 300 generated archives installed, some over existing files, some damaged, some into a folder with a trailing separator, every file's bytes, mode and time compared |
| The blast radius, with `EMUSEN_SERVICES_NATIVE` unset and `0` | 535 pass and 8 skipped of 543, each way |

The blast radius was the three parity classes and the systems table's, `CheatCodecParityTests`, every cheat class
(`ChtFileTests`, the `CheatDatabase…` classes, `CheatPruneCommandTests`, `CheatChtCommandTests`,
`CheatMasterSwitchTests`, `CheatPersistenceCommandTests`, `GenesisCheatFormatsTests`, `PsxCheatFormatsTests`,
`MarsCheatTests`, `RomPatchResolutionTests`, `CoreFactoryCheatWiringTests`), Mistress's cheat windows and lists
(`CheatDatabaseWindowTests`, `CheatDatabasePruneWindowTests`, `ActiveCheatsWindowTests`, `GameCheatListTests`,
`MainWindowCheatListTests`, `PadCheatsTests`, `MarsRtEngineTests`, the cheat pad's fit audit, `AccessibilityTests`,
`SheetLookTests`, `SettingPathsOpenTests`), `SessionServiceParityTests`, and, for `emusen-dotnet`'s rule,
`GoldenCorpusTests` (its `cheat` file, `cheat db` cases included, unchanged), `TextModuleParityTests`,
`NativeSwitchTests` and `DianaShSwitchTests`; with `GalaxiaParityTests` and `LeafAssemblyTests`.

**Seeded faults**, each made, built, run and reverted:

| Fault | Caught by |
|---|---|
| a `.cht` flag read as true only in lower case | the parity class (texts) |
| the quotes kept on an empty quoted value | the parity class (texts) |
| a four-byte search size read as two | the parity class (texts) |
| a write's raw repeat count written | the parity class (written lists) |
| a game starting with the text ranked as one named by it | not at first, since a game named by the text sorts before one starting with it; caught once the trees held a game whose name the search shortens (`Mario.zip`) |
| a games filter not trimmed | the parity class (trees) |
| a linked system folder taken as inside the tree | the parity class (trees) |
| a link measured by its target | not at first, since the trees' only links were folders; caught once they held a linked file in a system folder |
| an entry's Unix mode not given to the file it makes | the parity class (archives) |
| every zip time read as standard time | the parity class (archives, through July and the two DST edges) |
| the target's trailing separator kept | the parity class (archives) |
| the facade reading every cheat as a RAM poke | the parity class (texts) |

### 15.4 SR9: the HTTP client, measured and chosen; the download still C#

SR9 chose one HTTP and TLS crate for the whole platform by a measured comparison of two candidates. The step measured
them on Linux on 2026-10-17, each in a scratch crate outside the repository built as the platform is (release, fat
LTO, one codegen unit, panic abort), downloading the libretro cheat archive (`CheatDatabaseInstaller.LibretroCheatsUrl`,
36,872,908 bytes) three times. Windows and macOS are paused (`EmuSen_RustPlatform.md` §31), so the comparison there
waits for the port, as §31 says of this gate.

| | `ureq` 3, `rustls` | `reqwest` 0.12, blocking, `rustls-tls` |
|---|---|---|
| Crates in its tree on Linux | 23 | 87 |
| Clean build of the tree | 20.4 s | 24.5 s |
| Added to a library | about 2.48 MB | about 3.34 MB |
| Download, three runs | 0.24–0.27 s | 0.25–0.27 s |
| Peak memory, the body held | 41 MB | 77 MB |
| Licences | MIT, Apache-2.0, ISC, BSD-3-Clause, and `webpki-roots`' CDLA-Permissive-2.0 | the same, with Unicode-3.0 and more |

For scale, `emusen_platform` is 4.74 MB today, and `curl` took the same 0.22–0.25 s; the network, not the client,
bounded every run. Both bundle Mozilla's root certificates (`webpki-roots`) unless told to read the system's.

Recommended to the tester: `ureq` with `rustls`, a quarter of the crates, three quarters of the size, half the memory
and no asynchronous runtime for what is one blocking download, at the same speed; and for the certificates, each
system's store (`rustls-platform-verifier`), which is what .NET's `HttpClient` does today, rather than Mozilla's roots
bundled, one more licence and a list the project would have to keep current.

*Decided 2026-10-18 by the tester, each as recommended:* `ureq` 3 with `rustls` is the platform's one HTTP and TLS
client (SR9, §11.1), and certificates are checked against each system's own store through `rustls-platform-verifier`,
not a bundled `webpki-roots`. Nothing was adopted in this step: the download (`CheatDatabaseInstaller.Fetch`) stays
C#, and `Cargo.lock` gained no HTTP crate. **Moving the cheat database's download to Rust on `ureq` is the next piece
of work after this step is merged**; the install then takes its bytes from the Rust download rather than from the C#.
*Built 2026-10-18, §16.*

*Also decided 2026-10-18 by the tester:* `MediumLock` stays backed by the library under Galaxia's own switch, as §15.2
built it.

### 15.5 Sizes, against §7, and what is left

| | §7's estimate | Built |
|---|---:|---:|
| C# replaced | 1,250 | about 1,020 (`FirmwareLibrary` 125 and the overview's rows, `SaveMediaManager` 181, `MediumLock` 106, `ChtFile` 270, `CheatDatabase` 118, the pruner 131, the install about 45; `CheatImport` and `CheatWrite` stay) |
| Rust, with in-crate tests and the C layers | 3,100 (2,500–3,750) | 2,497: `emusen-firmware` 337 with Galaxia's `setting_folders` 160 and its C layer 96; `emusen-savemedia` 466 with Galaxia's `medium_lock` 168 and its C layer 142; `emusen-cheats` 865 and its C layer 263 |
| C# facade | 300 | about 730 (`ServiceLibrary` 152, `FirmwareNative` 76, `SaveMediaNative` 80, `MediumLockNative` 47, `CheatsNative` 228, and the forwarding in the services) |
| The parity classes | — | 690 (175, 235 and 280) |

On the finished branch the three parts' blast radii together, less the missing Venus golden of §15.1, gave 699 pass
and 8 skipped of 707 with `EMUSEN_SERVICES_NATIVE` unset and the same with it `0`. Rebased on 2026-10-18 onto `WiseMan`
at `73bffab6` (B2's first three parts and MoonRT's PAL machine in), with the interface renumbered 16, the same run gave
the same counts both ways, and the three parity classes with Endymion's device and Serenity's parity classes 36 pass.

The Rust ran at about 2.4 times the C# it replaced, at §7's central ratio. The facade is more than twice §7's figure,
as B3's was, because each service has its own call shapes and the cheat database its handle; it is deleted whole at
state 3.

**Left for later:**

- State 3, the C# services deleted, at the tester's word after goldens (§6, B10). Until then a change to a service is
  made in both halves, and the parity classes fail on one alone.
- The download on `ureq`, next after this step (§15.4); the install then takes its bytes from it rather than from the C#.
  *Built 2026-10-18, §16.*
- The overview's enumeration of systems and engines, with B6 and B7; `CheatImport` with the registry at B7; the
  firmware store's reading of `FirmwareDirectory` from Rust once a Rust host loads `appsettings.json` itself.
- On Windows and macOS: the zip times' local zone, and every gate of this section, when the port resumes. The
  portable job of `EmuSen_RustPlatform.md` §31 compiles the three crates for both; it was not run here, since this
  machine's toolchain carries no standard library for either target.

## 16. What was built, 2026-10-18: the cheat database's download in Rust

*Measured on 2026-10-18 on branch `cheatdb-download`, from `WiseMan` at `c922c968`, with .NET 10.0.112. The piece of
work §15.4 named as next: the download (`CheatDatabaseInstaller.Fetch`) on the HTTP and TLS client SR9 chose, the
install it feeds being Rust already (§15.3). The decisions that bear on it: SR9 (`ureq` 3 over `rustls`, certificates
from each system's own store through `rustls-platform-verifier`, one client for the whole platform), SR7 (the C# as
failover behind `EMUSEN_SERVICES_NATIVE`), and `EmuSen_RustPlatform.md` §31 (Windows and macOS paused, every crate
still compiled for both).*

**What the C# did, read before anything was built.** `Fetch(url)` made an `HttpClient` of default settings with a
five-minute `Timeout` and called `GetByteArrayAsync`, holding the archive in memory for `ZipArchive`, which must seek.
Three things a reader might expect of a download it never had: **no progress** (each caller shows a fixed line until the
call returns), **no cancellation** (none of the three callers, `cheat db update`, the session's `DB_UPDATE` and
Mistress's Cheat Database window, can stop it), and **no file**: the archive is never written to disk, so there is no
partial file to leave behind and no atomic write to make; a failed download throws before `Install` runs, and nothing
is half-installed. Its request carries **no `User-Agent`**: .NET's `HttpClient` sends none unless told to, and EmuSen
never told it (measured: the request is `GET <path> HTTP/1.1`, `Host: <host:port>` and nothing else). Each of these is
kept: the Rust holds the archive in memory, sends no `User-Agent`, and the public `Fetch` neither reports progress nor
can be cancelled. Progress and cancellation exist in the Rust download for a host that drains them (§16.3); no C#
caller uses them, since the service's public shape is unchanged for B2.

### 16.1 The crate, `emusen-net`

The client is a crate of its own (`Platform/net/`), not a module of `emusen-cheats`, because scraping (B9) is its next
user and one client means one owner of its rules. `ureq` brings no TLS of its own here: `tls.rs` is a connector over
`rustls` whose verifier is the platform verifier, since `ureq`'s own rustls connector takes no verifier and a test's
extra root needs one. `ring` is the crypto provider, `ureq`'s default. What `HttpClient.GetByteArrayAsync` does with a
client of default settings, each rule measured against .NET 10.0.112 with a server of the measurement's own, and
reproduced:

- **The request**: `GET`, the path and query, `Host` with the port when one is written, nothing else; no
  `User-Agent`, no `Accept`, no `Accept-Encoding` (.NET decompresses nothing by default). One difference on the wire
  stays: `ureq` writes header names in lower case (`host:`), .NET `Host:`; HTTP header names are case-insensitive
  (RFC 9110 §5.1), and the tests compare names without case.
- **Redirects**: 300, 301, 302, 303, 307 and 308 with a `Location`, resolved against the request's URL as RFC 3986
  §5.2 resolves a reference (`location.rs`, held to the RFC's own table of examples); 50 at the most, after which the
  fifty-first answer is the answer (measured: 51 requests, then "301 (Moved Permanently)"); never from https to http,
  and never to a scheme other than these two, the redirect's own answer being then the answer. `ureq`'s redirects are
  off (`max_redirects(0)`): it follows https to http and fails on a missing `Location`, where .NET returns the answer.
- **One timeout over the whole call**: the clock starts when the call does and covers each redirect and the body; the
  remainder is each request's `ureq` timeout. A server that answers each hop in a second, with a timeout of 1.5
  seconds, fails, as .NET's does.
- **A request no byte answered is sent again three times**: a connection that ends after the request and before any
  byte of an answer is retried, four requests on four connections in all (measured on .NET; `ureq` never retries).
- **The body**: by its length, by chunks, or to the connection's close; a body that ends before its length or its last
  chunk fails, one that ends at the close is whole. A close without TLS's `close_notify` ends a body as a close does, in
  both (`ureq` reads an end of file, a reset or an abort as the close). The most a body may hold is .NET's
  `MaxResponseContentBufferSize`, `int.MaxValue`, refused at its length or as it grows past it.
- **Certificates**: the system's store through `rustls-platform-verifier` (on Linux the files `openssl-probe` finds,
  as .NET's OpenSSL reads them), never a bundled list; TLS 1.2 and 1.3. A test's own root is trusted beside the store
  only when the request names one (§16.3).
- **What `ureq` does not keep**, a status line as sent: .NET's words carry the server's reason phrase (a 404 sent as
  "404 Nope Nope" is "404 (Nope Nope)", one sent with none is "404."), and an answer that is not HTTP is named by its
  first line. `wire.rs` is a transport over each connection that keeps the first line after each request, and whether
  any byte came at all, which is also how an unanswered request is told from one whose answer broke. It is built on
  `ureq`'s `unversioned` transport interface, which may change between `ureq`'s minor versions; the version is pinned,
  and the parity class fails if a new one changes what the line gives.
- **A proxy** in the environment (`HTTPS_PROXY`, `ALL_PROXY`, `NO_PROXY`) is used by both; `ureq` reads it as .NET
  does for a CONNECT proxy, and neither half was tested through one.

### 16.2 The words for each failure

Every caller shows the exception's `Message` (`cheat db update: <message>`, Mistress's "Download failed: <message>",
the session's last error), so the Rust gives .NET's words and the facade throws the exception .NET throws. Measured:

| Failure | .NET throws | Message (inner) |
|---|---|---|
| a status not 2xx, a redirect not followed included | `HttpRequestException` | `Response status code does not indicate success: 404 (Not Found).`, or `...: 404.` with no reason sent |
| the timeout | `TaskCanceledException` | `The request was canceled due to the configured HttpClient.Timeout of 300 seconds elapsing.` (`TimeoutException`: `The operation was canceled.`) |
| a body cut before its length or last chunk | `HttpRequestException` | `Error while copying content to a stream.` (`HttpIOException`: `The response ended prematurely. (ResponseEnded)`) |
| no byte after the request, four times | `HttpRequestException` | `An error occurred while sending the request.` (the same `HttpIOException`) |
| a certificate refused | `HttpRequestException` | `The SSL connection could not be established, see inner exception.` (`AuthenticationException`, below) |
| a connection refused | `HttpRequestException` | `Connection refused (localhost:1)`, the system's words and the URL's host and port (`SocketException`) |
| a name not found | `HttpRequestException` | `Name or service not known (host:443)` (`SocketException`) |
| an answer that is not HTTP | `HttpRequestException` | `Received an invalid status line: 'SSH-2.0-OpenSSH'.` |
| a URL not absolute, or of another scheme | `InvalidOperationException`, `NotSupportedException` | .NET's two sentences, `The 'ftp' scheme is not supported.` |

The certificate's inner words are .NET's chain status, which rustls does not give: it says "unknown issuer" both of a
chain that ends at a root no store trusts and of one the server left open. The verifier is wrapped (`Chains`), and an
unknown issuer is told apart as .NET tells it: a chain that, followed from the leaf through what the server sent, ends
at a certificate that issued itself is `UntrustedRoot`; one that does not is `PartialChain`; and where the leaf is not
for the host's name as well, `The remote certificate is invalid according to the validation procedure:
RemoteCertificateNameMismatch, RemoteCertificateChainErrors`. A trusted chain for another name is
`...validation procedure: RemoteCertificateNameMismatch`, measured on .NET with the test's root given as the system's
store through `SSL_CERT_FILE`, since the parity class's oracle can only trust a test root through its own callback,
whose refusal has words of its own. Other refusals (an expired certificate is `NotTimeValid`) follow the same table;
anything rustls refuses that .NET has no status for is given in rustls's words. A cancel, which .NET's call has no
form of, is `OperationCanceledException`, "The operation was canceled.".

### 16.3 The C layer, the facade and the switch

A download is a handle running on a thread of the library's own (`emusen_net_download_start`), drained by the host:
`_progress` (where it stands, the body's bytes so far, its length once known), `_wait` (a wait with a timeout),
`_cancel` (it stands cancelled at once, whatever its socket is doing, and its fetch stops at its next read), `_body` and
`_failure` (a JSON object of the kind, the words, the inner words and the status), `_free`. No callback crosses
(`EmuSen_RustPlatform.md` §36, interface 17). The request's `test_roots`, a PEM text trusted beside the system's
store, is for a host's tests against a server of their own: no setting, variable or command reaches it, and
`CheatDatabaseInstaller.Fetch` never passes one; it is not a switch a player can flip.

`NativeDownload` in Crystal is the facade: `Fetch(url, timeout)` starts, waits and returns the body or throws the
failure's exception, and its instance methods give the progress and the cancel to a caller that wants them. The switch
is the file services' own: `CheatDatabaseInstaller.Fetch` takes the Rust where `ServiceLibrary.Active` and the URL can
cross, and `ManagedFetch`, the C# as it was, otherwise; the download's seven exports joined `ServiceLibrary.Exports`, so
a library without them leaves all the services on the C#. `FetchOverride`, the tests' hook, is unchanged and comes
first. `FetchTimeout` names the five minutes both halves are given. `ManagedFetch(url, timeout, testRoot)` is public, as
B5's other twins are, for the parity class.

### 16.4 The gates

| Gate | Result |
|---|---|
| `cargo test --release -p emusen-net -p emusen-platform` | 5 and 13 pass, `emusen-net`'s own and its tests against a server of their own; the platform's 31 and 4, its header test reading the new C layer; `emusen-cheats`' 8 unchanged |
| `emusen_platform.h` as C99 and C++17, `-Wall -Wextra -pedantic -Werror` | clean |
| `CheatDownloadParityTests`, against an HTTPS server on 127.0.0.1 made by the test, certificates made at test time, never the internet | 12 pass |
| The blast radius, with `EMUSEN_SERVICES_NATIVE` unset and `0` | 312 pass and 8 skipped of 320, each way |
| The golden corpus | in the blast radius; its `cheat` file, `cheat db` cases included, unchanged |

The parity class runs each case through the C# (`ManagedFetch` then `ManagedInstall`) and the Rust (`NativeDownload`
then `emusen_cheats_install`), and compares what each said (the exception's type and words and the inner exception's,
or the counts installed and skipped), every file each wrote with its bytes' hash, and the requests the server saw
(their number, the connections, and each head with header names in lower case). The cases: an archive whole, chunked
and after a relative redirect; fifty-one redirects and one from https to http; 404, 500, 503, a reason of the server's
own and none; a connection cut halfway through a body by its length, through a chunk and at a close, the last giving a
corrupt archive; a corrupt archive served whole; a server that stalls in the body, one that never answers and one whose
two hops each take a second, each against a timeout of 1.5 seconds; a self-signed certificate, one issued by a root
the store does not hold, and one for another name, each with the system's store alone; an answer that is not HTTP; a
request no byte answers; a refused connection. Progress and the cancel, which the C# has not, are held to the server's
bytes: progress drained to the body's length and its total, never going back, and a cancel during a stalled body
answered within 200 ms with nothing installed. The C# emits no progress events, so there were none to compare.

The blast radius was the parity class and B5's three (`CheatServicesParityTests`, `FirmwareParityTests`,
`SaveMediaManagerParityTests`, whose switch gained the download's exports), every `CheatDatabase…` class, Mistress's
Cheat Database window and its prune sheet, `SettingPathsOpenTests` and `SheetLookTests` (which swap or read
`CheatDatabaseInstaller`), `CheatPruneCommandTests`, `SessionServiceParityTests` (the session's `DB_UPDATE`),
`GoldenCorpusTests`, and for the interface's number `GalaxiaParityTests`, `EndymionParityTests`,
`SystemTableParityTests`, `SessionHeaderTests`, `NativeSwitchTests`, `DianaShSwitchTests` and `LeafAssemblyTests`.

**Seeded faults**, each made, built, run and reverted; the Rust ones against `emusen-net`'s own tests, the facade's
against the parity class:

| Fault | Caught by |
|---|---|
| a redirect from https to http followed | `a_redirect_from_https_to_http_is_not_followed` |
| ten redirects the most | `a_relative_redirect_is_followed_and_fifty_is_the_most` |
| an unanswered request not sent again | `a_request_no_byte_answers_is_sent_four_times` |
| the standard reason phrase in place of the one sent | `a_status_names_the_reason_the_server_sent` |
| the timeout restarted at each redirect | `the_timeout_covers_the_body_and_the_wait_for_an_answer` |
| the body's length never given to progress | the three tests that drain progress |
| a cancel that waits for the fetch to notice | not at first, since the test's server sent its body in pieces and the fetch noticed between two; caught once the server stalled in the body |
| a chain ending at a self-issued root not told from an open one | `a_certificate_the_system_does_not_trust_is_refused` |
| a name mismatch on an untrusted chain not told | `a_certificate_the_system_does_not_trust_is_refused` |
| the test's roots ignored | eleven tests |
| a 3xx taken as success | the two redirect tests |
| the status line kept with its carriage return | four tests |
| a close without `close_notify` read as an error | none: an equivalent mutant, since `ureq` reads the error a missing `close_notify` gives as the close, and the special case it would have removed was deleted as redundant |
| the facade throwing a timeout as `HttpRequestException` | the parity class (the slow and silent servers) |
| the facade throwing a refused certificate without its cause | the parity class (both certificate cases) |
| the facade giving the timeout in seconds where milliseconds are read | the parity class (three cases) |

### 16.5 Size, and the download by hand

The library built with the `dist` profile, as a publish builds it: 4,737,096 bytes at `c922c968` and 6,763,544 with
this step, **2,026,448 bytes more**, about 2.03 MB, against §15.4's 2.48 MB, which measured `ureq` with its default
features, Mozilla's bundled roots among them. The client's tree for Linux is 24 crates (§15.4 counted 23 with Mozilla's roots in
place of the system's); their licences,
MIT, Apache-2.0, ISC and BSD-3-Clause, all compatible with the GPL-3.0, are in `THIRD_PARTY_NOTICES.md` §1.4 with each
ISC and BSD text, and `ring`'s three, in `licenses/`. Every version is pinned exactly in `Platform/net/Cargo.toml`.

**The download by hand**, run once on 2026-10-18 on the desktop into `~/.cache/emusen/probe/cheatdb-live/`, not the
player's cheat folder: the libretro archive (`LibretroCheatsUrl`) fetched by `ManagedFetch` and then by
`NativeDownload`, each 36,872,908 bytes with the same SHA-256 (`7CCC166B…34DF8D5C`), byte for byte the same; 0.38 s
and 0.24 s. The Rust's progress, drained every 50 ms, gave its length from the first answer, never went back and
ended at the body's length. Each archive installed by its own half into its own folder gave 27,915 files installed
and 9 skipped, and the two trees the same, file for file and byte for byte.

### 16.6 Left for later, and for the tester

- **CI's portable job and `ring`** (`EmuSen_RustPlatform.md` §36). `ring`'s build script compiles C for its target,
  and the portable job's Ubuntu runner has no C toolchain or headers for `x86_64-pc-windows-msvc` or
  `aarch64-apple-darwin`; measured here, the check stops at `ring` for both. With the C compiler replaced by `true` for a
  local probe, every Rust crate checked clean for macOS; Windows could not be checked past `ring`. The job will fail on
  both targets once this is merged. The choices are the tester's: give the job each target's headers (an MSVC CRT and
  Windows SDK, as `xwin` fetches them, and a macOS SDK), leave the platform library out of the portable check until the
  ports resume, or take a pure-Rust crypto provider, which `rustls` does not recommend for production. Not worked around
  here.
- **On Windows and macOS** every gate of this section, when the ports resume (§31): the verifier there is the system's
  own (Schannel's and Security.framework's chain checks), whose words for a refused chain may differ from Linux's.
- **The status line's watcher** depends on `ureq`'s `unversioned` transport interface; a new `ureq` is taken only with
  the parity class run.
- **State 3**: the C# download is deleted with the other services' C# at the tester's word (§6, B10); until then a
  change to it is made in both halves.
- **Scraping (B9)** takes `emusen-net` as it is; the redirect, timeout and certificate rules are the platform's from now.
