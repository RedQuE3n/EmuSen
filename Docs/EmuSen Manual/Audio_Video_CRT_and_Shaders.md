# Audio, Video, CRT, and Shaders

EmuSen's real-time audio and video pipelines live in two focused `Platform/` crates: **`Platform/endymion`** for SDL3 audio playback, phase-continuous resampling, dynamic rate control, and UI cue routing, and **`Platform/serenity`** for Vulkan slang shader chains, SPIR-V SQLite caching, zero-readback GPU frame handoff, display frame pacing, handheld LCD filters, and EmuSen's physically modelled CRT filter.

For persisted audio and display keys see [Settings and Configuration](Settings_and_Configuration.md). For controller input also owned by Endymion see [Input and Controllers](Input_and_Controllers.md).

---

## 1. Audio Pipeline (`Platform/endymion`)

Retro hardware produces audio at console-specific clock rates (such as `32040 Hz` on SNES, `44100 Hz` on PlayStation, or dynamically changing DAC rates on Nintendo 64) while host audio hardware and displays run on independent oscillators. Endymion synchronizes audio continuously without crackles, buffer underruns, or frame pacing glitches.

### 1.1 `LinearResampler` (`resampler.rs`)

`LinearResampler` performs phase-continuous linear interpolation over interleaved stereo 16-bit signed PCM frames (`L, R, L, R`):

- Maintains fractional phase position (`frac: f64`) and the previous stereo sample pair (`prev_left`, `prev_right`) across calls so buffer boundaries are seamless.
- Steps with double-precision accumulation (`step = 1.0 / ratio`) and rounds output samples to `i16` using round-half-to-even quantization.
- Validates every ratio up front, returning `RatioError::NotPositive` (`<= 0.0`) or `RatioError::NotFinite` (`NaN` / `±inf`) rather than corrupting audio or looping infinitely.

### 1.2 `DynamicRateControl` (`rate_control.rs`)

`DynamicRateControl` absorbs clock drift between emulated hardware timing and the host audio stream by nudging the resampling ratio smoothly rather than dropping or duplicating audio packets:

- **Proportional queue steering:** Reads queued output frames (`queued_frames`) from the SDL3 stream and steers toward `target_queued_frames` (derived from `target_latency_ms` and `sample_rate`):

  ```text
  delta = (queued_frames - target_queued_frames) / target_queued_frames
  ratio = nominal_ratio * (1.0 - clamp(delta, -1.0, 1.0) * max_deviation)
  ```

- **Default tuning:** `max_deviation = 0.005` (±0.5% pitch skew — below typical audible pitch-shift perception) centered on `nominal_ratio = 1.0`.
- **Overflow shedding hysteresis:** During fast-forward bursts or host stalls, if queued audio exceeds `target * shedding_entry_factor` (`3.0x`), DRC enters shedding mode and skips submitting incoming audio until the queue drains below `target * shedding_exit_factor` (`2.0x`), counting each transition in `shedding_events`.

### 1.3 SDL3 Streams, Sample-Rate Reopen, and Device Selection (`audio.rs`)

`AudioPlayer` drives an **SDL3** audio stream (`SDL_OpenAudioDeviceStream`) paired with `DynamicRateControl`:

- **Upfront stream open:** Opens the playback stream during initialization so `is_available()` reports true audio availability before the first frame executes.
- **Dynamic sample-rate reopen:** Whenever `submit()` receives samples at a `sample_rate` differing from `open_sample_rate` (such as N64 games switching Audio Interface DAC rates mid-game), `AudioPlayer` closes and reopens the SDL3 stream at the new rate while retaining the current gain (`volume`) and recalculating `DynamicRateControl`'s target queue depth.
- **Named output device selection and fallback:** `playback_devices()` queries SDL3 playback devices as `(id, name)` pairs. `set_device(rate, Some(name))` selects a matching device by name and reopens an active stream on it. If the requested device is disconnected or fails to open, `open_stream()` transparently falls back to `SDL_AUDIO_DEVICE_DEFAULT_PLAYBACK` and returns `false` so the frontend can surface the fallback without silencing the game.
- **Volume persistence:** Gain (`0.0 .. 1.0`) is applied via SDL3 stream gain and automatically restored across device switches and sample-rate reopens.
- **Clean teardown:** `dispose()` is idempotent, closes the SDL3 stream, and releases `SDL_INIT_AUDIO`.

