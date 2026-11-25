# Mercury (Game Boy / Game Boy Color) — Hardware Reference (`EmuSen/Cores/Nintendo/MercuryRT - GB`)

Live crate: `EmuSen/Cores/Nintendo/MercuryRT - GB` (`mercuryrt`, Core ABI v1, dependencies: `emusen-native`, `silvermillennium-sm83`).

`MercuryRT` emulates the Nintendo Game Boy (DMG) and Game Boy Color (CGB), including DMG-on-CGB compatibility mode. For CGB extensions, HDMA, double speed, model selection, and DMG-on-CGB compatibility palettes, see [`Mercury_CgbAndModels.md`](Mercury_CgbAndModels.md).

---

## 1. Crate & Module Map

| Module | Responsibility |
|---|---|
| `src/cpu/{mod.rs, alu.rs, opcodes.rs}` | Sharp SM83 CPU, M-cycle bus ticking (`CpuBus`), interrupt dispatch, `HALT`/`STOP`, illegal opcode error |
| `src/ppu/{mod.rs, timing.rs, render.rs, compat.rs}` | 160×144 LCD controller, mode 0–3 timing, STAT IRQ line, background/window/sprite compositor, CGB boot compatibility palettes |
| `src/apu/{mod.rs, channels.rs}` | 4-channel APU (Pulse 1 with sweep, Pulse 2, Wave, Noise), DIV-clocked frame sequencer, stereo mixer, DC-blocking capacitors |
| `src/memory/{bus.rs, cartridge.rs, mappers.rs, mod.rs}` | Address decode, I/O register masks, DIV/TIMA timer, OAM DMA, CGB HDMA, joypad, cartridge header & MBC1/2/3/5 boards |
| `src/machine.rs`, `src/state.rs` | `Model` resolution, frame loop (`70,224` T-cycles/frame at `4,194,304` Hz), `MERC` save states (versions `5..=11`) |
| `src/debug.rs`, `src/ffi/v1.rs` | Debugger hooks/breakpoints/stack, Core ABI v1 exports, `Model` setting, SM83 disassembly via `silvermillennium-sm83` |

---

## 2. Clocks & CPU — Sharp SM83 (`src/cpu/`)

- **Master clock:** `CPU_CLOCK_HZ = 4,194,304` Hz (`1,048,576` M-cycles/s in single speed; `8,388,608` Hz in CGB double speed).
- **Frame duration:** `CYCLES_PER_FRAME = 70,224` PPU T-cycles (`154` scanlines × `456` dots, `~59.7275` Hz).
- **M-cycle bus granularity:** Every instruction ticks the bus (`CpuBus::tick(4)`) before each memory read/write and on internal cycles so timers, PPU modes, OAM DMA, and interrupts advance mid-instruction rather than at instruction end. Stack write ordering for `PUSH`, `RST`, `CALL`, `CALL cc`, and `RET cc` places reads and pushes on their exact hardware M-cycles.
- **Interrupts:** 5 interrupt sources (`$0040` VBlank, `$0048` STAT, `$0050` Timer, `$0058` Serial, `$0060` Joypad) with `IME`, delayed `EI` enable (including `EI; EI` chaining before dispatch), `HALT` wake, and the DMG `HALT` bug (`halt_bug` double-fetching the next opcode byte when `IME == 0` and `(IE & IF) != 0`).
  - **Interrupt vector re-evaluation:** During the 5-M-cycle interrupt dispatch, `IE & IF` is re-sampled after pushing the high byte of `PC` onto the stack (matching hardware `ie_push` behavior: if the push overwrote `$FFFF` and cleared the triggered bit, dispatch selects the next highest pending interrupt or jumps to `$0000`).
- **Post-boot state (no boot ROM):** Execution starts at `PC = $0100`, `SP = $FFFE` with hardware post-boot register values:
  - **DMG:** `AF = $01B0`, `BC = $0013`, `DE = $00D8`, `HL = $014D`.
  - **CGB (colour mode):** `AF = $1180`, `BC = $0000`, `DE = $FF56`, `HL = $000D`.
  - **CGB (DMG compatibility mode):** See [`Mercury_CgbAndModels.md`](Mercury_CgbAndModels.md).
- **Illegal opcodes:** The 11 unimplemented SM83 opcodes return `IllegalOpcode { opcode, pc }` rather than silently executing as `NOP`.
- **Disassembly:** Provided by [`silvermillennium-sm83`](../SilverMillennium-HW/README.md) (`DEBUG_DISASSEMBLE`).

---

## 3. PPU — 160×144 LCD Controller (`src/ppu/`)

- **Geometry & modes:** `160 × 144` pixels, `154` scanlines (`0..=143` visible, `144..=153` VBlank), `456` T-cycles per scanline:
  - **Mode 2 (OAM Scan):** `OAM_SCAN_CYCLES = 80` dots.
  - **Mode 3 (Drawing):** `BASE_DRAWING_CYCLES = 172` + `(SCX & 7)` dots (`drawing_end`).
  - **Mode 0 (HBlank):** Remaining dots up to `456`.
  - **Mode 1 (VBlank):** Scanlines `144..=153`.
