# Endou — Sega Game Gear Hardware (Jadeite Module)

Hardware reference for the Game Gear system (`gg`) inside `endou` (`EmuSen/Cores/Sega/Endou - Master System/`), implemented across `src/gg.rs`, `src/ggio.rs`, `src/eeprom.rs`, `src/boot/gg.s`, and `src/jadeite.rs`. Core Z80, bus, mappers, and VDP timing shared with the Master System are covered in [Endou_Hardware.md](Endou_Hardware.md) and [Beryl_Devices.md](../Beryl-HW/Beryl_Devices.md).

> [!NOTE]
> The `gg` system descriptor (`src/v1.rs`) is currently marked `development: true` (with its `bios_gg` setting hidden) while player-facing integration is finalized; the underlying hardware emulation, LCD modes, stereo PSG, system ports, EEPROM, and 1 KiB open boot ROM are implemented and tested.

---

## Console Models, Timing, and Mode Selection

- **Models (`src/model.rs`)**: `GameGear` (`gg`, export) and `JapaneseGameGear` (`ggjp`, domestic Japan). Both run NTSC exclusively (`53,693,175 Hz` master clock, `262` scanlines, `59,736` Z80 cycles per frame; `has_pal() == false`).
- **VDP and PSG**: `beryl_sms_vdp::Variant::GG` (`315-5378`) and `beryl_sn76489::Variant::SEGA` with per-channel stereo routing. Work RAM is `8 KiB`; undriven bus reads return `$FF` (`floating_bus() == false`).
- **GG mode vs Master System cartridge mode (`media::Media::gg_mode`)**: Hardware selects mode from a cartridge pin. `endou` enters native GG mode for `Console::Gg` images unless the `TMR SEGA` header explicitly declares a Master System region code (`$3` or `$4`), in which case the machine runs in Master System compatibility mode (`vdp.gg_mode = false`).
- **Controller (`src/v1.rs`)**: Single port (`ports() == 1`, `gg.pad`), 7 buttons (D-pad, 1, 2, and Start). In GG mode, Start is read via port `$00` bit 7 and does not trigger NMI; in Master System cartridge mode, Start acts as the Pause button (rising-edge NMI).

---

## 12-Bit LCD and Master System Shrink (`src/gg.rs`)

### Native Game Gear Mode (`160 × 144`, 12-Bit Colour)

- **CRAM**: `64` bytes holding `32` 12-bit entries (`----BBBBGGGGRRRR`), written via an even-address latch (`gg_latch`) and committed as a 16-bit word on odd-address writes. Colours convert through `beryl_sms_colours::gg_colour` (16 equal steps of `17` per 4-bit channel).
- **TMS9918 modes on `315-5378`**: Colour codes `0–15` display through CRAM entries `16–31`.
- **LCD viewport (`gg::window`)**: Crops the central `160 × 144` pixels (`GG_WINDOW = (48, 24, 160, 144)`) from the VDP's `256`-wide picture. In `224`-line mode, the window stays fixed in frame position and starts at line `40` (`gg_window_top(224) = 40`, lines `40–183`).
- **Left-column mask**: VDP R0 bit 5 has no effect on `315-5378` (`left_mask = false`).

### Master System Compatibility Mode (`160 × 146` LCD Shrink)

When a Master System cartridge runs on the Game Gear (`vdp.gg_mode == false`), `315-5378` uses 32-byte 6-bit CRAM (`SMS_COLOURS`) and `gg::Shrink` maps the full picture onto all `146` rows of the `160 × 146` LCD panel without per-frame allocation:

- **Horizontal subpixel shrink (240 → 160 columns)**: Takes the central `240` columns (`FIRST_COLUMN = 8`, columns `8..248`) and combines every three source pixels `(a, b, c)` into two output pixels by RGB subpixel elements: `[a.R, a.G, b.B]` and `[b.R, c.G, c.B]`.
- **Vertical 4-tap filter (218 → 146 rows)**: Maps `218` vertical lines (`LINES = 218`: lines `3..221` of a 224-line display, or a 192-line display padded with `13` lines of VDP `border` colour above and below). For panel row `y`, `s = first + 3 * (y / 2) + (if y % 2 == 0 { -1 } else { 0 })`, and each RGB component averages four consecutive lines with integer weights `(1, 2, 2, 1) / 6` rounded to nearest (`(sum + 3) / 6`). On a 192-line display this produces 8 pure border rows at the top (`0..8`) and bottom (`138..146`).