### 1.4 UI Sound Cue Routing

Interface navigation cues (`scroll`, `systembrowse`, `select`, `back`, `favorite`, `launch`, `quicksysselect`) play through a dedicated SDL3 stream separate from gameplay audio (`audio.rs`):

- Fixed internal format: **48 kHz stereo 32-bit float**.
- Short UI clips are decoded once at startup and played directly with low latency so menu sounds never disturb the game session's `DynamicRateControl` queue or game mute/volume controls.

---

## 2. Video Pipeline, Vulkan Handoff, and Shaders (`Platform/serenity`)

`Platform/serenity` owns the Vulkan rendering device, RetroArch `.slang` / `.slangp` shader compilation, SPIR-V caching, zero-readback GPU frame intake, built-in filters, and display frame pacing.

### 2.1 Slang Shader Chain and SPIR-V SQLite Cache

- **Multi-pass `.slangp` pipeline (`chain.rs`, `preset.rs`, `compiler.rs`, `reflection.rs`, `parameters.rs`):** Executes both built-in EmuSen filters and external RetroArch slang shader presets. Each pass specifies:
  - Target scale (`Scale::Source`, `Scale::Viewport`, or explicit `Axis::{Source, Viewport, Fixed}` per axis).
  - Source and named child texture sampling (`nearest` vs `linear`).
  - Half-float intermediate framebuffers (`float: true`) for linear-light HDR/phosphor math.
  - Multi-frame history buffers (`history1 .. historyN`, used by LCD ghosting and CRT phosphor persistence) and self-feedback (`feedback: true`).
  - Uniform parameters exposed live to the frontend shader settings UI.
- **SPIR-V SQLite cache (`cache.rs`, `spirv-cache-schema.sql`):** Compiled vertex and fragment SPIR-V blobs are persisted in a SQLite database keyed by shader source hash, compiler version, and baked structural constants. Reloading a preset or launching a console with a complex multi-pass CRT shader retrieves cached SPIR-V binaries immediately without re-running GLSL compilation.

### 2.2 Zero-Readback Vulkan Device-Picture Handoff (`lend.rs` and `ring.rs`)

Hardware-rendered cores transfer frames directly inside GPU memory without copying pixels back to CPU RAM:

- **In-process Vulkan lend (`lend.rs`, Core ABI `EMUSEN_CAP_DEVICE_PICTURE`):** Serenity lends a dedicated Vulkan 1.2+ queue and a **3-slot** (`SLOTS = 3`) timeline-semaphore ring (`done` and `taken` Vulkan timeline semaphores) to the core via `emusen_device_lend`. The core submits rendering commands to its lent queue and signals its `ready` timeline semaphore; Serenity's shader chain waits on `ready`, samples the GPU image directly, and signals `done` once the slot can be safely overwritten.
- **Cross-process Vulkan ring (`ring.rs`, Core ABI `EMUSEN_CAP_DEVICE_RING`):** For Landlock-confined Libretro cores running inside `emusen-core-runner`, Serenity allocates external-memory Vulkan images (`OPAQUE_FD` on Linux) and exports their memory and synchronization handles over the broker socket. The isolated runner renders directly into the shared GPU ring and hands completed slot indices back to Serenity.

### 2.3 Display Frame Pacing (`pace.rs`)

`DisplayClock` observes actual compositor swap counts and timestamps (steady-clock nanoseconds) without blocking or sleeping:

- Smooths measured swap intervals using an exponential moving average (`period_ns = period_ns * 0.9 + sample * 0.1`), accurately dividing multi-swap intervals across frames.
- Locks emulation to one frame per display swap (`locks(interval_ns)`) whenever the emulated console's native frame interval falls within **`LOCK_SHARE = 0.01`** (±1%) of the display refresh period — locking NTSC NES (`60.0988 Hz`) smoothly to a `60 Hz` panel while letting a `50 Hz` PAL game pace on its own independent clock.

---

## 3. Built-In Screen Filters (`filters/panels.rs`)

In addition to external `.slangp` presets and the physically modelled CRT, `Platform/serenity/src/filters/` ships three families of built-in filters:

| Family | Filter Name(s) | Technical Basis & Parameters |
|---|---|---|
| **Simple Effects** | `None`, `Scanlines`, `Crt` (`effect.<Name>.{nearest,linear}`) | Single-pass viewport filters for crisp nearest-neighbor scaling, bilinear filtering, or lightweight scanlines. |
| **CRT (Lottes)** | `CRT (Lottes)` | Single-pass port of Timothy Lottes' public-domain `crt-lottes.slang` for television consoles with 11 parameters: `hardScan` (`-8.0`), `hardPix` (`-3.0`), `warpX` (`0.031`), `warpY` (`0.041`), `maskDark` (`0.5`), `maskLight` (`1.5`), `brightBoost` (`1.0`), `hardBloomPix` (`-1.5`), `hardBloomScan` (`-2.0`), `bloomAmount` (`0.15`), `shape` (`2.0`). |
| **Monochrome Handheld LCDs** | `Game Boy LCD` (DMG), `Game Boy Pocket LCD`, `Game Boy Light LCD` | Two-pass filters with 3-frame LCD ghosting history (`response`: `0.33` / `0.12` / `0.12`) and `0.86`-coverage dot matrix (`shadowOpacity`: `0.25` / `0.2` / `0.1`). Uses SameBoy's measured 4-shade + off-panel palettes (`DMG`, `POCKET`, `LIGHT`). |
| **Colour Handheld LCDs** | `Game Boy Color LCD`, `Game Boy Advance LCD`, `Game Boy Advance SP (AGS-101) LCD` | Two-pass filters applying Pokefan531's measured linear sRGB matrices (`GBC_GBA_LUMINANCE = 0.91`, `SP101_LUMINANCE = 0.935`), 3-frame response (`0.33` / `0.25` / `0.0`), and RGB subpixel stripe grids after `authentic_gbc` / `mGBA`. |

---

## 4. Physically Modelled CRT Filter (`filters/crt.rs`)

EmuSen's flagship built-in `CRT` filter synthesizes specialized Vulkan GLSL passes from physical parameters modelling the entire analog broadcast chain: console video encoder, receiver luma/chroma separation, chroma demodulation, electron-beam spot growth, aperture/slot/dot phosphor masks, phosphor decay, glass halation, and colour gamut conversion.

### 4.1 Three Quality Tiers (`Quality`)

| Tier | Pass & Signal Model | Intended Hardware |
|---|---|---|
| **`Accurate`** | Full analog composite/S-Video signal synthesis sampled across **`SIGNAL_SAMPLES = 2048`** horizontal points per active line, bilinear notch/comb luma trap, CXA2025AS demodulator axes, two-exponential P22 phosphor tails, **`128 × 96`** glare/halation pass, and full beam/mask/curvature model. | Desktop and modern handheld GPUs where maximum analog accuracy is desired. |
| **`Balanced`** *(default)* | Preserves console colourimetry, scanline spot profile, mask structure, curvature, and core signal behavior at lower pass and sample cost. | Default everyday play across desktops and Linux handhelds. |
| **`Performance`** | Streamlined low-overhead pass graph retaining geometry, scanlines, and phosphor gamut for constrained GPUs or extended battery life. | Integrated GPUs and low-TDP handheld profiles. |

