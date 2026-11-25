# Mars — Reality Signal Processor (RSP)

Live Rust core crate: `EmuSen/Cores/Nintendo/MarsRT - N64` (crate `marsrt`).

---

## 1. Overview and Local Memory (`src/memory/sp.rs`, `src/rsp/mod.rs`)

The Reality Signal Processor (RSP) inside the Reality Coprocessor (RCP) pairs a 32-bit MIPS-subset **Scalar Unit (SU)** with a 128-bit **Vector Unit (VU, COP2)** running over two private 4 KiB memories:

| Memory | Size | RSP Address Mask | CPU Physical Base | CPU Mirroring |
| --- | --- | --- | --- | --- |
| **DMEM** (Data) | `4,096` B (`0x1000`) | `0xFFF` (`DATA_MASK`) | `0x0400_0000` | Alternates with IMEM every 8 KiB across `0x0400_0000..0x0403_FFFF` |
| **IMEM** (Instruction) | `4,096` B (`0x1000`) | `0xFFC` (`PC_MASK`) | `0x0400_1000` | Alternates with DMEM every 8 KiB across `0x0400_0000..0x0403_FFFF` |

- Both memories are big-endian.
- On the CPU bus (`src/memory/bus_access.rs`), DMEM and IMEM are **whole-word latching** windows: any CPU store (`SB`, `SH`, `SW`, `SD`) writes a full 32-bit word into DMEM/IMEM.
- All RSP instruction fetches mask `pc` to `0xFFC`; all RSP scalar and vector data addresses mask byte offsets to `0xFFF`, wrapping seamlessly from `0xFFF` back to `0x000`.
- At power-on, the RSP starts halted (`halted = true`, `broke = false`, `pc = 0`, `next_pc = 0`).

---

## 2. SP Interface Registers and DMA (`src/memory/sp.rs`)

Mapped on the CPU bus at `0x0404_0000..0x0404_001F` (and `SP_PC` at `0x0408_0000..0x0408_0007`), and reachable from the RSP itself via `MFC0` / `MTC0` registers `0..7` (COP0 registers `8..15` forward to DP command registers `0x0410_0000..0x0410_001C`):

| CPU Address | Offset | RSP COP0 | Register | Read | Write |
| --- | --- | --- | --- | --- | --- |
| `0x0404_0000` | `0x00` | `$c0` | `SP_MEM_ADDR` | `mem_address` | `value & 0x1FF8` (bit 12 selects IMEM vs DMEM; 8-byte aligned) |
| `0x0404_0004` | `0x04` | `$c1` | `SP_DRAM_ADDR` | `dram_address` | `value & 0x00FF_FFF8` (8-byte aligned) |
| `0x0404_0008` | `0x08` | `$c2` | `SP_RD_LEN` | `0xFF8` | Start RDRAM -> SP DMA |
| `0x0404_000C` | `0x0C` | `$c3` | `SP_WR_LEN` | `0xFF8` | Start SP -> RDRAM DMA |
| `0x0404_0010` | `0x10` | `$c4` | `SP_STATUS` | Assembled status word | Clear/set bit pairs (see below) |
| `0x0404_0014` | `0x14` | `$c5` | `SP_DMA_FULL` | `0` | Ignored |
| `0x0404_0018` | `0x18` | `$c6` | `SP_DMA_BUSY` | `0` | Ignored |
| `0x0404_001C` | `0x1C` | `$c7` | `SP_SEMAPHORE` | Returns old `semaphore` (`0`/`1`), then sets `semaphore = true` | Any write clears `semaphore = false` |
| `0x0408_0000` | — | — | `SP_PC` | `processor.pc` (`0x000..0xFFC`) | Sets `pc = value & 0xFFC`, `next_pc = (pc + 4) & 0xFFC` |

### 2.1 `SP_STATUS` Read and Write Bitfields

**Read layout (`status_word`):**

| Bit(s) | Flag |
| --- | --- |
| `0` (`0x0001`) | `HALT` (`processor.halted`) |
| `1` (`0x0002`) | `BROKE` (`processor.broke`) |
| `5` (`0x0020`) | `SINGLE_STEP` (`single_step`) |
| `6` (`0x0040`) | `INTR_ON_BREAK` (`interrupt_on_break`) |
| `7..14` | `SIG0..SIG7` (`signals << 7`) |

