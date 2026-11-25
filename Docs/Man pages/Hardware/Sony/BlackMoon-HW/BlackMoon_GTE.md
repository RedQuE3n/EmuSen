# BlackMoon GTE (`blackmoon-gte`)

Crate path: `EmuSen/Cores/Sony/BlackMoon-HW/blackmoon-gte/`.
Implements the Sony PlayStation Geometry Transformation Engine attached to R3000A COP2. `BUILT = true`.

## Public Interface

- **`Gte`**: holds all 64 registers (`Registers { data: [u32; 32], control: [u32; 32] }`) and no clock or in-flight command state.
  - `data(r)` / `set_data(r, v)` — `MFC2`/`SWC2` and `MTC2`/`LWC2` transfers on data registers `0..32`.
  - `control(r)` / `set_control(r, v)` — `CFC2` and `CTC2` transfers on control registers `0..32`.
  - `register(n)` — debugger read of register `0..64` (data first, then control).
  - `command(word) -> u32` / `command_observed(word, observer) -> u32` — executes the COP2 command specified by the low 25 bits of the instruction word, writes all outputs immediately, and returns the command's cycle count for the caller's interlock.
- **Tables and helpers**:
  - `DATA_REGISTERS: [&str; 32]` (`VXY0`..`LZCR`) and `CONTROL_REGISTERS: [&str; 32]` (`RT11RT12`..`FLAG`).
  - `COPROCESSOR: emusen_mips::Coprocessor` — register names and the 22 canonical GTE command mnemonics for `emusen-mips`.
  - `CYCLES: [u32; 64]` — cycle duration for every 6-bit command number (`0x00..=0x3F`).
  - `divide(h, sz3) -> (u32, bool)` and `UNR_TABLE: [u8; 257]` — hardware-exact Newton-Raphson reciprocal division.
- **Precision observer seam**: `Observer::projected(&Projection)` and `Unobserved`.

## Register Semantics

Register transfers (`MTC2`, `LWC2`, `CTC2`, `MFC2`, `SWC2`, `CFC2`) never saturate and never modify `FLAG`. Writes take effect immediately for subsequent reads and commands (no register write-hazard delay is modelled).

| Register(s) | Write Behavior | Read Behavior |
|---|---|---|
| `VZ0`–`VZ2`, `IR0`–`IR3`, `RT33`, `L33`, `LB3`, `DQA`, `ZSF3`, `ZSF4` | Low 16 bits sign-extended to 32 bits | Stored 32-bit word |
| `OTZ`, `SZ0`–`SZ3` | Low 16 bits zero-extended to 32 bits | Stored 32-bit word |
| `H` (`control[26]`) | Low 16 bits sign-extended to 32 bits on readback; commands use `H` as unsigned 16-bit | Stored sign-extended word |
| `SXYP` (`data[15]`) | Pushes FIFO: `SXY0 ← SXY1 ← SXY2 ← value` | Mirrors `SXY2` |
| `IRGB` (`data[28]`) | Unpacks 5-bit R/G/B fields into `IR1..IR3` as `field * 0x80` | Packs `IR1..IR3` (`IR / 0x80`, clamped `0..=0x1F`) into 15-bit RGB |
| `ORGB` (`data[29]`) | Read-only (ignored) | Same packed 15-bit RGB view as `IRGB` |
| `LZCS` (`data[30]`) | Full 32-bit word | Full 32-bit word |
| `LZCR` (`data[31]`) | Read-only (ignored) | Leading count of matching sign bits of `LZCS` (`1..=32`) |
| `FLAG` (`control[31]`) | Keeps bits `12..=30`; sets bit `31` if any error bit in `30..=23` or `18..=13` is set | Stored word |
| All others | Full 32-bit word | Stored 32-bit word |

## Command Execution and Arithmetic Rules

Command word bit fields:
- Bit `19`: `sf` — shift right by `12` (`sf = 1`) or `0` (`sf = 0`).
- Bit `10`: `lm` — lower saturation bound for `IR1..IR3` (`0` when `lm = 1`, `-0x8000` when `lm = 0`).
- Bits `17..=18`, `15..=16`, `13..=14`: `MVMVA` matrix, vector, and translation selectors (ignored by all other commands).
- Bits `0..=5`: 6-bit command number.

`FLAG` is cleared at the start of every command and finalized (including summary bit 31) when the command finishes.

### Accumulator, Overflow, and Saturation Rules

- **`MAC1`–`MAC3` 44-bit wrapping accumulators**: Each row sum (`T * 0x1000 + M1*V1 + M2*V2 + M3*V3`) executes in a signed 44-bit wrapping accumulator. Overflow (`> 0x7FF_FFFF_FFFF` or `< -0x800_0000_0000`) is checked after **every individual addition**, so a single command can raise both the positive and negative overflow flags for the same `MACn`.
- **`IR1`–`IR3` saturation source**: `IR1..IR3` are saturated from the **32-bit truncated `MACn` register** (after `SAR (sf * 12)`), not from the wider 44-bit sum. The same 32-bit truncation and overflow check applies to the depth-cue intermediate difference `(FCn << 12) - MACn`, which always saturates with `lm = 0`.
- **`MAC0` full-sum saturation**: `MAC0` checks signed 32-bit overflow (bits 16/15) and stores its low 32 bits, while `SX2`/`SY2` (`MAC0 >> 16`, clamped `-0x400..=0x3FF`, bits 14/13), `IR0` (`MAC0 >> 12`, clamped `0..=0x1000`, bit 12), and `OTZ` (`MAC0 >> 12`, clamped `0..=0xFFFF`, bit 18) saturate from the **full 64-bit `MAC0` sum**.
- **`RTPS` / `RTPT` specifics**:
  - `SZ3` is pushed from the `MAC3` sum shifted right by 12 regardless of `sf`, clamped to `0..=0xFFFF` (bit 18).
  - `FLAG` bit 22 (`IR3` saturation) is evaluated against `-0x8000..=0x7FFF` on `sum3 >> 12` regardless of `lm`, while `IR3` itself still clamps using `lm`.
  - `RTPT` performs depth cueing (`DQA * quotient + DQB` → `IR0`) only on the third vertex (`V2`).
