# Sapphire: IOP, SIF, EE DMAC/INTC/Timers, SPU2, CDVD, and SIO2

Hardware reference for the I/O Processor (IOP), Sub-Bus Interface (SIF), EE DMA Controller, EE Interrupt Controller, EE Timers, SPU2, CDVD drive controller, and SIO2 serial peripherals in `sapphire`.

---

## 1. I/O Processor (IOP)

`src/iop/` runs the shared `blackmoon-r3000` R3000A CPU crate (`PRId = 0x1F`, reset vector `0xBFC0_0000` with `SR.BEV = 1`) at one-eighth of the EE clock (`clock::IOP = 8`, 36.864 MHz nominal) over Sapphire's IOP bus (`IopBus`).

### 1.1 IOP Bus, Cache, and Scratchpad (`src/iop/bus.rs`, `icache.rs`)

- **Memory & Wait States**:
  - **IOP RAM**: 2 MiB (`2 << 20`), mirrored across the first 8 MiB (`0x0000_0000..0x0080_0000`); load stall `RAM_STALL = 4` IOP cycles.
  - **Scratchpad**: 1 KiB at `0x1F80_0000..0x1F80_0400` in KUSEG and KSEG0 only (0 wait states).
  - **Boot ROM**: 4 MiB shared with the EE at `0x1FC0_0000..0x2000_0000`; load stall `ROM_STALL = 6` IOP cycles.
  - **I/O Windows**: SIF (`0x1D00_0000..0x1D00_0100`), CDVD (`0x1F40_0000..0x1F50_0000`), IOP I/O (`0x1F80_1000..0x1F81_0000`), SPU2 (`0x1F90_0000..0x1F91_0000`); I/O load stall `IO_STALL = 2` IOP cycles. DVD-player ROM space (`0x1E00_0000..0x1F00_0000`) and unmapped RAM mirrors (`0x0080_0000..0x1D00_0000`) read zero and ignore writes.
  - **KSEG2 Cache Control**: `0xFFFE_0130` (`BCC`), plus zero-reading control space `0xFFFE_0000..0xFFFE_0200`.
- **Instruction Cache (`src/iop/icache.rs`)**: 4 KiB direct-mapped (256 lines × 4 words), per-word valid bits, `SR.IsC` isolated read/write support, and pre-decoded `emusen_mips::Op` cache entries alongside each word (re-decoded on save-state load).

### 1.2 IOP Interrupt Controller (`src/iop/intc.rs`)

Mapped at `I_STAT` (`0x1F80_1070`, write 0 to acknowledge), `I_MASK` (`0x1F80_1074`, 26 active bits `0x03FF_FFFF`), and `I_CTRL` (`0x1F80_1078`, read returns bit 0 and atomically clears it). Drives R3000A `Cause.IP2` when `(I_CTRL & 1) != 0 && (I_STAT & I_MASK) != 0`.

Key IRQ lines: `0` VBLANK_START, `1` GPU/SBUS (raised by EE `SIF_CTRL` bit 18), `2` CDVD, `3` DMA, `4–6` TIMER0–2, `7–8` SIO0/1, `9` SPU2, `10` PIO, `11` VBLANK_END, `12` DVD, `13` PCMCIA, `14–16` TIMER3–5, `17` SIO2, `22` USB, `24` FWRE, `25` FDMA.

### 1.3 IOP Timers (`src/iop/timer.rs`)

Six closed-form event-driven counters (no per-cycle polling):

| Timers | Width | Register Bases (`+0` Count, `+4` Mode, `+8` Target) | Clocks & Prescalers | Gate Signal |
| --- | --- | --- | --- | --- |
| **T0–T2** | 16-bit | `0x1F80_1100`, `0x1F80_1110`, `0x1F80_1120` | T0: IOP / Pixel (`EXTC`); T1: IOP / H-blank (`EXTC`); T2: IOP / IOP÷8 (`PSCL2`) | T0: H-blank; T1: V-blank; T2: Stop/Run |
| **T3–T5** | 32-bit | `0x1F80_1480`, `0x1F80_1490`, `0x1F80_14A0` | T3: IOP / H-blank (`EXTC`); T4–T5: IOP ÷ `{1, 8, 16, 256}` (`PSCL45`) | T3: V-blank; T4–T5: Stop/Run |

