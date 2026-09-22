# Mars_Native — Rust components beside the C# core

*Started 2026-09-22.* The first question this page answers is whether a component of Mars rewritten in Rust, called
from the C# core, makes the core faster. The first component tried, the signal processor, is **exact and slower**,
and this page records why. It stays in the tree, off by default, as the exact second implementation it is and as the
groundwork a later component would stand on.

## 1. The library and how Mars finds it

`EmuSen.Native/` is a Rust crate at the repository's root (`emusen-native`, edition 2024), built as a C-ABI dynamic
library. It uses `panic = "abort"`, fat LTO, one codegen unit, and keeps line tables so that a panic's backtrace names
its lines.

`EmuSen.csproj` runs `cargo build --release` before the core compiles, and copies the library beside the assemblies as
a `None` item. MSBuild carries that item into every consumer's output (Mistress, Hotaru, WiseMan) and into a publish.
Cargo runs only for the host's own runtime identifier. A publish for another platform, or a machine without cargo,
gets no library.

`MarsNative` loads it from `AppContext.BaseDirectory`. It refuses a library whose `emusen_native_interface_version` is
not the build's (1), and installs the panic log. `EMUSEN_MARS_NATIVE=0` turns the library off. **Every component falls
back to its C# twin when the library is absent, refused or off.** `MarsNativeTests` pins the load.

## 2. A panic never crosses into C#

`emusen_native_set_crash_log` gives the library a path, `DataStore.Logs/native_crash_<pid>.txt`. Its panic hook writes
the message and a backtrace there, and the process then aborts. A panic in an emulator component is a bug, and
unwinding into the runtime is undefined, so stopping with a record is the only honest choice. Mistress's own crash log
cannot see a native abort, which is why the library keeps its own.

## 3. The signal processor in Rust

`src/mars/rsp.rs` ports the C# processor's **plain** path: the scalar unit, the vector unit's arithmetic, all 24
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
