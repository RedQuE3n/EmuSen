# Moon (NES) — Hardware Reference (`EmuSen/Cores/Nintendo/MoonRT - NES`)

Live crate: `EmuSen/Cores/Nintendo/MoonRT - NES` (`moonrt`, Core ABI v1, dependencies: `emusen-native`, `emusen-6502`).

`MoonRT` emulates the Nintendo Entertainment System (NTSC 2A03/2C02, PAL 2A07/2C07, and Dendy 6527P/6538). For PAL and Dendy timing, PPU, APU, and DMA differences, see [`Moon_PAL.md`](Moon_PAL.md).

---

## 1. Crate & Module Map

| Module | Responsibility |
|---|---|
| `src/cpu/{mod.rs, opcodes.rs}` | Ricoh 2A03/2A07 6502 core, cycle-exact bus accesses, interrupt polling, official and undocumented opcodes |
| `src/ppu/{mod.rs, timing.rs, render.rs}` | 2C02/2C07/6538 PPU registers, per-dot timing, per-dot sprite-0 hit, scanline composition, `nes.ppu` raw output |
| `src/apu/{mod.rs, channels.rs}` | 5-channel APU (Pulse 1/2, Triangle, Noise, DMC), 6-step frame sequencer, non-linear DAC, output filters |
| `src/memory/{bus.rs, cartridge.rs, mappers.rs, mod.rs}` | CPU bus decode, open bus, OAM/DMC DMA halts, iNES / NES 2.0 loader, 16 cartridge mappers, controllers |
| `src/model.rs` | Single owner of NTSC, PAL, and Dendy clock ratios, scanline counts, and hardware feature switches |
| `src/machine.rs`, `src/state.rs` | Frame loop, reset sequencing, `MOON` save-state serialization (versions 3–7) |
| `src/ffi/{v1.rs, units.rs, mod.rs}` | Core ABI v1 exports, `region` setting, `nes.ppu` raw picture, debugger hooks, 9 inspectable hardware units |

---

## 2. CPU — Ricoh 2A03 / 6502 (`src/cpu/`)

- **Bus-cycle execution:** Every CPU cycle performs exactly one bus read or write, advancing the PPU (3 dots on NTSC/Dendy, 3.2 dots on PAL) and APU (1 CPU cycle) around the access (`MemoryBus::tick` runs the first 2 dots—or 3 on a PAL long cycle—and steps the APU; `end_cycle` runs the final dot and updates NMI edge detection). Dummy reads on read-modify-write, indexed page crosses, and branches hit the real bus.
- **Decimal mode disabled:** The `D` flag (`FLAG_D`, bit 3) is stored in `P` and preserved on stack/flags, but `ADC` and `SBC` execute binary arithmetic only.
- **Vectors & reset:** `NMI = $FFFA`, `RESET = $FFFC`, `IRQ/BRK = $FFFE`. `RESET` consumes `RESET_CYCLES = 8` cycles before fetching the first instruction.
- **Interrupt sampling:** NMI is edge-triggered (`nmi_line` rising edge latches `nmi_pending`); IRQ is level-triggered (`irq_line && !I`). Both are sampled on the penultimate cycle of each instruction (`nmi_sampled_earlier`/`irq_sampled_earlier` pipeline) with branch-crossing suppression (`suppress_just_arrived_irq`), and NMI preempts IRQ when both are pending.
- **Undocumented opcodes:** Implemented across the full 256-opcode matrix (`SLO`, `RLA`, `SRE`, `RRA`, `SAX`, `LAX`, `DCP`, `ISC`, `ANC`, `ALR`, `ARR`, `SBX`, `USBC` `$EB`, `ANE`/`LXA` using `UNSTABLE_MAGIC = 0xEE`, `SHA`/`SHX`/`SHY`/`TAS`, `LAS`, multi-byte `NOP`s, and `JAM` halting the CPU until reset).
- **Disassembly:** Provided through the shared [`emusen-6502`](../../Shared/CpuTables.md) crate (`DEBUG_DISASSEMBLE`).

---

## 3. PPU — Ricoh 2C02 (`src/ppu/`)

