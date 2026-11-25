# Sapphire: System Architecture, Clocks, Memory Maps, Save States, and Core ABI

System-level hardware and integration reference for `sapphire`: master clocks, NTSC/PAL timing ratios, EE and IOP physical memory maps, MCH/RDRAM initialization, BIOS and direct ELF fast-boot contracts, the `SPH2` save-state format, Core ABI v1 interface, idle skipping, and current verification boundaries.

---

## 1. Master Clocks and Timing Ratios

`src/clock.rs` anchors the entire machine to a single monotonic EE cycle counter so no subsystem drifts:

| Clock Domain | Nominal Rate | Exact Ratio to EE Clock | Source Constant |
| --- | --- | --- | --- |
| **EE Core (`EE_HZ`)** | 294.912 MHz | `1` EE cycle | `clock::EE_HZ = 294_912_000` |
| **EE Bus (`BUSCLK`)** | 147.456 MHz | `2` EE cycles / bus cycle | `clock::BUS = 2` |
| **IOP (`IOP_HZ`)** | 36.864 MHz | `8` EE cycles / IOP cycle | `clock::IOP = 8` |
| **SPU2 Sample Clock** | 48,000 Hz | `768` IOP cycles (`6,144` EE cycles) | `spu2::CYCLES_PER_SAMPLE = 768` |
| **PCRTC VCK (NTSC / PAL / 480p)** | 54.0 MHz | `375 / 2048` VCK per EE cycle | `clock::VCK_54 = (375, 2048)` |
| **PCRTC VCK (1080i)** | 74.25 MHz | `12375 / 49152` VCK per EE cycle | `clock::VCK_74 = (12375, 49152)` |
| **IOP Timer 0 Pixel Clock** | 13.5 MHz | `8192 / 375` EE cycles / tick | `clock::PIXEL = (8192, 375)` |

### NTSC / PAL Field Rates (`src/pcrtc/timing.rs`)

- **NTSC (Interlaced)**: 3,432 VCK/line × 525 lines/frame (262.5 lines/field -> 900,900 VCK/field), yielding an exact field rate of `54,000,000 / 900,900 = 6000 / 1001` (~59.940 Hz). Non-interlaced NTSC uses 263 lines/field.
- **PAL (Interlaced)**: 3,456 VCK/line × 625 lines/frame (312.5 lines/field -> 1,080,000 VCK/field), yielding an exact field rate of `54,000,000 / 1,080,000 = 50 / 1` (50.000 Hz). Non-interlaced PAL uses 314 lines/field.

---

## 2. EE and IOP Memory Maps

### 2.1 EE Physical Memory Map (`src/ee/map.rs`, `src/board.rs`)

