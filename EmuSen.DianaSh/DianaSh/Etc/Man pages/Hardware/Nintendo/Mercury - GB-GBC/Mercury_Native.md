# Mercury_Native — a plan to port Mercury to Rust

*Written 2026-09-23, before any code.* On 2026-09-23 it was decided that every EmuSen core will be ported to Rust once
Mars is finished (`EmuSen_Stack.md` §2.1), and Mercury is the one to plan now. This page is the plan. It follows the
method `Mars_Native.md` §5 used for MarsRT, and it does what §2.1 asks of every 2D port: it states its own speed
prediction before the work begins, because the 2D cores' remaining costs were never traced to the language.

The page keeps two things apart. **Measured** means a number taken on 2026-09-23 on the desktop described in §1.1.
**Argued** means reasoning from the code or from Mars's evidence, with no measurement behind it. Each prediction is
numbered (P1–P6) so that it can be retired later, as `Mars_Native.md` retired P1–P10.

## 1. What the port is for

### 1.1 The baseline (measured)

**How it was measured.** The harness has no Mercury bench, so a throwaway console app was built against the
checkout's `EmuSen.csproj`, in Release on .NET 10.0.111, on the Ryzen 7 7700X. It does the following:

1. It loads a scratch copy of the ROM with the battery disabled.
2. It boots through 900 frames, pressing Start and A alternately every 45 frames.
3. It times 3,000 frames flat out, one `RunFrame` each, draining the audio and taking the picture after every frame.

Every run is its own process. Games are interleaved within a round, and there are three rounds. The machine was shared
with other work, and round 3 was noisier than rounds 1–2 (Link's Awakening's p99 was 1.89 ms). So the
table gives medians alongside means. Each game's state hash was identical in every round, so the runs were
deterministic.

| Game | Mode | Mean ms/frame (3 rounds) | Median (3 rounds) | % of full speed |
|---|---|---|---|---|
| Tetris (Rev 1) | DMG, no MBC | 0.612 / 0.601 / 0.601 | 0.607 / 0.599 / 0.600 | 2737–2786% |
| Link's Awakening | DMG, MBC1 + RAM | 0.542 / 0.538 / 0.608 | 0.541 / 0.534 / 0.544 | 2753–3113% |
| Kirby's Dream Land | DMG, MBC1 | 0.672 / 0.661 / 0.716 | 0.665 / 0.659 / 0.685 | 2337–2532% |
| Pokémon Yellow | **CGB** (header `$80`), MBC5 | 0.680 / 0.681 / 0.731 | 0.675 / 0.678 / 0.694 | 2289–2463% |
| Synthetic `JR -2` loop, CGB | single speed | 0.598 / 0.567 | 0.575 / 0.567 | 2802–2953% |
| The same loop after `KEY1` + `STOP` | **double speed** | 0.891 / 0.863 | 0.888 / 0.857 | 1880–1940% |

**The renderer's share.** `SkipRendering` skips exactly `RenderBackground` and `RenderSprites` and nothing else
(`Ppu.Render.cs:21-25`). With it on, over two rounds:

| Game | Mean ms/frame, rendering skipped | Renderer's share |
|---|---|---|
| Tetris | 0.509 / 0.508 | ≈0.095 ms (16%) |
| Link's Awakening | 0.446 / 0.458 | ≈0.09 ms (16%) |
| Pokémon Yellow | 0.553 / 0.553 | ≈0.13 ms (19%) |

**A component profile.** It used `ipsample.py` (`~/.cache/emusen/probe/mars-speed/agent-cpu-tools/ipsample/`), which
takes interrupt-style samples of the main thread and has no safepoint bias. The sampler works on the C# Mercury
process. Two runs were taken on Pokémon Yellow, about 7,500 samples each.

- **With inlining.** Leaf attribution collapses into `RunFrame`, which holds 47.2% of samples, because the JIT inlines
  the CPU step and the bus clock into it. The named leaves are:
  - `Ppu.RenderBackground`, 20.0%;
  - `BreakpointRegistry.ShouldBreak`, **6.7%**;
  - `MemoryBus.TimerBitMask`, 5.5%;
  - `MemoryBus.Read`, 2.7%;
  - `StepOneCycle`, 2.4%;
  - `Apu.DacOutput` and `EmitSample`, 2.6%;
  - `CheatRegistry.TryPatchRom`, 0.9%.
- **With `DOTNET_JitNoInline=1`.** This inflates the cost of calls, so its proportions are distorted, but it attributes
  by method. Grouped by component:
  - the per-T-cycle clock (`StepOneCycle`, `StepBaseClock`, `Tick`, `StepOamDma`, `TimerBitMask`, `OnDivBit`), ≈26%;
  - the APU's per-cycle channel timers and the mixer, ≈21%;
  - the PPU's dot counter, ≈7%;
  - the scanline renderer, ≈17%;
  - the debugger seams (`ShouldBreak` and the empty `List` enumeration under it, `Coverage.IsArmed`), ≈15% (inflated);
  - the SM83 itself (`Step`, `Execute`, `Advance`, fetch), under 10%.

**What the profile says about Mercury's cost.** A Mercury frame is not the cost of the game's program. It is the cost
of clocking every device once per T-cycle. `MemoryBus.Tick` loops `StepOneCycle` once per T-cycle
(`MemoryBus.cs:383-387`). Each iteration steps:

- the PPU's dot counter;
- four APU channel timers and the mixer's fractional accumulator;
- the OAM DMA;
- the 16-bit DIV counter;
- the frame-sequencer edge;
- the TIMA edge detector.

That is 70,224 iterations a frame, or 140,448 in double speed, where the CPU runs twice as many cycles and the timer
doubles with it. This is why a `JR -2` loop costs what Tetris costs. It is also why double speed costs 1.5× a single-speed
frame: the base clock still runs 70,224 times, and the second half of the CPU's cycles pays only for the CPU, the DMA,
the timer and the mapper. **None of the measured games uses double speed,** because the library has no `$C0`
cartridge. A double-speed game is the heaviest case Mercury has, and it was measured only synthetically.

