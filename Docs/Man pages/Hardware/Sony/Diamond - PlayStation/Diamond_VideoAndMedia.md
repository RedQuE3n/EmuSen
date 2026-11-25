# Diamond Video, Media, Audio, and Enhancement Pipeline (`diamond`, `precision`, `renderer`)

Crates:
- `EmuSen/Cores/Sony/Diamond - PlayStation/src/{gpu,cdrom,mdec,spu}/` — accurate PlayStation hardware subsystems.
- `EmuSen/Cores/Sony/Diamond - PlayStation/precision/` — PGXP-style sub-pixel vertex & depth side-channel.
- `EmuSen/Cores/Sony/Diamond - PlayStation/renderer/` — Vulkan compute multi-resolution renderer over [`emusen-gpu`](../../Shared/Gpu.md).

## GPU (`src/gpu/`)

### VRAM, Command FIFO, and Registers

- **VRAM (`src/gpu/vram.rs`):** `1024 × 512` 16-bit halfwords (`1 MiB`), supporting masked fills (`GP0(02h)`), VRAM-to-VRAM blits (`GP0(80h)`), CPU-to-VRAM writes (`GP0(A0h)`), and VRAM-to-CPU reads (`GP0(C0h)`) with hardware parameter masking and edge wrapping.
- **Command processor & FIFO (`src/gpu/mod.rs`):** `GP0` (`0x1F80_1810`) feeds a `16`-word FIFO (`FIFO_WORDS = 16`). Commands execute whole when their required words arrive and mark the GPU busy for their modelled drawing duration (`src/gpu/timing.rs`, measured in `1/256` GPU clock cycles converted to CPU cycles). `GP1` (`0x1F80_1814`) handles reset (`STATUS_RESET = 0x1480_2000`), display enable/range/mode, DMA direction, and `GPUREAD` queries.
- **GPU revisions:** Supports both `Pin208` (early 208-pin GPU, optional 2 MiB texture-page bit via `GP1(09h)`) and `Pin160` (later 160-pin GPU) models (`src/machine.rs`).

### Rasterizer, Textures, and Display Scanout

- **Rasterizer (`src/gpu/draw.rs`, `src/gpu/raster.rs`):** Draws flat and Gouraud triangles/quads, monochrome and textured rectangles/sprites, lines and polylines, applying drawing-area clipping (`GP0(E3h)`/`E4h`), drawing offset (`GP0(E5h)`), 4×4 Bayer dithering, 4 semi-transparency blend modes (`0.5B+0.5F`, `B+F`, `B-F`, `B+0.25F`), and mask-bit set/check (`GP0(E6h)`).
- **Texture & CLUT caches (`src/gpu/texture.rs`):** Models the `2 KiB` texture cache and CLUT cache for 4-bit and 8-bit paletted pages (plus direct 15-bit pages and `GP0(E2h)` texture window masking/offset). Stale cached texels/CLUTs remain visible until missed or flushed by `GP0(01h)` / CLUT reload, matching hardware behavior.
- **Line-by-line display scanout (`src/gpu/display.rs`):** Reads VRAM line-by-line as `Video` reaches each scanline, respecting `GP1(05h)` start coordinates, `GP1(06h)` horizontal display range (`(((X2 - X1) / dot_clock) + 2) & !3`), `GP1(08h)` dot clock dividers (`256`/`320`/`368`/`512`/`640`), 15-bit and 24-bit color depth, and 480i/576i field interleaving (up to `MAX_WIDTH = 640`, `MAX_HEIGHT = 576`).

## Precision Layer and Vulkan Compute Renderer (`precision/`, `renderer/`, `v1/`)

All visual enhancements run as a non-invasive `Layer` (`src/layer.rs`) alongside the accurate software GPU. Nothing produced by `precision` or `renderer` ever feeds back into PlayStation VRAM, CPU reads, or deterministic emulation state.

### Core ABI v1 Enhancement Settings (`v1/src/lib.rs`)

| Key | Setting | Values | Effect |
|---|---|---|---|
| `precision` | Precise geometry | Switch (`false`/`true`) | Sub-pixel vertex positions + perspective-correct texturing |
| `resolution` | Internal resolution | `1`, `2`, `3`, `4` | Vulkan compute upscale (`1×`–`4×`, up to `2560 × 2304`) |
| `colour` | True colour | Switch (`false`/`true`) | 8-bit-per-channel shading/blending without 5-bit dither |
| `filtering` | Texture filtering | `off`, `3d`, `all` | Bilinear texel filtering on 3D polygons only or all draws |

When all four settings are at hardware defaults (`false`, `1`, `false`, `off`), the enhancement renderer is inactive and the software GPU produces the output directly.

### Precision Side-Channel (`precision/src/lib.rs`)

Tracks untruncated 3D geometry through the CPU and memory:
- Receives `blackmoon_gte::Projection` records on every `RTPS`/`RTPT` into a `32,768`-entry ring (`RING = 32 * 1024`).
- Receives `blackmoon_r3000::Move` events (`Load`, `Store`, `FromCop2`, `ToCop2`, `LoadCop2`, `StoreCop2`, `Copy`, `Shift`) shadowing GPRs, COP2 data registers, 512K RAM words, and 256 scratchpad words.
- Matches incoming `GP0` vertex words (and DMA2 transfers) to their original camera-space projection, emitting `Fraction { dx, dy, z }` (offsets in `1/256`-pixel sub-pixels within `GUARD = 2.0` pixels of the console integer vertex, plus 12-fractional-bit camera depth `z`).

### Vulkan Compute Renderer (`renderer/src/`)

