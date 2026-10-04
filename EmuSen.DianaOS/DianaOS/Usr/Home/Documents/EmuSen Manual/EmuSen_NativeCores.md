# EmuSen — one native interface for the Rust cores

*Built in part on 2026-09-30: §7's steps 1 and 2 as far as MoonRT needs them. §12 says what was built, how the old and
new ABIs coexist, and what is owed; P2 is retired there.*

*Partly superseded on 2026-10-01 by `EmuSen_CoreAPI.md`, which makes this interface the starting point of a stable,
versioned core ABI (`emusen_core_*`, major 1) with DianaOS as the runtime above it. Each decision it supersedes is
marked where it stands, and kept: §2's non-goal of a stable ABI for third parties, §3.2's version scheme and exact
matching, §3.4's create signature, §3.6's `FrameInfo`, §3.7's drain, §3.8's axis units, §3.13's last paragraph, §5's
registration records for native engines, §9 Q3 and Q4, and one item of §10. What is not marked stands, and its calls
carry over into the stable set (`EmuSen_CoreAPI.md` §6).*

*This revision: the first, 2026-09-28. A design, not an implementation: nothing described here is built. It
defines one C ABI that every Rust core exports, one generic C# host over it, one place a console and its engines are
registered, and the order in which the three Rust cores move onto it. Every claim about the code is cited to a file and
line, read on `WiseMan` at `cd6bc71e`. Claims marked **measured** were measured on 2026-09-28; claims marked
**argued** are reasoning that a later stage must prove, and §8's predictions say how. libretro is compared from its
published design only; nothing of its headers is quoted.*

Companion docs: `EmuSen_Stack.md` §2.1 (why the cores are Rust and the frontends C#), `EmuSen_Multicore.md` (`ICore`,
`CoreFactory`, `CoreCatalog` and the capability interfaces), `EmuSen_RustState.md` (the state crate, `emusen-state`,
being written on the `moon-rust` branch on the same day, whose status range §3.3 builds on), `EmuSen_Libretro.md` (the
libretro specification, whose §7 wrapper §6 here relates to), and each port's own page: `Mars_Native.md`,
`Mercury_Native.md`, `Moon_Native.md`.

---

## 0. Summary

**Decided 2026-09-28:**

1. One common native interface and a generic C# host are designed **before MoonRT reaches its stage 4**, so the NES
   engine is built on them rather than on a fourth bespoke shim.
2. MercuryRT and MarsRT **move onto it afterwards**, MarsRT last and with its PGO profile retrained.
3. **The debugger's plumbing is part of the design**, so that MoonRT's stage 5 uses it rather than making a third copy
   of the hooks.

The design, in one paragraph. Each Rust core implements one trait, `NativeCore`, from one shared crate, and one macro
generates its exports under fixed names, `emusen_native_*`, exactly as MarsRT's two library-level exports already are.
The C# side has one loader, one handle class, one abstract `ICore` base and one debugger bridge. What stays per console
is data and a handful of overrides: create-time settings, the button order and stick scaling, the space names, the
status-to-exception table, the battery files, the frame-end order, settings, the debugger's view, and the extras only
one console has. A console and its engines are registered in one record, so rewind and the other per-engine features
are declared rather than found by `is MarsCore`.

---

## 1. Where things stand (measured 2026-09-28)

### 1.1 Three ABIs with one skeleton

Exported functions, counted in the source and, for MarsRT and MercuryRT, with `nm -D --defined-only` on the libraries in
`EmuSen/obj/`: **MoonRT 22, MercuryRT 35, MarsRT 85.** An earlier count of 90 for MarsRT did not reproduce. The
in-progress state crate's page records the same three numbers against the same base.

MarsRT's 85 are three groups:

- 2 library-level (`src/lib.rs:51`, `:62`);
- 62 for the machine: `src/ffi/mod.rs` 40, `ffi/debug.rs` 16, and 2 each in `ffi/blocks.rs`, `ffi/multiple.rs` and
  `ffi/threads.rs`;
- 21 for C# Mars's native components: `ffi/rdp.rs` 6, `ffi/vi.rs` 8 and `rsp/mod.rs` 7. These are a component library,
  not a core interface. C# Mars's `Rsp.Native.cs` calls the RSP's; WiseMan's `MarsNativeRdpTests.cs` and
  `Fixtures/MarsRTViScan.cs` call the others.

No C# caller was found for four of them: `mars_machine_run_frame`, `mars_machine_vi_fields`, `mars_rsp_reciprocal` and
`mars_rsp_set_simd`.

| Concept | MoonRT | MercuryRT | MarsRT |
|---|---|---|---|
| Version | `moon_interface_version` = 1 (`lib.rs:18`, `:119`) | `mercury_interface_version` = 3 (`lib.rs:18`) | `emusen_native_interface_version` = 9 (`lib.rs:21`, `:51`) |
| Crash log | `moon_set_crash_log`, file `moonrt_crash_<pid>` | `mercury_set_crash_log`, `mercuryrt_crash_<pid>` | `emusen_native_set_crash_log`, `native_crash_<pid>` (`MarsNative.cs:37`) |
| Create | `new(image, len, *status)` (`ffi/mod.rs:40`) | `new(rom, len, model, *status)` (`:36`) | `load_rom(rom, expansion, save, pak)`, null with no status (`:367`) |
| Run | `run_frame` → status (`:115`) | `run_frame(*detail)` → status (`:100`) | `run_frame`, `advance`, `present`, all void (`:397`, `:408`, `:419`) |
| Picture | fixed `frame` (`:188`) | fixed `frame` (`:164`) | `frame_info` (width, height, row repeat, serial) and `frame_bytes` (`:541`, `:555`) |
| Sound | i64 `audio_buffered`, `drain(out, len, max)` (`:202`, `:211`) | the same (`:178`, `:187`) | u64 `audio_buffered`, `drain(out, max)` with no length (`:507`, `:516`), `audio_sample_rate`, `peek_audio` |
| Input | `set_buttons(port, mask)` (`:153`) | `set_buttons(mask)` (`:118`) | `press(port, mask, pressed)`, `set_stick` (`:485`, `:496`) |
| Options | `set_options` bit 0 skips pixels (`:164`) | the same (`:130`) | `set_options` bits 0, 1, 2 and 4 (`:471`) |
| State | load, size, save, layout | the same | the same with a snapshot flag, plus `restore_state` and `state_kind` (`:286`, `:318`, `:327`) |
| Memory | `space_size`, `read_space`, `write_space`, i32 addresses (`:230`–`:255`) | the same (`:252`–`:276`) | `memory_size`, `read_memory`, `write_memory`, u32 addresses (`:676`–`:705`) |
| Battery | a `write_space(2, …)` after create (`MoonRtCore.cs:90`) | the same (`MercuryRtCore.cs:118`) | files at create, then `save_data`, `pak_data`, `mark_saved` (`:598`–`:633`) |
| Cheats | 256-entry tables per patched address (`:269`) | the same (`:289`) | a list of (offset, value, compare) triples (`:722`) |
| Debug | none yet | `mercury_debug_*`, 10 exports | `mars_debug_*`, 9 exports and 7 test exports |

### 1.2 The status codes collide

| Code | MoonRT | MercuryRT | MarsRT at `cd6bc71e` |
|---|---|---|---|
| −5 | — | a string length `BinaryReader` refuses | an RDRAM size neither 4 nor 8 MB (`MarsMachine.cs:70`) |
| −6, −8 | — | — | pending display-processor words (`MarsMachine.cs:71`, `:73`) |
| −9 | not an iNES image | an image shorter than its header | not a Nintendo 64 image |
| −10 | an unsupported mapper | an unsupported cartridge type | a space with no number (`ffi/mod.rs:28`) |
| −11 | PRG truncated | an unknown model | a read-only space (`ffi/mod.rs:30`) |
| −20 | — | an illegal opcode (`ffi/mod.rs:93`) | — |
| −31…−33 | a C# exception reproduced, −30 less its kind (`ffi/mod.rs:18`) | — | — |

A null handle returns −1 from MoonRT and MercuryRT. MarsRT returns 0 from its unsigned getters
(`mars_machine_audio_buffered`, `_frame_bytes`, `_pc`, `_rdram_bytes`), and nothing from its void calls.

**The state crate settles part of this.** `emusen-state` owns −1 to −8 for every core, and gives each core −9 and below.
MarsRT's three state codes move from −5, −6 and −8 to −12, −13 and −14, and its interface number goes from 9 to 10
(`EmuSen_RustState.md` §3 on that branch). §3.3 builds the rest of the space around that decision and changes nothing
in it.

### 1.3 The debug hooks: one design, drifted

MercuryRT's `src/debug.rs` and MarsRT's `src/cpu/hooks.rs` share their stop reasons, entry layouts, capacities and most
of their code. After renaming the cores and the program counter's type, 183 of their normalised lines are identical,
out of 322 and 261. They have drifted:

| | MercuryRT | MarsRT |
|---|---|---|
| Program counter | `u16` | `u64` |
| Stack | targets only, `Vec<u16>` (`debug.rs:83`) | (source, target), `Vec<(u32, u32)>` (`hooks.rs:82`) |
| Registry's stack pushed down | `set_stack` (`debug.rs:144`) | no; the stack stands where the last observed frame left it |
| Returns with nothing open | logged; the registry counts them | logged and counted, `unmatched_returns` (`hooks.rs:83`) |
| Interrupt as a call | kind 2 (`debug.rs:44`) | never pushed |
| Delay slot | — | `return_after_slot` (`hooks.rs:69`) |
| Resuming a stopped frame | `budget` (`debug.rs:95`) | `frame_open`, `frame_start`, `frame_fields` (`hooks.rs:93`–`:96`) |
| Log capacity | a field, set small by the crate's tests (`debug.rs:97`) | a constant |
| Write's instruction address | set afterwards by `stamp` (`debug.rs:255`) | known when noted |
| "Anything armed" | in C# | `armed()` (`hooks.rs:150`) |
| Profile | `BTreeMap` (`debug.rs:91`), on purpose: a `HashMap` changed the plain `MemoryBus::read`'s code (`Mercury_Native.md` §8.5.5) | `HashMap` (`hooks.rs:90`) |
| Coverage | 64K addresses, 8 KB | 24-bit addresses, 2 MB, and a second processor's (`ffi/debug.rs:151`) |
| `PROFILING` flag | 32 (`ffi/debug.rs:17`) | 64, with `RSP_COVERAGE` at 32 (`ffi/debug.rs:17`–`:18`) |
| Observed run's return | `i32`, negative for an illegal opcode, with a detail word | `u32` reasons |

The **mechanisms** differ by design and stay that way. Mercury's observed step is monomorphised over `ObservedBus`
(`debug.rs:264`, `machine.rs:145`). Mars's interpreter takes a const generic, every seam is behind
`HOOKED`, and the call seams read `if HOOKED && self.hooks.calls` (`interp.rs:119`, `:242`, `:454`, `:481`). Both keep the tables behind a box
(`machine.rs:58`, `cpu/mod.rs:115`). For Mars that box was measured as the fix for a layout cost (`Mars_Native.md`
§6.5.5, row 3). What is shared is the data and the ABI, not the mechanism.

The C# bridges, `MercuryRtCore.Debug.cs` and `MarsRtCore.Debug.cs`, share 81 normalised lines of 134 and 209:
`RunObserved`, `PushTables`, `Drain` and `Grow`. They differ in four places:

- where the first instruction's breakpoint is checked: in C# for Mars (`MarsRtCore.Debug.cs:76`), in Rust for Mercury;
- Mercury's push-down of the registry's stack;
- Mercury's observed host writes (`MercuryRtCore.Debug.cs:84`);
- Mars's second coverage map.

### 1.4 Duplicated in the C# shims

The comparison below renamed the cores, dropped blank lines and braces, and compared the lines as sets.

- **The loaders.** `MoonNative.cs` and `MercuryNative.cs` differ in three lines: the comment, the version and the
  variable's name. `MarsNative.cs` differs in its export names and log name too, and also serves C# Mars's components.
