# Moon (NES) — Core, timing and address spaces

Covers `Cores/Nintendo/Moon - NES/MoonCore*.cs`: the `ICore` implementation, the core's own timeline, save states, and the named address spaces everything else reads through. See `Moon_CPU.md` for the processor, `Moon_PPU.md` for video, `Moon_Memory.md` for the bus and cartridge, and `Moon_Debug.md` for `IDebugTarget`.

---

## 1. What `MoonCore` is

`MoonCore` owns `Cartridge`, `Cpu`, `MemoryBus`, `Ppu` and `Apu`, and drives them one frame at a time. It is split three ways: `MoonCore.cs` (the `ICore` surface and save states), `MoonCore.Schedule.cs` (the timeline — §2), and `MoonCore.Spaces.cs` (§4).

`Cart`/`Cpu`/`Bus`/`Ppu`/`Apu` are public properties beyond anything `ICore` asks for. That is the same deliberate escape hatch `VenusCore` documents: `ICore` exists to kill the duplicated frame loop, not to hide the hardware from a debug target or a frontend that legitimately needs the concrete machine. `MoonDebugTarget` is built entirely on those properties.

`ICore` deliberately does not abstract input, so `MoonCore.SetButton(port, NesButton, pressed)` sits on the concrete type — exactly as `ICore.cs`'s own header comment anticipates.

---

## 2. Timing

One master clock, NTSC only:

| Quantity | Value |
|---|---|
| Master clock | 21,477,272 Hz |
| Master clocks per CPU cycle | 12 |
| Master clocks per PPU dot | 4 |
| Dots per scanline | 341 |
| Master clocks per scanline | 1364 |
| Scanlines per frame | 262 |
| Frame rate | 21477272 / (262 × 1364) ≈ **60.0985 Hz** |

**PAL is not implemented.** A PAL NES runs 312 scanlines off a different crystal with a 3.2 CPU-clocks-per-dot ratio, and the iNES header's TV-system bits are unreliable enough that guessing would be worse than not trying. NTSC is assumed unconditionally.

### 2.1 Why the CPU budget needs an accumulator

1364 master clocks per scanline divided by 12 per CPU cycle is 113.667 — not a whole number. Truncating it every line would lose two thirds of a cycle each time and drift by roughly 175 cycles a frame, which is about six scanlines' worth of CPU time a second.

`EarnCpuCycles` carries the remainder in `_cpuRemainder`, so the CPU gets 113 or 114 cycles per line and lands on 29,780 a frame. Moon is the only core that needs the carry at all, since Venus's 1364/6 divides evenly. `The_cpu_budget_does_not_drift_from_the_master_clock_across_many_frames` is what pins it: truncating instead would lose ~175 cycles a frame and the test's ±8 window would not survive twenty of them.

A second, independent carry runs alongside it: an instruction can overshoot the end of its budget by a few cycles, so `_cpuBudget` is allowed to go negative and the overshoot is simply owed to the next span. Both together are what make `The_timeline_lands_exactly_on_a_frame_boundary` hold.

### 2.2 The timeline, after the PPU took its own clock — and after Crystal came out

`MoonCore.Schedule.cs` is four fields and two methods. There is no scheduler object, no event id and no handler interface; a scanline boundary is `_lineStartClock + MasterClocksPerScanline` and closing one is `EndScanline`.

| Field | What it is |
|---|---|
| `_masterClock` | the core's public clock (`MasterClock`), pinned by `The_timeline_lands_exactly_on_a_frame_boundary` |
| `_lineStartClock` | where the line being paced began; `NextScanlineBoundary` is the CPU's deadline |
| `_cpuRemainder` | the 113.667-cycles-per-line carry — §2.1 |
| `_cpuBudget` | cycles owed forward when an instruction overshoots its span |

**Two predictions died here, and both are worth recording.** The first: this section used to expect that per-dot timing would mean *more* scheduler events. It went the other way — the PPU wanted a clock of its own rather than a queue of appointments, which is the shape Mesen uses too, and `_currentScanline` was removed outright when it became state nothing read.

