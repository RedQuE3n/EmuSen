# EmuSen — the backend in Rust: the line, the frontend API, the system packs and the services

*This revision: the first, 2026-10-17. A design; B1, B3 and B5 have since been built (§12, §13, §15, 2026-10-17), B2's first three parts (§14, 2026-10-17), the cheat database's download on SR9's client (§16, 2026-10-18), B4's first phase for MercuryRT and for MarsRT, each an inventory and a proposal with nothing built (§17 and §18, 2026-10-18), and for MoonRT and VenusRT, VenusRT's shim retired (§19, 2026-10-18), with the decisions (§20) and MoonRT's steps that need no ABI change (§21, 2026-10-18), and B4's shared part with core ABI 1.13 (§22, 2026-10-18), and MoonRT's units, pack views and switch (§23, 2026-10-18), and debugger expressions naming the units' registers (§24, 2026-10-18). It answers the tester's goal of 2026-10-16, "to
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
  MarsRT's counters, which need a telemetry minor. Retired at B4. *Found not to hold for MarsRT on 2026-10-18 (§18.2):
  beside the counters, the lines of a picture at a multiple, a controller plugged in, the cheat gate and the TLB walk
  behind `addr` have no v1 home; §18.4 proposes them.*
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

A reset is in the test too. When 4a was written, resetting a Diamond machine running the triangle PS-EXE killed the host
process on any path, a null `emusen_core_reset` called for a core that claims no reset. It was reported for Diamond's
owner on 2026-10-18 and fixed in `b23cbff6`: the host refuses a reset a core does not claim, and Diamond claims RESET.
No v1 engine makes a machine again inside a state load of another size, so "a state of another size" has no case among
the v1 engines today. MarsRT's rebuild is its own shim's, and MarsRT takes no lend.

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


### 14.13 Part 4b: Mistress on the API

**What moved.** Mistress now drives every core through `EmuSen.Crystal.Client` alone. Its project names neither
`EmuSen` nor `EmuSen.Crystal`, and `FrontendReferenceTests.Mistress_names_no_runtime_type` holds it to SR2 as it holds
Pharaoh, Hotaru and emusen-shell. Before the move it named 72 types of the two assemblies, by the same metadata
reading. The check that sees a violator now reads DianaSh's C# shell, which is no frontend and names the debug target.

The pieces that replaced what Mistress named:

| Mistress before | Mistress now |
|---|---|
| `EmulatorSession`, `ICore`, `CoreFactory`, `CoreOptions` | `GameSession` over a `CrystalSession`, opened with `CrystalOpen` |
| `RewindBuffer`, `RewindThumbnail`, `SpeedController` | the session's `rewind_*` and `speed_*`; the reel reads `rewind_reel` and `rewind_picture` |
| its own lend (`SlangDeviceLend`) and `CorePictures` | `open_params.device` from `GameFrameControl.LendableDevice`, `picture_lend` borrowed with `SlangLend.Borrow`, `raw_copy` |
| the request queue and pause signal of the emulation thread | run control and `wait` (C1); a halt is the frame's `halted` (C2) |
| `CheatRegistry`, the codecs, `CheatImport`, `CheatDatabase*` | `CheatList` over `cheat_op`, the database's ops on session 0 or the running session |
| `MemoryCards`, `SaveMediaManager`, `MediaOutcome` | `cards_op` |
| `FirmwareOverview`, `FirmwareLibrary`, `EmulatorSession.MissingFirmwareFor` | `firmware_overview`, `firmware_needed`, `firmware_install` |
| `CoreSetting`, `ICoreSettings`, `ApplyConsoleSettings` | `settings_schema` and `settings_set`; the runtime applies a console's settings to its sessions |
| `IDebugTarget`, `ICheatCodeCodec`, `ICpuTraceSwitch` in the console window | `CreateForService` over the session's table, `CreateWithoutSession` with no game; the window's commands are `HostCommand`s |
| `DebugSettings.MasterLoggingEnabled` | the runtime's `log_switch` and `log_status` |
| `CoreCatalog`, `CoreDescriptor`, `SystemPacks`, `ControllerPorts` | `RuntimeCatalog`, a facade over `systems`, `shelves` and `route` |
| records.db's state rows, `Refusal` | states.db through `state_record` and `state_forget`; `state_load`'s refusal |

**The catalog's facts.** `RuntimeCatalog` reads `systems` and `shelves` once and keeps them until it is told to forget,
which a library refresh does, and asks `route` once for each version of a file (its path, size and time). Each fact and
where it now comes from:

| Fact | Source | Owner |
|---|---|---|
| The library's shelves: name, label, system, ES-DE names, and the core each belongs to | `shelves`, new in 1.1 | the systems table, each shelf a system of a console's core, named for the core |
| A file's console, shelf and engine | `route`; `shelf` new in 1.1 | routing |
| A console's pad, its most ports, each player's controller as its settings stand | `systems[].controllers`, new in 1.1 | the engine's facts, from core info, as `core` is |
| The engines a console offers a choice of | the schema's `Engine` row | the runtime's settings |
| A console's host, cheat folders, OpenVGDB names, cover shape, ScreenScraper id, first release | `systems` | the systems table |
| The extensions the scan and the file picker take | the shelves' cores' extensions | the systems table |
| A machine's television standard | `session_info.television_standard`, new in 1.1 | machine info |
| The bytes OpenVGDB hashed for a file | `emusen_systems_openvgdb`, the platform library's own C interface | the systems table's rule |

The OpenVGDB rule is the library's, not a session's or the runtime's: the cover lookup asks it of files no session runs.
The library is a component of its own at B8 (§2.4), which will read the systems table itself, so the rule is read from
the platform library's interface directly rather than added to the frontend API.

Three things are Mistress's own presentation, and stay in Mistress:
- **The order of the console filter**: every console, then the shelves in release order. Before the move it followed the
  catalog's registration order, which differed from the sidebar's for no reason a player could see.
- **When the engine's row is shown** in Graphics Settings: where the schema offers more than one engine.
- **The Speed menu's choices**: 100, 300, 25 and unthrottled, set through `speed_set`. The speed itself is the runtime's.

The name a console started empty goes by ("SNES (no disc)") moved from `EmuSen.Cores.ConsoleSession` to Galaxia's
`SaveLibrary.ConsoleSessionName`, beside the save library's other names; the runtime's `ConsoleSession` calls it.

**1.1's additions in this part**, each additive, none changing a struct:

| Addition | What it is | Owner |
|---|---|---|
| `shelves` | the library's shelves in release order | the systems table |
| `systems[].controllers` | `{"buttons","axes","ports","players"}` | the engine's facts, from core info |
| `route`'s `shelf` | the shelf a file sits on | routing |
| `session_info.television_standard` | "ntsc", "pal" or null; `session_info.state` is now documented in the header as well | machine info |
| `settings_schema` rows' `note` and `shown_with` | where the core words the value chosen, that sentence as the stored values stand; the row a row is shown with | the core's setting, a card's medium |
| `cards_op` LIST's `kind`, `capacity`, `unit`, `mode`, `mode_key`, `slots`, `next_name`, `import_due` | what the card window showed from `MemoryCards` | the card manager |
| `CHEAT_DB_STATUS`'s `attribution` | the database's licence words | the cheat database |

The crate's baseline gained one line, `emusen_crystal_shelves` at `@1.1`; the header test reads 140 exports.

**A defect in the binding, shown and fixed.** `CrystalSession.CheatOp` read every op with the copy-out protocol's two
calls. An op that changes the list and answers a value rather than a record, such as `IMPORT` or `ADD`, was therefore
made twice. Measured on 2026-10-18 with a probe on the C# implementation: one `IMPORT` of a list of one cheat through
`CheatOp` left two cheats. `CrystalSession.CheatChange` makes such an op with one call (one cheat), and
`ManagedCrystal11Tests.A_cheat_change_through_the_binding_is_made_once` holds it. No client had called a change through
the binding before this part. Record-answering changes, `DB_PRUNE` with "apply" and `DB_UPDATE`, were already safe: the
provider keeps their record between the two calls.

**Endymion's claims.** The button that starts a game from the library could claim player 1 for its pad, if the game's
first poll saw it still held. A quicker launch on the API made the race show every time
`MultiplayerTests.The_pad_pressed_first_in_a_game_is_player_1_whichever_connected_first` ran first in its process. The
same defect was fixed in Endymion on its own on 2026-10-18 (`385eb29d`, `EmuSen_Input.md` §8.13.8): a pad held as claims
begin claims nothing until let go. Mistress on the API takes that fix as it is.

**The loop.** The emulation thread calls `frame`, paces against the display from `frame_info`'s interval and speed, and
drains the sound the frame reports playable. Between frames it drains the session's events:
- a state loaded resets the audio rate control (C7);
- a run event updates the window's pause, resume and halt;
- a setting re-reads the ports;
- a disc event updates the drive's menu.

A picture taken on the device goes to the frame control as the borrowed lend. Otherwise it is copied into one of four
buffers that `FrameHandOff` owns and gets back from the control, so a frame allocates next to nothing
(`FrameHandOffTests.With_the_picture_copied_as_Mistress_copies_it_a_frame_allocates_next_to_nothing`). The raw picture
is selected and copied the same way when the screen's filter reads one.

Which way the picture reaches the screen is said once in the log, at a session's first picture (`[picture] <engine>:
on the device …` or `read back: …` with the reason), as proposed with the device lend.

The launch screen's load under the screen now opens the session on a worker for every engine, not only the v1 engines:
`open` may be called on any thread.

**States.** Save and Load State use the API's slots, and leaving a game writes the resume state; the runtime writes the
picture and the record. Mistress keeps the question about another copy of the game, which reads the record's hash, and
asks nothing else of the record. Its `Refusal`, which compared the record's state version with the running core's, is
gone: what may load is `state_load`'s answer. A refusal (`STATE_REFUSED`) is shown as `Load State: <the words>`.

`MistressStateRefusalTests` now runs. Its PAL and NTSC copies differ, so the window first asks about another copy, and
the test answers Load Anyway, which is what reaches the core: MoonRT refuses with "a state saved on another console" and
the status line says so, with no "newer build". The test as committed in 4a had no such answer, because Mistress's
`Refusal` came before the question. The behaviour it was written to show is unchanged.

**Reset** (decided 2026-10-18 by the tester). The menu's Reset, the HUD's Restart and the pad menu's Restart call
`ResetEmulation`. It presses the console's own reset through `reset` and keeps the session. A core that claims no reset
answers `NOT_SUPPORTED`, and only then is the game started again in a new session, its cheat list carried over:
- `MainWindowEmulationMenuTests.Reset_presses_the_console_s_reset_where_the_core_claims_one` (VenusRT: the same session
  and the same machine);
- `MainWindowEmulationMenuTests.Reset_starts_the_game_again_where_the_core_claims_no_reset` (MercuryRT: a new session,
  the list kept).

**The windows.**
- **Cheats.** `CheatList` is the running session's list over `cheat_op`, or with no game a list held in Mistress as a
  cheat file, which the next game starts with, as the window's registry did before. A typed code is decoded by
  `CHEAT_CLAIMS` and `CHEAT_DECODE` and added with `CHEAT_ADD`, so only the running game's tab takes one. Another console's
  tab, and every tab with no game, says that a game of it must run (decided 2026-10-18; see the proposal below). The
  database window lists, loads, prunes and downloads through the database's ops, on session 0 with no game. Leaving a
  game keeps its list in the game's file, as before.
- **Memory cards** read and change cards through `cards_op`; the offer of a shared card's import reads LIST's
  `import_due` and Mistress's own record that it asked, in graphics.json.
- **Firmware.** The firmware page reads `firmware_overview` and words it itself (`FirmwareRow`); the runtime's
  `FirmwareItem` keeps only the data. The prompt before a game uses `firmware_needed` and `firmware_install`.
- **Graphics Settings** builds a console's rows from its schema; a row's note is the schema's, read again after each
  change, and a card's slot rows are shown with the mode row they name.
- **Controller Bindings** takes each console's pad and players from `systems`.
- **The DianaSh console** is a shell over the session's table, or without a session, and its window commands (pause,
  resume, coretop, vstop, feed, clear) are `HostCommand`s.
- **The hardware dashboard** reads the session through `SessionReadout`, at each poll, so a new game needs no update.
- **Debug Logging** is the runtime's log switch.

`FrameImageWriter`, which encodes a picture as PNG with SkiaSharp, moved from Hotaru to Serenity
(`EmuSen.Serenity.Imaging`), since Mistress's screenshots now need it too; Hotaru dropped its own SkiaSharp reference.
`GameFrameControl.TryLend`, whose one caller was Mistress's own lend, became `LendableDevice`, the presenter's device for
the runtime to lend. Both are recorded in `EmuSen_Surfaces.md` §6.2.

**§3.12's candidates in Mistress.** Each was shown failing on the Mistress of before the move (`71c6f548`), with the test
adapted only in how it reads the old window's fields, and passes after it (`MistressRunControlTests`):

| # | Shown on the old Mistress | On the API |
|---|---|---|
| C1 | Failed: the `resume` typed in the console left the game paused | The window's resume is run control's; the paused frame thread parks in `wait` and drains the console's queue there |
| C2 | Failed: the breakpoint stopped the game, but the window still said `Running: Loop.sfc` | `frame` returns `halted` and the session advances nothing; the window says `[BREAKPOINT] Halted at $…`, and a resume lets it go |
| C5 | Failed: the poke landed while the game was paused, before any frame ended | the list is the session's and is applied in the frame-end order only, so Apply while paused pokes at the next frame |

**Behaviour that changed:**
- A cheat code can be typed only on the running game's tab.
- The console filter is in release order.
- The early load under the launch screen covers every engine.
- A state another build wrote is no longer refused by version in Mistress; the runtime and the core decide.
- Graphics Settings' SNES tab shows VenusRT's own setting, SA-1 catch-up, which the runtime's schema offers and applies.
  The window read the catalog's list before, which holds none for the SNES, so the tab said "No core settings yet"
  over a setting the runtime took. The six interface recordings of the tab were recorded again
  (`GraphicsSettings`, `GraphicsSettingsSnesEngine` and `GraphicsSettingsGameCropNote`, at both sizes); nothing else of
  the 518 audited states changed.

**Proposed, not built:**
- **Cheat codecs with no game.** `cheat_codecs`, `CHEAT_CLAIMS` and `CHEAT_DECODE` on session 0 with `{"console"}`, as an
  additive 1.1 extension of the ten game-free ops, would let a code be typed for a console before a game of it starts,
  as the window allowed before. It changes the decision that fixed the game-free ops at ten, so it is the tester's to
  make.
- **One session readout.** Mistress's `SessionReadout` and Hotaru's are twelve lines alike. The one place both could share
  is a toolkit-free assembly that knows both the binding and Serenity's `ISessionReadout`, which does not exist. The
  duplication is left until a third frontend wants it.

**Left as it was:**
- The library and scraping stay Mistress's until B8. `FileRecords` keeps records.db's shader-pack record; its state
  methods have no caller in Mistress now, and records.db's state rows are read once, by D8's copy.
- The sound is still handed to Endymion as an array drained each frame, as in part 3.
- A rewind snapshot taken on a frame the speed leaves undrawn keeps no picture, so a reel opened after fast-forwarding
  shows only the moments that were drawn. This was so before the move as well, where the reel's tests set the speed's
  frame skip to none; on the API the tests play at normal speed instead.

**The gates of part 4b:** Mistress's tests in three runs, measured on 2026-10-18: the window and library set 615 passed and 7 skipped; Big Picture and scraping 1,073 passed and 46 skipped; the window fit audit passed after the six recordings above. The blast radius beside them (the Crystal and binding tests, input and player claims, firmware, the device lend, screen filters, `FrameImageWriter`, the golden corpus, the shell over a session, core registration and the frontend reference test) 522 passed and 8 skipped. The ABI baseline's crate tests and the session service's header test pass.

### 14.14 The session service's 1.3

*Built 2026-10-18.* D10's three functions are in `emusen_session.h` 1.3, served by Crystal, and recorded in full in
`EmuSen_DianaSh.md` §13.15: `games` answers what `systems` answers, and `open_game` and `open_console` open a session
with the open options of the session the table serves, announce it with a session event and leave it to the client
that drives frames. Their twins are `emusen_crystal_open_game` and `emusen_crystal_open_console`, additive in 1.1;
`games`' twin is `systems` itself, so the twin check names that one pair rather than export the same JSON twice. The
frontend service of §3.11 keeps its two window operations and gains nothing.

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

### 15.6 Built 2026-10-18: the store recognises a file by its content

*Decided 2026-10-18 by the tester* (`EmuSen_Firmware.md` §7.9): the firmware store also recognises a player's file by
its MD5, whatever it is called, from checksums each core lists in its own info (core ABI 1.12, `EmuSen_CoreAPI.md`
§6.32). In this crate it is a third step of `search`, after the names and the split forms of each folder: `by_content`
lists the folder's own entries, keeps the files whose length, a link's by its target, is the request's size, takes
their MD5 by Galaxia's `rom_hash::md5`, and picks the first file of the first listed checksum, file names compared in
the order of their UTF-16 code units, which is the C#'s ordinal order. A file's MD5 is kept for the life of the
process by its path, length and modification time. The C# twin, `FirmwareLibrary.Managed`, takes the same step
(`ByContent`, `Md5Of`). `FirmwareFound` and `FirmwareLocated` gained `ByContent`, the C layer's requests `md5` and its
answers `content`, and the platform interface went from 17 to 18 (`EmuSen_RustPlatform.md` §38). The overview's
`FirmwareItem.FoundAs` and the session's line name the player's file; the frontend API's overview does not carry it yet
(`EmuSen_Firmware.md` §7.5).

The gates, seeded faults and measurements are `EmuSen_Firmware.md` §7.10.

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
  and the parity class fails if a new one changes what the line gives. *Decided 2026-10-18 by the tester: relying on
  the `unversioned` interface is accepted, `ureq` pinned and the parity class its guard.*
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
  both targets as it stood. *Decided 2026-10-18 by the tester:* Windows is checked as `x86_64-pc-windows-gnu` with
  MinGW-w64 on the Ubuntu runner, `x86_64-pc-windows-msvc` kept for the crates without C, and `aarch64-apple-darwin` is
  checked on the Mac mini (`EmuSen_RustPlatform.md` §29, §31, §36).
- **On Windows and macOS** every gate of this section, when the ports resume (§31): the verifier there is the system's
  own (Schannel's and Security.framework's chain checks), whose words for a refused chain may differ from Linux's.
- **The status line's watcher** depends on `ureq`'s `unversioned` transport interface, accepted by the tester on
  2026-10-18; a new `ureq` is taken only with the parity class run.
- **State 3**: the C# download is deleted with the other services' C# at the tester's word (§6, B10); until then a
  change to it is made in both halves.
- **Scraping (B9)** takes `emusen-net` as it is; the redirect, timeout and certificate rules are the platform's from now.

### 16.7 Two parity cases that failed once in a combined run (2026-10-18)

**What was seen.** In the combined run that merged B2's part 4b (818 tests of the Crystal, Mistress and golden
classes, after a build of the solution), `An_archive_installs_alike_whole_chunked_and_after_a_redirect` failed in 51 ms
and `Redirects_are_refused_alike_past_fifty_and_from_https_to_http` in 259 ms, each with one side reporting
`HttpRequestException: An error occurred while sending the request.` Both passed when run alone. The run's log was
kept only as far as that line, so which side threw, and the inner exception, are not known.

**What it was not.** Nothing runs beside the class: it is in the `ProcessGlobals` collection, whose definition disables
parallel running, so no other test runs while it does, though threads an earlier test left may. Not a timeout:
both failures came in well under the 20 s each fetch is given, and a longer one would change nothing. Not a server's
close racing a reused connection: with the server holding each connection open 300 ms after its reply, so that a
client reusing it meets a server about to close it, every case passed, because .NET sends a request again when a
reused connection closes before any answer, and `fetch.rs` retries an unanswered request as .NET does (§16.2).

**Measured on 2026-10-18 without a failure:** the same combined filter three times in a tree at `701dade7`, 823 of
823 each; and the class fifteen times with its test host held to two CPUs shared with two busy loops, 12 of 12 each.

**Left as it is, said better.** With no cause found there is no condition to wait on, and a sleep is no fix. A case
that differs now says why in full: the failure's message adds every exception an update threw, whole, with its inner
exceptions to the bottom, and what the test's server itself caught and swallowed, in accepting a connection or in
serving one. Neither enters the comparison, which still reads only what both sides report, so a passing case is
unchanged. The next failure will name its side and its cause.

## 17. B4's first phase for MercuryRT, 2026-10-18: the Game Boy's shim inventoried, and where each piece goes

*Read on 2026-10-18 on branch `b4-mercury`, from `WiseMan` at `480bf5e2`. Phase 1 of §6's B4 row for one core: an
inventory and a proposal, with nothing built. Decided 2026-10-18 by the tester: B4 may touch MercuryRT to retire its
shim, while the port's own open stages stay paused, so nothing proposed here changes what the machine emulates; it
moves what the shim adds above the core ABI v1 into MercuryRT's exports and info, into the Game Boy's packs, or into
the generic host, or it retires it. The MoonRT and VenusRT shims and the MarsRT shim are inventoried in sections of
their own written beside this one; the pieces of the host the three share are marked "shared" below, and the three
sections are expected to be renumbered when they are merged. Claims are marked as at the head of this page.*

### 17.1 The shim, measured

`EmuSen/Cores/Nintendo/MercuryRT - GB/Shim/` is four files and 914 lines [M: `wc -l`]: `MercuryRtCore.cs` (255),
the engine over `PortEngine`; `MercuryRtDebugTarget.cs` (544), the debugger's view, which reads MercuryRT's save state
through the listing `state_layout` prints (`Mercury_Native.md` §14.5, §15.2); `MercuryMachine.cs` (84), a machine of
the pre-stable interface; and `MercuryNative.cs` (31), which loads the library through both interfaces. §4.1 counted
1,114 lines at `165e60b4`; B3 has since moved `GbCartridge` and the SM83 disassembler out to
`EmuSen.Crystal/Systems/GameBoy/` (§13.3, §13.5).

What reaches it [M: `grep -rn` over shipped code]: `CoreFactory.GameBoy` builds it for `.gb` and `.gbc`
(`CoreFactory.cs:37, 53`), `CoreFactory.Running` and `Bundle` name it (`:161, :185`), and `CoreCatalog` reads two of
its lists, `PadButtons` (`CoreCatalog.cs:202`) and `ModelSettings` (`:284`), and keeps `MercuryRtEngine` among its
`SoleEngines` (`:345`), which is what stops the generic branch from opening MercuryRT's sidecar in the shim's place. No
frontend names a type of the shim. On the Rust side MercuryRT exports the core ABI v1 (`src/ffi/v1.rs`) beside the
pre-stable `emusen_native_*` set (`src/ffi/mod.rs:38–203`) and twelve extensions of its own, six on each interface
(`mod.rs:210–303`, `v1.rs:264–321`).

### 17.2 Every behaviour the shim adds above the ABI, and where it goes

The routes are those of `EmuSen_CoreAPI.md` §30.6, in its order of preference, refined for this step: **host**,
the generic `CoreEngine` or `CoreDebugTarget`, from what v1 carries, no surface changed; **pack**, the Game Boy's
packs in `emusen-systems` and their C# halves (§13), from v1's spaces and registers; **space**, a space MercuryRT adds
to its machine info, a change to the core and none to the ABI; **capability**, an optional capability v1 already
defines, claimed by MercuryRT; **info**, MercuryRT's core or machine info; **retired**, with no caller left, which is
shown; **minor**, an addition to the core ABI. Each row names the shim's line and the destination's export, field or
member. "Shared" marks a destination the MoonRT and MarsRT shims need as well.

#### 17.2.1 Loading, refusals and the battery

| # | What the shim does | Where (file:line) | Route | Destination |
|---|---|---|---|---|
| L1 | Reads the header before the library sees the image and refuses a cartridge type no board implements, with `NotSupportedException("Cartridge type $XX is not implemented - see Mercury_Memory.md §4.")` | `MercuryRtCore.cs:163–169`; `GbCartridge.cs:29`; Rust `gb/cartridge.rs:51–60` | info, retired | MercuryRT's `create` already refuses the same types with its own status `-10`, its `status_text` "a cartridge type no board implements" (`v1.rs:87–96`) and the detail "cartridge type $XX is not implemented" (`v1.rs:104–106`). The pre-check is a second owner of which boards MercuryRT builds, and goes. The pack's header reader stops refusing: it names every type the header table defines (a console fact) and leaves the refusal to the engine (§13.3 said as much). A list of implemented types in core info was weighed and is not proposed: no host reads one, and `create`'s refusal is the engine's statement already (Q8) |
| L2 | The same refusals and an illegal opcode mapped to the C# oracle's exception types and messages: `NotSupportedException` for `-10` and `-20`, `InvalidDataException` for `-9` and `-11`, the fault band to the reproduced .NET exceptions | `MercuryRtCore.cs:200–218`; `PortEngine.cs:150–161`; `MercuryMachine.cs:38–50, 64–65` | retired | SR8: the oracle was deleted on 2026-10-16 (`Mercury_Native.md` §15). The host's `CoreRefusedException` carries the core's status and words, as for every v1 core. Through the frontend API this moves two answers, each a candidate defect of today to be shown first (§17.5): a refused Game Boy cartridge is `NoEngine` (−1109) today, through `OpenGame`'s catch of `NotSupportedException` (`ManagedRuntime.Open.cs:61`), where every other core's refused game is a fault; afterwards it is a fault like theirs |
| L3 | The state's two refusals made in C# before the bytes reach the machine: `EndOfStreamException` for a short state, `InvalidDataException("Not a Mercury save state.")`, a version outside 5 to 11 | `MercuryRtCore.cs:191–198`; `PortEngine.cs:242–250` | retired | MercuryRT's `state_load` refuses each itself (`machine.rs:255–259`, with `state.rs`' `TRUNCATED`, `FOREIGN` and `VERSION`), and machine info states the format, version and `loads_from` 5 to 11 (`v1.rs:138`). Candidate defect C2 (§17.5): the shim's `InvalidDataException` carries no `emusen.status`, so the frontend API reports a refused Game Boy state as a fault (`InvalidDataException: Not a Mercury save state.`) rather than `StateRefused` (−1114) (`ManagedSession.States.cs:38`, `ManagedCrystal.cs:25–34`); a short one as `Io` |
| L4 | The battery rule: file 0 passed only when the header says the board has a battery and RAM (`hasRam: HasBattery && RamLength > 0`), filed under `GB` or `GBC` by extension | `MercuryRtCore.cs:168`; `BatterySave.cs:26–27` | info, and one line of `create` | The folder is already the host's: `CoreEngine.LoadRom` files a `.gbc` game's save under `GBC` and a `.gb` game's under `GB` (`CoreEngine.cs:258–260`), which equals `GameBoyFolder` for every path. The `hasRam` rule is not: with core info listing no battery files, the host passes file 0 by its `.srm` rule at the first create (§6.3 of the core ABI), and MercuryRT's `create` writes it into cartridge RAM whatever the board (`v1.rs:109–118`), so a stray `.srm` beside a game whose board has RAM and no battery would fill that RAM at power-on. Proposed: MercuryRT's `battery_files` lists `{ which: 0, suffix: ".srm" }` for both systems, and `create` takes file 0 into cartridge RAM only where the board keeps a battery, the rule its machine info already states (`v1.rs:123, 137`). One create, as today. `GameBoyFolder` is then named only by tests and retires with them (Q6) |
| L5 | The mixer's coefficients computed in C# and sent at every load: `4194304.0 / 44100` and `Math.Pow(0.999958, …)` through `mercuryrt_core_set_sample_rate` | `MercuryRtCore.cs:41, 171–181`; `MercuryMachine.cs:53–57` | retired | MercuryRT computes the same pair when it builds a machine (`machine.rs:81–83`). *Measured on 2026-10-18* on this machine: Rust's `powf`, CPython's `math.pow` and .NET 10's `Math.Pow` give `3FEFDF574D84E421` from `4057C6F8C751F177`, the same bits, so the call changes nothing on Linux. It goes, with both exports (`v1.rs:264–268`, `mod.rs:210–214`), which then have no caller. The goldens' sound column is the gate (§17.4) |
| L6 | The pre-stable machine: `MercuryMachine`, its `Model=<number>` setting text, and the exports `mercuryrt_step`, `_cgb_hardware`, `_serial`, `_rom_patch`, `_ram_length`, `_set_sample_rate` with `native_exports!` and the `NativeCore` impl | `MercuryMachine.cs:1–84`; `MercuryNative.cs:12–20, 29`; `mod.rs:38–303` | retired | Called by two test classes alone, `MercuryRtPlatformTests` (one case, the platform digests) and `NativeHostTests` (three cases) [M]. `_cgb_hardware` restates machine info's `system`; `_ram_length` restates `space_size(2)`; the rest have v1 twins (`mercuryrt_core_step`, `_serial`, `_rom_patch`). The platform digests move to a v1 machine first. Deleted with the pre-stable interface (`EmuSen_CoreAPI.md` §13.5), at the deletion step of §17.7 (step 8) |
| L7 | The switch `EMUSEN_MERCURY_NATIVE=0`, which turns the Game Boy off as the pre-stable loader did | `MercuryNative.cs:10, 23`; `PortEngine.cs:45` | retired | Generic discovery has no per-core switch: `EMUSEN_VENUS_NATIVE=0` already leaves the SNES running (`CoreRegistrationGolden.txt:1908–1911`). Named outside the shim by three test classes (`NativeHostTests`, `MercuryRtFallbackTests`, `CoreRegistrationEquivalenceTests`) and the registration golden's `off` rows. `MercuryRtFallbackTests` then shows a build without the library by hiding it from discovery, and the `off.factory[GB …]` rows of the golden change from "turned off by EMUSEN_MERCURY_NATIVE=0" to the SNES's answer (Q5) |

#### 17.2.2 The machine as the engine presents it

| # | What the shim does | Where | Route | Destination |
|---|---|---|---|---|
| E1 | The Model setting's schema: key, label, help, three choices, default `Auto` | `MercuryRtCore.cs:54–80, 134–142`; `CoreCatalog.cs:284` | info (F20) | MercuryRT's `settings_schema` (`v1.rs:71–85`), already present with the same key, label, values and default, `scope: create`, `restart: true`. Its help is shorter than the shim's, which `EmuSen_Settings_Reference.md` §4.47 quotes the substance of; proposed: the shim's sentence moves into the schema, so the one owner keeps the reviewed text (Q7). `CoreCatalog.SettingsByConsole["GB"]` goes; `SettingsFor` reads a catalog console's sole discovered engine's sidecar, as `ManagedRuntime.Schema` already does for the SNES (`ManagedRuntime.Settings.cs:70`), which adds the SNES's own rows to the registration golden as well (shared, Q11) |
| E2 | A change of Model before the game's first frame loads the game again at once | `MercuryRtCore.cs:121–132` | host (shared) | Documented player-visible behaviour (`EmuSen_Settings_Reference.md` §4.47, "as the Expansion Pak row does"), and the reason a game's own Model reaches the machine through the frontend API at all: `CoreFactory.StoredSettings` hands the engine only the console's values at create (`CoreFactory.cs:57–63`), and `ApplyStored` layers the game's file over them only after the load (`ManagedRuntime.Open.cs:41`, `ManagedSession.cs:203–217`). On `CoreEngine` a create-scope `Set` is kept for the next load (`CoreEngine.cs:909–920`), and each open makes a new engine from the console's values, so on every v1 engine today a game's own create-time setting does not reach the machine when the game is opened, only a later load in the same session: candidate defect C3 (§17.5). Proposed: `CoreEngine.Set` of a create-scope key, with a game loaded and no frame run, makes the machine again with the new value, the shim's rule for every engine (Q4) |
| E3 | `CoreName` "GB" or "GBC", following the console running, a state from the other console included | `MercuryRtCore.cs:111–112` | retired | Machine info's `system` (`gb`, `gbc`, `v1.rs:125`) says which console runs; the console the session is filed under is `CoreEngine.Console`. `CoreName` becomes `Info.Name`, "MercuryRT", as on every v1 core. The name reaches further than the debugger's headings: DianaSh's `cov`, `load` and `dump` file under `Logs/<CoreName>`, Hotaru's screenshots and recordings too, and Hotaru keys a game's pad bindings by it (`GameWindow.axaml.cs:135, 389`, through the service's `engine`). That last is candidate defect C4 (§17.5), already present for every v1 core, which B4 would extend to the Game Boy (Q9) |
| E4 | Constants answered before a load: 160 by 144, `4194304 / 70224` Hz, state version 11, display aspect 10:9 (`DisplayShape.GameBoy`) | `MercuryRtCore.cs:27–38, 113–119` | info, retired | Machine info carries every one (`v1.rs:127–128, 138`) and `frame_info` the aspect (`:163`); `CoreEngine` answers them once a machine exists (`CoreEngine.cs:95–106, 734`) and 0, 60 or 0 before. `DisplayShape.GameBoy` is then named only by `DisplayAspectTests` (`:50`), which reads it before a load, and retires with that case (F26) |
| E5 | The pad: bit order right, left, up, down, A, B, Select, Start; port 0 alone; the whole pad sent each time; buttons held before a load sent after it | `MercuryRtCore.cs:23, 47–52, 183–188`; `PortEngine.cs:204, 215–223` | info, host | Core info's `gb.pad` names each button's bit and port (`v1.rs:18–41`), and `CoreEngine.SetButton` routes by it, a port with no controller ignored. `PadButtons` for `CoreCatalog.ButtonsByConsole` (`:202`) is read from the console's engine's core info, not restated in the pack (§5.3: "controllers and their controls: core info"). A button held across a load is not carried by `InputLatch.Reset`; since B2 a game ignores the press that launched it until every button is let go (`2216d6d4`), so nothing a player does depends on the carry [A]; recorded, not kept |
| E6 | The channel mutes held in the engine, cleared by a load, sent at once | `MercuryRtCore.cs:243–253`; `PortEngine.cs:297–301` | host | `CoreDebugTarget.SetChannelMuted` (`CoreDebugTarget.cs:270–275`), machine info's `audio.channels` naming the bits (`v1.rs:129`). The target keeps its mask across a reload where the machine starts unmuted; a target is made per load in every frontend, so the two agree [A]; recorded |
| E7 | A read of `CPUBUS` brings the ROM patches up to the registry first; a write to it while a debugger listens is reported at once | `PortEngine.cs:259–283` | host (shared) | Proposed: `CoreEngine.ReadSpace` calls `RefreshCheats` first, a version compare when nothing changed, for every core, so a debugger read after a code is entered answers as the next frame will. The write: MercuryRT's own `write_space(6)` notes the store in its hooks (`machine.rs:211–217`) and `CoreDebugBridge` drains it with the next frame rather than at once; recorded, not kept, unless the golden corpus's `watch` or `writers` shows it |
| E8 | The debugger's registries owned by the engine: labels outlive a target; coverage records call targets as entry points; the call stack resets at each load | `MercuryRtCore.cs:89–98, 180` | host (shared) | `CoreDebugTarget.Labels` is made per target (`CoreDebugTarget.cs:178`) and `CoreDebugBridge` wires no `EntryPointObserver` (only the three shims do, `MercuryRtCore.cs:92`, `MoonRtCore.cs:30`, `MarsRtCore.Debug.cs:63`). Proposed: `CoreEngine` owns a `LabelRegistry` and the bridge wires coverage's entry points, for every v1 core |
| E9 | `NativeDebugBridge` in place of the adapter's own, with its "a target listens" rule for stores and `Counters()` | `PortEngine.cs:109–112, 164, 210, 226–239`; `MercuryRtCore.cs:145–150` | host | `CoreDebugBridge`, the generic one, which logs stores whenever a watch or data breakpoint exists. `NativeDepth` and `Listening` are read by tests alone [M] and go with them |
| E10 | The mirror's registries supplied by the subclass (`MirrorCheats` with its setter, `MirrorCallStack`, `MirrorCoverage`), the spaces by name and the store-reporting spaces 1 to 5 | `MercuryRtCore.cs:100–104, 152–160` | info, retired | Machine info names the spaces and flags `reports_stores` on 1 to 5 and `side_effects` on `CPUBUS` (`v1.rs:131–135`) |
| E11 | The status words of the core's band | `MercuryRtCore.cs:209–218` | retired | The same four sentences are MercuryRT's `status_text` (`v1.rs:87–96`) |

#### 17.2.3 The debugger's view

`MercuryRtDebugTarget` shows the machine by reading its save state through the listing's paths (`Cpu.A`,
`Bus.Ppu.Lcdc`, `Cpu.<Cycles>k__BackingField`), so it is per-core code that reaches past the core ABI into the layout
of one engine's state. SR8 does not allow it to survive, and `CoreDebugTarget`, which every other v1 core is shown
through, publishes less: no video or sound registers, no sprites, no audio channels, no address decode, and the views
a pack gives only where the pack has them (`CoreDebugTarget.cs:98–108`). Item by item:

| # | What the view shows | Where | Route | Destination |
|---|---|---|---|---|
| D1 | CPU registers: A, F, BC, DE, HL, SP, PC, the flags Z N H C, IME, HALT, IE, IF and the cycle count | `MercuryRtDebugTarget.cs:175–198` | capability | `DEBUG_REGISTERS` (bit 16), already optional in v1: machine info's `processors[0].registers` lists the sixteen in this order with these widths (`v1.rs:136` lists none today) and `debug_registers(0)` returns them. This is `EmuSen_DianaSh.md` Q28's recommendation for MercuryRT; a register condition then costs what it costs on Nephrite rather than 6.84 µs a hit (§13.12 there). Q28 is not yet decided (Q2) |
| D2 | Video registers: LCDC, STAT as the CPU reads it, SCY, SCX, LY, LYC, BGP, OBP0, OBP1, WY, WX, the mode, the dot, the window's line; and in colour mode VBK, SVBK, BCPS, OCPS (as read), KEY1 | `:200–230` | **minor** | No v1 surface carries registers that belong to no processor. §17.3 proposes `units` |
| D3 | Sound registers `$FF10`–`$FF26` as read, the power bit, the sequencer's step, and the timer's DIV, TIMA, TMA, TAC; and a pseudo-register `Board:<name>` of value 0 | `:232–246` | **minor**; the board retired | The same `units`; the board's name is the pack's header reader's, which the summary already prints (`:526`), so the pseudo-register is not kept (Q5) |
| D4 | Sprites: the forty OAM entries with LCDC's height, the colour palette or the DMG one, hidden when parked off-screen; `MaxSprites` 40 | `:110, 275–296` | pack | The pack decodes OAM (a v1 space) with LCDC from D2's unit; `ISystemDebugViews` gains `Sprites` and `MaxSprites` (shared: MoonRT's view shows sprites too, and VenusRT's §30.6 item 6 needs the same members) |
| D5 | Palettes and the swatch: on a Game Boy, BGP, OBP0, OBP1 through four greys; in colour mode the sixteen palettes from palette RAM; in compatibility mode the DMG registers through palette RAM | `:20, 298–356, 475–498` | space and pack | Palette RAM is in no space (`v1.rs:15`). MercuryRT adds two, `BGPAL` and `OBJPAL`, 64 bytes each (ids 7 and 8, appended, so the existing numbering holds); the pack decodes them and the registers. The four greys are the view's presentation of shades a Game Boy does not define, stated in the pack |
| D6 | Audio channels: each channel's on and DAC bits, a level from its envelope or wave shift, the mute, and a line of its duty, frequency, length, sweep, divisor | `:248–273` | **minor** and pack | The internal counters (envelope volume, length counter, frequency, sweep) are in no register a CPU reads. Each channel is a unit of kind `channel` in §17.3's form; the pack turns a unit's registers into `DebugAudioChannelInfo`, its level rule (the wave's shift) being the console's; `ISystemDebugViews` gains `AudioChannels` |
| D7 | HDMA as one DMA channel in colour mode: blocks left, H-blank mode, source, destination | `:363–380` | **minor** and pack | HDMA's source and destination counters are in no readable register; a unit of kind `dma`, listed in colour mode only; the pack's `DmaChannels` reads it (its signature gains the units) |
| D8 | The tilemap entry, with the colour attribute from VRAM bank 1 in colour mode; the stride 1 | `:112–113, 417–426` | pack | The pack's `DecodeTilemapEntry` needs the mode: colour mode is machine info's `gbc` with the header's flag at `$0143`, which the pack reads through the `ROM` space with its own header reader; its signature gains the spaces |
| D9 | A tile's pixels (2bpp) and the tile sheet in greys | `:428–473` | pack | Over `VRAM`, a v1 space |
| D10 | Disassembly and static references (SM83) | `:382–415` | capability | `DEBUG_DISASSEMBLE` (bit 17), as VenusRT, Nephrite and Diamond decode in the core (§6.14: "the core decodes; the command formats"). The decoder is `emusen-systems`' `gb::disasm` (115 lines, with the C# `Sm83Disassembler` as its failover); a core does not link the platform's pack crate, so it moves to a crate of its own in `EmuSen/Cores/Shared/`, `emusen-sm83`, which MercuryRT and `emusen-systems` both use, as `emusen-mips` serves Diamond and the pack (F8; Q3) |
| D11 | Interrupt vectors, the five at `$40`–`$60` | `:133–141` | pack | A console fact; `InterruptVectors` |
| D12 | Address decode for `addr`: the fixed ranges, VRAM by VBK, WRAM by SVBK | `:500–517` | pack | `ISystemDebugViews` gains `ResolvePhysical`, reading VBK and SVBK from D2's unit (shared: §30.6 item 11) |
| D13 | The summary: console, frame, LY and mode; the cartridge's board, banks, RAM, battery and clock; the CPU line; the PPU line; the colour line; the APU line with the queue | `:519–542` | host and pack | `CoreDebugTarget.GetSummaryText`'s head line, then the pack's `Summary`, whose input gains the units and the spaces (the header from `ROM`). The head line becomes the generic one ("MercuryRT (Rust) 0.1.0, gb, frame 2"), a difference to record (Q5) |
| D14 | The load bars: none (an empty list) | `:91–92` | host | `CoreDebugTarget.HardwareLoad` shows one bar, `frame`, from the engine's timing; a difference to record |
| D15 | The audio the debugger peeks | `:360–361` | capability | `AUDIO_PEEK`, claimed already (`v1.rs:44`), read by `CoreDebugTarget.GetAudioSamples` |

**Counted.** Of 33 rows, 21 need no surface (host, info, pack, space or an optional capability v1 defines), 8 are
retired with their callers shown, and 4 (D2, D3, D6, D7) need registers that belong to no processor. §9's P3 predicted
that every shim's additions would fit an optional capability already defined, MarsRT's counters excepted; for MercuryRT
it does not hold, by those four, if the debugger is to keep what it shows. They have one form in common, below.

### 17.3 The one proposal that widens a surface: units, a core ABI minor

**What is missing.** A console's debugger shows registers that belong to no processor: the Game Boy's PPU, APU,
timer and HDMA here; the SNES's PPU and S-DSP (§30.6 items 1, 2 and 7 of `EmuSen_CoreAPI.md`); the NES's cartridge
board, which the tester decided on 2026-10-17 goes "into MoonRT's own debug registers capability and its info" (§13.4);
the Genesis's VDP. `IDebugTarget` already has places for them (`VideoRegisters`, `ApuRegisters`), and every C#
target filled them. v1 carries registers only per processor (`processors[].registers`, read by `debug_registers`).

**The forms weighed, per the project's rule for a feature that would widen a stable surface:**

| | Form | Verdict |
|---|---|---|
| a | **An extension of an existing surface**: machine info gains `units`, beside `processors`, and `debug_registers` reads a unit's id as it reads a processor's | **Proposed.** No new export and no new capability bit; one new field of machine info, which an older host ignores, read through an export cores already have |
| b | The registers listed as processor 0's | Rejected: `regs cpu` would show the PPU's registers as the SM83's, and a condition `if LY==0` would read a register that is no processor's |
| c | A processor with no program counter (`pc_bits: 0`) for each unit | Rejected on skew: a host of 1.12 makes a `DebugCpu` of every processor, so it would offer breakpoints, coverage and `step` on the PPU |
| d | The space route alone: a view space of the registers as latched, decoded by the pack | Covers part only: the I/O registers a CPU reads are already reachable through `CPUBUS`, but the mode, the dot, the window's line, the sequencer, the channels' counters and HDMA's are in no register file; a space holding them would have a layout of one engine's choosing, which the pack would then have to know, the coupling SR8 removes |
| e | A module, or a component of its own | Not applicable: the registers are state the core alone holds, read once a frame by the host that already reads its processors' |

**The change, sketched for 1.next** (no number is taken here):

```c
/* Machine info, §6.4: one new optional field, read by a host of 1.next and ignored by an older one. */
"units": [ { "id": 16, "name": "PPU", "kind": "video",
             "registers": [ { "name": "LCDC", "bits": 8 }, { "name": "STAT", "bits": 8 } ] },
           { "id": 17, "name": "APU", "kind": "audio", "registers": [ … ] },
           { "id": 18, "name": "Pulse 1", "kind": "channel", "registers": [ … ] },
           { "id": 22, "name": "HDMA", "kind": "dma", "registers": [ … ] } ]

