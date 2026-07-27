# Venus (SNES) — Memory & Bus

Covers everything under `Cores/Nintendo/Venus - SNES/Memory/`: the CPU-side address bus, cartridge (LoROM) mapping, DMA/HDMA, the NMI/IRQ subsystem, and the hardware multiply/divide unit. This is the first page in a new per-hardware-component documentation set (see `Man pages/Hardware/README.md` for why this set exists and how it relates to the rest of `Man pages/`).

---

## 1. `MemoryBus` (`Memory/MemoryBus.cs`) — CPU address decode

The 65816's single 24-bit address space, decoded into whichever subsystem actually owns a given address. `MemoryBus` itself is deliberately thin — it dispatches, it doesn't implement. Genuinely separate pieces of hardware (DMA, the interrupt subsystem, the math unit, PPU registers, the APU port) each own their own state and logic in their own class; `MemoryBus.Read8`/`Write8` is just the address-decode switchboard connecting a raw address to the right one.

### 1.1 Hardware bank vs. general bank

Banks `$00-$3F` and `$80-$BF` are "hardware banks" — their low 8KB (`$0000-$1FFF`) mirrors WRAM, their `$2000-$5FFF` region carries every memory-mapped register, and their upper 32KB (`$8000-$FFFF`) is ROM. Every other bank (`$40-$6F`, `$70-$7D`, `$7E-$7F`, `$C0-$FF`) has no register window at all — `$7E-$7F` is full linear WRAM, `$70-$7D`/`$F0-$FF` is SRAM, everything else is ROM via the cartridge's LoROM mirroring.

### 1.2 Register dispatch table

Within a hardware bank's `$2000-$5FFF` window, `Read8`/`Write8` route by offset:

| Offset range | Owner | Notes |
|---|---|---|
| `$2100-$213F` | `Ppu` | video registers |
| `$2140-$217F` | `Spc700` (APU port) | mirrored across the whole range, not just the canonical `$2140-$2143` |
| `$2180-$2183` | WMDATA/WMADDL/M/H | second, independent path into WRAM — see §1.3 |
| `$4016-$4017`, `$4218-$421B` | `Input` | joypad serial + auto-read registers |
| `$4200`, `$4207-$420A` | `InterruptController` | NMI enable, H/V-IRQ timer setup |
| `$4202-$4206` | `MathUnit` | multiply/divide operands + triggers |
| `$420B`, `$420C` | `Dma` | general DMA trigger, HDMA enable |
| `$420D` | `MemoryBus.FastRomEnabled` | MEMSEL bit 0 — FastROM enable, see §1.6 |
| `$4210-$4212` | `InterruptController` | RDNMI, TIMEUP, HVBJOY reads |
| `$4300-$437F` | `Dma` | per-channel DMA/HDMA registers |
| everything else in `$0000-$7FFF` not listed above | open bus | see §1.4 |
| `$8000-$FFFF` | `Cartridge` (ROM) | never intercepted by the register decode — see the guard note below |

**The `offset < 0x8000` open-bus fallback must never catch `offset >= 0x8000`.** An earlier version of this decode lacked that guard and intercepted ROM reads too, including the reset vector fetch at `$00:FFFC-FFFD` — the CPU came up with `PC=$000000` instead of `$008000` and never executed a single real instruction. Caught via a still-black-screen report right after that change shipped; the guard is now load-bearing, not incidental.

### 1.3 WRAM's three access paths

WRAM (`Ram[]`, 128KB) is reachable three separate ways on real hardware, and all three matter for anything observing WRAM traffic (see §5):

