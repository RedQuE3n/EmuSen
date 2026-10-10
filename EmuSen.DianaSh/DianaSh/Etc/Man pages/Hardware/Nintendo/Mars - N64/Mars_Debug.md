# Mars — the debugger's hooks

*Phase F's third slice, landed 2026-09-18. `MarsCore.cs` (the registries and the per-instruction checks),
`Debug/MarsDebugTarget.cs`, `Debug/MarsDebugSpaces.cs`, the call and interrupt seams in `Cpu/`, the store report in
`Memory/MemoryBus.cs`, and `CouldBreak` in the shared `BreakpointRegistry`. Tests in `MarsDebugTests` and
`BreakpointCouldBreakTests`. The disassemblers themselves are `Mars_Disassembler.md`.*

---

## 0. What grades this

**No reference grades a debugger.** What does is the commands: a breakpoint must stop in front of its instruction, a
step must run what it counted, `step over` must come back to the caller, a watch must see a store where it landed.
`MarsDebugTests` drives each hook with a few instructions at the boot entry and asserts the address it stopped at,
and one test runs `disasm cpu` through the command itself, which is half of the phase's done-when
(`Mars_Gameplan.md` §4.6). The rest of the evidence is the breakage round (§8) and the cost (§7).

*Since 2026-09-22 the claims are `MarsDebugClaims`, run against this core as `MarsDebugTests` and against MarsRT as
`MarsRtDebugTests` with the same transcripts; MarsRT's hooks, which are tables in its Rust loop rather than these
seams, are `Mars_Native.md` §6.5, and so is what that stage found about this core (§6.5.2: the call stack is tracked on
every frame, armed or not).*

## 1. Where the registries live

**On the core, as on Mercury and Venus.** `MarsCore` owns `Breakpoints`, `Watches`, `FrameLog`, `Coverage`,
`RspCoverage`, `CallStack` and `Labels`, and the debug target hands them out. A breakpoint or a label therefore outlives
any one debugger session on the same core, and the core can consult them from its own loop without knowing a target
exists. `A_frontend_bundle_wires_the_target_to_the_core` checks that the bundle a frontend loads shares them.

## 2. Breakpoints, stepping and the call stack

- **The check runs before each instruction**, on the program counter the processor is about to fetch, as a 32-bit
  virtual address in an `int` (so KSEG0 addresses are negative; `ParseHex` reads eight hex digits the same way). A
  halt returns from `RunFrame` without finishing the frame; the next `RunFrame` runs the halted instruction before
  checking anything, which is what makes `continue` leave a breakpoint instead of stopping on it again.
- **A step counts from a halt.** The halted instruction runs unchecked, then each checked one counts — the same
  protocol as the other cores, which `A_step_runs_exactly_the_instructions_it_asked_for` fixes at three.
- **A call is `jal`, `jalr` or a taken branch-and-link, and it is pushed as it executes.** Pushing then means the
  call's own delay slot is already one frame deeper, so `step over` does not stop in it.
- **A return is `jr` through `ra`, and it is popped when its delay slot has run.** The pop was first placed at the
  start of the step after the slot, which is one instruction too late, because the breakpoint check runs before that
  step: `step over` stopped one instruction past the return. ~~"popped at the start of the following step"~~. It now
  happens at the end of the slot's own step, and `Stepping_over_a_call_stops_after_it_in_the_caller` and the step-out
  case pin the landing address. A `jr` through any other register is a jump, not a return
  (`A_jump_through_another_register_does_not_return`) — which is the convention, and wrong for code that returns
  through another register or tail-calls through one.
- **Interrupts are not frames on the stack.** `runto irq` is fed — the processor reports each interrupt it takes —
  and `runto frame` is fed at each frame's end, but exception handlers do not appear in `bt`.
- **`runto` a scanline is not fed.** The video interface counts half-lines, not scanlines, and nothing maps one to the
  other for the command yet.

## 3. Watches and data breakpoints

**A processor store is reported, byte by byte, in the space it landed in**: `RDRAM` with its physical offset, `DMEM`,
`IMEM` or `PIFRAM`. The report reads back what the store left, so in the windows that take only whole words
(`Mars_Memory.md` §2.4) a byte store reports the four bytes it actually wrote. Each byte goes to `watch` and to `bp
write`; the context is the storing instruction's address.

**What is not reported:** transfers — the PI, SI and SP DMAs, and every pixel the RDP writes — which change RDRAM
without a processor store, so a watch on data a transfer brings in sees nothing; the MI's repeat (`Mars_Memory.md`
§8.4), whose store is reported and whose repeat is not; and reads, which `watch r` and `bp read` would need and which
would cost a report on every load. A watch on the `CPU` space sees nothing either: stores are reported by where they
landed, not by the virtual address that named it.

## 4. The memory spaces

