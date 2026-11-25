# Nephrite — Sega CD / Mega-CD Attachment and Open BIOS

Hardware, attachment-boundary, open BIOS, and backup RAM reference for the Sega CD / Mega-CD subsystem (`src/mcd/`) of `nephrite` (`EmuSen/Cores/Sega/Nephrite - Genesis/`). Base Genesis hardware is documented in [Nephrite_Genesis.md](Nephrite_Genesis.md).

---

## Attachment Boundary and Cargo Features (`src/interface.rs`, `src/boundary.rs`)

Attachments (Sega CD on the expansion port, 32X in the cartridge slot) are isolated from `src/genesis/` behind `src/interface.rs`:

- **Compile-time feature isolation (`Cargo.toml`)**: `default = ["mcd", "s32x"]`, with `mcd = ["dep:emusen-disc"]` and `s32x = []`. Unit tests (`src/boundary.rs`) enforce both source-level isolation (production code in `src/mcd/` and `src/s32x/` may import only `crate::interface` and its own module tree; `src/genesis/` and `src/interface.rs` never name attachments or assembly modules) and feature-subset compilation (`--no-default-features`, `--features mcd`, `--features s32x`).
- **Generic zero-cost dispatch**: `Genesis<A = Bare>` is parameterized over `A: Attachment`. When bare (`Bare`), every method is `#[inline(always)]` returning `Answer::Pass` / `false` / `None`, adding zero runtime overhead to standalone Genesis games.
- **Core traits (`src/interface.rs`)**:
  - `Attachment`: `const EXPANSION: bool` (sets Genesis version register `$A10001` bit 5 low when true), `read8`, `read16`, `transfer16` (VDP DMA bus reads), `write` (with lane mask `$FF00`/`$00FF`/`$FFFF`), `run_to(Master)`, and `sound(Master) -> Option<&mut Steps>` (mixed *after* the Genesis output circuit).
  - `AttachmentState`: Atomic two-phase `read_state` → `check` (verifying disc fingerprint `Refusal::WrongMedia`) → `apply_state`, plus `write_state`.
  - `AttachmentView`: Exposes memory spaces from fixed non-overlapping ID ranges (`GENESIS_SPACES = 0..16`, `MCD_SPACES = 16..32`, `S32X_SPACES = 32..48`) and fixed processor IDs (`GENESIS_PROCESSORS = 0..2`, `MCD_PROCESSORS = 2..3` with `S68K = 2`, `S32X_PROCESSORS = 3..5`).
- **Mode 1 vs Mode 2**: Disc boot runs in Mode 2 (Sega CD boot ROM at `$000000`–`$01FFFF`, PRG-RAM window at `$020000`–`$03FFFF`, Word RAM at `$200000`–`$23FFFF`). Setting `mcd_beneath = mcd.beneath` mounts the Sega CD beneath a cartridge in Mode 1 (boot ROM and memories shifted to `$400000` while `$000000`–`$3FFFFF` stays with the cartridge; save state uses system byte `3`).

---

## Sega CD Hardware Subsystems (`src/mcd/`)

### Sub-CPU, Clocks, and Gate Array (`src/mcd/mod.rs`)

- **Sub-68000**: A second `beryl-m68k` CPU clocked at `SUB_HZ = 12,500,000 Hz` (`50 MHz ÷ 4`), synchronized against the Genesis master clock via exact rational ratio.
- **Memories (Space IDs `17–21`)**:
  - `PRGRAM` (ID `17`, `512 KiB`): Main CPU accesses four 128 KiB banks via `$020000`–`$03FFFF` (`BK1–BK0` in `$A12003`) while `SBRQ` holds the sub-CPU bus; sub-CPU maps all 512 KiB at `$000000`–`$07FFFF` with write-protection granularity `WP7–WP0` (`$A12002`).
  - `WORDRAM` (ID `18`, `256 KiB`): Shared via `word.rs` in 2M mode (single 256 KiB bank owned by main or sub via `DMNA`/`RET`) or 1M mode (two 128 KiB banks swapped by `RET`/`DMNA`, with main CPU cell-reordered view `cell_word` at `$220000`–`$23FFFF` and sub-CPU 4bpp dot-image priority writes `PM1–PM0` at `$080000`–`$0BFFFF`). Sub-CPU accesses to main-owned Word RAM park the sub-CPU (`parked`, `owed`) until ownership returns; main accesses see open bus.
  - `PCMRAM` (ID `19`, `64 KiB`): RF5C164 wave memory.
  - `BRAM` (ID `20`, `8 KiB`): Internal backup RAM mapped on odd bytes of `$FE0000`–`$FE3FFF` (mirrored to `$FEFFFF`).
  - `CARTRAM` (ID `21`, optional `128 KiB` Sega Backup RAM Cartridge via `ramcart = mcd.ramcart`): Size code `log2(size / 8 KiB)` at `$400001`, RAM on odd bytes of `$600000`–`$6FFFFF`, write enable bit 0 at `$700001`.
