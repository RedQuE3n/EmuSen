# Mars — VR4300 CPU, Memory Bus, and Interfaces

Live Rust core crate: `EmuSen/Cores/Nintendo/MarsRT - N64` (crate `marsrt`).

---

## 1. System Clocks and Execution Model

The Nintendo 64 main processor is a 64-bit MIPS III NEC VR4300 clocked at **93,750,000 Hz**. All device scheduling in `marsrt` runs against a single monotonic cycle counter (`MemoryBus::cycles` in `src/memory/bus.rs`):

| Clock / Parameter | Value | Source |
| --- | --- | --- |
| VR4300 processor clock | `93,750,000` Hz | `src/vi/mod.rs`, `src/memory/ai.rs` |
| COP0 `Count` rate | `46,875,000` Hz (`cycles >> 1`) | `src/memory/bus_access.rs` |
| NTSC VI / AI video clock | `48,681,818` Hz | `src/vi/mod.rs` |
| PAL VI / AI video clock | `49,656,530` Hz | `src/vi/mod.rs` |
| Frame cycle safety cap | `4,100,000` cycles (`CYCLE_CAP`) | `src/machine.rs` |
| Standard RDRAM | `4 MiB` (`0x0040_0000`) + `2 MiB` hidden bits | `src/memory/bus.rs` |
| Expanded RDRAM (Expansion Pak) | `8 MiB` (`0x0080_0000`) + `4 MiB` hidden bits | `src/memory/bus.rs` |

Each `Machine::run_frame` (`src/machine.rs`) advances the VR4300 until `vi.fields` increments (one video field boundary) or `CYCLE_CAP` cycles elapse. Every executed instruction advances `bus.cycles` by `1 + extra_cycles`, steps the RSP in lockstep when unhalted, and dispatches scheduled device events (`vi_catch`, `ai_catch`, `si_catch` in that order) whenever `bus.cycles >= bus.next_event`.

---

## 2. VR4300 CPU (`src/cpu/`)

### 2.1 Register File and Pipeline Model (`src/cpu/mod.rs`, `src/cpu/interp.rs`)

- **General-purpose registers:** 32 × 64-bit registers (`gpr[0..32]`), with `r0` hardwired to zero. 32-bit operations sign-extend their result into all 64 bits (`write32`).
- **Multiply/divide registers:** 64-bit `hi` and `lo`.
- **Program counter and branch delay:** `pc`, `next_pc`, `current_pc`, `branch_pending`, and `in_delay_slot`. Every branch and jump executes one delay-slot instruction before transferring control to the target. Branch-likely instructions (`BEQL`, `BNEL`, `BLEZL`, `BGTZL`, `BLTZL`, `BGEZL`, `BLTZALL`, `BGEZALL`, `BC1FL`, `BC1TL`) nullify the delay slot when not taken by advancing `pc = next_pc` and `next_pc = pc + 4` directly. Link instructions write the return address (`pc + 8` from the branch instruction) regardless of whether the branch is taken.
- **COP2 latch:** `cop2_latch` (`u64`) backs `MFC2`/`DMFC2`/`MTC2`/`DMTC2` when `Status.CU2` is set; `CFC2` reads `0`, `CTC2` is a no-op, and other COP2 encodings raise `ReservedInstruction`.
- **Load-linked / Store-conditional:** `LL`/`LLD` set `linked_flag = true` and write `physical >> 4` to COP0 `LLAddr` (`17`). `SC`/`SCD` perform the store only when `linked_flag` is set, write `1` or `0` to `rt`, and clear `linked_flag`. Exceptions and `ERET` clear `linked_flag`.
- **Instruction stalls and arithmetic quirks:**
  - Base instruction cost is 1 cycle. Multiply and divide instructions add fixed stall cycles (`extra_cycles`):

| Operation | Extra Stall Cycles | Total Cycles |
| --- | --- | --- |
| `MULT` / `MULTU` | `5` (`MULTIPLY_STALL`) | `6` |
| `DMULT` / `DMULTU` | `8` (`MULTIPLY_DOUBLE_STALL`) | `9` |
| `DIV` / `DIVU` | `37` (`DIVIDE_STALL`) | `38` |
| `DDIV` / `DDIVU` | `69` (`DIVIDE_DOUBLE_STALL`) | `70` |

  - Signed 32-bit `MULT` reads `rs` as a full 64-bit signed integer and sign-extends `rt` from **35 bits** (`MULTIPLY_OPERAND_BITS = 35`), matching VR4300 hardware behavior on non-canonical 32-bit inputs, then sign-extends the low and high 32-bit halves into `lo` and `hi`.
  - Integer division never raises exceptions:
    - `DIV` by 0: `lo = 1` if dividend `< 0`, else `-1` (`u64::MAX`); `hi = dividend` (sign-extended).
    - `DIV` of `i32::MIN / -1`: `lo = i32::MIN` (sign-extended), `hi = 0`.
    - `DIVU` by 0: `lo = u64::MAX`, `hi = dividend` (sign-extended from 32 bits).
    - `DDIV` / `DDIVU` follow the same 64-bit rules without 32-bit truncation.
  - `SRA` and `SRAV` shift the full 64-bit `rt` value arithmetically before truncating and sign-extending from 32 bits (`shift_right_arithmetic`).
  - `LWR` preserves the upper 32 bits of `rt` when `shift != 0` and sign-extends from 32 bits only when `shift == 0` (all four bytes loaded). `LWL`, `LDL`, `LDR`, `SWL`, `SWR`, `SDL`, `SDR` merge bytes at the aligned word or doubleword boundary.
  - `CACHE` (`0x2F`) validates 4-byte alignment and translates the virtual address (raising `AddressErrorLoad` or TLB exceptions if invalid), then performs no cache-line side effect.
  - Trap immediates (`TGEI`, `TGEIU`, `TLTI`, `TLTIU`, `TEQI`, `TNEI`) sign-extend the 16-bit immediate to 64 bits even for unsigned comparisons.

### 2.2 Privilege Modes, Virtual Segments, and `Status.RE` (`src/cpu/segments.rs`, `src/cpu/cop0.rs`)

The active privilege mode (`Mode::Kernel`, `Mode::Supervisor`, `Mode::User`) is derived from `Status`:
- If `Status.EXL` (bit 1) or `Status.ERL` (bit 2) is set, the CPU operates in **Kernel** mode regardless of `Status.KSU` (bits 4:3).
- Otherwise `KSU == 1` selects **Supervisor**, `KSU == 2` selects **User**, and `KSU == 0` or `3` selects **Kernel**.

