# Mercury (Game Boy / Game Boy Color) — CGB Extensions & Console Models (`EmuSen/Cores/Nintendo/MercuryRT - GB`)

Live crate: `EmuSen/Cores/Nintendo/MercuryRT - GB` (`mercuryrt`). Covers CGB-specific hardware (`src/memory/bus.rs`, `src/ppu/{mod.rs, timing.rs, render.rs}`), console model selection (`src/machine.rs`, `src/ffi/v1.rs`), and DMG-on-CGB compatibility mode (`src/ppu/compat.rs`).

---

## 1. Console Models & Mode Resolution (`src/machine.rs`, `src/ffi/v1.rs`)

`MercuryRT` exposes one create-scope setting, `Model` (category *System*, effect `none`), with three choices: `Auto` (default), `Game Boy`, and `Game Boy Color`.

`Model::console_for` resolves the target hardware at load time from `Model` and cartridge header byte `$0143` (`$00` = DMG only, `$80` = CGB-enhanced, `$C0` = CGB-only):

| Cartridge `$0143` | `Auto` | `Game Boy` | `Game Boy Color` |
|---|---|---|---|
| `$00` (DMG only) | DMG | DMG | **CGB (DMG compatibility mode)** |
| `$80` (CGB-enhanced) | CGB (colour mode) | DMG (game's monochrome path) | CGB (colour mode) |
| `$C0` (CGB-only) | CGB (colour mode) | DMG (runs game's own DMG error screen) | CGB (colour mode) |

`MemoryBus` fixes three immutable flags at construction:
- **`cgb_hardware`:** Physical console is a Game Boy Color (allocates 16 KiB VRAM and 32 KiB WRAM).
- **`cgb`:** Colour mode active (`cgb_hardware && cart.is_cgb()`).
- **`dmg_compat`:** DMG compatibility mode (`cgb_hardware && !cart.is_cgb()`).

A `$C0` cartridge forced onto `Game Boy` is deliberately booted rather than refused, matching real DMG hardware (whose boot ROM does not inspect `$0143`).

---

## 2. Game Boy Color Hardware Extensions (`cgb == true`)

### 2.1 Memory Banking
- **VRAM (`VBK`, `$FF4F`):** Bit 0 selects VRAM bank 0 or 1 (`16 KiB` total); unused upper bits read back as `1` (`0xFE | bank`).
- **WRAM (`SVBK`, `$FF70`):** Bits `2:0` select WRAM bank `1..=7` at `$D000–$DFFF` (writing `0` maps bank `1`; `$C000–$CFFF` is fixed to bank `0`; `32 KiB` total). Unused bits `7:3` read back as `1` (`0xF8 | svbk`).

### 2.2 Colour Palettes (`$FF68–$FF6B`)
- **Storage:** 8 background palettes and 8 object palettes, 4 RGB555 colors each (two 64-byte arrays, initialized to `0xFF` white at reset).
- **Access:** Indexed via `BCPS`/`BCPD` (`$FF68`/`$FF69`) and `OCPS`/`OCPD` (`$FF6A`/`$FF6B`). Bit 7 of the index register auto-increments the 6-bit index on **writes** to the data port only (reads never increment). Data ports are inaccessible during PPU Mode 3.
- **RGB555 → RGB888 expansion:** Each 5-bit channel expands via `(c << 3) | (c >> 2)` so `$1F` maps to `$FF`. No LCD color-correction curve is baked into the framebuffer; panel emulation belongs to frontend shaders.

### 2.3 Rendering Differences in Colour Mode
1. **VRAM bank 1 tile attributes:** Every tile-map entry has a paired attribute byte at the same offset in VRAM bank 1: bits `2:0` = BG palette `0..=7`, bit `3` = VRAM tile bank `0`/`1`, bit `5` = X flip, bit `6` = Y flip, bit `7` = BG-to-OAM priority.
2. **Sprite priority:** Determined solely by OAM index (earliest OAM entry wins); DMG X-coordinate sorting is bypassed.
3. **`LCDC` bit 0 reinterpretation:** Clearing bit 0 does **not** blank the background or window; instead it disables master BG/window priority so all sprites render in front of background/window pixels regardless of attribute priority bits.

### 2.4 HDMA / GDMA (`$FF51–$FF55`)
Source (`HDMA1–2`, `$FF51–$FF52`, low 4 bits ignored) and destination (`HDMA3–4`, `$FF53–$FF54`, masked to VRAM `$8000–$9FF0`, 16-byte aligned, writing into the active `VBK` bank) are write-only (reads return `$FF`).

Writing `HDMA5` (`$FF55`) controls transfer length `(val & 0x7F) + 1` blocks of 16 bytes:
- **General-purpose DMA (bit 7 = 0, when no HBlank DMA is active):** Copies all blocks immediately and charges the CPU stall accumulator (`32` CPU T-cycles per 16-byte block in single speed, `64` CPU T-cycles per block in double speed — constant `~8 µs` per block regardless of CPU speed).
- **HBlank DMA (bit 7 = 1):** Transfers one 16-byte block on each visible scanline's Mode 3 → Mode 0 transition, accumulating the same per-block stall (`32` / `64` T-cycles).
  - **Paused during `HALT`:** When the CPU enters `HALT` (`CpuBus::set_halted(true)`), HBlank transitions transfer no blocks until the CPU wakes.
  - **Cancellation:** Writing bit 7 = `0` to `$FF55` while an HBlank DMA is active stops the transfer rather than starting a GDMA; reading `$FF55` afterward returns `0x80 | remaining_minus_1`. While active, `$FF55` reads `remaining_minus_1`; when finished, `$FF55` reads `$FF`.
  - **Overflow stop:** Any transfer whose destination overflows past `$9FFF` terminates immediately.

### 2.5 Double Speed Mode (`KEY1`, `$FF4D`)
- Writing bit 0 of `KEY1` arms a speed switch; executing `STOP` while armed toggles single speed (`4.194304` MHz) ↔ double speed (`8.388608` MHz) and clears the armed bit.
- **Clock split:** CPU, `DIV`, and timers run at 2× rate; the **PPU stays at normal rate** (`4.194304` MHz, `59.7275` Hz refresh, `140,448` CPU T-cycles per frame in double speed), ticked on every other CPU cycle.
- **Known limits:** The speed switch executes instantaneously (omitting the ~2,050-cycle hardware pause); `STOP` without bit 0 armed is a no-op; `OPRI` (`$FF6C`) and `RP` (`$FF56` IR) are not actively emulated (unmapped/stubbed in colour mode).

---

## 3. DMG-on-CGB Compatibility Mode (`dmg_compat == true`, `src/ppu/compat.rs`)

When a monochrome (`$0143 == $00`) cartridge runs under `Model = Game Boy Color`, `MercuryRT` reproduces the CGB boot ROM handoff and hardware lockouts without requiring a proprietary BIOS file.

### 3.1 Post-Boot CPU Register Handoff
- `A = $11`, `F = $80`, `C = $00`, `DE = $0008`, `SP = $FFFE`, `PC = $0100`.
- `B`: 8-bit sum of cartridge title bytes `$0134..=$0143` if the cartridge has Nintendo licensee (`$014B == $01`, or `$014B == $33` with `$0144..=$0145 == "01"`), else `$00`.
- `HL`: `$991A` when `B` is `$43` or `$58`, else `$007C`.

### 3.2 Automatic Compatibility Palette Selection (`src/ppu/compat.rs`)
Implements the CGB boot ROM's title-checksum colorization tables (79 title checksums, 29 fourth-letter disambiguators, 94 palette/shuffle entries, 29 offset triples, 30 four-color RGB555 palettes):
1. Non-Nintendo licensees always receive default palette `0` (BG `7FFF 1BEF 6180 0000`, OBJ0/OBJ1 `7FFF 421F 1CF2 0000`).
2. Nintendo cartridges compute the 8-bit sum of `$0134..=$0143` and look up the first matching index:
   - Indices `0..=64` map directly to palette numbers `0..=64`.
   - Indices `65..=78` disambiguate via the 4th title byte (`$0137`) across up to 3 rows (`65 + col`, `79 + col`, or `93`), falling back to palette `0` on mismatch.
3. Each entry's 3-bit shuffle mask assigns `(e0, e1, e2)` to `BG`, `OBJ0`, and `OBJ1`:
   - `BG = e2`
   - `OBJ0 = e0` if bit 0 set, else `e2`
   - `OBJ1 = e1` if bit 2 set, else (`e0` if bit 1 set, else `e2`)
4. Palettes are uploaded via auto-incrementing `OCPD` (OBJ0 + OBJ1, 16 bytes) and `BCPD` (BG0, 8 bytes), leaving `BCPS` reading `$C8` (`$88 | $40`) and `OCPS` reading `$D0` (`$90 | $40`). Unused BG palettes `1..=7` and OBJ palettes `2..=7` remain white.
- **Not modelled:** Startup D-pad/button palette override combinations (no boot logo screen), logo VRAM tilemap leftovers, and AGB mode (`B + 1`).

### 3.3 Rendering & Register Lockouts in Compatibility Mode
- **Rendering (`write_shade`):** Runs the DMG pipeline (no bank-1 tile attributes, DMG X-coordinate sprite priority, DMG `LCDC` bit 0 blanking), then maps each 2-bit shade (after `BGP` / `OBP0` / `OBP1` indirection) through CGB BG palette 0 or OBJ palette 0/1 (selected by sprite attribute bit 4).
- **Locked registers:**

| Registers | Behavior in DMG Compatibility Mode |
|---|---|
| `KEY0` (`$FF4C`), `KEY1` (`$FF4D`) | Read `$FF`, writes ignored (`STOP` never switches speed) |
| `HDMA1–5` (`$FF51–$FF55`), `RP` (`$FF56`) | Read `$FF`, writes ignored |
| `BCPD` (`$FF69`), `OCPD` (`$FF6B`) | Read `$FF`, writes ignored (palettes locked after boot) |
| `OPRI` (`$FF6C`), `SVBK` (`$FF70`) | Read `$FF`, writes ignored (WRAM locked to bank 1) |
| `VBK` (`$FF4F`) | Readable (`$FE \| bank`) and writable for CPU bank selection; renderer reads bank 0 only |
| `BCPS` (`$FF68`), `OCPS` (`$FF6A`) | Index registers remain readable (`$C8` / `$D0` at boot) and writable |
