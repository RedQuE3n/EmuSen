# EmuSen — the platform in Rust: Galaxia, Endymion and Serenity's Vulkan half

*This revision: the twelfth, 2026-10-10: half a surrogate pair and a NUL in a list asked of every Galaxia facade, 28 found handing such text to a binding that refuses it and now answering it with their C# through one rule, `GalaxiaNative.Takes`, with the reachability measured on Linux (§21). The eleventh, the same day: `GameCheats`' naming of a game's own cheat list given its Rust half, a new export and interface 7, with its parity case and seeded faults (§20). The tenth, the same day: Endymion and Serenity's slang half moved to state 2, the library their default and their C# one variable away, through Galaxia's switch (§19). The ninth, the same day: the four defects step 4b's parity class found in the C# slang runtime shown and fixed in the C#, each by a test that failed first (§18); Q27 to Q30 decided (§9). The eighth, the same day: step 4b built, Serenity's Vulkan device and chain in Rust behind the same switch, still off, with the lent readback, the bench's pictures the same from both and the validation layer silent on the library's device (§17); Q25 and Q26 decided (§9). The seventh, 2026-10-09: step 4a built, Serenity's presets, sources, parameters, compiler, reflection and SPIR-V cache in Rust behind a switch of their own that is off, over the Shaderc and the SQLite the program already ships (§16). The sixth, 2026-10-08: step 3b built, Endymion's device half (the audio players, the pad manager's devices and the simulated pads) in Rust behind Endymion's switch, still off (§14), with the open items gathered in §15. The fifth, the same day: step 2c, Galaxia moved to state 2, the library its default and its C# one variable away (§13). The fourth, the same day: step 3a built, Endymion's resampler, rate control, port router and seat rules in Rust behind a switch of their own that is off (§12). The third, the same day: step 2b built, config files, the models, the error log and the suggestion text behind the same switch (§11), which completes Galaxia's code; its gate is step 2c. The second, 2026-10-07: step 2a, the library and Galaxia's tree, names and bytes (§10). The first, the same day, was a design with nothing built. It plans the
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
2. **Cauldron is folded into DianaSh, not ported.** Its reason to exist as an assembly ended when LunaP left the
   repository (`EmuSen_LunaP.md` §19), and in the Rust plan telemetry belongs to the core ABI (`EmuSen_CoreAPI.md`
   §6.14). §4.4 records the fold.
3. **DianaSh comes after these three.**
4. **Mistress, Hotaru and Pharaoh stay C#** and keep working unchanged. Each ported piece sits behind a C interface
   that the C# side calls through P/Invoke, as the Rust cores already do. Nothing may break partway.
5. **The user-interface toolkit (Qt against Slint) is not part of this work** (`EmuSen_Stack.md` §6).

### 0.2 The design in one paragraph

The three components become Rust crates in one Cargo workspace at the top of the repository, `Platform/`, linked
into **one** native library, `emusen_platform`, which every C# program ships beside its assemblies (§2). Each crate
has a Rust API first, which a Rust DianaSh will call directly, and a thin C layer over it, which the existing C#
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
- `EmuSen_CoreAPI.md` §8.2, "Telemetry stays in Cauldron, which DianaSh already references", is revised by the fold
  (§4.4).

---

## 1. What there is to port (measured 2026-10-07)

| Component | C# lines | References | Packages | Consumers |
|---|---|---|---|---|
| `EmuSen.Galaxia` | 1,603 in 26 files | none (the leaf) | none | every assembly; 337 C# files name `EmuSen.Galaxia`, 198 of them in WiseMan |
| `EmuSen.Endymion` | 1,445 in 14 files | Galaxia | SDL3-CS, SDL3-CS.Native, Avalonia (base) | Mistress, Hotaru, WiseMan |
| `EmuSen.Serenity`, `Slang/` only | 2,225 in 10 files, of 5,466 | Galaxia, Cauldron, LunaP | Silk.NET.Vulkan, Silk.NET.Shaderc, Microsoft.Data.Sqlite with its SQLite bundle, and five of Avalonia and Skia | `GameFrameControl`, Mistress's Shaders window, WiseMan |
| `EmuSen.Cauldron` | 327 in 6 files | none | none | DianaSh, EmuSen, Serenity, WiseMan |

**How Galaxia is reached.** The path API (`ConfigRoot`, `ConfigStore`, `DataStore`, `SaveLibrary`, `AtomicFile`) is
called from 7 files of `EmuSen`, 18 of Mistress, one each of DianaSh, Hotaru, Pharaoh and Serenity, and 120 of
WiseMan. `ConfigFile<T>` is instantiated outside Galaxia for three types whose vocabulary Galaxia cannot name:
`Dictionary<…, SDL.GamepadButton>` (`EmuSen.Endymion/Input/GamepadBindings.cs`),
`Dictionary<…, Avalonia.Input.Key>` (`EmuSen.Mistress/Input/ControllerKeyBindings.cs`) and
`Dictionary<HotkeyAction, Key>` (`EmuSen.Mistress/Input/HotkeyBindingMap.cs`). That distinction shapes §4.1.

**Where SDL is called.** Every SDL function call in the tree is inside Endymion (`AudioPlayer.cs`, `UiSoundPlayer.cs`
and the pad files). Mistress names SDL's enumerations (`SDL.GamepadButton`, `SDL.GamepadType`) for its diagrams and
bindings, and calls no SDL function.

**What tests hold the components.** `EmuSen.WiseMan/Galaxia/` (13 classes, 1,735 lines), with
`DianaSh/HierTests.cs` and `DianaSh/DianaShSandboxTests.cs`; `Audio/` (five classes) and `Input/` (nine) for
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

*Added 2026-10-15 (§26):* `dotnet/` (`emusen-dotnet`, .NET's text, number and file rules, split from Galaxia's crate),
`dianash/` (`emusen-dianash`, DianaSh's shell core, with its two headers and baseline), `dianash-module/` (what a
command module is written with), `dianash-text/` (the `dianash.text` module) and `dianash-program/` (the `dianash`
program); the library exports DianaSh's host API besides the platform's interface.

