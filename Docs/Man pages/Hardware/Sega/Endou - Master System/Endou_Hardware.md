# Endou — Sega Master System, Mark III, and SG-1000 Hardware

Hardware and emulation reference for `endou` (`EmuSen/Cores/Sega/Endou - Master System/`), EmuSen's Core ABI v1 Rust core for the Sega Master System, Mark III, SG-1000, and Game Gear. Game Gear–specific video, I/O, EEPROM, and 1 KiB boot ROM details are covered in [Endou_GameGear.md](Endou_GameGear.md). Shared chip internals (`beryl-z80`, `beryl-sms-vdp`, `beryl-sms-colours`, `beryl-sn76489`, `beryl-fm-tables`) are documented under [Beryl-HW](../Beryl-HW/README.md).

---

## Console Models and Silicon Matrix

`endou` models six 8-bit home consoles (`src/model.rs`) plus two Game Gear variants:

| Model (`model` key) | VDP | PSG | YM2413 FM | Work RAM | Control Ports `$3E`/`$3F` | Reset Button | Floating Bus | BIOS |
|---|---|---|---|---|---|---|---|---|
| `MarkIII` (`mark3`) | `315-5124` (`SMS1`) | Sega | None | 8 KiB | Yes (Japanese I/O) | No | Yes | None |
| `MarkIIIFm` (`mark3fm`) | `315-5124` (`SMS1`) | Sega | Add-on (`FmUnit::AddOn`) | 8 KiB | Yes (Japanese I/O) | No | Yes | None |
| `JapaneseSms` (`jsms`) | `315-5124` (`SMS1`) | Sega | Built-in (`FmUnit::BuiltIn`) | 8 KiB | Yes (Japanese I/O) | No | Yes | 8 KiB (no header check) |
| `Sms1` (`sms1`) | `315-5124` (`SMS1`) | Sega | None | 8 KiB | Yes (Export I/O) | Yes | Yes | 8 KiB (export check) |
| `Sms2` (`sms2`) | `315-5246` (`SMS2`) | Sega | None | 8 KiB | Yes (Export I/O) | No | No (`$FF`) | 8 KiB (export check) |
| `Sg1000` (`sg1000`) | `TMS9918A` | Discrete | None | 1 KiB | None | No | Yes | None |

- **Default model selection (`src/v1.rs`)**: For `sms`, `auto` selects `JapaneseSms` when the market is Japan and `Sms2` for all export markets (Americas, Europe, Brazil). For `sg1000`, `auto` selects `Sg1000` (overridable to `MarkIII`).
- **Regions**: `sms.jp` (Japan, NTSC), `sms.us` (Americas, NTSC), `sms.eu` (Europe, PAL), `sms.br` (Brazil, NTSC). Europe alone runs PAL; all other markets run NTSC.

---

## Master Clocks and Frame Timing

`src/machine.rs` and `src/bus.rs` derive all device clocks from exact rational master crystal ratios:

| Parameter | NTSC | PAL |
|---|---|---|
| Master crystal | `53,693,175 Hz` | `53,203,424 Hz` |
| Z80 clock (master ÷ 15) | `3,579,545 Hz` | `3,546,894.93 Hz` |
| VDP clock (master ÷ 5, `VDP_PER_Z80 = 3`) | `10,738,635 Hz` | `10,640,684.8 Hz` |
| Z80 clocks per scanline (`LINE`) | `228` | `228` |
| Scanlines per frame | `262` | `313` |
| Z80 clocks per frame | `59,736` | `71,364` |
| Frame rate | `59.922743 Hz` (`53,693,175 / 896,040`) | `49.701459 Hz` (`53,203,424 / 1,070,460`) |
| Audio output rate (`AUDIO_RATE`) | `48,000 Hz` | `48,000 Hz` |

- **Bus-owned scheduling**: Each Z80 machine cycle advances `bus.clock` before its access (`fetch` 4, `read` 3, `write` 3, `input` 4, `output` 4, `acknowledge` 6 T-states). The VDP and sound chips catch up on every port access, interrupt check, and frame boundary (`bus.settle()`). Instruction overshoot past `frame_end` carries into the next frame so long-term frame and audio rates stay exact.
- **CPU power-on**: Starts via `Z80::power_on()` (PC, I, R, IFF1, IFF2, IM cleared; all other register pairs `$FFFF`).

---

## Memory Map and Port `$3E` Enables