Supports `ZRET`, target/overflow IRQs (`CMP`/`OVF`), one-shot vs repeat (`REPT`), pulse vs toggle (`LEVL`), and active-low `INTF` bit 10.

### 1.4 IOP DMA Controller (`src/iop/dma.rs`)

13 channels (`0` MDEC_IN, `1` MDEC_OUT, `2` SIF2, `3` CDVD, `4` SPU2 Core 0, `5` PIO, `6` OTC, `7` SPU2 Core 1, `8` DEV9, `9` SIF0, `10` SIF1, `11` SIO2_IN, `12` SIO2_OUT):

- **Registers**: Channels 0–6 at `0x1F80_1080 + 0x10 * n`; channels 7–12 at `0x1F80_1500 + 0x10 * (n - 7)` (`MADR`, `BCR`, `CHCR`, `TADR`); global control at `DPCR` (`0x1F80_10F0`, reset `0x0777_7777`), `DICR` (`0x1F80_10F4`), `DPCR2` (`0x1F80_1570`), `DICR2` (`0x1F80_1574`), `DMACEN` (`0x1F80_1578`), and `DMACINTEN` (`0x1F80_157C`, reset `1` so channel interrupts reach IRQ 3).
- **Modes**: Burst (with chopping), Slice, Linked-List, and Chain mode. Channels 9 (`SIF0`) and 10 (`SIF1`) always operate in Chain mode (`mode_of` forces `3`). Each word holds the bus for 1 IOP cycle with a 1-cycle gap between slice blocks/list nodes.

---

## 2. Sub-Bus Interface (SIF)

`src/sif/mod.rs` connects the EE and IOP via shared mailbox registers and three 32-word (128-byte) FIFOs:

| Register | EE Address (`0x1000_F200 + off`) | IOP Address (`0x1D00_0000 + off`) | Semantics |
| --- | --- | --- | --- |
| `MSCOM` | `+0x00` | `+0x00` | EE writes, both read |
| `SMCOM` | `+0x10` | `+0x10` | IOP writes, both read |
| `MSFLG` | `+0x20` | `+0x20` | EE write sets bits (`|= v`); IOP write clears bits (`&= !v`) |
| `SMFLG` | `+0x30` | `+0x30` | IOP write sets bits (`|= v`); EE write clears bits (`&= !v`) |
| `CTRL` | `+0x40` | `+0x40` | EE reads `0xF000_0102`, IOP reads `0xF000_0002`; EE bit 18 raises IOP IRQ 1; EE bit 19 asserts/releases IOP reset |
| `BD6` | `+0x60` | `+0x60` | Shared 32-bit storage |

- **DMA FIFOs (`FIFO_WORDS = 32`)**:
  - **SIF0** (IOP -> EE): IOP DMAC ch 9 -> EE DMAC ch 5.
  - **SIF1** (EE -> IOP): EE DMAC ch 6 -> IOP DMAC ch 10.
  - **SIF2** (Bidirectional): EE DMAC ch 7 <-> IOP DMAC ch 2 (direction set by EE `D7_CHCR.DIR`).
- The EE transfers 128-bit quadwords (4 words) while the IOP transfers 32-bit words. FIFO mutations set `wake_ee` / `wake_iop` flags so blocked DMA channels on the opposite processor resume immediately.

---

## 3. EE DMAC, EE INTC, and EE Timers

### 3.1 EE DMAC (`src/dmac.rs`)

Ten 128-bit channels transferring at 1 quadword per bus cycle (`clock::BUS = 2` EE cycles):

