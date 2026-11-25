# Sapphire: GIF, VIF, Graphics Synthesizer, PCRTC, and IPU

Hardware reference for the Graphics Interface (GIF), Vector Interfaces (VIF0/VIF1), Graphics Synthesizer (GS) memory layout and rendering pipelines (software workers and Vulkan compute), Programmable CRT Controller (PCRTC), and Image Processing Unit (IPU) in `sapphire`.

---

## 1. Graphics Interface (GIF)

The GIF (`src/gif/mod.rs`, `src/gif/packing.rs`) sits between the EE/VU/VIF subsystems and the GS host interface, mapped at registers `0x1000_3000..0x1000_3100` (`CTRL`, `MODE`, `STAT`, `TAG0`–`TAG3`, `CNT`, `P3CNT`, `P3TAG`) and a 16-qword PATH3 FIFO mirrored over `0x1000_6000..0x1000_7000`.

### 1.1 Paths, Arbitration, and Masking

Three input paths feed the GIF in strict priority order:

| Path | Source | Notes |
| --- | --- | --- |
| **PATH1** | VU1 `XGKICK` (`vu1.data`, wrapping at 1024 qwords) | Highest priority; queued via `gif::Path1Port` |
| **PATH2** | VIF1 `DIRECT` and `DIRECTHL` | Middle priority; `DIRECTHL` does not preempt an interrupted PATH3 |
| **PATH3** | EE DMAC channel 2 (and EE 128-bit stores to `0x1000_6000`) | Lowest priority; buffered through a 16-qword FIFO (`DEPTH = 16`) |

- **Packet Ownership**: Once granted (with 1 bus cycle of arbitration overhead), a path holds the GIF from its opening GIFtag until the final qword of a GIFtag with `EOP = 1`.
- **PATH3 Masking**: PATH3 is excluded from arbitration while `GIF_MODE.M3R` or VIF1's `MSKPATH3` (`GIF_STAT.M3P`) is set.
- **Intermittent Mode (`GIF_MODE.IMT`)**: During PATH3 `IMAGE` transfers, every 8-qword slice boundary yields to waiting PATH1 or `DIRECT` requests, latching `GIF_STAT.IP3`, `P3CNT`, and `P3TAG` so PATH3 resumes seamlessly once higher-priority packets finish.

### 1.2 GIFtag and Transfer Formats (`src/gif/packing.rs`)

Each `Packet` parses a 128-bit GIFtag (`NLOOP` [14:0], `EOP` [15], `PRE` [46], `PRIM` [57:47], `FLG` [59:58], `NREG` [63:60] where `0` means `16`, and 16 × 4-bit register descriptors in bits `[127:64]`). Reading any GIFtag initializes the internal GIF `Q` register to `1.0` (`0x3F80_0000`). If `NLOOP > 0`, `FLG == PACKED`, and `PRE == 1`, `PRIM` is written immediately.

| Format (`FLG`) | Rate (bus cycles / qword) | Behavior |
| --- | --- | --- |
| `PACKED` (`0`) | 1 | Unpacks 128-bit qwords per 4-bit descriptor (`PRIM`, `RGBAQ` using GIF `Q`, `ST` updating GIF `Q`, `UV`, `XYZF2`, `XYZ2`, `TEX0_1/2`, `CLAMP_1/2`, `FOG`, `XYZF3`, `XYZ3`, `A+D` writing register `[71:64]` with data `[63:0]`, `NOP`) |
| `REGLIST` (`1`) | 1 per 64-bit half (up to 2) | Writes two packed 64-bit register values per qword; descriptors `0xE` and `0xF` are no-ops |
| `IMAGE` (`2` or `3`) | 2 | Streams two 64-bit halves per qword directly to `HWREG` (`0x54`) |

---

## 2. Vector Interfaces (VIF0 and VIF1)