| Address Range | Mapping |
|---|---|
| `$0000`–`$BFFF` | Active slot medium (Cartridge ROM/RAM and/or BIOS ROM per port `$3E`). Card and expansion slots are empty and undriven. |
| `$C000`–`$FFFF` | Work RAM (`8 KiB` mirrored every `$2000` on Mark III / SMS; `1 KiB` mirrored every `$0400` on SG-1000) unless disabled by port `$3E` bit 4 or overridden by Sega mapper RAM (`$FFFC` bit 4). Writes to `$FFFC`–`$FFFF` always reach Sega mapper registers while also writing work RAM when enabled. |

- **Port `$3E` (`memory_control`)**: Active-low enables for expansion (`bit 7`), cartridge (`bit 6`, `CARTRIDGE_OFF`), card (`bit 5`), work RAM (`bit 4`, `WORK_RAM_OFF`), BIOS (`bit 3`, `BIOS_OFF`), and I/O chip (`bit 2`, `IO_OFF`). Absent on SG-1000.
- **BIOS and cartridge overlap**: An 8 KiB BIOS mirrors across `$0000`–`$BFFF` while enabled. When both BIOS and cartridge are enabled simultaneously, the cartridge wins on `Sms1` (`MarkIII` has no BIOS) and the two bytes are bitwise ANDed on `Sms2`.
- **Undriven bus reads**: `Sms2` returns `$FF`. `Sms1`, `MarkIII`, `MarkIIIFm`, `JapaneseSms`, and `Sg1000` return the last byte driven on the data bus (`data_bus`).
- **Game Genie patches**: Up to `$BFFF` (`PATCH_HIGH`), matching cartridge reads by address and optional compare byte (`bus::Patch`).

---

## Cartridge Identification and Mappers

`src/media.rs` strips any 512-byte copier header (`len > 512 && len % 0x4000 == 512`), accepts images up to `4 MiB`, and scans `$7FF0`, `$3FF0`, and `$1FF0` for the 16-byte `TMR SEGA` header. `src/board.rs` selects the cartridge board strictly in order of precedence without write heuristics:

1. **CRC-32 table (`TABLE`)**: 17 Master System battery-backed Sega releases (8 KiB SRAM, `Mapper::Sega`, `battery: true`), *The Castle* (`SgRam`, 8 KiB volatile RAM), and 6 Game Gear 93C46B EEPROM releases (`Mapper::SegaEeprom`).
2. **Codemasters header (`$7FE0`–`$7FEF`)**: Validated by non-zero bank count, zeroed bytes `$7FEA`–`$7FEF`, and 16-bit little-endian word checksum (excluding `$7FF0`–`$7FFF`) summing with its complement at `$7FE8` to zero (`Mapper::Codemasters`, 8 KiB RAM).
3. **ROM size**: Images `≤ 48 KiB` (`PLAIN_MAX`) use `Mapper::Plain`; larger images use `Mapper::Sega` with 32 KiB volatile RAM.

| Mapper | Power-On Banks | Bank Registers | Cartridge RAM |
|---|---|---|---|
| `Plain` | Fixed (mirrored) | None | None |
| `SgRam` | Fixed | None | `$8000`–`$BFFF` (mirrored by RAM size) |
| `Sega` | Slots 0/1/2 = `0, 1, 2` | `$FFFC` control, `$FFFD`–`$FFFF` 16 KiB slots 0–2 (`$0000`–`$03FF` fixed to bank 0) | `$FFFC` bit 3 maps 16 KiB into `$8000`–`$BFFF` (bit 2 selects 2nd 16 KiB bank); bit 4 maps over `$C000`–`$FFFF` |
| `Codemasters` | Slots 0/1/2 = `0, 1, 0` | Writes to `$0000`–`$3FFF`, `$4000`–`$7FFF`, `$8000`–`$BFFF` set slots 0–2 (no fixed first 1 KiB) | Slot 1 register bit 7 maps 8 KiB RAM at `$A000`–`$BFFF` |

- **Sega bank shifting**: `$FFFC` bits 1–0 select a bank-number offset (`0x00`, `0x18`, `0x10`, `0x08`) applied whenever `$FFFD`–`$FFFF` is subsequently written.

---

## I/O Port Map and Settled Hardware Quirks

Only the low 8 bits of the Z80 port address are decoded, primarily by `A7`, `A6`, and `A0` (`src/bus.rs`, `src/io.rs`):

| Port Range | Even Read | Odd Read | Even Write | Odd Write |
|---|---|---|---|---|
| `$00`–`$3F` | Undriven | Undriven | Memory control (`$3E`) | I/O control (`$3F`) |
| `$40`–`$7F` | VDP V counter (undriven on SG-1000) | VDP H counter (undriven on SG-1000) | SN76489 PSG | SN76489 PSG |
| `$80`–`$BF` | VDP data | VDP status (`STATUS_BITS` driven + undriven low 5 bits on SMS) | VDP data | VDP control |
| `$C0`–`$FF` | Controller port A (`$DC`, if I/O enabled) | Controller port B (`$DD`, if I/O enabled) | `$F0` YM2413 address, `$F2` audio control (FM models only) | `$F1` YM2413 data (FM models only) |