The second followed from it. Once the PPU had its own clock, Moon's remaining use of `EmuSen.Crystal` was one event that fired at a fixed stride and drove no hardware — a pacing tick expressed through an event table, a handler interface and a runaway guard, none of which it needed. Venus was in the same position for the same reason. A scheduler earns its keep when several devices contend for a timeline at times a game chooses at runtime; with one appointment per core at a constant interval, it was ceremony. The whole assembly was folded back into the two cores on 2026-08-05 and deleted; the retired design doc is `Man pages/Old/EmuSen_Crystal_Scheduler.md`, and §7.1 there is still the best statement of *why* a lagging device wants a clock and a sync point rather than an event id — the argument that survives its own mechanism.

---

## 3. Partly-resolved deviation: the frame is still drawn a line at a time

**The clock is now per dot; the renderer is not.** As of 2026-08-05 the PPU advances three dots per CPU cycle from inside `MemoryBus.Tick`, so every flag and every mapper A12 edge lands on the dot hardware puts it on (`Moon_PPU.md` §1). What is still per line is the *composition*: `RenderScanline` draws line N in one go at dot 256.

So the remaining consequence is narrower than it was: a mid-scanline write to `$2005`/`$2006`/`$2001` still takes effect for the whole line, and a sprite-0 *hit* is still reported for the line rather than at the pixel that caused it. Vblank, NMI, the odd-frame dot skip and mapper IRQs are no longer approximations.

Finishing it means a real fetch pipeline with shift registers, which is a rewrite of `Ppu.Render.cs` rather than of this file — the timing loop this section used to blame has already moved.

### 3.1 What drives a frame now

`MoonCore` no longer counts scanlines. `Ppu.FrameComplete` is set when the PPU wraps past the pre-render line, and `RunCpuUntilBudgetSpent` returns on it. `NextScanlineBoundary` still exists, but only to pace how many CPU cycles run between checks — §2.2's "when per-dot timing arrives, this is where the extra events go" turned out not to be the shape it took, because the PPU ended up owning its own clock instead.

---

## 4. Named address spaces

`MoonCore.Spaces.cs` is the single place that maps a space name to real storage. Three consumers share it: `CheatRegistry.ApplyAll`, `FrameLogRegistry.RecordFrame`, and every `IDebugMemorySpace` the debug target publishes.

| Name | Backing | Writable | Notes |
|---|---|---|---|
| `RAM` | 2 KB work RAM | yes | mirrored to `$1FFF` on the bus |
| `PRGROM` | cartridge PRG | no | a debug write must not corrupt the image |
| `PRGRAM` | cartridge PRG RAM | yes | battery-backed when the header says so |
| `CHR` | pattern tables | only if CHR RAM | a CHR ROM board silently refuses |
| `CIRAM` | nametable RAM | yes | 4 KB, board mirroring applied on read |
| `OAM` | sprite RAM | yes | 256 bytes |
| `PALETTE` | palette RAM | yes | 32 bytes, backdrop holes as in `Moon_PPU.md` §2.4 |
| `CPUBUS` | the live bus | yes | **has side effects** — see below |

`CPUBUS` is the only space that declares `HasSideEffects`. That is not caution: a read of `$2002` clears the vblank flag and resets the address latch, and a read of `$2007` advances the VRAM pointer. A `mem CPUBUS 2002` from the prompt genuinely changes what the running game sees, and the flag is how the shell knows to say so.

---

## 5. Save states

Magic `MOON` little-endian, **version 3**. Version 2 was 2026-08-05, because the per-dot PPU and the cycle-accurate APU both added fields and `StateSerializer` writes fields positionally; version 3 is the same day, when the timeline folded out of Crystal and stopped being a nine-`long` span. A stale state would otherwise have passed the magic and version checks and then been read as garbage, which is the exact failure this header exists to prevent. Anything saved before that date is now rejected with a message.

The blob carries the frame counter, the line-start clock, the CPU budget carry, the master clock, the CPU-cycle remainder, and then `StateSerializer` blobs for the cartridge, the mapper, the CPU, the bus, the PPU and the APU.

Version 3 is slightly *more* than a re-layout: Crystal's span held `Now` and eight event slots, seven of which were always `Never`, and it had nowhere to put `ClockAccumulator`'s remainder — so §2.1's carry was silently dropped on every load. `_cpuRemainder` is written now, which is why a v3 state resumes on the same sub-cycle phase it was saved on.