64-bit addressing (`wide_addressing`) is controlled per mode by `Status.UX` (bit 5), `Status.SX` (bit 6), and `Status.KX` (bit 7). In Supervisor and User modes, executing any 64-bit doubleword instruction (`DSLL*`, `DSRL*`, `DSRA*`, `DMULT*`, `DDIV*`, `DADD*`, `DSUB*`, `DADDI*`, `LDL`, `LDR`, `SDL`, `SDR`, `LLD`, `LD`, `SCD`, `SD`) without the mode's extended addressing bit set raises `ReservedInstruction`.

**32-bit (narrow) address map** (also used for compatibility addresses `>= 0xFFFF_FFFF_8000_0000` in wide mode):
- Addresses must be sign-extended 32-bit values (`address == address as i32 as i64 as u64`), or they raise an address error.

| Virtual Range (32-bit) | Segment | Kernel | Supervisor | User |
| --- | --- | --- | --- | --- |
| `0x0000_0000..0x7FFF_FFFF` | `kuseg` / `useg` | TLB Mapped | TLB Mapped | TLB Mapped |
| `0x8000_0000..0x9FFF_FFFF` | `kseg0` | Direct (`addr & 0x1FFF_FFFF`) | Illegal | Illegal |
| `0xA000_0000..0xBFFF_FFFF` | `kseg1` | Direct (`addr & 0x1FFF_FFFF`) | Illegal | Illegal |
| `0xC000_0000..0xDFFF_FFFF` | `ksseg` / `sseg` | TLB Mapped | TLB Mapped | Illegal |
| `0xE000_0000..0xFFFF_FFFF` | `kseg3` | TLB Mapped | Illegal | Illegal |

**64-bit (wide) address map** (`address < 0xFFFF_FFFF_8000_0000`):

| Top 2 Bits (`63:62`) | Segment | Permitted Modes | Valid Bits & Translation |
| --- | --- | --- | --- |
| `00` | `xkuseg` | Kernel, Supervisor, User | Bits `61:40` must be `0`; TLB Mapped |
| `01` | `xksseg` | Kernel, Supervisor | Bits `61:40` must be `0`; TLB Mapped |
| `10` | `xkphys` | Kernel | Bits `58:32` must be `0` (`PHYSICAL_GAP`); Direct (`addr as u32`) |
| `11` | `xkseg` | Kernel | Bits `61:40` must be `0` and `< 0xC000_00FF_8000_0000`; TLB Mapped |

**Reverse-endian mode (`Status.RE`, bit 25):** Active only in `Mode::User`. Every instruction fetch and data load/store mirrors its effective address within its enclosing 8-byte doubleword prior to TLB translation:
```
access = (address ^ 7) & !(size - 1)
```
Alignment checks and faulting `BadVAddr` reporting still use the unmirrored virtual `address`. Unaligned partial loads/stores (`LWL`/`LWR`/`LDL`/`LDR`/`SWL`/`SWR`/`SDL`/`SDR`) mirror with `size = 1` before aligning to the 4- or 8-byte word.

### 2.3 Coprocessor 0, Timer, and Exceptions (`src/cpu/cop0.rs`)

Kernel mode accesses COP0 unconditionally; Supervisor and User modes require `Status.CU0` (bit 28) or raise `CoprocessorUnusable` (coprocessor 0).

| Reg | Name | Write Mask / Read Behavior |
| --- | --- | --- |
| `0` | `Index` | `value & 0x8000_003F` (bit 31 = TLB probe miss, bits 5:0 = index) |
| `1` | `Random` | Read-only; counts down from `31` to `Wired` per instruction (`period = 32 - wired` if `wired <= 31`, else `96 - wired`, masked to 6 bits) |
| `2` | `EntryLo0` | `value & 0x3FFF_FFFF` |
| `3` | `EntryLo1` | `value & 0x3FFF_FFFF` |
| `4` | `Context` | Writable bits `63:23` (`0xFFFF_FFFF_FF80_0000`); bits `22:4` (`BadVPN2`) filled by hardware on address/TLB faults |
| `5` | `PageMask` | `value & 0x01FF_E000` |
| `6` | `Wired` | `value & 0x3F`; writing resets `random_start = instructions` |
| `8` | `BadVAddr` | Read-only; populated on address error and TLB exceptions |
| `9` | `Count` | Read returns `(cycles >> 1) + count_bias`; write updates `count_bias` and reschedules timer |
| `10` | `EntryHi` | `value & 0xC000_00FF_FFFF_E0FF` (Region `63:62`, VPN2 `39:13`, ASID `7:0`) |
| `11` | `Compare` | 32-bit timer target; writing clears `Cause.IP7` (bit 15) and reschedules `timer_due` |
| `12` | `Status` | `value & 0xFFF7_FFFF`; updates privilege mode and interrupt recheck |
| `13` | `Cause` | `IP2` (bit 10) driven by MI level, `IP7` (bit 15) edge-latched by Count/Compare, `ExcCode` bits `6:2`, `CE` bits `29:28`, `BD` bit `31` |
| `14` | `EPC` | Saved faulting PC (`current_pc - 4` if in delay slot) when `Status.EXL == 0` |
| `15` | `PRId` | Read-only constant `0x0000_0B22` |
| `16` | `Config` | `(value & 0x0F00_800F) \| 0x7006_6460`; reset value `0x7006_E463` |
| `17` | `LLAddr` | `value & 0xFFFF_FFFF` |
| `20` | `XContext` | Writable bits `63:33` (`0xFFFF_FFFE_0000_0000`); Region `32:31` and BadVPN2 `30:4` filled on faults |
| `26` | `PErr` | `value & 0xFF` |
| `27` | `CacheErr` | Hardwired to `0` |
| `28` | `TagLo` | `value & 0xFFFF_FFFF` |
| `29` | `TagHi` | Hardwired to `0` |
| `30` | `ErrorEPC` | Full 64-bit read/write |
| `7, 21..25, 31` | Unused | Writes update `cop0_latch` only; reads return `cop0_latch` |

