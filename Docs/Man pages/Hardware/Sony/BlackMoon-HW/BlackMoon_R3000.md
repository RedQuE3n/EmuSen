# BlackMoon R3000A (`blackmoon-r3000`)

Crate path: `EmuSen/Cores/Sony/BlackMoon-HW/blackmoon-r3000/`.
Implements the MIPS I R3000A processor used as the Sony PlayStation CPU (`diamond`, default `PRId = 0x0000_0002`) and the PlayStation 2 I/O Processor (`sapphire`, `PRId = 0x0000_001F`). `BUILT = true`.

## Public Interface

- **`R3000`**: holds `regs: Registers`, `hilo_ready: u64` (bus cycle when `HI`/`LO` become readable), `prid: u32`, and debugger metadata (`processor: usize`, `space: u32`).
  - `step(&mut self, bus)` — runs one instruction, interrupt, or exception with `Unobserved`.
  - `step_observed(&mut self, bus, observer)` — runs one step with `emusen_native::debug::Observer` reporting instruction boundaries, data loads (`Observer::read`), data stores (`Observer::wrote`), calls, exceptions, and returns.
  - `run(&mut self, bus, limit) -> (u32, Step)` — executes up to `limit` steps until a non-instruction `Step`, `!bus.running()`, or `bus.stop_before(pc)` ends the run.
  - `load_ahead(&self, word) -> Option<(u32, u32)>` — predicts the start address and byte count that the instruction `word` at `pc` would load without mutating CPU state.
- **`Size`**: `Byte` (1), `Half` (2), `Tribyte` (3), and `Word` (4).
  - Unaligned partial loads/stores (`LWL`, `LWR`, `SWL`, `SWR`) transferring 3 bytes issue a single `Size::Tribyte` access at the lowest byte address rather than splitting into a halfword and a byte.
  - Byte and halfword stores pass the full 32-bit register value to `Bus::write`; the bus decides how byte lanes reach devices.
- **`Bus` trait**:
  - Required: `fetch`, `read`, `write`, `isolated_read`, `isolated_write`, `idle`, `cycles`, `interrupt`, `cop2_data`, `set_cop2_data`, `cop2_control`, `set_cop2_control`, `cop2_command`.
  - Optional (with defaults): `fetch_decoded` (allows a bus with a decoded I-cache to return `(word, emusen_mips::Op)` directly), `bus_error` (triggers `IBE` after fetch or `DBE` after data access), `running` (allows `R3000::run` to stay in its fast loop while no bus event is due), `stop_before(pc)` (lets a core stop a run at a specific PC), `const MOVES: bool = false`, and `moved(Move)`.
- **`COP0`, `dialect`, `DEBUG_REGISTERS`, `reference`**:
  - `DEBUG_REGISTERS` exports 45 register names: the 32 GPRs (`zero`..`ra`), `hi`, `lo`, `pc`, and the 10 active COP0 registers (`SR`, `CAUSE`, `EPC`, `BadVaddr`, `TAR`, `DCIC`, `BPC`, `BDA`, `BPCM`, `BDAM`).
  - `dialect(cop2)` constructs the `emusen_mips::Dialect` combining `MIPS_I`, `COP0`, and an optional console-supplied COP2 descriptor.
  - `reference(word, pc)` extracts static call/read/write targets for disassemblers and debuggers.

## Execution Order and Architectural Delays

### Step Pipeline

Each `step` runs in this order:
1. Sample `Bus::interrupt()` into `CAUSE.IP2` (bit 10). `IP2` tracks the external line regardless of `SR.IEc` or `SR.IM2`.
2. Check pending interrupts using `SR` and `CAUSE` software bits — or the previous instruction's `interrupt_view` if the immediately preceding instruction was `MTC0` to `SR` (an `MTC0` to `SR` reaches the interrupt check one instruction late, so one following instruction executes before an enabled interrupt fires). If an interrupt is taken, the interrupted instruction is fetched first; if it is a COP2 command (`COP2 imm25`) with `SR.CU2` set, the command is still forwarded to `Bus::cop2_command` before entering the exception handler with `EPC` pointing at that instruction.
3. Call `observer.before(...)`; if it requests a stop, return `Step::Observed(reasons)` without running the instruction.
4. Check fetch address error (`pc & 3 != 0`, or kernel address `pc >= 0x8000_0000` in user mode `SR.KUc == 1`); if faulted, raise `AdEL` without fetching.
5. Check the COP0 debug unit PC breakpoint (`BPC`/`BPCM`).
6. Fetch (via `Bus::fetch_decoded`) and execute; if `Bus::bus_error()` is true after the fetch, raise `IBE`.

### Branch and Load Delay Slots

