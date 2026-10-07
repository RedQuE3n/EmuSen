# EmuSen — the core API: a stable ABI for cores, and DianaOS as the runtime above it

*This revision: the second, 2026-10-03: §15's eighteen questions decided, each as this page recommended (§0.2, §15),
and §13.2's items 1 and 2 built: the header, the `core` module with `core_exports!`, and the guard (§18, which also lists
the points of §4–§6 that building them made precise); then items 3 and 4, the host adapter and discovery (§19); item 5, the SNES's system pack (§20); and item 6, the
conformance kit's core suite (§21); and MoonRT's and MercuryRT's exports onto v1 beside their pre-stable ones (§22,
§23). Since then, 2026-10-04: the registration-equivalence test, D1's oracle (§25), and MoonRT's and MercuryRT's
shims as subclasses of the adapter (§26). Previous revision: the first, 2026-10-01. A normative specification, not an implementation: nothing in it is built except
three prototype checks recorded in §5.1 to §5.3. It supersedes parts of `EmuSen_NativeCores.md`, listed in §0.3 and
marked in place there. Every claim about the code is cited to a file, read on `WiseMan` at `f69c94a4`. Claims marked
**measured** were measured on 2026-10-01; claims marked **argued** are reasoning that a later step must prove, and
§16's predictions say how. libretro is compared from its published design and from general knowledge; nothing of its
header or its text is reproduced.*

Companion docs: `EmuSen_NativeCores.md` (the interface this one stabilises, and its record of what was built),
`EmuSen_RustState.md` (the state codec and the status band −1 to −8), `EmuSen_Multicore.md` (`ICore`, the capability
interfaces, `CoreFactory`), `EmuSen_Debugging_Tools_Reference_v5.md` §3 (DianaOS, `IDebugTarget` and the registries),
`EmuSen_Cauldron.md` §3.1 (telemetry versus the debugger), `EmuSen_Stack.md` §6 (the later stack), `EmuSen_Libretro.md`
(the libretro specification), and `VenusRT_Plan.md` (the core that is to be the first built on this).

---

## 0. The aim, and the summary

### 0.1 What the framework is for

**EmuSen is meant to be a next-generation emulation platform that is accurate first, but offers settings for
performance. It folds in the features of ES-DE and RetroArch, and it is user friendly. It is meant to be Unix-like
throughout, and that is where DianaOS comes in: DianaOS is meant to be front and centre, the runtime every core runs
under and every frontend talks to.** (Stated 2026-10-01. No earlier page records the last two sentences in these
terms; §1.3 quotes what the pages did say.)

Each part of the aim becomes a test this specification is judged by:

| Aim | What it requires of the design | Where |
|---|---|---|
| Accurate first | The accurate configuration is the default, and a setting that trades accuracy away says so and what it costs | §2.1, §6.13, §12.4 |
| Settings for performance | Every core can describe its speed settings in one vocabulary, so every frontend presents them the same way | §6.13 |
| RetroArch's and ES-DE's features | Each feature has one owner among the layers, and the core ABI does not close the door on the ones not yet adopted | §11 |
| User friendly | A new core appears in every frontend, with its name, consoles, bindings screen and settings in readable words, with no per-core code | §2.4, §6.3, §6.8, §6.13 |
| Unix-like | Organisation, inspection and scripting follow Unix where that earns its place, and the hot path does not | §2.2, §8.5 |
| DianaOS front and centre | Frontends are clients of DianaOS, not of cores; the shell is one more client | §3, §8, §9 |

### 0.2 Decided 2026-10-01

1. **EmuSen has a stable, versioned core ABI.** A core built against version 1 keeps working with later frontends, a
   frontend keeps working with older cores, and the contract is documented well enough for an outside author to write
   a core against it. This replaces exact version matching (`EmuSen_NativeCores.md` §9 Q3).
2. **DianaOS is the framework's runtime.** Cores sit under the core ABI; DianaOS sits above them and owns discovery,
   sessions, the debugger, cheats, states, settings and input routing; frontends, scripts and the shell are its
   clients.
3. **The platform is Unix-like where that fits**, and the specification says where it does not.
4. **DianaOS stays C# for this version** (`EmuSen_Stack.md` §6 defers its Rust port), and its frontend API is shaped so
   that it can later be offered as a C interface without changing shape (§9).

**Decided 2026-10-03:** the eighteen questions of §15, each as this page recommended and with nothing changed. Among
them: the stable set is `emusen_core_*` beside the pre-stable `emusen_native_*` (Q1); descriptors are JSON (Q2); the
"bus" is §8.3's call surface, event stream and two hand-offs (Q5); the accurate-default rule covers `accuracy` and
`enhancement` settings, not `latency` (Q12); the runtime goes in `DianaOS/Sys/` (Q13); the SNES's new cheat codecs go
in its system pack, not in VenusRT's shim (Q14); the axis value is a normalised `double` (Q15); and MarsRT moves
straight to v1 at its step 5 (Q16).

### 0.3 The design in one paragraph

A core is a shared library that exports one C interface, `emusen_core_*`, described normatively by one header,
`emusen_core.h` (§5). Its version is a major and a minor: the major is 1 and is expected never to change; within it
everything evolves by addition, discovered by name and capability bit, with every struct carrying its size and every
enumeration open (§4). A core describes itself in data: who it is, which consoles it runs, its controllers, its
settings with their accuracy cost, its memory, its processors and its files (§6.3, §6.4, §6.13). A frontend never
loads a core. DianaOS finds cores in a cores directory, checks them before and after loading (§7), and offers sessions
to its clients through a frontend API that is a call surface over handles with a pull-model event stream, the same
rules as the core ABI one layer up (§8, §9). A conformance kit makes "EmuSen v1 compliant" a test result (§12). The
existing interface's calls carry over almost whole (§6): what changes is that the per-console C# data the host holds
today moves into the core's own descriptors, so that VenusRT, the next core, needs no per-core C# at all (§13.2).

**What this page supersedes in `EmuSen_NativeCores.md`,** each marked there in place:

- §2's non-goal "A stable ABI for third parties";
- §3.2's version scheme, `(COMMON << 16) | CORE` matched exactly, and the `emusen_native_` prefix for the stable set;
- §3.4's create signature, §3.6's `FrameInfo` layout, §3.7's drain signature and §3.8's axis units;
- §3.13's last paragraph and §9 Q4, which kept settings lists in C#;
- §5's registration records, for native engines;
- §9 Q3, exact matching;
- §10's rejection of "messages across the boundary", for descriptors read once (§6.3).

---

## 1. Where things stand (read 2026-10-01)

### 1.1 The pre-stable common interface, as built

`emusen-native` (`EmuSen/Cores/Shared/emusen-native/`) holds the `NativeCore` trait and `native_exports!`
(`src/abi.rs`), the debugger's hooks as data (`src/debug.rs`), and the state codec. The C# host in
`EmuSen/Cores/Native/` holds `NativeCoreLibrary` (the loader), `NativeInterface` (the export table), `NativeMachine`
(the handle), `NativeRtCore<TMachine>` (the `ICore` base), `NativeDebugBridge`, and the legacy path MarsRT still uses
(`LegacyNativeMachine.cs`).

| Core | Version | Capabilities | `emusen_native_*` exports |
|---|---|---|---|
| MoonRT | `0x0001_0004` (`ffi/mod.rs:13`) | `RESET`, `MUTES`, `ROM_PATCHES`, `DEBUG`, `DEBUG_STACK` (`:34`) | 38: the 24 required, 3 optional, 11 debug |
| MercuryRT | `0x0001_0005` (`ffi/mod.rs:14`) | `MUTES`, `ROM_PATCHES`, `DEBUG`, `DEBUG_STACK` (`:38`) | 37 |
| VenusRT | `0x0001_0001` (`src/ffi.rs:9`) | none (`:16`) | 24 |
| MarsRT | its own ABI, interface 10 | — | none on the common set; step 5 is deferred with its PGO retrain until the port is finished |

### 1.2 What the code would freeze, and its gaps

Read from the code, not the design:

1. **Exact matching on both halves.** `NativeCoreLibrary.Load` refuses any library whose version is not the one the
   host was built with (`NativeCoreLibrary.cs:64`). Every change to a core's own number, which also guards its status
   band and settings keys (`abi.rs:132`), forces a rebuilt host.
2. **The snapshot kind is refused.** `state_size`, `state_save` and `state_layout` answer any kind but 0 with
   `NOT_SUPPORTED` (`abi.rs:673`, `:684`, `:713`). §3.9's kind 1 exists only in MarsRT's legacy ABI.
3. **Five optional groups are not generated:** `present`, `phases`, `audio_peek`, `set_axis` and `set_settings`
   (`abi.rs:4`). VenusRT's plan claims `SETTINGS` and `PHASES` (`VenusRT_Plan.md` §8).
4. **Registration is per core.** `CoreFactory.Create` is a switch on the file extension with one branch per engine
   (`CoreFactory.cs:41`); `EngineNotice`, `Running` and `Bundle` name every engine type; `.sfc` always means C# Venus.
   §5's registration records were never built (`EmuSen_NativeCores.md` §12.2).
5. **The console's facts are C# data in each shim:** button bits, space names and flags, the status table and its
   words, battery files, state pre-checks, the patch range, settings (`EmuSen_NativeCores.md` §4.5). A core cannot
   describe itself, so an outside core cannot be presented at all.
6. **No struct carries its size.** `FrameInfo` and `NativeFile` (`abi.rs:75`, `:91`) cannot grow without breaking
   every host.
7. **The drain does not report its rate.** `audio_drain(handle, out, len, max_frames)` (`abi.rs:640`); §9 Q10 decided
   that it should and it was not built.
8. **A signature disagrees across the boundary.** Rust's `emusen_native_set_crash_log` returns `i32` (`abi.rs:518`);
   the host calls it as `delegate* unmanaged<nint, void>` (`NativeCoreLibrary.cs:81`). On x86-64 and AArch64 a
   discarded return register is harmless, which is why nothing failed; nothing checks the two sides against each
   other, which is why nothing noticed. §5.3's check finds it (**measured**).
9. **The host frees from the finaliser thread** (`NativeMachine.cs:231`), so "one thread per handle" already admits
   a second thread, at the end of a handle's life.
10. **The observed run reports only processor 0's program counter** (`abi.rs:351`); a halt on the SA-1 or the SPC700
    cannot say where it stopped (`VenusRT_Plan.md` §8).
11. **Libraries load only from beside the assemblies** (`NativeCoreLibrary.cs:58`), and only by a crate name the C#
    code already knows. There is no discovery.
12. **`Region` exists in the trait and is never exported** (`abi.rs:145`), nor is the frame rate; `ICore.FrameRateHz`
    is each shim's C#.

### 1.3 DianaOS today, and what the pages say it is for

**The code.** `EmuSen.DianaOS` is 131 C# files in a Unix hierarchy (`EmuSen.DianaOS/DianaOS/`):

- `Bin/`: the interpreter (`DianaOSInterpreter.cs`, a bash-alike with pipes, redirection, `$(...)` and control flow),
  its scheduler, 36 Unix commands (`ls`, `cat`, `grep`, `sed`, `awk`, `xxd`, `ps`, `kill`, `tmux`, `su`, `useradd`,
  `passwd`, `shutdown` and the rest) and 48 EmuSen commands (`mem`, `dump`, `diff`, `bp`, `watch`, `step`, `cheat`,
  `state`, `resume`, `callers`, `readers`, `channels`, `cov`, `disasm` and the rest), with `ManPages.cs`;
- `Lib/`: `IDebugTarget` and its value types, `ICheatCodeCodec` and `DelegateCheatCodec`, `DelegateDebugMemorySpace`,
  the expression evaluator, `HostAction`, `DebugTools`;
- `Var/`: the registries (`Watch`, `Breakpoint`, `CallStack`, `Coverage`, `AccessCounter`, `Freeze`, `Label`,
  `FrameLog`, `DmaLog`, `RegisterFlow`), the cheat engine (`CheatRegistry`, `CheatDatabase` with its installer, pruner
  and importer, `ChtFile`, `CheatWrite`), `DianaOSSessionManager` (the shell's `tmux` sessions), `FrameRecorder`;
- `Etc/`: logging, the sandbox, the user registry, the man pages; `Dev/`: the console line reader; `Usr/Home`: the
  manual.

So DianaOS already owns the debugger's vocabulary and registries and the whole cheat engine except the per-console
codecs. What it does not own is anything that runs a core: `ICore`, `CoreFactory`, `CoreCatalog`, `BatterySave` and
the native host are in the `EmuSen` assembly (`EmuSen/Cores/`), `EmulatorSession`, `RewindBuffer`, `SpeedController`
and `FirmwareLibrary` are in `EmuSen/Common/`, and the frame loops are each frontend's own.

**The reference graph** (the `.csproj` files): Galaxia and Cauldron reference nothing; DianaOS references Cauldron and
Galaxia; `EmuSen` references DianaOS, Serenity, Cauldron and Galaxia; Mistress, Hotaru and Pharaoh reference `EmuSen`.
DianaOS is therefore *below* the assembly that holds the cores, which is why it cannot construct one: "The same
applies to `EmuSen.DianaOS`, which is referenced *by* `EmuSen` rather than the other way round: the shell takes an
`ICatalogue` it is handed and cannot construct one" (`EmuSen_Stack.md` §2). 68 files under `EmuSen/Cores/` use
DianaOS types, 62 of them inside the C# cores (counted with `grep -l`), because the C# cores feed the registries
directly.

**What the pages say DianaOS is for.** No page found says, in so many words, that DianaOS is the bus and controller of
the cores. What they say is narrower, and points the same way:

- "The debug toolchain (`DianaOS/IDebugTarget.cs` and everything built on it) was deliberately designed core-agnostic
  from day one" (`EmuSen_Project_Overview_v2.md`, the multi-core paragraph).
- "a deliberately separate, core-agnostic layer sitting *beside* the hardware simulation, not inside it … so a future
  standalone "Diana" build has a clean, compiler-enforced line to register only the former" (the same page, on the
  debug toolchain).
- "built specifically so it isn't SNES-only, and so the same code can eventually back a GUI debug window, not just
  console printouts. **Standing policy going forward: if something built for one investigation seems logical and
  reusable, it goes in here**" (`EmuSen_Debugging_Tools_Reference_v5.md` §3).
- The project file's own description: "The core-agnostic debug/scripting shell" (`EmuSen.DianaOS.csproj`).
- `man hier`: "/ your home, and the whole of what this shell can see", and "Every directory above is located by
  EmuSen.Galaxia, not by this shell - the sandbox forwards to it, so a core never asks a debugger where a save goes"
  (`ManPages.cs:1379`).
- `man su`: accounts are "Unix flavor, not real access control … That enforcement (e.g. a restricted account that
  can't run mutating commands) is real future work"; its example is `su parent`, and `useradd`'s is `useradd kid`.

The pages therefore record a core-agnostic debugger and shell meant to grow, a Unix hierarchy that is the player's
home, and accounts meant for a household. They do not record the runtime role. §8.1 states it, and §15 records the
decisions on the questions the code left open.

### 1.4 How each client reaches a core today

- **Mistress** creates an `EmulatorSession` (`EmuSen/Common/EmulatorSession.cs`), which calls `CoreFactory`, and runs
  its own emulation thread and `FramePacer` (`EmuSen.Mistress/FramePacer.cs`). Rewind is decided per engine by
  `EngineFeatures` (`MainWindow.Rewind.cs:21`).
- **Hotaru** runs its own loop in `GameWindow` with the shell's always-live reader thread
  (`EmuSen_Frontend_Driver.md`).
- **Pharaoh** calls `CoreFactory.Create` and `Bundle` itself (`Program.cs:203`–`:235`) and runs `FrameRunner`. Its
  `--commands` scripts mix frames, input and debug verbs, and an unknown verb "falls through to the real
  DianaOSInterpreter" (`CommandsScriptRunner.cs:19`). That is a small precedent for the layering of §3: a client
  whose debugger is DianaOS.
- **The shell** is given an `IDebugTarget` by whichever host built it (`DianaOSInterpreter.CreateDefault`), and asks
  its host to load a core by returning `HostAction.LoadCore` (`Lib/HostAction.cs`), which only the host can perform.

Each of the three programs therefore has its own loop, its own way of choosing a core and its own copy of the
frame-end order, and the shell can do only what its host chose to wire.

---

## 2. Principles

### 2.1 Accuracy first, speed by choice

- **The default configuration of every core is its most accurate one.** A setting that trades accuracy for speed
  defaults to the accurate value.
- **A trade-off is declared, not inferred.** The settings schema (§6.13) gives every setting an `effect`: `none`
  (plain configuration), `exact` (speed with no change to the output), `accuracy` (speed at an accuracy cost),
  `latency` (speed or smoothness at a cost in delay) or `enhancement` (a deliberate departure from the hardware's
  output). `accuracy`, `latency` and `enhancement` carry a `cost` sentence in plain words.
- **The kit checks the claims it can check** (§12.4): an `accuracy` setting's default is its declared accurate value,
  an `enhancement`'s is the hardware's, an `exact` setting changes nothing at any value, and a machine made with no
  settings equals one made with every default.

The vocabulary already exists informally. MarsCore's hints say "Exact; faster on any machine with t[hreads]" and
"Exact at any count" (`Mars - N64/MarsCore.cs`, `ThreadedRdp`, `RdpWorkers`), and `VenusRT_Plan.md` §9 Q3 decided that
VenusRT's first speed lever "becomes a setting before anything in the PPU or CPU is loosened". The schema makes the
words a field.

### 2.2 Unix-like where it fits

Unix-likeness governs the runtime's organisation and its inspection and scripting surface. It does not govern how a
frame, its sound or its input move, unless it can be shown to cost nothing there, and it cannot (§8.3). Each mapping
is tested in §8.5 and adopted or declined with its reason. The test of the principle as a whole is the one in §8.6:
anything a player can do in Mistress, a shell user or a script can do through DianaOS, and the reverse.

### 2.3 What is kept from the interface of 2026-09-28

- **Pull, never push.** The host calls; the library returns. The library never calls the host. Every service a
  libretro core reaches through a callback is offered here as something the host passes in or pulls out (§6.20).
- **Handles.** Several machines from one library at once, as the tests, the side-by-side engine proofs and run-ahead
  need.
- **Copy out.** Buffers are the host's; no pointer into the core crosses. MarsRT's RDRAM must wait for its workers'
  drain before a read (`EmuSen_NativeCores.md` §3.1).
- **Signed returns and the length-query idiom**, **panic = abort with a crash log**, **one thread per handle at a
  time**, **the status bands** of `EmuSen_NativeCores.md` §3.3.

### 2.4 No per-core code above the ABI

A frontend, the runtime and the shell present a new core from its descriptors alone: its name and consoles in the
library and the engine row, its controllers on the bindings screen, its settings in the settings window with labels,
help and trade-off notes, its memory in `spaces` and `mem`, its processors in `regs` and `step`. Per-core C# survives
only where a C# oracle's exact behaviour is reproduced (MoonRT's and MercuryRT's exception types and messages, and
their mirror debuggers), and it retires with that oracle (§13).

### 2.5 The hot path never crosses a text or file layer

A frame is `advance`, perhaps `present`, `frame_info`, `frame_copy`, two audio calls and `events`: typed binary calls
with fixed arguments. JSON is read at discovery and at load; settings text is sent between frames when something
changed; the file views of §8.5 are for inspection. None of these is on the per-frame path.

---

## 3. The layers

```
  ┌───────────────────────────────────────────────────────────────────────────────────┐
  │ Clients: Mistress · Hotaru · Pharaoh · the DianaOS shell · scripts · the kit's    │
  │          runtime suite                                                            │
  └──────────────────────────────┬────────────────────────────────────────────────────┘
                                 │  the DianaOS frontend API (§9): handles, calls, a drained
                                 │  event stream; C-shaped, bound idiomatically in C#
  ┌──────────────────────────────▼────────────────────────────────────────────────────┐
  │ DianaOS, the runtime (§8)                                                         │
  │   discovery and trust · systems table · sessions and the frame loop               │
  │   services: debugger · cheats · states and rewind · settings · input routing ·    │
  │   firmware · battery · speed · logs and crashes                                   │
  │   inspection surface: /proc, /dev, ps, kill, the shell                            │
  └──────────────┬───────────────────────────────────────────────┬────────────────────┘
                 │ the engine SPI (internal; §9.4)               │
  ┌──────────────▼──────────────┐                 ┌──────────────▼────────────────────┐
  │ the v1 adapter: one class    │                 │ managed engines: the C# cores,    │
  │ over any emusen_core library │                 │ until each retires (§9.5)         │
  └──────────────┬──────────────┘                 └───────────────────────────────────┘
                 │  the core ABI v1 (§4–§6): emusen_core.h, C, stable
  ┌──────────────▼────────────────────────────────────────────────────────────────────┐
  │ Core libraries: MoonRT · MercuryRT · VenusRT · MarsRT · an outside author's core  │
  └───────────────────────────────────────────────────────────────────────────────────┘
```

| Layer | Owns | Never does |
|---|---|---|
| **Core library** | the machine; its own descriptors; its state format; its threads | file or network I/O (other than its crash log), calls into the host, process-global changes (§6.16) |
| **Core ABI** | the contract between a core and any host, its versioning and its baseline | carry console nouns: a console's vocabulary is descriptor data |
| **DianaOS runtime** | discovery, trust, sessions, the loop, every service a player sees as "the emulator's", the systems table (console facts that are not one engine's), the inspection surface | draw, play sound on a device, read a key or a pad, or know a UI toolkit |
| **Frontend API** | what any client may ask of the runtime, and its stability | expose a core's handle or the engine SPI |
| **Clients** | presentation: windows, shaders, the audio device, physical bindings, pacing against a display, the library and its scraping and themes | name a core, an engine type or a console's internals |

Two layers carry a stability promise, the core ABI and the frontend API, each with a recorded baseline guarded in CI
(§5.4, §9.3). The engine SPI between the runtime and its adapters carries none: it is internal to DianaOS and its two
implementations.

---

## 4. The stability policy of the core ABI

### 4.1 Versions

```
uint32_t emusen_core_abi_version(void);     // (MAJOR << 16) | MINOR, as built
```

- **The major is 1** and changes only on a breaking change (§4.3). It is expected never to change. A host refuses any
  library whose major is not one it speaks, and calls nothing else in it.
- **The minor** is the revision of the header the core was built against. Each addition to the baseline (§5.5)
  increments it. A host accepts every minor of its major, older or newer than its own.
- **The minor gates nothing by itself.** What a core offers is discovered from its exports and capability bits (§4.2).
  The minor is recorded in reports and used only to recognise a documented defect of a given revision, if one is ever
  found (§4.6).
- **A core's own version** is a string in its info (§6.3), for the player and the crash log. It plays no part in
  compatibility. Today's `CORE` half, which guards a core's status band, extensions and settings keys, is retired:
  the band is described by `status_text`, the settings by the schema, and the extensions version themselves (§4.7).

### 4.2 Evolution by addition

- **Optional exports**, discovered by name and claimed by a capability bit. A library exports an optional name if
  and only if it claims its bit; the conformance kit checks both directions (C2). A host calls an optional export
  only when the bit is set.
- **Capability bits are open.** A host ignores a bit it does not know. Bits 48–63 are never assigned (§6.1).
- **Every struct begins with `uint32_t size`**, set by whoever allocated it. The callee reads and writes only the
  prefix both know: `min(size, the callee's own size)`. A struct passed in an array is accompanied by its element
  size (`file_size`, `event_size`). A struct shorter than its v1.0 size is refused with `EMUSEN_BAD_STRUCT`.
  **The callee never writes `size`**, not even in a struct it fills: it stays the allocator's, and the bytes past the
  callee's own size are left as they were (C12's canary). A host that wants absent fields to read as zero zeroes the
  struct before the call; the core's minor tells it which fields a core of that minor writes.
- **Fields are only appended.** A new field's zero value must mean what an older peer assumes when the field is
  absent.
- **Enumerations are open.** A core given a value it does not know (a state kind, a space id, a port, an axis, a
  pixel format, an option bit) answers `EMUSEN_NOT_SUPPORTED` or ignores it where the header says ignoring is safe;
  never undefined behaviour. A host given an event kind or a descriptor field it does not know skips it.
- **Descriptors are JSON documents** (§6.3); a reader ignores fields it does not know, and a field a host requires is
  never removed within major 1.
- **Reserved space**: status codes in the interface's band not yet assigned; capability bits 18–47; event kinds; JSON
  field names beginning `x-` for a core's private use, never assigned by the specification.

### 4.3 What breaks version 1

Any of these requires major 2, and no such change is planned:

1. removing or renaming a required export, or an optional one that a library claiming its bit must have;
2. changing an export's parameters, return type, calling convention or meaning;
3. changing a struct field's offset, type or meaning, or shrinking a struct;
4. reusing or renumbering a status code, capability bit, enumeration value or constant;
5. making an optional export required, or adding a required export;
6. tightening what the host must do before a call, or loosening what the core guarantees after one;
7. removing a descriptor field a host may rely on, or changing its type or meaning;
8. changing the settings text's grammar, the length-query idiom or the ownership rules.

A core's **state format** is not part of the ABI. Whether a state from version *n* of a core loads in version *n+1* is
that core's own promise, recorded in its descriptors (§6.9).

### 4.4 What does not

- a new optional export with a new capability bit;
- a new field at the end of a struct, with its zero meaning "absent";
- a new value of an open enumeration, a new event kind, a new status code in an unassigned part of the interface's
  band;
- a new descriptor field, or a new value of a descriptor's enumerated string;
- a new header constant;
- a clarification of the text that changes no behaviour a conforming peer could observe.

### 4.5 Deprecation instead of removal

A call, field or value that should no longer be used is marked `EMUSEN_DEPRECATED` in the header with the minor that
deprecated it and its replacement. Within major 1:

- a core may stop exporting a deprecated *optional* export, by clearing its bit;
- a core must keep a deprecated *required* export working;
- a host must keep accepting cores that export it, and may stop calling it;
- the conformance kit keeps testing it for cores that claim it.

Nothing is removed until a major 2 that is not planned.

### 4.6 Version skew, in both directions

**A newer host with an older core.** The host calls only what the core has: exports by name, behaviour by bit. It
passes its own struct sizes and reads only the prefix the core filled. It supplies every descriptor field the core
omits from the defaults this specification states for that field. An older core is therefore presented with fewer
features, never with a failure.

