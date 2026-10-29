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

## 15. E1, the EE's S36 and S37 classes and the dynamic prefix (2026-10-29)

*Decided 2026-10-29 by the tester: E1 starts with the dynamic prefix (§3.2), before the instruction classes.*

### 15.1 The prefix's estimate, written before the build

*Set on 2026-10-29, before any of the prefix's code was written. Nothing in this subsection is measured.*

**Where the prefix lives** [A]. Since E0 the static form's code holds no timing at all: its cycles are constants in each
exit's record, which the shared exit applies (§14.3, "What the exits commit"). So a dynamic prefix needs no code of its
own. Where the entry's test fails, the block notes it in the context (a store laid cold) and runs on into the same body
in place of jumping to the Cranelift function; at the exit, the shared exit runs the core's own issue model from the
entry's model over the first steps, and at the first step *c* where that model is the static snapshot shifted in time
for everything the rest of the block can read, takes the record's constants rebased by one add. Where no such *c*
comes before the exit, the model runs over every step, which is the interpreter's timing and exact by definition. The
convergence is tested per entry, not found by the planner for a class of entries as §3.2 proposed: a test at run time
needs no bound on the class and is exact for every member of it. **No entry falls back whole.**

**The bytes** [E]. E0's working set on the arena's covered blocks was the static form for the 61.4 % of steps it ran and,
for the rest, the entry's test and then the Cranelift function with today's levers, about 424 hot bytes a guest
instruction on the arena (`Sapphire_Redesign.md`, "The exact design's work a block"): about
0.614 × 103.4 + 0.386 × (7.5 + 424) ≈ **230**. With the prefix every covered step runs the exact form, whose code is
unchanged: **103.4**. The estimate of the saving is therefore about **127 hot bytes a guest instruction on the working
set**; the stop rule's half is 63. The Cranelift function's bytes on the same blocks are measured in the same run (the
shadow's `today+levers`), so the 424 becomes a measurement there.

**The static share *s*** [E]. Of E0's entries not at rest, the reason is most often the pair slot left open (an entry by
fall-through, or after an instruction the interpreter ran) or a result still in flight from such an instruction. An
open slot whose first instruction cannot pair converges at *c* = 1; one that pairs shifts the pairing parity until a
dependent instruction issues alone in both models, which on the EE's dependency-dense code is usually the next one or
two. Taking 70 % of those entries at *c* about 1.5, 15 % at about 3.5 and 15 % never before their exit (3.64 steps an
entry, E0's mean for them): about 27 M of the arena's 121 M covered steps in prefixes, so ***s* about 78 %, 72 to 85 %**.

**The nanoseconds** [E]. The code's bytes are those E0's ring measured at 103: 2.14 to 2.25 ns a guest instruction on
the desktop and 2.32 to 2.36 on the handheld with today's entries, now for every covered step rather than 61 % of them.
The prefix's own cost is in the shared exit, where the ring cannot see it: a copy of the issue model (about 900 bytes),
then one issue and one comparison a step of the prefix, estimated at **15 to 35 ns an entry not at rest**, about 1.5 to
3.5 ns a guest instruction of those entries. Whether that is less than the Cranelift function's cost on the same
entries is not known from the ring, so **the arena window's wall time with the baseline dispatched is recorded beside
E0's**, two rounds each.

**P-E4** [E] said the working set would be within 1.15 times the static form's if *s* ≥ 90 % and the prefix ended within
four instructions. Here the working set's code is the static form's by construction (the ratio is 1.0 at any *s*), and
*s* instead prices the shared exit's work: the share of steps whose cycles are constants rather than computed.

**Stop rules** for this step: the working set saves under 63 hot bytes a guest instruction, or any entry's cycles or
model differ from the issue model's.

### 15.2 The prefix, and why it is exact

**What runs.** The entry's test of §14.3 stays in the code; where it fails it stores 1 to `Context::moved` and jumps on
into the block's entry, in place of the Cranelift function (`lower.rs`). The code past the entry is the static form's,
unchanged, since it holds no cycle. Each exit's helper (`blocks/baseline.rs`) then reads the flag. At rest it applies the
record as E0 did. Otherwise it copies the core's issue model and runs `prefix::run`: at each step *k* from 0 it asks the
model whether it now converges with the static form (`Timing::converges`), and if not, issues step *k* through
`Timing::issue` itself and notes its cycle. It stops at the first convergence, or at the exit's own step count.

**The table.** For the prefix each block lays a table first in its data (`prefix::table_bytes`): each step's resources,
category and row as the model takes them, for each *k* up to `kmax` what the steps from *k* on read before they write
it, and the static snapshot after *k* steps. `kmax` is at most `PMAX` (8) and never past the branch, so that every
snapshot the prefix compares is the same for both outcomes. A record finds the table from its own offset in the data.

**Convergence** (`Timing::converges`, the model's own module): the model *D* after *k* steps converges with the static
snapshot *s* after *k* steps at the base *b* = *D*.last − *s*.last when (1) the pair slot and the pending group end are
the same, (2) with the slot open, the last category, pipe and writes are the same, since only a pairing reads them,
(3) the floor is the same shifted by *b*, or both at most their last issue plus one, where no issue rule can see it, and
(4) every resource the rest of the block reads before writing it is ready at the same cycle shifted by *b*, or ready in
both by their last issue plus *q*, with *q* = 0 where the slot is open and 1 where it is closed. The argument for (4):
with the slot closed the next instruction issues at the last plus one or later, so a ready cycle bounded by that never
decides a maximum; with it open the next may pair at the last itself, so the bound is the last; and every later issue
cycle is no earlier, so the bound holds to the block's end. Every other field the rest of the block can read is then
equal up to *b*, the model is invariant under a shift of every cycle (no rule reads a cycle's absolute value), and so
every later step issues at the static form's cycle plus *b* and leaves the static model plus *b*. At *k* = 0 the
condition is the entry's test of §14.3 exactly.

**What the exit commits.** Past a convergence at *c*: the steps' cycles are the prefix's before *c* and *b* plus the
record's constants from *c*; the model is the record's snapshot applied at *b*, with the ready cycles of the resources
only the prefix wrote taken from *D* (condition (4) let them differ where no later step can see the difference, so the
snapshot does not hold them); Random moves by the prefix's unpaired cycles plus the snapshot's after *c*. With no
convergence before the exit the model is *D* itself, which is `Timing::issue` run over every step.

**Proven, not only argued** (`ee/jit/baseline/tests.rs`, `the_prefix_then_the_static_form_is_the_issue_model_from_every_entry`):
over random blocks of the slice, single and dual issue, each with eight random entry models made by random prefixes
(multiplies and dividers among them, taken branches among them), at every exit's step count and for both outcomes, the
steps' cycles, the model field for field, the unpaired cycles and the pairing equal the issue model run directly. Of
the 35,742 random entries not at rest it met, 13,673 converged at step 1, 18,154 later, and 3,915 never before the
block's end.

### 15.3 The classes' estimate, written before their build

*Set on 2026-10-29, after the prefix's bytes and share were measured (§15.4) and before any of the classes' code was
written. Nothing in this subsection is measured.*

**What is added**: the rest of S36's and S37's classes (`shape.rs`): HI and LO's moves on both pipelines, the multiplies
and multiply-adds of both, the divides of both, SA's four moves, the twelve traps, PREF, and the unaligned loads and
stores (LWL, LWR, LDL, LDR, SWL, SWR, SDL, SDR). The static form's snapshot gains the two multipliers' free cycles, and
the convergence test reads them as it reads a ready cycle. HI, LO and SA are held as block values beside the general
registers, written back at an exit only where an instruction before it wrote them, so a rollback finds them untouched.
The macro-assembler gains a multiply and a divide (`Assembler` 1.1, an extension: two operations, nothing changed).

**The bytes** [E]. On the arena run of §15.4 the blocks the baseline covers took 126.8 M of the 154.7 M compiled steps
(82 %); the other 18 % ran the Cranelift function, 524.8 hot bytes a guest instruction on blocks like them. The working
set over every compiled step is about 0.82 × 105.3 + 0.18 × 525 ≈ **181**. With every shaped block covered (all of
them hold only S36's and S37's classes, so only blocks of one instruction stay out) and the new classes a little
larger than the average word (a divide's zero and overflow paths are cold; an unaligned access is an aligned one plus
a shift and a merge), the working set is about 0.99 × 107 + 0.01 × 525 ≈ **111**: a saving of about **70 hot bytes a
guest instruction** over every compiled step; the stop rule's half is 35.

**The static share** [E]: of every compiled step, 59.5 % today (92.0 M of 154.7 M); with the new blocks covered at about
the share the covered ones have (72.6 %), about **71 %, 68 to 74 %**: a divide's 37 cycles in flight at a block's
entry converge only once a step waits for it.

**The nanoseconds** [E]: the ring at about 107 bytes, near E0's 103 (desktop 2.14 to 2.25, handheld 2.32 to 2.36).

**Stop rules**: the working set over every compiled step saves under 35, or any class cannot be made exact.

### 15.4 The prefix's results (measured 2026-10-29)

The arena window (`jak-arena`, 300 fields from the state at 9,000, `--defaults`) with the baseline dispatched and
`EMUSEN_SAPPHIRE_FORMS_BYTES=1`, all figures from that one run: 13,082 of the blocks compiled are the baseline's,
132,133 guest instructions.

**The bytes, the gate.** The exact form is 105.3 hot bytes a guest instruction (E0's 103.4 with the entry's test at 9.4
rather than 7.5: the flag's store and its jump back cost 1.9 where E0's jump through the data word cost less; the
measured build lays them hot before the entry, so that every branch of the test stays short). The Cranelift function
on the same blocks in the same run, the form E0 ran where the test failed, is **524.8**, not the 424 the estimate took
from the redesign's page (that figure was the arena's whole mix with every lever; the dispatcher runs today's form
without the levers). The working set on the covered blocks, weighted by steps:

| | static form | the rest | working set |
| --- | --- | --- | --- |
| E0 | 61.4 % at 103.4 | 38.6 % at 7.5 + 524.8 | **269** |
| E1, the prefix | every step at 105.3 | none | **105.3** |

**The saving is 164 hot bytes a guest instruction on the covered blocks' working set, against an estimate of 127**; the
stop rule's 63 is met.

**The static share *s*.** Of 126.79 M covered steps, 92.02 M (**72.6 %**) took their cycles from the static form's
constants, against an estimate of 72 to 85 % (central 78): the estimate's low end. Of 32.38 M covered entries, 18.70 M
(57.8 %) were at rest; of the 13.68 M that were not:

| converged after | 1 step | 2 | 3 | 4 | 5 | 6 | 7 | 8 | never before the exit |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| entries | 4.92 M | 0.54 M | 0.16 M | 62 k | 27 k | 37 k | 121 | 225 | **7.93 M (58 %)** |

So the estimate's 70 % converging early was wrong: an entry not at rest either converges at once (36 %) or, more often,
not at all before it leaves (58 %), and those run the issue model over every step in the shared exit. Why they do not
converge is not yet counted; the candidates are short entries (3.9 steps an entry on average), the pairing parity of
§3.2, and the limit of the test at the branch (`kmax`).

**The nanoseconds of the real field.** The window's mean milliseconds a field, two rounds each, interleaved on the
desktop (other jobs on the machine; the EE is 13 % compiled in this window, the rest interpreted or idle-skipped):

| | round 1 | round 2 |
| --- | --- | --- |
| the Cranelift tier (baseline not dispatched) | 754.1 | 760.9 |
| E0's baseline (7cef70e11) | 792.7 | 785.8 |
| E1's baseline, the prefix | 788.7 | 793.9 |

E1 and E0 are equal within the rounds' spread: the prefix's work in the shared exit costs about what the Cranelift
function's entries saved, so the bytes the prefix removes do not show in this window's wall time. Both baselines are
about 4 % slower than the Cranelift tier on this window, which E0 did not measure; the cause is not yet located (the
sampler's EE row is §7.4's measurement from E2), and it is recorded as a finding, not as the prefix's.

**Exactness.** The picture, sound and state digests of the arena window are the base's (611B58F7CFE67E8B,
BCA8F00F4743A955, 8B710CA05FDDF3A0) in every one of these seven runs, the shadow's included.

### 15.5 The classes, as built

- **`emusen-assembler` 1.1**: `mul` (the product's low 64 bits) and `divrem` (quotient and remainder at 32 or 64 bits,
  signed or not, the divisor outside the backend's `DIV_REGS`, which on x86-64 are rax and rdx); graded against a Rust
  model in every register (`multiply_and_divide_match_their_models_in_every_register`). Nothing of 1.0 changed.
- **The planner's table** (`R5900Isa`) holds every S36 and S37 class `shape.rs` names. The planner itself is
  unchanged: HI, LO and SA are not general registers, so its reads and writes do not carry them, and the lowering knows
  them by the instruction (`specials`). §3.1 put them in the `Isa` table; at E1 nothing the planner decides needs them.
- **Values** (`lower.rs`): LO, HI, LO1, HI1 and SA are value ids beside the general registers' halves, homed by the same
  linear scan, read from the core at first use and written back at an exit only where an instruction before it wrote
  them; the state after the last load or store saves them before a later write, as it saves a register, so a rollback
  writes back what stood there.
- **Multiplies** (`multiply`): both operands sign- or zero-extended to 64 bits and multiplied once, the accumulator
  added for MADD; LO and rd the low word sign-extended, HI the high word. **Divides** (`divide`): a zero divisor and,
  signed, a divisor of -1 out of line (the first takes `quotient_signed`'s and `quotient_unsigned`'s results of
  `Sapphire_Disputes.md` D-6, the second negates, which is the wrapping quotient of the minimum too), so the hardware
  divide never faults. **SA** (MFSA, MTSA, MTSAB, MTSAH) as `ops.rs` writes it. **Traps**: the compare, and the exit
  before the instruction where it holds, as an overflow's; the interpreter raises.
- **PREF** is a one-byte load that loads nothing: a hit in either way or the scratchpad does nothing, and everything else
  (a miss, an uncached page, a translation that fails) leaves to the interpreter, which prefetches or ignores it. An
  uncached PREF leaves rather than passing, so that its record never names a line a later access could reuse.
- **The unaligned forms** (`unaligned`): the address is kept whole through the translation and the cache's dispatch, so
  the record names the byte as every other access's does; then the unit's address and pointer are made from it, the
  shift from the byte within the unit, and the merge is `ops.rs`'s (LWL's and LDL's from the high end, LWR's and LDR's
  from the low end, LWR keeping the register's upper word unless the unit is whole); a store logs its quadword, marks
  the line dirty, then merges into the line.
- **The static form** carries the multipliers: their free cycles are snapshotted and applied with the ready cycles
  (`Snap::mac`, at the bits `MAC_BITS` of its masks), the entry's test compares them, the prefix's convergence test
  reads them as it reads a ready cycle, and the prefix keeps a multiplier only it used.

### 15.6 The classes' results (measured 2026-10-29), and the stop

The same arena run as §15.4's terms, with the classes built:

| | §15.4 (prefix only) | with the classes |
| --- | --- | --- |
| blocks covered, of those compiled | 13,082 | 13,281 of 15,021 |
| guest words covered, of those compiled | 132,133 | 138,364 of 140,104 (98.8 %) |
| covered steps, of every compiled step | 126.8 M of 154.7 M (82.0 %) | 130.9 M of 152.3 M (86.0 %) |
| the exact form's hot bytes a guest instruction | 105.3 | **104.2** (the entry's test 8.0, laid hot) |
| on the prototype's terms | 82.8 | 83.1 |
| the Cranelift function on the covered blocks | 524.8 | 523.0 |
| the working set over every compiled step | 0.820 × 105.3 + 0.180 × 524.8 = **180.8** | 0.860 × 104.2 + 0.140 × 523.0 = **162.9** |
| *s* over every compiled step | 59.5 % | **63.5 %** (73.8 % of the covered steps) |
| compile time, a block | 40.7 µs | 43.2 µs |

**The saving is 17.9 hot bytes a guest instruction against an estimate of 70; the stop rule's 35 is not met, so E1
stops here.** The classes cover what the estimate said they would (98.8 % of the words), and their code is no larger
than the average word's. **What the estimate got wrong**: every one of the 1,740 blocks still on the Cranelift tier is
a block of one instruction, which E0's `covers` leaves out, and together they run 14.0 % of the compiled steps, where
the estimate took them as about 1 %. Which code they are was not counted. Covering them is the next lever for the
working set, and it is a choice of scope rather than a correction of this stage (§15.8).

The arena's digests with the classes are the base's (611B58F7CFE67E8B, BCA8F00F4743A955, 8B710CA05FDDF3A0). Of the
13.59 M entries not at rest, 5.03 M converged after one step and 7.72 M (57 %) never before their exit, as in §15.4.

### 15.7 Exactness (measured 2026-10-29)

| Check | Base | E1's baseline dispatched (the final build) |
| --- | --- | --- |
| Jak's first 60 dock fields (picture, sound, state) | 0F8671F57AB63C25, D5F15509CCDD1709, 818361BFE5D2DBF9 | the same |
| the dock, 300 fields from 6,000 | 830E3FEFEE1BE6B1, 2C225C73DB4D3F9D, 9CF1BED1AA64EAEE | the same |
| the arena, 300 fields from 9,000 | 611B58F7CFE67E8B, BCA8F00F4743A955, 8B710CA05FDDF3A0 | the same, and in every earlier run of §15.4 and §15.6 |
| `tests/blocks.rs`, test-checked, `EMUSEN_SAPPHIRE_BASELINE=1` for every test | — | 29 of 30, the new test among them; the 30th is below |
| the prefix and the static form against the issue model (`tests.rs`), with E1's classes in the random blocks | — | every entry, every exit, both outcomes |
| `emusen-assembler`'s grading | — | 10 of 10 |

**The one test not run to its end**: `every_load_and_store_runs_compiled_on_every_region`, with every test on the
baseline, ran past 18 minutes and was stopped; without the variable (the baseline dispatched only by its own tests) it
passes in 173 s. It walks a long straight program in which a block starts after almost every access that leaves, and
with both forms compiled a block's code passes the 256 MiB code budget again and again (two flushes in its first 90 s,
counted with a temporary print): the compiler's time, not a result. The steps' cycles were moved from every exit's
record into the block's table once (`prefix::Table::times`), 128 bytes a record less (the arena's data 924.2 bytes a guest instruction to 634.9, against E0's 810.8; hot bytes 104.1 in that run), which did not change that. Its
instructions are graded in the random programs, which hold every load, store and PREF on the cached, uncached,
scratchpad and unmapped regions, and in S7's programs, all passed on the baseline.

