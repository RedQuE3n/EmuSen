# Venus — NEC DSP Open High-Level Replacements (HLE)

Core crate directory: `EmuSen/Cores/Nintendo/VenusRT - SNES` (crate `venusrt`).

This manual documents the live open high-level replacements (`DspEngine::Hle`) for the NEC uPD77C25 and uPD96050 SNES cartridge coprocessors implemented in `src/chips/dspengine.rs`, `src/chips/dsphle.rs`, `src/chips/dsp2.rs`, `src/chips/dsp4.rs`, and `src/chips/st010.rs`. When external firmware files are not provided, `venusrt` runs these clean-room behavioral implementations behind the exact same DR, SR, and RAM port interface as the low-level engine (`Venus_Coprocessors.md`).

---

## 1. Engine Selection, Fallback, and State Isolation (`src/chips/dspengine.rs`, `src/v1.rs`)

### 1.1 Automatic Fallback Policy

NEC DSP firmware is never mandatory in `venusrt`. During `Core::create` (`src/v1.rs`):

1. **Firmware Supplied (File 2, or File 2 + File 3)**: `Machine::attach_dsp` loads the 8,192-byte or 53,248-byte firmware image into `DspEngine::Lle(NecDsp)`. `machine_info().firmware` reports `(2, FirmwareSource::File)`.
2. **Firmware Absent + Replacement Exists (`dsp1`, `dsp1b`, `dsp2`, `dsp4`, `st010`)**: `Machine::attach_replacement` looks up `Cartridge::nec_firmware()` via `Program::for_stem` and attaches `DspEngine::Hle(DspHle::new(program))`. `machine_info().firmware` reports `(2, FirmwareSource::Replacement)`.
3. **Firmware Absent + No Replacement (`dsp3`, `st011`)**: `cart.dsp` remains `None` and the cartridge runs without its coprocessor. `machine_info().firmware` reports `(2, FirmwareSource::Absent)`.

| Chip Stem | Chip Name | Firmware Size | Open HLE (`Program`) | Status Without Firmware |
|---|---|---|---|---|
| `dsp1` | DSP-1 | 8,192 B | `Program::Dsp1` | Full command set; level flight matches closely; steep tilts past ~80° diverge |
| `dsp1b` | DSP-1B | 8,192 B | `Program::Dsp1b` | Full command set; racing/baseball/Pilotwings level flight play closely; *Lock On* steep tilts draw no ground |
| `dsp2` | DSP-2 | 8,192 B | `Program::Dsp2` | *Dungeon Master* commands cycle-accurate; data ROM dump zeros; scaling timing estimated |
| `dsp3` | DSP-3 | 8,192 B | None | No HLE replacement; runs without chip |
| `dsp4` | DSP-4 | 8,192 B | `Program::Dsp4` | Protocol and `00h` multiply exact; road/scenery unimplemented (*Top Gear 3000* menus work, races stay black) |
| `st010` | ST010 | 53,248 B | `Program::St010` | *F1 ROC II* mailbox cycle-accurate; road and opponents play closely; battery file byte-identical with LLE |
| `st011` | ST011 | 53,248 B | None | No HLE replacement; runs without chip |

### 1.2 Clean-Room Table Rule and Lockstep Oracle

- **Runtime-Generated Math Tables**: No replacement source file embeds lookup tables copied from mask ROMs. A unit test (`no_replacement_source_holds_a_table_literal` in `src/chips/dsphle.rs`) mechanically enforces that `dsphle.rs`, `dsp2.rs`, `st010.rs`, and `dsp4.rs` contain no numeric array literal longer than 16 elements. All trigonometric, arctangent, and perspective tables are generated at runtime from closed-form formulas (such as `quarter_sine` using a 14-term `f64` Taylor series so all platforms produce identical bits).
- **Differential Oracle (`src/chips/dsporacle.rs`)**: Provides a port-only lockstep driver (`Chip` trait) that compares `DspHle` against `NecDsp` solely through `DR`, `SR`, ST01x `RAM`, and chip cycle counts when test firmware is present on a developer machine.

### 1.3 Save States, Battery Files, and Debugger Differences

