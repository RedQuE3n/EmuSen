# Venus (SNES) — CPU (65816)

Covers `Cores/Nintendo/Venus - SNES/Cpu/`: the 65816 core (`Cpu.cs`), addressing modes (`Cpu.AddressModes.cs`), the execution opcode table (`Cpu.OpcodeTable.cs`), opcode implementations (eight `Cpu.Opcodes.*.cs` files, one per instruction category — §1), and the standalone disassembler. See `Man pages/Hardware/README.md` for what this doc set is and how it relates to the rest of `Man pages/`.

The disassembler's own quirks and caveats are documented in depth in `Man pages/EmuSen_Debugging_Tools_Reference_v5.md` §3.7 (it's a debug-tooling consumer as much as a CPU component) — this page only covers what's specific to it as a *second, independent* opcode table living alongside the execution one.

---

## 1. Structure

`Cpu` is a `partial class` split by concern, not by convenience: `Cpu.cs` holds core state and the fetch/execute loop, `Cpu.AddressModes.cs` holds every `AddrXxx` addressing-mode delegate, `Cpu.OpcodeTable.cs` builds the 256-entry dispatch table, and every `OpXxx` operation lives in one of eight `Cpu.Opcodes.*.cs` files, split by instruction category the way most 65816 references group them:

| File | Covers |
|---|---|
| `Cpu.Opcodes.System.cs` | NOP/unknown-opcode fallback, BRK/COP/RTI, WAI/STP, MVN/MVP |
| `Cpu.Opcodes.Stack.cs` | PHP/PLP, PHA/PLA, PHX/PLX, PHY/PLY, PHB/PLB, PHK, PHD/PLD, PEA/PEI/PER |
| `Cpu.Opcodes.LoadStoreTransfer.cs` | LDA/LDX/LDY, STA/STX/STY/STZ, every T__ register transfer, XBA |
| `Cpu.Opcodes.Arithmetic.cs` | INC/DEC (A, memory, X, Y), CMP/CPX/CPY, ADC/SBC |
| `Cpu.Opcodes.Logical.cs` | ORA/AND/EOR, TRB/TSB, BIT/BIT-immediate |
| `Cpu.Opcodes.Shift.cs` | ASL/LSR/ROL/ROR, accumulator and memory forms |
| `Cpu.Opcodes.Branch.cs` | JMP/JML/JSR/JSL/RTS/RTL/BRA/BRL, every conditional branch |
| `Cpu.Opcodes.Flags.cs` | CLC/SEC/CLI/SEI/CLV/CLD/SED, REP/SEP, XCE |

This used to be one 1229-line `Cpu.Opcodes.cs` — split for the same reason `DebugCommandProcessor` and the PPU renderer were: a monolith the size of "every CPU operation in one file" stopped being something you could hold in your head at once, and none of the split boundaries needed to touch any actual instruction logic to fix that. `Cpu.OpcodeTable.cs` and `Cpu.AddressModes.cs` were deliberately **not** split the same way — see the note at the end of this section.

Each `Instruction` entry (built once, in `Cpu.OpcodeTable.cs`) pairs an addressing-mode delegate (computes an effective address, may consume operand bytes) with an operate delegate (does the actual work at that address) — mirroring how real 6502/65816 opcode references describe instructions as (addressing mode × operation).

**Why `Cpu.OpcodeTable.cs` stays one file.** Unlike the `Op*` method bodies, its 256 entries are already terse one-liners — the file is long because there are 256 opcodes, not because any individual entry is hard to read. It's also organically grown in registration order (see its own inline comments — "fills gaps... after the ROM halted on 0x9E," "0xE6 is what the ROM halted on," etc.), not opcode-category order, and this exact table got a dedicated verification pass against oxyron.de (§7) before being trusted. Re-deriving a per-category split for all 256 entries would risk introducing a transcription error into a table that's already been verified once — real risk for a benefit this table doesn't actually need, since it's already scannable as a flat list. `Cpu.AddressModes.cs` (243 lines) was left alone for a simpler reason: it's already a single, cohesive category (every `AddrXxx` delegate) and isn't especially large to begin with.

---

## 2. `LastInstructionPC`/`LastInstructionPB` — why these exist alongside `PC`/`PB`

`PC`/`PB` are live: `Fetch8()` advances `PC` past the opcode/operand bytes almost immediately, well before that instruction's actual side effects (a memory write, say) happen. Debug tooling that wants "which instruction just wrote this value" (e.g. `Dma.cs`'s `LogSourceAddrWrite`, or a write watch's context string) needs the PC/PB *as they were when this instruction started*, not the live ones.

**This was a real, found-the-hard-way bug**, not a preemptive design choice: every DMA source-address write trace was showing the *same* next-instruction bytes regardless of which actual `STA` had just fired, because by the time the write's side effect ran, `PC` had already moved on to the next instruction. `LastInstructionPC`/`PB` are snapshotted at the top of `Step()`, before `Fetch8()` runs, specifically to fix this.

---

