# Mars — the signal processor's scalar half

*Phase C's first slice, landed 2026-09-17. The RSP as a processor: its program counter,
the scalar instruction set, the interface registers it is driven through, and the clock
it shares with the main CPU. `Rsp/Rsp.cs`, with the interface in `Memory/SpInterface.cs`.
Tests in `MarsRspTests`. The vector unit, which landed the same day as the next slice, is
`Mars_RspVector.md`.*

---

## 1. What the RSP is, for the purposes of this slice

A second MIPS processor inside the RCP, with 4KB of instruction memory and 4KB of data
memory and nothing else. Its scalar half is MIPS I with most of the hard parts removed:
**no exceptions, no alignment, no 64-bit words, no multiply or divide, no branch-likely,
no TLB and no caches.** What it has instead is a vector coprocessor on coprocessor 2, which
is the reason the processor exists and has its own page (`Mars_RspVector.md`).

The removals are what make this half small, and they are also where it differs from the
main CPU in ways that are easy to carry over by accident. §3 and §4 are both of that kind.

## 2. The program counter is an offset into instruction memory

Twelve bits, with the low two always clear, so every counter value is `& 0xFFC`. A write
of `0xFFFFFFFF` to the register reads back `0xFFC`, and a counter that runs off the end of
instruction memory wraps to its start rather than leaving it. It lives at `0x04080000`, a
page away from the other interface registers.

Mars models it with the same two-counter delay-slot arrangement as the main CPU
(`Mars_Cpu.md` §3), with both counters masked at every step.

### 2.1 Three delay-slot behaviours the corpus pins

- **A break in a delay slot still lets the branch decide the counter.** A taken branch
  with a `BREAK` in its slot halts with the counter at the branch target; an untaken one
  halts at the instruction after the slot. That falls out of the two-counter model without
  a special case, which is some evidence the model is the right shape.
- **A jump-and-link whose link register is its own target register reads the target
  first.** `JALR $at, $at` must jump to the old value of `$at` and then overwrite it with
  the return address. Writing the link first and then reading the target — which is the
  order the operation's name suggests — jumps to the return address instead. Mars had it
  that way round; the corpus caught it, and a test now reproduces the corpus's case.
- **The link of a branch-and-link is written whether or not the branch is taken.**

## 3. Nothing traps, so the signed and unsigned forms agree

`ADD` and `ADDU` are the same instruction here, as are `SUB` and `SUBU`, and `ADDI` and
`ADDIU`. With no exception mechanism there is nothing for an overflow to do, and the
corpus's vector `0x12345678 + 0xFFFFEDCB` wraps to `0x12344443` under both encodings.

This is the first of the two places where carrying the main CPU's code across would have
been wrong in a way no test on the main CPU could catch, because on the main CPU the
distinction is real.

## 4. A data address is twelve bits, and alignment does not exist

Every load and store reaches data memory only, at `address & 0xFFF`. Everything above
those twelve bits is discarded rather than faulting, so `0x7FFD` and `0x1FFD` both name
`0xFFD`.

**An unaligned access is ordinary.** A `LW` at `0x001` reads the four bytes at `0x001`
through `0x004`, big-endian, and a `LW` at `0xFFD` reads `0xFFD`, `0xFFE`, `0xFFF` and then
wraps to `0x000`. Mars implements every data access as a byte loop with the mask applied
to each byte, which makes both properties structural rather than special cases. The
corpus's load table is reproduced in `MarsRspTests` for the offsets that exercise each:
aligned, one byte in, three bytes in, straddling the top, and wrapping.

This is the second place the main CPU's code would have been wrong: there, an unaligned
`LW` is an address error (`Mars_Cpu.md` §6).

## 5. The interface registers, from both sides

Eight registers at `0x04040000` — the two DMA addresses, the two DMA lengths, status, the
two DMA progress flags and the semaphore — and the counter at `0x04080000`. Phase A built
the DMA half of these for the boot handoff (`Mars_Memory.md` §6); this slice adds the
processor behind them.

**The RSP reaches the same registers through its own coprocessor zero**, by index: 0–7
are these eight, and 8–15 are the display processor's command registers. So `MFC0` and
`MTC0` on the RSP are not a separate mechanism but a second address for the same
hardware, and Mars routes them through the bus rather than duplicating the registers.

Status is assembled on read rather than stored: the processor owns halt and break, and
the interface owns single-step, interrupt-on-break and the eight signals.

### 5.1 A write names a request per field, and naming both halves is no request

Status is written as pairs of bits — clear halt and set halt, clear interrupt and set
interrupt, and so on through the eight signals. **A write with both bits of a pair set
leaves that field unchanged.** It is not a clear followed by a set, nor a set followed by
a clear; it is nothing.

