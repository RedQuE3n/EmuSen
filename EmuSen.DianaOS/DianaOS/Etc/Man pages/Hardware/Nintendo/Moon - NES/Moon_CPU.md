# Moon (NES) — CPU (2A03 / 6502)

Covers `Cores/Nintendo/Moon - NES/Cpu/`: the 6502 core (`Cpu.cs`), the bus contract (`ICpuBus.cs`), addressing modes (`Cpu.AddressModes.cs`), the 256-case dispatch (`Cpu.OpcodeTable.cs`), and the operation bodies (nine `Cpu.Opcodes.*.cs` files). See `Man pages/Hardware/README.md` for what this doc set is.

This is the first component of the second core, and the first real test of whether the toolchain built around `IDebugTarget`/`ISingleStepTarget` is genuinely core-agnostic or only asserted to be. §7 records what that test has actually shown so far — which, as of this writing, is the CPU and nothing else. There is no PPU, no APU, no mapper and no `ICore` implementation yet.

---

## 1. Structure

`Cpu` is a `partial class` split by concern, the same way `Venus`'s is:

| File | Covers |
|---|---|
| `Core/Cpu.cs` | Registers, flags, `Step()`, interrupt entry, stack and bus primitives |
| `Core/ICpuBus.cs` | The two-method bus contract (§2) |
| `Opcodes/Cpu.AddressModes.cs` | Every `AddrXxx` effective-address computation, including its dummy reads |
| `Opcodes/Cpu.OpcodeTable.cs` | The 256-case `Dispatch()` switch and the mnemonic table |
| `Opcodes/Cpu.Opcodes.Arithmetic.cs` | ADC/SBC, the compares, INC/DEC on memory |
| `Opcodes/Cpu.Opcodes.Logical.cs` | AND/ORA/EOR, BIT |
| `Opcodes/Cpu.Opcodes.Shift.cs` | ASL/LSR/ROL/ROR, value helpers and memory forms |
| `Opcodes/Cpu.Opcodes.LoadStoreTransfer.cs` | LDA/LDX/LDY and TXS |
| `Opcodes/Cpu.Opcodes.Branch.cs` | The shared `Branch()` helper |
| `Opcodes/Cpu.Opcodes.Stack.cs` | PHA/PHP/PLA/PLP |
| `Opcodes/Cpu.Opcodes.Flags.cs` | The flag setters, including the delayed-I path |
| `Opcodes/Cpu.Opcodes.System.cs` | BRK/RTI/JSR/RTS, JMP indirect, JAM |
| `Opcodes/Cpu.Opcodes.Undocumented.cs` | Everything in §6 |

Register transfers and the simple stores are written inline in the dispatch switch rather than given `Op` methods — on a 6502 they are one statement each, and a wrapper would be longer than the body.

---

## 2. Every cycle is a bus access

`ICpuBus` has exactly two methods, `Read` and `Write`. There is no `Tick()`, no `Idle()`, and no per-opcode cycle table anywhere in this core.

That is not a simplification — it is what the hardware does. A real 6502 drives the address bus on **every** cycle of every instruction. Cycles that do no useful work are not idle; they are redundant reads of whatever address happened to be on the bus, which is why a 6502 datasheet's cycle-by-cycle tables list an address for every line. Modelling those reads is therefore both more accurate *and* cheaper than modelling them as abstract time, because the cycle count stops being a number that has to be maintained by hand and becomes a consequence of the code.

### 2.1 Why there is no cycle table

`Venus`'s CPU carries `OpcodeCycles[256]` plus a `_addrModeExtraCycles` side channel for the penalties the static table cannot express. That design is right for a 65816, whose bus timing depends on *where* the access lands (`GetAccessSpeedCycles`) as much as on the opcode.

On a 6502 every access is one cycle, full stop, so the same information is already present in the instruction's control flow. A table would be a second, independently-maintained copy of a fact the code already states — and the copy is the one that goes stale. `Step()` counts accesses and returns the total.

This is the property that made §7's trace validation possible at all: because cycle counts are emergent, a vector suite that ships per-cycle bus activity checks the addressing modes and the cycle counts *at the same time*, and there is no third place for the two to disagree.

### 2.2 `LastInstructionPC`

Snapshotted at the top of `Step()`, before the opcode fetch moves `PC`. Same rationale as `Venus`'s `LastInstructionPC`/`LastInstructionPB` (see `Venus_CPU.md` §2), which was a real found-the-hard-way bug there: any debug consumer asking "which instruction did this" needs the address the instruction *started* at, not the live `PC`, which has already advanced past the operand bytes by the time side effects run.

---

## 3. Addressing modes, and the dummy reads that define their cycle counts

### 3.1 Indexed reads pay for a page cross; indexed writes always pay

`AddrAbsoluteIndexed` and `AddrIndirectIndexed` take an `alwaysFixup` flag.

