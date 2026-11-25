# Venus — SNES Hardware Reference

Core crate directory: `EmuSen/Cores/Nintendo/VenusRT - SNES` (crate `venusrt`).

This manual documents the Super Nintendo Entertainment System (SNES / Super Famicom) hardware architecture as implemented in `venusrt`: the Ricoh 5A22 (WDC 65C816 CPU, memory bus, DMA/HDMA, H/V IRQ, math unit, joypad controller), the S-PPU (PPU1/PPU2 video chips), the S-SMP and S-DSP audio subsystem, the Core ABI v1 state and debugger interfaces, and the core's verified accuracy boundaries. Cartridge coprocessors (SA-1, SuperFX/GSU, OBC1, and NEC DSP low-level emulation) are documented in `Venus_Coprocessors.md`, and the built-in high-level NEC DSP replacements are documented in `Venus_DspHle.md`.

---

## 1. Crate Architecture

| Module | Hardware / Subsystem Responsibility |
|---|---|
| `src/cpu/mod.rs`, `alu.rs`, `ops.rs`, `control.rs` | WDC 65C816 core: registers, addressing modes, ALU/BCD, control flow, hardware interrupts, bus cycle trait |
| `src/bus.rs` | Ricoh 5A22 system bus, master clock, scanline/frame scheduler, access speeds, open bus (MDR), WRAM port |
| `src/scpu.rs` | 5A22 internal peripherals: general DMA, HDMA, NMI and H/V IRQ latches, hardware multiply/divide, joypad ports |
| `src/cart.rs` | Copier header stripping, ROM header scoring and reset-handler verification, LoROM/HiROM/ExHiROM decode, SRAM, DSP port maps |
| `src/ppu.rs` | S-PPU1/S-PPU2 registers, VRAM/CGRAM/OAM ports, H/V counter latch, span-based scanline renderer, Modes 0–7, sprites, windows, color math, hi-res/interlace |
| `src/apu/spc700.rs`, `smp.rs`, `boot.rs`, `dsp.rs` | Sony SPC700 CPU, S-SMP memory/timers/ports, built-in 64-byte boot program, cycle-stepped 32-cycle S-DSP audio engine |
| `src/machine.rs`, `state.rs` | Top-level `Machine` coordinator, reset semantics, `VNRT` binary save-state codec |
| `src/v1.rs`, `spaces.rs`, `debugger.rs`, `probe.rs` | Core ABI v1 exports, memory spaces, multi-processor debugging, breakpoints, watches, coverage, cheats, patches |
| `src/disasm/` | Stateless disassemblers for 65C816 (`w65816.rs`), SPC700 (`spc700.rs`), GSU (`gsu.rs`), and uPD77C25/uPD96050 (`upd77c25.rs`) |
| `src/chips/` | Cartridge coprocessors (`sa1.rs`, `gsu.rs`, `obc1.rs`, `necdsp.rs`) and open DSP replacements (`dspengine.rs`, `dsphle.rs`, `dsp2.rs`, `dsp4.rs`, `st010.rs`) |

---

## 2. WDC 65C816 CPU (`src/cpu/`)

### 2.1 Registers and Operating Modes

The 65C816 core (`crate::cpu::Cpu`) drives one bus call per CPU cycle through the `crate::cpu::Bus` trait (`read`, `write`, `idle`, `implied_cycle`, `halted`), passing cycle pins (`VDA`, `VPA`, `VPB`, `MLB`, `E`, `M`, `X`).

| Register | Width | Description |
|---|---|---|
| `A` (C) | 16-bit | Accumulator (`A` = low byte, `B` = high byte). When `M = 1`, 8-bit operations modify `A` and preserve `B`. |
| `X`, `Y` | 16-bit | Index registers. When `X = 1` (or `E = 1`), high bytes are forced to `$00`. |
| `S` | 16-bit | Stack pointer. Forced to `$01xx` in emulation mode (`E = 1`). Power-on initializes `S = $0102` before the reset sequence performs three dummy stack reads, leaving `S = $01FF`. |
| `D` | 16-bit | Direct page base register (always in bank `$00`). Cleared to `$0000` on reset. |
| `PBR` | 8-bit | Program Bank Register (bits 23–16 of instruction fetches). Cleared on reset and interrupts. |
| `PC` | 16-bit | Program Counter (wraps within `PBR` bank). |
| `DBR` | 8-bit | Data Bank Register (bits 23–16 of absolute/indirect data accesses). Cleared on reset. |
| `P` | 8-bit | Processor status flags: `N` (`$80`), `V` (`$40`), `M` (`$20`), `X` (`$10`), `D` (`$08`), `I` (`$04`), `Z` (`$02`), `C` (`$01`). |
| `E` | 1-bit | Emulation mode flag (`true` on reset; toggled via `XCE`). Forces `M = 1`, `X = 1`, and stack page `$01xx`. |

### 2.2 Addressing Modes and Cycle Rules

`src/cpu/ops.rs` implements all 16 memory addressing modes cycle by cycle:

- **Direct Page (`dp`, `dp,X`, `dp,Y`, `(dp)`, `(dp,X)`, `(dp),Y`, `[dp]`, `[dp],Y`)**:
  - When `D & $00FF != 0`, every direct-page mode incurs 1 extra internal cycle.
  - In emulation mode with `DL == 0`, direct-page arithmetic and 16-bit direct-page pointer fetches wrap within the 256-byte page (`D & $FF00 | (addr & $00FF)`).
  - Direct-page multi-byte operand reads/writes wrap within bank `$00` (`Wrap::Bank0`), whereas absolute, long, and indirect-target accesses carry across banks (`Wrap::Linear`).
- **Absolute Indexed (`addr,X`, `addr,Y`)**: Adds 1 internal cycle (at the uncarried address) on a write, whenever `X = 0` (16-bit index), or when the low-byte addition crosses a 256-byte page.
- **Stack Operations in Emulation Mode**: Classic 6502 stack instructions keep `S` inside page 1 (`$0100–$01FF`), while 65C816-native stack instructions (`PEA`, `PEI`, `PER`, `PHD`, `PLD`, `PHK`, `PHB`, `PLB`, `JSL`, `RTL`) use 16-bit stack decrements/increments during execution and restore the high byte to `$01` afterward.
- **Implied One-Byte Instruction Quirk**: Single-byte implied instructions perform an internal second cycle at `PBR:PC+1` via `Bus::implied_cycle`. On the 5A22 bus (`src/bus.rs`), if that address falls in the B-bus region (`address & $40FF00 == $2100`), the read strobe still triggers the B-bus register's side effect (such as advancing the WRAM port `$2180`) in 6 master clocks while discarding the byte and leaving `MDR` unchanged.
- **Write Without `VDA | VPA`**: A bus write with neither `VDA` nor `VPA` asserted executes as a 6-master-clock internal cycle and leaves both memory and `MDR` untouched.

### 2.3 ALU, `BIT`, and Decimal Mode (`src/cpu/alu.rs`)

- **`BIT`**: Immediate `BIT #imm` updates only `Z` (`A & imm == 0`) and leaves `N` and `V` unchanged. All non-immediate `BIT` forms also copy the top two bits of the memory operand into `N` and `V` (`$80/$40` in 8-bit mode, `$8000/$4000` in 16-bit mode).
- **BCD Decimal Mode (`P.D = 1`)**: Both 8-bit (2-nibble) and 16-bit (4-nibble) `ADC` and `SBC` adjust each BCD digit sequentially. For `ADC`, `V` is computed against the unadjusted top-nibble sum before adding 6. For `SBC`, `V` is computed from the binary subtraction result, and per-nibble borrows subtract 6. Both update `N` and `Z` from the final decimal result.

