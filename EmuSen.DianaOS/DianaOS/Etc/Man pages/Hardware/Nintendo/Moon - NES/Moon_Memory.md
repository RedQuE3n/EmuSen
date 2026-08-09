# Moon (NES) — Memory, cartridge and mappers

Covers `Cores/Nintendo/Moon - NES/Memory/`: the CPU address decode (`MemoryBus.cs`), the iNES image (`Cartridge.cs`), the board contract (`IMapper.cs`) and the five boards under `Mappers/`. Input lives in `Input/Controller.cs` and is documented in §5.2 because it is reached through this decode.

---

## 1. The CPU address decode

| Range | Destination |
|---|---|
| `$0000-$1FFF` | 2 KB work RAM, mirrored four times |
| `$2000-$3FFF` | the eight PPU registers, mirrored every 8 bytes |
| `$4000-$4013` | APU registers |
| `$4014` | OAM DMA (§5.1) |
| `$4015` | APU status |
| `$4016` | controller 1, and the strobe for both |
| `$4017` | controller 2 on read, APU frame counter on write |
| `$4018-$401F` | disabled test registers; open bus |
| `$4020-$FFFF` | the cartridge |

Only 2 KB of RAM exists. `$0800`, `$1000` and `$1800` are the same storage, which is why the tests assert a write at `$0005` is readable at `$1805` — a game relying on that mirror is not doing anything unusual.

`$4017` genuinely does two different things depending on direction. That is not a decode bug.

---

## 2. The iNES image

A 16-byte header, an optional 512-byte trainer, then PRG and CHR.

`PrgRom` is sized from header byte 4 in 16 KB units and **must** be present in full; a file shorter than its header claims throws rather than being zero-padded, because the missing bytes would include the reset vector and the failure would surface later as an unexplained crash.

CHR is treated more leniently: a short CHR region is zero-padded rather than rejected, since trimmed dumps exist in the wild and a missing tile is a visible, diagnosable problem rather than a silent one. `chrBanks == 0` means the board has 8 KB of CHR **RAM** instead, which the game writes itself.

PRG RAM is always allocated as 8 KB regardless of what byte 8 says, because that field is unreliable on iNES 1.0 and no board here needs more.

### 2.1 NES 2.0

Detected by byte 7 bits 2-3 reading exactly `0b10` — that specific pattern, not merely "nonzero". When present, the mapper number gains four more bits from byte 8 and the PRG/CHR sizes gain upper bits from byte 9. Submappers, exponent-form sizes and the extended RAM fields are not read.

### 2.2 The archaic header, and why byte 7 is often not a header byte

Byte 7's high nibble carries the mapper number's upper four bits — but only if byte 7 is a header byte at all, and on a large fraction of circulating dumps it is not. Copiers of the early 1990s wrote their own signature into bytes 7-15, the commonest being the ASCII string `DiskDude!`. Its leading `D` is `$44`, so a naive read hands the mapper an unearned `$40`.

Two tells distinguish an archaic header, and this loader requires **neither** individually — either one is enough:

- **Byte 7 bits 2-3 are neither `0b10` (NES 2.0) nor `0b00`.** This is the rule the reference implementation uses (`NesHeader::GetRomHeaderVersion`), and it catches `DiskDude!` intact, since `$44 & $0C == $04`.
- **Bytes 12-15 are not all zero.** This catches the partially-scrubbed case, where a tool has cleared byte 7's low bits to `$40` but left `ude!` in bytes 12-15. The reference does not test this; the iNES specification does, and on this library it is the difference between a correct and an incorrect mapper for a further thirty images.

When a header is archaic the mapper is byte 6's high nibble alone. Measured over a 3,536-image set, 184 images change interpretation, and the correction alone moves board coverage from 84.2% to 88.5% — **more than the six boards added alongside it were worth**, which is worth stating plainly because the boards were the intended work and the header was not.

