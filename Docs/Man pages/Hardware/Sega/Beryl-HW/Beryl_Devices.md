# Beryl Shared Devices and Data Crates

Reference manual for the non-processor hardware crates (`beryl-sms-vdp`, `beryl-sn76489`) and shared data/asset crates (`beryl-sms-colours`, `beryl-fm-tables`, `beryl-font`) in `EmuSen/Cores/Sega/Beryl-HW/`.

---

## `beryl-sms-vdp`

Video Display Processor family covering the Sega `315-5124` (Mark III / Master System I), `315-5246` (Master System II), `315-5378` (Game Gear), and Texas Instruments `TMS9918A` (SG-1000), plus shared mode 4 line composition (`compose`) used by the Genesis `315-5313` in `nephrite`.

The crate follows the non-processor contract: the caller owns the clock (`Vdp::run`), `compose` reads VRAM through a `Vram` trait, and rendered pictures store raw CRAM entries or TMS colour codes (`Vec<u16>`) rather than RGB colours.

### Hardware Variants

`Variant` encodes silicon differences across five constants:

| Feature | `SMS1` (`315-5124`) | `SMS2` (`315-5246`) | `GG` (`315-5378`) | `TMS9918A` | `GENESIS` (`compose` only) |
|---|---|---|---|---|---|
| State ID (`variant.id`) | `1` | `2` | `3` | `4` | `5` (not a `Vdp`) |
| Register table AND masks (`masks`) | Mode 4 + Graphics II | None | None | Graphics II | None |
| 224-line / 240-line modes | No / No | Yes / Yes | Yes / No | No / No | No / No |
| Mode 4 sprite zoom (`zoom` / `zoom_all`) | First `N - 4` of `N` widened | All zoomed | All zoomed | No mode 4 | Disabled |
| Left-column mask R0 bit 5 (`left_mask`) | Yes | Yes | No effect | — | Yes |
| Fine-scroll leading pixels (`fine_blank`) | Backdrop | Backdrop | Backdrop | — | Wrapped prior column |
| Right-column vertical lock (`lock_by_counter`) | Column counter 24–31 | Column counter 24–31 | Column counter 24–31 | — | Pixel ≥ 192 |
| CRAM | 32 bytes (`--BBGGRR`) | 32 bytes (`--BBGGRR`) | 64 bytes (32 × 12-bit via latch in GG mode) | None | Console's own |
| TMS9918 colour code display | Fixed 6-bit SMS palette colour | Fixed 6-bit SMS palette colour | CRAM entry `16 + code` | Raw 4-bit code | Console's own |
| Invalid text mode (`M4` set) | `M1` set | `M1` set, `M2` clear | `M1` set, `M2` clear | — | — |

### Ports and Command Protocol

- **Control write (1st byte)**: Updates the low 8 bits of `address` immediately and sets `second`.
- **Control write (2nd byte)**: Updates the upper 6 bits of `address` and 2-bit `code`, clearing `second`. Code `0` prefetches VRAM into `buffer` and increments `address`; code `2` writes the first byte to register `n & $0F` (registers 11–15 are ignored on Sega VDPs). On `TMS9918A`, bit 7 set selects register `bits 2–0` (8 registers total); otherwise bit 6 selects write vs read with prefetch.
- **Control read (status)**: Returns frame flag (bit 7), sprite overflow / 5th sprite (bit 6), and sprite collision (bit 5), clearing those flags, `line_flag`, and `second`. On Sega VDPs, `STATUS_BITS = 0xE0` marks the driven bits; bits 4–0 come from the console's undriven bus. On `TMS9918A`, all 8 bits are driven (bits 4–0 hold the 5th sprite index or the scan termination index).
- **Data write**: With `code == 3` on Sega VDPs, writes CRAM (directly on SMS; on Game Gear in `gg_mode`, even addresses latch `gg_latch` and odd addresses commit the 16-bit word `{gg_latch, byte}`). Otherwise writes VRAM (mapped through `tms::address_4k` when `TMS9918A` has R1 bit 7 clear). Always updates `buffer` and increments `address` modulo `$4000`.
- **Data read**: Returns `buffer`, then refills `buffer` from VRAM at `address` and increments `address`.
- **V/H counters**: V counter reads the current line's hardware V counter sequence; H counter returns `h_latch`, updated on a TH rising edge reported by the console (`latch_h`).