**Write layout (`sp_write_status`):** Clear/set pairs obey the rule `asks(value, clear_bit, set)`: if **both** the clear and set bits of a pair are `1` in the same write, the pair is ignored as a no-op.

| Clear Bit | Set Bit | Effect |
| --- | --- | --- |
| `0` (`0x0001`) | `1` (`0x0002`) | Clear / set `HALT` (clearing `HALT` calls `start(pc)`, setting `next_pc = (pc + 4) & 0xFFC`) |
| `2` (`0x0004`) | — | Clear `BROKE` |
| `3` (`0x0008`) | `4` (`0x0010`) | Clear / raise `MI_INTR_SP` |
| `5` (`0x0020`) | `6` (`0x0040`) | Clear / set `SINGLE_STEP` |
| `7` (`0x0080`) | `8` (`0x0100`) | Clear / set `INTR_ON_BREAK` |
| `9 + 2*k` | `10 + 2*k` | Clear / set `SIGk` (`k = 0..7`, bits `9..24`) |

### 2.2 2D Rectangular SP DMA (`sp_transfer`)

Writing `SP_RD_LEN` (RDRAM -> SP) or `SP_WR_LEN` (SP -> RDRAM) encodes:
- `length = ((value & 0xFFF) | 7) + 1` (bytes per row, rounded up to a multiple of 8).
- `rows = ((value >> 12) & 0xFF) + 1` (`1..256` rows).
- `skip = (value >> 20) & 0xFFF` (byte stride added to `dram_address` after each row).

Transfers select IMEM (`mem_address & 0x1000 != 0`) or DMEM (`0`), starting at `bank_offset = mem_address & 0xFF8`. Within a row, addresses reaching `0x1000` wrap back to `0x000` of the same 4 KiB bank. After all rows complete, `mem_address` retains its bank bit with the final wrapped `bank_offset`, and `dram_address` advances by `rows * (length + skip)`. Before reading or writing RDRAM, SP DMA synchronizes with the threaded RDP page marks (`site::SP_DMA`).

---

## 3. Scalar Unit (MIPS Subset, `src/rsp/mod.rs`)

- **Registers:** 32 × 32-bit GPRs (`gpr[0..32]`, `r0` hardwired to zero), 12-bit `pc` and `next_pc`.
- **Supported scalar instructions:**
  - **Shifts:** `SLL`, `SRL`, `SRA`, `SLLV`, `SRLV`, `SRAV` (shift counts masked to 5 bits).
  - **Arithmetic & logic:** `ADD`/`ADDU` (identical, no overflow trap), `SUB`/`SUBU` (identical, no overflow trap), `ADDI`/`ADDIU` (identical), `AND`/`ANDI`, `OR`/`ORI`, `XOR`/`XORI`, `NOR`, `SLT`/`SLTI`, `SLTU`/`SLTIU`, `LUI`.
  - **Branches & jumps (1 delay slot, wrapped to `0xFFC`):** `J`, `JAL`, `JR`, `JALR`, `BEQ`, `BNE`, `BLEZ`, `BGTZ`, `BLTZ`, `BGEZ`, `BLTZAL`, `BGEZAL`. Link writes `next_pc` (`pc + 8` masked to `0xFFC`) to `r31` (or `rd` for `JALR`).
  - **DMEM loads & stores (unaligned allowed, big-endian, masked to `0xFFF`):** `LB`, `LH`, `LW`/`LWU`, `LBU`, `LHU`, `SB`, `SH`, `SW`.
  - **System events (`rsp_event` in `src/memory/sp.rs`):**
    - `MFC0` / `MTC0` (`op = 0x10`, `rs = 0x00 / 0x04`): register `rd & 0x0F` reads/writes SP (`0..7`) or DP (`8..15`) registers.
    - `BREAK` (`op = 0x00, funct = 0x0D`): sets `halted = true` and `broke = true`; if `!was_broke && interrupt_on_break`, raises `MI_INTR_SP`.
- **Omitted MIPS features (matching hardware):** No exceptions, no overflow traps, no `MULT`/`DIV`/`HI`/`LO`, no branch-likely instructions, no COP1 FPU, no 64-bit operations. Unimplemented opcodes execute as no-ops.

---

## 4. Vector Unit — COP2 (`src/rsp/mod.rs`, `src/rsp/simd.rs`)