/* §6.14: DEBUG_REGISTERS reads a unit as it reads a processor; the text gains one sentence. */
int64_t emusen_core_debug_registers(const emusen_machine *machine, uint32_t processor, int64_t *out, size_t len);
/* `processor` is a processor's id or, since 1.next, a unit's; ids are unique across both lists. */
```

- **`kind`** is open (`video`, `audio`, `channel`, `timer`, `dma`, `board`, a core's own word), as every enumerated
  string of machine info is. A host shows `video` as `VideoRegisters`, `audio` and `timer` in `ApuRegisters` in the
  order machine info lists them, `channel` and `dma` to the pack's views, and anything else under the unit's name.
- **Only with `DEBUG_REGISTERS`.** A core that lists units claims the capability, and answers `debug_registers` for
  each listed id with its registers in their listed order. A unit is listed only while the machine has it (HDMA and
  the colour registers on a Game Boy Color in colour mode); a change is announced by the `MACHINE_INFO` event, as a
  state from the other console already announces the system's.
- **Skew.** A 1.12 host ignores `units` and shows a 1.next core as now. A 1.next host reading a 1.12 core finds no
  `units` and shows none. A host asks only for ids machine info lists. No version check is needed beyond the field's
  presence.
- **What moves.** `emusen-native`: `MachineInfo.units` and its JSON (about 40 lines), the crate's check that ids are
  unique and that each listed unit answers with its count; the header's text and the baseline; `CoreDescriptors.cs`
  reading `units` and `CoreDebugTarget` mapping them (about 60 lines of C#); the kit: C14 checks each unit's count
  against `debug_registers`, and C15 reads every unit once; the test core lists one unit, and a seeded fault that
  answers a unit with one register short fails C14 alone. `EmuSen_CoreAPI.md` §6.4 and §6.14 gain the normative text
  and §17 its changelog row.
- **What a pack reads of a unit** is its registers by name, as the SNES's pack reads its symbols from the 65816's
  registers by name (§31.2 of `EmuSen_CoreAPI.md`). The Game Boy's pack states the names it reads (Pan Docs' names
  for the registers a CPU sees; the internal counters by the names the pack's page states), so a second Game Boy engine
  that publishes the same names gets the same views.

**Who else would use it.** VenusRT's §30.6 items 1, 2, 6, 7 and 21 become buildable (the SNES's sprites need OBSEL,
which only a PPU unit gives); MoonRT's board registers and the NES's PPU have a home; Nephrite's VDP has one. This is
why it is proposed as a minor of v1 and not as a `mercuryrt_*` extension, which no generic host may call (SR8).

**If it is not accepted** (Q1), the Game Boy is shown as Nephrite is: its CPU registers, disassembly, memory,
breakpoints, coverage, the call stack, palettes, tiles, the tilemap, sprites (with LCDC read through `CPUBUS`),
vectors and `addr`, and no video, sound or channel registers and no HDMA row. The golden corpus's `regs`, `channels`
and the summary's PPU and APU lines change accordingly.

### 17.4 The gates for phase 2

Each gate fails before the change it guards or is recorded as already passing, as §6 requires.

1. **MercuryRT's own oracles, unchanged** (`Mercury_Native.md` §15): the goldens of §14.4 there, graded by
   `MercuryGoldenTests` with `EMUSEN_MERCURYRT_CORPUS` and `EMUSEN_MERCURYRT_ROMS` set (the 36 frame runs and 3 step
   programs, the 176 random programs, the 173 corpus traces and the four games), now through `CoreEngine`; the
   hardware corpus held to `MercuryCorpusBaseline.tsv` under each of the three models (Auto 126 passed, 42 failed, 5
   without a verdict; 124, 44 and 5 on each forced console); `MercuryStateLayoutTests`' committed listings; and the
   crate's own tests, 38 in five binaries. A trace that moves is a failure, never a re-recording: nothing in this step
   is meant to change the machine.
2. **The kit's corpus on MercuryRT**: `emusen-core-conform` over the four games of the goldens (Tetris, Kirby's Dream
   Land, Link's Awakening, Pokémon Yellow, from the gitignored `TestRoms`) and over the 173 corpus ROMs, every case
   passing as on 2026-10-03 (§24.3 of `EmuSen_CoreAPI.md`), with C14 and C15 now finding the new spaces, the
   processor's registers and, if Q1 is accepted, the units; C13 counts `firmware_for` rounds as before.
3. **The debugger's golden corpus** (`GoldenCorpusTests`): the 29 files with a `mercuryrt` case either unchanged or
   each difference listed and decided before it is re-recorded. Expected, if §17.2's destinations are taken: the
   heading word `GB` becomes `MercuryRT` (`regs`, `cpus`, `spaces`, `dma`, and every refusal that names the core);
   the summary's first line becomes the generic head; the `Board:` row leaves `regs`; `coretop`, which has no
   `mercuryrt` case, would show one bar. Everything else the corpus prints on MercuryRT is meant to stay byte for byte,
   which is the measure of the pack's views. The Rust shell must print the same, as for every file.
4. **The registration golden** (`CoreRegistrationEquivalenceTests`): `probe` and `core` for the Game Boy's rows
   become `CoreEngine`; the `off` rows of L7; the Model row from the sidecar (unchanged in text if Q7 is taken); the
   SNES's own setting rows if Q11 is taken. Every other console's rows unchanged.
5. **Shown first, then fixed**: candidates C1 to C4 of §17.5, each by a test that fails on today's tree, or recorded as
   no defect.
6. **The cost**: `BreakpointConditionCostBench` (`EMUSEN_BP_BENCH=1`) on MercuryRT, a register condition about 1 µs a
   hit rather than 6.84 (P7 below); and the frame through `CoreEngine` against the frame through the shim on the four
   games, by `pacebench`'s frame-time distribution, equal within its run-to-run spread.
7. **`grep`** for a type under `EmuSen.Cores.Nintendo.MercuryRT` in shipped code: 0, as the B4 row asks.

**The WiseMan classes that cover the shim today** [M: every class whose file names a type of the shim or its rig, or
loads a `.gb` image through the factory, and the debugger's corpus], 46 run on 2026-10-18 at `480bf5e2`:
the Mercury claims `MercuryRtAccessBlockingTests`, `MercuryRtApuTests`, `MercuryRtAuditTests`,
`MercuryRtCartridgeTests`, `MercuryCartridgeTests`, `MercuryRtCgbTests`, `MercuryRtCoreSurfaceTests`,
`MercuryRtCpuTests`, `MercuryRtPpuTests`, `MercuryRtSerialTests`, `MercurySerialTests`, `MercuryDefectTests`,
`MercuryModelTests`, `MercuryStateLayoutTests`; the engine's `MercuryRtCoreTests`, `MercuryRtCoreAbiTests`,
`MercuryRtDebugTests`, `MercuryRtDebugTargetTests`, `MercuryRtPlatformTests`; the oracles `MercuryGoldenTests`,
`MercuryHardwareTestRomTests`; the commercial-ROM classes `MercuryCommercialRomDeterminismTests`,
`MercuryCommercialRomAudioTests`, `MercuryCommercialRomRenderSkipTests`; Mistress's `MercuryModelSettingTests`,
`MercuryRtFallbackTests`, `MainWindowEmulationMenuTests`; and `AudioBufferLimitTests`, `BatterySaveTests`,
`CheatCodecAdapterTests`, `ControllerPortTests`, `CoreFactoryCheatWiringTests`, `CoreOutputHashProbe`,
`CoreRegistrationEquivalenceTests`, `EngineFeaturesTests`, `NativeHostTests`, `RewindToMomentTests`,
`RustShimDisposeTests`, `RustStateHeaderTests`, `CheatCodecParityTests`, `ManagedCrystalTests`, `StateRefusalTests`,
`ConsoleBindingsTests`, `DisplayAspectTests`, `BreakpointConditionTests` and `GoldenCorpusTests`. Not in that run and
in the blast radius: `MercuryCommercialRomBatteryTests` (it names `BatterySave.GameBoyFolder` only), which phase 2
adds.

**The baseline, measured on 2026-10-18** at `480bf5e2`, Debug, .NET 10.0.112, with `EMUSEN_MERCURYRT_CORPUS` naming the
173 corpus ROMs and `EMUSEN_MERCURYRT_ROMS` the goldens' games:

| | Cases | Result |
|---|---:|---|
| The 46 classes above, one run | 775 | 774 passed, 1 failed: `The_games_run_as_their_goldens_recorded`, because the games folder first named held three ROMs besides the four with goldens, and the test grades every ROM it finds ("games/3296CE126E2EBF61 has no golden"); the run's setup, not a defect |
| `MercuryGoldenTests` and `MercuryHardwareTestRomTests` again, the folder holding the four games alone | 59 | 59 passed: the four games reproduced over 3,600 frames and again from their recorded states of frame 900; 173 corpus traces as recorded; the corpus's verdicts Auto 126 passed, 42 failed, 5 without one, and 124, 44 and 5 on each forced console; the broken trace caught in each column |
| MercuryRT's crate, `cargo test --profile test-checked` | 38 | 38 passed, in five binaries (8, 9, 16, 5 and 0 doc-tests) |
| The kit, `emusen-core-conform` on the release library, over the four games | 4 images | the core's cases pass (C1 to C5, C9 to C13, C15, C22, C23; capabilities `0x3830`, one setting, C13's 71,713 rounds); 4 of 4 images pass |
| The kit over the 173 corpus ROMs | 173 images | the core's cases pass; 173 of 173 images pass, in 128 s six at a time |

By class, the 775: `GoldenCorpusTests` 100 (every file, of which 29 have a `mercuryrt` case), `MercuryModelTests` 111,
`MercuryRtAuditTests` 69, `MercuryGoldenTests` 56, `MercuryStateLayoutTests` 52, `MercuryRtDebugTargetTests` 34,
`MercuryRtDebugTests` 33, `MercuryRtPpuTests` 27, `MercuryRtApuTests` and `MercuryRtCgbTests` 23 each,
`MainWindowEmulationMenuTests` 22, `MercuryRtCpuTests` 18, `BatterySaveTests` 17, `ConsoleBindingsTests` 16,
`CoreFactoryCheatWiringTests`, `DisplayAspectTests` and `MercuryRtCoreSurfaceTests` 13 each,
`BreakpointConditionTests` 12, `MercuryRtCoreTests` and `MercuryRtSerialTests` 9 each, `CheatCodecAdapterTests` and
`MercuryRtCoreAbiTests` 8 each, `MercuryCartridgeTests` and `MercuryDefectTests` 7 each, `AudioBufferLimitTests` and
`ManagedCrystalTests` 6 each, `CheatCodecParityTests`, `EngineFeaturesTests`, `MercuryRtPlatformTests`,
`MercurySerialTests` and `NativeHostTests` 5 each, `ControllerPortTests`, `MercuryRtAccessBlockingTests` and
`MercuryRtCartridgeTests` 4 each, `CoreRegistrationEquivalenceTests`, `MercuryHardwareTestRomTests`,
`MercuryModelSettingTests`, `RewindToMomentTests` and `RustStateHeaderTests` 3 each, `CoreOutputHashProbe`,
`MercuryRtFallbackTests`, `RustShimDisposeTests` and `StateRefusalTests` 2 each, and the three commercial-ROM classes
1 each. Those three ran their empty case only: no commercial ROM lies under the install's `TestRoms`, which is where
they look, so the four games are graded by `MercuryGoldenTests` and the kit alone.

### 17.5 Candidate defects read at the seam, none yet shown

Each was read from the code and not run; phase 2 shows each by a test that fails first, or records it as no defect.

- **C1. A refused Game Boy cartridge is "no engine" through the frontend API.** `GbCartridge.Read` throws
  `NotSupportedException`, which `ManagedRuntime.OpenGame` turns into `NoEngine` (−1109) (`ManagedRuntime.Open.cs:61`),
  the answer for a file no engine opens; every other engine's refusal is `CoreRefusedException`, a fault, with the
  core's words. Removed by L1 and L2.
- **C2. A refused Game Boy state is a fault through the frontend API**, not `StateRefused` (−1114): the shim's
  pre-check throws an `InvalidDataException` with no `emusen.status` (`MercuryRtCore.cs:194, 197`), which
  `ManagedSession.LoadState` does not take (`ManagedSession.States.cs:38`), and a state shorter than 8 bytes is `Io`.
  Removed by L3.
- **C3. A game's own create-time setting does not reach a v1 engine's machine when the game is opened**, on every v1 core, only a later load in the same session: E2. The shim
  hides it for the Game Boy, and MarsRT's shim for the Expansion Pak. Fixed in the host, for every engine.
- **C4. Hotaru keys a game's pad bindings by the engine's name** (`GameWindow.axaml.cs:135, 389`; the service's
  `engine` is the target's `CoreName`), where Mistress keys them by the console (`EmuSen_Settings_Reference.md`
  §4.47). On the SNES, the Genesis and the PlayStation the key is the engine's name, so bindings a player made in
  Mistress would not be found in emusen-shell's window, and a Game Boy Color game on the Color looks up `GBC`, which no
  binding set is named. B4 would move the Game Boy's key from `GB` to `MercuryRT`. A frontend's fix, outside B4,
  proposed before phase 2 switches the Game Boy (Q9).

### 17.6 What only the Game Boy's shim keeps alive, for the deletion step

Measured on 2026-10-18 [M: `grep -rnw` over every project, tests included]. **Nothing in `PortEngine.cs` or in the
pre-stable host is kept alive by the Game Boy's shim alone**: the MoonRT shim derives from `PortEngine` too
(`MoonRtCore.cs:12`), reads `PortMachine` (`MoonRtDebugTarget.cs:82`) and builds a `MoonMachine` on the pre-stable
interface, the VenusRT shim's `VenusMachine` and `VenusNative` use `NativeMachine`, `NativeInterface` and
`NativeCoreLibrary`, and MarsRT's `MarsNative.Library` is a `PortLibrary` (`MarsNative.cs:11`). So the deletion step
waits on all three branches. What the Game Boy's shim alone uses, member by member:

| Piece | Used only by the Game Boy's shim or its tests |
|---|---|
| `PortEngine.MirrorCheats`' setter override and `MirrorCoverage`/`MirrorCallStack` supplied by a subclass | `MercuryRtCore.cs:100–102` (MoonRT takes the defaults) |
| `PortMachine.Save(byte[])` and `StateSize` | `MercuryRtCore.cs:234–236` (MarsRT's are its own) |
| `PortLibrary`'s `requiredExports` with an export no other port needs | `mercuryrt_core_set_sample_rate` (`MercuryNative.cs:23`); MarsRT passes its own |
| `PortMachine.RomPatch` through `PortLibrary.RomPatchExport` | `MercuryRtCoreTests.cs:175–179` and `MoonRtFrontendTests` both; not Mercury's alone |
| `NativeMachine`'s settings text, non-empty | `MercuryMachine.cs:30` (`Model=<n>`); MoonRT and VenusRT pass `""` |
| `NativeCoreLibrary.Common("mercuryrt", …)`, `MercuryNative.Api`, `RequiredCapabilities` | `NativeHostTests` (three cases) and `MercuryRtPlatformTests` (one) |
| MercuryRT's six pre-stable exports and `native_exports!` | `MercuryMachine` alone |

Found on the way, for the deletion step's plan though not Mercury's:

- **`NativeRtCore<T>` has no subclass left** [M]. It survives for its static `RomPatchTriples`, which the generic
  `CoreEngine.RefreshCheats` calls (`CoreEngine.cs:849`) and two tests name (`NativeHostTests.cs:23–27`,
  `GenesisCheatFormatsTests.cs:96`). The method moves to `CoreEngine` (or to `CheatRegistry`) before `NativeRtCore.cs`
  is deleted, with `INativeDebugBridge`, which it declares and only `NativeDebugBridge` implements.
- **`PortLibrary` is the only part of `PortEngine.cs` MarsRT uses.** Its load rule (the switch variable, the file
  beside the assemblies, the required capabilities and exports) has a generic twin in discovery except for the switch
  (L7); MarsRT's branch decides whether its shim keeps it.
- `DisplayShape.GameBoy` is named by this shim alone in shipped code (E4).

### 17.7 The build plan for phase 2

Each step is committed on its own, with the blast radius of §17.4 run before the next, and the shim kept as the Game
Boy's engine until step 7 switches it, as the B4 row's failover asks ("its shim stays until its core's exports pass").

1. **Show the candidates.** C1 and C2 by `StateRefusalTests`/`ManagedCrystalTests` cases through the frontend API on a
   synthetic Game Boy image; C3 by a `ManagedCrystalTests` case on a v1 engine with a create-time setting in a game's
   file; C4 by a Hotaru case if Q9 sends it with this step. Each fails on today's tree.
2. **The host's generic pieces (shared).** `CoreEngine` owns a `LabelRegistry`; `CoreDebugBridge` wires coverage's
   entry points; `CoreEngine.ReadSpace` refreshes ROM patches first; a create-scope `Set` before the first frame makes
   the machine again (C3 passes). `RomPatchTriples` moves into `CoreEngine`. Every v1 core's debug and settings tests
   in the blast radius; built once for the three B4 branches, by the branch Q12 names, the others rebasing on them.
3. **MercuryRT's own exports and info**, with its goldens and the kit after: `DEBUG_REGISTERS` for processor 0;
   `emusen-sm83` (moved from `gb::disasm`, the pack re-exporting it, its parity class unchanged) and
   `DEBUG_DISASSEMBLE`; the spaces `BGPAL` and `OBJPAL`; `battery_files` and `create`'s battery rule (L4); the Model's
   help (Q7). The C# shim reads `DEBUG_REGISTERS` from here on, so Q28's cost is measured on the shim before the switch.
4. **The minor, if Q1 accepts it**: `units` in `emusen-native`, the header, the baseline, the host's reader, the kit's
   C14 and C15 with a seeded fault; then MercuryRT's units (PPU, APU, timer, the four channels, HDMA). The kit's corpus
   on MercuryRT and on every other core (no other core lists a unit, so every other report is unchanged).
5. **The Game Boy's pack views**, in Rust in `emusen-systems` (`gb::views`) and in C# beside them under the parity
   rule of §13: sprites, palettes and swatch, tiles and the sheet, the tilemap entry, the channels and HDMA from units,
   vectors, `ResolvePhysical`, the summary lines; `ISystemDebugViews` gains `Sprites`, `MaxSprites`, `AudioChannels`,
   `ResolvePhysical` and the units as an input (shared), and `emusen_systems_view` the matching views (a platform
   interface bump). A parity class over generated OAM, VRAM, palette RAM and register sets, each answer compared with
   the shim's target on the same machine, run while the shim still exists, with seeded faults.
6. **The pack's header reader stops refusing** (L1), both halves, the parity class's header cases amended to name
   every type; `MercuryCartridgeTests`' refusal case moves to MercuryRT's create.
7. **The switch.** `MercuryRtEngine` leaves `SoleEngines` and joins `SoleDiscoveredEngines`; `CoreFactory`'s `.gb`
   arm becomes `Missing("Game Boy", MercuryRtEngine)`, and `Running`'s and `Bundle`'s cases go;
   `ButtonsByConsole` and `SettingsByConsole` lose the Game Boy (E1, E5). WiseMan's `MercuryRig` and `MercuryDebugRig`
   build a `CoreEngine` and call `mercuryrt_core_step`, `_tick`, `_stop`, `_serial` and `_rom_patch` as the core's
   own extensions (§4.7 of the core ABI), with `GbModel` and the space names moved into the fixtures. Every gate of
   §17.4; the golden corpus's differences listed and, once decided, re-recorded in this commit alone.
8. **The deletion.** The shim's four files, `DisplayShape.GameBoy`, `BatterySave.GameBoyFolder` with the tests that
   named them, the `mercuryrt_core_set_sample_rate` export; and, once the other two branches are through, `PortEngine.cs`,
   the pre-stable host (`Native{CoreLibrary,Interface,Machine,RtCore,DebugBridge}`), MercuryRT's `src/ffi/mod.rs`
   pre-stable half, `emusen-native`'s `abi` macro and the three dead classes of §4.1, in one commit across the cores.
9. **The records.** `Mercury_Native.md` gains its section for the step, `Mercury_Debug.md` the views' new home,
   `EmuSen_Settings_Reference.md` §4.47 the host's rule, and this section its "what was built" sequel with the sizes:
   §7 estimated 1,500 to 3,000 lines of Rust across the four cores' exports; for MercuryRT about 250 in the core,
   about 120 in `emusen-native` and the kit for the minor, about 700 in the pack's two halves with its parity class,
   about 150 in the host, against 914 deleted from the shim and about 270 of pre-stable exports [A].

**Predictions to retire.**

- **P7.** A register condition on MercuryRT costs within 0.5 µs of Nephrite's 1.88 µs a hit, measured by
  `BreakpointConditionCostBench` at step 3. Retired at step 3.
- **P8.** No golden trace, corpus verdict or kit report on MercuryRT changes at any step; the only recorded outputs
  that change are the golden corpus's lines of §17.4 gate 3 and the registration golden's of gate 4. Retired at step 7.

### 17.8 Questions for the tester

Asked one by one, each with its recommendation; nothing here is decided by this section.

- **Q1. The minor `units`** (§17.3): machine info's `units` read through `DEBUG_REGISTERS`, proposed for 1.next as an
  extension of an existing surface? *Recommended: yes*, since four of the Game Boy's views and five of VenusRT's
  §30.6 items have no other home that keeps a generic host. Without it the Game Boy is shown as Nephrite is (§17.3,
  last paragraph).
- **Q2. `EmuSen_DianaSh.md` Q28 for MercuryRT**: MercuryRT exports `DEBUG_REGISTERS` for its processor within B4?
  *Recommended: yes*, as Q28 recommended; the capability is optional in v1 already.
- **Q3. The SM83's decoder in `EmuSen/Cores/Shared/emusen-sm83`**, used by MercuryRT's `DEBUG_DISASSEMBLE` and by
  `emusen-systems`, amending F8's "the others in their packs" for the SM83 as `emusen-mips` was placed for MIPS?
  *Recommended: yes*: the core decodes, as every generic core does, and the pack keeps its one owner. The alternative,
  `CoreDebugTarget` falling back to a pack's disassembler for a core without the capability, keeps the decoder where it
  is but gives the host a second way to disassemble.
- **Q4. A create-time setting changed before a game's first frame makes the machine again, in `CoreEngine`, for every
  v1 engine** (E2, C3)? *Recommended: yes*: it is the documented behaviour of the Model and Expansion Pak rows, and it
  makes a game's own create-time setting take effect at its first load on every engine.
- **Q5. The golden corpus's and registration golden's expected differences** (§17.4 gates 3 and 4: `GB` to
  `MercuryRT` in headings and log folders, the generic summary head, the `Board:` pseudo-register dropped, one load
  bar, the `off` rows): accept them, re-recorded in the switch's commit? *Recommended: yes*: each brings the Game Boy to
  what every v1 core prints.
- **Q6. The battery rule** (L4): core info lists file 0 and `create` takes it only on a board with a battery, rather
  than core info listing none and the host creating twice when a save exists? *Recommended: the first*: one create, and
  the rule beside the machine info that states it.
- **Q7. The Model's help**: the shim's longer sentence becomes the schema's, so the window keeps its words?
  *Recommended: yes.*
- **Q8. Unimplemented cartridge types**: refused by MercuryRT's `create` alone, the pack naming every type, rather than
  a list of implemented boards in core info? *Recommended: the first*: no host would read the list, and it would be a
  field of 1.next with no reader.
- **Q9. Hotaru's binding key** (C4): fixed before phase 2's switch, as a frontend change of its own shown by a failing
  test first, so that the Game Boy's bindings do not move? *Recommended: yes.*
- **Q10. The off switch `EMUSEN_MERCURY_NATIVE`** (L7): retired with the pre-stable loader, a build without the library
  shown by hiding it from discovery as for the SNES? *Recommended: yes.*
- **Q11. `CoreCatalog.SettingsFor` reads a catalog console's sole discovered engine's schema** (E1), which adds the
  SNES's rows to the registration golden too? *Recommended: yes*, coordinated with the VenusRT branch, so that the
  console's settings have one owner in every list.
- **Q12. Order across the three B4 branches**: the shared host pieces (§17.7 step 2, the `units` minor,
  `ISystemDebugViews`' new members) built once by whichever branch reaches them first, the others rebasing? Or one
  branch named to build them? *Recommended: one branch builds the shared host pieces and the minor as a step of their
  own, merged before any core switches*, so that no two branches change `CoreEngine`, the header or the pack's views
  interface at once.

### 17.9 Built 2026-10-18: steps 1 (C1), 3 and 6

*Measured on 2026-10-18 on branch `b4-mercury`, from `WiseMan` at `4b42f4f1`, after §20's decisions. The steps of
§17.7 that need no ABI change and no host piece the three cores share; C2 to C4, step 2, the minor and the pack's views
are another branch's (§20), and steps 4, 5, 7 and 8 wait on it.*

- **C1, shown and removed.** `GameRefusalTests` opens an MBC7 image (`$22`) through `CrystalRuntime`: before the change
  the open failed with `NoEngine` (−1109); after it, with a fault whose words name MercuryRT and `$22`. Two changes made
  it: the header reader stopped refusing (step 6, below), and the shim throws `-10` as the host's
  `CoreRefusedException`, in the words the generic host will print after the switch ("MercuryRT (Rust) refused the
  game: cartridge type $22 is not implemented."), rather than the deleted C# core's `NotSupportedException`.
- **Step 6.** `gb::cartridge` and `GbCartridge.ReadManaged` name every type the header table defines: MMM01
  (`$0B`–`$0D`), MBC6 (`$20`), MBC7 (`$22`), Pocket Camera (`$FC`), TAMA5 (`$FD`), HuC3 (`$FE`), HuC1 (`$FF`), and
  `unknown` for the codes the table leaves undefined. The refusal of an image shorter than the header stays, a
  console fact. `SystemCodeParityTests`' header case, every type at five lengths, holds the two halves equal unchanged;
  `MercuryCartridgeTests` names eight boards and has MercuryRT's create refuse `$22` with status −10.
- **Step 3**, MercuryRT's own exports and info (`src/ffi/v1.rs`, `src/machine.rs`): `DEBUG_REGISTERS` with the
  sixteen registers of D1 in machine info's `processors[0]`; `DEBUG_DISASSEMBLE` over any space, decoded by
  `EmuSen/Cores/Shared/emusen-sm83` (no standard library; the table that was `gb::disasm`, which now re-exports it, so
  `DebugCodeParityTests` grades the moved table unchanged); `BGPAL` and `OBJPAL` as read-only spaces 7 and 8 of 64
  bytes, a write refused with `READ_ONLY`; `battery_files` naming file 0 `.srm` for both systems, and `create` keeping
  file 0 only on a board with a battery; the Model's help as §4.47 of the settings reference words it. The shim still
  shows the machine through its own target and reads none of this: its spaces, registers and disassembly are its own
  until the switch, so the debugger's golden corpus is unchanged.
- **P7, retired as predicted, but not where §17.7 put it.** §17.7 measured it "on the shim before the switch"; reading
  `BreakpointExpressions` showed that a condition refreshes the whole target, and the shim's refresh retakes its whole
  state snapshot whatever its registers come from, so the shim could not show the saving. It was measured instead
  through the generic engine the Game Boy switches to, by a row `mercuryrt-v1` added to
  `BreakpointConditionCostBench` (Release, `EMUSEN_BP_BENCH=1`, 2,926 hits a frame):

  | Target | A register condition, a hit | A register logpoint, a hit | `if 0`, a hit |
  |---|---|---|---|
  | MercuryRT through its shim | 5.27 µs | 5.29 µs | 0.30 µs |
  | MercuryRT through `CoreEngine` and `DEBUG_REGISTERS` | 0.98 µs | 1.08 µs | 0.42 µs |
  | Nephrite (`CoreDebugTarget`), the same run | 1.35 µs | 1.27 µs | 0.57 µs |

  0.98 µs is within 0.5 µs of Nephrite's 1.88 of §13.12 and of its 1.35 here. At 2,926 hits a frame the condition now
  costs about 2.9 ms a frame against 15.4 through the shim.

**The gates as measured.**

| | Result |
|---|---|
| MercuryRT's crate, `cargo test --profile test-checked` | 42 in six binaries (8, 9, 16, 5, the new `v1` with 4, 0 doc-tests), all pass; four seeded faults (the battery rule dropped, Z read from bit 6, the branch's base off by one, palette RAM writable) each fail one case |
| `emusen-sm83` | 3 pass |
| `emusen-systems` and `emusen-platform`, `cargo test --release` | 31, 4 and 19 pass |
| The kit on the release library | capabilities `0x33830`; the core's cases pass, C14 finding 9 spaces and file 0 listed by core info and kept by no image of the four; 4 of 4 games and 173 of 173 corpus ROMs pass |
| WiseMan, §17.4's 46 classes with `GameRefusalTests`, `MercuryCommercialRomBatteryTests`, `SystemCodeParityTests`, `DebugCodeParityTests`, `SystemTableParityTests` and `CoreDiscoveryTests`, the goldens' games and corpus set | 809 cases: 808 passed, and `MercuryRtCoreAbiTests`' capability case failed as it should, pinning the old set; with the two capabilities added it passes, 8 of 8. `GoldenCorpusTests` 100 of 100 and the registration golden unchanged; the goldens' 56 cases, the four games and the 173 corpus traces as recorded |
| The header's and refusal's classes again with `EMUSEN_SYSTEMS_NATIVE=0` | 25 of 25 |

### 17.10 Built 2026-10-18: steps 4, 5, 7 and 8 (MercuryRT's part) and 9; the Game Boy on the generic engine

*Measured on 2026-10-18 on branch `b4-mercury`, from `WiseMan` at `e75ad6bf` (core ABI 1.13 and §22's host pieces in)
and merged with `a17ff513` (MoonRT's switch, §23) before the last runs. The rest of §17.7 under §20's decisions. The
deletion of `PortEngine.cs`, the pre-stable host, MercuryRT's pre-stable half (`src/ffi/mod.rs`) and the three dead
classes waits for MarsRT's switch, as §20 orders it.*

**Step 4, MercuryRT's units** (`src/ffi/v1.rs`, ids from 256, read through `debug_registers`): `PPU` (video) with the
fourteen registers the shim's target printed, STAT read as the CPU reads it, and in colour mode VBK, SVBK, BCPS, OCPS
and KEY1 after them; `APU` (audio), `$FF10`–`$FF26` as the CPU reads them, the power bit and the sequencer's step;
`Timer` (timer), DIV, TIMA, TMA, TAC; `Pulse 1`, `Pulse 2`, `Wave` and `Noise` (channel), each channel's on and DAC
bits and the counters no register shows; `HDMA` (dma) in colour mode only. Seven units, eight in colour mode; the
bus's own I/O read, which disturbs nothing, serves the registers a CPU reads. The generic target's video and sound
sections therefore print as the shim's did, but for the `Board:` row (§20).

**Step 5, the Game Boy pack's views in both halves.** `GameBoyDebugViews` (C#) and `emusen-systems`' `gb::views`
(Rust), the second reading the same bytes in the same order, in batches where the C# reads one at a time so that a
view takes at most sixteen rounds: palettes and the swatch (DMG greys from BGP, OBP0, OBP1; the Color's sixteen from
`BGPAL` and `OBJPAL`; compatibility mode through palette RAM), the tile sheet, a tile's pixels, the tilemap entry
with bank 1's attribute in colour mode, the five vectors, forty sprites at most from OAM with LCDC's height, the
channels from their units, HDMA as one row, the address map behind `addr`, and the summary's cartridge, CPU, PPU,
colour and sound lines. Colour mode is the PPU unit's VBK; compatibility mode two VRAM banks without it, so no view
reads the machine's state. Two surface changes came with them:

- `ISystemDebugViews` gained `Palettes` and `PaletteSwatch` overloads taking the units, defaulting to the old
  members, since a Game Boy's palettes are registers; `CoreDebugTarget` passes them, and its view ring learns again
  when machine info changes, as a state from the other console changes it.
- **Platform interface 21** (MarsRT's N64 pack took 20 first, §18.10.2): `emusen_systems_view` takes the systems
  `gb` and `gbc`, a request's `units` (each a name,
  a kind and its registers) and `mutes`; `NativeDebugViews` sends the units with each view that takes them, and a
  units-reading summary to the library first, a system without that view (the NES today) answering from the C#
  views. This is the pattern the NES's Rust twins can follow.

`DebugCodeParityTests.Every_game_boy_view_answers_alike_and_reads_alike`: 48 rounds of generated spaces and units in
each console mode (a ROM cut at the header's length, OAM short, palette spaces absent or named in lower case, units
left out, values past their widths), every tilemap entry and tile of a 16 KiB bank, the address map from end to end;
answers and reads alike. Seeded faults, each caught: a DMG sprite's palette from bit 3, the wave's level from 3 − vol,
the I/O registers called addressable.

**Step 7, the switch.** `MercuryRtEngine` left `CoreCatalog`'s sole engines and joined its sole discovered engines
beside the SNES's and the NES's; `CoreFactory`'s `.gb` arm names the missing engine, as the SNES's does; the Game Boy's
pad is read from its engine's info and its settings from its engine's sidecar, which gives the SNES and the NES their
own rows in `SettingsFor` as well (§17.8 Q11). MercuryRT's ROM became a read-only space, as the shim's target showed
it (C14 now checks the refusal), and an illegal opcode in an observed frame is refused in the core's words with its
detail word. WiseMan holds MercuryRT through `Fixtures/MercuryEngines` (the generic engine over its library, its own
`mercuryrt_core_step`, `_tick`, `_stop`, `_serial` and `_rom_patch`, `GbModel` and the facts its machine info
states); `MercuryRig` reads an illegal opcode as the corpus's verdicts and the goldens' transcripts record it, so the
hardware-test runner's `IllegalOpcode` holds. The shim-against-adapter comparison left with the shim, and
`MercuryRtFallbackTests` points discovery at an empty folder, as the SNES's test does.

**Three host defects the switch showed**, each by a test that failed on the switched tree before its fix:

- *A save kept beside a game was copied into Saves for a board without battery RAM*
  (`BatterySaveTests.A_cartridge_without_ram_has_no_save_at_all`). The generic engine opens file 0 for the first create,
  as core info lists it, and `BatterySave.Open` copied an old home's save in on opening. `Open` now reads the old home
  in place, and `Settle` copies it in when a load knows the machine keeps the file. The same rule reached the NES: a
  battery-less NES game's save beside it is no longer copied into Saves (the registration golden's `NES plain` rows).
- *A refused debug frame dropped its detail word*: `CoreDebugBridge` now carries it, as `CoreMachine.Advance` does.
- *A Game Boy view could not see the units' registers it needs for palettes*: the overloads above.

**Step 8, MercuryRT's deletions.** The shim's four files, `DisplayShape.GameBoy`, `BatterySave.GameBoyFolder` and
`mercuryrt_core_set_sample_rate`; `EMUSEN_MERCURY_NATIVE` retired with them (§20). `grep` for a type under
`EmuSen.Cores.Nintendo.MercuryRT` in every project: 0.

**The debugger's corpus, recorded again**, eleven files with a `mercuryrt` case:

| Difference | Files | Standing |
|---|---|---|
| `GB` becomes `MercuryRT` in headings and refusals | `regs`, `cpus`, `spaces`, `dma`, `mem`, `tile`, `counters` | decided (§20) |
| the summary's head is the generic one | `summary` | decided (§20) |
| the `Board:ROM` row leaves `regs` | `regs` | decided (§20) |
| `SM83` becomes `CPU`, machine info's name for the processor | `cpus`, `regs`, `setreg` | as §20 decided `CPU` for the 2A03; **for the tester to confirm for the SM83** |
| `BGPAL` and `OBJPAL` listed; the ROM read-only as before | `spaces`, `mem`, `tile`, `counters` | follows from §17.2 D5, decided |
| `channels` after `mute 0 on` shows the channel muted | `mute` | a correction: the old recording read the shim's snapshot from before the mute |
| `eval symbols` lists the processor's registers, `frame` and `stackdepth`; the PPU's, APU's and timer's register names are no longer symbols | `eval` | **for the tester**: the host piece §23.6 recommends for every core with units; built here once and taken out again so as not to move the NES's recording |
| the summary's sound line loses `buffered=1476 frames` | `summary` | **for the tester**: the queue's length is the engine's, not a register or a space a pack reads |

**The registration golden**, recorded again: the Game Boy's rows are `CoreEngine`, `CoreDebugTarget`, the engine's
name and the Model row from the sidecar (its text the shim's, §17.9); the `off` rows run the Game Boy, no switch
turning it off; a tampered sidecar now refuses MercuryRT as trust refuses any discovered engine (the shim loaded its
library unchecked); the SNES's and the NES's settings rows appear; the NES's battery rows as above. Every other row is
unchanged.

**The gates as measured**:

| | Result |
|---|---|
| MercuryRT's crate, `cargo test --profile test-checked` | 43 in six binaries, all pass |
| `emusen-sm83`; `emusen-systems` and `emusen-platform` (`--release`) | 3; 24, 31 and 4, all pass |
| The kit on the release library | ABI 1.13; the core's cases pass, C14 finding 9 spaces, the ROM refused a write, 7 or 8 units; 4 of 4 games and 173 of 173 corpus ROMs |
| WiseMan on the merged tree, 99 classes: §17.4's, the parity classes, every Mercury class, the Moon classes, the frontend API's, the host's (`SharedHostTests`, `CoreAbi113Tests`, `CoreAdapterTests`, `CoreEngineThreadTests`, `DebugPlumbingTests`), the other cores' engine, debug and media classes, Mistress's settings and engine windows, `GoldenCorpusTests`, with the goldens' games and corpus set | 1,410 cases, 1,407 passed. Two failures were the registration test reading the golden's copy from the build before the recording (it passes after a build: the class and `GoldenCorpusTests`, 103 of 103); the third, `GraphicsSettingsWindowLayoutTests`' N64 case at 3x with antialiasing 3x, is the N64's settings window, which this step does not touch: it failed the same way in each of three combined runs and passes alone, the class 14 of 14. `GoldenCorpusTests` passes on both shells, 100 of 100 with `EMUSEN_DIANASH_NATIVE=0` as well |
| MercuryRT's oracles on the generic engine | the goldens' 56 cases (four games, 173 corpus traces, the synthetic and random programs) as recorded; the corpus baseline under all three models; `MercuryStateLayoutTests` 52 |

**P8 does not hold as written.** No golden trace, corpus verdict or kit report on MercuryRT changed, but the corpus
changed in four ways §17.4 had not listed (the processor's name, the new spaces, the muted channel, the symbols and
the queue), each in the table above.

**Left**: the cross-core deletion (§20); the units' registers as expression symbols and the summary's queue, for the
tester; the Rust twins of the other consoles' new view members.

---

## 18. B4's first phase for MarsRT, 2026-10-18: the shim inventoried, a proposal, the gates and a plan

*Read on 2026-10-18 on branch `b4-marsrt`, from `WiseMan` at `480bf5e2` (core ABI 1.12). The first phase of §6's B4
row for one core: every behaviour MarsRT's shim adds above the core ABI v1 and every private export it calls, each
given a destination; the telemetry proposal D11 asks for before anything is built; the gates the second phase must
pass; a plan in steps; and the questions only the tester can answer. **Nothing was built**: no export, no ABI change,
no edit to MarsRT or the host. Line numbers are of the files at `480bf5e2`. Claims are marked as at the head of this
page.*

### 18.1 What the shim is

- **Nine files, 1,478 lines, 1,247 of them not blank** [M: `wc -l`, `grep -cv '^\s*$'`], in
  `EmuSen/Cores/Nintendo/MarsRT - N64/Shim/`: `MarsRtCore.cs` (473) and `MarsRtCore.Debug.cs` (293), one class, a
  subclass of the generic adapter `CoreEngine`; `MarsRtDebugTarget.cs` (228); `Disassembler/RspDisassembler.cs` (235);
  `MarsMachine.cs` (94), the state tests' handle; `N64Console.cs` (68), the console's facts read with no library;
  `MarsDebugSpaces.cs` (46); `MarsNative.cs` (27), the loader; `Cheats/MarsCheatCodecs.cs` (14).
- **Shipped code names it in four places** [M: `grep` over every project but WiseMan]: `CoreFactory.cs:38–39` (the N64
  arm, `new MarsRtCore(expansionPak: true)`), `:160` (the engine's name), `:194–195` (the bundle, with
  `MarsRtDebugTarget`); `CoreCatalog.cs:203`, `:242` and `:283` (`N64Console`'s pad, axes and settings rows);
  `ControllerPorts.cs:43` (`N64Console.Ports`); and MarsRT's PGO driver (`pgo/driver/Program.cs:27`, tooling, which
  needs only `ICore` and `ICoreSettings` and stays). Mistress names `N64Console` in a comment only
  (`Input/ControllerDiagrams.cs:85`). No DianaSh command, no part of Hotaru, Serenity, Pharaoh or the shell names a shim
  type: each reaches MarsRT through `ICore`, `ICoreSettings`, `IFrameProfiler` and `IDebugTarget`, so every behaviour
  below reaches them through those interfaces, and the debugger's behaviour through DianaSh's commands.
- **The 21 `marsrt_core_*` extensions** (`MarsNative.cs:14–17`) are all required: `PortLibrary` refuses the library if
  one is missing (`PortEngine.cs:44–56`), so the N64 runs nothing without them. MarsRT also exports **21 component
  exports** the shim does not call: `mars_rsp_*` (7), `mars_rdp_*` (6) and `mars_vi_scan_*` (8) [M:
  `grep -c 'extern "C" fn mars_'`].
- **36 WiseMan files name a shim type** (§18.5). WiseMan is not shipped, so the B4 gate's `grep` does not count them;
  each must still move before the shim goes.

### 18.2 Every behaviour above the ABI, and where it goes

The destinations, in order of preference, each later one widening a surface more:

- **host**: the generic adapter (`CoreEngine`, `CoreDebugBridge`, `CoreDebugTarget`) already does it, or does it with a
  change of its own that names no core; no surface changes.
- **cap**: an optional capability the core ABI v1 already defines, which MarsRT does not claim today.
- **info**: core info or machine info, a descriptor MarsRT writes; no ABI change.
- **pack**: the N64's system pack, `emusen-systems`' `n64` module and its C# facade `Crystal/Systems/N64/`; the pack's
  own interface and the platform library's C interface grow, the core ABI does not.
- **test**: kept as a `marsrt_core_*` extension (§4.7 of the core ABI page) reached only from a WiseMan fixture by
  name through `CoreLibrary.Export`, never from shipped code. A test is not the host, so the core ABI page's §2.4, as SR8
  closed it, is kept.
- **dead**: no caller anywhere, shown by `grep` over every `.cs`, `.rs`, `.py` and `.fs` file outside build output;
  deleted.
- **minor**: no v1 home; proposed in §18.4 as part of a core ABI minor, written "1.next" until approved.

**(a) The engine and its frame**

| # | What the shim adds | Read by | Goes to |
|---|---|---|---|
| A1 | `CoreName` is `"N64"` (`MarsRtCore.cs:75`) | the debugger's headings (`regs`, `cpus`, `spaces`), the session service's `engine` (`SessionService.Machine.cs:28`), the frame recorder's file names, Hotaru's title | **host**: core info's `name`, `"MarsRT"`, as every v1 core names itself (Nephrite's headings read `Nephrite`); a difference in the golden corpus (§18.6.3) |
| A2 | Width and height before a game: 640 × 480 (`:76–77`, `N64Console.cs:17–18`) | nothing in the factory's paths reads it between a core's creation and its load [A]; `MarsRtFrontendTests:366` reads the multiple after a load | **host**: 0 × 0 before a machine, as for every v1 core; after create the blank is `frame_info`'s, 640 × 480, as now |
| A3 | `DisplayAspect` is the television's 4:3 (`:79`) | Serenity's presenter | **info**: machine info's `aspect` and every `frame_info` already say 4/3 (`v1.rs:498`, `:557`), and `CoreEngine.DisplayAspect` reads them; the same value |
| A4 | `DisplayLines`: the console's lines in a picture drawn at a multiple (`:81`, `N64Console.cs:37`) | the CRT filter through `ICore.DisplayLines` (`EmuSen_CRT.md` §16), the frontend API's `display_lines` (`ManagedSession.Frame.cs:84`), the digest probe | **minor**: `frame_info` carries no such field; without one the generic adapter answers 0, and the CRT filter would draw 1,152 scanlines over a 576-line game at 2× (§18.4.2) |
| A5 | `FrameRateHz` from the last frame's cycles, 93,750,000 over `marsrt_core_counters[2]` (`:83`, `N64Console.cs:11`, `:14`) | every frontend's pacer, the load bars' budget (`CoreDebugTarget.cs:67`), the digest probe | **info**: machine info's `frame_rate`, which MarsRT writes as the region's nominal rate (`v1.rs:497`); measured from the digest files of `Mars_Native.md` §7.3 (`~/.cache/emusen/probe/mars/v1/digest-after2/`), the shim's rate alternates field by field, 50.016032 on 336 frames and 50.016005 on 262 of Ocarina of Time's 602 from power-on (Super Mario 64: 338 and 261), with 22.87 and 27.39 on the boot's cap-length frames. Recommended: machine info's rate is the VI's programmed field pair as an exact ratio (93,750,000 × 2 over the two fields' cycles), raised by `MACHINE_INFO` when the VI's timing registers change and the nominal rate before they are programmed (Q6) |
| A6 | `StateVersion` is `MarsMachine.StateKind`, 1 (`:87`) | the state records | **info**: machine info's `state.version`, `STATE_VERSION`, 1 |
| A7 | `ControllerPorts` is 4 (`:89`) | `ControllerPorts.Of` | **info**: machine info's four ports, which `ControllerPorts.OfEngine` already reads (`ControllerPorts.cs:17–22`) |
| A8 | `LastFramePhases` read into one array kept (`:395–410`) | `ManagedSession.Frame.cs:188–195` | **cap**: `PHASES` is claimed and machine info names the same five phases (`v1.rs:521`); the adapter's `PhaseMilliseconds` already allocates nothing |
| A9 | The frame-end work in Mars's order: cheats (gated, D1), freezes, the frame log, the breakpoints' frame, the periodic save (`:375–382`) | every frame | **host**: the runtime's order (`CoreEngine.cs:515–522`), which records the frame log *before* the cheats, so a logged address a cheat holds shows the game's value rather than the cheat's. The port's own order existed to match C# Mars (`EmuSen_NativeCores.md` §8.2's R4), retired as an oracle on 2026-10-16; recorded as a difference, not kept |
| A10 | `PeekAudioSamples` (`:413`) | `MarsRtDebugTarget.GetAudioSamples` | **cap**: `AUDIO_PEEK` is claimed; `CoreDebugTarget.GetAudioSamples` peeks the same way (`CoreDebugTarget.cs:122`) |
| A11 | A refused image becomes `InvalidDataException("MarsRT refused … as a Nintendo 64 image")`; the call stack is reset and the halt cleared (`:344–350`) | a frontend's message | **host**: `CoreRefusedException` with `status_text`'s words, "not a Nintendo 64 cartridge image" (`v1.rs:448`). The call stack: the adapter's bridge does not reset its `CallStack` on a new machine (`CoreDebugBridge.cs:39–48`), a candidate defect of every v1 core reloaded in one engine, to be shown by a test before anything changes (C2 of §18.3) |
| A12 | `Boot`: the corpus's hand-off with the hidden `Boot` setting, and no battery file while it runs (`:353–366`, `:341–342`) | `MarsRtTests:53` | **test**: the schema already carries `Boot` (create scope, hidden, `v1.rs:214`), so a test makes `new CoreEngine(library, { Boot = true, ExpansionPak = true })` with `BatteryRamDisabled`; no ABI change |

**(b) Settings**

| # | What the shim adds | Read by | Goes to |
|---|---|---|---|
| B1 | Typed properties over the settings: `IdleSkip`, `RspWhole`, `RenderScale`, `Antialiasing`, `Gpu`, `ThreadedRdp`, `RdpWorkers`, `DeferredPresentation`, `SkipRepeatedScans`, `UseBlocks`, `VerifyBlocks`, `BlockTier`, `VerifyRdp` (`:92–187`) | tests only (§18.5) | **host**: `Get` and `Set` by key |
| B2 | `VideoSettings`, the rows in C# with MarsRT's hints (`:196–209`), and `Settings` returning them | Mistress's Graphics Settings window through `ICoreSettings` | **info**: the schema (`v1.rs:149`), which `CoreEngine.Settings` already turns into rows. The two texts differ in one place [M]: `RdpWorkers`' help ends "…, as on Mars." in the schema (`v1.rs:156`) and not in the shim (`MarsRtCore.cs:204`); the schema's words are the ones that stay, with "as on Mars" taken out, since C# Mars is gone |
| B3 | The antialiasing note, a C# function of the values (`N64Console.cs:43–53`, rule at `:40`) | the settings window's row note | **cap**: `SETTING_NOTES`; MarsRT computes the same sentence where `EffectiveAntialiasing`'s rule already lives (`ffi/multiple.rs:8`). A note then needs a running machine; with none the window shows no note (Q15) |
| B4 | `Set` clamps a count to its range and refuses other text with `ArgumentException`; `Get` throws on an unknown key (`:211–228`) | `ManagedSession.ApplySetting`, tests | **host**: the adapter refuses anything outside the domain with `ArgumentException` (`CoreEngine.cs:909–912`), and `ApplySetting` then falls back to the default; `Get` of an unknown key answers `""`. A stored count past its range is clamped today and becomes the default; recorded |
| B5 | `ExpansionPak`: made `true` by the factory, and a change before the first frame rebuilds the machine at once (`:43–48`, `:64–73`) | `CoreFactory.cs:38`, then every frontend's `ApplyStored` | **host**: the factory's v1 arm already passes the stored create-scope values at construction (`CoreFactory.cs:33`, `StoredSettings`), which is what the rebuild existed to reach; the setter goes |
| B6 | A loaded state of the other RDRAM size sets `ExpansionPak` to match (`:427–435`) | `Get("ExpansionPak")` after a load | **host**, by retiring it: MarsRT raises `MACHINE_INFO` and rebuilds itself (`v1.rs:378–381`), machine info's `RDRAM` size states the machine, and the setting stays the player's choice for the next load (Q10) |

**(c) States and saves**

| # | What the shim adds | Read by | Goes to |
|---|---|---|---|
| C1 | A refused state is thrown again as a new `InvalidDataException` (`:432`) | `ManagedSession.States.cs:38` | **host**. The new exception does not carry the status the adapter's carried (`CoreMachine.cs:403`), so the frontend API's filter, `StatusOf(e) is not null`, cannot match it, and a refused MarsRT state is answered as a fault rather than `StateRefused`: a candidate defect read from the code (C1 of §18.3), removed by the generic adapter, to be shown by a test first |
| C2 | `LoadState(ReadOnlySpan<byte>)`, `Save(bool)` (`:438–453`) | tests | **host**: `LoadState(Stream)`, `SaveState`, `SaveSnapshot` (`ISnapshotCore`) |
| C3 | `BatteryContents` (`:338`) | `BatterySaveTests:320`, `MarsRtFrontendTests:531` | **host**: `Machine.Battery(0)` |
| C4 | `MarsMachine`: a machine for the state alone, `marsrt_core_load_fields`, `marsrt_core_state_kind`, `Describe` (`MarsMachine.cs:10–80`) | `MarsNativeStateTests`, `MarsRtRewindTests`, `MarsRtThreadsTests`, `NativeHostTests`, `RustStateHeaderTests` | **test** for the two exports; `Describe` repeats `status_text` and the crate's codes, so the fixture uses `CoreLibrary.Words` (one owner) |
| C5 | `SaveProcessor`, `marsrt_core_save_cpu` (`:456–462`) | none [M: no `.cs` outside the shim names it] | **dead** |

**(d) Cheats**

| # | What the shim adds | Read by | Goes to |
|---|---|---|---|
| D1 | **The frame end applies cheats only while COP0's `Status.IE` is set** (`:377`, `Cop0` at `:270`, `marsrt_core_cop0`), Project64's rule, which keeps a cheat restored at power-on from breaking the boot code's checksum (`Mars_Cheats.md` §5.1: Super Mario 64 spins at `0x800001CC` without it) | every frame with a cheat on | **minor** (§18.4.4). v1's `CHEAT_POKES` was meant for this core (§6.12 of the core ABI page), but the host sends it only one-byte `Set`s (`CoreEngine.cs:861`): of Project64's 21,103 N64 code lines, the 16-bit writes (38.2%), the tests (7.4%) and the repeaters (4.2%) would still be applied by the host, ungated (`Mars_Cheats.md`'s census) |
| D2 | The cheat spaces by name, case-insensitively, a read past the end zero (`:231–245`) | the cheat registry | **host**: `CoreEngine.SpaceOf` matches as case-insensitively; past the end is MarsRT's own rule in `space_read` |
| D3 | `MarsCheatCodecs.GameShark()` and `Pair()` (`Cheats/MarsCheatCodecs.cs:9`, `:12`) | `CheatCodecAdapterTests`, `MarsCheatTests` | **pack**: `N64System.GameShark()`, which the factory already uses (`CoreFactory.cs:195`); the alias goes |

**(e) Input**

| # | What the shim adds | Read by | Goes to |
|---|---|---|---|
| E1 | **A controller plugged in or pulled out**, `SetControllerConnected` over `marsrt_core_set_present` (`:369–372`) | `EmulatorSession.cs:33`, the frontend API's `connected` (`ManagedSession.cs:233`), Endymion's router on every poll (`EmuSen_Input.md` §8.6) | **minor** (§18.4.3). The N64's games tell an empty port from a full one; `ICore.SetControllerConnected` defaults to nothing (`ICore.cs:56`) and the adapter does not override it, so without a v1 home a second player's controller would never be present |
| E2 | `RunSteps` sends the held buttons before it steps (`:416–422`) | `MarsRtTests`, `MarsRtDebugTests` | **test**, with `marsrt_core_run_steps`; the fixture sends the latch as `SendInput` does |

**(f) The debugger**

| # | What the shim adds | Read by | Goes to |
|---|---|---|---|
| F1 | Registries owned by the core: watches, frame log, breakpoints, both coverages, the call stack, labels (`MarsRtCore.Debug.cs:39–45`), so a breakpoint or label outlives a prompt (`Mars_Debug.md` §1) | every debug command | **host**: the adapter owns watches, frame log and breakpoints, the bridge the coverages and call stack; labels are the target's (`CoreDebugTarget.cs:178`), and a target lives as long as its bundle, which the factory makes per load, as the shim's registries in practice did |
| F2 | The observed loop: tables pushed, `debug_run_frame`, logs drained (`MarsRtCore.Debug.cs:73–200`) | every armed frame | **host**: `CoreDebugBridge` does the same over the same exports, with two differences, F12 and F13 |
| F3 | The CPU's registers, `marsrt_core_cpu_registers`: PC, the 32 GPRs, HI, LO and the cycle count, named by `MarsOperands.Gpr` (`MarsRtDebugTarget.cs:123–132`) | `regs`, `eval`, `setreg`'s list, conditions, Mistress's debugger | **cap**: `DEBUG_REGISTERS`, as `EmuSen_DianaSh.md` Q28 asked of the other ports: the 36 names in machine info's `processors[0].registers` (empty today, `v1.rs:515–518`) and `debug_registers` giving the values |
| F4 | The RSP's registers, `marsrt_core_rsp_registers`: PC (16 bits), halted (1), the 32 scalars; *broke* is read and not shown (`:134–140`) | `regs`, `regs rsp`, `cop` | **cap**: `DEBUG_REGISTERS` for processor 1, the 34 shown today |
| F5 | The VI's fourteen registers by their libultra names, `marsrt_core_vi_registers` (`:17–20`, `:142–147`) | `regs`' video section, `eval`, the summary | **info** and **pack**: a read-only `view` space `VI` of 56 bytes, the registers big-endian as stored (MarsRT's machine info), and the names the pack's (`ISystemDebugViews.VideoRegisters(spaces)`, a new member with its Rust twin, as §31.2 of the core ABI page reads the SNES's spaces by name). No ABI change; `spaces` lists one more space (§18.6.3) |
| F6 | Processor descriptions "VR4300 (MarsRT)" and "RSP (MarsRT)" (`:74`, `:82`) | `cpus` | **info**: machine info's names, `CPU` and `RSP`, which the adapter lowercases into the words `cpu` and `rsp` the commands take; the descriptions change in `cpus` (§18.6.3) |
| F7 | **The RSP cannot halt**: its breakpoints are a registry never pushed, and `CanHalt` is false (`:25–26`, `:88`), since the RSP runs inside the bus's clock (`Mars_Debug.md` §5) | `bp rsp`, `step rsp`, `cpus`' "supports" | **host**: the crate's default answers `NOT_SUPPORTED` to `debug_set_breakpoints` for any processor but 0 that a core does not take (`emusen-native/src/core/mod.rs:198–206`), and the bridge ignores the answer (`CoreDebugBridge.cs:154`). The target asks once per machine, with an empty table, and marks a refusing processor as one that cannot halt; the core ABI page's §6.14 says so in a clarification (Q16). Without it, `bp rsp add` would be accepted and never hit |
| F8 | **Disassembly by space**: the VR4300 for RDRAM (at KSEG0), the ROM (at `0xB0000000`), PIF RAM (at `0xBFC007C0`) and the processor's view; the RSP for IMEM and DMEM (`:152–188`); static references (`:170–175`) | `disasm`, `callers`, `trace`, the call graph | **pack**: MarsRT claims no `DEBUG_DISASSEMBLE`, and the adapter's target disassembles nothing without it (`CoreDebugTarget.cs:238`). The VR4300's table is already `emusen-mips`' and reached through the pack's `n64::disasm` (§13.5); the RSP's moves from `RspDisassembler.cs` into `emusen-mips` as a dialect beside it, the item §13.6 left for this step, by B3's method (a parity class against the C# before the C# goes). The target then asks the pack when the core does not decode (Q7) |
| F9 | **A virtual address's physical place**, `addr`: the TLB walked without faulting (`marsrt_core_physical`), then the physical map named (`MarsDebugSpaces.cs:23–44`) (`:96`, `MarsRtCore.Debug.cs:252–263`) | `addr`, `DebugPlumbingTests:58–60` | **minor** for the walk (§18.4.5): only the core holds the TLB. **pack** for the map: the N64's physical map is the console's, whichever engine runs it, read with the RDRAM space's size |
| F10 | The summary: frame, console time from the cycle count, RDRAM size, the CPU's PC, SP and RA, the VI's origin, width and sync, and the picture's size (`:215–226`) | `summary`, Mistress's debugger | **pack**: `ISystemDebugViews.Summary` given the spaces as well as the registers, so the N64's lines come from the CPU's registers (the cycle count is register 35, F3) and the `VI` space; the head line becomes the adapter's (§18.6.3) |
| F11 | Tiles and tilemaps refused with the N64's words (`:200–204`) | `tile`, `tilemap` | **host**: the adapter answers empty, and `tile` says the core decodes no tiles (§30.4 of the core ABI page) |
| F12 | A watch's log line names the store's PC with eight digits, `PC=A4000040` (`MarsRtCore.Debug.cs:161`) | `watch log` | **host**: the bridge writes `PC=$A4000040` (`CoreDebugBridge.cs:205`, `X4` pads but does not cut); one character differs |
| F13 | An interrupt noted from the stop reason, `INTERRUPT` (16), with no frame pushed (`MarsRtCore.Debug.cs:36`, `:91`), since MarsRT's loop stops for an interrupt without pushing a call (`Mars_Native.md` §7.2) | `bp irq`, the call stack | **host**: the bridge notes interrupts only from call kinds 2 to 5 (`CoreDebugBridge.cs:225–227`); it notes one on the stop reason as well, for any core, the reason being v1's (`emusen-native/src/debug.rs:20`) |
| F14 | The frame log reads a value big-endian (`MarsRtCore.Debug.cs:227–237`) | `framelog` | **pack**: the N64 is big-endian whichever engine runs it; the adapter reads little-endian (`CoreEngine.cs:893–898`) and takes the order from the system's pack (Q13) |
| F15 | Expression symbols: every register of the three providers, the VI's and the RSP's among them (the golden `eval symbols` lists `BURST`, `CONTROL`, `Halted`) | `eval`, conditions | **host**: the adapter's context is the main processor's registers, the pack's symbols and `stackdepth` (`CoreDebugTarget.cs:182–190`); the VI's names come back as pack symbols over the `VI` space, the RSP's do not (§18.6.3) |
| F16 | Test hooks: `WriteBus32`, `ReadBus32`, `SetCop0`, `RaiseInterrupt` with `MiInterrupt`, `StepRsp`, `DebugCounterValues` (`MarsRtCore.Debug.cs:9–19`, `:266–291`) | `MarsDebugRig`, `MarsCheatTests`, `MarsDebugTests`, `MarsRtDebugTests`, `CoreFactoryCheatWiringTests`, `MainWindowSaveStateThreadTests` | **test**, with `marsrt_core_bus_write32`, `_bus_read32`, `_set_cop0`, `_mi_raise`, `_rsp_step`; `marsrt_core_debug_counters`' third value is telemetry (H4) |

**(g) The console's facts with no library loaded (`N64Console`)**

| # | What the shim adds | Read by | Goes to |
|---|---|---|---|
| G1 | `Ports`, `PadButtons`, `PadAxes` (`N64Console.cs:20`, `:28–34`) | `CoreCatalog.cs:203`, `:242`, `ControllerPorts.cs:43`, the bindings windows | **info**: core info's `n64.pad` (`v1.rs:246`), read from the sidecar as `CoreCatalog.DiscoveredSystem` already reads a discovered console's (`CoreCatalog.cs:207–210`, `:245–248`); §5.3 makes controllers core info's. The buttons' order is the same, `PAD`'s (`v1.rs:30–45`) |
| G2 | `VideoSettings` (`:56–66`) | `CoreCatalog.SettingsFor("N64")` (`:283`) | **info**: the sidecar's schema (`CoreSidecarSettings`, `CoreCatalog.cs:336–337`); §5.3 retires `SettingsFor`'s rows |
| G3 | `ProcessorClockHz`, `CycleCap` (`:11`, `:14`) | `FrameRateHz` (A5) | **info**, with A5 |
| G4 | `ScreenWidthPixels`, `DefaultScreenHeight` (`:17–18`) | A2, `MarsRtFrontendTests:366` | with A2 |
| G5 | `PakExtension` (`:23`) | `BatterySaveTests:311`, `:323` | **info**: core info's battery suffixes (`v1.rs:61–62`) |
| G6 | `CButtonThreshold` (`:26`) | a comment in Mistress | **info**: `set_axis` in MarsRT (`v1.rs:26`); the comment's reference is moved |
| G7 | `LinesOf` (`:37`) | A4 | with A4 |
| G8 | `EffectiveAntialiasingOf`, `AntialiasingNote` (`:40–53`) | B3, `MarsCoreSettingsTests` | with B3 |

**(h) Counters and text**

| # | What the shim adds | Read by | Goes to |
|---|---|---|---|
| H1 | `marsrt_core_counters`: cycles, frames, the last frame's cycles, instructions, the idle loop's turns and instructions (`v1.rs:765–771`) | `FrameRateHz` (A5), the summary (F10), `MarsDebugTests:203–223`, `MarsRtDebugTests:97`, `MarsRtTests:94` | **minor**, `TELEMETRY` (§18.4.1); the cycle count is also register 35 (F3) |
| H2 | `marsrt_core_thread_counters`: the display processor's thread, 38 values (`ffi/threads.rs:8`, `:29–55`) | `MarsRtRewindTests:290`, `MarsRtThreadsTests:87` | **minor**, `TELEMETRY` |
| H3 | `marsrt_core_block_counters`: the recompiler's 13 (`cpu/blocks/mod.rs:260–281`) | none outside the shim's `BlockCounterValues` [M] | **minor**, `TELEMETRY`, since `Mars_Recompiler.md` reads them when measuring; or **dead** if the tester prefers (Q1) |
| H4 | The third value of `marsrt_core_debug_counters`, the exceptions entered (`v1.rs:972–977`; the first two are v1's `debug_counters`) | `MarsRtDebugTests:97` | **minor**, `TELEMETRY` |
| H5 | `marsrt_core_gpu_report`: what the device setting got, a sentence when it got nothing (`ffi/multiple.rs:29–32`) | test output and the digest probe only | **cap**: `SETTING_NOTES` for `Gpu` when the setting is not honoured, the sentence §6.13 of the core ABI page made notes for; and a log line naming the device when it is (`log_drain`, v1). No new export (Q9) |
| H6 | `marsrt_core_is_viewer_text`: the IS-Viewer's transcript, a game's or test ROM's debug output (`v1.rs:803–805`) | `MarsRtTests:59`, the corpus's summary | **host**: the core's log, a line for each newline written, through `log_drain` and `EVENT_LOG` (v1); the corpus test reads the log. *Corrected 2026-10-18 (§18.9):* the transcript is the harness's (`Skip`), in no state, so a load does not replay it (Q8) |

**(i) The loader and the component exports**

| # | What | Read by | Goes to |
|---|---|---|---|
| I1 | `MarsNative`: `PortLibrary` at the fixed name, `EMUSEN_MARS_NATIVE=0` to turn it off, and the 21 required extensions (`MarsNative.cs:8–17`) | the factory, `NativeHostTests:36` | **host**: discovery's sidecar (`marsrt.core.json`), as VenusRT is found (`CoreCatalog.SoleDiscoveredEngines`, `:348–351`); the switch goes, since nothing is left to fall back to (Q14) |
| I2 | `MarsDebugSpaces`' names and `MarsRtSpace`'s numbers (`MarsDebugSpaces.cs:8–15`, `MarsRtCore.cs:14`) | the shim, tests | **info**: machine info's spaces (`v1.rs:22`, `:501–514`), one owner; the cheat space's name stays the pack's (`N64System.Rdram`) |
| I3 | `mars_rdp_*` (6): "a test-only C ABI … for the differential against the C# `Rdp`" (`ffi/rdp.rs:1`) | none [M]; the C# `Rdp` was deleted on 2026-10-16 | **dead** |
| I4 | `mars_rsp_*` (7): "the C ABI C# calls" (`rsp/mod.rs:1408`) | only `NativeHostTests:39`, which asserts that `mars_rsp_new` exists | **dead**, with that assertion |
| I5 | `mars_vi_scan_*` (8) | `MarsRTViScan` for `MarsViDifferentialTests`' angrylion differential | **test** |

**What the inventory says of P3** (§9). P3 predicted that everything a shim adds fits a capability v1 already defines,
except MarsRT's counters. For MarsRT it does not hold: beside the counters, four behaviours have no v1 home and are
reached by players, not only by tests: the lines of a picture at a multiple (A4), a controller plugged in (E1), the
cheat gate (D1) and the TLB walk behind `addr` (F9). Everything else goes to a capability v1 has (`DEBUG_REGISTERS`,
`SETTING_NOTES`, `AUDIO_PEEK`, `PHASES`), to descriptors, to the N64's pack, to the generic adapter, to a test fixture,
or is dead.

### 18.3 Candidate defects read on the way, none yet shown

Each is to be shown by a test that fails before anything is changed, or recorded as no defect, as §3.12 asks.

- **C1. A refused MarsRT state is a fault to the frontend API.** `MarsRtCore.LoadState` (`:432`) throws a new
  `InvalidDataException` without the status the adapter attached (`CoreMachine.cs:403`), so `ManagedSession.LoadState`'s
  filter (`ManagedSession.States.cs:38`) does not match, and `ManagedCrystal.Fail` answers `SessionAbi.Fault` with the
  exception's type in the words, where every other v1 core answers `StateRefused`. The test: a truncated state loaded
  through the API on each core.
- **C2. The bridge's call stack outlives a reload.** `CoreDebugBridge.Attach` keeps `CallStack` as it was
  (`CoreDebugBridge.cs:39–48`); the shim reset its own on every load (`MarsRtCore.cs:348`). A v1 engine reloaded in
  place (a game loaded again into the same `CoreEngine`) would show the last game's frames in `bt`. The test: two loads
  into one engine with a call recorded before the second.
- **C3. An interrupt breakpoint never fires on a core that stops for an interrupt without pushing a frame.** F13: the
  bridge notes interrupts only from call kinds. The test: `bp irq` on the generic adapter over MarsRT, which the shim
  passes today.

### 18.4 The proposal: what the core ABI needs, in one minor, "1.next"

**The form, weighed as the project's rule asks.** Each item below was weighed as an **extension** of an existing
surface, a **module** (a pack, a host change, a test fixture) or a **component of its own**. None is a component of its
own: each is a fact or an operation of the emulated machine that only the core has, so it cannot live in a module
above the ABI without per-core code, which SR8 forbids, and it is too small to be a library. Each is therefore an
**extension of the core ABI**, an additive minor under the core ABI page's §4.2: optional, claimed by a bit or stated by a field whose
zero means "absent", safe never to be used (its §4.6), so an older host drives MarsRT as today and a newer host drives an
older core as today. They are proposed together because they land together in one core's step and the kit's cases are
written once; each is claimed on its own, and the tester may take any of them without the others (Q1 to Q5). **No
number is taken**: "1.next" is assigned when the proposal is approved, since MoonRT's step may propose a minor too.

#### 18.4.1 `TELEMETRY`: a core's own counters, named in machine info

D11's capability. **What it is for**: a dashboard, `coretop`, a test or a benchmark reads what the machine did (cycles,
instructions, a thread's waits, a recompiler's hits) without per-core code. **What it is not for**: text (H5 and H6 have
homes), registers (`DEBUG_REGISTERS`), or anything a frame depends on.

```c
#define EMUSEN_CAP_TELEMETRY (1ull << NN)   /* @since 1.next: emusen_core_telemetry, machine info's "telemetry" */

