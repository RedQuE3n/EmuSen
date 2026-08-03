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

**FastROM only speeds up banks `$80-$FF`.** Banks `$00-$3F:$8000-$FFFF` are always 8 master clocks no matter what `$420D` says — the bit selects the speed of the *upper* half of the address space, not of every ROM window. `GetAccessSpeedCycles` applied it to both halves until 2026-08-03, which would have run bank-`$00` code 25% fast in any FastROM game. Verified cell by cell against Mesen's `_masterClockTable` (`SnesMemoryManager.cpp`), which builds banks `$00-$3F` page `>= $80` as a flat 8 and only makes `$80-$BF`/`$C0-$FF` register-dependent.

Worth recording that this was a **latent** bug on the game that exposed it: Yoshi's Island never enables FastROM during the window under investigation, so fixing it changed that measurement by exactly zero instructions. It is still wrong, and would matter the moment a FastROM game runs code from a low bank.

Bit 0 selects FastROM (6 master clocks/access) vs. SlowROM (8) for the `$8000-FFFF` window of banks `$00-$3F`/`$80-$BF` and all of `$C0-FF` — everywhere else on the bus is a fixed speed regardless of this bit (banks `$40-$7D`/`$7E-$7F` always slow; the `$2000-3FFF`/`$4200-5FFF` register windows always fast; `$4000-41FF` always a slow 12). Previously entirely unhandled — a write here fell through to the generic open-bus fallback, so this project always behaved as SlowROM no matter what a game actually wrote. Read via `MemoryBus.GetAccessSpeedCycles(address)`, which `Cpu.Step()` now uses for real per-instruction master-clock accounting — see `Venus_CPU.md` §8 for the full story of why this exists and what changed alongside it.

---

## 2. `Cartridge` (`Memory/Cartridge.cs`) — cartridge mapping + save data

### 2.1 ROM loading