### 4.2 Six Physical Screen Classes (`SCREENS`)

Six receiver/monitor profiles (default index `1`: *Consumer TV, 20-inch*) define physical dimensions, phosphor mask, triad density, spot diameter, decoder, bandwidth, glass curvature, and beam convergence:

| Screen Class | Width (mm) | Mask | Triads Across | Spot Min / Max | Decoder | Chroma / Video BW (MHz) | Glass (`R/W`) | Convergence (Centre / Edge) |
|---|---:|---|---:|---|---|---:|---|---|
| Consumer TV, 14-inch | `264` | Slot | `440` | `0.158 / 0.316` | Notch | `0.5 / 4.2` | Spherical (`1.9`) | `0.6 / 1.0` |
| Consumer TV, 20-inch *(default)* | `386` | Slot | `515` | `0.108 / 0.217` | Notch | `0.5 / 4.2` | Spherical (`1.9`) | `0.6 / 1.0` |
| Consumer Trinitron, 27-inch | `549` | Aperture Grille | `732` | `0.076 / 0.153` | Comb | `0.5 / 4.2` | Cylindrical (`2.5`) | `0.6 / 1.0` |
| Professional monitor, 14-inch | `266` | Aperture Grille | `1064` | `0.126 / 0.252` | Comb | `1.3 / 10.0` | Cylindrical (`2.5`) | `0.4 / 0.5` |
| Professional monitor, 20-inch | `386` | Aperture Grille | `1245` | `0.087 / 0.173` | Comb | `1.3 / 10.0` | Cylindrical (`2.5`) | `0.5 / 0.7` |
| PC monitor, 15-inch | `280` | Dot | `1167` | `0.090 / 0.180` | Comb | `1.3 / 30.0` | Spherical (`3.3`) | `0.3 / 0.4` |

### 4.3 Complete CRT Parameter Reference

Eleven parameters marked **Structural** (`STRUCTURAL`) alter the generated GLSL pass graph or compile-time constants (cached in the SPIR-V SQLite cache); all other parameters update live uniforms instantaneously:

| Parameter ID | Type | Default | Range / Choices | Structural? |
|---|---|---|---|---|
| `quality` | Choice | `Balanced` | `Performance`, `Balanced`, `Accurate` | Yes |
| `signal` | Choice | `Composite` | `RGB`, `S-Video`, `Composite` | Yes |
| `standard` | Choice | `The game's` | `NTSC`, `PAL`, `The game's` | Yes |
| `palDecoder` | Choice | `Delay line` | `Delay line`, `Simple` | Yes |
| `screen` | Choice | `Consumer TV, 20-inch` | 6 `SCREENS` classes | Yes |
| `colour` | Choice | `North America` | `North America`, `Japan`, `Europe` | Yes |
| `gamut` | Choice | `sRGB` | `sRGB`, `Display P3` | Yes |
| `mask` | Choice | `The screen's own` | `The screen's own`, `None`, `Aperture grille`, `Slot mask`, `Dot mask` | Yes |
| `decoder` | Choice | `The screen's own` | `The screen's own`, `Notch`, `Comb` | Yes |
| `axes` | Choice | `The standard's` | `The standard's`, `Sony CXA2025AS, US axis`, `Sony CXA2025AS, Japan axis` | Yes |
| `chroma` | Slider | `1.0` | `0.5 .. 3.0` (step `0.1`, relative chroma bandwidth) | Yes |
| `curvature` | Slider | `1.0` | `0.0 .. 2.0` (step `0.05`) | No |
| `level` | Choice | `Bright` | `Bright`, `The tube's own` | No |
| `displayNits` | Slider | `300.0` | `100.0 .. 1000.0` cd/m² (step `10.0`) | No |
| `subpixels` | Choice | `RGB` | `None`, `RGB`, `BGR` | No |
| `maskPitch` | Slider | `1.0` | `0.5 .. 3.0` (step `0.05`) | No |
| `maskLimit` | Choice | `Kept fine` | `Enlarged with the picture`, `Kept fine` | No |
| `maskDepth` | Slider | `1.0` | `0.0 .. 1.0` (step `0.05`) | No |
| `spotSize` | Slider | `1.0` | `0.5 .. 2.0` (step `0.05`) | No |
| `spotGrowth` | Slider | `1.0` | `0.0 .. 2.0` (step `0.05`) | No |
| `glare` | Slider | `0.1` | `0.0 .. 3.0` (step `0.1`) | No |
| `persistence` | Slider | `0.1` | `0.0 .. 6.0` (step `0.1`) | No |
| `convergence` | Slider | `1.0` | `0.0 .. 3.0` (step `0.1`) | No |
| `tubeNits` | Slider | `100.0` | `50.0 .. 300.0` cd/m² (step `5.0`) | No |
| `gamma` | Slider | `2.4` | `2.0 .. 2.8` (step `0.05`) | No |
| `contrast` | Slider | `1.0` | `0.5 .. 1.5` (step `0.01`) | No |
| `brightness` | Slider | `0.0` | `-0.1 .. 0.1` (step `0.005`) | No |
| `interlace` | Choice | `Woven` | `Woven`, `Fields` | No |