`src/vif/mod.rs` and `src/vif/unpack.rs` implement VIF0 (DMAC channel 0, registers `0x1000_3800`, 8-qword FIFO at `0x1000_4000`) and VIF1 (DMAC channel 1, registers `0x1000_3C00`, 16-qword FIFO at `0x1000_5000`).

### 2.1 VIFcode Pipeline, Double Buffering, and Reverse Transfer

- **Registers**: `STAT`, `FBRST` (`RST`, `FBK`, `STP`, `STC`), `ERR` (`MII`, `ME0`, `ME1`), `MARK`, `CYCLE` (`CL`/`WL`), `MODE`, `NUM`, `MASK`, `CODE`, `ITOPS`, `ITOP`, `R0`–`R3`, `C0`–`C3`; VIF1 additionally provides `BASE`, `OFST`, `TOPS`, and `TOP`.
- **DMAtag TTE**: When a DMAC chain transfers its tag (`tag == true`), only the upper 64 bits (words 2 and 3) enter the VIFcode stream.
- **Commands**: `NOP`, `STCYCL`, `OFFSET`, `BASE`, `ITOP`, `STMOD`, `MSKPATH3`, `MARK`, `FLUSHE`, `FLUSH`, `FLUSHA`, `MSCAL`, `MSCALF`, `MSCNT`, `STMASK`, `STROW`, `STCOL`, `MPG` (waits for VU ready, then writes 64-bit instruction pairs into VU micro memory), `DIRECT`/`DIRECTHL` (forwards qwords to GIF PATH2), and `UNPACK`.
- **Interrupt & Stall (`i` bit)**: Unless masked by `ERR.MII`, a VIFcode with bit 31 set raises the VIF interrupt and stalls at the next VIFcode (`STAT.INT` and stall flags cleared by `FBRST.STC`), except `MARK`, which raises `STAT.MRK` without stalling the next command.
- **Double Buffering (VIF1)**: `BASE` and `OFST` configure two VU1 data buffers tracked by `STAT.DBF`. Executing `MSCAL`/`MSCALF`/`MSCNT` copies `TOPS` -> `TOP` and `ITOPS` -> `ITOP` (visible to VU1 via `XTOP`/`XITOP`), toggles `DBF`, and sets `TOPS = BASE + (DBF ? OFST : 0)`.
- **Local-to-Host Reverse Path**: When `VIF1_STAT.FDR` is set, VIF1 pulls 128-bit qwords from the reversed GS host interface (`Gif::host_read`) for DMAC channel 1 memory writes.

### 2.2 UNPACK Decompression and Write Masking (`src/vif/unpack.rs`)

- **Formats (`vn` [3:2], `vl` [1:0])**: Supports `S-32/16/8`, `V2-32/16/8`, `V3-32/16/8`, `V4-32/16/8`, and `V4-5` (`vn = 3, vl = 3`: 16-bit RGBA5551 expanded to `[r<<3, g<<3, b<<3, a<<7]`). `USN` selects zero-extension vs sign-extension for 8-bit and 16-bit elements. `S` broadcasts `[x, x, x, x]`, `V2` forms `[x, y, x, y]`, and `V3` forms `[x, y, z, 0]`.
- **CYCLE Skipping / Filling**: `WL == 0` acts as `256` (filling mode). When `CL >= WL` (skipping), every write consumes a packet vector and advances by `CL` qwords every `WL` writes. When `CL < WL` (filling), the first `CL` writes of each `WL`-qword block read from the packet and the remaining `WL - CL` writes emit fill data from `ROW`/`COL`/`MASK`.
- **MASK & MODE**: With the `m` bit set, `MASK` supplies a 2-bit per-lane specifier for write cycles `0..=3` (clamped at row 3 for cycles `>= 4`): `0` writes decompressed data (optionally modified by `MODE`: `0` direct, `1` offset `+ ROW`, `2` difference `+ ROW` updating `ROW`, `3` direct updating `ROW`), `1` writes `ROW`, `2` writes `COL[row]`, and `3` masks the lane (preserving existing VU data memory). On VIF1, `FLG` adds `TOPS` to the destination qword address.