### Timing Geometry (`timing`)

The VDP clock is master clock ÷ 5 (`2` VDP clocks per dot):

- **Scanline**: `DOTS = 342`, `LINE_CLOCKS = 684`.
- **Frame**: `NTSC_LINES = 262`, `PAL_LINES = 313`.
- **H counter**: 9-bit counter running `$000`–`$127` (`H_LAST`) then jumping to `$1D2`–`$1FF` (`H_JUMP`), read by the CPU as its upper 8 bits (`$00`–`$93`, `$E9`–`$FF`).
- **V counter sequences**:
  - NTSC 192: `$00`–`$DA`, `$D5`–`$FF` (262 lines)
  - NTSC 224: `$00`–`$EA`, `$E5`–`$FF` (262 lines)
  - NTSC 240: `$00`–`$FF`, `$00`–`$05` (262 lines, wraps through zero)
  - PAL 192: `$00`–`$F2`, `$BA`–`$FF` (313 lines)
  - PAL 224: `$00`–`$FF`, `$00`–`$02`, `$CA`–`$FF` (313 lines)
  - PAL 240: `$00`–`$FF`, `$00`–`$0A`, `$D2`–`$FF` (313 lines)

| Line Event | H Counter | Timing Constant | Behaviour |
|---|---|---|---|
| V counter step & sprite scan | `$1E8` | `H_AT_STEP = 0x1E8` (dot 0) | Advances V counter; evaluates sprites and overflow flag; raises frame flag + INT on line `$C1`/`$E1`/`$F1` |
| Line counter & X scroll latch | `$1E9` | `LINE_DOT = 1` | Decrements line counter on lines `0..=active` (reloads from R10 on underflow and raises `line_flag`), reloads on all other lines; latches `hscroll` |
| Active pixel 0 | `$00A` | `PIXEL_0_DOT = 34` | Active display spans H `$00A`–`$109` (256 pixels) |
| TH H-counter latch | +1 dot | `LATCH_DELAY = 2` VDP clocks | Latches upper 8 bits of H counter 2 VDP clocks after TH rises |

- **Scroll latching**: `vscroll` (R9) is latched at the start of line 0; `hscroll` (R8) is latched at `LINE_DOT` on each line. Name table base, patterns, and CRAM are read dynamically as pixels render.
- **Power-on alignment**: `Vdp::new` starts at line `lines - 1` (the final line of a frame) with registers, VRAM, and CRAM cleared, so a console stepping one full frame from power-on finishes cleanly between pictures with nothing in flight.

### Mode 4 Composition (`compose`) and TMS9918A Modes (`tms`)

- **Mode 4 (`compose`)**:
  - Display height is 224 when `M2 & M1` alone are set (on `lines_224` chips), 240 when `M2 & M3` alone are set (on `lines_240` chips), else 192. Invalid text mode renders the backdrop with no sprites.
  - Background: coarse scroll `32 - (hscroll >> 3)`, fine scroll `hscroll & 7` leading pixels filled with backdrop (or wrapped column on `GENESIS`). Top two rows (lines 0–15) lock horizontal scroll when R0 bit 6 is set; right columns (column counter 24–31, or pixel ≥ 192 on `GENESIS`) lock vertical scroll when R0 bit 7 is set. Name table is 32×28 (Y wraps at 224) in 192-line mode and 32×32 at `(R2 bits 3–2) | $0700` in taller modes.
  - `315-5124` masks: R2 bit 0 clear forces name table bit 10 low; R3 masks pattern bits 13–6 for planes 0–1; R4 bits 2–0 mask pattern bits 13–11 for planes 2–3; R5 bit 0 masks sprite X/pattern bit 7; R6 bits 1–0 mask sprite pattern bits 12–11.
  - Sprites: Y + 1 vertical placement, 8 or 16 tall (doubled by zoom), `$D0` terminates scan in 192-line mode only, 9th sprite on an active line sets overflow and stops scan, R0 bit 3 shifts X left by 8 (no edge wrap), R6 bit 2 provides pattern bit 8 (bit 0 ignored for 8×16). Earlier sprites win priority; two opaque sprite pixels within `0..256` while display is enabled set collision.
  - Priority mix: opaque sprite pixels appear over background unless the tile's priority bit is set and the tile pixel is non-zero; R0 bit 5 masks pixels `0..8` with backdrop (CRAM entry `16 + (R7 & 0x0F)`) when `left_mask` is true.
  - Blanked display (`R1 bit 6 == 0`): renders backdrop (`Vdp::border`), suppresses sprite collisions.