- **Compute pipelines (`renderer/src/device.rs`):** Runs four SPIR-V compute shaders (`walk.spv`, `scan.spv`, `clut.spv`, `copy.spv`) on `emusen-gpu` with double-buffered staging (`RECORDS = 32K`, `ENTRIES = 1M`, `TILE_RECORDS = 128K`, `SLOTS = 4096`, `MAX_SCALE = 4`).
- **Host device lend (Core ABI 1.9):** Accepts a host-lent Vulkan device (`DeviceLend`) at core creation so frames can be presented directly on the same GPU without CPU readback latency.
- **State loading & settling (Core ABI 1.5):** The renderer stores no data in `DMND` save states. On state load or reset, it seeds its scaled VRAM from the machine's canonical 1× VRAM; pictures settle as the game redraws (`SETTLES = 60` frames generally, `SETTLES_TRUE_COLOUR = 240` frames when true-color feedback effects are active).

## CD-ROM Controller (`src/cdrom/`)

Backed by [`emusen-disc`](../../Shared/Disc.md) for CUE/BIN, ISO, WAV, CHD, subchannel Q, and `.sbi` protection data.

- **Ports & interrupts (`src/cdrom/mod.rs`, `commands.rs`):** Implements the 4-bank register window (`0x1F80_1800`–`0x1F80_1803`), 16-byte parameter/response FIFOs, `INT1`–`INT5` delivery, and DMA channel 3 sector transfers.
- **Drive timing (`src/cdrom/drive.rs`):** Single speed = `75` sectors/s (`SECTOR_SINGLE = 451,584` CPU cycles), double speed = `150` sectors/s (`SECTOR_DOUBLE = 225,792` CPU cycles). Models spindle spin-up, distance-based `SeekL`/`SeekP` delays, pause head step-back, `Setmode` flags (`CDDA`, `AUTO_PAUSE`, `REPORT`, `FILTER`, `IGNORE`, `WHOLE`, `XA`, `DOUBLE`), lid/shell-open state, `GetID` SCEx region response, and `GetlocL`/`GetlocP`/`GetTN`/`GetTD`.
- **XA-ADPCM (`src/cdrom/xa.rs`):** Decodes Mode 2 Form 2 XA audio sectors (18 × 128-byte sound groups; 4-bit or 8-bit, mono or stereo, `37,800 Hz` or `18,900 Hz`) using the 4 ADPCM filter pairs (`POS = [0, 60, 115, 98]`, `NEG = [0, 0, -52, -55]`) and psx-spx's 7-phase 29-tap `ZIGZAG` FIR resampler to `44,100 Hz`.
- **Audio output (`src/cdrom/audio.rs`):** Buffers `44,100 Hz` CD-DA and XA-ADPCM frames in an `8,192`-frame stereo ring routed through the 2×2 `ATV0`–`ATV3` volume matrix (latched by `ADPCTL.CHNGATV`) into the SPU CD input.

## Macroblock Decoder — MDEC (`src/mdec/`)

- **Registers & FIFOs (`src/mdec/mod.rs`):** `MDEC0` (`0x1F80_1820`) accepts `Decode`, `Quant` (luminance + optional chrominance 64-byte quantization tables), and `Scale` (64-halfword IDCT scale table); `MDEC1` (`0x1F80_1824`) exposes status and soft reset. Input FIFO holds `IN_WORDS = 32` words (fed by DMA0); output FIFO holds `OUT_WORDS = 17` words (drained by DMA1, up to `BLOCK_WORDS = 48` words per 24-bit macroblock).
- **Decompression & timing (`src/mdec/decode.rs`):** Decodes RLE halfwords (terminated by `0xFE00`), de-quantizes in zig-zag order (`ZIGZAG`/`ZAGZIG`), runs the 2D IDCT using the uploaded scale matrix, converts YCbCr to 4-bit, 8-bit, 15-bit (with optional bit-15 mask set), or 24-bit RGB output, and charges `timing::BLOCK = 512` CPU cycles per decoded 8×8 block.

## Sound Processing Unit — SPU (`src/spu/`)

Uses [`blackmoon-spu`](../BlackMoon-HW/BlackMoon_SPU.md) for all 24 voices, ADSR envelopes, volume sweeps, noise LFSR, pitch modulation, and half-rate 39-tap reverb. Diamond's `src/spu/` adds the PlayStation-specific wrapper:

- **Sound RAM & registers (`0x1F80_1C00`–`0x1F80_1FFF`):** `512 KiB` sound RAM, per-voice registers, `MVOL`, `EVOL`, `KON`, `KOFF`, `PMON`, `NON`, `EON`, `ENDX`, `ESA`, `IRQA`, `TSA`, `DATA`, `ATTR` (`SPUCNT`), `RAM_CTRL`, `STATX`, `AVOL` (CD/external volume), `MVOLX`, and 32 reverb parameter registers.
- **Transfer FIFO & DMA4:** `32`-halfword (`16`-word) FIFO transferring at `CYCLES_PER_HALFWORD = 32` CPU cycles/halfword (one spare RAM slot per voice cycle) for both manual CPU writes and DMA4 reads/writes.
- **Capture buffers & IRQ9:** Writes CD Left (`0x0000`–`0x03FF`), CD Right (`0x0400`–`0x07FF`), Voice 1 (`0x0800`–`0x0BFF`), and Voice 3 (`0x0C00`–`0x0FFF`) capture buffers every sample; triggers `IRQ9` via `blackmoon_spu::Touch` when any voice, reverb, capture, or transfer access matches `IRQA` (in 8-byte units) while `SPUCNT.IRQ` is enabled.
- **Mix:** Sums dry voice output, reverb return, and CD audio input (`ATV`-scaled) at `44,100 Hz` (`CYCLES_PER_SAMPLE = 768` CPU cycles) into stereo `i16` samples.