**An older host with a newer core.** The host never calls the new exports and passes smaller structs. So the rule a
core must keep is: **every addition is safe never to be used.** A core that gains `X` must behave as a v1.0 core
when `X` is never called and the host's structs are v1.0's. Where a core cannot work unless the host does something
new, it lists that in its info's `host_requires` (§6.3): names of host obligations, each defined in this page. A host
that does not know a name in `host_requires` refuses the core cleanly, with a report that names it. v1.0 defines one
obligation, `present`: a host must call `emusen_core_present` after `advance` when `PRESENT` is claimed. Every v1.0
host meets it, so it need never be listed; it is named so the mechanism is exercised from the first day.

**A documented defect of one minor.** If a released minor is found to specify something wrongly, the correction is
an addition, and a host may treat cores whose recorded minor predates it accordingly. The mechanism exists so that
no correction ever needs a major.

### 4.7 Extensions

A core's own exports, outside the common set, keep the crate's prefix (`marsrt_*`, `mercuryrt_*`; `EmuSen_NativeCores.md`
§3.15). They are outside this page's promise. A host that uses one does so knowingly, for one core, and checks that
core's info (`id` and `version`) first. An extension that becomes common is added to the baseline under the
`emusen_core_` prefix and its old name is deprecated.

### 4.8 The exact-match rule, retired

`EmuSen_NativeCores.md` §9 Q3 decided exact matching on both halves, reasoning that "the libraries ship beside the
assemblies that load them. The version check guards a mismatched prebuilt library, not an ecosystem" (§2 there). That
was true of the binaries and missed the cost, which was in the source. Every change to the interface or to a core's
number moved every core and every shim together, and the frontends kept re-reading moving code. Exact matching does
not reduce that churn; it enforces it. It also makes an outside core impossible by construction. It is retired for
the stable set. The pre-stable `emusen_native_*` set keeps it until that set is deleted (§13.5).

---

## 5. The canonical artefact, and how it is guarded

### 5.1 Three artefacts, one contract

| Artefact | Role | Where |
|---|---|---|
| `emusen_core.h` | **the specification an outside author reads**: every export, struct, constant and enumeration, with the rules in comments, C99 and C++-safe | `EmuSen/Cores/Shared/emusen-native/include/emusen_core.h` |
| `emusen-native`'s `core` module and `core_exports!` | **the reference implementation for cores**: the trait a Rust core implements, and the macro that generates every export from it | the same crate, beside today's `abi` module |
| The v1 adapter in DianaOS | **the reference implementation for hosts** | `EmuSen.DianaOS/DianaOS/Sys/Native/` (§8.2) |
| `abi/v1/baseline.txt` | the recorded ABI, machine-readable (§5.4) | beside the header |

This page is the prose that explains them, and wins over the header's comments where they disagree; a disagreement is
a defect in the header. The draft header of Appendix A compiles cleanly as C99 with `-Wall -Wextra -pedantic
-Werror`, as C11 under clang and as C++17 (**measured**). Writing it found one defect of naming, now a rule: the first
draft named the type `emusen_core_frame_info` and the export `emusen_core_frame_info`, and C refuses a type and a
function of one name. **Exports are `emusen_core_*`, types are `emusen_*` without `core`, constants are `EMUSEN_*`.**

### 5.2 The header and the Rust definitions: checked, not generated

The header is written by hand, because its comments are normative prose and generated headers carry none. The Rust
definitions are written by hand, because they are the reference implementation and must read as Rust. They are kept
identical by a check that fails a build:

- A CI-only crate, `emusen-core-abi-check`, runs `bindgen` on `emusen_core.h` in its build script. Nothing generated
  is committed, and no core's build depends on it.
- The header declares a function type for every export (`emusen_core_advance_fn` and the rest). The check assigns each
  of `emusen-native`'s generated exports to bindgen's type of the same name, so a parameter or return that differs is
  a compile error.
- The check asserts `size_of`, `align_of` and every `offset_of!` of each `#[repr(C)]` struct against bindgen's struct,
  and every Rust constant against the header's macro.

**A prototype was run** (`~/.cache/emusen/probe/core-api/sigcheck/`, **measured**): bindgen 0.72 from the local cargo
cache, with libclang 22, on a header with one struct and two function types:

- with the signatures equal it builds;
- with one parameter changed from `*mut u64` to `*mut u32` it fails with `E0308: mismatched types`;
- with the 64-bit field moved to the end of the Rust struct, the size assertion fails;
- **with two 32-bit fields swapped, the first version of the check passed.** It asserted the struct's size and one
  field's offset, and a swap of two fields of one size changes neither. With an `offset_of!` assertion for the swapped
  field by name, it fails. So the check asserts every field's offset by name, never a sample.

`bindgen` needs libclang on the CI image; a core's own build never does.

### 5.3 The header and the C# host

The host's export table is a set of `delegate* unmanaged<…>` fields (`NativeInterface.cs`). .NET exposes their
signatures by reflection: `FieldInfo.GetModifiedFieldType()` gives a function-pointer type with
`GetFunctionPointerReturnType()` and `GetFunctionPointerParameterTypes()`. **Measured** in a scratch program
(`~/.cache/emusen/probe/core-api/fnptr/`) on .NET 10: a field declared `delegate* unmanaged<nint, ulong*, int>` reports
`System.Int32` and `System.IntPtr, System.UInt64*`, and one declared `delegate* unmanaged<nint, void>` reports
`System.Void`. A WiseMan test therefore compares every field of the v1 table with the baseline's line for that export,
and `Marshal.SizeOf`/`OffsetOf` of every C# struct with the baseline's layout. Run against today's table it would fail
on `set_crash_log`'s `void` (§1.2, item 8), which is the point.

### 5.4 The ABI baseline, and the check that guards it

`abi/v1/baseline.txt` holds one fact per line, sorted, for each target triple the project builds (x86-64 Linux,
x86-64 Windows, AArch64 and x86-64 macOS):

```
fn emusen_core_advance (emusen_machine*, uint64_t*) -> int32_t  @1.0
struct emusen_frame_info size 56 align 8                        @1.0
field emusen_frame_info.serial offset 40 type int64_t           @1.0
const EMUSEN_CAP_PRESENT 0x2                                    @1.0
enum emusen_event_kind.EMUSEN_EVENT_GEOMETRY 2                  @1.0
status EMUSEN_NOT_SUPPORTED -256                                @1.0
json info.systems[].extensions array<string> required           @1.0
```

It is generated by `emusen-core-abi-check --emit-baseline` from libclang's and bindgen's views of the header
(functions, structs, macros, enumerations), from `emusen-native`'s export table (an `export` line per export, required
or claimed by its bit) and from the JSON schemas of §6.2, §6.3, §6.4, §6.13 and §6.14 (descriptor fields). The facts
are generated for each of the four triples and must be identical, which the check proves rather than assumes; so the
file holds each fact once. A line keeps the tag it was committed with; a new line is tagged with the header's
`EMUSEN_CORE_ABI_MINOR`. Lines beginning `#` are comments. The CI job `abi-guard` runs on every
pull request:

1. It regenerates the facts from the pull request's header and diffs them with the committed baseline.
2. **A removed or changed line fails the job**, with the message that the change breaks version 1 and the rule of §4.3
   it breaks.
3. **An added line fails the job unless the same pull request adds it to the baseline** (§5.5).
4. It runs §5.2's Rust check and §5.3's C# check.
5. It checks every built core library with `nm`: every required export present, every optional export present if and
   only if its bit is claimed, and no `emusen_core_` symbol that the baseline does not list. That replaces today's
   export count in `rust-cores.yml` (lines 53–66, 163), which counts names and cannot tell a rename from an addition.

### 5.5 Adding to the baseline

A pull request that adds to version 1:

- adds the declaration to the header, with `@since 1.N`, and increments `EMUSEN_CORE_ABI_MINOR`;
- adds the lines to the baseline, each tagged `@1.N`;
- implements it in `emusen-native` (with its default for cores that do not claim it) and in the host adapter (with
  its behaviour for cores that do not have it);
- adds the conformance case that tests it, and a skew case (§12.5);
- records it in §17's changelog, with the reason.

No line tagged with a released minor is edited afterwards. A mistake in one is corrected by deprecation and addition.

---

## 6. The v1 surface, call by call

§3 of `EmuSen_NativeCores.md` is the starting point. Each piece is given a verdict:

- **In**: enters v1 as it is, renamed to the `emusen_core_` prefix;
- **Revised**: enters v1 changed, with the reason;
- **New**: not in the pre-stable set;
- **Extension**: stays outside the common set.

The prefix changes for the whole stable set, so that a library's two possible interfaces can never be confused by a
loader, and so that the baseline starts clean (§15, Q1). The pre-stable `emusen_native_*` names keep their exact
matching until they are deleted (§13.5).

### 6.0 The rules every export follows

Those of §2.3, plus:

- **No `void` export and no unsigned getter**, so that a null machine always reads as `EMUSEN_NULL` (−1). The two
  exceptions are library-level calls with nothing that can fail: `abi_version` and `capabilities`.
- **A null `out` with the length-query idiom asks only the length**; that now holds for every copying export,
  including `space_read`, which today answers a null `out` with `NULL` (`abi.rs:734`).
- **Errors are values.** No export unwinds, longjmps, raises a signal or calls `exit`.

### 6.1 Library-level calls

| Export | Verdict | Notes |
|---|---|---|
| `abi_version() -> u32` | **Revised** | major and minor (§4.1), in place of `(COMMON << 16) \| CORE` |
| `capabilities() -> u64` | **In** | bits 0–13 keep the pre-stable numbering (`abi.rs:21`); 14–17 are new (Appendix A); open (§4.2) |
| `info(out, len) -> i64` | **New** | the core's info document, JSON (§6.3) |
| `settings_schema(out, len) -> i64` | **New** | §6.13; settles `EmuSen_NativeCores.md` §9 Q4 |
| `firmware_for(image, len, out, out_len) -> i64` | **New** | what this image needs, from its header alone, JSON (§6.2), as `ICore.GetFirmwareRequirements` answers without loading (`ICore.cs`) |
| `status_text(status, out, len) -> i64` | **New** | words for a code in the core's band, so a host needs no per-core table |
| `set_crash_log(path) -> i32` | **In** | the host's declaration is corrected to `int32_t` (§1.2, item 8) |
| `log_drain(machine?, out, len) -> i64` | **New** | the core's log records, pulled (§6.18); a null machine drains the library's |

All are callable from any thread at any time, before and after any machine exists (§6.16).

### 6.2 Lifecycle

```c
emusen_machine *emusen_core_create(const emusen_create_params *params, int32_t *status);
int32_t emusen_core_free(emusen_machine *machine);
int32_t emusen_core_reset(emusen_machine *machine);                       // RESET
int64_t emusen_core_machine_info(const emusen_machine *machine, uint8_t *out, size_t len);
int64_t emusen_core_last_error(const emusen_machine *machine, uint8_t *out, size_t len);
```

- **`create` — Revised.** Its seven arguments (`abi.rs:528`) become one `emusen_create_params`, so it can grow: the
  image, the settings text (create-time and run-time keys together, §6.13), the files with their element size, the
  host's version, **the pixel formats the host accepts** (§6.6) and an optional error buffer. The contract of
  `EmuSen_NativeCores.md` §3.4 is otherwise kept: the image is the file's bytes and the core parses it; a refusal
  returns null with a status, and copies its detail into `error` when given: at most `error_len − 1` bytes of UTF-8,
  cut at a character boundary, then a NUL, so the host finds the end without a returned length; a failed create
  leaves nothing to free.
  The core copies what it keeps; no pointer in `params` outlives the call.
- **Files.** `which` 0 is the battery save. Other numbers are the core's, named in its machine info (`battery`, §6.11)
  or in `firmware_for`'s answer. **Firmware** is asked for before create, from the image, and passed as files:

  ```json
  [{ "which": 16, "name": "dsp1.rom", "label": "DSP-1 program and data", "size": 8192, "required": false,
     "parts": [["dsp1.program.rom", "dsp1.data.rom"]] }]
  ```

  `required: false` means the game loads without it, as VenusRT's plan requires for the NEC DSPs (`VenusRT_Plan.md`
  §4.2). `parts` lists alternative split forms, each part passed as its own file, numbered from `which` upwards.
  Where the files are kept is the runtime's (`FirmwareLibrary`, `home/Firmware`), never the core's.

  *Decided 2026-10-03, for every core the project builds:* no firmware entry is `required`. Each piece of firmware has
  an open replacement inside the core, so every game runs with no firmware folder, and an image the tester or player
  supplies is used, when present, as the exact path. A frontend never prompts for firmware. `EmuSen_Firmware.md` §0 is
  the policy; a foreign core may still declare `required: true`, and its games then wait for the file.

  *Added 2026-10-04, in v1.0's baseline* (`VenusRT_DspHle.md` §7.2, `VenusRT_Native.md` §38): an entry may carry
  `replacement: { effect, cost }`, saying what the core runs without the file. `effect` takes the settings schema's
  words (§6.13): `exact`, `accuracy` with a required `cost` in plain words, or `none` when the game runs without the
  chip, its `cost` saying so. An entry without it reads as no replacement. The host prompts only for a `required`
  entry, and shows one status line when a game runs on a replacement short of `exact` or without its chip. Kit case C4
  checks the words and creates an image whose entries are all optional with no files.

  *Amended 2026-10-05.* A host no longer shows a status line for a running replacement, only for a chip that runs
  with none (`VenusRT_Native.md` §66). The cost is shown on the firmware page (`EmuSen_Settings_Reference.md` §4.89),
  which takes **the cost's first sentence**, up to the first full stop followed by a space, as the summary a player
  reads, and shows the whole text on request. A core therefore opens each cost on one short sentence in a player's
  words ("Games may start a fraction of a second later."), with no engine name or address in it, and puts the detail
  after it. A cost of one sentence is shown whole. The page shows an entry's `label` followed by its `name` in
  brackets, so a label is the thing's name in plain words, without its size or file name.
- **`free` — In.** It may be called from any thread, provided no other call on that machine is in flight; that is what
  the host's finaliser already does (§1.2, item 9). Every thread the core started for the machine has stopped when it
  returns.
- **`reset` — In**, behind `RESET`: the console's reset button.
- **`machine_info` — New.** The descriptor of this machine for this game (§6.4). The host reads it after create and
  again after an `EMUSEN_EVENT_MACHINE_INFO`.
- **`last_error` — New.** The detail of the last failing call on this machine, in words, for the log and the
  player. A status stays the machine-readable answer; the text is never parsed.

### 6.3 Core info: who the core is, without loading a game

The analogue of libretro's system info, extended so that **a frontend can list a core, register it and present it
with no per-core code**. This replaces `EmuSen_NativeCores.md` §5's hand-written registration records for native
engines, and `CoreCatalog`'s per-engine constants and rows (§1.2, item 4).

**Why JSON.** `EmuSen_NativeCores.md` §10 rejected messages across the boundary because "they buy schema evolution
between binaries built separately, which never happens here". It now happens by decision, so that argument reverses
for what is read once. A descriptor is read at discovery and at load, never per frame. As JSON it is:

- extensible without a struct per descriptor;
- ignorable field by field by an older reader;
- validated by a published schema;
- readable by an outside author, and by `cat`.

The hot path stays binary (§2.5). The cost of the descriptors was not measured; it is argued to be a few
milliseconds per load (P8).

**The fields** (required ones in bold):

| Field | Meaning |
|---|---|
| **`abi`** | `"1.0"`, which must equal `abi_version` |
| **`id`** | a stable machine name, `[a-z0-9-]+`, unique among cores: `"venusrt"` |
| **`name`**, `display_name` | `"VenusRT"`; `"VenusRT (Rust)"`, today's engine-row text |
| **`version`** | the core's own version string |
| **`license`** | an SPDX expression |
| `authors`, `url`, `description` | for the about page and the core list |
| **`systems`** | the consoles it runs, below |
| **`capabilities`** | the names of the bits it claims, which must equal `capabilities()` |
| `host_requires` | host obligations beyond v1.0 (§4.6) |
| `deterministic` | true unless the core cannot promise §6.9's determinism; v1 compliance requires true |
| `accuracy` | `{ "measured_with": "defaults", "suite": "<name>", "notes": "…" }`: where the core's accuracy record lives, and that it was measured at the defaults |

Each **system** entry:

| Field | Meaning |
|---|---|
| **`id`** | the system's stable id: `"nes"`, `"snes"`, `"gb"`, `"gbc"`, `"n64"`; the runtime's systems table (§8.2) knows these, and an id it does not know is shown with the core's own `name` |
| **`name`** | `"Super Nintendo Entertainment System"` |
| **`extensions`** | `[".sfc", ".smc"]` |
| `regions` | `["ntsc", "pal"]` |
| `controllers` | the devices a port can hold, below |
| `development` | true when the core does not run the system's games yet; absent is false (§27.4, added 2026-10-06) |
| `firmware` | every firmware file the system can use, for the firmware window, even before a game. One entry for each image a game can name, with that image's own label, size, parts and replacement, as `firmware_for` gives them; entries for images that are alternatives in one file slot share its `which` (amended 2026-10-05, `VenusRT_Native.md` §67) |

A **controller** names its controls in the canonical vocabulary, so that the bindings screen is built from data and
the physical bindings stay the frontend's:

```json
{ "id": "snes.pad", "label": "Controller", "ports": [0, 1],
  "buttons": [ { "bit": 0, "control": "B", "label": "B" }, { "bit": 4, "control": "Up", "label": "Up" } ],
  "axes": [] }
```

An axis entry is `{ "axis": n, "control": "LeftX", "kind": "stick", "label": "Control Stick" }`: `axis` is the number
`set_axis` receives, `kind` is `stick` (−1 to 1) or `trigger` (0 to 1), so that a host can normalise a non-canonical
axis too. The `buttons` are listed in the order a bindings screen presents them, which need not be the order of their bits
(MoonRT lists Up, Down, Left, Right, Select, Start, B, A, as its C# oracle presents them). `control` is one of `PadButton`'s names (`EmuSen.Galaxia/Input/PadButton.cs`, append-only, 16 today) or, for an
axis, of `PadAxis`'s; the header carries both as open enumerations (Appendix A). `label` is the console's own word,
which the diagram shows: the N64's "C-Up", the Game Boy's "Select". A control with no canonical match is allowed, with
`"control": null`, and the frontend offers it as an unbound extra.

Appendix B gives MoonRT's info as it would be written.

### 6.4 Machine info: this machine, for this game

What depends on the game, read after create:

| Field | Meaning, and what it replaces |
|---|---|
| **`system`**, `region` | which system entry and region the game runs as |
| **`frame_rate`** | `{ "num": 21477272, "den": 357366 }`: the pacing rate, exact; `ICore.FrameRateHz` today |
| **`video`** | `base_width`, `base_height`, `max_width`, `max_height`, `aspect` as a ratio, `formats` the core can produce |
| **`audio`** | `rate` at start; `channels`: the names `set_mutes`' bits stand for |
| **`ports`** | which controller each port holds |
| **`spaces`** | `{ id, name, size, flags }`; `flags` an array of the strings `read_only`, `side_effects`, `reports_stores`, `cheats` (open, as any enumerated string). `size` and `battery`'s `length` are written by the crate from `space_size` and `battery`, and `state.kinds` from `SNAPSHOT`, so they cannot disagree with the exports (C14). The names are the C# oracle's where there is one (`MoonMachine.cs:17`), so the cheats and the debugger key on them as now |
| `processors` | `{ id, name, pc_bits, registers: [{ name, bits }], code_space }`, main processor first; `IDebugTarget.DebugCpus` and `regs` from data. `code_space`, appended 2026-10-04 for VenusRT's SPC700 (`VenusRT_Native.md` §36), is the id of the space the processor's code is listed from; absent, the host takes the space named `<name>BUS`, else `<name>PRG`, else for processor 0 the first space |
| `battery` | `{ which, suffix, length }`: `.srm`, the N64's pak; the path rule stays the runtime's (`SaveLibrary.SramPathFor`) |
| **`state`** | `{ format, version, kinds, loads_from }`: the magic and version a frontend records beside a state (`IStateFormat`), the kinds, and the versions it reads |
| `phases` | the names `phases` reports, in order |
| `patches` | `{ low, high }`: the address range `ResolveRomPatches` is kept to |
| `skip_rendering_state_neutral` | true when skipping rendering leaves the state as a rendered frame would; run-ahead needs it (§6.22), and VenusRT designs for it (`VenusRT_Plan.md` §4.1) |
| `achievements` | reserved; §6.22 |
| `firmware` | `{ which, source }` for each firmware file the game names: `file`, `replacement` or `absent`, the path this machine runs on. Added 2026-10-04 with §6.2's `replacement` (`VenusRT_Native.md` §38); absent, a host assumes nothing |

Every field a host requires has a stated default when absent, so an older core is read by a newer host (§4.6). The
defaults, as the host adapter applies them (§19): `frame_rate` 60/1; every `video` size 0 and `aspect` 0/0, so that the
first `frame_info` decides; `audio.rate` the `audio_rate` export's answer; `ports`, `spaces`, `processors`, `battery`,
`phases` and `firmware` empty; `state.kinds` `[0]`, its `format` empty and `version` 0; no `patches`, so no ROM patches are sent;
`skip_rendering_state_neutral` false.

### 6.5 A frame

| Export | Verdict | Notes |
|---|---|---|
| `advance(m, *detail) -> i32` | **In** | the machine to the frame's end; a negative return is a status with the core's detail word |
| `present(m) -> i32` | **In**, behind `PRESENT` | v1.0 obliges the host to call it after `advance` when claimed and rendering is not skipped (§4.6) |
| `set_options(m, flags) -> i32` | **Revised** | bit 0 skips rendering; **every other bit is reserved**, and a core ignores bits it does not know. The pre-stable "bits 1–23 are the core's" (`abi.rs:152`) goes: MarsRT's bits 1, 2 and 4 become settings with `effect` declared |
| `frame_count(m) -> i64` | **In** | |
| `phases(m, out, len) -> i64` | **In**, behind `PHASES` | nanoseconds; the names are machine info, not C# |
| `events(m, out, count, event_size) -> i64` | **New** | §6.19 |

### 6.6 The picture, and size negotiation

```c
int32_t emusen_core_frame_info(const emusen_machine *m, emusen_frame_info *out);
int64_t emusen_core_frame_copy(const emusen_machine *m, uint8_t *out, size_t len);
```

**`frame_info` — Revised.** `emusen_frame_info` gains `size`, `format`, `stride`, the display aspect and a reserved
word (Appendix A; 56 bytes, **measured** by `_Static_assert` under gcc and clang). `frame_copy` is **In**.

- **The pixel format is negotiated once.** The host lists what it accepts in `create`'s `pixel_formats`; RGBA8888
  (bit 0) is always accepted, because it is the only format `ICore.GetFrameBufferRgba` knows today. The core produces
  one of them and says which in every `frame_info`. A core that can produce only RGBA8888 ignores the mask. A host
  that receives a format it did not offer refuses the frame and logs it.
- **The size is reported every frame**, because pictures change size: VenusRT's is 256 or 512 wide (`VenusRT_Plan.md`
  §4.1), MarsRT's follows the game's video registers and its internal resolution. The host allocates once from machine
  info's `max_width` and `max_height` and never reallocates per frame. A size change also raises
  `EMUSEN_EVENT_GEOMETRY`, so a presenter can relayout before it draws.
- **The aspect** is that of the picture as a television showed it, so a frontend letterboxes without per-console
  code; `0/0` means square pixels.
- **`row_repeat`** (`ROW_REPEAT`) and **`serial`** (`FRAME_SERIAL`) keep their meanings: rows shown more than once,
  and a serial that moves only when the picture does (`EmuSen_Multicore.md` §14, §15). Without the bit, `row_repeat`
  is 1 and `serial` is the frame count; `core_exports!` writes both, so a core cannot report otherwise.
- **The console's own lines** are not in `emusen_frame_info` at 1.0, and the reserved word is where they go. A core that
  draws its frame at a whole multiple of the console's picture (an internal resolution) says how many of the console's
  lines the frame holds, so that a filter which draws a screen draws those and not the frame's rows
  (`EmuSen_CRT.md` §16); zero, which every 1.0 core writes, means the rows are the lines. Decided 2026-10-07: the
  frontends take it from `ICore.DisplayLines` now, since the only cores that draw at a multiple are not on this
  interface, and the word is named in the minor that first has a core to fill it.

### 6.7 Sound, and what its rate means

| Export | Verdict | Notes |
|---|---|---|
| `audio_rate(m) -> i32` | **In** | the rate of the next sample a drain returns |
| `audio_buffered(m) -> i64` | **In** | samples, two per stereo frame |
| `audio_drain(m, out, len, max_frames, *rate) -> i64` | **Revised** | **a drain never crosses a rate change**, and reports the rate its samples were made at. This builds `EmuSen_NativeCores.md` §9 Q10's decision, which was not built |
| `set_audio_limit(m, samples) -> i32` | **In** | the drop-oldest bound |
| `audio_peek(m, out, len) -> i64` | **In**, behind `AUDIO_PEEK` | a non-destructive copy, for `audiodump` |
| `set_mutes(m, mask) -> i32` | **In**, behind `MUTES` | bit n mutes machine info's channel n |

**At the boundary.** `audio_rate` is the rate of the next sample a drain would return, so it changes when a drain
passes the last sample made at the old rate, not when the core starts making samples at the new one. A drain with a
null `out` returns the samples buffered and still reports the rate. `EMUSEN_EVENT_AUDIO_RATE` is raised when
`audio_rate` changes, which the macro observes after `advance`, `state_load`, `set_settings` and `reset`, and after the
drain that crossed the change; that event is drained with the next frame's.

**The rate is a report, not a request**, as `EmuSen_NativeCores.md` §3.7 set out: Moon and Mercury report a fixed
44,100, Mars the rate the game programmed. A core whose output rate is a choice (Venus follows
`AudioSettings.SampleRate`) takes it as a setting key, `audio.rate`, of `effect: none`. A change raises
`EMUSEN_EVENT_AUDIO_RATE`. Samples are interleaved stereo `int16`; another format would be a new export, never a
reinterpretation of this one.

### 6.8 Input