| Ch | Name | Register Base | Supported Modes |
| --- | --- | --- | --- |
| 0 | `VIF0` | `0x1000_8000` | Normal, Source Chain |
| 1 | `VIF1` | `0x1000_9000` | Normal, Source Chain (bidirectional via `CHCR.DIR`) |
| 2 | `GIF` | `0x1000_A000` | Normal, Source Chain (MFIFO drain option) |
| 3 | `fromIPU` | `0x1000_B000` | Normal |
| 4 | `toIPU` | `0x1000_B400` | Normal, Source Chain |
| 5 | `SIF0` | `0x1000_C000` | Normal, Dest Chain |
| 6 | `SIF1` | `0x1000_C400` | Normal, Source Chain |
| 7 | `SIF2` | `0x1000_C800` | Normal (bidirectional via `CHCR.DIR`) |
| 8 | `fromSPR` | `0x1000_D000` | Normal, Dest Chain, Interleave (burst) |
| 9 | `toSPR` | `0x1000_D400` | Normal, Source Chain, Interleave (burst) |

- **Global Registers**: `D_CTRL` (`0x1000_E000`: `DMAE`, `RELE`, `MFD`, `STS`, `STD`, `RCYC`), `D_STAT` (`0x1000_E010`: `CIS[9:0]`, `SIS`, `MEIS`, `BEIS`, masks `CIM[9:0]`, `SIM`, `MEIM`), `D_PCR` (`0x1000_E020`: `CPC`, `CDE`, `PCE`), `D_SQWC` (`0x1000_E030`), `D_RBSR` (`0x1000_E040`), `D_RBOR` (`0x1000_E050`), `D_STADR` (`0x1000_E060`), `D_ENABLER` (`0x1000_F520`, resets to `0x1201`), `D_ENABLEW` (`0x1000_F590`, `CPND` bit 16).
- **Chains, Stack, MFIFO, and Stall Control**: Full Source Chain (`refe`, `cnt`, `next`, `ref`, `refs`, `call`, `ret`, `end`) with 2-level address stack (`ASR0`/`ASR1`, `CHCR.ASP`), `TTE`, and `TIE`; Dest Chain (`cnt`, `cnts`, `end`); MFIFO ring buffering (`D_RBOR`/`D_RBSR`, drain channel `VIF1` or `GIF` fed by `fromSPR`); and Stall Control (`STS` updates `D_STADR`; `STD` holds `VIF1`/`GIF`/`SIF1` when `MADR + 8 qwords > D_STADR`).
- **Lines**: Drives EE `INT1` from `D_STAT` (channel, stall, MFIFO empty, or bus error) and `CPCOND0` from `(!D_PCR & 0x3FF) | (D_STAT & 0x3FF) == 0x3FF`. When `board.round > 0`, channels 1, 2, 8, and 9 run on the ahead transaction slot (`Ahead`) with completion flags/interrupts stamped to land at their exact EE cycle.

### 3.2 EE INTC (`src/intc.rs`)

Mapped at `I_STAT` (`0x1000_F000`, write-1-to-clear) and `I_MASK` (`0x1000_F010`, write-1-to-toggle) over 15 edge-latched sources (`0x7FFF`): `0` GS, `1` SBUS, `2` VB_ON, `3` VB_OFF, `4` VIF0, `5` VIF1, `6` VU0, `7` VU1, `8` IPU, `9–12` TIMER0–3, `13` SFIFO, `14` VU0WD. Drives EE `INT0` whenever `(I_STAT & I_MASK) != 0`.

### 3.3 EE Timers (`src/timer.rs`)

Four 16-bit timers at `0x1000_0000 + n * 0x800` (`COUNT` `+0x00`, `MODE` `+0x10`, `COMP` `+0x20`, and `HOLD` `+0x30` on T0 and T1 only):

- **Clocks (`CLKS`)**: `BUSCLK` (2 EE cycles), `BUSCLK/16`, `BUSCLK/256`, or H-blank rising edge.
- **Gating (`GATE`/`GATS`/`GATM`)**: H-blank or V-blank gate supporting hold-during-high (`GATM = 0`) and reset on rising/falling/both edges (`GATM = 1..3`), disabled if both clock and gate select H-blank.
- **Events**: Evaluated lazily at read/write/edge/next-event time; rising edges of `EQUF` (`CMPE`) and `OVFF` (`OVFE`) signal EE INTC `TIMER0 + n`.