---

## 3. Graphics Synthesizer (GS)

`src/gs/` implements the 4 MiB embedded DRAM (`local.rs`, `psm.rs`), general and privileged registers (`mod.rs`), host/local transfers (`transfer.rs`), drawing kick and vertex queue (`draw.rs`), exact integer rasteriser (`raster.rs`), pixel pipeline (`pixel.rs`), texture/CLUT/LOD/fog engine (`texture.rs`, `exact.rs`), multi-threaded CPU workers (`workers.rs`), and Vulkan compute renderer (`device/`).

### 3.1 Local Memory Swizzling (`src/gs/psm.rs`, `src/gs/local.rs`)

GS local memory holds `WORDS = 1 << 20` 32-bit words (4 MiB), organized into `PAGES = 512` pages of 2048 words (32 blocks × 64 words; each block has 4 columns × 16 words). Every pixel coordinate `(x, y)` resolves through precomputed static page tables (`CT32`, `Z32`, `CT16`, `CT16S`, `Z16`, `Z16S`, `T8`, `T4`) shared by all transfers, rasterisers, and the PCRTC:

| Format | Code | Bits | Page (px) | Block (px) | Column (px) | Block Order | Storage Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `PSMCT32` | `0x00` | 32 | 64 × 32 | 8 × 8 | 8 × 2 | `Ct32` | Full 32-bit RGBA |
| `PSMCT24` | `0x01` | 24 | 64 × 32 | 8 × 8 | 8 × 2 | `Ct32` | Low 24 bits (`[23:0]`), high byte untouched |
| `PSMCT16` | `0x02` | 16 | 64 × 64 | 16 × 8 | 16 × 2 | `Ct16` | 2 pixels per 32-bit word |
| `PSMCT16S` | `0x0A` | 16 | 64 × 64 | 16 × 8 | 16 × 2 | `Ct16s` | Alternate block permutation |
| `PSMT8` | `0x13` | 8 | 128 × 64 | 16 × 16 | 16 × 4 | `Ct32` | 4 pixels/word; rows 2–3 (even col) / 0–1 (odd col) rotated by 4 px |
| `PSMT4` | `0x14` | 4 | 128 × 128 | 32 × 16 | 32 × 4 | `Ct16` | 8 pixels/word; same column-parity nibble rotation |
| `PSMT8H` | `0x1B` | 8 | 64 × 32 | 8 × 8 | 8 × 2 | `Ct32` | Stored in bits `[31:24]` of `PSMCT32` words |
| `PSMT4HL` | `0x24` | 4 | 64 × 32 | 8 × 8 | 8 × 2 | `Ct32` | Stored in bits `[27:24]` of `PSMCT32` words |
| `PSMT4HH` | `0x2C` | 4 | 64 × 32 | 8 × 8 | 8 × 2 | `Ct32` | Stored in bits `[31:28]` of `PSMCT32` words |
| `PSMZ32` | `0x30` | 32 | 64 × 32 | 8 × 8 | 8 × 2 | `Z32` | `Ct32` block index XOR `0x18` |
| `PSMZ24` | `0x31` | 24 | 64 × 32 | 8 × 8 | 8 × 2 | `Z32` | Low 24 bits (`[23:0]`), high byte untouched |
| `PSMZ16` | `0x32` | 16 | 64 × 64 | 16 × 8 | 16 × 2 | `Z16` | `Ct16` block index XOR `0x18` |
| `PSMZ16S` | `0x3A` | 16 | 64 × 64 | 16 × 8 | 16 × 2 | `Z16s` | `Ct16s` block index XOR `0x18` |

For worker synchronization and GPU dirty tracking, each of the 512 pages is split into two lanes (`UNITS = 1024`): lane 0 covers word bits `[23:0]` (`PSMCT24`/`PSMZ24`) and lane 1 covers word bits `[31:24]` (`PSMT8H`/`PSMT4HL`/`PSMT4HH`).