### 3.1 Timing vs Rendering Granularity
The PPU clock runs **per dot** (`0..=340` per scanline, `262` lines on NTSC, `312` on PAL/Dendy), while pixel composition runs **per scanline** at dot `256`:
- **Exact dot:** VBlank flag set/clear (scanline 241 dot 1 / pre-render dot 1), NMI assertion, `$2002` read race (`suppress_v_blank` on scanline 241 dot 0), sprite-0 and overflow clear, scroll increments/copies (`copy_horizontal` at dot 257, `copy_vertical` at dots 280–304 of pre-render), odd-frame dot skip on NTSC when rendering is enabled, A12/CHR bus fetch edges for mapper IRQs, and **sprite-0 hit detection** on its exact pixel dot (`x + 1`).
- **Per scanline:** Background and sprite pixel composition executes once at dot 256 from `render_v` (latched at dot 257 after horizontal/vertical scroll copies).

### 3.2 Registers, Loopy Scroll & VRAM Rules
- **Loopy registers:** 15-bit `v` and `t` (`yyy NN YYYYY XXXXX`), 3-bit `fine_x`, and `write_toggle` (`w`).
  - `$2000` writes nametable bits into `t[11:10]`.
  - `$2002` read clears `write_toggle` and bit 7 (VBlank).
  - `$2005`: `w=0` sets `fine_x` and coarse X; `w=1` sets fine Y and coarse Y.
  - `$2006`: `w=0` sets `t[13:8]` (clearing bit 14); `w=1` sets `t[7:0]` and copies `v = t`.
  - Coarse Y wraps at `29` (toggling vertical nametable); setting coarse Y to `30` or `31` wraps at `31` without toggling.
- **`$2007` read buffer:** Reads below `$3F00` return the buffered previous byte and refill `read_buffer` from VRAM; palette reads (`$3F00..=$3FFF`) return palette RAM immediately while refilling `read_buffer` from the mirrored nametable beneath.
- **Palette RAM mirroring (`palette_offset`):** 32 bytes; `$3F10`, `$3F14`, `$3F18`, and `$3F1C` mirror `$3F00`, `$3F04`, `$3F08`, and `$3F0C` (`if (index & 0x13) == 0x10 { index &= 0x0F }`).
- **Per-bit open-bus decay:** `Ppu.open_bus` tracks per-bit decay timers (`open_bus_decay: [u8; 8]`, `OPEN_BUS_DECAY_FRAMES = 36` frames ≈ 600 ms). Reading `$2002` refreshes only bits 7–5 while bits 4–0 continue decaying; palette reads drive bits 5–0 and return bits 7–6 from the open-bus latch; OAM attribute bits 2–4 read back `0`.

### 3.3 Sprite-0 Hit & Rendering
- **Per-dot sprite-0 hit (`detect_sprite0_hit`):** Runs on dots `1..=256` within sprite 0's 8 columns. Enforces both-layers-enabled, `x != 255`, left-8-pixel masking (`PPUMASK` bits 1–2), non-side-effecting CHR peek (`peek_chr`), and shifter-reload validity via `rendering_since` (tracking when rendering was last turned on so tiles not yet loaded into hardware shifters cannot hit).
- **RGB palette & `nes.ppu` raw picture:**
  - RGB output (`256 × 240`, `FRAME_BYTES = 245,760`) uses a 64-entry NTSC `NES_PALETTE` with grayscale (`PPUMASK` bit 0 masking index with `0x30`) and does **not** tint RGB output for emphasis bits.
  - Pre-palette raw picture (`RAW_NES_PPU = "nes.ppu"`, `RAW_BYTES = 122,880`, 2 bytes/pixel little-endian `u16`): bits `0..=5` hold the 6-bit palette color index and bits `6..=8` hold raw `PPUMASK` emphasis bits `5..=7` as written (used by composite/CRT filters; PAL/Dendy swap bits 5–6 inside the filter rather than pre-swapping in the core). Core machine info declares a 512-entry palette matching `NES_PALETTE` repeated across all 8 emphasis values.

---

## 4. APU — 5-Channel Ricoh 2A03 Audio (`src/apu/`)

