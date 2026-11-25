# Venus — SNES Cartridge Coprocessors

Core crate directory: `EmuSen/Cores/Nintendo/VenusRT - SNES` (crate `venusrt`).

This manual documents the four cartridge coprocessor families implemented in `venusrt`: the SA-1 (`src/chips/sa1.rs`), the SuperFX / GSU (`src/chips/gsu.rs`), the OBC1 (`src/chips/obc1.rs`), and the NEC uPD77C25 / uPD96050 low-level engine (`src/chips/necdsp.rs`). Built-in high-level replacements for the NEC DSP family are documented in `Venus_DspHle.md`.

---

## 1. Cartridge Detection and Bus Dispatch (`src/cart.rs`, `src/bus.rs`)

Cartridge headers select enhancement chips automatically during `Cartridge::new`:

| Chip | Header Trigger | RAM Allocation (`cart.sram`) | S-CPU Bus Interception (`src/bus.rs`) |
|---|---|---|---|
| **SA-1** | `chipset >> 4 == 3` (`$32–$35`) and `map_mode & $0F == 3` | Header `sram_bytes()` (BW-RAM) | `Sa1::snes_maps(addr)` overrides standard cart decode; catches up on every S-CPU instruction (or `sa1_catch_up` bound) and before any board access |
| **SuperFX / GSU** | `chipset` in `$13..=$1A` and `map == LoRom` | Extended header `FFBDh` (`1024 << n` for `1..=7`), default `32 KiB` (`$8000`) | `Gsu::snes_maps(addr)` overrides standard cart decode; catches up on every S-CPU instruction and before any board access |
| **OBC1** | `chipset >> 4 == 2` (e.g. `$25`) and `map == LoRom` | Header `sram_bytes()` (`8 KiB`) | `Obc1::snes_maps(addr)` intercepts `$00–$3F`/`$80–$BF:$6000–$7FFF` |
| **NEC DSP / ST01x** | `chipset` in `$03..=$05` (DSP-n) or `$F6` (ST010/ST011) | Standard SRAM (or on-chip 4 KiB RAM for ST01x) | `Cartridge::dsp_port(addr)` intercepts mapped DR/SR/RAM slots; catches up before each port read/write and at frame end |

Coprocessor interrupts drive `System::cart_irq` (from `Sa1::snes_irq()` or `Gsu::snes_irq()`), which ORs directly with the 5A22 H/V IRQ line.

---

## 2. SA-1 (`src/chips/sa1.rs`)

The SA-1 is a second WDC 65C816 (`crate::cpu::Cpu`) embedded in the cartridge alongside 2 KiB of internal RAM (`I-RAM`), battery-backed cartridge RAM (`BW-RAM`), a Super MMC ROM bank controller, dual-side interrupt/vector logic, an H/V timer, a 40-bit arithmetic unit, a variable-length bitstream reader, and a DMA/character-conversion engine.

### 2.1 Clocking and Bus Contention Arbitration