- **Branch delay**: `BranchDelay { slot, taken, target }` holds the pending branch state. Targets and link return addresses are computed relative to the delay-slot address (`pc + 4`, or the earlier branch's target when a branch executes inside another branch's delay slot). `BLTZAL` and `BGEZAL` always write `ra`, comparing `rs` before linking.
- **Load delay**: `LoadDelay { register, value }` commits after the next instruction reads its operands:
  - `MFC0`, `MFC2`, and `CFC2` use the same one-instruction load delay as memory loads.
  - If the following instruction writes to the same register as a pending load, the instruction's write wins.
  - If the following instruction starts another load targeting the same register, the first load is cancelled and never lands; the register retains its prior value until the second load lands.
  - `LWL` and `LWR` merge directly with any pending load value for their target register, allowing back-to-back `LWL`/`LWR` pairs without an extra delay.
  - An exception lets any pending load land before exception entry.
  - Loads targeting `R0` are tracked through the delay slot and discarded on landing.

### Fast Run Loop (`R3000::run`)

When a boundary is quiet (`DCIC.DE == 0`, no pending `MTC0 SR` `interrupt_view`, and no interrupt active), `R3000::run` executes common instructions (`ALU`, shifts, branches/jumps, and standard loads/stores) on a tight path that skips full boundary checks while checking the external interrupt line, PC alignment/segment, and `bus.running()` / `bus.stop_before(pc)` after each instruction. Any change to `SR`/`CAUSE`/`DCIC`, interrupt line edge, fetch fault, or rare opcode falls back to the full step path with identical architectural behavior.

## Multiply/Divide Timing

`MULT`, `MULTU`, `DIV`, and `DIVU` compute `HI` and `LO` immediately and record `hilo_ready = bus.cycles() + latency`. When `MFHI` or `MFLO` runs before `hilo_ready`, it calls `bus.idle(hilo_ready - now)`.

| Operation | Operand `rs` Range | Latency (cycles) |
|---|---|---|
| `MULTU` | `0x0000_0000..=0x0000_07FF` | 6 |
| `MULTU` | `0x0000_0800..=0x000F_FFFF` | 9 |
| `MULTU` | `0x0010_0000..=0xFFFF_FFFF` | 13 |
| `MULT` | `0x0000_0000..=0x0000_07FF` or `0xFFFF_F800..=0xFFFF_FFFF` | 6 |
| `MULT` | `0x0000_0800..=0x000F_FFFF` or `0xFFF0_0000..=0xFFFF_F7FF` | 9 |
| `MULT` | `0x0010_0000..=0x7FFF_FFFF` or `0x8000_0000..=0xFFEF_FFFF` | 13 |
| `DIV` / `DIVU` | Any | 36 |

Integer division never traps. Division by zero produces `LO = 0xFFFF_FFFF` (unsigned or non-negative signed dividend) or `1` (negative signed dividend) with `HI = rs`; signed `0x8000_0000 / -1` produces `LO = 0x8000_0000`, `HI = 0`.

## Coprocessor 0, Exceptions, and Debug Unit

### COP0 Registers

| Register | Name | Read (`MFC0`) | Write (`MTC0`) |
|---|---|---|---|
| `cop0r3` | `BPC` | Full word | Full word |
| `cop0r5` | `BDA` | Full word | Full word |
| `cop0r6` | `TAR` | Full word | Ignored |
| `cop0r7` | `DCIC` | Full word | Mask `0xFF80_F03F` |
| `cop0r8` | `BadVaddr` | Full word | Ignored |
| `cop0r9` | `BDAM` | Full word | Full word |
| `cop0r11` | `BPCM` | Full word | Full word |
| `cop0r12` | `SR` | Full word | Mask `0xF247_FF3F` |
| `cop0r13` | `CAUSE` | Full word | Software interrupt bits `8..=9` (`0x0000_0300`) only |
| `cop0r14` | `EPC` | Full word | Ignored |
| `cop0r15` | `PRId` | `R3000::prid` | Ignored |
| `cop0r0..r2, r4, r10` | Unmapped | Raises `RI` | Ignored |
| `cop0r16..r31` | Unimplemented | Reads `0` (also permitted in user mode) | Ignored |

`CFC0` and `CTC0` raise `RI`. For `COP0` commands (bits `0..=5`): `0x10` is `RFE` (pops the `SR` mode stack `KU`/`IE`, leaving `KUo`/`IEo` unchanged); TLB encodings (`TLBR`, `TLBWI`, `TLBWR`, `TLBP`) raise `RI`; all other `COP0` command encodings act as no-ops.

### Coprocessor Usability

- `MFC0`/`MTC0`/`CFC0`/`CTC0`/`COP0` require kernel mode or `SR.CU0 == 1` (except `MFC0` of `cop0r16..r31`, which returns `0` without faulting).
- `LWCz`, `SWCz`, `BCzF`, and `BCzT` check `SR.CUz` in both kernel and user mode (raising `CpU` if `CUz == 0`).
- `BCzF` is always taken and `BCzT` is never taken when `CUz == 1`.
- Absent coprocessors (`COP1`, `COP3`) with their `CU` bit set treat `MFC`/`CFC` as loading `0`, `LWC` as reading and discarding one word from the bus, and `SWC0`/`SWC1`/`SWC3` as storing `0` on the bus (subject to alignment, debug breaks, and bus errors).

