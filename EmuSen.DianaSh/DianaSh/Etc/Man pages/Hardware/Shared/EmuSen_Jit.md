# EmuSen_Jit — a recompiler's shared machinery

*Started 2026-10-25, at Sapphire's stage S35.* `emusen-jit`, `EmuSen/Cores/Shared/emusen-jit/`: the compiler's thread
and its queue, the slots a block's code is published in, the Cranelift module with its W^X batches and its code budget,
the counters, and the shape of the verifier that steps an interpreter beside the blocks. It is MarsRT's
`cpu/blocks/jit/mod.rs` and the processor-free half of `cpu/blocks/verify.rs` (`Mars_Native.md` §5.8.4, §5.8.5),
moved out of the N64 core so that the PlayStation 2's recompilers (`Sapphire_Plan.md` §9.6, stages S36 to S41) use the
same thread, publication and verifier rather than a copy. It is generic over a processor's **lowering**: a trait that
turns a block's words into Cranelift IR and names its exits.

*Decided 2026-10-24 by the tester* (`Sapphire_Plan.md` §11 Q10, as recommended): the crate is extracted from MarsRT at
S35, MarsRT's own tests and speed the gate. The plan's §9.6 gives the reason and the limit: two processors need the same
thread, publication and verifier, which are subtle (W^X, atomics ThreadSanitizer sees, the code budget) and were proven
once; the emitters lower different semantics and share nothing worth a generic layer, which is `Diamond_Plan.md` §9.1's
argument against a shared interpreter.

The marks are `Diamond_Plan.md`'s: **[M]** measured on 2026-10-25 with the tool named; **[R]** read from a source;
**[A]** argued; **[C]** a choice.

## 1. Its one job, and what it leaves to its users

**In it**: the compiler's thread, started at the first block a dispatcher hands over and stopped when its owner drops
it; the queue between the dispatcher and that thread; the slot each block's code is published in; the Cranelift module
the thread owns, written in batches whose pages become executable once and never writable again; the code budget past
which the module is dropped whole; the counters the thread keeps; and the verifier's shape, a reference machine stepped
once for every instruction the blocks run and compared after each.

**Not in it**: any instruction set, any block shape, any dispatcher, any machine. The lowering (MarsRT's
`cpu/blocks/jit/emit.rs`, 1,070 lines), the block shapes and the cache (`shape.rs`, `cache.rs`), the decoded tier, the
dispatcher with its guard and its exits (`cpu/blocks/mod.rs`), and the machine the verifier compares (`verify.rs`'s
`Interpreter`) stay with the processor. Sapphire's EE and VU emitters will be Sapphire's own (S36 to S41). Nothing in
the moved code named the VR4300: it was searched for every `Cpu`, `MemoryBus`, RDRAM, TLB and Mars identifier, and the
only ones were the compiled block's signature, the context it reads and the exit values, which stay in MarsRT [R].

**Dependencies**: `cranelift-codegen`, `cranelift-frontend`, `cranelift-jit`, `cranelift-module` and
`cranelift-native` 0.136.0, the version MarsRT measured (`Mars_Native.md` §5.8.4), Apache-2.0 WITH LLVM-exception,
compatible with the project's GPL-3.0. Nothing else: not `emusen-native`, since nothing here is a core's state or ABI.

**Who depends on it**: MarsRT (`marsrt -> emusen-jit`, a path dependency named in its `Cargo.toml` and in
`Mars_Native.md` §5.8.10), the first core to; Sapphire since S36 (`sapphire -> emusen-jit`, named in its `Cargo.toml` and
in `Sapphire_Native.md` §44), for the EE, and since S40 for VU micro mode (`Sapphire_Native.md` §45); Diamond may adopt it
when its own recompiler is wanted (`Diamond_Speed.md` §7.2). The CI job that tests every crate under `EmuSen/Cores/Shared/` tests this one without a
change to the workflow.

## 2. The interface, version 1.0.0

