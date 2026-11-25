# Mars — Reality Display Processor (RDP) and Video Interface (VI)

Live Rust core crate: `EmuSen/Cores/Nintendo/MarsRT - N64` (crate `marsrt`).

---

## 1. DP Command Interface and Command Stream (`src/memory/dp.rs`, `src/rdp/mod.rs`)

The Display Processor command interface is mapped at physical `0x0410_0000..0x0410_001F` (and reachable from the RSP as COP0 registers `$c8..$c15`):

| CPU Address | Offset | Register | Read | Write |
| --- | --- | --- | --- | --- |
| `0x0410_0000` | `0x00` | `DPC_START` | `start` | If `!start_valid`, sets `start = value & 0x00FF_FFF8`, `start_valid = true` |
| `0x0410_0004` | `0x04` | `DPC_END` | `end` | Sets `end = value & 0x00FF_FFF8`; if `start_valid`, loads `current = start` and clears `start_valid`; if `!freeze`, sets `running = true` and consumes command words |
| `0x0410_0008` | `0x08` | `DPC_CURRENT` | `current` | Ignored |
| `0x0410_000C` | `0x0C` | `DPC_STATUS` | `xbus(0) \| freeze(1) \| start_gclk(3) \| pipe_busy(5) \| buffer_ready(7=1) \| start_valid(10)` | Bits `1:0`: `1` clears / `2` sets `XBUS`; bits `3:2`: `1` clears (and resumes fetch) / `2` sets `FREEZE` |

- **Command source:** When `XBUS` is clear, 64-bit command words are fetched big-endian from RDRAM at `current`; when `XBUS` is set, they wrap within SP DMEM (`current & 0xFFF`). Each word advances `current += 8` until `current >= end` or `freeze` is set.
- **Command lengths (`command_length`):** Identified by `id = (word >> 56) & 0x3F`:
  - `0x08..=0x0F` (triangles): `4` edge words `+ 8` if shaded (`id & 4`), `+ 8` if textured (`id & 2`), `+ 2` if Z-buffered (`id & 1`) — `4` to `22` words total.
  - `0x24` (`TextureRectangle`), `0x25` (`TextureRectangleFlipped`): `2` words.
  - All other commands (`SetColorImage` `0x3F`, `SetMaskImage` `0x3E`, `SetTextureImage` `0x3D`, `SetCombine` `0x3C`, `SetEnvColor` `0x3B`, `SetPrimColor` `0x3A`, `SetBlendColor` `0x39`, `SetFogColor` `0x38`, `SetFillColor` `0x37`, `FillRectangle` `0x36`, `SetTile` `0x35`, `LoadTile` `0x34`, `LoadBlock` `0x33`, `SetTileSize` `0x32`, `LoadTlut` `0x30`, `SetOtherModes` `0x2F`, `SetPrimDepth` `0x2E`, `SetScissor` `0x2D`, `SetConvert` `0x2C`, `SetKeyR` `0x2B`, `SetKeyGB` `0x2A`, `SyncFull` `0x29`): `1` word.
- **`SyncFull` (`0x29`):** Clears `dp.running` and raises `MI_INTR_DP`.

### 1.1 Threaded RDP Execution and Multi-Worker Row Striping (`src/memory/dp_threads.rs`, `src/rdp/split.rs`)

When `ThreadedRdp` is enabled (default `true`), `dp_take_onto_thread` reads command words on the emulation thread, marks touched 4 KiB RDRAM pages (`PAGES = 2048`) in `marks` (write hazard) and `write_marks` (read hazard), and publishes words into a `65,536`-entry ring (`RING = 1 << 16`):
- **Exact synchronization:** Any CPU fetch/load/store, block validation, SI/AI/VI/SP DMA, or cheat access checks the single atomic page mark on its fast path and waits (`wait_read` / `wait_write` / `wait_range`) only when an in-flight RDP command touches that page. Optional byte-level verification (`VerifyRdp`, `EMUSEN_MARSRT_VERIFY_RDP=1`) asserts that every byte touched by the RDP was covered by a mark.
- **Multi-worker striping (`RdpWorkers`, `1..8`):** Each worker owns rows `y % workers == worker` (`src/rdp/split.rs`). Commands are classified at their final word (`classify`):
  - Primitives with cross-row dependencies (reading `COMBINED` from a prior pixel in 1-cycle or first-cycle of 2-cycle, 2-cycle first-blend reading memory alpha, spans extending past `color_image_width`, or overlapping color/depth buffers) serialize on the leader (`Step::Leader`).
  - `SetColorImage` and `SetMaskImage` synchronize all workers (`Step::All`); texture loads overlapping recently drawn regions join workers first (`Step::AllJoined`).

