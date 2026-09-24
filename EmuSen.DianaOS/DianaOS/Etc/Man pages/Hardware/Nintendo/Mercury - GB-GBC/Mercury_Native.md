# Mercury_Native — a plan to port Mercury to Rust

*Written 2026-09-23, before any code.* On 2026-09-23 the user decided that every EmuSen core will be ported to Rust once
Mars is finished (`EmuSen_Stack.md` §2.1), and Mercury is the one to plan now. This page is the plan. It follows the
method `Mars_Native.md` §5 used for MarsRT, and it does what §2.1 asks of every 2D port: it states its own speed
prediction before the work begins, because the 2D cores' remaining costs were never traced to the language.

The page keeps two things apart. **Measured** means a number taken on 2026-09-23 on the desktop described in §1.1.
**Argued** means reasoning from the code or from Mars's evidence, with no measurement behind it. Each prediction is
numbered (P1–P6) so that it can be retired later, as `Mars_Native.md` retired P1–P10.

## 1. What the port is for

### 1.1 The baseline (measured)

**How it was measured.** The harness has no Mercury bench, so a throwaway console app was built against the
worktree's `EmuSen.csproj`, in Release on .NET 10.0.111, on the Ryzen 7 7700X. It does the following:

1. It loads a scratch copy of the ROM with the battery disabled.
2. It boots through 900 frames, pressing Start and A alternately every 45 frames.
3. It times 3,000 frames flat out, one `RunFrame` each, draining the audio and taking the picture after every frame.

Every run is its own process. Games are interleaved within a round, and there are three rounds. The machine was shared
with another agent's work, and round 3 was noisier than rounds 1–2 (Link's Awakening's p99 was 1.89 ms). So the
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

1. **Consistency with the stack.** The user's decision is that every core is Rust, and Mercury is the smallest core
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

**The name is MercuryRT, and the folder is `EmuSen/Cores/Nintendo/MercuryRT - GB/`.** The user decided both. The
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

*Decided by the user on 2026-09-23:* every C# core will eventually move to its own legacy branch. The comparisons of
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

**3. A live differential on the legacy branch, built on demand.** The legacy branch keeps C# Mercury and its WiseMan
tests. A test project there loads MercuryRT's library built from `main`, by a path, as `MarsRtPublish.targets`'
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
2026-09-22, with agents working in parallel, on a crate now about 35,000 lines. Mercury's machine is about 3,550 lines of
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
| 6 | Debugger hooks: breakpoints, stepping, coverage and the write log as pushed tables and drained logs; `MercuryRtDebugTarget` over the state view | `MercuryDebugTargetTests` on both engines; games with every table armed identical to plain runs | 1–2 days |
| 7 | Platforms and CI: the second crate in the workflow's matrix; `MarsRtPublish.targets` and `EmuSen.csproj`'s single-library build generalised to a list of crates | Crate tests on four platforms; WiseMan's synthetic systems per platform | hours to half a day |
| 8 | Parity: the rest of WiseMan's Mercury tests through `MercuryRig`; mutants for every stage not yet covered | §3.5, §3.6 | a day |
| 9 | Goldens: traces and interval states recorded from C# (§3.7), MercuryRT graded against them on `main`, and the legacy-branch differential project written and run once | Every golden reproduced by MercuryRT; one deliberately broken trace caught | half a day |
| 10 | The C# core's move to its legacy branch (Q5) | A decision, and stage 9 passing on `main` without it | — |

**What "finished" means** is `Mars_Native.md` §6's three criteria, unchanged:

1. A player gets MercuryRT on every platform, in the measured configuration, without choosing it.
2. Nothing the C# core can do is lost, or the loss is written down.
3. The C# core's role is decided.

Criterion 2's written list would begin with whatever §6.5 of that page lists for MarsRT that Mercury also has, such as
rewind. Mercury has no `ISnapshotCore`, so rewind uses the full state on both engines.