The chip adds the index to the low byte first and puts that half-computed address on the bus immediately, before it knows whether the addition carried. If it did carry, the high byte needs fixing, and the access at the un-fixed address has already happened — that wasted access is the "+1 cycle on page cross".

For a **read**, the chip can skip the fixup cycle when there was no carry, because the half-computed address was correct. For a **write** or a **read-modify-write** it cannot: a write to the wrong address is not recoverable by noticing afterwards, so the chip always spends the cycle, and always reads (never writes) at the un-fixed address. Hence `STA abs,X` is a flat 5 cycles while `LDA abs,X` is 4 or 5.

Passing `alwaysFixup: true` for every store and RMW form is the entire difference, and getting it backwards is invisible to a final-state check and immediately visible in a bus trace.

### 3.2 Zero-page indexing wraps

`AddrZeroPageIndexed` returns `(byte)(pointer + index)`, and `AddrIndexedIndirect`/`AddrIndirectIndexed` read the pointer's high byte from `(byte)(pointer + 1)`. The index addition and the pointer fetch both stay inside page zero — `LDA ($FF),Y` reads its pointer from `$FF` and `$00`, not `$FF` and `$0100`.

The dummy read at the un-indexed zero-page address is what makes `zp,X` four cycles instead of three.

### 3.3 Read-modify-write writes twice

`ReadModifyWriteFetch` reads the operand and then writes the **unmodified** value straight back, before the caller writes the modified one. Real hardware does this because the ALU result is not ready in time to skip the intermediate write.

On a flat RAM this is invisible. On a real NES it is emphatically not: an `INC $2007`-style access to a hardware register hits that register twice, and the double write is the documented cause of several mapper-IRQ behaviours. It is modelled here so it is already right when a bus with real hardware behind it exists.

### 3.4 Stack pulls read the old top first

`PullWithDummy` reads `$0100|S` *before* incrementing `S`, then reads the real value at `$0100|(S+1)`. The first read is the chip idling on the stack address while the pointer increments. This is why `PLA` is four cycles and `PHA` is three.

### 3.5 `JMP ($xxFF)` does not carry

`OpJMPIndirect` computes its high-byte address as `(pointer & 0xFF00) | ((pointer + 1) & 0x00FF)`. The pointer's low byte increments without carrying into the high byte, so `JMP ($02FF)` takes its low byte from `$02FF` and its high byte from `$0200`, not `$0300`.

This is the most famous 6502 bug and it is load-bearing: real code relies on it, and "fixing" it breaks games.

---

## 4. Flags

### 4.1 `B` and `U` are not real bits

The status register physically has six flip-flops. Bits 4 (`B`) and 5 (`U`) do not exist as storage — they only ever appear as values *on the bus* when `P` is pushed, and the value pushed depends on what caused the push:

| Cause | Pushed value |
|---|---|
| `PHP`, `BRK` | `P \| 0x30` — both bits set |
| `NMI`, `IRQ` | `(P & ~0x10) \| 0x20` — `B` clear, `U` set |
| `PLP`, `RTI` (pull) | `(pulled \| 0x20) & ~0x10` |

`P` is therefore held internally with `U` set and `B` clear, and only `OpPHP`/`OpBRK`/`ServiceInterrupt` ever deviate.

This is not reasoning from a datasheet — it was read directly off the vectors before the flag code was written. `28.json` (PLP) pulls `0x88` and reports a final `p` of `0xA8`; `08.json` (PHP) holds `p` at `0xA4` and puts `0xB4` on the bus. Those two data points pin all three rows above.

### 4.2 `D` latches but does nothing

`SED`/`CLD` set and clear the flag, `PHP` pushes it and `PLP` restores it, and `ADC`/`SBC` ignore it completely. The 2A03 is a 6502 with the decimal-mode circuitry disabled at the mask level — the flag survives, the behaviour does not.

`OpSBC` is written as `OpADC((byte)~operand)`, which is exact for binary mode and would be wrong if decimal mode existed here. It does not, so it is exact.

Note this is the mirror image of `Venus`'s gap: the 65816 there *does* have decimal mode and does not implement it (`Venus_CPU.md`, and the "known gaps" list in `README.md`). Here there is nothing to implement.

---

## 5. Reset and interrupts

### 5.1 Reset

`Reset()` zeroes `A`/`X`/`Y`, sets `P` to `I|U`, sets `S` to `0xFD`, and reads `PC` from `$FFFC`. The stack pointer's value is not arbitrary: the reset sequence performs three stack *reads* where a real interrupt would push, decrementing `S` three times from an undefined power-on value. `0xFD` is what that produces from the `0x00` a cold chip is usually observed at, and it is what every other implementation and every test ROM assumes.