---

## 2. Triangle Rasterization and Edge Walking (`src/rdp/walker.rs`, `src/rdp/fill.rs`)

The edge walker steps major (`xh`, `dxhdy`) and minor (`xm`/`xl`, `dxmdy`/`dxldy`) edges across **quarter-pixel sub-scanlines** (`k = (yh & !3)..=(lower | 3)`):
- **Edge stepping:** `xh`, `xm`, `xl` are 28-bit signed 12.16 fixed-point values masked `& !1`; sub-scanline increments are `(dxdy >> 2) & !1`. The minor edge switches from `xm`/`dxmdy` to `xl`/`dxldy` at `k == ym`.
- **Scissor clipping (`SetScissor` `0x2D`):** `scissor_left/top/right/bottom` are 10.2 quarter-pixel coordinates (`0..4095`). `clip_edge` converts 12.16 `x` to eighth-pixels with a sticky LSB (`((x >> 1) & 0x1FFF) != 0`), clamping to `[scissor_left * 2, scissor_right * 2]`. Interlaced field scissoring (`scissor_field`, `scissor_keep_odd`) skips even or odd rows via `field_keeps`.
- **Attribute interpolation:** Eight 16.16 fixed-point attributes (`R, G, B, A, Z, S, T, W`) step down the major edge once per scanline by `attribute_de` and across each span pixel by `shade_step` (`dx & !0x1F`), `depth_step` (`dx`), and `texture_step` (`dx & !0x1F`). At `sample_sub` (`3` if `major_slope_negative == major_on_left`, else `0`), each row records `span_major_x` and sub-pixel-corrected starting `span_attributes`.
- **Rectangles (`FillRectangle`, `TextureRectangle`, `TextureRectangleFlipped`):** Walked through the same edge walker with `major_on_left = true` and zero slope. In `COPY` and `FILL` cycle modes, bottom/right edges are inclusive (`bottom |= 3`); in `1-CYCLE` and `2-CYCLE` modes they are exclusive.

---

## 3. Coverage and Sub-Pixel Correction (`src/rdp/coverage.rs`, `src/rdp/tables.rs`)

Each pixel samples **8 of 16 sub-pixels** on a 4×4 grid (two samples per sub-scanline, staggered `0xA` on even sub-lines and `0x5` on odd sub-lines):
- `row_coverage` builds an 8-bit coverage mask per pixel across `[left..=right]`. Coverage count is `mask.count_ones()` (`0..8`), and `coverage_bit` is bit 7 (`mask & 0x80 != 0`).
- **Partial-coverage correction (`COVERAGE_OFFSETS`):** For pixels with `coverage < 8`, shade and depth are shifted to the first covered sub-sample `(ox, oy)` using `shade_correct_dx/dy` and `depth_correct_dx/dy`.
- **Stored 3-bit coverage in RDRAM:**
  - **16-bit RGBA5551 (`color_image_size == 2`, `format == 0`):** Top bit in pixel LSB (`bit 0`), lower 2 bits in `rdram_hidden[word]`.
  - **16-bit IA (`format != 0`):** Bits `7:5` of the 16-bit pixel word (`hidden = 0`).
  - **32-bit RGBA8888 (`color_image_size == 3`):** Alpha bits `7:5` (`stored << 5`).
- **Coverage destination modes (`coverage_destination` bits 9:8 of `OtherModes`):**
  - `0` (`CLAMP`): `min(7, blend ? cov + mem : cov - 1)` (clamps on overflow bit 3).
  - `1` (`WRAP`): `(cov + mem) & 7`.
  - `2` (`ZAP`): `7`.
  - `3` (`SAVE`): `mem`.

---

## 4. Z-Buffer and Depth Comparison (`src/rdp/depth.rs`)