/* @since 1.next, TELEMETRY: the values of machine info's "telemetry" entries, in their order, as of the last call that
 * ran the machine (advance, present, debug_run_frame, reset, state_load); the length-query idiom: copies
 * min(len, count) values and returns the count. Never allocates, never changes the machine; on the machine's thread,
 * between calls, as phases. */
int64_t emusen_core_telemetry(const emusen_machine *machine, int64_t *out, size_t len);
```

Machine info gains one field, read once at create and again on `MACHINE_INFO`:

```json
"telemetry": [
  { "name": "cpu.cycles",       "label": "Processor cycles",            "unit": "cycles",       "kind": "total" },
  { "name": "cpu.instructions", "label": "Instructions",                "unit": "instructions", "kind": "total" },
  { "name": "frame.cycles",     "label": "Cycles in the last frame",    "unit": "cycles",       "kind": "frame" },
  { "name": "rdp.workers",      "label": "Rasteriser threads running",  "unit": "count",        "kind": "level" }
]
```

- `name`, `[a-z0-9_.]+`, stable for the core's major; `label`, for a person; `unit`, an open string (`cycles`,
  `instructions`, `ns`, `bytes`, `count`); `kind`, open: `total` (counts up from create; a state load or reset may set
  it either way, so a host differences two reads with neither between), `frame` (the last frame's), `level` (as it
  stands now). Absent, the core has none.
- **The list is fixed for a machine**: a counter of a part that is off reads 0 (MarsRT's thread counters already do,
  `ffi/threads.rs:31`), so a host's buffer is sized once per machine.
- **MarsRT's list**, 58 values: the six of `marsrt_core_counters`, the 38 thread counters, the 13 recompiler
  counters and the exceptions entered; the names are written from the comments that order them today
  (`MarsRtCore.cs:189`, `:192`; `ffi/threads.rs:27–28`).
- **Other cores**: VenusRT could publish each processor's instructions and DMA's bytes, Diamond its GTE's operations,
  Nephrite its Z80's; a core that claims nothing is never asked and machine info carries no field. The kit adds a case:
  the export and the bit agree (C2's rule), the count equals the list's length, a call changes no state (C7's
  comparison), and a `total` never moves between two calls with no machine-running call between.
- **The host**: `CoreEngine.Telemetry(Span<long>)` beside `PhaseMilliseconds`, and a provider on `ICoreTelemetry`
  (`Counters`, default empty, so the C# targets and fakes need nothing) filled into a `RingProvider` as the registers
  are (§14.5), refreshed by `RefreshProviders` only; the session service's `telemetry` gains a kind for it in its next
  minor, and `coretop` lists them under the load bars.
- **Cost a frame: none unless read.** Nothing is computed for the export: the values are fields MarsRT keeps already
  (atomics for the threads, read `Relaxed`). Read, it is one call copying 58 `int64_t`s into a buffer the host
  made once; no allocation on either side (the recompiler's `counters()`, which returns a `Vec` today, is written into
  the caller's slice instead).

**Alternatives weighed.** (a) Keep the counters as `marsrt_core_*` extensions read only by tests: no ABI change, and
nothing shipped reads them today [M]; but `coretop` and the session service could never show them, and every core that
wants the same would add private exports a host must not use. (b) Fold them into `phases`: phases are nanoseconds a
phase a frame, and counts of other units in that array would make its names lie. (c) A JSON export: text on a path a
dashboard reads every frame, against §2.5 of the core ABI page. (d) Counters as a memory space: a space is the
machine's memory, and a counter of host threads is not. Recommended: the export and the field, as above.

#### 18.4.2 `frame_info.lines`: the console's lines in a picture drawn at a multiple

```c
typedef struct emusen_frame_info {  /* 64 bytes since 1.next; 56 accepted */
    /* ... the fields of 1.0, unchanged ... */
    int64_t bytes;
    int32_t lines;     /* @since 1.next: the console's lines in this picture where it is drawn at a whole multiple of
                        * them; 0, its rows are its lines (ROW_REPEAT's repeats aside) */
    uint32_t reserved2;
} emusen_frame_info;
```

Appended, as the core ABI page's §4.2 grows a struct, so a core that writes 56 bytes leaves it zero in a host's zeroed struct, and zero
means what `ICore.DisplayLines`' zero means today. MarsRT writes `height / (width / 640)` at a multiple and 0
otherwise, the shim's rule (`N64Console.cs:37`). Cost: one more field in a struct the host already reads; none a
frame. Other cores: any that draws above its console's resolution (an enhancement setting) states its lines the same
way; the rest write nothing. The kit: `lines` is 0 or divides `height`.

*Weighed:* a machine-info field (the multiple changes with a run-time setting, so it would need an event on every
change and is a picture's fact, not a machine's); the host computing it from `RenderScale` (per-core code); repurposing
`reserved` inside the struct (the core ABI page's §4.2 grows a struct only at its end).

#### 18.4.3 `PRESENCE`: a controller in a port, or none

```c
#define EMUSEN_CAP_PRESENCE (1ull << NN)    /* @since 1.next: emusen_core_set_present */

