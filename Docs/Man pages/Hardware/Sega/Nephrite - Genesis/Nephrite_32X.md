# Nephrite — Sega 32X Attachment (Stub Status)

Reference for the Sega 32X attachment boundary (`src/s32x/mod.rs`) in `nephrite` (`EmuSen/Cores/Sega/Nephrite - Genesis/`).

> [!IMPORTANT]
> Sega 32X emulation is **not yet implemented**. Both the 32X attachment (`src/s32x/mod.rs`) and its shared Hitachi SH-2 CPU crate ([`beryl-sh2`](../Beryl-HW/Beryl_Cpus.md#beryl-sh2), `BUILT = false`) are architectural stubs. The `32x` system descriptor in `src/v1.rs` is marked `development: true` so frontends do not expose it to players, and 32X images currently advance frame counts without executing 32X hardware.

---

## Stub Contract (`src/s32x/mod.rs`)

`S32x` implements `Attachment`, `AttachmentState`, and `AttachmentView` from `src/interface.rs` behind the `s32x` Cargo feature (enabled by default and verified independently by `src/boundary.rs`):

- **Bus behaviour (`Attachment`)**:
  - `EXPANSION` is `false` (the 32X sits in the cartridge slot rather than the side expansion port).
  - `read8` and `read16` return `Answer::Pass`, `write` returns `false`, `run_to` is a no-op, and `sound` returns `None`.
- **Save state (`AttachmentState`)**: `type Saved = ()`; writes and reads no attachment register block (its memory spaces are handled through `AttachmentView`).
- **Reserved Memory Spaces (`S32X_SPACES = 32..48`)**:

| Space ID | Name | Size | Status |
|---|---|---|---|
| `34` | `SDRAM` | `256 KiB` (`0x4_0000`) | Allocated zeroed buffer |
| `35` | `FRAMEBUFFER` | `256 KiB` (`0x4_0000`) | Allocated zeroed buffer |
| `36` | `PALETTE` | `512 bytes` | Allocated zeroed buffer |

- **Reserved Processor IDs (`S32X_PROCESSORS = 3..5`)**: `MSH2 = 3` (Master SH-2) and `SSH2 = 4` (Slave SH-2).
- **Media & Firmware Metadata**:
  - Recognizes `.32x` images (`EXTENSIONS = [".32x"]`) and cartridge headers whose system field contains `"32X"` (`is_32x`).
  - Declares the three optional 32X boot ROM firmware entries (`boot_firmware`):
    - `32X_G_BIOS.BIN` — 68000 boot ROM (`256` bytes)
    - `32X_M_BIOS.BIN` — Master SH-2 boot ROM (`2,048` bytes)
    - `32X_S_BIOS.BIN` — Slave SH-2 boot ROM (`1,024` bytes)
  - All three currently report `Replacement::None` because open 32X boot replacements are not yet written.

## Unbuilt Components

The following subsystems are not yet implemented:

- Hitachi SH7604 instruction execution, 4 KiB 4-way cache, and on-chip peripherals in `beryl-sh2` (`BUILT = false`).
- 32X system registers (`$A15100`–`$A151FF`), communication ports, DREQ/FIFO DMA, PWM audio, and dual-SH-2 bus arbitration.
- 32X Super VDP (packed-pixel, direct-colour, and run-length modes, frame buffer swapping, and Genesis VDP video mixing).
- Open 68000, Master SH-2, and Slave SH-2 boot ROM replacements.