- **Base Cycle Rates**: The SA-1 runs at **10.74 MHz** (`FAST = 2` master clocks per cycle for I-RAM, ROM, I/O, and internal cycles). BW-RAM accesses run at half speed (**5.37 MHz**, `BWRAM = 4` master clocks per cycle).
- **Hardware Contention Holds**: Both CPUs share physical ROM (`kind = 1`), BW-RAM (`kind = 2`), and I-RAM (`kind = 3`). The S-CPU always has bus priority and never stalls. On every S-CPU cycle or DMA byte targeting one of these three memories, `System::hold_cycle` / `System::hold_sa1` registers a hold interval `[from, to)` on `Sa1`:
  - Normal S-CPU cycle: holds `[end - clocks + 3, end)` (from SYSCLK rising edge to cycle end).
  - DMA byte transfer: holds `[clock + 4, clock + 8)`.
  - If an upcoming SA-1 access overlaps an S-CPU hold on the same memory `kind`, `Sa1::access` stalls the SA-1 until `hold.to` (restarting BW-RAM's 2-cycle window if interrupted).
  - If an S-CPU hold arrives after the SA-1 has already speculatively stepped past its horizon, `Sa1::hold` inspects its 16-entry `log` of recent accesses, detects any conflicting access, and shifts that access and all subsequent SA-1 timestamps forward by `to - start`.
- **Catch-Up Setting**: By default (`sa1_catch_up = "1"`), `Machine::instruction_end` catches the SA-1 up after every S-CPU instruction. Optional values `"4"` and `"16"` defer catch-up while the S-CPU executes outside the cartridge; board accesses, frame boundaries, and observed debugger frames always catch up immediately.
- **Reset Behavior**: On power-on, `ccnt = $20` (SA-1 held in reset) and `cpu.s = $01FF`. Clearing `CCNT` bit 5 (`$2200`) schedules `reset_pending`, which runs the 65C816 reset sequence (three stack reads leaving `S = $01FC`, loading `PC` from `CRV` `$2203–$2204`). Setting `CCNT` bit 5 back to 1 holds the SA-1 in reset and clears `CIWP` (`$222A = 0`).

### 2.2 Dual 65C816 Memory Maps

| Address Range | S-CPU View (`snes_read` / `snes_write`) | SA-1 View (`Sa1Bus::load` / `store`) |
|---|---|---|
| `$00–$3F`, `$80–$BF` : `$0000–$07FF` | Console WRAM (not mapped to SA-1) | **2 KiB I-RAM** (2 master clocks; writes gated by `CIWP`) |
| `$00–$3F`, `$80–$BF` : `$2200–$23FF` | SA-1 I/O registers (`$2300` SFR, `$230E` version `$23`; SNES-side writes) | SA-1 I/O registers (`$2301–$230D`; SA-1-side writes) |
| `$00–$3F`, `$80–$BF` : `$3000–$37FF` | **2 KiB I-RAM** (writes gated by `SIWP`) | **2 KiB I-RAM** mirror (writes gated by `CIWP`) |
| `$00–$3F`, `$80–$BF` : `$6000–$7FFF` | **8 KiB BW-RAM window** selected by `BMAPS & $1F` (`$2224`) | **8 KiB BW-RAM window** selected by `BMAP` (`$2225`): linear (`BMAP & $1F`) if bit 7 = 0, or **bitmap view** (`BMAP & $7F`) if bit 7 = 1 |
| `$00–$3F`, `$80–$BF` : `$8000–$FFFF` | Super MMC LoROM regions + optional `SNV`/`SIV` vector override in bank `$00`/`$80` | Super MMC LoROM regions + mandatory `CNV`/`CIV`/`CRV` vectors at `$00/$80:FFEA/FFEE/FFFC` |
| `$40–$4F` : `$0000–$FFFF` | **Linear BW-RAM** (up to 256 KiB; reads stream Character Conversion 1 when active) | **Linear BW-RAM** (4 master clocks) |
| `$50–$5F` : `$0000–$FFFF` | Unmapped on SA-1 | **Linear BW-RAM** mirror (`address & $3FFFF`) |
| `$60–$6F` : `$0000–$FFFF` | Unmapped on SA-1 | **BW-RAM Bitmap View** (1 MiB address window: 4bpp or 2bpp per `BBF` `$223F`) |
| `$C0–$FF` : `$0000–$FFFF` | Super MMC HiROM (four 1 MiB banks) | Super MMC HiROM (four 1 MiB banks, accelerated via cached `rom_banks` table) |

### 2.3 Super MMC ROM Banking (`$2220–$2223`)

Four registers (`CXB`, `DXB`, `EXB`, `FXB` in `mmc[0..4]`, initialized to `[0, 1, 2, 3]`) map four 1 MiB ROM blocks:
- **HiROM (`$C0–$FF`)**: Always uses `mmc[(bank >> 4) & 3] & 7` as the 1 MiB block index (`block << 20 | (bank & $0F) << 16 | offset`).
- **LoROM (`$00–$3F`, `$80–$BF`, `$8000–$FFFF`)**: Region index is `((bank >> 5) & 1) | ((bank >> 6) & 2)`. When `mmc[region] & $80 != 0`, uses `mmc[region] & 7`; otherwise uses the default identity `region`.
- All offsets pass through `Cartridge::mirror(at, rom.len())`.

### 2.4 BW-RAM Protection and Bitmap View

- **Write Protection (`SBWE` `$2226`, `CBWE` `$2227`, `BWPA` `$2228`)**: The protected region at the start of BW-RAM has size `min(256 << (bwpa & $0F), 256 KiB)`. Writes at offsets `>= protected_size` are always allowed; writes below `protected_size` are allowed only when `(sbwe | cbwe) & $80 != 0`. (`bwpa` initializes to `$FF`.)
- **I-RAM Protection (`SIWP` `$2229`, `CIWP` `$222A`)**: 8-bit masks where bit `k` enables writes to the `k`-th 256-byte block of I-RAM (`(offset >> 8) & 7`) from the S-CPU (`SIWP`) or SA-1 (`CIWP`).
- **Bitmap Mode (`BBF` `$223F`)**: Bit 7 = `0` selects **4bpp** (2 pixels/byte, low nibble first); bit 7 = `1` selects **2bpp** (4 pixels/byte, bits 1–0 first). Accessible to the SA-1 in banks `$60–$6F` and in `$6000–$7FFF` when `BMAP` (`$2225`) bit 7 is set.

### 2.5 SA-1 Register Map (`$2200–$230E`)

| Address | Side | Name | Function |
|---|---|---|---|
| `$2200` | SNES W | `CCNT` | Bit 7: SA-1 IRQ trigger (`cfr |= $80`); Bit 6: SA-1 RDY/wait halt; Bit 5: SA-1 reset (`1`=hold reset & clear `CIWP`, `1→0`=trigger reset); Bit 4: SA-1 NMI trigger (`cfr |= $10`, sets `nmi_pending` if `cie & $10`); Bits 3–0: message to SA-1 (`cfr` low nibble). |
| `$2201` | SNES W | `SIE` | S-CPU IRQ enable: bit 7 = SA-1 IRQ, bit 5 = Character Conversion 1 IRQ. |
| `$2202` | SNES W | `SIC` | S-CPU IRQ clear (`sfr &= !(v & $A0)`). |
| `$2203–$2208` | SNES W | `CRV`, `CNV`, `CIV` | SA-1 16-bit Reset, NMI, and IRQ vectors (low/high pairs). |
| `$2209` | SA-1 W | `SCNT` | Bit 7: S-CPU IRQ trigger (`sfr |= $80`); Bit 6: S-CPU IRQ vector override (`SIV`); Bit 4: S-CPU NMI vector override (`SNV`); Bits 3–0: message to S-CPU. |
| `$220A` | SA-1 W | `CIE` | SA-1 interrupt enable: bit 7 = S-CPU IRQ, bit 6 = Timer IRQ, bit 5 = DMA IRQ, bit 4 = S-CPU NMI. |
| `$220B` | SA-1 W | `CIC` | SA-1 interrupt clear (`cfr &= !(v & $F0)`; bit 4 clears `nmi_pending`). |
| `$220C–$220F` | SA-1 W | `SNV`, `SIV` | S-CPU override NMI and IRQ vectors (active when `SCNT` bit 4 / bit 6 set). |
| `$2210–$2215` | SA-1 W | `TMC`, `CTR`, `HCNT`, `VCNT` | Timer control (`$2210`: bit 7 linear 512×512 vs H/V 341×lines, bits 1–0 = `1` H, `2` V, `3` H+V), restart (`$2211`), 9-bit `hcmp` (`$2212–$2213`) and `vcmp` (`$2214–$2215`). Timer counts dots (`(clock - timer_zero) / 4`) and sets `cfr |= $40` on match. |
| `$2220–$2223` | SNES W | `CXB–FXB` | Super MMC 1 MiB bank registers. |
| `$2224`, `$2225` | SNES/SA-1 W | `BMAPS`, `BMAP` | `$6000–$7FFF` BW-RAM window bank select (SNES vs SA-1; bit 7 of `BMAP` selects bitmap mode). |
| `$2226–$222A` | SNES/SA-1 W | `SBWE`, `CBWE`, `BWPA`, `SIWP`, `CIWP` | BW-RAM and I-RAM write protection registers. |
| `$2230` | SA-1 W | `DCNT` | Bit 7: DMA enable; Bit 5: char-conv enable; Bit 4: char-conv 1 (`1`) vs 2 (`0`); Bit 2: dest BW-RAM (`1`) vs I-RAM (`0`); Bits 1–0: source (`0`=ROM, `1`=BW-RAM, `2/3`=I-RAM). |
| `$2231` | Both W | `CDMA` | Character conversion parameters: bits 1–0 depth (`0`=8bpp, `1`=4bpp, `2`=2bpp, `3`=8bpp), bits 4–2 tile width (`1 << min(v, 5)`). Bit 7 terminates Character Conversion 1 (`char1 = None`). |
| `$2232–$2237` | Both W | `SDA`, `DDA` | 24-bit source (`$2232–$2234`) and destination (`$2235–$2237`) DMA addresses. Writing `$2236` starts Char Conv 1 (raising `sfr |= $20`) or Normal DMA to I-RAM; writing `$2237` starts Normal DMA to BW-RAM. |
| `$2238–$2239` | SA-1 W | `DTC` | 16-bit Normal DMA byte count. |
| `$223F` | SA-1 W | `BBF` | BW-RAM bitmap format (bit 7: `0`=4bpp, `1`=2bpp). |
| `$2240–$224F` | SA-1 W | `BRF0–15` | 16-byte bitmap register file. Writing `$2247` or `$224F` in Char Conv 2 converts 8 pixels into a tile row in I-RAM (`row = 0..15`, double-buffered across 2 tiles). |
| `$2250–$2254` | SA-1 W | `MCNT`, `MA`, `MB` | Arithmetic control (`$2250`: `0`=signed 16×16 multiply, `1`=signed/unsigned Euclidean division, `2/3`=40-bit signed MAC; bit 1 resets `MR` and `overflow`), 16-bit `MA` (`$2251–$2252`), 16-bit `MB` (`$2253–$2254`; writing `$2254` executes). |
| `$2258–$225B` | SA-1 W | `VBD`, `VDA` | Variable-length bit reader: `$2258` bits 3–0 = bit length (`0` = 16 bits, else `1..=15`), bit 7 = auto-increment on `$230D` read (`0` = fixed mode advances immediately on `$2258` write). `$2259–$225B` = 24-bit ROM start address (writing `$225B` resets `vbit = 0`). |
| `$2300` | SNES R | `SFR` | S-CPU flags: bit 7 SA-1 IRQ, bit 6 S-CPU IRQ vector override, bit 5 Char Conv 1 IRQ, bit 4 S-CPU NMI vector override, bits 3–0 message. |
| `$2301` | SA-1 R | `CFR` | SA-1 flags: bit 7 S-CPU IRQ, bit 6 Timer IRQ, bit 5 DMA IRQ, bit 4 S-CPU NMI, bits 3–0 message. |
| `$2302–$2305` | SA-1 R | `HCR`, `VCR` | Latched H/V timer counters (reading `$2302` latches both `hlatch` and `vlatch`). |
| `$2306–$230A` | SA-1 R | `MR0–MR4` | 40-bit arithmetic result (multiply product, MAC sum, or division quotient in `MR0–1` and remainder in `MR2–3`). Division by zero produces `0`. |
| `$230B` | SA-1 R | `OF` | Bit 7 = 40-bit MAC signed overflow flag (`-(2^39)..2^39`); bits 6–0 = `mdr`. |
| `$230C–$230D` | SA-1 R | `VDPL/H` | 16-bit unaligned bitstream window at `VDA + vbit` in ROM. Reading `$230D` advances `vbit += vlen()` when `VBD` bit 7 is set. |
| `$230E` | SNES R | `VC` | Version code (`$23`). |

### 2.6 DMA and Arithmetic Details

- **Normal DMA**: Copies `DTC` bytes between ROM/BW-RAM/I-RAM and I-RAM/BW-RAM, charges `DTC * 2` master clocks for ROM→I-RAM or `DTC * 4` master clocks for all BW-RAM transfers, and sets the SA-1 DMA IRQ flag (`cfr |= $20`).
- **Character Conversion 1**: Activated by writing `DDA` (`$2236`) with `DCNT` bits 5 and 4 set. Each S-CPU read from linear BW-RAM (`$40–$4F`) returns the next bitplaned byte computed on the fly from the bitmap at `SDA` according to `CDMA` depth and tile width, until cleared via `CDMA` bit 7.
- **Character Conversion 2**: Activated with `DCNT & $B0 == $A0`. Writing `$2247` (low 8 bytes) or `$224F` (high 8 bytes) converts 8 packed pixels in `BRF` into 2/4/8bpp SNES bitplanes in I-RAM at `(DDA & $7FF) + tile * (8 * bpp)`.
- **Arithmetic Unit**: Mode `1` performs Euclidean division (`a.div_euclid(b)`, `a.rem_euclid(b)` for signed 16-bit `MA` and unsigned 16-bit `MB`), yielding a non-negative remainder; `MB == 0` yields `MR = 0`.
- **Debugger & Save State**: Exposes memory spaces `SA1IRAM` (10), `BWRAM` (11), `SA1BUS` (12) and processor 2 (`SA1`, 24-bit PC, 65C816 disassembler). Full state including `I-RAM` and the 16-entry contention log is packed in the `Coprocessor.Sa1` save-state group.

---

## 3. SuperFX / GSU (`src/chips/gsu.rs`)

The Graphics Support Unit (Mario Chip 1, GSU-1, and GSU-2) is a 16-bit RISC graphics coprocessor with 16 general-purpose registers, a 512-byte code cache, a ROM prefetch buffer, and bitplane pixel-plotting hardware.

### 3.1 Variants, Clocks, and Bus Ownership

- **Variant Selection**: Cartridges with `chipset` in `$13..=$1A` instantiate GSU-1 (`VCR = 1`) for ROM sizes `<= 1 MiB` and GSU-2 (`VCR = 4`) for ROM sizes `> 1 MiB`. Game Pak RAM size is read from extended header byte `FFBDh` (`1024 << n` for `1..=7`), defaulting to 32 KiB.
- **Clock Rate (`CLSR` `$3039`)**: Bit 0 = `0` selects **10.74 MHz** (`cycle = 2` master clocks; uncached memory `slow = 3` GSU cycles); bit 0 = `1` selects **21.48 MHz** (`cycle = 1` master clock; uncached memory `slow = 5` GSU cycles).
- **Bus Ownership (`SCMR` `$303A`)**:
  - While `SFR.GO` (`$3030` bit 5) is set and `SCMR.RON` (bit 4) is set, the GSU owns ROM: S-CPU ROM reads return the hardcoded running vector table `[0x0100, 0x0100, 0x0104, 0x0100, 0x0100, 0x0108, 0x0100, 0x010C]` indexed by `(address >> 1) & 7` (allowing SNES interrupt handlers to bounce into WRAM).
  - While `SFR.GO` and `SCMR.RAN` (bit 3) are set, the GSU owns Game Pak RAM: S-CPU reads return open bus (`None`) and S-CPU writes are dropped.

### 3.2 Memory Map and I/O Registers

| Region | S-CPU Mapping | GSU Mapping |
|---|---|---|
| `$00–$3F`, `$80–$BF` : `$3000–$34FF` | GSU registers (`$3000–$303F`, mirrored across `$3000–$30FF` and `$3300–$34FF` with `$3020–$302F` mapped to `$3030–$303F`) and 512-byte code cache (`$3100–$32FF`) | Not addressable |
| `$00–$3F`, `$80–$BF` : `$6000–$7FFF` | Low 8 KiB of Game Pak RAM (when `!(GO && RAN)`) | — |
| `$00–$3F`, `$80–$BF` : `$8000–$FFFF` | LoROM 32 KiB banks (or running vectors when `GO && RON`) | LoROM 32 KiB banks (banks `$00–$3F`) |
| `$40–$5F`, `$C0–$DF` : `$0000–$FFFF` | HiROM 64 KiB banks (or running vectors when `GO && RON`) | HiROM 64 KiB banks (banks `$40–$5F`) |
| `$70–$71`, `$F0–$F1` : `$0000–$FFFF` | Full Game Pak RAM (up to 128 KiB, when `!(GO && RAN)`) | Game Pak RAM (`RAMBR` selects bank `$70`/`$71`) |

**Key I/O Registers (`$3000–$303F`)**:
- `$3000–$301F`: `R0–R15` (16-bit). S-CPU writes latch the even (low) byte into `latch` and commit the full 16-bit word on the odd (high) byte write. Writing `R14` high byte refills `rom_buffer` and schedules `rom_ready`; writing `R15` high byte starts execution (`SFR.GO = 1`, prefetches first opcode into `pipe`).
- `$3030–$3031` (`SFR`): Status/Flag Register: bit 1 `Z`, bit 2 `CY`, bit 3 `S`, bit 4 `OV`, bit 5 `GO`, bits 9–8 `ALT2`/`ALT1`, bit 12 `B` (WITH), bit 15 `IRQ` (cleared on S-CPU read of `$3031`). Clearing `GO` via `$3030` resets `CBR = 0` and invalidates all cache lines; setting `GO` starts execution at `R15`.
- `$3033` (`BRAMR`): Backup RAM register.
- `$3034` (`PBR`): Program Bank Register (writing invalidates cache lines).
- `$3036` (`ROMBR`, R): ROM buffer bank (set by GSU `ROMB`).
- `$3037` (`CFGR`, W): Bit 7 masks GSU IRQ to S-CPU; bit 5 (`MS0`) selects fast multiply (`MULT`/`UMULT` 0 extra cycles instead of 1; `FMULT`/`LMULT` 1 multiplier hold cycle instead of 5).
- `$3038` (`SCBR`, W): Screen Base Register (in 1 KiB units, `scbr * $400`).
- `$3039` (`CLSR`, W): Clock Speed Register (bit 0: `0`=10.74 MHz, `1`=21.48 MHz).
- `$303A` (`SCMR`, W): Screen Mode Register: bits 1–0 color depth (`0`=2bpp, `1/2`=4bpp, `3`=8bpp), bit 2 + bit 5 screen height (`00`=128, `01`=160, `10`=192, `11`=OBJ), bit 3 `RAN`, bit 4 `RON`.
- `$303B` (`VCR`, R): Version Code Register (`1` for GSU-1, `4` for GSU-2).
- `$303C` (`RAMBR`, R): RAM bank (`0` or `1`, set by GSU `RAMB`).
- `$303E–$303F` (`CBR`, R): Cache Base Register.

### 3.3 R15 Pipeline, Prefixes, Cache, and Memory Timing

- **Pipelined Fetch & Branch Delay Slot**: `Gsu::pipe` holds the next opcode byte. Each `step()` executes `pipe` while advancing `R15` (unless the previous opcode wrote `R15`, setting `jumped = true`) and fetching the next `pipe` byte. Consequently, every branch (`BRA`, `BGE`, `BLT`, `BNE`, `BEQ`, `BPL`, `BMI`, `BCC`, `BCS`, `BVC`, `BVS`), `LOOP`, `JMP`, `LJMP`, and `STOP` executes the byte immediately following it in its delay slot before transferring control.
- **Prefix Registers**: `ALT1` (`$3D`), `ALT2` (`$3E`), `ALT3` (`$3F`), `TO Rn` (`$10–$1F` when `!b`), `FROM Rn` (`$B0–$BF` when `!b`), and `WITH Rn` (`$20–$2F`, setting `sreg = dreg = n`, `b = true`, `SFR` bit 12) modify the next non-prefix opcode and reset afterward. Branches (`$05–$0F`) explicitly preserve active prefixes across the branch.
- **512-Byte Code Cache (`32 × 16`-byte lines)**: Addresses with `pc.wrapping_sub(cbr) < 512` hit the cache (`1` GSU cycle per byte if `lines` bit is set). On a cache miss, the entire 16-byte line is loaded from ROM/RAM at a stall cost of `16 * slow() + 2` GSU cycles. Writing byte 15 of any cache line via `$3100–$32FF` marks that line valid; `CACHE` (`$02`) sets `CBR = R15 & $FFF0` and invalidates all lines if `CBR` changed; `LJMP` invalidates all lines.
- **ROM Buffer (`R14`)**: Writing `R14` immediately loads `rom_buffer` from `ROMBR:R14` and schedules `rom_ready` at `5` cycles (10.74 MHz) or `7` cycles (21.48 MHz) after the opcode. `GETB`/`GETBH`/`GETBL`/`GETBS` (`$EF`) and `GETC` (`$DF`) stall for any remaining cycles until `rom_ready`.
- **Game Pak RAM**: 16-bit RAM reads and writes pair `addr` with `addr ^ 1` (swapping bytes at odd addresses). Stores (`STB`/`STW`/`SM`/`SBK`) post to the RAM port and set `ram_ready`; subsequent RAM loads, stores, `RAMB`, or `RPIX` stall until `ram_ready`.

### 3.4 Plot Hardware (`PLOT` / `RPIX`)

- **Color & Plot Option Registers**: `COLOR` (`$4E`) and `GETC` (`$DF`) write `COLR` through `POR` bit 2 (high-nibble swap `(v & $F0) | (v >> 4)`) and bit 3 (freeze high nibble). `CMODE` (`ALT1 + $4E`) writes `POR` bits 4–0:
  - Bit 0: Plot color 0 opaque (`1`) vs transparent (`0`).
  - Bit 1: Dither (for 2bpp/4bpp, uses upper nibble of `COLR` when `(x ^ y) & 1 != 0`).
  - Bit 2: High-nibble color source.
  - Bit 3: Freeze upper 4 bits of `COLR` (and limits transparency test to low nibble in 8bpp).
  - Bit 4: Force OBJ mode layout.
- **Bitplane Tile Address**: For `(X = R1 & $FF, Y = R2 & $FF)` and `bpp` in `{2, 4, 8}`:
  - Height `0` (128 px): `tile = (x / 8) * $10 + (y / 8)`
  - Height `1` (160 px): `tile = (x / 8) * $14 + (y / 8)`
  - Height `2` (192 px): `tile = (x / 8) * $18 + (y / 8)`
  - Height `3` (OBJ mode, or `POR` bit 4): `tile = (y / $80) * $200 + (x / $80) * $100 + ((y / 8) & $0F) * $10 + ((x / 8) & $0F)`
  - Row byte offset: `tile * 8 * bpp + scbr * $400 + (y & 7) * 2`, with bitplane `p` at `base + (p >> 1) * 16 + (p & 1)` and bit `7 - (x & 7)`.
- **Pixel Cache Timing**: `PLOT` (`$4C`) updates RAM immediately, sets the bit in `pc_valid`, increments `R1`, and models primary/secondary 8-pixel row cache flushes (`pcf_ready`) when all 8 pixels are plotted or when `(Y, X/8)` moves to a new block (1 RAM access per bitplane if all 8 pixels plotted, else 2 accesses for read-modify-write). `RPIX` (`ALT1 + $4C`) flushes and reads back the pixel at `(R1, R2)` with cost `1 + stall + 3 * bpp * slow() + 1` cycles.
- **Debugger & Save State**: Exposes spaces `GSURAM` (8), `GSUBUS` (9) and processor 2 (`GSU`, 24-bit PC at `PBR:pipe_at`, 27 registers, `src/disasm/gsu.rs`). Full state including 512-byte cache and timing deadlines is packed in `Coprocessor.Gsu`.

---

## 4. OBC1 (`src/chips/obc1.rs`)

The OBC1 (used by *Metal Combat: Falcon's Revenge*, LoROM chipset `$25`) provides 8 KiB of battery-backed SRAM at `$00–$3F` / `$80–$BF`, offsets `$6000–$7FFF`, with eight OAM-helper registers overlaid at `$7FF0–$7FF7`.

| Address | R/W | Function |
|---|---|---|
| `$7FF0` | R/W | Byte 0 (X low) of active sprite in OAM workspace (`base + index * 4 + 0`) |
| `$7FF1` | R/W | Byte 1 (Y) of active sprite (`base + index * 4 + 1`) |
| `$7FF2` | R/W | Byte 2 (tile number) of active sprite (`base + index * 4 + 2`) |
| `$7FF3` | R/W | Byte 3 (attributes) of active sprite (`base + index * 4 + 3`) |
| `$7FF4` | R/W | High OAM bits (`base + $200 + index / 4`). **Write** updates only the 2-bit pair for `index & 3` (`shift = (index & 3) * 2`, mask `3 << shift`). **Read** returns the entire underlying SRAM byte without isolating the 2 bits. |
| `$7FF5` | R/W | Workspace base select: bit 0 = `0` selects SRAM offset `$1C00` (`$7C00`), bit 0 = `1` selects SRAM offset `$1800` (`$7800`). |
| `$7FF6` | R/W | Active sprite index (`0..=127`, masked to `regs[6] & $7F`). |
| `$7FF7` | R/W | General latch register (read back as written). |

All 8 register bytes are stored in the `Coprocessor.Obc1` save-state group; the 8 KiB SRAM lives in `cart.sram` (space 6).

---

## 5. NEC uPD77C25 / uPD96050 Low-Level Engine (`src/chips/necdsp.rs`, `src/cart.rs`)

When the host supplies a firmware image (Core ABI file 2 combined, or file 2 program + file 3 data), `Machine::attach_dsp` instantiates `DspEngine::Lle(NecDsp)` executing the native 24-bit DSP instruction set.

### 5.1 Chip Variants, Firmware Sizes, and Clocks

| Property | uPD77C25 (DSP-1, DSP-1B, DSP-2, DSP-3, DSP-4) | uPD96050 (ST010, ST011) |
|---|---|---|
| Combined firmware size | **8,192 bytes** | **53,248 bytes** |
| Program ROM | `2,048 × 24-bit` (6,144 B; 11-bit `PC` mask `$07FF`) | `16,384 × 24-bit` (49,152 B; 14-bit `PC` mask `$3FFF`) |
| Data ROM | `1,024 × 16-bit` (2,048 B; 10-bit `RP` mask `$03FF`) | `2,048 × 16-bit` (4,096 B; 11-bit `RP` mask `$07FF`) |
| Data RAM | `256 × 16-bit` (512 B; 8-bit `DP` mask `$00FF`) | `2,048 × 16-bit` (4 KiB, battery-backed; 11-bit `DP` mask `$07FF`) |
| Call stack depth | 4 levels | 8 levels |
| Clock ratio (to master clock) | **7.60 MHz** (`760,000 / 2,147,727`) | **10.00 MHz** (`1,000,000 / 2,147,727`) |

- **Endian Auto-Detection**: `NecDsp::from_firmware` accepts both little-endian and legacy big-endian firmware dumps by inspecting the first four 3-byte instructions for the big-endian `JRQM` opcode prefix (`$97, $C0`).
- **Title-Based Firmware Naming (`Cartridge::nec_firmware`)**:
  - Chipset `$F6`: title containing `"F1 ROC"` or `"EXHAUST"` → `st010.rom` (53,248 B); otherwise → `st011.rom` (53,248 B).
  - Chipset `$03..=$05`: `"DUNGEON MASTER"` → `dsp2.rom`; `"GUNDAM"` → `dsp3.rom`; `"TOP GEAR 3000"` / `"TG3000"` / `"TG 3000"` → `dsp4.rom`; `"PILOTWINGS"` → `dsp1.rom`; all other DSP titles → `dsp1b.rom` (8,192 B).

### 5.2 Board Port Mappings (`Cartridge::dsp_map`, `Cartridge::dsp_port`)

| Board Variant (`DspMap`) | Selection Rule | `DR` (Data) | `SR` (Status) | On-Chip RAM |
|---|---|---|---|---|
| `LoRom30` | LoROM `<= 1 MiB`, no SRAM | `$30–$3F`, `$B0–$BF` : `$8000–$BFFF` | `$30–$3F`, `$B0–$BF` : `$C000–$FFFF` | — |
| `LoRom20` | LoROM `<= 1 MiB`, with SRAM | `$20–$3F`, `$A0–$BF` : `$8000–$BFFF` | `$20–$3F`, `$A0–$BF` : `$C000–$FFFF` | — |
| `LoRom60` | LoROM `> 1 MiB` | `$60–$6F`, `$E0–$EF` : `$0000–$3FFF` | `$60–$6F`, `$E0–$EF` : `$4000–$7FFF` | — |
| `HiRom` | HiROM / ExHiROM | `$00–$1F`, `$80–$9F` : `$6000–$6FFF` | `$00–$1F`, `$80–$9F` : `$7000–$7FFF` | — |
| `St` | uPD96050 / chipset `$F6` | `$60–$67`, `$E0–$E7` : even `$0000–$0FFE` | `$60–$67`, `$E0–$E7` : odd `$0001–$0FFF` | `$68–$6F`, `$E8–$EF` : `$0000–$0FFF` (4 KiB little-endian byte view) |

### 5.3 Host Handshake and Instruction Set Architecture

- **Status Register (`SR`) & `DR` Handshake**: S-CPU reads of `Port::Sr` return the high byte `(sr >> 8) as u8` (`RQM` = `$8000`, `DRS` = `$1000`, `DRC` = `$0400`).
  - When `DRC = 1` (8-bit mode): a single S-CPU read or write of `DR` transfers the low byte and clears `RQM`.
  - When `DRC = 0` (16-bit mode): the first S-CPU read/write transfers the low byte and sets `DRS`; the second transfers the high byte and clears both `DRS` and `RQM`.
  - On the DSP side, reading source `8` (`DR`) or writing destination `6` (`DR`) sets `RQM = 1`. Writing destination `7` (`SR`) preserves hardware-controlled `RQM` and `DRS`.
- **Registers**: 16-bit `A`, `B` (each with 6 flags: `S1`, `S0`, `C`, `Z`, `OV1`, `OV0`), `K`, `L` (combinatorial signed multiplier `(K * L) << 1` producing high word `M` and low word `N`), `TR`, `TRB`, `SR`, `DR`, `SO`, `SI`, plus pointers `PC`, `RP`, `DP`, and `SP`.
- **Reset**: Clears `PC`, `FA`, `FB`, and `SR`; sets `RP` to all ones (`$03FF` or `$07FF`); preserves `RAM` and all other registers.
- **24-Bit Instruction Format** (1 cycle per instruction):
  - `OP` (`bits 23..22 = 00`) and `RT` (`01`, pops `PC` from stack): Reads source `src`, executes 1 of 15 ALU operations (`OR, AND, XOR, SUB, ADD, SBB, ADC, DEC, INC, CMP, SHR1, SHL1, SHL2, SHL4, XCHG`) on `A` or `B` against `RAM[DP]`/`IDB`/`M`/`N`, updates flags (signed overflow sets `OV0`, copies `S0` to `S1`, and toggles `OV1`; non-arithmetic ops clear `OV0/OV1` and copy `S0` to `S1`), writes `IDB` to `dst` (destinations `11` and `12` perform parallel `K`/`L` loads from Data ROM or `RAM[DP | $40]`), modifies `DP` low nibble (`nop/+1/-1/0`) and XORs bits 7–4, and optionally decrements `RP`.
  - `JP` (`10`): Unconditional jump (`$100/$101`), call (`$140/$141`), uPD96050 indirect `JMP SO` (`$000`), and conditional jumps on `A`/`B` flags (`C, Z, OV0, OV1, S0, S1`), `DP` low nibble (`0` / `$F`), serial acknowledges (unwired on SNES: always ready), and `RQM` (`JRQM` / `JNRQM`). Conditional jumps on uPD96050 preserve `PC` bit 13.
  - `LD` (`11`): Loads 16-bit immediate `(op >> 6)` into destination `op & $F`.
- **Debugger & Save State**: LLE exposes `DSPRAM` (13), `DSPPRG` (14, 3 bytes per opcode), and processor 2 (`DSP`, 14 registers, `src/disasm/upd77c25.rs`). Save states store `DspEngine = 0` with `Coprocessor.NecDsp` (registers, stack, flags, cycle count, and full RAM).