### 2.4 Interrupts, Vectors, `WAI`, and `STP` (`src/cpu/control.rs`, `src/machine.rs`)

| Vector | Native Mode (`E = 0`) | Emulation Mode (`E = 1`) |
|---|---|---|
| `COP` | `$00FFE4` | `$00FFF4` |
| `BRK` | `$00FFE6` | `$00FFFE` |
| `ABORT` | `$00FFE8` | `$00FFF8` |
| `NMI` | `$00FFEA` | `$00FFFA` |
| `RESET` | `$00FFFC` | `$00FFFC` |
| `IRQ` | `$00FFEE` | `$00FFFE` |

- **Interrupt Sampling & `I`-Flag Delay**: The 5A22 samples `/NMI` (`nmi_at_cycle`) and `/IRQ` (`irq_at_cycle`) at the start of each bus cycle, so the check between instructions reflects the interrupt lines as seen just before the final cycle of the preceding instruction. When `CLI` (`$58`), `SEI` (`$78`), `PLP` (`$28`), `REP` (`$C2`), or `SEP` (`$E2`) changes `P.I`, the interrupt check immediately following that instruction uses the value of `P.I` from *before* the instruction executed.
- **Priority**: Pending edge-triggered NMI always takes precedence over level-triggered IRQ.
- **Hardware Interrupt Sequence**: Fetches and discards the opcode at `PC`, executes 1 internal cycle, pushes `PBR` (native mode only), `PCH`, `PCL`, and `P` (with bit 4 clear in emulation mode), sets `I = 1`, clears `D = 0` and `PBR = 0`, and loads `PC` from the vector with `VPB` asserted. `RESET` performs three dummy stack reads instead of writes and forces `E = 1`, `D = 0`, `DBR = 0`, `PBR = 0`, `M = 1`, `X = 1`, `I = 1`, `P.D = 0`.
- **`WAI` (`$CB`)**: Halts execution in 6-master-clock idle steps until `/NMI`, H/V IRQ, or cartridge IRQ asserts. Upon wakeup, `Machine::step` executes two 6-clock idle cycles and takes NMI (or IRQ if `I` is clear) immediately without waiting for another instruction.
- **`STP` (`$DB`)**: Halts the CPU indefinitely in 6-master-clock idle steps until hardware reset.

---

## 3. Master Clock and Frame Timeline (`src/bus.rs`, `src/v1.rs`)

### 3.1 NTSC and PAL Clocks

Region is derived from the cartridge header's country byte (`$02–$0C` or `$11` selects PAL; all others select NTSC).

| Parameter | NTSC (60 Hz) | PAL (50 Hz) |
|---|---|---|
| Master clock | `236,250,000 / 11` Hz (~21.477272 MHz) | `21,281,370` Hz (~21.281370 MHz) |
| Exact frame rate ratio (`v1.rs`) | `236,250,000 / (11 * 357,366)` (~60.0988 Hz) | `21,281,370 / 425,568` (~50.0070 Hz) |
| Standard scanline length | 1,364 master clocks (340 dots: 338×4 + 2×6) | 1,364 master clocks |
| Short scanline | Line 240, odd field (`field = true`), non-interlaced: **1,360 master clocks** | None (all lines 1,364) |
| Long dots (6 master clocks each) | Dots 323 (`HC 1292..1297`) and 327 (`HC 1310..1315`) on 1,364-clock lines | Same |
| Scanlines per frame | 262 (263 when `interlace && !field`) | 312 (313 when `interlace && !field`) |
| Nominal master clocks per frame | 357,366 average (`357,368` even / `357,364` odd) | 425,568 |

### 3.2 Scanline and Frame Events

The scheduler (`Timing::schedule` in `src/bus.rs`) computes `next_event` for each scanline so cycles that do not cross an event advance the clock directly without per-cycle overhead.

| Event | Line (`V`) | Master Clock (`HC`) | Notes |
|---|---|---|---|
| Power-on / Reset CPU start | `0` | `132` (`POWER_ON_AT`) | First 65C816 cycle begins 132 clocks into line 0; DMA 8-clock phase counts from line 0 clock 0. |
| HDMA initialization | `0` | `24` (`HDMA_INIT_AT`) | Reloads table addresses and first entries for all channels enabled in `$420C`. |
| Visible picture dots | `1..=224` | `88..1112` (dots `22..=277`) | `FIRST_DOT = 22`; `HBLANK` (`$4212` bit 6) is set when `HC >= 1096` (dot 274) or `HC < 4`. |
| DRAM refresh pause | Every line | `>= 534` (`REFRESH_AT`, H=133.5) | Pauses CPU/DMA for **40 master clocks** once per line at the first cycle boundary at or past `HC = 534`. |
| HDMA scanline trigger | `0..=224` | `>= 1097` (`HDMA_SEEN_FROM`) / `1100` (`HBLANK_AT`) | A CPU cycle whose middle (`line_clock + 3`) reaches `1100` with an active channel arms a two-cycle pipeline before running the line's HDMA transfers. Inside DMA, triggers at `HC >= 1112` (dot 278). |
| V-Blank start | `225` (`VBLANK_LINE`) | `0` | Sets `$4212` bit 7, clears active HDMA mask, reloads OAM address (unless in forced blank), and presents the finished frame. |
| NMI flag (`$4210` bit 7) | `225` | `6` (`NMI_AT`) | Sets `nmi_flag`; reading `$4210` before `HC = 10` leaves the flag set. |
| Auto-joypad start | `225` | `298` (`JOYPAD_AT`, H=74.5) | If `$4200` bit 0 is set, latches controller states into `$4218–$421F` and sets busy flag (`$4212` bit 0) for **4,224 master clocks**. |
| Frame wrap | `262`/`263` (NTSC), `312`/`313` (PAL) | `0` | Resets `line = 0`, toggles `field`, increments `frame`, clears `vblank`, `nmi_flag`, and `hdma_init_done`. PPU clears sprite overflow flags if not in forced blank. |

---

## 4. Memory Map and 5A22 Peripherals (`src/bus.rs`, `src/cart.rs`, `src/scpu.rs`)

### 4.1 CPU Address Space and Access Speeds

| Banks | Offset Range | Speed (Master Clocks) | Mapping |
|---|---|---|---|
| `$00–$3F`, `$80–$BF` | `$0000–$1FFF` | 8 | Low 8 KiB mirror of WRAM (`$7E0000–$7E1FFF`) |
| `$00–$3F`, `$80–$BF` | `$2000–$3FFF` | 6 | B-bus PPU (`$2100–$213F`), APU ports (`$2140–$217F`), WRAM port (`$2180–$2183`), coprocessor I/O/I-RAM |
| `$00–$3F`, `$80–$BF` | `$4000–$41FF` | 12 | Legacy joypad ports (`$4016–$4017`) |
| `$00–$3F`, `$80–$BF` | `$4200–$5FFF` | 6 | 5A22 internal I/O (`$4200–$421F`), DMA/HDMA (`$4300–$437F`) |
| `$00–$3F`, `$80–$BF` | `$6000–$7FFF` | 8 | Cartridge SRAM (HiROM banks `$20–$3F`, `$A0–$BF`), DSP/OBC1/GSU/SA-1 expansion |
| `$00–$3F` | `$8000–$FFFF` | 8 | SlowROM cartridge area |
| `$80–$BF` | `$8000–$FFFF` | 6 if `$420D` bit 0 set, else 8 | FastROM-capable cartridge area |
| `$40–$7D` | `$0000–$FFFF` | 8 | Cartridge ROM / SRAM (LoROM SRAM in `$70–$7D:$0000–$7FFF`) |
| `$7E–$7F` | `$0000–$FFFF` | 8 | 128 KiB Work RAM (WRAM) |
| `$C0–$FF` | `$0000–$FFFF` | 6 if `$420D` bit 0 set, else 8 | FastROM-capable cartridge area (LoROM SRAM in `$F0–$FF:$0000–$7FFF`) |

