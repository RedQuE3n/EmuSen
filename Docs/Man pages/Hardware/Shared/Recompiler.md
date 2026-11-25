# Shared Recompiler Infrastructure (`emusen-jit`, `emusen-assembler`, `emusen-mips-planner`)

Live crates in `EmuSen/Cores/Shared/`:
- `EmuSen/Cores/Shared/emusen-jit` (`emusen_jit`, v1.3.0)
- `EmuSen/Cores/Shared/emusen-assembler` (`emusen_assembler`, v1.3.0)
- `EmuSen/Cores/Shared/emusen-mips-planner` (`emusen_mips_planner`, v1.0.0)

These three crates provide the shared runtime, x86-64 macro-assembler, and MIPS block planner used by EmuSen's dynamic recompilers:
- **`MarsRT` (N64 VR4300):** Uses `emusen-jit` (Cranelift background compiler tier, default features).
- **`Sapphire` (PS2):** Uses `emusen-jit` (Cranelift tier for VU1 microprograms) and `emusen-jit` with the `native` feature + `emusen-assembler` + `emusen-mips-planner` for its tailored EE (R5900) recompiler.

---

## 1. `emusen-jit` — Compiler Thread, Publication, Code Memory & Verifier (v1.3.0)

Dependencies: `cranelift-{codegen,frontend,jit,module,native}` 0.136.0; optional feature `native` enables `emusen-assembler` and `libc`.