The failure mode this replaced is the reason it was found. Before mappers 64-69 existed, a `DiskDude!` image asked for mapper 65, found nothing, and threw a `NotSupportedException` naming the number: loud, immediate, and obviously a loader problem. The moment those boards existed, the same image would have loaded silently into the wrong board and rendered a blank screen — a *worse* outcome produced by adding correct code. An unimplemented mapper is a diagnosable state; a wrongly-implemented one is not, and a feature that converts the first into the second is a regression however much it adds.

The residue is honest. Two images in this set (`Adventures in the Magic Kingdom (U) [a1]`, `Dr Blario (Dr Mario Hack)`) carry byte 6 and byte 7 nibbles that are simply transposed, with clean bytes 12-15; no structural rule recovers them. The reference plays them correctly because it looks the ROM up by CRC in a game database and overrides the header outright (`GameDatabase::SetGameInfo`). That is a different mechanism, not a better version of this one, and it is the reason those two images cannot be used to test a board here — see §4.11.

---

## 3. Mirroring

The PPU has 2 KB of nametable RAM for four 1 KB nametables, so two of the four are always aliases. Which two is a property of the *board*, not the PPU — `Mirroring` comes from `IMapper`, not from the header, because boards like AxROM change it at runtime.

- **Horizontal** — `$2000`/`$2400` are one page, `$2800`/`$2C00` the other. Vertical scrolling.
- **Vertical** — `$2000`/`$2800` are one page. Horizontal scrolling.
- **Single-screen lower/upper** — all four map to one page. AxROM picks which.
- **Four-screen** — the board brought its own extra 2 KB; all four are distinct.

---

## 4. Boards

| iNES | Name | PRG | CHR | Notes |
|---|---|---|---|---|
| 0 | NROM | fixed 16K or 32K | fixed | a 16 KB board mirrors its one bank into `$C000` |
| 1 | MMC1 | switchable, 3 modes | 4K or 8K banks | serial register, §4.4 |
| 2 | UxROM | `$8000` switches, `$C000` fixed to last | CHR RAM | |
| 3 | CNROM | fixed | whole 8K switches | bus conflicts not modelled |
| 4 | MMC3 | four 8K windows | eight 1K windows | scanline IRQ, §4.6 |
| 7 | AxROM | 32K switchable | CHR RAM | picks its own single-screen page |
| 9 | MMC2 | `$8000` switches, three fixed | two 4K windows | PPU-driven CHR latch, §4.10 |
| 11 | Color Dreams | 32K switchable | 8K switchable | GxROM's fields swapped, §4.7 |
| 64 | RAMBO-1 | three 8K windows | eight 1K windows | scanline **or** cycle IRQ, §4.15 |
| 65 | Irem H3001 | three 8K windows | eight 1K windows | 16-bit cycle IRQ, §4.11 |
| 66 | GxROM | 32K switchable | 8K switchable | one register does both, §4.7 |
| 67 | Sunsoft-3 | `$8000` switches, `$C000` fixed | four 2K windows | 16-bit cycle IRQ, §4.12 |
| 68 | Sunsoft-4 | `$8000` switches, `$C000` fixed | four 2K windows | CHR ROM as nametables, §4.13 |
| 69 | Sunsoft FME-7 | three 8K windows plus `$6000` | eight 1K windows | command/parameter pair, §4.14 |
| 71 | Camerica | `$C000` switches, `$8000` fixed | CHR RAM | §4.7 |
| 79 | NINA-003-006 | 32K switchable | 8K switchable | register at `$4100`, §4.7 |

Measured against a 3,536-image set, these sixteen boards cover **89.2%** of it. The progression is worth recording because it is not what was expected: the ten boards before this pass covered 84.2%, correcting the archaic-header rule (§2.2) moved that to 88.5% without adding a single board, and the six boards then added 0.7%. The headline number moved four times as far from a five-line change in the loader as from six new mappers, and the six mappers were the intended work.

The remaining gaps are mappers 6 (77 images) and 17 (29), which are not boards but FFE copier formats and sit almost entirely under `Hacks` and `Pirate`; 99 (33), the VS System, which needs a second console and a DIP-switch bank rather than a mapper; and then a long tail of one- and two-image multicarts. Nothing above 2.2% each.