Depth buffers store 16-bit words in RDRAM plus 2 hidden bits per pixel (`rdram_hidden`), encoding an **18-bit Z value** and a **4-bit `delta_z` exponent**:
- **Compression (`compress_depth` / `decompress_depth`):** 18-bit `z` (`0..0x3FFFF`) packs into a 3-bit exponent of leading ones (`0..7`, bits `15:13`) and an 11-bit mantissa (bits `12:2`).
- **`delta_z` slope:** Computed from `|dz/dx| + |dz/dy|` via `normalize_delta_z` (next power of two, `1..0x8000`) or `SetPrimDepth` (`primitive_delta_z`) when `primitive_depth` is enabled. Its 4-bit log2 encoding (`delta_z_encoding`) stores top 2 bits in depth word bits `1:0` and low 2 bits in `rdram_hidden`.
- **Margin and blend decision:** `slope = highest_bit((delta_z | stored_slope) & 0xFFFF)` (with low-precision adjustment when stored exponent `< 3`), `margin = slope << 3`. `overflow = ((mem_cov + cov) & 8) != 0`; `farther = z + margin >= old`; `in_front = z < old`; `nearer = z - margin <= old`; `maximum = old == 0x3FFFF`.
- **Depth modes (`depth_mode` bits 11:10 of `OtherModes`):**

| Mode | Name | Survival Rule |
| --- | --- | --- |
| `0` | Opaque | `maximum \|\| (overflow ? in_front : nearer)` |
| `1` | Interpenetrating | If `in_front && farther && overflow`, scales `cov = ((((old >> shift) - (z >> shift)) & 0xF) * cov) >> 3` and passes; else Opaque rule |
| `2` | Translucent | `in_front \|\| maximum` |
| `3` | Decal | `farther && nearer && !maximum` |

Depth comparison also computes `blend_shift_a` / `blend_shift_b` (and `past_shift_a` / `past_shift_b` in 2-cycle mode) from `clamp(delta_z_encoded - stored_encoded, 0, 4)` when the blender selects memory alpha (`second_alpha == 1`).

---

## 5. Texture Memory (TMEM), Formats, Filtering, and LOD

### 5.1 TMEM Layout and Bank Interleaving (`src/rdp/texture_memory.rs`)

TMEM is **4 KiB** (`4,096` bytes = `2,048` 16-bit words) organized as 4 interleaved 16-bit banks in two 2 KiB halves:
- **Odd-row bank swizzle:** On odd texel rows (`t & 1 != 0`), bank pairs swap (`word_index ^ 2`, `byte_index ^ 4`), preventing bank conflicts during bilinear 2×2 fetches across adjacent scanlines.
- **Split layouts:**
  - **RGBA32 (`format == 0, size == 3`):** Stores `RG` in the lower 2 KiB (`0x000..0x3FF` words) and `BA` in the upper 2 KiB (`0x400..0x7FF` words).
  - **YUV (`format == 1`):** Deinterleaves chroma (`U/V`) into the lower 2 KiB and luma (`Y`) into the upper 2 KiB.
  - **TLUT palettes (`LoadTlut` `0x30`):** Each 16-bit palette entry replicates 4× across all four banks in the upper 2 KiB (`0x400..0x7FF`) so four simultaneous palette lookups never collide.
- **Load commands:** `LoadTile` (`0x34`) loads rectangular sub-regions row by row; `LoadBlock` (`0x33`) streams contiguous 8-byte chunks advancing `t` by `dxt` (`tile.th << 8`); `LoadTlut` (`0x30`) loads 16-bit palette entries. Four-bit `texture_image_size == 0` loads perform no transfer.

### 5.2 Tile Descriptors, Perspective Division, and Texel Formats (`src/rdp/textures.rs`, `src/rdp/tables.rs`)

Eight tile descriptors (`tiles[0..8]`) specify `format` (0=RGBA, 1=YUV, 2=CI, 3=IA, 4=I; formats `>= 5` read as I), `size` (0=4b, 1=8b, 2=16b, 3=32b), `line` (8-byte row stride), `memory` (TMEM base in 8-byte units), `palette` (4-bit CI4 bank), `sl/tl/sh/th` (10.2 tile bounds), and per-axis `clamp`, `mirror`, `mask`, and `shift`.

- **Perspective division (`divided_coordinates`):** When `OtherModes.perspective` (bit 51) is enabled, `s` and `t` multiply by a 32,768-entry piecewise-linear `DIVIDE_TABLE[w & 0x7FFF]` built from 64 reciprocal segments (`reciprocal_point(6) = 0x3A83` matching hardware ROM quirk), then clamp to 16-bit signed coordinates with overflow flags.
- **Coordinate transform order:**
  1. `shifted(coord, shift)`: right-shift if `shift < 11`, left-shift by `16 - shift` if `shift >= 11`.
  2. Subtract tile origin (`sl << 3`, `tl << 3`) and test `beyond` against `sh`/`th`.
  3. Clamp (always active when `mask == 0` or `clamp` is set) to `[0, ((sh >> 2) - (sl >> 2)) & 0x3FF]`.
  4. Mirror (if `mirror` is set and bit `min(mask, 10)` is 1, bitwise invert) and mask to `mask` bits (max 10 bits).