**Seeded faults**, each built alone, run against the random programs of `tests/blocks.rs` (test-checked, every test on
the baseline) or the prefix's own test, and reverted:

| Fault | Caught by |
| --- | --- |
| the convergence test's bound the last issue plus one with the slot open | the prefix's test (`tests.rs`) |
| what only the prefix wrote not kept from its model | the prefix's test |
| the steps past the convergence rebased from the entry's last issue, not the prefix's base | all 4 random-program tests (the board's time one or two cycles apart) |
| a signed divide by zero giving all ones for a negative dividend | all 4 random-program tests |
| LWR always sign-extending, the register's upper word not kept | 1 random-program test |
| MADD's accumulator without HI | all 4 random-program tests |
| **the owed fault**: the replay's guard testing the entry's next event, not the board's | **not caught**, neither by the new test nor by the DMAC's two |

**The owed test** (§14.8): `loads_and_stores_beside_a_busy_vif1_leave_the_interpreters_machine` keeps VIF1 busy with an
UNPACK of 256 words sent from the scratchpad while the blocks write the words still to be sent, and the two machines
stay equal. Its blocks meet the unit's events at their entry (12,777 entries left to the interpreter at the event, 110
rewinds), but no replay met an event moved earlier inside its guard (no DMAC stop), so the seeded fault passes. **The
gap stays open**: reaching it needs the DMAC held by a full FIFO at a block's entry and freed by the VIF during the
replay, which this program does not arrange; how the board schedules VIF1's FIFO was not read further.