| Physical Address Range | Size / Region | Target |
| --- | --- | --- |
| `0x0000_0000..0x0200_0000` | 32 MiB | Main RDRAM (`Map::rdram`, with 64-byte line watch bitmap for EE JIT) |
| `0x0200_0000..0x1000_0000` | Unpopulated | Reads `0`, writes ignored |
| `0x1000_0000..0x1000_2000` | 8 KiB | EE Timers T0–T3 (`+0x800` stride) |
| `0x1000_2000..0x1000_2040` | 64 B | IPU registers (`CMD`, `CTRL`, `BP`, `TOP`) |
| `0x1000_3000..0x1000_3100` | 256 B | GIF registers |
| `0x1000_3800..0x1000_4000` | 2 KiB | VIF0 (`0x1000_3800`) and VIF1 (`0x1000_3C00`) registers |
| `0x1000_4000..0x1000_6000` | 8 KiB | VIF0 FIFO (`0x1000_4000`) and VIF1 FIFO (`0x1000_5000`) |
| `0x1000_6000..0x1000_7000` | 4 KiB | GIF PATH3 FIFO |
| `0x1000_7000..0x1000_7020` | 32 B | IPU Output (`0x1000_7000`) and Input (`0x1000_7010`) FIFOs |
| `0x1000_8000..0x1000_F000` | 28 KiB | EE DMAC channels 0–9 and global registers (`D_CTRL`–`D_STADR`) |
| `0x1000_F000..0x1000_F600` | 1.5 KiB | EE INTC (`0x1000_F000`), SIF (`0x1000_F200`), MCH (`0x1000_F430`/`F440`), `D_ENABLER` (`F520`), `D_ENABLEW` (`F590`) |
| `0x1100_0000..0x1101_0000` | 64 KiB | VU0 micro/data (4 KiB each, mirrored across 16 KiB) and VU1 micro/data (16 KiB each) |
| `0x1200_0000..0x1201_0000` | 64 KiB | GS privileged registers (`GA[6:0]`); `0x1201_0000..0x1400_0000` raises a bus error |
| `0x1C00_0000..0x1C20_0000` | 2 MiB | EE window directly into IOP RAM |
| `0x1F40_2000..0x1F40_2040` | 64 B | CDVD byte registers (accessible from EE and IOP) |
| `0x1FC0_0000..0x2000_0000` | 4 MiB | Boot ROM (read-only) |
| `0x2000_0000..0x4000_0000`, `0x8000_0000..` | Bus Error | Asserts `BUSERR*` (sets `BadPAddr`, `Status.BEM`, raises Bus Error exception) |
| `0x4000_0000..0x8000_0000` | Extend | Unpopulated retail extended RAM window |

The 16 KiB EE Scratchpad (`SPRAM`) is reached solely via TLB entries with `EntryLo0.S = 1` (conventionally mapped at `0x7000_0000`). Narrow (< 4-byte) register writes preserve unmodified plain bits while zeroing write-1-to-clear/action bits (`Board::narrow`).

### 2.2 IOP Physical Memory Map (`src/iop/bus.rs`)

| Physical Address Range | Size / Region | Target |
| --- | --- | --- |
| `0x0000_0000..0x0080_0000` | 8 MiB window | 2 MiB IOP RAM mirrored four times |
| `0x1D00_0000..0x1D00_0100` | 256 B | SIF registers (IOP side) |
| `0x1E00_0000..0x1F00_0000` | Absent | DVD-player ROM window (reads zero, writes ignored) |
| `0x1F40_0000..0x1F50_0000` | 1 MiB | CDVD registers (base `0x1F40_2000`) |
| `0x1F80_0000..0x1F80_0400` | 1 KiB | IOP Scratchpad (KUSEG and KSEG0 only) |
| `0x1F80_1000..0x1F81_0000` | 60 KiB | IOP INTC, DMAC (ch 0–12), Timers (T0–T5), SIO2 (`0x1F80_8200`) |
| `0x1F90_0000..0x1F91_0000` | 64 KiB | SPU2 registers (`0x1F90_0000..0x1F90_07FF`; remainder reads zero) |
| `0x1FC0_0000..0x2000_0000` | 4 MiB | Boot ROM (shared with EE) |
| `0xFFFE_0130` (KSEG2) | 4 B | IOP Cache Control (`BCC`) |

---

## 3. MCH / RDRAM Serial Initialization (`src/mch.rs`)

During cold boot, the BIOS initializes RDRAM over the Memory Controller Hub's serial interface:
- `MCH_RICM` (`0x1000_F430`): serial address `[27:16]`, operation `[9:6]` (`0` = read, `1` = write), device `[4:0]`, start `[31]` (clears immediately on write).
- `MCH_DRD` (`0x1000_F440`): data register for serial reads and writes.
- Models `DEVICES = 2` 128 Mbit RDRAM chips (32 MiB total):
  - Serial read of register `0x21` returns `IDENTITY = 0x1F` for the first two queries, then `0` until a serial write to `0x21` with `MCH_DRD` bit 7 clear resets the enumeration counter.
  - Register `0x23` reads `0x0D0D`; register `0x24` reads `0x0090`; register `0x40` echoes `RICM & 0x1F`; all other reads return `0`.

---

## 4. BIOS ROM and Direct ELF Fast-Boot Contract