- **The handles.** `MoonMachine.cs` and `MercuryMachine.cs` share 83 of 120 and 125 lines.
- **The adapters.** `MoonRtCore.cs` and `MercuryRtCore.cs` share 135 of 191 and 203 lines. That is the cheats, skip
  rendering, lending, state by path and by stream, spaces by name, mirror sync, mutes and dispose. Only 33 lines are
  common to all three, because `MarsRtCore.cs`'s 557 lines are mostly its settings, extras and picture handling.

### 1.5 Where the framework reaches around `ICore`

- **Rewind.** Mistress disables rewind with a type check, in two places: `MainWindow.Rewind.cs:21` and
  `MainWindow.axaml.cs:1331`, `is EmuSen.Cores.Nintendo.Mars.MarsCore`. The reason is recorded (`Mars_Native.md`
  §6.6.3): the C# core's snapshot can deadlock with several workers. It is a property of an engine, found by its type.
- **Venus's hotkeys.** Hotaru reaches Venus with `_core as VenusCore` (`GameWindow.axaml.cs:47`), for three
  debug hotkeys. `EmuSen_Multicore.md` §5 records these as legitimate. Nothing here changes them.
- **The factory.** `CoreFactory` takes a Venus-only parameter, `venusFrameTimings`, in `Load` and `Bundle`. It names
  the Rust engines in `Create` (`CoreFactory.cs:44`–`:45`), `EngineNotice` (`:68`–`:69`) and `Running` (`:74`), and has
  a type switch in `Bundle` (`:84`).
- **Adding an engine to an existing console** touches seven places:
  - two engine-name constants and an `EngineByConsole` row in `CoreCatalog.cs` (`:228`–`:243`);
  - `Create`, `EngineNotice`, `Running`, `Bundle` and a `using` in `CoreFactory.cs`;
  - a row in `RustCores.props`.
- **Adding a console** touches about sixteen, adding to those:
  - the descriptor and its cheat systems;
  - `Registry` (`:77`), `Cores` (`:96`) and `EsdeNames` (`:115`);
  - `ButtonsByConsole` (`:160`), `AxesByConsole` (`:174`) and `SettingsByConsole` (`:216`);
  - `CheatCodecsFor` (`CoreFactory.cs:148`).

---

## 2. Goals and non-goals

**Goals.**

1. **One C ABI** for every Rust core, with fixed names, one status space, a version and capability bits, so a new core
   is a trait implementation and a registration, not a new shim.
2. **One generic C# host**, so the loader, the handle, the frame loop, the state plumbing, the picture lending, the
   cheats' hand-off and the debugger's bridge exist once.
3. **Nothing observable changes.** Every existing save state still loads, every engine is still exact against its C#
   oracle, and every shim still throws its oracle's exception types.
4. **MarsRT loses no speed.** The machine's work for a frame stays one call, two with the present split.
5. **The debugger's hooks exist once**, as data and ABI, with each core's mechanism left where it is.
6. **Registration in one place**, and engine features declared rather than inferred from a type.

**Non-goals.**

- **Libretro itself.** §6 compares; `EmuSen_Libretro.md` remains the plan for a libretro build, and §6.3 says how it
  would sit on this interface.
- **The C# cores.** Venus, Moon, Mercury and Mars keep their own `ICore` implementations. Registration covers them.
  The ABI does not, because they have none.
- **A stable ABI for third parties.** The libraries ship beside the assemblies that load them. The version check
  guards a mismatched prebuilt library, not an ecosystem.
  *Superseded 2026-10-01: a stable ABI for third parties is now a goal, decided that day; `EmuSen_CoreAPI.md` §4.8
  says why this non-goal's reasoning missed the cost.*
- **Changing any state format.** States stay the C# formats, written by `emusen-state`.
- **Moving the hooks' mechanisms.** `ObservedBus` and `HOOKED` stay in their cores.
- **MarsRT's component exports** (`mars_rsp_*`, `mars_rdp_*`, `mars_vi_*`). They serve C# Mars and its tests, and keep
  their names.

---

## 3. The interface

### 3.1 The rules every export follows

- **Pull, never push.** The host calls; the library returns. Rust never calls managed code, the rule `Mars_Native.md`
  §3.2 set when the calls became `SuppressGCTransition`. The picture and the sound are pulled after a frame.
- **Handles.** Every machine export takes an opaque `*mut c_void` from `emusen_native_create`. Several machines coexist.
  The tests already run many at once, and MercuryRT's armed game proof ran three engines side by side.
- **Copy out.** Memory, pictures, sound, logs and states are copied into buffers the host owns. No pointer into Rust
  memory crosses the boundary. MarsRT's RDRAM is written by display-processor workers, and a read waits for their
  drain first (`ffi/mod.rs:171`–`:174`). A pointer would bypass that wait.
- **Signed returns.** A count or a flag word is ≥ 0. A negative value is a status (§3.3). An export with nothing to
  return returns `i32` 0. No export is `void` and no getter is unsigned, so a null handle always reads as −1.
- **The length-query idiom.** An export that copies takes `(out, len)` and returns the whole length. A null `out` asks
  the length. This is what MoonRT's `frame` and MarsRT's `frame_bytes` already do.
- **A panic never crosses.** Every library is built with `panic = "abort"`, and its crash log records the panic first
  (`Mars_Native.md` §2).
- **One thread per handle at a time.** The emulation thread owns a handle. Different handles may be driven from
  different threads. The library may run threads of its own behind a handle, as MarsRT does.

### 3.2 Names, version and capabilities

**Fixed names.** Every core exports the same names, `emusen_native_*`. A library holds exactly one core. Three
libraries in one process do not collide:

- .NET resolves each export against the handle it loaded (`dlsym`, `GetProcAddress`);
- on Linux `NativeLibrary.TryLoad` loads without `RTLD_GLOBAL`, so one library's symbols never satisfy another's.

The exports the macro generates are leaves: nothing inside a crate calls them. So even a loader that did use
`RTLD_GLOBAL` could not redirect a core's internal calls (argued). The cost of fixed names is §10's R5: two cores
cannot be linked statically into one image.

**Extension exports** use the crate's name as prefix, `moonrt_*`, `mercuryrt_*` and `marsrt_*` (§3.15). MarsRT's
component exports keep `mars_*`, which keeps the two roles apart by name.

**The version** is one `u32`:

```
emusen_native_interface_version() -> u32      // (COMMON << 16) | CORE
```

- `COMMON` is this interface's version, starting at 1.
- `CORE` is the core's own number, which guards its extension exports, its status table and its settings keys.

The host refuses any library whose two halves are not exactly the ones it was built with. MarsRT already exports this
name, returning 9, or 10 after the state crate. A library of today therefore reads as `COMMON` 0 and is refused with a
clear report, rather than being mistaken for the new interface.

*Superseded 2026-10-01 for the stable set by `EmuSen_CoreAPI.md` §4.1 and §4.8: a major and a minor, the major
matched and every minor accepted, features found by name and bit, under the new prefix `emusen_core_`. The `CORE` half
retires there: a core's status band is described by `status_text`, its settings by its schema, and its extensions
version themselves. This paragraph's rule still governs the pre-stable `emusen_native_*` set until that set is
deleted (§13.5 there).*

**Capabilities** are a `u64` of bits, static for the library:

```
emusen_native_capabilities() -> u64
```

| Bit | Name | Meaning | Moon | Mercury | Mars |
|---|---|---|---|---|---|
| 0 | `RESET` | `emusen_native_reset` does the console's reset button | ✓ | | |
| 1 | `PRESENT` | `present` does work; without it the picture is made during `advance` | | | ✓ |
| 2 | `SNAPSHOT` | state kind 1 exists (§3.9) | | | ✓ |
| 3 | `AXES` | `set_axis` is read | | | ✓ |
| 4 | `AUDIO_PEEK` | `audio_peek` exists | | | ✓ |
| 5 | `MUTES` | `set_mutes` exists | ✓ | ✓ | |
| 6 | `SETTINGS` | `set_settings` takes keys after create | | | ✓ |
| 7 | `PHASES` | `phases` reports the frame's parts | | | ✓ |
| 8 | `FRAME_SERIAL` | `frame_info.serial` moves only when the picture does | | | ✓ |
| 9 | `ROW_REPEAT` | `frame_info.row_repeat` can exceed 1 | | | ✓ |
| 10 | `BATTERY_DIRTY` | `battery` reports whether the data changed | | | ✓ |
| 11 | `ROM_PATCHES` | `set_rom_patches` exists | ✓ | ✓ | ✓ |
| 12 | `DEBUG` | the `emusen_native_debug_*` exports exist | stage 5 | ✓ | ✓ |
| 13 | `DEBUG_STACK` | `debug_set_stack` is honoured | stage 5 | ✓ | |

A C# interface cannot be implemented by one instance of a class and not by another. So the capabilities do not decide
which interfaces an engine implements; each engine's class declares its own, as today. The bits are checked against
those declarations when the library loads. A class that declares `ISnapshotCore` over a library without `SNAPSHOT` is
not `Available`, and the report says why. Frontends keep probing by interface (`EmuSen_Multicore.md` §5), and see
exactly the interfaces they see now.

### 3.3 One status space

| Range | Owner | Codes |
|---|---|---|
| 0 and up | — | success: a count, a length, flags or reasons |
| −1 to −8 | `emusen-state`, as decided there | −1 null handle or buffer, −2 truncated, −3 foreign state, −4 unknown version, −5 bad string, −7 buffer too small; −6 and −8 reserved |
| −9 to −255 | **the core** | each core's own refusals: bad images, boards, models, its state's own rules |
| −256 to −319 | **this interface** | below |
| −320 to −383 | **a C# exception reproduced** | −320 − kind, below |
| −384 and below | reserved | — |

Every core code in use today lies between −9 and −33, including MarsRT's −12 to −14 after the state crate. So **no
core renumbers anything in its own band.**

**The interface's codes:**

| Code | Name | Replaces |
|---|---|---|
| −256 | `NOT_SUPPORTED` | a kind or an operation the core lacks, such as a snapshot asked of MoonRT |
| −257 | `NO_SUCH_SPACE` | MarsRT's −10 |
| −258 | `READ_ONLY` | MarsRT's −11 |
| −259 | `UNKNOWN_SETTING` | — |
| −260 | `BAD_SETTING` | a value the core cannot take |
| −261 | `NO_SUCH_PORT` | — |
| −262 | `BAD_FILE` | a battery or pak image the core refuses at create |

**The fault kinds.** MoonRT's pattern generalises. A port from a C# core that throws records the first exception the C#
core would have thrown, and returns it as a status for the shim to rethrow as the same type (`Moon_Native.md` §8.1,
"A C# exception is a fault, not a panic"). `EmuSen_RustState.md` §6 lists it as a candidate for sharing. The kinds are
.NET's types, so they are shared vocabulary:

| Kind | Status | Exception |
|---|---|---|
| 1 | −321 | `IndexOutOfRangeException` |
| 2 | −322 | `DivideByZeroException` |
| 3 | −323 | `ArgumentOutOfRangeException` |
| 4 | −324 | `InvalidOperationException` |
| 5 | −325 | `OverflowException` |

**What moves:**

- MoonRT's −31 to −33 become −321 to −323 at its stage 4, before any engine row exists. Its stage-1 mutant F4 ("the
  fault statuses count up from −30") is caught by the shim's mapping, and the mapping moves with the numbers.
- MercuryRT's illegal opcode stays −20, in its own band. It carries a detail word, and its exception is built from it
  (`MercuryMachine.IllegalOpcode`).
- MarsRT's −10 and −11 become −257 and −258.

No state file changes: statuses are return values, never data.

**On the C# side,** the generic host maps −1 to −8 and −256 to −383 itself. The console supplies the table for its own
band, and the messages. Those messages are the C# oracle's, some pinned by tests: "Not a Moon save state.",
"MarsRT refused the state: …".

### 3.4 Create, free, reset

