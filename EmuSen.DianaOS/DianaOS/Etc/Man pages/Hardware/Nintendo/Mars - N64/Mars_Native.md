# Mars_Native — Rust components beside the C# core

*Started 2026-09-22.* The first question this page answers is whether a component of Mars rewritten in Rust, called
from the C# core, makes the core faster. The first component tried, the signal processor, is **exact and slower**,
and this page records why. It stays in the tree, off by default, as the exact second implementation it is and as the
groundwork a later component would stand on.

## 1. The library and how Mars finds it

`EmuSen/Cores/Nintendo/MarsRT - N64/` is a Rust crate (`marsrt`, edition 2024), built as a C-ABI dynamic library,
`libmarsrt.so` (`marsrt.dll`, `libmarsrt.dylib`). It holds MarsRT, the N64 core in Rust that §5 plans, and the C#
Mars's opt-in native components, of which §3's signal processor is the one so far. It uses `panic = "abort"`, fat LTO,
one codegen unit, and keeps line tables so that a panic's backtrace names its lines. It was `EmuSen.Native/` at the
repository's root until 2026-09-22, when it became MarsRT and moved beside the cores it sits among; the two exports
below kept their `emusen_native_` names, because the C# Mars's twin already speaks them.

`EmuSen.csproj` runs `cargo build --release` before the core compiles, with cargo's target directory at
`EmuSen/obj/marsrt/` so that the build's output stays where MSBuild already ignores it, and copies the library beside
the assemblies as a `None` item. MSBuild carries that item into every consumer's output (Mistress, Hotaru, WiseMan)
and into a publish. Cargo runs only for the host's own runtime identifier. A publish for another platform, or a
machine without cargo, gets no library.

`MarsNative` loads it from `AppContext.BaseDirectory`. It refuses a library whose `emusen_native_interface_version` is
not the build's (2 since §5.1 added the machine's state; 1 before), and installs the panic log. `EMUSEN_MARS_NATIVE=0`
turns the library off. **Every component falls back to its C# twin when the library is absent, refused or off.**
`MarsNativeTests` pins the load.

## 2. A panic never crosses into C#

`emusen_native_set_crash_log` gives the library a path, `DataStore.Logs/native_crash_<pid>.txt`. Its panic hook writes
the message and a backtrace there, and the process then aborts. A panic in an emulator component is a bug, and
unwinding into the runtime is undefined, so stopping with a record is the only honest choice. Mistress's own crash log
cannot see a native abort, which is why the library keeps its own.

## 3. The signal processor in Rust

`src/rsp.rs` ports the C# processor's **plain** path: the scalar unit, the vector unit's arithmetic, all 24
vector loads and stores, and the reciprocal tables built by the same arithmetic (`Mars_RspVector.md` §10). The C#
SIMD path is required to equal the plain path, so either is the oracle. The shift counts of the reciprocal unit are
taken modulo 32 with `wrapping_shl` and `wrapping_shr`, because C# masks a shift count to five bits and the corpus's
model depends on it (§10.1).

### 3.1 Shared memory

- **The registers, the vector registers and the accumulator** are the C# processor's own arrays, now allocated on the
  pinned object heap (`GC.AllocateArray(..., pinned: true)`).
- **IMEM and DMEM** are the bus's arrays, pinned the same way.
- Rust holds their addresses. Nothing moved and nothing is copied, so the CPU, DMA, the RDP, the debugger, cheats and
  save states see exactly the bytes they saw.
- **The scalars** (program counters, halt and break, VCO, VCC, VCE, the divide unit) stay authoritative in C#, because
  the CPU's compiled blocks read `Halted` after every instruction. They are copied into a `#[repr(C)]` struct before
  each call and back after it.
- The C# SIMD accumulator is widened before a call (`WidenAccumulator`).

### 3.2 The native side never calls C#

The calls use `delegate* unmanaged[SuppressGCTransition]`, the cheapest form, about a nanosecond. A native function
called that way must not call back into managed code: a GC could begin during the callback and the thread would not be
in the state the runtime assumes.

