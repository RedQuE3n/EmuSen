# Moon (NES) — Core, timing and address spaces

Covers `Cores/Nintendo/Moon - NES/MoonCore*.cs`: the `ICore` implementation, the Crystal schedule, save states, and the named address spaces everything else reads through. See `Moon_CPU.md` for the processor, `Moon_PPU.md` for video, `Moon_Memory.md` for the bus and cartridge, and `Moon_Debug.md` for `IDebugTarget`.

---

## 1. What `MoonCore` is

`MoonCore` owns `Cartridge`, `Cpu`, `MemoryBus`, `Ppu` and `Apu`, and drives them one frame at a time. It is split three ways: `MoonCore.cs` (the `ICore` surface and save states), `MoonCore.Schedule.cs` (the Crystal half), and `MoonCore.Spaces.cs` (§4).

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

`ClockAccumulator` exists for exactly this and carries the remainder, so the CPU gets 113 or 114 cycles per line and lands on 29,780 a frame. `Crystal`'s own doc calls this out; Moon is the first core to actually need it, since Venus's 1364/6 divides evenly.

A second, independent carry runs alongside it: an instruction can overshoot the end of its budget by a few cycles, so `_cpuBudget` is allowed to go negative and the overshoot is simply owed to the next span. Both together are what make `The_scheduler_lands_exactly_on_a_frame_boundary` hold.

### 2.2 One event, like Venus

`Scheduler` is driven with a single `ScanlineBoundary` event, for the same reason Venus has only one: at this granularity nothing happens between scanline boundaries that the scheduler would need to order. When per-dot timing arrives (§3), that stops being true and this is where the extra events go.

---

## 3. Known deviation: the frame is drawn a line at a time, after the fact

`EndScanline` renders line N *after* the CPU has run line N's worth of cycles. Real hardware draws the line as those cycles execute.

The consequence is specific: a mid-scanline write to `$2005`/`$2006`/`$2001` takes effect for the whole line rather than from the dot it happened at, and a sprite-0 hit is reported at the end of the line instead of at the pixel that caused it. Games that split the screen at a scanline boundary — which is most of them, since that is what sprite-0 and mapper IRQs are for — behave correctly. Games that change scroll mid-line, or poll sprite 0 expecting sub-scanline resolution, will not.

This is the same tradeoff `Venus` documents and the same one `README.md` lists as a known gap there. It is a real limitation, not a temporary shortcut that happens to work: fixing it means moving to per-dot stepping, which is a rewrite of this file's timing loop rather than a patch to it.

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

Magic `MOON` little-endian, version 1. The blob carries the frame counter, the current scanline, the line-start clock, the CPU budget carry, Crystal's whole timeline span, and then `StateSerializer` blobs for the cartridge, the mapper, the CPU, the bus, the PPU and the APU.

The mapper is written separately from the cartridge because it is a different object with its own banking state; a state that restored the cartridge but not the mapper would resume with the wrong bank mapped and jump into the wrong code immediately.

`PrgRom` is `[SkipInState]` — it is the ROM file, is identical on both sides of a save, and would otherwise put a copy of the whole cartridge in every state. `Chr` is **not** skipped, because on a CHR-RAM board it is live video memory the game writes to.

Loading a state whose magic is not `MOON` throws rather than reinterpreting the bytes, so pointing `loadstate` at a Venus state fails with a message instead of a corrupted machine.

---

## 6. Status

Implemented: the CPU (`Moon_CPU.md`, validated), the cartridge and five mappers, the PPU's registers and scanline rendering, controllers, OAM DMA, save states, cheats, the frame log, breakpoints and coverage.

Not implemented: audio synthesis (`Moon_APU.md`), PAL, per-dot timing (§3), rewind hooks beyond what `ICore` gives for free, and mapper 4 (MMC3), which is the most significant single gap for library coverage — see `Moon_Memory.md` §4.