| Item | What it is |
| --- | --- |
| `trait Lowering: Clone + Send + 'static` | a processor's lowering; cloned into each compiler thread it starts |
| `Lowering::Options` | how a block is compiled, as the processor chooses (MarsRT's: registers held, the verifier's calls, inline stores, the variant beside a running signal processor) |
| `Lowering::NAME` | the thread is `<NAME>-compiler`; what it prints starts `<NAME>:` |
| `Lowering::EXITS` | the names of the values a compiled block returns, by value; `exit_name::<L>(v)` reads them |
| `Lowering::lower(module, func, functions, words, start, options)` | writes one block's signature and body into `func`, which the crate has cleared; an `Err` is a refusal |
| `Slot` | where a block's code is published: `published()` (the address, read with Acquire), `settled()` (published or refused), `wait()` |
| `FAILED` | what a refused block's slot holds; the dispatcher runs that block another way for ever |
| `Counters` | queued, finished, compiled, nanoseconds compiling, bytes of code, failed; atomics, read with Relaxed except `finished` |
| `Compiler<L>` | the thread and its module: `new(lowering, perf_map)`, `enqueue`, `drain`, `counters`; dropping it stops the thread and frees every byte of code |
| `Jit<L>` | the compiler as a dispatcher holds it: `enqueue` (starts the thread at the first block), `counters`, `drain`, `reset`, `over_budget`, and the settings `budget` and `perf_map` with the count `flushes`; a clone keeps the settings and no code |
| `verify::Reference` | a processor's reference machine: `step`, `differs` (what a step can change), `differs_whole` (the rest, at a run's end), over a borrowed `Subject` |
| `verify::Verifier<R>` | the reference and its step count: `follow`, `check`, `check_all`, each panicking with the first difference |