### 4.1 Registers, Accumulator, Flags, and Hidden State

| State | Width / Shape | Details |
| --- | --- | --- |
| Vector registers `v0..v31` | `32 × 128-bit` (`[[u16; 8]; 32]`) | 8 big-endian 16-bit elements per register; byte 0 is the high byte of element 0 |
| Accumulator | `8 × 48-bit` | Stored in three thirds `accumulator[0..3][0..8]` (high/mid/low `u16`s); wraps modulo `2^48`, sign-extended when read |
| `VCO` | `16-bit` | Low byte (`7:0`): carry/borrow (or `VCH` sign-differ); High byte (`15:8`): not-equal |
| `VCC` | `16-bit` | Low byte (`7:0`): `<=` / compare result; High byte (`15:8`): `>=` (clip high) |
| `VCE` | `8-bit` | Clip extension bit per lane |
| `divide_input`, `divide_output`, `divide_input_loaded` | `u16`, `u16`, `bool` | Hidden 32-bit reciprocal/RSQRT state |

### 4.2 Element Broadcast Selector (`vt` Shuffle)

Every computational COP2 instruction (`op = 0x12`, bit `25 = 1`) carries a 4-bit selector `e` in bits `24:21`, `vt` in `20:16`, `vs` in `15:11`, `vd` in `10:6`, and `funct` in `5:0`. All source lanes (`vs`, broadcast `vt`, and `vd`) are snapshotted before any result lane is written back to `vd`.

| Selector `e` | `vt` Element Paired with Lane `i` (`0..7`) | Broadcast Mode |
| --- | --- | --- |
| `0`, `1` | `i` | Whole vector (no broadcast) |
| `2`, `3` | `(e - 2) + (i & 6)` | Quarter (`0q`, `1q`) |
| `4..7` | `(e - 4) + (i & 4)` | Half (`0h..3h`) |
| `8..15` | `e - 8` | Scalar lane (`0..7`) |

### 4.3 Scalar-Vector Transfers (`op = 0x12`, bit `25 = 0`)

- **`MFC2` (`rs = 0x00`):** Reads two consecutive bytes at byte index `e = (inst >> 7) & 0xF` and `(e + 1) & 0xF` from `vr[rd]` and sign-extends to 32 bits into `gpr[rt]`. At `e = 15`, reads byte 15 (high) and wraps to byte 0 (low).
- **`MTC2` (`rs = 0x04`):** Writes `(gpr[rt] >> 8)` to byte `e` of `vr[rd]`, and writes `gpr[rt] as u8` to byte `e + 1` **only if `e < 15`** (at `e = 15`, the low byte is dropped with no wrap).
- **`CFC2` (`rs = 0x02`) / `CTC2` (`rs = 0x06`):** Selects control register by `rd & 3`: `0` = `VCO`, `1` = `VCC`, `2` or `3` = `VCE`. `CFC2` sign-extends `VCO` and `VCC` from 16 bits and zero-extends `VCE` from 8 bits.

### 4.4 Vector Loads (`LWC2`, `0x32`) and Stores (`SWC2`, `0x3A`)

Encoded with 5-bit `format` (`15:11`, valid `0..11`), 4-bit byte index `e` (`10:7`), and 7-bit signed `offset` (`6:0`) scaled by `TRANSFER_SCALES` nibble (`0x4444_3344_3210`, i.e. shifts `0, 1, 2, 3, 4, 4, 3, 3, 4, 4, 4, 4` for formats `0..11`), then added to `gpr[rs]` and masked with `0xFFF`:

| Format | Scale | Load (`LWC2`) | Store (`SWC2`) | Register Wrap & Behavior |
| --- | --- | --- | --- | --- |
| `0` | `<< 0` (1 B) | `LBV` (1 B) | `SBV` (1 B) | Load stops at byte 15; store wraps modulo 16 |
| `1` | `<< 1` (2 B) | `LSV` (2 B) | `SSV` (2 B) | Load stops at byte 15; store wraps modulo 16 |
| `2` | `<< 2` (4 B) | `LLV` (4 B) | `SLV` (4 B) | Load stops at byte 15; store wraps modulo 16 |
| `3` | `<< 3` (8 B) | `LDV` (8 B) | `SDV` (8 B) | Load stops at byte 15; store wraps modulo 16 |
| `4` | `<< 4` (16 B) | `LQV` | `SQV` | Transfers `16 - (addr & 0xF)` bytes to end of 16 B boundary; load stops at byte 15, store wraps |
| `5` | `<< 4` (16 B) | `LRV` | `SRV` | Transfers `(addr & 0xF)` bytes from start of 16 B boundary; load stops at byte 15, store wraps |
| `6` | `<< 3` (8 B) | `LPV` | `SPV` | 8 packed bytes (`<< 8` load; store shifts `>> 8` when `(e + i) & 8 == 0`, else `>> 7`) |
| `7` | `<< 3` (8 B) | `LUV` | `SUV` | 8 unsigned bytes (`<< 7` load; store shifts `>> 7` when `(e + i) & 8 == 0`, else `>> 8`) |
| `8` | `<< 4` (16 B) | `LHV` | `SHV` | 8 bytes strided by 2 across 16 B window (`<< 7` load; `>> 7` store across byte pairs) |
| `9` | `<< 4` (16 B) | `LFV` | `SFV` | Load uses fixed offset pattern `[e, 4-e, 8-e, 12-e, 8-e, 12-e, -e, 4-e]` (`<< 7`), writes `min(8, 16-e)` bytes from `e`; store writes 4 bytes (`>> 7`) for selectors `{0,1,4,5,8,11,12,15}` or four `0`s otherwise |
| `10` | `<< 4` (16 B) | No-op | `SWV` | Store writes all 16 bytes rotated within 16 B window aligned to `addr & !7` |
| `11` | `<< 4` (16 B) | `LTV` | `STV` | Transposes across 8-register group `vt & !7` (1 element per register) with bit-3 address rotation |

### 4.5 Clamping Rules and Vector Computational Operations

Three clamping helpers govern VU results from the 48-bit signed accumulator `acc`:
- **Signed clamp (`clamp_signed(acc >> 16)`):** Saturates bits `47:16` to `[-32768, 32767]` (`0x8000..0x7FFF`).
- **Unsigned clamp (`clamp_unsigned(acc)`):** Returns `0` if `acc < 0`, `0xFFFF` if `acc > 0x7FFF_FFFF`, else `(acc >> 16) as u16`.
- **Low clamp (`clamp_low(acc)`):** If `(acc >> 16) < -32768` returns `0`; if `(acc >> 16) > 32767` returns `0xFFFF`; else returns `acc as u16` (low 16 bits).