### 4.1 Retail BIOS Loading (`src/bios.rs`, `src/machine.rs`, `src/v1.rs`)

- Sapphire requires a 4 MiB (`BIOS_BYTES = 4 << 20`) retail PlayStation 2 BIOS image corresponding to the console's market (`BIOS_JAPAN = 16`, `BIOS_AMERICAS = 17`, `BIOS_EUROPE = 18`). An open BIOS replacement is not yet implemented; without a BIOS, both processors stay held and the machine outputs black/silent fields.
- `src/bios.rs` embeds Redump MD5 checksums for 27 Japanese, 15 Americas, and 16 European retail BIOS revisions (canonical filenames `ps2-0230j-20080220.bin`, `ps2-0230a-20080220.bin`, `ps2-0230e-20080220.bin`) so frontend firmware discovery matches any retail dump by content.
- `Machine::insert_bios` maps the identical 4 MiB image at `0x1FC0_0000` for both the EE and IOP and resets both CPUs to `0xBFC0_0000`. Starting with an empty tray (`starts_empty("ps2") == true`) boots into the BIOS OSDSYS menu.

### 4.2 Direct ELF Fast-Boot (`src/machine.rs`, `src/media.rs`, `src/iop/tty.rs`)

When an EE ELF executable (32-bit little-endian MIPS `ET_EXEC`) is loaded directly instead of a disc image:
1. The BIOS boots normally until `Machine::at_entry()` becomes true: IOP has set `SIF.SMFLG` bit 18 (`IOP_BOOTED = 0x4_0000`), the EE is executing in user mode (`!ee.kernel()`), and the EE is outside a branch delay slot.
2. `Machine::enter_program()` flushes and invalidates all EE caches (`flush_caches`), copies every `PT_LOAD` segment into RDRAM (`address & 0x1FFF_FFFF`), zero-fills BSS bytes (`file..memory`), writes `argv[0] = "host:program.elf\0"` into `ARGS = 0x01F0_0000`, sets `$v1 = 7` (`ExecPS2`), `$a0 = entry`, `$a1 = gp` (from `PT_MIPS_REGINFO`), `$a2 = 1`, `$a3 = 0x01F0_0000`, and raises `Exception::Syscall` so the retail EE kernel initializes a clean main thread without patching any BIOS code.
3. `iop.tty.device` is enabled: `src/iop/tty.rs` dynamically locates the `ioman` module export table (via `EXPORT_MAGIC = 0x41C0_0000` stores tracked in `IopBus::export_tables`) and intercepts export 7 (`write(fd = 1, ...)`) into the 256 KiB read-only `TTY` memory space.

---

## 5. `SPH2` Save-State Format (`src/state.rs`)

`SPH2` is Sapphire's binary state format (`STATE_MAGIC = "SPH2"`, current `STATE_VERSION = 5`, backward-compatible load down to `OLDEST_STATE_VERSION = 4`). Layout invariants are pinned byte-for-byte against `tests/sph2/version-5.layout`.

- **Header**: `Magic` (u32), `Version` (i32), `Frames` (i64), `Market` (u8: `0` Japan / `1` Americas / `2` Europe), `Bios` (u32 file ID, `0` if none), `Image` (u64 FNV-1a media/tray identity), `Stop` (u8), `Entered` (optional u64 frame of ELF fast-boot entry).
- **Variable Picture Block**: `Picture` geometry, aspect ratio, field parity, and raw RGBA raster bytes (validated before any mutation so a rejected or truncated load modifies zero machine state).
- **Fixed Hardware Blocks (`System::write_state` order)**: `Ee` (registers, COP0/1, timing, TLB, caches, `InterruptAfter` added in v5), `Rdram` (32 MiB), `Scratchpad` (16 KiB), `Vpu` (VU0/1 + micro pipes + PATH1), `Vif0`, `Vif1`, `Gif`, `Gs` (registers, 4 MiB local memory, CLUT, transfer, vertex queue), `Pcrtc`, `Dmac`, `Intc`, `Timers` (T0–T3), `Iop` (R3000A, 2 MiB RAM, I-cache, scratchpad, INTC, DMAC, T0–T5, TTY), `Sif`, `Cdvd` (state, RTC, NVM session metadata), `Sio2`, `Spu2` (both cores + 2 MiB RAM), `Ipu`, `Mch`, `Scheduler` (`now`, `next`, `EdgesTaken`).
- **Exclusions & Queue Limits**: BIOS ROM, disc data, memory card images, and NVM EEPROM bytes are never embedded in states. All dynamic queues serialize into fixed-size slots; `System::fits()` rejects saving with `STATUS_QUEUE_PAST_STATE` (`-15`) if any queue overflows its slot count or ahead-of-board transactions have not settled.

