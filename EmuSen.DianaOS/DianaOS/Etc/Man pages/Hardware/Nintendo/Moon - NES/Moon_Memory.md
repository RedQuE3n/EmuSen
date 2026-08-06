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
| 11 | Color Dreams | 32K switchable | 8K switchable | GxROM's fields swapped, §4.7 |
| 66 | GxROM | 32K switchable | 8K switchable | one register does both, §4.7 |
| 71 | Camerica | `$C000` switches, `$8000` fixed | CHR RAM | §4.7 |
| 79 | NINA-003-006 | 32K switchable | 8K switchable | register at `$4100`, §4.7 |

Measured against a 3,536-ROM set, these ten boards cover **83.8%** of it, up from 59.0% before MMC3. The largest remaining gaps are mappers 6 (77 ROMs), 64 (66), 65 (56) and 99 (34) — none above 2.2% each. An unimplemented mapper number throws a `NotSupportedException` naming the number, rather than loading as NROM and behaving inexplicably.

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

### 4.7 The four simple boards

**GxROM (66)** and **Color Dreams (11)** are the same idea with the register's two fields swapped: GxROM reads PRG from bits 4-5 and CHR from bits 0-1, Color Dreams the reverse. Both switch a whole 32K PRG bank and a whole 8K CHR bank from one write.

**Camerica (71)** is UxROM's layout with the bank register at `$C000-$FFFF` instead of `$8000`. Fire Hawk alone drives single-screen mirroring from `$9000`, so a write there is taken as the tell — a cart that never touches `$9000` keeps its header mirroring.

**NINA-003-006 (79)** puts its one register at `$4100-$5FFF`, below the cartridge window entirely. The bus already routes `$4020-$FFFF` to the mapper, so nothing special was needed to reach it.

### 4.4 MMC1's serial register, and why the CPU's honesty matters

MMC1 is not written directly. A write with bit 7 set resets the shift register and ORs the control register with `0x0C`; otherwise bit 0 of the value is shifted in, and the fifth such write commits all five bits to whichever of four registers address bits 14-13 select.

The trap: **an RMW instruction writes twice on consecutive cycles**, and the real board only honours the first. `Moon_CPU.md` §3.3 documents that this core's `INC`/`DEC`/`ASL`/`LSR`/`ROL`/`ROR` faithfully write the unmodified value back before the modified one — so an `INC $8000` here really does produce two writes one cycle apart, exactly as hardware would, and a mapper that ignored the rule would shift in two bits instead of one and mis-commit every fifth write.

`Cartridge.CpuCycle` is stamped by the bus before every mapper write for this reason alone, and `Mmc1.WritePrg` drops a write whose cycle is exactly one past the previous one. This is a case where getting the CPU right first made a downstream bug possible to avoid rather than possible to discover.

### 4.5 The scanline hook

`IMapper.OnScanline()` is called from `Ppu.EndScanline` for each visible line, and only while rendering is enabled — a real MMC3 counts PPU address-line transitions that only happen when the PPU is fetching, so a counter that ticked with rendering off would fire IRQs during vblank that hardware never would.

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