1. **Direct bank access** — `$7E:0000-$7E:FFFF` and `$7F:0000-$7F:FFFF`, the full linear 128KB.
2. **Low-bank mirror** — `$00-$3F`/`$80-$BF` bank's own `$0000-$1FFF` mirrors the first 8KB of WRAM, for zero-page/stack-adjacent code that doesn't want to spend an extra bank byte.
3. **WMDATA port (`$2180`)** — reads/writes the byte at a 17-bit address held in `$2181-$2183` (WMADDL/M/H) and auto-increments it afterward. A second, independent route into the same 128KB array, used by code that wants to touch WRAM from a context where direct `$7E/$7F` addressing isn't convenient. WMADDL/M/H are write-only per hardware docs (reading them isn't a defined register), so they're deliberately not handled as readable. Confirmed via the SNESdev wiki's MMIO registers page (mirrored at snesdev.mesen.ca) and fullsnes; previously `$2180-$2183` fell through entirely unhandled.

All three paths call `ReadObserver`/`WriteObserver` (see §5) — missing any one of them was a real, found-the-hard-way bug: the Yoshi/coin WRAM investigation kept showing "zero writes" to a range that clearly held real, varying data, because the WMDATA and low-bank-mirror paths weren't reporting to the watch registry yet. Fixed one path at a time; all three are covered now.

### 1.4 Open bus modeling

Real SNES hardware retains the last value driven onto the data bus (`_lastBusValue`), and answers a read from unmapped memory with whatever that was — not a fixed `0x00`. Confirmed via the SNESdev wiki's dedicated Open Bus page. `_lastBusValue` is updated on *every* read or write, hardware-bank or not, so even a write to a nonexistent register still affects what a subsequent open-bus read sees.

This matters beyond the generic unmapped-memory case: **RDNMI (`$4210`), TIMEUP (`$4211`), and HVBJOY (`$4212`) only drive SOME of their 8 bits on real hardware** — the rest read back open bus rather than a fixed 0, which is what this project did before `_lastBusValue` existed. `InterruptController`'s read methods take `lastBusValue` as a parameter for exactly this reason.

**Not modeled:** open-bus decay (~2 frames on real hardware, per the same SNESdev page). Accepted gap — no game is known to depend on it, and modeling decay would need a per-bit-line timestamp rather than one plain byte.

### 1.5 H-blank approximation

HVBJOY's H-blank bit (`$4212` bit 6) is approximated from `LineCycles >= 1099` against the per-scanline cycle budget (`VenusCore.CyclesPerScanline = 1364`, real master clocks per scanline — see §1.6/§3.1 and `Venus_CPU.md` §8.5), rather than tracked per-dot — real hardware's H-blank begins around dot 274 of 341 (~80% through the line; `1099 = 1364 * 183/227`, preserving the same ~80.6% threshold this constant used before `CyclesPerScanline` was rescaled from CPU-cycle-units to real master clocks). Games commonly poll this bit to synchronize timing-sensitive work with H-blank; without *some* approximation here, such a wait loop never exits. A known scanline-level timing simplification, consistent with the rest of this emulator's timing model (see §4's NMI-timing note for the same granularity applied to interrupts).

### 1.6 MEMSEL (`$420D`) — FastROM enable

Bit 0 selects FastROM (6 master clocks/access) vs. SlowROM (8) for the `$8000-FFFF` window of banks `$00-$3F`/`$80-$BF` and all of `$C0-FF` — everywhere else on the bus is a fixed speed regardless of this bit (banks `$40-$7D`/`$7E-$7F` always slow; the `$2000-3FFF`/`$4200-5FFF` register windows always fast; `$4000-41FF` always a slow 12). Previously entirely unhandled — a write here fell through to the generic open-bus fallback, so this project always behaved as SlowROM no matter what a game actually wrote. Read via `MemoryBus.GetAccessSpeedCycles(address)`, which `Cpu.Step()` now uses for real per-instruction master-clock accounting — see `Venus_CPU.md` §8 for the full story of why this exists and what changed alongside it.

---

## 2. `Cartridge` (`Memory/Cartridge.cs`) — LoROM mapping + save data

### 2.1 ROM loading