An unimplemented mapper number throws a `NotSupportedException` naming the number rather than loading as NROM and behaving inexplicably. §2.2 argues that this loudness is load-bearing and not merely tidy.

### 4.6 MMC3, and the scanline counter

Eight bank registers behind a select/data pair at `$8000`/`$8001`. `$8000` carries the register index **and** both mode bits, so a bank-select write also sets the PRG and CHR layouts — a caller that writes only the index silently resets both modes.

- **PRG**, four 8K windows. Mode 0: `R6`, `R7`, second-to-last, last. Mode 1: second-to-last, `R7`, `R6`, last.
- **CHR**, eight 1K windows. `R0`/`R1` address 2K pairs and ignore bit 0 of the written value; `R2`-`R5` are single pages. `$8000` bit 7 swaps the two halves.
- **Work RAM** at `$6000`: `$A001` bit 7 enables, bit 6 write-protects. A disabled window reads 0.
- **Mirroring** from `$A000` bit 0, unless the header says four-screen.

The IRQ counter reloads when it is zero or a reload was requested, otherwise decrements; it fires when it reaches zero with IRQs enabled. `$C000` latches the reload value, `$C001` requests a reload, `$E000` disables *and* acknowledges, `$E001` enables. The line stays asserted until `$E000`, which is why `MoonCore` treats it as level-triggered.

### 4.6a A12 is watched per fetch, not counted per line

The counter is clocked by the PPU's **A12 line rising**, and `Mmc3.OnPpuAddress` now sees every address the PPU puts on its bus — one call per fetch, from `Ppu.SetBusAddress`. A rise counts only if A12 was low for at least three PPU clocks first, which is what stops the rapid low-high-low of a normal fetch pattern from clocking the board several times per tile. The model is Mesen's `MMC3::IsA12RisingEdge`.

**Writes to `$2006` clock it too**, and that is the half that is easy to miss. A CPU write to `PPUADDR` drives the PPU's address bus directly, so a game can toggle A12 and step the counter with no rendering happening at all. `mmc3_test`'s `1-clocking` and `3-A12_clocking` test exactly this and fail without it — "Should decrement when A12 is toggled via PPUADDR". `$2007` reads and writes re-drive the bus the same way when they advance the pointer.

The sentinel matters: "A12 not currently low" is `-1`, not `0`, because dot 0 is a real clock the first fetch of a run can land on. Using `0` made the very first rise of the machine's life invisible.

**Previously this was one clock per rendered line**, from `Ppu.EndScanline`. That was enough for Super Mario Bros. 3 — including the pre-render line, whose omission once put its status-bar split one line off and corrupted a band of the title screen — but it is not enough for the suite: `mmc3_test` was 0/6 under it and is 3/6 with real edge detection. What remains failing (`2-details`, `4-scanline_timing`, `6-MMC6`) is finer still.

### 4.6b The filter is measured in CPU cycles, and the empty sprite slots still clock it

Two bugs, found together on 2026-08-07 by diffing SMB3's title screen against Mesen and Nestopia through the reference probe (`EmuSen_Debugging_Tools_Reference_v5.md` §3.45). Both are about *how often* the counter clocks, and between them they moved the frame from 5043 differing pixels to 1536.

**The filter's units.** §4.6a says a rise counts only after A12 has been low "at least three PPU clocks". That is the wrong unit. Mesen's `IsA12RisingEdge` compares against `_console->GetMasterClock()`, and `NesConsole::GetMasterClock()` returns `_cpu->GetCycleCount()` — **CPU cycles**. Three CPU cycles is nine PPU dots, and this counter is in dots, so the threshold was three times too permissive.

What that costs is specific. During the sprite-fetch phase (dots 257-320) each of the eight slots reads two garbage nametable bytes (A12 low) and then two pattern bytes (A12 high). The low stretch between one slot's pattern fetch and the next slot's is about four dots — under the real filter, over a three-dot one. So the board saw **eight rises per scanline instead of one**, and SMB3's title split fired six scanlines early: its floor was drawn from row 190 where hardware puts it at 196.