- **Rendering:** Each visible scanline is rendered at the Mode 3 → Mode 0 transition (`dot == drawing_end`):
  - **Background & Window:** Separate tile maps (`$9800` / `$9C00`), unsigned (`$8000`) or signed (`$8800`) tile data addressing, internal window line counter advancing only on lines where the window actually renders (`WY <= LY`, `WX <= 166`, window enabled). On DMG, clearing `LCDC` bit 0 blanks background and window to shade 0 (white).
  - **Sprites (OAM):** Up to 10 sprites per scanline selected in OAM order (`8×8` or `8×16`). On DMG, overlapping sprite pixels resolve by leftmost X coordinate first, breaking ties by OAM index.
  - **DMG output palette:** Four neutral grayscale shades (`0xFF`, `0xAA`, `0x55`, `0x00`) mapped through `BGP` (`$FF47`), `OBP0` (`$FF48`), and `OBP1` (`$FF49`); display tinting is left to frontend shaders.
- **Access blocking:** CPU reads/writes to OAM during Modes 2–3 and VRAM during Mode 3 are blocked (returning `$FF` / ignored writes). CGB palette data ports (`BCPD`/`OCPD`) are likewise blocked during Mode 3.
- **STAT IRQ line:** Combined rising-edge detector over LY=LYC, Mode 0, Mode 1, and Mode 2 sources (`update_stat_line`).
  - Turning the LCD off resets `LY = 0`, parks `STAT` mode at `0`, freezes the `LY=LYC` coincidence flag as-is (saved in state version 11), and suppresses all STAT interrupts while disabled.
  - On entering VBlank (line `144`), an enabled Mode 2 (OAM) STAT source also pulses the STAT line (matching hardware `vblank_stat_intr` on DMG and CGB).

---

## 4. APU — 4-Channel Stereo Audio (`src/apu/`)

- **Clocking & frame sequencer:** Driven directly by the falling edge of the system `DIV` counter bit (bit 12 in single speed, bit 13 in CGB double speed — `512` Hz), stepping length counters (`256` Hz), sweep (`128` Hz), and volume envelopes (`64` Hz).
- **Channels (`src/apu/channels.rs`):**
  - **Channel 1 (Pulse + Sweep):** 4 duty patterns, frequency sweep with negate-mode tracking (leaving negate mode after a negate calculation disables the channel; state version 10 tracks the negate flag), envelope, length counter.
  - **Channel 2 (Pulse):** 4 duty patterns, envelope, length counter.
  - **Channel 3 (Wave):** 16-byte / 32×4-bit wave RAM (`$FF30–$FF3F`), volume shift (`0%`, `100%`, `50%`, `25%`), length counter.
  - **Channel 4 (Noise):** 15-bit / 7-bit width-selectable LFSR, clock shift & divisor, envelope, length counter.
- **Length counter & power quirks:**
  - Extra length-counter clocking on enable/trigger during the first half of the frame sequencer period is modelled.
  - Powering off the APU (`NR52` bit 7 = 0) clears registers (`$FF10–$FF25`) and preserves length counters on DMG while clearing them on CGB.
- **Stereo mixer & output (`Mixer`):** Maps DAC outputs through `NR51` panning and `NR50` left/right master volume into interleaved stereo 44,100 Hz PCM (`cycles_per_sample = 4,194,304.0 / 44,100.0`), with per-channel DC-blocking high-pass capacitors (`left_capacitor`, `right_capacitor`, serialized from state version 8). Supports per-channel mutes (`MUTES`) and non-destructive audio peek (`AUDIO_PEEK`).

---

## 5. Memory Map, Timer, DMA, Joypad & Cartridges (`src/memory/`)

### 5.1 CPU Address Space & I/O
- `$0000–$7FFF`: Cartridge ROM (bank 0 fixed at `$0000–$3FFF`, switchable bank at `$4000–$7FFF`).
- `$8000–$9FFF`: 8 KiB VRAM (16 KiB across 2 banks via `VBK` on CGB hardware).
- `$A000–$BFFF`: External cartridge RAM / RTC registers (when enabled by MBC).
- `$C000–$DFFF`: 8 KiB WRAM (32 KiB across 8 banks via `SVBK` on CGB hardware; `$E000–$FDFF` Echo RAM mirrors `$C000–$DDFF`).
- `$FE00–$FE9F`: 160-byte OAM (`40` sprites × 4 bytes).
- `$FF00–$FF7F`: Hardware I/O registers (`unused` registers and unused bits return `1` / `$FF` per hardware read masks; `HDMA1–4` are write-only).
- `$FF80–$FFFE`: 127-byte High RAM (HRAM); `$FFFF`: Interrupt Enable (`IE`).
- **ROM patches:** Game Genie patch map supported via Core ABI `ROM_PATCHES`.