- **Gate array (`Gate`)**: Hosts `SRES`/`SBRQ` sub-CPU reset/busreq, forced-reset 4-write sequence, `RES0` 100 ms peripheral reset pulse (`PERIPHERAL_RESET = SUB_HZ / 10`), `HIVECT` (main H-INT override at `$A12006`), communication flags (`CFM`/`CFS`), eight 16-bit command words (`$A12010`/`$FF8010`), eight 16-bit status words (`$A12020`/`$FF8020`), 30.72 µs stopwatch and level-3 timer (`TICK = 384` sub clocks), 4bpp colour-expansion font unit (`font_color`, `font_bits`), LEDs, and sub-CPU interrupt mask/pending (`IEN1–IEN6`).

### Graphics ASIC (`src/mcd/asic.rs`)

Rotation and scaling engine (`$FF8058`–`$FF8066`, active only in 2M Word RAM mode):

- Traces each line of an image buffer (`hdots` × `vdots`, cells stored vertically in columns like VDP sprites) using 4-byte trace vectors from `$FF8066` across a `256 × 256` or `4096 × 4096` stamp map of `16 × 16` or `32 × 32` stamps (with horizontal flip, 90° rotation, optional map repeat `RPT`, and Word RAM priority mode `PM1–PM0`).
- **Timing**: Each line takes `13 + 2 * (offset & 3) + 9 * ((hdots + offset - 1) >> 2)` sub clocks plus `3`-clock refresh stalls every `192` clocks and `3` stolen clocks per concurrent sub-CPU or CDC DMA Word RAM access. Completion raises sub-CPU level 1 interrupt.

### CDC — Sanyo LC8951 (`src/mcd/cdc.rs`)

- 16-register interface via `$FF8004`–`$FF8007` (5-bit auto-incrementing address register `ar`, holding at 0) and a `16 KiB` ring buffer (`BUFFER = 0x4000`).
- Decodes incoming CD-ROM data sectors into buffer memory, sets `HEAD0–HEAD3` / `STAT0–STAT3`, and raises `DECI` (held for `DECI_LENGTH = 67,000` sub clocks unless `STAT3` is read first).
- Transfers buffered data (`DBC` byte count, `DAC` address, `DOUTEN`, `DTTRG`) to `MAIN` (`2`), `SUB` (`3`) host-data registers (`EDT`/`DSR`), or by DMA (`DMA_WORD = 16` sub clocks/word) to `PCM` (`4`), `PRG` (`5`), or `WORD` (`7`), raising `DTEI` on completion. Falling edge on the masked `INT` pin (`DTEIEN`/`DECIEN`) requests sub-CPU level 5.

### CDD Drive Microcontroller and CD-DA (`src/mcd/cdd.rs`, `src/mcd/cdda.rs`)

- **CDD (`$FF8036`–`$FF804B`)**: Exchanges 10 status nibbles and 10 command nibbles (with 4-bit inverted sum checksum) every `1/75 s` (`SECTOR = [166_667, 166_667, 166_666]` sub clocks) while `HOCK` is set, raising sub-CPU level 4. Models drive state machine (`Spinning`, `SPIN_UP = 75`; `ReadingToc`, `TOC_READ = 20`; `Seeking`, `SEEK_BUSY = 12`, `LAND_SHORT = 14`; `Approach`, `SETTLE = 3`; `Paused`, 8-sector `LOOP = 7`; `Scanning`; `Jumping` using radial track formula `sqrt(18300^2 + 3385 * s)`; `TrayOpen`). Supports cue sheets, CHDs, raw 2352-byte tracks, and 2048-byte `SEGADISCSYSTEM` ISOs via `emusen-disc`, and hints sequential host reads via `MEDIA_SEEK`.
- **CD-DA (`cdda.rs`)**: Feeds `588` stereo 16-bit samples per audio sector (`44.1 kHz`) through the LC7883 bilinear de-emphasis filter (`50 µs / 15 µs` at 44.1/32/48 kHz per `DEF1–DEF0`), per-sample stepping fader (`FULL = 0x400`), `GAIN = (158, 1024)`, and a 5-tap `UNHOLD` droop-compensation filter before summing with PCM into the attachment `Steps` stream.

