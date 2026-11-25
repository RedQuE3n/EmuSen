# Moon (NES) — PAL & Dendy Hardware (`EmuSen/Cores/Nintendo/MoonRT - NES`)

Live crate: `EmuSen/Cores/Nintendo/MoonRT - NES` (`moonrt`). Single owner of all console model differences: `src/model.rs`, read at load by `MemoryBus::set_model`, `Ppu::set_model`, and `Apu::set_model`, and exposed through Core ABI v1 (`src/ffi/v1.rs`).

---

## 1. NTSC vs PAL vs Dendy Hardware Matrix (`src/model.rs`)

| Parameter | NTSC NES (`2A03` / `2C02`) | PAL NES (`2A07` / `2C07`) | Dendy (`6527P` / `6538`) |
|---|---|---|---|
| Master clock | `21,477,272` Hz | `53,203,425 / 2` Hz (`26,601,712.5` Hz, `6 × 4.43361875` MHz) | `53,203,425 / 2` Hz (`26,601,712.5` Hz) |
| Master clocks / CPU cycle | `12` | `16` | `15` |
| CPU clock | `1,789,773.0` Hz | `1,662,607.03` Hz | `1,773,447.5` Hz |
| Master clocks / PPU dot | `4` | `5` | `5` |
| PPU dots / CPU cycle | `3` | `3.2` (`16 / 5`: four 3-dot cycles + one 4-dot cycle) | `3` |
| Scanlines / frame | `262` (`341` dots/line) | `312` (`341` dots/line) | `312` (`341` dots/line) |
| Pre-render scanline | `261` | `311` | `311` |
| VBlank & NMI line (dot 1) | `241` (`20` vblank lines) | `241` (`70` vblank lines) | `291` (`51` post-render + `20` vblank lines) |
| Odd-frame skipped dot | Yes (when rendering enabled) | No | No |
| CPU cycles / frame | `29,780⅔` | `33,247.5` | `35,464` |
| Frame rate (`machine_info`) | `21,477,272 / 357,368` (`60.0985` Hz) | `53,203,425 / 1,063,920` (`50.0070` Hz) | `53,203,425 / 1,063,920` (`50.0070` Hz) |
| `region` reported | `"ntsc"` | `"pal"` | `"pal"` |
| Black border | None | Top 1 px, left 2 px, right 2 px (`$0E`, unemphasised) | Same as PAL |
| `PPUMASK` emphasis bits 5 & 6 | Bit 5 = Red, Bit 6 = Green | Bit 5 = Green, Bit 6 = Red | Bit 5 = Green, Bit 6 = Red |
| APU Noise / DMC / Frame Counter | 2A03 (NTSC) tables | 2A07 (PAL) tables | 2A03 (NTSC) tables |
| DMA halt restriction | Halts on any read cycle | Halts only on instruction cycle 0 (opcode fetch / IRQ entry / jam) | Same as NTSC |
| Save-state version written | `6` | `7` (appends `Model` + `dot_phase`) | `7` (appends `Model` + `dot_phase`) |

---

## 2. Region Detection & Setting (`src/memory/cartridge.rs`, `src/ffi/v1.rs`)

- **Cartridge header (`Cartridge::header_model`):** Reads **NES 2.0** byte 12 bits `1:0`:
  - `0` → NTSC NES
  - `1` → PAL NES
  - `2` → Multi-region (run as NTSC NES)
  - `3` → Dendy
  - **iNES 1.0 headers:** Bytes 9 and 10 TV-system flags are ignored because they are unreliable in circulating dumps; iNES 1.0 images default to NTSC unless overridden via settings.
- **Create-scope setting (`region`):** Category *Console*, effect `none`:
  - `auto` (default): Use NES 2.0 header (NTSC for iNES 1.0).
  - `nes.ntsc`: Force NTSC NES.
  - `nes.pal`: Force PAL NES.
  - `nes.dendy`: Force Dendy.