/* @since 1.next, PRESENCE: a controller in `port` (present 1) or none (0), seen by the game as the console's own
 * hardware shows it; machine info's ports say which controller. Part of the state where the console's own hardware
 * keeps it. A port past machine info's is EMUSEN_NO_SUCH_PORT. */
int32_t emusen_core_set_present(emusen_machine *machine, uint32_t port, uint32_t present);
```

What MarsRT's `marsrt_core_set_present` does today (`v1.rs:1006–1013`), under the common prefix, as the core ABI page's §4.7
says an extension that becomes common is moved. The host's `CoreEngine.SetControllerConnected` sends it through the input
latch, as a button reaches the machine as the next frame starts (`ControllerPortTests`' rule, `Mars_Native.md` §7.5).
Cost: Endymion's router sends the flags on every poll; the latch sends only a change, a comparison a frame and no
allocation. Other cores: a console whose games can tell (the N64, a PlayStation's memory-card-and-pad port, the
Saturn's) claims it; the rest do not and are never called. *Weighed:* a `pad<n>` setting as Nephrite's controllers use
(a setting is the player's stored choice, while presence changes on every disconnect and is carried in the state); the
general `set_device` that `EmuSen_CoreAPI_Extensions.md` §10.1 foresaw (it would duplicate the controller choice the
settings already make; narrower is better until a device kind needs it).

#### 18.4.4 `EMUSEN_FRAME_CHEATS_HELD`: the frame at whose end the console's cheat devices hold off

```c
#define EMUSEN_FRAME_CHEATS_HELD 2u   /* @since 1.next, frame_info.flags bit 1: a host applies no cheat of its own after
                                       * this frame; a core that claims CHEAT_POKES holds its own likewise */
```

MarsRT sets it when `Status.IE` is clear at the frame's end (D1). The adapter reads `frame_info` once after each
`advance` and skips its cheats for that frame; an explicit `ApplyCheats`, Mistress's Apply, stays ungated, as today
(`Mars_Cheats.md` §5.1). Cost: one call a frame returning a struct, no allocation. Other cores: none sets it today; a
core whose references gate cheats on a machine state may. *Weighed:* `CHEAT_POKES` alone (half the census's lines stay
ungated, D1); a richer `CHEAT_POKES` carrying tests, widths and repeats (the runtime's whole cheat model across the ABI,
against "codecs stay out of the ABI", §6.12 of the core ABI page); dropping the gate (the boot's checksum breaks, a
defect already measured).

#### 18.4.5 `DEBUG_TRANSLATE`: a processor's address on its bus

```c
#define EMUSEN_CAP_DEBUG_TRANSLATE (1ull << NN)   /* @since 1.next, with DEBUG: emusen_core_debug_translate */

/* @since 1.next: the address processor `processor`'s bus sees for `address`, as its own translation gives it (a TLB
 * consulted, never faulted, no state changed): 1 with *bus written, 0 where nothing maps it, or a status. */