### 3.2 Privileged Registers, Events, and Transfers

- **Privileged Map (`0x1200_0000..0x1201_0000`)**: Decoded by `GA[6:0] = ((paddr >> 12 & 1) << 6) | (paddr >> 4 & 0x3F)`. Accesses in `0x1201_0000..0x1400_0000` raise a bus error. `CSR` carries `SIGNAL`, `FINISH`, `HSINT`, `VSINT`, `EDWINT`, `FLUSH`, `RESET`, `FIELD`, `FIFO_EMPTY`, `REV = 0x1B`, and `ID = 0x55`; `IMR` resets to `0x7F00` (all masked). Writing a second `SIGNAL` while `CSR.SIGNAL` is still set stalls the GS host interface until cleared.
- **Transfers (`src/gs/transfer.rs`)**: `TRXDIR` starts Host-to-Local (`0`, via `HWREG`), Local-to-Host (`1`, via reversed host interface when `BUSDIR = 1`), or immediate Local-to-Local copies (`2`, honoring `TRXPOS` traversal order). Transfer streams pack pixels tightly from bit 0 across 64-bit boundaries with coordinates wrapping modulo `2048`.

### 3.3 Rasteriser, Texturing, and Pixel Pipeline

- **Vertex Queue & Primitives (`draw.rs`, `raster.rs`)**: Supports Point, Line, LineStrip, Triangle, TriangleStrip, TriangleFan, and Sprite (Prohibited `111` advances the queue without drawing). Coordinates are 12.4 fixed-point (sixteenths of a pixel) with `XYOFFSET_1/2` subtracted at kick time. Coverage is exact integer arithmetic: top-left edge rule for triangles, `[x0, x1) × [y0, y1)` for sprites, diamond-exit rule (`|dx| + |dy| < 1/2`) for lines, and half-up rounding for points.
- **Texturing, CLUT, and Fog (`texture.rs`, `exact.rs`)**:
  - **CLUT Buffer**: 512 × 16-bit internal temporary buffer (`CBP0`/`CBP1` cache tags). `PSMCT32` CLUTs split low/high halves across entries `0..255` and `256..511`; supports CSM1 (`8×2` / `16×16` interleaved blocks) and CSM2 (linear row via `TEXCLUT`), plus `TEX1.MTBA` automatic `MIPTBP1` generation.
  - **Exact Texture Coordinates & LOD**: `src/gs/exact.rs` computes perspective-corrected `S/Q`, `T/Q`, and mipmap `LOD` without intermediate rounding using fixed-size 128-bit fast paths and 768-bit (`12 × u64`) multi-precision fallbacks. Supports `REPEAT`, `CLAMP`, `REGION_CLAMP`, and `REGION_REPEAT`, point/bilinear/trilinear mipmap filters, `TEXA` alpha expansion (`AEM`/`TA0`/`TA1`), texture functions (`MODULATE`, `DECAL`, `HIGHLIGHT`, `HIGHLIGHT2`), and per-pixel fog blending with `FOGCOL`.
- **Pixel Pipeline (`pixel.rs`)**: Executes in hardware order:
  1. **Scissoring (`SCISSOR_1/2`)** and **Scanline Mask (`SCANMSK`)**.
  2. **Texture Mapping**, **Fog**, and **AA1 Coverage** (coverage replaces alpha unless `ABE` is set and `A != 0x80`; Z writes are suppressed below full coverage `0x80`).
  3. **Alpha Test (`TEST.ATE`/`ATST`/`AREF`/`AFAIL`)**: Supports `KEEP`, `FB_ONLY`, `ZB_ONLY`, and `RGB_ONLY` (behaves as `FB_ONLY` on non-32-bit targets).
  4. **Destination Alpha Test (`DATM`)** and **Depth Test (`ZTE`/`ZTST`/`ZMSK`)**: Clamps Z to format maximum (`z_max`).
  5. **Alpha Blending (`ALPHA_1/2`, `PABE`)**: `((A - B) * C >> 7) + D`, gated by `PABE` (`As & 0x80 != 0`) when enabled.
  6. **Dithering (`DTHE`/`DIMX`)**: Applied exclusively to 16-bit frame formats (`PSMCT16`/`PSMCT16S`), followed by **Color Clamp (`COLCLAMP`)** and **Alpha Correction (`FBA`)** (ORing `0x80` into alpha).
  7. **Format Conversion & Write Masks**: Packs `RGBA16` (`pack16`) and applies `FBMSK` (packed to 16-bit on 16-bit targets) and `ZMSK`.