---

## 4. SPU2

`src/spu2/` implements the dual-core PlayStation 2 sound processor on `blackmoon-spu`, mapped at `0x1F90_0000..0x1F90_0800` with `2 MiB` of sound RAM (`RAM_BYTES = 2 << 20`, 20-bit halfword addresses) and native stereo output at exactly 48,000 Hz (`CYCLES_PER_SAMPLE = 768` IOP cycles).

- **Dual 24-Voice Cores (`src/spu2/cores.rs`)**: Core 0 (IOP DMA channel 4) and Core 1 (IOP DMA channel 7) each hold 24 `blackmoon_spu::Voice` channels (ADPCM decode, pitch modulation `PMON`, noise `NON`, ADSR envelopes, per-voice dry/wet mix `VMIXL/R` & `VMIXEL/R`, `KEY_ON`/`KEY_OFF`/`ENDX`).
- **AutoDMA (`src/spu2/adma.rs`)**: Streams double-buffered 16-bit stereo PCM into `MEMIN` (Core 0 left `0x2000`/`0x2100`, right `0x2200`/`0x2300`; Core 1 offset `+0x400`, 256 samples = 512 bytes per side per buffer). Completing a 256-sample playback buffer frees it and wakes the core's IOP DMA channel.
- **Core 0 -> Core 1 Routing, Reverb, and Output Areas (`src/spu2/mod.rs`)**: Each sample ticks Core 0 (with zero external input) and passes Core 0's master-volume output into Core 1's external input (`AVOL`), while each core mixes external input (`AVOL`), AutoDMA sound data (`BVOL`), and voices through `MMIX` into dry output and its `blackmoon_spu::Reverb` effects area, then scales by sweeping `MVOL`. Core 1's output feeds the host `SampleQueue`. Per-sample monitor copies are written to sound RAM output areas (`0x400`/`0x600`/`0x800`/`0xA00`/`0x1000`+ halfword offsets).
- **Cross-Core IRQ Watch**: Every sound RAM read/write by either core's voices, reverb, DMA, manual `DATA` transfer, or output-area write checks both cores' armed `IRQA` halfword addresses (`Watch`), latching `IRQ_INFO` (`0x7C2` bits 2 and 3) and raising IOP IRQ 9.

---

## 5. CDVD Drive Controller

`src/cdvd/` implements the MechaCon register interface at `0x1F40_2000` (accessible from both IOP and EE buses, IOP DMA channel 3, IOP IRQ 2) backed by `emusen-disc` (`Disc = emusen_disc::Medium<Kept>`), keeping host disc bytes zero-copy.