Strips a 512-byte copier header if present (detected by `fileBytes.Length % 32768 == 512` — a headerless LoROM divides cleanly by 32KB, a headered one doesn't).

### 2.1a Map modes — `ICartridgeMapper`, and why mapping is not `Cartridge`'s job

`Cartridge` used to *be* the mapper: `Read8`/`Write8` had LoROM's address arithmetic written into them inline, with a `// --- LoROM ROM Mapping ---` comment and no notion that any other map existed. That silently made every HiROM game unbootable, and there was no seam at which to add one.

Mapping now lives behind **`ICartridgeMapper`** (`Memory/Mappers/`), a deliberately tiny interface:

```csharp
CartridgeAddress Resolve(byte bank, ushort offset);   // -> Unmapped | Rom(offset) | Sram(offset)
```

A mapper is **pure address arithmetic** — it holds no ROM or SRAM bytes, does no bounds checking, and does no mirroring. `Cartridge` keeps all of that (`Offset < _rom.Length`, `% _sram.Length` per §2.3). The payoff is that a map mode becomes a testable function of `(bank, offset)` with no ROM, no core and no file involved — `EmuSen.WiseMan/Memory/CartridgeMapperTests.cs` asserts the whole table directly, including the exact reset-vector case below.

**`LoRomMapper`** (mode `$20`) — ROM in the upper half of every bank (`bank & 0x7F` folds the `$80-$BF` mirror onto `$00-$3F`); SRAM at `$70-$7D`/`$F0-$FF` below `$8000`. Behaviour is byte-identical to the old inline code.

**`HiRomMapper`** (mode `$21`) — full 64KB ROM banks at `$C0-$FF`, mirrored at `$40-$7D`; the *upper half of those same banks* visible at `$00-$3F`/`$80-$BF`; SRAM in a `$6000-$7FFF` window at `$20-$3F`/`$A0-$BF`. Both ROM windows collapse to one expression, `((bank & 0x3F) << 16) | offset`, because the `$00-$3F` view is literally the top half of the same-numbered `$C0` bank.

**Detection** scores both header locations (`$7FC0` for LoROM, `$FFC0` for HiROM) rather than trusting either blindly: +2 if the map-mode nibble matches that location, +2 if the header's checksum and complement XOR to `$FFFF`. Highest score wins. A single byte read at a fixed offset is not enough — the "header" of a HiROM image at `$7FC0` is ordinary game data that can look plausible.

**Why this mattered — Donkey Kong Country.** DKC (and DKC2, DKC3) are HiROM, cartridge type `$02` = ROM + RAM + battery: **no coprocessor**, contrary to the reasonable first guess given how the game looks. Rare's trick was pre-rendered SGI artwork compressed into ordinary tiles and sprites, which needs no special silicon — only a large ROM, hence HiROM. The failure was one address:

| | `$00:FFFC` resolves to | bytes there | reset PC |
|---|---|---|---|
| LoROM (what ran) | file offset `$7FFC` | `00 00` | **`$0000`** |
| HiROM (correct) | file offset `$FFFC` | `00 80` | `$8000` |

The CPU reset into zeroed WRAM and executed `BRK` forever, so not one instruction of the game ever ran — a black screen with `INIDISP`/`BGMODE`/`TM` all still `0`. With `HiRomMapper` selected, DKC boots through the Rare logo, "Nintendo Presents", the DK/boombox intro cutscene, and reaches its title screen.

**One bus change was needed.** `MemoryBus.ReadInternal` returned open bus for any `offset < 0x8000` in a hardware bank before the cartridge was ever consulted — correct for LoROM, but it swallows HiROM's `$20-$3F:$6000-$7FFF` SRAM window. That early return is now `if (offset < 0x8000 && !_cartridge.MapsAddress(address))`, so the mapper decides. The write path already fell through to the cartridge and needed no change.

**`Sa1Mapper`** (mode `$23`) — the first coprocessor cart, and the first mapper that is *not* pure address arithmetic: the SA-1's bank registers move ROM and BW-RAM around at runtime, so the arithmetic lives on the chip and the mapper forwards to it. It also needed three new `CartridgeRegion` members (`IRam`, `Sa1Register`, `Sa1Vector`), since a cartridge can now decode to more than ROM and SRAM. See `Venus_SA1.md`. The `MapsAddress` escape hatch added for HiROM below turned out to be exactly what let this land with no `MemoryBus` change at all.

**`SuperFxMapper`** (cartridge type `$1x`) — static arithmetic, unlike `Sa1Mapper`, but it lives on the chip so both sides share one decode. See `Venus_SuperFX.md`; that chip is **implemented but not yet correct end to end**.

**`CoprocessorOverlayMapper`** (cartridge types `$0x`, `$2x`, `$Fx`) — the other shape a coprocessor mapper can take. The NEC DSPs and the OBC1 claim only a small window and leave the rest of the cartridge on its ordinary LoROM/HiROM decode, so rather than each writing out a whole map, they supply just the overlay and this wrapper falls through to `LoRomMapper`/`HiRomMapper` on a miss. See `Venus_NecDSP.md` §3 and `Venus_OBC1.md` §1. The DSP family needs a firmware dump this project does not ship; without one the cartridge stays on the plain map and the chip is simply absent.

**Not yet implemented:** ExHiROM (mode `$25`, the >4MB Star Ocean/Far East of Eden layout) and the remaining coprocessor carts (CX4, S-DD1, SPC7110, ST018, SGB, BS-X). Those are new `ICartridgeMapper` implementations plus, for the coprocessors, a chip to talk to — and for one with its own 65816, the `ICpuBus` seam `Venus_SA1.md` §2.1 describes.

### 2.2 SRAM sizing from the ROM header

The RAM Size byte lives at header offset `+$18`, so the file offset depends on which header §2.1a's detection picked: `$7FD8` for LoROM, `$FFD8` for HiROM. It used to be hardcoded to `$7FD8`, which read arbitrary game data as the SRAM size on any HiROM cart. It encodes SRAM size as `1KB << N`; `N=0` means no SRAM. Verified via the SNESdev wiki's ROM header page and a WLA-DX header example. Clamped to `N<=7` (512KB, SnesLab's documented real-hardware maximum) defensively against a corrupt header.

**This used to be hardcoded to 2KB for every game** — happens to match Super Mario World, wrong for anything else. Super Metroid's RAM Size byte is 3 (8KB), and its boot-time anti-piracy check specifically depends on the *real* chip size (see §2.3) — an undersized array made that check fail.

### 2.3 SRAM mirroring — why it's `% _sram.Length`, not a hard range check

SRAM is mapped to the lower 32KB (`$0000-$7FFF`) of banks `$70-$7D` and `$F0-$FF`. Real SRAM chips only decode as many address lines as their actual size needs, so a chip smaller than the full 32KB window mirrors repeatedly within it — modulo addressing reproduces that for free.

**Super Metroid's boot-time anti-piracy check deliberately writes a test pattern through one mirror address and reads it back through another**, specifically to detect cartridges/emulators that don't mirror correctly (see tcrf.net/Super_Metroid). This previously used a hard range check against the array length instead of wrapping — anything past the (also wrong, hardcoded-2KB) array size silently read as open bus rather than mirroring, failing the check.

### 2.4 Save file convention

`var/games/<rom-name>.srm` — a dedicated folder under the project root's Unix-shaped layout (see `EmuSen_Debugging_Tools_Reference_v5.md` §3.3/`man hier` - this was `Saves/` before that layout existed), deliberately *not* derived from the ROM's own directory (which could be anywhere on disk, possibly read-only, and isn't necessarily "ours" to write into — ROMs can load from any path via the CLI arg or the Avalonia frontend's file picker). Only the save *filename* comes from the ROM; the folder is always relative to where the emulator runs from.

`LoadSram()` tolerates a save file that doesn't exactly match the allocated SRAM size (copies whichever is smaller) rather than failing outright — a mismatch most likely means this ROM's header-reported SRAM size differs from whatever created the file, not a corrupted save. A save that fails to load doesn't prevent the game from booting.

`SaveSram()` is called periodically (see `VenusCore.RunFrame`'s autosave) and on shutdown, not on every SRAM write — cheap enough (a few KB, plain overwrite) that this is a convenience choice, not a performance necessity.

### 2.4a The battery save is emulator state, and it silently broke a three-day measurement

**On a SuperFX cart the `.srm` *is* GSU work RAM.** Game Pak RAM is one chip holding the GSU's variables, its framebuffer and the battery-backed save (`Venus_SuperFX.md` §1), and `LoadSram()` copies `_batteryRamSize` bytes straight into the low end of it. So restoring a save does not only restore the player's progress — it pre-loads whatever the GSU last left in those addresses.

That interacts badly with state anchoring (`EmuSen_Debugging_Tools_Reference_v5.md` §3.15d). `tapuntil A GSURAM 1E1A F0,01` is supposed to walk Yoshi's Island's file select and its whole opening cutscene. With a `.srm` written by an earlier run of the same script, `$1E1A` **already holds `$01F0` at power-on**, so the verb reports success at frame 0, having pressed nothing, and every command after it measures a machine that is still in its boot sequence. That is exactly what happened to §10.7's "the camera velocity is zero" measurement, and it is why the finding could not be reproduced from a clean boot.

Two consequences worth carrying:

- **A run that persists state is not a reproducible run.** Two invocations of one `--commands` script reached the anchor at frame 789 and at frame 0 purely because the first one wrote a save. Mesen's probe never had the problem — it points `FolderUtilities::SetHomeFolder` at a fresh directory beside its dumps — which means the two emulators were anchored on different scenes for the whole comparison.
- **`--nobattery` exists for this.** It sets `Cartridge.BatteryRamDisabled`, so the run neither reads nor writes the `.srm`. Every comparison against the Mesen probe should use it; see §3.15's own entry.

### 2.5 Region detection from the country byte

The header's country byte (`+$19`, so `$FFD9` on HiROM / `$7FD9` on LoROM) says which territory the cartridge was sold in, and therefore which console it expects. `Cartridge` classifies it once at load into a `ConsoleRegion` (`ConsoleRegion.cs`), which `VenusCore.LoadRom` turns into both the frame timing and the `STAT78` region bit — see `Venus_CPU.md` §8.5c.

| Country byte | Territory | Region |
|---|---|---|
| `$00` | Japan | NTSC |
| `$01` | USA | NTSC |
| `$02`-`$0C` | Europe, Scandinavia, France, Netherlands, Spain, Germany, Italy, China, Indonesia | **PAL** |
| `$0D` | South Korea | NTSC |
| `$0F` | Canada | NTSC |
| `$10` | Brazil | NTSC |
| `$11` | Australia | **PAL** |
| anything else | — | NTSC |

Two entries are worth stating explicitly because they look wrong at a glance. **Brazil (`$10`) is NTSC here** despite being a PAL territory: Brazilian SNES units run PAL-M, which is 60 Hz with 262 lines — PAL only in its colour encoding, which an emulator producing RGB never reproduces anyway. Only the *timing* matters at this layer, and PAL-M's timing is NTSC's. **Australia (`$11`) is PAL**, which some emulators get wrong; snes9x's own check is `region >= 2 && region <= 12`, which stops one short of it.

**The fallback is NTSC, deliberately.** An unrecognized byte (a homebrew ROM that never filled the field in, a bad dump, or the `$EA` filler that `SyntheticRom` leaves there) gets NTSC rather than a guess, because NTSC is what this emulator did unconditionally before regions existed — an unknown ROM behaves exactly as it always has, and only a positively-identified PAL cartridge changes behavior.

---

## 3. `Dma` (`Memory/Dma.cs`) — general DMA + HDMA

8 independent channels, each with its own control byte, source address/bank, transfer size, and (for HDMA) an indirect-addressing mode.

### 3.1 General DMA

Triggered by a `$420B` write (channel bitmask). Transfer direction (`bToA`), address-step direction, and the 4-byte repeating destination-register pattern are all derived from the channel's control byte. `TransferPatterns` encodes the 8 hardware-defined patterns (e.g. pattern 4 = `0,1,2,3`, writing 4 consecutive PPU registers per source byte — used for OAM/CGRAM-adjacent multi-register transfers).

**`PendingCpuCycles` — DMA now charges real CPU time.** `ExecuteGeneralDma` used to transfer every byte "for free": the whole byte loop ran without `Cpu.Step()` ever seeing any of that time elapse, so a large transfer (e.g. a full VRAM graphics upload, common during a game's boot sequence) cost the same handful of cycles as the `STA $420B` instruction that triggered it. Real hardware charges ~8 master cycles of per-channel setup plus ~8 master cycles per byte transferred, so the charge is `1 + bytes` per active channel, accumulated in `Dma.PendingCpuCycles` and drained into `Cpu.Step()`'s own return value on the very next call (covering the `_stopped`/`_waitingForInterrupt` early-return paths too, since a DMA triggered by the instruction immediately before a WAI/STP would otherwise have its cost silently dropped). This matters beyond raw scanline-budget accuracy: this project's CPU→SPC700 pacing (`VenusCore.RunFrame`'s `scaledSpc700Cycles` calculation, §1 of `Venus_APU.md`) is driven entirely by `Cpu.Step()`'s returned cycle count, so any period where DMA cycles went uncounted was a period where the SPC700 was shorted its proportionate share of cycles too. Found chasing (but did not turn out to be the cause of) a Super Metroid boot hang — see `Venus_APU.md`'s own note on that investigation.

**Unit convention, updated alongside `Venus_CPU.md` §8's dynamic cycle-penalty work**: `PendingCpuCycles`'s own units were never rescaled by that change (still `1 + bytes`, an assumed flat "8 master clocks per unit" DMA-specific approximation, not itemized per-channel target region) — but `DrainPendingDmaCycles()` on the `Cpu.cs` side now multiplies by 8 before adding it into `Step()`'s return value, since that return value is real master clocks now (§8.3 of `Venus_CPU.md`) rather than the old abstract "CPU cycle" unit this comment originally assumed 1 DMA-unit already equaled. Net effect on DMA's own real-master-clock cost is unchanged from before that rescaling - only the unit `Step()` hands back to its caller changed.

### 3.1a A-bus arbitration (`CopyDmaByte`) — bank byte matters, not just the offset

Real hardware blocks a DMA channel's A-bus address (whichever side, source or destination, it's playing this transfer) from ever reaching a `$21xx` PPU/APU register or the DMA controller's own registers (`$420B`, `$420C`, `$4300-$437F`) — the read side gets open bus, the write side is simply dropped. `CopyDmaByte` ported this (from Mesen2's `SnesDmaController::CopyDmaByte`) alongside the WRAM/`$2180` bus-conflict case, but the initial port checked only the low 16 bits of the address (`aBusOffset`) against those ranges — **not the bank byte**. `$2100-$21FF` etc. only mean "hardware register" in banks `$00-$3F`/`$80-$BF` (the same `isHardwareBank` convention §1.1 already uses for CPU accesses); the identical offsets in WRAM banks `$7E`/`$7F` are ordinary RAM with no register mirroring at all.

Missing that bank check meant a perfectly normal WRAM source address like `$7E21C0` (bank `$7E`, offset `$21C0` — which happens to fall in `$2100-$21FF`) got wrongly treated as a blocked register access on every transfer, silently replacing real WRAM data with stale open-bus. Found via Super Mario World's spin-jump: SMW shares one OBJ tile slot between Mario's regular pose (staged at `$7E2080`, an address that never collides) and other poses staged at addresses like `$7E21C0` that do — both are DMA'd into the same VRAM tile slot, and on frames where the `$7E21C0`-sourced transfer happened to run last, the whole tile came out as a solid open-bus fill, which decoded through Mario's palette as two bright yellow columns through his torso, intermittently, only on Big Mario (whose spin animation touches two stacked OBJ tiles instead of Small Mario's one, doubling the chance of landing on a colliding source address). Real hardware never sees this because the bank check is implicit in the actual address-decode hardware. Fixed by gating `aBusBlocked` on `aBusIsHardwareBank` first, same as every other bank-sensitive check in this file.

**Worth re-checking**: the still-open "Yoshi/coins don't render" note in `EmuSen_Games_Tested.md` also centers on a WRAM staging buffer (`$7E8000-$7E97FF`) feeding a VRAM DMA — the same general shape as this bug. Not confirmed to be the same root cause, but worth trying again now that this bank check is fixed.

### 3.2 HDMA

Re-armed once per frame (`InitHdma`) and stepped once per scanline (`ExecuteHdma`) — this is how per-scanline effects (window wipes, palette changes, Mode 7 matrix updates) get their data without CPU intervention every line.

**7-bit line counter**: bit 7 of the line-counter byte is a "repeat" flag, not part of the count — only the lower 7 bits actually count down. Getting this wrong (treating the whole byte as the counter) breaks any HDMA table using the repeat flag, which is most of them.

**Indirect addressing** (`Control` bit 6): instead of reading transfer data directly from the table, each block's 2-byte indirect address is fetched from the table and *that* address is where the actual transfer data lives — lets one table entry cover an arbitrary-sized block instead of being limited by inline table bytes.

### 3.2a Enabling HDMA mid-frame ($420C 0->1)

`InitHdma` runs at scanline 0 and arms whichever channels `$420C` names *at that moment*. A game that only ever writes `$420C` in vblank is therefore fine, and that is nearly every game. **Yoshi's Island is not one of them**, and the shape of what it does is worth recording because it defeated a long investigation that never suspected the DMA engine at all.

Its intro is driven by three H/V-IRQs per frame, programmed through `$4209`/`$420A` and dispatched from a `$7E:0125` phase counter at `$7E:C821`:

| Scanline | Phase | What it does |
|---|---|---|
| 12 | 0 | `LDA $094A` / `STA $420C` — **HDMA on** (`$F0`, channels 4-7) — then `INIDISP = $00`, next V-target `$0E` |
| 14 | 1 | `INIDISP = $0F` (screen to full brightness), next V-target `$C6` |
| 198 | 2 | `INIDISP = $8F` (force blank), `STZ $420C` — **HDMA off** — next V-target `$0C` |

So `$420C` is `$F0` only between scanlines 12 and 198, and is `$00` across every scanline 0. Arming solely at scanline 0 meant **`dma stats` reported zero HDMA transfers over 700 frames** while the game requested it 415 times — the channels were configured, the tables were in WRAM, and nothing ever ran.

The fix is `WriteHdmaEnable`: a bit going 0->1 arms that channel there and then, exactly as `InitHdma` would have, and leaves already-armed channels alone. Frame-start init is unchanged.

**This is modeled behavior, not a silicon measurement, and the honest version of the claim matters.** bsnes and Mesen both arm only at scanline 0 (`SnesDmaController::InitHdmaChannels` early-returns when `HdmaChannels` is zero, having first cleared every channel's `DoTransfer`), so a mid-frame enable there resumes from whatever table pointer the channel was left holding. For this game that state is the previous frame's terminator, which cannot produce a correct picture — yet the game demonstrably works on hardware and in Mesen. Re-arming from the top of the table is the only reading consistent with the game's own design, and it is what this core does. If a future test ever pins the real 0->1 semantics, this is the paragraph to correct.

**What it is worth, measured.** Yoshi's Island's intro renders its story text (`Venus_SuperFX.md` §10.5). Sixteen other games — SMW, LttP, Super Metroid, Chrono Trigger, DKC2, FFVI, Super Mario Kart, MMX, EarthBound, Secret of Mana, Super Castlevania IV, Illusion of Gaia, All-Stars+World, Mario's Time Machine, Rock 'n Roll Racing, TMNT IV — produce **byte-identical `framesum` digests** over 200 frames before and after, and all 1680 `EmuSen.WiseMan` tests pass. Games that enable HDMA in vblank see the newly-enabled channel armed once at the write and again at scanline 0, which lands on the same state.

### 3.3 Debug logging gates

All console output here is behind `DebugSettings` flags (`DmaVerboseLogging`, `DmaSourceAddrLogging`, `WindowHdmaLogging`) — see `Man pages/Hardware/README.md`'s note on `DebugSettings` vs. the newer `DianaOS/` toolchain (WatchRegistry, FrameLogRegistry) for why these older ad hoc flags still exist alongside the newer core-agnostic tools.

`LogSourceAddrWrite` specifically was added to investigate why Yoshi's sprite-graphics DMA channel always used a constant source address instead of a varying one — a constant, never-changing source is the signature of an uninitialized or wrongly-computed pointer. It reports which PC wrote the source address, including the raw 4 bytes at that PC (enough to cover any 65816 instruction length) for manual identification without needing a full disassembler.

---

## 4. `InterruptController` (`Memory/InterruptController.cs`) — NMI / H-V-IRQ / vblank status

### 4.1 Why this is one class, not split further

An earlier architecture review proposed splitting H/V-IRQ into its own "IrqController," leaving NMI/vblank state on `MemoryBus`. On inspection that split would have been artificial: the vblank flag feeds *both* the `$4210` (RDNMI) read *and* the `$4200`-write NMI-rising-edge check (§4.2), and a single `$4200` write sets NMI enable and H/V-IRQ enable together on real hardware. Splitting them would add cross-object coupling instead of removing any — this class is everything that's genuinely one hardware subsystem, extracted together.

Deliberately does **not** own `LineCycles`/`CurrentScanline` (those stay on `MemoryBus`, mirrored into `Ppu`) even though HVBJOY's H-blank bit needs a value derived from `LineCycles` — that's PPU-timing state, not an interrupt-controller concern. `MemoryBus` computes the H-blank bool itself and passes it into `ReadHVBJOY`, and does the same for the auto-joypad window (§4.4a) — `InAutoJoypadWindow` is a `static` function of the position it is handed, so owning the constants doesn't mean owning the state.

### 4.2 NMI rising-edge quirk

**NMI fires on the rising edge of (NMITIMEN bit 7 AND the vblank flag), not just once at the start of vblank if already enabled.** Confirmed via three independent sources (fullsnes, SNESdev wiki's MMIO_registers page, and its Errata page separately) all agreeing. If a game disables NMI, does some work, then re-enables it via `$4200` *while the vblank flag is still set* (still during vblank, hasn't read `$4210` yet), NMI fires immediately at that write — not at the next frame's vblank start. This can mean more than one NMI in a single vblank.

Modeled via `PendingImmediateNmi`, set in `Write4200`, consumed once per scanline boundary in the main loop — the same scanline-level granularity already used for H/V-IRQ checks throughout this project. Not cycle-exact, but a real, deliberately-accepted simplification rather than a silent gap.

### 4.3 RDNMI auto-clear at end of vblank

Real hardware clears RDNMI's vblank flag at the *end* of vblank regardless of whether the game ever read `$4210` that frame (fullsnes: "The flag gets reset automatically at end of Vblank, and gets also reset after reading from this register"; SNESdev wiki's MMIO_registers page independently agrees). Previously this project only cleared the flag on a `$4210` read, so a game skipping the read in one frame carried a stale flag into the next frame's active display period. `EndVBlank()` now clears it explicitly, called from the main loop at the same point `InVBlank` flips false.

### 4.4 Register bit layouts

- **`$4210` RDNMI**: `Nxxx VVVV` — bit 7 vblank flag, bits 4-6 open bus, bits 0-3 CPU version (`2`, unchanged since the original S-CPU).
- **`$4212` HVBJOY**: `VHxx xxxJ` — bit 7 vblank, bit 6 hblank, bits 1-5 open bus, bit 0 joypad auto-read in-progress (modeled — see §4.4a).
- **`$4211` TIMEUP**: `Txxx xxxx` — bit 7 timer/IRQ flag, bits 0-6 open bus.
- **Disabling an IRQ also acknowledges a pending one** (`Write4200`) — documented hardware behavior (fullsnes), something NMI does *not* do.

### 4.4a The auto-joypad read (`$4200` bit 0, `$4212` bit 0)

Real hardware performs one automatic controller read per frame, starting shortly after vblank begins and shifting all 16 bits out over roughly three scanlines. `$4212` bit 0 reads 1 for the duration. The standard idiom in a game's NMI handler is to spin until that bit clears and only then read `$4218`-`$421B`, so that it gets *this* frame's input rather than a half-shifted value. Illusion of Gaia's handler does exactly this at `$80:8355`:

```
LDA $4212
ROR A
BCS -3      ; spin while bit 0 is set
LDA $4218
```

**This used to be unmodeled** — bit 0 was masked off (`lastBusValue & 0x3E`) and always read back 0, so every such wait loop exited on its first iteration instead of spinning for ~4200 master clocks. The value read was still correct (the latch happened at the top of vblank), so nothing visibly broke; what it distorted was *timing* — an NMI handler finished measurably earlier than on hardware, changing how much of vblank was left for everything after it.

Now modeled as three pieces:

- **`InterruptController.AutoJoypadEnabled`** — `$4200` bit 0. When clear, the read never runs: bit 0 never sets, and `$4218`-`$421B` never update. This is real behavior, not a shortcut, and it means a ROM that never writes `$4200` sees no controller input at all (`SyntheticRom`'s boot stub writes `$4200 = $01` for exactly this reason).
- **`InterruptController.InAutoJoypadWindow(scanline, lineCycles)`** — a pure function of PPU position, so `MemoryBus` can ask for it per read with no state machine. The window runs from master clock 258 to 4482 measured from the start of scanline 225, i.e. it closes 390 clocks into scanline 228. `MemoryBus` computes it and passes it into `ReadHVBJOY`, the same split already used for the H-blank bit (§4's `LineCycles` note).
- **The latch moved off the top of vblank.** `VenusCore` now calls `LatchAutoJoypad()` at the end of scanline 227 — master clock 4092, the last per-scanline tick still inside the window — rather than at scanline 225. A game that waits for bit 0 correctly (reading at ≥4482) therefore still sees fresh data, while one that reads early sees the previous frame's, as on hardware.

- **The read leaves the `$4016`/`$4017` shift registers spent.** Hardware's auto-read isn't a separate data path — it drives the strobe and 16 clock pulses on the *same* controller port lines the manual `$4016`/`$4017` serial protocol uses. By the time it finishes, the pads have shifted all 16 button bits out, so a manual read taken afterwards without a fresh strobe pulse gets the past-the-end state: bit 0 reads back **1**, the same "no more buttons" convention `ReadJoy1Serial` already shifts in. `LatchAutoJoypad()` therefore sets `_shiftJoy1`/`_shiftJoy2` to `$FFFF` alongside latching `$4218`-`$421B`. It is gated on `AutoJoypadEnabled` for the same reason the latch itself is: a game doing its own manual polling turns `$4200` bit 0 off precisely so hardware stops trampling its read, and this models that trampling rather than papering over it.

Frontends set button state once per frame before `RunFrame()`, so moving the latch within the frame cannot change the *value* any well-behaved game observes — only the timing of when it becomes visible.

**Why the spent-shift-register piece matters — Super Mario All-Stars' player/port assignment.** Before it was modeled, `_shiftJoy1`/`_shiftJoy2` sat at `$0000` from power-on until something wrote the strobe, so a bare `LDA $4016` returned **0** where hardware returns 1. All-Stars runs a one-shot port-assignment routine at `$00:86F9` the moment a sub-game launches:

```
LDA $4016 : AND #$01 : EOR #$01 : ASL A : STA $701FF4   ; player 1's $4218 index
LDA $4017 : AND #$01 :            ASL A : STA $701FF6   ; player 2's $4218 index
```

Both reads are of the idle, already-clocked-out port, so on hardware both return 1 and the routine stores 0 for player 1 (reads `$4218`/`$4219`) and 2 for player 2 (reads `$421A`/`$421B`). Returning 0 inverted both: player 1 was assigned port **2** — an empty port — and player 2 got the real controller. The shell's own menus poll both players' button words and so still responded to Start, which made the failure look like it began at gameplay: Super Mario Bros. 1 would boot to World 1-1 with Mario permanently inert, since his input came from a controller that was never plugged in. The two indices live in SRAM (`$70:1FF4`/`$70:1FF6`) and persist into the `.srm`, but the routine rewrites them on every sub-game launch, so a save written while the bug was live corrects itself on the next launch.

**Still simplified:** the individual 16 shift steps aren't modeled (only the window's open/close edges and the spent shift registers it leaves behind), and the window is anchored to fixed scanline/clock constants rather than being rescheduled per frame from a live master clock.

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

### 6.1 The PPU's own memories are observed by `Ppu`, not `MemoryBus`

`MemoryBus` can only ever tag a write `"WRAM"` or `"IO"`. It never sees a VRAM, CGRAM or OAM address at all — those don't exist until a data port (`$2118`/`$2119`, `$2122`, `$2104`) decodes the address register behind it, which happens inside `Ppu`. So `Ppu` carries its own `WriteObserver` field, set by `SnesDebugTarget` alongside the bus ones, and reports `"VRAM"`, `"CGRAM"` and `"OAM"` from the data-port write paths.

Hooking it at the port rather than at the bus means **a DMA-driven write is observed identically to a CPU store** — both arrive through the same port, so a tile upload shows up whether the game used `STA $2118` in a loop or a general-purpose DMA. VRAM addresses are reported *after* `VMAIN` address translation (§2.2 of `Venus_PPU.md`), i.e. the address the byte actually landed at, which is the one worth watching.

**Fixed bug: `watch add VRAM ...` was accepted and then silently never matched.** `WatchCommand` validated the space name against `GetMemorySpaces()` — which does list `VRAM`/`CGRAM`/`OAM`, since `mem`/`dump`/`snapshot` read those arrays directly — so the command reported success. But nothing in the emulator ever called an observer with those tags, so the watch sat there recording nothing, indistinguishable from "this address is never written."

Worth knowing how that misleads in practice, because it cost real time here: a watch on a never-observed space returns no events, and if you then query the wrong watch ID you get somebody else's events and can conclude the space filtering itself is broken. It isn't — `WatchRegistry.Record` compares the tag correctly. **`watch list` before `watch summary` is the habit that catches this**, since a fresh DianaOS session already has watches registered and your new one will not be `#1`.

Found chasing Super Metroid's scattered-coloured-pixel artifact, where the question "what actually writes these tile bytes" was unanswerable until VRAM became watchable. With the hook in place it answered immediately: the Common Room Elements tileset arrives by general-purpose DMA on channel 1, triggered by `STA $420B` at `$80:965E`, with source parameters staged in WRAM at `$05C0`.

---

## 7. Reset-vector guard, and other correctness regressions caught the hard way

A running list of bugs specifically in this bus/memory layer that shipped once and were caught by symptom rather than by review, kept here so the same mistake isn't repeated:

- **ROM reads intercepted by the open-bus fallback** (§1.2) — black screen, PC stuck at `$000000`.
- **SRAM hard-range-checked instead of mirrored** (§2.3) — Super Metroid's anti-piracy check failed.
- **SRAM size hardcoded to 2KB** (§2.2) — wrong for every game except the one it was written against.
- **WMDATA/low-bank-mirror WRAM writes not reaching `WriteObserver`** (§1.3) — the Yoshi/coin investigation's watch tooling showed "zero writes" to a range that demonstrably held real, varying data; the gap was in the tooling, not the game.
- **Hardware registers ($4016-$421B, DMA/IRQ/math-unit/PPU/APU-port registers) never reached `ReadObserver`/`WriteObserver` at all** (§6) — only WRAM accesses did. Found investigating Super Mario All-Stars' Controller-2 input quirk: `watch add`-ing `$4016`/`$4218` recorded zero events despite a CPU trace proving they were read every single frame. Same class of gap as the WMDATA one above (the tooling, not the game/core logic, which had always computed the right values) - fixed by adding the same observer calls to every register branch in `ReadInternal`/`Write8`, tagged `"IO"`.
- **`CopyDmaByte`'s A-bus arbitration checked the offset without the bank byte** (§3.1a) — a normal WRAM source address whose low 16 bits happened to land in `$2100-$21FF` (e.g. `$7E21C0`) was wrongly blocked and replaced with open-bus garbage. Found via Super Mario World's spin-jump: two bright yellow vertical bars intermittently through Big Mario's torso.
