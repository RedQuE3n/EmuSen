# EmuSen — DianaSh: a standalone shell, its command modules, and the surfaces around it

*This revision: the third, 2026-10-10: S0 built, the golden corpus, its harness and what it recorded (§13.4). The
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
  `state`, and its console's `clear`; `MainWindow.axaml.cs:598`, `DianaShConsoleWindow.cs:80`), Hotaru four
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
| Mistress | a console window per main window: a session manager, a scheduler per session, read-only lines run on the UI thread, the rest drained by the emulation thread each frame (`DrainPendingFromEmulationThread`) | `DianaShConsoleWindow.cs` (175 lines), `MainWindow.axaml.cs:1332` |
| Hotaru | the F4 prompt and an always-live terminal on a reader thread; while halted at a breakpoint, lines go to `TakeHaltedLine` on the thread that owns the core; `HostAction.Resume`, `Step`, `Shutdown`, `LoadCore` and `SwitchSession` steer its loop | `GameWindow.axaml.cs` (704 lines), `DianaShShellWindow.cs`, `Program.cs` |
| Pharaoh | an unknown `--commands` verb falls through to an interpreter built over the bundle's debug target | `CommandsScriptRunner.cs:19`, `Program.cs:237` |
| WiseMan | `CreateDefault` over fake targets and real cores, 645 cases in `DianaSh/` (55 files, 9,967 lines) and 43 more files elsewhere | `Fixtures/DianaShellFixtures.cs` |
| The conformance kit | none yet; `EmuSen_CoreAPI.md` §12.2 plans Pharaoh as the runtime suite's client | — |

Every host reaches the shell's classes directly, and every one constructs its commands' dependencies itself. The
copying shows: `SyncSchedulersToSessions` is identical in `DianaShConsoleWindow.cs` and `GameWindow.axaml.cs`
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
golden outputs of `man` for every page (§13.2, S1).

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
| `FrameRecorder` | not a command's: Hotaru's F6 recording, which stays the frontend's |
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
| `games` (JSON: the cores and systems a frontend can open) and `open_game(core, path)` | `CoreCommand` and `HostAction.LoadCore`; retired when Crystal opens sessions itself |
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

### 13.2 The steps

No step leaves a broken shell; each has its gate, and each is its own commit or group of commits.

| Step | What | Gate |
|---|---|---|
| **S0** | The golden corpus of §13.1, recorded from the C# shell | it covers every usage line (a test enumerates them); two recordings on two runs are identical |
| **S1** | The C# shell reorganised without changing what it prints: the 98 manual pages out of `ManPages.cs` into files read by the C# `man`; the five aliases and the read-only fast path made one table; `summary` and `log` turned into ordinary command classes; `Var/`'s shell state (`CommandHistory`, the sessions) separated from the debugger's registries | the golden corpus; `ManPageTests` |
| **S2** | The service API's header, `emusen_session.h`, and a C# provider over today's `IDebugTarget` and registries, exposing the table through `[UnmanagedCallersOnly]` functions; the registries, the cheat engine, `IDebugTarget`, the telemetry contracts and the system packs moved out of `EmuSen.DianaSh` into Crystal's assembly, `EmuSen.Crystal`, beneath the shell and beside the cores (§16 Q1), which `EmuSen.DianaSh` and the cores reference | a parity class calling every service function and the C# member it maps (§9.3) on the fake target and each core, equal; the golden corpus |
| **S3** | The Rust shell core, `Platform/dianash/`, with its builtins, lookup, manifests, the host API in the platform library and the `dianash` program; every C# command still runs, registered with the Rust shell as host commands through `dianash_add_commands` by a C# bridge that calls each `IDianaShCommand.Execute`; the C# frontends choose the C# interpreter or the Rust one by `EMUSEN_DIANASH_NATIVE` (§13.3) | the golden corpus, with the variable unset (the C# interpreter) and `1` (the Rust shell with C# commands); the language's 645 cases both ways |
| **S4** | `dianash.text` in Rust, command by command, each replacing its C# bridge entry when its golden file passes | that command's golden file and its WiseMan cases both ways; the same corpus run on Windows and macOS CI runners for the text module, since it must behave identically there |
| **S5** | The four EmuSen modules in Rust over the session service, module by module, each command replacing its bridge entry when its golden file passes; `nano` as a program | that command's golden file both ways, on the fake provider and on each core; seeded faults in the module, as each platform step had (`EmuSen_RustPlatform.md` §6.5) |
| **S6** | The shell to state 2, as `EmuSen_RustPlatform.md` §3.9 defines it: the Rust shell by default, `EMUSEN_DIANASH_NATIVE=0` selecting the C#, a missing library falling back with one line in the error log | a full WiseMan run with the variable set; the tester's word |
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
`CoreCatalog.Registry`; `state` over a save and a load that write and read a file, defaulting to a fixed path in the
home's save-state folder; the bundle's two cheat codecs and trace switch; a session manager unless the case says
otherwise; and the build's supported cheat systems.

**How the status is read.** `$?` lives in the interpreter's variables, and the only member that reads them,
`GetVariable`, is not public. The harness reads it by reflection, as it reads `_orderedCommands` for the list of
commands and their usage lines. Neither the shell's code nor its project file was changed for the corpus: an
`InternalsVisibleTo` would have done the same job with a check at compile time, and was rejected because the step's
condition was that recording change nothing in the shell. The price is that renaming either member breaks the harness
when it runs, not when it builds, and does so on its first case.

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
Mars core, running them, throws `IndexOutOfRangeException` in `Cpu.StepBlock` within the first frame. That is a defect
of the core, not of the shell, and is recorded here rather than worked on.

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
10. `dump`, `load` and `cov save|load` join their file name to `Logs/<core>/` without the sandbox's resolution, so
    `../../../x` reads and writes outside the root (shown inside the corpus's temporary folder), and an absolute name
    would reach any path; `core` and `state` pass their paths to the host unresolved too (`dump`, `cov`, `core`).
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
    call stack, and both report frame 0 after two frames, MoonRT naming itself Moon.
20. Nephrite names its main processor `m68k`, not the `cpu` every command takes as the main one, so `regs cpu` and
    `disasm cpu` refuse there; and its `tile` throws `IndexOutOfRangeException`.
21. `core`'s registry has no entry for the Genesis, so a v1 core cannot be loaded with it.

---

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
  the language. Retired at S3.
- **P3.** The Rust `grep`, `sed` and `awk` cannot use a regular-expression crate unchanged and match .NET's engine byte
  for byte on every case of the corpus; a translation layer or a recorded set of differences will be needed. Retired at
  S4, with the decision recorded.
- **P4.** The standalone `dianash` starts in under 20 ms on the desktop with no module loaded. Retired at S3.
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
| **Q15** | The sandbox's case-insensitive comparison on Linux, which can accept a path outside the root as inside it (§12.1): kept as today, or made case-sensitive where the file system is? | Kept through S3 so the golden corpus holds, then corrected in the C# and the Rust together, with a test that shows the defect first *Decided 2026-10-10 as recommended.* |
| **Q16** | The golden corpus committed as files in WiseMan (§13.1)? | Yes. It holds no ROM data, and it is the record that lets C# be deleted at S7 *Decided 2026-10-10 as recommended.* |
| **Q17** | The host API's versioning: stable from S3, though its only hosts until then are the project's own frontends built from the same commit? | Stable from S3. The baseline costs one CI step, and an outside frontend or the kit can then embed the shell the day it wants to *Decided 2026-10-10 as recommended.* |

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

## Appendix A. `dianash_module.h`, sketched

Not normative until S3 is built; written so that the rules above can be checked against a shape.

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
