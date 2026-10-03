# Moon_Native — a plan to port Moon to Rust

*Written 2026-09-24, before any port code.* On 2026-09-23 it was decided that every EmuSen core moves to Rust
(`EmuSen_Stack.md` §2.1). MarsRT (N64) and MercuryRT (Game Boy) are done. This page plans the port of Moon, the NES
core, as MoonRT. It follows the method of `Mars_Native.md` §5 and takes its shape, stage for stage, from
`Mercury_Native.md`, the other 2D port. As `EmuSen_Stack.md` §2.1 requires of every 2D port, it states its speed
prediction before the work begins and prices it against a measured C# baseline.

The page keeps two things apart. **Measured** means a number taken on 2026-09-24 on the desktop described in §1.1.
**Argued** means reasoning from the code, or from Mars's and Mercury's evidence, with no measurement behind it. Each
prediction is numbered (P1–P6) so that it can be retired later. Mercury's prediction was refuted (`Mercury_Native.md`
§8.2), and this plan is written in the light of why.

## 1. What the port is for

### 1.1 The baseline (measured)

**How it was measured.** The harness has no NES bench, so a throwaway console app (`moonbench`, in
`~/.cache/emusen/probe/moonrt/`) was built against the checkout's `EmuSen.csproj`, in Release on .NET 10, on the Ryzen 7
7700X. It does what Mercury's bench did:

1. It loads a scratch copy of the ROM with the battery disabled.
2. It boots through 900 frames, pressing Start for 5 frames of every 90 and A for 5 frames 45 later.
3. It times 3,000 frames flat out, one `RunFrame` each, taking the picture and draining the audio after every frame.

Every run is its own process, under the bench lock, with the load average under 1.2. Games are interleaved within a
round. Each game's state hash was the same in every round and in every mode, so the runs were deterministic.

The four games cover the four cases the plan set out:

| Game | Board | Why |
|---|---|---|
| Super Mario Bros. (JU, PRG 0) | NROM (0) | no banking at all |
| The Legend of Zelda (U, PRG 0) | MMC1 (1), battery, CHR RAM | the serial register, and a battery save |
| Super Mario Bros. 3 (U, PRG 0) | MMC3 (4) | the A12 scanline IRQ, and DMC drums |
| Mike Tyson's Punch-Out!! (U, PRG 0) | MMC2 (9) | DMC speech (the APU-heavy case), and a CHR latch the PPU's own fetches move |

| Game | Mean ms/frame (3 rounds) | Median (3 rounds) | % of full speed (16.639 ms) |
|---|---|---|---|
| Super Mario Bros. | 1.313 / 1.300 / 1.358 | 1.300 / 1.279 / 1.343 | 1225–1280% |
| Zelda | 1.173 / 1.170 / 1.177 | 1.168 / 1.164 / 1.173 | 1414–1422% |
| Super Mario Bros. 3 | 1.575 / 1.575 / 1.573 | 1.555 / 1.565 / 1.558 | 1056–1058% |
| Punch-Out!! | 1.545 / 1.503 / 1.466 | 1.561 / 1.520 / 1.483 | 1077–1135% |
| Synthetic `JMP` loop, rendering off | 0.774 | 0.807 | 2150% |

A Moon frame costs about twice a Mercury frame (`Mercury_Native.md` §1.1: 0.54–0.68 ms).

**`SkipRendering` is not the renderer's share here.** In Mercury it skipped the whole line renderer. In Moon it skips
only the pixel writes at the end of `Composite` (`Ppu.Render.cs:177`): background decode, sprite evaluation, sprite
0, overflow and priority all still run, because they set status bits the game polls. With it on, over two rounds:

| Game | Mean ms/frame, pixel writes skipped | Saved |
|---|---|---|
| Super Mario Bros. | 1.221 / 1.198 | ≈0.10 ms (8%) |
| Zelda | 1.083 / 1.078 | ≈0.09 ms (8%) |
| Super Mario Bros. 3 | 1.490 / 1.492 | ≈0.08 ms (5%) |
| Punch-Out!! | 1.415 / 1.400 | ≈0.10 ms (7%) |

**The idle machine.** The synthetic ROM is a `JMP $8000` loop that never enables rendering and never writes `$4014`.
It still costs 0.77 ms, which is 50–65% of a game's frame. That is the price of clocking the PPU three dots and the APU
one cycle per CPU cycle, with nothing to draw and nothing to play.

**A component profile.** `ipsample.py` (`~/.cache/emusen/probe/mars-speed/agent-cpu-tools/ipsample/`) sampled the
main thread of the C# Moon on Super Mario Bros. 3, 15 s after a 4 s boot, about 14,100 samples a run.

