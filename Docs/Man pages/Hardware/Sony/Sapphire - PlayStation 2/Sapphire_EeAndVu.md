# Sapphire: Emotion Engine, Vector Units, Recompilers, and Scheduler

Hardware reference for the Emotion Engine (EE), Vector Units (VU0 and VU1), floating-point arithmetic, dynamic recompilers, event-driven board scheduler, and live Two Faces execution split in the `sapphire` PlayStation 2 core.

---

## 1. Emotion Engine (R5900)

The Emotion Engine core (`src/ee/`) implements the MIPS III subset plus Sony's R5900 extensions, decoded by `emusen_mips::r5900` and executed through `Ee` (`src/ee/mod.rs`).

### 1.1 Architectural State

| Resource | Geometry | Notes |
| --- | --- | --- |
| GPRs (`gpr`) | 32 × 128-bit | `r0` hardwired to zero; scalar MIPS ops read/write the low 64 bits; `LQ`/`SQ`, `QMFC2`/`QMTC2`, and MMI operate on all 128 bits |
| Multiply/Divide (`hi`, `lo`) | 2 × 128-bit | Dual integer pipelines: `HI`/`LO` (pipeline 0, low 64 bits) and `HI1`/`LO1` (pipeline 1, upper 64 bits) |
| Shift Amount (`sa`) | 64-bit | Byte/bit funnel shift amount for `QFSRV`, accessed by `MFSA`/`MTSA`/`MTSAB`/`MTSAH` |
| FPU (COP1) | 32 × 32-bit `fpr`, 32-bit `acc`, `fcr31` | Single-precision only; `FCR0 = 0x2E30` (implementation `0x2E`, revision `0x30`) |
| Scratchpad (`spram`) | 16 KiB (`16 << 10`) | Direct zero-wait local RAM mapped via TLB entries with the `S` bit set |

### 1.2 Multimedia Instructions (MMI)

`src/ee/mmi.rs` implements the R5900 128-bit parallel multimedia instructions across 8-bit, 16-bit, 32-bit, and 64-bit lanes:

- Parallel arithmetic: wrapping (`PADDB`/`H`/`W`, `PSUBB`/`H`/`W`, `PADSBH`), signed-saturating (`PADDSB`/`H`/`W`, `PSUBSB`/`H`/`W`), and unsigned-saturating (`PADDUB`/`H`/`W`, `PSUBUB`/`H`/`W`).
- Parallel compare, extremum, and magnitude: `PCEQB`/`H`/`W`, `PCGTB`/`H`/`W`, `PMAXH`/`W`, `PMINH`/`W`, `PABSH`/`W`.
- Parallel shifts, rotates, packs, interleaves, and funnel shift: `PSLLH`/`W`, `PSRLH`/`W`, `PSRAH`/`W`, `PPACB`/`H`/`W`, `PEXTLB`/`H`/`W`, `PEXTUB`/`H`/`W`, `PEXT5`, `PPAC5`, `PINTH`, `PINTEH`, `PEXEH`/`W`, `PEXCH`/`W`, `PROT3W`, `PREVH`, `PCPYH`, `PCPYLD`, `PCPYUD`, `QFSRV`, `PLZCW`.
- Parallel multiply/divide using both 64-bit `HI`/`LO` halves (`PMULTW`, `PMULTUW`, `PMULTH`, `PMADDW`, `PMADDUW`, `PMADDH`, `PMSUBH`, `PDIVW`, `PDIVUW`, `PDIVBW`, `PMFHI`, `PMFLO`, `PMTHI`, `PMTLO`, `PMFHL.*`, `PMTHL.LW`).

### 1.3 COP0, Exceptions, Counters, and Breakpoints

`src/ee/cop0.rs` defines the 32 COP0 registers, writable masks, performance counters, and hardware debug registers:

- **Identity & Reset**: `PRId = 0x2E00` (implementation `0x2E`). On reset (`Ee::reset()`), `PC = 0xBFC0_0000`, `Status = ERL | BEV` (`0x0040_0004`), `Config = SIZES | 2` (`IC = 16 KiB`, `DC = 8 KiB`, `kseg0` uncached, caches disabled), `Random = 47`, `Wired = 0`.
- **Exception Levels**:
  - **Level 1 (`Status.EXL`)**: `Interrupt` (0), `TlbModified` (1), `TlbRefill`/`TlbInvalid` load/store (2/3), `AddressLoad`/`AddressStore` (4/5), `BusInstruction`/`BusData` (6/7), `Syscall` (8), `Break` (9), `Reserved` (10), `Unusable` (11), `Overflow` (12), `Trap` (13). `Status.BEM` masks subsequent bus errors after latching `BadPAddr` (16-byte aligned).
  - **Level 2 (`Status.ERL`)**: Reset/NMI, Performance Counter (`Level2::Counter`), and Debug (`Level2::Debug`, `Cause.EXC2 = 3`).
- **Interrupts & Delay**: External `Int0` (EE INTC -> `Cause.IP2`), `Int1` (EE DMAC -> `Cause.IP3`), and `Count == Compare` (`Cause.IP7`). External interrupts recognised on `Int0`/`Int1` complete `INTERRUPT_DELAY = 2` instructions before raising the exception. `BC0F`/`BC0T` sample the DMAC `CPCOND0` line.
- **Performance Counters (`COP0` reg 25)**: `PCCR` (`CTE` bit 31) and two 31-bit counters `PCR0`/`PCR1` (bit 31 `OVFL`), counting cycles, single/dual issue, branches, TLB misses, I-cache/D-cache misses, DTLB accesses, non-blocking loads, completed instructions, COP1/COP2 completions, and loads/stores.
- **Hardware Breakpoints (`COP0` reg 24)**: `BPC` (enables `IAE`, `DRE`, `DWE`, `DVE`, mode gates, global disable `BED`, status flags `IAB`/`DRB`/`DWB`), `IAB`/`IABM`, `DAB`/`DABM`, and `DVB`/`DVBM`.

### 1.4 TLB, Caches, and Scratchpad

- **TLB (`src/ee/tlb.rs`)**: 48 fully associative even/odd page-pair entries (`Entry { mask, hi, lo0, lo1 }`), supporting 7 page sizes (4 KiB to 16 MiB via `PageMask`), 8-bit `ASID`, global bit `G` (AND of `EntryLo0.G` and `EntryLo1.G`), and scratchpad bit `S` (`EntryLo0` bit 31). When `S = 1`, the entry matches above 16 KiB (`0xFFFF_C000`), maps directly into `Ee::spram` (`vaddr & 0x3FFF`), and `TLBR` reads `C = 2`. A single-entry pure lookup memo caches the last 4 KiB page and ASID until a TLB write invalidates it.
- **Memory Modes (`src/ee/memory.rs`)**: `kseg0` (`0x8000_0000..0x9FFF_FFFF`) uses `Config.K0` (`0` write-through `Through`, `3` write-back/write-allocate `Cached`, `7` `Accelerated`, else `Uncached`); `kseg1` (`0xA000_0000..0xBFFF_FFFF`) is always `Uncached`; under `Status.ERL = 1`, `kuseg` maps 1:1 uncached. Clearing `Config.DCE` forces cached data accesses to `Uncached`.
- **Caches (`src/ee/cache.rs`)**:
  - **Instruction Cache (`ICache`)**: 16 KiB, 2-way set associative, 128 sets × 64-byte lines, LRF (Least Recently Filled, XOR of way LRF bits) replacement preferring an invalid way first, with per-word steering/BHT side storage for `CACHE IXLDT`/`IXSDT`.
  - **Data Cache (`DCache`)**: 8 KiB, 2-way set associative, 64 sets × 64-byte lines, write-back / write-allocate, per-way lock bit `L` (locked lines are kept clean and skipped as victims), LRF replacement.
  - **Uncached Accelerated Buffer (`Ucab`)**: One 128-byte direct-mapped line tagged by `paddr >> 7`; invalidated by any EE store.
  - **BTAC (`Btac`)**: 64-entry storage backing `CACHE BXLBT`/`BXSBT` and invalidations (not used for speculative prediction).

### 1.5 Superscalar Timing Model

`src/ee/timing.rs` models the R5900 in-order two-way superscalar issue unit across six physical pipes (`I0`, `I1`, `Ls`, `Br`, `C1`, `C2`) and 15 instruction categories paired by the EE Core dual-issue compatibility table (`Pair::O`, `Pair::X`, `Pair::Y` with 1-cycle stall).