| Export | Verdict | Notes |
|---|---|---|
| `set_buttons(m, port, mask, changed) -> i32` | **In** | `changed` is kept for the reason `EmuSen_NativeCores.md` §3.8 found: MarsRT's pad is in its state, MoonRT's and MercuryRT's are not |
| `set_axis(m, port, axis, value: f64) -> i32` | **Revised**, behind `AXES` | **the value is normalised**: sticks −1 to 1, right and down positive, triggers 0 to 1, as `ICore.SetAxis` and `PadAxis` define them (`PadAxis.cs`) |

**Why the axis is revised.** §3.8 there passed "the console's own units", scaled by each console's C# (Mars rounds
`value × StickReach`). A host with no per-core code cannot know a console's units, so the scaling moves into the core.
The value stays a `double` so that a port reproduces its C# oracle's rounding to the bit: C# computes from a
`double`, and IEEE arithmetic on the same operands gives the same result in Rust (argued; MarsRT's step 5 is its
test). An integer normalisation, as libretro uses, would quantise before the core's own rounding and could differ in
the last step.

**Descriptors** are in core info (§6.3) and machine info's `ports` (§6.4). A frontend binds physical keys and pads to
the canonical controls, as it does now; the runtime routes each control to a port's bit by the descriptor. Devices
other than a pad (a mouse, a light gun, a multitap: `VenusRT_Plan.md` §9 Q8 lists them as later scope) would be a
later minor's `set_device` and device kinds; v1.0 leaves the field `controllers` open for them. Outputs, such as
rumble, would be pulled through events (§6.19); v1.0 defines none.

### 6.9 State

| Export | Verdict | Notes |
|---|---|---|
| `state_size(m, kind) -> i64` | **In** | |
| `state_save(m, kind, out, len) -> i64` | **In** | |
| `state_load(m, data, len) -> i32` | **In** | C#'s whole `LoadState`; a failed load changes nothing |
| `state_layout(m, kind, out, len) -> i64` | **In** | one line per field |

**Revised in behaviour, not in signature:**

- **Kinds are an open enumeration**: 0 the state, 1 the snapshot (`SNAPSHOT`). **The snapshot is generated by the
  macro**, closing §1.2's item 2: a core implementing the trait's `save_snapshot` gets kind 1, and one that does not
  answers `NOT_SUPPORTED`, as now.
- **The size contract.** A kind's size is constant for the life of a machine, **except across a `state_load` that
  changes the machine's configuration**. MarsRT's load of a state of the other RDRAM size rebuilds the machine
  (`EmuSen_NativeCores.md` §3.9), so the pre-stable trait's absolute rule (`abi.rs:129`) is false for it. Such a load
  raises `EMUSEN_EVENT_STATE_SIZE`. A host that needs a fixed size, such as a libretro adapter's
  `retro_serialize_size`, reports the larger of the two.
- **Determinism.** Given the same image, settings, files and the same `set_buttons`/`set_axis` sequence at the same
  frames, a core produces the same pictures, samples and states. Its threads, its recompiler and its host's speed may
  not change them. MarsRT's threaded display processor and recompiler are bit-identical by measurement
  (`Mars_Native.md`), so the rule asks nothing they do not already do. The kit tests it (C8).
- **Saving settles first, loading is whole, a failed load changes nothing**: kept from §3.9 there.
- **The format is the core's**, versioned by machine info's `state` field (§4.3). A port's format is its C# oracle's;
  VenusRT's is its own (`VenusRT_Plan.md` §5.6).

### 6.10 Memory spaces

`space_size`, `space_read`, `space_write`: **In**, with the 32-bit address and 64-bit size of §3.10 there. **Revised:**
the names and flags move from C# into machine info (§6.4); the ids stay the core's; the null-`out` length query is
added to `space_read` (§6.0). What lies past a space's end stays the core's rule.

### 6.11 Battery data

`battery(m, which, out, len, *flags)` and `battery_saved(m, which)`: **In.** **Revised:** machine info lists the files
(`which`, suffix, length), so the runtime chooses paths with no per-core code. `BATTERY_DIRTY` keeps its meaning.

### 6.12 Cheats

| Export | Verdict | Notes |
|---|---|---|
| `set_rom_patches(m, triples, count) -> i64` | **In**, behind `ROM_PATCHES` | `ResolveRomPatches`' list, as now |
| `set_cheat_pokes(m, quads, count) -> i64` | **New**, behind `CHEAT_POKES` | (space, address, value, compare) the core applies itself at its frame's end, for a core whose oracle gates them |

**Codecs stay out of the ABI.** A code is parsed by the runtime's codecs, which are a console's knowledge, not an
engine's (§8.2), and resolved to patches and pokes. By default the runtime applies the pokes itself through
`space_write` between `advance` and `present`, as `NativeRtCore.EndFrame` does today. A core whose oracle applies them
under a condition the host cannot see claims `CHEAT_POKES` and applies them itself: MarsRT holds its pokes while
interrupts are off (`EmuSen_NativeCores.md` §3.12), which today needs C# that reads the Status register.

### 6.13 The settings schema

`EmuSen_NativeCores.md` §9 Q4 deferred the schema until the C# cores retire, "so there is never a second source for
the same list". A framework that presents outside cores cannot wait. **Decided here, and confirmed by §15 Q2:** the core
is the one source for its settings, and the C# lists (`MarsCore.VideoSettings`, `MercuryCore.ModelSettings`,
`CoreCatalog.SettingsFor`) are generated from or replaced by the schema engine by engine, so that there is still one
source per engine.

```c
int64_t emusen_core_settings_schema(uint8_t *out, size_t len);                         // library-level
int32_t emusen_core_set_settings(emusen_machine *m, const uint8_t *text, size_t len);  // SETTINGS
int64_t emusen_core_setting_notes(const emusen_machine *m, uint8_t *out, size_t len);  // SETTING_NOTES
```

**A setting** (JSON; required fields in bold):

| Field | Meaning |
|---|---|
| **`key`** | `[A-Za-z0-9_.-]+`, stable for major 1 of the core's schema |
| **`label`**, **`help`** | readable words: the row's text and its explanation |
| **`kind`** | `switch`, `count`, `choice`, `text`; open |
| **`default`** | text, as `CoreSetting.Default` is |
| `min`, `max`, `step` | for a count |
| `choices` | `[{ "value": "2", "label": "2× (1280×960)", "help": "…" }]` |
| **`scope`** | `create` (read only at create: `Model`, `ExpansionPak`) or `run` (applied between frames) |
| `category` | `"Video"`, `"Audio"`, `"Speed"`, `"System"` |
| **`effect`** | `none`, `exact`, `accuracy`, `latency`, `enhancement` (§2.1) |
| `cost` | required unless `effect` is `none` or `exact`: what the player gives up, in plain words |
| `accurate` | required for `accuracy`: the accurate value, **which must be the default** |
| `hardware` | required for `enhancement`: the hardware's value, **which must be the default** |
| `advanced`, `hidden` | shown behind "advanced", or not at all (`VerifyRdp`, `BlockTier`) |
| `restart` | the setting takes effect only at the next load |

Two entries as they would be written for MarsRT:

```json
{ "key": "RenderScale", "label": "Internal resolution", "kind": "choice", "default": "1",
  "choices": [{ "value": "1", "label": "1× (console)" }, { "value": "2", "label": "2×" }],
  "scope": "run", "category": "Video", "effect": "enhancement", "hardware": "1",
  "cost": "Each step costs its square in drawing: 2× is four times the work.",
  "help": "The picture drawn at a multiple of the console's, beside the exact drawing games read back." }
{ "key": "RdpWorkers", "label": "Rasteriser threads", "kind": "count", "min": 1, "max": 16, "default": "2",
  "scope": "run", "category": "Speed", "effect": "exact",
  "help": "How many processors share each list. Exact at any count." }
```

**Presentation follows from `effect`,** the same in every frontend: speed settings that change nothing (`exact`) in
one group; trade-offs (`accuracy`, `latency`) in a group labelled as trade-offs, each with its `cost` beside it;
`enhancement` in its own group. Mistress's Graphics Settings window renders from the schema instead of from
`CoreCatalog.SettingsFor`.

**Setting values** are sent as `key=value` lines: a key as above, a value of UTF-8 with no CR or LF, whitespace around
both trimmed (`Settings::parse`, `abi.rs:108`). A switch's values are `true` and `false`, as `CoreSetting` writes them;
a count's a decimal integer within `min` and `max`; a choice's one of its `value`s. A key given twice in one text is
`EMUSEN_BAD_SETTING`. `core_exports!` checks every line against the schema before the core sees it, and at create it
fills every key not given with its default, so a core's `create` always receives the whole schema's values and C4's
"no settings equals every default" holds by construction. `set_settings` applies every line of one call together, as MarsRT's
coupled groups need (`EmuSen_NativeCores.md` §3.13). Create-time keys go in `create`'s settings text; sending one to
`set_settings` is `EMUSEN_BAD_SETTING`. An unknown key is `EMUSEN_UNKNOWN_SETTING`, and the host drops it rather than
failing the load, so a stored setting from a newer core is harmless to an older one.

**Notes.** `CoreSetting.Note` is a function from the console's values to a sentence "for beside the row when the core
will not use the value as chosen" (`CoreCapabilities.cs`). A function cannot cross; its result can.
`setting_notes` (`SETTING_NOTES`) returns `{ "key": "sentence" }` for the machine's current values, read after each
`set_settings`. MarsRT's antialiasing note is the case.

**A finding to settle.** MarsCore's `DeferredPresentation` ("the picture … reaches the screen one frame late[r]")
defaults to on (`MarsCore.cs`). Under this schema it is `effect: latency`. The rule that defaults are accurate
applies to `accuracy` and `enhancement`; §15 Q12 decided that it does not extend to latency.

### 6.14 The debug interface

The eleven exports of `EmuSen_NativeCores.md` §3.14, as built for MoonRT and MercuryRT (`abi.rs:276`–`:450`), are
**In**, behind `DEBUG` and `DEBUG_STACK`, with these **revisions**:

- **`debug_run_frame` reports which processor stopped**, `(m, flags, *processor, *pc, *detail)`. A halt on the SA-1 or
  the SPC700 can then say where it is (§1.2, item 10; `VenusRT_Plan.md` §8).
- **`debug_set_breakpoints` takes a processor**, because a coprocessor runs different code at the same addresses and
  has its own registry (`IDebugTarget.CoprocessorBreakpoints`; `EmuSen_Debugging_Tools_Reference_v5.md` §3.1a).

**New**, so that a debugger works on a core it has no C# for:

- `debug_registers(m, processor, out, len)` (`DEBUG_REGISTERS`): the values, in the order of machine info's
  `processors[].registers`;
- `debug_disassemble(m, processor, space, address, count, out, len)` (`DEBUG_DISASSEMBLE`): JSON records of address,
  bytes, mnemonic, operand text and, where knowable from the bytes alone, a static reference with its kind. These are
  the fields of `DisassembledInstruction` and `ClassifyStaticReference`, and the rule "the core decodes; the command
  formats" (§3.1a there) is kept: the core decodes, DianaOS formats.

**Extensions, not v1:** the console-shaped views (sprites, palettes, tiles and tilemaps, DMA channels, coprocessor
register flow). A generic debugger shows memory, registers, disassembly, breakpoints, watches, stepping, the call
stack, coverage and the profile for any core that claims `DEBUG`, and a console's views come from its system pack in
DianaOS (§8.2) or from a core's extension exports. §15 Q10 decided that views do not enter v1.0.

### 6.15 The status-code space

The bands of `EmuSen_NativeCores.md` §3.3 are **In**, fixed for major 1: −1 to −8 the state crate's; −9 to −255 the
core's; −256 to −319 the interface's; −320 to −383 a reproduced .NET exception; below, reserved. **New:**
`EMUSEN_BAD_STRUCT` (−263) and `EMUSEN_BAD_IMAGE` (−264), a generic refusal for a core without a finer code.

- **The core's band is described by `status_text`**, so a generic host can report it. A port's C# shim may still map
  its band to its oracle's exception types and messages, which are pinned by tests (§2.4).
- **The fault band is for ports.** A clean-room core such as VenusRT has no C# exception to reproduce and does not use
  it.

### 6.16 Threading and re-entrancy

- **One thread at a time per machine.** Different machines may be driven from different threads at once.
- **Library-level calls** (§6.1) are safe from any thread, concurrently with each other and with machine calls.
- **No re-entrancy is possible**, because the core never calls the host.
- **The core may run threads of its own** behind a machine. They are idle when no call is in flight (or their work is
  invisible to every export), and stopped when `free` returns.
- **`free` from any thread**, provided nothing else is in flight on that machine (§6.2).
- **What a core never changes in the process:** signal handlers, the floating-point environment as seen after a call
  returns, the locale, the working directory, environment variables, standard streams in normal operation. It never
  calls `exit` or `abort` except through `panic = "abort"` after its crash log is written.
- **A library is never unloaded** by a host. A core may rely on that, and may not rely on thread-local destructors
  running at process exit.

### 6.17 Memory ownership and lifetimes

| What | Allocated by | Freed by | Lifetime |
|---|---|---|---|
| Every buffer passed to an export | the host | the host | the call |
| The machine | the core, in `create` | the core, in `free` | until `free` |
| A descriptor, a log record, a state, a picture | copied into the host's buffer | the host | the host's |
| Anything the core keeps from `create`'s params | copied by the core | the core | the machine's |

No export returns a pointer except `create`, whose machine is opaque. No pointer passed in may be kept after the call
returns. The length-query idiom makes every allocation the host's, so neither side ever frees the other's memory, and
the two sides may use different allocators.

### 6.18 Errors, logs and the crash log

- **A status** is the machine-readable answer; **`last_error`** is its detail in words for the last failing call on a
  machine; **`create`'s error buffer** carries a refusal's detail, since a refused create has no machine.
- **Logs are pulled.** A core appends records to a bounded queue; the host drains them with `log_drain` when
  `EMUSEN_EVENT_LOG` says some are waiting, and at least once a second. A record is one line,
  `level<TAB>category<TAB>text<LF>`, with `level` one of `error`, `warn`, `info`, `debug`; tabs and line breaks in the
  category or text become spaces, and text is cut at 4,096 bytes. A drain copies whole records, oldest first, as many
  as fit, removes them and returns the bytes copied; a null `out` returns the bytes waiting. A full queue (1,024
  records) drops the oldest and counts the drops, and the next drain begins with the record
  `warn<TAB>log<TAB>N records dropped`. Records made outside any call on a machine, on a core's own threads for
  instance, go to the library's queue (a null machine), and so do those a refused create or a freed machine left. A core writes nothing to standard output in normal
  operation; the runtime writes the records to the session's log under `/Logs` (§8.5).
- **The crash log** keeps `EmuSen_NativeCores.md`'s design: the host passes a path once per library, the core's panic
  hook writes the panic and its backtrace there before the process aborts, and the runtime's own crash record (today
  `EmuSen.Mistress/CrashLog.cs`) names that file.

### 6.19 Events

```c
int64_t emusen_core_events(emusen_machine *m, emusen_event *out, size_t count, size_t event_size);
```

**New.** A core queues events during any call; the host drains them after `advance`, after `state_load` and after
`set_settings`. A drain copies at most `count` events, removes them and returns how many it copied; a null `out`
returns how many are waiting; an `event_size` below 24 is `EMUSEN_BAD_STRUCT`. `GEOMETRY`, `AUDIO_RATE`,
`STATE_SIZE` and `LOG` are raised by `core_exports!` itself on observing the change, so a core need not; `BATTERY`
and `MACHINE_INFO` are the core's own to raise. The queue (256) is bounded; on overflow the core drops the oldest and queues `EMUSEN_EVENT_MACHINE_INFO`,
whose meaning ("read everything again") makes any loss safe. v1.0's kinds: `AUDIO_RATE`, `GEOMETRY`, `STATE_SIZE`,
`BATTERY` (a file changed), `LOG` and `MACHINE_INFO`. A host skips a kind it does not know. Events are how a core
tells its host anything, without a callback.

### 6.20 What libretro's callbacks give, offered without them

| A libretro core asks its frontend, by callback or environment call, to… | Here |
|---|---|
| take the frame's picture, take its samples, poll and read the input | pulled after `advance`; input set before it |
| set a pixel format | negotiated at `create` (§6.6) |
| change the geometry or the timing | `EMUSEN_EVENT_GEOMETRY`, `AUDIO_RATE`; `frame_info` each frame |
| log | the log queue (§6.18) |
| find the system directory, the save directory | never asked: the runtime passes files in at `create` (§6.2) |
| read core options, learn they changed | `create`'s settings, `set_settings`, the schema (§6.13) |
| declare input descriptors, controller info, memory maps, subsystems | descriptors (§6.3, §6.4) |
| declare serialization quirks | machine info's `state`, `skip_rendering_state_neutral`, §6.9's contract |
| get the frontend's time, a performance counter | not offered: a deterministic core needs neither; a profiling core reports phases (§6.5) |
| set rumble, read sensors, a camera, a location | later minors, as events out and setters in; not in v1.0 |
| get a hardware-rendering context | not offered; §6.22 |

### 6.21 The whole v1.0 set

| Group | Required | Optional (bit) |
|---|---|---|
| Library | `abi_version`, `capabilities`, `info`, `settings_schema`, `firmware_for`, `status_text`, `set_crash_log`, `log_drain` | — |
| Lifecycle | `create`, `free`, `machine_info`, `last_error` | `reset` (`RESET`) |
| Frame | `advance`, `set_options`, `frame_count`, `events` | `present` (`PRESENT`), `phases` (`PHASES`) |
| Picture | `frame_info`, `frame_copy` | — |
| Sound | `audio_rate`, `audio_buffered`, `audio_drain`, `set_audio_limit` | `audio_peek` (`AUDIO_PEEK`), `set_mutes` (`MUTES`) |
| Input | `set_buttons` | `set_axis` (`AXES`) |
| State | `state_size`, `state_save`, `state_load`, `state_layout` | kind 1 (`SNAPSHOT`) |
| Memory | `space_size`, `space_read`, `space_write` | — |
| Battery | `battery`, `battery_saved` | dirty flags (`BATTERY_DIRTY`) |
| Cheats | — | `set_rom_patches` (`ROM_PATCHES`), `set_cheat_pokes` (`CHEAT_POKES`) |
| Settings | — | `set_settings` (`SETTINGS`), `setting_notes` (`SETTING_NOTES`) |
| Debug | — | ten (`DEBUG`), `debug_set_stack` (`DEBUG_STACK`), `debug_registers` (`DEBUG_REGISTERS`), `debug_disassemble` (`DEBUG_DISASSEMBLE`) |

