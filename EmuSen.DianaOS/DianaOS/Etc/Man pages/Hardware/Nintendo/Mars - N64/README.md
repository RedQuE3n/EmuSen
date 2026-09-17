# Mars (Nintendo 64)

What exists is a plan: [`Mars_Gameplan.md`](Mars_Gameplan.md), pinned 2026-09-15, which is the "what should I work on next" doc for Mars in the same role `Mercury_Gameplan.md` plays for Mercury — with the difference that it was written before any code rather than during it, and says where it expects to be wrong.

Two decisions are already taken and carry their reasoning there: the whole RCP is emulated at low level (§2.1), and correctness leads with performance as its own later phase (§2.2). §3 is the oracle question — what a Nintendo 64 core can be graded against — and it carries the one unverified assumption the phase order rests on.

[`Mars_References.md`](Mars_References.md) is the second page here: four GPL emulator codebases checked out for study, the procedural rule that keeps them read-only (§2 — they are read for mechanism, and the implementation is written from hardware documentation), what each is actually good for, and §5, which is the record of what they got *wrong*. That last section is the one that fed back into the plan.

[`Mars_TestOracle.md`](Mars_TestOracle.md) is what the plan's §3 was waiting on: the hardware test corpus can report a text verdict headlessly, so the phase order stands. It carries the protocol, the four things Mars must do before the ROM will say a word — two of which fail silently if got wrong — and the finding that the RDP has no hardware-grounded oracle anywhere, only a very good software one.

[`Mars_Documentation.md`](Mars_Documentation.md) is the assessed source list — which documents are authoritative, where they contradict each other, where one of them is outright wrong, and what an implementer needs that no document anywhere answers. Two gaps there shape the work: the RDP's subpixel-mask rule, and system timing in its entirety. It also settles the plan's §2.1 condition by showing the condition was posed wrongly.

[`Mars_Rom.md`](Mars_Rom.md) is the first page describing code rather than plans: Phase 0's container handling and header parser, which emulate nothing. Read §1.1 before touching the loader — the container is decided by the magic word and never by the extension, and the reason is a documented disagreement between sources about what the extensions mean.

[`Mars_Memory.md`](Mars_Memory.md) is Phase A's first slice: the address map, the debug port, and the single machine clock. §3.1 carries the decision that `Count` is derived rather than incremented, which is this project's answer to a failure mode another emulator's commit log documents over years.

[`Mars_Cpu.md`](Mars_Cpu.md) is the VR4300's integer core: the dispatch shape, 64-bit registers, the two-program-counter delay-slot model, and the two decisions taken in advance — exceptions are thrown so that a faulting instruction cannot leave partial state (§4), and only cycle counts the vendor manual tabulates are charged (§5).

[`Mars_Tlb.md`](Mars_Tlb.md) is address translation for the mapped segments: paired entries, the linear scan, and §3's three distinct failures — a miss, an entry that is invalid, and a store to a page that is not writable — which vectoring had previously been unable to tell apart.

[`Mars_Boot.md`](Mars_Boot.md) is the handoff that replaces the PIF, and the record of what happened when a real ROM ran through it: the corpus boots through libdragon's bootcode into its own entry point, and stops at a coprocessor-1 move, which is where Phase B begins.

[`Mars_Cop0.md`](Mars_Cop0.md) is what the oracle found once it could speak: coprocessor zero is almost nothing but special cases — write masks, constants, registers hardware fills in, and seven that are not registers at all but a latch holding the last COP0 write. §1 is the headline: with this slice every CPU, COP0, TLB, exception and LL/SC test the corpus runs now passes, which is the condition `Mars_Gameplan.md` §4.1 set for Phase A.

[`Mars_FpuMath.md`](Mars_FpuMath.md) is Phase B's body: the software float, and the measurements that justify it having been written rather than borrowed from the host. §1 settles the reversed NaN convention, §3 records that the part refuses to compute with denormals at all, §4.1 settles a rounding question the survey left open, and §8 is a square-root bug that passed every tidy test value. §10 is two commercial cartridges running twenty million instructions without a fault.

[`Mars_Fpu.md`](Mars_Fpu.md) is Phase B's first slice: the coprocessor register files and the paths into them, with no arithmetic at all. Read §9 first — with this in place the hardware corpus starts printing verdicts, and the tally it prints is now a ratchet in the test suite. §9.2 is two integer defects the corpus found and that slice deliberately did not fix; `Mars_Cpu.md` §14 is where they were settled, and neither turned out to be what it looked like.

[`Mars_Privilege.md`](Mars_Privilege.md) is the VR4300's three modes: how the mode is derived — §1, where the answer is that the mode field is not what decides it — and §2's address map, which is three different maps chosen by mode and is kept row for row from the corpus's own forty-five case table. §4 is the asymmetry worth knowing before implementing it: kernel mode runs the doubleword instructions with its own 64-bit addressing bit clear, and the other two modes do not. §6 works out the rule for reverse-endian addressing, which is the next thing the corpus asks for and which nothing yet implements.

[`Mars_ReverseEndian.md`](Mars_ReverseEndian.md) is `Status.RE`, the bit that lets a big-endian processor run little-endian user code. §2 is the whole feature — one exclusive-OR, settled against the corpus's twenty-four result rows rather than read off its helper function, because the obvious reading is right for aligned accesses and wrong for the merging ones. §4 is the part that is easy to leave out (instruction fetch is mirrored too) and §5 is the TLB defect the feature's test harness exposed, which is what five of its seven corpus tests were actually failing on.

[`Mars_Rsp.md`](Mars_Rsp.md) is Phase C's first slice: the signal processor's scalar half, which is MIPS with no exceptions, no alignment and no 64-bit words. §3 and §4 are the two places carrying the main CPU's code across would have been wrong in ways no main-CPU test could catch; §5.1 is a status-register rule the corpus measures for every field; §6 explains why an unbuilt vector instruction does nothing rather than stopping the machine, which is the opposite of the choice made for the FPU and for a reason.

[`Mars_Corpus.md`](Mars_Corpus.md) is the hardware corpus as a running instrument rather than as a protocol. **The run now completes** and the corpus reports its own verdict — §1. Read §2 before quoting any "every X test passes" claim from another page here: a run that stops early reports on the part of the world it reached, two claims had to be qualified because of it, and one of them was this page's own. §3 is the census of the complete run, including seventeen CPU groups that no earlier run reached. §8 is why no earlier run could: the corpus needs an Expansion Pak, and the harness had been running it on a 4MB machine.

The remaining per-component pages (`Mars_CPU.md`, `Mars_RSP.md`, `Mars_RDP.md`, and the rest) get added here as the phases that build those subsystems land, the same way `Venus - SNES/`'s and `Mercury - GB-GBC/`'s were.

See `Man pages/EmuSen_Core_Naming_Scheme.md` for the full core naming scheme, and `Man pages/Hardware/README.md` for how this documentation set is organized.