int32_t emusen_core_debug_translate(const emusen_machine *machine, uint32_t processor, uint32_t address, uint32_t *bus);
```

MarsRT's `marsrt_core_physical` under the common prefix; the pack names the place (F9). Cost: none a frame; asked by a
command. Other cores: VenusRT's "physical address of a CPU address" (§30.6's item 11 of the core ABI page, an
extension there) and Diamond's segments take the same call. *Weighed:* the TLB as a view space the pack walks (two
owners of the VR4300's TLB match); `addr` answering only KSEG0 and KSEG1 by arithmetic in the pack (a mapped address,
which is where games run their code after `osMapTLB`, would be unresolved).

### 18.5 The tests that cover the shim today, and their baseline

**36 WiseMan files name a shim type** [M: `grep -w` for the shim's ten type names], and three more reach it through
`MarsDebugRig` (`MarsDebugTests`, `BreakpointConditionTests`, `BreakpointConditionCostBench`). What each reads beyond
`ICore`:

| File | Shim types named | Shim members read |
|---|---|---|
| `Cores/AudioBufferLimitTests` | `MarsRtCore` | the audio peek |
| `Cores/BatterySaveTests` | `MarsRtCore`, `N64Console` | `BatteryContents`, `PakExtension` |
| `Cores/BreakpointConditionTests`, `BreakpointConditionCostBench` (opt-in) | `MarsDebugRig` | through the rig |
| `Cores/CheatCodecAdapterTests` | `MarsCheatCodecs`, `MarsDebugSpaces` | the codec alias, the space names |
| `Cores/ControllerPortTests` | `MarsRtCore` | `ControllerPorts` |
| `Cores/CoreFactoryCheatWiringTests` | `MarsRtCore`, `MarsRtSpace`, `MarsDebugSpaces` | `Peek`, `Poke`, `ReadBus32` |
| `Cores/CoreOutputHashProbe` (opt-in) | `MarsRtCore` | `LoadState` |
| `Cores/CoreRegistrationEquivalenceTests` | `MarsRtCore`, `MarsNative` | the engine row, `FrameRateHz` |
| `Cores/DebugPlumbingTests` | `MarsDebugSpaces` | `Resolve` |
| `Cores/EngineFeaturesTests` | `MarsRtCore` | the engine's interfaces |
| `Cores/MarsCheatTests` | `MarsRtCore`, `MarsRtDebugTarget`, `MarsRtSpace`, `MarsCheatCodecs`, `MarsDebugSpaces` | `SpaceSize`, `Peek`, `Poke`, `ViRegisters`, `WriteBus32`, `ReadBus32`, `Pc` |
| `Cores/MarsCoreSettingsTests` | `MarsRtCore`, `N64Console` | the typed settings, `EffectiveAntialiasing`, `VideoSettings`, the note |
| `Cores/MarsDebugTests` | `MarsDebugRig` | `Instructions`, `RspCoverage`, `Listening`, `StepRsp`, the bus hooks |
| `Cores/MarsDisassemblerTests` | `RspDisassembler` | the RSP's decode |
| `Cores/MarsNativeStateTests`, `NativeHostTests`, `RustStateHeaderTests` | `MarsMachine`, `MarsNative` | the state handle, the loader |
| `Cores/MarsNativeTests` | `MarsNative` | the loader |
| `Cores/MarsRtCoreAbiTests` | `MarsRtCore`, `MarsNative` | `VideoSettings`; it already runs the generic adapter over MarsRT beside the shim |
| `Cores/MarsRtDebugTests` | `MarsRtCore`, `MarsRtDebugTarget`, `MarsDebugRig` | `RunSteps`, `Instructions`, `RspCoverage`, `DebugCounterValues` |
| `Cores/MarsRtDigestProbe` (opt-in) | `MarsRtCore` | `GpuReport`, `DisplayLines` |
| `Cores/MarsRtFrontendTests` | `MarsRtCore`, `MarsRtDebugTarget`, `MarsRtSpace`, `N64Console`, `MarsDebugSpaces` | the typed settings, `GpuReport`, `SpaceSize`, `Peek`, `Cop0`, `CpuRegisters`, `BatteryContents`, `ExpansionPak` |
| `Cores/MarsRtRewindTests`, `MarsRtThreadsTests` | `MarsRtCore`, `MarsMachine` | `ThreadCounterValues`, `GpuReport`, `Save` |
| `Cores/MarsRtTests` | `MarsRtCore` | `Boot`, `RunSteps`, `IsViewerTranscript`, `IdleTurnsPassed`, `IdleSkip`, `RspWhole` |
| `Cores/MarsViDifferentialTests` with `Fixtures/MarsRTViScan` | `MarsNative` | `mars_vi_scan_*` |
| `Cores/RewindToMomentTests` | `MarsRtCore` | `Save`, `LoadState` |
| `Crystal/N64PakFormatTests` | `MarsNative` | the library, for a machine of its own |
| `Crystal/ManagedCrystalTests` | `MarsRtCore`, `MarsRtDebugTarget` | `CpuRegisters` |
| `Fixtures/MarsDebugRig` | `MarsRtCore`, `MarsRtDebugTarget`, `MarsRtSpace`, `MiInterrupt` | every test hook (F16) |
| `Input/ConsoleBindingsTests` | `MarsRtCore` | the pad |
| `Mistress/MainWindowSaveStateThreadTests` | `MarsRtCore` | `CpuRegisters`, `ReadBus32` |
| `Mistress/MarsRtEngineTests` | `MarsRtCore`, `MarsRtSpace` | `ThreadedRdp`, `DeferredPresentation`, `Peek` |
| `Mistress/MultiplayerTests` | `MarsRtCore` | `ControllerPorts`, presence |
| `Mistress/ResumeWhereYouLeftOffTests` | `MarsRtCore` | `LoadState` |
| `Serenity/DisplayAspectTests`, `ScreenFilterRenderTests` (its area part) | `MarsRtCore`, `N64Console` | `DisplayLines`, the aspect |

Beside them, three classes reach MarsRT without naming a shim type and are in the step's blast radius:
`GoldenCorpusTests` and `SessionServiceParityTests` (the `marsrt` golden target, `Fixtures/GoldenTargets.cs:51`, `:66`)
and `Mistress/ActiveCheatsWindowTests` (the N64's GameShark, through `RuntimeGame`). The Mistress windows that read `CoreCatalog`'s N64 rows
(`GraphicsSettingsWindowLayoutTests`, `ControllerBindingsDiagramTests`, `InputSettingsWindowTests`) join it at step 6.

**The baseline, run once on 2026-10-18** at `480bf5e2` with the branch's own build, in the default environment (no
`EMUSEN_MARSRT_STATES`, no commercial ROM copies, so the real-game cases return early as they are written to), under
the shared test lock:

| Run | Classes, tests passed |
|---|---|
| First, 902 passed of 902, 10 min 30 s | `AudioBufferLimitTests` 6, `BatterySaveTests` 17, `BreakpointConditionTests` 12, `CheatCodecAdapterTests` 8, `ConsoleBindingsTests` 16, `ControllerPortTests` 4, `CoreFactoryCheatWiringTests` 13, `CoreRegistrationEquivalenceTests` 3, `DebugPlumbingTests` 3, `EngineFeaturesTests` 5, `GoldenCorpusTests` 100, `ManagedCrystalTests` 6, `MarsCheatTests` 75, `MarsCoreSettingsTests` 12, `MarsDisassemblerTests` 550, `MarsNativeStateTests` 4, `MarsNativeTests` 1, `MarsRtCoreAbiTests` 4, `MarsRtDebugTests` 33 (with `MarsDebugClaims`' cases, which it inherits), `MarsRtTests` 2, `N64PakFormatTests` 5, `NativeHostTests` 5, `RewindToMomentTests` 3, `RustStateHeaderTests` 3, `SessionServiceParityTests` 12 |
| Second, 443 passed of 443, 3 min 57 s, the GPU cases on the RX 6800 alone | `ActiveCheatsWindowTests` 25, `ControllerBindingsDiagramTests` 24, `DisplayAspectTests` 13, `GraphicsSettingsWindowLayoutTests` 14, `InputSettingsWindowTests` 23, `MainWindowSaveStateThreadTests` 3, `MarsRtEngineTests` 8, `MarsRtFrontendTests` 16, `MarsRtRewindTests` 16, `MarsRtThreadsTests` 15, `MarsViDifferentialTests` 151, `MultiplayerTests` 11, `ResumeWhereYouLeftOffTests` 8, `ScreenFilterRenderTests` 116 (the whole partial class, its area part among them) |

**1,345 tests, none failed, none skipped.** The opt-in probes and benches (`MarsRtDigestProbe`, `CoreOutputHashProbe`,
`BreakpointConditionCostBench`) are not in the count; they run only with their variables set, and the digest probe is
step 1's (§18.7).

### 18.6 The gates for the second phase

The B4 row's gates (§6), made particular to MarsRT. Each is run before the first change (step 1 of §18.7) and again
after the last; a result that differs is listed with its cause and decided, never re-recorded silently.

#### 18.6.1 MarsRT's goldens and corpus

- **The digest probe** (`MarsRtDigestProbe`, `EMUSEN_MARSRT_DIGEST_OUT` and `_GAMES`), Super Mario 64 and Ocarina of
  Time only (the probe's two graded games), the 26 runs `Mars_Native.md` §7.5 left and the two GPU runs on the RX 6800
  alone: every picture, every drain of sound, every state and its length, frame by frame, equal to the baseline
  recorded at step 1. It drives MarsRT through `ICore`, `ICoreSettings`, `ISnapshotCore`, `IFrameSerial` and
  `IRepeatedRows` only, so the same file runs on the shim and on the adapter, once its two shim reads (`GpuReport`,
  `DisplayLines` through the shim's type) read the setting note and `ICore.DisplayLines`. Expected differences, each to
  be confirmed and no other accepted: the rate column, if Q6 is answered as recommended; and none in `lines` once
  `frame_info.lines` (§18.4.2) is in.
- **The N64 test ROM corpus**: n64-systemtest's summary, *Failed 46 of 4637 tests*, and the recompiler's transcript line
  for line the interpreter's (`MarsRtTests`), read from the log if Q8 says so; the crate's own corpus test.
- **The crate's tests**, `cargo test --profile test-checked` in `MarsRT - N64/`, the count at step 1 and every test
  passing after, with the new ones listed; the GPU tests on the RX 6800 alone, one run at a time.
- **The kit**: `emusen-core-conform` on copies of the two games, 600 frames, every case passing (today 19, C16 not
  applying, `Mars_Native.md` §7.4), with the minor's new cases; and `MarsRtCoreAbiTests`' run of it on the three
  synthetic cartridges.
- **Speed**: `EmuSen_NativeCores.md` §8.1's P3, the plain frame within ±1% of the step-1 build, interleaved, through the
  PGO driver on the two games; and §14.5's frame call allocating nothing with MarsRT open, since the adapter's frame end
  gains one `frame_info` read (§18.4.4).
- **The B4 grep**: no type under `EmuSen.Cores.Nintendo.MarsRT` in any shipped project, the PGO driver aside.

#### 18.6.2 The WiseMan classes

The classes of §18.5, the same filter, every test passing or each change of count explained by a test moved, rewritten
or removed in step 7.

#### 18.6.3 The debugger's golden corpus

**Measured on 2026-10-18, not predicted.** The corpus's `marsrt` target was pointed, in a working copy and not
committed, at the generic adapter over the same library (`new CoreEngine(MarsNative.Library.Library!, { ExpansionPak =
true })`, bundled by `CoreFactory.Bundle`, so the generic target and the N64's codec), and the corpus recorded again
with `EMUSEN_DIANASH_GOLDEN=record`; the files were then put back. This is the adapter **as it stands today**, before
any of §18.7's work. Of 100 cases, 91 pass unchanged (among them `bp`, `bt`, `cov`, `profile`, `trace`, `pal`,
`sprites`, `channels`, `core`, `man`, `watch`, `framelog`, `callers`), and nine files differ, each in its `marsrt` case
alone:

| File | The shim's output | The adapter's today | After §18.7 |
|---|---|---|---|
| `regs` | "N64 CPU registers:", 36 named values, the VI's 14, the RSP's 34 | "MarsRT CPU registers:" and nothing else | the registers back through `DEBUG_REGISTERS` (F3, F4) and the VI's through the pack (F5); the heading's name differs (A1) |
| `setreg` | "VR4300 (MarsRT) registers:", the list; the refusal names 36 registers | "CPU registers:", empty; "Available: (none)" | as the shim's but for the heading, "CPU registers:" (F6) |
| `cpus` | "N64 debug processors", "VR4300 (MarsRT)", "RSP (MarsRT)", the RSP without `bp` | "MarsRT debug processors", "CPU", "RSP", **the RSP offering `bp`** | the names stay machine info's (F6); `bp` gone again by the halting probe (F7) |
| `disasm` | ten and four decoded lines | "MarsRT target has no disassembler, or nothing was returned." | the same lines through the pack (F8) |
| `eval` | 52 symbols: the GPRs, `HI`, `LO`, `Cycles`, `PC`, the VI's 14, `Halted`, `frame` | `frame` and `stackdepth` | the main processor's registers, the VI's names as pack symbols, `frame`, `stackdepth`; the RSP's `Halted` not (F15) |
| `summary` | three lines: frame, console time and RDRAM; the CPU's PC, SP and RA; the VI | "MarsRT (Rust) 0.1.0, n64, frame 2" | the adapter's head line and the pack's CPU and VI lines (F10) |
| `spaces`, `mem` | "N64 memory spaces:" | "MarsRT memory spaces:" | the same, with the `VI` space added (F5) |
| `vectors` | "N64 does not report an interrupt vector table." | "MarsRT does not …" | as the adapter's (A1) |

So after the second phase the differences expected, and to be decided by Q11, are of three kinds only: the engine's
name where the shim said "N64" (`regs`, `cpus`, `spaces`, `mem`, `vectors`), the processors' descriptions (`cpus`, `setreg`), and the summary's head line
and the RSP's absent symbol; plus the `VI` space's line in `spaces` and `mem`, and the error text of `mem` that lists
the spaces. Any other difference fails the gate.

### 18.7 The second phase, in steps

Each step leaves every program working and is merged on its own gate; the shim stays, reached through the factory as
now, until step 6 switches the N64 to the generic adapter (§6's B4 failover). Steps 2 to 4 widen no surface and can be
built before the tester answers Q1 to Q5; step 5 waits for those answers and for the minor's number.

1. **Baselines on the branch's base**, before any change: the digest probe on Super Mario 64 and Ocarina of Time
   (`EMUSEN_MARSRT_DIGEST_GAMES`, the 26 runs of `Mars_Native.md` §7.5 and the two GPU runs on the RX 6800); MarsRT's
   crate tests with `--profile test-checked` and their count; the kit on the two games' copies, 600 frames; the
   WiseMan counts of §18.5; and `EMUSEN_DIANASH_GOLDEN`'s `marsrt` cases through the generic adapter as §18.6.3 ran them.
2. **The three candidate defects** of §18.3, each by a test that fails first, or recorded as none. C1's fix is the
   generic adapter itself (step 6); C2's and C3's are the bridge's, for every core.
3. **MarsRT's own work, no surface widened**: `DEBUG_REGISTERS` for both processors with their names in machine info
   (F3, F4); the `VI` view space (F5); `SETTING_NOTES` for the antialiasing note and the `Gpu` setting's report, and a
   log line naming the device (B3, H5); the IS-Viewer's lines into the log (H6, if Q8 says so); machine info's frame
   rate from the VI, with `MACHINE_INFO` on a change (A5, if Q6 says so); `RdpWorkers`' help without "as on Mars"
   (B2). The crate's tests, the kit (C2 and C14 cover the new bit and fields) and the digest probe (every column but
   the rate's, which is listed) after it.
4. **The pack and the host, no surface of the core ABI widened**: the RSP's table into `emusen-mips` as a dialect, with
   a parity class over every RSP word pattern against `RspDisassembler` before the C# goes (B3's method, §13.5); the
   N64's per-space disassembly, physical map, VI register names and symbols, summary lines and byte order in
   `emusen-systems`' `n64` module with its C# facade (the platform interface's next number); in the generic target the
   pack's disassembly where a core decodes none, the processor chosen by its code space, the `VideoRegisters` from the
   pack, the halting probe (F7), the summary with spaces; in the bridge the interrupt noted from the stop reason (F13)
   and the call stack reset on a new machine (C2); the frame log's byte order from the pack (F14).
5. **The minor, once approved and numbered**: the header and `baseline.txt`, `emusen-native`'s trait methods and
   `core_exports!` groups, the C# `CoreInterface`, the kit's cases (§18.4), the normative text as a new §6 section of
   `EmuSen_CoreAPI.md` with its changelog row, and an entry in `EmuSen_CoreAPI_Extensions.md`; then MarsRT's
   implementations (`marsrt_core_counters`, `_thread_counters`, `_block_counters`, `_set_present` and `_physical`
   become the common exports; the gate sets the frame flag) and the host's use of each (`CoreEngine.Telemetry`,
   `SetControllerConnected`, `DisplayLines`, the gated frame end, the target's `ResolvePhysical`, the telemetry
   provider and the session service's kind).
6. **The switch**: the N64 moves from `SoleEngines` to `SoleDiscoveredEngines` (`CoreCatalog.cs:345`, `:348`), so the
   factory's v1 arm opens it; the factory's N64 arm, its `Running` and `Bundle` cases (`CoreFactory.cs:38–39`, `:160`,
   `:194–195`), `CoreCatalog`'s three N64 rows and `ControllerPorts`' (`:203`, `:242`, `:283`; `:43`) go, the catalog
   reading core info as for any discovered console; `CoreRegistrationGolden.txt` changes only where §18.6 lists it.
7. **The tests move**: each class of §18.5 onto `CoreEngine` and `CoreDebugTarget`, the test hooks into one WiseMan
   fixture, `MarsRtTestHooks`, calling the `marsrt_core_*` that stay by name through `CoreLibrary.Export`; `MarsDebugRig`
   onto the generic target; `MarsCoreSettingsTests` onto the schema and its notes; `MarsDisassemblerTests` onto the
   pack's parity class. A test whose claim was the shim's alone (`Get` of an unknown key throwing, the count clamped) is
   rewritten to the adapter's rule or removed, each named in the step's record.
8. **The deletions**: the shim's nine files; `marsrt_core_save_cpu`, `mars_rdp_*` and `mars_rsp_*` (dead), and each
   `marsrt_core_*` that step 5 made common; `PortLibrary`'s use for MarsRT (`PortEngine` stays until MoonRT's and
   MercuryRT's steps).
9. **The gates of §18.6**, all of them, and then the pages: `Mars_Native.md` a section for the step, `Mars_Debug.md` and
   `Mars_Cheats.md` where the debugger and the gate moved, `EmuSen_Mips.md` for the RSP's dialect, and this section's
   results.

**Sizes, against §7's B4 row** [A]: in MarsRT about 500 to 800 lines of Rust (registers, the view space, notes, the log,
the minor's five items and their tests); in `emusen-native` about 250 (the trait methods, exports and kit cases); in
`emusen-mips` and `emusen-systems` about 700 (the RSP's dialect at about the C#'s 235 times the measured 1.8 of
§13.6, the N64's views, the parity class's Rust half); in the host about 300 of C#; about 1,478 lines of C# deleted
with the shim and about 300 of Rust with the dead exports.

### 18.8 Questions for the tester

Each is asked on its own, so that no answer is read as deciding another. Q1 to Q5 are the minor's items; any may be
taken without the rest.

| | Question | Recommendation |
|---|---|---|
| **Q1** | `TELEMETRY` (§18.4.1): a core's counters as names in machine info and one export of values, for `coretop`, the session service, tests and benchmarks; or keep MarsRT's counters as private extensions that only tests read? | The capability, as D11 already recommended: it is the only form in which a dashboard can show any core's counters without per-core code, and it costs nothing a frame unless read. With it, the recompiler's 13 counters (H3), which nothing reads today, are kept in the list rather than deleted |
| **Q2** | `frame_info.lines` (§18.4.2), appended to the struct, for the CRT filter at an internal resolution above 1×? | Yes: without it the filter draws a 2× picture's rows as the console's lines, a visible change; it is one field in a struct already read |
| **Q3** | `PRESENCE` (§18.4.3) as the narrow `set_present`, or the general `set_device` of `EmuSen_CoreAPI_Extensions.md` §10.1? | The narrow call: presence is what the N64 needs and is part of its state; a device kind would duplicate the controller choice the settings already make, and can be added when a light gun needs it |
| **Q4** | The cheat gate (§18.4.4): a frame flag the core sets when its console's cheat devices hold off; or `CHEAT_POKES` alone, leaving the 16-bit writes, tests and repeaters ungated; or no gate? | The flag: it keeps every code gated as today, one call a frame, and the gate is the measured fix for Super Mario 64's boot (`Mars_Cheats.md` §5.1) |
| **Q5** | `DEBUG_TRANSLATE` (§18.4.5) for `addr` through the TLB; or `addr` answering only the unmapped segments by arithmetic in the pack? | The export: games run mapped code (GoldenEye's `0x7F` region, `Mars_Native.md`'s mapped blocks), and the walk is the core's alone; VenusRT's item 11 takes the same call |
| **Q6** | The frame rate (A5): machine info's rate as the VI's programmed field pair, exact and constant while the VI's timing stands, raised by `MACHINE_INFO` when it changes; or the shim's per-frame alternation kept, by a per-frame field in `frame_info`? | Machine info's: it is the pacing rate by definition (§6.4 of the core ABI page), the alternation is half a millionth, and an event a frame would re-read machine info every frame. The digest probe's rate column then changes to the pair's mean, listed in the step's record |
| **Q7** | Disassembly (F8): the N64's pack decoding when a core claims no `DEBUG_DISASSEMBLE`, with the RSP's table moved into `emusen-mips`; or MarsRT claiming `DEBUG_DISASSEMBLE` over the same crate? | The pack: B3 put the VR4300's, 6502's and SM83's disassemblers in the packs, and a second N64 engine gets them free; the core keeps the right to claim the capability later, which the target would then prefer |
| **Q8** | The IS-Viewer (H6): its lines into the core's log, read by every frontend's log and by the corpus test; or a test-only extension as now? | The log: it is a game's debug output, which a player debugging homebrew wants to see, and v1 already carries logs |
| **Q9** | The device report (H5): a note on the `Gpu` setting when it is not honoured, and a log line naming the device when it is? | Yes: both are existing surfaces, and the report is a sentence about a setting, which is what notes are for |
| **Q10** | `ExpansionPak` after a state of the other memory size (B6): the setting left as the player chose it, the machine's memory stated by machine info; or kept following the state? | Left as chosen: the setting is the next load's choice, and the host has no way to follow a state without a rule for one core |
| **Q11** | The golden corpus's `marsrt` cases (§18.6.3): accept the differences listed there and record the cases again from the generic adapter? | Yes, after the step's run lists them again: each is the adapter's rule that every v1 core already follows, not a loss of what the debugger shows |
| **Q12** | The test hooks (A12, C4, E2, F16, I5): kept as `marsrt_core_*` extensions behind one WiseMan fixture; or the tests moved into MarsRT's crate and the exports deleted? | The fixture now: the tests are many (§18.5) and graded; moving them into the crate is a later piece of work that changes no shipped surface |
| **Q13** | The frame log's byte order (F14): a fact of the N64's pack; or a `big_endian` flag on machine info's spaces? | The pack: every N64 engine's memory is big-endian, and a flag would make each core restate the console |
| **Q14** | `EMUSEN_MARS_NATIVE=0` (I1): retired with `PortLibrary`'s use for MarsRT, since nothing is left to fall back to? | Retire it |
| **Q15** | The antialiasing note with no game running (B3): none, since a note comes from a machine; or kept by a rule in the settings window? | None: a rule in the window is per-core code, and the note appears as soon as a game runs |
| **Q16** | A processor that cannot halt (F7): known from `debug_set_breakpoints`' refusal, with §6.14 of the core ABI page saying so in a clarification; or a `halts` field on machine info's processors? | The refusal and the clarification: the crate's default already answers so for every core built on it, and a field would state the same fact twice |

### 18.9 Built 2026-10-18: steps 1 to 4 for MarsRT, the parts that need no ABI change

*Measured on 2026-10-18 on branch `b4-marsrt`, from `WiseMan` at `4b42f4f1`, under the decisions of §20. What was
asked: §18.7's step 1, step 2 for MarsRT's own candidates, step 3, and step 4's pack parts with platform interface 19.
Steps 5 (the minor, which another branch builds with the host pieces the cores share) and 6 onward are not begun, and
the shim is untouched but for reading the pack's code.*

**Step 1, the baselines**, taken from the branch's base before any change, with copies of the measurement games
(`~/.cache/emusen/pgo/games/measure`) and the build kept beside them (`~/.cache/emusen/probe/mars/b4/`):

| Baseline | Result |
|---|---|
| WiseMan, §18.5's classes, the same two filters | 902 of 902 and 443 of 443 (the GPU classes on the RX 6800 alone) |
| MarsRT's crate, `--profile test-checked`, with the measurement states, the corpus and the GPU tests | 498 of 498 |
| The digest probe, Super Mario 64 and Ocarina of Time only | 26 runs, 8,654 lines; and the two runs at 2× on the RX 6800 |
| The corpus, n64-systemtest | *Failed 46 of 4637 tests*, the recompiler's transcript the interpreter's |
| The kit, 600 frames on each game | 22 cases passing on each; 48 exports, capabilities `0x1fde`, six spaces |

The golden corpus through the generic adapter is §18.6.3's, measured in the first phase; nothing in this step changed
what the adapter shows of MarsRT, so it was not run again.

**Step 2.** C1 is MarsRT's own (the shim's rethrow), and the generic adapter is its fix, at the switch; it is to be
shown failing then. C2 and C3 are the bridge's and go to the shared branch, with the halting probe (F7).

**Step 3, MarsRT's own work** (`src/ffi/host.rs`; `Mars_Native.md` §9): `DEBUG_REGISTERS` for both processors with
their names in machine info (F3, F4); the `VI` space, read-only, the registers big-endian (F5); `SETTING_NOTES` for the
antialiasing note and the device's report (B3, H5); the device's report and the IS-Viewer's lines in the log (H5, H6);
machine info's frame rate from the VI, with `MACHINE_INFO` when it moves (A5); `RdpWorkers`' help (B2). The shim reads
none of it yet, so nothing a frontend or the debugger shows changes.

**Step 4's pack parts.** The RSP's table is `emusen-mips`' `rsp.rs` (`EmuSen_Mips.md` §7), the item §13.6 left; its C#
moved to `Crystal/Systems/N64/RspDisassembler.cs` as the failover, `Decode` forwarding to the library. The N64 module
of `emusen-systems` gains `n64::views`: each space's code listed by the processor that runs it at the address it sees
it at (F8), the physical map (F9's second half), the VI's registers by name from the `VI` space (F5) and the summary's
CPU and VI lines (F10). The C# facade is `Crystal/Systems/N64/N64Views.cs`, each answer the library's unless
`EMUSEN_SYSTEMS_NATIVE=0`, each `…Managed` twin the C# the shim held, moved: `DecodeIn`, `Physical`,
`VideoRegisters`, `Summary`. The shim now reads `DecodeIn` and `Physical` and the register names from it. Wiring them
into `ISystemDebugViews` and the generic target is the shared branch's; the facade's shapes are what it needs.

**Platform interface 19.** `emusen_systems_disassemble` takes the processor `rsp`, and `emusen_systems_view` the system
`n64` with the views `disassemble`, `instruction`, `physical`, `video_registers` and `summary`; no export is added, but
an older library would refuse them, so the number moves (`PlatformLibrary.InterfaceVersion`, `emusen_platform.h`,
`lib.rs` together). The header compiles clean as C99 and as C++17.

**The frame log's byte order** (F14) is a fact of the table: each data file gains `byte_order`, `big` for the N64,
`little` for the NES, SNES, PlayStation and Game Boy, and null for the Genesis family, which no one has stated yet and
a reader takes as little-endian, as today (a question below). `SystemPack.ByteOrder` is the literal, `SystemPacks.BigEndian`
reads the table's under the switch, and `CoreEngine` keeps the answer per machine and reads a frame log's value in that
order. No core's output changes today: MarsRT's shim reads its own, and every other system's order is the one the
adapter already read. `MarsRtCoreAbiTests.The_v1_adapter_logs_an_n64_value_big_end_first` failed first, the adapter
logging `0x3412` for the bytes `12 34`, and passes with the change. *The Genesis family was set `big`, measured on
2026-10-18 (`Nephrite_Native.md` §66): every space on the 68000's side, the Sega CD's and the 32X's holds a word big-end
first; the Z80's RAM and bus hold the Z80's words little-end first and now read swapped, a per-space byte order in
machine info proposed for them; `NephriteTests.The_frame_log_reads_a_genesis_value_big_end_first` failed first and
passes with the change.*

**The gates, after:**

| Gate | Result |
|---|---|
| WiseMan, §18.5's first filter and the parity classes the step touches (`DebugCodeParityTests`, `SystemTableParityTests`, `CheatCodecParityTests`, `SystemCodeParityTests`, `SaveMediaParityTests`, `GalaxiaParityTests`) | 990 of 990: the 902 of the baseline, `MarsRtCoreAbiTests` 5 (one new), `DebugCodeParityTests` 7 (two new), `SystemTableParityTests` 8 (one new), and the others' 72 |
| WiseMan, the second filter | 442 of 443 in the first run: `GraphicsSettingsWindowLayoutTests`' case of 3× resolution and 3× antialiasing found the note hidden, a window that reads `CoreCatalog`'s C# rows, which this step did not touch; the class alone three times, 14 of 14 each, and the filter again, 443 of 443. Recorded as intermittent, not pursued |
| The crate | 505 of 505: the 498 and the seven of `ffi::host` |
| The digest probe | every line of the 26 runs and of the two GPU runs identical to the baseline's |
| The corpus | the same summary and transcript |
| The kit | 22 cases passing on each game; 50 exports, capabilities `0x19fde`, seven spaces |
| `emusen-mips`, `emusen-systems`, `emusen-platform` (`--profile test-checked`) | 13, 23, and 31 with the four of `dianash_abi` |
| The parity class on the RSP | every primary opcode crossed with every function and with eleven element and register patterns, four ways, and 200,000 random words: over 340,000 words at five IMEM addresses, the library and the C# alike |
| The parity class on the N64's views | 4,000 words decoded in nine spaces (names in two cases and one unknown), the physical map at its 26 boundaries and 2,000 random addresses with three RDRAM sizes, the VI's registers and the summary on 50 generated memories, a short VI space and a lower-case name among them |

**Seeded faults**, each made, built, run and reverted:

| Fault | Caught by |
|---|---|
| the RSP's half-selectors counted from 3 rather than 4 | `Every_rsp_word_disassembles_alike`, and `The_n64_views_answer_alike` through IMEM and DMEM |
| PIF RAM's offset in the physical map one too high | `The_n64_views_answer_alike` |

**A question this step raises**, for the tester: the Genesis family's `byte_order`. The 68000 and the SH-2 are
big-endian, and if Nephrite's spaces hold memory in the processors' own byte order, as MarsRT's do, the frame log reads
a Genesis value with its bytes swapped today. Recommended: Nephrite's owner states it after reading how its spaces lay
out a word, with a test like MarsRT's; it is left null here because setting it changes that core's output.
*Answered 2026-10-18: `big` for all three, the byte-order paragraph above and `Nephrite_Native.md` §66.*

### 18.10 Built 2026-10-18: steps 5 to 9 for MarsRT, on the generic host alone

*Built on branch `b4-marsrt` from `WiseMan` at `e75ad6bf` (core ABI 1.13 and §22's host pieces), then merged with
`WiseMan` at `9eb357a9`, which carries MoonRT's last phase (§23). What was asked: §18.7's steps 5 to 9, the N64 pack's
views wired into `ISystemDebugViews` and the generic target, platform interface 20 if the pack needed one, and the
golden corpus re-recorded only as §20 decided. MarsRT now runs on `CoreEngine` and `CoreDebugTarget` alone. `PortEngine`
and the pre-stable host stay, for MercuryRT's shim (§22.7). What MarsRT itself does is `Mars_Native.md` §10.*

#### 18.10.1 Step 5: the five items, and the units

- **Units: one, the VI.** It is the one part that runs no code whose registers the debugger showed, so it is a `video`
  unit, id 256, its fourteen registers read by `debug_registers`. §9's `VI` space, the same registers as bytes, is
  removed, so the registers have one owner. The AI, PI, SI, MI and RI were never shown and are not listed.
- **TELEMETRY**: 58 counters, the shim's three counter extensions' values under names in machine info; `Blocks::counters`
  became an array, so a read allocates nothing.
- **PRESENCE**: `set_present` for ports 0 to 3. **`frame_info.lines`**: rows over the width's multiple of the 640-wide
  raster, counting stored rows as §6.35 says. **`EMUSEN_FRAME_CHEATS_HELD`** on a frame that ended with `Status.IE`
  clear (`Mars_Cheats.md` §5.1). **DEBUG_TRANSLATE** for processor 0, kernel mode's view through the TLB.
- MarsRT claims `0x3819fde` and defines 53 `emusen_core_` symbols. Its own symbols left are ten `marsrt_core_`
  extensions that only WiseMan's fixture reads by name (`run_steps`, `is_viewer_text`, `cop0`, `bus_write32`,
  `bus_read32`, `set_cop0`, `mi_raise`, `rsp_step`, `state_kind`, `load_fields`) and the eight `mars_vi_scan_*` of the VI
  differential.

#### 18.10.2 The N64 pack in the generic target, and platform interface 20

`N64DebugViews` fills §31.7's members: `Summary` with the VI's line read from the unit, `ResolvePhysical` from the pack's
physical map, and a new member, `ISystemDebugViews.Disassemble(space, address, count)`, empty by default. The generic
target lists a space's code through it where the core claims no `DEBUG_DISASSEMBLE`, which MarsRT does not. The
processor is chosen by the space, so IMEM is RSP code. `emusen_systems_view`'s `n64` `summary` now takes the VI's
registers as a `vi` array, and the `video_registers` view went with the space it read, so the platform interface moves
to **22** (`PlatformLibrary.InterfaceVersion`, `emusen_platform.h`, `lib.rs`).

#### 18.10.3 Step 6: the switch

The N64 leaves `SoleEngines` for `SoleDiscoveredEngines`, so the factory's v1 arm opens it as it opens the SNES and the
NES. The factory's N64 arm (now "missing engine"), its `Running` and `Bundle` cases, the catalog's N64 rows for buttons,
settings and axes (`AxesByConsole` is gone) and `ControllerPorts`' N64 case are deleted. A new
`CoreCatalog.SoleEngineSystem` reads the sole engine's system entry from its sidecar, so `ButtonsFor`, `AxesFor` and
`ControllerPorts` need no library loaded. `SettingsFor` falls back to the sole engine's sidecar settings. Two effects
reach other consoles, both read in the registration golden. With no game loaded, the SNES now lists VenusRT's
`sa1_catch_up` and the NES MoonRT's `region`, as they list when a game runs. With no game running, the settings window
shows no antialiasing note, as Q15 decided.

#### 18.10.4 Step 7: the tests moved

Every test reaches MarsRT through `CoreEngine`, `CoreDebugTarget` or the factory. `Fixtures/MarsRt.cs` opens
`libmarsrt.so` beside the tests by its path, with no discovery, and reads what the shim's members read. It uses 1.13's
surfaces (units, telemetry, setting notes) and the ten extensions, as C# 14 extension members on `CoreEngine`.
`MarsRtTestCore` is a `CoreEngine` with the typed setting properties the tests set. `MarsMachine` moved to the fixtures.
Changed or replaced, each because its claim was the shim's alone:

| Was | Now | Why |
|---|---|---|
| `MarsCoreSettingsTests.The_catalogue_offers_the_console_s_eight_settings_and_the_core_answers_them_and_its_own` | `The_catalogue_offers_the_schema_s_settings_with_no_game_loaded` | the catalog reads the schema; nine settings, `Recompiler` among them |
| `…A_count_outside_its_range_is_clamped_and_text_that_is_neither_is_refused` | `…A_value_outside_the_schema_is_refused_and_an_unknown_key_reads_as_nothing` | the adapter refuses where the shim clamped (§18.2's B4) |
| `GraphicsSettingsWindowLayoutTests.The_antialiasing_row_says_when_the_resolution_holds_it_back`, `…The_antialiasing_note_follows_the_resolution_as_it_changes` | `With_no_game_running_the_antialiasing_row_has_no_note`; the note itself in `MarsCoreSettingsTests` on a running machine | Q15 |
| `MarsRtFrontendTests.A_state_of_the_other_memory_size_carries_the_pak_with_it` | `…rebuilds_the_machine_and_leaves_the_setting` | the adapter's rule for a state of the other size |
| `MarsRtEngineTests.With_the_library_turned_off_an_n64_game_says_the_core_is_not_in_this_build` and its child process | `With_no_marsrt_beside_the_program_an_n64_game_says_the_engine_is_missing`, in process | `EMUSEN_MARS_NATIVE` is retired; discovery is read per call |

The presence asserts in `MultiplayerTests`, `DisplayAspectTests` and `ScreenFilterRenderTests`' lines case were also
changed. They now run a frame or load a game first, because the adapter sends a port's change as the next frame starts
and a shape comes from a frame. `NativeHostTests` holds `mars_rsp_new` and `mars_machine_new` absent.

#### 18.10.5 Step 8: what was deleted

The shim's nine files: eight in this step (1,204 lines) and `RspDisassembler.cs` in step 4. In MarsRT, `ffi/rdp.rs`,
the RSP's C ABI and the counter, register, physical, presence and `save_cpu` extensions (355 lines). Also
`EMUSEN_MARS_NATIVE` (the registration test's switch list now holds MercuryRT's alone) and `PortLibrary`'s use for
MarsRT. The B4 grep, `Nintendo.MarsRT` over shipped code, finds only the PGO driver, which holds no type of it: its
check is `CoreEngine { Info.Id: "marsrt" }`.

#### 18.10.6 Defects the switch showed, each failing first

| Defect | Shown by | Fix |
|---|---|---|
| **C1**, the shim's: a refused N64 state reached the frontend API as a fault (-1108), not `StateRefused` (-1114) | `SharedHostTests.A_refused_state_reaches_the_API_as_state_refused(n64)`, failing on a probe build at `8feb278c` with the shim | the generic adapter; passes after the switch |
| The setting note was read only after a setting changed, so a game opened at 2× resolution and 4× antialiasing showed no note | `MarsCoreSettingsTests.The_note_is_read_when_the_game_is_loaded` | `CoreEngine.Adopt` reads the notes once the settings are applied |
| A view space's read wrapped at its length, so the kseg0 view gave `0x00` for a byte at its address where `0x5A` was | `MarsRtCoreAbiTests`' kseg0 view case | `CoreDebugTarget` wraps only spaces that are not views |
| `CoreEngine.DisplayLines` gave the stored rows' lines (240) for a frame it hands over 480 rows tall, its rows repeated. The CRT filter would have averaged two rows and two columns into each line | the digest's `lines` column on all 26 runs; `ScreenFilterRenderTests.A_field_s_lines_are_counted_in_the_rows_the_frame_is_handed_with` (480 expected, 240) | `DisplayLines` is multiplied by the repeat when the engine repeats the rows, so it counts the rows `ScreenHeight` holds (`EmuSen_CoreAPI.md` §6.35's host line) |
| MarsRT created only port 0 holding a controller, where §6.36 says every port holds one. The latch holds a change only, so a pad plugged into port 2 after the frontend had emptied it was never sent | `MultiplayerTests.Four_pads_play_the_n64s_four_ports_in_a_big_screen_session` (ports 2 to 4 stayed empty); `ffi::host`'s presence test | MarsRT's `create` fills the four ports. The digest probe empties ports 2 to 4 after loading, as Mistress's router does for one player, so its runs stay the baseline's |

#### 18.10.7 What was re-recorded

| Record | What changed |
|---|---|
| The debugger's corpus, `marsrt` cases | Only §18.6.3's kinds. "MarsRT" for "N64" in `cpus`, `mem`, `regs`, `spaces` and `vectors`. "CPU" and "RSP" for "VR4300 (MarsRT)" and "RSP (MarsRT)" in `cpus` and `setreg`. The generic head in `summary`, and the VI line without the picture's size. `eval` loses the VI's fourteen names and the RSP's `Halted` and gains `stackdepth` (F15). `regs` lists the same values, the VI's from the unit. `disasm` is unchanged, through the pack |
| `CoreRegistrationGolden.txt` | Re-recorded on `WiseMan`'s NES rows. For the N64, as §23.4 for the NES: `CoreEngine` and `CoreDebugTarget` for the shim's types, "MarsRT" as the identity's name, `registered[MarsRT (Rust)] = False` (discovered, not registered by hand), the "off" world running MarsRT since its switch is retired, and the settings and hints from the schema, `Recompiler` among them. For the SNES and the NES, the offline settings of 18.10.3 |

The schema's hints arrive with the host's cost sentence appended. Where MarsRT's help already says the cost (`RenderScale`,
`DeferredPresentation`, `Antialiasing`), the hint says it twice (18.10.9).

#### 18.10.8 The gates, measured on 2026-10-18

After the merge with `WiseMan`, from the branch's last build, against the baselines of 18.9 and 18.6.3:

| Gate | Result |
|---|---|
| WiseMan, §18.5's first filter with the classes the switch reaches (44 classes: 18.9's 31, plus `SystemTableParityTests`, `CoreDiscoveryTests`, `CoreDebugViewTests`, `CoreAdapterTests`, `CoreEngineThreadTests`, `SnesSystemPackTests`, `MercuryRtDebugTargetTests`, MoonRT's frontend and ABI classes, VenusRT's two debugger classes, Nephrite's and Diamond's debugger classes) and the new allocation case | **1,098 of 1,098** |
| WiseMan, the second filter and `MoonRtEngineTests`, the GPU's classes on the RX 6800 alone | **438 of 438**. The first run, before the presence fix, failed two: the multiplayer case (18.10.6), and `GraphicsSettingsWindowLayoutTests.The_N64s_controls_show_the_configs_values_and_the_defaults_where_it_has_none`, which read 4 workers where the config said 2. It passed in the next two runs. 18.9 also recorded one case of this class failing once. Recorded as intermittent |
| The digest probe, Super Mario 64 and Ocarina of Time only, 26 runs and 8,654 lines, and the two runs at 2x on the RX 6800, 606 lines | **Every picture, drain of sound, state, length, shape, `lines` and aspect identical to the step-1 baseline.** The rate column differs on 8,587 and 599 lines: it is machine info's limit, 50.016020, where the shim gave each field's own rate (Q6, 18.9). The 41 and 5 frames of the cycle cap keep 22.865854 |
| The corpus, n64-systemtest | *Failed 46 of 4637 tests*, and the recompiler's transcript is the interpreter's |
| The kit, 600 frames on each game | 22 cases passing on each, core ABI 1.13, capabilities `0x3819fde`; one unit with its registers, 58 counters read twice alike, `set_present` refused past the ports |
| MarsRT's crate, `--profile test-checked`, with the measurement states, the corpus and the GPU tests | 507 of 507: 18.9's 505 and the two cases of `ffi::host` that step 5 added; the presence case now also holds every port full at create, and failed on that before the fix |
| C1 | failing on the shim (a probe build at `8feb278c`: -1108 for -1114), passing on the generic engine |
| Allocation (§14.5) | `ManagedCrystalTests.MarsRTs_frame_picture_and_sound_allocate_nothing`: 300 frames, pictures and drains, 0 bytes on the calling thread. MarsRT's debugger snapshots are still 0 bytes |
| Speed (P3), the PGO driver, `mistress=1 warmup=120`, 600 frames from the measurement states, 8 rounds interleaved with the order swapped each round, under the timing lock and a quiet gate of half a processor | Super Mario 64, the loop: before 3.606 ms a frame (3.543 to 3.941), after 3.556 (3.512 to 3.780), paired median -1.3%. Ocarina of Time: before 4.585 (4.527 to 4.774), after 4.533 (4.511 to 4.693), -1.1%. `RunFrame` alone: -1.4% and -1.2%. The ranges overlap, and each game ends on one state hash in both builds. "Before" is `be199ca2` (the shim, with step 5 done); "after" is this branch before the presence fix, which touches `create` alone. Both are unguided `dist` builds. An earlier set of 8 rounds ran under a gate of three processors, with the machine busy in four of them. It gave +2.4% and -1.5%, with outliers up to 6.5 ms, and is kept beside the runs but not used |
| The B4 grep | `Nintendo.MarsRT` in shipped code: only the PGO driver's path, no type |

The timing files are kept under `~/.cache/emusen/probe/mars/b4/timing/` (`r1.tsv`, `r2.tsv`), and the digest files beside them.

#### 18.10.9 Left, and for the tester

- **A state's ports and the latch.** §6.36 makes presence part of a state, and Mistress re-sends every port each poll
  "since a state loaded carries the ports it was saved with". But the latch sends only a change from what it last
  held, not from what the core holds. Suppose a state is loaded whose ports differ from the router's. The frontend's
  view then never reaches the core until a pad is plugged in or pulled out. Not shown by a test yet. *Recommended*:
  after a state loads, `CoreEngine` marks every port's presence changed, so the next frame sends the frontend's view.
  This is a host change for every core with `PRESENCE`.
- **A frontend that routes no ports.** Since the ABI's rule is now MarsRT's, an N64 game opened by a host that never
  calls `SetControllerConnected` sees four controllers where the shim gave it one. Mistress's router empties the
  ports no player holds, so play in Mistress is unchanged.
- **The doubled cost sentence** in three N64 hints (18.10.7). *Recommended*: MarsRT's help drops the sentence the
  schema's cost restates. That is a change of text the player reads, so it is for the tester.
- **The units' registers in expressions**, as §23.6 recommends for every core: the VI's names are no longer symbols.
- **The PGO profile** is still void (`Mars_Native.md` §7.6), so the speed figures are unguided against unguided.

---

## 19. B4 for MoonRT and VenusRT, phase 1, 2026-10-18: what the two shims add, where each piece goes, and the gates

*Read on 2026-10-18 on branch `b4-moon-venus`, from `WiseMan` at `480bf5e2`, against the core ABI at 1.12
(`emusen_core.h`). The step is §6's B4 row for two of its four cores; MercuryRT's shim stays while that core is paused,
and MarsRT's is another branch's. `PortEngine` and the pre-stable host (`EmuSen/Cores/Native/Native*.cs`) are deleted
only once every shim is gone (§13.5 of `EmuSen_CoreAPI.md`), so they stay. The decisions this section works under:
SR8 (no per-core code above the ABI once B4 is through, §11.1), D11 (each shim's additions as its core's own exports, a
minor proposed before it is built, §11.2), R2 (§10), Q28 of `EmuSen_DianaSh.md` (export `DEBUG_REGISTERS` from both
cores, decided yes), §13.4's second answer (`MoonBoards`' paths, rules and implemented mappers to MoonRT's own exports)
and §2.4 and §13.4 of `EmuSen_CoreAPI.md`. Nothing here changes the core ABI; §19.4's addition is a proposal, written
as **1.next**, since its number is assigned when it is approved and MarsRT's step may propose one too.*

The routes a piece can take are named as in `EmuSen_CoreAPI.md` §30.6, in their order of preference, each later one
widening a stable surface: **host** (the generic adapter `CoreEngine`, `CoreDebugTarget` or `CoreDebugBridge` composes
it from what v1 carries), **pack** (the console's system pack, a fact of the console applied to what any engine of it
publishes), **core** (the core's own descriptors or an optional capability v1 already defines: no ABI change),
**extension** (a core's own export under §4.7 of that page, for tests or one console's pack) and **minor** (an addition
to the ABI under its §4.2). A fifth answer is **retired**: behaviour that existed only to reproduce a deleted C#
oracle's exact words or types, which §2.4 of that page let the shims keep and SR8 ends.

### 19.1 The two shims, and who reaches them

| | VenusRT | MoonRT |
|---|---|---|
| Files | `VenusMachine.cs` 45, `VenusNative.cs` 22 | `MoonRtCore.cs` 171, `MoonRtDebugTarget.cs` 326, `MoonBoards.cs` 75, `MoonMachine.cs` 54, `MoonNative.cs` 31 |
| Shipped callers | none: the SNES runs on `CoreEngine` through discovery (`CoreFactory.cs:33`, `CoreCatalog.cs:348`'s `SoleDiscoveredEngines`) since `VenusRT_Native.md` §68.5 | `CoreFactory.cs:36` and `:48–50` (the `.nes` arm), `:162` (`Running`), `:175–182` (`Bundle`), and `CoreCatalog.cs:345` (`SoleEngines`) |
| Test callers | 3 files: `Fixtures/Snes/SnesEngines.cs`, `VenusRtCoreAbiTests`, `VenusRtTestRomRunnerTests` | 33 files (§19.7.3) |
| Interface | the pre-stable `emusen_native_*` set only (`NativeMachine`) | the v1 adapter by subclass (`PortEngine`), and the pre-stable set for the machine the step goldens drive (`MoonMachine`) |

[M] by `wc -l` and `grep -rln` over `.cs` files outside `bin/` and `obj/`.

### 19.2 VenusRT's shim, item by item

| # | What it adds | Where (file:line) | Where it goes | ABI |
|---|---|---|---|---|
| V1 | The library loaded through the pre-stable interface: `NativeCoreLibrary.Common("venusrt", …)`, interface version 1, no optional export required | `VenusNative.cs:8–18` | **retired**: discovery and the sidecar already load the library on v1 for every client; the tests move to `CoreLibrary` and `CoreMachine` (§19.9) | none |
| V2 | The switch `EMUSEN_VENUS_NATIVE` | `VenusNative.cs:9` | **retired**: discovery never read it, so since §68.5 it turned off only this test path; a build without the library already says so in the player's words (`CoreFactory.cs:45–46`) | none |
| V3 | The loader's report | `VenusNative.cs:20` | **host**: `DiscoveredCore.Report` and `CoreLibrary.Report`, which `CoreFactory.Missing` already prints | none |
| V4 | The space names by id | `VenusMachine.cs:9` | **core**: machine info's `spaces`, the same eight names (`v1.rs:13`) and the chips' seven after them | none |
| V5 | File 0 the battery, file 1 the SPC700's boot image, file 2 (and 3) a NEC DSP's firmware | `VenusMachine.cs:13–20` | **core**: core info's `firmware` entries `which` 1 and 2 (`v1.rs:66–95`); v1's `create` takes the same numbering (`v1.rs:274–285`) | none |
| V6 | Status −9 as `InvalidDataException` with C#'s words | `VenusMachine.cs:22–32` | **core** for the words (`status_text`, `v1.rs:264–270`); the exception type **retired** | none |
| V7 | A space read whole by its name | `VenusMachine.cs:34–43` | a test fixture's helper over `CoreMachine.ReadSpace` | none |

**VenusRT needs no ABI change and no change to its machine.** Its shim was a test harness that outlived its purpose:
the frontend has driven VenusRT through the generic adapter and `CoreDebugTarget` since stage 6. Its retirement was
built in this phase, as a commit of its own (§19.9).

### 19.3 MoonRT's shim, item by item

#### 19.3.1 Loading and identity

| # | What it adds | Where | Where it goes | ABI |
|---|---|---|---|---|
| M1 | The library through the pre-stable interface for `MoonMachine` (interface version 4, five capabilities required) | `MoonNative.cs:9–20` | **retired** with `MoonMachine` (M14) | none |
| M2 | The v1 library at its fixed name beside the assemblies, the switch `EMUSEN_MOON_NATIVE`, the capabilities and extensions the shim calls, the report's words | `MoonNative.cs:10, 23, 26`; `PortEngine.cs:12–57` | **host**: discovery, the sidecar and trust (`CoreDiscovery`, `CoreFactory.V1Library`), as for VenusRT; `CoreCatalog`'s `MoonRtEngine` moves from `SoleEngines` (`:345`) to `SoleDiscoveredEngines` (`:348`); the switch **retired** as V2 | none |
| M3 | `CoreName` "NES" | `MoonRtCore.cs:38` | **host**: core info's `name`, "MoonRT", as VenusRT's "VenusRT" is; the shell's headings and the log folder change (§19.5, Q3) | none |
| M4 | Width 256 and height 240, the NTSC frame rate, state version 6 and a 4:3 shape, before any game | `MoonRtCore.cs:14, 21, 39–42, 45` | after a load **core**, already: machine info's `video` and `frame_rate`, `frame_info`'s 4/3, `state.version` (`v1.rs:144–156, 184`); before a load **host**, the adapter's neutral answers (0, 60 Hz, 0, 0), as for every v1 core | none |
| M5 | The two pads, each button's bit, ports 0 and 1 only | `MoonRtCore.cs:27, 102–116`; `PortEngine.cs:215–223` | **core**, already: core info's `nes.pad` with the same bits (`v1.rs:52–61, 80–86`) and machine info's two ports; the adapter ignores a port the machine does not list, and its latch sends the changed bits where the shim sent the whole pad, to the same effect (`set_buttons` keeps the bits not changed, `v1.rs:233–237`) | none |
| M6 | `ControllerPorts`, the NES pack's two | `MoonRtCore.cs:118` | **host**: `ControllerPorts.OfEngine` from machine info's ports | none |

#### 19.3.2 The engine's rules

| # | What it adds | Where | Where it goes | ABI |
|---|---|---|---|---|
| M7 | The header parsed before the library sees the image (`NesHeader`) | `MoonRtCore.cs:52, 84–89` | **retired** as a pre-check: the core refuses a malformed image itself (−9, −11, `v1.rs:117–125`); its two readers move: the battery rule into the core (M8), the summary's facts to §19.4 and the pack (D14) | none |
| M8 | A battery file opened only for a board MoonRT builds whose header claims a battery, and not when battery RAM is disabled | `MoonRtCore.cs:81–91` | **core**: `create` takes file 0 only for a cartridge with a battery (today it writes any file 0 into PRG RAM, `v1.rs:126–135`), and machine info lists the file only then, as it already does (`v1.rs:141, 155`); "disabled" is the adapter's (`CoreOptions.BatteryRamDisabled`, `CoreEngine.cs:274, 376`). A board MoonRT does not build is refused at create, so needs no rule. See C1 | none |
| M9 | The state checked before the core sees it: shorter than 8 bytes `EndOfStreamException`, another magic "Not a Moon save state.", a version outside 3 to 7 | `MoonRtCore.cs:16–18, 120–126`; `PortEngine.cs:242–250` | **core**, already: `TRUNCATED`, `FOREIGN` and `STATE_VERSION` with `last_error`, and machine info's `state` (`format` "MOON", `version`, `loads_from` 3 to 7, `v1.rs:156`); the C# words **retired**. See C2 | none |
| M10 | Exception types for the core's statuses: −9 and −11 `InvalidDataException`, −10 `NotSupportedException`, the reproduced .NET faults `IndexOutOfRangeException`, `DivideByZeroException`, `ArgumentOutOfRangeException` | `MoonRtCore.cs:128–140`; `MoonMachine.cs:30–35`; `PortEngine.cs:153–161` | **retired** (SR8). The words are the core's already: `status_text` gives the same seven sentences (`v1.rs:100–112`), so a refusal reads "MoonRT (Rust) refused the game: a mapper no board implements (mapper 5 is not implemented)." See §19.5 for what the frontend API reports instead | none |
| M11 | The status words, and `NativeMachine.Shared`'s words naming "Moon" | `MoonRtCore.cs:130, 142–152`; `PortEngine.cs:150` | **core**, already (`status_text`); the shared words are the adapter's (`CoreLibrary.Words`) | none |
| M12 | RESET, mapping a refusal to its exception and letting a halt go | `MoonRtCore.cs:94–100` | **host**, already: `CoreEngine.Reset` over `RESET` clears the halt (`CoreEngine.cs:622–628`) | none |
| M13 | The five channels' mutes kept by the engine | `MoonRtCore.cs:23, 160–169` | **host**: `CoreDebugTarget.SetChannelMuted` over `MUTES`, the channels named by machine info (`v1.rs:147`) | none |

#### 19.3.3 The debugger: `MoonRtDebugTarget` and what `PortEngine` does for it

The target saves the machine's whole state on every `RefreshProviders`, into a ring of records, and reads every view
out of it by the state's layout listing (`MoonRtDebugTarget.cs:72–96`, `MoonRtCore.cs:55–64`). That is the 6.8 to
9.1 µs a register condition costs on MoonRT (Q28), and a state save a frame while a debugger is attached; every item
below that reads registers from it moves to an export that reads the one value asked for.

| # | What it adds | Where | Where it goes | ABI |
|---|---|---|---|---|
| D1 | Eight spaces: `PRGROM` read-only, `CPUBUS` with side effects | `MoonRtDebugTarget.cs:56–57` | **core**: machine info flags `PRGROM` `read_only` (it does not today, `v1.rs:152`, though `write_space` ignores it, `machine.rs:245–259`); `CPUBUS`'s `side_effects` is set already | none |
| D2 | The 2A03's registers: A, X, Y, S, PC, P, the six flags as 1-bit values, the cycle count | `MoonRtDebugTarget.cs:148–160` | **core**: `DEBUG_REGISTERS` (Q28), machine info's `processors[0].registers` listing the same thirteen names and widths, read from the CPU's own fields; `regs cpu` then shows them, which it does not today ("cpu reports no registers on this core", `setreg.golden`) | none |
| D3 | One processor, "cpu", described "2A03", its code listed from `CPUBUS` | `MoonRtDebugTarget.cs:132–141` | **core**, already: processor 0 "CPU" (`v1.rs:154`), its code space found by the `<name>BUS` rule (`CoreDebugTarget.cs:225–233`); the description changes from "2A03" to "CPU" (Q4) | none |
| D4 | The PPU's registers: PPUCTRL, PPUMASK, PPUSTATUS, OAMADDR, the loopy v, t, x and w, the scanline, three status flags | `MoonRtDebugTarget.cs:162–173` | **minor**: §19.4's device registers, a device of kind `video` | 1.next |
| D5 | The APU's twenty registers `$4000`–`$4013`, the frame counter and its interrupt, then `Board:<name>` and the board's registers | `MoonRtDebugTarget.cs:175–189` | **minor**: §19.4, a device of kind `audio` and one of kind `cartridge` (B2, B3) | 1.next |
| D6 | Sprites: the 64 OAM entries, hidden below line 240, 8×8 or 8×16 by PPUCTRL bit 5; `MaxSprites` 64 | `MoonRtDebugTarget.cs:112, 191–206` | **pack**: `ISystemDebugViews` gains `Sprites` and `MaxSprites`, the NES pack's over the `OAM` space and the video device's PPUCTRL. PPUCTRL is write-only on the console (a read of `$2000` is open bus), so the pack has no route to the sprite height without §19.4, as the SNES's sprites had none without OBSEL (`EmuSen_CoreAPI.md` §31.1) | 1.next, through D4 |
| D7 | The five channels: active, a level of 0 to 100, muted, and a detail of duty, period, volume, length and sweep; derived from the APU's internal counters by three display rules (`Moon_Native.md` §9.4) | `MoonRtDebugTarget.cs:209–242` | **pack** over §19.4's `audio` device, which lists the internal counters the rules read (length counters, envelope decay, sweep, timer periods, the DMC's bytes remaining and output); `ISystemDebugViews` gains `AudioChannels(devices, mutes)`. The rules are facts of the 2A03, not of MoonRT, so they are the pack's | 1.next, through D5 |
| D8 | Two load bars, `CPU+APU` (the frame's time) and `PPU` (always 0) | `MoonRtDebugTarget.cs:245–251` | **host**, already: `CoreDebugTarget.HardwareLoad`'s one `frame` bar from the engine's timing. The `PPU` bar was C# Moon's, whose scheduler timed it; MoonRT never did (Q7) | none |
| D9 | The audio peek | `MoonRtDebugTarget.cs:256–257` | **host**, already: the same call, empty, since MoonRT claims no `AUDIO_PEEK` | none |
| D10 | Disassembly and the static references, by the NES pack's `Nes6502Disassembler` | `MoonRtDebugTarget.cs:261–288` | **core**: `DEBUG_DISASSEMBLE`, over the 6502 decoder moved out of `emusen-systems`' `nes::disasm` into a shared crate both use, as Diamond and the N64 pack share `emusen-mips` (§13.5). Alternative: a pack fallback in `CoreDebugTarget` for a core without the capability (Q2) | none |
| D11 | Tiles, tilemap entries, the tile sheet and the palette swatch; the palettes | `MoonRtDebugTarget.cs:66, 116, 290–296` | **pack**, already: `CoreDebugTarget` reads the same `NesDebugViews` | none |
| D12 | The three vectors | `MoonRtDebugTarget.cs:130` | **pack**, already | none |
| D13 | A CPU address's fixed destination: RAM, `PPUREG`, `APUREG`, `PRGRAM` | `MoonRtDebugTarget.cs:299–306` | **pack**: `ISystemDebugViews.ResolvePhysical`, the NES's fixed memory map, which `CoreDebugTarget` forwards | none |
| D14 | The summary: engine and frame and scanline; mapper, board name, PRG and CHR sizes, the nametable arrangement, battery; the CPU line; two PPU lines; and "APU no synthesis modelled - registers and the frame IRQ only" | `MoonRtDebugTarget.cs:308–324` | **host** for the head (`CoreDebugTarget.GetSummaryText`) and **pack** for the rest: `ISystemDebugViews.Summary` given the devices' registers as well as the main processor's; the sizes from the `PRGROM` and `CHR` spaces, the battery from machine info, the mapper and the arrangement from the `cartridge` device (B2), the board's name from `nes::boards`. The APU line is false, MoonRT's APU synthesising the five channels a player hears, and is dropped (C4) | 1.next, through D4 and D5 |
| D15 | The stores the core logs under spaces 8 and 9, named `PPUREG` and `APUREG` | `MoonRtCore.cs:67–79`; `memory/bus.rs:17–21, 239–253` | **core**: machine info lists `PPUREG` (8 bytes) and `APUREG` (32) as `view` spaces with `side_effects` and `reports_stores`, read and written through the CPU's bus at `$2000+a` and `$4000+a`. `CoreDebugBridge` drops a store whose space machine info does not list (`CoreDebugBridge.cs:201`), so without this a watch on `PPUREG` would never fire | none |
| D16 | A read of `CPUBUS` brings the ROM patches up to the registry first; a store to it is reported while a watch listens | `PortEngine.cs:260–283` | the read: **host**, `CoreEngine.ReadSpace` refreshing the patches on a machine that takes them, which serves VenusRT as well, once shown needed (C6); the report of a debugger's own store **retired**, since a poke from the shell is not a store the program made | none |
| D17 | Coverage records the entry points the call stack sees | `MoonRtCore.cs:29–30` | **host**: `CoreDebugBridge` wires `CallStackRegistry.EntryPointObserver` to processor 0's coverage, which no v1 core has today (C5) | none |
| D18 | Labels, the registries, `ApplyCheats`, the interrupt vectors' list | `MoonRtCore.cs:47–49`; `MoonRtDebugTarget.cs:103–119` | **host**, already: `CoreDebugTarget` and `CoreEngine` hold the same members | none |

`CoreDebugTarget` also brings what the shim's target lacks: the pack's expression symbols and the call stack's depth
for `eval`, and freezes (`EmuSen_CoreAPI.md` §31.1, items 15 and 19).

#### 19.3.4 `MoonBoards`

| # | What it adds | Where | Where it goes | ABI |
|---|---|---|---|---|
| B1 | Each board's name by mapper number | `MoonBoards.cs:16` | **pack**, already: `nes::boards` (§13.5) | none |
| B2 | Each board's nametable arrangement, as a rule over its fields in MoonRT's state | `MoonBoards.cs:18–24, 37–66` | **core**: the board's own arrangement, which MoonRT computes to run, published as a register `Mirroring` of the `cartridge` device in `NesMirroring`'s order, with `Mapper` beside it (§19.4); the sixteen rules restated in C# go | 1.next |
| B3 | Each board's registers for `regs`, by state path and width, MMC3's `IrqsFired` counter among them | `MoonBoards.cs:26–33, 42–64` | **core**: the `cartridge` device's registers, named and read by the board that holds them (§19.4) | 1.next |
| B4 | Which mappers MoonRT implements | `MoonBoards.cs:69–73` | **core**: the refusal of an unbuilt board at create (−10 with the mapper in its detail, `v1.rs:120–123`); its one other reader was M8's rule. No frontend asks before loading, so no core info field is proposed | none |

#### 19.3.5 The pre-stable machine the tests drive

| # | What it adds | Where | Where it goes | ABI |
|---|---|---|---|---|
| M14 | `MoonMachine` over the pre-stable exports, with its battery as file 0, `RunFrame`, the whole-pad `SetButtons` | `MoonMachine.cs:9–53` | **retired**: the step goldens and the patch tests move to `CoreMachine` on the v1 handle | none |
| M15 | `moonrt_step`, one instruction with the frame loop's DMA charge, NMI edge and stolen cycles | `MoonMachine.cs:11, 46`; `ffi/mod.rs:196–201` | **extension**: `moonrt_core_step` on the v1 handle, a test extension as `mercuryrt_core_step` is (`EmuSen_CoreAPI.md` §26.1); the step chains of `MoonGoldenTests` read it | none |
| M16 | `moonrt_rom_patch`, the patch table's answer for one read | `MoonMachine.cs:12, 52`; `ffi/mod.rs:208–211` | **extension**, already: `moonrt_core_rom_patch` (`v1.rs:306–308`), which `PortMachine.RomPatch` calls | none |
| M17 | MoonRT's pre-stable exports, `native_exports!` | `ffi/mod.rs:189` | **retired** once M14 to M16 are moved; CI's symbol for `moonrt` becomes `emusen_core_` (`rust-cores.yml:299`) | none |

**Counted:** 46 items across the two shims. 19 are carried already, by v1, the host or the pack, with nothing to
change; 11 move with a change to a core's own descriptors or to an optional capability v1 defines, to the host or to
the pack, none of them a surface; 2 are test extensions; 7 retire with their oracle; and **7 need the one addition of
§19.4**, four directly (D4, D5, B2, B3) and three through it (D6, D7, D14).

### 19.4 The one addition: device registers, proposed for core ABI 1.next

**What is missing.** v1 names registers only for processors (`processors[].registers`, read by `debug_registers`,
§6.4 and §6.14 of `EmuSen_CoreAPI.md`). The NES's PPU, its APU and the cartridge's board are not processors: they run
no code, have no program counter, and take no breakpoint. The shim reaches their registers by saving the whole state and
reading fields by path, which is the per-core code SR8 removes and the cost Q28 measured.

**The proposal: an optional `devices` list in machine info, answered by the existing `debug_registers`.**

```c
/* emusen_core.h, the comment on the existing export changes; no new prototype, no new capability bit */
/* DEBUG_REGISTERS: the values, in machine info's processors[].registers order; returns the count. @since 1.next the
 * id may also be a device's (machine info's devices[], ids 256 and above), the values in its registers order. */