```
emusen_native_create(image: *const u8, len: usize,
                     settings: *const u8, settings_len: usize,   // UTF-8 "key=value" lines
                     files: *const NativeFile, file_count: usize,
                     status: *mut i32) -> *mut c_void             // null on refusal, the reason in status
emusen_native_free(handle) -> i32
emusen_native_reset(handle) -> i32                                // RESET

#[repr(C)] struct NativeFile { which: u32, data: *const u8, len: usize }
```

**The create contract.**

- **The image is the file's bytes.** The core's own loader parses it: byte order, header, board.
- **The settings are the create-time keys.** Each is one `key=value` line. They are the same keys and value
  vocabulary as §3.13's, restricted to those the core reads only when it builds a machine:
  - MercuryRT's `Model`, which today is the `model` argument (`ffi/mod.rs:36`);
  - MarsRT's `ExpansionPak`, today the `expansion_pak` argument (`ffi/mod.rs:370`).
  An unknown key is `UNKNOWN_SETTING`.
- **The files are what the host read from disk for this game.**
  - `which` 0 is the battery save.
  - `which` 1 and up are the console's: MarsRT's controller pak is 1.
  - Which files to read, and from which paths, is the console's C# rule (§4.5). `--nobattery` still means that no file
    is passed.
- **A refusal returns null**, with a status: the core's own band for a bad image, `BAD_SETTING` or `BAD_FILE`.
- **A failed create leaves nothing to free.**

**Why the battery moves into create for MoonRT and MercuryRT.** Today both shims create the machine and then write the
save into space 2, clipped to the RAM's length (`MoonRtCore.cs:90`, `MercuryRtCore.cs:118`). MarsRT has to take its
files at create, because its load reads them (`ffi/mod.rs:377`–`:378`). One contract serves all three, provided a
create that copies file 0 into the battery RAM, clipped to its length, before any frame is equivalent to create
followed by a `write_space(2, 0, …)` of the same clipped bytes. For both 8-bit cores, space 2's write is a plain store
into cartridge RAM, and nothing runs between the two steps (argued). The oracle is §7's state comparison straight after
load.

*Revised 2026-10-01 by `EmuSen_CoreAPI.md` §6.2: the stable `create` takes one size-prefixed parameter struct,
adding the host's version, the accepted pixel formats and an error buffer; the contract above is kept.*

**Reset.** Only MoonRT has one, which the blargg runner uses (`Moon_Native.md` §2.1). The others do not export it, and
the capability bit says so.

### 3.5 A frame: advance, present, options, counters

```
emusen_native_advance(handle, detail: *mut u64) -> i32    // the machine to the frame's end; no picture where PRESENT
emusen_native_present(handle) -> i32                      // PRESENT: the picture from memory as it now stands
emusen_native_set_options(handle, flags: u32) -> i32      // bit 0 skips rendering; bits 1-23 the core's; 24-31 reserved
emusen_native_frame_count(handle) -> i64                  // ICore.TotalFrames
emusen_native_phases(handle, out: *mut i64, len) -> i64   // PHASES: the last frame's parts, in nanoseconds
```

**`advance` and `present` are MarsRT's split, made general.** MarsRT's shim runs `advance`, then the frame's C#
work, then `present`. The C# work is:

- the cheats, held while interrupts are off;
- the frame log;
- `NoteFrame`;
- the periodic battery save.

The picture is made after the cheats, because `present` scans the memory the cheats wrote (`MarsRtCore.cs:572`–`:594`).
MoonRT and MercuryRT write their pixels during the frame, so they have nothing to present. They lack `PRESENT`, and the
host never makes that call for them. Their frame-end C# work is the same list in their oracle's order
(`MoonRtCore.cs:131`, `MercuryRtCore.cs:144`). The cheats they apply write memory, not the picture.

- **`advance`'s return.** Zero is a frame run to its end. A negative value is a status, with the core's detail word:
  MercuryRT's `(opcode << 16) | pc`, and MoonRT's fault. MarsRT's `run_frame` and `advance` return nothing today. They
  gain a return, which is always 0.
- **Options.** Bit 0 means the same in all three today: it skips rendering (`ffi/mod.rs:164`, `:130`, `:471`). MarsRT's
  bits 1, 2 and 4 are its own and stay where they are.
- **The frame count.** It is read once after `advance` and once after a state load, then cached. MarsRT's shim reads
  it through `mars_machine_counters`, copying six values three times a frame (`MarsRtCore.cs:119`, `:583`–`:585`). The
  cache makes that one small call.
- **Phases.** MarsRT reports five phases (`ffi/mod.rs:61`). MoonRT reports one, timed in C# around the call
  (`MoonRtCore.cs`, `LastFramePhases`). A core without `PHASES` gets that C# timing from the host. The phase names are
  the console's C# data, as MarsRT's are today.

**The boundary per frame, counted for MarsRT's plain frame as Mistress drives it** (measured by reading the code):

- today, about thirteen crossings:
  - `advance`, `cop0` and `present`;
  - three counter reads;
  - `frame_info` and two `frame_bytes`;
  - `frame_profile`;
  - two audio calls;
  - Mistress's `AudioSampleRate` read.
- through the generic host, about ten: the frame count is read once instead of three times.

The machine's work is in `advance` and `present` alone, as it is now. The rest are constant-time getters. None of the
calls uses `SuppressGCTransition`, so each costs a GC transition, on the order of ten nanoseconds (argued, not measured
here). That is under a microsecond a frame, against a 5.7 ms Super Mario 64 frame (`Mars_Native.md` §6.5.5): about
0.02 per cent. §8's P3 is how that is checked.

### 3.6 The picture

```
emusen_native_frame_info(handle, out: *mut FrameInfo) -> i32
emusen_native_frame_copy(handle, out: *mut u8, len: usize) -> i64

#[repr(C)] struct FrameInfo { width: i32, height: i32, row_repeat: i32, flags: u32, serial: i64, bytes: i64 }
// flags bit 0: the last present walked, as C#'s Scan() reports
```

*Revised 2026-10-01 by `EmuSen_CoreAPI.md` §6.6: the stable `frame_info` gains a size, the pixel format, the stride
and the display aspect (56 bytes), and the format is negotiated at create, RGBA8888 always accepted.*

The format is RGBA8888, the only format `ICore.GetFrameBufferRgba` knows. Without `FRAME_SERIAL`, the serial is the
frame count. The host then implements no `IFrameSerial`, and copies the picture when it is asked for, as
`MoonRtCore.GetFrameBufferRgba` does. With it, the host copies only when the serial moves, as MarsRT's `TakePicture`
does (`MarsRtCore.cs`, `TakePicture`), and the engine's class declares `IFrameSerial`. The lending of `EmuSen_Multicore.md`
§16 is the host's, once.

### 3.7 Sound, and what its rate means

```
emusen_native_audio_rate(handle) -> i32                                  // Hz of the samples a drain returns now
emusen_native_audio_buffered(handle) -> i64                              // samples, two a stereo frame
emusen_native_audio_drain(handle, out: *mut i16, len: usize, max_frames: i64) -> i64
emusen_native_audio_peek(handle, out: *mut i16, len: usize) -> i64       // AUDIO_PEEK
emusen_native_set_mutes(handle, mask: u32) -> i32                        // MUTES
```

**The rate is a report, not a request.** `ICore.AudioSampleRate` is "known before any samples exist, so a device can
open; a core may change it when its machine does" (`ICore.cs:48`–`:49`). The three rates EmuSen has are three cases
of that one contract:

- **Moon, Mercury and their ports report a fixed 44,100**, because their mixers resample to it
  (`MoonRtCore.cs:41`, `MercuryRtCore.cs:44`). MercuryRT's mixer takes its rate from C#: `SetSampleRate(44100)`
  evaluates the division and `Math.Pow` in C#, so that the coefficients are the C# core's to the bit
  (`MercuryMachine.cs:84`–`:91`, `Mercury_Native.md` §3.3). That is a console rule. It stays an extension,
  `mercuryrt_set_sample_rate`, and the core reports 44,100 through the common call.
- **Venus follows a setting**, `AudioSettings.SampleRate` (`VenusCore.cs:377`). Venus is C# and has no native engine. A
  future one would take the output rate as a setting key and report it here.
- **Mars reports the rate the game programmed into the DAC**: the video clock over the period, rounded to even, and
  44,100 until the game sets it (`ai.rs:67`, `:81`–`:83`). It changes during play.

So the host reads the rate after every `advance`. Before any machine exists it uses the console's default, 44,100 for
all four. The drain gains a buffer length it lacks on MarsRT (`ffi/mod.rs:516`). One question is not answered here, as
it is not answered today: samples buffered at the old rate but drained after a change are labelled with the new one
(§9, Q10).

*Revised 2026-10-01 by `EmuSen_CoreAPI.md` §6.7: the stable drain never crosses a rate change and reports the rate of
the samples it returns, which builds Q10's decision.*

### 3.8 Input, and the finding that decides its shape

```
emusen_native_set_buttons(handle, port: u32, mask: u32, changed: u32) -> i32   // bits in `changed` take `mask`'s values
emusen_native_set_axis(handle, port: u32, axis: u32, value: i32) -> i32       // AXES; the console's units
```

**Why `changed`.** A "whole mask" call, as MoonRT and MercuryRT have, and a "press one bit" call, as MarsRT has, are
not equivalent. The difference is observable through a state load:

- **MarsRT's controller buttons and stick are in the state**, because C# Mars serialises them
  (`controller.rs:17`–`:26`: `Buttons`, `StickX` and the rest).
- **MoonRT's pads and MercuryRT's joypad are `Skip`**, in no state (`MoonRT bus.rs:29`–`:30`, `MercuryRT bus.rs:64`).

After a load, MarsRT's pad holds the state's buttons until a press changes one bit, and today's shim sends only
changes. A host that re-sent a whole mask would overwrite the loaded buttons on the next call, and diverge from C# Mars.

With `changed`, one call covers both:

- MoonRT and MercuryRT are sent `changed = 0xFF`, their current semantics;
- MarsRT is sent the one bit that moved, which is `press` exactly (`machine.rs:457`–`:461`).

The bit order stays the console's (§4.5). The host keeps no mask of its own for a core whose pad is in its state.

**Axes are in the console's own units.** The scaling is C#'s rule:

- Mars rounds `value × StickReach`, and turns Y (`MarsRtCore.cs`, `SetAxis`);
- the C stick becomes four buttons at a threshold, which reach the core as buttons, not axes.

So the console's C# part scales, and the core receives an `i32`. *Superseded 2026-10-01 by `EmuSen_CoreAPI.md`
§6.8: a host with no per-core code cannot know a console's units, so the stable `set_axis` takes a `double`
normalised as `ICore.SetAxis` defines it, and the core scales.* `set_axis` on a port or axis the core lacks returns
`NO_SUCH_PORT`, or 0 where the C# oracle ignores it. `SetButton` ignores ports it lacks, and the host keeps that rule.

### 3.9 State, snapshot and restore

```
emusen_native_state_size(handle, kind: u32) -> i64
emusen_native_state_save(handle, kind: u32, out: *mut u8, len: usize) -> i64
emusen_native_state_load(handle, data: *const u8, len: usize) -> i32
emusen_native_state_layout(handle, kind: u32, out: *mut u8, len: usize) -> i64
```

- **Kind 0 is the state**, the C# core's format, byte for byte.
- **Kind 1 is the snapshot** (`SNAPSHOT`): a state written without waiting for work on other threads. For MarsRT this is
  the v2 state, the v1 body plus a fixed tail of pending display-processor words. It is what Mistress's rewind captures
  (`Mars_Native.md` §6.6.3, `EmuSen_Rewind_And_FastForward.md` §1.8). A core without it answers kind 1 with
  `NOT_SUPPORTED`. The host then implements no `ISnapshotCore`, so `RewindBuffer` falls back to `SaveState(Stream)`,
  exactly as now (`RewindBuffer.cs:102`).
- **Saving settles first.** MarsRT settles its clocks before a save, as a C# save does (`ffi/mod.rs:342`). That is the
  core's business inside the export.