| Operation / Event | Latency (EE cycles) | Throughput / Cost |
| --- | --- | --- |
| ALU / Shift / Logical / Cond. Move / Load (hit) | 1 | 1 (Store latency 0) |
| `MULT`/`MADD` (pipe 0 or 1), Multimedia Multiply | 4 | 2 |
| `DIV`/`DIVU` (pipe 0 or 1), Multimedia Divide | 37 | 37 |
| COP1 Add/Sub/Mul/MADD/CVT/Move | 4 | 1 (`MTC1`/`MFC1`/`LWC1`: 2) |
| COP1 `DIV.S` / `SQRT.S` | 8 | 7 |
| COP1 `RSQRT.S` | 14 | 13 |
| COP2 FMAC / IALU | 4 / 1 | 1 / 1 |
| COP2 `VDIV`/`VSQRT` / `VRSQRT` | 7 / 13 | 7 / 13 |
| Cache line refill (`REFILL`) / writeback (`WRITEBACK`) | 8 | 4 bus cycles |
| Uncached transfer (`UNCACHED`) / UCAB refill (`UCAB_REFILL`) | 2 / 16 | 1 / 8 bus cycles |

---

## 2. Floating-Point Format (`float.rs` and `float_rz.rs`)

Both the EE FPU (`src/ee/fpu.rs`) and the Vector Units (`src/vu/ops.rs`, `src/vu/efu.rs`) share the PlayStation 2 single-precision format implemented in `src/float.rs`.

### 2.1 PS2 Single-Precision Rules (`src/float.rs`)

- **No NaN, Infinity, or Denormals**: An 8-bit exponent field of `0` is treated as zero regardless of fraction bits (flushed to signed zero on output). An exponent field of `255` is an ordinary exponent; overflow saturates to `±MAX` (`0x7FFF_FFFF` / `0xFFFF_FFFF`).
- **Addition / Subtraction (`add`, `sub`)**: Aligns the smaller magnitude's 24-bit significand to the larger operand plus `GUARD = 1` bit, drops bits below that guard position, adds/subtracts, and truncates towards zero. Two zero inputs yield `-0` only when both inputs are negative zero.
- **Multiplication (`mul`)**: Recodes the second operand (`b`, i.e., `FPR[ft]` on COP1) in radix-4 Booth form and omits partial-product columns below `DROPPED_COLUMNS = 15`, then truncates. Because only `b` is Booth-recoded, multiplication is non-commutative in its least significant bit. A closed-form integer fast path (`product`) computes `x * y` and subtracts `loss(x, y)` only when `y & READ != 0` and `(x * y) & 0x7F_FFFF < MOST_LOST`.
- **Multiply-Add (`fused`)**: Computes `mul(a, b)` first; if the product overflows, returns signed `MAX` with overflow; if `exponent(acc) == 255`, preserves `acc` with overflow; otherwise calls `add(acc, term)`. Reports both sum and product overflow/underflow so COP1 `FCR31.SU` and VU sticky flags track multiply underflow independently.
- **Division, Square Root, and RSQRT**: `div` and `sqrt` round to nearest (ties to even), delegating to host IEEE-754 `f32` (`host::div`, `host::sqrt`) whenever inputs have exponents `1..=254` and the quotient remains in `2..=254`, falling back to exact integer arithmetic otherwise. `rsqrt(a, b)` divides `a` by `sqrt(b)` and truncates towards zero, using `a`'s sign alone. Division by zero produces signed `MAX` with `invalid` for `0/0` and `divide` otherwise.
- **Ordering & Conversions**: Comparisons (`order`) treat all zero-exponent values as equal; `MAX.S`/`MIN.S` (`raw_order`) compare raw sign-magnitude bits so `-0 < +0` and zero-exponent values with nonzero fractions sort above zero. `to_word` / `to_fixed` (`FTOI0`/`4`/`12`/`15`) truncate towards zero and clamp to `0x7FFF_FFFF` / `0x8000_0000` when the adjusted exponent exceeds `0x9D`.

### 2.2 Flush-to-Zero / Round-to-Zero SIMD Kernel (`src/float_rz.rs`)