Strips a 512-byte copier header if present (detected by `fileBytes.Length % 32768 == 512` — a headerless LoROM divides cleanly by 32KB, a headered one doesn't).

### 2.2 SRAM sizing from the ROM header

The RAM Size byte at SNES address `$00:FFD8` (ROM file offset `$7FD8` for LoROM) encodes SRAM size as `1KB << N`; `N=0` means no SRAM. Verified via the SNESdev wiki's ROM header page and a WLA-DX header example. Clamped to `N<=7` (512KB, SnesLab's documented real-hardware maximum) defensively against a corrupt header.

**This used to be hardcoded to 2KB for every game** — happens to match Super Mario World, wrong for anything else. Super Metroid's RAM Size byte is 3 (8KB), and its boot-time anti-piracy check specifically depends on the *real* chip size (see §2.3) — an undersized array made that check fail.

### 2.3 SRAM mirroring — why it's `% _sram.Length`, not a hard range check

SRAM is mapped to the lower 32KB (`$0000-$7FFF`) of banks `$70-$7D` and `$F0-$FF`. Real SRAM chips only decode as many address lines as their actual size needs, so a chip smaller than the full 32KB window mirrors repeatedly within it — modulo addressing reproduces that for free.

**Super Metroid's boot-time anti-piracy check deliberately writes a test pattern through one mirror address and reads it back through another**, specifically to detect cartridges/emulators that don't mirror correctly (see tcrf.net/Super_Metroid). This previously used a hard range check against the array length instead of wrapping — anything past the (also wrong, hardcoded-2KB) array size silently read as open bus rather than mirroring, failing the check.

### 2.4 Save file convention

`var/games/<rom-name>.srm` — a dedicated folder under the project root's Unix-shaped layout (see `EmuSen_Debugging_Tools_Reference_v5.md` §3.3/`man hier` - this was `Saves/` before that layout existed), deliberately *not* derived from the ROM's own directory (which could be anywhere on disk, possibly read-only, and isn't necessarily "ours" to write into — ROMs can load from any path via the CLI arg or the Avalonia frontend's file picker). Only the save *filename* comes from the ROM; the folder is always relative to where the emulator runs from.

`LoadSram()` tolerates a save file that doesn't exactly match the allocated SRAM size (copies whichever is smaller) rather than failing outright — a mismatch most likely means this ROM's header-reported SRAM size differs from whatever created the file, not a corrupted save. A save that fails to load doesn't prevent the game from booting.