### 15.8 What E1 leaves

- **E1 stopped at its rule** (§15.6): the working set over every compiled step is 162.9 hot bytes a guest instruction,
  not the estimate's 111, because the blocks of one instruction (14 % of the arena's compiled steps) stay on the
  Cranelift tier. Covering them is the remaining lever of this stage's kind: it would take the working set towards
  0.99 × 104 ≈ 103 if their code is like the rest.
- **The prefix's cost is in the shared exit** (§15.4): 57 % of the entries not at rest never converge before they leave,
  and run the issue model over every step there; the arena's wall time with the prefix equals E0's. Why they do not
  converge (short entries, pairing parity, `kmax` at the branch) is the first count to make before any change.
- **Both baselines are about 4 % slower than the Cranelift tier on the arena's wall time** (§15.4), unmeasured before.
  The sampler's EE row (§7.4) should locate it before E2 builds on the baseline.
- **The design's corrections at E1**: the prefix needs no code and lives in the shared exit (§15.1); convergence is
  tested per entry at run time rather than found by the planner for a class (§3.2); HI, LO and SA stay out of the
  planner's `Isa` table (§15.5); the multipliers' free cycles join the static snapshot; the steps' cycles are a block's,
  not each exit's.
- **The owed test's fault** (§15.7) is still not caught.