---

## 6. Core ABI v1 Interface (`src/v1.rs`)

Sapphire exports Core ABI v1 (`emusen_native::core_exports!`) as system `"ps2"` (`id = "sapphire"`, deterministic, `extensions = [".elf", ".iso", ".chd", ".cue", ".bin"]`).

- **Capabilities**: `AXES | MEDIA | BORROW | DEBUG | DEBUG_STACK | DEBUG_REGISTERS | DEBUG_DISASSEMBLE | DEBUG_READS`.
- **Settings Schema**:
  - `region`: `"auto"` (default; disc serial/VMODE -> BIOS market -> Americas), `"ps2.jp"`, `"ps2.us"`, `"ps2.eu"`.
  - `pad1`, `pad2`: `"ps2.dualshock2"` (default), `"ps2.digital"`, or `"none"` (`pad2` only).
  - `ee_tier`: `"recompiler"` (default) or `"interpreter"` (`EMUSEN_SAPPHIRE_TIER`).
  - `vu1`, `vu0`: `"recompiler"` (default) or `"interpreter"` (`SAPPHIRE_VU1`, `SAPPHIRE_VU0`).
  - `idle_skip`: `"on"` (default) or `"off"` (`EMUSEN_SAPPHIRE_IDLE`).
  - `gs_workers`: `"auto"` (default, `clamp(threads/4, 1, 4)`), `"1"`–`"4"` (`SAPPHIRE_GS_WORKERS`).
- **Ports, Drives, and Batteries**: 2 controller ports (`DualShock 2` with 16 buttons + 4 axes; `Digital Controller` with 14 buttons); 1 removable DVD tray (`DRIVE = 0`); 2 removable memory card slots (`CARD_FILES = [0, 1]`, medium `ps2.card`, suffixes `.ps2`/`.mcd`); 3 per-market console EEPROM files (`NVM_FILES = [2, 3, 4]`, medium `ps2.nvm`, suffixes `.jp.nvm`/`.us.nvm`/`.eu.nvm`).
- **Memory Spaces (11 total)**: `1` `RAM` (32 MiB, cheats/watchpoints), `2` `SCRATCH` (16 KiB, watchpoints), `3` `GSMEM` (4 MiB), `4` `VU0MICRO` (4 KiB), `5` `VU0DATA` (4 KiB), `6` `VU1MICRO` (16 KiB), `7` `VU1DATA` (16 KiB), `8` `IOPRAM` (2 MiB), `9` `SPU2RAM` (2 MiB), `10` `BIOS` (4 MiB, read-only), `11` `TTY` (256 KiB, read-only).
- **Debugger Processors (4 total)**: `0` `R5900` (32-bit PC), `1` `IOP` (32-bit PC), `2` `VU0` (12-bit PC), `3` `VU1` (14-bit PC).
- **Custom Refusal Status Codes**: `-9` not PS2 image, `-10` ELF not EE MIPS, `-11` ELF truncated, `-12` PlayStation 1 disc, `-13` ISO missing `BOOT2`, `-14` BIOS mismatch on state load, `-15` queue past state slots, `-16` non-disc inserted in drive, `-17` unreadable/short disc image.

---