`SaveSram()` is called periodically (see `VenusCore.RunFrame`'s autosave) and on shutdown, not on every SRAM write — cheap enough (a few KB, plain overwrite) that this is a convenience choice, not a performance necessity.

---

## 3. `Dma` (`Memory/Dma.cs`) — general DMA + HDMA

8 independent channels, each with its own control byte, source address/bank, transfer size, and (for HDMA) an indirect-addressing mode.

### 3.1 General DMA

Triggered by a `$420B` write (channel bitmask). Transfer direction (`bToA`), address-step direction, and the 4-byte repeating destination-register pattern are all derived from the channel's control byte. `TransferPatterns` encodes the 8 hardware-defined patterns (e.g. pattern 4 = `0,1,2,3`, writing 4 consecutive PPU registers per source byte — used for OAM/CGRAM-adjacent multi-register transfers).

**`PendingCpuCycles` — DMA now charges real CPU time.** `ExecuteGeneralDma` used to transfer every byte "for free": the whole byte loop ran without `Cpu.Step()` ever seeing any of that time elapse, so a large transfer (e.g. a full VRAM graphics upload, common during a game's boot sequence) cost the same handful of cycles as the `STA $420B` instruction that triggered it. Real hardware charges ~8 master cycles of per-channel setup plus ~8 master cycles per byte transferred, so the charge is `1 + bytes` per active channel, accumulated in `Dma.PendingCpuCycles` and drained into `Cpu.Step()`'s own return value on the very next call (covering the `_stopped`/`_waitingForInterrupt` early-return paths too, since a DMA triggered by the instruction immediately before a WAI/STP would otherwise have its cost silently dropped). This matters beyond raw scanline-budget accuracy: this project's CPU→SPC700 pacing (`VenusCore.RunFrame`'s `scaledSpc700Cycles` calculation, §1 of `Venus_APU.md`) is driven entirely by `Cpu.Step()`'s returned cycle count, so any period where DMA cycles went uncounted was a period where the SPC700 was shorted its proportionate share of cycles too. Found chasing (but did not turn out to be the cause of) a Super Metroid boot hang — see `Venus_APU.md`'s own note on that investigation.

**Unit convention, updated alongside `Venus_CPU.md` §8's dynamic cycle-penalty work**: `PendingCpuCycles`'s own units were never rescaled by that change (still `1 + bytes`, an assumed flat "8 master clocks per unit" DMA-specific approximation, not itemized per-channel target region) — but `DrainPendingDmaCycles()` on the `Cpu.cs` side now multiplies by 8 before adding it into `Step()`'s return value, since that return value is real master clocks now (§8.3 of `Venus_CPU.md`) rather than the old abstract "CPU cycle" unit this comment originally assumed 1 DMA-unit already equaled. Net effect on DMA's own real-master-clock cost is unchanged from before that rescaling - only the unit `Step()` hands back to its caller changed.

### 3.2 HDMA

Re-armed once per frame (`InitHdma`) and stepped once per scanline (`ExecuteHdma`) — this is how per-scanline effects (window wipes, palette changes, Mode 7 matrix updates) get their data without CPU intervention every line.

**7-bit line counter**: bit 7 of the line-counter byte is a "repeat" flag, not part of the count — only the lower 7 bits actually count down. Getting this wrong (treating the whole byte as the counter) breaks any HDMA table using the repeat flag, which is most of them.

**Indirect addressing** (`Control` bit 6): instead of reading transfer data directly from the table, each block's 2-byte indirect address is fetched from the table and *that* address is where the actual transfer data lives — lets one table entry cover an arbitrary-sized block instead of being limited by inline table bytes.

### 3.3 Debug logging gates

All console output here is behind `DebugSettings` flags (`DmaVerboseLogging`, `DmaSourceAddrLogging`, `WindowHdmaLogging`) — see `Man pages/Hardware/README.md`'s note on `DebugSettings` vs. the newer `DianaOS/` toolchain (WatchRegistry, FrameLogRegistry) for why these older ad hoc flags still exist alongside the newer core-agnostic tools.

`LogSourceAddrWrite` specifically was added to investigate why Yoshi's sprite-graphics DMA channel always used a constant source address instead of a varying one — a constant, never-changing source is the signature of an uninitialized or wrongly-computed pointer. It reports which PC wrote the source address, including the raw 4 bytes at that PC (enough to cover any 65816 instruction length) for manual identification without needing a full disassembler.

---

## 4. `InterruptController` (`Memory/InterruptController.cs`) — NMI / H-V-IRQ / vblank status

### 4.1 Why this is one class, not split further

An earlier architecture review proposed splitting H/V-IRQ into its own "IrqController," leaving NMI/vblank state on `MemoryBus`. On inspection that split would have been artificial: the vblank flag feeds *both* the `$4210` (RDNMI) read *and* the `$4200`-write NMI-rising-edge check (§4.2), and a single `$4200` write sets NMI enable and H/V-IRQ enable together on real hardware. Splitting them would add cross-object coupling instead of removing any — this class is everything that's genuinely one hardware subsystem, extracted together.

Deliberately does **not** own `LineCycles`/`CurrentScanline` (those stay on `MemoryBus`, mirrored into `Ppu`) even though HVBJOY's H-blank bit needs a value derived from `LineCycles` — that's PPU-timing state, not an interrupt-controller concern. `MemoryBus` computes the H-blank bool itself and passes it into `ReadHVBJOY`.

### 4.2 NMI rising-edge quirk

**NMI fires on the rising edge of (NMITIMEN bit 7 AND the vblank flag), not just once at the start of vblank if already enabled.** Confirmed via three independent sources (fullsnes, SNESdev wiki's MMIO_registers page, and its Errata page separately) all agreeing. If a game disables NMI, does some work, then re-enables it via `$4200` *while the vblank flag is still set* (still during vblank, hasn't read `$4210` yet), NMI fires immediately at that write — not at the next frame's vblank start. This can mean more than one NMI in a single vblank.

Modeled via `PendingImmediateNmi`, set in `Write4200`, consumed once per scanline boundary in the main loop — the same scanline-level granularity already used for H/V-IRQ checks throughout this project. Not cycle-exact, but a real, deliberately-accepted simplification rather than a silent gap.

### 4.3 RDNMI auto-clear at end of vblank

Real hardware clears RDNMI's vblank flag at the *end* of vblank regardless of whether the game ever read `$4210` that frame (fullsnes: "The flag gets reset automatically at end of Vblank, and gets also reset after reading from this register"; SNESdev wiki's MMIO_registers page independently agrees). Previously this project only cleared the flag on a `$4210` read, so a game skipping the read in one frame carried a stale flag into the next frame's active display period. `EndVBlank()` now clears it explicitly, called from the main loop at the same point `InVBlank` flips false.