- **TMS9918A modes (`tms`)**: Selected when `M4` is clear (or always on `TMS9918A`). Supports Graphics I, Graphics II (with `315-5124`/`TMS9918A` R3/R4 third-of-screen AND masks), Text (40×6 between 8-pixel borders using top 6 pattern bits), Multicolor (4×4 nibbles), and undocumented `1+2`, `2+3`, `1+3`, `1+2+3` modes. Sprites are limited to `TMS_SPRITES = 4` per line, 8×8 or 16×16 (optionally 2× magnified), early-clock shift of 32 pixels, `$D0` terminator, and collision on any two overlapping set sprite bits regardless of colour (colour 0 is transparent). On `TMS9918A`, R1 bit 7 clear enables 4K DRAM address mapping (`tms::address_4k`).
- **Game Gear window**: `GG_WINDOW = (48, 24, 160, 144)` for 192-line displays; `gg_window_top(height)` returns `24 + (height - 192) / 2` (lines `40–183` in 224-line mode).

### Dot-Stamped Renderer, Access Slots, and State

- **Dot-stamped rendering**: Each scanline advances through three scheduled events (`LINE_DOT`, end of active pixels, `LINE_CLOCKS`). Before any register, VRAM, or CRAM write or status read, the VDP renders the current line up to the current dot; the remainder renders at the line's end. Lines with no mid-line writes compose in a single pass. Sprite pixels and earliest collision dot are precomputed during sprite evaluation so `vdp.skip = true` preserves exact collision/status/interrupt timing without writing pixels.
- **Access slots**: The VDP does not drop rapid CPU data-port writes during active display; real hardware access-slot timing varies by revision and many commercial titles exceed the Mark III manual's conservative 29-cycle active-display figure. Fetch-ahead pipeline offset (a few dots ahead of pixel output) is likewise unmodelled.
- **State (`src/state.rs`)**: Serializes variant ID, `pal`/`gg_mode`, 11 registers, 16 KiB VRAM, 64-byte CRAM, `address`, `code`, `buffer`, `gg_latch`, flags, `line_counter`, `hscroll`, `vscroll`, `h_latch`, `fifth`, clock/line/event counters, and up to 8 active line sprites (reconstructing sprite pixels on load). Rejects mismatched variant/standard or invalid line/clock geometry without modifying state.

---

## `beryl-sms-colours`

Zero-dependency RGBA (`[u8; 4]`) colour tables shared by `endou` and `emusen-systems`'s SMS/GG debugger views:

- **`LEVELS: [u8; 4]`**: `[0, 85, 170, 255]` — 2-bit channel steps on the `315-5246`.
- **`BLUE_5124: [u8; 4]`**: `[0, 0x68, 170, 255]` — `315-5124` blue channel with its first non-zero step boosted to `$68`.
- **`SMS_COLOURS: [[u8; 4]; 64]`** and **`SMS1_COLOURS: [[u8; 4]; 64]`**: 64-entry tables indexed by `--BBGGRR` CRAM bytes for `315-5246` and `315-5124` respectively.
- **`gg_colour(word: u16) -> [u8; 4]`**: Converts a 12-bit Game Gear CRAM word (`----BBBBGGGGRRRR`) using 16 equal steps of `17` per 4-bit channel (`0, 17, …, 255`).
- **`TMS_COLOURS: [[u8; 4]; 16]`**: The 16 `TMS9918A` colours computed from Texas Instruments Table 2-3 luminance and colour-difference voltages (`Y`, `R-Y`, `B-Y` relative to `0.47`, converted to 8-bit RGB; code 0 maps to black over no external video).

---

## `beryl-sn76489`

Texas Instruments SN76489 and Sega VDP-integrated Programmable Sound Generator (PSG). Shared by `nephrite` (Genesis) and `endou` (Master System, Mark III, SG-1000, Game Gear).