Compiled under the optional `lanes` Cargo feature (`--features lanes`), `src/float_rz.rs` provides an AVX2 8-lane kernel that reproduces `float::add`, `float::mul`, `float::fused`, and VU MAC flags bit-for-bit using host x86-64 floating-point instructions under explicit MXCSR configuration:

- **`MODE = 0x1F80 | 0x6000 | 0x0040 | 0x8000` (`0x9FC0`)**: Masks all SIMD exceptions (`0x1F80`), selects round-towards-zero (`RC = 11`, `0x6000`), enables Denormals-Are-Zero (`DAZ`, `0x0040`), and enables Flush-To-Zero (`FTZ`, `0x8000`). `enter()` / `leave(old)` swap MXCSR via `stmxcsr`/`ldmxcsr`.
- **Guard-Bit Alignment in `raw::add`**: Trims the smaller operand at the larger operand's guard bit using a signed magic power-of-two constant `c` (`exp(large) - 1` with `sign(small)`): `(s + c) - c` under round-towards-zero discards bits below the guard bit before adding `l`.
- **Booth Correction in `raw::mul`**: Computes the host `_mm256_mul_ps` product and integer low-product check in parallel; short multiplicands apply a branchless `-1 ULP` adjustment via negative-digit counts, and rare lanes within `MOST_LOST` call `float::mul` (`raw::fixed`).
- **Exponent-255 Fallback**: Any active lane whose result has `|r| > 0x7F7F_FFFE` (exponent field `255` or host overflow) triggers `trouble` and delegates the vector to the integer SIMD fallback (`src/float_lanes.rs`), preserving exact PS2 bits and MAC flags (`Z`, `S`, `U`, `O`).

---

## 3. Vector Units (VU0 and VU1)

The Vector Processing Units (`src/vu/`) implement both vector units (`Vu`, `Vpu`), shared instruction execution (`src/vu/ops.rs`), VU1's Elementary Function Unit (`src/vu/efu.rs`), and cycle-accurate micro-mode pipelines (`src/vu/pipe.rs`).

### 3.1 Architecture and Memory Map

| Unit | Micro Memory | Data Memory | EE Physical Window (`0x1100_0000..0x1101_0000`) | Special Features |
| --- | --- | --- | --- | --- |
| **VU0** | 4 KiB (`512` 64-bit pairs) | 4 KiB (`256` qwords) | `0x1100_0000..0x1100_3FFF` (micro, mirrored), `0x1100_4000..0x1100_7FFF` (data, mirrored) | COP2 macro mode; VU0 data offsets `>= 0x4000` map to VU1's registers |
| **VU1** | 16 KiB (`2048` 64-bit pairs) | 16 KiB (`1024` qwords) | `0x1100_8000..0x1100_BFFF` (micro), `0x1100_C000..0x1100_FFFF` (data) | EFU (`P` register), `XGKICK` -> GIF PATH1, VIF1 double buffering (`TOP`/`TOPS`) |

Each `Vu` holds:
- 32 × 128-bit floating-point vector registers `VF00–VF31` (`VF00` fixed to `[0.0, 0.0, 0.0, 1.0]`).
- 16 × 16-bit integer registers `VI00–VI15` (`VI00` fixed to `0`; indices `16..31` read `0` and ignore writes).
- `ACC` (4 × 32-bit), `I` (32-bit immediate), `Q` (divider output), `P` (VU1 EFU output), `R` (23-bit LFSR random register), `TPC` (program counter), and three flag registers: `Status` (12-bit: `Z/S/U/O/I/D` + sticky `ZS..DS`), `MAC` (16-bit: `O/U/S/Z` × `x/y/z/w`), and `CLIP` (24-bit: 4 × 6-bit clipping judgement history).

### 3.2 VU0 COP2 Macro Mode (`src/ee/cop2.rs`)