### 15.9 The ring at the bytes reached (measured 2026-10-29)

R0's `ring mix` (2,048 hot blocks, 8,152 cold, 36 laps a pass) asked for 104 bytes, the final build's exact form; the
ring's code came to 103.7 with today's 2.87 instructions an entry and 107.4 with §4.2's 14.5. Two rounds each, ns a
guest instruction:

| | today's 2.87 an entry | §4.2's 14.5 |
| --- | --- | --- |
| desktop (other jobs on the machine) | 2.06 to 2.15 | 1.92 to 1.98 |
| handheld, on the charger (ACAD online 1 before and after), `hh.sh guard` first | **2.39 to 2.44** | 4.72 to 4.76 |
| E0 at 103 (§14.7), handheld | 2.32 to 2.36 | 2.66 to 3.00 |
| the marks, handheld | **2.75** (Jak's opening), **2.28** (the arena's fields), two threads; 1.53 one thread | |

With today's entries the handheld runs under the mark for Jak's opening and 1.05 to 1.07 times the arena's, about E0's
point, as the bytes are. With §4.2's longer entries it ran 4.7 ns in both rounds, against E0's 2.66 to 3.00 at nearly
the same code size (106.7 against 107.4 bytes, 15.9 MB of code): the ring's code is the same generator's, so the
difference is the handheld's state on the day, not E1's, and it is recorded without a cause. The ring is synthetic: it
does not hold the shared exit's prefix, whose cost §15.4 measures in the window's wall time instead.

