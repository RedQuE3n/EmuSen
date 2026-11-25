# BlackMoon SPU Voice Engine (`blackmoon-spu`)

Crate path: `EmuSen/Cores/Sony/BlackMoon-HW/blackmoon-spu/`.
`#![no_std]` crate with zero dependencies and zero allocations. Implements the sample-by-sample audio voice and reverb engine shared by `diamond` (PlayStation SPU at 44,100 Hz, 512 KiB RAM) and `sapphire` (both PlayStation 2 SPU2 cores at 48,000 Hz, 2 MiB RAM).

## Responsibility Split

| Owned by `blackmoon-spu` | Owned by Each Console Core |
|---|---|
| `voice::Voice`: per-voice state, SPU-ADPCM decode, 4-point Gaussian interpolation, ADSR phase machine, per-voice volume sweeps, `voice::step` pitch modulation | Register addresses, bit layouts, and read/write side effects |
| `envelope::Envelope` & `Rate`: shared ADSR and volume-sweep step engine | Sample clock and output sample rate (44,100 Hz vs 48,000 Hz) |
| `noise::Noise`: LFSR noise generator and countdown timer | Voice noise-enable routing and timer reset on register write |
| `reverb::Reverb`: half-rate stereo reverb algorithm, 39-tap FIR down/up-sampling filter, work-area address wrap | Reverb register decoding into `reverb::Params` and `reverb::Area` |
| `tables`: 512-entry `GAUSS`, 39-tap `FIR`, 5 ADPCM filter pairs (`POS`, `NEG`) | Voice mixing, dry/wet routing, DMA/FIFO transfers, capture buffers, CD/external inputs, IRQ comparison |

Sound RAM is lent by the core as a power-of-two `&[u8]` (or `&mut [u8]` for reverb); all byte addresses wrap via `ram.len() - 1`. Every sound-RAM read or write invokes `Touch::touch(&mut self, byte_addr: u32)` so the core can evaluate its own IRQ address match (`()` disables watching).

## Voice Processing (`voice.rs`)

### Units and Key On/Off

- `Voice::start` and `Voice::repeat` are stored as `u32` addresses in **8-byte units** (matching PlayStation `SSA`/`LSAX`; PlayStation 2 halfword addresses shift right by 2 before entering `Voice`). `Voice::address` is the current ADPCM block byte offset.
- `Voice::key_on(ram)` sets `address = (start * 8) & mask`, clears `counter`, `decoded`, the 31-sample ring (`samples`), the 2-sample ADPCM history (`history`), and `envelope`, enters `Phase::Attack`, and latches `repeat = address >> 3` if the first block's flag byte has bit 2 (loop start) set.
- `Voice::key_off()` switches `phase` to `Phase::Release`.

### SPU-ADPCM Decode

Each 16-byte block encodes `BLOCK_SAMPLES = 28` signed 16-bit samples in 7 halfwords following a 2-byte header:
- Byte 0: low nibble `shift` (`0..=12`, values `13..=15` clamp to `9`), bits `4..=6` `filter` (`0..=4`, values `5..=7` use `(0, 0)`).
- Five filter coefficient pairs (`POS` / `NEG`, divided by 64): `(0, 0)`, `(60, 0)`, `(115, -52)`, `(98, -55)`, `(122, -60)`.
- Prediction computes each product's `>> 6` shift separately before summing:
  `sample = clamp((nibble_shifted >> shift) + (h0 * POS[f] >> 6) + (h1 * NEG[f] >> 6), -0x8000, 0x7FFF)`.
- Words are decoded lazily as the pitch counter reaches them; on block end, any remaining words are decoded so decoder history stays exact, the last 3 samples copy into `samples[0..3]` for Gaussian continuity, and block flag byte 1 controls looping:
  - Bit 0 (loop end): jump to `repeat * 8`. If bit 1 (loop repeat) is clear, also enter `Phase::Release` and zero `envelope.level`.
  - Bit 2 (loop start): latched on block entry into `repeat = address >> 3`.

### Pitch Modulation and Gaussian Interpolation

- `voice::step(pitch, modulator)` computes the 4.12 fixed-point counter increment. When modulated by the previous voice's post-envelope output (`modulator: Some(prev)`):
  `step = (((pitch as i16 as i64) * (prev as i32 + 0x8000)) >> 15) as u32 & 0xFFFF`, then clamped to `MAX_STEP = 0x4000`.
- When not in noise mode, `Voice::tick` selects Gaussian index `g = (counter >> 4) & 0xFF` across the 4 samples `s[0..4]` and sums all four products before shifting right by 15:
  `(GAUSS[0xFF - g]*s[0] + GAUSS[0x1FF - g]*s[1] + GAUSS[0x100 + g]*s[2] + GAUSS[g]*s[3]) >> 15`.
- `Voice::out` is scaled by the current `envelope.level` **before** the envelope advances for the next sample. `Voice::volumes()` then steps both left/right volume sweeps and returns `[left, right]` as `i32`.

## Envelopes and Volume Sweeps (`envelope.rs`)

Both ADSR phases and swept volume registers (`bit 15 == 1`) share `Envelope::tick(Rate)`:
- `Rate` carries `shift` (`0..=31`), `step` (`0..=3`), `decreasing`, `exponential`, and `negative` (inverted phase).
- Base step magnitude is `7 - rate.step` (bitwise-inverted when `decreasing != negative`), shifted left by `max(0, 11 - shift)`.
- Counter increment is `0x8000 >> max(0, shift - 11)`.
- **Exponential increase** slows above `level > 0x6000` (shift `< 10`: `step >>= 2`; shift `== 10`: `step >>= 1`, `increment >>= 1`; shift `>= 11`: `increment >>= 2`).
- **Exponential decrease** scales `step = (step * level) >> 15`.
- Rates with `shift * 4 + step != 0x7F` clamp `increment >= 1` (so shifts `26..=30` step every 32,768 samples); rate `0x7F` (`shift = 31, step = 3`) never steps.
- When `counter + increment` sets bit 15, `counter` resets to `0` and `level` clamps to `[-0x8000, 0x7FFF]` (increasing), `[0, 0x7FFF]` (positive decreasing), or `[-0x8000, 0]` (negative decreasing).
- Direct volume registers (`bit 15 == 0`) set `current.level = (reg << 1) as i16`.

## Noise Generator (`noise.rs`)

`Noise::tick(shift, step)` subtracts `(step & 3) + 4` from `timer`. When `timer < 0`, `level` shifts left by 1 with inverted parity of bits `15, 12, 11, 10` in bit 0, and `timer` recharges by `0x2_0000 >> (shift & 0xF)` (applied a second time if still negative).

## Reverb (`reverb.rs`)

`Reverb::tick(ram, watch, params, area, enabled, input)` runs at half the sample rate by alternating left and right sides on consecutive samples:
- Inputs are pushed into a 64-entry ring (`RING = 64`) and decimated through the 39-tap symmetric `FIR` filter (`>> 15`).
- Each side executes the reflections (`mSAME`, `mDIFF`), 4-tap comb filter (`mCOMB1..4`), and 2-stage all-pass filter (`mAPF1..2`) in hardware access order. When `enabled == false` (reverb master write disable), writes still call `Touch::touch` without modifying sound RAM.
- All reverb offsets wrap within `Area { start, end }` (degenerate areas `< 2` bytes clamp to 2 bytes). After the right-side sample, `Reverb::address` advances by 2 bytes, wrapping to `area.start` at `area.end` and clamping to `>= area.start`.
- Half-rate outputs are zero-stuffed into the output rings and upsampled through the 39-tap `FIR` (`>> 14`, saturated to `i16`).
