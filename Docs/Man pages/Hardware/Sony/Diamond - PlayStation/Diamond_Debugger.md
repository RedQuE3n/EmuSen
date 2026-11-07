# Diamond_Debugger — the debugger, stage 12

*Written 2026-10-11, at the plan's stage 12 (`Diamond_Plan.md` §7, row 12), with the tester's go of the same day.* The
plan asks for the generic debugger over `DEBUG`, `DEBUG_REGISTERS` and `DEBUG_DISASSEMBLE`, the R3000A observed through
the `Observer` trait (§4.7, §9.1), and holds it to G7: breakpoints, stepping, watches, coverage and the call stack, each
a test, and every table armed giving the plain run's digests (the kit's C15). This page records how each part was
built, what was measured, and what was left and why. The build record's entry is `Diamond_Native.md` §19.

The marks are the plan's: **[M]** measured on 2026-10-11 with the tool named; **[R]** read from a source; **[A]**
argued; **[C]** a choice.

## 1. What a host is given

Nothing in the frontends or DianaSh is Diamond's: `CoreDebugTarget` and `CoreDebugBridge` drive any v1 core that claims
`DEBUG` (`EmuSen_CoreAPI.md` §6.14, §19), and Diamond fills the same tables Nephrite's and VenusRT's do.

- **Capabilities**: `DEBUG`, `DEBUG_STACK`, `DEBUG_REGISTERS` and `DEBUG_DISASSEMBLE`, added to `AXES` and
  `BATTERY_DIRTY` (`src/v1.rs`).
- **Processors**: processor 0 is `R3000`, 32 bits of address, its code space `R3000BUS`, its 45 registers
  (`blackmoon_r3000::DEBUG_REGISTERS`: the general registers by their conventional names, HI, LO, the PC, then
  coprocessor 0's SR, CAUSE, EPC, BadVaddr, TAR, DCIC, BPC, BDA, BPCM and BDAM). Processor 1 is `GTE`, with its 32 data
  and 32 control registers as `blackmoon_gte::Gte::register` reads them, psx-spx's names [R]. Both rows were declared at
  stage 1; this stage gave them values.
- **Spaces**: `R3000BUS`, id 0 as the plan's §4.3 reserved it, 4 GiB, flagged `read_only`, `view` and
  `reports_stores`; then the six memories of earlier stages, of which `RAM` and `SCRATCH` now say `reports_stores`.
- **The GTE runs no program.** `debug_pc` answers 0 for it, as the kit's C14 needs an answer for every processor
  named, and `debug_set_breakpoints` on processor 1 answers `NOT_SUPPORTED`, which the bridge already ignores. A
  breakpoint on a GTE command is the R3000A's breakpoint on the COP2 instruction [C].

## 2. The observed frame

### 2.1 One step, two ways

`machine::step_on` is the step the plain frame always ran (the shell entry's side-load, the putchar tap, the
processor's step, the clock's events), made generic over the observer and over the machine's parts, so that the
debugger can lend the observer the hooks beside the processor and the bus. `Machine::step` calls it with `Unobserved`;
its `before` is a constant zero and its other methods empty, so the plain frame compiles to the loop it was [A, and
§6.3 measures it]. The debugger's frame, `Machine::run_frame_debug`, calls it with `Watch`, an observer over the shared
`emusen_native::debug::Hooks`.

### 2.2 Where a stop falls

`blackmoon-r3000`'s step takes a pending interrupt first, then asks the observer, then runs the instruction
(`BlackMoon_R3000.md` §5.1). A stop therefore falls in front of an instruction with nothing of it run: no fetch, no
cycle, no event. Asking again stops again, and the state is unchanged (`a_breakpoint_halts_in_front_of_its_instruction_and_resumes`).
An interrupt is never stopped in front of: the step that takes it runs the exception entry, and the observer is asked
in front of the handler's first instruction, where `INTERRUPTS` stops.

`run::UNCHECKED` runs the next step unasked, and only the next step: if that step takes an interrupt instead, the
handler's first instruction is asked as usual. A frame a stop left open is continued by the next call, whatever its
flags; `field_done` is cleared only when a frame begins. A state loaded while a frame is open closes it
(`Debugger::close`), so the next frame is a whole one (`a_state_loaded_at_a_stop_starts_a_whole_frame`). A plain
`advance` called on an open frame finishes it, as it would any frame, and closes it.

The order of the checks in `Watch::before` is the stop first and the record after: a step stopped in front of is not
counted in coverage or the profile, and the resumed step, unasked, is counted once.

### 2.3 Stores

The processor reports each byte of a store after it has landed, and not a store that bus-errored or one made with the
cache isolated (`BlackMoon_R3000.md` §2). `Watch::wrote` reports it twice when `WRITES` is armed: in `R3000BUS` at the
address stored to, and in `RAM` or `SCRATCH` at the offset it lands at, so a watch on either names the same store. The
offset is `bus::reached`, which applies `decode`'s two rules for RAM and the scratchpad to the RAM window the bus maps
at that step; a test sweeps every segment, both RAM sizes and the windows' edges and holds it equal to `decode`
(`a_store_is_reported_where_decode_lands_it`). The window is read before each observed step, so a change of RAM_SIZE
is honoured from the next store [A].

**Not reported** [C, each a limit of the generic surface rather than of the console]: stores by DMA, by the GPU into
VRAM, by the SPU into its RAM and by the CD controller into its buffer, none of which the R3000A makes; and loads,
since v1 has no read watch (§5.1). *Since 2026-10-11* loads are reported, under the core ABI's 1.2 (§9).

### 2.4 The call stack

| Event | Reported as | When |
|---|---|---|
| JAL, JALR with rd ≠ 0, BLTZAL and BGEZAL taken | a call, from the branch to its target | at the branch |
| JR ra | a return | once its delay slot has run |
| An interrupt | an IRQ frame, from the interrupted instruction to the vector | at the exception's entry |
| Any other exception (SYSCALL, BREAK, address and bus errors, overflow, a reserved instruction, a coprocessor unusable) | a BRK frame | at the exception's entry |
| RFE | a return | at once |

Calls at the branch and returns after the slot are MarsRT's choice for the VR4300 (`interp.rs`, its
`return_after_slot`) [R], and the reason is the host's step over: `ArmStepToDepth(d)` at a JAL stops when the depth is
back at `d`. A return noted at the JR would bring the depth to `d` in front of the JR's delay slot, inside the routine;
noted after the slot, the stop falls in front of the instruction after the call's own slot, which is where a step over
should land. A call noted only after its slot would stop a step over at once, in front of the slot, at depth `d`. RFE
sits in the delay slot of the handler's `jr k0`, so its return is already after the jump's slot and is noted at once;
the machine tells the two apart by whether a branch is pending after the step that returned
(`run_frame_debug`).