- **Open Bus (`MDR`)**: Every CPU read updates `System::mdr` with the bus value (unmapped addresses return `mdr`). CPU writes update `mdr` with the written value. Several status registers (`$4016–$4017`, `$4210–$4212`) merge open-bus bits from `mdr` into their unused bit positions; PPU registers use separate `ppu1_mdr` and `ppu2_mdr` latches.
- **WRAM Port (`$2180–$2183`)**: `$2181` (bits 0–7), `$2182` (bits 8–15), and `$2183` (bit 0 = bank bit 16) set the 17-bit `wram_address`. Reading or writing `$2180` accesses `wram[wram_address]` and post-increments modulo 128 KiB.

### 4.2 Cartridge Detection and Mapping (`src/cart.rs`)

- **Copier Header**: If `image.len() % 1024 == 512`, the leading 512-byte copier header is stripped. Images shorter than 32 KiB (`0x8000`) are rejected (`ImageTooShort` / ABI status `-9`).
- **Header Scoring**: Candidates are evaluated at `$007FC0` (LoROM), `$00FFC0` (HiROM), and `$40FFC0` (ExHiROM, only for images > 4 MiB) using map-mode agreement, checksum/complement sum (`complement ^ checksum == $FFFF`), reset vector `>= $8000` pointing to a plausible startup opcode, ASCII title bytes, and ROM/SRAM size bounds. If the top-scoring header's reset handler executes `<= 1` I/O writes within 4,000 instructions without crashing, `find_header` checks alternate candidates and prefers one that performs `>= 4` I/O writes without crashing (`Machine::reset_evidence`).
- **ROM Mirroring**: Non-power-of-two ROM sizes (such as 3 MiB) mirror recursively by splitting at the largest power of two (`Cartridge::mirror`): offsets in `[0, size)` map directly, while offsets past `size` wrap into the remainder block.
- **Standard Maps**:
  - **LoROM**: Offset `$8000–$FFFF` in all banks (except WRAM `$7E–$7F`) maps `((bank & $7F) << 15) | (offset & $7FFF)`. SRAM maps at `$70–$7D` and `$F0–$FF`, offsets `$0000–$7FFF`, masked to SRAM size.
  - **HiROM / ExHiROM**: Banks `$40–$7D` and `$C0–$FF` map linear 64 KiB banks `((bank & $3F) << 16) | offset`; banks `$00–$3F` and `$80–$BF` mirror the upper 32 KiB (`$8000–$FFFF`). ExHiROM adds `$400000` when `bank & $80 == 0`. SRAM maps at `$20–$3F` and `$A0–$BF`, offsets `$6000–$7FFF`.
- **SRAM Size and Battery**: When `chipset & $0F` indicates RAM (`1, 2, 4, 5, 9, A`), SRAM size is `1024 << sram_size` bytes for `1 <= sram_size <= 8`. Battery backing is indicated by `chipset & $0F` in `{2, 5, 6, 9, A}`.

### 4.3 5A22 Register Map (`$4016–$4017`, `$4200–$421F`, `$4300–$437F`)

| Address | R/W | Name | Function |
|---|---|---|---|
| `$4016` | R/W | `JOYSER0` | Write bit 0 = controller strobe (high reloads shift registers from current pads). Read returns `(mdr & $E0) | bit`, shifting port 0 MSB-first (fills with `1`s after 16 bits). |
| `$4017` | R | `JOYSER1` | Read returns `(mdr & $E0) | $1C | bit`, shifting port 1 MSB-first. |
| `$4200` | W | `NMITIMEN` | Bit 7: NMI enable; Bits 5–4: H/V IRQ mode (`0`=off, `1`=H, `2`=V, `3`=H+V); Bit 0: auto-joypad enable. Enabling NMI while `nmi_flag` is already set triggers an NMI edge delayed past the immediately following cycle start (`nmi_hold`). Switching to mode 2 on `line == vtime` at or past `irq_line_point(0)` immediately sets `irq_flag`. Clearing mode to 0 clears `irq_flag`. |
| `$4201` | W | `WRIO` | Programmable I/O port (initialized to `$FF` on power-on). A falling edge on bit 7 latches PPU H/V counters; bit 7 must be high for `$2137` reads to latch counters or `$213F` reads to clear the latch flag. |
| `$4202` | W | `WRMPYA` | 8-bit unsigned multiplicand A. |
| `$4203` | W | `WRMPYB` | 8-bit unsigned multiplicand B; starts 8-cycle multiplication (`RDMPY = 0`, `RDDIV = (B << 8) | A`). |
| `$4204–$4205` | W | `WRDIVL/H` | 16-bit unsigned dividend. |
| `$4206` | W | `WRDIVB` | 8-bit unsigned divisor; starts 16-cycle division (`RDMPY = dividend`, `shifter = divisor << 16`). |
| `$4207–$4208` | W | `HTIMEL/H` | 9-bit H-IRQ target dot (`0..=511`). |
| `$4209–$420A` | W | `VTIMEL/H` | 9-bit V-IRQ target scanline (`0..=511`). |
| `$420B` | W | `MDMAEN` | General DMA channel mask (bits 0–7). Non-zero write arms `dma_wait = 2` CPU cycles before pausing the CPU. |
| `$420C` | W | `HDMAEN` | HDMA channel enable mask (bits 0–7). |
| `$420D` | W | `MEMSEL` | Bit 0: FastROM enable (6 master clocks for banks `$80–$FF` ROM). |
| `$4210` | R | `RDNMI` | Bit 7: NMI flag; bits 6–4: `mdr`; bits 3–0: CPU version `$02`. Reading clears `nmi_flag` unless `line == 225 && line_clock < 10`. |
| `$4211` | R | `TIMEUP` | Bit 7: H/V IRQ flag; bits 6–0: `mdr`. Reading clears `irq_flag` unless within 4 master clocks of the comparator trigger point. |
| `$4212` | R | `HVBJOY` | Bit 7: V-Blank; Bit 6: H-Blank; Bits 5–1: `mdr`; Bit 0: auto-joypad busy. |
| `$4214–$4215` | R | `RDDIVL/H` | 16-bit division quotient (or residual shift register during multiplication). |
| `$4216–$4217` | R | `RDMPYL/H` | 16-bit multiplication product or division remainder. |
| `$4218–$421F` | R | `JOY1–4L/H` | Auto-read joypad results (`JOY1`/`JOY2` populated; `JOY3`/`JOY4` read 0). Serial bit order places `B` at bit 15 down to `R` at bit 4 (`B, Y, Select, Start, Up, Down, Left, Right, A, X, L, R`). |
| `$43x0–$43xB` | R/W | DMA ch `x` (`0..=7`) | `x0`=DMAP, `x1`=BBAD, `x2–x4`=A1T/A1B, `x5–x6`=DAS (count / indirect addr), `x7`=DASB (indirect bank), `x8–x9`=A2A (HDMA table ptr), `xA`=NTRL (HDMA line counter), `xB`=unused RAM byte. |