- **Loading is C#'s whole `LoadState`**, not the field walk alone. MarsRT's `restore_state` reads the fields, derives
  what C# derives, rebuilds the machine for a state of the other RDRAM size, and presents the picture unless rendering
  is skipped (`ffi/mod.rs:286`–`:311`). MoonRT's and MercuryRT's loads are already the whole of it. MarsRT's fields-only
  `load_state`, which its state tests use, becomes the extension `marsrt_state_load_fields`, together with
  `marsrt_state_kind` (`ffi/mod.rs:273`, `:318`).
- **A failed load changes nothing**, in every core, as each already guarantees.
- **The C# refusals that come before the library** stay in the console part:
  - MoonRT's and MercuryRT's magic and version checks, with the C# messages (`MoonRtCore.cs` and
    `MercuryRtCore.cs`, `LoadState`);
  - MercuryRT's "5 to 7" rule;
  - MarsRT's Expansion Pak flag, updated when a state changes the RDRAM size (`MarsRtCore.cs:749`–`:759`).
- **The arrays the rewind reuses.** MarsRT's shim reuses its arrays so that a capture allocates no state (`Mars_Native.md`
  §6.6.3: 252 bytes a capture instead of 12,987,630). This moves into the host, and every engine gets it.

**`emusen-state`'s `state_exports!`** generates load, size, save and layout under names a core chooses
(`EmuSen_RustState.md` §2 on that branch). It has no `kind` argument. The native macro subsumes it: kind 0 goes to the
same `StateMachine` walk, and kind 1 goes to the core's snapshot or is refused.

### 3.10 Memory spaces by id

```
emusen_native_space_size(handle, space: u32) -> i64
emusen_native_space_read(handle, space: u32, address: u32, out: *mut u8, len: usize) -> i64
emusen_native_space_write(handle, space: u32, address: u32, data: *const u8, len: usize) -> i64
```

- **Ids are the core's.** They are MoonRT's eight (`MoonMachine.cs`, `SpaceNames`), MercuryRT's and MarsRT's six
  (`ffi/mod.rs:33`–`:40`).
- **The names and flags are C# data**, because they are the C# oracle's names, which the cheats and the debugger key on
  (§4.5). The flags say whether a space is read-only, whether its reads have side effects (MoonRT's and MercuryRT's
  `CPUBUS`), whether its stores are reported to the debugger, and whether cheats may write it.
- **The address is 32 bits.** MoonRT and MercuryRT pass C#'s `int` today, and MarsRT a `u32`; the bits are the same.
  The size is 64 bits, because MarsRT's processor view is 2³² bytes (`ffi/mod.rs:678`–`:680`).
- **What lies past a space's end is the core's rule:** MarsRT reads zeros and drops writes, MoonRT and MercuryRT read
  through their decode.
- **The read takes `*mut`.** A read with side effects needs it; MarsRT's reads are pure, and the signature allows both.

### 3.11 Battery data

```
emusen_native_battery(handle, which: u32, out: *mut u8, len: usize, flags: *mut u32) -> i64  // flags bit 0 changed, bit 1 tracked
emusen_native_battery_saved(handle, which: u32) -> i32
```

`which` numbers match create's files. MarsRT reports the save chip (0) and the first port's pak (1), each with a
changed flag, and is told when the host has written them (`ffi/mod.rs:598`–`:643`). MoonRT and MercuryRT do not track
changes, and their C# oracles write every 300th frame regardless (`MoonRtCore.cs:131`–`:141`). They report bit 1 clear,
and the host writes whenever the period comes round, as now. With `BATTERY_DIRTY` it writes only what changed, as
MarsRT's `SaveSram` does (`MarsRtCore.cs:684`–`:706`).

For the 8-bit cores the length is the core's cartridge RAM. The Rust cartridge is built from the same header the C#
`Cartridge` parses, so the length equals `header.PrgRam.Length` and `header.Ram.Length` by construction (argued). A
test pins it. A cartridge without a battery reports length 0, and the host writes nothing. That is the `HasBattery`
rule, applied by the core where the shim applies it today.

### 3.12 Cheats and ROM patches

```
emusen_native_set_rom_patches(handle, words: *const u32, count: usize) -> i64   // ROM_PATCHES: (address, value, compare) triples
```

**One form, the one MarsRT already takes.** `CheatRegistry.ResolveRomPatches` lists "every byte TryPatchRom would
substitute below <limit>, in the order it tries them, resolved by its own rule" (`CheatRegistry.cs:401`). MarsRT takes
that list as triples, with `u32::MAX` for no compare (`ffi/mod.rs:722`–`:737`). MoonRT and MercuryRT take something
else: they probe `TryPatchRom` for every address and a set of probe values, and send a 256-entry table for each
touched address. MoonRT probes `$4020`–`$FFFF` (`MoonRtCore.cs:262`), MercuryRT `$0000`–`$7FFF`
(`MercuryRtCore.cs:289`).

- **The tables stay in the cores.** They are the hot representation in the bus's read, so each 8-bit core keeps its
  `HashMap<u16, …>` of tables (`MercuryRT bus.rs:71`, `MoonRT bus.rs:17`). Each builds it in Rust from the triples: for
  each original value, the first entry at the address whose compare is absent or equal.
- **Equivalence** with `TryPatchRom` is argued, and the argument rests on `ResolveRomPatches`' own promise. The
  oracle is exhaustive over each core's range: for every address, every original byte and registries of mixed compares
  and repeats, the Rust lookup must equal `TryPatchRom`.
- **The flattening loop is deleted.** It is the costliest thing either 8-bit shim does when a cheat changes: up to
  0x8000 addresses times the probe set.
- **What is carried over, not introduced.** Both 8-bit engines refresh the table only at the start of a frame. The C#
  cores consult the registry on every read. So a debugger read of `CPUBUS` between adding a cheat and the next frame
  sees the old ROM on the Rust engines. The host refreshes before each run and after each load, as today, and this
  design does not change the divergence. It is recorded here so that it is not found again.

The pokes, `CheatRegistry.ApplyAll` over the space exports, stay C#, with each console's gate. MarsRT's gate is the
Status register's interrupt enable (`MarsRtCore.cs`, `ApplyCheatsAtFrameEnd`).

### 3.13 Settings as key and value

```
emusen_native_set_settings(handle, text: *const u8, len: usize) -> i32   // SETTINGS: UTF-8 "key=value" lines, applied together
```

The keys are the console's `CoreSetting` keys, and the values are text, as they already are on the C# side
(`EmuSen_Multicore.md` §13). The console's C# part validates a value as its oracle does, with the oracle's messages
(`MarsRtCore.cs`, `Set`). It then sends it in the library's vocabulary: `Antialiasing` is sent as `2`, not `2x`.
Settings the player never sees go the same way, such as `VerifyRdp`, `VerifyBlocks` and `BlockTier`.

**Applied together, because MarsRT's settings are coupled.** Its shim diffs what it wants against what it last sent,
and sends each group in one call (`MarsRtCore.cs:160`–`:177`):

- the options;
- the threads and their worker count;
- the recompiler;
- the resolution multiple, antialiasing and device, which are applied together because each constrains the others
  (`Mars_Native.md` §6.4.8).

A one-key-at-a-time call would reconfigure the multiple twice for a change of scale and antialiasing. So the host
sends every changed pair of a frame boundary in one text, and the core applies them as one change. That replaces
`mars_machine_set_threads`, `_set_recompiler` and `_set_multiple`. Afterwards the host compares the frame serial
before and after, and takes a new picture if it moved, which is today's rule.

Where the settings list lives is §9's Q4. It stays C# for now (`MarsCore.VideoSettings`, `MercuryCore.ModelSettings`),
because `CoreCatalog.SettingsFor` must answer before any library is loaded.
*Superseded 2026-10-01 by `EmuSen_CoreAPI.md` §6.13: the core exports its settings schema, read at discovery from a
sidecar without loading the library (§7.1 there), so the list can answer before any library is loaded and still
have one source per engine.*

### 3.14 The debug interface

```
emusen_native_debug_set(handle, flags: u32, depth_target: i32, depth_guard: i32) -> i32
emusen_native_debug_set_stack(handle, pairs: *const u32, count: usize) -> i32        // DEBUG_STACK: (source, target), innermost last
emusen_native_debug_set_breakpoints(handle, pairs: *const i32, count: usize) -> i32
emusen_native_debug_set_ranges(handle, kind: u32, triples: *const u32, count: usize) -> i32  // 0 watches, 1 data breakpoints
emusen_native_debug_run_frame(handle, flags: u32, pc: *mut u64, detail: *mut u64) -> i32   // ≥0 stop reasons, <0 status
emusen_native_debug_writes(handle, out: *mut u32, len: usize) -> i64                 // (space, address, value, pc)
emusen_native_debug_calls(handle, out: *mut u32, len: usize) -> i64                  // (kind, source, target)
emusen_native_debug_profile(handle, out: *mut i64, len: usize) -> i64                // (owner, instructions)
emusen_native_debug_coverage(handle, processor: u32, out: *mut u8, len: usize, recorded: *mut i64) -> i64
emusen_native_debug_counters(handle, out: *mut i64, len: usize) -> i64               // depth, unmatched returns, then the core's
emusen_native_debug_pc(handle, processor: u32, pc: *mut u64) -> i32
```

**What is already common, and stays:**

- **Stop reasons**: `FRAME` 0, `BREAKPOINT` 1, `EACH` 2, `DEPTH` 4, `DATA` 8, `INTERRUPT` 16, `RING` 32. They are
  identical in both cores today (`debug.rs:10`–`:24`, `hooks.rs:7`–`:21`).
- **The run flags**: `UNCHECKED` 1, `CONTINUE` 2, identical in both.
- **The entry layouts**, of four, three and two values.
- **The drain idiom**: copied whole and cleared only when the buffer holds them all.

**What is reconciled.**

- **The flags.** `CALLS` 1, `WRITES` 2, `INTERRUPTS` 4, `EACH` 8 and `PROFILING` 16. Coverage is armed per processor,
  bit 8 + n for processor n. Mercury's `COVERAGE` 16 and `PROFILING` 32 move, and so do Mars's 16, 32 and 64. Both
  bridges are replaced at the same step, so no flag is ever read by a bridge of the other numbering.
- **Processors.** 0 is the main CPU. MarsRT's signal processor is 1, which replaces `which`
  (`ffi/debug.rs:151`).
- **Call kinds.** 0 is a return, 1 a call and 2 an interrupt dispatched, as in MercuryRT. MarsRT never emits 2, which
  is its C# oracle's behaviour.
- **The program counter** is `u64` everywhere. MercuryRT's `u16` widens at the export, and its `mercury_machine_pc`
  becomes `debug_pc`.
- **Counters.** Index 0 is the depth and index 1 the unmatched returns. MarsRT's exceptions entered follow at index 2.
  MercuryRT's `mercury_debug_depth` is index 0.

**The shared data**, in the shared crate's `debug` module:

- the `Hooks` struct, holding:
  - the flags, the depth target and guard;
  - the breakpoint pairs and the two range lists;
  - the stack of `(u32, u32)` pairs;
  - the unmatched count;
  - the two logs, with a capacity field;
  - the coverage bitmaps, a coverage address width per processor: 16 bits for MercuryRT, 24 and 12 for MarsRT;
  - the profile, as a **`BTreeMap`**;
- its methods: `configure`, `set_stack`, `note_call(source, target, kind)`, `note_return`, `flush`, `record`,
  `stop_before`, `covers`, `note_write`, `stamp` and `armed`;
- the drain helpers.

**What stays in each core** is the mechanism: MercuryRT's `ObservedBus`, MarsRT's `HOOKED` interpreter, MarsRT's
`return_after_slot`, and each core's frame-continuation state (MercuryRT's budget, MarsRT's frame clock and fields).
Each core still boxes the hooks. Every seam still tests its flag before it calls, as MarsRT's interpreter does, so a
disarmed seam costs the same test it costs now.