The EE drives VU0 directly through COP2 (`Status.CU2` required):
- **Control Registers (`CCR[2, 0..31]`)**: `0..15` alias `VI00–VI15`; `16..18` are `Status`, `MAC`, `CLIP`; `19` reads `0x2E30`; `20..22` are `R`, `I`, `Q`; `24` retains bits masked by `0x0C0C`; `26` is `TPC`; `27` `CMSAR0`; `28` `FBRST` (force break `FB0`/`FB1`, reset `RS0`/`RS1`, debug/trace interrupt enables `DE`/`TE`, kept mask `0x0C0C`); `29` `VPU-STAT` (run/stop/busy flags `VBS`, `VDS`, `VTS`, `VFS`, `VGW`, `DIV`, `EFU` for VU0 in bits `0..7` and VU1 in bits `8..15`); `31` `CMSAR1` (writing while VU1 is idle starts VU1 at `value >> 3`).
- **Interlocks**: `QMFC2.I` and `CFC2.I` wait until VU0 micro mode finishes (`last + 4` cycles); `QMTC2.I` and `CTC2.I` wait until VU0 finishes or executes a pair with the `M` bit set (`release0`); macro VU instructions wait for VU0 micro completion (`last + 3` cycles); `VCALLMS`/`VCALLMSR` start VU0 micro mode with a 3-cycle startup latency (`pipe::START = 3`), plus a 1-cycle stall immediately after a macro instruction.
- **VU1 Branch & Window**: `BC2F`/`BC2T`/`BC2FL`/`BC2TL` test whether VU1 is running a microprogram (`Vpu::vu1_busy`). VU0 data-memory accesses at `>= 0x4000` read/write VU1's `VF00–VF31`, `VI00–VI15`, `Status`, `MAC`, `CLIP`, `R`, `I`, `Q`, `P`, and `TPC` via `Vu::window_read`/`window_write`.

### 3.3 Micro Mode Pipelines, EFU, XGKICK, and Double Buffering

`src/vu/pipe.rs` executes 64-bit upper/lower instruction pairs in lockstep at their `T` stage:
- **FMAC & Flag Pipeline**: FMAC register results and `MAC`/`Status`/`CLIP` flag updates become visible at the `S` stage, `FMAC = 4` cycles after the pair's `T` stage. Register reads stall per-lane (`vf_ready[32][4]`) and integer loads (`ILW`/`ILWR`) stall via `vi_ready[16]`. `ACC`, `I`, `Q`, `P`, and `R` are never hardware-interlocked.
- **Integer Branch Hazard**: An integer-unit write in cycle `t - 1` is not yet visible to a conditional branch in cycle `t`; `branch_vi` returns the prior value stored in `vi_last_old`.
- **Divider & EFU**: `DIV`/`SQRT` land in `Q` after 7 cycles; `RSQRT` after 13 cycles. VU1's EFU (`src/vu/efu.rs`) computes `ESADD`, `ERSADD`, `ELENG`, `ERLENG`, `ESUM`, `ERCPR`, `ESQRT`, `ERSQRT`, `ESIN`, `EATAN`, `EATANxy`, `EATANxz`, and `EEXP` by running the hardware's exact FMAC/divider polynomial nano-code sequences over `crate::float` (preserving Booth rounding behavior, without touching flags) and supports a second overlapping EFU result in flight (`p_later`).
- **XGKICK & PATH1**: VU1 `XGKICK` submits a quadword data address in VU1 data memory to `Link::path1` (`gif::Path1Port`). If PATH1 is still transferring a prior kick, the subsequent pair stalls until `kick_wait` (`path1_free`).
- **Double-Buffered Memory**: VIF0/VIF1 maintain `BASE`/`OFFSET`/`TOP`/`TOPS` and `ITOP`/`ITOPS` (exposed to VU microprograms via `XTOP` and `XITOP` through `Link::top` and `Link::itop`). On VU1, `MSCAL`/`MSCALF`/`MSCNT` swap `TOP`/`TOPS` so VIF1 can unpack the next geometry batch into the alternate buffer while VU1 transforms and kicks the current buffer.
- **Stop & Debug Bits**: Upper instruction bits control `I` (32-bit immediate in lower word), `E` (end after 1-pair delay slot), `M` (interlock release), `D` (debug halt, sets `VDS` and raises VU0/VU1 INTC if `FBRST.DE` enabled), and `T` (trace halt, sets `VTS` and raises INTC if `FBRST.TE` enabled).

---

## 4. Dynamic Recompilers

Sapphire uses two distinct JIT architectures tailored to the EE and VU execution models. Both are enabled by default in the Core ABI (`ee_tier = "recompiler"`, `vu1 = "recompiler"`, `vu0 = "recompiler"`).

