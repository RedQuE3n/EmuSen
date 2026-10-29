# EmuSen_Recompiler — a tailored recompiler on Cranelift's assemblers (a design, 2026-10-28)

*A design for the tester's approval; nothing in it is built. Its first guest is Sapphire's EE (the R5900), as the byte
face's emitter under `EmuSen_TwoFaces.md`; by design it is shared, and a second project (§10) moves MarsRT's VR4300 onto
it and decides the place of the other Cranelift lowerings. It lives here, beside `EmuSen_Jit.md`, rather than in
Sapphire's folder for two reasons: the parts it proposes are crates under `EmuSen/Cores/Shared/`, and the name
`Sapphire_Recompiler.md` already holds the EE recompiler's build record from S37 (§48). Its sections are this page's own
(§1 to §13); a section with a page named is that page's. Its stages are E0 to E4 (the EE) and A0 to A4 (the absorption,
§10); its predictions P-E1 to P-E8.*

*Marks, as `EmuSen_Jit.md` uses them: **[M]** measured, with the section it was measured in; **[R]** read from a
source; **[A]** argued; **[C]** a choice; and **[E]** an estimate, which is never a measurement. Every figure below
carries one.*

## 1. In one paragraph

Sapphire's exact levers took the static form of the EE's compiled code from 274 to 119 hot bytes a guest instruction on
the arena [M, `Sapphire_Redesign.md`, "The exact design's work a block"], and the ring at that size runs 2.68 to 2.85 ns
a guest instruction on the handheld, at the mark of 2.75 for Jak's opening and 1.2 times the arena's 2.28 [M, ibid.].
About 22 to 31 of the remaining bytes are Cranelift's code generation and the memory context read again after calls
that a hot path never makes [E, ibid., "What a Winch-style emitter would remove"]. **This page proposes an emitter of
the project's own**, written in the manner of Wasmtime's Winch: a single pass from a block's plan to machine code
through a macro-assembler interface, with an x86-64 backend on `cranelift-assembler-x64` and an aarch64 backend later,
fixed host registers for the context and the memory context, block-local registers for the guest's, no call on any hot
path, hot and cold code in separate arenas, and every exact lever of the forms made a rule of the design rather than a
patch. It is three parts with written interfaces (§3): a MIPS block planner shared by the R5900 and the VR4300, a
guest-neutral macro-assembler with one backend a host, and `emusen-jit`'s runtime grown to own code memory, linking and
invalidation. Cranelift stays, as the portable tier and as the VU's backend (§3.9, §10). **The first stage (§8, E0) is a
vertical slice that builds the static form for real**, which no stage has yet done, and measures on the same blocks in
the same run what it saves against Cranelift's forms; it stops if it saves less than half its estimate.

## 2. Goals and non-goals

**Goals.**

1. **The byte face's EE first** [C, the tester's scope of 2026-10-28]: the EE's compiled code at or under about 96 hot
   bytes a guest instruction over the blocks a field actually runs, static and dynamic forms together (§7.4), with every
   result identical to the interpreter's.
2. **Identical results to the cycle face at defined checkpoints** (`EmuSen_TwoFaces.md` §5). Every form in this design
   is exact against the EE's interpreter, timing included, as the forms were (§5). So within the EE the emitter bends
   nothing, and by `EmuSen_TwoFaces.md` §3 an emitter proven equal cycle for cycle may serve both faces; it is designed
   for the byte face and graded against the cycle face's interpreter.
3. **The handheld (Zen 4, x86-64) is the full-speed target** [C]. The x86-64 backend is built first and tuned on it.
4. **aarch64 designed now, built later** (Apple Silicon, ARM handhelds): nothing in the planner, the lowering or the
   runtime assumes x86-64, a 4 KiB host page, writable-and-executable memory or a coherent instruction cache (§3.7,
   §3.8).
5. **One owner for each fact**: a guest's semantics stay in its core; what two recompilers need alike is in one shared
   crate (§3, §10).

**Non-goals.**