So Rust never calls C#. When the next instruction is an **event**, a COP0 move or a BREAK, Rust stops before it and
returns, and C# runs that one instruction through `StepManaged`. That is the old `StepOne` body, unchanged. Every
interaction with the rest of the machine therefore still happens in C#, in the code that was already proven: DMA,
status, semaphore, the RDP's command registers, the break's interrupt. The native side is pure computation over shared
memory.

Three entry points go native when `Rsp.UseNative` is set:

| Entry point | Native call |
|---|---|
| The lock-step `StepOne` | `mars_rsp_step` |
| `SpInterface.Step` | `mars_rsp_run` in a loop, with events run in between |
| The idle loop's `RunBlocks` | `mars_rsp_run` to the first event, which C# then runs, as `RunBlocks` stops after an event block |

The debugger's coverage, and single-stepping, always take the C# path.

### 3.3 Exactness, proven the usual way

**Differential test.** `MarsNativeRspTests.The_native_processor_leaves_the_state_the_csharp_one_leaves` builds a
synthetic machine and fills the processor with a random program covering every instruction class: scalar arithmetic,
branches and jumps, scalar loads and stores, all 64 vector functions, the four COP2 moves, all vector load and store
formats, and BREAKs and COP0 reads so that the hand-back runs. It also randomises the registers, vector registers,
accumulator, flags, divide unit and DMEM. Half the register and element values are drawn from the edges (0, ±1,
0x7FFF, 0x8000, 0xFFFF and small integers), so that equal operands, carries and clamp boundaries occur.

From one saved state it runs 3,000 steps three ways, and compares the whole save state byte for byte:
- the C# plain path;
- the C# SIMD path;
- Rust.

It does this for 150 seeds through each of the three entry points.

**Mutants.** Nine were made in the Rust code:

| Mutant | Result |
|---|---|
| The plain fraction multiply's rounding constant | caught |
| MTC2 writing its second byte with a wrap | caught |
| The carry flag of the carrying add (`>` to `>=`) | survived, then caught after the edge values were added |
| SLTIU (`<` to `<=`) | survived, then caught after the edge values were added |
| CLIP-low's extension flag inverted | caught |
| The unsigned clamp's boundary (`>` to `>=`, at 0x7FFFFFFF) | **survives** |
| The transposed store's rotation sign | **equivalent**, cannot be caught |

The unsigned clamp's boundary is reachable only through a VMACU whose accumulator lands exactly there; a plain VMULU's
result is always even. The rotation sign is equivalent because the quantity rotated, `aligned >> 1`, is always a
multiple of 4, and ±4 are the same modulo 8. No test could catch it, and the code is correct either way.

**The rest of the suite with the native processor forced on** (`EMUSEN_MARS_NATIVERSP=1`) passes:
- the RSP and vector classes;
- the microcode and game-probe tests;
- the block, idle, SP-memory and DMA tests;
- the n64-systemtest hardware corpus, which still reports exactly "Failed 46 of 4637".

`MarsIdleTests` now pins `UseNative` off, because it asserts on the C# processor's own compiled blocks, which the native
path does not use. The whole Mars suite passes with the default off (3,545 tests).

**In play.** Three interleaved rounds of 1,500 frames flat out, from gameplay states in SM64, OoT and GoldenEye's Dam,
gave the same state checksum in all eighteen runs: `EA3ECCB37EC9E0C9`, `8CB10CB4CADBE713` and `2977CD4026CE8E75`.

### 3.4 The measurement, and why the component loses

**Mean frame, three interleaved rounds:**

| Game | C# | Rust, every entry point | Rust, stepped paths only (idle loop keeps C#'s compiled blocks) |
|---|---|---|---|
| SM64 | 5.81–6.04 ms | 6.57–6.66 ms (+11%) | 6.10–6.26 ms (+4%) |
| OoT | 7.33–7.39 ms | 8.09–8.19 ms (+11%) | 8.16–8.19 ms (+11%) |
| GoldenEye | 18.36–18.65 ms | 23.80–24.19 ms (+29%) | 21.94–21.98 ms (+18%) |

**Per step, on the differential test's random programs** (`MarsNativeRspTests.Bench`, `EMUSEN_RSP_BENCH=1`, Release):