**Why the profile is a `BTreeMap` in MarsRT too.** `Mercury_Native.md` §8.5.5 measured a second `HashMap`
instantiation, over the hasher the Game Genie table uses, changing what LLVM inlined into the plain `MemoryBus::read`.
MoonRT's bus has the same `HashMap` (`MoonRT bus.rs:17`). MarsRT has no other `HashMap` in its sources, so for MarsRT
the change removes the last instantiation of std's hasher (measured by `grep`; the effect on its code is argued nil, and
§8's P4 checks it). The drain order changes from arbitrary to sorted. The registry only adds each owner's count
(`CallStack.NoteInstructions`), so the order is not observable (argued, and pinned by comparing the registry's totals).

**`set_stack` for MarsRT is an opportunity, not part of the move.** `Mars_Native.md` §6.5.6 lists "the stack in a
plain frame" as not done. Honouring `DEBUG_STACK` would be one way to do it, and it would be a behaviour change with
its own oracle. It is left out of the migration.

### 3.15 Extension exports

A console-specific feature is a named export with the crate's prefix, looked up by name by that console's C# part,
and absent from the common set. Some exports become common or become settings:

- MarsRT's `set_threads`, `set_recompiler` and `set_multiple` become settings;
- its `frame_profile` becomes `phases`, and its `peek_audio` becomes `audio_peek`;
- its `rdram_bytes` becomes the extension `marsrt_rdram_bytes`.

What stays an extension:

| Crate | Extensions |
|---|---|
| `moonrt` | `step` |
| `mercuryrt` | `step`, `serial`, `cgb_hardware`, `set_sample_rate` |
| `marsrt` | `boot`, `run_steps`, `counters`, `is_viewer_text`, `save_cpu`, `cop0`, `cpu_registers`, `rsp_registers`, `vi_registers`, `gpu_report`, `threads_counters`, `blocks_counters`, `state_load_fields`, `state_kind`, `rdram_bytes`; the test exports `physical`, `bus_read32`, `bus_write32`, `set_cop0`, `mi_raise`, `rsp_step` |

Extensions are ordinary typed exports. §10 rejects a single multiplexed `extension(id, in, out)` call.

### 3.16 The whole common set

| Group | Exports | Required |
|---|---|---|
| Library | `interface_version`, `capabilities`, `set_crash_log` | all |
| Lifecycle | `create`, `free`, `reset` | `reset` by capability |
| Frame | `advance`, `present`, `set_options`, `frame_count`, `phases` | `present`, `phases` by capability |
| Picture | `frame_info`, `frame_copy` | all |
| Sound | `audio_rate`, `audio_buffered`, `audio_drain`, `audio_peek`, `set_mutes` | `audio_peek`, `set_mutes` by capability |
| Input | `set_buttons`, `set_axis` | `set_axis` by capability |
| State | `state_size`, `state_save`, `state_load`, `state_layout` | all |
| Memory | `space_size`, `space_read`, `space_write` | all |
| Battery | `battery`, `battery_saved` | all |
| Cheats | `set_rom_patches` | by capability |
| Settings | `set_settings` | by capability |
| Debug | eleven, §3.14 | by capability |

That is 42 names. Twenty-three are required of every core. A library exports only the optional names its capabilities
claim, so that `nm` and the bits agree, and a test checks it.

**What each library would export** (argued):

- MoonRT: 26 at stage 4 and 37 with its debug layer, plus `moonrt_step`.
- MercuryRT: 36 and 4 extensions, 40 against 35 today.
- MarsRT: 39 common, 21 extensions and 21 components, 81 in all, against 85 today. The four exports with no caller
  (§1.1) are not carried over.

### 3.17 The Rust side: one trait, one macro, one crate

Each core implements one trait, and one line makes its exports:

```rust
pub trait NativeCore: Sized {
    const CORE_VERSION: u16;
    const CAPABILITIES: u64;
    fn create(image: &[u8], settings: &Settings, files: &[File<'_>]) -> Result<Self, i32>;
    fn advance(&mut self, detail: &mut u64) -> Result<(), i32>;
    fn present(&mut self) -> Result<(), i32> { Ok(()) }
    fn set_options(&mut self, flags: u32);
    fn frame_count(&self) -> i64;
    fn frame_info(&self) -> FrameInfo;
    fn frame(&self) -> &[u8];
    fn set_buttons(&mut self, port: u32, mask: u32, changed: u32) -> Result<(), i32>;
    fn audio_rate(&self) -> i32;
    fn audio_buffered(&self) -> usize;
    fn drain_audio(&mut self, out: &mut [i16], max_frames: usize) -> usize;
    // state through emusen-state's StateMachine for kind 0; spaces, battery, patches, and the optional rest
    // as methods with a NOT_SUPPORTED default, and `fn debug(&mut self) -> Option<&mut Hooks>`.
}
emusen_native::exports!(Machine);
```

- **The macro** writes each export as a thin wrapper. The wrapper checks the handle, converts pointers to slices
  (`emusen_state::ffi::input`, `copy_text`), calls the method and maps its `Result` to a status. It also takes
  MoonRT's thread-local fault before it returns, so that no fault outlives the call that made it.
- **Nothing hot moves.** `advance` calls what `mars_machine_advance` calls now (`ffi/mod.rs:80`–`:85`).
  The trait is static dispatch, and every crate is built with fat LTO and one codegen unit (the three `Cargo.toml`
  files' `[profile.release]`). So code in a path dependency is optimised with the core as one unit (argued from the
  build settings; §8's P4 checks the symbol sizes).
- **One crate, not two.** The assembly-pruning rule (`EmuSen_Multicore.md` §9.2) is that a boundary must buy
  separability that someone uses. `emusen-state` and this interface are used by exactly the same three cores, and
  neither is used without the other. `emusen-state::ffi` already holds the status range and the helpers every export
  uses. So the interface goes into the same crate, as modules beside the codec: `abi` for the trait and the macro,
  `debug` for the hooks and `status`. Whether the crate is then renamed `emusen-native` is §9's Q1.

---

## 4. The generic C# host

All of it lives in the `EmuSen` assembly, in `EmuSen/Cores/Native/`. No new project is made: a new assembly would buy
no separability that anything uses (`EmuSen_Multicore.md` §9.2).

*Built in part 2026-09-28, over today's per-core exports: `NativeCoreLibrary`, `NativeMachine` (with a `NativeExports`
table under the present names), `NativeRtCore<TMachine>` for the two 8-bit shims, and `RomPatchTable`, with the names
and shapes below; MarsRT's shim does not use the base yet. What was built, what was not and why is in
`EmuSen_Settings_Reference.md` §4.85.7.*

### 4.1 `NativeCoreLibrary`, the loader

One instance per library, replacing `MoonNative`, `MercuryNative` and `MarsNative`'s loading half. It is built from:

- the crate's name (`marsrt`), from which the file name follows: `libmarsrt.so`, `libmarsrt.dylib` or `marsrt.dll`;
- the switch variable, `EMUSEN_MARS_NATIVE=0` and its peers, kept as they are;
- the core version it was written against.

It then:

- loads the library;
- checks both halves of the version (§3.2);
- reads the capabilities and checks them against the engine class's declared interfaces;
- installs the crash log at `DataStore.Logs/<crate>_crash_<pid>.txt`;
- resolves every export of the common set once;
- keeps `Available`, `Report` and `Export(name)` for extensions.

MarsRT's crash log is renamed from `native_crash_<pid>` to `marsrt_crash_<pid>`. A generic name would be shared by
three libraries in one process, each of which keeps its own panic hook. `Mars_Native.md` §2 is updated with it.
`MarsNative` stays as the component loader, a few lines over the same library object.

### 4.2 `NativeMachine`, the handle

Holds the `nint`, a finaliser and `Dispose`, as `MoonMachine` and `MercuryMachine` do. It also holds the typed
`delegate* unmanaged` table, resolved once per library, and one method for each common export:

- the length-query idiom, done once;
- the status check, mapping shared codes and asking the console's table for its own;
- the reused arrays of §3.9.

It replaces `MoonMachine.cs`, `MercuryMachine.cs` and `MarsMachine.cs`, except for their status tables, which move to
the console part.

### 4.3 `NativeRtCore`, the `ICore` base

An abstract class that implements `ICore`, `IStateFormat`, `ICheatRegistryHost`, `IFrameBufferPool` and `IDisposable`.
It also provides public members for the optional interfaces. A subclass then opts in by naming the interface in its
own declaration, since an inherited public method satisfies an interface a subclass declares:
`SaveSnapshot`, `FrameSerial`, `RowRepeat`, `RepeatRows`, `LastFramePhases`, `Settings`, `Get`, `Set`.

It holds the ~120 lines the 8-bit shims duplicate, and MarsRT's generalisations of them:

- **`LoadRom`**:
  - read the image;
  - ask the console for its battery files and create-time settings;
  - create;
  - replace the old handle;
  - reset the registries' per-game state;
  - send options, settings and patches.
- **`RunFrame`**:
  - refresh the patches;
  - take the observed loop when anything is armed, else `advance`;
  - `EndFrame()`, a virtual method whose default is Mercury's order (frame log, cheats, `NoteFrame`, the periodic
    battery save) and which Moon and Mars override with theirs;
  - `present` where the core has it and rendering is not skipped;
  - take the picture;
  - read the phases.
- **The picture**: lending, copying on demand or on a moved serial, and the blank picture before any frame.
- **Sound**, and its rate (§3.7).
- **State** by path, by stream and by span, with the reused arrays and the console's pre-checks.
- **Spaces by name**, through the console's table, and the frame log's reader.
- **The battery**: the paths chosen at load, the period, the changed flags and `battery_saved`.
- **Cheats**: `ResolveRomPatches` with the console's limit, sent when the registry or its version moves; `ApplyAll` with
  the console's gate.
- **Mutes and mirror sync** for the consoles whose debugger reads a mirror.
- **Dispose.**

### 4.4 `NativeDebugBridge`

The `RunObserved`, `PushTables`, `Drain` and `Grow` that MercuryRT and MarsRT both have. It is parameterised by four
things only:

1. **Where the first instruction is checked.** MarsRT checks it in C# with `ShouldBreak(Pc)` before the first call
   (`MarsRtCore.Debug.cs:76`) and passes `UNCHECKED`. MercuryRT passes `UNCHECKED` only when resuming, and lets Rust
   check the table. Both are exact against their oracles today. Unifying them is §9's Q7, and until it is decided the
   bridge takes the console's choice.
2. **The processors**, and the `CoverageRegistry` each one feeds: MarsRT's `RspCoverage` is processor 1.
3. **Whether host writes to a space are observed**: MercuryRT's `CPUBUS`, as C# Mercury's bus reports them
   (`MercuryRtCore.Debug.cs:84`–`:89`).
4. **Whether the registry's stack is pushed down**: `DEBUG_STACK`.

The space numbering it needs is the console's table, shared with §4.3.

### 4.5 What each console still supplies

| | MoonRT | MercuryRT | MarsRT |
|---|---|---|---|
| Library | `moonrt`, `EMUSEN_MOON_NATIVE` | `mercuryrt`, `EMUSEN_MERCURY_NATIVE` | `marsrt`, `EMUSEN_MARS_NATIVE` |
| Create settings | none | `Model` | `ExpansionPak` |
| Battery files | `.srm` beside the ROM when `HasBattery` | `.srm` when the header has RAM | `SaveLibrary.SramPathFor`, and the pak beside it |
| Buttons | A, B, Select, Start, Up, Down, Left, Right | Right, Left, Up, Down, A, B, Select, Start (`MercuryRtCore.cs:18`) | joybus bits (`MarsRtCore.cs:62`–`:65`), sent as changes |
| Axes | — | — | the stick's scale and turn, the C buttons' threshold |
| Spaces | eight names | six names; stores reported in five | six, and the debugger's name mapping |
| Status table | its band, with the C# exception types (`MoonMachine.cs:54`) | its band; −20 with the opcode | its band |
| State pre-checks | magic, version 3 | magic, versions 5 to 7 | the RDRAM size's effect on the Pak |
| Frame end | frame log, cheats, battery | frame log, cheats, `NoteFrame`, battery | cheats while interrupts are on, frame log, `NoteFrame`, battery |
| Patch limit | CPU addresses | ROM addresses below `$8000` | the ROM's length |
| Settings | none | `Model`, by reloading before the first frame | the video settings, the ignored ones kept, the recompiler |
| Debugger view | a mirror `MoonCore` loaded from the state | a mirror `MercuryCore` | live reads (8 MB of RDRAM) |
| Extras | `Reset` | serial log, CGB flag, sample rate | IS-Viewer, counters, registers, boot, steps, GPU report |