## 7. Idle Skipping (`src/idle.rs`, `src/ee/idle.rs`)

Idle skipping is enabled by default (`idle_skip = "on"`) and performs runtime proof-based loop skipping with no hardcoded game addresses:

- **Interpreter Proof Loop**: Tracks up to `HEADS = 1024` backward-branch loop heads (`SPAN = 0x400`, `MAX_TURN = 256` instructions). On the `ARRIVALS = 3`-rd arrival (with exponential backoff up to `2^12` on refusal), it snapshots `ee::idle::Mark` and steps a full turn while classifying every memory/register access across 25 refusal kinds (`Refusal`).
- **Admissible Loops**: A turn qualifies when it performs no side-effecting stores, no cache refills, no exceptions, no COP0/COP2/timer/VIF/GIF/IPU/SIF reads (pure `INTC`/`DMAC` polling reads are allowed via `Board::pure_register`), and leaves every register unchanged or advancing by a constant linear induction step unused by branches or addresses.
- **Exact Fast-Forward**: Qualified loops advance EE cycles, `Count`/`Compare`, `Random`, issue timing, and induction registers across multiple turns at once while advancing the board and IOP up to the earliest event, DMAC write into a watched address (`dmac::Watch`), interrupt line change, or field start.
- **Compiled-Code Waits**: When the EE recompiler runs (`board.round > 0`), compiled wait loops skip directly to `Board::wait_deadline()` (the next scheduled event while graphics/VU units are idle, else `deadline`, bounded by unmasked `Count == Compare`), reporting `Context::link = WAITED`.

---

## 8. Verified Accuracy Boundaries (G1–G11)

The core's current factual verification status (as declared in `v1::GATES`):

| Gate | Status | Current Verified Behavior |
| --- | --- | --- |
| **G1 (CPU / IOP Suites)** | Met | `ps2autotests` CPU programs match reference outputs except one `RSQRT` edge-case line; `blackmoon-r3000` passes the R3000 test suite. |
| **G2 (BIOS + ps2autotests)** | Met | 75 of 78 `ps2autotests` suites match expected output in full on retail BIOS; the remaining 3 differ on known isolated `RSQRT`, `BC1` delay-slot, and VU1 EFU rounding edge cases. |
| **G3 (GS Pixel Verification)** | Open | GS drawing and texturing test programs match PCSX2 reference captures or document known edge differences; cross-verification against a second independent GS reference remains open. |
| **G4 (Frontend Integration)** | Met | Conformance kit, ABI fit audit, and CRT profile pass. |
| **G5 (Game Golden Anchors)** | Open | Retail boot and gameplay anchors verified on multiple titles (including *Jak and Daxter*, *Kingdom Hearts*, and *Dark Cloud 2* through menu/memory-card prompt); full anchor coverage across all benchmark titles remains in progress. |
| **G6 (Full-Speed Performance)** | Open | Frame rates on retail 3D workloads remain below full 59.94 Hz console speed on desktop CPUs; performance optimization (EE JIT, VU JIT, Vulkan compute GS, ahead transactions) is ongoing. |
| **G7 (Debugger)** | Met | All 4-processor debugger capabilities (breakpoints, stepping, registers, stack, disassembly, memory reads/watchpoints) verified by automated tests. |
| **G8 (Clean-Room Audit)** | Open | Final independent clean-room verification pass pending. |
| **G9 (Open BIOS)** | Open | Retail PS2 BIOS image required; open-source PS2 BIOS replacement not yet written. |
| **G10 (Empty-Tray Boot)** | Met | Console powers on with an empty closed tray to the BIOS OSDSYS menu and supports runtime tray open/disc swap via Core ABI `MEDIA`. |
| **G11 (Memory Card Manager)** | Met | PS2 memory card format (`emusen-ps2-card`) integrated with the frontend memory card manager; retail BIOS MagicGate authentication (`0xF0`/`0xF3`/`0xF7`) is unimplemented, so games running on retail BIOS do not yet authenticate memory cards through MagicGate. |