`RDRAM`, `DMEM`, `IMEM`, `PIFRAM` and a read-only `ROM`, each the machine's own array, and **`CPU`**: the processor's
32-bit virtual address space, translated the way kernel mode's 32-bit segments translate it, with the TLB consulted
for the mapped segments and never faulted.

**The CPU space reaches memories only.** Registers read zero there, because a register can answer a read by changing:
the RSP's semaphore is taken by reading it, and the cartridge's second domain would name a save chip the game had not
yet touched (`Mars_Save.md` §1). `The_cpu_space_reads_memories_through_the_segments_and_leaves_registers_alone` reads
the semaphore's address and checks the semaphore is still free. The ROM is readable through it and not writable.

**Its size is `int.MaxValue`**, the largest an `int` can say; the space is four gigabytes and its addresses above
`0x7FFF_FFFF` are negative `int`s, so a range written across that boundary would read backwards. `addr` resolves an
address through the same translation, to the memory it names or to a register block marked not addressable.

## 5. The RSP

Listed as a processor of its own, with its registers, its coverage and `IMEM` as its code space, so `disasm rsp` and
`cov rsp` work. **It cannot be halted.** The RSP runs inside the bus's clock, a slice of cycles at a time inside one
processor instruction; stopping it mid-slice would leave the main processor with an instruction half-accounted, and
the halt path has no way to resume it there. The command says so rather than arming something that never fires.

## 6. Disassembly through the target

Words are read big-endian from the named space and decoded by the processor that runs code there, at the address it
runs it from: `IMEM` and `DMEM` by the RSP's decoder at their offset; `CPU` by the VR4300's at the virtual address;
`RDRAM` at its KSEG0 address, `ROM` at its KSEG1 cartridge address and `PIFRAM` at its own, so that a branch in any
of them prints the target a processor would reach. `callers` classifies each listed instruction with the same decoder,
and `Each_space_is_decoded_by_the_processor_that_runs_it_at_its_own_address` pins the two choices that matter.

## 7. What it costs

The hooks run on every instruction, so they were measured: 300 frames of Super Mario 64 (Europe) through a frontend's
bundle in a Release build, against the commit before, at least twice each.

| | seconds |
| --- | --- |
| before the hooks | 28.3–29.6 |
| with the hooks | 30.6–33.2 |
| with the hooks and without the breakpoint check alone | 28.7–28.8 |
| with the hooks, the check gated by `CouldBreak` | 29.1–31.0 |

**The breakpoint check was the whole cost**: removing only it brought the time back. `ShouldBreak` pays for a method
call and a walk of an empty list when nothing is armed. `CouldBreak`, added to the shared registry for this, is false
exactly when `ShouldBreak` would return false and change nothing (`EmuSen_Debugging_Tools_Reference_v5.md` §3.26), and
with the check behind it the difference is inside the noise of these runs. The store report and the call seams cost
nothing measurable: the same build with and without a debug target attached ran in the same time.

~~The store report and the call seams cost nothing measurable~~ **Corrected in Phase G** (`Mars_Performance.md` §16):
the store report cost about 27 nanoseconds a store once a target was attached, hidden here behind the breakpoint
check's larger cost. The bus now asks the observer's `Listening` first, and reports only while a watch, a data
breakpoint or the uninitialised-read check exists.

**Since Phase G a frame with nothing armed skips the check entirely** (`Mars_Performance.md` §13): Mars asks the
registry's `IsQuiet`, and whether coverage or the profiler is armed, once at the start of each frame. The consequence:
a breakpoint armed from another thread while such a frame runs takes effect from the next frame.

## 8. The breakage round

Twenty-one rules broken in turn, against `MarsDebugTests` and `BreakpointCouldBreakTests`. **Sixteen were caught on
the first run; the five that were not now are**, each by a case added for it:

| survived | case added |
| --- | --- |
| only a `jr` through `ra` returns | `A_jump_through_another_register_does_not_return` |
| the CPU space's second 4KB of the RSP's window is IMEM | the CPU-space case, now reading IMEM too |
| RDRAM's code is seen from KSEG0 | `Each_space_is_decoded_by_the_processor_that_runs_it_at_its_own_address` |
| IMEM's code is the RSP's | the same |
| a load hands the watch to the new bus | `A_watch_survives_a_reload` |

`CouldBreak`'s six conditions were broken separately: each one dropped fails `BreakpointCouldBreakTests`.

## 9. What is not here

- **Conditional breakpoints and `eval`.** There is no expression context, so a condition has nothing to evaluate
  registers against, and holds.
- **Access counters, freezes, interrupt vectors, DMA channel tables and named break conditions**, each an optional
  surface of the target that returns nothing.
- **Watching reads, and anything a transfer writes** (§3).
- **Breakpoints on the RSP** (§5).
- **What a debugger costs under load.** §7 measures nothing armed; a breakpoint list or a watch costs what the shared
  registries cost, which is the same on every core.