---

## 5. Registration in one place

### 5.1 The record

*Superseded 2026-10-01 for native engines by `EmuSen_CoreAPI.md` §6.3 and §7: a v1 core describes itself in its info
and is found by discovery, with no hand-written record. A hand-written registration survives only for the C# cores, as
managed engines, until each retires (§9.5 there). The records below were never built (§12.2).*

```csharp
public sealed record EngineRegistration(
    string Name,                                   // "MarsRT (Rust)", the Engine row's choice
    Type CoreType,                                 // what Running() and EngineNotice() match
    Func<bool> Available, Func<string> Report,
    Func<ICore> Create,
    Func<ICore, CheatRegistry?, IDebugTarget> DebugTarget,
    EngineFeatures Features);                      // RewindCapture, and later others

public sealed record ConsoleRegistration(
    CoreDescriptor Descriptor, IReadOnlyList<string> Aliases,
    IReadOnlyList<PadButton> Buttons, IReadOnlyList<PadAxis> Axes, IReadOnlyList<CoreSetting> Settings,
    Func<(ICheatCodeCodec? AutoDetect, ICheatCodeCodec? Explicit)> CheatCodecs,
    IReadOnlyList<LibraryShelf> Shelves,
    IReadOnlyList<EngineRegistration> Engines,     // the first is the reference
    string DefaultEngine);
```

Each console has one registration file beside its core, and `CoreCatalog` keeps a single list of the four. Everything
`CoreCatalog` answers today is read off that list:

- `Registry`, `Cores` and the shelves;
- `ButtonsFor`, `AxesFor` and `SettingsFor`;
- `EngineFor`, and the engine-name constants, which become the registrations' names.

Everything `CoreFactory` does is read off it too:

- `Create` picks the engine, the default or the one stored, and falls back to the reference when the chosen engine is
  not `Available`;
- `Bundle` asks the engine for its debug target and the console for its codecs;
- `EngineNotice` and `Running` match `CoreType`;
- `CheatCodecsFor` asks the console.

Venus's frame-timing parameter becomes a capture inside Venus's `DebugTarget` factory, so that nothing Venus-only
stays in the factory's signature.

**What adding an engine costs afterwards:** one `EngineRegistration` in the console's file, and a `RustCores.props`
row. **Adding a console** costs one registration file, one line in the list and a row. The row stays: MSBuild needs it
to build the crate (`RustCores.props:9`–`:26`).

### 5.2 Features instead of type checks