| Funct | Mnemonic | Accumulator Write | Result / Flags |
| --- | --- | --- | --- |
| `0x00` / `0x08` | `VMULF` / `VMACF` | Full 48-bit: `(base) + ((s_i16 * t_i16) << 1)`, `base = 0x8000` (`VMULF`) or `acc` (`VMACF`) | `clamp_signed(acc >> 16)` |
| `0x01` / `0x09` | `VMULU` / `VMACU` | Full 48-bit: same as `VMULF`/`VMACF` | `clamp_unsigned(acc)` |
| `0x02` / `0x0A` | `VRNDP` / `VRNDN` | Full 48-bit: if `(acc >= 0) == positive`, `acc += t_i16 << (16 if vs & 1 else 0)` | `clamp_signed(acc >> 16)` |
| `0x03` | `VMULQ` | Full 48-bit: `(s_i16 * t_i16 << 16) + (0x1F_0000 if product < 0)` | `clamp_signed(acc >> 17) & 0xFFF0` |
| `0x04` / `0x0C` | `VMUDL` / `VMADL` | Full 48-bit: `base + ((s_u32 * t_u32) >> 16)` | `clamp_low(acc)` |
| `0x05` / `0x0D` | `VMUDM` / `VMADM` | Full 48-bit: `base + (s_i16 * t_u16)` | `clamp_signed(acc >> 16)` |
| `0x06` / `0x0E` | `VMUDN` / `VMADN` | Full 48-bit: `base + (s_u16 * t_i16)` | `clamp_low(acc)` |
| `0x07` / `0x0F` | `VMUDH` / `VMADH` | Full 48-bit: `base + ((s_i16 * t_i16) << 16)` | `clamp_signed(acc >> 16)` |
| `0x0B` | `VMACQ` | Full 48-bit: odd-bit-21 rounding adjustment (`±0x20_0000`) | `clamp_signed(acc >> 17) & 0xFFF0` |
| `0x10` / `0x11` | `VADD` / `VSUB` | Low 16 bits (unclamped sum/diff) | `clamp_signed(s ± t ± VCO.carry)`; clears `VCO` |
| `0x13` | `VABS` | Low 16 bits (`-t`, `0`, or `t` by sign of `s`, unclamped) | Same, clamping `s < 0 && t == 0x8000` to `0x7FFF` |
| `0x14` | `VADDC` | Low 16 bits (`s + t`) | `s + t`; sets `VCO[7:0]` carry on `> 0xFFFF`, clears `VCO[15:8]` |
| `0x15` | `VSUBC` | Low 16 bits (`s - t`) | `s - t`; sets `VCO[7:0]` borrow (`< 0`) and `VCO[15:8]` not-equal (`!= 0`) |
| `0x1D` | `VSAR` | Unchanged | Reads accumulator slice: `e=8` high (`>>32`), `e=9` mid (`>>16`), `e=10` low (`>>0`), else `0` |
| `0x20..0x23` | `VLT`, `VEQ`, `VNE`, `VGE` | Low 16 bits (`vd`) | Signed compare using `VCO` carry/not-equal tie-breakers; sets `VCC[7:0]`, clears `VCC[15:8]` and `VCO` |
| `0x24` | `VCL` | Low 16 bits (`vd`) | Low-half clip using `VCO`/`VCC`/`VCE`; updates `VCC`, clears `VCO` and `VCE` |
| `0x25` | `VCH` | Low 16 bits (`vd`) | High-half clip; sets `VCO`, `VCC`, and `VCE` (`sum == -1` on differing signs) |
| `0x26` | `VCR` | Low 16 bits (`vd`) | One's-complement clip; updates `VCC`, clears `VCO` and `VCE` |
| `0x27` | `VMRG` | Low 16 bits (`vd`) | Selects `s[i]` if `VCC[i]` set, else `t[i]`; clears `VCO` |
| `0x28..0x2D` | `VAND`, `VNAND`, `VOR`, `VNOR`, `VXOR`, `VNXOR` | Low 16 bits (`vd`) | Bitwise boolean operations |
| `0x30..0x32` | `VRCP`, `VRCPL`, `VRCPH` | Low 16 bits (all 8 lanes of broadcast `t`) | Single lane `vd[vs & 7]` from 512-entry reciprocal LUT (see §4.6) |
| `0x33` | `VMOV` | Low 16 bits (all 8 lanes of broadcast `t`) | Writes single lane `vd[vs & 7] = t[vs & 7]` |
| `0x34..0x36` | `VRSQ`, `VRSQL`, `VRSQH` | Low 16 bits (all 8 lanes of broadcast `t`) | Single lane `vd[vs & 7]` from 512-entry inverse-sqrt LUT (see §4.6) |
| `0x37`, `0x3F` | `VNOP`, `VNULL` | Unchanged | No-op (does not write `vd` or accumulator) |
| All other `funct` | Undocumented | Low 16 bits (`s + t`) | Writes `0` to `vd` |

### 4.6 Reciprocal and Inverse Square Root LUTs (`src/rsp/mod.rs::tables`, `evaluate`)

Both 512-entry `u16` tables are computed once at initialization (`TABLES`):
- **Reciprocal (`reciprocal[0..512]`):** `reciprocal[0] = 0xFFFF`; for `i = 1..511`, `(((1 << 34) / (i + 512)) + 1) >> 8`.
- **Inverse square root (`root[0..512]`):** For `i < 256`, `scale = i + 256`; for `i >= 256`, `scale = ((i - 256) << 1) + 512`; finds the largest `r` via binary search from `1 << 17` with `scale * r^2 < (1 << 44)`, storing `r >> 1`.

**Execution rules (`reciprocate`, `evaluate`):**
- `VRCPH` / `VRSQH`: writes previous `divide_output` to `vd[vs & 7]`, latches `divide_input = vt[e & 7]`, sets `divide_input_loaded = true`.
- `VRCPL` / `VRSQL`: if `divide_input_loaded` is true, forms 32-bit operand `(divide_input << 16) | vt[e & 7]`; otherwise sign-extends 16-bit `vt[e & 7]` to 32 bits. Clears `divide_input_loaded`.
- `VRCP` / `VRSQ`: always sign-extends 16-bit `vt[e & 7]` to 32 bits and clears `divide_input_loaded`.
- Special inputs: `0 -> 0x7FFF_FFFF`; `0xFFFF_8000 -> 0xFFFF_0000`; inputs `> 0xFFFF_8000` subtract `1` before sign/magnitude normalization. The low 16 bits go to `vd[vs & 7]` and the high 16 bits latch into `divide_output`.