**The empty slots are not idle.** Fixing the units alone made the floor vanish entirely, which is the more interesting half. `SpritePatternBaseForLine` returned `$0000` whenever no sprite on the line had bit 0 of its tile index set — including when the line had *no sprites at all*. On the SMB3 title screen most lines are empty, so A12 never rose on them, the counter barely moved, and the IRQ never arrived.

Hardware does not idle those slots. It fetches tile `$FF` through every unused one, and for 8x16 sprites the table is chosen by the tile index's low bit, so `$FF` always selects `$1000`. Mesen's `LoadSpriteTileInfo` does the same and labels it: *"Fetches to sprite 0xFF for remaining sprites/hidden - used by MMC3 IRQ counter"*. The counter therefore clocks once on **every** rendered line whether anything is on it or not, which is exactly the property a scanline counter needs to be worth having.

**The lesson worth carrying.** Both bugs are the same shape: a quantity copied from a reference implementation without its unit, and a fetch modelled as "nothing happens" because nothing is *displayed*. A board watching the address bus does not care what is displayed. `MoonMapperTests` now pins the filter width directly — twenty rises with an eight-dot gap must not clock the counter, and a nine-dot gap must.

**Still open.** 1536 pixels (2.5% of the frame) remain: our floor spans rows 197-225 against Mesen's 196-224, one scanline late, and rows 194-195 are the right shape in the wrong colour. Latching `RenderV` at dot 321 instead of 257 — which is where hardware's next-line prefetch actually reads — was tried and changed *nothing measurable*, so it was reverted rather than kept on the strength of an argument. The remaining line is not yet explained.

### 4.7 The four simple boards

**GxROM (66)** and **Color Dreams (11)** are the same idea with the register's two fields swapped: GxROM reads PRG from bits 4-5 and CHR from bits 0-1, Color Dreams the reverse. Both switch a whole 32K PRG bank and a whole 8K CHR bank from one write.

**Camerica (71)** is UxROM's layout with the bank register at `$C000-$FFFF` instead of `$8000`. Fire Hawk alone drives single-screen mirroring from `$9000`, so a write there is taken as the tell — a cart that never touches `$9000` keeps its header mirroring.

**NINA-003-006 (79)** puts its one register at `$4100-$5FFF`, below the cartridge window entirely. The bus already routes `$4020-$FFFF` to the mapper, so nothing special was needed to reach it.

### 4.4 MMC1's serial register, and why the CPU's honesty matters

MMC1 is not written directly. A write with bit 7 set resets the shift register and ORs the control register with `0x0C`; otherwise bit 0 of the value is shifted in, and the fifth such write commits all five bits to whichever of four registers address bits 14-13 select.

The trap: **an RMW instruction writes twice on consecutive cycles**, and the real board only honours the first. `Moon_CPU.md` §3.3 documents that this core's `INC`/`DEC`/`ASL`/`LSR`/`ROL`/`ROR` faithfully write the unmodified value back before the modified one — so an `INC $8000` here really does produce two writes one cycle apart, exactly as hardware would, and a mapper that ignored the rule would shift in two bits instead of one and mis-commit every fifth write.

`Cartridge.CpuCycle` is stamped by the bus before every mapper write for this reason alone, and `Mmc1.WritePrg` drops a write whose cycle is exactly one past the previous one. This is a case where getting the CPU right first made a downstream bug possible to avoid rather than possible to discover.

### 4.5 The scanline hook — **retired 2026-08-08**

This section described `IMapper.OnScanline()`, called from `Ppu.EndScanline` once per visible line. That hook no longer exists; §4.6a replaced it with per-fetch A12 edge detection, and §4.6b then corrected the filter's units. The argument it made — that a counter which ticked with rendering disabled would fire IRQs hardware never would — survives the replacement and is now a property of the fetch path itself, since `FetchForDot` returns immediately when rendering is off.