`EngineFeatures.RewindCapture` is false for Mars (C#) and true for every other engine. Mistress's two sites read the
running engine's feature, and the reason they give ("not kept for Mars (C#)") comes from the registration's own text.
*Built 2026-09-28, ahead of the records: `EngineFeatures` and the `IEngineFeatures` an engine implements to declare it
live in `CoreCapabilities.cs`, `MarsCore` declares the withheld rewind and its text, and Mistress's two sites read it
(`EmuSen_Settings_Reference.md` §4.85.4). The registration reads the same declaration when it is built.*
Anything later that is a property of an engine rather than a capability of a core goes in the same place. A property
of an engine is a policy decision about an implementation that does support the interface.

### 5.3 A position of `EmuSen_Multicore.md` retired

`EmuSen_Multicore.md` §2 held that "`Bundle` is the only `switch` on a concrete core type in the project, and that is
deliberate … a type switch in one file is a better place for that than a registration mechanism nobody else needs".
That was true of four consoles with one engine each. It stopped being true when engines doubled the cases:

- `Bundle` has six cases;
- `Running` and `EngineNotice` repeat the engine names;
- `Create` chooses engines inline;
- the catalogue holds eight per-console dictionaries.

What the registration buys is that a console is described once. That argument does not extend to the debug targets'
construction, which stays per console inside each registration's factory. §2 and §7 of that page are revised when
this is built, not before.

Pharaoh's `PeerProbe` (`is MoonCore`) and Hotaru's Venus hotkeys are one-console debug affordances. They stay, as
`EmuSen_Multicore.md` §5 already argues.

---

## 6. Against libretro

From libretro's published design, not its headers. `EmuSen_Libretro.md` §1 has the cited version.

### 6.1 What is the same

- **A C ABI of fixed, prefix-free names**, one core per library, and a version export checked before anything else.
- **A frame at a time**, with the frontend owning presentation, pacing and audio output.
- **States as size, save and load**, in which the core owns the bytes. libretro also distinguishes the purpose of a
  save, for run-ahead and netplay, much as the kinds of §3.9 do.
- **Memory named by id.** libretro's save RAM, system RAM and video RAM are ids, as the spaces here are.
- **Options as text keys and values**, set between frames.
- **A panic or crash never unwinds into the frontend**, which is libretro's rule and the reason for the crash log here.

### 6.2 What is deliberately different, and why

| libretro | Here | Why |
|---|---|---|
| One global instance a library | Handles | The tests run many machines at once, the engines run side by side, and run-ahead in RetroArch has to load a second copy of the library file to get a second instance |
| Callbacks: the core pushes video, audio and input polls during `retro_run` | The host pulls after `advance` | Rust never calls managed code (`Mars_Native.md` §3.2); a callback per sample or poll would be a reverse transition each time; and MarsRT's C# work between `advance` and `present` needs the split |
| Raw pointers to memory | Copies | MarsRT's RDRAM must wait for its drain before a read (§3.1); a pointer's lifetime would outlive a `free` |
| Cheats as code strings the core parses | Resolved patch lists and C#'s pokes | The codecs and `CheatRegistry` are shared C#; each core would otherwise carry every codec |
| No debugger interface | §3.14 | EmuSen's debugger is a research tool, and the C# oracles' registries are what it reports to |
| An environment callback for everything else | Named exports and capability bits | Typed, checked at load, and visible to `nm` |
| Fixed output rate in the AV info, changed through an environment call | A rate read after each frame | `ICore`'s contract already allows a change, and MarsRT's rate is the game's |
| The core polls its input | The host sets the pad before the frame | The pad's state belongs to the console; for Mars it is in the state (§3.8) |
| Battery as a stable pointer | A copy with a changed flag | MarsRT's save chip and pak are tracked, and the host decides when to write |

### 6.3 How a libretro build would sit on this

`EmuSen_Libretro.md` §7 proposes `emusen-libretro`, a crate with a `RetroCore` trait and a macro that generates the
`retro_*` exports. Its trait and §3.17's `NativeCore` describe the same machine to two hosts. The recommendation is one
trait, not two: `emusen-libretro` becomes an adapter from `NativeCore` to the `retro_*` exports. It would hold the
global slot, poll input and push the frame's picture and samples through the callbacks after `advance` and `present`.
It would add nothing to the cores. That turns §7's "what stays per core" into what this design already asks each core
to supply. It does not decide any of that document's §9 questions.

---

## 7. The migration, in order

Each step names its oracle before it begins, as `Mars_Native.md` and `Moon_Native.md` do. The effort figures are in
the unit this project's record supports: comparable stages of the three ports took a day or less by their dates. Hold
them loosely.

| Step | What | Oracle that nothing changed | Effort |
|---|---|---|---|
| **0** | Decisions: §9's questions answered, and `emusen-state` merged, since the status range and the crate are its | — | an hour |
| **1** | The shared crate's `abi`, `status` and `debug` modules; the macro; the C# host's four classes and the registration records, with the four consoles' registrations over today's engines | A registration-equivalence test: every answer `CoreCatalog` and `CoreFactory` give today (engines, buttons, axes, settings, codecs, shelves, notices), for every console and extension, before and after. `nm -D` of every library is unchanged, since no core uses the crate yet | 1 day |
| **2** | **MoonRT stage 4 on it.** The exports are generated; `moon_*` retires, since nothing outside WiseMan calls them. `MoonRtCore` is rewritten over `NativeRtCore`, the battery goes into create, the patches become triples, and the fault codes move to −321…−323. The Engine row goes in through the registration, with Moon (C#) the default | Stage 4's own oracles (`Moon_Native.md` §4): commercial ROMs through `ICore` on both engines, states crossing engines both ways, and the fallback. `MoonRtStateTests`, 71 cases, and `MoonRtMachineTests` unchanged. The exhaustive patch test of §3.12. States equal straight after a load with a battery (§3.4). The bespoke `MoonRtCore` exists now, so the generic host is timed against it interleaved (P2) | stage 4's day, and half a day |
| **3** | **MoonRT stage 5 on the shared hooks**, through `NativeDebugBridge` | Stage 5's: `MoonDebugTargetTests` on both engines, and games with every table armed identical to plain runs; `Moon_Native.md`'s P3 | stage 5's 1–2 days, less the bridge's copy |
| **4** | **MercuryRT.** Its exports are regenerated, `MercuryRtCore` goes over the base, its hooks move to the shared crate, and its flags are renumbered | The 36 test methods in `MercuryRt*Tests`, `MercuryDebugTargetTests` on both engines, and stage 6's mutants run again on the shared hooks (`Mercury_Native.md` §8.5.4). States of versions 5, 6 and 7 load and re-save byte for byte. `frames` example timing and the plain path's symbol sizes, by §8.5.5's method (P4) | 1 day |
| **5** | **MarsRT.** Its exports are regenerated and its extensions renamed; `MarsRtCore` goes over the base; its hooks move to the shared crate, with the profile a `BTreeMap`; its flags are renumbered; −10 and −11 become −257 and −258; the crash log is renamed | The 57 test methods in `MarsRt*Tests` and `MarsRTViTests`. The rewind tests' state and picture per landing (`Mars_Native.md` §6.6.3). `pacebench`'s state hash. The three gameplay states' hashes in `examples/threads`. The probe on Super Mario 64 and Ocarina of Time only. v1 states and v2 snapshots from both engines, of both RDRAM sizes, load | 1–2 days |
| **5a** | **MarsRT's PGO profile retrained** | `pgo/train.sh` (about 55 minutes, `Mars_Native.md` §6.17.8); the build's verdict `matched`; no function without data under `-pgo-warn-missing-function` on the CI flavour (§6.17.7) | an hour and a half |
| **5b** | **MarsRT timed** | Interleaved builds before and after, the order swapped each round, on Super Mario 64, Ocarina of Time, the Dam and DK64's title, as §6.17.6 timed them (P3) | 2 hours |

*Step 1's registration-equivalence test was not built with step 1. It was built on 2026-10-04, ahead of the move to
DianaOS that it now guards (`EmuSen_CoreAPI.md` §25).*

**Why this order.** MoonRT first, because it has no engine row yet, so nothing a player runs can change, and because
its bespoke shim exists to be timed against. MercuryRT second, because its measured sensitivity to layout and hashers
(`Mercury_Native.md` §8.5.5) is the sharpest test of the claim that nothing hot moves. MarsRT last, because it is the
default engine and PGO-built, and because the crate should already be proven twice before it is used there.

**Why the profile is retrained even though nothing hot moves.** Moving the crate's path dependencies changes the
source digest. The digest is being extended to cover them (`EmuSen_RustState.md` §4.3 on its branch), so the verdict
becomes `stale`. `Mars_Native.md` §6.17.6 makes a retrain the rule before any speed number is quoted (rule 3). It also
measured that a stale profile loses little, and only in steps. What loses records is renamed or moved functions: the
exports, which are cold, and the hooks, which only an observed frame runs and training never arms. So the prediction is
that the retrain changes nothing measurable (P5). The retrain is done anyway, because 5b's number is to be quoted.

**MercuryRT is paused** (since 2026-09-24). Step 4 touches a paused port, so §9's Q2 asks whether it is done then or
when the port resumes. Until it is done, MercuryRT keeps its own ABI and bridge, and the shared crate carries no
MercuryRT-only concession. *Done 2026-09-30 (§12.5), as Q2 decided.* The shared crate still carries no MercuryRT-only
concession. The Game Boy's parts are the `Model` setting and six `mercuryrt_*` extensions.

---

## 8. Costs, risks and predictions

### 8.1 Predictions to be retired

| # | Prediction | Retired when |
|---|---|---|
| P1 | Every existing save state of every engine loads and re-saves byte-identically through the new host | Steps 2, 4 and 5. **Held for MercuryRT 2026-09-30** (§12.5) |
| P2 | MoonRT's plain frame through the generic host is within ±1% of the bespoke `MoonRtCore`'s, interleaved | **Retired 2026-09-30, held:** −0.65% to +0.48% on the four games, geometric mean +0.06% (§12.3) |
| P3 | MarsRT's plain frame is within ±1% of today's on each of the four measured states, retrained, interleaved | Step 5b |
| P4 | The plain path's functions keep their sizes in MercuryRT's and MarsRT's symbol tables. Only `create` and the functions that allocate or configure the hooks differ | Steps 4 and 5. **MercuryRT 2026-09-30: held for the plain path once one inlining loss was removed; the cold functions that differ include `load_state` and `write_space`; and equal sizes did not mean equal speed** (§12.5) |
| P5 | Retraining MarsRT's profile moves no measured game by more than half a point against the stale profile | Step 5a |
| P6 | The C# shims shrink from 2,514 lines (the eleven loader, handle, adapter and bridge files of §1.4) to about 1,900, the host about 850 of them | Step 5 |
| P7 | The Rust `ffi` modules and hooks shrink by about 600 lines net, the shared crate gaining about 700 | Step 5 |
| P8 | No test of the three RT suites changes an assertion; only constructors and names change | Steps 2, 4 and 5. **Held for MercuryRT 2026-09-30** |

P6 and P7 are estimates from the duplication of §1.3 and §1.4, not derivations. They are held to ±30%.

### 8.2 Risks

- **R1, layout.** Both 2D ports measured a few per cent from moving a field (`Mercury_Native.md` §8.5.5, `Mars_Native.md`
  §6.5.5). The shared `Hooks` stays behind each core's box, so the machine's field is still a pointer. The machine
  structs are not otherwise touched. P4 and the timing steps are the guard. Step 4 showed that only the timing
  guards placement: MercuryRT's plain frame lost 2.5–7.5% with every hot function the same size (§12.5).
- **R2, the hasher.** §3.14. MarsRT's `HashMap` goes, and MoonRT is built with the `BTreeMap` from the first day.
  P4 is the guard.
- **R3, the input semantics.** §3.8's finding. A host that re-sends whole masks would pass every test that does not load
  a state while holding a button. The oracle for step 5 must include one: load a state saved while a button was held,
  run without pressing, and compare to C# Mars.
- **R4, a behaviour difference hidden in the frame-end order.** Each console's `EndFrame` must be its oracle's order.
  The default is Mercury's, so a console that forgets to override gets Mercury's order. The registration-equivalence
  test cannot see this. Each console's own frame-by-frame oracle can.
- **R5, fixed names forbid static linking** of two cores into one image. Nothing EmuSen builds does that. It matters
  only for a platform without dynamic loading, which `EmuSen_Libretro.md` §9 Q3 raises for iOS. If it ever does, one
  function-table export can be added beside the names (§10).
- **R6, churn in a paused port.** §7, and §9's Q2.
- **R7, the debugger bridge's first-instruction check.** Unifying MarsRT's and MercuryRT's forms without proof could
  change when a halt lands. The bridge keeps each console's form until §9's Q7 is answered by a test on both.

---

## 9. Questions, decided 2026-09-28

Each question as it was put, and its decision.

1. **Q1, the crate's name.** Decided: the state crate is named `emusen-native` from the start, not renamed later, since
   it will hold the interface (§3.17).
2. **Q2, MercuryRT while paused.** Decided: it moves at step 4 regardless. It is the Game Boy engine in use, and moving
   it is framework work, not a resumption of its port.
3. **Q3, exact version matching.** Decided: exact on both halves (§3.2). *Superseded 2026-10-01 by
   `EmuSen_CoreAPI.md` §4: a stable major with additive minors.*
4. **Q4, where a settings list lives.** Decided: in C# until the C# cores retire; the `settings_schema` export is added
   then, so there is never a second source for the same list. *Superseded 2026-10-01 by `EmuSen_CoreAPI.md` §6.13: the
   schema is part of v1, and the one-source rule is kept engine by engine.*
5. **Q5, `EmuSen_Multicore.md` §2's position on `Bundle`'s switch.** Decided: retired (§5.3).
6. **Q6, MarsRT's crash log's file name.** Decided: `native_crash_<pid>` becomes `marsrt_crash_<pid>` (§4.1).
7. **Q7, the first instruction's breakpoint check.** Decided: in Rust, in every core, as MercuryRT does it (§4.4).
8. **Q8, one trait for this interface and a libretro build.** Decided: one trait, so `NativeCore` is shaped now for
   `reset`, a region and a session's constant state size (§6.3).
9. **Q9, the handheld.** Decided: step 5b is also timed on the Legion Go S, on charger power (`Mars_Performance.md`
   §42), beside the desktop's rounds.
10. **Q10, sound at a rate change.** Decided: fixed. The drain reports the rate its samples were made at (§3.7), with
    a test at a rate change.
11. **Q11, the 8-bit cores' lazy patch refresh.** Decided: the host refreshes before a host read of `CPUBUS`, so a
    debugger read matches the C# core; the change carries its own oracle (§3.12).
12. **Q12, MarsRT's stack push-down.** Decided: fixed as a separate change, outside this migration (§3.14).

---

## 10. Rejected alternatives

- **Adopt libretro as the interface.** It would bring callbacks into managed code, one global instance, raw memory
  pointers, cheats parsed by each core and no debugger (§6.2). Every one of those is a rule this project set for a
  reason and measured against. The libretro build remains possible as an adapter (§6.3).
- **Callbacks, of any kind.** A panic hook is the only thing Rust runs on the host's behalf, and it does not call
  C#. A callback per frame event would cost a reverse transition each time. It would also forbid the cheapest call form
  (`Mars_Native.md` §3.2), and it would make "the library never runs managed code" untrue.
- **A single global instance.** It forbids the tests' many machines, two engines in one process, and any run-ahead
  without copying the library file.
- **One function-table export**, `emusen_native_api(version) -> *const Api`. It allows static linking and costs one
  lookup, but its layout needs versioning, `nm` stops showing what a library offers, and optional entries still need
  bits. It is a cheap addition later if R5 ever matters.
- **A multiplexed extension call**, `extension(id, in, out)`. It loses the types and the load-time check, for no gain
  over a named export, which costs nothing.
- **Messages across the boundary** (JSON, protobuf, flatbuffers). They add a parse per call and a dependency. They buy
  schema evolution between binaries built separately, which never happens here. The settings text of §3.13 is a
  handful of lines between frames.
  *Retired 2026-10-01 for descriptors read once, by `EmuSen_CoreAPI.md` §6.3: binaries built separately are now
  intended, so schema evolution is what is wanted. The rejection stands for every call on the hot path.*
- **Generated C# bindings** (cbindgen, csbindgen). A build dependency for 42 names. A test that compares `nm -D`'s list
  with the C# table catches drift as well.
- **Sharing only the C# helpers** and keeping three ABIs. It leaves the status collisions, the drifted hooks and a
  bespoke shim per new core. That is the state this document replaces.
- **A second Rust crate beside `emusen-state`**, or a new C# assembly. Neither boundary would be used separately
  (§3.17, §4).
- **Exposing memory without copying** for the debugger's 8 MB view. MarsRT's debugger reads live through copies of
  what it shows, and a pointer would bypass the drain wait (§3.1).
- **Keeping the settings exports** (`set_threads`, `set_multiple`, `set_recompiler`) as common calls. They are one
  console's groupings. The batch of §3.13 keeps their atomicity without putting N64 nouns into the common set, as
  `EmuSen_Multicore.md` §5 refused SNES nouns on `ICore`.

---

## 11. What this does not cover

- The C# cores' `ICore` implementations, beyond registering them.
- MarsRT's component exports and C# Mars's use of them.
- A settings schema from the library (Q4), achievements, rumble, the 64DD or the Transfer Pak. None of these exists in
  a Rust core today.
- Windows and macOS. The loader's rules are argued for all three hosts (§3.2). Nothing here was run on anything but
  Linux x64.

---

## 12. What was built: steps 1 and 2 for MoonRT (2026-09-30)

*§7's first two steps, cut to what MoonRT needs. The prediction retired here is P2. P1 and P8 hold for MoonRT's part
and stay open for the other two cores.*

### 12.1 The Rust side

`emusen-native` gains the module `abi` (`src/abi.rs`), as §3.17 places it. It holds:

- the constants: `COMMON_VERSION` 1, `version(core)`, the fourteen capability bits of §3.2, the interface's codes
  −256 to −262 and the fault base −320 of §3.3, and the five fault kinds;
- `NativeFile`, `File`, `FrameInfo` (§3.6's layout), `Region`, and `Settings`, the `key=value` parser;
- the `NativeCore` trait;
- `install_crash_log`;
- `native_exports!`.

**The trait, shaped for §9 Q8.**

- It has `reset` behind `RESET`.
- It has a `region()`, `Ntsc` by default. No export reads it yet; it is there for a libretro adapter's AV info.
- It requires that kind 0's state size is constant for a machine's life, which `retro_serialize_size` needs. MoonRT's
  is, and a test pins it over 1,800 frames of each bench game.
- `end_call` is the macro's hook for §3.17's rule that no fault outlives the call that made it. MoonRT's takes its
  thread-local fault.

**The macro writes only the optional groups it is given.** It is invoked as
`native_exports!(Machine; reset, mutes, rom_patches)`. A `macro_rules` macro cannot read a trait's constant, so §3.16's
rule that "a library exports only the optional names its capabilities claim" is enforced from both ends:

- a compile-time assertion that the named groups' bits are exactly the optional bits in `CAPABILITIES`;
- `The_capabilities_and_the_exports_agree`, which loads the library and checks every bit against its exports, in
  both directions.

**Only MoonRT's optional groups are written:** reset, mutes and ROM patches. The generator for present, phases, audio
peek, axes, settings, snapshot and the debug exports is not written. Each comes with the core that first needs it.

**One export was added to the common set: `emusen_native_set_audio_limit(handle, samples) -> i32`.** §3.7 omitted it,
although every core then had it (`*_machine_set_audio_limit`, the host's `AudioSettings.AudioBufferMaxSamples`). It is
required, which makes 24 required names, not 23. MoonRT exports 27 common names:

- the 24 required;
- `reset`, `set_mutes` and `set_rom_patches`.

It also exports two extensions: `moonrt_step` (§3.15) and `moonrt_rom_patch`, a test view of the patch table that
§3.12's exhaustive oracle reads. §3.16's argued 26 for MoonRT is one short, by the audio limit.

**MoonRT, moved.**

- Its `moon_*` exports are retired, as §7 step 2 said.
- Its core version is 3, so its interface version is `0x0001_0003`. It was `moon_interface_version` 2.
- Its faults are −321 to −323 (§3.3).
- The battery file is written into PRG RAM inside `create`, clipped to it, before any frame (§3.4).
- The ROM patches arrive as `ResolveRomPatches`' triples, and Rust builds the bus's 256-entry tables from them (§3.12).
  For each original byte it keeps the first entry whose compare is absent or equal.

### 12.2 The C# side, and how the two ABIs coexist

In `EmuSen/Cores/Native/`:

- `NativeCoreLibrary.Common(crate, variable, coreVersion, requiredCapabilities)`:
  - the fixed names;
  - both halves of the version matched exactly (§9 Q3);
  - the capabilities read, and the library refused, with a report, when it lacks one its engine needs;
  - the crash log at `<crate>_crash_<pid>`.
- `NativeInterface` holds the export table, the constants and the lists of required and optional names.
- `NativeMachine` is the handle:
  - the create contract, with pinned files;
  - one method per export;
  - the shared status words;
  - `ExceptionFor`, which asks the console's band first and then maps −321 to −325 to .NET's types.
- `NativeRtCore<TMachine>` is the `ICore` base of §4.3. It does the load, the frame, the picture, the sound, the state,
  the spaces, the battery, the cheats, the mutes and the mirror.
  - It reuses one state array while the size stands (§3.9).
  - It refreshes the patches before a read of the CPU's bus (§9 Q11).
  - It declares `IEngineFeatures` with every feature kept (§5.2).
  - It carries `INativeDebugBridge`, an empty seam that stage 5's bridge fills.

**The old path is kept beside the new one, under other names.** The per-core classes are renamed
`LegacyNativeMachine`, `LegacyNativeExports` and `LegacyNativeRtCore<T>`. `MercuryMachine`, `MercuryRtCore` and
`MarsMachine` derive from them as before, and nothing else in them changed. `NativeCoreLibrary`'s original
constructor still serves MercuryRT's and MarsRT's own version and crash-log exports. MarsRT's Rust source is
untouched.

The two paths cannot meet:

- A library is read either as common, through `Common`, or as legacy, through the old constructor.
- MarsRT's library does export `emusen_native_interface_version`, but it answers 10. That reads as common 0, so it
  would be refused if it were ever loaded as common.

At steps 4 and 5 the legacy files are deleted.

**What is owed from these steps:**

- **The registration records (§5.1).** The NES's row went in the existing way, as a `CoreCatalog` entry and three
  lines in `CoreFactory`. The records, and the equivalence test that step 1 names, were not built. Moving four
  consoles' registrations is not cheap enough to do beside a stage.
- **§9 Q7, the first instruction's breakpoint check in Rust.** MoonRT has no breakpoints until stage 5, so there is
  no check to place yet. Stage 5 places it in Rust.
- **The snapshot kind.** Kind 1 is refused with `NOT_SUPPORTED`, as MoonRT needs no snapshot.

### 12.3 P2, retired

**P2 held.** It predicted that MoonRT's plain frame through the generic host would be within ±1% of the bespoke
`MoonRtCore`'s, interleaved.

How it was measured, on 2026-09-30:

- `moonbench` was built against 77d71298, the bespoke shim, and against this step's tree.
- Five rounds, each engine's order swapped every round, under the timing lock, with a C# control run beside them.
- Load average 1.1–1.7.
- Every state hash was the same across both builds and both engines.

| Game | Bespoke shim, ms/frame | Generic host | Change |
|---|---|---|---|
| Super Mario Bros. | 1.012 | 1.005 | −0.65% |
| Zelda | 0.913 | 0.917 | +0.48% |
| Super Mario Bros. 3 | 1.336 | 1.336 | +0.01% |
| Punch-Out!! | 1.194 | 1.199 | +0.40% |

The geometric mean is +0.06%. Every game's two ranges across rounds overlap. So the difference is inside the noise,
and no direction is claimed.

### 12.4 Step 3: the debugger's hooks, shared, with MoonRT on them (2026-09-30)

**What is shared.** `emusen-native`'s `debug` module (`src/debug.rs`) holds the data of §3.14 and its helpers:

- `Hooks`, with `configure`, `armed`, `set_stack`, `note_call`, `note_return`, `flush`, `record` and `record_on`,
  `stop_before`, `covers`, `note_write`, `stamp`, `set_breakpoints`, `set_ranges` and `counters`;
- the stop bits, the flags, the run flags and the call kinds;
- `Range`, `Write`, `Call`, `MAX_DEPTH` and `LOG_CAPACITY`;
- per-processor `Coverage`;
- `drain`, `drain_profile` and `drain_coverage`.

`native_exports!` gains two groups:

- `debug`, which has ten of §3.14's exports: `debug_set`, `debug_set_breakpoints`, `debug_set_ranges`,
  `debug_run_frame`, `debug_writes`, `debug_calls`, `debug_profile`, `debug_coverage`, `debug_counters` and
  `debug_pc`;
- `debug_stack`, which has `debug_set_stack`.

They sit over three `NativeCore` methods: `debug_hooks`, `debug_run_frame` and `debug_pc`. The C# side is
`NativeDebugBridge`, with `RunFrame`, `PushTables`, `Drain`, `Grow`, `Observed` for a host store, `Counters` and
`ProgramCounter`. It fills `NativeRtCore`'s seam, whose `RunFrame(resuming, out haltedAt)` returns false on a halt.

**Not generic over the program counter.** Every address in the hooks is a `u32`, which holds the 6502's, the SM83's
and the low word MarsRT's C# oracle compares. The processor's width is given only to coverage, at `Hooks::new`: 16 bits
for a 6502 or an SM83, and 24 and 12 for MarsRT. A type parameter would have made each core's hooks a different type
for no case that needs one.

**The drift of §1.3, reconciled as decided:**

| | Now |
|---|---|
| Stack | (source, target) pairs, as MarsRT's; `set_stack` takes pairs |
| Returns with nothing open | counted (`unmatched_returns`), as MarsRT's, and logged, as MercuryRT's |
| `armed()` | on the shared hooks, as MarsRT's |
| Log capacity | a field, as MercuryRT's |
| `stamp` | shared, as MercuryRT's |
| Call kinds | 0 return, 1 call, then C#'s `CallFrameKind` from 2: IRQ 2, NMI 3, BRK 4, COP 5. MercuryRT's 2 is unchanged; the NES needed NMI and BRK apart |
| An interrupt of any kind | sets the one-shot `INTERRUPT` stop when `INTERRUPTS` is armed |
| Profile | a `BTreeMap` |
| Coverage | per processor, armed by bit 8 + n |
| Flags | `CALLS` 1, `WRITES` 2, `INTERRUPTS` 4, `EACH` 8, `PROFILING` 16 |
| `return_after_slot` | not shared: it is MarsRT's delay slot, a mechanism, and stays in MarsRT (§3.14) |
| Frame continuation | not shared: MoonRT's is `frame_open` in its machine, as MercuryRT's is its budget and MarsRT's its frame clock |

**The host reports each interrupt from the call log.** The bridge calls `Breakpoints.NoteInterrupt` for every push
of kind 2 or more as it drains the log. MercuryRT's bridge calls it from the `INTERRUPT` stop's reason. The difference
matters because the NES has three kinds and the reason carries none. The log is always drained at the stop that
follows a dispatch, so the note lands before the handler's first instruction is asked about, as C# has it.

**MoonRT's mechanism** is `Moon_Native.md` §8.4.2: the CPU generic over two views of the bus, with a flag version
measured and not kept.

**Owed for MercuryRT's and MarsRT's moves, steps 4 and 5:**

- **MercuryRT.** Its `debug.rs` is the shared module's ancestor. It adopts the module with four changes:
  - its stack becomes pairs;
  - its flags are renumbered (`COVERAGE` 16 → bit 8, `PROFILING` 32 → 16);
  - `note_call`'s `interrupt: bool` becomes a kind;
  - `mercury_debug_*` becomes `emusen_native_debug_*`, with `mercury_machine_pc` becoming `debug_pc`.
  Its `ObservedBus` stays its own. Not done now: it is step 4's, with its oracles, and was
  to be done here only if trivially safe; the renumbered flags alone make it not trivial.
- **MarsRT.** Its hooks move with its profile (`HashMap` → `BTreeMap`), its 24- and 12-bit coverage onto
  `Hooks::new(&[24, 12])`, and its `PROFILING` 64 and `RSP_COVERAGE` 32 onto 16 and bit 9. That is step 5, with the
  PGO retrain after it. Its Rust is untouched here.
- **An "armed equals plain" helper in the crate** for the three cores' Rust tests, as MercuryRT's `tests/debug.rs`
  has for itself. The generic loop needs only `NativeCore`, but it was not written. MoonRT's oracle for it is the C#
  one, `MoonRtArmedEqualsPlainTests`, through the whole host.

### 12.5 Step 4: MercuryRT on the interface (2026-09-30)

**What moved.** The full record is in `Mercury_Native.md` §8.7. In outline:

- **The library.** The 36 `mercury_*` exports became 37 `emusen_native_*` ones:
  - the 24 every core has;
  - `set_mutes` and `set_rom_patches`;
  - the ten `debug_*` exports and `debug_set_stack`.
- **Version and capabilities.** The version is `0x0001_0005`, and the capabilities are `MUTES | ROM_PATCHES | DEBUG |
  DEBUG_STACK`.
- **What only the Game Boy has.** The model is the create setting `Model`, and the battery save is file 0. Six
  `mercuryrt_*` extensions carry the rest: the sample rate's two doubles, the serial log, `cgb_hardware`, `step`,
  `rom_patch` and `ram_length`.
- **The status codes.** They are −9, −10, −11 and −20, in the core's band. −20 has the opcode and pc in the detail word,
  which `NativeMachine`'s new `FrameException` seam turns back into C#'s exception.
- **The shim.** `MercuryRtCore` is over `NativeRtCore` and `NativeDebugBridge`. `LegacyNativeRtCore` and
  `RomPatchTable` are deleted, and `LegacyNativeMachine` is left for MarsRT alone.
- **The debugger** is on the shared `Hooks`, with the four changes §12.4 listed.
- **CI.** The export count in `rust-cores.yml` now counts `emusen_native_` for MercuryRT and MoonRT. MoonRT's row
  still counted `moon_`, which step 2 retired, so its job would have counted none and failed.

**Oracles.** All hold:

- The crate's tests pass.
- 586 WiseMan tests over both engines pass: the Mercury, native-host, battery, engine and state-header ones.
- A digest over the picture, the sound and the state matches pre-move MercuryRT on 24 runs. That is 7 games in the
  three models from boot and from a state, and 3 library resume states.
- The fallback to Mercury (C#), with `EMUSEN_MERCURY_NATIVE=0` and without the library, is tested in a child process.

**P4 held for the plain path, and was not enough.** The symbol diff found one inlining regression, and once it was
removed every function of the plain path had its old size. The functions that still differ are all cold, though not
all are `create` or the hooks: `load_state`, the patch table's builder, and `write_space` by one byte. The plain frame
was still 2.5–7.5% slower on DMG games. Built with functions and jump targets
aligned, the moved core is as fast as the old one or faster, so what had moved was the linker's placement of unchanged
code (`Mercury_Native.md` §8.7.3). MercuryRT is now built aligned, from its own `.cargo/config.toml`. It is 0.2–4.2%
faster than before the move through `ICore`, and the armed frame costs 1.14–1.25× the plain one. §8.2's R1 therefore
has a second half: sizes guard against what is inlined, but not against where it lands, and only a timing does.

**Retired or advanced here.** P1 held for MercuryRT. The digest's library resume states, `MercuryRtStateTests`, and
`MercuryDefectTests`' version-5 states from the unmodified build all load on the moved core and re-save as before.
P8 held for step 4: no WiseMan assertion changed. The crate's `tests/debug.rs` changed only how the hooks are
configured and read.

**What the two 8-bit shims still duplicate**, as candidates for `NativeRtCore`. None was moved, so that this step
changes MoonRT not at all.

- The bridge's construction, one coverage table of `0x10000 / 8` over the mirror's registries, with the call stack's
  frame provider.
- `Debug`, `Pc` and the six registry pass-throughs: `Watches`, `FrameLog`, `Breakpoints`, `Coverage`, `Labels` and
  `CallStack`.
- The `WriteSpace` override that sends a listened CPU-bus store through `Observed`.
- `ReportedName` and `ReportedSpace`, which are one table of (id, name) pairs.
- `CheckState`'s magic-and-version check, with the two constants as parameters.
- `MuteMask` as a loop over a channel count.

Together that is about 60 lines in each shim.

**Left for MarsRT, step 5:**

- the hooks, profile and flags of §12.4;
- `LegacyNativeMachine` deleted with its last user;
- `rust-cores.yml`'s `symbol: mars_` changed with the exports;
- the PGO retrain;
- whether its placement matters as MercuryRT's did. PGO lays out the hot code already, so the prediction is that it
  does not, and 5b's interleaved timing is the check.