### Exception Entry and `CE`/`TAR` Rules

- General exception vector: `0x8000_0080` (`SR.BEV == 0`) or `0xBFC0_0180` (`SR.BEV == 1`).
- Debug-unit breakpoint vector: `0x8000_0040` (`SR.BEV == 0`) or `0xBFC0_0140` (`SR.BEV == 1`).
- `EPC` receives the faulting instruction address, or `branch_pc` with `CAUSE.BD = 1` when faulting in a delay slot.
- Whenever `CAUSE.BD` is set, `TAR` is written on every branch: the branch target if taken, or the fall-through address (`slot + 4`) if not taken. `CAUSE.BT` is set only when the branch is taken.
- `CAUSE.CE` (bits `28..=29`) is loaded from the faulting instruction's opcode bits `26..=27`, except on fetch address errors, `DBE`, and debug-unit breaks, where `CE` is `0`.
- `BadVaddr` is updated only on `AdEL` and `AdES`.

### Cache Isolation and Debug Unit (`DCIC`)

- When `SR.IsC == 1`, data loads/stores go to `Bus::isolated_read`/`isolated_write` instead of memory, and are omitted from debugger `Observer::read`/`wrote` and `Bus::moved` reports. Instruction fetches are unaffected.
- PC and data breakpoints match when `DCIC.DE == 1`, the corresponding enable bits (`PCE`, or `DAE` + `DR`/`DW`) are set, and the masked address matches (`(addr ^ BPC) & BPCM == 0` or `(addr ^ BDA) & BDAM == 0`).
- `KD` vs `UD` selection tests bit 31 of the compared address (`KD` when bit 31 is set, `UD` when bit 31 is clear).
- For data breakpoints, `LWL` and `SWL` compare the word-aligned bus address (`addr & !3`); all other loads/stores compare the effective address.
- Precedence on data accesses: debug-unit status bits (`DB`, `DA`, `R`/`W`) are evaluated and latched first; address errors (`AdEL`/`AdES`) take precedence over trapping to the debug vector (while keeping the latched `DCIC` bits); trapping debug breaks (`DCIC.TR == 1`, `ExcCode = 9`) take precedence over `DBE`.
- Trace (`DCIC.TE`) and jump-redirection bits are not modelled.

## Value-Move Side-Channel (`Bus::MOVES`)

When `B::MOVES == true`, the processor reports vertex-relevant data movement through `Bus::moved(Move)`:

| Variant | Triggering Instructions | Timing |
|---|---|---|
| `Move::Load` | `LB`, `LBU`, `LH`, `LHU`, `LW`, `LWL`, `LWR` | After bus read succeeds (omitted on `DBE` or `IsC`) |
| `Move::Store` | `SB`, `SH`, `SW`, `SWL`, `SWR` | Before `Bus::write` so receiving devices (e.g. GP0) see the source register immediately |
| `Move::FromCop2` | `MFC2` | After COP2 read |
| `Move::ToCop2` | `MTC2` | Before COP2 write |
| `Move::LoadCop2` | `LWC2` | After bus read |
| `Move::StoreCop2` | `SWC2` | Before bus write |
| `Move::Copy` | `ADDU`/`ADD`/`SUBU`/`SUB`/`OR`/`XOR` with `R0`, `ADDIU`/`ADDI`/`ORI`/`XORI` with `0`, shifts by `0` | On instruction completion |
| `Move::Shift` | `SLL`, `SRL`, `SRA` with constant shift `1..=31` (`Shift::{Left, Right, Arithmetic}`) | On instruction completion |

Moves targeting `R0` and accesses under `SR.IsC` are never emitted.

## State Block Layout

`R3000` serializes a fixed **209-byte** block via `emusen_native::State`:

| Offset | Size | Type | Field |
|---:|---:|---|---|
| 0 | 128 | `u32[32]` | `R` |
| 128 | 4 | `u32` | `Hi` |
| 132 | 4 | `u32` | `Lo` |
| 136 | 4 | `u32` | `Pc` |
| 140 | 40 | `u32[10]` | `Cop0` (`SR`, `CAUSE`, `EPC`, `BadVaddr`, `TAR`, `DCIC`, `BPC`, `BDA`, `BPCM`, `BDAM`) |
| 180 | 6 | `bool, bool, u32` | `Branch.{Slot, Taken, Target}` |
| 186 | 6 | `bool, u8, u32` | `Load.{Pending, Register, Value}` |
| 192 | 8 | `u64` | `HiLoReady` (absolute bus cycle) |
| 200 | 9 | `bool, u32[2]` | `InterruptView.{Held, Values}` |