## 3. NMI / IRQ entry sequences

Both `Nmi()` and `Irq()` run the real 65816 interrupt sequence: push `PB` (native mode only — emulation mode's stack can't hold it), push `PC`, push `P`, clear `D`, set `I`, then jump to the vector. The handler is expected to end with `RTI`.

- **NMI is non-maskable** — always services regardless of the `I` flag, and always wakes a WAI-halted CPU (see §4). Vector: `$FFEA` native / `$FFFA` emulation.
- **IRQ is maskable** — only actually jumps to the vector if `I` is clear, matching real hardware holding off IRQs while that flag is set. `Irq()` returns whether it actually fired, so the caller (which decides *when* to attempt this, based on H/V-IRQ timer conditions — see `Man pages/Hardware/Venus_Memory.md` §4) knows whether to also acknowledge the pending condition. Vector: `$FFEE` native / **`$FFFE` emulation — shares BRK's vector**, a real, documented 6502/65816 quirk, not a typo.
- **WAI wakes on any interrupt condition, masked or not** — only whether it's actually *serviced* (jumps to the vector) respects the `I` flag. Both `Nmi()` and `Irq()` clear `_waitingForInterrupt` unconditionally, before any masking check.

---

## 4. WAI / STP

Verified via 6502.org/apprize.best's "Programming the 65816" excerpt and the NESDev WAI thread.

- **WAI** halts instruction fetch/execute until any interrupt condition wakes the CPU (see §3's masking note — NMI always services, IRQ wakes but only services if unmasked; if masked, execution just resumes at the instruction after WAI with nothing serviced).
- **STP** stops the CPU entirely. Real hardware only wakes on a hardware RESET — modeled as `Cpu.Reset()` clearing `_stopped`; no other instruction or interrupt clears it. This matches real hardware, where STP is meant for power-down, not a resumable pause.

Neither fetches or executes anything while active — `Step()` just returns idle cycles (3 for STP, 2 for WAI) so the caller's per-scanline cycle budget still advances.

---

## 5. Addressing modes — notes on the less obvious ones

- **`(sr,S),Y`** (`AddrStackRelativeIndirectY`): the 16-bit pointer lives on the stack (always Bank 0, same as plain stack-relative), combined with `DB` and indexed by `Y` — same combine-then-index pattern as `(dp),Y`. Verified against the documented 65816 opcode matrix (oxyron.de, cross-checked against softpixel's table) for the eight `0x_3` opcodes that use it (`ORA`/`AND`/`EOR`/`ADC`/`STA`/`LDA`/`CMP`/`SBC (sr,S),Y`).
- **`BRL` / `AddrRelativeLong`**: 16-bit signed offset, unlike `BRA`'s 8-bit (`AddrRelative`) — that's the entire reason `BRL` exists, to reach branch targets too far away for the short form.
- **`JMP (abs)` / `AddrAbsoluteIndirect`**: the 16-bit pointer lives in bank 0; the 16-bit target read from it combines with the *current* program bank.
- **`JML [abs]` / `AddrAbsoluteIndirectLong`**: the 16-bit pointer lives in bank 0; a full 24-bit target (offset low, offset high, bank) is read from it — the bank comes from the pointer table itself, not the current PB.
- **`JMP/JSR (abs,X)` / `AddrAbsoluteIndexedIndirect`**: the pointer address is `(abs + X)` inside the *current* program bank, and the 16-bit target read from it also stays in the program bank.
- **`[dp]` / `AddrDirectIndirectLong`**: 24-bit pointer read from the direct page, no `Y` indexing (contrast with `AddrDirectIndirectLongY`, which does index by `Y` after reading the same 24-bit pointer shape).

---

## 6. `BIT` — immediate vs. non-immediate is a real behavioral split, not just an addressing-mode difference

`BIT` (non-immediate): `Z` from `A & mem`; `N` and `V` are copied from the top two bits of the memory operand (bits 7/6 in 8-bit mode, 15/14 in 16-bit). `BIT` immediate (`#$nn`) is the documented special case: it **only** affects `Z`, leaving `N` and `V` alone — there's no "memory operand" to read flag bits from when the operand is the instruction stream itself. `OpBIT`'s two branches implement this distinction directly.

---

## 7. Opcode table verification status

The *execution* opcode table (`Cpu.OpcodeTable.cs`) got a dedicated verification pass against oxyron.de before being trusted — completed 236→256/256, verified against oxyron.de, caught one cross-reference typo (from the original project handoff). This is the table `Step()` actually dispatches through; treat it as verified.

The *disassembly* table (`Snes65816Disassembler`, separate by design — see this page's intro) has **not** had the equivalent treatment, per its own header comment and `EmuSen_Debugging_Tools_Reference_v5.md` §3.7 — no way to build/run this project from wherever it's being edited to cross-check it the same way. Built carefully against the same standard 65816 opcode matrix, spot-checked against a couple of real captured instruction bytes, but short of a real verification pass. Treat disassembly output as a solid first draft, not a verified reference, until it gets one.