- **Channel expansion:** 5-bit RGBA5551 channels expand to 8 bits via `FIVE_TO_EIGHT` (`(v << 3) | (v >> 2)`). Paletted modes (`palette_enabled`, bit 47) index the upper 2 KiB TMEM as RGBA16 or IA16 (`palette_intensity_alpha`, bit 46).

### 5.3 3-Point Barycentric Filtering and YUV Conversion (`src/rdp/filter.rs`)

When `sample_four` (bit 45) is set, the RDP fetches a 2×2 texel neighborhood `(t0, t1, t2, t3)` and performs **N64 3-point barycentric filtering** instead of 4-point bilinear filtering:
- The 5-bit fractional coordinates `(s_frac, t_frac)` split the texel square along its diagonal: `upper = ((s_frac + t_frac) & 0x20) != 0`.
  - Lower triangle (`!upper`): `t0 + ((s_frac * (t1 - t0) + t_frac * (t2 - t0) + 0x10) >> 5)`.
  - Upper triangle (`upper`): `t3 + (((0x20 - s_frac) * (t2 - t3) + (0x20 - t_frac) * (t1 - t3) + 0x10) >> 5)`.
  - **Mid-texel average (`mid_texel`, bit 44):** When `s_frac == 0x10 && t_frac == 0x10`, returns `(t0 + t1 + t2 + t3 + 2) >> 2`.
- **YUV conversion (`SetConvert` `0x2C`, `K0..K5`):** Chroma channels (`U`, `V`) are half horizontal resolution (`s_frac_rg = (s_frac >> 1) | ((s & 1) << 4)`). When `bilinear` is clear (or `convert_one` is active in cycle 2), `converted` applies `R = Y + ((K0*V + 0x80) >> 8)`, `G = Y + ((K1*U + K2*V + 0x80) >> 8)`, `B = Y + ((K3*U + 0x80) >> 8)`.

### 5.4 Level of Detail and Mipmaps (`src/rdp/lod.rs`)

- **1-cycle LOD:** Measures horizontal texel movement (`max(|Δs|, |Δt|)` in 17-bit coordinates) between adjacent pixels (and wraps to the next drawn row at span end for long spans).
- **2-cycle LOD (`two_cycle_level_of_detail`):** Measures both horizontal (`+dx`) and vertical (`+dy & !0x7FFF`) texel movement and takes the maximum.
- **Mipmap level and fraction (`level_signals`):**
  - `lod < 32` (magnification): `level = 0`; with `detail_enabled` or `sharpen_enabled`, clamps to `min_level` and sets bit 8 (`0x100`) when sharpening.
  - `lod >= 32` (minification): `level = LOG2[(lod >> 5) & 0xFF]`, `fraction = ((lod << 3) >> level) & 0xFF`, clamped to `max_level` when distant or overflowing.
  - Selects tile `(base_tile + level + detail_offset) & 7` (and the next mipmap tile in 2-cycle mode for trilinear blending via `LOD_FRACTION`).

---

## 6. Cycle Modes, Color Combiner, Blender, and Chroma Key

### 6.1 Cycle Types (`OtherModes` bits 53:52)

| Cycle Type | Module | Summary |
| --- | --- | --- |
| `0` (`1-CYCLE`) | `src/rdp/one_cycle.rs` | Single combiner cycle (uses `second_combine_cycle` selectors), single blender cycle (`first_blend_cycle`), Z compare/update, dither, RDRAM write |
| `1` (`2-CYCLE`) | `src/rdp/two_cycle.rs` | Pipelined dual combiner (`first` -> `second`, swapping `texel0`/`texel1` between cycles) and dual blender (`first` un-divided -> `second` with optional divide); pixel `n+1`'s first cycles overlap pixel `n`'s second cycles |
| `2` (`COPY`) | `src/rdp/copy.rs` | Streams 8 bytes/step (4 × 16-bit or 8 × 8-bit texels) directly from TMEM to color image, honoring TMEM bank contention (`copy_bank_word`) and optional `alpha_compare`; no-op on 32-bit color images |
| `3` (`FILL`) | `src/rdp/fill.rs` | Writes `fill_color` byte lanes directly across spans (including hidden bits on odd addresses); no-op on 4-bit color images |