- **With inlining,** the named leaves are:
  - `MemoryBus.Tick`, 23.8% (the PPU's dot loop and `Apu.Step` inline into it);
  - `Ppu.Composite`, 13.8%;
  - `Apu.Mix`, 11.0%;
  - `Ppu.BackgroundFetch`, 9.1%;
  - `MoonCore.RunCpuUntilBudgetSpent`, 8.8% (the CPU's step inlines into it);
  - `Ppu.RenderBackground`, 6.6%;
  - `PulseChannel.Muted`, 2.9%; `Mmc3.OnPpuAddress`, 2.6%; `Ppu.SpriteFetch`, 2.3%;
  - `CheatRegistry.TryPatchRom`, 1.8%; `BreakpointRegistry.ShouldBreak`, 1.3%; `[vdso]` (the frame loop's two
    `Stopwatch.GetTimestamp` calls a scanline), 0.9%.
- **With `DOTNET_JitNoInline=1`,** which inflates calls but attributes by method, grouped by component:
  - the APU, ≈33%, of which `Mix` alone is 16.6%;
  - the PPU's per-dot clock and fetch address generation, ≈20%;
  - the line renderer (`Composite`, `RenderBackground`, the palette and nametable helpers), ≈17%;
  - the MMC3 (`OnPpuAddress`, `A12Watcher.Rose`, the bank lookups), ≈7%;
  - the debugger seams (`ShouldBreak` and the empty `List` enumeration under it, `TryPatchRom`), ≈7%, plus 3% in
    virtual-stub dispatch;
  - the 6502 and the bus decode, under 10%.

**What the profile says about Moon's cost.** As in Mercury, a frame is not the cost of the game's program. It is the
cost of clocking every device once per CPU cycle, and here three times for the PPU. Each CPU cycle, `MemoryBus.Tick`
(`MemoryBus.cs:62-80`) runs:

- three PPU dots, each deciding which of four fetch addresses to put on the bus and offering it to the board's A12
  watcher (`Ppu.Timing.cs:113-189`);
- one APU cycle: triangle and DMC timers every cycle, the pulses and noise every other one, the frame sequencer, and
  **a full evaluation of the non-linear DAC**, with two or three floating-point divisions (`Apu.cs:275-338`);
- the board's CPU-cycle hook, where a board has one;
- two interrupt-line updates into the CPU.

That is 29,781 ticks and 89,342 dots a frame. The DAC alone is about 90,000 divisions a frame, for 734 output samples.

**The low-end target (measured).** The proxy was `--throttle`'s mechanism, `systemd-run --user --scope -p
CPUQuota=N% -p CPUQuotaPeriodSec=5ms`, as `EmuSen.Pharaoh/CpuThrottle.cs` uses it.

| Quota | Game | Mean ms/frame | % of full speed |
|---|---|---|---|
| 33% | Super Mario Bros. 3 | 5.98 / 6.22 | 268–278% |
| 33% | Zelda | 4.84 / 4.83 | 344% |
| 10% | Super Mario Bros. 3 | 21.39 (p90 29.6) | **78%** |

**At a tenth of a core, Moon is below full speed.** Mercury never was: it measured 178% at the same quota. This is
the first finding the port has to answer to (§1.2).

### 1.2 What that says the port is for (argued)

Moon is 10–14× full speed on the desktop, about 2.7× on a third of a core, and 0.78× on a tenth of one. At the
console's rate the desktop spends about 1.5 ms × 60.1 ≈ 90 ms of CPU time a second on it, 9% of one core.

**So the port is for three things, in order:**

1. **Consistency with the stack, and the method's third instance.** The project's decision is that every core is Rust.
   Moon is the second 2D core, and the first whose picture feeds its machine (§2.5). It tests whether Mercury's
   finding, that a line-for-line 2D port moves the costs without changing them, generalises, or was Mercury's own.
2. **Weak-machine headroom, which Moon actually lacks.** Unlike Mercury, Moon has a measured configuration below 1×
   (10% quota, 78%). A line-for-line port is not expected to close that gap by itself (P4). The port's value there is
   that it puts the core where the design levers below can be built once, in one language, with the C# core still
   available as their oracle.
3. **Power.** CPU time per emulated second is energy on a handheld. Argued, not measured.

**What it is not for.** The largest levers on Moon's frame are algorithmic in either language:

- **The DAC evaluated per cycle.** `Mix` is a pure function of five channel outputs, which change far less often than
  once a cycle. Caching the last result and re-evaluating only when an output changes is exact, since it is the same
  arithmetic on the same inputs.
- **The fetch addresses computed per dot for every board.** Ten of sixteen boards ignore `OnPpuAddress`. `BusAddress`
  is state, but its value at any dot is a function of `V`, the dot and `PPUCTRL`, so it can be derived when observed.
- **Sprite pixels decoded per screen pixel.** `SpritePixel` re-reads both pattern planes for every pixel of every
  sprite column (`Ppu.Render.cs:192-224`). A per-line decode is exact on every board except MMC2, whose latch observes
  every `ReadChr` (§2.5).

Each of these is a design change, not a port, and each is kept out of the port for Mercury's reason: the port's oracle
is exactness against the C# core, and a change made during the port has none. They are listed in §5 as later
questions, each to carry its own prediction.

### 1.3 The speed prediction, and how it was derived

**What Mercury measured.** `Mercury_Native.md` §8.2 retired its P1 (predicted 1.6×) at 1.0–1.2× for whole frames. The
machine core without the renderer gained 1.06–1.41×; the renderer was **slower** in Rust (0.65–0.8×). Its reading:
"the port moved the costs without changing them, and the cost is the design (one step of every device per T-cycle),
not the language". Why one game's machine gained a sixth of another's was not explained.

**Which of Moon's costs a language change can move (argued):**

| Moon cost (share of an SMB3 frame, §1.1) | What bounds it | Factor assumed |
|---|---|---|
| Debugger seams in the plain loop (≈6% inlined) | Code that MoonRT's plain frame does not contain (`Mars_Native.md` §6.5's shape) | removed |
| The DAC in `Mix` (≈14%) | Floating-point division latency, which is the same instruction in both | 1.0–1.15, central 1.05 |
| APU timers, sequencer, `Muted`, envelopes (≈10%) | Branchy counter code over small objects; Mercury's analogue gained 1.06–1.41 | 1.1–1.5, central 1.3 |
| PPU per-dot clock and the board's A12 path (≈30%) | The same shape as Mercury's per-T-cycle clock, plus an indirect call per fetch that becomes an `enum` match | 1.0–1.45, central 1.2 |
| Line renderer, composition included (≈22%) | Per-pixel loops over byte arrays; Mercury's renderer was **slower** in Rust | 0.75–1.2, central 0.95 |
| 6502, bus decode, the rest (≈18%) | Mars's interpreter gained 1.5–1.75; Mercury's SM83 was not measured apart | 1.1–1.6, central 1.3 |

**P1 (speed, desktop).** A line-for-line MoonRT, with no algorithmic change, runs a plain frame in **0.69 to 0.99 of the
C# Moon's time, central 0.83 (1.2×)**, on §1.1's four games. The composite comes from the table:
0 + 0.14/1.05 + 0.10/1.3 + 0.30/1.2 + 0.22/0.95 + 0.18/1.3 ≈ 0.83. The ends of the range take every row's pessimistic or
optimistic factor together. Per game, the central values are Super Mario Bros. 1.31 → about 1.09 ms, Zelda 1.17 → 0.97,
Super Mario Bros. 3 1.57 → 1.31, Punch-Out!! 1.50 → 1.25.

This is deliberately below Mercury's refuted central of 1.6×. The two rows that decide it are the ones Mercury
measured: the per-cycle clock (Mercury: about 1.2) and the renderer (Mercury: under 1). The one new row, the DAC, is
bounded by an instruction whose cost no language changes.

**P2 (the idle machine).** The synthetic loop's 0.77 ms is the PPU's clock and the APU's DAC and nothing else. MoonRT
runs it in 0.60–0.77 ms (1.0–1.3×). If it gains more than the games do, the renderer is where the games lose it.

**P3 (seams).** MoonRT's plain frame spends no sampled time on breakpoint, coverage, ROM-patch or phase-timer checks,
against about 6% (inlined) in C#.

**P4 (what a player sees).** Nothing observable changes on the desktop or at 33% quota. At 10% quota, Super Mario Bros.
3 goes from 78% to 80–113% of full speed, central 94%: **MoonRT alone probably does not reach full speed on a tenth of
a core.** That is stated now so that §1.2's design levers are priced against the port's result, not the C# one.

**P5 (the boundary).** Crossing the boundary once a frame costs under 3% of MoonRT's frame: the call, a 245,760-byte
RGBA copy and about 3 KB of audio.

**P6.** No recompiler and no threads are needed for MoonRT to beat C# on every measured configuration.

**The first like-for-like number** comes at the end of stage 2 (§4), with both engines' pixel writes skipped. That is
exact like for like, because `SkipRendering` removes the same code on both sides. P1 is re-priced there.

*Retired on 2026-09-30 (§8.2.3), with the pixel writes on and through the shim.* P1 held, at 0.72–0.78 of C#'s time.
P2 was refuted in the favourable direction, at 0.52 ms.

## 2. Shape

### 2.1 The whole machine, one boundary a frame

The whole machine is in Rust: the 2A03's CPU and APU, the bus, OAM DMA, the DMC's fetches, both pads, the PPU with its
per-dot clock and line composition, the cartridge and all sixteen boards. A C# shim, `Shim/MoonRtCore.cs`, implements
what `MoonCore` implements: `ICore`, `IFrameProfiler`, `ICheatRegistryHost`, `IStateFormat` (`MoonCore.cs:17`), and
`IFrameBufferPool` as MercuryRT's does. Moon implements neither `ICoreSettings` nor `ISnapshotCore`.

`MoonCore` has one public member beyond `ICore` that a caller needs: `Reset()`, the RESET button, which the blargg
test-ROM runner uses (`Moon_Core.md` §6, `Moon_TestRoms.md` §2.2). MoonRT carries it.

**The end of a frame runs in C#, in C#'s order** (`MoonCore.Schedule.cs:367-383`): `TotalFrames++`, the phase
timings, `FrameLog.RecordFrame`, `ApplyCheats`, and every 300th frame `SaveSram`. One subtlety: C#'s `RunFrame`
calls `EndScanline` *after* `EndFrame` returns (`MoonCore.cs:153-157`), so the cheats are applied before the
timeline's last line is closed. The two touch disjoint state (memory against four `long`s), so MoonRT closes the line
in Rust and C# then applies the cheats, with the same result.

The in-frame seams become MarsRT's §6.5 mechanisms, because Rust never calls C#:

| C# seam | Where | In MoonRT |
|---|---|---|
| `Breakpoints.ShouldBreak(pc)` before every instruction | `MoonCore.cs:173` | Tables pushed down; the frame returns early with a reason and a pc; C# confirms with `ShouldBreak` |
| `Coverage.Record(pc)` | `MoonCore.cs:181` | A Rust bitmap C# reads |
| `IWriteObserver.OnWrite` on writes to RAM, PPU and APU registers, PRG RAM | `MemoryBus.cs:135,148,169,176` | A write log C# drains after the frame, in order |
| `IRomReadPatcher.TryPatch` on every cartridge read (Game Genie) | `MemoryBus.cs:115` | A patch table keyed by **CPU address**, pushed down when the registry changes (§2.4) |
| `ApuWriteTrace` and `NesPpuWriteLogging` on register writes | `MemoryBus.cs:127,143` | Out of the plain frame; a debug build of the log is a stage-5 question |
| Two `Stopwatch.GetTimestamp` calls per scanline | `MoonCore.cs:152-158` | The shim reports one phase, the whole frame; `IFrameProfiler` stays implemented |

### 2.2 Name and place

The name is **MoonRT**, in `EmuSen/Cores/Nintendo/MoonRT - NES/`, by the project's pattern for every Rust core. The crate is
`moonrt`, a `cdylib` + `rlib` with MercuryRT's `[profile.release]` and `[profile.dist]` and no dependencies. Its layout
mirrors the C# folders:

| Rust | C# |
|---|---|
| `cpu/` (the step, the address modes, the opcodes including the undocumented ones) | `Cpu/Core`, `Cpu/Opcodes` |
| `memory/` (bus, cartridge, `mappers.rs` with the sixteen boards as one `enum`, the A12 watcher, pads) | `Memory/`, `Input/` |
| `ppu/` (registers, per-dot timing, composition) | `Ppu/` |
| `apu/` (sequencer and mixer, channels, DMC) | `Apu/` |
| `machine.rs` (`RunFrame`'s timeline, `Reset`, the spaces) | `MoonCore*.cs` |
| `state.rs`, `naming.rs` | `StateSerializer`, as `Mars_Native.md` §5.1 ported it |
| `ffi/` | the shim's interface |
| `Shim/` (`MoonNative.cs`, `MoonMachine.cs`, `MoonRtCore.cs`) | — |

The 6502 disassembler, the cheat codecs, `MoonCore.Spaces.cs`'s names, `ApuWriteTrace` and the `Validation` adapters
stay C#.

### 2.3 The C ABI (interface 1, proposed)

`delegate* unmanaged` function pointers, as MercuryRT's. A negative return is a status; a panic never crosses into C#.

- **Lifecycle:** `moon_interface_version`, `moon_set_crash_log`, `moon_machine_new(image, len, status) → handle`,
  `moon_machine_free`. The iNES parse runs on both sides: C# needs it for the unsupported-board exception and the
  battery flag before any handle exists.
- **Frame:** `moon_machine_run_frame(handle, detail) → status`. Zero is a completed frame. A negative status is a C#
  exception MoonRT has to reproduce (§6.3: an image whose board indexes outside its PRG), with the shim throwing the
  same type.
- **Reset:** `moon_machine_reset`, `MoonCore.Reset()`.
- **Input:** `moon_machine_set_buttons(handle, port, mask)`, eight bits in `NesButton` order.
- **Picture:** `moon_machine_frame(handle, out, len)`, 256×240 RGBA, copied.
- **Sound:** `moon_machine_audio_buffered`, `moon_machine_drain_audio(handle, out, len, max_frames)`, and the channel
  mutes. The filter coefficients use only `+ − × ÷` and `Math.PI` (`Apu.cs:79-100`), so, unlike Mercury's `Math.Pow`,
  Rust computes them itself; the argument is §3.3's.
- **State:** `_save_state_size`, `_save_state`, `_load_state`, `_state_layout`.
- **Memory:** `moon_machine_read_space`/`_write_space(handle, space, addr, buf, len)` over the eight spaces of
  `MoonCore.Spaces.cs`, CPUBUS through the real decode with its side effects, and `_space_size`.
- **Options and cheats:** `moon_machine_set_options` (skip rendering), `moon_machine_set_rom_patches`.
- **Debugger (stage 5):** `moon_debug_*`, in `Mars_Native.md` §6.5's shapes.
- **Test ABI:** `moon_machine_step` (one `RunCpuUntilBudgetSpent` iteration without the budget: the pending DMA
  charge, the NMI edge, one `Cpu.Step`, the stolen cycles).

*Revised 2026-09-28: `moon_machine_set_audio_limit` joins the sound exports, carrying `AudioSettings.AudioBufferMaxSamples`
to the Rust queue, and the interface version is 2 (`EmuSen_Settings_Reference.md` §4.85.2).*

### 2.4 What stays C#

- the frontends, `CoreFactory` and `CoreCatalog`;
- every registry (watches, frame log, breakpoints, cheats, coverage, labels) and their rules;
- the cheat codecs and `CheatRegistry.ApplyAll`'s poke loop, over `write_space`;
- the disassembler, and `.srm` I/O through `AtomicFile` beside the ROM, with `CoreOptions.BatteryRamDisabled`;
- the debug target's presentation. As in MercuryRT (`Mercury_Native.md` §8.3), `MoonDebugTarget` gains an optional
  host that routes reads, writes and refreshes, and reads a mirror `MoonCore` loaded from MoonRT's state on each
  `RefreshProviders`. A Moon state is 20–30 KB (§3.1), so the transfer costs well under a millisecond.

**The Game Genie table.** Moon's patcher is asked with the CPU address (`MemoryBus.cs:115`), where Mercury's was asked
with the ROM address, so the shim's flattening probes `$4020`–`$FFFF` rather than `$0000`–`$7FFF`. Otherwise it is
Mercury's: one entry per patched address, 256 values of `0x100 | patched` or 0, rebuilt when `CheatRegistry.Version`
moves.

### 2.5 Where Moon differs from Mercury in ways that change the method

- **The renderer feeds the machine.** In Mercury the renderer could be skipped without changing the state
  (`Mercury_Native.md` §6.5), so stage 2 was proven by state alone before any pixel existed. In Moon, `RenderScanline`
  runs whatever `SkipRendering` says, and composition writes machine state three ways:
  - sprite-0 hit and sprite overflow are `Status` bits (`Ppu.Render.cs:125,170`);
  - every `ReadChr` of the renderer (two per background tile, two per *sprite pixel*) passes through the board, and
    MMC2's `ReadChr` moves its latch (`Mmc2.cs:49-64`), so the renderer's read pattern is part of Punch-Out!!'s state;
  - the evaluated sprites (`_spriteCount`, `_spriteIndices`, both `[SkipInState]`) decide `SpritePatternBaseForLine`,
    and so the A12 edges MMC3 and RAMBO-1 count during the sprite fetches (`Ppu.Timing.cs:170-183`).
  So composition is stage 2, line for line, and only the pixel writes are left for later. The renderer cannot be
  rewritten for speed during the port at all on MMC2, and elsewhere only where the read pattern is preserved.
- **The CPU and the bus hold each other.** `MemoryBus.Tick` sets the CPU's NMI and IRQ lines (`MemoryBus.cs:65,79`),
  `RunOamDma` sets the IRQ line (`:194`), and a mapper write is stamped with `Cpu.Cycles` (`:173`). In Rust the bus
  cannot hold the CPU, so the CPU applies the lines after each tick and after each write, in C#'s order; nothing
  between the C# assignment and the Rust one reads them.
- **The DMC reads through the bus from inside the APU's step.** `Apu.Dmc.ReadMemory` is `Bus.Read` (`MoonCore.cs:99`),
  called from `Apu.Step` inside `Bus.Tick`. The address is always `$8000`–`$FFFF` in a running machine, but a loaded
  state can hold any. MoonRT splits the fetch: the DMC asks for an address, the APU's step performs it through a
  reader over the bus's other parts, and `$4015`, the APU reading itself, is answered by the APU.
- **The mapper interface has sixteen implementations and five optional hooks** (`IMapper.cs:16-51`). In Rust they are
  one `enum`, and the hooks are `match` arms; `ClocksOnCpuCycle` stays a flag read once, as C# reads it.
- **The machine is small enough to compare after every instruction.** A Moon state is 20–30 KB.
- **The oracle corpus exists and is wired.** `/home/red/nes-test-roms` holds `christopherpow/nes-test-roms`, and the
  runner (`NesTestRomRunner`) speaks blargg's `$6000` protocol. Unlike Mercury, stage 0 has no corpus work to do.

## 3. The oracle and the state format

### 3.1 The state, byte for byte (read from the code; checked by a reflection walk in stage 1)

*Since 2026-09-28 the codec is shared.* The writer, the reader, the layout listing, the naming rule, `Skip<T>` and the
state half of the C ABI live in `emusen-native` (`EmuSen_RustState.md`), which every Rust core builds from. MoonRT's `state.rs` keeps its magic, version and refusals; its state exports are `state_exports!`. Its status codes
were already inside the shared rule and did not change.

The format is `StateSerializer`'s reflection walk (`EmuSen/Common/StateSerializer.cs:36-44`), and MoonRT reproduces it
exactly. From the code:

- **The header.** The magic `"MOON"` (`0x4E4F4F4D`), version 3 (`MoonCore.cs:28-30`), then five `long`s: `TotalFrames`,
  `_lineStartClock`, `_cpuBudget`, `_masterClock`, `_cpuRemainder` (`MoonCore.cs:259-265`).
- **The body.** The walks are `Cart`, `Cart.Mapper`, `Cpu`, `Bus`, `Ppu`, `Apu`, in that order (`MoonCore.cs:267-272`).
  Within each class, fields come in the ordinal order of their names: every upper-case name before `_`, and `_` before
  every lower-case letter.
- **The bus is written twice.** `Cpu._bus` is `private readonly ICpuBus`, not `[SkipInState]` (`Cpu.cs:35`). An
  interface-typed field is written as a class: a presence byte and the runtime object's fields. So the `Cpu` walk
  carries a whole `MemoryBus` (2 KB of RAM, open bus, the DMA and stolen-cycle counts), and the `Bus` walk writes it
  again. On load the last copy wins. This is Mercury's triple cartridge in another place.
- **Classes inside classes.** `Mmc3._a12` and `Rambo1._a12` (`A12Watcher`, its `_lowSince`), each APU channel, and
  each `Envelope` are presence byte plus fields.
- **Readonly fields are serialised and overwritten on load:** `PulseChannel._onesComplement`, the boards' `readonly int[]`
  bank tables, and `A12Watcher._minimumDotsLow` is not, because it is `[SkipInState]`.
- **The layout depends on the ROM.** No tag says which board follows; the header's mapper number builds the board.
  CHR is serialised (it may be RAM) and its length is the header's; PRG ROM is not.
- **Not in the state, and so kept across a load:**
  - the picture;
  - the APU's box-filter accumulator, sample fraction, the three filters' memories, and its sample queue
    (`Apu.cs:51-69`);
  - both pads, held buttons *and* their shift registers and strobe (`MemoryBus.cs:23-24`);
  - the renderer's line scratch and evaluated sprites (`Ppu.cs:68-71`), which feed A12 on the pre-render line
    (§2.5, §6.2);
  - `MemoryBus._mapperClocksOnCpu`, which is derived from the board.

  A C# `LoadState` leaves these at the instance's own values, so MoonRT must behave the same way, and the frame
  comparison loads both engines fresh (§3.3).

### 3.2 How the reader and writer are proven

`Mars_Native.md` §5.1's three separate proofs, as Mercury used them:

1. **Byte round trip.** A C# state is read by MoonRT and written back, compared byte for byte, for a running synthetic
   machine on every one of the sixteen boards, with CHR ROM and CHR RAM, and with noise in every field.
2. **A layout listing** of the form `offset len type path`, compared line by line against a C# reflection walk of the
   same objects.
3. **`naming.rs`**, which checks every label against the field actually written.

### 3.3 The frame-by-frame comparison

Both engines are loaded fresh, `LoadRom` then `LoadState`, so the skipped fields of §3.1 start equal. They run the same
input script. After every frame: the C# save state of each; the RGBA frame; the drained samples.

**The mixer can be bit-exact (argued).** The mixer is IEEE doubles in a fixed order (`Apu.cs:275-338`), with no
transcendental: the filter coefficients use `Math.PI` and the four basic operations, each correctly rounded on every
platform. Neither RyuJIT nor rustc contracts to FMA by default. The clamp-then-truncate to `short` matches Rust's
`as i16` for in-range values.

### 3.4 The test ROM corpus, line for line

`Moon_TestRoms.md` §5 recorded 45 of the 63 ROMs that return a verdict passing, at a 2,400-frame budget, on
2026-08-05. The corpus at `/home/red/nes-test-roms` holds more than those suites (`mmc3_test_2`, `cpu_dummy_reads`,
`dmc_tests`, `sprite_overflow_tests` and others), and screen-only ROMs besides. As for Mercury, **the corpus is a
transcript, not a pass list**: for every ROM, both engines must give the same verdict, on the same frame, with the same
`$6004` text, the same number of resets, and the same state at the verdict frame. A C# baseline is recorded first, so
that a change to either engine shows as a line.

#### 3.4.1 AccuracyCoin, beside the corpus (baseline measured 2026-09-28)

**The ROM.** AccuracyCoin, by 100thCoin (Chris Siebert), is a single NROM image that runs 144 scored tests of the CPU,
DMA, APU and PPU on one screen of results.

- Source: `https://github.com/100thCoin/AccuracyCoin`, commit `673ef550db296136d52229961e7d39366116882a`
  (2026-09-23). The repository has no tags or releases; the prebuilt `AccuracyCoin.nes` is checked in and last
  changed in that commit.
- Licence: MIT.
- `AccuracyCoin.nes`: 40,976 bytes, SHA-256 `4fe8c2bc9abc6f4d418da47b73f62cba89fcacd950fae763097a1681d650e839`.
  It is iNES 1.0, mapper 0, 32 KB of PRG and 8 KB of CHR ROM, and it targets an NTSC RP2A03G and RP2C02G.
- Where it lives: `~/.cache/emusen/probe/moonrt/accuracycoin/`. Like the corpus, it is third-party and never
  committed. WiseMan reads it from `EMUSEN_ACCURACYCOIN`.

It was fetched because NES_MiSTer's 2025–2026 fixes to its bus, DMA and PPU were each graded by one of its sub-tests
(§3.7.1, Q7). It lets the audit settle those disagreements with a ROM instead of leaving them as candidates.

**How it reports**, read from its source (`AccuracyCoin.asm`):

- One result byte per test in RAM, `$0400`–`$0495`, at the addresses `:146-310` defines.
  - The low two bits are 1 for a pass and 2 for a fail (`:17074-17083`).
  - Bits 7–2 are the error code on a fail, and on a pass the "behaviour" number of a test with several passing
    behaviours (`:977-983`).
  - 0 means not run, and `$FF` marked to be skipped.
- The table's tallies are `$37` (tests), `$38` (passed) and `$3F` (skipped) (`:83-91`).
- The menu is ready when `$EC` reaches `$0A` (`:436-468`).
- From the menu, Start runs every page and then draws the table. Presses are edge-detected in the NMI, so the button
  must go from released to pressed.
- **The end** is the NMI hook at `$0700` rewritten to `JMP PressStartToContinue` (`:1097-1102`), which is
  `4C 08 93` in this build.
  - `$35`, `RunningAllTests`, is not the end: the PPU open-bus test clears it and sets it again part-way through.
  - The address `$9308` was found by searching the PRG for the routine's bytes, not from a listing, and is confirmed
    by the table appearing on screen at that frame.

**The harness.** `NesTestRomRunner` speaks only blargg's `$6000` protocol and sends no input, so it would report no
result here.

- `EmuSen.WiseMan/Fixtures/AccuracyCoinRun.cs` drives the protocol above over any NES core's frame, RAM and Start
  button:
  - power on;
  - wait for `$EC = $0A`;
  - thirty frames later, hold Start for five frames;
  - run to the table's hook, within 8,000 frames;
  - read the result block and the tallies.
- `MoonAccuracyCoinTests` decodes synthetic bytes. With the ROM present, it also runs the C# core and holds it to the
  baseline below, byte for byte, with the frame the menu was ready and the frame the table was drawn.

**The C# Moon's baseline, on the unmodified tree at `cd6bc71e`: 89 of 144 pass, 55 fail, none skipped.** That is also
the ROM's own screen, "TESTS PASSED: 89 / 144".

- The menu was ready at frame 23, Start was held for frames 54–58, and the table was drawn at frame 4,099.
- Two runs gave the same RAM, picture and log.

| Page | Passed | The failures (error code) |
|---|---|---|
| 1 CPU behaviour | 8 of 9 | Open bus (1) |
| 2 Addressing-mode wraparound | 6 of 6 | — |
| 3–9 The unofficial read-modify-write, `*AX`, DCP and ISC opcodes | 52 of 52 | — |
| 10 SH* | 1 of 6 | SHA ind,Y and abs,Y, SHS, SHY and SHX (7: the wrong target address when RDY falls two cycles before the write) |
| 11 Unofficial immediates | 8 of 8 | — |
| 12 CPU interrupts | 0 of 3 | Interrupt-flag latency (8), NMI overlapping BRK (2) and IRQ (1) |
| 13 APU registers and DMA | 0 of 10 | DMA with open bus, `$2002`, `$2007` read and write, `$4015`, `$4016`; DMC DMA bus conflicts; DMC DMA with OAM DMA; explicit and implicit DMA abort |
| 14 APU | 3 of 9 | Frame counter IRQ (7), 4-step (2) and 5-step (2); DMC (18); APU register activation (1); controller strobing (4) |
| 15 CPU behaviour 2 | 2 of 5 | Instruction timing (2), implied dummy reads (3), internal data bus (1) |
| 16 Power-on state | not scored | drawn, not scored; the PPU reset flag at `$0360` reads fail 1 |
| 17 PPU behaviour | 5 of 5 | — |
| 18 PPU vblank timing | 3 of 7 | NMI timing, suppression, at vblank's end, and disabled at vblank (1 each) |
| 19 Sprite evaluation | 0 of 8 | all eight fail with code 1 |
| 20 PPU miscellany | 1 of 7 | `$2004` (4), `INC $4014` (2), the rendering flag (2), `$2007` while rendering (1), the `$2004` and `$2007` stress tests (2 each) |
| 21 Advanced background evaluation | 0 of 5 | all five |
| 22 Advanced sprite evaluation | 0 of 4 | all four |

**What the baseline does not say.** Code 1 on page 19 is "a sprite zero hit did not occur", and many of pages 20–22's
tests begin by relying on sprite 0 hit working. So page 19's failures may cascade: one cause could fail most of pages
19–22. The same may hold for pages 12, 13 and 15 and the frame counter, many of which depend on DMC DMA timing. Both
readings are inferred from the error codes and not investigated; the audit of §3.7 begins with the CPU and the bus.
The corpus's 45 of 63 and this ROM's 89 of 144 measure different things, and neither is folded into the other.

### 3.5 Every board

Moon implements sixteen boards (`Cartridge.cs:166-186`): NROM, MMC1, UxROM, CNROM, MMC3, AxROM, MMC2, Color Dreams,
RAMBO-1, Irem H3001, GxROM, Sunsoft-3, Sunsoft-4, Sunsoft FME-7, Camerica and NINA-003-006. Each gets a synthetic
program that writes its registers, runs its IRQ where it has one, and banks PRG and CHR, compared frame by frame; and
random programs on each, compared instruction by instruction. Real games cover the boards the library has copies of.
`MoonMapperTests`' 31 cases run through a rig on both engines in stage 7.

### 3.6 When the C# core leaves the main branch

`Mercury_Native.md` §3.7's three layers, unchanged: golden traces recorded from C# before the move (per frame, 8-byte
hashes of state, picture and sound; the corpus transcripts), full states at intervals kept locally for bisection, and a
live differential on the legacy branch loading MoonRT's library by path.

**One layer Moon can add, and the recommendation.** The 6502 has a ground truth that needs no C# core at all:
SingleStepTests' `nes6502/v1`, 2.56 million cases with per-cycle bus traces, which the C# CPU passes
(`Moon_CPU.md` §7). A test ABI that steps MoonRT's CPU over a flat 64 KB bus would let the vectors grade the Rust CPU
directly, after the move as before it. The vectors are third-party, not committed, and not currently on this machine,
so this is proposed, not planned into a stage.

### 3.7 The MiSTer core as auditor

**Decided 2026-09-28: each component is audited against NES_MiSTer before it is ported.** The reason is that Moon's
accuracy was never finished: 45 of the corpus's 63 verdict ROMs pass (§3.4), and a line-for-line port copies every
defect into Rust along with every rule. The C# core stays the byte-exact oracle of §3.2–§3.3. The audit's job is to
decide what that oracle should say before it is copied. As on the other consoles, the FPGA core is a referee and not a
grader (`Venus_Referee.md` §0, `Mercury_Referee.md`).

*Nothing below has been audited yet.* §3.7.1 is the provenance read of 2026-09-28. §3.7.2 and §3.7.3 are the rule and
the method. §3.7.4 is a first look at §6's defects, not their audit. §3.7.5 is where the audit starts. The RTL line
numbers are at head `26e2efb`, and each is to be re-read when its rule is audited.

#### 3.7.1 The checkout, and where each part came from

The checkout is `github.com/MiSTer-devel/NES_MiSTer`, cloned with full history to `~/Projects/nes-mister-reference`:
644 commits from 2017-06-14 to 2026-09-16, head `26e2efb`. It is based on Ludvig Strigeus's fpganes (`README.md:3`),
and his 2012–2013 copyright header survives on `rtl/nes.v`, `ppu.sv`, `video.sv`, `dsp.v` and `compat.v`. His *code*
mostly does not. Measured by `git blame -M -C` against the initial commit `7f2b42f`, 42 of `ppu.sv`'s 2,366 lines
survive, 21 of `nes.v`'s 1,178, 18 of `apu.sv`'s 1,290 and 7 of `MMC3.sv`'s 1,262. What survives is structure and
module names.

| Part | Lineage | What agreement is worth |
|---|---|---|
| CPU (`rtl/t65/`) | FPGAARCADE's general-purpose T65(b), "Ver 315 SzGy April 2020" (`T65.vhd:1-12`), validated against the Lorenz and VICE suites on a C64 core. It replaced fpganes's CPU in `9deb6cd` (2018-11-09). The NES edits are few: undocumented-opcode operands (`12d8d52`), the reset `S` (`b61cea7`), `KIL` (`3490cf5`) and the cold reset. Its RDY halts reads only (`T65.vhd:284`), which is the generic 6502 rule. | **Weak**, as with the Game Boy core's borrowed T80 (`Mercury_Referee.md`): its rules are the 6510's, graded on another machine |
| The CPU's wrapper, DMA and the bus (`nes.v`) | fpganes's `DmaController` skeleton and its comment (`nes.v:4-19`). The logic was rewritten in `06a733f` (2019) and `5e480a7` (2025). The internal/external bus split and the DMC's bus conflicts came in `39ab619` (2026-09-15), and each commit of that series names the AccuracyCoin sub-test it fixes | **Real support**; the grader behind it is a ROM Moon's corpus lacks (§3.7.5) |
| PPU (`ppu.sv`) | fpganes's module structure (`ClockGen`, `SpriteSet`, `BgPainter`, `PixelMuxer`). About 2,050 of its lines are by one author, Kitrinx. Visual 2C02 is cited for two timings (`ppu.sv:243`, `:249`). The OAMDATA comment at `:841-845` reads as nesdev's text. The 2026-09 fixes (`21d9bd3`, `b36e519`, `4e00172`) are graded by AccuracyCoin | **Real support**, except where it follows Visual 2C02 or nesdev and Moon followed the same source |
| APU (`apu.sv`) | Rewritten in 2020 (`7b2422d`). The noise tables are "read directly from the netlist" (`apu.sv:441`), and the frame counter is the 2A03's LFSR (`:785`; the PAL numbers are "educated guesses") | **Real support**: where it follows the netlist, that is the chip's own evidence rather than another emulator's |
| Boards (`rtl/mappers/`) | Split out of fpganes's `mmu.v` in `4d54c68` (2019). MMC1 and MMC2 descend from fpganes. MMC3 does too, heavily reworked, and its clone boards cite Nintendulator (`MMC3.sv:154`, `:377`). The rest are by several contributors, from nesdev | **Weak** where both implement the nesdev wiki's description of a board; **real** where the RTL's shape is visibly the hardware's (the A12 filter counters) |
| Composite encoder (`26e2efb`, "Netlist accurate composite encoder for the PPU") | Video output only: sync and burst windows, and a DAC and phase model. It names no netlist node and touches no emulation logic | **None** for Moon, which emits palette RGB |

**A negative result.** A grep of `rtl/` for Mesen's, Nestopia's, puNES's and Visual 6502's identifiers found none. So,
unlike `N64_MiSTer`'s rasteriser (`Mars_RdpReferee.md`), this core's agreement is not cheapened by borrowed names,
except in the CPU and where a comment names its source. The weighting is closer to the SNES core's than to the N64
core's.

#### 3.7.2 The reading rule

1. **Provenance first.** A rule is weighted by where the RTL got it (§3.7.1), and it is read only after that is
   established.
2. **Shared names mean a shared algorithm.** Where the RTL and Moon carry the same identifiers, or the same source's
   wording, agreement is weak and disagreement is the informative direction. The T65 CPU is the standing case.
3. **"Not implemented" is a real answer.** Three examples from the provenance read:
   - MMC1's WRAM-disable bit is not honoured (`MMC1.sv:195`).
   - FME-7's 5B envelope is absent (`Sunsoft.sv:380`, "not used in any games").
   - The triangle's ultrasonic output is held at its last sample on purpose (`e4b944b`, 2019, "Fix HDMI audio
     artifacts"; `apu.sv:299`). That one is a deliberate departure, so it is no referee point at all.
4. **A referee, never a grader.** The RTL is read for the one rule in dispute. Nothing simulates it, and no test
   compares against it.
5. **Only a hardware test ROM settles anything.** The corpus (§3.4) is the stronger oracle wherever it covers the rule.
   An RTL answer with no ROM behind it leaves a defect *candidate*.
6. **The RTL's save state is not the console.** What its states carry says nothing about hardware (D2 and D3 below).

#### 3.7.3 The method, component by component, from stage 2

1. Before a component is ported (the CPU, the bus and DMA, the PPU, the APU, each board), list the rules where the C#
   core and the RTL could differ, and read both.
2. Record each disagreement in a referee table in the style of `Venus_Referee.md` §2. Each row gives:
   - the rule;
   - Moon's lines and the RTL's lines;
   - the lineage weight from §3.7.1;
   - what the corpus covers;
   - the verdict.
3. **Demonstrated** means a test fails on the unmodified build. A disagreement demonstrated to be a C# defect is fixed in
   the C# core first, with that test, and then ported. The C# core stays the byte-exact oracle, and the fix lands in both
   engines.
4. An **argued** disagreement is recorded as a defect candidate, with no fix.
5. §6's defects enter the same table, with the RTL's answer beside them (§3.7.4).

**P7, stated before the audit:** most disagreements will be in the PPU and in the DMA and bus, where the RTL changed in
2025–2026 against AccuracyCoin. Few will be in the CPU, where both engines implement the same public 6502 rules and
Moon's passes SingleStepTests (§3.6).

#### 3.7.4 §6's defects, with the RTL's answer (a first look, 2026-09-28)

| Defect | The RTL | What that is worth |
|---|---|---|
| D1, DMA steps the APU and not the PPU | DMA halts the CPU through RDY alone (`nes.v:435-436`, `pause_cpu` at `:65`). The APU's enable is the CPU's, ungated (`apu_ce = cpu_ce`, `:213`, `:511`). The PPU runs off its own divider, which `pause_cpu` never enters (`:240`, `:298`, `:625`). So **both advance through every DMA cycle**: the RTL agrees with Moon's APU and disagrees with Moon's PPU | Strong in kind: one master clock is how the console is built, not an algorithm anyone borrowed. It corroborates §6.1's reading that the missing cycles are the PPU's, the half `Moon_Memory.md` §4.8 records as compensating for the boards clocked on the CPU. So the fix is in the frame loop, as Q3 says, and it is not made here |
| D2, a mid-frame state lacks the evaluated sprites | The RTL's state carries the evaluation's pointers and counters (`regs_savestates.sv:14`, `ppu.sv:586-609`). It does not carry secondary OAM, which is reset to `$FF` on load (`:622-623`), or the sprite shifters | Not a hardware question (rule 6). The RTL's own mid-frame states have the same class of omission |
| D3, the pads' shift registers are not state | Not saved either: `joy_out` and `joy_latch` (`nes.v:583-590`) are outside the top-level state words | Not a hardware question |
| D4, an image with no PRG throws | The loader refuses only a missing magic or a trainer (`NES.sv:1577`). A zero-PRG image loads with a zero PRG mask (`:1466-1467`) | No answer: the RTL does not handle the case |

#### 3.7.5 The first rules to audit

All sixteen of Moon's boards exist in the RTL (`cart.sv`), so no board is without a referee. The corpus has no
AccuracyCoin, which is the ROM the RTL's 2025–2026 fixes were graded by (Q7).

- **CPU** (weak evidence):
  - interrupt sampling delayed by a taken branch without a page cross (`T65.vhd:577-582`);
  - IRQ sampled while RDY is low (`:576-578`);
  - `SHA`, `SHX`, `SHY` and `SHS` under RDY and across a page (`:5-6`, `:411-415`);
  - `KIL` (`T65_MCode.vhd:579-591`).
- **DMA and the bus:**
  - the halted CPU re-drives its read address, so a DMC fetch repeats a `$2007` read or clocks a pad (`nes.v:465`,
    `:634`, `NES.sv:654-658`);
  - a DMC fetch's read of `$4016`/`$4017` drives D0–D4 only (`nes.v:466-468`);
  - `$4015` bit 5 comes from the internal bus (`:428`);
  - OAM DMA's get/put alignment, and a DMC fetch inside OAM DMA (`:54-61`);
  - the internal and external open-bus latches (`:832-869`).
- **PPU:**
  - the `$2002` read that lands on vblank's set (`ppu.sv:1896-1897`, `:1998`);
  - the odd-frame skip decided on the pre-render line (`:329`);
  - sprite 0 hit at x = 255 and in the left clip (`:1673-1681`);
  - sprite overflow's diagonal increment (`:752-756`);
  - `$2004` during rendering, and the attribute byte's `$E3` mask (`:730`, `:846-850`);
  - OAM row corruption when rendering starts (`:663-675`);
  - a `$2007` access during rendering incrementing coarse X and Y (`:88`, `:98`);
  - palette writes while rendering (`:1824`);
  - open-bus decay (`:2003-2060`);
  - register writes ignored before the first pre-render line (`:1402`).
- **APU:**
  - the `$4017` write's delay and its immediate clock (`apu.sv:800-802`, `:816-819`);
  - a length-counter halt written on a clock (`:44-51`);
  - the sweep's mute (`:167`, `:225`);
  - the DMC enable's pipeline to its first fetch (`:556`, `:595-597`);
  - `$4011` against the output clock (`:639-641`).
- **MMC1:**
  - the second of two consecutive writes ignored (`MMC1.sv:107-125`);
  - the reset write ORing `$0C` (`:108-110`);
  - SUROM's A18 from the CHR register (`:163`).
- **MMC3:**
  - the counter's reload (`MMC3.sv:425`), and new against old behaviour (`:382`, `:409`, `:672`);
  - the A12 filter, about three M2 cycles low (`:668`, `:682-683`), which is Moon's own three
    (`Mmc3.A12MinimumLowDots`);
  - `$C001` and `$E000` (`:522-523`).
- **The other boards:**
  - bus conflicts only on CNROM and a few others (`nes.v:689`, `generic.sv:1317`), none on UxROM, AxROM, Color
    Dreams or GxROM;
  - MMC2's latch triggers: exact on `$0FD8`/`$0FE8`, and a range on `$1FD8`–`$1FDF` (`MMC2.sv:145-146`);
  - RAMBO-1's A12 filter, which the RTL counts in 16 M2 cycles (`MMC3.sv:79-91`). Moon's is 30 dots, ten CPU cycles
    (`Rambo1.A12MinimumLowDots`), the first visible difference. RAMBO-1's CPU mode, the reload of `latch | 1` and the
    IRQ a cycle late follow at `:151-180`;
  - FME-7's IRQ, cleared only when its trigger bit is written 0 (`Sunsoft.sv:108`);
  - Irem H3001's counter (`misc.sv:1175-1181`);
  - NINA-003/006's decode (`generic.sv:893`).

### 3.8 Stage 2a's audit: the CPU and the bus (2026-09-28)

*The first component audited by §3.7's method. The prediction on record was P7 (§3.7.3): disagreements would gather in
the DMA and the bus, and few would be in the CPU.* The witnesses were:

- AccuracyCoin's CPU- and bus-side failures (§3.4.1), each error code decoded from its source;
- the corpus's transcript, line by line (§3.4);
- NES_MiSTer's RTL, weighted by §3.7.1's provenance, with its T65 CPU as weak evidence.

**What the decoding found first.** Most of the CPU- and bus-side failures are not separate defects. Sixteen AccuracyCoin
results depend on one routine, the DMC-DMA cycle sync (`DMASync`, `AccuracyCoin.asm:18809-18839`). It assumes the rate
15 DMA arrives every 432 CPU cycles, and that a read cycle's stall is halt, dummy, alignment and get. Moon passes the
sync's pre-test, that a DMA puts its byte on the bus (`result_DMCDMASync_PreTest` = 1). It fails every check of the
cycle a DMA lands on:

- DMA + open bus, `$2007` read and write, `$4015` and `$4016` reads, and the DMC's bus conflicts;
- DMC DMA with OAM DMA, and both aborts;
- instruction timing, implied dummy reads, and the interrupt-flag latency's test 8;
- all five SH* results (code 7, RDY two cycles before the write).

So these are **cascades of one model**, not sixteen rules.

#### 3.8.1 The referee table

Weight is §3.7.1's lineage for the RTL's lines. "Witness" is the hardware test ROM that covers the rule, where one does.

| # | Rule | Moon (C#) | NES_MiSTer | Weight | Witness | Verdict |
|---|---|---|---|---|---|---|
| R1 | A taken branch delays an IRQ that just arrived | `Cpu.SuppressJustArrivedIrq` (`Moon_CPU.md` §5.5) | IRQ and NMI sampling held during a branch (`T65.vhd:576-584`) | weak | `5-branch_delays_irq` passes; AccuracyCoin's latency tests 1–7 pass | **agree** |
| R2 | IRQ sampled while RDY holds the CPU | no RDY: a DMA never halts the CPU mid-instruction | "detect irq even if not rdy" (`T65.vhd:576-578`) | weak | latency test 8 (a cascade of R7) | not comparable until R7 |
| R3 | SH*'s high-byte term dropped when RDY falls before the write | `UnstableStore` has no RDY input | `rdy_mod` set by RDY low at cycle 3 (cycle 4 for `$93`, `T65.vhd:411-415`), from VICE's tests (`:5-6`) | weak | SH* code 7, five results | **disagree**, candidate C4, needs R7 |
| R4 | `KIL`/`JAM` wedges the CPU | three cycles, then the bus held (`Moon_CPU.md` §6.4) | `KIL` in the microcode (`T65_MCode.vhd:579-591`) | weak | SingleStepTests' vectors (§3.6) | **agree** |
| R5 | An NMI in the first four cycles of `BRK` or of an IRQ takes their vector | none: the vector is chosen before the pushes | `NMI_entered` switches `BRK`'s vector (`T65.vhd:666-675`, `:780`, `:794-796`) | weak, but the rule is the 6502's own documented hijack | `2-nmi_and_brk` and `3-nmi_and_irq` fail 1; AccuracyCoin `$462` and `$463` fail | **disagree**, candidate C1 (below) |
| R6 | The DMC's period is the rate table's value | `StepTimer` reloads with the value and counts to 0, so a bit lasts value + 1 cycles: 440 per byte at rate 15, not 432 | an LFSR (`apu.sv:547-553`, `:593-605`, `:647-648`); stepping its reload values to `$100` gives 213 steps at rate 0 and 26 at rate 15, and with the reload 214 and 27 APU cycles, exactly 428 and 54 CPU cycles | real (netlist) | `8-dmc_rates` fails 3, and passes with the fix | **defect D5, demonstrated; fix not committed** (§3.8.3) |
| R7 | A DMC fetch halts the CPU on a read cycle: halt, dummy, alignment, get | a fetch lands inside the cycle's tick, the CPU is not halted, and a flat 4 cycles are charged afterwards (`DmcChannel.cs`, `MemoryBus.Tick`) | `DmaController` holds RDY (`nes.v:21-72`); `dmc_state` starts only on a CPU read and a put cycle (`:54`) | real (AccuracyCoin-graded) | the sixteen cascades above | **disagree**, candidate C2, the next CPU and bus work |
| R8 | A halted CPU re-drives its address, so a fetch during a read repeats it | nothing is repeated | the address bus falls back to the CPU's (`nes.v:465`); the pads clock on a falling edge of their select (`NES.sv:654-658`) | real | DMA + `$2007` read and write, `$4016` read (cascades) | part of C2 |
| R9 | A DMA's read of `$4016`/`$4017` drives D0–D4 only | not reachable: no DMA reads the pads | `dma_data_bus` (`nes.v:466-468`) | real | none until C2 | part of C2 |
| R10 | `$4015` answers on the internal bus: bit 5 floats, the external bus keeps its value | the full status returned and written to the bus latch, bit 5 zero | `apu_reg_value` fills bit 5 from `internal_bus_data`; `open_bus_data` takes the external bus (`nes.v:424-428`, `:847-848`) | real | AccuracyCoin open bus codes 7 and 9 | **defect D8, fixed** |
| R11 | Nothing answers a read of `$4020`–`$5FFF` on these boards | PRG RAM's mirror (`address & 0x1FFF`) on every board | open bus unless `prg_allow` (`$6000`+ or a PRG read, `nes.v:853-866`, `MMC1.sv:194-195`) | real | AccuracyCoin open bus code 1; `test_cpu_exec_space_apu` fails 2 | **defect D6, fixed** |
| R12 | Two bus latches, internal and external | one latch, `OpenBus` | `open_bus_data` and `internal_bus_data` (`nes.v:841-850`) | real | AccuracyCoin internal data bus code 1 (a cascade of C2 and D6) | candidate C5; the one latch reproduces D8's cases |
| R13 | OAM DMA waits for a get cycle: 513 or 514 cycles | always 513, the copy instantaneous | `spr_state` 1 → 3 only on a CPU read and a get (`nes.v:58`) | real | DMC + OAM DMA (a cascade) | part of C2, and §3.7.4's D1 |
| R14 | `$4016`'s strobe reaches the pads on a put cycle | immediately on the write | `joy_out <= joy_latch` on `put_ce` (`nes.v:583-590`) | real | controller strobing code 4 | **disagree, demonstrated**; candidate C6, needs get/put parity |
| R15 | DMA leaves the PPU running (D1) | the PPU does not see OAM DMA's 513 cycles | `ppu_ce` independent of `pause_cpu` (`nes.v:240`, `:298`) | real | §6.1's measurement | recorded in §3.7.4; lands with C2 |

**P7 held.** Of the five CPU rules, R1 and R4 agree, R2 and R3 wait on the DMA model, and only R5 disagrees on its own.
Of the ten bus and DMA rules, eight disagree. As P7 said, the disagreements are in the DMA and the bus. The CPU's
disagreement is the one place the documented rule and the RTL both say Moon is short.

#### 3.8.2 What was fixed, and what it changed

| Defect | Test that fails on the unmodified core | Witness that moved |
|---|---|---|
| **D6** reads of `$4020`–`$5FFF` float | `MoonCpuBusDefectTests.A_read_of_the_expansion_area_is_open_bus` | `test_cpu_exec_space_apu`: fail 2 → pass; AccuracyCoin open bus: fail 1 → fail 7 |
| **D8** `$4015` floats bit 5 and leaves the bus | `MoonCpuBusDefectTests.A_read_of_4015_floats_bit_5_and_leaves_the_bus` | AccuracyCoin open bus: fail 7 → pass |

- **Where the fixes are:** both are in `MemoryBus.Read`, and MoonRT's `bus.rs` carries the same two edits.
- **Writes:** D6 fixes reads only. A write to `$4020`–`$5FFF` still reaches the board, which must see it because
  NINA-003-006's register is at `$4100`. Most boards also store it in PRG RAM's mirror, which is wrong in the same way
  as the read was. No witness covers it, so it is candidate C7.
- **Save state:** the format is unchanged.

**Output changes (measured 2026-09-28, against `10c59639`):**

- **The four bench games:** identical state hashes at frames 900 and 3,900 with the bench's input:
  - Super Mario Bros. `E3BFD7E43ACA761B` / `2DA2BBA9743B909E`;
  - Zelda `320DCAE6E4073688` / `E61F6EAA163F691C`;
  - Super Mario Bros. 3 `258BA3EB5CA8637D` / `F210BBD8B96E2350`;
  - Punch-Out!! `A34AC61F9177AC3A` / `84BB41D777C97B56`.

  None of them reads the expansion area or depends on `$4015`'s bus behaviour.
- **The corpus:** of 263 ROMs, one line changed, `test_cpu_exec_space_apu`, from fail 2 to pass. Every other line's
  verdict, frame, resets and text hash is the base's. The count is 90 passed, 24 failed, 141 without a verdict and 8
  unsupported, against 89, 25, 141 and 8.
- **AccuracyCoin:** 89 → **90 of 144**. The open bus test passes. No other result byte changed, and the table is drawn
  at frame 4,117 instead of 4,099, because the open bus test now runs to its end.

**MoonRT.** Both fixes were carried into the Rust bus, and MoonRT was proven against the fixed C# core:

- `MoonRtMachineTests`, 72 cases: random programs compared after every instruction, undocumented opcodes and JAM,
  interrupts, OAM DMA and the DMC's stolen cycles, among the rest;
- `MoonRtRandomProgramTests`, 32 cases;
- `MoonRtStateTests`, 71 cases;
- AccuracyCoin through MoonRT (`MoonAccuracyCoinTests.MoonRT_scores_as_the_csharp_core_does`): every result byte and the
  table's frame match C#'s;
- all 324 Moon-filtered cases pass.
- the corpus on both engines (`MoonRtCorpusTests`, from `EMUSEN_MOONRT_CORPUS`): 255 ROMs run and 8 refused by both; 362,375 frames with identical state; 114 verdicts, 90 passed, the same as C#'s.

This stage's claim is the CPU and the bus. The same classes cover MoonRT's boards, PPU and APU, and those pass too, but
they are later stages' to claim, after their own audits.

#### 3.8.3 Negative results

- **The DMC period fix is correct, and cannot land alone.** With D5, `8-dmc_rates` passes, but AccuracyCoin stops at
  test 82, the interrupt-flag latency. The trace shows why:
  - The test's last case runs `JSR $4013`. It expects a DMC DMA to land on the opcode fetch there and put `$90`
    (`BCC`) on the bus, whose target reads `$60` (`RTS`) from the PPU's read buffer (`AccuracyCoin.asm:9500-9530`).
  - With the period right, the DMA arrives close enough to matter. But Moon's DMA does not halt the CPU (R7), so the
    fetch's byte feeds the wrong access.
  - The CPU then executes into PPU registers (a write at `pc=$3125`) and never returns.

  The wrong period had been compensating for the missing halt: it kept the sync from ever succeeding. So D5 and C2
  must land together. The one-line fix and its tests are kept out of the tree
  (`~/.cache/emusen/probe/moonrt/stage2a/d5-dmc-period.patch`) until C2 exists.
- **The hijack fix (C1) moves no witness.** A prepared patch implements R5, and its unit tests pass. But
  `2-nmi_and_brk`, `3-nmi_and_irq` and AccuracyCoin `$462`/`$463` still fail with it. AccuracyCoin also fails four of
  its vblank-timing tests (NMI timing, suppression, at vblank's end, disabled at vblank), all code 1. So the NMI itself
  arrives on the wrong dot, and both witnesses wait on that, which is the PPU's audit.
  - By §3.7.3's rule, a fix no hardware test confirms is a candidate, so C1 was not committed
    (`stage2a/d7-nmi-hijack.patch`).
  - `Moon_CPU.md` §5.6 said this was blocked because the PPU advanced a scanline at a time. The PPU now steps three dots
    a CPU cycle, so the hijack is expressible. What blocks it now is the NMI's timing, not its granularity.
- **Stage 2a's prediction of its own cost was right in kind and wrong in size.** Expected: several small CPU fixes.
  Found: two small bus fixes, and one model (R7) behind most of the rest.

#### 3.8.4 What is left for the CPU and the bus, in order

1. **C2: DMC DMA as a halt on the CPU's read cycles, with get/put parity**, together with D5's period and R13's OAM DMA
   alignment. Its oracle is the sixteen cascades, the corpus's `dmc_dma_during_read4`, `sprdma_and_dmc_dma` and
   `4-irq_and_dma`, and the four games' hashes, which it will change.
2. **C6: the strobe on put cycles.** It needs the parity C2 introduces, and its witness is controller strobing code 4.
3. **C1: the hijack**, after the PPU's NMI timing.
4. C3 (R2), C4 (R3), C5 (R12) and C7 (writes to `$4020`–`$5FFF`) are measured by C2's witnesses once it exists.

**Reuse.** `AccuracyCoinRun` takes any NES core's frame, RAM read and Start button, which is how MoonRT runs it. Nothing
in the CPU or the bus was found that a second core shares: Mercury's bus and the SM83 have none of these rules.

### 3.9 Stage 2b: DMA as a halt of the CPU (2026-09-29)

*Candidate C2 of §3.8.4, with D5's period and R13's alignment.* The prediction on record, from §3.8.3, was that D5 and
C2 must land together. It held: with both in, AccuracyCoin runs through, and D5's `8-dmc_rates` passes with no
regression elsewhere.

#### 3.9.1 The model

**A DMA waits for the CPU's next read cycle and halts it there.**

- The halt is `Cpu.Read`'s first step. Writes are never halted, as T65's RDY is read-only (`T65.vhd:284`).
- Each halted cycle is a full cycle: the PPU, the APU and the board are clocked, the CPU's cycle count and the
  instruction's cycles advance, and the interrupt lines are sampled, as T65 samples them "even if not rdy".
- So OAM DMA's 513 or 514 cycles now pass for the PPU too. §6.1's D1 goes with it (§3.9.3).

**The DMC's fetch.**

- The DMC now has a real sample buffer (`SampleBuffer`, `BufferFull`).
- The output unit takes the buffer at the end of each byte, and an empty buffer with bytes left asks for a fetch
  (`DmaRequested`).
- The fetch is a halt, a dummy, an alignment cycle when the next would be a put, and the get that reads the sample:
  3 or 4 cycles. That is the sequence `DMASync` assumes (`AccuracyCoin.asm:18832`), and the RTL's `DmaController`
  starts only on a CPU read (`nes.v:54`).
- The halt, dummy and alignment cycles repeat the CPU's read with all its side effects. That is R8: `$2007` advances v,
  and `$2002` clears vblank.

**Enabling the DMC.** A `$4015` write that enables it with an empty buffer asks for the first byte after 2 cycles on
a get and 3 on a put. That is the RTL's enable pipeline (`apu.sv:556`, `:595-597`) read as a delay, and it is the
first number here calibrated against a witness, DMA + `$2002` read. Its `STA $4015 / LDA $2002` needs the halt on the
`LDA`'s read.

**OAM DMA.** A halt, one alignment cycle when the next would be a put, then a get read and a put write per byte, 513
or 514 cycles (R13; `nes.v:58`). A DMC request during the copy takes the next get, and the copy realigns on the put
after it.

**Get and put** are the APU's cycle parity that `$4017`'s write delay already used (`Apu.IsGetCycle`), so the DMA,
the frame counter and the pads agree by construction.

**The pads.**

- **A read of `$4016`/`$4017` on the cycle after one of the same register neither clocks the pad nor sees a new bit.**
  The pad clocks on its select's falling edge (`NES.sv:654-658`), so a held read sees the bit the first read saw.
- **A CPU halted on `$4000`–`$401F` keeps the 2A03's registers enabled through the get.** The fetch's low five bits then
  select one too: a fetch at `...16` holds `$4016`'s select, and one at `...15` reads `$4015`.
- **`$4016`'s OUT0 reaches the pads at the end of a get cycle** (C6). This is the edge the RTL calls `put_ce`, which
  latches `joy_out` (`nes.v:583-590`).

**The save state is version 4.** It is version 3's walks plus a tail of 8 bytes:

- the DMC's buffer, its full flag and the load delay;
- a pending OAM DMA and its page.

A version 3 state still loads, with an empty buffer, so its reader asks for its byte at once. MoonRT reads and writes
the same tail. `MoonRtStateTests.A_version_3_state_loads_in_both_engines_alike` builds a version 3 state from a
version 4 one and loads it into both engines: they agree when loaded, and again after ten frames.

- **Kept as always zero:** `StallCycles`, `PendingDmaCycles` and `StolenCycles`. No DMA charges cycles afterwards any
  more, and keeping them leaves version 3's layout untouched. They are candidates for removal at the next format change.
- **Not in the state:** the pads' held-read cycle and OUT0 latch, like the pads' shift registers (§6.2's D3). A state
  loads with no read on the cycle before it.

#### 3.9.2 What it fixed, and the proofs

**New tests, each failing on the unmodified core** (`MoonCpuBusDefectTests`):

- **D1:** `An_oam_dma_leaves_the_frame_its_length_and_sound`. Before the change a frame with OAM DMA made more sound
  than one without, as §6.1 measured (746.43 against 733.80).
- **C2 and R8:** `A_dmc_fetch_repeats_a_halted_read_of_2007`. Unmodified, v moved 3,964 for 3,963 counted reads, which
  is no repeat at all.
- **D5:** `A_dmc_byte_lasts_eight_periods_of_its_rate`, three rates. Its version against version 3's DMC API failed at
  all three rates, before that API changed (§3.8.3).
- `MoonCoreTests`: the OAM DMA test now checks the copy under a halted CPU in 513–514 cycles, and the controller test
  gives the strobe two cycles to land.

**AccuracyCoin: 90 → 102 of 144** (measured 2026-09-29). The results table is drawn at frame 3,061, and the menu is
ready at frame 17 instead of 23. Of the tests that depend on `DMASync` (§3.8):

| Now pass | Still fail, with the code now |
|---|---|
| DMA + open bus, DMA + `$2007` read and write, DMA + `$4015` read, DMA + `$4016` read, instruction timing, implied dummy reads, interrupt-flag latency | DMC bus conflicts (1 → 2, "did not correctly emulate the bus conflict with the APU registers"), DMC + OAM DMA (1 → 2, the overlapped cycle count), explicit and implicit abort (1 → 2, the aborted cycle count), SH* ×5 (7, C4) |

- **Passing besides:** DMA + `$2002` read, the frame counter's 4-step and 5-step tests, and controller strobing (C6).
- **Moved but still failing:**
  - the Delta Modulation Channel test, from code 12 to 15;
  - internal data bus, from 1 to 2 (the bus conflict with `$4015`);
  - APU register activation, from 1 to 4 (OAM DMA reading APU registers).
- **The code-1 failures are gone.** Every remaining DMA failure is past the "timing is off" check. What is left is
  the conflicts' data and the overlapped and aborted cycle counts.

**Negative results on the way.**

- The first held-read rule returned the next bit. That broke controller clocking, code 5: a double read must see the
  same value.
- The first conflict rule selected the register by the halted CPU address. That broke DMA + `$4016` read.
- Moving the OAM DMA's alignment to reads on puts fixed the strobe and broke both frame-counter tests. The alignment
  was right and the strobe's edge was wrong: the pads take OUT0 at the end of a get.

All three were caught by AccuracyCoin between runs, and none is in the tree.

**The corpus:** of 263 ROMs, 90 → **94 passed**, with no previously passing ROM failing:

- `apu_test` passes, from fail 1;
- `7-dmc_basics`, from fail 19;
- `8-dmc_rates`, from fail 3;
- `4-irq_and_dma`, from fail 1;
- `sprdma_and_dmc_dma` and its 512 variant still fail 1, with changed text.

On both engines (`MoonRtCorpusTests`) the corpus runs identically: 255 ROMs run and 8 refused by both; 362,421
frames with identical state; 114 verdicts, 94 passed.

#### 3.9.3 The games: hashes changed on purpose, and still played

**D1 is gone.** Every game in the playability run emits 733.79 stereo samples a frame, the rate's own 733.80, against
737.8–747.7 before (§1.1).

**New state hashes**, frames 900 and 3,900 with the bench's input:

| Game | Frame 900 | Frame 3,900 |
|---|---|---|
| Super Mario Bros. | `DAC809EDDF58F2EC` | `0C7C41E0DF6B6C15` |
| Zelda | `66FC06B58F8F81E7` | `B97DD6CE3362CB82` |
| Super Mario Bros. 3 | `1C6FE76307F71AF0` | `42DF99FC109B4757` |
| Punch-Out!! | `A0A79292F5B2AC34` | `46CF6D9898B447D0` |

Every change is expected, since every game runs OAM DMA every frame and it now costs the PPU its cycles too.

**Playability** (`moontruth play`, in the scratch directory):

- **What was run:** the four games and the ROMs of the library's eleven Moon states. The ROMs were copied to scratch,
  reading the library only.
- **From boot:** 3,600 frames with the bench's input.
- **From each state:** 1,800 frames. All the states were written as version 3, so this is also the check that old
  states load.

**Result:** no fault and no JAM, except one. The picture changed in most seconds of every run. The exception is an
SMB1
hack, *Mario's Adventure*:

- its resume state JAMs, and from boot its picture changes in 1 of 60 seconds;
- the unmodified build (`603567de`) does exactly the same.

So it is not this change, and it is recorded as a finding to investigate apart.

**MoonRT against the fixed core.** All of these pass:

- 331 Moon-filtered cases, which include:
  - `MoonRtMachineTests`' random programs after every instruction, undocumented opcodes and JAM, interrupts, OAM DMA
    and the DMC's fetches;
  - the real games from boot and from a transferred state;
  - the state oracles;
  - AccuracyCoin, result for result, with the table at the same frame.
- The crate's 6.

#### 3.9.4 What is left

1. **SH* under RDY (C4):** five results, code 7. The halt now exists to test it.
2. **The APU-register conflict's data.** DMC bus conflicts code 2, internal data bus code 2, and APU register activation
   code 4 (OAM DMA reading `$4000`–`$401F`).
3. **The overlapped and aborted cycle counts:** DMC + OAM DMA, and explicit and implicit abort, each code 2.
   - The DMC's slot inside an OAM DMA is simplified here to one get and one realignment put.
   - A `$4015` write that disables the DMC mid-fetch cancels it outright.
4. **C1, the NMI hijack,** after the PPU's NMI timing (§3.8.3).
5. **Reuse:** nothing here is shared with another core. Mercury's DMA is its own, and MarsRT is untouched.

### 3.10 Stage 2c: the PPU and the NMI (2026-09-29)

*The PPU's audit by §3.7's method. The prediction on record, from §3.8.3, was that C1, the NMI hijack, waited on the
NMI's timing. It held: with the NMI sampled where the RTL samples it, C1's four witnesses pass.* The decoding of
AccuracyCoin's PPU failures, read from its source before anything was changed, found two roots behind nearly every
code 1, and both were confirmed by fixing them:

- **OAMADDR.** Hardware holds OAMADDR at 0 through the sprite fetches of every rendered line. Moon never did, so a test
  that left it nonzero (`$2003` in the PPU open-bus test) shifted every later OAM DMA, and sprite 0 became `$FF`.
  That one rule made the Sprite 0 Hit test fail, and with it every test that first checks a hit.
- **Where the CPU's access and the NMI's sample fall in a cycle.** Moon clocked all three dots, sampled /NMI, then made
  the access. So a `$2002` read or a `$2000` write could never affect the NMI of its own cycle, and every blargg NMI
  table was one dot out.

#### 3.10.1 The referee table

| # | Rule | Moon (C#) before | NES_MiSTer | Weight | Witness | Verdict |
|---|---|---|---|---|---|---|
| P1 | OAMADDR is 0 through dots 257–320 of a rendered line | never reset | `oam_addr <= '0` while `cycle < 320` in the sprite fetches (`ppu.sv:802-803`) | real | AccuracyCoin page 19's code 1s | **defect, fixed** |
| P2 | `$2004` reads `$FF` while secondary OAM clears, dots 1–64 | `Oam[OamAddress]` | during the clear `oam_data <= 8'hFF` (`ppu.sv:691`), which is what `$2004` reads (`oam_bus`, `:526`) | real | Address `$2004` behaviour, code 4 → 6 | **defect, fixed** |
| P3 | The CPU's access falls after the cycle's second dot, and /NMI is sampled after the third | three dots, /NMI, then the access | NMI is `nmi_occured && vbl_enable` (`ppu.sv:1998`), and the `$2002` read clears it with `set_nmi = entering_vblank & ~clear_nmi` (`:1896-1897`); where the CPU's access falls was fitted to blargg's tables | real, fitted | `05-nmi_timing` … `08-nmi_off_timing`; AccuracyCoin NMI timing, suppression, at vblank's end, disabled at vblank | **defect, fixed** |
| P4 | The odd frame's skipped dot is decided at dot 338 | rendering tested at 339 | `if (cycle == 338) skip_next <= … is_rendering` (`ppu.sv:328-330`) | real | `10-even_odd_timing`, fail 3 → pass | **defect, fixed** |
| P5 | `$4015`'s frame-IRQ bit clears at the next get after a read; the IRQ line at once | one flag for both | `FrameInterrupt` cleared on the read, `frame_interrupt_buffer` only on `aclk1` (`apu.sv:808-822`) | real | DMA + `$2002` read (its `SLO $4015,X` sync); frame counter IRQ, code 7 → 13 | **defect, fixed** (an APU rule, needed here) |
| C1 | An NMI in BRK's or an IRQ's first four cycles takes their vector; the sequence ends without polling | none | `NMI_entered` (`T65.vhd:666-675`, `:780`) | weak | `2-nmi_and_brk`, `3-nmi_and_irq`, AccuracyCoin `$462`/`$463` | **defect, fixed**; the no-poll half was found by the corpus's last rows |
| P6 | Sprite 0 hits on its pixel's dot | at dot 256, when the line is composed | per-pixel shifters (`ppu.sv:1673-1681`) | real | `$2002` flag timing; Rendering Flag Behaviour | **candidate**; a look-ahead was built and measured, and moved no witness (§3.10.3) |
| P7 | `$2002`'s sprite flags latch about 1.9 dots after vblank's | one instant for all three | the asm's own account (`AccuracyCoin.asm:1776-1781`) | — | `$2002` flag timing | candidate |
| P8 | Background and sprite shift registers, serial in, stale shifters, ALE with `$2007` | a scanline renderer: the line composed at dot 256 from `RenderV` | per-dot pipelines (`ppu.sv` BgPainter, SpriteSet) | real | Advanced BG and sprite evaluation, `$2004` and `$2007` stress | **disagree by design**: Moon has no per-dot pipeline; the next PPU work |
| P9 | Secondary OAM, its overflow flag and misaligned addresses | none: sprites evaluated from slot 0 at dot 256 | `oam_temp`, `oam_secondary_ovr` (`ppu.sv:537`, `:744-758`) | real | Frozen OAM2, Misaligned OAM2, sprites on line 0 | disagree by design, as P8 |
| P10 | Open-bus decay about 600 ms, per bit | per bit, counted in frames (`Moon_PPU.md` §2.5) | per bit group (`ppu.sv:2003-2060`) | real | `ppu_open_bus` passes | **agree** |
| P11 | Palette writes while rendering | allowed | blocked (`ppu.sv:1824`, `:1838`) | real | none failing | candidate |

**P3's model, and its calibration.** The PPU now runs two dots before the CPU's access and the third after it
(`MemoryBus.Tick` and `EndCycle`), and the CPU samples /NMI at the cycle's end. The split is the one number here fitted
to witnesses: all four of blargg's tables and AccuracyCoin's four NMI tests pass with it, and every one was a dot
out without it. Moving only the sample to the cycle's end, and keeping three dots before the access, fixed the
suppression window's direction but left every table one dot late.

#### 3.10.2 What was fixed, and the proofs

**Tests that fail on the unmodified core** (`MoonPpuDefectTests`):

- `Rendering_returns_oamaddr_to_zero` (P1);
- `A_read_of_2004_while_secondary_oam_clears_is_ff` (P2);
- `The_nmi_and_vblank_timing_roms_pass`: seven corpus ROMs, from `EMUSEN_MOONRT_CORPUS`, unmodified 7 of 7 failing
  (P3, P4, C1).
- P5 has no unit test. Its witnesses are DMA + `$2002` read, which regressed without it, and the frame counter IRQ's
  code.

**AccuracyCoin: 102 → 113 of 144** (measured 2026-09-29; the table at frame 3,602, the menu at 24).

| Page | Now pass | Moved, still failing |
|---|---|---|
| 12 CPU interrupts | NMI overlapping BRK, NMI overlapping IRQ | — |
| 14 APU | — | frame counter IRQ 7 → 13 |
| 18 vblank timing | NMI timing, suppression, at vblank's end, disabled at vblank: **7 of 7** | — |
| 19 sprite evaluation | sprite 0 hit, sprite overflow, misaligned OAM DMA | arbitrary sprite zero 1 → 2, suddenly resize 1 → 5, OAM corruption 1 → 2 |
| 20 PPU miscellany | `INC $4014` | `$2004` 4 → 6, `$2007` read while rendering 1 → 2 |
| 21 advanced BG | attributes as tiles | stale shift registers 1 → 3, ALE + read 1 → 2, hybrid addresses 1 → 2 |

- **What remains of the code-1 cascade:** only tests that need P8's or P9's per-dot pipelines.
- **The DMC load delay was re-fitted.** P5 made the `SLO $4015,X` sync of DMA + `$2002` read work, which §3.9's value
  had been fitted around. The delay is now three cycles on a get and two on a put. Three on both also passes; the
  parity-dependent one is kept for the RTL's pipeline.

**The corpus:** 94 → **104 passed** of 263, with no ROM that passed failing:

- `ppu_vbl_nmi` and its 05, 06, 07, 08 and 10;
- `cpu_interrupts_v2` and its 2 and 3;
- `sprdma_and_dmc_dma`.

On both engines (`MoonRtCorpusTests`) the corpus runs identically: 255 ROMs run and 8 refused by both; 363,934
frames with identical state; 114 verdicts, 104 passed.

#### 3.10.3 Negative results

- **The sprite-0 look-ahead (P6) moved nothing, and was not kept.** At dot 1 of each line, it found sprite 0's first
  opaque pixel over an opaque background, and set the flag on that pixel's dot. It did not read the board's CHR through
  its side effects, since MMC2's latch changes on a read.
  - With it, no AccuracyCoin result improved.
  - Rendering Flag Behaviour went from code 2 back to 1. That test needs background shift registers that clock with
    only sprites on (P8), which a look-ahead from `RenderV` cannot give.
  - It is a candidate for when the per-dot pipeline exists.
- **The first cause of page 19 was not sprite 0's timing.** The decoding's first hypothesis was that the hit came too
  late at dot 256. The source showed the test waits 3,000 cycles, so timing could not matter. A stale OAMADDR was the
  cause.
- **A fix elsewhere exposed an APU defect.** Moving P3's access point made DMA + `$2002` read fail. The test's
  `$4015` sync had never worked (P5), and the test had passed by alignment alone. Fixing P5 restored it.

#### 3.10.4 The games

- **State hashes at frame 900 and 3,900:**
  - Super Mario Bros., Zelda and Punch-Out!! are unchanged from §3.9.3;
  - Super Mario Bros. 3's frame 3,900 is now `7D10D7B97B83D054`.

  The NMI now arrives up to a dot earlier relative to the CPU. That moved no instruction boundary in the first three,
  and one in SMB3's run.
- **Pictures:** before and after PNGs of all four games at both frames are in `~/.cache/emusen/probe/moonrt/stage2c/pictures/`.
  All eight are pixel-identical. SMB3's changed state did not reach the picture by frame 3,900.
- **Playability** (§3.9.3's run): the same results as 2b. There was no fault and no JAM, 733.79 stereo samples a frame,
  and the picture changing throughout. The one exception is again the SMB1 hack's resume state, which the base build
  also JAMs.
- **The save state stays version 4.** The new fields are all out of the state, since none holds a value across an
  instruction boundary that a state would need:
  - the odd-frame skip decided at dot 338 and spent at 339;
  - the hijack's `_vectored`;
  - the readable frame-IRQ bit, which a load sets from the IRQ.

**MoonRT** carries every rule, with the same model and the same fitted values. All of these pass:

- 334 Moon-filtered cases, which include:
  - the machine's per-instruction random programs, interrupts and DMA;
  - the real games from boot and from a transferred state;
  - the state oracles;
  - AccuracyCoin, result for result;
- the crate's 6.

**Reuse:** nothing here is shared with another core, and MarsRT is untouched.

#### 3.10.5 What is left

1. **P8 and P9, the per-dot pipelines**, which the rest of pages 19–22 need. The scanline renderer composes a line at
   dot 256 from `RenderV`, so mid-line effects and stale registers are out of reach. That is a design change, and needs
   its own step: the renderer feeds the machine (§2.5), and the four games' pictures would change.
2. P6 and P7 once P8 exists; P11.
3. From §3.9.4: SH* under RDY, the APU-register conflict's data, and the overlapped and aborted DMA counts.

### 3.11 Stage 2d: the CPU and bus leftovers (2026-09-29)

*§3.9.4's and §3.10.5's CPU and bus list, with §3.9's two simplifications replaced. The prediction on record was that
each needed the halt that 2b built, and nothing more. It held: every rule below is stated in terms of that halt.* As in
the earlier steps, the target tests' expectations were read from the ROM's source before anything was changed. Those
tests are the DMA cycle-count tables, the SH* tables, and the bus-conflict tables.

#### 3.11.1 The rules, and their weight

| # | Rule | Moon before | Source | Witness | Verdict |
|---|---|---|---|---|---|
| C4 | A DMA halting SH*'s dummy read, the cycle before the write, drops the `& (H+1)` term from the stored value; the address is unchanged | always ANDed | T65's `rdy_mod`, set by RDY low at that cycle (`T65.vhd:411-416`), choosing `Write_Data_AX` over `_AXB` (`T65_MCode.vhd:146-178`) | the five SH* results, code 7 | **fixed**; T65 is weak evidence, AccuracyCoin the witness |
| B1 | A DMA's read reaches the 2A03's registers only while the halted CPU's address is in `$4000`–`$401F`, chosen by the DMA address's low five bits; otherwise `$4000`–`$5FFF` reads the bus | an OAM DMA from page `$40` read `$4015` and the pads | `apu_cs` decodes the CPU's address, `joypad1_cs` the muxed one (`nes.v:501`, `:559-560`) | APU register activation, code 4 | **fixed** |
| B2 | Under that conflict a pad read puts the pad's bits under the bus's top three; `$4015` clears the frame IRQ and leaves the external bus alone | the pad's select only was held | `dma_data_bus` (`nes.v:466-468`), `open_bus_in` (`:576`), `apu.sv:808` | DMC DMA bus conflicts, code 2 | **fixed** |
| B3 | The 2A03's internal data bus is a latch of its own: every access loads it, `$4015` reads with bit 5 taken from it; the DMC's get does not load the sample into it when `$4015` answers | one latch | `internal_bus_data` (`nes.v:424-426`, `:836-838`) | internal data bus, code 2 | **fixed** |
| B4 | A DMC fetch inside an OAM DMA runs its halt and dummy alongside the copy, takes a get at least two cycles after its halt, costs the copy a realigning put, and carries on alone if the copy ends first; one request as the copy's halt shares it | the next get, and one put | the ROM's own account (`AccuracyCoin.asm:12677-12696`) and `DmaController` (`nes.v:54-61`) | DMC DMA + OAM DMA's two tables, code 2 | **fixed**; §3.9's simplification replaced |
| B5 | A disabling `$4015` write lets the request fall through the enable's lag (3 cycles on a get, 2 on a put); a fetch commits after its halt, and a request gone by then costs the halt alone | the fetch cancelled outright | `dma_req = ~have_buffer & enable_3` (`apu.sv:556`, `:593-594`, `:642-643`); "a DMC hardware bug can make trigger fall before it's done" (`nes.v:62`) | explicit abort's table, matched value for value | **fixed**; §3.9's simplification replaced |
| B6 | The last byte of a non-looping sample drops the enable and leaves the same lag (3 cycles), so an emptied buffer inside it asks once more | none | `enable <= loop` on the last fetch (`apu.sv:633`), with `enable_3` lagging | implicit abort | **fixed**; its `$500` table was not reproduced by any lag tried (§3.11.3), and the test passes on the other two |

**Two values here are fitted.** B5's lag is the same 3:2 pipeline as §3.10's load delay. Tried against the explicit
abort's table, 2:2, 3:3 and 4:3 each miss by one or two entries, and 3:2 matches all sixteen. B6's 3 was chosen among
2, 3, 4 and 5 as the only one to reproduce the implicit abort's `$520` table exactly.

#### 3.11.2 The proofs

**Tests that fail on the unmodified core** (`MoonCpuBusDefectTests`):

- `A_dma_on_shas_dummy_read_drops_the_high_byte_term` (C4). It uses a flat bus that halts the chosen read, and the
  case without a halt is the control;
- `An_oam_dma_from_page_40_leaves_the_frame_irq_when_the_cpu_is_elsewhere` (B1);
- `A_disable_lets_the_dmc_request_fall_after_the_pipelines_lag` (B5).
- B2, B3, B4 and B6 are measured by AccuracyCoin's tables, which `MoonAccuracyCoinTests` pins byte for byte.
- §3.8's `$4015` test now sets both latches, as every real access does. The rule it checks is unchanged.

**AccuracyCoin: 113 → 124 of 144** (measured 2026-09-29; the table at frame 3,616):

| Page | Now pass |
|---|---|
| 10 SH* | SHA ind,Y, SHA abs,Y, SHS, SHY, SHX: **6 of 6** |
| 13 APU registers and DMA | DMC DMA bus conflicts, DMC DMA + OAM DMA, explicit abort, implicit abort: **10 of 10** |
| 14 APU | APU register activation |
| 15 CPU behaviour 2 | internal data bus |

The DMC channel test moved from code 15 to 21 and still fails; it is the APU's.

**The corpus:** 104 → **105 passed** (`sprdma_and_dmc_dma_512`), and no ROM that passed fails. On both engines it runs identically: 255 ROMs, 363,935 frames with identical
state, 105 passed.

**The games:**

- The four bench games' state hashes at frames 900 and 3,900 are unchanged from §3.10.4, and so are all eight
  pictures. None of the rules is reached by their play: no SH*, no DMA reading `$40xx`, and no disable during a fetch.
- The playability run (§3.9.3) gives the same results as 2c: no fault and no JAM except the SMB1 hack's state, which
  the base build also JAMs.
- The save state stays version 4. The new fields are out of it: the internal bus is set from `OpenBus` on a load, and
  SH*'s halt flag lives inside one instruction.

**MoonRT** carries every rule. All of these pass:

- 338 Moon-filtered cases, including AccuracyCoin result for result;
- the crate's 6.

**Reuse:** nothing here is shared with another core, and MarsRT is untouched.

#### 3.11.3 Negative results

- **Cancelling after every halted cycle was wrong.** The first version of B5 let a fetch be dropped at any of its
  cycles. That gave a cost of 2 where the table wants 1. The table fits a fetch that commits after its halt alone.
- **Taking `$4015`'s bit 5 from the internal latch after the fetch** left the internal data bus test failing. The
  fetch's own read had already loaded the sample into the latch. Bit 5 is the latch as it was before the fetch.
- *Retracted in §3.12.4: the table did match, and this reading was taken after the ROM had cleared the page.*
  **The implicit abort's `$500` table** (the load DMA landing X cycles before the output unit's boundary) was not
  reproduced by any lag tried: it stayed zero, where the key has ones at X = A and B. The test passes, because its
  answer sets are chosen by `$508` and the other two tables match. Which part of the rule is missing is not known.

#### 3.11.4 What is left

- **The DMC channel's code 21, and the APU's own audit.**
- **C7** (writes to `$4020`–`$5FFF`): no witness turned up, so it stays a candidate.
- **The per-dot PPU pipelines** of §3.10.5.

### 3.12 Stage 2e: the APU (2026-09-29)

*The APU audited against NES_MiSTer's `apu.sv` (26e2efb). The file was rewritten in 2020, and its noise and pitch
tables and its frame counter are LFSRs read from the 2A03 netlist, so its agreement counts as evidence here. The
exception is where the file marks its own holes: the triangle's ultrasonic output is held on purpose (`allow_us`,
`sample_latch`, `apu.sv:299`) and is no referee point, and the PAL tables are marked speculative. §3.11.4 predicted
that the DMC channel's code 21 was the APU's. That held, and the frame counter's code 19 was the other APU failure.
The audit also found four rules that no AccuracyCoin test reaches. Two of them have a witness in the corpus that the
runner could not read, blargg's 2005 set, which reports in `$F0` rather than through `$6000`.* As in §3.7, each target
test's expectation was read from the ROM's source before anything was changed.

#### 3.12.1 The rules, and their weight

| # | Rule | Moon before | Source | Witness | Verdict |
|---|---|---|---|---|---|
| A1 | In four-step mode `$4015`'s bit 6 rises at 29828 and 29829 even with `$4017`'s inhibit set. The IRQ line does not rise, and the flag is set at 29830 only when uninhibited | never set while inhibited | `set_irq` loads `frame_interrupt_buffer` whatever the inhibit is, and `frame_int_disabled` clears only `FrameInterrupt` (`apu.sv:781-783`, `:811-814`, `:828-830`) | Frame Counter IRQ, code 19 ("J": the flag at 29828 while suppressed), and I, K and L beside it | **fixed**; agreement |
| A2 | A `$4015` enable makes the DMC request wait out the enable's lag (3 cycles on a get, 2 on a put) whatever the buffer holds, so a buffer that empties inside the lag asks only once the lag ends | a full buffer at the write set no lag, so a buffer emptied 1–2 cycles later asked at once | `dma_req = ~have_buffer & enable_3` (`apu.sv:556`), and the enable runs its pipeline from the write whatever the buffer holds (`:593-594`, `:642-643`) | Delta Modulation Channel, code 21 ("L": the write 2 cycles before the timer's clock), and M and N | **fixed**; agreement; the lag is §3.10's 3:2, not refitted |
| A3 | A length-halt write takes effect one cycle late: a halt written the cycle before a length clock does not stop that clock, and a release written then does not let it through | the clock saw the new halt | blargg's 2005 notes, "Write to halt flag is delayed by one clock" (`blargg_apu_2005.07.30/readme.txt`) | `10.len_halt_timing`, code 3 | **fixed**; the ROM is the witness; the RTL leaves it open (below) |
| A4 | A length reload in the cycle before a length clock is lost when the count was running, because the clock decrements the old count. When the count was 0 the reload is kept and not decremented | reloaded, then decremented | `lc_on_1` is sampled at the get before the clock, and the clock overwrites the load: "This deliberately can overwrite being loaded from writes" (`apu.sv:32-33`, `:46-51`) | `11.len_reload_timing`, code 4 | **fixed**; agreement |
| A5 | The noise table is in CPU cycles. The timer runs at the APU rate, so it reloads with half the entry less one | reloaded with the entry itself at the APU rate, so every period was 2(t+1) CPU cycles instead of t: 2.5× too long at the shortest, 2× at the longest | `noise_ntsc_lut` "read directly from the netlist" (`apu.sv:442`), counted to `'h400` on the put (`:458-466`). Stepping each seed gives 4, 8, 16 … 4068 CPU cycles, Moon's table exactly | none among the ROMs; the CPU cannot read the noise channel | **fixed**; netlist provenance, unit-tested |
| A6 | The triangle's length and linear counters hold its sequencer, not its timer | the timer stopped with the sequencer | the timer reloads every period, and only `SeqPos` is gated on `IsNonZero & ~LinCtrZero` (`apu.sv:318-326`) | none; the CPU cannot read the triangle's phase | **fixed**; argued from the RTL, unit-tested |

**Agreement, no change.** The RTL was also read for the rules below, and Moon already matches each one:

- **The `$4017` delay.** `w4017_1` passes to `w4017_2` on the get (`apu.sv:818`), then `frame_reset_2` on the put
  (`:826`), and the LFSR reloads `7FFF` on the next get (`:817`). That is 3 cycles for a write before a get and 4 for a
  write before a put.
- **The sequencer's cycles.** The LFSR takes 3728, 7456, 11185, 14914 and 18640 steps from `7FFF` to its five decodes.
  In CPU cycles, with `ClkE`/`ClkL`'s one-cycle put delay, those are the clocks at 7457, 14913, 22371, 29829 and 37281.
  Those are the six-entry table's clocks (Moon_APU.md §2.1), and the IRQ's decode starts at 29828.
- **Five-step mode's immediate clock.** It falls when the delayed write lands (`w4017_2 & seq_mode`).
- **A `$4015` read against the IRQ's set.** When a read clears the interrupt in the same cycle that `set_irq` sets it,
  the set wins, because it comes later in the block (`:808-814`).
- **The sweep.**
  - It mutes the channel below period 8, or when an add carries out of 11 bits; a negate never mutes (`:167`).
  - It writes a new period only when the shift is non-zero and the frequency is valid.
  - Pulse 1 negates in ones' complement, pulse 2 in twos' (`:162`).
- **The pulse sequencer and the envelope.**
  - A `$4003` write resets the sequencer but not the timer (`:241-246`).
  - The four duty patterns, read against `SeqPos` counting down, are Moon's table.
  - The envelope's divider, decay and loop match.
- **The DMC's pitch table.** Its LFSR seeds count out 428, 380 … 54 CPU cycles, Moon's table exactly.
- **The rest of the DMC.**
  - The output level moves in steps of 2 within 0–127.
  - The silence flag is taken at the byte boundary.
  - The IRQ rises on a non-looping last byte.
  - A `$4010` write with bit 7 clear drops the IRQ.
  - A `$4015` enable restarts the sample only when the channel was off.
- **The length table.** The RTL stores each entry less one and counts to its own zero.

**Candidates, argued only.** Each is a disagreement with no witness, recorded and not changed.

- **K1, `$4011` against a DMC clock.** The put's level adjust starts from the level latched at the get
  (`dmc_volume_next`, `apu.sv:571`, `:612-615`, `:645`). A `$4011` write between the two therefore loses bits 6–1.
  Moon applies the write and the clock to the same value.
- **K2, `$4015` disabling a channel.** The RTL clears the length counter at the next put (`:28`, through
  `enabled_buffer`, `:991`). Moon clears it at once. Only a reload written in between could tell the two apart.
- **K3, the triangle's linear reload value.** It is latched at the get (`LinCtrPeriod_1`, `:330`). A `$4008` write in
  the cycle before a linear clock reloads the old value.
- **K4, `$4015`'s bit 5.** The RTL drives it as 0 (`:1006`), without marking that as a hole. Moon takes it from the
  internal bus (B3), where AccuracyCoin is the witness and overrules the RTL.
- **K5, A3 and A4 on the other channels.** Pulse 2, the triangle and the noise channel share the same code, but both
  ROMs use pulse 1 only. A *halted* count reloaded in the cycle before a clock keeps the reload, as the RTL's
  `halt ? len_counter_int` does. blargg's "completely ignored" is stated only for the unhalted case.
- **K6, the envelope's loop bit.** It is the same bit as the length halt, but A3 delays only the halt, which is all
  that blargg's text says. The envelope's side is not tested.

**Where the RTL leaves A3 open.** The clock uses `len_counter_next`, computed at the get before it from the halt as it
stood then (`apu.sv:33`). It also consults the current `halt` (`:48`). Which of the two samples a write reaches depends
on where the write falls inside the CPU cycle, and the RTL's level-sensitive `write` does not settle that. So the ROM
is the only witness. In Moon's numbering blargg's two writes land at C−2 and C−1 for a clock at C, and C−1 is the only
placement where both halves of each test agree. That places A3 and A4 in Moon's cycle numbering. Neither is a value
fitted among alternatives.

#### 3.12.2 The proofs

**Tests that fail on the unmodified core** (`MoonApuDefectTests`, measured by stashing the core's directory):

- `The_frame_flag_rises_for_two_cycles_while_the_irq_is_inhibited` (A1);
- `An_enable_with_a_full_buffer_waits_out_its_lag` (A2), for the buffer emptying 1 and 2 cycles after the write;
- `A_halt_written_the_cycle_before_a_clock_does_not_stop_it` (A3), with the write two cycles early as the control;
- `A_reload_the_cycle_before_a_clock_is_lost_or_kept_whole` (A4), both halves;
- `The_noise_channel_clocks_once_per_table_period` (A5), at indices 0, 3, 8 and 15. It takes the greatest common
  divisor of the gaps between the output's changes;
- `The_triangle_timer_runs_while_its_sequencer_is_held` (A6). Every step before and after a 1,234-cycle hold must sit on
  one lattice of period + 1;
- `Blarggs_2005_apu_tests_pass_on_both_engines`: all eleven of blargg's 2005 ROMs read `$F0` = 1 on each engine. They
  run from `EMUSEN_MOONRT_CORPUS` when it is set, as the corpus test does. 10 and 11 fail on the unmodified core.

**AccuracyCoin: 124 → 126 of 144** (measured 2026-09-29; the table at frame 3,620):

| Page | Passed | Still failing (code) |
|---|---|---|
| 1–13 CPU, unofficial opcodes, interrupts, APU registers and DMA | 94 of 94 | — |
| 14 APU | **9 of 9** (Frame Counter IRQ and Delta Modulation Channel now pass) | — |
| 15 CPU behaviour 2 | 5 of 5 | — |
| 17 PPU behaviour | 5 of 5 | — |
| 18 PPU vblank timing | 7 of 7 | — |
| 19 Sprite evaluation | 3 of 8 | `$2002` flag timing (1), suddenly resized sprite (5), arbitrary sprite zero (2), misaligned OAM (1), OAM corruption (2) |
| 20 PPU miscellany | 2 of 7 | `$2004` (6), the rendering flag (2), `$2007` while rendering (2), the `$2004` and `$2007` stress tests (2 each) |
| 21 Advanced background evaluation | 1 of 5 | stale shift registers (3), serial in (2), ALE + read (2), hybrid addresses (2) |
| 22 Advanced sprite evaluation | 0 of 4 | sprites on scanline 0 (2), stale shift registers (3), frozen OAM2 increment (2), misaligned OAM2 address (3) |

Every CPU and APU page now passes. All 18 remaining failures are on the PPU's pages 19–22 (§3.10.5).

**The corpus:** unchanged at **105 passed**. Every ROM's outcome, frame count, reset count and transcript hash matches
2d's. The runner's `$6000` protocol grades none of the four rules. blargg's 2005 set is graded by the test above: 9 of
11 before, 11 of 11 now. On both engines the corpus test runs identically ROM by ROM.

**The games:**

- The four bench games' **CPU RAM** at frame 3,900 is identical to 2d's.
- All eight **pictures** are byte-identical to 2d's.
- The **state hashes** change, because the noise and triangle timers are part of the save state:

  | Game | 900 | 3,900 |
  |---|---|---|
  | SMB | 5FEE46B00AE060B1 | 9E947B8B7F05CE56 |
  | Zelda | D0F3FC71E3D9D4C1 | 6AEE7A8942BDE7CE |
  | SMB3 | BA44BCE2704A4E0C | B0087A51E432D6AC |
  | Punch-Out!! | 5DC308CD1083A630 | CC29D393FFAC1228 |

- **Playability** from boot and from the 11 library states (§3.9.3): the same verdicts as 2d. Every run plays at
  733.79 samples a frame, and the picture changes in the same number of seconds. There is no fault and no JAM, except
  the SMB1 hack's resume state, which the base build also JAMs.

**The save state stays version 4.** Every field the rules add is `[SkipInState]`:

- the halt a length clock sees;
- the count a reload replaced;
- the flag that a channel register was just written.

Each lives for one cycle after a write. A load re-derives them: the halt as the register holds it, and no reload
pending. A state taken on the cycle right after a length write therefore loses that one cycle's delay, and both
engines lose it identically.

**MoonRT** carries all six rules in `apu/channels.rs` and `apu/mod.rs`, and the load's re-derivation in `machine.rs`.
All of these pass:

- 358 Moon-filtered WiseMan cases, including AccuracyCoin result for result and blargg's 2005 set on both engines;
- the corpus test: 255 ROMs, 363,935 frames with identical state, 105 passed;
- the crate's 6.

**Reuse:** nothing here is shared with another core, and MarsRT is untouched.

#### 3.12.3 The sound

The four bench games were run for 3,900 frames with the bench's inputs, on 295ca478 and on this stage, once with the
mix and once with each channel soloed through the debug mute (measured 2026-09-29):

| | samples/frame | pulse 1, pulse 2, DMC | triangle RMS | noise RMS | mix RMS | clipped |
|---|---|---|---|---|---|---|
| SMB | 733.79 → 733.79 | identical | 818.9 → 819.1 | 723.3 → 597.7 | 2818.3 → 2806.7 | 0 → 0 |
| Zelda | 733.79 → 733.79 | identical | 178.9 → 178.8 | 0 → 0 (unused) | 1058.9 → 1059.1 | 0 → 0 |
| SMB3 | 733.79 → 733.79 | identical | 892.3 → 892.3 | 1389.1 → 1083.9 | 1985.0 → 1915.4 | 0 → 0 |
| Punch-Out!! | 733.79 → 733.79 | identical | 1336.7 → 1337.1 | 813.2 → 775.3 | 2947.4 → 2945.2 | 0 → 0 |

- **Pulse and DMC.** Both pulses' and the DMC's samples are byte-identical. None of A1–A4 is reached by these games'
  playing.
- **Noise.** The noise channel's samples change in every game that uses it (A5). Its RMS falls by 5–22%, because it now
  toggles two to two and a half times as often. More of its energy then sits above the 14 kHz low-pass and inside each
  output sample's box average. That is the expected consequence of the right pitch, not a level change.
- **Triangle.** Its samples change slightly (A6). The phase it resumes at after a hold now depends on the free-running
  timer.
- **Nothing is broken.** No game went silent, no channel ran away, and none clipped. Peak levels are within 4% of
  before.
- **The mixer is unchanged.** `Mix`, the filters and the sample queue were not touched, and none of the six rules
  needed them to be.

#### 3.12.4 Negative results, and a retraction

- **§3.11.3's "`$500` table not reproduced" was a misreading, and the table matched at 2d's head.** Retracted. It was
  measured on 2026-09-29 by logging the test's own stores to `$500`–`$54F`. On 295ca478 and on this stage alike, those
  stores are key 1, key 2 and key 3 exactly: `$50A` and `$50B` are 1, `$52A` is 1, and `$54A`–`$54F` are 4. The earlier
  reading was taken from RAM after the test ended, as `dump` takes it. By then the ROM has stored zeros over
  `$500`–`$521`, and those stores are in the same log. So the reading showed zeros that are not the test's results. No APU rule was needed.
- **`dmc_tests` (four ROMs) could not be graded.** They have no text, no `$6000` or `$F0` protocol, and a plain grey
  screen on both builds. The corpus's `tvsha1` hashes a television output this harness does not produce. They remain
  unread.
- **The RTL alone would not have found A3.** Read naively, `apu.sv:48` stops a clock with the newly written halt, which
  is the defect. The rule came from blargg's notes and his ROM.

#### 3.12.5 What is left

- **Pages 19–22:** the per-dot PPU pipelines of §3.10.5, which hold all 18 remaining AccuracyCoin failures.
- **K1–K6** above, and **C7** from §3.11: candidates with no witness.
- **The mixer**, which is stage 3's.

### 3.13 Version 5: the mixer in the state (2026-10-03)

**Decided 2026-10-03:** a state load must resume exactly, sound included, so the mixer joins the state. Until version 4
C# Moon marked its mixer `[SkipInState]`, and MoonRT, whose state is C# Moon's byte for byte, did the same. A machine
loaded with another's state therefore resumed its sound with the resampler and the output filters as the loading
machine had them, which the conformance kit's C7 and C8 caught (`EmuSen_CoreAPI.md` §22.3).

**The format.** Version 5 appends, after version 4's DMA tail, 60 bytes, in C# Moon (`Apu.WriteMixer`) and MoonRT
(`Machine::write_state`) alike:

| Field | Type | Bytes |
|---|---|---|
| `Apu._sampleAccumulator` | double | 8 |
| `Apu._sampleCount` | int | 4 |
| `Apu._cycleFraction` | double | 8 |
| `Apu._hp90`, `_hp90Prev`, `_hp440`, `_hp440Prev`, `_lp14k` | double each | 40 |

Doubles are `BinaryWriter.Write(double)`'s eight little-endian bytes, which the shared codec's new `f64` writes and
reads. The magic stays `MOON`. The cycles per sample and the filters' coefficients are constants of the 44.1 kHz output
and stay out; so does the undrained sample queue, whose length varies, because a state's size must be constant
(`EmuSen_CoreAPI.md` §6.9): a state is taken at a frame boundary after the host has drained the frame's sound.

**Older states.** Both engines still read versions 3 and 4. For them the mixer keeps what it held, exactly as before
version 5, so a player's older states load as they always did; every state written is version 5.

**What it proves** (measured 2026-10-03):

- `MoonRtStateTests.A_state_loaded_into_a_second_machine_resumes_its_sound_exactly`: 300 frames of the bench's
  input, the state loaded into a second machine, 300 more: the samples are identical frame by frame, and so are the
  states, in C# Moon and in MoonRT, on mappers 0 and 4.
- `An_older_state_loads_in_both_engines_alike`: version 3 and version 4 states, cut from a version 5 one, load alike
  in both engines and are written back as version 5, byte for byte the same.
- The layout walk names the new fields and both engines' layouts still agree by name, offset, length and type.
- **The bench's state hashes change only by the added bytes.** For each of the four games at frame 3,900, the version
  5 state is the version 4 state with its version field 4 → 5 and the 60 mixer bytes appended: with those two
  differences undone it hashes to the recorded version 4 hash. The new hashes, the same from both engines: Super Mario
  Bros. `0ED799A150F20023` (25,362 bytes), The Legend of Zelda `C615B5D3AB959325` (25,390), Super Mario Bros. 3
  `5D8C02E36AA890C0` (148,320), Punch-Out!! `EF1FEC5B2CFF3B66` (148,271).
- The conformance kit's C1–C8, C10 and C11 pass on all four games.

## 4. Stages

MercuryRT's stages 1–5 were done in one day; Moon's machine is about 5,200 lines of C#: the folder's 6,609, less the
debug target (553), `ApuWriteTrace` (115), the disassembler (114), the cheat codecs (137), the spaces (103), the
validation adapters (330) and the README.

| Stage | What it covers | Its oracle | Cost |
|---|---|---|---|
| 0 | `moonbench` (done, §1.1); the corpus-transcript command for both engines | The baseline of §1.1 reproduced | hours |
| 1 | The crate skeleton; Rust structs for every serialised type, all sixteen boards; `state.rs`; the layout listing and `naming.rs` | §3.2's three proofs | hours |
| 2 | The machine: the 6502 with every undocumented opcode and `JAM`, the interrupt sampling and delayed `I`, the bus, OAM DMA, the DMC fetch and its stolen cycles, pads, the PPU's registers, open-bus decay, per-dot clock and fetches, **composition without pixel writes**, the APU's channels and sequencer, all sixteen boards with the A12 and CPU-cycle hooks, the timeline and `Reset` | Random 6502 programs compared after every instruction on every board; a synthetic program per board; four games and more by state every frame; the corpus transcripts; the first speed number | a day |
| 3 | The mixer and the pixel writes: `Mix`, the box filter, the three filters, the queue and its drop-oldest rule, `Drain`; the palette writes | Samples and RGBA identical every frame | hours |
| 4 | Shim and frontends: `MoonRtCore`; the Engine row for NES in `CoreCatalog.EngineFor` (Moon (C#) stays the default); `CoreFactory`; battery saves; both cheat codecs with the Game Genie table; `FrameLog`; exception mapping; the crash log; the debug target over a mirror | Commercial ROMs through `ICore` on both engines; states crossing engines both ways; the fallback | a day |
| 5 | Debugger hooks: breakpoints, stepping, coverage and the write log as pushed tables and drained logs | `MoonDebugTargetTests` on both engines; games with every table armed identical to plain runs | 1–2 days |
| 6 | CI: the crate in the workflow's matrix and the publish targets | Crate tests on four platforms | hours |
| 7 | Parity: the rest of WiseMan’s Moon tests (134 test methods) through a rig; mutants for every stage | §3.5, the mutant table | a day |
| 8 | Goldens (§3.6) | Every golden reproduced by MoonRT | half a day |

*Decided 2026-09-28:* from stage 2 on, each component is audited against NES_MiSTer before it is ported (§3.7).

**Finished** means `Mars_Native.md` §6's three criteria: a player gets MoonRT without choosing it; nothing the C# core
does is lost, or the loss is written down; the C# core's role is decided.

## 5. Risks and open questions

- **Q1, naming.** MoonRT, by the project's standing pattern.
- **Q2, one state crate or three copies.** With MoonRT there are three copies of `state.rs` (MarsRT's, MercuryRT's
  with strings, MoonRT's). `Moon_Memory.md` §6 set the project's own threshold for hoisting a shared shape: "when a
  third core makes the shape a rule instead of a coincidence". This is the third. **The recommendation is a small
  `emusen-native` crate** holding the writer, the reader, the layout and the naming rule, extracted after MoonRT's stage
  1 and after the work in progress in MarsRT has merged, because the extraction touches MarsRT. Until then MoonRT
  copies MercuryRT's pattern.
- **Q3, whether to fix §6.1's defects first.** The recommendation is **no** for D1: it is the other half of a
  compensating error `Moon_Memory.md` §4.8 records, and fixing it is a design change to the frame loop that MMC3 and
  RAMBO-1 were validated against. It should be done once, after the port, with the corpus and Skull & Crossbones as
  its oracle, in whichever engine is then canonical.
- **Q4, whether the port carries Moon's known simplifications.** The per-line renderer, the instantaneous OAM DMA, the
  flat 4-cycle DMC fetch, and the 18 corpus failures. The recommendation, as for Mercury, is to port them exactly.
- **Q5, the C# Moon's future**, and when the Engine row's default flips. As for Mercury.
- **Q6, the design levers of §1.2.** Each is exact by argument and could be built in both engines while C# is the
  oracle, or in MoonRT alone afterwards with the corpus and the goldens as its oracle. That choice is left open.
- **Q7, AccuracyCoin.** The RTL's 2025–2026 fixes to its bus, DMA and PPU were each graded by an AccuracyCoin
  sub-test (§3.7.1). The corpus at `/home/red/nes-test-roms` does not hold it. With it, the audit's disagreements in
  those parts could be settled by a ROM instead of left as candidates. It is third-party and would not be committed.
- **Risk: P1 is wide, and centred low on purpose.** If stage 2's like-for-like number comes in under 1.1×, the port is
  still worth finishing on ground 1 of §1.2, and the levers become the next question.
- **Risk: bit-exact sound across platforms.** Argued in §3.3, untested on Windows and macOS until stage 6.

## 6. What the review found in the C# core

### 6.1 Defects demonstrated (measured 2026-09-24, on the unmodified WiseMan build)

- **D1: the APU runs ahead of the picture by every DMA cycle, and produces 1.7% more sound than its rate says.**
  - *Cause.* `RunOamDma` steps the APU through all 513 cycles (`MemoryBus.cs:193`) and the DMC's stolen cycles do the
    same (`:71-77`), but neither steps the PPU, and a frame ends when the PPU completes one (`MoonCore.cs:187`).
    `Moon_Memory.md` §4.8 records the PPU's half of this as a compensating error for the boards clocked on the CPU; its
    consequence for the APU is not recorded there.
  - *Measured.* Two synthetic NROM images, identical except that one writes `$4014` from its NMI handler: 733.80
    stereo samples a frame without the write, **746.43 with it**. 44,100 Hz at 60.0985 frames a second is 733.8. Every
    game measured is at 737.8–747.7 (§1.1), so the core emits about 44,860 samples for each emulated second it labels
    44,100.
  - *Consequences.* A frontend's audio sync absorbs about 760 extra samples a second. The frame sequencer's
    29,830-cycle IRQ and the DMC's IRQ drift against vblank by 513 cycles a frame, about 4.5 scanlines, in any game
    that DMAs every frame, which is nearly all of them. `MasterClock` also runs about 4.5 lines a frame ahead of the
    PPU, because the budget pacing charges the DMA while the PPU does not see it.

### 6.2 Defects argued, not demonstrated

- **D2: a state saved mid-frame does not carry the evaluated sprites.** `_spriteCount` and `_spriteIndices` are
  `[SkipInState]` (`Ppu.cs:70-71`) but decide the pattern table the sprite fetches put on the bus
  (`Ppu.Timing.cs:170-183`). On the pre-render line they hold line 239's evaluation. A state saved at a breakpoint
  between line 239 dot 256 and line 261 dot 320, with 8×16 sprites, loads into a fresh core with a different A12
  pattern on that line, so an MMC3's counter can differ. States saved between frames are unaffected, because line 0
  re-evaluates before its sprite fetches.
- **D3: the pads' shift registers and strobe are not state** (`MemoryBus.cs:23-24`). A state saved mid-poll resumes
  the poll from the instance's own register. Frame-boundary states are unaffected in games that finish their poll in
  the NMI.
- **D4: an image with zero PRG banks, or a board indexing outside its PRG, throws from inside the machine.**
  `Cartridge.Parse` accepts `image[4] == 0` (`Cartridge.cs:112-146`); the first PRG read is then `% 0` or an index
  past an empty array (`Nrom.cs`, `Mmc1.cs` and the rest), and `LoadRom` throws `DivideByZeroException` or
  `IndexOutOfRangeException` while reading the reset vector. `IremH3001`'s power-on `_prgBanks[2]` is the page count
  less two, negative for an 8 KB PRG. MoonRT must refuse with the same exception, not panic.

### 6.3 Places the design does not port directly

- **The state format is a reflection walk,** with the bus written twice (§3.1).
- **Interfaces in the hot loop:** `ICpuBus` twice per access (`Cpu.cs:208-229`), `IMapper` for every PRG and CHR read
  and every PPU fetch (`Ppu.Timing.cs:185-189`), `IRomReadPatcher` on every cartridge read, a nullable `IWriteObserver`,
  and a `Func<ushort, byte>` for the DMC. In Rust: a concrete bus, an `enum` of boards, a pushed table, a drained log,
  and the split fetch of §2.5. On Venus's evidence (`Venus_CPU.md` §9.3–§11), removing the dispatch is not itself
  where a gain is.
- **Debugger seams on the plain path:** `ShouldBreak` per instruction over an empty list (`MoonCore.cs:173`),
  `Coverage.IsArmed` per instruction, `TryPatchRom` per cartridge read through a patcher that is never null
  (`MoonCore.cs:96`), two `Stopwatch` reads a scanline, and two static settings reads per register write
  (`MemoryBus.cs:127,143`). About 6% of an inlined frame together.
- **Exceptions as control flow:** an unsupported board throws `NotSupportedException` (`Cartridge.cs:184`); D4's
  images throw from inside the frame. Under `panic = "abort"` these become statuses the shim rethrows.
- **The picture is handed out live** (`MoonCore.cs:197`); the shim's copy changes that, for the better, as MercuryRT's
  did.
- **Collections that allocate:** the APU's `Queue<short>` and `Drain`'s array per call (`Apu.cs:57,419-429`); the
  mapper `DebugState` arrays per call. In Rust a ring buffer.

### 6.4 Dead or inert code (the port still carries whatever is in the state)

- `Mmc3._irqsFired` is a diagnostic counter (`Mmc3.cs`, "Diagnostic only"), and it is in the state.
- `Apu.Registers` is kept only for the debug target, and is in the state.
- `Cpu._instructionCycles` is zeroed at the start of every `Step` and is in the state.
- `ICpuBus.TickStolen`'s default body has no caller in the core.
- There are no `TODO`, `FIXME` or `HACK` markers in the core.

### 6.5 Negative results

- `SkipRendering` does not change the state: every game's state hash was the same with the pixel writes on and off
  (§1.1).
- The four games' runs were deterministic across processes: one state hash per game in every run of the same length,
  over five rounds, both pixel-write modes and a CPU quota.

## 7. Predictions to be retired

| # | Prediction | Retired when |
|---|---|---|
| P1 | A line-for-line MoonRT's plain frame takes 0.69–0.99 of C#'s time, central 0.83 (1.2×), on the four games | **Retired 2026-09-30, held:** 0.72–0.78 through the shim, geometric mean 1.34× (§8.2.3) |
| P2 | The idle machine (the `JMP` loop) runs in 0.60–0.77 ms against C#'s 0.77 | **Retired 2026-09-30, refuted in the favourable direction:** 0.52 ms, 1.48× (§8.2.3) |
| P3 | No sampled time on breakpoint, coverage, patch or phase-timer checks in MoonRT's plain frame | Stage 5, with every table disarmed |
| P4 | Nothing observable changes on the desktop or at 33%; SMB3 at 10% goes from 78% to 80–113%, central 94% | Stage 4 |
| P5 | The once-a-frame boundary costs under 3% of MoonRT's frame | Stage 4 |
| P6 | No recompiler and no threads are needed for MoonRT to beat C# everywhere measured | Stage 4 |
| P7 | The audit's disagreements fall mostly in the PPU and in the DMA and bus, and few in the CPU (§3.7.3) | When stage 2's components have been audited |

## 8. The work, stage by stage

Each stage is recorded here as it is proven, as `Mercury_Native.md` §8 records MercuryRT's.

### 8.1 Stage 1: the state, byte for byte (done 2026-09-24; mutants 2026-09-28)

**How the field list was established.** Not by reading the classes. A throwaway reflection walk (`layoutdump`, in the
probe cache) printed every field the serializer walks, with its type and array length, for a synthetic image of each of
the sixteen boards. It confirmed §3.1's reading, including the one surprise of the plan: `Cpu._bus` is an
interface-typed field that is not `[SkipInState]`, so the `Cpu` walk carries a presence byte and the whole
`MemoryBus` (`OpenBus`, `PendingDmaCycles`, the 2 KB of `Ram`, `StolenCycles`), which the `Bus` walk then writes again.
`Nrom` has no serialised fields at all; MMC3 has the most (fourteen, one of them the `A12Watcher` class).

**What was built.** The crate is `EmuSen/Cores/Nintendo/MoonRT - NES/`:

- `state.rs`: MercuryRT's codec without the string primitive, which Moon's state does not need;
- a Rust struct for every serialised class, one `enum` arm per board, and `Skip<T>` for every `[SkipInState]` field
  the machine still needs (the PPU's line scratch and evaluated sprites, the mixer, the pads, the board);
- `machine.rs`: the header, then the six walks, with `Cpu::write_state` writing the bus inside itself under `_bus`;
- `naming.rs`, MarsRT's rule over eight modules, taught to read Rust's raw identifier `r#loop` as C#'s `Loop`;
- a C ABI (`moon_machine_new`, `_free`, `_load_state`, `_save_state_size`, `_save_state`, `_state_layout`);
- on the C# side, `Shim/MoonNative.cs` (the loader, `EMUSEN_MOON_NATIVE=0` to turn it off) and `Shim/MoonMachine.cs`
  (the handle).
- The build. The crate is a row of `EmuSen/Cores/RustCores.props`, beside MarsRT's and MercuryRT's, and `EmuSen.csproj`
  builds it into `obj/moonrt`. It has no `pgo/` profile, so it builds unguided (`Mars_Native.md` §6.17). The row also
  puts it in the foreign-platform publish (`RustCoresPublish.targets`). The Rust cores workflow builds and tests it on
  four platforms and runs `MoonRtStateTests` against each library. That is stage 6's wiring, not its proof: nothing
  has yet run on a platform other than Linux x64. The first version of the stage had its own cargo target. It became
  the row when WiseMan's generic Rust-core build was merged, on 2026-09-28.

*Order.* The machine's behaviour was written in the same pass, because the structs are the same structs.
`MoonRtMachineTests` is its oracle, and its proof is stage 2's. This stage's claim is the state alone: that MoonRT holds a C# state and gives it back, byte for byte.

**A C# exception is a fault, not a panic.** D4's images (§6.2) make C# throw from inside the machine: an index outside
an array, a division by zero, a clock running backwards. Under `panic = "abort"` the same arithmetic in Rust would kill
the frontend. MoonRT does each such access through a helper that records the first fault in a thread-local and returns
zero; the frame, the load or the step then returns a status, and the shim throws C#'s exception type. The machine is
left where Rust left it, not where C#'s exception left it, which is a divergence only in a machine already broken.

**The oracles, all identical** (`EmuSen.WiseMan/Cores/MoonRtStateTests.cs`, 71 cases; crate tests, 10):

- **Running machines.** All sixteen boards, each with 8 CHR ROM banks and with CHR RAM, ran a program that turns
  rendering and a pulse note on, counts in RAM, copies the count to PRG RAM and scrolls by it, for 300 frames. The C#
  state went into Rust and back out byte for byte, and its layout listing matched C#'s own reflection walk line for
  line: 158–173 fields, 25,294–82,737 bytes.
- **Noise.** The same 32, with every serialised field of every object, the header's four clocks included, filled with
  distinct noise.
- **Bytes no C# writer makes.** Four bools and three class flags of 2 read as C#'s reader reads them, and the re-save
  equals the C# core's own.
- **Class flags of zero.** The flags of `Cpu._bus` and `Apu.Dmc` set to 0 make C#'s reader leave both objects as they
  were and read on. Rust does the same, so the re-save matches C#'s.
- **Refusals.** A truncated state, a foreign magic and a wrong version change nothing. The truncated state is also one
  ten frames later than the machine's, so a partial read cannot go unseen. An image with no magic, whether 3 bytes
  or full length, and an MMC5 image are refused with C#'s exception types.
- **D4.** An image with no PRG is refused on NROM, MMC1 and MMC3 with the exception type the C# core throws from
  `LoadRom`: `IndexOutOfRangeException` on NROM, `DivideByZeroException` on the others.
- **Real games.** Super Mario Bros., Zelda, Super Mario Bros. 3 and Punch-Out!! are identical at frames 0, 300, 600 and
  900 with the bench's input, from `EMUSEN_MOONRT_ROMS`, a directory of scratch copies; absent, the case passes unrun.

**Mutants: 24; 22 caught, 2 equivalent** (measured 2026-09-24 and 2026-09-28). The runner (`mutants.py`, in the
scratch directory `~/.cache/emusen/probe/moonrt/`) applies one edit to the crate. It then runs `cargo test`, builds the
library, runs WiseMan's `MoonRtState` filter against it with the four games, and restores the file. S1–S11 are the
first round's. S12 was being run when that round stopped, on 2026-09-24, and the source was left mutated: the stage-1
commit carried S12's exchanged clocks. The next commit restores them, and every mutant was run again on the merged tree.
Five survived the first full round. Three of them (S7, R1 and C1) were caught after one new test each, and the other
two are argued equivalent below.

| Mutant | Caught by |
|---|---|
| S1 the CPU's `A` and `X` exchanged in the writer | crate naming rule; WiseMan running machines, noise, odd bytes, games |
| S2 the bus inside `Cpu._bus` read and thrown away | **survived** (equivalent, below) |
| S3 MMC3's `_mirroring` and `_prgMode` exchanged under the right labels | crate naming rule; WiseMan noise only |
| S4 the envelope's `Loop` and `ConstantVolume` in declaration order in both reader and writer | WiseMan running machines, noise, games; the crate's round trip cannot see it |
| S5 a bool read as `== 1` | WiseMan odd bytes only |
| S6 `Apu._cycleCount` written as an `i32` | crate round trip and bus test; WiseMan every case that compares bytes |
| S7 a failed load keeps its partial read | crate and WiseMan refusal cases, **after** the truncated state was made one ten frames on; before, the partial read wrote back what was already there |
| S8 CHR not written | crate round trip and bus test; WiseMan every byte comparison |
| S9 RAMBO-1's A12 watcher moved after `_bankSelect` | crate round trip; WiseMan running machines, noise |
| S10 the pulse's `_onesComplement` not read back | WiseMan noise only; it never changes in a running machine |
| S11 the PPU's `V` and `T` exchanged in the reader | crate naming rule; WiseMan running machines, noise, odd bytes, games |
| S12 the header's `_cpuBudget` and `_masterClock` exchanged in the writer | crate naming rule and round trip; WiseMan running machines, noise, odd bytes, games |
| F1 the fault helper keeps the last fault, not the first | crate D4 test; WiseMan D4 |
| F2 an out-of-range byte read gives 0 without a fault | crate D4 test; WiseMan D4 |
| F3 a remainder by zero gives 0 without a fault | crate D4 test; WiseMan D4 |
| F4 the fault statuses count up from −30 | WiseMan D4 only: the shim maps −31 and −32 to the exception types |
| N1 the naming rule keeps C#'s leading underscore | crate naming rule (both tests) |
| N2 the naming rule puts no underscore after a digit | **survived** the round (equivalent over this state, below); now caught by the crate's rule test |
| N3 the naming rule splits every capital of an acronym | crate naming rule (both tests) |
| R1 an image of 16 bytes or more not checked for its magic | crate and WiseMan, **after** a full-length image without the magic was added; before, only a 3-byte one was refused |
| R2 MMC5 accepted as NROM | WiseMan refusal case only |
| R3 the state's version not checked | crate and WiseMan refusal cases |
| R4 a header claiming exactly the file's PRG refused as truncated | WiseMan running machines, noise, games, refusals (every CHR-RAM image ends at its PRG) |
| C1 a class read whatever its present flag | WiseMan's class-flag case only, **after** it was written; no C# writer makes a flag of 0 |

**The two survivors, argued:**

- **S2 is equivalent through every reader.** `Cpu._bus` carries the whole `MemoryBus`: `OpenBus`, `PendingDmaCycles`,
  `Ram` and `StolenCycles`. The `Bus` walk that follows reads every one of those fields again, so the first copy never
  stands. The crate's `the_bus_is_written_twice_and_the_last_copy_stands` pins the half that matters, that the second
  copy wins. No test can tell the mutant apart, and none is owed: if C# ever stops rereading the bus, the layout
  listing changes and every byte comparison fails.
- **N2 is equivalent over this state's names, and is now pinned.** Of the 154 distinct names in the layout listing,
  five carry a digit (`_a12`, `_chrBank0`, `_chrBank1`, `Pulse1`, `Pulse2`) and none has a capital after the digit. So
  the rule's digit branch is never taken. One assertion, `snake("_bank0Mode") == "bank0_mode"`, was added to the rule's
  own test, and the mutant fails it. A future field such as MMC5's `_chrBank0Upper` would otherwise pass unseen.

**Which oracle carries which class.** The naming rule alone catches a value written under the wrong label
(S3, S11). Only WiseMan catches a reorder made in both reader and writer (S4), because the Rust round trip is
self-consistent. The noise case alone catches a field a running machine never changes (S3, S10), and the odd-bytes case
alone catches the reader's rule for bytes C# never writes (S5). Without any one of the four, a mutant here survives.

**What stage 1 does not establish.** It establishes that MoonRT holds a C# state and gives it back. It says nothing
about running one. The `Skip<T>` fields exist in Rust but no stage-1 test reads them. The fault helper is exercised
only through D4's load, not inside a running frame.

### 8.2 Stage 3: the mixer and the pixel writes (done 2026-09-30)

*§4's table gives stage 3 the mixer (`Mix`, the box filter, the three filters, the queue with its drop-oldest rule,
`Drain`) and the palette writes, with "samples and RGBA identical every frame" as the oracle. Stage 2 had already
written both paths. They are the same structs, and `MoonRtPair` compared samples and pictures on the games from the
start. So this stage is mostly proof: it drives the paths that no earlier comparison reached, extracts the queue into
the shared crate, and takes the first speed number with everything on.*

#### 8.2.1 The oracle

`EmuSen.WiseMan/Cores/MoonRtSoundAndPictureTests.cs` covers each path with a case:

| Case | What it drives | Result (measured 2026-09-30) |
|---|---|---|
| `A_low_audio_limit_drops_the_oldest_pairs_alike` | Six audio limits in turn on a program that plays all five channels: 2,000, 1,001 (odd), 3, 0, 500 and 128,000. The queue is drained only every seventh frame, and between drains both queues' lengths are compared. | 294 frames, 96,426 samples identical |
| `A_muted_channel_mixes_alike` | The same program with no channel muted, each of the five alone, and all five muted | seven masks, 300 frames and 440,274 samples each, identical |
| `The_bench_games_mix_each_channel_alike` | The four bench games, 1,200 frames each, once for each channel alone | twenty runs, 1.76 million samples each, identical |
| `Skip_rendering_switched_mid_run_draws_alike` | `SkipRendering` switched eight times, every 37–44 frames, with the picture compared on every drawn frame | 324 frames identical |
| `Palette_writes_grayscale_and_emphasis_draw_alike` | A palette entry rewritten every frame, and `$2001` cycled through all sixteen combinations of grayscale and the three emphasis bits | 512 frames identical |
| `Library_states_sound_and_draw_alike` | The eleven library states (§3.9.3), each loaded into both engines and run for 900 frames with the bench's input | all eleven, 1.32 million samples and 900 pictures each, identical |

**The earlier comparisons still hold, now with sound and picture.**

- `Real_games_run_identically_from_boot_and_from_a_transferred_state` still runs the four games from boot and from their
  frame-3,000 states.
- The corpus test now draws and compares sound and picture on every frame. It used to skip the pixel writes and compare
  state alone. Result: 255 ROMs, 363,935 frames identical in state, sound and picture, and 105 passed.
- The Moon-filtered cases: 370 of 370.

**Stage 2e's noise change (A5) is in every comparison.** The bench games' noise-alone runs compare the channel's new
periods sample for sample. In every run the two engines' mixes agree to the last sample.

**What the cases could not include:**

- **A sample-rate change.** Moon has none to test: `AudioSampleRate` is a constant 44,100. The only caller of
  `Apu.SetSampleRate` is `LoadRom`, and MoonRT has no export for it.
- **Emphasis colours.** Neither engine applies them (`Moon_PPU.md` §3.5), so the palette case shows only that the two
  engines ignore emphasis identically. The oracle is the C# core, and emphasis is a gap in the core, not a port
  difference.

**Four mutants, all caught:**

| Mutant | Caught by |
|---|---|
| the trim drops one sample instead of a pair | the audio limit case |
| the trim runs two samples late | the audio limit case |
| mute bits reversed | four of the seven mute masks |
| grayscale ignored in the pixel writes | the palette case |

**Nothing needed fixing.** No case found a difference, so no C# test was owed.

#### 8.2.2 The shared queue

`SampleQueue` in `emusen-native` replaces the copies in MoonRT and MercuryRT (`EmuSen_RustState.md` §2.1).

- **MercuryRT's audio is unchanged byte for byte.** Its platform cases hold the sound of four programs to SHA-256
  digests recorded on linux-x64 before the change, and all 159 MercuryRT-filtered cases pass.
- **MarsRT stays out.** Its Rust is untouched, and it joins at its migration step. Its library, rebuilt against the
  extended crate, disassembles to the same instructions as before.

#### 8.2.3 The first speed number with everything on

Measured on 2026-09-30 with §1.1's `moonbench`, through `MoonRtCore`, so the shim's once-a-frame boundary is included.

- 3,000 timed frames after a 900-frame boot.
- Picture taken and audio drained every frame.
- Five rounds interleaved game by game and engine by engine, under the timing lock.
- The load average was 1.5–1.8, with only the desktop's usual applications running. That is higher than §1.1's 1.2,
  but interleaving puts any drift on both engines alike. The spread across rounds was under 5% for C# and under 2% for
  MoonRT.
- Every game's state hash was the same on both engines in every round.

| Game | C# Moon, mean ms/frame (range) | MoonRT (range) | MoonRT / C# | Speed-up | MoonRT, % of full speed |
|---|---|---|---|---|---|
| Super Mario Bros. | 1.412 (1.401–1.447) | 1.010 (1.009–1.011) | 0.716 | 1.40× | 1647% |
| Zelda | 1.264 (1.237–1.300) | 0.915 (0.910–0.920) | 0.724 | 1.38× | 1819% |
| Super Mario Bros. 3 | 1.716 (1.693–1.742) | 1.341 (1.329–1.354) | 0.782 | 1.28× | 1241% |
| Punch-Out!! | 1.576 (1.571–1.585) | 1.196 (1.192–1.201) | 0.759 | 1.32× | 1391% |
| Synthetic `JMP` loop (3 rounds) | 0.770 | 0.521 | 0.677 | 1.48× | 3194% |

The geometric mean over the four games is **1.34×**. With the pixel writes skipped (three rounds), the geometric
mean is 1.33×, and the frame costs are:

| Game | C# ms/frame | MoonRT ms/frame |
|---|---|---|
| Super Mario Bros. | 1.282 | 0.944 |
| Zelda | 1.153 | 0.865 |
| Super Mario Bros. 3 | 1.609 | 1.254 |
| Punch-Out!! | 1.509 | 1.120 |

**P1 is retired, and it held.** P1 predicted 0.69–0.99 of C#'s time, central 0.83 (1.2×). The measured values are
0.72–0.78, inside the range and on the fast side of its central value. This is the number P1 was to be settled by. It is through the shim, not the engine alone, so stage
4's measurement repeats it rather than replaces it.

**P2 is refuted in the favourable direction.** The idle machine was predicted at 0.60–0.77 ms and measured at
0.52 ms. That is 1.48×, more than any game gains.

P2's own reading was that a larger gain on the idle machine would mean "the renderer is where the games lose it". The
pixel writes alone do not bear that out:

- Full minus skip gives the pixel writes' cost. It is 0.05–0.09 ms in MoonRT, against 0.07–0.13 ms in C#.
- So the Rust pixel path is not slower, unlike Mercury's renderer (`Mercury_Native.md` §8.2).
- The games' smaller gain must then come from what they add to the idle loop: the CPU's work, the boards, and the line
  composition that runs whether or not pixels are written.

Which of those three it is was not measured. `ipsample.py` over both engines is the tool for it.

**C# Moon itself got slower through stage 2's audit.** Against §1.1's baseline, the four games are 5–9% slower, and the
idle loop is unchanged within its noise. That is the cost of the halts, the per-dot rules and the APU fixes. Both
engines carry it. It is why the C# column here is not §1.1's.

#### 8.2.4 What is left

- Stage 4: the shim, the frontends and the engine row.
- **P3–P6 stay open.**
  - P4 needs the quota runs through the frontend.
  - P5 needs the boundary priced on its own; here it is inside MoonRT's column.
- Whether the games' smaller gain comes from the CPU, the boards or the composition, which was not measured (§8.2.3).

### 8.3 Stage 4: MoonRT in the frontends, on the common native interface (done 2026-09-30)

*§4's stage-4 row, built on `EmuSen_NativeCores.md` rather than on a fourth bespoke shim, as that page decided on
2026-09-28. What the interface is and how the old ABIs sit beside it is that page's §12. This section is MoonRT's
side: the row, the oracles and what is owed.*

#### 8.3.1 What was built

- **The library.**
  - MoonRT exports the common interface as core 3: the 24 required names, `reset`, `set_mutes` and `set_rom_patches`.
  - Its capabilities are `RESET | MUTES | ROM_PATCHES`, and the host requires all three.
  - Its `moon_*` exports and `moon_interface_version` are gone.
  - Its extensions are `moonrt_step`, which the instruction-level rigs use, and `moonrt_rom_patch`, the patch table's
    test view.
  - Its faults are −321 to −323. They were −31 to −33, and the move is §3.3's.
- **The shim.**
  - `MoonMachine` is a `NativeMachine`, with Moon's band: `Not an iNES image`, the mapper, the truncated PRG, and
    −323's `masterDelta` exception.
  - `MoonRtCore` is a `NativeRtCore<MoonMachine>`. It supplies the header parse and the battery rule
    (`Cartridge.FromImage`, `HasBattery && PrgRam.Length > 0`), the button order, the ports, the spaces, the state
    pre-checks and the mirror.
  - The rigs of stages 1–3 (`MoonRtPair`, the state and machine tests) call the same `MoonMachine` methods as before.
    None of their assertions changed.
- **The row.**
  - `CoreCatalog.EngineFor("NES")` offers *Moon (C#)*, the default and first, and *MoonRT (Rust)*.
  - `CoreFactory` builds MoonRT when it is chosen and `MoonRtCore.Available`, and otherwise Moon with the loader's
    report in the notice.
  - The bundle has Moon's two cheat codecs and MoonRT's debug target over its mirror.
  - The registration record of `EmuSen_NativeCores.md` §5.1 is owed. The row went in the existing way (§12.2 there).
- **The rest of the row.**
  - Battery saves go through `BatterySave` into `Saves/NES`. The file is handed to the core at create.
  - Both cheat codecs, with Game Genie's patches sent as triples.
  - `FrameLog` and the frame-end order, in the base's `EndFrame`, which is Moon's.
  - Exception mapping, with the −320 band in the host.
  - The crash log at `moonrt_crash_<pid>`.
  - The debug target over the mirror.
  - Rewind, through `IEngineFeatures`: kept.

**§9 Q7 is not reached.** MoonRT has no breakpoints yet, so there is no first instruction to check. Stage 5 places
that check in Rust.

#### 8.3.2 The oracles (measured 2026-09-30)

`MoonRtFrontendTests` drives both engines through `CoreFactory` and `ICore`:

| Case | What it holds | Result |
|---|---|---|
| The row and the factory | Moon the default; MoonRT built when named, with no notice; the same codec types; rewind kept; an unknown engine's notice names MoonRT as running | pass |
| The bench games and the library states | The four games for 1,800 frames with the bench's input, and the eleven library states for 600 frames each, loaded into fresh instances. Picture, sound and state are identical after every frame, and the state's size is constant through each run | pass |
| States crossing | A C# state loaded by MoonRT re-saves byte for byte, and runs beside a C# instance loaded with the same state; the reverse the same | pass |
| The battery | A marked save is read by MoonRT, rewritten at frame 300 with the mark kept, and loaded by Moon; the two machines are equal straight after a load with a battery (§3.4 there); Moon's rewrite is read back by MoonRT; a cartridge without a battery reports none; the length equals the C# `PrgRam`'s | pass |
| Game Genie | `SXIOPO` on Super Mario Bros.: 900 frames identical on both engines, and different from the plain run | pass |
| The patch triples | Against `TryPatchRom` at every address from `$4020` to `$FFFF`: every byte at the touched addresses, three probes elsewhere. The registry mixes compares, an unconditional patch behind a compared one, a shadowed repeat, both ends of the range, one below it and a disabled one | pass |
| A read after a new cheat | A CPU-bus read between frames sees a patch added since the last frame, as the C# core does (§9 Q11 there) | pass |
| Capabilities and exports | Each bit's exports are present exactly when it is claimed, every required name is there, and `moon_machine_new` is not | pass |
| A missing library | Refused with "not found beside the assemblies" | pass |
| A reproduced exception | D4's image with no PRG throws the C# type through `MoonRtCore.LoadRom` | pass |

The instruction- and frame-level rigs pass unchanged through the new ABI:

- `MoonRtStateTests`, `MoonRtMachineTests` and `MoonRtSoundAndPictureTests`;
- AccuracyCoin through both engines;
- the corpus test, with sound and picture: 255 ROMs, 363,935 frames identical, 105 passed.

`MoonRtEngineTests` covers Mistress:

- It picks MoonRT from the NES tab's Engine row and plays frames, with rewind filling.
- It saves a state through the hotkey, and the C# Moon loads it.
- It runs Moon by default.
- The fallback runs in a child process started with `EMUSEN_MOON_NATIVE=0`. Mistress runs Moon, and the status bar
  carries "turned off by EMUSEN_MOON_NATIVE=0".
- The missing-library case is held at the loader, not in Mistress. Taking the library away from a running test host
  would take it from every other test too.

The fit audit gains `GraphicsSettingsNesEngine`, the NES tab at 1280×800 and 1920×1200, and passes. So do
`MarsRtEngineTests`, whose check that no other console has an Engine row now admits the NES.

**What passes on the old ABIs:** MercuryRT's and MarsRT's suites, 498 cases, and `NativeHostTests`.

#### 8.3.3 The speed

`EmuSen_NativeCores.md` §12.3 retires P2 as held: the generic host is within 0.7% of the bespoke shim on each game,
with a geometric mean of +0.06%. The same rounds' C# control gives MoonRT 1.28–1.40× C# Moon, as §8.2.3 measured.

P4, P5 and P6 stay open:

- **P4** needs the quota runs, and was not run.
- **P5** needs the boundary priced on its own.
- **P6** holds on the desktop, but the quota runs are what it is to be retired by.

#### 8.3.4 What is left

- **Stage 5:** the debugger over `NativeDebugBridge`. That includes the first instruction's check in Rust (§9 Q7),
  and the `DEBUG` capability.
- **The registration records** (`EmuSen_NativeCores.md` §5.1), and the equivalence test.
- **MercuryRT and MarsRT** on the common interface, at that page's steps 4 and 5, when the legacy classes go.

### 8.4 Stage 5: the debugger on the shared hooks (done 2026-09-30)

*§4's stage-5 row, built on `EmuSen_NativeCores.md` §3.14 and §4.4 with its §9 decisions. The shared data and its
ABI are that page's §12.4. This section is MoonRT's mechanism and oracles.*

#### 8.4.1 The oracle first

The C# Moon had no call stack or profile, and its data breakpoints never fired (`Moon_Debug.md` §7). Each was added
to the C# core before MoonRT, so that MoonRT has something to be graded against. Adding them also found one defect:
a resume from a breakpoint earned the halted scanline's CPU cycles twice. A halted and resumed machine then differed
from an unhalted one in its CPU budget. The same section has the measurement, and both engines now earn the halted
scanline once.

#### 8.4.2 MoonRT's mechanism

- **The hooks** are `emusen_native::debug::Hooks`, boxed in the bus as MercuryRT's and MarsRT's are.
- **The observed frame** is `Machine::run_frame_debug`, which is `RunFrame` with these additions:
  - Before each instruction, the hooks are asked whether to stop. The first instruction is asked too, unless the host
    sends `UNCHECKED`, which it does on a resume and after a stop it refused. That places §9 Q7's check in Rust.
  - Coverage and the profile are recorded.
  - Each store is stamped with its instruction's address.
  - A stop leaves the frame open. The resume, observed or plain, then does not earn the halted scanline again.
- **The seams are compiled out of the plain frame.** The CPU is generic over `CpuBus`, and it is driven through one
  of two views of the bus:
  - `Plain`, used by the plain frame, `step` and reset. Its store is the bus's write, and its call and return notes
    are empty.
  - `Observed`, used by the observed frame. It reports each store under C#'s spaces while writes are armed: `RAM`,
    `PRGRAM` (all of `$4020`–`$7FFF`, as C# reports it), `PPUREG` and `APUREG`, but not `$4014` or `$4016`. It also
    notes JSR, RTS, RTI, BRK and the interrupts, with the call kinds of §12.4 there.
  - A host store to `CPUBUS` is reported while writes are armed, as C#'s bus reports it.
  - *The first version was measured and not kept.* It put a flag in the bus that every store tested, and the flag
    cost 0.8 and 1.2% of the plain frame on Super Mario Bros. 3 and Punch-Out!!. A build with the test removed was
    0.8–0.9% faster than stage 4, so the cost was the branch, not the new field.
- **The host side** is `NativeDebugBridge`, over `NativeRtCore`'s seam.
  - An observed frame runs whenever the registries are not quiet, coverage or the profiler is armed, or a debug
    target listens for stores.
  - The mirror's call stack stamps its pushes with the frame the bridge is draining.

**What is not the C# core's, and why.** MoonRT's call stack moves only in observed frames. A frame with nothing armed
is plain, and its calls reach no registry. So after plain frames the registry's stack stands where the last observed
frame left it, while the C# core's has followed every call. `The_two_engines_halt_and_step_alike` compares the depth
only in observed frames for that reason. It is the rule MercuryRT follows (`Mercury_Native.md` §8.5), and
`Mars_Native.md` §6.5.6 lists the same gap for MarsRT. Keeping the stack in a plain frame would put the seams back into
it. Whether that is worth its cost is not measured.

#### 8.4.3 The oracles (measured 2026-09-30)

**Both engines, one debug target contract.**

- `MoonDebugTargetContract` holds the 24 debug-target cases that go through `IDebugTarget` alone, and runs them once
  per engine, 48 cases in all:
  - memory spaces, disassembly and static references;
  - tiles, nametables, palettes and sprites, each written through the target's own spaces;
  - the bus's side effects, the video registers set through `$2006`, and the vectors;
  - a watch, a breakpoint and its resume, the summary, and the registers after a frame.
- `MoonDebugTargetTests` keeps the five that poke the C# machine directly: register pokes, and the injected and
  measured hardware load.

**Behaviour, on both engines** (`MoonDebugEngineTests`):

- a breakpoint's halt, its program counter, the frame and the call stack's top, and its resume;
- stepping in, over and out;
- run-to NMI and run-to frame;
- a watch logging the CPU's store with its `PC=$8012` context;
- a data breakpoint halting after the store;
- coverage and the profile;
- a store through `CPUBUS` reaching a watch, and one to `$2000` logged under `PPUREG`;
- a halt and resume leaving the machine as an unhalted run.

`NesRunToFrameTests` now halts MoonRT at the start of the frame after the one run to, as Moon.

**Hits identical.** `The_two_engines_halt_and_step_alike` runs one script on both engines and compares after every
call:
- five breakpoint halts and resumes;
- twelve single steps;
- a step out;
- a run to the NMI;
- four data-breakpoint frames;
- three plain frames.
It compares the halt, the program counter, the frame, the call stack's depth and the whole state's hash. All 26 lines
are identical, the plain frames' depth excepted as §8.4.2 says.

**Armed equals plain.** `MoonRtArmedEqualsPlainTests` runs each game twice on MoonRT:
- once armed with a breakpoint no game executes, watches over all of RAM and PRG RAM, a data breakpoint no store
  matches, coverage and the profiler;
- once with nothing armed.
The four bench games run for 900 frames and four library states for 400. Picture, sound and state are byte-identical
after every frame. The armed runs took the observed frame (over 100,000 instructions covered each) and filled their
watches.

**Nothing else moved.** The Moon-filtered and native cases pass, 436 of 436, from a clean rebuild of the library.

**Mutants: 12, all caught** (measured 2026-09-30).

The round ran with the stage-5 filters: the contract, the engine cases, armed-equals-plain and run-to-frame. Its
three survivors were then handled:
- H2 and H6 are caught by the shared crate's own tests, which the round did not run. Each was shown failing there
  with the mutant applied.
- M2 is caught by a PPU-register watch case added for it.

| Mutant | Caught by |
|---|---|
| H1 no breakpoint stops the frame | five cases, the engine script among them |
| H2 a watched store also sets the data stop | the crate's write and full-log tests (a refused stop, so no C# case can see it) |
| H3 stores never stamped with their instruction | the watch's `PC=$8012` context |
| H4 a return pops nothing | stepping out, and the engine script |
| H5 a coverage bit set one address off | the coverage case |
| H6 an IRQ does not set the interrupt stop | the crate's interrupt test over IRQ, NMI and BRK (the NES programs here raise no IRQ) |
| M1 a stop does not leave the frame open | the halt-and-resume cases, the engine script |
| M2 PPU-register stores logged under RAM | `A_watch_on_the_ppu_registers_logs_the_store_under_its_register`, added for it |
| M3 the first instruction never checked | run-to frame |
| O1 the observed frame's budget one cycle off at its end | armed-equals-plain, the engine script |
| O2 a reported store touches the open bus | armed-equals-plain, the engine script |
| G1 the observed view's return note empty | stepping, the engine script, the breakpoint case's stack |

O1 and O2 change only the observed frame. Armed-equals-plain is the oracle built to catch that class, and it caught
both.

#### 8.4.4 The speed (measured 2026-09-30)

Five rounds under the timing lock, interleaved game by game. The comparison is stage 4 (8a8296d6) against this stage,
debug off, with MoonRT and C# both armed as above beside them. The load average was 0.7–1.6. Every run of a game had
one state hash, whether armed or not and on either engine.

| Game | Stage 4, ms/frame | Stage 5 plain | Change | Stage 5 armed | Armed ÷ plain | C# armed ÷ plain |
|---|---|---|---|---|---|---|
| Super Mario Bros. | 0.991 | 0.992 | +0.10% | 1.145 | 1.15× | 1.13× |
| Zelda | 0.902 | 0.900 | −0.18% | 1.026 | 1.14× | 1.12× |
| Super Mario Bros. 3 | 1.313 | 1.300 | −0.97% | 1.551 | 1.19× | 1.15× |
| Punch-Out!! | 1.175 | 1.174 | −0.10% | 1.381 | 1.18× | 1.16× |

- **The plain frame is not slower.** The geometric mean is −0.29%, and only Super Mario Bros. 3's ranges fail to
  overlap, on the faster side.
- **The armed frame costs 14–19%** in MoonRT, about what arming costs the C# core. Armed MoonRT is still faster than
  plain C# Moon on every game.

#### 8.4.5 What is left

- **The call stack in a plain frame** (§8.4.2).
- **Read breakpoints and read watches**, on both engines (`Moon_Debug.md` §7).
- **The DianaOS console against a MoonRT target, and Mistress's debugger window.** They are covered only through the
  interface here.
- **MercuryRT and MarsRT onto the shared hooks**, at `EmuSen_NativeCores.md` §7's steps 4 and 5 (§12.4 there).