- **Clocking:** Stepped every CPU cycle (`triangle` and `DMC` timers tick every CPU cycle; `pulse 1`, `pulse 2`, and `noise` timers tick every second CPU cycle). Output is box-filtered over CPU cycles to `SAMPLE_RATE = 44,100` Hz stereo (duplicated mono).
- **Frame sequencer (`$4017`):** Six-entry cycle table (NTSC):
  - **4-step:** `[7457, 14913, 22371, 29828, 29829, 29830]` — quarter frames at steps 0/1/2/4, half frames at steps 1/4, frame IRQ flag asserted across cycles `29828`–`29830` (with inhibit set, flag still rises at `29828`–`29829` while IRQ line stays low).
  - **5-step:** `[7457, 14913, 22371, 29829, 37281, 37282]` — quarter/half frames at 1 and 4, never asserts IRQ.
  - Writes to `$4017` apply IRQ inhibit immediately but delay mode/reset by 3 or 4 CPU cycles depending on CPU/APU cycle parity; 5-step mode clocks quarter+half when the delayed write lands. Soft reset re-issues the previous `$4017` mode through the same delay.
- **Channels (`src/apu/channels.rs`):**
  - **Pulse 1 & 2:** 4 duty cycles (`12.5%`, `25%`, `50%`, inverted `25%`), envelope, length counter, sweep (Pulse 1 uses ones' complement negate, Pulse 2 uses two's complement; muted when period `< 8` or sweep target `> $7FF`).
  - **Triangle:** 32-step sequence (`15..0, 0..15`), linear + length counter gating sequencer while timer continues counting; periods `< 2` silence output to avoid DC clicks.
  - **Noise:** 15-bit LFSR (tap bit 6 in short mode, bit 1 in normal mode), NTSC CPU-cycle period table `[4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068]`.
  - **DMC:** 7-bit delta counter (`0..=127`, step ±2), NTSC CPU-cycle rate table `[428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106, 84, 72, 54]`, reading memory through the live bus via CPU-halting DMA.
- **Non-linear DAC & output filters:**
  - `pulse_out = 95.88 / ((8128 / (p1 + p2)) + 100)`
  - `tnd_out = 159.79 / ((1 / (tri/8227 + noise/12241 + dmc/22638)) + 100)`
  - Followed by one-pole 90 Hz high-pass, 440 Hz high-pass, and 14 kHz low-pass filters (`OUTPUT_GAIN = 80000.0`). Per-channel mutes are supported via Core ABI `MUTES`.

---

## 5. Memory Map, DMA, iNES & Supported Mappers (`src/memory/`)

### 5.1 CPU Address Space & DMA
- `$0000–$1FFF`: 2 KiB internal WRAM (`$0800` mirrored 4×).
- `$2000–$3FFF`: PPU registers (`$2000–$2007` mirrored).
- `$4000–$4017`: APU, OAM DMA (`$4014`), and controller ports (`$4016`/`$4017`). Consecutive-cycle reads of `$4016`/`$4017` are tracked (`held_read`) so repeated reads on adjacent cycles clock the shift register once.
- `$4020–$FFFF`: Cartridge space (8 KiB PRG RAM at `$6000–$7FFF`, PRG ROM windows at `$8000–$FFFF`).
- **OAM & DMC DMA (`bus.rs`):** Both OAM DMA and DMC sample fetches halt the CPU on read cycles and perform real bus reads (`dma_read`), modelling the 2A03 internal bus (`$4000–$401F` registers answering when the halted CPU address falls inside `$4000–$401F`, including NTSC DMC controller-read bit corruption).
- **ROM patches:** Game Genie patch table (`RomPatches`, per-address 256-entry compare-value map) exposed via Core ABI `ROM_PATCHES`.

### 5.2 Cartridge Loader (`src/memory/cartridge.rs`)
- Parses 16-byte iNES 1.0 and NES 2.0 headers (NES 2.0 detected strictly when byte 7 bits `3:2 == 0b10`), plus optional 512-byte trainer.
- Archaic iNES headers with copier garbage in bytes 7–15 (e.g., `DiskDude!`) mask off the corrupted upper mapper nibble.
- PRG ROM must be present in full; short CHR regions are zero-padded; `chr_banks == 0` allocates 8 KiB CHR RAM. PRG RAM is always allocated as 8 KiB.
- Unsupported mapper numbers fail explicitly at load (`RomError`).