**Count/Compare timer:** When `Count` crosses `Compare`, `timer_reached` latches `Cause.IP7` (`1 << 15`), records `last_count`, and schedules the next wrap (`2^32` half-cycles later). Writing `Compare` clears `IP7`.

**Interrupts:** Checked before fetching an instruction whenever `recheck` is set or MI's asserted level changes. An interrupt fires (`ExcCode = 0`) when `Status.IE == 1`, `Status.EXL == 0`, `Status.ERL == 0`, and `((Cause >> 8) & (Status >> 8) & 0xFF) != 0`.

**Exception vectors:** Base is `0xFFFF_FFFF_BFC0_0200` when `Status.BEV` (bit 22) is set, otherwise `0xFFFF_FFFF_8000_0000`.
- TLB Refill (`EXL == 0` and miss): `+0x000` (narrow addressing) or `+0x080` (wide addressing).
- All other exceptions (including nested TLB misses when `EXL == 1`): `+0x180`.

**`ERET`** returns to `ErrorEPC` and clears `Status.ERL` if `ERL == 1`, otherwise returns to `EPC` and clears `Status.EXL`; clears `linked_flag` and executes without a delay slot.

### 2.4 Translation Lookaside Buffer (`src/cpu/tlb.rs`)

The VR4300 TLB holds **32 paired-page entries** (`entries[0..32]`), searched linearly from index `0` to `31`.

- **Page sizes:** `PageMask` bits `24:13` (`0x01FF_E000`) are paired via `paired_page_mask(raw)`: each two-bit pair takes the value of its upper bit (`higher = raw & 0x0155_4000; higher | (higher >> 1)`), supporting 4 KiB (`0x0000_0000`) through 16 MiB (`0x01FF_E000`) per half-page.
- **Entry storage on `TLBWI`/`TLBWR`:**
  - `global = EntryLo0.G & EntryLo1.G` (single shared global bit stored into both halves).
  - `entry_hi = EntryHi & 0xC000_00FF_FFFF_E0FF & !page_mask`.
  - `entry_lo0` / `entry_lo1` keep `0x03FF_FFFE` (20-bit PFN + cache/dirty/valid) ORed with `global`.
- **Translation (`try_translate`):** Matches `((addr ^ entry_hi) & !(page_mask | 0x1FFF) & 0xC000_00FF_FFFF_FFFF) == 0` and `(global || entry_asid == asid)`. Selects `entry_lo1` if `(addr & ((pair_mask >> 1) + 1)) != 0`, else `entry_lo0`.
  - `V == 0` -> `TLBLoad` / `TLBStore` (non-refill).
  - Store with `D == 0` -> `TLBModification`.
  - No entry matched -> `TLBLoad` / `TLBStore` (refill).
  - Matched -> `physical = ((PFN << 12) & !half_mask) | (addr & half_mask)`.
- **`TLBP`:** Writes the matching index (`0..31`) to `Index`, or `0x8000_0000` on miss.

### 2.5 Floating-Point Unit — COP1 Soft-Float (`src/cpu/cop1.rs`, `src/cpu/softfloat.rs`)

COP1 requires `Status.CU1` (bit 29) or raises `CoprocessorUnusable` (coprocessor 1).

- **Register file modes (`Status.FR`, bit 26):**
  - `FR == 1` (full mode): 32 independent 64-bit registers (`fpr[0..32]`).
  - `FR == 0` (half mode): 16 even/odd 64-bit pairs. `LWC1`/`SWC1`/`MFC1`/`MTC1` on odd register `n` access the upper 32 bits of `fpr[n & !1]`; `LDC1`/`SDC1`/`DMFC1`/`DMTC1` and the `fs` operand of arithmetic instructions mask `index & !1`. Word moves preserve the untouched half of the register, whereas computed 32-bit results clear the upper 32 bits of `fpr[fd]`.
- **Control registers:** `FCR0` (FIR) returns `0x0000_0A00`; `FCR31` (`FCSR`) has writable mask `0x0183_FFFF` (flush-to-zero bit 24, condition bit 23, causes bits 17:12, enables bits 11:7, sticky flags bits 6:2, rounding mode bits 1:0). Writing `FCSR` immediately triggers a `FloatingPoint` exception (`ExcCode = 15`) if any enabled cause or the unimplemented cause (bit 17, always unmasked) is set.
- **Software IEEE/VR4300 arithmetic (`src/cpu/softfloat.rs`):**
  - **Formats:** Single (`23`-bit mantissa, `8`-bit exponent, bias `127`, default quiet NaN `0x7FBF_FFFF`) and Double (`52`-bit mantissa, `11`-bit exponent, bias `1023`, default quiet NaN `0x7FF7_FFFF_FFFF_FFFF`).
  - **NaN & subnormal rules:** On MIPS III / VR4300, the top mantissa bit set (`nan_select_bit`) identifies a signaling/supported NaN (`FloatClass::Nan`) that produces `default_nan` + `INVALID` on arithmetic; top mantissa bit clear (`FloatClass::NanUnsupported`, IEEE quiet NaN) and all **subnormal** inputs raise an **Unimplemented Operation** (`FCSR` bit 17) without modifying the destination.
  - **Underflow & Flush-to-Zero (`FCSR.FS`, bit 24):**
    - If `FS == 0`, any underflowing result raises Unimplemented Operation.
    - If `FS == 1` and neither Underflow enable (bit 8) nor Inexact enable (bit 7) is set, the underflow flushes to `±smallest_normal` when rounding away from zero (`ROUND_POSITIVE` with `+`, `ROUND_NEGATIVE` with `-`), or `±0` otherwise, raising `UNDERFLOW | INEXACT`.
    - If Underflow or Inexact trap enable is set on an underflowing result, VR4300 raises Unimplemented Operation instead.
  - **Integer conversions:** `CVT.W`/`CVT.L`/`ROUND`/`TRUNC`/`CEIL`/`FLOOR` refuse (Unimplemented Operation) on NaN, Infinity, Subnormal, or out-of-range magnitudes (`exponent >= 32` for word, `exponent >= 53` for long, or rounded magnitude exceeding signed range). `CVT.S.L`/`CVT.D.L` refuse 64-bit integers outside `[-2^55, 2^55 - 1]`.
  - **Rounding modes (`FCSR` bits 1:0):** `0` = Round-to-Nearest-Even, `1` = Round-Toward-Zero, `2` = Round-Toward-`+∞`, `3` = Round-Toward-`-∞`.
  - **Host fast path (`host_single`):** Single-precision `ADD.S`, `SUB.S`, `MUL.S` in `ROUND_NEAREST` execute in host `f64` when both operands are normal, exponent difference for add/sub is `<= 28`, and the exact result is normal and finite; every edge case falls back to the 128-bit integer soft-float unit.

