# Shared ISA Tables & Single-Step Test Harness (`emusen-mips`, `emusen-6502`, `emusen-singlestep`)

Live crates in `EmuSen/Cores/Shared/`:
- `EmuSen/Cores/Shared/emusen-mips` (`emusen_mips`, v0.1.0)
- `EmuSen/Cores/Shared/emusen-6502` (`emusen_6502`, v0.1.0)
- `EmuSen/Cores/Shared/emusen-singlestep` (`emusen_singlestep`, v0.1.0, dev-only)

These crates hold instruction-set decode tables, disassemblers, test assemblers, and the shared SingleStepTests harness runner. None of them implement CPU execution; execution semantics stay inside each processor crate so architectural differences never collapse into flag-driven interpreters.

---

## 1. `emusen-mips` — MIPS Instruction Tables, Disassemblers & Assembler

Zero runtime dependencies. Covers five MIPS-family decoders/disassemblers used across `blackmoon-r3000`, `blackmoon-gte`, `MarsRT`, `Sapphire`, and `Platform/systems`:

| Module | Target Architecture | Capabilities |
|---|---|---|
| `src/decode.rs` | MIPS I (R3000A) | `Word` bit-field extractor, `decode(u32) -> Op` (58 core ops + 9 coprocessor forms × 4 slots + `Op::Reserved`), `Op::canonical()`, `Op::operands()`. Follows R3000A partial decoding: unused fields (`sa`, `rs`, `rt`, `code`) are ignored; `REGIMM` checks bit 16 (`BLTZ`/`BGEZ`) and links only when `rt[20:17] == 0b1000`; coprocessor bit 25 selects `COPz` regardless of bits `24:21`, and `BCzF`/`BCzT` checks `rt[0]` alone. |
| `src/disasm.rs` | MIPS I / Dialect-driven | Zero-allocation `Display` disassembler (`disassemble(word, pc, dialect)`). Uses `Dialect` and `Coprocessor` data tables so each chip crate supplies its own COP register and command names (`blackmoon-r3000` COP0, `blackmoon-gte` COP2). Supports `Spelling::AppendixA` (lowercase, `fp` for `$30`, signed hex immediates, `nop`, `jalr rs` when `rd == 31`, `.word` for reserved) and `Spelling::Vr4300`. |
| `src/asm.rs` | MIPS I / R5900 / VU | Line and multi-line assembler (`assemble_line`, `assemble`) with `label:` targets, `#`/`;` comments, `.word`, range/alignment/256 MiB jump-region checks, and no synthetic pseudo-instructions (`li`, `la`, `move`, `b` omitted so every line maps 1:1 to a machine word). |
| `src/vr4300.rs` | MIPS III (N64 VR4300) | Standalone decode/disassembler returning `vr4300::Instruction` (mnemonic, operands, and resolved static target address for branches, jumps, and `$zero`-based loads/stores). Uses VR4300 spelling (`s8` for `$30`, folded forms `beqz`/`b`/`move`/`neg`/`not`, COP0 and FPU register names). |
| `src/rsp.rs` | N64 RSP (Scalar + VU) | Standalone decode/disassembler returning `vr4300::Instruction` for the RSP scalar MIPS subset, `mfc0`/`mtc0` over the 16 SP/DP registers (libultra names), and COP2 vector loads/stores/operations with SGI mnemonics and element selectors. Wraps branch/jump targets to 4 KiB IMEM and `$zero`-based memory references to 4 KiB DMEM. |
| `src/r5900.rs` | PS2 Emotion Engine (R5900 / C790) | 290-mnemonic decode, disassemble (`Dialect::R5900`, Appendix A spelling), and assemble tables covering primary, `SPECIAL`, `REGIMM`, `MMI`/`MMI0`–`MMI3`, `COP0` (including `Debug`/`Perf` sub-tables `MFBPC`–`MTDVBM`, `MFPS`/`MFPC`/`MTPS`/`MTPC`), `COP1` (single-precision PS2 EE subset: `RSQRT.S`, `ADDA.S`/`SUBA.S`/`MULA.S`/`MADD.S`/`MSUB.S`/`MADDA.S`/`MSUBA.S`, `MAX.S`/`MIN.S`, `C.LT`/`C.LE`, `BC1FL`/`BC1TL`, `SQRT.S fd, ft` with `fs == 0`), `SYNC.P`, `PMFHL`/`PMTHL`, three-operand `MULT`/`MADD` (pipeline 0 and 1), `LQ`/`SQ`, and `COP2` (`LQC2`/`SQC2`, `QMFC2`/`QMTC2`/`CFC2`/`CTC2` with `.i` interlock bit 0, and VU0 macro ops). Excludes unimplemented MIPS III ops (`DMULT*`, `DDIV*`, `LL*`, `SC*`). |
| `src/vu.rs` | PS2 VU0 & VU1 | Unified micro-mode (64-bit VLIW upper/lower instruction pair with `I`/`E`/`M`/`D`/`T` flags in upper bits `31:27`) and macro-mode (`COP2` Special1/Special2, `v`-prefixed) tables, operand formats, `dest` field order (`x` = bit 24 down to `w` = bit 21), micro-only lower instructions, EFU, and `XGKICK`, plus VU assembler encoding. |