**A call abandoned by an exception in its delay slot.** An interrupt taken in front of a JAL's delay slot, or an
exception the slot itself raises, sets EPC to the JAL (Cause's BD), so the JAL runs again after the handler's RFE and
reports its call a second time. Pushed at the JAL and never undone, the first call would stay on the stack for good, a
frame for every such interrupt. The observer therefore notes a return in front of the exception's frame when a call
is open whose slot the step runs, and a return waiting for its slot is dropped for the same reason. **Shown** by
`interrupts_push_and_rfe_pops_and_a_call_interrupted_in_its_slot_is_abandoned`: 240 frames of a loop of three calls with
the vertical blank's interrupt on, each interrupt stopped in front of its handler, the depth 1 whenever EPC is one of
the calls with BD set, no unmatched return, and the depth 0 at the loop's head afterwards. With the abandon removed the
test fails at the first such interrupt, the depth 2 [M].

**What the stack cannot follow** [A]: a JALR that links in a register other than ra returns through `jr` of that
register, which is a jump, so its frame stays open; a routine left by a jump rather than `jr ra` (a tail call, a
`longjmp`) likewise; an exception handler that never returns, such as one that resets. The hooks' cap of 512
(`MAX_DEPTH`) bounds the stack, and the returns past it still pop.

### 2.5 Coverage and the profile

Coverage is kept over 24 bits of address (`COVERAGE_BITS`), because the host's registry keeps one bit for each of
2²⁴ addresses (`CoverageRegistry`: `address & 0xFFFFFF`) and its buffer is at most 2 MiB (`CoreDebugBridge.Attach`). An
address's mirrors in KUSEG, KSEG0 and KSEG1 therefore share a bit, which is what a player wants of RAM; the BIOS's code
falls at C00000h and the first expansion region's at 000000h, the same bits as RAM's first 8 MiB, a collision no
program met [A]. The profile charges each instruction to the routine open when it ran, so a call's delay slot is
charged to the routine it calls and a return's delay slot to the routine returning
(`coverage_and_the_profile_record_what_ran`: seven instructions a pass outside and three calls of nine, to 1%).

## 3. Registers and disassembly

`debug_registers` answers the lists of §1 as the processor and the engine hold them
(`the_registers_are_the_processors_and_the_gtes`). A register a load is about to write shows its value before the
load lands; the load delay is architectural on the R3000A, and the pending value is not one of the 45 [C].

`debug_disassemble` lists R3000A code with `emusen-mips`'s disassembler and the R3000A's dialect, coprocessor 0 by
`blackmoon-r3000`'s table and coprocessor 2 by `blackmoon-gte`'s `COPROCESSOR`, so GTE commands are named (`rtps`) and
its registers too (`mtc2 t0, VXY0`) (`EmuSen_Mips.md` §3). Nothing was copied: the text is the shared crate's, split
at its first space into the record's mnemonic and operands. A listing starts at the word boundary below the address
asked. From `R3000BUS` it reads `Board::peek`, RAM, the scratchpad and the BIOS as `decode` finds them and every other
address, whose reads would change a device or fault, as FFh (`a_peek_reads_the_memories_and_nothing_else`). From a
memory space it reads the memory's bytes, and computes its targets as code there runs: RAM and the scratchpad in KSEG0,
the BIOS at BFC00000h, any other space at its offsets. The static references are those the bytes alone give: a JAL's
or BxxZAL's target as a call, and a load's or store's address when its base is `zero` as a read or a write
(`code_lists_from_the_bus_and_from_ram_with_the_gtes_names`).

## 4. Tests

Each claim of §1–§3 is a test; [M] 2026-10-11.

- **The crate** (`src/debugger.rs`, ten; `src/bus.rs`, two): the armed frame with nothing to hit, and a frame stopped
  before every instruction (over 100,000 stops in six frames), give the plain frames' state, picture and sound with the
  vertical blank interrupting calls in flight; a breakpoint halts in front of its instruction, holds, and resumes;
  `EACH` stops in order through a JAL, its slot and the routine; the call stack, step over and step out; interrupts and
  RFE, and the abandoned call; watches in the bus, RAM and the scratchpad and a data breakpoint; coverage and the
  profile; the registers; the listing; a state loaded at a stop. The two in `bus.rs`: `reached` against `decode`, and
  `peek`.
- **The tables one by one** (`tests/programs.rs`, the stage's block at its end): each of nine tables armed alone (calls;
  writes watched over all of RAM and the scratchpad; data breakpoints over all of RAM; the profile; coverage;
  interrupts; breakpoints over KSEG0's RAM; `EACH`; and the kit's C15 set, all with nothing to hit), every stop
  continued and every log drained as a host does, gives the plain run's digests of picture, sound and state, on the
  project's programs, ps1-tests' release on the open kernel (opt-in, `EMUSEN_DIAMOND_PS1TESTS`) and one of the
  tester's discs on the open kernel with no firmware (opt-in, `EMUSEN_DIAMOND_DISC`). §6.1 lists them.
- **WiseMan**, `DiamondDebugTests`, nine, through discovery, `CoreEngine`, `CoreDebugTarget`, the bridge and DianaSh's
  own commands, on a hand-assembled PS-EXE on the open kernel that sets VXY0 and calls a counting routine from a loop:
  the processors, spaces and registers (`cpus`, `regs gte`); a breakpoint halts and resumes; single steps through the
  delay slots and `step 3`; the call stack (`bt`) and a step over; write watches in RAM, the scratchpad and the bus,
  with the storing instruction; coverage; disassembly by processor and by space, the JAL a call reference; every
  registry armed with nothing to hit giving the plain run's state frame for frame; and the target a frontend's debugger
  window is given (`CoreFactory.Bundle`), its summary, `regs`, `disasm cpu` and `bp add`. `DiamondTests`' capability
  assertion gained the four bits.

## 5. What was not built, and why

### 5.1 Read watchpoints

The stage was asked for "memory read/write watchpoints on the bus". v1 has none for reads: `debug_set_ranges`
takes watch ranges and data-breakpoint ranges of stores only (`EmuSen_NativeCores.md` §3.14), the bridge skips a watch
of `WatchKind.Read` and a data breakpoint `OnRead` (`CoreDebugBridge.PushTables`), and `Observer` has no method for a
load. Diamond could keep loads to itself, but nothing above the ABI would ask for them. The forms it could take
(the project's rule for a feature that would widen a stable surface):

- **An extension of the existing surface** [recommended]: range kinds 2 and 3 for read watches and read breakpoints in
  `debug_set_ranges` (a minor, additive under `EmuSen_CoreAPI.md` §4.2), a `read_log` drained beside the writes, an
  `Observer::read` with an empty default so that every other processor crate compiles as it does, and the bridge
  pushing the two kinds it now skips. Every core that does not claim the new bit is unchanged.
- **A module of its own**: none fits; a read watch is one of the debugger's tables, not a second job.

It is proposed here for the tester's decision and not built. Its cost on Diamond would be one call per load in the
observed frame only. *Decided 2026-10-11: built as proposed (§9), the minor numbered 1.2.*

### 5.2 The rest

- **Console views** (VRAM, the SPU's voices, the GPU's state): extensions outside v1 by the plan's §4.7 and
  `EmuSen_CoreAPI.md` §6.14; none built. VRAM is a space already, readable in the memory view.
- **Breakpoints on the GTE as a processor**: refused (§1).
- **The call stack in plain frames**: kept only while observed, as on MarsRT; `CoreEngine` runs a plain `advance`
  unless a table is armed (`EmuSen_CoreAPI.md` §30).

## 6. Measured

### 6.1 Armed against plain, table by table

[M, `tests/programs.rs`' three tests, release build, 2026-10-11.] Each row: every one of the nine tables of §4 gave the
plain run's three digests (FNV-1a of each frame's picture, of the sound's samples, of the final state). Shown are the
state's digest, and the stops of three tables, to show each was exercised.

| Program | Frames | State digest, plain = every table | Stops: data breakpoints over RAM | Stops: interrupts | Stops: `EACH` |
|---|---|---|---|---|---|
| `exceptions.s` | 10 | DB8EC06D81D2D9FA | 16 | 12 | 1,129,228 |
| `gte-fuzz.s` | 10 | 4D06B19F401F0F65 | 117,522 | 1 | 5,287,246 |
| `spu-voices.s` | 20 | 67B2DB8CE3470868 | 2,849 | 0 | 8,067,741 |
| `raster-order.s` | 10 | 25C4979DF3AC9097 | 18,433 | 0 | 5,471,340 |
| `mdec-timing.s` | 10 | 0EB1095315A4ADB6 | 3,179 | 0 | 5,542,346 |
| `cd-read.s`, a synthetic disc | 60 | 315709F56AF4DAC1 | 1,132 | 0 | 17,741,493 |
| ps1-tests `timers` | 30 | 073A3C5E19629C84 | 39,176 | 31 | 6,423,206 |
| ps1-tests `gpu/triangle` | 30 | C3CEB31A0B2BC1E5 | 29,595 | 31 | 12,529,026 |
| ps1-tests `gte/test-all` | 30 | 0049BF000246F724 | 134,068 | 8 | 2,651,534 |
| ps1-tests `spu/test` | 30 | 39518D795FE4D5FA | 26,696 | 31 | 16,524,365 |
| ps1-tests `mdec/movie-15bit` | 30 | 22479A4FC5AF3D59 | 27,837 | 31 | 12,135,806 |
| ps1-tests `dma/otc-test` | 30 | 2BD338CFC6438750 | 95,807 | 32 | 11,907,523 |
| A copied disc (Ape Escape, one track), open kernel | 120 | 24DC571637C6827F | 17,521 | 0 | 67,756,299 |

The interrupts table's stops on the project's programs are their exceptions (`exceptions.s` raises twelve of the BRK
kind) or a full log's (`gte-fuzz.s`'s one); the programs that enable no interrupt have none. 117 armed runs, 117 equal.

### 6.2 The kit

[M, `emusen-core-conform --frames 300`, single-image form, the release library.] All fifteen cases pass, C15 among them,
on ps1-tests' `gpu/triangle`, `spu/test`, `gte/test-all`, `cpu/cop`, `mdec/movie-15bit` and `dma/otc-test`, and on the
copied disc of §6.1 on the open kernel with no files; and at 120 frames on `timers`. C14 now counts 7 spaces and 2
processors. No run on the player's BIOS was made: its code would be among what the armed tables observe.

### 6.3 The cost

[M, `examples/run`, 600 frames, pinned to one processor, the builds before and after the stage run alternately, five
runs each, ms a frame, fastest and median.]

| Program | Before | After, unarmed | After, armed (`--armed`, the C15 set) |
|---|---|---|---|
| `timers` | 6.872 / 7.280 | 6.981 / 7.059 | 8.268 / 8.359 |
| `gpu/triangle` | 7.193 / 7.217 | 7.134 / 7.276 | 9.144 / 10.192 |
| `gpu/bandwidth` | 8.431 / 8.597 | 8.357 / 8.415 | 10.711 / 10.874 |
| `spu/test` | 7.553 / 7.641 | 7.297 / 7.454 | 8.915 / 8.950 |
| Crash Bandicoot by `--boot`, open kernel | 7.496 / 7.525 | 7.451 / 7.484 | 9.069 / 9.131 |

(The armed column is from a second measurement of three runs each, beside its own before and after, which agreed with
these within 3.2%.) **Unarmed, no change**: the differences run both ways, −3.4% to +1.6% on the fastest runs, inside
the runs' own spread. **Armed, 18% to 27% more** than that measurement's unarmed build, most on the GPU programs, whose calls and stores are many; the frame's
cost is the hooks' per-instruction check, coverage and the call log, none of which the plain frame runs.

## 7. Framework notes

- **A 4 GiB view.** `R3000BUS` is the first view the size of a 32-bit space. `CoreDebugTarget.ViewSpaces` copies a
  space without side effects whole on first touch for a system pack's views; were a pack for `psx` to gain views, a
  view of `R3000BUS` would ask for a 2 GiB copy (the size clamped to `int.MaxValue`). No `psx` pack has views today, so
  nothing reaches it; reading a `view` space live there would close it. Recorded in `EmuSen_CoreAPI.md` §29.4, not
  changed here.
- **Coverage wider than 24 bits** would be merged from a buffer the core did not fill (`CoreDebugBridge.Drain` merges
  `min(length, buffer)` bytes when the core answers a length larger than the buffer it then declines to copy into).
  Diamond keeps 24 bits, so it does not arise; recorded for a core that might.

## 8. Reading log

Read for this stage: `Diamond_Plan.md` §4.3, §4.7, §7, §8 and §9.1; `Diamond_Native.md`; `BlackMoon_R3000.md` §2 and
§5; `BlackMoon_GTE.md` §2 and §9; `EmuSen_Mips.md`; `EmuSen_CoreAPI.md` §6.14, §24, §29 and §30;
`Nephrite_Native.md` §41–§43; `VenusRT_Native.md` §36; and the project's own code: `emusen-native`'s `debug.rs` and
`core/`, Nephrite's `debugger.rs`, `genesis.rs` and `v1.rs`, MarsRT's `interp.rs` (its return after the slot), the kit's
`lib.rs`, and the host's `CoreDebugBridge.cs`, `CoreDebugTarget.cs`, `CoverageRegistry.cs`, `WatchRegistry.cs` and
DianaSh's `DisasmCommand.cs`, with WiseMan's `NephriteDebugTests.cs` and `CoreDebugViewTests.cs` as the pattern. **No
emulator's source and no emulator's debugger was read; the player's BIOS was not opened, traced, disassembled or run
under a table**; the open kernel was run observed in tests and listed by its address space, as the project's own code.
No dispute was taken.

## 9. Read watches and read data breakpoints, built 2026-10-11

*Decided 2026-10-11: §5.1's extension, built as proposed. The surface is the core ABI's minor 1.2
(`EmuSen_CoreAPI.md` §6.14.1; its record `EmuSen_CoreAPI_Extensions.md` §13); the processor's half is
`BlackMoon_R3000.md` §12.* Marks as in §1; [M] measured on 2026-10-11.

### 9.1 What a host is given

- **`DEBUG_READS`** beside the four debug bits of §1, and the export `emusen_core_debug_reads` (`src/v1.rs`).
- **`reports_loads`** on `R3000BUS`, `RAM` and `SCRATCH`, the three spaces that already say `reports_stores`.
- Range kinds 2 (read watches) and 3 (read data breakpoints) in the shared hooks, gated by `READS`, and the read log.

### 9.2 Loads reported

`blackmoon-r3000` reports each byte of a load once the bus has answered it, through `Observer::read`, whose default is
empty (`BlackMoon_R3000.md` §12). `Watch::read` reports it as `Watch::wrote` reports a store (§2.3): in `R3000BUS` at
the address read and in `RAM` or `SCRATCH` at the offset it lands at, by the same `bus::reached`. The loads are LB, LBU,
LH, LHU, LW, LWL and LWR, at the bytes of the word they reach, and LWC2 at its word (LWC0, LWC1 and LWC3, which Diamond
reads and drops, likewise). **Not reported** [C]: a load that bus-errored, a load with the cache isolated, fetches, and
every read the R3000A does not make, DMA's among them: a DMA channel reading RAM for the GPU or the SPU is the DMA
controller's access, as its stores are (§2.3), and a watch on it would need a second observer in the bus, which nothing
above the ABI asks for.

### 9.3 A stop in front of the load

A read data breakpoint stops in front of the loading instruction with nothing of it run, as a breakpoint does (§2.2),
and not after it as a store's must. To know the load before the step, `run_frame_debug` asks, only while `READS` is set
and a kind 3 range exists, and not for a step resumed unasked:

1. the shell's side-load first (`machine::side_load`, split out of `step_on` unchanged), so that the instruction asked
   about is the one the step will run;
2. **`Board::peek_code`**: the word the fetch would give, with nothing changed: the I-cache's word where the fetch
   would hit, so that code the cache holds stale is read as the processor will run it, and RAM's or the BIOS's
   otherwise; none for code anywhere else, whose fetch could change a device, so no stop is foreseen there;
3. **`R3000::load_ahead`** on that word: the first byte and the count the load would read, or none where the
   step's own rules end it first (`BlackMoon_R3000.md` §12);
4. each byte as `R3000BUS`'s space reads it (`Board::peek`).

`Watch::before`, asked only once no interrupt is taken (§2.2), then logs each byte a kind 3 range covers, in the bus and
in the memory it lands in, with that value, and adds `STOP_READ` to the reasons. Resumed with `UNCHECKED`, the step
runs unasked and the load is made; a kind 2 range over the same byte then logs it again, read. **The limit** [A]: a
load to an address no device answers is foreseen and then takes DBE without reading, the one case only the read can
tell; and the value logged ahead is the memory's before the load, which a DMA landing in the same step could change.

### 9.4 Tests

Each claim a test [M]:

- **The processor** (`blackmoon-r3000`, `every_load_reports_the_bytes_it_read_and_load_ahead_names_them_first`): LWL's
  and LWR's bytes, and `load_ahead` against the step on eight loads in five modes at four offsets, 128 equal and the
  22 aligned loads to no device foreseen (`BlackMoon_R3000.md` §12). The suite's 55,000 cases pass.
- **The crate** (`src/debugger.rs`, two new, twelve in all): `read_watches_see_loads_in_the_bus_ram_and_the_scratchpad`,
  a loop of LW, LBU, LWL of two bytes, LWR of two, LH and LWC2: each byte logged with its value and its instruction,
  LWL at 80020005h reading 20004h–20005h and LWR at 80020006h reading 20006h–20007h, LWC2 at its word, the bus's watch
  naming the same loads as RAM's, each pass's LW reading the count the last stored; and
  `a_read_data_breakpoint_halts_in_front_of_the_load_and_resumes`: a stop in front of the LH, the state and the clock
  unchanged when asked again, two kind 3 entries with the scratchpad's byte, the load run once resumed; ranges in the
  bus and in RAM stopping at the LWR or the LWL whose bytes they cover, and KSEG1's mirror of an address the loop reads
  through KSEG0 never stopping. The C15 test of §4 now arms the read kinds too.
- **The tables one by one** (`tests/programs.rs`): two more, reads watched over all of RAM and the scratchpad, and read
  data breakpoints over all of RAM, every stop continued; and the kit's C15 set with both read kinds. All eleven tables
  gave the plain run's three digests on the six project programs, the six ps1-tests programs on the open kernel and
  Ape Escape on the open kernel: 143 armed runs, 143 equal. The read data breakpoints over RAM stopped 2,431,336 times
  in 30 frames of `timers`, 1,752,797 times in 120 of Ape Escape, 0 times in `mdec-timing.s`, which reads no RAM.
- **WiseMan** (`DiamondDebugTests`, two new, eleven in all), through discovery, the bridge and DianaSh's commands on a
  hand-assembled loop: `watch add RAM 20000 4 read` and its fellows record each load with `PC=` the loading
  instruction, LWL's and LWR's bus addresses, and a write watch beside them only the store; `bp read SCRATCH 10` halts
  in front of the LBU, the frame not counted, and resumed runs one pass; `bp read R3000BUS 80020006` halts at the LWR
  and not the LWL. The armed-with-nothing-to-hit test now holds a both watch and a read data breakpoint. The four debug
  bits of `DiamondTests` gained the fifth.

### 9.5 Measured

**The kit** [M, `emusen-core-conform --frames 300`]: all fifteen cases pass on `gpu/triangle`, `spu/test`,
`gte/test-all`, `cpu/cop`, `mdec/movie-15bit` and `dma/otc-test`, C15 with the read kinds armed beside the store
kinds and equal to the plain run, and kinds 2 and 3 accepted, kind 4 refused.

**The plain frame** [M, `examples/run`, 600 frames, pinned to one processor, the build before and the build after run
alternately, five runs each, ms a frame, fastest and median]. The plain step's machine code is unchanged:
`exec::step::<Board, Unobserved>` disassembles to the same 1,352 instructions in both builds, differing only in the
addresses of its jump tables, and is 17ECh bytes in both; `Observer::read`'s loop compiles to nothing. The default
builds nevertheless differ, by where the linker places the hot functions; the same two sources linked with every
function aligned to 64 bytes (`-C llvm-args=-align-all-functions=6`) differ by no more than the runs' spread:

| Program | Before | After | Before, aligned | After, aligned |
|---|---|---|---|---|
| `timers` | 6.875 / 6.927 | 7.431 / 7.494 | 6.981 / 7.018 | 6.989 / 7.035 |
| `gpu/triangle` | 7.660 / 7.760 | 7.795 / 7.893 | 7.460 / 7.507 | 7.524 / 7.563 |
| `gpu/bandwidth` | 8.607 / 8.728 | 9.108 / 9.225 | 8.529 / 8.571 | 8.568 / 8.583 |
| `spu/test` | 7.143 / 7.191 | 7.910 / 7.925 | 7.412 / 7.491 | 7.451 / 7.617 |
| Crash Bandicoot by `--boot`, open kernel | 4.136 / 4.172 | 4.431 / 4.544 | 4.167 / 4.203 | 4.183 / 4.228 |

**Read as** [A]: the change costs the plain frame nothing, aligned 0.1% to 0.9% on the fastest runs; the default
builds' 1.8% to 10.7% is the link's placement, which any change to the observed path moves. That the before build is
itself 3.6% faster than its aligned link on `spu/test` and 2.7% slower on `gpu/triangle` shows the same placement
working both ways. Which link the shipped build should use is a question of its own, for the stage that optimises the
step (stage 13), and is not decided here. The first measurement of `spu/test` was confirmed by six more alternate
runs each (4,267–4,303 ms against 4,705–4,738 ms), and by a build with the load report removed alone, which ran as
the before build: the difference follows the placement, not the code.

**Armed** [M, three runs each, the after build]: with the C15 set of §6.3 (`--armed`) 5.96 to 11.59 ms; with a read
watch and a read data breakpoint beside it over RAM's last four bytes (`--armed-reads`), 8.69 to 16.20, 33% to 46%
more, the cost of the read-ahead check on every instruction and of a call per load. Only an observed frame pays it.

### 9.6 Reading log

Read for this step: §2–§5 of this page, `BlackMoon_R3000.md` §2, §5 and §6, `EmuSen_CoreAPI.md` §4–§6, §12, §17 and
§24, `EmuSen_CoreAPI_Extensions.md` §0 and §1; the project's code: `emusen-native`'s `debug.rs`, `core/` and `abi.rs`,
`emusen-core-abi-check`, the kit's `lib.rs`, `blackmoon-r3000`'s `exec.rs`, Diamond's `debugger.rs`, `bus.rs`,
`machine.rs` and `icache.rs`, the host's `CoreDebugBridge.cs`, `CoreInterface.cs`, `CoreDescriptors.cs`,
`BreakpointRegistry.cs` and `WatchRegistry.cs`, and DianaSh's `WatchCommand.cs`, `BreakCommand.cs` and their pages. No
emulator's source and no emulator's debugger was read; the player's BIOS was not opened, traced or watched.