- **Engine Tag Isolation**: In `VNRT` save states (`src/machine.rs`, `src/state.rs`), `Coprocessor.DspEngine` stores `0` for `DspEngine::Lle` (`Coprocessor.NecDsp`) and `1` for `DspEngine::Hle` (`Coprocessor.DspHle`). Loading a state saved under the opposite engine fails cleanly with `STATUS_OTHER_DSP_ENGINE` (`-11`). Replacement states require `STATE_VERSION >= 21` (and `>= 22` for DSP-1/1B, which added projection state).
- **Interchangeable ST010 Battery Saves**: `DspHle` allocates the full `2,048 × u16` (4 KiB) on-chip RAM for `Program::St010`, exposed at `$68–$6F:0000–$0FFF` and saved as `.srm` (space 6). Because every ST010 HLE command writes the exact same RAM locations (including working words), `.srm` files are freely interchangeable between HLE and LLE.
- **Debugger Visibility**: Under HLE, processor 2 (`DSP`) and space 14 (`DSPPRG`) are omitted because no native uPD77C25/uPD96050 instruction stream executes. Space 13 (`DSPRAM`) remains exposed whenever `ram()` is non-empty (ST010 HLE).

---

## 2. Shared Port and Event Scheduler (`src/chips/dsphle.rs`)

`DspHle` shares the exact same clock ratios as `NecDsp`: `760,000 / 2,147,727` (7.60 MHz) for DSP-1/1B/2/4 and `1,000,000 / 2,147,727` (10.00 MHz) for ST010. Rather than stepping an instruction loop, `DspHle::run_to(clock)` jumps directly between scheduled port events (`Next::Request`, `Next::Widen`, `Next::Offer`, `Next::Idle`, `Next::Done`) and DSP-2 SR flag transitions (`d2.mode_at`).

- **Handshake Fidelity**: `host_read` and `host_write` replicate `RQM` (`$8000`), `DRS` (`$1000`), and `DRC` (`$0400`) identically to hardware, including completing transfers regardless of read vs write direction.
- **Two-Parameter Phase Timing**: Each transaction phase schedules its next `RQM` rising edge at:
  `due = max(rise + work, cycles + notice)`
  where `rise` is the chip cycle of the previous `RQM` rising edge, `cycles` is the completion cycle of the S-CPU transfer, `work` is the chip's internal execution time from the last edge, and `notice` is the minimum turnaround after the S-CPU finishes its transfer (commonly `2` cycles on DSP-1B).
- **3-Cycle DRC Widening**: When transitioning from 8-bit command mode (`DRC = 1`) to 16-bit parameter mode (`DRC = 0`) on DSP-1/1B, `Next::Request { wide: true }` raises `RQM` first and schedules `Next::Widen` 3 chip cycles later to clear `DRC`, matching hardware timing.

---

## 3. DSP-1 / DSP-1B Replacement (`src/chips/dsphle.rs`)

### 3.1 Protocol and Idle Behavior

- At reset, `idle_word = $80` and `SR` settles to `$8400` (`RQM | DRC`, 8-bit command mode).
- Command bytes `$40–$FF` are ignored (the chip re-requests an 8-bit command byte after `NOTICE = 2` cycles).
- Valid command bytes `$00–$3F` transition through 16-bit parameter inputs (`Stage::Taking`) and 16-bit result outputs (`Stage::Giving`), returning to `Stage::Idle` with `DR = $80` and `DRC = 1`.
- Unlisted command bytes in `$00–$3F` accept 1 dummy 16-bit transfer and return immediately to idle.

### 3.2 Generated Fixed-Point Tables and Math Primitives

- **Sine and Derivative Tables (`tables()`, `sine(a)`)**: Generates 256-entry 16-bit sine (`floor(32768 * sin(2π·i/256))`) and derivative (`floor(32768 * (2π/256) * cos(2π·i/256))`) tables from the 14-term `quarter_sine` Taylor series. For a 16-bit angle `a` (`2^16` = one full turn), `sine(a)` uses the top 8 bits `i = a >> 8` and linear interpolation over the low 8 bits `f = (a & $FF) << 7`:
  `clamp_i16(t[i] + ((f * t[256 + i] + 16384) >> 15))`
  Cosine is `sine(a.wrapping_add(0x4000))`.
- **Reciprocal Normalization (`inverse`, `reciprocal`)**: Normalizes non-zero signed inputs into `[0x4000, 0x8000)` or `[-0x8000, -0x4000)` via bit shifts `s`, then computes `floor(2^29 / |m|)` (capped at `0x7FFF`), returning `(mantissa, exponent = 1 + s - b)`. For `a == 0`, returns `(0x7FFF, 0x2F)`.