## 16. The covered blocks without their Cranelift function (2026-10-29)

*Approved by the tester on 2026-10-29, from `Sapphire_Speed.md` §60.4 and §60.7's third lever: a block the baseline
covers stops paying for a Cranelift compile, and the code budget's drops are counted rather than inferred. Behind the
`baseline` feature, as every stage before E3; the default build is unchanged.*

### 16.1 The estimate, written before the measurement

*Nothing in this subsection is measured.* §60.4 found every covered block compiled twice, as a Cranelift function and
as tailored code, with Cranelift and regalloc2 94 % of the compile thread and a covered block 26 KB against the Cranelift
tier's 12 KB, so that the 256 MiB budget dropped every block in mid-window. Without the function [E]:

- **P-B1**: no drop on the arena or the dock, the tailored code alone at a few KB a block;
- **P-B2**: the compile thread's busy share falls by at least ten times, at 43 µs a block against about 15 ms;
- **P-B3**: the compiled share at least the Cranelift tier's (40.1 % on the arena, §60.5), since the queue empties far
  faster; and the arena's field at or below the Cranelift tier's, removing §60.4's 46 ms;
- **P-B4**: compiled on the dispatcher's thread, about 50 blocks a field at 43 µs, about 2 ms a field, and the compiled
  share near §60.5's waited 45 to 52 %.