- **`MVMVA` special selectors**:
  - Translation selector `2` (far color `FC`) computes `MACn = (Mn2*V2 + Mn3*V3) >> (sf * 12)`; the dropped first portion `FCn * 0x1000 + Mn1*V1` is still evaluated for `MAC` overflow and `IR` saturation flags (against `lm = 0`), then discarded.
  - Matrix selector `3` uses the hardware garbage matrix `[-R*0x10, +R*0x10, IR0; RT13, RT13, RT13; RT22, RT22, RT22]`.
- **Unlisted command numbers**: All 42 unlisted encodings clear `FLAG`. Command `0x00` executes as `RTPS` (taking 16 cycles); the other 41 unlisted commands leave all registers other than `FLAG` unchanged.

### Division (`UNR_TABLE`)

`RTPS` and `RTPT` divide `H` by `SZ3` using the PlayStation's unsigned Newton-Raphson reciprocal algorithm (`src/divide.rs`):
- If `H >= SZ3 * 2`, return `0x1_FFFF` and set `FLAG` bit 17 (divide overflow).
- Otherwise normalize both operands left by `z = clz16(SZ3)`, look up `u = UNR_TABLE[(d - 0x7FC0) >> 7] + 0x101` (where `UNR_TABLE[i] = max(0, (0x40000 / (i + 0x100) + 1) / 2 - 0x101)`), run two fixed-point Newton steps, and return `min(0x1_FFFF, (n * d + 0x8000) >> 16)` without setting bit 17.

## Command Cycle Table and Interlock

| Cmd | Mnemonic | Cycles | Cmd | Mnemonic | Cycles | Cmd | Mnemonic | Cycles |
|---|---|---:|---|---|---:|---|---|---:|
| `0x01` | `RTPS` | 15 | `0x16` | `NCDT` | 44 | `0x2A` | `DPCT` | 17 |
| `0x06` | `NCLIP` | 8 | `0x1B` | `NCCS` | 17 | `0x2D` | `AVSZ3` | 5 |
| `0x0C` | `OP` | 6 | `0x1C` | `CC` | 11 | `0x2E` | `AVSZ4` | 6 |
| `0x10` | `DPCS` | 8 | `0x1E` | `NCS` | 14 | `0x30` | `RTPT` | 23 |
| `0x11` | `INTPL` | 8 | `0x20` | `NCT` | 30 | `0x3D` | `GPF` | 5 |
| `0x12` | `MVMVA` | 8 | `0x28` | `SQR` | 5 | `0x3E` | `GPL` | 5 |
| `0x13` | `NCDS` | 19 | `0x29` | `DCPL` | 8 | `0x3F` | `NCCT` | 39 |
| `0x14` | `CDP` | 13 | | | | | | |

Unlisted command numbers carry their hardware timing counts in `CYCLES` (e.g. `0x00` = 16, `0x02` = 13, `0x17` = 31, `0x3C` = 18).

**Interlock behavior (implemented by the core's bus):**
- When a command issues at bus clock `now` (after instruction fetch), `gte_ready = now + cycles`.
- Any COP2 access (`MFC2`, `CFC2`, `SWC2`, `MTC2`, `CTC2`, `LWC2`, or another GTE command) arriving at `now >= gte_ready` proceeds with zero stall.
- Any COP2 access arriving while `now < gte_ready` stalls until `gte_ready + 1` (one cycle after the command completes).

## Precision Side-Channel (`Projection`)

When `command_observed` runs `RTPS`, `RTPT`, or `0x00`, it invokes `observer.projected(&Projection)` once per projected vertex with:
- `vertex: u8` (`0`, `1`, or `2`)
- `sums: [i64; 3]` — camera-space `TR * 0x1000 + RT * V` (12 fractional bits, before `sf` shift)
- `quotient: u32` — `H / SZ3` (17-bit, 16 fractional bits)
- `x: i64`, `y: i64` — `MAC0` screen-coordinate sums before `>> 16` and saturation
- `z: u16` — pushed `SZ3`
- `word: u32` — pushed `SXY2`
- `h: u16`, `offset: [i32; 2]` — `H`, `OFX`, and `OFY` read by the command

## State Block Layout

`Gte` serializes a fixed **256-byte** block via `emusen_native::State`:

| Offset | Size | Type | Field |
|---:|---:|---|---|
| 0 | 128 | `u32[32]` | `Data` |
| 128 | 128 | `u32[32]` | `Control` |

The bus's `gte_ready` deadline belongs to the console bus state rather than `blackmoon-gte`.