| Entry point | C# | Rust |
|---|---|---|
| Lock-step single step | 21.1 ns | 15.3 ns |
| `SpInterface.Step` | 20.5 ns | 7.9 ns |
| `RunBlocks` | 26.0 ns | 9.0 ns |

The Rust interpreter is faster than the C# interpreter at every entry point on that code. The games are slower for two
reasons, one per path.

- **In bulk, the competition is not the C# interpreter but the C# compiled blocks.** Random code defeats the block
  compiler, which is why C# costs 26 ns there. In a game, 60 to 90 per cent of the processor's steps run in the two-tier
  compiled blocks of `Mars_Rsp.md` §11.2, at about 3 ns a step. A native interpreter at 8 to 9 ns loses to them. This is
  what the stepped-only column removes.
- **In lock-step, the boundary costs more than the step.** GoldenEye makes about 462,000 lock-step calls a frame, and
  the stepped-only mode made its frame 3.4 ms slower, about 7 ns more per step than C#.

  The C# `RspRan` has the whole interpreter inlined (`Mars_Rsp.md` §14's disassembly). The native call pays twenty
  scalar copies, the accumulator check and the call before it interprets.

  The first explanation offered was that the processor spins on COP0 status reads while the CPU is busy, and that
  every such read costs a wasted native call. **It was refuted by count:** only 4.5% of GoldenEye's lock-step calls met
  an event (7.4% in OoT).

**What would make a native processor pay:**
- **Not moving the scalars into native memory.** Removing the twenty copies would recover a few nanoseconds of the
  seven at most, still against a C# step that has no call at all.
- **What would change the verdict:**
  - *Batching the lock-step*, that is, the run-ahead of `Mars_Rsp.md` §13's route two. Run-ahead is where the bulk
    paths' 8 ns would count, but only against C#'s compiled blocks at 3 ns, so it needs a native compiler too.
  - *A native CPU backend* that calls the processor native to native, with no managed boundary on each cycle.

Either is a larger piece of work than this page's, and this page's measurement is the reason to price it before
building it.

**Why this does not generalise.** The loss is a property of the granularity, not of Rust. A component entered once
per emulated cycle from managed code that already inlines its competitor cannot win on the call. A component entered
once per frame or per task, such as a rasteriser, faces no such boundary. Nothing here measures one.

## 4. State of the switch

`Rsp.UseNative` is off. `EMUSEN_MARS_NATIVERSP=1` turns on every entry point, and `=step` turns on only the stepped
ones. Both are for measurement and differential testing. With the switch off, Mars runs exactly as before this page:
the three games' frame times with the switch off matched the preceding commit's.

## 5. The whole machine in Rust (planned 2026-09-22)

§3.4's lesson decides the shape. A component entered once per emulated cycle loses on the boundary. So the port moves
the **whole machine**, and the boundary is crossed **once per frame**: input in, and the picture and audio samples out.

**It is a second N64 core, not a replacement, and it is named MarsRT.** Its crate is a folder beside the other cores,
`EmuSen/Cores/Nintendo/MarsRT - N64/`, rather than a component inside Mars's, because it is a core of its own. The
Rust machine runs behind a C# shim that implements the same interfaces as `MarsCore` (`ICore`, `ISnapshotCore`,
`ICoreSettings`, `IFrameSerial`, `IRepeatedRows`, `IStateFormat`, `ICheatRegistryHost`), so the frontends and the
tooling do not change. The C# Mars stays canonical, exact and the fallback, and a setting chooses between them.

**The C# save state stays the format.** The shim transfers state between the two machines field by field, so a state
written by either loads in either, and every existing `.state` file keeps working. That transfer is also the oracle's
seam: run both machines from one state, and compare the C# save state of each after every frame. This is the
measurement the whole of Phase G used, applied across two languages.

**The order,** each stage proven against the C# core before the next begins:

| Stage | What it covers | Its oracle |
|---|---|---|
| 1 | The machine's skeleton and bus, RDRAM, cartridge and PI, boot, and the CPU interpreter with COP0, the timer and interrupts, the TLB and software floating point | The CPU parts of n64-systemtest; boot frames against the C# core |
| 2 | The signal processor (§3, already exact) and SP DMA | §3.3's differential; the microcode tests |
| 3 | The RDP rasteriser and its workers | The RDP differential against angrylion and against the C# RDP |
| 4 | VI scan-out, audio, SI, PIF and joybus, the save chips | Frame-by-frame state against the C# core in the golden-probe games |
| 5 | The recompiler, on Cranelift | Its interpreter, and the C# recompiler's block tests |
| 6 | The GPU path, on ash | The C# GPU path's tests |

**What is deliberately kept out:**
- The debugger's deep inspection: the Rust core serves the debug target through the state transfer, not live.
- The coverage recorder, single-stepping and cheats: these run on the C# core until the Rust one has an equivalent.

**Why the port may still fail to pay,** stated before it starts:
- The C# core's remaining costs are memory traffic in the recompiler (`Mars_Recompiler.md` §12.3), the RSP's lock-step
  (§3.4), and the RDP's pixel work on other threads.
- A Rust machine removes the boundary from all three and gives the recompiler register allocation. It does not change
  what the lock-step requires: one RSP step per CPU cycle.
- The estimate before any measurement is 1.4 to 1.7× on GoldenEye and less on SM64 and OoT (`Mars_Native.md` §3.4 is
  why estimates here are held loosely). Stage 1's interpreter is the first point at which a like-for-like number exists,
  and it is to be measured against the C# interpreter, with the recompiler off in both, before stage 5 is priced again.

### 5.1 The state, byte for byte

*Stage 1a, 2026-09-22.* The port's first step is the machine's data and not its behaviour: every field the C#
serializer walks, held in Rust structs, and a reader and writer under which MarsRT's state **is** the C# save state,
byte for byte. The C# serializer therefore stays the format, as §5 requires, and a state written by either machine
loads in the other because it is the same bytes. Nothing runs yet, and nothing a C# load derives after reading is
derived here.

**What was built.**

- **One module per C# class,** in `src/`:

  | Module | C# classes | Lines |
  | --- | --- | --- |
  | `cpu`, `tlb` | `Cpu`, `Tlb`, `TlbEntry` | 74, 44 |
  | `bus` | `MemoryBus`, and the tail it writes by hand | 205 |
  | `mi`, `pi`, `ai`, `vi`, `isviewer` | `MiInterface`, `PiInterface`, `AiInterface`, `Vi`, `IsViewer` | 36, 28, 47, 52, 27 |
  | `si`, `controller` | `SiInterface`, `Controller`, `ControllerPak` | 33, 58 |
  | `sp` | `SpInterface`, and `Rsp`'s registers | 109 |
  | `dp`, `rdp` | `DpInterface` and a snapshot's words; `Rdp`, `Color`, `TextureTile` | 71, 411 |
  | `save` | `SaveChip`, `Eeprom`, `Sram`, `FlashRam` | 176 |
  | `machine` | `MarsCore`'s header, and the `Machine` that owns everything | 223 |
  | `state`, `ffi`, `naming` | the encodings; the C ABI; the naming rule's test | 410, 105, 101 |

  §3's interpreter, `rsp.rs`, is untouched. Its registers and the machine's `sp::Rsp` are two structs until stage 2
  joins them.
- **The naming rule.** Each Rust field is the snake_case of its C# name without the leading underscore, and an
  auto-property's backing field `<X>k__BackingField` is named for `X`. There are two exceptions: `SaveChip.Type` is
  `kind`, since `type` is a keyword, and a snapshot's words, which are no C# field, are `DpInterface::pending`. What
  C# marks `[SkipInState]` but a hand-written part of the format carries is held where C# holds it:
  `SiInterface::due` and `pending_read`, `Controller::pak`, and `MemoryBus::registers` and `save`. `Color` declares
  its fields red, green, blue, alpha and writes them in the serializer's order, which is A, B, G, R.
- **The writer takes each field's C# name as an argument, and the reader carries it as a comment.** The order is
  therefore written out twice, once in each function, and both copies are checked (below).