### 16.2 What was built

- **`emusen-jit` 1.2** (`EmuSen_Jit.md` §8): the thread asks for the tailored code first and compiles the function only
  where `Native::needs_function` says (a fallback word, or a measurement that reads it); a tailored block it cannot place
  is refused and makes the budget drop the module; `Jit::compile_now` makes and publishes a tailored block on the
  caller's thread; `Jit::spent` and `Jit::held` count across drops.
- **Sapphire**: the baseline's `Native` has no fallback word, as it had none since E1, and asks for the function only
  under `EMUSEN_SAPPHIRE_FORMS_BYTES=1` (whose shadow reads the function, §14.4) or `EMUSEN_SAPPHIRE_BASELINE_FUNCTION=1`
  (E1's form, both compiled, kept for this section's "before"). `EMUSEN_SAPPHIRE_BASELINE_NOW=1` (`Blocks::now`)
  compiles a covered block on the dispatcher's thread when it reaches the threshold, in place of queueing it; a block
  `compile_now` declines is queued as before. The tests' `eager` dispatcher reads the same variable.
  `EMUSEN_SAPPHIRE_BUDGET_MIB=<n>` sets the code budget, for measurement only and off by default; the baseline's own
  budget is §16.5's.
- **The bench** prints a line `ee recompiler`: the budget's drops in the window and the budget, the bytes held at the
  end, the compile thread's busy share of the window's wall time and its blocks a field, and the dispatcher's own
  compile time and blocks a field.

### 16.3 What relied on the function, and where each goes now

Read from the code on this branch:

1. **An entry not at rest.** Since E1 the tailored code's prefix handles it in the shared exit (§15.2); the code has no
   fallback word (`ee/jit/mod.rs`, `fallback: None`), and the baseline's lowering never emits `jump_via_data`, the
   assembler's only jump through a data word. So no path of the tailored code reached the function before this change.
2. **A rollback** (`blocks.rs`, `block`, the arm for an exit other than `DONE`): the stores are undone or the state
   after the last access kept, and the steps run again on the interpreter. The block's code is not entered again.
3. **The verifier.** `baseline::covers` refuses `Options::verify`, so the per-step verifier's blocks are never covered
   and keep their Cranelift function; the per-block verifier steps the reference interpreter after each exit and calls
   no code.
4. **No tailored code for a covered block** (the planner or `static_times` declines it, or the lowering returns none):
   `needs_function` is true for an absent `Native`, so the function is compiled and published as before.
5. **The code memory full.** 1.1 published the function in place of tailored code it could not place. With no function
   the slot is refused, the dispatcher's `settle` makes the block `Refused` and the interpreter runs it, and the next
   `block` call finds `over_budget` true and forgets every block.
6. **`compile_now`** declines exactly the cases of 4 and of a `Native` that needs its function, and the dispatcher then
   enqueues the block, so the sync path never publishes less than the queue would.

Every case ends in the tailored code or the interpreter, which are exact against each other by §14.3 and §15.2; which
of them runs a step changes no result.

**How it fits E2.** E2's runtime (§8) brings 2.0's reservation with its two arenas, link slots and invalidation, and
decides compiling on the dispatcher's thread (§3.9). This section changes none of 1.1's interfaces: the `Lowering`
trait is unchanged, `Native` gains one field, and `compile_now` uses 1.1's `native::Memory` as it stands. When E2 builds
its arenas, `compile_now` places into them in place of its own reservation, and the measurements below are its
evidence for the decision.

### 16.4 Results (measured 2026-10-29)