### 4.1 EE Tailored x86-64 Block Recompiler (`src/ee/jit/`, `src/blocks.rs`)

The EE recompiler compiles guest basic blocks directly to native x86-64 machine code on x86-64 Unix hosts using `emusen-assembler` (`X64`) and `emusen-mips-planner` (it does not use Cranelift). On other targets, `isa::supported()` returns `false` and the EE runs on the interpreter.

- **Block Discovery & Shaping (`src/ee/jit/shape.rs`)**: Blocks start only at aligned cached instruction fetches (`Config.ICE = 1`, no pending interrupt, outside delay slots, performance counters and hardware breakpoints off). A block holds up to `shape::MAX = 1024` words (clamped to a single 4 KiB page) of `Plain` integer/multiply/divide/SA/trap/SYNC instructions, `Memory` loads/stores/`PREF`/`LWC1`/`SWC1`/`LQC2`/`SQC2`, and `Call` instructions (MMI, COP1 register ops, COP2 calls, `CACHE`), ending after a `Branch` and its delay slot or before any `Other` instruction (privileged COP0, `BC0*`/`BC2*`, etc.).
- **Compilation Policy & Cache (`src/blocks.rs`)**:
  - **Threshold**: Blocks compile synchronously on the dispatcher thread on their `THRESHOLD = 64`-th entry (`EMUSEN_SAPPHIRE_THRESHOLD`).
  - **Code Budget**: `BUDGET = 480 MiB` (`EMUSEN_SAPPHIRE_BUDGET_MIB`); exceeding the budget flushes code memory and resets all blocks.
  - **Front Table & Direct Linking**: A `FRONT = 65536`-entry jump table maps `(pc >> 2) & 0xFFFF` to block code pointers. Completed exits patch incoming link slots directly so consecutive compiled blocks chain without returning to Rust until `ctx.time >= ctx.deadline`.
  - **Self-Modifying Code Watch**: `Map::watch` tracks 64-byte physical lines covered by compiled blocks. Writes populate `map.written`; on the next entry, `Blocks::written` re-checks code words and marks modified blocks `State::Stale`, unlinking all incoming jumps.
- **Lowering & Memory Fast Paths (`src/ee/jit/lower.rs`, `src/ee/jit/isa.rs`)**:
  - **Fixed Host Registers**: `R15` (`Context`), `R14` (`Ee`), `R13` (`pages`), `R12` (`time`); 7 general pool registers (`RBX`, `RBP`, `RCX`, `R8`–`R11`) allocated in first-use order.
  - **Translation Cache**: `PAGES = 4096`-entry software TLB indexed by `(vaddr >> 12 ^ vaddr >> 24) & 0xFFF`, tagged by a 32-bit generation (with 4 recent `(tlb_stamp, mode/ASID/Config, rdram_ptr)` generations cached so kernel/user switches avoid thrashing). Fast paths inline scratchpad accesses and D-cache hits directly; misses, uncached accesses, and misaligned accesses invoke `slow`/`fill`. Register-page loads (`kind::REGISTER`) take a streamlined direct `Bus::read` path in `full()` without full register spill/reload.
  - **Native Vector MMI & Light COP2**: Supported lanewise MMI operations (`LANEWISE`: `PADDB/H/W`, `PSUB*`, `PMAX*`, `PMIN*`, `PCEQ*`, `PCGT*`, `PSLL*`/`PSRL*`/`PSRA*`, `PPAC*`, `PEX*`, `PINT*`, `PMFHL.*`, `PMULTH`, `PMADDH`, `PMSUBH`) lower directly to x86-64 SIMD vectors when supported. Macro COP2 ops with VU0 idle (`LIGHT` flag when `ctx.quiet != 0`) invoke VU0 register helpers directly without waking the full board.
  - **Folded Timing**: `isa::cost` runs `ee::timing::Timing` from rest over the block's plan at compile time (including `FETCH_PER_LINE = 1/16` cycle per cache line) so compiled code adds a single folded cycle count at each block exit; `Ee::blocks_ran` updates `Count`, `Compare` (`Cause.IP7`), and `Random` on exit.
  - **Lockstep Verification**: `Blocks::verify(Verify::Blocks, reference, quiet)` steps a shadow interpreter (`Shadow`) beside compiled runs and compares GPRs, `HI`/`LO`, `SA`, `PC`, `FPR`/`ACC`/`FCR31`, exceptions, delay-slot state, scratchpad, and RDRAM.