### 3.3 DSP-1 / DSP-1B Command Table

| Command | Name | In (16b) | Out (16b) | Implementation & Accuracy |
|---|---|---|---|---|
| `00h` / `20h` | Multiply | 2 | 1 | **Exact**: `(K * I) >> 15` (`20h` ORs bit 0 with `1`). |
| `08h` | Radius | 3 | 2 | **Exact**: `((x² + y² + z²) << 1)` as 32-bit little-endian (`L2`, `H2`). |
| `18h` / `38h` | Range | 4 | 1 | **Exact**: High word of `((x² + y² + z² - r²) << 1)` (`38h` adds 1). |
| `0Fh` | Memory Test | 0 (1 dummy) | 1 | **Exact**: Returns `0x0000` (4,121 / 3,337 work/notice cycles). |
| `2Fh` | ROM Version | 0 (1 dummy) | 1 | **Exact**: Returns `0x0100` (`Program::Dsp1`) or `0x0101` (`Program::Dsp1b`). |
| `1Fh` | Data ROM Dump | 0 (1 dummy) | 1,024 | Transfer shape modeled; outputs zeros (no mask ROM embedded). |
| `04h` | Trigonometric | 2 | 2 | **Approximate**: `((I * sin(a)) >> 15, (I * cos(a)) >> 15)` via generated `sine()`. |
| `0Ch` | Rotate 2D | 3 | 2 | **Approximate**: `(x*cos + y*sin) >> 15`, `(y*cos - x*sin) >> 15`. |
| `1Ch` | Rotate 3D | 6 | 3 | **Approximate**: Sequential Z (`I1`), Y (`I2`), X (`I3`) rotations with 16-bit wrapping per axis. |
| `10h` / `30h` | Inverse | 2 | 2 | **Approximate**: Exact exponent and shift-dependent cycle timing (`14` for `a=0`, `36 + 2s + 2*edge` for `a>0`, `40 + 2s + edge` for `a<0`); mantissa within ~2 units. |
| `28h` | Distance (Sqrt) | 3 | 1 | **Approximate**: `min(isqrt((x² + y² + z²) & $FFFFFFFF), $7FFF)` with bit-width-dependent timing (`-1` cycle on DSP-1). DSP-1's original overflow bug is not reproduced. |
| `01h`, `11h`, `21h` | Attitude (A/B/C) | 4 | 0 | **Approximate**: Computes 3×3 Q15 matrix `floor((m / 2) * Rx(I4)·Ry(I3)·Rz(I2))` stored in `att[0..3]`. |
| `0Dh`, `1Dh`, `2Dh` | Objective (A/B/C) | 3 | 3 | **Approximate**: Multiplies stored matrix `att[n]` by 3D vector `(x, y, z)`. |
| `03h`, `13h`, `23h` | Subjective (A/B/C) | 3 | 3 | **Approximate**: Multiplies transpose `att[n]^T` by 3D vector `(x, y, z)`. |
| `0Bh`, `1Bh`, `2Bh` | Scalar (A/B/C) | 3 | 1 | **Approximate**: Dot product of first row of `att[n]` with `(x, y, z)`. |
| `02h` | Parameter | 7 | 4 | **Approximate**: Stores 7 projection inputs (`Fx, Fy, Fz, Lfe, Les, Aas, Azs`) in `proj` and returns `(Vof, Vva, Cx, Cy)`. |
| `0Ah`, `1Ah`, `2Ah`, `3Ah` | Raster | 1 | 4 (repeating) | **Approximate**: Streams Mode 7 matrix `(A, B, C, D)` for scanline `v`, `v+1`, `v+2`, ... continuously until the S-CPU overwrites the 4th output word with a different value (commonly `$8000`). |
| `06h` | Project | 3 | 3 | **Approximate**: Projects 3D world point `(x, y, z)` to `(H, V, M)` using stored `proj`. |
| `0Eh`, `1Eh`, `2Eh`, `3Eh` | Target | 2 | 2 | **Approximate**: Unprojects screen `(H, V)` to ground `(X, Y)` from `(Cx, Cy)` and line-`V` raster matrix. |
| `14h` | Gyrate | 6 | 3 | **Approximate**: Updates Euler angles `(Az, Ax, Ay)` by body rates `(U, F, L)` using `reciprocal()` for `1 / cos(Ax)`. |

### 3.4 Projection Limit Angle and Per-Game Coverage