---

## I/O Ports, Stereo, and TH Latch (`src/ggio.rs`, `src/bus.rs`)

In GG mode (`vdp.gg_mode == true`), ports `$00`–`$06` are decoded separately from `$3E`/`$3F`. In Master System cartridge mode, ports `$00`–`$06` do not exist: reads return `$FF`, even writes reach `$3E`, and odd writes reach `$3F`.

| Port | Read (GG Mode) | Write (GG Mode) | Power-On Read |
|---|---|---|---|
| `$00` | Bit 7: Start (`0` = pressed); Bit 6: `NJAP` (`1` = export, `0` = Japan); Bit 5: `NNTS` (`0` = NTSC); Bits 4–0: `0` | Ignored | `$C0` (export) / `$80` (Japan) |
| `$01` | EXT parallel data (`PC0–PC6`; inputs read `1`, outputs read latched `data`; `TON`/`RON` force `PC4`/`PC5` high) | Latch `data` | `$7F` |
| `$02` | EXT pin direction + `NINT` (`1` = input) | Set `direction` | `$FF` |
| `$03` | Serial transmit byte (`transmit`) | Set `transmit`; if `TON` set, holds `TXFL` busy for `10 * Z80_HZ / BAUD` clocks | `$00` |
| `$04` | Serial receive byte (`$FF`; Gear-to-Gear cable unmodelled) | Ignored | `$FF` |
| `$05` | Serial control bits 7–3 (`BAUD`, `RON`, `TON`, `INT`) + `TXFL` in bit 0 (`RXRD`/`FRER` clear) | Set writable bits 7–3 (`v & 0xF8`) | `$00` |
| `$06` | `$FF` | SN76489 stereo register (`sound.write_stereo`) | `$FF` |

- **Serial baud rates (`BAUD`)**: Bits 7–6 of port `$05` select `4800`, `2400`, `1200`, or `300` bps. Each transmitted frame takes 10 bit times (start + 8 data + stop) against `Z80_HZ = 53_693_175 / 15`.
- **Stereo routing (port `$06`)**: Bits 0–3 enable PSG channels 0–3 on the right output; bits 4–7 enable channels 0–3 on the left output. Powers on at `$FF` (all channels on both sides).
- **Port `$3E` ignored bits**: `model.memory_control_ignored() = 0xE4` — bits 7, 6, 5, and 2 have no effect on a Game Gear, keeping cartridge and I/O permanently enabled. Only bit 4 (`WORK_RAM_OFF`) and bit 3 (`BIOS_OFF`) function.
- **Port `$3F` and TH H-counter latch**: The Game Gear behaves like an export Master System II for port `$3F` (`io_control() == true`). Writing `$3F` (or any odd port `$07`–`$3F` in GG mode) updates TH/TR directions and levels; a rising edge on either TH pin latches the VDP H counter (`vdp.latch_h()`).

---

## 93C46B Serial EEPROM (`src/eeprom.rs`, `src/cart.rs`)

Six released Game Gear baseball cartridges in `board::TABLE` use `Mapper::SegaEeprom`: Sega's ROM bank mapper with no cartridge RAM and a Microchip `93C46B` (`64 × 16-bit` = `128` bytes, persisted as the battery file, erased state `$FF`).

- **Board register (`cart.rs`)**: Mapped at `$8000` (`EEPROM_REGISTER`) only while `$FFFC` bit 4 (`EEPROM_ENABLE = 0x10`) is set. Bit 3 of `$FFFC` maps no RAM on EEPROM boards.
  - Bit 0 (`EEPROM_DI`): `DI` input on write; returns `DO` on read when driven, or the last written `DI` bit when `DO` is high-Z.
  - Bit 1 (`EEPROM_CLK`): Serial clock (samples on rising edge).
  - Bit 2 (`EEPROM_CS`): Chip select (active high; falling edge resets to `Standby` and floats `DO`).
