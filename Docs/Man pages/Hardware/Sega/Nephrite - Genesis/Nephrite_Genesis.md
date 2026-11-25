# Nephrite — Sega Genesis / Mega Drive Hardware

Hardware and emulation reference for the Genesis / Mega Drive subsystem (`src/genesis/`) of `nephrite` (`EmuSen/Cores/Sega/Nephrite - Genesis/`). Shared CPUs, PSG, FM tables, and mode 4 composition live under [Beryl-HW](../Beryl-HW/README.md); Sega CD and 32X attachments are documented in [Nephrite_SegaCD.md](Nephrite_SegaCD.md) and [Nephrite_32X.md](Nephrite_32X.md).

---

## Master Clock, Processors, and Bus Timing (`src/genesis/mod.rs`)

Both processors and all peripherals share a single rational master clock (`master_hz`):

| Parameter | NTSC | PAL |
|---|---|---|
| Master clock | `4,725,000,000 / 88 Hz` (`53,693,181.82 Hz`) | `53,203,425 Hz` |
| 68000 clock (`M68K = 7` master clocks) | `7.67045 MHz` | `7.60049 MHz` |
| Z80 clock (`Z80_T = 15` master clocks) | `3.57955 MHz` | `3.54689 MHz` |
| Scanline length (`LINE`) | `3,420` master clocks | `3,420` master clocks |
| Audio output rate (`sound::RATE`) | `48,000 Hz` | `48,000 Hz` |

### 68000 and Z80 Bus Rules

- **Power-on alignment**: The VDP starts on line `START_LINE = 159`, with the 68000's first bus cycle aligned `START = 745` master clocks into that line.
- **Bus and work RAM refresh**:
  - Cartridge bus refresh fires every `REFRESH = 128 * 7` (`896`) master clocks (first at `START + 889 - REFRESH`): a cartridge access at or after a pending request stalls `REFRESH_WAIT = 2 * 7` clocks; accesses to other spaces clear the pending request without wait. A refresh triggered during a held VDP write holds the bus busy for `REFRESH_BUSY = 126` master clocks.
  - 68000 work RAM refresh begins at `RAM_REFRESH_FIRST = START + 1428` and repeats `RAM_REFRESH_AFTER = 806` master clocks after completion: a RAM access inside `RAM_REFRESH_WINDOW = 133` master clocks stalls `RAM_REFRESH_WAIT = 3 * 7` clocks.
- **68000 → Z80 area (`$A00000`–`$A0FFFF`)**: Each access incurs `Z80_AREA_WAIT = 7` extra master clocks (5 68000 clocks total), with the Z80-side strobe ending `Z80_AREA_LEAD = 4 * 7` master clocks before the 68000 cycle finishes (and an extra 68000 clock when starting `1..=6` 68000 clocks after a refresh request). Writes to `$A11200` move the shared Z80/YM2612 reset line at the same lead time.
- **Z80 → 68000 bank window (`$8000`–`$FFFF`)**: Maps a 32 KiB window selected by the 9-bit serial shift register `z80_bank` (bits 23–15 of the 68000 address). Each windowed access costs `WINDOW_Z80_WAIT = 41` master clocks (`2.75` T-states) on the Z80 and stalls the 68000 by `WINDOW_68K_STALL = 66` master clocks (`9.5` 68000 clocks) at its next bus access.
- **Z80 interrupt (`Z80_INT`)**: Asserted with VDP V-INT and held for one scanline (`3,420` master clocks).
- **TMSS (`$A14000`)**: Models with `version != 0` require `"SEGA"` written to `$A14000` before touching the VDP; accessing the VDP while locked halts the machine (`locked_up = true`), matching hardware.
- **68000 locked `TAS` writes and open bus**: The 68000 bus drops locked writes from `TAS` (read-modify-write leaves RAM unchanged). Unmapped 68000 reads return the last program-space prefetch word (`prefetch`).
- **Models (`Model`)**: Tracks `overseas`, `pal`, `version` (low nibble of `$A10001`), and `model2` (Model 1 VA3–VA6.5 discrete YM2612 + output filter vs Model 2 VA0–VA1.8 integrated YM3438 + output filter).