The DSP-1 projection model (`Projection` in `src/chips/dsphle.rs`) tracks a tilt-angle threshold `LIMIT = 0x38CE` (~80° in `Azs`):
- Below `LIMIT`, `Vof = 0` and standard perspective division computes Mode 7 ground matrices and object projection within a few units of the chip. *Super Mario Kart*, racing/baseball titles, and level flight in *Pilotwings* play closely to hardware.
- Past `LIMIT`, the replacement clamps the view angle to `±LIMIT`, computes `Vof = Les * tan(Δ)`, adjusts raster `KK` by `cos(Δ)`, and switches to extended cycle timings (`Parameter` 541/499, `Raster` 130/118, `Project` 354, `Target` 118). Because steep camera pitches in *Lock On* rely on chip-specific behavior beyond this branch, *Lock On* does not draw ground terrain without `dsp1b.rom`.

---

## 4. DSP-2 Replacement (`src/chips/dsp2.rs`)

Used by *Dungeon Master*. Operates primarily in 8-bit `DR` mode (`DRC = 1`), dynamically toggling 16-bit mode (`DRC = 0`) and user status flags `USF0` (`$2000`) / `USF1` (`$4000`) via timed `d2.mode_at` events.

### 4.1 Persistent State and Command Dispatch

- **Persistent On-Chip RAM**: `colour` (4-bit transparent nibble), `overlay` (80-byte opaque-nibble count history), and `row` (104-byte scaling input buffer) persist across commands and across hardware `reset()`.
- **Command Masking**: `base = c & $3F` when `c & $0F == $0E`, `0x09` when `c & $0F == $0A`, otherwise `c & $0F`.
- **Idle Words**: Power-on idle word is `$0000`; each command updates `idle_word` upon completion (e.g., `$00FF` for `0Fh`/`2Eh`/`3Eh`, `$0003` for `01h`, `$FFFF` for `06h`/`0Eh`, `$0400` for `1Eh`, and specific output-derived words for `04h`/`05h`/`07h–0Ch`).

### 4.2 DSP-2 Command Table

| Command | Function | Inputs | Outputs | Details & Timing |
|---|---|---|---|---|
| `00h` | Convert 1 Bitmap Row | 4 bytes | 4 bytes | Converts 8 packed 4bpp pixels (high nibble left) into 4 SNES bitplanes. Cycle-accurate. |
| `01h` | Convert 1 Tile | 32 bytes | 32 bytes | Converts one 8×8 4bpp bitmap tile into SNES 4bpp tile layout. Cycle-accurate. |
| `02h` | Convert `N` Tiles | `1 + 32*m` bytes (`m = n` for `1..=3`, else `4`) | `32*m` bytes | Multi-tile 4bpp conversion with per-tile phase timing masks (`TILE_FIRST`, `TILE_LATER`). Cycle-accurate. |
| `03h` | Set Transparent Color | 1 byte (`c & 15`) | 0 | Stores 4-bit transparent nibble in `colour`. Cycle-accurate. |
| `04h` | Overlay 1 Byte | 2 bytes (`under`, `over`) | 1 byte | Replaces each nibble of `under` with `over` where `over` nibble != `colour`. Timing: `15 + opaque(over)`. |
| `05h` | Overlay `N` Bytes | `1 + 2*n` bytes (`n` capped at 80) | `n` bytes | Multi-byte transparent overlay; records `overlay[i]` opaque counts and reproduces the `n % 16 == 1` 16-byte tail timing quirk, `USF0` busy pulse, and `n == 0` `USF1` flag. |
| `06h` | Reverse & Swap Row | `1 + n` bytes | `n` bytes | Reverses byte order and swaps nibbles (`rotate_left(4)`) of each byte. Cycle-accurate. |
| `07h` / `08h` | 32-Bit Signed Add / Sub | 8 bytes (`a`, `b`) | 4 bytes | Signed 32-bit `a + b` / `a - b`; sets `USF1` on signed overflow (2 cycles after byte 2). Cycle-accurate. |
| `09h` / `0Ah` | 16×16 Signed Multiply | 4 bytes (`k`, `l`) | 4 bytes | Computes uPD77C25 product `m = (k * l) * 2` and packs `(m >> 17)` (high) and `((m as i16) >> 1)` (low). Cycle-accurate. |
| `0Bh` / `0Ch` | 32-Bit Unsigned / Signed Divide | 8 bytes (`a`, `b`) | 8 bytes (`q`, `r`) | Sets `USF0` while dividing; `b == 0` raises `USF1` and returns 0 bytes. Cycle-accurate timing formula based on `popcount(|q|)` and negation cycles. |
| `0Dh` | Scale 4bpp Row | `2 + ceil(a/2)` bytes (`a`, `b`, pixels) | `ceil(b/2)` bytes | Scales row of `a` pixels to `b` pixels using `step = (a << 10) / (b + 1)` (when `b < a`, samples `(j * step) >> 10`, else identity). Raises `USF1` during scaling. Timing uses a least-squares estimate (`scale_estimate`, within ~11% scaling down and ~6% up). |
| `0Eh` | Memory Test | 1 word (16-bit mode) | 1 word (`0`) | Switches `DRC = 0`, returns `0`, restores `DRC = 1` before idle. |
| `1Eh` | Data ROM Dump | 1 word (16-bit mode) | 1,024 words | Transfer shape and timing modeled; returns zeros (no mask ROM embedded). |
| `2Eh` | Version / Status | 1 word (16-bit mode) | 1 word (`$0100`) | Switches `DRC = 0`, returns `$0100`, restores `DRC = 1`. |
| `3Eh` / `0Fh` | Mode Reset / Noop | 0 | 0 | `0Fh` clears `USF1` and returns to idle (`$00FF`); `3Eh` briefly pulses 16-bit mode then narrows back to idle (`$00FF`). |