- **Registers**: `N_COMMAND` (`0x04`), `N_STATUS`/params (`0x05`), `ERROR` (`0x06`), `BREAK` (`0x07`), `I_STAT` (`0x08`: `DATA_READY`, `COMPLETE`, `POWER_OFF`, `EJECTED`), `DRIVE` (`0x0A`), `STICKY` (`0x0B`), `DISK_TYPE` (`0x0F`), `S_COMMAND` (`0x16`), `S_STATUS`/params (`0x17`), `S_RESULT` (`0x18`), and ReadKey authentication registers (`0x20..0x24`, `0x34`, `0x38`).
- **N Commands (timed, raise IRQ 2)**: `NOP` (`0x00`), `NOP_SYNC` (`0x01`), `STANDBY` (`0x02`), `STOP` (`0x03`, 166.7 ms), `PAUSE` (`0x04`), `SEEK` (`0x05`), `READ_CD` (`0x06`: 2048/2328/2340 B), `READ_CDDA` (`0x07`: 2352 B), `READ_DVD` (`0x08`: 2064 B frame), `GET_TOC` (`0x09`), and `READ_KEY` (`0x0C`: derives disc key bytes and sets authentication status at `0x38`).
- **S Commands (immediate)**: `SUBCOMMAND 0x03` (`MECHACON_VERSION` -> `[0x03, 0x06, 0x02, 0x00]`), `UPDATE_STICKY` (`0x05`), `TRAY` (`0x06`), `READ_RTC` (`0x08`), `WRITE_RTC` (`0x09`), `FORBID_DVD` (`0x15` -> `0x05`), `REGION` (`0x36` -> 15-byte market block for Japan/Americas/Europe), BIOS boot probes (`0x17`, `0x1A`, `0x1E`, `0x22`, plus 12 acknowledged `0x00`-reply commands), and NVM config commands (`0x40`–`0x43`).
- **Seek & Read Timing (`src/cdvd/timing.rs`)**: Uses `IOP_HZ = 36_864_000`, 24× CD (`3,686,400 B/s`) and 4× DVD (`5,529,600 B/s`) block formulas, contiguous thresholds (`< 8` CD / `< 16` DVD sectors), fast seek (30 ms, `< 4371` CD / `< 14764` DVD), full seek (100 ms), spin-up (333 ms), and dual-layer Opposite Track Path radial distance calculation.
- **NVM EEPROM (`src/cdvd/nvm.rs`)**: 1024-byte (`BYTES = 1024`) console EEPROM organized as 4 areas × 16 blocks × 16 bytes, accessed via `OpenConfig`/`ReadConfig`/`WriteConfig`/`CloseConfig` sessions and persisted as battery medium `ps2.nvm` (`.jp.nvm`, `.us.nvm`, `.eu.nvm`).
- **RTC (`src/cdvd/rtc.rs`)**: BCD real-time clock anchored to 2000-01-01 00:00:00, advancing strictly on IOP cycles (`ReadRTC`/`WriteRTC`).

---

## 6. SIO2 Serial Interface

`src/sio2/` implements the IOP serial controller at `0x1F80_8200..0x1F80_8280` (`SEND3[16]`, `SEND12[8]`, `FIFOIN`, `FIFOOUT`, `CTRL`, `RECV1`, `RECV2 = 0xF`, `RECV3`, `ISTAT`), fed by IOP DMA channels 11 (`SIO2_IN`) and 12 (`SIO2_OUT`) and raising IOP IRQ 17 on queue completion. Each byte transfers in `BYTE = 147` IOP cycles (~250 KB/s), supporting command lengths up to `LENGTH_MAX = 512` bytes across 4 logical ports (0–1 controller connectors, 2–3 memory card slots).

- **DualShock 2 (`src/sio2/pad.rs`)**: Full state machine for peripheral byte `0x01`: digital, analog, and pressure-sensitive modes (12 pressure buttons mapped from digital input as `0xFF`/`0x00`), configuration mode (`0xF3`), response mask configuration (`0x4F`), vibration motor mapping (`0x4D`, small/large motor state), analog lock (`0x44`), and query commands (`0x40`–`0x4C`).
- **Multitap (`src/sio2/tap.rs`)**: Peripheral byte `0x21` answered on both controller and card ports of a tapped slot (`SLOTS = 4`), supporting `0x12`/`0x13` (report 4 slots) and `0x21`/`0x22` (select pad or memory card slot `0..3`).
- **PS2 Memory Card (`src/sio2/card.rs`)**: Backed by `emusen-ps2-card` (`RAW_PAGE = 528` bytes = 512 data + 16 spare, `PAGES_PER_BLOCK = 16`, `PAGES = 16384`, `CARD_BYTES = 8,650,752`), peripheral byte `0x81`. Implements probe (`0x11`/`0x12`), address setup (`0x21` erase / `0x22` write / `0x23` read), specs (`0x26`), terminator configuration (`0x27`/`0x28`), chunked page write/read (`0x42`/`0x43`, up to 128 bytes with XOR checksum), program/commit (`0x81`, NAND bit-clearing AND semantics), and block erase (`0x82`, sets block to `0xFF`), with NAND flash busy delays (`LOAD = 25 µs`, `PROGRAM = 250 µs`, `ERASE = 2 ms` in EE cycles). MagicGate authentication commands (`0xF0`, `0xF3`, `0xF7`) receive no response.