---

## 3. Physical Memory Map (`src/memory/bus_access.rs`)

| Physical Address Range | Size / Window | Target | Notes |
| --- | --- | --- | --- |
| `0x0000_0000..0x003F_FFFF` (or `0x007F_FFFF`) | 4 MiB / 8 MiB | RDRAM (`bus.rdram`) | Paired with `bus.rdram_hidden` (1 byte per 16-bit word, storing 2 hidden bits) |
| `0x0040_0000..0x007F_FFFF` (4 MiB config) | 4 MiB | Unpopulated RDRAM | Reads `0`, writes ignored |
| `0x03F0_0000..0x03FF_FFFF` | 1 MiB | RDRAM Registers | Repeats 16-word post-IPL3 table every 64 B (`[0]=0xB419_0010`, `[2]=0x2B3B_1A0B`, `[6]=0x101C_0A04`); writes ignored |
| `0x0400_0000..0x0403_FFFF` | 256 KiB | SP DMEM / IMEM | 4 KiB DMEM + 4 KiB IMEM alternating every 8 KiB; whole-word latching |
| `0x0404_0000..0x0404_001F` | 32 B | SP Registers | DMA, status, semaphore (`src/memory/sp.rs`) |
| `0x0408_0000..0x0408_0007` | 8 B | SP PC | RSP 12-bit program counter (`0xFFC`) |
| `0x0410_0000..0x0410_001F` | 32 B | DP Command Registers | RDP command DMA and status (`src/memory/dp.rs`) |
| `0x0430_0000..0x0430_000F` | 16 B | MI Registers | MIPS Interface (`src/memory/mi.rs`) |
| `0x0440_0000..0x0440_0037` | 56 B | VI Registers | 14 Video Interface registers (`src/vi/mod.rs`) |
| `0x0450_0000..0x0450_0017` | 24 B | AI Registers | Audio Interface (`src/memory/ai.rs`) |
| `0x0460_0000..0x0460_0033` | 52 B | PI Registers | Peripheral Interface (`src/memory/pi.rs`) |
| `0x0470_0000..0x0470_001F` | 32 B | RI Registers | Stubbed in `bus.registers` with `RI_SELECT` (`0x0470_000C`) initialized to `0x14` |
| `0x0480_0000..0x0480_001B` | 28 B | SI Registers | Serial Interface (`src/memory/si.rs`) |
| `0x0500_0000..0x07FF_FFFF` | 48 MiB | Cart Domain 2 Addr 1 | Cartridge bus (64DD region); unmapped reads `0` |
| `0x0800_0000..0x0FFF_FFFF` | 128 MiB | Cart Domain 2 Addr 2 | Save window (SRAM / FlashRAM); whole-word latching |
| `0x1000_0000..0x1FBF_FFFF` | ~252 MiB | Cart Domain 1 Addr 2 | Cartridge ROM (`read_cart32` with optional ROM patches); reads past ROM end return `0` |
| `0x13FF_0000..0x13FF_0FFF` | 4 KiB | IS-Viewer | Debug text buffer & length trigger (`src/memory/isviewer.rs`) |
| `0x1FC0_0000..0x1FC0_07BF` | 1,984 B | PIF ROM | Unmodeled stub space (boot uses HLE handoff) |
| `0x1FC0_07C0..0x1FC0_07FF` | 64 B | PIF RAM | Joybus / CIC-6105 command block; whole-word latching |

**Bus width and latching rules:**
- **Whole-word latching windows** (SP DMEM/IMEM, PIF RAM, Save window `0x0800_0000..0x0FFF_FFFF`): any CPU store (`SB`, `SH`, `SW`, `SD`) writes a full 32-bit word (`value << (8 * (4 - size - (addr & 3)))` for `size < 8`, or the upper 32 bits for `SD`).
- **Cartridge bus store latch (`0x0500_0000..0x1FBF_FFFF`, excluding IS-Viewer):** A CPU store latches its whole word in `pi.stored` for `225` CPU cycles (`STORE_DECAY_CYCLES`, equivalent to 150 RCP cycles at 62.5 MHz), setting `PI_STATUS` I/O busy. Subsequent stores while busy are dropped; the next CPU read `< 8` bytes from cartridge ROM consumes and returns the latched word once.
- **Unmodeled stub registers:** Addresses outside modeled regions read/write `bus.registers` (`BTreeMap<u32, u32>`), preserving values across save states.

---

## 4. Hardware Interfaces

### 4.1 MIPS Interface — MI (`src/memory/mi.rs`, `0x0430_0000`)

| Offset | Register | Read | Write |
| --- | --- | --- | --- |
| `0x00` | `MI_MODE` | `repeat_count[6:0] \| repeating<<7 \| ebus<<8 \| upper<<9` | `[6:0]` repeat count; bit `7`/`8` clear/set repeat; `9`/`10` clear/set ebus; `11` clear DP interrupt; `12`/`13` clear/set upper. Set wins over clear if both are set. |
| `0x04` | `MI_VERSION` | `0x0202_0102` (`RCP_VERSION`) | Ignored |
| `0x08` | `MI_INTR` | Pending bits `[5:0]` | Ignored |
| `0x0C` | `MI_INTR_MASK` | Mask bits `[5:0]` | Bit pairs `2*i` (clear) / `2*i+1` (set) for sources `0..5` |

**Interrupt sources (`MiInterrupt`):** bit `0` = SP, `1` = SI, `2` = AI, `3` = VI, `4` = PI, `5` = DP. MI drives VR4300 `Cause.IP2` (`1 << 10`) level-triggered whenever `(pending & mask) != 0`.

**Repeat mode:** When `mi.repeating` is armed, the next CPU store to RDRAM (`< 0x03F0_0000`) clears `repeating` and fills `repeat_count + 1` bytes starting at `physical`, wrapping within the 2 KiB RDRAM row (`REPEAT_ROW_MASK = 0x7FF`) using the 64-bit pattern (doubled 32-bit word for `size < 8`).

### 4.2 Peripheral Interface — PI (`src/memory/pi.rs`, `0x0460_0000`)