**55 names: 32 required, 23 optional** (counted in Appendix A's header). The pre-stable set names 24 required and 19
optional exports (`NativeInterface.cs`), of which the macro generates 14. The eight new required names are all trivial for a core with nothing to say (an empty JSON
list, an empty queue), and the macro writes them from the trait's defaults.

**The Rust side.** `emusen-native` gains a `core` module with the v1 trait and `core_exports!`, generated as the
pre-stable macro is: a thin wrapper per export, the handle checked, pointers turned into slices, `end_call` after
every call. The trait keeps static dispatch, so nothing hot moves (`EmuSen_NativeCores.md` §3.17). The descriptors are
written by the crate from trait methods returning Rust values, so that no core hand-writes JSON, and the crate's
writer is the one the schema test validates.

### 6.22 Doors kept open: run-ahead, netplay, achievements, hardware rendering

None of these is assumed in; §15 Q7–Q9 record when each is to be taken up. What each would need from the ABI, so that v1
does not close the door:

- **Run-ahead** runs frames ahead and rolls back each frame. It needs a cheap snapshot (`SNAPSHOT`, generated now),
  determinism (§6.9, required), a hidden frame whose skipped rendering does not change the state
  (`skip_rendering_state_neutral`, §6.4), and sound from hidden frames discarded, which the host can already do by
  draining and dropping. Nothing new in the ABI.
- **Netplay** needs determinism *across machines*: the same build on two hosts, and perhaps two platforms, must agree.
  §6.9 promises it on one host. A cross-host promise needs a conformance case run on two platforms (§12) and possibly a
  field `deterministic_across_hosts`. Rollback netplay is run-ahead's machinery plus a network.
- **Achievements** (RetroAchievements' rcheevos) read a console's memory by the console's own address map every frame.
  `space_read` serves that by copy; what is missing is the map from the console's achievement addresses to spaces.
  Machine info reserves `achievements` for it, in the shape libretro's memory-map descriptors take (a list of address
  ranges onto spaces). The hardcore mode's policy (no states, no cheats, no rewind) is the runtime's.
- **Hardware rendering** (`EmuSen_Libretro.md` §2.8) would need the core to draw into a context the host owns, which
  is a call into the host or a shared device handle. MarsRT's GPU path (`Mars_Gpu.md`) keeps its device inside the
  core and copies out the picture, which fits v1. A shared-device minor is possible later as an addition; v1.0 leaves
  `pixel_formats` open for a device-image format.

---

## 7. Core discovery and trust

### 7.1 Where cores are found

- **The project's own cores** are in `/lib/EmuSen/cores/` in a published build, under the `/lib/EmuSen` that `man hier`
  already describes as "the whole .NET application directory", and in each frontend's output directory when running
  from source.
- **Cores the player adds** are in `/Cores` in the player's home (the sandbox root of `man hier`). Nothing else is
  searched: not the ROM library, not the working directory, not a path a game or a configuration file names, not the
  system's library path.
- **Discovery reads descriptors without loading.** Each library has a sidecar, `<file>.core.json`, holding its
  `info()` and `settings_schema()` documents and the library's SHA-256. The project's build writes them, by loading
  each core once in a build step. The runtime lists systems and engines from the sidecars alone, so starting a
  frontend runs no core code (P6). A library is loaded only when a game is opened on it, and its `info()` must then
  equal its sidecar's, or it is refused as altered.
- **A library without a sidecar** is listed as "not yet read" and is read, under §7.2's rules, only when the player
  asks.

### 7.2 Trust

**What cannot be checked.** Loading a shared library runs its initialisers before any export is called. So no check
after `dlopen` can protect the process from a library that means harm; the only boundary is the decision to load it.
An in-process core can also crash the process. The crash log records it; nothing in-process prevents it.

**So the rules are about which libraries are loaded at all.**

1. **The project's own cores** are listed in a manifest shipped with the build, `cores.manifest`, with each library's
   SHA-256. A library whose hash matches is loaded without asking.
2. **A library the project did not build** is loaded only after the player approves it once. The approval shows its
   name, author, licence, the systems it claims and its hash, says plainly that a core runs with the player's full
   permissions, and records the hash. A changed file is a new library and is asked about again.
3. **Before loading**, the runtime checks, without running anything:
   - the file is a regular file inside a cores directory after resolving links, owned by the player or the
     installation, and not writable by others;
   - its header (ELF, Mach-O or PE) names this process's architecture;
   - its size is below a bound (512 MB);
   - its hash, against the manifest or the approvals.
4. **After loading**, before anything else: `emusen_core_abi_version` exists and its major is 1; every required export
   resolves; `capabilities()` and the exports agree in both directions; `info()` is valid UTF-8 JSON under 1 MiB that
   validates against the schema; its `abi` equals the version; every `host_requires` name is known. Any failure leaves
   the library loaded but never called again, with a report naming the check.
5. **Descriptor text is data.** Labels, help and names are displayed escaped and never interpreted: not as markup, not
   as a shell line, not as a path.

**What the runtime never does:** load with `RTLD_GLOBAL`; unload a library; load from a directory not listed in §7.1;
load a library that failed a check again in the same process; call into a library whose major it does not speak; pass
a pointer the core could keep; give a core a path to write to other than its crash log.

### 7.3 A possible later boundary: the broker

Because the ABI never calls the host and copies everything out, it can be carried over a pipe or shared memory to a
helper process without changing either side: a broker process loads the core, and the runtime's adapter forwards each
call. That would contain a foreign core's crash and confine its permissions. It costs a copy of every picture per
frame and a round trip per call, and nothing needs it yet. It is recorded because the pull model is what makes it
possible. §15 Q11 decided that foreign cores do not require it: they load in-process after approval, and the
broker is deferred.

---

## 8. DianaOS as the runtime

### 8.1 The role

**Decided 2026-10-01:** DianaOS is the runtime under every client and over every core. A frontend asks DianaOS for a
session; it never creates a core, holds an `ICore`, or names an engine type. The debugger and the cheats are DianaOS
services offered the same way for every core that has the capability, and they are reached the same way from the
shell, Mistress, Hotaru and Pharaoh, because all four are clients of the same runtime (§8.6).

This is continuous with what the pages record (§1.3): a core-agnostic debugger "meant to back a GUI debug window",
a standing policy that reusable tooling "goes in here", and a Unix hierarchy that is the player's home. It goes
further than they do, in one respect that changes the code: DianaOS becomes the layer that *runs* cores, where today
it is a library that the assembly holding the cores references.

### 8.2 What the runtime owns

| Responsibility | Decision |
|---|---|
| **Core discovery and loading** | §7: the cores directories, sidecars, the manifest, approvals, the checks. Replaces `NativeCoreLibrary`'s fixed crate names (§1.2, item 11) |
| **The systems table** | the console facts that belong to no engine: display names, manufacturers and years, cheat-database folders, ES-DE and ScreenScraper names, OpenVGDB's hashing rule, cover aspect, shelves, the controller diagram. Today these are in `CoreCatalog` (`CoreCatalog.cs`, the descriptors and `EsdeNames`), mixed with per-engine constants. A system entry is data with a few functions (a hashing rule); an engine's info names systems by id (§6.3) |
| **System packs** | per-console code that is not an engine's: the cheat codecs, and the debugger's console-shaped views (§6.14). Today the codecs live with each C# core (`Cores/Nintendo/*/Cheats/`), so a second engine of a console must borrow the first's. They move to `Sys/Systems/<id>/` and are keyed by system id |
| **Sessions and machines** | a session is one running machine: its engine, game, settings, ports, state and services. Several at once, each with an id (§8.5) |
| **The frame loop** | the emulation thread and the frame-end order, once: advance, the runtime's frame-end work (frame log, cheat pokes, frame notice, battery period, rewind capture), present, the picture and the sound handed off, events drained |
| **The debugger** | one set of registries per session (`Var/`, as now); the generic native debug target over §6.14's exports; a C# core's own `IDebugTarget` during the transition; stepping and halts |
| **Cheats** | the codecs (system packs), `CheatRegistry` and the database (already in `Var/`), resolution to ROM patches and pokes, application (§6.12) |
| **States and rewind** | save, load, slots and resume states through the core's state calls; `RewindBuffer` and its snapshot capture; the state's format and version recorded beside it (`IStateFormat` today) |
| **Settings** | read from the core's schema, layered system → engine → game, persisted through Galaxia's models (`GraphicsConfig`'s `Consoles`), sent to the core at create and between frames |
| **Input routing** | canonical controls from a client, routed to a port and a core bit by the descriptors; port assignment; the input latch applied at the frame boundary |
| **Firmware and battery** | `firmware_for` answered from `FirmwareLibrary`'s store; battery files read at create and written on the period or on `BATTERY_DIRTY` |
| **Speed** | fast-forward and slow motion (`SpeedController` today), rendering skipped on hidden frames |
| **Logs and crashes** | the core's drained log and the crash log, filed per session |
| **The inspection surface** | `ps`, `kill`, `/proc`, `/dev` (§8.5), and the shell |

**What stays out of the runtime, and why:**

- **Presentation**: windows, shaders and filters (Serenity), the audio device and its rate control (Endymion), physical
  key and pad bindings, controller-first navigation, the library, scraping and themes. These need a UI toolkit or a
  device, and DianaOS has none: "This keeps `EmuSen.DianaOS` — a core-agnostic library with no UI toolkit dependency
  at all — from needing to know that Avalonia, or any windowing toolkit, exists" (`EmuSen_Debugging_Tools_Reference_v5.md`
  §3.3b).
- **Telemetry** stays in Cauldron, which DianaOS already references. The 2026-08-04 split (debugger versus dashboard)
  stands.
- **The library catalogue's driver** (`SqliteCatalogue`, `EmuSen/Common/Catalogue/`) stays out, under the rule that
  "the contract may live in a leaf; the driver may not" (`EmuSen_Stack.md` §2). The runtime needs no package
  reference: loading a library, reading JSON and threading are in the base library.

**One assembly, not two.** The pruning principle asks whether a new boundary buys separability that someone uses
(`EmuSen_Multicore.md` §9.2). The runtime and the shell are used together by every client that exists; the shell
without a runtime is what DianaOS's own `Program.cs` runs and keeps. So the runtime goes into the DianaOS assembly, in
a new top-level folder, `Sys/` (§15 Q13), and no assembly is added. The C# cores stay in `EmuSen`, which already
references DianaOS, and register with the runtime as managed engines (§9.5). The reference graph does not change
direction.

### 8.3 The "bus": a call surface, an event stream and two hand-offs

The runtime sits between cores and clients, and what passes is of four kinds. Each takes the cheapest form that
serves it:

| What | Form | Why |
|---|---|---|
| **Commands**: open, pause, step, load a state, add a cheat, set a breakpoint, change a setting | **calls** on the session, queued to the frame boundary and run on the emulation thread; reads of state answered from snapshots on the caller's thread | This is the model `DianaOSInterpreterScheduler` already runs: read-only lines immediately, others drained by "only the thread that owns the core" (`EmuSen_Debugging_Tools_Reference_v5.md` §3.3a). It generalises from the shell to every client |
| **Notifications**: halted, stepped, state saved, cheat changed, engine notice, battery written, rate changed, a core's log line | **an event stream**, ordered, with any number of subscribers, each with its own bounded queue that it drains | Many readers with different speeds; a slow reader must not slow the loop |
| **The picture** | **a latest-frame slot**, lent buffers (`EmuSen_Multicore.md` §16) | One producer, several readers (the window, a recorder, a screenshot); each wants the latest, not every frame |
| **The sound** | **a ring** drained by the audio sink, with taps for recording | One consumer in real time; the rate travels with the samples (§6.7) |

**This is not a message bus.** A message bus would put frames, samples and input through one queue of typed messages.
For the picture that is a copy and an allocation per frame that the slot avoids; for the sound it is a latency the
ring avoids; for input it is a delay the latch avoids. The two ports measured layout and inlining effects of a few per
cent (`EmuSen_NativeCores.md` §8.2, R1), and the frame budget leaves no room for a dispatch layer on the hot path.
What is bus-like is the event stream, which carries what clients need to *hear about*, and the call surface, which
carries what they ask. §15 Q5 decided that "bus" means this, not a literal message bus.

### 8.4 The frame loop and pacing

- **The runtime owns the loop and the emulation thread.** Today Mistress (`MainWindow`'s emulation thread and
  `FramePacer.cs`), Hotaru (`GameWindow`) and Pharaoh (`FrameRunner`) each have their own, with their own frame-end
  order. One loop removes that.
- **Pacing is a strategy the client supplies.** When to start the next frame (against the audio clock, the display's
  refresh, a throttle, or as fast as possible) depends on the display and the audio device, which the runtime does not
  know. The runtime defines a pacer seam, a call it makes before each frame with the frame's timing, and runs
  unpaced for a headless client.
- **Where the present pacing work lands.** A frame-pacing change is being built at the time of writing in the
  frontends' shared pacing code. It stays there. When the runtime's loop lands (§10, D5), that code becomes the
  implementation of the pacer seam, in the shared presentation assembly beside the display it times against, and the
  loop around it moves into the runtime. Nothing in this page asks that work to change shape, only that its interface
  be a strategy object the loop calls.

### 8.5 The Unix mapping, tested

| Framework concept | Unix analogue | Verdict | Reason |
|---|---|---|---|
| A running session | **a process**: `ps` lists it with its id, engine, game, frame and state; `kill <id>` ends it | **Adopted** | `ps` and `kill` already list and end breakpoints, watches and shell sessions with one id space (`man ps`, `man kill`); a machine is the thing a player most wants to list and end, and several run at once |
| Pause, resume, reset | **signals**: `kill -STOP`, `kill -CONT`; `kill -s RESET` | **Adopted**, STOP and CONT as in Unix; reset as a named DianaOS signal | STOP and CONT mean exactly pause and resume. Reset has no Unix signal; overloading HUP (conventionally "reload") would mislead |
| A machine's memory | **device files**: `/dev/<session>/<space>`, so `xxd /dev/s1/WRAM`, `cmp`, `grep -c` work | **Adopted as the inspection surface**, read through `space_read` | It composes with the existing tools. A space whose reads have side effects (`side_effects`, §6.4) is refused to a plain read, as `search` refuses it (§3.1a there), and readable only with an explicit flag. Writes go through `write`, not through redirection, so a stray `>` cannot poke a running game |
| A session's status | **`/proc/<session>/`**: `status`, `machine` (the machine info), `settings`, `events` | **Adopted**, read-only text | Cheap, scriptable, `cat`-able; `tail -f /proc/s1/events` follows the event stream |
| Cores | **loadable modules**: a `cores` command listing discovered cores, their state of trust and their sidecars | **Adopted as a listing**; the "kernel driver" metaphor declined beyond it | A core is not a driver of host hardware; it is closer to a program the runtime executes. `lsmod`'s role, listing what is loaded and from where, is the useful part |
| Configuration | **`/etc/EmuSen/`**, as `man hier` describes; per-core and per-game settings as files there | **Adopted for what the player edits**, as now; declined for what the program writes | `EmuSen_Stack.md` §4 put program-written data in SQLite and config the player edits in JSON. Per-game settings layered over a system's are player-edited, so they are files: `/etc/EmuSen/games/<game id>.json` |
| Logs | **`/var/log`** | **Kept as `/Logs`** | `man hier` already places logs at `/Logs` in the home. Moving them buys nothing |
| Saves, states, cheats | **`/var`** | **Declined; kept in the home** (`/Saves`, `/Saves/Save States`, `/Cheats`) | They are the player's documents. Unix keeps an account's documents in its home and the system's state in `/var` |
| Users | **accounts**: `su`, `useradd`, `passwd` | **Exists as identity only** (`man su`); profiles later, as their own step (§15 Q4) | `su parent` and `useradd kid` read as a household. Profiles would give each account its bindings, settings overlay and perhaps saves |
| Frames, sound and input | **pipes and streams** | **Declined for the hot path**; offered as inspection | A pipe per picture adds a copy, framing and back-pressure to every frame. `cat /dev/s1/frame > shot.rgba` as a one-off is offered; the loop never uses it |
| Composition | **pipelines and scripts** | **Exists and is extended** | The interpreter has pipes, redirection, `$(...)` and control flow. Every session command (§8.6) is a shell verb, so a script can drive anything a frontend can |
| The sandbox | **chroot** | **Kept** | `DianaOSSandbox` walls the shell to the home, and `/dev` and `/proc` are virtual directories inside that root |
| Shell sessions and machines | `tmux` sessions and processes | **Kept distinct** | `tmux` sessions are shells (`Var/DianaOSSessionManager.cs`); machine sessions are processes. `ps` lists both, as it already lists both breakpoints and shells |

### 8.6 The shell as a client, and the parity test

Today the shell can do only what its host wired: it is handed an `IDebugTarget`, and loading a core is a
`HostAction.LoadCore` its host must perform. As a client of the runtime it opens, lists and ends sessions itself,
and its debugger and cheat commands call the same session services as Mistress's windows. So a breakpoint set in
Mistress shows in `ps`, a cheat added with `cheat add` appears in Mistress's Active Cheats window, and a `step` from
the shell moves the same halted session Mistress shows, because there is one runtime and one registry per session.

**The test of the principle.** Every operation of the frontend API (§9) has a shell verb, and every shell verb that
acts on a session is an operation of the frontend API. A WiseMan test enumerates the API's operations and the
interpreter's commands and checks the mapping table in both directions, so that a feature added to one surface and
not the other fails a build. Pharaoh's `--commands` scripts, which already fall through to the interpreter
(`CommandsScriptRunner.cs:19`), become ordinary shell scripts.

---

## 9. The frontend API

### 9.1 What a client sees

- **The runtime**: the systems and engines it found (JSON, as the descriptors are); opening a session from a game and
  an optional engine; the sessions running; approving a core (§7.2); the event stream.
- **A session**: run, pause, resume, step and reset; close; its info (engine, system, machine info, capabilities, run
  state); the latest picture and the sound; input as canonical controls by port; states, slots and rewind; cheats;
  breakpoints, watches, stepping, registers, memory, disassembly, the call stack and coverage; settings by key, with
  their schema; its events.
- **A shell**: open one over the runtime, submit a line, read its output; the console windows host it this way.

### 9.2 Shaped for a C interface later

`EmuSen_Stack.md` §6 records the plan to port DianaOS to Rust with "a language-neutral C interface" as its frontend
API, deferred until more cores exist. This version stays C#. So that the port does not change the API's shape, the
contract is written now as it would be written in C:

- **Handles and functions.** The runtime, a session, a subscription and a shell are opaque handles; every operation
  is a function taking a handle and plain arguments: integers, UTF-8 strings, byte spans, size-prefixed structs.
- **No language-specific constructs in the contract.** No .NET events, generics, delegates, tasks or exceptions cross
  it. A result is a status, as in the core ABI, with the detail from a `last_error` call; a list or a descriptor is
  JSON copied out; a notification is drained from a subscription, and a blocking wait with a timeout is offered, which
  C can express and which is not a callback.
- **The same ownership and threading rules as the core ABI** (§6.16, §6.17): buffers are the caller's, nothing is
  retained, any client thread may call, and mutating calls run at the frame boundary as §8.3 describes.

**In C# this is two layers.** The contract layer, `EmuSen.DianaOS.Api`, is the C-shaped surface: static functions over
handle types, returning statuses. The binding, `EmuSen.DianaOS.Client`, wraps it idiomatically: classes that dispose
their handles, exceptions built from statuses and `last_error`, a `ChannelReader` over a subscription, records parsed
from the JSON. Frontends use the binding. When the runtime becomes Rust, the contract layer becomes declarations over
a `diana.h` with the same functions, and the binding does not change. The cost now is a thin layer of forwarding; the
saving is that the port does not touch a frontend.

### 9.3 Its stability policy

The counterpart of §4, one layer up:

- **The contract layer has a major and a minor** with §4's meanings, and is guarded by a recorded baseline: the .NET
  public-API analyzer's `PublicAPI.Shipped.txt` for the contract namespace, plus a shape check that only
  C-expressible types appear in it. A removed or changed line fails a pull request; an added line must be added to the
  baseline in the same pull request.
- **Additive evolution as in §4.2**: new functions; size-prefixed structs; open enumerations; JSON fields ignored when
  unknown.
- **The binding may evolve more freely**, since it is source the frontends compile, but follows the usual .NET rules:
  no new abstract members on a public interface a client implements, `[Obsolete]` before removal, a major only on a
  break.
- **Capabilities are flags, never type probes.** Today a frontend asks `core is ISnapshotCore` and `as IFrameSerial`
  (`RewindBuffer.cs:102`, `EmulatorSession.cs`). A client of the API asks the session's capability flags, so that an
  engine's class can never decide what a frontend sees.

### 9.4 The managed side: the recommendation

**`ICore` does not become a stable public API for frontend authors.** The stable boundaries are two: the core ABI
below and the frontend API above, and both are language-neutral. `ICore`, with its optional interfaces, becomes the
runtime's **engine SPI**: the internal contract between the runtime and its two kinds of engine, the v1 adapter and
the managed engines. It must stay `public` while the C# cores in the `EmuSen` assembly implement it, so it is placed in
an `Engines` namespace, excluded from the public-API baseline, and documented as carrying no promise. When the last C#
core retires, it can become `internal` to DianaOS, and the libretro engine of §14.1, if built, implements it there.

The reasons:

- A third party writing a core should need a C compiler, not .NET; the ABI serves that.
- A third party writing a frontend should need the runtime's API, not an engine's interface; holding an `ICore` is
  exactly the per-core reach-around of `EmuSen_NativeCores.md` §1.5 that this page removes.
- Freezing `ICore` would freeze a shape built for C# cores: `GetFrameBufferRgba` returns a new array, the optional
  interfaces are found by type, and rewind needs `is` checks. The ABI already does better on each.
- Stack §6 retires C#. A C#-only stable API would be a promise the plan does not intend to keep.

### 9.5 The C# cores under DianaOS

Venus, Moon, Mercury and Mars remain until each is retired by its Rust successor's gate (`VenusRT_Plan.md` §7 for the
SNES). Until then:

- **They are managed engines**: the `EmuSen` assembly registers each with the runtime at start, with a hand-written
  registration (system id, engine name, a factory, its debug target's factory, its features). This is the one place
  where per-core registration survives, and it retires engine by engine.
- **They implement the engine SPI as today** (`ICore` and the optional interfaces) and keep their own `IDebugTarget`s.
- **They do not speak the ABI.** Exporting them through it would need Native AOT, which measured slower and broke
  the reflective state serializer (§14.8).
- **They are conformance-tested at the runtime's level only**: the kit's runtime suite (§12.2) runs through the
  frontend API over any engine, so the C# oracle and its Rust successor are graded by one suite.
- **They remain the oracles** of their ports, as `EmuSen_NativeCores.md` §7 uses them.

---

## 10. The gap list, and the order

### 10.1 Piece by piece, from the code

| Piece | Today | Destination | How |
|---|---|---|---|
| `IDebugTarget`, `IDebugMemorySpace`, the debugger's value types | `DianaOS/Lib/` | stays | — |
| The registries (`Watch`, `Breakpoint`, `CallStack`, `Coverage`, `AccessCounter`, `Freeze`, `Label`, `FrameLog`, `DmaLog`, `RegisterFlow`) | `DianaOS/Var/` | stay; owned per session by the runtime | — |
| The cheat engine (`CheatRegistry`, `CheatDatabase` and its installer, pruner and importer, `ChtFile`, `CheatWrite`), `ICheatCodeCodec`, `DelegateCheatCodec` | `DianaOS/Var/`, `Lib/` | stay | — |
| The shell, its commands and sessions, the scheduler | `DianaOS/Bin/`, `Var/` | stay; `HostAction.LoadCore` replaced by a runtime call | revise |
| Per-console cheat codecs (`VenusCheatCodecs`, `MoonCheatCodecs`, `MercuryCheatCodecs`, `MarsCheatCodecs`) | `EmuSen/Cores/Nintendo/*/Cheats/` | system packs, `DianaOS/Sys/Systems/<id>/`; the SNES's written fresh, because `VenusRT_Plan.md` §9 Q5 excluded Venus's by provenance | move, and one rewrite |
| `ICore`, `CoreCapabilities.cs`, `CoreOptions`, `CoreHooks`, `FrameBufferLending`, `ICheatRegistryHost` | `EmuSen/Cores/` | `DianaOS/Sys/Engines/` as the engine SPI (§9.4) | move; the C# cores already reference DianaOS |
| `CheatRomPatcher` | `EmuSen/Cores/` | stays with the C# cores (only Mars, Mercury and Moon use it) | retires with them |
| `BatterySave`, `RewindBuffer`, `RewindThumbnail`, `SpeedController`, `FirmwareLibrary` | `EmuSen/Cores/`, `EmuSen/Common/` | runtime services | move |
| `CoreCatalog` | `EmuSen/Cores/` | split: console facts to the systems table; engines from discovery and managed registrations; settings rows from schemas | replace |
| `CoreFactory`, `CoreBundle` | `EmuSen/Cores/` | the runtime's session open | replace |
| `EmulatorSession` | `EmuSen/Common/` | the session | replace |
| The native host (`NativeCoreLibrary`, `NativeInterface`, `NativeMachine`, `NativeRtCore`, `NativeDebugBridge`) | `EmuSen/Cores/Native/` | the v1 adapter and loader in `DianaOS/Sys/Native/`; the pre-stable classes retire (§13.5) | move and replace |
| `LegacyNativeMachine` | `EmuSen/Cores/Native/` | deleted with MarsRT's step 5 | retire |
| Per-console native shims (`MoonRtCore`, `MercuryRtCore`, `MarsRtCore`; VenusRT's `VenusNative`, `VenusMachine`) | each core's `Shim/` | none for VenusRT; thin oracle-compatibility subclasses of the adapter for the ports, until their C# cores retire (MoonRT's and MercuryRT's done 2026-10-04, §26) | shrink |
| `AudioSettings`, `GraphicsSettings` (static configuration) | `EmuSen` | values handed to the runtime from Galaxia's models | behind the API |
| The library catalogue (`SqliteCatalogue`) | `EmuSen/Common/Catalogue/` | stays out of DianaOS | behind Galaxia's `ICatalogue` |
| Frame loops and pacing | Mistress, Hotaru, Pharaoh | the runtime's loop; pacing behind the seam (§8.4) | replace |
| Port routing and button bits | `NativeRtCore.ButtonBit`, each C# core's `SetButton` | the runtime, from descriptors | move |
| The settings window's rows | `CoreCatalog.SettingsFor` | the runtime, from schemas | replace |
| The crash record | `EmuSen.Mistress/CrashLog.cs` | split: the runtime records core faults and halts; the frontend keeps its own | split |
| Telemetry | Cauldron | stays | — |
| Physical bindings, presentation, library, scraping, themes | the frontends, Serenity, Endymion | stay | — |

### 10.2 The order

| Step | What | Oracle | Effort |
|---|---|---|---|
| **D1** | The move: the engine SPI, the native host, the services and the systems table into `DianaOS/Sys/`; the C# cores registered as managed engines; `CoreFactory` and `CoreCatalog` answered from the runtime | The registration-equivalence test `EmuSen_NativeCores.md` §7 step 1 named: every answer `CoreCatalog` and `CoreFactory` give, for every console and engine, before and after; built 2026-10-04 ahead of the move (§25). The whole suite | 2–3 days |
| **D2** | Sessions, the loop and the services in the runtime; Pharaoh moved onto them first, as the headless client; then the shell (its `LoadCore` gone), Hotaru, Mistress | Each client's own tests; Pharaoh's `framesum` and `audiosum` digests equal before and after on the bench games; Mistress's rewind and state tests | 1½–2 weeks |
| **D3** | The frontend API's contract layer and binding; its baseline; the parity test of §8.6 | The parity test; the public-API baseline recorded | 3–4 days |
| **D4** | The inspection surface: sessions in `ps` and `kill`, signals, `/proc`, `/dev`, `cores` | Shell tests per verb; a side-effect space refused | 2 days |
| **D5** | The pacer seam, around the pacing work in progress once it lands | Mistress's and Hotaru's pacing tests unchanged | 1 day |

**D1 to D5 need not precede VenusRT's stage 6** (§13.2). They are ordered so that each leaves every client working,
and the clients move one at a time.

---

## 11. The framework services catalogue

Every feature of RetroArch and ES-DE the platform means to fold in, the layer that owns it, whether it exists, and the
gap. "ABI" is the core ABI, "Runtime" is DianaOS, "Frontend" is a client or the shared presentation assemblies.

### 11.1 RetroArch's side

| Feature | Layer | Today | Gap |
|---|---|---|---|
| Rewind | Runtime; ABI `SNAPSHOT` | `EmuSen/Common/RewindBuffer.cs`, driven by Mistress; snapshots only from MarsRT's legacy ABI | generated snapshot (§6.9); the service in the runtime |
| Save states, slots, resume | Runtime; ABI state | `ICore.SaveState`; Mistress's resume states and `records.db` | the service in the runtime; format and version from machine info |
| Shaders and filters | Frontend | Serenity's SkSL chain and slang runtime (`EmuSen.Serenity/Slang/SlangChain.cs`) | none at the ABI; aspect and geometry from `frame_info` |
| Input bindings and remapping | Frontend binds; Runtime routes | per-console bindings in Mistress (`Input/ControllerKeyBindings.cs`) and its rebind window | routing from descriptors; per-game remaps not found in the code |
| Cheats | Runtime; ABI patches and pokes | DianaOS's engine; codecs per C# core; patches on the pre-stable ABI | codecs as system packs; `CHEAT_POKES` |
| The debugger | Runtime; ABI debug | DianaOS's registries and shell; each core's own target; MoonRT's and MercuryRT's bridge | the generic native target with registers and disassembly (§6.14) |
| Frame pacing and sync | Frontend strategy; Runtime loop | `EmuSen.Mistress/FramePacer.cs`, Hotaru's loop; a change in progress | the seam (§8.4) |
| Audio sync and resampling | Frontend | Endymion's `DynamicRateControl` and `AudioPlayer` | the drain's rate (§6.7) |
| Fast-forward, slow motion | Runtime | `EmuSen/Common/SpeedController.cs` | into the runtime |
| Run-ahead | Runtime | not present | §6.22; §15 Q7 |
| Netplay | Runtime and network | not present | §6.22; §15 Q8 |
| Achievements | Runtime | not present | §6.22; §15 Q9 |
| Overlays and bezels | Frontend | slang presets in the shader window (`ShaderSettingsWindow.cs`) | none at the ABI |
| Per-game settings overrides | Runtime | the per-game engine choice only (`MainWindow.GameOptions.cs`, ES-DE's "alternative emulator") | layered settings (§8.2) |
| Core options | ABI schema; Runtime; Frontend presents | `ICoreSettings` and `CoreCatalog.SettingsFor`, hand-written per console | the schema (§6.13) |
| The core list, core info, "load core" | ABI info; Runtime discovery | `CoreCatalog`, hand-written | §6.3, §7 |
| Firmware management | ABI `firmware_for`; Runtime | `ICore.GetFirmwareRequirements`, `FirmwareLibrary`, Mistress's prompt | the export |
| Recording and screenshots | Runtime records; Frontend saves | `DianaOS/Var/FrameRecorder.cs`, Pharaoh's `record`, the frontends' screenshots | none |
| Logs | Runtime | per-frontend log writers; no core log | the drained log (§6.18) |

### 11.2 ES-DE's side

| Feature | Layer | Today | Gap |
|---|---|---|---|
| The library | Frontend; Galaxia's `ICatalogue` | Mistress's library over `SqliteCatalogue` | systems and extensions from the runtime, not `CoreCatalog` |
| Scraping (ScreenScraper) | Frontend | `EmuSen.Mistress/Scraping/`, `Library/GameMetadata.cs` | ScreenScraper's system ids in the systems table |
| Themes | Frontend | `EmuSen.Mistress/BigPicture/Theme/ThemeLoader.cs`, Art Book Next | ES-DE system names in the systems table (today `CoreCatalog.EsdeNames`) |
| Collections | Frontend | Mistress's `games.db` | none |
| Metadata editing | Frontend | `Library/MetadataDraft.cs` | none |
| Controller-first navigation | Frontend | `Input/PadWindowRouter.cs`, the Game Mode sheets | none |
| Alternative emulator per game | Runtime; Frontend | `MainWindow.GameOptions.cs` | engines from discovery |

The catalogue's lesson: what the ABI must carry is small (descriptors, snapshot, determinism, firmware, the drain's
rate), and what the platform lacks is mostly the runtime's ownership of services that already exist in the frontends.

---

## 12. The conformance kit

### 12.1 What makes a core "EmuSen v1 compliant"

Passing every mandatory case of the kit whose version matches the minor the core claims, on an image the core's author
supplies. The cases:

| Case | Checks |
|---|---|
| **C1, loading** | the library loads; `abi_version`'s major is 1; every required export resolves; no `emusen_core_` symbol outside the baseline |
| **C2, info** | `info()` validates against the schema; `abi` equals the version; `capabilities` names equal the bits; every claimed optional export exists and every unclaimed one does not |
| **C3, the schema** | the settings schema validates; keys are unique; every default lies in its own domain; **an `accuracy` setting's default is its `accurate` value and an `enhancement`'s is its `hardware` value**; every trade-off has its `cost` |
| **C4, create and refusal** | **a malformed image never harms the core** (decided 2026-10-03): each of an empty, a garbage and a truncated image is either refused with null, a documented status and an error text, leaving nothing to free, or accepted and run for the kit's frames without crashing, hanging or corrupting the host. Refusal is required only where the system pack declares its image format self-delimiting (it records its own length, or a checksum the core must verify); the report says of each image whether it was refused or accepted and ran; **a machine created with no settings equals one created with every default stated, over N frames** |
| **C5, capability honesty** | every claimed capability works on a running machine; every unclaimed one is refused with `NOT_SUPPORTED` or absent; an unknown kind, space, port, axis, format or option bit is refused or ignored as the header says |
| **C6, the frame** | N frames run; `frame_info` agrees with `frame_copy`'s length and the format is one offered; a size change is preceded by `GEOMETRY`; samples drain at the rates the drain reports, with an `AUDIO_RATE` event at each change |
| **C7, state** | save, load, save is byte-identical; a kind's size is constant over N frames except after a `STATE_SIZE` event; a truncated, foreign or random state is refused and the machine's state hash is unchanged; a state saved on machine A and loaded on machine B makes B continue exactly as A; the snapshot kind, if claimed, loads |
| **C8, determinism** | two machines fed the same image, settings, files and scripted input give identical picture, sound and state digests over N frames; again across a save and load at frame N/2; and, where `skip_rendering_state_neutral` is claimed, with rendering skipped on alternate frames |
| **C9, exact settings** | for every setting with `effect: exact`, every value (each choice; a count's bounds and middle) gives digests identical to the default's |
| **C10, isolation** | two machines on two threads, interleaved, each equal to its solo run; one freed while the other runs |
| **C11, error paths** | a null machine gives −1 everywhere; null buffers; the length query; a short buffer; a struct below its v1.0 size gives `BAD_STRUCT`; `status_text` answers for every code the core returned |
| **C12, skew** | the kit acts as a host at minor 0 with v1.0's struct sizes, and as a host at a minor far ahead with larger structs whose extra bytes hold a canary the core must leave untouched; both runs pass C6–C8 |
| **C13, library-level concurrency** | the library-level calls from several threads while a machine runs |
| **C14, descriptors against exports** | each space's size equals `space_size`, a `read_only` space refuses a write, each battery file's length equals `battery`'s, each processor named answers `debug_pc` when `DEBUG` is claimed |
| **C15, debug** (if claimed) | every table armed with nothing to hit gives the plain run's digests; a breakpoint at the first instruction stops before it; `EACH` stops at every step; a halt reports its processor |

The digests are those Pharaoh's `framesum` and `audiosum` already define: an FNV-1a fold of each frame's hash
(`CommandsScriptRunner.cs:533`) and its audio counterpart (`:552`), so that a kit result and a Pharaoh result can be
compared directly.

### 12.2 Where it lives, and Pharaoh

**Recommended: a new stand-alone runner for the core suite, and Pharaoh refactored onto the frontend API as the
runtime suite's client.**

- **The core suite, `emusen-core-conform`,** is a Rust program in `EmuSen/Cores/Shared/emusen-core-conform/`. It loads
  a library itself, through the ABI and nothing else. It must test the core as the header specifies it, not as the C#
  adapter happens to call it, and an outside author must be able to run it with a C toolchain and cargo, without .NET.
  The project already has the precedent of a Rust headless runner that drives any libretro core (`Reference/probe-rs/`,
  `--backend`).
- **It absorbs Pharaoh's core-agnostic pieces as definitions, not as code:** the hold, release and tap input script,
  the frame cap, the two digests and `waitstable`'s stability rule are specified in the kit's text and implemented in
  Rust to the same definitions.
- **Pharaoh** becomes what §8 makes it, a headless client of DianaOS (step D2), and gains the kit's **runtime suite**:
  C4, C6–C10 and C15 again, but through the frontend API over any engine, including the C# cores (§9.5). That grades
  the runtime and the v1 adapter, and lets a C# oracle and its Rust successor be compared by one suite.
- **What does not move:** Pharaoh's Venus-typed traces (`CpuBinaryTrace`, `GsuBinaryTrace`), the 65816 and SPC700
  single-step targets, the peer probe and the library indexer are not part of either suite. The Venus-only parts retire
  with C# Venus or are given VenusRT equivalents, as `VenusRT_Plan.md` §4.8 already says.

Making Pharaoh itself the core runner was weighed and not chosen: it would test cores through the C# adapter, it needs
.NET, and its loading goes through `CoreFactory`'s extension switch, under which `.sfc` always means C# Venus
(`CoreFactory.cs:43`).

### 12.3 How a third party runs it

```
cargo install --path EmuSen/Cores/Shared/emusen-core-conform        # or a released binary
emusen-core-conform --core ./libmycore.so --image game.bin --frames 1200 \
                    [--input script.txt] [--settings key=value ...] --report report.json
```

The exit status is 0 when every mandatory case passes. The report lists each case with its verdict and evidence, the
kit's version and the core's `abi`, `id` and `version`. A core is described as "EmuSen v1.N compliant" with the report
that showed it.

**Images.** The kit runs on images the author supplies; it ships none. The project's own CI uses images it builds
(`EmuSen.WiseMan`'s `SyntheticRom` for the SNES, and builders of the same kind for the other systems where they exist
or are added); commercial ROMs are never committed, and a local run may point the kit at the library in
`AppSettings.RomDirectory`, which it reads and never writes.

### 12.4 How the kit checks that defaults are the accurate configuration

The kit cannot judge accuracy; it checks the three claims that make "accurate by default" mean something:

1. **The declaration** (C3): every `accuracy` setting defaults to its `accurate` value and every `enhancement` to its
   `hardware` value.
2. **The defaults are what runs** (C4): a machine made with no settings equals one made with every default stated, so
   no hidden configuration differs from the schema's.
3. **The record was taken at the defaults**: info's `accuracy.measured_with` is `"defaults"`, and for the project's own
   cores the accuracy suites (the test-ROM runner generic over engines, `VenusRT_Plan.md` §3.3; the single-step suites;
   each port's oracle) run with no settings. The kit checks the field; the project's CI checks the practice.

C9 adds the converse for speed: a setting that claims to cost nothing is shown to cost nothing.

### 12.5 Skew fixtures for hosts

Two cores built from the kit's crate test a *host*: `conform-oldest`, which claims minor 0, exports only the required
set and writes v1.0's struct sizes; and `conform-future`, which claims a far minor, unknown capability bits, unknown
event kinds, unknown descriptor fields and larger structs. A host passes when it runs both, ignores what it does not
know and reads only what was written. WiseMan runs both against the v1 adapter on every change to it.

---

## 13. Migration

### 13.1 Each core onto v1

| Core | From | Path | Oracle | Effort |
|---|---|---|---|---|
| **VenusRT** | the pre-stable set, capabilities 0, early (stage 3) | switched to `core_exports!` as soon as it exists; its descriptors written as its stages add hardware; no `VenusRtCore` class at stage 6 | its own stage oracles unchanged; the kit's core suite from then on | half a day at the switch, then part of each stage |
| **MoonRT** | the pre-stable set, version `0x0001_0004` | regenerated with `core_exports!`; descriptors from `MoonMachine`'s tables; `MoonRtCore` becomes a thin subclass of the adapter keeping the C# exception types, the state pre-check messages and the mirror debugger | `MoonRtStateTests`, `MoonRtMachineTests` and the engine tests unchanged (P8 of `EmuSen_NativeCores.md`); the bench's state hashes; the plain frame within ±1% interleaved (P1); the kit | 1 day |
| **MercuryRT** | the pre-stable set, `0x0001_0005` | the same; `Model` becomes a create-time schema key, the six `mercuryrt_*` extensions stay | the 586 WiseMan tests §12.5 there names; its digest over 24 runs; the kit; the plain frame timed interleaved, aligned build kept | 1 day |
| **MarsRT** | its legacy ABI, interface 10 | **directly onto v1 at its step 5**, skipping the pre-stable set: the snapshot, `PRESENT`, `PHASES`, `AXES`, `SETTINGS`, `FRAME_SERIAL`, `ROW_REPEAT`, `BATTERY_DIRTY`, `CHEAT_POKES`; its settings from the schema; its extensions renamed | step 5's list (`EmuSen_NativeCores.md` §7: the 57 test methods, rewind per landing, `pacebench`'s hash, the three gameplay states, the probe on Super Mario 64 and Ocarina of Time), R3's held-button state test, the kit; then 5a (PGO retrained) and 5b (timed) | 2–3 days, plus 5a and 5b; **deferred with its retrain until the port is finished** |

### 13.2 What must land before VenusRT's stage 6

Stage 6 is "Frontends on the common interface" (`VenusRT_Plan.md` §6), planned as `VenusRtCore : NativeRtCore<VenusMachine>`
with the SNES's console part in C#. **This specification replaces that plan: VenusRT's stage 6 is to add no per-core
C# class.** For that, before stage 6 begins:

1. **The header, the `core` module and `core_exports!`**, including the generated snapshot, `present`, `phases`,
   `set_axis`, `set_settings`, `setting_notes`, the events queue, the log queue and the three descriptor writers.
   *About 2 days.*
2. **The guard**: §5.2's Rust check, §5.3's C# check, §5.4's baseline and the `nm` check in CI. VenusRT is then the
   first core born under the guard. *About 1 day.*
3. **The v1 adapter and loader in the host**: major and minor read, the descriptors parsed, one generic engine class
   over any v1 core, implementing every optional interface of the engine SPI with a neutral answer where the core lacks
   the capability (an `ISnapshotCore` that falls back to the full state, an `IFrameSerial` that reports the frame
   count, an empty `ICoreSettings`), so that what a frontend sees equals what it saw without the interface. *About 2–3
   days.* It sits in `EmuSen/Cores/Native/` until D1 moves it.
4. **Discovery enough for an engine row**: the sidecars, and `CoreFactory`/`CoreCatalog` answering a v1 engine of a
   known system from its info, with one generic branch in place of a per-engine one. The SNES's row then lists Venus
   (C#) and VenusRT without code naming VenusRT. *About 1 day.*
5. **The SNES's system pack**: its systems-table entry (already `CoreCatalog`'s data) and the two codecs written fresh
   (Pro Action Replay, Game Genie), keyed by system id. *About 1 day.* (`VenusRT_Plan.md` §9 Q5 placed the rewrite in
   VenusRT's shim; §15 Q14 decided to place it here instead.)
6. **The kit's core suite, at least C1–C8 and C10–C11**, which becomes part of stage 6's oracle. *About 2 days.*

**In all, about 9–10 days**, against a stage 6 that `VenusRT_Plan.md` §6 recommends running after stage 4 (five steps
away). The DianaOS moves (D1–D5), MoonRT's and MercuryRT's moves and MarsRT's step 5 need not precede it.

### 13.3 The order

1. Decisions: §15 answered, 2026-10-03, every question as recommended.
2. §13.2's items 1–2, then VenusRT's switch.
3. §13.2's items 3–6.
4. MoonRT, then MercuryRT, onto v1 (each with its timing, since MercuryRT measured layout sensitivity before).
5. VenusRT's stage 6, on the adapter.
6. D1–D5 (§10.2), clients one at a time.
7. MarsRT's step 5 onto v1, with 5a and 5b, when its port is finished.
8. The pre-stable set and the legacy path deleted (§13.5).

### 13.4 How the managed console parts shrink

| Console part of `EmuSen_NativeCores.md` §4.5 | Where it goes |
|---|---|
| Library name, switch variable | discovery and the sidecar; the switch variables stay as an override |
| Create settings | the schema's `create` keys |
| Battery files | machine info's `battery`; the path rule stays the runtime's |
| Buttons, axes | the descriptors; axis scaling into the core (§6.8) |
| Spaces | machine info |
| Status table | `status_text`; the oracle's exception types stay in the port's shim |
| State pre-checks | the core's own refusals, with `last_error`; a port keeps its oracle's messages in its shim |
| Frame-end order | the runtime's one order (§8.2), with a port's shim overriding it where its oracle's differs |
| Patch limit | machine info's `patches` |
| Settings | the schema |
| Debugger view | the generic native target; the mirrors stay in the ports' shims until their C# cores retire |
| Extras | extensions, used by the one console's system pack |

### 13.5 The legacy paths retire

- **MarsRT's legacy ABI** (`LegacyNativeMachine`, `LegacyNativeExports`, `NativeCoreLibrary`'s per-core constructor) is
  deleted at MarsRT's step 5, its last user, as `EmuSen_NativeCores.md` §12.5 already plans.
- **The pre-stable `emusen_native_*` set** (`NativeCoreLibrary.Common`, `NativeInterface`, `NativeMachine`,
  `NativeRtCore`, the `abi` module's macro) is deleted when MoonRT, MercuryRT and VenusRT are on v1. Nothing outside the
  repository calls it, so it needs no deprecation period. The `debug` module's `Hooks` stay; they are data the v1 macro
  uses.
- **`rust-cores.yml`'s `symbol` column** is replaced by the `nm` check of §5.4.

---

## 14. Rejected alternatives

### 14.1 Adopting libretro itself as the ABI

This is a real option and is weighed as one.

**For it.**
- It exists, is documented, and is what an outside core author already knows. Hundreds of cores speak it, and
  RetroArch and other frontends would host EmuSen's cores with no adapter.
- It already carries much of what §6 adds: system info, AV info, core options with categories and descriptions,
  input descriptors and controller info, memory maps for achievements, serialisation quirks, hardware rendering.
  `EmuSen_NativeCores.md` §6 credited it less than that.
- Its licence (MIT) is no obstacle.

**Against it, as EmuSen's own contract.**
- **One instance per library.** Several machines in one process need a copy of the library file each, as RetroArch's
  second-instance run-ahead does. The tests, the side-by-side engine proofs and the kit's isolation and determinism
  cases (C8, C10) run several at once.
- **Callbacks during the frame.** The core calls the frontend for video, audio, input and everything else while it is
  inside `retro_run`. For a managed host each is a reverse transition, a managed exception escaping one is fatal to the
  process, and the host's code runs on the core's stack in the middle of a frame. The cost in time is small (argued:
  tens of calls a frame); the cost in rules is the point. It also leaves no place for MarsRT's C# work between
  `advance` and `present`.
- **The environment call is untyped**: a command number and a pointer, support discovered by calling. That is the
  property this ABI exists to remove: typed exports, checked at load, visible to `nm`, guarded by a baseline.
- **Memory by pointer** would bypass MarsRT's drain wait (§2.3).
- **No debugger.** EmuSen's research depends on one, and DianaOS's services are built on it.
- **Cheats as strings** the core parses would put every codec in every core.

**So libretro is supported at the boundary, in both directions, rather than adopted.** Outbound, `emusen-libretro`
(`EmuSen_NativeCores.md` §6.3, `EmuSen_Libretro.md` §7) turns any v1 core into a libretro core; v1's descriptors are
chosen so that the adapter fills `retro_system_info`, the options and the input descriptors from them with no per-core
code, which is a design test of this page (P7). Inbound, a libretro engine under DianaOS could let a player run any
libretro core inside Mistress, with what libretro carries and without the debugger. That is how EmuSen would fold in
RetroArch's library of cores, and §15 Q6 decided it in principle, as its own plan after D1.

### 14.2 Keeping exact matching

§4.8. It enforces the churn it was meant to contain, and forbids an outside core.

### 14.3 A managed-only API

`ICore` as the only contract, with Rust cores behind per-core shims. It brings back the shim per core that the native
interface removed, requires .NET of an outside author, cannot carry the libretro adapter, and contradicts
`EmuSen_Stack.md` §6.

### 14.4 A function-table export

`emusen_core_api(version) -> const table*`, as some plug-in systems do. It would allow static linking (R5 of
`EmuSen_NativeCores.md`) and one lookup. It is still rejected: the table's layout needs its own versioning, `nm` stops
showing what a library offers, optional entries still need bits, and the baseline check would read a struct instead
of symbols. It remains a cheap later addition if a platform without dynamic loading is ever targeted.

### 14.5 Generating one artefact from another

- **The header from the Rust (cbindgen).** It makes the Rust the specification, and a generated header carries no
  normative prose.
- **The Rust from the header (bindgen in every core's build).** It puts libclang into every core's build. §5.2 uses
  bindgen in one CI-only crate instead.
- **All three from an interface description language.** A fourth artefact and a generator to maintain, for 55
  functions. To be reconsidered only if §5's checks prove costly in practice.

### 14.6 Binary descriptors

A struct per descriptor, as libretro's system info is. Each would need its own size and versioning and its own
three-language layout check, and lists of strings and nested lists would need pointers into the core's memory. JSON
read once is cheaper to evolve and to check (§6.3).

### 14.7 A message bus, and streams on the hot path

§8.3 and §8.5: a copy and a dispatch per frame for nothing a client needs.

### 14.8 The C# cores through the ABI by Native AOT

Native AOT ran 10 to 12 per cent slower than the JIT on Venus, and about a fifth slower on Mars's interpreter, where
its save state came out at 48 bytes because the reflection `StateSerializer` walks finds nothing
(`Mars_Performance.md` §25, citing `Venus_CPU.md` §8). Either result rules it out. The C# cores stay managed engines
until they retire.

### 14.9 Out-of-process cores by default

§7.3. A copy of every picture and a round trip per call, for a protection only foreign cores might need.

### 14.10 A new assembly for the runtime

§8.2. It would buy separability nobody uses, against the pruning rule.

---

## 15. Questions, decided 2026-10-03

Each question as it was put, and its decision. Every recommendation this page made was accepted as written, and
nothing was changed.

1. **Q1, the prefix.** Stable exports `emusen_core_*`, types `emusen_*`, beside the pre-stable `emusen_native_*` until
   that set is deleted? Decided: yes. A library's two possible interfaces then never meet in a loader, the baseline
   starts clean, and the header's name matches its exports. The cost is the macro, a CI column and the C# table, all of
   which change anyway.
2. **Q2, descriptors as JSON.** Decided: yes (§6.3, §14.6).
3. **Q3, is a session a process?** Listed by `ps`, ended by `kill`, paused and resumed by `kill -STOP` and `-CONT`.
   Decided: yes, with reset as the named signal `RESET` (§8.5).
4. **Q4, are users real profiles?** Today accounts are identity only (`man su`). Decided: profiles later, as their own
   step: each account with its bindings, its settings overlay and its own saves folder, `su` switching all three. Until
   then the runtime keys nothing on the account, so the step adds and changes nothing.
5. **Q5, what "bus" means.** A literal message bus, or §8.3's call surface, event stream and two hand-offs? Decided:
   §8.3.
6. **Q6, a libretro engine inbound.** Should DianaOS host libretro cores as engines, so a player can use them inside
   Mistress? Decided: yes in principle, after v1's adapter and the runtime's move (D1), and as its own plan; the engine
   SPI is shaped so it fits.
7. **Q7, run-ahead.** Decided: adopted after VenusRT becomes the SNES default, since VenusRT designs its skipped
   rendering to be state-neutral; the ABI needs nothing more (§6.22).
8. **Q8, netplay.** Decided: not before run-ahead; first a cross-host determinism case is added to the kit and run on
   two platforms.
9. **Q9, achievements.** Decided: together with Q6, since rcheevos' integration and its console memory maps are what
   libretro's frontends already carry; the ABI reserves `achievements` in machine info.
10. **Q10, console-shaped debug views in the ABI.** Sprites, palettes, tiles, DMA. Decided: not in v1.0. They stay
    system packs and extensions, and enter a later minor only if a second core of a console needs the same view.
11. **Q11, foreign cores.** Loadable at all, and if so in-process with a one-time approval by hash, or only through a
    broker (§7.3)? Decided: in-process with approval; the broker is deferred.
12. **Q12, latency and the accurate default.** Should `latency` settings also default to the lowest latency? MarsCore's
    `DeferredPresentation` defaults to on, a frame late. Decided: the rule covers `accuracy` and `enhancement` only; a
    `latency` setting's default is its author's choice, shown with its cost; MarsRT's default is reviewed at its step 5.
13. **Q13, the runtime's folder.** `DianaOS/Sys/`, beside `Bin`, `Lib`, `Var`, `Etc` and `Dev`? Decided: yes. `/sys` is
    where Unix exposes devices and drivers to the system, which is the runtime's subject.
14. **Q14, where the SNES's new codecs live.** `VenusRT_Plan.md` §9 Q5 put the rewrite in VenusRT's shim. Decided: in
    the SNES's system pack instead, written fresh as decided there, so that any SNES engine has them and the shim is not
    needed at all. `VenusRT_Plan.md` §9 Q5 records the change.
15. **Q15, the axis value.** A `double` normalised to −1..1, the core scaling (§6.8)? Decided: yes.
16. **Q16, MarsRT straight to v1.** Skipping the pre-stable set at its step 5? Decided: yes, so it moves once, with one
    retrain.
17. **Q17, publishing the frontend API.** Package `EmuSen.DianaOS` for outside frontends now, as LunaP is packed for
    Pegasus? Decided: the stability policy applies now (the baseline, §9.3), and the package is published once D3 has
    landed.
18. **Q18, a `Dev/` for machines.** `Dev/` in the source tree holds the console line reader. Is the hierarchy's `/dev`
    meant for machine devices, as §8.5 proposes for the virtual tree? Decided: yes in the virtual tree; the source
    folder keeps its meaning (the host's devices) and is not renamed.

---

## 16. Predictions to be retired

| # | Prediction | Retired when |
|---|---|---|
| P1 | MoonRT's plain frame through `core_exports!` and the v1 adapter is within ±1% of the pre-stable host's, interleaved, as `EmuSen_NativeCores.md` §12.3 measured the last move | MoonRT's move |
| P2 | VenusRT's stage 6 adds no per-core C# class: the SNES needs its system pack and nothing else | stage 6 |
| P3 | §5.2's check fails on a seeded signature change and a seeded swap of two same-sized fields; §5.3's fails on today's `set_crash_log` type until it is corrected | **Retired, held, 2026-10-03** (§18.4) |
| P4 | MoonRT and MercuryRT pass the kit's core suite after their moves with no change to their machines' Rust beyond the trait's new methods | each move |
| P5 | A core built at minor 0 runs unchanged on a host built after the first addition (`conform-oldest`) | the first minor |
| P6 | Starting a frontend opens no core library: discovery reads only sidecars (checked with `strace`) | §13.2 item 4 |
| P7 | The libretro adapter fills system info, options and input descriptors from v1's descriptors with no per-core code | the adapter, if built |
| P8 | Reading and parsing a core's three descriptors costs under 5 ms per load | the adapter |
| P9 | C9 finds no `exact` setting in MarsRT that changes its digests | MarsRT's step 5 |

---

## 17. Changelog of the ABI, and what was not checked

**Changelog.** v1.0: this page; the header, the baseline and the reference macro committed 2026-10-03 (§18), with the
host adapter, discovery, the SNES's system pack and the kit's core suite after them (§19–§21). No minor has been
released.

**Not checked here.**
- Layouts were checked on x86-64 Linux only, with gcc 16 and clang. The baseline's other three targets are argued from
  the C rules the header keeps (fixed-width types, natural alignment, no bit fields, no packing).
- The public-API analyzer was not tried on the DianaOS project.
- The descriptors' parsing cost was not measured (P8).
- Whether an `UnmanagedCallersOnly` is needed anywhere: by construction it is not, since nothing calls back.
- The pages were searched for DianaOS's original intent (the manual, `Old/`, the man pages); the runtime role is not
  recorded before this page (§1.3).

---

## 18. What was built, 2026-10-03: §13.2's items 1 and 2

### 18.1 The artefacts

| Artefact | Where | What it is |
|---|---|---|
| The header | `EmuSen/Cores/Shared/emusen-native/include/emusen_core.h` | 55 exports, 32 required and 23 optional, each with its function type; the structs of Appendix A; the status codes, capability bits, flags and debug constants; the open enumerations. It compiles as C99 under gcc with `-Wall -Wextra -pedantic -Werror`, as C11 under clang and as C++17, with the layouts of §5.1 asserted (**measured**) |
| The `core` module | `emusen-native/src/core/` | `sys` (the header's structs and constants in Rust), `desc` (the descriptors as Rust values and the crate's JSON writers), `outbox` (events, logs, the last error), `schema` (a validator for the subset of JSON Schema the schemas use), `exports` (the generic body of every export) and `mod.rs` (the `Core` trait and `core_exports!`) |
| The schemas | `emusen-native/abi/v1/*.schema.json` | info, machine info, settings, firmware, setting notes and disassembly, as JSON Schema 2020-12 |
| The baseline | `emusen-native/abi/v1/baseline.txt` | 373 facts: 55 `fn`, 55 `export`, 4 `struct`, 32 `field`, 47 `const`, 18 `status`, 33 `enum` and 129 `json` lines, all `@1.0` |
| The test core | `emusen-native/examples/v1_test_core.rs` | a counter, built as a `cdylib`, that claims all 18 capability bits so that the macro generates every export |
| The Rust check | `EmuSen/Cores/Shared/emusen-core-abi-check/` | §5.2 and §5.4, run in CI only |
| The export check | `emusen-native/abi/v1/export_check.py` | §5.4's item 5, on any built library |
| The C# table and check | `EmuSen/Cores/Native/CoreInterface.cs`, `EmuSen.WiseMan/Cores/CoreAbiTests.cs` | §5.3 |

Nothing that runs today changed. MoonRT, MercuryRT, VenusRT and MarsRT still export the pre-stable set; the `abi`
module gained two `Settings` methods and nothing else; the C# host's loader is untouched.

### 18.2 How `core_exports!` is arranged

**One generic body per export.** The macro writes, for each export, a `#[no_mangle] extern "C"` function of the
header's signature whose body is one call to `emusen_native::core::exports::<name>::<T>`. Each body is generic over
the core, so dispatch stays static as §6.21 requires, and the macro itself holds no logic that a reader must expand to
review. The optional groups are named at the invocation (`core_exports!(Machine; reset, settings, debug)`), and a
compile-time assertion requires them to equal the claimed bits that have exports, and forbids bits 18 to 47. The
argument against generating optional exports from `CAPABILITIES` alone is mechanical: a `#[no_mangle]` item cannot be
made conditional on a constant's value.

**The instance.** The machine pointer the host holds is a `Box<Instance<T>>`: the core, its outbox, and what the crate
last observed of it (the picture's size, the audio rate, the state sizes). After the calls that can change those, the
crate compares and queues `GEOMETRY`, `AUDIO_RATE` or `STATE_SIZE` itself. The reason is the one §2.4 gives for
descriptors: a rule that every core must remember is a rule some core will forget, and the crate can observe these
three changes exactly. `BATTERY` and `MACHINE_INFO` cannot be observed from outside and remain the core's.

**Logs, events and words without a context argument.** A core calls `log`, `emit` or `detail` from anywhere in its
code. During a machine call a thread-local route, set and restored by each export, sends them to that machine's
outbox; outside one they go to the library's log. The alternative, a context passed to every trait method, would
thread a parameter through every core's internals for the sake of its rarest operation. `detail` gives the failure the
call in flight is about to return its words, which become `last_error` or create's error text; without it the crate
uses `status_text`.

**What the crate enforces rather than documents.** Settings are checked against the schema before the core sees
them, and create's are completed with every default (§6.13). `set_options` passes bit 0 only. `frame_info` writes the
frame count as the serial and a row repeat of 1 for a core without the bits. Machine info's space sizes, battery
lengths and state kinds are read from the exports. `abi` and `capabilities` in core info come from the version and
the constant. Each of these closes a way for a descriptor to disagree with the export it describes.

### 18.3 How the guard is arranged

- **Two views by bindgen.** The check's build script runs bindgen on the header twice: once whole, for its structs and
  macros, and once for its function types only, with the five structs replaced by `emusen-native`'s own Rust types.
  The test core's generated exports are then assigned to the second view's types, so that a parameter or return that
  differs is `E0308`. Comparing the first view's structs with the crate's is done field by field: offsets by
  `offset_of!` against each named field, and types by placing the two fields in one array, which Rust permits only for
  one type. A field pointing to another of the header's structs is compared as a pointer of the same mutability.
- **Facts by libclang.** Functions, structs, fields and enumerations are read through libclang for each of the four
  triples, freestanding, with the compiler's own include directory found as bindgen finds it: a libclang loaded at run
  time did not always locate `stddef.h` (**measured**: the first run parsed the Linux triple and failed the other
  three). Macro values come from bindgen's evaluation. The header's function types are compared with its
  declarations, so the header cannot disagree with itself.
- **Nothing left unchecked.** At run time the check compares the header's functions with the exports assigned and
  with `emusen-native`'s export table, its fields with the fields compared, and its macros and enumeration constants
  with the constants compared. A declaration added to the header and not to the check fails it.
- **The diff.** Against the committed baseline, any difference fails, with §4.3's rule for a removal or change and
  §5.5 for an addition. Given the base branch's baseline (`--against`), every one of its lines must survive unchanged
  with its tag, new lines must carry the header's minor, and the minor must exceed every released tag.
- **CI.** `rust-cores.yml` gains the job `abi-guard` (Ubuntu with libclang: the check's tests, then `--check
  --against` the base commit's baseline), a pull-request trigger, the export check on the test core on all three
  shared runners, the export check on every built library where the runner can load it (all but osx-x64), and the C#
  check in the WiseMan job. A pre-stable library passes the export check as "not on the stable ABI", and keeps its old
  count until it moves.

### 18.4 What was measured

**P3, seeded faults.** Each was made on the working tree, the check run, and the tree restored byte for byte:

| Fault | Result |
|---|---|
| `set_mutes`' mask `uint32_t` to `uint16_t` in the header's declaration and function type | build fails, `E0308` |
| `FrameInfo`'s `aspect_num` and `aspect_den` swapped in the Rust: two `u32`s, so size and alignment unchanged | build fails, `emusen_frame_info.aspect_num` by name |
| `emusen_core_present_fn` returning `int64_t` while the declaration returns `int32_t` | build fails, `E0308` |
| a header field renamed, the Rust unchanged | build fails, `E0609` |
| C#: `SetMutes` declared with a `ulong` mask | `CoreAbiTests` fails, naming `emusen_core_set_mutes` |
| C#: `set_crash_log` declared returning `void`, as `NativeCoreLibrary.cs:81` calls the pre-stable export | the comparer reports the return and the parameter (a pinned test) |
| export check: a baseline requiring an export the library lacks, and lacking one it has | both reported |

The second row is the swap §5.2's prototype first missed; the per-field assertion catches it, as that section
required. P3 is retired as held.

**The test core** exports exactly 55 `emusen_core_` symbols and nothing else (`nm -D --defined-only`), and the export
check passes on it. `emusen-native`'s suite is 50 tests (34 before, 16 for the `core` module), the check's 3. MoonRT,
MercuryRT and VenusRT pass their own suites against the changed crate and MarsRT builds against it. With the
four libraries rebuilt from it, WiseMan's `CoreAbi`, `NativeHost`, `MoonRt`, `MercuryRt` and `VenusRt` tests pass, 419 of
419.

### 18.5 Points of §4 to §6 made precise by building them

Each is now stated in its section; none changes a decision.

1. The callee never writes a struct's `size`, and leaves bytes past its own size alone (§4.2).
2. Create's error text is NUL-terminated within `error_len` (§6.2), since the call returns no length for it.
3. The event drain returns the number copied, with a null `out` the number waiting (§6.19); the log drain copies
   whole records and has a stated line format, record bound and drop notice (§6.18).
4. Which events the crate raises and which the core raises (§6.19); when `AUDIO_RATE` is raised relative to the
   drain that reaches the new rate (§6.7).
5. A switch's values are `true`/`false`, a duplicate key is refused, and create receives every default (§6.13).
6. A space's flags are an array of strings; an axis entry's fields; firmware's `which` is at least 1, 0 being the
   battery (§6.3, §6.4, the schemas).
7. `status_text` answers `EMUSEN_NOT_SUPPORTED` for a code it has no words for (the header).
8. The baseline's tags, its `export` lines and its one-fact-for-all-triples form (§5.4).
9. Appendix A is superseded by the committed header, which adds the function types and the constants the exports'
   meanings depend on.

### 18.6 Switching a core onto `core_exports!`

For the core that moves next, VenusRT, and for MoonRT and MercuryRT after it:

1. Implement `emusen_native::core::Core`. Its required methods are those of `NativeCore` less `CORE_VERSION` and
   `ENGINE`, with `create` taking a `Create` (the image, the settings resolved against the schema, the files, the pixel
   formats), plus `info()` and `machine_info()`. `frame_info` returns `core::FrameInfo`, whose `FrameInfo::rgba(width,
   height)` fills the format, stride and length. The state stays `StateMachine`.
2. Invoke `core_exports!(Machine; …)`, naming the optional groups claimed.
3. Move the console's facts the shim held into `info()` and `machine_info()` (the controller's bits, the spaces and
   their flags, the battery files, the state format, the patch range), the settings into `settings_schema()`, and the
   status words into `status_text()`.
4. Run `export_check.py` on the built library and the core's own suite; `emusen-core-abi-check` needs no change.

**The transition.** Until §13.2's item 3 lands, no C# host loads a v1 library, and a core's existing C# shim reaches
it only through `emusen_native_*`. VenusRT's does (`Shim/VenusNative.cs`, `Shim/VenusMachine.cs`), and so do WiseMan's
SNES fixtures (`Fixtures/Snes/SnesEngines.cs`) and its test-ROM runner. A library may therefore keep `native_exports!`
beside `core_exports!` for that period: the two sets have disjoint names, so neither loader sees the other's.
**Measured** on a scratch crate built from the test core with both macros: the library exports 55 `emusen_core_`
and 25 `emusen_native_` symbols, answers `0x10000` and `0x10001` from the two version exports, and passes the export
check. The one cost is in the core's own source: where `NativeCore` and `Core` share a method name, a call on `self`
must name its trait. The pre-stable macro is removed when the adapter replaces the shim.

### 18.7 Not done here

§13.2's items 3 to 6 (the adapter and loader, discovery, the SNES's system pack, the conformance kit). The pre-stable
host's `set_crash_log` declaration (§1.2, item 8) is left as it is: it retires with that host (§13.5), and the v1 table
declares it correctly. `VenusRT_Plan.md` §4.4's last sentence, which still placed the codec rewrite in the shim, was
corrected with §19.

---

## 19. What was built, 2026-10-03: §13.2's items 3 and 4

### 19.1 The host: loader, machine, engine and debug target

All in `EmuSen/Cores/Native/` until D1 moves them (§10.2).

| Class | File | What it is |
|---|---|---|
| `CoreLibrary` | `CoreLibrary.cs` | One library, loaded once from its path and checked as §7.2's step 4 asks: major 1; every required export; capabilities and exports agreeing in both directions; info and schema UTF-8 JSON under 1 MiB; info's `abi`, capability names and `host_requires` matching. A refusal names the check, and the same object, with the same verdict, answers every later `Open` of that path |
| `CoreDescriptorReader` and its records | `CoreDescriptors.cs` | Core info, machine info, the settings schema, firmware and notes as C# records |
| `CoreMachine` | `CoreMachine.cs` | One handle, one method per export; every refusal thrown with the core's own words, from `last_error` or `status_text` |
| `CoreEngine` | `CoreEngine.cs` | The one engine class: `ICore` and every optional interface of the engine SPI over any v1 core |
| `CoreDebugTarget` | `CoreDebugTarget.cs` | `IDebugTarget` from the descriptors: spaces, processors, registers, disassembly, mutes |

**The engine's answers, capability by capability.** Where the core has the capability the engine passes it through;
where it lacks it the answer is the one a frontend saw before the interface existed:

| Interface | With the capability | Without |
|---|---|---|
| `ISnapshotCore` | state kind 1 | the full state |
| `IFrameSerial` | `frame_info.serial` | the frame count, which the macro writes |
| `IRepeatedRows` | rows repeated by the engine, or handed once with `RowRepeat` | a repeat of 1 |
| `IFrameProfiler` | machine info's phases with `phases`' times | one phase, `frame`, timed by the engine |
| `ICoreSettings` | the schema's switch, count and choice rows; a run-scope change sent with `set_settings`, its note read back | the same rows; a run-scope change kept and sent at the next load, inside create's settings |
| cheats | ROM patches within machine info's `patches`; with `CHEAT_POKES`, plain one-byte pokes as quads and the rest through `space_write` | every poke through `space_write` |
| `IStateFormat`, `IFrameBufferPool`, `IEngineFeatures`, `ICheatRegistryHost` | as the pre-stable host has them | — |

Input is routed by the descriptors: a `PadButton` to every bit of the port's controller labelled with it, a `PadAxis`
to `set_axis` with the axis number the controller gives, clamped to the kind's range.

**The frame** follows §8.2's order: `advance`, the frame-end work (frame log, cheats, the breakpoints' frame notice,
the battery period), `present` when claimed and rendering is not skipped, then the events drained. `GEOMETRY` re-reads
the picture's shape, `AUDIO_RATE` the rate, `MACHINE_INFO` the descriptor, `BATTERY` writes the battery file where the
core tracks changes, and `LOG` drains the log, which is drained once a second as well.

**Battery files before create.** File 0 is read from the runtime's path rule (`BatterySave`, `.srm`) before create.
After it, machine info names the files the machine keeps; where those differ in number or suffix and one of them
exists on disk, the engine creates the machine again with them. The cost is a second create in that case only. It is
recorded because v1.0 gives a host no way to know a core's battery files before a game is loaded: core info could
carry them per system in a later minor (§4.4 permits the field), and §15 does not decide it.

### 19.2 Discovery and the engine row

`CoreDiscovery.cs` holds the sidecar, the discovered core and the scan.

- **The sidecar**, `<library>.core.json`: `{ "sidecar": 1, "library": <file name>, "sha256": <hex>, "abi": "1.0",
  "capabilities": <bits>, "info": <core info>, "settings": <schema> }`. `CoreSidecar.Write` loads the library once
  and writes it, which is the build step's job (§7.1). The library is named by file name only and must sit beside its
  sidecar.
- **The scan** reads the sidecars in the directories beside the assemblies (and their `cores` folder) and loads
  nothing. Two sidecars with one `id` keep the first. A sidecar marked `"development": true` is skipped unless
  development cores are asked for (§27, added 2026-10-04).
- **Opening** a discovered core, when a game is opened on it, checks the library's SHA-256 against the sidecar's
  before loading, and its info against the sidecar's (as JSON, field by field) after. Either difference refuses it with
  a report that the engine notice carries.
- **The engine row.** `CoreCatalog.EngineFor(console)` appends every discovered engine whose system id is the
  console's (a small table maps `snes`, `nes`, `gb`/`gbc` and `n64` to the four consoles) to the console's row, and
  creates the row for a console that had one engine, with the C# reference as its default. The SNES's row therefore
  lists `Venus (C#)` and a discovered SNES engine, with no code naming the engine.
- **The factory.** `CoreFactory.Create` has one generic branch before its per-console switch: the discovered engine
  asked for by name, or, for an extension no C# core claims, the first engine that claims it. An engine that cannot be
  opened falls through to the switch, and `EngineNotice` says why in the same words as the pre-stable engines' notices.
  `Bundle` gives a `CoreEngine` the generic debug target and the codecs of its system's pack (§13.2 item 5).

### 19.3 What was tested

On the two test cores, `v1_test_core` (every capability) and `v1_plain_core` (none), which WiseMan builds and copies
beside its tests: `CoreAdapterTests` (10 cases: the loader's acceptance and refusals, a refused game's words, frames,
picture, sound and input from the descriptors, settings, states and snapshots, the neutral answers, cheats both ways,
battery files written and read back, the debug target, firmware), `CoreDiscoveryTests` (5 cases: listing without
loading, a swapped library never loaded, altered info refused, the SNES row and the factory's refusal with its notice,
a game no C# core claims opened on the v1 engine with its bundle), and `CoreAbiTests`, which now also holds the host's
export table and capability numbers to the baseline.

**P6, in part.** Listing the engines and asking which extensions are supported loads no library, by the loader's own
record of what it has opened (`CoreLibrary.IsOpen`). P6's `strace` check of a whole frontend's start remains for when
a project core ships with a sidecar.

### 19.4 Not done here

- **The build step** that writes the project's sidecars: no project core is on v1 yet. The conformance runner's
  `--sidecar` mode (§21.1) is the tool it is to run.
- **Trust beyond the sidecar** (§7.2 rules 1–3): `cores.manifest`, the player's one-time approval, the file checks
  before loading, and the player's `/Cores` directory.
- **Halting through `debug_run_frame`**: the debug target observes and disassembles, but its processors report
  `CanHalt = false` until a bridge over §6.14's run exports is built.
- **Validation of descriptors by their schemas in C#**: the loader checks the fields a host relies on; the full schema
  validation is the kit's (C2, C3).

---

## 20. What was built, 2026-10-03: §13.2's item 5, the SNES's system pack

### 20.1 Where it is, and what it holds

`EmuSen.DianaOS/DianaOS/Sys/Systems/`, the folder §15 Q13 decided, holds `SystemEntry`, `SystemPack` and the registry
`SystemPacks.For(id)`. The SNES's pack is `Sys/Systems/Snes/`:

- **The systems-table entry** (`SnesSystem.Entry`): the id `snes`, the name, the console `SNES`, Nintendo and 1990, the
  extensions, the libretro cheat-database folders, OpenVGDB's system and its hashing rule (the image without a 512-byte
  copier header), the cover aspect and the ES-DE names. `CoreCatalog`'s SNES row is now built from it, so the row and
  any SNES engine read one source; the values are those the row held, moved, not changed.
- **The space names the SNES's engines share**: `CpuBus` and `WRAM`, so that a cheat means the same memory on each. The
  names are VenusRT's own (`Shim/VenusMachine.cs`, `SpaceNames`); an SNES engine's machine info must use them.
- **The two codecs**, Pro Action Replay (auto-detected) and Game Genie (explicit), in `SnesCheatFormats`.
  `CoreFactory.Bundle` gives a v1 engine the codecs of the pack its machine's system names.

### 20.2 The formats, and where they were read

Both are written from one source: fullsnes, "SNES Cart Cheat Devices - Code Formats", in the copy `VenusRT_Plan.md` §2.1
pins (`~/.cache/emusen/probe/venusrt/docs/fullsnes.txt`).

- **Pro Action Replay**, `AAAAAADD`: a 24-bit address and a byte. fullsnes states that the device rewrites WRAM on each
  vertical blank and accepts WRAM only as 7E0000h–7FFFFFh, not by its mirrors, so those addresses become `WRAM` pokes at
  their offset; every other address is a `CpuBus` poke, which is how the device patches cartridge ROM and SRAM.
  `7E000000` is the device's "do nothing" padding code and applies nothing. The pre-boot codes (FE and FF), the
  multi-byte prefix `DEADC0DE` and the device-control codes (`C0DEnn00`) are refused rather than applied as pokes,
  which they are not.
- **Game Genie**, `DDAA-AAAA`: the digits enciphered by fullsnes's table (Genie `DF4709156BC8A23E` for hex 0 to F),
  the value the first two, the 24-bit address the last six with its bits in the documented order
  `ijklqrst opabcduv wxefghmn` for the address `abcdefgh ijklmnop qrstuvwx`. A Game Genie code patches ROM reads, so it
  is a ROM patch, sent to a core with `ROM_PATCHES` within machine info's `patches`.
- **Which claims which.** The two are both eight hex digits, so the Game Genie claims the printed form with its dash
  and the Action Replay the bare form; `cheat gg` decodes either form as Game Genie.

**Tests** (`SnesSystemPackTests`): vectors worked by hand from fullsnes's table and order, one address bit and the
value byte at a time; the shuffle shown to be a permutation that the encoder inverts over a thousand random codes; the
WRAM and bus pokes and the padding code; each format claiming only its own form, and the cheat command's routing; the
SNES row held to the entry. **What the tests cannot show:** the vectors check the code against the document, not the
document against a device. No hardware or reference result was consulted, by the rule of item 5.

### 20.3 Provenance

Nothing of C# Venus was read to write the pack. One line of C# Venus was seen: while searching for the SNES's space
names, a `grep` pattern matched a line of `Venus - SNES/Cheats/VenusCheatCodecs.cs`, which shows only that its Action
Replay codec targets a space named `CpuBus`. Nothing further in that folder was read. The name has an independent
source, VenusRT's own `VenusMachine.SpaceNames`, which is the one cited above.

## 21. What was built, 2026-10-03: §13.2's item 6, the conformance kit's core suite

### 21.1 The runner

`EmuSen/Cores/Shared/emusen-core-conform/`, as §12.2 placed it: a Rust library and the binary `emusen-core-conform`.
It loads a library through the ABI and nothing else (`libloading` and `emusen-native`'s structs), and runs on an image
the author supplies:

```
emusen-core-conform --core LIB --image FILE [--frames N] [--input SCRIPT] [--settings KEY=VALUE]... [--file WHICH=PATH]... [--report REPORT.json]
emusen-core-conform --sidecar LIB
```

The exit status is 0 when every case passes; the report lists each case's verdict and evidence with the kit's version
and the core's `abi`, `id` and `version`. `--sidecar` writes `<LIB>.core.json` in §19.2's format, with the descriptors
as the library wrote them, for the build step §7.1 asks for.

- **Input** is Pharaoh's hold, release and tap as lines `hold F PORT MASK`, `release F PORT MASK`, `tap F PORT MASK`
  (the mask in hex, `tap` holding for one frame). Without a script, port 0's buttons are pressed in turn, eight frames
  on and eight off, from the first controller of the core's info.
- **Digests** are Pharaoh's: each frame's FNV-1a hash folded per frame (`framesum`), each sample folded as an unsigned
  16-bit value (`audiosum`), and the state's bytes hashed.

### 21.2 What each case checks

| Case | As built |
|---|---|
| C1 | major 1; every required export resolves; where `nm` exists, no `emusen_core_` symbol outside version 1's exports |
| C2 | info validates against `info.schema.json`; its `abi` is the export's; its capability names are the claimed bits; each optional export present exactly when its bit is claimed |
| C3 | the schema validates; keys unique; each default in its own domain; an `accuracy` default its `accurate` value, an `enhancement` default its `hardware` value; every trade-off with a cost |
| C4 | an empty, a garbage and a half-length image each refused with a negative status that `status_text` names and an error text, or, unless `--self-delimiting` is given, accepted and run for up to 300 frames on a thread of its own within 60 seconds (a frame refused with a status is a clean stop); each image's outcome in the report as `refused`, `accepted and ran`, `accepted and stopped` or `hung`; a machine with no settings equal to one with every default stated, frames, sound and state, over up to 300 frames; each firmware entry's `replacement.effect` one of `exact`, `accuracy`, `none`, with a `cost` unless `exact`, and an image whose entries are all `required: false` created with no files |
| C5 | each claimed capability answers on a running machine without `NOT_SUPPORTED`; an unclaimed snapshot and state kind 77 refused; an unlisted space refused; reserved option bits ignored; port 99 ignored or `NO_SUCH_PORT`; an unknown pixel-format bit accepted with RGBA8888 produced |
| C6 | over N frames: `frame_info`'s length is `frame_copy`'s, the format the one offered, the size within its stride; a size change preceded by `GEOMETRY`; each change of the drained rate met by an `AUDIO_RATE` of that rate in its frame or the next |
| C7 | the state's size constant until a `STATE_SIZE`; save, load, save byte-identical; a truncated, a foreign and an empty state refused with the machine unchanged; machine B loaded with A's state continuing as A; a claimed snapshot loading and restoring the state it was taken from |
| C8 | two machines alike giving identical digests in both halves and the same end state; a machine loaded with the state at N/2 continuing identically; where skipping is declared state-neutral, rendering skipped on odd frames leaving the same end state |
| C10 | two machines on two threads, each equal to the solo run, with a third created and freed while they run |
| C11 | every machine export with a null machine answering `EMUSEN_NULL`, and create with null params; the length query with a one-byte buffer; `create_params` of 80 bytes, a 16-byte file element, a 48-byte `frame_info` and 16-byte events each `BAD_STRUCT`; `status_text` answering for every code the core returned in the run |

### 21.3 What was tested

On the test cores, built again as this crate's examples: both pass every case. A third build of the same machine with
two seeded faults, each machine differing from the last and a truncated state accepted, fails C4, C7, C8 and C10 and
passes the rest, which is the set those faults break (`tests/kit.rs`). The kit's SHA-256 is checked against FIPS
180-4's examples. CI runs the crate's tests on the three shared runners with the other shared crates.

### 21.4 C4's rule, decided 2026-10-03

The first version of C4 required every malformed image, a half-length one included, to be refused. That is a fair test
of a format that records its own length, such as the test core's, and not of a SNES cartridge image: half of a 1 MiB
image is a well-formed 512 KiB image, and refusing it would refuse genuine smaller dumps. **Decided:** what C4 requires
is that the core is never harmed by a bad image. A malformed image is refused with words, or loads and runs the kit's
frames without crashing, hanging or corrupting the host; refusal is required only where the system pack declares its
image format self-delimiting. The SNES's pack does not declare it (`SystemEntry.SelfDelimitingImages` is false); the
test core's format does, and the kit's tests pass `--self-delimiting` for it.

The kit runs an accepted image on a thread of its own and waits 60 seconds; a hang is reported and the thread left, so
one bad core cannot stall the suite. A crash ends the kit's process, which is itself the failure; corruption of the host
that does not crash it cannot be observed from inside the process (§7.3's broker is the boundary that would). **Tested:**
the test core refuses all three images and passes with or without `--self-delimiting`; a lenient build of it, which
takes any non-empty image whole, passes without the flag and fails with it, and its report names `accepted and ran`
(`tests/kit.rs`).

### 21.5 Not done here

C9 and C12–C15 are not in §13.2's item 6; they were built afterwards (§24). The build step that runs `--sidecar` on
the project's cores waits for the first project core on v1.

---

## 22. MoonRT onto v1, 2026-10-03: §13.3's step 4, its first half

### 22.1 What was done

- **The exports.** `src/ffi/v1.rs` implements `emusen_native::core::Core` for MoonRT's machine and invokes
  `core_exports!` beside the pre-stable `native_exports!`, as §18.6 describes: the library now exports 46 `emusen_core_`
  symbols (the 32 required and `RESET`, `MUTES`, `ROM_PATCHES`, `DEBUG` and `DEBUG_STACK`'s), its 38 `emusen_native_`
  and its two `moonrt_` extensions. The pre-stable implementation is unchanged.
- **The descriptors** carry what the shim held in C#: the pad's eight bits under their canonical controls, listed in
  `MoonCore.PadButtons`' order; the eight spaces under `MoonMachine.SpaceNames`, `CPUBUS` marked as reading with side
  effects; the battery file where the cartridge has one; the state's magic `MOON`, version 4, read from 3; the patch
  range 4020h–FFFFh; the frame rate as `MoonCore.FrameRateHz` computes it, 21,477,272 / (262 × 1,364); the five APU
  channels by name; and `status_text` for its band in the shim's words. Settings: none.
- **The shim stays MoonRT's loader.** The build writes MoonRT's sidecar (below), so discovery lists it. So that this
  changes nothing, an engine the build registers by hand (`CoreCatalog.IsRegisteredEngine`) keeps its own branch:
  `CoreFactory`'s generic branch and the engine notice pass it by. `MoonRT (Rust)` therefore still runs as
  `MoonRtCore`, Moon (C#) stays the NES's default, and the NES row is unchanged.
- **The sidecar.** `RustCores.props` marks MoonRT `CoreAbi 1`; `EmuSen.csproj`'s `WriteCoreSidecars` builds the kit's
  runner and runs `--sidecar` on each such library, and the sidecar is copied beside it. CI does the same in the
  library job for any library that exports `emusen_core_abi_version`, on every platform whose library the runner can
  load.

### 22.2 The oracle, §13.1's row, measured 2026-10-03

| Oracle | Result |
|---|---|
| The state and machine tests, the engine tests | WiseMan's `MoonRt`, `NativeHost`, `BatterySave`, `MoonCheat`, engine-row and factory tests: 331 pass, unchanged, on the library with both interfaces; the crate's own 6 pass |
| The adapter against the shim | `MoonRtCoreAbiTests`: the generic `CoreEngine` over MoonRT's v1 exports and `MoonRtCore` over its pre-stable ones, run side by side for 240 frames with input on both pads, give the same picture every frame, the same samples every frame and byte-identical states; a cheat applied through the adapter lands |
| The bench's state hashes | `moonbench`'s four games, 900 boot and 3,000 timed frames: one state hash per game in all forty runs, before and after, and the same as C# Moon's |
| The plain frame, interleaved | below: not slower; 1.37% faster by the geometric mean, which is outside ±1% on the fast side |
| The kit | C1–C8, C10 and C11 pass on all four games since Moon's state version 5 (§22.3); before it, C7 and C8 failed |

**The timing.** Five rounds under the bench lock, the four games interleaved and the two builds alternated within each
game. "Before" is the same source without `v1.rs`; both were built by cargo's release profile with the same toolchain
(MoonRT has no PGO profile). The load average was 6.0–6.4 throughout, against the 0.7–1.6 of §8.4.4 of
`Moon_Native.md`, so single runs spiked (p90 to 2.5 ms on both builds alike) and the comparison is of medians:

| Game | Before, median p50 ms | After | Change | Median of means, change |
|---|---|---|---|---|
| Super Mario Bros. | 1.159 | 1.139 | −1.73% | −1.58% |
| The Legend of Zelda | 0.956 | 0.938 | −1.88% | −1.34% |
| Super Mario Bros. 3 | 1.386 | 1.368 | −1.30% | −0.85% |
| Mike Tyson's Punch-Out!! | 1.264 | 1.257 | −0.55% | −0.24% |

The frame is not slower on any game. It is faster by more than P1's band allows on three, which is the kind of code
layout effect `EmuSen_NativeCores.md` §8.2 measured before, and it was measured on a loaded machine.

**Measured again, 2026-10-03,** batched with MercuryRT's (§23.3): "before" as above against the library with both
interfaces and state version 5, five rounds each held until the load fell under 2.0 (starting at 1.73–1.97; the load
at the start of the MoonRT runs was 1.73–6.01):

| Game | Before, median p50 ms | After | Change |
|---|---|---|---|
| Super Mario Bros. | 1.149 | 1.097 | −4.53% |
| The Legend of Zelda | 0.916 | 0.918 | +0.22% |
| Super Mario Bros. 3 | 1.320 | 1.326 | +0.45% |
| Mike Tyson's Punch-Out!! | 1.218 | 1.234 | +1.31% |

The geometric mean is −0.66%, inside P1's band. Super Mario Bros. is faster outside it in every round, as it was in the
first measurement, and Punch-Out!! is 1.31% slower, outside it by 0.31 points on a loaded machine. P1 holds for the
geometric mean and is not retired game by game; the machine was never quiet, and a quiet run remains owed.

### 22.3 The kit's C7 and C8: a finding about Moon's state format

On every game C7's "B, loaded with A's state, continues as A" and C8's "across a save and load at N/2" fail on the
sound alone: the picture and the state agree, and the samples differ (on Super Mario Bros., B made 2 fewer samples
over 300 frames and a different digest).

**The cause is the oracle's format, not the port.** C# Moon marks its mixer `[SkipInState]`: the resampler's
accumulator and cycle fraction and the three filters' states (`Apu/Apu.cs`, `_sampleAccumulator`, `_cycleFraction`,
`_hp90`, `_hp440`, `_lp14k` and the rest). A machine loaded with a state therefore resumes its sound with those reset,
and MoonRT, whose state is C# Moon's byte for byte, does the same. **Measured** with a scratch probe that runs a machine
300 frames, loads its state into a second, and runs both 300 more through `ICore` without input: on Punch-Out!! the
samples differ from the first one after the load, in C# Moon and in MoonRT alike, with equal states and equal sample
counts; on Super Mario Bros. they agree. The kit's scripted input makes the difference show on every game.

**Decided 2026-10-03:** accuracy comes first, and a state load must resume exactly, sound included. The mixer joins
Moon's state as version 5, in C# Moon and MoonRT together and byte for byte the same; both still read versions 3 and 4
with the mixer kept as it was, and every state written is version 5 (`Moon_Native.md` §3.13). The kit asks no
declaration of approximate sound, and none is offered.

**After it** (measured 2026-10-03): the second machine's samples are identical to the first's in both engines, C1–C8,
C10 and C11 pass on all four games, and the bench's state hashes change only by the added bytes: each version 5 state
is its version 4 state with the version field 4 → 5 and 60 bytes appended, which hashes to the old value with those
undone. MoonRT is v1-compliant on the four games. P1's timing is still to be measured on a quiet machine.

---

## 23. MercuryRT onto v1, 2026-10-03: §13.3's step 4, its second half

### 23.1 What was done

- **The mixer gap first.** Before the move, Mercury's mixer was checked for §22.3's gap, and it had it: the resampler's
  cycle fraction and the two output capacitors were `[SkipInState]`. They join Mercury's state as version 8, in both
  engines, as Moon's did (`Mercury_Native.md` §10).
- **The exports.** `src/ffi/v1.rs` implements the `Core` trait and invokes `core_exports!` beside `native_exports!`:
  45 `emusen_core_` symbols (the 32 required and `MUTES`, `ROM_PATCHES`, `DEBUG` and `DEBUG_STACK`'s), the pre-stable
  37 and the six `mercuryrt_` extensions unchanged. The crate keeps its aligned build (`Mercury_Native.md` §8.7.3).
- **The descriptors.** Two systems, `gb` (`.gb`) and `gbc` (`.gbc`), each with the pad's eight bits listed in
  `MercuryCore.PadButtons`' order; machine info's `system` names the console running, so a Game Boy game forced onto
  the colour console is `gbc`; the seven spaces under `MercuryMachine.SpaceNames`; the battery file; the state's magic
  `MERC`, version 8, read from 5; the patch range 0000h–7FFFh; the frame rate 4,194,304 / 70,224; the four channels;
  and `status_text` in the shim's words. **The Model setting** is the schema's one entry, a create-time choice whose
  values are C# Mercury's own words (`Auto`, `Game Boy`, `Game Boy Color`), the values frontends already store.
- **The shim stays the loader**, and Mercury (C#) the default, by §22.1's rule; `RustCores.props` marks MercuryRT
  `CoreAbi 1`, so the build writes its sidecar.
- **A defect of the adapter, found by this core and fixed.** The adapter chose a game's battery folder from the
  catalogue's console, under which a `.gbc` game is `GB`. The shim and C# Mercury file a `.gbc` game's save under `GBC`
  (`BatterySave.GameBoyFolder`). The adapter now takes the folder from the system the extension names, which gives
  `GBC` for it and the same folders as before for every other core.

### 23.2 The oracle, §13.1's row, measured 2026-10-03

| Oracle | Result |
|---|---|
| The 586 WiseMan tests §12.5 of `EmuSen_NativeCores.md` names | WiseMan's Mercury, native-host, battery and engine tests: 590 pass on the library with both interfaces, with the version 8 state |
| The adapter against the shim | `MercuryRtCoreAbiTests`: the generic `CoreEngine` over the v1 exports equals `MercuryRtCore` over the old ones for 240 frames with input, on a Game Boy, a Game Boy Color and with the colour console forced: picture and samples every frame, and the state. The shim gives the mixer C#'s own `Math.Pow` coefficients and the v1 path Rust's `powf`; they agree here (linux-x64) |
| The digest over runs | the platform digests: sound and picture unchanged on all four programs and both engines; the states changed only by version 8's bytes (`Mercury_Native.md` §10) |
| The kit | C1–C8, C10 and C11 pass on four games (Tetris, Kirby's Dream Land, Link's Awakening, Pokémon Yellow). Under C4's rule a Game Boy image is not self-delimiting; MercuryRT refuses an empty image and accepts the garbage and half-length ones, each then stopping cleanly at an illegal opcode or running |
| The plain frame, interleaved, aligned build | §23.3 |

### 23.3 The timing

Measured 2026-10-03 with `mercbench`, `moonbench`'s method for the Game Boy (900 frames booted with Start and A
pressed in turn, 3,000 timed through `ICore`, the picture taken and the sound drained every frame). "Before" is
MercuryRT as it stood before this step (state version 7, no v1 exports), "after" this step's library; both built by
cargo's release profile from the crate's directory, so both carry its alignment flags, which their symbol tables
confirm. Five rounds under the bench lock, interleaved with MoonRT's re-run (§22.2), the two builds alternated within
each game. **The load:** each round was held until the one-minute load average fell under 2.0, and the rounds started
at 1.83, 1.94, 1.97, 1.73 and 1.92; other work on the machine raised it within the rounds, and the load at the start
of the Mercury runs was 2.11–4.68. This is not the quiet machine §8.7.3 of `Mercury_Native.md` had, and is stated as
such.

| Game | Before, median p50 ms | After | Change |
|---|---|---|---|
| Tetris | 0.4473 | 0.4482 | +0.20% |
| Kirby's Dream Land | 0.5003 | 0.4986 | −0.34% |
| The Legend of Zelda: Link's Awakening | 0.4178 | 0.4140 | −0.91% |
| Pokémon Yellow | 0.5486 | 0.5490 | +0.07% |

Every game within ±1%; the geometric mean −0.24%. **The bench's state hashes** were one per game per build in every
run; each version 8 state is its version 7 state with the version field 7 → 8 and the 24 mixer bytes appended,
byte for byte (checked by undoing the two on all four), and C# Mercury writes the same bytes as MercuryRT.

---

## 24. The kit's remaining cases, 2026-10-03: C9 and C12–C15

### 24.1 What each case checks, as built

| Case | As built |
|---|---|
| C9 | every setting of `effect: exact`, at each value other than its default (both values of a switch, each choice, a count's minimum, middle and maximum), on a machine of its own over up to 300 frames, giving the default's frame, sound and state digests |
| C12 | C6, C7 and C8 run twice more: as a host of minor 0 whose `create_params`, file elements, `frame_info` and events are version 1.0's sizes, and as a host of minor 99 whose structs are each 32 bytes longer, those bytes filled with the canary `0xA5`; every case passes in both, and after every call the kit finds each canary intact |
| C13 | four threads call `abi_version`, `capabilities`, `info`, `settings_schema`, `firmware_for` on the image, `status_text` on four codes and `log_drain` with no machine, round after round, while a machine runs up to 300 frames; every answer equals the one taken before the threads started, and the machine's digests equal a run alone |
| C14 | for each space in machine info, `space_size` equals the declared size; a `read_only` space answers a one-byte write with a negative status and reads back unchanged; each battery file's `length` equals `battery`'s length query; when `DEBUG` is claimed, `debug_pc` answers 0 for each processor named |
| C15 | where `DEBUG` is claimed: a machine with calls, writes, profiling and coverage 0 armed, a breakpoint on an address no processor has, and a range over the top of each space, run through `debug_run_frame` (a `RING` stop continued, any other stop a failure) gives the plain run's digests; on a machine two frames in, a breakpoint at processor 0's next instruction stops with `BREAKPOINT` at that pc on a processor machine info names; with `EACH` set, a `CONTINUE` call stops with `EACH` without moving the machine, and eight `UNCHECKED` steps each answer 0 or `EACH` and each move it |

**Points of §12.1 made precise.** §12.1's C15 says "a breakpoint at the first instruction stops before it"; the kit
places it at processor 0's pc after two frames, since the reset vector's first instruction has already run by the time
a host could set anything through a created machine. Its "`EACH` stops at every step" is held to a `CONTINUE` call
stopping at once, which is what a debugger's step needs, and to eight steps that each move the machine. C12's "a minor
far ahead" is 99 and its "larger structs" 32 bytes longer; C13's "several threads" are four.

### 24.2 The seeded faults

The test core's machine (`emusen-native/examples/common/test_core.rs`) gained five faults, one for each case, and a
seventh build of it, `kit_faulty_full_core`, claims every capability with all five on. The faults, and the case each
breaks:

| Fault | What the core then does | Case | The kit's words |
|---|---|---|---|
| 8 | its `Threads` setting, declared `exact`, adds threads − 1 to a RAM byte each frame | C9 | `Threads=2 is declared exact and changes the output` |
| 16 | `create` refuses any host version but 1.0 | C12 | `a host of minor 99 ...: C6 fails: create refused the image` |
| 32 | `info`'s description carries a call counter | C13 | `a library-level answer changed while a machine ran: "info"` |
| 64 | a write to the read-only ROM space succeeds | C14 | `space 1 (ROM) is read-only and a write to it answered 1` |
| 128 | `debug_run_frame` drops the frame's sound | C15 | `every table armed with nothing to hit gives ... the plain run ...` |

**Tested** (`emusen-core-conform/tests/kit.rs`): the two test cores pass all fifteen cases; the full faulty build
fails exactly C9 and C12–C15; and each fault alone, set through `EMUSEN_TEST_CORE_FAULTS` in a runner process of its
own (the variable narrows the faults a build includes and is read once per process), fails exactly its own case. The
earlier faulty build fails C4, C7, C8, C10 and, through machines that differ from a solo run, C9's, C12's and C13's
comparisons as well.

### 24.3 The project's cores

Run 2026-10-03 from the merged tree's release libraries, with the runner's 600 frames:

| Core | Images | Result | Notes |
|---|---|---|---|
| MoonRT | Super Mario Bros., The Legend of Zelda, Super Mario Bros. 3, Punch-Out!!, with `--self-delimiting` | all fifteen pass on all four | no settings, so C9 has nothing to check; C14 finds 8 spaces and the battery file on Zelda; C15 runs on its one processor |
| MercuryRT | Tetris, Kirby's Dream Land, Link's Awakening, Pokémon Yellow | all fifteen pass on all four | its one setting, `Model`, is `effect: none`, so C9 has nothing to check; 7 spaces; C15 runs |
| VenusRT | A Link to the Past, Donkey Kong Country, with the SPC700 boot ROM as file 1 | all fifteen pass on both | no settings; C14 finds 8 spaces and one battery file; `DEBUG` is not claimed, so C15 has nothing to check |

**No defect was found in any of the three.** Two observations, neither a failure. C9 has not yet met a setting it
can check on a project core; MarsRT's step 5 is the first (P9). And C13's rounds differ by three orders of magnitude,
about 225,000 on MoonRT and 75,000 on MercuryRT against 2,856 on A Link to the Past and 264 on Donkey Kong Country:
VenusRT's `firmware_for` builds a whole cartridge from the image to read its header (`VenusRT - SNES/src/v1.rs:81`), so
each call costs in proportion to the image, a 4 MiB copy for Donkey Kong Country. The ABI sets no cost on a
library-level call, and the case still ran its checks a few hundred times; it is recorded for VenusRT's crate and was
not changed here.

---

## 25. The registration-equivalence test, 2026-10-04: D1's oracle

### 25.1 What it records

D1 (§10.2) moves the engine SPI, the native host and the systems table into `DianaOS/Sys/` and answers `CoreFactory`
and `CoreCatalog` from the runtime. Its oracle, named in `EmuSen_NativeCores.md` §7 step 1 and not built there, is now
`EmuSen.WiseMan/Cores/CoreRegistrationEquivalenceTests.cs`, held to the committed golden
`CoreRegistrationGolden.txt` beside it. It was written before any move and passes unchanged on `WiseMan` at
`400f6087`; the move is to keep it passing without an edit.

The recorder (`RegistrationRecorder` in the same file) asks every question a frontend asks of the registration and
writes one `key = value` line per answer: 1,875 lines in the `built` world below and 1,223 in `off`:

| Group | What is recorded |
|---|---|
| The catalogue | the registry's keys; the cores, their release order, the library's filter choices and shelves (with their ES-DE names); each console's extensions, cheat-database folders, maker and year, cover aspect, OpenVGDB systems, and the OpenVGDB hashing rule applied to each synthetic image (length and hash of the bytes hashed); every extension claimed, unclaimed and empty, through `IsRomExtension`, `ByExtension`, `ConsoleForRom` and `CoreFactory.IsSupported`; `ShelfFor` and `IsGameBoyColor` on six files; `ByAnyName`, `ByDisplayName`, `ShelfByName`; each console's buttons, axes, rebind controls, system ids and settings rows (label, kind, default, range, choices, hint and the note at the defaults); `ConsoleForSystem` |
| The engine rows | for the four consoles and one unknown: the row (or none), its choices, default and hint; the engines discovery adds; `EngineChosen` with nothing stored and with a value stored; `IsRegisteredEngine` for every engine name; whether each registered library is available; each discovered core, its systems and whether it opens |
| The factory | for every synthetic image of every console (plain, battery-backed, and the Game Boy Color and byte-swapped forms) and every engine name a frontend could store (none, each choice of the row, an unknown name, and another console's engine): the type `ForFirmwareProbe` builds and the firmware it asks for; the type `Load` builds, its engine notice, its debug target's type, its trace switch, both cheat codecs; the core's name, picture size, frame rate, sound rate, buttons, axes, state version, engine features and `ICoreSettings` rows with their values; and the files the load and one `SaveSram` leave under a fresh home, with older builds' save locations pre-filled so that a copy-in shows |
| Configured engines | `ConfiguredEngine` with nothing stored and with each choice stored in `graphics.json` |
| Cheat codecs | `CheatCodecsFor` every console and display name, none, empty and unknown; `DefaultCheatCodecs`; and each distinct codec's verdict on eighteen fixed codes (whether it claims the code, the address and value, the compare, the writes) |
| Settings before a game | each engine of each row built by `Create` and its `ICoreSettings` rows |

It is recorded in four worlds:

- **`built`**: the libraries as the build placed them beside the assemblies.
- **`off`**: a child `dotnet test` process started with `EMUSEN_MOON_NATIVE`, `EMUSEN_MERCURY_NATIVE`,
  `EMUSEN_MARS_NATIVE` and `EMUSEN_VENUS_NATIVE` set to 0, since a library is loaded once per process and only a
  fresh process records the fallbacks and their notices honestly (the method of `MercuryRtFallbackTests`).
- **`nothing-discovered`**: discovery pointed at an empty directory.
- **`refused-discovered`**: discovery pointed at a directory holding the build's sidecars beside files that are not
  their libraries, so that opening each is refused before loading.

### 25.2 What is held fixed, and what is named instead

The ICore's type is recorded by its name without its namespace, and a codec by its name, kind, space and decodes,
because D1 may move a type between assemblies without changing what a frontend sees. Paths are written relative to the
test's temporary root and the assemblies' directory, and a platform's library file name as `<lib crate>`, so that one
golden serves every platform. One value is the machine's: Mars's rasteriser threads default to one per three cores
(`MarsCore.VideoSettings`), and the recorder writes that default as `<one per three cores>` where it equals the
formula. The set of capability interfaces a core implements is deliberately not recorded: §13.1's ports gain the
adapter's neutral interfaces (§19.1) while every answer a frontend reads through them stays the same, and those answers
are what is recorded.

**Recording.** With `EMUSEN_RECORD_REGISTRATION=1` the two tests write their sections into the source tree's golden
instead of comparing; the diff is then reviewed as the record of what changed. A deliberate change to an engine's
answers (a new setting, a firmware policy, a new engine) re-records the golden in the same commit. On a mismatch the
assertion lists the lines gone and the lines new.

### 25.3 What was measured

**Determinism.** Three runs in a row, in compare mode, gave the golden byte for byte; the whole class takes about 4 s,
the child process included.

**Seeded faults.** One line of each section edited in the output copy of the golden: each test failed, naming exactly
the edited line as gone and the true answer as new. One code mutation, the one D1 is most likely to make: `CoreFactory`'s
generic branch with its `IsRegisteredEngine` guard removed, so that a registered engine discovery also lists is opened
by the generic adapter. Both sections failed, 38 and 33 lines gone (MoonRT's and MercuryRT's probes, cores, debug
targets and codecs turning into the adapter's). The source was restored and the test passed again.

**What it surfaced.** On `400f6087`, VenusRT asks for its sound unit's boot ROM as `required` and refuses a game
without it ("the sound unit's 64-byte boot ROM, file 1, was not given"). That contradicts the firmware policy decided
on 2026-10-03 (`EmuSen_Firmware.md` §0), which VenusRT's own work is bringing it to; the golden records the present
answers, and the change that brings VenusRT under the policy re-records its lines. *Re-recorded 2026-10-04*, when `WiseMan`'s
VenusRT with its own SPC700 boot program (`b82c8e4e`) was merged in: in both worlds the four VenusRT lines of the
request and the refusal became the 28 lines of a game loaded on the adapter, and no other line moved. The new lines show
one thing worth a later look: on both synthetic SNES images the adapter copies an older build's save into the SNES
folder at load (`BatterySave`'s copy-in, since the adapter opens file 0 for every game), and VenusRT, which tracks
battery changes, writes nothing at the first `SaveSram`; Venus (C#) writes the cartridge's 2 KiB. Neither is wrong by
§6.11, and §26's ports open file 0 only for a cartridge with battery RAM, as their oracles do. Venus (C#) gives the synthetic
LoROM, whose header byte for SRAM is the image's fill, 128 KiB of battery RAM; that is the image's doing, not a defect.

**What it cannot see.** A difference in the frame-end order (`EmuSen_NativeCores.md` §8.2, R4), anything a frame
produces, and an engine's debugger once halted. Each core's own frame-by-frame oracles and the adapter-against-shim
tests hold those.

## 26. MoonRT and MercuryRT through the v1 adapter, 2026-10-04: §13.1's row, the shims

### 26.1 What was done

§22 and §23 gave both libraries the v1 exports and left the shims loading the pre-stable ones. Each shim is now a
subclass of the generic adapter and loads its library through the v1 exports alone.

- **The adapter opened for subclassing.** `CoreEngine` is no longer sealed. A subclass may replace the registries
  (`Watches`, `FrameLog`, `Breakpoints`), the identity a frontend reads (`CoreName`, the picture's size, the frame rate,
  the state version), `SetButton`, `ReadSpace` and `WriteSpace`, `LoadRom` and `LoadState(Stream)`, `Cheats`, the
  `ICoreSettings` members and the halt pair. Two protected hooks carry what cannot be a whole member: `BatteryFile`,
  which opens battery file *n* of a game, and `AdvanceFrame`, which runs the machine to the frame's end and may halt
  instead, skipping the frame-end work. `RefreshCheats` is protected and `FrameBuffers` public. A failed `advance`
  now carries its detail word on `CoreRefusedException.Detail`, and `CoreLibrary.Export` resolves a core's own
  extensions (§4.7). Nothing in the generic engine's own answers changed but one, below.
- **The shared part of a port's shim** is `PortEngine` (`EmuSen/Cores/Native/PortEngine.cs`), with `PortLibrary` and
  `PortMachine` beside it. It holds what `NativeRtCore` held for both consoles, re-based on the adapter: the oracle's
  exception for a status (the console's band first, then the reproduced .NET faults), the state pre-checks before the
  bytes reach the core, the battery file as the oracle opens it, the whole pad sent on each change, the spaces by the
  C# names with a bus read refreshing the ROM patches first, the store a listening debugger is told of, the mirror's
  registries and the halts through the mirror's bridge. `PortLibrary` loads the library at its fixed name beside the
  assemblies, as the pre-stable loader did, so the switch variable and the engine notices keep their words ("turned
  off by EMUSEN_MOON_NATIVE=0", "… not found beside the assemblies"), and refuses one lacking the capabilities or
  extensions its shim calls.
- **The mirror's bridge** (`NativeDebugBridge`) now calls the v1 debug exports. Its logic is unchanged: the same
  tables, the first instruction checked unless resuming, the same drain order.
- **`MoonRtCore` and `MercuryRtCore`** are `PortEngine` subclasses holding their consoles' data: the pad bits and port
  rule, the space names and the reported store spaces, the C# header parse that gives C#'s exceptions, the state
  pre-check messages, the status tables (moved here from `MoonMachine` and `MercuryMachine`, which now read them), the
  identity a frontend reads, the mutes from the mirror's channels, and the mirror debugger. `MercuryRtCore` keeps the
  Model setting's semantics (a change before the first frame loads the game again) over the schema's create-time key,
  and sends C#'s mixer coefficients after every create (`Mercury_Native.md` §3.3).
- **The libraries.** `emusen-native` gained `core::exports::core_of` and `core_of_mut`, the core behind a v1 handle, for
  a core's own extensions. MoonRT exports `moonrt_core_rom_patch`, and MercuryRT `mercuryrt_core_rom_patch` and
  `mercuryrt_core_set_sample_rate`: the pre-stable extensions of the same purpose, over a v1 handle, sharing their
  bodies. The `_core_` infix mirrors `emusen_core_` against `emusen_native_`, and the names stay when the pre-stable
  ones go. The pre-stable exports are untouched, and `MoonMachine` and `MercuryMachine` with their machine and state
  tests still use them; they are deleted with VenusRT's (§13.5). `NativeRtCore` now has no subclass; it stays only for
  `RomPatchTriples` until then.

**What a frontend sees differently.** The shims now implement the adapter's neutral interfaces as well
(`ISnapshotCore` writing the full state, `IFrameSerial` the frame count, `IRepeatedRows` a repeat of 1, and for MoonRT
an empty `ICoreSettings`), and each answers through them what a frontend saw without them (§19.1). A refusal past the
pre-checks keeps the oracle's exception type and takes the core's own words (`last_error`), as §13.4 has it. An engine
whose library is not in use refuses at construction rather than at `LoadRom`, with the same exception type and words;
`CoreFactory` asks `Available` first, so a frontend never meets it.

### 26.2 Two defects found on the way

**MercuryRT did not raise `MACHINE_INFO` when a state from the other console changed its system.** §6.19 makes the
event the core's own. A machine running a Game Boy game as a Game Boy, loaded with a state made on the Game Boy Color,
becomes a Game Boy Color, and its machine info's `system` changes from `gb` to `gbc`; the host was not told, and read
`gb`. *Demonstrated* by `MercuryRtCoreAbiTests.A_state_from_the_other_console_tells_the_host_its_machine_info_changed`
against the unchanged library ("Expected gbc, Actual gb"). The machine's `load_state` now compares the console before
and after and emits the event; through the pre-stable path, which has no outbox, the emission is dropped. Both
directions pass. `MercuryRtCore` reads its `CoreName` (`GB` or `GBC`) from machine info in consequence, where the
pre-stable shim asked an extension.

**A registry handed to the adapter applied only from the next frame.** `CoreFactory.Bundle` hands a frontend's
registry to the engine; the adapter kept applying the previous one through `ApplyCheats` until a frame refreshed it,
where every C# core and the pre-stable shims apply the new one at once. *Demonstrated* by
`CoreAdapterTests.A_registry_handed_over_applies_before_the_next_frame` with the fix removed ("Expected 85, Actual 0").
For a core without `CHEAT_POKES` the hand-over now takes effect immediately; a core with it is unchanged, since its
pokes are split from the registry at the next refresh in any case.

### 26.3 The oracle, §13.1's row, measured 2026-10-04

| Oracle | Result |
|---|---|
| The state, machine and engine tests | the 984 WiseMan tests of the MoonRT, MercuryRT, Mercury, native-host, battery, cheat, debugger, engine-row, factory, adapter, discovery, ABI and registration areas pass unchanged, with the new defect test 985; no test file changed but the two that gained a case |
| The adapter against the shim | `MoonRtCoreAbiTests` and `MercuryRtCoreAbiTests` pass: the generic adapter and the shim, now both on the v1 exports, give the same picture and sound every frame and the same state |
| The bench's state hashes | one per game in all eighty runs of each bench, before and after, the same values (`0ED799A150F20023` for Super Mario Bros. and so on, as `moonbench` and `mercbench` print them) |
| The kit | C1–C15 pass on MoonRT's four games and MercuryRT's four, 600 frames each |
| §25's registration-equivalence test | unchanged, both worlds |
| The crates | `emusen-native` 51 tests, MoonRT's 6 and MercuryRT's 28 pass; `export_check.py` passes on both libraries (46 and 45 `emusen_core_` symbols) |

MercuryRT's crate tests had two failures on `WiseMan` before this step, both stale fixtures from state version 8
(`Mercury_Native.md` §11); they were corrected first, in a commit of their own.

### 26.4 The timing

Measured 2026-10-04 with `moonbench` and `mercbench`, built against `f1b23338` ("before": the pre-stable shims) and
against this step ("after"), both by cargo's release profile from the crates' directories. Ten rounds under the bench
lock, the two builds alternated within each game and the order swapped each round. **The load:** the one-minute
average was 2.76–6.90 across the runs, from other work on the machine; this is not a quiet machine.

| Game | Before, median p50 ms | After | Change |
|---|---|---|---|
| Super Mario Bros. | 1.109 | 1.098 | −0.99% |
| The Legend of Zelda | 0.919 | 0.923 | +0.38% |
| Super Mario Bros. 3 | 1.337 | 1.348 | +0.82% |
| Mike Tyson's Punch-Out!! | 1.236 | 1.261 | +1.98% |
| Tetris | 0.4455 | 0.4503 | +1.07% |
| Kirby's Dream Land | 0.4966 | 0.5026 | +1.20% |
| The Legend of Zelda: Link's Awakening | 0.4126 | 0.4143 | +0.41% |
| Pokémon Yellow | 0.5526 | 0.5516 | −0.18% |

The geometric mean is +0.58%, inside P1's band. Punch-Out!! is outside it by about a point, as it was by 0.31 in
§22.2's loaded measurement, and Tetris and Kirby's Dream Land by about a fifth of one. The first five rounds alone gave
+1.12% and the second five +0.56%, which is the size of the noise at this load. The adapter does a little more per
frame than the pre-stable shim did (an event drain, a `frame_info` before each picture copy, the picture copied by
rows), each a call or a copy of a few microseconds at most against frames of 0.4–1.4 ms. P1 holds for the geometric
mean; a quiet run remains owed for the per-game figures, as it does for §22.2's.

### 26.5 Not done here

- **One bridge.** VenusRT's branch builds its own v1 debug bridge for the adapter (processors beyond the first, the
  engine's own registries). The mirror's bridge here differs from it in whose registries it feeds, not in what it
  sends; when both are on `WiseMan` they can become one class with the registries as parameters.
- **The pre-stable set's deletion** (§13.5), with `NativeRtCore`, `MoonMachine` and `MercuryMachine` and the moving of
  their machine and state tests onto the v1 handle, waits for VenusRT to drop its own.

---

## 27. Cores in development, 2026-10-04

### 27.1 The rule

A core may be built and tested long before it is fit to offer: Nephrite's stage 0 is a stub that recognises an image
and shows a black picture (`Nephrite_Native.md` §1). Discovery would offer it for any file its systems claim. *Decided
2026-10-04:* such a core is marked **in development** until its gate, and is then never offered to a player, while the
tests and the conformance kit run it as any other core. The mark names no core in code; it is a property of a build.

### 27.2 The mechanism

- **`RustCores.props`**: a crate's row carries `<InDevelopment>true</InDevelopment>` (an item definition defaults it to
  false). Removing the line is the whole of offering the core.
- **The sidecar**: the build runs `emusen-core-conform --sidecar LIB --development` for such a crate, which adds
  `"development": true` to the sidecar of §19.2. The field is the host's, outside core info, so the library, its info
  and the kit are unchanged, and an older host that ignores the field lists the core as before.
- **Discovery**: `CoreDiscovery` skips a development sidecar unless `IncludeDevelopment` holds: with
  `EMUSEN_DEVELOPMENT_CORES=1` in the environment, for a developer trying the core in a frontend, or when a test calls
  `CoreDiscovery.UseDevelopment(true)`. Skipped, the core claims no extension, so `CoreFactory.IsSupported` and
  `Create` do not reach it, and no engine row lists it.
- **Publish**: the library and its sidecar are copied into the build output, beside the tests, but marked
  `CopyToPublishDirectory="Never"`, and a publish for another platform neither takes nor asks for a prebuilt copy.
- **CI**: `rust-cores.yml` passes `--development` for a crate whose matrix row says `development: true`.
- ~~**The library's shelves** are unaffected by design: Mistress lists the extensions of `CoreCatalog`'s console rows,
  and a system reached only through discovery has no row (`Nephrite_Plan.md` §8.3).~~ *Since 2026-10-06 a system only a
  discovered engine runs has a row made from its system pack (`EmuSen_Settings_Reference.md` §4.92); a development core
  is still found only when asked for, so its shelf shows only then.*

### 27.3 What was tested (2026-10-04)

`CoreDiscoveryTests.A_core_in_development_is_listed_only_when_asked_for`, on the test core: a development sidecar is
not listed, its extension is unsupported and `Create` refuses it; asked for, it is listed and supported; the sidecar
rewritten without the mark is listed either way. `NephriteTests.A_player_is_not_offered_the_core_in_development`: the
built sidecar carries the mark, and with development cores not asked for, no Genesis, Sega CD or 32X extension is
supported, claimed by the catalogue or listed by Mistress's library, and `Create` refuses a `.gen` file. The kit's
`a_sidecar_carries_the_librarys_own_descriptors_and_hash` checks the field is written only when asked. **Measured:** a
Release publish of `EmuSen.csproj` holds the other four libraries and their sidecars and no `libnephrite.so`; the
registration golden of §25 is byte-identical to its state before Nephrite existed, since a player's discovery no
longer sees the core.

*Amended 2026-10-07: the Genesis is offered to players* (`Nephrite_Plan.md` §7). Nephrite's row carries no mark, the
registration golden is re-recorded with the Genesis's lines, and the Nephrite test is now
`NephriteTests.A_player_is_offered_the_genesis_and_not_the_sega_cd_or_the_32x`. The core-level mark is exercised by
the test core in `CoreDiscoveryTests` and by a copy of Nephrite's library with a development sidecar in
`FirmwareOverviewTests`; the Sega CD and the 32X stay marked by §27.4.

### 27.4 A system in development, 2026-10-06

A core may run one of its systems before the others: Nephrite runs the Genesis at stage 6, and its Sega CD and 32X
are stubs until stages 10 and 12. The core-level mark of §27.2 cannot say that, so a system entry of core info (§6.3)
may carry `"development": true`.

- **It is additive** under §4.2: an optional field of the info's system entry, in `info.schema.json`. Absent means
  false, and a core writes it only when it is true, so every other core's info is byte-identical. A host that does not
  know the field behaves as before and offers every system the core lists.
- **What a host does with it**: `CoreDiscovery.Offered` keeps such a system's extensions unclaimed, so
  `CoreFactory.IsSupported` and `Create` do not reach it, unless development cores are asked for (§27.2), where a
  developer can still open its files. Its console gets no Graphics Settings tab or firmware page row even then
  (`CoreCatalog.DiscoveredConsoles`, `FirmwareOverview`), because those say what a player can run. Removing the mark
  from the core's info is the whole of offering the system.
- **The kit** (C2) lists each system in development, and fails a core whose system in development shares an extension
  with a system it offers, since a host routing by extension would offer it through that one
  (`systems_in_development`, `a_system_in_development_shares_no_extension_with_an_offered_one`).
- **Tested**: `NephriteTests.A_shipped_core_offers_only_the_systems_its_info_says_run`, on Nephrite's library with its
  sidecar written as shipped: `.md` supported, `.iso` and `.32x` not; with development cores asked for, both
  supported and still no Sega CD or 32X tab.

---

## Appendix A. The draft header

*Superseded, 2026-10-03, by the committed header, `EmuSen/Cores/Shared/emusen-native/include/emusen_core.h`, which is
the normative artefact (§5.1). It keeps this draft's shape and adds: every function type; the flag, debug and
battery constants the exports' meanings depend on; explicit values for every enumeration constant; `state_save` on a
non-const machine, since saving settles the core's threads first; and the rules of §6.18 and §6.19 as comments. The
draft is kept as the record of what was reviewed.*

Compiled cleanly on 2026-10-01 as C99 (`gcc -std=c99 -Wall -Wextra -pedantic -Werror`), as C11 under clang and as C++17,
with `_Static_assert`s confirming `emusen_frame_info` at 56 bytes with `serial` at offset 40, `emusen_create_params` at
88, and `emusen_file` and `emusen_event` at 24 each. The draft is the shape to be reviewed, not the release: the full
function-type list of §5.2 and the `@since` tags are added when it is committed beside the crate.

```c
/* emusen_core.h - the EmuSen core ABI, version 1 (draft of 2026-10-01).
 *
 * The normative text is EmuSen_CoreAPI.md; this header is its machine-checked half. Every
 * export is `emusen_core_<name>`, C calling convention, no callbacks into the host.
 *
 * Rules every export follows (EmuSen_CoreAPI.md section 6.0):
 *  - A negative int32_t/int64_t return is a status (section 6.15); zero or more is success.
 *  - Copy out: a call that fills a buffer takes (out, len) and returns the whole length;
 *    a null `out` asks only the length. No pointer into the core's memory is ever returned.
 *  - The host owns every buffer. A pointer passed in is valid for that call only.
 *  - A struct crossing the boundary begins with `size`, the bytes the caller allocated.
 *    The callee reads and writes only min(size, what it knows).
 *  - Enumerations are open: an unknown value is refused with EMUSEN_NOT_SUPPORTED or
 *    ignored where this header says so, never undefined.
 *  - One thread at a time per machine; library-level calls are safe from any thread.
 */
#ifndef EMUSEN_CORE_H
#define EMUSEN_CORE_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define EMUSEN_CORE_ABI_MAJOR 1u
#define EMUSEN_CORE_ABI_MINOR 0u
#define EMUSEN_CORE_ABI_VERSION ((EMUSEN_CORE_ABI_MAJOR << 16) | EMUSEN_CORE_ABI_MINOR)

/* An opaque machine. Several may exist at once, from one library. */
typedef struct emusen_machine emusen_machine;

/* ---- Status codes (section 6.15). Bands are fixed for the life of major 1. ---------------- */
#define EMUSEN_NULL               (-1)   /* a null machine or required buffer */
#define EMUSEN_TRUNCATED          (-2)   /* a state ended early */
#define EMUSEN_FOREIGN            (-3)   /* not this core's state */
#define EMUSEN_STATE_VERSION      (-4)   /* a state version this core does not read */
#define EMUSEN_BAD_STRING         (-5)
#define EMUSEN_BUFFER_TOO_SMALL   (-7)
/* -9 .. -255: the core's own band; emusen_core_status_text names them. */
#define EMUSEN_CORE_BAND_FIRST    (-9)
#define EMUSEN_CORE_BAND_LAST     (-255)
#define EMUSEN_NOT_SUPPORTED      (-256)
#define EMUSEN_NO_SUCH_SPACE      (-257)
#define EMUSEN_READ_ONLY          (-258)
#define EMUSEN_UNKNOWN_SETTING    (-259)
#define EMUSEN_BAD_SETTING        (-260)
#define EMUSEN_NO_SUCH_PORT       (-261)
#define EMUSEN_BAD_FILE           (-262)
#define EMUSEN_BAD_STRUCT         (-263)  /* a struct whose size is below its v1.0 size */
#define EMUSEN_BAD_IMAGE          (-264)  /* a generic refusal of the image, for cores without a finer code */
/* -320 .. -383: a .NET exception reproduced by a port, -320 less its kind (ports only). */
#define EMUSEN_FAULT_BASE         (-320)

/* ---- Capability bits (section 6.1). Open: a host ignores bits it does not know. ------------ */
#define EMUSEN_CAP_RESET            (UINT64_C(1) << 0)
#define EMUSEN_CAP_PRESENT          (UINT64_C(1) << 1)
#define EMUSEN_CAP_SNAPSHOT         (UINT64_C(1) << 2)
#define EMUSEN_CAP_AXES             (UINT64_C(1) << 3)
#define EMUSEN_CAP_AUDIO_PEEK       (UINT64_C(1) << 4)
#define EMUSEN_CAP_MUTES            (UINT64_C(1) << 5)
#define EMUSEN_CAP_SETTINGS         (UINT64_C(1) << 6)
#define EMUSEN_CAP_PHASES           (UINT64_C(1) << 7)
#define EMUSEN_CAP_FRAME_SERIAL     (UINT64_C(1) << 8)
#define EMUSEN_CAP_ROW_REPEAT       (UINT64_C(1) << 9)
#define EMUSEN_CAP_BATTERY_DIRTY    (UINT64_C(1) << 10)
#define EMUSEN_CAP_ROM_PATCHES      (UINT64_C(1) << 11)
#define EMUSEN_CAP_DEBUG            (UINT64_C(1) << 12)
#define EMUSEN_CAP_DEBUG_STACK      (UINT64_C(1) << 13)
#define EMUSEN_CAP_CHEAT_POKES      (UINT64_C(1) << 14)
#define EMUSEN_CAP_SETTING_NOTES    (UINT64_C(1) << 15)
#define EMUSEN_CAP_DEBUG_REGISTERS  (UINT64_C(1) << 16)
#define EMUSEN_CAP_DEBUG_DISASSEMBLE (UINT64_C(1) << 17)
/* Bits 18..47 are for later minors; 48..63 are never assigned, for a core's private use in tests. */

/* ---- Open enumerations ----------------------------------------------------------------- */
enum emusen_pixel_format {          /* section 6.6 */
    EMUSEN_PIXEL_RGBA8888 = 0,      /* bytes R, G, B, A; every host accepts it */
    EMUSEN_PIXEL_BGRA8888 = 1,      /* bytes B, G, R, A */
    EMUSEN_PIXEL_RGB565   = 2       /* little-endian 16-bit words */
};

enum emusen_state_kind {            /* section 6.9 */
    EMUSEN_STATE_FULL     = 0,      /* the save state; written after settling */
    EMUSEN_STATE_SNAPSHOT = 1       /* SNAPSHOT: written without waiting on the core's threads */
};

enum emusen_event_kind {            /* section 6.19; a host skips kinds it does not know */
    EMUSEN_EVENT_AUDIO_RATE   = 1,  /* a: the new rate in Hz */
    EMUSEN_EVENT_GEOMETRY     = 2,  /* a: width, b: height of the picture now produced */
    EMUSEN_EVENT_STATE_SIZE   = 3,  /* a: the new size of kind b */
    EMUSEN_EVENT_BATTERY      = 4,  /* a: the battery file that changed */
    EMUSEN_EVENT_LOG          = 5,  /* a: records waiting in emusen_core_log_drain */
    EMUSEN_EVENT_MACHINE_INFO = 6   /* the machine descriptor changed; read it again */
};

/* The canonical controls a core maps its pad bits onto; append-only (section 6.8). */
enum emusen_control {
    EMUSEN_CONTROL_B = 0, EMUSEN_CONTROL_Y, EMUSEN_CONTROL_SELECT, EMUSEN_CONTROL_START,
    EMUSEN_CONTROL_UP, EMUSEN_CONTROL_DOWN, EMUSEN_CONTROL_LEFT, EMUSEN_CONTROL_RIGHT,
    EMUSEN_CONTROL_A, EMUSEN_CONTROL_X, EMUSEN_CONTROL_L, EMUSEN_CONTROL_R,
    EMUSEN_CONTROL_L2, EMUSEN_CONTROL_R2, EMUSEN_CONTROL_L3, EMUSEN_CONTROL_R3
};

enum emusen_axis {
    EMUSEN_AXIS_LEFT_X = 0, EMUSEN_AXIS_LEFT_Y, EMUSEN_AXIS_RIGHT_X, EMUSEN_AXIS_RIGHT_Y,
    EMUSEN_AXIS_LEFT_TRIGGER, EMUSEN_AXIS_RIGHT_TRIGGER
};

/* ---- Structures ------------------------------------------------------------------------ */
typedef struct emusen_file {   /* a file the host read for this game (section 6.2) */
    uint32_t size;
    uint32_t which;                 /* 0 the battery save; others from machine or firmware info */
    const uint8_t *data;
    size_t len;
} emusen_file;

typedef struct emusen_create_params {
    uint32_t size;
    uint32_t host_abi_version;      /* EMUSEN_CORE_ABI_VERSION the host was built with */
    const uint8_t *image;
    size_t image_len;
    const uint8_t *settings;        /* UTF-8 "key=value" lines: create-time and run-time keys */
    size_t settings_len;
    const emusen_file *files;
    size_t file_count;
    size_t file_size;               /* sizeof(emusen_file) as the host knows it */
    uint64_t pixel_formats;         /* bit n: format n accepted; bit 0 always set */
    uint8_t *error;                 /* optional: a refusal's UTF-8 detail, copied up to error_len */
    size_t error_len;
} emusen_create_params;

typedef struct emusen_frame_info {
    uint32_t size;
    uint32_t format;                /* enum emusen_pixel_format */
    int32_t width;
    int32_t height;
    int32_t stride;                 /* bytes per row in frame_copy's output */
    int32_t row_repeat;             /* ROW_REPEAT: each row is shown this many times */
    uint32_t flags;                 /* bit 0: the last present walked the picture */
    uint32_t aspect_num;            /* display aspect of this picture, 0/0 for square pixels */
    uint32_t aspect_den;
    uint32_t reserved;
    int64_t serial;                 /* FRAME_SERIAL: moves only when the picture does */
    int64_t bytes;                  /* frame_copy's whole length */
} emusen_frame_info;

typedef struct emusen_event {
    uint32_t size;
    uint32_t kind;                  /* enum emusen_event_kind */
    int64_t a;
    int64_t b;
} emusen_event;

/* ---- Library-level calls: no machine, any thread --------------------------------------- */
uint32_t emusen_core_abi_version(void);                         /* (major << 16) | minor */
uint64_t emusen_core_capabilities(void);
int64_t  emusen_core_info(uint8_t *out, size_t len);            /* JSON, section 6.3 */
int64_t  emusen_core_settings_schema(uint8_t *out, size_t len); /* JSON, section 6.13 */
int64_t  emusen_core_firmware_for(const uint8_t *image, size_t image_len, uint8_t *out, size_t len);
int64_t  emusen_core_status_text(int32_t status, uint8_t *out, size_t len);
int32_t  emusen_core_set_crash_log(const char *path);
int64_t  emusen_core_log_drain(emusen_machine *machine, uint8_t *out, size_t len);

/* ---- Lifecycle ------------------------------------------------------------------------- */
emusen_machine *emusen_core_create(const emusen_create_params *params, int32_t *status);
int32_t emusen_core_free(emusen_machine *machine);
int32_t emusen_core_reset(emusen_machine *machine);                            /* RESET */
int64_t emusen_core_machine_info(const emusen_machine *machine, uint8_t *out, size_t len);
int64_t emusen_core_last_error(const emusen_machine *machine, uint8_t *out, size_t len);

/* ---- A frame --------------------------------------------------------------------------- */
int32_t emusen_core_advance(emusen_machine *machine, uint64_t *detail);
int32_t emusen_core_present(emusen_machine *machine);                          /* PRESENT */
int32_t emusen_core_set_options(emusen_machine *machine, uint32_t flags);      /* bit 0 skips rendering */
int64_t emusen_core_frame_count(const emusen_machine *machine);
int64_t emusen_core_phases(const emusen_machine *machine, int64_t *out, size_t len); /* PHASES */
int64_t emusen_core_events(emusen_machine *machine, emusen_event *out, size_t count, size_t event_size);

/* ---- Picture --------------------------------------------------------------------------- */
int32_t emusen_core_frame_info(const emusen_machine *machine, emusen_frame_info *out);
int64_t emusen_core_frame_copy(const emusen_machine *machine, uint8_t *out, size_t len);

/* ---- Sound: interleaved stereo int16 --------------------------------------------------- */
int32_t emusen_core_audio_rate(const emusen_machine *machine);
int64_t emusen_core_audio_buffered(const emusen_machine *machine);
int64_t emusen_core_audio_drain(emusen_machine *machine, int16_t *out, size_t len, int64_t max_frames, int32_t *rate);
int32_t emusen_core_set_audio_limit(emusen_machine *machine, uint64_t samples);
int64_t emusen_core_audio_peek(const emusen_machine *machine, int16_t *out, size_t len); /* AUDIO_PEEK */
int32_t emusen_core_set_mutes(emusen_machine *machine, uint32_t mask);                  /* MUTES */

/* ---- Input ----------------------------------------------------------------------------- */
int32_t emusen_core_set_buttons(emusen_machine *machine, uint32_t port, uint32_t mask, uint32_t changed);
int32_t emusen_core_set_axis(emusen_machine *machine, uint32_t port, uint32_t axis, double value); /* AXES */

/* ---- State ----------------------------------------------------------------------------- */
int64_t emusen_core_state_size(const emusen_machine *machine, uint32_t kind);
int64_t emusen_core_state_save(emusen_machine *machine, uint32_t kind, uint8_t *out, size_t len);
int32_t emusen_core_state_load(emusen_machine *machine, const uint8_t *data, size_t len);
int64_t emusen_core_state_layout(const emusen_machine *machine, uint32_t kind, uint8_t *out, size_t len);

/* ---- Memory spaces, by the ids machine_info lists -------------------------------------- */
int64_t emusen_core_space_size(const emusen_machine *machine, uint32_t space);
int64_t emusen_core_space_read(emusen_machine *machine, uint32_t space, uint32_t address, uint8_t *out, size_t len);
int64_t emusen_core_space_write(emusen_machine *machine, uint32_t space, uint32_t address, const uint8_t *data, size_t len);

/* ---- Battery --------------------------------------------------------------------------- */
int64_t emusen_core_battery(const emusen_machine *machine, uint32_t which, uint8_t *out, size_t len, uint32_t *flags);
int32_t emusen_core_battery_saved(emusen_machine *machine, uint32_t which);

/* ---- Cheats: resolved by the host's codecs --------------------------------------------- */
int64_t emusen_core_set_rom_patches(emusen_machine *machine, const uint32_t *triples, size_t count); /* ROM_PATCHES */
int64_t emusen_core_set_cheat_pokes(emusen_machine *machine, const uint32_t *quads, size_t count);   /* CHEAT_POKES */

/* ---- Settings, keys from the schema ---------------------------------------------------- */
int32_t emusen_core_set_settings(emusen_machine *machine, const uint8_t *text, size_t len);          /* SETTINGS */
int64_t emusen_core_setting_notes(const emusen_machine *machine, uint8_t *out, size_t len);          /* SETTING_NOTES */

/* ---- Debug (section 6.14) -------------------------------------------------------------- */
int32_t emusen_core_debug_set(emusen_machine *machine, uint32_t flags, int32_t depth_target, int32_t depth_guard);
int32_t emusen_core_debug_set_stack(emusen_machine *machine, const uint32_t *pairs, size_t count);   /* DEBUG_STACK */
int32_t emusen_core_debug_set_breakpoints(emusen_machine *machine, uint32_t processor, const int32_t *pairs, size_t count);
int32_t emusen_core_debug_set_ranges(emusen_machine *machine, uint32_t kind, const uint32_t *triples, size_t count);
int32_t emusen_core_debug_run_frame(emusen_machine *machine, uint32_t flags, uint32_t *processor, uint64_t *pc, uint64_t *detail);
int64_t emusen_core_debug_writes(emusen_machine *machine, uint32_t *out, size_t len);
int64_t emusen_core_debug_calls(emusen_machine *machine, uint32_t *out, size_t len);
int64_t emusen_core_debug_profile(emusen_machine *machine, int64_t *out, size_t len);
int64_t emusen_core_debug_coverage(emusen_machine *machine, uint32_t processor, uint8_t *out, size_t len, int64_t *recorded);
int64_t emusen_core_debug_counters(emusen_machine *machine, int64_t *out, size_t len);
int32_t emusen_core_debug_pc(const emusen_machine *machine, uint32_t processor, uint64_t *pc);
int64_t emusen_core_debug_registers(const emusen_machine *machine, uint32_t processor, int64_t *out, size_t len); /* DEBUG_REGISTERS */
int64_t emusen_core_debug_disassemble(emusen_machine *machine, uint32_t processor, uint32_t space, uint32_t address,
                                      uint32_t count, uint8_t *out, size_t len);                          /* DEBUG_DISASSEMBLE */

/* ---- Function types, one per export, so a host or a check can name each signature ------ */
typedef uint32_t (*emusen_core_abi_version_fn)(void);
typedef emusen_machine *(*emusen_core_create_fn)(const emusen_create_params *, int32_t *);
typedef int32_t (*emusen_core_advance_fn)(emusen_machine *, uint64_t *);
typedef int32_t (*emusen_core_frame_info_fn)(const emusen_machine *, emusen_frame_info *);
typedef int64_t (*emusen_core_audio_drain_fn)(emusen_machine *, int16_t *, size_t, int64_t, int32_t *);
/* ... and one typedef for every other export in the committed header; this draft shows five. */

#ifdef __cplusplus
}
#endif
#endif
```

## Appendix B. MoonRT's info, as it would be written

```json
{
  "abi": "1.0",
  "id": "moonrt",
  "name": "MoonRT",
  "display_name": "MoonRT (Rust)",
  "version": "0.4.0",
  "license": "GPL-3.0-or-later",
  "description": "A Rust port of Moon, the project's NES core, exact against it.",
  "capabilities": ["RESET", "MUTES", "ROM_PATCHES", "DEBUG", "DEBUG_STACK"],
  "deterministic": true,
  "accuracy": { "measured_with": "defaults", "suite": "Moon's C# core and the blargg ROMs", "notes": "" },
  "systems": [{
    "id": "nes",
    "name": "Nintendo Entertainment System",
    "extensions": [".nes"],
    "regions": ["ntsc"],
    "controllers": [{
      "id": "nes.pad", "label": "Controller", "ports": [0, 1],
      "buttons": [
        { "bit": 0, "control": "A", "label": "A" },      { "bit": 1, "control": "B", "label": "B" },
        { "bit": 2, "control": "Select", "label": "Select" }, { "bit": 3, "control": "Start", "label": "Start" },
        { "bit": 4, "control": "Up", "label": "Up" },    { "bit": 5, "control": "Down", "label": "Down" },
        { "bit": 6, "control": "Left", "label": "Left" }, { "bit": 7, "control": "Right", "label": "Right" }
      ],
      "axes": []
    }],
    "firmware": []
  }]
}
```

The bit order is MoonRT's (`EmuSen_NativeCores.md` §4.5: A, B, Select, Start, Up, Down, Left, Right). The version
string, licence and description are placeholders for the review; the crate's own metadata supplies them.