### PCM Sound Source — Ricoh RF5C164 (`src/mcd/pcm.rs`)

- 8 channels clocked at `12.5 MHz ÷ 384` (`32,552.08 Hz`), reading 8-bit sign-magnitude samples from 64 KiB wave RAM (mapped to the sub-CPU through 16 × 4 KiB banks `wave_bank`) with `16.11` fixed-point phase increments (`FD`), `$FF` loop markers jumping to `LS`, per-channel `ENV` and stereo `PAN` nibbles, and 16-bit saturating summation.
- Output passes through a 1-pole `1,600 Hz` low-pass filter (`LOW_PASS = 1600.0`, fitted to hardware recordings) and mixes with CD-DA at `PCM_GAIN = (267, 1024)`.

---

## Open Sega CD BIOS Contract (`build.rs`, `src/mcd/bios/`, `src/mcd/frontend.rs`)

`nephrite` ships an original 128 KiB open Sega CD BIOS (`OPEN_BIOS`, built at compile time by `build.rs` from `src/mcd/bios/{main.s,sub.s,bram.s}` using `beryl-m68k`'s `asm` feature and `beryl-font` packed 4bpp tiles) used automatically whenever the player does not supply a console BIOS (`bios_CD_U.bin`, `bios_CD_E.bin`, or `bios_CD_J.bin`, exact `131,072` bytes).

- **ROM layout (`build.rs`)**: Main-CPU boot program from `$000000`, `beryl-font` 64 tiles at `FONT_AT` (`$00E000`), and the 24 KiB (`$6000`) sub-CPU BIOS image at `SUB_AT` (`$010000`, code ending by `$5400`, work area `$5400`–`$5BFF`, stack `$5C00`–`$5E7F`, system tables/jump entries `$5E80`–`$5FFF`).
- **Main-CPU contract (`main.s`)**: Initializes Genesis VDP/PSG/Z80/TMSS, loads the sub-CPU image into PRG-RAM `$0000`–`$5FFF`, releases sub-CPU reset, displays Nephrite's status screen while booting, installs the standard main work-RAM jump table (`$FFFD00`, `_MLEVEL6` at `$FFFD06`, `_MLEVEL4` at `$FFFD0C`) and VDP register shadow (`$FFFDB4`), loads the disc's initial program (IP) to `$FF0000`, and transfers control.
- **Sub-CPU contract (`sub.s`)**: Exposes the documented jump table (`_CDSTAT` `$5E80`, `_BOOTSTAT` `$5EA0`, `_USERMODE` `$5EA6`, `_SETJMPTBL` `$5F0A`, `_WAITVSYNC` `$5F10`, `_BURAM` `$5F16`, `_CDBOOT` `$5F1C`, `_CDBIOS` `$5F22`, user call slots `_USERCALL0`–`3`, and exception/level vectors `$5F40`–`$5FA0`), drives CDD/CDC sector reads and TOC parsing, loads the sub-CPU system program (`USER_HEADER` at `$6000`), and services `_CDBIOS` and `_BURAM` calls.
- **Documented limitations (`BIOS_COST`)**: Skips regional security lockouts (boots discs of any region), provides its own no-disc status screen without a CD player or memory manager UI, does not yet route games to a backup RAM cartridge through `_BURAM`, and may differ for titles relying on undocumented BIOS addresses, ROM code reads, or exact loop timing.

---

## Backup RAM Format and `_BURAM` Calls (`src/mcd/bios/bram.s`, `src/mcd/openbram.rs`)

Internal 8 KiB backup RAM (`.brm`, medium `mcd.backup`) and Backup RAM Cartridges (`.crm`, medium `mcd.ramcart`, sizes `8 KiB` to `512 KiB`) share one volume format (stored on odd bus bytes, packed contiguously in battery files):

### Volume Layout and Footer

- **64-byte blocks**: Block `0` is reserved, file data allocates upward from block `1`, directory blocks grow *downward* from the footer (`S - 0x80 - 0x40 * j`), and the final 64 bytes (`S - 0x40`) hold the footer:
  - `-0x40`: `11 × 0x5F`, `00 00 00 00`, `0x40`
  - `-0x30`: Free 64-byte block count (`u16` BE), repeated 4 times (`free = S/64 - 2 - sum(file_blocks) - (n + 2) / 2`; freshly formatted = `S/64 - 3`, i.e., `125` on 8 KiB internal RAM, `2045` on 128 KiB cartridge)
  - `-0x28`: File count (`u16` BE), repeated 4 times
  - `-0x20`: `"SEGA_CD_ROM\0"` (12 bytes, checked strictly)
  - `-0x14`: `01 00 00 00`
  - `-0x10`: `"RAM_CARTRIDGE___"` (formatted if at least one of the first three 32-bit words `"RAM_"`, `"CART"`, `"RIDG"` matches)
- **Directory entries**: Each 64-byte directory block is a coded block carrying 32 decoded bytes (`U`): `U[16..32]` holds entry `2j + 1` and `U[0..16]` holds entry `2j + 2`. Each 16-byte entry stores an 11-byte ASCII name (no terminator), 1-byte flag (`$00` normal, `$FF` protected), 16-bit BE start block, and 16-bit BE block count.

### Protected Block Encoding (Directory & Protected Files)

Normal file data blocks store 64 raw bytes; every directory block and every protected-file data block encodes 32 data bytes `U[0..32]` into 64 stored bytes:

1. **CRC-16/XMODEM framing**: `C = crc16_xmodem(U)` (`poly 0x1021`, init `0`), forming a 36-byte message `M = C_be || U[0..32] || (C ^ 0xFFFF)_be` (`288` bits = `48` 6-bit symbols `s[0..47]`).
2. **Reed–Solomon over GF(64)** (`x^6 + x + 1`, `α = 0b000010`): The 48 symbols form 8 interleaved codewords (`K = 0..7`) of 6 data symbols + 2 parity symbols `p0, p1` placed in rows 6 and 7 (`s[48..63]`), with roots `1` and `α` (`S0 = Σ d_k`, `S1 = Σ d_k α^(7-k)`, `p0 = (S0 + S1) / (α + 1)`, `p1 = S0 + p0`). Each symbol sits in bits `7–2` of stored byte `i`.
3. **Column check bits (bits `1–0`)**: Viewing the 64 stored bytes as an 8×8 grid, each column `c` computes a 16-bit shift register `R` over rows `0..7` (shifting left by 2 bits, feeding back top 2-bit cell `o` via `(o << 8) ^ (o << 6) ^ (o << 4) ^ o`, and XORing `T[b] = 4^(b+2) - 2` for each set bit `b` of the 6-bit symbol), storing `(R >> (14 - 2*r)) & 3` in bits `1–0` of byte `(r, c)`.

### `_BURAM` Function Table (`$5F16`, `src/mcd/bios/bram.s`)

Called on the sub-CPU with function number in `D0.W`; returns with Carry clear on success:

| `D0.W` | Call | Summary |
|---|---|---|
| `0` | `BRMINIT` | Initializes scratch area and checks internal BRAM status/format |
| `1` | `BRMSTAT` | Returns free blocks and file count from the volume footer |
| `2` | `BRMSERCH` | Searches directory for an 11-byte filename (`A0`), returning entry metadata |
| `3` | `BRMREAD` | Reads normal or protected blocks of a named file into caller buffer (`A1`) |
| `4` | `BRMWRITE` | Allocates and writes a normal or protected file, updating directory and footer |
| `5` | `BRMDEL` | Deletes a named file, compacting directory/data and updating footer counts |
| `6` | `BRMFORMAT` | Writes the 64-byte Sega footer to initialize the volume |
| `7` | `BRMDIR` | Lists directory entries matching a wildcard pattern into caller buffer |
| `8` | `BRMVERIFY` | Verifies written data against the stored normal/protected file blocks |