- **Controller ports (`$DC`/`$DD`)**: Active-low throughout. `$DC` holds Port A Up/Down/Left/Right/B1/B2(TR) in bits 0–5 and Port B Up/Down in bits 6–7. `$DD` holds Port B Left/Right/B1/B2(TR) in bits 0–3, Reset button in bit 4 (`1` when absent or unpressed; only `Sms1` has Reset), cartridge `CONT` high in bit 5, and Port A/B `TH` in bits 6–7. On `Sg1000`, `$DD` bits 7–4 read `1`.
- **Region detection via `$3F` (`io.rs`)**: Export models (`Sms1`, `Sms2`) drive `TH` and `TR` output levels back when configured as outputs in `$3F` (bits 0–3 direction `1` = input, bits 4–7 level). Japanese models (`MarkIII`, `MarkIIIFm`, `JapaneseSms`) read `TH` outputs as `0` regardless of output level and keep `TR` as an input. Any rising edge on either `TH` pin latches the VDP H counter (`vdp.latch_h()`).
- **Pause button**: Edge-triggered on NMI (`$0066`) at the start of a frame when `PAUSE` transitions from released to pressed (not a joystick port bit).
- **FM audio control (`$F2`)**: Writable only on FM models (`MarkIIIFm` keeps bit 0; `JapaneseSms` keeps bits 1–0). Readable only when I/O is disabled (`$3E` bit 2 set): bits 1–0 return the stored value, bits 4–2 read `0`, and bits 7–5 reflect bits 11, 7, and 3 of the scanline counter since power-on (`bus.clock / LINE`).
- **Interrupt acknowledge**: No device drives the bus during Z80 acknowledge; returns `$FF` on `Sms2` and `data_bus` on floating-bus models.

---

## Audio Mixing and YM2413 (OPLL)

`src/sound.rs` mixes `beryl-sn76489` (clocked at Z80 clock ÷ 16, scaled by `GAIN = (1, 2)` so a full-volume channel is `4,096`) and the YM2413 (`src/opll.rs`, 1 sample every `CLOCKS_PER_SAMPLE = 72` Z80 clocks, scaled by `FM_GAIN = 6`) through `emusen_native::steps::Steps` to `48,000 Hz` stereo PCM.

- **Port `$F2` audio routing**:
  - `FmUnit::AddOn` (Mark III FM Sound Unit): bit 0 selects FM (`1`) or PSG (`0`); never both simultaneously.
  - `FmUnit::BuiltIn` (Japanese Master System): bit 0 enables FM; PSG is enabled whenever bits 0 and 1 are equal (`00` = PSG only, `01` = FM only, `10` = mute, `11` = PSG + FM).
