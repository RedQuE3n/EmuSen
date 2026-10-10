# Venus (SNES) — SA-1

Covers everything under `Cores/Nintendo/Venus - SNES/Coprocessors/Sa1/`, plus the `Sa1Mapper` that hangs off `ICartridgeMapper`. The SA-1 is the first coprocessor this core implements, so this page also documents the general seam the *next* one should reuse (§2.1).

---

## 1. What the chip is, and how it gets built

The SA-1 is a whole second computer bolted onto the cartridge: a **65C816 running at 10.74MHz** (twice the S-CPU's fastest speed), **2KB of internal SRAM (I-RAM)** shared with the S-CPU, a bank controller ("Super MMC") that reprojects ROM and BW-RAM under software control, and four fixed-function accelerators — a timer, an arithmetic unit, a variable-length bit stream reader, and a DMA engine.

Detection is the ROM header's **map mode byte at `+$15`**: mode `$23` at the LoROM header location (`$7FD5`) means SA-1. `Cartridge`'s constructor checks it and, when it matches, constructs the chip and installs `Sa1Mapper` instead of `LoRomMapper`:

```
map mode $23, cart type $35, ROM $0C (4MB), SRAM $05 (32KB)  ->  Kirby's Dream Land 3
```

The header's SRAM-size byte sizes **BW-RAM** — on an SA-1 cartridge the save RAM and the coprocessor's bulk work RAM are the same chip, so `Cartridge._sram` is handed straight to the SA-1 and both sides address it.

**Why the black screen without it.** With no SA-1, KDL3's S-CPU still ran, but everything the coprocessor was supposed to compute stayed zero — including the buffer holding the decompressed SPC700 sound driver. The S-CPU uploaded 64KB of zeroes to the APU, the SPC700 jumped to entry point `$0000` and NOP-slid through empty RAM, and the S-CPU parked forever at `$00:DEAC` in `LDA $2140 / BNE` waiting for a sound driver that would never signal ready. `INIDISP` never left `$80`, so the screen stayed in forced blank with VRAM, CGRAM and OAM all still zero. The `$2200`-range writes the game was making went to open bus the whole time.

---

## 2. Running two 65816s

### 2.1 `ICpuBus` — the seam

`Cpu` used to hold a concrete `MemoryBus`. It only ever used four members of it, so those are now an interface (`Cpu/Core/ICpuBus.cs`):

```csharp
byte Read8(uint address);
void Write8(uint address, byte data);
int  GetAccessSpeedCycles(uint address);   // master clocks per access
int  TakePendingDmaCycles();               // cycles stolen by a DMA, cleared on read
```

`MemoryBus` implements it (`TakePendingDmaCycles` wraps the `Dma.PendingCpuCycles` drain that used to be reached through `_bus.Dma`), and so does `Sa1Bus`. The entire 65816 — every addressing mode, every opcode — is then reused unchanged for the coprocessor. **This is the seam any future coprocessor with its own CPU should use**; nothing about it is SA-1-specific.

`Cpu`'s constructor also takes a `name` (tags trace output) and `logReset` (off for the SA-1, whose reset line the game drives directly and may toggle repeatedly).

### 2.2 Timebase

The SA-1 shares the console's master clock rather than having its own crystal, at exactly **two master clocks per SA-1 cycle** (10.7386MHz against 21.4772MHz). That makes synchronisation trivial compared to the SPC700, which needs the fractional-remainder carry described in `Venus_CPU.md` §8.5b: `VenusCore.RunFrame` hands `Sa1.Run()` the same master-clock figure `Cpu.Step()` just returned, and `Sa1Bus.GetAccessSpeedCycles` returns a flat `2` for every address.

`Run()` carries an unspent-clock budget between calls so a partially-affordable instruction isn't executed early or dropped. The two cores are interleaved **per S-CPU instruction**, which is also where the SA-1's IRQ line into the S-CPU is polled.

> Flat 2-cycle timing is a simplification: real BW-RAM access from the SA-1 side costs more than I-RAM. No game is known to depend on the difference, and nothing here measures it.

### 2.3 Verifying the timebase — `ClocksRun` / `ClocksOffered`

Because §2.2's whole claim is *"the SA-1 gets exactly the master clocks the S-CPU just spent"*, it is worth being able to check that rather than trust it. `Sa1` keeps two cumulative counters, both `[SkipInState]` so they never touch the save-state layout:

- **`OfferedMasterClocks`** — every clock handed to `Run()`, whether the CPU executed or not.
- **`ExecutedMasterClocks`** — only the clocks a `Cpu.Step()` actually consumed.

They surface as `ClocksRun` / `ClocksOffered` in `regs`' coprocessor block, so the per-frame rate is a two-`regs` diff around a `frames N`:

```sh
# Pharaoh --commands: regs / frames 60 / regs
ClocksRun = 0x0000000001472E36     # 21,442,102 over 60 frames = 357,368/frame
```

The figure to compare against is `_totalScanlines * VenusCore.CyclesPerScanline` — **357,368** master clocks per NTSC frame (262 x 1364), which is 178,684 SA-1 cycles at §2.2's 2:1 ratio, i.e. a full 10.7386MHz. A shortfall against that means the SA-1 is being starved somewhere; a surplus means it is being clocked twice.

The gap between the two counters is the halted time: `Run()` zeroes the budget without executing when RESB or RDYB is set (§4.1), so `Offered - Run` is how long the game parked the chip. Both were verified equal-and-exact on Kirby's Dream Land 3, on the level-select map and in gameplay alike, while investigating a reported pacing complaint — the SA-1's timebase was ruled out as the cause.

---

## 3. The two memory maps

The Super MMC gives each CPU a *different* view of the same cartridge. `Sa1.ResolveScpu` and `Sa1.ResolveSa1` (`Sa1.Memory.cs`) are the two decoders; both return the same `CartridgeAddress` type the other mappers use, extended with three SA-1 regions — `IRam`, `Sa1Register`, `Sa1Vector`.

| Address | S-CPU sees | SA-1 sees |
|---|---|---|
| `$00-$3F/$80-$BF:0000-07FF` | WRAM (decoded by `MemoryBus`) | **I-RAM** |
| `$00-$3F/$80-$BF:2200-23FF` | SA-1 registers | SA-1 registers |
| `$00-$3F/$80-$BF:3000-37FF` | I-RAM | I-RAM |
| `$00-$3F/$80-$BF:6000-7FFF` | BW-RAM window (`BMAPS`) | BW-RAM window (`BMAP`) |
| `$00-$3F/$80-$BF:8000-FFFF` | ROM, LoROM-style | ROM, LoROM-style |
| `$40-$4F:0000-FFFF` | BW-RAM, flat | BW-RAM, flat |
| `$C0-$FF:0000-FFFF` | ROM, HiROM-style | ROM, HiROM-style |

The SA-1 has **no WRAM and no PPU at all**, which is why it gets I-RAM mirrored into page zero: that is where it puts its direct page and stack.

**No `MemoryBus` change was needed for any of this.** The escape hatch added for HiROM's SRAM window (`Venus_Memory.md` §2.1a — `if (offset < 0x8000 && !_cartridge.MapsAddress(address))`) already lets the mapper claim `$2200-$23FF`, `$3000-$37FF` and `$6000-$7FFF`, and the write path already fell through to the cartridge. `Cartridge.Read8`/`Write8` grew a case per new region and nothing else moved.

### 3.1 Super MMC ROM banking (`$2220-$2223`)

ROM is addressed in four **1MB super banks**. Registers `CXB`/`DXB`/`EXB`/`FXB` each select which super bank appears in one slot:

| Register | LoROM-style slot | HiROM-style slot | Power-on super bank |
|---|---|---|---|
| `$2220` CXB | `$00-$1F` | `$C0-$CF` | 0 |
| `$2221` DXB | `$20-$3F` | `$D0-$DF` | 1 |
| `$2222` EXB | `$80-$9F` | `$E0-$EF` | 2 |
| `$2223` FXB | `$A0-$BF` | `$F0-$FF` | 3 |

**Bit 7 is what makes the register take effect at all.** With it clear the slot keeps its power-on bank regardless of what bits 2-0 say; with it set, bits 2-0 choose. Getting this backwards silently maps a 4MB game entirely onto its first megabyte.

LoROM-style is `(super << 20) | ((bank & 0x1F) << 15) | (offset & 0x7FFF)` — 32 banks of the upper 32KB. HiROM-style is `(super << 20) | ((bank & 0x0F) << 16) | offset` — 16 full 64KB banks. Both cover exactly 1MB.

### 3.2 BW-RAM windows (`$2224`, `$2225`)

Each side gets its own 8KB window at `$6000-$7FFF` and its own register selecting which 8KB block of BW-RAM appears there — `BMAPS` for the S-CPU, `BMAP` for the SA-1. They are independent, which is the point: both CPUs can work on different parts of BW-RAM simultaneously without fighting over one window.

Banks `$40-$4F` are flat BW-RAM on both sides. That is 1MB of address space for at most 256KB of chip, so `Cartridge` mirrors modulo the real size exactly as it does for ordinary SRAM (`Venus_Memory.md` §2.3).

### 3.3 Bitmap mode (`$2225` bit 7, `$223F`)

With `BMAP` bit 7 set, the SA-1's `$6000-$7FFF` window stops being plain memory and presents BW-RAM as a **virtual bitmap**, where each byte in the window is one 2bpp or 4bpp *pixel* (`$223F` bit 7 picks which). Reads mask and shift the sub-byte field out; writes read-modify-write it back.

> Implemented but **not verified against hardware** — no ROM in the local set exercises it. It only engages when a game sets `BMAP` bit 7, so a game that never touches it cannot be affected.

### 3.4 Write protection (`$2226-$222A`)

`SBWE`/`CBWE` (BW-RAM write enable), `BWPA` (protected area), and `SIWP`/`CIWP` (I-RAM write protection, one bit per 256-byte block) are **stored so reads are faithful, but not enforced**. Games set them correctly and then write only where they are allowed; enforcing them can only turn a correct write into a dropped one. Revisit if a game is ever seen corrupting memory it should not reach.

---

## 4. Control, messaging and interrupts

The two CPUs talk over a symmetric pair of registers: each side has a control register it writes and a status register the other side reads.

### 4.1 `$2200` CCNT — the S-CPU's control over the SA-1

| Bit | Meaning |
|---|---|
| 7 | Request IRQ to the SA-1 |
| 6 | `RDYB` — 1 parks the SA-1 CPU |
| 5 | `RESB` — 1 holds the SA-1 CPU in reset |
| 4 | Request NMI to the SA-1 |
| 3-0 | 4-bit message to the SA-1 |

Power-on value is `$20`: **the SA-1 boots held in reset and does nothing until the game releases it.** The `1 -> 0` edge on bit 5 is the boot handshake — it resets the SA-1 CPU, which then fetches its reset vector from `CRV` (`$2203/4`) and starts running. `RDYB` parks the CPU *without* resetting it, so releasing it resumes where it stopped.

### 4.2 `$2209` SCNT — the SA-1's control over the S-CPU

Bit 7 requests an IRQ to the S-CPU, bits 5/4 enable the NMI/IRQ vector overrides below, bits 3-0 are the message back.

Each direction has an enable register (`$2201` SIE / `$220A` CIE) and a write-1-to-clear register (`$2202` SIC / `$220B` CIC). A request latches regardless of the enable bit — the enable only decides whether the line is actually asserted — so `$2300`/`$2301` can show a pending request the receiving CPU has masked off.

`Sa1.ServiceInterrupts` latches whether an interrupt has already been entered (`_sa1IrqTaken`/`_sa1NmiTaken`) so a level-held request doesn't re-enter the handler every instruction.

### 4.3 Vector override

The SA-1 can substitute its own NMI/IRQ vectors for the **S-CPU's**, and hardware does it by intercepting the fetch rather than by patching anything: reads of `$00:FFEA/EB` return `SNV` (`$220C/D`) and `$00:FFEE/EF` return `SIV` (`$220E/F`), gated by `SCNT` bits 5 and 4 respectively. That is why the decoder needs a `Sa1Vector` region — it is an address decode, not a value.

The **SA-1's own** vectors work the same way on its side of the bus but are *unconditional*: `$00:FFFC` reads `CRV`, `$00:FFEA` reads `CNV`, `$00:FFEE` reads `CIV`, with no enable bit. The S-CPU's override bits never leak onto the SA-1 side.

---

## 5. Timer (`$2210-$2215`, `$2302-$2305`)

Two modes, picked by `$2210` bit 7.

**HV mode** counts the video dot clock (master ÷ 4): H wraps at 341, then V increments and wraps at the region's scanline count (`Sa1.TotalScanlines`, set by `VenusCore.LoadRom`). Bits 1/0 enable the V and H compares; with only H enabled it fires every line, with only V it fires once per frame at `H == 0`, with both it fires on the exact pair.

**Linear mode** is a free-running counter incrementing once per SA-1 cycle, compared against `(VCNT << 16) | HCNT`.

`$2302` latches both counters so the 32-bit read is coherent; `$2303-$2305` return the latch. Writing `$2211` resets all counters.

> Linear mode's exact width and compare semantics are approximate — nothing in the local ROM set exercises it.

---

## 6. Arithmetic unit (`$2250-$2254`, `$2306-$230B`)

Three operations selected by `$2250`: bit 0 picks multiply vs. divide, bit 1 selects cumulative (multiply-accumulate). **Writing the control register also clears the accumulator and overflow flag.** Writing the *high* byte of `MB` (`$2254`) is what runs the operation. The 40-bit result reads back at `$2306-$230A`, overflow at `$230B`.

Signedness is the part that is easy to get wrong:

- **Multiply** — both operands signed. Result is 32-bit.
- **Divide** — dividend (`MA`) signed, **divisor (`MB`) unsigned**.
- **Accumulate** — signed products summed into a 40-bit accumulator; overflow sets `$230B`. The accumulator is held as a full signed value so a running sum that dips negative keeps accumulating correctly, and only the low 40 bits are ever exposed.

### 6.1 Division rounding

Hardware **floors toward negative infinity** rather than truncating toward zero, which keeps the remainder non-negative. C#'s `/` and `%` truncate, so a negative dividend needs the correction explicitly:

```
-1000 / 7   truncating -> q = -142, r = -6      (wrong)
            flooring   -> q = -143, r =  1      (hardware)
```

The quotient lands in the low word of the result and the remainder in the high word. Divide by zero yields a zero quotient and returns the dividend as the remainder.

---

## 7. Variable-length bit stream (`$2258-$225B`, `$230C/D`)

A ROM reader that pulls **1-16 bit fields from arbitrary bit offsets**, which is what the SA-1 decompressors are built on — it is why an SA-1 game can store graphics in a packed bitstream and still decode them at speed.

State is a 24-bit ROM address plus a 3-bit offset within the byte. The 16 bits currently under the cursor are assembled little-endian and shifted down:

```csharp
uint window = rom[address] | (rom[address+1] << 8) | (rom[address+2] << 16);
return (ushort)(window >> bitOffset);
```

Three bytes are needed because a 16-bit field at bit offset 7 spans three of them. `$2258` bits 3-0 set the field width (**0 means 16**, not 0) and bit 7 picks how the cursor advances: **auto** advances on each read of `$230D` (the high byte, i.e. after the whole field has been read), **fixed** advances when `$2258` itself is written. Writing the *high* address byte (`$225B`) restarts the stream at bit 0.

---

## 8. DMA (`$2230-$2239`, `$2240-$224F`)

**Normal DMA** is implemented: a block copy between ROM / BW-RAM / I-RAM, with the source device in `$2230` bits 1-0 and the destination in bit 3. The trigger is whichever byte *completes* the destination address — `$2236` for an I-RAM destination, `$2237` for BW-RAM. Same-device transfers (BW-RAM to BW-RAM, I-RAM to I-RAM) are ignored by hardware. ROM sources read through the SA-1's own map, so the Super MMC applies; BW-RAM and I-RAM sources are flat device offsets. Completion raises the DMA IRQ flag on both sides.

The transfer is modelled as **instantaneous** rather than stealing cycles, which is why `Sa1Bus.TakePendingDmaCycles` always returns 0.

### 8.1 Character conversion — not implemented

Both character-conversion modes are **decoded and reported but not converted**:

- **Type 1** is driven by the S-CPU reading through the `$6000-$7FFF` window mid-transfer, with the SA-1 converting linear bitmap data into planar SNES tiles on demand.
- **Type 2** is driven by the SA-1 staging pixel data into the `BRF` register file (`$2240-$224F`) and the hardware converting each half as it is completed.

A game that needs either gets **wrong tile data rather than a hang**, and `Sa1Dma` prints one line saying so instead of failing silently. KDL3 never requests either, which is why it renders correctly without them. This was left unimplemented deliberately: the bitplane layout is straightforward but the exact trigger and addressing semantics are not something to guess at, and shipping a plausible-looking guess would be worse than a stated gap.

---

## 9. Save states

The SA-1's state is **appended after the four original blobs**, and only for an SA-1 cartridge — see `EmuSen_Save_States.md` §3. `Cartridge.Sa1` is `[SkipInState]` precisely so that a state version 1 file, which predates the SA-1 entirely, still has exactly the layout it was written with. A v1 file cannot be an SA-1 game, because SA-1 games could not run when v1 was the current format.

`StateSerializer` also had to learn that an **interface-typed field** (`Cpu._bus`, now a `MemoryBus` for the S-CPU and an `Sa1Bus` for the SA-1) is walked like a class — `Type.IsClass` is false for interfaces, so it would otherwise have thrown on the first save.

Verified by saving mid-gameplay in KDL3, reloading, running one frame, and comparing against the continuous run: byte-identical.

---

## 10. Status

Boots and plays **Kirby's Dream Land 3** — HAL logo, intro cutscene, title screen, and level 1 gameplay all render correctly, with the sound driver uploading properly.

Known gaps, all called out above: character-conversion DMA (§8.1), BW-RAM bitmap mode unverified (§3.3), linear timer mode approximate (§5), write protection stored but not enforced (§3.4), and flat 2-cycle SA-1 bus timing (§2.2).

Other coprocessors (SuperFX, DSP-*, CX4, S-DD1) remain unimplemented. The `ICpuBus` seam (§2.1) and the `CartridgeRegion` extension (§3) are the parts of this work that generalise.

---

## 11. Debugging the chip

Everything above describes emulating the SA-1. This section is about *inspecting* it, which for a long time the toolchain could barely do: `regs` grew a coprocessor block early, but `mem`, `disasm`, `watch`, `search`, `snapshot` and `bp` all saw only the S-CPU's world. The chip that runs the game's actual logic was the one part of the machine you could not point a debugger at.

The rule this section follows is the one `IDebugTarget.CoprocessorRegisters` already set: **a debugger must never change what it observes.**

### 11.1 Side-effect-free reads

`ReadRegister` has exactly two side effects — `$2302` re-latches the timer counters (§5) and `$230D` advances the bitstream cursor (§7). `DebugPeekRegister` is the same switch with those two answered from the existing latch instead, and `DebugReadSa1` is `ReadSa1` routed through it. Everything a debug space reads goes through those, so pointing `mem` at a live chip cannot perturb it.

### 11.2 `BWRAM` — the space that was missing entirely

BW-RAM is the SA-1's main work and save RAM, and until it got its own space it was **completely invisible**. The `SRAM` space is a `BusDebugMemorySpace` anchored at `$70:0000`, and an SA-1 cart maps BW-RAM at `$40-$4F` (§3) — bank `$70` decodes to `Unmapped`, so `mem SRAM` faithfully reported 32KB of zeroes on Kirby's Dream Land 3 while the game was actively using it.

`BWRAM` wraps the array directly, so it is inert and flat-indexed. Note that `Cartridge` hands the SA-1 the *same* array it uses for `_sram` — BW-RAM and cartridge save RAM are one buffer, which is why an SA-1 cart publishes it under the name that reflects what the chip calls it.

### 11.3 `SA1BUS` — the chip's own address space

The two CPUs see genuinely different maps off the same cartridge (§3), so an S-CPU-shaped view cannot answer "what is the SA-1 executing". `SA1BUS` is a 24-bit `DelegateDebugMemorySpace` over `DebugReadSa1`, Super MMC banking applied, which makes the whole existing toolchain work on SA-1 code for free:

```sh
regs                       # Coprocessor block: PB/PC of the second CPU
disasm SA1BUS 82D7 12      # what it is actually running
```

**`disasm` picks its decoding CPU from the space.** Immediate-operand widths come from M/X/E, and using the S-CPU's flags to decode SA-1 code silently mis-lengths every immediate — `LDA #$0001` decoded as `LDA #$01` desynchronises the stream and the rest of the listing becomes garbage (`BRK #$8F` and similar). `DecodingCpuFor` maps `SA1BUS`/`SA1IRAM` to `Sa1.Cpu`; everything else keeps the S-CPU. The caveat in `Disassemble`'s own comment still applies — flags are those of *right now*, so anywhere other than the current PC is best-effort.

### 11.4 Watch coverage on both sides

Two separate paths write the SA-1's memories, and neither passed through anything the watch registry could see:

| Writer | Path | Space reported |
|---|---|---|
| The SA-1 itself | `Sa1.WriteSa1` | `SA1IRAM` / `BWRAM` |
| Its DMA | `Sa1.WriteBwRamByte` | `BWRAM` |
| The S-CPU | `Cartridge.Write8` | `SA1IRAM` / `BWRAM` |

`MemoryBus.Write8` routes cartridge writes straight to `Cartridge.Write8` without notifying its observer, so `Cartridge` now carries a `WriteObserver` of its own. That also fixes plain `SRAM` watches, which never fired on any game.

Writes the chip makes itself go through `IWriteObserver.OnCoprocessorWrite` (a defaulted interface member, so other observers are unaffected) purely so the event is labelled with the **SA-1's** PC. The S-CPU's PC is meaningless for a write the second CPU made, and a log that mislabels them is worse than one that omits them:

```
      24x  W  SA1 PC=0xC22698     <- the coprocessor
      16x  W  PC=0xC22709         <- the S-CPU
```

### 11.5 Breakpoints on the second CPU

`Breakpoints` and `CoprocessorBreakpoints` are **separate registries**, because the two CPUs run different code at the same addresses — an SA-1 game's `$00:82D7` is not the S-CPU's `$00:82D7`. `IDebugTarget.CoprocessorBreakpoints` defaults to null, so a core with no separately-steppable coprocessor is unaffected.

The shell reaches it with an optional scope word, leaving the existing forms untouched:

```sh
bp add 8000            # S-CPU, exactly as before
bp sa1 add 0082D7      # the coprocessor
bp sa1 list
```

`Sa1.Run` checks `BreakpointChecker` before each instruction and returns early **with `_clockBudget` intact**, so the unspent clocks are not lost and resuming re-enters exactly where it stopped. `VenusCore.RunFrame` polls `HaltedAtBreakpoint` after `Run()` and unwinds out of the frame, recording `_haltedOnCoprocessor` so that resuming arms the skip-one-check flag on *that* CPU — arming the S-CPU's for an SA-1 halt would let the SA-1 re-break on the same PC forever. `IsHaltedOnCoprocessor` lets a frontend say which chip stopped.

`EmuSen.Pharaoh`'s `FrameRunner` now notices the halt, stops the batch and reports it, so this works headlessly. A halted frame is deliberately **not** counted, snapshotted for rewind, or reported as advanced — `RunFrame` returned mid-frame, so it did not happen:

```
> bp sa1 add 0082D7
SA-1 breakpoint #1 added at $0082D7.
> frames 30
[BREAK] SA-1 halted at $0082D7 (frame 0).
```

### 11.6 Is the chip running at rate?

`perf` reports the figure §2.3's counters exist for, against the `MasterClocksPerFrame` a full-rate frame allows:

```
  sa-1: 357368 clocks/frame run of 357368 offered (100.0% of the 357368 a full-rate frame allows)
```

A shortfall against 100% means the chip is being starved or is spending time halted (§4.1); a surplus means it is being clocked twice. This is the measurement that rules the SA-1's timebase in or out of a pacing complaint in one line, rather than by inference.