**Known DSP-2 HLE Limits**:
- `1Eh` returns zeros rather than proprietary data ROM bytes.
- `0Dh` compute latency is an analytical estimate rather than exact cycle count.
- Inputs exceeding the chip's internal buffers (`n > 80` on `05h`, `a > 208` on `0Dh`) are clamped rather than overflowing into adjacent DSP RAM.
- All commands used by *Dungeon Master* match hardware outputs and play identically to the firmware image.

---

## 5. ST010 Replacement (`src/chips/st010.rs`)

Used by *F1 ROC II: Race of Champions* / *Exhaust Heat II* (`chipset = $F6`).

### 5.1 Mailbox Protocol

1. On reset, `DspHle` enters `Stage::StStart` and offers a single 16-bit startup word (`0x0000`) on `DR`.
2. Once the S-CPU reads that word, the chip enters `Stage::StMailbox` and starts polling RAM word `$10` (bytes `$20–$21`) every **3 chip cycles** (`poll = cycles + 2`, then advancing in multiples of 3).
3. The S-CPU places command inputs in shared RAM (`$68–$6F:0000–$0FFF`), writes the command code to byte `$20`, and sets bit 7 of byte `$21` (`$80`).
4. On the first 3-cycle poll at or after the write, the replacement schedules `Next::Done` at `poll + work`, executes the command, writes all output and working words into `self.ram`, clears `self.ram[0x10] = 0`, and resumes polling at `cycles + 2`.

Command codes `9..=15` alias to `c - 8` (`st_base()`).

### 5.2 Generated Tables

- **Sine (`sine(i)`, 256 steps/turn)**: `floor(32767.0 * quarter_sine(q) + 0.5)` by quadrant, signed.
- **Arctangent (`atan_n(a, b)`, 32×32 table)**: `floor(256.0 * atan2(a, b) / (2π) + 0.5)` in 256ths of a turn.
- **Perspective Scale (`perspective(n)`, 176 scanlines `0..176`)**: `(2 * 39421 + (5*n + 44)) / (2 * (5*n + 44))` (half-up rounded `39421 / (5n + 44)`).

### 5.3 ST010 Command Table