| Offset | Register | Read | Write |
| --- | --- | --- | --- |
| `0x00` | `PI_DRAM_ADDR` | `dram_address` | `value & 0x00FF_FFFE` |
| `0x04` | `PI_CART_ADDR` | `cart_address` | `value & 0xFFFF_FFFE` |
| `0x08` | `PI_RD_LEN` | `0` | Start RDRAM -> Cart DMA of `(value & 0x00FF_FFFF) + 1` bytes |
| `0x0C` | `PI_WR_LEN` | `0` | Start Cart -> RDRAM DMA of `(value & 0x00FF_FFFF) + 1` bytes |
| `0x10` | `PI_STATUS` | Bit `1` = I/O busy (`cycles < stored_until`); bit `3` = PI interrupt pending | Bit `1` clears `MI_INTR_PI` |

**DMA behavior:**
- Transfers execute immediately and raise `MI_INTR_PI`.
- **Cart -> RDRAM (`pi_from_cartridge`):** Splits transfers into blocks of at most 128 bytes (`BLOCK_SIZE`), clamped at 2 KiB RDRAM row boundaries (`ROW_SIZE = 0x800`). Emulates the hardware misalignment quirk: a misaligned `dram_address & 7` reduces both the block read count and bytes stored on the first block, advancing `dram_address` to the next 8-byte boundary after each block and `cart_address` by `(block + 1) & !1`.
- **RDRAM -> Cart (`pi_to_cartridge`):** Copies `length` bytes, rounding `cart_address` up to 2 bytes and `dram_address` up to 8 bytes.

### 4.3 Serial Interface — SI and Joybus (`src/memory/si.rs`, `src/memory/joybus.rs`, `0x0480_0000`)

| Offset | Register | Read | Write |
| --- | --- | --- | --- |
| `0x00` | `SI_DRAM_ADDR` | `dram_address` | `value & 0x00FF_FFF8` |
| `0x04` | `SI_PIF_AD_RD64B` | `0` | Trigger 64-byte PIF RAM -> RDRAM DMA |
| `0x10` | `SI_PIF_AD_WR64B` | `0` | Trigger 64-byte RDRAM -> PIF RAM DMA |
| `0x18` | `SI_STATUS` | Bit `0` = DMA busy (`due != i64::MAX`); bit `12` = SI interrupt pending | Any write clears `MI_INTR_SI` |

**Timing:** Each 64-byte transfer schedules completion at `cycles + 0x1200` (`TRANSFER_CYCLES = 0x900 * 2` = 4,608 CPU cycles). On a PIF -> RDRAM read, PIF processes its command block immediately, buffers the destination address in `pending_read`, copies the 64 bytes into RDRAM when `due` expires, and raises `MI_INTR_SI`.

**Joybus command block (`joybus::run`):**
- Walks the 64-byte PIF RAM across channels `0..3` (controller ports 1–4) and channel `4` (cartridge EEPROM):
  - `0xFE` (`END`): terminate block; `0x00` (`SILENT`): advance channel; `0xFD` (`PAD`) / `0xFF` (`SKIP`): skip byte.
  - Otherwise `send = ram[at] & 0x3F`, `receive = ram[at + 1] & 0x3F`. Unanswered channels set bit `7` (`0x80`, `NO_REPLY`) in the receive-length byte; replies longer than `receive` set bit `6` (`0x40`, `OVER_RUN`).
- **Controller commands:**
  - `0x00` (`INFO`) / `0xFF` (`RESET`): replies `[0x05, 0x00, pak]` where `pak` is `0x02` (no pak), `0x01` (pak present), or `0x03` (new pak inserted since last status check; clears `inserted` flag).
  - `0x01` (`STATE`): replies 4 bytes `[buttons_hi, buttons_lo, stick_x, stick_y]`.
  - `0x02` (`PAK_READ`): reads 32-byte chunk at `((hi << 8) | (lo & 0xE0))` plus 1-byte CRC-8.
  - `0x03` (`PAK_WRITE`): writes 32-byte chunk and replies with 1-byte CRC-8.

### 4.4 Audio Interface — AI (`src/memory/ai.rs`, `0x0450_0000`)

| Offset | Register | Read | Write |
| --- | --- | --- | --- |
| `0x00` | `AI_DRAM_ADDR` | Remaining `length` | Buffers `value & 0x00FF_FFF8` into active (`queued == 0`) or next (`queued == 1`) slot |
| `0x04` | `AI_LEN` | Remaining `length` | Queues `value & 0x0003_FFF8` bytes (2-deep FIFO); raises `MI_INTR_AI` when a buffer **begins** |
| `0x08` | `AI_CONTROL` | Remaining `length` | Bit `0` = `dma_enabled` |
| `0x0C` | `AI_STATUS` | `0x8000_0001` (full, `queued > 1`) \| `0x4000_0000` (busy, `queued > 0`) \| `0x0200_0000` (DMA enabled) \| `0x0110_0000` (always set) | Any write clears `MI_INTR_AI` |
| `0x10` | `AI_DACRATE` | Remaining `length` | `value & 0x3FFF` (14-bit divider) |
| `0x14` | `AI_BITRATE` | Remaining `length` | Ignored |

**Sample rate and DMA quirks:**
- Sample period is `max(dac_rate + 1, 0x200)` (`SHORTEST_PERIOD = 512`) video clocks, yielding `round(video_clock / period)` Hz (`44,100` Hz default while `dac_rate == 0`).
- Each sample fetches one big-endian stereo `i16` pair `(left, right)` from RDRAM. Crossing an 8 KiB page (`PAGE_SIZE = 0x2000`) emulates the hardware delayed-carry quirk: the address wraps within the current 8 KiB page and sets `carry = true`, adding `0x2000` on the *following* sample fetch.
- Output samples queue into `ai.samples` (default cap `128,000` samples, dropping oldest pairs if exceeded).

### 4.5 SP, DP, VI, and RI Interfaces (Summary)