### 3.4 Multi-Threaded CPU Workers and Vulkan Compute Renderer

Sapphire provides two parallel GS execution backends over the exact same 4 MiB local memory layout:

1. **Multi-Threaded CPU Workers (`src/gs/workers.rs`, shipping Core ABI default)**:
   - Controlled by the `gs_workers` Core ABI setting (`"auto"` selects `clamp(available_parallelism / 4, 1, 4)`; `"1"` runs single-threaded on the emulation thread; up to `MAX = 8` via `SAPPHIRE_GS_WORKERS`).
   - Publishes drawing kicks and 32-unit host-to-local upload batches into a 4096-slot lock-free ring with 256 CLUT snapshot slots. Workers partition rows into bands of 32 or 64 rows (aligned to page height) and enforce exact page-lane read/write dependencies (`local::Units`), guaranteeing bit-identical output to single-threaded execution.

2. **Vulkan Compute GS Renderer (`src/gs/device/`, `shaders/walk.comp` -> `shaders/walk.spv`)**:
   - Enabled via `SAPPHIRE_GS_DEVICE` (or `Gs::set_device` / `Gs::set_device_on`, accepting either an owned `emusen-gpu::GpuDevice` or a lent frontend device via `caps::BORROW`). Requires 64-bit shader integers (`device.int64()`).
   - **Tiled Compute Walk**: `Planner` (`device/plan.rs`) encodes primitives into 32-word records, 32-word environments, and 256-word CLUTs, grouping them into dependency stretches binned into `16 × 8` pixel tiles (one compute workgroup per tile, one invocation per pixel, records staged in 16-record shared-memory chunks). Specialization constants compile per-stretch shader classes (`HAS_TRIANGLES`, `HAS_SPRITES`, `HAS_LINES`, `HAS_TEXTURE`, `HAS_STQ`, `HAS_LOD`, `HAS_FOG`, `HAS_AA`, `EXACT_TEXTURE`) on demand.
   - **Host-Device Page Coherence**: The CPU keeps a 4 MiB mirror for CLUT loads, local-to-host downloads, PCRTC scanlines, and save states. Dirty CPU pages stage via `Op::Upload`; device-drawn lanes (`newer`) read back before CPU observation.
   - **Arithmetic & Feedback Tolerances**:
     - Coverage, Z interpolation, color shading, blending, dithering, and memory swizzling run in exact integer arithmetic matching the software rasteriser.
     - Single-precision texture coordinates use `SPV_KHR_float_controls` (round-towards-zero and flush-to-zero on 32-bit floats) with `precise` arithmetic to prevent FMA contraction; devices without `float_controls` automatically fall back to `EXACT_TEXTURE = true`. In single-precision mode, minor 1–2 channel differences can appear on isolated pixels of perspective-textured surfaces; setting `SAPPHIRE_GS_EXACT_TEXTURE` restores exact integer texture math.
     - **Self-Reading Feedback Draws** (primitives sampling the same pages they render into): By default, `Planner` emits `Op::Snapshot` to copy source pages to a device `snapshot` buffer so the stretch reads pre-stretch pixels deterministically (`SAPPHIRE_GS_ORDERED` instead applies records sequentially within one workgroup). Setting `SAPPHIRE_GS_EXACT` (`Gs::set_exact(true)`) falls back self-reading primitives to the CPU software rasteriser and enables `EXACT_TEXTURE = true`, making device output bit-identical to the software reference.