- **Variants (`Variant { width, taps }`)**:
  - `Variant::SEGA`: 16-bit LFSR with white-noise feedback taps at bits 0 and 3 (`0x0009`) — used on Master System, Game Gear, and Genesis.
  - `Variant::DISCRETE`: 15-bit LFSR with taps at bits 0 and 1 (`0x0003`) — used on SG-1000.
- **Port writes (`write(u8)`)**:
  - Latch byte (`bit 7 == 1`): sets `latch = (v >> 4) & 7` (channel `0–3`, bit 0 selects attenuation vs tone/noise) and writes the low 4 bits.
  - Data byte (`bit 7 == 0`): writes the upper 6 bits of the latched tone's 10-bit half-period, or the low bits of attenuation / noise control. Tone registers take effect immediately on each byte; writing the noise register resets the LFSR `shift` to `1 << (width - 1)`.
- **Game Gear stereo (`write_stereo(u8)`)**: Port `$06` register (`stereo`, default `$FF`). Bits 0–3 route channels 0–3 to the right output; bits 4–7 route channels 0–3 to the left output.
- **Clocking and synthesis (`tick`, `advance`)**:
  - Runs at the chip's internal clock (input clock ÷ 16). Each counter decrements when non-zero; reaching zero reloads the counter and toggles the channel's output bit.
  - Tone channels with half-period `0` or `1` hold their output high while their counter rests (used for PCM sample playback via attenuation writes).
  - Noise reloads from `$10`, `$20`, `$40` (rates `0–2`) or tone channel 2's register (rate `3`). On a `0 -> 1` transition of the noise bit, `shift` shifts right, feeding back the parity of `shift & taps` in white-noise mode (bit 2 set) or `shift & 1` in periodic mode.
  - `VOLUME: [i32; 16]` maps attenuation `0–14` in 2 dB steps from `8192` down to `326`, and `15` to `0` (silence). Output is unipolar `[left, right]` summed from `levels()`.
  - `advance(ticks, changed)` fast-forwards across ticks where no counter reaches zero and invokes `changed(t, [left, right])` only when the output level changes.
- **State (`write_state` / `read_state`)**: Fixed block holding `tone[3]`, `volume[4]`, `noise`, `latch`, `stereo`, `counter[4]`, `bit[4]`, and `shift`.
- **Accuracy bounds**: Both variants start silent (Sega integrated behaviour) and hold tone half-periods `0` and `1` high. Discrete-chip power-on tone and analogue DC decay are unmodelled; DC removal belongs to each console's audio path.

---

## `beryl-fm-tables`

Zero-dependency crate holding the two 256-entry Yamaha FM operator ROM tables shared identically by the YM2413 (`endou`) and YM2612 (`nephrite`):

- **`LOGSIN: [u16; 256]`**: Quarter-wave log-sine attenuation in 4.8 fixed point:
  $$\text{LOGSIN}[x] = \operatorname{round}\!\left(-\log_2\!\left(\sin\!\left(\frac{(x + 0.5)\,\pi}{512}\right)\right) \cdot 256\right)$$
- **`EXP: [u16; 256]`**: Ascending exponential table with implicit leading 1,024:
  $$\text{EXP}[x] = \operatorname{round}\!\left(\left(2^{x / 256} - 1\right) \cdot 1024\right)$$

Both tables are stored as exact integer constants (independent of host `libm`) and verified by unit tests against their closed-form definitions. Chip-specific ROMs (YM2413 instrument/rhythm patches, YM2612 LFO/envelope tables) stay in their respective cores.

---

## `beryl-font`

Zero-dependency build-time crate providing the 5×7 glyph set (`src/font.txt`) used by Sega open firmware (`nephrite`'s open Sega CD BIOS and `endou`'s open boot ROMs). No runtime core links `beryl-font`; each core's `build.rs` reads the glyphs and encodes them into its console's native tile format (packed 4bpp for Genesis, 4 bitplanes for Master System / Game Gear).

- **`SOURCE: &str`**: Raw text glyph table.
- **`Glyph { char: u8, rows: [u8; 7] }`**: Seven 5-bit rows per character, with the leftmost pixel in bit 4 (`0x10`) and rightmost in bit 0 (`0x01`).
- **`glyphs() -> impl Iterator<Item = Glyph>`**: Yields all 47 glyphs in source order: space, digits `0`–`9`, uppercase `A`–`Z`, and punctuation `. , - : ! ? / ( ) '`.