- **Not an optimising compiler.** No IR passes, no global register allocation, no instruction scheduling. The measured
  limit is code bytes streamed in cold (`Sapphire_Redesign.md`, R0.2 and its addenda), not the quality of a hot loop,
  and a single pass also compiles faster [R, the Winch RFC: 15 to 20 times faster compilation, code 1.1 to 1.5 times
  slower than Cranelift's on WebAssembly].
- **Not a change of what a block means.** The block's contract (§6) is today's, made explicit. The board's replay at
  exits (`Sapphire_Native.md` §44.5) stays until the board at events (`Sapphire_Redesign.md` R5) retires it.
- **Not a replacement for Cranelift everywhere.** What stays on Cranelift is said in §3.9 and §10.
- **Not the VU's lane arithmetic.** R0.4 is closed as failed on the VU's floats (`Sapphire_Redesign.md`, R0
  conclusion; `Sapphire_Speed.md` §59), and no emitter changes that: the VU's cost is its float helpers, not its code's
  bytes (§10.4).
- **Not a reopening of the redesign's stop.** By the R0 conclusion the plan does not continue to R1 without the
  tester's choice. This design is the EE's lever if the work goes on; what it buys and what it cannot buy is stated in
  §7.5 and §12 Q5.

## 3. Architecture: the parts, each with one job and a versioned interface

```
 guest words ──► [mips-planner: decode → Plan]  (shared MIPS front end; per-guest Isa table)
                         │ Plan (plain data, v1)
                         ▼
      [guest lowering: Plan → Assembler calls]    (per guest, in the core: semantics, context, timing, helpers)
                         │ Assembler trait (v1), generic, monomorphised
                         ▼
      [assembler backend: x64 now, aarch64 later] (guest-neutral: encodings, labels, sections, helper calls)
                         │ bytes, relocations, link sites
                         ▼
      [emusen-jit 2: arena, publication, links, invalidation, budget, verifier] (guest- and host-neutral)
```

| Part | Its one job | Where it sits | Interface (version) |
| --- | --- | --- | --- |
| **Front end** `emusen-mips-planner` | a block's shape and facts: where it ends, delay slots, each instruction's reads and writes, the reuse keys, the exits and what each must write back | `EmuSen/Cores/Shared/emusen-mips-planner/`, new | `Isa` trait in (a per-guest table over `emusen-mips`'s decode); `Plan` out, plain data, 1.0 |
| **Timing planner** | static regions, the quiescent entry test, parity variants, the dynamic remainder | each core (Sapphire's `ee::jit::timing`; MarsRT's ticks) | fills `Plan`'s per-instruction cycle and region fields |
| **Guest lowering** | the guest's semantics as macro-assembler calls | each core (`ee::jit::lower`, `cpu::blocks::lower`) | generic over `M: Assembler` |
| **Macro-assembler** `emusen-assembler` | encodings, labels and branch relaxation, hot and cold sections, constants, the helper-call sequence and its trampolines | `EmuSen/Cores/Shared/emusen-assembler/`, new | `Assembler` trait 1.0; backends `x64` (feature, default on x86-64) and `aarch64` (later) |
| **Runtime** `emusen-jit` | code memory and W^X, publication, link slots, invalidation, the budget, the counters, the verifier's shape; the Cranelift tier | `EmuSen/Cores/Shared/emusen-jit/`, today's crate | 2.0: the arena and the link and invalidation API added; 1.0's `Lowering` kept as the Cranelift tier (§10.3) |

**New cross-component dependencies, named as the code-design rules require**: `sapphire -> emusen-mips-planner`,
`sapphire -> emusen-assembler`, `emusen-assembler -> cranelift-assembler-x64` (already in Sapphire's and MarsRT's lock files as a
dependency of `cranelift-codegen` [R, `Mars_Native.md` §5.8.4]), and, in §10's project, `marsrt -> emusen-mips-planner`,
`marsrt -> emusen-assembler`, `marsrt -> emusen-mips` (MarsRT takes only the disassembler today [R, `EmuSen_Mips.md` §1]).
`emusen-mips-planner -> emusen-mips` for the decode. Nothing in the shared crates names a console.

### 3.1 The guest front end: decode and block planning

**What is shared.** Every MIPS guest the project has (the R3000A, the VR4300, the R5900) has the same control-flow
grammar: a branch or jump with one delay slot, the likely branches that annul their slot when not taken, a block that
cannot end inside a slot, and a slot that is itself a branch or an ender refused [R, `Mars_Native.md` §5.8,
`Sapphire_Recompiler.md` §48.2]. They also share what the forms need from planning: which instruction last wrote each
general register (so that two accesses on "the same base" are provable from the plan, §5.4), the displacement of each
access, the dirty set at each exit, and the block's line and page limits. That is the front end's job, written once.

**What is per guest, through the `Isa` trait** [C]: the decode (`emusen-mips`'s tables, which stay separate by design
since one opcode cannot carry two rules [R, `EmuSen_Mips.md` §8.1]); each instruction's class (compiled, memory,
branch, ender); its register reads and writes, including HI, LO, HI1, LO1, SA and the 128-bit halves; which
instructions may raise; the block's maximum length; and the line size for the instruction-cache limit. The R5900 and
the VR4300 differ in each:

| | R5900 (EE) | VR4300 |
| --- | --- | --- |
| General registers | 128 bits; most integer operations touch the low 64 and keep the high half (§3.3) | 64 bits; in 32-bit mode results sign-extended from 32 |
| Extra state | HI1, LO1, SA; MMI's 128-bit operations; the second multiplier | the 32- and 64-bit modes (KSU, UX, SX, KX) that make 64-bit operations reserved in a 32-bit user mode; FR for the FPU |
| Missing | DMULT, DDIV, LL, SC and their family are reserved [R, `EmuSen_Mips.md` §8.1] | MMI, LQ and SQ |
| Timing | the issue model: dual issue, pairing by category and pipe, load, multiplier and divider latencies, the bus's free cycle (`Sapphire_Native.md` §44.4) | a fixed tick an instruction plus the vendor's multiply and divide stalls (`Mars_Native.md` §5.8.2) |
| Block validity | the instruction cache's lines and their generations (`Sapphire_Redesign.md` §4.2) | the words compared with memory at entry and the bus's write count (`Mars_Native.md` §5.8.1) |

**So the shared front end serves both** [A]: the differences are all table entries, timing or validity, none of them
the shape. The timing planner and the validity rule stay with each core, since they are each machine's facts.

**The `Plan`** is plain data: for each instruction its word, class, reads and writes, the definition each read
reaches (an index into the block, or "from the entry"), its memory width and the reuse key (§5.4); for each exit its
kind, its place and its dirty set; for the block its regions (§3.2). It is printable, so a planner test compares text,
and it carries nothing of a host. **This is the interface the critic asked for** [A]: planning is decided once and
handed to a lowering that knows no host, which hands calls to a backend that knows no guest; aarch64 is a second
backend, not a second planner or a second lowering.

### 3.2 Regions: static, dynamic, and their share

The EE's issue model is a chain of maximums over the instruction sequence and the state at entry
(`Sapphire_Redesign.md` §4.2). The timing planner splits a block into **static regions**, whose issue cycles fold to
constants for a quiescent start, and a **dynamic remainder** that computes the model at run time. Two refinements of
§4.2 are proposed here [A]:

- **Quiescent for what the block reads.** The entry test compares only the in-flight fields the region could wait on:
  a block that reads neither HI nor LO and starts no multiply does not test the divider's busy cycle, since the issue
  model stalls on it only through those. The planner knows the set; the test is a few compares on it, laid in the
  block's shared entry.
- **A dynamic prefix with a static suffix.** Where the test fails, the block does not run its whole dynamic form: the
  planner computes, for the entry-state class the test excludes (a bounded set: the remaining load latency, the
  multipliers' and divider's remaining cycles, the previous category and pipe, the parity), the first instruction *c*
  at which the model's state relative to the issue cycle is the same for every member of the class. The dynamic code
  covers instructions before *c*, and then joins the static code at *c* with its offsets rebased by one add. Whether
  such a *c* exists and how early it falls is a census (E1); pairing parity may never converge, which is why a block
  whose entries come in both parities takes a variant for each (`Sapphire_Redesign.md` §4.2).

**The share.** The critic's first objection is right: the 119 bytes were the static prototype's, shadow-compiled and
never run, and the dynamic form is today's emitter with every lever at 424 hot bytes [M, "The exact design's work a
block", "today's, with all of them"]. If a fraction *s* of the executed instructions ran static code at about 90 bytes
and the rest dynamic code at about 400, the working set would cost 90*s* + 400(1 − *s*) bytes a guest instruction, and
the mark of 96 would need *s* of at least 98 % [E, arithmetic on the figures above]. **No one has counted *s*.** The
dynamic prefix exists to shrink the 400 to its prefix's length, and E0 counts *s* before anything depends on it (§8).
The mark is therefore stated on the blocks' working set, both forms weighted by what a field runs (§7.4), not on the
static form alone.

### 3.3 A guest-neutral IR, or none: none, and a macro-assembler instead

**Recommended: no IR** [C]. A lowering calls the macro-assembler directly, as Winch generates "machine code directly
from Wasm bytecode, with no intermediate representation" [R, the Winch RFC]. Three reasons:

1. **What remains to save is what an IR pipeline adds.** Option 1's and the levers' disassembly counted register-to-
   register moves from the allocator's splits, block parameters zeroed, constant-pool operands, jump-only blocks and
   conditions made with `setcc` before a test: about 32 bytes a guest instruction on memory-dense blocks, about 22 to
   24 on the average block [M counts, E bytes, "What a Winch-style emitter would remove"]. They come from SSA with block
   parameters and a general allocator; a direct emitter with a fixed register assignment does not make them.
2. **The exact forms are block-local facts** the planner already computes (the same base, the same line, what changed);
   they need no optimiser to find them.
3. **Portability does not need an IR here.** It needs a narrow interface between the guest and the host, which the
   `Assembler` trait is, as Winch's per-ISA `Assembler` is [R, the Winch RFC: "each architecture gets its own `Assembler`"].

**The `Assembler` trait, version 1.0** (the shape, not the full list): three-operand integer operations in 32 and 64 bits
(an x86-64 backend adds the move a two-operand form needs where the destination is not the first source); compares
that set flags and branches on them, never a materialised condition unless a value is wanted; loads and stores of 1 to
16 bytes at `[base + displacement]`, the backend materialising a displacement out of its range; 128-bit moves and the
MMI's lane operations (from E3); labels with forward and backward references, relaxed to the short form where they
reach; a **section** argument on every label and sequence (`Hot` or `Cold`, §3.7); immediates of any width, the backend
choosing the encoding (x86-64's sign-extended 32 bits, aarch64's MOVZ and MOVK or logical immediates); `call_helper(id)`
and `jump_stub(id)` (§3.5); and `link_site(exit)` (§3.6). Nothing in it names a guest.

### 3.4 The register strategy

**Fixed for the whole compiled tier, on x86-64 System V** [C]:

| Host register | Holds | Why |
| --- | --- | --- |
| r15 | the context (`Context`: helpers, horizon, translation, line pointer, the exit's scratch) | read by every block and every stub; one register, never moved into an argument register (the register moves counted on the levers' blocks, 4.85 a guest instruction, include the context's moves into `rdi` [M]) |
| r14 | the core (`Ee`: the guest registers in memory, COP0, the issue model) | every write-back and every first read |
| r13 | the translation cache's base, or the page table's under R4 | every first access on a base |
| r12 | the cycle `last` at the block's entry | every static offset is `last` plus a constant, folded at the exit |
| rsp | the host stack, 16-byte aligned at every call | |

**Block-local** [C]: the guest registers the block names take host registers from a pool of the other eleven general
registers (and xmm0 to xmm15 for 128-bit values), assigned by the planner in first-use order with a linear scan over
the block's straight line; a register the block cannot hold is read from and written to the core where used. A guest
register is held either as its low 64 bits in a general register, its high half untouched in the core, or whole in an
xmm register; a change between the two writes it back first. The VR4300 holds all 64 bits in a general register.

**Held across helpers: everything, because no helper clobbers anything** [A, from option 1's measurements]. Option 1
showed that once nothing is held across a System V call the spills go but the reloads become loads of the same size
(step 2 saved 7.7 against 35.6 targeted [M]); the cure is a hot path with no call, and helpers that preserve every
register. So:

- **The hot path contains no call.** The measured hot exit's call (the replay, or `commit` under `stub`) becomes a jump
  to a shared exit stub (§3.5) after the dirty registers are stored; the stub's own call is outside every block.
- **A cold path that returns into the block** (a translation-cache fill, a data cache miss's refill once R3's slow paths
  return) calls through a trampoline that saves what the Rust helper may clobber, so the block's registers survive and
  nothing is reloaded. `forms::trampolines` built that on Cranelift's `preserve_all` [M, option 1, step 3]; here the
  backend emits the trampolines once an arena.
- **A cold path that leaves** stores the dirty registers and jumps to the exit stub; it holds nothing.

### 3.5 Helpers and their calling convention

**The helper convention, version 1** [C], the compiled tier's own and not System V's: arguments in rdi, rsi and rdx
(the context is always r15 and is not passed); the answer in rax; every other general and xmm register preserved; the
stack aligned by the trampoline, which then makes an ordinary `extern "C"` call to the Rust helper. A **helper table**
in the core declares each helper's id, its arguments and **its write set**: the fields of the core and the context it
may change. That table is the "what the helpers can write" table of option 1 (`Sapphire_Redesign.md`, Option 1) made a
declaration the planner reads: a field a helper may write is treated as changed after a call to it, whatever its value,
and the `watch` instrument checks the declaration in test builds. On aarch64 the same convention maps to x0 to x2 and
x0, with the trampoline saving x0 to x18 and v0 to v31 as AAPCS64 requires of a callee that preserves all.

**Shared stubs**, emitted once an arena and reached by a jump: the exit stub (the replay or `commit`, then the
dispatcher's return), the leave stub (a load or store leaving after a step), the miss stubs. Each takes its per-exit
constants in fixed registers (rdi a packed word, rsi the place, rdx the end cycle, as `commit_word` packs them today
[R, `ee/jit/mod.rs`]), so a block's exit is the stores of its dirty registers, three moves of immediates and a jump.

### 3.6 Block linking and chaining

C# Mars's chaining took the dispatcher out of nineteen entries in twenty and gained nothing, because the cost was the
bytes brought in cold (`Mars_Recompiler.md` §12.3) [M]; R0.2's ring is already chained and its cost grows with the bytes
live, not the entries (`Sapphire_Redesign.md`, R0.2) [M]. Chaining is therefore a second-order lever here, built because
the redesign's longer blocks and the board at events will need it, not for its own speed.

**Proposed: link slots in data, not patched code** [C]. Each exit to a static target jumps through a slot, `jmp [rip +
slot]` (6 bytes against a patched `jmp rel32`'s 5), the slots in a read-write data page beside the arena. A slot holds
either the target block's entry (its horizon and line checks first) or the dispatcher's return stub. Three consequences:

- **W^X stays absolute**: published code is never written again, on Linux or under macOS's MAP_JIT, and an aarch64 host
  needs no instruction-cache flush for a link change, since no instruction changes.
- **Unlinking is a store**: invalidating a block writes the return stub into every slot that names it, from a list of
  incoming slots kept per block (the method of [R, Bellard 2005, §3.6: "linked lists are also maintained to undo direct
  block chaining"]); a translation change, which makes every link's target suspect, resets every slot with one fill
  [R, ibid. §3.5: "when MMU mappings change, the chaining of the TBs is reset"].
- **An indirect jump** (JR, JALR) compares its target with a one-entry cache in its own slot pair and jumps through it
  on a match, the dispatcher otherwise.

A patched direct jump is the measured alternative if the slot's indirect jump costs more than its byte; that is E2's
measurement, and it would need a writable alias of the code pages, which this design avoids by default.

### 3.7 Code memory and W^X

**One reservation, two arenas** [C]: a region reserved once a process (1 GiB, so every rel32 reaches), its bottom for
hot code and its top for cold, each filled upward in batches. A batch is written through a writable mapping, made
read-and-execute once with one call, and never written again, which is `emusen-jit`'s rule today
(`EmuSen_Jit.md` §3.3) [R]. Separate arenas pack the hot lines of different blocks together, where Cranelift lays each
function's cold blocks after its own hot ones (`Sapphire_Redesign.md`, R0.2's hot bytes); the fetched footprint is then
the hot arena's bytes, which the new counter reads directly rather than from layout tags.

**Portability** [R, `Sapphire_Redesign.md` §4.9]: the host page size is read at run time, never assumed; the
protection change goes through one function that is `mprotect` on Linux and `pthread_jit_write_protect_np` with MAP_JIT
on macOS; on aarch64 every batch is followed by the instruction-cache maintenance the architecture requires, through the
`wasmtime-internal-jit-icache-coherence` crate already in the dependency tree [R, `Mars_Native.md` §5.8.4].
**The budget** stays whole: past it, every block is dropped, the reservation reset and the link slots refilled with the
return stub (`EmuSen_Jit.md` §3.4).

### 3.8 Invalidation on self-modifying code and TLB changes

The runtime owns the mechanism; each guest owns the rule [C].

- **The runtime**: a block is keyed by its guest address and a guest-supplied key; it records its incoming link slots;
  `invalidate(range)` and `invalidate_all()` unlink and forget, and the code stays until the budget (as now).
- **The EE**: a block is valid while its instruction-cache lines hold the generations it recorded, checked at entry and
  at each line it crosses; uncached code pages carry a version stamp raised by every store and DMA write into them, with
  C12's demotion of pages that keep changing (`Sapphire_Redesign.md` §4.2). A TLB write, a mode change or Config's write
  raises the translation generation (`Ee::translation_key` [R, `Sapphire_Redesign.md` "Exactness, from the code"]) and
  calls `invalidate_all` on the links.
- **The VR4300**: the words compared with memory at entry and the bus's write count, blocks behind the TLB held to their
  page (`Mars_Native.md` §5.8.1) [R], unchanged by the move.

### 3.9 Tiering

| Tier | What | Who |
| --- | --- | --- |
| 0 | the interpreter, the cycle face's, also the byte face's for what no block holds | every guest |
| 1 | **the baseline**: this design, single pass | the EE first; the VR4300 in §10 |
| — | **the Cranelift tier**: today's `Lowering` on `emusen-jit` 1.0's module | the VU; any host the baseline has no backend for; the EE and VR4300 until their baseline is accepted |

**No optimising tier is proposed** [A]. The measured limit is bytes in the cache; an optimiser's gains are in the hot
loop, which the one-block ring puts at 0.75 ns a guest instruction against 2.0 to 2.4 for the working set [M, R0.2's
addenda]. Were a field ever bound by a hot loop, a second tier could be Cranelift over the same `Plan`, but nothing
measured asks for it. **Compiling on the dispatcher's thread** may become possible if the baseline compiles a block in a
few microseconds (Cranelift took about a third of a millisecond for seven instructions [M, `Mars_Native.md` §5.8.4]);
it is measured at E0 and decided at E2, and either way results are identical, since compiled and interpreted code are.

## 4. What the block may assume, and what ends it

The contract of a compiled EE block, from today's code and the levers' proofs (`Sapphire_Redesign.md`, "Exactness,
from the code") [R], made the planner's rules:

**A block may assume, between its entry and any exit:**
1. its words are the words it was planned from (§3.8's rule checked at entry and at each new line);
2. nothing it holds changes a translation: TLBWI, TLBWR, TLBP, MTC0, ERET, SYSCALL, BREAK, CACHE and every coprocessor
   instruction end it before themselves (until E3 covers COP1 and COP2, which change no translation);
3. the data cache changes only by its own accesses: a miss leaves before its access, CACHE ends it, the board is handed
   the scratchpad and nothing of the core, and DMA does not snoop the data cache (the core's model);
4. no exception is taken inside it: an access that would fault, a misaligned one, a trap or an overflow leaves before
   its instruction to the interpreter, which raises exactly what it raises;
5. no instruction's cycle reaches the horizon (§5.7);
6. no helper writes a field outside its declared write set (§3.5).

**A block ends at**: a control transfer it cannot follow; an instruction of rule 2; the end of its page; a line the
instruction cache does not hold; the planner's maximum length (64 today [R, `shape.rs`]).

## 5. Exactness by construction: each lever as a rule of the design

| Lever, as measured | What it was | Where it lives in this design |
| --- | --- | --- |
| **The cache path** (exact, 37.0 hot bytes saved [M]) | a hit in either way and the scratchpad inline, everything else cold | the lowering's access template: dispatch on the page's kind and writable bit as one masked compare, both ways' tags, the D bit; the rest a jump to a cold miss stub |
| **Changed-only write-back** (31.2 [M]) | an exit stores only what differs from the entry's value | the `Plan`'s dirty set at each exit: a held register or model field is stored only if an instruction before that exit wrote it, or a helper whose write set names it was called |
| **Nothing held across calls** (7.7 [M]) | values read again after each call | superseded: no call on the hot path, and helpers that preserve every register (§3.4, §3.5) |
| **One translation a page** (7.7 [M]) | a later access on the same base reuses the page behind one compare | the `Plan`'s reuse key: the base's reaching definition and the displacement; the guard `(a ^ a₀) < 4096`, the full path cold |
| **One tag check a line** (34.9 [M]) | the same for a line, `< 64`, a store only after a store, the key poisoned past the caches | the same key; the planner applies the store rule and the poison rule, so a lowering cannot reuse wrongly |
| **The shared exit stub** (19.6 [M]) | one helper for the replay and the commit | one stub an arena, reached by a jump with three immediates (§3.5) |
| **The horizon** | a compare at every load and store after the first instruction | one compare at the entry of a static region (its cycles are constants), as the levers' page estimated at about 8 bytes [E]; a compare at each access in dynamic code |

**Why each stays exact.** The page and line reuse rest on rules 2 to 4 of §4, which the planner enforces by ending the
block; the reuse never outlives a block, since the planner's records start empty at each. Changed-only write-back rests
on rule 6, which the helper table declares and the watch checks. The shared stub computes what the per-exit code
computed from the same inputs in the same order with respect to the replay (`Ee::commit_exit` [R]). The one horizon
check rests on the static region's cycles being the entry's `last` plus constants: if the last access's constant is
under the horizon, every earlier one is; where it fails the block leaves at its entry with no step run, which the
dispatcher already handles. **What this design adds to the proof obligations**, and is not yet measured: the static
form's folded cycles and its quiescent test (§3.2), and the dynamic prefix's convergence point; the seeded faults of
§7.3 are aimed at them.

## 6. The VU, and why it stays where it is

The VU is a VLIW of two pipes with its own hazards and flag landings, compiled whole from its start and validated
against micro memory (`Sapphire_Native.md` §45) [R]. It has no MIPS control-flow grammar to share, so the front end does
not serve it. Its compiled code is mostly calls to the exact float functions, and its cost is those functions: R0.4
closed on them at 1.48 to 1.51 ns a lane operation on the handheld against 1.125, and the exact host-float kernel at
1.66 to 1.70 (`Sapphire_Redesign.md`, R0 conclusion; `Sapphire_Speed.md` §59) [M]. An emitter that saves the VU code's
bytes would not move that. **So the VU stays on the Cranelift tier deliberately** [C], takes `emusen-jit` 2.0's runtime
with no change to its lowering, and moves to the macro-assembler only if a census shows its code's bytes or its compile
time in a field's cost (§10.4).

## 7. Verification

### 7.1 Every block against the interpreter

- **The verifier** (`emusen-jit`'s shape, `EmuSen_Jit.md` §4) steps the interpreter beside every compiled instruction,
  everything written back first, with the baseline as with Cranelift; it is the first oracle of every stage.
- **The random programs** of S36 and S37 (`tests/blocks.rs`) [R], run at every stage with the baseline on, and the
  test that walks one base across a page and many lines [R, "The exact design's work a block"].
- **The backend's own grading**, before any guest: every `Assembler` operation encoded and executed on random operands
  against a Rust model, and the encoder's output disassembled and compared, as `cranelift-assembler-x64` itself is
  fuzzed "against a known-good disassembler" [R, its README].
- **Form coverage**: a counter for every fast path, guard, reuse, cold path and stub, and a stage passes only if every
  one has run in some test. Jak's cutscenes alone cannot carry the evidence (the critic's fourth objection): the counter
  shows which paths only Jak reaches, and a synthetic program is written for each.

### 7.2 The machine's digests

The bench's picture, sound and state digests on the dock's first 60 fields, the dock and the arena with the baseline
dispatched, equal to the interpreter tier's (`Sapphire_Redesign.md`'s method) [R]; ps2autotests' 78 texts; the boot's
760 frames; and, as the scoreboard grows, its games at their anchors (`Sapphire_Redesign.md` §7.4). Kingdom Hearts and
Dark Cloud 2 do not reach gameplay yet [R, the R0 addendum]; their boots run under the verifier as the second and third
programs beside Jak. From R1 the reference is the cycle-ordered model.

### 7.3 Seeded faults

Three to seven a stage, aimed at what the stage could break. Proposed for E0: a static offset one cycle short; the
quiescent test passing a pending load's register; a parity variant entered at the wrong parity; a dirty set missing a
register written only on a likely branch's taken path; a link slot left pointing at an invalidated block; a trampoline
that does not preserve xmm registers; a guard's bound of 4096 for a line. Each reverted, with what caught it recorded.

### 7.4 The measurement gates

| Gate | Instrument | Mark |
| --- | --- | --- |
| **Hot bytes a guest instruction, by purpose** | the baseline's hot arena counted directly; Cranelift's forms on the same blocks in the same run (`EMUSEN_SAPPHIRE_FORMS_BYTES`'s shadow) | the stage's estimate (§8); 96 overall |
| **The working set's bytes** | the census of block entries a field (`--working` [R, R0 addendum]) joined with each form's bytes: Σ over the forms a field enters, weighted by entries | about 96, static and dynamic together (§3.2) |
| **The static share *s*** | entries and instructions run in static code over all compiled ones | P-E2 |
| **ns a guest instruction** | `ring mix` at the bytes measured, desktop and the handheld on the charger | 2.75 (Jak's opening) and 2.28 (the arena's fields), two threads (§7.5) |
| **The real fields** | the bench windows, desktop and handheld, the EE's row by the sampler | the EE's share of §3.5's allowance (`Sapphire_Redesign.md`) |
| **Compile time** | µs a block | recorded; decides §3.9's thread |

### 7.5 The marks and the split

The marks 2.75 and 2.28 are the two-thread allowance: the EE's 9.0 ms of a 13.4 ms field (`Sapphire_Redesign.md`,
second addendum) [R]. On one thread the EE's allowance is 1.53 ns on the handheld (1.13 on the desktop at the tables' factor of 1.35),
which **no measured form meets**, the ring at 96 bytes included (2.24 to 2.43 on the handheld)
[M, R0's third addendum]. So the critic's third objection holds: the emitter's gates are only full-speed gates if the
split (§4.7 there) is taken. This page states every result against both marks, and §9 places the split's decision.

### 7.6 The phase-shift comparison

Every form here is exact, so its comparison is equality and needs no alignment. The byte face's acceptance
(`EmuSen_TwoFaces.md` §5) needs one where timing moves (the split, C1): two runs' streams of frames, sound and canonical
state at input polls, aligned within a stated bound. **Proposed as a DianaSh command written in Rust**, `phasecmp` (the
name a proposal): two plain-text streams in, one row a field (its number, the input-poll index, the picture digest, the
sound chunk's digest, the canonical state digest), and out the alignment found, the largest shift, and the first record
that no shift within the bound matches. It belongs to the board's stages, not this emitter's, and is listed so that it
is not forgotten: it is stage P in §8.

## 8. The EE's stages

*Each estimate is written before its stage starts, and the stage stops early under its stop rule. Sizes are lines that
stay. Every stage is behind a feature, off in the default build, until E3.*

| Stage | What | Estimate, before | Gate | Stop rule | Size |
| --- | --- | --- | --- | --- | --- |
| **E0, the vertical slice** | `emusen-assembler`'s x64 subset; `emusen-mips-planner`'s shape, reads, writes and reuse keys; the EE's lowering for the integer ALU without multiply and divide, the aligned loads and stores (LB to LQ, SB to SQ) on the scratchpad and cached-hit paths with page, line and stub reuse, and every branch and jump with its slot; **the static form for real**: folded cycles, the quiescent test for what the block reads, parity variants; the dynamic remainder through today's Cranelift emitter; dispatched by today's `blocks.rs` with its replay, for blocks the slice covers | **P-E1**: on the same blocks in the same run, the baseline 22 to 31 hot bytes a guest instruction under Cranelift's forms (`static+stub`), so about 88 to 92 on the arena's mix if it held there [E]; **P-E2**: *s*, unknown, recorded; **P-E3**: a block compiled in under 30 µs [E]; the ring at the bytes measured on the handheld near R0's 2.24 to 2.43 at 96 bytes | exact on the verifier, `tests/blocks.rs` and the three windows' digests; the seeded faults caught; bytes, *s*, compile time and the ring recorded | **stop if the slice saves less than half of P-E1's low end** (11 bytes a guest instruction on the same blocks), or if any form cannot be made exact | about 2,800 (assembler 1,000; planner 500; lowering and timing 800; tests 500) |
| **E1, the EE's S36 and S37 classes** | multiply, divide and both pipelines; the unaligned loads and stores; PREF; SA; traps; the dynamic prefix with its convergence point, in the baseline | **P-E4**: the working set's bytes within 1.15 times the static form's, if *s* ≥ 90 % and the prefix ends within 4 instructions [E]; otherwise the dynamic code dominates and this is said | exact as E0; *s* and the convergence point's distribution counted on the three windows | stop if the working set's bytes exceed 130 a guest instruction with the prefix in: the static form then cannot carry the mark, and the remedy is C1 (the tester's, §12) | about 1,500 |
| **E2, the runtime** | `emusen-jit` 2.0: the reservation and its two arenas, link slots, invalidation, the indirect jump's slot; one horizon check a static region; the perf map opened once with appends (D-1102's proposed fix) | **P-E5**: link slots within 2 % of a patched jump on the ring [E]; **P-E6**: hot bytes down about 8 with the one horizon check [E, the levers' page] | MarsRT's block tests and probe (Super Mario 64 and Ocarina of Time) digests and speed unchanged on its untouched Cranelift path; Sapphire's VU tests unchanged; ring and windows recorded | stop if MarsRT's digests or speed move | about 1,200 |
| **Decision** | the split (`Sapphire_Redesign.md` §11 Q4), and whether R1 has been built | — | before E3 | — | — |
| **E3, coverage** (R3's work on this emitter) | MMI in xmm lanes, COP1 and COP2 macro calling the float helpers, the slow paths that return (refill and write-back through trampolines), the data cache's refill as a helper | `Sapphire_Redesign.md` P-R2 (95 % compiled) and P-R8 | R3's gates against the reference (the cycle-ordered model once R1 exists): digests, the verifier, **the recompiled tier faster than the interpreter tier** on all three windows | R3's | about 3,000 |
| **E4, blocks** (R4's work on this emitter) | the page table of 2²⁰ entries, line generations, uncached stamps, longer blocks to transfers (14.5 instructions an entry measured [M, R0's second addendum]) | P-R3; the ring at §4.2's entries at the bytes reached | R4's | R4's | about 1,500 |
| **P, the phase-shift command** | `phasecmp` (§7.6) | — | equal runs report shift 0; a run shifted by k fields reports k; a changed field is named | — | about 400 |
| **aarch64** | the backend, when the Mac or ARM work resumes | — | every `Assembler` test and every E0 to E4 test on an aarch64 host | — | about 1,500 |

**P-E7** [E]: if E0 to E4 meet their estimates and *s* is high, the EE's row of a field on the handheld is within the
two-thread allowance on Jak's opening; the arena's fields stay about 1.2 times over at 119 bytes and reach their 2.28
only nearer 90 (the ring's measured slope between 119 and 96 bytes [M, the levers' page and R0's third addendum]).
**P-E8** [E]: compile time falls by at least ten times against Cranelift's, as Winch's does on WebAssembly.

**The first stage proves or refutes the approach by itself.** E0 decides three things nobody has measured: whether a
direct emitter saves what the disassembly says Cranelift adds (P-E1), whether the static form can be built exact at
all, and how much of a field it covers (*s*). If P-E1 fails, the Winch hypothesis is refuted on the project's own
blocks, the work stops, and Cranelift's forms stand as the EE's emitter.

## 9. How it fits the plan's order

- **The cycle face first.** `EmuSen_TwoFaces.md` §7 and the redesign's decision 1 put the cycle-ordered reference (R1)
  before the byte face's stages. E0 to E2 change nothing a player runs: they are measurement prototypes behind a
  feature, graded against today's interpreter, as the forms were. E3 is the first stage that would make the baseline
  the byte face's EE, and it waits for R1 and grades against it.
- **The redesign's stop.** R0's stop is accepted (`Sapphire_Redesign.md`, R0 conclusion), and R1 waits for the
  tester's choice of compromise or target. This design does not reopen it; it is what the EE's row needs if the work
  continues, and its stages up to E2 are useful whatever is chosen, since every target the stop leaves needs the EE's
  bytes down.
- **The split** is decided before E3 (§7.5). Until then every gate is reported against both marks.
- **The open decisions**, plainly: the split; R1's start; whether C1 is wanted where E1 shows the dynamic code
  dominating; the target after R0.4's closure; the crates' names and places.

## 10. Absorbing the other Cranelift recompilers (a project of its own, after E2)

*Added at the tester's request of 2026-10-28: plan for the new recompiler to absorb the project's other Cranelift
recompilers, not only to be reusable by them. It is its own project, started after E2 has proven the EE's first stages,
with its own stages (A0 to A4) and its own decision (§12 Q4), and it is not folded into E0 to E4.*

### 10.1 The three lowerings today

| Lowering | Guest | Where | What it compiles | Exits |
| --- | --- | --- | --- | --- |
| `cpu::blocks::jit` | the VR4300 (MarsRT) | `MarsRT - N64/src/cpu/blocks/jit/` | blocks of up to 64 words; the arithmetic, branches, aligned loads and, with the registers held, stores inline; the rest a call to the interpreter's handler (`Mars_Native.md` §5.8.2) | done, raised, finish |
| `ee::jit` | the R5900 (Sapphire's EE) | `Sapphire - PlayStation 2/src/ee/jit/` | blocks of up to 64 instructions: integer, branches, loads and stores through the translation cache and the data cache (§44, §48) | done, rollback |
| `vu::jit` | the VUs (Sapphire) | `Sapphire - PlayStation 2/src/vu/jit/` | whole microprograms keyed by their start (§45) | stall, stop, outside, release, bound |

All three are `emusen-jit` 1.0 `Lowering`s [R, `EmuSen_Jit.md` §6].

### 10.2 What all three share, and what stays per guest

| Concern | Shared, and where | Per guest |
| --- | --- | --- |
| Code cache and memory, W^X | `emusen-jit` 2.0's reservation and arenas (§3.7) | — |
| Publication | `emusen-jit`'s slots, unchanged (§3.2 there) | when a block is handed over (the sixty-fourth entry today) |
| Invalidation | the runtime's mechanism (§3.8) | the rule: line generations (EE), words and write counts (VR4300), micro memory (VU) |
| Block linking | link slots (§3.6) | which exits link; the VU links none (its programs are whole) |
| The helper ABI | the convention and the trampolines (§3.5), in `emusen-assembler` | the helper table and its write sets |
| The register strategy | the fixed registers and the pool (§3.4), in `emusen-assembler`'s backend description | which guest registers a block holds, and their widths |
| The emitter's backends | `emusen-assembler`: x64, later aarch64 | — |
| The verifier's shape | `emusen-jit`, unchanged (§4 there) | the reference machine and its comparison |
| Decode and block planning | `emusen-mips-planner` for the two MIPS cores (§3.1) | the `Isa` table; the VU's discovery of a microprogram stays the VU's |
| Guest semantics, timing | — | each core's lowering and timing planner |

### 10.3 What happens to `emusen-jit`: kept, grown, and its Cranelift path kept as the portable tier

**Recommended** [C]: `emusen-jit` is neither absorbed nor retired. It becomes the runtime of both tiers: version 2.0 adds
the arenas, link slots and invalidation, and keeps 1.0's `Lowering` over Cranelift's module as **the Cranelift tier**,
for the VU, for any host without a baseline backend, and as each guest's fallback until its baseline is accepted. A
major version, because the code memory moves from `cranelift-jit`'s module to the crate's own reservation for the
baseline (the Cranelift tier keeps its module). Retiring it would put its thread, publication and verifier, "subtle …
and proven once" [R, `EmuSen_Jit.md`, the plan's §9.6], into a new crate for no gain; absorbing it into `emusen-assembler`
would give the assembler a second job.

### 10.4 Where the VU fits

As §6: it takes the runtime and stays on the Cranelift tier. It would move to the macro-assembler only if a census of a
field showed its code bytes or its compile time in the VU's cost, at a stage of its own; the float helpers, which bind it
today, are the same functions whichever emitter calls them.

### 10.5 The migration, with MarsRT kept working

| Stage | What | Gate | Stop rule |
| --- | --- | --- | --- |
| **A0** | `emusen-jit` 2.0 published to all three users; the three lowerings unchanged on the Cranelift tier | MarsRT's 21 block tests at every tier, its probe on Super Mario 64 and Ocarina of Time, digests equal and speed within noise; Sapphire's bench digests and VU tests equal | any digest moves |
| **A1** | MarsRT's VR4300 on `emusen-mips-planner` and `emusen-assembler`'s x64 as a new tier beside its Cranelift tiers (a setting, off): its 64-bit registers held whole, the 32-bit mode's sign extension, the mode test at entry, its ticks folded as constants (`Mars_Native.md` §5.8.2), its three exits | the verifier beside every instruction; its random programs and 21 mechanisms; the probe's digests on both games equal to the Cranelift tiers' | not exact, or slower than its Cranelift tier on both games over two rounds |
| **A2** | the baseline as MarsRT's default on x86-64 if A1's speed is at least its Cranelift tier's; the Cranelift tier kept for other hosts | the probe's digests and speed, two rounds; the tester's acceptance | as A1 |
| **A3** | the VU's census (§10.4): its code's bytes and compile time a field | recorded | — |
| **A4** | aarch64: the backend serves both MIPS guests together | every test of A1 and E0 to E4 on an aarch64 host | — |

**Estimate before A1** [E]: MarsRT's emitter makes 80 to 100 bytes a guest instruction (`Mars_Recompiler.md` §12.3,
measured on the C# recompiler) and its own Rust lowering is not counted; the probe's games already run in 5.9 to 15.0 ms a
frame on the desktop (`Mars_Native.md` §5.8.8) [M], so A1 buys MarsRT little speed and the project one emitter in place of two.
That is its purpose, and the gate is equality, not speed.

## 11. Risks, ranked, with the cheapest check for each

| # | Risk | Cheapest check | When |
| --- | --- | --- | --- |
| 1 | The direct emitter saves much less than the disassembly's 22 to 31 bytes | E0's count on the same blocks in the same run | E0 |
| 2 | The static share *s* is low, so the dynamic code dominates the working set (the critic's first objection) | E0 counts *s*; a cheaper first look is a counting patch on today's dispatcher that tests quiescence for what each block reads at every entry, in the stage's scratch | E0, first day |
| 3 | The static form cannot be made exact (a model input the planner missed) | the verifier and E0's seeded faults on folded offsets and the quiescent test | E0 |
| 4 | The one-thread mark (about 1.53 ns on the handheld) applies because the split is not taken | the tester's decision; until then both marks reported | before E3 |
| 5 | The EE alone does not reach a field's full speed while the VU's floats stay short (R0.4 closed) | none cheap; the target is the tester's (§12 Q5) | now |
| 6 | The ring is synthetic and the real fields add helpers and interpreted code | the windows' EE row by the sampler at each stage | E2 onward |
| 7 | Hand-written encodings carry defects Cranelift's do not | the backend's execution tests and disassembler comparison before any guest (§7.1) | E0 |
| 8 | `cranelift-assembler-x64` is "Cranelift-specific" and its API may change between releases | pinned; only `emusen-assembler`'s backend names it | each upgrade |
| 9 | MarsRT disturbed by the runtime's 2.0 | A0's gate on its probe and tests | A0 |
| 10 | aarch64's assembler: no standalone crate for it exists beside the x64 one; its encoder lives in `cranelift-codegen`'s public `isa::aarch64::inst` with `MachBuffer` [R, the crate's source, 0.136.4], which the Winch RFC planned to factor out [R]; whether that suits a direct emitter was not checked | read the module's API, or write the few dozen encodings the trait needs and grade them as §7.1 does | before the aarch64 stage |
| 11 | Maintenance: one backend a host is code Cranelift maintained for free | the Cranelift tier stays for every host without one | ongoing |

## 12. Questions for the tester

*Decided 2026-10-28 by the tester, all as recommended:* the design is approved and E0 proceeds as a measurement
prototype behind a feature, before R1; the crates take plain names, `emusen-mips-planner` and `emusen-assembler`, beside
`emusen-jit` 2.0; the split is decided alongside the reference work, before E3; the absorption (§10) is its own project
after E2; the target stays full speed, with VU1 recorded as a named open item and taken up as its own work after E2;
handheld runs on the charger at each stage are routine. A critical review of the whole design comes before E3.

1. **[decision] Approve the design, and E0 as a measurement prototype behind a feature, before R1.** It changes nothing
   a player runs, as the forms did not, and its stop rule ends the approach if P-E1 fails. Recommended: yes.
2. **[decision] The three parts as crates under `EmuSen/Cores/Shared/`** (`emusen-mips-planner`, `emusen-assembler`, and
   `emusen-jit` 2.0), with the new dependencies of §3 named. Recommended: yes; the names are proposals.
3. **[decision] The split before E3** (`Sapphire_Redesign.md` §11 Q4), since the emitter's full-speed marks are the
   two-thread allowance. Recommended: decide it from E1's numbers, as the redesign recommends from R0's.
4. **[decision] The absorption (§10) as its own project after E2**: `emusen-jit` kept as the runtime with its Cranelift
   path as the portable tier, the VU staying on Cranelift, MarsRT moved by A0 to A2 with its block tests and its probe
   on Super Mario 64 and Ocarina of Time as the gate. Recommended: yes, started only when E2 has passed.
5. **[decision] The target after R0.4's closure.** The EE's emitter cannot close the VU's gap. Recommended: pursue E0 to
   E2 whatever the target, since every target the stop leaves needs the EE's bytes down, and choose the target (a
   compromise of §4.8 there, or a narrower set of fields) before E3.
6. **[routine] The handheld on the charger for each stage's ring and windows**, under `systemd-run --user`, as R0's
   follow-ups ran. Recommended: yes.

## 13. What was read for this design, and exposure

`Sapphire_Redesign.md`, all of it; `Sapphire_Speed.md` §58 and §59 (§44 is `Sapphire_Native.md`'s, whose §44.1 to
§44.6 the redesign summarises and which was read only through it and the code); `Sapphire_Disputes.md` D-1100 to
D-1104; `EmuSen_TwoFaces.md`; `EmuSen_Jit.md` and the `emusen-jit` crate's manifest; `EmuSen_Mips.md` §1, §6 and §8.1;
Sapphire's `ee/jit/mod.rs`, `ee/jit/shape.rs`, `ee/jit/forms.rs`'s declarations, `vu/jit/mod.rs`'s declarations and
`blocks.rs`'s head; MarsRT's `cpu/blocks/jit/mod.rs`'s head; `Mars_Native.md` §5.8 to §5.8.4, §5.8.8 and §5.8.9;
`Mars_Recompiler.md` §12 and §12.3; `CONTRIBUTING.md`'s design rules.

Outside the project, each read on 2026-10-28:
- `cranelift-assembler-x64` 0.136.4, its README and `src/lib.rs` and `src/api.rs` (the `CodeSink` trait: labels and
  deferred targets are the caller's to resolve), and its meta crate's `instructions/jmp.rs` (the jumps in 8- and 32-bit
  forms), from the local crate registry;
- `cranelift-codegen` 0.136.4, `src/isa/mod.rs` and `src/isa/aarch64/mod.rs` and `inst/mod.rs` (the aarch64 `inst` and
  `emit` modules public), and `src/isa/call_conv.rs`;
- the Bytecode Alliance, "Wasmtime baseline compilation" (the Winch RFC),
  https://github.com/bytecodealliance/rfcs/blob/main/accepted/wasmtime-baseline-compilation.md;
- the Wasmtime project, `winch/README.md`, https://github.com/bytecodealliance/wasmtime/blob/main/winch/README.md;
- F. Bellard, "QEMU, a Fast and Portable Dynamic Translator", USENIX Annual Technical Conference, FREENIX Track, 2005,
  §3.4 to §3.8, https://www.usenix.org/legacy/event/usenix05/tech/freenix/full_papers/bellard/bellard.pdf (a paper
  describing a design, not source code).

No emulator's source, no GS renderer, no ps2sdk, nothing sealed, no firmware and no game data was read. Nothing was
built or run for this page.

## 14. E0, the vertical slice (2026-10-28)

### 14.1 The estimate, written before the build

*Set on 2026-10-28, before any of E0's code was written, as §8 requires. Nothing in this subsection is measured.*

**P-E1** [E], restated from §8 with the terms it will be measured on. On the blocks the slice covers, compiled in the
same run as the shadow's Cranelift forms (`EMUSEN_SAPPHIRE_FORMS_BYTES=1`), the baseline's hot bytes a guest
instruction are **22 to 31 under `static+stub`'s on the same blocks**; if that held over the arena's whole mix it would
take 119.1 to about 88 to 92. **The stop rule**: E0 stops if the saving is under 11 bytes a guest instruction on the same
blocks, or if any form cannot be made exact.

**The terms of the comparison** [A], found while reading `forms.rs` for this estimate. The `static` forms that the 119.1
was measured on are R0.2's static prototype: C1's constant charge of one cycle an instruction, no store of each step's
cycles for the replay, no update of Random, no undo log for stores, and no test at the entry (`forms::is_static`). They
were compiled in the shadow and never run. The baseline's static form runs, so it has to carry what the prototype left
out: the exact issue model's offsets (folded, which costs no code), the quiescent test at the entry, the undo log of
every store, and the model's write-back at each exit. So the baseline is counted twice, from one lowering on the same
blocks:
- **on the prototype's terms**, without the entry's test and the undo log: the form that P-E1 and the stop rule are
  judged on;
- **exact**, as it runs: the form the ring is measured at, and what the working set will cost.

The exact form is expected to be larger by about 5 to 9 bytes a guest instruction for the entry's test (about 40 to 70
bytes a block of 9.2 instructions) and about 8 to 12 for the undo log (about 70 bytes a store at about 0.15 stores a
guest instruction) [E]. The model's write-back, Random and the steps' cycles go into the shared exit's helper, fed by a
table of the exit's constants, so they add no hot bytes beyond the table's address.

**P-E2** [E]: *s* unknown; recorded. **P-E3** [E]: a block compiled in under 30 µs. **The ring** [E]: at the bytes
measured, near the levers' 2.33 to 2.43 ns a guest instruction on the desktop at 119 bytes if P-E1 fails, and about 1.9
to 2.0 if it holds (the ring's slope between 119 and 96 bytes on the desktop); on the handheld near R0's 2.24 to 2.43 at
96 bytes.

*Restated on 2026-10-28, after the estimate's commit and before the measurement: `ee-horizon` (`Sapphire_Redesign.md`,
"One horizon check a block") landed while E0 was being built and took the per-access horizon compare out of Cranelift's
forms, 118.6 to 105.7 hot bytes on the arena. The baseline never had that compare. So the comparable Cranelift form is
now `static+horizon`, and the bytes are stated against it as well as against `static+stub`, the form the estimate named.*

### 14.2 What was built

Behind Sapphire's `baseline` feature (which takes `forms` and `emusen-jit/native` with it), off in the default build;
the default and `forms` builds compile and behave as before. `EMUSEN_SAPPHIRE_BASELINE=1` dispatches it.

- **`emusen-assembler` 1.0** (`EmuSen/Cores/Shared/emusen-assembler/`): the `Assembler` trait of §3.3 (three-operand
  integer operations at 32 and 64 bits, shifts by a constant or a register, compares, `set` and `select` on a condition,
  loads and stores of 1 to 8 bytes at `[base + displacement]`, operations on memory, labels, branches and jumps,
  sections, stub jumps and calls, the block's data and the address of a label) and its x86-64 backend. Instructions are
  encoded by `cranelift-assembler-x64` 0.136.4, pinned; jumps are laid out by the backend, rel8 where they reach within
  a section and rel32 otherwise, and everything that crosses a section, reaches a stub or reads the data leaves a
  relocation. Hot bytes are counted by purpose as they are laid. The helper convention's trampoline (§3.5) is the
  backend's: arguments in rdi, rsi and rdx, the answer in rax, every other register saved, the context in r15 handed to
  the Rust helper as its first argument. Variable shifts use BMI2's `shlx`, `shrx` and `sarx`, which mask their count as
  MIPS does; every x86-64 host the project targets has BMI2.
- **`emusen-mips-planner` 1.0** (`EmuSen/Cores/Shared/emusen-mips-planner/`): `Isa`, `Plan`, `shape` and `plan`. A plan
  is plain data and printable: each step's class, reads and writes, the definition its base register reaches, its dirty
  set, and the reuse keys of §5 (at most four earlier accesses, the latest for each base value, as the forms keep them).
  It does not depend on `emusen-mips`: the guest's table supplies the decode, so the planner's surface is narrower than
  §3 drew it.
- **`emusen-jit` 1.1** (`EmuSen_Jit.md` §7): the tailored code published in place of the Cranelift function, in the
  crate's own W^X code memory, its fallback word filled with the function's address.
- **Sapphire's lowering** (`ee/jit/baseline/`): the planner's table for E0's slice (the integer ALU but HI/LO, SA and the
  traps; LB to LQ and SB to SQ; every branch and jump), the static form's cycles (§14.3), and the lowering itself
  (`lower.rs`), which writes x86-64 through the trait with the fixed registers of §3.4 (r15 the context, r14 the core,
  r13 the translation cache, r12 the entry's last issue), seven pool registers for guest values and records in first-use
  order, frame slots beyond them, and four scratch registers. The dispatcher's helpers are `blocks/baseline.rs`.

### 14.3 The static form, and why it is exact

**The cycles are the core's own model.** `static_times` runs `Timing::issue` over the block from a model at rest
(`Timing::at_rest`: one instruction issued, nothing in flight, the pair slot closed) once with the branch not taken and
once taken, and records each step's cycle after it and the whole model after each step as offsets from the entry's last
issue (`Snap`). Nothing about the issue model is written again in the lowering, so the static form cannot disagree with
the interpreter's model about any rule of pairing, latency or stall.

**When the offsets hold** (the entry's test, `Timing::rests_for`): the pair slot closed and no group end pending
(`open` and `group_ends` zero), `floor` at most the last issue plus one, and every resource the block reads before it
writes it ready by the last issue plus one. The argument: the first instruction cannot pair (the slot is closed), so it
issues at the larger of its operands' ready cycles, `floor` and the last issue plus one, which is the last plus one when
the test holds, as from rest. Every later instruction issues no earlier than that, so an entry value bounded by the last
plus one never decides a later maximum or a later pairing compare (`earliest <= last`); and the fields the test does not
read (the last category, pipe and writes) matter only to a pairing the closed slot forbids. **The "plus one" is the
design's refinement measured here**: the instruction before a block (typically the delay slot of the jump that entered
it) leaves its result ready at the last issue plus one, and a test of "ready by the last issue" would fail on most
entries.

**Proven, not only argued** (`ee/jit/baseline/tests.rs`): over random blocks of the slice, single and dual issue, and
random entry models made by issuing random prefixes (multiplies and dividers among them), every entry the test accepts
gives, at every step, the cycle the model gives and a model equal field for field to the snapshot applied to the entry,
for the branch taken and not taken.

**What the exits commit.** The model after the steps, Random moved once each issue cycle that did not pair, and the
group are applied by the shared exit's helper from a record of the exit's constants (`Rec`: steps, the last load's or
store's steps, the undo entries, the place, the steps' cycles, the model for the branch not taken and taken), and then
Count, the cycles seen, IP7, the instructions issued, the UCAB and the place as `Ee::commit_exit` commits them. The
helper picks the taken model where the place the exit leads to is not the address after the slot, which is when the
model's `taken` was set. So no exit stores the model in code, and the static form's exits are the dirty registers'
stores, the record's address, at most the place and a reason, and a call.

**Where the design was corrected while building**, each kept exact:
1. *The exit is a call, not "the stores, then a jump"* (§3.4, §3.5). The replay at an exit may keep fewer steps than ran,
   and a rollback must find the core's registers as the block found them (or as its last load or store left them), so
   the registers cannot be stored before the replay. `Sapphire_Redesign.md` had already noted this ("Not exact as it
   stands"). The exit calls the shared exit through the trampoline, which preserves every register, compares the steps
   kept, and only then stores the dirty registers and jumps to the shared return. It costs about 8 bytes an exit more
   than the design's form.
2. *The state after the last load or store survives later writes.* Cranelift kept it as SSA values; a fixed home is
   overwritten. A register the last access's state holds is saved to a frame slot before an instruction after that
   access writes it, and only then; the partial write-back of a rollback reads the slot.
3. *Out-of-line code begun on the cold section is jumped over.* A translation in a guard's full path (itself cold) laid
   its cold miss path straight after its hit, so the hit fell into the fill for ever. Found on Jak's dock window (the
   bench stalled at the first field); not by `tests/blocks.rs`, whose one-base walk holds LWL and LWR and so was never
   in the slice. The walk is now also run within the slice
   (`the_baselines_guards_fail_across_a_page_and_leave_the_interpreters_machine`), which hangs without the fix.
4. *No per-access horizon compare*: since `ee-horizon` the dispatcher's one check a block guards the baseline's blocks as
   it guards the `horizon` lever's (`blocks.rs`, `one_check`).

### 14.4 The bytes (measured 2026-10-28)

The arena window (`jak-arena`, 300 fields from the state at 9,000) with `EMUSEN_SAPPHIRE_FORMS_BYTES=1` and the
baseline dispatched: 13,068 of the blocks compiled are E0's, 132,065 guest instructions. All forms in a column are from
that one run, on those blocks. Hot bytes a guest instruction:

| Purpose | `static+stub` | `static+horizon` | **baseline, the prototype's terms** | baseline, exact |
| --- | --- | --- | --- | --- |
| untagged | 43.8 | 40.0 | 0.0 | 0.0 |
| entry | 1.5 | 1.0 | 1.2 | 1.2 |
| guest registers | 2.9 | 2.9 | 4.7 | 4.7 |
| horizon and line checks | 9.7 | 0.8 | 0.7 | 0.7 |
| address and alignment | 4.1 | 4.2 | 5.4 | 5.4 |
| translation | 9.0 | 9.0 | 12.4 | 12.4 |
| cache tags and dispatch | 33.6 | 34.0 | 45.2 | 45.2 |
| data access | 2.6 | 2.5 | 2.2 | 2.2 |
| the instruction's work | 1.4 | 1.2 | 4.6 | 4.6 |
| exits' write-back | 10.2 | 10.2 | 6.5 | 6.5 |
| the entry's test | — | — | — | 7.5 |
| the undo log | — | — | — | 13.1 |
| **hot, in all** | **118.9** | **105.9** | **82.8** | **103.4** |
| cold | 273.3 | 270.1 | 159.0 | 159.0 |

The baseline's data (the exits' records, 360 bytes each) is 810.8 bytes a guest instruction, never fetched on a hot path.

- **P-E1 holds.** On the prototype's terms the baseline saves **23.1 hot bytes a guest instruction against
  `static+horizon`**, inside the estimate of 22 to 31, and 36.1 against `static+stub`. The stop rule's 11 is met.
- **Where the saving is.** Cranelift's untagged bytes (its moves, block parameters, constants from the pool and
  conditions made before a test, 40.0) are gone; the baseline's own bytes are larger in the translation, the cache
  dispatch, the guest registers and the instruction's work (about 22 together), where Cranelift folded or shared what a
  single pass recomputes: the page index from the address, the kind's mask, way 1's tag compares and the line's address
  are written out each time, and a guest register in a frame slot costs a load and a store. Those are the next bytes to
  take.
- **Exact, as it runs: 103.4**, 2.5 under `static+horizon`, which is itself not runnable as measured (it is the static
  prototype). The entry's test costs 7.5 (estimated 5 to 9) and the undo log 13.1 (estimated 8 to 12). The undo log is
  today's emitter's log written out; the static prototype simply left it out.

**Compile time** (P-E3): 42.5 µs a block on average in the shadow's run (the planner, the static cycles, the lowering
and the layout; the desktop), against an estimate of under 30. Cranelift's was about a third of a millisecond for seven
instructions on MarsRT (§3.9), so the baseline is about eight times faster, under P-E8's ten.

### 14.5 The static share *s* (measured)

On the same arena run (the 300 fields and the 60 before them): entries into E0's blocks 31.18 M, of which 18.34 M
(**58.8 %**) ran the static form and the rest its Cranelift function; steps 121.2 M, of which 74.46 M (**61.4 %**) static.
Of every compiled step of the run (147.06 M) the static form ran **50.6 %**. So, against §3.2's arithmetic, *s* is far
from the 98 % the mark of 96 would need on the static form alone: the dynamic prefix of E1 (§3.2) is the lever that
decides whether the working set can reach the mark, as P-E4 says.

### 14.6 Exactness (measured)

| Check | Base (no baseline) | Baseline dispatched |
| --- | --- | --- |
| Jak's first 60 dock fields (picture, sound, state) | 0F8671F57AB63C25, D5F15509CCDD1709, 818361BFE5D2DBF9 | the same |
| the dock, 300 fields from 6,000 | 830E3FEFEE1BE6B1, 2C225C73DB4D3F9D, 9CF1BED1AA64EAEE (`Sapphire_Redesign.md`'s base) | the same |
| the arena, 300 fields from 9,000 | 611B58F7CFE67E8B, BCA8F00F4743A955, 8B710CA05FDDF3A0 (`Sapphire_Redesign.md`'s base) | the same |
| `tests/blocks.rs`, test-checked profile | — | 29 of 29, with the two baseline tests |
| the static form against the issue model (`tests.rs`) | — | every accepted entry, every step |
| `emusen-assembler`'s grading | — | 9 of 9 |
| `emusen-mips-planner` | — | 4 of 4 |
| `emusen-jit`, with and without `native` | 12 of 12 | 13 of 13 |

**Seeded faults**, each built alone, run against `tests/blocks.rs` (test-checked, baseline dispatched) or the crate's
own tests, and reverted:

| Fault | Caught by |
| --- | --- |
| the state after the last load or store not saved before a later write (the partial write-back reads the newer value) | 2 tests: `loads_and_stores_beside_the_dmac_leave_the_interpreters_machine`, the S6 programs |
| the entry's test without its ready compares (a pending result taken as at rest) | 7 tests, among them all three random-program tests, the verifier's and the new walk's |
| the line guard as wide as a page | 2 tests: the new walk across a page, the S6 programs |
| the trampoline not saving r11 | `emusen-assembler`'s trampoline test, once its helper clobbers every caller-saved register; the first form of that test, whose Rust helper happened to leave r11 alone, missed it, and was strengthened |
| out-of-line code begun on the cold section not jumped over (the defect of §14.3, 3) | the new walk across a page, which hangs; Jak's dock window found it first |

Not seeded, as the forms' page did not: a translation or line reused across a TLB write, an eviction or CACHE, which
the planner rules out by ending the block before them (§4).

### 14.7 The ring at the bytes reached (measured)

R0's `ring mix` at the baseline's exact bytes (103) and on the prototype's terms (83), two rounds, the ring binary from
this tree (GLIBC_2.34 at most). The desktop ran with other jobs on the machine at CPU weight 100. ns a guest
instruction:

| Bytes asked (ring code) | Desktop, today's 2.87 an entry | Desktop, §4.2's 14.5 | Handheld, today's | Handheld, §4.2's |
| --- | --- | --- | --- | --- |
| 103 (102.7 / 106.7), the exact form | 2.14–2.25 | 1.54–1.55 | **2.32–2.36** | 2.66–3.00 |
| 83 (81.5 / 85.9), the prototype's terms | 1.72–1.78 | 1.22–1.25 | 1.90–2.01 | 1.55–1.66 |
| the levers' 119 (`Sapphire_Redesign.md`) | 2.33–2.43 | 1.73–1.77 | 2.68–2.85 | 2.57–2.76 |
| the marks, handheld | | | **2.75** (Jak's opening) and **2.28** (the arena's fields), two threads; **1.53** one thread | |

The handheld ran on the charger (ACAD online 1 before and after), `hh.sh guard` first, through `systemd-run --user`,
its scratch removed afterwards. **At the exact form's bytes the handheld runs 2.32 to 2.36 ns a guest instruction with
today's entries: under the mark of 2.75 for Jak's opening, and 1.02 to 1.03 times the arena's 2.28.** With §4.2's
entries the second round was noisy (2.66 and 3.00). On the prototype's terms (83 bytes) it would be 1.90 to 2.01, under
both two-thread marks. No measured form meets the one-thread mark of about 1.53 (§7.5). These are the ring's, a
synthetic working set at the measured bytes, not the real fields (§7.4's last rows, from E2).

**The two figures.** 82.8 is the baseline on the static prototype's terms, the form the 105.9 and 118.9 were measured
in, and the one P-E1 is judged on. 103.4 is the form that runs and was graded on every digest: the same code plus the
entry's test (7.5), which sends an entry not at rest to the Cranelift function, and the undo log of every store (13.1),
which a rollback needs. Neither of those is in the Cranelift forms' 105.9, because the static prototype never ran.

### 14.8 What E0 leaves

- **For E1, from the measurements:** *s* at 59 to 61 % on the arena makes the dynamic prefix (§3.2) the stage's main
  lever; the single pass's own bytes (the translation, the cache dispatch and the guest registers' slots, about 22 above
  Cranelift's) are the second; the undo log (13.1) is the third, and one horizon check a static region cannot take it.
- **The owed test** (from `ee-horizon`): a busy VIF1 or GIF beside a block, covering an event moved mid-replay. No test
  catches that gap today, and it does not fall within E0's slice, so it is added to E1's verification plan.
- **Form coverage** (§7.1) was not instrumented with counters at E0; the walk across a page was added when the dock
  window found its path untested. E1 should add the counters before it adds paths.