int64_t emusen_core_debug_registers(const emusen_machine *machine, uint32_t processor, int64_t *out, size_t len);
```

```json
"devices": [
  { "id": 256, "name": "PPU", "kind": "video",
    "registers": [ { "name": "PPUCTRL", "bits": 8 }, { "name": "v", "bits": 15 }, { "name": "Scanline", "bits": 16 } ] },
  { "id": 257, "name": "APU", "kind": "audio",
    "registers": [ { "name": "$4000", "bits": 8 }, { "name": "FrameCounter", "bits": 8 }, { "name": "Pulse1.LengthCounter", "bits": 8 } ] },
  { "id": 258, "name": "Board", "kind": "cartridge",
    "registers": [ { "name": "Mapper", "bits": 12 }, { "name": "Mirroring", "bits": 3 }, { "name": "BankSelect", "bits": 3 } ] }
]
```

- **Ids** share `debug_registers`' argument with processors and start at 256, so they never meet a processor's (a
  processor is below 24, by the coverage flags' rule). `kind` is an open enumeration (`video`, `audio`, `cartridge`,
  `other`; a host shows a kind it does not know under `other`). Registers are `{ name, bits }` as a processor's.
- **Who may list it:** a core claiming `DEBUG_REGISTERS`. Listing it is the claim; no capability bit is added, since
  no export is. A host that does not know the field ignores it (§4.2); a host of 1.next reading a core of an older minor
  finds no field and shows no devices, as today.
- **The kit:** C14's rule extended: every device machine info lists is answered by `debug_registers` with its count of
  values, and an id machine info does not list is refused with `NOT_SUPPORTED` and changes nothing.
- **The crate:** `emusen-native`'s `MachineInfo` gains `devices: Vec<Device>` and `Device { id, name, kind,
  registers }`; the `Core` trait's `debug_registers` already takes any id.
- **The host:** `CoreDescriptorReader` reads `devices`; `CoreDebugTarget.VideoRegisters` is the `video` devices'
  registers and `ApuRegisters` the `audio` devices', followed for each `cartridge` device by a row `Board:<label>` and
  its registers, the label the pack's (`ISystemDebugViews.DeviceLabel`, the NES's from `Mapper` by `nes::boards`) or
  else the device's name, so that `regs` prints what it prints today. Each provider is filled from the ring each
  refresh, allocating nothing (§14.5).

**The forms weighed** (the project's rule for a feature that would widen a stable surface):

| Form | What it would be | Verdict |
|---|---|---|
| **Extension of an existing surface** | the field above, read by the export that already reads registers by id | **recommended**: it is the job `DEBUG_REGISTERS` already does, for a second kind of thing that holds registers; one call shape, one host path, one kit case |
| A new export | `debug_device_registers(m, device, …)` with its own bit | rejected: the same signature twice, and two exports a host must keep in step |
| A module or plugin | a DianaSh command module that reads devices | not a form that exists here: a module reaches a core only through the session service, which reaches it only through the ABI, so it would still need this surface underneath |
| A component of its own | a debugger broker reading cores' internals | rejected: it would cross the boundary §6 of `EmuSen_Surfaces.md` forbids crossing except by a written surface, which is this |
| No ABI: a view space per device | §30.6's preferred "space" route: a read-only space whose bytes are the registers, at offsets the NES pack defines | rejected for these: the loopy `v` and `t` are 15-bit internal registers, the scanline and the cycle count are counters and the APU's lengths and envelopes are internal state, none of them byte-addressed on the console, so the layout would be a second binary contract outside the ABI and its baseline; and a board's registers (MMC3's `IrqsFired` is MoonRT's own diagnostic counter) are facts of one engine, which a pack-defined layout cannot own (§5.3) |
| No ABI: processors with no program counter | each device as a `processors[]` entry with `pc_bits` 0 | rejected: `cpus`, `bp`, `step`, coverage and the call stack all take the list as processors, so each would need an exception |
| No ABI: keep reading the state by path | what the shim does | rejected: it is per-core host code (SR8) and a whole-state save per refresh (Q28) |

**Who else reads it.** VenusRT's §30.6 items 1 and 2 (the PPU's latched registers and the S-DSP's), which unblock its
item 6 (sprites) and the PPU lines of its summary; Nephrite's VDP and YM2612, which have no route today; MercuryRT's
PPU and APU at its own B4, where its shim reads them by path as MoonRT's does. The surface is shaped by two consoles'
needs and serves at least four.

**Its size**, estimated: the crate and header about 60 lines, the host's reader and providers about 120, the kit case
about 60, MoonRT's three devices about 250 (the board registers of eleven boards among them), the NES pack's sprites,
channels and summary about 250 in Rust and its C# twin until state 3.

**P3, retired in part.** §9's P3 held that every behaviour a shim adds fits an optional capability v1 already defines,
except MarsRT's counters. For MoonRT it does not hold: the PPU's, the APU's and the board's registers fit none. They
fit one extension of an existing capability, and everything else does fit, or retires.

### 19.5 What a player, the shell and a frontend would see change

Each is a difference the gates of §19.7 will show, listed so that it is decided rather than discovered:

1. **The engine's name in the shell**: `regs`, `spaces` and `cpus` head with "MoonRT" for "NES", and `dump`, `load`
   and `cov`'s files go under `Logs/MoonRT/` for `Logs/NES/` (`DumpCommand.cs:39`, `LoadCommand.cs:34`,
   `CovCommand.cs:89`); the session service's `engine` field reads "MoonRT". VenusRT reads "VenusRT" today (Q3).
2. **`cpus`** describes the processor "CPU" for "2A03", and lists `regs` among what it supports (Q4).
3. **`regs cpu`** prints the thirteen registers, where it said the processor reports none.
4. **`summary`**'s first line becomes the adapter's ("MoonRT (Rust) 0.1.0, nes, frame 2") with the pack's lines after
   it; the false APU line goes.
5. **`coretop`** shows one bar, `frame`, for `CPU+APU` and a `PPU` bar that was always 0 (Q7).
6. **`spaces`** gains `PPUREG` and `APUREG`, and `PRGROM` shows read-only.
7. **`eval`** gains the pack's symbols and `stackdepth`; **`freeze`** works on the NES.
8. **A refused game** reaches the frontend API as `Fault` with the core's words where it reached it as `NoEngine`
   (`ManagedRuntime.Open.cs:61`, `ManagedCrystal.cs:29`), the shim's `NotSupportedException` for an unbuilt board
   having been read as "no engine" (Q5).
9. **A refused state** carries the core's words ("MoonRT (Rust) refused the state: not this core's state.") for C#'s,
   and reaches the frontend API as `StateRefused` (C2).
10. **The engine notice** for a missing library is discovery's ("no sidecar for it was found beside the program", or
    the trust check's words) for the shim's ("… not found beside the assemblies"); `EMUSEN_MOON_NATIVE=0` no longer
    turns the core off (Q8).
11. **The registers' values** in the golden corpus's `moonrt` cases. The recorded `regs` and `summary` show the PPU's
    registers as zeros at frame 2 with the CPU's past reset; the exports read live values, so these lines may change.
    Each such line is to be checked against the machine's own state (`state_layout`) before it is re-recorded.

### 19.6 Candidate defects, to be shown by a failing test in phase 2 before any is fixed

- **C1. A battery file beside a game without one.** On the adapter's path a `.srm` that exists beside a game whose
  header claims no battery is passed as file 0 (`CoreEngine.cs:276–278, 397`) and written into PRG RAM by
  `v1.rs:126–135`; the shim never opened it (`MoonRtCore.cs:90`). Not a defect today, since the shim stands in front;
  it would become one at the switch. The fix is the core's (M8).
- **C2. A refused state escapes the frontend API's mapping.** The shim's pre-check throws `InvalidDataException`
  without the core's status, so `ManagedSession.LoadState` (`ManagedSession.States.cs:38`) does not report it as
  `StateRefused`; a foreign or truncated state on the NES reaches the caller as some other failure. Removed by the
  switch, which lets the core refuse the state.
- **C3. An unbuilt board reads as "no engine".** The frontend API's `NoEngine` for a game the one engine refuses
  (§19.5, 8). Neither `NoEngine` nor `Fault` says "refused"; recorded for the frontend API, not B4.
- **C4. The summary's APU line is false**: "no synthesis modelled" is a line carried over from C# Moon's target,
  and MoonRT mixes the five channels into every frame's sound (`MoonRtDebugTarget.cs:322`).
- **C5. No v1 core records coverage entry points**, which the shims wire (D17); `cov`'s entry list is empty on
  VenusRT, Nephrite and Diamond.
- **C6. A read between frames after a cheat changed** answers without the new ROM patches on the adapter, which
  refreshes them only at a frame or `ApplyCheats` (`CoreEngine.cs:479, 870–875`); the shim refreshed them on a bus read
  (D16). To be shown on VenusRT, which has the adapter's behaviour today, before the host is changed.

### 19.7 The gates for phase 2

#### 19.7.1 For each core

| Gate | MoonRT | VenusRT |
|---|---|---|
| Goldens | `MoonGoldenTests`, all 112 cases with the corpus (`EMUSEN_MOONRT_CORPUS`) and the games (`EMUSEN_MOONRT_ROMS=library`) present, the step chains through `moonrt_core_step`; no trace re-recorded | `VenusGoldenTests` (already through the adapter, so a check that nothing moved) |
| Corpus baseline | `MoonCorpusBaseline.tsv`, through `MoonGoldenTests`' corpus traces; `MoonTestRomTests`, `MoonAccuracyCoinTests` | `VenusRtTestRomRunnerTests.The_corpus_reproduces_the_recorded_baseline` over `VenusRtBaseline.tsv` (`EMUSEN_VENUSRT_CORPUS`); `VenusRtGoldenTests`' twelve games' hashes (G5, `EMUSEN_VENUSRT_GOLDENS=library`) |
| The kit | `emusen-core-conform` on the four images of `EmuSen_CoreAPI.md` §28.15, `--self-delimiting`, 600 frames, every case passing as on 2026-10-16 | the same on its two images there, and on the test-ROM corpus's images (§19.9) |
| The bench | `moonbench`'s state hash per game unchanged (`0ED799A150F20023` for Super Mario Bros., `EmuSen_CoreAPI.md` §26.3) | — |
| The debugger's corpus | `GoldenCorpusTests`, every `moonrt` case unchanged or its difference one of §19.5's, each decided before re-recording | unchanged |
| No per-core type | `grep -rn "EmuSen.Cores.Nintendo.MoonRT"` in shipped code: 0 | `grep -rn "EmuSen.Cores.Nintendo.VenusRT"`: 0 |

#### 19.7.2 The WiseMan classes that cover the shims, measured before any change

Run once on 2026-10-18 at `480bf5e2` with no opt-in variable set, so that each opt-in case passes unrun as it does in
CI (§19.9 runs the VenusRT ones opted in):

| Class | Cases | Reaches |
|---|---:|---|
| `MoonGoldenTests` | 112 | MoonRT's shim and `MoonMachine`; the corpus, step and game traces pass unrun without their variables |
| `MoonRtRawPictureTests` | 69 | MoonRT's shim |
| `BatterySaveTests` | 17 | MoonRT's shim (the battery rule) |
| `ConsoleBindingsTests` | 16 | MoonRT's shim |
| `CoreFactoryCheatWiringTests` | 13 | MoonRT's shim (`Bundle`) |
| `DisplayAspectTests` | 13 | MoonRT's shim (the shape) |
| `MoonSpriteZeroTests` | 13 | MoonRT's shim |
| `MoonCheatTests` | 9 | MoonRT's shim |
| `MoonDebugEngineTests` (with `MoonDebugTargetContract`) | 8 | `MoonRtDebugTarget` |
| `MoonRtFrontendTests` | 8 | MoonRT's shim through the factory |
| `MoonTestRomTests` | 8 | MoonRT's shim |
| `AudioBufferLimitTests` | 6 | MoonRT's shim |
| `ManagedCrystalTests` | 6 | MoonRT's shim through the frontend API |
| `MoonRtPalTests` | 6 | MoonRT's shim |
| `EngineFeaturesTests` | 5 | MoonRT's shim |
| `NativeHostTests` | 5 | MoonRT's two libraries' loaders |
| `ControllerPortTests` | 4 | MoonRT's shim |
| `MoonRtEngineTests` (Mistress) | 4 | MoonRT's shim and its switch |
| `CoreRegistrationEquivalenceTests` | 3 | both switches, in its "off" world |
| `MoonRtCoreAbiTests` | 3 | MoonRT's shim against the adapter |
| `PadButtonTests` | 3 | MoonRT's shim |
| `RustStateHeaderTests` | 3 | MoonRT's shim (the state's magic) |
| `MoonAccuracyCoinTests` | 2 | MoonRT's shim |
| `MoonPpuDefectTests` | 2 | MoonRT's shim |
| `RustShimDisposeTests` | 2 | MoonRT's shim |
| `StateRefusalTests` | 2 | MoonRT's shim (the state refusal) |
| `MistressStateRefusalTests` | 1 | MoonRT's shim |
| `NesRunToFrameTests` | 1 | MoonRT's shim |
| `ScreenFilterRenderTests`, the MoonRT case | 1 | MoonRT's shim |
| `VenusRtCoreAbiTests` | 9 | `VenusMachine`, `VenusNative` |
| `VenusRtTestRomRunnerTests` | 11 | `VenusMachine` through `VenusRtSnesEngine`, `VenusNative` |
| `VenusRtGoldenTests` | 3 | `VenusMachine` through `VenusRtSnesEngine` |
| `VenusGoldenTests` | 24 | neither (VenusRT through the adapter), run as the check that nothing moved |
| `VenusRtDebugTests`, `VenusRtDebugViewTests` | 19, 11 | neither (`CoreDebugTarget`), the same check |
| `GoldenCorpusTests` | 100 | `MoonRtDebugTarget` on its `moonrt` cases; `CoreDebugTarget` on VenusRT's |
| **All** | **522** | **522 passed, 0 failed, 0 skipped**, 3 min 49 s |

`ScreenFilterRenderTests`' area case that makes a `MoonRtCore` (`ScreenFilterRenderTests.Area.cs:212`) was not in the
filter, nor were `CoreOutputHashProbe` and `ReferenceDumpSnapshots`, which are opt-in probes; all three are moved
in phase 2 with the rest. The class's other cases reach no shim and were left out.

#### 19.7.3 The classes that name a shim type, to be moved in phase 2

MoonRT: `AudioBufferLimitTests`, `BatterySaveTests`, `ControllerPortTests`, `CoreFactoryCheatWiringTests`,
`CoreOutputHashProbe`, `CoreRegistrationEquivalenceTests`, `EngineFeaturesTests`, `MoonAccuracyCoinTests`,
`MoonCheatTests`, `MoonDebugEngineTests` (with `MoonDebugTargetContract`), `MoonGoldenTests` and `Fixtures/MoonGolden.cs`,
`MoonPpuDefectTests`, `MoonRtCoreAbiTests`, `MoonRtFrontendTests`, `MoonRtPalTests`, `MoonRtRawPictureTests`,
`MoonSpriteZeroTests`, `MoonTestRomTests`, `NativeHostTests`, `NesRunToFrameTests`, `PadButtonTests`,
`RustShimDisposeTests`, `RustStateHeaderTests`, `ManagedCrystalTests`, `StateRefusalTests`, `ConsoleBindingsTests`,
`MistressStateRefusalTests`, `MoonRtEngineTests` (Mistress), `ReferenceDumpSnapshots`, `DisplayAspectTests`,
`ScreenFilterRenderTests` (two cases). Most hold an `ICore` and need only `CoreFactory.Create`; the ones that read the
shim's own members (`Header`, `Layout`, `Pc`, `Machine.RomPatch`, `IsChannelMuted`) read machine info, the registers
and the extensions instead.

### 19.8 The build plan for phase 2

**VenusRT first, and done** (§19.9): no ABI change, no change to its machine, tests only.

**MoonRT, in this order**, each step graded before the next:

1. **The core, with no ABI change**: `DEBUG_REGISTERS` for the 2A03 (D2); `PRGROM` read-only (D1); `PPUREG` and
   `APUREG` as view spaces (D15); `create` taking file 0 only for a cartridge with a battery, C1 shown first (M8);
   `moonrt_core_step` (M15); and `DEBUG_DISASSEMBLE` on the shared 6502 crate if Q2 is answered so. The machine and
   its state do not change, so the goldens and the bench hashes must not move.
2. **The host and the pack, with no ABI change**: `ResolvePhysical` in the NES pack (D13); the entry-point observer
   (C5); the patch refresh on a read if C6 is shown; the pack's disassembler fallback if Q2 is answered the other way.
3. **The tests moved** (§19.7.3) from `MoonRtCore` and `MoonMachine` to `CoreEngine`, `CoreMachine` and the
   extensions, run while the shim still stands, so that each moved test is shown to pass on both before the switch.
4. **If §19.4 is approved**: the crate and header at 1.next, the host's reader and providers, the kit case; MoonRT's
   three devices with `MoonBoards`' registers and arrangement rules moved into the boards that hold them (B2, B3); the
   NES pack's sprites, channels, board label and summary over them (D6, D7, D14). If it is declined, `regs` keeps the CPU
   alone and the PPU, APU, board, sprite and channel views are lost on the NES at the switch, recorded in
   `Moon_Debug.md` as such.
5. **The switch**: `CoreFactory`'s `.nes` arm and `Bundle`'s `MoonRtCore` arm go, `MoonRtEngine` joins
   `SoleDiscoveredEngines`, and the shim's five files are deleted; MoonRT's `native_exports!`, `moonrt_step` and
   `moonrt_rom_patch` go, CI's symbol becomes `emusen_core_` (M17).
6. **The gates** of §19.7, and the `GoldenCorpusTests` differences re-recorded only as decided.

After MoonRT, `PortEngine` has one subclass, `MercuryRtCore`, and the pre-stable set two callers, MercuryRT's shim and
VenusRT's internal use of `NativeCore` (§19.9); both go with MercuryRT's B4.

### 19.9 Built in this phase: VenusRT's shim retired

*Built on 2026-10-18 on branch `b4-moon-venus`, as a commit of its own, `060dd905`, before this section's.*

**What was done.**

- `Shim/VenusMachine.cs` and `Shim/VenusNative.cs` are deleted, 67 lines; nothing shipped named them, and
  `grep -rn "EmuSen.Cores.Nintendo.VenusRT"` over the tree now finds nothing.
- **The runner** (`VenusRtSnesEngine`, `Fixtures/Snes/SnesEngines.cs`) drives the bare v1 machine, `CoreMachine` over
  the library beside the tests, opened by its path. It passes the same files as before (1 the boot image when given, 2
  the DSP's firmware when found) and no settings, so the machine runs at the schema's defaults, which are the accurate
  ones; `DspRequest` asks the same library's `firmware_for`.
- **`VenusRtCoreAbiTests`** compares the adapter with the bare v1 machine under it, the picture and the sound every frame
  and the state at the end, where it compared it with the pre-stable machine over the same Rust; what it holds, that the
  adapter adds nothing to what the core makes, is unchanged. **`VenusRtTestRomRunnerTests`**' library case now holds
  that no pre-stable export is left in the library, and that a short image is refused with −9 through v1.
- **The library** no longer invokes `native_exports!` (`ffi.rs`): it defines 49 `emusen_core_` symbols and no
  `emusen_native_` symbol, where it defined 24 of those beside the 49, and `export_check.py` passes it against the 1.12
  baseline. Its `impl NativeCore for Machine` stays as the helper v1's `create` and `battery` call (Q9). CI's symbol for
  `venusrt` is `emusen_core_` (`rust-cores.yml:300`).
- **The switch** `EMUSEN_VENUS_NATIVE` has no reader left (Q8). `CoreRegistrationEquivalenceTests.OffVariables` still
  names it, to no effect; MoonRT's step edits the same line and tidies it.

**The gates, measured on 2026-10-18.** The corpus, G5's games and the goldens are graded against their committed
records, so a pass after the change is the comparison with before; the crate's tests and the kit were run on both
sides. Reference runs were left out (`EMUSEN_VENUSRT_NO_MESEN=1`, `EMUSEN_MESEN_PROBE` pointed at nothing), so Mesen's
columns are the baseline's frozen ones and no reference run wrote a firmware image into its cache.

| Gate | At `480bf5e2` | With the shim retired |
|---|---|---|
| WiseMan's blast radius: `VenusRtCoreAbiTests` 9, `VenusRtTestRomRunnerTests` 11, `VenusRtGoldenTests` 3, `VenusRtDebugTests` 19, `VenusRtDebugViewTests` 11, `NativeHostTests` 5, `CoreRegistrationEquivalenceTests` 3, `GoldenCorpusTests` 100 | 161 of 161 pass (within §19.7.2's run) | 161 of 161 pass, 5.4 min, the opt-in cases below included |
| The test-ROM corpus, `The_corpus_reproduces_the_recorded_baseline`, 295 ROMs (`EMUSEN_VENUSRT_CORPUS`) | — | passes in 4 min 9 s: every VenusRT verdict and detail as `VenusRtBaseline.tsv` holds them |
| G5, `The_goldens_against_Mesen`, the twelve games from the library (`EMUSEN_VENUSRT_GOLDENS=library`) | — | passes: every one of the 36 anchors' hashes as recorded, no game skipped |
| `VenusGoldenTests` with the corpus and the games (`EMUSEN_VENUSRT_ROMS=library`) | 24 of 24 pass, unopted | 24 of 24 pass in 7.1 min: the 295 corpus traces in 5 min 8 s, the six games' in 1 min 20 s |
| The crate, `cargo test --profile test-checked` | 145 pass | 145 pass |
| The kit, `emusen-core-conform` at 600 frames on 17 images of the corpus (every eighteenth line of its `unique-roms.txt`), four at a time | the core's cases and 17 of 17 images pass, 40.3 s | the same, every verdict and every line of evidence the same, but the timings, the memory figures and C13's count of rounds |

**Not done here.** The kit's two commercial images of `EmuSen_CoreAPI.md` §28.15 were not run, the corpus's images
standing in; the C# side of the pre-stable host (`NativeInterface`, `NativeMachine`, `NativeCoreLibrary`) keeps its
other callers, MoonRT's and MercuryRT's machines and `NativeHostTests`.

### 19.10 Questions for the tester

Asked each on its own; none is decided by this section.

- **Q1. Device registers** (§19.4): add `devices` to machine info at core ABI 1.next, answered by `debug_registers`,
  as an extension of `DEBUG_REGISTERS` with no new export or bit? *Recommended: yes*, for the reasons of §19.4's table.
  Declined, MoonRT's switch loses the PPU, APU, board, sprite and channel views (§19.8, step 4), unless a pack-defined
  register space is chosen instead for the PPU and the APU, which keeps no board registers.
- **Q2. MoonRT's disassembly** (D10): (a) the core claims `DEBUG_DISASSEMBLE`, its decoder a shared crate in
  `EmuSen/Cores/Shared/` that `emusen-systems`' `nes::disasm` moves into and both use; or (b) `CoreDebugTarget` falls
  back to the console's pack for a core without the capability. *Recommended: (a)*: the ABI's rule is that the core
  decodes (`EmuSen_CoreAPI.md` §6.14), the host keeps one disassembly path, the crate has one owner as `emusen-mips`
  has for the MIPS cores and the N64's pack, and MercuryRT's SM83 can take the same road at its own B4.
- **Q3. The engine's name** in the shell's headings, the session service's `engine` and the debugger's log folder
  becomes "MoonRT" for "NES" (§19.5, 1)? *Recommended: yes*, as VenusRT's is "VenusRT"; files a player already has
  under `Logs/NES/` are left where they are, being the debugger's own dumps rather than saves.
- **Q4. The processor's description** "CPU", machine info's name, for "2A03" (D3)? *Recommended: yes*, the name every
  other core's main processor has in machine info, so that `cpus` reads alike on every console.
- **Q5. The oracle's exception types retire** (M10), so a game MoonRT refuses reaches the frontend API as `Fault` and a
  refused state as `StateRefused` (§19.5, 8 and 9)? *Recommended: yes*. Whether a refused game deserves a number of its
  own in the frontend API (C3) is a question for that API, not for B4.
- **Q6. The board registers**, MMC3's diagnostic `IrqsFired` counter among them, kept in `regs` as the `cartridge`
  device's (B3)? *Recommended: yes*: `Moon_Debug.md` §3.1 records what the counter found, and a register costs nothing
  until it is read.
- **Q7. One load bar**, `frame`, for the shim's `CPU+APU` and `PPU` (D8)? *Recommended: yes*: the `PPU` bar has read 0
  since C# Moon left, MoonRT timing no phases.
- **Q8. The switch variables retire**: `EMUSEN_MOON_NATIVE` with MoonRT's shim, and `EMUSEN_VENUS_NATIVE`, whose only
  reader was VenusRT's (deleted in §19.9); `EmuSen_CoreAPI.md` §13.4's "the switch variables stay as an override" is
  amended to say that discovery reads none, a library being left out by removing it or its sidecar? *Recommended: yes*.
  For VenusRT this records what was already so for every client but the deleted test path since `VenusRT_Native.md`
  §68.5.
- **Q9. VenusRT's `NativeCore` implementation** stays as the helper v1's `create` and `battery` call, exporting
  nothing, until the pre-stable set is deleted with MercuryRT's shim (§19.9)? *Recommended: yes*: moving its bodies into
  the v1 trait now touches VenusRT's create path for no change in behaviour, and the module goes then in any case.
- **Q10. The order**: MoonRT's steps 1 to 3 of §19.8, which need no ABI change, built while Q1 is open? *Recommended:
  yes*: they are independent of Q1, each is graded on the goldens as it lands, and they leave the switch small.

---

## 20. B4's decisions, 2026-10-18

The three first phases (§17, §18, §19) were answered by the tester on 2026-10-18, each as its section recommended:

- **Core ABI 1.13 is one minor of six items**: the registers of the parts that are no processor (§17.3's `units` and
  §19.4's `devices`, one proposal under two names, built once under one name), and §18.4's five: `TELEMETRY`,
  `frame_info.lines`, `PRESENCE`, `EMUSEN_FRAME_CHEATS_HELD` and `DEBUG_TRANSLATE`. P3 is retired as wrong (§9).
- **The names a retired shim showed give way to the generic host's**: the engine's name in the shell's headings and log
  folders ("MoonRT", "MercuryRT" for "NES", "GB"), "CPU" for "2A03", one load bar for the shim's several; the debugger's
  golden corpus is re-recorded with each difference listed, as §17.4, §18.6.3 and §19.5 list them.
- **The order**: one branch builds the host pieces the three cores share (§17.7 step 2, §18.7, §19.8) and the minor,
  merged before any core switches to the generic host; meanwhile each core's steps that need no ABI change go ahead.
- **Every other question** of §17.8, §18.8 and §19.10 as recommended, among them: the switches `EMUSEN_MOON_NATIVE`,
  `EMUSEN_MERCURY_NATIVE`, `EMUSEN_MARS_NATIVE` and `EMUSEN_VENUS_NATIVE` retired with their shims, there being no C#
  left to fall back to; Hotaru's pad bindings keyed by console rather than engine, fixed before a switch; the SM83 and
  6502 disassemblers in shared crates the cores and the packs both use; the IS-Viewer's output in the core's log; the
  oracles' exception types retired, so a refused game reaches the frontend API as a fault and a refused state as
  `StateRefused`.

---

## 21. B4's second phase for MoonRT, steps 1 to 3, 2026-10-18: the core's own exports, the pack's fixed map, the tests moved

*Built on branch `b4-moon-venus` from `WiseMan` at `4b42f4f1`, under §20's decisions: §19.8's steps 1 to 3, the ones that
need no core ABI change and no host piece the cores share. The entry-point observer (C5), the patch refresh on a read
(C6), `ISystemDebugViews`' new members, the oracles' exception types' retirement and core ABI 1.13 are another
branch's, `b4-shared`; step 4 (MoonRT's devices) waits for it, and step 5 (the switch) for step 4. MoonRT's machine and
its state did not change: every golden, the corpus and the games' traces hold as recorded.*

### 21.1 Step 1: the core

- **C1, shown first.** `MoonRtCoreAbiTests.A_battery_file_beside_a_game_without_one_is_not_loaded_on_the_adapter`
  puts a `.srm` of `A5h` beside a game whose header claims no battery and loads it on the shim and on the generic
  engine. Against the unchanged library the shim read 0 and the adapter **165**, the save written into PRG RAM
  (§19.6). MoonRT's v1 `create` now reads file 0 only for a cartridge with a battery and leaves another's unread; the
  case passes, and the crate's `a_battery_file_is_taken_only_by_a_cartridge_with_a_battery` holds it beside machine
  info, which lists the file only then. The pre-stable `create`, which `MoonMachine` alone calls, is unchanged.
- **`DEBUG_REGISTERS`** (Q28): machine info's processor 0 names the shim target's thirteen, A, X, Y, S, PC, P, the six
  flags N, V, D, I, Z and C as 1-bit values and the 64-bit cycle count, and `debug_registers` reads them from the CPU's
  own fields, not from a saved state.
- **`PRGROM` read-only.** Flagged `read_only`, and a write to it is refused with `READ_ONLY` and changes nothing:
  §19's D1 named only the flag, but the kit's C14 holds a `read_only` space to refusing a write, which the core's
  `write_space` had silently ignored.
- **`PPUREG` and `APUREG`** (spaces 8 and 9, 8 and 32 bytes), the ids the core already logged stores under, now listed
  with `side_effects` and `reports_stores`, read and written through the CPU's bus at `$2000+a` and `$4000+a`, nothing
  past their ends. **Not `view`, correcting §19's D15**: §29 of `EmuSen_CoreAPI.md` has a view promise that it is
  read-only and that its reads change nothing, and a read of `$2002` clears the vertical blank.
- **`moonrt_core_step`**, the test extension on the v1 handle, with `moonrt_step`'s body: one instruction with the frame
  loop's DMA charge, NMI edge and stolen cycles.
- **`DEBUG_DISASSEMBLE` over a shared crate** (Q2). `emusen-systems`' `nes::disasm` moved whole into
  `EmuSen/Cores/Shared/emusen-6502` (the table, the modes, lengths, operand text and references, with a `Reference`
  of its own), which MoonRT's `debug_disassemble` and the NES pack both read; the pack's module re-exports it and maps
  the references to its own, so `emusen_systems_disassemble` and `DebugCodeParityTests` are unchanged. The core reads
  each instruction as the shim's target did, the opcode and then its bytes, through the space asked for. **Two new
  dependencies, stated as the project's rule asks**: MoonRT on `emusen-6502`, and `emusen-systems` on it, as both
  sides already depend on `emusen-mips` for the MIPS cores.
- **A correction to §19's D16**: a host's store to `CPUBUS` is reported by MoonRT itself when the debugger listens to
  stores (`machine.rs`, `write_space`'s space 7, under the last instruction), so nothing retires there; the shim's
  `Observed` path only routes the same report.

MoonRT now exports 51 `emusen_core_` symbols (49 before) and claims `0x433821` (`0x403821` before);
`export_check.py` passes it against the 1.12 baseline. `CoreDebugTarget` over the v1 engine shows the 2A03's
registers and disassembly exactly as the shim's target does:
`MoonRtCoreAbiTests.The_cores_registers_and_disassembly_are_what_the_shims_target_shows` compares the registers after
30, 60 and 90 frames of a busy MMC3 program, and 64 instructions from six addresses (the program, its loop, the reset
code, the vectors, RAM and PRG RAM) with their static references.

### 21.2 Step 2: the NES pack's fixed memory map

`nes::views::resolve_physical` in Rust and `NesDebugViews.ResolvePhysical` in C#: RAM, `PPUREG`, `APUREG` and `PRGRAM`
for the ranges the console fixes, nothing for `$8000` and up. **Not wired**: `ISystemDebugViews` has no such member, and
adding one is `b4-shared`'s, with the facade's call and its parity case; `CoreDebugTarget.ResolvePhysical` forwards to it
then. Until then `MoonRtCoreAbiTests.The_nes_packs_fixed_map_is_the_shim_targets` holds the C# function to the shim
target's for every address from −1 to `$10000`, and the crate's test pins the Rust function's edges.
`NesSystem` names the two windows (`PpuRegisters`, `ApuRegisters`).

### 21.3 Step 3: the tests moved

**The mechanism.** `Fixtures/MoonEngines.cs` makes the engine a moved test runs: the generic `CoreEngine` over MoonRT's
library beside the tests, opened by its path, or, while `EMUSEN_MOONRT_ENGINE=shim`, the shim, so that every moved test
runs on both until the switch, when the variable and its branch go. The shim is a `CoreEngine` subclass, so a moved test
holds a `CoreEngine` either way. The step chains run on a bare v1 machine through `moonrt_core_step`, or on `MoonMachine`
with the variable set. `MoonEngines.Field` reads a state field by the machine's layout listing, for the tests that read
the shim's `Layout`.

**The corpus traces' word for a refused image.** Eight corpus traces record `verdict Unsupported NotSupportedException`,
the shim's exception for a board MoonRT does not build. The generic engine refuses the same image with
`CoreRefusedException` and status −10, so `MoonEngines.RefusalWord` reads a refusal by its status into the word the
traces hold (−10 `NotSupportedException`, −9 and −11 `InvalidDataException`), and the traces stand on both engines.
Whether they are re-recorded with words of the core's own is the switch's question, as the exception types retire
there (§20).

**Moved**: `MoonGoldenTests` and `Fixtures/MoonGolden.cs` (the frame runs, the corpus, the games and the step chains),
`MoonRtRawPictureTests`, `MoonRtPalTests`, `MoonSpriteZeroTests`, `MoonTestRomTests`, `MoonPpuDefectTests`,
`MoonAccuracyCoinTests`, `NesRunToFrameTests`, `RustShimDisposeTests`, `RustStateHeaderTests` (the header read as "MOON"
and 6 rather than from the shim's constants), `PadButtonTests`, `EngineFeaturesTests`, `AudioBufferLimitTests`,
`BatterySaveTests` (its MoonRT rows) and `ScreenFilterRenderTests`' MoonRT case.

**Not moved, and why:**

| Class | What holds it to the shim |
|---|---|
| `MoonCheatTests` | it reads the bus after adding a patch, between frames, which the adapter answers without the patch until a frame (C6, `b4-shared`) |
| `MoonDebugEngineTests` with `MoonDebugTargetContract`, and `ManagedCrystalTests`' MoonRT case | the debugger's views and registries through the shim's target; the generic target needs `b4-shared`'s pieces and step 4's devices |
| `MoonRtFrontendTests`, `CoreFactoryCheatWiringTests`, `ControllerPortTests`, `CoreRegistrationEquivalenceTests`, `MoonRtEngineTests`, `StateRefusalTests`, `MistressStateRefusalTests` | what `CoreFactory` and the frontend API build for a `.nes`, which changes at the switch, and the oracle's exception types and switch variable, which retire there |
| `ConsoleBindingsTests`, `DisplayAspectTests`, `ScreenFilterRenderTests`' area case | the shim's identity before a game: "NES" for its name, a 4:3 shape and its lines with no game, which §19.5's decided differences change at the switch |
| `NativeHostTests` | the pre-stable host itself, which stays until MercuryRT's shim goes |
| `CoreOutputHashProbe`, `ReferenceDumpSnapshots` | opt-in probes, moved at the switch with the factory |

### 21.4 The gates, measured on 2026-10-18

| Gate | Result |
|---|---|
| C1 against the unchanged library | fails: expected 0, actual 165 |
| WiseMan, the blast radius on the generic engine (the moved and not-moved classes above, `MoonRtCoreAbiTests` 6, `DebugCodeParityTests` 5 and `GoldenCorpusTests` 100), with the corpus (`EMUSEN_MOONRT_CORPUS`) and the games (`EMUSEN_MOONRT_ROMS=library`) | **453 of 453 pass**, 6 min 30 s: the corpus's 263 ROMs as recorded, the seven games' 3,600 frames each as recorded and again from their recorded states, every step chain through `moonrt_core_step`, 105 corpus passes as the C# baseline had |
| The same with `EMUSEN_MOONRT_ENGINE=shim` | **453 of 453 pass**, 6 min 27 s |
| `GoldenCorpusTests` | 100 of 100 in each run, nothing re-recorded |
| MoonRT's crate, `cargo test --profile test-checked` | 26 pass, the four new among them |
| `emusen-6502` | 3 pass |
| `emusen-systems`, `emusen-platform` | 31, 4 and 20 pass (the 20 with `resolve_physical`'s case) |
| The kit, `emusen-core-conform` at 600 frames, `--self-delimiting`, on 20 corpus images (every thirteenth `.nes` of the corpus, less a board MoonRT does not build), the library at `4b42f4f1` and this one | the core's cases and 20 of 20 images pass on both; the reports differ only in the two exports and the capability word, and in timings, memory and C13's rounds |

`moonbench`'s hashes (§19.7.1) were not taken again: it is a program kept outside the tree, and the games' traces hold
the same games' state hashes under the bench's input at every frame.

### 21.5 What is left

- **After `b4-shared` merges**: `CoreDebugTarget.ResolvePhysical` wired to the pack's function; `MoonCheatTests` moved
  once C6's refresh is in the host; step 4, MoonRT's three devices under 1.13's one name, with `MoonBoards`' registers
  and arrangement rules moved into the boards, and the pack's sprites, channels and summary over them.
- **Step 5, the switch**: `CoreFactory`'s `.nes` arm, `MoonRtEngine` to `SoleDiscoveredEngines`, the shim's five files,
  MoonRT's `native_exports!`, `moonrt_step` and `moonrt_rom_patch`, `EMUSEN_MOONRT_ENGINE` and its branch, the classes of
  §21.3's second table, and the eight corpus traces' refusal word decided.

---

## 22. B4's shared part, 2026-10-18: the host pieces the cores share, and core ABI 1.13

*Built on 2026-10-18 on branch `b4-shared`, from `WiseMan` at `4b42f4f1`, under §20's decisions: one branch builds
the host pieces the three B4 cores share (§17.7 step 2, §18.7 steps 4 and 5's host and bridge parts, §19.8 step 2's
host parts) and the minor, merged before any core switches to the generic host. No core was switched and no shim was
deleted; MercuryRT, MarsRT and MoonRT claim none of the minor's items, which their own branches take. Claims are marked
as at the head of this page.*

### 22.1 The candidate defects, shown before anything was fixed

Each was written as a test in `SharedHostTests` and run on a tree with nothing of the fixes in it (a throwaway copy of
`4b42f4f1` with only the tests, the two markers the test core gained for them, and Hotaru's key named as a member that
still returned the engine's name) [M]. All thirteen cases failed there, each on its own assertion, and pass after:

| Candidate | The test | Today's answer | After |
|---|---|---|---|
| §17.5 C2, §18.3 C1, §19.6 C2: a refused state through the frontend API | a foreign state of 64 bytes and a short one of 4, on a Game Boy, an NES and an N64 game | `Fault` (−1108) on all three | `StateRefused` (−1114) |
| §19.5 item 8: a game no board builds | a mapper-200 NES image | `NoEngine` (−1109) | `Fault`, "MoonRT (Rust) refused the game: mapper 200 …" |
| §17.5 C3: a game's own create-time setting | the test core's `Ram`, set for the game, the game opened again | 7, the console's | 42, the game's |
| §17.2 E2: a create-time setting before the first frame | `Set("Ram", "42")` with no frame run, then `9` after one | 7 | 42 at once; `9` kept for the next load |
| §18.3 C2: the call stack across a new machine | a call recorded, the game loaded again | depth 1 | 0 |
| §18.3 C3, F13: an interrupt stopped for without a call | `runto irq` on an image whose loop stops for interrupts and logs no call | never halts | halts at frame 3 |
| §18.2 F7: a processor that cannot halt | an image with a second processor whose breakpoints are refused | `helper` offered a halt | `helper` cannot halt |
| §19.6 C5: coverage's entry points | a call pushed with coverage armed | none recorded | the target recorded, as a call |
| §17.2 E8: labels | a label made through one target, read through another | absent | present |
| §19.6 C6: a read between frames after a patch | a ROM patch at `$00:8005` on VenusRT, `CpuBus` read before the next frame | the unpatched byte | the patched byte |
| §17.5 C4: Hotaru's binding key | the key for a test-core session | `V1TestCore`, the engine | `TEST`, the console |

**One cause or two.** The refused state was one cause in two shapes: every shim answered the core's refusal with an
exception that carried no `emusen.status`, which is what `ManagedSession.LoadState` takes a refusal by. MercuryRT's and
MoonRT's shims threw their oracles' `InvalidDataException` and `EndOfStreamException` from a pre-check before the core
saw the bytes (`PortEngine.CheckState`), and MarsRT's shim threw a new `InvalidDataException` around the adapter's,
dropping its data. The fix is one rule, the one §20 decided: the oracles' exception types retire, so the core refuses
and `CoreMachine.Load`'s exception, which carries the status, reaches the API.

### 22.2 What was built

- **The oracles' exception types retired where they reach the frontend API.** `PortEngine.CheckState` and its two
  overrides are gone, so a state reaches the core and is refused in its words with its status; `PortEngine.LoadRom`
  no longer maps the core's refusal to an oracle's type, so a refused game is `CoreRefusedException`, a fault, as on
  every v1 core; `MarsRtCore.LoadState` no longer wraps the refusal. *Not retired here*: the frame-time mapping
  (`PortEngine.AdvanceFrame`, MoonRT's `Reset`), which MercuryRT's hardware-test runner reads as its "illegal opcode"
  verdict (`MercuryHardwareTestRomTests`), and Mercury's header pre-check (`GbCartridge.Read`, §17.2 L1); both go with
  their shims at the core branches' switches.
- **`CoreEngine` owns a `LabelRegistry`**, which `CoreDebugTarget.Labels` returns.
- **`CoreDebugBridge`**: the call stack reset at each new machine; coverage's entry-point observer wired to processor
  0's coverage; an interrupt noted from `STOP_INTERRUPT` when the round's call log held none (from the log when it
  did, so it is noted once); and the halting probe, an empty table pushed to each processor once a machine, a refusal
  marking it as one that cannot halt, its tables no longer pushed and its `DebugCpu.CanHalt` false. `EmuSen_CoreAPI.md`
  §6.14 gained the clarification, no minor.
- **`CoreEngine.ReadSpace` brings the ROM patches up first**, a version compare when nothing changed.
- **A create-scope `Set` before the first frame makes the machine again**, the machine "fresh" from its create until
  a frame runs or a state loads; a refusal at create puts the old value back and keeps the old machine.
  `CoreEngine.Keep` holds a value for the next load alone, for MarsRT's shim's `Boot`, which sets create-time keys
  around its own load.
- **`RomPatchTriples` moved into `CoreEngine`**; `NativeRtCore` calls it there.
- **Hotaru keys pad bindings by the session's console** (`HotaruSession.BindingsConsoleOf`), as Mistress does; logs,
  screenshots and recordings stay under the engine's name.
- **The pack views' shared surface**, `EmuSen_CoreAPI.md` §31.7: `ISystemDebugViews` gained `MaxSprites`, `Sprites`,
  `AudioChannels`, `ResolvePhysical` and `UnitLabel`, and `Summary`, `DmaChannels` and `DecodeTilemapEntry` overloads
  taking the spaces and the units; `DebugUnit` is a unit as a pack sees it; `CoreDebugTarget` wires each, the
  palettes', sprites' and channels' copies through one ring (`ViewRing<T>`). **No console fills them**: each fill
  needs the core's own registers and spaces and so is the core branch's, with its Rust twin in `emusen-systems`. The
  platform library's C interface did not change, so this part takes no platform interface number (it is 18 here, and
  19 is MarsRT's branch's): `emusen_systems_view` takes a
  view by name, and the new members answer the C# views' until a console adds its Rust twins.

### 22.3 Tested

The blast radius of the first part, measured on 2026-10-18 on the branch's build, Debug, .NET 10.0.112 [M]: 57 classes,
every class that names a shim or its rig, drives `CoreEngine`, `CoreDebugBridge` or `CoreDebugTarget` on any v1 core,
or reaches a state, a setting, a cheat or the debugger through the frontend API, with `GoldenCorpusTests`:
`SharedHostTests`, `CoreAdapterTests`, `CoreEngineThreadTests`, `BatterySaveTests`, `BreakpointConditionTests`,
`CheatCodecAdapterTests`, `ControllerPortTests`, `CoreFactoryCheatWiringTests`, `CoreRegistrationEquivalenceTests`,
`DebugPlumbingTests`, `DiamondDebugTests`, `DiamondTests`, `EngineFeaturesTests`, the Mars classes (`MarsCheatTests`,
`MarsCoreSettingsTests`, `MarsDebugTests`, `MarsNativeStateTests`, `MarsRtCoreAbiTests`, `MarsRtDebugTests`,
`MarsRtFrontendTests`, `MarsRtTests`, `MarsRtEngineTests`), the Mercury classes (`MercuryRtCoreAbiTests`,
`MercuryRtCoreTests`, `MercuryRtDebugTests`, `MercuryDefectTests`, `MercuryModelTests`, `MercuryCartridgeTests`,
`MercuryModelSettingTests`), the Moon classes (`MoonCheatTests`, `MoonDebugEngineTests`, `MoonRtCoreAbiTests`,
`MoonRtFrontendTests`, `MoonRtPalTests`, `MoonRtEngineTests`), `NativeHostTests`, `NephriteDebugTests`,
`RustShimDisposeTests`, `RustStateHeaderTests`, `RewindToMomentTests`, `VenusRtCoreAbiTests`, `VenusRtDebugTests`,
`VenusRtDebugViewTests`, `DebugCodeParityTests`, `FrontendDefectTests`, `HotaruOnTheApiTests`, `ManagedCrystalTests`,
`ManagedCrystal11Tests`, `SessionServiceParityTests`, `StateRefusalTests`, `MistressStateRefusalTests`,
`GenesisCheatFormatsTests`, `GoldenCorpusTests`, `SnesSystemPackTests`, `DisplayAspectTests` and
`ConsoleBindingsTests`: **784 tests, 783 passed, 1 failed**, in 12 min 53 s. The failure was a test's own words: the
refused NES game's message carries the core's detail ("mapper 200 is not implemented") rather than `status_text`'s
sentence; the assertion was corrected, and `SharedHostTests`, `CoreAdapterTests` and `NativeHostTests` run again:
**32 of 32 passed**. No golden file changed: the debugger's corpus passed unchanged, the shims' cores still being the
shims, and the generic target's new members answering as before where no pack fills them.

**Again on `WiseMan` at `bd47a7c0`**, after MercuryRT's and MoonRT's second phases were merged (§17.9, §21) and this
part was rebased onto them: the same classes with the moved and new ones (`GameRefusalTests`, `MoonGoldenTests`,
`MoonAccuracyCoinTests`, `MoonPpuDefectTests`, `MoonRtRawPictureTests`, `MoonSpriteZeroTests`, `MoonTestRomTests`,
`NesRunToFrameTests`, `PadButtonTests`, `AudioBufferLimitTests`, `MercuryRtFallbackTests`, `MercuryStateLayoutTests`):
**1,066 tests, 1,065 passed, 1 failed**, in 14 min 43 s. The failure was Pharaoh's NES test-ROM runner, which took
"no engine" as its word for a board the engine does not build; that board is now refused as every v1 engine refuses a
game, a fault in the core's words, and the runner reads a fault whose words say the game was refused as the same skip.
Run again with its neighbours (`MoonTestRomTests`, `GameRefusalTests`, `SharedHostTests`, `MoonGoldenTests`,
`MoonRtCoreAbiTests`): **140 of 140 passed**. MercuryRT's shim had already turned its `-10` into the host's
`CoreRefusedException` (`GameRefusalTests`), which this part's removal of the load-time mapping keeps.

**Sizes, against §7's B4 row** [M: `git diff --numstat`]: about 215 lines of C# added and 100 removed in the host and
the shims, 22 in the test core, 199 in `SharedHostTests`; §7 estimated 300 for "the generic C# adapter" and §17.7
about 150 for these host pieces.

### 22.4 Core ABI 1.13, as built

One minor of six items, as §20 decided; the normative text is `EmuSen_CoreAPI.md` §6.33 to §6.38 and its changelog's
row, the summary `EmuSen_CoreAPI_Extensions.md` §15.

- **Units, under that name.** §17.3's `units` and §19.4's `devices` were one proposal; `units` was taken because 1.9
  already gave "device" to the Vulkan device a host lends (`emusen_device_lend`, `DEVICE_PICTURE`) and the extensions
  page's §10.1 to a controller kind. **Ids from 256**, §19.4's choice, kept: processors are below 24 by the coverage
  flags, so `debug_registers` reads either with no second argument. Machine info's `units` is written only when a
  core lists any, through `Core::units`, beside `processors`, so no existing core's machine info changes.
- **`TELEMETRY`** (bit 23): `emusen_core_telemetry` and machine info's `telemetry`, `Core::counters` and
  `Core::telemetry(&self, out: &mut [i64]) -> usize`, which writes into the host's buffer and allocates nothing.
- **`frame_info.lines`**: `emusen_frame_info` grown to 64 bytes. `emusen-native`'s `frame_info` refused any struct
  shorter than its own until now, which would have refused every 1.12 host's 56 bytes once the struct grew; it now
  takes a struct of 56 or more and writes what both sides know, §4.2's rule, shown by a crate test with a struct of 56.
- **`PRESENCE`** (bit 24): `emusen_core_set_present`, the narrow form; the crate refuses a `present` other than 0 or 1
  before the core sees it.
- **`EMUSEN_FRAME_CHEATS_HELD`** (frame flag 2), no bit.
- **`DEBUG_TRANSLATE`** (bit 25, only with `DEBUG`, which `core_exports!` checks at compile time):
  `emusen_core_debug_translate`.

**The host.** `CoreInterface` at 1.13 with the three bits, the three exports and `FrameInfo.Lines`;
`CoreMachineInfo.Units` and `.Telemetry`; `CoreMachine.Telemetry`, `.SetPresent` and `.Translate`;
`CoreEngine.Telemetry(Span<long>)`, `SetControllerConnected` (through the input latch, a change sent as the next frame
starts), `DisplayLines` (the last frame's `lines`) and the frame end, which reads `frame_info` once while it holds
cheats of its own and skips them after a held frame; `CoreDebugTarget`'s `VideoRegisters` from `video` units,
`ApuRegisters` from `audio`, `timer` and `cartridge` units (a board headed by `Board:<label>`), `Counters` (a new
`ICoreTelemetry` member, defaulted to none) and `ResolvePhysical` translating first. Every provider is filled from a
ring at each refresh and allocates nothing. Not built: the session service's telemetry kind and `coretop`'s list,
which are the frontend API's next minor (§18.4.1 put them there).

**The kit.** Each check is made only where its item is claimed or listed, so that a core claiming none is reported
as at 1.12: C2 holds the three new exports to their bits (as it does every export); C5 calls each claimed export;
every frame's checks hold `lines` to 0 or a divisor of the height; C14 checks the units, the counters, presence's
refusals and, on a core claiming `CHEAT_POKES` with a `cheats` space, that a held frame applies none of the core's
pokes (one machine runs 32 frames with a poke, and only if a frame is held do two machines, with the poke and
without, run side by side); C15 checks the translation. The test core claims all six; six seeded faults, 1 << 28 to
1 << 33 (the test cores' faults are now 64 bits wide), each fail their case alone.

### 22.5 Tested

Measured on 2026-10-18 [M]:

| What | Result |
|---|---|
| `emusen-native`, `cargo test --profile test-checked` | 79 passed (78 before, the new one the minor's items on the test core) |
| `emusen-core-abi-check` tests; `--check --against` the 1.12 baseline | 6 passed; 509 facts agree on 4 triples, with the Rust and with the baseline (485 at 1.12; `struct emusen_frame_info` keeps its 1.0 line of 56 bytes, the two fields their own lines) |
| The header as C99, C11 and C++17, `-Wall -Wextra -pedantic -Werror` | compiles |
| `emusen-core-conform`, `cargo test --profile test-checked` | 8 and 26 passed (two new: each 1.13 fault alone, and the items reported only where claimed) |
| `export_check.py` on every core's library and the test core | every core's exports as the baseline requires, ABI 1.13; the test core 65 symbols |
| **The kit's corpus on every core**, at 600 frames, the old kit on the libraries of `WiseMan` before the minor against the new kit on the same tree with it: MoonRT (4 NES images, `--self-delimiting`), MercuryRT (4), VenusRT (2), MarsRT (Super Mario 64, Ocarina of Time), Nephrite (`genesis-sample.txt`, 64), Diamond (`psx-sample.txt`, 9) | **91 reports (85 images and the six summaries), every case's verdict and evidence the same** once the minor's number (C1's "core ABI 1.13", the heading's `abi`) and C13's count of rounds are set aside; every core's info is the same but its `abi`, and its settings schema the same to the byte. MoonRT and MarsRT were run again after their second phases were merged (their own registers claimed), with the same result (5 and 3 reports) |
| WiseMan, the minor's blast radius, first run (49 classes: `CoreAbi113Tests`, `CoreAbiTests`, `CoreAdapterTests`, `CoreDiscoveryTests`, `CoreRegistrationEquivalenceTests`, `SharedHostTests`, `CoreEngineThreadTests`, the VenusRT, Diamond, Nephrite, MarsRT, MoonRT and MercuryRT adapter and debug classes, `MultiplayerTests`, `ControllerPortTests`, `PadButtonTests`, `GameRefusalTests`, the `ManagedCrystal` classes, `FrontendDefectTests`, `HotaruOnTheApiTests`, `SessionServiceParityTests`, `StateRefusalTests`, `GoldenCorpusTests`, `DisplayAspectTests`, the cheat, breakpoint, battery and rewind classes) | 687 tests, 684 passed, 3 failed: `CoreAbiTests`' count of 62 exports and a unit's expected first register (the tests' own numbers, corrected), and **`The_frame_call_allocates_nothing`**, a real regression: the target's ring walked machine info's units with `foreach` over an `IReadOnlyList`, whose enumerator is an allocation a frame once a core lists any; both loops were made index loops |
| The same on `WiseMan` at `e1350291` (MarsRT's second phase merged), with `MarsCoreSettingsTests`, `MarsDebugTests`, `MarsRtDebugTests`, `MarsRtEngineTests`, `MercuryRtDebugTests`, `MoonRtFrontendTests` and `NativeHostTests` added (56 classes) | **787 tests, 787 passed**, 13 min 49 s |

### 22.6 Sizes, against §7

[M: `git diff --numstat`] In `emusen-native`, the header and the guard, about 290 lines added (the trait's five
methods and three groups, the descriptors' writer, the three exports, the header's text) and 300 more in the test
core, the crate's tests and the schema; in the kit about 270 (its API, the checks, the tests); in the host about 160
of C#, and 139 in `CoreAbi113Tests`. §17.3 estimated 40 for the units in the crate and 60 in the host, §19.4 60 in the
crate and header, 120 in the host and 60 in the kit, and §18.7 250 in `emusen-native` and the kit for its five items;
together about 590 against about 720 built, the difference mostly the test core's six faults and the kit's tests of
them. The whole shared part, both halves, is about 1,000 lines of Rust and C and 375 of C# beside about 340 of tests,
inside §7's B4 row (1,500 to 3,000 of Rust across the cores, 300 of generic C#), of which the cores' own fills are
still to come.

### 22.7 For the core branches, and what was not done

- **No core claims an item.** MarsRT's counters, presence, cheat gate, lines and translation, MoonRT's and
  MercuryRT's units, and each console's pack views of §31.7 are their branches' work; the host reads each as soon as a
  core claims it.
- **The frame-time exception mapping** of `PortEngine` and MoonRT's `Reset` stays until those shims go (§22.2).
- **The session service and `coretop`** do not yet show the counters (§22.4).
- **No platform interface number was taken**: neither half changed the platform library's C interface.
- **The shims hide `CoreEngine.Labels`** with labels of their own (MercuryRT's, MoonRT's and MarsRT's registries,
  compiler warning CS0108); left as it is, since each goes with its shim.

---

## 23. B4's last phase for MoonRT, 2026-10-18: the units, the pack's views and the switch

*Built on branch `b4-moon-venus` from `WiseMan` at `e75ad6bf`, core ABI 1.13 and §22's host pieces merged: §19.8's steps
4 to 6, and §21's two items left for after the shared part (the fixed map wired, `MoonCheatTests` moved). MoonRT now runs
on the generic engine and target alone. `PortEngine` and the pre-stable host stay, for MercuryRT's shim, and go in one
commit after its switch.*

### 23.1 Step 4: MoonRT's units

Nine units on 1.13's `units` (`EmuSen_CoreAPI.md` §6.33), ids from 256, each register read by `debug_registers` from the
part that holds it (`src/ffi/units.rs`), one function giving a unit's names and values so machine info's list and the
export cannot disagree:

| Id | Unit | Kind | Registers |
|---|---|---|---|
| 256 | PPU | `video` | the shim target's twelve: PPUCTRL, PPUMASK, PPUSTATUS, OAMADDR, v, t, x, w, the scanline and three status flags |
| 257 | APU | `audio` | `$4000` to `$4013`, FrameCounter, FrameIRQ |
| 258 to 262 | Pulse 1, Pulse 2, Triangle, Noise, DMC | `channel` | the internal counters the channels view reads: duty, timer period, sweep, length counter, envelope (constant, volume, decay), the DMC's bytes remaining, output, rate, address, length and loop |
| 263 | Board | `cartridge` | `Mapper`, then the board's own registers |
| 264 | Cartridge | `other` | Mirroring (the board's arrangement, in `NesMirroring`'s order), ChrRam, Battery |

**`MoonBoards` moved into the boards.** Each board's registers for `regs` are now `Mapper::debug_registers`
(`memory/mappers.rs`), with the names and widths C#'s boards gave them, MMC3's `IrqsFired` among them (Q6), and the
nametable arrangement is `Mapper::mirroring`, the one the PPU draws by, where the shim restated sixteen rules over
state paths. The channels are of kind `channel` and the cartridge's facts of kind `other` so that `regs`, which shows
the `video`, `audio`, `timer` and `cartridge` units (§22.4), prints what the shim printed: the one added row is the
board's `Mapper`, which the pack's label and summary need and a unit's label cannot be given otherwise
(`UnitLabel` sees one unit).

### 23.2 Step 4: the NES pack's views

`NesDebugViews` fills §31.7's members over the units and the spaces: `MaxSprites` 64; `Sprites` from `OAM` and the
PPU's PPUCTRL (8x8 or 8x16); `AudioChannels` with `Moon_Native.md` §9.4's three display rules, which are the 2A03's and
so the pack's; `UnitLabel`, a board by its mapper number's name from `nes::boards`; `Summary` from the `PRGROM` and
`CHR` spaces' sizes and the units; and `ResolvePhysical`, the fixed map of §21.2, which `CoreDebugTarget.ResolvePhysical`
now reaches. `NativeDebugViews` answers a summary that reads units from the C# views and one that reads only the
processor's registers from the library as before; the new members have no Rust twins yet, as §22.2 allows, so no
platform interface number was taken. `nes::views::resolve_physical` stands as the map's Rust half for the twins.

**Shown against the shim's target before it was deleted** (commit `ac65711e`): on all 32 boards' programs
(`MoonPrograms.Boards`), the CPU, video and APU registers, the sprites, the five channels and the summary were the same
at frames 30, 60, 90 and 120, but for the board's `Mapper` row and the summary's head; 38 of 38 of
`MoonRtCoreAbiTests` passed. That comparison left with the shim.

### 23.3 Step 5: the switch

- **Deleted**: the shim's five files (`MoonRtCore`, `MoonRtDebugTarget`, `MoonBoards`, `MoonMachine`, `MoonNative`), 649
  lines; MoonRT's pre-stable exports (`native_exports!`, its `NativeCore` implementation, `moonrt_step`,
  `moonrt_rom_patch`), so the library defines 51 `emusen_core_` symbols, `moonrt_core_step`, `moonrt_core_rom_patch` and
  no `emusen_native_` symbol; CI counts `emusen_core_` for `moonrt`.
- **`CoreFactory`**: the `.nes` arm and `Bundle`'s and `Running`'s shim cases go; a `.nes` opens on the generic engine
  through discovery, and a build without the library says "This build has no NES core: MoonRT (Rust) is missing (…)".
- **`CoreCatalog`**: `MoonRtEngine` leaves `SoleEngines` for `SoleDiscoveredEngines`, beside VenusRT's.
- **`EMUSEN_MOON_NATIVE` retired** (§20); `EmuSen_Settings_Reference.md` and `EmuSen_CoreAPI.md` §26.5 carry dated notes,
  and `Moon_Native.md` §10 and `Moon_Debug.md` §8 record the retirement.
- **The tests**: `Fixtures/MoonEngines.cs` loses the shim branch; every class of §21.3's second table now holds a
  `CoreEngine`, the debugger's classes through the generic target (`MoonDebugEngineTests`' armed-against-plain runs read
  the bridge's coverage and call stack), the frontend classes through the factory, the library case holding that no
  pre-stable export is left, and the missing-library cases through discovery (`MoonRtEngineTests` needs no child process
  any more, discovery being read per call). `NativeHostTests` tests the pre-stable loader on MercuryRT's library alone.
  `MoonRtCoreAbiTests`' comparisons with the shim became the factory's one-engine case, the fixed map through the target,
  and the adapter against the bare v1 machine.

### 23.4 Step 6: what changed, and what was re-recorded

Re-recorded as §20 decided, each file's change read before it was kept:

| Record | What changed |
|---|---|
| The debugger's corpus: `regs`, `spaces`, `cpus`, `bp`, `mem`, `counters`, `addr`, `tile`, `tilemap` (`moonrt` cases) | "MoonRT" for "NES" in headings and messages; `cpus` "CPU" for "2A03" and `regs` among what it supports; `spaces` and the space lists gain `PPUREG` and `APUREG` |
| `regs`, `setreg` | `regs cpu` prints the thirteen registers; `regs` adds the board's `Mapper` row |
| `summary` | the generic head ("MoonRT (Rust) 0.1.0, nes, frame 2"); the scanline at the end of the PPU's second line; the false APU line gone (C4) |
| `mute` | channel 0, muted, now reads `muted=True`: **a defect of the shim's target**, whose frame record cleared its mutes and then read them back from itself (`MoonRtDebugTarget.cs:78–79` as it stood), so every channel read unmuted whatever was muted |
| `eval symbols` | the processor's registers, the pack's symbols and `stackdepth`; **the PPU's and the APU's register names are no longer symbols**, the generic target's expressions reading the main processor alone (§23.6) |
| `CoreRegistrationGolden.txt` | the NES's rows: `CoreEngine` and `CoreDebugTarget` for the shim's types, "MoonRT" as the identity's name, `registered[MoonRT (Rust)] = False`, the `available` line without MoonRT, the "off" world running MoonRT, and for a game without a battery a save placed in an earlier folder now copied into `Saves/NES`, as the SNES's already is (the core then ignores it, C1) |
| Eight corpus traces of `MoonGoldenTests` | `verdict Unsupported CoreRefusedException` for `NotSupportedException`, with the reason written into each; no other line of any trace moved |

**Retired as recommended** (§19.3.3 D16): a store the debugger makes on `CPUBUS` lands but is not logged as the
program's; `MoonDebugEngineTests.A_store_through_the_bus_space_lands_and_is_not_the_programs` holds it, where the
shim's test held the report.

### 23.5 The gates, measured on 2026-10-18

| Gate | Result |
|---|---|
| MoonRT's goldens, the corpus and the games opted in | 112 cases pass: every step chain, the corpus's 263 ROMs as recorded (the eight re-recorded verdicts the only lines moved), the seven games' 3,600 frames each as recorded and again from their recorded states; 105 corpus passes, as the C# baseline had |
| WiseMan, the blast radius (53 classes: every class of §21's two tables, `MoonRtCoreAbiTests`, `CoreRegistrationEquivalenceTests`, `GoldenCorpusTests`, `CoreDebugViewTests`, `SharedHostTests`, `CoreAdapterTests`, `GameRefusalTests`, `DebugCodeParityTests`, `CoreDiscoveryTests`, `CoreAbiTests`, `CoreAbi113Tests`, the frontend API's and Hotaru's classes, `SessionServiceParityTests`, `BreakpointConditionTests`, `DebugPlumbingTests`, `RewindToMomentTests`, `CheatCodecAdapterTests`, `MultiplayerTests`, `NesGameGenieCodecTests`) | 599 tests, **598 passed, 1 failed**: `CoreDebugViewTests`' MoonRT case held the shim's summary head; corrected to the generic head §20 decided, the class run again, **5 of 5** |
| WiseMan, the classes that reach an NES game through Mistress, Hotaru, the shell or the library (19 classes: `ConsoleContextTests`, `CoreCommandTests`, `MainWindowLogDirectoryTests`, `GameEngineAndPlayTimeTests`, `ManagedCrystalGroupTests`, `RuntimeCatalogTests`, `MenusFollowupTests`, `GameCheatListTests`, `FrameSchedulerTests`, `CatalogueTests`, the themed library's five, `LibraryScreenTests`, the scene classes, `CoreDebugViewTests`) | **229 of 229 pass** |
| MoonRT's crate, `cargo test --profile test-checked` | 27 pass (the units' case new) |
| The kit, 20 corpus images at 600 frames, `--self-delimiting` | the core's cases and 20 of 20 images pass, at core ABI 1.13, the units checked by C14 |
| `export_check.py` against the baseline | ABI 1.13, 51 `emusen_core_` symbols; no `emusen_native_` symbol, and of MoonRT's own only `moonrt_core_step` and `moonrt_core_rom_patch` |
| No per-core type | `grep -rn "EmuSen.Cores.Nintendo.MoonRT"` over shipped code: 0 |

`moonbench`'s hashes were not taken again, as in §21.4: the games' traces hold the same games' states under its input
at every frame.

### 23.6 Left, and for the tester

- **Register names in expressions.** The shim's target let `eval`, a condition or a logpoint name the PPU's and the
  APU's registers; the generic target's expressions read the main processor's registers alone, so `bp … if
  PPUCTRL==0x80` no longer parses on the NES. Reading the units' registers in the expression context is a host change
  for every core with units (MercuryRT's and MarsRT's too), and so a shared piece, not this branch's; *recommended*: add
  it to the host, the units' names after the processor's, a name both hold resolving to the processor's. *Decided
  2026-10-18 and built: §24.*
- **The new pack members' Rust twins** for the NES, with their parity cases, when the views' C# is to be retired (B7).
- **MercuryRT's switch**, after which `PortEngine`, `NativeRtCore` and the pre-stable host go in one commit (§22.7).

---

## 24. Debugger expressions name the units' registers, 2026-10-18

*Decided 2026-10-18 by the tester, as §23.6 recommended: the generic host's expressions (`eval`, breakpoint conditions,
logpoints) name the registers of a core's units (core ABI 1.13, `EmuSen_CoreAPI.md` §6.33) after the processor's own.
Built on branch `b4-unit-exprs` from `WiseMan` at `9eb357a9`. A shared host change: it serves every core that lists
units: MoonRT's nine, MarsRT's one (the VI, which reached `WiseMan` at `08812dde` and was merged into this branch
before it went back), and MercuryRT's once its switch lands.*

### 24.1 The regression, shown first

MoonRT's shim had no expression context of its own, so `DebugExpressions.For` gave it the fallback,
`DebugTargetExpressionContext`, which names every row of `CpuRegisters`, `VideoRegisters`, `ApuRegisters` and
`CoprocessorRegisters` bare; `PPUCTRL` and `FrameIRQ` were symbols. The generic target publishes a context built from
the main processor's registers, the pack's symbols and `stackdepth`, so after the switch (§23.4, `eval symbols`) no
unit register was nameable. `UnitExpressionTests` was written first and run on the unchanged tree, Release, measured on
2026-10-18 [M]: on a SyntheticRom NES image whose loop sets PPUCTRL to `$10`, the PPU's fine x to 3 and v to `$2108`,
and pulse 1's duty to 3, four of its five cases failed: `eval ppu.PPUCTRL` answered "Unknown symbol 'ppu.PPUCTRL'",
`bp add 801E if "ppu.PPUCTRL==0x11"` was refused at the add, and the context resolved nothing. The fifth, a core that
lists no units (Nephrite), passed before as after, which is its point. The bench's new MoonRT row, a condition naming
`ppu.PPUCTRL`, was refused at the add likewise.

### 24.2 The design, settled before building

- **Spelling.** A unit register is `<unit>.<register>`, the unit's name lowercased with every character but letters,
  digits and `_` dropped: `ppu.PPUCTRL`, `pulse1.Duty`, `board.Mapper`. The register alone is a symbol too where
  exactly one unit has a register of that name, so §23.6's `bp … if PPUCTRL==0x80` reads as it did under the shim.
  Case does not matter, as for every symbol. The expression language already reads `.` inside an identifier (the
  SNES pack's `flag.c`), so no change to `ExpressionEvaluator` is needed.
- **Order, and the clashes.** A scope's existing symbols are consulted first, unchanged: `frame`, the processor's
  registers, the pack's symbols and `stackdepth`, then labels; the units come after all of them. The rule this buys is
  that nothing that resolved before a core listed units changes meaning, whatever units a core later adds. So on the
  NES `x` and `v` stay the CPU's X and overflow flag while `ppu.x` and `ppu.v` are the PPU's, and a label `PPUCTRL`
  (common in NES disassemblies, at `$2000`) keeps `PPUCTRL` as its address while `ppu.PPUCTRL` reads the register.
  The shim put its video and APU rows before labels; that order was not kept, because a label is something the player
  defined and a unit register's bare name a convenience, and the qualified form is always there.
- **Two units, one name.** A bare name two units share is a symbol of neither, and is refused as unknown: on the NES
  `Duty` (both pulses) and `LengthCounter` (four channels). A condition silently reading the wrong channel would be
  worse than a refusal at the add, which points at `eval symbols`. The same rule covers two units whose spellings
  coincide; no core has such a pair.
- **What cannot be spelled.** A register name that is not an identifier of the language, such as the APU's `$4000`
  to `$4013` and the Game Boy's `$FF10` onwards as MercuryRT's shim names them, is not a symbol: `$4000` is a number to the tokenizer, and the shim
  listed those names in `eval symbols` without any expression being able to reach them. `regs` shows them. Dropping
  characters to make a spelling (`apu.4000`) was weighed and not taken, as a second rule to learn for one register file.
- **Every scope.** Units belong to no processor, so they follow every scope's own symbols: the unscoped context the
  main processor's registry reads, and each processor's (`eval <cpu>`, `bp <cpu> … if`), including processor 0 named
  explicitly, whose context stays its registers alone as before.
- **`eval symbols`** lists the qualified forms; the man page states the bare rule. Listing both would double the
  list on the NES for names that are the same registers.
- **Side effects.** None, by the ABI: `emusen_core_debug_registers` takes `const emusen_machine *` in the normative
  header (`emusen_core.h`), and `emusen-native`'s `Core::debug_registers` takes `&self`, which a unit's id (§6.33,
  "read through `debug_registers`") shares with a processor's. MoonRT's `units.rs` reads fields. The text of §6.33 does
  not say "no side effects" in words; §24.5 asks whether it should.
- **Cost.** A condition naming only processor registers must take the path it took. So the order above is kept by
  construction rather than by asking the scope first: a table built once for each machine info holds the unit
  spellings, with every name the scope itself answers besides labels (the processor's registers from machine info,
  the pack's `SymbolNames`, `stackdepth`, `frame`) left out of it. A name not in the table goes to the scope exactly as
  before, after one dictionary miss; a name in it is checked against the labels (a dictionary lookup, since labels
  change at any time) and then read. Asking the scope first was tried and measured: a name the scope does not hold
  walks the ring provider's `Current`, which builds a list, and asks the pack's `Symbol`, which on the NES is a JSON
  round trip to the platform library, about 12 KB allocated a lookup (3.6 MB over 300) [M]. The read is `CoreMachine.Registers(id, span)`
  into a buffer on the stack, at evaluation time; nothing is allocated. The target's context is now made once for
  each machine info rather than on every call of `Expressions`, so an evaluation no longer allocates a context either
  (the evaluator's own parse still allocates its tokens, as before). The design rests on one property of the packs:
  `ISystemDebugViews.Symbol` answers only names its `SymbolNames` lists, which `eval symbols` already relies on.

### 24.3 What was built

- **`UnitExpressionContext`** (`EmuSen/Cores/Native`), one job: a scope's context, then the units. Its table
  (`UnitSymbols`) maps each spelling to a unit's id, a register's index and the unit's register count, rebuilt when
  the machine's info is a new object, which a new machine and a `MACHINE_INFO` change both make.
- **`CoreDebugTarget`**: `Expressions` wraps the main context and caches it for each machine info; each `DebugCpu` it
  makes carries an `Expressions` of its own registers wrapped the same way, which `DebugExpressions.For(target, cpu)`
  already prefers. No surface changed: `IExpressionContext`, `IDebugTarget`, `DebugCpu`, Crystal and the session
  service are as they were; the session service's `eval` and `symbols` reach the new contexts through
  `DebugExpressions` as both shells do.
- **`man eval`** gained a UNITS section with the rules above.

### 24.4 Measured on 2026-10-18

**The tests** [M], Debug: `UnitExpressionTests`' five cases, which failed four of five before (§24.1) and pass after:
`eval ppu.PPUCTRL`, `eval PPUCTRL`, `eval ppu.ppuctrl` answering `$10` and `pulse1.Duty` 3; a condition on
`ppu.PPUCTRL==0x11` never halting while a logpoint records `ppu.v=0x2108  PPUCTRL=0x10`, and one on
`ppu.PPUCTRL==0x10 && pulse1.Duty==3` halting at `$801E`; the clashes (`x` 5 and `ppu.x` 3, `v` the CPU's flag and
`ppu.v` `$2108`, a label `PPUCTRL` at `$2000` answering `PPUCTRL` while `ppu.PPUCTRL` stays `$10`, `Duty` refused
by `eval` and at a breakpoint's add while `pulse1.Duty` and `pulse2.Duty` answer 3 and 0, `eval cpu ppu.PPUCTRL`);
300 lookups of unit registers allocating 0 bytes; and Nephrite, which lists no units, naming exactly the 22 symbols
of the corpus's recording on processor 0 and on each processor what its own context named before.

The blast radius, 20 classes (`UnitExpressionTests`, `BreakpointConditionTests`, `MoonDebugEngineTests`,
`NephriteDebugTests`, `VenusRtDebugTests`, `VenusRtDebugViewTests`, `DiamondDebugTests`, `CoreDebugViewTests`,
`SessionServiceParityTests`, `ExpressionEvaluatorTests`, `GoldenCorpusTests`, `SharedHostTests`, `CoreAbi113Tests`,
`CoreAdapterTests`, `MercuryRtDebugTests`, `MoonRtFrontendTests`, `DebugPlumbingTests`, `BreakAndCovCommandTests`,
`HotaruOnTheApiTests`, `CoprocessorDebugTests`): **369 tests, 367 passed, 2 failed**, both the corpus's recordings
this change moves. Re-recorded, each difference read:

| Record | What changed |
|---|---|
| `eval.golden`, case "on moonrt", `eval symbols` | 53 lines added, the units' qualified names (`apu.FrameCounter` to `triangle.TimerPeriod`); the APU's `$4000` to `$4013` absent, as unspellable; no existing line moved |
| `man.golden`, case "the page for eval", `man eval` | the page's new UNITS section, 25 lines, and the output's length; no other page's recording changed |

No other file of the corpus changed (a recording pass also stripped a trailing empty line from `disasm.golden`, which
was put back, as no case of it changed). `GoldenCorpusTests` with `UnitExpressionTests` again: **105 of 105 pass**.

**The cost**, Release, `BreakpointConditionCostBench` (`EMUSEN_BP_BENCH=1`), per hit over a breakpoint never reached,
on the tree before the change (its tests and the bench's new row only) and after:

| Row | Condition, before / after | Logpoint, before / after | `if 0`, before / after |
|---|---|---|---|
| MercuryRT on the generic engine (`mercuryrt-v1`), `HL==0x1234` | 1.08 / 1.13 µs | 1.02 / 1.06 µs | 0.44 / 0.36 µs |
| Nephrite, `D2==0` | 1.48 / 1.38 µs | 1.42 / 1.48 µs | 0.61 / 0.54 µs |
| MoonRT, `A==0x99` | 5.11 / 4.74 µs | 4.89 / 4.99 µs | 0.48 / 0.40 µs |
| MoonRT, `ppu.PPUCTRL==0x99` (new) | refused / 4.62 µs | refused / 4.82 µs | 0.47 µs after |

A processor-register condition costs what it did: every pair moves less than the bench moves on a path this change
does not reach, MercuryRT's shim, whose register condition went from 4.93 to 5.43 µs between the same two builds, and
the pairs move in both directions. A unit-register condition on MoonRT costs what a processor-register one does there.

### 24.5 Left, and for the tester

- **MoonRT's register condition is about 4.7 µs a hit, against about 1.1 on MercuryRT's generic row**, with units or
  without. It is not the units' read: `BreakpointExpressions`' `Live` calls `RefreshProviders` once an evaluation
  first names a symbol, and on MoonRT that refreshes the pack's view rings (palettes, sprites, channels), which copy
  the spaces they read, at every hit. Refreshing only the register providers there would bring it near MercuryRT's;
  that is `BreakpointExpressions`' rule (`EmuSen_DianaSh.md` §13.12), so it is proposed, not built. MercuryRT will
  meet the same cost once it lists units; MarsRT's, its VI listed and the N64 pack's views
  refreshed, cost 1.4 to 1.9 µs a hit (§24.6).
- **§6.33's text** says units are read through `debug_registers` but not, in words, that the read changes nothing;
  the header's `const` and the trait's `&self` say it. *Recommended*: a clarification of one sentence there, no minor.
- **MercuryRT's switch** needs nothing more: once its target is `CoreDebugTarget`, its units are in every scope by
  these rules, and its `eval symbols` recording will gain the qualified names, as MarsRT's did (§24.6).

### 24.6 After MarsRT's switch, measured on 2026-10-18

`WiseMan` at `08812dde` (MarsRT on the generic engine and target, its one unit the VI: id 256, `video`, fourteen
registers by libultra's names) was merged into this branch with no conflict; `CoreDebugTarget`'s view-space reads and
its ring's re-learning on a machine-info change sit beside this section's changes untouched. MarsRT's VI reads its
registers through `&self` (`vi.read32`), so the read changes nothing, as §24.2 requires. Nothing of the design moved:
the VI's names clash with neither processor's (the CPU's and the RSP's registers are MIPS names, `PC`, `HI`, `LO`,
`Cycles` and `Halted`).

**Re-recorded**, each line read: `eval.golden`, case "on marsrt", `eval symbols`: 14 lines added, `vi.BURST`,
`vi.CONTROL`, `vi.H_START`, `vi.H_SYNC`, `vi.LEAP`, `vi.ORIGIN`, `vi.V_BURST`, `vi.V_CURRENT`, `vi.V_INTR`,
`vi.V_START`, `vi.V_SYNC`, `vi.WIDTH`, `vi.X_SCALE`, `vi.Y_SCALE`, and the output's length (206 to 379); no other line
of any file changed (`disasm.golden`'s trailing empty line, stripped again by the recording pass, was put back).

**Tested**, Debug: §24.4's 20 classes with `MarsRtDebugTests` and `MarsDebugTests`: **402 tests, 402 passed**.

**The bench**, Release, with a MarsRT row on `vi.ORIGIN==1` added, run twice (µs a hit over a breakpoint never
reached; the machine was shared with other test runs, and the first row of the first run read an `if 0` floor of 1.90
against 0.39 to 0.42 elsewhere, so its numbers are set aside):

| Row | Condition, run 1 / run 2 | `if 0`, run 1 / run 2 |
|---|---|---|
| MarsRT, `t0==0` | 1.68 / 1.90 | 0.39 / 0.42 |
| MarsRT, `vi.ORIGIN==1` (new) | set aside / 1.42 | set aside / 0.42 |
| MoonRT, `ppu.PPUCTRL==0x99` | 4.68 / 4.59 | 0.36 / 0.48 |
| MoonRT, `A==0x99` | 4.72 / 5.00 | 0.38 / 0.41 |
| MercuryRT on the generic engine, `HL==0x1234` | 0.96 / 1.13 | 0.35 / 0.38 |
| Nephrite, `D2==0` | 1.36 / 1.86 | 0.52 / 0.60 |

A unit-register condition costs what a processor-register one does on the same core. The spread between the two runs
of one build (Nephrite 1.36 and 1.86) is wider than any difference §24.4 found between the builds before and after.