**The low-end target (measured).** The low-end proxy was `--throttle`'s mechanism: `systemd-run --user --scope -p
CPUQuota=N% -p CPUQuotaPeriodSec=5ms`, the arguments `EmuSen.Pharaoh/CpuThrottle.cs` uses.

| Quota | Game | Mean ms/frame | % of full speed |
|---|---|---|---|
| 33% | Tetris | 2.56 / 2.35 | 654 / 712% |
| 33% | Pokémon Yellow | 2.89 / 2.72 | 578 / 615% |
| 10% | Pokémon Yellow | 9.42 (p90 19.1, from quota granularity) | 178% |

The quota overstates slowdown, as `EmuSen_Debugging_Tools_Reference_v5.md` §3.42 records. At 33% the frame is 3.9–4.3×
the unthrottled one, not 3×.

### 1.2 What that says the port is for (argued)

Mercury is far past full speed everywhere it has been measured:

- 23–31× on the desktop;
- about 6× on a third of one of its cores;
- 1.8× on a tenth of one.

At the console's rate the desktop spends about 0.6 ms × 59.73 ≈ 36 ms of CPU time a second on it, which is 3.6% of
one core.

**So the port is not for speed a player could notice.** It is for three things, in order:

1. **Consistency with the stack.** The project's decision is that every core is Rust, and Mercury is the smallest core
   and the first 2D one. That makes it the cheapest test of whether `Mars_Native.md` §5's method transfers to a
   cycle-granular 2D machine. This is its main value to the research: it is the method's second instance, at about
   one-tenth of Mars's size.
2. **Headroom on machines weaker than any yet measured.** At 10% quota a double-speed game would be near 1× (argued
   from §1.1's 1.5× double-speed factor: about 178% / 1.5 ≈ 120%). A fast-forward or rewind multiple is the
   other place headroom is spent.
3. **Power.** On a battery-powered handheld, CPU time per emulated second is energy. This was argued, not measured.

**What it is not for.** The largest lever on Mercury's frame is algorithmic in either language, not linguistic. The
per-T-cycle clock could advance the timer, the APU and the PPU in batches, catching each device up when it is
observed. That is a design change, not a port. It is kept out of the port, because the port's oracle is exactness
against the C# core, and the lever is listed in §5 as a later question with no prediction made for it.

### 1.3 The speed prediction, and how it was derived

**Mars's like-for-like ratios** (`Mars_Native.md` §5.2.3, §5.3.4, §5.4.4, all measured) are:

- the Rust interpreter against the C# interpreter, 1.53–1.74×;
- the RDP rasteriser, 2.13–2.44×;
- the VI scan-out against one C# band, 1.6–1.9×.

`Mars_Native.md` §5.2.3 records that *why* the Rust interpreter was faster was not measured. A mechanism cannot be
transferred, only an analogy.

**Which of Mercury's costs the Mars ratios plausibly transfer to (argued):**

| Mercury cost (share of the frame, §1.1) | Closest Mars analogue | Factor assumed |
|---|---|---|
| Scanline renderer (16–19%, measured by `SkipRendering`) | The RDP rasteriser and the scan-out: per-pixel loops over byte arrays | 1.6–2.4, central 2.0 |
| SM83 interpreter (under 10%) | The CPU interpreter | 1.5–1.75, central 1.6 |
| Per-T-cycle device clock and APU timers (≈55–60%) | **None measured.** It is branchy counter code over small objects (`Bus → Ppu`, `Apu → Pulse1 → Envelope`), reached through two interfaces (`ICpuBus`, `IMapper`) | 1.1–1.8, central 1.4 |
| Debugger seams in the plain loop (≈7% with inlining) | MarsRT's `step_in::<HOOKED>`, where the plain step carries no seam (`Mars_Native.md` §6.5) | the slice removed |
| Mixer, drain, end of frame (the rest, ≈8%) | Nothing close | 1.0–1.6, central 1.3 |

**What does not transfer, and why the middle row is wide.** `Venus_CPU.md` §9.3–§11 measured that hand-devirtualising
the SNES's `ICpuBus` bought nothing: dynamic PGO had already inlined the call. So the prediction cannot count on removing
Mercury's interface calls as such. Whatever the tick loop gains has to come from three places:

- flat struct layout, one pointer chase fewer per device field;
- bounds checks the compiler can drop;
- LLVM's optimisation of the loop as a whole.

None of these has been measured in this project for a loop of this shape.

**P1 (speed, desktop).** A line-for-line MercuryRT, with no algorithmic change, runs a plain frame in **0.50 to 0.80 of
the C# Mercury's time, central 0.62 (1.6×)**, on §1.1's four games. Per game, the central values are:

- Tetris 0.60 → about 0.37 ms;
- Link's Awakening 0.54 → about 0.33 ms;
- Kirby 0.67 → about 0.41 ms;
- Yellow 0.68 → about 0.42 ms.

The composite comes from the table: 0.17/2.0 + 0.58/1.4 + 0 + 0.10/1.6 + 0.08/1.3 ≈ 0.62. The ends of the range take
every row's pessimistic or optimistic factor together.

**P2 (renderer).** The renderer's slice, which is 0.09–0.13 ms in C#, ports at about 2× (1.6–2.4×).

**P3 (seams).** MercuryRT's plain frame spends no time `ipsample`/`rtsample` can see on breakpoint, coverage or
ROM-patch checks, against about 7% in C#.

**P4 (what a player sees).** Nothing a player can observe changes, on the desktop or at 33% quota. At 10% quota,
Pokémon Yellow goes from 178% to 250–350% of full speed.

**P5 (the boundary).** Crossing the boundary once a frame costs under 3% of MercuryRT's frame. The crossing is the call,
a 92,160-byte RGBA copy, and about 2.9 KB of audio. `Mars_Native.md` §3.4's lesson, that a boundary crossed every
cycle loses, does not bite because nothing crosses per cycle (§2.1).

**The first like-for-like number** comes at the end of stage 2 (§4): MercuryRT without its renderer against C# Mercury
with `SkipRendering` on. That comparison is exact, because `SkipRendering` removes exactly the code stage 2 has not yet
ported. P1 is re-priced there, before stage 3 begins.

## 2. Shape

### 2.1 The whole machine, one boundary a frame

This is MarsRT's shape. The whole machine is in Rust: CPU, bus, timer, DMA, HDMA, serial sink, joypad, the five boards,
PPU and APU. A C# shim implements the interfaces `MercuryCore` implements, which are `ICore`, `ICheatRegistryHost` and
`IStateFormat` (`MercuryCore.cs:13`). Mercury implements neither `ICoreSettings` nor `ISnapshotCore`, so the shim has no
settings list to mirror.

**The boundary is easier to hold once a frame than it was in Mars (argued).** Nothing inside a Mercury frame needs C#
except the debugger. Mercury's picture is written during the frame, one line at a time, so there is no scan-out to split
from the frame as MarsRT's `advance`/`present` pair does. `EndFrame`'s work runs after the frame, in C#, over the ABI, in
C#'s order (`MercuryCore.cs:170-178`):

1. `TotalFrames++`;
2. `FrameLog.RecordFrame`;
3. `ApplyCheats`;
4. every 300th frame, `SaveSram`.

The four in-frame seams become MarsRT's §6.5 mechanisms, because Rust never calls C#:

| C# seam | Where | In MercuryRT |
|---|---|---|
| `Breakpoints.ShouldBreak(pc)` before every instruction | `MercuryCore.cs:136` | Tables pushed down, including the single-step countdown `ShouldBreak` decrements. The frame returns early with a reason and a pc. C# confirms with `ShouldBreak`, so the registry's semantics stay in C#. |
| `Coverage.Record(pc)` | `MercuryCore.cs:144` | A Rust bitmap C# reads |
| `IWriteObserver.OnWrite` on every write to five spaces | `MemoryBus.cs:169,175,182,190,202` | A write log C# drains after the frame, in order |
| `IRomReadPatcher.TryPatch` on every ROM read (Game Genie) | `MemoryBus.cs:123` | **A patch table pushed down** whenever the registry changes (§2.5) |

### 2.2 Name and place (decided 2026-09-23)

**The name is MercuryRT, and the folder is `EmuSen/Cores/Nintendo/MercuryRT - GB/`.** Both were decided then. The
pattern for every Rust core is `<Name>RT - <console>`, as `MarsRT - N64/` sits beside `Mars - N64/`.

The crate is `mercuryrt`, a `cdylib` + `rlib` with MarsRT's `[profile.release]` and `[profile.dist]`, and no
dependencies. Its layout mirrors the C# folders:

| Rust | C# |
|---|---|
| `cpu/` (core, opcodes, `$CB`, ALU) | `Cpu/Core`, `Cpu/Opcodes` |
| `memory/` (bus, CGB registers and HDMA, cartridge, `mappers/`, joypad) | `Memory/`, `Input/` |
| `ppu/` (timing, render, CGB palettes) | `Ppu/` |
| `apu/` (sequencer and mixer, channels) | `Apu/` |
| `machine.rs` (`RunFrame`'s loop, `EndFrame`'s Rust half) | `MercuryCore.cs` |
| `state.rs`, `naming.rs` | `StateSerializer` as `Mars_Native.md` §5.1 ported it |
| `ffi/` | the shim's interface |
| `Shim/` (`MercuryRtCore.cs`, `.Debug.cs`, `MercuryRtDebugTarget.cs`) | — |

The SM83 disassembler, the cheat codecs and `MercuryCore.Spaces.cs`'s naming stay C#.

### 2.3 The C ABI (interface 1, proposed)

*Retired 2026-09-30.* These `mercury_*` names were replaced by the common native interface. The interface
MercuryRT has now is in §8.7.

All calls go through `delegate* unmanaged` function pointers, as MarsRT's do (`Mars_Native.md` §3.2). A negative
return is a status, and a panic never crosses into C# (§2 of that page).

- **Lifecycle:** `mercury_interface_version`, `mercury_set_crash_log`, `mercury_machine_load_rom(rom, len, sram, len,
  save_path_utf8, len, flags) → handle`, `mercury_machine_free`.
  - `save_path` exists only because C# puts the path in the state (§6.1, D3). If that is fixed first, the parameter goes.
  - The ROM's header parse runs on both sides. C# needs it for `CoreName`, the title and the unsupported-board
    exception before any handle exists.
- **Frame:** `mercury_machine_run_frame(handle) → status`. Zero means the frame completed. A negative value means an
  unimplemented opcode, with its pc, and the shim rethrows C#'s `NotSupportedException` with C#'s message
  (`Cpu.Opcodes.cs:202-204`).
- **Input:** `mercury_machine_set_buttons(handle, mask)`, eight bits in `MercuryCore.PadButtons` order.
- **Picture:** `mercury_machine_frame(handle, out, len)`, 160×144 RGBA. It is copied into a shim buffer, as MarsRT
  copies. Note that C# returns the PPU's live array (§6.3).
- **Sound:**
  - `mercury_machine_audio_buffered`;
  - `mercury_machine_drain_audio(handle, out, max_frames) → frames`, interleaved stereo;
  - `mercury_machine_set_sample_rate(handle, cycles_per_sample, charge_factor)`. The two doubles are computed in C#, so
    that `Math.Pow` is evaluated once, by one runtime (§3.3).
- **State:** `mercury_machine_save_state_size`, `_save_state`, `_load_state`, `_state_layout` (`Mars_Native.md` §5.1's
  listing).
- **Saves and memory:**
  - `mercury_machine_save_data` (cart RAM for `SaveSram`);
  - `mercury_machine_read_space`/`_write_space(handle, space, addr, buf, len)` over the seven spaces of
    `MercuryCore.Spaces.cs`. CPUBUS reads go through the real decode, side effects included, as C#'s do.
- **Harness:** `mercury_machine_serial(handle, out, len)` and `_clear_serial`. WiseMan reads verdicts off
  `Bus.SerialLog`.
- **Options:** `mercury_machine_set_options(handle, flags)`: skip rendering, and channel mutes.
- **Cheats:** `mercury_machine_set_rom_patches(handle, table, count)`.
- **Debugger:** `mercury_debug_set`, `_set_breakpoints`, `_run_frame(handle, flags, *pc) → reasons`, `_writes`,
  `_coverage`, in `Mars_Native.md` §6.5's shapes.
- **Test ABI:** `mercury_machine_step` (one instruction) and `mercury_machine_tick(handle, cycles)`.

### 2.4 What stays C#

The following stay in C#:

- the frontends, `CoreFactory` and `CoreCatalog`;
- every registry (watches, frame log, breakpoints, cheats, coverage, labels) and their semantics;
- the cheat codecs (`GbGameGenieCodec`, `GbGameSharkCodec`) and `CheatRegistry.ApplyAll`'s poke loop, over
  `write_space`;
- the SM83 disassembler;
- `.srm` file I/O through `AtomicFile`, and `CoreOptions.BatteryRamDisabled`;
- the debug target's presentation (sprite, palette and tile-sheet decoding).

**The debug target, proposed (argued).** For read-only inspection, `MercuryRtDebugTarget` should deserialise MercuryRT's
state into a hidden C# `MercuryCore` on `RefreshProviders` and let the existing 637-line `MercuryDebugTarget` read from
that. A Mercury state is 17–148 KB (§3.1), so the transfer costs well under a millisecond. It gives the debugger every
register, palette, channel and board field with no new ABI. `Mars_Native.md` §5 planned the same, but at 6–12 MB MarsRT
could not afford it every refresh. The view is stale by at most one refresh. Writes go through `write_space`.

### 2.5 Where Mercury differs from Mars in ways that change the method

- **No recompiler, and none would pay (argued from §1.1).** The SM83 is under a tenth of the frame. A block compiler
  cannot skip the per-cycle device clock that every memory access must drive, because the bus is cycle-granular
  (`Cpu.cs:137-149`: every access ticks four T-cycles first). Mars's stage 5 has no counterpart here.
- **No threads, no GPU, no render scale.** A whole frame is 0.5–0.9 ms. Mars's §5.6 threads and §6.4 device path have
  no counterpart, which removes the stages where MarsRT met its races.
- **The hot loop is branchier and smaller.** It is one loop of about ten device steps per T-cycle, not one instruction
  loop plus coprocessors. In Rust the bus should own the devices as plain fields, the mapper should be an `enum`
  rather than a trait object, and `ICpuBus` should become a concrete type or a monomorphised generic.
- **The machine is small enough to compare after every instruction.** Mars compared random programs every 16
  instructions, because a 6 MB state is not free. A Mercury state can be compared after every `Step` in the
  random-program oracle.
- **The renderer never feeds the machine, and it is not in the state.** `FrameRgba` and the renderer's scratch are
  `[SkipInState]` (`Ppu.cs:73-87`). So the machine core can be proven by state equality alone before any pixel is
  ported, which Mars needed a stub RDP and a "framer" to approximate (`Mars_Native.md` §5.2.2). One trap:
  `WindowLine++` sits inside `RenderScanline` but outside the render guard (`Ppu.Render.cs:27-28`), and it *is* state.
  It belongs to stage 2, not to the renderer.
- **Game Genie patches must be in Rust.** MarsRT left ROM patches out, because they needed calls from Rust to C#
  (`Mars_Native.md` §5.5). On the Game Boy, Game Genie is the main cheat format, so MercuryRT needs a pushed-down table.
  `CheatRegistry.TryPatchRom` resolves a patch from the CPU address and the *original byte* (the compare field), so the
  table has to be (address → list of (compare, value)), rebuilt when the registry changes. The registry's matching
  rules stay in C#, which flattens them into the table.
- **The oracle corpus exists but is not wired.** Mars had n64-systemtest in place. Mercury's hardware corpus was not
  installed on this machine, and WiseMan cannot read four of blargg's suites (§3.4). Wiring it is stage 0.

## 3. The oracle and the state format

### 3.1 The state, byte for byte (measured by a reflection walk)

*Since 2026-09-28 the codec is shared.* The writer, the reader, the layout listing, the naming rule, `Skip<T>` and the
state half of the C ABI live in `emusen-native` (`EmuSen_RustState.md`), which every Rust core builds from. MercuryRT's `state.rs` keeps its magic, version and refusals; its state exports are `state_exports!`. Its status codes
were already inside the shared rule and did not change.

The format is `StateSerializer`'s reflection walk, and MercuryRT must reproduce it exactly. The facts below were
checked by walking live objects with the serializer's own query (`EmuSen/Common/StateSerializer.cs:36-44`):

- **The header.** It is the magic `"MERC"` (`0x4352454D`), version 5 (`MercuryCore.cs:21-24`), `TotalFrames` (i64) and
  `_cyclesIntoFrame` (i64).
- **The body.** The walks are `Cart`, `Cart.Mapper`, `Cpu` and `Bus`, in that order (`MercuryCore.cs:215-218`). Within
  each class, fields come in the ordinal order of their names.
- **Compiler-generated backing fields come first.** `Cpu`'s walk starts with `<Cycles>k__BackingField` (i64) and
  `<LastInstructionPC>k__BackingField` (u16), because `<` sorts before every letter. They are followed by `A`, `B`, `C`,
  `D`, `E`, `F`, `H`, `Halted`, `Ime`, `L`, `PC`, `SP`, `_haltBug`, `_imeScheduled`, `_interruptPending`.
- **The cartridge is written three times.** The first is `Cart` itself. The second is each mapper's own
  `private readonly Cartridge _cart` (`Mbc1.cs:9`, `Mbc2.cs:11`, `Mbc3.cs:9`, `Mbc5.cs:9`, `NoMbc.cs:6`). The third is
  `MemoryBus._cart` (`MemoryBus.cs:26`), none of them `[SkipInState]`. Each copy is a presence byte, the cart RAM, and
  `_savePath`. On load all three read into the same object, so the last copy wins. MercuryRT writes identical bytes three
  times and reads all three.
- **A .NET string is in the state.** `Cartridge._savePath` (`Cartridge.cs:43`) is written as a `BinaryWriter` string: a
  7-bit-encoded length, then UTF-8, with null written as `""`. MarsRT's `state.rs` has no string primitive, so one is
  needed (§6.1, D3).
- **The layout depends on the ROM.** No tag says which board follows. The mapper's fields come from the header's
  cartridge type, so MercuryRT must build the same board from the same ROM before it can read a state.
- **Other encodings.** `readonly` fields are serialised and overwritten on load: `PulseChannel.HasSweep` and
  `LengthCounter.Maximum`. The enum `PpuMode` is an i32. A bool is one byte, and any nonzero byte reads as true.
- **Sizes.** Tetris 17,338 bytes, Kirby 17,348, Link's Awakening 41,924 (three copies of 8 KB cart RAM), Pokémon Yellow
  148,420 (three copies of 32 KB, 16 KB VRAM, 32 KB WRAM).
- **Not in the state, and so kept across a load:**
  - the picture (`FrameRgba`);
  - the APU's fractional sample accumulator, both high-pass capacitors, and its sample queue (`Apu.cs:40-51`);
  - the joypad's held buttons (`MemoryBus.cs:46`);
  - the serial log.

  A C# `LoadState` does not reset these; it leaves the instance's own values. So what a loaded state sounds like
  depends on the instance's history, and MercuryRT must behave the same way.

### 3.2 How the reader and writer are proven

Mercury uses `Mars_Native.md` §5.1's three separate proofs, unchanged:

1. **Byte round trip.** A C# state is read by MercuryRT and written back, and the two are compared byte for byte. The
   inputs are §1.1's four games at frames 0, 900 and 3,900, the double-speed synthetic, and one synthetic ROM per board
   (no MBC, MBC1, MBC2, MBC3+RTC, MBC5+rumble), from `SyntheticGbRom`.
2. **A layout listing.** It has the form `offset len type path` and is compared line by line against a C# reflection
   walk of the same objects.
3. **`naming.rs`.** It checks every label against the field actually written.

The byte round trip is blind to a mislabelled field of the right width, which is why MarsRT's third proof caught what
the first could not. That applies here with more force, because three copies of the cartridge give many same-width
fields to swap.

### 3.3 The frame-by-frame comparison

Both engines are loaded fresh, with `LoadRom` then `LoadState`, so that the skipped fields of §3.1 start at their reset
values on both sides. They run the same input script. After every frame three things are compared:

- **state:** the C# save state of each;
- **picture:** the RGBA frame, once the renderer is ported;
- **sound:** the drained samples, once the mixer is ported.

The inputs are §1.1's four games and the double-speed synthetic: 3,000 frames each, from boot and from states taken at
frame 900, with the bench's Start/A script.

**The mixer can be bit-exact (argued).** The mixer is IEEE doubles in a fixed order (`Apu.cs:358-412`). Neither RyuJIT
nor rustc contracts to FMA by default, and the clamp-then-truncate conversion to `short` matches Rust's `as i16` for
in-range values. The one transcendental is `Math.Pow` for the high-pass charge factor (`Apu.cs:51,58`). It is evaluated
once, in C#, and passed down, so no platform's `pow` has to agree with another's.

### 3.4 The hardware corpus, line for line (measured, first run)

**Getting the corpus.** Neither `~/.config/EmuSen/TestRoms/hardware/` nor the checkout's `TestRoms/` had the corpus. So
WiseMan's `A_hardware_test_rom_reports_a_pass` has been passing vacuously on this machine, by design
(`Mercury_HardwareTests.md` §3). Both corpora were fetched into scratch space: blargg's `gb-test-roms`, and mooneye's
`mts-20240926-1737-443f6e1`.

**Reading verdicts.** 173 ROMs were run on the C# Mercury, capped at 3,600 frames, with WiseMan's own verdict rules.
blargg's newer suites (`dmg_sound`, `cgb_sound`, `oam_bug`, `mem_timing-2`) report through **cartridge RAM, not the
serial port**: signature `DE B0 61` at `$A001`, status at `$A000` (`$80` while running), text from `$A004`. WiseMan's
serial-only reader returns NoVerdict for all 39 of them, so the run added a reader for that protocol.

| Suite | Passed | Of | Notes |
|---|---|---|---|
| blargg `cpu_instrs` | 12 | 12 | combined ROM and all eleven singles |
| blargg `instr_timing` | 1 | 1 | |
| blargg `mem_timing` | 4 | 4 | |
| blargg `mem_timing-2` | 4 | 4 | through cart RAM |
| blargg `dmg_sound` | 4 | 13 | through cart RAM; failures 01, 03, 05, 08, 09, 10, 11, 12 and the combined ROM |
| blargg `cgb_sound` | 6 | 13 | through cart RAM; failures 01, 03, 05, 09, 11, 12 and the combined ROM |
| blargg `oam_bug` | 2 | 9 | through cart RAM |
| blargg `halt_bug`, `interrupt_time` | — | 2 | screen only, no machine-readable verdict |
| mooneye `acceptance` | 34 | 75 | see below |
| mooneye `emulator-only` | 26 | 28 | failures: `mbc1/multicart_rom_8Mb`, `mbc1/ram_64kb` |
| mooneye `misc` | 0 | 8 | all are CGB or AGB model tests on a DMG header, which Mercury runs as a DMG by design |
| mooneye `utils`, `madness`, `manual-only` | 1 | 4 | not auto-verdict ROMs, except `utils/dump_boot_hwio` |
| **Total** | **94** | **173** | |

mooneye `acceptance`'s 41 failures break down as follows:

- **11 `boot_*` ROMs.** Nine are for models Mercury does not claim (dmg0, mgb, sgb, sgb2, S). `boot_div-dmgABCmgb` and
  `boot_hwio-dmgABCmgb` fail on the model Mercury does claim.
- **13 instruction-timing ROMs** (`call_timing`, `push_timing`, `ret_timing`, `rst_timing` and the rest). These are
  `Mercury_Gameplan.md` §4's first simplification, internal cycles settled at the end of an instruction, seen through
  OAM DMA.
- 3 `oam_dma_*` ROMs;
- `ie_push`;
- 8 PPU interrupt and LCD-on timing ROMs;
- 3 timer reload ROMs;
- the serial clock alignment ROM;
- `unused_hwio-GS`.

**Retiring the page's earlier predictions.** This is the first run of the corpus. It retires `Mercury_HardwareTests.md`
§4's predictions, which that page says were written before the ROMs were obtained:

- `cpu_instrs` passes, and `instr_timing` passes, as predicted.
- `mem_timing` and `mem_timing-2` pass, as predicted.
- `oam_bug` fails, as predicted, though 2 of its 9 ROMs pass.
- `dmg_sound` partly fails, as predicted (4 of 13 pass).
- `cgb_sound` passes 6 of 13.

That page should record this. This plan does not edit it.

**For the port, the corpus is a transcript, not a pass list.** WiseMan's theory asserts `Passed`. With the corpus
installed it would fail 79 of 173, so it cannot be the port's oracle as it stands. The oracle is **equality of the two
engines' transcripts**. For every ROM that means the same verdict, on the same frame, with the same serial bytes, the
same cart-RAM text, and the same state at the verdict frame. Behind that sits a recorded C# baseline, so that a change
to either engine shows as a line.

### 3.5 WiseMan's tests against both engines

Mercury has about 226 test cases in 164 methods, across `EmuSen.WiseMan/Cores/Mercury*Tests.cs` and parts of
`CoreFactoryCheatWiringTests`. They reach into the C# objects directly. `MercuryPpuTests` names `Bus` 134 times,
`MercuryCgbTests` 144 and `MercuryApuTests` 105. MarsRT solved the same problem by making `MarsDebugTests` abstract,
with one rig per engine (`Mars_Native.md` §6.5). The proposal for Mercury is a `MercuryRig`:

- **Reads** go through a C# view deserialised from the engine's state, as in §2.4. So `rig.Bus.Ppu.Ly` means the same
  thing on both engines, and most assertions carry over unedited.
- **Setup writes** go through `write_space`. The rest, such as tests that set a device field directly, go through a
  state round trip: edit the view, then load it into the engine.
- **Timing** uses `run_frame`, `step` and `tick`.

Tests that are about C# alone stay C#-only, and the page that records the parity lists them by name:

- the reflection serializer's own tests;
- `SkipRendering`'s allocation checks.

### 3.6 Mutants

The mutants follow `Mars_Native.md` §5.2.4's rule: every stage ends with hand-made mutants of the Rust code, each
caught by a named oracle, and survivors get a test.

Two classes deserve mutants on purpose:

- **ordering in `StepOneCycle`**, such as the timer edge before the reload delay, or the DMA before the DIV increment;
- **the three cartridge copies** in the state.

The first could hide below game-level comparison and be caught only by mooneye's timer ROMs. The second could hide below
a byte round trip and be caught only by `naming.rs`.

### 3.7 When the C# core leaves the main branch

*Decided 2026-09-23:* every C# core will eventually move to its own legacy branch. The comparisons of
§3.2 to §3.5 need both engines, so they have to be kept in a form that survives the C# core leaving `main`. The
proposal has three layers, each covering what the one before it cannot.

**1. Golden traces, recorded from the C# engine and committed.** Each golden run records, per frame, an 8-byte
truncated SHA-256 of three things: the C# save state, the RGBA frame, and the drained samples. That is the
pacebench "state sha" convention. Each run also records its input script and the ROM's SHA-256. The corpus contributes
its transcript per ROM (§3.4): the verdict, the frame, the serial bytes, and the cart-RAM text.

A 3,000-frame run is about 72 KB of hashes before compression. The recorded runs are:

- every synthetic ROM, which `SyntheticGbRom` rebuilds in the test, so ROM and trace are both reproducible from the
  repository alone;
- every corpus ROM;
- §3.3's four games.

A hash contains nothing of the ROM, so the traces of commercial games can be committed, and their tests skip when the
ROM is absent, as `CommercialRomLibrary`'s already do. MercuryRT is then graded against these files on `main` for as
long as the traces stand.

**2. Full states at intervals, kept locally.** A hash says that a run diverged, not where. So each golden run also
keeps a full C# state every 300 frames: 17–148 KB each, ten a run. They are gitignored, kept in a local cache beside
the probe's (`~/.cache/emusen/`), and are the input for bisection. Once a failing trace names a frame, the nearest
earlier state is loaded into MercuryRT and run to that frame. The two states are then diffed with the layout listing
of §3.2, which names every byte. For synthetic ROMs these states can be regenerated from the legacy branch at any
time.

**3. A live differential on the legacy branch, built on demand.** *Dropped 2026-10-16 by the tester's decision
(§14.7); the rest of this paragraph is the proposal as written.* The legacy branch keeps C# Mercury and its WiseMan
tests. A test project there loads MercuryRT's library built from `main`, by a path, as `RustCoresPublish.targets`'
`EmuSenNativePrebuilt` already does for foreign platforms. It then runs §3.3's frame-by-frame comparison live. This is
the tool for when a golden trace disagrees and the question is "which engine moved?". Three things couple the branches:

- the C ABI, whose interface version the legacy project pins;
- the state format;
- the input script format.

**Three rules keep this honest.**

- **Goldens are recorded from the C# engine only, and before the move.** Recording them from MercuryRT would make it
  self-graded.
- **A deliberate change to MercuryRT's behaviour after the move is a re-baseline.** It is a commit that replaces the
  affected traces and states its reason. The trace files' history becomes the record of every intended departure from
  the C# oracle.
- **The format authority passes to Rust.** When the reflection walk leaves `main`, the committed layout listings (one
  per board, from `state_layout`) become the state format's specification. A state written by the legacy branch still
  loads on `main`, which is the check that the two have not drifted.

## 4. Stages

The costs are in `Mars_Native.md` §6's units and are held loosely. For scale, MarsRT's stages 1a to 5 were all dated
2026-09-22, with several lines of work in parallel, on a crate now about 35,000 lines. Mercury's machine is about 3,550 lines of
C#: the 4,620 in its folder, less the debug target (637), the disassembler (194), the cheat codecs (124) and the
memory-space naming (110).

| Stage | What it covers | Its oracle | Cost |
|---|---|---|---|
| 0 | Decisions Q3 and Q6 (§5); the corpus installed under `TestRoms/hardware/`; WiseMan's verdict reader taught blargg's cart-RAM protocol; a Mercury bench and a corpus-transcript command in the harness; the C# baseline recorded | The baseline table of §3.4, reproduced by the harness | hours |
| 1 | The crate skeleton; Rust structs for every serialised type, including all five boards; `state.rs` with a string primitive; the layout listing and `naming.rs` | §3.2's three proofs | hours |
| 2 | The machine core, without pixels or mixer output: SM83, bus decode and access blocking, timer, OAM DMA, HDMA with its stall, serial sink, joypad, interrupts, all five boards with MBC3's clock, KEY1 and double speed, the PPU's mode machine with `WindowLine`, and the APU's registers, channels and sequencer (they are state) | The corpus transcripts (§3.4); random SM83 programs compared after every instruction; four games and the synthetic ROMs by state every frame (§3.3); the first speed number, against C# with `SkipRendering` (§1.3) | half a day to a day |
| 3 | The mixer: `EmitSample`, the high-pass, the queue and its drop-oldest rule, `Drain` | Samples identical every frame against C#; `Apu` unit tests through the rig | hours |
| 4 | The renderer: DMG and CGB background, window and sprites, both priority orders | RGBA identical every frame; the synthetic PPU and CGB tests through the rig | hours |
| 5 | Shim and frontends: `MercuryRtCore`; an Engine row for GB in `CoreCatalog.EngineFor`; `CoreFactory.Create`; battery saves; cheats, both codecs, with the Game Genie table; `FrameLog`; exception mapping; crash log | The seven commercial-ROM suites on both engines; states crossing engines in both directions; the fallback when the library is absent | a day |
| 6 | Debugger hooks: breakpoints, stepping, coverage and the write log as pushed tables and drained logs; `MercuryRtDebugTarget` over the state view (*done 2026-09-24, §8.5*) | `MercuryDebugTargetTests` on both engines; games with every table armed identical to plain runs | 1–2 days |
| 7 | Platforms and CI: the second crate in the workflow's matrix; `MarsRtPublish.targets` and `EmuSen.csproj`'s single-library build generalised to a list of crates | Crate tests on four platforms; WiseMan's synthetic systems per platform | hours to half a day |
| 8 | Parity: the rest of WiseMan's Mercury tests through `MercuryRig`; mutants for every stage not yet covered (*done 2026-10-14, §12*) | §3.5, §3.6 | a day |
| 9 | Goldens: traces and interval states recorded from C# (§3.7), MercuryRT graded against them on `main`, and the legacy-branch differential project written and run once (*done 2026-10-15, §14; the differential dropped 2026-10-16, §14.7*) | Every golden reproduced by MercuryRT; one deliberately broken trace caught | half a day |
| 10 | The C# core's move to its legacy branch (Q5) | A decision, and stage 9 passing on `main` without it | — |

**What "finished" means** is `Mars_Native.md` §6's three criteria, unchanged:

1. A player gets MercuryRT on every platform, in the measured configuration, without choosing it.
2. Nothing the C# core can do is lost, or the loss is written down.
3. The C# core's role is decided.

Criterion 2's written list would begin with whatever §6.5 of that page lists for MarsRT that Mercury also has, such as
rewind. Mercury has no `ISnapshotCore`, so rewind uses the full state on both engines.

## 5. Risks and open questions

- **Q1, naming.** *Decided 2026-09-23:* MercuryRT, in `MercuryRT - GB/` (§2.2).
- **Q2, one native library or two.** A second `cdylib` means a second file shipped and a second interface version. It
  also means either a copy of MarsRT's 410-line `state.rs`, which needs a string primitive MarsRT lacks, or a small
  shared crate both depend on. The shared crate touches MarsRT, where other work is in progress. The recommendation is
  a separate crate now and the extraction once both are stable, but the decision is still open.
- **Q3, whether to fix §6.1's C# defects before the port.** The recommendation is yes, each proven by a test before and
  after, because after the port every fix costs two implementations in lock-step:
  - D1 (sample rate) changes the sound but not the state;
  - D2 (the clock in double speed) changes how state evolves, but not the format;
  - D3 (the host path in the state) and the triple cartridge copy are one format change, to state version 6. That
    shrinks Yellow's state by 64 KB and removes `_savePath` and the string primitive the port would otherwise need.
  *Decided 2026-09-24: fix all three now, in both engines in lock-step, each shown by a test before and
  after; D3 is state version 6, and version-5 states still load.*
- **Q4, whether the port carries Mercury's known simplifications.** The simplifications include the per-line renderer,
  mode 3 not lengthening for sprites, internal cycles at the end of an instruction, instant GP-HDMA, and the 79
  non-passing corpus ROMs of §3.4. The recommendation is to port them exactly and improve afterwards, because an
  improvement made during the port has no oracle. After the port, improvements are made either in both engines, while C#
  is the oracle, or in Rust alone with the corpus as the oracle, once C# is frozen. That second choice was left open.
  *Decided 2026-09-24: improvements after the port land in both engines in lock-step, with C# the
  oracle, until the C# core moves to its legacy branch.*
- **Q5, the C# Mercury's future.** *Partly decided 2026-09-23:* every C# core will eventually move to a
  legacy branch. §3.7 says how the oracle survives the move. Two questions stay open: when the Engine row's default
  flips to MercuryRT, and whether C# is still offered to players until the move.
  *Decided 2026-09-24: the Engine row's default flips to MercuryRT after stages 6 and 7 and a session of
  play on the handheld, the condition MarsRT's flip had (`Mars_Native.md` §6.2).*
- **Q6, the corpus's place.** Is the corpus kept gitignored under `TestRoms/hardware/`, as `Mercury_HardwareTests.md`
  §3 intends? Is WiseMan's theory changed from "passes" to "matches the recorded baseline"? That changes a committed
  test's meaning, which is why it is asked rather than done.
  *Decided 2026-09-24: the corpus stays gitignored under `TestRoms/hardware/`, and the committed theory
  becomes "matches the recorded baseline", so a verdict that changes in either direction fails until the baseline is
  re-recorded.*
- **Risk: P1 is wide by design.** No component like Mercury's tick loop was measured in Mars. If stage 2's number comes
  in under 1.2×, the port is still worth finishing on grounds 1 and 3 of §1.2. The later lever, batching the devices
  (§1.2), would then be the next question, with its own prediction.
- **Risk: bit-exact sound across platforms.** §3.3 removes `pow`. Nothing else in the mixer is platform-dependent by
  argument. That argument is untested on Windows and macOS until stage 7. *Stage 7 (§8.6) wrote the test, as digests
  recorded on linux-x64 that the workflow holds win-x64 and osx-arm64 to. The result waits on the workflow's first
  run.*

## 6. What the review found in the C# core

### 6.1 Defects demonstrated (measured 2026-09-23, on the unmodified WiseMan build)

- **D1: the APU produces 44,150.6 samples a second and labels them 44,100.**
  - *Cause.* `SetSampleRate` computes `_cyclesPerSample = MercuryCore.CpuClockHz / sampleRate` in integer arithmetic
    (`Apu/Apu.cs:57`), which gives 95 rather than 95.108. The field's initialiser (`Apu.cs:46`) is right, but
    `LoadRom` overwrites it (`MercuryCore.cs:107`).
  - *Measured.* Every run in §1.1 drained 739.200 stereo frames a frame, where 44,100 Hz wants 738.353: 0.115% fast.
  - *Documentation.* `Mercury_Apu.md` §6 says "95.11 clocks per sample, with the remainder carried so the rate cannot
    drift". The remainder is carried, but the rate is wrong.
  - *Consequences.* A frontend's audio sync absorbs about 51 extra samples a second. The audio differential against
    gambatte (`Mercury_HardwareTests.md` §6) compared against a mislabelled rate.
- **D2: MBC3's real-time clock runs twice as fast in CGB double speed.**
  - *Cause.* `MemoryBus.Tick` passes CPU cycles to `Mapper.Tick` (`MemoryBus.cs:386`), and `Mbc3.Tick` counts them
    against `CpuClockHz` (`Mbc3.cs:88-98`). The cartridge's clock has its own 32.768 kHz crystal and does not double.
  - *Measured.* On a synthetic MBC3+TIMER CGB ROM, 1,195 frames (20.01 s of console time) read 20 s at single speed and
    **39 s** after `KEY1` + `STOP`.
- **D3: the host's save path is machine state.**
  - *Cause.* `Cartridge._savePath` (`Cartridge.cs:43`) is not `[SkipInState]`.
  - *Measured, first case.* A state saved while playing a ROM at path A and loaded while playing the same ROM at path B
    made B's `SaveSram` write **A's** `.srm`. B's was never created. This was observed with two scratch copies.
  - *Measured, second case.* Under `--nobattery`, `_savePath` is null and is written as `""`. Loading such a state sets
    it to `""`. The next 300th-frame `SaveSram` (`MercuryCore.cs:177`) then calls `AtomicFile.Write("", ram)`. That
    **writes the cart RAM to a file named `.tmp` in the process's working directory** (8,192 bytes for Link's
    Awakening, observed), and then fails the rename, silently (`AtomicFile.cs:34-41`).
  - *Consequences.* A frontend's `--nobattery` sandbox is not sandboxed after a state load. States do not travel
    between machines without redirecting saves.

### 6.2 Defects argued, not demonstrated

- **No joypad interrupt, and DMG `STOP` does nothing.** `Interrupt.Joypad` is declared (`MemoryBus.cs:17`) and never
  requested anywhere in the tree. `Stop()` returns at once unless a CGB speed switch is armed
  (`MemoryBus.Cgb.cs:119-121`).
  - `STOP` is in `Mercury_Gameplan.md` §4; the missing interrupt is not.
  - A game that sleeps in `STOP` or `HALT` waiting for a button keeps running, or waits forever.
  - No ROM was found that shows it.
- **`boot_div-dmgABCmgb` and `boot_hwio-dmgABCmgb` fail on the model Mercury claims (§3.4).** The post-boot DIV phase
  (`MemoryBus.Reset` zeroes the counter) and the post-boot I/O values are not the DMG boot ROM's. This is known in
  spirit (`Mercury_Cpu.md` §5) and now has a failing ROM.

### 6.3 Places the design does not port directly

- **The state format is a reflection walk** (`StateSerializer.cs:36-44`). It carries backing-field names, ordinal
  order, the triple cartridge and a string (§3.1). The port writes it by hand, as `Mars_Native.md` §5.1 did, and
  proves it three ways.
- **Interfaces in the hot loop:**
  - `ICpuBus` (`Cpu.cs:16`), called twice per memory access (`Cpu.cs:139-148`);
  - `IMapper`, for every cartridge read and write, and `Tick` on every access (`MemoryBus.cs:122,131,160,174,386`);
  - `IRomReadPatcher` on every ROM read (`MemoryBus.cs:123`);
  - a nullable `IWriteObserver` on every write.

  In Rust these become a concrete bus, an `enum` of boards, a pushed table and a drained log. On Venus's evidence,
  removing the virtual calls is not itself where the gain is (§1.3).
- **Debugger seams on the plain path.** `Breakpoints.ShouldBreak` runs before every instruction with nothing armed. It
  was 6.7% of samples with inlining (`MercuryCore.cs:136`). `Coverage.IsArmed` runs per instruction (`:144`). The ROM
  patcher is always non-null (`MercuryCore.cs:46,101`), so `CheatRegistry.TryPatchRom` runs on every ROM read (0.9%).
  A C# gate on "anything armed" would give C# the same ≈7% back. That would move P1's baseline, so it belongs to Q3's
  decision.
- **Delegates at the end of a frame.** They are `FrameLog.RecordFrame(TotalFrames, ReadForFrameLog)` and
  `Cheats.ApplyAll(ReadForCheat, WriteForCheat)` (`MercuryCore.cs:174,181`). They stay in C#, over `read_space` and
  `write_space`.
- **Exceptions as control flow.** An unimplemented opcode throws (`Cpu.Opcodes.cs:202-204`), and so does an
  unsupported board (`Cartridge.cs:144`). Under `panic = "abort"` a Rust panic would kill the frontend, so both become
  statuses, and the shim rethrows the same exception with the same message.
- **The picture is handed out live.** `GetFrameBufferRgba` returns the PPU's own array (`MercuryCore.cs:183`), which the
  next frame overwrites in place. MarsRT returns a copy. A frontend presenting on another thread can see a torn Mercury
  frame today. The shim's copy changes that behaviour, for the better, and the change should be written down.
- **Collections that allocate or grow.** The APU's `Queue<short>` is dequeued one sample at a time, and `Drain`
  allocates each call (`Apu.cs:40,418-428`), 3.2 KB a frame. The serial log is a `List<byte>` that grows for the whole
  session (`MemoryBus.cs:62,347`); `SerialText` copies it all (`:95`). In Rust these become a ring buffer and a bounded
  log, and the bound is the only behavioural change.

### 6.4 Dead or inert code (the port still carries whatever is in the state)

- `Cartridge.SuperGameBoy` is parsed (`Cartridge.cs:72`) and never read.
- `Mbc5._rumbling` is computed (`Mbc5.cs:50`) and reaches only `DebugState`. No rumble output reaches a frontend.
- `Ppu.FrameCount` and `Cpu.Cycles` are in the state and read only by tests.
- `Cartridge.HeaderChecksumValid` is computed and never enforced. That is documented (`Mercury_Cpu.md` §5).
- There are no `TODO`, `FIXME` or `HACK` markers in the core.

### 6.5 Negative results

- `SkipRendering` was checked for leaks into the machine. It does not change the state: the state hashes were identical
  with rendering on and off at the same frame in all three games. Its bookkeeping outside the guard (`WindowLine`) is
  correct.
- The three copies of the cartridge are not a correctness defect. They are one object, so a load cannot desynchronise
  them. They cost only size.
- None of the measured games uses double speed or HDMA's general-purpose mode under load. Stage 2's oracle leans on
  the synthetic ROMs and `cgb_sound` for those paths, as `Mercury_RealCartridges.md` §6.1 already says for the C# core.

## 7. Predictions to be retired

| # | Prediction | Retired when |
|---|---|---|
| P1 | A line-for-line MercuryRT's plain frame takes 0.50–0.80 of C#'s time, central 0.62 (1.6×), on the four games | Stage 2, re-priced like for like against `SkipRendering`; final at stage 5 through the shim |
| P2 | The renderer ports at about 2× (1.6–2.4×) | Stage 4 |
| P3 | No sampled time on breakpoint, coverage or patch checks in MercuryRT's plain frame, against about 7% in C# | Stage 6, with every table disarmed |
| P4 | Nothing observable changes on the desktop or at 33% quota; Yellow at 10% quota rises from 178% to 250–350% | Stage 5 |
| P5 | The once-a-frame boundary costs under 3% of MercuryRT's frame | Stage 5 |
| P6 | No recompiler and no threads are needed for MercuryRT to beat C# on every measured configuration | Stage 5; refuted by any configuration in which it does not |

*Retired 2026-09-23 (§8.2):* P1 is refuted, with 0.82–1.02 of C#'s time measured. P2 is refuted in direction: the
renderer is slower in Rust. P5 holds. P3, P4 and P6 are still open.

*Retired in part 2026-09-24 (§8.5.5):* P3 holds by construction for breakpoints and coverage, not by sampling. The plain
frame has no breakpoint or coverage check, because those seams are compiled into the observed step alone. The patch check
remains in the plain frame. With no Game Genie code enabled it is a test of an empty `Option` on each ROM read (§8.3),
and it was not sampled, so that third of P3 is open. P4 and P6 are still open.

## 8. The work, stage by stage

### 8.1 Stage 1: the state, byte for byte (done 2026-09-23)

*Order.* Stage 1 was done before stage 0. The state reader needs no corpus, and stage 0's two decisions (Q3, Q6) were
still open.

**What was built.** The crate is `EmuSen/Cores/Nintendo/MercuryRT - GB/`. It holds Rust structs for every serialised
type, and a `State` impl for each class C# walks. `Mapper` is an enum of the five boards, where C# has an interface.
There is no board tag in the state, so the header builds the board and the state then fills it. The crate has:

- `state.rs`: MarsRT's codec plus `BinaryWriter` strings, with the 7-bit length and `ReadString`'s refusals;
- `machine.rs`: the header, then the cartridge, board, CPU and bus walks;
- a C ABI of `mercury_machine_new`, `_free`, `_load_state`, `_save_state_size`, `_save_state` and `_state_layout`;
- `naming.rs`, MarsRT's rule applied to eight modules.

On the C# side there are `Shim/MercuryNative.cs` (the loader, `EMUSEN_MERCURY_NATIVE=0` to turn it off) and
`Shim/MercuryMachine.cs` (the handle). `EmuSen.csproj` builds the crate with cargo beside MarsRT's, into
`obj/mercuryrt`, and copies `libmercuryrt.so` into every consumer's output.

**The oracles, all identical** (`EmuSen.WiseMan/Cores/MercuryRtStateTests.cs`, 25 cases; crate tests, 11):

- **Running machines.** Eleven synthetic ROMs cover every board, with and without RAM, battery, clock and rumble, on
  both consoles. Each runs a program that counts in WRAM, copies the count to cart RAM, keys a pulse note and scrolls,
  for 300 frames. The C# state goes into Rust and back out byte for byte. Its layout listing matches C#'s own
  reflection walk line for line: 170–177 fields, 17,338–410,564 bytes.
- **Noise.** The same eleven, with every serialised field filled with distinct noise. The save path is non-ASCII and
  long enough for a two-byte length.
- **Bytes no C# writer makes.** Four bools and a class flag of 2, and a PPU mode of 7, read as C#'s reader reads them.
- **Refusals.** A truncated state, a foreign magic and a wrong version change nothing. A short image and an unsupported
  board are refused, and the board with C#'s exception type.
- **Real games.** Tetris, Link's Awakening, Kirby's Dream Land and Pokémon Yellow are identical at frames 0, 300, 600
  and 900, with the bench's input. They run from `EMUSEN_MERCURYRT_ROMS`, a directory of scratch copies; absent, the
  case passes without running.

**Mutants: 10, all caught.** The runner (scratch, `mutants.py`) applies one edit, runs `cargo test` and WiseMan's
`MercuryRt` filter, then restores the file. Only one oracle caught each of four mutants:

| Mutant | Caught by |
|---|---|
| Cpu `A`/`B` swapped in the writer | crate, WiseMan running machines |
| String length prefix continuing at 127 | crate only; WiseMan's 150-byte path is past 127 either way |
| An unknown PPU mode written back as 0 | WiseMan odd-bytes case only |
| `_baseClockPhase` read into `_lastTimerEdge` | crate, WiseMan noise |
| MBC3's `_cart` written after `_clock` | crate, WiseMan running and noise |
| A bool read as `== 1` | WiseMan odd-bytes case only |
| The save path not read back | crate, WiseMan noise |
| Envelope `Period`/`Timer` swapped in the writer | crate, WiseMan running |
| A failed load keeping the partial read | crate only |
| `LengthCounter.Maximum` not read | WiseMan noise only; it never changes in a running machine |

**What stage 1 does not establish.** It establishes that MercuryRT holds a C# state. It says nothing about running one.
The skipped fields of §3.1 exist in Rust (`Skip<T>`) but are untouched by any test yet.

### 8.2 Stages 2–4: the machine, the mixer and the renderer (done 2026-09-23)

*Order.* §4 planned three stages: the machine first, then the mixer, then the renderer. They were written together and
proven together, because the oracle below compares all three every frame. Each claim can still be read on its own:
state equality with `SkipRendering` on proves the machine without the renderer, sample equality proves the mixer, and
RGBA equality proves the renderer.

**What was built.** Every line of the C# machine has a Rust counterpart, in the C# order:

- `cpu/`: the SM83's step, opcodes, `$CB` table and ALU, generic over a `CpuBus` trait so the step is monomorphised;
- `memory/bus.rs`: decode, access blocking, the timer's edge detector, OAM DMA, both HDMA modes and their stall,
  KEY1 and STOP, the serial sink, the joypad matrix, and the per-T-cycle clock;
- `memory/mappers.rs`: the five boards as an enum, MBC3's clock included;
- `ppu/`: the mode machine, the STAT line, the window counter and the line renderer;
- `apu/`: the channels, the sequencer and the mixer.

The known C# defects are copied deliberately:

- D1's integer division, with `Math.Pow` evaluated in C# and passed down;
- D2's clock counted in CPU cycles;
- D3's save path in the state.

The C ABI gained the calls a frame needs: run a frame, step, buttons, options, sample rate, mutes, picture, audio and
serial. `examples/frames.rs` runs the machine in a process with no .NET in it.

**The oracle** (`EmuSen.WiseMan/Cores/MercuryRtMachineTests.cs`, 38 cases, 7 min 22 s). It compares the whole C#
state after every frame, and after every instruction where stated. It compares the drained samples and the RGBA frame
after every frame. It compares the serial log. When they differ, it names the first differing field from the layout.
Everything below was identical:

- **Synthetic programs.** Three programs ran on all eleven boards, 600 frames each.
  - A counter.
  - An interrupt program: timer, STAT, vblank and serial handlers into HALT, OAM DMA every vblank, a window, sprites,
    and a wave note.
  - A colour program: both HDMA modes every frame, VRAM and WRAM banking, palette ports, and KEY1 plus STOP toggling
    double speed at frames 60, 316 and 572.
- **Random programs.** On every board, 16 random ROMs ran at 5,000 instructions each, compared after every instruction:
  775,483 instructions. Four seeds keep the eleven illegal opcodes; 26 programs stopped on one, in both engines, at the
  same instruction, with C#'s message.
- **Games.** Tetris, Link's Awakening, Kirby's Dream Land and Pokémon Yellow ran 3,000 frames from boot with the bench's
  input. Each then ran 600 frames from a state C# saved at frame 3,000, loaded into both engines fresh.
- **The corpus.** All 173 ROMs of §3.4 ran to their verdicts or 3,600 frames: 31,986 frames. The serial log was
  compared every frame, and the state every 30 frames and at the end.

A first attempt at the transferred-state case gave the expected failure. C# was continued from frame 3,000 while Rust
was loaded fresh, and the sound differed at frame 1. §3.1's skipped mixer fields carried the C# instance's history, which
no state holds. Loading both engines fresh made them identical. The test resets the fresh C# core's save path to null
after loading, because D3 would otherwise make its 300th-frame save write `.tmp` into the working directory.

**The first like-for-like speed (measured, retiring P1 and P2).** Three interleaved rounds were run with
`mercbench`, flat out, under the bench lock, with the picture copied and the audio drained every frame on both engines.
Both engines ended every run with the same state hash.

| Game | C# p50 ms (3 rounds) | MercuryRT p50 ms | C#/RT |
|---|---|---|---|
| Tetris | 0.641 / 0.669 / 0.627 | 0.553 / 0.790 / 0.541 | 1.16 / 0.85 (a noisy round) / 1.16 |
| Link's Awakening | 0.560 / 0.574 / 0.618 | 0.505 / 0.508 / 0.508 | 1.11 / 1.13 / 1.22 |
| Kirby's Dream Land | 0.681 / 0.701 / 0.704 | 0.596 / 0.604 / 0.592 | 1.14 / 1.16 / 1.19 |
| Pokémon Yellow | 0.705 / 0.705 / 0.712 | 0.720 / 0.711 / 0.711 | 0.98 / 0.99 / 1.00 |
| Double-speed loop | 0.912 / 0.896 / 0.897 | 0.759 / 0.757 / 0.827 | 1.20 / 1.18 / 1.08 |

With the renderer skipped, both engines ran the same machine, the like-for-like comparison §1.3 named:

| Game | C# p50 ms | MercuryRT p50 ms | C#/RT |
|---|---|---|---|
| Tetris | 0.528 / 0.539 / 0.545 | 0.396 / 0.391 / 0.386 | 1.33 / 1.38 / 1.41 |
| Pokémon Yellow | 0.565 / 0.554 / 0.558 | 0.531 / 0.525 / 0.512 | 1.06 / 1.06 / 1.09 |

- **P1 is refuted.** A line-for-line MercuryRT takes 0.82–1.02 of C#'s time on whole frames (about 1.0–1.2×), against
  a predicted 0.50–0.80 with a central 0.62. Yellow gains nothing. The machine core gains 1.06–1.41×, near the
  pessimistic end of the row §1.3 marked as unmeasured. Why Yellow's machine gains a sixth of what Tetris's does is not
  explained. Both run the same per-cycle loop.
- **P2 is refuted in direction.** The renderer is slower in Rust. It is the difference between the two tables:
  0.14–0.16 ms against C#'s about 0.10 on Tetris, and 0.19 ms against about 0.15 on Yellow.
- **P5 holds.** `examples/frames` runs the same frames with no .NET in the process: Tetris 0.513/0.519 ms whole and
  0.370/0.369 skipped, Yellow 0.692/0.691 and 0.508/0.522. That is the same as through the ABI, so the once-a-frame
  boundary costs nothing measurable.

**Where MercuryRT's time goes** (`rtsample.py`, 5,558 samples of Yellow). By innermost inlined function, the per-cycle
clock takes about 52%:

- `tick`, 18%;
- `step_one_cycle`, 16%;
- the channels' `step_timer`, 7%;
- `step_oam_dma`, 6%;
- `on_div_bit`, 2%;
- the counters' `wrapping_add`, 7%.

The renderer takes about 21%: `render_background` 12%, `color_pixel` 5%, `tile_row_address` 2% and `expand` 2%. This
is the same picture §1.1 drew for C#. The port moved the costs without changing them, and the cost is the design (one
step of every device per T-cycle), not the language. The lever named in §1.2 (catching devices up in batches), and a
renderer written for the compiler rather than line for line, are the next questions. Each needs its own prediction.

**The debugging trap met on the way, and an intermittent failure it explains.** The first mutant run stopped with
every command's output failing. `/tmp` is a 16 GB tmpfs, shared with other processes' scratch space, and it had filled.
The crate's build output now lives under `~/.cache/emusen/probe/mercuryrt/`, as the project keeps probe binaries.

In the main checkout, the first run after the merge failed five random-program cases: one after 7 s, then four in
under a millisecond each, with no message captured. The failure did not reproduce in three later runs. Each case writes
16 temporary ROMs through `SyntheticGbRom.WriteTemp`, and that pattern is what a filling temporary directory produces.
It was reproduced: the same binary ran with `TMPDIR` on a private 64 MB tmpfs (`unshare -rm`), and the tmpfs was filled
20 s in. One case failed after 18 s, and the other ten failed in 1 ms or less, each with `IOException: No space left on
device` on its ROM file.

Two other explanations were tried and do not match:

- **A mutant library left in a build output.** Mutant M1 fails six cases, each after seconds, on `Cpu.F`.
- **Overwriting the loaded library during a run**, with a different build or a byte-identical copy. Both crash the test
  host outright.

The failure is environmental, and no code was changed for it.

### 8.3 Stage 5: the shim and the Engine row (done 2026-09-23)

**What was built.** `Shim/MercuryRtCore.cs` implements `ICore`, `ICheatRegistryHost`, `IStateFormat` and
`IFrameBufferPool` over the machine:

- **Header.** It parses the header with the C# `Cartridge`, so a bad image throws C#'s own exceptions.
- **Battery saves.** It reads the battery save as C# reads it. It writes it with the path the machine's state carries,
  as C# does (D3).
- **Frame.** It runs C#'s end-of-frame work in C#'s order: frame log, cheats, and the 300th-frame save.
- **Picture.** It lends picture copies through `FrameBufferLending`, as MarsRT does. C# Mercury hands out its live
  array (§6.3).

`CoreCatalog.EngineFor("GB")` offers *Mercury (C#)*, the default, and *MercuryRT (Rust)*. `CoreFactory` builds MercuryRT
only when it is asked for and available. `EngineNotice` says why it is not running when it is not. The Engine row
appears on Mistress's GB tab through the existing window code.

**Game Genie without calls from Rust to C#.** `CheatRegistry` gained a `Version`, bumped by every change. When it moves,
the shim flattens `TryPatchRom` into a table: one entry per address a patch touches, 256 entries of `0x100 | patched`
or 0. It finds the addresses by probing each ROM address with every compare byte in use, plus zero, then asks the
registry for all 256 originals of each address it found. The registry's rules stay in C#. The table is exact for every
original byte, so a compare-gated code behaves across banks as it does in C#.

**The debugger, for now.** `MercuryDebugTarget` gained an optional `MercuryDebugHost`, which routes reads, writes,
refreshes, cheats, the frame count and mutes. The shim hands out a C# target over a mirror `MercuryCore`, which is
loaded from MercuryRT's state on every `RefreshProviders`. Memory reads and writes go to MercuryRT directly. The mirror's
registries are the shim's, so watches, labels and the frame log behave as on C#.

~~**Not yet:** breakpoints, stepping, coverage, and the write log that feeds watches. These are stage 6's hooks. The
Engine row's hint says so.~~ *Built in stage 6 (§8.5), and the hint now says MercuryRT stops at breakpoints. Since then
the target is `MercuryRtDebugTarget`, which keeps this mirror for what it shows.*

**Evidence** (`EmuSen.WiseMan/Cores/MercuryRtCoreTests.cs`, 6 cases, all through `ICore` or `IDebugTarget` alone):

- **Cheats.** A compare-gated ROM patch and a RAM poke ran for 300 frames, then 60 more with the master switch off.
  State, picture and sound were identical every frame.
- **Battery.** A random `.srm` was loaded by both engines, and after 300 frames both wrote the same bytes. C#'s save was
  moved aside before MercuryRT's frame, so the comparison cannot pass on one engine's file.
- **States.** States saved on either engine were byte-identical, and each loaded on the other. After the crossing, 120
  frames were identical. Both refusals are C#'s.
- **Debugger.** The target's CPU registers and frame count equal the C# target's, and a WRAM write through it reaches
  MercuryRT.
- **Games.** The four games ran 1,500 frames through `CoreFactory` with input, identical.

The blast radius was the Mercury, cheat, factory, catalog, engine and graphics-settings filters: 568 tests. Two failed.
They asserted that only the N64 has an Engine row, and were updated to expect the Game Boy's.

### 8.4 Mutants for stages 2–5 (done 2026-09-23)

**Method.** There were 23 hand-made mutants. Each was applied, then the crate's tests and WiseMan's `MercuryRt` filter
ran with the four games, and the file was restored. The first round ran **without the corpus**: the runner passed
`EMUSEN_MERCURYRT_ROMS` and not `EMUSEN_MERCURYRT_CORPUS`, so the corpus case passed without running. That is one
reason survivors were many. The survivors were re-run with the corpus and with one new targeted test each.

| Mutant | First round | After the new test and the corpus |
|---|---|---|
| M1 ADD's half-carry ignores the carry-in | random programs | — |
| M2 EI takes effect at once | random programs | — |
| M3 the HALT bug never arms | survived | halt-bug test; corpus |
| M4 a taken CALL costs 20 | random programs | — |
| M5 TIMA reloads after 3 cycles | interrupt programs | — |
| M6 the timer counts rising edges | interrupt programs | — |
| M7 an OAM DMA byte lands a cycle early | survived | **survived** (argued below) |
| M8 VRAM open while drawing | random programs | — |
| M9 an HDMA block charges 16 cycles | HDMA program, random programs, games | — |
| M10 STAT on level, not edge | interrupt programs | — |
| M11 mode 3 ignores SCX's fine scroll | busy programs | — |
| M12 the window counts only when rendering | survived | window test with rendering skipped, instruction by instruction |
| M13 the DMG sprite sort unstable on equal X | games only | — |
| M14 the CGB priority bit ignored | survived | priority test |
| M15 the sweep's second check dropped | survived | sweep test; corpus |
| M16 the envelope on sequencer step 6 | games only | — |
| M17 the high-pass charges after the output | busy programs | — |
| M18 MBC1's bank 0 not remapped | survived | bank-zero test; corpus |
| M19 MBC3 latches on any 1 | survived | latch test |
| M20 KEY1's switch keeps the base-clock phase | pattern ambiguous, not run | **survived** (equivalent, below) |
| M21 the ROM patch table ignored | cheat test | — |
| M22 an absent CARTRAM space reads 0 | survived | space-by-space test |
| M23 the save path always null | battery test | — |

**Result.** 21 of 23 are caught. Seven were caught only after a test was written for them. Two (M13 and M16) are caught
by the real games alone.

**The two left, argued rather than tested:**

- **M20 is equivalent.** The base-clock phase flips once per T-cycle in double speed, and the bus always ticks whole
  machine cycles, which are an even number of T-cycles. So the phase is false at every instruction boundary. In single
  speed it is never touched, so STOP's reset of it changes nothing.
- **M7 is equivalent at every machine-cycle boundary.** Both placements have landed the same number of bytes by the end
  of each 4-cycle group. The one reader that could tell them apart is the renderer's OAM read at `drawing_end`, a dot
  inside a machine cycle, during an OAM DMA, with sprites enabled. No test aligns a DMA with a line's render, so this
  case is untested, not proven equivalent.

**Divergences kept, and written down:**

- A truncated state is refused whole by MercuryRT. C# stops part-way through reading it, leaving a half-loaded machine,
  and throws `EndOfStreamException`.
- MercuryRT's `GetFrameBufferRgba` is a copy.

### 8.5 Stage 6: the debugger's hooks (done 2026-09-24)

**What was missing.** After stage 5, `bp`, `step` and `watch` armed on MercuryRT and nothing happened. The shim ran every
frame through the plain loop, and the debug target's mirror (§8.3) was a copy that never ran. §2.1 listed the four seams
and the mechanism for each. `Mars_Native.md` §6.5 had built that mechanism for MarsRT: tables pushed down, a frame that
ends early with a reason, logs drained, and the registry deciding.

**The prediction, stated before the code.** The plain frame would be unchanged: after/before p50 within ±2% on each of
§1.1's four games. The seams were to be compiled into an observed step only, and the one layout change was to be one
pointer added to `Machine` after the bus.

#### 8.5.1 A finding first: the C# core's own debugger lacked five seams

The claims were written as one abstract set, `MercuryDebugClaims` over a `MercuryDebugRig`, as `MarsDebugClaims` was.
They were run first against the unmodified C# core. **13 of 26 failed there.** Each failure is a registry that
`MercuryDebugTarget` hands out and the core never feeds:

- no call stack, so `step over`, `step out`, `bt`, `bp depth`, `profile` and `cov funcs` had nothing;
- data breakpoints never fired, because `OnWrite` fed the watches alone;
- `runto irq` and `runto frame` never fired;
- a watch went deaf after `LoadRom`, which builds a new bus that the target had not attached to.

The machine was not at fault in any of them, and none is D1–D3. The five seams were added to C# Mercury in their own
commit (50f86a6), each held by a claim, before MercuryRT was given the same claims. `Mercury_Debug.md` §7 records what was
added and why an interrupt is a frame on this core when it is not on Mars. With them all 26 claims pass on C#.

The stage asked for stepping on MercuryRT, `step over` included, under the C# target's claims, and the C# target had no
`step over`. The alternative was to port the refusal. That would have left `step over` refused on both engines, and it
was not chosen. The seams are C#'s debugger, not its machine: no state, picture or sound changed, and §8.2's oracle ran
unchanged over them (§8.5.4).

#### 8.5.2 What the machine does

`src/debug.rs` holds the tables and the logs (`Hooks`, boxed, in `Machine.hooks: Skip<…>`). None of it is in the state,
so a load keeps it, as MarsRT's does.

**The seams are on the bus type, not on a flag.** The SM83's step is generic over `CpuBus`. That trait gained two
methods, `note_call` and `note_return`, whose default bodies are empty. `ObservedBus` wraps the machine's bus and the
hooks, forwards every access, and implements those two methods and `write`'s report. The plain step is still
monomorphised on `MemoryBus` itself, so it carries no load, test or branch for any of this. MarsRT reached the same
place with a `HOOKED` const parameter on `step_in` (`Mars_Native.md` §6.5.1). Mercury's step was already generic over its
bus, so the bus was the cheaper place.

**The observed frame** (`Machine::run_frame_debug`) is `RunFrame`'s loop. Before each step it asks `stop_before(pc)`,
which returns a bit-set of reasons:

- the address is in a breakpoint range;
- stop before every step;
- the depth is at or under the target, or over the guard;
- a store landed in a data-breakpoint range;
- an interrupt was dispatched;
- a log is full.

If any reason holds, the frame returns with the machine at a step boundary. Otherwise it records the coverage bit and the
profiler's count, as C#'s loop does before `Cpu.Step`, and steps. Two flags carry C#'s two kinds of resume:

- `RUN_UNCHECKED` runs the first step unasked, as a halt's instruction runs;
- `RUN_CONTINUING` keeps the budget the frame began with, as C#'s loop keeps it past a `ShouldBreak` that said no.

The second matters in one case. A frame that switches to double speed ends on the 70,224-cycle budget it began with,
before the slowed PPU completes. The crate's `a_frame_continued_past_a_refused_stop_keeps_the_budget_it_began_with`
holds it, and fails on mutant M9.

**Stores.** `ObservedBus::write` asks `MemoryBus::reported_space` where C#'s `Write` would report the store, before the
store lands. VRAM and OAM report only when the CPU can reach them. Cart RAM reports whatever the board does with the
byte, as C# does. The value logged is the byte written, not the byte read back. Mars rereads, but C#'s Mercury passes
`data` to its observer.

The storing instruction's address is written after the step (`Hooks::stamp`). The SM83 sets `LastInstructionPC` before
an instruction's first bus access, and a dispatch leaves it alone. So after a step it is the value C#'s context factory
read at every store in that step.

A host write to `CPUBUS` is reported too, whether from a GameShark poke at a frame's end or the console's `write`,
because C#'s `WriteSpace` goes through `Bus.Write` and its observer hears it. MarsRT has no such path.

**Calls.** `CALL` when taken and `RST` note (source, target) after the push. The dispatch notes (return address,
vector) as an interrupt. `RET`, a taken `RET cc` and `RETI` note a return.

The stack here is the registry's own. The shim pushes the registry's frames' targets down with the other tables
(`mercury_debug_set_stack`), and the frame moves them. MarsRT keeps an independent stack in Rust (§6.5.1 of that page),
which a `bt reset` or the registry's cap can leave disagreeing with the registry. Here they cannot disagree past one
push of the tables, and `The_registrys_stack_is_the_depth_the_machine_stops_at` holds it.

The profile's runs are kept by owner in a `BTreeMap`, drained sorted. §8.5.5 explains why it is not a `HashMap`.

**Coverage** is one bit for each of the 64K addresses, 8 KB. That is the first 8 KB of the registry's 24-bit bitmap,
which `CoverageRegistry.Merge` ORs in. MarsRT's bitmap is 2 MB.

#### 8.5.3 What the shim does

`MercuryRtCore.Debug.cs` follows `MarsRtCore.Debug.cs` with four differences.

- **The frame is observed** when `!Breakpoints.IsQuiet || Coverage.IsArmed || CallStack.IsProfiling || Listening`.
  `Listening` also needs a debug target to have been made. C#'s bus reports stores only once a `MercuryDebugTarget` has
  attached, and `A_store_is_reported_only_while_a_target_listens` holds both engines to that.
- **No first check in C#.** MarsRT's `RunObserved` asks `ShouldBreak(pc)` in C# before the first step. Here Rust asks.
  The tables are a superset of every case in which `ShouldBreak` returns true or changes anything: a breakpoint range
  covers every logpoint, condition and hit count; *stop before each* is pushed while a step counts down or a break is
  owed; and the depth covers the depth checks. So skipping the question where no table holds skips only calls that
  would have returned false and changed nothing. The difference shows only when a frame begins past its budget, where
  C#'s loop never asks. MarsRT's pre-check would ask.
- **Host writes** to `CPUBUS` push the tables, write, and drain while the core listens (§8.5.2).
- **The target is its own class.** `MercuryRtDebugTarget` wraps the stage-5 `MercuryDebugTarget` over the mirror for
  everything it shows. It hands out the core's registries and a `DebugCpu` whose program counter is MercuryRT's, read
  live (`mercury_machine_pc`). Stage 5's target read the mirror's counter, which is stale between refreshes. Its summary
  names `(MercuryRT)`.

At each stop the logs are drained in the order the C# observers would have run: each store to `Watches.RecordWrite`
(context `PC=$xxxx`) and to `Breakpoints.NoteWrite`, then each push and pop, then the profile's runs, then the coverage
bitmap. An interrupt stop calls `NoteInterrupt(Irq)`. Then the registry is asked, and its answer is the halt. Rust's
tables are candidates, and nothing of the registry's semantics is in Rust. At a frame's end the shim calls
`Breakpoints.NoteFrame` after the cheats, as C#'s `EndFrame` now does.

The interface is 2: `mercury_debug_set`, `_set_stack`, `_set_breakpoints`, `_set_ranges`, `_run_frame`, `_writes`,
`_calls`, `_profile`, `_coverage`, `_depth`, and `mercury_machine_pc`.

#### 8.5.4 The evidence, and the mutants

**The claims.** `MercuryDebugClaims` (`EmuSen.WiseMan/Cores/MercuryDebugTests.cs`) is 26 cases, run as
`MercuryDebugTests` on C# and as `MercuryRtDebugTests` on MercuryRT, with the same expected addresses, counts and
contexts. They cover:

- a breakpoint and its resume, `step n` from a halt and between frames, a logpoint's 5,800-odd passes a frame, and a
  disabled breakpoint and a forbidden range;
- watches: every store with its instruction as context, a call's push into HRAM, a cheat's poke and a console `write`,
  and a reload;
- coverage armed, disarmed and cleared, including a disarmed frame that a breakpoint makes observed;
- a halt that leaves coverage and the profile current;
- a data breakpoint halting after its store, and one on a value never written, refused at every store;
- `step over`, `step out` and `bt` through a call and through an interrupt to its `RETI`, and the depth guard;
- `runto irq` and `runto frame`, the profiler's counts, and `cov funcs`;
- an illegal opcode in an observed frame, with C#'s message;
- halts at the vblank vector on every third frame, against the program run plain;
- the frontend bundle's wiring;
- the console's own `bp`, `step`, `step out`, `step over` and `bt`.

Before the stage the set had 25 of these claims. Twelve of them failed on C#, and so did the C#-only case, which is
§8.5.1's 13 of 26. On MercuryRT 22 of the 25 failed. Three passed:

- the disabled breakpoint and the forbidden range, where nothing was to halt;
- the data breakpoint on a value never written, where nothing was to halt either;
- the illegal opcode, which the plain frame throws.

One claim differs by engine, and each side is pinned by a named test (`Mercury_Debug.md` §7). The C# core tracks the
stack in every frame. MercuryRT tracks it only in an observed one, as MarsRT does.

MercuryRT has seven cases of its own:

- a call made in an observed frame is on the stack;
- a plain frame leaves the stack where the last observed frame left it;
- the registry's stack is the depth the machine stops at, after a `bt reset`;
- stores are reported only once a target listens, compared with C#'s event for event;
- the three proofs below: games and synthetic programs armed, and games halted.

`MercuryDebugTargetTests` became an abstract set run on both engines, and its 34 cases pass on each.

**Exactness with every table armed and nothing halting** (`A_game_with_every_table_armed_is_the_game_run_plain`, and
the synthetic twin). Both engines were armed alike:

- a breakpoint nothing reaches;
- a logpoint at the vblank vector;
- a breakpoint over `$0048`–`$005F` inside a forbidden range, so the registry refuses every stop;
- data breakpoints over WRAM's first page and all of HRAM, on the value 256, which no byte has;
- a depth guard of 512;
- coverage and the profiler;
- watches over 4 KB of WRAM, HRAM, OAM, all of VRAM and cart RAM.

The runs were compared against C# run plain, after every frame, in state, picture and sound. At the end the registries'
transcripts were compared between the engines: every hit count, logpoint total, coverage map, profile by routine,
backtrace, unmatched return, entry point, watch's total, site histogram and last 500 events. All identical:

| Run | Frames | Steps recorded and charged | Stores watched | Logpoint passes | Entry points | Depth at the end |
|---|---|---|---|---|---|---|
| Tetris | 600 | 4,492,667 | 118,488 | 579 | 92 | 510 |
| Link's Awakening | 600 | 7,587,570 | 141,152 | 515 | 105 | 511 |
| Kirby's Dream Land | 600 | 4,208,689 | 277,637 | 535 | 122 | 3 |
| Pokémon Yellow | 600 | 8,605,943 | 107,335 | 531 | 144 | 5, 8 unmatched returns |
| interrupts, count, call, VRAM and OAM programs | 300 each | 1.76–5.26 million | 2–877,799 | 0–300 | 0–2 | 0 |

Tetris's and Link's Awakening's stacks climb to the cap of 512 on both engines. Those games leave routines by roads
other than `RET`, as Mars's games did (`Mars_Native.md` §6.5.3). The VRAM and OAM programs store through their whole
space whether the PPU is drawing or not. They exist so that a store the bus refuses is not reported (mutants M7 and M8).

**Exactness halted and resumed** (`A_game_halted_at_breakpoints_and_resumed_is_the_game_run_through`). A breakpoint at
`$0040` was armed on every third frame of 900 on both engines. At each halt the C# and Rust states were compared, the
two were resumed, and each frame's end was compared against C# run plain, with picture and sound. The halts were:

- the interrupts program, 300;
- Tetris, 290;
- Kirby's Dream Land, 277;
- Pokémon Yellow, 260;
- Link's Awakening, 258.

All were identical. The crate's `tests/debug.rs` (15 cases) proves the same properties on the machine alone. Its cases
include a frame continued past a refused stop after a speed switch, a full log drained with nothing lost, and games
armed against plain from `EMUSEN_MERCURYRT_ROMS`: four games, 600 frames, identical.

**The blast radius.** The Mercury, CoreCatalog, CoreFactory, Engine and GraphicsSettings filters were run with the four
games and the corpus at the stage's last commit: 397 tests, all passing. That includes `MercuryRtMachineTests`, since the SM83's opcodes gained the
`note_*` calls. The crate has 35 tests: 20 unit tests, 9 of them the hooks', and the 15 in `tests/debug.rs`.

**Mutants: 19, all caught.** Each was applied, then the crate's tests and WiseMan's two debug classes ran with the
games, and the file was restored (scratch `mutants.py`).

| Mutant | Caught by |
|---|---|
| M1 the tables asked after the step, of the address that ran | crate (6); WiseMan (21) |
| M2 a return does not pop the machine's stack | crate (4); WiseMan (7), the armed games among them |
| M3 the coverage bitmap kept, and recording, once disarmed | crate `coverage_records…`; WiseMan `Coverage_disarmed_records_nothing_even_in_a_frame_that_is_observed` |
| M4 a store's instruction never stamped | crate; WiseMan's watch contexts and the armed games |
| M5 a host write to `CPUBUS` not reported | crate; `A_watch_hears_a_cheat_poke_and_a_debugger_write_to_the_bus` |
| M6 an interrupt dispatch not noted | crate (2); WiseMan, the armed games among them |
| M7 a VRAM store the bus refuses reported anyway | the armed games and the VRAM program only |
| M8 an OAM store the bus refuses reported anyway | the OAM program only |
| M9 a continued frame begins its budget again | crate `a_frame_continued_past_a_refused_stop_keeps_the_budget_it_began_with` only |
| M10 the depth guard one frame late | crate; `The_depth_guard_halts_a_call_past_it` |
| M11 the profile's run charged after the push | crate; the halt, profiler and armed-game cases |
| M12 stop-before-each only where a breakpoint is | crate; the step cases |
| M13 a data breakpoint's store never stops | crate; `A_data_breakpoint_halts_after_the_store_that_wrote_it` |
| M14 an interrupt dispatch never stops | crate; `Run_to_interrupt_stops_at_the_vector` |
| C1 the shim drains the logs only at a frame's end | the halt, data-breakpoint and armed-game cases |
| C2 no stop before each instruction for a break owed | `Run_to_frame_stops_at_the_start_of_the_frame_after_it` only |
| C3 the registry's stack not pushed down | `The_registrys_stack_is_the_depth_the_machine_stops_at` only |
| C4 stores reported with no target listening | `A_store_is_reported_only_while_a_target_listens` only |
| C5 a resumed frame checks the instruction it halted at | the step, step-over and console cases |

MarsRT's coverage mutant survived at the claims' level (`Mars_Native.md` §6.5.4). M3 does not survive here, because a
claim was written for the one path that can see it: a frame that is observed for another reason while coverage is
disarmed.

**Two mutants that no claim can catch, by design.** The tables are candidates, and the registry refuses any stop it
would not have made. So a table that is too *wide* costs a crossing and changes nothing a claim can see. Two were run
to show it:

- W1, the depth guard one frame early (`>=`);
- W2, a breakpoint range one address wider at its start.

Both survived every oracle. The crate's unit tests now pin both edges exactly, and both mutants fail there. A table that
is too *narrow* misses a stop, and every M-mutant above is of that kind.

#### 8.5.5 Speed

**The first measurement refuted the prediction.** The `frames` example was built before the stage (0301306) and after
it. The two were interleaved under the bench lock, the order swapped each round, 3,000 frames each, with identical state
hashes. Plain-frame p50 in ms:

| Game | before, 3 rounds | after, 3 rounds | after/before |
|---|---|---|---|
| Tetris | 0.528, 0.527, 0.519 | 0.496, 0.495, 0.500 | 0.95 |
| Link's Awakening | 0.462, 0.463, 0.464 | 0.487, 0.486, 0.489 | 1.05 |
| Kirby's Dream Land | 0.559, 0.557, 0.568 | 0.550, 0.548, 0.547 | 0.98 |
| Pokémon Yellow | 0.604, 0.605, 0.601 | 0.596, 0.597, 0.598 | 0.99 |

Two games moved 5%, in opposite directions, steadily across rounds. That is outside ±2%, and the direction is not a
cost.

**What moved.** The two binaries' symbol tables were compared. Every function of the plain path has the same size,
including `run_frame`, `execute`, `call`, `push` and `pop`, with two exceptions:

- `Machine::load_rom`, which allocates the hooks;
- `MemoryBus::read`, from 1,068 bytes to 837.

`read` was not edited. It holds the Game Genie table's `HashMap` lookup, and the profile's `HashMap<u32, i64>` added a
second instantiation over the same hasher. That changed what LLVM inlined into `read`.

A third build added only the boxed field to the pre-stage crate. It moved Tetris to 0.502 ms and Link's Awakening to
0.468 ms: part of the shift, in the same directions.

**The profile moved to a `BTreeMap`.** After that, the symbol tables are identical except for `load_rom`. The timing was
repeated with the three builds rotated:

| Game | before | layout only | after | after/before |
|---|---|---|---|---|
| Tetris | 0.516, 0.520, 0.542 | 0.501, 0.505, 0.501 | 0.527, 0.525, 0.522 | 1.00–1.02 |
| Link's Awakening | 0.467, 0.461, 0.463 | 0.468, 0.472, 0.469 | 0.469, 0.468, 0.470 | 1.01–1.02 |
| Kirby's Dream Land | 0.566, 0.559, 0.562 | 0.548, 0.547, 0.547 | 0.566, 0.563, 0.563 | 1.00 |
| Pokémon Yellow | 0.603, 0.602, 0.605 | 0.603, 0.603, 0.599 | 0.615, 0.611, 0.611 | 1.01–1.02 |

**The prediction holds on the second build.** Every game is within ±2%, and the after/before ratio sits between 1.00
and 1.02 on all four. That consistent direction is inside the band and was not investigated further.

The layout-only build is 3% faster than both on Tetris and Kirby. Its boxed field is a 320-byte array rather than
`Hooks`, which is the one other difference. So the struct's layout moves these games by a few per cent either way, as
it moved MarsRT's (`Mars_Native.md` §6.5.5). That is a property of the machine's layout, not of the hooks.

**An observed frame's cost** was not measured alone. The armed game proof, three engines with a state saved each frame,
takes 21 s for 2,400 frames.

#### 8.5.6 What is not done

- **Reads are not reported** on either engine. `watch r`, `bp read` and `bp uninit` arm and never fire. MercuryRT
  logs, and so marks for the uninitialised-read check, only the stores a watch or data breakpoint covers. C# marks every
  store, but nothing reads the marks.
- **The stack in a plain frame** is not tracked, as on MarsRT (§8.5.4).
- **Transfers.** OAM DMA and HDMA are not reported as stores on either engine, and host writes to spaces other than
  `CPUBUS` are not reported either, as in C#.
- **`runto scanline`** is not fed, and **conditions** have no expression context, on either engine
  (`Mercury_Debug.md` §6).
- **The dashboard's audio peek** on MercuryRT's target is empty. The mirror's APU queue is not in the state (§3.1). This
  is stage 5's gap, and MarsRT closed its own in `Mars_Native.md` §6.6.2.
- **The C# core's plain frame** was not timed with its new seams (`Mercury_Debug.md` §7).
- **Speed of an observed frame**, above.
- **The settings reference** has no Game Boy engine entry to update. `EmuSen_Settings_Reference.md` §4.44 covers the
  N64 alone. The Engine row's hint in `CoreCatalog` now says MercuryRT stops at breakpoints.

### 8.6 Stage 7: platforms and CI (2026-09-24)

*Numbering.* Stage 6, the debugger's hooks, was done at the same time on its own branch. §8.5 is left for its record.

**What the stage started from.** Stage 1 added MercuryRT to the build beside MarsRT, as a second copy of MarsRT's
properties and target. The publish side and the workflow still knew only MarsRT. `MarsRtPublish.targets` removed the
host's library from a foreign publish by MarsRT's three file names, and `marsrt.yml` built one crate.

**Predictions, stated before measuring.**

- **S1.** An unmodified win-x64 publish made on this machine carries the Linux `libmercuryrt.so`, with no warning.
  The reason is that the publish targets remove only MarsRT's names. MercuryRT would then be missing on Windows, and
  nothing would say so.
- **S2.** After the change, the three local build cases of `Mars_Native.md` §1 behave the same with either crate:
  a normal build succeeds, a broken crate fails the build, and a build without cargo warns and succeeds.
- **S3.** The Game Boy's picture and state are identical on linux-x64, win-x64 and osx-arm64 in both engines, because
  the machine is integer arithmetic. The sound is identical too (§5's risk, argued in §3.3).
- **S4.** The part of S3 most likely to fail is `Math.Pow`, which each platform's C library answers. If it differs,
  the two engines still agree with each other on every platform, because the shim evaluates the power in C# and
  passes it down (§3.3). The platform then differs from Linux in both engines at once.

**The design.** The machinery is one list, not a second copy:

- **`EmuSen/Cores/RustCores.props`** has one row per crate: the library's name, the crate's folder, the core's name,
  its console, and the C# core the console falls back to.
- **`EmuSen.csproj`** imports it. `FindCargo` probes once. `BuildRustCores` runs `cargo build` once per row, batched
  over the list, into `$(EmuSenCargoTargetRoot)/<library>`. That root is `EmuSen/obj` by default, so the libraries
  stay in `obj/marsrt` and `obj/mercuryrt`, which scratch scripts already copy. It is a property so that a scratch
  build can move cargo's output off the tree; this stage's builds went to `~/.cache/emusen/probe/mercuryrt-platforms/`.
  The host library names are formed from a prefix and a suffix, so a new crate adds no file-name properties.
- **`EmuSen/Cores/RustCoresPublish.targets`** replaces `MarsRtPublish.targets` and is imported by Mistress and
  Hotaru. On a foreign runtime identifier it removes every row's three host names from the publish. For each row, it
  then either takes `$(EmuSenNativePrebuilt)/<rid>/<library>` or warns with that row's console and fallback. The
  foreign test compares against the portable identifier as well as the SDK's (`fedora.44-x64` here), as before.
  *Amended 2026-10-15:* the host's sidecars, `<library>.core.json`, are dropped with the host's libraries, and a
  library on the core ABI v1 takes its own sidecar from beside it in the prebuilt folder, where the workflow's
  artefact puts it. Measured before the change, on a win-x64 publish with every prebuilt library present: the tree
  held the five `.dll` libraries of the v1 cores and, beside them, the five Linux sidecars `lib<name>.so.core.json`,
  naming libraries the tree did not hold, and no sidecar of its own. Argued from `CoreDiscovery`: a sidecar is read
  without its library being looked for, and opening the core then refuses it as missing beside its sidecar, so such a
  build would refuse VenusRT, MoonRT, MercuryRT, Nephrite and Diamond, leaving the Genesis and the PlayStation with
  no engine. The same publish after the change holds the five `<name>.dll.core.json` and no Linux sidecar.
- **`.github/workflows/rust-cores.yml`**, which was `marsrt.yml`, is a crate × platform matrix: two crates on the same
  four runners, which gives eight jobs and eight artefacts named `<crate>-<rid>`. A second job, `wiseman`, runs on
  linux-x64, win-x64 and osx-arm64:
  - it checks out the repository and `RedQuE3n/EmuSen.LunaP` (`openemu-library`) side by side, as `LunaP.props`
    expects;
  - it builds WiseMan with `EmuSenNative=false`, so that cargo stays out of the build;
  - it copies the two shipped libraries beside the test assembly;
  - it runs the Rust-against-C# tests of both cores (`MarsRt`, `MarsRT`, `MarsNative`, `MercuryRt`), which are 770
    tests here.

The list has one limitation: the workflow's matrix repeats it, because YAML cannot read an MSBuild file. A new crate
is therefore a row in two files, and the workflow's comment says so.

**The platform digests** (`EmuSen.WiseMan/Cores/MercuryRtPlatformTests.cs`). The existing tests compare MercuryRT
with C# Mercury on one machine. S3 is a claim across machines, so it needs a fixed reference. The reference is
digests recorded here on linux-x64 and committed. The file has three tests:

- **Picture, sound and state.** Four synthetic programs run 600 frames on both engines:
  - "four channels" keys all four channels, panned apart, and fills tile data while the LCD is on;
  - "busy" (§8.1's program) runs on an MBC5 colour cartridge;
  - "interrupts" runs on MBC3 with a wave note;
  - "colour" uses HDMA and double speed.

  The SHA-256 of every sample, of every frame's RGBA and of the final state must first be equal between the two
  engines on the platform, and then equal to Linux's.
- **The C# mixer's doubles.** The same programs, except "colour", run on C# alone. After every frame the high-pass
  filter's two capacitors are hashed bit for bit.
- **The two powers.** `Math.Pow(0.999958, 95)` and `Math.Pow(0.999958, 95.108…)` must give Linux's bits,
  `3FEFDF60DC188482` and `3FEFDF574D84E421`.

**How sensitive the digests are (measured with mutants).**

| Mutant | Picture, sound and state (4) | C# mixer's doubles (3) | Powers |
|---|---|---|---|
| The charge factor one ulp higher, in both engines | all pass | not run (written after this mutant) | pass (the test computes its own) |
| The same, in C# alone | all pass; the engines' samples still agree | **all 3 fail** | pass |
| The charge factor 2²⁰ ulps higher, in both engines | "four channels" **fails** on sound; the rest pass | not run | pass |

So the samples alone would not show a platform whose `pow` differs in the last place. A relative difference of
2×10⁻¹⁰ reaches the samples of only one of the four programs within 600 frames. This result says what each test is
for:

- the powers test detects a library difference directly;
- the capacitor digest detects any difference in the C# mixer's arithmetic below what a sample's truncation hides;
- the sample digests detect what reaches the listener.

MercuryRT's own doubles cannot be reached without a new export. Adding one is ABI growth, which is outside this
stage, so on another platform MercuryRT's mixer is held only by its samples: to Linux's, and to C#'s, frame by frame.

**Evidence, on this machine.**

- **S1 holds** on the unmodified tree. The win-x64 publish of Mistress put a 680,648-byte ELF `libmercuryrt.so` in
  `lib/EmuSen/`, and there was no `mercuryrt.dll`. The one warning was MarsRT's.
- **S2 holds.** The list's order puts MarsRT first.

  | Case | Result |
  |---|---|
  | Normal build | Both crates built, both libraries in `EmuSen/bin` and in WiseMan's output, byte-identical to cargo's |
  | MercuryRT with a syntax error | Build failed, cargo exit 101, after MarsRT had built; checked again on the final targets |
  | MarsRT with a syntax error | Build failed, cargo exit 101, and MercuryRT was not attempted |
  | Cargo hidden from `PATH` | Build succeeded with one warning naming both cores, and no library in the output |

  The last row first showed a second warning: the probe's own MSB3073 for the missing command. The probe now ignores
  its exit code instead of continuing on error (`Mars_Native.md` §1).
- **The fallback happens.** A throwaway console program was put beside the no-cargo build's assemblies. It asked
  `CoreFactory` for MercuryRT and got `MercuryCore`, with the notice "MercuryRT (Rust) is not available
  (libmercuryrt.so not found beside the assemblies); Mercury (C#) is running." With the two libraries copied in, it
  got `MercuryRtCore` and no notice. With `EMUSEN_MERCURY_NATIVE=0` set, it got C# again, with that reason. All three
  runs gave the same frame-60 picture hash.
- **Publishes.** All used Mistress, `-c Release --self-contained`. The stand-ins were small files, because only their
  placement was under test.

  | Publish | Warnings | Rust libraries in `lib/EmuSen/` |
  |---|---|---|
  | linux-x64 (the host by its portable name) | none | `libmarsrt.so` 8,334,176 B and `libmercuryrt.so` 680,648 B, both identical to cargo's `dist` output |
  | win-x64, no prebuilt directory | MarsRT's and MercuryRT's | none |
  | win-x64, both stand-ins | none | `marsrt.dll`, `mercuryrt.dll` |
  | win-x64, MarsRT's stand-in only | MercuryRT's | `marsrt.dll` |
  | osx-arm64, both stand-ins | none | `libmarsrt.dylib`, `libmercuryrt.dylib` |

  The same console program, run inside the linux-x64 publish, loaded both published libraries and got
  `MercuryRtCore`. The `dist` profile takes MercuryRT's library from 11.1 MB (release, with line tables) to 0.68 MB.
- **The CI job's Linux instance, reproduced.** The committed tree was extracted with `git archive`. LunaP's
  `openemu-library` was extracted beside it from GitHub's tarball, with the committed `LunaP.props` unchanged. It was
  built as the job builds it, and the two `dist` libraries were copied in by the job's own step script.
  - Before the copy, 4 of the 8 platform tests failed, each with "libmercuryrt.so not found beside the assemblies".
    The other four are the C#-only tests.
  - After the copy, the job's filter passed **770 of 770 in 1 min 27 s**.
  - The crate job's Linux steps also passed: `cargo test --release --target x86_64-unknown-linux-gnu` passed MercuryRT's
    11 tests, then the `dist` build ran. Its `Exports` script, run with the matrix values substituted, counted 24
    `mercury_` exports.
- **Lint.** `actionlint` 1.7.12 with `shellcheck` 0.11.0 found one warning (`ls | grep`), which was fixed, and then
  none.

**What only CI can verify, and has not yet run.** Nothing in this list was run here; the workflow is correct by
reading and by lint alone.

- **S3 and S4 on win-x64 and osx-arm64.** They are answered by the `wiseman` job's first step, which prints each
  platform's digests and powers in full. The outcomes read as follows:
  - all green holds S3;
  - a failed powers test together with failed capacitor digests is S4, a platform library, and the sample digests
    show whether the listener can hear it;
  - an engine-against-engine failure would be a defect in MercuryRT on that platform.
- **The crate on the three other targets.** This covers MercuryRT's 11 crate tests there, the `dist` build for all
  four identifiers, and the exports, which are counted on Linux and macOS but not on Windows.
- **MarsRT's WiseMan tests on Windows and macOS.** `Mars_Native.md` §6.3 left these for later, and this job is their
  first run.
- **The layout on the runners.** This is the two-checkout layout, `setup-dotnet` 10, and the artefact download into one
  folder.

**What is not done.**

- **osx-x64 has no WiseMan run.** Its runner is arm64, and its crate tests run under Rosetta and may fail.
- **The csproj's cargo path is exercised only on Linux.** The `wiseman` job builds with `EmuSenNative=false`, so it
  tests the libraries that ship rather than a Windows or macOS developer's build. `FindCargo` under `cmd` and the
  `lib`-less Windows name in `BuildRustCores` are argued, not run.
- **MercuryRT's crate suite is thin.** It holds stage 1's 11 codec tests; its machine tests live in WiseMan. On a
  foreign platform the crate job proves that it compiles and round-trips state, and WiseMan proves the rest.
- **No foreign publish has been run on its platform,** for MercuryRT as for MarsRT.
- **The games and the corpus stay local.** They need ROMs, and CI has none.
- **LunaP is taken from a branch, not a commit.** A LunaP change that breaks EmuSen's build would show up here as a
  red job, not as an EmuSen change.
- **Criterion 1 of §4 is still not met.** The Engine row's default for GB is still C# Mercury (Q5). This stage makes
  MercuryRT available on every platform; it does not make it the one players get.

### 8.7 On the common native interface (2026-09-30)

MercuryRT is the second core moved onto the interface of `EmuSen_NativeCores.md` §3. This is that page's step 4
(§7 there), after MoonRT's steps 2 and 3. The machine, the state and `ObservedBus` did not change. What changed is how
the library is reached, and which crate the debugger's hooks come from.

#### 8.7.1 What moved

**The exports.** The 36 `mercury_*` exports are gone. The library now exports 37 `emusen_native_*` functions,
generated by `native_exports!(Machine; mutes, rom_patches, debug, debug_stack)` from `impl NativeCore for Machine` in
`src/ffi/mod.rs`:

- the 24 that every core has;
- `set_mutes` and `set_rom_patches`;
- the ten `debug_*` exports and `debug_set_stack`.

The interface version is `0x0001_0005`: the common half 1 and MercuryRT's half 5. Versions 1 to 4 were
`mercury_interface_version`'s. The capability word is `MUTES | ROM_PATCHES | DEBUG | DEBUG_STACK`, and the macro
checks at compile time that it matches the groups generated.

**What only the Game Boy has** is split two ways:

- **A create argument.** `Model` is the console, as `GbModel`'s number: 0 Auto, 1 Game Boy, 2 Game Boy Color. Any
  other key is refused with the interface's unknown-setting status. The battery save is file 0, written into
  `CARTRAM` and clipped to its size, as `MercuryCore.LoadRom` reads it. The CGB flag is not an argument. Whether the
  machine is a Game Boy Color follows from the model and the header, and a state from the other console changes it
  (`Mercury_Model.md` §5).
- **Six named extensions** (`mercuryrt_*`):
  - `set_sample_rate`, whose two doubles C# computes (§3.3);
  - `serial`, the link port's log, which the test ROMs report through;
  - `cgb_hardware`;
  - `step`, one instruction, for the step-by-step tests;
  - `rom_patch`, the exhaustive patch test's view of the table;
  - `ram_length`, the header's RAM size, battery or not.

**The status codes** use the core's own band:

| Status | Meaning |
|---|---|
| −9 | an image shorter than the 336-byte header |
| −10 | a cartridge type no board implements |
| −11 | a model that is not Auto, Game Boy or Game Boy Color |
| −20 | an opcode no SM83 has; the frame's detail word carries `(opcode << 16) | pc` |

The C# shim turns −10 into C#'s `NotSupportedException`, with the cartridge type. It turns −9 and −11 into
`InvalidDataException`. It rebuilds −20 into the C# CPU's own message from the detail word, through `NativeMachine`'s
new `FrameException` seam.

**The debugger** is on `emusen-native`'s `Hooks` (`EmuSen_NativeCores.md` §12.4). Its `debug.rs` keeps only
`ObservedBus` and `new_hooks()`, which is `Hooks::new(&[16])`. It adopts the module with the four changes §12.4 lists:

- the stack is (source, target) pairs;
- the flags are renumbered, with coverage at bit 8 (was 16) and `PROFILING` at 16 (was 32);
- `note_call` takes a kind: 1 for `CALL`/`RST`, 2 for an interrupt;
- `mercury_debug_*` became `emusen_native_debug_*`, and `mercury_machine_pc` became `debug_pc(0)`.

**The C# shim.** `MercuryRtCore` is now a `NativeRtCore<MercuryMachine>`, and `MercuryMachine` a `NativeMachine`. The
debugger's seam is a `NativeDebugBridge`, as MoonRT's is. Four files were deleted:

- `MercuryRtCore.Debug.cs`, whose work the bridge does;
- `LegacyNativeRtCore.cs`, the 8-bit base class only MercuryRT still used;
- `RomPatchTable.cs`, whose byte table the Rust side now builds from triples;
- the crate's `src/ffi/debug.rs`.

`LegacyNativeMachine` stays for MarsRT alone. The shim's three files come to 274 lines, down from four files of 456.
The host lost the 280 lines of the two deleted classes. The GB Engine row still defaults to Mercury (C#).

#### 8.7.2 The oracles

| Oracle | Result |
|---|---|
| The crate's 28 tests, 4 of them §8.7.4's | pass |
| WiseMan's Mercury, native-host, battery, engine and state-header tests, 591 in Release with §8.7.4's additions; 842 with MoonRT's and MarsRT's frontend tests | pass |
| 7 games × 3 models, from boot for 900 frames and from a state 300 more; 3 library resume states for 600: a digest chained over the picture, the sound and every 50th frame's state, against MercuryRT at 82e47ac | 24 of 24 identical |
| States cross engines both ways; the battery save loads and writes the same bytes on both; a ROM patch and a RAM poke, Game Genie's and GameShark's kinds, act identically (`MercuryRtCoreTests`) | pass |
| Battery saves land in `Saves/GB` and `Saves/GBC` on both engines, and a GB game run as a colour console keeps its save in `GB` (`BatterySaveTests`) | pass |
| The debugger's contract on both engines (`MercuryDebugTargetTests`); synthetic programs and a game with every table armed equal their plain runs (`MercuryRtDebugTests`) | pass |
| `EMUSEN_MERCURY_NATIVE=0` in a child process gives Mercury (C#) with the notice (`MercuryRtFallbackTests`, new); a library without the common exports is refused (`NativeHostTests`, new) | pass |

The digest tool drives the machine through `ICore`, as a player does. It compares the old build's
`mercury_machine_run_frame` against the new build's `emusen_native_advance`.

#### 8.7.3 Speed: placement, not work

**The first measurement refuted "no slower".** The plain frame measured 3–7% slower on the three DMG games and level on
Link's Awakening DX. A symbol diff showed that `MemoryBus::read` had shrunk from 1,069 bytes to 651. This was §8.5.5's
finding again, from a different source. The new `mercuryrt_rom_patch` extension was a second call site of the patch
table's `HashMap::get`, and with two callers LLVM stopped inlining `get` into `read`. The extension now scans the
map, which is a test's view and cold. After that, `read` is 1,069 bytes again. Every function of the plain path then
has the size it had before the move:

- `run_frame`;
- `execute` and `execute_cb` over `MemoryBus`;
- `read`, `write`, `read_io` and `write_io`.

The only functions whose sizes differ are cold: `load_state`, the patch table's builder, `run_frame_debug`,
`write_space` (one byte), a drop glue, and `call` over `ObservedBus`.

**The frame was still 2.5–7.5% slower on DMG games.** It stayed that way with identical sizes. The Rust-only `frames`
example showed no difference, and the C# loop over the bare machine showed the same slowdown. What differed was where
the linker put the hot functions. `run_frame` moved from offset 0 to 16 within its 64-byte line, `execute` from 32 to
0, and `read` from 48 to 0.

To separate placement from work, both trees were rebuilt with every function aligned to 64 bytes and every block
that is only jumped to aligned to 32
(`-Cllvm-args=-align-all-functions=6 -Cllvm-args=-align-all-nofallthru-blocks=5`). Built that way, the moved core was
level with or faster than the old one on all four games. Functions alone aligned brought the two within about 1%.
Measured 2026-09-30, `raw` loop, ms/frame, 3 interleaved rounds under the timing lock:

| Game | old | new | old, aligned | new, aligned |
|---|---|---|---|---|
| Tetris | 0.4586 / 0.4490 / 0.4427 | 0.4774 / 0.4719 / 0.4717 | 0.4473 / 0.4444 / 0.4449 | 0.4428 / 0.4419 / 0.4438 |
| Kirby's Dream Land | 0.5201 / 0.5143 / 0.5202 | 0.5381 / 0.5327 / 0.5363 | 0.5197 / 0.5185 / 0.5243 | 0.5116 / 0.5115 / 0.5089 |
| Link's Awakening | 0.4560 / 0.4564 / 0.4582 | 0.4779 / 0.4759 / 0.4756 | 0.4483 / 0.4456 / 0.4468 | 0.4426 / 0.4393 / 0.4430 |
| Link's Awakening DX (CGB) | 0.7127 / 0.7288 / 0.7142 | 0.7190 / 0.7146 / 0.7158 | 0.7127 / 0.7080 / 0.7131 | 0.7149 / 0.7089 / 0.7130 |

**So the crate is built aligned.** The two flags are in the crate's own `.cargo/config.toml`. Cargo reads that file
from the directory it runs in, so `EmuSen.csproj` now runs cargo from each crate's directory, as CI always has. The
file's `build.rustflags` is merged with the PGO flags the build passes with `--config`, and with an empty
`CARGO_BUILD_RUSTFLAGS`, as CI sets for a crate without a profile. `RUSTFLAGS` would still replace it. MoonRT's and
MarsRT's libraries come out byte for byte as before. The cost is size: `execute` grows from 87 KB to 94 KB, and the
library by about 50 KB.

**The result**, through `ICore` as a player runs it, measured 2026-09-30. The table gives the best of 3 interleaved
rounds under the timing lock, the order swapped each round, 3,000 frames, with one state hash per game across all
builds:

| Game | old plain | new plain | new/old | old armed | new armed | new/old | new armed/plain |
|---|---|---|---|---|---|---|---|
| Tetris | 0.4417 | 0.4392 | −0.6% | 0.5161 | 0.5065 | −1.9% | 1.15× |
| Kirby's Dream Land | 0.4967 | 0.4871 | −1.9% | 0.5709 | 0.5576 | −2.3% | 1.14× |
| Link's Awakening | 0.4088 | 0.3916 | −4.2% | 0.4875 | 0.4908 | +0.7% | 1.25× |
| Link's Awakening DX (CGB) | 0.6607 | 0.6597 | −0.2% | 0.8015 | 0.8074 | +0.7% | 1.22× |

"Armed" is a frame on the observed path with three tables set:

- a breakpoint over `$FEF0–$FEFF`, which never fires;
- a watch over the whole of WRAM, so every WRAM store is logged;
- coverage.

**Why this does not generalise.** The alignment is a fix for one library, measured on one machine, the Ryzen 7 7700X.
It was not measured on the handheld's Zen 2 or on the M1, and on arm64 its effect is unknown. MarsRT is laid out by
its PGO profile, which already places the hot code. MoonRT was not re-measured with the flags. Neither is changed
here. The finding that does generalise is the negative one. Equal
symbol sizes, which is `EmuSen_NativeCores.md`'s P4, do not show that the speed is unchanged. The linker's placement
moves an 8-bit core by several per cent with no function changed.

#### 8.7.4 Mutants

Nineteen mutants of the moved logic were run one at a time, measured 2026-09-30. Each ran against the crate's tests and
WiseMan's Mercury, native-host, battery, engine and state-header tests, 586 in the first round.

| # | Mutation | First round | With the new tests |
|---|---|---|---|
| R1 | `set_buttons` ignores the changed mask | survived | caught (crate) |
| R2 | `set_buttons` accepts port 1 | survived | caught (crate) |
| R3 | every model parsed as Auto | caught (16) | — |
| R4 | an unknown setting accepted | survived | caught (crate) |
| R5 | the battery file not written | caught (3) | — |
| R6 | the battery given back without a battery | survived | caught (crate) |
| R7 | an unconditional patch overwrites an earlier one | survived | caught (crate, WiseMan) |
| R8 | a compared patch overwrites an earlier one | survived | caught (crate, WiseMan) |
| R9 | a plain frame's detail word loses the opcode | survived | caught (WiseMan) |
| R10 | each mute moved to the neighbouring channel | survived | caught (WiseMan) |
| R11 | skip rendering ignored | survived | caught (WiseMan), at the second attempt |
| R12 | `debug_pc` answers for processor 1 | caught (17) | — |
| R13 | an observed frame's detail word loses the opcode | caught (1) | — |
| R14 | patch addresses past 16 bits wrap | survived | caught (crate) |
| C1 | −10 thrown as `InvalidDataException` | caught (1) | — |
| C2 | `PortFor` sends every port to pad 1 | survived | caught (WiseMan) |
| C3 | the illegal-opcode exception not rebuilt | caught (1) | — |
| C4 | a listened bus store not observed | caught (1) | — |
| C5 | the patch range widened to `$FFFF` | survived | caught (WiseMan) |

**Twelve survived the first round, and all twelve are caught now.** The new tests fall into two groups.

**The crate's `tests/ffi.rs`** calls `NativeCore` directly. R1, R2, R4, R6 and R14 cannot be reached through the host:

- it always sends a whole button mask;
- it sends only port 0;
- it sends only the `Model` key;
- it asks for the battery only when the header has one;
- it filters the patch range before building triples.

These mutants therefore break the interface's contract without changing anything a player sees. The contract is what a
second host, such as a libretro adapter, would rely on.

**The WiseMan tests** added are:

- the triples checked against `TryPatchRom` for every ROM address, as MoonRT's test does;
- a second pad ignored on both engines;
- an illegal opcode in a plain frame;
- each channel muted alone on both engines.

`mercuryrt_rom_patch` had been added for the patch test, but that test had not been written.

**R11 survived the first test written for it.** The machine pair compared pictures only when rendering was on. It now
compares them with rendering skipped as well, but on the interrupt program that change alone caught nothing, because
that program's picture is white whether it is rendered or not. The colour program writes its palettes every frame, and
it catches the mutant.

#### 8.7.5 What is not done

- **The alignment on the handheld.** It is not timed there. The Legion Go S is the machine where the Game Boy engine
  is played, so this is owed before any claim about the player's frame.
- **The duplication between the two 8-bit shims.** It is listed in `EmuSen_NativeCores.md` §12.5 as a candidate for
  `NativeRtCore`, and was not moved.
- **Reads are still not reported**, and the other gaps of §8.5.6 stand. The move changed none of them.

## 9. The three defects fixed, in both engines (2026-09-24)

*Written 2026-09-24, on the decision recorded under Q3 (§5).* Each defect of §6.1 was fixed in C# Mercury and
MercuryRT together. For each, a test was written first and committed while it failed on both engines, on the unmodified
build (`291a661`). The tests are `EmuSen.WiseMan/Cores/MercuryDefectTests.cs`, one case per engine through `ICore`
alone, so neither engine's answer is read through the other's objects.

### 9.1 D1: the sample rate

**Before.** 600 frames of 70,224 cycles are 10.0456 s of console time. At a labelled 44,100 Hz that is 443,012.0 stereo
frames. Both engines drained **443,520**, which is 44,150.6 a second: §6.1's measurement, reproduced.

**The fix.** `Apu.SetSampleRate` divides in doubles, so a sample is 95.1089 clocks rather than 95. MercuryRT does not
divide on its own: the shim's `MercuryMachine.SetSampleRate` computes the same double and `Math.Pow` once, in C#, and
passes both down (§3.3). The Rust machine's own default, used only where no shim is present (`examples/frames.rs`),
got the same change.

**Prediction, stated before the run.** Both engines drain 443,012 ± 1; the two engines stay identical in sound, because
they share the two doubles; and nothing in the state moves, because the mixer's accumulator is not state (§3.1).

**After.** Both engines drained **443,012**, 44,100.0 a second. The parity suite (`MercuryRt` filter, 81 cases, the
four games and the corpus included) passed: state, picture and sound identical every frame. Kirby's 3,000 frames from
boot are 210,572,292 cycles, and 2 × ⌊210,572,292 / 95.1089⌋ = 4,428,022 samples is what both engines drained. That
the state did not move is checked in §9.3, against hashes of states the unmodified build wrote.

**What it changes.** Every Game Boy game's sound comes out 0.115% slower in samples per frame, which is the rate the
frontends were already told. `Mercury_Apu.md` §6's "95.11 clocks per sample" was the design and is now the behaviour.
The audio differential against gambatte (`Mercury_HardwareTests.md` §6) compared against a mislabelled rate before this
date; that comparison was not re-run. No corpus verdict can move: sound never feeds the machine.

### 9.2 D2: the cartridge's clock in double speed

**Before.** A synthetic MBC3+TIMER colour cartridge enables the clock, optionally sets `KEY1` and executes `STOP`,
then latches the clock and copies its seconds register to WRAM forever. After 1,200 frames, which are 20.09 s of
console time, both engines read **40 s** in double speed and 20 s in single speed.

**The fix.** `MemoryBus.Tick` hands the board base-clock cycles: the CPU's cycles in single speed, half of them in
double speed. The halving is exact. Every `Tick` is a whole number of machine cycles, which is four T-cycles, or a
DMA stall of 16 or 32. So the count is always even, and `_baseClockPhase` is false at every `Tick` boundary: the
argument by which §8.4 found mutant M20 equivalent. `Mbc3.Tick` still counts to `CpuClockHz`, now in base-clock
cycles. That is 128 of them per tick of a 32.768 kHz crystal.

**Prediction.** Both engines read 20 s at either speed. A frame-by-frame lock-step run of the double-speed program on
both engines is identical in every field, the clock's sub-second count included. Nothing that never enters double speed
changes. The four games do not. Of the eleven boards, none runs the clock in double speed, and no corpus ROM runs an
MBC3 clock at all. So their states at recorded frames still hash as the unmodified build's did (§9.3).

**After.** Both engines read 20 s at both speeds, and the lock-step case
(`D2_the_clock_in_double_speed_runs_identically_on_both_engines`) was identical for 1,200 frames in state, picture and
sound.

**The referee.** The MiSTer Game Boy core counts its MBC3 clock on `ce_32k` (`rtl/mappers/mbc3.v:193`, the sub-second
counter; `:244-251`, the carry into seconds). `ce_32k` is `clk_sys` divided by 1,024 (`Gameboy.sv:405-410`), and
`clk_sys` is 33.554432 MHz (`rtl/pll/pll_0002.v:28`), which gives exactly 32,768 Hz. It is generated beside, not from,
the CPU enables `ce_cpu` and `ce_cpu2x` that `speedcontrol` produces (`Gameboy.sv:798-820`). So the RTL agrees with the
fix: the clock does not double. The RTC in `mbc3.v` is its own Verilog, not a translation of an emulator's code, so this
agreement is evidence of some weight.

The referee also shows two things Mercury does not do. Neither was changed here.

- **A write to the seconds register zeroes the sub-second counter** (`mbc3.v:228-230`). Mercury's `Mbc3.WriteRam`
  leaves `_cyclesIntoSecond` as it was. That makes the next second arrive early by up to a second after a game sets the
  clock. This is a defect candidate with no test. It is not in §6.1, and Q4's rule (§5) says it is fixed in both
  engines together when it is taken up.
- **Fast-counting of seconds restored from a save file** (`mbc3.v:174`, `diffSeconds`). This catches the clock up by
  wall-clock time between sessions. Mercury's clock is not in the `.srm` at all, and it stops while the emulator is
  closed. That is a missing feature (`Mercury_Memory.md` §9), not a timing defect.

### 9.3 D3: the host's path in the state, and state version 6

**Before.** A battery cartridge saved a state at frame 10 while played from path A. The state was loaded while the same
ROM was played from path B, and play continued to frame 300, where the autosave runs. On both engines the loader
wrote **A's** `.srm` and never B's. With the loader under `--nobattery`, it still wrote A's. With the saver under
`--nobattery`, the state carried `""`, and the loader wrote its cart RAM to **`.tmp` in the working directory**. These
are §6.1's two measurements, now on both engines.

**The fix, and the format change it forces.** The path is the host's. A running machine still needs one, so the field
stays in the class. What changes is that the state walk no longer visits it. The same version drops the two extra
copies of the cartridge, each mapper's `_cart` and the bus's, which cost Pokémon Yellow 64 KB (§3.1). All three
fields carry a new attribute, `[RetiredFromState]` (`EmuSen_Save_States.md` §7). The current format skips them. A
version-5 read walks them where version 5 wrote them: the path is read and dropped, and each copy is read into the
one cartridge, as version 5's reader did.

- **C#.** `MercuryCore` writes version 6 and reads 5 and 6.
- **MercuryRT.** Rust has no reflection, so its reader carries the version the header names (`StateReader::before`).
  The cartridge, each board and the bus read the retired fields only for a version before 6. `Cartridge` lost its
  `save_path`.
- **The C ABI.** `mercury_machine_new` lost its path argument, and `mercury_machine_save_path` is gone, so the
  interface version is 2. The shim keeps the path itself, from `LoadRom`, as C#'s `Cartridge` does. This removes the
  ABI parameter and the string primitive §2.3 said D3 alone required. The string reader stays, for version 5.

*Superseded in part, 2026-09-24.* The Model setting made the state version 7, one byte naming the console after the
version, and gave `mercury_machine_new` a model argument, so the interface version is 3 (`Mercury_Model.md` §5).
Everything above about version 6 still holds, and version 7 reads 5 and 6 as described here.

*Superseded in part, 2026-09-28.* `mercury_machine_set_audio_limit` carries `AudioSettings.AudioBufferMaxSamples` to the
Rust queue, which now trims after the new pair as C#'s shared `SampleQueue` does, so the interface version is 4
(`EmuSen_Settings_Reference.md` §4.85.2).

**Prediction, stated before the run.**

- All six cases of the path test pass on both engines.
- Every version-6 state is smaller by the path and two copies of the cart RAM. For Yellow that is 148,420 → 82,879.
- The unmodified build's version-5 states load on both engines into the machine that wrote them.
- Nothing about how a machine runs changes.

**How "version 5 still loads" was proven, rather than argued.** Before the fix, the unmodified build ran the
synthetic program on all eleven boards for 300 frames, set the save path to `/mercury-v5/A.srm`, and saved. The
SHA-256 prefixes of those eleven states are in the test (`Version5Boards`). After the fix, the test runs the same
program on the fixed build, and regenerates version 5's bytes with `StateSerializer.Write(includeRetired: true)`. It
then asserts four things:

1. the bytes hash to the recorded values;
2. both engines read them into a machine whose version-6 state equals the writer's;
3. C#'s path stays null;
4. the two engines then run 120 frames identically.

(1) is a double check. It shows the retired walk reproduces version 5 byte for byte. It also shows that D1 and D2 left
these eleven machines' states exactly where the unmodified build put them. The same was done for the four games, from
states the unmodified build saved at frames 3,000 and 3,600 and kept outside the repository
(`EMUSEN_MERCURY_V5_STATES`). Each game's version-5 state at frame 3,000 was loaded into both engines. Each engine ran
600 frames with the bench's input, and the result equalled the unmodified build's own frame-3,600 state loaded fresh.

**After.**

- The path test: 6 of 6 pass.
- Version 5 on the eleven boards: 11 of 11. Version 5 on the four games: 4 of 4.
- Crate tests: 11. The Rust test builds a version-5 state from a version-6 one, and gives each of the three copies its
  own fill; the last copy stands.
- State-format filters: `MercuryRtStateTests` (layout, noise, odd bytes, refusals and games), `StateSerializerTests`
  (with a case for the attribute) and `MercuryRtCoreTests`. All pass, 60 cases in all.

Sizes, version 5 → version 6:

| Machine | Version 5 | Version 6 |
|---|---|---|
| Tetris (no MBC) | 17,338 | 17,333 |
| Kirby's Dream Land (MBC1, no RAM) | 17,348 | 17,343 |
| Link's Awakening (MBC1 + 8 KB) | 41,924 | 25,535 |
| Pokémon Yellow (MBC5 + 32 KB, CGB) | 148,420 | 82,879, as predicted |
| Synthetic MBC3 + clock + 32 KB, CGB | 148,489 | 82,897 |
| Synthetic MBC5 + rumble + 128 KB | 410,615 | 148,415 |

The synthetic states' version-5 sizes include the 17-byte test path three times; the games' do not, because they
were saved under `--nobattery` with a null path.

**The whole parity suite and the corpus after all three fixes.** The `MercuryRt` and `MercuryDefectTests` filters
passed, 106 cases, with the games and the corpus. The C# corpus transcript matches the recorded baseline
(`Mercury_Native.md` §3.4) in all 173 verdicts and verdict frames, 94 passing. It was taken after D1 and D2, and again
after D3. **No baseline was re-recorded, because none moved.** None could: D1 touches no state, no corpus ROM runs an
MBC3 clock, and D3 changes only the serialized form.

**What changed in the tests.** Two changes follow from the format. `MercuryRtStateTests`' layout walk skips retired
fields as the serializer does. Its odd-bytes case now patches `Mapper._ramEnabled` in place of the retired
`Mapper._cart` presence flag. The noise case no longer writes a save path, because no path is in the state.
`MercuryRtPair` no longer resets the path after a transfer, a workaround §8.2 recorded for D3. It gained a `state`
argument for the version-5 cases.

**What this does not cover.**

- A version-5 state written by a MercuryRT built before this date is the same bytes as C#'s (§8.1), so it is
  covered.
- A state from any version before 5 was refused before and still is.
- The frontends' own state records (`StateRecord.StateVersion`) now say 6 for new Game Boy states. Mistress refuses
  only newer versions (§6 of `EmuSen_Save_States.md`), so an old record for a version-5 state still loads.

### 9.4 The recorded baselines the fixes moved, re-recorded deliberately

Stage 7 (§8.6) committed `MercuryRtPlatformTests` on WiseMan while the fixes were being made. It holds both engines'
sound, picture and state after 600 frames of four synthetic programs, and the C# mixer's capacitors, to digests
recorded on linux-x64. It was merged into this branch after D3, and so it met the fixes for the first time there.

**Prediction, stated before the run.**

- Every sound digest and every capacitor digest moves (D1: a different sample clock and a different charge factor).
- Every state digest moves (D3: version 6 is a different byte string).
- No picture digest moves: none of the three fixes reaches the renderer.
- The two engines stay equal to each other on every digest.

**Result.** It held on all four programs. Seven of eight cases failed against the old digests, each on its sound or
capacitors. Only `The_high_pass_charge_factors_are_linuxs` passed; it computes both `Math.Pow` values itself and
names both. The four picture digests came out byte-identical to stage 7's, and the two engines agreed on all twelve
new digests. The new values were then recorded in the test.

| Program | Sound (before → after) | State (before → after) | Capacitors (before → after) |
|---|---|---|---|
| four channels | `80D5E574…` → `420DFAA8…` | `9E73C418…` → `04BEB9D4…` | `9597B2BA…` → `50E85615…` |
| busy | `C5414EFA…` → `B327BA1B…` | `768FAE58…` → `2C9A5F6A…` | `510D50CA…` → `8FB248BA…` |
| interrupts | `8199C9D1…` → `2802C96F…` | `838CF3A8…` → `033ADBB7…` | `15CA5785…` → `4D930F46…` |
| colour | `28854D0C…` → `D3A2D8F2…` | `D5272466…` → `900245C1…` | — |

These were recorded on linux-x64 only. Windows and macOS meet the new digests on CI's next run, which is §8.6's
open item: nothing is claimed for them here. `pow(seed, 95)` in that test is now the charge factor of a mixer that
no longer exists. It is kept, because the two values side by side are how a reader sees what D1 changed.

**Other baselines.** The corpus transcript and the state hashes of §9.3 did not move, so nothing else was
re-recorded.

**What the referee pass adds to D2** (`Mercury_Referee.md` §2.8, committed on WiseMan after D2). Both references agree
against Mercury on four more MBC3 rules:

- writing the seconds register zeroes the sub-second count (the one §9.2 also found);
- the registers are 6, 6, 5 and 9 bits wide;
- a register carries only on reaching exactly 59, 59 or 23;
- MBC5's RAM enable compares all eight bits.

None of these fell out of D2's test: its program never writes the clock and never lets a register overflow. So none was
fixed here, and each stays a candidate with no test, to be fixed in both engines together (Q4, §5).

### 9.5 Mutants of the three fixes

**Method.** Fourteen hand-made mutants were run in a copy of the tree at `a372058`, the D3 commit, so that no source in
the working tree was touched. For each mutant:

- one edit was applied;
- the crate's tests were run, for a Rust edit;
- WiseMan was built;
- the `MercuryDefectTests`, `MercuryRtStateTests`, `StateSerializerTests`, `MercuryRtCoreTests` and HDMA filters were
  run with the four games;
- the file was restored.

The unmutated copy passed all 68 cases first, as the control. The runner is `mutants.py`, in the scratch directory
(`~/.cache/emusen/probe/mercury-defects/mut/`).

| Mutant | Caught by |
|---|---|
| M1 C#'s `SetSampleRate` divides in integers again | the D1 test (C#), and every lock-step comparison, because the shim computes its own rate |
| M2 the shim divides in integers | the D1 test (Rust), and every lock-step comparison |
| M3 the Rust machine's own default divides in integers | **survived** (argued below) |
| M4 C# counts the clock in CPU cycles | both D2 tests |
| M5 Rust counts the clock in CPU cycles | both D2 tests |
| M6 C# halves the clock's cycles at single speed too | both D2 tests, and the version-5 hash of the MBC3 board |
| M7 C#'s save path back in the state | 47 cases: the D3 tests, the layout listing, and every cross-engine comparison |
| M8 Rust does not consume version 5's path | the crate test, and every version-5 case |
| M9 Rust reads the cartridge copies in version 6 too | the crate test, the D3 tests, and every state comparison |
| M10 the serializer restores a retired value instead of dropping it | the serializer's own case, and the version-5 cases (C#'s path is no longer null) |
| M11 Rust lets the first cartridge copy stand, not the last | **the crate test only** |
| M12 the shim forgets the save path | the D3 tests, and the battery test |
| M13 C# reads version 5 without its retired fields | every version-5 case |
| M14 Rust's reader is not told the header's version | every version-5 case |

**Thirteen of fourteen are caught.** M11 is caught only by the crate's test, the one that gives each of version 5's
three copies its own fill. A version-5 state that C# wrote has three identical copies, so no WiseMan case can see which
copy stands.

**M3 is equivalent through the ABI.** `Machine::load_rom` sets the Rust machine's default rate, and
`MercuryMachine`'s constructor replaces it with the shim's before any frame runs. The default is read only by
`examples/frames.rs`, which has no shim. So the mutant changes the samples that example drains and nothing any
frontend runs. It is left untested on purpose.

## 10. State version 8: the mixer in the state (2026-10-03)

**Decided 2026-10-03,** for Moon first (`Moon_Native.md` §3.13) and found to apply here: a state load must resume
exactly, sound included. Mercury marked its mixer's running state `[SkipInState]`, so a machine loaded with another's
state resumed its sound from its own mixer. The gap was checked for before MercuryRT's move onto the core ABI v1, and
it was there: the resampler's `_cycleFraction` and the two output capacitors.

**The format.** Version 8 appends, after the four walks, 24 bytes, in C# Mercury (`Apu.WriteMixer`) and MercuryRT
(`Machine::write_state`) alike: `Bus.Apu._cycleFraction`, `Bus.Apu._leftCapacitor` and `Bus.Apu._rightCapacitor`,
each a double. The magic stays `MERC`. The cycles per sample and the charge factor stay out: they follow
`SetSampleRate`, which is the host's configuration, not the machine's state (§9.1). Versions 5 to 7 still load in both
engines, the mixer keeping what it held, as before; every state written is version 8.

**What it proves** (measured 2026-10-03):

- `MercuryRtStateTests.A_state_loaded_into_a_second_machine_resumes_its_sound_exactly`: 300 frames, the state loaded
  into a second machine, 300 more, on a Game Boy and a Game Boy Color: identical samples frame by frame and identical
  states, in both engines.
- D3's test: a version 5 state still loads alike in both engines; it equals its source up to the mixer, which version
  5 does not carry.
- §9.4's recorded platform digests: the four synthetic programs' state digests are re-recorded, and the test now holds
  each new state to its old one. With its version field set back to 7 and its last 24 bytes removed, each version 8
  state hashes to the version 7 digest recorded on 2026-09-24. Sound and picture digests did not move.
- The layout walk names the three new fields, and both engines' layouts agree.


## 11. On the core ABI v1 alone, and two stale crate tests (2026-10-04)

**The shim.** `MercuryRtCore` is a subclass of the generic v1 adapter and loads MercuryRT through the v1 exports
(`EmuSen_CoreAPI.md` §26). It keeps C# Mercury's exceptions, its state pre-check messages, its battery folder rule
(`BatterySave.GameBoyFolder`), the Model setting's reload before the first frame and the mirror debugger. The mixer's
coefficients are still C#'s (§3.3): after every create the shim computes the division and `Math.Pow` and sends them
through `mercuryrt_core_set_sample_rate`, the v1 handle's twin of `mercuryrt_set_sample_rate`. `mercuryrt_core_rom_patch`
does the same for the exhaustive patch test. The pre-stable exports and `MercuryMachine` stay for the machine and
state tests until the pre-stable set is deleted.

**`MACHINE_INFO` on a state from the other console.** Since a state records its console (§9.3, `Mercury_Model.md` §5),
loading one made on the other console rebuilds the machine as that console, and machine info's `system` changes. The
v1 host was not told, and kept reading the old system: `MercuryRtCoreAbiTests` showed `gb` after a Game Boy Color state
was loaded. `Machine`'s `load_state` now compares the console before and after and emits the event; the pre-stable
path has no outbox and drops it. The shim's `CoreName` follows machine info.

**Two crate tests, stale since state version 8.** `cargo test --release` in the crate failed two machine tests on
`WiseMan` from `ce9dec07` (§10) on:

- `a_state_that_is_not_mercurys_or_is_cut_short_changes_nothing` used version 8 as a version no build reads. Version 8
  is now the current one, so the state loaded. The test now uses `STATE_VERSION + 1`.
- `version_6_writes_the_cartridge_once_and_version_5s_three_copies_read_with_the_last_standing` builds a version-5
  state from the current save. It kept the mixer's 24 bytes, which version 8 appends and a version-5 reader never reads,
  so cutting the state's last byte cut only those, and the truncated state loaded where the test expected `TRUNCATED`.
  The fixture now ends the version-5 state before the mixer.

The reader was right in both cases; the fixtures described the format before version 8. All 28 crate tests pass.

## 12. Stage 8: parity (2026-10-14)

*Decided 2026-10-14:* MercuryRT is finished now, stage by stage: this stage, then the goldens (stage 9), then the C#
core's move to its legacy branch (stage 10), then the Game Boy's default flipped to MercuryRT after a session of play on
the handheld (Q5). This section is stage 8's record. The older branch `mercuryrt-parity`, opened 2026-09-24 and left at
the stage's first step, holds no work and was not used.

### 12.1 WiseMan's tests on both engines

**The count before.** 554 WiseMan cases named for Mercury were listed at `175f9c36`. Most already ran both engines:
the state, machine, random-program, game, corpus, platform, debugger, defect, ABI and Model suites. 243 ran on C#
Mercury alone:

| Class | Cases | What it drove |
|---|---|---|
| `MercuryPpuTests` | 27 | `Bus.Tick`, `Bus.Write`, VRAM and OAM arrays, the picture |
| `MercuryApuTests` | 23 | the APU's registers through the bus, channel fields, samples, mutes, the debug target's peek |
| `MercuryCgbTests` | 23 | banking, palettes, HDMA's stall, `Bus.Stop`, single instructions |
| `MercuryCpuTests` | 18 | `Cpu.Step` on a loaded machine |
| `MercuryCycleTimingTests` | 15 | C#'s `Cpu` on a fake `ICpuBus` that records each access |
| `MercurySerialTests` | 14 | the link port, the serial log, and the verdict reader |
| `MercuryCoreTests` | 13 | the `ICore` surface, the timer, OAM DMA, a breakpoint |
| `MercuryCartridgeTests` | 12 | `Cartridge.FromImage` and the boards' objects |
| `MercuryAccessBlockingTests` | 4 | the PPU's blocking of VRAM and OAM, a dot at a time |
| `MercuryCommercialRom*Tests` | 7 | seven classes over the commercial images, one case each without them |
| `MercuryHardwareTestRomTests` | 1 | the corpus, asserting `Passed` (§12.2) |
| `MercuryModelTests`, the palette tables | 86 | `CompatibilityPalettes`, C#'s table of the Color's palettes for Game Boy games |

**The rig.** `MercuryRig` (`EmuSen.WiseMan/Fixtures/MercuryRig.cs`) is §3.5's proposal, built as proposed with one
change of place: the engine is held through its `ICore`, as a frontend holds it, so a test's frames go through the
shim a player runs and not around it. It has one implementation per engine:

- **Reads** go through `View`, a C# `MercuryCore`. On C# it is the core itself. On MercuryRT it is a second
  `MercuryCore` on the same ROM and model, loaded from MercuryRT's state whenever it is read after the machine moved.
  So `rig.Bus.Ppu.Ly`, `rig.Cpu.A` and `rig.Bus.Apu.Pulse1.Envelope.Volume` mean the same on both engines.
- **Setup writes** are of two kinds. A write the CPU would make (`rig.Write`, `rig.Read`) goes through the bus's
  decode on each engine: `Bus.Write` on C#, `write_space` on `CPUBUS` on MercuryRT. A field edit (`rig.Bus.Vram[i] = x`,
  `rig.Cpu.PC = a`, `rig.Bus.InterruptFlags = 0`) is made on the view and reaches MercuryRT by a state round trip. A
  view that was read is saved before the machine next runs, and loaded into MercuryRT only when its bytes differ from
  what was pulled.
- **Timing** is `RunFrame` through `ICore`, `Step` (one instruction with the frame loop's acknowledge and stall) and
  `Tick` (`MemoryBus.Tick`, the machine without its CPU). `Stop` is `MemoryBus.Stop`, the speed switch KEY1 armed.
- **What no state carries** goes through the engine: the picture, the samples, the serial log, the pad, the mutes,
  the breakpoints and the debug target.

Four test extensions were added to the library for the rig, under the crate's prefix and outside the ABI's promise
(`EmuSen_CoreAPI.md` §4.7): `mercuryrt_core_step`, `mercuryrt_core_tick`, `mercuryrt_core_stop` and
`mercuryrt_core_serial`. `step` and `serial` share their bodies with the pre-stable `mercuryrt_step` and
`mercuryrt_serial`, as `rom_patch` and `set_sample_rate` already did. They are on the v1 handle so that the rig
survives the pre-stable set's deletion (`EmuSen_CoreAPI.md` §26.5). Nothing a frontend calls changed.

**The classes.** Each converted class is an abstract set of claims, `Mercury*Claims`, run as `Mercury*Tests` on C# and
as `MercuryRt*Tests` on MercuryRT, as `MercuryDebugClaims` already was (§8.5.4) and as `Mars_Native.md` §6.5 did for
MarsRT. The `MercuryRt` prefix puts the Rust half in CI's `MercuryRt` filter, so the workflow's `wiseman` job runs it on
Windows and macOS too. The commercial-image classes and the corpus class take the engine as a theory argument instead,
because their cases already came from data.

**The prediction, retired.** §3.5 argued that "most assertions carry over unedited". It held for the assertions: no
expected value in the converted classes changed. What had to change was every call that runs machine logic, because a call on the view would have run C#'s logic on MercuryRT's
state and passed for the wrong reason: `Bus.Write`, `Bus.Read`, `Bus.Tick`, `Bus.Stop` and `Cpu.Step` became the rig's.
Five assertions were rewritten, each for a reason the rig's design forces:

- Two captured sub-objects (`var pulse = core.Bus.Apu.Pulse1`, `var wave = ...`) were read after the machine ran. A
  captured view object goes stale on MercuryRT, so the reads go through `rig.Bus` each time.
- `The_flag_register_never_keeps_its_low_nibble` set `Cpu.AF` and read `F`, which tests a C# property setter. It now
  runs `LD BC,$12FF / PUSH BC / POP AF`, the SM83's own way of writing F.
- `A_save_state_carries_the_port_registers_but_not_the_transcript` cleared the C# serial log by hand, which no
  interface offers. It now clocks a second byte out after the save and asserts that the load leaves both bytes in the
  log, the same claim from the other side.
- `Reset_clears_both_the_port_and_the_transcript` called `MemoryBus.Reset`. It now loads the game again, which is where
  the bus is reset on either engine.

The two CPU-test helpers also changed what a step is. They ran `Cpu.Step` and then ticked the bus by the step's cycles
again, so the machine ran each instruction twice over. That had no effect on any assertion; the rig's step is the
frame loop's.

**The count after.** 762 cases are named for Mercury; 750 run under the `Mercury` filter, and all pass.

| | Claims | Cases |
|---|---|---|
| Now on both engines | 215 | 4 + 23 + 4 + 23 + 7 + 13 + 18 + 27 + 9 + 1 + 86 claims; 208 new MercuryRT cases in WiseMan (the eleven palette-number rows run both engines inside one case) |
| C#-only, by design | 23 | listed below |
| On no engine | 5 | the verdict reader's own cases, `HardwareTestRomLibrary.ReadVerdict` on bytes |

The commercial-image classes were run with the four scratch copies (Tetris, Kirby's Dream Land, Link's Awakening and
Pokémon Yellow) linked into the gitignored `TestRoms/`: 56 of 56 pass, seven claims × four games × two engines.

**C#-only, and why.**

- **`MercuryCycleTimingTests`, all 15.** They hand C#'s `Cpu` a fake `ICpuBus` that records every read, write and tick
  in order. A WiseMan test cannot hand MercuryRT a bus. The same claims now run on the Rust SM83 in the crate, through
  its `CpuBus` trait: `tests/cycle_timing.rs`, six tests that cover the fifteen cases (the ten opcodes of the theory are
  one loop). Both pass, so the claims are on both engines, in two harnesses.
- **`MercuryCartridgeTests`, the eight header cases**: the title, the checksum, the three values of the colour flag,
  the shorter colour title, MBC2's built-in RAM size, and an image too short for a header. They test
  `Cartridge.FromImage`. MercuryRT's shim parses the header with that same C# class before the image reaches Rust
  (`MercuryRtCore.Prepare`), so the C# answer is the one a player of either engine gets. MercuryRT's own reading of the
  board and the RAM size is held by the four board claims, which now run on both engines through the bus, and by the
  state tests, whose layouts depend on it. The board claims replace `mapper.ReadRom` and `mapper.WriteRom` on C#'s
  objects with the CPU's reads and writes.
- **§3.5's two named examples.** The reflection serializer's own tests are `StateSerializerTests`, outside the Mercury
  classes, and stay about C#. `SkipRendering`'s allocation checks do not exist for the Game Boy: no WiseMan test
  measures what a Mercury frame allocates, on either engine, so there was nothing to keep C#-only.

**The palette tables.** The 86 cases called C#'s `CompatibilityPalettes` directly. MercuryRT has its own copy of the
table (`ppu/compat.rs`), which §8's tests held only through one representative title per palette number
(`Every_palette_number_renders_identically_on_both_engines`): a wrong entry for one listed game in Rust would have
survived. Each row now also builds the game's header, creates the machine as a Game Boy Color on each engine, and reads
colour RAM through the view before the first instruction. TCRF's 73 rows and Radar Mission are a case per engine; the
eleven palette-number rows check both engines in one case; the default palette is a case per engine.

### 12.2 The corpus as a recorded baseline

*Decided 2026-09-24 (Q6, §5):* the corpus stays gitignored under `TestRoms/hardware/`, and the committed theory becomes
"matches the recorded baseline".

**What existed.** The committed form was `MercuryHardwareTestRomTests.A_hardware_test_rom_reports_a_pass`, one case per
ROM asserting `Passed`, on C# only and through the serial port only. It would have failed 79 of 173 ROMs with the corpus
installed (§3.4), so it was an assertion nobody could run green. The two lock-step corpus runs
(`MercuryRtCorpusTests` and `MercuryModelTests.The_hardware_corpus_runs_identically_on_each_console`) compare the engines
with each other every frame, but against nothing recorded, so a change made to both engines alike passes them. The
per-ROM verdict tables of §3.4, §9.3 and `Mercury_Model.md` were scratch transcripts, never committed. The theory was
completed in place rather than added beside it.

**The form.** `MercuryHardwareTestRomTests.Each_hardware_test_rom_says_what_the_baseline_recorded` is six cases, one per
engine and Model (Auto, Game Boy, Game Boy Color). Each runs every ROM of the corpus to its verdict or 3,600 frames, with
rendering skipped (§6.5) and battery saves off, and compares each verdict with its column of
`EmuSen.WiseMan/Cores/MercuryCorpusBaseline.tsv`. The file has one row per ROM: its path under the corpus, the first
sixteen hex digits of its SHA-256, and the six verdicts. A verdict is read in the order the ROMs report:

- the serial log, by `HardwareTestRomLibrary.ReadVerdict` (§2 of `Mercury_HardwareTests.md`): `Passed`, `Failed`;
- blargg's cart-RAM protocol (§3.4), by `HardwareTestRomLibrary.ReadCartRamStatus`: `Passed` for status 0, `Failed(n)`
  for status n, so a change in which subtest fails also shows;
- `NoVerdict` at the cap, and `IllegalOpcode` if the machine stops on one.

The case fails on any difference in either direction, a regression or an improvement, and names the ROM, the recorded
verdict, the new one and the frame it was reached on. A ROM whose hash differs from the recorded one is reported as a
different ROM rather than as a moved verdict, so a newer corpus does not pass for a behaviour change. A ROM recorded and
absent, or present and unrecorded, also fails. With no corpus, each case says so and passes, as the class always has.

**Where the corpus is found.** `HardwareTestRomLibrary.Root`, `TestRoms/hardware/` under the configuration root, searched
recursively; `EMUSEN_MERCURYRT_CORPUS` overrides it, and is now owned there and read by the lock-step tests too, so the
two cannot point at different corpora.

**Re-recording** is deliberate, as LunaP's `EMUSEN_API_APPROVE` is:

```
EMUSEN_MERCURY_CORPUS_RECORD=1 dotnet test EmuSen.WiseMan --filter "FullyQualifiedName~MercuryHardwareTestRomTests"
```

Each case then writes its column into the source tree's file (found from the test's own source path), keeping the
other columns, and passes. The diff of that file is the review, and the commit that carries it states its reason, as
§3.7 asks of a re-baseline. A filter naming one engine and model re-records one column.

**The baseline, recorded 2026-10-14** on the 173 ROMs of §3.4 (blargg's `gb-test-roms` and mooneye's
`mts-20240926-1737-443f6e1`):

| Engine | Auto | Game Boy | Game Boy Color |
|---|---|---|---|
| Mercury (C#) | 94 passed, 74 failed, 5 no verdict | 94, 74, 5 | 94, 74, 5 |
| MercuryRT | 94, 74, 5 | 94, 74, 5 | 94, 74, 5 |

The two engines agree on every ROM in every column. Auto and Game Boy agree everywhere; Game Boy Color differs from them
on two ROMs only, in opposite directions: `acceptance/boot_regs-dmgABC` passes on a Game Boy and fails on the Color, and
`misc/boot_regs-cgb` the reverse (`Mercury_Model.md`). The equal counts across the columns are those two cancelling.

**Checked against what was recorded before.** The Auto column reproduces §3.4's 94 of 173, and all three columns equal,
ROM by ROM, the three scratch transcripts of 2026-09-24 that the Model work recorded (`Mercury_Model.md`), up to the
spelling of "no verdict". So the baseline is not a new measurement of the machines; it is the old one, now committed.

**That it fails both ways** was shown before it was committed: with `cpu_instrs.gb` recorded as `Failed` in the C#/Auto
column and `boot_regs-dmgABC` recorded as `Passed` in the MercuryRT/Game Boy Color column, those two cases failed, each
naming its ROM ("recorded Failed, now Passed at frame 1784"; "recorded Passed, now Failed at frame 11"), and the other
four passed. The run takes 1 min 52 s for all six with the corpus, most of it the C# columns: about 33 s each against 4 s
for MercuryRT's, with four ROMs at a time. That ratio was not investigated; it is the harness's cost, not the machines'.

### 12.3 What the engines disagreed on

Two differences appeared, both in the first run of the converted classes on MercuryRT. Neither is in the machine.

**E1: MercuryRT's debug target had no audio peek.** `MercuryRtApuTests.The_debug_target_reports_four_channels_and_a_live_peek`
failed ("Assert.NotEmpty() Failure: Collection was empty") on the unchanged library and shim. §8.5.6 listed this as
known: the target read the mirror's APU queue, and the queue is in no state, so the mirror's is always empty. C# Mercury
peeks its live queue. *Fixed by addition:* MercuryRT now has the v1 capability `AUDIO_PEEK` (`peek_audio`, a copy of the
queue without draining it), the shared `SampleQueue` gained `peek` with a test of its own, and `MercuryRtDebugTarget`
reads `CoreMachine.PeekAudio`. The capability word in `MercuryRtCoreAbiTests` gained the bit. After the fix the case
passes on both engines, and nothing a game hears changed: the peek does not drain.

**E2: a button pressed between frames.** `MercuryRtCoreSurfaceTests.A_pressed_button_pulls_its_line_low_in_the_selected_nibble`
failed (expected 0, read 8): pressed through `ICore.SetButton`, the button did not show in `$FF00` before the next frame.
C# Mercury writes the joypad at once. The v1 adapter holds the pad in its input latch and hands it to the machine before
each frame, so that a frontend may press from any thread (`Diamond_Players.md` §12.7). This is a choice of the adapter's,
shared by every v1 core, and it is not a defect of either machine. It shows only to a read made between frames by a
host: a game reads the pad while it runs, which is inside a frame, and §8.2's frame-by-frame comparisons with input
scripts are identical on both engines. The claim now presses, runs a frame, and then reads, and it passes on both. The
difference is kept and written down here; a debugger poke of `$FF00` between frames is the one place a person would see
it.

**Nothing else differed.** Every other converted claim passed on MercuryRT at the first run, and the corpus baseline's
columns agree ROM for ROM.

**The frame's cost.** The machine's per-cycle code did not change (`cpu/`, `memory/`, `ppu/`, the APU's channels and
mixing); the library gained four test extensions and a peek, and the mixer's state walk moved into `Mixer` (§12.4).
The crate's bench (`examples/frames`, 3,000 frames after 900, p50 in ms) was run on the crate at `175f9c36` and at this
stage, three interleaved rounds under the timing lock each time, measured 2026-10-14 after the extensions and again
2026-10-15 after the walk moved:

| Game | before (14th) | after (14th) | before (15th) | after (15th) |
|---|---|---|---|---|
| Tetris | 0.4325, 0.4355, 0.4366 | 0.4308, 0.4363, 0.4364 | 0.4320, 0.4305, 0.4302 | 0.4294, 0.4275, 0.4293 |
| Kirby's Dream Land | 0.4924, 0.4908, 0.4887 | 0.4843, 0.4895, 0.4958 | 0.4867, 0.4841, 0.4842 | 0.4813, 0.4786, 0.4800 |
| Link's Awakening | 0.4004, 0.3996, 0.3993 | 0.3998, 0.4010, 0.3988 | 0.3988, 0.3977, 0.3984 | 0.3989, 0.3976, 0.3979 |
| Pokémon Yellow | 0.5396, 0.5378, 0.5408 | 0.5378, 0.5312, 0.5377 | 0.5374, 0.5287, 0.5370 | 0.5290, 0.5286, 0.5304 |

Every pair is within 1.6%. On the 15th every "after" is at or under its "before", by up to 1.6% on Yellow; a cold
state walk cannot make a frame faster, so that is placement or noise, and is not claimed. The example links the
machine and not the cdylib's export table, so this says the machine's code is where it was; §8.7.3's finding that
placement in the shipped library can move a frame was not re-measured through `ICore`.

### 12.4 Mutants for what had none

**What was already covered.** Stage 1 (§8.1, 10), stages 2–5 (§8.4, 23), stage 6 (§8.5.4, 19 and two wide-table ones),
stage 7's digests (§8.6), the move onto the common interface (§8.7.4, 19), D1–D3 (§9.5, 14) and the Model setting
(`Mercury_Model.md` §6.8, 23). Nothing covered state version 8's mixer (§10), the v1 adapter's own code (§11), the
Color's palette table in Rust, the machine areas that only the converted claims reach, or this stage's extensions.
§3.6 also asked for two classes on purpose.

**Method,** run 2026-10-14. The crate and `emusen-native` were copied to scratch space, and a copy of WiseMan's build
output was made there, so that no source and no output in the tree was touched. For each mutant the runner
(`mutants.py`, scratch) applied one edit to the copy, built the library with release settings less LTO, ran the
crate's tests, put the library in the output copy (its sidecar's hash updated), and ran WiseMan's `MercuryRt`,
`MercuryModelTests` and `MercuryDefectTests` filters with the four games (`EMUSEN_MERCURYRT_ROMS`), then restored the
file. The unmutated control passed: the crate, and 514 of 514 WiseMan cases. The four ordering mutants were then run a
second time against the corpus alone: the baseline (§12.2) and the lock-step `MercuryRtCorpusTests`. Each WiseMan run is
bounded twice: vstest's hang detector at five minutes, which names the test that hung and aborts the run, and an outer
limit that kills the run's whole process group.

**The first round hung once.** The first runner bounded each command at an hour and killed only its direct child.
Mutant X2 (the tick extension one T-cycle short) turned `Tick(1)` into no tick at all, so the access-blocking claims'
`while (Dot != dot) Tick(1)` never returned. The runner's hour expired, and the test host it had started outlived it,
holding the shared test lock until it was killed by hand two hours later. The runner now starts each command in its own
session and kills the group, and the hang detector catches the same mutant in five minutes. A hang is a catch: it
is recorded below with the test that hung.

| # | Mutation | Caught by |
|---|---|---|
| **The ordering of `step_one_cycle`** (§3.6's first class) | | |
| O1 | TIMA's reload counted down after the edge detector, not before | the interrupts program on all eleven boards, `The_window_counts_its_lines…` and the rendering-skip pair; **not** by the four games, the corpus's verdicts, or its lock-step run |
| O2 | the OAM DMA byte before the PPU and APU, not after | **survived** the first round and the corpus; caught by the new `An_oam_dma_byte_landing_on_the_composite_dot_draws_identically` |
| O3 | the frame sequencer reads DIV before its increment | 38 cases: the rig's envelope and sweep claims, the synthetic programs, the games, the platform digests; in the corpus, the lock-step run |
| O4 | DIV incremented after the timer, so its edge sees the old counter | 22 cases: the interrupts program, random programs, the games, the platform digests; in the corpus, the lock-step run and all three MercuryRT columns of the baseline |
| **Same-width neighbours in the state** (§3.6's second class, as it stands since D3) | | |
| S1 | the writer puts each capacitor under the other's name | 23 cases, the games' byte round trip and the lock-step programs among them; **not** by the crate |
| S2 | the reader takes the capacitors in the other order | `Real_games_states_come_back_from_rust_byte_for_byte` only, a case that needs the games |
| S3 | the capacitors written in the other order, their labels with them | 45 cases; the running-machine and noise round trips among them |
| S4 | MBC3 writes its ROM and RAM banks under each other's names | the crate's naming rule and board round trip; 19 WiseMan cases |
| S1b, S2b | S1 and S2 again, after the mixer moved into `Mixer::write_state` and `read_state` | the crate's naming rule; and as before (23 and 1 cases) |
| **State version 8 and the v1 adapter** (§10, §11) | | |
| V1 | version 8's mixer not read back | the crate; 22 cases, the sound-resume claim among them |
| V2 | no `MACHINE_INFO` for a state from the other console | `A_state_from_the_other_console_tells_the_host_its_machine_info_changed`; the debugger's call stack across a rebuild |
| V3 | the setting "Game Boy" builds Auto | `The_debugger_shows_the_console_the_game_runs_on_on_both_engines` only |
| V4 | a write to a space the machine does not have accepted | **survived**; caught by the new crate test `a_space_the_machine_does_not_have_is_refused_through_the_core_abi` |
| **This stage's additions** | | |
| X1 | the audio peek returns nothing | E1's claim, `The_debug_target_reports_four_channels_and_a_live_peek` |
| X2 | the tick extension one T-cycle short | 19 rig claims (PPU, APU, CGB, OAM DMA, the O2 test); then **a hang** in `MercuryRtAccessBlockingTests.Oam_is_closed_for_the_scan_as_well_as_the_draw`, stopped by the hang detector |
| X3 | the stop extension does nothing | four rig claims: the APU's and the CGB's double speed, the colour state's round trip |
| X4 | a step leaves an HDMA stall pending | the random programs, instruction by instruction |
| T1 | two checksums of the Color's palette table swapped | the TCRF rows on MercuryRT; `Every_palette_number_renders_identically_on_both_engines` |
| **Machine areas the converted claims reach** | | |
| N1 | noise's short mode feeds bit 5, not 6 | `Short_mode_makes_the_noise_repeat_after_127_steps`; a game; the platform digests; random programs |
| N2 | wave volume code 2 shifts by two | the games only |
| N3 | NRx4 reads its length-enable bit as 1 | `The_unreadable_register_bits_all_read_back_as_one`; the games |
| N4 | STAT's unwired top bit reads 0 | `Stat_reports_the_mode_the_lyc_match_and_its_unwired_top_bit`; the games |
| N5 | eleven sprites a line | `Only_the_first_ten_sprites_on_a_line_are_drawn`; the games |
| N6 | SVBK 0 selects bank 0 | `Svbk_banks_the_upper_wram_window_and_treats_zero_as_one`; the HDMA program; random programs |
| N7 | cancelling an hblank HDMA keeps its blocks | `Clearing_bit_seven_cancels_a_running_hblank_hdma` only |
| N8 | SC's clock-speed bit read as 0 on a Game Boy | `Unused_control_bits_read_back_as_ones_on_a_dmg`; three Model cases |
| N9 | a write to echo RAM dropped | `Echo_ram_is_the_same_storage_as_work_ram`; random programs |
| N10 | writing DIV clears only its visible byte | the random programs only |
| N11 | the palette index steps without bit 7 | `The_palette_index_auto_increments_only_when_its_top_bit_is_set` only |
| N12 | the window line counts before WY is reached | the window claim; the window step test; the games |

**Result.** All 31 are caught; three needed a new test (O2, V4, and S1/S2's move under the naming rule). The converted
claims are the only oracle for three of them (N7, N11, X1), and the only one without the games for three more (N3–N5);
CI has no games. N2 is caught by the games alone, and N10 by the random programs alone: the converted
`Writing_div_resets_the_counter` cannot see N10, because DIV is the counter's high byte and a test that reads only DIV
reads zero either way. The claim was left as it was: it says what C# says, and the random programs hold the rest.

**§3.6's two predictions, retired.**

- *"Ordering could hide below game-level comparison and be caught only by mooneye's timer ROMs."* Half right. O1 hid
  below the four games, as predicted, but it hid below the corpus too: no verdict moved and the lock-step run stayed
  identical. What caught it was a synthetic program that takes timer, STAT and vblank interrupts every frame and is
  compared state for state. O2 hid below everything until a test aligned a DMA with the composite dot, which §8.4 had
  argued was the only place it could show (M7). O3 and O4 were caught widely. The class is real, and the corpus is not
  its oracle; the lock-step synthetic programs are.
- *"The cartridge copies could hide below a byte round trip and be caught only by `naming.rs`."* The copies are gone
  (D3, §9.3), and §9.5's M11 is their last trace. The class that replaces them is any two same-width neighbours in the
  walk, and the prediction failed for the first such pair tried: `naming.rs` never saw the mixer's three doubles,
  because the rule reads only lines of the form `w.f64("Label", self.field)` and `self.field = r.f64()?; // Label`, and
  version 8 wrote them as `mix.left_capacitor` with no comment on the read. S1 was caught by WiseMan's comparisons with
  C#, and S2 by one case that needs the games. The mixer's walk now lives in `Mixer::write_state` and `read_state`,
  under the group `Bus.Apu`, so the bytes and the layout listing are the same and every line is under the rule; S1b and
  S2b fail the crate's naming test. A layout that is self-consistent (S3) is outside any rule that compares a line with
  itself, and is caught only against C#'s walk; when C# leaves `main`, the committed layout listings of §3.7 take that
  role.

