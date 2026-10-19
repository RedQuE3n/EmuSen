# EmuSen — the surfaces between components: what is tangled, and the defined surfaces proposed

*This revision: the first, 2026-10-10. An audit, not an implementation: nothing in it is built, and no code changed
to make it. Every claim about the code is cited to a file, read on `WiseMan` at `96c3f563`, and to the sibling
toolkit checkout `EmuSen.LunaP` on `openemu-library` at `e098b08`. Claims marked **measured** were measured on
2026-10-10 by the commands or the tool §1 names; claims marked **argued** are reasoning that a later step must
prove, and the step's gate in §7 says how. Revised 2026-10-10 where `EmuSen_DianaSh.md` §16 Q1 decides it: §7's S5 and
§8's Q2 (§17.2 there); this page's own ten questions are not decided.* *Revised 2026-10-17: S1 and S3 built, with their gates as measured (§7.1).*

Companion docs: `EmuSen_CoreAPI.md` (the core ABI v1, and in §8–§10 the runtime and the frontend API that this page
does not redesign), `EmuSen_NativeCores.md` (the pre-stable core interface and its C# host), `EmuSen_RustPlatform.md`
§3 (the platform library's C interface, whose rules §6 reuses), `EmuSen_DianaSh.md` (the shell's stable ABI and API,
being designed beside this page; §6.3 says where the two meet), `EmuSen_Multicore.md` §9 (the leaf rules),
`EmuSen_Stack.md` §2.3 (the contract-in-a-leaf rule), `EmuSen_LunaP.md` (how EmuSen consumes the toolkit) and LunaP's own `docs/LunaP.md` §1 and §16 (its layering rule).

---

## 0. Why, and the result in brief

### 0.1 The question

The tester, 2026-10-10: the project has grown "spaghetti", and the aim is "clean, clear, sane defined surfaces". Two
boundaries already have a written, versioned interface: the core ABI v1 (`EmuSen_CoreAPI.md` §4–§6, guarded by a
baseline) and the platform library's C interface (`EmuSen_RustPlatform.md` §3, matched exactly). This page audits
every other boundary between components, by measurement rather than impression, ranks what it finds by the harm each
problem does, and proposes for each component a defined surface: what it exposes, what it hides, and which stability
rule holds it.

### 0.2 What was found, in brief

The graph has no cycle, the Rust crates share code as the project's rule asks, and the three `InternalsVisibleTo`
friends are all the test harness (§2, §5). The tangles are elsewhere, and in order of harm (§4):

1. **There is no frontend API.** Mistress drives cores through `EmulatorSession`, Hotaru and Pharaoh through `ICore`
   directly; each runs its own frame loop, and the copies already disagree: five post-load sequences for one operation,
   rewind capture that asks the engine in one client of three, a shell `load` that works in Hotaru and not in Mistress
   (§4.1).
2. **The core library depends on the presentation layer**, `EmuSen → Serenity`, for two constants and one record, so
   the headless runner ships 24 Avalonia assemblies. Nothing states or tests the rule this breaks (§4.2).
3. **Global mutable statics are interfaces**: 104 declarations, 38 used and 26 written across components. `GraphicsSettings`
   is a second, hand-synchronised copy of `GraphicsConfig` that the presentation layer reads, against the rule written
   for it; two such statics are on record as having serialised and crashed the test suite (§4.3).
4. **`CoreCatalog` is a god object** of nine concerns, and the systems table is spread over three assemblies (§4.4).
5. **Frontends name the C# cores' internals**: Hotaru 4 types, Pharaoh 18 (§4.5).
6. **`IDebugTarget` is a god interface**: 45 members, 7 implementers (§4.6).
7. **Nothing is hidden by default**: 268 of EmuSen's 337 public types have no caller but the tests, and the tests reach
   106 private members of Mistress's `MainWindow` by reflection (§4.7).
8. **Facts are duplicated where code is not**: 19 places say which bit is which button, settings have three
   definitions, an extension up to six (§4.8).

Below these, narrow consumers taking whole assemblies (§4.9), environment variables and files with no recorded
contract (§4.10), and LunaP reached by an absolute path (§4.11).

### 0.3 What is proposed, in brief

Six stability rules, five of them the project's own (§6.1): the core ABI's stable C ABI, the platform library's exact
C interface, a versioned C# API for the frontend API's binding and for LunaP, the Rust path-dependency rule, `internal`,
and one new rule, a **recorded internal C# surface**, which promises nothing but writes an assembly's public surface into
the repository so that a change to it is a line in a diff. Each component is given one of them, with what it exposes
and hides (§6.2); DianaSh's is left to `EmuSen_DianaSh.md`, with the five things this audit asks that design to answer
(§6.3). An eleven-step order follows (§7), each step with a gate, beginning with a recording of today's surfaces and the
cut of `EmuSen → Serenity`, and placing the runtime work that `EmuSen_CoreAPI.md` §10.2 already plans where that page
puts it. Ten questions close it (§8).

### 0.4 What this page does not do

- **It does not redesign DianaSh.** The shell's target surface, its stable ABI and API, is `EmuSen_DianaSh.md`'s.
  This page measures DianaSh's tangles as they stand (§4) and states, in §6.3, what the rest of the platform needs
  from that design; it does not choose its shape.
- **It does not revise the two surfaces that already exist.** The core ABI v1 and the platform library's C interface
  are taken as given. Where a proposal here needs a rule, it reuses theirs.
- **It does not repeat `EmuSen_CoreAPI.md` §10.** That section's gap list and its steps D1–D5 already plan the move of
  the engine SPI, the native host, `CoreCatalog`, the sessions and the frame loop into the runtime. §7 orders this
  page's steps around them rather than re-planning them.

---

## 1. Method

### 1.1 What was measured, and with what

| What | How |
|---|---|
| Project references, packages, `InternalsVisibleTo` | every `.csproj`, `.props` and `.targets` read; `grep -n "ProjectReference\|PackageReference\|InternalsVisibleTo"`, and `[assembly: InternalsVisibleTo]` in `.cs` files |
| Crate dependencies | every `Cargo.toml` read; every `Cargo.lock` counted for duplicated versions |
| Which assembly uses what of which | a metadata reader (`System.Reflection.Metadata`) over the assemblies built from `96c3f563` with `dotnet build EmuSen.WiseMan -p:EmuSenNative=false` (the C# alone; no Rust is needed to read the C#). It walks every method body's IL and records each reference to a type or member of another EmuSen assembly, with the type in which the reference sits. The tool is not committed; its rules are in §1.2 so that the counts can be reproduced |
| Public surface | the same reader: a type is **public** if it and every type enclosing it is public (or nested protected); a member is public if public or protected, counted by name, so overloads count once and property accessors count as their `get_`/`set_` methods |
| Mutable statics | the same reader: public static fields that are neither `readonly` nor `const`, public static setters, and every `stsfld`/`ldsfld` of them from another assembly |
| Environment variables, reflection, files, duplication, the frontends | `grep` over the sources excluding `bin/`, `obj/` and `target/`, with the commands given beside each count |

### 1.2 What the reader can and cannot see

- **An outside caller** is a type in another assembly whose IL names the member. A lambda's or an iterator's
  compiler-generated class is folded into the type that wrote it. "Used by one caller" means exactly one such type.
- **Constants are invisible**: the compiler copies a `const` into the caller. §4.2 meets this, where an edge carried
  by constants shows up as one type.
- **Reflection, XAML bindings by name and `dynamic` are invisible** to an IL walk; they are counted by `grep` instead
  (§4.7).
- **Members are matched by name.** A name with an internal and a public overload is tallied as internal, which
  inflates §2.3's internal-member counts for constructors; the four Endymion constructors this flagged as used by
  Mistress and Hotaru were checked by hand and are public.
- **WiseMan is counted separately throughout.** It is the test harness, and its references show what is tested, not
  what is depended on. "Non-test" means every assembly but WiseMan.

### 1.3 Marks

**Measured** means a count or an output on 2026-10-10 at `96c3f563`. **Argued** means reasoning, with the step that
must show it. Where a document states a rule, the rule is quoted with its section, and §2.5 says whether the code
keeps it.

---

## 2. The dependency graph (measured)

### 2.1 The C# projects

Nine projects in the solution and the toolkit beside it, in the layers of `EmuSen_CoreAPI.md` §3, top to bottom (§4.2 says why that page's and not `EmuSen_Multicore.md` §9's). Arrows are `ProjectReference`s; every arrow points down except one; LunaP arrives through
`LunaP.props`, which names the sibling checkout's project by absolute path.

```
  layer          project     ──► its ProjectReferences
  ─────────────  ──────────  ──────────────────────────────────────────────────────────────────────────
  clients        Mistress    ──► EmuSen, Serenity, Endymion, Galaxia, LunaP
                 Hotaru      ──► EmuSen, Serenity, Endymion, LunaP
                 Pharaoh     ──► EmuSen
                 WiseMan     ──► the other eight, and LunaP                                  (tests)

  presentation   Serenity    ──► DianaSh, Galaxia, LunaP
  and devices    Endymion    ──► Galaxia
                 LunaP       ──► (Avalonia packages only; a separate repository)

  cores          EmuSen      ──► DianaSh, Galaxia, Serenity     ◄── §4.2: the cores reach up into presentation
                               ┄┄ the Rust cores' libraries, built by RustCores.props, called by P/Invoke

  shell          DianaSh     ──► Galaxia

  leaf           Galaxia     ──► nothing
                               ┄┄ emusen_platform, built by Platform.targets, called by P/Invoke
```

The same graph as a table, with what the IL walk found on each edge. "IL refs" counts instructions that name a type
or member of the target; "types" counts the distinct target types named.

| From | To | Declared | IL refs | Types | Note |
|---|---|---|---:|---:|---|
| EmuSen | DianaSh | yes | 1,053 | 48 | the debugger's registries, `IDebugTarget`, telemetry |
| EmuSen | Galaxia | yes | 50 | 10 | `PadButton`, `DataStore`, `SaveLibrary`, `GraphicsConfig` |
| EmuSen | **Serenity** | yes | **4** | **1** | `DisplayReading`; plus two `const`s the IL cannot see (§4.2) |
| DianaSh | Galaxia | yes | 80 | 10 | |
| Endymion | Galaxia | yes | 20 | 5 | |
| Serenity | DianaSh | yes | 28 | 6 | telemetry for the `coretop` window only |
| Serenity | Galaxia | yes | 27 | 4 | |
| Serenity | LunaP | via props | 47 | 12 | |
| Mistress | EmuSen | yes | 445 | 31 | |
| Mistress | DianaSh | **no** | 243 | 27 | reached transitively through EmuSen |
| Mistress | Galaxia | yes | 597 | 17 | |
| Mistress | Endymion | yes | 190 | 11 | |
| Mistress | Serenity | yes | 85 | 15 | |
| Mistress | LunaP | via props | 2,151 | 99 | |
| Hotaru | EmuSen | yes | 106 | 18 | 4 of the 18 are Venus's internals (§4.5) |
| Hotaru | DianaSh | **no** | 85 | 23 | |
| Hotaru | Galaxia | **no** | 6 | 4 | |
| Hotaru | Endymion | yes | 34 | 6 | |
| Hotaru | Serenity | yes | 16 | 5 | |
| Hotaru | LunaP | via props | 66 | 17 | |
| Pharaoh | EmuSen | yes | 229 | 40 | 18 of the 40 are Moon's or Venus's internals (§4.5) |
| Pharaoh | DianaSh | **no** | 42 | 8 | |
| Pharaoh | Galaxia | **no** | 6 | 3 | |
| WiseMan | EmuSen · DianaSh · Galaxia · Endymion · Serenity · Mistress · Hotaru · Pharaoh · LunaP | yes | 13,957 · 4,184 · 2,248 · 1,463 · 1,733 · 7,190 · 64 · 66 · 1,541 | 213 · 139 · 41 · 19 · 47 · 203 · 10 · 4 · 81 | |

Five edges carry traffic that no `.csproj` declares: every frontend reaches DianaSh, and Hotaru and Pharaoh reach
Galaxia, through EmuSen's references. That is legal MSBuild, and it means a frontend's dependency list says less than
the frontend does.

*Revised 2026-10-11, by `EmuSen_DianaSh.md` §13.6 (that page's S2).* The graph and the table above are as measured at
`96c3f563`, and the IL counts were not taken again. Since that step a new assembly, `EmuSen.Crystal`, referencing
Galaxia alone, sits beneath the shell and beside the cores, holding `IDebugTarget`, the registries, the cheat engine,
the telemetry contracts and the system packs. The sets are now: DianaSh ──► Crystal, Galaxia; EmuSen ──► Crystal,
Galaxia, Serenity, its edge to DianaSh gone; Serenity ──► Crystal, Galaxia, LunaP, its edge to DianaSh gone; Mistress,
Hotaru and Pharaoh declare DianaSh and Crystal, which closes three of the five undeclared edges (Hotaru's and Pharaoh's
to Galaxia remain); WiseMan declares Crystal. S0's recorded file does not exist yet, so `LeafAssemblyTests` alone pins
the sets of Crystal, DianaSh, EmuSen and Serenity.

**Packages.** Galaxia and DianaSh take none. Endymion takes three (SDL3-CS, SDL3-CS.Native, Avalonia's base);
Serenity nine (four Avalonia packages, SkiaSharp, Silk.NET.Shaderc, Silk.NET.Vulkan, Microsoft.Data.Sqlite,
SQLitePCLRaw); EmuSen four (Microsoft.Data.Sqlite, SkiaSharp, SQLitePCLRaw, Silk.NET.Vulkan); Mistress seven; Hotaru
six; Pharaoh two (the SQLite pair, "for LibraryIndexer/SqliteCatalogue only", which EmuSen already brings:
`EmuSen.Pharaoh.csproj:18–21`); WiseMan eight; LunaP six, all Avalonia.

### 2.2 The Rust crates

Thirteen separate Cargo builds, each with its own `Cargo.lock`, plus the probe's. No workspace spans the cores.

```
   marsrt ─┬──────────────────────────────┐  ash 0.38, cranelift-{codegen,frontend,jit,module,native} 0.136
   mercuryrt ─┤                            │
   moonrt ────┤                            │
   venusrt ───┤                            │
   nephrite ──┼─► beryl-m68k ─┐            │
              ├─► beryl-z80 ──┤            │
              ├─► beryl-sh2 ──┤            │
              └─► beryl-sn76489 (none) ┐   │
                              ▼        ▼   ▼
                           emusen-native  (no dependencies: state codec, both core ABIs, debug hooks, crash log,
                              ▲   ▲   ▲     a JSON reader for test suites, a JSON writer for descriptors)
   emusen-core-abi-check ─────┘   │   │   clang-sys, bindgen
   emusen-core-conform ───────────┘   │   libloading 0.8
   Platform/platform ─────────────────┘   + emusen-galaxia, emusen-endymion, emusen-serenity
     emusen-serenity ─► emusen-galaxia;   ash 0.38, libloading 0.9, sha2
     emusen-endymion:  libloading 0.9, md-5        emusen-galaxia: md-5
   probe-rs (WiseMan/Reference): crc32fast, libloading 0.9 (optional), bindgen (optional)
```

What each crate takes from `emusen-native`, by `grep -rhoE "emusen_native::[a-z_]+"`:

| Crate | Modules used (occurrences) |
|---|---|
| marsrt | `ffi` (4), `naming` (2) — still on its own legacy ABI |
| mercuryrt | `debug` 6, `ffi` 4, `core` 3, `abi` 3, `naming` 2, `native_exports` 1, `core_exports` 1 |
| moonrt | `debug` 11, `ffi` 4, `naming` 2, `samples`, `native_exports`, `core_exports`, `core`, `abi` 1 each |
| venusrt | `ffi` 9, `json` 7, `core` 7, `debug` 5, `abi` 2, `native_exports` 1, `core_exports` 1 |
| nephrite | `debug` 20, `core` 6, `abi` 6, `ffi` 3, `steps` 1, `core_exports` 1 |
| beryl-m68k · z80 · sh2 · sn76489 | `json`+`debug`+`core` · `debug`+`json`+`core` · `debug` · nothing |
| Platform/platform | `abi` 1: `install_crash_log`, the only use (`Platform/platform/src/lib.rs:127`) |

The Rust graph is acyclic and shallow, and it keeps the project's sharing rule where the rule has been applied: the
Sega cores' processors are Beryl's, and every core's state, ABI glue and debug hooks are `emusen-native`'s. Two
things in it are worth a line each. MercuryRT, MoonRT and VenusRT export the pre-stable ABI and the v1 ABI side by
side (`native_exports!` and `core_exports!`), as `EmuSen_CoreAPI.md` §13.5 plans until the pre-stable set is deleted.
And `libloading` is resolved at two versions, 0.8.9 in four lock files and 0.9.0 in two.

### 2.3 `InternalsVisibleTo`

Three declarations, all naming `EmuSen.WiseMan`: `EmuSen.Galaxia.csproj:32`, `EmuSen.Endymion.csproj:21` and
`EmuSen.Serenity/AssemblyInfo.cs:4`. No non-test assembly is a friend of another. What the friend uses, by the IL walk:
from Galaxia 12 internal types and 91 internal members; from Serenity 14 internal types and 66 internal members, every
one of them the `Managed` half of a component kept as the oracle of its Rust port, or a probe seam
(`SlangProbe`, `ISlangProbe`, `GameFrameControl.DrawOp`); from Endymion 3 internal types. These are the parity
classes of `EmuSen_RustPlatform.md` §10.5, §12 and §16.5 calling both halves, which is what the declarations' own
comments say they are for. **This boundary is clean.**

### 2.4 Cycles avoided by workaround

None of the graph's edges closes a cycle. Seven places in the code exist because one would have:

| # | Workaround | Evidence | Verdict |
|---|---|---|---|
| C1 | Galaxia is held to zero references, so that `EmuSen → DianaSh → Galaxia` stays a chain | `EmuSen.Galaxia.csproj:7–13`; `EmuSen_Galaxia.md` §1 | sound; its costs are C2 and C3 |
| C2 | The catalogue's contract `ICatalogue` and its schema live in Galaxia, the driver `SqliteCatalogue` in EmuSen | `EmuSen_Stack.md` §2.3 rule 2 | sound |
| C3 | Config stays JSON partly because a settings database "would need an interface in Galaxia, a driver in EmuSen, and injection through every frontend" | `EmuSen_Stack.md` §4.1 | a stated choice; not a defect |
| C4 | `IDebugTarget`, `ICheatCodeCodec`, `ICpuTraceSwitch` are declared in DianaSh and implemented in EmuSen, so the shell names no core | `EmuSen.csproj:40–46` | sound dependency inversion |
| C5 | The shell cannot create a core, so `load` returns `HostAction.LoadCore` for its host to perform | `EmuSen.DianaSh/DianaSh/Lib/HostAction.cs:24` | a seam each host fills differently (§4.1) |
| C6 | The systems table's record, `CoreDescriptor`, is declared in the shell's command folder so the `core` command can list it; `CoreCatalog` fills it | `EmuSen.DianaSh/DianaSh/Bin/Commands/EmuSen/CoreCommand.cs:13`; `EmuSen/Cores/CoreCatalog.cs:12–70` | a data contract filed as a command (§4.4) |
| C7 | The platform library is built by Galaxia's project through an `Import`, "neither a project nor a package reference, so the leaf rule above stands" | `EmuSen.Galaxia.csproj:53–56` | sound; the rule's letter holds and its purpose is unharmed |

### 2.5 The leaf rules, as stated and as kept

| Rule | Stated in | Kept? (measured) |
|---|---|---|
| Galaxia: no `ProjectReference`, no `PackageReference` | `EmuSen.Galaxia.csproj:7`; `EmuSen_Galaxia.md` §1 | **kept** (0 and 0; `LeafAssemblyTests.cs:104`) |
| Endymion references only Galaxia | `EmuSen_Multicore.md` §9; `EmuSen_Input.md` §4 | **kept** (`LeafAssemblyTests.cs:23`) |
| Serenity references only Galaxia | `EmuSen_Multicore.md` §9's table | **not kept, and the test was changed to match**: Serenity references Galaxia, DianaSh and LunaP (`EmuSen.Serenity.csproj:51,58,61`), and `LeafAssemblyTests.cs:42` now pins that set. `EmuSen_Serenity.md` §1 records the change; `EmuSen_Multicore.md` §9's table does not |
| "A device layer is handed what it needs and reaches for nothing. Not a core, not a session wrapper, not a global settings static. Every knob is a parameter." | `EmuSen_Multicore.md` §9 | Endymion **kept** (it reads none of the static settings classes). Serenity **not kept**: `GameFrameControl.cs:228,256,605` and `FramePresenter.cs:72–75` read the static `GraphicsSettings` |
| No leaf reaches the core assembly | `EmuSen_Multicore.md` §9.2 | **kept** (`LeafAssemblyTests.cs:108–117`) |
| DianaSh references only Galaxia, and no UI toolkit | `EmuSen_Debugging_Tools_Reference_v5.md` §3.3b; `LeafAssemblyTests.cs:61` | **kept** |
| LunaP names nothing of EmuSen's | LunaP's `docs/LunaP.md` §1, §16 | **kept**: no project reference; its one mention of an EmuSen name is a comment (§4.11) |
| The core assembly does not depend on the presentation layer | implied by `EmuSen_CoreAPI.md` §3; stated in no rule of `EmuSen_Multicore.md` §9, which counts Serenity a leaf | **broken**: `EmuSen → Serenity` (`EmuSen.csproj:39`); no test looks (§4.2). *Kept since 2026-10-17: stated in `EmuSen_Multicore.md` §9.4 and pinned by `LeafAssemblyTests` (§7.1)* |
| A client never names a core, an engine type or a console's internals | `EmuSen_CoreAPI.md` §3, as the target | **not yet kept**: Hotaru names 4 of Venus's types and Pharaoh 18 of Moon's and Venus's (§4.5) |

`EmuSen_Audio_Sync.md`'s opening paragraph still calls Endymion "a leaf project with no project references at all"; its own §7.1
records why that stopped being true on 2026-08-05.

---

## 3. Each component's public surface (measured)

### 3.1 The C# assemblies

| Assembly | C# files / lines | Public types | Public members | Types with a non-test outside caller | …named by one non-test assembly only | Members with a non-test outside caller | …named by one outside type only |
|---|---:|---:|---:|---:|---:|---:|---:|
| EmuSen | 326 / 56,046 | 337 | 4,944 | **69 (20%)** | 54 | 307 | 227 (74%) |
| EmuSen.DianaSh | 142 / 16,867 | 217 | 1,451 | 80 (37%) | 58 | 331 | 190 (57%) |
| EmuSen.Galaxia | 28 / 2,643 | 30 | 564 | 29 (97%) | 14 | 361 | 219 (61%) |
| EmuSen.Endymion | 16 / 3,159 | 16 | 231 | 11 (69%) | 5 | 83 | 49 (59%) |
| EmuSen.Serenity | 29 / 6,560 | 49 | 589 | 17 (35%) | 13 | 66 | 52 (79%) |
| EmuSen.Mistress | 158 / 29,027 | 276 | 3,468 | 0 | — | 0 | — |
| EmuSen.Hotaru | 11 / 1,563 | 16 | 69 | 0 | — | 0 | — |
| EmuSen.Pharaoh | 13 / 2,353 | 13 | 95 | 0 | — | 0 | — |
| EmuSen.LunaP (sibling) | 148 / 26,133 | 166 | 2,624 | 99 (60%) | 77 | 645 | 465 (72%) |
| EmuSen.WiseMan | 565 / 121,614 | 672 | 6,739 | — | — | — | — |

Line counts are `wc -l` over the `.cs` files outside `bin/`, `obj/` and `target/`; the rest are the IL walk of §1.2.
LunaP's figures leave out its consumer outside this repository, Pegasus, which takes it as a package
(`EmuSen.LunaP.csproj:72–73,139`, version 0.11.0); they say what EmuSen uses of LunaP, not what LunaP owes.

How many distinct non-test types outside each assembly name it at all:

| Assembly | Its non-test outside callers, by caller |
|---|---|
| EmuSen | Mistress 24, Pharaoh 11, Hotaru 4 (WiseMan 343) |
| EmuSen.DianaSh | EmuSen 41, Mistress 20, Hotaru 4, Pharaoh 2, Serenity 1 (WiseMan 146) |
| EmuSen.Galaxia | Mistress 38, EmuSen 15, DianaSh 13, Endymion 5, Serenity 3, Hotaru 1, Pharaoh 1 (WiseMan 198) |
| EmuSen.Endymion | Mistress 4, Hotaru 2 (WiseMan 47) |
| EmuSen.Serenity | Mistress 9, Hotaru 3, EmuSen 1 (WiseMan 48) |
| EmuSen.LunaP | Mistress 61, Hotaru 7, Serenity 2 (WiseMan 108) |

**What the table says.** Galaxia and Endymion are already close to defined surfaces: almost everything they make
public is used by another component, which is what a surface is. EmuSen is the reverse: four public types in five
have no caller outside the assembly except the tests, and of the members that do have one, three in four have exactly
one. Its public surface is not a surface; it is the default visibility of everything, and the C# cores' internals
(`Mars.Memory.MemoryBus`, 67 test types; `Mars.Cpu.Core.Cpu`, 35; `Venus.VenusCore`, 58) are public because the tests
reach them that way. Mistress's 276 public types have no caller but WiseMan; WiseMan also reaches 106 *private*
members of `MainWindow` by reflection, on 228 lines (`grep -rn "typeof(MainWindow)" EmuSen.WiseMan`), because Mistress
declares no `InternalsVisibleTo` and exposes no surface of its own to test through.

### 3.2 The candidates for a god object, checked

| Type | Size (methods incl. accessors · properties · fields) | Implementers | Non-test outside callers | Verdict |
|---|---|---|---|---|
| `ICore` (`EmuSen/Cores/ICore.cs`) | 30 · 16 · 0 | 7 in EmuSen (Venus, Moon, Mercury, Mars, MarsRT's shim, `CoreEngine`, `NativeRtCore<T>`), 4 test fakes | Pharaoh 5 types, Hotaru 2, Mistress 1 (one member: `TotalFrames`) | **wide but cohesive**: everything on it is the frame, the picture, the sound, input, state or SRAM. The harm is who holds it (§4.1), not its width |
| `IDebugTarget` (`EmuSen.DianaSh/DianaSh/Lib/IDebugTarget.cs`), extending `ICoreTelemetry` | 29 · 19 · 0, plus `ICoreTelemetry`'s 16 · 11 | 7, all in EmuSen | Pharaoh 2, Hotaru 1, Mistress 1; outside DianaSh only `GetMemorySpaces`, `Watches` and `ApplyCheats` are called by a non-test caller | **a god interface**: memory, watches, frame log, breakpoints (two kinds), coverage (two), call stack, labels, counters, freezes, expressions, CPUs, register flow, vectors, DMA, cheats, disassembly, tiles, tilemaps and channel muting on one type that every core must implement whole. It is named in 91 of DianaSh's 142 files, 47 of them commands (`grep -rlw IDebugTarget`); its implementers are seven classes in one assembly. `EmuSen_CoreAPI.md` §6.14 already replaces it at the ABI with capability-flagged parts |
| `CoreCatalog` (`EmuSen/Cores/CoreCatalog.cs`, 401 lines, `static`) | 46 · 10 · 30 | — | Mistress 17 types, Hotaru 3 | **a god object**: console lists in release order, library shelves, ROM extensions and header rules (`WithoutInesHeader`, `InByteSwappedOrder`, `IsGameBoyColor`), buttons, axes and controller diagrams, settings rows, discovered engines and engine choice, cheat systems, screen filters. Nine concerns behind one static name; 20 of its members have a non-test caller and Mistress alone calls 19 |
| `GraphicsSettings` (`EmuSen.Serenity/GraphicsSettings.cs`, namespace `EmuSen.Graphics`) | 3 · 0 · 8 static fields | — | Mistress 3 types, Hotaru 2 | **not a god object; a global**: small, and wrong in kind (§4.3) |
| `AppSettings` (`EmuSen.Galaxia/Models/AppSettings.cs`) | 123 · 58 · 72 | — | Mistress 10 types, DianaSh 1, Pharaoh 1 | **wide, and correctly so**: a plain model of one file, persisted by `ConfigFile<T>`. Its width is Mistress's preferences; it is not a coupling point |
| `DianaShInterpreter` | — | — | Mistress 1, Hotaru 3, Pharaoh 2 | narrow: `CreateDefault`, `Submit`, `Execute`, `History`, `IsAwaitingMoreInput`, `GetWelcomeBanner` are all a client calls |

---

## 4. The problems, ranked by harm

### 4.0 How they are ranked

Harm is judged, not measured, on three questions asked of each problem: **how many components it couples** that
should not know each other; **whether it stands in the way of decided work** (the runtime of `EmuSen_CoreAPI.md`
§8–§10, the C# cores' retirement, the platform's port in `EmuSen_RustPlatform.md`); and **whether it is silent**,
that is, whether anything fails when it gets worse. The evidence under each is measured; the ranking is **argued**.

### 4.1 No frontend API: three frontends drive cores three ways, each with its own loop

**Mechanism.** There is one shared session type, `EmuSen/Common/EmulatorSession.cs` (142 lines), and only Mistress
uses it; Hotaru and Pharaoh hold an `ICore` directly (`ICore` is named by 5 of Pharaoh's types, 2 of Hotaru's and 1
of Mistress's). Even Mistress reaches through it: `session.Core` appears 20 times in 3 files, with 6 capability probes
by type (`as IFrameBufferPool`, `MainWindow.Rewind.cs:69,144`; `as IFrameProfiler`, `MainWindow.axaml.cs:1263`;
`is IRepeatedRows`, `:987`; `is IStateFormat`, `MainWindow.Identity.cs:127,151`; `is ICoreSettings`,
`MainWindow.axaml.cs:484`) and one cast to a concrete engine type (`as EmuSen.Cores.Native.CoreEngine`, `:940`).
No shared loop or runner exists. Each frontend builds its core its own way (`MainWindow.axaml.cs:934`;
`Hotaru/Program.cs:169–179`; `Pharaoh/Program.cs:203–206,235` and again at `Reference/ProbeRunner.cs:45–48`) and
assembles the shared pieces (`FrameScheduler`, `SpeedController`, `RewindBuffer`, `AudioPlayer`, `PortRouter`) in a
loop of its own: Mistress's `EmulationLoop` at `MainWindow.axaml.cs:1239–1464`, Hotaru's at
`GameWindow.axaml.cs:386–475`, Pharaoh's `FrameRunner.cs` (227 lines).

**What the duplication already costs, measured as divergence:**

| Concern | Mistress | Hotaru | Pharaoh |
|---|---|---|---|
| Rewind capture | gated on the engine's opt-out, `EngineFeatures.Of(core).RewindCapture` (`MainWindow.axaml.cs:1349`) | **ungated** (`GameWindow.axaml.cs:435`) | **ungated** (`FrameRunner.cs:177`) |
| After a state load | hotkey: rewind cleared and rate control reset (`MainWindow.axaml.cs:702–704`); console: **rate control not reset** (`:617–620`) | hotkey: both (`:570–572`); console: **neither** (`Program.cs:189`) | rewind cleared, providers refreshed (`CommandsScriptRunner.cs:240–244`) |
| `HostAction`s the shell returns | `SwitchSession` only (`DianaShConsoleWindow.cs:135`) | `Shutdown`, `Resume`, `Step`, `LoadCore`, `SwitchSession` (`GameWindow.axaml.cs:238–240,647–657`) | discarded: it calls `Execute` (`CommandsScriptRunner.cs:419`), which returns `Submit(line).Output` alone (`DianaShInterpreter.cs:210`) |
| A breakpoint halt after `RunFrame` | `IsHaltedAtBreakpoint` named nowhere (`grep`: 0; how Mistress stops was not traced) | `:426` | `FrameRunner.cs:161` |
| The frame hand-off | `FrameHandOff.cs`: repeated rows, aspect, display lines, lending, de-duplication by frame serial | `Latest<FrameData>` with picture, width and height only (`GameWindow.axaml.cs:117`) | — |
| Screenshot | expands repeated rows (`MainWindow.Resume.cs:90`) | does not (`:536–546`) | does not (`CommandsScriptRunner.cs:267`) |

Five post-load sequences for one operation, three answers to "does this engine capture rewind", and a shell whose
`load` works in one frontend and not the other: each a defect or a latent one, and each born of the same cause. The
shell's host code is copied too: `SyncSchedulersToSessions` is identical in `Mistress/Views/DianaShConsoleWindow.cs:109`
and `Hotaru/Views/GameWindow.axaml.cs:347` (`diff`: no difference), and `BuildInterpreter`, `RegisterInitial` and
`RebuildAll` nearly so.

**Why it ranks first.** It couples every frontend to the engine SPI, and it is the thing `EmuSen_CoreAPI.md` §8–§9
exists to remove: of the frontend API's operations, the code has none of the runtime-level ones (listing sessions,
approving a core, the event stream, a runtime-owned loop, close, reset at the `ICore` level), none of the contract
layer (`EmuSen.DianaSh.Api`, `.Client`, handles and statuses, a `PublicAPI.Shipped.txt`, the §8.6 parity test), and
every gap-list piece of §10.1 marked "move" or "replace" still at its "today" location. Each feature added to a
frontend's loop meanwhile adds a fourth or fifth copy for the runtime to absorb. The fix is that page's D2–D3 and
`EmuSen_DianaSh.md`'s; this page orders the steps that make it cheaper (§7).

### 4.2 The core library depends on the presentation layer, for two constants and one record

**Mechanism.** `EmuSen.csproj:39` references `EmuSen.Serenity`. The IL walk finds one type used across that edge,
`DisplayReading` (`EmuSen/Common/FrameScheduler.cs:35,51,57,77`), and the source adds two `const`s the IL cannot show,
`DisplayShape.Television` and `DisplayShape.GameBoy` (`EmuSen.Serenity/DisplayShape.cs:6,9`), named by the five C# cores'
`DisplayAspect` (`VenusCore.cs:108`, `MoonCore.cs:76`, `MercuryCore.cs:97`, `MarsCore.cs:227`, `MarsRtCore.cs:120`).
Six of Venus's renderer files also carry a `using EmuSen.Graphics` that names nothing.

**What it drags.** Serenity takes four Avalonia packages, SkiaSharp, Shaderc, Vulkan and SQLite, and references LunaP
and DianaSh. So every assembly that takes the cores takes a UI toolkit: Pharaoh, the headless runner, whose only
declared reference is EmuSen, ships 24 Avalonia assemblies and LunaP in its build output (`ls
EmuSen.Pharaoh/bin/Debug/net10.0 | grep -ci avalonia`: 24). By file name, the packages that arrive only through
Serenity (HarfBuzz, Shaderc, ANGLE's GLES, Avalonia, LunaP) come to about 186 MB of that Debug output's 687 MB, which
carries every platform's natives; a publish for one platform carries one platform's share.

**Which rule it breaks, said exactly.** The two pages that describe the layers disagree here. `EmuSen_Multicore.md` §9
counts Serenity among the "leaf layers" that "sit below everything else and are not allowed to know a core exists";
under that picture the core may reference Serenity as it references Galaxia, and the rule is kept. `EmuSen_CoreAPI.md`
§3, decided later as the target, gives presentation to the clients ("windows, shaders, the audio device") and has the
runtime under them "never … know a UI toolkit"; under that picture the cores reaching Serenity is a reach upward. The
drawing of §2.1 uses the second picture. The harm does not depend on the choice: either way, a component with no
window takes a window toolkit, and nothing would notice the edge growing, because `LeafAssemblyTests.cs` pins what the
leaves reference and nothing of what EmuSen references.

**Why it ranks second.** It is silent, it puts a toolkit under every headless consumer, and it is the cheapest fix on
this page: `FrameScheduler` is pacing, which `EmuSen_CoreAPI.md` §8.4 already places "in the shared presentation
assembly beside the display it times against", and its only callers are Mistress and Hotaru
(`MainWindow.axaml.cs:1245`, `GameWindow.axaml.cs:392`), which reference Serenity already. A display's shape is a
console fact, which §8.2 of the same page gives to the systems table.

### 4.3 Global mutable statics are interfaces between components

**Mechanism.** By the IL walk, 38 public static mutable fields and 22 public static setters in EmuSen, 8 fields and 1
setter in Serenity, 6 setters in Galaxia, 2 in DianaSh; by `grep` over every assembly, 104 declarations, of which 38
are used and 26 written from another non-test project, and 39 used (36 written) by WiseMan. The ones that carry
traffic between components:

| Static | Declared | Written by | Read by |
|---|---|---|---|
| `GraphicsSettings` (8 fields; namespace `EmuSen.Graphics` in the Serenity assembly) | `EmuSen.Serenity/GraphicsSettings.cs:6–21` | `LoadFromDisk` from Mistress and Hotaru (`Program.cs:38`, `:48`); `SyncToDisplay` also by `PreferencesWindow.cs:276`, **together with** `GraphicsConfig.SyncToDisplay`, by hand | Serenity itself (`GameFrameControl.cs:228,256,605`; `FramePresenter.cs:72–75`); Mistress on the emulation thread each frame (`MainWindow.axaml.cs:1293`); Hotaru (`GameWindow.axaml.cs:140–143,399`). Two of the eight, `TargetFps` and `VSyncEnabled`, are loaded and saved and read by nothing |
| `AudioSettings` (7 fields) | `EmuSen/Settings/AudioSettings.cs` | `LoadFromDisk`, both frontends | Mistress and Hotaru, passed into `AudioPlayer` (`MainWindow.axaml.cs:192`, `GameWindow.axaml.cs:76`) |
| `DebugSettings` (13 fields, 20 setters) | `EmuSen/Settings/DebugSettings.cs` | Mistress's `DebugSettingsWindow.cs:22–56` (15 flags, every one read only by Venus); Hotaru (`Program.cs:180–181`); Pharaoh, by reflection from `--flag` names (`HeadlessDebugOptions.cs:185–203`) and directly (`CommandsScriptRunner.cs:362,376`) | Venus (20 files), Moon (1). `MasterLoggingEnabled` is a forward to the shell's own static, `DianaShLogging.MasterEnabled` (`DebugSettings.cs:7–11`) |
| `CoreOptions.BatteryRamDisabled` | `EmuSen/Cores/CoreOptions.cs:7` | Pharaoh (`Program.cs:193`, `TestRomRunner.cs:12`); WiseMan (94 writes) | the cores at `LoadRom` |
| Galaxia's overrides (`ConfigStore.OverrideDirectory`, `OverrideLegacyDirectory`, `DataStore.OverrideDirectory`, `ErrorLog.DirectoryOverride`, `ErrorLog.Redactor`, `ConfigDiagnostics.Sink`) | `EmuSen.Galaxia` | both frontends (`Sink`); Mistress (`Redactor`, `CrashLog.cs:15`); MarsRT's PGO driver; WiseMan (`DataStore.OverrideDirectory` 171 writes, the most-mutated static in the tests) | Galaxia |
| `LunaSettings.Store`, `.Diagnostics` | LunaP | both frontends (`Program.cs:32–33`, `:42–43`) | LunaP |

**What it costs.** Three things, of which the first is measured. *Parallel tests:* `EmuSen_Debugging_Tools_Reference_v5.md`
§3.56 records that `CoreOptions.BatteryRamDisabled` read live kept six classes in one serial collection, and that
`CpuBinaryTrace`'s process-wide buffer crashed a test that never names it once parallelism was turned on.
*Two definitions of one setting:* `GraphicsSettings` is a second copy of `GraphicsConfig`, synchronised by hand in
`PreferencesWindow.cs:276`. *A presentation layer reading globals:* `EmuSen_Multicore.md` §9 says of a device layer,
"Not a core, not a session wrapper, not a global settings static. Every knob is a parameter." Endymion keeps that
(it reads no static settings class); Serenity does not.

**Why third.** It couples every frontend to the cores' internals by a channel no type graph shows (Mistress's debug
window is a Venus window that names no Venus type), it serialises the test suite, and it has to be undone before a
session can hold its own settings (`EmuSen_CoreAPI.md` §8.2, "Settings").

### 4.4 `CoreCatalog`, and the systems table spread over three assemblies

**Mechanism.** `EmuSen/Cores/CoreCatalog.cs` is a static class of 46 methods and 30 fields over nine concerns (§3.2).
Mistress alone calls 19 of its members from 17 types; Hotaru 3 types. The console facts it serves are declared in three
places in three assemblies: the record `CoreDescriptor` in the shell's command folder
(`EmuSen.DianaSh/DianaSh/Bin/Commands/EmuSen/CoreCommand.cs:13`), its rows in `CoreCatalog` (`:12–70`) and in DianaSh's
`Sys/Systems/Snes` and `Genesis` packs, and a display's shape in Serenity (§4.2). The frontends switch on console names
besides: Mistress five times (`Input/ControllerDiagrams.cs:17–20`; `Views/ActiveCheatsWindow.axaml.cs:203`), and Hotaru
adds SNES work-RAM watches for every console, unguarded (`GameWindow.axaml.cs:137–138`). LunaP carries console nouns in
`Controls/Drawn/BadgeGlyph.cs:37–49` (`enum ControllerShape { Unknown, Gamepad, Nes, Snes, Nintendo64 }`).

**Why fourth.** It is a god object with one heavy client, and its replacement is already decided and has an oracle
(`EmuSen_CoreAPI.md` §10.2 D1; `WiseMan/Cores/CoreRegistrationEquivalenceTests.cs`, 17 facts and theories). It ranks
below §4.1–§4.3 because it is not silent: the equivalence test fails if it drifts.

### 4.5 Frontends name the C# cores' internals

**Mechanism.** By the IL walk, of the types of EmuSen each frontend names, Hotaru names 4 of Venus's
(`VenusCore`, `Venus.Memory.MemoryBus`, `Venus.Video.Ppu`, `Venus.Video.Renderer`: `GameWindow.axaml.cs:48,518,521,529`,
three debug dumps) and Pharaoh 18 of Moon's and Venus's (`MoonCore`, its `MemoryBus`, `Ppu` and `Cartridge`, the
single-step targets and test loaders of the 65816, SPC700 and 6502, the binary traces and their differs, the NES test-ROM
runner), with six `is MoonCore` tests in `Reference/PeerProbe.cs:22–59`. Mistress names none; its one `using
EmuSen.Cores.Nintendo.Venus.Debug` (`MainWindow.axaml.cs:18`) names nothing.

**Why fifth.** It is bounded: every one of these types belongs to a C# core that retires behind its Rust successor
(`EmuSen_CoreAPI.md` §9.5), and Pharaoh's validation runners are the tool those oracles are graded by. The harm is that
nothing stops the pattern reaching a Rust core: the rule "a client never names a core" (`EmuSen_CoreAPI.md` §3) has no
test.

### 4.6 `IDebugTarget` is a god interface

**Mechanism.** 45 members, 27 of them required: 29 of its own (11 required, 18 with default bodies) and 16 of
`ICoreTelemetry`'s (`EmuSen.DianaSh/DianaSh/Lib/IDebugTarget.cs:134`). It hands out eleven concrete registry types
(`WatchRegistry`, `BreakpointRegistry`, `CheatRegistry`, …) rather than operations, so a registry's every public member is
part of what every core is coupled to. Seven implementers, all in EmuSen, 2,997 lines among them; it is named in 91
of DianaSh's 142 files. Outside DianaSh, the non-test callers use three of its members (`GetMemorySpaces`, `Watches`,
`ApplyCheats`).

**Why sixth.** It is wide, but it is one assembly's dependency inversion (§2.4 C4) and it works; its replacement for the
Rust cores is already the ABI's capability-flagged debug interface (`EmuSen_CoreAPI.md` §6.14), and `CoreDebugTarget` is
the one adapter over it. What is worth deciding now is that it grows no further (§8 Q7).

### 4.7 Nothing is hidden by default

**Mechanism.** §3.1: 268 of EmuSen's 337 public types have no caller outside it but the tests; Mistress's 276 have none
at all. Where an assembly grants WiseMan friendship, its tests reach internals cleanly (§2.3); where it does not, they
reach privates by reflection. `BindingFlags.NonPublic` appears 191 times in WiseMan and 4 times elsewhere; `typeof(X).GetField/Method/Property`
257 and 5. Of WiseMan's reflection, 235 sites aim at Mistress (227 at `MainWindow`, in 72 files), reaching 106 private
members, among them `StopEmulationThread` (14 sites), `_appSettings` (12), `_session` (11). Serenity's internals are looked up
by name in five places although Serenity already names WiseMan its friend (`WiseMan/Serenity/ScreenFilterChainTests.cs:288`,
`ScreenFilterRenderTests.cs:75`, `ScreenFilterRenderTests.Crt.cs:126`, `ShaderBench.cs:94`). In shipped code, one site
reaches into another assembly's privates, Avalonia's (`EmuSen.Serenity/RenderTimerRate.cs:9–36`, `SleepLoopRenderTimer.DesiredFps`).

**Why seventh.** It is the reason no component has a surface to speak of; but on its own it breaks nothing, and a
rename of a private field breaks a test at run time rather than a build.

### 4.8 Duplicated contracts

**Method.** `grep` for each contract's definitions, and a clone check over the sources: whitespace normalised, comments,
`use` lines and brace-only lines stripped, every window of N lines hashed, and a window counted when it appears in more
than one crate (Rust) or core folder (C#); function bodies of six lines or more hashed whole. **Deliberate** below means
documented and held equal by a test: the C# halves of the platform kept as oracles (26 `Managed` twins, 69
native-or-managed branches, six parity classes), the C# cores kept as the oracles of MoonRT, MercuryRT and MarsRT, the
pre-stable ABI beside v1 until `EmuSen_CoreAPI.md` §13.5 deletes it, and the state codec's two halves, which must read each
other's files. None of those is a defect; each has a date on which it ends. What follows is the rest.

| Contract | Definitions | Held equal by | Evidence |
|---|---:|---|---|
| Which bit is which button | **19**: 4 of the canonical sixteen (`PadButton.cs:4`, `emusen_core.h:142`, `desc.rs:11` with a second list of its names at `:32`, the platform's port `pad.rs:7–15`); 13 per-console hardware tables over five consoles; 2 default SDL maps | about 6 by a test; the C# names against Rust `Control::name` by nothing but the descriptor's join by name (`CoreDescriptors.cs:128`); the N64's table copied byte for byte into the MarsRT shim (`MarsCore.cs:31–34,317`, `MarsRtCore.cs:63–66,551`; `diff`: the ten switch lines identical) by nothing | `Endymion/Input/GamepadBindingMap.cs:22–38` and `Mistress/Views/InputSettingsWindow.Diagram.cs:285–291` write the default map twice |
| An audio or graphics setting | 3 each, with defaults: the static class, Galaxia's model, the Rust model (`AudioSettings.cs:8–21`, `AudioConfig.cs:8–14`, `models.rs:163–173`; `GraphicsSettings.cs:8–21`, `GraphicsConfig.cs:9–15`, `models.rs:177–190`) | model against Rust model by a parity test; **static against model by nothing** | 11 hand-written mappings between them |
| The core's audio rate | C# Venus answers the player's device rate as its own (`VenusCore.cs:384`, `AudioSettings.SampleRate`); VenusRT fixes 32,000 and refuses the key (`machine.rs:15`, `ffi.rs:281`) | — | one key, two meanings by engine |
| A ROM extension | up to 6 per console (the Game Boy's: `CoreCatalog.cs:36,180`, `CoreFactory.cs:47`, `BatterySave.cs:27`, MercuryRT's `v1.rs:64`, `Pharaoh/Reference/LibraryIndexer.cs:41`) | — | 12 places list consoles or cores (`CoreCatalog`'s six tables, `CoreFactory`'s switches, `ControllerPorts.cs:39–43`, `RustCores.props`, `rust-cores.yml`, the system packs, each v1 core's own descriptor) |
| An engine's name | 3 (`CoreCatalog.cs:313–337`, `RustCores.props` `<Fallback>`, each core's `display_name`) | — | |
| SNES colour, 5 to 8 bits | 3 formulas: `<< 3` (`Ppu.Registers.cs:178–180`), `* 255 / 31` (`SnesDebugTarget.cs:911–913`), `(c << 3) \| (c >> 2)` (VenusRT `ppu.rs:633`) | — | the renderer and the debugger of one core disagree; whether visibly is **not measured** |
| A save slot's file name | 3: the C# writer (`SaveLibrary.cs:9–55`), its Rust port (`saves.rs:6–42`, deliberate), a hand-written inverse parser in `Mistress/Library/MediaLibrary.cs:44–46` | the first two by parity; the parser by nothing | |
| Rust glue copied where `emusen-native` could hold it | 6 `StateError` enums with their `impl Status` and `From<Truncated>` (`grep -rn "enum StateError"`: MarsRT, MercuryRT, MoonRT, VenusRT, Nephrite, the test core); the load/size/save trio in three `machine.rs`; MarsRT's own debug hooks (`cpu/hooks.rs:8–30` against `emusen-native/src/debug.rs:40–60`, 98 lines) and its own sample queue (`memory/ai.rs:29`); two single-step runners in Beryl and a third in VenusRT | — | the clone check's 10-line windows: 354 lines in 13 files, 178 of them MercuryRT against MoonRT |
| Literals in place of a constant | 44,100 at 20 sites; 128,000 at 4; 48,000 F32 at 3; `"EmuSen"` and `"Logs"` spelled in `ErrorLog.cs:75`; Mercury's model key twice in one crate (`ffi/mod.rs:30`, `ffi/v1.rs:31–32`) | — | |

**C# cores.** No 20-line window repeats between C# core folders. At 10 lines, 188 lines in 6 files: Mars against its
shim (the shim mirroring its oracle), and Mercury's debug-target wiring against Moon's (`MercuryDebugTarget.cs:78,139`
against `MoonDebugTarget.cs:79,110`), which a shared base for the seven `IDebugTarget` implementers would absorb. Two
CRC-32 classes are hand-written in the frontends (`Mistress/Scraping/RomHashes.cs:36`, `Pharaoh/Reference/SignatureWriter.cs:57`),
and LunaP's menu colours are pasted into `Mistress/Views/MainWindow.axaml:184–187`.

**The shape of it.** There is no large copied subsystem; the sharing rule held for code. What it did not reach is
**facts**: a button order, an extension, a rate, a name, a colour formula, written as a literal where a constant or a
descriptor exists. Facts are the systems table's and the descriptors' (`EmuSen_CoreAPI.md` §6.3, §8.2), and most of
these rows retire with `CoreCatalog` (§4.4) and the C# cores. The ones that do not are the Rust glue, which is
`emusen-native`'s to absorb, and the settings' third copy, which §7 S3 removes.

**Why eighth.** The dangerous rows are held by nothing, but each is small and local, and the deliberate pairs, which
are most of the duplicated volume, are the best-guarded code in the project.

### 4.9 Narrow consumers take whole assemblies

Serenity references DianaSh for 6 types (28 IL references), all telemetry, for `Dashboards/CoretopWindow.cs`, and so
takes the interpreter, the 48 files of its core commands and the sandbox. EmuSen takes DianaSh for 48 types. Five edges are used
and not declared (§2.1): every frontend's reach into DianaSh, and Hotaru's and Pharaoh's into Galaxia. This ranks low
because none of it closes a cycle or reaches a toolkit (DianaSh has no package at all); it is listed so that
`EmuSen_DianaSh.md` can weigh it (§6.3).

### 4.10 Environment variables and files as unrecorded interfaces

**Environment variables.** 226 distinct names are read (153 of them `EMUSEN_*`), at 394 sites (251 C#, 143 Rust); 49
change what shipped code does. The seven `*_NATIVE` switches are a documented interface (`EmuSen_RustPlatform.md`
§3.9). The rest are per-core tuning and diagnostic flags, three of them shipped and documented nowhere
(`EMUSEN_SYNC_MARGIN_MS`, `MainWindow.axaml.cs:1246`; `EMUSEN_MARS_NOINLINEJUMPS`; `EMUSEN_MARSRT_PERFMAP`, which writes
`/tmp/perf-<pid>.map`). One knob is read by two cores, `EMUSEN_MARS_GPU_DEVICE` (`Mars - N64/Rdp/Gpu/GpuDevice.cs:101`,
`MarsRT - N64/src/rdp/gpu/device.rs:195`); *since 2026-10-13 it is `EMUSEN_GPU_DEVICE`, read by the C# class and by
`emusen-gpu`, the shared crate MarsRT's device moved into (`EmuSen_Gpu.md` §4); since 2026-10-16 the crate alone reads it, the C# class deleted with Mars (C#)*. **No shipped component sets a variable another reads** (0 found); every
set-here-read-there pair is a test driving a child process. This is weaker coupling than it looks, and ranks low.

**Files.** Four contracts cross components with no shared type:

| File | Writers and readers | Defect |
|---|---|---|
| `*.core.json`, a v1 core's sidecar | written by hand in Rust (`emusen-core-conform/src/sidecar.rs:41`); read as untyped JSON in C# (`CoreDiscovery.cs:17,53,137`, `CoreDescriptors.cs:92–126`) | the schema is in two places and neither is a type |
| crash files | four writers, three names: `crash_<time>.txt` (`Mistress/CrashLog.cs:27`), `<id>_crash_<pid>.txt` (`Cores/Native/CoreLibrary.cs:117`), `<crate>_crash_<pid>.txt` (`NativeCoreLibrary.cs:26`), `<name>_crash_<pid>.txt` (`Galaxia/Native/PlatformLibrary.cs:50`) | the log pruner matches `emusen_*.log` only (`Galaxia/Library/ErrorLog.cs:13,141`; `Platform/galaxia/src/error_log.rs`), so crash files are never pruned |
| the battery suffix `.srm` | each Rust core's descriptor (`MercuryRT - GB/src/ffi/v1.rs:137`, `MoonRT - NES/src/ffi/v1.rs:119`, `VenusRT - SNES/src/v1.rs:307`, `Nephrite - Genesis/src/v1.rs:346`), defaulted again in `CoreDescriptors.cs:164`, and in Galaxia twice (`SaveLibrary.cs:9`, `Platform/galaxia/src/saves.rs:6`) | the core says, as the ABI intends; the duplicate default is the host's |
| config files Galaxia does not list | `gamepadbindings.json` (`Endymion/Input/GamepadBindings.cs:69`), `keybindings.json` and `hotkeybindings.json` (Mistress), LunaP's `luna.json`, `windows.json`, `panes.json`, `tables.json`, which both frontends point at the same directory; seven SQLite files owned outside Galaxia | `EmuSen_Galaxia.md` calls Galaxia "the one answer to where a file lives"; for these it is not |

Twelve shipped `Path.Combine` calls build a path under a Galaxia root outside Galaxia (for example
`Mistress/Library/GameRecords.cs:95`, `Pharaoh/Reference/LibraryIndexer.cs:30`, `Serenity/Slang/SpirvCache.cs:20`).

### 4.11 LunaP is reached by an absolute path

`LunaP.props:11` is a `ProjectReference` to `/home/red/Projects/EmuSen.LunaP/src/EmuSen.LunaP/EmuSen.LunaP.csproj`. The
build therefore works on one machine's layout, and every checkout of EmuSen builds against whatever that one checkout
of LunaP has checked out, with no commit recorded: on 2026-10-10, `openemu-library` at `e098b08`, no tag. The file's
own comment says it returns to one `PackageReference` when a LunaP release carries the library controls
(`EmuSen_LunaP.md` §9). The toolkit's own layering rule, "Avalonia and nothing else" (LunaP's `docs/LunaP.md` §1 and
§16), **is kept**: its one `EmuSen.` mention is a comment (`Settings/JsonSettingsStore.cs:12`), and
`LeafAssemblyTests.cs:47` and LunaP's own `LayeringTests.cs` pin it. Of its 161 public type names, Mistress names 108,
WiseMan 95, Hotaru 15 and Serenity 12; 42 are named by none of the four, which overstates the unused set, since
extension classes are used without being named.

One index fact found on the way: the README's line for `EmuSen_LunaP.md` points readers to its §1, §12 and §16 for the
layering rule and themes; since the toolkit left the repository those sections are in LunaP's own `docs/LunaP.md`, and
the page here has nine sections.

---

## 5. What is already clean (measured)

An audit that reports only defects misleads about proportion. These boundaries were measured and found sound, and §6
builds on them rather than replacing them:

- **The Rust crate graph** (§2.2): acyclic, two levels deep, every core on `emusen-native`, the Sega cores' processors in
  Beryl. The sharing rule held for code: no 20-line window repeats between C# core folders, and between Rust crates only
  92 lines do, all MercuryRT against MoonRT glue (§4.8).
- **`InternalsVisibleTo`** (§2.3): three friends, all the test harness, all for the parity classes they name.
- **Galaxia's and Endymion's public surfaces** (§3.1): 29 of 30 and 11 of 16 public types used by another component.
  Galaxia's leaf rule is kept to the letter and serves its purpose.
- **LunaP's layering rule** (§4.11): kept, and pinned on both sides of the repository boundary.
- **The shell has no toolkit and no core**: DianaSh references Galaxia alone and takes no package; nothing in it names
  Avalonia, Mistress or Hotaru except man pages and comments.
- **No shipped component passes an environment variable to another** (§4.10).
- **The two written surfaces**, the core ABI v1 and the platform library's C interface, which this page did not audit
  and against which it found nothing.

---

## 6. The defined surfaces proposed

### 6.1 Six stability rules, five of them already the project's

A surface is defined when three things are written down: what crosses it, what does not, and what a change to it
costs. Every component below is given one of six rules. Five reuse rules the project already keeps; one, D, is new
and is the smallest addition that makes a C# assembly's surface visible without promising anything.

| Rule | What it promises | Guard | Source |
|---|---|---|---|
| **A. Stable C ABI** | a major that is never meant to change; additive minors; deprecation instead of removal; skew in both directions handled | the header, the Rust and the C# checked against a recorded baseline in CI | `EmuSen_CoreAPI.md` §4, §5 |
| **B. Exact C interface** | nothing across releases; the library and its only client are built from one commit; one number matched exactly | an export check when the library loads | `EmuSen_RustPlatform.md` §3.2 |
| **C. Versioned C# API** | semantic versions; additive minors; `[Obsolete]` before removal; no new abstract member on an interface a client implements | the public-API analyzer's `PublicAPI.Shipped.txt`, failing a pull request on a removed or changed line | `EmuSen_CoreAPI.md` §9.3, for the frontend API's binding |
| **D. Recorded internal C# surface** *(new)* | **nothing** across commits; but the public surface is a file in the repository, so a type or member that becomes public, or stops being, is a reviewed line in a diff; and the assembly's reference set is pinned | a baseline file per assembly, written and compared by S0's reader rather than the analyzer package (§8 Q3); `LeafAssemblyTests` for the references | argued here |
| **E. Internal** | nothing | `internal`; the test harness reaches it through `InternalsVisibleTo`, never reflection | — |
| **R. Internal Rust API** | nothing; a path dependency, built from the same commit | the ABI checks of the crates that consume it | `EmuSen_RustState.md` §2 |

Two rules from the existing pages carry across unchanged and are stated once here so that the table below can lean
on them: **pull, never push** (no surface calls back into its client; a delegate becomes a drained queue,
`EmuSen_RustPlatform.md` §3.7), and **the hot path crosses no text or file layer** (`EmuSen_CoreAPI.md` §2.5).

### 6.2 Per component

| Component | Exposes | Hides | Rule |
|---|---|---|---|
| **Rust cores** (MarsRT, MercuryRT, MoonRT, VenusRT, Nephrite) | the core ABI v1 and nothing else; their `rlib` API to their own tests | everything | **A**. MarsRT is the one still on its legacy ABI, moving at its step 5 (`EmuSen_CoreAPI.md` §13.3 item 7); the pre-stable `emusen_native_*` set retires when the others need it no longer (§13.5) |
| **`emusen-native`** | to cores: the state codec, `core_exports!`, the debug hooks, the crash-log hook, the descriptor writer, the test suites' JSON reader, since 1.3 `Kept` and `keep` (`EmuSen_CoreAPI.md` §6.24), and since 1.4 `seek` (§6.25) | — | **R**. Its consumers are the five cores, the four Beryl crates, the kit's two tools and the platform library (`abi::install_crash_log` only) |
| **`emusen-disc`** | to disc cores, the kit and the platform library: a disc as sectors over any backing, opening an image with its companions, the fingerprint, the set readers, and since 2026-10-12 which file and byte hold a sector (`EmuSen_Disc.md`) | its readers' internals | **R**. Depends on the `chd` crate alone, never on `emusen-native` or Galaxia |
| **`emusen-gpu`** *(2026-10-13)* | to cores that draw on a GPU: one Vulkan compute device, chosen by rank or by name (`EMUSEN_GPU_DEVICE`), storage buffers, compute programs, and submissions waited for, pending or ahead; surface 1.0, additions as minors (`EmuSen_Gpu.md` §3) | every Vulkan type: no `ash` handle crosses it until a lent device's constructor (§5 of that page) | **R**. Depends on `ash` 0.38 alone, the version wgpu-hal uses; its users are MarsRT and, from its stage E3, Diamond |
| **Beryl** | to Sega cores: the four processors | their internals | **R**, with the contracts its `README.md` already states |
| **The platform library** | `emusen_platform.h`, interface 10 since 2026-10-12 (`emusen_galaxia_mapped_advise`, `EmuSen_RustPlatform.md` §24) | the crates' Rust API to everything but a later Rust DianaSh | **B**, as built |
| **Galaxia** | where things live (`ConfigRoot`, `ConfigStore`, `DataStore`, `SaveLibrary`), how they are written (`ConfigFile<T>`, `AtomicFile`, `ErrorLog`), the plain models, `PadButton`/`PadAxis`/`PadControl`, `RomHash`, `ICatalogue` — **and, newly, every file name under its roots and the one crash-log rule** (§7, S8); since 2026-10-12 a disc's files, `Media/` (`ImageSets`, `ArchiveImages`, `MappedFile`, `DiscFiles`, `DiscSet`; `EmuSen_Galaxia.md` §3.8), moved from the runtime | `Native/`, every `Managed` half | **D**. Its one test seam, the directory overrides, becomes one scoped override rather than four global setters (argued; §4.3) |
| **Endymion** | `AudioPlayer`, `UiSoundPlayer`, `DynamicRateControl`, `GamepadManager`, `GamepadBindingMap`, `GamepadBindings`, `PortRouter`, `PlayerSlots`, `ConnectedPad`, `DefaultPadKeyMap` | `Native/`, the `Managed` halves, `SimulatedPad(s)` and `SdlPadDevices` (WiseMan is their only caller) | **D**. Already nearly defined: 11 of 16 public types have a non-test caller (§3.1) |
| **Serenity** | `GameFrameControl`, `FramePresenter`, `DisplayClock` **and the pacer that reads it** (`FrameScheduler`, moved in, §7 S1), the filter choices and their parameters for a settings window, `PictureCrop`, `CoretopWindow`; since 2026-10-17 `PictureSlots` and `CoretopWindow.Over(ISessionReadout)` with `SessionTelemetry`, for a frontend on the frontend API (`EmuSen_Serenity.md` §2.11; decided by the tester); since 2026-10-18 `FrameImageWriter` (moved from Hotaru, for Mistress's screenshots too) and `GameFrameControl.LendableDevice` in place of `TryLend`, the presenter's device for the runtime to lend (`EmuSen_Backend.md` §14.13) | the slang chain, its compiler, reflection, cache and Vulkan device; `BuiltInShaders`; `FilterChain` | **D**. Every knob a parameter, as `EmuSen_Multicore.md` §9 already requires: no static settings class (§7 S3) |
| **EmuSen** (the C# cores) | one registration per managed engine, and its engine-SPI implementation, as `EmuSen_CoreAPI.md` §9.5 already decides | every console's internals | the engine SPI is public with no promise, in an `Engines` namespace outside every baseline (`EmuSen_CoreAPI.md` §9.4); everything else **E**. References DianaSh and Galaxia, nothing else (§7 S1) |
| **DianaSh** | **its target is `EmuSen_DianaSh.md`'s** (§6.3); since 2026-10-17 `HostCommand`, `CoreChoice`, `CreateForService` with sessions, `CreateWithoutSession` and `CoretopCommand(Action)`, which name no runtime type, for a frontend under SR2 (`EmuSen_DianaSh.md` §13.14; decided by the tester) | — | that page's |
| **The frontend API's binding** (`EmuSen.Crystal.Client`) | the binding's classes over `emusen_crystal.h`; and one key a program carries on its own assembly, `[assembly: AssemblyMetadata("EmuSen.CrystalNative", "1")]`, written by the MSBuild property `EmuSenCrystalNative` set to `true` in the program's project (`Directory.Build.targets`). The binding reads it from the entry assembly when `EMUSEN_CRYSTAL_NATIVE` is unset, and it makes the Rust Crystal that program's default; the variable's `1` or `0` still decides either way, and any other value is the C# (decided 2026-10-18, `EmuSen_Backend.md` §29.11 Q3; built in §29.27) | the contract layer, and both implementations behind it | **C** |
| **Mistress, Hotaru, emusen-shell, Pharaoh** | nothing; they are clients. Pharaoh since B2's part 2, Hotaru and emusen-shell since part 3 and Mistress since part 4b name no runtime type (`FrontendReferenceTests`, `EmuSen_Backend.md` §14.11) | all their own types | **E**, with `InternalsVisibleTo` to WiseMan in place of reflection (§8 Q4). As clients they hold the frontend API and the presentation and device layers, never an `ICore` or an engine type (`EmuSen_CoreAPI.md` §3) |
| **LunaP** | its toolkit | its internals | **C**, as it already is in practice: it is a package with an outside consumer. Reached from a package, not an absolute path, once a release carries what Mistress needs (§4.11) |
| **WiseMan** | nothing; no shipped assembly references it | — | may be the friend of any assembly; must not need reflection into one that grants it friendship |

### 6.3 Where this page meets `EmuSen_DianaSh.md`

The shell's target surface is that page's decision. What this audit found that the design has to answer, without
choosing the answer:

1. **Narrow consumers take the whole assembly.** Serenity names 6 of DianaSh's 217 public types, all telemetry, and
   EmuSen names 48, mostly the debugger's registries and `IDebugTarget`; each takes the interpreter, the commands and
   the sandbox with them (§4.9). Whether the contracts a non-shell component consumes are separated by namespace and
   baseline or by assembly is the design's choice. `EmuSen_Multicore.md` §9.2's pruning test and the 2026-10-07
   decision to fold Cauldron into DianaSh both bear on it, and this page does not reopen the second.
2. **The systems table is filed as a command** (`CoreDescriptor`, `Bin/Commands/EmuSen/CoreCommand.cs:13`) and its
   rows are filled in the core assembly (`CoreCatalog.cs:12–70`). `EmuSen_CoreAPI.md` §8.2 makes the table the
   runtime's; it wants a home that is data.
3. **`IDebugTarget` is one interface every engine implements whole** (§3.2), where `EmuSen_CoreAPI.md` §6.14 offers the
   debugger in capability-flagged parts.
4. **`HostAction` is handled by each host its own way** (§4.1): the frontend API's session calls replace it.
5. **A global in the shell carries a core's setting**: `DebugSettings.MasterLoggingEnabled` forwards to
   `DianaShLogging.MasterEnabled` (`EmuSen/Settings/DebugSettings.cs:7–11`).

---

## 7. The migration order

Each step leaves the solution building and every client working, and each has a gate that fails before the step and
passes after it, or holds a recorded number. Steps marked **(theirs)** are already planned elsewhere and are placed here
only so that the order is whole. The order puts the cheap and silent problems first (S1, S3), the planned runtime work
where its pages put it (S5, S6), and the visibility work after the code it would otherwise have to be done twice for (S7).

| Step | What | Gate | Ranked problem |
|---|---|---|---|
| **S0** | **Record the surfaces.** The reader of §1 becomes a WiseMan tool test that writes, per assembly, the public types, the non-test outside callers and the edges of §2.1 to a file in the repository, and fails when a reference set changes without that file changing. `LeafAssemblyTests` gains EmuSen's own reference set as it stands today, `{ DianaSh, Galaxia, Serenity }`. *Revised 2026-10-11: that pin was added by `EmuSen_DianaSh.md`'s S2 as `{ Crystal, Galaxia, Serenity }` (§2.1's note)* | the test passes on `96c3f563`'s numbers; a later step that changes them updates the file in the same commit | all; it turns §3's table into something a diff shows |
| **S1** | **Cut EmuSen → Serenity.** `FrameScheduler` and `PacingDecision` move into Serenity beside `DisplayClock` (`EmuSen_CoreAPI.md` §8.4 already places pacing there); `DisplayShape`'s two constants move to the systems table in DianaSh (§8 Q2; *overtaken 2026-10-17: the shape is machine info's, and the constants went to Crystal's assembly, §7.1*); Venus's six dead `using EmuSen.Graphics` go; the `ProjectReference` goes | S0's pin becomes `{ DianaSh, Galaxia }` (*since 2026-10-11, `{ Crystal, Galaxia }`, §2.1's note*); `ls EmuSen.Pharaoh/bin/Debug/net10.0 \| grep -ci avalonia` goes from 24 to 0; the pacing and display-aspect tests pass unchanged. Predicted, **argued**: Pharaoh's Debug output loses about 186 MB. *Built 2026-10-17; the gates as measured are §7.1's* | §4.2 |
| **S2** | **Declare what is used.** The five transitive edges of §2.1 become `ProjectReference`s | builds; S0's file unchanged except for the declared column | §4.9 |
| **S3** | **Settings are parameters, not globals.** `GameFrameControl` and `FramePresenter` take the window size, title, resizing, filtering and display sync as properties the frontend sets from `GraphicsConfig`; `GraphicsSettings` is deleted, and with it the hand synchronisation of `PreferencesWindow.cs:276` and the two knobs nothing reads (§8 Q6). The frontends read `AudioConfig` for `AudioPlayer`'s arguments, and Venus's audio rate stops being the device's (`VenusCore.cs:384`). `CoreOptions.BatteryRamDisabled` becomes a create option. `DebugSettings` waits for `EmuSen_DianaSh.md` (§6.3 item 5) | `grep` for `GraphicsSettings\.` in shipped code: 0 (today 17 lines outside its own file, comments excluded), and for `AudioSettings\.` outside the EmuSen assembly: 0 (today 4 lines); the settings, preferences and battery-latch tests pass; one of the suite's serial collections can be removed (`EmuSen_Debugging_Tools_Reference_v5.md` §3.56). *Built 2026-10-17; the gates as measured are §7.1's* | §4.3, §4.8 |
| **S4** | **A client names no console's internals, by test.** A WiseMan metadata test fails when Mistress, Hotaru or Pharaoh names a type under `EmuSen.Cores.Nintendo.*` or `EmuSen.Cores.Sega.*`, with an allow-list holding today's 4 (Hotaru) and 18 (Pharaoh) that may only shrink. Hotaru's three Venus debug dumps become shell commands over `IDebugTarget` | the test; the allow-list's length recorded | §4.5 |
| **S5** **(theirs)** | `EmuSen_CoreAPI.md` §10.2 **D1**: the engine SPI, the native host and the systems table into `DianaSh/Sys/`; `CoreCatalog` and `CoreFactory` answered from the runtime. One addition from this audit: the registration facts of §4.8's table (extensions, engine names, button orders) each get one home there. *Revised 2026-10-10, by decision (`EmuSen_DianaSh.md` §16 Q1, §17.2): "into `DianaSh/Sys/`" is read as "into the runtime's own assembly", Crystal's, beneath the shell and beside the cores* | the registration-equivalence test (`CoreRegistrationEquivalenceTests.cs`); the extension and engine-name counts of §4.8 at one each | §4.4, §4.8 |
| **S6** **(theirs)** | **D2–D3**, and `EmuSen_DianaSh.md`: sessions, the one frame loop, the services, the frontend API's contract and binding; Pharaoh first, then the shell, Hotaru, Mistress | that page's gates; and from this audit, the divergences of §4.1's table each closed by one test that runs the operation through every client (five post-load sequences become one; rewind capture asks the engine in every client) | §4.1 |
| **S7** | **Hide by default.** With the C# cores reduced to registrations (S5), EmuSen's console internals become `internal`, reached by WiseMan through `InternalsVisibleTo`. Mistress, Hotaru and Pharaoh grant WiseMan friendship and their types become `internal` file by file, each file's reflection replaced as it goes (§8 Q4). Galaxia, Endymion and Serenity take rule D's baseline (§8 Q3); Endymion's simulated pads and Serenity's internals already used only by tests become `internal` | S0's counts: EmuSen's public types without a non-test caller from 268 toward the engine SPI's; `BindingFlags.NonPublic` in WiseMan from 191 toward the Avalonia seams alone; the three baselines exist and a removed line fails a build | §4.7 |
| **S8** | **One answer to where a file lives, kept.** Galaxia lists every file under its roots, including the three binding files, LunaP's four stores (by name; LunaP stays unaware) and the seven databases; one crash-file rule, named by Galaxia and used by the four writers, inside the pruner's reach; the sidecar given a C# record mirroring `sidecar.rs`, with a round-trip test over a written sidecar | a test that every shipped `Path.Combine` under a Galaxia root names a Galaxia constant (today 12 do not); a crash file of each writer pruned on schedule; the sidecar round trip | §4.10 |
| **S9** | **The switches, recorded.** One table in Galaxia of every environment variable that changes shipped behaviour, each with its page; a test that every `EMUSEN_*` name read in shipped code is in it and documented | the test; today 49 names, 3 undocumented | §4.10 |
| **S10** | **LunaP from a package**, the day a release carries what Mistress needs, as `LunaP.props` already says | `LunaP.props` holds one `PackageReference`; a clean clone on another machine builds | §4.11 |

**Why this order and not another.** S1 before S3, because S3 changes `GameFrameControl`'s inputs and S1 moves its
pacing neighbour; doing S3 first would touch `FrameScheduler` in the wrong assembly. S4 before S6, because the
allow-list tells D2 which of Pharaoh's verbs are still console-shaped. S7 after S5, because hiding EmuSen's types before
the engine SPI moves would mean deciding twice which of them are the SPI. S8 and S9 are independent of the rest and can
run beside any step.

### 7.1 S1 and S3, as built (measured 2026-10-17)

Both were built on 2026-10-17 on top of `WiseMan` at `69974969`, as step B0 of `EmuSen_Backend.md` §6. Neither changes
what a player sees. The figures above were measured at `96c3f563`; where the code had moved since, the corrections are
recorded here rather than by rewriting the measured text.

**S1, the edge cut.**

- `FrameScheduler`, `PacingDecision` and `PacingMode` are `EmuSen.Serenity/FrameScheduler.cs`, beside `DisplayClock`.
  The pacer named the core assembly's `SpeedController`, which presentation cannot reach, so `Next` and `Interval` take
  the two values they read from it, whether the speed is 100% and the frame's interval at it; the frontends pass
  `_speed.IsNormalSpeed` and `_speed.FrameInterval(hz)`, and the tests reach the same call through a three-line
  adapter in WiseMan (`FrameSchedulerSpeed.cs`), so their bodies are unchanged.
- `DisplayShape` is `EmuSen.Crystal/DisplayShape.cs`, in Crystal's assembly and outside the systems table's namespace;
  the cores' shims and Serenity's CRT filter both read it there. **The step's own words are corrected here:** "to the
  systems table" (this row and §8 Q2) was overtaken on 2026-10-17 by `EmuSen_Backend.md` §5.3 and its fact F26, decided
  under D6, which give a display's shape to the core's machine info, per frame. The v1 cores already report it there;
  what the constants still serve is the shims' answer before a machine exists (`DisplayAspectTests` asks a core with no
  game) and the CRT filter's tube. Moving the shims onto machine info alone would change that answer, so B0 leaves the
  constants in the plainest place both assemblies reach, and they go with the shims at B4. `SystemEntry` gained
  nothing, and `PsxSystem.DisplayAspect`, the pack's second copy that §5.3 retires, is left for that step.
- The two Serenity types the cores handed through, which §4.2's IL walk did not count because they arrived after it
  (`EmuSen_Backend.md` §4.5 counted ten files), now cross in forms the core assembly owns: the device lend as
  `IDeviceLend`, which is the core ABI's `emusen_device_lend` (`CoreInterface.DeviceLend`) with `Take` and `Settle`,
  and the raw picture as `CoreRawPicture`. Mistress, which holds both sides, wraps Serenity's `SlangLend` in
  `SlangDeviceLend` and turns a `CoreRawPicture` into Serenity's `RawPicture` (`EmuSen.Mistress/CorePictures.cs`);
  `EmuSen_DevicePicture.md` §17 and `EmuSen_CRT.md` §22 say so.
- §4.2's "five C# cores' `DisplayAspect`" were three shims by this date (MoonRT, MercuryRT, MarsRT), and its six dead
  `using EmuSen.Graphics` lines had gone with C# Venus: none was left in the core assembly to remove.
- The `ProjectReference` is gone; `EmuSen.csproj` references Crystal and Galaxia.

| Gate | Before | After |
|---|---|---|
| `LeafAssemblyTests`: the core assembly's EmuSen references | `{ Crystal, Galaxia, Serenity }` | `{ Crystal, Galaxia }`, pinned (`The_core_assembly_references_Crystal_and_Galaxia_only`) |
| `ls EmuSen.Pharaoh/bin/Debug/net10.0 \| grep -ci avalonia`, a clean build with `-p:EmuSenNative=false` | 24 | 0 |
| Pharaoh's Debug output, the same build (`du -sm`) | 686 MB | 492 MB |
| The pacing and display-aspect tests | pass | pass, with only their `using` lines and the adapter's call form changed |

**A defect the cut made, found and closed before it was committed.** SkiaSharp's Linux library is a package of its own,
`SkiaSharp.NativeAssets.Linux`, which the core assembly never declared: it arrived through Avalonia. With the edge cut,
Pharaoh's output had SkiaSharp's managed assembly and its Windows and macOS libraries but no `libSkiaSharp.so`, so its
recorder's PNG writer (`PngFile`, through `record on`) would have failed on Linux at its first frame. WiseMan could not
see it, since WiseMan takes Serenity itself. `PharaohDependencyTests` reads Pharaoh's dependency file and failed on
`linux-x64` and `linux-arm64`; the core assembly now declares the package, and the test passes. It also pins the gate
above: no Avalonia library and no Serenity in Pharaoh's dependencies.

**The prediction, retired.** "Pharaoh's Debug output loses about 186 MB": it lost 194 MB (686 to 492), within the
estimate's error. A first build without the Linux Skia library had lost 342 MB; the difference, 149 MB, is that library's
builds for thirteen Linux targets, which the estimate had rightly counted as the core assembly's own.

**S3, settings as parameters.**

- `GraphicsSettings` is deleted. `GameFrameControl` takes `BilinearFiltering` and `SyncToDisplay` as properties, and
  `FramePresenter` forwards those two and the window's `WindowTitle`, `WindowWidth`, `WindowHeight` and
  `WindowResizable`. Hotaru's window reads `graphics.json` for its size (`GraphicsConfig.WindowSize()`, the static's
  clamp) and resizing and sets its picture's two properties; its title is the window's own "EmuSen", which is all
  `WindowTitle` ever held, since the field was never loaded from the file. `TargetFps` and `VSyncEnabled` lose their
  fields and keep their keys (§8 Q6).
- **Display sync has one owner at run time**, the picture's control. Mistress sets `GameFrame.SyncToDisplay` from
  `graphics.json` at start; Preferences writes the file through the main window's `GraphicsConfig` and tells the
  window, which sets the control (`PreferencesWindow.SyncToDisplayChanged`); the emulation thread reads the control
  each frame and hands the value to the pacer. The field is `volatile`, so the UI thread's write is seen by the
  emulation thread's next read without a lock. The hand synchronisation of `PreferencesWindow.cs:276`, two stores
  written together, is gone: the file is the setting, the control is the setting applied.
- `AudioSettings` is deleted, not only kept out of the frontends: with C# Venus gone, the fields the cores still read
  were the buffer limit and a fallback rate, and both have a better owner. The frontends read `audio.json` themselves
  (`AudioConfig.Load().Bounded()`, holding the static's clamps) for `AudioPlayer`'s three arguments. The buffer limit
  became the second field of the create option below, `AudioBufferMaxSamples`, which Mistress and Hotaru take from the
  same file (`CoreOptions.From`) and Pharaoh, which never loaded it, leaves at its default. A session with no core
  reports `AudioConfig.DefaultSampleRate`. `SampleQueue`, which no shipped core uses any more, keeps its default limit
  where it used to follow the static.
- Both programs seed the two files on the first run with `SeedIfMissing()` on each model, as `LoadFromDisk` did.
- **`CoreOptions` is a create option**: an immutable record a core holds (`CoreEngine.Options`, set before
  `LoadRom`), passed through `CoreFactory`'s five create and load calls and `EmulatorSession.Options`, in place of a
  process-wide static (`EmuSen_Multicore.md` §6.1). Pharaoh's `--nobattery` and its test-ROM runner make their cores
  with it; MarsRT's `batteryRamDisabled` constructor parameter, a second way of saying the same thing, went with the
  static. In the suite, the 60 files that set the static now make their cores with `TestCores.NoBattery`, or, for the
  four tests about the battery itself, with the battery on, and the Mistress windows they open are given the same
  option before they load (`TestCores.WithoutBattery`).
- `DebugSettings` is unchanged, as the step says.
- **Not done: removing a serial collection**, the last clause of S3's gate. Most classes that set the static also set
  another process-wide one (`DataStore.OverrideDirectory`, `FirmwareLibrary.Directory`, the discovery switches) and
  stay serial for it; a few no longer assign any by `grep` (`BreakpointConditionTests`, `MercuryModelTests`,
  `MoonGoldenTests`, `MoonRtRawPictureTests` among them), but whether each can leave needs the per-class audit of
  `EmuSen_Debugging_Tools_Reference_v5.md` §3.57, which this step did not make.

| Gate | Before (`69974969`) | After |
|---|---|---|
| `grep` for `GraphicsSettings\.` in shipped code, comments excluded | 20 lines (21 with the one comment; §7's 17 was measured at `96c3f563`) | 0 |
| `grep` for `AudioSettings\.` outside the EmuSen assembly | 4 | 0 (and 0 inside it: the class is gone) |
| `grep` for `CoreOptions.BatteryRamDisabled =` in the solution | 127 lines (2 in Pharaoh, 1 in MarsRT's PGO driver, 124 in WiseMan, restores included) | 0; `--nobattery` is a value in a record |
| The settings, preferences and battery-latch tests (`SettingsPersistenceTests`, the Sync to display menu case, `BatterySaveTests`, `GenesisBatteryTests`, `MercuryDefectTests`' D3) | pass | pass |
| The whole WiseMan suite, run in four parts under the test lock | — | 6,427 tests: 6,354 passed, 71 skipped, 2 failed; both fail the same way on `69974969` (below) |

The two failures are not this step's. `MultiplayerTests`' handheld-notice case with `claim: True` failed four runs of
four on `69974969` and two of three on this step, the class's first-press case once; it depends on the order the
simulated pads are polled in. `VenusGoldenTests.No_corpus_verdict_is_worse_than_the_csharp_baseline` finds no golden
for the baseline's 108 corpus ROMs under a `bin/` folder (`…/undisbeliever-snes-test-roms/bin/` first): the repository's
`.gitignore` ignores every `bin/` directory, so those traces were never committed and a fresh checkout cannot pass. Both are left for their owners.

---

## 8. Questions for the tester

**Decided 2026-10-11:** all ten, each as recommended. Q2 reads as revised: the display shapes go to the systems table in Crystal's assembly. The migration of §7 proceeds in its order, S0 first.

Each with a recommendation. None is decided by this page.

**Q1. Should the rule that the core assembly does not depend on the presentation layer be stated, and pinned by a
test?** *Recommended: yes*, in `EmuSen_Multicore.md` §9 beside the rule it mirrors, pinned by S0's reference set and
made true by S1. Nothing argues for the edge: it carries two constants and one record (§4.2).

**Q2. Where do the display shapes go?** (a) the systems table in DianaSh, which both EmuSen and Serenity already
reference; (b) Galaxia; (c) a literal in each C# core. *Recommended: (a)*, because `EmuSen_CoreAPI.md` §8.2 gives console
facts to the systems table, the Rust cores already report their shape through machine info, and Galaxia is where things
live on disk, not what a console is.
*Revised 2026-10-10, by `EmuSen_DianaSh.md` §16 Q1 (§17.2 there): the systems table is the runtime's, in an assembly
of its own beneath the shell named Crystal, not in DianaSh, so (a) reads "the systems table in Crystal's assembly". The question
itself is not decided.*

**Q3. Should Galaxia, Endymion and Serenity carry rule D, a recorded public surface?** *Recommended: yes, with the
baseline files written by S0's reader rather than by the public-API analyzer package*, because the analyzer is a
`PackageReference` and Galaxia's rule admits none (`EmuSen.Galaxia.csproj:7`). The cost is one file per assembly and a
reviewed line per surface change; the frontends do not need it, and DianaSh's rule is `EmuSen_DianaSh.md`'s.

**Q4. Should Mistress grant WiseMan friendship and make its types `internal`, replacing the reflection into
`MainWindow`?** *Recommended: yes, file by file* (S7). The 228 `typeof(MainWindow)` sites and the 106 private members
they reach are a surface that exists only at run time, so a rename breaks a test when it runs rather than when it
builds. The alternative, a deliberate test surface on `MainWindow`, is worth having where a test drives the window as a
player would, and is not this question.

**Q5. Should Serenity keep its reference to DianaSh for the `coretop` window?** *Overtaken 2026-10-11: the telemetry
contracts moved to Crystal's assembly with `EmuSen_DianaSh.md`'s S2, and Serenity's reference moved with them (§2.1's
note).* *Recommended: yes, until
`EmuSen_DianaSh.md` decides where the telemetry contracts a non-shell consumer takes should live.* The edge reaches no
toolkit and closes no cycle (§4.9). This page does not propose reviving a telemetry assembly; the 2026-10-07 fold stands.

**Q6. `GraphicsSettings.TargetFps` and `VSyncEnabled` are loaded and saved and read by nothing. Remove them?**
*Recommended: remove the static fields with S3, and keep the two keys in `graphics.json`* until the file's format
changes for another reason, because `EmuSen_RustPlatform.md` §5.1 holds config files byte-identical between the two
implementations and a removed key is a format change with a parity test of its own.

**Q7. Should `IDebugTarget` be frozen?** *Recommended: yes*: no new member; a new debugger capability goes to the core
ABI's debug interface (`EmuSen_CoreAPI.md` §6.14) and the one generic adapter over it, `CoreDebugTarget`. Its seven
implementers retire with the C# cores, so splitting it now would spend work on code with an end date (§4.6).

**Q8. Pharaoh's validation runners name 18 of the C# cores' types. Accept them?** *Recommended: accept them for the C#
cores, on S4's shrinking allow-list, and refuse the pattern for any Rust core*, whose equivalents are the conformance
kit's (`EmuSen_CoreAPI.md` §12.2). They are the tool the oracles are graded by, and they retire with them.

**Q9. Where does the table of environment switches live?** *Recommended: Galaxia*, which already holds the platform
switch and is reachable from every program; the Rust cores' own variables are listed there too, as documentation held
by S9's test, though Rust reads them.

**Q10. Should the copied Rust glue move into `emusen-native` now?** *Recommended: the six `StateError` enums and the
state trio with the next change to any of those cores; MarsRT's own hooks and sample queue at its step 5*, because a new
dependency in MarsRT renames every function its PGO profile names (`EmuSen_RustState.md` §4.3) and that step already
retrains it.

---

## 9. The counts, in one place

| Count | Value | How |
|---|---|---|
| C# projects; Rust crates in separate Cargo builds | 9 (+ LunaP); 13 (+ the probe) | `.csproj` and `Cargo.lock` files |
| Project-reference edges; used but undeclared | 23 in `.csproj` files and 4 to LunaP through `LunaP.props`; 5 | §2.1 |
| Cycles; workarounds that exist to avoid one | 0; 7 | §2.4 |
| `InternalsVisibleTo`; non-test friends | 3; 0 | §2.3 |
| Public types: EmuSen · DianaSh · Galaxia · Endymion · Serenity · Mistress | 337 · 217 · 30 · 16 · 49 · 276 | IL walk, §3.1 |
| …with a non-test outside caller | 69 · 80 · 29 · 11 · 17 · 0 | IL walk |
| Public static mutable fields and setters in the solution's shipped assemblies (IL walk: EmuSen 60, Serenity 9, Galaxia 6, Mistress 6, DianaSh 2); declarations by `grep`, LunaP included; used / written from another non-test project | 83; 104; 38 / 26 | §4.3 |
| Frontends' frame loops; post-load sequences | 3; 5 | §4.1 |
| Console-internal types named by Hotaru · Pharaoh · Mistress | 4 · 18 · 0 | IL walk, §4.5 |
| `IDebugTarget` members (required); implementers | 45 (27); 7 | §4.6 |
| `CoreCatalog` methods · properties · fields; concerns | 46 · 10 · 30; 9 | §3.2 |
| Reflection into privates, WiseMan · shipped | `NonPublic` 191 · 4 | §4.7 |
| Places that say which bit is which button | 19 | §4.8 |
| Environment variable names; changing shipped behaviour; undocumented among those | 226; 49; 3 | §4.10 |
| Shipped paths under a Galaxia root built outside Galaxia | 12 | §4.10 |
| Avalonia assemblies in Pharaoh's build output | 24; 0 since S1 (2026-10-17) | §4.2, §7.1 |