The prose is kept rather than deleted because the reasoning was correct and only the mechanism changed; a reader who finds `OnScanline` in an old commit should be able to learn why it existed and why it stopped.

---

### 4.8 Boards that count CPU cycles, and the cycles this core does not give them

Three of the boards below (§4.11, §4.12, §4.14) and one mode of a fourth (§4.15) drive their IRQ counter from the CPU clock rather than from the PPU's address bus. `IMapper` exposes that as `OnCpuCycle()`, gated behind `ClocksOnCpuCycle`.

**The gate is not decoration.** `MemoryBus.Tick` is the hottest loop in the core, and every board would otherwise pay an interface dispatch per CPU cycle for a feature ten of the sixteen do not use. The bus asks once, in its constructor, and stores the answer; the reference does the same thing for the same reason (`BaseMapper::EnableCpuClockHook`).

**What the hook deliberately omits is more interesting than what it includes.** `RunOamDma` transfers all 256 bytes in one call and charges 513 cycles to the frame budget; the PPU is *not* stepped for them. The same is true of the cycles the DMC steals. So in this core the PPU's timeline runs 513 cycles per DMA behind the CPU's, and a board clocked on the CPU's would drift ahead of the picture by that much every frame.

The first implementation clocked the mapper through both, on the reasoning that hardware plainly does. Skull & Crossbones then rendered its entire raster **five scanlines too high**, stably, at every frame sampled. 513 CPU cycles is 1539 dots is 4.51 scanlines, and the game re-latches its counter five times a frame, so the error could neither accumulate nor cancel — it simply sat there at one DMA's worth. Removing those two calls made the frame pixel-identical to the reference.

This is a **compensating error and is recorded as one**. The physically correct fix is for the PPU to advance during OAM DMA; the mapper hook is then free to count every cycle, as hardware does. What is implemented instead keeps the board on the same timeline the PPU is actually on in this core, which produces the right picture for the wrong reason. It was chosen over the correct fix because stepping the PPU through DMA touches the frame loop that §4.6b's MMC3 work was validated against days earlier, and buying one board's raster with a regression in another's would be a poor trade. The correct fix remains open; when it lands, `OnCpuCycle` should be restored to every cycle in the same change, and Skull & Crossbones is the test that will say whether both halves were done.

### 4.9 Nametables that are not the PPU's

`IMapper.SuppliesNametables` lets a board answer `$2000-$2FFF` from its own memory instead of the PPU's 2 KB of CIRAM. Only Sunsoft-4 uses it here (§4.13). `Ppu.ReadCiram` consults it per read rather than caching the answer, because the board toggles it at runtime; writes are dropped while it is on, since the substituted memory is CHR ROM.

**The `CIRAM` debug space does not follow it.** `MoonCore.ReadSpace` reads `Ppu.Ciram` directly, so while a board is supplying nametables the debugger shows the PPU's own RAM — which is stale, not what is on screen. This is left as it is deliberately: the space is named CIRAM and shows CIRAM, and a space that silently changed its backing store depending on a mapper register would be worse to debug against than one that is consistently literal. Anyone chasing a Sunsoft-4 nametable should read `CHR` at the page the board selected instead.

### 4.10 MMC2, and a bank chosen by the PPU rather than the CPU

Punch-Out!!'s board is the only one here whose CHR banking is driven by the *PPU's own fetches*. Two 4 KB windows each have two banks and a one-bit latch selecting between them; the latch is set by the address of a pattern fetch. The `$0000` window watches two exact addresses, `$0FD8` and `$0FE8`; the `$1000` window watches two eight-byte runs, `$1FD8-$1FDF` and `$1FE8-$1FEF`. The asymmetry is real hardware behaviour and not a simplification — MMC4 (mapper 10, not implemented) uses ranges for both.

**The ordering is the part worth getting right.** The fetch that trips a latch is still served by the bank on its way out; only the next fetch sees the new one. The reference expresses this with a deferred `_needChrUpdate` flag applied at the head of the following address change; `Mmc2.ReadChr` gets the same result by reading first and updating the latch afterwards, which is the same statement written from the other side.

