# EmuSen — DianaSh: a standalone shell, its command modules, and the surfaces around it

*This revision: the eleventh, 2026-10-16: breakpoint conditions and logpoints restored on every core, checked when
they are set and read at the hit, with the session service at 1.2 for the refusal (§13.12); Q28 and Q29 (§16.3). The
tenth, the same day: S5 built, EmuSen's four command modules in Rust over the session service,
which went to version 1.1 for what they needed, the host API to 1.1 for a service per session, `nano` as the first
program the shell runs, and two defects the port found shown and fixed in both shells, a root reached through a link
and a trace of no instructions (§13.11); Q25 to Q27 (§16.2). The ninth, the same day: S6's state 2 reached ahead of S5, by decision, the Rust shell the default and
the C# its failover until Windows and macOS are proven, with the facade's cost sampled and cut (§13.10). The eighth,
2026-10-16: Q18 to Q24 decided as recommended, and Q18, Q19, Q20's page and Q21 built
in both shells (§13.9). The seventh, 2026-10-15: S3 built, the Rust shell core in `Platform/dianash/` with its builtins,
sessions and host API (`dianash.h`, stable, with its baseline), every C# command still running under it through a
bridge, the facade choosing it under `EMUSEN_DIANASH_NATIVE=1`, and the standalone `dianash` (§13.7); S4 built,
`dianash.text` in Rust, all sixteen commands (§13.8); Q15's case-insensitive root comparison corrected in both shells
(§13.7); seven questions from S2 to S4 (§16.1). The sixth, 2026-10-11: S2 built, Crystal's assembly beneath the shell, the session service's header
`emusen_session.h` and its C# provider, held to `IDebugTarget` by a parity class on the fake target and every core (§13.6).
The fifth, the same day: S1 built, the C# shell reorganised without changing what it prints (§13.5). The fourth, 2026-10-10: §13.4's sandbox escapes, finding 10, fixed. The third, the same day: S0
built, the golden corpus, its harness and what it recorded (§13.4). The
second, the same day: the seventeen questions of §16 decided, each as recommended, and the runtime named Crystal (Q1).
The first, the same day: a design, with nothing built: the three decisions of §0.1, what the shell is
today, measured from the code (§1), and the five surfaces that are to replace what the code does now: the shell core
(§4), the command module ABI (§5, §6), commands run as programs (§7), the session service that modules call (§9) and
the host API by which a frontend embeds the shell (§10). Their stability policy (§11), the platforms (§12), the
migration (§13) and the questions it leaves (§16) follow, and §17 answers what `EmuSen_Surfaces.md` asks of this
design. Every claim about the code is cited to a file, read on `rust-platform` at `96c3f563`, whose code `d206c3aa`
leaves unchanged. Claims marked **measured** were measured on 2026-10-10 on the development desktop;
claims marked **argued** are reasoning that a later step must prove, and §15's predictions say how.*

Companion docs: `EmuSen_CoreAPI.md` (the core ABI v1, whose rules every surface here follows, and §8 there, which
this page revises, §0.3), `EmuSen_NativeCores.md` (the conventions of a C interface in this project),
`EmuSen_RustPlatform.md` §2, §3 and §7.3 (the platform library and its C layer), `EmuSen_Stack.md` §6 (the plan to
bring the runtime above the cores to Rust), `EmuSen_Debugging_Tools_Reference_v5.md` §3 (the shell and every command,
as built in C#), `EmuSen_Galaxia.md` §3 (the tree the shell sees, and the macOS bundle).

**The audit of the project's surfaces** was written beside this page and is recorded in `EmuSen_Surfaces.md`. Where
the two meet on the shell's surface, this page decides; §17 answers the five points that page's §6.3 puts to this
design, and names each place where the two disagree, with the recommendation.

---

## 0. The decisions, and the design in brief

### 0.1 Decided 2026-10-10

1. **DianaSh is a standalone, cross-platform Unix shell, at the heart of EmuSen.** It stands alone: on Linux, macOS
   and Windows it is a plain terminal shell, with no emulator in it.
2. **Commands are modules of their own, which the shell calls upon; they are not code built into it.** Today all 83
   commands are compiled into the shell's assembly and listed by hand in `DianaShInterpreter.CreateDefault`; every
   command's `Execute` takes the emulator's `IDebugTarget`; and every manual page is in one file of 2,922 lines,
   `ManPages.cs` (§1).
3. **DianaSh has a stable, versioned ABI and API, as the cores have** (`EmuSen_CoreAPI.md`, core ABI v1). The aim is
   clean, clear and well-defined surfaces: every boundary is a written interface with stated rules, and nothing
   reaches across it otherwise.

### 0.2 The design in one paragraph

DianaSh becomes a Rust shell core, a library with a small terminal program over it, that knows no emulator type
(§4). It reads a line, parses it as today's language, and runs each command it names. A command is one of three
kinds, all called the same way: a **builtin**, for the few commands that are the language or change the shell's own
state (`cd`, `export`, `source`, `history`, `su`); a **module**, a shared library that exports the command module ABI
`dianash_module_*` and is described by a manifest the shell reads without loading it (§5, §6); or a **program**, an
ordinary executable run as a process (§7). A module that must reach a running core asks the shell for a **service**
by name and version, and the shell hands it the table its host provided, without reading it. EmuSen's one service is
the session service, `emusen.session@1`, a C function table owned by Crystal and not by the shell: memory
spaces, processors and registers, disassembly, breakpoints and the rest of the debugger, cheats, telemetry, states,
and requests to the frontend (§9). A frontend embeds the shell through the host API `dianash_*` (§10), and provides
the services. All three surfaces, the host API, the module ABI and the session service, are versioned as the core ABI
is: a major that is expected never to change, a minor that grows by addition, structs that carry their size, and a
baseline guarded in CI (§11). The C# shell is held to the Rust one by golden outputs recorded from WiseMan before
anything moves, and the two are chosen between by `EMUSEN_DIANASH_NATIVE`, as the platform's components were (§13).

### 0.3 What this revises

- **`EmuSen_CoreAPI.md` §0.2 item 2, §3 and §8: "DianaSh is the framework's runtime."** The runtime that page designs,
  which discovers cores, runs sessions and owns the debugger, the cheats, states, settings and input routing, is kept
  whole. What changes is where it lives: it is no longer the shell. The shell is a client of the runtime, as Mistress
  is, and reaches it only through the session service (§3, §9). §8.2's "One assembly, not two", which put the runtime
  and the shell in one assembly because "the runtime and the shell are used together by every client that exists",
  no longer holds: decision 1 makes a client that is the shell without the runtime. The runtime is a component of its
  own, named **Crystal** (decided 2026-10-10, §16 Q1), and this page calls it Crystal from here on. The name is
  reused: an earlier `EmuSen.Crystal`, a shared timing assembly, was deleted on 2026-08-05, and the pages that mention
  it (`Man pages/Old/EmuSen_Crystal_Scheduler.md`, `EmuSen_Stack.md`) describe that deleted assembly, not this
  component.
- **`EmuSen_CoreAPI.md` §15 Q13, the runtime's folder `DianaSh/Sys/`.** A folder inside the shell's tree for a
  component the shell must not contain is the tangle decision 3 forbids. `Sys/Systems/` (the system packs) is
  Crystal's, and moves with it (§13). The same holds of `EmuSen_Surfaces.md` §7's S5 and its Q2, which place the engine
  SPI, the native host, the systems table and the display shapes in DianaSh; §17.2 gives the recommendation.
- **`EmuSen_CoreAPI.md` §9.2, a `diana.h` for the frontend API.** The frontend API stays Crystal's; its session
  operations are what §9 offers to modules, so the two share one definition (§9.1). The shell's own C interface is a
  different header, `dianash.h` (§10).
- **`EmuSen_Debugging_Tools_Reference_v5.md` §3.3 and §3.3b: one `IDianaShCommand` with an ambient `IDebugTarget`, and
  `HostAction` for control flow.** Both are retired by the module ABI (§6.2, §6.6).
- **`EmuSen_Stack.md` §6, "DianaSh would become a Rust program, the platform's runtime and shell".** It becomes two
  Rust components, the shell and Crystal, with a written interface between them.

---

## 1. What there is (measured 2026-10-10)

### 1.1 The assembly

`EmuSen.DianaSh` holds 142 C# files, 16,867 lines, in a Unix hierarchy:

| Folder | Files | Lines | What it holds |
|---|---|---|---|
| `Bin/` | 88 | 10,357 | the interpreter (798), its scheduler (69), `Program.cs` (37), 83 commands in 84 files under `Bin/Commands/` (`Unix/` 2,532 lines, `EmuSen/` 3,999), `ManPages.cs` (2,922) |
| `Lib/` | 22 | 2,349 | the lexer (339), parser (285) and AST (103); `IDebugTarget` and its value types (223); `IDianaShCommand`, `DianaShResult`, `HostAction`; the expression evaluator (325); `RuntimeStats` (217); `Cauldron/`, the telemetry contracts (6 files) |
| `Var/` | 22 | 3,457 | the debugger's ten registries; the cheat engine (`CheatRegistry`, `CheatDatabase` with its installer, pruner and importer, `ChtFile`, `CheatWrite`); `CommandHistory`, the `tmux` sessions; `FrameRecorder`; `DebugCpu` |
| `Etc/` | 4 | 179 | the sandbox, the account registry (`DianaShUserRegistry`), the logging master switch, the suspended commands |
| `Dev/` | 1 | 141 | the console line reader |
| `Sys/` | 5 | 384 | the system packs: the SNES's and the Genesis's cheat formats |

Its public surface is 217 public types and 1,300 lines that begin `public`. It references Galaxia, and is referenced
by `EmuSen` (the cores), Serenity, Mistress, Hotaru, Pharaoh and WiseMan.

**The shell proper** (the lexer, parser, AST, interpreter, scheduler, sandbox, users, logging switch, suspensions,
line reader, history and sessions) is 2,065 lines of the 16,867. Everything else in the assembly is either a command or
something a command or a core needs from the emulator.

### 1.2 The commands

There are **83 command classes**, in 84 files: `Bin/Commands/Unix/` holds 37 (36 files, `TestCommand.cs` holding
`test` and `[`), and `Bin/Commands/EmuSen/` holds 46 (48 files, two of them helpers: `DebugCommandHelpers.cs` and
`SnapshotStore.cs`). `CreateDefault` lists 81 of them by hand (`DianaShInterpreter.cs:68`–`:163`); the other two,
`core` and `state`, are constructed by the frontends with delegates of their own. Besides the classes:

- **Six builtins live in the interpreter's `Dispatch`** (`DianaShInterpreter.cs:519`–`:582`): `help`, `man`,
  `summary`, `export`, `unset` and `source` (with `.`). `summary` is the emulator's: it returns
  `_target?.GetSummaryText()`.
- **Five aliases are mapped by hand, twice**, in `Dispatch` and again in `TryGetReadOnlyFastPath`: `c` to `resume`,
  `quit` to `shutdown`, `s` to `step`, `jobs` to `ps`, `hexdump` to `xxd`.
- **The frontends add their own**: Mistress seven (`pause`, its own `resume`, `coretop` and `vstop` openers, `feed`,
  `state`, and its console's `clear`; `MainWindow.axaml.cs:598`, `HotaruWindow.cs:80`), Hotaru four
  (`coretop` with its window opener, `state`, `core`, and its shell window's `clear`; `Program.cs:184`,
  `DianaShShellWindow.cs:60`). A frontend's command replaces a same-named default (`DianaShInterpreter.cs:165`).

Of the 83, read from each class's body:

- **every one of the 46 debugger commands reads the target**, except four that need something else of the host:
  `core` (a core registry), `state` (two delegates), `resume` (it returns `HostAction.Resume`) and `log` (it sets
  `DianaShLogging.MasterEnabled`, a process global that the emulator's log writer reads, `EmuSen/Settings/DebugSettings.cs`);
- **of the 37 Unix commands, three read the target**: `ps` lists breakpoints and watches beside shell sessions, `kill`
  removes them, and `vstop` checks it for null. The other 34 never touch it, and are handed it on every call;
- **eleven take the interpreter or its session manager through their constructors**: `cd`, `su`, `whoami`, `who`,
  `useradd`, `userdel` and `passwd` a `Func<DianaShInterpreter>` that `CreateDefault` closes over itself
  (`selfAccessor`, `:83`); `tmux`, `ps`, `kill` and `who` the `DianaShSessionManager`; `history` the history;
- **nineteen resolve paths through `DianaShSandbox`**, a static class, and ten name Galaxia's types directly.

**The read-only flag** is a property of each class (`IDianaShCommand.IsReadOnly`); 42 of the 83 report true. A
command with a read-only verb and a mutating one reports false (§3.3b there).

**The manual** is one dictionary of 98 pages in `ManPages.cs`, keyed by command name, read by the interpreter's `man`.

### 1.3 What a command is handed: `IDebugTarget`

`IDebugTarget` (`Lib/IDebugTarget.cs`) declares 29 members under 28 names (`Disassemble` has two overloads), and
inherits 16 from `ICoreTelemetry` (`Lib/Cauldron/ICoreTelemetry.cs`): 45 in all, 27 of them required (11 of its own
and the 16 inherited), as `EmuSen_Surfaces.md` §4.6 counts too. It is named in 91 of the shell's 142 files; outside
the shell, the non-test callers use three of its members (`GetMemorySpaces`, `Watches`, `ApplyCheats`; the same
§4.6). Fourteen of its 28 are registries or nullable
registries: `Watches`, `FrameLog`, `Breakpoints`, `CoprocessorBreakpoints`, `Coverage`, `CoprocessorCoverage`,
`CallStack`, `Labels`, `AccessCounters`, `Freezes`, `Expressions`, `RegisterFlow`, `DmaLog` and `Cheats`. These are
classes of the shell's own `Var/` folder, handed out as objects, so a command calls their methods directly and the
cores feed them directly: the C# cores call `NoteWrite`, `ShouldBreak`, `Record` and the rest from inside their memory
buses and instruction loops (`EmuSen_Debugging_Tools_Reference_v5.md` §3.2a). Each registry therefore has two faces in
one class: what a command asks (`AddBreakpoint`, `RemoveWatch`, `Arm`, `Tail`) and what a core reports (`NoteWrite`,
`RecordRead`, `ShouldBreak`).

Seven classes implement `IDebugTarget`: the four C# cores' (`SnesDebugTarget`, `MoonDebugTarget`,
`MercuryDebugTarget`, `MarsDebugTarget`), two Rust ports' mirror debuggers (`MarsRtDebugTarget`,
`MercuryRtDebugTarget`), and `CoreDebugTarget`, which presents any core ABI v1 core from its descriptors and its
debug exports (`EmuSen/Cores/Native/CoreDebugTarget.cs`).

### 1.4 The tangles, counted

The shell is beneath the emulator in the reference graph and above it in what it knows:

| Who | Reaches into the shell for | Counted |
|---|---|---|
| `EmuSen` (the cores and the host) | `IDebugTarget` and every value type of the debugger, the registries, the cheat engine and codecs, the telemetry providers, the logging switch | 79 files; `DebugRegisterValue` alone 296 times, `IRealtimeProvider` 59, `CheatRegistry` 43, `BreakpointRegistry` 40 |
| Serenity | the telemetry contracts, for its dashboard | 3 uses of `ICoreTelemetry`, and a project reference |
| Mistress | the interpreter, the scheduler, the session manager, the sandbox, and the cheat engine for its Active Cheats and cheat database windows | 11 files; `ICheatCodeCodec` 21, `IDebugTarget` 15, `DianaShResult` 15, `IDianaShCommand` 14, `CheatRegistry` 9 |
| Hotaru | the interpreter, the scheduler, the sessions, the line reader, `FrameRecorder`, `HostAction` | 5 files; the interpreter 11, `IsAwaitingMoreInput` 10, `HostAction.Shutdown` 5 |
| Pharaoh | the interpreter as the fall-through of its `--commands` verbs | 2 files |
| WiseMan | everything | 145 files; `.Submit(` 443 times, `CreateDefault` 189, `CheatRegistry` 136, `DianaShSandbox` 95 |

And within the shell:

- **Every command is handed the emulator**, the 34 that never use it included (§1.2).
- **Commands are wired to the interpreter by closures** (`selfAccessor`), because `IDianaShCommand.Execute` gives a
  command no way to reach the shell that runs it.
- **`CreateDefault` takes seven parameters**, five of them the emulator's (`ICheatCodeCodec` twice, `ICpuTraceSwitch`,
  a supported-systems function, and the target), so the shell's constructor names the cheat engine's codecs.
- **Process globals hold shell state**: the sandbox, the account registry, the logging switch and the suspensions are
  static classes, so two shells in one process share accounts and every shell shares the emulator's log switch.
- **`Var/` mixes the shell's state with the debugger's**: `CommandHistory` and the `tmux` sessions beside the
  breakpoint registry.
- **`HostAction` carries five unrelated things** (`Lib/HostAction.cs`): two the shell's own (`Shutdown`,
  `SwitchSession`), and three the emulator's (`Resume`, `Step`, `LoadCore`).
- **The welcome banner lists cores** (`GetWelcomeBanner(IEnumerable<string> supportedCores)`).

### 1.5 How each host embeds it today

| Host | How | Where |
|---|---|---|
| The standalone program | `CreateDefault(null)` and a read loop over `ConsoleLineReader` | `Bin/Program.cs` |
| Mistress | a console window per main window: a session manager, a scheduler per session, read-only lines run on the UI thread, the rest drained by the emulation thread each frame (`DrainPendingFromEmulationThread`) | `HotaruWindow.cs` (175 lines), `MainWindow.axaml.cs:1332` |
| Hotaru | the F4 prompt and an always-live terminal on a reader thread; while halted at a breakpoint, lines go to `TakeHaltedLine` on the thread that owns the core; `HostAction.Resume`, `Step`, `Shutdown`, `LoadCore` and `SwitchSession` steer its loop | `GameWindow.axaml.cs` (704 lines), `DianaShShellWindow.cs`, `Program.cs` |
| Pharaoh | an unknown `--commands` verb falls through to an interpreter built over the bundle's debug target | `CommandsScriptRunner.cs:19`, `Program.cs:237` |
| WiseMan | `CreateDefault` over fake targets and real cores, 645 cases in `DianaSh/` (55 files, 9,967 lines) and 43 more files elsewhere | `Fixtures/DianaShellFixtures.cs` |
| The conformance kit | none yet; `EmuSen_CoreAPI.md` §12.2 plans Pharaoh as the runtime suite's client | — |

Every host reaches the shell's classes directly, and every one constructs its commands' dependencies itself. The
copying shows: `SyncSchedulersToSessions` is identical in `HotaruWindow.cs` and `GameWindow.axaml.cs`
(compared with `diff`, 2026-10-10), and each host answers `HostAction` its own way: Mistress handles `SwitchSession`
alone, Hotaru all five, and Pharaoh none, because it calls `Execute`, which returns the output and drops the action
(`EmuSen_Surfaces.md` §4.1). So `core <name> <path>` loads a game in Hotaru and does nothing in Mistress.

---

## 2. Principles

Every surface of this page follows these, and §11 says how each is kept.

- **One owner per surface, and a written interface for it.** A surface is a header, with its rules in this page; what
  is not in the header does not cross. No component holds another's object across a surface: handles and copied data
  only.
- **The shell knows no emulator noun.** Not a core, a target, a register, a cheat or a frame. The test is mechanical:
  the shell core's crate depends on no crate of the emulator but Galaxia's tree, and no identifier of `emusen_session.h`
  appears in it (§11.4).
- **The core ABI's rules, one layer up** (`EmuSen_CoreAPI.md` §2.3, §4, §6.0, §6.16, §6.17): signed statuses and a
  last-error text; the length-query idiom; every struct begins with its size; open enumerations; JSON for documents read
  once, typed calls for anything repeated; no pointer kept after a call; nothing unwinds across a boundary.
- **Pull, with one stated exception.** The core ABI never calls its host. A command cannot work that way: it is a
  program, and a program makes calls of its system while it runs. So a module calls the service tables it is handed,
  and only during its own call, on that call's thread (§6.0). No surface stores a function pointer of another's to call
  later.
- **Stable, not exact.** The platform library's interface is matched exactly, because it and its host come from one
  build (`EmuSen_RustPlatform.md` §3.2). These surfaces are not, because a module or a host may come from elsewhere,
  as an outside core may (§11).
- **Behaviour is held by golden outputs, not reasoned about.** What a command prints today is recorded byte for byte
  before anything moves, and every step is graded against the record (§13.1).
- **Unix where it fits**, as `EmuSen_CoreAPI.md` §2.2 has it: a command is a name on a path, it has argv, standard input
  and output and an exit status, and its manual is a page beside it.

---

## 3. The layers

```
  ┌────────────────────────────────────────────────────────────────────────────────────────┐
  │ Hosts: the dianash terminal program · Mistress's and Hotaru's console windows ·        │
  │        Pharaoh · WiseMan · the conformance kit                                         │
  └───────────────┬───────────────────────────────────────────────┬────────────────────────┘
                  │ the host API, dianash.h (§10)                  │ provides services (§9)
  ┌───────────────▼───────────────────────────────────┐            │
  │ DianaSh, the shell core (§4)                      │            │
  │   lexer · parser · expansion · pipes · redirection│            │
  │   variables · history · control flow · users ·    │            │
  │   the sandbox · shell sessions · command lookup · │            │
  │   builtins · manifests · man                      │            │
  └───────┬───────────────────────────┬───────────────┘            │
          │ the module ABI,            │ processes (§7)             │
          │ dianash_module.h (§6)      │                            │
  ┌───────▼───────────────────┐  ┌────▼─────────────┐              │
  │ command modules           │  │ programs         │              │
  │ text tools · debugger ·   │  │ nano · a player's│              │
  │ cheats · session · host's │  │ own · any program│              │
  └───────┬───────────────────┘  └──────────────────┘              │
          │ services, by name and version: emusen.session@1 (§9),  │
          │ emusen.frontend@1 (§9.9)                               │
  ┌───────▼────────────────────────────────────────────────────────▼───────────────────────┐
  │ Crystal (EmuSen_CoreAPI.md §8): sessions, the loop, the debugger's registries,         │
  │ cheats, states, settings, telemetry; until it is built, an adapter over IDebugTarget   │
  └───────────────────────────────────────┬────────────────────────────────────────────────┘
                                          │ the core ABI v1, emusen_core.h
  ┌───────────────────────────────────────▼────────────────────────────────────────────────┐
  │ Core libraries                                                                         │
  └────────────────────────────────────────────────────────────────────────────────────────┘
```

| Layer | Owns | Never does |
|---|---|---|
| **Shell core** | the language and its state: lines, words, variables, history, control flow, the working directory, the current account, shell sessions; finding, checking and calling commands; the manuals' lookup | name an emulator type; read a service table; hold a module's or a host's object; write outside the sandbox on its own account |
| **Module ABI** | what a command is given and gives back, and how it is found, loaded, versioned and trusted | carry an emulator noun: those are a service's |
| **Command module** | one or more commands | keep a pointer it was handed past its call; call a service outside its call; link Galaxia or open the tree itself (it asks the shell, §6.3) |
| **Program** | one command, as a process | reach a running core (§7.4) |
| **Services** | what a host offers commands: EmuSen's session service and frontend service | know that a shell exists: a service is called by a module, and the same table serves a frontend's own windows (§9.1) |
| **Host API** | how a frontend runs a shell, feeds it lines, takes its output and provides its services | expose the shell's internals |
| **Crystal** | everything `EmuSen_CoreAPI.md` §8.2 gives it | depend on the shell |

Three surfaces are stable and guarded (§11): the host API, the module ABI and the session service. The frontend
service (§9.9) is stable too, and small.

---

## 4. The shell core

### 4.1 What it is

A Rust crate, `dianash`, in the platform workspace (`Platform/dianash/`, beside `galaxia`; §16 Q11), built two ways:

- **as the terminal program `dianash`**, the standalone shell of decision 1: the crate, Galaxia's crate for the tree,
  a line editor and the text tools' module linked in, so that a plain install on any of the three systems needs one
  file to have a working shell (§8.2);
- **as the C interface `dianash_*`** in the platform library, which the C# frontends call (§10, §13).

It owns, and nothing else does:

| What | As today |
|---|---|
| **The lexer, parser and AST** | the grammar of `EmuSen_Debugging_Tools_Reference_v5.md` §3.17 and §3.17b, unchanged: quoting, `$VAR`, `${VAR}`, `$?`, `$(...)`, `|`, `>`, `>>`, `<`, `;`, `&&`, `||`, `!`, `if`/`elif`/`else`, `for`, `while`, `until`, `break`, `continue`, multi-line continuation |
| **Expansion** | word splitting with quoted spans kept; an unquoted word that expands to nothing vanishes; `$(...)` in a subshell whose assignments do not leak (`DianaShInterpreter.cs:684`–`:797`) |
| **Pipes and redirection** | a pipeline's stages run in order, each given the whole output of the one before (§4.3); redirections resolved through the sandbox |
| **Variables** | one scope per shell, copied into a subshell; `export` and `unset`; the known simplification that `FOO=bar cmd` sets `FOO` persistently is kept, and `export`ed variables now reach programs' environments (§7.2) |
| **History** | per shell; `!!` and `!N`, expanded before recording; `history` and `history clear` |
| **Control flow and its budget** | the ten-second loop budget, shared with subshells; twenty nested `source`s at most (`:23`, `:34`) |
| **Users** | identity only, as `man su` says: `root` always present, `su`, `useradd`, `userdel`, `passwd` with a hash nothing gates on (`Etc/DianaShUserRegistry.cs`). Per shell process, not a static |
| **The sandbox** | the shell's `/` is Galaxia's `DataStore.UsrHome`; a path is resolved chroot-style and refused outside it (`Etc/DianaShSandbox.cs`). The rule moves; Galaxia's crate gives the root (§12.1) |
| **Shell sessions** | `tmux new`, `switch`, `kill`; a session is an interpreter with its own variables and history, names and accounts surviving a rebuild (`Var/DianaShSessionManager.cs`) |
| **Jobs** | `ps` and `kill` over one id space: shell sessions, and whatever modules list (§6.8) |
| **Command lookup** | builtins, then aliases, then modules, then programs (§5.2) |
| **Manuals** | `man` reads a command's page from beside its module (§5.5); `help` lists every command's usage |
| **The welcome banner** | the shell's own, with no list of cores; a host may add lines (§10.4) |

### 4.2 The builtins

A builtin is a command the shell runs itself because it is the language, or because it reads or changes the shell's
own state, which no module may touch (§6.0). There are 29 names, counting `.` and the new ones:

| Group | Builtins |
|---|---|
| The language | `source` (and `.`), `export`, `unset`, `true`, `false`, `test`, `[` |
| Directory and identity | `cd`, `pwd`, `su`, `whoami`, `who`, `useradd`, `userdel`, `passwd` |
| The shell's state | `history`, `tmux`, `ps` (and `jobs`), `kill` |
| Its manuals and output | `help`, `man`, `echo`, `clear` |
| Ending | `shutdown` (and `quit`, and `exit`, which today is not a command; §16 Q12) |
| Lookup and trust | `which`, `command` (new: which kind and which file answers a name; §5.2), `trust` (new: a player's module approved, §5.4) |

`echo`, `true`, `false`, `test` and `[` are builtins in every shell that matters, and for the same reason here: they
are what conditions and loops are made of, and a script runs them thousands of times (§7.5). `clear` asks the host to
clear its screen (§10.4). Today's `summary` leaves the shell, for the session module (§8).

### 4.3 Text, bytes and the pipeline

Today a command takes a string and returns one (`IDianaShCommand.Execute`, `DianaShResult.Output`), and a pipeline
hands each stage the whole output of the last (`DianaShInterpreter.cs:439`). The module ABI passes bytes, with the
rules that keep today's behaviour exactly:

- **Standard input is a byte span or absent.** Absent means the command is first in its pipeline or stands alone;
  empty means the stage before it printed nothing. The distinction is today's (`IDianaShCommand.cs`, "null stdin is
  not empty stdin") and is kept.
- **Standard output is bytes, UTF-8 by convention.** The shell treats it as text where the language needs text: word
  splitting of `$(...)`, the trailing newline it trims, and the newline it adds between a pipeline's output and the
  next line (`:452`).
- **A pipeline is buffered, as now.** Each stage finishes before the next begins. Streaming between two programs is
  possible later without a change to any module (§7.3), and is not offered in version 1, because the golden outputs
  are of buffered pipelines and a streamed one interleaves standard error differently.
- **Standard error is new.** Today a command's errors are part of its output, `Error: ...` (`:556`). The module ABI
  gives a command a second stream; the shell prints it where the output goes, in order, so that what the player sees
  is today's byte for byte, and a redirection takes only standard output, as in Unix (§16 Q10).
- **The exit status is a signed 32-bit integer**, as today's `ExitCode`; `$?` prints it in decimal.

### 4.4 What the shell core does not own

The shell keeps no emulator state and no frontend's. Concretely, each of these leaves the assembly that is the shell
today, with its owner named in §9: the debugger's registries, the cheat engine, `IDebugTarget` and its value types,
the telemetry contracts, `FrameRecorder`, `RuntimeStats`, the expression evaluator (its language is the debugger's),
the logging master switch, the system packs, and the scheduler's split of lines by thread (§6.5).

---

## 5. Commands: finding, trusting and loading them

### 5.1 Three kinds, called one way

To the player a command is a name: `mem`, `grep`, `nano`. Whether it is a builtin, a module or a program is visible
only to `which` and `command -v`. All three receive argv, standard input, the working directory and the current account, and give
back standard output, standard error and an exit status. A module may also be handed services (§6.3); a builtin and a
program cannot be.

### 5.2 Where commands are found

**The path.** The shell's variable `PATH` lists directories inside the sandbox, as a Unix shell's does, with the
default `/bin:/lib/dianash`. Two more directories are searched first and are not on the path, because they are the
install's and lie outside the sandbox (`man hier`: the install is "outside the shell's reach on purpose"):

| Where | What | Who puts it there |
|---|---|---|
| `<install>/lib/dianash/` | the modules and programs a release ships, with their manifests and manuals | the project's build (§12.4 for each platform) |
| a host's own | commands a frontend provides in-process (§10.3) | the host, at `dianash_new` |
| `/lib/dianash` in the home | modules the player adds | the player |
| `/bin` in the home | programs the player adds | the player |

**The order of lookup**, for a name that is not a path: a builtin; an alias (§5.3); a host's command; a shipped module
or program; then the path in order. A name containing `/` is a program at that path, resolved through the sandbox.
`which <name>` prints the answer and its kind, `command -v` its file. Two commands of one name in one directory are a
packaging defect, reported at the shell's start, and the first in the manifest's order is used.

**Shadowing.** Today a frontend's command replaces a default of the same name (`DianaShInterpreter.cs:165`), and two
frontends rely on it (`resume` in Mistress, `coretop` in both). Version 1 keeps the rule for a host's commands only:
a host's command shadows a shipped one, and `which` says so. A player's module or program never shadows a builtin or a
shipped command; it is reachable by its path.

### 5.3 The manifest

Every module, and every program that wants a manual or aliases, has a manifest beside it, `<file>.dianash.json`, read
without loading anything (as a core's sidecar is, `EmuSen_CoreAPI.md` §7.1):

```json
{ "manifest": 1, "id": "emusen.debug", "version": "1.0.0", "abi": "1.0",
  "kind": "module", "file": "libdianash_debug.so", "sha256": "…",
  "services": [{ "name": "emusen.session", "major": 1, "minor": 0, "required": true }],
  "commands": [
    { "name": "mem", "usage": "  mem <space> <addr> [<len>]    xxd-style hexdump (default 16 bytes)",
      "read_only": true, "man": "man/mem.1" },
    { "name": "write", "usage": "  write <space> <addr> <value>  write one byte, if the space is writable",
      "read_only": false, "man": "man/write.1" } ],
  "aliases": { "hexdump": "xxd" },
  "jobs": ["bp", "watch"] }
```

| Field | Meaning |
|---|---|
| `manifest` | the manifest format's version, 1 (§11.2) |
| `id`, `version` | the module's name, reverse-dotted, and its own version, for the player and the crash log; neither plays a part in compatibility |
| `abi` | the module ABI version it was built against, `major.minor` |
| `kind` | `module` or `program` |
| `file`, `sha256` | the library or executable beside the manifest, and its hash |
| `services` | each service by name with the major and the least minor it needs; `required: false` lets the module run without it and answer as today's commands answer "No ROM loaded" (§6.3) |
| `commands` | each command: its `name`; its `usage`, the line `help` prints, kept verbatim from today's `Usage`; `read_only` (§6.5); `man`, its page's file |
| `aliases` | names the shell maps to a command of this manifest, as today's five are |
| `jobs` | the id prefixes whose entries this module lists in `ps` and removes with `kill` (§6.8) |

A reader ignores a field it does not know. The `help` listing is built from manifests alone, in a stable order (the
shipped modules' order, then the rest alphabetically), so starting a shell loads no module.

### 5.4 Trust

The rules of `EmuSen_CoreAPI.md` §7.2, applied to commands:

1. **Shipped modules and programs** are listed with their hashes in the release's `dianash.manifest`, and run without
   asking.
2. **A module the player adds** is loaded only after the player approves it once, by hash, with its id, version,
   commands and requested services shown, and the plain statement that a module runs inside the program with the
   player's full permissions and, if it asks for `emusen.session`, can read and change a running game. A changed file
   is asked about again. Approval is a shell command, `trust <path>`, run by the player, never by a script line that
   loading the module could have written (§16 Q13).
3. **A program the player adds** runs as any program does in any shell. It is the player's file in the player's home;
   a shell that asked before running `/bin/x` would not be a Unix shell. It is never handed a service (§7.4).
4. **Before loading a module**: a regular file inside one of §5.2's directories after resolving links; its header names
   this process's architecture; its hash matches the manifest or the approvals. **After loading**: the module ABI's
   major is one the shell speaks; every required export resolves; `dianash_module_manifest()` equals the file beside
   it. Any failure leaves the module loaded and never called again, and the command answers with the check that failed.
5. **A manifest is data.** Usage lines and manuals are printed, never interpreted.

### 5.5 Manuals

Each command's page is a file beside its module, `man/<name>.<section>`, in today's format (`NAME`, `SYNOPSIS`,
`DESCRIPTION`, `EXAMPLES`, plain text wrapped at 80 columns), so `man` prints it with no code of the module's run. Pages
that are not a command's (`hier`, `dianash` itself, the language's) are the shell's own, in `<install>/lib/dianash/man/`.
`man` with a name that has no page falls back to the usage line, as now (`DianaShInterpreter.cs:631`). The 98 pages of
`ManPages.cs` become 98 files in the migration's first steps, still read by the C# `man`, and the move is graded by the
golden outputs of `man` for every page (§13.2, S1). *Done 2026-10-11, 99 files by then* (§13.5).

### 5.6 Loading

A module is loaded the first time one of its commands runs, never at start, and never unloaded (as cores are not,
`EmuSen_CoreAPI.md` §6.16). A process loads each module once, whatever number of shells it runs. A module's library
state is therefore shared by every shell in the process, which is why the ABI gives each call its own context and a
module keeps per-shell state only through it (§6.4).

---

## 6. The command module ABI

The normative header is `dianash_module.h` (Appendix A sketches it). Exports are `dianash_module_*`, types `dianash_*`,
constants `DIANASH_*`, as the core ABI's naming rule has it (`EmuSen_CoreAPI.md` §5.1).

### 6.0 The rules every export and every table follows

- **The core ABI's**, as §2 lists them, including no `void` export and no unsigned getter but the two library-level
  calls, and nothing that unwinds, `longjmp`s, raises a signal or calls `exit`.
- **A module calls what it is handed, and only during its call.** The `dianash_call` given to `run` carries the shell's
  table (`io`) and the services the manifest asked for. A module may call them only before `run` returns, on the thread
  that called `run`, and must not keep a pointer to any of them, or to anything they returned, past that. This is the
  one place in the project where a library calls its host, and the reason is stated in §2.
- **No module touches the shell's state.** Variables, history, the working directory, the current account, the sessions: a module
  reads the few it is given and changes none. A command that must change the shell's state is a builtin (§4.2).
- **A module keeps no per-shell state of its own in globals.** What a command must remember between calls in one shell
  (`search`'s candidates, `snapshot`'s baselines) it keeps in the call's per-shell slot (§6.4).

### 6.1 Library-level exports

```c
uint32_t dianash_module_abi_version(void);                            /* (MAJOR << 16) | MINOR */
int64_t  dianash_module_manifest(uint8_t *out, size_t len);           /* the manifest's JSON, length-query */
int32_t  dianash_module_run(const dianash_call *call, dianash_result *result);
/* optional, claimed by the manifest's "jobs" */
int64_t  dianash_module_jobs(const dianash_call *call, uint8_t *out, size_t len);
int32_t  dianash_module_kill(const dianash_call *call, const uint8_t *id, size_t id_len);
```

`run` serves every command of the module; the call names which. There is no `init`: a module that needs to set
something up does it on its first call, and a module with nothing to release needs no `fini`, because a module is
never unloaded.

### 6.2 A call, and what it gives back

```c
typedef struct { const uint8_t *data; size_t len; } dianash_bytes;

typedef struct {
    uint32_t size;                 /* the shell's size of this struct */
    dianash_bytes command;         /* the name it was called by, after alias mapping */
    const dianash_bytes *argv;     /* argv[0] is the name as typed */
    size_t argc;
    dianash_bytes stdin_data;
    uint32_t stdin_present;        /* 0: first in its pipeline or alone; 1: a stage before it ran */
    uint32_t flags;                /* DIANASH_CALL_READ_ONLY_CONTEXT, DIANASH_CALL_INTERACTIVE */
    dianash_bytes cwd;             /* the working directory as the shell shows it, "/" rooted */
    dianash_bytes user;            /* the current account */
    const dianash_io *io;          /* the shell's table: output, paths, cancellation, the per-shell slot */
    const dianash_service *services;  /* those the manifest asked for and the host provides */
    size_t service_count;
} dianash_call;

typedef struct {
    uint32_t size;
    int32_t exit_status;           /* $? */
} dianash_result;
```

`run` returns `DIANASH_OK` (0) when the command ran, whatever its exit status, and a negative status when the call
itself could not be made: an unknown command name, a struct below its version 1.0 size (`DIANASH_BAD_STRUCT`), a
fault caught at the boundary (§6.7). A command that fails in the player's sense (no such space, a bad address) returns
0 with a non-zero `exit_status` and its words on standard error, as today's `DianaShResult.Fail` does.

**What a result does not carry: an action for the host.** Today `DianaShResult.Action` lets a command steer its host's
loop (`HostAction`). The module ABI has no such field. Each of today's five actions has another owner:

| `HostAction` today | Becomes |
|---|---|
| `Shutdown` (`shutdown`, `quit`) | a builtin; the host learns it from `dianash_submit`'s status (§10.2) |
| `SwitchSession` (`tmux`) | a builtin; the host reads the current session from the shell |
| `Resume` (`resume`, `c`, a bare `continue`) | a session service call, `run_control(RESUME)`; the host learns of it from Crystal's event stream, as Mistress's windows will |
| `Step` (`step`, `s`) | `run_control(STEP, …)`, likewise |
| `LoadCore` (`core <name> <path>`) | a frontend service call, `open_game` (§9.9), since the frontend opens games until Crystal does |

So a command never changes the control flow of the loop that called it, and the rule of
`EmuSen_Debugging_Tools_Reference_v5.md` §3.3b ("whose control flow changes") is answered once, by the layers: the
shell's, by a builtin; the emulator's, by its own service and its events (§16 Q5).

### 6.3 What the shell gives a call: the io table

```c
typedef struct {
    uint32_t size;
    void *ctx;                                                     /* opaque, this call's */
    int32_t (*out)(void *ctx, const uint8_t *data, size_t len);    /* append to standard output */
    int32_t (*err)(void *ctx, const uint8_t *data, size_t len);    /* append to standard error */
    int32_t (*resolve)(void *ctx, const uint8_t *path, size_t len, uint32_t flags,
                       uint8_t *out, size_t cap, int64_t *needed);  /* a shell path to a real one, through the sandbox */
    int32_t (*display)(void *ctx, const uint8_t *real, size_t len,
                       uint8_t *out, size_t cap, int64_t *needed);  /* a real path back to the shell's "/" form */
    int32_t (*cancelled)(void *ctx);                               /* 1 once the player cancelled or the budget ran out */
    int32_t (*slot)(void *ctx, void **state, void (**release)(void *));  /* the per-shell state of this module */
    int32_t (*environment)(void *ctx, const uint8_t *name, size_t len,
                           uint8_t *out, size_t cap, int64_t *needed);  /* an exported variable; ABSENT when unset */
    int32_t (*terminal)(void *ctx, dianash_terminal *out);         /* the terminal, when the call is interactive */
} dianash_io;
```

- **`resolve` is the only way a module reaches a file.** It applies the sandbox exactly as `DianaShSandbox.TryResolve`
  does today, including the refusal and its words, and the suspensions (`rm` and `mv` refused inside a published
  root that holds its own install, `Etc/DianaShCommandSuspensions.cs`) through its `flags` (`DIANASH_RESOLVE_REMOVE`,
  `DIANASH_RESOLVE_WRITE`). A module never links Galaxia's crate, so there is one copy of the tree's rules in a process
  and one set of overrides (§16 Q11).
- **`slot` gives a module one pointer of state per shell**, created by the module on first use and released by the
  shell, through the `release` the module set, when that shell ends. `search` keeps its candidates there and
  `snapshot`/`diff` their baselines (today `SearchCommand`'s fields and `SnapshotStore`), so two shells in one process
  no longer share them.
- **`terminal`** answers the size and mode of a real terminal when there is one; a module that needs raw input (none
  today but `nano`, which becomes a program, §8) is told there is none in a console window.

**Services.** Each entry of `services` is `{ size, name, major, minor, table }`: the host's table for a service the
manifest named, at the version the host has. The shell copies the entry from what its host gave it (§10.3) and never
reads `table`. A service the manifest required and the host does not provide stops the call before `run`, with
`<command>: needs emusen.session, which this program does not provide`; one not required is simply absent, and the
command answers as today's debugger commands answer a null target. Today's words, from
`DebugCommandHelpers.RequireTarget`, are kept for the debugger's modules, so the golden outputs hold.

### 6.4 Threading and re-entrancy

- **A module's `run` may be called from several threads at once**, for different shells, and must be safe for that.
  Calls for one shell are never concurrent.
- **A call runs on the shell's thread.** There is no longer a split of lines by thread in the shell
  (`DianaShInterpreterScheduler`, §1.5): reads that are safe from any thread are the session service's to answer from
  snapshots, and changes are Crystal's to apply at the frame boundary (§9.2). A mutating call therefore waits, at
  most until the next frame boundary, or not at all when the session is paused or halted. That keeps today's
  guarantee, that no mutation races `RunFrame`, without the shell knowing frames exist (§16 Q6).
- **No re-entrancy.** A module may not call `dianash_*` host functions, and a service table may not call back into the
  module.

### 6.5 The read-only flag

`read_only` is per command in the manifest, as today's `IsReadOnly` is per class. Its use changes:

- **The shell uses it to classify a line**, as `TryGetReadOnlyFastPath` does today (`DianaShInterpreter.cs:212`),
  with the same rules: one simple command, no assignment, no redirection, a literal name. A host may ask (§10.2) whether
  a line is read-only, to run it on a thread that must not wait for a frame.
- **The service table enforces it.** A call marked `DIANASH_CALL_READ_ONLY_CONTEXT` that reaches a mutating service
  function is refused with `EMUSEN_SVC_READ_ONLY` and the words `<command> is read-only and tried to change the
  session`. Today a wrong flag races the emulation thread silently; with the table it cannot. Each service function is
  marked mutating or not in its header (§9).
- **A command with mixed verbs** still declares `read_only: false`, as now. A later minor may let a manifest name
  read-only verbs (`"read_only_verbs": ["list", "log"]`); version 1 does not, because the golden outputs were recorded
  with today's per-class rule.

### 6.6 Cancellation and the budget

- **The ten-second budget** of today's loops (`MaxExecutionTime`, `DianaShInterpreter.cs:21`) stays the shell's, and
  is now visible to commands: `io->cancelled` answers 1 once it is spent, or once the player presses Ctrl-C at a
  terminal or the host calls `dianash_cancel` (§10.2).
- **A module polls it** in any loop that can run longer than about 50 ms, and returns with exit status 130 (the Unix
  status for an interrupt) and `Interrupted` on standard error. Today's commands never check, and their only loops
  that can run long are `search`, `memfind` and `callers`/`writers`/`readers` over a large space.
- **A module that never returns cannot be stopped.** It is in the process; nothing in a process can end one thread's
  call safely. The shell reports that the command is still running after the budget, once, and the host keeps
  answering (`dianash_submit` is then refused with `DIANASH_BUSY` until it returns). A program is different: it is a
  process, and is ended (§7.3). This is a reason a command that may run long and needs no session should be a program.

### 6.7 Faults

A panic in a Rust module, or an exception that would leave one written in another language, must not cross. Unlike a
core, which aborts the process after writing its crash log (`EmuSen_CoreAPI.md` §6.18), a module's export catches it
and returns `DIANASH_FAULT` (−320) with the message on standard error, and the shell prints `Error: <message>` as
today's `Dispatch` does for an exception (`:556`). Today's rule is that "a malformed command must read as failure to
`&&`/`if`, never take the console down" (`:553`), and a player typing into a running game's console has more to lose
from an abort than a core's author has from a fault being caught. So modules are built with `panic = "unwind"` and the
crate that generates their exports, `dianash-module`, wraps every export in `catch_unwind` (§16 Q4). A module whose
fault left it inconsistent is marked faulted by the shell after its third fault in one process and is not called again.

### 6.8 Jobs: `ps` and `kill`

Today `ps` lists breakpoints, watches and shell sessions as one table, and `kill bp3` removes a breakpoint
(`Bin/Commands/Unix/PsCommand.cs`, `KillCommand.cs`). The two become builtins, and a module that lists things in the
same id space says so in its manifest (`"jobs": ["bp", "watch"]`):

- `ps` prints the shell's sessions, then calls `dianash_module_jobs` of each loaded module that claims jobs, which
  returns lines in today's format (`  bp3        breakpoint  $00812C (enabled, hit 2x)`);
- `kill <id>` finds the module that claims the id's prefix and calls `dianash_module_kill`; with no claimant it is a
  shell session's name, as now;
- a module not yet loaded has nothing to list, because nothing of it can have been made.

When Crystal lists its sessions as processes (`EmuSen_CoreAPI.md` §8.5), the session module claims their prefix and
`ps` shows them the same way.

### 6.9 Status codes

The module ABI's band is −1 to −8 the shared codes (`DIANASH_NULL` −1, `DIANASH_BAD_STRING` −5,
`DIANASH_BUFFER_TOO_SMALL` −7), −256 to −319 the interface's (`DIANASH_NOT_SUPPORTED` −256, `DIANASH_BAD_STRUCT` −263,
`DIANASH_UNKNOWN_COMMAND` −265, `DIANASH_SERVICE_MISSING` −266, `DIANASH_BUSY` −267, `DIANASH_ABSENT` −268, an answer
and not a failure), and −320 `DIANASH_FAULT`. These
are the core ABI's numbers where the meaning is the same, so a log that prints both cannot confuse them.

### 6.10 Writing a module

The reference implementation for modules is a Rust crate, `dianash-module`, beside the shell's: a trait a module
implements (`fn run(&self, call: &Call, out: &mut Output) -> i32`) and a macro, `module_exports!`, that generates every
export, the manifest check and the fault boundary from it, as `core_exports!` does for cores (`EmuSen_CoreAPI.md` §18).
A module in C, or in any language that can export C functions, writes the five exports by hand against the header.

---

## 7. Commands as programs

### 7.1 What a program is given

A program is run as a process, as a Unix shell runs one:

- **argv** as the module's, with `argv[0]` the name as typed;
- **standard input** the previous stage's output, or the shell's own terminal when the program stands alone and the
  shell has one, so that an interactive program (`nano`) owns the terminal while it runs;
- **standard output and error** captured into the pipeline, or the terminal's when it stands alone and is last;
- **the working directory** set to the real directory behind the shell's `cwd`, which is inside the sandbox;
- **the environment**: the host process's own, with every `export`ed variable added and `DIANASH_ROOT` (the sandbox's
  real root), `DIANASH_USER` and `DIANASH_SHELL` (the shell's version) set;
- **its exit status** as `$?`; a program ended by a signal gives 128 plus the signal's number, as Unix shells give.

### 7.2 What a program is not given

- **No service.** A program cannot reach a running core: there is no session service across a process boundary in
  version 1 (§7.4).
- **No sandbox.** A program sees real paths and can open any file the player can; the sandbox is "a walled garden, not a
  security boundary" (`man hier`), and a process is outside its wall. The shell resolves a program's own path and its
  working directory through the sandbox, and passes arguments unchanged, as every shell does.

### 7.3 Pipes, ending and Windows

- **Pipes** are buffered as §4.3 says: a program's output is read to its end before the next stage starts. Two programs
  in a row could be joined by a real pipe and streamed; version 1 does not, so that every pipeline has one meaning.
- **Ending**: on Ctrl-C or when the budget is spent, a program is sent `SIGINT`, then `SIGTERM` after 500 ms, then
  `SIGKILL` after another 500 ms, and on Windows a console control event and then `TerminateProcess`.
- **On Windows** a program is found with its `.exe` (and `.cmd`, run through `cmd.exe /c`), arguments are quoted by
  the rules `CommandLineToArgvW` reverses, and no shell is invoked for an `.exe`.

### 7.4 Why a program reaches no core

A session service across a process boundary is possible: the service table is pull-shaped and copies everything out,
so it could be carried over a pipe as the core ABI could be carried to a broker (`EmuSen_CoreAPI.md` §7.3). It would
cost a round trip per call, and a debugger command makes many calls (`mem` one per 16 bytes today, `search` one per
address). Nothing needs it: every command that reaches a core is the project's own and can be a module. It is
recorded as possible, and not offered (§16 Q3).

### 7.5 What a process costs (measured, and estimated)

**Measured on Linux**, 2026-10-10, on the development desktop (Ryzen 7 7700X, Fedora 44, kernel 7.2.7, the
`powersave` governor), from a Rust program that starts the command, writes its input on a thread, reads its output to
the end and waits for it; 500 runs after 20 of warm-up, medians with the 95th percentile in parentheses. The
programs were scratch programs, removed once the figures were recorded here:

| What is started | No input | 64 KB in | 1 MB in |
|---|---|---|---|
| `/usr/bin/true` | 0.30 ms (0.43) | 0.31 ms (0.43) | 0.29 ms (0.41) |
| a Rust program that counts lines, dynamically linked to libc | 0.46 ms (0.64) | 0.53 ms (0.73) | 1.39 ms (1.70) |
| `/usr/bin/wc -l` | 0.65 ms (0.85) | 0.65 ms (0.83) | 0.89 ms (1.11) |
| a .NET 10 program that counts lines, through its apphost | 14.9 ms (16.0) | — | — |
| **the same Rust work as a loaded module's call** | **0.04 µs** | **15 µs** | **248 µs** |

With no pipes at all, starting and waiting for `/usr/bin/true` costs 0.29 ms (0.40), and for the Rust program 0.45 ms
(0.63): the floor of a process is about a third of a millisecond. A module's first load, `dlopen` and `dlsym`, cost
51 to 63 µs, once per process.

**Today's shell, for comparison**, measured the same day from a scratch program over the C# interpreter, 2,000 runs
after 200 of warm-up: `true` 1.1 µs, `echo hello` 1.7 µs, `echo a b c | grep b | wc -l` 6.0 µs, and a loop of ten
`echo $i | grep 1` 30 µs. As programs, that loop's twenty commands would cost about 9 ms on Linux: three hundred times
as long.

**Estimated for macOS and Windows** (argued, from the platforms' published characteristics; not measured, since
nothing of this step runs on the Mac or a Windows machine):

| | A small native program, warm | The first run of a new file |
|---|---|---|
| Linux | 0.3–0.5 ms, as measured | the same |
| macOS | about 1–2 ms: `posix_spawn` and `dyld`'s binding of the system libraries, with signature checks | tens to hundreds of ms while Gatekeeper and XProtect check a quarantined or new file |
| Windows | about 5–15 ms: `CreateProcess` builds a process and loads `kernel32` and the C runtime | tens of ms more while an antivirus scans the new file |

**What follows.** A command run once at a prompt costs nothing a player could see as a program, on any of the three. A
command run in a script's loop does: a hundred-iteration loop of a two-stage pipeline would take about 0.1 s on
Linux, 0.2–0.4 s on macOS and 1–3 s on Windows, against the shell's ten-second budget, where today it takes 0.3 ms.
WiseMan's 645 shell cases alone run thousands of pipelines. So a command that scripts use in loops, and every text
filter is one, belongs in a module or the shell; a program is for what is naturally one (§8).

---

## 8. Which command goes where

### 8.1 The recommendation, for each of today's 83 and the six builtins

| Where | Commands | Why |
|---|---|---|
| **Builtins** (29, §4.2) | `source` `.` `export` `unset` `true` `false` `test` `[` `cd` `pwd` `su` `whoami` `who` `useradd` `userdel` `passwd` `history` `tmux` `ps` `kill` `help` `man` `echo` `clear` `shutdown`; new: `exit` `which` `command` `trust` | each is the language, or reads or changes the shell's own state (§4.2), which no module may (§6.0) |
| **`dianash.text`**, shipped, no service (16) | `sed` `grep` `wc` `sort` `uniq` `awk` `cat` `head` `tail` `ls` `find` `cp` `mv` `rm` `touch` `xxd` (with `hexdump`) | scripts run them in loops (§7.5); they must behave identically on three systems, which the platform's own `grep` and `sed` do not (BSD's, GNU's and none on Windows); they reach files only through the sandbox, which a program cannot (§7.2); and their outputs are golden |
| **`emusen.debug`**, `emusen.session` required (26) | `spaces` `mem` `write` `regs` `setreg` `cpus` `disasm` `tile` `tilemap` `sprites` `pal` `channels` `mute` `vectors` `addr` `dump` `load` `search` `memfind` `snapshot` `diff` `callers` `writers` `readers` `eval` `label` | the inspection and editing of a machine's state |
| **`emusen.trace`**, `emusen.session` required (16) | `bp` `watch` `step` (with `s`) `runto` `bt` `resume` (with `c`) `trace` `counters` `freeze` `profile` `cov` `framelog` `copflow` `cophist` `dma` `log` | what arms the debugger and controls a run; these share the registries' services (§9.4) and are the commands that claim `ps`'s `bp` and `watch` |
| **`emusen.cheat`**, `emusen.session` required (1) | `cheat` | the largest command (470 lines) and its own subject, with the cheat database |
| **`emusen.session`**, `emusen.session` and `emusen.frontend` optional (4) | `core` `state` `summary` `coretop` | the session as a whole, its telemetry, and what only a frontend can do (open a game, a window) |
| **A program, shipped** (1) | `nano` | it owns the terminal for as long as it runs, which a process does naturally on all three systems; it needs no session; and a full-screen editor run in a loop is not a case |
| **The host's commands** (§10.3) | Mistress's `pause`, `resume`, `coretop`/`vstop` openers, `feed`, `clear`; Hotaru's `clear` | they act on the frontend's own window, as today; under §9.9 most become calls of `emusen.frontend` from the shipped modules, and the host's own shrink to `clear` |
| **Decided by Q14** | `vstop` | today it reports the .NET runtime underneath; a Rust shell in a terminal has no .NET runtime, and in a C# frontend the .NET runtime is the host's |

So a release ships one program and five modules besides the shell, and every one of today's names answers as it does
now. **The grouping is a recommendation, not a requirement of the ABI**: a module may be split later without a script
noticing, because a command is found by name (§16 Q2).

### 8.2 What the standalone shell carries

On a system with no emulator installed, the `dianash` program has its builtins, `dianash.text` linked into its own
binary (so one file is a working shell), and `nano` beside it. The four EmuSen modules may be present or not; when
present with no host to provide `emusen.session`, each of their commands answers as today's do with no ROM loaded.

---

## 9. The session service: what modules call to reach a running core

### 9.1 What it is, and who owns it

`emusen.session@1` is a C function table, `emusen_session_v1`, defined by a header Crystal owns,
`emusen_session.h`. It is the session part of Crystal's frontend API (`EmuSen_CoreAPI.md` §9.1: run, pause,
resume, step; memory; breakpoints and watches; registers; disassembly; the call stack and coverage; cheats; states;
settings; events), written once as C and offered to two kinds of client: a frontend's windows call it directly, and a
module calls the same table handed to it through the shell. So the parity rule of `EmuSen_CoreAPI.md` §8.6 (every
shell verb that acts on a session is an operation of the frontend API, and the reverse) holds by construction: there
is one definition.

**Who provides it.** Crystal, once built (`EmuSen_CoreAPI.md` §10.2, D1 to D3). Until then, a C# adapter over
today's `IDebugTarget` and its registries (§13.2, S2), so that modules can be written and graded before Crystal
exists. The table is the same either way; only its provider changes.

**Its shape.** Every function takes the table's `ctx` first, then a session id where it names one (0 for the session
the host made current), returns a signed status, and copies results out by the length-query idiom. Lists and
descriptors are JSON; values read in bulk are byte spans; ids are 32-bit integers. Each function's comment in the
header says whether it **reads** (answered from the session's latest snapshot, on the calling thread, never waiting for
a frame) or **changes** (applied at the next frame boundary, the call waiting for it; refused in a read-only context,
§6.5).

### 9.2 The groups