---

## VDP, Slot Schedule, FIFO, DMA, and Rendering (`src/genesis/vdp.rs`, `src/genesis/render.rs`)

### Scanline Timing and Access Slots

Each scanline (`3,420` master clocks) is precomputed for `H32` (342 pixels, 171 slots of 4 serial clocks at `MCLK/5`) and `H40` (420 pixels, 210 slots using `MCLK/4` outside HSync and `MCLK/5` with three `MCLK/4` pairs inside HSync):

- **H counter**: `H32` runs `$10A`–`$127` then jumps to `$1D2`–`$109`; `H40` runs `$14A`–`$16C` then jumps to `$1C9`–`$149`. Active pixel 0 begins at `H = $018`.
- **Slot tables**: Slot 1 starts at `H = $1E9` in both widths.
  - `H40`: 18 external slots per active line (`H40_EXTERNAL`), 5 DRAM refresh slots (`H40_REFRESH`).
  - `H32`: 16 external slots per active line (`H32_EXTERNAL`), 4 DRAM refresh slots (`H32_REFRESH`).
  - Blank lines expose every non-refresh slot; the first blank line transitions from the active table to the blank table at `H = $14E` (`H40`) or `$10E` (`H32`).
- **Interrupts and blanking**: V-INT raises at `H = $001`; VBlank toggles at `H = $14C` (`H40`) or `$10C` (`H32`). HBlank is clear during `$00B..$166` (`H40`) or `$00A..$126` (`H32`). Control-port reads/writes and HV counter reads sample `HV_LATE = 14` master clocks before the end of the 68000 cycle.

### FIFO, DMA, and Memory Quirks