### 5.3 Supported Mappers (`src/memory/mappers.rs` — 16 Boards)

| iNES # | Board | Key Features |
|---:|---|---|
| 0 | NROM | Fixed 16 KiB (mirrored) or 32 KiB PRG, fixed 8 KiB CHR |
| 1 | MMC1 | Serial shift register (consecutive-cycle write filter), 4/8 KiB CHR, 3 PRG modes |
| 2 | UxROM | Switchable 16 KiB `$8000`, fixed last 16 KiB `$C000`, CHR RAM |
| 3 | CNROM | Fixed PRG, switchable 8 KiB CHR (bus conflicts not modelled) |
| 4 | MMC3 | Four 8 KiB PRG windows, eight 1 KiB CHR windows, A12 scanline IRQ (`MMC3_A12_MINIMUM_LOW_DOTS = 9`) |
| 7 | AxROM | Switchable 32 KiB PRG, CHR RAM, software single-screen mirroring |
| 9 | MMC2 | Switchable 8 KiB `$8000`, three fixed banks, PPU `$FD`/`$FE` CHR latch switching |
| 11 | Color Dreams | 32 KiB PRG + 8 KiB CHR bank register |
| 64 | RAMBO-1 | Three 8 KiB PRG windows, eight 1 KiB CHR windows, A12 (`RAMBO1_A12_MINIMUM_LOW_DOTS = 30`) or CPU-cycle IRQ |
| 65 | Irem H3001 | Three 8 KiB PRG windows, eight 1 KiB CHR windows, 16-bit CPU-cycle IRQ |
| 66 | GxROM | Single-register 32 KiB PRG + 8 KiB CHR switching |
| 67 | Sunsoft-3 | Switchable 16 KiB `$8000`, four 2 KiB CHR windows, 16-bit CPU-cycle IRQ |
| 68 | Sunsoft-4 | Switchable 16 KiB `$8000`, CHR ROM nametable mapping |
| 69 | Sunsoft FME-7 | Three 8 KiB PRG windows + switchable `$6000` RAM/ROM, eight 1 KiB CHR windows, CPU-cycle IRQ |
| 71 | Camerica | Switchable 16 KiB `$8000`, fixed last bank `$C000`, CHR RAM |
| 79 | NINA-003-006 | `$4100` register selecting 32 KiB PRG + 8 KiB CHR |

---

## 6. Core ABI v1, Save States & Accuracy Boundaries

- **Core ABI capabilities (`src/ffi/v1.rs`):** `RESET`, `MUTES`, `ROM_PATCHES`, `DEBUG`, `DEBUG_STACK`, `RAW_PICTURE` (`nes.ppu`), `DEBUG_REGISTERS`, `DEBUG_DISASSEMBLE`, plus 9 inspectable hardware units (`src/ffi/units.rs`: PPU, APU, 5 channels, board, cartridge).
- **Save states (`MOON`, `src/machine.rs`, `src/state.rs`):** Loads versions `3..=7`. NTSC writes version `6` (omitting trailing `Model`/`DotPhase` bytes for golden compatibility); PAL and Dendy write version `7`. Cross-console state loads are refused with status `-12`.
- **Known accuracy boundaries:**
  - **Scanline-level pixel composition:** Mid-scanline writes to `$2005`/`$2006`/`$2001` (including mid-line grayscale or palette changes) apply to the whole scanline rather than per pixel. Sprite-0 hit itself is evaluated per dot, using `render_v` and `peek_chr` rather than hardware pixel shift registers.
  - **PPU RGB emphasis:** The 8-bit RGB framebuffer omits emphasis tinting (emphasis bits are preserved faithfully in `nes.ppu` raw output).
  - **Sprite overflow bug:** Evaluates cleanly up to 8 sprites per line; the 2C02 diagonal OAM evaluation overflow quirk and `$2003`/`$2004` OAM corruption quirks are not modelled.
  - **Mapper bus conflicts & expansion audio:** Discrete-logic bus conflicts (e.g., CNROM/UxROM) and Famicom expansion audio chips (VRC6/VRC7/FDS/Sunsoft 5B) are not modelled.
  - **PAL OAM refresh / decay:** 2C07 vblank OAM refresh (scanlines 265–310) and OAM DRAM decay are not modelled.