| Group | Functions (version 1.0) | Reads or changes |
|---|---|---|
| **Session** | `sessions` (JSON list), `info` (engine, system, frame count, run state, capabilities), `summary` (free text) | reads |
| **Run control** | `run_control(op, processor, count)`: pause, resume, step, step over, step out, run to an address, an interrupt, a scanline or a frame; `halt_state` (halted, where, on which processor) | changes; reads |
| **Memory** | `spaces` (JSON: name, size, writable, side effects, view), `space_read`, `space_write`, `resolve_physical` | reads; changes; reads |
| **Processors and registers** | `processors` (JSON, as `DebugCpus`), `registers(processor)` (JSON: name, value, bits), `set_register` | reads; changes |
| **Disassembly** | `disassemble(processor or space, address, count, hints)` (JSON records: address, bytes, mnemonic, operands, static reference and its kind) | reads |
| **Breakpoints and halts** | `bp_add`, `bp_add_data`, `bp_set_enabled`, `bp_remove`, `bp_list`, `bp_log`, `bp_log_clear`, `bp_arm_condition`, `bp_conditions`, `bp_depth_guard`, `bp_forbid_range`, `bp_uninitialized` | changes, but `bp_list`, `bp_log` and `bp_conditions` read |
| **Watches and frame logs** | `watch_add`, `watch_remove`, `watch_list`, `watch_events`, `watch_summary`, `watch_clear`; `framelog_add`, `framelog_remove`, `framelog_list`, `framelog_samples`, `framelog_clear` | as the verbs say |
| **Coverage, profile, counters** | `coverage_op(processor, op, …)` (arm, disarm, clear, mark, query, new since mark, entry points, export, import, merge); `profile_op`; `counters_op`; `call_stack(processor, count)` | as the verbs say |
| **Freezes and labels** | `freeze_add`, `freeze_remove`, `freeze_list`, `freeze_clear`; `label_add`, `label_remove`, `label_list`, `label_at`, `label_load`, `label_save` | as the verbs say |
| **Chip views** | `dma_channels`, `dma_log_op`, `register_flow_op`, `coprocessor_history`, `vectors`, `tilemap_entry`, `tile_pixels`, `sprites`, `palettes`, `audio_channels`, `set_channel_muted` | reads, but the `_op`s that arm and `set_channel_muted` change |
| **Expressions** | `evaluate(processor, text)` (value or the evaluator's words), `symbols(filter)` | reads |
| **Cheats** | `cheat_op(op, JSON)` (add, poke, Game Genie, ROM patch, list, enable, disable, remove, clear, save, load, prune, database queries), `cheat_codecs` (what the session's system accepts) | changes, but list and queries read |
| **Trace** | `trace(processor, count)`, `trace_off` | changes |
| **Telemetry** | `telemetry(kind)`: CPU, video, APU, coprocessor registers, sprites, palettes, audio channels, hardware load; `tile_sheet`, `palette_swatch` (RGBA spans); `audio_peek` | reads |
| **States** | `state_save(path)`, `state_load(path)`, `state_default_path` | changes; reads |
| **Logs** | `log_switch(on or off)`, `log_status` | changes; reads |
| **Events** | `events_drain(subscription)` | reads |

**One rule fixes the boundary of each group: the service exposes what a command asks, never what a core reports.**
Today's registries mix both faces (§1.3). The service carries the first (`AddBreakpoint`, `RemoveWatch`, `Tail`) and
none of the second (`NoteWrite`, `ShouldBreak`, `RecordRead`), which stays between Crystal and its engines: the C#
cores feed the registries in-process as now, and a v1 core receives them as tables through its debug exports, as
`CoreDebugBridge` does today (`EmuSen/Cores/Native/CoreDebugBridge.cs`).

**The formatting stays the command's.** The service returns values and records; the command prints them in today's
words and columns. `EmuSen_Debugging_Tools_Reference_v5.md` §3.1a's rule, "the core decodes; the command formats", gains
a third layer and keeps its meaning: the core decodes, the service carries, the command formats. The one exception is
`summary`, whose text is the core's, as it is today.

### 9.3 Every member of `IDebugTarget`, mapped

| `IDebugTarget` / `ICoreTelemetry` member | Session service |
|---|---|
| `GetMemorySpaces()`, `IDebugMemorySpace.Read`, `.Write`, `.HasSideEffects` | `spaces`, `space_read`, `space_write`; side effects and writability in `spaces`' JSON |
| `Watches` | the watch functions |
| `FrameLog` | the frame-log functions |
| `Breakpoints`, `CoprocessorBreakpoints` | the breakpoint functions, with a processor argument; the coprocessor registry is the processor's (`EmuSen_CoreAPI.md` §6.14 made the same choice) |
| `Coverage`, `CoprocessorCoverage` | `coverage_op` with a processor |
| `CallStack` | `call_stack`, `profile_op` |
| `Labels` | the label functions |
| `AccessCounters` | `counters_op` |
| `Freezes` | the freeze functions |
| `Expressions` | `evaluate`, `symbols`; the evaluator moves to the service's side, because its symbols are the session's |
| `DebugCpus` | `processors` |
| `RegisterFlow` | `register_flow_op` |
| `InterruptVectors` | `vectors` |
| `ResolvePhysical` | `resolve_physical` |
| `BreakConditions` | `bp_conditions` |
| `DmaChannels`, `DmaLog`, `NameDmaDestination` | `dma_channels` (each with its destination's name), `dma_log_op` |
| `Cheats`, `ApplyCheats` | `cheat_op`; applying is Crystal's, at the frame boundary (`EmuSen_CoreAPI.md` §6.12) |
| `Disassemble` (both), `ClassifyStaticReference` | `disassemble`, whose records carry the static reference |
| `TilemapEntryStride`, `DecodeTilemapEntry`, `DecodeTilePixels` | `tilemap_entry` (with the stride), `tile_pixels` |
| `SetChannelMuted` | `set_channel_muted` |
| `CoreName`, `FrameCount` | `info` |
| `MaxSprites`, `CpuRegisters`, `VideoRegisters`, `ApuRegisters`, `CoprocessorRegisters`, `Sprites`, `Palettes`, `AudioChannels`, `HardwareLoad` | `telemetry(kind)`, `sprites`, `palettes`, `audio_channels` |
| `RefreshProviders` | none: refreshing is Crystal's, each frame (`EmuSen_Debugging_Tools_Reference_v5.md` §3.66.5) |
| `RenderTileSheet`, `RenderPaletteSwatch`, `GetAudioSamples` | `tile_sheet`, `palette_swatch`, `audio_peek` |
| `GetSummaryText` | `summary` |

All 44 member names have a home, and one, `RefreshProviders`, leaves the surface because it was never a command's.

### 9.4 The rest of what commands are handed today

| Today | Session service, or elsewhere |
|---|---|
| `ICheatCodeCodec` (two, passed to `CreateDefault`), `supportedCheatSystems` | `cheat_codecs` and `cheat_op`: the codecs are the system packs', owned by Crystal (`EmuSen_CoreAPI.md` §8.2) |
| `ICpuTraceSwitch` (passed to `CreateDefault`) | `trace`, `trace_off` |
| `StateCommand`'s two delegates and its default path | `state_save`, `state_load`, `state_default_path` |
| `CoreCommand`'s core registry | the frontend service's `games` and `open_game` (§9.9) |
| `CoretopCommand`'s and `VstopCommand`'s window openers, Mistress's `pause`, `resume`, `feed` | the frontend service (§9.9) and run control |
| `DianaShLogging.MasterEnabled` | `log_switch`, `log_status`: the switch is the emulator's log writer's, not the shell's |
| `SnapshotStore`, `SearchCommand`'s candidates | the module's per-shell slot (§6.3) |
| `FrameRecorder` | not a command's: Hotaru's F6 recording and Pharaoh's `record`. *Revised at S2 (§13.6): it went to Crystal, not to Hotaru, because it has two frontends and neither may reference the other* |
| `RuntimeStats` | `vstop`'s, decided with Q14 |

### 9.5 The registries' new owner

The ten registries, the cheat engine and `DebugCpu` leave the shell's `Var/` for Crystal, where `EmuSen_CoreAPI.md`
§8.2 already placed "one set of registries per session". Before Crystal's sessions and loop are built, they move into
Crystal's assembly, beneath the shell and beside the cores (§13.2, S2; §16 Q1), so that neither the shell nor a module
references them.

### 9.6 The service's own rules

- **Versioned as the module ABI is** (§11), with its own major and minor, and its table extended only by appending
  function pointers; a caller checks the table's `size` before calling a function a later minor added.
- **Open enumerations** for `op` codes, telemetry kinds and run-control operations: an unknown one is
  `EMUSEN_SVC_NOT_SUPPORTED`.
- **Capabilities, not types.** A session's `info` lists what it supports (breakpoints, coverage, a call stack, chip
  views); a module asks the list, never the engine. Today a command learns a feature is missing from a null registry;
  the service answers `EMUSEN_SVC_NOT_SUPPORTED` with today's words for it (`man cov`'s "not available on this core").
- **No pointer into the session crosses**, as no pointer into a core crosses the core ABI: memory is copied out by
  `space_read`, pictures by `tile_sheet`.

### 9.7 Status codes

The service's band is −1024 to −1279, the platform's (`EmuSen_RustPlatform.md` §3.5), with its own names:
`EMUSEN_SVC_NO_SESSION` −1100, `EMUSEN_SVC_NO_SUCH_SPACE` −1101, `EMUSEN_SVC_READ_ONLY` −1102,
`EMUSEN_SVC_NOT_SUPPORTED` −1103, `EMUSEN_SVC_BAD_ARGUMENT` −1104, `EMUSEN_SVC_NO_SUCH_ID` −1105,
`EMUSEN_SVC_PARSE` −1106 (an expression or a code that does not parse, with the parser's words). A command maps each to
today's words.

### 9.8 What the shell sees of all this

Nothing. The shell copies a service entry from its host to a module's call by name and version, and never reads the
table. No identifier of `emusen_session.h` appears in the shell's crate, and CI checks that (§11.4).

### 9.9 The frontend service

What only a frontend can do, offered as a second, small table, `emusen.frontend@1`, by Mistress and Hotaru and not by
Pharaoh or the terminal program:

| Function | Today |
|---|---|
| `open_window(kind)`: `coretop`, `vstop`, a debugger view | `CoretopCommand`'s and `VstopCommand`'s delegates; Mistress's `CoretopWindowCommand`, `VstopWindowCommand` |
| `activate_game_window` | Mistress's `feed` |
| `games` (JSON: the cores and systems a frontend can open) and `open_game(core, path)` | `CoreCommand` and `HostAction.LoadCore`; retired when Crystal opens sessions itself. Since 2026-10-18 the session service's 1.3 has them, with `open_console` (§13.15) |
| `pause`, `resume`, `is_paused` | Mistress's `pause` and `resume`; retired into run control when Crystal owns the loop |

It is a service like any other: owned by Crystal's header family, never read by the shell, and refused politely
where absent (`coretop -w: this program has no windows`).

---

## 10. The host API

### 10.1 What a host is

A host runs one or more shells and provides their services: the terminal program, Mistress's and Hotaru's console
windows, Pharaoh's `--commands`, WiseMan's tests and, when it gains a runtime suite, the conformance kit. The header is
`dianash.h`; exports `dianash_*`; the rules are §2's, which are the core ABI's and the platform library's
(`EmuSen_RustPlatform.md` §3.3–§3.7: the caller's buffers, nothing kept, statuses and a per-thread last error, the
library never calling the host except through the tables a host itself installs for its own commands).

### 10.2 The calls

```c
uint32_t dianash_abi_version(void);
int64_t  dianash_version_text(uint8_t *out, size_t len);

dianash_shell *dianash_new(const dianash_config *config);   /* null on failure; the words in last_error */
int32_t  dianash_free(dianash_shell *sh);

int32_t  dianash_submit(dianash_shell *sh, const uint8_t *line, size_t len);
            /* DIANASH_DONE, DIANASH_NEEDS_MORE (a block, quote or $(...) open), DIANASH_EXIT (shutdown, quit, exit),
               DIANASH_BUSY (a command still running); never blocks on input */
int64_t  dianash_output_take(dianash_shell *sh, uint8_t *out, size_t len);   /* what the last submit printed */
int32_t  dianash_last_status(const dianash_shell *sh);                       /* $? */
int32_t  dianash_is_read_only(const dianash_shell *sh, const uint8_t *line, size_t len);  /* §6.5's classification */
int32_t  dianash_cancel(dianash_shell *sh);                                  /* any thread */

int64_t  dianash_prompt(const dianash_shell *sh, uint8_t *out, size_t len);  /* "DianaSh #: " or "> " */
int64_t  dianash_history(const dianash_shell *sh, uint8_t *out, size_t len); /* entries, each NUL-ended */
int64_t  dianash_banner(const dianash_shell *sh, uint8_t *out, size_t len);
int64_t  dianash_commands(const dianash_shell *sh, uint8_t *out, size_t len);   /* JSON: name, kind, usage */
int64_t  dianash_requests_drain(dianash_shell *sh, dianash_request *out, size_t count, size_t size);
            /* what only the host can do: clear the screen, the current shell session changed */

int32_t  dianash_set_services(dianash_shell *sh, const dianash_service *services, size_t count);
int32_t  dianash_add_commands(dianash_shell *sh, const uint8_t *manifest, size_t len,
                              int32_t (*run)(const dianash_call *, dianash_result *));
```

- **A host submits a line and takes what it printed.** Output is pulled; nothing is pushed into a host's window.
  `dianash_submit` may wait for a frame boundary while a command changes a session (§6.4), so a host calls it from a
  thread that may wait: Mistress's console window from a worker it already has for that, not its UI thread. A host that
  must not wait asks `dianash_is_read_only` first, as `DianaShInterpreterScheduler.SubmitFromAnyThread` does today, and
  queues the rest.
- **Shell sessions** (`tmux`) are inside one `dianash_shell`; the host learns that the current one changed from a
  request, and reads the prompt and history of whichever is current.
- **`dianash_new`'s config** (a size-prefixed struct) gives the sandbox's root (by default Galaxia's), the account,
  the shipped commands' directory, whether the call is interactive (a terminal) or not (a window), extra banner lines,
  and the budget.

### 10.3 What a host provides

- **Services**, by `dianash_set_services`: each `{ size, name, major, minor, table }`. The shell keeps the entries (a
  host's tables live as long as its shell), copies them into calls, and reads none of them.
- **Its own commands**, by `dianash_add_commands`: a manifest (the same format as §5.3, `kind: host`) and one `run`
  function that serves them, which the shell calls exactly as a module's `run`. A C# host's `run` is an
  `[UnmanagedCallersOnly]` function. This is how Mistress keeps `clear` for its window, and how, during the migration,
  every C# command still runs under a Rust shell (§13.2, S3).

### 10.4 The terminal program as a host

`dianash` is a host of its own library: it reads lines with its line editor (history on the arrow keys, as
`ConsoleLineReader` offers today), submits them, prints output, honours `DIANASH_EXIT`, maps Ctrl-C to
`dianash_cancel`, answers the `clear` request with the terminal's clear, and lets an interactive program own the
terminal while it runs (§7.1). It provides no service. On a system with EmuSen installed it is still emulator-free; a
frontend's console is the place to reach a running game.

*Planned 2026-10-10: a panes mode of this program, a terminal interface for the host, is §18.*

*Built 2026-10-10, in C#: the EmuSen shell program, `emusen-shell` (`EmuSen.Shell`), is this program with EmuSen's commands loaded, `hotaru` among them, until the Rust `dianash` exists; Hotaru folded into it, its game window now a class library that provides the command, and the command's `--output` leaves room for §18's terminal picture as a second value - see `EmuSen_Frontend_Driver.md` §0.*

### 10.5 Each of today's hosts, on the host API

| Host | Today (§1.5) | On the host API |
|---|---|---|
| The terminal | `Program.cs`'s loop | `dianash` |
| Mistress | `CreateDefault` with seven commands of its own; a scheduler per session; the emulation thread drains | `dianash_new`; `emusen.session` and `emusen.frontend` set; `clear` added; read-only lines on the UI thread, the rest on its worker; no drain from the emulation thread |
| Hotaru | the F4 prompt, the always-live terminal, `TakeHaltedLine` while halted, the `HostAction`s | the same calls; while halted, the prompt thread submits directly, since a halted session applies changes at once (§6.4); resume and step arrive as the session's events |
| Pharaoh | an interpreter over its bundle's target | a shell with `emusen.session` over its run, its unknown verbs submitted as lines |
| WiseMan | `CreateDefault` over fakes and cores | the C# facade of §13.3 over the host API, with a fake `emusen.session` provider for the cases that use fake targets |
| The kit | none | the host API, when its runtime suite exists (`EmuSen_CoreAPI.md` §12.2) |

---

## 11. Versioning and stability

### 11.1 Three surfaces, one policy

The host API, the module ABI and the session service each have a version `(MAJOR << 16) | MINOR`, answered by their
own `abi_version` (the session service in its table's header field). Each follows `EmuSen_CoreAPI.md` §4 in full: the
major is 1 and is expected never to change; every minor of a major is accepted, older or newer; what a peer offers is
discovered by name, by size and by capability, never by comparing minors; a documented defect of one minor is
corrected by an addition.

### 11.2 What a minor may add, and what needs a major

**A minor may add** a new export or table entry at the end, with a capability where it is optional; a field at the end
of a struct whose zero means "absent"; a new value of an open enumeration (a status in an unassigned part of a band, a
request kind, an `op` code, a telemetry kind); a manifest field; a builtin (it is new syntax a script could not have
used); a new service.

**A major is needed for** removing or renaming an export, a table entry or a manifest field a peer may rely on;
changing a signature, a struct field's offset or meaning, or a code's number; making an optional thing required;
changing the ownership rules, the length-query idiom, the call-only rule of §6.0 or the read-only refusal; and, for the
shell, changing what a script already means: a builtin's behaviour, the grammar, expansion or the output of a shipped
command. **The language is a stable surface too**, held by the golden outputs (§13.1): a change to what a recorded line
prints is a breaking change, made only with a recorded reason and a new golden file.

**The manifest format** has its own version (`"manifest": 1`), extended only by fields a reader may ignore.

### 11.3 An old module, a new shell, and the reverse

- **A newer shell with an older module**: the shell passes its `dianash_call` at its own size and reads only the
  prefix of `dianash_result` the module's minor writes. The io table is the shell's own and grows by appending; a module
  never calls an entry past the `size` it was given. A module built against version 1.0 keeps working on every 1.x shell.
- **An older shell with a newer module**: the module reads the `size` of the call and the io table and must work
  without what they lack, or answer `DIANASH_NOT_SUPPORTED` with words that say which shell version it needs. Its
  manifest's `abi` names the least minor it needs; a shell older than that refuses it at lookup with
  `<command>: needs DianaSh 1.N`, without loading it.
- **A module of another major** is refused at lookup, from its manifest, without loading.
- **A service of an older minor than a manifest needs** stops the call with `DIANASH_SERVICE_MISSING`; a newer one is
  fine, because the module calls only what its minor knows.
- **Adapting.** No adapter layer between majors is planned, because no major 2 is planned. If one is ever made, the
  shell of major 2 would carry a shim table for major-1 modules, as the core ABI's deprecation rule allows
  (`EmuSen_CoreAPI.md` §4.5); nothing is removed from major 1.

### 11.4 How the surfaces are guarded

As the core ABI's (`EmuSen_CoreAPI.md` §5): each header is written by hand and normative; its Rust definitions are
checked against it with bindgen and every field's offset by name; a recorded baseline per surface
(`abi/dianash/v1/baseline.txt`, `abi/emusen-session/v1/baseline.txt`) is diffed on every pull request, a removed or
changed line failing it and an added line requiring the baseline in the same request. And two checks of this page's own:

- **The shell's crate names no service.** A CI step fails if any identifier declared in `emusen_session.h` or the
  frontend service's header appears in `Platform/dianash/`, and if the crate's dependency graph contains any crate of
  the emulator but Galaxia's.
- **No module links Galaxia.** A shipped module's symbol table is checked for none of `emusen_galaxia`'s exports and
  its crate graph for no `emusen-galaxia`.

---

## 12. Cross-platform

### 12.1 Paths

- **The shell's `/` is the same on every system**: Galaxia's `UsrHome`, the player's home in the tree (`EmuSen_Galaxia.md`
  §3, §3.4), which on macOS is `~/Library/Application Support/EmuSen/home` and elsewhere `<root>/home`. The shell always
  shows `/`-separated paths rooted there, including on Windows, where a drive letter never appears inside the shell.
- **The sandbox's comparison** is today `OrdinalIgnoreCase` on every system (`DianaShSandbox.IsInsideRoot`). On Linux
  that admits a path that differs from the root only by case, which on a case-sensitive file system is a different
  directory: a path outside the root can be accepted as inside it. It is reproduced byte for byte in the migration's
  first step so that the golden outputs hold, and is a finding for §16 Q15 to decide, with a test of its own.
  *Corrected 2026-10-15, as Q15 decided, in both shells (§13.7): case is ignored only on Windows and macOS, whose
  file systems ignore it by default.*
- **Real paths given to programs** are native: backslashes and a drive on Windows.
- **Line endings**: files the shell and its modules write end lines with `\n` on every system, as today's `File.WriteAllText`
  of `\n`-joined text does.

### 12.2 Processes

`posix_spawn` on Linux and macOS, `CreateProcessW` on Windows, through Rust's standard library; the environment and
quoting as §7.1 and §7.3 say. A program is never run through a system shell, so no system shell's quoting can change an
argument.

### 12.3 Terminals

- **Raw input and the line editor**: termios on Linux and macOS, the console API on Windows with virtual-terminal
  sequences enabled. Today's `ConsoleLineReader` does history, left and right, Home, End, Backspace and Delete, anchored
  past the prompt, with known gaps in wrapping (`Dev/ConsoleLineReader.cs`). Version 1 keeps exactly that, written for
  the three systems, rather than taking a full line-editing crate whose behaviour differs (§16 Q12).
- **Encoding**: UTF-8 in and out; on Windows the console's code page is set to 65001 for the shell's lifetime.
- **A console window is not a terminal.** Mistress's and Hotaru's windows submit whole lines and print output; there
  is no raw mode, which is why `nano` refuses them today and will, as a program, refuse when `dianash_new`'s config says
  the shell is not interactive.

### 12.4 Where things are installed

| | Shipped modules and programs | Player's modules | Player's programs |
|---|---|---|---|
| Linux, a published tree | `lib/EmuSen/dianash/` (under the `/lib/EmuSen` `man hier` already names) | `home/lib/dianash` | `home/bin` |
| Windows, a published tree | `lib\EmuSen\dianash\` | the same, in `home` | the same, in `home` |
| macOS, `EmuSen.app` | `EmuSen.app/Contents/PlugIns/dianash/`, the bundle's place for loadable code, signed with the bundle | `~/Library/Application Support/EmuSen/home/lib/dianash` | `…/home/bin` |
| A checkout | each frontend's output directory, as cores are (`EmuSen_CoreAPI.md` §7.1) | the checkout's `home` | the checkout's `home` |
| The standalone `dianash` alone | beside the binary, in `dianash/` | its root's `home/lib/dianash` | its root's `home/bin` |

**The macOS bundle.** A module inside the bundle is signed with it and loads under the bundle's signature. A module or
program the player adds is outside the bundle: it is loaded with `dlopen` after approval (§5.4), which the bundle's
ad-hoc signature permits today because the hardened runtime is not enabled
(`Publish/DianaShPublishLayout.targets`, `DianaShSignMacAppBundle`). If the hardened runtime is ever enabled, library
validation would refuse a player's module unless the bundle carries the
`com.apple.security.cs.disable-library-validation` entitlement; that is recorded so the choice is made knowingly. A
downloaded program carries the quarantine attribute and Gatekeeper checks it on first run (§7.5's first-run row).

**Windows** loads a module by its full path with `LoadLibraryExW` and `LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR`, so a
module's own dependencies are found beside it and never on `PATH`.

---

## 13. The migration

### 13.1 The oracle: golden outputs, recorded first

Before any line moves, WiseMan records what today's shell prints, byte for byte: a corpus of scripts and the exact
standard output, exit status and host action each line gives, over fixed inputs.

- **Coverage**: every one of the 83 commands, the six builtins and five aliases, and every verb and form its usage
  line names (the usage lines are the checklist, read by the recorder), including each failure the code words; the
  language's own cases (the 645 of `DianaSh/` exercise most of them, with assertions that check parts; the corpus
  checks the whole); and `man` for all 98 pages and `help`.
- **Targets**: a fake `IDebugTarget` whose every member answers fixed data, for the commands' formatting; and each C#
  core and each v1 core on images the tests build (`SyntheticRom` for the SNES, and the other systems' builders), for
  the commands that read live state, run for a fixed number of frames so the state is deterministic. No commercial ROM
  is used, and the corpus contains no ROM data.
- **The files** are committed, one per command (`EmuSen.WiseMan/DianaSh/Golden/<command>.golden`), as text: each case
  its line, then its output, status and action. A difference fails with the first differing byte and the case.
- **What is not golden**, named in the corpus with its reason: anything that prints a time, a process id or a path of
  the machine the test runs on (`vstop`'s numbers, `pwd` outside the sandbox's fixed root); these are matched by
  pattern, and the pattern is in the file.
- **A command only the Rust shell has** (added 2026-10-25 with `chd`, the first module with no C# class,
  `EmuSen_GamePackages.md` §12.3): its file carries the line `shell rust`, and is run, and expected among the
  commands, on the Rust shell alone; it is recorded there. The listings of `help` and of `man` with no argument, which
  name every command, are held as patterns in which that command's usage lines are optional, so that both shells
  match them and a recording from the C# shell keeps them.

### 13.2 The steps

**Decided 2026-10-11:** the go for S1. *S1 built the same day* (§13.5), *and S2 after it* (§13.6).
**Decided 2026-10-15:** port DianaSh to Rust now, S3 first, after the sandbox's escapes; and S4 after S3's gate.
*S3 built the same day* (§13.7), *and S4* (§13.8). S6 and S7 wait for the tester.

No step leaves a broken shell; each has its gate, and each is its own commit or group of commits.

| Step | What | Gate |
|---|---|---|
| **S0** | The golden corpus of §13.1, recorded from the C# shell | it covers every usage line (a test enumerates them); two recordings on two runs are identical |
| **S1** *(done 2026-10-11, §13.5)* | The C# shell reorganised without changing what it prints: the 98 manual pages out of `ManPages.cs` into files read by the C# `man`; the five aliases and the read-only fast path made one table; `summary` and `log` turned into ordinary command classes; `Var/`'s shell state (`CommandHistory`, the sessions) separated from the debugger's registries | the golden corpus; `ManPageTests` |
| **S2** *(done 2026-10-11, §13.6)* | The service API's header, `emusen_session.h`, and a C# provider over today's `IDebugTarget` and registries, exposing the table through `[UnmanagedCallersOnly]` functions; the registries, the cheat engine, `IDebugTarget`, the telemetry contracts and the system packs moved out of `EmuSen.DianaSh` into Crystal's assembly, `EmuSen.Crystal`, beneath the shell and beside the cores (§16 Q1), which `EmuSen.DianaSh` and the cores reference | a parity class calling every service function and the C# member it maps (§9.3) on the fake target and each core, equal; the golden corpus |
| **S3** *(done 2026-10-15, §13.7)* | The Rust shell core, `Platform/dianash/`, with its builtins, lookup, manifests, the host API in the platform library and the `dianash` program; every C# command still runs, registered with the Rust shell as host commands through `dianash_add_commands` by a C# bridge that calls each `IDianaShCommand.Execute`; the C# frontends choose the C# interpreter or the Rust one by `EMUSEN_DIANASH_NATIVE` (§13.3) | the golden corpus, with the variable unset (the C# interpreter) and `1` (the Rust shell with C# commands); the language's 645 cases both ways |
| **S4** *(done 2026-10-15, §13.8)* | `dianash.text` in Rust, command by command, each replacing its C# bridge entry when its golden file passes | that command's golden file and its WiseMan cases both ways; the same corpus run on Windows and macOS CI runners for the text module, since it must behave identically there |
| **S5** *(done 2026-10-16, §13.11)* | The four EmuSen modules in Rust over the session service, module by module, each command replacing its bridge entry when its golden file passes; `nano` as a program | that command's golden file both ways, on the fake provider and on each core; seeded faults in the module, as each platform step had (`EmuSen_RustPlatform.md` §6.5) |
| **S6** *(state 2 reached 2026-10-16 ahead of S5, by decision, §13.10; the shell as a library green on Windows and macOS CI the same day, §13.11; the standalone program there not yet run)* | The shell to state 2, as `EmuSen_RustPlatform.md` §3.9 defines it: the Rust shell by default, `EMUSEN_DIANASH_NATIVE=0` selecting the C#, a missing library falling back with one line in the error log | a full WiseMan run with the variable set; the tester's word |
| **S7** | State 3: the C# interpreter and the C# commands deleted | below |

**When C# command code may be deleted.** Only at S7, and only when all of these hold: the golden corpus passes with
every command in Rust; the corpus has been run on all three systems; the shell has been in state 2 through at least one
release played; and the tester agrees. Until then every C# command class stays, reachable with `EMUSEN_DIANASH_NATIVE=0`.
A C# command is not deleted when its Rust module passes, only marked as superseded.

**Crystal's steps run beside these and do not wait for them.** `EmuSen_CoreAPI.md` §10.2's D1 to D5 build the
runtime; when Crystal provides `emusen.session`, S2's C# provider retires. Nothing in S0 to S7 depends on that
having happened, because the provider is behind the table.

### 13.3 How the C# frontends call the Rust shell meanwhile

- **A C# facade, `DianaShInterpreter`, keeps its public members** (`Submit`, `Execute`, `IsAwaitingMoreInput`,
  `History`, `CommandNames`, `GetWelcomeBanner`, `TryGetReadOnlyFastPath`, `CurrentUser`), as the platform's facades
  did (`EmuSen_RustPlatform.md` §3.9). With the switch on, each forwards to the host API; with it off, to the C# it is
  today, moved into a nested `Managed` class and not rewritten. `HostAction` is synthesised by the facade from the
  host API's status and requests and the session's events, so Hotaru's loop does not change until it moves to events
  itself.
- **The switch** is `EMUSEN_DIANASH_NATIVE`, read once per process, through Galaxia's `PlatformSwitch`: in state 1 the
  C# unless the variable is `1`; in state 2 the Rust unless it is `0`.
- **`CreateDefault`'s parameters** become the services a frontend sets: its codecs, trace switch and supported systems
  are what the C# provider of S2 reads from the bundle, and its sessions are the Rust shell's own.
- **The scheduler stays for the C# side** until S6, and with the Rust side the facade answers its four entry points
  from the host API (§10.2: `dianash_is_read_only` for `SubmitFromAnyThread`; `DrainPending` with nothing left to
  drain, since Crystal's boundary does the waiting).

### 13.4 S0 as built: the corpus, its harness, and what it recorded

*Built 2026-10-10; the counts below are of that commit.*

**What is in it.** Ninety files in `EmuSen.WiseMan/DianaSh/Golden/`: one for each of the 83 commands (`[` is
`bracket.golden`, since the bracket cannot begin a portable file name), one for each of the six builtins (`help`,
`man`, `summary`, `export`, `unset`, `source`), and `shell.golden` for the two lines of `help` that describe the
language rather than a command (control flow, pipes, quoting, redirection, history expansion, substitution). An alias
is recorded in its command's file (`c` in `resume.golden`, `quit` in `shutdown.golden`, `s` in `step.golden`, `jobs` in
`ps.golden`, `hexdump` in `xxd.golden`) and once more, typed in capitals, in `shell.golden`. There are 524 cases and
1,944 recorded lines, about 540 KB; `man.golden` holds one case per page, 98 of them, so a change to one page fails one
case and not the file.

**The format.** Line-oriented text in which every recorded block is preceded by its length in bytes, so that an output
may hold any byte, a newline included or a final newline absent, without an escaping rule for a second shell to
reimplement. A file is a header, then its cases:

| Line | Meaning |
|---|---|
| `command <name>` | the command the file is for |
| `usage <n> <text>` | the command's usage lines, numbered, as the live shell gives them; written by the recorder |
| `uncovered <n> -- <reason>` | a usage line no case can run, and why |
| `unreachable "<text>" -- <reason>` | a failure the code words that no input reaches, and why |
| `case <title>` … `end` | one case, run on its own shell |
| `target <name>` | what the shell is attached to (below) |
| `frames <n>` | frames run on a core before the first line (default 2) |
| `covers <n>[-<m>],…` | the usage lines the case exercises |
| `sessions none` | the shell is built with no session manager, as a host without `tmux` builds it |
| `vary numbers` | the case's outputs are recorded as patterns in which every number may differ |
| `file <path> <bytes>` | a file written into the case's home before the first line, its bytes following |
| `$ <line>` | a line given to `Submit` |
| `out <bytes>` / `match <bytes>` | what it printed, or a regular expression the whole of it must match |
| `status <code> [more]` | `$?` after the line, and `more` where the shell asked for another line |
| `action <record>` | the host action it returned, as the record prints, or `none` |

The recorder rewrites every file from its `$` lines, so a case is written by its inputs alone; a pattern already in a
file is kept as long as the output still matches it, which is how a hand-written pattern survives a re-recording.

**How a case runs** (`GoldenRunner`). Each case gets a new home, a new target and a new shell. The home is a temporary
folder that `DataStore` and `ConfigStore` are pointed at, the legacy configuration folder too, so no file of the machine
the test runs on is read; it is given the skeleton the shell itself creates, then the case's files, and the working
directory is its root. Accounts other than `root` are removed, since the account registry is process-wide, and the
logging switch is set to its default, off. The shell is `CreateDefault` with what a host adds to it: `core` over
`CoreCatalog.Registry`, with the hosts' answer to which consoles an engine starts empty (since 2026-10-17, §13.13);
`state` over a save and a load that write and read a file, defaulting to a fixed path in the home's save-state folder;
the bundle's two cheat codecs and trace switch; a session manager unless the case says otherwise; and the build's
supported cheat systems.

**How the status is read.** `$?` lives in the interpreter's variables, which nothing public reads. `EmuSen.DianaSh`
grants WiseMan friend access (`InternalsVisibleTo`), and the interpreter answers the harness through two `internal`
members named for what they answer: `LastExitStatus`, the last line's `$?`, and `RegisteredCommands`, every command in
the order `help` lists them, which the usage gate reads. Neither prints anything, so the recordings are the same with
them as without. The corpus was first recorded with the two read by reflection on the private members; that was
replaced the same day, since the project's rule of 2026-10-10 is that nothing reaches past a surface by reflection.
Friend access keeps the check at compile time: renaming either member breaks WiseMan's build, not its first case.

**The targets.**

- `fake`: `GoldenDebugTarget`, whose every member answers fixed data: four plain spaces, an `IO` space with read side
  effects, an unsized one, and a 64 KB `CpuBus` mapping RAM at `$0000`, a twelve-instruction program at `$8000` and two
  vectors; a made-up instruction set of four mnemonics, so that `disasm`, `callers`, `readers` and `writers` have
  something to find; two processors, the second unable to halt; and every registry.
- `bare`: the same with every optional member absent, for each command's "this core does not report" branch; `nobus`:
  the same without `CpuBus`, for the one failure of `vectors` that needs it.
- `none`: no target, for every command's "No ROM loaded".
- The cores: `venus`, `venus-sa1`, `venus-gsu` and `venusrt` on `SyntheticRom`'s images, `moon` and `moonrt` on
  `SyntheticNesRom`'s, `mercury` and `mercuryrt` on `SyntheticGbRom`'s, `mars` and `marsrt` on `SyntheticN64Rom`'s, and
  `nephrite`, the v1 core, on `SyntheticMdRom`'s; each built through `CoreFactory`, run for its frames and attached
  through `CoreFactory.Bundle`, as a frontend attaches one.

The N64 image carries a branch-to-self at its entry point. Without it the image is zeros past its header, and the C#
Mars core, running them, threw `IndexOutOfRangeException` in `Cpu.StepBlock` within the first frame. That was a defect
of the core, not of the shell; *fixed 2026-10-10*: the CPU runs the zeros as no-operations up to the last 256 bytes of
RDRAM, where the dispatcher looked up a write mark past the end of its table (`Mars_Recompiler.md` §21). The corpus
keeps the branch-to-self, so that the debugger's commands meet a core sitting at its entry point.

**What is matched by pattern.** The home's real path, the temporary folder's and the install's are written as `<ROOT>`,
`<TEMP>` and `<INSTALL>` before anything is compared. One case is a pattern, `ls -l`'s, because it prints modification
times. Nothing else in the corpus printed a time, a process id or a path of the machine; the commands that would,
`vstop` and `coretop`, refuse in a test host because its standard streams are redirected, and their refusal is what is
recorded.

**The gate**, `GoldenCorpusTests`:

1. Each file's cases reproduce their recording; a difference names the file, the case, the line, the first differing
   byte and the text around it.
2. There is one file for every command and builtin, and no other.
3. Every usage line is covered by a case, and every file's recorded usage lines equal the live ones, so a usage line
   added to a command fails the gate until a case covers it. One line is listed as uncovered: `cheat db update`, which
   downloads the libretro database over the network.
4. Every literal a `Fail(…)` call words, cut at its interpolations and kept where eight characters or longer, appears
   in an output of its command's file, or of any file for the shared helpers and the interpreter, or is listed as
   unreachable. Ten are listed: those of `mv` and `rm`, which are suspended and refuse before they read an argument, and
   `userdel`'s refusal to remove the shell's own account, which no input reaches, because only `root` may remove an
   account and `root` is refused by name first.
5. Every page in `ManPages.cs` has a case.
6. Two recordings of the whole corpus in one process are byte-identical. The comparison of item 1 always runs in a
   process other than the one that recorded, so it is the second half of "two recordings on two runs".

On 2026-10-10 all six held: 95 tests passed.

**What the corpus records that looks wrong.** These are recorded and not fixed. The corpus holds them as today's
behaviour, so that at S3 to S5 the Rust shell must reproduce each of them, case for case, until it is decided; a fix
made to the C# before then is a re-recording of the cases it changes, and should be its own commit for that reason.

The language:

1. A failure reported with status 0: a redirection that fails (`cat < nosuch.txt`, `echo x > /../escape.txt`,
   `echo x > Logs`), a syntax error (`|`, `&& echo x`), and `!99` for a history entry that does not exist (`shell`).
2. A line that ends in `|`, `||` or `>` asks for another line, and then rejects whatever line comes as a syntax error,
   so the continuation it asked for cannot be given (`shell`, "stray operators").
3. `source` returns 0 whatever the script did: its last command failing, the nesting limit, an unterminated block
   (`shell`, "a script's failures and its status").
4. `continue` at the prompt resumes emulation and prints nothing, while `break` does nothing (`shell`).
5. `A=1 echo x` leaves `A` set in the shell, where bash scopes it to the command.

The commands:

6. Identifiers are read as hexadecimal while they are printed in decimal: `bp remove 10` reports "#10 removed" and
   removes #16, and `bp off 99` reports "No breakpoint #153"; `cheat`, `framelog` and `watch` read their identifiers the
   same way (`bp`, "ids are read as hexadecimal").
7. Counts differ in base between commands: `step` and `trace` read hexadecimal (`step 10` steps 16), `runto scanline`,
   `runto frame`, `cophist` and `mute` decimal (`step`, `trace`).
8. A refusal's status depends on the command: `bp on 99` fails with 1 and `bp remove 99` with 0, and most
   "Usage:" and "not found" answers are 0.
9. `su` asks for no password, even for an account that `passwd` has given one, and `su TESTER` switches to an account
   spelled differently from the one that exists (`su`).
10. *Fixed 2026-10-10, on the tester's decision.* `dump`, `load` and `cov save|load` joined their file name to
    `Logs/<core>/` without the sandbox's resolution, so `../../../x` read and wrote outside the root (shown inside the
    corpus's temporary folder), and an absolute name would reach any path; `core` and `state` passed their paths to the
    host unresolved too, and `test -f`/`-d`, found in the sweep that followed, asked the machine about the path as
    typed. Each now resolves through the sandbox and refuses above the root, with two stated rules where a path
    legitimately lies outside it: `core` takes an absolute path inside the configured ROM library
    (`AppSettings.RomDirectory`), and `state` with no path uses the frontend's own default as it gives it. The cases
    were re-recorded in the commits that changed them; `SandboxEscapeTests` shows each escape. The sweep found every
    other command already resolving its paths through `TryResolve`; it also found that the check is on a path's text
    and does not follow a symbolic link, which is left as it is (`man hier`).
11. `write` keeps the low byte of a wider value and reports success for an address past the end of its space; `load`
    reports every byte loaded when some fall past the end; `dump … 0` prints its range as `0x0-0xFFFFFFFF` (`write`,
    `load`, `dump`).
12. `memfind` takes a final word of decimal digits as the limit, so `memfind RAM 03 0A 11` searches for two bytes, not
    three; and the form its own message offers, `'00 ?? 11'`, is refused, because spaces inside one word are not
    removed (`memfind`).
13. `mute` reports success for a channel that does not exist, and any word but `on` unmutes; on Venus, `channels` still
    shows a channel unmuted on the line after `mute 0 on` (`mute`).
14. `wc` counts one line more for a file than for the same text piped (`wc -l a.txt` gives 3 for two lines, `cat a.txt
    | wc` gives 2); the file's count is the wrong one (`wc`).
15. `cp a.txt a.txt` fails with the platform's message about a file in use by another process (`cp`).
16. `regs` and `setreg` do not accept the `cop` word that every other scoped command accepts (`regs`, `setreg`).
17. `pal 9`, for a palette that does not exist, prints nothing with status 0 (`pal`).
18. A malformed number reaches the player as the platform's own message, "Could not find any recognizable digits."
    (throughout).

The cores, outside the shell, shown by it:

19. The Rust cores' debug targets publish less than their C# counterparts: VenusRT no hardware break conditions, DMA
    channels, vectors, CPU registers, audio channels, sprites, palettes or coprocessor history; MoonRT and MercuryRT no
    call stack, and both report frame 0 after two frames, MoonRT naming itself Moon. *The defects fixed 2026-10-10*
    (`EmuSen_CoreAPI.md` §30.2, §30.3): frame 0 and the stale registers were the targets' first snapshot, taken from a
    mirror not yet loaded from the Rust core's state, and VenusRT's missing CPU registers the same defect in the v1
    target; MoonRT now names itself. The rest is missing surface, inventoried item by item with the route each would
    take in `EmuSen_CoreAPI.md` §30.6. *Built 2026-10-10* (`EmuSen_CoreAPI.md` §31): what needs no surface, through the
    host and the SNES's system pack, so VenusRT now publishes palettes, tiles and the tile sheet, DMA channels,
    vectors, the flag symbols and the stack depth, freezes, its coprocessor in `regs`, the summary's CPU line and the
    engine's load bar. Sprites need the video registers and remain with them; the hardware break conditions, audio
    channels, coprocessor history and MoonRT's and MercuryRT's call stack from the start of a run remain as §30.6 and
    §31.4 leave them.
20. Nephrite names its main processor `m68k`, not the `cpu` every command takes as the main one, so `regs cpu` and
    `disasm cpu` refuse there; and its `tile` throws `IndexOutOfRangeException`. *Fixed 2026-10-10*
    (`EmuSen_CoreAPI.md` §30.1, §30.4): `cpu` names the first processor, which both surfaces already state is the
    main one, and `tile` refuses on a core that decodes no tiles.
21. `core`'s registry has no entry for the Genesis, so a v1 core cannot be loaded with it. *Fixed 2026-10-10*
    (`EmuSen_CoreAPI.md` §30.5): the registry is built with the catalog's other lists, discovered consoles included.

The cases findings 19 to 21 changed were re-recorded in a commit of their own, eleven files: `bp`, `bt`, `core`, `cov`,
`disasm`, `eval`, `man` (the `cpus` and `tile` pages), `regs`, `setreg`, `summary` and `tile`. No other file moved.

### 13.5 S1 as built: the C# shell reorganised, and nothing it prints changed

*Built 2026-10-11, in five commits of code, each of which passed the gate below on its own.*

S1's row asks for four changes and forbids one effect. The four are taken in turn, with what moved and why it moved
there; the effect forbidden, a change to any recorded output, is §13.4's gate, and its result closes the section.

**The manual pages.** The dictionary in `ManPages.cs` held 99 pages, not the 98 counted in §1: `hotaru`'s page was
added the same day as that count, when Hotaru became a command (`EmuSen_Frontend_Driver.md` §0). Each page is now a
file of its own, `EmuSen.DianaSh/DianaSh/Lib/dianash/<module>/man/<name>.<section>`, 146,015 bytes in all, and
`ManPages.cs` went from 3,071 lines to 43, a reader that keeps the old `Lookup(name)` so that `man`, `ManPageTests`
and `ThemeVocabularyTests` call it unchanged. The layout follows from three things this page has already fixed.
§5.3's manifest names a command's page by a path relative to the manifest, `man/mem.1`; §12.4 puts every shipped
module in one folder, `lib/EmuSen/dianash/` in a published tree; and §5.5 puts the shell's own pages in the same
folder's `man/`. The installed form is therefore one flat folder, `dianash/man/`, holding every page, and the files are
built into exactly that: a published tree has them at `/lib/EmuSen/dianash/man/`, and a checkout in each frontend's
output directory, which is §12.4's row for a checkout. When S4 and S5 write the first manifests, each will name pages
that are already where it looks for them; nothing is moved again. In the source the pages are grouped by the owner
§8.1 assigns them, one folder per future module, so that the module that will carry a page is visible from its path
and a module's build can take its folder whole:

| Folder | Pages | Which |
|---|---|---|
| `dianash` | 30 | the builtins of §4.2 that have a page, the language's five (`if`, `for`, `while`, `break`, `continue`), and `hier` |
| `dianash.text` | 16 | §8.1's text module |
| `emusen.debug` | 26 | §8.1's debug module |
| `emusen.trace` | 16 | §8.1's trace module, `log` and `resume` included |
| `emusen.cheat` | 1 | `cheat` |
| `emusen.memcard` | 1 | `memcard`, the card manager, added 2026-10-15 (`EmuSen_SaveMedia.md` §6.4) |
| `emusen.session` | 4 | `core`, `state`, `summary`, `coretop` |
| `nano` | 1 | the one shipped program |
| `host` | 5 | what a frontend registers or reads itself: `pause`, `feed`, `vstop` (§16 Q14), `hotaru`, `theme` |

The section is the manual's own: 1 for every command, 5 for `theme`, which describes a file format, and 7 for `hier`,
as `hier(7)` is filed on Linux. `[`'s page is `[.1`, as Debian's coreutils ships it; the corpus called its file
`bracket.golden` for want of a portable file name, but a page's name is what `man` is asked for, and from S4 the
manifest's `man` field could name any file. The folder is found the way the shell finds the rest of the install, through
Galaxia: `ConfigRoot.ShellModulesDirectory`, the program's own directory joined with `ShellModulesDirName`, `dianash`,
which `EmuSen.DianaSh.csproj`'s copy rule also uses. It is outside the sandbox, as `man hier` says the install is.
`ManPages` indexes the folder once per process, accepting `<name>.<digit>` and nothing else, and reads a page when it is
asked for; a file ends in the newline a text file carries, and the page printed does not, which is how the dictionary
held them. A `.gitattributes` beside the pages fixes their line endings at LF, since `man` prints the bytes and a
checkout that rewrote them would fail every page's case on Windows.

The move was checked before it was trusted. The pages were extracted from the C# literals by a script, and each
extracted page was compared with its recorded output in `man.golden` before `ManPages.cs` was cut: 99 of 99 were
byte-identical. Measured on 2026-10-11 from a `linux-x64` publish of the standalone shell, the 99 files land in
`lib/EmuSen/dianash/man/`, and `man ls` and `man hier` print from them.

**The aliases and the read-only fast path.** The five aliases were mapped by the same five `if` lines twice, in
`Dispatch` and in `TryGetReadOnlyFastPath`, and each then looked the command up in its own way. `Bin/CommandTable.cs`
now owns the three facts the two shared: which name runs which command, which names are aliases of which commands, and
whether a name is read-only. `Dispatch`, `man` and the fast path all ask it, and a subshell shares its parent's table as
it shared the two collections before. Whether a command is read-only remains each command's own declaration
(`IDianaShCommand.IsReadOnly`), as it will be each command's `read_only` in a manifest (§5.3); the table is the one
place that answers the question for a name, aliases included.

**`summary` and `log` as ordinary command classes.** `summary` was answered inside `Dispatch`; it is now
`SummaryCommand`, registered like every other command. Two details keep its outputs as they were. It is placed after a
host's own commands, because `help` has always listed it there and an ordinary registration would have put it before
`core`, `state` and `hotaru`; and it reports itself not read-only, because the fast path classified it so as a builtin,
and declaring it read-only would let Mistress and Hotaru run `GetSummaryText` off the emulation thread while a frame
runs.
In substance it is read-only; making it so is a change of threading, not of organisation, and is left to be decided.
`log` was already a class, but it wrote `DianaShLogging.MasterEnabled`, a static in the shell's `Etc/` that the
emulator's log writer reads through `DebugSettings.MasterLoggingEnabled`, and `WatchRegistry` reads directly. The switch is
the emulator's (§9.4, §17.1 item 5), so it moved to the debugger's side as `Var/DebugLogging`, where it leaves the shell
with the registries at S2, and `LogCommand` is handed a getter and a setter, as `StateCommand` is handed its delegates,
instead of naming a process global. The shell's `Etc/` now holds no static that another component reads.

**`Var/`.** `CommandHistory`, `DianaShSession` and `DianaShSessionManager` moved to `Var/Shell/`, namespace
`EmuSen.DianaSh.DianaSh.Var.Shell`. What remains in `Var/` is the debugger's: the ten registries, the cheat engine,
`DebugCpu`, the logging switch and `FrameRecorder`. The shell's three files were moved rather than the debugger's
nineteen because the cost of a namespace is paid by its users: the three classes are named in 13 files besides their
own, the 44 public types of the nineteen in 195 (counted on 2026-10-11), and S2 renames the registries' namespace once
anyway, when it moves them to Crystal's assembly. Nothing in either half referred to the other, so the separation
needed no code changed beyond the namespace.

**The gate.** Measured on 2026-10-11: the golden corpus, `ManPageTests` and `ThemeVocabularyTests` pass, 96, 103 and 6
tests; with the rest of the blast radius (all of WiseMan's `DianaSh/` cases, `CoreRegistrationEquivalenceTests` and its
golden file, the sandbox and history tests, the Mistress and shell console-window tests, the leaf-assembly check and
Galaxia's tests), 1,259 pass and 8 are skipped, the same 8 `SettingPathsOpenTests` that carry a skip of their own
(`EmuSen_Config_Reference.md` §7). No file in `Golden/` changed, and none had to. Four test files changed, none of them
a recording: `GoldenCorpusTests`' item 5 lists the pages through `ManPages` instead of reading `ManPages.cs` with a
regular expression; the harness names `DebugLogging`, and one test constructs `LogCommand` with its two delegates; and
three of the four gained the `Var.Shell` using. `ManPageTests` and the corpus's list of builtins still name `summary` as a
builtin, which is no longer true and does no harm, since each takes a registered command first; they were left as
they were so that the gate ran unchanged.

**Found while building it.** None of these is fixed here; each is a change to what the shell prints or to where a file
lies, and so is not S1's.

1. Two outputs not in the corpus did change, as a direct consequence of `summary` becoming a registered command: it is
   now among `CommandNames`, and among the names an unknown command's hint is drawn from, so `sumary` now answers "Did
   you mean 'summary'?", which it could not before. A case for it belongs in `summary.golden`, recorded in a commit of
   its own.
2. An exception thrown by a core's `GetSummaryText` used to escape `Dispatch` and end the whole line, leaving `$?` as it
   was; it is now caught there, as every command's is, and gives status 1 with the same "Error:" text. No case reaches
   it, since no target in the corpus throws there.
3. `man hier` does not mention `/lib/EmuSen/dianash/man/`. Saying so changes the recorded page, and is a re-recording
   of `man.golden`'s `hier` case.
4. On macOS the bundle's program directory is `Contents/MacOS`, so the pages are in `Contents/MacOS/dianash/man/`,
   not §12.4's `Contents/PlugIns/dianash/`. `PlugIns` is the bundle's place for loadable code, and whether the bundle's
   signing accepts plain files there has not been tried on the Mac; the pages move there with the first module, at S3,
   which is one line of Galaxia and one rule of the publish layout, and leaves the `man/<name>.<section>` form as it is.
5. `ShellModulesDirectory` exists in Galaxia's C# only. Its one reader until S3 is the C# `man`; the Rust Galaxia gains
   the entry, and the parity test with it, when the Rust shell needs it.
6. `FrameRecorder` is still in `Var/`, though §9.4 calls it Hotaru's and not a command's. S2's list of what moves does not
   name it, and S2 should decide whether it goes to Crystal with the rest of `Var/` or to Hotaru. *Decided at S2: Crystal
   (§13.6).*
7. Two pages run past the 80 columns §5.5 states: `hier` to 83 and `hotaru` to 81.

### 13.6 S2 as built: Crystal's assembly, the session service's header, and its C# provider

*Built 2026-10-11, in three commits of code, each of which built the whole solution and passed the gate below on its
own: the move (`47e64e42`), the header and the provider (`a651c322`), and the parity class (`befe35df`).*

S2's row asks for three things and sets one gate. They are taken in the order they were built, because each depends
on the one before it: the provider names Crystal's types, and the parity class calls the provider.

**What moved, and under which names.** Forty-three files left `EmuSen.DianaSh` whole, by `git mv`, for a new assembly
at the repository's root, `EmuSen.Crystal`; two declarations were taken out of command files; and one helper was
lifted out of the shell's commands. Nothing in a moved file changed but its namespace and its `using` lines.

| Namespace | Folder | What | Files | Types |
|---|---|---|---|---|
| `EmuSen.Crystal.Debugger` | `Debugger/` | `IDebugTarget` and its seven value types; the ten registries; `DebugCpu` and `DebugCpus`; the expression evaluator, its context contract and its two contexts; `DebugTools`; `DelegateDebugMemorySpace`; `ICpuTraceSwitch`; the logging switch, `DebugLogging`; `IHistoricalCoprocessorTarget`, which was declared in `cophist`'s command file; and `DebugExpressions` (below) | 21 | 44 |
| `EmuSen.Crystal.Cheats` | `Cheats/` | the cheat engine: `CheatRegistry`, `CheatWrite`, `ChtFile`, `CheatImport`, the database with its installer and pruner; `ICheatCodeCodec` and `DelegateCheatCodec` | 9 | 22 |
| `EmuSen.Crystal.Telemetry` | `Telemetry/` | the six telemetry contracts, until now namespace `EmuSen.Cauldron` | 6 | 12 |
| `EmuSen.Crystal.Systems` | `Systems/` | the system packs (`SystemPack`, the debug views, the SNES's, the three Sega systems' and the PlayStation's), and `CoreDescriptor`, which was declared in `core`'s command file | 9 | 12 |
| `EmuSen.Crystal.Recording` | `Recording/` | `FrameRecorder` | 1 | 1 |

That is 46 files and 91 types, 88 of them public, 5,430 lines. The new code of the second commit is a sixth namespace,
`EmuSen.Crystal.Session`, 9 files, and the header, `EmuSen.Crystal/include/emusen_session.h`.

Three choices of naming are argued here. *Five namespaces, not one*: the assembly holds five subjects that its users
take separately (Serenity takes only the telemetry, the catalog only the systems), and a namespace per subject lets a
file's `using` lines say which it takes; it is the same reason the cores' namespaces are per console. *`Debugger`, not
`Debug`*: `EmuSen.Debug` and `EmuSen.Cores.Debug` already exist, and inside Crystal a namespace `EmuSen.Crystal.Debug`
would shadow `System.Diagnostics.Debug` for every sibling namespace, a fault that appears only when someone first writes
`Debug.Assert` there. *`EmuSen.Cauldron` renamed*: the namespace outlived its assembly at the fold of 2026-10-07, which
§17.1 item 1 did not reopen; the contracts now live with the component that owns them, and their namespace names it.
`LeafAssemblyTests`' rule that the telemetry contracts name nothing of the debugger is kept, restated as "nothing of
`EmuSen` outside `EmuSen.Crystal.Telemetry`", which is stricter than the old "nothing of `EmuSen.DianaSh`".

**What stayed.** The language (lexer, parser, AST), the interpreter, `CommandTable`, the scheduler, the sandbox, the
accounts, the suspensions, the line reader, `Var/Shell/`, `IDianaShCommand`, `DianaShResult`, `HostAction`,
`ManPages`, every one of the 83 command classes, and `RuntimeStats`, which is `vstop`'s and so a host command's (§16
Q14). `EmuSen.DianaSh` is now 104 C# files and 9,002 lines (counted 2026-10-11). Two of the moved pieces were the
reason the cores still needed the shell after the registries left: `CoreCatalog` builds its rows from
`CoreDescriptor`, and `SnesDebugTarget` implements `IHistoricalCoprocessorTarget`. Both were declarations in command
files; with them in the shell, the cores would have kept a reference to it for two types. `CoreDescriptor` goes where
§17.1 item 2 already sends it, beside the system packs. `IHistoricalCoprocessorTarget` moves unchanged, so `IDebugTarget`
gains no member and `EmuSen_Surfaces.md` §8 Q7's freeze holds. `DebugExpressions` is the one piece of new code in the
move: which expression context a processor evaluates in was decided by two functions of the shell's
`DebugCommandHelpers`, and the provider needs the same answer, so the decision moved to Crystal and the helpers now
call it, rather than the provider carrying a copy.

**`FrameRecorder`'s home, decided: Crystal.** §9.4 called it Hotaru's, which is half right. It is not a command's: no
command constructs it. But it is not one frontend's either: it has two users, Hotaru's F6 and Pharaoh's `record` verb
(`CommandsScriptRunner.cs`), and Pharaoh cannot reference Hotaru without taking Avalonia into the headless runner,
which `EmuSen_Surfaces.md` §4.2 counts as a defect to remove. Putting it in Hotaru would therefore have made a second
copy in Pharaoh, two owners of one fact. It names nothing of a frontend: of the target it reads only `CoreName` and
`FrameCount`, and it runs `ffmpeg` as a child process. `EmuSen_CoreAPI.md` §11.1 already places recording with the
runtime ("Runtime records; Frontend saves"), so Crystal is where that page's design puts it; §9.4's row is revised in
place. The argument does not generalise to everything two frontends share: what they share as presentation goes to
Serenity, and what they share as devices to Endymion. `FrameRecorder` is neither; it is a session's record of itself.

**The reference graph after the move**, as `LeafAssemblyTests` now pins it:

| Assembly | References of the project's own | Changed |
|---|---|---|
| `EmuSen.Crystal` | Galaxia | new |
| `EmuSen.DianaSh` | Crystal, Galaxia | Crystal added |
| `EmuSen` (the cores) | Crystal, Galaxia, Serenity | the shell removed, Crystal added |
| `EmuSen.Serenity` | Crystal, Galaxia, LunaP | the shell replaced by Crystal |
| Mistress, Hotaru, Pharaoh | the shell and Crystal, declared | until now reached through the cores |
| WiseMan | Crystal, declared | added |

The cores and Serenity reference the shell no longer, which is what §17.1 item 1 and §17.2's last point asked; the
shell's consumers are its hosts alone. Mistress, Hotaru and Pharaoh had reached the shell through `EmuSen` without
declaring it; the edge is now theirs, as `EmuSen_Surfaces.md` §7's S2 asks of every used edge. `EmuSen`'s reference
to Serenity remains: removing it is that page's S1, not this step. Three new facts pin the sets (Crystal references
Galaxia alone, the shell Crystal and Galaxia, the cores Crystal, Galaxia and Serenity), with the changed Serenity set;
a reference set fixes at build time what an assembly can name, so the first of them is the proof that Crystal names
nothing of the shell or of a frontend. The surface audit's own pins, its S0 file, are not built yet; §2.1 there
records the new sets with a note.

**The header.** `emusen_session.h` is version 1.0 of `emusen.session@1`: a table, `emusen_session_v1`, whose first
three fields are its size, its version and the provider's `ctx`, followed by 79 function pointers in the groups of
§9.2, in that table's order (648 bytes on a 64-bit system). It follows `emusen_core.h`'s conventions and its rule list
is that header's one layer up: signed statuses, with the words read by `last_error` on the same thread; UTF-8 by
pointer and length, a null pointer with a zero length being the absent string; the length-query idiom; JSON for
records and byte spans for bulk values; open enumerations; nothing unwinds. A caller checks the major and, through
`EMUSEN_SESSION_HAS`, the table's size before calling a function a later minor appended. Where it departs from §9.2 and
from Appendix B's sketch, and why:

- **Two functions added, and two statuses.** `label_clear`, because `label clear` exists and §9.2's list missed it.
  `EMUSEN_SVC_ABSENT` (−1107), an answer rather than a failure, as `EMUSEN_PLATFORM_ABSENT` is (no label at an
  address; no physical decoding reported). `EMUSEN_SVC_FAULT` (−1108), for a failure inside the session the provider
  adapts, with the exception's words, so that nothing unwinds into a C caller. The project's shared −1 (`NULL`) and −5
  (`BAD_STRING`) are kept beside the band, as `emusen_platform.h` keeps them.
- **A change answers by value.** Appendix B gave `bp_add` an `int64_t` id and the rest of the changes `int32_t`
  statuses; the built header extends that to every change: an id, a count or 0 is the return value, and nothing is
  copied out. The rule was forced by the parity class: its client, written as a C caller writes one, asked each record's
  length before copying it, and a cheat added through `cheat_op` was added three times, once for the query, once for
  a short buffer and once for the copy. A change has no length to ask. The one change in 1.0 that must answer with a
  record, `label_load` (its record holds the lines it refused), applies only on the call whose buffer holds all of it;
  the provider answers a length query by parsing on a scratch registry, which gives the same record because a label
  file's errors depend on its text alone. `run_control` returns `int64_t` for the same reason, since running to an
  address adds a breakpoint whose id the caller needs.
- **Addresses are 32-bit patterns.** `IDebugTarget` passes addresses as `int`, and a MIPS kernel address
  (`0x80000000` and above) is negative there. The header passes every address as `uint32_t` and writes it unsigned in
  JSON; the provider reinterprets the bits both ways and never range-checks them. A first draft range-checked them
  and would have refused every Mars address in kernel space; it was found reading Mars's targets before the parity
  class ran.
- **Which processor an index names.** `processors` lists `DebugCpus`, and an index into it picks a processor's
  registries. A session that names no processors has processor 0, the target's own registries, and, where it reports
  a coprocessor's breakpoints or coverage, processor 1; this is how `CoprocessorBreakpoints` and `CoprocessorCoverage`
  become "the processor's", as §9.3 asked.
- **Prefixes chosen against collision.** `emusen_core.h` already defines `EMUSEN_RUN_*`, `EMUSEN_CALL_*` and
  `EMUSEN_NOT_SUPPORTED`; the session header's names are `EMUSEN_SESSION_*` and, for statuses, `EMUSEN_SVC_*`, and a
  test compiles all three project headers into one program.
- **What `cheat_op` carries.** §9.2 listed "save, load, prune". Save and load name files, and prune deletes them in
  the database's folder; a module reaches files through the shell (§6.3), not through a session. The service carries
  the cheat file's JSON instead, `EXPORT` and `IMPORT` (exactly Galaxia's `CheatFile` with its own serializer options),
  and the database's three queries; pruning stays with the command until the cheat module is written (S5).
- **The two percentages** (`profile_op`'s hottest routines and the hardware load) are JSON numbers with a fraction;
  every other number is an integer. A command prints them with a format of its own, so the service carries the
  double that the member returned rather than a rounding of it.

The header's own half of the gate is `SessionHeaderTests`: the managed mirror, `EmuSenSessionV1`, holds the header's
fields in order with each function pointer's return and parameter types; `SessionAbi` holds every one of the header's
94 `#define`s by name and value; every status lies in the band; the provider fills every pointer of both its tables and
states their size and version; and a C compiler, given the three headers with `-std=c99 -Wall -Wextra -Werror
-pedantic`, lays the table out at the offsets .NET does. The last test returns early on a machine with no C compiler,
which on the development desktop was not the case.

**The provider.** `EmuSen.Crystal.Session.SessionService` takes an `IDebugTarget` (or none) and a
`SessionServiceOptions`, a record of what a host hands the shell today: the two cheat codecs, the trace switch and the
supported cheat systems of `CreateDefault`, the cheat database's folder, and `state`'s save, load and default path
(§13.3 said these become what the provider reads). It hands out two tables. Each table's `ctx` is a `GCHandle` to a
binding of the provider and a flag, and the second table's flag makes every change answer `EMUSEN_SVC_READ_ONLY`,
which is §16 Q7's enforcement by the table rather than by trust: a host gives a read-only context that table. The 79
entry points are `[UnmanagedCallersOnly]` statics, one per function, each of which converts its arguments, calls one
managed method, and catches every exception, a refusal becoming its status and anything else `EMUSEN_SVC_FAULT`; the
words go to a thread-static `last_error`. The managed methods are grouped as §9.2 groups them, and each reaches the
member §9.3 maps it to and nothing past it. Three things it cannot do the way Crystal will:

1. **No frame boundary.** It has no loop of its own, so a change is applied at once on the calling thread. Its host
   must call the table only where a command may touch the target today: on the emulation thread, or through the
   scheduler (§13.3). The header permits this in its rule for changes, and says so.
2. **`pause` is refused.** There is no loop to pause; pausing is the frontend's until Crystal owns the loop (§9.9).
3. **Run requests are events.** `run_control` arms what `step`, `runto` and `resume` arm and raises an event of kind
   `run` whose `op` is `step` or `resume`, the two host actions those commands return today; `halt_state` reports the
   last one as `requested`. The facade of §13.3 can synthesise `HostAction.Step` and `HostAction.Resume` from either.

No host hands the tables out yet. The first caller is the shell's host API at S3, and the first module at S5; until
then the provider is reached by WiseMan alone.

**The gate.** *Measured on 2026-10-11.* `SessionServiceParityTests` is a C caller written in C#: it calls each
function through the table's pointers, as a module would, asking each record's length before copying it, and
compares what comes back with the member it maps. A read is compared field by field with the member's answer; a
change is compared by the state the member reports after it, and the member's own changes are read back through the
service; where the member is absent the service must answer `NOT_SUPPORTED`, and where the member throws it must
answer `FAULT` with the exception's words. Each case fails unless all 79 functions were reached. It ran on the golden
corpus's three fake targets (`fake`, `bare`, `nobus`) and its eleven cores, built and run for two frames as the corpus
builds them (the builder is now `GoldenTargets`, shared by the two). All fourteen were equal. What each session
reported of itself, through `info`:

| Target | Processors | Capabilities beyond breakpoints, watches, frame logs and cheats | Codecs |
|---|---|---|---|
| `fake`, `nobus` | 2 | coprocessor breakpoints and coverage, coverage, call stack, labels, counters, freezes, expressions, register flow, DMA log and channels, vectors, break conditions | none |
| `bare` | 0 | none | none |
| `venus` | 2 | coverage, call stack, labels, counters, freezes, expressions, register flow, DMA log and channels, vectors, break conditions, coprocessor history, trace | Pro Action Replay/Game Wizard, Game Genie |
| `venus-sa1`, `venus-gsu` | 3 | as `venus`, and the coprocessor's breakpoints and coverage | as `venus` |
| `venusrt` | 2 | coverage, call stack, labels, freezes, expressions, DMA channels, vectors | Pro Action Replay, Game Genie |
| `moon`, `moonrt` | 1 | coverage, call stack, labels, vectors | address:value, Game Genie |
| `mercury`, `mercuryrt` | 1 | coverage, call stack, labels, vectors | GameShark, Game Genie |
| `mars`, `marsrt` | 2 | coverage, the RSP's coverage, call stack, labels | GameShark |
| `nephrite` | 2 | coprocessor breakpoints and coverage, coverage, call stack, labels, freezes, expressions | Action Replay, Game Genie |

The table is §13.4 finding 19's inventory seen from the service's side: VenusRT publishes no counters, register flow,
DMA log, break conditions or coprocessor history, and the parity class confirms that the service says so as
`NOT_SUPPORTED` exactly where the target's member is null. The PlayStation's system pack moved with the rest, but the
corpus has no PlayStation target yet, so Diamond is not in the run.

With the parity class, `SessionHeaderTests`, a case with no session (every session function answers
`EMUSEN_SVC_NO_SESSION` with today's words, "No ROM loaded - this command needs an active debug target.") and a case
for the read-only table, the gate is 21 tests. The golden corpus, `ManPageTests`, `ThemeVocabularyTests`,
`CoreRegistrationGolden.txt` and `LeafAssemblyTests` passed after each of the three commits; no file in `Golden/`
changed, and `CoreRegistrationGolden.txt` did not. The counts, after the last commit: the parity class 16 (its
fourteen targets and the two cases), `SessionHeaderTests` 5, the golden corpus 96, `ManPageTests` 103,
`ThemeVocabularyTests` 6, `CoreRegistrationEquivalenceTests` 3 and `LeafAssemblyTests` 17, 246 in all. Since the move
touches every assembly that names the debugger, the blast radius was run as well: all of WiseMan's `DianaSh/` cases,
the Crystal tests, the core-registration, leaf-assembly, cheat, debugger, system-pack, core-adapter and ABI tests, the
console-window, `coretop`, `vstop` and feed-window tests of Mistress and Hotaru, the shell program's tests, the
telemetry and hardware-load tests and Galaxia's: 2,155 passed and 9 were skipped, the 8 `SettingPathsOpenTests` and
one Big Picture picture tool, each of which carries a skip of its own. Every project of the solution builds,
frontends, Pharaoh and the shell program included.

**Found while building it.** None of these is fixed here: the first two change what a command prints, and the third
is the registries' own behaviour, which S2 moved and was not to change.

1. **`step over` and `step out` can report success and arm nothing.** `BreakpointRegistry.ArmStepToDepth` refuses
   when the registry was never told its call stack, and `StepCommand` ignores its answer and prints "Stepping
   over...". Every core wires the two together; the golden corpus's fake target does not, and `step.golden`'s first
   case records the false success. The service answers `NOT_SUPPORTED` there, so at S5 the step module will differ
   from that case; whether the command should refuse too is the tester's to decide, and the case is re-recorded if so.
2. **`copflow clear` after `copflow off` clears nothing.** `RegisterFlowRegistry.Clear` re-arms the window it was
   armed with and returns at once when disarmed, so the log of a disarmed window survives a clear while the command
   reports "Coprocessor register flow log cleared." No case of the corpus clears after `off`.
3. **Importing a coverage map is a merge.** `CoverageRegistry.Import` adds an entry point's counts to those already
   recorded, so a map imported twice doubles them. That is the method's stated contract ("maps from several runs
   accumulate"); it is recorded because `cov load` says nothing of it, and a module calling `IMPORT` should know.
4. **Pages that cite the old paths.** `EmuSen_Debugging_Tools_Reference_v5.md`, `EmuSen_CoreAPI.md` and eleven other
   pages name `DianaSh/Var/…`, `DianaSh/Sys/…` or `EmuSen.Cauldron`. Most cite a file as it was read at a stated
   commit, which a move does not make false; the overview's tree, `DESCRIPTION.md`'s table of assemblies and the
   index, which describe the tree as it is, are updated.

### 13.7 S3 as built: the Rust shell core, its host API, the bridge and the facade

*Built 2026-10-15. Decided the same day, before it: port DianaSh to Rust now, starting with S3; and first, fix the
sandbox escapes of §13.4 finding 10, which on inspection had been fixed on 2026-10-10 (below).*

S3's row asks for six things and sets one gate. They are taken in the order a line meets them: the shell core that
runs it, the host API a frontend reaches it through, the bridge by which a C# command still answers it, the facade
that chooses between the two shells, the standalone program, and the gate; what it costs, what was found and what
remains close the section.

**The sandbox's escapes, first.** The decision of 2026-10-15 put the fix of finding 10 before S3. The tree already
held it: the merge `b6aef096` of 2026-10-10 closed the escapes of `dump`, `load` and `cov` (`b143e4df`), `core`
(`c92914ef`), `state` (`ce1c2a75`) and `test -f`/`-d` (`dbf4000f`), each commit with the test that showed its escape
first and the golden cases it re-recorded (§13.4 finding 10). Nothing was rebuilt. What was done is what the rule
asks of a fix claimed: it was run again on this branch before S3 began, `SandboxEscapeTests` and the golden corpus
together, 110 tests, all passing; and every command that names a file was read once more for a path that reaches
the machine without `TryResolve` or `TryResolveRom`, and none was found. The case-insensitive comparison of Q15 is a
different defect, kept through S3 by decision and corrected at its end (below).

**The shell core, `Platform/dianash/`.** A crate, `emusen-dianash`, in the platform workspace beside Galaxia's, as
§4.1 and Q11 placed it: 13 modules, about 2,900 lines with their tests, no dependency of the emulator's but
Galaxia's crate, and `sha2` for `passwd`'s hash. Its parts, and how each was written:

| Module | What | Written as |
|---|---|---|
| `lexer`, `ast`, `parser` | the grammar of `EmuSen_Debugging_Tools_Reference_v5.md` §3.17b | the C# `Lexer` and `Parser`, method for method, over Unicode scalar values where the C# reads UTF-16 units |
| `interp` | statements, pipelines, expansion, redirection, `source`, `$(...)`, the budget | the C# `DianaShInterpreter`, member for member, so that its quirks (§13.4 findings 1 to 5) are reproduced by construction rather than by recollection |
| `builtins` | the builtins below, with `help` and `man` | each C# command class's `Execute`, word for word |
| `table` | which name runs which command, the aliases, the read-only flags, `help`'s order | the C# `CommandTable`, one per session |
| `shell` | a shell: its sessions, accounts, host registrations, requests and output | the C# `DianaShSessionManager` and the parts of `CreateDefault` that are the shell's |
| `host` | a call built for a host's command, its io table, and what it wrote gathered back | new: §6.2 and §6.3 |
| `sandbox`, `accounts`, `man` | the sandbox over Galaxia's tree, the accounts, the pages beside the program | `DianaShSandbox.TryResolve`, `DianaShUserRegistry`, `ManPages` |
| `dotnet` | the .NET rules the C# shell's words rest on: `Trim`, `int.TryParse`, `ToLowerInvariant`, padding by UTF-16 length, `File.ReadAllText`'s decoding and lines, and the words of `FileNotFoundException` and `UnauthorizedAccessException` | measured against the C# through the corpus |
| `abi` | the structs and numbers of the two headers | `include/dianash_module.h` and `include/dianash.h` |

Writing the interpreter member for member was a choice, argued here because it is the one place a cleaner Rust
design was available and refused. The C# keeps a line's host action in a field that every nested submit resets, so
that a script's action is lost when a later script line runs and an earlier line's survives an empty script; a
subshell copies the root's stopwatch by reference, so a `$(...)` never gets a fresh budget, while a sourced line of
the root restarts it; `su` inside `$(...)` changes the session's account, because the C# command reached the
session's interpreter through its closure, not the subshell's copy. A design written from the description of these
behaviours rather than from the code would have had to rediscover each from a failing case; a design written from
the code has them by construction, and the golden corpus then checks that the construction was faithful. Where the
port departs from the code, it says so: `char.IsLetter` and `IsLetterOrDigit` outside ASCII are approximated by
Unicode's Alphabetic and Numeric properties, which also count the letter numbers and a few combining marks; and a
character outside the Basic Multilingual Plane, two units to the C#, is one to the Rust, which matters only where
`$` is followed by one. Neither occurs in the corpus.

**Which builtins.** §4.2 lists 29. S3 builds 18 of them as builtins, beside the language's five (`help`, `man`,
`export`, `unset`, `source` with `.`): `echo`, `cd`, `pwd`, `tmux`, `ps`, `kill`, `whoami`, `who`, `su`, `useradd`,
`userdel`, `passwd`, `true`, `false`, `test`, `[`, `history` and `shutdown` (with `quit`, and `jobs` for `ps`). The
rule that chose them is §6.0's: a command that reads or changes the shell's own state is the shell's. Every one of
these does, or is one of the five conditions and loops are made of. Three of §4.2's are not built:

- `clear` stays a host's command. Both frontends replace it with their own, which clears their window, and a builtin
  that asked the host to clear would have needed the request §10.4 describes and a host that answered it; until the
  frontends move to the host API's requests, the frontend's own command is what clears.
- `exit`, `which`, `command` and `trust` are new names. Each would change what `help` lists, and `help` is a recorded
  case that both shells must print alike until S6. *Built 2026-10-16 in both shells at once, as Q21 decided (§13.9).*

A host's command takes the place of a builtin of its name, as a frontend's command has always replaced a default,
except for the language's five, which no C# command could replace either; WiseMan's `HostActionPlumbingTests`
register a `shutdown` of their own, and the rule is what keeps that test meaningful under both shells.

**Sessions and accounts in the shell.** A Rust shell holds its sessions, as §10.2 has it: each an interpreter with
its own variables, history and account, listed under a name, one of them current; `tmux new` makes one from the
creating session's commands, as the C# builder made one from its `CreateDefault`'s arguments. A session a host
makes is unlisted until the host attaches it under a name, which is how a C# host's `RegisterInitial` and
`CreateSession` keep their order (the interpreter is built before the name is given). The accounts are the shell's,
shared by its sessions, not the process's: two shells in one process no longer share accounts, which §4.1 asked
for and which no case of the corpus can see, since each case has a shell of its own.

**Requests instead of host actions.** §6.2 retires `HostAction`. Until the frontends move to events, the shell
reports what the C# shell reported as an action, as a request a host drains after the line: `exit` for `shutdown`
and `quit`; `session`, with the session's id and the name as the command gave it, for `tmux` and `kill`; `continue`
for a bare `continue` that reached the prompt, which the shell knows only as a fact of its language and the C# host
reads as `resume` (finding 4); and `host`, with a value of the host's own, for a host command that returned an
action. The line's last request is the one it gives, a `source` stops at the first line that raises one, and a
subshell's is dropped, each as the C# field behaved.

**The host API as built.** `dianash.h` is §10.2's sketch with four changes, each forced by a host that existed:

1. *Every call that reads or changes a session names it*, 0 being the current one, as `emusen_session.h` names its
   sessions. The C# hosts hold one interpreter per session and submit to whichever they hold; a shell that only knew
   its current session could not answer a host submitting to another, which Mistress's per-session schedulers do.
2. *Sessions are managed by the host as well as by `tmux`*: `dianash_session_new`, `_attach`, `_switch`, `_kill` and
   `_replace`, which are what `DianaShSessionManager`'s public members need (`RebuildAll` is `_replace`, since a
   rebuilt session keeps its name, place and currency).
3. *`dianash_last_status` writes `$?` through a pointer*, since a command may exit with a negative status that would
   read as a failure of the call; *`dianash_user`, `dianash_set_user` and `dianash_awaiting`* are the facade's
   `CurrentUser` and `IsAwaitingMoreInput`.
4. *Re-entry is refused, not deadlocked.* A shell is used by one thread at a time; another thread's call waits, and
   a call made on the thread running one of the shell's lines, from inside a host's command, answers `DIANASH_BUSY`.

The module ABI's header, `dianash_module.h`, is Appendix A's with these changes: `slot` is an exchange (a null
`*state` reads, a non-null one keeps, releasing what was kept); `resolve` answers `DIANASH_OUTSIDE` (−273) with the
refusal's words in place of the path, since a module must print the shell's words and not invent its own; the
optional `jobs` writes its lines through the io table rather than a buffer, and `kill` is given a result and
answers `DIANASH_ABSENT` for an id that is not its own; and five statuses were added in the band (`EXISTS`,
`NO_SUCH_SESSION`, `LAST_SESSION`, `BAD_MANIFEST`, `OUTSIDE`). A host's commands are added with a
`dianash_host_commands` (a manifest, a context and `run`, `jobs` and `kill`), and a host command's result is a
`dianash_host_result`, a module's with the request appended.

**The baseline.** Both headers' facts, every constant, every export's signature with its parameter types, every
struct's size and alignment and every field's offset, are generated by `platform/tests/dianash_abi.rs` and recorded
in `Platform/dianash/abi/v1/baseline.txt`, 116 facts tagged `@1.0`. The test holds the headers to the Rust (the
constants by value, each export's Rust signature against the header's, each layout against what a C compiler gives
from the headers with `-std=c11 -Wall -Wextra -Werror -pedantic`), fails on a fact missing from the baseline until
it is recorded (`DIANASH_ABI_EMIT=1`), fails on a baseline line that is no longer a fact, and, given the base
branch's baseline (`DIANASH_ABI_AGAINST`), on any line that branch had and this one removed or changed. The core
ABI's guard job runs it with the base's baseline on every pull request; the platform job compiles `dianash.h` as C99
and C++17. A second test, `dianash/tests/surface.rs`, is §11.4's: the crate names no identifier `emusen_session.h`
declares, and depends on no crate of the emulator's but Galaxia's. The layouts are recorded for a 64-bit target;
a 32-bit one is not a target the project builds.

**The bridge.** Every C# command the Rust shell does not answer itself is registered with it as a host command,
through `dianash_add_commands`, by `HostBridge` (`EmuSen.DianaSh/DianaSh/Bin/Native/`). The facade sends one
manifest: each command's name, usage and read-only flag as its class declares them; `help`'s order, which is the C#
`CreateDefault`'s, builtins included; today's three aliases the shell does not own (`c`, `s`, `hexdump`); the `bp`
and `watch` job prefixes; and the line `help` prints about hexadecimal addresses, which is EmuSen's and not the
shell's, as a note. A call reaches `HostBridge.Run`, an `[UnmanagedCallersOnly]` function, which finds the command
and calls its `Execute` with the target, the arguments and the standard input, null where the shell says none was
given, and catches what it throws as the C# dispatcher did, `Error: <message>` with status 1. Three things needed a
decision:

- *Each session has its own command objects*, as each C# session had its own `CreateDefault`: `search`'s
  candidates, `snapshot`'s baselines and the cheat command's state are per session. The bridge keeps a session's set
  in the io table's slot, made on the session's first call by a factory that builds what `CreateDefault` builds; a
  host's own commands are shared, as the C# builder shared them. The Rust side holds only weak handles; the C# objects
  live as long as the facade that made them.
- *Host actions travel as numbers.* A C# command's `HostAction` is kept by the bridge under a number that the Rust
  shell carries as a `host` request, and the facade turns it back into the object after the line.
- *`ps` and `kill` ask the bridge.* The breakpoints' and watches' rows are listed by `PsCommand.JobLines` and removed
  by `KillCommand.TryJob`, both lifted out of the two C# classes so that the classes and the bridge share one copy.

**The facade.** `DianaShInterpreter` keeps every public member, and its C# body moved, unchanged but for its name,
into a nested `Managed` class (§13.3). Under `EMUSEN_DIANASH_NATIVE=1` each member forwards to the host API through
`DianaShNative` and `NativeShell`; with the variable unset, or set otherwise, nothing of the Rust shell is loaded.
This is state 1 of `EmuSen_RustPlatform.md` §3.9. The Rust shell also needs the platform library's Galaxia, since it
reads the tree through it (Q11); a process that has Galaxia on its C# falls back to the C# shell with one line in
the error log. `DianaShSessionManager` forwards to the Rust shell's sessions once a facade has been made with it,
handing out one session object per session so that a host may keep using them as keys; `CommandHistory` reads the
Rust session's history, and refuses `Add` and `Clear`, which no host calls, since the Rust shell's history changes
only by the lines it runs. `GetWelcomeBanner` stays the facade's own in both states: it lists the host's cores, which
the shell's banner (`dianash_banner`) no longer does (§4.1).

**The standalone `dianash`.** The program of §10.4 and §8.2, `Platform/dianash/src/bin/dianash/`: the shell core,
Galaxia's tree and a line editor, 0.7 MB stripped of nothing. The editor is `ConsoleLineReader`'s keys written for
the three systems (termios on Linux and macOS, the console's virtual-terminal input on Windows): history on the
arrows, Left, Right, Home, End, Backspace, Delete and Escape, anchored past the prompt, with the same gap in
wrapping (Q12). Two keys the .NET runtime answered outside the reader are answered by the editor, since raw mode
takes them from the terminal: Ctrl-C discards the line, and stops a running line through the shell's cancellation;
Ctrl-D on an empty line ends input. `dianash -c LINE...` runs lines and exits with the last one's `$?`. Its page is
`man dianash`.

**The gate.** *Measured on 2026-10-15*, before S4, over WiseMan's DianaSh tests (the golden corpus among them, and
`SandboxEscapeTests`, the language's cases, the users', sessions', scripting and continuation tests), the shell and
`vstop` windows' tests, the leaf-assembly pins and Galaxia's, each run in a process of its own:

| `EMUSEN_DIANASH_NATIVE` | Passed | Failed | Skipped |
|---|---|---|---|
| unset (the C# shell) | 1,282 | 0 | 8 |
| `1` (the Rust shell, C# commands through the bridge) | 1,280 | 0 | 8 |

The 8 skipped are `SettingPathsOpenTests`, each with a skip of its own. The native run counts two fewer because
`NativeSwitchTests`, which proves that the variable chose the shell it names rather than a silent fallback, was
written after it; it passed in the next native run (§13.8). Every one of the corpus's 538 cases passed byte for
byte through the Rust shell, with its status, its continuation and its host action. No golden file changed.

The first native run failed three tests, and both causes were the host's side, not the language's: an empty line
reached the shell as the absent string, because C#'s `fixed` over an empty array gives a null pointer, so the
facade now passes a pointer with no length; and `tmux kill` of the current session dropped the session while the
host still held it, where the C# interpreter went on answering, so a killed session is now unlisted and kept. P2
predicted that the first run would differ only where the C# relies on .NET's own text rules and nowhere in the
language; it held, and more strongly than predicted, since with every text command still the C#'s there was no
.NET text rule left to differ. **P2 is retired.**

**What it costs, measured.** On the development desktop on 2026-10-16, after the merge of WiseMan, 2,000 runs after
200 of warm-up, medians in microseconds with the 95th percentile in parentheses. The first two columns are
`ShellRoundTripBench` through the facade, in the gate's two runs, the C# shell's and the Rust shell's (whose builtins
and text tools are Rust's, and whose other commands cross the bridge); the third is the shell core alone
(`dianash/tests/bench.rs`) with `dianash.text` and a host's 70 commands registered, as a C# host registers them:

| Line | C# shell (facade) | Rust shell (facade) | Rust shell core alone |
|---|---|---|---|
| `true` | 1.2 (1.8) | 2.5 (2.9) | 0.51 (0.55) |
| `echo hello` | 1.7 (2.7) | 2.3 (2.8) | 0.67 (0.71) |
| `echo a b c \| grep b \| wc -l` | 6.2 (6.9) | 9.8 (10.3) | 4.88 (5.09) |
| ten of `echo $i \| grep 1` in a loop | 33.0 (51.7) | 49.2 (52.0) | 26.7 (28.7) |
| `spaces`, a C# command (no target) | 3.3 (3.9) | 5.6 (6.3) | — |

The core's own cost of a line, lexing, parsing, dispatch and the output's copy, is half a microsecond, and every line
of the corpus's kind costs the core less than it costs the C#. Through the facade the Rust shell is the slower of
the two, and the measurements say where. A line costs about 2 µs more, the four calls into the library per line
(submit, the output, the requests and their JSON, `$?`); a C# command crossing the bridge costs about 2.3 µs more
than calling it in the C# (`spaces`). And a module's call inside the test process costs about 2 µs more than the same
call in the bench, which two hypotheses tested on the bench do not explain: the host's 70 commands in the table add
6% to the loop, not the 80% observed, and reading the root from Galaxia's tree adds 3 µs over the loop's ten calls.
What remains is in the C# process and was not profiled; it is left as an open item (below), since no line a player
types is near a cost a player could see, and §7.5's argument for a module over a program holds at either figure.
The machine was loaded by other work during part of the day, and figures taken under load ran up to 60% higher;
every figure in the table was taken within one quiet hour, and each column is internally consistent.
`dianash -c true`, the standalone program started, run and ended, takes 0.61 ms (0.84 ms) with the text
module linked in, against P4's 20 ms; **P4 is retired.**

**Q15, corrected at the end of S3.** The root was compared ignoring case on every system (§12.1). It was shown first,
in both shells, before anything was changed: in a checkout whose root is `<checkout>/home`, a directory `HOME` beside
it, another directory on Linux, was entered by `cd "<checkout>/HOME"`, read by `cat` and by `source` (which ran the
script found there), answered by `test -f`, and written by `echo x > "<checkout>/HOME/written.txt"`, by the C#
shell and the Rust one alike. Both now ignore case only on Windows and macOS, whose file systems ignore it by default
(`DianaShSandbox.PathCase`, `sandbox::IGNORES_CASE`), so on Linux such a path is no longer taken as it is: it is read
chroot-style instead, as any refused absolute path is, and lands inside the root, where nothing is. The same lines
then reach nothing outside. `SandboxEscapeTests.A_directory_differing_from_the_root_only_by_case_is_outside_it`
and the sandbox's Rust test hold it. A file system made to ignore case on Linux, or to tell cases apart on a Mac, is
not detected; the rule follows each system's default, and a probe of the actual file system was judged more
machinery than the one directory it would protect. No golden case changed.

**Found while building it.** None of these changes what a case prints.

1. *The help's line about hexadecimal addresses is EmuSen's*, not the shell's; it now arrives as a note in the host's
   manifest, and the standalone `dianash` does not print it.
2. *Mistress's console window reads the history on its UI thread* while the emulation thread may be running a line
   (`HistorySource = () => _shell.History.Entries`). The C# read a list another thread was changing; the Rust shell
   makes the read wait for the line, which is the safe answer and costs the window a line's time at most.
3. *A C# command can no longer call back into the shell running it*: the host API refuses re-entry with
   `DIANASH_BUSY`. No command of today's does; the eleven that reached the interpreter through closures (§1.2) are all
   builtins now.
4. *The facade's `CommandHistory` refuses `Add` and `Clear`* under the Rust shell, whose history changes only by the
   lines it runs. No host calls either.
5. *Open: a module's call costs about 2 µs more inside the C# process than in the bench* (§13.7's cost). Two
   hypotheses were measured and refuted; the next step is a sample of the WiseMan process running the loop, with
   `ipsample.py`, before anything is changed. *Closed 2026-10-16 (§13.10): the comparison was against a Debug C#, and the sample found ignore-case
   comparison and `getcwd`, both fixed.*

### 13.8 S4 as built: `dianash.text` in Rust

*Built 2026-10-15, after S3's gate; each command committed on its own once its golden file passed both ways.*

**Three crates, and why three.** A module may not link Galaxia (Q11), and the text tools need .NET's rules, which
until now lived in Galaxia's crate. So the rules came out of it, and the module's side of the ABI was given a crate
of its own:

| Crate | What | Depends on |
|---|---|---|
| `emusen-dotnet` (`Platform/dotnet/`) | .NET's rules as pure functions: casing (`ToLowerInvariant`, the ordinal comparisons ignoring case), `CompareOrdinal`, `char.IsWhiteSpace` and `IsLetter`, lengths in UTF-16 units and padding by them; `int`, `long` and `double.TryParse`; `File.ReadAllText`'s decoding, its lines, its writes and the words of its exceptions | nothing |
| `dianash-module` (`Platform/dianash-module/`) | what a module is written with: the module ABI's types (moved from the shell's crate), a safe view of a call, the `Module` trait, `run_module` with the fault boundary of §6.7, and `module_exports!`, which makes a module a library exporting `dianash_module_*` | nothing |
| `dianash-text` (`Platform/dianash-text/`) | the module, `dianash.text`: sixteen commands and the `hexdump` alias | the two above, and `regex` |

Galaxia's crate now re-exports the rules it gave up, so none of its users changed; the shell's crate names them
through a module of re-exports, and is the owner of none. The standalone program moved to a package of its own,
`Platform/dianash-program/`, because it links the module and the shell's core must not depend on a module; the move
also put its sources where they are tracked, since the repository ignores every folder named `bin/` and the
program's first home was one.

**A module linked in.** A host links a module and gives it to the shell (`Shell::add_module`); its commands join
every session's table, and a call to one is built exactly as a host command's is, the same `dianash_call` and io
table, and run through `run_module`, so that the module's code is the code a library build would export. The
platform library and the `dianash` program both link `dianash.text`. The facade leaves out of its manifest every
command the shell lists as a module's, as it leaves out the builtins, so the C# class of a command the Rust shell has
is never registered; with the variable unset the C# class runs, as before.

**Standard error (Q10).** A module's failure words go to standard error, and the shell prints a stage's standard
error where the line's output goes as the stage ends, taken by no pipe and no redirection. No case of the corpus
pipes or redirects a failing text command, so none changed; the difference Q10 intended is real and visible, for
instance `cat nosuch | wc -l`, where the C# piped the failure's words into `wc` and printed 1, and the Rust shell
prints the words and then 0. It is recorded here rather than in the corpus, since a recorded case must read alike in
both shells until S6.

**What reproducing the C# took.** Each command was written from its C# class, as the interpreter was:

- *Ordinal order is UTF-16's.* `sort`, `ls` and `find` order by `StringComparer.Ordinal`, which compares UTF-16
  units and so puts a character outside the Basic Multilingual Plane before U+E000 to U+FFFF; Rust's string order
  is UTF-8's and puts it after. `emusen-dotnet`'s `compare_ordinal` compares the units.
- *`sort -n` is `OrderBy` over `double.TryParse`*, stable, with NaN first and −0 equal to 0. `TryParse` reads the
  machine's culture, so the C#'s numeric order depended on the machine; the Rust reads en-US's rule on every machine,
  measured with a probe of .NET 10 on 2026-10-15 rather than taken from its documentation: `,` between digits and
  never before the first (`,5` is no number, `1,,0` is ten), `∞` and not `Infinity` for infinity, `NaN` in any case
  and after a sign. The parity class below found the first two differences of the first draft, which had read
  `Infinity` and `,5` as the invariant culture does.
- *`wc`'s word count splits on `char.IsWhiteSpace`*, and its line count keeps the C#'s count of a file's final newline
  (§13.4 finding 14).
- *`cp` refuses a file onto itself*, as `File.Copy` does, with .NET's words; Rust's `fs::copy` would have truncated
  the file first.
- *`find -name` is `FileSystemName.MatchesSimpleExpression`*: `*`, `?` and `\`, case ignored.
- *`ls -l`'s time is local*, from the C library's time zone on Linux and macOS and the system's on Windows.
- *`mv` and `rm` are suspended* (`man rm`), and print the C#'s refusal; nothing else of them was ported, since nothing
  else of them runs.
- *`xxd` ends its lines with `\n` on every system*, where the C#'s `AppendLine` wrote `\r\n` on Windows; the Rust is
  the one that agrees with itself on the three systems S4 must run on.

**Patterns (P3).** `grep`, `sed` and `awk` take .NET regular expressions. They are run by the `regex` crate, with a
translation for what the two read differently (a `{` that begins no quantifier is a literal to .NET and an error to
the crate), `sed`'s replacement written in the crate's syntax (`&`, `\N`, a literal `$`, and a group the pattern does
not have left as the text `$N`, as .NET leaves it), and a refusal worded as .NET's `RegexParseException` words the
errors the two share: an unterminated set, too few or too many parentheses, a quantifier following nothing, a
trailing backslash, a range in reverse order. What the crate does not run at all, look-around and back-references in
a pattern, is refused with words of its own. Case is ignored by Unicode's simple folding where .NET uses its own
table; the two differ only on a handful of characters, the Kelvin sign among them. P3 predicted that the crate could
not match .NET's engine byte for byte unchanged and that a translation layer or a recorded set of differences would
be needed; both were, the layer being `pattern.rs` and the differences this paragraph. **P3 is retired.** No case
of the corpus meets a difference.

**A parity class on generated text.** The corpus holds the inputs its cases were written with; §14 asks for
generated ones too. `TextModuleParityTests` writes 200 generated texts (lines of ASCII, accented and dotted letters,
a character outside the Basic Multilingual Plane, a private-use one, signed and grouped numbers, the words and
symbols of infinity and not-a-number, tabs, empty lines, `\r\n` endings and none) and runs 36 command lines over
each through both: the C# class's `Execute` with the file's text as its standard input, and the Rust shell, which
the test makes directly whichever shell the switch chose, with the line's input redirected from the file. Every
`sort`, `uniq`, `wc`, `head`, `tail`, `grep`, `sed`, `awk`, `xxd` and `cat` form the corpus uses is among them, and
patterns, replacements with `&`, `\1` and `$1`, and `awk`'s field, record and pattern forms. Its first run found two
differences in 7,200 comparisons, both `sort -n` over `Infinity`, the culture's rule above; since the fix all 7,200
agree, in both of the gate's runs.

**The gate, per command.** Every one of the sixteen golden files passed both ways, run once with all sixteen in Rust
and committed one command at a time, each commit naming its file. The runs, over the same filter as S3's: before the
merge of WiseMan, with the variable set to 1, 1,292 tests, 1,284 passed and 8 skipped, `NativeSwitchTests` and the
parity class among them; after the merge (`5411c52b`), both ways, 1,292 tests, 1,284 passed and 8 skipped, none
failed. One post-merge pair failed fifteen golden files on their `venusrt` cases, in both shells, because the build had
been run with a different cargo on the path, whose libraries reached the output while the sidecars described the
others, so discovery refused VenusRT and the C# Venus answered in its place; with the libraries rebuilt by the
project's cargo, every case passed. It is recorded because the symptom, a whole core's cases failing at once in
both shells, is easy to misread as the shell's. The Rust-side runner
(`dianash/tests/golden.rs`) also runs every case of the corpus that needs no host's command through the crates
alone, the text tools linked in: 56 cases, all passing.

**Its cost.** In the shell core alone, `echo a b c | grep b | wc -l` costs 4.9 µs and the ten-iteration loop 26.7 µs,
against 6.2 µs and 33.0 µs in the C# shell; a `grep` compiles its pattern on every call, as the C# does, and that is
most of a stage's cost. Through the facade, see §13.7's table and its open item.

### 13.9 The decisions of 2026-10-16, built

*Built 2026-10-16, each in a commit of its own, the defect of each fix shown by a test before the fix.*

| | What | Shown first | Golden |
|---|---|---|---|
| Q18 | `step over` and `step out` refuse, status 1 and no host action, when `ArmStepToDepth` declines because the breakpoints were never told the call stack: "This core does not report a call stack to its breakpoints, so `step over` cannot be armed." | `CallStackAndStepTests`' new case on the corpus's fake target read "Stepping over..." | `step.golden`'s first case, at its two such lines (`step over`, `step cpu over`) |
| Q19 | `RegisterFlowRegistry.Clear` empties the log of any window ever armed and leaves it as armed as it was | `RegisterFlowTests`' new case found the access still counted after a clear of a disarmed window | a new case in `copflow.golden`, 'a clear after off' |
| Q20 | `man cov` says that a map's entry counts add, the same map loaded twice counting each entry twice | — (the behaviour is kept) | `man.golden`'s case for the page |
| Q21 | `exit` as a third name for `shutdown`; `which`, `command` (with `-v`) and `trust` as the language's own, answered before any table in both shells; `trust` records a file's SHA-256 in the `trusted` file of the `dianash` folder beside the program's configuration, at the prompt only | — (new) | `help.golden` and `shutdown.golden`'s usage line re-recorded, a case for `exit`; `which.golden`, `command.golden` and `trust.golden` new; `man.golden`'s three new pages and the cases that print `help` |

Two choices in Q21 needed making. *Where the three names live*: as the language's, like `source`, rather than as
commands of a table, because `command` must dispatch past the aliases and `trust` must know whether it runs inside a
script, which only the interpreter knows; and because the language's names are the only ones no host's command takes
the place of, so `which` and `trust` cannot be shadowed. *What `which` calls a builtin*: the names §4.2 gives the shell,
which both shells can list alike. A host's command that takes the place of one of them is a command to the Rust shell
and still a builtin to the C#; that is the one place the two answer differently, it occurs in no case of the corpus,
and `man which` says so. `trust` approves; nothing loads a player's module yet (§5.4 and §6 are built for modules linked
into a program), so the approvals wait for the step that loads one.

The gate after the four commits, both ways, over S3's filter: 1,297 tests, 1,289 passed, 8 skipped, none failed. The
crate-alone corpus runner passes 59 cases and fails none.

### 13.10 S6's state 2, reached ahead of S5 by decision

*Decided 2026-10-16 by the tester:* make the Rust shell the default now, ahead of S5, and keep the C# shell as the
failover until the Rust one has been tested on Windows and macOS. *Built the same day.*

**The switch.** `DianaShNative`'s switch is now the platform's `PlatformSwitch`, as Galaxia's, Endymion's and
Serenity's are (`EmuSen_RustPlatform.md` §3.9): `EMUSEN_DIANASH_NATIVE=0` runs the C# shell, and any other value, or
none, runs the Rust shell when the library attaches, and the C# otherwise, with one line written to the host's sink
and the error log, `<library> is not in use (<why>); DianaSh runs on its C# implementation.` Attaching is a check that
runs over any export resolver before the process's pointers are taken: every export of the facade's list present,
`dianash_abi_version`'s major this facade's, and a shell made and freed. So the three ways the tester named, a library
missing, its DianaSh exports or version not this build's, and the Rust shell failing to attach, are each a refusal with
its reason, and each is proven by `DianaShSwitchTests`: a folder with no library; the real library's exports with
`dianash_abi_version` answering 2.0; the real exports less `dianash_session_replace`; and `dianash_new` answering
null. Each falls back with exactly one line, the switch asks its load once however often it is asked, and `0` never
asks at all. The resolver is the test's seam, and no test changes the process's own pointers.

**Each system, and what proves it.**

| System | Default | Failover | Proven |
|---|---|---|---|
| Linux | the Rust shell | the C#, by the variable or a refused library | yes: the gate below, both ways |
| Windows | the Rust shell where the published tree carries `emusen_platform.dll`; release 0.9.0 carries none, and there the C# runs, with the line | the C# | no: the Rust shell has not been run on Windows |
| macOS | the Rust shell where the bundle carries `libemusen_platform.dylib` | the C# | no: not run on a Mac |

The C# shell is kept, with every C# command class, until each of the two has been tested, per the decision; S7's
conditions (§13.2) are unchanged and add to it. The Windows half of the standalone `dianash`'s terminal code was
type-checked at S3 and has never run.

**The facade's cost, sampled.** §13.7 left a gap open: through the facade the Rust shell cost 2.5 µs a line against
the C#'s 1.2, and a module's call about 2 µs more in the C# process than in the bench. Two things changed the
picture once it was measured properly.

First, the C# figures were a Debug build's. WiseMan builds Debug, and a scratch program built Release, running the
same lines through the facade, gave the C# shell 0.5 µs for `true` and 11.0 µs for the ten-iteration loop, against
1.2 and 33.0 in WiseMan. Against Release C#, the Rust shell through the facade was 3.8 times as slow on `true`
(1.9 µs) and 4.5 times on the loop (50.2 µs). The Rust library is always a release build, so only a Release C# is a
fair comparison.

Second, `ipsample.py`, the project's sampler of one thread by `ptrace`, which no safepoint biases, extended for this
to name the platform library's own symbols, over the loop placed 42% of the thread's time in ignore-case comparison, `simple_upper` and the
Unicode upper-case tables it calls, and 13% in `getcwd`. Neither earlier hypothesis had been the cause. The
comparisons came from `Table::find`, a linear scan in which every name was compared character by character through
the full Unicode path; the `getcwd` from every module's call filling its `cwd`. Three fixes followed, each giving the
same answers as before:

1. ASCII's own paths in `emusen-dotnet`'s case rules (`simple_upper`, `lower_invariant`, `equals_ignore_case`,
   `to_lower`, `starts_with_ignore_case`), exact because .NET's ordinal case for ASCII is ASCII's, with a test that holds
   every ASCII character to the general path; and an index in the table by folded name, rebuilt when its entries change.
2. The working directory's display kept for the line that read it, forgotten by `cd` and at the next line.
3. Each `grep`, `sed` pattern compiled once per shell and kept in the module's slot, 64 at most, since compiling was
   most of the remaining allocation (`malloc`'s `morecore` 11%); and the facade skips parsing a request list that
   is `[]`.

Medians in µs, Release on both sides, the scratch program on the development desktop:

| Line | C# shell | Rust shell, first | Rust shell, after the fixes |
|---|---|---|---|
| `true` | 0.5 | 1.9 | 0.7 |
| `echo hello` | 0.6 | — | 1.0 |
| `echo a b c \| grep b \| wc -l` | 2.3 | — | 3.8 |
| ten of `echo $i \| grep 1` | 11.3 | 50.2 | 11.7 |
| `spaces`, a C# command | 2.2 | — | 3.9 |

What remains is the facade's four calls a line, a few tenths of a microsecond, and a C# command's crossing of the
bridge, about 1.7 µs (UTF-8 both ways, the slot, the handles); both are recorded and left, since a line a player types
is three orders of magnitude from anything a player could see, and the decision did not make the cost a blocker. §13.7's
open item is closed by this paragraph.

**The gate.** *Measured on 2026-10-16*, over §13.7's filter with the windows of Mistress, Hotaru and Serenity that
reach a console (`FeedWindowTests`, `CoretopWindowTests`, `AccessibilityTests`, `DianaShShellWindowTests`,
`VstopWindowTests`): with the variable unset, now the Rust shell, 1,331 tests, 1,323 passed, 8 skipped; with
`EMUSEN_DIANASH_NATIVE=0`, the same, before the cost's fixes and again after them. The 8 are `SettingPathsOpenTests`'
own skips. `DianaShSwitchTests`' five cases and `NativeSwitchTests` are among them; the crate-alone corpus runner
passes its 59 cases.

### 13.11 S5 as built: EmuSen's four command modules in Rust, and `nano` as a program

*Built 2026-10-16, in a commit for the ground the modules stand on, one for each of two defects shown first, one for
each module, and one for the shell's programs with `nano`.*

S5's row asks for four modules over the session service, each command replacing its bridge entry when its golden
file passes, `nano` as a program, and a gate of each command's golden file both ways, on the fake provider and on
each core, with seeded faults in the modules. What was built is taken in the order a call meets it: how a module is
handed the session, the client it calls it through, what the service lacked and gained, the modules themselves, the
two defects the port found, the programs, the gate, and what remains.

**The session, handed to each session.** §9.1 has the host provide `emusen.session@1` and the shell copy it into
calls unread. The C# facade now makes a provider over each shell session's target, with the codecs, the trace switch,
the supported cheat systems and the cheat database's folder that `CreateDefault` is given, and hands its table to that
session alone. The host API's `dianash_set_services` gives one set to every session of a shell, and the C# hosts hold
one target per session: Mistress's rebuild after a core change makes new sessions over the new target while the old
ones still answer. A second call was therefore added, `dianash_session_set_services`, the services one session's calls
are handed in place of the shell's of the same name; it is the host API's minor 1.1, recorded in the baseline as
`@1.1`, and `tmux new` copies the creating session's services into the session it makes. The provider keeps no
pointer of the shell's and the shell reads nothing of the table, as §9.8 requires; the facade keeps each provider
alive as long as its shell.

**Run control, read back as host actions.** §6.2 retires `HostAction` from a module's result: a module asks the
session's loop to run through `run_control`, and the host learns of it from the session's events. After each line the
facade drains the run requests of every provider its shell holds and, where the line raised no request of the shell's
own, gives the last as `HostAction.Step` or `HostAction.Resume`, so Hotaru's loop and the corpus's recorded actions
are unchanged. Two cases differ from the C# by construction and occur in no case of the corpus: a `$(step)` arms a
step that the C#'s subshell dropped the action of, and a line mixing `tmux switch` with `step` gives the shell's
request rather than the later of the two.

**The client.** `Platform/session-service`, crate `emusen-session-service`, is the service from a caller's side:
`emusen_session_v1` as Rust lays it out, held to `emusen_session.h` by a test that has a C compiler print every
field's offset and the table's size and compares them, and every number the header defines; a client that calls the
table by the header's rules (the length query, a change never asked twice, the words of a refusal read from
`last_error` on the same thread); and what every EmuSen module shares over it: `FindSpace`'s rule and words,
`ParseHex` and `ParseRange` over .NET's `Convert.ToInt32(text, 16)` with its words as measured on .NET 10, the scope
words of `DebugCpus`, a label's description, and the bare name of a core's log folder resolved through the call.
.NET's JSON and the C# `Suggestion` moved from Galaxia's crate to `emusen-dotnet`, which a module may link, Galaxia
re-exporting both, as the text rules moved at S4 (§13.8). The module crate gained what the four needed and the text
module now shares: a call's services, `ps`'s jobs and `kill` with the fault boundary of §6.7 and their exports, an
`Answer` (a C# command's returned text as standard output, its `Fail` as standard error with status 1, an exception
the dispatcher caught as `Error: <message>`), and a manifest writer.

**What the service gained: version 1.1.** Each addition was forced by a command whose C# reached something the
version 1.0 table could not, and each is an addition a 1.0 caller never meets (§11.2):

| Addition | Forced by |
|---|---|
| `"pc"` in `processors` | `cpus` prints each processor's program counter, and `disasm <cpu>` with no address starts there |
| `EMUSEN_SESSION_PROCESSOR_SESSION`, a processor argument naming none | a command that names no processor reaches the target's own registries, call stack and expression context, which on a session listing processors need not be processor 0's: `eval` on Nephrite or VenusRT reads the system pack's flags through the target's context and not through the first processor's |
| `run_control(RESUME)` and `events_drain` answered with no session running | `resume` with no game loaded still returned `HostAction.Resume` (`resume.golden`) |
| a channel as a 32-bit pattern | `mute -1 on` reached the core as typed (`mute.golden`) |
| `EMUSEN_SESSION_PROFILE_RESET_STACK` | `bt reset` |
| `"destination_name"` in the DMA log's transfers and destinations | `dma log` and `dma stats` name each destination as the core names it |
| `CHEAT_DECODE` with `"unchecked"`, and `CHEAT_CLAIMS` to `CHEAT_DB_UPDATE` | `cheat add` asks each codec whether it claims a code and then decodes with the codec's own words, writes first; the cheat lists' files, their names and their reading are Galaxia's `CheatFile`, and the database's folder, its matching, pruning and download are Crystal's cheat engine, none of which a module may link (§16 Q11) |

§13.6 had kept the cheat files with the command, a module reaching files only through the shell. That held for a file
the player names, and `cheat import` and `cheat export` still read and write theirs through the call's `resolve`; it
does not hold for the cheat lists, whose location, legal names and lenient reading are Galaxia's rules, which a second
copy in a module would make two owners of. `cheat save`, `load` and `files` therefore ask the provider by name. Two of
the new operations change the disk and answer with a record, `CHEAT_DB_PRUNE` with `"apply"` and `CHEAT_DB_UPDATE`;
their first call applies and the record waits until a buffer takes it, as `events_drain` keeps its events, so the
length query never applies them twice.

**The modules.** Each was written from its C# classes, as S4's commands were, and is linked into the platform library
beside `dianash.text`; the facade leaves every module command out of its host manifest, so a C# command class runs
only under `EMUSEN_DIANASH_NATIVE=0`, and stops claiming `bp` and `watch` for `ps` once the trace module claims them.

| Module | Commands | Kept in the call's per-shell slot |
|---|---|---|
| `emusen.debug` (`Platform/emusen-debug`) | the 26 of §8.1 | `search`'s candidates; the snapshots `snapshot` takes and `diff` compares, in the order .NET's `Dictionary` enumerates them, a removed entry's place taken by the next |
| `emusen.trace` (`Platform/emusen-trace`) | the 16 of §8.1, with `c` and `s` | nothing; `ps` lists its breakpoints and watches and `kill` removes them |
| `emusen.cheat` (`Platform/emusen-cheat`) | `cheat` | nothing |
| `emusen.session` (`Platform/emusen-session`) | `summary`, `coretop` | nothing |

`core` and `state`, which §8.1 also gives the session module, stay each host's own command: every host registers both
already, `core` with its core registry and `state` with the save and load of its own window, and a module's `core`
needs the frontend service's `games` and `open_game` (§9.9), which no host provides yet. `coretop` refuses as the C#
refused in a console window; its live dashboard is not ported, because a call's output reaches the player only when the
call returns (§4.3), and redrawing a terminal four times a second is the terminal host's panes mode (§18), not a
command's.

**Two defects, shown and fixed first.** *`trace 0` armed a trace that never ended* (`c6244ff8`): Venus's switch counts
down only from above 0, so a count of 0, or one above `0x7FFFFFFF` read as negative, left verbose tracing on for good
while the command printed "Trace armed for the next 0 instructions." The session service had refused such a count
from the start, so the module could not reproduce the C#'s answer without arming the same defect. `TraceCountTests`
showed it on the C# shell; both shells now refuse, status 1, in the service's words, and `trace.golden`'s case and
`man trace` were re-recorded in that commit. *A root reached through a link was outside itself* (`ab00917c`): macOS
spells its temporary folder under `/var`, a link to `/private/var`, and a working directory set inside a home there
reads back resolved, as `getcwd` answers, so both shells refused every file in it. This is the question §27.6 of
`EmuSen_RustPlatform.md` left to DianaSh, and it is decided so: a path is inside the sandbox when it lies under the
root as spelled or under the root with every link along it followed. Only the root is resolved. Resolving the path
too, as "canonicalise both sides" would, was rejected for two reasons: a path that does not exist yet, a file about to
be written, has no canonical form, and following links inside the home would change what finding 10 of §13.4 left
deliberately as text (`man hier`). The C# had compared text alone, as .NET does; it follows each component's link as
.NET reads it, the Rust asks the system's canonical path. `LinkedRootTests` showed the refusal on the C# shell and the
sandbox's Rust test the wrong chroot-style answer before the change, and both pass after it on Linux, where the test
makes its own link; on macOS CI the golden corpus itself runs in a home under `/var`. Since the C# Venus was retired
(`f1f47969`), no core registers a trace switch, `trace.golden`'s cases run on VenusRT and print that none is registered
whatever the count, and the refusal is held by `TraceCountTests` over a switch that records what it is told, the
golden file listing it as unreachable.

**`nano` as a program.** The Rust shell had no programs (§7). It now finds those shipped beside it, each described by
a manifest `<name>.dianash.json` of kind `program` in the `dianash` folder, read without running anything, and runs
one as a process when it is named: argv as typed, the previous stage's output as its input, the real directory behind
the shell's working directory, the environment with every exported variable and `DIANASH_ROOT`, `DIANASH_USER` and
`DIANASH_SHELL`, its exit status as `$?`, 128 and the signal where one ended it, and the terminal itself when the shell
runs at one and the program stands alone. A program the player cancels, or one past the line's budget, is ended; the
graded sequence of §7.3 (an interrupt, then a terminate, then a kill) is not built, and the end is immediate. `nano` is
the C# editor rewritten in the `dianash` package beside the standalone shell, sharing its terminal code; it reads the
sandbox's root from `DIANASH_ROOT` and refuses in the C#'s words. `Platform.targets` builds it with the library and
copies it and its manifest into `dianash/`; CI ships both in the platform library's artefact. The trust rules of §5.4
(a shipped program listed with its hash in the release's manifest) are not built; a program is found by its manifest
alone.

**The gate.** *Measured on 2026-10-16*, over §13.10's filter with the session service's tests (`SessionServiceParityTests`,
`SessionHeaderTests`): with every module in, 1,360 tests, 1,352 passed and 8 skipped, none failed, with the variable
unset (the Rust shell) and with `EMUSEN_DIANASH_NATIVE=0`, run with the debug and trace modules, again with `nano`, and
again with the cheat module; after the merge of `WiseMan` that retired the C# Mars and Mercury (`ed089b07`), whose
two cores left the corpus's targets, 1,358 tests, 1,350 passed and 8 skipped, both ways; after the merge that
retired the C# Moon and Venus (`fa3f4561`), 1,303 tests, 1,295 passed and 8 skipped, both ways, once `trace.golden`
listed the count's refusal as one no target of the corpus reaches. The 8 are
`SettingPathsOpenTests`' own skips. Every golden file of every moved command passes, on the fake targets and on every
core of the corpus, with the C# class of each command unregistered; the C# shell passes the same files with the
classes it still has. `NativeSwitchTests` now prints which shell a run used and, under `EMUSEN_DIANASH_EXPECT`, fails
a run that fell back; it also holds that the modules answer their commands. `cargo test --release` in `Platform/`
passes, the session client's header test among it; the crate-alone corpus runner passes its 59 cases. 

**Seeded faults.** Ten faults were seeded one at a time across the four modules and their shared client, each a
change a port could plausibly make (a bound off by one, a comparison loosened, a word or a space changed, a digit
count), and the corpus run on the Rust shell after each: eight failed it, each in the file of the command seeded. Two
passed: `mem` showing `~` (0x7E) as `.`, since no case dumps that byte, and `cov <addr>` printing ` ...` after a list it
showed whole, since no case of the corpus has any executed byte at all: the runner runs a core's frames before a
case's first line, so coverage armed by the case never sees an instruction. Both are gaps in the corpus, not in the
modules (below).

**What answers on no core since the C# cores were retired.** The merge of `WiseMan` at `fa3f4561` brought the
retirement of all four C# cores (Mars `9b5a56ee`, Mercury `250430d1`, Moon `d40b8b81`, Venus `f1f47969`), and with
them went the only targets that answered part of what the debug and trace modules carry. Every core is now reached
through `CoreDebugTarget`, which leaves `IDebugTarget`'s `AccessCounters`, `RegisterFlow`, `BreakConditions`, `DmaLog`
and `ResolvePhysical` at their defaults, sets no `RegisterWriter` on any processor, and no core's bundle carries an
`ICpuTraceSwitch`. The moved commands therefore refuse, answer that the core reports nothing, or do less than they say, on
every core today, through the Rust modules and the C# classes alike. A tenth row, `bp add … if` and `bp add … log`, stood
here until the same day: no breakpoint registry had a `ConditionEvaluator`, so a condition counted as true and a logpoint
recorded its expression's text. Those now answer on every core (§13.12).

| Command | What every core answers, as the corpus records it on the Rust targets |
|---|---|
| `bp when` | "This core reports no hardware conditions to break on.", status 1, on all five |
| `setreg` | a register the processor names is refused as one the core cannot write; the corpus's only case on a core is a name that does not exist |
| `counters` | "This core does not report access counters.", status 1 |
| `copflow`, `cophist` | no register-window traffic and "This core does not keep coprocessor register history." (VenusRT, the one core with a coprocessor case) |
| `dma log`, `dma stats` | "\<core\> records no DMA history.", status 1; `dma` itself lists VenusRT's channels |
| `addr` | no decode from any core; MercuryRT's and MoonRT's answers are the memory spaces' own mapping, and Nephrite's and the rest's "does not report an address decode" |
| `channels` | per-channel rows from MoonRT and MercuryRT; "(this core reports no audio channels)" from VenusRT, MarsRT and Nephrite, so the SNES's eight voices have no view |
| `disasm … m8 m16 x8 x16` | the width hints reach the service and no disassembler reads them |
| `trace` | "No CPU trace switch is registered for this target." |

None of this is the modules': each answer is the provider's, and each command answers as before on the fake target the
corpus grades it against. Which of them a core should answer is a decision for each core's own work, not S5's.

**On Windows and macOS.** The WiseMan job of `rust-cores.yml` gained two steps, the DianaSh tests (the golden
corpus, the shell's tests and the session service's, 1,077 to 1,079 of them) on the Rust shell, with
`EMUSEN_DIANASH_EXPECT=rust` so that a fallback to the C# fails the run, and on the C#; the shell loads the platform
library each system's own job built, with `nano` beside it. Three runs:

| Run | Linux, Rust / C# | macOS (osx-arm64), Rust / C# | Windows (win-x64), Rust / C# |
|---|---|---|---|
| 38932922787 (modules, before the fixes) | 1,078 / 1,078 of 1,079 | 1,063 / 1,063 | 1,050 / 1,030 |
| 38943963900 (attempt 2) | 1,077 / 1,077 of 1,077 | 1,076 / 1,076 | 1,075 / 1,070 |
| 38964112111 (after the merge retiring the C# Moon and Venus, `cc3f37d5`) | to be recorded | to be recorded | to be recorded |

Each run's log says `DianaSh in this run: the Rust shell` with the system's identifier on all three. The first run's
failures were read one by one (`beecdd43`): on macOS every one was a path through `/private/var`, the resolved spelling
of the home the shell now accepts, which the test's patterns did not name; on Windows a path's separator, a line's end,
a Windows path unquoted in a shell line, a drive's path refused where the test expected a Unix path's chroot-style
reading, and the publish tests' Linux host; on Linux the text parity class's culture. Two were the shell's: `memcard`
showing a path reached through the resolved root as `../../../private/…`, and the debug module's log files taking a
drive's path otherwise than `Path.GetFullPath`. The second run left `cd /LOGS`, which finds `Logs` on the two systems
that ignore case, and a test's unescaped JSON. The C# shell's further failures on Windows are its commands' own: their
`AppendLine` ends lines with `\r\n` there, and `TrimEnd('\n')` then leaves a `\r` at the end of `channels`, `label`,
`mute`, `sprites` and `tilemap`, which the Rust modules do not print.

**What S7 would delete, counted 2026-10-16.** S7 is not begun: the tester's condition is that Windows and macOS are
confirmed first (§13.10), and §13.2's conditions add a release played in state 2. When it comes, it would delete:

- *the C# interpreter*: `DianaShInterpreter`'s nested `Managed` class (659 lines), `Lib/Lexer.cs`, `Lib/Parser.cs`,
  `Lib/Ast.cs`, `Bin/CommandTable.cs`, `Etc/DianaShUserRegistry.cs`, `Etc/DianaShCommandSuspensions.cs` and the C#
  standalone `Bin/Program.cs`, 865 lines besides `Managed`, and the facade's switch with its fallback, so that
  `EMUSEN_DIANASH_NATIVE=0` is refused as state 3 has it (`EmuSen_RustPlatform.md` §3.9);
- *the C# command classes the Rust shell answers*: 34 of `Bin/Commands/Unix/`'s 36 files and 45 of
  `Bin/Commands/EmuSen/`'s 50 (`SnapshotStore` among them), about 5,800 lines;
- *the tests that construct those classes directly*, which become tests of the Rust commands through the shell, and
  the C# half of `TextModuleParityTests`.

It would not delete, because something still uses them: `CoreCommand` and `StateCommand` (Q25), `MemcardCommand`
(the `emusen.memcard` module, added after §8.1, is not ported), `CoretopCommand` (Hotaru registers it with its window
opener), `ClearCommand` and `VstopCommand` (a host's, §8.1 and Q14), with `DebugCommandHelpers`, which `coretop`
still calls, and `HostBridge`, `IDianaShCommand` and `DianaShResult` under them; `DianaShInterpreterScheduler`, which Mistress's and Hotaru's console windows still drive;
`ConsoleLineReader`, which Hotaru's terminal still reads lines with; `HostAction`, which the frontends still read; and
`DianaShSandbox`, which `core`, `state`, `memcard` and Hotaru's launch prompt still resolve paths through.

**What remains unproven.** The CI runs above prove the shell as a library inside WiseMan's test host on the three
systems. They do not prove:

1. *the standalone `dianash` program on Windows or a Mac*: its raw-mode line editor (termios, and the Windows
   console's virtual-terminal input), Ctrl-C's handler, Ctrl-D, the code page set to UTF-8, and `ls -l`'s local time
   from the system's time zone (the C library's on Unix, the system's on Windows, `dianash.text`'s `clock.rs`). The
   Windows half was type-checked at S3 and has never run;
2. *`nano` at a terminal on any system*: CI runs its three refusals, not its editor;
3. *a program run by the shell at a terminal*, owning it while it runs, on any system;
4. *the frontends' own windows on Windows and macOS with the Rust shell*: Mistress's console, Hotaru's terminal and
   F4 prompt, `emusen-shell`, each over a running game (WiseMan's window tests run headless, and on Linux);
5. *a published tree*: a win-x64 or osx-arm64 publish carrying `nano` beside the library, and a signed bundle
   loading it.

The proposal: a CI job per system that builds `dianash` and `nano` and drives them through a pseudo-terminal, a Rust
test in the `dianash` package over the `portable-pty` crate, which opens a ConPTY on Windows and a pty on Unix: type
lines with the editor's keys and compare the screen, run `ls -l` against a file of a known time under a fixed `TZ`, end
a running loop with Ctrl-C, edit and save a file with `nano` and compare it. Items 4 and 5 want a person's hands: a
publish of the branch played for an hour on the Mac mini and on a Windows machine, the console opened and a debugger
session run in each, written down here before S7.

**What it costs.** Nothing was measured beyond the gate's own time, which is unchanged within its noise (1 minute 9
seconds for 1,352 tests before the merge); a debugger command now crosses the module ABI and the session service's table where it
crossed the bridge, and both crossings are of the size §13.10 measured.

**Found while building it, and left.** Each is outside S5's row or a change to what a recorded case prints.

1. *`watch add` and `framelog add` print the space as typed but store it as the provider names it*, where the C#
   stored it as typed, so a later `watch list` shows `RAM` for a watch added on `ram`. No case of the corpus types a
   space in another case.
2. *A negative count reaches some operations as a refusal*: `disasm`, `tile`'s bpp and `bp log` take their counts as
   `uint32_t`, and the provider refuses one above `2^31 - 1` where the C# passed the negative number on. No case meets
   it.
3. *Two gaps in the corpus*, found by the seeded faults: no `mem` dump holds `~`, and no case records an executed
   byte, so `cov`'s listing of them, `cov new`'s and `cov funcs`' are unrecorded. The first is a case in `mem.golden`;
   the second needs the runner to arm coverage before a core's frames, a directive beside `frames`.
4. *The standalone `dianash` does not link EmuSen's modules.* With no session service their commands would answer "No
   ROM loaded", as §8.2 says; linking them in is a choice of what one file carries, left for the release that ships it.

### 13.12 Breakpoint conditions and logpoints on every core, and the session service's 1.2

*Built 2026-10-16, the day the tester agreed to restore them.*

**The defect, shown first.** Until the C# cores were retired, the only owner that set `BreakpointRegistry`'s
`ConditionEvaluator` and `LogEvaluator` was C# Venus's `SnesDebugTarget`. The registry reads a missing evaluator as a
condition that holds and a logpoint as its expression's text, so after `f1f47969` every core answered `bp add … if` by
halting at every hit and `bp add … log` by recording the words typed (§13.11's table, `VenusRT_Native.md` §68.5).
`BreakpointConditionTests` showed it before anything changed, through whichever shell ran and on MoonRT, MercuryRT and
MarsRT: `bp add 8008 if A==0x99` halted MoonRT at `$8008` where A is 7, `bp add 0153 if HL==0x1234` halted MercuryRT
where HL is `$C000`, a logpoint recorded `HL` rather than `HL=0xC000`, and a condition ending in `&&` was accepted. Six
of its eight cases failed on the Rust shell and the same six with `EMUSEN_DIANASH_NATIVE=0`.

**Where the one owner sits.** In Crystal, as `BreakpointExpressions`, beside `DebugExpressions`, which already chose the
expression context for `eval` and for the session service. Two calls put a condition or a logpoint into a registry: the
C# `bp` command, and the service's `bp_add` and `bp_add_data`, which the Rust module calls. Both now go through
`BreakpointExpressions.Add` or `AddData`, which (1) checks the condition and each comma-separated part of the log with
`ExpressionEvaluator.Check`, a parse that resolves every symbol, those in a branch short-circuiting would skip included,
and reads no memory; (2) on a failure throws, adding nothing; (3) points the registry's two evaluators at this target
and this registry; and (4) adds the breakpoint.

The evaluators are set where a condition enters a registry, not in a target's constructor as §68.5 of
`VenusRT_Native.md` proposed for `CoreDebugTarget`. Two facts decided it. Four classes are debug targets today, not
one: `CoreDebugTarget` for every engine of the core ABI (VenusRT, Nephrite, Diamond) and the shims' own targets for
MarsRT, MercuryRT and MoonRT; setting the evaluators in each constructor would make four owners of one rule. And a
registry outlives the target it was reached through: `CoreFactory.Bundle` makes a new target on each call over the
engine's one registry, so an evaluator captured by an older target would be the one a later breakpoint ran under. The
place every core passes through, whichever target and whichever shell, is the add. The cost is that a caller adding a
condition to a registry directly, bypassing both, gets the registry's old reading; the only such callers are the
registry's own unit tests, which set their evaluators themselves.

**The scope.** A condition reads what `eval` reads at the same scope. The registry `IDebugTarget.Breakpoints` names,
the main processor's, reads `DebugExpressions.For(target)`, as an unscoped `eval` does; another processor's registry
reads `DebugExpressions.For(target, cpu)`, as `eval <cpu>` does. That is the rule C# Venus kept, now held on every
core. On the Genesis it has a visible consequence, held by `NephriteDebugTests`: `bp add 302 if A==5` is refused,
since the 68000 has no register named A, while `bp z80 add 1 if "AF>>8==5"` halts the Z80 with A equal to 5.

**Read at the hit.** Every target's context reads its register providers' `Current`, which a host refreshes once a
frame (`EmuSen_Debugging_Tools_Reference_v5.md` §3.66.5). A hit falls inside a frame, so the context refreshes the
target's providers when an evaluation first names a symbol, once per evaluation, on the emulation thread, which is the
thread §3.66.5 requires. An expression of literals and memory reads alone refreshes nothing, the memory spaces reading
the live core. Refreshing every provider rather than the registers alone is forced by two shims: MoonRT and MercuryRT
read their registers out of a state snapshot that only `RefreshProviders` retakes, because neither core exports the
ABI's `DEBUG_REGISTERS`. The golden case "a true condition read at the hit" shows the reading is live: `S==0xF4` passes
over the hit at which S is `$F7` and halts at the next.

*Changed 2026-10-18* (`EmuSen_Backend.md` §26.7, decided in §20 there): with the shims retired, every core reads its
registers through `DEBUG_REGISTERS`, so the reason above no longer holds, and refreshing every provider had become the
cost: on a core whose pack decodes views, the view rings copied the spaces they read at every hit. A hit now refreshes
the register providers alone, through `DebugExpressions.RefreshRegisters`, beside the choice of context it serves:
`CpuRegisters`, `VideoRegisters`, `ApuRegisters` and each `DebugCpu`'s registers (a coprocessor's is one of them), the
rows the zero-per-core context reads among them. No context names a view: a pack's symbols are computed from the
registers just refreshed, and the generic target reads a unit's registers live (§24 there).

**The session service's 1.2.** `bp_add` and `bp_add_data` now evaluate the condition and log that 1.1 stored unread,
and refuse with `EMUSEN_SVC_PARSE`, the evaluator's words following, a condition or a log part that does not parse or
names an unknown symbol, adding nothing, as `eval` already refused such an expression. Nothing else changed: the
table's layout and every other call are 1.1's, so the minor is raised without an appended function, and a module
built against 1.0 or 1.1 needs nothing, the refusal arriving as any other. `EMUSEN_SESSION_MINOR`, `SessionAbi.Minor`
and the Rust client's `abi::MINOR` are 2. Both shells print a refusal alike, `Error: <words>` with status 1: the C#
class's exception through its dispatcher's catch, the Rust module's refusal through `Answer::fault`.

**`bp when` is not the same mechanism, and was left.** A condition is an expression tested where an address is
reached; a hardware condition is an event the core detects wherever it happens and reports by name
(`BreakpointRegistry.NoteCondition`), from a list it publishes (`IDebugTarget.BreakConditions`). No core publishes
one, and the core ABI carries no event for a named condition. One part is nearly there: the debug bridge already turns
the call kinds BRK (4) and COP (5) into `NoteInterrupt`, which fires the `brk` and `cop` conditions when they are armed,
so a target publishing those two names would answer them on a core that reports such interrupts. The rest of C# Venus's
list (`stp`, `wdm`, `ppuaccess`, `autojoy`) needs the core to report them, an extension of the debug surface (a debug
event kind, a minor version of the core ABI), proposed in §16.3 (Q29) and not built.

**The goldens.** No recorded case held the always-true behaviour, because the corpus never ran a core after a case's
first line, so no breakpoint was ever reached. The runner gained a line, `run <frames>`, placed before a `$` line: the
host runs up to that many frames on the case's core first, stopping at a halt and resuming one, as a frontend's loop
runs them. The crate-alone runner of the Rust tests passes over the word, its cases having no core. §13.11's third
finding asked for a directive of this kind to record coverage; this one serves it too, though that case is not
written. `bp.golden` gained four cases: on MoonRT, a false condition beside a logpoint at the same address (the
condition hit 0 times, the logpoint 6, recording `A=0x0  S=0xF7` and on), a true condition read at the hit (halted at
`$8000` on its second hit), and four malformed expressions refused (a condition, an unknown symbol, a log part, and a
condition on a scoped processor) with `bp list` empty after them; and on the fake target, a condition and a log part on
data breakpoints refused and a valid pair accepted. `man.golden`'s three recordings of `man bp` were recorded again for
the page's new text, and each difference is the page's. All were recorded from the C# shell and are printed byte for
byte by the Rust shell; no other file of the corpus changed.

**What it costs.** *Measured on 2026-10-16*, Release, by `BreakpointConditionCostBench` (opt-in, `EMUSEN_BP_BENCH=1`):
on each core's debug-test loop, one address hit every frame many times, the median of 300 frames after 30 of warm-up,
and per hit the frame's time over a breakpoint that is never reached, divided by the hits. "Before" is the same bench
on the tree with the two add paths reverted to the registry's own calls:

| Core (its target) | Hits a frame | A plain breakpoint, resume to next halt, before / after | A breakpoint never reached, a frame, before / after | `if 0`, a hit | A register condition, a hit | A register logpoint, a hit (before: its text) |
|---|---|---|---|---|---|---|
| MoonRT (shim) | 1,103 | 0.90 / 0.90 µs | 580.4 / 582.4 µs | 0.36 µs | 9.08 µs | 8.97 µs (0.26) |
| MercuryRT (shim) | 2,926 | 0.50 / 0.50 µs | 407.7 / 408.8 µs | 0.34 µs | 6.84 µs | 6.93 µs (0.96) |
| MarsRT (shim) | 394 | 0.70 / 0.50 µs | 19.4 / 16.1 µs | 0.32 µs | 1.12 µs | 2.06 µs (0.50) |
| Nephrite (`CoreDebugTarget`) | 1,800 | 0.90 / 0.90 µs | 754.2 / 752.1 µs | 0.67 µs | 1.88 µs | 1.83 µs (0.47) |

A breakpoint without a condition costs what it did: no line of the path it takes changed, and the two columns agree
within the bench's noise. `if 0` is the floor of a condition, leaving the core's frame and entering it again with the
expression parsed, a third to two thirds of a microsecond. Naming a register adds the refresh, about a microsecond
where the core exports its registers, and 6.8 to 9.1 microseconds on MercuryRT and MoonRT, where the refresh retakes
the whole state snapshot and rebuilds every provider. That is real frame time on the hottest loop a core runs: 2,926
hits a frame on MercuryRT come to 20 ms a frame, over the 16.7 ms a frame lasts, so a register condition on an
instruction that runs every few microseconds slows such a game below full speed. The lever is in the two cores, not
here (`DEBUG_REGISTERS` exported, and the shims' registers read from it), a per-core change proposed in §16.3 (Q28)
rather than built.

**Tests.** *Measured on 2026-10-16*: `BreakpointConditionTests` (MoonRT, MercuryRT and MarsRT, a false condition, a
true one, a logpoint and the refusals) and `NephriteDebugTests`' case on both processors, which failed before the
change as above and pass after it; `ExpressionEvaluatorTests`' cases of `Check`; `SessionServiceParityTests`, whose
breakpoint walk on every processor of every target now adds `frame >= 0` (a symbol every context has) and asserts the
two `PARSE` refusals; and `BreakAndCovCommandTests`, whose case of a bad logpoint now expects the refusal at the add
and the error of one that fails at the hit (`1 / 0`). Over the filter of the DianaSh classes, the session service's
tests and every core's debug tests (Diamond, Nephrite, MarsRT, MercuryRT, MoonRT, VenusRT and the generic view): 1,174
tests, all passed, on the Rust shell with `EMUSEN_DIANASH_EXPECT=rust` and again with `EMUSEN_DIANASH_NATIVE=0` (the
cost bench's four rows among them, which return at once without their variable). `cargo test --release` in
`Platform/`: 202 passed, 1 ignored, none failed.

### 13.13 `core <corename> --console` in the corpus (2026-10-17)

The Diamond console-menu work (`Diamond_Shell.md` S2) gave `core` a second usage line, `core <corename> --console`,
a longer usage refusal and a refusal for a console no engine starts empty, and changed `man core`. The corpus was not
re-recorded with it, so on WiseMan five tests of the gate failed: the cases of `core.golden`, `man.golden` and
`help.golden`, the usage gate (the recorded usage lines were not the live ones) and the failure gate (three fragments
of `CoreCommand.cs` no case printed: the new usage text, and the two halves of "no core in this build starts the …
with nothing in it").

**The runner.** `GoldenRunner` built `core` over the registry alone, so every console would have been refused. A host
builds it with `HotaruSession.StartsEmpty`, which asks `CoreCatalog.EmptyStartFor`, read from the engines' core info
(core ABI 1.11's `starts_empty`); the runner now does the same, so the corpus records what a host's shell prints. The
recording therefore depends on the build discovering Diamond's engine, as `DiamondShellTests` already does.

**The cases added to `core.golden`**, each covering usage line 2: the PlayStation started empty by three names
(`playstation`, `diamond`, `PlayStation`), each returning `StartConsole`; consoles no engine starts empty (`snes`,
`nephrite` and `genesis`, which both name the Genesis, and `n64`), each refused; `--console`'s refusals: an unknown
core, the flag in the core's place, the flag with a file after it or before it (both the usage refusal), the flag
alone, and `--CONSOLE`, which is not the flag and is read as a path; and the same two lines on a shell attached to
VenusRT, which print what they print at a launch prompt, since what a running session does with `StartConsole` is the
host's.

**What the re-recording moved.** `core.golden`: the new usage line, the three usage refusals' new text, and the four
new cases. `help.golden` and the listing in `man.golden`: the new line of `help`, so the listing grows by 91 bytes and
`help | wc -l` goes from 295 to 296. The page for `core` in `man.golden`: its NAME line, which now names the console
started empty, the second SYNOPSIS line, the section *The console with nothing in it* and the example
`core playstation --console`. No other file or case moved; `man hier`, re-recorded with the firmware folder, already
held. The gate passes on the Rust shell and with `EMUSEN_DIANASH_NATIVE=0`: `core` is a host command, run by the same
C# class under both.

---

### 13.14 The shell's C# surface for a frontend on the frontend API (decided 2026-10-17)

emusen-shell moved onto the frontend API in B2's part 3 (`EmuSen_Backend.md` §14.11). SR2 holds a frontend on that
API to naming no type of `EmuSen` or `EmuSen.Crystal`. Five members of this shell's C# surface named one in their
signatures, so a frontend calling them failed the check even when it passed only null:

- `IDianaShCommand.Execute` names `IDebugTarget`;
- `CreateDefault` names `IDebugTarget`, `ICheatCodeCodec` and `ICpuTraceSwitch`;
- `CoreCommand` took a dictionary of `CoreDescriptor`;
- `CoretopCommand` took `Action<IDebugTarget>`.

The tester decided on 2026-10-17 to extend the surface with members that name only this assembly's types or the
BCL's, and to leave SR2 without an exemption. Each addition is additive and stays within the existing surface (rule
D of `EmuSen_Surfaces.md` §6.1):

| Addition | What it is for |
|---|---|
| `HostCommand` (`Lib/HostCommand.cs`) | An abstract `IDianaShCommand` whose `Run(args, stdin)` takes no target, for a host's own commands: emusen-shell's `hotaru` and its window's `clear`. A class deriving from it names no runtime type; the interface is implemented in this assembly. |
| `CreateForService(table, extras, sessions)` | Part 2's factory, now with `tmux` sessions. Each session `tmux new` makes is over the same table. Over a service, a C# command's `resume` or `step` is asked of the table's `run_control`, as the Rust shell's modules already ask it, and the session's run events answer it (below). |
| `CreateWithoutSession(extras, cheatSystems)` | A shell with no game, which a launch prompt makes before the API has a session to hand it. |
| `CoreChoice` (`Lib/CoreChoice.cs`) and `CoreCommand(IReadOnlyDictionary<string, CoreChoice>, startsEmpty)` | The `core` vocabulary as plain data: a display name, a console and the extensions loaded. A frontend fills it from the API's `systems`, whose `core` field the runtime fills from its registry. `CoreCommand`'s refusals and its words are unchanged, held by the golden corpus. |
| `CoretopCommand(Action)` | `coretop -w` hands the frontend nothing; a frontend on the API opens its window over its own session. |

The old signatures that took `CoreDescriptor` or `Action<IDebugTarget>` had no caller left once emusen-shell moved,
and were removed in the same step. Only tests used them, and the tests now convert the catalog's registry with a
helper of their own. `CreateDefault` stays, because Mistress and the cores' tests call it.

**`HostAction` from events.** `EmuSen_DianaSh.md` Q5 decided that `HostAction` is retired by the session service and
its events, with the C# facade synthesising today's actions until Hotaru moves. Over a service table, the facade
reports `Resume` and `Step` because the session's run control raised them:

- On the Rust shell, a module's call raises them, and `DrainRunRequest` already turns them into actions.
- On the C# shell, the facade asks `run_control(RESUME)` for a command's own `Resume` or `Step`, then drains the
  provider's run events.

A run event the frontend raised itself, outside any line, is drained before the line, so it is never reported as
that line's (`ShellOverSessionTests`). `Shutdown`, `SwitchSession`, `LoadCore` and `StartConsole` stay the shell's
and the frontend's actions, as Q5 decided.

### 13.15 The session service's 1.3: `games`, `open_game` and `open_console` (2026-10-18)

*Built 2026-10-18 on branch `b2-frontend-api`, as `EmuSen_Backend.md` §3.11 and D10 placed them: in this service,
served by Crystal, rather than in the frontend service of §9.9.*

**What 1.3 appends**, after `events_drain`, so a 1.2 caller's table is unchanged and a 1.3 caller checks the table's
size before calling one (`EMUSEN_SESSION_HAS`):

| Function | Reads or changes | What it answers |
|---|---|---|
| `games(ctx, out, len)` | reads | the frontend API's `systems`, the same JSON: each system with its engines, `starts_empty` and `core`, the names `core <name>` knows an engine's core by (`EmuSen_Backend.md` §14.11) |
| `open_game(ctx, session, path, engine)` | changes | a new session over the game at `path`, its id; an absent engine is the settings' choice |
| `open_console(ctx, session, console, engine)` | changes | as `open_game`, a console started empty, named as `games` lists it with `starts_empty` |

**A session opened as the one named was.** The session argument does not name the session to replace; it names the
session whose open options the new one is given: the open flags (no battery, single rows), the sound buffer's limit,
and the presenter's device, so a game opened from a console window inside Mistress is lent Mistress's device as the
game it replaces was. The session named runs on. An open waits for no frame boundary, because it changes no running
session; it is a change all the same, and a read-only table refuses it with `READ_ONLY` (§6.5).

**Who takes the new session.** The runtime announces it with a session event, `{"kind":"session","session":<id>,"op":
"opened"}`, which a frontend's subscription to session 0 already receives (`EmuSen_Backend.md` §3.5). The client that
drives frames takes it with the binding's `SessionOf(id)` or closes it; until one does, it is open and runs no frame.
This keeps the decision of whether a second game replaces the first, or plays beside it, the frontend's, which is the
only component that knows whether it has one game window or several.

**Its twins.** SR4 asks every function of this service for a frontend-API twin of the same arguments. `open_game`'s and
`open_console`'s are new exports, `emusen_crystal_open_game` and `emusen_crystal_open_console`, additive in the API's
1.1. `games` has one already: `emusen_crystal_systems` takes the same arguments after the runtime and answers the same
JSON, so a second export answering it would be two owners of one fact. The twin check
(`Platform/crystal/tests/crystal_abi.rs`) therefore holds a list of twins named otherwise, of one entry, and checks
`games` against `systems` by return and parameters as it checks every other pair. 142 exports now, 80 of them twins.

**Where a host opens nothing.** A provider whose host hands it no opener answers `games` and both opens with
`NOT_SUPPORTED` and the words "this host opens no games through the session service". The C# provider's options carry
the runtime's two delegates (`SessionServiceOptions.Games` and `Open`); the managed runtime fills them for each session
it opens.

**Statuses through the table.** A failure of the runtime's open keeps its own status and words: `IO` for a file that is
not there, `NO_ENGINE` for a console no engine starts empty, `FIRMWARE` for missing firmware, as `open` answers them;
`BAD_ARGUMENT` for an absent path or console, and `NO_SESSION` for a session the table does not serve.

**Tests.** `SessionService13Tests`: `games` is `systems`; a game opened from a session's table is announced, opened
with the first session's options, listed beside it, and both run a frame; the twin opens as the table does and the
runtime's refusals come through; a read-only table refuses and a table names only its own session.
`SessionServiceParityTests` reaches all three functions on every golden target and checks the provider with no opener;
the header and table checks, C# and Rust, read 82 functions.

**Not built, proposed.** No host hands a table to a console with no game running: `session_service` needs a session,
so `core <name> <path>` typed in Mistress's console before any game still goes through the host's own `core` command
and `HostAction.LoadCore`. Answering `session_service(rt, 0, flags)` with a game-free table, of the cheat ops' kind
(§13.11), would let a module's `core` open the first game too; it widens what `session_service` accepts, so it is put
to the tester rather than built. Moving `core` into the session module, Q25's second half, is S7's.

### 13.16 `emusen.dev` and `dupes`: a duplication gate (decided 2026-10-27)

Every other rule the project keeps has a mechanical check before a merge; duplication had none. A read-only audit on
2026-10-27 ran PMD's CPD over the Rust and C# files of the last twenty merges and found copies of existing code in
the added lines.

**Form.** A fifth command module, `emusen.dev` (`Platform/emusen-dev-module/`), for the project's own development
tools, with `dupes` as its first command (`man dupes`). It is linked into the standalone `dianash` program and not into
the platform library, since PMD is a developer's tool and is never shipped. It reads the repository directly through
`git` and the program named by `EMUSEN_CPD`, not through the shell's root, and takes the repository from `--repo` or
`EMUSEN_REPO` because the standalone shell moves its own directory.

**Rule.** A finding is a duplicated block of at least `--warn-tokens` (75) tokens with a place the added lines since
`--base` cover at least half of (the audit's rule); one of at least `--min-tokens` (100) fails the run (exit 1), a smaller
one prints as a warning, and a duplicate a change only touches prints as a warning whatever its size. What is
not compared is `dupes.ignore` (generated tables, tests, examples, benches, the WiseMan harness, build output, the
reference trees) and a duplicate the project keeps is recorded in `dupes.accept` with its reason, both at the
repository root and both plain text. An accepted pair is skipped only while both places stand at the recorded lines.

**Measured 2026-10-27.** CPD skips the files its lexer cannot read (20 Rust sources); the count is printed with
every run. The whole-tree audit with the committed ignore list
reports 132 findings at 100 tokens or more and 145 warnings, in about 45 s; `--base` on the last merge's parent
and on `WiseMan~1` reported none.

**Tests.** The crate's unit tests: the overlap with added lines, the ranking, the ignore and accept files, the options
and the exit statuses on synthetic CSV and diff text, and the missing-PMD message. Seeded faults, each caught by one
test: an overlap off by one line, the accept file ignored, a warning returned as a failure.

## 14. How it is tested

- **The golden corpus** (§13.1), the oracle of every step.
- **Parity**: each Rust module against its C# commands on the corpus's inputs and on generated ones, as each platform
  component's parity class did, with seeded faults to show the class speaks.
- **A module conformance kit**, `dianash-module-conform`, the counterpart of `emusen-core-conform`
  (`EmuSen_CoreAPI.md` §12): it loads a module through the ABI alone and checks its manifest against its exports, every
  command's run with no services, with each required service absent, with a short buffer and a struct of the oldest
  size, a fault caught at the boundary, cancellation honoured within its bound, and the per-shell slot released.
- **Skew fixtures**: a module and a host built at minor 0, and at a far minor with larger structs and canaries, as
  `EmuSen_CoreAPI.md` §12.5 has for cores.
- **The surface checks** of §11.4, and the baselines.
- **The three systems**: the text module's corpus on Linux, Windows and macOS CI runners; the rest on Linux, where the
  cores' images are built.

---

## 15. Predictions to be retired

- **P1.** A module's call costs under a microsecond besides the command's own work, and the first load of a module under
  0.2 ms on the desktop (0.04 µs and 51–63 µs measured for a minimal module, §7.5). Retired at S4 by timing the corpus.
- **P2.** The golden corpus's first run against the Rust shell with C# commands (S3) differs only where the C# relies on
  .NET's own text rules (regular expressions in `grep`, `sed` and `awk`, number formatting, case mapping), and nowhere in
  the language. Retired at S3. *Held, 2026-10-15 (§13.7): the first run differed nowhere in the language; its two
  differences were the host's (an empty line passed as absent, a killed session dropped).*
- **P3.** The Rust `grep`, `sed` and `awk` cannot use a regular-expression crate unchanged and match .NET's engine byte
  for byte on every case of the corpus; a translation layer or a recorded set of differences will be needed. Retired at
  S4, with the decision recorded. *Held, 2026-10-15 (§13.8): both were needed, `pattern.rs` and its list of
  differences, and with them every case of the corpus matched.*
- **P4.** The standalone `dianash` starts in under 20 ms on the desktop with no module loaded. Retired at S3. *Held,
  2026-10-15 (§13.7): 0.60 ms with no module, 0.61 ms with `dianash.text` linked in, started, run and ended.*
- **P5.** Starting a small program costs 1–2 ms on the Mac mini and 5–15 ms on a Windows machine (§7.5). Retired when
  the text module's corpus first runs there.

---

## 16. Questions for the tester

**Decided 2026-10-10:** all seventeen, each as recommended, with nothing changed. Q1's name for the runtime is Crystal,
decided the same day; its assembly and folder take the name at step S2 (§13.2). Q15 is decided as recommended: the
sandbox's case-insensitive comparison is kept through S3 so that the golden corpus holds, and is then corrected in the
C# and the Rust together, with a test that shows the escape first.

| | Question | Recommendation, and the decision |
|---|---|---|
| **Q1** | The runtime of `EmuSen_CoreAPI.md` §8 is no longer the shell (§0.3). Does it become a component of its own, with a name and a folder outside the shell's, and do the registries, the cheat engine, `IDebugTarget`, the telemetry contracts and the system packs move into it now (S2), before the runtime itself is built? | Yes. A new assembly beneath the shell and beside the cores, for now holding exactly what S2 moves, named for the runtime it will become; the name is the tester's to give. `EmuSen_CoreAPI.md` §0.2 item 2, §8.2's "one assembly" and Q13, and `EmuSen_Surfaces.md` §7's S5 and its Q2, are then revised there, with a note pointing here (§17.2) *Decided 2026-10-10 as recommended; the component is named Crystal, also decided 2026-10-10.* |
| **Q2** | Five shipped modules by subject (§8.1), or one module per command? | Five. The ABI does not care, a later split changes no script, and 83 libraries would mean 83 loads and 83 signatures in a macOS bundle for no gain *Decided 2026-10-10 as recommended.* |
| **Q3** | The text tools as an in-process module, not as programs, on the measured cost of a process (§7.5)? And no session service across a process boundary in version 1 (§7.4)? | Yes to both. The program kind stays for `nano`, a player's own programs and any command that may run long and needs no session *Decided 2026-10-10 as recommended.* |
| **Q4** | A fault in a module: caught at the boundary and reported (§6.7), unlike a core, which aborts? | Caught. It keeps today's rule that a malformed command never takes the console down; a module faulting three times is not called again *Decided 2026-10-10 as recommended.* |
| **Q5** | `HostAction` retired: shell control by builtins, the emulator's by the session service and its events, opening a game by the frontend service (§6.2)? | Yes; the C# facade synthesises today's actions until Hotaru moves to events (§13.3) *Decided 2026-10-10 as recommended.* |
| **Q6** | Every command runs on the shell's thread, and a change to a session waits for the frame boundary in Crystal, so the shell's scheduler retires (§6.4)? | Yes. The waiting is Crystal's business, and its boundary is where `EmuSen_CoreAPI.md` §8.3 already puts every command *Decided 2026-10-10 as recommended.* |
| **Q7** | The read-only flag enforced by the service table rather than trusted (§6.5)? | Yes *Decided 2026-10-10 as recommended.* |
| **Q8** | Manual pages as one file per command beside its module, in today's format (§5.5)? | Yes, moved in S1 while the C# still reads them *Decided 2026-10-10 as recommended.* |
| **Q9** | The service API handed to modules as a function table they call during their call: the one stated exception to "the library never calls the host" (§2, §6.0)? | Yes. A command is a program, and its service calls are its system calls; tables are versioned by size and need no symbol to resolve, and a C# provider can fill one *Decided 2026-10-10 as recommended.* |
| **Q10** | Standard error as a stream of its own, printed in order so the player sees today's bytes, and taken by no `>` (§4.3)? | Yes. Today `cmd > f` writes an error into `f`; with the change it prints it, as every Unix shell does. It is the one intended difference from the golden corpus, recorded as such in its file *Decided 2026-10-10 as recommended.* |
| **Q11** | The shell core as a crate in the platform workspace, exported through the platform library for the C# frontends and linked statically into the standalone `dianash`; modules never link Galaxia (§4.1, §6.3)? | Yes. One copy of Galaxia's tree and overrides per process is what the platform library's lending already taught (`EmuSen_RustPlatform.md` §16.2) *Decided 2026-10-10 as recommended.* |
| **Q12** | The line editor written to today's `ConsoleLineReader`'s behaviour for three systems, rather than a line-editing crate (§12.3)? And `exit` as a third name for `shutdown` (§4.2)? | Yes to both. A crate can follow once the golden corpus no longer depends on the editor's exact keys *Decided 2026-10-10 as recommended.* |
| **Q13** | Modules a player adds: allowed, after a one-time approval by hash, as foreign cores are (§5.4)? | Yes, with approval only through `trust` at the prompt *Decided 2026-10-10 as recommended.* |
| **Q14** | What `vstop` reports in a Rust shell, which has no .NET runtime: the hosting process's own figures (the .NET runtime's in a C# frontend, the shell process's in the terminal), or only in the C# frontends, as a host command? | A host command in the C# frontends while they exist, and in the terminal the shell process's own memory and threads, under the same name *Decided 2026-10-10 as recommended.* |
| **Q15** | The sandbox's case-insensitive comparison on Linux, which can accept a path outside the root as inside it (§12.1): kept as today, or made case-sensitive where the file system is? | Kept through S3 so the golden corpus holds, then corrected in the C# and the Rust together, with a test that shows the defect first *Decided 2026-10-10 as recommended. Corrected 2026-10-15 at the end of S3 (§13.7).* |
| **Q16** | The golden corpus committed as files in WiseMan (§13.1)? | Yes. It holds no ROM data, and it is the record that lets C# be deleted at S7 *Decided 2026-10-10 as recommended.* |
| **Q17** | The host API's versioning: stable from S3, though its only hosts until then are the project's own frontends built from the same commit? | Stable from S3. The baseline costs one CI step, and an outside frontend or the kit can then embed the shell the day it wants to *Decided 2026-10-10 as recommended.* |

### 16.1 Questions from S2, S3 and S4 (2026-10-15)

*Decided 2026-10-16 by the tester:* all seven as recommended. Q18, Q19, Q21 and Q20's page are built after S4, each
in a commit of its own (§13.9); Q22, Q23 and Q24 need nothing until the steps they name.

| | Question | Recommendation |
|---|---|---|
| **Q18** | `step over` and `step out` report "Stepping over..." and arm nothing when the registry was never told its call stack (§13.6 finding 1). Should the command refuse instead? | Yes. Refuse with the words `cov` and the others use for a core that does not report something ("this core does not report a call stack"), status 1, in the C# now with a test that shows the false success first, and re-record `step.golden`'s first case in that commit. The session service already answers `NOT_SUPPORTED` there, so the S5 module would otherwise have to reproduce a false success on purpose *Decided 2026-10-16 by the tester: as recommended.* |
| **Q19** | `copflow clear` after `copflow off` clears nothing while saying it did (§13.6 finding 2). Should it clear a disarmed window's log? | Yes: `Clear` empties the log whether or not the window is armed, and re-arms only one that was armed. A case for it in `copflow.golden`, recorded in the fix's commit, with a test that shows the stale log first *Decided 2026-10-16 by the tester: as recommended.* |
| **Q20** | Importing a coverage map adds its counts to those recorded (§13.6 finding 3). Keep it, or replace? | Keep it: accumulating maps from several runs is the method's stated purpose. Say so in `man cov` ("load adds the file's counts to those already recorded") and in the session service's comment for `IMPORT`; no behaviour changes and no case moves *Decided 2026-10-16 by the tester: as recommended.* |
| **Q21** | `exit`, `which`, `command` and `trust`, decided in §4.2 and Q12 and Q13, are not built, because each changes `help`, which both shells must print alike until S6 (§13.7). Add them to both shells now, re-recording `help` once, or wait for S6? | Add them to both now, in one change: four small C# classes and four Rust builtins, `help.golden` re-recorded and a golden file for each new name. Waiting leaves the standalone `dianash` without `exit` for its whole life before S6, which a terminal shell should not lack *Decided 2026-10-16 by the tester: as recommended.* |
| **Q22** | `ps`'s answer with nothing to list ("No active breakpoints, watches, or sessions.") and `kill`'s refusal without a session manager ("isn't a bp<N>/watch<N> id") name the debugger's jobs, though they are now the shell's words (§13.7). Reword them? | Not before S7: the corpus holds them, and both shells must print them alike until the C# goes. Then: "No active jobs or sessions." and a refusal naming the claimed prefixes, read from the manifests *Decided 2026-10-16 by the tester: as recommended.* |
| **Q23** | The platform library aborts on a panic, so `dianash.text`, linked into it, gets the crash log and not §6.7's `Error:` and status −320 if it ever panics (§13.8). Keep the library's abort, or build it to unwind? | Keep the abort: one crash log per process is the library's rule (`EmuSen_RustPlatform.md` §3.5), and the text module has no path that panics on input. A module shipped as a library of its own is built to unwind and gets the boundary, as §6.7 says *Decided 2026-10-16 by the tester: as recommended.* |
| **Q24** | Q10's stderr changes what a pipeline prints when a text command in it fails (`cat nosuch \| wc -l`: the C# printed 1, the Rust shell prints the words and 0), and no recorded case shows it, since a recorded case must read alike in both shells (§13.8). Record a case for it now in a file of its own that only the Rust shell runs, or at S6? | At S6, when the Rust shell becomes the default and the corpus may hold its answers: a case in `shell.golden`, re-recorded that day with the reason *Decided 2026-10-16 by the tester: as recommended.* |

### 16.2 Questions from S5 (2026-10-16)

| | Question | Recommendation |
|---|---|---|
| **Q25** | `core` and `state` stay each host's command (§13.11): a module's `core` needs the frontend service, and `state` the save and load each host hands its own command today. Build `emusen.frontend@1`'s `games` and `open_game`, and have the hosts give the provider their save and load, so that both move to the session module before S7? | Yes, before S7: until then the C# `CoreCommand` and `StateCommand` are the only `core` and `state` there are, and S7 cannot delete them. The frontend service is small (§9.9) and its first two calls are these |
| **Q26** | The session service's 1.1 (§13.11) gives the cheat lists and the cheat database to the provider, which §13.6 had kept with the command, and lets two changes answer with a record that waits for a buffer. Keep that shape? | Keep it: the files' rules are Galaxia's and the database Crystal's, and a second copy in a module would make two owners of each. The waiting record is `events_drain`'s rule, already in the header |
| **Q27** | The DianaSh steps of CI's WiseMan job report and do not gate (§13.11), as every platform step does (`EmuSen_RustPlatform.md` §10.6). Make the Rust shell's step a gate on all three systems once it has passed there? | Yes, once two consecutive runs pass on all three: it is the evidence S7 rests on, and a step that cannot fail cannot hold it |

### 16.3 Questions from §13.12 (2026-10-16)

| | Question | Recommendation |
|---|---|---|
| **Q28** | A condition or logpoint naming a register costs 6.8 to 9.1 µs a hit on MercuryRT and MoonRT against about 1 µs elsewhere, because their shims read registers out of a whole-state snapshot that only `RefreshProviders` retakes, neither core exporting `DEBUG_REGISTERS` (§13.12). Export it from both cores and read the shims' processor registers from it? | Yes, in each core's own work rather than in the shared layer: the capability is already an optional one of the core ABI v1, so it changes no surface, and the shared layer needs nothing. Until then a register condition on the hottest loop of a Game Boy game can cost more than a frame |
| **Q29** | `bp when` answers nothing on any core (§13.12). Restore it in two parts: the `brk` and `cop` conditions published by the targets, which the debug bridge already fires from the call kinds a core reports; and the rest (`stp`, `wdm`, `ppuaccess`, `autojoy` and each console's own) as a debug event kind added to the core ABI? | The first part now, as a small extension of the shared layer (one list beside `CoreDebugTarget`'s and the shims'), proposed before it is built. The second as an extension of the debug surface, an optional capability and a minor version of the core ABI, only when a core's own work wants a condition reported |

---

## 17. Where this page meets `EmuSen_Surfaces.md`

### 17.1 The five points of its §6.3, answered

| | The point | The answer here |
|---|---|---|
| 1 | **Narrow consumers take the whole assembly**: Serenity names 6 of the shell's 217 public types, all telemetry, and `EmuSen` 48, mostly the debugger's registries and `IDebugTarget`, and each takes the interpreter, the commands and the sandbox with them | **By assembly, not by namespace.** At S2 (§13.2) the contracts a non-shell component consumes leave the shell: `IDebugTarget` and its value types, the registries, the cheat engine, the telemetry contracts and the system packs move to Crystal's assembly (§9.5, §16 Q1). `EmuSen` and Serenity then reference it and not the shell, and the shell's consumers are its hosts alone. `EmuSen_Multicore.md` §9.2's pruning test asks whether a boundary buys a separability someone uses; decision 1 is that someone, since a shell with no emulator in it cannot carry the debugger's registries. The fold of Cauldron into DianaSh on 2026-10-07 is not reopened: the telemetry contracts are not revived as an assembly of their own, they move with the debugger they were folded beside, to the component that owns both |
| 2 | **The systems table is filed as a command** (`CoreDescriptor`, `Bin/Commands/EmuSen/CoreCommand.cs:13`), and its rows are filled in the core assembly | `CoreDescriptor` and the table's rows go to Crystal's assembly as data, beside the system packs, which is where `EmuSen_CoreAPI.md` §8.2 puts the table; the `core` command moves to the session module (§8.1) and asks the frontend service for the cores it can open (`games`, §9.9). No command file declares a console fact |
| 3 | **`IDebugTarget` is one interface every engine implements whole**, where the core ABI offers the debugger in capability-flagged parts | Commands never see it again: they call the session service, whose `info` lists capabilities and whose functions answer `NOT_SUPPORTED` where a session lacks one (§9.6). `IDebugTarget` itself is frozen and moved unchanged (§17.2), and the S2 provider adapts it to the table, as `CoreDebugTarget` adapts the core ABI's debug exports to it |
| 4 | **`HostAction` is handled by each host its own way** | Retired (§6.2): shell control by builtins whose effect a host reads from `dianash_submit`'s status, the emulator's by run control and its events, opening a game by the frontend service. Every host then gets one answer for each, Pharaoh included, and the identical `SyncSchedulersToSessions` copies disappear because shell sessions live inside the shell (§10.2) |
| 5 | **A global in the shell carries a core's setting**: `DebugSettings.MasterLoggingEnabled` forwards to `DianaShLogging.MasterEnabled` | The switch is the emulator's log writer's, and leaves the shell at S1 for Crystal's assembly; `log on`, `log off` and `log status` call `log_switch` and `log_status` of the session service (§9.4). The shell holds no static that another component reads |

### 17.2 Where the two pages disagree, and the recommendation

- **Where Crystal's pieces go.** `EmuSen_Surfaces.md` §7's S5, following `EmuSen_CoreAPI.md` §10.2's D1, moves the
  engine SPI, the native host and the systems table into `DianaSh/Sys/`, and its Q2 recommends the display shapes for
  "the systems table in DianaSh". Under decision 1 the shell contains no emulator, so neither can be in it.
  **Recommended: this page's**, Crystal's own assembly beneath the shell (§16 Q1), which the cores and Serenity
  reference in place of today's references to the shell. The audit's reasoning for its Q2 carries over unchanged once
  "DianaSh" there is read as Crystal: console facts belong to the systems table, and the table to Crystal.
- **What the shell's surface rule is.** The audit leaves it to this page (its §6.2, the row "DianaSh"). This page gives
  the three C surfaces that page's rule **A**, stable with a baseline (§11), and the C# facade the frontends call during
  the migration (§13.3) its rule **D**, a recorded public surface with no promise across commits, until the facade is
  deleted at S7.
- **`IDebugTarget` frozen.** The audit's Q7 recommends that it grow no further and not be split, since its seven
  implementers retire with the C# cores. **The two pages agree**: S2 moves it unchanged, the session service is where a
  new debugger capability is offered to commands, and the core ABI's debug interface is where a core offers one.
- **Serenity's reference for `coretop`.** The audit's Q5 keeps it until this page decides where the telemetry contracts
  live. **Decided here, with Q1:** with Crystal's assembly; Serenity's reference moves there at S2, and Serenity
  then references the shell no longer.


---

## 18. A terminal interface, as a host's extension (planned 2026-10-10)

Decided 2026-10-10 by the tester: the shell gets a full terminal interface, and `hotaru` can show its game in the
terminal, as a planned extension built on the Rust shell after S3. Nothing here is built.

### 18.1 What it is

A **panes mode** of the terminal host (§10.4; `emusen-shell` in C# meanwhile, the Rust `dianash` program after S3):
the prompt, with the line editor of §12.3, beside panes that each show a command's output live, refreshed at a rate
the pane names. The panes are the debugger's views as a terminal debugger lays them out: registers, disassembly at
the program counter, memory, breakpoints, `coretop`, `vstop`, the feed. It is what `gdb`'s TUI and `htop` are to their
programs, and it works over SSH, so a game running on one machine can be debugged from another's terminal with no
window forwarded.

### 18.2 Where it sits, and why there

The form, by the project's rule for a feature that must not give a component a second job:

- **The shell core is unchanged.** It still produces text through the host API (§10) and knows nothing of a terminal's
  layout; a pane is the host's.
- **Commands are unchanged.** A pane re-runs a command and shows its text, as `watch` and `coretop`'s refresh do today;
  no command learns to draw a pane, so the Unix rule that a command is text in and text out holds.
- **A host extension.** The panes mode is the terminal host's own, an option of the program (`--panes`) and a layout
  file in the player's configuration, read as text. The host API gains nothing for it, except where a pane needs a
  command's output without its echo, which the host already has (§10.2).
- **Built on `ratatui` and `crossterm`**, which are pure Rust and work on Linux, macOS and Windows, and not on ncurses,
  which is C and Unix-only.

### 18.3 The game's picture in a terminal

An option of `hotaru` (`hotaru --in-terminal`), never its default: the graphical window stays the accurate picture,
with the CRT filter and the RetroArch presets, and the terminal's picture is a convenience for SSH and for a glance.

| Protocol | Picture | Terminals |
|---|---|---|
| **kitty's graphics protocol**, through shared memory on the same machine and as data over SSH | the console's pixels, scaled by the terminal; no filter | kitty, WezTerm, Ghostty, Konsole in part |
| **Sixel** | colour quantised to the terminal's palette | foot, WezTerm, xterm, VTE |
| **Unicode half-blocks** (two pixels to a character cell, each a colour) | low resolution; any colour terminal | everywhere |

The capability is discovered from the terminal when `hotaru` starts and falls back in that order. Bandwidth is the
constraint over SSH: a 256 × 224 frame is 229 KB in RGBA, about 14 MB/s at 60 frames a second before encoding, so over a
network the picture is sent at a reduced rate or at half-blocks, chosen by the measurement the host makes and stated
in the pane's title **[A]**.

### 18.4 What it needs from the rest of the design

- **`hotaru`** (the command the C# frontend Hotaru was folded into, 2026-10-10) takes an output choice from the
  start, so the terminal picture is an addition and not a change to its surface.
- **The session service** (§9) already gives a pane what it shows: registers, memory and the rest are commands over
  it. A picture in a pane needs the frame, which `hotaru` holds, not the shell.
- **Its gate**: the golden corpus (§13.1) holds every command a pane runs, so a pane cannot change what a command
  prints; the panes mode itself is tested by rendering its layout to a buffer (`ratatui`'s test backend) and comparing
  the buffer.

---

## Appendix A. `dianash_module.h`, sketched

Not normative until S3 is built; written so that the rules above can be checked against a shape. *S3 built it: the
normative header is `Platform/dianash/include/dianash_module.h`, and §13.7 says where it departs from this sketch.*

```c
/* dianash_module.h - the command module ABI of DianaSh. See EmuSen_DianaSh.md §5 and §6. */
#ifndef DIANASH_MODULE_H
#define DIANASH_MODULE_H
#include <stddef.h>
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif

#define DIANASH_MODULE_ABI_MAJOR 1u
#define DIANASH_MODULE_ABI_MINOR 0u

#define DIANASH_OK                 0
#define DIANASH_NULL              (-1)
#define DIANASH_BAD_STRING        (-5)
#define DIANASH_BUFFER_TOO_SMALL  (-7)
#define DIANASH_NOT_SUPPORTED     (-256)
#define DIANASH_BAD_STRUCT        (-263)
#define DIANASH_UNKNOWN_COMMAND   (-265)
#define DIANASH_SERVICE_MISSING   (-266)
#define DIANASH_BUSY              (-267)
#define DIANASH_ABSENT            (-268)   /* an answer, not a failure: no such variable */
#define DIANASH_FAULT             (-320)

#define DIANASH_CALL_READ_ONLY_CONTEXT (1u << 0)
#define DIANASH_CALL_INTERACTIVE       (1u << 1)

#define DIANASH_RESOLVE_READ   (1u << 0)
#define DIANASH_RESOLVE_WRITE  (1u << 1)
#define DIANASH_RESOLVE_REMOVE (1u << 2)   /* refused where the suspensions say (man rm) */

typedef struct { const uint8_t *data; size_t len; } dianash_bytes;

typedef struct {
    uint32_t size;
    uint32_t columns, rows;
    uint32_t raw_capable;
} dianash_terminal;

typedef struct {
    uint32_t size;
    void *ctx;
    int32_t (*out)(void *ctx, const uint8_t *data, size_t len);
    int32_t (*err)(void *ctx, const uint8_t *data, size_t len);
    int32_t (*resolve)(void *ctx, const uint8_t *path, size_t len, uint32_t flags, uint8_t *out, size_t cap, int64_t *needed);
    int32_t (*display)(void *ctx, const uint8_t *real, size_t len, uint8_t *out, size_t cap, int64_t *needed);
    int32_t (*cancelled)(void *ctx);
    int32_t (*slot)(void *ctx, void **state, void (**release)(void *state));
    int32_t (*environment)(void *ctx, const uint8_t *name, size_t len, uint8_t *out, size_t cap, int64_t *needed);
    int32_t (*terminal)(void *ctx, dianash_terminal *out);
} dianash_io;

typedef struct {
    uint32_t size;
    dianash_bytes name;          /* "emusen.session" */
    uint32_t major, minor;
    const void *table;           /* the service's own table; never read by the shell */
} dianash_service;

typedef struct {
    uint32_t size;
    dianash_bytes command;
    const dianash_bytes *argv;
    size_t argc;
    dianash_bytes stdin_data;
    uint32_t stdin_present;
    uint32_t flags;
    dianash_bytes cwd;
    dianash_bytes user;
    const dianash_io *io;
    const dianash_service *services;
    size_t service_count;
} dianash_call;

typedef struct {
    uint32_t size;
    int32_t exit_status;
} dianash_result;

uint32_t dianash_module_abi_version(void);
int64_t  dianash_module_manifest(uint8_t *out, size_t len);
int32_t  dianash_module_run(const dianash_call *call, dianash_result *result);
int64_t  dianash_module_jobs(const dianash_call *call, uint8_t *out, size_t len);        /* optional */
int32_t  dianash_module_kill(const dianash_call *call, const uint8_t *id, size_t id_len); /* optional */

#ifdef __cplusplus
}
#endif
#endif
```

## Appendix B. `emusen_session.h`, its first entries sketched

*The sketch as the design left it. The header as built is `EmuSen.Crystal/include/emusen_session.h`, and §13.6 says where
it departs from this.*

```c
/* emusen_session.h - the session service, emusen.session@1. Owned by Crystal. See EmuSen_DianaSh.md §9. */
#define EMUSEN_SESSION_MAJOR 1u
#define EMUSEN_SESSION_MINOR 0u

#define EMUSEN_SVC_NO_SESSION     (-1100)
#define EMUSEN_SVC_NO_SUCH_SPACE  (-1101)
#define EMUSEN_SVC_READ_ONLY      (-1102)
#define EMUSEN_SVC_NOT_SUPPORTED  (-1103)
#define EMUSEN_SVC_BAD_ARGUMENT   (-1104)
#define EMUSEN_SVC_NO_SUCH_ID     (-1105)
#define EMUSEN_SVC_PARSE          (-1106)

typedef struct emusen_session_v1 {
    uint32_t size;               /* the provider's size of this table */
    uint32_t version;            /* (MAJOR << 16) | MINOR */
    void *ctx;
    int64_t (*last_error)(void *ctx, uint8_t *out, size_t len);

    /* reads */
    int64_t (*sessions)(void *ctx, uint8_t *out, size_t len);
    int64_t (*info)(void *ctx, uint32_t session, uint8_t *out, size_t len);
    int64_t (*summary)(void *ctx, uint32_t session, uint8_t *out, size_t len);
    int64_t (*spaces)(void *ctx, uint32_t session, uint8_t *out, size_t len);
    int64_t (*space_read)(void *ctx, uint32_t session, const uint8_t *space, size_t space_len,
                          uint32_t address, uint8_t *out, size_t len);
    int64_t (*processors)(void *ctx, uint32_t session, uint8_t *out, size_t len);
    int64_t (*registers)(void *ctx, uint32_t session, uint32_t processor, uint8_t *out, size_t len);
    int64_t (*disassemble)(void *ctx, uint32_t session, uint32_t processor, const uint8_t *space, size_t space_len,
                           uint32_t address, uint32_t count, const uint8_t *hints, size_t hints_len,
                           uint8_t *out, size_t len);

    /* changes: applied at the next frame boundary, refused in a read-only context */
    int32_t (*space_write)(void *ctx, uint32_t session, const uint8_t *space, size_t space_len,
                           uint32_t address, const uint8_t *data, size_t len);
    int32_t (*set_register)(void *ctx, uint32_t session, uint32_t processor, const uint8_t *name, size_t name_len,
                            uint64_t value);
    int32_t (*run_control)(void *ctx, uint32_t session, uint32_t op, uint32_t processor, int64_t argument);
    int64_t (*bp_add)(void *ctx, uint32_t session, uint32_t processor, const uint8_t *spec, size_t spec_len);
    int32_t (*bp_remove)(void *ctx, uint32_t session, uint32_t id);
    /* … the groups of §9.2, each appended in the order the baseline records … */
} emusen_session_v1;
```
