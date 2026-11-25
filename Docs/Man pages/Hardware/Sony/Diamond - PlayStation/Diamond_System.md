# Diamond System Architecture (`diamond`)

Crate path: `EmuSen/Cores/Sony/Diamond - PlayStation/` (library `diamond`, published Core ABI v1 library `v1/` -> `libdiamond.so`).
Implements the Sony PlayStation (`psx`) system board around [`blackmoon-r3000`](../BlackMoon-HW/BlackMoon_R3000.md) and [`blackmoon-gte`](../BlackMoon-HW/BlackMoon_GTE.md).

## Clocks and Video Timing Domains

Diamond runs two primary clock domains (`src/video.rs`, `src/spu/mod.rs`):

| Domain | Frequency / Ratio | Notes |
|---|---|---|
| CPU / System clock | `33.8688 MHz` | Master reference clock owned by `Board::now` (`u64`) |
| SPU audio clock | `44,100 Hz` (`768` CPU cycles/sample) | Stepped sample-by-sample on device access and frame end |
| NTSC video clock | `53.693175 MHz` (`1,431,818 / 451,584` half-clocks per CPU cycle) | `3,412.5` video clocks (`6,825` half-clocks) per line, `263` lines/field |
| PAL video clock | `53.203425 MHz` (`1,418,758 / 451,584` half-clocks per CPU cycle) | `3,405` video clocks (`6,810` half-clocks) per line, `314` lines/field |

Video positions are tracked in **half video clocks** so NTSC's `3,412.5`-clock scanline is exact. Frames complete when a field's vblank begins.

## Memory Map and Bus Timing (`src/bus.rs`, `src/memctl.rs`)

### Physical Address Regions

| Region | Physical Range | Size / Notes |
|---|---|---|
| Main RAM | `0x0000_0000`–`0x007F_FFFF` | `2 MiB` (`0x20_0000`) mirrored across the bank window configured in `DRAM_CTRL` (`0x1F80_1060`, kernel default `0x0B88`) |
| Expansion 1 (`DEV0`) | `0x1F00_0000`+ | Programmable base (`0x1F80_1000`) and delay/size (`0x1F80_1008`) |
| Scratchpad | `0x1F80_0000`–`0x1F80_03FF` | `1 KiB` (`0x400`) fast data RAM (accessible via KUSEG/KSEG0; unmapped in KSEG1) |
| Hardware I/O | `0x1F80_1000`–`0x1F80_1FFF` | Memory control, SIO0/SIO1, IRQ, DMA, timers, CD-ROM, GPU, MDEC, SPU |
| Expansion 2 (`DEV8`) | `0x1F80_2000`+ | Programmable base (`0x1F80_1004`), DUART TTY transmit at `0x1F80_2023`, POST register |
| BIOS ROM (`DEV2`) | `0x1FC0_0000`–`0x1FC7_FFFF` | `512 KiB` (`0x8_0000`), open kernel (`diamond-kernel.bin`) or player-supplied retail BIOS |
| BIU / Cache Control (`BCC`) | `0xFFFE_0130` (KSEG2) | I-cache enable, block size, tag-test mode, scratchpad enable (`KERNEL_BCC = 0x0001_E988`) |

Unmapped regions (including KSEG2 outside `0xFFFE_0130`) signal `Bus::bus_error()` so `blackmoon-r3000` raises `IBE` or `DBE`.

### Wait States and Write Queue

- **Memory control (`0x1F80_1000`–`0x1F80_1020`):** `MemControl` computes first-cycle and sequential access times (`Timing { first, seq, wide, increment }`) from the `DEV0`, `DEV1`, `DEV2` (BIOS), `DEV4` (SPU), `DEV5` (CD-ROM), `DEV8`, and `COM_DELAY` registers.
- **CPU load/store timing:**
  - Main RAM load stall: `RAM_STALL = 4` CPU cycles beyond the instruction's cycle.
  - Internal I/O register load stall: `IO_STALL = 2` CPU cycles.
  - Write queue: 4-entry FIFO draining stores to RAM (`RAM_DRAIN = 8` cycles) without stalling the CPU unless full or serialized by a load.
- **GTE interlock:** `Board::gte_ready` tracks the bus cycle when the active GTE command finishes; any COP2 access arriving early stalls until `gte_ready + 1`.

## Instruction Cache (`src/icache.rs`)

- **Geometry:** `4 KiB` direct-mapped cache (`256` lines × `4` words = `1024` words). Each line stores a physical tag (bits `31..=12`) plus 4 per-word valid bits (bits `3..=0`).
- **Tag-test and isolation:** When `SR.IsC == 1`, CPU data loads/stores reach `isolated_read`/`isolated_write` on the cache tags or data words according to `BCC.TAG` (bit 2) and `BCC.IS1` (bit 11). `BCC` bits 6 and 10 read zero (`WRITABLE = !(1 << 6 | 1 << 10)`).
- **Decoded-word cache (exact speed optimization):** Alongside `words: [u32; 1024]`, `ICache` caches `ops: [emusen_mips::Op; 1024]` decoded on refill (and rebuilt via `ICache::redecode` after state loads). `Bus::fetch_decoded` returns `(word, op)` directly on I-cache hits with zero behavioral difference.

## DMA Controller (`src/dma.rs`)

Seven channels at `0x1F80_1080`–`0x1F80_10FF` (`MADR`, `BCR`, `CHCR` per channel; `DPCR` at `0x1F80_10F0`, reset `0x0765_4321`; `DICR` at `0x1F80_10F4`):

