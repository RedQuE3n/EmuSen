# Beryl CPUs and Assemblers

Reference manual for the processor crates (`beryl-m68k`, `beryl-z80`, `beryl-sh2`) and the shared assembler front end (`beryl-asm`) in `EmuSen/Cores/Sega/Beryl-HW/`.

---

## `beryl-m68k`

Motorola 68000 CPU core (`BUILT = true`). Used as the Genesis main CPU and the Sega CD sub-CPU in `nephrite`.

### Bus Interface and Programmer's Model

- **`Access`**: `{ address: u32, size: Size, function: u8, locked: bool }`. Carries the full 24-bit address (including `A0` as byte select for `Size::Byte`), cycle width (`Size::Byte` or `Size::Word`), function code `FC2–FC0` (`function::USER_DATA = 1`, `USER_PROGRAM = 2`, `SUPERVISOR_DATA = 5`, `SUPERVISOR_PROGRAM = 6`, `INTERRUPT_ACKNOWLEDGE = 7`), and `locked` for `TAS`'s read-modify-write bus cycle where `AS` stays asserted between read and write. The bus decides whether a locked write completes (on the Genesis, the main bus drops the write).
- **`Bus` trait**:
  - `read(&mut self, access: Access) -> u16` — 4-clock read cycle plus any bus wait states; byte reads return the byte in the low 8 bits.
  - `write(&mut self, access: Access, value: u16)` — 4-clock write cycle plus waits.
  - `idle(&mut self, clocks: u32)` — internal processor clocks in manual cycle order.
  - `interrupt_level(&mut self) -> u8` — current `IPL2–IPL0` value (`0` = none, `7` = non-maskable).
  - `acknowledge(&mut self, level: u8) -> Option<u8>` — interrupt acknowledge cycle in CPU space (FC 7); returns a device vector or `None` for autovector (`24 + level`). The bus accounts for cycle clocks including E-clock autovector synchronization.
  - `address_error(&mut self, access: Access, write: bool)` — abandoned odd-address word cycle (`AS` asserted without `UDS`/`LDS`); defaults to `self.idle(4)`.
  - `reset_devices(&mut self)` — external pulse driven by the `RESET` instruction.