---

## 5. Live Rust SIMD and Decoded Execution Architecture (`src/rsp/simd.rs`, `src/rsp/decoded.rs`)

### 5.1 Host SIMD Vector Path (`src/rsp/simd.rs`)

On `x86_64` hosts supporting **SSE2, SSSE3, and SSE4.1** (`simd_supported()`), the VU executes all 8 lanes in parallel by default (`EMUSEN_MARSRT_RSP_SIMD=0` disables it):
- **Accumulator layout:** `Rsp::accumulator` is stored as `[[u16; 8]; 3]` (high, middle, low thirds), matching three `__m128i` registers directly without transposing per instruction; `add48` computes exact 48-bit wrapping sums across thirds with SSE2 unsigned-saturation carry detection.
- **Broadcast:** Precomputed 16-entry `SELECT` table drives `_mm_shuffle_epi8` (`pshufb`) for selectors `2..15` (skipped entirely for whole-vector selectors `0..1`).
- **SIMD loads/stores:** `LBV`/`LSV`/`LLV`/`LDV`/`LQV` (`format <= 4`) moving even-aligned whole elements within DMEM bounds execute as a single 16-byte big-endian swap (`pshufb`) and `_mm_blendv_epi8`; odd, wrapping, or specialized formats delegate to the scalar reference path.
- **Single-lane ops:** `VRCP*`, `VRSQ*`, and `VMOV` call the shared scalar LUT helpers while still writing the broadcast `t` lanes into the low accumulator third.

### 5.2 Decoded IMEM Table (`src/rsp/decoded.rs`)

Enabled by default (`EMUSEN_MARSRT_RSP_BLOCKS=0` disables it), `Decoded` caches one 16-byte-aligned `Entry` per IMEM word (`WORDS = 1024`):
- Each entry holds a monomorphized function pointer (`Handler`), the raw host-endian 32-bit IMEM word (`raw`), and flags (`EVENT`, `BRANCH`, and straight-run length up to `LONGEST = 64`).
- Handlers specialize away no-op writes to `r0` and split SIMD vector ops into `VECTOR_WHOLE` (no shuffle) vs `VECTOR_SHUFFLED` tables at decode time.
- **Self-modifying / DMA safety:** Every instruction checks `e.raw == raw_at(imem, at)` before dispatch; any mismatch triggers `redecode`, updating the entry and recomputing straight-run lengths for up to 64 preceding words.
- **Straight-run batching (`straight`):** Outside delay slots (`next_pc == (pc + 4) & 0xFFC`), pure non-branch/non-event runs execute in a tight loop checking only `e.raw`, updating `pc` and `next_pc` once at the end of the run.
- **Machine integration (`src/memory/sp.rs`):** `MemoryBus::sp_step` runs `rsp_step_core` / `rsp_run_core` up to the cycle budget or next event (`MFC0`/`MTC0`/`BREAK`), handing only events back to `rsp_event`. During VR4300 idle-loop skipping (`src/cpu/idle.rs`), `rsp_run_to_event` executes the RSP in bulk up to the next scheduled system event or RSP event. When debugger RSP coverage (`cov rsp`) is armed (`sp.trace`), execution switches to `run_traced` to record every executed PC bit.

### 5.3 Accuracy Boundaries

- **1:1 cycle stepping with VR4300:** Each VR4300 cycle steps one RSP instruction while unhalted (hardware runs the RSP at 62.5 MHz, 2/3 of the 93.75 MHz VR4300 clock, with dual-issue scalar + vector pairing).
- **No dual-issue / vector pipeline hazards:** Scalar and vector instructions execute strictly sequentially in program order with zero pipeline latency; hazard-dependent microcode timing is not modeled.
- **Instantaneous SP DMA:** `SP_RD_LEN`/`SP_WR_LEN` complete atomically on register write; `SP_DMA_BUSY` and `SP_DMA_FULL` always read `0`.