| Ch | Device | Sync Modes / Notes |
|---:|---|---|
| `0` | `MDEC_IN` | RAM → MDEC input FIFO (sync 1 slice) |
| `1` | `MDEC_OUT` | MDEC output FIFO → RAM (sync 1 slice) |
| `2` | `GPU` | RAM ↔ GP0/GPUREAD (sync 1 slice, sync 2 linked list) |
| `3` | `CDROM` | CD-ROM sector buffer → RAM (sync 0 burst) |
| `4` | `SPU` | RAM ↔ SPU transfer FIFO (sync 1 slice) |
| `5` | `PIO` | Expansion port |
| `6` | `OTC` | Ordering-table reverse fill (`CHCR` writable mask `0x5100_0000`, `BACKWARD` fixed to `1`) |

- **Timing constants:** `DMA_GAP = 10` CPU cycles between blocks/list nodes, `NODE_OVERHEAD = 4` cycles per linked-list node, and `CHOP_OVERHEAD = 6` cycles per chopped burst window.
- **Arbitration:** Channels obey `DPCR` enable bits and priorities; transfer completion sets `DICR` channel IRQ flags and drives `IRQ3` when enabled.

## Root-Counter Timers (`src/timers.rs`) and Interrupts (`src/irq.rs`)

### Timers (`0x1F80_1100`, `0x1F80_1110`, `0x1F80_1120`)

Three 16-bit counters evaluated lazily from their source tick counters:
- **Timer 0:** System clock or GPU dot clock; syncs against HBlank.
- **Timer 1:** System clock or HBlank; syncs against VBlank.
- **Timer 2:** System clock or system clock ÷ 8 (`Source::System8`); stop/free-run sync modes.
- Supports target compare (`CMP`, `ZRET`), overflow (`OVFL`), repeat (`REPT`), toggle vs pulse (`LEVL`), and reset-hold behavior (wrapping on target holds at `0` for an extra tick; wrapping on `0xFFFF` does not).

### Interrupt Controller (`0x1F80_1070` `I_STAT`, `0x1F80_1074` `I_MASK`)

Eleven active interrupt sources (`SOURCES = 0x7FF`): `0` VBlank, `1` GPU, `2` CD-ROM, `3` DMA, `4`–`6` Timers 0–2, `7` SIO0, `8` SIO1, `9` SPU, `10` Lightpen/PIO. `I_STAT` acknowledges bits on write-zero (`I_STAT &= value`), and `(I_STAT & I_MASK) != 0` drives `blackmoon-r3000`'s `CAUSE.IP2`.

## SIO0, Controllers, Multitap, and Memory Cards (`src/sio0.rs`, `src/pad.rs`, `src/multitap.rs`, `src/card.rs`)

- **SIO0 (`0x1F80_1040`–`0x1F80_104E`):** Serial port supporting controller address `0x01` and memory-card address `0x81` across ports 1 and 2 (`CTRL.PORT`). Default kernel baud rate is `0x0088` cycles/bit; `/ACK` asserts `ACK_AFTER = 412` cycles after transfer and holds low for `ACK_LOW = 100` cycles, raising `IRQ7` when `DSRIEN` is enabled.
- **Controllers (`src/pad.rs`):**
  - **Digital Pad (SCPH-1080):** ID `0x41`, 2 button bytes.
  - **DualShock (SCPH-1200):** Digital mode (`0x41`), analog mode (`0x73`, 2 button bytes + 4 axis bytes), configuration mode (`0xF3` via command `0x43`), analog LED lock, motor mapping (`0x4D`), and dual-motor rumble output.
- **Multitap (`src/multitap.rs`):** ID `0x80` on port 1/2, routing single-slot and 4-slot (`A`–`D`) combined controller reads plus per-slot memory cards.
- **Memory Cards (`src/card.rs`):** 128 KiB (`0x2_0000` bytes, 1024 × 128-byte sectors) per slot implementing read (`0x52`), write (`0x57`), and get-ID (`0x53`) commands, `FLAG_NEW` dirty/fresh bit, and host persistence. On-card filesystem structures are shared in [`emusen-psx-card`](../../Shared/MediumFormats.md).

## Exact Idle Skipping (`src/idle.rs`)

Diamond accelerates busy-wait loops without altering observable state or cycle counts:
- `prove` inspects candidate loops (backward branch within `SPAN = 0x400`, turn length `<= MAX_PATH = 64` instructions) to prove that every instruction in a turn is side-effect-free with respect to registers/RAM outside the loop invariant and exits only when a scheduled hardware event fires.
- Once two consecutive turns execute in identical cycle counts, `Idle::arrive` advances `Board::now` by whole loop iterations up to the next scheduled event boundary, leaving CPU registers, caches, and device timelines identical to instruction-by-instruction stepping.

## Debugger Spaces, State Format, and Media Support

- **Memory spaces (`src/machine.rs`):** `1` `RAM` (2 MiB), `2` `SCRATCH` (1 KiB), `3` `VRAM` (1 MiB), `4` `SPURAM` (512 KiB), `5` `BIOS` (512 KiB, read-only), `6` `TTY` (64 KiB DUART capture buffer, read-only).
- **Save states (`src/state.rs`):** Magic `"DMND"`, `STATE_VERSION = 10` (reads versions `8..=10`), storing CPU, GTE, bus/timers/DMA/IRQ/SIO0/CD-ROM/GPU/MDEC/SPU registers, all writable RAMs, TTY buffer, disc fingerprint (rejecting mismatched discs with `WRONG_MEDIA`), and the current display picture. When shared memory cards are configured outside per-game states, the card slot block carries tag `"SHRD"`.
- **Media & boot:** Boots CD images via [`emusen-disc`](../../Shared/Disc.md), side-loads PS-EXE files via the `0x8003_0000` mid-boot hook, and supports empty-drive startup into [`diamond-shell`](Diamond_OpenKernel.md).