**Divergence kept.** None. **Equivalents argued.** None this round: O2, the one candidate (§8.4's M7), is now shown
not to be equivalent, and has its test.

### 12.5 What is not done, and what stage 9 needs

**Not done.**

- MBC2's built-in RAM size, the header's title and checksum, and the short-image refusal are held on C# alone (§12.1).
- E2 is kept, not changed (§12.3).
- The C# columns of the corpus baseline take eight times MercuryRT's; not investigated.
- The frame's cost through `ICore`, as a player runs it, was not re-measured; only the machine's, by the crate's bench.
- The handheld was not part of this stage.

**What stage 9 needs from this one.**

- **The view.** `MercuryRig` reads MercuryRT through a C# `MercuryCore` loaded from its state. When the C# core leaves
  `main` (stage 10), the claims lose both their C# half, which goes to the legacy branch with it, and the view their
  MercuryRT half reads through. Either the state classes stay on `main` as a reader with no machine behind them, or
  the view becomes a reader driven by the committed layout listings of §3.7. That choice is stage 9's to propose.
- **The corpus.** The verdicts are recorded (§12.2). The goldens add, per ROM, the frame of the verdict, the serial
  bytes and per-frame hashes, from C#. The baseline's C# columns then stand as the record of what C# said.
- **The traces** of §3.7 for the synthetic programs (`MercuryRtMachineTests`' busy, interrupts and colour programs on
  eleven boards, the random programs), the corpus and the four games. O1 shows which matter most: the lock-step
  synthetic programs caught what neither the games nor the corpus did.
- **The palette expectations.** The TCRF rows carry their own expected strings; the palette-number rows and the
  default take theirs from C#'s `CompatibilityPalettes`, which would need freezing as data when the C# core moves.
- **The legacy differential project** pins the core ABI, the state format and the input script, and now also the four
  test extensions the rig calls (`mercuryrt_core_step`, `_tick`, `_stop`, `_serial`).

## 13. The audit of the referee's candidates (2026-10-15)

*Decided 2026-10-14:* the defect candidates of `Mercury_Referee.md` §2 are audited before stage 9, so that the goldens C#
records are recorded after the fixes. Q4's rule holds (§5): a defect is fixed in C# Mercury and MercuryRT together, and
the two engines stay identical. This section is the audit's record, step by step, in the shape of `Moon_Native.md`
§3.8–§3.12.

### 13.0 The split, written before any code

**The evidence, and what is not evidence.** The referee page found each candidate by reading two other implementations,
the MiSTer core's RTL and Mesen 2. Those readings are leads. Nothing in this section rests on them, and neither tree was
opened for it. A defect is proven by one of two things, and a fix is graded by the same:

- **a test ROM's own source**, which states the rule it checks and, for mooneye, the M-cycle at which each access is
  expected. blargg's sources ship with the corpus. mooneye's do not ship with the release the corpus was taken from
  (`mts-20240926-1737-443f6e1`), so the test suite's repository was cloned beside it, MIT-licensed, at `31510e1`
  (2026-07-14), into the scratch directory (`~/.cache/emusen/probe/mercury/sources/`);
- **a public document**: Pan Docs (`gbdev/pandocs` at `0191af0`, 2026-10-03), and Gekkio's *Game Boy: Complete Technical
  Reference* (`Gekkio/gb-ctr` at `5ce83a1`, 2026-08-16, CC BY-SA), whose instruction tables give every access its
  M-cycle. Both were cloned to the same place. Nothing from either is copied into the tree; each rule is cited by file.

A candidate is **proven** by a WiseMan test that fails on both engines, through `MercuryRig` or `ICore`, before the fix,
and passes after. It is **refuted** when its own test or the documents contradict it, and the test is kept. It is **out
of reach** when the only sources for the rule are the forbidden ones. It is **a question** when building it would change
something a player sees by choice rather than by correction, or would add a setting or a surface.

**The order.** The steps follow the subsystems, in order of how little interpretation stands between the test and the
claim. §2.1–§2.4 come first, as the referee page asks. §2.5 follows §2.4, because §2.4's experiment decides how many of
the thirteen instruction-timing ROMs are left for it.

| Step | Candidates | Proven or graded by | Reach |
|---|---|---|---|
| A, read-back | §2.1 unmapped I/O reads `$FF`; HDMA1–4 write-only (§2.1's last paragraph); the undocumented `$FF72`–`$FF77` | blargg `dmg_sound`/`cgb_sound` `01-registers` (the mask row at `01-registers.s:115`); mooneye `bits/unused_hwio-GS` and `misc/bits/unused_hwio-C`, whose sources list every address and mask; Pan Docs `CGB_Registers.md` (HDMA1–4 "write-only"; `$FF72`–`$FF75`), `Audio_details.md` (PCM12/PCM34) | reachable |
| B, timer | §2.2 the reload cycle's write arbitration | mooneye `timer/tima_write_reloading`, `timer/tma_write_reloading`; Pan Docs `Timer_Obscure_Behaviour.md`, "Timer overflow behavior", rules 1–3 | reachable; `rapid_toggle` is examined in the step, not assumed |
| C, the CPU's dispatch | §2.3 the vector chosen after the high push; §2.14 STOP's cycle count; §2.13 `EI` before a dispatch | `interrupts/ie_push`, whose comments state the rule round by round; §2.14 is Mercury against its own count, a unit test; §2.13 against GBCTR's interrupt and `EI` description | §2.3 and §2.14 reachable; §2.13 reachable only if GBCTR or Pan Docs states it |
| D, OAM DMA | §2.4 the start delay, then the experiment on the thirteen instruction-timing ROMs | `oam_dma_start`, `oam_dma_timing`, `oam_dma_restart`, whose headers give the M-cycle at which OAM closes; Pan Docs `OAM_DMA_Transfer.md` | reachable |
| E, access placement | §2.5 PUSH, RST, RET cc; §3's CALL | each `*_timing` ROM's header (e.g. `push_timing.s`: "M = 1: internal delay; M = 2: memory access for high byte"); GBCTR's per-instruction tables | reachable; after D |
| F, cartridge boards | §2.8: the RAM bank masked by the RAM's size; MBC1M; MBC3's clock; MBC5's RAM enable | `emulator-only/mbc1/ram_64kb` and `multicart_rom_8Mb`, verified by their author on a genuine MBC1; Pan Docs `MBC1.md` (MBC1M, and how such a cartridge is identified), `MBC3.md`, `MBC5.md` | the first two reachable. **MBC5's eight-bit compare is contradicted by Pan Docs** (`MBC5.md:30`, "any value whose bottom 4 bits equal $A"), so it is examined as a refutation. **MBC3's four rules are out of reach**: Pan Docs states none of them, and the ROMs that test them (rtc3test, `mbc3_rtc_prelim`) are not in the corpus |
| G, the joypad interrupt | §2.7 | Pan Docs `Interrupt_Sources.md`, "INT $60" (a high-to-low change of P1 bits 0–3); a synthetic program | reachable; no corpus ROM tests it |
| H, colour registers | §2.11 the palette data ports in mode 3; SVBK's read-back | Pan Docs `Palettes.md:56` (writes ignored, reads `$FF`, in mode 3) and `:81` (the index still increments); `CGB_Registers.md`'s SVBK table (bits 2–0 read/write, 0 maps bank 1) | the first reachable from the document; SVBK's rests on reading a bit table, and is built only if the table is unambiguous |
| I, HDMA | §2.9: the double-speed charge; a cancelled transfer's `$FF55`; the destination's overflow; HDMA during HALT; a start in hblank; a start with the LCD off | Pan Docs `CGB_Registers.md`, "Transfer Timings" (8 µs a block at either speed: 16 fast M-cycles), "Reading from Register FF55" (bit 7 reads 1 with the blocks left), the overflow warning, the HALT paragraph | the first four reachable from the document; **a start in hblank** is left by Pan Docs as "should not be started" and **a start with the LCD off** is not described, so both are out of reach |
| J, sound | §2.10's rows | each blargg ROM's source, which states the rule and prints the failure Mercury printed | reachable row by row; `10-wave trigger while on` (§3) is examined, not assumed |
| K, STAT and the LCD's edges | §2.6 rules 1–6 | mooneye `ppu/*` sources and Pan Docs `STAT.md` | reachable; the most interpretation stands here, so it comes last |

**Questions for the tester, written now and not built** (each recommendation is argued in the step that meets it):

- **§2.12, LCDC bit 0 on a Game Boy.** Pan Docs (`LCDC.md:103`) says the background and window "become blank
  (white)", which is what Mercury draws; the referee's two readings show BGP's colour 0. The only test named for it,
  Mealybug's `m3_lcdc_bg_en_change`, changes the bit mid-line, which a per-line renderer cannot be graded on. It changes
  what a player sees in fade routines, and the public evidence sides with Mercury. *Recommendation:* leave it, record
  the dispute, and revisit with the per-dot renderer.
  *Decided 2026-10-15, as recommended:* LCDC bit 0 clear on a Game Boy stays blank (white), as Pan Docs has it. The
  referee's two readings are recorded as a dispute, to be revisited with a per-dot renderer. Nothing is built.
- **MBC5's RAM enable** (referee §2.8; §13.6). GBCTR says only `$0A` enables RAM; Pan Docs says any value whose low
  nibble is `$A`. Mercury follows Pan Docs, and no test ROM settles it. *Recommendation:* keep Pan Docs' rule, since the
  other could cost a game its saves, until a test ROM or a cartridge settles it.
  *Decided 2026-10-15, as recommended:* MBC5's RAM is enabled by any value whose low nibble is `$A`, as Pan Docs has
  it and as both engines already did. Nothing changes until a test ROM or a cartridge settles the dispute.
- **HDMA during HALT** (referee §2.9; §13.9). Pan Docs says an hblank transfer pauses while the CPU is halted. Building
  it needs the CPU to tell the bus it is halted, a new member of `ICpuBus`/`CpuBus` in both engines. *Recommendation:*
  add it (`ICpuBus.Halted`, set where HALT is entered and left), with Pan Docs as the oracle and MagenTests' HDMA case
  fetched as a witness if its licence allows.
  *Decided 2026-10-15, as recommended:* built in both engines, proven first by a claim failing on both (§14.2).
- **The PPU's line boundary** (§13.11.3). *Decided 2026-10-15:* its own piece of work, after stage 9. The goldens are
  recorded without it, and when it lands it is a deliberate re-baseline under §3.7's second rule.
- **§3's further leads** (the speed switch's duration, the serial port's transfer time, DMG `STOP`, the post-boot DIV)
  are outside §2 and are not part of this audit.

**What a step does.** For each step: a test that fails on both engines first, committed with the step; the fix in
`Mercury - GB-GBC/` and `MercuryRT - GB/` together; the crate's bench and C#'s cost before and after; the corpus
baseline re-recorded only for the ROMs that move, every move named; hand-made mutants of each fix in both engines, each
caught by a named oracle; and the four games' state traces, which must not move unless the move is explained.

### 13.1 Step A: what an I/O read answers

**The rule.** An I/O address that no register answers reads `$FF` and keeps nothing written to it. Before this step,
every `$FF00`–`$FF7F` address that no case claimed fell through to the `Io` array, which stored a write and returned it,
and read `$00` after power-on. That is the referee's §2.1. The evidence is the tests' own:

- mooneye `acceptance/bits/unused_hwio-GS` lists every unused address of a Game Boy and says "unused bits and unused IO
  all return 1s", verified on DMG, MGB, SGB and SGB2;
- its sibling `misc/bits/unused_hwio-C` lists the Color's, verified on CGB, AGB and AGS. It runs a Game Boy cartridge,
  so it describes the compatibility mode (`Mercury_Model.md` §4). It also fixes what the undocumented registers do
  there: `$FF72` and `$FF73` keep all eight bits, `$FF75` bits 4–6, `$FF74` reads `$FF`, and `$FF76`/`$FF77` read the
  silent channels' zero;
- blargg's `01-registers` ORs each sound register with a mask, and its row for `$FF27`–`$FF2F` is nine `$FF`s
  (`dmg_sound/source/01-registers.s:115`).

Pan Docs adds the colour mode's side, for which no corpus ROM exists: HDMA1–4 are write-only and read `$FF` after boot
(`CGB_Registers.md`; `Power_Up_Sequence.md`'s table); `$FF72`–`$FF74` are fully read/write in colour mode and `$FF75`
bits 4–6; RP (`$FF56`) has its read-enable and LED bits writable and reads `$3E` after boot, bit 1 being 1 while no
light arrives; OPRI keeps its one bit; PCM12 and PCM34 are the four channels' digital outputs, a nibble each
(`Audio_details.md`). IR.md's general statement, "like other Game Boy MMIO registers, unused bits read high", is the
rule the two OPRI and RP read-backs follow.

**The proof.** `MercuryAuditClaims` (`EmuSen.WiseMan/Cores/MercuryAuditTests.cs`) states nine claims, eleven cases with
the theories, run as `MercuryAuditTests` on C# and `MercuryRtAuditTests` on MercuryRT. Committed while they failed
(`753654eb`): **18 of 18 failed, on both engines**, the Game Boy's for example with "`$FF03` wrote 00 read 00".

**The fix,** the same in both engines (`MemoryBus.ReadIo` and `WriteIo`, `MemoryBus.Cgb.cs`; `bus.rs`):

- the fall-through reads `$FF` and drops the write, on every console and in both colour modes;
- `$FF46` gained its own read case, since DMA's read-back had been riding on the fall-through;
- colour mode answers `$FF4D`, `$FF4F`, `$FF55`, `$FF56`, `$FF68`–`$FF6C` and `$FF70`, then `$FF72`–`$FF77`;
- the compatibility mode answers `$FF4F`, `$FF68`, `$FF6A`, then `$FF72`, `$FF73`, `$FF75`–`$FF77`; the explicit
  list of registers it locked is gone, because locking is now what every other address does;
- `Apu.Pcm12`/`Pcm34` (`pcm12`/`pcm34`) read the channels' existing `Output`.

`$FF56`, `$FF6C` and `$FF72`–`$FF75` keep their bits in the `Io` array, which is already in the state, at their own
offsets. **The state format is unchanged**: no field was added, and the `Io` bytes of addresses nothing answers now stay
zero.

**The corpus** (re-recorded, `MercuryCorpusBaseline.tsv`):

| ROM | Before | After |
|---|---|---|
| `blargg/dmg_sound/rom_singles/01-registers` | Failed(2), every column | **Passed**, every column |
| `blargg/cgb_sound/rom_singles/01-registers` | Failed(2), every column | **Passed**, every column |
| `blargg/dmg_sound/dmg_sound`, `blargg/cgb_sound/cgb_sound` | Failed(1) | Failed(3): the combined ROMs now stop at their third test, `03-trigger` |
| `mooneye/acceptance/bits/unused_hwio-GS` | Failed | **Passed** under Auto and Game Boy on both engines; Failed under Game Boy Color, where its author's hardware fails it too |
| `mooneye/misc/bits/unused_hwio-C` | Failed | **Passed** under Game Boy Color on both engines; Failed under Auto and Game Boy, as on a DMG |

So each column gains three passes, 94 → 97 of 173, and the engines agree on every ROM in every column.

**Retired.** The referee page named `boot_hwio-dmgABCmgb` and `misc/bits/unused_hwio-C` as plausible. The second passed,
but only with the Color's undocumented registers added, which §2.1 did not name. The first still fails, as does
`misc/boot_hwio-C` under Game Boy Color (`Mercury_Model.md` §6.3 put it down to §2.1). Their tables also require the
post-boot values the boot ROM leaves, P1 `$CF`, IF `$E1` and DIV `$AD` among them, which is the referee's §3 and not
part of this audit.

**The four games.** `MercuryDefectTests.D3_real_games_version_5_states_resume_where_the_unmodified_build_went` runs each
game from the unmodified build's frame-3,000 state for 600 frames on both engines, with the bench's input, and compares
the result with that build's frame-3,600 state. It had been failing since state version 8 (§10), unnoticed because it
runs only with `EMUSEN_MERCURY_V5_STATES` set: a version-5 state carries no mixer, so the loaded frame-3,600 state's
mixer is not the game's. It was shown failing on the unmodified code at Kirby's byte 17,345, the second byte of the
mixer's first double, and now compares the state up to where version 8's mixer begins. With that repair it passes
after this step: the four games run exactly as the unmodified build ran them. A scratch trace of 3,600 frames from
boot on MercuryRT (`~/.cache/emusen/probe/mercury/audit/trace/`) gave the same full-state hash at every 300th frame
before and after: none of the four games writes an address nothing answers in that time.

**Cost** (p50 ms of 3,000 frames after 900, three interleaved rounds; the crate's `examples/frames`, and C#'s plain
frame through `MercuryCore` in Release, measured 2026-10-15):

| Game | MercuryRT before | after | C# before | after |
|---|---|---|---|---|
| Tetris | 0.4268, 0.4261, 0.4231 | 0.4394, 0.4330, 0.4350 | 0.6284, 0.6401, 0.6214 | 0.6280, 0.6224, 0.6293 |
| Kirby's Dream Land | 0.4836, 0.4806, 0.4789 | 0.4792, 0.4843, 0.4793 | 0.6703, 0.6762, 0.6676 | 0.6795, 0.6768, 0.6853 |
| Link's Awakening | 0.3978, 0.3967, 0.3977 | 0.3995, 0.3992, 0.3982 | 0.5542, 0.5570, 0.5576 | 0.5619, 0.5588, 0.5572 |
| Pokémon Yellow | 0.5298, 0.5285, 0.5314 | 0.5360, 0.5374, 0.5361 | 0.7171, 0.7177, 0.7158 | 0.7149, 0.7415, 0.7155 |

The change is not on the per-cycle path: only the I/O read and write's fall-through moved. Rust's Tetris is 2.4% slower
and Yellow 1.2%, outside their round-to-round spread; C# moves within 1.3% either way. A read of an unclaimed address
cannot cost a frame 2%, so this is the placement effect of §8.7.3 on a changed `match`, and is recorded rather than
explained further.

**Mutants.** Every mutant of this audit is one edit in a scratch copy of the tree at the step's commit
(`~/.cache/emusen/probe/mercury/audit/mut/`, `mut.py`): a Rust edit rebuilds the library and runs the crate's tests, a
C# edit rebuilds `EmuSen.dll`; either goes into a copy of WiseMan's output, which runs the audit's claims and, where
named, a wider filter; the edit and the output are restored after each. The unmutated control passes first. Step A's,
each made in C# (c) and in Rust (r) separately:

| # | Mutation | Caught by, in both engines |
|---|---|---|
| A1 | the fall-through reads the `Io` array again | `On_a_game_boy_every_unclaimed_io_address_reads_ff_and_keeps_no_write` |
| A2 | HDMA1 readable in colour mode | `On_a_game_boy_color_in_colour_mode_the_gaps_and_the_hdma_addresses_read_ff` |
| A3 | `$FF74` readable in the compatibility mode | the undocumented registers' compatibility case, and the compatibility gaps |
| A4 | `$FF75` stores bits 0–6 | **no read**: bits 0–3 read 1 whatever is stored. Caught only by the lock-step corpus run (`The_hardware_corpus_runs_identically_on_each_console`), where `unused_hwio-C` writes `$FF75` and the two engines' states then differ; a mutant made in both engines at once is equivalent through every read |
| A5 | RP reads light received | `The_infrared_port_reads_its_writable_bits_and_no_light` |
| A6 | PCM12's nibbles swapped | both `Pcm12_carries_pulse_twos_digital_output_in_its_high_nibble` cases |
| A7 | `$FF46`'s read case removed, so DMA's read-back falls to `$FF` | **only the corpus baseline**, three columns; so a claim was added, `The_dma_register_reads_back_the_page_written` |
| A8 | OPRI's unused bits read 0 | `Opri_reads_back_its_one_bit_in_colour_mode` |
| A9 | (C#) an unclaimed write still lands in `Io`, reads unchanged | seven lock-step cases: the random programs instruction by instruction, and the corpus rom by rom |

### 13.2 Step B: the timer's reload cycle

**The rule.** Pan Docs (`Timer_Obscure_Behaviour.md`, "Timer overflow behavior") names two machine cycles after TIMA
overflows: in cycle A TIMA reads `$00`; in cycle B TMA is copied into it and the interrupt is requested. Three rules
follow, and mooneye's `timer/tima_write_reloading` and `timer/tma_write_reloading` test the last two at exactly those
cycles, 27 to 30 and 28 to 31 `NOP`s after the timer is armed:

1. a TIMA write in cycle A cancels the reload and the interrupt;
2. a TIMA write in cycle B is lost: "TIMA will be equal to TMA at the end of the cycle anyway";
3. a TMA write in cycle B reaches TIMA in the same cycle.

Mercury had rule 1 (§7 of the referee page corroborates it) and not 2 or 3: its reload was a one-T-cycle event with
nothing after it, and a CPU write lands at the end of its machine cycle, after the reload, so it won.

**The proof.** Four claims (`MercuryAuditClaims`, the timer at 262,144 Hz one count from overflowing): rule 1, rules 2
and 3, and a control one cycle later. Committed failing (`bc6f6cf5`): rules 2 and 3 failed on both
engines, TIMA reading `$7F` where `$FE` was expected and `$FE` where `$7F` was; rule 1 and the control passed, as they
should.

**The fix.** `_timaReloadDelay` (`tima_reload_delay`) already counted the T-cycles to the reload. It now also counts
the rest of the machine cycle the reload landed in, as a negative number: −4 at the reload, back to 0 four T-cycles
later. A TIMA write while it is negative is dropped, and a TMA write while it is negative is copied into TIMA. Counting
in T-cycles rather than flagging "this machine cycle" keeps the rule right when a DIV or TAC write makes the overflow
fall on the first T-cycle of a machine cycle instead of the fourth.

- **The state format is unchanged.** The field and its width are the same. A state from an older build never holds a
  negative value, and means the same thing to this one. A state this build writes in the four T-cycles after a reload
  holds a negative value, which an older build reads as idle: the right answer, since its reload has already happened.
- **Cost.** The first version tested `< 0` before `> 0` on every T-cycle, and cost the crate 1.3–2.4% on three games
  (Tetris 0.4278 → 0.4394 ms p50, Kirby 0.4833 → 0.4896). The rare work now sits behind one `!= 0` test, as before the
  fix, in `StepTimaReload`/`step_tima_reload`:

| Game | MercuryRT step A | step B | C# step A | step B |
|---|---|---|---|---|
| Tetris | 0.4349, 0.4337, 0.4305 | 0.4352, 0.4267, 0.4354 | 0.6191, 0.6229, 0.6235 | 0.6270, 0.6251, 0.6306 |
| Kirby's Dream Land | 0.4838, 0.4824, 0.4809 | 0.4801, 0.4839, 0.4863 | 0.6705, 0.6803, 0.6669 | 0.6927, 0.6875, 0.6803 |
| Link's Awakening | 0.3958, 0.3973, 0.3949 | 0.3963, 0.3984, 0.4014 | 0.5768, 0.5555, 0.5574 | 0.5550, 0.5591, 0.5534 |
| Pokémon Yellow | 0.5311, 0.5380, 0.5366 | 0.5342, 0.5363, 0.5378 | 0.7026, 0.7092, 0.7264 | 0.7071, 0.7124, 0.7212 |

  MercuryRT is within its round-to-round spread on all four. C#'s Kirby is 2% slower and the rest within 1%; C# moved
  by as much between two runs of the same build in step A.

**The corpus.** `tima_write_reloading` and `tma_write_reloading` pass, in all six columns; nothing else moved. 99 of 173
in each column.

**`timer/rapid_toggle`, examined.** It still fails, and the referee page offered no mechanism. Run to its timer
interrupt on C#, it arrives with BC = `$FFD8`; its source expects `$FFD9`. So the sixteenth increment comes one loop
iteration, 17 machine cycles, late. Each iteration enables and disables the timer, and on a DMG a disable while the
selected bit is high counts once (Pan Docs, "Relation between Timer and Divider register"), which Mercury models; how
many iterations disable with bit 9 high depends on where in a machine cycle a TAC or DIV write takes effect, which
Mercury puts at the cycle's end. No public document states that phase (GBCTR has no timer chapter yet), so the ROM is
**out of reach** for this audit, and its mechanism is recorded as a lead, not a finding.

**The four games** ran as the unmodified build ran them: the real-games version-5 test passed, and MercuryRT's trace of
3,600 frames from boot is unchanged at every 300th frame. The whole `Mercury` filter, with the games and the corpus,
passed: 779 cases.

**Mutants**, each in C# and in Rust separately:

| # | Mutation | Caught by, in both engines |
|---|---|---|
| B1 | a TIMA write in the reload cycle lands | `A_tima_write_in_the_reload_cycle_is_ignored` |
| B2 | a TMA write in the reload cycle stays in TMA | `A_tma_write_in_the_reload_cycle_reaches_tima_as_well` |
| B3 | the window one T-cycle short | at first only the lock-step window program, `The_window_counts_its_lines_with_rendering_skipped_instruction_by_instruction`, which caught each engine's mutant against the other's code; a mutant made in both engines would have survived. A claim was added for the case it needs, a reload on a cycle's first T-cycle after a DIV write: `A_reload_that_lands_early_in_its_cycle_still_ignores_a_tima_write_at_the_cycles_end` |
| B4 | the window one T-cycle long | `After_the_reload_cycle_tima_and_tma_writes_are_ordinary` |
| B5 | the window counts away from zero and never closes | `After_the_reload_cycle_tima_and_tma_writes_are_ordinary` |

### 13.3 Step C: the interrupt dispatch, `EI` before it, and STOP's count

**The rules, and their sources.**

- **The vector is chosen between the two pushes** (referee §2.3). mooneye's `interrupts/ie_push` states it round by round
  in its source: with SP at `$0000` the high byte of PC is written to IE, and if that leaves nothing enabled "PC is set to
  `$0000` instead of the normal jump address", IF keeps its bit and IME stays clear (rounds 1–2); if it leaves a lower
  priority enabled, that one is dispatched and the higher stays requested (round 4); the low byte, written to IE with SP
  at `$0001`, is "too late to cancel" (round 3). Mercury chose the vector from IE & IF at the start of the step.
- **Where the dispatch's writes fall.** Pan Docs (`Interrupts.md`, "Interrupt handling"): two wait states, then PC pushed
  in two more machine cycles, then the jump in a fifth. Mercury pushed in the first two and settled the other three at
  the end. ie_push alone does not need this, since only the order of the two pushes and the choice matters to it; the
  document does, and the referee page recorded the placement as a three-way disagreement that Pan Docs settles.
- **`EI` before a dispatch** (referee §2.13). GBCTR gives `EI` as `IR, intr = fetch_cycle(addr=PC); ... IME = 1`: IME is
  set in the machine cycle after `EI`, which is the next instruction's fetch. After `EI; EI`, the second `EI`'s effect is
  spent before the dispatch that follows it, and the dispatch clears IME. Mercury kept `_imeScheduled` through the
  dispatch, so the handler's first step set IME, allowing a nested interrupt the hardware would not take. Pan Docs says
  only that the effect is "delayed by one instruction", which both readings satisfy. The game the referee page names,
  Batman – The Video Game, was not run: the document is the evidence here, and the game a witness for later.
- **STOP's count** (referee §2.14). `STOP` fetches its two bytes, eight T-cycles, and reported four; the frame's budget
  and `Cpu.Cycles` undercounted every STOP by four. No outside source is needed: the cycle harnesses' existing claim that
  "the bus advances by exactly what the step reports" fails for `$10`.

**The proof** (committed failing, `67e36e58`):

- `MercuryAuditClaims`: rounds 1 and 4 of ie_push, `EI; EI` with IME read in the handler, and STOP's step, failed on both
  engines; round 3, the control, passed on both.
- `MercuryCycleTimingTests` on C# and `tests/cycle_timing.rs` on the crate, the two harnesses that record each tick and
  access in order (§12.1): the dispatch's writes after 12 and 16 T-cycles, and `$10` in the "bus advances" theory,
  failed on both.

**The fix**, the same in both engines:

- `ServiceInterrupt` ticks the two wait states, writes the high byte, reads what is pending, writes the low byte and
  jumps to the vector, or to `$0000`; IME and the scheduled `EI` are both cleared. The bit it reports is the one chosen
  after the push, or −1 for a cancelled dispatch, so the frame loop's acknowledge clears the right IF bit, or none.
- What is pending is read through a new member of the CPU's bus, `ICpuBus.PendingInterrupts` (`CpuBus::pending_interrupts`),
  IE & IF's five live bits, rather than through a read of `$FFFF` and `$FF0F`, which would show in a debugger's trace as
  accesses the CPU never makes. It is the bus's only new surface; the two harnesses' recording buses answer it from their
  memory, and their existing dispatch claim now sets IE and IF there as well as passing them to the step.
- STOP returns eight.
- **The state format is unchanged.**

**The corpus.** `interrupts/ie_push` passes in all six columns; nothing else moved, so the write placement moved none
of the timing ROMs (`intr_timing`, `di_timing-GS`, `reti_intr_timing`, `halt_ime1_timing2-GS` and the rest pass as
before). 100 of 173 in each column.

**The four games** are unchanged: MercuryRT's trace from boot is identical at every 300th frame, and the real-games
version-5 test passes.

**The recorded platform digests moved, and why** (`MercuryRtPlatformTests`, §8.6, §9.4, §10). After 600 frames, two of
the four synthetic programs' state digests changed, and nothing else: every sound and picture digest is as recorded, and
the two engines agree on every digest.

- *colour*: one field differs, `Cpu.Cycles`, by 12: the program executes STOP three times in 600 frames, and each now
  counts four more T-cycles. That is §2.14 and nothing else.
- *interrupts*: one byte differs, `$C102`, the timer handler's counter, which reads `$8E` where it read `$B3`, six fewer
  timer handlers every 100 frames while the vblank and STAT counters are unchanged. The program takes timer interrupts
  every 256 T-cycles beside vblank and STAT. When vblank or STAT is requested in a timer dispatch's first three machine
  cycles, the late choice now serves it first, the timer stays requested through a long vblank handler, and its next
  overflow merges with the pending one. To separate the two halves of the fix, a scratch copy of the crate with the new
  write placement but the vector chosen at the start of the dispatch was run on the same program: its state after 600
  frames is byte for byte step B's. The placement moves nothing here; the late choice moves this.

The new digests were recorded for both (the `stateVersion7` column too, which is the same state with its version set back
and the mixer removed, so it moves with the state). They are linux-x64's; Windows and macOS meet them on CI's next run.

**Cost** (as in §13.1; from this step on other work shared the machine, load average about 3):

| Game | MercuryRT step B | step C | C# step B | step C |
|---|---|---|---|---|
| Tetris | 0.4394, 0.4364, 0.4306 | 0.4354, 0.4329 (and one round of 0.8868, a stall) | 0.6343, 0.6253, 0.6310 | 0.6562, 0.6217, 0.6345 |
| Kirby's Dream Land | 0.4908, 0.4914, 0.4917 | 0.4982, 0.4919, 0.4822 | 0.6904, 0.6930, 0.6926 | 0.7057, 0.6934, 0.6899 |
| Link's Awakening | 0.4021, 0.4024, 0.3995 | 0.3987, 0.4029, 0.3987 | 0.5610, 0.5679, 0.5636 | 0.5624, 0.5641, 0.5703 |
| Pokémon Yellow | 0.5302, 0.5284, 0.5348 | 0.5482, 0.5418, 0.5488 | 0.7210, 0.7187, 0.7160 | 0.7226, 0.7154, 0.7238 |

The dispatch runs a few times a frame and STOP never in these games, so the work cannot cost a measurable fraction of a
frame. Yellow on MercuryRT looked 3% slower; four more alternating pairs gave 0.5326–0.5390 before and 0.5350–0.5452
after, about 1%, which is within what placement and the shared machine move.

**Mutants**, each in C# and in Rust separately:

| # | Mutation | Caught by |
|---|---|---|
| C1 | the vector chosen before the high push | ie_push's rounds 1 and 4, both engines |
| C2 | a cancelled dispatch jumps to `$0040` | ie_push's round 1, both engines |
| C3 | the scheduled `EI` survives the dispatch | `Ei_ei_before_a_dispatch_leaves_ime_clear_in_the_handler`, both engines |
| C4 | the wait states after the push instead of before | C#: `MercuryCycleTimingTests.A_dispatch_pushes_on_its_third_and_fourth_machine_cycles`; Rust: the crate's `a_dispatch_pushes_on_its_third_and_fourth_machine_cycles`. Nothing through the rig sees it, as §12.1 found for every placement claim: the step's total is the same |
| C5 | STOP reports four | the rig's STOP claim on both engines, and each harness's "bus advances" theory |

### 13.4 Step D: OAM DMA's start delay, and the experiment on the instruction-timing ROMs

**The rule.** mooneye's `oam_dma_start` states the timing in its header: "M = 0: write to `$FF46` happens; M = 1: nothing
(OAM still accessible); M = 2: new DMA starts, OAM reads will return `$FF`", and for a restart, that the previous
transfer runs on, OAM closed, through M = 0 and M = 1. `oam_dma_timing` aligns one read to "exactly one cycle before the
OAM DMA end" and one "exactly after": counting its loop, OAM is closed at the 161st machine cycle after the write and
open at the 162nd. So the transfer owns OAM for machine cycles 2 to 161, 160 of them, as Pan Docs' "160 M-cycles"
(`OAM_DMA_Transfer.md`) requires. Mercury started the transfer inside the write and closed OAM from machine cycle 1 to
159.

**The proof.** Three claims from those headers (`Oam_stays_open_for_one_machine_cycle_after_the_dma_write`,
`Oam_is_closed_from_the_second_to_the_161st_machine_cycle_after_the_write`,
`A_restarted_dma_keeps_oam_closed_and_ends_160_cycles_after_its_start`), committed failing on both engines (`2fcd47f9`).

**The fix**, the same in both engines. The write sets a start delay of five T-cycles; the transfer starts on the fifth,
the first T-cycle of machine cycle 2, latching its page from `$FF46`; a transfer already running carries on until then.
Mercury lands a CPU access at the end of its machine cycle, after the rest of the machine has run it, so a transfer that
owns OAM through machine cycle 161 must still read as running at that cycle's end: its counter covers 161 machine cycles
(644 T-cycles) and copies a byte on the fourth T-cycle of each of cycles 2 to 161. The idle test at the top of the step
stays one comparison, of the counter and the delay together.

**State version 9.** The delay is machine state the walk cannot carry: a new field in `MemoryBus` would sit in the middle
of the reflection walk and move every older state's bytes. So it follows the mixer, as version 8's mixer followed the
walks (§10): `MemoryBus.WriteTail`/`ReadTail` in C#, `write_tail`/`read_tail` under the group `Bus` in Rust, with the
field `[SkipInState]` in C#. Both engines write 9 and read 5 to 9; a state before 9 loads with no start pending, since
its transfers started at the write. `A_version_8_state_loads_with_no_dma_start_pending` shows it on both engines: a
version-9 state saved one T-cycle after a DMA write, cut to version 8, loads with the delay zero and every other byte as
saved. The layout listing (`MercuryRtStateTests`), the real-games and version-5 tests' tail, and the platform test's
conversion to version 7 learned the four bytes; that conversion gives each of the four synthetic programs exactly its
step-C version-7 digest, so nothing but the version and the tail moved in their states, and their state digests were
re-recorded for that.

**Three older claims described the old model** and were brought to the documented one, in both engines' classes:
`Oam_dma_copies_a_page_into_sprite_memory` (161 machine cycles, not 160), `Oam_dma_is_still_running_half_way_through`
(not running one cycle after the write, still running at the 161st, done at the 162nd) and
`Oam_reads_ff_while_a_dma_is_running` (open at 1, closed from 2 to 161, open at 162).

**The corpus, and the experiment §2.4 asked for.** The referee page argued, without demonstrating it, that the start
delay rather than CPU access placement explained eight of the thirteen instruction-timing ROMs, and that the first
experiment was to add the delay and re-run them before touching the CPU. Done, with nothing else changed:

| ROM | Before | After |
|---|---|---|
| `oam_dma_start`, `oam_dma_timing`, `oam_dma_restart` | Failed | **Passed** |
| `call_timing`, `call_cc_timing`, `jp_timing`, `jp_cc_timing`, `ret_timing`, `reti_timing`, `add_sp_e_timing`, `ld_hl_sp_e_timing` | Failed | **Passed** |
| `push_timing`, `rst_timing`, `ret_cc_timing`, `call_timing2`, `call_cc_timing2` | Failed | Failed |

in all six columns, 111 of 173 in each. **The prediction held exactly**: the eight of the referee's right-hand column
pass, and the five of its left-hand column, the ones whose probed access Mercury places a machine cycle early, still
fail. §3.4's "internal cycles settled at the end of an instruction, seen through OAM DMA" was true of
those five and not of the other eight (the referee page's correction 1, §10, now settled). The five are step E's.

**The four games.** The real-games test passes (their states after 600 frames, up to the tail, are the unmodified
build's), and MercuryRT's trace from boot keeps the same PC, work RAM and VRAM at every 300th frame; its full-state hash
moved, by the version and the tail. The whole `Mercury` filter passes, 799 cases.

**Cost:**

| Game | MercuryRT step C | step D | C# step C | step D |
|---|---|---|---|---|
| Tetris | 0.4302, 0.4387, 0.4340 | 0.4404, 0.4425, 0.4409 | 0.6325, 0.6301, 0.6247 | 0.6366, 0.6289, 0.6304 |
| Kirby's Dream Land | 0.4900, 0.4997, 0.4894 | 0.4961, 0.4931, 0.4926 | 0.6944, 0.7006, 0.6840 | 0.6792, 0.6873, 0.6793 |
| Link's Awakening | 0.4023, 0.4036, 0.3966 | 0.4063, 0.4014, 0.4040 | 0.5614, 0.5590, 0.5570 | 0.5637, 0.5612, 0.5700 |
| Pokémon Yellow | 0.5365, 0.5402, 0.5398 | 0.5413, 0.5425, 0.5392 | 0.7231, 0.7181, 0.7136 | 0.7499, 0.7199, 0.7140 |

The idle test now reads two fields where it read one, on every T-cycle. MercuryRT's Tetris is 1.6–2.5% slower and the
rest within 1%; Tetris has moved by as much between runs of identical code in this audit (0.4231–0.4394 for the code of
steps A and B), so the step's own cost is under the noise of this shared machine and is not claimed. C# is unchanged.

**Mutants**, each in C# and in Rust separately, all caught by the step's claims on both engines:

| # | Mutation | Caught by |
|---|---|---|
| D1 | the transfer starts on the T-cycle after the write | all three timing claims and the version-8 claim |
| D2 | the transfer starts a machine cycle late | the same four |
| D3 | the counter ends with the last byte, so OAM opens at the 161st cycle | the timing and restart claims |
| D4 | a write stops the running transfer | the restart claim |
| D5 | a state before version 9 keeps the delay the machine held | the version-8 claim |

### 13.5 Step E: where PUSH, RST, CALL and RET cc touch the stack

**The rules.** Each of the five instruction-timing ROMs still failing after step D gives its instruction's machine
cycles in its header, and GBCTR's instruction tables give the same:

- `PUSH rr` and `RST n`: "M = 1: internal delay; M = 2: memory access for high byte; M = 3: memory access for low byte"
  (`push_timing.s`, `rst_timing.s`). GBCTR's `PUSH rr` shows why: SP is decremented in M2 before the first write.
- `CALL nn` and `CALL cc, nn` taken: the operand in M1–M2, "M = 3: internal delay", the pushes in M4–M5
  (`call_timing2.s`, `call_cc_timing2.s`).
- `RET cc` taken: "M = 1: internal delay; M = 2: PC pop: memory access for low byte; M = 3: … high byte; M = 4:
  internal delay" (`ret_cc_timing.s`); GBCTR's `RET cc` has the condition in M2 and the pops in M3–M4.

Mercury ticked each access in place and settled every internal cycle at the end of the instruction
(`Mercury_Cpu.md` §3.1), so all five put their stack accesses one machine cycle early. The referee page found the first
three (§2.5) and CALL (its §3, with Mesen alone, the RTL having kept the Z80's order); the test sources and GBCTR settle
all five without either.

**The proof.** `Each_stack_access_lands_on_the_machine_cycle_the_hardware_gives_it`, in both cycle harnesses (C#'s
`MercuryCycleTimingTests`, the crate's `tests/cycle_timing.rs`), five cases, the T-cycles ticked before each access:
committed failing on both engines (`7639db9c`). No claim through the rig can see placement, since a step's total is
unchanged (§12.1); the corpus and the harnesses are the oracles.

**The fix**, the same in both engines. A push begins with an internal machine cycle of its own, ticked in place
(`InternalCycle`/`internal_cycle`), which covers PUSH, RST and both CALLs at once, since all three push through it; the
dispatch of step C ticks its own two wait states and does not use it. A taken `RET cc` ticks its condition cycle before
its pops. Every instruction's total is unchanged. The harnesses' claim that internal cycles settle at the end of an
instruction kept its meaning and moved to `RET`, whose internal cycle does follow its pops (`ret_timing.s`: "M = 3:
internal delay").

**The corpus.** `push_timing`, `rst_timing`, `ret_cc_timing`, `call_timing2` and `call_cc_timing2` pass in all six
columns, and nothing else moved: 116 of 173 in each. With step D, **all thirteen** instruction-timing ROMs of §3.4 pass.

**The four games** are unchanged, at every 300th frame of MercuryRT's trace and in the real-games test, and so are the
four synthetic programs' recorded digests. The whole `Mercury` filter passes, 806 cases.

**Cost:**

| Game | MercuryRT step D | step E | C# step D | step E |
|---|---|---|---|---|
| Tetris | 0.4345, 0.4363, 0.4452 | 0.4360, 0.4495, 0.4419 | 0.6270, 0.6300, 0.6283 | 0.6400, 0.6297, 0.6523 |
| Kirby's Dream Land | 0.4938, 0.4920, 0.4968 | 0.4935, 0.4940, 0.4961 | 0.6858, 0.6877, 0.6898 | 0.6775, 0.6838, 0.6787 |
| Link's Awakening | 0.4058, 0.4040, 0.4050 | 0.4061, 0.4068, 0.4057 | 0.5919, 0.5656, 0.5660 | 0.5682, 0.5601, 0.5681 |
| Pokémon Yellow | 0.5388, 0.5439, 0.5335 | 0.5478, 0.5409, 0.5535 | 0.7207, 0.7207, 0.7254 | 0.7055, 0.7120, 0.7037 |

The same ticks are issued in a different order, a tick of four split from the end into the middle; every pair overlaps
within the spread of this shared machine, Yellow's MercuryRT by the least (about 1.5% apart at the medians).

**Mutants**, each in C# and in Rust separately: E1, a push's internal cycle settled at the end again; E2, RET cc's
condition cycle settled at the end again; E3, the push's internal cycle between its two writes. Each is caught by its
engine's harness, `Each_stack_access_lands_on_the_machine_cycle_the_hardware_gives_it` (C#: four cases for E1 and E3, one
for E2; Rust: the one test), and by nothing through the rig, as expected of placement. Without the harnesses the corpus
would be the only oracle, and CI has no corpus.

### 13.6 Step F: the cartridge boards

**MBC1's RAM bank on a chip of one bank** (referee §2.8, first item). mooneye's `emulator-only/mbc1/ram_64kb`, "verified
using a flash cartridge with a genuine MBC1B1 chip", expects that "since we have only one 8 kB bank, we always access it
regardless of what we do with `$4000` and `$6000`". GBCTR's MBC1 chapter gives the mechanism: the RAM's A0–A12 come
from the cartridge bus, and only A13–A14 from BANK2, so a 64 Kbit chip, which has no A13 or A14, ignores them. Mercury
computed an offset past the RAM's end in mode 1 and answered `$FF`, dropping the write.

**MBC1M** (referee §2.8, second item). mooneye's `multicart_rom_8Mb` gives the expected bank for every register value;
its source says a multicart "cannot be detected from the header alone" and needs a heuristic. Pan Docs (`MBC1.md`,
"MBC1M") gives both the wiring, BANK1 read as four bits for the bank (all five still for the 0→1 rule) and BANK2 on
bank bits 4–5, and the heuristic: such carts "can normally be identified by having a Nintendo copyright header in bank
`$10`". Mercury had no MBC1M.

**The proof.** Committed failing (`5ad71031`): `An_mbc1_with_one_ram_bank_answers_every_bank_number_with_it` and
`An_mbc1_multicart_puts_bank2_on_bank_bits_4_and_5` failed on both engines; the control,
`A_megabyte_mbc1_without_the_second_logo_banks_as_an_ordinary_mbc1`, passed.

**The fix**, the same in both engines:

- MBC1's RAM bank is taken modulo the number of 8 KiB banks the header gives (`Mbc1.RamBank`, `Mbc1::ram_bank`).
- MBC1M is decided once, from the image, by `Cartridge.IsMbc1Multicart` (`cartridge::is_mbc1_multicart`): an MBC1 image
  of exactly 1 MiB whose bank `$10` opens, at `$0104`, with the same 48 bytes as bank 0, provided those bytes are not all
  one value. Comparing with the cartridge's own logo keeps the logo's bytes out of the tree; the second condition keeps
  a blank synthetic image from matching itself, and has its own claim
  (`A_megabyte_mbc1_with_a_blank_logo_in_both_banks_is_ordinary`). Of the corpus's thirteen MBC1 images, only the
  multicart matches. The result is a shift kept beside the board (`_bank2Shift`, `bank2_shift`, 4 or 5), derived from
  the image and not in the state, as the header's other facts are.
- **The state format is unchanged.**

**Two of §2.8's items were not built.**

- **MBC5's RAM enable is a dispute between the two documents.** GBCTR (`mbc5.typ`, "based on my MBC5 research") says
  RAMG is a full eight-bit register and only `$0A` enables RAM, `$8A` included among the values that disable it; Pan
  Docs (`MBC5.md`) says actual MBCs enable on any value whose low nibble is `$A`. The referee's two readings side with
  GBCTR; Mercury follows Pan Docs. No test ROM in the corpus, or known to this audit, writes such a value. A game that
  enabled RAM with `$1A` would lose its saves under GBCTR's rule, which a player would notice, so this is a question for
  the tester (§13.0), with the recommendation to keep Pan Docs' rule until a test ROM settles it.
- **MBC3's clock rules** (seconds resetting the sub-second count, the registers' widths, carrying only at 59/59/23): Pan
  Docs states none of them, GBCTR's MBC3 chapter is five lines long, and the ROMs that test them (rtc3test,
  `mbc3_rtc_prelim`) are not in the corpus. **Out of reach.**

MBC3's and MBC5's own RAM banks use the same bounds check MBC1 had. The wiring argument would apply to them, but no test
ROM or document covers a bank number beyond those boards' chips, so they were left as they are and recorded as an
argued candidate.

**The corpus.** `emulator-only/mbc1/ram_64kb` and `multicart_rom_8Mb` pass in all six columns; nothing else moved: 118
of 173 in each, and all 28 `emulator-only` ROMs pass. **The four games** are unchanged (Kirby's Dream Land and Link's
Awakening are MBC1 boards, the latter with 8 KiB of RAM): MercuryRT's trace and the real-games test. The whole `Mercury`
filter passes, 814 cases.

**Cost.** MBC1's banks now shift by a field rather than a constant on every ROM read through the board.

| Game | MercuryRT step E | step F | C# step E | step F |
|---|---|---|---|---|
| Tetris (no MBC) | 0.4468, 0.4425, 0.4411 | 0.4451, 0.4502, 0.4441 | 0.6571, 0.6378, 0.6404 | 0.6324, 0.6349, 0.6367 |
| Kirby's Dream Land (MBC1) | 0.4974, 0.5008, 0.5357 | 0.5061, 0.5089, 0.5225 | 0.6773, 0.6837, 0.7132 | 0.6791, 0.6965, 0.6966 |
| Link's Awakening (MBC1) | 0.4301, 0.4194, 0.4096 | 0.4247, 0.4061, 0.4099 | 0.6058, 0.5834, 0.6208 | 0.5715, 0.5699, 0.5766 |
| Pokémon Yellow (MBC5) | 0.5544, 0.5495, 0.5620 | 0.5523, 0.5442, 0.5637 | 0.7114, 0.7166, 0.7141 | 0.7322, 0.7150, 0.7204 |

The machine was busier than in earlier steps (round-to-round spread up to 7%), and every pair overlaps; Kirby's
MercuryRT medians are 0.5008 and 0.5089, 1.6% apart, the largest difference, and within the spread.

**Mutants**, each in C# and in Rust separately:

| # | Mutation | Caught by, in both engines |
|---|---|---|
| F1 | the RAM bank not taken modulo the chip | `An_mbc1_with_one_ram_bank_answers_every_bank_number_with_it` |
| F2 | no image is ever taken for a multicart | `An_mbc1_multicart_puts_bank2_on_bank_bits_4_and_5` |
| F3 | BANK1 keeps all five bits on a multicart | at first **nothing**: the claim's case BANK2 = 1, BANK1 = `$10` gives bank `$10` either way. The claim gained BANK2 = 0, BANK1 = `$10`, which is bank 0 on an MBC1M (the second row of `multicart_rom_8Mb`'s table) and bank `$10` under the mutant; caught after |
| F4 | a blank logo counts as a logo | `A_megabyte_mbc1_with_a_blank_logo_in_both_banks_is_ordinary` |
| F5 | the logo compared at bank `$20` instead of `$10` | the multicart claim |

### 13.7 Step G: the joypad interrupt

**The rule.** Pan Docs (`Interrupt_Sources.md`, "INT $60 — Joypad interrupt"): it is requested "when any of P1 bits 0–3
change from High to Low", which happens when a button is pressed while its half is selected. A select write that
pulls a held button's line low is the same edge on the same lines, and is treated as one. Mercury declared the
interrupt and never requested it (`Mercury_Native.md` §6.2 argued it; the referee's §2.7 names Double Dragon 3 as the
game the references were fixed for). No corpus ROM tests it.

**The proof.** Four claims (`MercuryAuditClaims`, a program spinning with interrupts off): a press on the selected half,
and a select write that selects a held button, request it; a press on the other half and a release do not. The two
requests failed on both engines, committed failing (`606801b9`); the two controls passed.

**The fix**, the same in both engines: P1's four lines are read before and after each change that can move them, a
button set through the frontend (`MemoryBus.SetButton`, which `MercuryCore.SetButton` now calls; the Rust machine's
`set_buttons`) and a write to `$FF00`, and a line that fell requests the interrupt. No state is added: the lines are a
function of the select bits and the buttons. A button reaches either engine at a frame's edge, as it did before; the
request lands there too.

**The corpus** did not move. **The four games** behave as before: MercuryRT's trace from boot keeps the same PC, work
RAM and VRAM at every 300th frame, while the full-state hash moved at every one. The difference is IF bit 4: each game
polls P1 with the bench's Start or A held, which now requests the interrupt, and none of them enables it (the IE values
each game wrote in 3,600 frames: Tetris `$00`, `$01`, `$08`, `$09`; Kirby `$00`, `$05`; Link's Awakening `$00`, `$01`,
`$03`; Yellow `$00`, `$0C`, `$0D`, `$0F`; no joypad dispatch in any). Tetris, followed instruction by instruction for 400
frames on step F's crate and this one, runs the same PC and A throughout and differs first in IF. The real-games test,
which compares with the unmodified build's states, therefore masks IF bit 4 on both sides; with that, it passes.

**Retired.** §6.2's "a game that sleeps in STOP or HALT waiting for a button ... waits forever" is half answered: HALT
with IE bit 4 now wakes on a press. DMG `STOP` is still not a stop, which is the referee's §3 and outside this audit.

**Cost.** A `$FF00` write now reads the pad twice more; games poll it a few times a frame.

| Game | MercuryRT step F | step G | C# step F | step G |
|---|---|---|---|---|
| Tetris | 0.4447, 0.4424, 0.4442 | 0.4378, 0.4371, 0.4370 | 0.6406, 0.6299, 0.6264 | 0.6370, 0.6275, 0.6613 |
| Kirby's Dream Land | 0.5005, 0.4985, 0.4956 | 0.4925, 0.4951, 0.4965 | 0.6832, 0.6831, 0.6825 | 0.6842, 0.6884, 0.6803 |
| Link's Awakening | 0.4050, 0.4033, 0.4025 | 0.4045, 0.4028, 0.4030 | 0.5667, 0.7320, 0.5622 | 0.5620, 0.5591, 0.5656 |
| Pokémon Yellow | 0.5442, 0.5522, 0.5495 | 0.5468, 0.5400, 0.5520 | 0.7079, 0.7039, 0.7124 | 0.7192, 0.7090, 0.7058 |

No change beyond the spread (Tetris's MercuryRT moved 1.5% faster, which is placement).

**Mutants**, each in C# and in Rust separately, all caught on both engines: G1, a select write requests nothing
(`Selecting_the_half_of_a_held_button_requests_the_joypad_interrupt`); G2, any change of a line requests it
(`Releasing_a_button_requests_nothing`); G3, a press requests nothing
(`Pressing_a_selected_button_requests_the_joypad_interrupt`); G4, line 3 not watched (the selecting claim, whose held
button is Start, line 3).

### 13.8 Step H: the colour registers

**The rules** (referee §2.11), from Pan Docs alone; no corpus ROM tests either.

- `Palettes.md`: "the CRAM data registers are inaccessible when the PPU is reading from CRAM, that is, during Mode 3:
  writes are ignored, and reads return `$FF`", and the index "is automatically incremented ... after each write to
  this register, even if the write fails due to CRAM being inaccessible". Mercury read and wrote BCPD and OCPD in every
  mode.
- `CGB_Registers.md`: SVBK's bits 2–0 are read/write, and a written 0 "maps bank 1 instead"; its post-boot table gives
  SVBK as `$F8`, which is a 0 read back. Mercury stored 1 for a written 0, so it read `$F9`.

**The proof.** `The_palette_data_ports_are_closed_while_the_ppu_draws` (a theory over BCPS/BCPD and OCPS/OCPD) and
`Svbk_reads_back_a_written_zero_and_still_maps_bank_one`, committed failing on both engines (`a5bb10c7`).

**The fix**, the same in both engines: the two data ports answer `$FF` and drop a write while the LCD is on and the
PPU in mode 3, and the index still steps; SVBK keeps the written three bits, and the work RAM window maps 0 to bank 1
(`MemoryBus.MappedWramBank`, which the debugger's physical addresses now use too; `wram_offset` in Rust). The power-on
value stays 1, as the post-boot values are the referee's §3 and not part of this audit, so SVBK reads `$F9` until a
game writes it. `Svbk_banks_the_upper_wram_window_and_treats_zero_as_one`, a stage-8 claim, asserted the stored bank
was 1 after a written 0, and now asserts the mapped bank. **The state format is unchanged**; an older state never holds
0, and means the same.

**The corpus** did not move. **The four games**, Pokémon Yellow the colour one among them, are unchanged in MercuryRT's
trace (the full state hash too) and in the real-games test. The whole `Mercury` filter passes, 828 cases.

**Cost.** A work RAM access in the banked window now tests SVBK for 0. The machine was loaded by other work during this
bench, and several runs on both sides stalled to about twice their time (Tetris 0.904 before, Kirby 0.992 and Yellow
1.032 after, on MercuryRT; Yellow 1.213 before on C#), so only the unstalled runs are compared:

| Game | MercuryRT step G | step H | C# step G | step H |
|---|---|---|---|---|
| Tetris | 0.4511, 0.4615 | 0.4406, 0.4347, 0.4536 | 0.6341, 0.6281, 0.6641 | 0.6287, 0.6366, 0.6428 |
| Kirby's Dream Land | 0.5013, 0.4933, 0.5124 | 0.4922, 0.5100 | 0.6836, 0.6956, 0.6981 | 0.6819, 0.6758, 0.7226 |
| Link's Awakening | 0.4050, 0.4029, 0.4083 | 0.4040, 0.4054, 0.4101 | 0.5639, 0.5702, 0.5719 | 0.5595, 0.5597, 0.5875 |
| Pokémon Yellow | 0.5420, 0.5659 | 0.5562, 0.5485 | 0.7050, 0.7745 | 0.7095, 0.7230, 0.7470 |

No difference stands out from the spread; this step's number is the weakest of the audit and is recorded as such.

**Mutants**, each in C# and in Rust separately, all caught on both engines by the step's two claims: H1, BCPD readable
in mode 3; H2, OCPD writable in mode 3; H3, a refused write leaves the index where it was; H4, SVBK stores a written 0
as 1; H5, SVBK 0 maps bank 0.

### 13.9 Step I: HDMA

**The rules** (referee §2.9), from Pan Docs' `CGB_Registers.md` (HDMA5); no corpus ROM uses HDMA.

1. **Cost in double speed.** "In both Normal Speed and Double Speed Mode it takes about 8 μs to transfer a block of $10
   bytes. That is, 8 M-cycles in Normal Speed Mode, and 16 'fast' M-cycles in Double Speed Mode." Mercury charged 16 CPU
   T-cycles a block in double speed, four fast M-cycles; the hardware's 16 fast M-cycles are 64. The referee page found
   `Mercury_Cgb.md` §4.1 inconsistent with its own premise; Pan Docs shows the premise was wrong as well.
2. **`$FF55` after a stop.** "Reading from FF55 will return how many $10 blocks remained (minus 1) in the lower 7 bits,
   but Bit 7 will be read as 1." Mercury read `$FF`. The referee page listed this as a dispute in which "Pan Docs may
   differ from both"; this Pan Docs (2026-10-03) agrees with the referee's readings, so it is settled here.
3. **The destination's overflow.** "If the transfer's destination address overflows, the transfer stops prematurely."
   Mercury wrapped the destination within VRAM and carried on. What the registers read after such a stop is, in Pan
   Docs' words, still to be investigated; Mercury reads it as finished, `$FF`.

**The proof.** Three claims, committed failing on both engines (`1632df86`); the normal-speed cost was the control.

**The fix**, the same in both engines: a block charges 32 CPU T-cycles in normal speed and 64 in double; a stop keeps
`HdmaBlocksLeft` and clears `HdmaIsHBlankDriven`, so "blocks owed and not hblank-driven" is a stopped transfer, which
reads back with bit 7 set and is never advanced (a general-purpose transfer finishes inside the write, so that pair
meant nothing before); and a block that leaves the destination at 0 ends the transfer. **The state format is
unchanged**: an older state never holds a stopped transfer's blocks, and means the same. Two stage-8 claims stated the
old rules and were brought to Pan Docs' (`A_general_purpose_hdma_costs_twice_the_cycles_in_double_speed`, 4 × 64;
`Clearing_bit_seven_cancels_a_running_hblank_hdma`, `$82`). `Mercury_Cgb.md` §4 and §4.1 are corrected in place (the
referee page's correction 2, §10).

**Not built.**

- **HDMA during HALT** (referee §2.9's last rule). Pan Docs states it: "upon halting the CPU, the transfer will also be
  halted and will be resumed only when the CPU resumes execution", citing a test ROM (alloncm's MagenTests) that is not
  in the corpus. Mercury's bus moves hblank blocks from inside the PPU's tick and does not know the CPU is halted; the
  CPU would have to tell it, a new member of the CPU's bus surface in both engines. That is put to the tester (§13.0)
  rather than added.
- **A transfer started in hblank, and one started with the LCD off.** Pan Docs says only that an hblank transfer
  "should not be started (write to FF55) during a HBlank period", and says nothing of the LCD being off. **Out of
  reach.**

**The corpus** did not move, and **the four games** are unchanged (MercuryRT's trace, the real-games test). The
*colour* synthetic program's state digest moved, in one field: `Cpu.Cycles`, 230,496 lower after 600 frames. The
program runs a general-purpose and an hblank transfer every frame in double speed and otherwise sits in HALT, so every
block now stalls the CPU 48 T-cycles longer and the CPU counts that many fewer of its own; sound and picture are as
recorded, and both engines agree. Its state and version-7 digests were re-recorded. The whole `Mercury` filter passes,
836 cases.

**Cost.** Nothing changed on a path the four games take every frame (none of them runs HDMA in the bench):

| Game | MercuryRT step H | step I | C# step H | step I |
|---|---|---|---|---|
| Tetris | 0.4446, 0.4527, 0.4402 | 0.4395, 0.4533, 0.4424 | 0.6321, 0.6525, 0.6355 | 0.6341, 0.6532, 0.6398 |
| Kirby's Dream Land | 0.5032, 0.5070, 0.4986 | 0.5180, 0.5065 (and a 0.9391 stall) | 0.6945, 0.7039, 0.6911 | 0.7742, 0.6994, 0.6897 |
| Link's Awakening | 0.4290, 0.4134, 0.4064 | 0.4255, 0.4184, 0.4062 | 0.5932, 0.5860, 0.5705 | 0.6245, 0.5901, 0.5641 |
| Pokémon Yellow | 0.5658, 0.5539, 0.5530 | 0.5752, 0.5558, 0.5615 | 0.7447, 0.7299, 0.7193 | 0.7503, 0.7013, 0.7155 |

**Mutants**, each in C# and in Rust separately, all caught on both engines: I1, a double-speed block charges 16 (the
cost theory's double-speed case); I2, a stop forgets the blocks owed; I3, a stopped transfer reads without bit 7; I5, a
stopped transfer still moves at hblank (all three by `A_stopped_hblank_hdma_reads_back_the_blocks_it_still_owed`); I4,
the destination wraps and the transfer goes on (`A_transfer_that_runs_off_the_end_of_vram_stops`).

### 13.10 Step J: the sound quirks blargg tests

The referee's §2.10 rows are taken one rule at a time, each graded by its blargg ROM and by Pan Docs'
`Audio_details.md`, "Obscure Behavior", which states the same rules in words. The RTL and Mesen readings the referee
page compared are SameBoy's algorithm (its §0), and are not used.

#### 13.10.1 The length counter's first half

**The rules** (`03-trigger.s`; Pan Docs, the "extra length clocking" and trigger bullets): when the frame sequencer's
next step is one that does not clock length, enabling length in NRx4 (it was disabled, it is now enabled, the counter
is not zero) clocks the counter once, and if that empties it without a trigger the channel is disabled; and a trigger
that reloads an empty counter while length is enabled loads the maximum less one. `CGB_02` is not defined in either
suite's source, so both expect the later rule, in which the length must be enabled now as well. Mercury's sequencer
counter holds the next step to run, so "the next step does not clock length" is an odd `SequencerStep`.

**The proof.** Three claims, six cases with the theory's second-half controls, committed failing on both engines
(`4790cb4f`).

**The fix**, the same in both engines and for all four channels: `LengthCounter.Enable` (`enable`) clocks on the
enabling write in the first half and reports whether that emptied it; the NRx4 writers share `WriteLengthEnable`
(`write_length_enable`), which also sets the counter to the maximum less one ahead of a trigger that would reload an
empty enabled counter, so the channel's own trigger leaves it there. **The state format is unchanged.**

**The corpus.** `03-trigger` passes on both suites in all six columns, every subtest of it, the enable-in-first-half
(#3) and trigger cases (#6–#12) alike; the combined `dmg_sound` and `cgb_sound` now stop at `05-sweep details`,
Failed(5) for Failed(3). 120 of 173 in each column. **The four games** and all four synthetic programs' digests, sound
included, did not move. The whole `Mercury` filter passes, 848 cases.

**Mutants**, each in C# and in Rust separately: J1, no extra clock; J2, the first half taken as the even step; J3, no
reload to the maximum less one; J4, an emptied counter leaves the channel on: each caught on both engines by the
step's claims. J5, enabling an enabled length clocks it too, was caught at first only by the corpus (`03-trigger` #4),
which CI does not have; `Only_enabling_a_disabled_length_clocks_it` was added and catches it on both engines. The
step's cost was not measured separately: the change touches only NRx4 writes, a few a frame.

#### 13.10.2 The length counters across a power cycle

**The rules.** `08-len ctr during power.s`: "On CGB, length counters are reset when powered up. On DMG, they are
unaffected, and not clocked." `11-regs after power.s`, DMG: "While powered off, writes to NR41 are NOT ignored"; CGB:
"Powering off should clear NR41". Pan Docs (`Audio_Registers.md`, NR52) makes every register read-only while the APU
is off "except NR52 ... and the length timers (in NRx1) on monochrome models". `Mercury_Apu.md` §4 had kept the CGB
rule on both consoles and said where to start if a DMG test ROM ever failed on it; two did.

**The proof.** Two theories over a Game Boy and a colour cartridge, committed failing (`a6684d54`): the DMG half of
the first (a length written while off is kept, NR11's duty bits are not) and the CGB half of the second (a power cycle
clears the counters) failed on both engines; the other halves passed.

**The fix**, the same in both engines. The APU learns the console (`Apu.CgbHardware`, `cgb_hardware`, set by the bus,
not in the state): on a DMG a write to NR11, NR21, NR31 or NR41 while the APU is off loads the length counter and
nothing else; on a CGB powering up clears the four counters. **The state format is unchanged.** `Mercury_Apu.md` §4 is
revised (the referee page's correction 5, first half).

**The corpus.** Each suite now passes where its own console runs it:

| ROM | Auto | Game Boy | Game Boy Color |
|---|---|---|---|
| `dmg_sound` `08-len ctr during power` | Failed(1) → **Passed** | Failed(1) → **Passed** | Failed(1) |
| `dmg_sound` `11-regs after power` | Failed(4) → **Passed** | Failed(4) → **Passed** | Failed(4) |
| `cgb_sound` `11-regs after power` | Failed(4) → **Passed** | Failed(4) | Failed(4) → **Passed** |
| `cgb_sound` `08-len ctr during power` | Passed | **Passed → Failed(1)** | Passed |

on both engines. The one verdict that went from passing to failing is the colour suite's test run on the Game Boy: its
header says a DMG keeps its counters where the CGB clears them, and the expected checksum is the CGB's, so a DMG fails
it; it passed before only because Mercury gave the DMG the CGB's rule. (`Auto` runs `cgb_sound` on the Color, since its
header asks for colour.) Each `dmg_sound` ROM run on the Color still fails for the converse reason. Passes: 123 under
Auto, 121 under Game Boy and Game Boy Color, the same in both engines.

**The four games**, the recorded digests and the whole `Mercury` filter (858 cases) are unchanged.

**Mutants**, each in C# and in Rust separately, all caught on both engines: J6, a DMG ignores lengths while off; J8, NR11
written while off loads its duty too (both by the first theory's DMG case); J7, a CGB keeps its counters through a power
cycle (the second theory's CGB case). Cost was not measured separately: only NR52 and writes while the APU is off
changed.

#### 13.10.3 Leaving the sweep's negate mode

**The rule** (`05-sweep details.s` #4–#6; Pan Docs): "clearing the sweep direction bit in NR10 after at least one sweep
calculation has been made using the subtraction mode since the last trigger causes the channel to be immediately
disabled". Mercury did not model it.

**The proof.** `Leaving_negate_mode_after_a_subtraction_silences_channel_one` failed on both engines, with
`Leaving_negate_mode_before_any_subtraction_keeps_channel_one` and `A_trigger_forgets_an_earlier_subtraction` as
controls (`004041c8`).

**The fix**, the same in both engines: channel 1 remembers that a calculation was made in negate mode since its last
trigger (`PulseChannel.SweepNegated`, `sweep_negated`; set in `ComputeSweep`, cleared by the trigger before its own
calculation), and an NR10 write that clears the direction bit while it is set disables the channel.

**State version 10.** The flag is machine state the walk cannot carry, so, as §13.4's delay did, it follows the tail:
both engines write 10 and read 5 to 10, a state before 10 loading with no subtraction made. `The_sweeps_negate_flag_lives_in_version_10s_tail`
shows it on both engines: a state saved after a subtraction keeps the rule after a load, and the same state cut to
version 9 loads without it. The version-8 claim of §13.4 now cuts five bytes. The four synthetic programs' state
digests moved by the version and the byte only (each one's version-7 form is unchanged) and were re-recorded.

**The corpus.** `05-sweep details` passes on both suites in all six columns. The combined `dmg_sound` and `cgb_sound`
now stop at their eighth test (`08-len ctr during power`) on the console they were not written for, and at their ninth
(`09-wave read while on`) on their own. Passes: 125 under Auto, 123 under Game Boy and Game Boy Color. **The four
games** run as before (MercuryRT's trace keeps the same PC, work RAM and VRAM; the real-games test passes); the whole
`Mercury` filter passes.

**Mutants**, each in C# and in Rust separately, all caught on both engines: J9, leaving negate mode never disables;
J11, the flag never set (both by the rule's claim and the version-10 claim); J10, a trigger keeps the flag
(`A_trigger_forgets_an_earlier_subtraction`); J12, the flag lost through the tail (C#: not written; Rust: not read;
the version-10 claim).

#### 13.10.4 The wave channel: out of reach

The remaining rows, `09-wave read while on`, `12-wave write while on` and `12-wave` (CGB) and `10-wave trigger while on`
(DMG), test what wave RAM answers while channel 3 plays and what a retrigger corrupts. Pan Docs states the rules in
words (`Audio_details.md`: a trigger leaves the first sample to the old buffer and plays from the second; a DMG
retrigger while the channel reads a byte rewrites the first bytes of wave RAM). Each ROM, though, grades them by a
checksum of 69 reads or rewrites, each taken two T-cycles later than the last, against the channel's fetch position
after a trigger. That position depends on the exact delay between the trigger and the first fetch, which neither Pan
Docs nor GBCTR gives, and which the ROMs encode only as a checksum. Finding it by trying delays until the checksum
matches would be fitting a model to a test, not deriving it from evidence, and the only sources that state it are the
references this audit does not use. **Out of reach**, with the rules recorded for when a document gives the timing.

### 13.11 Step K: STAT with the LCD off, and line 144's mode-2 interrupt

The referee's §2.6 names six rules; their ROMs' sources are the evidence, Pan Docs' `STAT.md` adding little beyond "the
PPU mode reports 0 when the PPU is disabled". Three rules are built here; the other three are recorded as one piece of
work for later (§13.11.3).

#### 13.11.1 The rules built

- **Rule 4, the comparison freezes with the LCD off** (`ppu/stat_lyc_onoff.s`, rounds 1–3): "the bit should be retained
  and not reset to 0"; "changing LYC should not have an effect, because the comparison clock is not running";
  "enabling the PPU starts the comparison clock again". Mercury compared `Ly == Lyc` live, with LY parked at 0.
- **Rule 5, no STAT interrupt with the LCD off** (the same rounds: a frozen comparison holding the line "should suppress
  the interrupt", and round 4 expects one when the comparison starts matching as the LCD comes on). Mercury dropped the
  line to low when the LCD went off and recomputed it on every STAT or LYC write, so an LYC write with the LCD off could
  raise the interrupt.
- **Rule 2, the mode-2 source at line 144.** `ppu/vblank_stat_intr-GS.s` (verified on DMG, MGB, SGB, SGB2; fails on the
  Color): "if bit 5 (mode 2 OAM interrupt) is set, an interrupt is also triggered at line 144 when vblank starts ...
  vblank and stat_m2_144 are triggered at the same time". Its sibling `misc/ppu/vblank_stat_intr-C.s` (verified on CGB,
  AGB, AGS) says that on the Color "stat_m2_144 is triggered one cycle before a vblank is". Mercury raised neither.

**The proof** (`a6c492ea`): the freeze, the silence with the LCD off and the DMG's line-144 interrupt failed on both
engines; the comparison that starts matching at LCD-on passed. That commit's Color case expected no interrupt at line
144, the reading of `-GS`'s "fail: CGB" before its `-C` sibling was read; the `-C` header corrected it, and the Color's
claim (`On_the_color_line_144s_mode_2_interrupt_comes_a_machine_cycle_before_vblank`) was written with its fix. Its
before-and-after is the corpus: `vblank_stat_intr-C` failed under Game Boy Color and now passes.

**The fix**, the same in both engines:

- the comparison is taken when the LCD goes off (`Ppu.LycWhileOff`, `lyc_while_off`) and answers STAT's bit 2 and the
  LYC source until the LCD comes back, when the live comparison resumes;
- the STAT line does not move while the LCD is off: it keeps its level, and nothing written then can raise it;
- on a DMG the mode-2 source counts for the update that starts line 144; on a CGB it is raised four T-cycles (a
  machine cycle) before line 144, from the dot loop's visible-line branch.

**State version 11.** The frozen comparison is machine state for as long as the LCD is off, so it joins the tail:
both engines write 11 and read 5 to 11, and a state before 11 loads with the bit clear
(`The_frozen_comparison_lives_in_version_11s_tail`). The four synthetic programs' state digests moved by the version and
the byte only, their version-7 forms unchanged.

**The corpus.** `ppu/vblank_stat_intr-GS` passes under Auto and Game Boy, and `misc/ppu/vblank_stat_intr-C` under Game
Boy Color, on both engines: 126 under Auto, 124 under Game Boy and under Game Boy Color. `stat_lyc_onoff` still fails:
its round 1 also expects STAT to read mode 0 right after the LCD comes on, which is rule 3.

**The four games** keep the same PC, work RAM and VRAM at every 300th frame of MercuryRT's trace, and the real-games
test passes.

**Mutants**, each in C# and in Rust separately: K1, the comparison runs with the LCD off; K3, no line-144 interrupt on
a DMG; K5, the frozen bit taken after LY is cleared (C#) or not carried by the tail (Rust): each caught on both engines
by the step's claims. Two survived the first claims and gained one each: K2, the STAT line moving with the LCD off,
which the frozen comparison's cases could not see because the line was already high (now
`A_source_enabled_while_the_lcd_is_off_raises_nothing`, an hblank source enabled while the LCD is parked in mode 0); and
K4 in Rust, the Color's interrupt a machine cycle too early, which the claim allowed because it only looked at the cycle
it should arrive in (it now also asserts nothing has arrived a T-cycle before).

#### 13.11.2 Retired

The referee page put `stat_lyc_onoff` down to rules 4 and 5. Both were needed and neither was enough: it waits on rule 3
too. Its `intr_2_0_timing` pass and three `intr_2_*` failures it read as the skew between the IRQ and STAT's mode bits;
that reading stands, untested, in §13.11.3.

#### 13.11.3 Not built: the line boundary at T-cycle resolution

Rules 1 (the mode-2 source as a pulse), 3 (no mode 2 on the first line after LCD-on), 6 (LY reading 0 early in line
153) and the `intr_2_*` skew are one change, not four. `ppu/lcdon_timing-GS.s` gives the table: line 0 "starts with
mode 0 and goes straight to mode 3", "is late by 2 T-cycles", and LY reads 1 four T-cycles before STAT reports line 1's
mode 2. Mercury moves LY, the mode and the STAT line together at dot 456. Building these means a line boundary in which
LY, the mode bits, the STAT sources and access blocking change on their own dots, which is the PPU's structure rather
than a rule inside it, and it would move every game's timing. It is recorded as the next piece of PPU work, to be
designed against the `lcdon_*`, `intr_2_*` and `stat_lyc_onoff` tables, and not built in this audit. Rule 3 alone was
considered and not built: the table shows mode 0 at LCD-on, but no source this audit may use says whether an enabled
mode-0 source raises the interrupt then, and building half the rule would choose that unproven.

### 13.12 The audit's cost, measured by instructions as well as time

From step H on the machine was shared with other work (load averages of 3 to 11), and wall-clock benches moved by
several percent between runs of identical code; some runs stalled to twice their time. So the whole audit was also
measured by the user-space instructions MercuryRT's bench retires (`perf_event_open`, scratch `tools/icount.py`), which
do not depend on load, for 3,900 frames of each game:

| Snapshot | Tetris | Pokémon Yellow |
|---|---|---|
| before the audit | 37,795,896,576 | 38,919,540,695 |
| step A | 37,795,692,935 | 38,919,044,363 |
| step B, first form (`< 0` before `> 0`) | 38,069,364,254 (+0.72%) | 39,192,775,825 |
| step B as committed | 37,795,697,868 | 38,919,033,278 |
| step C | 37,795,767,074 | 38,919,332,004 |
| step D, first form (a second field in the idle test) | 38,353,849,297 (**+1.48%**) | 39,477,083,752 |
| steps E–J | 38,300,425,160 to 38,358,967,638 | 39,451,837,904 to 39,497,762,079 |
| step K | 38,305,921,129 | 39,489,963,379 |
| **after the fold below** | **37,763,224,412 (−0.09%)** | **38,947,229,369 (+0.07%)** |

and Kirby's Dream Land +0.42%, Link's Awakening +0.12% over the audit. Step D's idle test, which read the start delay
beside the counter on every T-cycle, was the one real cost; it was folded (`dae00467`): a fresh write sets the counter
itself to cover the machine cycle before the transfer starts, so the idle test reads one field again, and the separate
delay is used only for a restart written while a transfer runs, which keeps the old transfer alive until then. The
claims, the corpus, the traces and every recorded digest were unchanged by the fold. The version-8 claim of §13.4 now
holds a restart's delay, since a fresh write no longer uses the tail; states written by this branch's earlier commits
with a fresh delay pending are not a population anyone holds and are not converted.

**Time is a different matter for MercuryRT after step K.** With the same instruction count as step J (to 0.01%), the
crate's bench ran 4–6% slower, interleaved run for run (Tetris 0.4475–0.4680 ms at step J against 0.4747–0.4786 after K,
on a loaded machine). Three variants that each removed a part of step K (the comparison in the STAT read, the frozen
bit's field, the console field and the dot check) did not recover it; marking the once-a-line `begin_scanline` cold and
not inlined, and the STAT line's update not inlined, recovered about half (Tetris 0.4552–0.4632, Yellow 0.5749–0.5988
against step J's 0.5648–0.5781). That is the placement effect §8.7.3 found, with the code doing the same work, and is
recorded as such: the attributes were kept, as true statements about those paths, and the remainder is left for a
measurement on a quiet machine before stage 9's goldens, through `ICore` as a player runs it. C#'s frame did not move
measurably across the audit.

### 13.13 What the audit settled, and what is left

**Verdicts, candidate by candidate:**

| Referee | Candidate | Verdict | Settled by |
|---|---|---|---|
| §2.1 | unmapped I/O reads `$FF`; HDMA1–4 write-only | proven, fixed | `01-registers` ×2, `unused_hwio-GS`, `unused_hwio-C` (with the Color's `$FF72`–`$FF77`, RP and OPRI) |
| §2.2 | the timer's reload cycle | proven, fixed | `tima_write_reloading`, `tma_write_reloading`; `rapid_toggle` out of reach |
| §2.3 | the vector chosen after the high push | proven, fixed | `ie_push`; the writes on M3/M4 by Pan Docs |
| §2.4 | OAM DMA's start delay | proven, fixed | the three `oam_dma_*` and the eight instruction-timing ROMs it predicted |
| §2.5, §3 CALL | PUSH, RST, RET cc, CALL, CALL cc placement | proven, fixed | the five remaining instruction-timing ROMs; GBCTR |
| §2.6 | STAT edges: rules 2, 4, 5 | proven, fixed | `vblank_stat_intr-GS` and `-C` |
| §2.6 | rules 1, 3, 6 and the `intr_2_*` skew | not built | one later change to the PPU's line boundary (§13.11.3) |
| §2.7 | the joypad interrupt | proven, fixed | Pan Docs |
| §2.8 | MBC1 RAM bank on a small chip; MBC1M | proven, fixed | `ram_64kb`, `multicart_rom_8Mb` |
| §2.8 | MBC5 RAM enable | a dispute (GBCTR against Pan Docs) | put to the tester |
| §2.8 | MBC3's four clock rules | out of reach | no document; the ROMs are not in the corpus |
| §2.9 | HDMA: double-speed cost, `$FF55` after a stop, the overflow stop | proven, fixed | Pan Docs |
| §2.9 | HDMA during HALT | proven by Pan Docs, not built | needs a new bus member; put to the tester |
| §2.9 | HDMA started in hblank or with the LCD off | out of reach | Pan Docs silent |
| §2.10 | the length quirks, the negate rule, the power rules | proven, fixed | `03-trigger`, `05-sweep details`, `08`, `11` |
| §2.10 | the wave rows (`09`, `10`, `12`) | out of reach | their timing exists only as checksums |
| §2.11 | palette ports in mode 3; SVBK read-back | proven, fixed | Pan Docs |
| §2.12 | LCDC bit 0 on a DMG | decided: a dispute, white kept | Pan Docs; the tester, 2026-10-15 |
| §2.13 | `EI; EI` before a dispatch | proven, fixed | GBCTR's `EI` |
| §2.14 | STOP's cycle count | proven, fixed | the cycle harnesses |

**The corpus**, from 94 passes in every column to **126 under Auto and 124 under Game Boy and Game Boy Color**, the
same in both engines: all thirteen instruction-timing ROMs, the three `oam_dma_*`, `ie_push`, both timer reload ROMs,
both `emulator-only` MBC1 failures, both `unused_hwio` ROMs, `vblank_stat_intr-GS` and `-C`, and nine blargg sound
ROMs on the consoles they were written for. One verdict went from passing to failing, `cgb_sound`'s `08` run on the
Game Boy, because a DMG now keeps its length counters as the ROM's header says it does (§13.10.2).

**The referee page's corrections (its §10)** settled here: 1 (§3.4's thirteen ROMs, §13.4), 2 (`Mercury_Cgb.md` §4.1,
§13.9), 5 (`Mercury_Apu.md` §4, §13.10.2; §3.4's wave read-back remains, out of reach), 6 (the joypad interrupt,
§13.7). Not settled: 3 (LCDC bit 0, decided as a dispute), 4 (`Mercury_Memory.md` §6's DMA-from-VRAM claim, untested),
7 (`Mercury_Cgb.md` §6 and §5), 8 (`Mercury_Ppu.md` §3, now true of Mercury: STAT no longer fires with the LCD off).

**The state format** went from version 8 to 11, each addition after the mixer, each read back to version 5:
9 a restart's OAM DMA delay, 10 channel 1's negate flag, 11 the frozen LY=LYC bit.

**No RTL line was opened.** Every rule rests on a test ROM's source or on Pan Docs or GBCTR; no emulator source was
read. **The four games** run as the unmodified build ran them throughout, the joypad's IF bit aside (§13.7).

**Questions for the tester** (§13.0): MBC5's RAM enable (keep Pan Docs' rule), and HDMA during HALT (add the bus
member). The PPU's line boundary is proposed as its own piece of work. *All three decided 2026-10-15 as recommended*
(§13.0); HDMA during HALT is built in §14.2.

**What changes for stage 9.** The goldens are recorded from C# after this audit, so they carry these rules. The state
version the goldens and the legacy differential project pin is 11, with its tail. The corpus baseline the goldens
extend is this one, 126/124/124. `MercuryAuditTests` is a further class of claims on both engines that, like
`MercuryCycleTimingTests`, stage 10 must keep when the C# core leaves `main`; the four placement claims of §13.5 and the
dispatch claim of §13.3 live only in the two cycle harnesses.

## 14. Stage 9: the goldens (2026-10-15)

*Decided 2026-10-15:* the audit's questions (§13.0, §13.13) are answered as recommended. MBC5's RAM enable keeps Pan
Docs' rule, and nothing changes. HDMA pauses during `HALT`, built in both engines with a new member on the CPU's bus
(§14.2). The PPU's line boundary (§13.11.3) is its own piece of work after this stage: the goldens are recorded without
it, and its arrival will be a deliberate re-baseline under §3.7's second rule. After the C# core leaves `main`, the
rig reads a state through a reader driven by the committed layout listings (§14.5), and that reader is held to C#'s
reading of every state the tests make for as long as both exist. *Decided 2026-10-16 by the tester:* MercuryRT is
no longer compared against C# Mercury, and the legacy differential was dropped (§14.7). This section is stage 9's
record, in the order the work was done.

### 14.1 The audit's open slowdown, measured again

**The question** (§13.12): after step K MercuryRT ran 4–6% slower with the same instructions, and marking the
once-a-line path cold had recovered about half. That was measured on a loaded machine. It was measured again here, on
the crate's bench (`examples/frames`, the p50 of 3,000 frames after 900) and through `ICore` (a scratch console app
holding `MercuryRtCore` as a frontend does, 900 frames of the bench's input, then 3,000 timed with the picture and the
sound taken), each run under a counter of the user-space instructions and cycles it retires (`perf_event_open`,
scratch `pcount.py`). Builds were snapshots of the crate at the commit before the audit (`aea4f182`, "pre"), at `461f03b7`
("head"), and with §14.2's fix ("hdma"), each with the crate's own alignment flags.

**The load.** The machine was shared with four other lines of work throughout. Rounds under a one-minute load of 10 to
48 gave instruction counts only: wall time and cycles moved by a factor of two with the load, since another process
on a core's second thread takes its cycles. A watcher then ran a round only when the load fell under 2; six such rounds
were taken at 22:04–22:31, with the load beside each run between 1.48 and 3.62. Later comparisons ran at 2 to 4, and are
given with their load.

**The loss stands, and it is not work.** At equal instructions (head against pre: Tetris −0.04%, Kirby's Dream Land
+0.45%, Link's Awakening +0.19%, Pokémon Yellow +0.12% over 3,900 frames), the medians of the six quiet rounds:

| Game | crate bench, pre → head | through `ICore`, pre → head |
|---|---|---|
| Tetris | 0.4299 → 0.4553 ms (+5.9%) | 0.4582 → 0.4707 (+2.7%) |
| Kirby's Dream Land | 0.4894 → 0.5141 (+5.0%) | 0.4974 → 0.5242 (+5.4%) |
| Link's Awakening | 0.4027 → 0.4207 (+4.5%) | 0.4153 → 0.4319 (+4.0%) |
| Pokémon Yellow | 0.5392 → 0.5753 (+6.7%) | 0.5612 → 0.5768 (+2.8%) |

The runs of each pair were within 1–2% of their median, apart from one stalled `ICore` run, and user cycles moved
with the time. So this is not noise.

**Finding the cause.**

- **Where.** An interrupt-style sampler of the bench (ptrace on the main thread, the pc mapped to the binary's
  symbols, scratch `rsample.py`) put the growth in `Cpu::execute` (about +15% in absolute time) and `jump_relative`
  (+12%), with `run_frame` +3% and the renderer −3%.
- **Size.** `execute` had grown from 103,025 bytes to 122,865 over the audit while the executed instructions did not.
  A bisection over the audit's commits gave its size at each: unchanged through step A, then +3.8 KB at B (the
  timer), +4.1 KB at D (OAM DMA), +3.6 KB at K, +5.2 KB at the fold and +3.2 KB with the cold attributes. The time
  drifted with it: 0–2% already at step A, whose `execute` was exactly the same size; about +2% at J; +4.5–8.3% at
  K; +7–10% at the fold. The fold removed 1.4% of the instructions and still made the frame slower.
- **Why.** `MemoryBus::tick` is `#[inline(always)]`, so the whole per-T-cycle step (PPU, APU, OAM DMA, DIV and the
  timer) is copied into every bus access in every opcode arm. A change to that step is multiplied across the copies;
  frontend-stall cycles rose by 180 million on Tetris against a gap of 640 million, while branch misses rose by 1.1
  million.
- **The experiments.** Loop alignment beside the crate's flags moved each game differently (`-align-loops=32`: Tetris
  +0.6%, Link's Awakening +6.4%), so it was not a fix. Taking `tick` out of line shrank `execute` to 8.5 KB but cost
  8.5% more instructions, and gave mixed results. It did make pre and head level with each other (Pokémon Yellow −1.0%
  against −1.1%, Tetris +7.0% against +5.5%), which locates the difference in the copies' layout rather than their
  work.

**The fix** (`d327880b`), in MercuryRT alone, since the C# core has no such copies:

- the PPU's dot loop tests whether the dot is one where something happens (the line's last machine cycle or its
  end, the end of the OAM scan, the end of drawing) and only then calls `dot_event`, which is the old body unchanged;
- a running OAM DMA's cycle is a call (`step_oam_dma_running`), and an idle one is the same single test as before.

The copies now carry an increment and three compares. `execute` is 85 KB, and the bench retires 1.3–1.7% fewer
instructions than either build (Tetris 37.27 billion over 3,900 frames, against pre's 37.80 and head's 37.78).
Measured four interleaved rounds each, at a load of 3.4–4.1:

| Game | crate bench, pre / head / fix | through `ICore`, pre / head / fix |
|---|---|---|
| Tetris | 0.4384 / 0.4713 / 0.4439 (+1.3%) | 0.4692 / 0.4850 / 0.4629 (−1.3%) |
| Kirby's Dream Land | 0.4935 / 0.5198 / 0.4942 (+0.1%) | 0.5133 / 0.5405 / 0.5109 (−0.5%) |
| Link's Awakening | 0.4088 / 0.4278 / 0.4118 (+0.7%) | 0.4273 / 0.4539 / 0.4322 (+1.2%) |
| Pokémon Yellow | 0.5495 / 0.5852 / 0.5553 (+1.0%) | 0.5765 / 0.5938 / 0.5673 (−1.6%) |

The bound left is ±1.6%, which is inside one round's spread. The four games' states at every 300th frame to 3,600 and
the colour program's state are byte for byte the same as before the fix, and every golden reproduces (§14.4).

**Why this does not generalise.** The fix makes the copies small. It does not make the layout of the copies that
remain any more stable, and a future change to the dot loop can move the frame again. This is §8.7.3's finding a
second time, now with a mechanism named. Two measures would do more: a smaller per-T-cycle step, or one copy, which
would need the per-access ticks restructured, and neither was tried. The Ryzen 7 7700X is the only machine measured.

### 14.2 HDMA during HALT

**The rule.** Pan Docs (`CGB_Registers.md`, HDMA5): "Upon halting the CPU ..., the transfer will also be halted and
will be resumed only when the CPU resumes execution." It cites a test ROM (alloncm's MagenTests) that is not in the
corpus and was not fetched; Pan Docs is the oracle. The reading built is the plainest one: a visible line that enters
mode 0 while the CPU is halted moves no block, and the blocks still owed move at the hblanks after the wake. Nothing in
the rule says that a block missed during `HALT` is made up at the wake, so none is.

**The proof** (`cd70f78e`). `An_hblank_hdma_moves_nothing_while_the_cpu_is_halted_and_resumes_after` arms a four-block
hblank transfer, enables only the vblank interrupt with IME clear and halts; at line 144 the CPU wakes. It failed on
both engines: `$FF55` read `$FF` at the wake, all four blocks having moved during the first four lines of the halt.
Its control, `An_hblank_hdma_beside_a_running_cpu_moves_a_block_each_line`, the same program with a `NOP` for the
`HALT`, passed on both.

**The fix** (`9173647f`), the same in both engines. The CPU's bus gains one member, `ICpuBus.SetHalted(bool)` and
`CpuBus::set_halted`. The CPU calls it when `HALT` puts it to sleep and when a pending interrupt wakes it, before
anything else runs in that step; the HALT bug's path never sleeps and never calls it. The bus keeps the flag and
`OnHBlankStarted` (`on_h_blank_started`) moves no block while it is set. The flag is a copy of `Cpu.Halted`, so it is
not in the state: a load sets it from the CPU's field, and a reset clears it. **The state format stays at version
11.** The two cycle harnesses (C#'s `MercuryCycleTimingTests`, the crate's `tests/cycle_timing.rs`) record the call
as an `H:` entry and gained `Halt_and_its_wake_are_told_to_the_bus`, which fixes where in the step each call falls.
`A_state_saved_in_halt_keeps_the_hblank_hdma_paused_after_a_load` holds the load's half on both engines.

**What moved.** The corpus has no HDMA, and its baseline did not move. The four games' MercuryRT states every 300
frames to frame 3,600, with the bench's input, are byte for byte those of the unmodified build. One recorded digest
moved: the colour synthetic program halts between vblanks with an hblank transfer armed, so its blocks no longer move.
Its state at frame 600 differs in four fields: `Cpu.Cycles`, 478,432 higher, since the CPU is no longer stalled by the
blocks; `Bus.HdmaSource` and `Bus.HdmaDestination`, which stay where each frame's write puts them; and two bytes of
VRAM. Its sound and picture digests did not move, and both engines agree on the new state digest, which is
re-recorded with this reason. Because the program no longer exercises an hblank transfer at all, its goldens add a
variant whose main loop spins instead of halting (`colour-running`, §14.4), and the debugger's armed-against-plain
test gained the halting program, which is the one that sees the observed step pass the call on.

**Cost.** Nothing on the per-cycle path moved for a game: the check is in the once-a-line hblank call and in `HALT`, and
the four games' instruction counts are equal to within 0.0001% (Tetris 37,779,889,419 against 37,779,879,495). In the
six quiet rounds of §14.1, hdma differs from head by −1.4 to +1.3 points in the crate's bench, in both directions,
and by +0.3 to +2.8 points through `ICore`; at equal instructions that is the layout effect §14.1 describes, and §14.1's
fix, built on hdma, measures it away. C#'s frame was not
re-timed: the change there is the same two calls and one test.

**Mutants**, each in C# and in Rust separately, run 2026-10-16 by §13.1's method (one edit in a scratch copy of the
fix's tree, the library or `EmuSen.dll` rebuilt into a copy of WiseMan's output; the claims of §13, the two cycle
harnesses and the debugger's armed-against-plain test, and on a survivor the lock-step, state, platform and model
classes besides). The unmutated control passed, 138 cases.

| # | Mutation | Caught by |
|---|---|---|
| H1 | an hblank block moves while the CPU is halted | the HALT claim and its state-load claim, and the halting colour program armed against plain, on both engines |
| H2 | `HALT` does not tell the bus | the HALT claim and the armed colour program; in C# the cycle harness, in Rust the crate's `halt_and_its_wake_are_told_to_the_bus` |
| H3 | the wake does not tell the bus | the HALT claim and its state-load claim; the cycle harness of the engine mutated |
| H4 | a load does not set the bus's flag from the CPU's | `A_state_saved_in_halt_keeps_the_hblank_hdma_paused_after_a_load` only |
| H5 | (Rust) the observed step does not pass `HALT` on | `Synthetic_programs_with_every_table_armed_are_the_programs_run_plain` only, with the colour program added for it |
| H6 | a reset keeps the bus's flag | **survived**, in both engines, through the wide filter as well: **equivalent**. A load of a game builds a new bus, whose flag is already clear, and nothing else resets a bus; the line states what a reset means and is kept |

H4 and H5 each have one oracle, and each was written for it.

### 14.3 The synthetic programs, out of the classes that leave

The programs the lock-step tests ran were fields of `MercuryRtMachineTests`, `MercuryRtPlatformTests` and
`MercuryRtStateTests`, classes that compare MercuryRT with C# Mercury and so leave `main` with it. They now live in
`EmuSen.WiseMan/Fixtures/MercuryPrograms.cs`: the busy and interrupts programs, the colour program halting or spinning,
four channels, the colour priority, sweep, bank-zero and MBC3-latch programs, the HALT-bug and OAM DMA step programs,
the random programs and the eleven boards. The old classes call them there, so no program has two copies.
`SyntheticGbRom` computed the header checksum through C#'s `Cartridge`; it now has its own, so a synthetic image is
built from the repository alone, with no Mercury class, which §3.7 assumed of it.

### 14.4 The golden traces

**The form.** A trace is a text file, one per run, under `EmuSen.WiseMan/Cores/MercuryGoldens/`. Its header names the
run, the ROM's full SHA-256, the model, the input script with a fingerprint of the masks it presses, whether rendering
is skipped, and the engine it was recorded from. Each frame of a frame run is a line `frame state picture sound`: the
first eight bytes of SHA-256 over the engine's save state, over the RGBA frame, and over the drained samples as
little-endian bytes, which is the pacebench "state sha" of §3.7. A value equal to the one on the line before is written
`=`, so a still picture or a silent frame costs a character. An instruction run (a step program) is too long to keep
a hash per instruction, so each instruction's state hash is folded into a chain, `chain = SHA-256(chain ‖ state)`, and
the chain is written every 100 instructions and at the end: a difference names a hundred-instruction window. A corpus
run ends at its verdict, by `MercuryHardwareTestRomTests`' rule, and adds its transcript: the verdict and its frame,
the serial bytes, and blargg's cart-RAM text where the ROM reports through it.

**The catalogue** (`MercuryGoldenTests`):

| Runs | Files | What | Rows |
|---|---|---|---|
| `programs/busy-*`, `programs/interrupts-*` | 22 | §12.5's two lock-step programs on each of the eleven boards, 600 frames | 13,200 |
| the interrupts program on a forced console, and with rendering skipped | 3 | a Game Boy cartridge on the Color, a colour one on the Game Boy, 600 frames; 300 frames skipped (O1's rendering-skip pair) | 1,500 |
| `programs/colour`, `colour-running`, `colour-skipped` | 3 | HDMA and double speed, halting and spinning (§14.2), the spinning one also skipped | 1,500 |
| `programs/four-channels` and the small lock-step programs | 8 | the platform digests' sound program; colour priority, the sweep's second check, bank zero on four boards, MBC3's latch | 650 |
| `steps/halt-bug`, `steps/window`, `steps/oam-dma` | 3 | the HALT bug and `EI`, the window counted with rendering skipped (O1), OAM DMA byte by byte: 40, 20,000 and 3,000 instructions | 231 |
| `steps/random-*` | 176 | sixteen random programs on each board, 5,000 instructions or to the first illegal opcode | 7,675 |
| `corpus/…` | 173 | every corpus ROM under Auto, to its verdict or 3,600 frames, with its transcript | 36,120 |
| `games/<sha>` | 4 | §3.3's four games, 3,600 frames with the bench's script | 14,400 |

392 files and 2.9 MB of text (3.8 MB on disk), of which the corpus is 1.4 MB; a third of the frame columns are `=`.
The C# states kept while recording are 198 files, 9 MB. Only the corpus's Auto column is traced frame by frame: the
other two models' verdicts are in the baseline (§12.2), and their traces would have tripled the corpus's share for two
columns that differ from Auto on a handful of ROMs. The corpus traces' verdicts reproduce that baseline's Auto column,
126 passed, 42 failed and 5 without a verdict.

**Recording** is from the C# engine only, as §3.7's first rule requires, and only on purpose:

```
EMUSEN_MERCURY_GOLDEN_RECORD=Mercury EMUSEN_MERCURYRT_ROMS=<games> EMUSEN_MERCURYRT_CORPUS=<corpus> \
  dotnet test EmuSen.WiseMan --filter "FullyQualifiedName~MercuryGoldenTests"
```

The variable names the engine to record from; the other engine's cases then report that they were not graded. While
recording, every frame run keeps the C# state of every 300th frame under `~/.cache/emusen/mercury-goldens/<run>/`
(`EMUSEN_MERCURY_GOLDEN_STATES` moves it), outside the tree and so never committed.

**Grading.** Each case runs its program on the engine through `MercuryRig`, so through `ICore` as a frontend holds it,
builds the same trace, and compares it with the committed one: the header, then line by line, then the transcript. The
first difference names the frame and the part ("frame 300: the state differs (recorded …, now …)"). When the machine
running the test holds the recorded interval states, two more things happen. A failure is bisected: the engine is run
to the first recorded state at or after the failing frame, and the fields its state differs in are named through the
layout listing (§14.5). And a frame run that passes is graded a second time from the recorded C# state of an early
interval frame (300 for a 600-frame run, 900 for a game), loaded into a fresh machine, which is §3.3's "from states"
comparison with the state from the other engine. C# Mercury is graded as well while it is on `main`; its rows leave
with it.

**Re-baselining.** After the move, a deliberate change to MercuryRT's behaviour is recorded from MercuryRT with
`EMUSEN_MERCURY_GOLDEN_RECORD=MercuryRt`, and the commit that replaces the traces states the reason (§3.7's second
rule). The `recorded` line of each trace then says which engine each file speaks for, so the history of the directory
is the list of intended departures from the C# oracle. The PPU's line boundary will be the first.

**The result** (2026-10-15). Every trace was reproduced on both engines: the 36 frame runs and 3 step programs, the
176 random programs, the 173 corpus ROMs and the four games, 1,061 of the filter's 1,062 cases on the first grading run
(the one failure was the palette file of §14.6, since mended). The 27 synthetic programs of 600 frames were also reproduced
from their kept C# state of frame 300, and the four games from that of frame 900, on both engines. The grading run of
`MercuryGoldenTests` and `MercuryStateLayoutTests` alone takes six minutes, most of it the C# rows; with the C# rows gone
it will take about two.

**One deliberately broken trace, caught.** `A_deliberately_broken_trace_is_caught` changes one hex digit of frame 300 of
`programs/interrupts-03-02-80` in memory, once each in the state, picture and sound column, and grades MercuryRT: each
fails with "frame 300: the state differs", "the picture differs" or "the sound differs". The same was done once to the
committed file itself: frame 300 of `programs/interrupts-03-03-80.trace` was changed from `D2D19B97AE91C3B1` to `…C3B0` and the
synthetic-program theory run: that run failed on both engines with "frame 300: the state differs (recorded
`D2D19B97AE91C3B0`, now `D2D19B97AE91C3B1`)", and the bisection found the recorded C# state of frame 300 equal to the
engine's field for field. That is the correct reading, since the trace and not the engine had moved, and the message
now says so in words. The file was restored.

### 14.5 The layout listings, and the reader they drive

**The listings.** `state_layout` prints one line per field, `offset length type path`, and the layout depends on the
cartridge (the board's fields, the size of cartridge RAM) and on the console (VRAM and WRAM). So the listings are one
per board, console and RAM size, named `<board>-<gb|gbc>-ram<bytes>.layout`, under `MercuryGoldens/layouts/`, written
verbatim from MercuryRT's `state_layout` (`EMUSEN_MERCURY_LAYOUT_RECORD=1` writes one that a test needed and found
missing). Eighteen are committed, 100 KB: the five boards on each console with the RAM sizes the programs' eleven boards
use (none with 0 and 8 KB, MBC1 with 8 and 32 KB, MBC2's 512 bytes, MBC3 with 32 KB, MBC5 with none, 32 and 128 KB).
`Every_board_on_every_console_has_a_committed_listing` runs each of the eleven boards under each model, so a board the
programs use cannot lack one. `MercuryStateLayoutTests` holds each committed listing to MercuryRT's `state_layout` and to C#'s
reflection walk for a cartridge of that shape, and `MercuryStateLayout.StateVersion` to the version both engines write:
a listing is recorded again when the format moves. When the walk leaves `main`, these files are the format's
specification (§3.7's third rule).

**The reader** (`EmuSen.WiseMan/Fixtures/MercuryStateLayout.cs`) needs no machine. It picks the listing for a state
from three facts the state and the image give: the board from the header's cartridge type, the console from the state's
own console byte, and the RAM size from the state's length, since nothing else in a layout of one board and console
varies. It then reads or writes any scalar field by its path and any byte array in place. The rig gains `Field`,
`Flag`, `FieldByte`, `FieldBytes`, `SetField` and `SetFieldByte`: a read is a state saved through `ICore` and read
through the listing; an edit is the same state rewritten and loaded back, which is how a field edit already reached
MercuryRT (§12.1), now without a C# object between.

**Both readers agree.** While C# Mercury is on `main`, every state read through a listing is also read by C#'s classes,
and `MercuryStateAgreement` compares the two field by field: each scalar by value, each array byte for byte, the
mixer's doubles by their bits. On the C# engine the state is the core's own; on MercuryRT it is loaded into the rig's
view, which the rig's view reads also go through, so every pull of the view is checked as well. The whole `Mercury` filter, 1,062 cases, ran with the check in place, including every converted claim that reads
`rig.Bus` on MercuryRT (which pulls the view at each read), and no field of any state disagreed. The view
is retired from the rig at stage 10, once this check has stood; the claims written against the listing need nothing
else.

**What uses it.** `MercuryAuditTests` (§13) now reads and edits the machine only through the listing and the bus: its
34 reads of `rig.Cpu` and `rig.Bus` became paths such as `Cpu.PC`, `Bus.Apu.Pulse1.Length.Counter` and
`Bus._stallCycles`, `PpuMode` became the state's numbers, and its version constant is the listings'. Its only Mercury
type left is `GbModel`, the setting's enum, which MercuryRT's shim uses too. The palette claims of §14.6 read colour RAM
the same way.

### 14.6 The palettes, frozen as data

The TCRF rows of `MercuryModelTests` carry their expected palettes. The palette-number rows and the default palette
took theirs from C#'s `CompatibilityPalettes`, so they would have lost their oracle with it (§12.5). The 94 palettes are
now `MercuryGoldens/palettes.tsv`: per palette number, the title checksum and fourth letter of a Nintendo header that
selects it (the first the search over checksums and letters finds) and the three palettes in RGB888, as TCRF writes
them. `The_frozen_palettes_are_csharps_table` writes the file from C#'s table (`EMUSEN_MERCURY_PALETTES_RECORD=1`) and
otherwise holds the file to it; the palette-number rows and the default compare each engine's colour RAM with the file;
and `Every_frozen_palette_is_what_mercuryrt_loads` builds each row's header and reads MercuryRT's colour RAM through the
listing, with no C# table consulted. The palette number a header selects is still asserted against C#'s lookup in the
palette-number rows; that assertion leaves with C#, and the palettes it selects stay held.

### 14.7 The legacy differential, dropped

*Decided 2026-10-16 by the tester:* MercuryRT is no longer compared against C# Mercury; the legacy differential was
dropped. MercuryRT's oracle from here on is the goldens of §14.4, recorded from C# Mercury before this decision,
together with the hardware test corpus and its baseline (§12.2) and the conformance kit. Nothing more is spent on
comparisons of the two engines.

What had been built, for the record. `EmuSen.MercuryLegacy/` was §3.7's third layer: a test project outside
`EmuSen.sln` that referenced C# Mercury and loaded a MercuryRT library by path through EmuSen's generic v1 host. It
pinned the core ABI's major, the six `mercuryrt_core_*` extensions it called, state version 11, the bench script's
fingerprint against the committed game goldens and `ICpuBus`'s members, then compared the two engines frame by frame
and instruction by instruction. It had been run once, on 2026-10-15, before the decision reached this work: 17 of 17
in 5.4 minutes, with the synthetic programs on every board, 765,483 random instructions, the 173 corpus ROMs over
36,120 frames and the four games over 3,600 frames each identical. The project was removed from the tree with the
decision. Of §3.7's three layers, the first (the traces) and the second (the kept states, used for bisection) stand,
and the third does not exist.

### 14.8 What survives the C# core's move, and what stage 10 needs

**On `main` without C# Mercury**, the following stand as they are: the goldens and their grader on MercuryRT; the
layout listings and the reader; `MercuryAuditTests`' claims, through the rig's MercuryRT half; the frozen palettes
with `Every_frozen_palette_is_what_mercuryrt_loads`; `MercuryPrograms` and `SyntheticGbRom`; the corpus baseline's
MercuryRT columns; and the crate's own tests. **`MercuryCycleTimingTests`** cannot drive MercuryRT, which takes no bus
from C#, and leaves with the C# core; its claims are all in the crate's `tests/cycle_timing.rs`, case for case: the
four single traces, the eleven opcodes of the bus-advance theory as one loop, the interrupt's cost and its push
cycles, the five stack placements of §13.5, the halted step, and since §14.2 the HALT-and-wake trace. Nothing in it is
held only in C#.

**What stage 10 needs from the tester**, beyond the decision itself (Q5):

- **The rig's C# half and view.** `CsharpRig`, the view and `MercuryStateAgreement` go with the C# core; `MercuryRig`
  keeps its MercuryRT half and the listing-driven field access. The other `Mercury*Claims` classes still read
  `rig.Bus` and `rig.Cpu` through the view (on about 200 lines of the PPU, CGB, APU, CPU, core, serial and access-blocking
  classes) and must be converted as `MercuryAuditTests` was, or leave with C#: a choice for that stage.
- **What MercuryRT's shim takes from C# Mercury.** `MercuryRtCore` uses `GbModel`, `Cartridge.FromImage` for the header
  and its refusals, `Apu.HighPassSeed` for the mixer's coefficients, and a hidden `MercuryCore` as the debugger's
  mirror (§8.5). These are the shim's dependencies, not the tests', and have to move into the shim or a small shared
  file before the C# core can leave; the debugger's mirror is the largest, and the layout reader is a candidate for
  replacing it.
- **The lock-step classes** (`MercuryRtMachineTests`, `MercuryRtStateTests`, `MercuryRtPlatformTests`' C# half,
  `MercuryRtCorpusTests`, the model lock-step cases) leave with C#, their comparisons now held by the goldens alone
  (§14.7).

### 14.9 What is not done

- **The corpus's Game Boy and Game Boy Color columns** are graded by verdict alone (§12.2), not frame by frame.
- **The PPU's line boundary** (§13.11.3), decided as the next piece of work; it will re-baseline the goldens.
- **MagenTests' HDMA case** was not fetched as a witness for §14.2; Pan Docs is its only oracle.
- **The handheld**, as in every stage since 7, and the alignment there (§8.7.5).

## 15. Stage 10: MercuryRT alone (2026-10-16)

*Decided 2026-10-16 by the tester: MercuryRT is the default and the C# core is deleted.* C# cores are not kept as
oracles once a Rust core replaces them; the C# core leaves with the tests that need it, and MercuryRTs

## 15. Stage 10: MercuryRT alone (2026-10-16)

*Decided 2026-10-16 by the tester: MercuryRT is the default and the C# core is deleted.* C# cores are not kept as
oracles once a Rust core replaces them: the C# core leaves with the tests that need it, and MercuryRT's oracles are
the goldens of §14.4, the hardware test corpus with its baseline (§12.2) and the conformance kit. This section is
stage 10's record, in the order the work was done.

### 15.1 The default

The Game Boy's engine row now names MercuryRT (Rust) first, and so as its default; Mercury (C#) stayed a choice until
the C# core was deleted (§15.3). Before the switch was committed, MercuryRT was graded on the oracles that outlive the
C# core, measured on 2026-10-16 from the build of the switch:

- **The corpus**, the 173 blargg and mooneye ROMs under each of the three models, held to the baseline's MercuryRT
  columns: no verdict moved, so Auto stands at 126 passed, 42 failed and 5 without a verdict, as recorded.
- **The goldens**: the 36 frame runs and 3 step programs, the 176 random programs, the 173 corpus traces and the four
  games (3,600 frames each with the bench's input) all reproduced, and the deliberately broken trace was caught in
  each of its three columns.