---

## 2. `emusen-6502` — 2A03 / 6502 Disassembly Table

Zero runtime dependencies (`src/lib.rs`). Single owner of the 256-entry NES 2A03 instruction table shared by `MoonRT`'s debugger (`debug_disassemble`) and `Platform/systems`'s NES disassembler:

- **`NAMES: [&str; 256]`:** Uppercase mnemonics for all 256 opcodes, including undocumented instructions (`SLO`, `RLA`, `SRE`, `RRA`, `SAX`, `LAX`, `DCP`, `ISC`, `ANC`, `ALR`, `ARR`, `ANE`, `SHA`, `TAS`, `SHY`, `SHX`, `LXA`, `LAS`, `SBX`, `USBC` at `$EB`, `JAM`, and multi-byte `NOP`s).
- **`Mode` (13 addressing modes):** `Implied`, `Accumulator`, `Immediate`, `ZeroPage`, `ZeroPageX`, `ZeroPageY`, `Absolute`, `AbsoluteX`, `AbsoluteY`, `Indirect`, `IndexedIndirect`, `IndirectIndexed`, `Relative`.
- **Functions:**
  - `mode_of(u8) -> Mode`, `mnemonic_of(u8) -> &'static str`, `length_of(u8) -> usize` (1, 2, or 3 bytes).
  - `format_operand(opcode, address, low, high) -> String`: Formats operands in standard `$`-prefixed hex syntax (`#$3F`, `$1234,X`, `($FFFC)`, `($10),Y`, `A`) and resolves `Relative` branch targets from `address + 2 + offset as i8`.
  - `classify(opcode, operand) -> Option<(Reference, i32)>`: Classifies static `Absolute` and `ZeroPage` targets as `Reference::Call` (`$20` JSR, `$4C` JMP), `Reference::Write` (`STA`/`STX`/`STY`/`SAX`/`INC`/`DEC`/`ASL`/`LSR`/`ROL`/`ROR`), or `Reference::Read` (`LDA`/`LDX`/`LDY`/`CMP`/`CPX`/`CPY`/`ADC`/`SBC`/`AND`/`ORA`/`EOR`/`BIT`) for debugger cross-references.

---

## 3. `emusen-singlestep` — Shared Single-Step CPU Test Harness

Development-only crate (`dev-dependency` in CPU crates; never ships in release binaries). Depends on `serde_json`. Factors out the common SingleStepTests JSON suite runner shared by `beryl-z80`, `beryl-m68k`, `blackmoon-r3000`, and `VenusRT` (`w65816` and `spc700`):

- **Suite loading:** `suite_files`, `read_cases`, `int`, and `ram` enumerate per-opcode JSON files in deterministic order and parse initial/final register states and `[address, value]` RAM pairs from `serde_json::Value`.
- **Grading & reporting:**
  - `Outcome` / `Outcome::from_checks(regs, mem, cycles)`: Reports the first mismatch in strict priority order: registers, then memory, then bus cycles. Tracks `capped` (cases truncated at a suite's cycle limit) and `explained` (failures with documented hardware exceptions, counted separately from `unexplained`).
  - `first_mismatch`: Finds the first differing index or length mismatch between expected and actual bus-cycle lists.
  - `FileReport`, `run_cases`, `run_suite`, `total`, `summary`, `print_misses`: Executes opcode files in parallel across worker threads while reporting results in deterministic file order.
- **CPU-specific boundary:** Register sets, bus implementations, cycle formats, and suite-specific quirks stay inside each CPU crate's test harness, which supplies a per-case grading closure returning `Outcome`.