### 4.2 VU0 / VU1 Microprogram JIT (`src/vu/jit/`)

Both VU0 and VU1 share a Cranelift-based microprogram recompiler built on `emusen-jit::Jit<VuLowering>` (`cranelift-codegen`, `cranelift-frontend`, `cranelift-jit`, `cranelift-module`):

- **Whole-Microprogram Discovery**: Starting from a microprogram entry address, `discover()` traverses reachable pairs across fall-throughs, relative branches, `BAL`, and `JALR` return addresses up to each `E`-bit delay slot (`JR` targets exit as `OUTSIDE` back to the dispatcher).
- **Compilation Policy**: Default `Settings` compile asynchronously on `emusen-jit`'s background thread at `threshold = 2` entries, retain up to `VARIANTS = 32` distinct code versions per start address (so overlays uploaded in rotation stay hot), and flush the JIT module at `BUDGET_MIB = 128 MiB`. Environment overrides (`SAPPHIRE_VU_THRESHOLD`, `SAPPHIRE_VU_WAIT`, `SAPPHIRE_VU_VERIFY`, `SAPPHIRE_VU_VARIANTS`, `SAPPHIRE_VU_BUDGET_MIB`) configure measurement and verification.
- **Validation**: `MicroMemory::lent()` increments whenever micro memory is mutably borrowed; each `Program` skips byte-by-byte comparison when `lent()` matches its last verified stamp, and re-verifies its covered pairs only after a write.
- **Exact Semantics & Dead-Flag Elimination**: Pipeline timing, field-level hazards, `vi` branch hazards, write priority, integer ops, and control flow compile into Cranelift IR, while PS2 floating-point arithmetic calls `crate::float` directly for bit-exact results. Dead `MAC`/`Status` flag writes overwritten before any observable read are elided (`dead_flags`). Optional non-default compromises `C3` (host floats) and `C4` (immediate flags) exist behind `SAPPHIRE_VU_C3=1` / `SAPPHIRE_VU_C4=1` and are always off by default.

---

## 5. Event-Driven Scheduler and IOP Sync

`Board` (`src/board.rs`) drives all peripherals on a single monotonic EE cycle clock (`board.now`), advancing units lazily rather than polling cycle by cycle.

### 5.1 Event Horizon (`Board::reschedule`)

`board.next` holds the minimum scheduled EE cycle across:
1. **PCRTC (`pcrtc_next`)**: Next H-blank (only when `timers_h || gs.wants_hsync()`), V-blank rise/fall, V-sync, or displayed scanline readback.
2. **EE DMAC (`dmac.next_event()`)**: Next active channel transfer step.
3. **EE Timers (`timers_next`)**: Cached minimum compare/overflow event across T0–T3, recomputed only when a timer register changes (`timers_changed()`).
4. **VIF0 / VIF1 (`vif[n].next_event()`)** and **GIF (`gif.next_event()`)**.
5. **Deferred Transaction Interrupts (`lands`)**: Earliest cycle of an interrupt raised by an ahead-of-board VIF1/GIF/GS transaction.

When `board.next > t` and the GIF, both VIFs, and both VUs are idle (`Board::quick_to`), `run_to` takes the quick path: it advances `board.now`, catches up the IOP, and skips all event loops. Otherwise `run_now` fires all events with `next <= t` (landing deferred interrupts, syncing timers, taking PCRTC edges, reading due scanlines, and looping VIF/DMAC/IPU steps until no DMAC event remains at `at`), then executes `tail(t)` to advance VIFs, VUs (propagating `VU0`/`VU1` interrupts), GIF, and IOP.

### 5.2 IOP Synchronization (`src/iop/mod.rs`)

The IOP runs at exactly one-eighth of the EE clock (`clock::IOP = 8`, 36.864 MHz nominal vs 294.912 MHz EE).