### 4.4 Physical Constants and Reference Literature

Every constant in `Platform/serenity/src/filters/crt.rs` is grounded in broadcast standards or published CRT measurements:

- **Phosphor Primaries & White Points (`COLOURS`, `DISPLAYS`):**
  - **North America (SMPTE 170M / SMPTE-C, D65):** R `(0.630, 0.340)`, G `(0.310, 0.595)`, B `(0.155, 0.070)`, W `(0.3127, 0.3290)`.
  - **Japan (NTSC-J P22, 9300K + 27 MPCD):** R `(0.618, 0.350)`, G `(0.280, 0.605)`, B `(0.152, 0.063)`, W `(0.281, 0.311)`.
  - **Europe (EBU / ITU-R BT.470, D65):** R `(0.640, 0.330)`, G `(0.290, 0.600)`, B `(0.150, 0.060)`, W `(0.3127, 0.3290)`.
  - Mapped to host **`sRGB`** or **`Display P3`** primaries (D65) with **ITU-R BT.1886** transfer gamma (`2.4` default) and dark-room black floor `BLACK_LEVEL = 0.01 / 176.0`.
- **Phosphor Persistence (`Tails`):** Two-exponential decay fitted to Kuhn's measured P22 phosphor decay curves (`TAIL_FAST = 0.590`, `TAIL_SLOW = 0.945`, `GREEN_FAST = 0.0679`, `GREEN_SLOW = 0.0094`, `BLUE_FAST = 0.0601`, `BLUE_SLOW = 0.0081`, `TAIL_SCALE = 0.1606`), automatically time-scaled for `50 Hz` PAL fields (`PAL_FIELD_MILLISECONDS / NTSC_FIELD_MILLISECONDS`).
- **Receiver Demodulation & Luma Trap:** Standard mathematical YUV axes plus measured **Sony CXA2025AS** US (`R = 112° / 0.83`, `G = 252° / 0.30`) and Japan (`R = 95° / 0.78`, `G = 240° / 0.30`) axes, paired with a bilinear notch luma trap whose lower `-3 dB` edge lies at `TRAP_LOWER_EDGE_MEGAHERTZ = 2.70 MHz`.
- **Halation / Glare:** `128 × 96` pass with `GLARE_SIGMA = 0.0668` and `GLARE_WEIGHT = 0.0299`.
- **Broadcast Subcarriers:** NTSC `3.579545 MHz` (`59.94 Hz` field) and PAL `4.43361875 MHz` (`50.0 Hz` field, `52.0 µs` active line, `283.7516` cycles/line).