- **4-word write FIFO**: Ring buffer (`fifo: [Entry; 4]`) retaining entry data after drain. A word arriving into an empty FIFO cannot land before `WRITE_START = 176` master clocks. When full, the 68000 write stalls until `FIFO_RELEASE = 24` master clocks after the slot that frees an entry (aligned to the next 68000 clock).
- **DMA modes (`Dma`)**:
  - **68000 bus transfer (`Dma::Bus`)**: Begins `DMA_START = 88` master clocks after the command and fetches words every `DMA_FETCH = 20` master clocks (routed through `Attachment::transfer16` when an attachment is fitted).
  - **Fill (`Dma::FillWait` / `Dma::Fill`)**: Triggered by the subsequent data-port write; writes the high byte to VRAM (or FIFO's next entry to CRAM/VSRAM) with address bit 0 inverted, advancing source registers like copy.
  - **VRAM copy (`Dma::Copy`)**: Copies bytes within VRAM with address bit 0 inverted.
- **128 KiB VRAM mode on 64 KiB hardware (`vram128_address`)**: Remaps addresses as `(a >> 1 & 1 ^ 1) | (a & 0x3FC) | (a >> 9 & 2) | (a & 0xF800) >> 1`.
- **SAT cache**: First 4 bytes of each sprite entry are cached on VRAM writes (`sat_cache`) rather than re-read when the SAT base register changes.

### Mode 5 and Mode 4 Rendering (`src/genesis/render.rs`)

- **Span-based mode 5 renderer**: Renders lines up to each mid-line register, VRAM, CRAM, or VSRAM write. Supports planes A and B (32/64/128-cell widths/heights clamped to 4,096 cells), full/cell/line horizontal scroll, full and 2-cell vertical scroll (`VSCROLL_LEAD = 27`, `PATTERN_LEAD = 24` pixels ahead), window plane (including the window horizontal-scroll bug), sprite link traversal on the preceding line with per-line/per-frame limits and sprite masking, shadow/highlight, double-resolution interlace (`320 × 480` max), 8-colour mode, and CRAM dots.
- **Colour DAC ladder (`LADDER`)**: 15-step measured DAC levels `[0, 29, 52, 70, 87, 101, 116, 130, 144, 158, 172, 187, 206, 228, 255]` (shadow `0–7`, normal even steps `0, 2, …, 14`, highlight `7–14`).
- **Mode 4**: Delegated to `beryl_sms_vdp::compose` with `Variant::GENESIS`, using the Genesis's mode 4 VRAM bit swizzle (`vram4_address`) and board DAC levels (`MODE4_RG = [0, 95, 161, 255]`, `MODE4_B = [0, 109, 161, 255]`).

---

## Audio: YM2612 / YM3438, PSG, and Output Circuits (`src/genesis/ym2612.rs`, `fm.rs`, `lfo_tables.rs`, `sound.rs`)

### YM2612 Timing, Operator ROMs, and LFO Vibrato

- **Clocking and slot pipeline (`ym2612.rs`)**: One sample = `144` 68000 clocks = `1,008` master clocks (`24` slots of `42` master clocks each, slot edges at `SLOT_EDGE = 32` master clocks past the sample deadline).
  - Busy flag lasts `BUSY = 33 * 42` master clocks from the next slot edge; on the discrete Model 1 YM2612 (`discrete = true`), busy reads only on port 0, while on the Model 2 integrated YM3438 it reads on all four ports.
  - Each register group has its measured slot-edge deadline relative to the sample (`TAKE_FEEDBACK = 6`, `TAKE_PHASE_TEST = 11`, `TAKE_SPECIAL = 13`, `TAKE_OPERATOR = 18`, `TAKE_FREQUENCY = 24`, `TAKE_EG_TEST = 31`, `TAKE_LFO = 35`, `TAKE_TOP_BIT_TEST = 35`, `TAKE_PAN = 42`, `TAKE_DAC_TEST = 48`, `TAKE_DAC = 64`), and key-on (`$28`) is sampled per channel starting `KEY_LEAD = 472` master clocks before the deadline.
  - Timers A (10-bit) and B (8-bit with `/16` divider) tick `LOAD_LEAD = KEY_LEAD - 2 * 42` before the deadline; overflow flags become readable at `FLAG_LEAD`. CSM mode keys channel 3 on Timer A overflow/load.
  - Test register `$21` bit 6 enables hardware test readback (`test_read`) of 14-bit operator outputs, 9-bit channel outputs (in hardware pin order `1, 5, 3, 2, 6, 4`), channel 4 S4 phase bits, and envelope step pulses.
- **Operator ROM provenance (`fm.rs`, `beryl-fm-tables`)**: Both the 256-entry 4.8 log-sine ROM (`LOGSIN`, quarter-wave sampled at `(x + 0.5) * π / 512` to mirror cleanly across all four quadrants without duplicating peak or zero crossing) and the 256-entry 10-bit exponential ROM (`EXP`, `round((2^(x/256) - 1) * 1024)` with implicit leading `1024`, read at `255 - frac` for `2^-x`) come from `beryl-fm-tables`. Operator output computes 14-bit signed values with 10-bit 4.6 attenuation, detune (`DETUNE`), SSG-EG inversion/alternation, and envelope rate stepping every 3rd sample (`SHIFT`, `STEPS`, `EXTRA`).
- **Measured LFO vibrato tables (`lfo_tables.rs`)**:
  - LFO runs 128 counts per cycle across 8 periods (`LFO_PERIOD = [108, 77, 71, 67, 62, 44, 8, 5]` samples per count). Tremolo (`AMS`) shifts a `126..=0..=126` triangle by `AMS_SHIFT = [0, 3, 1, 0]`.
  - Vibrato (`PMS 0–7`) runs `PM_STEPS = 32` steps per cycle (quarter-wave `0–7` mirrored and negated; steps `q = 0, 1` are always 0). Using only the top 7 bits of F-number (`top = (fnum >> 4) & 0x7F`; low 4 bits never contribute), `pm_offset` sums selected shifted terms `top >> k` (`PM_TERMS[pms][q]`) and shifts the sum right by `PM_SHIFT[pms][q]` — rounding after summing rather than per bit — yielding a half-F-number offset added to `(fnum << 1)` and wrapped to 12 bits (`pm_fnum`), with phase increment base `(pm_fnum << block) >> 2`. All `524,288` `(PMS, fnum, step)` combinations match the board hash `0x8F2D_DBC5`.

### PSG, Mixing, and Console Output Circuits (`sound.rs`)

- **PSG**: `beryl-sn76489` (`Variant::SEGA`) clocked every `PSG_CLOCK = 15 * 16` (`240`) master clocks. Relative levels are `FM_GAIN = 16` per 9-bit YM2612 step and `PSG_GAIN = (81, 250)` (full-volume PSG channel = `0.324` of the YM2612 DAC's 512-step swing).
- **Zero-order-hold compensation (`UNHOLD`)**: A 7-tap linear-phase FIR at 48 kHz restores the 2.1 dB sinc droop introduced by holding FM samples across the step synthesizer to within `0.08 dB` up to 20 kHz; PSG samples delay by 3 output samples to stay aligned.
- **Model output filters (`Circuit`)**: Fixed-point 24-bit biquad + high-pass cascades applied to the FM + PSG mix (attachment audio such as Sega CD CD-DA/PCM adds *after* the console circuit):
  - `Circuit::MODEL1` (VA3–VA6.5): 1st-order low-pass at `3,216 Hz` + high-pass at `23 Hz`.
  - `Circuit::MODEL2` (VA0–VA1.8): 2nd-order low-pass at `5,805 Hz` (`Q = 1.16`) + 1st-order low-pass at `4,111 Hz` + high-pass at `16 Hz`.
  - `Circuit::FLAT`: Bypass for raw testing.

---

## Cartridges, Mappers, I/O, and Save States (`cart.rs`, `eeprom.rs`, `io.rs`, `state.rs`)

- **ROM & SRAM (`cart.rs`)**: ROM up to 4 MiB mirrors by next power of two. Header-declared SRAM maps over ROM via `$A130F1` bit 0 (bit 1 write-protects) or unconditionally when mapped past the ROM's end; supports even, odd, or 16-bit lanes and persists in Genesis Plus GX–compatible 64 KiB+ address-offset battery files (also reading compact sequential files).
- **Sega SSF mapper**: Activated by `"SEGA SSF"` system header or ROMs `> 4 MiB`; `$A130F3`–`$A130FF` map 512 KiB pages into banks `1–7` (bank 0 fixed at page 0).
- **Serial I²C EEPROMs (`eeprom.rs`)**: 21 known boards (`BOARDS`) across Sega, Acclaim, Codemasters, and EA in Mode 1 (`X24C01`, 7-bit address in command byte), Mode 2 (`24C01`–`24C16`), and Mode 3 (`24C32`–`24C65` with 2-byte addresses and 64-byte page mask).
- **Sonic & Knuckles lock-on**: Detected by serial `"GM MK-1563"`; maps top cartridge at `$200000`–`$3FFFFF` (open bus when empty) and optional 256 KiB patch ROM over `$300000`–`$3FFFFF` when `$A130F1` bit 0 is set.
- **Controllers & Multitaps (`io.rs`)**: Ports A, B, and EXP at `$A10000`–`$A1001F`. Supports 3-button and 6-button pads (timeout `SIX_BUTTON_TIMEOUT = 8192 * 7` master clocks), Sega Team Player (`Plug::TeamPlayer`, up to 8 pads across both ports), EA 4 Way Play (`Plug::FourWay`), and Codemasters J-Cart boards (`JCARTS`, `$38FFFE`/`$3FFFFE`).
- **Save states (`state.rs`)**: Magic `"NPHR"`, current version `20` (loads Genesis versions `16..=20`, Sega CD / mode 1 version `20`).

## Accuracy Bounds

- Model 2 audio models the linear filter response of VA0–VA1.8 boards; non-linear op-amp clipping/distortion on Model 2 amplifiers is not modelled.
- Light guns (Menacer, Justifier), Sega Mouse, and serial modem peripherals on the EXP port are not modelled.
- Genesis mode 4 right-column vertical scroll lock triggers at pixel 192 (`Variant::GENESIS` in `beryl-sms-vdp`) rather than by the column counter.