The mapper is written separately from the cartridge because it is a different object with its own banking state; a state that restored the cartridge but not the mapper would resume with the wrong bank mapped and jump into the wrong code immediately.

`PrgRom` is `[SkipInState]` — it is the ROM file, is identical on both sides of a save, and would otherwise put a copy of the whole cartridge in every state. `Chr` is **not** skipped, because on a CHR-RAM board it is live video memory the game writes to.

Loading a state whose magic is not `MOON` throws rather than reinterpreting the bytes, so pointing `loadstate` at a Venus state fails with a message instead of a corrupted machine.

---

## 6. The RESET button

`MoonCore.Reset()` is a *soft* reset — the console's RESET line, not its power switch. It is the first one in this project: neither core had a reset path before, and no frontend offers one yet. It exists because the blargg test-ROM protocol has a status that means "press RESET now" and a suite that cannot be run without honouring it (`Moon_TestRoms.md` §2.2).

The whole distinction is what survives:

| | `LoadRom` (power-on) | `Reset()` (RESET line) |
|---|---|---|
| Work RAM | cleared | **kept** |
| PRG RAM / battery save | reloaded from disk | **kept** |
| CIRAM, palette, OAM | cleared | **kept** |
| `PPUCTRL`, `PPUMASK`, `$2005`/`$2006` latch, read buffer | cleared | cleared |
| `PPUSTATUS`, `v`, `t`, fine X | cleared | **kept** |
| Every APU channel, the frame sequencer | cleared | cleared |
| Mapper bank registers | reconstructed | **kept** |
| CPU `A`/`X`/`Y` and flags other than `I` | cleared | **kept** |
| CPU `S` | 0 → `0xFD` | **decremented by 3** |
| CPU `PC` | vector at `$FFFC` | vector at `$FFFC` |

Each piece is a `SoftReset()` on the component that owns the state, and the component's own `Reset()` now calls it and then clears the rest — so the two paths cannot drift apart by having the hard one forget something the soft one does.

The CPU rows are what `blargg`'s `cpu_reset/registers` checks, and it states the rule in one line: *"Reset should set I flag, subtract 3 from S, nothing more."* `Cpu.Reset()` was doing power-on semantics — zeroing `A`/`X`/`Y` and slamming `S` to `0xFD` — so a soft reset wrongly wiped the registers. The two are now split, and the power-on path expresses `0xFD` the way hardware produces it: set `S` to 0, then let `SoftReset`'s three phantom pushes take it to `0xFD`. `ResetStackPointer` survives as the documented constant rather than as an assignment.

`Apu.SoftReset()` clears `$4015` and restarts the sequencer but deliberately does **not** touch the triangle's control/halt flag — `apu_reset/len_ctrs_enabled` pins exactly that, by halting the triangle before the reset and requiring its length counter to survive where the other three drain.

Two of these are worth stating rather than leaving to the table. The **PPU keeps `v`/`t`** because hardware leaves them undefined across a reset rather than zeroing them, and a game that relies on either is broken on real hardware too. **Mapper registers are kept** because most boards genuinely have no reset line wired to them; the boards that do (a few discrete ones latch bank 0) are not among the ten implemented here, so there is nothing to special-case yet.

The scheduler is re-armed from scratch, which restarts the master clock at zero. Nothing outside the frame loop reads absolute clock across a reset, so this is invisible — but it does mean a reset lands on a frame boundary rather than mid-line, which is not what hardware does. At scanline granularity (§3) that is already the resolution of everything else here.

---

## 7. Status

Implemented: the CPU (`Moon_CPU.md`, validated), the cartridge and ten mappers including MMC3, the PPU's registers and scanline rendering, all five APU channels (`Moon_APU.md`), controllers, OAM DMA, save states, cheats, the frame log, breakpoints and coverage.

Not implemented: PAL, per-dot timing (§3), rewind hooks beyond what `ICore` gives for free, expansion audio, and the mappers past the ten in `Moon_Memory.md` §4 — which is now the largest single gap for library coverage, at 16.2% of the local set.

Only the CPU is validated against ground truth. The PPU, APU and mappers rest on synthetic fixtures and four commercial ROMs; the blargg test-ROM suites are the standing gap — see `Moon_TestRoms.md`.