- **`Registers`**: `d: [u32; 8]`, `a: [u32; 8]` (`a[7]` is the active mode's stack pointer), `other_sp: u32`, `sr: u16`, `pc: u32`, and `prefetch: [u16; 2]` (`IRD` and `IRC`). Between instructions, `pc` addresses the word in `prefetch[0]` and `prefetch[1]` holds the following word. Helper methods: `supervisor()`, `usp()`, `ssp()`.
- **`M68000`**: exposes `new()`, `reset(&mut bus)`, `step(&mut bus) -> Step`, and `step_observed(&mut bus, &mut observer) -> Step`. `Step` variants are `Instruction`, `Exception(u8)`, `Stopped`, `Halted`, and `Observed(u32)`.

### Instruction Boundaries and Exceptions

Each step evaluates events at the instruction boundary in strict priority order:

1. **Pending trace (`trace_pending`, vector 9)**: Highest group-1 priority; taken before interrupts. Owed after normal instructions and group-2 exceptions (`TRAP`, `TRAPV`, `CHK`, divide-by-zero), never after illegal instructions, privilege violations, or address errors.
2. **Pending interrupt**: Taken when the sampled IPL exceeds the SR interrupt mask, or when IPL rises from `< 7` to `7` (edge-triggered non-maskable interrupt, tracked via `last_level`). Level 7 held into its handler does not re-trigger until the SR mask is lowered below 7.
3. **`STOP`**: Idles 4 clocks per step until ended by an interrupt or trace; the stacked PC points to the instruction after `STOP`.
4. **Instruction execution**: Invokes `observer.before(...)`, then executes the instruction.

| Exception | Bus Sequence | Clocks |
|---|---|---|
| Interrupt | 6 idle; stack PC low; acknowledge cycle (FC 7); 4 idle; stack SR, PC high; read vector; prefetch, 2 idle, prefetch | 44 (with 4-clock ack) |
| Reset | 14 idle; read SSP and PC from `$000000`–`$000007` (supervisor program space); prefetch, 2 idle, prefetch | 40 |
| Trace | 4 idle; stack PC low, SR, PC high; read vector 9; prefetch, 2 idle, prefetch | 34 |

### Settled Hardware Quirks

- **Interrupt sampling during `MOVE` to memory**: Any `MOVE` whose destination is memory samples `IPL2–IPL0` *before* its first write (`move_sample`). The boundary immediately after the `MOVE` uses that earlier sample, so an interrupt raised by the store itself is seen at the boundary after the following instruction. Other stores (including `MOVEM` to memory) sample at the instruction boundary. An address error in the `MOVE` discards `move_sample`.
- **Group 0 address errors**:
  - The faulting odd-address word access runs on the bus as an abandoned 4-clock cycle (`AS` without data strobes), followed by 8 internal clocks before the 7-word exception frame is stacked.
  - The stacked access address carries all 32 bits (the bus sees 24 bits).
  - Postincrement `(An)+` is never applied before a faulting access; a faulting `(An)+` operand leaves `An` unchanged for all widths and for `CMPM`. `MOVE.L` to `-(An)` updates `An` only after both words are stored; `ADDX.L`/`SUBX.L` adjust after both operand reads.
  - Stacked PC rules: opcode address + 2 advanced by 2 per fetched absolute/immediate word and per word/byte predecrement (displacements of `(d16,An)`, `(d8,An,Xn)`, and PC-relative modes do not advance it). `DBcc` taking an odd branch stacks opcode + 2; `JSR` via `(d16,An)`, `(d8,An,Xn)`, `(d16,PC)`, or `(d8,PC,Xn)` stacks opcode + 2 (whereas `(An)` and absolute modes stack the return address); `MOVEM` to registers via `(d8,An,Xn)` or `(d8,PC,Xn)` stacks 4 less than other modes.
  - For `MOVE.B`/`MOVE.W` to `-(An)`, the store occurs after the final prefetch, so the stacked IR is the next opcode, and the status word's `I/N` bit is set if the step following that next opcode fetch would be an exception (pending trace, illegal/Line-A/Line-F opcode, or user-mode privilege violation).
  - When `MOVE.L` from a register to `(d16,An)` or `(d8,An,Xn)` faults on its first store, `N` is set from bit 31, `Z` is cleared if the high word is non-zero (left untouched otherwise), and `V`, `C`, `X` are untouched.
- **PC-relative function codes**: `(d16,PC)` and `(d8,PC,Xn)` read operands in program space (FC 2 or FC 6).
- **`RTE` and `RTR` read order**: Read the status/CCR word first, then PC high word, then PC low word.
- **`TRAPV` prefetch**: Performs its prefetch in the current privilege mode before entering supervisor mode on trap.
- **SR/CCR immediate writes**: `ORI`, `ANDI`, `EORI`, and `MOVE` to CCR or SR refill the prefetch queue from the word immediately following the instruction.
- **`CHK`, `DIVU`, and `DIVS`**:
  - `CHK` without trap: 6 idle clocks then prefetch. With trap: 8 idle clocks, or 10 when the register is negative and `(bound - reg)` as a 16-bit difference is non-negative; sets `N` from register sign, `Z` if register is zero, clears `V` and `C`.
  - Division cycle counts follow the exact per-quotient-bit flowchart timing.
  - Division by zero leaves `X`, clears `V` and `C`, sets `N` from dividend bit 31 and `Z` if high word is zero for `DIVU`, and sets `Z` / clears `N` for `DIVS`.
  - Division overflow sets `N` and `V` and clears `Z` and `C` (`N` and `Z` on overflow are undefined in the Motorola manual and remain a known minor uncertainty).
- **BCD (`ABCD`, `SBCD`, `NBCD`)**: Binary and decimal nibble carries select correction and reproduce exact undefined `N` and `V` flag outcomes.
- **Shifts and quick arithmetic**: `ASR` with count ≥ operand width sets `C` and `X` to the sign bit. `ASL.B` alters only the low byte. `ADDQ.L` and `SUBQ.L` to an address register take 8 clocks.
- **`LINK A7` and `UNLK`**: `LINK A7` pushes the original `A7` prior to decrement. `UNLK` reads memory before updating `A7` so an address error leaves `A7` intact.
- **`DBcc`**: Writes the decremented counter only after the branch target prefetch succeeds.
- **`BTST` immediate**: Idles 2 clocks after prefetch, matching register form.
- **Illegal opcode decoding**: `CMPI` with PC-relative destinations (`$0C3A`, `$0C3B`, `$0C7A`, `$0C7B`, `$0CBA`, `$0CBB`) and `$4E78`–`$4E7F` raise illegal-instruction exceptions.

### State Block, Disassembler, and `asm` Feature

- **State (`src/state.rs`)**: Fixed 86-byte layout (`D`, `A`, `OtherSp`, `Sr`, `Pc`, `Prefetch`, `Stopped`, `Halted`, `LastLevel`, `TracePending`, `TraceResume`). `LastLevel` packs `last_level` in bits 2–0 and `move_sample` in bit 7 (present) + bits 6–4 (level).
- **Disassembler (`src/disasm.rs`)**: Emits Motorola syntax (`MOVE.W #$1234,-$10(A5)`, `$00FF0000.L`, target addresses on PC-relative modes including indexed `$000008(PC,D1.W)`), `DC.W` for undecoded opcodes, and static call/read/write references.
- **Assembler (`src/asm.rs`, feature `asm`)**: Supports full 68000 Motorola syntax (`HS`/`LO`, `DBRA`, `SP`, both `d(An)` and `(d,An)` forms, register lists, `dc.b`/`.w`/`.l`, `ds`, `dcb`, `org`, `even`, `align`, `equ`/`=`/`set`). Unsized absolute addresses select `.W` if resolvable on pass 1 within 24-bit sign-extended word range (`$0000`–`$7FFF`, `$FF8000`–`$FFFFFF`), otherwise `.L`. Unsized branches start as `.W` and relax monotonically to `.S` when in byte range (excluding displacement 0). Used at compile time to build the open Sega CD BIOS in `nephrite`.

---

## `beryl-z80`

Zilog Z80 CPU core. Used as the Genesis sound CPU in `nephrite` and the main CPU of the Master System, Mark III, SG-1000, and Game Gear in `endou`.

### Bus Interface and Programmer's Model

- **`Bus` trait**:
  - `fetch(&mut self, address: u16, refresh: u16) -> u8` — M1 opcode fetch followed by the refresh cycle carrying `(I << 8) | R` on the address bus.
  - `read(&mut self, address: u16) -> u8` / `write(&mut self, address: u16, value: u8)` — memory cycles.
  - `input(&mut self, port: u16) -> u8` / `output(&mut self, port: u16, value: u8)` — I/O cycles with the full 16-bit address bus driven by the Z80.
  - `idle(&mut self, t_states: u32)` — internal T-states without a bus cycle.
  - `int_line(&mut self) -> bool` — level-sensitive `INT` sampled at instruction boundaries.
  - `nmi_edge(&mut self) -> bool` — falling-edge `NMI` since last query.
  - `acknowledge(&mut self) -> u8` — 6-T-state interrupt acknowledge cycle (M1 + IORQ + 2 automatic wait states); returns the bus byte (mode 0 opcode or mode 2 vector; `$FF` on open bus).
- **`Registers`**: main pairs (`af`, `bc`, `de`, `hl`), alternate pairs (`af_`, `bc_`, `de_`, `hl_`), `ix`, `iy`, `sp`, `pc`, `i`, `r`, `wz` (MEMPTR), `q`, `p`, `iff1`, `iff2`, `im`, and `ei_pending`.
- **Shared debugger register table**: `pub const REGISTERS: [(&str, u32); 18]` and `Registers::values(&self) -> [i64; 18]` expose `AF`, `BC`, `DE`, `HL`, `AF'`, `BC'`, `DE'`, `HL'`, `IX`, `IY`, `SP`, `PC`, `I`, `R`, `WZ`, `IM`, `IFF1`, `IFF2` in a single canonical order used by both `nephrite` and `endou`.
- **Power-on vs zeroed state**:
  - `Z80::new()` zero-initializes all registers (used by `nephrite`).
  - `Z80::power_on()` clears `pc`, `i`, `r`, `iff1`, `iff2`, and `im` (per Zilog reset behaviour) and initializes `af`, `bc`, `de`, `hl`, `af_`, `bc_`, `de_`, `hl_`, `ix`, `iy`, `sp`, and `wz` to `$FFFF` (used by `endou`).
- **`Step`**: `Instruction`, `Interrupt(u32)`, `Halted`, `Observed(u32)`.

### Interrupts, `HALT`, and Hardware Quirks

Instruction boundaries check events in this order: NMI edge, INT (when `iff1` is set and `ei_pending` is false), `HALT` NOP repeat, observer stop, then instruction fetch.

| Event | Bus Activity | T-States |
|---|---|---|
| NMI | M1 at PC (opcode ignored) + 1 internal T-state; push PC high then low; jump to `$0066` | 11 |
| INT (mode 0) | Acknowledge cycle; execute single-byte bus opcode (e.g. `RST` executes its 5 remaining T-states) | 13 (`RST`) |
| INT (mode 1) | Acknowledge cycle + 1 internal T-state; push PC; jump to `$0038` | 13 |
| INT (mode 2) | Acknowledge cycle + 1 internal T-state; push PC; read 16-bit vector from `(I << 8) | bus_byte` | 19 |
| `HALT` | M1 at PC without advancing PC; `R` increments each repeat | 4 per step |

- **IFF1/IFF2 and `HALT`**: NMI clears `iff1` while preserving `iff2` (restored by `RETN`); INT clears both. Accepting any interrupt leaves `HALT`, sets `WZ` to the handler address, and increments `R` for the acknowledge M1 cycle. Mode 0 models single-byte bus instructions only (Genesis, Master System, and Game Gear use mode 1).
- **NMOS `LD A,I` / `LD A,R` interrupt fault**: `p` records whether the previous instruction was `LD A,I` or `LD A,R`; if INT is accepted immediately afterward, `P/V` in `F` is cleared.
- **Undocumented instructions and flags**: Full support for `IXH`/`IXL`/`IYH`/`IYL`, `SLL`, `DD CB` / `FD CB` dual-destination bit/shift forms, `IN (C)`, `OUT (C),0`, `ED` NOPs, `WZ` (MEMPTR), and `Q` (undocumented `X`/`Y` flags in `SCF`/`CCF` computed from `(Q ^ F) | A`).
- **Prefix and register-load `Q` rules**: `DD`/`FD` prefixes, `POP AF`, and `EX AF,AF'` leave `Q` at zero.
- **Repeating block instructions**: `LDIR`/`LDDR`, `CPIR`/`CPDR`, `INIR`/`INDR`, and `OTIR`/`OTDR` on a repeating iteration rewind `PC` to the instruction, set `WZ` to `PC + 1`, take `X`/`Y` from bits 13 and 11 of `PC`, and for repeating I/O instructions adjust `H` and `P/V` according to the carry of the counter and `B`.
- **`HALT` PC**: Executing `HALT` advances `PC` past the opcode while `halted` holds execution on repeated NOP fetches until an interrupt occurs.

### State Block, Disassembler, and `asm` Feature

- **State (`src/state.rs`)**: Fixed 35-byte layout (`Main`, `Alternate`, `Index`, `Sp`, `Pc`, `I`, `R`, `Wz`, `Q`, `P`, `Iff1`, `Iff2`, `Im`, `EiPending`, `Halted`).
- **Disassembler (`src/disasm.rs`)**: Emits Zilog syntax with `$` hexadecimal (`LD A,(IX-$05)`, `JR $0200`, `RST $38`), undocumented forms (`SLL`, `IXH`, `SET 0,(IY-$01),B`, `IN (C)`, `OUT (C),0`), and `DB $ED,$xx` for `ED` NOPs. Redundant `DD`/`FD` prefixes are consumed with their instruction.
- **Assembler (`src/asm.rs`, feature `asm`)**: Accepts Zilog syntax for all documented and undocumented instructions, canonical shortest encodings (`LD HL,(nn)` as `$2A`, `IM 0` as `$ED $46`, `NEG` as `$ED $44`, `RETN` as `$ED $45`), `org`, `align`, `db`/`defb`/`defm`, `dw`/`defw`, `ds`/`defs`, and constants via `equ`/`=`/`defl` (`set` is reserved for the Z80 instruction). Used at compile time to assemble `endou`'s open boot ROMs and test programs.

---

## `beryl-sh2`

Hitachi SH-2 (`SH7604`) CPU crate for the Sega 32X (`nephrite`) and future Saturn (`zoisite`) cores.

> [!NOTE]
> `beryl-sh2` is currently an unimplemented stub (`pub const BUILT: bool = false`). `Sh2::step` and `Sh2::step_observed` always return `Err(NotBuilt)`. No SH-2 instructions, cache logic, or on-chip peripherals execute yet.

### Defined Contract

- **External `Bus` trait**:
  - `read(&mut self, address: u32, size: Size) -> u32` and `write(&mut self, address: u32, size: Size, value: u32)` with `Size::{Byte, Word, Long}` for accesses that miss the on-chip cache and on-chip peripherals.
  - `idle(&mut self, cycles: u32)` for internal stalls, multiplier/divider latency, and cache hits.
  - `interrupt_level(&mut self) -> u8` (`IRL3–IRL0`, `0` = none) and `acknowledge(&mut self, level: u8) -> Option<u8>` (external vector or `None` for auto-vector).
- **On-chip boundary**: The 4 KiB 4-way cache and SH7604 on-chip modules (DMAC, FRT, WDT, DIVU, INTC, SCI, BSC) belong inside `beryl-sh2`; the console bus only services off-chip transfers.
- **`Registers`**: `r: [u32; 16]`, `sr`, `gbr`, `vbr`, `mach`, `macl`, `pr`, `pc` (all `u32`).
- **`Sh2`**: Holds `regs: Registers` and `slave: bool` (`Sh2::new(slave)`) to distinguish master and slave CPUs in dual-SH-2 machines. `Step` defines `Instruction`, `Exception(u8)`, and `Sleeping`.

---

## `beryl-asm`

Zero-dependency assembler front end shared by `beryl-m68k` and `beryl-z80` behind their optional `asm` features. No console core links `beryl-asm` into its runtime library.

- **Core types**:
  - `Program`: `{ origin: u32, bytes: Vec<u8>, symbols: BTreeMap<String, i64> }`.
  - `AsmError`: `{ line: usize, message: String }` (1-based line number; `0` for whole-program errors).
  - `Expr`: AST supporting integer literals, symbols, current PC (`*`, or `$` when not starting a hex number), unary `-`/`~`/`!`, and binary operators in C precedence (`* / %`, `+ -`, `<< >>`, `&`, `^`, `|`, and comparisons `==`, `!=`, `<>`, `<=`, `>=`, `=`, `<`, `>`).
- **Literals and syntax**: Decimal, `$` and `0x` hexadecimal, `%` binary, up to 4-byte character constants, and `'` permitted inside identifiers after an alphanumeric character (supporting Z80 `AF'`).
- **Scoping and directives**: Global labels, local labels (`.name` and `@name`) scoped to the preceding global label, processor-chosen constant keywords (`equ`/`=`/`set` for 68000; `equ`/`=`/`defl` for Z80), `if`/`else`/`endif` conditionals over constants and caller-supplied defines, `include` via a caller-supplied reader callback, and line splitting into label, mnemonic, and operand list.