**Not `EmuSen/Cores/Shared/`.** That folder holds the crates cores link (`emusen-native`, the ABI check, the
conformance kit). A core does no file I/O and calls nothing above it (`EmuSen_CoreAPI.md` §3's table), so a platform
crate beside them would be one `path =` away from a core depending on the platform, which the layering forbids.
Keeping them apart makes that dependency visible in a diff.

**Not inside each C# project** (`EmuSen.Galaxia/rust/`). The C# projects are retired at the end of `EmuSen_Stack.md`
§6, and the crates are not; a crate inside a project folder would also put cargo's `target/` under an MSBuild glob,
which `EmuSen.csproj` already needs five `DefaultItemExcludes` patterns to keep out of the cores' folders. A
top-level folder outside every project has neither problem, and a Rust DianaSh joins it as `Platform/dianash/`.

**One workspace, unlike the cores.** Each core is its own crate with its own `Cargo.lock`, because a core is shipped
and versioned alone. The platform crates are one program, which a Rust DianaSh will link as one: one lock file keeps
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
- **It is the shape of the end state.** A Rust DianaSh links the three crates into one runtime.

The cost: a program that uses only Galaxia ships a library containing Endymion's and Serenity's code. Neither links
SDL, Shaderc or Vulkan at build time (§2.8), so the cost is file size, a few megabytes (argued; measured in step 2a).

### 2.3 A Rust API first, a C layer over it

Each component crate exposes an ordinary Rust API (`galaxia::DataStore::saves()`, `galaxia::config::load::<AppSettings>()`,
`serenity::Chain::build(...)`), with Rust types, `Result` and ownership. The C layer lives only in `emusen-platform`,
one module per component, and does nothing but convert: pointers to slices, statuses to and from `Result`, models to
and from JSON. This is the arrangement of `emusen-native`'s `core` trait and its `core_exports!` macro
(`EmuSen_CoreAPI.md` §18.2), and it is what lets a Rust DianaSh use the crates without passing through C.

### 2.4 Dependencies

The cores are almost dependency-free by habit (`emusen-native` has none). The platform needs bindings to C libraries
the C# side already binds, and two hashes:

| Crate | For | Licence |
|---|---|---|
| `ash` | Vulkan, loaded at run time | MIT or Apache-2.0 |
| `libloading` | SDL3 and Shaderc, loaded at run time from the shipped files (§2.8) | ISC |
| `rusqlite` with `bundled` | the SPIR-V cache (§4.3). *Not taken: Q25, decided 2026-10-09, has the cache call the SQLite the program ships (§16.2).* | MIT (SQLite itself public domain) |
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

Since 2026-10-16 a run builds only the libraries whose inputs no earlier build had, and takes the rest from that
earlier build (§28).

### 2.8 Native libraries the platform loads, and the one it brings

- **SDL3 and Shaderc are loaded by path from the program's directory**, the files `SDL3-CS.Native` and
  `Silk.NET.Shaderc.Native` already ship, through `libloading` and hand-written declarations of the functions used
  (29 of SDL's, counted from Endymion's call sites, and the 14 of Shaderc's that `SlangCompiler` calls; **measured**). The binaries do not
  change, so behaviour that depends on them cannot: Shaderc's SPIR-V is the same bytes (§8, P6), and SDL's pad
  mappings are the same database. The C# facade passes the directory; the dynamic loader returns the same instance
  for the same file, so a C# fallback and the Rust side in one process share one SDL (argued from `dlopen`'s
  reference counting; checked in step 3b). *As built in step 3b, the C# lends the library the handle of the SDL it
  has loaded, which is one instance by construction and was checked; loading by path is kept for a Rust program
  with no C# beside it (§14.2).*
- **Vulkan's loader** (`libvulkan.so.1`) is the system's, as it is for Silk.NET; one loader, separate instances.
- **SQLite is the exception: `rusqlite` brings its own copy**, and `EmuSen` already ships `e_sqlite3` through
  `SQLitePCLRaw`. Two SQLite libraries in one process may each open *different* files safely; opening the *same*
  file from both breaks POSIX advisory locking, because closing one copy's descriptor releases the other's locks
  (SQLite's own documentation lists linking two copies into one application as a way to corrupt a database). The
  rule is therefore: **one database file, one SQLite copy, per process.** Only Serenity's cache moves, and it moves
  whole (§4.3); `games.db`, `records.db`, `media.db` and the catalogue stay with `e_sqlite3`. The parity tests of
  step 4a open the C# and the Rust cache on separate files. *As built in step 4a and decided on 2026-10-09 (Q25),
  the library brings no SQLite: it calls the `e_sqlite3` the C# has loaded, lent by its handle, so the process
  holds one copy and this paragraph's hazard does not arise (§16.2). A Rust program with no C# beside it will need
  a SQLite of its own.*
- *Added by step 4b:* **Vulkan's loader is opened through `libloading`**, by the name each system knows it by, and
  `ash` is taken without its own loader, so the library carries one version of `libloading` and no second (§17.2).

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
and it is transitional, ending when the C# does. A Rust DianaSh calls the crates' Rust API (§2.3), and the stable
interface a non-Rust client would use is DianaSh's frontend API (`EmuSen_CoreAPI.md` §9.2), which this does not
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
  surrogate cannot be encoded and is refused with `EMUSEN_BAD_STRING` (−5) before the call. *As built, the refusal is
  the binding's `EncoderFallbackException` and no export is called; each facade must choose the C# for such text,
  through `GalaxiaNative.Takes` (§21).*
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
  build), `NO_DEVICE` −1028 (no Vulkan device or audio device), `COMPILE` −1029 (a shader). *As built by step 4b
and decided on 2026-10-10 (Q30), −1028 is `DEVICE`, for no Vulkan device and for a call a device refused alike,
since the host does one thing with either.* *As built by step 2b
  (§11.2), `PARSE` covers both a text that is not JSON and one its model does not allow, since the host does one
  thing with either; −1026 is unassigned.* *Added by step 2a (§10.2):*
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

**Where each stands, 2026-10-10:** all three are in state 2: Galaxia since step 2c (§13), Endymion, both its halves
under the one variable, and Serenity's slang half, the device and the chain with it, since §19. None is in state 3.

---

## 4. Per component: what moves, what C# keeps, what it buys

### 4.1 Galaxia

**What moves to Rust:**

- **The tree:** `ConfigRoot` (the walk to `EmuSen.sln` or `.dianashroot`, the macOS bundle rule, the seed directory;
  `EmuSen_Galaxia.md` §3.4), `ConfigStore`, `DataStore`, both overrides and the legacy directory.
- **The names:** `SaveLibrary`, all five rules (`EmuSen_Galaxia.md` §5); and, *added 2026-10-10*, `GameCheats`' name
  for a game's own cheat list (`EmuSen_Config_Reference.md` §3.4b, here §20).
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
- `CheatFileWrite`'s hex helpers (`TryHex`, `Strip`, `TryParseNumbers`), which DianaSh's C# `CheatRegistry` calls per
  entry; they move with DianaSh.
- `SuggestingEnumConverter`, for tier 2 only.

**What does not move in this port:** `games.db` (schema 6), `records.db` and `media.db` are Mistress's, with their
migration lists in `EmuSen.Mistress/Library/` (`GameRecords.cs`, `FileRecords.cs`); Mistress stays C#, so they stay
with it, and `EmuSen_CoreAPI.md` §3 places the library with the clients in any case. The catalogue's driver
(`EmuSen/Common/Catalogue/SqliteCatalogue.cs`) stays in `EmuSen` and joins DianaSh when DianaSh moves.

**What it buys, plainly.** The on-disk formats, their parsing and validation, the migrations, the atomic writes and
the save library's paths become Rust that a Rust DianaSh uses directly, through §2.3's API, with nothing to rewrite.
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

**What DianaSh will need from it** is the logic half only: input routing is the runtime's (`EmuSen_CoreAPI.md` §8.2:
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

**What it buys.** For DianaSh, nothing: presentation is a client's (`EmuSen_CoreAPI.md` §3). For the C# programs, one
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

Its six files move into `EmuSen.DianaSh`, keeping the `EmuSen.Cauldron` namespace so no caller changes; Serenity
references DianaSh in its place; the project and its four `ProjectReference`s go. `EmuSen_Cauldron.md` becomes a
section of the DianaSh documentation, keeping its §3.1 rule on what belongs to telemetry and what to the debugger.
That is step 1b. *Done 2026-10-07: the files are in `DianaSh/Lib/Cauldron/` and the page is
`EmuSen_Debugging_Tools_Reference_v5.md` §3.66, with the split rule at §3.66.3.1 and the fold's own record at §3.66.1.1.* In the Rust plan the read surface it defines is the core ABI's debug interface
(`EmuSen_CoreAPI.md` §6.14) seen through DianaSh, so a separate Rust crate for it would be a boundary nothing uses
(`EmuSen_Multicore.md` §9.2).

---

## 5. Formats, and the parity rule

### 5.1 What must not change

- **Config JSON**: every file under `/etc/EmuSen`, the `cheats/` category, and the legacy files under
  `~/.config/EmuSen` read for migration.
- **Saves and states**: `.srm`, `.state` and their names; their bytes are a core's, and Galaxia passes them through.
- **Cheats**: `cheats/<game>.json` (`CheatFile`); the `.cht` database is DianaSh's and is not touched here.
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
  end. That matters only once a Rust DianaSh edits `GraphicsConfig`, and is recorded here for that step.
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

## 7. Order, estimate, and what DianaSh will need

### 7.1 The steps

| Step | What | Its gate |
|---|---|---|
| 1a | this page | the tester's go |
| 1b | Cauldron folded into DianaSh (C#) | every frontend builds; the telemetry, coretop and dashboard tests |
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

### 7.3 What DianaSh will need from them

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
  way. *Held, 2026-10-08 (§14.6).*
- **P5.** The Rust chain's render path allocates nothing managed per frame, against 15–118 KB in `Bind` today, and
  the gen-2 collection in N64 4× royale goes; GPU time does not change. *Held, 2026-10-10, as to what it can be held
  to (§17.6): the chain allocates nothing managed, and a frame's 312 bytes are the picture handed to Skia; no
  collection of the second generation came in any run of either chain, so its going was not seen; GPU time agreed.*
- **P6.** Shaderc loaded from the same file yields byte-identical SPIR-V for all 1,350 of the pack's passes. *Held,
  2026-10-09 (§16.6).*
- **P7.** The platform library is under 4 MB in the `dist` profile on linux-x64 with all three crates in it. *Held,
  2026-10-10: 1,445,832 bytes in a publish (§17.6).*

---

## 9. Decisions, 2026-10-07

The twelve questions this page first left open were decided by the tester on the day it was written, each as it
recommended and with nothing changed; all three components are to be ported as planned, and DianaSh after them.

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

Three more, raised by step 2a (§10.6, §10.7) and decided before step 2b:

- **Q13.** Decided 2026-10-08: Q5's rule holds for .NET's I/O words as for System.Text.Json's. A diagnostic's frame is
  Galaxia's and is kept exactly; the words inside it are the system's as Rust reports them.
- **Q14.** Decided 2026-10-08: the platform's CI steps report and do not gate until Galaxia's gate (§10.6).
- **Q15.** Decided 2026-10-08: §5.5's correction stands as marked: the host passes .NET's own `ApplicationData` to the
  library, and the claim about macOS is withdrawn until it is run there.

Seven more, raised by steps 2b (§11.8), 3a (§12.7) and 2c (§13.4):

- **Q16.** Decided 2026-10-08: a model in Rust is a schema and a bound document, as built; structs can be generated
  later (§11.2).
- **Q17.** Decided 2026-10-08: `StateRecord`'s sidecar stays in C#, as an import that ends with the files it reads.
- **Q18.** Decided 2026-10-08: the error log prunes by instants where the C# compares local times (§11.4).
- **Q19.** Open: what text that is not valid UTF-16 becomes once the C# that answers for it is gone. It is decided
  with that C#'s retirement and not before (§11.2, §15).
- **Q20.** Decided 2026-10-08: `GamepadBindings` stays in C#, as built (§12.2).
- **Q21.** Decided 2026-10-08: the Rust refuses a NaN or infinite resample ratio, for which the C# never returns
  (§12.3).
- **Q22.** Decided 2026-10-08: CI's Galaxia and platform steps keep reporting and do not gate yet (§13.3).

Two more, raised by step 3b (§14.7):

- **Q23.** Open: a device layer the library does not know, once Endymion has no C# to run it. It belongs to
  Endymion's state 3 and is decided then (§15).
- **Q24.** Decided 2026-10-08: SDL lent by its handle stands as built, with loading by path kept in the crate for a
  Rust program with no C# beside it (§14.2).

Two more, raised by step 4a (§16.7):

- **Q25.** Decided 2026-10-09: the cache calls the `e_sqlite3` the program ships, lent by its handle, as built. This
  changes Q7, which named `rusqlite` with SQLite bundled, and the reason is §2.8's own: two copies of SQLite in one
  process cannot see each other's locks on a file, and the cache's file is opened by both the C# and the library in
  one process (§16.2). What follows is recorded with it: a Rust program with no C# beside it has no `e_sqlite3`
  lent to it and will need a SQLite of its own, opened by path or built in, which is decided when such a program
  exists (§15).
- **Q26.** Decided 2026-10-09: the compiler's identities stay separate for now, as §4.3 planned, so neither
  implementation serves the other's rows. It is looked at again when Serenity moves to state 2 (§15).
  *Decided again 2026-10-10, after §19.4's measurement: they stay separate until the C# is retired, which ends the
  question. A machine compiles each preset once more after an update, 60 to 130 ms a Mega Bezel preset on the desktop.*

Four more, raised by step 4b (§17.7):

- **Q27.** Decided 2026-10-10: the host decodes lookup images with Skia and hands the pixels over, as built. What
  decodes them with no C# beside the library is decided at Serenity's retirement.
- **Q28.** Decided 2026-10-10: the four differences found in the C# chain and device are fixed now, in the C#, each
  shown by a test before it is fixed (§18). They are the one exception to the C# being moved and not rewritten.
- **Q29.** Decided 2026-10-10: Vulkan's own handles for the instance and the device stay exported, as built.
- **Q30.** Decided 2026-10-10: one status, `DEVICE` −1028, for no device and for a call a device refused, as built;
  §3.5 is corrected to match.

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

**`Active` is false unless `EMUSEN_GALAXIA_NATIVE` is exactly `1` and the library loads** (state 1 of §3.9; since
step 2c, state 2, §13). The
variable is read once, before anything touches the library, so with it unset the library is never opened. That was
checked on a running program and not only read from the code: the standalone DianaSh shell, run under the dynamic
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
| WiseMan's Galaxia classes with `DianaShSandboxTests` and `HierTests`, variable unset | 204 pass |
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
  file or directory (os error 2)."). §9's Q5 decided this for System.Text.Json's words; it was applied here to
  .NET's I/O words by the same reasoning. *Decided so on 2026-10-08 (§9, Q13).*
- **The order of a migration's diagnostics** follows the walk's, which is breadth-first as .NET's was measured to
  be, and within a directory the system's; the parity class compares them sorted.
- `ConfigFile<T>`, the models, `ErrorLog` and the suggestion text are step 2b and are C# only.

---

## 11. What was built, 2026-10-08: step 2b, config files, the models, the error log and the suggestion text

Measured on the development desktop, as §10 was (Fedora 44, x86-64, .NET 10, Rust 1.98.1). Nothing has run on Windows
or macOS. With this step every rule of Galaxia has a Rust form behind the switch; none of its C# has been removed.

### 11.1 The artefacts

| What | Where |
|---|---|
| JSON as System.Text.Json reads and writes it: the reader, the escaping, the number layout, and `File.ReadAllText`'s decoding | `Platform/galaxia/src/json.rs` |
| A model as data, the binder and the writer | `model.rs` |
| The four models and the five classes they hold, the settings upgrade, the two values a cheat computes | `models.rs` |
| `ConfigFile<T>`: reading, the copy from before Galaxia, saving, deleting | `config.rs` |
| The cheat lists as files: which there are, what one may be called | `cheats.rs` |
| `ErrorLog` | `error_log.rs` |
| `Suggestion` | `suggestion.rs` |
| .NET's string rules counted in UTF-16 code units | `dotnet_text.rs` |
| The C layer for all of these: 23 exports, 45 in the library; the interface's version is 2 | `Platform/platform/src/galaxia_config.rs`, `include/emusen_platform.h` |
| The facades | `ConfigFile.cs`, `Models/AppSettings.cs`, `AudioConfig.cs`, `GraphicsConfig.cs`, `CheatFile.cs`, `Library/ErrorLog.cs`, `Text/Suggestion.cs`, `Native/GalaxiaNative.cs` |
| The parity class's second half, 15 cases | `EmuSen.WiseMan/Galaxia/GalaxiaParityTests.Config.cs` |

About 2,140 lines of Rust and 830 of its tests, against §7.2's 1,900 for the step with its tests; Galaxia whole is
3,620 and 1,560, against 3,000. The estimate priced the rules it knew of. What it did not price is §11.3: each rule of
.NET's that had to be found, reproduced and held.

### 11.2 How a model crosses, and what was built otherwise than §4.1 planned

**Reading.** The library reads the file (copying it in first from where it sat before Galaxia, when it is only
there), decodes it as .NET decodes a file, parses it, binds it to the model's schema and, when asked, upgrades it. What
it returns is the **bound document**: JSON holding every field the model can set, in the model's order, a number as
the token the file had. The C# binds that to the class with System.Text.Json and the same options as before. A file
that will not load comes back as a status and words; the C# puts them in Galaxia's frame (`<path>: <words> Falling
back to defaults.`), sets `LastLoadError` and reports, exactly where it did.

**Writing.** The C# serializes the class, and the library binds that document to the schema and writes it with its
own writer: the bytes on disk are Rust's, and the parity class holds them to the bytes C# produced a moment before.
The folder is made first, as in C#, so a class that cannot be serialized still leaves its folder behind.

**The other files.** A `ConfigFile<T>` whose `T` is not one of the four models (the key and pad bindings, keyed by
Avalonia's `Key` and SDL's buttons) takes the same route for everything but the binding: the library finds, copies,
decodes and writes the file, and System.Text.Json binds the type, since its vocabulary is a frontend's (§4.1, tier 2).

**Otherwise than planned, each for a reason:**

- **A model is a schema as data and a bound document, not a Rust struct.** §2.3 wrote `galaxia::config::load::<AppSettings>()`.
  What a file may hold is wider than a struct says: any C# reference may be null, a number may be a token no `f64`
  writes back (`1e999` loads, as infinity, and then cannot be saved), a map's keys repeat. A document keeps all of it,
  and a struct with every field optional would keep it badly. A Rust DianaSh reads a field by name through the
  document; structs can be generated from the same declarations when a consumer wants them. It is a choice that
  could be made otherwise, and is the first question of §11.8.
- **`StateRecord`'s sidecar was not ported.** §4.1 listed it. It is read only to import the files builds before
  2026-09-26 wrote (`EmuSen_Galaxia.md` §5.3a), it is never written, and reading it needs System.Text.Json's `DateTime`
  (an ISO 8601 profile, converted to local time) and `required` members. Reproducing those for an import that runs
  once per old state buys nothing; it stays C# and goes when the import does.
- **The time of a log entry is text the caller supplies.** C# formats it with `{now:yyyy-MM-dd HH:mm:ss.fff}`, in which
  `:` is the culture's time separator and the year is the culture's calendar's. A library that formatted it itself
  would write a different line on a machine whose culture differs from the invariant one, so it formats nothing and
  owns the rest (§11.4).
- **Text that is not valid UTF-16 cannot cross**, since it has no UTF-8. C# strings may hold half a surrogate pair and
  Rust's may not. For a suggestion, a cheat list's name and a log folder the facade asks the C# when such text
  appears; for a log entry it shapes the entry in C#, which then fails at the write exactly as it always has
  (§11.3's last row). When the C# is retired these become "replaced by U+FFFD first", a small change of behaviour
  that belongs to that step. *Found 2026-10-10: the other facades did not ask, and threw; all now do (§21).*

### 11.3 System.Text.Json and .NET, measured for this step

§5.2's table was the start. These were measured before or while the reader, the binder and the writer were written,
each in a scratch program on .NET 10, and each is a case of the parity class:

| Rule | What .NET does |
|---|---|
| A comment | allowed wherever white space is, **except between a name and its colon**; a line or paragraph separator (U+2028, U+2029) inside a `//` comment is an error |
| Depth | 64 levels are read, the 65th is refused, counted through values that are skipped too |
| An integer property | takes digits and a sign only: `1e2` and `1.0` are refused; `-0` is 0 |
| A `double` or `float` property | takes any number token; `1e999` **loads**, as infinity, and the document then cannot be written |
| `null` | allowed for a string, a class, a list, a map and a nullable; refused for `int`, `long`, `double`, `float`, `bool` |
| A member written twice | the last wins **whole**: a class given twice is the second one with defaults, not the two merged |
| A map's key written twice | keeps its first place and takes its last value; keys keep their case, where property names do not |
| An escape that is not valid UTF-16 (`\ud800`) | refused in a value that is bound, in any name of a bound object and in a map's key; **accepted inside a value that is skipped** |
| A property with no setter | **written** (every cheat file carries `IsRomPatch` and `EffectiveWrites`), and on reading skipped without a look at its value |
| A number's shortest digits, when two candidates are equally near | the one whose last digit is even; Rust's own formatting takes the upper |
| A number's layout | scientific past 17 digits before the point for a `double` and 9 for a `float`, and below 0.0001; the exponent has two digits at least |
| `File.ReadAllText` | honours UTF-16 and UTF-32 byte-order marks as well as UTF-8's |
| `StringComparer.OrdinalIgnoreCase.Compare` | orders by upper-cased UTF-16 units (`_` sorts after the letters), a surrogate pair after any single unit |
| `File.Delete` | says nothing of a file that is not there, and throws for a folder that is not |
| `File.GetLastWriteTime` on a link | the link's own time, not its target's |
| `File.AppendAllText` with half a surrogate pair | throws, **having created the file** |

**Two predictions of this step that were wrong,** kept because each was built on:

- *The layout turns scientific at 15 digits for a double and 7 for a float.* It is 17 and 9: the writer uses the
  round-trip count, not the display precision. Measured before the formatter was written, so no code carried it.
- *`OrdinalIgnoreCase` past the first character outside ASCII orders by the characters as written.* That was
  recalled from the runtime's source and built; the parity class showed it orders by their upper case throughout
  (§11.5).

### 11.4 The error log

The C# formats the time, calls the library to shape the entry, runs its redactor over the result, and hands the
text back to be appended; the library owns the entry's shape, the day's file name, the pruning and the eight
megabytes. Two things about the bound are C#'s and are kept: the entry's length is counted in UTF-16 code units
against a size in bytes, and the entry that fills the day is written whole with one notice after it.

**One difference is not reproduced, and is recorded.** C# prunes a file whose local last-write time is before the
local time fourteen days ago; the library compares instants. The two differ only for a file written within the hour
that a change to or from summer time moves, twice a year, on a retention of fourteen days.

### 11.5 The parity class, and what it found

Its second half asks, of the C# and the library side by side:

- 1.8 million numbers formatted: random bits of both widths, what a person types, every power of ten;
- every character of the first plane escaped in a value and in a key, lone surrogates included;
- 4,000 files of random bytes under every byte-order mark decoded;
- each model's schema against its class by reflection, and its new instance against `new T()`;
- 1,600 seeded instances written by C# and by the library, and read back through each other;
- about 9,000 documents setting every property of every class, at every place a class is held, to each of 59
  values, with a map's keys repeated;
- about 28,000 malformed texts: a hand-written list, every truncation and every one-character loss of two documents
  for each model, and 4,000 random damages each;
- 27 file scenarios for each model and for a frontend's type, and a 28th where a class can be made that will not
  serialize: missing, bad, null, a folder in the file's place, the
  copy from the old place and its failures, categories, saving over a file, a folder and a blocked folder, what
  cannot be serialized, deleting, and each side reading what the other wrote;
- the settings upgrade over eight files; the cheat lists saved to any path, named and ordered; the error log's bytes
  over 26 sequences, with its pruning and its bound; the log folder's choice; 6,000 suggestions.

**What the first run found.** Six cases failed. Four were defects in the Rust:

1. **Digits, 675 of the 1.8 million numbers.** Half way between two candidates of the shortest length, .NET takes
   the even one and Rust the upper (1482412.25 as a `float` is `1482412.2` to .NET). The formatter now asks, for the
   one number in ten that could be such a tie, whether it is exactly one, and takes the even digit.
2. **The caseless order,** in the cheat lists and in which suggestion comes first (§11.3's second retired
   prediction).
3. **A log entry with half a surrogate pair** was written, with U+FFFD in its place, where C# fails and leaves an
   empty file.
4. **A log file's age** was read from a link's target; .NET reads the link's own.

The other two were the tests' own. **The readers agreed on every text from the first run**: no malformed text and
no property document was read differently. That is what measuring §11.3 first bought, and silence from a test is
evidence only once the test is shown to speak, which is what the seeded faults are for.

**Seeded faults: 77, in every part of the step.** The parity class caught 67 on its first run. The ten it missed
were all faults of the test, not of luck, and each was closed:

- four were hidden because the test judged the library's reading by binding its answer again in C#, which refused
  on the library's behalf what the library had wrongly accepted (`null` for an `int`, 2147483648) and tidied what it
  had wrongly kept (a key twice). The library's refusal is now its own to make;
- one was hidden by the transport: a list crossed with its items separated, so an empty item vanished. Each item is
  now followed by its terminator, and an empty name is an item;
- five were cases the corpus lacked: a category with a file of its name in the old place, a class that cannot be
  serialized, a save over a folder, an entry between the bound in code units and the bound in bytes, and the dotted
  capital I against `i`.

All 77 are now caught by the parity class. The crates' own tests caught 70 at first and catch 77 after seven cases
were added.

### 11.6 What a load costs: P1 measured, failed, and then held

`AppSettings.Load()` on a 3,086-byte file, 3,000 times in a scratch program, release builds:

| | Load | Save | A state's path |
|---|---|---|---|
| C# | 0.023 ms | 0.028 ms | 0.16 µs |
| The library, as first built | 0.116–0.122 ms | 0.130–0.141 ms | 0.69 µs |
| The library, as committed | 0.045 ms | 0.057 ms | 0.51 µs |

**P1 predicted under 0.1 ms and the first measurement was over it.** The cost was not where it was first looked
for: building each field's path for an error message that is rarely needed was removed, and changed nothing. It was
the name lookup, which compared every member's name without case against every field in turn, casing both each
time: about 1,700 caseless comparisons for one settings file. A name in its own case is now found by plain
equality, and only a name that is not falls back to the caseless search. The load is then twice the C#'s, which is
the doubled parse P1 expected, and P1 holds at 0.045 ms.

### 11.7 The default path, and what was tested

**The default is untouched, shown three ways.** The C# of the seven files was moved and not rewritten: every
statement of each at `f3df2ea0` is still in the file, apart from eight lines that changed by necessity: the three
`Load` members and `ResetForTests`, which gained their branch; `ErrorLog`'s clock, which became a parameter so that a
test can give both implementations one instant; its entry, now built by a method of its own that the facade also
calls; and its settings read and `CheatFile`'s folder, which name the C# rule directly. With the variable
unset or `0`, the standalone shell and a program that loads and saves settings 6,000 times open
`libemusen_platform.so` no times under the loader's trace, and with `1` they open it. And the existing tests pass
unchanged.

| Run | Result |
|---|---|
| `cargo test` in `Platform/` | 88 pass (76 in `emusen-galaxia`, 12 in `emusen-platform`) |
| The header as C99 and as C++17, `-Wall -Wextra -pedantic -Werror` | clean |
| WiseMan's Galaxia classes with `DianaShSandboxTests` and `HierTests`, variable unset | 219 pass |
| The same, `EMUSEN_GALAXIA_NATIVE=1` | 219 pass |

The 219 are the 173 that existed and the parity class's 46. A linux-x64 publish adds and drops no file against step
2a's; `libemusen_platform.so` grows from 532,888 to 766,456 bytes.

**Not done, and not claimed:**

- **Windows and macOS**, as in §10.7, and now also .NET's line ending in a written file (CRLF on Windows, argued
  from `Environment.NewLine`) and file-name patterns that ignore case on both.
- **Tests outside Galaxia's with the switch set.** The shell's `cheat save` and `cheat load` and Mistress's windows
  reach these facades; their tests were run only as part of nothing wider than the filter above, which is the
  variable unset for them. They belong to the gate (step 2c).
- **P2** is still not measured. **P3**, that the first differences would be in §5.2's rows and nowhere else, is
  retired as false: none was in §5.2's rows, which were built in from the start, and all four were elsewhere.
- **The order of two cheat lists whose names differ only by case** is the folder's own, in both implementations,
  and is compared without case.

### 11.8 Questions for the tester

Four things this step did otherwise than the design said, or that the design did not foresee. *Decided 2026-10-08:
Q16, Q17 and Q18 stand as built. Q19 stays open until the C# that answers for such text is retired.*

| | Question | As built |
|---|---|---|
| Q16 | A model in Rust: a struct, or a schema and a bound document | a schema and a document, which keeps everything a file may hold; structs can be generated later (§11.2) |
| Q17 | `StateRecord`'s sidecar | left in C#, as an import that ends with the files it reads (§11.2) |
| Q18 | Pruning by instants where C# compares local times | accepted: the two differ only within the hour a change of summer time moves (§11.4) |
| Q19 | Text that is not valid UTF-16, once the C# that answers for it is gone | replaced by U+FFFD before it crosses, decided with the C#'s retirement and not before (§11.2) |

---

## 12. What was built, 2026-10-08: step 3a, Endymion's logic half

Measured on the development desktop, as §10 and §11 were. Endymion's resampler, rate control, port router and seat
rules, and `PadControls`' rules, have a Rust form behind a switch of their own, `EMUSEN_ENDYMION_NATIVE`, in state 1:
the C# runs unless the variable is `1`, and the library is not opened while it is unset. No C# was removed, and
nothing of the device half (§4.2) was touched.

### 12.1 The artefacts

| What | Where |
|---|---|
| The crate, with no dependencies | `Platform/endymion/` (`emusen-endymion`) |
| `LinearResampler` | `src/resampler.rs` |
| `DynamicRateControl` | `src/rate_control.rs` |
| `PortRouter`'s rules: what changed, the mirror, the keyboard's player, which ports hold a controller | `src/router.rs` |
| `PlayerSlots`' rules: seating, trading seats, forgetting a reservation | `src/slots.rs` |
| `PadControls.Resolve`, `Combine` and `For`, and `Math.Clamp` | `src/pad.rs`, `src/lib.rs` |
| The C layer: 31 exports, 76 in the library; the interface's version is 3 | `Platform/platform/src/endymion.rs`, `include/emusen_platform.h` |
| The switch and the entry points | `EmuSen.Endymion/Native/EndymionNative.cs` |
| The facades | `LinearResampler.cs`, `DynamicRateControl.cs`, `Input/PortRouter.cs`, `Input/PlayerSlots.cs` |
| The parity class, 8 cases | `EmuSen.WiseMan/Endymion/EndymionParityTests.cs` |

About 560 lines of Rust in the crate and 590 in its C layer, with 310 of tests, against §7.2's 900 for the step with
its tests. The excess is the C layer, which §7.2 priced as if it were Galaxia's: a router answers with a list of
changes and a seat rule with a list of moves, and each kind of answer needed its own struct, its pending copy (§12.3)
and its test.

### 12.2 The facade, and what was built otherwise than §4.2 planned

**Chosen per instance.** Each facade's public constructor asks the switch; an internal one takes the choice as an
argument, so the parity class can hold a C# instance and a library instance side by side in one process (§12.5).
The C# of each class moved, unrewritten, into a nested `Managed` class (`PlayerSlots`: into `…Managed` methods), and a
facade built on the C# forwards every call to it. With the variable unset no facade is built any other way.

**`PortRouter` reads the pads in C# and decides in Rust.** The pads are `GamepadManager`'s, which is the device half
and stays C# until step 3b. So for each poll the facade reads each heard player's buttons as a mask and the game's
axes as numbers, asks the keyboard each of its 24 controls once, works out which players are seated (with
`FirstControllerOnly` applied), and hands all of it to the library, which returns the changes to make; the facade
makes them through the core's delegates, in the library's order. Two consequences, neither visible in what the core
is told, which the parity class holds: the C# asks the keyboard for a control each time it needs one, several times
in one send, where the library is given each control once, so the two could differ only for a keyboard that changed
in the middle of a send; and `Reset` copies the list of axes, where the C# kept the caller's list and would have seen
a later change to it.

**`PlayerSlots` keeps its seats in C#.** A seat holds a `ConnectedPad`, an SDL handle that the library cannot hold
while the device half is C#. The facade passes a snapshot of the eight seats (open or reserved, GUID, path) and the
library returns which seat each pad goes to. The crate's `PlayerSlots<P>` holds its own seats over any type that is a
pad, which a Rust DianaSh will use; the C layer drives it over a snapshot. A pad whose GUID or path cannot be encoded
as UTF-8 is seated by the C#, as §11.2 does for such text.

**Otherwise than planned, each for a reason:**

- **`GamepadBindings` stays C#.** §4.2 counted the binding files' format in the logic half. Its file is read and
  written through Galaxia's tier-2 `ConfigFile`, which since 2b is the library's for everything but the binding; what
  is left is the binding, keyed by `PadButton` and SDL's `GamepadButton`, which §4.2 itself keeps as the C#
  vocabulary, and the legacy flat shape's upgrade, a few lines over those types. Porting it would move a type map
  and no rule. It is Q20 of §12.7.
- **`PadControls` stays a Galaxia class in C#.** Its rules are reproduced in the crate, which the router needs, and
  the parity class holds them to the C#'s. The C# class is not made a facade: it is Galaxia's, under Galaxia's
  switch and not Endymion's, and its other callers (the catalogue's list of a console's controls and the bindings
  window's stick diagram) ask it outside any poll, where a crossing buys nothing.

### 12.3 The C# rules, reproduced, and one difference

Each of these was read off the C#, built and is a case of the parity class:

| Rule | What the C# does |
|---|---|
| A sample between two | `(short)Math.Round(prev + (next - prev) * frac)`: the difference an integer, the product a double, rounded **half to even** |
| `Math.Clamp` | returns NaN for a NaN value, as a stick's reading can be |
| A ratio of zero or below | refused, with "Resample ratio must be positive." |
| The first call ever | takes its first frame as the previous one, and makes no output from it |
| Shedding | begins above the target times 3 and ends at or below the target times 2, each a comparison of doubles; shed input is counted, the resampler reset at each edge |
| A null input to `Process` | sets the last ratio and then throws `NullReferenceException`; the library sets it and returns its NULL status, which the facade throws as the same exception |
| The counters | 32- and 64-bit, wrapping |
| The changes a poll makes | the connected flags of ports 2 and up, then each port's changed buttons, then **every** axis the game reads, changed or not |
| A trade of seats with a pad from no seat | the displaced pad goes to the lowest free seat, **asked after the mover sat down**, so a closed pad just seated counts as free |
| A pad's returning seat | by GUID and path, then GUID alone, then the lowest seat with no pad connected |

**One difference is deliberate.** Given a ratio that is NaN or infinite, the C# loops for ever: no step it makes
reaches the next input frame. The library refuses it with "Resample ratio must be finite.", which the facade throws
as the C# throws its own refusal. Neither frontend computes such a ratio (`DynamicRateControl` clamps its own, and a
NaN nominal ratio would be a defect upstream), so the difference is in what a defect does, not in what a player
hears. It is Q21.

**The C layer.** A call whose answer is a list (samples, changes, seat moves) writes what fits and keeps the rest in
the handle, returning the whole length; `…_take` copies what was kept. This is the length-query idiom of §3.3 for a
call that cannot be made twice, since a second poll would see nothing changed. The rate control's state crosses as
one struct, which is why the call that reads it is `emusen_endymion_rate_read` and not `…_state`: C forbids the
type and the function sharing the name. The interface's version is 3, and the C# asks for 3 exactly (§3.2).

### 12.4 The default path, and what was shown

**The default is untouched, shown three ways.** Every statement of the four classes at `d7f4d70e` is still in its
file, apart from six lines changed by necessity: the two constructors that became `Managed`'s, the rate control's
resampler, now the C# one by name, and `PlayerSlots`' `Seat` and `Highest`, now its `…Managed` methods. With the
variable unset, Endymion's resampler, rate control, router and seat tests, 53 cases, open `libemusen_platform.so` no
times under the loader's trace (`LD_DEBUG=files`), and with `1` they open it once. And the existing tests pass
unchanged.

| Run | Result |
|---|---|
| `cargo test` in `Platform/` | 108 pass (17 in `emusen-endymion`, 76 in `emusen-galaxia`, 15 in `emusen-platform`) |
| `cargo clippy` over the workspace | clean |
| The header as C99 and as C++17, `-Wall -Wextra -pedantic -Werror` | clean |
| WiseMan's Audio, Input and Endymion classes, `AudioProperties` and `LeafAssemblyTests`, variable unset | 186 pass |
| The same, `EMUSEN_ENDYMION_NATIVE=1` | 186 pass |
| Mistress's `MultiplayerTests`, `PadInputPathTests`, `ControllersTests` and `PlayerPreferencesTests`, which drive the router and the seats through a running game, both ways | 23 pass each way |

The 186 are 178 that existed and the parity class's 8. Galaxia's parity class had a case that counted every
function the header declares against Galaxia's exports, which Endymion's 31 broke; it now counts Galaxia's and the
library's own, as Endymion's case counts Endymion's, and Galaxia's classes pass with it (§13.3). Two of them, the buffer tests of MercuryRT and MoonRT, and two
of Mistress's, the Genesis's four-pad games, need their cores' libraries beside the tests and were run with them.

**CI** gains two steps beside Galaxia's, Endymion on its C# and on the library, which report and do not gate (§10.6).

**Not done, and not claimed:** Windows and macOS, as before; and P4, which is the device half's (step 3b).

### 12.5 The parity class, and what it found

It asks, of the C# and the library side by side, with seeded random input:

- the resampler over 400 runs of 30 calls: streams of silence, full-scale noise, steps and small values, of odd and
  even lengths, at a ratio near one or one of a list of awkward ones, with resets between;
- the refusals: ratios of zero, below zero, NaN and the infinities, and a null or empty input;
- the rate control over 300 runs of 60 steps, every property set at random between calls, the queue walked and
  jumped through each shedding edge, and a null input and a negative nominal ratio;
- the pad rules over every axis reading of a list (NaN and the infinities included) for every control, every
  combination of held directions, and `For` over random button and axis lists;
- the seats over 150 runs of 80 steps of pads connecting, going, returning by GUID and path, trading seats, being
  taken out and forgotten, with GUIDs shared and empty;
- the router over 120 runs of 70 steps, logging every call it makes to the core, through ports added and removed,
  the mirror and the keyboard's player changed, `FirstControllerOnly`, and every query between;
- that every export the C# looks up is in the header, and that the header declares no other of Endymion's.

**The first run found nothing**: no difference in any case. Silence is evidence only once the test is shown to
speak, so 38 faults were seeded in the Rust, each built into the library and run against the class:

- **35 were caught** by the class's own failure.
- **One was caught by the test host stopping.** "One sample is enough" lets a lone sample through to the first
  call's priming, which reads past the array; the library aborts, as its profile requires (§3.5), and the run ends
  with two cases passed. The crate's own test of a lone sample catches it outright.
- **Two survived, and both are equivalent**, so neither is a gap: taking the difference of two samples as a double
  instead of an integer is exact either way, since a difference of two shorts is; and a reset that kept the
  position and the previous frame changes nothing, because the first call after it primes both afresh.

### 12.6 Publishing

A self-contained linux-x64 publish of Mistress has the same 454 files as one of step 2b's commit (`d7f4d70e`), built
the same way beside it; `libemusen_platform.so` grows from 766,456 to 799,480 bytes, so P7's 4 MB still holds with
two of the three crates in.