- **A load parses into a fresh machine and replaces the old one only when it succeeds,** so a truncated or refused
  state changes nothing. The C# load reads into the live machine.
- **A state of version 1 cannot be written while display-processor words are pending.** C# drains them by running
  them before it writes, and that needs the processor stage 3 ports. A snapshot carries them.
- **The C ABI** is `mars_machine_new(rdram_bytes)`, `_free`, `_rdram_bytes`, `_load_state(ptr, len)`,
  `_state_kind` (the version last loaded: 1 a state, 2 a snapshot), `_save_state_size(snapshot)`,
  `_save_state(ptr, len, snapshot)` and `_state_layout(snapshot, ptr, len)`. A negative return is a status. The
  interface version is 2. `MarsRT - N64/Shim/MarsMachine.cs` wraps the handle.

**How the field lists were established.** They were not transcribed from a summary or from reading the classes. A
throwaway reflection walk over a live `MarsCore`, using the serializer's own query (every instance field, ordinal
sort, `[SkipInState]` and alias fields dropped), printed every serialized field with its type and array length. A
throwaway script generated the modules from that listing, and they were then edited by hand. It found 210 fields in 21
types, among them 19 in `Cpu`, 15 in `MemoryBus`, 13 in `Rsp`, 17 in `TextureTile` and **74 in `Rdp`**. The brief this
stage was planned from said 78; the count here is the serializer's. The listing is not trusted as final either,
because the tests below make the same query at test time: a field added to a C# class later fails them.

**Three properties, each checked separately.** A byte round trip alone cannot see a consistent mistake. If the
reader and the writer both exchange two fields of one size, the bytes come back whole and the Rust machine holds
each value in the other's field. So the evidence is split three ways.

1. **The reader inverts the writer on C#'s bytes.** A C# state is loaded into MarsRT, MarsRT writes it back, and the
   bytes are compared (`MarsNativeStateTests`).
2. **Every field sits where the C# serializer puts it, under its C# name.** MarsRT's writer can emit a layout: one
   line per field, giving offset, length, type and path, such as `816 8 u64 Cpu.Hi`. The test builds the same
   listing by walking the C# objects as the serializer walks them, and compares the two line by line. A 4 MB state
   has 504 lines and a snapshot 507.
3. **Each label names the field on its own line.** `naming.rs` reads the modules' own source. In 214 writer lines and
   215 reader lines, it checks that the label or comment is the C# name of the field written or read under the
   naming rule. This is the only one of the three that could catch a Rust field holding its neighbour's value under
   the correct label.

**The round trips,** all identical:

| Case | What it covers |
| --- | --- |
| A running machine, state and snapshot | 300 frames of a synthetic ROM with an EEPROM, 9.7M cycles; a 6,433,277-byte state loaded into an 8 MB MarsRT, which rebuilt itself to 4 MB |
| Noise in every field, eight cases | every save type, 4 MB and 8 MB, state and snapshot; three paks and an empty port; five stub registers; an SI transfer under way |
| A snapshot with 37 pending words | the tail written by hand, since the C# thread seldom stands with words unrun |
| An odd bool | four bool bytes set to 2: MarsRT's re-save equals the C# core's re-save, which writes them back as 1 |
| `sm64.state`, `oot.state`, `ge-dam.state` | 12,724,733, 12,755,456 and 12,724,733 bytes, all version 1 on 8 MB; copies read through `EMUSEN_MARSRT_STATES`, and passed unrun without it |

The odd-bool case also found that the C# core's own load and save is the identity on that state once the pak's
dirty flag is already set, which is what made it usable as the oracle there.

**Mutants.** Eighteen were made in the Rust state code. The first sixteen were run against the WiseMan tests; then all
eighteen were run against those tests and `cargo test`, and each survivor was run again after the test written for it.

