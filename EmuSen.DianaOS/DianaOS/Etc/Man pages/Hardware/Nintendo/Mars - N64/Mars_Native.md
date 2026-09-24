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
machine without cargo, gets no library. A `-r linux-x64` publish on this machine, whose SDK names the host
`fedora.44-x64`, does get it, because the runtime identifier is not passed on to `EmuSen.csproj` (§5.5.3).

`MarsNative` loads it from `AppContext.BaseDirectory`. It refuses a library whose `emusen_native_interface_version` is
not the build's (4 since §5.5 gave the host the machine's memories and a frame in two halves, 3 since §5.2 gave the
machine its behaviour, 2 since §5.1 added its state, 1 before), and installs
the panic log. `EMUSEN_MARS_NATIVE=0` turns the library off. **Every component falls back to its C# twin when the
library is absent, refused or off.**
`MarsNativeTests` pins the load.

## 2. A panic never crosses into C#

`emusen_native_set_crash_log` gives the library a path, `DataStore.Logs/native_crash_<pid>.txt`. Its panic hook writes
the message and a backtrace there, and the process then aborts. A panic in an emulator component is a bug, and
unwinding into the runtime is undefined, so stopping with a record is the only honest choice. Mistress's own crash log
cannot see a native abort, which is why the library keeps its own.

## 3. The signal processor in Rust

`src/rsp/mod.rs` ports the C# processor's **plain** path: the scalar unit, the vector unit's arithmetic, all 24
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

*Layout, 2026-09-22.* The crate's `src/` mirrors the C# Mars's folders:

| Rust | C# |
|---|---|
| `cpu/` | `Cpu/` |
| `memory/` (the bus and every device) | `Memory/` |
| `rsp/` | `Rsp/` |
| `rdp/` | `Rdp/` |
| `vi/` (timing, with the scan-out as `vi/scan`) | `Vi/` |
| `rom/` | `Rom/` |
| `ffi/` (the C ABI and the test ABIs) | the shim's interface |

`machine.rs` (the counterpart of `MarsCore` and `Boot`), `state.rs` and the tests stay at the top. The move changed
module paths only. The exported symbols and the state layout did not change, and the 314 crate tests and 398 WiseMan
tests passed before and after it.

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
| 5 | The recompiler, on Cranelift (§5.8) | Its interpreter, and the C# recompiler's block tests |
| 6 | The GPU path, on ash | The C# GPU path's tests |

**What is deliberately kept out:**
- The debugger's deep inspection: the Rust core serves the debug target through the state transfer, not live.
- The coverage recorder, single-stepping and cheats: these run on the C# core until the Rust one has an equivalent.
  *Cheats since §5.5:* MarsRT applies the registry as Mars does; the coverage recorder and single-stepping are still out.

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

  §3's interpreter, `rsp/mod.rs`, is untouched. Its registers and the machine's `sp::Rsp` are two structs until stage 2
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

### 5.2 The machine core

*Stage 1b, 2026-09-22.* MarsRT runs a game. Everything of the C# machine except the RDP's rasteriser (§5.3) and the
VI's scan-out (§5.4) is ported: the VR4300 interpreter, the bus, every device, the signal processor in lock-step,
boot, the frame, and everything a C# load derives after reading. The C# interpreter is the oracle. The claim is that
MarsRT leaves the C# save state after every frame, byte for byte. Once §5.3 and §5.4 were merged, it also shows the
same picture and plays the same sound.

**What was built.**

| Module | The C# it ports | Lines |
| --- | --- | --- |
| `cpu/interp.rs` | `Cpu.Step` and the fetch, `TranslateAccess`, `Cpu.Dispatch`, and the ALU, branch, load and store, unaligned, multiply and divide, trap and COP2 files | 816 |
| `cpu/cop0.rs` | `Cop0Registers` and `Cpu.Opcodes.Cop0`: the registers and their masks, Random, the interrupt check, the timer, exceptions, the TLB instructions | 349 |
| `cpu/cop1.rs` | `Fpu`, `Cpu.Opcodes.Cop1` and `Cop1Math`: the register file in both modes, the control word, the formats, compare, delivery | 284 |
| `cpu/softfloat.rs` | `SoftFloat`, `SoftFloatMath`, `SoftFloatConvert`, `HostSingle` | 606 |
| `cpu/segments.rs`, `cpu/tlb.rs` | `Segments`; `Tlb.TryTranslate`, `Probe`, `PairedPageMask` | 66, 110 |
| `cpu/idle.rs` | the idle test in `StepBlock`, then `RunIdle`, `RspRan`, `AfterInstruction` | 130 |
| `memory/bus_access.rs` | `MemoryBus`: reads and writes by region and width, `Store`, `Load`, the cartridge latch, the MI's repeat, `Tick`, `RunEvents`, `Settle`, `Reschedule`, `Count` | 404 |
| `memory/mi.rs`, `memory/pi.rs`, `memory/si.rs`, `memory/joybus.rs` | `MiInterface`; `PiInterface`; `SiInterface`; `Joybus` | 126, 135, 134, 119 |
| `memory/ai.rs`, `vi/mod.rs` | `AiInterface`; `Vi.Timing` and the VI's registers | 237, 173 |
| `memory/sp.rs`, `memory/dp.rs` | `SpInterface`, and the processor's two instructions that reach the machine; `DpInterface`'s registers and `Take` | 496, 179 |
| `memory/save.rs`, `memory/controller.rs`, `memory/isviewer.rs`, `rom/mod.rs` | `SaveChip`, `Eeprom`, `Sram`, `FlashRam`; `ControllerPak`; `IsViewer`; `RomImage`, `Cic`, `SaveTypes` | 479, 150, 50, 229 |
| `machine.rs` | `Boot.HandOff`, `LoadRom` with `LoadSaves`, `RunFrame` with `RunQuietly`, `LoadState` | 390 |
| `ffi/mod.rs`, `Shim/MarsRtCore.cs` | the C ABI; `MarsCore`'s interfaces over it | 429, 355 |
| `tests/` (nine files), `examples/frames.rs` | the C# unit tests of §5.2.1, item 6; a timed run for profiling | 4,169, 46 |

A module that stage 1a gave a state struct counts that code too. `rsp/mod.rs`, §3's interpreter, was changed in one
respect, described below.

**How the C# maps onto Rust.**

- **One owner, and the devices are methods of the bus.** Each C# device holds a `_bus` and reaches its neighbours
  through it. In Rust, `MemoryBus` owns every device, and a device's behaviour that touches another is a method of
  the bus (`pi_write32`, `si_catch`, `ai_settle`, …). A self-contained device keeps its own methods (`MiInterface`,
  `IsViewer`, the save chips). The CPU is `Cpu::step(&mut self, bus: &mut MemoryBus)`: two disjoint borrows, and no
  shared ownership anywhere.
- **What C# marks `[SkipInState]` is a `Skip<T>`.** It is excluded from the state, as before, and also from the
  machine's equality, so stage 1a's round trips still compare what a state holds. These are the CPU's mode,
  `_recheck`, `_assertedSeen`, `_timerDue` and its two inputs, the bus's next event and write counter, the VI's and
  AI's schedules, the undrained samples, the IS-Viewer's transcript, `SaveChip._saved`, and the cartridge.
- **An exception is a `Result`.** `Exec` is `Result<(), Raised>`, where `Raised` is empty and the fault is written
  into the CPU's own record, as C#'s single `_exception` is. An `Exec` is therefore one byte, and `?` carries a fault
  to the step, which enters it and ticks once, as C#'s `catch` does. `_extraCycles` is left as the raising
  instruction set it, as C#'s is: the catch never clears it.
- **The signal processor runs over the machine's own fields.** §3's interpreter read everything through raw
  pointers into C#'s pinned arrays, and copied its scalars in and out. It now reads through a `Memory` trait.
  `Pinned` implements that trait for the C# twin, with the same pointers, the same copies and the same exports, and
  `MarsNativeRspTests` still pass. `sp::Lent` implements it for MarsRT, lending `sp::Rsp`'s fields and the two
  memories for each call, so nothing is copied. The two instructions that reach the machine, a COP0 move and a
  break, run in `rsp_event`, which is `StepManaged` for those two. `SpInterface.Step` runs through the
  interpreter's `run` between events. That is equivalent to C#'s instruction-by-instruction loop, because neither
  the halt nor the single-step bit can change except at an event.
- **The idle loop is entered from the interpreter.** C# runs `RunIdle` only from a compiled block. The oracle has
  its blocks off, so it never skips a turn. MarsRT has no compiled blocks, so it looks for the loop itself. A taken
  branch to its own address records that address. When the program counter returns there in kernel mode with no
  branch pending, the two words are checked as `BlockShape.Idle` checks them: `0x1000FFFF`, or a J to itself,
  followed by a no-operation. A mapped address qualifies for the first form only, and only inside its page. The
  interrupt check then runs as the step's own would, and a raise is entered as `Step` enters it. `RunIdle` follows
  from there. Two things differ from C#:
  - `StepBlock`'s catch also sets `_lastCount`, which `Step`'s does not. MarsRT follows `Step`, since `Step` is the
    oracle's path.
  - Inside the loop, C#'s `RspRan` steps the processor by `StepOne` when one cycle is owed. `StepOne` does not halt
    after an MTC0 that set the single-step bit, which `SpInterface.Step`, the tick's path, does. The whole-turn path
    runs to the next event and has the same gap. MarsRT steps as a tick steps in both places, which keeps it equal
    to the interpreter.
    - *Demonstrated, not argued.* `A_processor_that_single_steps_itself_under_the_idle_loop_halts_where_the_interpreter_halts_it`
      starts a seven-word program under an idle CPU. The program sets its own single-step bit and then counts in a
      register. The C# interpreter halts it with the count at 0. The C# core with its blocks, and so with `RunIdle`,
      halts it at 1. MarsRT halts it at 0 and matches the interpreter's whole state.
    - *Its reach.* The gap needs a processor that single-steps itself, and no microcode does. The C# is the oracle and
      this stage does not own it, so the one-line fix is left there: `RspRan` should step through
      `SpInterface.Step` whenever the bit could change.
- **The frame is `RunFrame` with nothing armed.** It is `RunQuietly`'s loop to the VI's next field or the cycle cap,
  with the idle test in front of each step. Then come `_lastFrameCycles` and `TotalFrames`, then the scan (§5.4),
  once at the field's end and once after a load. The periodic `SaveSram` belongs to the host, and the shim does it
  every 300 frames, as `MarsCore` does.
- **A load runs what the C# load runs after reading.** `restore_state` parses into a fresh machine, as
  stage 1a's `load_state` does, and keeps the cartridge, the save chip's `_saved` and the options. Then, in C#'s
  order, it runs:
  1. `Written++`;
  2. the save chip and every pak marked dirty;
  3. `Ai.DropUndrained`;
  4. `Vi.Rebase` and `Ai.Rebase`, then `Reschedule`;
  5. the snapshot's pending words replayed through the RDP with no sync raised, as `ReadPending` replays them;
  6. `Cop0Written`, which rebuilds the mode, the check and the timer's due cycle.

  The RDP's own `Refresh` runs in its `read_state` (§5.3). A save settles the two clocks first, as `WriteState`
  does. The raw `load_state` stays for stage 1a's byte round trips. It now rebases the clocks too, so that a save's
  settle adds nothing to a state that was never run.
- **The C ABI** grew to boot a cartridge (`mars_machine_load_rom`, with the save and pak files the host read, or
  `mars_machine_boot` as the corpus boots), run a frame or a number of steps, press a button or set a stick, drain
  audio and read its rate, and read the picture, the counters, the IS-Viewer's transcript, the CPU's fields alone,
  and the save chip and pak for the host to write. The interface version is 3. `MarsRtCore` implements `ICore`,
  `ISnapshotCore`, `IStateFormat`, `IFrameSerial` and `IRepeatedRows` over it, with `MarsCore`'s button map, stick
  reach and save paths. ~~It is not registered in `CoreFactory`.~~ It is, since §5.5, behind a setting.

#### 5.2.1 The evidence

Six oracles. Each can see something the others cannot.

1. **The hardware corpus, line by line.** `The_corpus_reports_line_for_line_what_the_csharp_core_reports` boots
   `n64-systemtest` in both cores, as `MarsCorpusTests` boots it: the interpreter alone, with no saves and no
   frames. It then compares the IS-Viewer's transcripts line by line through the summary. With the RDP merged, all
   1,722 lines are identical, ending in the C# core's own "Failed 46 of 4637 tests". The C# interpreter took 3.0 s
   for its 309,310,184 instructions, and MarsRT 1.8 s.
2. **A synthetic operating system, from boot.** `SyntheticN64System` is a cartridge whose boot code copies an image
   in by PI DMA and enters an idle loop. Its handler serves every interrupt each field:
   - it acknowledges the SP, SI, AI, VI, PI and DP;
   - it runs a joybus state machine of writes and delayed reads, carrying a controller's state, a pak read, and an
     EEPROM read and write;
   - it queues AI buffers and makes an aligned PI transfer, a misaligned odd one and a cartridge-latch round trip;
   - it DMAs an RSP program in, in plain rows and in skipped rows, and starts it. The program does vector
     arithmetic, DMAs back, hands the RDP a list from DMEM over XBUS, takes the semaphore, and breaks with its
     interrupt on;
   - it does single and double arithmetic, conversions, compares and a branch on them;
   - it multiplies and divides, does 64-bit arithmetic, the unaligned family and a linked pair;
   - it goes through the TLB, probes and reads it back, reads Random and Count, reads DMEM, PIF RAM, the RDRAM
     registers, the MI's version and a stub register, and uses the MI's repeat.

   The main program first raises one of each of five exceptions, which the handler steps over. The two cores are
   compared after the load and after each of 240 frames, with the input changing every frame, with and without the
   RSP, and with the picture scanned and compared as well. That is 62,117,781 cycles, and MarsRT passed 30,704,510
   idle turns in them. `count-forever`, a counter in RDRAM with no idle loop, runs 60 frames.
3. **States in both directions.** A state and a snapshot the C# core wrote at frame 37 are loaded by both cores. The
   two then run 163 frames side by side. A state MarsRT wrote is loaded by both and runs 70 frames. Every frame is
   identical. Both cores load, because a load is observable: the first version of the test compared MarsRT, loaded,
   with the C# core that had merely run to frame 37, and they differed at once in the pak's dirty flag, which a C#
   load forces true.
4. **Random programs, compared every sixteen instructions.** `A_random_program_leaves_the_state_the_csharp_interpreter_leaves`
   takes 160 seeds. Each fills 8,192 words with instructions from every class:
   - the special and immediate ALU, with a third of the two-register forms naming one register twice so that equal
     operands are common;
   - all 28 loads and stores, through base registers into the data;
   - branches, mostly forward so that a taken loop does not hold the program in a few words, and jumps;
   - every COP1 format and function, the moves, and branches on the condition;
   - COP0 moves, Status among them, with values that switch the FPU's half and full modes, 64-bit addressing,
     reverse endian and the three modes;
   - the TLB instructions, traps, syscall, break, sync, cache and COP2.

   The registers come from edge values, the floats from every class including both NaNs and subnormals, and the TLB
   and the timer are random. Each vector steps over the faulting instruction. Both interpreters step from one state.
   The CPU's fields and the cycle count are compared every sixteen instructions, and the whole state after 24,000.
   The finer comparison is not a refinement. An SLTU compared with `<=` survived the end-state comparison on all 48
   seeds of the first version, because a wrong register is usually overwritten before the end. Comparing every
   sixteen steps caught it on 21 of 160, and on 14 of 160 once Status writes were added to the mix and took a share
   of the instructions.
5. **Real games: state, picture and sound, every frame.** Super Mario 64, Ocarina of Time and GoldenEye, each from
   power-on and from a gameplay state, with the input driven. The C# oracle is the interpreter with its RDP on the
   emulation thread and its presentation immediate. After every frame, the two cores are compared byte for byte in
   three things: their save states; their pictures, with width, height and row repeat; and the samples each played,
   drained whole, with the rate.

   | Game | From | Frames | Instructions | Result |
   | --- | --- | --- | --- | --- |
   | Super Mario 64 | power-on | 1,500 | 2,809,461,902 | identical |
   | Ocarina of Time | power-on | 1,500 | 2,805,841,811 | identical |
   | GoldenEye | power-on | 1,500 | 2,653,825,385 | identical |
   | Super Mario 64 | its state | 1,500 | 4,495,093,609 to the state's end | identical |
   | Ocarina of Time | its state | 1,500 | 32,986,246,047 to the state's end | identical |
   | GoldenEye | the Dam | 1,500 | 9,129,341,685 to the state's end | identical |

   The 1,500-frame runs compared the state and the picture. Sound was added to the comparison afterwards, and all
   six runs were repeated for 300 frames with it: identical. Instructions are counted from power-on, so a state's
   runs include the instructions its state had already counted.

6. **The C# unit tests, in Rust.** `src/tests/` ports 281 of the 286 tests in twenty `Mars*Tests` files with their
   own values: the CPU's arithmetic, traps, unaligned access, exceptions and interrupts; the FPU's arithmetic and
   register file; the TLB and segment map; the bus, MI and cartridge; DMA; the serial interface; event and VI
   timing; audio; the save chips; the CIC and ROM image. With the crate's own tests and the two written for §5.2.4's survivors, `cargo test` runs 314. The five
   not ported need what MarsRT does not hold: a segment's cached flag, the private save table's titles, a ROM's
   title, and a load from a file. All passed at the first run. The port found one difference, which the machine
   core then fixed. `MemoryBus::new` had left the RSP running and port 0 empty, where the C# constructors leave the
   RSP halted and port 0 present. `Machine::boot` had set both, so no run was affected. The constructors now set
   them.

**The idle skip changes nothing.** The synthetic system's 120 frames leave the same state three ways: stepped
plainly, with the idle loop passed whole, and with the RSP run to its events beside it
(`The_idle_skip_changes_nothing_marsrt_computes`). Since the oracle never skips, every comparison above that ran
with the skip on is also a comparison of the skip. That is every one but the corpus and the random programs, which
step the interpreter as `MarsCorpusTests` does.

#### 5.2.2 Before the RDP was merged

The machine core was proven first against a stub RDP, whose `accept` took every word and ran nothing. The stub's
effect on the comparison was measured, not assumed, and it is recorded here because it separates what the machine
core does from what the RDP does.

- **Exactly, the games parted at the first frame the RDP's work reached the machine.** That was frame 1 of the Dam,
  2 of Super Mario 64's state, 3 of Ocarina of Time's, and 22, 57 and 121 from power-on for Ocarina of Time,
  GoldenEye and Super Mario 64. Two mechanisms were at work. The RDP's own fields never moved. And no full sync was
  ever answered, so the MI's DP interrupt never rose, and the next handler the C# core ran was one MarsRT did not.
- **A framer, a measurement aid, answered the syncs.** It framed the words into commands by `Rdp.Length`, answered
  `SyncFull`, and recorded the RDRAM each primitive could draw into, from the colour and depth images and the
  scissor. With it, and with the RDP's fields, the hidden bits and the recorded memory left out of the comparison,
  five of the six runs held for all 300 frames. Ocarina of Time parted: at frame 6 of its state, in the CPU, and at
  frame 265 from power-on, in two bytes of RDRAM outside the recorded memory. The CPU parting means the machine read
  something the C# RDP had drawn and the stub had not. The two bytes mean either that, or a write the recorded memory
  missed. The comparison could not tell those apart.
- **So the RDP was made to draw aside.** `Rdp.DrawAt(1, …)` points the C# processor at memory of its own. It reads
  textures from the machine's RDRAM as before, and its drawing and hidden bits land where nothing reads them. Against
  that C# core, with the framer on, every run held for all 300 frames: state and picture alike, all six, with only
  the RDP's own fields different. The machine core was therefore exact wherever the RDP's output was invisible to the
  machine. The Ocarina of Time partings came from the game reading what the RDP drew, and not from the machine core.
- **The corpus had the same shape.** Every one of the C# core's 1,722 lines appeared in MarsRT's transcript in
  order. Beside them were four failures, all in the RDP STATUS tests. Without the framer they timed out waiting for
  the pipe to go idle. With it they failed on the auxiliary frame buffer's colour, which only drawing fills. MarsRT's
  count was "Failed 50 of 4637".

Merged with §5.3's RDP, every one of these runs is exact without exclusions, and the framer was deleted.

#### 5.2.3 Speed: the first like-for-like number

§5 asked for MarsRT's interpreter to be measured against the C# interpreter, with compiled code off in both, before
stage 5 is priced again. This is that number.

**The method.** `MarsRtTests.Bench` loads each game's state into a fresh core and times 300 frames flat out, Release,
on this sixteen-processor machine. It does that four ways, and repeats the four in turn for three rounds:
- the C# interpreter: `UseBlocks` and `Rsp.UseBlocks` off;
- the C# core with both on;
- MarsRT's interpreter with the idle skip off;
- MarsRT with it on.

Both cores run their RDP on the emulation thread (`ThreadedRdp` off), and neither scans the picture
(`SkipRendering`). The C# presentation is therefore excluded from both. So is the thread the C# RDP normally has
to itself, which the shipped core uses and this comparison does not. The numbers are milliseconds a frame, and the
range is over the three rounds.

| Game | C# interpreter | MarsRT interpreter | Ratio | MarsRT, idle skip | C#, blocks |
| --- | --- | --- | --- | --- | --- |
| Super Mario 64 | 33.90–34.18 | 19.46–19.53 | 1.74× | 10.80–10.82 | 18.27–18.46 |
| Ocarina of Time | 31.46–31.56 | 18.53–18.58 | 1.70× | 11.16–11.20 | 17.58–17.64 |
| GoldenEye, the Dam | 60.19–62.67 | 39.48–40.32 | 1.53× | 26.09–26.14 | 33.72–34.24 |

**What the table says, and what it does not.**

- **Interpreter against interpreter, MarsRT is 1.5 to 1.75 times faster.** Both do the same work instruction for
  instruction, as §5.2.1 proves, and both pay the same RDP on the same thread. GoldenEye gains least. Its frames are
  the longest, and its RDP's share is the largest, as the next point shows.
- **The RDP is part of the frame here.** The same bench against the stub RDP, before §5.3 was merged, put MarsRT's
  interpreter at 12.70–13.01, 13.17–14.20 and 29.82–32.70 ms, while the C# columns were unchanged. So MarsRT's RDP
  costs about 6.6, 5 and 9 ms a frame in these three states. That is roughly a quarter to a third of MarsRT's
  interpreted frame.
- **The idle skip is not an optimisation of the C# interpreter.** It is what C#'s blocks give the idle loop, and
  MarsRT reaches it without compiling anything. With it, MarsRT is faster than the C# core with its whole
  recompiler: 1.70×, 1.57× and 1.30×. That comparison is of machines doing the same thing by different means, and
  is not like for like.
- **None of this is the recompiler's number.** §5's estimate of 1.4–1.7× on GoldenEye was for a Rust machine *with*
  a recompiler, against the C# one with its own. A Rust interpreter with the idle skip already stands at 1.3× there.
  That narrows what stage 5 must buy, and it is not a measurement of what stage 5 would buy. The estimate is held
  as loosely as §3.4 says estimates here must be.
- **Why MarsRT's interpreter is faster was not measured.** No profiler is installed on this machine. A reason
  offered without one would be an assertion.

#### 5.2.4 Mutants

Thirty-one were made in the Rust, each applied alone. Each was run against `cargo test` and against the WiseMan
comparisons: the random programs (160), the synthetic system and the states (9), the corpus, and the six games at
60 frames. A count is the tests that failed. The corpus is one test, so it reads 1.

| Mutant | Random | Synthetic | Corpus | Games | `cargo test` |
| --- | --- | --- | --- | --- | --- |
| CPU: SLTU compares with `<=` | 14 | — | 1 | 6 | — |
| CPU: the multiplier reads 36 bits of its second operand | 3 | — | 1 | — | 2 |
| CPU: a delay slot's fault returns to the slot, not the branch | 115 | 7 | 1 | 6 | caught |
| CPU: LWR's partial merge drops the register's upper half | 22 | — | 1 | — | 2 |
| CPU: the timer is due at half the cycle | 151 | 7 | 1 | 4 | 1 |
| CPU: the TLB picks the odd page by the page mask, not the bit above it | 52 | 7 | 1 | 2 | caught |
| CPU: a store-conditional lands without the link | 75 | — | 1 | — | — |
| CPU: the interrupt check forgets the line it saw | — | — | — | — | 1 |
| CPU: Random counts down from 30 | 22 | 7 | 1 | 2 | 1 |
| FPU: round to nearest ties away from even | — | — | 1 | 1 | 1 |
| FPU: an unordered compare's invalid flag inverted | 15 | — | 1 | — | — |
| FPU: the flags recorded a bit too high | 58 | 7 | 1 | 6 | 2 |
| FPU: to-integer accepts one past the positive limit | 3 | — | 1 | — | — |
| FPU: half mode ignores the pair on a word read | 9 | — | 1 | — | 2 |
| AI: the page carry lands at once | — | — | — | 4 | 2 |
| VI: the line interrupt tested on odd half lines too | — | — | — | — | 1 |
| SI: a transfer takes half its time | — | 6 | — | 6 | — |
| SI: a read's bytes land at once, not at the transfer's end | — | — | — | — | 2 |
| PI: a trimmed block writes its last byte | — | 7 | 1 | — | 2 |
| SP: the DMA's skip ignored | — | 4 | — | 1 | 1 |
| RSP: a COP0 move names the DP's registers as the SP's | — | 4 | — | 5 | — |
| Joybus: an over-long reply not flagged | — | — | — | — | 1 |
| EEPROM: a write does not wrap inside its block | — | — | — | — | 1 |
| DP: a second start taken before an end | — | — | 1 | — | — |
| Load: a state's pak not marked changed | 160 | 3 | — | 3 | — |
| Idle: the processor's whole turns ignore the parity of the steps they ran | — | — | — | 5 | — |
| Bus: the cartridge latch decays in 112 cycles | 1 | — | — | — | survived, then caught |
| SP: a break raises the interrupt when already broken | — | survived, then 1 | — | — | — |
| MI: a mask pair's clear beats its set | — | — | — | — | survived, then caught |
| Idle: one turn more passed at once | — | — | — | — | **equivalent** |
| FPU: the host path takes sums 29 exponents apart | — | — | — | — | **equivalent** |

**What the pattern says.**

- *Two survivors were argued equivalent, and the argument is the evidence.* The idle loop's bulk takes
  ⌊(stop − cycles − 1) / 2⌋ − 1 whole turns. Without the final −1 it still never passes stop − 1, and the single
  steps after it end on the same cycle with the same count and slot parity. The C#'s −1 is a margin of one turn.
  The host path's limit is the other. With exponents 29 apart, the smaller single lies below 2⁻²⁸ of the larger.
  An addition cannot carry and a subtraction loses at most a bit, so the exact result spans at most 53 bits and the
  double holds it whole. C#'s 28 is a margin of one exponent. A limit much wider would not be equivalent. Once the
  smaller operand falls below half a double ulp of the larger, the double sum rounds to the larger exactly, and the
  inexact flag is lost.
- *Three survived and were caught once a test was written for each,* and each gap is instructive.
  - The latch's decay test measured itself against the constant it tests, so the mutant moved both sides.
    `the_stored_word_lasts_225_cycles` now states the number.
  - No test broke twice without clearing the broke bit between, since the synthetic system restarts with `0x105`.
    `A_second_break_before_the_broke_bit_is_cleared_raises_no_interrupt_in_either_core` compares the two cores
    there.
  - No test wrote both bits of a mask pair at once. `a_mask_write_carrying_both_bits_of_a_device_sets_it` states
    C#'s rule, clear then set.
- *The random programs carry the CPU and the FPU; the rest carry the devices.* Every CPU and FPU mutant but two was
  caught by the random programs. Of the two, the rounding tie is caught by the corpus and the games, and the
  forgotten line only by `cargo test`. That second one is nearly equivalent in state: a check run on every step while
  the line is up is idempotent, and only its cost and one early exit from the idle loop move. The device mutants
  fall to the synthetic system, the games and the ported unit tests in varying mixtures. Seven were caught by one
  oracle alone:
  - the VI's odd half lines, the SI's early landing, the joybus's over-run flag and the EEPROM's wrap, by
    `cargo test` alone;
  - the idle loop's parity, by the games alone;
  - the DP's refused second start, by the corpus alone;
  - the latch's decay, by one random seed of 160, before its own test was written.

  No single oracle would have caught them all.
- *The finer comparison of the random programs mattered.* The SLTU mutant, caught here by 14 of 160 seeds, survived
  the first version of that test on all 48 seeds, which compared only the final state.

#### 5.2.5 What this stage finished for the others, and what it leaves

**What §5.3 and §5.4 left for this stage, done.** The machine calls `accept` for every word `Take` reads, with
RDRAM and its hidden bits lent. It raises the MI's DP interrupt and clears `_running` on a full sync, as `FullSync`
does. It replays a snapshot's words with no sync raised. It calls `scan` at the field's end and after a load. It
keeps one `Scanout` across loads, and rebuilds it only when a state of the other RDRAM size rebuilds the C# machine,
and its raster with it.

**Left out.**

- **The CPU's compiled blocks,** by the stage's brief. The oracle runs without them as well. The idle loop's skip,
  which C# reaches only through them, is kept.
- **The threaded RDP, its page marks and every `WaitFor*`.** MarsRT runs the RDP inline on the emulation thread, as
  C# does with `ThreadedRdp` off, and the marks exist only to make the threaded path wait.
- ~~**The debugger:** breakpoints, coverage, the call stack's observers, watches and the frame log. `RunFrame`'s
  debugging loop is not ported. MarsRT runs `RunQuietly`'s loop, the C# path taken when nothing is armed.~~ *§6.5:
  ported as tables in the Rust loop and a frame that stops with its reasons; the plain frame is still `RunQuietly`'s.*
- **Cheats, `ICoreSettings`, the multiple, antialiasing, the device and deferred presentation.** The shim scans at
  one, immediately. *Cheats and `ICoreSettings` arrived in §5.5, deferred presentation in §5.6, and the multiple,
  antialiasing and the device in §6.4.*
- ~~**`CoreFactory` registration.** Nothing chooses MarsRT yet.~~ *§5.5: a per-console setting chooses it.*

### 5.3 The RDP

*Stage 3's first half, 2026-09-22.* The display processor's rasteriser, ported from the C# `Rdp` partials to Rust and
proven against them byte for byte: RDRAM, hidden RDRAM, texture memory and the processor's serialized state. It is
the single-threaded path at scale one, the one `ThreadedRdp = false` and `RdpWorkers = 1` run. The seam is the one
stage 1 left, `Rdp::accept(&mut self, word, &mut RdpMemory) -> bool`, and its signature did not change.

**What was ported.** Every command the C# `Rdp.Accept` runs, one module per C# file:

| Module | C# file | Lines |
| --- | --- | --- |
| `rdp/mod.rs` | `Rdp.cs`: gathering, command lengths, dispatch; the stage-1a state, unchanged | 539 (425 before) |
| `rdp/modes.rs` | `Rdp.Modes.cs`: the registers, the decoded modes, `Refresh` | 213 |
| `rdp/fill.rs`, `rdp/walker.rs` | `Rdp.Fill.cs`, `Rdp.Walker.cs`: images, scissor, fill, triangles, the edge walk | 172, 205 |
| `rdp/one_cycle.rs`, `rdp/two_cycle.rs` | the combiner, blender, dither and colour image; the pipelined two-cycle mode | 604, 256 |
| `rdp/copy.rs` | `Rdp.Copy.cs` | 231 |
| `rdp/textures.rs`, `rdp/filter.rs`, `rdp/lod.rs` | coordinates, perspective and fetch; four-texel filtering and palettes; level of detail | 276, 219, 218 |
| `rdp/texture_memory.rs` | `Rdp.TextureMemory.cs`: tiles, and the tile, block and palette loads | 232 |
| `rdp/coverage.rs`, `rdp/depth.rs`, `rdp/chroma_key.rs` | coverage; the depth encoding, compare and store, shade and depth correction; the key | 68, 194, 20 |
| `rdp/tables.rs` | the builders of the dither, blend-quotient, divide, five-to-eight, log and coverage-offset tables | 175 |
| `rdp/replay.rs` | a `cargo test` over the recorded game streams (below) | 76 |
| `ffi/rdp.rs` | a test-only C ABI over one processor: new, free, load and save its state, texture memory, accept words | 141 |

**What the port leaves out, and the argument that no serialized byte depends on it.** The processors that share a
list (`Classify`'s steps, `Configure`, `Owns`, the stamps, `TakeScratchFrom`, `CopyStateFrom`, `RecordAliasedRead`,
the drawn-to extents), drawing at a multiple (`DrawAt`, `Widen`, `Rescale` and every `_scaled` branch), the device
path (`Rdp.Gpu.cs` and the change flags it reads), the interface's verifier (`Touch`, `Wrote`, `RunningWord`), the
counters (`Primitives` and its three siblings), `Drew`, and the Debug build's `VerifyModes`. All are `[SkipInState]`.
With one processor `Owns` is always true and `Classify` returns `Ready` for every command; at scale one every product
with `_scale` is the identity, a rectangle's `inclusive` is zero, and `_scaled` selects the branches that were ported.
*The shared list was ported in §5.6, and drawing at a multiple and the device path in §6.4, each still
`[SkipInState]`, so the argument stands for them as ported.*
That is an argument from reading. The evidence is the differential below, which would show a byte the argument missed.

**Four decisions.**

- **The decoded modes are a field, `Rdp::modes`, whose equality always holds.** Two processors are equal when their
  state is, because the modes are a function of `other_modes` and `combine`; `DpInterface` derives equality over the
  processor, and §5.1's whole-machine comparison compares machines that way. `read_state` ends with `refresh()`, as `Default` does,
  so no loaded processor carries stale modes, and `refresh()` is public for a caller that writes either word itself.
  C#'s Debug build checks at every draw that no mode is stale. Rust has no such check: a caller that assigns
  `other_modes` without `refresh()` draws with the old modes.
- **C#'s integer semantics are written out.** C# `int` arithmetic wraps and masks a shift count to five bits. Release
  Rust does the same, but a debug build panics on overflow, so every sum or product whose operands come from a command
  or from memory uses `wrapping_*`, and the two shifts whose counts come from memory (`1 << storedEncoded`, where a
  hidden byte may exceed 3) use `wrapping_shl`. The debug replay below is the evidence that none was missed on the
  three games' lists; it is not evidence for lists no game sent.
- **The reciprocal table is built with integer rounding.** C# builds it with `Math.Round`, which rounds half to even.
  No quotient 2^20/(64+s) for s in 0..64 is a tie, so rounding half up gives the same table, and a test compares the
  two at all 65 segments. With that, every table is a compile-time `static`, and no per-pixel path reads a lazy cell.
- **Texel fetch borrows the processor immutably.** A tile is read by reference and the fetch writes nothing, which the
  C# code already obeyed; the Rust port needs it to hold texture memory and a tile at once.

**The seam the proof uses.** `DpInterface.IWordWatcher`, `[SkipInState]` and null outside tests, is called before and
after each word the interface's inline path gives the processor. Its cost on that path is one null check per word.
The threaded path has no watcher, and nothing here runs threaded.

**Four oracles, each seeing something the others cannot.**

1. **The MarsRdpTests cases, run a second time with MarsRT beside them.** `MarsRdpTests.NewBus()` became virtual, and
   `MarsNativeRdpTwinTests` inherits every case with a twin attached to each bus. Before each word the twin copies the
   bus's RDRAM and hidden RDRAM, so the CPU's writes between lists reach it, and runs the word in Rust; after the C#
   processor has run it, the twin compares RDRAM, hidden RDRAM, texture memory and the two state serializations. The
   Rust state is taken from C# once, at the first word, and never again, so a divergence persists and is seen. **29
   cases, 30 buses, 367 words, 27 full syncs, identical after every word**, and the base class's own assertions pass
   in both classes.
2. **Three games' command streams.** A game is run from its state with the list inline and one processor, and the
   watcher records every word for 120 frames, beside the RDRAM, hidden RDRAM and processor state at the frame
   boundary where recording began. Both processors replay the stream from those bytes and are compared before the
   first word, after every full sync, and at the end:

   | Game | Words | Frames | Full syncs | Comparisons | Result |
   | --- | --- | --- | --- | --- | --- |
   | Super Mario 64 (`sm64.state`) | 978,596 | 120 | 60 | 61 | identical |
   | Ocarina of Time (`oot.state`) | 974,648 | 120 | 40 | 41 | identical |
   | GoldenEye (`ge-dam.state`) | 2,811,622 | 120 | 60 | 61 | identical |

   What the streams exercise, by decoding them as the processor gathers them:

   | | SM64 | OoT | GoldenEye |
   | --- | --- | --- | --- |
   | one-cycle triangles | 50,786 | 5,219 | 60 |
   | two-cycle triangles | 0 | 31,867 | 116,817 |
   | triangles with a depth test or update | 49,226 | 35,859 | 104,526 |
   | texture rectangles | 480, copy mode | 520, one-cycle | 1,680, one-cycle |
   | fill rectangles (fill / one-cycle) | 180 / 0 | 80 / 40 | 120 / 180 |
   | loads (block / tile / palette) | 4,920 / 0 / 0 | 12,693 / 40 / 2,600 | 11,688 / 0 / 9,048 |
   | texel formats drawn | RGBA16, IA16, IA8 | RGBA16, CI8, I4, I8, IA16, IA8, IA4 | CI4, CI8, I4, I8, IA8, RGBA32 |

   Every textured primitive in the three streams is bilinearly filtered. No stream holds a YUV texel, a flipped
   texture rectangle, a keyed combine, or a copy outside SM64. **The replay is not the game's run:** the CPU's and the
   signal processor's writes to RDRAM during the 120 frames are not recorded, so a texture a game loaded after
   recording began is replayed from the bytes that were there before. Both processors see the same inputs, which is
   what a differential needs; the pictures they agree on are the replay's, not the game's.
3. **Random modes over well-formed primitives.** Three thousand seeded lists each set a random colour image (all four
   sizes), scissor (sometimes interlaced), texture image, one to three random tiles each with a tile, block or palette
   load, one or two tile sizes, a random combine, every colour register, the key and conversion constants and the
   primitive depth, and then draw one to four primitives, each under random other modes: fill rectangles, texture
   rectangles of both kinds, and triangles of all eight kinds on the edges of five the other tests draw, their
   attribute blocks kept, perturbed or random. A third of the primitives use a keyed combine, (A − centre) × scale,
   with key widths small enough that the distance lands in the alpha's range. The memories start as noise, hidden
   RDRAM included. **3,000 lists, 150,820 words, 7,586 primitives (2,895 one-cycle, 2,805 two-cycle, 933 copy, 953
   fill), 1,456,851 bytes of colour and depth changed, identical after every list.**
4. **The streams again, in a debug build.** The game differential writes the C# processor's final memories and state
   beside each stream, and `rdp::replay` replays the streams with overflow checks on and compares the end. Identical
   for all three, and no overflow check fired. This oracle sees only the end, and is the one `cargo test` can run
   without .NET; it needs `EMUSEN_MARSRT_RDP` and passes unrun without it.

**Mutants.** Twenty-four were made in the Rust port and each run against all four oracles, the source restored and
rebuilt after each round by a script. The first round had only the twin and the game streams:

| # | Mutant | Twin cases | Game streams | Random lists | Debug replay |
| --- | --- | --- | --- | --- | --- |
| 1 | combiner: colour rounding `+ 0x80` as `+ 0x7F` | 1 | 3 | caught | caught |
| 2 | combiner: input C 13 reads the primitive's level-of-detail fraction | 2 | 1 | caught | caught |
| 3 | combiner: nine-bit sign extension on bit 8 alone | 0 | 0 | caught | survived |
| 4 | blender: the second weight without its +1 | 3 | 3 | caught | caught |
| 5 | blender: the divisor without its +4 | 1 | 3 | caught | caught |
| 6 | blender: dither rounds up at equality | 1 | 3 | caught | caught |
| 7 | two-cycle: the first blend shifted by this pixel's slope, not the last's | 0 | 0 | caught | survived |
| 8 | two-cycle: the texels not exchanged for the second cycle | 1 | 0 | caught | survived |
| 9 | texture fetch: odd rows do not swap words | 4 | 3 | caught | caught |
| 10 | texture fetch: RGBA16's alpha from bit 1 | 2 | 2 | caught | caught |
| 11 | texture fetch: the perspective flag at w < 0, not w ≤ 0 | 0 | 0 | caught | survived |
| 12 | filter: the lower triangle rounds with `0x0F` | 1 | 3 | caught | caught |
| 13 | texture memory: loads do not swap banks on odd rows | 0 | 0 | caught | survived |
| 14 | level of detail: distant only past the top level | 2 | 1 | caught | caught |
| 15 | depth: mode 0 ignores overflow | 1 | 3 | caught | caught |
| 16 | depth: a margin of four slopes, not eight | 0 | 3 | caught | caught |
| 17 | depth: the compressed mantissa keeps one bit fewer | 1 | 3 | caught | caught |
| 18 | depth: correction divides by sixteen, not thirty-two | 1 | 3 | caught | caught |
| 19 | coverage: left samples without the rounding step | 5 | 3 | caught | caught |
| 20 | coverage: the second sub-scanline's samples unshifted | 7 | 3 | caught | caught |
| 21 | coverage: stored coverage wraps instead of clamping | 2 | 1 | caught | caught |
| 22 | walker: edges that touch count as crossed | 5 | 3 | caught | caught |
| 23 | copy: alpha compare keeps one byte of each pair | 0 | 1 | caught | caught |
| 24 | chroma key: the other rounding at nibble seven | 0 | 0 | caught | survived |

**Five survived the first round** (3, 7, 11, 13, 24), and each is behaviour no list in it reached: a combiner input
between 0x100 and 0x17F under a nonzero multiplier, a two-cycle first blend weighing memory alpha where this pixel's
slope and the last's differ, a w whose integer part is exactly zero, and a key distance inside 0 to 255. Mutant 13 is
nearly equivalent: the load's first bank index has bit 1 cleared, so for a load starting on a word that is a multiple
of four the swap only permutes an aligned group of four banks among themselves, and the two versions differ only for
a load starting one word past such a boundary. The random lists were written for these survivors and caught four at
once; the keyed case was added when the chroma key survived them too, since random widths almost never put a distance
in range. **No mutant survives the final suite, and the random lists alone catch all 24.** A mutant that only they
catch is a rule none of the three games' lists exercises, so for those five rules the claim of exactness rests on
random inputs and not on any game's.

**Speed.** Each processor runs a whole stream from the same start, with no comparison timed and the memories copied
outside the clock. C# calls `Rdp.Accept` per word, as the interface does; Rust takes words through the ABI until each
full sync, so the boundary is crossed about once a frame. Release builds, one warm-up round of each, then three rounds
interleaved:

| Stream | C#, ms (three rounds) | Rust, ms | C# / Rust | Per frame, C# → Rust |
| --- | --- | --- | --- | --- |
| SM64 | 1,701.8, 1,703.6, 1,697.1 | 698.3, 697.2, 697.9 | 2.43–2.44 | 14.17 → 5.82 ms |
| OoT | 1,331.0, 1,342.4, 1,333.7 | 550.0, 549.3, 553.7 | 2.41–2.44 | 11.13 → 4.59 ms |
| GoldenEye | 2,220.8, 2,262.5, 2,266.2 | 1,042.7, 1,057.3, 1,058.1 | 2.13–2.14 | 18.75 → 8.77 ms |

The ratio is not a comparison of languages alone. The C# single-threaded path still pays for the split it does not
use: a stamp per row and a `_coverageStamp` fill beside each coverage fill, a worker test per row, the verifier's test
per byte touched, and the primitive counters. How much of the factor those account for was not measured. The result
also differs in kind from §3.4's: there the component was entered once per emulated cycle and lost on the boundary,
and here the boundary is crossed once per full sync, which is the shape §5 chose the whole machine for.

**What surprised.**

- **The port was exact the first time each oracle ran.** That says nothing about how sensitive the oracles are, which
  is why the mutants were run; they are the measurement of the proof, and they changed the proof twice.
- **SM64's recording changes only 10,868 bytes of RDRAM over 120 frames,** against 393,249 for OoT and 213,331 for
  GoldenEye. Its frame buffers cycle and the scene holds still, so each buffer is redrawn with what it held. Every full
  sync is still compared, but the final memory is weak evidence for that game.

**Left out, and not proven.**

- Everything listed under what the port leaves out: the shared list, the multiple, the device, the verifier. *Since
  proven: the shared list and the verifier in §5.6, the multiple and the device in §6.4.*
- `DpInterface` itself, the interface's registers and its ring, which belong to the machine stage.
- YUV texels, flipped texture rectangles and keyed combines are covered by the random lists and by no game's.
- The replay covers lists as they arrive at a processor, not the game's own evolution of memory; the machine stage's
  frame-by-frame comparison against the C# core is what will cover that.

### 5.4 The scan-out

*Stage 4's first part, 2026-09-22.* MarsRT's video interface turns its registers and RDRAM into a picture.
`src/vi/scan.rs` is C#'s `Vi.Scan()` followed by `MarsCore.Compose`, on the immediate path, at a `RenderScale` of
one, with no antialiasing and no compute device. The C# scan-out is the oracle. The claim is that the two agree byte
for byte in everything a scan leaves behind: the frame, its width, height and row repeat, the whole raster with its
coverage bytes, the held lines, the blank flag, and the value `Scan()` returns.

**What was built.**

| Module | The C# it ports | Lines |
| --- | --- | --- |
| `vi/scan.rs` | `Vi.Prepare`, `Measure`, `Borders`, `Hold`, `Fade`, `Expire`, `Darken`, `FrameHeight`; `MarsCore.Compose` | 327 |
| `vi_scan/walker.rs` | `Vi.Walk` and `Vi.Walker`: the fetch, the line window, the two slots and their sample caches, `Filter`, `Dither` | 366 |
| `vi_scan/filters.rs` | `Pixel`, `Pull`, `Runners`, `Step`, `Divot`, `Median`, `Mix`, `Between`, the gamma table and `Root` | 138 |
| `vi_scan/tests.rs` | `MarsViTests`' angrylion pixels, and the filters against formulations written independently of them | 416 |
| `ffi/vi.rs` | nothing: a test-only C ABI, one VI and its scan-out behind a handle | 125 |

`EmuSen.WiseMan/Fixtures/MarsRTViScan.cs` wraps the test ABI, and `MarsRTViTests` and one theory added to
`MarsViDifferentialTests` drive it. The port keeps the C# structure closely, down to the window's sliding rule and the
slots' stamps. Every cache in the walk is a pure function of memory and the registers (`Mars_Video.md` §2.12), so a
different cache would be equally exact; keeping the same one keeps the same cost profile, which makes §5.4.4's
comparison one of languages rather than of algorithms.

**The seam moved in one respect.** The stub was `scan(vi: &Vi, …)`. A scan writes `_held` and `_wasBlank`, and both
are serialized (§5.1). A scan that could not write them would leave MarsRT's state different from the C# state after
the same frame, which is the property the whole port is graded by. The signature is therefore
`scan(vi: &mut Vi, rdram, hidden, out: &mut Scanout) -> bool`. What the C# marks `[SkipInState]` for the scan, the
raster and the walker's caches, lives in `Scanout`, so `vi/mod.rs`'s state layout is untouched. A machine holds one
`Scanout` for its life and keeps it across a load, as the C# raster is kept across one. `Scanout` also carries
`repeat_rows`, C#'s `RepeatRows`, false as Mistress sets it.

**The return value keeps C#'s meaning, and that meaning is narrower than the stub's comment said.** `Scan()` is true
when a walk ran. It does not say whether there is a picture. `Present` composes the raster whatever `Scan()`
returned, since a scan that walks nothing may still have cleared, darkened or faded lines, and a raster nothing
rewrote is still on the screen (`Mars_Video.md` §2.4). `scan` composes on every call too, so `out.frame` is always the
frame to show. The stub's "false when the VI shows nothing" would have invited a caller to keep the previous frame on
false, which the C# never does.

**What is left out:** the deferred path (`Prepare`, `Capture` and `Walk` on the pool, and the repeat test of
`Mars_Video.md` §2.8), the multiple, averaging, the device, and the bands (*all but the bands ported since, in §5.6 and
§6.4*). Only the last bears on exactness, and the
argument that it does not is `Mars_Video.md` §2.12's: the rows are independent and the fetch bug's counter has a
closed form, so one band over every row writes the raster four bands write. The differential below is incidentally a
second test of that argument across two implementations: on the sixteen-processor machine it ran on, the C# oracle
walked every game picture in four bands and MarsRT in one.

#### 5.4.1 The evidence

Four oracles, each able to see something the others cannot.

1. **angrylion's own pixels, without the tools.** `cargo test` runs `MarsViTests`' twenty-seven scans and checks its
   81 constants, colour and coverage byte alike. All 81 matched on the first run.
2. **Formulations written independently of the code.** The runners-up against the FPGA's sort-and-clamp, over all
   1,835,008 configurations of a pixel and six neighbours (`Mars_VideoFilter.md` §5). The median against a sort. The
   square root against the floating-point root at all 16,384 entries. The gamma lookup, the dither's five-bit step and
   the pull's rounding against the FPGA's formulas, stated in the test. The guard columns and the coverage they keep,
   and a scan after the walker's stamps wrap (§5.4.2).
3. **The C# VI, on synthetic registers.** `MarsRTViTests` scans the register sets of `MarsViTests` and of
   `MarsDeferredPresentationTests`' immediate walk, twenty-nine of them, three times over with the hidden bits random,
   all 3 and all 0: 87 scans. `MarsRT_scans_out_what_the_csharp_vi_and_the_reference_scan_out` replays every one of
   the 151 cases of `MarsViDifferentialTests` through MarsRT, and compares each frame with the C# VI's and, since the
   reference is built on this machine, with angrylion's. The painter program of `MarsDeferredPresentationTests` runs
   through the core's own `Present` for twelve frames, rows sent once and repeated, progressive and interlaced.
4. **The C# VI, on real frames.** For each of three games, a C# `MarsCore` runs from power-on for 1,500 frames and
   from a gameplay state for 600, with immediate presentation, the display processor on the emulation thread,
   `RenderScale` 1 and `RepeatRows` false. After each frame the core's own `Present` runs, and MarsRT scans the same
   registers over the same RDRAM.
   - MarsRT carries its own held lines from the start, and they are compared before the scan as well as after, so a
     divergence in them cannot be hidden by copying the C# values across.
   - `Present` discards `Scan()`'s return value, so it is taken from `Prepare` on a scratch bus given the same
     registers and blank flag, which is what `Scan()` returns at one.
   - Every tenth frame the same memory is also scanned by both under eight other control words, keeping every other
     register as the game set it.

**Every comparison was identical.**

| Game | Frames compared | Of which walked | Swept scans | Modes the game set: control / width / x step / y step |
| --- | --- | --- | --- | --- |
| Super Mario 64 (PAL) | 2,100 | 1,953 | 1,680 | `13016` / 320 / `200` / `400` throughout |
| Ocarina of Time (PAL) | 2,100 | 2,078 | 1,680 | `311E` / 320 / `200` / `400` for 20 frames at boot, then `13016` / 320 / `200` / `354` |
| GoldenEye, the Dam | 2,100 | 2,042 | 1,680 | `311E` and `13016` at 320 wide, NTSC for 55 frames and then PAL; `13016` / 440 / `2C0` / `49D` for 1,441 |

A frame that did not walk is one with its origin at zero or a second blank in a row, and it was still composed and
compared. `13016` is a sixteen-bit picture in anti-alias mode 0 with divot and the dither filter; `311E` is mode 1 with
divot and gamma. Both set bit 2, the gamma dither, which neither implementation models (`Mars_VideoPasses.md` §3.1),
so the two agree on a picture that differs from the console's by exactly the dither. Ocarina of Time's vertical step
of `354` is the corpus's one fractional step down, and GoldenEye's 440-wide buffer its one fractional step across.
**No game here uses a thirty-two-bit frame buffer or anti-alias modes 2 and 3.** Those were reached only by the sweep,
which reads the games' sixteen-bit memory as thirty-two-bit pixels and resamples it in both modes: real bytes through
paths no game here takes. That is weaker evidence than a game that takes them, and stronger than none.

#### 5.4.2 Where the C# fails, and MarsRT does not follow it

Three inputs make the C# scan-out misbehave. MarsRT does something defined at each. None is reachable by a game in
the ordinary run of play.

- **The walker's stamps wrap, and one of its two caches is not emptied.** A walker stamps each slot's line with a
  counter that wraps at `int.MaxValue`. The wrap clears `_sampledRow` and not `_plainRow`, so a slot stamped 1 after
  the wrap finds the plain samples of a slot stamped 1 before it.
  - *Demonstrated, not argued.* A throwaway test scanned one row, set the walker's counter to `int.MaxValue − 1` by
    reflection, changed the frame buffer and scanned again: 879 bytes of the raster differed from a fresh VI's scan
    of the new memory. Clearing `_plainRow` at the same point made them identical.
  - *Its reach.* A walker takes one or two stamps a row, so the wrap comes after about ten hours of continuous play
    in one process at the worst (a 480-row interlaced picture reading two new lines a row, in one band) and days at
    the usual. Even then, a stale sample needs an offset no scan has written since the stamp last had that value,
    which in practice means a mode change long before. *Sharpened the same day:* an entry is re-tagged whenever a row
    touches it, so the offset's first touch after the wrap must also fall exactly on the stamp it was left with.
  - MarsRT empties both caches, and `a_scan_after_the_stamp_wraps_reads_no_sample_from_before_it` holds it.
    ~~The C# is not changed here, since it is the oracle and this stage does not own it; the one-line fix is left for
    it.~~ **Fixed in the C# the same day,** with a regression test that constructs the mode change and fails by 2,886
    bytes without the fix (`Mars_Video.md` §2.13). MarsRT already cleared both caches, so the port did not change, and
    the two implementations now agree at the wrap too; the other two inputs below remain C# failures.
- **`Borders` indexes `_held` past its end** once an interlaced picture's active lines exceed 625, which needs a
  vertical sync above 669 half lines. The C# throws out of `RunFrame`. MarsRT stops at the raster's last line.
- **A hidden byte above 3 can carry a channel outside a byte** through the filter's pull, and the C# gamma lookup then
  throws. No writer in Mars stores such a byte (`Mars_VideoFilter.md` §1). MarsRT reads zero there.

MarsRT also treats an RDRAM longer than twice its hidden bits as ending where the hidden bits do. The test ABI
refuses the mismatch outright, and the C# cannot construct it.

#### 5.4.3 Mutants

Thirty-one were made in the Rust, each applied alone and run against `cargo test` and against the WiseMan suites
above, with the games at 200 frames from power-on and 200 from the state. The WiseMan count is the tests that failed,
of 160.

| Mutant | `cargo test` | WiseMan |
| --- | --- | --- |
| Filter: the upper-left neighbour one column nearer | caught | 74, all three games |
| Filter: a neighbour of coverage 6 counted whole | caught | 74 |
| Filter: the fetch bug never folds the row below | caught | 27, of the games Ocarina of Time only |
| Filter: the pull rounds with `+ 3` | survived, then caught | 44 |
| Filter: the low rescan starts at the leader | caught | 75 |
| Filter: the plain maximum instead of the runner-up | caught | 75 |
| Filter: mode 1 reads no coverage | caught | 64 |
| Fetch: the sixteen-bit coverage's halves exchanged | caught | 71 |
| Dither: six bits compared | survived, then caught | 44 |
| Dither: the neighbour above dropped | caught | 52 |
| Divot: the right pixel's coverage ignored | caught | 31 |
| Divot: ties fall to the right first | **equivalent** | survived |
| Divot: the right pixel never the median | caught | 33 |
| Divot: the right neighbour two across | caught | 33 |
| Gamma: the entry one above | survived, then caught | 60 |
| Gamma: before the mix rather than after | caught | 44 |
| Mix: rounds with `+ 15` | caught | 101 |
| Geometry: NTSC's left offset 107 | caught | 151, no game |
| Geometry: the top halved by a shift | survived, then caught | 36, no game |
| Geometry: the left guard seven columns | survived, then caught | 139 |
| Geometry: the right guard eight columns | survived, then caught | 150 |
| Walker: the vertical fraction one bit low | caught | 113 |
| Walker: the fetch bug fires on the repeat itself | caught | 32, of the games Ocarina of Time only |
| Walker: an interlaced field starts on the other line | caught | 154 |
| Walker: a dark column clears its coverage | survived, then caught | 79, no game |
| Walker: the stamp wrap empties only one cache | caught | **survived** |
| Borders: a held line gets three frames | caught | 61 |
| Borders: below the picture the count is watched | **survived** | 2 |
| Borders: a spent line darkened whole | **survived** | 1 |
| Compose: the row repeat always one | caught | 4, the games and the painter only |
| Compose: the coverage byte kept | caught | 7, the games and the painter only |

**What the pattern says.**

- *The tie mutant was predicted equivalent before it ran.* When both ends pass the median's test they are equal, so
  which one is returned cannot change a value. `Mars_VideoPasses.md` §2 says ties fall to the left pixel first, which
  is true of the code and invisible in its output.
- *Seven survived `cargo test` and were caught once a test was written for each.* Three are the pull's rounding, the
  dither's five bits and the gamma index, which the tool-free test's sixteen-bit pixels cannot reach; that is what
  `Mars_VideoFilter.md` §4.3 and `Mars_VideoPasses.md` §4.3 found for the C# a slice ago. The others are the
  half-line truncation, the two guards and the coverage a dark column keeps. Two border rules still survive
  `cargo test`, and are held only by the named differential cases written for them in `Mars_Video.md` §3.2.
- *The games are a weak oracle for geometry and the only one for composition.* Every game here is PAL except
  GoldenEye's first 55 frames. The NTSC mutant passed in that game, so a one-column shift leaves those 55 pictures
  unchanged, as it would a uniform one; that they are the black screen before the logo is likely and was not checked.
  No game's start is an odd half line above the offset. The composition mutants, conversely, cannot be seen by the
  differential, which compares rasters; only a composed frame shows them.
- *The stamp wrap is the one rule only `cargo test` holds,* because no real run reaches two thousand million lines.

#### 5.4.4 Speed

Per scan and composition, from each gameplay state: 300 frames, with the C# `Present` and MarsRT's `scan` timed on
the same frame one after the other and the order swapped every frame, three rounds, Release. Medians in
milliseconds; the range is over the rounds.

| Game | C#, four bands | MarsRT, one thread | C#, one band | MarsRT, beside it |
| --- | --- | --- | --- | --- |
| Super Mario 64 | 1.76–1.80 | 2.85–2.89 | 4.55–4.57 | 2.81–2.82 |
| Ocarina of Time | 2.82–2.88 | 4.48 | 8.46–8.51 | 4.44–4.45 |
| GoldenEye | 1.94–2.02 | 3.16–3.17 | 5.12–5.25 | 3.12–3.15 |

The first pair is the C# as it ships on this sixteen-processor machine, where `Vi.Bands` is four. The second pair
held the C# to one band by running the process with `DOTNET_PROCESSOR_COUNT=4`. **MarsRT is 1.6 to 1.9 times faster
than the C# walk it ports, and 1.6 times slower than four of them in parallel.**

Before these numbers, one change was measured on a Rust-only loop over one dumped frame per game. Forcing the four
per-sample functions inline (`fetch`, `fetched`, `filter`, `undither`) took a fifth off MarsRT's time with identical
rasters, and was kept. Storing the window as four bytes a pixel gained nothing, and inlining `sample` as well lost
time; both were reverted.

**Why this does not settle the stage's question.** §5 moved the whole machine so that the boundary is crossed once a
frame, and the scan-out is a per-frame component that meets no boundary at all, so it compares the Rust with the C# on
equal terms. The result says that a single-threaded Rust walk beats a single-threaded C# walk. It does not say that
the port pays, because the C# does not run its walk single-threaded: it defers it to another thread
(`Mars_Video.md` §2.7) and splits it into bands. A machine that walks for 3 to 4.5 ms a frame on its own thread costs
the frame nothing if the walk overlaps the next one, and all of it if it does not; which of the two MarsRT does is the
machine stage's decision.

**Left for the machine stage.** The call itself: the machine calls `scan` at the field's end, where `RunFrame` calls
`Present`, and after a load, where `LoadState` does, and does not reset the `Scanout` when it loads. The deferred
presentation, the repeat test, the bands, the multiple and the device are not ported. *The deferred presentation and
the repeat test arrived with §5.6, the multiple and the device with §6.4; the bands are still not ported.* *The bands arrived with §6.11, for the deferred walk only; the immediate walk is still one band.*

### 5.5 MarsRT in the frontends (2026-09-22)

MarsRT can now run a game in Mistress. The graphics window's N64 tab has an **Engine** row, *Mars (C#)* or *MarsRT
(Rust)*, stored in `graphics.json` as `Consoles.N64.Engine`. The default is Mars (C#), and nothing chooses MarsRT
unless the player does. `EmuSen_Settings_Reference.md` §4.44 is the player's side of the switch; this section is the
core's. The claim is narrower than §5.2's. It is not that MarsRT is exact, which §5.2 to §5.4 prove, but that what a
frontend does with a Mars core it now does with MarsRT, by the same rules, and that where it cannot, the frontend is
told.

**Where the choice is stored, and why there.** Two places were weighed: `AppSettings` (`appsettings.json`), and a key
beside the core settings in `graphics.json`.

- *The choice is per console.* It picks an N64 implementation; a second implementation of another console would be
  another console's key. `graphics.json`'s `Consoles` is the one store keyed by console, and `AppSettings` would need a
  per-console map invented for one entry.
- *The window already builds such a row.* It builds a dropdown from any `CoreSetting` of kind `Choice`, so the engine
  cost one catalogue entry and a few lines of window code. The preferences' one earlier core picker was removed
  (§4.36 of the settings reference) because it drove nothing and aliased the library filter, and a picker there would
  repeat that shape.
- *The engine is not one of the core's settings.* `ICoreSettings.Set` is applied to a running core between frames.
  The engine decides which core exists, so it is read before any core does, and no core could apply it. It is
  therefore declared by the catalogue (`CoreCatalog.EngineFor`), not listed in `SettingsFor`, which stays "the settings
  the core offers", and `ApplyConsoleSettings` hands a core only the keys it declares. The screen filter (§4.40 of the
  settings reference) set the precedent: a key stored beside the core's, and owned by someone else.
- *The cost of the choice.* A change while a game runs does nothing until the next load, as the Expansion Pak's does
  after the first frame. The row's hint says so.

**The value's path.** `MainWindow.LoadGame` reads the value for the ROM's console (`CoreCatalog.ConsoleForRom`) and
sets `EmulatorSession.Engine` before `LoadRom`. The session passes it to `CoreFactory.Load(…, engine)`, which builds
`MarsRtCore` only when the value is *MarsRT (Rust)* and `MarsRtCore.Available` holds. Every other caller of the factory
(Hotaru, Pharaoh, the probe, the tests) passes no engine and gets the C# Mars, as before.

**The fallback.** When MarsRT is asked for and the library is absent, refused or turned off (`EMUSEN_MARS_NATIVE=0`),
the factory builds the C# Mars, and `CoreBundle.Notice` says why, quoting `MarsNative.Report`: *"MarsRT (Rust) is not
available (turned off by EMUSEN_MARS_NATIVE=0); Mars (C#) is running."* Mistress appends it to the status line after
the game's name and prints it with `[core]`. An engine name the build does not know, such as a hand edit, runs the
default and says that instead.

**What the shim now implements for a frontend.**

- **Settings.** `MarsRtCore.VideoSettings` is Mars's list, key for key, so one tab serves either engine and a value set
  under one is kept for the other. Only `ExpansionPak` acts: before the first frame it rebuilds the machine, and after
  it the change waits for the next load, as Mars's does. The other seven are accepted and checked by their kind: a
  switch must be a boolean, a count is clamped to its range, and a choice must be one of its names, so a hand edit
  falls back to the default in Mistress as it does for Mars. They are then recorded, read back and ignored. Each one's
  hint begins "MarsRT does not implement this yet and ignores it." Those are the hints the core answers. The window
  shows the catalogue's, which are Mars's, so the Engine row's hint carries the qualification there.
  *Since §5.6:* `ThreadedRdp`, `RdpWorkers`, `DeferredPresentation` and `SkipRepeatedScans` act as well, and three
  settings remain ignored. *Since §6.4, 2026-09-23, none does: `RenderScale`, `Antialiasing` and `Gpu` act, their
  hints are Mars's, and `IgnoredHint` prefixes nothing.*
- **Cheats, in `MarsCore.RunFrame`'s order.** They are applied after the frame, before the periodic save and before
  the picture, and only while Status.IE is set (`Mars_Cheats.md` §5.1). That order needed the frame split in two:
  `mars_machine_advance` runs the machine to the field's end, and `mars_machine_present` scans. `ApplyCheats`, the
  paused frontend's Apply, is not held, as Mars's is not. Reads and writes go a byte at a time through
  `mars_machine_read_memory` and `mars_machine_write_memory` by space number: RDRAM, DMEM, IMEM and PIF RAM. Past a
  memory's end a read is zero and a write is dropped, as `ReadForCheat` and `WriteForCheat` have it. A write that lands
  counts as one the idle loop would see (`Written`), as C#'s does. The gate reads `mars_machine_cop0(12)`.
- **ROM patches are not applied.** C# consults `CheatRomPatcher` on every cartridge read. MarsRT would need either a
  call from Rust into C# on that path, which §3.2 rules out, or the patch list held beside the cartridge reads in
  `memory/bus_access.rs`, which this stage did not own. No N64 code format produces a ROM patch (the explicit codec
  slot is empty, `Mars_Cheats.md` §1). So only a patch added to the registry by hand is affected, and on MarsRT it is
  silently not applied.
- **Battery saves** are as §5.2 left them: the `.srm` and `.mpk` are read at `LoadRom` through
  `SaveLibrary.SramPathFor` and `AtomicFile`, `CoreOptions.BatteryRamDisabled` is latched there, and a changed chip or
  pak is written by `SaveSram` and on every 300th frame. What is new is the evidence below.
- **The frame handed out is a copy.** `GetFrameBufferRgba` returns a new array each call. The shim refills one buffer
  per picture, and Mistress hands the array to its render thread, so a live array would be written by the next frame
  while it is drawn. MarsCore hands out its live buffer and has the same exposure on its immediate path; that was not
  changed here. The cost is one allocation of the frame's size per picture shown. *Retired 2026-09-23 by §6.13: the frame
  handed out is still a copy, but into an array lent rather than given. A caller that holds it keeps it unwritten, as
  before; one that hands it back through `IFrameBufferPool` gets it lent again, and Mistress does, so the allocation is
  gone from its loop. MarsCore is unchanged and keeps the exposure described here.*
- **The debugger.** `MarsRtDebugTarget` gives the console Mars's memory names: RDRAM, DMEM, IMEM, PIFRAM, ROM (read
  only), and CPU, the processor's kernel view through the direct segments and the TLB, never faulting. It gives the
  CPU's, the RSP's and the VI's registers, disassembly by Mars's two disassemblers, the summary and the cheats. Both
  processors are published with `CanHalt = false`, so `bp` and `step` refuse with "cannot be halted by this core"
  rather than arming a registry nothing consults. Watches, the frame log, coverage, the call stack, labels and the
  dashboard's audio peek are absent or inert. The memory exports read memories only, never registers, so a read
  disturbs nothing. *Since §6.5:* the processor can halt, and every registry but the audio peek is live; the target
  hands out the core's own, as `MarsDebugTarget` does.
- **One thread.** The shim must be driven from one thread. A state load replaces the Rust machine, and a read racing
  it would read freed memory, where the C# Mars's arrays are merely racy. Every current caller is on the emulation
  thread: the frame loop, and the console, whose commands Mistress drains there.
- **Rewind stays off for MarsRT.** §4.21b of the settings reference turned it off for Mars because a snapshot taken
  with several rasteriser workers hung. MarsRT has no workers, so that reason does not reach it today. But §5.6 gives
  it a threaded RDP, and the rule is kept per console until a snapshot is proven there, rather than argued per engine.
  *Since §5.6:* a snapshot with the workers running is proven headlessly in every frame of six games (§5.6.7). The
  rule stands until one is proven in play.
- **States** are unchanged: MarsRT reads and writes Mars's format (§5.1), version 1 through `IStateFormat`. A state
  of the other memory size rebuilds the machine in Rust, and the shim's `ExpansionPak` now follows it, as MarsCore's
  `_expansionPak` does. A state's record gives the same core name for both engines, so it does not say which wrote it.

**The C ABI, interface 4.** The new exports are `mars_machine_advance`, `_present`, `_memory_size`, `_read_memory`,
`_write_memory`, `_cop0`, `_cpu_registers`, `_rsp_registers` and `_vi_registers`. The spaces are numbered 0 RDRAM,
1 DMEM, 2 IMEM, 3 PIF RAM, 4 ROM and 5 CPU; status −10 names no space, and −11 refuses a write to ROM.
`mars_machine_run_frame` stays for the test ABI.

**What a threaded RDP will owe this section.** C#'s cheat and debugger accesses call `Dp.WaitFor` before touching
RDRAM, because the threaded RDP may still be drawing into it. MarsRT runs its RDP inline, so there is nothing to wait
for. `Core::read_memory` and `Core::write_memory` in `ffi/mod.rs` are the one place cheats and the debugger reach
RDRAM, and §5.6's wait belongs there. Until it is added, a threaded MarsRT's cheats would race its RDP.
*Paid in §5.6.4:* both functions now wait for every worker, and `site_9_the_hosts_read_and_write_wait_for_the_drain`
holds them to it.

#### 5.5.1 The evidence

The tests run the real code paths on WiseMan's headless platform. Where a C# counterpart exists, it is the oracle.

1. **Mistress, a real window** (`MarsRtEngineTests`). A `SyntheticN64System` cartridge with a 4 Kbit EEPROM, which
   writes EEPROM block 2 every other field, sits in a sandboxed library.
   - The N64 tab has the Engine row with both names and *Mars (C#)* selected, and no other console has one. Choosing
     MarsRT stores it.
   - The game runs on Mars until MarsRT is chosen, and on MarsRT after. Its frames advance, the frame serial moves,
     and the rewind buffer stays empty.
   - A state saved by the hotkey on MarsRT loads into a C# Mars. A load by the hotkey returns MarsRT to within the
     sixty frames the test allows.
   - **The battery save.** A marker is placed in block 5 of the `.srm`, which the cartridge never writes. After forty
     frames and a return to the library, the file holds the marker, so the start read it, and new bytes in block 2, so
     the stop wrote them. A C# Mars loads the same bytes.
   - A cheat imported from a `.cht` file into the window's own registry reaches RDRAM.
   - **The fallback, in a process of its own.** `MarsNative` loads once per process, so the variable cannot be set
     after the fact. The test starts `dotnet test` on one test of the same assembly with `EMUSEN_MARS_NATIVE=0`. That
     child opens a window, asks for MarsRT, and records what it got: `MarsCore`, the report "turned off by
     EMUSEN_MARS_NATIVE=0", and the notice, which was also in the status line. The parent reads the record, so a child
     that ran nothing fails.
2. **The core** (`MarsRtFrontendTests`, `MarsRtSaveFileTests`).
   - The factory builds MarsRT for `.z64`, `.n64` and `.v64` only when asked, with the GameShark codec and the new
     target. An unknown engine name runs Mars, and the notice names it.
   - The settings: Mars's keys and each hint; ignored values checked and read back; unknown keys refused; the
     Expansion Pak rebuilding before the first frame and not after; and a 4 MB state carrying the Pak into an 8 MB
     machine.
   - Cheats: a `.cht` import landing at the frame's end; interrupts off holding a cheat in both engines while Apply
     does not; every writable space reached, and nothing past an end or in ROM.
   - **With a cheat on, MarsRT and Mars agree for sixty frames.** Both run the synthetic system with a code that writes
     two pixels inside the visible picture and a word the handler counts in. Their save states and their pictures are
     compared after every frame.
   - The frame is a copy the next frame does not touch.
   - `disasm`, `regs` and `mem` work, `bp` and `step` refuse, and the CPU space follows the TLB the game set, compared
     at seven addresses with Mars's own target.
   - Battery: a save read and written back for either engine to read; nothing written when nothing changed; a saved
     chip written again only after it changes again; the three-hundredth frame's save; and `batteryRamDisabled`
     reading and writing nothing.
3. **Super Mario 64's Unlimited Lives.** This uses a scratch copy of the European cartridge, the libretro database's
   `.cht` for it, and its code `803094DD 0064`, imported through `CheatImport.FromChtFile` with the bundle's codec.
   Both engines are started from `sm64.state` with the cheat on and run 120 frames. RDRAM `0x3094DD` is `0x64`, and
   the two engines' 12.7 MB states are identical. It runs only with `EMUSEN_MARSRT_STATES` and `EMUSEN_MARSRT_SM64_CHT`
   set, and passes unrun without them.
4. **`cargo test`:** five tests on the memory exports, 319 in all.

The blast radius was run as well: every `MarsRt*`, `MarsNative*`, `MarsCoreSettings`, `MarsDebug`, `MarsSaveFile`,
`MarsCheat` and `CoreFactoryCheatWiring` class, the graphics window's two classes, and the save-state thread tests.
All 569 passed after WiseMan's regrouping was merged, and the whole WiseMan suite, 6,554 tests, passed once at the end.

#### 5.5.2 Mutants

Thirteen were made, each applied alone and run against the three new classes, and the Rust ones against `cargo test`
as well.

| Mutant | Caught by |
| --- | --- |
| The gate removed: cheats applied with interrupts off | the interrupts-off test |
| The gate inverted | four: both `.cht` imports, the interrupts-off test, the sixty-frame comparison |
| Cheats applied after the picture rather than before | the sixty-frame comparison alone, by its picture |
| The `.srm` not handed to the machine at load | the core's and Mistress's battery tests |
| A changed chip not written | four battery tests |
| The chip not marked saved after writing | the written-again test, **which was written for it** |
| No save on the three-hundredth frame | the three-hundredth-frame test |
| The battery switch ignored | the `batteryRamDisabled` test |
| The live frame buffer handed out | the copy test |
| The factory building MarsRT without asking whether it is there | the fallback, in its child process |
| Rewind left on for MarsRT | Mistress's run test, by the buffer's depth |
| A landed write not counted as one the idle loop sees (Rust) | `cargo test` alone; **equivalent between frames** |
| The CPU space ignoring the TLB (Rust) | **survived**, then caught once the TLB comparison was written |

**What the pattern says.**
- *Two had no test before they were made.* No test asked for a save to be written twice, and none read a mapped
  address through the CPU space, so neither the saved mark nor the TLB was held. A test was written for each; the
  first was written before its mutant ran, the second after the mutant survived.
- *One is equivalent at the level a host can see.* A cheat writes between frames, when no idle run is in progress to
  compare the counter against, so leaving `Written` alone changes nothing a frame computes. The Rust test pins it
  because C# counts it, and because a threaded RDP (§5.6) may make a write outside a frame matter.
- *The order mutant is caught by one oracle only,* the comparison of pictures. A cheat that writes outside the frame
  buffer leaves the same state whichever side of the scan it runs.

#### 5.5.3 The publish, and a defect predicted and refuted

**The prediction.** `EmuSen.csproj` runs cargo only when `RuntimeIdentifier` is empty or equals
`NETCoreSdkRuntimeIdentifier`, and this machine's Fedora-built SDK names the host `fedora.44-x64`. From reading the
condition, a `-r linux-x64` publish looked as though it would get no library, and `out/linux-x64/Mistress` has none.

**The measurement refuted it.** The requested command (`dotnet publish EmuSen.Mistress -c Release -r linux-x64
--self-contained true -p:DebugType=none -p:ErrorOnDuplicatePublishOutputFiles=false`), run on the unmodified tree into
a scratch folder, put `libmarsrt.so` in `lib/EmuSen/` with every export. The runtime identifier given to the publish
is not passed on to the referenced library project, which builds for no identifier, and so for the host. The copy in
`out/` predates the library. The condition was not changed.

**The load, headless.** A throwaway console program, run by the system `dotnet`, set its base directory to the
published `lib/EmuSen/` and loaded that folder's own `EmuSen.dll`. It wrote `Consoles.N64.Engine = MarsRT (Rust)` into a
scratch `graphics.json`, read the value back and set it on an `EmulatorSession`, as `LoadGame` does, and loaded a
scratch copy of Super Mario 64.

| Run | Library | Core | Result |
| --- | --- | --- | --- |
| As published | the published `libmarsrt.so`, interface 4 | `MarsRtCore` | 300 frames and a 640×576 picture |
| `EMUSEN_MARS_NATIVE=0` | turned off | `MarsCore` | the notice, "turned off by EMUSEN_MARS_NATIVE=0" |
| The library moved aside | "not found beside the assemblies" | `MarsCore` | the notice, quoting that |

This checks the published core and the published factory, not the published window. The window's own path is
§5.5.1's first group, on the build tree.

#### 5.5.4 What is left

- ROM-patch cheats, as argued above.
- ~~The debugger's breakpoints, stepping, watches, coverage, call stack and labels, all of which need hooks inside the
  Rust loop. The deep inspection §5 planned through the state transfer was not built.~~ *Done in §6.5, as hooks inside
  the Rust loop; the state transfer stays the inspector's read path.*
- The seven settings MarsRT ignores. Each arrives with the stage that implements it: the threaded RDP and deferred
  presentation with §5.6, and the multiple, antialiasing and the device later. *Four arrived with §5.6, and the last
  three with §6.4.*
- The wait a threaded RDP will need in `Core::read_memory` and `Core::write_memory`. *Done in §5.6.4.*
- An engine choice in Hotaru and Pharaoh, which still build the C# Mars.
- `IFrameProfiler` phases, and the dashboard's audio peek.

### 5.6 Threads (2026-09-22)

MarsRT now takes work off the emulation thread the way the C# core does. The display processor's list runs on a
thread of its own, or is shared by several processors, each shading its own rows. The scan-out's walk runs on another
thread while the next frame runs. The claim is that neither changes anything the machine computes:
- With the display processor threaded, the state, the picture and the sound after every frame are the unthreaded
  machine's.
- With presentation deferred, the picture shown after frame *n* + 1 is the immediate picture of frame *n*.

The design being ported is the C# core's (`Mars_Rdp.md` §2.6 to §2.8, `Mars_Video.md` §2.7 and §2.8). MarsRT on one
thread is the oracle. The C# core threaded is a second oracle wherever it is exact, and §5.6.2, §5.6.5 and §5.6.6
record four places where it is not. *All four were fixed in C# on 2026-09-23, each with its test turned round
(`Mars_Rdp.md` §2.9, `Mars_Video.md` §2.8).*

The switches are `Machine::set_threaded_rdp`, `set_rdp_workers` and `set_deferred`, all off by default. The path
§5.2 to §5.4 proved therefore stays the default until this one has been proven in play. The shim now honours Mars's
`ThreadedRdp`, `RdpWorkers`, `DeferredPresentation` and `SkipRepeatedScans` keys, with MarsRT's defaults of off, one,
off and on. Their hints no longer say "ignored", and the settings still ignored are three: the multiple, antialiasing
and the device. *Retired by §6.4 (2026-09-23): the three are honoured, and no setting is ignored.* *Since 2026-09-22 the shim's defaults are Mars's own — on, one per three cores, on — by §6.2; the
machine in the library still boots with them off.* The interface version is 5, which adds `mars_machine_set_threads` and `mars_threads_counters`
(`src/ffi/threads.rs`).

| Module | The C# it ports | Lines |
| --- | --- | --- |
| `memory/ram.rs` | nothing in C#: RDRAM and the processor as raw allocations (§5.6.1) | 297 |
| `memory/dp_threads.rs` | `DpInterface`'s threaded half: the ring, the shadow, the marks, ranges and boxes, the waits, `Join`, `Hold`/`Pause`/`Resume`, the workers, the barrier and the verifier | 1,478 |
| `memory/dp.rs` | `Take` onto the ring, `WritePending`, and the cheat's two accesses | 391 |
| `rdp/split.rs` | `Classify`, `Serialised`, `Owns`, the stamps, `RecordAliasedRead`, `TakeScratchFrom` and `CopyStateFrom` | 303 |
| `vi/scan.rs`, `vi/scan/presenter.rs` | `Prepare`, `Capture`, the walk apart, `Repeated`, `PresentDeferred` and `JoinPresentation` | 537, 130 |
| the wait sites | a line or two each in `bus_access`, `interp`, `idle`, `si`, `ai`, `sp` and `ffi` | — |
| `tests/threads.rs`, `tests/sites.rs`, `tests/games.rs` | `MarsThreadedRdpTests`, and more (§5.6.7) | 751, 260, 190 |

The crate's diff against 7ea79b1 is 35 files, 4,447 lines in and 167 out.

#### 5.6.1 Shared memory under Rust's rules

In the C# design, two threads share RDRAM, its hidden bits and the processor, and the page marks decide who may touch
which byte when. Rust's rule for data races is stricter than "the bytes come out right".
- Two accesses to one byte from two threads, one of them a write, must be ordered by happens-before. Otherwise the
  program's behaviour is undefined, whatever value the read would have found.
- A reference counts as an access to every byte it covers, in the models Miri checks, because forming one retags those
  bytes.

So the port could not keep RDRAM as a `Vec<u8>` reached through `&mut [u8]` while a drain writes into it. Three types
change:

- **`Ram`** holds RDRAM and its hidden bits as one raw allocation each. Indexing a byte or a range forms a reference to
  those bytes alone, never to the whole memory. The CPU's direct loads, stores and fetch and the bus's word access go
  through its `read`, `write`, `be32` and `put_be32`, which touch only the bytes named. `Deref` to the whole slice
  remains for a machine whose drain is joined: a state, a load, the unthreaded scan and the tests.
- **`RdpMemory`** is now raw pointers with accessors. A worker holding it for a whole list therefore asserts nothing
  about the bytes the machine touches meanwhile. Every byte the processor reads or writes passes one accessor, which is
  also where the verifier checks it (§5.6.2).
- **The processor lives in a `Detached` box** reached through its raw pointer. The machine's `&mut` of its own fields
  therefore never covers the processor a worker is running.

**The invariant,** as `dp_threads.rs` states it:
- Between a word's publish and the join, the workers own the processors and may touch the bytes the word's marks name.
- The machine's thread touches such a byte only after an Acquire of every worker's count at or past the last word that
  touches it.
- Every word goes into the ring before the count that publishes it is stored with Release. So a worker's Acquire of
  that count sees the word and everything the machine wrote before it.
- The ring, the marks, the ranges and the boxes are atomics. They are Relaxed where one thread writes them, with
  Release and Acquire fences where the verifier reads what the machine is writing.

#### 5.6.2 The interface on a thread

The port follows C#'s structure closely.
- `Take` reads each word as the unthreaded path does, and `publish` puts it in a ring of 65,536.
- The shadow gathers the stream as the processor does, and marks the pages each command can reach:
  - a draw's own rows (`MarkDraw`, §2.6.2);
  - a load's bytes, with the sixteen-byte window it reads through;
  - the batch's images, marked idle until the batch ends, when they are downgraded to its count.
- Ranges (8,192), idle ranges (256) and boxes (16,384) record the bytes behind the marks.
- A waiter tests its page's mark, which is one load and a compare on every fast path. It is freed as a bystander by the
  ranges, or narrowed by the boxes when the read is small, and otherwise waits for the words that matter.
- A full sync is answered when its word is published. The MI's interrupt and the status word therefore change at the
  cycle at which they change unthreaded.

MarsRT departs from C# in the following places.

- **A wait tests every byte of the access,** where C# tests its first byte (`Mars_Rdp.md` §2.6.3 records that
  suspicion). A doubleword load whose second half falls in a pending range now waits. This is stricter than C#, so it
  cannot cost exactness.
- **A range read keeps the coarse wait.** C# narrows every read by the boxes that hold its first eight bytes,
  including a capture of several pages. It therefore frees a range read whose first bytes no pending draw holds, even
  when pending draws hold the rest.
  - *In C#:* `The_csharp_interface_lets_a_range_read_pass_the_draws_that_hold_all_but_its_first_bytes` pauses the
    thread with a fill of rows 8 and 9 pending. C#'s `WaitForReadRange` over the page at row 6 then returns at once
    (narrowed 1, freed 1), and the bytes it lets through are the undrawn ones.
  - *In MarsRT:* only reads of eight bytes or fewer are narrowed.
    `a_range_read_whose_first_bytes_no_draw_holds_still_waits_for_the_draws_that_hold_the_rest` holds such a read
    waiting until the drain is released.
  - *Why the games miss it:* every VI capture, every SP DMA into the RSP and every SI transfer to the PIF is a range
    read of more than eight bytes, so the defect sits on the C# core's most-used wait. No game comparison has caught
    it, because the drain has usually finished before the capture reads.
  - *Fixed in C# on 2026-09-23, commit 7fbf4b1, the test turned round:* the C# now narrows only a read inside one
    aligned doubleword, and `MarsThreadedRdpTests.A_range_read_whose_first_bytes_no_draw_holds_still_waits_for_the_draws_that_hold_the_rest`
    holds the read waiting (`Mars_Rdp.md` §2.9.2).
- **The workers are threads of MarsRT's own.** C# queues a single drain on the pool and gives several workers threads
  of their own. A MarsRT worker parks when it has nothing to do.
  - The publish wakes it after a Relaxed test of its sleeping flag.
  - A batch's end and every wait wake it surely: a SeqCst fence pairs with the one the worker makes before it parks,
    so one of the two sees the other.
- **The verifier checks every byte,** where C# checks the first byte of each access. It is on in debug builds and with
  `EMUSEN_MARSRT_VERIFY_RDP=1`.
  - The hidden bits are checked as the second byte of their pair, which every writer of them writes.
  - A box is found by halving the search, since the boxes' last words are unique and grow with the index.
  - While one draw touches neighbouring bytes, the last covering range and box are remembered, and the memory accepts
    only what the full scan would have accepted.
  - Without that memory the verifier cost 2.5 s a frame in the Dam; with it the cost is 0.44 s.

#### 5.6.3 The wait sites

| Site | C# | MarsRT |
| --- | --- | --- |
| 0, bus read; 10, while taking | `MemoryBus.Read32` | `read32`: every byte, halfword and doubleword on the bus, and every device's DMA |
| 1, bus write | `MemoryBus.Write32` | `write32` |
| 2, 3, 4: load, store, fetch | `Cpu.Load`, `Cpu.Store`, `FetchInstruction` | `interp.rs`'s direct paths, over the access's size |
| 5, block | `Cpu.StepBlock`, before a block is shaped | the idle test's two words (`idle.rs`), since MarsRT has no blocks |
| 6, SI | `SiInterface.Land`, `Transfer` | the landing and the transfer to the PIF |
| 7, AI | `AiInterface.Play` | the AI's read of each sample word |
| 8, VI | `Vi.Scan`, `Vi.Capture` | `present_now`, `present_deferred` |
| 9, cheat | `ReadForCheat`, `WriteForCheat` | `cheat_read8`, `cheat_write8`, and the host's access (§5.6.4) |
| 11, SP DMA | `SpInterface.Transfer`, per row | the SP's transfer, per row |

`tests/sites.rs` has fifteen tests, at least one for each site, and the host's access is among them. Each holds the
drain with a draw or a load pending, makes the site's access, and releases the drain from another thread after
300 ms. The access must have waited for the whole hold, and must see what the list run at once leaves. The tests exist because the game comparisons did not catch a
dropped wait (§5.6.9). A read in the moment before the drain writes its bytes is rare, so a game leaves the catch to
timing.

#### 5.6.4 Holding the drain: states, snapshots, the switches and the host

- **A state joins, and a snapshot holds.** `write_state` waits for every word, as C#'s `Write` does. A snapshot, for rewind, holds the workers at a word boundary instead. It writes the words not yet
  run into the tail C#'s `WritePending` defines, and resumes them. A load replays those words on the machine's thread
  before a drain starts again, as `ReadState` does. The tail's layout is C#'s. But a snapshot with words pending has
  been loaded only by MarsRT; loading one into the C# core was not tested.
- **Holds nest.** `save_state_vec` sizes a snapshot and then writes it, so it holds and resumes twice. At first the
  inner resume released an outer hold, and a snapshot's tail then held 30 of 36 pending words. A hold is now a count,
  and only the last resume lets the workers go.
- **A pause is a numbered request,** and a worker answers with the number at the word where it stands. MarsRT does
  this so that a stale answer cannot be taken for a new request. Whether C#'s flags can be confused that way was not
  tested.
- **The switches act between frames.** A drain starts at the next frame, or at once when the setter is called, once no
  words from a load are pending. It takes the shadow from the processor. Turning it off joins it and hands the
  processor back. A change of worker count restarts the drain.
- **The host's memory access joins.** `Core::read_memory` and `write_memory`, through which the cheats and the
  debugger go, now wait for every worker first. C# waits per byte at site 9. These two functions reach the whole
  memory as one slice, so a narrower wait would still leave the reference that §5.6.1 forbids. §5.5 recorded this
  wait as owed; it is now paid. The shim's `ApplyCheats` is unchanged, since every one of its bytes passes these
  functions.

#### 5.6.5 Deferred presentation

`present_deferred` proceeds in four steps.
1. It joins the walk still out.
2. It runs `Prepare` on the emulation thread, since the held lines and the blank flag are state.
3. It captures the lines the walk can reach, after waiting for them at site 8.
4. It hands the walk and the composition to a presenter thread.

The job is one boxed value, moved between the two threads under a state word that is stored with Release and loaded
with Acquire. So nothing is shared while the walk runs. The shown frame is swapped at the next present, at
`join_presentation`, or at a load, and the frame serial moves only then. A load presents at once and forgets the
capture, as C#'s does. With the display processor threaded, the immediate path also waits at site 8 and walks only
the bytes it waited for, so the walk cannot read a byte the drain is still writing.

**One C# rule fails here, and MarsRT does not follow it.**
- *The rule.* C# skips a deferred scan whose geometry and bytes repeat the last walk's, and keeps the picture on show.
- *Why it fails.* `Prepare` has already advanced the borders. A held line that expires in that scan darkens the
  raster, so the picture on show is no longer the raster's.
- *In C#:* `The_csharp_deferred_path_keeps_a_stale_picture_when_a_repeat_follows_an_expired_line` shrinks a picture
  and then repeats it. C# skips five scans, and for two frames it shows a picture that differs from the immediate
  picture of the frame before.
- *In MarsRT:* the scan-out notes whether a border or a blank has changed the raster since the last walk, and walks a
  repeat when one has. `a_deferred_scan_repeating_after_a_held_line_expired_shows_the_darkened_picture` holds this.
- *Fixed in C# on 2026-09-23, commit 419583a, the test turned round:* the C# follows MarsRT's rule, a darkening counted
  only where it changed a byte, and `MarsDeferredPresentationTests.A_deferred_repeat_after_a_held_line_expired_shows_the_darkened_picture`
  finds no stale picture (`Mars_Video.md` §2.8).

#### 5.6.6 The list shared by several processors

`set_rdp_workers(n)` runs the list on *n* processors. Each shades the rows whose number modulo *n* is its index. The
port is C#'s §2.8:
- `Classify`'s steps, with the serialised primitives drawn by the leader alone;
- the image changes and the hazard loads behind a barrier;
- the stamps and `RecordAliasedRead`;
- the leader's assembly of each scratch field from whichever processor wrote it last in raster order.

The pause point is the furthest word any worker has reached. Three of C#'s rules fail, and MarsRT does not follow them.
Each failure was found by a test.

- **A barrier's waiter can deadlock a pause.**
  - *How.* Suppose a pause is asked while some workers wait at a barrier for word *k* + 1 and the rest have not
    reached it. The rest stand at *k*, short of the barrier. The waiters are inside a command and cannot stand.
  - *Why C#'s argument misses it.* C# argues that "a processor short of the point can never be waited for at a
    barrier by one past it". That holds for workers past the point, but these waiters are at it.
  - *Found by.* MarsRT's test deadlocked on four workers, and the stacks showed two workers at the barrier and two
    standing.
  - *The fix in MarsRT.* A waiter at a barrier now raises the pause point to its own word. The standing workers then
    run on through the barrier, and all of them stand past it.
  - *In C#:* `The_csharp_workers_deadlock_when_a_pause_finds_some_at_a_barrier_and_the_rest_short_of_it` finds the C#
    pause never answered at its second attempt. The test is gated behind `EMUSEN_MARS_DEADLOCK_PROBE=1`, because the
    hang leaves threads spinning.
  - *A candidate, not a proof.* `Mars_Rdp.md` §2.8 records a freeze met in Super Mario 64 whose cause was inferred and
    not proven. This deadlock could cause it, and it has not been shown to.
  - *Fixed in C# on 2026-09-23, commit a72533c, the test turned round:* MarsRT's rule, a waiter raising the point to its
    own word, now in `DpInterface`; `MarsThreadedRdpTests.A_pause_that_finds_some_processors_at_a_barrier_and_the_rest_short_of_it_is_answered`
    and a stress of pauses and snapshots under a watchdog hang on the unfixed code and pass on the fixed (`Mars_Rdp.md`
    §2.9.1). The freeze is still not proven to have been this.
- **The level-of-detail fraction is a carry that C#'s inventory missed.**
  - *How.* A one-cycle row starts from the processor's own last fraction. A primitive that computes no fraction writes
    that value at every pixel, with its row's stamp. So each processor carries its own last value, and a primitive
    drawn alone then assembles a stale one.
  - *What it reaches.* It never reaches a picture, since a combiner that reads the fraction computes it. It is
    serialised state all the same.
  - *In C#:* `The_csharp_split_assembles_a_stale_level_of_detail_fraction_from_rows_that_computed_none` finds the C#
    state wrong in `_lodFraction` alone, for 8 of 16 seeds, with the memory right. Against MarsRT's split, the C#
    split parts at frame 6 of the Dam, again in `_lodFraction` alone.
  - *In MarsRT:* the fraction is stamped only on rows that compute it.
  - *Fixed in C# on 2026-09-23, commit 51f3377, the test turned round:* the C# stamps it the same way;
    `MarsThreadedRdpTests.A_primitive_drawn_alone_after_rows_that_measured_no_level_assembles_the_raster_orders_fraction`
    is exact for all sixteen seeds, and the Dam no longer parts from MarsRT's split (§5.6.7). The two-cycle path of
    both cores stamps the fraction the old way when level of detail is off; that is argued and not shown, and left
    (`Mars_Rdp.md` §2.9.3).
- **A load is joined only if a draw has reached its bytes since the image was set.**
  - *How.* A load before an image's first draw is run apart, so the draws after it may write its source before a
    slower processor has read it. Raster order has the load read first.
  - *Found by.* ThreadSanitizer, in Ocarina of Time from its state: an unordered read by one worker's `load_row` and
    write by another's `write_memory`, reported four times with two workers and once with four. The frames matched,
    by timing.
  - *In MarsRT:* any load whose source can meet the current colour or depth image is joined. The span tested is where
    a draw before the next image change could reach, the walker's 1,024 rows at the image's width. The next image
    change is itself a barrier. `a_load_from_the_current_image_before_its_first_draw_is_run_by_every_processor_together`
    holds this.
  - *In C#:* the rule is unchanged. Its race is argued from the code and not shown in C#, because a race that timing
    hides cannot be shown by a test without a race detector.
  - *Shown and fixed in C# on 2026-09-23, commit 4291e9e:* the C# game comparison with four processors from Ocarina's
    state, under a loaded machine, parted from the C# core unthreaded in 10 of 16 runs; with MarsRT's rule ported,
    in 0 of 18 (`Mars_Rdp.md` §2.9.5).

**One rule C# can keep and Rust cannot.**
- *The read.* In a shared primitive, the pixel at the image's width reads the next row's first bytes, and that row's
  owner may be writing them.
- *In C#:* the read is made, and the stamps discard what it read.
- *In Rust:* the read is a data race, and so undefined behaviour, even though its value is thrown away. MarsRT
  therefore reads nothing there (`blind`): that pixel reads through a view of no memory, and every read past its end
  reads zero.
- *Why that is exact:* the value is dead. The next row's owner makes the read that matters itself
  (`RecordAliasedRead`), and otherwise the value is superseded before anything reads it.

#### 5.6.7 The evidence, and the races sought

**The Rust oracles.**
- `tests/threads.rs` holds 24 cases:
  - `MarsThreadedRdpTests`' cases, ported;
  - the split's cases, with two to four workers;
  - a seeded stress of lists handed over in pieces, with loads from what was drawn, and processor reads and writes of
    every image between them (24 seeds with one worker, 12 each with two, three and four);
  - the cases for the level of detail, the barrier and the hazard load above;
  - two written because a mutant survived without them (§5.6.9): a lost wake-up, and a depth slope carried into a
    primitive drawn alone.
- `tests/sites.rs` holds the fifteen site tests.
- `tests/games.rs` runs a reference machine and a subject in one process, with the same input, and compares the save
  state, the picture and the sound after every frame. The subject's state is taken one of two ways:
  - joined;
  - as a snapshot loaded, with its words run, into a scratch machine. This way leaves the workers running across
    frames.

| Mode | Frames a game | Result, six games |
| --- | --- | --- |
| threaded, state joined every frame | 300 | identical |
| threaded, snapshot every frame | 300 | identical |
| deferred, picture a frame late | 300 | identical |
| threaded and deferred, snapshot every frame | 300 | identical |
| threaded, verifier on, snapshot every frame | 600 | identical; no byte outside a mark, a range or a box |
| two, three and four workers, snapshot every frame | 300 | identical |

The six are Super Mario 64, Ocarina of Time and GoldenEye, each from power-on and from its gameplay state. Each run
reports the workers' counters, so a run in which nothing reached the drain cannot pass. Super Mario 64 from power-on
draws nothing before frame 121. In 600 verified frames the drain ran 2.2 million words for it, and 14.5 million for
the Dam.

**Through the shim** (`MarsRtThreadsTests`), with the C# core's own harness:

| Comparison | Frames a game | Result, six games |
| --- | --- | --- |
| MarsRT threaded against MarsRT on one thread, state every frame | 300 | identical |
| the same, with the state every sixtieth frame, so the drain runs across frames | 300 | identical |
| threaded and deferred against one thread, a picture late | 300 | identical |
| two and four workers against one thread | 300 | identical |
| MarsRT threaded against the C# core threaded, blocks off, at once | 600 | identical |
| the same, both deferred | 600 | identical |
| MarsRT's four workers against the C# core's four | 300 | identical in five; the Dam parts at frame 6, in `_lodFraction` (§5.6.6). *Since the C# fix of 2026-09-23 (51f3377), identical in all six.* |

**The C# failures.** Each is shown by a WiseMan test that asserts the C# behaviour: the range read, the deferred
repeat, the fraction and the deadlock. Fixing the C# therefore fails the test, and the test is then to be turned
around. *Turned round on 2026-09-23 with the fixes (`Mars_Rdp.md` §2.9); the tests now live
in `MarsThreadedRdpTests` and `MarsDeferredPresentationTests` and assert the exact behaviour.*

**ThreadSanitizer, on a stable compiler.**
- *The setup.* No nightly compiler is installed, and the Fedora toolchain ships no Rust TSan runtime. The
  instrumentation is in rustc, and `RUSTC_BOOTSTRAP=1` unlocks it. `-Zexternal-clangrt` then links clang's
  `libclang_rt.tsan`, with clang as the linker and the ABI check waived:

      RUSTC_BOOTSTRAP=1 RUSTFLAGS="-Zsanitizer=thread -Zexternal-clangrt -Cunsafe-allow-abi-mismatch=sanitizer \
        -Clinker=clang -Clink-arg=-fsanitize=thread" cargo test --release --lib --target-dir <scratch>

- *The positive control* is a mutant (§5.6.9) that publishes the ring's count with Relaxed in place of Release. No
  equality test catches it. TSan reports it within seconds.
- *The blind spot.* Without rust-src, std is not rebuilt with instrumentation, so TSan cannot see std's own
  synchronisation. libtest's result channel is reported, and that report, and only that one, is suppressed. MarsRT's
  hand-offs are therefore its own atomics, never a std channel or mutex, so TSan sees every one of them.

| Run | Result |
| --- | --- |
| the thread tests, the site tests, the switches and the scan tests (51, and 55 once the last four were written) | no report |
| the games threaded (joined, snapshot, and deferred), 100 frames each, one worker | no report in 18 runs |
| the games with two and four workers, 100 frames each, before the hazard-load fix | five reports, all in Ocarina of Time from its state (§5.6.6) |
| the same, after it, with the hazard-load test | no report in 12 runs |

**What TSan decides, and what it does not.** TSan decides by happens-before, not by timing. It reports two accesses
that nothing orders even when they happened a millisecond apart. So a race is reported whenever both accesses occur
in a run, not only when they collide. It still sees only the accesses a run makes, and a path no run takes is not
checked. Nor does it see an ordering that is present but wrong, such as a read that waits for the wrong word. That is
what the equality comparisons and the site tests are for.

**Repetition under load.** The thread, site and switch tests were run 40 times at eight test threads: 38 cases, since
the last two had not yet been written, with the stress at 24 seeds. The runs went beside the mutant round's three
builds and test runs, so the workers met a loaded, preempting scheduler. All 40 runs passed. This is weak evidence on its own, since a schedule that no run happened to meet is not
excluded. It is recorded because a hang, which TSan does not report, would have shown here.

#### 5.6.8 Speed

The runs were made flat out, from the gameplay states, on this 16-core desktop with the machine otherwise idle, in
three rounds with the order rotated between rounds. The ranges given are the three rounds'.

**Against the C# core, both in one process through WiseMan** (`MarsRtThreadsTests.Bench`, `EMUSEN_MARSRT_BENCH=1`),
600 frames a run. MarsRT runs through its shim, cheats and picture included. The C# core runs in its production
configuration: compiled blocks in the CPU and the RSP, the threaded RDP with its default worker count
(`ProcessorCount / 3`, four here), and deferred presentation. MarsRT's split uses the same count.

| ms a frame | MarsRT, one thread | MarsRT threaded | MarsRT threaded, deferred | MarsRT, four workers, deferred | C# production |
| --- | --- | --- | --- | --- | --- |
| Super Mario 64 | 14.78–15.06 | 10.72–10.93 | 8.07–8.37 | 6.13–6.37 | 6.12–6.15 |
| Ocarina of Time | 17.10–17.31 | 13.87–13.94 | 8.66–8.73 | 7.22–7.61 | 7.36–7.64 |
| GoldenEye, the Dam | 29.91–29.95 | 21.49–21.55 | 17.90–18.21 | 17.99–18.03 | 14.75–16.22 |

**MarsRT's own modes without the shim** (`examples/threads`, the picture on, 300 frames a run). Every run's final
state hash was the same in every mode:

| ms a frame | one thread | threaded | threaded, deferred | four workers, deferred |
| --- | --- | --- | --- | --- |
| Super Mario 64 | 13.42–14.10 | 9.61–9.65 | 7.91–8.19 | **5.96–6.02** |
| Ocarina of Time | 15.23–15.34 | 12.40–12.45 | 8.70–8.74 | **7.04–7.15** |
| GoldenEye, the Dam | 28.27–28.41 | 19.59–19.66 | **16.93–17.04** | 17.19–17.35 |

**Where the time goes.** The workers' counters give the emulation thread's waits and the first worker's busy time
for one run of each mode:
- *Super Mario 64 on one worker.* The emulation thread waits 1.98 ms a frame, all of it at the scan, for a drain that
  is busy 6.1 ms. At four workers it waits nothing, and the drain's leader is busy 2.2 ms.
- *Ocarina of Time.* The waits are the CPU's loads (0.62 ms, site 2) and the RSP's transfers (1.06 ms, site 11). These
  are the waits `Mars_Rdp.md` §2.6.3 found in C#, and a transfer is a range, so it keeps the coarse wait. At two
  workers they fall to 0.11 ms, and at four to nothing.
- *GoldenEye.* The emulation thread waits under 0.05 ms at any count. The drain is busy 9.4 ms on one worker, and
  deferral takes the scan-out's 2.7 ms off the thread, but workers buy nothing, because the thread is the bound.

At four workers, then, no game's emulation thread waits for anything. The three numbers in bold in the second table
are the emulation thread's own cost: 6.0, 7.1 and 17.0 ms a frame.

**What the comparison says.**
- *At the C# core's production configuration,* MarsRT with the same worker count is level with it on Super Mario 64
  and Ocarina of Time, where the rounds overlap. On GoldenEye it is behind by 2 to 3 ms. GoldenEye is bound by its
  CPU, and the C# CPU runs compiled blocks where MarsRT interprets. A 300-frame run of the same comparison, made
  earlier, had MarsRT ahead on Ocarina of Time. The 600-frame rounds overlap, so that lead is withdrawn.
- *On one worker,* MarsRT is about twice as fast as the C# core on the first two games. The 300-frame run measured
  the C# core with one worker at 15.6–16.2 ms on Super Mario 64, 16.0–18.3 on Ocarina of Time and 19.9–21.6 on
  GoldenEye, against MarsRT's 8.0–8.4, 8.7 and 17.9–18.2. C#'s single drain is the bound there, which is
  `Mars_Performance.md` §35's finding (the rasteriser alone takes 14.0 ms for Mario's frame 400). MarsRT's drain does
  the same list in about 6 ms. This is the configuration a machine with few cores gets.
- *Through the shim,* MarsRT on one thread costs 0.7 to 2.1 ms a frame more than it does in the example. The extra
  includes the shim's copy of the picture and its audio. That overhead was not taken apart.

**The prediction of §5.7, retired.**
- *The prediction:* about 4 to 5 ms a frame for Super Mario 64, at or below the C# core. *The measurement:* 6.0 ms
  at four workers in the example and 6.1 to 6.4 through the shim. That is level with the C# core, not below it. On
  one worker it is 8.0 ms.
- *The prediction:* about 20 ms for GoldenEye, still above the C# core. *The measurement:* 16.9 to 17.3 ms in the
  example and 18.0 through the shim. That is still above the C# core, as predicted, by 2 to 3 ms.
- *Why it missed.* The prediction subtracted the whole measured cost of the RDP and the scan-out from the one-thread
  frame. What it did not subtract is what threading leaves on the emulation thread: the shadow and its marks, the
  scan's `Prepare` and the capture's copy. Super Mario 64 lost 7.4 ms of its 13.4, not the 8 to 9 the subtraction
  assumed. How the remaining 6.0 ms divides was not measured.
- *GoldenEye* lost 11.4 ms, close to the 12 the subtraction assumed. Its lower number owes more to the window than
  to the design. §5.7's 32.2 ms is the mean of 1,500 frames, while these 300 frames cost 28.3 ms on one thread.

**The one-thread path's cost,** checked because it is still the default. `examples/frames … scan` was run on the
branch's base (7ea79b1) and on this branch, interleaved, three rounds, with the same state hashes:

| ms a frame | before | after |
| --- | --- | --- |
| Super Mario 64 | 13.07–13.31 | 13.51–13.54 |
| Ocarina of Time | 15.19–15.23 | 15.57–15.67 |
| GoldenEye, the Dam | 28.46–28.51 | 28.25–28.47 |

The unthreaded path is 1.5 to 3.5 per cent slower on Super Mario 64 and 2.5 to 3 per cent slower on Ocarina of Time,
and unchanged on GoldenEye. The candidates are the marks' tests, which the unthreaded path makes too, and `Ram`'s
accessors, which check each access's bounds. Neither has been measured apart, and the cost is recorded here as a
regression that has been found and not yet explained.

#### 5.6.9 Mutants

Each mutant was applied alone to a copy of the crate, built apart, and run against the suites its code can reach:
- the thread, site, switch and scan tests, with the stress at 24 seeds;
- the threaded games, 120 frames;
- the games with the verifier on, 40 frames;
- the split games, 60 frames at two and four workers;
- the deferred games, 120 frames;
- the thread and site tests under TSan.

The unmutated crate passed every suite. The final round ran 44 mutants from a frozen copy of the source, and a
second round reran its five survivors, and two new capture mutants, once the tests below had been written. A hang
counts as caught.

| Mutant | Caught by |
| --- | --- |
| each of the thirteen waits dropped (sites 0 to 8 and 11, the SI's two, the VI's two, the SP's two) | its site test; the bus read and write also by TSan; the load and store also by the games |
| the host's read without its wait; its write without its join | the site test for the host, and TSan |
| the ring's count published with Relaxed; a worker's count stored with Relaxed | TSan only |
| a load's bytes not marked; the depth image not marked; a rectangle's box a row short | the verifier; the first two also by the games |
| a wait one word short; a narrowed read freed a word early; every waiter a bystander | the unit tests; the last also by the games and TSan |
| a range read narrowed by its first eight bytes (C#'s rule) | its unit test (§5.6.2); no game |
| a pause not waited for; a snapshot's tail dropped | the unit tests and the games |
| a batch's idle marks left idle | an assertion added for it this round (below) |
| the capture's hidden bits not copied; the join keeping the shown frame | the deferred games |
| a repeat skipped though the raster changed (C#'s rule) | its unit test (§5.6.5); no game |
| the capture ending at the last line, or one line past it | the unit tests and the deferred games |
| the split: every row stamping the fraction (C#'s rule); the coverage not stamped; the memory colour from an earlier writer; the read past a row's end not made by the next row's owner | the unit tests and the split games |
| the split: a load from a drawn image run apart; an image change without its barrier | the unit tests; the second also by TSan; not by the split games |
| the split: the completed count taken from the leader alone | the verifier, the split games and TSan |
| the split: the pixel at the width read though the next row is another's | TSan only |
| the split: a barrier's waiter leaving the pause point (C#'s rule) | a hang in the unit tests, and TSan |
| no kick before a wait | a hang in the lost wake-up test (below) |
| the leader drawing alone without assembling | the slope test (below) |

**Survivors, and what was done about each.**
- *A batch's idle marks left idle,* rather than downgraded to the batch's count at its end, survived round 2. It is
  conservative: a reader waits for the whole drain rather than for the batch, and exactness is unaffected. An
  assertion on the mark's value after a batch now catches it, since the downgrade is what C# does and what the speed
  depends on.
- *No kick before a wait* survived two rounds. The kick is there for a wake-up the publish can lose. The worker
  stores its sleeping flag and then tests the count. The publish stores the count and then tests the flag. Nothing
  orders the second pair, so each side can see the other's old value, and the worker parks with a word waiting. Only
  a wait's kick then wakes it, if a wait comes before the batch's end. No run met that schedule.
  `a_wait_inside_a_batch_wakes_a_worker_whose_publish_wake_up_was_lost` makes the loss happen, through a test switch
  that drops the publish's wake-up. It then takes a list word that the batch's own fill covers, so the take must wait
  inside the batch. The unmutated test passes in 0.06 s, and the mutant hangs in three runs of three.
- *The leader drawing alone without assembling* survived round 3. It was taken for equivalent until a probe counted
  what the assembly changes. Over the thread tests, it changed a scratch field 189 times, and the combiner's carried
  result never. The walker resets that result at every primitive (`Rdp.Walker.cs` does the same), so it is not a
  carry between primitives. It is also why the first attempts at a test, which relied on it, could not fail.
  - The carry that is real is the stored depth slope. A two-cycle primitive whose first blend reads memory alpha
    shifts by the slope of the pixel before, and at a primitive's first pixel that is the last row's slope.
  - `a_slope_carried_into_a_primitive_drawn_alone_is_the_raster_orders` gives the depth image a different slope
    encoding on each row, tests it with shared triangles that do not write it, and then draws four such primitives
    alone. The mutant parts RDRAM, and the unmutated crate is exact at two, three and four workers.
- *The capture a line short, three lines short, or without the span slack* survive every round, and are equivalent.
  `Vi.Reach` asks for four lines past the last line the picture steps to, and two lines and two row spans past that.
  The walk reads at most the line after its last, plus a span. The two mutants that cut into that need, ending the
  capture at the last line or one past it, are caught. The slack is C#'s, kept so that the two cores capture alike.

**The split games catch less than the unit tests.** Four split mutants pass 60 frames of six games at two and four
workers. Games rarely load from an image they are drawing, and rarely change the image while a slower worker is
behind, so the split's rules are held by the targeted tests and TSan rather than by play.

**Skipped.** The rounds ran every mutant listed. No mutant was written for the presenter's hand-off, the verifier's
cache, or the seqlock of the idle ranges. Those are covered only by the tests and TSan runs above.

#### 5.6.10 What is left

- ~~The multiple, antialiasing and the device, which MarsRT does not draw yet.~~ *Drawn since §6.4, on the threads and
  deferred.*
- ~~The C# core's four failures (§5.6.2, §5.6.5, §5.6.6). They are the C#'s to fix, and the WiseMan tests that show them
  are the ones to turn around when it is.~~ *Fixed and turned round 2026-09-23 (`Mars_Rdp.md` §2.9).*
- Rewind for MarsRT. §5.5 kept it off per console until a snapshot is proven. A snapshot with workers running is now
  proven headlessly in every frame of six games, but turning rewind on is a decision for play.
- The defaults. The switches are wired through the shim and default off. Whether to turn them on for MarsRT in
  Mistress is a decision for play, which headless tests cannot make. *Turned on 2026-09-22, §6.2.*

### 5.7 Where MarsRT stands against the C# core in production (2026-09-22)

After stage 1, MarsRT was measured single-threaded against the C# core's shipping configuration, on this 16-core
desktop, flat out, over three rounds of 1,500 frames from the gameplay states. MarsRT ran with its RDP and its
scan-out on the emulation thread (`examples/frames … scan`). The C# core ran with compiled blocks, the threaded RDP
and deferred presentation (pacebench, `PACED=0`).

| Game | MarsRT, one thread | C# production |
|---|---|---|
| SM64 | 13.13–13.18 ms | 5.71–5.90 ms |
| OoT | 15.06–15.15 ms | 7.10–7.17 ms |
| GoldenEye | 32.17–32.23 ms | 17.72–17.78 ms |

MarsRT is about 2.2 times slower in production terms. That is the expected shape. §5.2's like-for-like table has
MarsRT's CPU and RSP faster than C#'s, even against C#'s compiled blocks: 10.8 ms against 18.3 ms for SM64 with the
RDP on the emulation thread in both. What the production C# has and MarsRT does not yet have is work moved off the
emulation thread: the RDP (5 to 9 ms a frame here) and the scan-out (about 3 ms).

**The prediction, stated before §5.6 is measured:**
- With the RDP and the scan-out off its thread, MarsRT reaches about 4 to 5 ms a frame on SM64, at or below the C#
  core.
- GoldenEye reaches about 20 ms, still above it. The recompiler of stage 5 is what that gap is left to.
- The prediction is subtraction from measured parts, and it will be replaced by §5.6's measurement.

*Retired 2026-09-22 by §5.6.8.* Super Mario 64 reached 6.0 ms a frame at four workers, level with the C# core rather
than below it, and 8.0 ms on one. GoldenEye reached 17.0 ms, still above the C# core as predicted. Threading leaves
the shadow, the marks and the capture on the emulation thread, and the subtraction had not counted them.

*And the gap it left to stage 5 is closed (§5.8.8).* With the recompiler on, the same three rounds put the Dam at
14.95 ms a frame against the C# core's 14.88 to 16.71, Super Mario 64 at 5.96 against 6.12 and Ocarina of Time at
6.94 against 7.37. What closed GoldenEye's gap was not the compiler but keeping the decode: sixteen per cent of its
frame was the fetch and the translation of code behind the TLB.

### 5.8 The recompiler (2026-09-22)

*Stage 5.* MarsRT's processor now runs its code in **blocks**: runs of consecutive instructions decoded once, validated
against memory on every entry, and run between the checks the interpreter makes at every step. Where the C# has one
recompiler this page has three tiers, each proven against the interpreter before the next was built, because each is a
separate claim about what the emulator gains:

| Tier | What a block is | Where its code comes from |
| --- | --- | --- |
| 1, decoded | a loop over the block's instructions, each with the handler chosen when the block was shaped | nothing is compiled |
| 2, compiled | machine code for the block, the simple arithmetic, the branches and the aligned loads inline and everything else a call to the interpreter's own handler | Cranelift, on a thread of its own |
| 3, compiled with the registers held | the same, with the guest registers in host registers across the block and RDRAM's aligned stores made by the code itself | the same |

**The claim is the C# recompiler's** (`Mars_Recompiler.md` §0): a block leaves the machine in the state the interpreter
would have left it after the same instructions, at every point the machine can be observed — the registers, the
devices, the cycle count, the save state — and every event lands after exactly the instruction it lands after in the
interpreter. The tier is a setting; the default is off, and §5.8.7 says what is recommended.

**The design is C#'s where it can be, and this section records only where it differs.** Shaping (a branch and its slot,
an ender, sixty-four words, the end of memory), the refusal of a branch whose slot is a branch or an ender, the cache
by physical word address, the comparison of the block's words with memory on every entry, the dispatcher's address
rules, blocks behind the TLB held to their page, and compiling on another thread are `Mars_Recompiler.md` §1 to §4 and
§17, ported.

| Module | What it holds | Lines |
| --- | --- | --- |
| `cpu/blocks/mod.rs` | the dispatcher, the block, the tiers, the mapped fetch, the counters, and what a compiled block's exit leaves | 517 |
| `cpu/blocks/shape.rs` | what ends a block and what is refused, and the cycles an instruction ticks | 92 |
| `cpu/blocks/ops.rs` | each instruction's handler behind a C ABI, chosen once when the block is shaped | 179 |
| `cpu/blocks/decoded.rs` | tier 1: the block's instructions as the interpreter's steps, less the fetch and the interrupt check | 75 |
| `cpu/blocks/cache.rs` | blocks by physical word address, in pages allocated where code runs | 47 |
| `cpu/blocks/verify.rs` | the interpreter run beside the blocks and compared after every instruction | 77 |
| `cpu/blocks/jit/mod.rs` | the compiler's thread, its queue, the slots code is published in, and its counters | 239 |
| `cpu/blocks/jit/emit.rs` | the Cranelift IR for one block | 1,076 |
| `ffi/blocks.rs`, `Shim/MarsRtCore.cs` | the switch and the counters; `UseBlocks`, `BlockTier`, `VerifyBlocks` and a `Recompiler` setting | 33, +60 |
| `tests/blocks.rs` | the random programs and the twenty-one mechanisms, at every tier | 791 |

#### 5.8.1 What a block may skip, and what it may not

A block does not fetch, and does not run the interrupt check before each of its instructions. Everything else it does
is the interpreter's own code. What makes that exact is not an argument about each instruction but four conditions,
and a block leaves at the first step boundary where one of them fails:

- **The words are the words it was made from.** They are compared with memory on entry, as C# compares them, and the
  block leaves after any store of its own that could have landed in them, after any write to memory it did not make
  (the bus's write count), and after any step of the signal processor that wrote (§5.8.3). The drain of a threaded
  display processor is waited for over the block's own bytes before the comparison, at site 5.
- **The interrupt check would do nothing.** Its inputs are the recheck flag and the MI's line. A COP0 write, a TLB
  instruction and `ERET` end the block after themselves; a fault leaves it; the line can move only through a write to
  a device (which leaves the block) or the signal processor (§5.8.3).
- **No event and no timer is due.** The interpreter tests both after every instruction. A tier 1 block tests them too,
  since it runs the interpreter's own tick. A compiled block does not: the dispatcher admits it only when
  `cycles + the block's longest run < the earliest of the next event, the timer and the frame's cap`, so no instruction
  of it can reach one. The stop cannot move inside a block, because every device write and every COP0 write leaves it.
- **The straight line is the line the block was shaped from.** A likely branch that threw its slot away, a jump
  through a register and every other exit leave the block at the instruction the interpreter would have left it at.

**What the dispatcher leaves to one interpreter step** is C#'s list (`Mars_Recompiler.md` §3.1): a pending delay slot,
a mode that is not kernel, a misaligned or untranslatable program counter, an address outside RDRAM, a refused entry,
a mapped block that would cross its page — and one MarsRT has of its own, an instruction whose stall cycles are still
owed, which no state at a step boundary carries.

#### 5.8.2 What compiled code emits, and what it leaves to the dispatcher

A compiled block is one function, `(processor, bus, context) -> exit`, called through the C ABI. The context carries
the virtual address the block was entered at, the stop, RDRAM and its two mark tables, and the verifier's interpreter.

- **Inline:** the shifts, the logic, the compares, the non-trapping adds and subtracts in both widths, the immediates,
  `LUI`, the moves to and from `HI` and `LO`, `SYNC`; every branch and jump but the coprocessor's; and the seven
  aligned integer loads, whose fast case is the interpreter's own — a direct kernel address, aligned, inside RDRAM,
  its page not marked — with the slow case the call the block would have made. Tier 3 adds the four aligned stores,
  with the MI's repeat and the page's mark tested as the interpreter tests them.
- **A call to the interpreter's handler** for everything else, with the counters and the current address written back
  first, and the two program counters as well where the handler reads them.
- **The counters are compile-time constants along the straight line.** `bus.cycles` and the instruction count are read
  once at entry and written back only where an observer could look: before a call, at every exit, and after every
  instruction while the verifier is on. The multiplies and divides add the vendor's stall to that constant.
- **Three exits.** *Done* leaves the machine at a step boundary and the dispatcher sets the count a step leaves.
  *Raised* leaves the counters and the registers as they stood before the faulting instruction, and the dispatcher
  enters the exception and ticks one cycle, as the interpreter's step does. *Finish* is a store that may have reached
  a device or the block's own words, or an ender: the instruction ran and the dispatcher pays its tick, which is the
  interpreter's own, events, timer and all.
- **A loop stays in its block** (`Mars_Recompiler.md` §3.4) when the block's last branch is relative and returns to its
  own first word: the guard is tested again with the cycles the pass spent, and the loop is taken while it holds. The
  idle loop is not looped over, because the frame loop passes it whole (§5.2).
- **The registers, at tier 3,** are Cranelift variables, loaded at entry for every register the block names. Before a
  call, only the registers that handler reads are written back; after it, the registers it writes are read again; on
  every exit the rest follow. The dirty set is a compile-time set along the straight line, and at a loop's head it is
  every register the block writes anywhere, which is a superset of what any pass can leave dirty.

#### 5.8.3 The signal processor beside a block: measured, and not kept

While the processor runs, the interpreter steps it once per CPU cycle from inside the tick, and a block must do the
same. A second compiled variant does: it tests the halt flag after every instruction, calls a helper that steps the
processor as the tick steps it, and leaves the block when that step wrote memory or moved the interrupt line — C#'s
`RspRan`, emitted. It is exact, and it is off.

*Measured* at step 2, three interleaved rounds of 600 frames from the gameplay states, four workers, deferred, the
variant on against off (blocks entered while the processor runs then run decoded):

| ms a frame | the variant on | the variant off |
| --- | --- | --- |
| Super Mario 64 | 5.90, 5.72, 5.77 | 5.79, 5.73, 5.74 |
| Ocarina of Time | 6.80, 6.79, 6.79 | 6.75, 6.79, 6.78 |
| GoldenEye, the Dam | 14.74, 14.79, 14.77 | 14.50, 14.52, 14.40 |

GoldenEye is two per cent better without it in every round, and the other two are unchanged. The call costs more than
the compiled instruction around it saves, which is `Mars_Rsp.md` §13's finding about a step at a time, in another
core. `EMUSEN_MARSRT_BESIDE=1` turns it on; the tests run with it on, so that the code it emits is still graded.

*The batch that would pay for it, priced and not built.* The processor's instructions between two of its events touch
only its own registers and its two memories, and a run of CPU instructions that touch no memory touches neither, so
the processor's steps for such a run could be taken in one call before it, its events never early. Counting the runs
of two or more memory-free instructions in the blocks GoldenEye enters beside a running processor: 44.3 million of
98.7 million compiled instructions stand in 15.5 million such runs, 2.9 instructions each. The batch would replace
44.3 million calls with 15.5 million, which at the seven nanoseconds a call measures is **0.3 milliseconds a frame**,
two per cent of the Dam's frame, for a run-ahead whose exactness rests on that disjointness. Not built.

#### 5.8.4 Cranelift, and the thread it runs on

**The dependency.** `cranelift-codegen`, `cranelift-frontend`, `cranelift-jit`, `cranelift-module` and
`cranelift-native`, 0.136.0, from crates.io, with `regalloc2`, `cranelift-entity`, `cranelift-bforest`,
`cranelift-bitset`, `cranelift-control`, `cranelift-assembler-x64`, `wasmtime-internal-core` and
`wasmtime-internal-jit-icache-coherence` behind them and the usual small crates (`anyhow`, `smallvec`, `hashbrown`,
`log`, `memmap2`, `region`, `target-lexicon`, `gimli`, `libc`). **Cranelift is Apache-2.0 WITH LLVM-exception**, which
is compatible with this project's GPL-3.0; the small crates are MIT or MIT/Apache-2.0. It is the code generator of
Wasmtime, it is not LLVM, and it compiles a block of seven instructions in about a third of a millisecond (§5.8.8).

**The thread.** A block is handed over at its sixty-fourth entry, as C# hands one over, and runs decoded until its code
is published. The compiler's thread owns the Cranelift module; nothing of the machine crosses to it but the block's
words, and nothing comes back but one pointer, stored with Release and read with Acquire. The queue is a spin lock of
MarsRT's own with `park_timeout`, not a channel, for the reason §5.6.7 gives: ThreadSanitizer sees MarsRT's own
atomics and not std's.

**W^X.** `cranelift-jit` writes a batch of functions into fresh pages and makes them readable and executable in one
call, which starts the next batch on a new page: a page that holds code is never writable again. A batch is every job
queued when the thread wakes, so the pages are shared by the blocks compiled together.

**Code is never freed one block at a time.** A block whose words changed is dropped, and its code stays in the module.
Past 256 MB of code the dispatcher drops every block and the module with it, and the blocks are compiled again as they
run; the counter says how often that happened. No game has reached it (GoldenEye's 600 frames compile 20 MB).

**A block Cranelift refuses** — none does now — publishes a sentinel rather than a pointer and runs decoded for ever;
the tests fail if the counter moves.

#### 5.8.5 The evidence

Six oracles, each able to see something the others cannot. Every one of them runs at every tier.

1. **The interpreter, stepped beside the blocks and compared after every instruction.** `verify.rs` holds a clone of
   the machine; the dispatcher steps it once for every instruction the blocks run — a call the compiled code makes
   itself, with everything written back first — and compares the processor, the run's derived fields and every device
   after each, and the whole machine, RDRAM included, at the end of the run. It is C#'s `VerifyBlockStep` made
   stronger: where C# proves that the check it skipped would have done nothing, this compares the machine it left.
   `EMUSEN_MARSRT_VERIFY_BLOCKS=1` or `MarsRtCore.VerifyBlocks` turns it on; the idle loop is mirrored rather than
   stepped, so that a frame of it does not take an hour.
2. **Random programs, compared every sixteen steps.** §5.2.1's differential, ported to Rust with its own generator, at
   every tier at once: 160 seeds, 24,000 steps each, the processor compared every sixteen and the whole machine at the
   end. A block is compiled at its second entry and waited for, so short programs run compiled: 2.0 million of the 2.6
   million instructions the blocks ran were compiled code. The same programs run again with the verifier on.
3. **The mechanisms, one at a time.** Seventeen tests, each a program written for one thing the dispatcher or the
   emitter has to get right: a hot loop with the video and audio interfaces due inside it, frames that end on a field
   and on the cycle cap, the timer's interrupt taken inside a block, a fault in the middle of one, a block that
   rewrites its own words, code replaced under a block from outside, a likely branch not taken, a branch in a delay
   slot left to the interpreter, the multiply and divide stalls, a store to the video interface inside a loop, a
   mapped loop across two pages whose frames are not neighbours with a decoy after the first, a page remapped by a TLB
   write alone, a jump to a misaligned address and one into a mapped segment, a state loaded over a machine with
   blocks, and the signal processor running its own loop beside the blocks.
4. **The hardware corpus.** `n64-systemtest` boots and runs to its end at each tier, and its transcript is compared
   line by line with the interpreter's, which `MarsRtTests` holds line by line to the C# core's. All 1,722 lines are
   identical at every tier, ending in "Failed 46 of 4637 tests".
5. **Real games, frame by frame.** Super Mario 64, Ocarina of Time and GoldenEye, from power-on and from the gameplay
   states, 600 frames each, the save state, the picture and the sound compared after every frame against MarsRT
   interpreted — unthreaded, and with four rasteriser workers and the picture deferred, where the state is taken as a
   snapshot so that the drain runs across frames.
6. **Against the C# core, through the shim.** `MarsRtTests` runs the same six games against the C# interpreter, and
   the three gameplay states again at each tier; `MarsRtThreadsTests` runs MarsRT recompiled and threaded against the
   C# core threaded, at once and deferred. The corpus runs there too, compared line by line with the C# core's own
   transcript.

**The runs, counted.** In Rust: 160 random programs at three tiers, 40 of them again under the verifier, 21 mechanism
tests at three tiers, the corpus at three tiers, and 36 game runs of 600 frames — three tiers, six games, unthreaded
and on four workers deferred — every one identical in state, picture and sound after every frame. Through WiseMan,
366 tests: 12 game runs against the C# interpreter, 9 more at the three tiers, 6 against the C# core threaded (each
at once and deferred), 120 random programs at the three tiers, 8 synthetic systems from boot, and the corpus twice.

**A run that carries its blocks against one that does not.** `Mars_Recompiler.md` §13 found the C# core's stale shape
through that asymmetry, and it is a test here: each game runs 100 frames with the recompiler on, its state is taken,
and a second machine loads it with an empty cache; both then run 100 frames and are compared after each. They agree
in all six, with caches that end far apart — 33,073 blocks live in the machine that played the Dam against 22,956 in
the one that loaded it.

**ThreadSanitizer**, built as §5.6.7 builds it, over the twenty-one block tests at every tier with the variant beside
a running processor on: one report, libtest's own result channel, the one §5.6.7 also had to suppress. Everything
else it first reported was std's own synchronisation, which is not instrumented here — an `Arc`'s refcount as a block
or its slot is dropped, and the page-size `OnceLock` inside the `region` crate. Two blind spots are worth stating
plainly: **the compiler's hand-off rides on `Arc`**, unlike the display processor's, so TSan cannot see that ordering;
and **compiled code is not instrumented at all**, so no access it makes is checked. What is checked is every access
the handlers it calls make.

#### 5.8.6 Mutants

Fifteen were made, each alone, on a copy of the crate built apart, and run against the suites its code can reach: the
block tests (the random programs at every tier, the verifier's pass and the seventeen mechanisms), the corpus at every
tier, and — for a mutant the first two let through — the six games at 90 frames, unthreaded and on four workers
deferred. A hang counts as a catch; none hung.

| Mutant | Caught by |
| --- | --- |
| the inline add subtracts | the block tests, the corpus |
| the inline unsigned compare is signed | the block tests |
| a branch's tick is not counted | the block tests, the corpus |
| the guard lets an instruction reach the stop | the block tests |
| the loop back is taken whatever the stop | the block tests |
| the loop back leaves the slot flag up | the block tests |
| a likely branch not taken runs its slot | the block tests |
| an exit after a store leaves the addresses stale | the block tests, the corpus |
| a compiled block is not compared with memory | the block tests |
| the registers a handler reads are not written back (tier 3) | the block tests, the corpus |
| an exit leaves its dirty registers in host registers (tier 3) | the block tests, the corpus |
| a step of the processor beside never leaves the block | **the games alone** |
| a register jump reads its target after its link | **survived**, then caught by the test written for it |
| a store into the block's own words does not leave it | **survived**, then caught by the test written for it |
| a decoded block ignores a write it did not make | **survived**, then caught by the test written for it |

**What the three survivors taught, which is the round's whole value.**

- *A test that compares every few steps grades the decoded tier only.* `run_steps(n)` gives a block a cycle cap of the
  steps that remain, and the guard refuses compiled code whose longest run could reach that cap. Eleven mechanism
  tests compared every five to thirty steps, so their blocks never ran compiled at all, and two mutants of the emitter
  walked through them. The strides are now two hundred and up, and the random programs' sixteen still admits compiled
  code because their blocks are short. *The lesson is general: a differential whose comparison is finer than the
  dispatcher's own unit of work does not test the dispatcher.*
- *A block that rewrites its own words every pass is never compiled.* The first test for the store into a block's own
  words changed a word each turn, so the comparison on entry discarded the block every turn and it never reached the
  threshold; the mutant that removed the check never met compiled code. The test now writes a word that alternates
  with a counter's bit that turns over as often as the address comes back to the block, so the block is compiled
  between writes and the pass that writes runs with code the memory no longer matches.
- *A write the block did not make must happen more than once, and land mid-block.* The first test for the signal
  processor's DMA over a running block transferred once, at a cycle past the words it wrote, so nothing stale ever
  ran. The test now transfers again and again with an alternating word.

#### 5.8.7 The switch, and what is recommended

`Machine::set_recompiler(bool)` turns it on and `Blocks::tier` chooses the tier; through the shim they are
`MarsRtCore.UseBlocks` (Mars's own key for the same thing) and `BlockTier`, and a `Recompiler` setting stands beside
Mars's keys in the graphics settings, with a hint that says it is exact. `VerifyBlocks` turns on the verifier. The environment carries the same three for measurement: `EMUSEN_MARSRT_TIER`, `EMUSEN_MARSRT_VERIFY_BLOCKS`,
`EMUSEN_MARSRT_THRESHOLD`; `EMUSEN_MARSRT_NOMAPPEDBLOCKS=1` leaves mapped code to the interpreter and
`EMUSEN_MARSRT_BESIDE=1` compiles the variant §5.8.3 rejected. The interface version is 6, which adds
`mars_machine_set_recompiler` and `mars_blocks_counters`.

**The recommendation is on, at step 2**, which is what `BlockTier` gives a caller who asks for no tier: it is the
fastest of the three in every game (§5.8.8), it is the cheapest to compile, and it is the only one that is never
slower than the interpreter. The stage was built with the default off, as every MarsRT switch has been until it was
proven in play, and turning it on was left as a decision for play, as the threads' switches were.

**That decision was taken on 2026-09-22, the day the stage landed: the shim's default is on**, at step 2 —
`MarsRtCore.UseBlocks` starts true and the `Recompiler` setting's default is `true` — on the strength of §5.8.5's
oracles and §5.8.8's measurement rather than hours of play, which the threads' switches had been given first. Two
things the default does not change. The machine in the library still boots with the recompiler off and is turned on
through `set_recompiler`, so the Rust tests' interpreter oracle stays the plain machine and the shim is the only place
the default lives; and `MarsRtTests.Twin` asks for the interpreter by name, so the comparisons against the C# core keep
their claim while the tier runs turn the blocks on themselves. What the default leaves to play is what the oracles
cannot see: a game none of the three is, driven by a player. Off is the interpreter, a switch away and still exact.

#### 5.8.8 Speed

**The method** is the phase's: three interleaved rounds of 600 frames from the three gameplay states, flat out, on this
sixteen-processor machine, the order rotated between rounds; every run's state hash is compared, and all of them agree.
Two shapes are measured. *Production* is MarsRT as a frontend runs it — four rasteriser workers, the picture deferred,
the scan-out and the shim's copy included — and is where the comparison with the C# core belongs. *The processor's own
view* is MarsRT on one thread with the picture not scanned, where the display processor is on the emulation thread and
a change in the CPU's cost is a third of what it is in production terms.

**Production, through the shim, against the C# core as it ships** (`MarsRtThreadsTests.Bench`, milliseconds a frame,
the three rounds):

| ms a frame | MarsRT interpreted | step 1, decoded | step 2, compiled | step 3, registers held | C# production |
| --- | --- | --- | --- | --- | --- |
| Super Mario 64 | 6.06, 6.11, 6.11 | 6.16, 6.38, 6.11 | **5.92, 6.01, 5.96** | 5.94, 5.97, 5.95 | 6.12, 6.22, 6.05 |
| Ocarina of Time | 7.17, 7.15, 7.19 | 7.44, 7.48, 7.50 | **6.94, 6.92, 6.98** | 6.98, 6.97, 6.97 | 7.37, 7.33, 7.78 |
| GoldenEye, the Dam | 17.95, 17.87, 17.92 | 15.14, 15.03, 14.95 | **14.95, 14.95, 14.99** | 14.94, 15.04, 15.00 | 16.71, 14.88, 14.95 |

**MarsRT's own runs, without the shim** (`examples/threads … split 4`, the same three rounds):

| ms a frame | interpreted | step 1 | step 2 | step 3 |
| --- | --- | --- | --- | --- |
| Super Mario 64 | 5.87, 5.95, 5.87 | 6.00, 5.98, 6.07 | **5.71, 5.74, 5.73** | 5.74, 5.75, 5.81 |
| Ocarina of Time | 7.00, 7.03, 6.95 | 7.29, 7.29, 7.26 | **6.75, 6.77, 6.82** | 6.82, 6.88, 6.85 |
| GoldenEye, the Dam | 17.89, 17.23, 17.17 | 14.35, 14.33, 14.38 | **14.36, 14.44, 14.29** | 14.30, 14.45, 14.39 |

**The processor's own view** (`examples/frames`, the display processor on the emulation thread, no scan, two rounds):

| ms a frame | interpreted | step 1 | step 2 | step 3 |
| --- | --- | --- | --- | --- |
| Super Mario 64 | 10.76, 10.61 | 10.74, 10.70 | 10.53, 10.44 | 10.62, 10.41 |
| Ocarina of Time | 10.89, 10.86 | 11.05, 11.11 | 10.59, 10.59 | 10.56, 10.60 |
| GoldenEye, the Dam | 25.18, 25.24 | 22.27, 22.45 | 22.28, 22.29 | 22.31, 22.34 |

**What each step bought.**

- **Step 1, the decode kept, is GoldenEye's whole gain and the other two games' small loss.** The Dam falls from 17.9
  to 15.0 milliseconds a frame, sixteen per cent, and nothing the compiler does afterwards adds to it. Super Mario 64
  and Ocarina of Time are one to four per cent *worse* than the interpreter: their blocks average seven instructions,
  and a dispatch that looks a block up, compares its bytes and opens a step costs more than the fetch and decode it
  spares at that length. GoldenEye's blocks are shorter still, five instructions, but its code is behind the TLB and
  its every fetch would otherwise be a translation; that is what the block's remembered page removes.
- **Step 2, the code, buys three to four per cent on the two games whose CPU is not the bound, and nothing on
  GoldenEye.** It also undoes step 1's loss. Its gain is where the compiled share is: two thirds of the block
  instructions in Super Mario 64 and Ocarina of Time run as machine code, and **two per cent** of GoldenEye's do,
  because nearly every block GoldenEye enters is entered while the signal processor runs beside it, and §5.8.3 leaves
  those decoded.
- **Step 3, the registers held, buys nothing**, which is `Mars_Recompiler.md` §11's result in another core: every call
  into a handler pays for the registers it must write back, real code calls every few instructions, and the code grows
  by a third (1,540 bytes a block against 1,122 in Super Mario 64) for an entry that must stream it in. It is kept as
  a tier and it is not the default.
- **Against the C# core**, MarsRT with the recompiler is at or below it on all three: 5.96 against 6.12 on Super Mario
  64, 6.94 against 7.37 on Ocarina of Time, and 14.95 against 14.95 on the Dam, where the C# core's own rounds range
  from 14.88 to 16.71. The gap §5.7 left to this stage — the Dam at 17 to 18 milliseconds against the C# core's 14.8
  to 16.2 — is closed.

**What it costs to compile.** Over 600 frames, at step 2:

| | blocks compiled | the thread's time | code | per block | of the blocks' instructions, compiled |
| --- | --- | --- | --- | --- | --- |
| Super Mario 64 | 2,938 | 0.81 s | 3.30 MB | 0.28 ms, 1,122 bytes | 60.2M of 86.4M |
| Ocarina of Time | 5,224 | 1.56 s | 6.71 MB | 0.30 ms, 1,285 bytes | 125.1M of 188.3M |
| GoldenEye, the Dam | 628 | 0.20 s | 0.74 MB | 0.32 ms, 1,185 bytes | 4.0M of 214.4M |

At step 3 the same blocks take 0.37 to 0.44 milliseconds each and 1,540 to 1,752 bytes. A block is handed over at its
sixty-fourth entry and runs decoded until its code arrives, so nothing waits for the compiler; the counts above are
what one thread did beside 600 frames, seven seconds of wall clock at most.

**The entry's own cost, bounded, and two changes that measured nothing.** An entry looks a block up, compares its
bytes with memory, opens the step and builds the context the code reads — about a fifth of the Dam's emulation thread
by an interrupt sample (the dispatcher 8 per cent, the C library, which is the comparison's `memcmp`, 7). Three things
were tried against it. *The comparison removed altogether* — inexact, so a bound and nothing more — is worth 0.1 to
0.35 milliseconds a frame: 5.64 against 5.74 in Super Mario 64, 6.66 against 6.85 in Ocarina of Time, 14.47 against
14.80 in the Dam, one round each. *The comparison written by hand*, eight bytes at a time rather than a call to
`memcmp`, and *the context built once a run* rather than once an entry, both measured nothing at all against the same
build, and by `Mars_Performance.md` §6's rule neither is carried. What is left of that fifth is the lookup and the
step's own opening, and `Mars_Recompiler.md` §9 to §12 is the record of what happens to a lever aimed at it.

**What the numbers do not say.** The three states are driven with no input, as the examples run them; the same states
under the WiseMan comparisons, where a button pattern is pressed every frame, put far more of GoldenEye's code through
blocks with the processor halted (17.3 million compiled entries against half a million here). A game's compiled share
is a property of what it is doing, and one state is not a sample.

#### 5.8.9 What is not done

- **The processor beside a block** runs decoded, because compiling for it measures worse (§5.8.3), and the batch that
  would change that is priced and not built.
- **Loads and stores through the TLB** call the interpreter, as they do in C# (`Mars_Recompiler.md` §16): only a
  direct kernel address is inline. A mapped block whose words would cross its page is interpreted rather than split.
- **The coprocessor is a call.** Its arithmetic is software floating point, exact by construction and long; the moves,
  the loads into it and its branch are calls as well. C#'s §16 inlines the two loads; MarsRT does not.
- **Nothing is chained, nothing is extended, and no block holds more than sixty-four words.** `Mars_Recompiler.md`
  §10 to §12 measured all three in C# and kept none; this page has not measured them in Rust, and its own numbers
  (§5.8.8) say where the time is instead.
- ~~**The debugger, the coverage recorder and single-stepping** are outside MarsRT altogether (§5.2.5), so nothing here
  had to keep them working.~~ *§6.5 brought them in, and an observed frame runs the interpreter, so the blocks still
  need not know them; what a debugger through the blocks would cost is §6.5.6.*
- **Code is not freed a block at a time.** §5.8.4's bound drops everything at once, and no game has reached it.
- **The verifier runs on one thread.** It needs the display processor unthreaded, since it clones the machine.

## 6. Finishing the port (planned 2026-09-22)

§5's six stages are five done and one open, and what the port set out to do is measured (§5.8.8): on this desktop,
MarsRT with its recompiler is at or below the C# core on all three games. This section is the plan for what remains,
written the way §5 was written before it began: what each stage covers, what its oracle is, what it costs, and the
prediction or the decision rule stated before the measurement that will retire it.

**What "finished" means here**, since the word has to be held to something:

1. **A player gets MarsRT** on every platform EmuSen publishes for, in the configuration the numbers were measured in,
   without choosing it.
2. **Nothing a player or the tooling could do on the C# core is lost on MarsRT**, or its loss is written down.
3. **The C# core's role is decided**, rather than left as the fallback by default.

**Where each stage's work stands against those three**, as of this writing:

| Stage | What it covers | Serves | Oracle | Cost |
| --- | --- | --- | --- | --- |
| A | The measurement the port was for: the handheld | 1 | the C# core on the same device | hours, once the device is reachable |
| B | The defaults: threads, the engine, the other frontends | 1 | §5.6 and §5.8's suites, a play session | hours |
| C | Every platform: the library built for the three other targets, and stripped | 1 | the crate's own tests and WiseMan's synthetic systems on each | a day |
| D | Stage 6: the multiple, antialiasing and the GPU path | 2 | the C# multiple and the C# GPU path, picture for picture | two to three days |
| E | The debugger's hooks in the Rust loop | 2 | the debugger's WiseMan tests, run against both engines | one to two days |
| F | Parity leftovers: ROM patches, the profiler's phases, rewind, the four C# failures, the single-thread regression | 2 | each item's own | a day, in pieces |
| G | The C# core's future | 3 | none: a decision, with the criteria below | — |

A and B come first because they are cheap and they decide what D is worth. C is mechanical and can run beside
anything. D and E do not depend on each other. F's items are side jobs. G is last, and its criteria are what A to E
deliver. *A is done (§6.1): the device is bound where the desktop is, so D is for the multiple and follows E.*

### 6.1 Stage A: the handheld

**The port was priced on a device, not this desktop**: GoldenEye at 68 per cent of console on the Legion Go S against
109 per cent here (§5). No MarsRT number exists from that device yet; it was unreachable on the day the recompiler
landed, and on the day the threads did.

**What is measured.** `examples/threads` and `examples/frames`, which need no .NET, on the three gameplay states:
the interpreter against tier 2, and one worker against as many as the device has cores for, three interleaved rounds
as §5.8.8 runs them; then the published Mistress with the C# engine and with MarsRT, since the shim and the scan-out
are part of a frame. The C# core's numbers on the same device, the same day, are the comparison; the desktop's are
not.

**What it decides.** Whether the handheld's bound is the emulation thread or the rasteriser. If the frame is CPU-bound
there as it is here, the levers left are the recompiler's (§5.8.9) and stage D matters only for the multiple; if it is
RDP-bound, because the device has fewer cores to split across, the split's shape and stage D's device path come
before anything else. The device has either four cores or eight depending on its model, and the split was measured
here on sixteen.

**Two things to settle before the numbers are trusted.** The examples link the desktop's glibc (2.43); the device's
version is not recorded, and an older one will refuse the binary at load. The fix is either to build on the device or
to pin the target's glibc through `cargo zigbuild`, which stage C wants anyway. And the published Mistress carries its
own .NET but not its own C library, so the same question applies to `libmarsrt.so` inside it, and the shim's report
(`MarsNative.Report`) is where a refused load shows.

**The prediction, before the measurement.** The desktop's ratio (the Dam at 15.0 against the C# core's 14.9 to 16.7)
carries over only if the device's core count lets the split run at four workers. At two workers the C# core and
MarsRT both lose the split's gain and the frame is the emulation thread's again, where MarsRT's advantage is the
recompiler's sixteen per cent on the Dam and three to four on the others. So: GoldenEye somewhat better than the C#
core there, not level, and the other two about level. Held loosely, as §3.4 taught.

**Measured, 2026-09-22, the same evening.** The device is a Legion Go S: a Ryzen Z1 Extreme, eight Zen 4 cores and
sixteen threads — the same count as this desktop's 7700X, so the prediction's premise was wrong before its numbers
were — with half the desktop's L3 (16 MB against 32), 24 GB, SteamOS 3.8.27, glibc 2.41 against the 2.34 the binaries
ask for, in desktop mode on the charger, `amd_pstate` active with `balance_performance`. Three tools ran, each from
the three gameplay states over 600 frames, the modes interleaved and rotated between rounds; every run's state hash
was the desktop's, so the machine is exact there as well.

*Production, through the shim, four workers, deferred, against the C# core as it ships* (`enginebench`, kept with the
speed tooling outside the repository; milliseconds a frame, three rounds — the C# core's first round carries .NET's
JIT warm-up and its second and third are the comparison):

| ms a frame | C# production | MarsRT interpreted | tier 1 | **tier 2** |
| --- | --- | --- | --- | --- |
| Super Mario 64 | 9.78, 8.09, 8.30 | 7.47, 7.43, 7.66 | 7.58, 7.73, 7.78 | **7.67, 7.57, 7.53** |
| Ocarina of Time | 12.61, 9.82, 9.95 | 8.81, 8.68, 8.84 | 9.22, 9.15, 9.30 | **8.89, 8.85, 9.07** |
| GoldenEye, the Dam | 22.98, 20.33, 20.31 | 22.10, 21.75, 21.93 | 18.69, 18.55, 18.62 | **18.70, 18.55, 18.64** |

*MarsRT alone, threaded* (`examples/threads … split 4`, two runs of three rounds): Super Mario 64 interpreted
6.88 to 7.39, tier 2 7.01 to 7.33; Ocarina of Time 8.32 to 8.53 against 8.42 to 8.79; the Dam 20.39 to 20.78 against
17.26 to 18.66. Tier 1 was 7.49 to 7.63, 8.75 to 9.11 and 17.27 to 17.65.

*MarsRT on one thread, no scan* (`examples/frames`, two rounds): Super Mario 64 12.54 and 12.08 interpreted, 12.11
and 11.89 at tier 2; Ocarina of Time 12.43 and 12.33 against 12.30 and 12.23; the Dam 28.50 and 28.34 against 25.22
and 25.23.

**What it says.**

- **MarsRT is eight to nine per cent below the C# core on the device on all three games**, where on the desktop it
  was three to five on two and level on the Dam. The Dam runs at 18.6 ms a frame against the console's 16.7: ninety
  per cent of full speed, against the C# core's eighty-two. Neither reaches it. The other two are far past it.
- **MarsRT loses less to the device than the C# core does.** From desktop to handheld the C# core's frame grows
  by 30 to 34 per cent and MarsRT's by 24 to 29: the same cores at a lower power budget and half the cache cost the
  managed runtime more than the native one, which is where the extra margin came from. That is the retired
  prediction's error in the other direction: it expected the two games without a CPU bound to be level, and they
  are not, for a reason that has nothing to do with core counts.
- **Tier 2 stays the right default there.** A first run put it one to four per cent behind the interpreter on the
  two light games, threaded; a second run of the same put it level, and on one thread it is one to three per cent
  ahead, as on the desktop. The device's round-to-round noise is about three per cent, and the first run was inside
  it. On the Dam it is twelve to fifteen per cent ahead in every configuration, and tier 1 costs two to five per
  cent on the light games as it does here.
- **The frame is the emulation thread's, on the device as on the desktop.** On the Dam the first rasteriser worker
  is busy 3.7 to 3.9 ms of an 18 ms frame and the emulation thread waits for the workers 0.000 ms a frame; the
  split takes the one-thread frame from 25.2 to 17.5 ms, a third, against the desktop's 35 per cent. So the
  handheld is bound where the desktop is: the CPU with the signal processor beside it. **Stage D's device path is
  for the multiple and not for the frame**, and it comes after stage E; what the Dam has left on the device is
  §5.8.9's list, the interpreter's own cost and the processor's, and the batch priced at two per cent.

**Measured again, 2026-09-23, after §6.10 to §6.12.** The same device, the same `custom` profile at its 40 W limit, on
the charger; yesterday's binary (the recompiler alone) and today's (the vector unit in host vectors, the decoded
table, the walk in bands) run interleaved, three rounds of 600 frames from the gameplay states, four workers, deferred,
tier 2; every run's state hash the desktop's.

| ms a frame | yesterday's MarsRT | today's MarsRT |
| --- | --- | --- |
| Super Mario 64 | 6.73, 7.21, 7.49 | **4.41, 4.62, 4.76** |
| Ocarina of Time | 8.05, 8.15, 8.89 | **5.53, 5.55, 6.03** |
| GoldenEye, the Dam | 16.77, 19.05, 18.00 | **11.45, 12.32, 12.43** |

And through the shim beside the C# core as it ships (`enginebench`, the C# core's first round its JIT's warm-up):

| ms a frame | C# production | MarsRT |
| --- | --- | --- |
| Super Mario 64 | 10.34, 7.88, 7.88 | **5.70, 5.16, 4.85** |
| Ocarina of Time | 12.27, 9.48, 9.37 | **5.68, 5.68, 5.80** |
| GoldenEye, the Dam | 21.57, 19.04, 19.38 | **11.89, 11.66, 11.74** |

**The Dam is at full speed on the device**: 11.7 ms a frame against the console's 16.7, about 140 per cent, where the
C# core is at 86 to 88. The C# core takes 1.6 times MarsRT's time on each of the three games here, against about 1.5
on the desktop. The device's rounds drift, yesterday's binary by 2.3 ms across three, which is its temperature rather
than the code; today's stays under 12.5 in every round. This is the measurement the port was priced on (§5, §6.1),
and it is the first on which a Nintendo 64 game EmuSen could not run at full speed on this device does.

**The tooling,** so that the run can be repeated: the two examples and the three states in `~/emusen-bench/rt/`
on the device with `rtbench.sh`, which interleaves modes and rounds; `enginebench`, a self-contained publish that
runs both engines from one state through the shim and prints the state hash of each, in
`~/.cache/emusen/probe/mars-speed/enginebench/` here and `~/emusen-bench/enginebench/` there. The device's glibc
was not a problem and no pin was needed; stage C's `cargo zigbuild` is therefore for the other platforms alone.

### 6.2 Stage B: the defaults, the engine, the other frontends

Three defaults are wrong for a player, and each is a line:

- **The threads.** MarsRT's `ThreadedRdp`, `RdpWorkers` and `DeferredPresentation` default to off, one and off
  (§5.6), where the C# core's default to on, a third of the processors clamped to four, and on. Every production
  number in §5.8.8 was taken at four workers, deferred. A player who picks MarsRT today gets the rasteriser on the
  emulation thread, about 8 ms a frame on Super Mario 64 (§5.7) against the 6 measured. The defaults should be the C#
  core's, by the same reasoning §5.8.7 gave for the recompiler: the suites are the evidence, and off is a switch
  away.
- **The engine.** `CoreCatalog`'s Engine row for the N64 defaults to the C# core. Once A and the thread defaults are in,
  it should default to MarsRT where the library loads, with the factory's fallback unchanged and its message kept, so
  that a platform without the library (stage C) still plays.
- **Hotaru and Pharaoh** build the C# core with no choice (§5.5.4). They should read the same per-console setting the
  factory reads, so the engine is one decision, not three.

**Oracle.** `MarsRtFrontendTests` and `MarsRtEngineTests` for the wiring; the exactness suites do not change. The
decision to flip is a play decision as §5.6.10 and §5.8.7 said, and it should follow a session on the desktop and one
on the handheld with the crash log clean. One thing MarsRT has that the C# core does not, which bears on the risk:
its pause point for a snapshot is a tested barrier (§5.6.4), where the C# core's is the deadlock §5.6.2 records.

**Done, 2026-09-22, the same evening as §6.1.** The shim's thread defaults are Mars's: `ThreadedRdp` on, `RdpWorkers`
one per three cores clamped to four, `DeferredPresentation` on, so a player who picks MarsRT gets the configuration
§5.8.8 and §6.1 measured, and the three hints no longer say "off until proven in play". The `Threads` transform now
rewrites hints alone and lets Mars's defaults through, so the two engines' rows agree by construction. The tests
that mean "MarsRT's interpreter on one thread" — `MarsRtTests.Twin` and the frontend comparisons — ask for it by
name, as §5.8.7 made them ask for the interpreter. `CoreFactory.ConfiguredEngine` reads `graphics.json`'s
`Consoles.<console>.Engine` for a ROM, and Hotaru, Pharaoh and the probe runner pass it to `Create` and print the
factory's notice when the engine asked for is not the one running; Mistress reads the same value through its own
config, so the engine is one decision. **The engine's default is still Mars (C#).** Flipping it is one line in
`CoreCatalog` and the factory's rule for a null engine, and it waits, as this stage said it should, for a session of
play on each machine with the crash log clean.

### 6.3 Stage C: every platform

**The gap.** `EmuSen.csproj` builds the crate only when the publish's runtime identifier is the host's
(`EmuSenNativeForHost`), so the `win-x64`, `osx-x64` and `osx-arm64` trees under `out/` carry no `marsrt.dll` or
`libmarsrt.dylib`, and on those machines MarsRT is silently unavailable: the factory falls back to the C# core and
says so in the Engine row's message. The linux-x64 library it does ship is 34 MB, because the release profile keeps
debug information for `ipsample` and the perf map.

**The route.** Two are open, and the first is recommended:

1. **Build the three libraries where they run.** A GitHub Actions workflow with a three-runner matrix (`ubuntu`,
   `windows`, `macos`) builds the crate, runs its own tests — the 391 need no ROM — and attaches the libraries as
   artefacts; the publish takes a prebuilt library for a foreign runtime identifier through a new csproj property,
   `EmuSenNativePrebuilt`, and the existing `Exists` guard keeps the fallback when none is given. This needs no
   toolchain on this machine, and it tests each library on the platform it is for.
2. **Cross-compile here** with `cargo zigbuild`, which links every target through one toolchain (Zig) and pins the
   Linux target's glibc. This machine's Rust is Fedora's package, without `rustup`, so the three targets' standard
   libraries would have to come from `rustup` first. It is the route for stage A's glibc pin, and a fallback for 1.

**Stripping.** A second profile, `[profile.dist]`, inheriting release with `strip = "debuginfo"`, for the published
library; the development build keeps its symbols. About 8 MB against 34.

**Two risks to test rather than argue.** `cranelift-jit` 0.136 has no code for Apple Silicon's JIT restrictions: it
takes a page writable, fills it, and turns it executable with `mprotect`, which Apple permits for a process without
the hardened runtime, and refuses for one with it unless the `allow-jit` entitlement is granted. A notarised
build needs the entitlement; a local one does not; and both need a run on a Mac with a `cargo test` of the crate to
say so. Windows has no such rule, and `MarsNative` already knows the library's name there; what needs checking is
only that the exported names survive the `.dll`'s linker unchanged.

**Oracle.** The crate's tests on each platform, and WiseMan's eight synthetic systems there; the corpus and the six
games need local ROMs and stay on this machine.

**Built, 2026-09-22.** Route 1, as recommended:

- `.github/workflows/marsrt.yml` builds the library on four runners — `ubuntu-22.04` for linux-x64, chosen for its
  glibc 2.35 so the library loads on older distributions than the one that built it; `windows-latest` for win-x64;
  `macos-latest` (arm64) for osx-arm64, and for osx-x64 as well, since Apple's toolchain on arm64 links x86_64
  without another tool. Each runs `cargo test --release` first (the 391 tests; the games and the corpus skip without
  their files), which is where the two risks above are answered rather than argued: the block tests run compiled
  code, so a platform whose JIT rules refuse Cranelift's pages fails there. The osx-x64 tests run under Rosetta and
  are allowed to fail; the library is built either way. Each job uploads one artefact, `marsrt-<rid>`, holding the
  library alone. It runs on demand and on a push that touches the crate, so it does not spend the macOS runners'
  minutes on every commit.
- **`[profile.dist]`** in `Cargo.toml` inherits release, drops the line tables and strips debug information: the
  linux library goes from 12.7 MB (34 in a tree that had kept full debug information) to **6.9 MB**. `EmuSen.csproj`
  builds `dist` when `_IsPublishing` is set, which is what `dotnet publish` sets, and `release` for a development
  build, so `ipsample` and the perf map keep their line tables where they are used.
- **`EmuSenNativePrebuilt`**, in `MarsRtPublish.targets` beside the crate, imported by the two published frontends.
  The first attempt put this in `EmuSen.csproj` and it could not work there, and the reason is worth recording: the
  SDK builds a referenced library *without* the publish's `RuntimeIdentifier` (it undefines the property on the
  reference), so the library project always sees the host, and a win-x64 publish made on Linux carried
  `libmarsrt.so` — the first verification found it there, 6.9 MB, the `dist` profile having flowed where the
  identifier had not. The frontends do see the identifier, and at `ComputeResolvedFilesToPublishList`, where
  `DianaOSPublishLayout.targets` already drops native `.pdb`s, the targets file removes any MarsRT library from a
  foreign publish and adds `$(EmuSenNativePrebuilt)/<rid>/<library>` (`win-*` → `marsrt.dll`, `osx-*` →
  `libmarsrt.dylib`) when the property names a directory holding it, or **warns** — "No MarsRT library for
  win-x64: this publish runs the N64 on Mars (C#) alone" — which closes the silence this stage opened with. The
  README's publish recipe shows the property. A second thing the verification found: a distribution's SDK names its
  host by distribution (`NETCoreSdkRuntimeIdentifier` is `fedora.44-x64` here) while a publish asks for the portable
  `linux-x64`, so "foreign" is tested against both names, or the host's own publish would have dropped its library.
  Verified on this machine: a win-x64 publish without a library warns and carries none, one with a stand-in carries
  it at `lib/EmuSen/marsrt.dll`, and the linux-x64 publish carries the 6.9 MB `dist` library.

**The workflow's first run, 2026-09-22 (run 35789632081), all four green.** Every job ran the 391 tests to a pass
before building: linux-x64 in 55 s, win-x64 in 45 s, osx-arm64 in 22 s, and osx-x64 **under Rosetta in 45 s** — the
run that was allowed to fail did not, so Cranelift's x86_64 code runs under Apple's translation as well as natively.
That is the Apple Silicon question answered on the arm64 job: the block tests compiled and ran code on it, so
`cranelift-jit`'s writable-then-executable pages are accepted by a process without the hardened runtime, which is
what an unsigned local build is. A notarised build still needs the entitlement, and that is not tested. The
libraries, downloaded and inspected here: `libmarsrt.so` 6.95 MB (ELF x86-64, needing glibc 2.34, as the desktop's
own build does), `marsrt.dll` 5.19 MB (PE32+ x86-64), `libmarsrt.dylib` 3.41 MB arm64 and 6.40 MB x86_64 (Mach-O);
each exports `mars_machine_run_frame`, `mars_machine_set_recompiler` and `mars_blocks_counters` under the names
`MarsNative` looks up. The jobs took four to twelve minutes, Windows the longest.

**What is not verified.** WiseMan's synthetic systems on the foreign platforms need a .NET job with LunaP checked
out beside the repository (`RedQuE3n/EmuSen.LunaP`), which is left for when a foreign publish is made from these
artefacts; and no publish for another platform has yet been made and run, which is stage B's engine default's
concern as much as this stage's.

### 6.4 Stage D: the multiple, antialiasing and the device

**What it is.** The three settings whose hint still read "MarsRT does not implement this yet" (*until §6.4, below*): `RenderScale`, the
rasteriser at one to four times the console's resolution (`Mars_Video.md`), `Antialiasing`, and `Gpu`, the device path
of `Mars_Gpu.md`, phases 0 to 7 with §14's tiles and §15's device average. In C# the multiple is 18 lines across six
files of the rasteriser and the walker's hardware widths; the device path is 1,288 lines and its shaders.

**Order inside the stage.** The CPU multiple first, because it is the oracle for the device's: `Mars_Video.md`'s
tests grade the C# rasteriser at 2× and 4× picture for picture, and the same tests grade MarsRT's once the shim
honours the key. Then the device path on `ash`, sharing the C# path's SPIR-V byte for byte, so that the differential
between the two cores' device paths has no shader in it, only the host code around it. Then antialiasing, which is
the device average.

**Oracle.** The C# core at the same multiple, picture for picture, every frame of the three games; the device against
the CPU rasteriser at the same multiple, as `Mars_Gpu.md` grades the C# device; the state unchanged by any of it,
since the multiple is the picture's and not the machine's. The GPU tests run on the RX 6800 only, by standing
instruction, with `EMUSEN_MARS_GPU_TEST_DEVICES=all` for the occasional cross-check.

**What it buys, stated first.** At 1× the emulation thread is the bound and the device helps nothing
(`Mars_Gpu.md` §16); the multiple is where it pays, at 150 to 250 per cent of the CPU rasteriser's rate at 4× on this
machine's discrete card. On the handheld's integrated device the figure is unmeasured, and stage A's result says
whether the multiple is what that device's frame has room for. If A finds the handheld CPU-bound at 1×, this stage is
for the desktop's multiple and comes after E; if it finds it RDP-bound, this stage's device path is the next lever
and comes before.
*Retired 2026-09-23 (§6.4.6): the 150 to 250 quoted here is `pacebench`'s unit, a console frame, not a rate of the
processor path. At 4× MarsRT's device ran Mario at 198 per cent and Ocarina at 208 in that unit, above the range, and
2.7 to 4.6 times MarsRT's processor at the same multiple.*

**Built, 2026-09-23.** The claim is `Mars_Video.md` §2.9–§2.11's and `Mars_Gpu.md` §5–§15's, made of MarsRT: the display
processor draws the picture at one to four times the console's resolution beside the machine's own drawing, which is
what the game reads back and what the state holds; the antialiasing setting draws finer still and averages it down,
the product held to four with the averaging giving way first; and with the device on, the multiple is shaded, walked
and averaged on a Vulkan compute device through `ash`, running the C# path's five SPIR-V binaries byte for byte. Each
is exact against the C# core at the same settings, picture for picture, and none of it changes the state. The three
hints that read "MarsRT does not implement this yet" now read as Mars's, and MarsRT ignores no setting of Mars's.

#### 6.4.1 The multiple on the processor

**What was ported** is C#'s own, and the page it lives on is `Mars_Rdp.md` §11: a processor at the multiple beside
the native one (`ScaledDrawing` in `memory/dp.rs`, C#'s `_scale`, `_scaledRdram`, `_scaledHidden` and
`_scaledProcessor`), fed every word after it and drawing into a shadow of RDRAM the multiple squared as large, its
loads reading the machine's RDRAM (`RdpMemory::scaled`, whose `texture` view is the machine's memory while its frame
is the shadow). The rasteriser's changes are the ones C# makes and no others: the colour and depth images and the
scissor scaled at their commands (`fill.rs`, `modes.rs`) and after a state copy (`Rescale`); a rectangle's inclusive
edge moved out to its pixel's last; the attribute steps divided by the multiple; the walker's hardware widths opened
(`walker.rs`: no 0xFFF, 0x1FFF or twelve-bit sign on a column, a limit or the major edge), and the start sub-scanline
(`ahead`: the edges are given at the console's row top, and the walk at the multiple starts some sub-scanlines on); the copy mode stepping each pixel by its own
coordinate (`draw_copy_scaled`, C#'s `DrawCopyScaled`); the texture rectangle's steps shared; and the walker's scratch
widened by the multiple (`Widen`), which is why its arrays became boxed slices. `set_scale` rebuilds the shadow and the
processor, as C#'s `Scale` setter does; a state read empties both, as `ResetScaled` does.

**On the drain** each worker gets a processor at the multiple beside its own (`ScaledStart`, `Threads::start`),
configured for the same share of rows, fed the same word right after it and made to *follow* the native one's
decision to draw a primitive alone (`Rdp::follow`), since the two cannot always decide it alike: a rectangle's own
right edge is a console coordinate at either multiple while the image's width is not, so a rectangle past the width
is alone at one and would not be at two. The leader assembles the others' scratch at the multiple as it does its own.
The shadow needs no marks of its own: it is written by the processors at the multiple only, in the order the native
ones write the machine's, and read by the scan-out behind the marks the native drawing already waits on.

**The scan-out** (`vi/scan.rs`) follows `Mars_Video.md` §2.9: `Prepare` takes the multiple once something has drawn
there (`ScaledDrawn`), the job carries the picture at the multiple, `ReachScaled` and the capture take the shadow's
lines as the machine's are taken, a repeat's shape includes the multiple and the device, `Darken` and the blank edit
both rasters, and `Compose` returns the width it composed, so the frontend receives 1,280 by 960 at two. The average
is `BoxAverage` over the raster at the multiple, taken in `Compose` and only when the averaging divides the raster's
multiple (§2.10). `Core::set_multiple` (`ffi/multiple.rs`) is `MarsCore.ApplyMultiple`: the drawn multiple is the
resolution times the effective averaging, held to four, and a walk still out is joined before either changes.

#### 6.4.2 The device

`rdp/gpu/device.rs` is `GpuDevice.cs` on `ash` 0.38, `rdp/gpu/mod.rs` is `GpuRasteriser.cs` and `rdp/gpu/record.rs` is
`Rdp.Gpu.cs`: the same enumeration and ranking (discrete, integrated, virtual, other; the portability extensions for
MoltenVK), the same buffers, one descriptor set a program written at record time, the same barrier after every
command and to the host at the end, one submission left pending (§14.2), the rows binned into 64 by 1 tiles (§14.1),
the scan on the device over its own memory (§13) and the average over a raster it keeps (§15). The shaders are not
compiled here: `include_bytes!` takes `Mars - N64/Rdp/Gpu/Shaders/*.spv`, and a test compares the five against the files
on disk, so the two cores' device paths share every instruction the device runs and differ only in host code.

**Without Vulkan.** `ash` is built with its `loaded` feature, so the loader is opened at run time by `libloading` and
the library links against nothing new. `Entry::load` failing is `"Vulkan is not available: …"` in the report and the
processor draws, as the C# path's `TryCreate` answers. Every device test asks `GpuDevice::device_names` first and
stands down, which is what a CI runner sees. Verified here by hiding every driver from the loader
(`VK_LOADER_DRIVERS_DISABLE='*'`): the sixteen device tests and the settings tests pass, and the interface test checks
that the asking is refused with a reason.

**Two decisions that are not C#'s.** A device that still fits the memory at a new multiple, or survives a state read,
is kept and emptied (`GpuRasteriser::clear`) rather than opened again, since opening a Vulkan device is the slowest
thing either core does between frames; what a keep must do is what `ResetScaled` does to the shadow, and
`after_a_state_is_read_the_device_holds_what_the_processor_path_holds` holds it. And the device is carried across a
state read by moving it from the old machine to the new, since MarsRT's read builds a machine and C#'s reads into the
old one.

**Licences.** `ash` is MIT OR Apache-2.0 and `libloading` ISC, both compatible with the GPL-3.0; recorded in
`Cargo.toml` beside Cranelift's note.

#### 6.4.3 The evidence

**Against the C# core, picture for picture.** `MarsRT_at_a_multiple_leaves_what_the_csharp_core_at_that_multiple_leaves`
runs MarsRT through the shim (four workers, deferred or not, the recompiler at tier 2) beside the C# core at the same
`RenderScale`, `Antialiasing` and `Gpu`, and a third machine, the C# core at one with the picture off, whose state the
multiple must not change. Every frame compares the state byte for byte, the picture's shape and every byte of it, and
the sound. Nine settings on each of the three play states, at once and deferred, 300 frames each: 2× and 4× on the
processor, 2× with 2× antialiasing (drawn at four, shown at two), 4× with 4× antialiasing (held to none by
`EffectiveAntialiasing`), 1× with 2× antialiasing, and on the device 2×, 4×, 2× with 2× antialiasing (averaged on the
device) and 1× with 4× antialiasing (drawn at four, averaged by four on the device). **54 runs, 16,200 frames, all
exact**; in every run the picture was at the multiple in at least 297 of the 300 frames, and on the device the two
cores' reports named the same card. The C# oracle runs unthreaded, for the race in §6.4.4.

**Against MarsRT itself, in the crate** (`cargo test --release`, 449 tests with the states, all passing):

- `a_machine_at_a_multiple_split_and_deferred_is_the_machine_at_once_at_that_multiple`: at 2× and 4×, four workers,
  deferred, recompiled, snapshot every frame, against the machine at once at the same multiple, picture for picture,
  and the machine at one in state; six runs of 300 frames, 3,600 frames.
- `a_machine_at_a_multiple_on_the_device_is_the_machine_at_once_on_the_processor`: the same at 2× and 4× on the device,
  against the processor; 3,600 frames, identical in state and picture except for Ocarina's play state, where 23 of 300
  pictures part after the device declines (§6.4.4), at both multiples.
- `a_machine_averaged_on_the_device_split_and_deferred_is_the_device_at_once`: 4× by 2, 4× by 4 and 2× by 2 on the
  device, deferred and split, against the device at once; 5,400 frames.
- The multiple's own ten (`tests/multiple.rs`): C#'s `Drawing_at_a_multiple_leaves_the_machines_memory_as_at_one`,
  `A_fill_at_a_multiple_is_the_fill_at_one_at_every_pixel_of_the_multiple`, `After_a_state_is_read_the_multiple_is_drawn_and_shown_again`,
  `A_frame_buffer_high_in_memory_scans_out_at_a_multiple_as_a_low_one_does` (the case `MarsViTests`' page, §2.11,
  records; at 2×, 3× and 4×, at once and deferred) and `The_average_of_a_square_is_its_rounded_mean_channel_by_channel`,
  and five of MarsRT's: the split at a multiple byte for byte against one processor over five lists (the last two a
  load from the drawn image with a live carry, and rectangles past the image's width), a change of the multiple
  between lists, the scan-out N times as wide and averaged back to the console's, the deferred walk a frame late, and
  the core's frame following `set_multiple`.
- The device's sixteen (`tests/gpu.rs`), `MarsGpuRasteriserTests`' cases on MarsRT: fills, shaded and depth-tested
  triangles (one and two cycles, sixteen and thirty-two bits, keyed), textures through the divider's range and edges,
  and copy-mode rectangles, each on the device against the processor at 2×, 3× and 4× over the whole memory at the
  multiple, with no primitive declined and no column past the width; the scissor wider than the image counted; the
  device refused the machine's own picture; the interface on and off at 2×, 3× and 4× with one, three and four
  workers; one setting at one changing nothing; the scan-out over thirteen VI modes at 2×, 3× and 4× at once and
  deferred; the average over twenty-three scans (borders, fades, blanks, fields, repeats) at 2× by 2, 4× by 2 and 4×
  by 4, walking repeats or skipping them; averaging moved onto the device between fields; a walk left pending with a
  frame drawn or a state read after it; the repeated capture with and without a scan between; the device after a
  state read; and the five shaders against the C# files.
- The C# settings cases on the shim (`MarsRtFrontendTests`): Mars's keys with every hint Mars's, the values checked,
  clamped and read back, `EffectiveAntialiasing` at 4× with 2× and at 2× with 2×, and `Gpu` set and read.

With every Vulkan driver hidden the device and multiple tests pass as well (27 of them, each device test saying it
stood down), and `cargo clippy --all-targets` is clean. `GpuDevice` is opened on the RX 6800 alone, by standing
instruction; the integrated device and llvmpipe were not run.

#### 6.4.4 Four defects found on the way, and a race of the C# core's

Each was found by an oracle, shown failing, fixed, and is now held by the test that found it or by one written for it.

- **The immediate scan claimed the deferred capture's device picture.** C#'s immediate scan is a job of its own
  (`_immediate`), so its device scan cannot mark the deferred job's; MarsRT's scan-out has one capture record, and
  `present_now`'s device scan wrote its count into it. A deferred repeat after an immediate scan then reused a device
  picture that was the immediate scan's. `a_repeated_capture_walked_again_on_the_device_is_the_processor_s`, with a
  scan between, failed at byte 164 at two; the fix restores the record after the immediate scan.
- **The device report did not survive a state read.** MarsRT's read builds a new machine and moves the device into it;
  the report stayed behind, so the shim said "off" while the device drew. Found by the C# comparison's report check;
  `after_a_state_is_read_the_device_holds_what_the_processor_path_holds` now asserts the report, and fails without the
  fix.
- **A repeat over an edited raster walked a capture the device never took.** MarsRT walks a deferred scan that repeats
  the last when a border or a blank has changed the raster since (§5.6.5, one of the C# failures it does not follow);
  C# never walks a repeat with `SkipRepeatedScans` on. The device's repeat logic (§14.3) was reached only with every
  scan walked, so MarsRT's extra walk went to the processor's path over the scaled capture, which with the device is
  never taken. While the device averages, the raster counts as edited at every scan (a span is sent whether or not it
  darkens anything), so every repeat took that path: the C# comparison failed in the four averaged cases at frame 8,
  and `a_machine_averaged_on_the_device_split_and_deferred_is_the_device_at_once` reproduced it in the crate at the
  same byte (20,544) of Ocarina's frame 8. The repeat now takes the device's path whenever it is walked; the averaging
  test runs with repeats skipped as well and fails without the fix, at step 1.
- **The scan decided whether the multiple had drawn before the drain had run what it was handed.** `Prepare` reads
  `ScaledDrawn` before any wait, and on the drain that flag is the workers' progress, so the first scan after a load or
  a change of the multiple showed the console's picture or the multiple's by timing: one run in three of Super Mario 64
  at two on the device showed 1,280 by 1,152 at frame 2 where the C# core showed 640 by 576. MarsRT now joins the drain
  first while nothing has drawn, which costs a join once per load or change, and the answer is the words handed over.
  **The C# core has the same race** (`Vi.Prepare` reads `DpInterface.ScaledDrawn`, which on the drain is the workers'
  `Drew`, before `Capture`'s wait), so the C# comparison's oracle is the C# core unthreaded, which is deterministic;
  documented here and not fixed, as the brief asks of C# findings. *Fixed in C# on 2026-09-23, commit 05d41e4:
  `Prepare` asks `ScaledDrawnHandedOver`, which joins first while nothing has drawn, as MarsRT does; the test that shows
  the race and its fix is `Mars_Rdp.md` §2.9.4's. The comparison at the multiple keeps its unthreaded oracle, which
  still suffices.*

**A finding about the C# device path, not a defect of the port.** A primitive the device declines (`Declined.Carry`,
`Declined.Image`) is drawn nowhere: `Rdp.Gpu.cs` returns, and the CPU path never sees it. `Mars_Gpu.md` §9.1 found
no carry in three frames of three games and redirected the work on that basis; from Ocarina's play state the device
declines 259 primitives in 300 frames (83 for the carry, 176 for an eight-bit image), and 23 of the 300 pictures part
from the processor's. MarsRT does the same, by construction, so the two cores' device paths agree and the device
against the processor on that state does not. Super Mario 64's and GoldenEye's states and all three games from
power-on decline nothing. The crate's game test on the device therefore lets a picture part from the processor's
only after a decline, and counts it.

*Considered for the C# on 2026-09-23 and left.* Drawing a declined primitive on the processor path would need the
device's memory read back before it and written back after, per primitive, which is the round trip the device path
exists to avoid (`Mars_Gpu.md` §9.1, §11.2), and a C# change alone would part the C# device path from MarsRT's, which
is the only comparison either has at the multiple on the device. It touches only the device at a multiple, never the
1× default. It remains a finding for the device's design, to be settled in both cores together.

#### 6.4.5 Mutants

Eleven, each put in by hand or by a runner in the scratchpad that restores the source after each, against the crate's
device and multiple tests and, for the one that survived them, the C# comparison:

| Mutant | Tests | Reading |
|---|---|---|
| tiles 32 wide on the host, the shader's 64 (the positive control) | 10 of the device tests fail | caught |
| the scan's submission drops the frame's last batch (a dropped pending write) | 3 fail | caught |
| the presenter reads the scanned picture without waiting for the device | 3 fail | caught |
| a device kept across a state read is not emptied | 1 fails | caught |
| the immediate scan's record not restored (§6.4.4's first defect) | 1 fails | caught |
| the report not carried across a state read (the second) | 1 fails | caught |
| a repeat over an edited raster walked on the processor (the third) | 1 fails, only with repeats skipped | caught |
| the processor at the multiple does not follow the native one's decision to draw alone | 1 fails | caught |
| the walk at the multiple masks the origin to twenty-four bits (§2.11's defect, reintroduced) | 3 fail | caught |
| no start sub-scanline at the multiple (`ahead` zero) | none in the crate; the C# comparison fails at frame 6 of Mario at 2× | caught by the C# oracle alone |
| a dark column's coverage cleared, not kept, in `write_device_picture` | none | **equivalent** |

The sub-scanline mutant survives the crate's device and multiple tests because every oracle there is MarsRT's own walker at the multiple: the
split, the device and the deferred path all share it, so a wrong start is wrong in all of them alike. Only the C# core
is an independent oracle for it, which is the argument for keeping the comparison through the shim. The coverage
mutant is equivalent on this path by construction: the raster `write_device_picture` fills is composed with its fourth
byte made opaque (`compose_into`), no scan reads a raster's coverage back, and while the device averages the raster is
the device's and this function is not called.

#### 6.4.6 Speed

**The prediction, stated before the measurement** (the brief's reading of `Mars_Gpu.md`): on this machine's RX 6800 the
device at 4× runs at 150 to 192 per cent, and 4× antialiasing shown at one at 174 to 248 per cent. Those are
`pacebench`'s figures for the C# device path in §14.6 and §15.4, and their unit is not the processor path's rate but a
game's frame rate on the console: 192 per cent is 10.38 ms, so 100 per cent is 19.9 ms, the figure every pair in those
tables gives. The ranges were for Mario, Ocarina and Wave Race; GoldenEye was not among them.

**How it was measured.** `MarsRtThreadsTests.Bench_at_a_multiple` (`EMUSEN_MARSRT_BENCH=1`, `SCALE`, `AA`), WiseMan
built in Release: from each play state, 600 frames flat out with the picture on, deferred and split over the shim's
default workers, the recompiler at tier 2 on MarsRT and the blocks on C#, four configurations interleaved and the
order reversed in the middle round, three rounds, medians. Each run's report named the card or said "off". The 1×
frame was measured apart, with `examples/threads` (`split 4 blocks`) against the same example built from a clean
extract of 83dca1d, interleaved and alternated the same way.

**1×, against 83dca1d**, ms a frame, medians of three, the joined state's hash identical in all eighteen runs:

| Game | 83dca1d | now |
|---|---|---|
| Super Mario 64 | 5.76 | 5.77 |
| Ocarina of Time | 6.80 | 6.79 |
| GoldenEye 007 | 14.39 | 14.44 |

Level, as it should be: at one the machine carries an empty `ScaledDrawing` and one test of the scale a word.

**At a multiple**, ms a frame, medians of three:

| Game | Setting | MarsRT processor | MarsRT device | C# processor | C# device |
|---|---|---|---|---|---|
| Super Mario 64 | 2× | 13.86 | **6.09** | 20.30 | 8.70 |
| | 4× | 41.71 | **10.03** | 62.40 | 12.54 |
| | 1×, antialiasing 4× | 41.52 | **6.40** | 63.50 | 10.26 |
| Ocarina of Time | 2× | 14.38 | **6.74** | 21.73 | 8.09 |
| | 4× | 43.66 | **9.55** | 63.77 | 10.69 |
| | 1×, antialiasing 4× | 44.75 | **7.11** | 64.10 | 8.95 |
| GoldenEye 007 | 2× | 20.83 | **15.70** | 24.77 | 17.09 |
| | 4× | 52.71 | **19.29** | 72.75 | 20.92 |
| | 1×, antialiasing 4× | 53.52 | **15.54** | 72.27 | 18.36 |

**The prediction's fate: exceeded, and retired.** In `pacebench`'s unit MarsRT's device at 4× runs Mario at 198 per
cent and Ocarina at 208, above the range's top of 192; with 4× antialiasing shown at one, Mario at 311 and Ocarina at
280, above 248. GoldenEye, outside the range's games, is at 103 and 128. Read the other way the brief read it, as a
ratio to the processor path at the same multiple, the device is 4.2, 4.6 and 2.7 times MarsRT's processor at 4×, and
6.5, 6.3 and 3.4 times with 4× antialiasing: the prediction's 1.5 to 2.5 was far short. Why the device gains more on
MarsRT than on C#: the device's own work is the same shaders, and the host's share, walking and binning rows and the
join, is where MarsRT's rasteriser is faster (§5.3, two times the C# one), so the device takes a larger fraction of a
smaller frame. MarsRT's device is ahead of C#'s in every row: C#'s takes 8 to 60 per cent longer. GoldenEye gains
least because its frame is the heaviest of the three before any drawing (14.4 ms at one, against 5.8 and 6.8), and
the device takes none of that.

`pacebench` itself was not run; its numbers and the bench's are both flat-out frame times from the same states, and
the C# device rows here (12.54 ms at 4× in Mario) sit near §14.6's (10.38) without matching it, since the bench
alternates four configurations in one process and `pacebench` runs one; the comparison that matters, MarsRT against C#
at the same settings in the same process, is interleaved.

#### 6.4.7 What is not done

- **The integrated device and llvmpipe.** Every device test ran on the RX 6800 alone; `EMUSEN_MARS_GPU_TEST_DEVICES=all`
  was not run, and neither the Legion Go S's device nor MoltenVK has seen MarsRT's device path. Stage A's question,
  whether the handheld's frame has room for the multiple, is unanswered.
- **The validation layer.** §14.4's configuration was not run over MarsRT's device path; its ordering is C#'s by
  construction (the same barriers, the same pending submission), and the tests' agreement is the only witness here.
- **The race in the C# core** (§6.4.4) is documented and not fixed; the C# comparison's oracle runs unthreaded because
  of it. A fix is for `Vi.Prepare` to wait on the drain before reading `ScaledDrawn` while nothing has drawn, as
  MarsRT now does.
- **The declined primitives** (§6.4.4) are drawn nowhere on either core's device path. The two options `Mars_Gpu.md`
  §9.1 priced, recomputing the neighbour or drawing the primitive on the processor, are now needed by a game and are
  not built.
- **The device's report in the window.** The shim exposes `GpuReport` as `MarsCore` does, and no frontend shows either.
- **The bands and the walk at a multiple on the processor split across threads** are C#'s and not ported; MarsRT's
  walk is one band, at the multiple as at one.
- **Mistress.** No play session at a multiple on MarsRT; headless only, by standing instruction.
- **The transition off the device** deviates for up to two frames as C#'s does (`Mars_Gpu.md` §15.2), and no test holds
  it on either core.


#### 6.4.8 A setting sent again every frame, and the overlap it threw away (2026-09-23)

**The defect.** Mistress sets `SkipRendering` before every frame (`MainWindow`'s loop, for fast-forward's skipped
frames), and `MarsRtCore`'s setter ran `ApplyOptions`, which sent every setting across the boundary again. The last of
them, `mars_machine_set_multiple`, began by joining the deferred presentation (`join_presentation`), whatever it was
given. So in Mistress, and nowhere else, every frame waited for the presenter to finish the walk it had just been
handed — the overlap §6.11 builds was discarded before the next frame began. The C# core's `SkipRendering` is a plain
property and has no such cost; the benches (`examples/threads`, the enginebench, §6.9's sampler) never set it and so
never saw it.

**How it was found, and a prediction that failed first.** A handheld session's `[fps]` line on Donkey Kong 64's title,
at 2x on the device with four workers, stood at 20-23 ms a frame against a `run` of 15-18. §6.13's frame lending had
already removed the collections, so the remainder was split (`EmuSen_Settings_Reference.md` §4.21's `outside:`
figures): requests 0.01 ms, audio 0.01-0.02, the hand-off 0.11-0.16, and **4.1-5.4 ms in "sleep and the rest"** — with
the loop behind its schedule, where the pacer (`FramePacer.Settle`) cannot sleep. The line left at the top of the loop
was the `SkipRendering` assignment. On this desktop, the same probe with and without that assignment before every
frame measured **no difference** (16.6 and 17.7 against 16.7 and 16.7 ms a frame): the desktop's presenter finishes
within the frame, so the join never waited. *The desktop's negative is recorded because it would have closed the
question wrongly;* on the handheld, three rounds of 600 frames from the user's state:

| ms a frame, loop | `SkipRendering` never set | set before every frame (Mistress) |
| --- | --- | --- |
| round 1 | 17.42 | 21.36 |
| round 2 | 17.42 | 21.35 |
| round 3 | 18.66 | 21.88 |

with `RunFrame`'s own mean the same in both columns (16.4-17.9 ms). The 3.2-4 ms is the join.

**The fix, on both sides of the boundary.** `Core::set_multiple` returns at once when the multiple, the averaging and
the device are what it already holds, so an unchanged value leaves a walk out; and `ApplyOptions` in the shim keeps
what it last sent to the current handle and sends only the calls whose values changed, re-sending everything to a new
handle. The early return is taken only once a multiple has been applied (`Core::multiple_applied`). *A first version
compared against the fresh `Core`'s remembered values alone, on the argument that a machine is only ever made with a
fresh `Core` whose values (one, none, no device) are a fresh machine's; the argument was false — a fresh scan-out's
averaging is zero, not one — and `the_drawing_is_the_resolution_times_the_averaging_held_to_four` caught it on the
first run, its first case left at an averaging of zero.*

**The evidence.** `the_multiple_sent_again_unchanged_leaves_the_deferred_walk_out` counts the scan-out's joins: a
`set_multiple` with the values already set must not join, and a changed one must. It failed on the code before the fix
("an unchanged multiple joined the walk out") and passes after. On the handheld, the Mistress-shaped loop (the setting
assigned before every frame) went from **21.46, 21.77, 21.91 ms a frame to 17.56, 17.65, 17.26**, interleaved, the
same as the loop that never assigns it. What remains at 2x on the device on this title, ~17.4 ms, is the core's own
frame, which §6.14 is about.

**What it does not cover.** Every other per-frame property Mistress touches was not audited for the same shape; the
shim's other setters now go through the same comparison, but a setter that calls the library directly would not.
### 6.5 Stage E: the debugger's hooks

**What was missing.** Breakpoints, stepping, watches, the coverage recorder, the call stack's observers, labels and the
frame log: everything `RunFrame`'s debugging loop does that `RunQuietly`'s does not (§5.2.5). MarsRT's debug target
served the inspector through the state transfer, which is read-only; `run_steps` existed and is exact through the
blocks (§5.8), so stepping was the one hook already there.

**The design constraint** is §3.2's rule, which stage 5 kept: the native side never calls C#. So the hooks cannot be
callbacks. They are tables C# pushes down — breakpoint addresses, watch ranges, a coverage buffer — and a frame that
ends early with a reason when one fires: the frame loop takes the observed path only when a table is non-empty, as
the C# takes `RunFrame`'s loop only when something is armed, and returns *why* it stopped and *where*, which the
shim turns into the debug target's events. Coverage is written by Rust into a buffer C# reads at the frame's end; the
call stack is tracked in Rust and read through the transfer.

**Built, 2026-09-22.** The claim is the C# core's (`Mars_Debug.md`): a breakpoint stops in front of its instruction
and a resume runs it, a step runs what it counted, `step over` comes back to the caller, a watch sees a store where it
landed, a data breakpoint halts after the store that wrote it, coverage and the profiler count only while armed. The
oracle is `MarsDebugTests`, made an abstract set of claims (`MarsDebugClaims`) over a rig (`MarsDebugRig`, one for
each engine) and run twice, as `MarsDebugTests` against the C# core and `MarsRtDebugTests` against MarsRT, with the same
expected addresses, counts and contexts. Of the original twenty-four, twenty-two are shared; the two that are not are
named below, and one claim was added to both.

#### 6.5.1 What the machine does: tables, an observed frame, and what it records

`cpu/hooks.rs` holds the tables and the logs (`Cpu.hooks`, a `Skip` field last in the struct so the compiled code's
offsets stand; `sp.trace` holds the signal processor's). Nothing in it is in the state; a load carries it across, as
the blocks are carried (`load_state`).

**The tables.** Enabled breakpoints as pairs of first and last address, compared as the registry compares them, as
signed 32-bit integers; the watches and the data breakpoints as ranges of one memory (RDRAM, DMEM, IMEM, PIF RAM by
`ffi::space`'s numbers — a watch on the `CPU` space sees nothing, as on the C# core, since a store is reported by
where it landed); the depth `step over` and `step out` wait for and the depth guard; and seven flags: track calls,
report stores, stop after an interrupt, stop before every instruction, record coverage, record the signal processor's
coverage, profile.

**The observed frame** (`Machine::run_frame_debug`) is `RunFrame`'s loop with the instructions between the checks run
in Rust: the interpreter alone, no block and no idle skip, the signal processor stepped from the tick as the interpreter
steps it. Before each instruction it asks the tables (`Hooks::stop_before`) and, if any holds, returns a bit-set of
reasons — the address is in a breakpoint range; stop before every instruction; the depth is at or under the target or
over the guard; a store landed in a data-breakpoint range; an interrupt was entered; a log is full — with the machine
at a step boundary and the program counter on the instruction it stopped in front of. Then it records what the C# loop
records before `Cpu.Step()`: the coverage bit at the address's low twenty-four bits, and one instruction for the
profiler's innermost routine. Two things the loop must keep straight, both C#'s: *the instruction a halt stopped in front
of runs unchecked* when the host resumes, which is what makes `continue` leave a breakpoint instead of stopping on it
again; and *a stop the registry says no to continues the frame*, its start and its cap kept, where a halt the host
returned from begins the frame's clock again at the next `RunFrame`, as C#'s does — `_lastFrameCycles` is in the state,
so the two kinds of resume had to be told apart (a first version measured every resume from the resume, and the
crate's halted-and-resumed test caught it at the first frame's end).

**The seams inside the step** are compiled into the observed step alone. `Cpu::step` is `step_in::<false>` and the
observed loop's `step_hooked` is `step_in::<true>`, one generic body whose seams are `if HOOKED && …`; the plain step,
the blocks' handlers (`ops.rs`) and the compiled code are the `false` instantiation and carry no load, no test and no
branch for any of this. The tables and the trace are boxed, so `Cpu` and the signal processor's interface each grow by
a pointer. Both were decided by measurement, §6.5.5, and the second is the one that mattered.

- *Calls and returns*, C#'s `CallObserver` and `ReturnObserver`: `jal`, `jalr` and a taken branch-and-link push as
  they execute, so the call's own delay slot is already a frame deeper; a `jr` through `ra` marks the return, and the
  pop comes at the end of the slot's own step (`return_after_slot`), which is what lands `step over` after the call
  and not one instruction past it. A `jr` through any other register is a jump. The stack is capped at 512 frames and a
  pop with nothing open is counted, as the registry does both. Every push and pop is also logged, so the C# registry
  can replay them and keep its own frames, its entry points and its call counts.
- *Stores*, C#'s `MemoryBus.Report`: after an `sb`, `sh`, `sw`, `sd`, `sc` or `scd` lands — the only stores C# reports;
  the unaligned and coprocessor stores go through `Write32` there and are not — the bytes it left are read back, a
  whole word for the windows that latch one, and each byte that a watch or a data breakpoint covers is logged with
  its memory, offset, value and the storing instruction's address, and a data breakpoint's byte sets the stop for the
  next boundary.
- *Interrupts*, C#'s `InterruptObserver`: `enter_exception` with the interrupt code sets the stop, so the frame ends
  before the handler's first instruction.
- *The signal processor's coverage*: the observed step's tick (`tick_traced`) takes a recording twin of the processor's
  loop while its trace is armed (`Rsp::run_traced`, and the two event instructions recorded by the machine that runs
  them); the plain tick does not look. So `cov rsp` armed alone puts the frame on the observed loop, where the C# core
  keeps `RunQuietly` and only drops `RunIdle`'s whole run of the processor and the native step: the same instructions
  are seen one at a time either way, and the difference is the frame's speed while the recorder is armed, not what it
  records.

**The four decisions the brief asked for**, then:

1. *The recompiler's blocks.* An armed CPU table runs the interpreter for the frame — C#'s rule, and exact by
   construction, since §5.8 proves the blocks against the interpreter and this stage proves the observed loop against
   the plain one. Nothing was built to stop inside a block or to see a tier-3 inline store; the tables are on the
   interpreter's paths only, and the compiled code is untouched. A finer rule waits for someone who debugs at speed.
2. *The idle-loop skip* is off in an observed frame, as it is in C#'s observed loop (`RunIdle` is reached only through
   `StepBlock`): the coverage recorder sees the loop's two instructions and the profiler charges them, which is what
   the C# core's counts show, and the state is the same either way (§5.2's proof of the skip).
3. *The RSP lock-step* is the interpreter's own: the processor stepped from the tick, one instruction a cycle, as
   `Cpu.Step` steps it in C#. With its coverage armed the tick records; the idle loop never runs it whole in an observed
   frame because the observed frame has no idle loop.
4. *A stop with the threaded RDP running* leaves the machine at a step boundary and the drain where it was: nothing
   waits at the stop itself, because nothing has to — every host read of RDRAM waits for the drain (`Core::read_memory`,
   §5.6.4), a state save waits or holds (`Frozen`), and the shim's picture is taken only at a frame's end. A state
   saved at a halt and one saved after the unstopped run of the same frame brought to the same instruction count are
   byte-identical (the crate's `a_frame_halted_and_resumed_is_the_frame_run_through`, and the game proofs below with
   four workers deferred).

#### 6.5.2 What the shim does: the registries pushed, the logs drained, the registry asked

`MarsRtCore` now owns the seven registries `MarsCore` owns (`Watches`, `FrameLog`, `Breakpoints`, `Coverage`,
`RspCoverage`, `CallStack`, `Labels`), wired the same way (`Breakpoints.CallStack`, the frame-number provider, the
entry-point observer), and `MarsRtDebugTarget` hands them out; `A_frontend_bundle_wires_the_target_to_the_core` holds
on MarsRT as it holds on the C# core, and the processor's `DebugCpu` says it can halt.

**The frame.** `RunFrame` asks the same question `MarsCore.RunFrame` asks — `Breakpoints.IsQuiet`, the two coverage
registries, the profiler, and whether a watch or a data breakpoint exists — and takes `mars_machine_advance` when
nothing is armed, which is the plain frame with the tables empty. Otherwise (`RunObserved`):

1. The first instruction is checked here, as C#'s loop checks it (`CouldBreak && ShouldBreak(pc)`), unless the frame
   resumes a halt, in which case it runs unchecked.
2. The registries are pushed down (`mars_debug_set`, `_set_breakpoints`, `_set_ranges`): the enabled breakpoints, the
   write watches and the write data breakpoints by memory, the depth target and the guard, and the flags. Calls are
   tracked and interrupts stop in every observed frame; stores are reported while the target listens; *stop before
   every instruction* is pushed while a step is armed, or while the registry already owes a break (a data or event
   break left pending by a halt that reported another), so that the registry gets its next question at the next
   instruction. Three read-only getters were added to `BreakpointRegistry` for this — `IsSingleStepArmed`,
   `StepDepthTarget`, `HasPendingBreak` — since the fields they read were private and the loop that used to read them
   is now elsewhere.
3. `mars_debug_run_frame` runs to the field's end or a stop.
4. The logs are drained into the registries in the order the observers would have been called: each logged byte to
   `Watches.RecordWrite` with `PC=<the store's address>` as its context and to `Breakpoints.NoteWrite`; each push and
   pop to `CallStack.NotePush` and `NotePop`, stamped with the frame they happened in; the profile's runs to a new
   `CallStackRegistry.NoteInstructions(owner, count)`, which charges a run a core counted itself; the coverage bitmap
   ORed into the registry by a new `CoverageRegistry.Merge(bitmap, instructions)` and cleared on the Rust side, so the
   registry stays the one accumulator and `cov clear` means what it means. An interrupt stop calls
   `Breakpoints.NoteInterrupt(Irq)`, as the C# observer would have during the step.
5. **The registry decides.** `ShouldBreak(pc)` is asked at the stop with everything it would have seen in C# already
   noted, and its answer is the halt: `IsHaltedAtBreakpoint`, `HaltedAddress`, `LastBreakReason` are its. If it says no
   — a condition that did not hold, a logpoint, a forbidden range, a data breakpoint whose value did not match, an
   interrupt nobody was running to — the frame continues from that instruction, unchecked, its clock kept. Rust's
   tables are therefore *candidates*, over-approximations the registry narrows; nothing of the registry's semantics is
   reimplemented in Rust, and a stop the registry refuses costs one crossing of the boundary and nothing else.

A log that fills (65,536 entries) stops the frame with its own reason, is drained, and the frame continues, so a watch
on a hot address loses no event and no count. At a frame's end the shim does what `MarsCore.RunFrame` does after its
loop: the cheats, `FrameLog.RecordFrame` (big-endian reads through the same memory reads the target's spaces use),
`Breakpoints.NoteFrame`, the periodic save, the picture. A halt returns before any of it, as C#'s does.

**What is C#'s alone, and why.** Two of the twenty-four claims stay on the C# fixture:

- *`A_jump_through_another_register_does_not_return`* runs a frame with nothing armed and reads a depth of one. The C#
  core tracks the call stack on every frame, armed or not: its observers are delegates the opcode handlers test for
  null, and `MarsCore.LoadRom` always sets them, so even `RunQuietly`'s blocks take the slow path for every call (a
  finding about the C# core, recorded and not changed here: `Mars_Debug.md` §7 measured the seams as free when the
  target was attached, and did not measure a frame with none, which is every frontend frame). MarsRT tracks the stack
  only in an observed frame, because a push per call on the plain path is the cost this stage was told not to add,
  and the compiled code would need a call-out it does not have. So on MarsRT `bt` with nothing armed shows the stack as
  the last observed frame left it, and a `step out` from a routine entered before observation began has nothing to
  pop to — the same as the C# core's after its 512-frame cap. The rule itself (a `jr` through another register is a
  jump) holds on MarsRT in an observed frame, which `A_jump_through_another_register_does_not_return_in_an_observed_frame`
  shows, and `A_plain_frame_leaves_the_stack_where_the_last_observed_frame_left_it` fixes the difference so it cannot
  drift unnoticed.
- *`A_store_is_reported_only_while_the_observer_listens`* is a test of the C# `MemoryBus` object. MarsRT's counterpart,
  `A_store_is_reported_only_while_the_target_listens`, counts the events a watch collects across a frame with it and a
  frame without, and the crate's `a_store_is_reported_only_while_something_listens_and_only_in_a_range_that_covers_it`
  holds the machine to the same rule.

One claim was added to both: `A_halt_leaves_the_coverage_and_the_profile_current`, because a halt is a frame's end for
every counter and the shim's drain at a stop is what makes it so.

#### 6.5.3 The evidence

*The claims.* `MarsDebugTests` (25: the 23 shared and the two that are the C# core's) and `MarsRtDebugTests` (33: the
23 shared, four of MarsRT's own, and the six game cases below) green, the same transcripts; the crate's own
`tests/debug.rs` (18) drives the machine directly, the hooks' unit tests in `cpu/hooks.rs` (6) the tables alone.

*Exactness, the tables armed and nothing that halts.* Every table armed — a breakpoint at an address no game runs, a
watch over four kilobytes of RDRAM and a data breakpoint over its first 256 bytes (in the crate the stop after each such
store is taken and resumed, as the host resumes when the registry says no; through the shim the data breakpoint carries
a value no byte can have, so the registry says no at every one of those stops), a stop after every interrupt, coverage on
both processors, the profiler, a depth guard — against the same game run plain, state, picture and sound after every
frame, 300 frames from the three gameplay states, unthreaded and on four workers deferred:

- the crate's `a_game_halted_at_breakpoints_and_resumed_is_the_game_run_through` and
  `an_observed_machine_with_every_table_armed_is_the_machine_plain` (`tests/games.rs`), the latter from power-on as
  well as from the states, twelve runs identical, the reference the interpreter as §5.6's runs have it;
- WiseMan's `MarsRtDebugTests.A_game_with_every_table_armed_is_the_game_run_plain`, through the shim and the
  registries, the reference the production configuration with its blocks: six runs identical. What the tables saw meanwhile, from the registries at the end of each run: 561, 547 and 548 million
instructions recorded and charged (Super Mario 64, Ocarina of Time, GoldenEye), 163, 117 and 408 million on the
signal processor, 5.4, 5.3 and 28.5 million store bytes watched on the page each game writes most (its 4 KB measured
over twenty frames: 17, 44 and 62 thousand bytes a frame), 688, 592 and 1,034 entry points, the data breakpoint's hit
count zero, and the stack at 470, 504 and 507 frames — near the registry's cap of 512, which is what a stack tracked
from an arbitrary instant and popped only by `jr ra` drifts to on a game whose threads and handlers leave by other
roads; the C# core's does the same from `LoadRom`.

*Exactness, halted and resumed.* A breakpoint at the general exception vector on every third frame, halted at and
resumed as a host resumes, the frames between run plain through the blocks; the unstopped run brought to each halt's
instruction count with `run_steps` and compared there, and at every frame's end in state, picture and sound:

- the crate's `a_game_halted_at_breakpoints_and_resumed_is_the_game_run_through`: Super Mario 64 535 halts, Ocarina
  of Time 2,538, GoldenEye 1,057 in 300 frames, 62, 119 and 115 mid-frame states compared, all identical, unthreaded
  and on four workers deferred;
- WiseMan's `A_game_halted_at_breakpoints_and_resumed_is_the_game_run_through`, the halts through
  `Breakpoints.ShouldBreak` and the resumes through `RunFrame`, the same games and modes, the same halt counts, all
  identical.

Two defects the resume proofs found and the claims did not, both in the first version: a resumed frame restarted
`_lastFrameCycles`' measure, so a frame that had stopped and gone on disagreed with the plain frame at its end in that
field alone (the crate's `a_frame_halted_and_resumed_is_the_frame_run_through`, frame 1, byte 20 of the state); and
the crate's halted game test compared a deferred picture with the frame before's, which is the rule for a subject
deferred against a reference that is not, where both machines there are deferred (frame 3). The first was the loop's
and is fixed by the `continuing` flag above; the second was the test's.

*The other suites.* The crate's 417 tests pass with `cargo test --release` (the games and the corpus skip without their
files) and `cargo clippy --all-targets` is clean. In WiseMan, the DianaOS filter's 721 tests, `MarsRtTests` (320),
`MarsRtEngineTests` (7), `HardwareLoadTests` (5), `MarsRtFrontendTests` (15) and `MarsRTViTests` pass. One frontend
test failed, as it should have: `Mars_s_debugger_commands_read_MarsRT_and_refuse_to_halt_it` pinned the behaviour this
stage retires, and is turned round as `…_halt_it_and_step_it` — a breakpoint added through `bp`, halted at, `step cpu`
landing on the delay slot, and `step rsp` still refused. Its old form had a second assertion that passed for the wrong
reason: `step main` failed because `main` names no processor (the main one is `cpu`) and is not a count, not because
the processor could not halt. `MarsRtThreadsTests` with the gameplay states was run at 100 frames a game and cut by the
run's fifty-minute bound after 26 of its cases, all passing; its comparisons against the threaded C# core take minutes
each and are outside this stage's blast radius, which is the observed loop the plain frame never enters. The corpus
(`n64-systemtest`) is not on this machine's path for WiseMan and did not run; it exercises `run_steps`, which this stage
did not change.

#### 6.5.4 Mutants

Five rules broken in turn, each one textual change, run against the crate's `tests::debug` (18) and WiseMan's
`MarsRtDebugTests` (33, the game proofs skipped):

| broken | crate | WiseMan |
| --- | --- | --- |
| a breakpoint fires after its instruction instead of before: the tables asked after the step, of the address that ran | six of the eighteen: `a_breakpoint_stops_in_front_of_its_instruction…`, `stopping_before_each_instruction…`, `a_call_is_pushed…`, `stepping_to_a_depth…`, `the_profiler_charges…`, `a_state_loaded…` | six of the thirty-three: the breakpoint, step, step-out, run-to-interrupt, halt-counters and state-at-a-halt claims |
| a return does not pop the stack: `note_return` logs the pop and leaves the frame | `a_call_is_pushed…popped_once_its_slot_has_run`, `stepping_to_a_depth…`, `a_state_loaded_keeps_the_tables_and_the_stack` | `Stepping_over_a_call_stops_after_it_in_the_caller`, `Inside_a_call_the_backtrace_names_it_and_stepping_out_returns_to_the_caller` |
| coverage recorded while disarmed: the bitmap kept and written whatever the flag says | `coverage_records_what_the_processor_ran_only_while_armed…` | none: **a survivor at the claims' level**, and why is worth stating. The shim drains the bitmap only while the registry is armed and `CoverageRegistry.Merge` refuses a merge while it is not, so a machine that records while disarmed is stopped twice before a claim could see it; the claim `Coverage_records_what_the_processor_ran_only_while_armed` holds against this mutant by the guards above it, and the crate's test is what holds the machine itself to the rule. |
| a data breakpoint halts before its store: the store reported before it lands, so the bytes reported are the old ones | `a_data_breakpoint_stops_after_the_store_that_wrote_it_and_a_watch_logs_every_byte`: the bytes logged are the old ones | `A_data_breakpoint_halts_after_the_store_that_wrote_it`: it halts with the old word in memory |
| a frame that stops but leaves the counters stale: the logs drained only at the field's end | none, by construction: the crate reads the machine's logs itself | `A_halt_leaves_the_coverage_and_the_profile_current`, `Inside_a_call_the_backtrace_names_it…` (an empty backtrace at the halt), `A_data_breakpoint_halts_after_the_store_that_wrote_it` (the pending note never reaches the registry, so it says no and the frame runs on) |

The fifth is the one this design makes possible and the C# core cannot have — the C# observers write into the
registries as the instruction runs — and it is the one the added claim was written for.

#### 6.5.5 Speed

*The plain frame must not slow down, and the tables must cost nothing while empty.* Measured with `examples/threads`
built before the hooks and after them, the two binaries interleaved and the order swapped each round, three rounds of
600 frames from each gameplay state, split four workers, deferred, blocks (the production configuration); the state
hash is the same in every run, so what is timed is exact.

| ms a frame, before → after | Super Mario 64 | Ocarina of Time | GoldenEye, the Dam |
| --- | --- | --- | --- |
| 1. flags tested on the shared step, the tables inline in `Cpu` | 5.90, 5.73, 5.72 → 5.77, 5.76, 5.77 | 6.79, 6.76, 6.77 → 6.79, 6.79, 6.80 | 14.34, 14.36, 14.33 → 14.48, 14.48, 14.47 |
| 2. the seams in the observed step alone, the tables still inline | 5.76, 5.73, 5.75 → 5.88, 5.89, 5.83 | 6.80, 6.76, 6.81 → 6.85, 6.81, 6.84 | 14.36, 14.41, 14.33 → 14.56, 15.03, 14.69 |
| 3. the seams in the observed step alone, the tables boxed | 5.73, 5.88, 5.69 → 5.72, 5.73, 5.72 | 6.78, 6.91, 6.81 → 6.78, 6.78, 6.80 | 14.42, 14.33, 14.31 → 14.34, 14.30, 14.35 |

**The prediction, and its retirement.** The first row cost GoldenEye one per cent in every round, and the prediction
was that the four flag tests on the shared paths were the cost, since the Dam's frame is mostly decoded blocks beside a
running signal processor and every one of their stores and jumps went through the tested handlers. The second row
retired it: with the seams compiled out of the plain step the Dam was no better, and Super Mario 64 was worse. The
third row found the cost: the tables — six vectors, a map, a bitmap's pointer, some three hundred bytes — were inline in
`Cpu`, which sits before the bus in `Machine`, and moving them shifted every hot field of the bus; boxed, the machine
keeps its shape and all three games are level within the rounds' noise (the Dam's rounds spread 0.1 ms in either
build). The split is kept although it measured nothing on its own: it costs one generic parameter in the source, and it
makes "the plain step carries no seam" true by construction rather than by a measurement that, as the second row shows,
cannot see a change smaller than a layout's.

*An observed frame's cost* was not measured on its own. The crate's armed comparisons — a plain interpreter and an
observed machine together, every frame's state saved and compared — take 13, 16 and 32 s for 300 frames of the three
states, so an observed frame is within an order of magnitude of a plain one; C#'s observed loop is slower still, and
neither is what a player runs. Nothing was done about it.

#### 6.5.6 What is not done

- **Breakpoints inside a block, watches on a tier-3 inline store, and coverage through the blocks**: the observed frame
  is the interpreter's, by the rule above. A debugger that wanted speed would need the blocks' dispatcher to consult
  the breakpoint table at each entry and the compiled code to call out for stores and calls, which is the C# core's
  design and its cost.
- **Reads** are not reported on either core (`Mars_Debug.md` §3), so `watch r`, `bp read` and the uninitialised-read
  check arm and never fire; the uninitialised-read check makes the target listen, as in C#, and its writes are not
  marked on MarsRT since nothing would read the marks.
- **Transfers** — the DMAs and the display processor's pixels — are not reported, as in C#.
- **The stack in a plain frame**, above.
- **`runto` a scanline** is not fed, as in C#.
- **Breakpoints on the RSP** (`Mars_Debug.md` §5): none on either core.
- **The dashboard's audio peek** (§5.5.4) is still empty.
- The interface is 7: `mars_debug_set`, `_set_breakpoints`, `_set_ranges`, `_run_frame`, `_writes`, `_calls`,
  `_profile`, `_coverage`, `_counters`, and the tests' `mars_machine_pc`, `_physical`, `_bus_write32`, `_bus_read32`,
  `_set_cop0`, `_mi_raise`, `_rsp_step`.

### 6.6 Stage F: parity leftovers

Each is small, has its own oracle, and can be given to an agent beside larger work.

- **ROM-patch cheats** (§5.5.4): the patch list held beside the cartridge reads in `memory/bus_access.rs`, pushed at
  the frame's end as the codes are. No N64 code format produces one, so this closes a hole a hand-made patch would
  fall through, not a game.
- **`IFrameProfiler` phases and the dashboard's audio peek** (§5.5.4): counters the shim reads once a frame.
- **Rewind for the N64** is off on both engines in Mistress, because the C# core's snapshot with several workers froze
  a game (`EmuSen_Settings_Reference.md` §4.21b, §4.44). MarsRT's snapshot is proven headlessly in every frame of six
  games (§5.6.4) and its pause point is a barrier the C# core lacks. Turning it on for MarsRT alone is a play decision
  of the same kind as stage B's, and the C# core's stays off until its failure is fixed. *The failure the tests could
  reach, the pause barrier's deadlock, was fixed in C# on 2026-09-23 (a72533c, `Mars_Rdp.md` §2.9.1); the freeze itself
  was never reproduced, so turning rewind back on is still a play decision.*
- ~~**The C# core's four failures** (§5.6.2, §5.6.5, §5.6.6): each has a WiseMan test that shows it and is marked as the
  C# core's; the work is to fix each in C# and turn its test around. The pause barrier's is the one that may be a
  game's freeze, and comes first.~~ *Done 2026-09-23: a72533c (the pause barrier), 7fbf4b1 (the range read), 419583a
  (the deferred repeat), 51f3377 (the fraction), and 05d41e4 for §6.4.4's race; `Mars_Rdp.md` §2.9.*
- **MarsRT's single-thread regression** of 1.5 to 3.5 per cent (§5.6.8), found and not explained: a bisection with the
  interleaved bench between the RDP merge and the threads' merge, about two hours, and either a cause or a recorded
  negative.

### 6.7 Stage G: the C# core

**The decision**, which is the project's and not this page's, has criteria this page can state. The C# core can be
retired from the Engine row when:

1. every published platform carries the library and MarsRT is the default there (B and C);
2. everything the debugger and the coverage recorder do runs on MarsRT (E);
3. a season of play has passed with no fault that was MarsRT's alone.

**What "retired" means.** Not deleted. Every exactness claim in §5 is a comparison against the C# core, and the
WiseMan suites that make those claims need it to build and run; `EmuSen_Stack.md`'s rule keeps a 2D core in C# and
says nothing against a reference. So the C# core stays as the oracle in the test harness for as long as MarsRT is
graded against it, and "retired" means it is no longer offered to a player. Its four failures (§6.6) matter less
then, but its exactness still does. *They were fixed on 2026-09-23.*

**The recommendation** is to keep it offered until 1 to 3 hold, and to keep it as the oracle indefinitely. A port
that removed its own reference would have to be graded against something else, and there is nothing else that is
exact to the frame.

### 6.8 What this plan does not cover

- Anything the recompiler could still gain (§5.8.9): the processor beside a block, the coprocessor's calls, chaining.
  Stage A says whether any of it is needed on the device that matters. *§6.9 measured where the frame goes and ranks
  what is left with a ceiling each: the processor's vector unit is first by a factor of two and a half, the processor
  beside a block second, and the presenter's join fourth.*
- The C# core's own remaining costs, which stop mattering once it is not what a player runs.
- A second 3D core. `EmuSen_Stack.md`'s rule says its language; nothing here says its design, and it should not be
  started until G is decided, so that the lessons of this port are the ones it starts from.

#### 6.9 Where a MarsRT frame goes (measured 2026-09-23)

§5.6.8 left one sentence open — "how the remaining 6.0 ms divides was not measured" — and §5.8.8 bounded only the
entry's fifth of the Dam's thread. §6.1 then found the handheld bound where the desktop is, on the emulation thread,
and named what is left there as "§5.8.9's list, the interpreter's own cost and the processor's" without a number for
any of them. This section is those numbers: the emulation thread of a production frame, sampled and divided into the
machine's components, for the three gameplay states, so that the levers left can be ranked by what each could buy at
most before any is built.

**The method.** An interrupt sampler of MarsRT's own (`rtsample.py`, kept with the speed tooling outside the
repository in `~/.cache/emusen/probe/mars-speed/rt-tools/`, with a README that repeats the run in two lines). It
spawns one of the crate's examples, waits a second for the load and the first frames, and then, about once a
millisecond, stops the emulation thread with `PTRACE_INTERRUPT`, reads its instruction pointer, and resumes it. An
address inside the example is attributed through the binary's own symbol table and `eu-addr2line`'s inline chain, so
that a function the compiler inlined is named by its innermost frame — `set_element` inside `vector_op`, `tick`
inside `decoded::run` — and a sample is grouped by the innermost frame that belongs to a component. An address in the
C library is attributed through the library's debuginfo, and, since a leaf such as `memcmp` or a system call keeps no
frame, the word at the stack pointer is read as its return address and names the caller: this is how the
comparison's `memcmp` is the dispatcher's and a `sched_yield` is the presenter's or a worker's. A compiled block's
address is attributed through the recompiler's perf map, by block and no finer. The runs are production's shape,
`examples/threads <rom> <state> 600 split 4 blocks` — four rasteriser workers, deferred presentation, tier 2 — twice
for each game, and the processor's own view, `examples/frames … 600 blocks` — one thread, no scan — once, plus two
runs with every thread stopped in turn: GoldenEye's workers, and Super Mario 64's six threads. The samples: 7,369 and 7,371 of the Dam's thread, 3,198 and 3,166
of Ocarina of Time's, 2,539 and 2,456 of Super Mario 64's, and 12,070, 5,266 and 5,120 on one thread. A row of a
percent is therefore twenty-five to seventy samples, and the standard error of the largest rows is half a point on
the Dam and a point on Super Mario 64; the two runs of each game agree to within that, and both are printed so that
the reader can see it rather than take it. The milliseconds are the shares multiplied by an unsampled run's frame,
taken in the same batch: 14.501, 6.778 and 5.801 ms threaded, 22.659, 10.661 and 10.396 on one thread. The six were
run twice more after the samples, on 2026-09-23 as another agent's test suite was finishing on the machine: 14.47 and
14.74, 6.84 and 7.84, 5.79 and 5.80 threaded; 22.32 and 22.94, 10.61 and 10.82, 10.40 and 10.41 on one thread. All but
Ocarina of Time's higher threaded run, taken first while the load was still falling, are within two per cent of the
batch's, and the batch's are the ones used.

**What the stop costs.** The sampled runs' own frames were 14.667 and 14.668 ms on the Dam, 7.229 and 7.185 on
Ocarina of Time and 6.111 and 5.962 on Super Mario 64 — one, six to seven and three to five per cent above the
unsampled frame. A stop costs about the same however long the frame is, so the light games pay more of it per frame, and the
first game's two runs agreeing to a microsecond says the machine was otherwise idle.

**What the attribution cannot do.** A sample inside the binary carries no caller, so a handler the compiled code calls
and the same handler a decoded block calls are one row; the split between them is bounded below from the counters
instead. A compiled block is a name and nothing inside it. A cache miss is charged to the instruction that waited for
it, not to whoever evicted the line — which is what an instruction-pointer sample measures, and is also the right
charge for a lever's ceiling, since the lever removes the waiter. The presenter's own thread was not sampled in the
main threaded runs, only its join on the emulation thread; one all-threads run of Super Mario 64 samples it. And the frame is the desktop's, with its 32 MB of L3 and
its idle cores: §6.1's device halves the cache, and the shares there are not measured.

**GoldenEye, the Dam** — the game that matters, at 90 per cent of its console on the device (§6.1). The emulation
thread, two runs, four workers, deferred, tier 2:

| component | run 1 % | ms | run 2 % | ms |
| --- | ---: | ---: | ---: | ---: |
| the interpreter: step, cop0, the rest of `Cpu` | 0.5 | 0.07 | 0.4 | 0.06 |
| compiled blocks: the code itself | 0.9 | 0.13 | 0.9 | 0.13 |
| decoded blocks: the loop | 7.0 | 1.02 | 6.9 | 1.00 |
| decoded blocks: the handlers | 7.6 | 1.11 | 7.5 | 1.09 |
| the dispatcher: lookup, shaping, the entry | 5.5 | 0.80 | 5.3 | 0.78 |
| the dispatcher: the entry comparison (`memcmp`) | 2.1 | 0.31 | 2.2 | 0.33 |
| RSP: the vector unit | 36.4 | 5.28 | 36.0 | 5.23 |
| RSP: the scalar unit and its loads and stores | 7.0 | 1.01 | 7.1 | 1.03 |
| RSP: decode and dispatch | 11.2 | 1.62 | 11.4 | 1.65 |
| RSP: the lock-step (`sp_step`, `tick`, `is_event`, `run`) | 5.6 | 0.81 | 5.8 | 0.85 |
| RSP: events (break, DMA, status) | 0.6 | 0.09 | 0.7 | 0.11 |
| CPU: the coprocessor (software float, moves, loads) | 3.7 | 0.53 | 3.5 | 0.50 |
| CPU: the TLB (`translate`) | 1.4 | 0.20 | 1.3 | 0.19 |
| CPU: the idle loop | 0.3 | 0.04 | 0.3 | 0.05 |
| RDP on this thread: marks, ranges, publish, shadow, `Take` | 3.7 | 0.54 | 3.7 | 0.54 |
| scan-out on this thread: prepare and the capture | 0.1 | 0.01 | 0.2 | 0.03 |
| scan-out on this thread: the presenter's join | 4.3 | 0.62 | 4.3 | 0.63 |
| bus and devices | 1.1 | 0.15 | 1.0 | 0.15 |
| the frame loop (`Machine::run_frame`) | 0.4 | 0.06 | 0.4 | 0.05 |
| other (allocation, the C library, unattributed) | 0.7 | 0.10 | 0.8 | 0.12 |
| **the frame** | 100 | 14.50 | 100 | 14.50 |
| samples | 7369 |  | 7371 |  |

*What the table says.*

- **The signal processor is 61 per cent of the frame, 8.8 ms.** Its vector unit alone is 5.25 ms, more than a third
  of the frame; the decode and dispatch of its instructions 1.6; the scalar unit with its loads and stores 1.0; the
  lock-step itself — `sp_step`, the tick, the `is_event` test and `run`'s loop — 0.8; its events 0.1. The Dam runs its
  processor the whole frame: by the C# core's counter on these frames (`pacebench`, 1.03 to 1.16 million steps a
  frame, and the two cores step it identically, §5.2) that is about 1.1 million steps, so the processor costs **8.0 ns
  a step**, of which the lock-step's own overhead is 0.75 ns and the instruction the rest. §3.4 measured MarsRT's
  interpreter at 7.9 ns a step on random programs and the C# blocks at about 3; the game confirms the first number
  and pays it on every step, since MarsRT compiles nothing for this processor.
- **The vector unit's hottest frame is a two-byte store.** `set_element` — one element of one register written — is 11
  per cent of the thread by itself, `element` 2.4, `set_acc` and `set_acc_low` 2, `clamp_signed` and `clamp_low` 2.
  §3 ports the C# processor's *plain* path, which the specification writes element by element; the C# core runs
  `Rsp.VectorSimd.cs`, the same arithmetic eight elements at a time, since `Mars_RspVector.md` §14. What the Dam's
  vector unit does is scalar work on a vector, and the samples say so.
- **The CPU is 30 per cent, 4.4 ms, and nearly all of it is the decoded tier.** The decoded blocks' loop is 1.0 ms and
  their handlers 1.1; the dispatcher's lookup, shaping and entry 0.8 and its comparison 0.3; the coprocessor 0.5; the
  TLB 0.2; the bus 0.15; compiled code 0.13; the interpreter proper, the idle loop and the frame loop 0.2 together.
  §5.8.8 counted two per cent of the Dam's block instructions compiled, because nearly every block is entered beside
  a running processor and §5.8.3 leaves those decoded; the profile is that count's price. The `lw` handler alone is
  1.2 to 1.5 per cent and `sw` 0.9, `addiu`, `sll` and `or` half a per cent each: the handler call is the cost of an
  instruction here, not the instruction.
- **Threading left 1.2 ms on the thread.** The display processor's marks, ranges, extents, shadow and `Take` are
  0.54 ms (`dp_take`, the words read from RDRAM into the ring, is 2.2 per cent by itself); the deferred presenter's
  join 0.63; the scan's own prepare and capture 0.02. The thread waits for the workers 0.000 ms, as the counters
  said in §5.6.8, and the rasteriser's own code does not appear on it.

The top twenty-five symbols of the first run, by containing function, so the grouping can be checked against the raw
count (a library symbol is followed by the caller the sampler read):

```
 32.08%  [threads] <marsrt::rsp::Rsp<marsrt::memory::sp::Lent>>::vector_op
 21.16%  [threads] marsrt::cpu::blocks::decoded::run
  7.27%  [threads] <marsrt::cpu::blocks::Blocks>::step
  5.36%  [threads] <marsrt::rsp::Rsp<marsrt::memory::sp::Lent>>::run
  4.30%  [libc.so.6] sched_yield  <- [threads] take
  3.49%  [threads] <marsrt::rsp::Rsp<marsrt::memory::sp::Lent>>::vector_load
  2.20%  [threads] <marsrt::memory::bus::MemoryBus>::dp_take
  2.14%  [libc.so.6] __memcmp_evex_movbe  <- [threads] equal_same_length<u8, u8>
  2.09%  [threads] <marsrt::rsp::Rsp<marsrt::memory::sp::Lent>>::vector_store
  1.45%  [threads] marsrt::cpu::blocks::ops::lw
  1.22%  [threads] <marsrt::rsp::Rsp<marsrt::memory::sp::Lent>>::add
  1.11%  [threads] <marsrt::rsp::Rsp<marsrt::memory::sp::Lent>>::compare
  1.03%  [threads] <marsrt::cpu::Cpu>::execute_format
  0.90%  [threads] marsrt::cpu::blocks::ops::sw
  0.88%  [threads] <marsrt::memory::bus::MemoryBus>::read32
  0.84%  [anon] ?
  0.69%  [threads] marsrt::cpu::blocks::ops::addiu
  0.69%  [threads] <marsrt::memory::bus::MemoryBus>::rsp_event
  0.62%  [threads] <marsrt::cpu::Cpu>::load_cop1
  0.62%  [threads] <marsrt::cpu::Cpu>::execute_cop1
  0.52%  [threads] marsrt::cpu::blocks::ops::sll
  0.49%  [threads] marsrt::cpu::blocks::ops::or
  0.41%  [threads] <marsrt::cpu::Cpu>::run_idle
  0.41%  [threads] <marsrt::machine::Machine>::run_frame
  0.41%  [threads] <marsrt::rsp::Rsp<marsrt::memory::sp::Lent>>::cop2
```

**Ocarina of Time**, the same shape, two runs:

| component | run 1 % | ms | run 2 % | ms |
| --- | ---: | ---: | ---: | ---: |
| the interpreter: step, cop0, the rest of `Cpu` | 0.4 | 0.03 | 0.4 | 0.03 |
| compiled blocks: the code itself | 4.0 | 0.27 | 4.0 | 0.27 |
| decoded blocks: the loop | 3.9 | 0.26 | 3.7 | 0.25 |
| decoded blocks: the handlers | 6.5 | 0.44 | 5.3 | 0.36 |
| the dispatcher: lookup, shaping, the entry | 6.8 | 0.46 | 8.1 | 0.55 |
| the dispatcher: the entry comparison (`memcmp`) | 2.4 | 0.17 | 1.7 | 0.12 |
| RSP: the vector unit | 35.2 | 2.39 | 36.9 | 2.50 |
| RSP: the scalar unit and its loads and stores | 6.2 | 0.42 | 5.2 | 0.35 |
| RSP: decode and dispatch | 8.2 | 0.56 | 7.7 | 0.53 |
| RSP: the lock-step (`sp_step`, `tick`, `is_event`, `run`) | 3.9 | 0.27 | 3.6 | 0.24 |
| RSP: events (break, DMA, status) | 0.3 | 0.02 | 0.4 | 0.03 |
| CPU: the coprocessor (software float, moves, loads) | 5.5 | 0.37 | 5.6 | 0.38 |
| CPU: the TLB (`translate`) | 0.3 | 0.02 | 0.3 | 0.02 |
| CPU: the idle loop | 0.3 | 0.02 | 0.6 | 0.04 |
| RDP on this thread: marks, ranges, publish, shadow, `Take` | 3.0 | 0.20 | 2.5 | 0.17 |
| scan-out on this thread: prepare and the capture | 0.2 | 0.01 | 0.1 | 0.01 |
| scan-out on this thread: the presenter's join | 9.9 | 0.67 | 10.4 | 0.71 |
| bus and devices | 1.5 | 0.10 | 1.8 | 0.12 |
| the frame loop (`Machine::run_frame`) | 0.3 | 0.02 | 0.2 | 0.01 |
| other (allocation, the C library, unattributed) | 1.1 | 0.07 | 1.3 | 0.09 |
| **the frame** | 100 | 6.78 | 100 | 6.78 |
| samples | 3198 |  | 3166 |  |

- **The processor is 54 per cent, 3.65 ms**, at 8.6 ns a step over the C# counter's 0.42 to 0.43 million steps a
  frame; its vector unit 2.4 ms, 36 per cent of the frame.
- **The presenter's join is the second component, 10 per cent, 0.7 ms.** The emulation thread stands in
  `Presenter::take`, spinning and then yielding, for the walk it handed over at the end of the frame before.
  §5.4.4 measured that walk at 4.5 ms for this game on one thread, and a frame that runs shorter than the walk
  before it — a field with no drawing — waits for the difference. §5.6.8 did not see this because it counted only the
  display processor's wait sites, and the presenter's join is not one of them.
- **The CPU is 32 per cent, 2.1 ms**, and here it is split: compiled code 0.27, the decoded tier 0.66 (a third of the
  instructions, §5.8.8), the dispatcher 0.65 of which the comparison 0.15, the coprocessor 0.37, the bus 0.1. The
  entry costs more than twice the compiled code it enters, which is `Mars_Recompiler.md` §9's finding in the other
  core.

```
 30.39%  [threads] <marsrt::rsp::Rsp<marsrt::memory::sp::Lent>>::vector_op
 11.32%  [threads] marsrt::cpu::blocks::decoded::run
  9.94%  [libc.so.6] sched_yield  <- [threads] take
  6.47%  [threads] <marsrt::cpu::blocks::Blocks>::step
  5.72%  [threads] <marsrt::rsp::Rsp<marsrt::memory::sp::Lent>>::run
  4.88%  [threads] <marsrt::rsp::Rsp<marsrt::memory::sp::Lent>>::vector_load
  2.44%  [libc.so.6] __memcmp_evex_movbe  <- [threads] equal_same_length<u8, u8>
  1.91%  [threads] <marsrt::cpu::Cpu>::execute_format
  1.56%  [threads] <marsrt::rsp::Rsp<marsrt::memory::sp::Lent>>::vector_store
  1.56%  [threads] <marsrt::rsp::Rsp<marsrt::memory::sp::Lent>>::add
  1.53%  [threads] <marsrt::memory::bus::MemoryBus>::dp_take
  1.00%  [threads] marsrt::cpu::blocks::compiled
  0.94%  [threads] marsrt::cpu::blocks::ops::lw
  0.94%  [threads] <marsrt::memory::bus::MemoryBus>::read32
  0.81%  [threads] marsrt::cpu::blocks::ops::sw
  0.75%  [threads] <marsrt::rsp::Rsp<marsrt::memory::sp::Lent>>::fraction
  0.69%  [threads] <marsrt::cpu::Cpu>::run_idle
  0.63%  [threads] <marsrt::cpu::Cpu>::load_cop1
  0.63%  [threads] <marsrt::cpu::Cpu>::execute_cop1
  0.50%  [threads] marsrt::cpu::blocks::ops::cop1
  0.50%  [threads] <marsrt::memory::bus::MemoryBus>::write32
  0.50%  [threads] marsrt::cpu::blocks::ops::sll
  0.47%  [threads] <marsrt::rsp::Rsp<marsrt::memory::sp::Lent>>::compare
  0.44%  [threads] <marsrt::memory::bus::MemoryBus>::rsp_event
  0.44%  [threads] marsrt::cpu::blocks::ops::lbu
```

**Super Mario 64**, two runs:

| component | run 1 % | ms | run 2 % | ms |
| --- | ---: | ---: | ---: | ---: |
| the interpreter: step, cop0, the rest of `Cpu` | 0.0 | 0.00 | 0.2 | 0.01 |
| compiled blocks: the code itself | 3.2 | 0.18 | 4.0 | 0.23 |
| decoded blocks: the loop | 1.9 | 0.11 | 1.9 | 0.11 |
| decoded blocks: the handlers | 2.9 | 0.17 | 2.8 | 0.16 |
| the dispatcher: lookup, shaping, the entry | 3.7 | 0.21 | 4.1 | 0.24 |
| the dispatcher: the entry comparison (`memcmp`) | 1.6 | 0.10 | 1.2 | 0.07 |
| RSP: the vector unit | 41.4 | 2.40 | 42.2 | 2.45 |
| RSP: the scalar unit and its loads and stores | 5.9 | 0.34 | 5.7 | 0.33 |
| RSP: decode and dispatch | 8.4 | 0.49 | 8.1 | 0.47 |
| RSP: the lock-step (`sp_step`, `tick`, `is_event`, `run`) | 4.7 | 0.27 | 3.9 | 0.22 |
| RSP: events (break, DMA, status) | 0.3 | 0.02 | 0.7 | 0.04 |
| CPU: the coprocessor (software float, moves, loads) | 3.0 | 0.17 | 2.9 | 0.17 |
| CPU: the TLB (`translate`) | 0.0 | 0.00 | 0.2 | 0.01 |
| CPU: the idle loop | 0.3 | 0.02 | 0.7 | 0.04 |
| RDP on this thread: marks, ranges, publish, shadow, `Take` | 2.8 | 0.16 | 2.2 | 0.13 |
| RDP on this thread: waiting for the workers | 0.2 | 0.01 | 0.1 | 0.01 |
| scan-out on this thread: prepare and the capture | 0.5 | 0.03 | 0.4 | 0.02 |
| scan-out on this thread: the presenter's join | 16.9 | 0.98 | 17.1 | 0.99 |
| bus and devices | 0.7 | 0.04 | 0.8 | 0.04 |
| the frame loop (`Machine::run_frame`) | 0.2 | 0.01 | 0.1 | 0.01 |
| other (allocation, the C library, unattributed) | 1.4 | 0.08 | 1.1 | 0.06 |
| **the frame** | 100 | 5.80 | 100 | 5.80 |
| samples | 2539 |  | 2456 |  |

- **The processor is 60 per cent, 3.5 ms**, at 6.8 ns a step over 0.52 million; the vector unit 2.4 ms, 42 per
  cent of the frame, the largest share it has in any of the three.
- **The presenter's join is 17 per cent, 1.0 ms**, the largest thing on the thread after the processor. The
  all-threads run below found the presenter busy 1.4 ms a frame on average, so the emulation thread waits for about
  seventy per cent of the walk it handed over: the walk overlaps the next frame's emulation very little, though the
  mean frame is four times as long as the mean walk. The per-frame times that would say which frames are short
  enough to wait were not recorded, and the reason is not measured.
- **The CPU is 18 per cent, 1.1 ms**: compiled code 0.21, the decoded tier 0.28, the dispatcher 0.31, the
  coprocessor 0.17, the rest 0.1. There is little left to compile here; what the frame has left is the processor
  and the join.

```
 35.57%  [threads] <marsrt::rsp::Rsp<marsrt::memory::sp::Lent>>::vector_op
 16.94%  [libc.so.6] sched_yield  <- [threads] take
 10.75%  [threads] <marsrt::rsp::Rsp<marsrt::memory::sp::Lent>>::run
  5.16%  [threads] marsrt::cpu::blocks::decoded::run
  4.06%  [threads] <marsrt::rsp::Rsp<marsrt::memory::sp::Lent>>::vector_load
  3.51%  [threads] <marsrt::cpu::blocks::Blocks>::step
  2.32%  [threads] <marsrt::rsp::Rsp<marsrt::memory::sp::Lent>>::add
  1.65%  [libc.so.6] __memcmp_evex_movbe  <- [threads] equal_same_length<u8, u8>
  1.65%  [threads] <marsrt::memory::bus::MemoryBus>::dp_take
  1.54%  [threads] <marsrt::rsp::Rsp<marsrt::memory::sp::Lent>>::vector_store
  1.26%  [threads] <marsrt::rsp::Rsp<marsrt::memory::sp::Lent>>::compare
  1.10%  [threads] <marsrt::rsp::Rsp<marsrt::memory::sp::Lent>>::reciprocate
  0.95%  [threads] <marsrt::cpu::Cpu>::execute_format
  0.63%  [threads] <marsrt::cpu::Cpu>::run_idle
  0.55%  [anon] ?
  0.51%  [threads] <marsrt::cpu::Cpu>::execute_cop1
  0.51%  [threads] <marsrt::cpu::Cpu>::load_cop1
  0.47%  [threads] marsrt::cpu::blocks::compiled
  0.47%  [threads] marsrt::cpu::blocks::ops::sw
  0.43%  [threads] <marsrt::memory::bus::MemoryBus>::read32
  0.43%  [threads] <marsrt::memory::bus::MemoryBus>::rsp_event
  0.39%  [threads] <marsrt::cpu::Cpu>::store_cop1
  0.35%  [threads] <marsrt::memory::dp_threads::Threads>::wait_range
  0.35%  [threads] threads::main
  0.32%  [threads] marsrt::cpu::blocks::ops::sh
```

**The coprocessor and the TLB, and how much of each is compiled code's calls.** The rows are the buckets above; the
split by caller is not observable (above), so it is bounded from §5.8.8's counters by supposing the calls follow the
instructions:

| ms a frame | GoldenEye | Ocarina of Time | Super Mario 64 |
| --- | --- | --- | --- |
| the coprocessor: `execute_format`, `execute_cop1`, the software float, its branch | 0.52 (3.6%) | 0.37 (5.6%) | 0.17 (2.9%) |
| of which the loads and stores into it (`load_cop1`, `store_cop1`) | 0.11 | 0.07 | 0.04 |
| the TLB: `translate`, `try_translate`, the mapped fetch | 0.20 (1.4%) | 0.02 | 0.01 |
| block instructions run as compiled code (§5.8.8) | 2% | 66% | 70% |
| so the coprocessor's calls from compiled code, if they follow the instructions | 0.01 | 0.25 | 0.12 |

§5.8.9's "the coprocessor is a call" costs, then, a quarter of a millisecond in Ocarina of Time and a tenth in Super
Mario 64, and the loads C#'s `Mars_Recompiler.md` §16 inlines and MarsRT does not are 0.04 to 0.11 ms. The TLB is a
cost in GoldenEye alone, where the code is mapped, and there it is already blocks behind the TLB (§5.8, ported from
`Mars_Recompiler.md` §17): the 0.2 ms is the loads and stores through it, which call the interpreter.

**What threading left on the emulation thread**, in milliseconds, against what it took off:

| ms a frame | GoldenEye | Ocarina of Time | Super Mario 64 |
| --- | --- | --- | --- |
| the display processor's marks, ranges, extents, shadow and `Take` | 0.54 | 0.18 | 0.15 |
| the scan's prepare and capture | 0.02 | 0.01 | 0.03 |
| the presenter's join | 0.63 | 0.69 | 0.99 |
| **left on the thread** | **1.19** | **0.88** | **1.17** |
| the rasteriser, on one thread (below) | 9.19 | 4.66 | 5.85 |
| the walk and composition, on one thread (§5.4.4) | 3.16 | 4.48 | 2.85 |

This is the number §5.6.8's retired prediction lacked: the subtraction assumed nothing was left, and 0.9 to 1.2 ms
is, two thirds to five sixths of it the join. The marks and the shadow, which that section named as the suspects,
are the smaller part.

**The unthreaded frame**, once, for contrast — `examples/frames`, the rasteriser on the emulation thread, the
picture not scanned:

| component | GoldenEye % | ms | Ocarina of Time % | ms | Super Mario 64 % | ms |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| the interpreter: step, cop0, the rest of `Cpu` | 0.2 | 0.05 | 0.3 | 0.03 | 0.1 | 0.01 |
| compiled blocks: the code itself | 0.1 | 0.01 | 2.7 | 0.29 | 1.3 | 0.13 |
| decoded blocks: the loop | 4.7 | 1.05 | 2.7 | 0.29 | 1.3 | 0.13 |
| decoded blocks: the handlers | 5.3 | 1.19 | 3.3 | 0.35 | 1.6 | 0.17 |
| the dispatcher: lookup, shaping, the entry | 3.7 | 0.83 | 4.8 | 0.51 | 2.5 | 0.26 |
| the dispatcher: the entry comparison (`memcmp`) | 1.2 | 0.27 | 1.5 | 0.16 | 0.4 | 0.04 |
| RSP: the vector unit | 23.0 | 5.21 | 23.8 | 2.54 | 24.0 | 2.49 |
| RSP: the scalar unit and its loads and stores | 4.4 | 0.99 | 3.6 | 0.39 | 2.8 | 0.29 |
| RSP: decode and dispatch | 6.8 | 1.54 | 4.5 | 0.48 | 3.6 | 0.37 |
| RSP: the lock-step (`sp_step`, `tick`, `is_event`, `run`) | 4.0 | 0.92 | 2.8 | 0.30 | 2.0 | 0.21 |
| RSP: events (break, DMA, status) | 0.4 | 0.08 | 0.2 | 0.02 | 0.3 | 0.03 |
| CPU: the coprocessor (software float, moves, loads) | 2.5 | 0.57 | 3.6 | 0.39 | 1.4 | 0.14 |
| CPU: the TLB (`translate`) | 0.8 | 0.18 | 0.0 | 0.00 | 0.1 | 0.01 |
| CPU: the idle loop | 0.3 | 0.06 | 0.1 | 0.01 | 0.5 | 0.05 |
| RDP on this thread: marks, ranges, publish, shadow, `Take` | 1.0 | 0.22 | 0.8 | 0.09 | 0.5 | 0.05 |
| RDP: the rasteriser (unthreaded shape only) | 40.6 | 9.19 | 43.7 | 4.66 | 56.3 | 5.85 |
| bus and devices | 0.5 | 0.11 | 0.7 | 0.07 | 0.6 | 0.06 |
| the frame loop (`Machine::run_frame`) | 0.3 | 0.06 | 0.2 | 0.02 | 0.1 | 0.01 |
| other (allocation, the C library, unattributed) | 0.5 | 0.11 | 0.6 | 0.07 | 0.6 | 0.07 |
| **the frame** | 100 | 22.66 | 100 | 10.66 | 100 | 10.40 |
| samples | 12070 |  | 5266 |  | 5120 |  |

The rasteriser is 41, 44 and 56 per cent of the one-thread frame; the split takes all of it off, at four workers,
and the join and the marks are what it costs. The processor's milliseconds are the same in both shapes — 8.7
against 8.8, 3.7 against 3.65, 3.4 against 3.5 — which is what they should be, since the processor does the same
steps whatever thread the rasteriser is on, and it is the check that the sampler's milliseconds are a measurement
and not an artefact of the shape sampled.

**The workers**, from the GoldenEye run with the workers' and the compiler's threads stopped in turn (`w1`, its own
frame 15.14 ms): each of the four rasteriser threads spends 20 per cent of the frame rasterising — 3.0 to 3.1 ms, against the
counter's "first worker busy" 3.41 to 3.49 — 58 per cent spinning in `dp_threads::sleep` (400 rounds of fifty
pauses before it parks) and 18 per cent parked in the futex; the compiler thread is parked 98 per cent of the time
and compiles in the rest. The Dam's four workers are busy a fifth of the frame each, so the split has three times
the rasteriser it needs here, and four cores spin for nine milliseconds a frame each waiting for words: nothing on
the desktop, and a power-budget question on §6.1's device, where the workers share the package with the thread that
is the bound. That run did not reach the presenter. A second, of Super Mario 64, where the join is largest, stopped all six
threads 2,431 to 2,435 times each (`w2`, fifteen seconds): the emulation thread's shares were those of the two main
runs within a point and a half; the presenter walked (`Walker::sample`, `walk`, `remembered`) and composed for 24 per
cent of the frame, 1.4 ms, and was parked for the other 76; the workers were as the Dam's, parked or spinning for
most of it.

**The prediction for the processor's share, and its fate.** The agent that built the sampler and took these runs
was stopped before writing them up, and no prediction of its survives in the results directory; this write-up
cannot reproduce one it did not see, and a prediction stated after the tables are read is not a prediction. What
the record allowed one to expect before the sample is nonetheless worth stating, because the sample tests it:
§3.4 priced MarsRT's processor at 7.9 to 15 ns a step and the C# blocks at 3, and `Mars_Rsp.md` §13 and
`Mars_Recompiler.md` §9 and `Mars_RspVector.md` §14 put the processor at 15 to 36 per cent of the C# thread *with* those blocks running 60 to
90 per cent of its steps; so MarsRT, interpreting every step, should have had the processor as its largest component
at roughly twice the C# share, and 8 ns times the step count should have given the milliseconds. Both hold: 54 to 61
per cent against C#'s 15 to 36, and 6.8 to 8.6 ns a step against the bench's 7.9. It is recorded as a consistency
check of §3.4's price against a game's step count, which is what it is, and not as a forecast retired.

**The levers, ranked, with a ceiling each.** The ceiling is the frame if the component cost nothing — "at most",
since no lever makes its component free — from the two-run means above. They are ranked by the milliseconds a
frame they would free summed over the three games, except fastmem, whose ceiling is the whole of a row it is only a
part of (below). The last column is what the C# history already measured for the same lever, where it did.

| # | if this were free | the Dam, from 14.50 | Ocarina, from 6.78 | Mario, from 5.80 | priced by the C# history? |
| --- | --- | --- | --- | --- | --- |
| 1 | the vector unit (5.25, 2.44, 2.43 ms) | 9.3 | 4.3 | 3.4 | `Mars_RspVector.md` §14: the same arithmetic in host vectors bought C# 4, 19 and 22 per cent with the processor at 27 to 36 per cent of its thread; here the vector unit alone is 36 to 42, and the plain path it would be checked against is what MarsRT already runs. Not a recompiler lever, and not measured in Rust. *Built in §6.10: 14.35 to 11.67, 6.75 to 5.57, 5.74 to 4.58 ms, the unit a sixth of each frame.* |
| 2 | the decoded tier (2.11, 0.66, 0.28) | 12.4 | 6.1 | 5.5 | §5.8.3 compiled the blocks entered beside a running processor and measured the Dam two per cent worse; the batch that would pay for it is priced at 0.3 ms and not built. Compiled code runs twice the decoded tier's instructions for 0.27 ms in Ocarina of Time, so the tier compiled at that rate would leave about 0.4 of the Dam's 2.1; the ceiling is the whole 2.1. |
| 3 | the processor's decode and dispatch (1.64, 0.54, 0.48) | 12.9 | 6.2 | 5.3 | `Mars_Rsp.md` §11 to §12: straight lines compiled with the handlers folded bought C# a quarter to a third of its thread, and §11.2's two tiers made them cheap enough for the device; §3.4 says why a Rust interpreter at 8 ns loses to them at 3. Not built in Rust. *Built in §6.12 as a decoded table, C#'s first tier: 11.69 to 10.44, 5.59 to 5.42, 4.57 to 4.47 ms; the Cranelift tier priced at 0.85 ms at most for the Dam and not built.* |
| 4 | the presenter's join (0.63, 0.69, 0.99) | 13.9 | 6.1 | 4.8 | Not priced for MarsRT: §5.6.8's counters could not see it. The C# core walks in four bands at 1.8 to 2.9 ms (§5.4.4) against MarsRT's one thread at 2.9 to 4.5, and a frame that is shorter than the walk before it waits; bands, a cheaper walk, or a second frame of latency would each take it, the last a play decision. *Built in §6.11: the join waited because the light field after a new picture is a quarter of the walk; the walk in four bands took Mario 4.69 to 3.65, Ocarina 5.70 to 4.79 and the Dam 11.65 to 11.03 ms, the join 0.02 to 0.2 ms.* |
| 5 | the dispatcher, comparison included (1.11, 0.65, 0.31) | 13.4 | 6.1 | 5.5 | `Mars_Recompiler.md` §9 to §12: the entry is memory, and extension, density and chaining measured nothing in C#; §5.8.8 measured the comparison removed at 0.1 to 0.35 ms and two rewrites at nothing. |
| 6 | the lock-step's own overhead (0.83, 0.26, 0.25) | 13.7 | 6.5 | 5.5 | §5.8.3's batch, 0.3 ms priced; `Mars_Rsp.md` §13's two routes, one slower and one a two per cent ceiling with an undo, neither kept. |
| 7 | the coprocessor (0.52, 0.37, 0.17) | 14.0 | 6.4 | 5.6 | `Mars_Recompiler.md` §16 declined a host-float fast case — the inexact flag bit for bit — and inlined the two loads, which here are 0.04 to 0.11 ms; §5.8.9 leaves the call. |
| 8 | the display processor's share of the thread (0.54, 0.18, 0.15) | 14.0 | 6.6 | 5.65 | `Mars_Performance.md` §28 priced C#'s sites; §5.6.8 said MarsRT's was not taken apart, and `dp_take` is the largest piece of it. |
| 9 | the TLB (0.20, 0.02, 0.01) | 14.3 | 6.76 | 5.79 | `Mars_Recompiler.md` §17's mapped blocks are already ported; what is left is the loads through it. |
| 10 | fastmem: every compiled load and store a host access (at most the compiled code's whole share: 0.13, 0.27, 0.21) | 14.4 | 6.5 | 5.6 | Not priced by the C# history, which could not take a page fault in managed code; priced here by the shares alone, below. |

**The verdict on fastmem.** A host-mapped guest memory — RDRAM reserved at a fixed offset from a base so that a
compiled load is one host instruction and a fault handler catches the rest — could serve only the loads and stores
the compiler emits, and those live inside the "compiled blocks" row: 0.13, 0.27 and 0.21 ms a frame in all, the
arithmetic, the branches and the entry's context included. The decoded tier's loads go through the interpreter's
handler and would not change, and the Dam's compiled share is two per cent, so its ceiling there is a hundredth of a
millisecond until §5.8.3's problem is solved. The inline fast case that fastmem would replace (§5.8.2) tests the
segment, the bounds, the alignment and then **the page's mark** — one 64-bit load from the 2,048-entry table the
threaded display processor keeps per 4 KB page, and a compare against zero (`emit.rs`, `direct`) — and takes the
slow call when any fails. The mark is the protocol §5.6.1 rests on: a page the drain is writing is read only after an
Acquire of `completed` past the word that writes it, and the mark is how a load knows to wait. A host mapping must
either keep that test, in which case it removes the segment, bounds and alignment tests — a handful of register
operations — and not the mark's load, which is the part that can miss, or replace it with page protection — `mprotect` of every marked page when its mark is set and again when it
clears, a system call and a TLB shootdown across the five threads each time, at every draw batch — which would put
on the thread a cost that is not on it today. So: fastmem's ceiling is under 0.3 ms a frame on every game, less
than the presenter's join alone, its realistic value a fraction of that, and it carries the one change this page's
threading cannot afford. It is ranked last and not recommended.

**What is predicted, to be retired by measurement.** The first lever and the fourth are the ones this section says to
build next — the second is §5.8.3's, measured a loss once and waiting on the batch, and the third is a compiler for
the processor, `Mars_Rsp.md` §11's whole design over again — and the prediction for each is stated here so it can be wrong in writing: a vector unit in host vectors,
exact against the plain path as §3 requires, takes the Dam from 14.5 to between 10 and 12 ms and the other two by
1 to 1.5 ms each — the vector unit's share is not all arithmetic, and C#'s §14 did not make its unit free (*held,
§6.10: 11.67 ms, and 1.18 and 1.15 off the others*); and
either bands or a deeper queue for the presenter take Super Mario 64 from 5.8 to about 5.0 and Ocarina of Time to
about 6.2, with the Dam gaining half a millisecond (*retired, §6.11.5: the bands took 1.04, 0.92 and 0.62 ms from the frames §6.10 left, more than predicted on every game, because the join depends on the light field and not on the mean frame this prediction priced it from*). Held as §3.4 taught, since neither is measured in Rust.

**What the numbers cannot say.**

- The three states run with no input, as the examples run them (§5.8.8), and one state is not a sample of a game:
  the Dam's compiled share under the WiseMan comparisons, where buttons are pressed, is thirty times this run's.
- Nothing inside a compiled block: the perf map names the block, and whether its time is its loads, its arithmetic
  or its entry is not in these samples. Item 10's ceiling is the whole row for that reason.
- Which caller a shared handler was called from — the coprocessor's table above is a proportion, not a count.
- The processor's step counts are the C# core's counter on the same frames; the two cores step the processor
  identically (§5.2), so the count is MarsRT's, but MarsRT does not yet print it.
- The presenter's own thread was sampled once, in one game, and not at the Dam or in Ocarina of Time.
- The device. §6.1 found the handheld bound where the desktop is, with half the cache; the *order* of the shares
  should carry over and their sizes need not, since a cache miss charged to `set_element` here is a larger miss
  there. Repeating the batch on the device is the two lines the README gives, once its Python has `elfutils`.
- Anything under half a per cent: those rows are ten to thirty samples.

#### 6.10 The vector unit in host vectors (2026-09-23)

§6.9 ranked the vector unit first of the levers left on the emulation thread — 5.25, 2.44 and 2.43 ms of the three
gameplay frames, a third of each, with `set_element`, one element of one register written, the hottest frame in it —
and stated a prediction for it: a vector unit in host vectors, exact against the plain path, would take the Dam from
14.5 ms to between 10 and 12, and the other two games down by 1 to 1.5 ms. This section is that unit, the evidence
that it is exact, and the measurement that tests the prediction. **The prediction held**: 11.67 ms at the Dam, 1.18
and 1.15 ms off the other two.

**The claim.** `src/rsp/simd.rs` runs the vector unit's arithmetic eight elements at a time in SSE2, SSSE3 and
SSE4.1, beside the element-by-element unit of §3, which is unchanged and is its oracle. Every vector operation and
every vector load and store leaves the processor, DMEM and the machine exactly as the element-by-element unit leaves
them: registers, the accumulator to its last bit, VCO, VCC, VCE and the divide unit.

**The layout, changed once for both paths.** The C# twin of §3 kept the register file as 256 elements and the
accumulator as eight 64-bit words, the save state's shapes. MarsRT's processor now keeps the register file as
thirty-two lanes of eight elements, element 0 first, so that a register is one 128-bit load and one store, and the
accumulator as three thirds of eight lanes — high, middle and low sixteen bits — as `Mars_RspVector.md` §15 keeps the
C# eight-lane unit's and as every fast reference keeps theirs, because SSE has no 64-bit arithmetic shift and no
64-bit compare, and a 48-bit value in thirds needs neither. The state is not changed: it still carries C#'s eight
words, widened from the thirds when written and narrowed when read (`rsp::widen`, `rsp::narrow`), and every state on
disk loads. The element-by-element unit reads and writes the same thirds through the `Memory` trait — `acc` composes a
word from them and `set_acc` splits one — so it computes exactly what it computed on the words.

*The bits above 47, found by the C# oracle and not by the crate.* A state no machine wrote can carry noise above bit
47 of an accumulator word, and `MarsNativeStateTests.Every_field_filled_with_noise_comes_back_from_rust_byte_for_byte`
fills every field with it and requires the state back byte for byte. The first version of the layout dropped those
bits at a load, and all eight of that test's cases failed, while the crate's processor tests, which draw no such
bits, passed (its full suite was not run on that version). They are
now a fourth array beside the thirds, `accumulator_top`, which no instruction reads. It follows the words' old rule
exactly: an instruction that writes a lane's whole accumulator (`accumulate`, and the eight-lane unit's
`set_thirds`) clears them, and one that writes only the low sixteen bits (`set_acc_low`, the unit's `set_low`) keeps
them. The random programs below give a quarter of their seeds noise there, and a mutant that keeps the bits through a
whole write is caught (the mutant table).

**What runs eight lanes at a time**, and how:

- *The element selector* is one `pshufb` with a key from a table of the sixteen patterns built at compile time from
  `Mars_RspVector.md` §2's rule; selectors 0 and 1 skip it. Both sources and the destination are loaded whole before
  anything is written, so reading before writing holds by construction.
- *The multiplies* are `pmullw` with `pmulhw` for signed by signed and `pmulhuw` for unsigned by unsigned; the mixed
  families (`VMUDM`/`VMADM` signed by unsigned, `VMUDN`/`VMADN` the other way round) take the unsigned high word less
  the unsigned operand where the signed one is negative, which is the exact thirty-two-bit product since every mixed
  product fits thirty-two signed bits. The fraction multiplies double the product in thirds with the sign taken
  before the doubling, which is right for the one product past thirty-one bits, `0x8000 × 0x8000`; the plain forms
  add `0x8000` as an exclusive-or of bit 15 with its carry propagated. The accumulating forms add in thirds with an
  unsigned carry from each third and the carry of a carry from the middle (`add48`), and there is no fourth third,
  which is the wrap of `Mars_RspVector.md` §6.1.
- *The clamps* are the three shapes of `Mars_RspVector.md` §7: the signed clamp of bits 47:16 is one `packssdw` of the
  middle and high thirds interleaved; the low clamp keeps the low third while the high third equals the middle's sign
  and is 0 or all ones by the high third's sign otherwise; the unsigned clamp is zero below zero and all ones once the
  high third is positive or zero with the middle's top bit set. The quarter multiplies shift the interleaved pair right
  one before the pack and mask the low four bits. `VRNDP`, `VRNDN` and `VMACQ` add a masked addend through `add48`.
- *The adds.* `VADD` saturates `min(s, t) + carry` then adds `max(s, t)`, since the smaller operand plus one saturates
  only when both are the largest positive value; `VSUB` saturates `t + carry` and corrects by one where that
  saturated; the accumulator's low third takes the unclamped sums. `VABS` is an and-not, an exclusive-or with the
  sign and a saturating subtract, which maps `-0x8000` to `0x7FFF` and leaves the accumulator the wrapped negation.
  `VADDC` and `VSUBC` take their carries and borrows from `paddusw`/`psubusw` against the wrapped result.
- *The compares, clips and merge* are lane masks: the flags are unpacked from VCO, VCC and VCE into masks by one
  broadcast, an and and a compare against the eight bit weights, and packed back by `packsswb` and `pmovmskb`. The
  choices are `pblendvb`. `VCL` updates a lane's less-or-equal only where the signs differed and not-equal was clear,
  and its greater-or-equal only where neither was set, as §8 of that page says.
- *The logic, `VSAR` and the nineteen functions that zero `vd`* are one or two instructions each.
- *The loads and stores of the byte to quad formats* (`LBV` to `LQV`, `SBV` to `SQV`) move whole elements where they
  do — an even element and an even count, and for a store no wrap inside the register — as one sixteen-byte read of
  DMEM, a byte exchange by `pshufb` and a masked blend into the register, or the register's bytes blended into
  DMEM's and written back. Where those sixteen bytes would leave DMEM the same move is taken element by element, with
  DMEM's wrap left to `data`'s mask, as the byte loop leaves it.

**What stays element by element, and why.** The six reciprocal instructions and `VMOV`: each reads one element, looks
up a table or moves one lane, and the eight-lane part of them — the shuffled `vt` into the accumulator's low third —
is one store; the SIMD path calls the element-by-element code for the rest, as the C# unit does (§14 of that page)
and as all three references do. The other thirteen load and store formats (`LRV`, `LPV`, `LUV`, `LHV`, `LFV`, `LTV`,
and `SRV`, `SPV`, `SUV`, `SHV`, `SFV`, `SWV`, `STV`), which rotate, widen, narrow or spread across registers: each
would be a shuffle table of its own, and how much of the games' traffic they are was not counted. `MFC2`, `MTC2`, `CFC2` and `CTC2`, which move sixteen bits. And the dispatch: the function's switch of
sixty-four is a jump table in both paths, and §6.9's decode row is untouched by this section.

**The switch.** `Memory::simd` says which path runs; it is true only where `rsp::simd_supported` found SSSE3 and
SSE4.1 at run time, and the SIMD functions carry `#[target_feature]` so they compile for any x86-64 and run only there.
The default is on, `EMUSEN_MARSRT_RSP_SIMD=0` turns it off for a process, `Machine::set_rsp_simd` for a machine (it
survives a state load, and is in no state and no equality), and `mars_rsp_set_simd` for the C# twin's component. On
aarch64 and every other host the module is not compiled and the element-by-element unit runs, as it did.

##### 6.10.1 The evidence

Five oracles, the first four the element-by-element unit.

1. **Random programs, compared after every step** (`tests::rsp`,
   `the_simd_vector_unit_leaves_the_state_the_element_by_element_one_leaves`). 150 seeds, each 1,024 instruction
   words — 40 per cent vector operations of every function and selector, 18 per cent vector loads and stores of every
   format including the four that do not exist, 6 per cent COP2 moves, and the scalar ALU, loads, stores, branches,
   breaks and COP0 reads — over registers drawn half at random and half from the edges (0, 1, 2, `0x7FFE`, `0x7FFF`,
   `0x8000`, `0x8001`, `0xFFFE`, `0xFFFF`, `0x00FF`, `0xFF00`, `0x0100`), each accumulator third drawn the same way,
   the flags a third of the time from their edges, the divide unit random, and a third of the scalar registers an
   address within seventeen bytes of an alignment or of DMEM's end. Two machines step 4,000 times from one state
   through `sp_step`, the machine's own entry, restarting at every break as `MarsNativeRspTests` restarts; after
   every step the two processors and the two DMEMs are compared, and at the end the two machines' whole states.
   600,000 steps: 221,664 vector operations, 108,846 vector loads and stores, 11,863 breaks. Identical.
2. **Every function at every selector**, twelve rounds each from fresh edge-drawn state, a round in three naming one
   register twice or all three the same: 12,288 single instructions, identical.
3. **The whole-element loads and stores at the edges**: both directions, the five formats, every element, every
   address of DMEM's last 48 bytes and first 32 — which is where the sixteen-byte window leaves DMEM and where a store
   would wrap the register — 12,800 instructions, identical.
4. **The clamps' boundaries** (`the_accumulating_multiplies_agree_where_each_clamp_turns`): the six accumulating
   multiplies, both rounds and `VMACQ`, with each lane's accumulator set so that the sum lands on one of twelve
   boundaries of the four clamps, or one either side — 0, `0x7FFF_0000`, `0x7FFF_FFFF`, `0x8000_0000`, `0xFFFF_FFFF`,
   2³², the largest and smallest 48-bit values and the negative mirrors — 18,000 instructions, identical. This test
   exists because of mutant 7 below.
5. **Against the C# core, and the games.** `MarsNativeRspTests`, C#'s plain and SIMD processors against MarsRT's
   through the component interface over the three entry points, 150 seeds of 3,000 steps each, runs the C# twin's
   Rust side with its SIMD path on, since it is the default, and passes; so do all 659 tests of WiseMan's
   `MarsRt|MarsNative` filter, the state tests above among them, and all 455 of the crate's. The three games, from
   power-on and from their gameplay states, 600 frames each, compared every frame in state, picture and sound
   against the machine whose vector unit is element by element (`a_machine_whose_vector_unit_runs_in_host_vectors_is_the_machine_element_by_element`):
   unthreaded and joined, and on four workers deferred through the recompiler with snapshots — twelve runs, 7,200
   frames, identical. Every benchmark run below ended on the same state hash as the build before this section:
   `4DEACE55468AA765`, `6881AF7D3E8BF471` and `18279384D87951D7`, in all 45 runs.

`cargo clippy --all-targets` is clean. The build for a host without the module — the osx-arm64 job's case — was
checked here by compiling the crate with `target_arch = "x86_64"` in `rsp/mod.rs` renamed to a value no target has:
clippy reported nothing but the renamed condition itself, so no item the scalar path needs is behind the x86 gate
and nothing left outside it is unused. The real aarch64 target is not installed on this machine, and the CI job was
not run.

##### 6.10.2 Mutants

Ten, each applied alone to the SIMD path and run against the crate's `tests::rsp`. Where a random-program failure is
listed, it is the first seed and step at which the two paths parted.

| Mutant | Random programs | Every function and selector | Loads and stores at the edges | Clamp boundaries |
| --- | --- | --- | --- | --- |
| The signed clamp saturates at `0x7FFE` | caught | caught | — | — |
| `VMUDL`'s product written to the middle third, not the low | seed 2, step 59 | function 04, selector 0 | — | — |
| Selector 13 (element 5) shuffles element 4 | seed 2, step 34 | function 00, selector 13 | — | — |
| `VADDC`'s carries inverted in VCO | seed 2, step 33 | function 14 | — | — |
| `VMUDL`'s unsigned high product taken signed in lane 7 | seed 2, step 59 | function 04 | — | — |
| `add48` drops the carry of a carry into the high third | seed 12, step 4 | function 02 (`VRNDP`) | — | — |
| The unsigned clamp's `> 0x7FFF_FFFF` taken as `>=` | survived | survived | — | **caught**, `VMACU` |
| The whole-element load takes an odd element | seed 4, step 51 | — | caught at `0xFD0` | — |
| `VCL` updates greater-or-equal where not-equal was set | seed 2, step 10 | function 24 | — | — |
| A whole accumulator write keeps bits 63:48 | seed 2, step 5 | function 00 | — | caught |

*What the pattern says.* The five mutants the plan named, and four more, fall to the random programs on their first
seeds and to the sweep. The seventh is §3.3's survivor again, in the other core and the other unit: the boundary is
reachable only through an accumulate whose sum lands on `0x7FFF_FFFF` exactly, and neither a uniform nor an
edge-drawn accumulator lands there — a plain `VMULU` never can, its sum being even — so it survived 600,000 steps
and 12,288 single instructions, and fell at once to a test that puts the sum there on purpose. §3.3 recorded the
C# twin's version as surviving; the clamp-boundary test would catch that one too, since it compares whatever the
element-by-element unit computes. One further mutant was argued equivalent and not made: the fast path first
refused a window that ran past DMEM's end, and loosening that bound by a byte changes nothing, because `data` and
`set_data` mask the address to twelve bits exactly as the byte loop does. The bound was removed; the element-by-element
fallback for such windows is test 3's case.

##### 6.10.3 Speed, and the prediction's fate

Three interleaved rounds of 600 frames from the gameplay states, on an otherwise idle desktop (load 1.3 at the
start), each round running the build before this section (WiseMan 8d5e2c7), this section's build, and this section's
build with `EMUSEN_MARSRT_RSP_SIMD=0` — the element-by-element unit on the new layout — in turn. Production's shape,
`examples/threads <rom> <state> 600 split 4 blocks`:

| ms a frame | before | SIMD | element by element, new layout |
| --- | --- | --- | --- |
| GoldenEye, the Dam | 14.477, 14.352, 14.306 | **11.658, 11.670, 11.696** | 14.894, 14.949, 15.002 |
| Ocarina of Time | 6.733, 6.754, 6.764 | **5.602, 5.563, 5.572** | 6.941, 7.040, 6.948 |
| Super Mario 64 | 5.737, 5.760, 5.721 | **4.629, 4.584, 4.579** | 5.905, 5.963, 5.984 |

Medians: the Dam 14.35 to 11.67 ms, **2.68 ms and 19 per cent**; Ocarina of Time 6.75 to 5.57, **1.18 ms, 17.5
per cent**; Super Mario 64 5.74 to 4.58, **1.15 ms, 20 per cent**. No round of one column overlaps another's. The
processor alone, `examples/frames … 600 blocks`, one thread, the rasteriser on it, three rounds of the first two
builds:

| ms a frame | before | SIMD |
| --- | --- | --- |
| GoldenEye, the Dam | 22.175, 22.084, 22.022 | 19.430, 19.424, 19.584 |
| Ocarina of Time | 10.548, 10.542, 10.514 | 9.093, 9.199, 9.130 |
| Super Mario 64 | 10.398, 10.434, 10.398 | 8.950, 8.972, 8.963 |

2.65, 1.41 and 1.44 ms. The Dam saves the same on one thread as on five; the two lighter games save a quarter of a
millisecond more on one, and the threaded runs' own counters say where it went: the emulation thread that waited
for the workers 0.000 to 0.007 ms a frame before now waits 0.12 to 0.14 ms in Ocarina of Time (sites 2 and 11, a load
and a transfer into the processor waiting for the draw) and 0.22 to 0.24 in Super Mario 64 (site 8, the deferred
capture waiting for the draw). The thread now reaches those sites before the workers have finished — the first
sign in this page that a faster emulation thread can meet the split's workers rather than never seeing them.

*The prediction.* "From 14.5 to between 10 and 12 ms, and the other two by 1 to 1.5 ms each": 11.67, and 1.18 and
1.15. **It held**, in the upper half of its range for the Dam. Its reasoning held too — "the vector unit's share is
not all arithmetic, and C#'s §14 did not make its unit free" — and the sample below says by how much: the unit
kept a third of its cost.

*The element-by-element unit on the new layout* is 3 to 4 per cent slower than before: `acc` now composes a word
from three loads and `set_acc` stores three thirds and the bits above them, where the words were one load and one
store. A first version, whose low-third write also went through `acc` and `set_acc`, was 10 to 12 per cent slower
(16.19, 15.83, 16.13 ms at the Dam in the same kind of round); giving the trait a `set_acc_low` that writes the
low third alone recovered most of it. The element-by-element unit is the
oracle and off by default, so its speed is recorded and not pursued.

##### 6.10.4 Where the frame goes now

One sampled run of each game, threaded, as §6.9's (`rtsample.py`; its grouping now also names the SIMD unit's
frames, which no older binary has, so §6.9's reports are unchanged); 5,898, 2,348 and 1,742 samples, the shares
multiplied by the medians above. The sampled frames ran 2 to 3 per cent above them.

| ms a frame, share | the Dam | Ocarina of Time | Super Mario 64 |
| --- | --- | --- | --- |
| RSP: the vector unit, with its loads and stores | **1.83, 15.7%** (5.25, 36.2%) | **0.90, 16.1%** (2.44, 36.0%) | **0.80, 17.4%** (2.43, 41.8%) |
| RSP: decode and dispatch | 1.90, 16.3% (1.64) | 0.72, 12.9% (0.54) | 0.67, 14.5% (0.48) |
| RSP: the lock-step | 0.96, 8.2% (0.83) | 0.30, 5.5% (0.26) | 0.29, 6.3% (0.25) |
| RSP: the scalar unit and its loads and stores | 0.79, 6.8% (1.02) | 0.25, 4.4% (0.39) | 0.31, 6.7% (0.34) |
| RSP in all | **5.57, 47.7%** (8.8, 61%) | **2.21, 39.6%** (3.65, 54%) | **2.07, 45.2%** (3.5, 60%) |
| the presenter's join | 0.79, 6.7% (0.63) | 0.86, 15.4% (0.69) | 1.13, 24.6% (0.99) |
| decoded blocks, loop and handlers | 2.48, 21.3% (2.11) | 0.68, 12.3% (0.66) | 0.24, 5.2% (0.28) |
| RDP: waiting for the workers | 0.00 | 0.06, 1.1% | 0.19, 4.1% |

In brackets, §6.9's two-run means. The vector unit is a sixth of each frame where it was a third to two fifths, and
at the Dam it is no longer the processor's largest part: its decode and dispatch is, 1.9 ms, which §6.9's third lever
— a compiler for the processor — is the answer to and which this section did not touch. On Super Mario 64 the
presenter's join is now the largest single thing on the thread, a quarter of the frame, which is §6.9's fourth lever.
Two readings are not explained. The decode row reads 0.18 to 0.26 ms more than §6.9's on every game and the
lock-step 0.04 to 0.13, and the Dam's decoded blocks 0.35 ms more, on code this section changed only by moving the vector dispatch into
`execute` and not at all; the frame fell by less than the vector unit's row (2.68 against 3.42 ms at the Dam) by
about that much. Whether that is the sampler's attribution across the new call into a `target_feature` function,
the layout of `Rsp::run` after the dispatch moved, or a cost that moved rather than vanished, one run a game cannot
say.

##### 6.10.5 What is not done

- **NEON.** aarch64 — the osx-arm64 build and any ARM handheld — runs the element-by-element unit. Nothing here
  measures what it would buy there; the arithmetic ports nearly instruction for instruction — `pshufb` is `tbl` —
  except `pmovmskb`, which NEON has no form of and which would be the lane masks anded with their bit weights and
  added across.
- **The handheld.** `deck@10.1.1.205` did not answer (the connection timed out) at the time of writing, so the Legion
  Go S — where §6.1 found the Dam at 90 per cent of its console and this section should matter most — is not measured.
  It needs only the two builds above and the three rounds.
- **The other thirteen load and store formats**, `VMOV` and the reciprocals, element by element, above, and no count
  of how often the games reach them.
- **The processor's decode and dispatch**, now its largest part at the Dam, and the lock-step: neither is this lever.
- **The element-by-element unit's 3 to 4 per cent**, recorded and not pursued.
- **The C# twin's component on the new trait.** `Pinned` keeps C#'s arrays, so its SIMD path reaches the accumulator
  through the trait's defaults, composing and splitting words each operation. It is exact — `MarsNativeRspTests`
  above — and its speed is not measured; the component is off in C# (§4).
- **AVX2.** The unit is 128 bits wide and every operation touches one register; 256-bit lanes would help only a
  pair of independent operations, which a step at a time never has.
- **The unexplained 0.3 ms** of §6.10.4, and the CI's own run of the osx-arm64 and win-x64 jobs.

#### 6.11 The presenter's join, and the walk in bands (2026-09-23)

§6.10.4 left the deferred presenter's join as the largest single thing on Super Mario 64's emulation thread, 1.13 ms
and a quarter of a 4.6 ms frame, and 0.86 and 0.79 ms of Ocarina of Time's and the Dam's. §6.9 had found the presenter
busy 1.4 ms of an average frame while the emulation thread waited 1.0 ms of it, "so the walk barely overlaps the next
frame; why was not measured". This section measures why, and removes most of the wait. **The finding** is that the walk
does overlap the next frame, all of it; the next frame is simply shorter than the walk. The games' fields alternate
between a heavy one that ends with a new picture and a light one in which the game waits for the retrace, and the
walk handed over after the heavy field is joined at the end of the light one. The mean frame, four times the mean
walk, hides a light field a quarter as long as the walk it waits for. **The fix** is the C# core's: the deferred walk in bands of rows, one on
the presenter and three on helper threads, so that the walk ends inside the light field. Super Mario 64 goes from 4.69
to 3.65 ms a frame, Ocarina of Time from 5.70 to 4.79, the Dam from 11.65 to 11.03, with the state hashes unchanged.

**The claim.** §5.6.5's claim stands unchanged: with presentation deferred, the picture shown after frame *n* + 1 is
exactly the immediate picture of frame *n*, and the state is the machine's at once. The bands are a way of computing the
same raster, not a different raster: every band leaves the rows it owns as the one-band walk leaves them, and the
immediate path, which is not banded, is the oracle.

##### 6.11.1 The diagnosis, measured before anything was changed

*The instruments.* Two were added, and both stay. The presenter now counts the joins that found its job unfinished and
how long they waited, and the scan-out the jobs joined and the presenter's own time over them (`Presenter::waits`,
`waited_nanos`, `Scanout::joined`, `presenter_nanos`); `examples/threads` prints them after the display processor's
wait sites, which is the counter §5.6.8 lacked. And `examples/threads … trace=<file>` writes one line a frame: the
emulation (`Machine::run_frame`), the present, the join's wait inside it, the presenter's time over the job that join
took back, whether this present handed a walk over, and site 8's wait. The trace times the two threads against each
other frame by frame, which a sample of either cannot: the presenter's time is its own clock around `Work::run`, the
join's is the emulation thread's around its spin.

The traced runs are production's shape (`split 4 blocks`, 600 frames from the gameplay states); the first twenty frames
are left out. A frame is *heavy* below if its emulation took more than 4 ms.

| 580 frames, before the fix | Super Mario 64 | Ocarina of Time | GoldenEye, the Dam |
| --- | --- | --- | --- |
| heavy fields, their emulation (median, ms) | 290, 5.36 | 194, 9.36 | 317, 20.99 |
| light fields, their emulation (median, ms) | 290, 0.81 | 386, 2.15 | 263, 2.08 |
| walks handed over, and after which fields | 290, every heavy one | 194, every heavy one | 559, nearly every field |
| the presenter's time a walk (median, ms) | 3.12 | 4.83 | 3.55 |
| joins that waited, of those that took a walk back | 290 of 290 | 193 of 193 | 281 of 558 |
| a waiting join's wait (mean, ms) | 2.31 | 2.63 | 1.70 |
| joins whose frame ran shorter than the walk they took back | 289 of 289 | 193 of 193 | 279 of 558 |
| the join's wait over all frames (ms a frame) | 1.15 | 0.88 | 0.82 |

*What the table says.*

- **Super Mario 64** runs two fields a picture on its PAL console. The field that ends with a new frame buffer at the
  origin is 5.4 ms; the next, in which the game sits in its idle loop until the retrace, is 0.8 ms. The walk of the new
  picture, 3.1 ms, goes out at the end of the heavy field and is joined at the end of the light one, which therefore
  waits 2.3 ms for it. At the end of the light field the capture repeats (§5.6.5's repeat test: 299 of the 600 scans), so
  nothing goes out, and the heavy field that follows overlaps no walk at all. Every one of the 290 light fields waits.
- **Ocarina of Time** draws one picture in three fields: 9.4 ms, then 2.2 and 2.2. Its walk, 4.8 ms, is joined at the
  end of the first light field, which waits 2.6 ms; the second light field repeats and waits nothing.
- **The Dam** walks every field — its capture does not repeat — and alternates fields of 21 and 2.1 ms. The walk after
  a light field overlaps a heavy one and is never waited for; the walk after a heavy field, 3.6 ms, is joined after a
  light one and waits 1.7 ms. That is the 281 of 558.

So the reason §6.9 did not have is the shape of the frame, not the presenter. The presenter starts at once and works
without pause — its time per job is the walk §5.4.4 measured on one thread, 2.85 to 4.5 ms, plus the composition,
0.09 ms (timed separately, once, on Super Mario 64) — and the walk overlaps the whole of the next field. The join
waits because that field is a quarter of the walk. The sample of §6.9 saw the same thing from the other side: a
presenter busy 1.4 ms of a 5.8 ms *mean* frame, which is 2.8 ms of every other one.

*What the diagnosis rules out.* Three remedies were considered before any was built, and the diagnosis decides
between them.

- **A second buffer**, so that the next capture need not wait for the last walk. At the light field's end nothing is
  captured (it repeats) or the capture is small (the Dam); what the join waits for is the *picture*, which §5.6.5 says
  must be the one shown after this frame. A second buffer would show it a frame later: the claim changed, not kept,
  and §6.9 already called that a play decision.
- **The join moved later** — to the host's read of the frame, say. Every host reads the frame after every frame, on
  the thread that runs the machine, so the wait would move and not shrink.
- **A shorter walk.** The light field is 0.8 ms on Super Mario 64 and about 2.1 ms on the other two; the walk it must
  contain is 3.1 to 4.8 ms. C#'s four bands walk in 1.8 to 2.9 ms (§5.4.4). That is the only one of the three that
  keeps the claim and shortens the wait, and it is the one built.

##### 6.11.2 The walk in bands

`src/vi/scan/bands.rs` is C#'s `Vi.Walk` banding (`Mars_Video.md` §2.12), on MarsRT's threads.

- *How many.* `bands = clamp(rows / 32, 1, count)`, C#'s `MinimumBandRows`; `count` is C#'s `Vi.Bands`, a quarter
  of the processors, one to four — four on this sixteen-processor desktop. `Scanout::bands`
  sets it for a scan-out (the tests), `EMUSEN_MARSRT_SCAN_BANDS` for a process (the measurements), one to eight.
- *Who walks.* The presenter walks the first band itself and hands each other band to a helper thread of its own,
  started when a walk first needs it and stopped with the presenter. A helper keeps a walker of its own across walks,
  as C#'s `_walkers[b]` does, so that its caches are its own.
- *What a band writes.* Band *b* is rows ⌊rows·b/n⌋ to ⌊rows·(b+1)/n⌋. Row *r* writes its columns from the raster
  line `top·stride + left + (lower ? width : 0) + stride·r`, and never reaches the next row's line, since `left ≥ 0` and
  `columns ≤ width − left ≤ stride` at one and at every multiple. The raster is therefore split with `split_at_mut` at
  each band's first line, and each band holds a chunk no other band holds. A write outside the chunk is an index out of
  range, which panics, so a wrong boundary is loud (mutants 5 and 6 below) rather than a race.
- *The hand-over* is the presenter's own protocol (§5.6.5) once per helper: a slot with a state word, the task written
  and `SUBMITTED` stored with Release, the helper's Acquire, its band, its panic if it raised one, `DONE` with Release,
  the presenter's Acquire in a spin that yields after 64 rounds. The task carries the walk's pass (the picture, the
  captured bytes, the raster's width) as a pointer to the presenter's stack and the chunk as a pointer and a length.
  That is the one `unsafe` lifetime the design adds; it is sound because the presenter waits for every helper before
  `Bands::walk` returns *or unwinds* — its own band runs under `catch_unwind`, and a panic of any band is resumed only
  after all have finished.
- *Why it is exact.* It is §2.12's argument, which C# has run on since its bands were built: every cache the walker
  keeps is a pure function of memory and the registers, `begin` empties the window and the slots, and the fetch bug's
  counter has a closed form at a band's first row (`rows` already computed it from the row before). A band walker
  therefore writes what the one-band walker writes for the same rows.
- *What is banded.* The deferred walk on the processor, at one and at every multiple (at a multiple the rows are the
  scaled picture's, `rows·n`). Not banded: the immediate walk (`present_now`, `scan`), which stays the oracle; the
  device's walk and `write_device_picture`, which the device path already takes off the processor; and the
  composition and the box average, which follow the walk on the presenter.

##### 6.11.3 The evidence

1. **Synthetic pictures in every number of bands**
   (`a_deferred_walk_in_any_number_of_bands_is_the_immediate_walk`). Nine geometries — progressive with whole steps;
   fractional both ways with divot and gamma; thirty-two-bit; interlaced on each field, the lower with the dither
   filter; pulled in at the left and clamped at the right; PAL's tallest, 288 rows; a top offset; and one of 63 rows,
   too short to split — each in one to eight bands, over a megabyte of noise in colour and coverage, three steps each
   with the frame buffer changed between them: the deferred picture after every step is the immediate picture of the
   step before, and after the last join the raster is the immediate raster byte for byte. 72 cases; the counter
   `banded_walks` is required to say that all three walks of a case ran in bands, or none for the short picture and
   for one band.
2. **The wait at the new site** (`the_join_waits_for_a_held_band_and_shows_the_whole_picture`), in the pattern of
   `tests/sites.rs`. With four bands, band 1, 2 and then 3 is held on its helper before its rows, and released from
   another thread after 300 ms. The join must wait for all of it, and the picture it then shows must be the immediate
   one. It does, for each band.
3. **Lost wake-ups** (`three_thousand_banded_walks_lose_no_wake_up`): 3,000 banded walks back to back, each joined by
   the next present and compared with the immediate picture, under a watchdog of two minutes. A wake-up lost by a
   helper or by the presenter hangs there and fails.
4. **The games.** From power-on and from the three states, 600 frames each, every frame compared in state, picture a
   frame late and sound with the machine at once, the tests of `tests/games.rs` run with the default of four bands:

   | test | runs | frames |
   | --- | ---: | ---: |
   | deferred, unthreaded (`a_deferred_picture_is_the_immediate_picture_of_the_frame_before`) | 6 | 3,600 |
   | deferred, one worker, snapshots (`a_threaded_and_deferred_machine_…`) | 6 | 3,600 |
   | four workers through the blocks, snapshots (`a_recompiled_machine_on_four_workers_deferred_…`) | 6 | 3,600 |
   | the same in two, three and eight bands (`a_deferred_walk_in_bands_is_the_immediate_walk_a_picture_late`, new) | 18 | 10,800 |
   | at 2× and 4×, four workers, deferred (`a_machine_at_a_multiple_split_and_deferred_…`) | 12 | 7,200 |
   | at 2× and 4× on the device, the RX 6800 (`a_machine_at_a_multiple_on_the_device_…`) | 12 | 7,200 |
   | averaged on the device, 4×/2, 4×/4, 2×/2 (`a_machine_averaged_on_the_device_…`) | 18 | 10,800 |

   90 runs, 54,000 frames, identical. The frames at the multiples walked the scaled raster in bands on the processor
   runs; the device runs' pictures were the device's, which the bands do not touch, and are the evidence that the
   device path is unchanged beside them. The counters, from a rerun of the four-worker test after the band counter was added: of 2,509 jobs joined in its six runs, 2,305 walked in bands — every job of the three states (300, 300 and 579) and, from power-on, the rest being jobs with nothing to walk or a picture under 64 rows, which the counter does not tell apart.
5. **ThreadSanitizer**, as §5.6.7 runs it. Two runs, after one control:

   | run | result |
   | --- | --- |
   | the positive control: the helper's `DONE` stored Relaxed, the scan tests | 29 reports, in `Helper::wait`, the walker's fetch and allocation, the composition and the band's panic slot; the equality tests passed but for the 3,000 walks, which ran out their watchdog under the instrumentation |
   | the scan tests, the thread tests and the site tests (18, 24, 15) | 9 reports, none with a frame in `bands.rs`, `presenter.rs` or the walker |
   | the scan tests again, and the games through the blocks on four workers deferred and in two, three and eight bands, 100 frames each (24 runs) | 9 reports, the same |

   Every report of the two clean runs is in std's own synchronisation, which §5.6.7's build cannot see: libtest's
   result channel (seven, the one §5.6.7 suppresses), the fence at the end of an `Arc`'s last drop (the test's hold
   flag, a thread handle and the recompiler's `jit::Slot`), a channel's thread-local context (the watchdog of test 3),
   and `OnceLock` (§6.10's `simd_default`, reached by two threads building machines at once). None is a hand-over of
   this section's; each helper's is its own atomic, which the control shows TSan does see.
6. `cargo test --release`, all 459 of the crate's tests, and `cargo clippy --all-targets`, debug and release, clean.
   WiseMan's `FullyQualifiedName~MarsRt` filter — `MarsRtTests`, `MarsRtThreadsTests` and `MarsRTViTests` among it — 602 tests, all passed, against the library built from this section's crate.

##### 6.11.4 Mutants

Six, each applied alone to `bands.rs` and run against the three band tests one at a time, with a limit of three
minutes each.

| Mutant | Any number of bands | The held band | 3,000 walks |
| --- | --- | --- | --- |
| a band's join skipped (band 1's helper never waited for) | caught: fractional, 5 bands | crashed, SIGSEGV | survived |
| a band handed back before its walk finished (the helper's `DONE` before its rows) | caught: progressive, 3 bands | hung | caught, walk 1 |
| a helper's band starts one row late | caught, 2 bands | caught, band 1 | caught, walk 1 |
| a helper's band stops one row early | caught, 2 bands | caught, band 1 | caught, walk 1 |
| the presenter's band walks one row into the next | caught: an index out of range | caught: the same | caught: the same |
| a chunk starts one row below its band's first row | caught: an index out of range | caught: the same | caught: the same |

*What the pattern says.* The two boundary mutants that move rows between bands without leaving a chunk are caught as
wrong pictures: a row no band walked keeps the step before's bytes. The two that move a boundary across a chunk are
caught as a panic in the band, which the presenter carries back to the join, and are never a silent race — the point
of splitting the raster by `split_at_mut` rather than by pointers. The skipped join survived the 3,000 small walks,
whose skipped band of 32 rows of 64 columns finishes before the composition reaches it; it fell to the larger
pictures and, as undefined behaviour does, crashed the held-band test outright. The early hand-back hung the held-band
test because the helper's second `DONE`, after its band, overwrote the presenter's `STOP`; a hang counts as caught.
In the same pattern as §5.6.9's positive control, a seventh mutant — the helper's `DONE` stored Relaxed — is invisible
to the equality tests on x86-64 and is ThreadSanitizer's control (below).

##### 6.11.5 Speed, and the prediction's fate

*The prediction*, written to the results directory after the diagnosis's trace and before the first line of the bands
was built: four bands cut each walk by about C#'s own ratio, 2.5 (§5.4.4, 4.55 against 1.78 ms), to 1.2, 1.9 and
1.5 ms; the light fields that join them are 0.8 to 1.0, 2.1 to 2.5 and 2.3 to 2.7 ms; so Super Mario 64 from 4.60 to
3.6 ± 0.15 ms, with a residual wait of 0.2 to 0.4 ms on every light field; Ocarina of Time from 5.56 to about 4.7; the
Dam from 11.67 to about 10.9, its whole 0.8 ms of join.

*The measurement.* Three interleaved rounds of 600 frames, `examples/threads <rom> <state> 600 split 4 blocks`, each
round running the build before this section (WiseMan 2d85f17, built from a clean checkout), this section's build, and
this section's build with `EMUSEN_MARSRT_SCAN_BANDS=1` in turn, every run under the shared lock with ten seconds'
pause before each game. The load average at the start of the runs was 1.4 to 5.1: another agent's runs were interleaved
with these under the same lock, and a run's own four spinning workers raise the next run's reading, so the number was
recorded and the rounds were compared with each other rather than rejected. The state hashes are
`4DEACE55468AA765`, `6881AF7D3E8BF471` and `18279384D87951D7` in all 27 runs, as in §6.10.

| ms a frame | before | four bands | one band, this build |
| --- | --- | --- | --- |
| GoldenEye, the Dam | 11.645, 11.608, 11.709 | **10.979, 11.026, 11.034** | 11.972, 11.768, 11.977 |
| Ocarina of Time | 5.602, 5.738, 5.704 | **4.682, 4.828, 4.787** | 5.786, 5.669, 5.685 |
| Super Mario 64 | 4.687, 4.648, 4.716 | **3.755, 3.625, 3.649** | 4.576, 4.613, 4.620 |
| the join, four bands (ms a frame) | — | 0.022–0.028, 0.001–0.002, 0.154–0.196 | 0.79–0.80, 0.88–0.91, 1.12–1.13 |

Medians: **the Dam 11.65 to 11.03 ms, 0.62 and 5.3 per cent; Ocarina of Time 5.70 to 4.79, 0.92 and 16 per cent;
Super Mario 64 4.69 to 3.65, 1.04 and 22 per cent.** No round of one column overlaps another's. The one-band column is
the control: the same build with the old walk is the old frame within two per cent on the two lighter games, and 2.8
per cent slower at the Dam, which the rounds do not separate from noise.

*The band count*, one run each on Super Mario 64 of the same build (`EMUSEN_MARSRT_SCAN_BANDS`), in the same session:

| bands | 1 | 2 | 3 | 4 | 6 | 8 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| ms a frame | 4.682 | 3.983 | 3.813 | 3.702 | 3.585 | 3.596 |
| the join, ms a frame | 1.115 | 0.508 | 0.313 | 0.189 | 0.103 | 0.071 |
| the presenter's time a walk, ms | 3.07 | 1.82 | 1.46 | 1.22 | 1.04 | 0.99 |

Six bands would buy another 0.1 ms here; the default stays C#'s four, because it is C#'s, because the handheld shares its
package with the thread that is the bound (§6.1), and because the eighth band's walk is no faster than the sixth's.

*The prediction's fate.* **It held for Super Mario 64 and Ocarina of Time and fell short at the Dam.** Super Mario 64
landed at 3.65 against 3.6 ± 0.15, the residual wait at 0.15 to 0.20 ms a frame, which is 0.3 to 0.4 ms on each light
field, as predicted; Ocarina of Time at 4.79 against about 4.7. The walks' ratio was C#'s: 3.1 to 1.2, 4.8 to 1.75, 3.55
to 1.26 ms, 2.5 to 2.8 times. At the Dam the join fell from 0.80 to 0.02 ms, all of it, as predicted, and the frame fell
by 0.62 ms, not 0.8. The Dam is the one game whose walks also overlap its heavy fields, and there four threads now
share the cache and the memory with the emulation thread for 1.3 ms where one did for 3.6; that the missing 0.2 ms is
that sharing is a reading, not a measurement. §6.9's own prediction for this lever, stated on the frames before §6.10
— "bands or a deeper queue take Super Mario 64 from 5.8 to about 5.0 and Ocarina of Time to about 6.2, with the Dam
gaining half a millisecond" — is retired: the bands took 1.04, 0.92 and 0.62 ms, more than its 0.8, 0.6 and 0.5 on
every game, from frames that §6.10 had already shortened. It had priced the lever from the mean frame, which is what
this section found the join does not depend on.

##### 6.11.6 Where Super Mario 64's frame goes now

`rtsample.py` on the emulation thread, one run before and two after, the shares multiplied by the medians above:

| ms a frame, share | before | after, run 1 | after, run 2 |
| --- | --- | --- | --- |
| RSP: the vector unit | 0.81, 17.3% | 0.90, 24.7% | 0.88, 24.1% |
| RSP: decode and dispatch | 0.62, 13.2% | 0.68, 18.7% | 0.69, 18.9% |
| RSP: the lock-step | 0.31, 6.6% | 0.24, 6.5% | 0.27, 7.5% |
| the presenter's join | **1.12, 24.0%** | **0.13, 3.5%** | **0.15, 4.0%** |
| RDP: waiting for the workers | 0.22, 4.7% | 0.12, 3.4% | 0.11, 2.9% |
| decoded blocks, loop and handlers | 0.33, 7.1% | 0.34, 9.2% | 0.33, 9.0% |
| samples | 1,793 | 1,210 | 1,197 |

The join is a twenty-fifth of the frame where it was a quarter; the signal processor's vector unit and its decode and
dispatch are now the first and second things on the thread, §6.9's third lever. The sampled runs' own frames were 4.78
and 3.73 to 3.79 ms, two to four per cent above the unsampled medians.

*Two runs discarded, and why.* The first "before" run of the emulation thread ran at 33 ms a frame, seven times its
unsampled frame, with 17.5 ms of it at site 8; it was repeated, and the repeat is the column above. The all-threads
runs that were to sample the presenter and the helpers beside the emulation thread ran at 17.5 ms (before) and hung the
sampler at the child's exit (both), so no sample of the presenter's thread is given: stopping every thread a thousand
times a second stalls threads that wait for each other by spinning, which the Dam's workers of §6.9 survived and this
game's presenter and workers did not. The per-frame trace of §6.11.1, which times both threads on their own clocks,
is the measurement of the presenter's side, and the trace after the fix gives: walks of 1.18, 1.75 and 1.26 ms
(medians), joins that waited 282 of 290, 20 of 193 and 73 of 558, and waits of 0.39, 0.14 and 0.46 ms when they did.

##### 6.11.7 What is not done

- **The handheld.** The Legion Go S gets a quarter of its processors by the same rule; how many bands that is there,
  and whether they cost the emulation thread more where it shares the package with them, is not measured.
- **The residual wait on Super Mario 64**, 0.15 to 0.20 ms a frame: a walk of 1.2 ms joined after a light field of
  0.8. Six bands take half of it; composing in bands, or starting the band helpers spinning rather than parked, might
  take more. Neither is tried.
- **The multiples' speed.** The bands at 2× and 4× are proven exact (§6.11.3) and not timed; at a multiple with
  antialiasing the box average runs after the walk on the presenter alone, and its share of the job is not measured.
- **The immediate walk** is not banded, so the non-deferred mode and the picture after a load walk on one thread as
  before.
- **The Dam's missing 0.2 ms** (§6.11.5), a reading and not a measurement.
- **The all-threads sample** of the presenter and its helpers (§6.11.6).

#### 6.12 The signal processor's instructions decoded once (2026-09-23)

§6.10.4 left the processor's decode and dispatch its largest part at the Dam, 1.9 ms of an 11.7 ms frame, with the
lock-step's own overhead another 0.96, and §6.9 ranked the lever third with a ceiling of 9.8 ms at the Dam if the
row cost nothing. This section takes the row apart, builds the first tier C#'s `Mars_Rsp.md` §11 built — IMEM decoded
once, each instruction's handler chosen at decode — under the lock-step's rule, and prices the second. **The Dam
went from 11.69 to 10.44 ms threaded and from 19.52 to 18.04 on one thread; the other two games by 0.10 to 0.18 ms
threaded and 0.33 to 0.42 on one. The prediction stated before the table was written held; the one revised from
the microbenchmark did not.** The Cranelift tier is priced and not built.

**The claim.** `src/rsp/decoded.rs` runs the processor from a table of one entry a word of IMEM. Every step it takes
leaves the processor, DMEM, IMEM, the interface and the MI exactly as the interpreter leaves them at the same step,
in the lock-step as in the whole runs, and every observable interaction — the COP0 moves, the break, the transfers,
the status and semaphore, the display processor's registers, the interrupt — happens at the same step, because the
table never runs one: an event is left to the machine's `rsp_event`, exactly where the interpreter leaves it.

##### 6.12.1 What decode and dispatch was made of

*Where the steps are taken.* A build with counters on every entry point (not kept), 300 frames of each gameplay
state, four workers, deferred, tier 2, per frame:

| per frame | the Dam | Ocarina of Time | Super Mario 64 |
| --- | ---: | ---: | ---: |
| steps one at a time from a tick (`sp_step(1)`), of them events | 372,733 (13,029) | 104,989 (5,139) | 49,235 (1,527) |
| calls of many cycles (`sp_step_many`), cycles | 1,451, 32,265 | 499, 7,295 | 175, 1,689 |
| whole runs from the idle loop (`rsp_run_to_event`), steps | 19,786, 648,696 | 9,851, 318,888 | 10,353, 462,603 |
| of the whole runs, those ended by an event | 19,413 | 9,636 | 10,041 |
| events in all | 33,541 | 14,944 | 11,610 |
| **share of the steps taken in whole runs** | **62%** | **74%** | **90%** |

So the part of the processor's work that may run many instructions at once — the idle loop's run to the next event,
§5.2's `RunIdle`, and the tick's many-cycle call — is most of it in every game, in runs of 33 steps on average at the
Dam and 45 in Super Mario 64. The Dam's steps one at a time are the 38 per cent §5.8.3 is about: the CPU's decoded
blocks entered beside a running processor tick it once an instruction.

*What the row was.* The Dam's baseline, sampled the same day with the build before this section (`b1`, 5,835 samples,
the shares times the 11.69 ms median below), and regrouped so that the innermost frames the rows are made of are
named (`split.py` beside the speed tooling; §6.9's grouping put the interpreter's step and its counters in two rows,
which this one keeps apart):

| the processor at the Dam, ms a frame | the interpreter | what the table does to it |
| --- | ---: | --- |
| `execute`: the opcode's jump table and the scalar arms inlined into it | 1.07 | the table replaces with one indirect call; the arms become handlers |
| `is_event`: the event test, where the fetch's load latency lands | 0.30 | the entry's kind, chosen at decode |
| the fetch's byte swap and the fetch | 0.31 | off the dispatch's path: the entry is found by the program counter alone |
| `special`'s second jump table | 0.17 | a handler per function |
| the fields `rt`, `rd` | 0.28 | not removed: the handlers take them from the word |
| the vector unit's own switch of 64 functions (inside `vector_op_simd`, 0.66 in all) | part of the vector row | a handler per function and selector kind |
| the counters' update, `run`'s loop, `tick`, `sp_step`, the lent memory | 0.73 | the counters and the halt once a straight run in whole runs; the rest not |
| **the processor in all** | **5.62** | |

The decode proper — fetch, swap, event test, two switches — is about 1.9 ms of the 5.62, and none of it depends on
anything but the word, which is what a table caches. What a table cannot remove is the call it puts in place of the
switch, the field extraction inside the handler, and, in the lock-step, everything around the step: the tick, the
halt tests, the lent memory, the counters, since the CPU may read the program counter after any of them.

*The microbenchmark* (`examples/rsp_bench`, kept): 24 whole machines taken while the processor ran in each gameplay
state, each run again 4,000 steps, one tick at a time or in runs to the next event, the events run untimed, so that
the number is the step's own. Three interleaved rounds, medians, ns a step, the build before against the table:

| ns a step | the Dam | Ocarina of Time | Super Mario 64 |
| --- | ---: | ---: | ---: |
| one tick at a time: before, the table | 4.31, 4.06 | 4.54, 4.15 | 3.91, 4.09 |
| in whole runs: before, the table | 3.49, 3.16 | 3.83, 3.45 | 3.51, 3.20 |

A third of a nanosecond a step in whole runs, a quarter to four tenths in the lock-step at two games and a sixth of one
*slower* at the third. On a processor running no-operations the whole run cost 1.28 to 1.65 ns a step before and 1.12 after, and
that number moved by 0.7 ns between two runs of one binary (1.12 and 1.86), so it is an alignment's as much as a
design's and is not used below.

##### 6.12.2 The design

*One entry a word, not blocks.* The table is 1,024 entries of sixteen bytes: the handler, the four bytes of IMEM it
was decoded from as the host reads them, and a word of information — whether the instruction is an event, a branch or
jump, or pure, and for a pure one the length of the run of pure instructions from it, at most 64. C#'s blocks are
runs found by their first address, compiled, and compared whole with IMEM on entry; a table indexed by the word
needs no lookup and no comparison of a run, is entered at any address — a branch into the middle of a run finds that
address's own entry and its own length — and has no shape to refuse. The price is an indirect call an instruction,
where C#'s compiled block calls its handlers directly.

*The handlers are the interpreter's arms.* `Rsp::execute` became a 64-way choice among `main::<OP>` and `special` one
among `special_function::<F>`, both generic over the constant they used to match on, and the vector unit's function
switch one among `vector_lanes::<F, KNOWN>`, where `KNOWN` says whether the selector is known at decode to take the
whole register, known to shuffle, or unknown, as it is to the interpreter. A handler is the arm itself: an
instruction interpreted and the same instruction from the table are one piece of source, as C#'s §12 required. At
decode, an instruction whose only effect is a write to `r0` — the ALU and shift functions, the immediates, the loads,
`MFC2` and `CFC2` — gets a handler that does nothing, `JALR` excepted, which also jumps; the vector operations get
the SIMD path's function directly when the host has it, and the handler still tests `Memory::simd` so that
`set_rsp_simd` needs no rebuild.

*Validity: every entry compared with IMEM before every use.* The alternative, clearing entries when IMEM is written,
has to find every writer, and there are at least seven: a transfer into IMEM, the CPU's stores of one, two, four and
eight bytes through the bus, boot's IPL head, the host's memory writes through the C ABI, a state load, and the crate's
own tests, which write `sp_imem` directly — `tests::rsp`'s selector sweep rewrites word 0 before every step. A writer
missed is an instruction run stale, silently. Comparing is one load of IMEM, which the step needs anyway for the
instruction's fields, and one compare; with it removed (an inexact build, for pricing only) the straight run of
no-operations cost 1.120 ns a step against 1.117 with it, so its cost is below this bench's resolution. This is also
what C# does: `Mars_Rsp.md` §11 invalidates nothing, since "every entry compares". The table survives a state load
(`load_state` carries it across, as it carries the recompiler), which the comparison makes safe.

*Straight runs, and the one piece of bookkeeping they drop.* In a whole run, outside a delay slot (`next_pc` is the
program counter plus four), the entry's length says how many pure instructions follow; the run takes up to that many,
cut to the budget left, comparing each word with IMEM as it goes and stopping at the first that has become impure,
and moves the two counters once at the end. A pure instruction neither reads nor writes the counters, so the
counters after the run are the interpreter's after the same steps. The length is a hint: a stale one can only end the
run early or be caught by the comparison of the word it overruns into, which then stops it. The halt is tested once
at the run's start, since only an event can halt the processor and the run stops before one.

*Where the lock-step allows it, and where it does not.* The table runs many instructions at once only where the
machine already did — the idle loop's run to the next event and the tick's many-cycle call — and adds no new place:
the question of which steps may be taken ahead of the CPU is §5.2's and §5.8.3's, unchanged. Inside those calls the
CPU does not run, so nothing can observe the counters between two steps, and IMEM cannot change, since the processor
writes it only by a transfer, an event, which ends the run. One tick at a time — 38 per cent of the Dam's steps — the
table caches the decode only: the counters are moved before the handler, as the interpreter moves them, so the CPU
reads the same program counter at every cycle. Making that path out of line, so that its indirect call has one site
rather than one inlined into each of the CPU's decoded operations, was measured and is not kept (the Dam 10.48 to
10.93 against 10.34 to 10.50 in the same three interleaved rounds, a first set that ended at a load of 5).

*The switch.* On by default; `EMUSEN_MARSRT_RSP_BLOCKS=0` turns it off for a process and `Machine::set_rsp_blocks` for
a machine, which survives a state load. The C# twin of §3 (`Pinned`) has no table and runs the interpreter.

##### 6.12.3 The evidence

1. **Random programs one tick at a time, compared after every step**
   (`tests::rsp_blocks::the_decoded_table_steps_as_the_interpreter_steps_one_tick_at_a_time`). Two machines, the
   table and the interpreter, from one state; programs of every class the table decodes differently — the vector unit
   at both selector kinds and its loads and stores, the scalar unit with `r0` among the destinations, branches either
   way by up to twelve words, jumps, register jumps and links, breaks, COP0 reads of the status, semaphore and display
   registers, and COP0 writes of the transfer registers and the status (single-step set one time in eight) — with
   registers primed so that the transfers bring code from RDRAM into IMEM; between steps the CPU writes a word into
   IMEM, half the time just ahead of the processor, or starts a transfer into it. 150 seeds of 3,000 steps, half with
   the SIMD path and half element by element: 450,000 steps, 52,014 events, 10,542 breaks, 1,784 transfers into IMEM by
   the processor, 58,482 taken branches landing in the middle of a straight run, 33,710 CPU writes and 11,173 CPU
   transfers into IMEM, 35,237 steps under single-step. After every step the processor, the interface, DMEM, IMEM and
   the MI are compared, and the whole states at the end. Identical.
2. **Every entry point** (`…runs_as_the_interpreter_runs_in_every_entry`): the same programs, each call drawn among a
   tick, a many-cycle call of 2 to 59 cycles and a whole run to the next event with a budget of 1 to 299; 150 seeds of
   800 calls, 39,730, 40,020 and 40,250 of each, 3,133,136 steps, 8,928 CPU writes and 2,971 CPU transfers into IMEM,
   compared after every call, the whole runs' step counts included, and the states at the end. Identical.
3. **A word rewritten under the table** (`a_word_rewritten_under_the_table_is_run_as_rewritten`): forty no-operations
   run straight, a CPU store turns the seventeenth into a break, and the run stops there with the interpreter's
   counters; a transfer then fills IMEM with `ADDIU r1, r1, 1` and 500 steps leave `r1` at 500. Both vector paths.
4. **The crate's processor tests with the table on**, since it is the default: `tests::rsp`'s SIMD oracles, whose
   sweeps rewrite IMEM word 0 before each of 43,088 single instructions, so that every one is a new decode.
5. **The games** (`tests::games::a_machine_whose_signal_processor_runs_decoded_is_the_machine_interpreted`): the three
   games from power-on and from their gameplay states, 600 frames each, the table against the interpreter, every
   frame compared in state, picture and sound, unthreaded and joined and on four workers deferred through the
   recompiler with snapshots: twelve runs, 7,200 frames, identical. Every benchmark run below ended on the state hash
   of the build before this section, `4DEACE55468AA765`, `6881AF7D3E8BF471` and `18279384D87951D7`, threaded and not,
   the same three §6.10 recorded.
6. **The rest.** All 459 of the crate's tests with the gameplay states present (their games at 120 frames); the
   corpus through MarsRT's three recompiler tiers, 1,722 lines identical to the interpreter's; WiseMan's
   `MarsRt|MarsNative` filter, 659 tests, among them `MarsNativeRspTests` — the C# twin's Rust side, which runs the
   interpreter this section refactored — and the corpus line for line against the C# core with the shim's default,
   the table, "Failed 46 of 4637". `cargo clippy --all-targets` is clean.

##### 6.12.4 Mutants

Seven, each applied alone to a copy of the crate and run against `tests::rsp_blocks` at 40 seeds without link-time
optimisation (`mutants.py` beside the speed tooling). The five the plan named are given in the table's terms, since a
table has no blocks to invalidate or enter.

| Mutant | Caught by |
| --- | --- |
| A straight run trusts every entry after its first: the table not checked after a transfer into IMEM | every entry, seed 1001, call 0 (the whole runs ran 6 and 67 steps); the rewritten word |
| An event counted pure for a run's length: a run past a status write | every entry, seed 1001, call 0 (6 and 10 steps); the rewritten word |
| A straight run begun in a delay slot: the branch's target dropped at a run's end | every entry, seed 1001, call 0 (counters `FF4, FF8` against `A00, A04`) |
| Selector 2 given the handler for a selector that takes the whole register | one tick at a time, seed 2, step 2,724; every entry, seed 1002, call 198 |
| A run entered in the middle given the length recorded at its head | every entry, seed 1001, call 0; the rewritten word |
| `JALR` to `r0` folded to nothing with the other writes to `r0` | one tick at a time, seed 1, step 53; every entry, seed 1001, call 660 |
| The run's halt test removed | **survives, equivalent**: every caller — the tick, the many-cycle loop after each event, the idle loop's run, the bench's entry — tests the halt first |

*What the pattern says.* The mutants of the straight run fall at the first whole call of the first seed, because a
random program's first run is nearly always long enough to cross what they break; the tick-only test catches none of
them, which is right — it never takes a straight run — and is why both tests exist. The two decode mutants fall to
both random tests, since a wrong handler is wrong on whichever path runs it.

##### 6.12.5 Speed, and the predictions' fate

*The prediction, stated before the table was written:* about a nanosecond a step off the processor's 5 — the
fetch's swap, the event test, the second switch and the vector unit's own switch, against the cost of a call — which
over the Dam's 1.05 million steps is **about 1.1 ms, the Dam to about 10.6 ms**, and proportionally less for the other
two. *Revised after the microbenchmark:* its third of a nanosecond in whole runs and quarter in the lock-step came to
**about 0.3 ms at the Dam**.

Three interleaved rounds of 600 frames from the gameplay states, the build before this section (a clean checkout of
2d85f17), this section's build, and this section's build with `EMUSEN_MARSRT_RSP_BLOCKS=0`; production's shape,
`examples/threads <rom> <state> 600 split 4 blocks`, at a load of 1.4 at the start:

| ms a frame | before | the table | the interpreter, this build |
| --- | --- | --- | --- |
| GoldenEye, the Dam | 11.685, 11.589, 11.760 | **10.423, 10.451, 10.441** | 11.706, 11.722, 11.678 |
| Ocarina of Time | 5.586, 5.599, 5.593 | **5.465, 5.398, 5.418** | 5.564, 5.567, 5.621 |
| Super Mario 64 | 4.571, 4.568, 4.584 | **4.467, 4.517, 4.460** | 4.590, 4.555, 4.555 |

Medians: the Dam 11.69 to 10.44 ms, **1.24 ms and 10.6 per cent**; Ocarina of Time 5.59 to 5.42, **0.18 ms, 3.1 per
cent**; Super Mario 64 4.57 to 4.47, **0.10 ms, 2.3 per cent**. No round of the table overlaps a round of the build
before. The processor alone, `examples/frames … 600 blocks`, one thread, three rounds (the first began at a load of
5, the rounds agree to 0.2 ms) and a fourth for the hashes:

| ms a frame | before | the table | the interpreter, this build |
| --- | --- | --- | --- |
| GoldenEye, the Dam | 19.433, 19.523, 19.667, 19.528 | 18.102, 18.042, 18.003, 18.013 | 19.780, 19.490, 19.521, 19.488 |
| Ocarina of Time | 9.129, 9.106, 9.322, 9.090 | 8.793, 8.705, 8.691, 8.696 | 9.151, 9.162, 9.147, 9.139 |
| Super Mario 64 | 8.961, 8.973, 8.984, 9.015 | 8.642, 8.595, 8.741, 8.654 | 8.988, 9.014, 8.993, 8.960 |

1.48, 0.42 and 0.33 ms. The two lighter games gain twice to three times as much on one thread as on five, and the
threaded runs' counters say where it went, as they did in §6.10.3: the emulation thread's wait for the workers rose
from 0.13 to 0.30 ms a frame in Ocarina of Time and from 0.27 to 0.43 in Super Mario 64, the thread now arriving
at the load and capture sites before the draw is done. The Dam's workers are never waited for, and it keeps its whole
gain.

*The predictions.* The first — 1.1 ms, the Dam to about 10.6 — **held**, at 1.24 ms and 10.44. The revision from the
microbenchmark — about 0.3 ms — **is retired**: the game gained four times what the bench's per-step difference
promised. The sample below says why: a quarter of the gain is not the processor's at all. The interpreter was
`#[inline(always)]` from `execute` up through the tick, so the whole of it was inlined into every call site of the
tick in the CPU's decoded blocks, and the CPU's own decoded loop (`decoded::run`) shrank when the table replaced it
with a lookup and a call: 1.18 to 0.90 ms at the Dam, 0.41 to 0.29 in Ocarina of Time, 0.18 to 0.10 in Super Mario 64.
A bench that runs the processor alone cannot see a cost that the processor's code imposes on the code it is inlined
into; the lesson is §3.4's again, from the other side — where the processor's step sits matters as much as what it
does.

*The interpreter on this build* is within a round of the build before in the games, threaded and not, but 10 per cent
slower in the microbenchmark's whole runs (3.85 to 4.04 ns a step against 3.45 to 3.54): the 64-way choices among
constants did not compile to quite the old switches. It is the table's oracle and off by default, so its speed is
recorded and not pursued.

##### 6.12.6 Where the processor's time goes now

The same grouping as §6.12.1, sampled the same day, `b1` before and `s1`, `s2` with the table (5,835, 5,077 and
5,151 samples at the Dam; 2,286 and 2,195 in Ocarina of Time; 1,690 and 1,832 in Super Mario 64), the shares times the
medians above:

| ms a frame | the Dam, before | the Dam, the table (two runs) | Ocarina, before, table | Mario, before, table |
| --- | ---: | ---: | --- | --- |
| decode and dispatch | 2.15 | 1.91, 1.92 | 0.65, 0.63 | 0.72, 0.61 |
| the vector unit | 1.96 | 1.23, 1.33 | 0.76, 0.61 | 0.73, 0.58 |
| the scalar unit, loads and stores | 0.68 | 0.85, 0.80 | 0.30, 0.31 | 0.29, 0.36 |
| the lock-step: tick, `sp_step`, the counters | 0.73 | 0.56, 0.54 | 0.24, 0.21 | 0.16, 0.11 |
| events | 0.10 | 0.09, 0.09 | 0.04, 0.04 | 0.02, 0.02 |
| **the processor** | **5.62** | **4.62, 4.68** | **1.98, 1.81** | **1.92, 1.68** |
| the CPU's decoded loop, for the reason above | 1.18 | 0.90, 0.93 | 0.41, 0.29 | 0.18, 0.10 |

The decode row keeps its size in name only: at the Dam it is now `Decoded::step` (0.55 ms — the tick's steps, one
lookup, one comparison and one call each, where the load of the entry and of IMEM stall), the run and its straight
loop (0.38), the handlers' entry frames and the SIMD path's test in them (about 0.35) and the fields the handlers
extract (0.2); the switches, the swap
and the event test are gone from it. The vector unit's row fell by 0.6 to 0.7 ms because its own switch of 64 went with
them, and the scalar row rose by 0.1 to 0.15 because the arms that `execute` held inline are now attributed to their
handlers. The processor at the Dam is 44 per cent of the frame, from 48.

##### 6.12.7 The second tier, priced and not built

A Cranelift tier would compile a straight run to one function that calls its handlers directly, or holds the scalar
ones inline, entered after one comparison of the run's words — `Mars_Rsp.md` §12's fold, on the crate's existing
compiler (`src/cpu/blocks/jit/`) and its thread. What it could remove is the table's own cost *inside whole runs*: the
run and straight loops, the indirect calls and the handlers' entries, and the scalar arms' register traffic — about
1.37 ms of the Dam's decode row outside `Decoded::step`, of which the whole runs take 62 per cent by the counters:
**a ceiling of about 0.85 ms at the Dam, 8 per cent**, less in the other two, whose decode rows are 0.6 ms in all.
It could not touch the tick's steps, which are one instruction a call and which `Mars_Rsp.md` §13 measured a compiled
single step to *lose* on, nor the lock-step's 0.55 ms. What the C# history says a fold buys once blocks exist is two to
three per cent of the frame (§12 of that page). Against a realistic third to a half of the ceiling — 0.3 to 0.4 ms —
stand a second compiled path under the exactness contract, a compile queue beside the CPU's, and its verifier. **Not
built**; the number to beat, should it be taken up, is the Dam at 10.44 ms.

##### 6.12.8 What is not done

- **The tick's steps.** 38 per cent of the Dam's steps are taken one at a time beside the CPU's decoded blocks; the
  table saves them the decode and not the call, the lookup's loads or the tick around it. §5.8.3's run-ahead batch,
  priced at 0.3 ms and not built, is still the lever there.
- **The handlers' fields.** Every handler extracts its registers from the word; entries with the fields decoded would
  need a larger entry or a second array, and neither was measured.
- **The handheld.** Not measured, as in §6.10.5; the change should matter there as here, since §6.1 found the
  handheld bound on the same thread.
- **The interpreter's 10 per cent in the microbenchmark**, above, recorded and not pursued.
- **aarch64.** The table is portable — only its vector handlers are x86-64's. The build without them was checked as
  §6.10.1 checked it, `target_arch = "x86_64"` renamed in `src/rsp/` to a value no target has: clippy reported the
  nineteen renamed conditions and nothing else. The real target and the osx-arm64 job were not run.

#### 6.13 The picture lent, not given (2026-09-23)

The handheld's `[fps]` line, read by the parent session on the Legion Go S, showed the loop's time outside `RunFrame`
(`total − run`) at about 1 ms at one multiple, **6 to 7 ms at two and 12 to 13 at four**, with `RunFrame` itself 16 to
24 ms against 11 to 12 in a bench of the same scene: consistent with full collections stopping every thread. A probe
on this desktop (`~/.cache/emusen/probe/mars-speed/fpsprobe/`, Donkey Kong 64's title) found the source: 1.2 MB
allocated a frame at one and 4.7 MB at two, a gen-2 collection every four or five frames, all of it
`GetFrameBufferRgba`, which returned `_frame.AsSpan().ToArray()` (§5.5). The C# core allocates nothing there,
because it hands out its live buffer. This section removes the allocation without taking on that exposure.

**The claim.** With Mistress wired as it now is, a MarsRT frame allocates no array: the picture is copied into an array
lent by the core, which Mistress hands back exactly once, when nothing can read it again. No array anything can still
read is written or lent again, the picture on screen never goes back to an older one, and a caller that does not hand
arrays back sees exactly the old behaviour.

##### 6.13.1 The design

*The contract* is a capability, `IFrameBufferPool.ReturnFrameBuffer(byte[])`, beside `IFrameSerial` in
`CoreCapabilities.cs`, and its terms are `EmuSen_Multicore.md` §16's. `GetFrameBufferRgba` still returns a copy that is
the caller's alone; a caller finished with it may return it once, from any thread, and the core may lend it again.
Hotaru, Pharaoh, the screenshot and resume-thumbnail paths and every test return nothing and are unaffected.

*The lending* is `EmuSen/Cores/FrameBufferLending.cs`: at most four arrays waiting and a record of the last eight lent,
under one lock. An array is taken back only if it is in the record, at the current length, with room; a second return,
a foreign array, one of a size since changed and anything returned after `Dispose` are dropped. `MarsRtCore` keeps its
own `_frame`, which `TakePicture` fills from the library as before, and `GetFrameBufferRgba` copies it into a lent
array. The copy is the price of keeping `_frame` the core's; §6.13.4 says what it costs.

*The frame control* (`EmuSen_Serenity.md` §2.8) takes a `release` with each offer and calls it when the offer is
superseded, is not the one its cached image came from, and has no draw operation that could still copy it. Doing so
safely needed the defect below fixed first: the control must only ever move forward.

*Mistress* hands frames over through `FrameHandOff`, whose frames carry a state so that the thread that moves one out of
*waiting* owns its array: the UI thread by presenting it, the emulation thread by replacing it unseen, which returns it
at once. The `release` it passes is a route to the session's core, cut in `ShutDownCurrentSession`, so that the last
picture on screen cannot keep an ended session's core and its native machine reachable.

**The defect fixed on the way.** The control copied an operation's array whenever the operation's version differed
from the cached one's, so an operation drawn late with an older version copied an older picture over a newer one.
`A_draw_operation_rendered_late_never_takes_the_picture_backwards` was written first and failed against the unchanged
control, the older operation drawing its red where the newer green had been shown (line 50 of the test, "Expected
(0, 200, 0), Actual (200, 0, 0)"). The rule is now "only newer than anything copied", with a stale operation drawing
the cache. It was latent, since Avalonia's compositor applies batches in order; with lent arrays it would have been a
torn picture, the older operation's array being by then another frame's.

**Why not the C# core's way.** Handing out `_frame` itself would remove the copy too, and the tearing risk with it
would be exactly MarsCore's. The frame control copies an offer out on the render thread, so a live array is safe only
while the render thread copies before the next `TakePicture`, which nothing guarantees and a paused render thread,
a minimised window or a slow compositor break. MarsCore is left as it is (§5.5).

##### 6.13.2 The evidence

In `EmuSen.WiseMan`, headless:

- `GameFrameReleaseTests` (Serenity, 4): the backwards draw above; an array given back once and only when nothing in the
  control can read it — an unread offer when superseded, a drawn one when a newer is drawn, the cached one never while
  it is cached, nothing when an operation is disposed twice or drawn again; a held operation disposed undrawn letting
  its array go; the same array offered four times, as Moon and Mercury offer theirs, never given back.
- `FrameHandOffTests` (Mistress, 5): frames replaced before the UI thread took them returned once and a presented one
  left to the control; a route kept per core and cut at the session's end; the ended session's core collected while
  its last picture is still on screen, and nothing returned to it afterwards; and, with MarsRT, the hand-off and the
  control wired as `MainWindow` wires them, the two tests that follow.
- *No tearing.* 400 frames of the synthetic N64 system on a seeded schedule that holds up to four draw operations,
  draws them late and out of order and disposes them at random: 188 pictures presented, 187 draws of which 83 were
  late and 54 fell on a changed picture. After every frame each array lent and not yet returned is byte-identical to
  the copy taken when it was lent, the core never lends one of them, and every draw is pixel-identical to the newest
  version drawn so far. The lending made 9 arrays and reused 391, so a premature return would have had its array
  rewritten within a frame or two.
- *Nothing of the frame's size allocated.* On the same path, 120 frames after 30 of warming: 667 to 670 bytes a frame against
  a 1,228,800-byte picture, and no array made after the warming (2 in all).
- `FrameBufferLendingTests` (5): a returned array lent again and a held one never; a double return lent once; a
  foreign array, even of the lent size, and one lent at another size dropped; the record and the free list bounded
  whatever is returned; a closed lending taking nothing.
- `MarsRtFrontendTests.The_frame_handed_out_is_a_copy_the_next_frame_does_not_touch`, kept under its name with the
  claim widened to the new contract: the held array stays byte-identical through twelve frames whose arrays are
  returned each time and reused (at most one made), is never among them, and once returned is lent again holding the
  new picture.
- The blast radius: the Mistress, Serenity (with the slang tests), Hotaru, MarsRT and lending classes, 996 tests,
  all passed.

##### 6.13.3 Mutants

Each applied alone, built, and run against the frame tests above, the source restored after each (`mutants.py`
beside the probe in `~/.cache/emusen/probe/mars-speed/fpspool/`):

| Mutant | Caught by |
| --- | --- |
| An offer held by a draw operation released one version early (`> _copiedVersion + 1`) | `With_MarsRT_lending_no_array_the_screen_can_read_is_written_and_the_picture_never_goes_back` |
| The stale operation's rule restored: any other version copies (`!=`) | the same; `A_draw_operation_rendered_late_never_takes_the_picture_backwards` |
| A frame the hand-off drops is not released | `Frames_replaced_before_the_screen_took_them_go_back_once_and_a_presented_one_is_the_screen_s`; the no-tearing test |
| The cached offer not protected | `An_array_is_given_back_once_and_only_when_nothing_here_can_read_it`; the no-tearing and allocation tests |
| The lending takes back an array it has not lent | `Returning_an_array_twice_does_not_lend_it_twice`; `A_foreign_array_or_one_lent_at_another_size_is_dropped` |
| The route not cut when the session ends | `Once_a_session_ends_the_picture_left_on_screen_neither_keeps_its_core_nor_returns_to_it`; `A_core_that_does_not_lend_gets_no_route_and_one_that_does_keeps_its_route_until_the_session_ends` |

*What the pattern says.* The one-version-early mutant is caught only by the randomised test, not by the scripted one:
the scripted sequence never holds an operation exactly one version past the last copy when that version is
superseded, and the randomised schedule does within its first frames. The foreign-array case survived the first
version of the lending test, whose foreign array was returned before any length was set and was dropped for its
length instead; a foreign array of the lent length was added and catches it.

##### 6.13.4 Speed, and the prediction's fate

*The prediction, stated before measuring:* in a desktop loop shaped like Mistress's, `total − run` at two and four falls
to the size of the copy into the lent array alone, about 1 ms at four, and the gen-2 collections disappear.

*The loop* is a probe, `~/.cache/emusen/probe/mars-speed/fpspool/`: `RunFrame`, the audio drained, and, when the serial
moved, `GetFrameBufferRgba` offered through a newest-wins slot to a render thread that copies it with
`SKImage.FromPixelCopy`, as the frame control does. In the pooled build the render thread returns the array it showed
once it has copied a newer one, and the producer returns one replaced unseen. Unpaced, Donkey Kong 64's title (480
lines, interlaced), rows sent once, the processor rasteriser, four workers, presentation deferred; 300 frames warmed
and 300 measured, the two builds interleaved, three rounds (`ab.sh`, `ab-2026-09-23.txt` there):

| Multiple (frame) | build | `RunFrame` ms | loop ms | `total − run` ms | of which the copy | allocated a frame | gen-2 in 300 | pause in 300 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1× (640×480) | new array | 3.41–3.53 | 3.48–3.61 | 0.07–0.08 | 0.07–0.08 | 1,202 KiB | 75 | 8.0–8.9 ms |
| | lent | 3.35–3.41 | **3.38–3.45** | 0.03 | 0.03 | 1.9 KiB | **0** | 0 |
| 2× (1280×960) | new array | 7.19–7.44 | 7.54–7.79 | 0.35–0.37 | 0.34–0.36 | 4,802 KiB | 100 | 30.0–31.0 ms |
| | lent | 7.25–7.32 | **7.51–7.61** | 0.26–0.28 | 0.25–0.27 | 1.9 KiB | **0** | 0 |
| 4× (2560×1920) | new array | 22.19–22.46 | 26.40–26.64 | 4.17–4.21 | 4.15–4.20 | 19,202 KiB | 100 | 100.7–101.2 ms |
| | lent | 24.10–24.73 | **25.71–26.39** | 1.61–1.67 | 1.59–1.65 | 1.9 KiB | **0** | 0 |

Every collection the new arrays caused was a gen-2 (the counts of all three generations are equal), since each array
is large-object heap.

*The prediction's fate.* **The collections disappear**, as predicted: 75 and 100 in 300 frames to none, and with them
8 to 101 ms of pause. **`total − run` falls to the copy alone**, as predicted in kind: what is left outside `RunFrame` is
the copy into the lent array to within 0.02 ms. **Its size was short of the prediction at four:** 1.6 ms, not about 1,
because the frame there is 2,560 by 1,920, 19.7 MB, twice the 640 by 240 picture the estimate had in mind; the copy
runs at about 12 GB/s at every multiple.

**What the prediction did not say, and the table does:** at four the loop is no faster. The 2.5 ms the emulation thread
no longer spends outside `RunFrame` reappear inside it, 22.2 to 24.1–24.7 ms, and the loop moves only from 26.4–26.6 to
25.7–26.4. The loop at four on this desktop is bound by the deferred walk (§6.11), whose join inside `RunFrame` absorbs
whatever the emulation thread saves. Two checks say so. Padding the lent build's loop with a spin after the offer
brings `RunFrame` back down by what it pads and leaves the loop where it was: 2.5 ms of spin, `RunFrame` 22.21 ms and
the loop 26.43; 5 ms, 20.16 and 26.87. And with presentation immediate, where there is no walk to join, `total − run`
falls from 3.76–3.81 to 1.38–1.39 ms with `RunFrame` unmoved within its noise (95 to 102 ms, the walk now inside it,
two rounds). At one and two, where the emulation thread is the bound, the loop gains about 0.1 ms, 4 and 1.5 per cent of it.

*What the desktop cannot show.* The handheld's excess was 6 to 13 ms and not 0.3 to 4: its collections are far dearer
than this machine's one millisecond each, and §6.1 found it bound on the emulation thread. The prediction for it,
stated here before the parent session measures it: `total − run` at two and four falls to the copy there, 1 ms or a
little more at four, `RunFrame` returns toward the bench's 11 to 12 ms where it was inflated by collections, and the
gen-2 count in Mistress's own log is zero.

##### 6.13.5 What is not done

- **The copy.** `GetFrameBufferRgba` still copies `_frame` into the lent array, 1.6 ms at four on this desktop. Writing
  the library's picture straight into a lent array in `TakePicture` would remove it, at the price of the core no longer
  owning the picture it answers screenshots and tests with, and of `GetFrameBufferRgba` lending the same array to two
  callers. Not built.
- **The C# core.** Its live buffer and its exposure are unchanged (§5.5); lending there would add a copy it does not
  make today.
- **The small allocations.** The audio drain's `short[]`, 1.5 KiB a frame, and 0.2 KiB inside `RunFrame`, both gen-0
  and collected every few thousand frames; and on Mistress's path the hand-off's frame and the control's offer, a
  few hundred bytes. None was removed.
- **A lease.** A caller that returns the same array twice across a re-lend is not detectable by array (`EmuSen_Multicore.md`
  §16); Mistress returns once by construction.
- **The handheld.** Not measured here, by instruction; the prediction for it is stated above.
- **A real window.** Every test is headless; Avalonia's live compositor, its order of rendering and disposing draw
  operations, and the GPU backend's copy were not instrumented. The rules of `EmuSen_Serenity.md` §2.8 do not depend
  on that order.