---

## 5. Per-Console Display Profiles and Overscan (`filters/screens.rs`)

`Platform/serenity/src/filters/screens.rs` assigns every console in the systems table its display kind (`Television` or `Lcd`), accurate filter profile, and overscan rule:

| Console | Display Kind | Accurate Profile | Overscan Crop? | Subcarrier & Timing Notes |
|---|---|---|---|---|
| **NES** | Television | `NES` | Yes | Direct PPU composite signal (`ppuSignal = 1.0`), `256` pixels at `5.3693175 MHz` NTSC / `5.3203424 MHz` PAL. |
| **SNES** | Television | `SNES` | Yes | `256` active pixels at `5.3693175 MHz` NTSC (`1/3` line & frame phase) / `5.3203425 MHz` PAL. |
| **Genesis** | Television | `Genesis` | Yes | `256` base pixel clock with PAL alternating V-switch (`switch_frame = true`). |
| **Sega CD** | Television | `Genesis` | Yes | Attachment with no independent video output (`hosts_picture = true`); outputs through Genesis video stage. |
| **32X** | Television | `Genesis` | Yes | Uses Genesis timing row (dedicated 32X mixed-video output encoder remains undocumented). |
| **N64** | Television | `N64` | Yes | `640` active clocks at `12.1704545 MHz` NTSC / `12.4141325 MHz` PAL (`phase_crawl = 0.25`). |
| **PlayStation** | Television | `PlayStation` | Yes | `2560` clocks at `53.693175 MHz` NTSC / `53.203425 MHz` PAL (`phase_crawl = 0.5`). |
| **PlayStation 2** | Television | `PlayStation 2` | Yes | `54.0 MHz` video clock (`PS2_VIDEO_MEGAHERTZ`), `2560` clocks per active line. |
| **Master System** | Television | `Master System` | Yes | Shares Genesis NTSC/PAL pixel-clock subcarrier timing. |
| **SG-1000** | Television | `SG-1000` | Yes | Dedicated `SG1000_PAL_CYCLES` ratio against the `10.738635 / 2 MHz` crystal. |
| **GB** | LCD | `Game Boy LCD` | No | Every pixel visible; measured DMG/Pocket/Light LCD panel fit. |
| **GBC** | LCD | `Game Boy Color LCD` | No | Every pixel visible; measured GBC colour LCD matrix and subpixel stripe grid. |

Hosted-only consoles (**Saturn**, **Dreamcast**) are tracked in `screens::OWED` for dedicated CRT profiles and do not apply automatic television overscan until their profiles are added.

### 5.1 Overscan Constants

- **`TELEVISION_OVERSCAN = 0.07`:** Models Sony's 7% normal-scan overscan figure on television CRTs; each edge hides `(1 - 1 / (1 + 0.07)) / 2 ≈ 3.27%` of the frame when television overscan cropping is active.
- **`MOST_AT_AN_EDGE = 0.25`:** Hard safety clamp guaranteeing manual or automatic overscan crops never remove more than 25% from any single edge (at least half the picture always remains visible).
- Handheld LCD consoles (`GB`, `GBC`) never crop overscan because every physical LCD pixel was visible on original hardware.

---

## Related Documentation

- [Settings and Configuration](Settings_and_Configuration.md) — `audio.json` and `graphics.json` configuration fields.
- [Core ABI and Libretro Host](Core_ABI_and_Libretro_Host.md) — `EMUSEN_CAP_DEVICE_PICTURE` and `EMUSEN_CAP_DEVICE_RING` contracts.
- [Input and Controllers](Input_and_Controllers.md) — Endymion's controller and hotplug subsystem.
- [Platform C Interfaces](Platform_C_Interfaces.md) — `emusen_platform.h` Serenity and Endymion C exports.