Mars had applied the clear and then the set, so set won. Eight corpus signal tests, the
interrupt-on-break test and two register-access tests all failed on it, and several of
them failed with values that looked like an unrelated defect — a status register reading
back with every signal raised — because the corpus writes many pairs at once and expects
most of them to be inert.

The corpus tests this rule for **every pair Mars applies it to**: the signals,
interrupt-on-break, the interrupt, and halt. Halt is the interesting one, because
clearing halt starts the processor: the corpus writes both halt bits with a program
loaded and checks the counter never moved, then clears halt alone and checks that it did.
Mars had applied the rule to all four before finding that out, on the reading that one
set-and-reset mechanism serves every field; the reading was right, and it is now a
measurement rather than a reading.

Clearing halt starts the processor at whatever the counter register holds, and setting it
stops the processor where it is — the counter is not reset, and a halted program resumes
from where it stopped.

## 6. An encoding this half does not own does nothing

The RSP raises no exceptions, so there is nothing a reserved or unimplemented encoding
could do except nothing. Every opcode outside the scalar set and the vector unit is a
no-op.

> **Superseded 2026-09-17 by the vector unit (`Mars_RspVector.md`).** This section recorded
> that `COP2`, `LWC2` and `SWC2` executed as no-ops while the unit was unbuilt, and why that
> was chosen over stopping the machine. The reasoning is kept because it was borne out: it
> read
>
> *That is a deliberate choice against the alternative this project used for the FPU
> (`Mars_Fpu.md` §6), where an unbuilt instruction stopped the machine loudly. There, the
> scaffold was worth its cost because nothing past the first unbuilt instruction could run.
> Here the opposite holds: the vector tests are a self-contained block, the corpus's
> programs report results through data memory whether or not the vector instructions did
> anything, and a loud stop would have cut the run short at exactly the moment it first
> became able to reach the end. The vector tests fail, individually and honestly, and
> everything after them runs.*
>
> The complete runs that followed are what located the seventeen CPU groups
> (`Mars_Corpus.md` §9), and the census they produced attributed exactly 155 groups to the
> unit — which is the number the unit then cleared.
>
> **One consequence the section did not foresee:** the vector unit's first unit tests were
> written against the no-op build to show they failed, and four cases passed anyway, because
> a no-op load leaves a register at the zero a test expected (`Mars_Corpus.md` §10). A
> silent no-op makes a test's failure depend on it not expecting the value nothing produces.

## 7. The clock

The RSP steps from the machine's one clock (`Mars_Memory.md` §3): each tick of the bus
executes one RSP instruction while the processor is running. **One instruction per tick is
a placeholder**, not a measurement. The two processors are clocked at different rates on
hardware, and timing accuracy is Phase G's (`Mars_Gameplan.md` §2.2).

What the placeholder does establish is that the two processors run in parallel rather than
one to completion before the other, which the corpus tests directly — the RSP raises a
signal, waits in a loop for the CPU to raise one back, and records whether it timed out.
That test passes.

## 8. What the corpus says

**Eighty-eight of the 248 RSP and SP test groups pass**: every scalar instruction, all
branches and jumps including the three delay-slot cases in §2.1, `BREAK` in all three of
its forms, the counter wrap, the counter masking, the parallel-running test, and every
interface register test. Of the 160 that fail, all but six are the vector unit. The six are
four `spmem` tests about how the *main CPU's* sub-word loads and stores reach the RSP's
memories — a bus question rather than a processor one — and one `RSP STATUS` test that is
in fact about the display processor freezing, which is Phase D.

**The larger result of this slice is not in that count.** With the RSP able to halt, the
corpus run got past the point where it had waited since Phase A — and ran straight into a
defect in how Mars was *running the corpus*, which is `Mars_Corpus.md` §8.

> **Update 2026-09-17, after the display processor's interface (`Mars_Rdp.md`): all 248 pass.**
> The last one was the display processor's after all, and cleared with the rest of its module.
>
> **Update 2026-09-17, after the main CPU's access to RSP memory (`Mars_Memory.md` §2.4):
> 247 of the 248 pass.** The last is *RSP STATUS: start-valid*, which lives in the corpus's
> `rdp` module at its RDP level and fails on *"RDP was told to freeze, but it didn't"*. No test
> of the RSP fails.
>
> **Update 2026-09-17, after the vector unit: 243 of the 248 pass.** The five that fail are
> the four `spmem` groups and the `RSP STATUS` group named above. That also corrects this
> section's arithmetic, which said *"all but six"* and then named five: 160 failing groups
> less 155 in the vector unit is five, and five is what the run now shows.

## 9. What this slice does not do

- **Single-step as more than a status bit.** It is stored and reported, and it halts the
  processor after each instruction; nothing is known about how hardware interacts it with
  a break or a delay slot.
- **Timing.** §7.