### 12.7 Questions for the tester

*Decided 2026-10-08: both stand as built (§9).*

| | Question | As built |
|---|---|---|
| Q20 | `GamepadBindings`, which §4.2 put in the logic half | left in C#: its file is already the library's, and what is left is a map over the C# vocabulary §4.2 keeps (§12.2) |
| Q21 | A NaN or infinite ratio, for which the C# never returns | refused with words, as the C# refuses a ratio of zero (§12.3) |

---

## 13. Step 2c, 2026-10-08: Galaxia in state 2

The tester gave Galaxia's gate its go on 2026-10-08, after the review of steps 2a and 2b. Galaxia now runs on the
library by default, and `EMUSEN_GALAXIA_NATIVE=0` selects its C#. No C# was removed, and `EmuSenPlatformRequired`
stays false, since every rule still has its C# behind it.

### 13.1 The gate's evidence

A full WiseMan run on 2026-10-08 with `EMUSEN_GALAXIA_NATIVE=1` passed 9,826 cases, skipped 53 and failed 5. All five
failed the same way with the variable unset: expectations written before the Genesis was offered, which still
listed the four Nintendo consoles, in the three tests that `258e8c18` corrects; this step is built on that commit. So no case outside Galaxia's own classes differed between the two implementations. With §10.5's and
§11.5's parity classes and their seeded faults, that is what §3.9 asks before state 2.

### 13.2 The switch in state 2

| The variable | The library loads | Galaxia runs on | Said |
|---|---|---|---|
| `0` | not asked | the C# | nothing |
| unset, `1` or any other value | yes | the library | nothing |
| unset, `1` or any other value | no: absent, refused, or its load throws | the C# | one line, once |

The choice is made once in a process, on the first question any facade asks, and kept; it is a small class,
`PlatformSwitch` in `GalaxiaNative.cs`, so that a test can make a choice of its own without touching the process's.
The line goes to the host's `ConfigDiagnostics` sink and to the error log as a warning of area `platform`:

```
<time> WARN [platform] libemusen_platform.so is not in use (libemusen_platform.so not found beside the assemblies); Galaxia runs on its C# implementation.
```

The words in the parentheses are `PlatformLibrary.Report`'s, so a library of the wrong interface version or one
missing an export is named as such. **The choice is recorded before the line is written**, because writing it asks
the switch again (the error log is a facade too), and that question must find the C# already chosen and not choose
a second time. A load that throws an exception of any kind is caught and reported the same way, with the
exception's type and words in the parentheses; Galaxia must never fail to start because of the library.

**Two consequences, recorded rather than changed.** The library is now opened by every program at its first use of
Galaxia, so §12.4's loader trace, which showed Endymion's tests opening nothing with its own variable unset, would
now show Galaxia opening it; Endymion's switch still decides only Endymion's half. And a crash in the library is now
a crash of the default path, which is why the crash log of §3.5 is installed when the library loads, as before.

### 13.3 What was tested

The blast radius only, as §6.7 has it:

| Run | Result |
|---|---|
| WiseMan's Galaxia classes with `DianaShSandboxTests` and `HierTests`, variable unset (the library) | 220 pass |
| The same, `EMUSEN_GALAXIA_NATIVE=0` (the C#) | 220 pass |

The 220 are 2b's 219 and one new case, which builds a switch over a folder without the library and asks it five times
for each of the variable unset, `1` and `0`: the C# each time, two loads tried and not three, the line exactly once
for each of the two that tried, both in the sink and in the error log, and none for `0`; and a load that throws, which
is the C# and the line with the exception's words. Three faults were seeded in the switch (nothing said, a throwing
load not caught, the fallback not remembered) and the case caught each.

The standalone DianaSh shell was also run from copies of its build in a sandboxed home, with and without the library
beside it. With it and the variable unset it opens `libemusen_platform.so` once under the loader's trace; with `0`,
no times; without it, unset or `1`, the C# runs and the day's error log holds the line once. Its output is the same
bytes in all four runs.

**Not done:** Windows and macOS, as before; and P2 is still not measured. CI's two Galaxia steps now run the C# with
`EMUSEN_GALAXIA_NATIVE=0` and the library with the variable unset, and still report rather than gate (Q22).

### 13.4 Questions for the tester

*Decided 2026-10-08: CI keeps reporting and does not gate yet (§9).*

| | Question | As built |
|---|---|---|
| Q22 | Whether CI's Galaxia and platform steps now gate | still reporting: §9's Q14 tied gating to Galaxia's gate, but Windows and macOS have never run these tests, and a platform whose library fails still runs on the C# |

---

## 14. What was built, 2026-10-08: step 3b, Endymion's device half

Measured on the development desktop, as the steps before it. The game's audio player, the interface's sound player,
the pad manager's device bookkeeping and reading rules, SDL's pads and the simulated ones have a Rust form behind the
same switch as step 3a, `EMUSEN_ENDYMION_NATIVE`, still in state 1: the C# runs unless the variable is `1`. No C# was
removed. No test opens a real audio device or a real pad: audio is SDL's dummy driver, and pads are simulated or SDL's
own virtual joysticks.

### 14.1 The artefacts

| What | Where |
|---|---|
| SDL3 by the 30 functions Endymion calls, loaded from a file or lent by the host | `Platform/endymion/src/sdl.rs` |
| The device layer (`IPadDevices`), SDL's pads, simulated pads and their sets | `src/devices.rs` |
| `GamepadManager`'s bookkeeping and reading rules, and `ConnectedPad` | `src/pads.rs` |
| `AudioPlayer` and `UiSoundPlayer` | `src/audio.rs` |
| The C layer: 55 exports, 86 of Endymion's and 131 in the library; the interface's version is 4 | `Platform/platform/src/endymion_devices.rs`, `include/emusen_platform.h` |
| The entry points and the lending of SDL | `EmuSen.Endymion/Native/DeviceNative.cs` |
| The facades | `AudioPlayer.cs`, `UiSoundPlayer.cs`, `Input/GamepadManager.cs`, `Input/ConnectedPad.cs`, `Input/SimulatedPad.cs`, `Input/SimulatedPads.cs` |
| The parity class, 11 cases | `EmuSen.WiseMan/Endymion/EndymionDeviceParityTests.cs` |

About 1,200 lines of Rust in the crate and 950 in its C layer, with 180 of tests, against §7.2's 1,100 for the step
with its tests. As in §12.1 the excess is the C layer, and for the same reason with more of it: five kinds of handle,
each with its queries, where §7.2 priced the rules.

### 14.2 The facades, and what was built otherwise than §4.2 planned