- **Microwire protocol (`eeprom::Eeprom`)**:
  - Powers on write-protected (`enabled = false`); `EWEN` (`00 11xxxx`) enables writes/erases and `EWDS` (`00 00xxxx`) disables them.
  - Commands begin with `CS = 1` and `DI = 1` on a rising `CLK` (`START`), followed by 2-bit opcode + 6-bit word address:
    - `READ` (`10`): Drives a dummy `0`, then shifts words MSB-first (`D15..D0`), auto-incrementing address modulo `64` while `CS` stays high.
    - `WRITE` (`01`) / `WRAL` (`00 01xxxx`): Shifts 16 data bits, then begins a self-timed program cycle (stored little-endian in the 128-byte backing buffer).
    - `ERASE` (`11`) / `ERAL` (`00 10xxxx`): Sets one word or all 64 words to `$FFFF` via a self-timed cycle.
  - **Self-timed durations** (at `3,579,545 Hz`): `WRITE` / `ERASE` = `WRITE_CLOCKS = 7,159` (`2 ms`), `ERAL` = `ERAL_CLOCKS = 21,477` (`6 ms`), `WRAL` = `WRAL_CLOCKS = 53,693` (`15 ms`). Raising `CS` again enters `Phase::Status`, driving `DO = 0` while `clock < busy_until` and `1` once ready.

---

## 1 KiB Open Boot ROM Contract (`src/boot/gg.s`, `src/boot.rs`)

- **Mapping**: When port `$3E` bit 3 is clear (reset value `$E3`), the `1,024`-byte boot ROM maps at `$0000`–`$03FF` (`bios_window() = 0x0400`) while `$0400`–`$BFFF` remains mapped to the cartridge slot. Disabling bit 3 (`$A8`) swaps `$0000`–`$03FF` back to the cartridge, so hand-over executes from RAM.
- **Open boot image (`OPEN_BOOT_GG`)**: Assembled by `build.rs` from `src/boot/gg.s` together with 27 compact 1bpp 5×7 glyphs (`A–Z` and `.`) from `beryl-font` packed at `GG_FONT_AT = 0x400 - 7 * 27 - 3` (`$0340`).
- **Execution contract**:
  1. Executes `DI`, `IM 1`, `LD SP,$DFF0` (with `RETI` at `$0038` and `RETN` at `$0066`), waits for VDP V counter `$B0`, and mutes all four PSG channels.
  2. Initializes VDP display, unpacks 1bpp glyphs to bitplane 0, writes black/white CRAM in the active mode's format (detected by reading port `$00`: low 5 bits `0` in GG mode vs `$FF` in Master System mode), and displays `"ENDOU OPEN BOOT"` inside the `160 × 144` LCD window (columns `6–25`, rows `3–20`).
  3. Checks for `"TMR SEGA"` at `$7FF0`, `$3FF0`, or `$1FF0` (checksum and region bytes are not checked, matching Game Gear boot ROM behaviour). If absent, displays `"THE GAME CANNOT / START. IT HAS NO / SEGA HEADER."` and halts.
  4. Hand-over: Disables display, clears VRAM, CRAM, and work RAM, loads Mark III power-on VDP registers (`[0x36, 0xA0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFB, 0x00, 0x00, 0x00, 0xFF]`), writes `$A8` to `$C000`, copies `OUT ($3E),A; JP $0000` (`5` bytes) to `$C700`, waits for V counter `$FF`, clears VDP status, leaves ports `$00`–`$06` at power-on defaults, and jumps to `$C700` with `A = $A8`, `SP = $DFF0`, `IM 1`, interrupts disabled.
- **Player boot ROM override**: Setting `bios_gg` (`open` default, `player` opt-in) selects optional firmware file `bios.gg` (file 4, exact `1,024` bytes; mismatched sizes return `BAD_FILE`). Save states (`ENDO`) record boot mode and boot ROM CRC-32 as well as `ggio::Ports` and `eeprom::Eeprom` state.