- **Lazy Catch-Up**: `Board::quick_to` and `Board::tail` call `iop.catch_up(board.now)` (and event/SIF/CDVD boundaries call `iop.run_to(at)`). `Iop::run_to` immediately returns if `bus.now * 8 >= t`.
- **Event-Driven IOP Horizon**: Inside `Iop::run_on`, the IOP advances `blackmoon-r3000` in runs bounded by its own next internal unit event (`bus.set_horizon(t)` across IOP timers, DMAC, CDVD, SIO2, and SPU2). When held in reset (`iop.held == true`, before BIOS load or while EE asserts SIF `CTRL` bit 19), only the IOP's peripheral clock and units advance.
- **SIF Wake-EE**: When the IOP writes into a SIF FIFO that unblocks an EE DMAC channel, it sets `sif.wake_ee_pending()`. `Board::deadline()` collapses to `0` so compiled EE code returns immediately, and `Board::take_wake()` / `quick_wake()` wakes the EE DMAC at `board.now`.

---

## 6. Live Two Faces Execution Split (`board.round`)

In the live codebase (`src/board.rs`, `src/blocks.rs`, `src/machine.rs`), Sapphire's "Two Faces" separation between cycle-ordered reference execution and batched fast execution is controlled at runtime by `board.round: u64` (rather than a compile-time generic bus trait):

| Aspect | Cycle-Ordered Reference Face (`board.round == 0`) | Batched / Compiled Byte Face (`board.round > 0`, default `ROUND = 64`) |
| --- | --- | --- |
| **Default Construction** | `Board::new()` / `System::new()` (used by unit tests, reference shadow verifier, and cycle-exact debugging) | `Machine::console()` sets `board.round = ROUND` (`64` EE cycles), overridable via `EMUSEN_SAPPHIRE_ROUND` |
| **EE Execution (`blocks.rs`)** | `blocks::block()` returns `None` immediately; every instruction runs via `System::step()` on the interpreter | Compiled x86-64 blocks run up to `deadline = next.min(now + round)` (also bounded by unmasked `Count == Compare` and `0` on pending IOP SIF wake) |
| **Board Catch-Up (`board.rs`)** | `run_to_held()` always executes `run_now()`; `owed` never lags `now` | While `t < now + round && next > t && !sif.wake_ee_pending()`, `run_to_held()` records `owed = owed.max(t)` and returns `Ran::Owed`. Any EE bus read/write to a peripheral calls `observe()` to catch up to `owed`; VU0-only COP2 accesses (`Bus::vpu0()`) call `observe_vu0()`, advancing only VIF0 and VU0 |
| **VIF1 / VU1 / GIF Ahead Transactions** | Disabled (`dmac.set_ahead(false)`, `ahead = 0`); VIF1, VU1, GIF, and DMAC advance strictly to `now` | Enabled (`dmac.set_ahead(true)`): `Board::step_dmac()` calls `Board::transact()`, which runs DMAC channels 1 (VIF1), 2 (GIF), 8 (`fromSPR`), and 9 (`toSPR`) together with VIF1, VU1, and GIF ahead of `board.now` up to the next vertical edge (`v_edge`) |
| **Ahead Consistency Mechanisms** | Unused (`ahead_log`, `foreign`, `lands` remain empty) | 1. **`Vu::ahead_log`**: Logs VU1 data-memory stores made ahead of `board.now` with their cycle and previous qword so GIF PATH1 reads (`mem_q_at`) see the exact value at the transfer cycle.<br>2. **`Vu::foreign`**: Queues VIF1 writes to a running VU1 by cycle (`foreign_next`) and lands them before VU1 executes past each cycle.<br>3. **`Board::lands`**: Buffers VIF1 and GS interrupts raised during ahead transactions (`raise_at`) and latches them into `INTC` at their exact cycle (`land(at)`). |
| **Frame & Save-State Settlement** | Always settled at every instruction | `Machine::advance()` loops while `pcrtc.fields < target || board.holds()`, where `holds()` is true if `!lands.is_empty() || !dmac.fits() || !vpu.fits()`. `System::settle()` then drains any remaining `owed` cycles before frame end or `SPH2` state serialization |

> [!NOTE]
> The live implementation switches between both faces dynamically through `board.round` on a single `Board` and `System` type; there is no separate compile-time generic board parameter. Setting `EMUSEN_SAPPHIRE_ROUND=0` (or selecting the EE interpreter with `round = 0`) forces the cycle-ordered reference path throughout the machine.