| Mutant | Result |
| --- | --- |
| `Hi` and `InDelaySlot` exchanged in the writer only | caught: bytes and layout |
| `Hi` and `Lo` exchange places in both, the names following | caught by the layout only; the real states' bytes came back whole |
| The RSP's `NextPc` and `Pc` exchanged in the reader only | caught: bytes |
| `Vi._wasBlank` dropped from both | caught: bytes and layout |
| `ControllerPak.Dirty` dropped from both | caught: bytes and layout |
| `DpInterface._current` and `_end` hold each other's values under the right labels | **survived**; caught once `naming.rs` was written for it |
| `Color` written R, G, B, A in both, the names following | caught by the layout only |
| The registers written in descending order | caught: bytes (the noise case) |
| `SiReadTo` left among the registers and `pending_read` not decoded | **survived** the WiseMan tests, whose bytes come back whole; caught by `cargo test`'s whole-machine comparison, which the first round did not run |
| The SI's due cycle decoded with its halves exchanged | caught: bytes and `cargo test` |
| The snapshot's padding not written | caught: bytes |
| A 16 Kbit EEPROM built as SRAM | caught: bytes |
| A bool read as `== 1` | **survived**, since no C# writer makes another byte; caught once the odd-bool case was written for it |
| A class's present flag read and ignored | **survived** for the same reason; caught once a `cargo test` that clears one was written |
| `TextureTile`'s `SH` and `SL` exchanged in the writer | caught: bytes and layout |
| The RDRAM size check accepting any size | caught: the refusal tests |
| The snapshot's padding left unread | **survived**, because the padding is the stream's last bytes; caught once MarsRT's refusal of a snapshot cut short inside it was tested |
| `Eeprom.Large` left as the type built it | caught: the noise case |

None survives the final suite. Two were caught only by an oracle that exists because the bytes could not see them:
the mislabel (`naming.rs`) and the undecoded SI field (`cargo test`, which compares the Rust machine and not its
output). Those two are the class a byte-exact claim does not cover by itself.

**What surprised, in the format.**

- **The order is the ordinal order of the names,** not declaration order. So `Color` is A, B, G, R. A backing field
  (`<`) sorts before every other name, and upper case sorts before an underscore.
- **The RDP walker's scratch is in every state:** `_edgeLeft`, `_edgeRight`, `_edgeInvalid`, the span arrays and
  `_coverage`, about 82 KB. The machine's own processor always has 1024 rows. Only the processors that draw at a
  multiple call `Widen`, and they are not serialized, so the format has a fixed size. If the serialized processor
  were ever widened, the state would change length with no version change.
- **The SI's transfer rides among the stub registers,** at three addresses no bus reaches. After a C# save the
  dictionary keeps them, because writing a state mutates it.
- **Every present flag in a Mars state is 1,** since every class field is built with its owner. The C# reader, given
  a 0, reads none of that object's fields and leaves the live object as it was. MarsRT, loading into a fresh machine,
  leaves power-on values. The two can differ only on bytes no C# writer produces.
- **The dirty flags are written, and the C# load then forces them true** on the save chip and on every pak. MarsRT
  keeps what it read, which byte-exactness requires. Marking them dirty is the host's business, and belongs to the
  shim that later writes the save files.
- **The C# reader seeks past a snapshot's padding without reading it,** so it accepts a snapshot truncated inside the
  padding; the test shows it. By its code, and untested, a count over 32,768 makes it read past the tail and then
  seek backwards. MarsRT refuses both.
- **`Sram._banks` is serialized, but the data's length is set by the save type,** which is read first. A state whose
  `_banks` disagrees with its type is read with the type's length by both readers.

**Left out.**

- **Everything `[SkipInState]`.** That is the derived or host state: the CPU's mode, timer and blocks; the bus's next
  event and write counter; the display processor's ring, threads, stamps, scaled processors and device; the VI's and
  AI's schedules; the frame buffers; the ROM; `SaveChip._saved`.
- **What a C# load does after reading:** `Cop0Written`, `AccumulatorWritten`, `Rebase`, `Reschedule`, `Refresh`,
  `RefreshShadow`, dropping undrained audio, and running pending words. All of it is behaviour and arrives with the
  stages that own it.
- **Power-on values.** `Machine::new` is zeros, except where a C# field initializer is a constant: `_lastFrameCycles`,
  the SI's idle markers, `RiSelect`'s 0x14, and the save chips' 0xFF. The rest is stage 1's boot.