### 4.4 Hardware Multiply / Divide Unit (`src/scpu.rs`)

The math unit advances 1 step per CPU cycle (`System::math_tick`), skipping the cycle of the write that started it (`math_fresh`):
- **Multiply (8 CPU cycles)**: On each step, if `RDDIV & 1 != 0`, adds `shifter` to `RDMPY`, then shifts `shifter <<= 1` and `RDDIV >>= 1`.
- **Divide (16 CPU cycles)**: On each step, shifts `shifter >>= 1`; if `RDMPY >= shifter`, subtracts `shifter` from `RDMPY` and shifts `1` into `RDDIV`, else shifts `0`. Division by zero completes naturally with `RDDIV = $FFFF` and `RDMPY = dividend`.

### 4.5 H/V IRQ Comparator (`src/bus.rs`)

- For horizontal target `h` (`0` in V-only mode 2, else `htime`), the comparator firing point is:
  - `irq_point(0) = 10`
  - `irq_point(h) = 14 + 4*h + 2*(h > 323) + 2*(h > 327)`
  - Effective flag/line point in end-of-cycle coordinates: `irq_line_point(h) = irq_point(h) + 4`.
- Values `h > 339` never fire. `h == 153` does not fire on the 1,360-clock short line or on the last line of a frame.
- Points `>= line_length` wrap into the beginning of the next scanline (matching the previous line's `V`), except across frame wrap (`line == 0`) or after the short line (`line == 241` on odd non-interlaced NTSC fields).

### 4.6 General DMA and HDMA (`src/scpu.rs`)

- **Transfer Unit Modes (`DMAP` bits 2–0)**:
  - `0`: `[0]` (1 byte)
  - `1`: `[0, 1]` (2 bytes)
  - `2`, `6`: `[0, 0]` (2 bytes)
  - `3`, `7`: `[0, 0, 1, 1]` (4 bytes)
  - `4`: `[0, 1, 2, 3]` (4 bytes)
  - `5`: `[0, 1, 0, 1]` (4 bytes)
- **A-Bus Restrictions**: DMA blocks A-bus addresses in `$00–$3F`/`$80–$BF` at `$2100–$21FF`, `$4300–$437F`, `$420B`, and `$420C` (reads open bus, drops writes). Because WRAM cannot simultaneously serve both A-bus and B-bus, any transfer between WRAM on the A-bus and `$2180` (`WMDATA`) on the B-bus is suppressed (writes to WRAM from `$2180` store `$00`; writes to `$2180` from WRAM do nothing and leave `wram_address` unchanged).
- **Clock Alignment**: Both DMA and HDMA pause alignment pads `to8 = 8 - ((clock + 132) % 8)` master clocks to align to the 8-clock DMA phase from line 0, and `align_after` pads to the next whole multiple of the resuming CPU cycle length (`6`, `8`, or `12`).
- **General DMA**: Overhead is 8 clocks plus 8 clocks per active channel plus 8 clocks per byte transferred (`count == 0` transfers 65,536 bytes). Address/count registers update live on every byte so an interrupting HDMA on the same channel sees current progress and cancels the remaining general DMA on that channel. DRAM refresh (`40` clocks) fires inline if crossed during DMA.
- **HDMA Init (`V = 0`, `HC = 24`)**: Always sets `hdma_active = $FF`. If `$420C` is non-zero, sets `hdma_transfer = $FF`, cancels overlapping general DMA channels, aligns, incurs 8 clocks overhead (`HDMA_OVERHEAD`), reloads `A2A` from `A1T`, reads `NTRL` (8 clocks) plus indirect address bytes if bit 6 of `DMAP` is set (16 clocks, or 8 clocks if `NTRL == 0`, loading the high byte and clearing the low byte).
- **HDMA Scanline (`V = 0..=224`)**: Runs in two phases after 8 clocks overhead:
  1. **Transfer phase** (channels 0–7): For each active channel with `hdma_transfer` set, transfers one mode unit at 8 clocks per byte (byte lands at step end), advancing `DAS` (indirect) or `A2A` (direct). Cancels overlapping general DMA.
  2. **Counter phase** (channels 0–7): 8 clocks per active channel; decrements the full 8-bit `NTRL` byte (`$00` wraps to `$FF`, giving 127 repeat lines; `$80` wraps to `$7F`, giving 127 non-repeat lines), sets `hdma_transfer` from bit 7, and if `NTRL & $7F == 0` reloads the next table entry (deactivating the channel on `NTRL == 0`).

---

## 5. S-PPU Video Subsystem (`src/ppu.rs`)

### 5.1 Memories and Latches

| Memory | Size | Access Mechanism |
|---|---|---|
| VRAM | 64 KiB (`32,768 × u16`) | Word-addressed via `VMADDL/H` (`$2116–$2117`), `VMAIN` (`$2115`), `VMDATAL/H` (`$2118–$2119`), `RDVRAML/H` (`$2139–$213A`). |
| CGRAM | 512 B (`256 × u16`, 15-bit BGR555) | Byte-paired via `CGADD` (`$2121`), `CGDATA` (`$2122`), `RDCGRAM` (`$213B`). High byte masks to 7 bits (`$7F`). |
| OAM | 544 B (`$000–$1FF` low table + `$200–$21F` high table) | 10-bit byte address via `OAMADDL/H` (`$2102–$2103`), `OAMDATA` (`$2104`), `RDOAM` (`$2138`). |

- **PPU Open-Bus Latches**: PPU1 (`ppu1_mdr`) updates on reads of `$2134–$2136`, `$2138–$213A`, and `$213E`; write-only addresses `$2104–$2106`, `$2108–$210A`, `$2114–$2116`, `$2118–$211A`, `$2124–$2126`, and `$2128–$212A` return `ppu1_mdr`. PPU2 (`ppu2_mdr`) updates on reads of `$213B–$213D` and `$213F` and supplies unused upper bits on `$213B` (bit 7), `$213C/$213D` (bits 7–1 on second read), and `$213F` (bit 5).
- **VRAM Port Rules**:
  - `VMAIN` (`$2115`) bits 3–2 select address remapping: `0` = none; `1` = 8-bit rotate left by 3 (`aaaa aaaa xxx yyyyy` → `aaaa aaaa yyyyy xxx`); `2` = 9-bit rotate left by 3; `3` = 10-bit rotate left by 3.
  - Bits 1–0 select word increment (`1`, `32`, `128`, `128`); bit 7 chooses whether increment triggers on high-byte (`1`) or low-byte (`0`) access.
  - Writes to `$2116/$2117` immediately prefetch `vram[vram_index()]` into `vram_buffer`. Reads of `$2139/$213A` return `vram_buffer`; when the stepping byte is read, `vram_buffer` reloads from the *old* address before stepping.
  - Writes to `$2118/$2119` modify VRAM only during V-Blank or forced blank (`INIDISP` bit 7); `vram_address` still increments regardless.
- **OAM Port Rules**:
  - Writes to `$2102/$2103` update the 9-bit `oam_reload` word address (and bit 7 of `$2103` sets `oam_rotation`) and immediately set `oam_address = oam_reload << 1`.
  - Writing `$2104` to the low table (`a < $200`) latches even bytes in `oam_low` and commits the 16-bit word on odd bytes; writes to `$200–$3FF` write immediately to `oam[$200 + (a & $1F)]`.
  - OAM address reloads from `oam_reload << 1` at the start of line 225 (if not in forced blank) and also when exiting forced blank on line 225.
- **Scroll and Mode 7 Write-Twice Latches**:
  - `BGnHOFS` (`$210D, $210F, $2111, $2113`): `hofs[bg] = (val << 8) | (bg_old & !7) | ((hofs[bg] >> 8) & 7)`.
  - `BGnVOFS` (`$210E, $2110, $2112, $2114`): `vofs[bg] = (val << 8) | bg_old`.
  - Mode 7 matrix/center/scroll (`$211B–$2120` and shared `$210D–$210E`): `m7[k] = (val << 8) | m7_old`, updating `m7_old = val`. Note that `$210D` and `$210E` update *both* Mode 7 scroll (`m7[6..7]` via `m7_old`) and BG1 scroll (`hofs[0]`/`vofs[0]` via `bg_old`).
- **H/V Counter Latch (`$2137`, `$213C`, `$213D`, `$213F`)**: Reading `$2137` (returns CPU open bus) when `WRIO` bit 7 is set latches `ophct = dot` and `opvct = line` (evaluated 4 master clocks earlier on `$2137` reads) and sets `latched`. Falling `WRIO` bit 7 also latches. Reading `$213F` (`STAT78`) returns `(field << 7) | (latched << 6) | (ppu2_mdr & $20) | (pal << 4) | $03` and resets `oph_second`/`opv_second` (and clears `latched` only if `WRIO` bit 7 is high). `$213E` (`STAT77`) returns `(time_over << 7) | (range_over << 6) | (ppu1_mdr & $10) | $01`.
- **Mode 7 Signed Multiplier (`$2134–$2136`)**: Instantaneous 24-bit signed product `(m7[0] as i16 as i32) * ((m7[1] >> 8) as i8 as i32)` (`M7A × M7B_hi`).

### 5.2 Background Modes and Priority Charts

| Mode | BG1 | BG2 | BG3 | BG4 | Special Features |
|---|---|---|---|---|---|
| 0 | 2bpp | 2bpp | 2bpp | 2bpp | Separate 32-color CGRAM regions per BG (`bg * $20 + pal*4 + c`) |
| 1 | 4bpp | 4bpp | 2bpp | — | `BGMODE` bit 3 elevates BG3 priority 1 above all other layers |
| 2 | 4bpp | 4bpp | OPT | — | Offset-per-tile (BG3 supplies H and V scroll per column) |
| 3 | 8bpp | 4bpp | — | — | BG1 supports Direct Color (`CGWSEL` bit 0) |
| 4 | 8bpp | 2bpp | OPT | — | BG1 supports Direct Color; BG3 supplies 1 OPT entry per column (bit 15 selects V vs H) |
| 5 | 4bpp | 2bpp | — | — | True 512px hi-res (16 half-pixel tiles; even half-pixels on sub, odd on main), supports interlace half-lines |
| 6 | 4bpp | — | OPT | — | True 512px hi-res + offset-per-tile, supports interlace half-lines |
| 7 | 8bpp | 7bpp* | — | — | Affine matrix transform over 128×128 tile (1024×1024 px) field; `SETINI` bit 6 enables EXTBG (BG2 uses bit 7 as priority, bits 6–0 as color) |

**Priority Orders (front to back)**:
- **Mode 0**: `OBJ3, BG1.1, BG2.1, OBJ2, BG1.0, BG2.0, OBJ1, BG3.1, BG4.1, OBJ0, BG3.0, BG4.0`
- **Mode 1 (`BG3 high = 0`)**: `OBJ3, BG1.1, BG2.1, OBJ2, BG1.0, BG2.0, OBJ1, BG3.1, OBJ0, BG3.0`
- **Mode 1 (`BG3 high = 1`)**: `BG3.1, OBJ3, BG1.1, BG2.1, OBJ2, BG1.0, BG2.0, OBJ1, OBJ0, BG3.0`
- **Modes 2, 3, 4, 5**: `OBJ3, BG1.1, OBJ2, BG2.1, OBJ1, BG1.0, OBJ0, BG2.0`
- **Mode 6**: `OBJ3, BG1.1, OBJ2, OBJ1, BG1.0, OBJ0`
- **Mode 7**: `OBJ3, OBJ2, OBJ1, BG1.0, OBJ0`
- **Mode 7 + EXTBG**: `OBJ3, OBJ2, BG2.1, OBJ1, BG1.0, OBJ0, BG2.0`

**Direct Color Mode**: On 8bpp layers (Mode 3 BG1, Mode 4 BG1, Mode 7 BG1) when `CGWSEL` bit 0 is set, an 8-bit pixel `BBGGGRRR` plus the 3-bit map palette `bgr` produces 15-bit RGB555: `R = RRRr0`, `G = GGGg0`, `B = BBb00` (Mode 7 has `bgr = 0`).

**Offset-Per-Tile (Modes 2, 4, 6)**: For visible tile column `T = (x + (hofs & 7)) >> 3`, column `T == 0` uses normal scroll registers; `T >= 1` fetches BG3's map entry at `((T - 1) << 3) + (hofs[2] & !7)`. Valid bit is `$2000 << bg`. In Modes 2 and 6, row `vofs[2]` provides H-offset (`h & $03F8 | (hofs & 7)`) and `vofs[2] + 8` provides V-offset (`v & $03FF`); in Mode 4, a single entry is read and bit 15 selects V (`1`) vs H (`0`).

**Mode 7 Transform**: Uses 13-bit signed center `(CX, CY)` and scroll `(HOFS, VOFS)` with `CLIP(v) = (v & $2000 != 0) ? (v | !$3FF) : (v & $3FF)` on `scroll - center`. Origin and Y products are truncated to 1/4-pixel precision (`& !63`) before adding `a * x` / `c * x` and shifting right by 8. `M7SEL` bits 0–1 flip screen X/Y; bits 7–6 choose outside-field behavior (`0x` = wrap, `10` = transparent, `11` = tile 0).

### 5.3 Sprites (OBJ)

Sprites are evaluated at the end of scanline `line - 1` for scanline `line` (`1..=224`):
- **Size Table (`OBSEL` bits 7–5)**: `0`=(8×8, 16×16), `1`=(8×8, 32×32), `2`=(8×8, 64×64), `3`=(16×16, 32×32), `4`=(16×16, 64×64), `5`=(32×32, 64×64), `6`=(16×32, 32×64), `7`=(16×32, 32×32).
- **Priority Rotation**: When `oam_rotation` is set (`$2103` bit 7), evaluation begins at sprite `(oam_address >> 2) & $7F`; otherwise sprite 0.
- **Range and Time Limits**: Up to **32 sprites** in vertical/horizontal range are collected (exceeding sets `range_over`, `$213E` bit 6). Tiles are then fetched in **reverse order** from the last collected sprite back to the first, up to **34 8×8 tiles** (exceeding sets `time_over`, `$213E` bit 7). Reverse loading ensures lower-index sprites overwrite higher-index ones into `obj_line`.
- **Settled Hardware Quirks**:
  - A sprite with signed 9-bit `X == -256` (`$100`) is treated as `X = 0` for range and time-overflow checks while still drawing off-screen.
  - Vertically flipped rectangular sprites (modes 6–7) flip as two stacked square blocks of side `width` rather than across full height.
  - OBJ interlace (`SETINI` bit 1) halves vertical display height (`h >> 1`) and samples row `2 * row + field`.
  - Tile numbers wrap independently within the 16×16 tile table in X and Y (`((c >> 4) + row/8) & $F` and `((c & $F) + col) & $F`).

### 5.4 Mosaic, Windows, Color Math, Hi-Res, and Rendering

- **Mosaic (`$2106`)**: Vertical block counter `mosaic_row` resets at line 1 and whenever `mosaic_row >= mosaic_size`, reloading `mosaic_size = (MOSAIC >> 4) + 1` only at block boundaries. In Mode 7 with EXTBG, BG2 uses BG1's enable bit for vertical mosaic and its own bit for horizontal mosaic. In true hi-res (Modes 5/6), a mosaic block's first sub-screen half-pixel fills both main and sub buffers; interlaced half-lines always repeat the even half-line when mosaic is enabled.
- **Windows (`$2123–$212F`)**: Two horizontal ranges `W1` (`$2126–$2127`) and `W2` (`$2128–$2129`, inclusive `[L..=R]`), per-layer enable/invert (`W12SEL`, `W34SEL`, `WOBJSEL`) and combination logic (`WBGLOG`, `WOBJLOG`: `0`=OR, `1`=AND, `2`=XOR, `3`=XNOR), gated on main/sub screens by `TMW`/`TSW`. The renderer caches the 6-bit window mask across runs between window edges (`next_window_edge`).
- **Color Math (`$2130–$2132`)**:
  - `CGWSEL` bits 7–6 select clip-to-black region and bits 5–4 select math-prevent region (`0`=never, `1`=outside color window, `2`=inside, `3`=always).
  - `CGADSUB` bit 7 selects subtract vs add; bit 6 halves the result (suppressed when main was clipped to black or when sub-screen mode fell through to the fixed color backdrop); bits 5–0 enable math per main layer (backdrop, OBJ, BG4–BG1). Only OBJ palettes 4–7 (`CGRAM >= $C0`) participate in color math.
  - `COLDATA` (`$2132`) updates R (bit 5), G (bit 6), and/or B (bit 7) 5-bit components of the fixed color.
- **Hi-Res and Interlace (`SETINI` `$2133`)**:
  - Modes 5/6 and pseudo-hi-res (`SETINI` bit 3) output 512 horizontal pixels per line, placing each sub-screen half-pixel to the left of the main-screen half-pixel. The sub half-pixel uses the backdrop (`CGRAM[0]`) when transparent and inherits the clip and color-math operation of the *preceding* main pixel (`before`).
  - A frame renders into a 256-wide buffer (`narrow`) until any non-blank hi-res span occurs, at which point `widen()` doubles prior lines into the 512-wide `canvas`.
  - Interlaced hi-res (Modes 5/6 with `SETINI` bit 0) weaves even and odd fields into a 512×448 output buffer (`frame_width = 512`, `frame_height = 448`); non-interlaced frames present as 256×224 or 512×224 RGBA8888 (5-bit RGB scaled by `(brightness + 1) / 16` then expanded `(c << 3) | (c >> 2)`).
- **Mid-Line Spans and Skip-Rendering**: Every PPU register write calls `draw_to(beam.line, pixel_of(beam.dot))` to render the span passed under the previous register values. Setting `Ppu::skip` (via Core ABI `set_options` bit 0) omits pixel decoding/output while still running register updates, counter latches, and sprite overflow evaluation, guaranteeing `skip_rendering_state_neutral = true`.

---

## 6. S-SMP / SPC700 and S-DSP Audio Subsystem (`src/apu/`)

### 6.1 S-SMP Architecture, Clocks, and Boot Program (`src/apu/smp.rs`, `boot.rs`, `spc700.rs`)

- **Clock Ratios**: The S-SMP/S-DSP operates at a nominal 1.024 MHz DSP cycle rate (32 cycles per 32 kHz audio sample), tracked against master clocks using exact integer ratios:
  - NTSC: `5,632 / 118,125`
  - PAL: `102,400 / 2,128,137`
- **Startup Alignment**: At power-on/reset, the SPC700 leads the 65C816 by `150` master clocks (`LEAD`) and executes `8` reset cycles (`RESET_CYCLES`) with `TEST = $0A`, `CONTROL = $B0`, `DSP.step = 1`, `DSP.every_other = true`, and `AUXIO4/5 = $FF`. This places the S-DSP's `KON` poll (cycle 30 of every other sample) at 62 and 126 cycles after each Timer 0/1 tick.
- **Boot ROM (`$FFC0–$FFFF`)**: When `CONTROL` bit 7 is set, reads from `$FFC0–$FFFF` return the 64-byte boot program while writes still pass through to the underlying 64 KiB APU RAM (when `TEST` bit 1 is set).
  - By default, `venusrt` runs its own 64-byte clean-room IPL uploader (`src/apu/boot.rs`, starting with `MOV X,#$EF`, `SP=$EF`, clearing `$01–$EF`, signaling `$BBAA` on ports 0–1, and implementing the `$CC` block-upload and jump handshake).
  - If the host provides a 64-byte `spc700.rom` (firmware file 1), `Machine::attach_boot` maps it in place of the built-in program and preserves it across resets.
- **Quarter-Cycle Port Synchronization (`$2140–$2143` ↔ `$F4–$F7`)**: The APU runs lazily behind the S-CPU and catches up on port accesses and at frame end. `Smp::run_to` steps instructions only while all cycles of an instruction start before the target master clock; writes from both CPUs are queued with quarter-DSP-cycle timestamps (`to_apu_q`, `to_cpu_q`, capacity 8 each). S-CPU writes land at the end of the S-CPU bus cycle; SPC700 writes land in the fourth quarter of the SPC700 cycle (`4 * cycles - 1`); SPC700 reads latch in the first quarter (`4 * (cycles - stretch)`). When the S-CPU reads `$2140–$2143` mid-instruction, `Smp::peek` side-effect-freely inspects the pending SPC700 instruction so already-written port bytes are visible immediately.
- **`TEST` (`$F0`) and `CONTROL` (`$F1`)**:
  - `TEST` bits 7–6 stretch each SPC700 cycle to `1, 2, 5, or 10` DSP cycles; bits 5–4 and 7–6 set the timer prescaler step `(1 << clk) + (2 << ((test >> 4) & 3))`.
  - `TEST` bit 1 enables APU RAM writes (and S-DSP echo RAM writes); bit 3 enables timers and bit 0 halts timers. Changing `TEST` evaluates falling edges of the gated first-stage prescaler levels (`prescale[0] >= 192` out of 384 for Timers 0/1, `prescale[1] >= 24` out of 48 for Timer 2), reproducing the hardware glitch where stopping a timer while its first stage is high clocks its divider once.
  - `CONTROL` bits 2–0 enable Timers 2–0 (a `0 → 1` rising edge resets `stage = 0` and `out = 0`; clearing to `0` stops the timer while preserving `out`). Bit 4 clears S-CPU→APU ports 0–1; bit 5 clears ports 2–3; bit 7 maps the boot ROM at `$FFC0–$FFFF`.
- **Timers (`$FA–$FF`)**: Timers 0 and 1 tick at 8 kHz (every 128 nominal cycles via prescaler modulo 384, step 3); Timer 2 ticks at 64 kHz (every 16 nominal cycles via prescaler modulo 48, step 3). Each divides by `TnDIV` (`0` = 256) and increments a 4-bit counter `TnOUT` (`$FD–$FF`), which resets to 0 on read.
- **Auxiliary Latches (`$F8–$F9`)**: `AUXIO4` and `AUXIO5` return their latched register values rather than underlying RAM.
- **SPC700 Core (`src/apu/spc700.rs`)**: Full 8-bit CPU (`A`, `X`, `Y`, `SP`, `PC`, `PSW` with `N V P B H I Z C`), direct-page `$00xx`/`$01xx` wrapping on 16-bit accesses, cycle-accurate bus ordering, and permanent halt on `SLEEP`/`STOP` (the SNES APU has no interrupt sources).

### 6.2 S-DSP 32-Step Audio Engine (`src/apu/dsp.rs`)

One S-DSP step executes on every 1.024 MHz DSP cycle; 32 steps (`step = 0..=31`) produce one 32 kHz stereo PCM pair. Registers `$00–$7F` are accessed via `$F2` (`DSPADDR`) and `$F3` (`DSPDATA`); addresses `$80–$FF` mirror `$00–$7F` as read-only.

- **Voice Pipeline (`S1`–`S9`)**: All 8 voices interleave across steps 0–31:
  - `S1`: Latch `SRCN`.
  - `S2`: Read BRR start/loop pointer from `DIR << 8 + SRCN * 4` (`+0` during `kon_delay == 5`, else `+2`), latch `PITCHL` and `ADSR1`.
  - `S3a`: Latch `PITCHH` (`& $3F`) and apply pitch modulation (`PMON`, voices 1–7) using `(last_out >> 4) * pitch >> 10`.
  - `S3b`: Read BRR block header and first data byte.
  - `S3c`: Advance `kon_delay`, interpolate or sample 15-bit LFSR noise (`NON`), scale by 11-bit `env` (`(sample * env) >> 11`), record `last_out` and `envx`, handle BRR end-without-loop (`header & 3 == 1` → Release, `env = 0`), handle `FLG` reset / `KOFF` / `KON` (acted on every other sample when `every_other` is set; `KON` sets `kon_delay = 5`, Attack mode, `env = 0`), and advance the envelope once `kon_delay` finishes.
  - `S4`: Mix left output `(out * VOLL) >> 6` into `main[0]` (and `echo[0]` if `EON`), decode next 4 BRR samples if `prev_delay` is 2–4 or `pos >= $4000`, and advance `pos += pitch` clamped to `$7FFF`.
  - `S5`: Mix right output into `main[1]` / `echo[1]` and prepare `ENDX` buffer (setting bit on loop, clearing on key-on).
  - `S6–S9`: Latch and commit `OUTX` (`out >> 7`), `ENDX`, and `ENVX` (`env >> 4`). Writing `ENVX` or `OUTX` from the S-SMP during the 2-cycle window between preparation (`S7`/`S6`) and commit (`S9`/`S8`) overrides the pending buffer.
- **BRR Decoding**: Each 9-byte block has 1 header byte (`shift = h >> 4`, `filter = (h >> 2) & 3`, `loop = h & 2`, `end = h & 1`) and 8 bytes (16 signed 4-bit nibbles), decoded 4 samples at a time into a 12-sample ring buffer. Shifts `0..=12` compute `(n << shift) >> 1`; shifts `13..=15` yield `-2048` for negative nibbles and `0` for non-negative. The four IIR filters (0: none; 1: `15/16 p1`; 2: `61/32 p1 - 15/16 p2`; 3: `115/64 p1 - 13/16 p2`) use exact bit-shift arithmetic, clamp to 16 bits, and wrap/clip to 15 bits (`clip15`).
- **Gaussian Interpolation**: Uses the 512-entry `GAUSS` table indexed by the 8-bit fraction `(pos >> 4) & $FF` over four consecutive ring-buffer samples. The first three products (`(GAUSS * s) >> 10`) sum with 16-bit wrapping, the fourth is added with 16-bit saturation, and the result is arithmetically shifted right by 1 (15 bits).
- **Envelopes (ADSR / GAIN)**: Driven by a shared global counter cycling `0x77FF` down to `0` at step 29, with 32 rate periods (`0, 2048, 1536, ..., 1`) and phase offsets (`0, 1040, 536`).
  - Release mode decrements `env` by 8 every sample to 0.
  - ADSR (`ADSR1` bit 7 = 1): Attack adds `+32` at rate `2*AR + 1` (or `+1024` at rate 31 when `AR == 15`), transitioning to Decay when out of `0..=$7FF`; Decay applies exponential step `e - (((e - 1) >> 8) + 1)` at rate `2*DR + 16`, transitioning to Sustain when `(new >> 8) & 7 == SL`; Sustain continues exponential decay at rate `SR`.
  - GAIN (`ADSR1` bit 7 = 0): Direct (`bit 7 = 0`) loads `(v & $7F) << 4` at rate 31; indirect modes (`bits 6–5`): `0`=linear decrease (`-32`), `1`=exponential decrease, `2`=linear increase (`+32`), `3`=bent increase (`+32` while `hidden_env < $600`, else `+8`). Phase transitions and `hidden_env` track the candidate `new` value even on samples where the rate counter does not fire.
- **Echo and 8-Tap FIR**:
  - Steps 22–23 read stereo 16-bit echo samples from `(t_esa << 8) + echo_offset`, shifting right by 1 into the 8-sample FIR ring buffer (`fir`), while steps 22–25 latch `C0–C7` (`$0F–$7F`).
  - `fir()` sums taps 0–6 with 16-bit wrapping (`(s * coeff) >> 6`), adds tap 7 with 16-bit saturation, and clears bit 0 (`& !1`).
  - Step 29 latches `echo_length = (EDL & $0F) << 11` when `echo_offset == 0`, latches `t_esa`, and writes left echo (if `!(FLG & $20)` and `TEST` bit 1 is set); step 30 writes right echo and advances `echo_offset` by 4 (wrapping at `echo_length`).
- **Cycle-26/27 Stereo Latch**: Step 26 computes the left output sample `(main * MVOLL >> 7) + (fir * EVOLL >> 7)` (muted to 0 if `FLG & $40`), stores it in `Dsp::left`, and latches `t_efb`. Step 27 computes the right sample and emits `[self.left, right]` together into `Smp::out`. Because `Dsp::left` is serialized in save states, frames or save states ending between steps 26 and 27 never drop a sample or invert left/right channels.

---

## 7. Core ABI v1, Save States, and Debugger (`src/v1.rs`, `src/state.rs`, `src/debugger.rs`, `src/spaces.rs`)

### 7.1 Core ABI v1 Capabilities and Descriptors

- **Core ID**: `venusrt` (system `snes`, extensions `.smc`, `.sfc`, regions `Ntsc` / `Pal`, deterministic `true`).
- **Capabilities**: `RESET | SNAPSHOT | BATTERY_DIRTY | ROM_PATCHES | CHEAT_POKES | SETTINGS | DEBUG | DEBUG_STACK | DEBUG_REGISTERS | DEBUG_DISASSEMBLE`.
- **Video / Audio**: RGBA8888, base `256×224`, max `512×448`, aspect `4:3`; audio `32,000` Hz stereo (`i16`).
- **Controllers**: 2 ports (`snes.pad`), 12 buttons (`B, Y, Select, Start, Up, Down, Left, Right, A, X, L, R` on bits 0–11).
- **Firmware Slots** (all optional):
  - File `0`: Battery RAM (`.srm`; maps to cartridge SRAM, or ST010/ST011 4 KiB on-chip RAM).
  - File `1`: `spc700.rom` (64 bytes, replaces built-in SPC700 boot program).
  - File `2` (+ optional File `3`): NEC DSP firmware (`8,192` B or `53,248` B combined, or split `.program.rom` + `.data.rom`).
- **Settings**:
  - `sa1_catch_up`: `"1"` (default, exact after every S-CPU instruction), `"4"`, or `"16"` (deferred catch-up when the S-CPU is not touching the SA-1 board; always exact on board accesses, frame end, and inside debugger frames).
- **ROM Patches & Cheat Pokes**: ROM patches (`set_rom_patches`) intercept cartridge reads (`address >= $6000` outside WRAM) with optional compare byte. Cheat pokes (`set_cheat_pokes`) target `CpuBus` (space 0) and `WRAM` (space 2) only, applied at frame end.
- **Hardware Reset (`Machine::reset`)**: Resets S-CPU, S-PPU, S-SMP, and cartridge coprocessors to power-on state (`H=0, V=0`, CPU at reset vector) while preserving WRAM, VRAM, CGRAM, OAM, APU RAM, cartridge SRAM, SA-1 I-RAM, NEC DSP firmware, and `frame` count.

### 7.2 Save-State Format (`VNRT`)

- **Magic**: `"VNRT"` (`STATE_MAGIC`). Legacy states starting with `"SNES"` (`0x53454E53`) are explicitly rejected as foreign with a descriptive message.
- **Version**: Current `STATE_VERSION = 22`; loads versions `17..=22` (version 20 added HDMA stage fields; version 21 added DSP-2/ST010/attitude HLE state; version 22 added DSP-1 projection HLE state).
- **NEC DSP Engine Isolation**: A `Coprocessor` group for a NEC DSP records `DspEngine` tag `0` (LLE) or `1` (HLE). Loading a state saved under the opposite engine returns `STATUS_OTHER_DSP_ENGINE` (`-11`) without modifying the machine.
- **Snapshots**: `save_snapshot` writes the full `VNRT` state (`snapshot_size == state_size`).

### 7.3 Memory Spaces and Multi-Processor Debugger

| Space ID | Name | Size | Read-Only | Cheats | Write Watch | Notes |
|---|---|---|---|---|---|---|
| `0` | `CpuBus` | `16 MiB` | No | Yes | Yes | Side-effect-free peek; writes route through `System::poke` (memory only, no register side effects) |
| `1` | `IO` | `1 KiB` | Yes | No | No | Last-written `$4000–$43FF` shadow |
| `2` | `WRAM` | `128 KiB` | No | Yes | Yes | Work RAM |
| `3` | `VRAM` | `64 KiB` | No | No | No | Little-endian byte view of `32,768 × u16` |
| `4` | `CGRAM` | `512 B` | No | No | No | Little-endian byte view of `256 × u16` |
| `5` | `OAM` | `544 B` | No | No | No | Low + high OAM tables |
| `6` | `SRAM` | Dynamic | No | No | Yes | Cartridge SRAM, or ST010/ST011 4 KiB on-chip RAM |
| `7` | `APURAM` | `64 KiB` | No | No | Yes | S-SMP RAM (disassembly reads mapped boot ROM at `$FFC0–$FFFF` when `CONTROL` bit 7 is set) |
| `8` | `GSURAM` | Dynamic | No | No | No | Present with SuperFX/GSU |
| `9` | `GSUBUS` | `8 MiB` | Yes | No | No | Present with SuperFX/GSU |
| `10` | `SA1IRAM` | `2 KiB` | No | No | No | Present with SA-1 |
| `11` | `BWRAM` | Dynamic | No | No | No | Present with SA-1 |
| `12` | `SA1BUS` | `16 MiB` | Yes | No | No | Present with SA-1 |
| `13` | `DSPRAM` | `512 B` / `4 KiB` | No | No | No | Present with NEC DSP (LLE or ST010 HLE) |
| `14` | `DSPPRG` | `6 KiB` / `48 KiB` | Yes | No | No | Present with NEC DSP LLE only (3 bytes per 24-bit instruction) |

**Debug Processors**:
- Processor `0` (`CPU`, 24-bit PC, `CpuBus`): 10 registers (`A, X, Y, S, D, PB, PC, DB, P, E`), full step/breakpoint/call-stack (`JSR`, `JSL`, `RTS`, `RTL`, `RTI`, `BRK`, `COP`, `NMI`, `IRQ`)/watch/coverage hooks.
- Processor `1` (`SPC`, 16-bit PC, `APURAM`): 6 registers (`A, X, Y, SP, PC, PSW`), caught up after every S-CPU instruction during observed frames via `Probe` (`src/probe.rs`) for breakpoints, coverage, and `APURAM` write watches.
- Processor `2` (present when cartridge has SA-1, GSU, or NEC DSP LLE): `SA1` (24-bit PC, `SA1BUS`, 10 registers), `GSU` (24-bit PC, `GSUBUS`, 27 registers), or `DSP` (16-bit PC in 3-byte opcode offsets, `DSPPRG`, 14 registers). Disassembly (`debug_disassemble`) selects the ISA from the target space first, falling back to the processor.

---

## 8. Accuracy Boundaries and Known Exceptions

The following limitations reflect the live implementation in `venusrt`:

1. **239-Line Overscan (`SETINI` bit 2) Not Implemented**: `VBLANK_LINE` is fixed at `225` (`src/bus.rs`) and picture height is fixed at `224` (or `448` interlaced) in `src/ppu.rs`. Games enabling 239-line mode still enter V-Blank at line 225 and present 224 lines.
2. **Active-Display PPU Memory Accesses**: VRAM writes (`$2118–$2119`) are gated to V-Blank and forced blank, matching hardware. CGRAM (`$2122`/`$213B`) and OAM (`$2104`/`$2138`) do not currently enforce active-display write/read address redirection or mid-scanline OAM internal-address corruption.
3. **Mode 7 Multiplier During Active Draw**: Reading `$2134–$2136` always returns `M7A × M7B_hi`; transient intermediate Mode 7 scanline multiplication products during active display are not modeled.
4. **Built-In SPC700 Boot Program vs `spc700.rom`**: Without `spc700.rom`, the clean-room uploader at `$FFC0–$FFFF` contains different instruction bytes and can differ by a few cycles during audio upload handshakes compared to the original Sony mask ROM.
5. **Controller Peripherals**: Only standard 2-port 12-button SNES controllers (`snes.pad`) are modeled; Multitap (ports 3–4 read 0), Super Scope, Mouse, and Justifier are not implemented.
6. **SA-1 Catch-Up Setting (`sa1_catch_up` > 1)**: Non-default values (`4` or `16`) defer SA-1 catch-up while the S-CPU executes outside the cartridge, which can delay SA-1 interrupts and shift ROM/BW-RAM contention timing within that window. The default (`"1"`) catches up after every S-CPU instruction.
7. **Coprocessor Coverage**: Supported cartridge chips are SA-1, SuperFX/GSU-1/GSU-2, OBC1, and NEC uPD77C25 / uPD96050 (DSP-1/1B/2/3/4, ST010, ST011). Chips not implemented in `venusrt` include CX4, S-DD1, SPC7110, S-RTC, ST018, BS-X, and MSU-1.
8. **NEC DSP Open HLE Limits**: Without firmware images, DSP-3 and ST011 have no HLE replacement (those games run without a DSP), DSP-4 HLE implements protocol/multiply only (menus work, races stay black), and DSP-1/1B HLE uses mathematical approximations that diverge on steep tilts past ~80° (`LIMIT = 0x38CE`). Full details are in `Venus_DspHle.md`.