### 6.2 Color Combiner (`src/rdp/one_cycle.rs`, `src/rdp/modes.rs`)

Both cycles evaluate `(A - B) * C + D` with 9-bit signed inputs (`extend9`: bits `8:7 == 11` negative; multiplier `C` sign-extended on bit 8):
- **Color `A` / `D` (`0..7`):** `COMBINED`, `TEXEL0`, `TEXEL1`, `PRIMITIVE`, `SHADE`, `ENVIRONMENT`, `1.0 (0x100)`, `0`.
- **Color `B` (`0..7`):** `COMBINED`, `TEXEL0`, `TEXEL1`, `PRIMITIVE`, `SHADE`, `ENVIRONMENT`, `KEY_CENTER`, `K4`, `0`.
- **Color `C` (`0..15`):** `COMBINED`, `TEXEL0`, `TEXEL1`, `PRIMITIVE`, `SHADE`, `ENVIRONMENT`, `KEY_SCALE`, `COMBINED_A`, `TEXEL0_A`, `TEXEL1_A`, `PRIM_A`, `SHADE_A`, `ENV_A`, `LOD_FRAC`, `PRIM_LOD_FRAC`, `K5`, `0`.
- **Alpha `A/B/D` (`0..7`):** `COMBINED_A`, `TEXEL0_A`, `TEXEL1_A`, `PRIM_A`, `SHADE_A`, `ENV_A`, `1.0 (0x100)`, `0`.
- **Alpha `C` (`0..7`):** `LOD_FRAC`, `TEXEL0_A`, `TEXEL1_A`, `PRIM_A`, `SHADE_A`, `ENV_A`, `PRIM_LOD_FRAC`, `0`.

`combine_second_cycle` clamps RGB to 8 bits (`clamp9`), applies `coverage_times_alpha` and `alpha_from_coverage`, and adds `dither_alpha` to pixel alpha and `blender_shade_alpha`.

### 6.3 Chroma Key (`src/rdp/chroma_key.rs`)

When `OtherModes.key_enabled` (bit 40) is set, `chroma_key` computes per-channel distance `(width << 4) + signed_dist` from the unclamped combiner output `(red, green, blue)` (inverting positive distances with the half-step nibble-8 rounding quirk), takes `min(R, G, B).clamp(0, 255)` as `key_alpha`, passes combiner input `A` straight through as pixel RGB, and substitutes `key_alpha` as pixel alpha when `!alpha_from_coverage`.

### 6.4 Blender and Dither (`src/rdp/one_cycle.rs`, `src/rdp/tables.rs`)

Each blend cycle selects `(P, A, M, B)`:
- **Colors `P`, `M` (`0..3`):** pixel/previous blend (`source`), `memory`, `blend_color`, `fog_color`.
- **Alpha `A` (`0..3`):** `pixel.a`, `fog_color.a`, `blender_shade_alpha`, `0`.
- **Alpha `B` (`0..3`):** `1 - A` (`!A & 0xFF`), `memory.a`, `1.0 (0xFF)`, `0`.

Weights are 5-bit (`>> 3`), shifted by `blend_shift_a`/`blend_shift_b` when `B == memory.a`. Without `force_blend`, the blender divides `(P * first + M * (second + 1))` through the 32,768-entry hardware non-restoring divider LUT `BLEND_QUOTIENTS`; with `force_blend` (or cycle 1 of 2-cycle), it shifts `>> 5` directly.

**RGB and Alpha Dither (`DITHER_TABLES`):** 4×4 Magic Square (`rgb = 0`) and Bayer (`rgb = 1`) matrices indexed by `(y >> scissor_field) & 3` and `x & 3`, with inverted/zeroed alpha dither variants (`alpha_dither = 0..3`). Noise dither (`rgb = 2`, `alpha = 2`) uses `0` deterministically (no PRNG noise).

---

## 7. Multi-Resolution and Vulkan Compute GPU Renderer (`src/rdp/gpu/`, `shaders/`)

### 7.1 Dual-Framebuffer Architecture (`src/memory/dp.rs`, `src/ffi/multiple.rs`)