**One SDL, lent by its handle.** §2.8 planned to load SDL by path and argued that the loader would return the same
instance. It is not left to argument: the C# asks .NET for the handle its own SDL calls resolve to
(`NativeLibrary.TryLoad("SDL3", …)` for SDL3-CS's assembly) and lends it to the library, which looks its 30 functions
up in that handle and never closes it. So a C# player and a library player in one process share SDL's subsystem
counts, hints and devices, wherever .NET found the file. **Checked:** a hint the C# sets is read back through the library's SDL, and gone when the C# resets it.
The crate also opens SDL from a path, for a Rust program with no C# beside it. The lending happens once, the first
time the device half is asked for; with the variable unset nothing asks.

**The manager is split where step 3a split the router.** The library opens, closes and reads the pads; who sits
where stays in the C# `PlayerSlots` (whose rules are the library's since 3a), and the bindings stay the C#
`GamepadBindingMap` (Q20). A start, a poll or a change of devices returns the pads opened and closed, in order, and the
facade does for each what the C# did in the same place: seats it, lights the players, raises `PadChanged`; then it has
the library update the devices and raises `Polled`. A pad is a key in the library, and the `ConnectedPad` the
programs hold reads through it. `IsPressed` is one crossing when the bindings are a map; for a player with bindings
of its own, which are a delegate, the stick and the trigger are asked first and the delegate only when neither
answers, as the C# asks it.

**Devices the library does not know run the C#.** `IPadDevices` is an interface a test may implement, and the library
never calls back (§3.7). A manager given SDL's devices or a simulated set of the library's is the library's; given
anything else it is the C# manager for that instance, and one handed such devices later (`UseDevices`) becomes the C#
manager from there with the settings it had. `SdlVirtualPadsTests`, which wraps the SDL layer to hide the pads on the
desk, therefore runs the C# either way; the library's SDL layer is held by a parity case of its own (§14.5), told
which ids it may see.

**The audio player is lent its rate control.** `AudioPlayer.RateControl` is an object the programs read, and it must
outlive the player. So the rate control is its own handle, as in 3a, lent to the player's constructor and to each
submit; a submit is one crossing, which makes the library's two SDL calls.

**A device is let go by `Dispose` and never by the collector.** The C# classes have no finalizer: a player or a
manager that is never disposed keeps its device until the process ends. The handles keep that: disposing closes the
device and leaves the object usable, as the C# object is after `Dispose`; the collector frees the library's memory
and touches no device.

**Otherwise than planned, each for a reason:**

- **`ConnectedPad` has no `Managed` class.** It is a dozen one-line members over a device layer; each gained a
  branch for the library's key and kept its C# beside it.
- **A simulated pad and its set are either the C#'s or the library's**, chosen when they are made, and a set takes
  pads of its own kind; mixing them is refused with an `ArgumentException`. The library's set is also a device layer
  to the C# manager, so the wrapper above works over either.
- **What two threads use locks itself.** The C# manager is read on the emulation thread while the window polls it,
  the volume is set from the window while the emulation thread submits, and the window resets the rate control the
  submit is steering by; the C# does all three with no lock and gets away with it. The same in Rust is a data race,
  on a map in the first case. So the pads handle, the audio player and the rate control each hold a mutex, as a
  simulated pad and a set do, and a submit takes the player's and then the rate control's, the one order in which
  two are held. The rate control's is a change to step 3a's handle, which the header had as used by one thread at a time.

### 14.3 The C# rules, reproduced, and the differences

| Rule | What the C# does |
|---|---|
| A poll | lets go of each pad no longer attached, announcing it with the seat it keeps; then asks the devices for changes **and** whether a rescan is due, both always; then opens what is new; then updates the devices; and does none of it, nor raises `Polled`, before the devices have started |
| A rescan | due one second after the last, and at once on the first poll |
| A closed pad | keeps the name and type it had when it closed; reads as nothing held |
| A pad with no name | "Unknown controller" |
| The stick as the d-pad | when the setting is on and the console does not read the stick; past a threshold of the deadzone clamped to 0.05 to 0.95, times 32,767, truncated |
| L2 and R2 | pressed at half the trigger's travel, read through the analog deadzone |
| An axis | SDL's sixteen bits over 32,767, clamped to −1 to 1, zero strictly below the deadzone |
| The rebind capture | updates the devices, then the first SDL button held on a pad the interface reads, the first pad first, in SDL's order |
| A simulated axis | clamped, times 32,767, rounded half to even |
| A simulated GUID | the pad's own, an empty one included, else the MD5 of its name |
| The latency target | `targetLatencyMs * sampleRate / 1000` in unchecked 32-bit arithmetic, set again when the device reopens |
| A submit at another rate | closes the device, opens it at the new rate and resets the rate control, even with nothing to play; a rate of zero or below changes nothing |
| `Dispose` of a player | closes the device and lets go of SDL's audio, **each time it is called** |
| An interface sound | decoded once to 48 kHz stereo floats; one that is missing, unreadable or empty opens nothing; a new one clears the stream first; a player once tried or disposed does not open again |

**Differences, each recorded and none reachable in play:**

- **The order of device calls inside one poll.** The C# opens a pad, seats it, lights the players and announces it
  before it opens the next; the library opens all that are new and the facade then seats, lights and announces each.
  What the programs are told and in what order is the same; a `PadChanged` handler that counted the device layer's
  open handles in the middle of a poll would see the later pads already open.
- **A rescan across an overflow.** For two times further apart than a tick count holds the C# throws; the library
  says a rescan is due. The clock is a stopwatch, which does not get there.
- **`Remember` copies the samples.** The C# keeps the caller's array, so a change made to it afterwards would be
  heard; the library keeps its own copy.
- **Text that is not valid UTF-16** (a name, a path, a sound's key) crosses with U+FFFD in place of half a surrogate
  pair, which for a path is what SDL's own binding sends. Two keys that differ only there become one. It is Q19's
  subject.
- **A null name for a simulated pad** is an `ArgumentNullException`; the C# stores it.

**Reproduced but not exercised:** a second `Dispose` lets go of SDL's audio a second time, in both. SDL counts its
subsystems for the whole process, so a test of it would close the audio under any other test's player; it is read
from the two sources and not run.

### 14.4 The default path, and what was tested

**The default is untouched.** Every statement of `AudioPlayer`, `UiSoundPlayer`, `SimulatedPad` and `SimulatedPads` at
`2f42d959` is still in its file but two (a simulated pad's path, now given by the facade from the one counter, and
`With`, which makes a set of its pads' kind). `GamepadManager`'s 129 statements are there but for 20 changed by
necessity: the seats, the bindings and the two events are the facade's, so the moved code names its owner for them,
and members the facade forwards to became public on `Managed`. `ConnectedPad` is the one class edited in place
(§14.2). With the variable unset the device half is not asked for, SDL is not lent, and every device object is the
C#'s. One existing test changed: `GamepadRescanTests` read the private `_lastRescan` by reflection and now asks the
manager for it, since the field moved.

| Run | Variable unset | `EMUSEN_ENDYMION_NATIVE=1` |
|---|---|---|
| `cargo test` in `Platform/` (25 in `emusen-endymion`, 76 in `emusen-galaxia`, 17 in `emusen-platform`) | 118 pass | |
| `cargo clippy` over the workspace; the header as C99 and as C++17 | clean | |
| WiseMan's Audio, Input and Endymion classes, `AudioProperties` and `LeafAssemblyTests` | 197 pass | 197 pass |
| Fourteen of Mistress's classes that play through a pad or the interface's sounds (`MultiplayerTests`, `PadInputPathTests`, `PadNavigationTests`, `PadMenuTests`, `PadCheatsTests`, `PadRewindReelTests`, `PadSettingsWindowTests`, `ControllersTests`, `ControllerBindingsDiagramTests`, `PlayerBindingsWindowTests`, `PlayerPreferencesTests`, `ThemedControllersTests`, `ThemedLibraryHostTests`, `ThemedLibraryReferenceTests`) | 196 pass, 1 skipped | 196 pass, 1 skipped |

The 197 are 3a's 186 and this step's 11; the one skipped is a picture tool. These are the gate §7.1 names for the
step: the pad and audio tests on the simulated and dummy devices, both ways.

**Not done, and not claimed:** Windows and macOS, as before; a real pad and a real audio device, which no test may
open; and Mistress's other classes, which reach the pad only to move through a menu.

### 14.5 The parity class, and what it found

It drives the C# and the library alike and compares:

- a simulated pad over 200 runs of 40 things done to it, and a set as a device layer over 120 runs of 50;
- a manager over 150 runs of up to 70 steps: pads plugged, pulled and plugged back, pressed and pushed, polled,
  seated, forgotten, every setting changed, the bindings replaced for all players and for one, the devices changed
  for another set, started, disposed and made again; after each step everything a program can read of it, the
  seats, the lights, the set's counts and the log of what was announced;
- readings at the edges: an axis exactly at each deadzone and a stick one step either side of each threshold, with
  deadzones outside their range and NaN;
- a rescan's timing over 20,000 pairs of times;
- devices the library does not know, and a manager moved onto them;
- SDL's virtual joysticks under each manager in turn: three attached, read, one pulled and replaced;
- the game's player on the dummy driver at five rates and latencies: opening, the volume, null and empty samples, a
  reopening, the rate control's target and reset, disposing and use after it;
- the interface's sounds: ten files decoded to the same bytes (five sounds, one of them empty and one long enough to
  be taken in two calls, a file that is no sound, a missing one, a folder, an empty path and a path with a NUL in
  it), and one sound replacing another.

**What the dummy driver allows.** Its queue drains in real time, so two players never see the same queue. What is
compared exactly is what does not depend on it (the device's rate, the target, the volume, the input counted,
shedding with a target no queue reaches or one below zero), and what does is held to a range: the output within the
deviation of the input, the interface's queue between half a sound and all of it.

**The first run found nothing.** Of 59 faults seeded in the Rust, the class as first written caught 52, and a 53rd
by the test host stopping (a player whose `dispose` left its device open, once SDL's audio had gone). Four it missed
were gaps and are closed: the readings at the edges (two faults: the deadzone's `<` and the threshold's floor),
which a random walk had not landed on; the devices' word left unconsumed when a rescan was also due, now asked of
the set after a poll; and the bound button, which the facade never passed because it asked the bindings in a
second crossing, and now passes when the bindings are a map. The last two survive and are not gaps: a trigger at
exactly half its travel does not exist in sixteen bits over 32,767, and SDL's virtual pads never report an empty
path, which is the only way to tell a kept one from a dropped one.

### 14.6 P4, and publishing

**P4 holds.** The C# submit makes two SDL calls, `GetAudioStreamQueued` and `PutAudioStreamData` (counted from the
code); the library's is one crossing. One frame of a 32 kHz console (534 frames) submitted 8,000 times to the dummy
driver, the median of nine rounds, three runs each way in turn: 5.4, 5.3 and 5.1 µs a submit on the C#, 5.0, 4.9 and
4.9 µs on the library, its two locks included. A set of runs an hour earlier had the C# at 5.4 to 5.9 µs, so the
difference is inside the spread between sessions, and either is three ten-thousandths of a 60 Hz frame.

**Publishing.** A self-contained linux-x64 publish of Mistress has the same 454 files as one of `2f42d959` built the
same way beside it; `libemusen_platform.so` grows from 799,480 to 953,664 bytes, so P7's 4 MB holds with two of the
three crates whole. In a publish `libSDL3.so` sits beside the assemblies and not under `runtimes/`, and the lending
finds it there: a small program published self-contained and run with the variable set reported the device half on
the library and opened the dummy device through it.

### 14.7 Questions for the tester

*Decided 2026-10-08: Q24 stands as built. Q23 stays open until Endymion's state 3 (§9, §15).*

| | Question | As built |
|---|---|---|
| Q23 | A manager over devices the library does not know | runs the C# manager for that instance (§14.2). At state 3 it has no C# to run: either `IPadDevices` stops being something a test may implement, or the library learns to call back, against §3.7 |
| Q24 | SDL lent by its handle, where §2.8 said loaded by path | lent, which is the same instance by construction and was checked; a Rust program with no C# opens it by path |

---

## 15. Open items

Kept here so that none is lost between steps; each names where it was found.

- *Closed 2026-10-10:* **The log folder in a home with no `.config`.** The C# takes its default log folder from .NET's `ApplicationData`,
  which is the empty string when the folder it names does not exist, so the default becomes `EmuSen/Logs` relative to
  the folder the program was started from. Found 2026-10-08 while running the shell in a sandboxed home (§13.3). It
  predates this work and is the C#'s and the library's alike, since the host hands the library the same answer (Q15).
  Fixed as proposed, the folder asked for without requiring it to exist, with a test of its own: `EmuSen_Galaxia.md`
  §3.6.
- **Q19:** text that is not valid UTF-16, once the C# that answers for it is gone (§11.2, §14.3). Until then every
  Galaxia facade answers it with its C# (§21).
- **Three differences left in Galaxia (§21.4):** an override that does not cross, set after the library loads; a ROM's
  hash for a path holding a NUL; a migration to a destination holding a NUL. None is reachable by a player.
- **Q23:** device layers the library does not know, at state 3 (§14.7).
- **P2** is not measured: whether any frontend asks Galaxia for a path once a frame (§8).
- **Windows and macOS:** nothing of the platform has run on either (§10.7, §11.7, §12.4, §14.4); CI's steps there
  report and do not gate (Q14, Q22). *Partly closed 2026-10-15:* the crates' tests and every platform step of the
  WiseMan job now run on win-x64 and osx-arm64, and on Windows they pass (§27); the four macOS failures left are
  listed in §27.4. The steps still report rather than gate.
- **A second `Dispose` of a player** lets go of SDL's audio twice; reproduced, not run (§14.3).
- **A Rust program with no C# beside it** will need a SQLite of its own (Q25); what decodes lookup images then is
  decided at Serenity's retirement (Q27).
- *Closed 2026-10-10:* **Q26, the compiler's identities,** looked at again at Serenity's state 2 as planned: the cost
  of their staying separate is measured (§19.4), and they stay separate until the C# is retired (§9).
- *Closed 2026-10-10:* Q27 to Q30 are decided (§9); the four defects of the C# chain and device are fixed, and
  `SlangSyncTests` now shows synchronisation validation is on (§18).
- *Closed 2026-10-10:* Endymion and Serenity are in state 2 (§19).
- **State 3** for each component is a later step of its own, after golden outputs (§6.6), at the tester's word; Q19
  and Q23 are decided with it.

---

## 16. What was built, 2026-10-09: step 4a, Serenity's slang half without a device

Measured on the development desktop, as the steps before it. A preset and its sources are read, each stage compiled,
its SPIR-V reflected and the compiled stages cached by a Rust form of the six classes that need no Vulkan device,
behind a switch of their own, `EMUSEN_SERENITY_NATIVE`, in state 1: the C# runs unless the variable is `1`. No C# was
removed. The device and the chain (`SlangVulkan`, `SlangChain`) and their glue (`SlangRunner`) are step 4b's and were
not touched; they call the six classes as they did. The SkSL filters are no part of this work.

### 16.1 The artefacts

| What | Where |
|---|---|
| The crate | `Platform/serenity/` (`emusen-serenity`) |
| `SlangPreset`, `SlangSource`, `SlangParameters` | `src/preset.rs`, `src/source.rs`, `src/parameters.rs` |
| .NET's text rules the readers lean on: the two character classes, number parsing, a file's lines | `src/text.rs`, `src/word.rs` |
| `SlangCompiler`, over the Shaderc the program ships | `src/compiler.rs` |
| `SpirvReflection` | `src/reflection.rs` |
| `SpirvCache`, over the SQLite the program ships | `src/cache.rs`, `src/sqlite.rs` |
| The C layer: 21 exports, 152 in the library; the interface's version is 5 | `Platform/platform/src/serenity.rs`, `include/emusen_platform.h` |
| The switch, the entry points and the lending of the two libraries | `EmuSen.Serenity/Native/SerenityNative.cs` |
| The facades | `EmuSen.Serenity/Slang/SlangPreset.cs`, `SlangSource.cs`, `SlangParameters.cs`, `SlangCompiler.cs`, `SpirvReflection.cs`, `SpirvCache.cs` |
| The parity class, 13 cases | `EmuSen.WiseMan/Serenity/SerenityParityTests.cs` |

About 1,860 lines of Rust in the crate and 620 in its C layer, with 390 of tests, against §7.2's 1,300 for the step
with its tests. Two things were not priced: SQLite's C interface written out by hand (§16.2), and .NET's rules for a
number and a line, which the readers take from the framework in one call each and the crate has to hold itself.

### 16.2 The two libraries, the facades, and what was built otherwise than planned

**Shaderc is the one the C# loaded, lent by its handle.** The C# asks its binding for the handle it holds
(`Silk.NET`'s native context) and lends it, with the path of the file it came from; the library looks its 14
functions up in that handle and never closes it. So a stage is compiled by the same code whichever side asks, and the
library's identity for the compiler carries the same file's SHA-256 as the C#'s. The crate also opens Shaderc from a
path, for a Rust program with no C# beside it. A publish carries the same `libshaderc_shared` it carried before, from
`Silk.NET.Shaderc.Native`, and nothing new.

**SQLite is the one the C# loaded too, and not a second copy.** §2.4 and §2.8 planned `rusqlite` with its own SQLite
bundled, and gave the rule that goes with two copies: one database file, one copy, per process. The cache's own
tests cannot keep that rule: they open the cache's file through Microsoft.Data.Sqlite while a cache has it open, to
hold a lock on it, to count its rows and to damage one (`SpirvCacheTests`), all in one process. With two copies the
locks are POSIX locks of the one process, which neither copy sees of the other, so a cache "locked" by the test
would not be locked to the library, and SQLite's own documentation gives this as a way to corrupt a file. So the
library calls the `e_sqlite3` the C# has loaded, lent by its handle as Shaderc is, through 16 of its functions
written out as SDL's are. One SQLite in the process keeps §2.8's rule by construction; the tests pass unchanged
both ways; a publish gains no library, and the platform library no megabyte of C. What it costs is the waits and
words Microsoft.Data.Sqlite puts round SQLite, which are now the crate's to reproduce (§16.3). A Rust program with
no C# opens SQLite from a path. It is Q25.

**What a reader makes crosses as JSON**, as §3.8 has it: a preset, a source, a reflection, each a document the
facade turns into the C# objects the chain already takes. A float is its 32 bits, so that a NaN and an infinity,
which a preset may hold, cross whole. A reader that fails answers with a document too, since what the C# throws
carries more than words: a `FileNotFoundException` names its file, an `ArgumentException` its parameter. A stage's
SPIR-V and a pruned module cross as bytes. A result too long for the facade's buffer waits on the calling thread
and is taken, so that no file is read and no stage compiled twice to learn a length.

**The cache is a handle that locks itself**, since a preset's passes are compiled side by side through one, and it
compiles outside its lock, as the C# does. Its clock is a C# delegate a test may set; the facade asks it once for
each compile and passes the time in, where the C# asks only when it is about to write, which only a clock that
counts its own calls could tell.

**The compiler's identity names its own binding**, as §4.3 planned, so by default the library does not serve a row
the C# kept, nor the C# one of the library's: a machine that changes the switch compiles its presets once more, and
the other's rows age out under the bound. The file itself is one: the schema is the single `spirv-cache-schema.sql`,
embedded by the C# and included by the crate, and `user_version` 1. Given one identity, each reads the rows the
other wrote as hits (§16.5). Whether the two should share an identity now that their SPIR-V is shown to be the same
is Q26.

**Otherwise than planned, each for a reason:**

- **The two merges take C# objects**, so they cross as JSON and back: `SlangParameters.Merge` and
  `SpirvReflection.Merge`. A merged parameter is the C# object first declared under its id, so that what else it
  carries (a built-in filter's `Choices`) stays with it.
- **`SlangParameters.Read` and `IsHeading` are not exports.** The first is the reader and the merge composed, which
  the facade composes from the two; the second is one comparison, which the crate has for a Rust caller.
- **What the C# is left to answer:** a null argument, text that is not valid UTF-16 (Q19), and a cache path that
  begins `file:`, which Microsoft.Data.Sqlite reads as a URI and no caller passes.
- **A module whose types hold themselves is refused.** The C# recurses until the stack ends, which no `catch`
  survives; the library stops at 256 levels and the facade throws an `ArgumentException`. No compiler emits such a
  module.

### 16.3 .NET's rules, measured for this step

Each was measured in a scratch program on .NET 10 before the code that depends on it was written, and each is a case
of the parity class:

| Rule | What .NET does |
|---|---|
| `\s` in a regular expression, and `char.IsWhiteSpace` | the same 25 characters, Unicode's White_Space, which is Rust's own; U+001C to U+001F are not among them |
| `\w` | 487 ranges of the first plane; half of a surrogate pair is not one, so no character past U+FFFF is |
| `^\s*#reference\s+"?([^"]+)"?` on a line that ends in two or more spaces, or in spaces and a quote | matches, by giving the last space back to the name, which then trims to nothing: the reference is to the preset's own folder, and fails as a file that does not exist |
| An unclosed quoted value | ends at the first `#`, one straight after the quote included |
| An unquoted value | ends at `#`, a space or a tab, and at no other white space |
| `float.TryParse` with `NumberStyles.Float` | white space U+0009 to U+000D and the space, a sign, digits with one point and at least one digit, an exponent only when digits follow its `e`, white space, then any number of NULs; failing that, `Infinity`, `-Infinity`, `NaN`, `+Infinity`, `+NaN` or `-NaN` in any case with any Unicode white space around them, and no NUL; `float.NaN` is the quiet NaN with its sign set |
| `int.TryParse` with `NumberStyles.Integer` | the same without the point and the exponent; past 32 bits is not a number |
| `(int)` of a float | saturates, and a NaN is 0 |
| `File.ReadLines` | decodes as `File.ReadAllText` does, breaks at `\r\n`, `\r` and `\n` and nowhere else, and gives no empty line for a final break |
| `ToLowerInvariant` | the only character outside ASCII it brings inside is the Kelvin sign, to `k`; the dotted capital I stays |
| `Path.GetFullPath` | refuses an empty path with "The value cannot be an empty string." and a NUL with "Null character in path.", both naming `path` |
| Shaderc's binding | hands a file name over as UTF-8 up to its first NUL |
| Microsoft.Data.Sqlite, a busy statement | tried again every 150 ms until the connection's timeout, which the cache sets to one second, each try waiting SQLite's own 250 ms |
| Microsoft.Data.Sqlite, a transaction | `BEGIN IMMEDIATE`, after `PRAGMA read_uncommitted = 0` |
| Microsoft.Data.Sqlite, a failure | "SQLite Error *n*: '*SQLite's words*'." with the primary code |

**One prediction of this step that was wrong:** that an empty path's words were "The path is empty.", recalled from
an older .NET. The parity class found it on its first run.

### 16.4 The default path, and what was tested

**The default is untouched.** Every statement of the six classes at `aa38950c` is still in its file but six, changed
by necessity: `SlangParameters.Merge` gained a form over the lists themselves, three lines; the compiler's `Api`
became internal, since the lending reads its handle; and the C# cache names the C# compiler for its identity and
its compiles. With the variable unset no facade asks the library anything, and Shaderc and SQLite are not lent.

That was checked on a running program and not only read from the code: a small program that reads a preset, merges
its parameters, compiles both stages through a cache, hits the cache and reflects the SPIR-V was run three ways.
With the variable unset or `0` it never asked for Serenity's half of the library, and the process held Shaderc and
SQLite and not `libemusen_platform`; with `1` it asked, and held all three. What it printed of the preset, the
bytes compiled and the cache's hit was the same in the three runs.

| Run | Variable unset | `EMUSEN_SERENITY_NATIVE=1` |
|---|---|---|
| `cargo test` in `Platform/` (21 in `emusen-serenity`, 25 in `emusen-endymion`, 76 in `emusen-galaxia`, 19 in `emusen-platform`) | 141 pass | |
| `cargo clippy` over the workspace; the header as C99 and as C++17 | clean | |
| WiseMan's `Slang*` and `Spirv*` classes of Serenity, the parity class, and Mistress's `ShaderSettingsWindowTests` and `ScreenFilterSettingTests` | 101 pass | 101 pass |
| The parity classes of Galaxia and Endymion, whose header and version this step changed | 66 pass | |

The 101 are 88 that existed and the parity class's 13. The tests that draw ran on the RX 6800 and on no other
device. With the variable set, `SpirvCacheTests`' locked database is waited on and given up as Microsoft.Data.Sqlite
waits and gives up: 1.1 s for each of the two statements skipped, and SQLite's own words for why the cache is off.

**CI** gains two steps, Serenity's slang half on its C# and on the library, which report and do not gate.

**Not done, and not claimed:** Windows and macOS, as before, and in particular whether .NET hands over the two
handles there as it does here; and anything of the device or the chain, which is step 4b.

### 16.5 The parity class, and what it found

It gives the C# and the library the same input and compares what each makes, or the exception each throws with its
type, its words and the file or parameter it names:

- the two character classes against .NET's own answer for each of the 65,536 code units;
- 200,000 numbers: every awkward piece of one, and strings of up to five of them, parsed as a float and as an
  integer;
- 2,500 presets written at random, up to three files deep in references: every spacing (seven characters of white
  space and one that only looks it), quoting, comment, line ending and byte-order mark, keys of every kind with and without their pass's
  number, values right, wrong and absurd, references that loop, miss, name a folder or hold a NUL; and for each
  that reads, its passes' parameters merged;
- 2,500 sources likewise, three files deep in includes: every pragma well and badly formed, includes present,
  absent, optional, empty, circular;
- a preset and a source that are there and may not be read;
- 3,000 merges of parameters, with every float that is not a number among them;
- 24 stages compiled, of which some do not compile, with names and text outside ASCII and a NUL in a name: the same
  bytes, or the same words;
- every module those make, and 21 written by hand to be wrong in one way each, and 150 damaged copies of each good
  one: reflected, pruned and merged;
- a cache of each kind on its own file through 12 runs of 40 steps: stages compiled, hit, bounded and evicted
  under a clock, a stage that does not compile, and the files closed, one row damaged in each alike, and opened
  again; after each step the counts, and at each close every row of both files;
- a file the C# wrote read by the library, added to, and read back by the C#, and the other way about;
- a file that is no database, one of another schema version, a row that is not a blob, a path that cannot hold a
  file, an empty path, and a database another connection is writing, where both give SQLite's own words;
- a key over 2,000 sets of parts;
- and, when a run names where libretro's pack is, every preset, source and stage of it (§16.6).

**What the first run found:** one difference, the words for an empty path (§16.3), in the preset reader and the
cache alike. Nothing else differed in any reader, in a single compiled byte, in a reflection or in a row.

**Seeded faults: 83, in every part of the step.** The class as first written caught 72. Nine it missed were gaps
in what it asked, and are closed:

- five in the preset reader, all the same gap: a random key was seldom given a value of its own kind in an unusual
  form, so a wrap mode or `true` in capitals, a scale that is no number beside a pass's scale that is, a count with
  a fraction, and a value with a no-break space in it were not met. A key is now given a value of its kind three
  times in five, and one preset asks all of them at once and is held to what the C# makes of it;
- the depth at which includes are taken for a loop, which no random file reached: chains of 32 and 33 includes and
  of 16 and 17 references are now read by both;
- two in the merge of two stages' blocks, since every stage in the corpus declared the same block at the same
  binding: two stages now declare another;
- one in the cache's waiting: a statement not tried again fails sooner and counts the same, so the time a blocked
  statement takes is now held to the connection's second.

The last two survive and are not gaps: Shaderc ignores the entry point's name for GLSL, so another name compiles to
the same bytes; and the cache never binds an empty blob, since no stage compiles to nothing.

### 16.6 P6, and publishing

**P6 holds.** Over libretro's slang pack, read where a run names it and never written: 2,658 presets and the 1,350
sources they name were read by both readers to the same result, nine of the sources refused by both alike; the
other 1,341 gave 2,682 stages, every one of which compiled through both to the same SPIR-V, byte for byte, with the
same reflection and the same pruned module. The run took 3 minutes 22 seconds on the desktop, and is the last case
of the parity class, which passes by when no pack is named.

**Publishing.** A self-contained linux-x64 publish of Mistress has the same 454 files as one of `aa38950c` built
the same way beside it; `libemusen_platform.so` grows from 953,664 to 1,147,432 bytes, so P7's 4 MB holds with all
three crates in, the Vulkan half still to come. `libshaderc_shared.so` and `libe_sqlite3.so` are the files the
publish carried before, beside the assemblies, and the lending finds both there: a small program published
self-contained and run with the variable set compiled a stage and hit its cache through the library.

### 16.7 Questions for the tester

| | Question | As built |
|---|---|---|
| Q25 | The cache's SQLite: `rusqlite` with its own copy bundled, as Q7 and §2.8 had it, or the one the program ships | the one the program ships, lent by its handle: two copies in one process cannot share a file's locks, which the cache's own tests rely on (§16.2). `rusqlite` is not taken |
| Q26 | Whether the library's compiler identity should be the C#'s, so that each serves the other's rows | separate, as §4.3 planned; P6 held over the whole pack, so one identity would be safe and would spare a machine one rebuild of its presets when the switch changes |

*Both decided 2026-10-09, as built (§9).*

---

## 17. What was built, 2026-10-09: step 4b, Serenity's Vulkan device and chain

Measured on the development desktop, on its Radeon RX 6800 and on no other device. A preset is built into a chain on
a Vulkan device, fed frames and drawn by a Rust form of `SlangVulkan` and `SlangChain`, behind the switch step 4a
made, `EMUSEN_SERENITY_NATIVE`, still in state 1: the C# runs unless the variable is `1`. No C# was removed, and the
C# device and chain were moved into `Managed` classes and not rewritten (§17.4). `SlangRunner` is C#, as §4.3 kept
it, and calls whichever chain the switch gives it; the SkSL filters are no part of this work. With this step all of
§4.3 is built; Serenity stays in state 1, and its move to state 2 is a later step at the tester's word.

### 17.1 The artefacts

| What | Where |
|---|---|
| `SlangVulkan`: the device, its buffers, images and samplers, its one command buffer, the layer's findings, a bench's probe | `Platform/serenity/src/vulkan.rs` |
| `SlangChain`: passes, pipelines, history, feedback, parameters, what a name means and what a block is filled with, the readbacks and their lending | `src/chain.rs` |
| A pass's `#pragma format`, as the C# binding's enumeration reads it | `src/format.rs` |
| The C layer: 20 exports, 172 in the library; the interface's version is 6; a new status, `DEVICE` −1028 | `Platform/platform/src/serenity_device.rs`, `include/emusen_platform.h` |
| The entry points, and the two structs that cross | `EmuSen.Serenity/Native/SerenityNative.cs` |
| The facades, each over its `Managed` class | `EmuSen.Serenity/Slang/SlangVulkan.cs`, `SlangChain.cs` |
| The probe's two new points | `EmuSen.Serenity/Slang/SlangProbe.cs` |
| The parity class, 24 cases | `EmuSen.WiseMan/Serenity/SerenityChainParityTests.cs` |

About 1,910 lines of Rust in the crate and 490 in its C layer, with 265 of tests, against §7.2's 2,200 for the step
with its tests. `ash` is the one crate added, for Vulkan's declarations (§17.2).

### 17.2 The device, the chain, the lent readback, and what was built otherwise than planned

**The device lives until the last thing made on it is gone.** A buffer and an image each hold the device they were
made on, so closing a device lets go of one hold among several, and Vulkan's device is destroyed when the last goes.
This is what §3.3 asked for in one sentence ("a chain freed under a held slot leaves the slot to the release") taken
to its end: a picture Skia still holds keeps its readback, the readback keeps the device, and the pixels stay where
they are until the picture lets go, on whatever thread. The C# destroys its device when asked, held pictures or
not, which by its code leaves a held picture over memory Vulkan has taken back; that is argued and was not run,
since it would read freed memory. The parity class does it to the library (§17.5).

**It locks itself, and waits under its lock.** One command buffer is recorded, submitted and waited for behind one
lock, as in the C#, so a chain is built on one thread while another draws. Waiting for the device to fall idle, which
a chain does when a frame changes size and when it ends, is done under the same lock, because Vulkan asks that
nothing else use the queue meanwhile. The C# waits outside its lock, and the layer says so when two chains on two
threads share a device (§17.4).

**What the layer says waits to be taken**, as §3.7 had it. With validation asked for, the device is made with the
Khronos layer, the same three settings the C# gives it, and a listener that queues each finding; the facade takes
them after every call into the device or a chain on it and hands each to the sink the test gave. So a finding
reaches the sink after the call that caused it returns and not during it, and on the thread that made the call. A
test that makes a Vulkan call of its own has to ask for the findings (`Heard`), which `SlangSyncTests`' check that
the layer is listening now does. Closing is in two steps so that what the layer says of the device's end is heard:
the device is let go of, the findings are taken, and then the handle is freed.

**The device's and the instance's own handles are exported**, for that check: it creates a buffer of no size
through the C# binding on the library's device, which the binding can do with nothing more than the handle. They are
Vulkan's handles and not pointers into the library, and the header says what may not be done with them (the queue is
the library's). A Rust client that presents the chain's picture itself would need them in any case. It is Q29.

**A chain is built by the library from the preset alone.** The C# preset crosses as the document the library's own
reader answers with (§16.2); the library reads each pass's source, compiles its stages through the cache it is given
or through Shaderc, reads the SPIR-V back, prunes the fragment stage and makes the pipeline, as many passes at a time
as it is told. Built in order it stops at the first pass that fails; built side by side it builds every pass and
gives the lowest-numbered failure, as the C# does, which the cache's counts show (§17.5). A failure crosses as a
document, as a reader's does, because the exception the C# throws for it has a type: a file not found with its name,
`SlangCompileException` with Shaderc's words, `InvalidOperationException` with the Vulkan call and its result,
`OverflowException` for a history too deep to count.

**A lookup image is decoded by the host.** The C# decodes it with Skia and draws it into an RGBA bitmap; a picture
is byte for byte the same only if those pixels are, and no other decoder is held to Skia's. So the facade decodes,
in the C# chain's own four lines, and hands the pixels over, one call for each image, after the passes are built: the
order the C# fails in is kept, a pass that does not compile before an image that does not decode. A Rust program
with no C# beside it will need a decoder, and it is Q27.

**The lent readback is §3.3's**, with its struct as declared there. A readback's number comes from a counter and
names it only while it is lent, in a table of the lent ones, so that a second release of a number is `ABSENT` and
does nothing, where a pointer for a number would have been freed twice. When every readback a chain may lend is
held, the picture is in the chain's own readback and the number is zero: the facade copies it, as the C# does. The
facade keeps the object Skia's release is given for each number from one frame to the next, so a frame allocates
only what Skia's picture is (§17.6).

**A bench's probe is the device's own.** The C# is listened to through `SlangProbe`, which is handed each command
buffer to write timestamps into; the library records the same points itself when its probe is on, the phases against
a clock started when it was turned on and each submission's timestamps read as soon as it has been waited for, and
the facade hands them to the same listener after each call. `ISlangProbe` gained the two points this takes, and the
bench reports the same stages either way (§17.6).

**Vulkan's loader is opened through `libloading`**, by the name each system knows it by, and `ash` is taken without
its default loader, which would have brought a second, older `libloading` into the library. `ash` is then
declarations and function tables alone.

**One status is new: `DEVICE`, −1028**, for no Vulkan device and for a Vulkan call that failed, at the number §3.5
kept for the first of the two. The facade throws `InvalidOperationException` for it with the library's words, which
are the C#'s: the call's name and the result as the C# binding prints it.

**Otherwise than planned, each for a reason:**

- **The chain is the device's kind.** A chain on the library's device is the library's and one on the C#'s is the
  C#'s; the switch is asked once, when a device is made. A test makes one of each and compares them.
- **Parameter values cross as floats in the parameters' order**, not by name: the facade looks each id up in the
  dictionary it was given, so that a dictionary that compares its keys its own way is still asked its own way.
- **A cache that is the C#'s cannot be compiled through by the library.** A chain on the library's device given
  one compiles each stage itself, as a chain given no cache does, and its pictures are the same. The two are made
  by one switch, so this arises only for a cache whose path the library does not take (§16.2), which no caller
  passes.
- **`SlangChain.ParseFormat` is an export of its own**, since it is public and its answer is the C# binding's: a
  pragma's word is a format when the binding's enumeration has a name for it once spelled the binding's way. The
  crate holds the enumeration's 322 names (§17.3).
- **What the C# is left to answer:** a device asked for by a name that is not valid UTF-16, which is then the C#
  device, and a format's word that is not, or is null.

### 17.3 .NET's and the binding's rules, measured for this step

Each was measured in a scratch program on .NET 10 with Silk.NET.Vulkan 2.23.0 before the code that depends on it
was written, and each is a case of the parity class or of the crate's own tests:

| Rule | What .NET does |
|---|---|
| `Enum.TryParse` of a pragma's word | white space before the word is dropped; a word that then begins with a digit or a sign is a number, and any 32-bit number is a format whether the enumeration defines it or not; white space of the six ASCII kinds may follow a number, then any number of NULs, and nothing else; a number past 32 bits is no format |
| the same, for a word that is not a number | names between commas, each with the white space round it dropped, every one of which must be a name with its case exact; the answer is their values or-ed together, for an enumeration that is not one of flags too |
| the spelling before it | the text before the first underscore as it is, each later part with its first letter raised and the rest lowered: so `ASTC_4x4_UNORM_BLOCK` is no format, since the binding spells it `Astc4x4UnormBlock`, and `1_000` is format 1000 |
| `char.ToUpperInvariant` and `string.ToLowerInvariant` | the only character outside ASCII that the first brings inside is the long s, to `S`; the only one the second does is the Kelvin sign, to `k`. The dotless i is not raised to `I` |
| the binding's `Format` | 322 names for 265 values |
| the binding's `Result`, printed | 52 names; a value it does not name prints as its number |
| `MemoryPropertyFlags`, printed | the names joined by a comma and a space |
| `(uint)Math.Round` of a float product | the product in single precision, rounded as a double with a half going to the even number, then held at the ends of 32 bits; what is no number is 0 |
| `(uint)Math.Floor(Math.Log2(n)) + 1` | the number of bits in `n`, and 1 for 0 |
| `new T[n + 1]` for `n` of `int.MaxValue` | `OverflowException` |
| `Contains` with `OrdinalIgnoreCase` | by Galaxia's rule for an ordinal comparison (§11.3): the long s is no `s` |

### 17.4 The default path, and what was tested

**The C# device and chain are the classes they were**, each now a `Managed` class inside its facade. Compared with
`f5ef19f7` line for line, indentation aside, `SlangVulkan.Managed` differs from the old `SlangVulkan` in ten lines
and `SlangChain.Managed` from the old `SlangChain` in eight: each constructor's name and the type of the device it
takes, the public pieces that moved up into the facade (the variable's name, the bound on lent readbacks, the count
of builders, the public constructors), and one line added, which lets the bench read the instance without
reflection. `SlangBuffer` and `SlangImage` name `SlangVulkan.Managed` as their owner. Nothing else in either moved.

That was checked on a running program too: §8.1's bench host (§17.6), built once from this branch and once from
`475fe4c3`, drew `crt-royale` four ways and reported at its end whether Serenity's half of the library had been asked
for and which kind of device the runner drew on. From the base, and from this branch with the variable unset or `0`,
it was never asked for and the device was the C#'s; with `1` it was asked for and the device was the library's. The
three hashed pictures were the same in all four runs.

| Run | Variable unset | `EMUSEN_SERENITY_NATIVE=1` |
|---|---|---|
| `cargo test` in `Platform/` (30 in `emusen-serenity`, 25 in `emusen-endymion`, 77 in `emusen-galaxia`, 22 in `emusen-platform`) | 154 pass | |
| `cargo clippy` over the workspace; the header as C99 and as C++17 | clean | |
| WiseMan's `Slang*` and `Spirv*` classes of Serenity, its two parity classes, `ShaderBenchTests`, and Mistress's `ShaderSettingsWindowTests` and `ScreenFilterSettingTests`, with libretro's pack named | 125 pass | 125 pass |
| The classes that draw (`SlangChainTests`, `SlangReadbackTests`, `SlangSyncTests`, `SlangRowsOnceTests`, `SlangFrameControlTests`, `SpirvCacheTests`), with the layer forced onto every device they make (below) | 56 pass | 56 pass |
| The chain parity class with the layer forced onto both devices | 24 pass | |
| `ShaderBenchTests` with a GL device asked for (`EMUSEN_SHADER_BENCH=1`), which the row above passes by | 1 pass | 1 pass |
| The parity classes of Galaxia and Endymion, whose header and version this step changed | 66 pass | |

The 125 are 88 that existed, step 4a's parity class less its case over the whole pack (12, run in step 4a and not
in the blast radius of this one), the bench's test, and the new class's 24. The bench's own pictures in the gated
`ShaderBenchTests` were the same three hashes both ways, and it allocated 3.7 KB a frame on the C# and 2.1 KB on the
library.

**The layer, and what it said.** The tests that ask for the layer (`SlangSyncTests`, and three cases of the parity
class) pass both ways with their sinks empty but for the fault each provokes to show the layer is listening. Beyond
them, the classes that make a device and draw were run with the layer forced onto every device they make, by
`VK_INSTANCE_LAYERS`, with synchronisation validation and `VK_LAYER_SYNCVAL_SHADER_ACCESSES_HEURISTIC=1`, and its
words gathered in a file. Two things about that file are recorded because both cost a run to learn: the layer writes
nothing to it, though it opens it, unless `VK_LAYER_DEBUG_ACTION=VK_DBG_LAYER_ACTION_LOG_MSG` is given when the
program has a listener of its own; and every instance opens the file anew and writes from its beginning, over what
the last instance wrote, so the file was made a named pipe with one reader gathering what every instance says.

- **On the library's device, the layer said nothing** but the faults the tests provoke: in the run of the drawing
  classes with it forced on, the four buffers of no size `SlangSyncTests` makes to show it is listening, and
  nothing else.
- **On the C#'s, the same four**, and in one run of three, once, `UNASSIGNED-Threading-MultipleThreads-Write` on
  the queue, from `SpirvCacheTests`' two chains built at once on one device. Four runs of that class alone on each
  device placed it: three of four on the C# drew it, at `vkQueueSubmit` once and `vkDeviceWaitIdle` twice; none of
  four on the library did (§17.5).
- **In the chain parity class with the layer forced onto both devices**, the provoked faults of each, two
  `WARNING-Shader-OutputNotConsumed` of the deliberately unread vertex output of the preset of §17.5, and 74
  objects left at the end of the C# devices by builds that fail part way. None of them was the library's.

The gate's other half, that the layer be silent, is held by the test as well as by these runs: three cases of the
parity class and `SlangSyncTests` give the library's device a sink and assert it empty after real work, and the
parity class then provokes, on both devices, a fault the layer always reports and a write-after-write hazard that
only synchronisation validation reports, and asserts both are heard. The second was added by this step's seeded
faults (§17.5): without it a device made with synchronisation validation off would have been silent too, and the
silence counted.

**CI** runs the new parity class in the step that tests Serenity on the library; a runner has no Vulkan device, so
there its cases that draw pass by, and those that need none (the format's reading, the exports, the structs' sizes)
run. Both steps still report and do not gate.

**Not done, and not claimed:** Windows and macOS, as before; any device but the RX 6800 (the desktop's integrated
Radeon and llvmpipe were not run, and the handheld and the laptop were not touched); and a real window, since
everything here is the chain's own pictures and the bench's, as in `EmuSen_Serenity.md` §9.8.

### 17.5 The parity class, and what it found

It makes two devices on the one card, the C#'s and the library's, builds the same preset on each, feeds both the
same frames and compares every picture byte for byte; or the exception each throws, with its type and its words:

- that both choose the same device, whatever the case of the name asked for and for no name at all, and give the
  same report for a name no device has;
- 61,000 words read as a pragma's format by both: every name the binding has, as it has it and as a pragma would
  spell it, pairs of them with a comma between, and strings of up to five pieces of names, numbers, signs, white
  space of nine kinds, NULs and the four characters whose case matters;
- a pass that gives the picture back, at several sizes and with its parameter set; three passes that are given a
  float target by the preset's word, an 8-bit one, and a last pass that asks for sRGB and for float and is given
  neither; a pass over frames with a mip chain;
- a preset of three passes written to read every name a pass may: the source through each wrap and each filter and
  past its edges, a mip chain at a level, between two and past its last, the newest frame and older ones, feedback
  by number and by alias, an earlier pass by number, by the preset's alias and by the source's own name, a later
  pass by number and by alias (which is nothing), a name that is nothing, two lookup images, every size the C#
  knows and one it does not, every count, and two parameters, one of them declared twice. Its sizes fall on halves,
  so that the rounding shows; it is drawn through seven frames, with rows repeated, with its parameters set, and
  built side by side;
- the same preset through frames that change size, repeat and viewport, with a redraw after each pair;
- parameters declared, valued and set alike, through dictionaries that hold no such id, a value that is no number,
  and keys compared without their case;
- a chain that cannot be built: a shader that is not there, one that does not compile, a history too deep to count,
  a lookup image that does not decode and one that is not there, two failing passes built in order and side by
  side; a render before any frame; a frame with a byte too few, and one with bytes to spare;
- a cache of each kind through three builds, in order and side by side: the same counts after each, the same
  pictures from a chain built through a cache and one built without, the same counts after a build that fails part
  way; and a cache the C# filled read by the library's chain as six hits and no miss;
- readbacks lent, held, copied and given back through one scripted run of both chains: the same count of readbacks
  and of copies after every picture, no held picture written over, one given back on another thread, a larger
  picture, and both chains ended while pictures are held;
- on the library alone, a picture held past its chain and its device, which keeps its pixels;
- the layer, with a sink: the library's chain through the preset above, built side by side, with pictures lent
  and held, and nothing said; then a buffer of no size, and the rule it breaks said; the same on both devices side
  by side; and what each device is left holding by builds that fail (below);
- the probe: each phase and each pass's timestamp, in order, on the stopwatch's clock;
- what a frame allocates through each chain (§17.6);
- and, when a run names where libretro's pack is, eight runs of seven of its presets, among them `crt-royale`,
  `crt-guest-advanced` and Mega Bezel's `POTATO` (ten passes, seven lookup images, 436 parameters): four frames
  each, then every parameter set half way to its largest and two more at another size.

**What the first run found: no picture differed.** Twenty of the class's first twenty-two cases passed as written,
and the two that failed were mistakes in the cases (a pass numbered 2 taken for the point numbered 2; a rule's name
looked for at the start of the layer's words, where it is not). Every picture of every preset, the pack's among
them, was the same from both on the first run.

**What it found in the C#**, which is left as it is, since no C# is rewritten in this step. Each is a case where the
two differ and the library's answer is the one kept:

- **A chain that fails after its passes are built leaves them on the device.** When a lookup image does not decode,
  or the history is too deep, the C# constructor throws with its passes made and nothing frees them. Under the
  layer, six such builds left 64 objects on the C# device at its end and none on the library's, whose chain is
  dropped whole. The exception is the same from both.
- **The device is waited for outside its lock.** `Advance` and `Dispose` call `vkDeviceWaitIdle` without the lock
  that guards the queue, so two chains on two threads on one device can have the queue in use twice at once. With
  the layer forced on, `SpirvCacheTests`' two chains built at once on the C# device drew
  `UNASSIGNED-Threading-MultipleThreads-Write` on the queue; the same run on the library drew nothing.
- **A second `RenderImage` before any frame** throws `NullReferenceException` in the C#: the first leaves a readback
  with no buffer behind, which the second asks the size of. The library says again that no frame has been given.
  Run in a scratch program both ways; `SlangRunner` always gives a frame first.
- **A device disposed under a held picture** is destroyed by the C# with the picture's memory, by its code and not
  run; the library's lives until the picture lets go (§17.2).

They are Q28. *All four were fixed in the C# on 2026-10-10, each shown first by a test that failed (§18).*

**Seeded faults: 99, in every part of the step**: the format's reading, the device's choosing, sampling, mip chains,
transitions, uploads, the host barrier, the layer and the probe; the chain's building, its formats, its history,
its sizes, what every name means and what every member is filled with; parameters, readbacks and their lending; and
the C layer. Each was built into the library, the class run against it, and the file put back; the files seeded
were compared with copies taken before the first, and were the same after the last. The class as first written
caught 90, one of them by bringing its test host down (a chain's end freeing a readback a picture held). Of the
other nine:

- one was written wrong, its text not found in the file; written right, it is caught;
- three were gaps in what the class asked, and are closed. No pass declared a parameter with a size's name, so
  the order of the two rules was not met: the preset of three passes now declares `MidSize`. No case made sure
  synchronisation validation was on, so a device made without it was silent and passed: the class now provokes a
  hazard only it reports (above). And no case looked at where a picture's pixels are, so a readback made anew each
  frame, rather than lent again, gave the same pictures and counts: the class now asserts one readback's memory
  serves every frame a picture is let go;
- one is caught by the crate's own test and not by the class, since no Vulkan call fails in a test: the result's
  name in a failure's words;
- four survive and are not gaps. Two are equivalent: a transition of some levels from the first taken for all
  of them, which every path follows with a transition of all; and the history kept when only its height changes,
  which cannot change unless the rows or their repeat do, and both are compared. Two cannot be reached on this
  card: the barrier that makes a readback visible to the host, since the memory is coherent and the layer does not
  follow the host's reads (`EmuSen_Serenity.md` §9.7 says the same of the C#'s); and a block's bytes not cleared
  before they are filled, since RADV gives new memory cleared and nothing else writes those bytes.

**One more thing it found, in a test of the C#'s:** `SlangSyncTests` shows the layer is listening by a buffer of no
size, which is reported whether synchronisation validation is on or not. It has the gap the parity class had and is
left as it is; it is an open item (§15).

### 17.6 P5, P7, the bench, and publishing

**The bench's pictures are the same.** §8.1's bench of `EmuSen_Serenity.md` (the real `GameFrameControl` draw path
on a GL context, the preset on the shared device, 60 frames of warm-up and 300 measured at 60 Hz, three more hashed
off the surface) was run for six cases in a 1080p window on the C# and on the library, twice each, interleaved,
each run its own process: Lottes, `crt-guest-advanced`, `crt-royale`, Mega Bezel's `POTATO` and `SMOOTH-ADV` at
256×224, and `crt-royale` at 2560×960 with its rows twice. The three hashes of every case were the same in all four
runs. The base commit's bench, built from `475fe4c3` with the C# chain, drew `crt-royale` with the same three hashes as
this branch's both ways.

**P5 holds, with its first clause said more exactly.** What the C# chain allocates a frame was its passes' lists
in `Bind`; the library's chain allocates nothing managed, and what is left of a frame's allocation is the picture
Skia is handed, which the facade makes in C#:

| A frame taken and a picture made and let go, bytes allocated on the drawing thread | C# chain | Library's |
|---|---|---|
| one pass | 1,072 | 312 |
| three passes | 4,848 | 312 |
| `crt-royale`, 12 passes | 14,608 | 312 |
| Mega Bezel `POTATO`, 10 passes | 12,512 | 312 |

The 312 bytes are the same whatever the preset, which the parity class holds. In the bench, where the control and
the bench's own probe allocate too, a frame fell from 3.7 to 2.1 KB for Lottes, from 15.1–18.6 to 3.7–3.8 KB for
`crt-guest-advanced`, `crt-royale` and `POTATO`, and from 120.6 to 8.1 KB for `SMOOTH-ADV`. The C# runs made a
generation-0 collection in four of twelve runs and a generation-1 in one; the library's twelve made none. No run
of either made a generation-2 collection, so the one `EmuSen_Serenity.md` §9.8 saw in N64 4× royale was not seen
here to be removed.

**GPU time does not change.** The passes' time agreed within 2% in five of the six cases (0.54 against 0.53 ms for
`crt-royale`, 2.24 against 2.22 for `SMOOTH-ADV`), which is less than one implementation differs from itself between
rounds. Lottes did not at first: 0.66 ms on the C# and 0.73 on the library in both rounds. Three more runs of each in a
changed order gave 0.64 to 0.72 for the C# and 0.64 to 0.74 for the library, the same range, and whichever ran
first after a pause was the faster: this one pass's time follows the card's clock, as §8.1 there records. Run flat
out, where the clock is steady, it is 0.474 ms in three runs of each, the same to the last digit. On the processor
the sizing and binding of a frame fell from 0.004 to 0.001 ms for one pass and the render thread's whole frame by
about 1%, which is not a reason to do anything.

**P7 holds.** The platform library a publish carries is 1,445,832 bytes with all three crates in it, against 4 MB;
it was 1,147,432 before this step, measured the same way in step 4a.

**Publishing.** A self-contained linux-x64 publish of Mistress has the same 454 files as one of `475fe4c3` built
the same way beside it. Vulkan's loader is the system's, as it is for Silk.NET, and nothing else is loaded that the
publish did not carry before.

### 17.7 Questions for the tester

| | Question | As built |
|---|---|---|
| Q27 | What decodes a lookup image when there is no C# | the host decodes with Skia and hands the pixels over, since the pictures are the same only if the decoder is (§17.2). A Rust program will need a decoder held to Skia's pixels, or the pictures re-recorded |
| Q28 | The four differences found in the C# chain and device (§17.5): mend the C# now, or leave it to its retirement | left as they are: no C# is rewritten in this step, none is reached by a frontend today, and the library does not have them. Mending the wait outside the lock is one line and a test with the layer forced on |
| Q29 | Vulkan's own handles for the instance and the device crossing the interface | exported, for the test that shows the layer is listening, and for a client that will present the picture itself (§17.2) |
| Q30 | −1028 as `DEVICE`, for no device and for a call a device refused, where §3.5 named it `NO_DEVICE` | one status for both, since the host does one thing with either |

---

## 18. The C# slang runtime's four defects, shown and fixed, 2026-10-10

Step 4b's parity class found four places where the C# chain and device do what the library's do not (§17.5), and
the tester decided they be fixed now (Q28), since the C# is what players run. Each was first shown by a case of a new
class, `EmuSen.WiseMan/Serenity/SlangDefectTests.cs`, failing on the C#; then fixed with the smallest change that
makes the C# do what the library does, in its own commit; then the case run again on the C# and, with
`EMUSEN_SERENITY_NATIVE=1`, on the library. The cases run on whichever device the switch gives, under the Khronos
layer with a sink, on the RX 6800. These fixes are in the `Managed` classes, the one exception to their being moved
and not rewritten.

| | Defect | How it was shown, on the C# | The fix |
|---|---|---|---|
| 18.1 | A chain that fails after its passes are built leaves them on the device | building one preset whose lookup image does not decode and one whose history is too deep to count, then ending the device: **4 buffers and images never given back, and the layer found 32 objects left** at `vkDestroyDevice`. The library: none | the rest of the constructor, after the passes, disposes the chain on any failure and rethrows; `Dispose` minds what was not yet made |
| 18.2 | The device is waited for outside the lock its queue is used under | a chain kept the queue busy on one thread while another changed size every frame, taking the device's wait: **`vkDeviceWaitIdle(): THREADING ERROR : object of type VkQueue is simultaneously used`** in three runs of three. The library: silent | the device waits for itself under its submit lock (`WaitIdle`), and the chain asks it to; three runs after, silent |
| 18.3 | A second picture asked for before any frame throws `NullReferenceException` | asking three times: the first said "No frame has been given to the chain yet.", **the second threw `NullReferenceException`**. The library: the same words three times | a readback with no buffer, which the first failed render leaves, is let go of like one too small |
| 18.4 | A device disposed under a held picture frees the picture's memory | without reading that memory: the chain and the device were disposed while a picture was held, and the layer's findings were read before the pixels. **The layer had already said `vkDestroyDevice(): Object Tracking - ... VkBuffer ... has not been destroyed`** of the readback. The library: silent, and the pixels intact | a device let go of ends when the last buffer or image made on it is given back, under a lock of its own, so a release on Skia's thread may end it |

**The device counts its buffers and images.** To show 18.1 by count and not only by the layer's words, the C#
device counts what it has made and not yet been given back (`Live`, for a test); 18.4's fix is built on the same
count. After the fixes the count is zero at the end of every case.

**After the fixes**, the four cases pass on the C# and on the library, and the chain parity class's case of failing
builds now holds the C# device to ending with nothing too: under the layer it held 64 objects there before, and now
none.

**`SlangSyncTests`' check that the layer is listening** made only a buffer of no size, which the layer reports
whatever its settings (§17.5). With synchronisation validation seeded off in the C# device, its six cases all
passed. It now also records the parity class's write-after-write hazard, which only synchronisation validation
reports: with the same fault four of the six fail, and with the C# restored all six pass both ways. The fault was
put back before the next build, and the file is the committed text.

**What was tested,** both ways, with the pack named and the heuristic on:

| Run | Variable unset | `EMUSEN_SERENITY_NATIVE=1` |
|---|---|---|
| `SlangDefectTests` before the fixes | 4 fail | 4 pass |
| `SlangDefectTests` after | 4 pass | 4 pass |
| Serenity's `Slang*` and `Spirv*` classes, both parity classes, `ShaderBenchTests`, `ShaderSettingsWindowTests`, `ScreenFilterSettingTests` | 129 pass | 129 pass |
| The drawing classes with the layer forced onto every C# device | 56 pass | |

In the last row the layer said only what the tests provoke: the threading finding of §17.4 is gone. The threading
case as first written passed on the C#, since two light chains seldom met at the queue; it was made to keep the queue
busy with long submissions before any fix, after which it failed every run on the C#.

---

## 19. Endymion and Serenity in state 2, 2026-10-10

The tester gave the two components' gates their go on 2026-10-10, after the four C# defects of §18 were fixed.
Endymion and Serenity's slang half now run on the library by default, as Galaxia has since step 2c, and
`EMUSEN_ENDYMION_NATIVE=0` and `EMUSEN_SERENITY_NATIVE=0` select their C#. No C# was removed, and
`EmuSenPlatformRequired` stays false, since every rule still has its C# behind it. Each component is its own commit.

### 19.1 The gate's evidence

A full WiseMan run on 2026-10-10 with `EMUSEN_ENDYMION_NATIVE=1` and `EMUSEN_SERENITY_NATIVE=1`, Galaxia on its
default, the library, passed 9,895 cases, failed none and skipped 53, in 18 minutes 39 seconds. It is the gate of
each component: with §12.5's, §14.5's, §16.5's and §17.5's parity classes and their seeded faults, it is what §3.9
asks before state 2. No case outside the components' own classes differed from a run on the C#.

### 19.2 The switches in state 2

Both are the same class as Galaxia's, `PlatformSwitch`, which became public and names its component in its line; no
second switch was written. The table of §13.2 holds for each, with its own variable:

| The variable | The library loads, and is lent what it calls | The component runs on | Said |
|---|---|---|---|
| `0` | not asked | the C# | nothing |
| unset, `1` or any other value | yes | the library | nothing |
| unset, `1` or any other value | no: absent, refused, its load throws, or a lending fails | the C# | one line, once |

The choice is made once in a process, on the first question any of the component's facades asks. What "loads"
means is each component's own: Endymion's every export resolved; Serenity's every export resolved and the Shaderc
and SQLite the C# loaded lent to it (§16.2). Endymion's device half follows Endymion's switch and needs SDL lent as
well; a library that cannot take it leaves the devices alone on the C#, with a line of their own, once. The lines
go where Galaxia's goes, the host's `ConfigDiagnostics` sink and the error log as a warning of area `platform`:

```
<time> WARN [platform] libemusen_platform.so is not in use (libemusen_platform.so not found beside the assemblies); Endymion runs on its C# implementation.
<time> WARN [platform] libemusen_platform.so is not in use (libemusen_platform.so not found beside the assemblies); Serenity's slang half runs on its C# implementation.
<time> WARN [platform] libemusen_platform.so is not in use for Endymion's devices (<why SDL was not lent>); they run on their C# implementation.
```

**Writing a component's line asks Galaxia's switch**, since the error log is Galaxia's, so with the library missing
the first component asked writes its line after Galaxia has chosen and written its own; each switch is asked once
and says one line, whatever the order. This was seen in a running program (§19.5).

**The consequence §13.2 recorded for Galaxia holds now for all three:** a crash in the library is a crash of the
default path, for audio, pads and shaders as for config, which is why the crash log of §3.5 is installed when the
library loads.

### 19.3 Endymion's split (Q23)

A pad manager over a device layer the library does not know runs the C#, chosen for the instance (§14.2): the
library knows SDL's pads and its own simulated ones. State 2 changes nothing there. The two layers the programs use,
`SdlPadDevices` and `SimulatedPads`, are both known, so every manager a program makes runs on the library by
default; the only other layers in the repository are two tests' wrappers, which run the C# as they did. That choice
is the manager's and not a fallback, so it writes no line. Q23, what such a layer becomes once there is no C#, stays
with Endymion's state 3.

### 19.4 Serenity's compiler identity (Q26), measured

The library's compiler names its own binding, so the rows a machine's cache holds from the C# are not the
library's: after the update every preset's stages are compiled once more, and the old rows age out under the
cache's bound. What that costs, on the bench of `EmuSen_Serenity.md` §8.1 (its `load` case, a preset built with
the program's builders, 15 on the desktop), through one cache file:

| Mega Bezel preset | C#, cold | C#, warm | library, first after the C# | library, warm | C# again |
|---|---|---|---|---|---|
| `SMOOTH-ADV`, 48 passes, 96 stages | 674 ms, 92 misses | 471 ms | **554 ms, 92 misses** | 425 ms | 471 ms, 96 hits |
| `POTATO`, 10 passes, 20 stages | 471 ms, 20 misses | 412 ms | **473 ms, 20 misses** | 404 ms | 410 ms, 20 hits |

So the update costs a desktop 60 to 130 ms once for each Mega Bezel preset, the first time it is drawn; the four
stages of `SMOOTH-ADV` that hit even on a cold cache are ones the preset uses more than once, found under the key
their first compile stored, on either side. The cost grows with the stages compiled and falls with the builders: compiled one after another, a
`SMOOTH-ADV` load spends 3.7 s in Shaderc, so a machine of four cores would pay about a second once. Both sets of
rows are kept and each is served to its own side, so turning the switch back and forth compiles nothing more.

**Whether Q26 should now be decided:** it can be, and the recommendation is to keep the identities separate, as
built. P6 held over the whole pack, so one identity would be safe; but the cost it would spare is paid once per
preset on each machine, and keeping the identities apart keeps a cache the C# filled from ever serving the
library's compile, and the other way about, while the two can still be chosen between. When the C# is retired the
question disappears. The identity was not changed.

*Decided 2026-10-10: separate, as recommended (§9).*

### 19.5 What is lent from the C#, and a missing library

SDL3, Shaderc and SQLite are loaded by the C#'s bindings and lent to the library; a fallback leaves them where they
were. Shown in two ways, with the library removed from a copy of the build:

- **WiseMan's audio, pad and shader classes**, run from a copy of its build with no `libemusen_platform.so` beside
  it and every variable unset: `Audio.`, `Input.`, `AudioProperties`, Serenity's `Slang*` and `Spirv*`, the two
  switch classes, `ShaderSettingsWindowTests` and `ScreenFilterSettingTests`, without the parity classes, which
  need the library: 279 pass.
- **A small program** that plays thirty frames of audio through `AudioPlayer` on SDL's dummy driver, presses a
  simulated pad through a `GamepadManager`, and draws a preset through a cache on the RX 6800, run three ways: from
  its own build on the defaults, every switch then on the library; with both variables `0`, on the C#; and from a
  copy with no library, on the C#. All three played, saw the pad held and drew the same picture. The first two wrote
  nothing to the log; the third wrote Galaxia's line, Endymion's and Serenity's, one each, and no device line,
  since Endymion's devices follow Endymion to the C#.

### 19.6 What was tested

The blast radius, as §6.7 has it, with each component's variable unset (its default) and `0`:

| Run | Default | `0` |
|---|---|---|
| Endymion: `Audio.`, `Input.` and `Endymion.` classes (its parity and switch classes among them), `AudioProperties`, `LeafAssemblyTests`, and `GalaxiaParityTests`, whose switch class this step changed | 246 pass | 246 pass |
| Mistress's callers of Endymion, as in §14.4 | 196 pass, 1 skipped | 196 pass, 1 skipped |
| Serenity: `Slang*` and `Spirv*`, its two parity classes and its switch class, `ShaderBenchTests`, `ShaderSettingsWindowTests`, `ScreenFilterSettingTests`, with the pack named, on the RX 6800 | 131 pass | 131 pass |

Each switch class makes the switch as the process does, through the component's own `SwitchFor`, over a folder with
no library: for unset, `1` and another value the C# and one line each, written once however often it is asked, to
the sink and to the error log; for `0` neither a load nor a line; and a load that throws is a fallback with the
exception's type and words. A second case gives a load that succeeds and holds each value to its side.

**CI:** the C# steps of Endymion and Serenity set their variable to `0`, and the library steps run on the defaults,
as Galaxia's do. They still report and do not gate.

**Not done, and not claimed:** Windows and macOS, as before; and a published program run without its library, which
the build copies above stand in for.

---

## 20. `GameCheats`' naming in Rust, 2026-10-10: the debt §3.4b recorded

`GameCheats` (`EmuSen_Config_Reference.md` §3.4b) was added to Galaxia's C# after Galaxia reached state 2, with
one rule of its own and no counterpart in the crate. It now has one, built as every other rule of Galaxia was:
reproduced in `Platform/galaxia`, exported through the C layer, called by the facade while the switch has chosen
the library, the C# kept beside it as `GameCheats.Managed`, and the two held together by the parity class. Measured
on the development desktop, as the steps before it; nothing has run on Windows or macOS.

### 20.1 What is the rule, and what is composition

The rule is the name: from a console and a ROM path, the list's name (the ROM's file name without its extension)
and its category (`cheats/<console>`), each character a file name cannot hold made `_`, a console that is null,
blank or no usable folder name falling back to the flat `cheats`, and no list at all for a path that gives a blank
name, `.` or `..`. That is what moved. Two things stayed C#, and the reason is the same for both: neither is a rule
of its own. `Load`'s preference for the console's file over the flat file of an earlier build is two `ConfigFile`
calls and a conditional; `SaveIfChanged` is a `Load`, a comparison of two documents as System.Text.Json writes
them, and a `Save`. `ConfigFile` already reaches the library for every read and write, so each of these runs on the
library's bytes whenever the switch has chosen it. Porting them would put a second owner on the comparison of two
cheat lists, which today has one, the C# serializer the facade already uses to hand a list to the library (§11.2).
A Rust DianaSh that keeps a game's list composes `game_list` with `config::load` and `config::save` the same way.

| What | Where |
|---|---|
| `game_list`, `GameList` and `sanitized`, with four unit tests on both platforms' characters | `Platform/galaxia/src/cheats.rs` |
| `emusen_galaxia_cheat_game_list`: the name, then the category, each followed by a NUL; `ABSENT` for no list | `Platform/platform/src/galaxia_config.rs`, `include/emusen_platform.h` |
| The C layer: 24 exports of Galaxia's second half, 46 of Galaxia's with the library's own, 173 in the library; the interface's version is 7 | the same |
| The binding and the facade | `EmuSen.Galaxia/Native/GalaxiaNative.cs` (`CheatGameList`), `Models/GameCheats.cs` |
| The parity case | `EmuSen.WiseMan/Galaxia/GalaxiaParityTests.Config.cs`, `A_games_list_is_named_as_the_csharp_names_it` |

**The facade.** `GameCheats` now holds the category the rule gave rather than the console, so `File` is one
expression on either side and the flat case is the category `cheats`, not a second branch. `GameCheats.For` asks
the library when `GalaxiaNative.Active` and both texts cross (§11.2: text that is not valid UTF-16 has no UTF-8, and
is answered by the C#, as `CheatFile.IsValidName` does). The C# rule moved into `Managed` with one change of
necessity: it judges names with `CheatFile.Managed.IsValidName`, where it called the facade before, so that the C#
side is C# throughout. No answer can hold a NUL, which would split the list that crosses: a NUL is a character no
file name holds on either platform, so it has been mended to `_` before the answer is built.

**The version.** §3.2 gives the interface one number, matched exactly, and no minor part, so there is no such thing
as a minor bump here: any change to the export set moves the number, as each step from 1 to 6 did. An addition alone
would be harmless to an older host, which never asks for the new export, but the facade resolves every export it
declares when the library loads and refuses one that lacks any (§3.1), so the number is what names that set. The
interface is 7, in `lib.rs`, the header and `PlatformLibrary.InterfaceVersion`.

### 20.2 The parity case, and its seeded faults

The case asks both sides for 55,812 lists: 22 consoles (null, empty, white space of three kinds, `.`, `..`, `...`, a
control character, separators of both platforms, a colon, a NUL, letters outside ASCII and outside the first plane)
against 30 paths (no extension, several periods, a final period, a name that is only an extension, `..` and `....`,
a NUL, the characters Windows refuses, Windows's drive, device and UNC spellings, which on Linux are plain names);
6,000 pairs built from §10.5's seeded path parts, which on Windows add drives and shares; and every character below
U+3000 alone and inside a word, as a console and in a ROM's name and extension. Each answer is compared as its
category and its name, or as no list. The first run found no difference.

**Fourteen faults were seeded,** each built into the library, the case and the crates' tests run against it, and the
file put back:

| Fault | Parity case | Crates' tests |
|---|---|---|
| the console not mended | caught | caught |
| the ROM's name not mended | caught | caught |
| Windows's characters on every platform | caught | caught |
| mended with `-` | caught | caught |
| the extension kept | caught | caught |
| the name cut at its first period | caught | caught |
| a name of `.` or `..` accepted | caught | caught |
| the console's folder not judged after mending | caught | caught |
| the flat category for every console | caught | caught |
| the name and the category crossed in the other order | caught | caught |
| the blank console not refused before the folder is judged | missed | missed |
| an empty path not refused before its name is taken | missed | missed |
| the category joined with `/` and not `Path.Combine` | missed | caught |
| no list answered as an empty text and not `ABSENT` | missed | caught |

The four the parity case misses change nothing it can see on Linux, each for a reason that can be stated. The first
two are equivalent in both halves: a blank console mends to a blank folder and an empty path to an empty name, and
each is then refused by the name check that follows, so the early refusals, kept because the C# has them, decide
nothing. The third differs only where the separator is not `/`, which the crate's test holds in Windows's style.
The fourth is the same answer to the facade, which reads an empty list and `ABSENT` alike as no list; the C layer's
own test holds the header's word.

### 20.3 What was tested

| Run | Default (the library) | `EMUSEN_GALAXIA_NATIVE=0` (the C#) |
|---|---|---|
| `GalaxiaParityTests` with `GameCheatListTests`, `ActiveCheatsApplyAndSaveTests` and `MainWindowCheatListTests` | 77 pass | 77 pass |

`cargo test` in `Platform/` passes 105 for `emusen-galaxia` and `emusen-platform` (82 and 23), the five new cases among them,
and the header compiles clean as C99 and as C++17 under `-Wall -Wextra -pedantic -Werror`. **Not done:** Windows and
macOS, as before; the Windows rows above are argued from the crate's tests in that style, not run.

---

## 21. Text the library cannot take, at every facade, 2026-10-10

§3.4 says that a C# string holding half a surrogate pair "cannot be encoded and is refused before the call", and §11.2
says which facades answer such text with their C# rule: a suggestion, a cheat list's name, a log folder, and since §20
a game's own list. Neither says what happens at the others. A reading of the code predicted the answer: every other
facade that hands the library a caller's text throws in state 2 where its C# answers, and the defect is reachable
through a ROM's path from the library scan or a typed shell argument. The first half of the prediction is confirmed;
the second is confirmed for Windows only, by argument, and retired for Linux by measurement. Measured on the
development desktop, as the steps before it.

### 21.1 The contract, and where it was not kept

**The refusal is the binding's.** `GalaxiaNative.Utf8` encodes with a strict `UTF8Encoding`, so text with half a pair
throws `EncoderFallbackException` (an `ArgumentException`) in the C#, before any export is called; the library never
sees it and never answers `BAD_STRING` for it. *As built, then, §3.4's "refused with `EMUSEN_BAD_STRING`" is an
exception in the binding and not a status.* Nothing in the binding falls back: the choice of side is the facade's, and
a facade that chose the library on `GalaxiaNative.Active` alone passes the exception to its caller. §3.9's state 2
promises the C#'s behaviour on the library, so each such facade broke it. Endymion is not affected: its binding
replaces half a pair with U+FFFD before the call, a difference of its own recorded in §14.3 and left to Q19.

**The call sites.** Every facade in `EmuSen.Galaxia` that reaches `GalaxiaNative` was read and classified by whether
it hands over text a caller supplies:

| Class | Count | Which |
|---|---|---|
| Guarded before this section | 8 | `CheatFile.IsValidName`; `GameCheats.For`; `ErrorLog.Usable`; `ErrorLog`'s shaping of an entry and its line; `Suggestion.Distance`, `Nearest` and `Hint` |
| Unguarded, caller text, now guarded | 28 | `SaveLibrary`'s five names; `ConfigStore.For` twice and `LegacyPathFor`; `ConfigFile<T>.Load`, `Save` and `Delete`; `AtomicFile.TryRead` and `Write`; `RomHash.Md5`; `CheatFile.SaveTo` and `LoadFrom`; `ConfigRoot.ComputeFor`, `MacDataDirectoryFor`, `BundleContentsFor` and `SeedDirectoryFor`; `DataMigration.LegacyRootFor`, `SeedFromBundle` with its folders, `Run` with its roots, `CopyTree` and `RemainingLibraryDirectories` with a root; `ErrorLog.Root`, `PathFor` and its writes |
| Unguarded, caller text, left as it is | 3 | the overrides of `ConfigStore` (two) and `DataStore` (one), §21.4 |
| No caller text, or none that can hold such text | the rest | every directory property; the program's own migrations; `CheatFile.ListNames`; the three models' own `Load`, whose names are constants; the documents the C# serializes, since System.Text.Json writes half a pair as `�` (measured); the program's folder and .NET's application-data folder handed over at load, whose refusal refuses the library through the switch (§13.2) |

### 21.2 What was measured

**The defect.** A case of the parity class asks each of the 28 with four texts (a high half alone, a low half alone, a
high half at the end, a pair in the wrong order), once through the facade and once through its C#, each in a folder
of its own, and compares the answers, the files left and the words said. Before the fix, on the library's side, the
first assertion threw. A throwaway sweep of the same calls, run beside it to see past the first failure, recorded
every one: 23 threw `EncoderFallbackException` out of the facade, among them `ConfigFile<T>.Load`, whose own `catch`
asked for the file's path again to frame its message and threw from inside its handler, against its contract of null
for anything unreadable; the other five answered otherwise without throwing: `CheatFile.SaveTo`, `ConfigFile<T>.Save`
and `Delete` false where the C# wrote and deleted, `CheatFile.LoadFrom` null where the C# read the list, and an error
log entry not written where the C# wrote it. With `EMUSEN_GALAXIA_NATIVE=0` every one answered, because .NET's own
file calls take such a path and write it to disk with U+FFFD in place of the half (measured: `a\uD800b.srm` lands as
the bytes `a EF BF BD b .srm`).

**A guard that did not reach downstream.** `GameCheats.For` was guarded and named the list by the C#, with the half
pair in its name; saving that list then went through `ConfigFile<T>`, which was not, and failed. A guard at one facade
holds only while every facade its answer flows into is guarded too, which is the argument for one rule applied at
all of them rather than case by case.

**A NUL, where a list comes back.** A failed write's words return as a list each item of which ends in a NUL (§3.5).
`AtomicFile.Write` to a path holding a NUL was said as two messages on the library's side, `<folder>/a` and
`b.srm: file name contained an unexpected NUL byte. The previous file is unchanged.`, where the C# says one. A read
of such a path is quiet on both sides and agrees. No other call was found to quote caller text back in a list:
`ErrorLog.Root` with a NUL in its folder, or a settings file naming one through `\u0000`, says nothing on either
side (measured), and a migration's messages quote the files it finds, which cannot hold a NUL.

**Reachability.** Measured with .NET 10 on Linux: a file whose name holds the encoded half of a pair (`ED A0 80`) or a
byte that is no UTF-8 (`FF`) is listed by `Directory.EnumerateFiles` with U+FFFD for each bad byte; an argument and an
environment variable holding them come back the same way; and System.Text.Json refuses to read `"\ud800"` into a
string. So on Linux no text from the file system, the command line, the environment or a config file reaches a facade
with half a pair: the prediction's route through the library scan does not exist there, and a typed shell argument
arrives already decoded. On Windows a file name is a sequence of UTF-16 units the file system does not validate, and
.NET lists it unchanged, so the scan does reach `SaveLibrary`, `RomHash`, `AtomicFile` and `GameCheats` with such a
name; this is argued, not run. On any platform, a string cut between the halves of a pair reaches any facade.

### 21.3 The fix, and why at this layer

One rule, in one place: `GalaxiaNative.Takes(a, b, listed)` is true when the switch has chosen the library and every
text crosses (`Crosses`, unchanged); `listed` adds that no text holds a NUL, for a call that may quote it back in a
list. Each of the 28 facades now chooses its side with `Takes` over the texts it hands over, where it used `Active`,
and the eight guarded before use it too, so the rule has one owner. `AtomicFile` passes `listed`; no other call
needed it.

Three other layers were weighed. **The binding** cannot fall back by itself: it does not know the C# rule a facade
holds, and teaching it would make each binding method a second facade. **Replacing half a pair with U+FFFD before
the call**, as Endymion's binding does, gives the same file on Linux but a different string back (a save's path, a
config path), and Q19 decided that change belongs to the C#'s retirement and not before. **Mending in the library**
would move the decision to Rust, which cannot see what the C# was given. The facade is where each side's rule is
chosen today, so the guard is a condition on that choice, and stays plain. The rule is cheap: one pass over each
text, on calls that are not on any frame's path (P2).

### 21.4 What was found and left

- **The three overrides.** Setting `ConfigStore.OverrideDirectory`, `OverrideLegacyDirectory` or
  `DataStore.OverrideDirectory` to such text after the library has loaded throws from the setter, having already set
  the C#'s field, so the library holds the previous value; set before it loads, the same text makes the load throw,
  which the switch catches and so runs all of Galaxia on the C# with its one line. They are set only by tests and by
  MarsRT's training driver, from `Path.Combine` over a scratch folder. A fix would have the switch remember that an
  override did not cross and send every question that depends on it to the C# while it holds; that is state the
  facades do not carry today, for a case no player reaches, and it is not built.
- **A ROM's hash for a path holding a NUL** throws `IOException` from the library, which maps Rust's `InvalidInput`
  to `IO`, and `ArgumentException` from the C#. Its one caller, Mistress's identity, reaches it only after a
  `FileInfo` of the same path, which throws `ArgumentException` on both sides (measured), and no path a file system
  lists holds a NUL. This is not a crossing defect: the text crosses intact, and the two sides differ in what they do
  with a path the system refuses.
- **A migration to a destination holding a NUL** throws `ArgumentException` in the C# and, in the library, copies
  nothing and reports each file as left in place. Its roots are the computed tree's and never text a player supplies.

### 21.5 What was tested

| Run | Default (the library) | `EMUSEN_GALAXIA_NATIVE=0` (the C#) |
|---|---|---|
| The two new cases, before the fix | both fail | the NUL case passes; the other fails only on its comparison of two log lines a moment apart, mended in the case before the fix was run |
| WiseMan's Galaxia classes with `GameCheatListTests`, `DianaShSandboxTests` and `HierTests` | 230 pass | 230 pass |
| `BatterySaveTests`, `ActiveCheatsApplyAndSaveTests` and `MainWindowCheatListTests` | 51 pass | 51 pass |

The cases are `Text_with_no_utf8_is_answered_by_every_facade_as_the_csharp_answers_it` and
`A_path_holding_a_nul_is_said_as_one_message_by_whichever_side_writes_it`, in
`EmuSen.WiseMan/Galaxia/GalaxiaParityTests.Crossing.cs`. Six faults were seeded and each was caught: `Takes`
ignoring `listed` (the NUL case), `Takes` judging only its first text, `Takes` as `Active` alone (both cases), and
the guard taken back out of `ErrorLog`'s writes, `DataMigration.CopyTree` and `ConfigFile<T>.Delete`, the last three
chosen from late in the case's order so that an earlier failure could not hide them. No Rust changed and the
interface stays 7. **Not done:** Windows, where §21.2's route through the scan is argued and not run.

## 22. Interface 8, 2026-10-11: the disc crate's set module, and Galaxia's Unpacked folder

At Diamond's stage 11 the library gained three calls over `emusen-disc`'s set module, `emusen_disc_companions`,
`emusen_disc_playlist` and `emusen_disc_sbi_beside` (`Platform/platform/src/disc.rs`), each a text in and a NUL-joined
text out by the length-query idiom, so that the runtime reads a cue sheet's companions and an `.m3u` with the parser
the cores use (`EmuSen_CoreAPI_Extensions.md` §2.3). It is a new dependency of the library on a crate of
`EmuSen/Cores/Shared/`, stated there, and it brings the `chd` crate and its decoders into the library, unused by these
three calls; `THIRD_PARTY_NOTICES.md` lists `chd` already, for Diamond. Galaxia's tree gained its twenty-first directory,
`EMUSEN_GALAXIA_DIR_UNPACKED` (`EmuSen_Galaxia.md` §3.7), in both halves. The interface's number went from 7 to 8, in
`emusen_platform.h`, `lib.rs` and `PlatformLibrary.InterfaceVersion` together, as §3.2's exact rule requires. The C#
facade is the runtime's `Crystal/Media/ImageSets.cs`, which takes the exports through `PlatformLibrary.Export` as
Endymion's and Serenity's facades do; it has no C# half, there being no second parser to keep. *Moved to
`EmuSen.Galaxia/Media/ImageSets.cs` on 2026-10-12 (§23).*

## 23. Interface 9, 2026-10-12: a mapped file

With the core ABI's minor 1.3 (`EmuSen_CoreAPI.md` §6.24), Galaxia's half gained three calls, `emusen_galaxia_map`,
`emusen_galaxia_mapped_bytes` and `emusen_galaxia_unmap` (`Platform/platform/src/galaxia.rs` over
`Platform/galaxia/src/mapped.rs`): a file mapped whole and read-only behind an opaque handle, its address and length,
and its release. The galaxia crate takes `memmap2` 0.9 (MIT OR Apache-2.0, already in `THIRD_PARTY_NOTICES.md` through
another crate) for the system's call on each platform. The C# facade is `EmuSen.Galaxia/Media/MappedFile.cs`, whose
C# half is `System.IO.MemoryMappedFiles`; `GalaxiaParityTests` holds the two alike (`EmuSen_Galaxia.md` §3.8).
`ImageSets` and `ArchiveImages` moved from the runtime's media module into Galaxia, the set module still reached
through `emusen_disc_*`. The interface's number went from 8 to 9 in `emusen_platform.h`, `lib.rs` and
`PlatformLibrary.InterfaceVersion` together.

## 24. Interface 10, 2026-10-12: advice on a mapped file

With the core ABI's minor 1.4 (`EmuSen_CoreAPI.md` §6.25), Galaxia's half gained `emusen_galaxia_mapped_advise`
(`Platform/platform/src/galaxia.rs` over `Mapped::advise` in `Platform/galaxia/src/mapped.rs`): the system advised
that a range of a mapping will be read soon, `madvise` with `MADV_WILLNEED` through `memmap2` on Linux and macOS and
`PrefetchVirtualMemory` on Windows, the range clipped to the file, 1 where the advice was taken and 0 where the range
holds nothing of the file or the system refused. The C# facade is `MappedFile.Advise`, whose C# half makes the same
system calls over the view it holds; `GalaxiaParityTests.Advice_is_taken_for_the_same_ranges_both_ways_and_the_advised_bytes_reach_memory`
holds the two to the same answer for eight ranges and, on Linux, shows the advised pages arriving untouched while the
rest stay on the medium (`EmuSen_Galaxia.md` §3.9). The Windows half was compiled for `x86_64-pc-windows-gnu` and not
run. The interface's number went from 9 to 10 in `emusen_platform.h`, `lib.rs` and `PlatformLibrary.InterfaceVersion`
together.

## 25. Interface 11, 2026-10-15: the slang device's identity, and a picture shown on it

For the game's picture handed to the compositor on the device (`EmuSen_DevicePicture.md` §4, §13). Serenity's device
enables `VK_KHR_external_memory_fd` and `VK_KHR_external_semaphore_fd` where the driver offers them, and the interface
gains `emusen_serenity_device_identity` (the device's UUID and whether it exports), `emusen_serenity_shown_new` and
`emusen_serenity_shown_free` (an image exported with its two binary semaphores, laid out linearly or in the device's
tiling, its memory not dedicated) and `emusen_serenity_chain_present` (the chain run into a shown image, waiting on
the device for its `free` semaphore and signalling its `ready`, returning without waiting). A chain's frame upload is
recorded into the submission that runs its passes rather than one of its own. The C# facades are `SlangVulkan.Identity`,
`SlangShown` and `SlangChain.Present`, the library alone serving them (the C# device exports nothing, so a C# chain
keeps the readback). The interface's number went from 10 to 11 in `emusen_platform.h`, `lib.rs` and
`PlatformLibrary.InterfaceVersion` together.

Interface 11 also carries the device's lend to a core (`EmuSen_DevicePicture.md` §17), added before 11 was merged: the
device is made at Vulkan 1.2 where the loader and the card offer it, with timeline semaphores and, where the card has
the queues, one queue lent to a core and one the host takes the core's pictures on; `emusen_serenity_lend_new` and
`_free` (a lend for one core machine, filled as the core ABI's `emusen_device_lend`), `emusen_serenity_lend_take` and
`_settle` (the core's picture copied into a ring of three device buffers once its `ready` is reached, `done` then
signalled) and `emusen_serenity_chain_advance_lent` (the newest copy as a chain's frame, read on the device by its next
render or present). The C# facade is `SlangLend` and `SlangChain.AdvanceLent`.

## 26. DianaSh in the workspace, 2026-10-15: the shell core, its host API, and .NET's rules split from Galaxia

DianaSh's migration step S3 (`EmuSen_DianaSh.md` §13.7) put the shell in this workspace, where §2.1 and Q11 there had
placed it, and S4 (§13.8 there) its text tools. What that changed here:

- **Five crates.** `emusen-dianash` (`Platform/dianash/`), the shell core, an rlib with no dependency of the
  emulator's but Galaxia's; `dianash-module`, what a command module is written with, depending on nothing;
  `dianash-text`, the text tools' module; `dianash` (`Platform/dianash-program/`), the standalone program; and
  `emusen-dotnet`, below.
- **The library exports a second interface.** `emusen_platform` exports DianaSh's host API, `dianash_*`, from
  `platform/src/dianash.rs` over the shell core, with the text module linked in. Unlike this page's interface, which
  is matched exactly because its only client is built from the same commit (§3.2), the host API is stable and
  versioned as the core ABI is, `(MAJOR << 16) | MINOR`, because an outside frontend or the conformance kit may embed
  the shell (`EmuSen_DianaSh.md` §16 Q17); this paragraph is the place §3.2's "where this does not generalise" names.
  Its headers are `Platform/dianash/include/dianash.h` and `dianash_module.h`, and its baseline
  `Platform/dianash/abi/v1/baseline.txt`, held by `platform/tests/dianash_abi.rs` and the core ABI's guard job.
  Adding it moved this interface's own number not at all: no `emusen_*` export changed, and DianaSh's facade checks
  `dianash_abi_version`'s major itself.
- **.NET's rules left Galaxia's crate.** A shell's command module may not link Galaxia, which holds the process's
  tree and overrides (Q11 there), but the text tools need the rules Galaxia had reproduced: casing, the ordinal
  comparisons, white space, lengths in UTF-16 units, and `File.ReadAllText`'s decoding. They moved, with their tests,
  to `emusen-dotnet` (`Platform/dotnet/`), a crate of pure functions with no dependency; `galaxia/src/dotnet_text.rs`,
  the five ignore-case and white-space functions of `dotnet_path.rs` and `json::decode` now re-export them, so no
  user of Galaxia's crate changed. The crate also holds what the shell added: `int`, `long` and `double.TryParse`, the
  text file's lines and writes, and the words of .NET's file exceptions.
- **Galaxia's crate gained** `tree::base_directory` and `tree::shell_modules_directory` (`ConfigRoot.ShellModulesDirectory`,
  where `man` reads its pages) and made `simple_upper` public; and `PlatformLibrary` answers `GalaxiaActive`, since the
  Rust shell reads the tree through the library's Galaxia and is not used while Galaxia runs on its C#.
- **CI**: the platform job compiles `dianash.h` as C99 and C++17 beside `emusen_platform.h`, and the core ABI's guard
  job runs DianaSh's ABI test with the base branch's baseline.

`cargo test --release` in `Platform/` passed on 2026-10-15 with every crate's tests, the new ones included.

*S5, 2026-10-16* (`EmuSen_DianaSh.md` §13.11) added six crates and moved two modules: `emusen-session-service`, the
session service from a caller's side, held to Crystal's `emusen_session.h` by a C compiler; the four EmuSen command
modules, `emusen-debug`, `emusen-trace`, `emusen-cheat` and `emusen-session`, linked into the library beside
`dianash.text`; and in the `dianash` package a second program, `nano`, which `Platform.targets` builds with the library
and copies into `dianash/`, and which the platform library's CI artefact now carries. Galaxia's `json` and `suggestion`
moved to `emusen-dotnet`, which a module may link, with Galaxia re-exporting them; the library exports
`dianash_session_set_services`, the host API's minor 1.1. The workspace's tests passed on 2026-10-16, 202 of them.

## 27. Windows, 2026-10-15: the library shipped on win-x64

Release 0.9.0 went out for Windows without `emusen_platform.dll`, so Galaxia, Endymion and Serenity's slang half ran
on their C# there: the `emusen_platform win-x64` job of `rust-cores.yml` failed three of the Galaxia crate's tests
(run 38813315159), so the job built and uploaded nothing, and the publish's message of §2.6 said so. The tester
treated the library's absence on Windows as a defect to be fixed before the next Windows build. This section records
what each failure was, what a review of the workspace for Windows assumptions found beside them, and what CI shows.

### 27.1 The three failures: two in the tests, one in the code

Each was decided by asking which side broke a contract the other was right to hold, and then shown on the runner.

1. **`mapped::…a_missing_one_is_the_systems_error`: the code.** `Mapped::open` opened the path and then refused a
   directory as `IsADirectory`. Unix opens a directory for reading, so the refusal was reached; Windows refuses to
   open one at all (`ERROR_ACCESS_DENIED`, Rust's `PermissionDenied`), so it was not, and a frontend was handed
   "Access is denied" for a folder. The status a C# caller sees was right on both systems, since `fail_io` maps both
   kinds to `ACCESS` and the facade throws `UnauthorizedAccessException`, as .NET does; the words were not. A failed
   open of a path that is a directory is now the same `IsADirectory`, so the words are "is a directory" everywhere,
   and the C layer's test reads them back through `emusen_platform_last_error`. The test's assertion was kept as it
   was: it now holds on every platform, which is the contract it stated.
2. **`migration::…whatever_the_spelling`: the test, and not the migration.** The test made `x/../home` beside the
   tree. Its temporary folder came from `std::fs::canonicalize`, which on Windows answers in the extended form
   `\\?\C:\…`, under which the system resolves neither `..` nor `/`: the spelling named a folder `x` and a child
   `..`, and the migration, told to copy into it, made `x` before the system refused `..`. That is the caller's
   spelling obeyed, not the tree touched. `TempDir` now strips the prefix from a drive path, keeping the long names
   canonicalising found, so the tests' paths are spelled as a player's are.
3. **`tree::…published_under_the_old_names_is_still_found`: the test.** It joined `bare + "/" + name` onto the same
   extended path, an invalid name there (`os error 123`). The temporary folder's fix removes the cause; the join now
   uses `Path.Combine`'s rule besides.

**Whether a tree is ever migrated onto itself.** The second failure looked like a defect of the kind that would touch
a player's tree on a file system that ignores case, so the rule's whole logic was examined rather than the one
assertion. `run` compares the two roots' full paths without regard to case, as `DataMigration.Run` does, and stops
when they are one; Windows' full path (`GetFullPathNameW`, which both .NET and `std::path::absolute` call) already
unifies `/` with `\`, `..`, doubled separators and trailing dots and spaces. What the comparison of text cannot see
(the `\\?\` form, an 8.3 name, a junction or a link, a trailing separator, and on macOS a decomposed character) it
does not need to: the walk that follows copies only where the destination lacks the file, and every file of a tree
finds itself. `migration::tests::a_tree_spelled_any_way_the_system_reads_as_itself_is_left_alone` holds that, both
ways, for every spelling the runner's file system reads as the tree: a trailing separator, `x/../`, two cases, a
symbolic link, and on Windows `/`, `\\?\`, a lower-case drive, `.`, ` ` and `. .` appended, the 8.3 name and a
junction, and on macOS `e` with a combining acute for `é`; the tree's bytes and times and its parent's entries are
unchanged. `DataMigrationTests` asks the same of each half through the switch, and the C# and the library agree on
every case (§27.4). Nothing was changed in the migration.

### 27.2 What a review of the workspace found besides

Every crate was read for the assumptions Windows breaks: separators and case, a file open elsewhere, a rename over
a file, long paths, links and junctions, line ends, the environment's folders, and names the C runtime exports.

- **A held mapping refused nothing (fixed).** `EmuSen_Galaxia.md` §3.8 says that on Windows a mapped file cannot be
  written, renamed or deleted while a game runs. The C# half makes that true by holding its `FileStream`, opened with
  `FileShare.Read`, for the mapping's life. The crate closed its file once the view was made, and a view holds no
  share mode, so another program's writer was let in and its writes reached the bytes a core was reading.
  `mapped::tests::a_mapped_file_is_read_and_not_written_renamed_or_deleted_while_held` failed on win-x64 (run
  38824787186: the writer opened) before the change; `Mapped` now holds its file, opened on Windows with
  `FILE_SHARE_READ` alone, which is .NET's `FileShare.Read`, and an empty file holds nothing, as in the C#.
- **The application-data folder came from the environment (fixed).** .NET's `SpecialFolder.ApplicationData` asks the
  system for the known folder and does not read `%APPDATA%`; the crate read the variable and answered an empty string
  without it, which makes the legacy config folder the relative path `EmuSen`. The C# host passes its own folder, so
  only a Rust host without C# would meet it. The test runs itself again without the variable and failed before the
  change (the child answered `""`); the crate now asks `SHGetKnownFolderPath` for `FOLDERID_RoamingAppData` with
  `KF_FLAG_DONT_VERIFY`, as .NET does. The home folder needs no change: `std::env::home_dir` falls back from
  `%USERPROFILE%` to the system's profile folder, and the tree uses it only on macOS.
- **Found sound, and left as they are.** `atomic::write` renames over the live file with `std::fs::rename`, which on
  Windows replaces an existing file as `File.Move(…, overwrite: true)` does; a reader elsewhere that holds the live
  file without sharing deletion makes the rename fail in either half alike, and the live file is then unchanged, as
  §10.4 promises. Rust opens files sharing reading, writing and deletion by default, so the library's own reads never
  block a rename; .NET's do, and that is the C#'s behaviour, unchanged. Long paths need nothing: the standard library
  passes an absolute path past `MAX_PATH` in the extended form itself. Every path is built by `dotnet_path`, which
  takes the platform's style as a value, and no test or code joins with a literal `/` any more. The text the crates
  read from the checkout (`emusen_platform.h`, the cache's schema) is matched within a line, so the CRLF that a
  Windows checkout gives it changes nothing. The libraries the platform loads are named per system (`vulkan-1.dll`)
  or lent by the host's handle; the exports are all `emusen_*`, and the one declaration of a C runtime name, the
  Vulkan device's `close`, was already linked as MSVC's `_close` by the release (it is reached only where a driver
  exports descriptors, which no Windows driver does, since Windows exports handles).
- **Found and not changed.** A cheat list's name is checked against .NET's `GetInvalidFileNameChars` in both halves,
  which lets through Windows' reserved device names (`CON`, `NUL`, `COM1`), so a list named `CON` is written to
  `CON.json`, a device on Windows 10 and an ordinary file on Windows 11. It is the C#'s rule as much as the
  library's, the parity class holds the two together, and changing what a player may type is a decision of its own.

### 27.3 What CI could not show before, and the one change to the workflow

The WiseMan job's platform steps (Galaxia, Endymion and Serenity's slang half, each on its C# and on the library)
followed the step that compares the Rust cores with the C# cores, and a failing step stops those after it unless
they say otherwise. That step had failed on every system for days over cores' tests (run 38829214351: Nephrite's,
Diamond's and core discovery's), so none of the platform's WiseMan tests had run on Windows, or on macOS, or in CI
at all, and the library's absence from 0.9.0 was not the only thing hiding them. Each platform step now runs unless
the run was cancelled (`if: !cancelled()`), and still reports without stopping the job. Two tests that are not the
platform's failed in some runs and passed in others, and since the WiseMan job waits on every crate's job, each
failure kept it from running: the conformance kit's `the_seeded_faults_fail_the_same_cases_in_a_corpus` on macOS
and Windows (C19 failing where the test expects it to pass; run 38829214351 was run again for it) and MarsRT's
`a_snapshot_taken_while_workers_wait_at_barriers_is_answered_and_loads` on macOS (run 38823650539).

### 27.4 What the parity class found on Windows, once it ran there

The parity class had made no symbolic link on Windows, so the library's rules for links had never been held to
.NET's there. It now asks the system once whether the process may make one (a runner may; a desktop without
developer mode may not) and makes its links wherever it may, and on Windows also junctions, which need no
privilege. Run 38834563430 was the first to run it on Windows, and it found these, each fixed and then held:

- **A link is what it was made for (the library, fixed).** .NET answers `File.Exists` and `Directory.Exists` on
  Windows from the entry's own attributes (`GetFileAttributesEx`, which does not follow a link), and its listings
  read the same attributes. A dangling directory link, a directory link to a file and a dangling junction were
  directories to .NET, (file, directory) = (False, True), and files to the library, (True, False), which followed the
  link and, finding nothing, counted the link as a file, as .NET does on Unix. A file link to a directory agreed, by
  accident: Windows will not open a directory through a file link, so the library's fallback reached the same answer.
  The crate now has one rule, `tree::is_directory`: on Windows the entry's own `FILE_ATTRIBUTE_DIRECTORY`, on Unix the
  followed answer with a dangling link a file; `listed_as_directory` gives the migration's walk and the cheat lists'
  listing the same rule from the listing.
- **A pattern ignores case where the system's listing does (the library, fixed).** The error log prunes the files
  matching `emusen_*.log`, which .NET's enumeration matches without regard to case on Windows and macOS. The crate
  matched the extension so but the prefix exactly, and left `EMUSEN_upper.log`, thirty days old, where the C# removed
  it; the same difference showed on macOS. `begins_as` matches the prefix as `ends_as` matches the extension.
- **An empty path in Windows' words (the library, fixed).** .NET on Windows refuses an empty or all-space path to
  `Path.GetFullPath` as "The path is empty.", where Unix calls an empty one "The value cannot be an empty string.";
  Serenity's `full` gave Unix's words on every system, so a preset or a cache named so was refused in different
  words. `dotnet_path::is_effectively_empty` states the rule once, for both.
- **Shaderc's words (the C#, fixed).** A pass named `broken é.slang` that does not compile was reported by the C# as
  `broken Ã©.slang:9: error: …` and by the library as named. Shaderc writes its message in UTF-8, the binding's
  string read it in the system's code page, which on Linux and macOS is UTF-8 already; the C# now decodes it as
  UTF-8. The device's name, which `SlangVulkan` reads the same way, is ASCII on every device seen and was left.
- **A held mapping, and a tree spelled as itself (agreeing).** Both halves let a reader in and refuse a writer, a
  rename and a deletion of a mapped file on Windows, once the library held its file (§27.2), and both migrate
  nothing onto a tree under any of §27.1's spellings.
- **What the tests asked of Unix (the tests, fixed).** Tests failed in both halves alike, each over an assumption
  of its own: a recursive `Directory.Delete` refuses a junction on Windows, so links are removed first;
  the parity class handed .NET an all-space path and a UNC root with no share, which it refuses on Windows, and
  these are skipped as the empty path already was (no caller hands the library either), the path and root cases now
  listing every difference before failing; `"/srv/States"` is the folder `GetDirectoryName` writes `"\srv\States"`; the
  log folder of a home with no `.config` is a Linux question, since Windows and macOS do not take .NET's
  application-data folder from the home; the sandbox refuses an install's drive path on Windows rather than
  re-rooting it, which is still not honouring it; a preset's shader path is the full path the loader makes; and a
  preset that cannot be built on a machine without Vulkan says the device is missing.

**Left on macOS.** The same runs show four failures there that are not Windows': the sandbox's two fallback tests
expect a temporary folder where macOS's rule gives Application Support, .NET's caseless comparison on macOS pairs
fifty letters from U+16EA0 that the crate's tables, Rust's, do not yet, and APFS refuses a
file name holding U+FFFF that the ordering case writes. *All four were taken up in §27.6.*

### 27.5 What CI shows now, and the publish

**CI, run 38845515166** (the branch merged with `WiseMan`): the four `emusen_platform` jobs pass, win-x64 among
them, with the workspace's 159 tests there (25 Endymion's, 78 the Galaxia crate's, 25 the C layer's and 31
Serenity's; Linux runs 86 of the Galaxia crate's, its Unix-only cases among them), and its artefact is
the library the WiseMan job loads. On win-x64 every platform step passes, each half chosen as its variable says (the
`The_switch_is_what_the_variable_says` cases of Galaxia, Endymion and Serenity, which assert the library's report of
interface 11):

| WiseMan step on win-x64 | Passed | Skipped | Failed |
|---|---|---|---|
| Galaxia on its C# | 196 | 8 | 0 |
| Galaxia on the platform library | 249 | 8 | 0 |
| Endymion on its C# | 167 | 7 | 0 |
| Endymion on the platform library | 186 | 7 | 0 |
| Serenity's slang half on its C# | 73 | 0 | 0 |
| Serenity's slang half on the platform library | 112 | 0 | 0 |

The job as a whole still failed then, on the cores' comparison step (eleven tests of Nephrite, Diamond and core
discovery on Windows) and on macOS's four of §27.4, which §27.6 takes up; its last run is recorded there.

**The publish carries the library.** `RustCoresPublish.targets` already took `emusen_platform.dll` from
`EmuSenNativePrebuilt/win-x64/` (§2.6), and it has no sidecar, not being a core; 0.9.0 lacked it only because the
job that makes it had failed. A win-x64 publish of Mistress was made from 0.9.0's tree with this section's changes,
with 0.9.0's own core libraries and the library from run 38845515166. Its tree holds 584 files where 0.9.0's holds
583, the one more being `lib/EmuSen/emusen_platform.dll` (919,040 bytes, SHA-256
`87480a9631be88c041debe935befddce094ce9a77c15a163a0a228848da23a27`, byte for byte the CI artefact). Of the rest,
the manual's pages differ and the managed assemblies differ, `EmuSen.Serenity.dll` by §27.4's change and the others
by the folder they were built in, which .NET writes into an assembly; nothing was uploaded to the release.

A second publish was made from the branch as it ends, merged with `WiseMan` (§26's DianaSh among it), with every
library of run 38881098100: 589 files, its `emusen_platform.dll` 2,387,456 bytes with DianaSh's host API in it,
SHA-256 `327cd85abe2f7552548a223eeda6b81ccb1882e9aee6d6f75cc173c20592d3f5`, byte for byte that run's artefact. The
first publish is 0.9.0 with the library; the second is what a 0.9.1 cut from this branch would carry.

### 27.6 The rest of the WiseMan job, on all three systems (2026-10-16)

With the platform's steps passing on Windows, the job itself was still red everywhere, on the cores' comparison step
and, on macOS and Linux, on some platform steps. Each failure was read against 0.9.0's own run (38820622827, at the
tag, before any change of this section): every one was there already, none came from this work or from the merge
with `WiseMan` that brought Nephrite's stage 10, and those in the platform's reach, or in discovery's and the
sidecars', were fixed here.

- **A sheet's tracks shelved as games on Windows and macOS (the code, fixed).** `ImageSets.Beside` took a `FILE` name as
  written wherever that file existed, which on a file system that ignores case keeps the sheet's spelling, so the
  shelf, comparing paths as the folder holds them, hid nothing (`Diamond_Players.md` §12, amended). Each part is now
  matched exactly, else by the one entry matching it ignoring case, and an `.m3u`'s discs are found the same way.
- **Shaderc's file unnamed on macOS (the code, fixed).** .NET's `Process.Modules` lists no loaded library on macOS, so
  the compiler identity was `shaderc unfound` in both halves and a changed Shaderc would have kept the cache's
  shaders. The file is now asked of `dladdr` for one of Shaderc's functions where the module list does not name it.
- **Windows' rules for open files (the tests, fixed).** Seven discovery cases and Nephrite's shipped-core case delete
  a library they loaded, which Windows refuses as access denied; the teardown now leaves it to the temporary folder.
  The two armed-debugger cases wrote their program twice to one path while the first engine held it mapped, which
  Windows refuses a writer (§27.2); each load now writes into a folder of its own.
- **macOS's rules (the tests, fixed).** The sandbox's root without a marker is Application Support on macOS
  (`EmuSen_Galaxia.md` §3.4), which two tests did not expect; caseless equality outside the BMP is answered from the
  system's ICU, which on macOS pairs fifty letters from U+16EA0 that the runtime's own tables, and the crate's, do not
  know, so only characters the runtime's tables assign are compared; and APFS refuses a name holding U+FFFF, which the
  ordering case no longer writes there.
- **Linux's Shaderc (not fixed here).** On the ubuntu-24.04 runner Silk.NET cannot load Shaderc's library at all,
  which fails Serenity's compiler, cache and parity cases and Diamond's shader case, in 0.9.0's run as in this
  branch's. The file is there, at `runtimes/linux-x64/native/libshaderc_shared.so`, and `ldd` finds every library it
  needs; on this project's Fedora machine the same test host loads that very file (traced with `LD_DEBUG`). Run
  38869334249 had the runner load it with Python's `ctypes`, and it loads: the system can load the library, and what
  fails is Silk.NET's search for it in the test host's folder on that runner. That is where the next step starts
  (the loader's trace in CI, or the path the binding is given); it is left open here, as it is not Windows'. *Found and fixed in §33: Silk.NET's resolver reaches `linux-x64` only from a fixed list of distributions, which has no Ubuntu.*

**After the second merge with `WiseMan`, run 38881098100.** Merging MercuryRT's stage 9 and the Sega CD's player
brought two failures of their own. On Windows 128 of MercuryRT's cases failed with "MercuryCore has no member
Magic": the committed state layouts (`EmuSen.WiseMan/Cores/MercuryGoldens/layouts/`) are split at `\n`, and a Windows
checkout gives them CRLF ends, so every field's path kept a `\r`; the fixture now reads a listing with its line ends
made `\n`, as the engines print it. On all three systems `ConsoleBindingsTests.The_catalog_lists_consoles_by_manufacturer_then_release`
expects the catalogue without the Sega CD, which the merge added to it; that expectation is the Sega CD's to
update and is left to it.

**Run 38891224338, the branch's last.** With the layouts read so, the WiseMan job's steps on win-x64 and osx-arm64
pass but for that one catalogue case, in Endymion's two steps: the cores' comparison 1,292 of 1,292 on each, Galaxia
196 and 249, Serenity 73 and 112, Endymion 166 and 185 of their 167 and 186. On linux-x64 the Shaderc cases above and
the catalogue case fail, and nothing else. Every crate's job passed, the four `emusen_platform` jobs among them.

**Run 38861419247, before that merge.** The WiseMan job passed on win-x64 and on osx-arm64, every step: the cores'
comparison 1,282 of 1,282 on each, and the platform's six steps as §27.5's table on Windows. On linux-x64 it failed on
the Shaderc cases above alone (one of the cores' step, eleven and forty-eight of Serenity's two). Every crate's job
passed, the four `emusen_platform` jobs among them, with DianaSh's crates.

### 27.7 What is left to decide

Whether this build replaces 0.9.0's Windows asset or is released as 0.9.1 is the tester's decision. In favour of
replacing: the program is 0.9.0's in every other respect, and 0.9.0's notes already say the library was left out of
the Windows build. In favour of 0.9.1: an asset under a published tag would change its bytes and its checksum, and
the build also carries the C# compiler's change of §27.4. The platform's WiseMan steps still report rather than gate
(§10.6); making them a gate on Windows is a decision of its own, now that they pass there.

## 28. CI builds only what changed, 2026-10-16

Before this change, every push that matched `rust-cores.yml`'s path filters rebuilt and retested all 28 libraries
(seven crates on four platforms), although most commits touch one core or none. This section records what a run
cost, the rule that replaced the full rebuild, the inputs a library's key is made of, and the runs that show it
working.

### 28.1 What a run cost (measured)

Run 38861419247 (branch `windows-platform`, 2026-10-16) is the reference:

- **Size.** It had 33 jobs and used 173 job-minutes. It took 56 minutes from push to its last job.
- **The critical path.** The three WiseMan jobs, which wait on every library job, started 54.5 minutes in. They
  waited for MoonRT's osx-arm64 job, which had queued 24 minutes for a macOS runner and then ran for 30.6. The
  macOS jobs started anywhere from 5 to 40 minutes in, so the account's macOS runners, not the work, set the pace.
- **The platforms.** Decided 2026-10-16: the libraries are built for Linux, Windows and macOS on Apple Silicon
  only. The osx-x64 leg is gone, with its Rosetta step, so a run has 21 libraries rather than 28. Seven
  jobs fewer reach the macOS queue that set the pace above.
- **The part §28's rule removes.** MoonRT's build cost is the subject of `Moon_Native.md` §8.6. That part is the
  rebuilding of libraries whose inputs had not changed.

### 28.2 The rule

**A library is built when no earlier successful build of exactly its inputs exists. Otherwise that build's files
are taken.**

- **Main, tags and a dispatch with `full` build every library**, whatever earlier runs hold.
- **A taken library is uploaded into the run** under the name the run would have given it (`<crate>-<rid>`).
  Every run therefore carries all 21 artefacts, and the WiseMan job and a publish read them as they did before.
- **The other jobs are unchanged.** `shared`, `abi-guard`, `diamond-kernel` and the WiseMan job run as before.

The workflow had no step that compared commits. Such a comparison would be the wrong basis anyway:

- **What "changed" means is unclear.** A push of several commits, a force-push, a revert or a branch's first push
  each make the previous commit the wrong reference.
- **It cannot tell which build to take.** A content key answers both questions at once: two trees with the same
  key hold the same inputs, wherever and whenever they were built.

### 28.3 The key

`.github/scripts/library_key.py key <crate folder> <name> <target>` prints the key. It is the first 24 hex digits
of the SHA-256 of these lines:

1. **A version.** The script's own key version, raised to retire every earlier build at once.
2. **The library's name and target triple.**
3. **The toolchain, as `rustc +<spec> -V` names itself.** The spec is the one the PGO step chooses: the version a
   target's `pgo/<name>.<target>.pgo` pins, or `stable`. A new stable release therefore changes every key that
   builds on stable.
4. **The git object id of each input folder and file**, which is a hash of its content:
   - **The crate's folder.** It holds the crate's `Cargo.lock` and its PGO profiles.
   - **Every folder its path dependencies reach, transitively.** Every `Cargo.toml` inside each folder is read, so
     a workspace's members and a folder's nested crates count too.
   - **The sidecar tool.** Every library job runs `emusen-core-conform --sidecar`, which writes the `.core.json`
     the artefact carries. So that crate and its own dependencies are inputs of every library: `emusen-disc`,
     `Platform/galaxia` and `Platform/dotnet`.
   - **The declared extras of §28.4.**
   - **`rust-cores.yml` and `library_key.py` themselves.**

The plan job computes every key. Each library job computes its own key again on its own runner, after installing
the toolchain it builds with, and names its keyed artefact by that. If a stable release falls between the two, the
build is filed under the toolchain that actually built it.

**What the key does not cover:**

- **The runner image.** A new image is not an input. Its C toolchain links the Windows and macOS libraries, and
  `ubuntu-22.04`'s glibc sets the Linux library's floor. A full build on main or a tag rebuilds everything on the
  current images. A branch may take a library its image no longer matches, built from identical sources and the
  same rustc.
- **Third-party actions.** Their versions are not inputs, beyond their tags in the workflow file.

### 28.4 Inputs outside the path dependencies: declared, and checked

Reading the crates found sources that name files outside their path dependencies. `EXTRA` in `library_key.py`
declares them, by the folder that reads them:

| Read by | Input |
|---|---|
| MarsRT (`include_bytes!` in its GPU module, and a test) | `EmuSen/Cores/Nintendo/Mars - N64/Rdp/Gpu/Shaders` |
| VenusRT's tests | `EmuSen.WiseMan/Reference/analysis/dsp1_tables.py`, `st010_tables.py` |
| Diamond's tests | `EmuSen.WiseMan/Reference/analysis/psxplay` |
| Diamond's kernel harness (in Diamond's folder) | `Diamond_OpenKernel.md` |
| `Platform/serenity` (`include_str!`) | `EmuSen.Serenity/Slang/spirv-cache-schema.sql` |
| `Platform/dianash`'s tests | `EmuSen.WiseMan/DianaSh/Golden`, `EmuSen.Crystal/include/emusen_session.h` |

**The check.** The list will go stale as code changes, so the plan checks it on every run:

- It reads every tracked `.rs` file of each library's folders for string literals that begin with `../`.
- It resolves each one from the file's folder and from its crate's manifest folder.
- A literal that names an existing path outside the declared inputs makes that library build, whatever earlier
  runs hold, with a warning that names the file and the path. Such a library uploads no keyed artefact, so it
  cannot be taken until the input is declared.

**The check's limits.** It sees literal relative paths, which is how every one of the inputs above is named. It
does not see a path assembled at run time from pieces, or one read from an environment variable. A test that reads
a game or a corpus through an environment variable reads nothing on a runner, so that gap costs nothing today.

### 28.5 Where builds are kept, and why artefacts rather than the cache

**Keeping a build.** Each library job uploads its files twice: as `<crate>-<rid>`, as before, and as
`<crate>-<rid>-key-<key>`.

**Finding one.** For each library, the plan asks the API for the newest unexpired artefact of that keyed name.
It refuses an artefact whose run came from a fork, which is judged by the run's head repository. It then
downloads the artefact and opens it, and takes it only if all of these hold:

- the library is present and not empty;
- nothing else is in it but the sidecar;
- the sidecar, if present, parses as JSON;
- the archive passes its own check.

Anything else, from a missing artefact to an expired or unreadable one, makes the library build. A failure of the
lookup can only lead to a build, never to a missing library.

**Taking one.** A `reuse` job copies the artefact into the run under `<crate>-<rid>`.

**Why not `actions/cache`.** The cache was the other candidate, and two of its properties rule it out here:

- **It is scoped by branch.** A branch reads only its own entries and the default branch's. Main lags `WiseMan`,
  so a branch cut from `WiseMan` would find almost nothing.
- **A hit cannot be checked before the jobs are laid out.** The library job's matrix has to be fixed when the
  workflow starts. With the cache, every one of the 21 jobs would have to start, macOS ones included, only to
  restore and finish.

Artefacts are repository-wide and named by their key. They are kept for the repository's retention period, 90
days by default. When one expires, the next run builds the library again.

**Trust.** An artefact can be uploaded only by a run of this repository. Anyone who can push to the repository
could therefore place a library under a key. That is the same trust as pushing code that CI builds and ships. The
fork check keeps pull requests from forks out.

### 28.6 Releases

Main and tags always build every library on its own runner, and the plan takes nothing for them. A branch's run
may take earlier builds of identical inputs. The plan's job summary lists, for each library, whether it was built
or taken, the run it was taken from, and its key. Release assets come from main or a tag, so they are always built
in the run that publishes them.

### 28.7 What still runs every time, and what triggers a full rebuild

**What still runs every time:**

- the plan job;
- the `shared` crates on three systems, which build the v1 test cores the WiseMan job needs;
- `abi-guard` and `diamond-kernel`;
- the WiseMan jobs.

**What triggers a full rebuild.** A change to an input every library shares rebuilds them all:

- `emusen-native`;
- the sidecar tool and what it depends on (`emusen-disc`, `Platform/galaxia`, `Platform/dotnet`);
- `rust-cores.yml`;
- `library_key.py`;
- a new stable release.

In the fourteen days to 2026-10-16, on `WiseMan`, `emusen-native` changed in 32 commits, `emusen-core-conform` in
23 and `rust-cores.yml` in 28. So a full rebuild stays common, and §8.6 of `Moon_Native.md` is what makes it
cheaper.

**Why the sidecar tool is an input.** Leaving it out was considered and rejected. The sidecar is part of what ships.
A change to the tool that writes it can change it. And a sidecar cannot be regenerated on another system's runner,
because writing it means loading the library.

### 28.8 The runs (measured 2026-10-16, branch `ci-speed`)

Each run below was pushed only after the one before it had finished. "Library minutes" sums the library jobs and
the reuse jobs. "Libraries ready" is when the last of them finished, counted from the push.

| Run | What the push changed | Built | Taken | Library minutes | Libraries ready |
|---|---|---|---|---|---|
| 38861419247 (`windows-platform`, before §28 and `Moon_Native.md` §8.6) | — | 28 | — | 187.7 | 54.5 min |
| 38921031260 | the workflow, which changes every key | 21 | 0 | 72.8 | 22.3 min |
| 38928103614 | MoonRT's `Cargo.toml` | 3, MoonRT's | 18 | 5.8 | 5.2 min |
| 38933024285 | `RustCores.props`, an input of no library | 0 | 21 | 2.1 | 4.3 min |

- **What each run built.** The plan built exactly what the change reached. A change to MoonRT built MoonRT on
  its three platforms and nothing else. A change outside every library's inputs built nothing.
- **What each part costs.** The plan job takes 0.5–0.8 minutes, and a reuse job about 6 seconds.
- **The two keys agree.** The key a library job computed on its own runner, Linux or Windows, matched the plan's
  for every library. The first run of the workflow, 38898739867, which still built osx-x64, checked this: its
  keyed artefacts' names equal the keys computed from the same tree off the runners.
- **The WiseMan jobs ran on every one of the three runs**, on libraries built, taken, or both, and gave the same
  result each time:

  | Platform | Result |
  |---|---|
  | win-x64 | passed |
  | osx-arm64 | passed; the cores' step 908 of 908 |
  | linux-x64 | the cores' step 907 passed and 1 failed |

  The one failure is `DiamondShaderTests.Every_compiled_shader_of_the_renderer_is_the_compilation_of_its_source`.
  It fails the same way on `WiseMan`'s own run 38897939664, so it predates this change. In 38933024285 the macOS
  `shared` job first failed the conformance kit's `the_seeded_faults_fail_the_same_cases_in_a_corpus`, the
  intermittent case §27.3 records, and passed when that job was run again. The WiseMan jobs ran after it.
- **The tests are now the critical path.** With nothing to build, a run still takes about 30 minutes, almost all of
  it the WiseMan jobs' 24–32 minutes of tests. That is the next place to look.

## 29. The Mac mini as the Apple Silicon runner, 2026-10-16

The Apple Silicon jobs of `rust-cores.yml` (the shared crates, each library's `osx-arm64` job and WiseMan's) run
on the project's own Mac mini when the run is a push or a manual start on a working branch. Runs on `main`, on a
tag, or with the repository variable `EMUSEN_MAC_RUNNER` set to `off` take GitHub's `macos-latest` as before. A
release's assets are therefore never built on the project's machine.

**Why.** GitHub's macOS machines are virtual and offer no Metal device, so the device picture and the Vulkan-on-Metal
tests could not run in CI, and the tests that measure time ran beside other tenants' work. The Mac mini is an Apple M1,
the lowest Apple Silicon EmuSen targets, with MoltenVK and the Vulkan loader installed (`vulkaninfo` reports the
Apple M1 through MoltenVK at Vulkan 1.4). Its caches persist between runs.

**The runner.** GitHub's runner 2.338.0 (osx-arm64), its archive checked against the release's published SHA-256,
lives in `~/actions-runner` under the tester's account, registered as `emusen-mac-mini-1` with the label `emusen-mac`,
and runs as a launchd agent while that account is logged in. Its jobs keep their own Rust and .NET:
`CARGO_HOME`, `RUSTUP_HOME` and `DOTNET_INSTALL_DIR` in the runner's `.env` point inside `~/actions-runner/_tools`, so
a job's toolchain never replaces the one the tester uses on that machine.

**Safety on a public repository.** The workflow runs on `push` and `workflow_dispatch` only; neither can be started
from a fork, since both need write access. The `runs-on` expression also names the Mac only for those two events, so
a `pull_request` trigger added later would still run on GitHub's machines. That expression must keep the event
condition.

**When the Mac is away.** FileVault is on, so after a restart the runner starts only once the tester logs in. A
job waiting for it stays queued, and does not fail. `gh variable set EMUSEN_MAC_RUNNER --body off` sends the next runs
to GitHub's machines, and deleting the variable brings them back.

**One runner, one job at a time.** GitHub ran the Apple Silicon jobs side by side; this machine runs them one after
another. Since §28, a push builds only the libraries it changed, which keeps the queue short; a second runner on the
same machine is the next step if it is not short enough.

## 30. Interface 13, 2026-10-17: the systems table

With the backend's step B1 (`EmuSen_Backend.md` §12), the workspace gained a crate, `emusen-systems`
(`Platform/systems/`), depending on `emusen-dotnet` alone: nine embedded data files, one per system, read and checked
once. The library gained one export over it, `emusen_systems_table` (`Platform/platform/src/systems.rs`), the table as
one compact JSON array by the length-query idiom, and the header-against-exports test reads the new module. The C#
facade is the runtime's `Crystal/Systems/SystemTable.cs`, which takes the export through `PlatformLibrary.Export` as
`ImageSets` does, and which, unlike `ImageSets`, has a C# half: the packs' literals, chosen by `EMUSEN_SYSTEMS_NATIVE`
through `PlatformSwitch`, in state 2 (§3.9). `SystemTableParityTests` holds the two halves equal. The interface's number
went from 12 to 13 in `emusen_platform.h`, `lib.rs` and `PlatformLibrary.InterfaceVersion` together.

## 31. Windows and macOS paused while Linux is finished, 2026-10-17

*Decided 2026-10-17 by the tester:* the Windows and macOS builds are paused until the backend and the frontend are
finished on Linux, and ported to afterwards, on the condition that the code stays modular, portable and efficient in
the meantime. Linux, Windows and macOS on Apple Silicon remain the targets (`EmuSen_LunaP_Kit.md` §0.1); only the
work on the last two waits.

**What runs.** `rust-cores.yml` tests, builds and ships `linux-x64` alone. The shared crates' job, the plan's
platforms and WiseMan's matrix each read the repository variable `EMUSEN_ALL_PLATFORMS`; set to `on`, all three
systems run as before, including §29's Mac mini, which stays registered and idle meanwhile.

**What keeps the code portable.** A job of its own, `portable`, runs on Linux whenever the other two systems are
paused and compiles every crate (the cores, the shared, Beryl and BlackMoon crates, and the platform workspace) for
`x86_64-pc-windows-msvc` and `aarch64-apple-darwin` with `cargo check --all-targets`, so code that would not build
there fails the run the day it is written. It links nothing and runs nothing: a fault that shows only at run time
(the Metal keyword of `EmuSen_Serenity.md` §3.11 is the example) waits for the port. The C# needs no job of its own,
because the Linux build already compiles its Windows and macOS branches; only its native assets differ by RID.

**What it means for the gates.** A gate written "on the three systems" reads "on Linux" until the port resumes, and
is run on the other two then: `EmuSen_Backend.md` B7's state 2 per program and SR9's measured choice of an HTTP
client, `EmuSen_DianaSh.md` S7's hour of play on Windows and macOS, and the state-3 gates of this page. Portable
means what §2 has always meant: a platform's own calls stay behind the platform library and Galaxia, never in a core,
a pack or Crystal.

## 32. Interface 14, 2026-10-17: the save media's format modules, and the packs' code after them

With the backend's step B3 (`EmuSen_Backend.md` §13), `emusen-systems` gained the packs' code and the library its C
layer in `Platform/platform/src/systems.rs`: first the save media's (`emusen_systems_media`, `_media_read`,
`_media_copy`, `_media_delete`, `_media_format`) over three crates with no standard library in
`EmuSen/Cores/Shared/`, and one status, `REFUSED` (−1034), for an operation the rules refuse, which is an answer with
words. The C# facade is `Crystal/Systems/SystemCode.cs`, chosen with the table by `EMUSEN_SYSTEMS_NATIVE`, whose
`Load` refuses a library without every export the code takes. An empty text may come as a null pointer with no length,
as .NET pins an empty array (`given` in `lib.rs`). The number went from 13 to 14 in `emusen_platform.h`, `lib.rs` and
`PlatformLibrary.InterfaceVersion` together. Later parts of B3 add exports to the same interface number, since nothing
was released between them. The cheat codecs followed (`emusen_systems_cheat_codecs`, `_cheat_decode`, `_cheat_encode`),
with the facade `NativeCheatCodec`.

## 33. Shaderc found where .NET found it, 2026-10-17

§27.6 left Linux's Shaderc open: on the ubuntu-24.04 runner `Silk.NET.Shaderc` could not load its library, which failed
the cores' step (`DiamondShaderTests`, `MarsRtShaderTests`, the job's gate since §31) and Serenity's compiler, cache and
parity cases in the two Serenity steps, while the same build passed on this project's Fedora machine.

**The cause, read from the sources.** `Shaderc.GetApi()` hands the one name `libshaderc_shared.so` to Silk.NET's
`DefaultPathResolver` (2.23.0, decompiled), which tries in order: the bare name, given to `NativeLibrary.TryLoad`
without an assembly, which is `dlopen` and searches only the system's directories; the program's folder; and
`runtimes/<rid>/native/` for each RID in a list it builds itself. That list starts from
`Microsoft.DotNet.PlatformAbstractions`' identifier, made from `/etc/os-release` (`ID` and `VERSION_ID`, or
`DOTNET_RUNTIME_ID` when set), and extends it from the deps file's runtime graph, which a .NET 10 build's
`EmuSen.WiseMan.deps.json` leaves empty, or else by a guess that reaches `linux-x64` only for an ID in a fixed list of
fifteen: alpine, android, arch, centos, debian, exherbo, fedora, freebsd, gentoo, linux, opensuse, rhel, sles, tizen,
pop. Fedora is in it and Ubuntu is not, so on the runner the list is `ubuntu.24.04-x64` alone and
`runtimes/linux-x64/native/libshaderc_shared.so`, where a build without a RID puts the file, is never tried. Linux Mint
(`linuxmint`) and SteamOS (`steamos`) are not in it either.

**How the desktop passed.** Not through the system's Shaderc: Fedora's `libshaderc` package installs only
`libshaderc_shared.so.1`, and the bare name misses it. `LD_DEBUG=libs` on one run of the two tests shows `dlopen` trying
`/lib64/libshaderc_shared.so` and `/usr/lib64/libshaderc_shared.so` and finding neither, then loading
`bin/Release/net10.0/runtimes/linux-x64/native/libshaderc_shared.so`, the file found through `fedora.44-x64`'s guess.
With `DOTNET_RUNTIME_ID=ubuntu.24.04-x64` the same two tests fail on that machine with the runner's words, and so does a
small program under `bwrap` with `/etc/os-release` reading `ID=ubuntu` and the system's file made unreadable (measured
2026-10-17).

**The product had it too.** Serenity's compiler, `SlangCompiler.Managed`, loaded Shaderc through the same `GetApi()`.
A published build is not affected by the list: its native files sit beside the program, and the resolver's second
candidate finds them. Two paths were: any build without a RID (a `dotnet run` of Mistress, the tests) on a distribution
outside the list, which could not compile a slang preset at all; and a published build on a machine where a Shaderc is
reachable by the bare name, as a development package's unversioned link makes it, which loaded the system's Shaderc
before the one it ships, contrary to §2.8 and §16.2. With a link of that name on the loader's path, the published
probe loaded `/usr/lib64/libshaderc_shared.so.1` through `GetApi()` (measured 2026-10-17); the compiler identity
carries that file's digest, so the cache stayed correct, but the SPIR-V was another library's.

**The fix.** `EmuSen.Serenity/Slang/ShadercLibrary.cs` builds the binding over a `LibraryLoader` of its own whose load is
`NativeLibrary.TryLoad(name, assembly, null)`: the probing .NET gives a `DllImport` of Serenity's assembly, which tries
the host's `NATIVE_DLL_SEARCH_DIRECTORIES` first. The host fills those from the deps file for the RID it was built for
(`runtimes/linux-x64/native/` in a build without a RID, the program's folder in a published one), not from
`/etc/os-release`, so the file is the shipped copy on every distribution, and the system's is reached only when no copy
ships. Silk.NET's resolver still supplies the names tried, so nothing it found before is lost. `SlangCompiler.Managed`
and the tests' `ShaderCompiler` both load through it; the handle Serenity lends the library (§16.2) is unchanged, a
`DefaultNativeContext`'s. No package is installed on the runner: the dependency is the program's, not the runner's.

**What holds it.** `SlangShadercLoadTests` starts a child test run with `DOTNET_RUNTIME_ID=ubuntu.24.04-x64`, the input
Silk.NET's list is keyed on, compiles a fragment stage through Serenity and a compute shader through the tests'
compiler, and requires the loaded file to be under the program's folder. It failed before the change with the runner's
`FileNotFoundException` and passes after it. Under `bwrap` as above, with the system's file unreadable, the two cores'
cases, `SlangCompileTests` and the new class pass, and `LD_DEBUG` shows the shipped copy loaded.

## 34. Interface 15, 2026-10-17: the packs' debugger code

With the backend's step B3, part (d) (`EmuSen_Backend.md` §13.5), the library gained
`emusen_systems_disassemble`, `emusen_systems_board_name` and `emusen_systems_view`
(`Platform/platform/src/systems_debug.rs`). A view reads the engine's memory in rounds, since no callback crosses
(§3.7): the host passes the bytes it has read for the request so far, the view asks for the next batch it lacks, and
the host reads it byte by byte in that order and asks again. The answer is an envelope, the JSON's length in four
little-endian bytes, the JSON, then a picture's RGBA or a tile's pixels. The C# facades are `NativeDebugViews` and the
disassemblers' own forwarding, chosen with the table by `EMUSEN_SYSTEMS_NATIVE`. The number went from 14 to 15 in
`emusen_platform.h`, `lib.rs` and `PlatformLibrary.InterfaceVersion` together, since 14 had been merged.