- **SP (`0x0404_0000`, `0x0408_0000`):** 2D rectangular DMA between RDRAM and DMEM/IMEM, RSP halt/break/single-step/signal control, and semaphore. Detailed in `Mars_Rsp.md`.
- **DP (`0x0410_0000`):** RDP command FIFO pointers (`START`/`END`/`CURRENT`) and `STATUS` (`XBUS`, `FREEZE`, `GCLK`, `PIPE_BUSY`). Detailed in `Mars_RdpAndVi.md`.
- **VI (`0x0440_0000`):** 14 registers driving half-line timing, `MI_INTR_VI`, and raster scan-out. Detailed in `Mars_RdpAndVi.md`.
- **RI (`0x0470_0000`):** Unmodeled beyond `RI_SELECT` (`0x0470_000C`) preset to `0x14` so IPL3 skips RDRAM calibration.

---

## 5. PIF Boot Handoff, CIC, ROM Formats, and IS-Viewer

### 5.1 ROM Detection and Normalization (`src/rom/mod.rs`)

Container byte order is identified from the first 32-bit big-endian word (never the file extension) and normalized to big-endian `.z64` layout (minimum length `0x1000` bytes):

| First 4 Bytes (BE) | Format | Conversion |
| --- | --- | --- |
| `0x8037_1240` | `.z64` (big-endian) | Native; copied directly |
| `0x3780_4012` | `.v64` (byte-swapped 16-bit) | Requires even length; swaps each 2-byte pair |
| `0x4012_3780` | `.n64` (little-endian 32-bit) | Requires multiple of 4 bytes; reverses each 4-byte word |

- **Region (`is_pal`):** Header byte `0x3E` matching `D`, `F`, `I`, `P`, `S`, `U`, `X`, or `Y` selects PAL; all others select NTSC.
- **ROM patches (`src/memory/rom_patches.rs`):** Sorted `(address, value, optional compare)` byte overrides applied dynamically inside `read_cart32` so both CPU reads and PI DMAs observe patched bytes without mutating the shared `RomImage`.

### 5.2 CIC Identification, Boot Handoff, and CIC-6105 Challenge (`src/machine.rs`, `src/rom/mod.rs`, `src/memory/si.rs`)

`rom::identify` sums every 32-bit big-endian word of IPL3 (`0x0040..0x0FFF`) into a 64-bit integer:

| CIC Variant | IPL3 64-bit Word Sum(s) | Seed (`$s6` / `r22`) | RDRAM Size Address |
| --- | --- | --- | --- |
| `NUS-6101` | `0x0000_00D0_027F_DF31`, `0x0000_00CF_FB63_1223` | `0x3F` | `0x0000_0318` |
| `NUS-6102` | `0x0000_00D0_57C8_5244`, `0x0000_007C_5624_2373` | `0x3F` | `0x0000_0318` |
| `NUS-6103` | `0x0000_00D6_497E_414B` | `0x78` | `0x0000_0318` |
| `NUS-6105` | `0x0000_011A_49F6_0E96` | `0x91` | `0x0000_03F0` |
| `NUS-6106` | `0x0000_00D6_D5BE_5580` | `0x85` | `0x0000_0318` |
| Unknown | Any other sum | `0x3F` | `0x0000_0318` |

**Boot handoff (`machine::hand_off`):**
1. Copies the first `0x1000` bytes of ROM into SP DMEM (`0x0400_0000`).
2. Writes the 8-word `IPL2_HEAD` stub into SP IMEM (`0x0400_1000`).
3. Writes the installed RDRAM byte count (`0x0040_0000` or `0x0080_0000`) big-endian to `0x0000_0318` (or `0x0000_03F0` for CIC-6105).
4. Initializes VR4300 state: `pc = 0xFFFF_FFFF_A400_0040`, `next_pc = pc + 4`, `r11 = pc`, `r29 (sp) = 0xFFFF_FFFF_A400_1FF0`, `r31 (ra) = 0xFFFF_FFFF_A400_1550`, `r20 (s4) = 0 (PAL) / 1 (NTSC)`, `r22 (s6) = cic_seed`, `Status = 0x3400_0000`, `Compare = 0xFFFF_FFFF`, `PRId = 0x0B22`, `Config = 0x7006_E463`.

**CIC-6105 challenge/response (`si_answer_challenge`, `rom::respond`):** When an SI read DMA executes with PIF RAM control byte (`pif_ram[0x3F]`) bit `0x02` set and the cartridge uses `NUS-6105`, PIF clears `pif_ram[0x2E..0x30]`, transforms the 15-byte challenge at `pif_ram[0x30..0x3F]` nibble-by-nibble (high nibble first, starting from `key = 0xB`, `mode = 0`, using the 32-entry LUT), writes the 15-byte response back in place, and clears bit `0x02`. For non-6105 cartridges, the flag is cleared without altering the buffer.

### 5.3 IS-Viewer (`src/memory/isviewer.rs`)

Mapped at physical `0x13FF_0000..0x13FF_0FFF` (4 KiB):
- Offset `0x20..0xFFF`: text staging buffer (`4,064` bytes).
- Offset `0x14` (`LENGTH_REGISTER_OFFSET`): writing a non-zero 32-bit word appends `min(value, 0xFE0)` bytes from offset `0x20` into `is_viewer.text` and retains the register value. The Core ABI v1 host layer (`src/ffi/host.rs`) streams new lines to the host log under category `"IS-Viewer"`.

---

## 6. Save Chips and Controller Paks (`src/memory/save.rs`, `src/memory/controller.rs`)

### 6.1 Save Type Resolution

Save type is resolved in priority order:
1. **ED64 header** (`rom[0x3C..0x3E] == "ED"`): high nibble of `rom[0x3F]` maps `0` = None, `1` = EEPROM 4K, `2` = EEPROM 16K, `3` = SRAM 256K, `4` = Banked SRAM 768K, `5` = FlashRAM, `6` = SRAM 1M.
2. **16 Kbit EEPROM table:** 54 `(crc1, crc2)` pairs in `src/rom/mod.rs` (`EEPROM_16K_TITLES`).
3. **Existing save file length:** `0x200` / `0x800` -> EEPROM 4K, `0x8000` -> SRAM 256K, `0x18000` -> Banked SRAM 768K, `0x20000` -> FlashRAM.
4. **Runtime auto-detection (`SaveChip`):** While `UNKNOWN`, the first Joybus EEPROM read/write selects `EEPROM_4K` (an `INFO`/`RESET` probe answers as 4 Kbit without locking the type); the first CPU read/write to `0x0800_0000` selects `FLASH_RAM`; the first PI DMA to `0x0800_0000` selects `SRAM_256K`.

### 6.2 Cartridge Save Devices (`src/memory/save.rs`)