`marsrt` supports upscaled rendering (`RenderScale` `1..4`) and supersampled anti-aliasing (`Antialiasing` `Off`/`2x`/`3x`/`4x`), with the combined drawing scale `scale = render_scale * effective_aa` capped at **`4`** (AA yields first when `render_scale * aa > 4`).

- **Exact 1× machine authority:** The console's own RDRAM and hidden bits are **always** rasterized at 1× on the CPU. Games reading back framebuffers, depth buffers, or render-to-texture data always see exact 1× memory.
- **Parallel scaled shadow:** When `scale > 1`, `ScaledDrawing` allocates a separate scaled RDRAM (`rdram.len() * scale^2`), scaled hidden buffer, and a second `Rdp` instance (`new_scaled`) that replays the same command stream with scaled coordinates, scissor, image strides, and divided attribute derivatives (`dx/de/dy /= scale`), while reading texture loads from the machine's 1× RDRAM.

### 7.2 Vulkan Compute Path (`src/rdp/gpu/mod.rs`, `src/rdp/gpu/record.rs`, `shaders/*.comp`)

When `Gpu` is enabled and `scale > 1` (at `scale == 1`, `gpu_report` is `"off at one"`), `GpuRasteriser` opens a Vulkan compute device via `emusen-gpu` (`EmuSen/Cores/Shared/emusen-gpu`) and binds the entire scaled RDRAM + hidden bits in a single device buffer (one `u32` per 16-bit console word: 16-bit data in bits `15:0`, 2 hidden bits in bits `17:16`). If Vulkan is unavailable or `max_buffer_bytes` is too small, `marsrt` falls back cleanly to CPU scaled shading and reports the reason in `gpu_report`.

**Pipeline and 5 embedded SPIR-V compute shaders:**
1. **Host walk & binning (`record.rs`, `gpu/mod.rs`):** The host walks edges at `scale`, packs 80-word primitive records (`PRIMITIVE_WORDS = 80`), 32-word row records (`ROW_WORDS = 32`), TMEM snapshots (`2,048` words), and tile sets (`32` words), and bins rows into **`64 × 1` pixel tiles** (`TILE_WIDTH = 64, TILE_HEIGHT = 1`) in exact draw order across `AHEAD` ring-buffered staging slots.
2. **`shaders/shade.comp` (`local_size = 64×1`):** Pure-integer GLSL 450 shader implementing the full RDP pixel pipeline (fill, copy, 1-cycle, 2-cycle, coverage, depth, TMEM/palettes/YUV/3-point filter/LOD, combiner, chroma key, blender, dither, and `DIVIDE_TABLE` SSBO). Flushes on color or depth image changes.
3. **`shaders/scan.comp` (`local_size = 8×8`):** Executes the VI scan-out walk (AA coverage filter, dither undo, divot, bilinear resample, gamma) directly over the GPU's scaled memory buffer without reading scaled RDRAM back to the host.
4. **`shaders/clear.comp`:** Applies up to `65,535` (`MAX_SPANS`) dark-border clear spans directly on the GPU raster buffer when GPU averaging is active.
5. **`shaders/average.comp` (`local_size = 8×8`):** Box-averages `side × side` supersampled pixels on the GPU and copies only the downsampled output frame back to host memory.
6. **`shaders/add.comp`:** Compute verification shader bundled alongside the pipeline binaries.