`Reset()` fetches the vector over the bus, so a caller that cares about cycle accounting pays the real chip's startup reads rather than teleporting.

### 5.2 NMI is an edge, IRQ is a level

`SetNmiLine` latches on a low-to-high transition and stays latched until serviced; `SetIrqLine` records a level and nothing more. This distinction is the entire reason an NES game can miss an IRQ but cannot miss an NMI, and it has to be modelled at the line rather than as a "raise interrupt" call, because whatever asserts IRQ (the APU frame counter, a mapper's scanline counter) is responsible for *holding* it until acknowledged.

### 5.3 `CLI`/`SEI`/`PLP` take effect one instruction late

The real chip polls for interrupts partway through an instruction, before that instruction's final cycle. An instruction whose *only* effect is to write `I` therefore has its write land **after** the poll that decides whether the next instruction is an interrupt sequence. The consequences are counter-intuitive and real:

- An IRQ pending when `SEI` executes **is still taken**, after `SEI` completes.
- An IRQ pending when `CLI` executes **is not** taken immediately; it waits one more instruction.

`SetInterruptDisable` and `OpPLP` therefore stash the new `I` in `_delayedI`/`_hasDelayedI` and leave `P`'s `I` bit alone; `PollInterrupts()` samples the interrupt state and *then* applies the stashed write, at the end of `Step()`. Because the write lands before `Step()` returns, a caller reading `P` afterwards sees the new value — the delay is visible to interrupt dispatch only, which is the point.

### 5.4 Known deviation: the poll is at the instruction boundary

`PollInterrupts()` runs once, after the instruction completes, rather than before the instruction's last cycle where the hardware polls.

For the case the delay actually matters — the flag-writing opcodes above — §5.3's mechanism reproduces the hardware result exactly. For the general case it does not: hardware's mid-instruction poll means a long instruction can latch an interrupt that arrives partway through it, and this model defers that to the next boundary. The visible effect is at most one instruction of latency on IRQ recognition.

This is a **deliberate, documented approximation, and it is not validated by anything** — the `nes6502` suite does not exercise interrupts at all (§7.1). Stated plainly rather than implied: everything in §7 is evidence about the instruction set, and none of it is evidence about this section. Revisit when a PPU exists and sprite-0/NMI timing gives something real to check against.

---

## 6. Undocumented opcodes

The NMOS 6502 decodes all 256 opcodes; the 105 the datasheet does not list still do something, because the same control lines fire for more than one instruction at once. Real NES games use them, so they are implemented rather than trapped.

The stable ones are all combinations: `SLO` is `ASL`+`ORA`, `RLA` is `ROL`+`AND`, `SRE` is `LSR`+`EOR`, `RRA` is `ROR`+`ADC`, `DCP` is `DEC`+`CMP`, `ISC` is `INC`+`SBC`, `SAX` stores `A & X`, `LAX` loads both `A` and `X`. Where both halves would set flags, the second half wins, which is why `OpDCP` does not flag its own decrement.

### 6.1 `ANC`, `ALR`, `SBX`

`ANC` ends with `C` copied from `N` — the AND's own sign bit — which makes it a one-instruction sign extension. `ALR` is `AND` then `LSR A`. `SBX` computes `(A & X) - imm` into `X` and sets `C` like a compare, ignoring `V` and the incoming carry.

### 6.2 `ARR` takes its flags from the shifter, not the adder

`ARR` is `AND` followed by `ROR A`, but `C` and `V` do not come from where either instruction would normally put them: `C` is bit 6 of the *result*, and `V` is bit 6 XOR bit 5 of the result. This falls out of the AND and the rotate racing each other through the adder, and it is the reason `ARR` cannot be written as a composition of the two documented opcodes.

### 6.3 The unstable opcodes

`ANE` (`0x8B`) and `LXA` (`0xAB`) mix the accumulator with a constant produced by the analog behaviour of the chip rather than by any logic: `A = (A | magic) & X & imm` and `A = X = (A | magic) & imm`. The constant varies between chips, temperature and supply voltage.

`UnstableMagic` is **`0xEE`**, and that value was not taken on faith — it was solved for from the vectors before the opcodes were written. `ab.json`'s first case has `A=0x07`, `imm=0x1D`, and a final `A` of `0x0D`; `(0x07 | m) & 0x1D == 0x0D` requires bit 3 set and bit 4 clear in `m`, which admits `0xEE` and rejects `0xFF`. `8b.json` independently agrees.

The `SH*` family (`SHA`/`SHX`/`SHY`/`TAS`) stores its register ANDed with the high byte of the *base* address plus one. When the index crosses a page boundary, the value being stored also **becomes** the high byte of the address it is stored to — `UnstableStore` models both halves, which is why those opcodes need `AddrAbsoluteIndexedUnstable`/`AddrIndirectIndexedUnstable` (which report `baseHigh` and `crossed`) instead of the ordinary indexed modes.

All of these pass §7's vectors, traces included. That is stronger evidence than usual for opcodes described as unstable, and it should still be read as "matches the chip the vectors were generated from".

### 6.4 `JAM`

The twelve `JAM`/`KIL` opcodes halt the chip with the bus pulled low; only reset recovers it.

The vectors are specific about how it gets there, and the first implementation here got it wrong by assuming: a jam is **three** cycles — the opcode fetch, then the byte after it read *twice* — and `PC` ends up back at the opcode, not past it. `OpJAM` performs both dead reads before rewinding `PC` and setting `Jammed`. This was the only thing the full 2.56M-case run rejected (§7.1); all twelve opcodes failed it identically, which is what a single shared cause looks like.

Once `Jammed` is set, `Step()` burns one access per call forever rather than looping inside itself, so a caller driving the CPU toward a cycle deadline still terminates. The vectors step a jam only once, so that part is a modelling choice, not a validated one.

---

## 7. Validation status

### 7.1 What was run

`EmuSen.Tomoe`'s `nes6502` target, against **SingleStepTests/ProcessorTests `nes6502/v1`** — 256 files, 10,000 cases each, 2,560,000 instructions. The data is third-party and is not committed here; fetch it from `https://github.com/SingleStepTests/ProcessorTests`.

```sh
dotnet run -c Release --project EmuSen.Tomoe -- nes6502 /path/to/nes6502/v1
```

The suite covers every opcode including the undocumented and unstable ones. It does **not** cover interrupts, reset, or anything about `SetNmiLine`/`SetIrqLine` — see §5.4.

### 7.2 The trace check is new, and it is the stronger claim

`Venus`'s 65816 and SPC700 adapters check final registers and final memory. That is a real check, but it is blind to an entire class of bug: an addressing mode that reads the right byte from the right address via the wrong number of bus cycles produces an identical final state.

The `nes6502` vectors ship a `cycles` array — every access, in order, with its address, its value and its direction — so this adapter checks that too. `EmuSen/Validation/IBusTraceTarget.cs` is the opt-in: `SingleStepTest.ExpectedTrace` is null for a suite whose loader does not carry one, and `SingleStepTestRunner` skips the comparison unless both the test and the target supply the data. The 65816 and SPC700 paths are therefore untouched and behave exactly as before.

Consequence worth stating: for this CPU, "passes the vectors" means every dummy read in §3, every page-cross fixup, and every cycle count is confirmed against ground truth — not merely the arithmetic. The cycle counts have no separate table to be wrong in (§2.1).

### 7.3 What this says about the core-agnostic claim

The shared rig took this core with one new file implementing `ISingleStepTarget`, one JSON loader, and one line in `EmuSen.Tomoe`'s registry. Nothing in `SingleStepTestRunner` or `ISingleStepTarget` needed changing to accommodate a second CPU architecture; the one addition (§7.2) was to exploit richer source data, not to work around an assumption baked in for the 65816.

That is a genuine result for `ISingleStepTarget`, and it is worth being precise about its scope: it says nothing yet about `IDebugTarget`, which is the much larger core-agnostic claim and has still never been implemented twice. See `Man pages/EmuSen_Core_Gameplan.md` §7.

### 7.4 `Nes6502CpuTests.cs` — what the vectors cannot cover

`EmuSen.WiseMan/Cores/Nes6502CpuTests.cs` is deliberately *not* a smaller copy of the vector run. The vector data is third-party and uncommitted, so the repo's own suite cannot depend on it; what that file holds is the part §7.1 leaves unproven, plus a few cycle rules worth stating where someone will actually read them:

- **Interrupts.** `nes6502/v1` never asserts an interrupt line, so `SetNmiLine`/`SetIrqLine`, NMI edge-latching, IRQ masking, the delayed-`I` behaviour of §5.3, and the status byte pushed on entry (§4.1) have no ground-truth coverage at all. Those tests are the only check any of it has.
- **Reset.** Also unexercised by the vectors — §5.1's stack pointer and vector fetch.
- **Cycle rules worth naming**: the page-cross rule and its store/RMW asymmetry (§3.1), the double write of an RMW (§3.3), the `JMP ($xxFF)` wrap (§3.5), zero-page and pointer wrapping (§3.2), branch costs, and `JAM` (§6.4).

**One trap, met the first time these were written.** Cleared memory reads back as `0x00`, which decodes as `BRK` — and `BRK` vectors through `$FFFE`, the same address as an IRQ. A test that asserts "the IRQ was *not* taken" by checking `PC != $9000` will therefore pass or fail for reasons having nothing to do with interrupts, because a `BRK` off the end of the code under test lands on exactly that address. Every case in that file NOP-fills the whole path the CPU can walk, and asserts an exact `PC` rather than an inequality.