All uninitialized save arrays default to `0xFF`.

- **EEPROM 4 Kbit / 16 Kbit (`Eeprom`):** Backed by a `2,048`-byte (`0x800`) buffer, accessed via Joybus channel 4 in 8-byte blocks (`0x00`/`0xFF` info returns `[0x00, 0x80/0xC0, 0x00]`, `0x04` read block, `0x05` write block returning `[0x00]`).
- **SRAM (`Sram`):** `1` bank (`32 KiB`), `3` banks (`96 KiB`), or `4` banks (`128 KiB`), each bank `0x8000` bytes. Single-bank SRAM mirrors across `0x0800_0000..0x0FFF_FFFF`; multi-bank SRAM selects `bank = (offset >> 18) & 3` and reads `0` past the last installed bank. Unpopulated save window CPU reads return open bus `((offset & 0xFFFF) * 0x0001_0001)`.
- **FlashRAM (`FlashRam`):** `128 KiB` (`0x2_0000`) arranged as 1,024 pages of 128 bytes with a 128-byte write buffer and 64-bit status register:
  - CPU reads at `offset & 4 == 0` return `status >> 32`; `offset & 4 != 0` returns `status as u32`.
  - CPU writes at any `offset != 0` dispatch by `(value >> 24)`:
    - `0x4B`: set erase page (`value & 0x3FF`).
    - `0x78`: erase mode (`status = 0x1111_8008_00C2_001D`).
    - `0xA5`: set program page (`value & 0x3FF`, `status = 0x1111_8004_00C2_001D`).
    - `0xB4`: program mode (PI DMA writes fill the 128-byte page buffer).
    - `0xD2`: execute active erase (fill page with `0xFF`) or program (commit page buffer).
    - `0xE1`: status/identify mode (`status = 0x1111_8001_00C2_001D`).
    - `0xF0`: read mode (`status = 0x1111_8004_F000_001D`, PI DMA reads `data[offset & 0x1FFFF]`).

### 6.3 Controller Pak (`src/memory/controller.rs`, `emusen-n64-pak`)

Each of the 4 controller ports can hold a `32 KiB` (`0x8000`) Controller Pak accessed via Joybus commands `0x02`/`0x03` in 32-byte chunks (`CHUNK_SIZE = 32`) with CRC-8 polynomial `0x85` over the 32 data bytes plus a trailing zero byte. Address bits `4:0` (address CRC) are masked out (`(hi << 8) | (lo & 0xE0)`). Out-of-range reads (`>= 0x8000`) return zeroes; writes are ignored.

Empty paks are formatted with ID block copies at chunk indices `1, 3, 4, 6` (`0x20, 0x60, 0x80, 0xC0`) with checksum `(sum, 0xFFF2 - sum)` and mirrored 256-byte page-index tables at pages `1` and `2` marking pages `5..127` free (`0x0003`).

Core ABI v1 exposes the cartridge save (file `0`, `.srm`) and up to four removable Controller Pak slots (files `1..4`, `.mpk` / `.2.mpk`..`.4.mpk`, medium ID `"n64.pak"`). Host-side card inspection and editing of `"n64.pak"` files is provided by the shared `#![no_std]` crate `emusen-n64-pak` (`EmuSen/Cores/Shared/emusen-n64-pak`).

---

## 7. VR4300 Block Recompiler and Idle Skip (`src/cpu/blocks/`, `src/cpu/idle.rs`)

### 7.1 Execution Tiers and Compilation (`src/cpu/blocks/mod.rs`, `src/cpu/blocks/jit/`)

The recompiler is strictly exact against the interpreter and sits entirely outside save states (rebuilt on demand from RDRAM). Compilation uses Cranelift (`cranelift-codegen`, `cranelift-frontend`, `cranelift-jit`, `cranelift-module` `0.136.0`) managed by the shared `emusen-jit` crate (`EmuSen/Cores/Shared/emusen-jit`) on a background compiler thread.

| Tier | `BlockTier` Setting | Execution Strategy |
| --- | --- | --- |
| `Tier::Decoded` | `1` | Pre-decoded handler array (`src/cpu/blocks/decoded.rs`, `ops.rs`) |
| `Tier::Compiled` | `2` (and default `0` / `Tier::BEST`) | Background Cranelift JIT; guest GPRs live in `Cpu::gpr`, inline RDRAM loads, pure ALU/shifts/branches inline |
| `Tier::Cached` | `3` | Cranelift JIT with guest GPRs cached in host registers across the block and inline RDRAM stores (`inline_stores`) |

- **Hotness threshold:** `THRESHOLD = 64` block entries (`EMUSEN_MARSRT_THRESHOLD` overrides) before a block is enqueued to `emusen-jit`.
- **Code budget:** `CODE_BOUND = 256 MiB`; exceeding the budget flushes the entire JIT module and block cache and recompiles active blocks on demand.
- **Running RSP policy:** By default (`beside_variant = false`), blocks entered while the RSP is unhalted run via the decoded path rather than compiling a separate RSP-stepping variant (`EMUSEN_MARSRT_BESIDE=1` enables it).

### 7.2 Block Shaping, Boundaries, and Invalidation (`src/cpu/blocks/shape.rs`, `cache.rs`)

- **Eligibility:** Blocks run only when `branch_pending == false`, `mode == Mode::Kernel`, `pc & 3 == 0`, `extra_cycles == 0`, and `pc` maps into RDRAM (either KSEG0/KSEG1 direct, or a TLB-mapped 32-bit kernel page cached by `(page, tlb_generation)`). Mapped blocks never cross a 4 KiB page boundary.
- **Shape limits (`MAX_WORDS = 64`):** `shape::shape` classifies each instruction as `Plain`, `Branch`, or `Ender` (COP0 writes/TLB/`ERET`, `SYSCALL`, `BREAK`, reserved opcodes).
  - `Ender` ends the block inclusively.
  - `Branch` includes its delay slot and ends after it, provided the slot fits within `limit` and is `Plain`; otherwise the branch is excluded (or refused if at word 0, leaving it to one interpreter step).