---

## 4. Programmable CRT Controller (PCRTC)

`src/pcrtc/mod.rs` and `src/pcrtc/timing.rs` generate all video timing signals (H-blank, V-blank, V-sync, `CSR.FIELD`) and assemble the RGBA8888 output frame for the frontend.

### 4.1 Video Modes and VCK Timing (`src/pcrtc/timing.rs`)

Timing runs on exact rational VCK-to-EE-cycle ratios (`clock::VCK_54`, `clock::VCK_74`) with ceiling conversion so rounding never accumulates across fields. Mode or `SMODE2.INT` changes latch at the next field start.

| Mode | VCK Clock | VCK / Line | Lines (Frame / Non-Int Field) | V-Blank / H-Blank / V-Sync | Default Picture | Aspect |
| --- | --- | --- | --- | --- | --- | --- |
| **NTSC** | 54 MHz (`VCK_54`) | 3432 | 525 / 263 | 22 lines / 552 VCK / line 4 | 640 × 448 (4 VCK/px) | 4:3 |
| **PAL** | 54 MHz (`VCK_54`) | 3456 | 625 / 314 | 26 lines / 576 VCK / line 4 | 640 × 512 (4 VCK/px) | 4:3 |
| **DTV 480p** | 54 MHz (`VCK_54`) | 1716 | 525 (progressive) | 45 lines / 276 VCK / line 4 | 640 × 480 (2 VCK/px) | 4:3 |
| **DTV 1080i** | 74.25 MHz (`VCK_74`) | 2200 | 1125 (interlaced) | 22 lines / 280 VCK / line 4 | 1920 × 1080 (1 VCK/px) | 16:9 |

### 4.2 Scanline Readout and Circuit Merge (`src/pcrtc/mod.rs`)

- **Scanline-Accurate Sampling**: Each active scanline is read from GS local memory at its scheduled EE cycle (`read_due`), reflecting mid-field `DISPFB`/`DISPLAY`/`PMODE`/`BGCOLOR` register updates from the next line onward. Output buffers cap at `MAX_WIDTH = 1920` and `MAX_HEIGHT = 1125`; interlaced fields weave into a single frame buffer.
- **Read Circuits 1 & 2**: Decode `DISPFB1/2` (`FBP`, `FBW`, `PSM`, `DBX`, `DBY`) and `DISPLAY1/2` (`DX`, `DY`, `MAGH`, `MAGV`, `DW`, `DH`). Supports `PSMCT32`, `PSMCT24`, `PSMCT16`, `PSMCT16S`, Z-buffer equivalents, and `PS-GPU24` (`0x12`, circuit 1 only, reading packed 24-bit RGB across `PSMCT16` storage). In `SMODE2.FFMD = 1` (FRAME mode), interlaced fields step buffer lines by `d / 2 / magv`.
- **Merge Circuit (`PMODE`)**: Computes `OUT2` (circuit 2 or `BGCOLOR`), selects background `D = SLBG ? BGCOLOR : OUT2`, and blends circuit 1 over `D` using `A = MMOD ? ALP : min(2 * A1, 255)` as `(S * A + D * (255 - A) + 127) / 255`. PCRTC feedback writeback (`EXTBUF`, `EXTDATA`, `EXTWRITE`) is not modelled.

---

## 5. Image Processing Unit (IPU)

`src/ipu/` implements the MPEG-2 / MPEG-1 macroblock decoder and pixel post-processor on EE DMAC channels 3 (`fromIPU`, 8-qword output FIFO at `0x1000_7000`) and 4 (`toIPU`, input FIFO at `0x1000_7010`), with registers `IPU_CMD` (`0x1000_2000`), `IPU_CTRL` (`0x1000_2010`), `IPU_BP` (`0x1000_2020`), and `IPU_TOP` (`0x1000_2030`).