### 4.4 Register bit layouts

- **`$4210` RDNMI**: `Nxxx VVVV` — bit 7 vblank flag, bits 4-6 open bus, bits 0-3 CPU version (`2`, unchanged since the original S-CPU).
- **`$4212` HVBJOY**: `VHxx xxxJ` — bit 7 vblank, bit 6 hblank, bits 1-5 open bus, bit 0 joypad auto-read in-progress (**not modeled** — auto-joypad read isn't its own timed process in this project; defaults to 0/"not in progress," correct outside the ~3-scanline window right after vblank starts where real hardware briefly reports 1 — an accepted, narrow gap).
- **`$4211` TIMEUP**: `Txxx xxxx` — bit 7 timer/IRQ flag, bits 0-6 open bus.
- **Disabling an IRQ also acknowledges a pending one** (`Write4200`) — documented hardware behavior (fullsnes), something NMI does *not* do.

---

## 5. `MathUnit` (`Memory/MathUnit.cs`) — hardware multiply/divide

`$4202-$4206` write (operands + triggers), `$4214-$4217` read (results) — a genuinely distinct piece of hardware, extracted out of `MemoryBus` for the same reason `InterruptController` was.

**Computed instantly on the triggering write** (WRMPYB/WRDIVB), not modeled with the real 8/16-cycle hardware delay — correct for any game that waits before reading the result, which is how this unit is used in practice; no known game depends on the delay itself being observable.

**Multiply and divide share the same physical result register on real hardware** — `_rdMpy` (read via `$4216/4217`) holds the last multiply's product *or* the last divide's remainder, whichever operation ran most recently. Modeled directly rather than as two separate fields to match that hardware reality.

**Divide by zero**: quotient `$FFFF`, remainder equal to the dividend — documented hardware behavior, not an exception or a zero result.

---

## 6. Debug-toolchain observer hooks (`Memory/IWriteObserver.cs`, `IReadObserver.cs`, `IFrameObserver.cs`, `IRomReadPatcher.cs`)

Four minimal interfaces `MemoryBus` calls through (`WriteObserver`, `ReadObserver`, `FrameObserver`, `RomPatcher` fields) with no idea what's listening on the other end — real emulation state and debug-toolchain plumbing stay separate. `SnesDebugTarget` (see `Man pages/EmuSen_Debugging_Tools_Reference_v5.md` §3.2) implements all four and owns the `WatchRegistry`/`FrameLogRegistry`/`CheatRegistry` instances that actually do something with the calls.

Kept as **separate interfaces** rather than one combined one — a future implementer that only cares about one kind of event can implement just what it needs, and adding a new one (as `IFrameObserver` was, and `IRomReadPatcher` most recently) is additive rather than a breaking change to an existing contract.

`ReadObserver`/`WriteObserver` are called on every matching memory access and have to stay cheap when nothing is registered (a null-conditional call, no allocation) — `OnRead` especially, since a read happens on every instruction fetch and operand read that touches a watched space, not just the writes a game actually makes. `FrameObserver` is only called once per frame, so its implementation can afford real work (see `FrameLogRegistry.RecordFrame`).

Tagged `"WRAM"` for actual RAM accesses and `"IO"` for hardware-register reads/writes (DMA, H/V-IRQ, math unit, input, APU port, PPU registers) — `watch add IO 4016 1` (or `mem IO 4218 4`) targets exactly the `$4016`/`$4218` a CPU trace's Target Addr already shows, same bank-0-offset convention as everything else in this doc. `SnesDebugTarget.GetMemorySpaces()` registers a matching `"IO"` `BusDebugMemorySpace` (bank 0, 64KB - registers mirror identically across every hardware bank, so that's the whole space that matters) purely so `watch add`/`mem`'s validation recognizes the name; the actual watch matching only ever looks at the string tag and address WatchRegistry.Record gets called with, independent of that space's own bounds. The two open-bus passthroughs (`$421C-$421F`, and the generic unmapped-gap fallback) are deliberately excluded - they return whatever last drove the bus, not a real register's content.