### 1.1 Cranelift Background Compiler (`src/lib.rs`)
Generic over a guest processor's `Lowering: Clone + Send + 'static`:
- **`Lowering` trait:** Defines `type Options: Copy + Send + 'static`, `const NAME: &'static str` (thread named `<NAME>-compiler`), `const EXITS: &'static [&'static str]` (exit reason names queried via `exit_name::<L>`), and `lower(&mut self, module, func, functions, words: &[u32], start: u32, options) -> Result<(), String>`. The lowering populates `func` and embeds any host callbacks (`iconst` + `call_indirect`); `emusen-jit` declares (`b<start:08x>_<serial>`), defines, finalizes, measures, and publishes the code.
- **Background thread & queue (`Compiler<L>`, `Jit<L>`):**
  - Started lazily on the first `enqueue`. Uses an internal Acquire/Release spin-locked queue with 1 ms `park_timeout` when idle so synchronization stays visible to ThreadSanitizer.
  - Each wake drains all queued jobs, compiles them in order, and calls `JITModule::finalize_definitions` once per batch, transitioning newly written pages to `r-x` (W^X: published Cranelift pages are never writable again).
- **Lock-free slot publication (`Slot`):** Single `AtomicUsize` holding `0` (pending), `FAILED = 1` (refused; dispatcher runs block another way permanently), or the executable entry pointer. Stored with `Release` *after* pages become executable; read with `Acquire` (`published()`, `settled()`, `wait()`).
- **Code budget & non-blocking retirement (`Jit::over_budget`):** Code is never freed per block. When compiled bytes exceed `budget`, `over_budget()` increments `flushes`, signals stop, and detaches/retires the old compiler thread without blocking on its remaining queue (the retiring thread marks remaining queued slots `FAILED`, finishes its in-flight block, and frees its module on exit). The dispatcher discards all cached entry pointers before resuming execution. Explicit `reset()` or `Drop` joins the thread after refusing remaining queued jobs.
- **Telemetry & profiling:** `Counters` tracks `queued`, `finished`, `compiled`, `failed`, `nanos`, and `bytes`; `Jit::spent()` accumulates totals across budget flushes; setting `perf_map` writes `/tmp/perf-<pid>.map` symbols for Linux `perf`.

### 1.2 Differential Reference Verifier (`src/verify.rs`)
- **`verify::Reference` trait:** Guest implements `step()`, `differs(&Subject)` (per-instruction architectural state), and `differs_whole(&Subject)` (full RAM/device check at run end).
- **`verify::Verifier<R>`:** Steps a reference interpreter alongside compiled/batched execution via `follow()`, `check()`, and `check_all()`, panicking immediately at the exact diverging instruction with the first mismatching field.

### 1.3 Standalone Native Code Memory (`src/native.rs`, `native` feature)
`native::Memory` is a single-owner code memory for tailored recompilers that emit machine code directly on the dispatcher thread (eliminating Cranelift IR/regalloc2 overhead):
- Reserves a contiguous **1 GiB** virtual region (`RESERVED = 1 << 30`) split into three arenas so all `rel32` branches/calls and RIP-relative accesses stay within ±2 GiB:
  - **Hot arena (`0 .. 512 MiB`):** Shared stubs (placed first at `Memory::new(stubs)`) followed by block fast paths laid side-by-side.
  - **Cold arena (`512 MiB .. 768 MiB`):** Slow/fallback paths.
  - **Data arena (`768 MiB .. 1 GiB`):** Permanently `rw-` (never executable) so block link tables and dynamic target words can be updated without modifying code pages.
- **W^X placement (`place(Code) -> Option<u64>`):** While no code in the memory is executing, temporarily marks touched hot/cold pages writable, writes code and patches relocations (hot/cold labels, data offsets, stub entries, and data address slots), restores `r-x` protection, and returns the block entry address (or `None` when any arena is full).
- **API:** `Memory::new(stubs)`, `place(code)`, `stub(id)`, `held()`.

---

## 2. `emusen-assembler` — Macro-Assembler & x86-64 Backend (v1.3.0)

`VERSION = (1, 3)`. Dependencies: `cranelift-assembler-x64` (used for verified instruction encoding).

### 2.1 Core `Assembler` Trait (`src/lib.rs`)
Provides a narrow, backend-neutral macro-assembler interface writing into split `Section::Hot` / `Section::Cold` code buffers plus 8-byte-aligned `data`:
- **Labels & accounting:** `label`, `bind`, `section`, `current`, `purpose` (up to `PURPOSES = 16` byte-count categories for code-density profiling), `finish(entry) -> Code`.
- **Scalar integer & memory operations (`Width::{W32, W64}`, 32-bit writes zero-extend to 64 bits):** `mov`, `mov_imm`, `alu` (`Add`, `Sub`, `And`, `Or`, `Xor`), `alu_mem` (1/4/8 bytes), `shift` (`Shl`, `Shr`, `Sar` by immediate or register), `not`, `neg`, `sext32`, `mul` (low 64 bits), `divrem` (signed/unsigned quotient & remainder; operands must avoid `DIV_REGS`), `cmp`, `cmp_mem`, `test`, `set(Cond, d)`, `select(Cond, d, s)`, `load` (1/2/4/8 bytes, signed/unsigned), `store`, `store_imm`, `lea`.
- **Control flow & linking:** `jump`, `branch(Cond, Label)` (`Eq`, `Ne`, `Lt`, `Ge`, `Le`, `Gt`, `Ltu`, `Geu`, `Leu`, `Gtu`, `Overflow`), `jump_stub`, `call_stub`, `lea_label`, `lea_data`, `jump_via_data`, `jump_mem`, `data`, `data_address` (emits an 8-byte data word patched with a label's runtime address on placement).
- **Output (`Code`):** Holds `hot`, `cold`, `data`, `entry`, `relocs: Vec<Reloc>`, `addresses`, and per-purpose byte counts.

### 2.2 Optional `Vector` Trait (v1.3)
Extends `Assembler` with 128-bit SIMD lane operations over scratch `Vreg` registers (caller-saved under System V ABI; nothing lives in `Vreg` across helper calls):
- **`vop(VOp, d, a, b)`:** 8/16/32/64-bit wrapping and signed/unsigned saturating add/sub, signed/unsigned min/max, equal/greater-than compares, bitwise ops (`And`, `Or`, `Xor`, `AndNot`), low/high lane interleaves, saturating packs, halfword/word multiplies, byte shuffle, and 32-bit float `Add`/`Sub`/`Mul`/`Min`/`Max`.
- **`vunary`**, **`vshift`** (constant shift), **`vshuffle`** (word / low-or-high halfword), **`vblend_h`** (constant mask), **`vmov`**, unaligned 16-byte **`vload`/`vstore`/`vload_data`**, **`vjoin`/`vsplit`** (between 128-bit `Vreg` and two general-purpose `Reg`s), and **`fpcr_load`/`fpcr_store`** (host floating-point control register, e.g., MXCSR for round-toward-zero / DAZ / FTZ).

### 2.3 x86-64 Backend (`src/x64.rs`)
- Implements `Assembler` and `Vector` on `X64`, encoding integer and 128-bit three-operand AVX/VEX vector instructions via `cranelift-assembler-x64` (with `LDMXCSR`/`STMXCSR` encoded directly).
- Exports standard register constants (`RAX`–`R15`, `ALL`, `DIV_REGS = [RAX, RDX]`, `HELPER_ARGS = [RDI, RSI, RDX]`), `Regs`, and `trampoline(a, context, helper)`.
- `x64::vector_supported() -> bool` probes host AVX and BMI2 support at runtime so callers can fall back cleanly when unavailable.
- No AArch64 backend is implemented yet; non-x86-64 hosts rely on Cranelift or interpreter paths.

---

## 3. `emusen-mips-planner` — MIPS Block Shaper & Dataflow Planner (v1.0.0)

`VERSION = (1, 0)`. Zero runtime dependencies (`src/lib.rs`).

Turns raw guest MIPS instruction words into plain, printable `Plan` data from a guest-supplied `Isa` table:
- **`Isa` trait:** Supplies `const MAX: usize` (maximum instructions per block) and `info(&self, word: u32) -> Info`, classifying each instruction into:
  - `Class::Plain`: Continues block.
  - `Class::Memory`: Load/store with `Access { base, disp, bytes, store, align }`; may exit block early at runtime on non-fast-path addresses.
  - `Class::Branch`: Terminates block together with its delay slot (which must be `Plain` or `Memory`); records `likely` for annulled delay slots.
  - `Class::Ender`: Ends block before this instruction.
- **`shape(isa, words) -> Option<Vec<u32>>`:** Extracts the maximal legal block prefix up to `I::MAX`, including a terminating branch and its delay slot only when both fit and the slot is `Plain` or `Memory`.
- **`plan(isa, words, start) -> Result<Plan, String>`:** Validates block structure and computes per-step and block-level facts:
  - **`exposed` / `written` masks:** 32-bit GPR masks identifying registers read before written (live-in) and registers written anywhere in the block (`$0` always excluded).
  - **`Step::dirty`:** Registers written earlier in the block prior to each step (for minimal writeback on early exits).
  - **`Step::base_def` & `Step::reuse` (`REACHES = 4`):** Tracks up to 4 most-recent memory accesses per `(base, Def)` pair so emitters can reuse prior address/page translations (`reuse.page = Some((earlier_idx, same_disp_and_align))`) and cache-line checks (`reuse.line = Some(earlier_idx)` when within 64 bytes and access permissions match).