One build of the bench with `--features baseline`, desktop, one round of each configuration at CPU weight 100, started
only while no build and no other timed bench ran; the one-minute load at each run's start and end is given. The
configurations: **Cranelift**, the feature compiled and not dispatched (§60.1's run B stands for the shipped tier);
**E1's form**, `BASELINE=1` with `BASELINE_FUNCTION=1`, both compiled as before this section; **tailored only**,
`BASELINE=1`; **now**, with `BASELINE_NOW=1` as well; and **at 480 MiB**, tailored only with the measurement-only
`EMUSEN_SAPPHIRE_BUDGET_MIB=480` (off by default; the shipped budget stays 256 MiB). Drops are the budget's flushes
inside the timed window; the busy share is the compile thread's nanoseconds over the window's wall time (`Jit::spent`).

| Arena (9,060 to 9,360) | ms a field | drops | compile thread busy | blocks compiled a field | compiled share | load |
| --- | --- | --- | --- | --- | --- | --- |
| Cranelift | 774.3 | 2 | 64.6 % | 195.4 | 40.7 % | 3.8 to 3.5 |
| E1's form | 786.3 | 5 | 34.8 % | 205.8 | 32.4 % | 3.5 to 3.2 |
| tailored only | 802.5 | 5 | 4.0 % | 496.2 | 47.3 % | 3.2 to 1.7 |
| now | 779.2 | 5 | 2.2 %, and 19.3 ms a field on the dispatcher (426.6 blocks) | 39.5 + 426.6 | 47.5 % | 1.7 to 1.9 |
| **at 480 MiB** | **776.6** | **0** | **0.5 %** | 51.5 | **48.5 %** | 3.4 to 2.6 |

| Dock (6,060 to 6,360) | ms a field | drops | compile thread busy | blocks compiled a field | compiled share | load |
| --- | --- | --- | --- | --- | --- | --- |
| Cranelift | 230.2 | 0 | 44.7 % | 26.0 | 46.2 % | 1.9 to 3.1 |
| E1's form | 235.2 | 0 | 25.2 % | 16.1 | 46.2 % | 3.1 to 3.9 |
| tailored only | 233.3 | 5 | 11.4 % | 463.1 | 50.8 % | 3.9 to 3.2 |
| now | 249.1 | 5 | 4.8 %, and 16.8 ms a field on the dispatcher (357.6 blocks) | 27.6 + 357.6 | 51.4 % | 3.2 to 2.4 |
| **at 480 MiB** | **231.3** | **0** | **1.3 %** | 49.7 | **53.2 %** | 2.6 to 2.3 |

**Exactness**: every run above gave the base digests (arena `611B58F7CFE67E8B`, `BCA8F00F4743A955`, `8B710CA05FDDF3A0`;
dock `830E3FEFEE1BE6B1`, `2C225C73DB4D3F9D`, `9CF1BED1AA64EAEE`), and so did the short window (6,060 to 6,120, weight 20)
tailored only and now (`0F8671F57AB63C25`, `D5F15509CCDD1709`, `818361BFE5D2DBF9`).

**The predictions.**

- **P-B1 is refuted at 256 MiB** and holds at 480. A tailored block is not a few KB but 8.7 KB (the arena's 204.5 MB
  for 23,420 blocks), mostly its data: the prefix's table and the exits' records (§15.7's 634.9 data bytes a guest
  instruction). With compiles cheap, every block reaching the threshold is compiled at once, the working set passes
  the budget, and each drop recompiles it: about 470 blocks a field, against the Cranelift tier's 26 to 195. At
  480 MiB the arena holds 359.5 MiB with no drop and compiles 51.5 blocks a field.
- **P-B2 holds**: the compile thread's busy share falls from 34.8 % (E1's form) to 4.0 %, and to 0.5 % with no drops.
- **P-B3 holds for the share, not for the time**: the compiled share rises 8 to 16 points over E1's form and 7 to 8
  over Cranelift's tier, but at 480 MiB the field only returns to the Cranelift tier's (776.6 against 774.3 on the arena,
  231.3 against 230.2 on the dock, inside one round's spread). §60.4's 46 ms are recovered; the steps moved into compiled
  code buy nothing here, since the covered blocks leave at the event four times as often (58.7 M entries against
  14.2 M over the arena) and roll back twice as often, which is the one-check form's (§14.3, 4), not this section's.
- **P-B4 holds for its cost and not its value**: 43 to 47 µs a block on the dispatcher, 17 to 19 ms a field while the
  drops churn; with the budget's drops removed the compile thread is under 2 % busy, so the queue no longer delays
  anything and compiling now buys nothing measurable.

**The earlier inference corrected.** §60.1 put Cranelift's compiles at about 15 ms a block from the counters since the
last drop; across drops (`Jit::spent`) its thread did 58,600 compiles on the arena in 150 s, **about 2.6 ms a block**,
and it was busy 64.6 % of the window by its own clock (the sampler's 100 % counts its time out of the batches too).
The Cranelift tier also drops twice in the arena's window, which §60 could not see.

### 16.5 What this leaves

- **The budget.** Decided 2026-10-29 by the tester: with the baseline dispatched the EE's budget is 480 MiB
  (`blocks::BASELINE_BUDGET`); every other build and setting keeps 256 MiB (`blocks::BUDGET`). The compiler's code
  reservation already holds it: 1 GiB, of which the half for cold code and data holds 512 MiB, and the runs at 480 MiB
  held 359.5 MiB at most with no drop for a full code memory (`Jit::full`, 0 in both). Were the reservation to fill
  first, the module is dropped as at the budget (`EmuSen_Jit.md` §8), so the limit is never a lasting refusal. A smaller
  prefix table would lower both.
- **Compiling now** is kept as an option and is exact; E2 decides it (§3.9), and these numbers say it is not needed
  while the compile thread is idle.
- **The tests**: `tests/blocks.rs` under test-checked with `EMUSEN_SAPPHIRE_BASELINE=1` for every test — see §16.6.
  No seeded faults were run for this section: the work was parked on 2026-10-29 for a structural redesign of the fast
  face, and the tester asked for the tests alone.

### 16.6 The tests (measured 2026-10-29)

| Check | Result |
| --- | --- |
| `tests/blocks.rs`, test-checked, `--features baseline`, `EMUSEN_SAPPHIRE_BASELINE=1` for every test, at the 480 MiB budget | **30 of 30, in 10.7 s** |
| `emusen-jit`, with `native` and without | 15 of 15, and 12 of 12 |
| Sapphire with and without the feature, and MarsRT | compile (`cargo check`, tests and the bench) |

`every_load_and_store_runs_compiled_on_every_region`, which ran past 18 minutes under E1 with every test on the baseline
and was stopped (§15.7), now passes with the rest of the file: its time was the compiler's, Cranelift compiling every
covered block beside its tailored code and the budget dropping them again and again, and neither happens now. That
closes §15.7's one test not run to its end.