- **YM2413 synthesis rules (`src/opll.rs`)**:
  - **Operator ROMs**: Uses `beryl_fm_tables::{LOGSIN, EXP}` directly (`operator()` computes 12-bit signed output; half-wave rectification returns `!0` on negative phases).
  - **DAC quantization (`dac`)**: Quantizes each carrier's magnitude top 8 bits plus bias on the two output pins (idle offset removed, yielding `−4` for quiet negative outputs and `0` for quiet positive outputs). Rhythm channels are doubled (`2 * rhythm_channel(...)`).
  - **Phase & modulation**: 19-bit phase accumulator (`MUL2` multiplier table), modulator feedback averages the last two shifted outputs (`(half(out[0]) + half(out[1])) >> (8 - fb)`), and the carrier consumes the modulator's previous sample output. Modulators lead carriers by 1 sample on the sample counter (`lead(slot)`).
  - **Envelope generator**: Five stages (`Damp`, `Attack`, `Decay`, `Sustain`, `Release`), 7-bit attenuation (`0..=127` in `0.375 dB` steps; `QUIET = 124`, `SILENT = 127`). Key-on enters `Damp` at rate 12 until `>= QUIET`, then enters `Attack` (resetting phase; carrier damp completion also resets its modulator's phase on melody/BD channels, and effective AR `>= 60` jumps immediately to `0`). Falling envelopes reaching `QUIET` cut to `SILENT` on the next update. Modulators hold level on key-off; carrier release uses rate 5 when sustain bit is set, `op.rr` when sustained patch, or rate 7 for percussive patches.
  - **Key scaling and LFO**: `KSL` attenuates at `1.5`, `3`, or `6 dB/octave` for codes `1`, `2`, `3`. Vibrato cycles 8 steps of `1,024` samples (`0, ±top/2, ±top` in half-F-number units). Tremolo steps a `0..=105` triangle every `64` samples (`am >> 3`).
  - **Rhythm mode (`$0E` bit 5)**: Channels 7–9 become Bass Drum (2-operator FM), High Hat + Snare Drum, and Tom-Tom + Top Cymbal driven by a 23-bit noise LFSR (`bit = (noise ^ (noise >> 14)) & 1`) and shared `cymbal_bit()` phase tap logic.
- **Measured instrument ROM (`src/opll/tables.rs`)**:
  - `INSTRUMENTS` (patches `1–15`, 8 register bytes each): 12 patches (Violin, Guitar, Clarinet, Oboe, Trumpet, Organ, Horn, Synthesizer, Vibraphone, Synth Bass, Acoustic Bass, Electric Guitar) match hardware output exactly; Piano, Flute, and Harpsichord are best-fit approximations.
  - `RHYTHM_INSTRUMENTS` (channels 7–9): Best-fit envelopes and spectra for Bass Drum, High Hat / Snare Drum, and Tom-Tom / Top Cymbal.

---

## Open Boot ROM Contract and Player BIOS Override

`build.rs` assembles `src/boot/` with `beryl-z80` (`asm` feature) and `beryl-font` into two 8 KiB Master System images (`OPEN_BOOT` for export models, `OPEN_BOOT_JAPAN` for `JapaneseSms`) plus the 1 KiB Game Gear image (`OPEN_BOOT_GG`).

- **Bus reset with BIOS**: Port `$3E` resets to `RESET_MEMORY_CONTROL = 0xE3` (BIOS, work RAM, and I/O enabled; card, cartridge, and expansion off).
- **Boot sequence**:
  1. Executes `DI`, `IM 1`, `LD SP,$DFF0`, writes `$E3` to `$3E` and `$A5` to `$3F`, silences the PSG, initializes VDP display and `beryl-font` tiles, shows `"ENDOU OPEN BOOT"`, and copies the slot-probing routine into work RAM at `$C700`.
  2. Probes card (`$CB`), cartridge (`$AB`), and expansion (`$6B`) slots in order. If empty (`starts_empty` supported for `sms`), re-enables BIOS and displays `"NO GAME IS IN THE CONSOLE."`.
  3. On export models (`Sms1`, `Sms2`), validates `TMR SEGA` at `$7FF0`/`$3FF0`/`$1FF0`, region `$4`, valid size code (`$A`–`$F`, `$0`–`$2`), and 16-bit ROM checksum (reading `$0000`–`$7FFF` directly and paging remaining banks through `$FFFF` into `$8000`–`$BFFF`, excluding the 16-byte header). Failures display an explanatory screen recommending Mark III and halt. `OPEN_BOOT_JAPAN` skips header validation.
  4. Hand-over: Restores `$FFFF = 2` if paged, clears VRAM/CRAM and work RAM (leaving `$C000 = $AB`, the stub at `$C700`, and `$DFFF = 2` if paged), sets VDP registers to Mark III power-on values (`[0x36, 0xA0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFB, 0x00, 0x00, 0x00, 0xFF]`), clears VDP status, waits for V counter `$FF`, writes `$AB` to `$3E`, and jumps to `$0000` with `A = $AB`, `SP = $DFF0`, `IM 1`, interrupts disabled.
- **Player BIOS override (`src/v1.rs`)**: Controlled per console via settings `bios_sms1`, `bios_sms2`, `bios_jsms` (`open` default, `player` opt-in) backed by optional firmware files `bios_sms1.sms` (file 1), `bios_sms2.sms` (file 2), and `bios_jsms.sms` (file 3). Only 8 KiB (`BIOS_SIZE = 0x2000`) images are currently mapped; larger BIOS images containing built-in games are rejected with `BAD_FILE`.

---

## Save States and Accuracy Bounds

- **Save-state format (`src/state.rs`)**: Magic `"ENDO"`, current version `7` (backward-compatible read of version `6` on non-FM consoles). Binds console, model, boot mode, and BIOS CRC-32 (`FOREIGN` on mismatch).
- **Known accuracy bounds**:
  - VDP access-slot write drops and pipeline fetch-ahead offsets are unmodelled (matching `beryl-sms-vdp`).
  - No analogue audio output stage (DC blocking filter / board low-pass) is modelled; output is scaled linear chip sums.
  - Only 8 KiB Master System BIOS ROMs are accepted for player override; larger built-in-game BIOS images are unmapped.
  - YM2413 Piano, Flute, Harpsichord, and rhythm patches are best-fit approximations rather than exact ROM bitstreams.
  - Card and expansion slots are unpopulated; light phaser, paddle, 3-D glasses, and sports pad peripherals are not modelled.