### 5.2 Timer, OAM DMA & Joypad
- **Timer (`DIV`/`TIMA`/`TMA`/`TAC`):** Internal 16-bit system counter (`DIV` exposes upper 8 bits; any write resets the full 16-bit counter and can trigger falling-edge `TIMA` increment or APU frame-sequencer tick). `TIMA` overflow models the 4-T-cycle reload delay window (writing `TIMA` during reload cancels the reload and IRQ; writing `TMA` during the reload cycle overrides the reloaded value).
- **OAM DMA (`$FF46`):** Transfers 160 bytes (`160` M-cycles = `640` T-cycles) with hardware's 1-M-cycle startup delay (including restart delay while an active transfer is running, saved in state version 9). During active OAM DMA, CPU reads outside HRAM return `$FF` (or OAM blocking as modelled by `bus.rs`).
- **Joypad (`$FF00`):** 2×4 button matrix (direction / action select bits 4–5); asserts Joypad IRQ (`IF` bit 4) on high-to-low transition of any selected button line.

### 5.3 Supported Cartridge Boards (`src/memory/cartridge.rs`, `src/memory/mappers.rs`)
Cartridge RAM is backed by `.srm` battery files (Core ABI battery slot 0) when the header type indicates battery support.

| Mapper | Cartridge Type Codes (`$0147`) | Features |
|---|---|---|
| `NoMbc` | `$00`, `$08`, `$09` | Fixed 32 KiB ROM, optional 8 KiB RAM |
| `Mbc1` | `$01`, `$02`, `$03` | Up to 2 MiB ROM / 32 KiB RAM, simple/advanced banking modes, automatic **MBC1M multicart** detection (`bank2_shift = 4`) |
| `Mbc2` | `$05`, `$06` | Up to 256 KiB ROM, built-in `512 × 4`-bit internal RAM (`$A000–$A1FF` mirrored) |
| `Mbc3` | `$0F`, `$10`, `$11`, `$12`, `$13` | Up to 2 MiB ROM, up to 32 KiB RAM, Real-Time Clock (`$08–$0C` latch/registers) |
| `Mbc5` | `$19`, `$1A`, `$1B`, `$1C`, `$1D`, `$1E` | 9-bit ROM bank (up to 8 MiB, bank 0 selectable), up to 128 KiB RAM, rumble bit (`has_rumble`) |

Unsupported cartridge types (such as HuC1, HuC3, MBC6, MBC7, Pocket Camera, MMM01) return `RomError::UnsupportedType` at load.

---

## 6. Core ABI v1, Save States & Accuracy Boundaries

- **Core ABI capabilities (`src/ffi/v1.rs`):** `AUDIO_PEEK`, `MUTES`, `ROM_PATCHES`, `DEBUG`, `DEBUG_STACK`, `DEBUG_REGISTERS`, `DEBUG_DISASSEMBLE`. Claims systems `gb` (`.gb`) and `gbc` (`.gbc`).
- **Save states (`MERC`, `src/machine.rs`):** Writes version `11`, loads versions `5..=11`. From version `7` onward, the state records `cgb_hardware` (`0` = DMG, `1` = CGB) and automatically rebuilds the machine to match the state's console on load while preserving host settings (held buttons, channel mutes, debug hooks, ROM patches, mixer rate).
- **Known accuracy boundaries:**
  - **Scanline-boundary PPU transitions:** `LY`, `STAT` mode bits, STAT IRQ sources, and access blocking transition together at fixed dot boundaries (`0`, `80`, `drawing_end`, `456`) rather than at staggered per-T-cycle hardware offsets. Consequently, `lcdon_timing` (first line after LCD-on skipping Mode 2 / 2-T-cycle delay), early `LY = 0` on line 153, `stat_lyc_onoff`, and fine Mode-2/Mode-0 STAT skew (`intr_2_*`) are not yet modelled; rendering also runs per scanline rather than via a pixel FIFO (mid-scanline `SCX`/`BGP` raster effects apply at the line level, and sprite-penalty Mode 3 extension beyond `SCX & 7` is not added).
  - **DMG STAT write quirk:** Spurious STAT IRQ on writing `$FF41` during Mode 0/1/LYC on DMG is not modelled.
  - **Wave channel hardware quirks:** Wave RAM corruption on re-trigger and active-playback wave RAM read-back byte offsetting are not modelled.
  - **Timer `rapid_toggle`:** Rapid `TAC` enable/frequency toggling edge cases beyond standard `TIMA`/`TMA` reload-cycle timing are not modelled.
  - **Serial link & IR:** Serial transfers clock internally without a peer link cable; CGB infrared port (`$FF56` RP) is unimplemented (reads `$FF`).
  - **Super Game Boy (`$0146 == $03`):** Header flag is parsed, but SGB packet protocol, borders, and palettes are not implemented.