## 5. Risks and open questions for the user

- **Q1, naming.** *Decided by the user on 2026-09-23:* MercuryRT, in `MercuryRT - GB/` (§2.2).
- **Q2, one native library or two.** A second `cdylib` means a second file shipped and a second interface version. It
  also means either a copy of MarsRT's 410-line `state.rs`, which needs a string primitive MarsRT lacks, or a small
  shared crate both depend on. The shared crate touches MarsRT, which another agent is working in. The recommendation is
  a separate crate now and the extraction once both are stable, but the decision is the user's.
- **Q3, whether to fix §6.1's C# defects before the port.** The recommendation is yes, each proven by a test before and
  after, because after the port every fix costs two implementations in lock-step:
  - D1 (sample rate) changes the sound but not the state;
  - D2 (the clock in double speed) changes how state evolves, but not the format;
  - D3 (the host path in the state) and the triple cartridge copy are one format change, to state version 6. That
    shrinks Yellow's state by 64 KB and removes `_savePath` and the string primitive the port would otherwise need.
  *Decided by the user on 2026-09-24: fix all three now, in both engines in lock-step, each shown by a test before and
  after; D3 is state version 6, and version-5 states still load.*
- **Q4, whether the port carries Mercury's known simplifications.** The simplifications include the per-line renderer,
  mode 3 not lengthening for sprites, internal cycles at the end of an instruction, instant GP-HDMA, and the 79
  non-passing corpus ROMs of §3.4. The recommendation is to port them exactly and improve afterwards, because an
  improvement made during the port has no oracle. After the port, improvements are made either in both engines, while C#
  is the oracle, or in Rust alone with the corpus as the oracle, once C# is frozen. That second choice is the user's.
  *Decided by the user on 2026-09-24: improvements after the port land in both engines in lock-step, with C# the
  oracle, until the C# core moves to its legacy branch.*
- **Q5, the C# Mercury's future.** *Partly decided by the user on 2026-09-23:* every C# core will eventually move to a
  legacy branch. §3.7 says how the oracle survives the move. Two questions stay open: when the Engine row's default
  flips to MercuryRT, and whether C# is still offered to players until the move.
  *Decided by the user on 2026-09-24: the Engine row's default flips to MercuryRT after stages 6 and 7 and a session of
  play on the handheld, the condition MarsRT's flip had (`Mars_Native.md` §6.2).*
- **Q6, the corpus's place.** Is the corpus kept gitignored under `TestRoms/hardware/`, as `Mercury_HardwareTests.md`
  §3 intends? Is WiseMan's theory changed from "passes" to "matches the recorded baseline"? That changes a committed
  test's meaning, which is why it is asked rather than done.
  *Decided by the user on 2026-09-24: the corpus stays gitignored under `TestRoms/hardware/`, and the committed theory
  becomes "matches the recorded baseline", so a verdict that changes in either direction fails until the baseline is
  re-recorded.*
- **Risk: P1 is wide by design.** No component like Mercury's tick loop was measured in Mars. If stage 2's number comes
  in under 1.2×, the port is still worth finishing on grounds 1 and 3 of §1.2. The later lever, batching the devices
  (§1.2), would then be the next question, with its own prediction.
- **Risk: bit-exact sound across platforms.** §3.3 removes `pow`. Nothing else in the mixer is platform-dependent by
  argument. That argument is untested on Windows and macOS until stage 7.

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

## 8. The work, stage by stage

### 8.1 Stage 1: the state, byte for byte (done 2026-09-23)

*Order.* Stage 1 was done before stage 0. The state reader needs no corpus, and stage 0's two decisions (Q3, Q6) are
the user's to make.

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
every command's output failing. `/tmp` is a 16 GB tmpfs, shared with other sessions' scratch space, and it had filled.
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

**Not yet:** breakpoints, stepping, coverage, and the write log that feeds watches. These are stage 6's hooks. The
Engine row's hint says so.

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