**Primitives declined by the GPU shader (`record.rs`):**
- 8-bit (`color_image_size == 1`) color images (two pixels share one 16-bit word, unsafe for concurrent pixel invocations -> `Declined::Image`).
- 1-cycle primitives reading `COMBINED` (carries previous pixel's combiner output -> `Declined::Carry`).
- 2-cycle primitives whose first combiner cycle reads `COMBINED`, whose first blend cycle weighs memory alpha (`second_alpha == 1`, carrying the previous pixel's depth slope), or with `convert_one` set (`Declined::Carry`).

---

## 8. Video Interface — VI (`src/vi/`)

### 8.1 VI Registers (`src/vi/mod.rs`, `0x0440_0000..0x0440_0037`)

Fourteen 32-bit registers (`REGISTERS = 14`), stored and read back verbatim (except `VI_V_CURRENT`, updated by half-line progression):

| Offset | Index | Register | Key Fields |
| --- | --- | --- | --- |
| `0x00` | `0` | `VI_CONTROL` | `[1:0]` type (`0`=blank, `1`=reserved, `2`=16b RGBA5551, `3`=32b RGBA8888); `[3]` gamma enable; `[4]` divot enable; `[6]` serrate (interlace); `[9:8]` AA mode (`0..1`=AA+resample, `2`=resample only, `3`=replicate/point); `[16]` dither filter enable |
| `0x04` | `1` | `VI_ORIGIN` | `[23:0]` framebuffer base address (aligned `& 0xFF_FFFE` for 16b, `& 0xFF_FFFC` for 32b; `0` = no scan) |
| `0x08` | `2` | `VI_WIDTH` | `[11:0]` framebuffer row stride in pixels |
| `0x0C` | `3` | `VI_V_INTR` | `[9:0]` half-line interrupt match target |
| `0x10` | `4` | `VI_V_CURRENT` | Read: `(half_line & !1) \| field_bit`; Write: clears `MI_INTR_VI` |
| `0x14` | `5` | `VI_BURST` | Stored (timing metadata) |
| `0x18` | `6` | `VI_V_SYNC` | `[9:0]` half-lines per field; `> 550` (`NTSC_SYNC_LINES + 25`) selects PAL clock (`49,656,530` Hz), else NTSC (`48,681,818` Hz) |
| `0x1C` | `7` | `VI_H_SYNC` | `[11:0]` video clocks per full line (`line_debt = H_SYNC * 93_750_000`) |
| `0x20` | `8` | `VI_LEAP` | Stored |
| `0x24` | `9` | `VI_H_START` | `[25:16]` start column, `[9:0]` end column (10-bit wrap) |
| `0x28` | `10` | `VI_V_START` | `[25:16]` start half-line, `[9:0]` end half-line (10-bit wrap) |
| `0x2C` | `11` | `VI_V_BURST` | Stored |
| `0x30` | `12` | `VI_X_SCALE` | `[27:16]` initial sub-pixel bias `start_x`, `[11:0]` 2.10 step `step_x` |
| `0x34` | `13` | `VI_Y_SCALE` | `[27:16]` initial sub-pixel bias `start_y`, `[11:0]` 2.10 step `step_y` |

### 8.2 Half-Line Timing and Interrupt (`src/vi/mod.rs`)

When both `VI_V_SYNC & 0x3FF > 0` and `VI_H_SYNC & 0xFFF > 0`, VI accumulates `debt += delta_cycles * (video_clock * 2)` and advances one half-line each time `debt >= (VI_H_SYNC & 0xFFF) * 93_750_000`.
- Field rate is `(video_clock * 2) / (V_SYNC * H_SYNC)` Hz.
- When `half_line >= V_SYNC`, `half_line` wraps to `0`, `fields` increments (ending `Machine::run_frame`), and `field` toggles if `serrate` (bit 6) is set.
- `MI_INTR_VI` raises whenever `half_line` is even and equals `VI_V_INTR & 0x3FF`.

### 8.3 Scan Geometry and Raster Composition (`src/vi/scan.rs`)

The VI scans into a persistent **`640 × 625`** (`RASTER_WIDTH × RASTER_HEIGHT`) RGBA raster (or `640*scale × 625*scale` at scale `> 1`):
- **Region offsets (`measure`):** Subtracts `108` columns and `34` half-lines for NTSC, or `128` columns and `44` half-lines for PAL. Negative starts advance `start_x` / `start_y` by the overshoot; width clamps to `640`. Non-clamped horizontal edges blank the first `8` and last `7` columns (`first_column = 8`, `last_column = columns - 7`).
- **Blank and border persistence (`borders`):** A blank frame (`type & 2 == 0`) clears the raster and 625-line hold counters (`held`), then still walks; a second consecutive blank skips scanning. Uncovered scanlines fade to black after a 2-field grace period (`held[line] = 2`).
- **Output dimensions (`compose_into`):** NTSC outputs `240` rows progressive (`480` interlaced); PAL outputs `288` rows progressive (`576` interlaced), multiplied by `output_scale / averaging`. Progressive frames report `row_repeat = 2` (unless `repeat_rows` is set) with opaque alpha `0xFF`.

### 8.4 VI Filter Passes (`src/vi/scan/walker.rs`, `src/vi/scan/filters.rs`)

For each output pixel `(column, row)`, the walker steps `across` by `step_x` and `down` by `step_y` in 2.10 fixed-point and applies the hardware filter chain in order:

1. **Pixel Fetch & AA Mode Gate:** Fetches 16-bit RGBA5551 (`red/green/blue` top 5 bits, 3-bit coverage from bit 0 + 2 hidden bits) or 32-bit RGBA8888 (coverage from alpha bits `7:5`). Reads past RDRAM return zero. When `anti_alias > 1` (resample-only or replicate), coverage is forced to `7`.
2. **AA Coverage Filter (`filter`, `pull`, `runners`):** Runs on pixels with `coverage < 7`. Samples 6 surrounding pixels (`(-1,-1), (+1,-1), (-2,0), (+2,0), (-1,+1), (+1,+1)`), keeps only fully covered (`coverage == 7`) neighbors plus the center pixel, finds the second-lowest and second-highest values (`runners`), and pulls the center channel toward them by `missing = 7 - coverage`:
   ```
   result = centre + (((low + high - 2 * centre) * missing + 4) >> 3)
   ```
   When two consecutive output rows map to the same source line (`bug == 1`), the hardware **VI fetch bug** folds the lower-row neighbors onto `(-2, 0)` and `(+2, 0)`.
3. **Dither Undo Filter (`undither`, `step`):** Runs on fully covered (`coverage == 7`) pixels when `VI_CONTROL` bit 16 is set. Compares the top 5 bits (`& 0xF8`) of all 8 surrounding neighbors (subject to the same `bug == 1` lower-row fold) against the center pixel and adds `±1` per neighbor (`signum`).
4. **Divot Filter (`divot`, `median`):** When `VI_CONTROL` bit 4 is set, replaces any pixel where `centre.cov & left.cov & right.cov != 7` with the channel-wise median of `(left, centre, right)`.
5. **Bilinear Resample (`mix`):** When `anti_alias != 3` (`REPLICATE`), linearly blends neighboring post-divot pixels vertically and horizontally using 5-bit weights `((coord >> 5) & 0x1F)` via `((far - near) * frac + 16) >> 5 + near`.
6. **Gamma Correction (`gamma`, `GAMMA` LUT):** When `VI_CONTROL` bit 3 is set, maps each 8-bit channel through a 16,384-entry square-root LUT `GAMMA[channel << 6] = isqrt(channel << 6) << 1`.

### 8.5 Deferred Presentation, Banded Parallelism, and Repeat Skipping (`src/vi/scan/presenter.rs`, `src/vi/scan/bands.rs`)

- **Deferred presentation (`DeferredPresentation`, default `true`):** Captures the exact reachable RDRAM slice (`reach`) on the emulation thread after synchronizing with the RDP (`site::VI`), then hands walking and composition to the dedicated `"MarsRT presenter"` thread while the next frame emulates (introducing 1 frame of presentation latency).
- **Multi-band walking (`src/vi/scan/bands.rs`):** Splits scans with at least `32` rows per band (`MINIMUM_BAND_ROWS = 32`) across up to `8` helper threads (`default_bands = clamp(cores/4, 1, 4)`, overridable via `EMUSEN_MARSRT_SCAN_BANDS`).
- **Repeated-scan skipping (`SkipRepeatedScans`, default `true`):** If the VI job geometry and captured RDRAM + hidden bytes are byte-for-byte identical to the previous deferred scan (and no border fade/blank occurred), the walk is skipped with identical output.
- **Supersampling (`box_average`):** When `Antialiasing > 1`, box-averages `side × side` pixel blocks on the host CPU or via `shaders/average.comp` on the GPU.

### 8.6 Accuracy Boundaries

- **No gamma dither PRNG (`VI_CONTROL` bit 2) or RDP noise dither PRNG:** Gamma LUT lookup uses zeroed low 6 bits (`channel << 6`) rather than LFSR noise; RDP noise dither modes read `0` deterministically for bit-exact reproducibility.
- **Frame-level VI snapshot:** VI scan-out runs once per field boundary using the end-of-field VI registers rather than per-scanline mid-frame register changes.
- **Instantaneous RDP command consumption (logical time):** On the emulation timeline, `DPC_CURRENT` advances to `DPC_END` and `SyncFull` fires `MI_INTR_DP` immediately when the list is accepted; RDP pipeline busy/clock counters (`DPC_CLOCK`/`DPC_BUFBUSY`/`DPC_PIPEBUSY`/`DPC_TMEM`) read `0`.
- **GPU multi-resolution limitations:** As noted in §7.2, 8-bit framebuffers and cross-pixel carry primitives are declined on the scaled GPU path (while the authoritative 1× CPU framebuffer always renders them faithfully).