**This core drives the latch from `ReadChr`, not from `OnPpuAddress`.** That looks like the wrong seam — `OnPpuAddress` is where every other board watches the bus — and it is a deliberate consequence of §4.6a. The addresses `Ppu.FetchForDot` publishes for pattern fetches are `BackgroundPatternBase | ((V >> 12) & 0x07)`: correct in bit 12, which is all an A12 watcher needs, and wrong in every other bit. A latch that must recognise `$0FD8` exactly cannot be driven from them. Should the fetch path ever publish true tile addresses, this should move to `OnPpuAddress` and the note should go with it.

### 4.11 Irem H3001, and a board with nothing to test it

Three switchable 8 KB PRG windows (`$8000`, `$A000`, `$C000`) with the fourth fixed to the last page, eight 1 KB CHR windows at `$B000-$B007`, and a 16-bit IRQ counter that decrements once per CPU cycle. The registers are decoded in full, so only the exact addresses listed do anything — a write to `$8001` is not a write to `$8000`.

The counter's latch is loaded high byte first (`$9005`) then low (`$9006`); `$9004` copies the latch into the counter and acknowledges; `$9003` bit 7 arms it and also acknowledges. Reaching zero fires **and disarms**, so a game gets one interrupt per reload — unlike the FME-7's counter (§4.14), which wraps and keeps running. The two are easy to conflate and the difference is the whole behaviour.

**No image in the reference library exercises this board.** The genuine H3001 titles are Japanese-only (`Daiku no Gen-san 2`, `Kaiketsu Yanchamaru 3`, `Spartan X 2`) and none is present. The four images the library reports as mapper 65 are: two hacks and one unknown whose headers are archaic and are really mapper 1 once §2.2 applies, and `Adventures in the Magic Kingdom (U) [a1]`, whose byte 6 and byte 7 nibbles are transposed — it is really an MMC3 game, and it renders a blank screen here whether loaded as mapper 65 or forced to mapper 4, so it is a bad dump rather than a board test.

**This board is verified by unit tests alone**, and that should be said out loud rather than inferred from the absence of a screenshot.

**One open difference, and a retracted explanation.** `Adventures in the Magic Kingdom (U) [a1]` renders a blank screen here and renders content in the reference. This was first written up as a non-finding on the grounds that the reference had overridden the header from its game database and so was running MMC3 rather than this board. **That explanation was wrong and was never checked.** The reference loads no database in a probe run — see `EmuSen_Debugging_Tools_Reference_v5.md` §3.47 — and the comparability gate added afterwards reports `mesen board=65` against `emusen board=65`. Both emulators are running *this* board on that file, and only one of them draws anything.

What is known: the game writes no bank register at all in 300 frames, `PPUMASK` reads `$1E` so rendering is enabled, the palette is still all zeroes, and `PC` sits at `$8020`. Forcing the same image to mapper 4 renders nothing either, so the dump is damaged in some way beyond its mapper nibble. That does not clear this board, and the difference should be treated as open rather than as understood.

### 4.12 Sunsoft-3

One switchable 16 KB PRG window with the second fixed to the last page, and four 2 KB CHR windows. Registers are decoded on `addr & $F800`, so each lives in the upper half of a 4 KB block: `$8800`, `$9800`, `$A800`, `$B800` for CHR, `$E800` for mirroring, `$F800` for PRG.

The counter is 16 bits fed through **one** address, `$C800`, high half first, with an internal toggle deciding which half a write lands in; `$D800` both arms the counter and resets that toggle to "high next", so a game that writes `$D800` before its two `$C800` writes always knows which half it is setting. The counter fires on the underflow *past* zero rather than on zero itself, then disarms.

**No image in the reference library uses this board either** — every image the library reported as mapper 67 turned out to be mapper 3 with a `DiskDude!` header (§2.2). Unit tests only, same as §4.11.