**What a lowering must do.** Fill `func`'s signature and body and finalise its builder; it does not declare or define
the function. The crate declares it `Local` as `b<start, 8 hex digits>_<serial>`, defines it, measures its bytes and
publishes its address. The compiled function's signature is the lowering's alone: the crate publishes an address, and
the dispatcher that knows the signature turns it into a function pointer (MarsRT's `jit::published`). A lowering that
calls back into the processor does so by an address it embeds (`iconst` and `call_indirect`), as MarsRT's handlers and
verifier hook do.

**What a dispatcher must do.** Treat an address as dead after `reset` or an `over_budget` that returned true, and forget
every block that holds one before it runs again (§3.4). Keep its own policy: when a block is handed over (MarsRT's
sixty-fourth entry), whether it waits for the code (MarsRT's tests do, `Slot::wait`), and what a refused block runs.

## 3. The mechanisms

### 3.1 The thread and its queue

The thread owns the module; nothing of a machine crosses to it but the block's words, its start and its options, and
nothing comes back but one address. The queue is a spin lock of the crate's own, taken with Acquire and let go with
Release, with `park_timeout` of a millisecond while it is empty, not a channel: ThreadSanitizer sees the crate's own
atomics and not std's, for the reason `Mars_Native.md` §5.6.7 gives [R]. Each wake takes every job queued, compiles them
in order, and makes them executable together (§3.3).

### 3.2 Publication

A slot is one word: zero until the thread is done with the block, `FAILED` (1) if the lowering or Cranelift refused it,
else the code's address. The address is stored with **Release after the batch's pages were made executable**, and read
with **Acquire**, so that a dispatcher that sees the address sees the bytes behind it and the pages' protection. This
is the one ordering the crate exists to get right, and §5's positive control shows ThreadSanitizer finding it broken.

### 3.3 W^X, in batches

`cranelift-jit` writes a batch's functions into fresh pages and makes them readable and executable in one call
(`finalize_definitions`), which starts the next batch on a new page: a page that holds published code is never
writable again [R, `Mars_Native.md` §5.8.4; M, §5's `code_pages_are_never_writable_and_a_later_batch_leaves_them_alone`].
A batch is every job queued when the thread wakes, so the pages are shared by blocks compiled together.

### 3.4 The code budget

Code is never freed a block at a time: a block whose words changed is dropped by its dispatcher, and its code stays in
the module. `Jit::over_budget` compares the bytes compiled with the budget and, past it, drops the compiler and its
module whole, counts a flush and returns true; the dispatcher then forgets every block and compiles them again as they
run. Code exactly at the budget is kept. **The thread is retired, not joined** (since 2026-10-27, `Sapphire_Speed.md`
§55): `over_budget` sets the thread's stop and lets it go; the thread refuses every job still queued (publishing `FAILED`,
so that no `Slot::wait` waits for ever), finishes the block in hand, and frees the module itself as it exits. That is
safe because the dispatcher has already forgotten every address the module published, and it was measured to matter:
Sapphire's VU1 dispatcher joined the thread at its budget while it compiled a long queue, and Jak and Daxter's arena
spent frames of 2 to 50 seconds there (`Sapphire_Speed.md` §51.6). A dropped compiler (`reset`, or the `Jit`'s drop)
still joins its thread, which now refuses its queue rather than compiling it. MarsRT checks it at the start of every run with a budget of 256 MiB, which no
game has reached (GoldenEye's 600 frames compile 20 MB, `Mars_Native.md` §5.8.4); a dispatcher that checks before each
run never starts one with more code than the budget, and holds at most one run's compiling past it.

### 3.5 The counters and the perf map

The thread counts the blocks queued, finished, compiled and refused, its time compiling and the bytes of code; MarsRT's
`blocks_counters` reports four of them. With `perf_map` set (MarsRT sets it from `EMUSEN_MARSRT_PERFMAP=1`), each
function's address, size and name go to `/tmp/perf-<pid>.map`, the path `perf` reads, as `Mars_Native.md` §6.3 uses it.

## 4. The verifier's shape

A processor's reference is its interpreter's machine, a step behind or level with the machine the blocks run. The
dispatcher calls `follow` once for every instruction the blocks run (from compiled code by a call the lowering emits,
everything written back first) and on every other path a step takes (an interpreter step, an interrupt at a block's
entry, an exit); `follow` steps the reference once and compares, and `check_all` compares the rest of the machine at the
end of a run. A block the verifier did not follow leaves the reference behind, and the next comparison says so. What a
step is and what is compared are the processor's: MarsRT's compares the processor, the run's derived fields and every
device after each step, and RDRAM at the end (`Mars_Native.md` §5.8.5's first oracle), with messages unchanged by the
move. The idle loop, which both machines run by the same code, is mirrored by MarsRT's own `Interpreter::idle` rather
than stepped.

## 5. The evidence

**The crate's tests**, `src/tests/mod.rs`, on a toy processor of four registers, sixteen words of memory and five
instructions, with its interpreter and its lowering [M, `cargo test --release`, 11 of 11]:

| Test | What it holds |
| --- | --- |
| `published_code_leaves_the_interpreters_machine_and_names_its_exits` | a block run as code ends where the interpreter ends; each exit is named |
| `code_is_seen_whole_by_the_thread_that_reads_its_slot` | 200 blocks, each read and run by the dispatcher's thread the moment its slot shows it |
| `code_pages_are_never_writable_and_a_later_batch_leaves_them_alone` | every published address in an `r-x` page of `/proc/self/maps`, before and after 80 more blocks, its bytes unchanged |
| `a_block_the_lowering_refuses_publishes_the_sentinel_and_the_next_compiles` | a refusal is `FAILED`, counted, and the module goes on |
| `past_its_budget_the_module_goes_whole_and_compiling_starts_again` | at the budget nothing happens; past it the module goes, the flush is counted, a new thread counts from nothing |
| `past_its_budget_the_compiler_is_retired_without_waiting_for_its_queue` | a thread held 500 ms in one block with eight queued behind it: `over_budget` returns in under 250 ms, and the eight are refused, not compiled (seeded fault: the join put back, caught) |
| `a_dispatcher_that_checks_its_budget_each_run_starts_no_run_past_it` | thirty runs at a 4 KiB budget, none started past it, at least three flushes |
| `the_verifier_steps_beside_every_compiled_instruction_and_finds_nothing` | 40 programs, the reference stepped exactly once an instruction |
| `the_verifier_finds_a_miscompiled_instruction_at_that_instruction` | an ADDI lowered as a subtraction is found after step 2 |
| `the_verifier_finds_a_block_it_did_not_follow` | a block run without its calls leaves the reference behind, and it is found |
| `the_verifiers_last_check_finds_memory_the_steps_do_not_compare` | `check_all` compares what `check` does not |

**ThreadSanitizer**, built as `Mars_Native.md` §5.6.7 builds it (`RUSTC_BOOTSTRAP=1`, `-Zsanitizer=thread
-Zexternal-clangrt`, clang's runtime) [M]:

- *The crate's ten tests*: 17 reports, every one in the blind spots §5.8.5 named — an `Arc`'s last drop (the slot's,
  whose fence std's uninstrumented code makes and TSan does not model) and the page-size `Once` inside the `region`
  crate. None outside them.
- *The positive control*: the slot's store made Relaxed. The ten tests pass with it on this x86-64 desktop, where the
  store compiles to the same instruction either way; TSan reports it at once, the test's read of the
  code's first bytes against `cranelift-jit`'s copy of them on the compiler's thread, in
  `code_is_seen_whole_by_the_thread_that_reads_its_slot`.
- *MarsRT's 21 block tests at every tier*, the variant beside a running signal processor on: §5.8.10 of `Mars_Native.md`.

**The blind spots, unchanged by the move**: a slot reaches the thread inside an `Arc`, so TSan cannot see the hand-off's
own ordering; and compiled code is not instrumented, so no access it makes is checked. What is checked is every access
the crate's thread and the handlers compiled code calls make, and the test's read of the published bytes.

**MarsRT, the gate**: its block tests, its corpus and its games identical at every tier, and its speed unchanged within
its noise, before and after on the same machine state; `Mars_Native.md` §5.8.10 and `Sapphire_Native.md` §42.

## 6. What is not done

- ~~**No second user yet.** The interface is MarsRT's needs made generic; Sapphire's S36 is its first test as a shared
  surface, and a change it needs is a minor version here.~~ *Since 2026-10-26 Sapphire's EE lowering (S36,
  `Sapphire_Native.md` §44) is the second; it needed no change to the interface, which stays 1.0.0: its lowering, its
  options, its two exits ("done", "rollback") and its own context, the thread, the slots, the budget and the verifier's
  shape as MarsRT uses them. VU micro mode's lowering (S40, §45) is the third, again with no change.*
- **Words are 32 bits.** ~~A VU's 64-bit instruction pair goes to a lowering as two words; nothing wider was needed.~~
  *At S40 a whole microprogram goes as triples of words, each pair's address, upper word and lower word, since its pairs
  are not contiguous; the start names the address the unit was entered at.*
- **The `Arc` hand-off** stays invisible to TSan; replacing it with a slot index into storage the crate owns would make
  it visible, and was left for a stage that changes behaviour rather than one that moves it.
- **Code is not freed a block at a time**, as before (§3.4).

## 7. Version 1.1: tailored code beside the function (`native`, 2026-10-28)

*Added for E0 of the tailored recompiler (`EmuSen_Recompiler.md` §14), behind the crate's `native` feature, which is off
by default: without it the crate builds and behaves as 1.0.0, and MarsRT does not enable it.*

- **The extension.** `Lowering` gains two provided methods, `native(words, start, options) -> Option<Native>` and
  `stubs() -> Vec<Code>`, both empty by default, so that every 1.0 lowering compiles unchanged: a minor version. A
  `Native` is an `emusen-assembler` `Code` (hot and cold bytes, data, relocations, entry) and the offset of an 8-byte data
  word that receives the Cranelift function's address, the tier below that the code falls back to.
- **On the thread.** Each job is lowered to its Cranelift function as before and then, if the lowering has one, to its
  tailored code (timed apart, `Counters::native_nanos`). After the module's batch is finalized the tailored code is
  placed, its relocations applied and the fallback word filled with the function's finalized address; the slot then
  publishes the tailored entry in place of the function (`Counters::native`). The budget counts both.
- **The code memory** (`native::Memory`, the precursor of 2.0's arenas, §3.7 there): one reservation of 1 GiB made with
  no access, the hot arena from its bottom and the cold with each block's data from its middle, so that every rel32
  between them and the stubs reaches. The stubs go first, at the first batch. Each batch is written into pages no earlier
  batch used, made writable, written, and made readable and executable with one `mprotect` each; a published page is
  never writable again. The page size is read at run time. The reservation is unmapped with the thread, as the module is.
- **New dependencies**, named as the design rules require: `emusen-jit -> emusen-assembler` and `libc`, both optional,
  under `native` alone.
- **Evidence**: `tests/native.rs`, a lowering whose function answers 7 and whose tailored code answers through a stub or
  falls back to the function, 300 blocks over several batches; the crate's twelve 1.0 tests unchanged with and without
  the feature.

## 8. Version 1.2: tailored code without the function, compiled now, and spent across drops (2026-10-29)

*Added for the baseline's covered blocks (`EmuSen_Recompiler.md` §16). Every 1.2 change is additive: a 1.0 lowering and
MarsRT compile and behave as before, and the `native` parts stay behind that feature.*

- **The function only where it is needed.** The thread now asks the lowering for its tailored code first. A `Native`
  whose code has no fallback word (`fallback` none) and does not ask for the function (`function`, a new field, for a
  measurement that reads the function) is published with no Cranelift function: nothing is lowered to IR, nothing goes
  through Cranelift or regalloc2, and only the tailored bytes count against the budget. `Native::needs_function` is the
  rule, and a fallback word implies it, so no tailored code can reach a function that was never compiled. Where the
  lowering has no tailored code, the function is compiled as in 1.0.
- **A tailored block the code memory cannot place** is refused (`FAILED`), since no function stands below it, and
  counted (`Counters::unplaced`, with `failed`); `over_budget` then drops the module as if the budget were passed
  (counted in `Jit::full` as well as `flushes`), so
  that a full reservation is never a lasting refusal. Where a function exists it is published instead, as in 1.1. The
  tailored bytes are counted against the budget when placed, not when lowered.
- **`Jit::compile_now(words, start, options)`**, `native` only: the tailored code made on the caller's thread and placed
  in a code memory the `Jit` owns (a second reservation, the same W^X batches, one block each), its slot published before
  the call returns. It returns none, having done nothing, where the lowering has no tailored code or the code needs its
  function, and the caller enqueues the block as before. Its bytes count against the same budget (`Jit::held`), and a
  drop frees its memory with the module. It is the forerunner of `EmuSen_Recompiler.md` §3.9's compiling on the
  dispatcher's thread, which E2 decides; it is not 2.0's arena.
- **`Jit::spent()`** returns what compiling has cost since the `Jit` was made, across its drops: blocks published,
  nanoseconds and bytes, on the compiler's threads and on the caller's (`Spent`). The counters themselves still restart
  with each module; `spent` keeps what each dropped one had reached, so that a bench can count the compiler's busy share
  over a window that holds a drop. A retired thread's block in hand is not in it.
- **The perf map** names a tailored block at its entry with its hot bytes' length; 1.1 gave it the function's size.
- **Evidence** (measured 2026-10-29): `tests/native.rs` gains two tests, a mixed batch in which only the blocks that need
  one compile a function (counted by the lowering), and `compile_now`'s publication, refusals, budget and kept totals;
  the 1.1 test of the fallback word is unchanged. 15 tests pass with the feature and 12 without.