---

## 3. PPU Differences (`src/ppu/`, `src/memory/bus.rs`)

- **PAL 3.2 dots per CPU cycle (`dot_phase`):** On NTSC and Dendy, each CPU cycle executes 2 PPU dots before the bus access and 1 dot after. On PAL (`long_cycles == true`), `MemoryBus` counts `dot_phase` modulo 5; every 5th cycle inserts a 3rd PPU dot before the bus access (`long_cycle_dot`), yielding 16 dots per 5 CPU cycles (`3, 3, 3, 3, 4`). `dot_phase` starts at `0` on load and is preserved across soft reset and save states (version 7).
- **2C07 / 6538 black border (`Ppu::draw_border`):** After each visible scanline is composed (leaving sprite-0 hit and flags identical to NTSC), line 0 and columns `0..=1` and `254..=255` are overwritten with black (`$0E` with zero emphasis bits in `nes.ppu`, black in RGB), regardless of rendering enable, grayscale, or emphasis bits.
- **Emphasis bits in `nes.ppu`:** `MoonRT` writes `PPUMASK` bits `5..=7` directly into `nes.ppu` bits `6..=8` without pre-swapping; downstream PAL CRT/composite filters perform the 2C07/6538 red/green swap based on machine info's `"pal"` region.
- **Unmodelled PAL PPU details:**
  - 2C07 vblank OAM refresh (scanlines 265–310) and OAM DRAM decay are not modelled; late-vblank OAM writes behave cleanly.
  - Pixel aspect ratio remains 4:3 in machine info (display aspect adjustment belongs to the frontend/filter layer).
  - Mid-scanline grayscale/scroll effects (e.g., `nmi_sync` PAL demo) are bounded by the scanline-level compositor.

---

## 4. APU & DMA Differences (`src/apu/`, `src/cpu/`)

- **2A07 PAL tables (`src/apu/channels.rs`, `src/apu/mod.rs`):**
  - **Noise periods (CPU cycles):** `[4, 8, 14, 30, 60, 88, 118, 148, 188, 236, 354, 472, 708, 944, 1890, 3778]`.
  - **DMC rates (CPU cycles):** `[398, 354, 316, 298, 276, 236, 210, 198, 176, 148, 132, 118, 98, 78, 66, 50]`.
  - **Frame counter (CPU cycles):**
    - 4-step: `[8313, 16627, 24939, 33252, 33253, 33254]` (flag rises `33,255` cycles after `$4017 = $00`, accounting for the 3-cycle write delay).
    - 5-step: `[8313, 16627, 24939, 33253, 41565, 41566]`.
  - **Dendy APU:** Uses the 2A03 NTSC noise, DMC, and frame-counter tables at the Dendy CPU clock (`1,773,447.5` Hz).
- **Audio resampling:** The mixer computes `cycles_per_sample = cpu_clock_hz / 44100.0` (`~37.70` cycles/sample on PAL, producing `~881.88` stereo sample pairs per 50.007 Hz frame = `44,100` Hz continuous output).
- **2A07 first-cycle DMA gating (`dma_on_first_cycle`):** On the PAL 2A07, `run_dma` halts the CPU only when the current instruction has spent zero cycles (opcode fetch, interrupt entry first read, or jammed read). This prevents DMC fetches from landing on `$4016`/`$4017` controller reads and eliminates the NTSC DMC controller-bit corruption bug. Dendy follows the 2A03 rule (halts on any read cycle).

---

## 5. Save-State Compatibility (`src/machine.rs`)

- NTSC states stay at **version 6** (no trailing `Model` or `dot_phase` fields).
- PAL and Dendy states write **version 7**, appending NES 2.0 model code (`1` = PAL, `3` = Dendy) and `Bus.dot_phase`.
- Loading a state saved on a different model (`NTSC` vs `PAL` vs `Dendy`) is rejected with status `-12`, leaving the running machine untouched.
