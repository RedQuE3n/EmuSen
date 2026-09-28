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
  `emusen-state` crate** holding the writer, the reader, the layout and the naming rule, extracted after MoonRT's stage
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
| P1 | A line-for-line MoonRT's plain frame takes 0.69–0.99 of C#'s time, central 0.83 (1.2×), on the four games | Stage 2, like for like with pixel writes skipped; final at stage 4 through the shim |
| P2 | The idle machine (the `JMP` loop) runs in 0.60–0.77 ms against C#'s 0.77 | Stage 2 |
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