### 4.13 Sunsoft-4, and CHR ROM standing in for the nametables

PRG and CHR are conventional: one switchable 16 KB window, four 2 KB CHR windows at `$8000`-`$B000`, `$F000` carrying both the PRG bank (bits 0-2) and the work-RAM enable (bit 4).

What makes the board unusual is `$E000` bit 4, which hands the nametables to CHR ROM. `$C000` and `$D000` then hold two 1 KB CHR page numbers, and the board's own mirroring decides which of the two each of the four nametables shows. Both registers force bit 7 of the written value, because the nametable pages live in the upper half of CHR.

**The unit mismatch is the trap.** The CHR windows count in 2 KB pages and the nametable registers count in 1 KB pages, in the same board, written through adjacent registers. Getting this wrong produces a screen that is plausibly wrong rather than obviously wrong, and §4.6b is a recent reminder of how expensive a quantity copied without its unit can be.

After Burner is the one genuine image present. It matches the reference exactly at frames 300, 900 and 1800.

### 4.14 Sunsoft FME-7

Everything on this board goes through a command/parameter pair: `$8000` selects one of sixteen commands, `$A000` supplies its argument. Commands 0-7 are the eight 1 KB CHR windows, 9-11 the three switchable 8 KB PRG windows, 12 mirroring, 13-15 the IRQ.

**`$6000` is a PRG ROM window by default**, which is unlike every other board here and is worth stating because it is the state the machine boots in. Command 8's bit 6 switches it to RAM and bit 7 then enables that RAM; with bit 6 clear the low six bits are an 8 KB ROM bank instead.

The counter decrements once per CPU cycle when command 13's bit 7 is set, and on underflow it **wraps and keeps counting**, raising the line only if bit 0 is also set. Acknowledging does not stop it — the interrupt returns one full period later. This is the opposite of §4.11's behaviour and the distinction is why both are spelled out.

The 5B expansion audio at `$C000`/`$E000` is accepted and discarded. This core carries no expansion audio for any board, so that is the existing state of the project rather than a gap introduced here; Mr. Gimmick is the only image in the library that would use it.

Batman: Return of the Joker matches the reference exactly at frames 300, 900 and 1800.

**Mr. Gimmick does not count as a second confirmation, though it was first recorded as one.** `Mr. Gimmick (E)` is a PAL image; the reference detects that and runs it at PAL timing, while this core reports NTSC unconditionally. The frames matched pixel for pixel anyway — the screen in question is static, and a static screen matches across almost any timing — so the agreement was real and meaningless at once. The comparability gate (`EmuSen_Debugging_Tools_Reference_v5.md` §3.48) reports it as `NOT-COMPARABLE: region differs`, which is the correct reading and was not the one taken by hand an hour earlier. Whether this core models PAL timing at all is a separate question this raised and did not answer.

### 4.15 RAMBO-1, and two ways to clock one counter

Tengen's board is MMC3's shape with three differences that matter.

**A third switchable PRG bank.** Register 15 joins 6 and 7, and `$8000` bit 6 decides whether the fixed page sits in the third window or the first.

**CHR that can be eight 1 KB pages instead of six.** `$8000` bit 5 gives the two 2 KB pairs their own registers (8 and 9). When it is clear the pairs are `R0`/`R0+1` and `R1`/`R1+1` — note that unlike MMC3 the low bit of the register is *not* masked, so an odd value moves the pair rather than being ignored.

**A counter that can be clocked by the CPU.** `$C001` bit 0 selects cycle mode, in which the counter advances once every four CPU cycles and the A12 path is disconnected entirely. Three details make this work:

- **The A12 filter is far wider than MMC3's.** The reference passes `30` to its shared `A12Watcher` where MMC3 effectively passes three CPU cycles; in this core both go through `Mappers/A12Watcher.cs` with the threshold stated in PPU dots, 30 against MMC3's 9. Deriving 30 from the reference needs care: its `_cyclesDown` starts at 1 on the falling edge and its test is `> minDelay`, so `> 30` means thirty dots elapsed, not thirty-one.
- **A reload lands one or two above the latch**, never on it — `latch + 1` when the latch is 0 or 1, `latch + 2` otherwise. This is what puts a RAMBO-1 split a line below where the same latch would put an MMC3's.
- **Leaving cycle mode still owes the counter one clock.** `$C001` clearing bit 0 while cycle mode was on sets a force-clock flag that survives until the divider next completes. The reference attributes this to Skull & Crossbones, which is the game that found the §4.8 bug here too.

The line falls two CPU cycles after an A12-clocked counter reaches zero and one after a cycle-clocked one, modelled as a small countdown in `OnCpuCycle`.

Klax, Shinobi and Skull & Crossbones all match the reference exactly at frames 300 and 900. Skull & Crossbones is the interesting one: it drives five `$C001` writes per frame, alternating between the two modes with latches of 227, 113, 116 and 206, and it is the only image found so far whose picture depends on the CPU-cycle path being on the same timeline as the PPU.

---

## 5. Transfers and input

### 5.1 OAM DMA

Writing a page number to `$4014` copies 256 bytes from `$XX00` into OAM starting at the current `OAMADDR`. The copy is performed immediately through the normal decode — so a page pointed at registers reads what those registers would give — and the cost is banked in `PendingDmaCycles` for the core's timing loop to subtract.

513 cycles is charged flat. Real hardware charges 514 when the transfer starts on an odd CPU cycle; that one-cycle alignment detail is not modelled.

### 5.2 Controllers

A pad is a parallel-load shift register. Writing `$4016` with bit 0 set holds it loading; clearing the bit latches the button state, after which each read of `$4016`/`$4017` returns the next bit and shifts.

Bit order is `A, B, Select, Start, Up, Down, Left, Right`. Past the eighth read a real pad returns 1s, which is modelled; the upper bits of the returned byte come from open bus, which is approximated by the last value the bus carried rather than emulated properly.

---

## 6. The write-observer seam

`MemoryBus` holds an `IWriteObserver?` and names no debug type. `MoonDebugTarget` implements the interface and supplies the meaning, exactly as `SnesDebugTarget` does for Venus — the core compiles and runs with the whole shell absent.

`Moon.Memory.IWriteObserver` is a deliberate twin of `Venus.Memory.IWriteObserver` rather than a shared type. Two identical one-method interfaces is not yet evidence of the right shared abstraction, and hoisting it would mean editing Venus to serve a Moon convenience. Worth revisiting when a third core makes the shape a rule instead of a coincidence.

Currently wired: work RAM, PPU register writes, APU register writes, and PRG RAM. The CHR/CIRAM/OAM/palette write paths inside the PPU do not report yet — the same additive gap Venus's own `Man pages` note for its VRAM/CGRAM/OAM paths.

## 7. The battery save

Boards with a battery (`HasBattery`, §4) keep PRG RAM across sessions in a `.srm` file, autosaved every 300 frames by `MoonCore` and again on shutdown.

**The path is still `Path.ChangeExtension(RomPath, ".srm")` — beside the ROM**, which is where it has always been and where Venus deliberately does *not* put it (`Venus_Memory.md` §2.4: a ROM can live anywhere, including somewhere read-only, and the ROM folder is the user's own library rather than emulator output). Moving Moon onto `SaveLibrary.SramPathFor` alongside Venus is a known, deliberately deferred change — it is user-visible and needs a copy-don't-move migration for saves that already exist. See `EmuSen_Galaxia.md` §6.

What did change: both directions go through `Galaxia`'s `AtomicFile`, so an interrupted autosave leaves the previous save whole instead of truncated (`EmuSen_Galaxia.md` §4). `LoadSram` copies whichever of the file and PRG RAM is smaller, tolerating a size mismatch rather than refusing to boot.

`CoreOptions.BatteryRamDisabled` (Pharaoh's `--nobattery`) leaves `_savePath` null, which disables the write as well as the read — see `EmuSen_Multicore.md` §6.
