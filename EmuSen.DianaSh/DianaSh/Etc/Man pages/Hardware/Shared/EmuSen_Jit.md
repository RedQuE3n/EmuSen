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
in `Sapphire_Native.md` §44), for the EE; Diamond may adopt it when its own recompiler is wanted (`Diamond_Speed.md`
§7.2). The CI job that tests every crate under `EmuSen/Cores/Shared/` tests this one without a
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
run. Code exactly at the budget is kept. MarsRT checks it at the start of every run with a budget of 256 MiB, which no
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
instructions, with its interpreter and its lowering [M, `cargo test --release`, 10 of 10]:

| Test | What it holds |
| --- | --- |
| `published_code_leaves_the_interpreters_machine_and_names_its_exits` | a block run as code ends where the interpreter ends; each exit is named |
| `code_is_seen_whole_by_the_thread_that_reads_its_slot` | 200 blocks, each read and run by the dispatcher's thread the moment its slot shows it |
| `code_pages_are_never_writable_and_a_later_batch_leaves_them_alone` | every published address in an `r-x` page of `/proc/self/maps`, before and after 80 more blocks, its bytes unchanged |
| `a_block_the_lowering_refuses_publishes_the_sentinel_and_the_next_compiles` | a refusal is `FAILED`, counted, and the module goes on |
| `past_its_budget_the_module_goes_whole_and_compiling_starts_again` | at the budget nothing happens; past it the module goes, the flush is counted, a new thread counts from nothing |
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
  shape as MarsRT uses them.*
- **Words are 32 bits.** A VU's 64-bit instruction pair goes to a lowering as two words; nothing wider was needed.
- **The `Arc` hand-off** stays invisible to TSan; replacing it with a slot index into storage the crate owns would make
  it visible, and was left for a stage that changes behaviour rather than one that moves it.
- **Code is not freed a block at a time**, as before (§3.4).