| Command | Name | Cycles | Inputs (RAM words) | Outputs & Behavior |
|---|---|---|---|---|
| `00h` | Noop | 15 | — | Clears mailbox `ram[0x10] = 0`. **Exact**. |
| `01h` | Bearing (`atan2`) | Variable (`64–78` base + halving loop) | `r[0]=x, r[1]=y` | Reduces `(|x|, |y|)` by halving until `< 32` (preserving 1 on `1 >> 1`), looks up `atan_n`, writes `r[0..=3]` working words and `r[8] = theta`. If `x == -32768` or `y == -32768`, writes `r[0..=3]` and hangs forever (`due = u64::MAX`), matching hardware. |
| `02h` | Sort Drivers | `41 + 10n + 6n(n-1) + 15*swaps` (`2<=n<=15`) | `r[0x12]=n`, keys `r[0x20..0x20+n]`, drivers `r[0x40..0x40+n]` | Fixed-pass descending bubble sort on keys and paired driver IDs; writes `r[0] = 0` (`n` if out of range `2..=15`, `24` or `28` cycles). **Exact**. |
| `03h` | Scale 2D Vector | 31 | `r[0]=x, r[1]=y, r[2]=m` | Writes 32-bit `x*m*2` to `r[8..10]` and `y*m*2` to `r[10..12]`. **Exact**. |
| `04h` | 2D Distance | `43 + 4*(|x|<=|y|) + 2*(x<0) + 2*(y<0)` | `r[0]=x, r[1]=y` | Alpha-max-plus-beta-min approximation (`alpha = ceil(32768*2*cos(π/8)/(1+cos(π/8)))`, `beta = round(32768*2*sin(π/8)/(1+cos(π/8)))`), writes `(mx, mn)` to `r[0..2]` and rounded result to `r[8]`. **Exact**. |
| `05h` | AI Driver Step | Variable (`bearing` + turn + accel + gate) | Car block `r[0x60..0x71]` | Computes bearing to waypoint `(dx, dy)`, steers heading `r[0x66]` by `±$280`, updates speed `r[0x6A]` (brakes if `|err| >= $1000`, else accelerates toward `r[0x6C]`), moves 29-bit position `r[0x62..0x66]` via `sine()`, checks waypoint gate box `(7,127)`/`(127,7)`, advances to next waypoint (`r[0x6F..0x71]`), and sets `r[0x6E] |= 8`. |
| `06h` | Multiply | 24 | `r[0]=k, r[1]=l` | Writes 32-bit `k*l*2` to `r[8..10]`. **Exact**. |
| `07h` | Road Raster Matrices | 4,987 | `r[0]` (angle in high byte) | Writes `(i, sin, cos)` to `r[0..3]` and 176 Mode 7 matrix entries `(l*c, l*s, l*-s, l*c)` into `r[0x78..0x128]`, `r[0x128..0x1D8]`, `r[0x1D8..0x288]`, `r[0x288..0x338]` using `perspective(n)`. |
| `08h` | Rotate 2D | 44 | `r[0]=x, r[1]=y, r[2]` (angle high byte) | Writes `high(x, c) + high(y, s)` to `r[8]` and `high(y, c) - high(x, s)` to `r[9]`. |

**Known ST010 HLE Limits**:
- `00h`, `02h`, `03h`, `04h`, and `06h` are bit-exact and cycle-exact.
- `01h` and `05h` match hardware except on two entries where the chip's original arctangent table differs from `round(256·atan2/2π)`, causing AI opponents to drift slightly from their exact firmware lines over time.
- `07h` and `08h` stay within 1 unit where the chip's sine table departs from `round(32767·sin)`.

---

## 6. DSP-4 Replacement (`src/chips/dsp4.rs`)

Used by *Top Gear 3000*. Operates in 16-bit `DR` mode (`DRC = 0`) throughout and returns `DR = $FFFF` at idle.

### 6.1 Protocol and Command Support

- **Command Aliasing**: `base = 0x10 | (c & 15)` when `c >= 0x20`; `0x0E` aliases to `0x03`.
- **Implemented Commands**:
  - `00h` (Multiply): **Exact**. Takes 2 signed 16-bit inputs and returns their 32-bit signed product `(a * b)` low word first (`outputs[0] = p as u16`, `outputs[1] = (p >> 16) as u16`), including the unhandshaken first-result latency (`7 + (outputs[0] >> 15)` cycles).
  - `14h`: Returns `0x0400` (1 word, 12-cycle notice).
  - Short protocol commands (`02h`, `03h`, `05h`, `06h`, `0Ah`, `0Bh`, `0Ch`, `11h`, `12h`, `13h`, `1Fh`): Model their exact input/output transfer counts (including silent/late-input edges on `0Ah`, `0Bh`, `0Ch`, 16-word `06h`, and 1,024-word `13h`) and measured phase timings, returning zeros for uncharacterized outputs.

### 6.2 Known DSP-4 HLE Limits

Road projection, horizon, and scenery-raster commands are not yet implemented in `src/chips/dsp4.rs`. Without `dsp4.rom`, *Top Gear 3000* boots and operates its menus normally via the DSP-4 protocol and `00h` multiplier, but active race screens remain black. Providing `dsp4.rom` switches seamlessly to `DspEngine::Lle` with full race rendering.