- **Cache layout and validation (`cache.rs`):** Indexed by physical word address (`physical >> 2`) in lazily allocated pages of `1,024` slots. On **every** block entry, `Block::matches` compares the live RDRAM bytes against `block.image`; any mismatch (DMA, self-modifying code, state load) discards and re-shapes the block.
- **Mid-block safety:**
  - Compiled blocks enter only when `bus.cycles + block.max_cycles < stop` (where `stop = min(bus.next_event, cpu.run.timer_due, cap_at)`), guaranteeing no device event, Count/Compare match, or frame cap fires mid-block.
  - Any store landing in `[block.start - 7 .. block.end)` or going through a bus device (`THROUGH_BUS`) exits the block immediately (`FINISH`) so the dispatcher ticks and revalidates.
  - Inline RDRAM loads/stores in JIT code test the threaded RDP's `read_marks` / `write_marks` page tables and bail to the slow handler when a page is marked.
- **Verification (`src/cpu/blocks/verify.rs`):** Enabling `VerifyBlocks` (`EMUSEN_MARSRT_VERIFY_BLOCKS=1`) runs a shadow interpreter lockstep beside every decoded/compiled instruction and asserts architectural equality.

### 7.3 Idle-Loop Skipping (`src/cpu/idle.rs`)

Whenever a branch targets its own `current_pc`, `cpu.run.idle_at` records that address. If the CPU reaches `idle_at` in Kernel mode with `branch_pending == false`, and memory holds `BEQ r0, r0, -1` (`0x1000_FFFF`) or a direct `J` to itself followed by a `NOP` (`0x0000_0000`), `try_idle` checks interrupts once and then advances whole 2-cycle loop turns up to `min(next_event, timer_due, cap_at)`. While the RSP is running and `rsp_whole` is enabled (and RSP single-step/tracing are off), `run_idle` advances the RSP in bulk via `rsp_run_to_event`, stopping early if any bus write or MI interrupt assertion occurs.

---

## 8. Save State Format, Core ABI v1, and Accuracy Boundaries

### 8.1 Save States and Snapshots (`src/machine.rs`, `src/state.rs`)

| Field | Value / Behavior |
| --- | --- |
| Magic | `0x5352_414D` (`"MARS"` little-endian) |
| Version `1` (`STATE_VERSION`) | Drained machine state; refuses to serialize if pending RDP words exist (`StateError::PendingInState`, status `-14`) |
| Version `2` (`SNAPSHOT_VERSION`) | Same body followed by pending RDP command count + padded `32,768` (`SNAPSHOT_WORDS`) 64-bit command words (overflow returns `-13`) |
| RDRAM size validation | Accepts `4 MiB` or `8 MiB` only (otherwise `StateError::RdramSize`, status `-12`); loading a state with the other size rebuilds the machine cleanly |
| Atomic load guarantee | Failed loads leave the live `Machine` unmodified |
| Excluded derived state | Recompiler blocks/JIT code, RDP threads/page marks, multi-resolution shadow buffers, GPU device handle, debug hooks, and undrained audio queue are excluded; `derive_after_load` rebases VI/AI clocks, replays snapshot RDP words, and re-runs `cop0_written` |

### 8.2 Core ABI v1 Surface (`src/ffi/v1.rs`, `src/ffi/mod.rs`, `src/ffi/host.rs`)

- **Capabilities:** `PRESENT`, `PHASES`, `AUDIO_PEEK`, `AXES`, `SETTINGS`, `SNAPSHOT`, `FRAME_SERIAL`, `ROW_REPEAT`, `BATTERY_DIRTY`, `ROM_PATCHES`, `SETTING_NOTES`, `DEBUG`, `DEBUG_REGISTERS`, `TELEMETRY`, `PRESENCE`, `DEBUG_TRANSLATE`.
- **Debug memory spaces:** `0` = `RDRAM`, `1` = `DMEM`, `2` = `IMEM`, `3` = `PIFRAM`, `4` = `ROM` (read-only), `5` = `CPU` (32-bit kernel virtual view via TLB without faulting).
- **Debug processors & units:** Processor `0` (`CPU`, 32-bit PC, `PC` + 32 GPRs + `HI` + `LO` + `Cycles`), Processor `1` (`RSP`, 12-bit PC, `PC` + `Halted` + 32 GPRs), Unit `VI` (14 × 32-bit registers).
- **User settings:** `ThreadedRdp` (default `true`), `RdpWorkers` (`1..8`, default `clamp(cores/3, 1, 4)`), `DeferredPresentation` (`true`), `SkipRepeatedScans` (`true`), `RenderScale` (`1..4`, default `1`), `Antialiasing` (`Off`/`2x`/`3x`/`4x`, default `Off`, combined product with `RenderScale` clamped to `4`), `Gpu` (`false`), `ExpansionPak` (`true`, create-scope), `Recompiler` (`true`), plus hidden test switches (`IdleSkip`, `RspWhole`, `VerifyBlocks`, `BlockTier`, `VerifyRdp`, `Boot`).
- **Controller mapping (`n64.pad`):** 4 ports, 14 digital buttons (4 C-buttons also driven from right stick axes 2/3 at threshold `±0.5`), analog stick axes 0/1 scaled by `127.0` (Y inverted) with round-ties-even.

### 8.3 Accuracy Boundaries

The following hardware behaviors are intentionally simplified or unmodeled in `marsrt`:
- **No VR4300 instruction or data cache simulation:** `CACHE` validates address alignment and TLB translation only; all memory accesses are coherent with RDRAM immediately, and instruction fetches read live RDRAM.
- **Simplified CPU pipeline timing:** Instructions execute in 1 cycle plus fixed multiply/divide stalls (`5`/`8`/`37`/`69`); RDRAM latency, TLB/cache misses, and pipeline interlocks are not modeled.
- **Instantaneous PI and SP DMA transfers:** PI and SP DMAs copy all bytes atomically on the triggering register write and assert completion immediately (unlike SI DMA, which delays completion by `4,608` cycles).
- **PIF boot HLE:** PIF IPL1/IPL2 ROM (`0x1FC0_0000`) is not executed; `machine::hand_off` establishes post-IPL2 state directly, with full CIC-6105 challenge/response emulation at runtime.
- **Stubbed RI and RDRAM control registers:** `RI_SELECT` (`0x0470_000C`) holds `0x14` and RDRAM registers (`0x03F0_0000`) return a fixed post-IPL3 table; dynamic RDRAM refresh/timing calibration is not modeled.
- **No 64DD hardware:** Cartridge Domain 2 Address 1 (`0x0500_0000..0x07FF_FFFF`) has no disk drive attached.