**`IRomReadPatcher` is the odd one out - it can change what the CPU sees, not just observe.** Called once, right where `ReadInternal` falls through to `_cartridge.Read8(address)` (ROM or SRAM - never WRAM or a hardware register, since those never route to the cartridge on real hardware either), with the byte the cartridge actually returned. If it returns true, that substituted byte replaces the cartridge's own value for that read - the exact mechanism a real Game Genie device uses (an interposer on the cartridge edge connector overriding specific addresses), as opposed to `RamPoke`-style cheats (see `EmuSen_Debugging_Tools_Reference_v5.md` §3.14), which write real bytes into real RAM every frame instead of intercepting a read. `SnesDebugTarget.TryPatch` forwards to `CheatRegistry.TryPatchRom`, which only ever matches `RomPatch`-kind entries - `RamPoke` entries are invisible to this hook entirely.

This class of field previously lived as concrete debug types directly on `MemoryBus` (a `WatchRegistry` field, a `Cpu` back-reference for PC context) — debug-toolchain plumbing that had no business on the bus itself, added there only because it was the convenient place at the time. The observer-interface pattern replaced that: `MemoryBus` knows nothing about `WatchRegistry`, `SnesDebugTarget`, or even that a debug console exists.

---

## 7. Reset-vector guard, and other correctness regressions caught the hard way

A running list of bugs specifically in this bus/memory layer that shipped once and were caught by symptom rather than by review, kept here so the same mistake isn't repeated:

- **ROM reads intercepted by the open-bus fallback** (§1.2) — black screen, PC stuck at `$000000`.
- **SRAM hard-range-checked instead of mirrored** (§2.3) — Super Metroid's anti-piracy check failed.
- **SRAM size hardcoded to 2KB** (§2.2) — wrong for every game except the one it was written against.
- **WMDATA/low-bank-mirror WRAM writes not reaching `WriteObserver`** (§1.3) — the Yoshi/coin investigation's watch tooling showed "zero writes" to a range that demonstrably held real, varying data; the gap was in the tooling, not the game.
- **Hardware registers ($4016-$421B, DMA/IRQ/math-unit/PPU/APU-port registers) never reached `ReadObserver`/`WriteObserver` at all** (§6) — only WRAM accesses did. Found investigating Super Mario All-Stars' Controller-2 input quirk: `watch add`-ing `$4016`/`$4218` recorded zero events despite a CPU trace proving they were read every single frame. Same class of gap as the WMDATA one above (the tooling, not the game/core logic, which had always computed the right values) - fixed by adding the same observer calls to every register branch in `ReadInternal`/`Write8`, tagged `"IO"`.
