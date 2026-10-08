# EmuSen — the platform in Rust: Galaxia, Endymion and Serenity's Vulkan half

*This revision: the second, 2026-10-07: step 2a built, the library and Galaxia's tree, names and bytes behind an opt-in switch (§10). The first, the same day, was a design with nothing built. It plans the
first part of `EmuSen_Stack.md` §6, the runtime above the cores moving to Rust, for the three platform components the
tester chose to begin with. Every claim about the code is cited to a file, read on `WiseMan` at `eec4ee5d`. Claims
marked **measured** were measured on 2026-10-07; claims marked **argued** are reasoning that a later step must prove,
and §8's predictions say how. Its twelve questions were decided the same day, as recommended (§9).*

Companion docs: `EmuSen_Stack.md` §6 (the plan this begins), `EmuSen_CoreAPI.md` (the conventions of a C interface
in this project, which §3 reuses), `EmuSen_NativeCores.md` §3–§4 (the pre-stable interface and its C# host),
`EmuSen_Galaxia.md`, `EmuSen_Config_Reference.md`, `EmuSen_Input.md`, `EmuSen_Audio_Sync.md` §7 and
`EmuSen_Serenity.md` §7–§10 (the components, as built in C#).

---

## 0. The decision, and the design in brief

### 0.1 Decided 2026-10-07

1. **The port starts with the platform pieces**, in this order: Galaxia, then Endymion, then Serenity's Vulkan and
   shader-cache half (`EmuSen.Serenity/Slang/`). One component at a time, each reported and merged before the next.
2. **Cauldron is folded into DianaOS, not ported.** Its reason to exist as an assembly ended when LunaP left the
   repository (`EmuSen_LunaP.md` §19), and in the Rust plan telemetry belongs to the core ABI (`EmuSen_CoreAPI.md`
   §6.14). §4.4 records the fold.
3. **DianaOS comes after these three.**
4. **Mistress, Hotaru and Pharaoh stay C#** and keep working unchanged. Each ported piece sits behind a C interface
   that the C# side calls through P/Invoke, as the Rust cores already do. Nothing may break partway.
5. **The user-interface toolkit (Qt against Slint) is not part of this work** (`EmuSen_Stack.md` §6).

### 0.2 The design in one paragraph

The three components become Rust crates in one Cargo workspace at the top of the repository, `Platform/`, linked
into **one** native library, `emusen_platform`, which every C# program ships beside its assemblies (§2). Each crate
has a Rust API first, which a Rust DianaOS will call directly, and a thin C layer over it, which the existing C#
assemblies call through `LibraryImport` facades that keep every public C# name and signature (§3). The C interface
follows the core ABI's rules (pull, never push; copy out; signed returns and the length-query idiom; no unwinding),
is versioned by exact match because nothing outside the repository uses it, and has one stated exception to copying
out: a lent Vulkan readback, which is a measured lever (§3.3). On-disk formats do not change; the gate for each
component is byte-identical output from the old C# and the new Rust over one corpus (§5, §6). Until its gate the C#
implementation stays in place as the oracle and the fallback, behind a switch, so that no step leaves a broken
platform (§3.9).

### 0.3 What this revises

- `EmuSen_Stack.md` §1 gave Rust "the lower level" and C# "everything else", and §2.1 revised that to cores only.
  §6 deferred the runtime's port "until more cores have been added"; 0.1 above starts its first part now. §6 carries
  a dated note pointing here.
- `EmuSen_Stack.md` §2.1's paragraph on `EmuSen.Endymion/AudioPlayer.cs` ("It stays C#") rests on a hop count that
  §4.2 re-examines. The paragraph is retired by this plan if Endymion's device half is ported (§9, Q8).
- `EmuSen_CoreAPI.md` §8.2, "Telemetry stays in Cauldron, which DianaOS already references", is revised by the fold
  (§4.4).

---

## 1. What there is to port (measured 2026-10-07)

| Component | C# lines | References | Packages | Consumers |
|---|---|---|---|---|
| `EmuSen.Galaxia` | 1,603 in 26 files | none (the leaf) | none | every assembly; 337 C# files name `EmuSen.Galaxia`, 198 of them in WiseMan |
| `EmuSen.Endymion` | 1,445 in 14 files | Galaxia | SDL3-CS, SDL3-CS.Native, Avalonia (base) | Mistress, Hotaru, WiseMan |
| `EmuSen.Serenity`, `Slang/` only | 2,225 in 10 files, of 5,466 | Galaxia, Cauldron, LunaP | Silk.NET.Vulkan, Silk.NET.Shaderc, Microsoft.Data.Sqlite with its SQLite bundle, and five of Avalonia and Skia | `GameFrameControl`, Mistress's Shaders window, WiseMan |
| `EmuSen.Cauldron` | 327 in 6 files | none | none | DianaOS, EmuSen, Serenity, WiseMan |

**How Galaxia is reached.** The path API (`ConfigRoot`, `ConfigStore`, `DataStore`, `SaveLibrary`, `AtomicFile`) is
called from 7 files of `EmuSen`, 18 of Mistress, one each of DianaOS, Hotaru, Pharaoh and Serenity, and 120 of
WiseMan. `ConfigFile<T>` is instantiated outside Galaxia for three types whose vocabulary Galaxia cannot name:
`Dictionary<…, SDL.GamepadButton>` (`EmuSen.Endymion/Input/GamepadBindings.cs`),
`Dictionary<…, Avalonia.Input.Key>` (`EmuSen.Mistress/Input/ControllerKeyBindings.cs`) and
`Dictionary<HotkeyAction, Key>` (`EmuSen.Mistress/Input/HotkeyBindingMap.cs`). That distinction shapes §4.1.

**Where SDL is called.** Every SDL function call in the tree is inside Endymion (`AudioPlayer.cs`, `UiSoundPlayer.cs`
and the pad files). Mistress names SDL's enumerations (`SDL.GamepadButton`, `SDL.GamepadType`) for its diagrams and
bindings, and calls no SDL function.

**What tests hold the components.** `EmuSen.WiseMan/Galaxia/` (13 classes, 1,735 lines), with
`DianaOS/HierTests.cs` and `DianaOS/DianaOSSandboxTests.cs`; `Audio/` (five classes) and `Input/` (nine) for
Endymion; `Serenity/SlangChainTests`, `SlangCompileTests`, `SlangPresetTests`, `SlangReadbackTests`,
`SlangSyncTests`, `SpirvCacheTests` and the bench for Serenity's half.

---

## 2. Crate layout, building and publication

### 2.1 Where the crates live

```
Platform/                      a Cargo workspace; no .csproj
  Cargo.toml                   [workspace], shared Cargo.lock, the cores' release and dist profiles
  include/emusen_platform.h    the C interface, written by hand (§3)
  galaxia/                     emusen-galaxia, an rlib: paths, files, config, the models
  endymion/                    emusen-endymion, an rlib: input vocabulary, routing, the audio sink, pads
  serenity/                    emusen-serenity, an rlib: presets, Shaderc, reflection, the SPIR-V cache, the device
  platform/                    emusen-platform, the cdylib: the C layer of all three, and nothing else
```

**Not `EmuSen/Cores/Shared/`.** That folder holds the crates cores link (`emusen-native`, the ABI check, the
conformance kit). A core does no file I/O and calls nothing above it (`EmuSen_CoreAPI.md` §3's table), so a platform
crate beside them would be one `path =` away from a core depending on the platform, which the layering forbids.
Keeping them apart makes that dependency visible in a diff.

**Not inside each C# project** (`EmuSen.Galaxia/rust/`). The C# projects are retired at the end of `EmuSen_Stack.md`
§6, and the crates are not; a crate inside a project folder would also put cargo's `target/` under an MSBuild glob,
which `EmuSen.csproj` already needs five `DefaultItemExcludes` patterns to keep out of the cores' folders. A
top-level folder outside every project has neither problem, and a Rust DianaOS joins it as `Platform/dianaos/`.

**One workspace, unlike the cores.** Each core is its own crate with its own `Cargo.lock`, because a core is shipped
and versioned alone. The platform crates are one program, which a Rust DianaOS will link as one: one lock file keeps
their shared dependencies at one version.

**Reuse.** The crates depend on `emusen-native` for `ffi::status`, `ffi::input`, `ffi::copy_text` and
`abi::install_crash_log` rather than copying them (read in `EmuSen/Cores/Shared/emusen-native/src/ffi.rs` and
`abi.rs`), as the cores share them. That is a dependency of the platform on a shared crate, not of a core on the
platform, so §2.1's rule holds.

### 2.2 One library, not three

**Recommended: one cdylib, `emusen_platform`** (`libemusen_platform.so`, `emusen_platform.dll`,
`libemusen_platform.dylib`), whose C layer covers every ported component.

- **Process-wide state exists once.** Galaxia holds globals that tests move: `ConfigStore.OverrideDirectory` and
  `DataStore.OverrideDirectory`, set by about thirty WiseMan fixtures (`EmuSen_Stack.md` §4.1). Three libraries would
  each link their own copy of `emusen-galaxia`; an override set through Galaxia's library would not move the paths
  Serenity's library computes (the SPIR-V cache is under `DataStore.Shaders`). One library has one copy.
- **One Rust standard library, one allocator, one panic hook and one crash log** per process, rather than three.
- **It is the shape of the end state.** A Rust DianaOS links the three crates into one runtime.

The cost: a program that uses only Galaxia ships a library containing Endymion's and Serenity's code. Neither links
SDL, Shaderc or Vulkan at build time (§2.8), so the cost is file size, a few megabytes (argued; measured in step 2a).

### 2.3 A Rust API first, a C layer over it

Each component crate exposes an ordinary Rust API (`galaxia::DataStore::saves()`, `galaxia::config::load::<AppSettings>()`,
`serenity::Chain::build(...)`), with Rust types, `Result` and ownership. The C layer lives only in `emusen-platform`,
one module per component, and does nothing but convert: pointers to slices, statuses to and from `Result`, models to
and from JSON. This is the arrangement of `emusen-native`'s `core` trait and its `core_exports!` macro
(`EmuSen_CoreAPI.md` §18.2), and it is what lets a Rust DianaOS use the crates without passing through C.

### 2.4 Dependencies

The cores are almost dependency-free by habit (`emusen-native` has none). The platform needs bindings to C libraries
the C# side already binds, and two hashes:

| Crate | For | Licence |
|---|---|---|
| `ash` | Vulkan, loaded at run time | MIT or Apache-2.0 |
| `libloading` | SDL3 and Shaderc, loaded at run time from the shipped files (§2.8) | ISC |
| `rusqlite` with `bundled` | the SPIR-V cache (§4.3) | MIT (SQLite itself public domain) |
| `sha2`, `md-5` | the cache's keys; `RomHash.Md5` | MIT or Apache-2.0 |

**No `serde`.** The JSON must reproduce `System.Text.Json`'s reading and writing exactly (§5.2), which no serde
format does: comments and trailing commas on read, its escaping and its number formatting on write. A hand-written
reader and writer are needed in any case, and the models map to them directly. All four licences are compatible with
the project's GPL-3.0 and go into `THIRD_PARTY_NOTICES.md` with the step that adds them.

### 2.5 Building

`Platform/Platform.targets`, imported by `EmuSen.Galaxia.csproj`, builds the workspace's cdylib with the conventions
`EmuSen.csproj` already uses for the cores: `FindCargo`, the `release` profile for a build and `dist` for a publish,
the target directory under `obj/` (`EmuSenCargoTargetRoot`), and `-p:EmuSenNative=false` to keep cargo out. The
library is added as a `None` item with `CopyToOutputDirectory`, which MSBuild carries to every project that references
Galaxia, so every program and the test assembly get it without a line of their own.

**Galaxia stays a leaf in the sense that matters.** Its rule is no `ProjectReference` and no `PackageReference`,
because anything it depended on upward would close a cycle (`EmuSen_Galaxia.md` §1). A build target and a native
library beside the assembly close none, and `LeafAssemblyTests` pins assembly references, which do not change.

**A machine without cargo.** Until a component's gate, its C# implementation is the fallback (§3.9), so the build
warns and the program runs as it does today. After the gate there is no fallback, and a build without cargo needs a
prebuilt library: `-p:EmuSenNativePrebuilt=<dir>`, as a foreign-platform publish of the cores takes today, or it
fails with a message naming what to pass (§9, Q4).

### 2.6 Publishing for another platform

`EmuSen/Cores/RustCoresPublish.targets` already drops the host's libraries and takes each crate's from
`$(EmuSenNativePrebuilt)/<rid>/`. It gains the platform library as one more item, with one difference: a core with a
C# fallback is a warning when its library is missing, and the platform library after its first gate is an error,
because the program would not start without it.

### 2.7 CI

`.github/workflows/rust-cores.yml` gains the workspace rather than a second workflow, because the WiseMan job that
runs there loads Galaxia in every test, so once Galaxia's switch defaults to Rust it needs the platform library as
much as the cores':

- the `shared` job runs `cargo test --release` in `Platform/` on the three operating systems;
- the `library` matrix gains `emusen_platform` for the four triples, with the export count's symbol prefix
  `emusen_galaxia_`;
- the `wiseman` job downloads the artifact beside the cores';
- the path filters gain `Platform/**`.

### 2.8 Native libraries the platform loads, and the one it brings

- **SDL3 and Shaderc are loaded by path from the program's directory**, the files `SDL3-CS.Native` and
  `Silk.NET.Shaderc.Native` already ship, through `libloading` and hand-written declarations of the functions used
  (29 of SDL's, counted from Endymion's call sites, and the 14 of Shaderc's that `SlangCompiler` calls; **measured**). The binaries do not
  change, so behaviour that depends on them cannot: Shaderc's SPIR-V is the same bytes (§8, P6), and SDL's pad
  mappings are the same database. The C# facade passes the directory; the dynamic loader returns the same instance
  for the same file, so a C# fallback and the Rust side in one process share one SDL (argued from `dlopen`'s
  reference counting; checked in step 3b).
- **Vulkan's loader** (`libvulkan.so.1`) is the system's, as it is for Silk.NET; one loader, separate instances.
- **SQLite is the exception: `rusqlite` brings its own copy**, and `EmuSen` already ships `e_sqlite3` through
  `SQLitePCLRaw`. Two SQLite libraries in one process may each open *different* files safely; opening the *same*
  file from both breaks POSIX advisory locking, because closing one copy's descriptor releases the other's locks
  (SQLite's own documentation lists linking two copies into one application as a way to corrupt a database). The
  rule is therefore: **one database file, one SQLite copy, per process.** Only Serenity's cache moves, and it moves
  whole (§4.3); `games.db`, `records.db`, `media.db` and the catalogue stay with `e_sqlite3`. The parity tests of
  step 4a open the C# and the Rust cache on separate files.

---

## 3. The C interface

### 3.1 Names

The core ABI's naming rule (`EmuSen_CoreAPI.md` §5.1), one layer up:

- library-level exports `emusen_platform_*` (version, crash log, last error);
- each component's exports `emusen_galaxia_*`, `emusen_endymion_*`, `emusen_serenity_*`;
- types `emusen_<component>_<name>` without a function of the same name; constants `EMUSEN_PLATFORM_*`.

The header `Platform/include/emusen_platform.h` is written by hand and is the specification; the Rust declares the
same functions; a WiseMan test resolves every export the C# facades declare when the library loads, so a missing or
renamed export fails a test rather than a player's start.

### 3.2 Versioning: exact, and why not stable

```c
uint32_t emusen_platform_abi_version(void);   /* one number, matched exactly */
```

The C# facade is compiled against one number and refuses a library with another, as the pre-stable core interface
did (`EmuSen_NativeCores.md` §3.2). Exact matching was retired for cores because it made an outside core impossible
and moved every core with every change (`EmuSen_CoreAPI.md` §4.8). Neither cost applies here. Nothing outside the
repository calls this interface; its only client is the C# side, built from the same commit and shipped beside it;
and it is transitional, ending when the C# does. A Rust DianaOS calls the crates' Rust API (§2.3), and the stable
interface a non-Rust client would use is DianaOS's frontend API (`EmuSen_CoreAPI.md` §9.2), which this does not
replace. **Where this does not generalise:** if any part of this interface acquired a client built elsewhere, it
would need §4 of the core API's policy and a baseline, and should not cite this paragraph.

### 3.3 Ownership and lifetimes

As `EmuSen_CoreAPI.md` §6.17: every buffer passed in is the caller's for the call; nothing passed in is kept; every
result is copied into the caller's buffer by the length-query idiom; objects with a life (a Vulkan device, a chain, a
cache, an audio sink, a pad set, a resampler) are opaque handles made and freed by the library. No export returns a
pointer except a handle's constructor.

**The one exception: a lent readback.** `SlangChain.RenderImage` hands Skia an `SKImage` made over the mapped
readback buffer itself, with no copy, and a slot is held until Skia's release call, on whatever thread Skia makes it
(`EmuSen_Serenity.md` §9.1). That lever was measured, and copying out would undo it. So:

```c
typedef struct { uint32_t size; const uint8_t *pixels; size_t bytes; int32_t width, height, row_bytes; uint64_t slot; } emusen_serenity_lent;
int32_t emusen_serenity_chain_render_lent(emusen_serenity_chain *c, int32_t w, int32_t h, emusen_serenity_lent *out);
int32_t emusen_serenity_lent_release(uint64_t slot);   /* any thread, exactly once */
```

The rule is the C# one, unchanged: a held slot is never written; a render takes a slot not held, makes another up to
three, and past that renders into a scratch readback and copies; a chain freed under a held slot leaves the slot to
the release. The pointer is valid from the call until its release and never after. This is the only pointer into
library memory that crosses, and it is named in the header as the exception.

### 3.4 Strings

- **In:** UTF-8 as `(const uint8_t *, size_t)`, never NUL-terminated by assumption. A C# `string` holding a lone
  surrogate cannot be encoded and is refused with `EMUSEN_BAD_STRING` (−5) before the call.
- **Out:** the length-query idiom: the export returns the whole length in bytes, writes only when the buffer holds
  it all, and a null `out` asks the length. The facade tries a stack buffer first (512 bytes, as built), so a path
  costs one call.
- **Paths** are UTF-8 on every platform; `std::path` converts on Windows. Paths are composed in Rust with the same
  separators .NET's `Path.Combine` uses on the platform, which the parity tests check on all three (§6.3).

### 3.5 Errors

- **Statuses**, signed, as everywhere in the project. The shared codes keep their numbers: −1 `NULL`, −5
  `BAD_STRING`, −7 `BUFFER_TOO_SMALL`, −256 `NOT_SUPPORTED`. The platform's own band is −1024 to −1279, clear of every
  core's and of the core interface's, so a log that prints both cannot confuse them: `IO` −1024, `PARSE` −1025,
  `SCHEMA` −1026 (a value of the wrong type, an enumeration name not known), `NEWER_FILE` −1027 (a file of a newer
  build), `NO_DEVICE` −1028 (no Vulkan device or audio device), `COMPILE` −1029 (a shader). *Added by step 2a (§10.2):*
  `ABSENT` −1030, which is an answer and not a failure (no such file, no seed directory, a null where .NET returns
  null); `NOT_FOUND` −1031 and `ACCESS` −1032, which the facade turns back into the exception types the C# threw; and
  `BAD_ARGUMENT` −1033.
- **Words** for the last failing call **on the calling thread**: `emusen_platform_last_error(out, len)`. Per thread
  rather than per handle, because most of Galaxia's calls have no handle; a P/Invoke runs on the caller's thread, so
  the facade reads it straight after the failing call.
- **Diagnostics are returned, not pushed.** Where C# reports through `ConfigDiagnostics.Report` today (a file that
  would not load, an unwritable save), the Rust returns the message with the status and the facade reports it; the
  sink, its `LastMessage` and every test that reads them stay as they are. *As built (§10.2):* one call can report
  several times (a migration, once for each file it leaves in place), so the messages wait in a queue of the calling
  thread, which the facade drains after the call with `emusen_platform_diagnostics` and reports in their order. It
  is still a pull.
- **No unwinding.** The library is built with `panic = "abort"` and installs the crash log the host names
  (`emusen_platform_set_crash_log`), with `emusen-native`'s hook. A malformed file never panics; a panic is a defect,
  recorded, as for the cores.

### 3.6 Threading

- **Library-level calls and Galaxia's calls are safe from any thread**, concurrently. The overrides and the computed
  root sit behind an `RwLock`; the error log's append is under one mutex, as `ErrorLog.Gate` is.
- **A handle is used by one thread at a time**, as a machine is (`EmuSen_CoreAPI.md` §6.16), except where the header
  names otherwise: the lent-slot release (§3.3) from any thread, and the Vulkan device, which serialises its own
  queue as `SlangVulkan` does with its lock (`EmuSen_Serenity.md` §7.5).
- **What does not change:** two saves of one config file from two threads race on the same `.tmp` name, today and
  after. The port reproduces behaviour, including this; fixing it is a separate change with its own test.

### 3.7 Callbacks: none

The library never calls managed code. Each place where the C# passes a delegate today becomes a pull:

| Today | After |
|---|---|
| `ConfigDiagnostics.Sink`, an `Action<string>` | the messages drained by the facade after the call (§3.5); the sink stays C# |
| `ErrorLog.Redactor`, a `Func<string, string>` | the C# facade formats and redacts the entry, then hands the line to Rust, which owns the file's name, the pruning and the size bound |
| `PortRouter`'s `setButton`, `setAxis` and `setConnected` | a poll returns the changes as an array of `(port, kind, control, value)`; the C# applies them to the core |
| the Vulkan debug messenger's sink (`EmuSen_Serenity.md` §10.4) | the library queues messages; the tests drain them |
| Skia's raster release delegate | the C# delegate calls `emusen_serenity_lent_release`, which is C# calling Rust, the permitted direction |

### 3.8 Structured data

Models and lists cross as **JSON**, as the core ABI's descriptors do (`EmuSen_CoreAPI.md` §15, Q2): config is read
once and written on a change, and is never on a per-frame path (`EmuSen_CoreAPI.md` §2.5). The per-frame calls
(the audio submit, the pad poll, a chain's render) take scalars, spans and size-prefixed structs.

### 3.9 The C# side, and the switch

Each assembly keeps its public types, names and signatures; their bodies forward. The loader is one class,
`EmuSen.Galaxia.Native.PlatformLibrary`, which loads the library once, checks the version, installs the crash log and
answers `Available` and `Report`; Endymion's and Serenity's facades use it, since both reference Galaxia.

Each component has a switch, as each Rust core has (`EMUSEN_MARS_NATIVE=0`): `EMUSEN_GALAXIA_NATIVE`,
`EMUSEN_ENDYMION_NATIVE`, `EMUSEN_SERENITY_NATIVE`. A component passes through three states, and each change is a
reported step:

| State | Default | `=0` | `=1` | Without the library |
|---|---|---|---|---|
| 1. built, before its gate | C# | C# | Rust | C#, silently |
| 2. after its gate | Rust | C# | Rust | C#, with a warning in the log |
| 3. C# retired | Rust | refused, with a message | Rust | the build fails (§2.5) |

So at every point the platform runs: in state 1 nothing a player sees changes; in state 2 the C# is one variable
away; state 3 comes only after golden outputs are recorded (§6.6) and the tester agrees.

---

## 4. Per component: what moves, what C# keeps, what it buys

### 4.1 Galaxia

**What moves to Rust:**

- **The tree:** `ConfigRoot` (the walk to `EmuSen.sln` or `.dianaosroot`, the macOS bundle rule, the seed directory;
  `EmuSen_Galaxia.md` §3.4), `ConfigStore`, `DataStore`, both overrides and the legacy directory.
- **The names:** `SaveLibrary`, all five rules (`EmuSen_Galaxia.md` §5).
- **The bytes:** `AtomicFile`, `DataMigration` (copy, never move, never overwrite), `RomHash.Md5`, and `ErrorLog`'s
  file: its daily name, its fourteen-day pruning and its 8 MB bound.
- **Config files, in two tiers**, because of §1's three foreign vocabularies:
  - **Tier 1, schemas Rust owns:** `AppSettings`, `AudioConfig`, `GraphicsConfig`, the three `BigPicture*` types,
    `CheatFile` and `StateRecord`'s sidecar. Rust reads the file leniently (§5.2), applies defaults and the
    migrations (`AppSettings.Upgraded`'s `SelectedCore` and `OnlineCovers` rules, `CheatFileEntry`'s pre-multi-write
    fields, the legacy-directory copy of `ConfigFile<T>.MigrateFromLegacy`), validates types and enumeration names
    with Galaxia's own suggestion message (`SuggestingEnumConverter`, `Text/Suggestion.cs`), and writes the file.
  - **Tier 2, generic files whose types C# owns** (key and hotkey bindings, keyed by Avalonia's `Key`): Rust owns
    the file's place, its legacy migration and its atomic write; System.Text.Json still binds the type. These move
    with the interface toolkit, not here.

**What C# keeps:**

- **The model classes, as thin records filled through the interface**: the same classes, properties and defaults,
  so the 337 files that name them do not change. `Load` asks Rust for the canonical JSON of the file, which C# binds
  with a source-generated, strict System.Text.Json context; `Save` serialises the canonical form, and Rust writes the
  file. The in-memory helpers (`GraphicsConfig.SetValue`, `NoteRecent`, `ForgetParameter`) stay C#: they touch no
  disk, and forwarding a dictionary edit through P/Invoke buys nothing.
- `ConfigFile<T>` as a type, forwarding; `ConfigDiagnostics` (the sink); the input enumerations `PadButton`,
  `PadAxis`, `PadControl` as mirrors of Rust's, pinned by a test as `PadButton` is pinned to `SnesButton`
  (`EmuSen_Input.md` §3); `ICatalogue` and `catalogue-schema.sql`, the contract.
- `CheatFileWrite`'s hex helpers (`TryHex`, `Strip`, `TryParseNumbers`), which DianaOS's C# `CheatRegistry` calls per
  entry; they move with DianaOS.
- `SuggestingEnumConverter`, for tier 2 only.

**What does not move in this port:** `games.db` (schema 6), `records.db` and `media.db` are Mistress's, with their
migration lists in `EmuSen.Mistress/Library/` (`GameRecords.cs`, `FileRecords.cs`); Mistress stays C#, so they stay
with it, and `EmuSen_CoreAPI.md` §3 places the library with the clients in any case. The catalogue's driver
(`EmuSen/Common/Catalogue/SqliteCatalogue.cs`) stays in `EmuSen` and joins DianaOS when DianaOS moves.

**What it buys, plainly.** The on-disk formats, their parsing and validation, the migrations, the atomic writes and
the save library's paths become Rust that a Rust DianaOS uses directly, through §2.3's API, with nothing to rewrite.
**For the C# programs it buys nothing measurable.** No config read is on a hot path, so no speed is gained, and
every load gains a JSON round trip (§8, P1) and every program a native library. The port is paid for by the later
step, and done now because the C# is alive to be the oracle: parity can be proven against a running implementation
today, and could only be proven against recorded outputs once the C# frontends are gone.

### 4.2 Endymion

Endymion has two halves of different kinds, and they are argued separately.

**The logic half:** `LinearResampler`, `DynamicRateControl`, `PortRouter`, `PlayerSlots`, the binding files'
format (`GamepadBindings`, including its legacy flat shape and per-player keys; `EmuSen_Input.md` §5.1, §8.4), and
`PadControls`' `Combine` and `Resolve` (in Galaxia today). Pure arithmetic and rules over state; bit-exact parity is
reachable (§6.3).

**The device half:** `AudioPlayer`, `UiSoundPlayer`, `GamepadManager`, `ConnectedPad` and `IPadDevices` with its
simulated implementation (`SimulatedPads`), which WiseMan's pad tests drive in SDL's place. In Rust the seam becomes
a trait with an SDL implementation and a simulated one, and the simulated one is reachable through the C interface,
so `PortRouterTests`, `GamepadManagerPadsTests`, `GamepadRescanTests` and the rest keep running headless.

**What C# keeps:** `DefaultPadKeyMap`, which maps Avalonia's `Key` (`EmuSen_Input.md` §4.3); SDL's enumerations as
the C# vocabulary of `GamepadBindingMap` and of Mistress's diagrams, so `SDL3-CS` stays referenced for its types
while no C# calls an SDL function; and the facades.

**`EmuSen_Stack.md` §2.1's argument, re-examined.** It kept `AudioPlayer.cs` in C# because "moving it to Rust would
*add* an FFI boundary rather than remove one: today there is one hop (C# → SDL3), and a Rust sink would make it
two". Counted as layers, that is right. Counted as transitions, which is what costs time, it is not: `Submit` makes
two SDL calls in steady play, `GetAudioStreamQueued` (through `QueuedFrames`) and `PutAudioStreamData` (read in
`AudioPlayer.cs`), and a Rust sink makes one call into Rust, which makes both SDL calls natively. So the port does
not add a transition per frame; it removes one (argued from the code; P4 measures it). Neither number matters at one
submit a frame. What the paragraph did get right stands: **the device half buys nothing for the C# programs now.**
Its reason is the later Rust client, which `EmuSen_CoreAPI.md` §8.2 gives the audio device and the pads, and the
oracle argument of §4.1. §9, Q8 asks whether to port it now or with that client.

**What DianaOS will need from it** is the logic half only: input routing is the runtime's (`EmuSen_CoreAPI.md` §8.2:
"canonical controls from a client, routed to a port and a core bit"), and so is the sound's ring (§8.3 there); the
sink that drains the ring, with its rate control and resampler, and the devices are a client's.

### 4.3 Serenity's Vulkan and shader-cache half

**What moves:** the ten files of `EmuSen.Serenity/Slang/`, less the runner's integration: `SlangPreset` (the
`.slangp` reader with `#reference` chains), `SlangSource` (includes, stages, pragmas), `SlangParameters`,
`SlangCompiler` (Shaderc, unoptimised, as now), `SpirvReflection` (including `WithoutUnreadInputs`, §10.3 there),
`SpirvCache` (the SQLite cache, its key, its damage handling, its bound), `SlangVulkan` (the device, one per
process, its validation sink for tests) and `SlangChain` (passes, history, feedback, parameters, the readback and its
lending, the subpass dependency of §10.2 there).

**What C# keeps:** `SlangRunner`, which is `GameFrameControl`'s glue (it builds a chain on a pool thread and draws
under the control's lock, `EmuSen_Serenity.md` §7.5), now calling the Rust chain; `GameFrameControl`, the SkSL
`FilterChain` and every built-in filter (the CRT tiers, the handheld LCDs), `DisplayClock`, `VblankCounter`,
`FramePresenter`, `GraphicsSettings` and the coretop dashboard. These draw through Avalonia and Skia and move with the
interface toolkit, which is not part of this work. `SlangCompileException` keeps its type and messages, rethrown by
the facade from the status and the words; the compiler's own text is Shaderc's and unchanged (§2.8).

**What it buys.** For DianaOS, nothing: presentation is a client's (`EmuSen_CoreAPI.md` §3). For the C# programs, one
prediction and no certainty: `EmuSen_Serenity.md` §9.8 records 15–118 KB of managed allocation a frame in `Bind` for
multi-pass presets and one gen-2 collection in N64 4× royale, which the Rust chain does not make (§8, P5). If P5 is
not borne out, the step buys nothing until a Rust client exists, and this page says so now rather than after. Its
case rests on that client and on the oracle argument; §9, Q9 asks whether the tester wants it at the third place
regardless.

**The cache's keys.** A key hashes the compiler's identity, which includes "the binding's version"
(`EmuSen_Serenity.md` §9.4). The Rust loader is a different binding over the same Shaderc file, so its identity
names itself, and its rows are new keys beside the C# side's: a cold build once per machine, then the old rows age
out under the bound. The schema and `user_version` 1 do not change, so either implementation can open the file.

### 4.4 Cauldron: folded, not ported

Its six files move into `EmuSen.DianaOS`, keeping the `EmuSen.Cauldron` namespace so no caller changes; Serenity
references DianaOS in its place; the project and its four `ProjectReference`s go. `EmuSen_Cauldron.md` becomes a
section of the DianaOS documentation, keeping its §3.1 rule on what belongs to telemetry and what to the debugger.
That is step 1b. *Done 2026-10-07: the files are in `DianaOS/Lib/Cauldron/` and the page is
`EmuSen_Debugging_Tools_Reference_v5.md` §3.66, with the split rule at §3.66.3.1 and the fold's own record at §3.66.1.1.* In the Rust plan the read surface it defines is the core ABI's debug interface
(`EmuSen_CoreAPI.md` §6.14) seen through DianaOS, so a separate Rust crate for it would be a boundary nothing uses
(`EmuSen_Multicore.md` §9.2).

---

## 5. Formats, and the parity rule

### 5.1 What must not change

- **Config JSON**: every file under `/etc/EmuSen`, the `cheats/` category, and the legacy files under
  `~/.config/EmuSen` read for migration.
- **Saves and states**: `.srm`, `.state` and their names; their bytes are a core's, and Galaxia passes them through.
- **Cheats**: `cheats/<game>.json` (`CheatFile`); the `.cht` database is DianaOS's and is not touched here.
- **`games.db` (schema 6) and the other two databases**: not touched (§4.1).
- **The SPIR-V cache**: schema and version unchanged (§4.3).

**The parity rule.** For every input in the corpus, the old C# and the new Rust produce byte-identical output: the
same bytes written, the same canonical model read, the same success or failure, the same diagnostic present or
absent. Existing files read identically, including files a person has edited by hand.

### 5.2 What System.Text.Json actually does (measured 2026-10-07)

Measured in a scratch program on .NET 10 on Linux, with `ConfigJson.Options`' settings (indented, comments skipped,
trailing commas, case-insensitive names) and `Enum.TryParse` as `SuggestingEnumConverter` calls it. Each row is a
place where a Rust writer or reader with ordinary defaults would differ:

| Case | .NET | Ordinary Rust (`serde_json`, `ryu`) |
|---|---|---|
| `1e-7`, `1e21` as `double` | `1E-07`, `1E+21` | `1e-7`, `1e21` |
| `1.0` as `double` | `1` | `1.0` |
| `-0.0` | `-0` | `-0.0` |
| `0.8f`, `1e-8f` as `float` | `0.8`, `1E-08` | `0.800000011920929` if widened to `f64` |
| `é<>&'+"` and a backtick | `é<>&'+"`` (upper-case hex) | the characters themselves, `\"` for the quote |
| `€`, `😀` | `€`, `😀` | the characters |
| tab, U+0001 | `\t`, `\u0001` | `\t`, `\u0001` |
| an empty map, an empty list | `{}`, `[]` | the same |
| line ends | LF on Linux (`NewLine` defaults to `Environment.NewLine`; CRLF on Windows, argued) | LF |
| `{"a":1,"A":2}` into `A` | `2`: the last match wins, case-insensitively | an error, or the first |
| `1.0` or `"1"` into `int` | an error | per implementation |
| top-level `null` | `null` returned, and no error | per implementation |
| text after the value | an error | the same |
| a UTF-8 BOM | skipped (`File.ReadAllText`) | an error |
| an invalid byte (`FF`) | read as U+FFFD | an error with `from_utf8` |
| `"Up, Down"` for a `PadButton` | `Down`: names are OR-ed (4 \| 5) | an error |
| `"B,Y"` | `Y` (0 \| 1) | an error |
| `"99"`, `7` | kept as the undefined value 99 or 7, and written back as `"99"`, `"7"` | an error |
| `" Up "`, `"up"`, `"-1"` | `Up`, `Up`, −1 | an error |

The Rust writer therefore formats doubles and floats with .NET's shortest round-trip form and exponent spelling,
keeps each model field's width (`AudioConfig.MasterVolume` is a `float`), escapes as `JavaScriptEncoder.Default`
does, and writes the platform's newline; the reader accepts what the table says .NET accepts and refuses what it
refuses.

### 5.3 Quirks are reproduced

A file that loads today must load identically (§5.1), and nothing distinguishes a quirk a person has relied on from
one nobody has met. So the enumeration rows of §5.2, the U+FFFD replacement and the case-insensitive last-match rule
are reproduced, each with a corpus case, rather than tidied (§9, Q6). Tidying any of them later is a behaviour
change with its own record.

### 5.4 Words are not

Galaxia's own messages are reproduced exactly: "'DPadUpp' is not a valid GamepadButton. Did you mean 'DPadUp'?",
the file path first and "Falling back to defaults." last (`ConfigDiagnosticsTests`). System.Text.Json's exception
text inside them ("The JSON value could not be converted to System.Int32. Path: $.I | LineNumber: 0 |
BytePositionInLine: 8.") is .NET's, and Rust gives its own words with the same path and position. No test asserts
the wording (they assert that a message exists, or Galaxia's own phrases), so the change is visible only to a player
reading a log (§9, Q5).

### 5.5 Three assumptions that do not carry over

- **A dictionary's order** on disk is the order C# enumerates it, and Rust preserves the order of the canonical JSON
  it is given, so they agree. They would not agree if Rust edited a model itself: .NET's `Dictionary` reuses a
  removed entry's slot, so a console forgotten and another added appears in the forgotten one's place, not at the
  end. That matters only once a Rust DianaOS edits `GraphicsConfig`, and is recorded here for that step.
- **.NET's special folders, not the platform's conventions.** `ConfigStore.LegacyDirectory` and
  `ErrorLog.DefaultRoot` use `SpecialFolder.ApplicationData`, which .NET maps to `$XDG_CONFIG_HOME` or `~/.config` on
  Linux and on macOS, and `%APPDATA%` on Windows. Rust reproduces that mapping; a convention crate would give
  `~/Library/Application Support` on macOS, which is a different directory. *Corrected by step 2a (§10.3): only the
  Linux half of that sentence was measured (`/home/<user>/.config`). The macOS half was written from memory and may
  be wrong for .NET 8 and later, which is believed to answer `~/Library/Application Support`; it has not been run on a
  Mac. The design no longer depends on the answer: the host passes .NET's own value to the library at start, as it
  passes the program's directory, so the library never has to guess what .NET would say.*
- **The program's directory is passed, not found.** `ConfigRoot` starts from `AppContext.BaseDirectory`. Rust's
  `current_exe()` is `dotnet` itself under `dotnet test`, so the facade passes the base directory once at start, and
  a Rust program that passes none starts from its own executable's directory.

---

## 6. Test strategy

### 6.1 Rust unit tests, per crate

Each crate's `cargo test` covers its own rules with no .NET present: the JSON reader and writer over §5.2's table;
`ConfigRoot`'s branches with the platform and home as parameters, as `ConfigRoot.ComputeFor` already allows; the
atomic write's abandoned-temp case; the resampler and rate control over fixed inputs; preset and source parsing over
fixtures; the reflection over committed SPIR-V. CI runs them on Linux, Windows and macOS (§2.7).

### 6.2 The existing WiseMan tests, through the facade

The classes of §1 run unchanged through the C# facade, **once under each setting of the component's switch**
(`=0` and `=1`) while it is in states 1 and 2 (§3.9), so the facade is held to what the C# was held to. Nothing in
them is edited; a test that had to change would mean the facade is wrong, the argument `EmuSen_Galaxia.md` §2 made
when the directories moved. The run is filtered to the component's classes and their direct consumers, never the
whole suite.

### 6.3 Parity tests, old against new

One class per component (`GalaxiaParityTests`, `EndymionParityTests`, `SerenityParityTests`) calls both
implementations in one process on the same inputs and compares bytes.

- **Galaxia.** A committed corpus of synthetic files, one per row of §5.2 and per migration; a corpus generated from
  seeded random models, serialised by both sides; the files of a real tree, copied into a scratch directory and
  read there, never committed and never written in place, with `screenscraper-developer.json` excluded because it
  holds credentials (`EmuSen_BigPicture.md` §5.7); and a table of ROM names for the save library (dots, no
  extension, non-ASCII, a separator). Output: the written bytes, the canonical model, the status, the diagnostic.
- **Endymion.** The resampler and rate control over recorded sample streams, sample for sample and state for state:
  `Math.Round` rounds half to even, so the Rust uses `round_ties_even`, not `round` (a rounding rule decides parity
  again, as in `EmuSen_Stack.md` §3.1, where the difference predicted did not exist; here it does); `PortRouter` over scripted pad and key sequences on the simulated pads,
  comparing every change sent to the core.
- **Serenity.** SPIR-V bytes for every pass compiled (all 1,350 of the pack's with `EMUSEN_SLANG_PACK`); reflection
  output; the cache's keys and behaviour on a damaged row and file, on separate files (§2.8); and the bench's pictures,
  byte for byte on the RX 6800, which is how `EmuSen_Serenity.md` §10.5 already compares, with the validation layer
  silent and its positive control seen.

### 6.4 The schema check

Rust's tier-1 models list themselves (each field's name, type and default) through one export; a WiseMan test reads
the C# models by reflection and compares the two, defaults included. A field added on one side only fails a test,
the role the core ABI's §5.3 check plays for its tables.

### 6.5 Seeded faults

Each parity class is shown to catch a fault of each kind before it is trusted, as the project's tests are: for
Galaxia at least an unescaped `+`, a `float` written as `double`, the BOM refused, the first case-insensitive match
kept, and the legacy copy overwriting; for Endymion `round` for `round_ties_even` and the shedding exit factor
swapped; for Serenity the subpass dependency dropped and the history stepped forward. Which ones survive is recorded
with the step.

### 6.6 At the gate

Before a component's C# is deleted (state 3), its parity corpus is run once more and its outputs are committed as
golden files beside the corpus, so that after the oracle is gone the Rust is still held to what the C# produced.
They are text and small images; no ROM, firmware or player file is among them.

### 6.7 What is run

Only what a step touches: the component's Rust crate, its WiseMan classes under both switch settings, its parity
class, and the build of every project. The full suite is not run for a step.

---

## 7. Order, estimate, and what DianaOS will need

### 7.1 The steps

| Step | What | Its gate |
|---|---|---|
| 1a | this page | the tester's go |
| 1b | Cauldron folded into DianaOS (C#) | every frontend builds; the telemetry, coretop and dashboard tests |
| 2a | the workspace, the library, `Platform.targets`, publish and CI; the C layer's library-level calls; Galaxia's tree, names and bytes (`ConfigRoot`, `ConfigStore`, `DataStore`, `SaveLibrary`, `AtomicFile`, `DataMigration`, `RomHash`) | §6.2 and §6.3 for these, in state 1 |
| 2b | the JSON reader and writer, `ConfigFile<T>` in both tiers, the tier-1 models and their migrations, `ErrorLog`, the suggestion text | §6.2–§6.5 for all of Galaxia |
| 2c | Galaxia to state 2 | the tester's review of 2a–2b |
| 3a | Endymion's logic half | its parity class bit-exact |
| 3b | Endymion's device half, if Q8 says now | the pad and audio tests on the simulated and dummy devices |
| 4a | presets, sources, parameters, Shaderc, reflection, the cache (no device) | SPIR-V and cache parity |
| 4b | the device and the chain, with the lent readback | the bench's pictures byte-identical; the layer silent |

State 3 for each component is a later step of its own, after golden outputs (§6.6), at the tester's word.

### 7.2 The estimate

In lines of Rust, argued from the C# each part replaces; each step reports the real figure beside this one:

| Component | Rust, with its tests | C# removed at state 3 | C# kept |
|---|---|---|---|
| Galaxia | about 3,000 (1,100 in 2a, 1,900 in 2b) | about 800, replaced by forwarding | about 800: the models, the enumerations, the contract, the sink; plus about 300 of facades |
| Endymion | about 2,000 (900 in 3a, 1,100 in 3b) | about 1,250 | about 150: `DefaultPadKeyMap` and the binding map's C# surface; plus about 250 of facades |
| Serenity's half | about 3,500 (1,300 in 4a, 2,200 in 4b) | about 2,100 | `SlangRunner`, 131; plus about 250 of facades |

The largest single risk is not size but §5.2: the JSON writer is small and its parity is all edge cases, which is why
2b is a step of its own.

### 7.3 What DianaOS will need from them

- **From Galaxia, nearly all of it:** the tree and its overrides (the shell is rooted at `DataStore.UsrHome`, `man
  hier`), `SaveLibrary` for states and battery files (`EmuSen_CoreAPI.md` §8.2's "States and rewind", "Firmware and
  battery"), `GraphicsConfig.Consoles` for settings layered system → engine → game, and the per-game files under
  `/etc/EmuSen/games/` that §8.5 there plans, `CheatFile` for `cheat save` and `cheat load`, `DataMigration` (called
  from `EnsureSkeleton`), the logs and crash records, `RomHash`, and the catalogue's contract. It will also need what
  Galaxia does not yet have: models it edits itself, at which point §5.5's dictionary order becomes its problem.
- **From Endymion, the logic half:** the input vocabulary, `PadControls`' rules and `PortRouter`'s (mirroring, the
  keyboard's player, which ports hold a controller), because routing is the runtime's. Not the devices, the
  resampler or the rate control, which are a client's.
- **From Serenity's half, nothing:** a headless runtime draws nothing. It serves the Rust client that §6 step 5 of
  the stack page brings.

---

## 8. Predictions to be retired

Stated before anything is built; each step reports which held.

- **P1.** A tier-1 load through Rust, including the C# bind of the canonical JSON, costs under 0.1 ms for
  `appsettings.json` (2,939 bytes in the development tree) on the desktop, so the doubled parse is invisible.
- **P2.** No frontend calls Galaxia's path API once a frame; a call counter over 600 frames of play finds none. If
  one exists, the facade caches its answer against an override generation counter.
- **P3.** The first parity differences of 2b appear in §5.2's rows and nowhere else.
- **P4.** Endymion's submit makes one P/Invoke instead of two, and its cost per frame is below measurement either
  way.
- **P5.** The Rust chain's render path allocates nothing managed per frame, against 15–118 KB in `Bind` today, and
  the gen-2 collection in N64 4× royale goes; GPU time does not change.
- **P6.** Shaderc loaded from the same file yields byte-identical SPIR-V for all 1,350 of the pack's passes.
- **P7.** The platform library is under 4 MB in the `dist` profile on linux-x64 with all three crates in it.

---

## 9. Decisions, 2026-10-07

The twelve questions this page first left open were decided by the tester on the day it was written, each as it
recommended and with nothing changed; all three components are to be ported as planned, and DianaOS after them.

- **Q1.** Decided 2026-10-07: the crates live in a Cargo workspace at `Platform/`, at the top of the repository (§2.1).
- **Q2.** Decided 2026-10-07: one library, `emusen_platform`, for every ported component (§2.2).
- **Q3.** Decided 2026-10-07: each component passes through the three states of §3.9; its C# stays as oracle and
  fallback until its gate, and is deleted only after its golden outputs are recorded (§6.6) and the tester agrees.
- **Q4.** Decided 2026-10-07: after a component's gate, a build without cargo fails unless it is given a prebuilt
  library, with a message naming `EmuSenNativePrebuilt` (§2.5).
- **Q5.** Decided 2026-10-07: System.Text.Json's words inside a diagnostic are replaced by Rust's own; Galaxia's own
  messages are kept exactly (§5.4).
- **Q6.** Decided 2026-10-07: .NET's reading quirks are reproduced exactly, each with a corpus case (§5.3).
- **Q7.** Decided 2026-10-07: the third-party crates are `ash`, `libloading`, `rusqlite` (bundled), `sha2` and
  `md-5`; the JSON reader and writer are written by hand (§2.4).
- **Q8.** Decided 2026-10-07: Endymion's device half is ported now, in step 3b, for the oracle's sake, and no gain
  is claimed for the C# programs (§4.2).
- **Q9.** Decided 2026-10-07: Serenity's half stays third whatever P5 shows; if P5 fails, the step is recorded as
  buying nothing until a Rust client exists (§4.3).
- **Q10.** Decided 2026-10-07: the lent readback is accepted as the interface's one pointer out (§3.3).
- **Q11.** Decided 2026-10-07: the interface is versioned by exact match, with no baseline guard beyond the export
  test (§3.2).
- **Q12.** Decided 2026-10-07: `EmuSen_Stack.md` §2.1's paragraph on `AudioPlayer.cs` is retired, and §4.2's count of
  calls stands in its place there.

---

## 10. What was built, 2026-10-07: step 2a, the library and Galaxia's tree, names and bytes

Everything here was measured on the development desktop (Fedora 44, x86-64, .NET 10, Rust 1.98.1). Nothing has run
on Windows or macOS; §10.7 says what that leaves open.

### 10.1 The artefacts

| What | Where |
|---|---|
| The workspace, with the cores' `release` and `dist` profiles | `Platform/Cargo.toml`, `Cargo.lock` |
| `emusen-galaxia`: `tree` (`ConfigRoot`, `ConfigStore`, `DataStore`), `saves` (`SaveLibrary`), `atomic` (`AtomicFile`), `migration` (`DataMigration`), `rom_hash` (`RomHash`), and `dotnet_path`, .NET's path and string rules | `Platform/galaxia/src/` |
| `emusen-platform`: the cdylib, its library-level calls and Galaxia's C layer; 22 exports | `Platform/platform/src/` |
| The interface's specification | `Platform/include/emusen_platform.h` |
| The build, and whether the library is required | `Platform/Platform.targets`, `Platform.props`, imported by `EmuSen.Galaxia.csproj` |
| The loader and the facade's native half | `EmuSen.Galaxia/Native/PlatformLibrary.cs`, `GalaxiaNative.cs` |
| The seven classes, each split into a forwarding surface and its kept C# | `ConfigRoot.cs`, `ConfigStore.cs`, `Library/DataStore.cs`, `SaveLibrary.cs`, `AtomicFile.cs`, `DataMigration.cs`, `RomHash.cs` |
| The parity class | `EmuSen.WiseMan/Galaxia/GalaxiaParityTests.cs` |

About 1,470 lines of Rust without their tests and 730 of tests, against §7.2's 1,100 for the step; `dotnet_path`,
which §7.2 did not foresee as a module of its own, is the difference (§10.3). The one third-party crate is `md-5`
0.11 with its seven dependencies, listed in `THIRD_PARTY_NOTICES.md` §1.4.

### 10.2 The facade, and how the default was kept untouched

Each of the seven classes keeps its public members. Each member is one line: the library when
`GalaxiaNative.Active`, else `Managed`, a nested class holding the C# that was there before. The C# was moved, not
rewritten: a comparison of every statement of the seven files at `8645e821` against the `Managed` classes finds all
of them verbatim except where a name gained its `Managed.` qualifier, an override became its backing field, or a
default argument stayed on the public member. `AtomicFile` and `RomHash` are verbatim whole.

**`Active` is false unless `EMUSEN_GALAXIA_NATIVE` is exactly `1` and the library loads** (state 1 of §3.9). The
variable is read once, before anything touches the library, so with it unset the library is never opened. That was
checked on a running program and not only read from the code: the standalone DianaOS shell, run under the dynamic
loader's own trace (`LD_DEBUG=files`) with `hier` and `ls /Saves`, opens `libemusen_platform.so` no times with the
variable unset or `0`, and opens it with `1`; its output is the same bytes in all three runs.

**Three things the design left open, as built:**

- **Diagnostics are a queue** (§3.5): one migration reports once for each file it leaves, so the messages of a call
  wait on the calling thread and the facade drains and reports them, in order, before it returns.
- **Statuses that are answers.** `ABSENT` (−1030) stands for .NET's `null`: no file, no seed directory, no directory
  name. It is not a failure and leaves no words. `NOT_FOUND` and `ACCESS` exist so that the facade can throw what the
  C# threw: `RomHash.Md5` on a missing file is a `FileNotFoundException`, on a missing folder a
  `DirectoryNotFoundException`, on a directory an `UnauthorizedAccessException`, and a migration over a directory it
  may not list an `UnauthorizedAccessException`, each as before and each held by a parity case.
- **The overrides live on both sides.** The C# keeps its three fields, which is what `Managed` reads; when the
  library is in the process each setter also tells it, under one lock that the loader takes when it first hands the
  library the values already set. So the library can be loaded late (by a test that only compares) and still agree.

### 10.3 .NET's rules, reproduced and measured

The tree is built from strings, and a path one character different names a different file. So `dotnet_path` does not
use `std::path`, whose rules are Rust's: it reproduces `Path.Combine`, `GetFileName`,
`GetFileNameWithoutExtension`, `ChangeExtension`, `GetDirectoryName`, `TrimEndingDirectorySeparator` and
`GetFullPath` as text functions, with the platform's style a parameter so the Windows rules are unit-tested on Linux.
What was measured against .NET 10 while writing it, each now a test:

| Rule | .NET | What an ordinary Rust port would do |
|---|---|---|
| `Path.GetFullPath("/a/b/")` | keeps the final separator; never follows a link | `canonicalize` drops it and resolves links |
| `Path.GetDirectoryName("a")`, `("/")` | `""`, `null` | `Path::parent` gives `""`, `None`; but `"/a//b//c"` gives `/a/b` in .NET, runs collapsed |
| `Path.GetFileNameWithoutExtension(".hidden")` | `""` | `file_stem` gives `.hidden` |
| `Path.ChangeExtension("/s/", ".png")` | `/s/.png` | `with_extension` gives `/s.png` |
| `File.Exists` on a dangling link | `true` | `Path::is_file` gives `false` |
| `Directory.EnumerateFiles(…, AllDirectories)` | walks into a linked directory; lists a dangling link as a file | a walk that does not follow links misses both |
| `File.Copy` | keeps the permissions and the modification time, to the nanosecond | `fs::copy` keeps the permissions and not the time |
| `string.IsNullOrWhiteSpace` | Unicode's White_Space, 25 characters in the first plane; **not** U+001C–U+001F, U+180E or U+200B | the same, as it turned out |
| `string.Equals(…, OrdinalIgnoreCase)` | one character to one, by Unicode's simple upper-case mapping, with the dotless i (U+0131) and the long s (U+017F) equal only to themselves | `to_uppercase` is the full mapping, and sends `ſ` to `S` |
| `SpecialFolder.ApplicationData` on Linux | `/home/<user>/.config` | — |

**A prediction of this step that was wrong,** kept because it was acted on: the first `is_white_space` added
U+001C–U+001F to Rust's set, from the belief that .NET counts them. It does not; the measurement printed .NET's set
and the addition was removed before any test ran against it.

**What is argued and not measured:** every Windows rule (`C:` roots, UNC, `\\?\`), written from the .NET runtime's
published source and unit-tested for self-consistency only; and `Path.GetFullPath` on Windows, where .NET asks the
system and this asks `std::path::absolute`, which asks the same system call.

### 10.4 `AtomicFile`: the same guarantee, no more and no less

Read from `AtomicFile.cs` and stated in `EmuSen_Galaxia.md` §4.2, which this step added: the directory is made, the
bytes are written whole to `<path>.tmp`, and the temp file is renamed over the live one. **Rename over, and no
`fsync`**, of the file or of its directory. That is atomic against an interrupted write, a crash or a kill, and is
not a promise against power loss.

The Rust does exactly that: `create_dir_all`, `fs::write` to the temp name, `fs::rename`. No flush was added. An
`fsync` would be a real improvement to argue for, and it would also be a change of behaviour on a path the cores take
every 300 frames, on a handheld's storage; it is left as a decision not taken, to be made in both implementations
at once and with a measurement, rather than slipped in under a port. Held by the parity class: the same bytes, the
same permissions on the file and on the directory made for it, no temp file left after a success, and after a write
over a path that is a directory the same stranded temp file on both sides.

### 10.5 The parity class, and what it found

`GalaxiaParityTests` calls `Managed` and the library side by side, whichever the switch chose, so it means the same
thing in both runs. Its cases:

- every path rule over 3,000 seeded strings built from awkward parts; the caseless and blank rules over every
  character of the first plane and the second;
- `File.Exists` and `Directory.Exists` over files, directories, links of three kinds and trailing separators;
- the root from about 300 seeded places and 17 built ones, for both platforms, with each branch asserted to be among them;
- every directory under all 125 combinations of the three overrides;
- every save name over 416 ROM names, 8 directory overrides, 8 slots and 7 consoles, with and without a data
  override;
- files of seven lengths written by each side and read by the other; what will not read, and whether it is said;
- a migration of one built tree by each side, compared file by file with bytes, permissions and times, with the ROM
  folders listed before and after; the declined migrations; a directory that will not list; the program's own three
  migrations with both destinations redirected into the test's folder;
- the hash over nine lengths around the block and buffer sizes, and the exceptions for what cannot be hashed;
- the header against the facade's export table.

No test here writes outside its own temporary folder, and none names the ROM library: the migration cases build
their own `Games` and `Roms` and assert they are untouched.

**Its first run failed four cases, and two of the four were defects in the Rust:**

1. **Caseless equality differed for 110 characters.** The first version upper-cased with `char::to_uppercase` and
   took the result when it was one character. That equated `ſ` with `S`, which .NET does not, and failed to equate
   the Greek letters with a subscript iota (U+1F80–U+1FF3) with their capitals, because Rust's mapping for them is
   two letters and .NET's is one. It decides whether a migration's two roots are one tree and whether a directory is
   a `.app`. Fixed, and now compared for every character of two planes.
2. **A dangling link was absent without a word.** `AtomicFile.TryRead` on one reports why it could not be read; the
   library answered `ABSENT` at the length query, before trying. Fixed: what `File.Exists` says is there is given a
   length, so the read is tried.
3. Two were the tests' own: an assertion that .NET's words end in a full stop (they do not always), and two trees
   built a millisecond apart compared by their times.

**Seeded faults.** Twenty-one were made in the Rust: twenty judged by the parity class and fifteen by the crates'
own tests, fourteen of them the same. The parity class catches eighteen of its twenty. The two it does not are equivalent on this host and are held
by a unit test or by nothing: an unwound `..` with no part before it arises only for a Windows drive-relative path
(caught by `dotnet_path`'s Windows cases), and the rule that a path ending in a separator is never a file is already
what Linux answers, so removing it changes nothing here (it survives both suites, and is kept for Windows, where it
is unverified). The crates' own suite first let four of its fifteen through; three were cases it lacked and now has.

### 10.6 Building, publishing and CI

- **Building.** `dotnet build` of any project builds the library once, through Galaxia, into
  `EmuSen.Galaxia/obj/emusen_platform/`, and MSBuild carries it beside every program's assemblies. With no `cargo`
  the build warns and goes on, and with `-p:EmuSenPlatformRequired=true` it fails with the message of §2.5 unless a
  prebuilt library is given; both were run, with a cargo that does not exist.
- **Publishing.** A linux-x64 publish of Mistress gains exactly one file, `lib/EmuSen/libemusen_platform.so` (about
  0.5 MB; it needs glibc 2.34, as the cores' libraries do, and links nothing but libc). A publish for another
  platform says when the library is absent, takes it from `EmuSenNativePrebuilt/<rid>/` when given, and fails when
  it is required and missing; the three were run for win-x64.
- **CI.** `rust-cores.yml` tests the workspace on the three systems, compiles the header as C99 and C++17, builds the
  library for the four triples, and runs Galaxia's tests both ways. **None of it gates yet**: every platform step is
  marked to report and not to stop the cores' jobs, because none has run on a Windows or macOS runner and the
  library is optional in state 1. They become gates with Galaxia's own.

### 10.7 What was tested, and what was not

| Run | Result |
|---|---|
| `cargo test` in `Platform/` | 41 pass (32 in `emusen-galaxia`, 9 in `emusen-platform`) |
| The header as C99 and as C++17, `-Wall -Wextra -pedantic -Werror` | clean |
| WiseMan's Galaxia classes with `DianaOSSandboxTests` and `HierTests`, variable unset | 204 pass |
| The same, `EMUSEN_GALAXIA_NATIVE=1` | 204 pass |

The 204 are the 173 that existed and the parity class's 31.

**Not done, and not claimed:**

- **Windows and macOS.** Nothing here ran on either. The Windows path rules are argued (§10.3); a case-insensitive
  filesystem, the macOS bundle on a Mac, and .NET's `ApplicationData` on macOS (§5.5's correction) are untested.
- **P2**, whether any frontend asks Galaxia for a path once a frame, is not measured. It matters only once the
  switch defaults to Rust, and belongs to the gate.
- **P7**: the library is 0.5 MB with this half of Galaxia in it. The prediction is for all three components.
- **The words of an I/O failure.** A diagnostic's frame is Galaxia's and is kept exactly (`<path>: … Left in
  place.`); the words inside it were .NET's exception message and are now the system's as Rust reports them ("No such
  file or directory (os error 2)."). §9's Q5 decided this for System.Text.Json's words; it is applied here to
  .NET's I/O words by the same reasoning, and is named so that the tester can say otherwise.
- **The order of a migration's diagnostics** follows the walk's, which is breadth-first as .NET's was measured to
  be, and within a directory the system's; the parity class compares them sorted.
- `ConfigFile<T>`, the models, `ErrorLog` and the suggestion text are step 2b and are C# only.