### 5.1 Commands and Bitstream Engine (`src/ipu/mod.rs`, `bits.rs`, `vlc.rs`)

All ten hardware commands are implemented:

| Code | Command | Function |
| --- | --- | --- |
| `0` | `BCLR` | Clears input FIFO and internal bit buffer, advancing by `BP` bits |
| `1` | `IDEC` | Decodes intra-coded MPEG-2/MPEG-1 macroblocks (`Y`, `Cb`, `Cr` -> CSC -> `RGB32`/`RGB16`, optional `SGN` bit-flip and dither) until next start code |
| `2` | `BDEC` | Decodes a single intra or non-intra macroblock (6 × 8×8 blocks) to raw 16-bit IDCT residuals (`OUT_FIFO`) |
| `3` | `VDEC` | Decodes one VLC symbol (MBAI, MBtype, motion code, or CBP) into `IPU_CMD.DATA` |
| `4` | `FDEC` | Advances bit stream by `FB` bits and peeks next 32 bits into `IPU_CMD.DATA` |
| `5` | `SETIQ` | Loads 64-byte intra (`IQM = 0`) or non-intra (`IQM = 1`) quantisation matrix |
| `6` | `SETVQ` | Loads 16 × 16-bit RGB16 colour lookup table for `PACK` vector quantisation |
| `7` | `CSC` | Converts `MBC` raw 4:2:0 YCbCr (`RAW8`, 384 bytes/MB) macroblocks to `RGB32` (64 qwords) or `RGB16` (32 qwords) |
| `8` | `PACK` | Converts `MBC` `RGB32` macroblocks to `RGB16` (32 qwords) or 4-bit `INDX4` (8 qwords) via `VQ` |
| `9` | `SETTH` | Sets transparent (`TH0`) and translucent (`TH1`) alpha thresholds |

- **MPEG-2 / MPEG-1 Decoding**: Follows ITU-T H.262 Annex B VLC tables (`vlc.rs`), zigzag and alternate scans (`SCAN`), linear/nonlinear quantiser scales (`Q_SCALE`), saturation and mismatch control, and an Annex A reference IDCT (`idct.rs`, separable `f64` with exact 100-bit fixed-point cosine disambiguation for values within `2^-20` of half-integers).
- **Execution Behavior**: Commands advance symbol-by-symbol once at least 32 bits are present in the input buffer and pause whenever the 8-qword output FIFO is full; macroblock decoding latency itself is not timed, and an invalid VLC sets `IPU_CTRL.ECD` and aborts the command without outputting the failed macroblock. Command completion raises `INT_IPU` on the EE INTC.

### 5.2 Colour Space Conversion, Dithering, and Vector Quantisation (`src/ipu/csc.rs`)

- **CSC**: Fixed-point YCbCr-to-RGBA conversion (`Y3 = (0x95 * (Y - 16)) >> 6`, `Cr4 = (0xCC * (Cr - 128)) >> 6`, `Cr5 = (0x68 * (Cr - 128)) >> 6`, `Cb4 = (0x32 * (Cb - 128)) >> 6`, `Cb5 = (0x102 * (Cb - 128)) >> 6`, half-up rounded and clamped to `[0, 255]`). Pixels with `R, G, B < TH0` output `0x0000_0000`; pixels with `R, G, B < TH1` get `A = 0x40`, otherwise `A = 0x80`. `IDEC` with `SGN = 1` XORs RGB with `0x0080_8080` prior to dithering.
- **Dithering & RGB16**: Applies the 4 × 4 ordered dither matrix (`DITHER`, in half-step units), clamps to `[0, 255]`, truncates RGB to 5 bits each, and sets bit 15 (`A1`) when `A == 0x40`.
- **VQ (`INDX4`)**: Maps each 16-bit RGB16 pixel to the first of the 16 `SETVQ` CLUT entries minimizing the sum of squared 5-bit RGB channel differences, packing 32 pixels per qword (8 qwords per macroblock).
