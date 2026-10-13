# Diamond_Enhancements — the enhancement layer, stages E1 and E3

*Written 2026-10-13, at stage E1 of `Diamond_Plan.md` §7, with the tester's go of the same day for precise geometry
and a shared GPU crate.* E1 builds the frame every enhancement of the plan's §6.3 will stand in, and the proof that the
frame changes nothing while it is off: the observer on the R3000A and the GTE with the GPU's receipt of a vertex word,
the attachment of a layer to the machine, the optional state block, the settings' category, the accurate goldens and
the exactness harness. No enhancement is built and none is offered: "Precise geometry" (§11 Q14 of the plan) arrives
with E4, the Vulkan renderer with E3. The build record's entry is `Diamond_Native.md` §24.

*Stage E3, the same day* (§14 on): the Vulkan renderer that draws the picture from the GPU's command stream beside the
software rasteriser, at 1×, equal to it. §14 is its form, decided and committed before any of its code.

The marks are the plan's: **[M]** measured on 2026-10-13 on the desktop (Ryzen 7 7700X, Fedora 44, Rust nightly
2026-10-01), with the tool named; **[R]** read from a source, named; **[A]** argued, with no measurement behind it;
**[C]** a choice.

---

## 1. The stage and its gate

The plan's row: "The enhancement layer's frame: the observer trait on the R3000A and the GTE, the optional state
block, the settings' category, the exactness harness. Gate: every accurate golden reproduced bit for bit with every
enhancement compiled in and off; states made with and without the layer load both ways; C3 and C9 pass." The tester
added for this step: the goldens recorded first, since stage 14's never were (`Diamond_Native.md` §23.5); C7 and C8
beside C3 and C9 on the eight bench discs at 1,800 frames; and the frame cost on aligned links against the base,
unobserved and observed. §8 gives the gate item by item.

The plan's rules for the layer (§6.2) are kept as these [C]:

- **One job each.** The R3000A and the GTE report; a layer interprets; the accurate GPU never reads a layer, and no
  layer is read by anything in the machine.
- **Off is the accurate machine.** With no layer attached the frame is the one it was, to the byte of every golden
  (§7) and within the noise of the bench (§9).
- **A layer only watches.** Its events are copies; it holds no reference into the machine; with one attached every
  golden is reproduced too (§7).

## 2. The design

### 2.1 Who tells what

```
  R3000A (blackmoon-r3000) ── Bus::moved(Move) ──┐
                                                  │   LayeredBus: the board as the processor's bus,
  GTE (blackmoon-gte) ── Observer::projected ─────┼── used only while a layer is attached
                                                  │
  Board (Diamond's bus) ── Layer::received ───────┘──▶ the attached layer (Board::layer)
       GP0 port at 1F801810h, DMA channel 2
```

- **The R3000A tells its bus what it moves** (`BlackMoon_R3000.md` §14): `Bus::MOVES`, an associated constant false
  by default, and `Bus::moved(Move)`, an empty default; every report is made under `if B::MOVES`.
- **The GTE tells its observer each projection** (`BlackMoon_GTE.md` §9): `blackmoon_gte::Observer::projected`, given to
  `command_observed`; the plain `command` passes `Unobserved`.
- **The board tells the layer each word GP0 takes** (`Diamond_GPU.md` §13), from the processor's store or from DMA
  channel 2 with the word's address in RAM, just before the GPU takes it.

A layer is `diamond::layer::Layer`: the GTE's `Observer` as its supertrait, `moved`, `received`, and its state block's
four methods (§4), each with an empty default. `Tally` is the test observer (§7.1).

### 2.2 Why the processor reports to its bus

`BlackMoon-HW/README.md` had said the R3000A's half would be "a second generic parameter" beside the debugger's
`Observer`. It is not, by this reasoning [A]. The layer must see the processor's moves, the GTE's projections and the
GPU's receipts in one order: a value stored by SWC2 must be known as a projected vertex, which only the projection
just before it can say, and a DMA word must be known by the store that put it in RAM. The GTE and the GPU are behind
the processor's bus, inside Diamond's board. A second parameter beside the bus would have to be the same layer the
board holds, borrowed twice. Told to the bus, the moves reach the board, and the board holds the layer; nothing is
borrowed twice and nothing crosses a crate's surface but the two additions with defaults.

### 2.3 Attaching a layer, and the frame

`Machine::attach(Box<dyn Layer>)` puts a layer in `Board::layer` and `detach` takes it out; E4's setting will do the
same between frames. `Board::layer` is `Attached`, which is no part of the machine's state: a copy of the board, as the
levers' checks make (`Diamond_Speed.md` §4.4, §13.3), carries none, so a check's copy steps unobserved and is compared
by the state as before.

`Machine::advance` chooses once a frame between two forms of one generic frame, `advance_in::<T: Through>`. `Through`
names the bus the processor steps through: `Plain`, the board itself, and the frame is the one it was; or `Layered`,
in which the processor steps through `LayeredBus`, the board wrapped as a bus whose every method is the board's, whose
`MOVES` is true and whose `moved` and `cop2_command` tell the layer. Each form is its own function, out of line, and
the plain one compiles to the function the base's `advance` was (§9.2). The single step and the levers' checked run
are generic the same way; the debugger's frame chooses at each step. The GP0 receipts are the board's own and told
whichever bus the processor steps through, since DMA runs from the board's events and from its accesses; with no
layer the receipt is one test of `Board::layer` a word.

### 2.4 What stays in force while a layer is attached

The decoded I-cache, runs of steps and the idle skipper are the same in both forms of the frame; nothing about them
asks whether a layer is attached [M, the source; the bench's run and skip counts are the same with `--observed`, 2,687
skips of 1,011,120 turns over Crash Bandicoot's first 600 frames both ways]. A skip passes a loop's turns without
running them, so the moves of the turns it skips are not told. That is harmless to E4's layer [A]: a turn the skipper
may pass reads only constants and the induction and stores only them (`Diamond_Speed.md` §4.2), its first turns ran
observed, and a word it stores again is the same word, or the induction, which is no projected vertex; a layer that
checks each integer it holds against the value it later meets (§3.4) cannot be misled by a skipped store. Should E4
find a case where it is, the skipper can be turned off for the layered form alone, which the frame's two forms make a
one-line change; it is not made, there being no such case.

## 3. The events, against E4's need

### 3.1 The GTE's projection

| Field | What it is | What E4 needs it for |
|---|---|---|
| `vertex` | V0, V1 or V2 | which of RTPT's three |
| `sums` | MAC1–MAC3's TR·1000h + RT·V, checked as the accumulators check them, before sf's shift | the vertex in the camera's space with 12 fraction bits; the third its depth before SZ3's shift and saturation |
| `quotient` | H/SZ3 as the division makes it, 1.16 | the projection's scale as the console used it; the depth a perspective-correct texture interpolates |
| `x`, `y` | MAC0's sums for SX and SY before SAR 16 and saturation | the screen position with 16 fraction bits as the console computed it |
| `z` | SZ3 as pushed | the integer the FIFO kept |
| `word` | the SXY2 word pushed | the integer the GPU will be given if the vertex reaches it unchanged |

Only RTPS and RTPT, and command 00h which runs as RTPS (`BlackMoon_GTE.md` D-16), make a screen coordinate or a vertex's
depth; AVSZ3 and AVSZ4 average SZ0–SZ3 into OTZ, an index into the program's ordering table, which the program uses
and the GPU never sees, and NCLIP only reads the FIFO [R, `BlackMoon_GTE.md` §5]. Both the sums and the screen sums are
given because the console's SX is made from IR1, saturated to 16 bits, and the reciprocal table's quotient; a layer that
wants the vertex where the console would have put it with more bits uses `x` and `y`, and one that wants it where the
geometry says, past the saturation, uses `sums` [A].

### 3.2 The processor's moves

`BlackMoon_R3000.md` §14 lists them: `Load` and `Store` of every width (with LWL, LWR, SWL and SWR), `FromCop2`
(MFC2), `ToCop2` (MTC2), `LoadCop2` (LWC2), `StoreCop2` (SWC2), and `Copy`, a register given another's value unchanged
by one of the move idioms. Why each:

- **MFC2, SWC2**: the two ways a projected word leaves the GTE, into a register or straight to memory.
- **Loads and stores**: how the word is carried through RAM and the scratchpad into a primitive, and how a primitive is
  sent to GP0 by the processor (a store to 1F801810h is a `Store` whose address is the port).
- **MTC2, LWC2**: how a word returns to the FIFO (a program that keeps vertices and re-pushes them through SXYP), and how
  the layer knows the FIFO was written by the program rather than by a projection.
- **Copies**: a vertex held in a register and moved to another before its store would otherwise lose its precision.
  Other computations are not told; a register changed by one no longer holds the integer the layer recorded for it.

Not told, and why: CFC2 and CTC2 (the control registers carry matrices, vectors and offsets, never a projected
vertex); an access that met no device or that the isolated cache took (no memory was read or written); any move into
R0. A load is told when it is made though its register is written one instruction later, which the layer's integer
check makes safe (§3.4).

### 3.3 The GPU's receipts

`Layer::received(word, from)`, `from` either `Receipt::Cpu` or `Receipt::Dma { address }`, the physical address in RAM
as channel 2 reads it, for every word GP0 takes in either way, told before the GPU takes it, so that a layer that later
watches `Gpu::execute` (E3) holds a command's side data when the command runs. GP1's words and GPUREAD's are not
receipts. A command's words are in the order received, which is the order the GPU executes them [R, `Diamond_GPU.md`
§2], so a layer can keep a queue of side data beside the FIFO without the accurate FIFO carrying any.

### 3.4 What E4's layer will do with them [A, a design for E4, not built]

A shadow of the FIFO's three SXY entries and SZ, of the 32 general registers, and of RAM and the scratchpad by word:
each entry the integer word it stands for and its precise values. A projection pushes the FIFO's shadow; MFC2, SWC2,
MTC2 and LWC2 move entries between the FIFO, a register and a word of memory; a load and a store move them between a
register and memory; a copy between two registers. A receipt from DMA looks up the word's address, from the processor
the register its store named; an entry is used only if its integer is the word received, otherwise the word is drawn
as the integer it is. That check makes stale entries harmless: a register or word changed by anything the layer is
not told of (a computation, a skipped turn, another DMA channel writing RAM, a load from the isolated cache) no longer
holds the integer, and falls back to the console's vertex. This is the plan's "looks the word up by address and
integer" (§6.2), with the processor's own register as the address of a store to GP0.

## 4. The state block

**The form.** After `Picture`, the last of version 10's blocks, zero or more optional blocks, each a tag of four bytes,
a length (`u32`) and that many bytes. An attached layer that has a tag writes one block; with no layer, nothing, so the
state is version 10's to the byte (5,267,532 bytes for a machine with one DualShock and no card) [M,
`the_version_10_layout_is_pinned` unchanged]. The layout listing names the block's fields under its tag (the test
observer's: `TALY.Tag`, `TALY.Length`, `TALY.Counts`). Loading reads the accurate blocks into copies, then the optional
ones: a block runs past the end, or a tail too short for a tag and a length, and the load is refused as truncated,
changing nothing; a tag no one knows is skipped by its length; the attached layer's tag is handed its bytes once the
accurate part has been taken, and a layer whose tag is absent is cleared. A layer that cannot read its own block
starts empty rather than refusing the state, since no enhancement block is needed to reproduce the machine.

**No new version** [C]. A reader of version 10 from before the blocks reads to `Picture` and stops, and has never
checked for bytes after it; a reader with the blocks finds none in every version-10 state made before them. So a block
says nothing to an old reader that it must understand, and nothing to a new reader that the version must announce.
Versions 8, 9 and 10 load as they did (`a_state_round_trips_mid_run_and_a_refused_one_changes_nothing`).

**Both ways, measured.** `a_layers_block_is_skipped_without_it_and_a_state_without_one_clears_it` holds: the bytes
before the block are the state without it; a machine without the layer loads the state with it and continues as the
machine that saved it, to the state; a layer given a state without its block starts empty; a state with the block gives
the layer its counts back; an unknown block before it is skipped; a block cut short is refused and changes nothing.
Across builds [M]: a state of Crash Bandicoot at frame 600 saved by this branch with the test observer attached
(5,267,612 bytes, 80 more) was loaded by WiseMan 177f5041's `run` example and run 300 frames, and its picture and RAM
were those of the same build loading the state saved without the layer, and of this branch loading it with the block.

**Where the framing lives** [C]. `emusen-native` owns the state's field encoding (`StateWriter`, `StateReader`: little
endian, arrays without lengths, a presence byte for a class) and no framing of optional blocks; no other shared crate
frames a state; and no other core has an optional block. The tagged tail is part of DMND's layout, which is Diamond's
(`state.rs`, `write_block` and `read_blocks`), built from the reader's `position` and the writer's counter. A second
core wanting optional blocks would be the reason to move the framing into `emusen-native` as a writer's and a reader's
method; one user is not.

## 5. The settings' category

`v1.rs` declares an enhancement as an `Enhancement`: its key, label and help, its cost in a player's words, and its
values, `Values::Switch` (the hardware's value `false`) or `Values::Choice(&[...])` (the hardware's value first).
`Enhancement::setting` makes the schema's entry: `kind` the switch or the choice, `default` the hardware's value by
construction, `effect: enhancement` with that `hardware` and the `cost`, `category: "Enhancements"`, `scope: run` and
no restart, so that it is changed between frames as E4's attach and detach need. `ENHANCEMENTS` is empty: no setting
is shown that does nothing, and `an_enhancement_is_declared_with_the_hardwares_value_its_default_and_none_is_offered_yet`
holds that the schema has none, that both forms pass `emusen-native`'s `Setting::check` (the rules the kit's C3
applies) with the hardware's value as the default and the group and scope above, and that one whose cost is blank is
refused by the same rules.

**No surface changed** [R, `EmuSen_CoreAPI.md` §6.13 and `CoreDescriptors.cs`]. The schema has had `effect:
enhancement`, `hardware` and `cost` since 1.0, and C3 checks them; Mistress's Graphics Settings window lists a core's
settings in the console's tab and shows a trade-off's cost beside its help (`CoreSettingDescriptor.Hint`), so an
enhancement of Diamond's appears there with nothing added. Grouping by `effect`, which the schema's text asks of a
frontend, is not built in Mistress yet; it is the frontend's when the first enhancement is offered (§11).

## 6. The goldens

### 6.1 What, and where

Recorded on WiseMan 177f5041's accurate path, before any of E1's code, in `EmuSen/Cores/Sony/Diamond -
PlayStation/tests/goldens/`, by `tests/programs.rs`'s three golden tests [M]:

| File | What | Lines | Checked |
|---|---|---|---|
| `programs.txt` | every program of `programs/`, 49, each for 60 frames: on the open kernel for the kernel's own, bare for the rest, a card in slot 1 for the card programs, stage 7's synthetic disc for the CD's | 49 | routinely |
| `ps1-tests.txt` | every program of ps1-tests' release build 158 but its tools, 48, on the open kernel, for the frames WiseMan's gates run them (900 for those graded by their TTY, 1,500 for `gpu/bandwidth`, 315 and 309 for the movies, 300 for the rest), `timers` on a PAL console as its log was | 48 | with `EMUSEN_DIAMOND_PS1TESTS` |
| `discs.txt` | the eight bench discs, each played by its press script of `EmuSen.WiseMan/Reference/analysis/psxplay/` from the anchor its second line gives for the open kernel, a blank card in slot 1, to frame 3,000; at every 300th frame | 8 discs, 80 anchors | with `EMUSEN_DIAMOND_DISCS`, the folder of the tester's copies |

A program's line holds FNV-1a digests of every frame's picture and of the sound, each folded from the start, and of the
state, VRAM and the TTY at the end; an anchor's the pictures and the sound folded to it and the state then. Digests and
frame numbers only: no picture, no sound, no game data. ps1-tests' verdicts are graded where they always were, by
WiseMan's gates against the console's logs and pictures; the goldens pin Diamond's own outputs, from which the verdicts
follow, and the gates were run on this branch's library too (§8). Mega Man Legends' copy lives beside the others as a
link to the probe's `mml/` folder.

### 6.2 Recording them again: the rule [C]

A golden changes only when an accurate stage changes what the machine computes, and then:

1. **In a commit of its own**, made after the accurate change is committed and before anything else, by
   `EMUSEN_DIAMOND_GOLDENS=record cargo test --release --test programs -- give_their_goldens` with both corpus
   variables set, which writes the three files from the plain run.
2. **Its message names the change that moved them** and the files and lines that changed, and each changed line is
   explained by that change in the change's record (its page or its disputes entry). Lines that changed and that the
   change does not explain are a defect, not a re-recording.
3. **Never with an enhancement's code.** A commit that touches a layer, the observer's hooks or a setting of `effect:
   enhancement` does not touch the goldens; if it moves one, it is wrong.
4. **A new program, script or disc** adds its line in the same way, its message saying it is an addition.

## 7. The exactness harness

### 7.1 The test observer

`diamond::layer::Tally` is a layer that counts every event by kind, folds every value it is told into a digest, and
keeps its counts in a block of its own (`TALY`, 72 bytes), so that it reads everything a layer can be told and
exercises the state block. It is public, so that the bench can cost it (§9).

### 7.2 What runs

Each golden test runs every golden twice: plain, with the layer compiled in and no layer attached, and with the test
observer attached from the start, detached for the state's digest at each anchor and attached again. Both must equal
the recorded line, digest for digest; the observer's counts are printed and must be non-zero. The program goldens run
routinely (Diamond's own in every `cargo test`, ps1-tests' with its corpus as every other corpus test), the discs'
opt-in. At the end of the stage [M]:

| Goldens | Plain | With the test observer | What the observer was told |
|---|---|---|---|
| `programs.txt`, 49 lines | equal | equal | 15,470,095 loads, 4,105,093 stores, 219,001 coprocessor 2 moves, 4,242,451 copies, 30,360 projections, 1,139 words to GP0 from the processor, 20,526 by DMA |
| `ps1-tests.txt`, 48 lines | equal | equal | 174,244,549 loads, 39,777,154 stores, 140,102 coprocessor 2 moves, 47,151,083 copies, 1,101 projections, 1,250,420 words from the processor, 21,231,047 by DMA |
| `discs.txt`, 8 discs, 80 anchors | equal | equal | 568,459,528 loads, 262,056,049 stores, 58,769,687 coprocessor 2 moves, 149,471,242 copies, 7,587,410 projections, 136,155 words from the processor, 109,390,428 by DMA |

By disc, over the first 3,000 frames: Crash Bandicoot 2,195,348 projections, Spyro the Dragon 3,703,884, Mega Man
Legends 978,426, Ape Escape 506,206, Silent Hill 176,128, Castlevania 17,998, Rayman 9,420, and **Tekken 3 none**: its
title is at frame 6,540 on the open kernel, so its first 3,000 frames are the logos and the opening, drawn without the
GTE's projection [M]. Whether its fights project through RTPS and RTPT is E4's to measure on a later stretch.

The plan's P9, "with every enhancement off, the build reproduces every accurate golden from E1 on, with no exception",
**held** for E1 [M]. It remains open for E2 to E6, each of which must keep it.

### 7.3 The kit's C9, and why the check stays Diamond's

The kit's C9 runs each `exact` setting at its other values and asks for the default's digests. Extending it to
`enhancement` would ask what, within one build? An enhancement at its hardware value is the default (C3), and a machine
made with every default stated is the machine made with none (C4); a third case would add nothing. What E1's gate asks
is a comparison across builds, the accurate build against the build with the layer compiled in, and the kit carries
no outputs of an earlier build and must ship no images (`EmuSen_CoreAPI.md` §12.3). So no case was added to the kit
[C], and no other core's cases changed. **Proposed** for the tester, not built: an option of the kit's, `--goldens
FILE`, which records the digests of C6's run at fixed frames on the author's images and checks a later build against
them, so that any core could hold its accurate path across an enhancement's branch. It would be an addition to the
kit's options, not to any case, and the core's author would keep the file.

## 8. The gate, item by item

| Item | Result |
|---|---|
| Every accurate golden, with every enhancement compiled in and off | **Met** [M]: 49 + 48 lines and 80 anchors, equal (§7.2) |
| The same with the test observer attached | **Met** [M] (§7.2) |
| States made with and without the layer load both ways | **Met** [M]: in the build, both directions and the refusals (§4); across builds, WiseMan's build loading this branch's state with a block (§4) |
| C3 | **Met** [M]: the kit's C3 passes on the library, eight settings and no enhancement; the category's two forms pass the same rules in the crate (§5) |
| C7, C8, C9 on the eight discs at 1,800 frames | **Met** [M]: the kit's corpus form, `--frames 1800`, on the eight discs, on the final library: every case passes on every disc (C6, C7, C8, C14, C16–C18 per disc; C1–C5, C9–C13, C15 once), the state 5,398,770 bytes with the kit's two DualShocks and a card, as at version 10; C9 notes that no setting declares `exact`; 265 s, four at a time |
| ps1-tests' verdicts and pictures through WiseMan's gates | **Met** [M]: WiseMan under the test lock with the corpus, `FullyQualifiedName~Diamond`, 46 of 46, the TTY gates, the VRAM gates and the kit case among them |
| No regression of the frame unobserved | **Met** [M] (§9.1) |

## 9. Cost

The instrument is `Diamond_Speed.md` §2's: `examples/bench` on links with every function aligned to 64 bytes, the
builds alternating on one core (`taskset -c 11`), the fastest mean of five rounds, milliseconds of the emulation
thread a frame. The base is WiseMan 177f5041. Two links of the base's own source, built in two folders, agreed within
0.3 per cent on every run here [M]; `Diamond_Speed.md` §2 found 0.1 to 0.9 per cent between two aligned links of one
source.

### 9.1 Unobserved

The final build against the base, five rounds [M]:

| Run | Frames | Base | Branch | Difference |
|---|---|---|---|---|
| `gpu/benchmark` | 1,200 | 2.334 | 2.332 | −0.1% |
| `gpu/bandwidth` | 1,200 | 2.989 | 2.942 | −1.6% |
| Crash Bandicoot | 3,000 | 2.306 | 2.303 | −0.1% |
| Mega Man Legends | 3,000 | 1.793 | 1.794 | +0.1% |
| Tekken 3 | 3,000 | 2.372 | 2.376 | +0.2% |

Every pair's picture, sound and state digests are equal. **No regression beyond the noise.** Earlier runs of builds
one step before this one (§9.2's item 3) had shown Mega Man Legends and Tekken 3 0.2 to 0.7 per cent above the base;
the final run does not, and `gpu/bandwidth`'s 1.6 per cent below is a placement effect of the kind §2 of
`Diamond_Speed.md` records, not a gain claimed.

### 9.2 What it cost before it was found

The first build, the reports under an empty default and `advance` choosing between two forms by a constant, cost more
than the instrument's error [M, five rounds]:

| Run | Base | First build | Difference |
|---|---|---|---|
| `gpu/benchmark`, 1,200 frames | 2.341 | 2.349 | +0.3% |
| `gpu/bandwidth`, 1,200 frames | 3.030 | 3.044 | +0.5% |
| Crash Bandicoot, 3,000 frames | 2.336 | 2.387 | +2.2% |
| Mega Man Legends, 3,000 frames | 1.808 | 1.839 | +1.7% |
| Tekken 3, 3,000 frames | 2.401 | 2.462 | +2.5% |

Each cause was found by building the part that might hold it alone, comparing the two links' symbol sizes (`nm -S`)
and disassembly, and timing the pair. No performance counters were at hand (the machine has no `perf`), so the
argument is by ablation.

1. **The processor's reports.** A build with only the BlackMoon crates changed cost Crash Bandicoot 0.7 per cent and
   Tekken 3 1.4. No call remained in the step, but `exec::run` for the board was 95 bytes longer, a register move
   added to every arm of the computing instructions (`BlackMoon_R3000.md` §14). Under `if B::MOVES`, `exec::run` and
   `Run::execute` for the board are the base's in size.
2. **The frame's shape.** The first form of the choice was a constant, `advance_in::<const LAYERED: bool>`, its step
   written `if LAYERED { through the layered bus } else { on the board }`. With the constant false the compiler made
   one copy of the instruction path in the frame's function where the base's `advance` holds two, specialised by the
   kernel's putchar tap, and placed the single steps otherwise: 13,417 bytes with them inlined, 5,037 out of line,
   against the base's 8,669. That build stood 0.8 to 1.3 per cent above the base on Tekken 3 in three runs, and so
   did the same build with the board's field and the receipt's test both removed (+1.2), which put the cost in the
   frame's shape. Written generic over `Through`, with one call where the constant had two, the plain frame, the
   single step and the checked run are each the size the base's was: 8,669, 6,765 and 4,845 bytes.
3. **The board's field: tried, and not shown.** With the frame's shape restored, Tekken 3 was still 0.5 to 1.2 per
   cent above the base in five runs, and at the base (+0.25) in a build whose `Attached` had no size. `Attached` is
   sixteen bytes, placed by the compiler before the memory-control registers and the I-cache, which it moves sixteen
   bytes within their host cache lines. The prediction was that this placement was the cost; a form of `Attached`
   filling a whole line, 64 bytes, so that every field after it kept its place in a line, measured +0.1 per cent once
   and +0.5 to +0.7 in four further runs, no nearer the base than sixteen bytes had been. The prediction did not
   hold, and the padding was taken out again.

Ruled out the same way: **the receipt's test on every GP0 word.** A build with DMA's receipts chosen at compile time
for a whole piece, and a build with no receipt told at all, were no faster than the build with the test (Tekken 3, the
disc with the most words by DMA, 11,500 a frame: +0.6, +0.8 and +0.6 per cent in one run of seven rounds) [M]. The
test stays, as the simpler form.

**The residue.** Between the restored frame and the final build, Mega Man Legends and Tekken 3 stood about half a per
cent above the base in some runs and not in others; a build with `Attached` of no size and the layered form's code gone
stood at the base. No executed cause was found, and the final run (§9.1) shows none. Should a later measurement show it
again, the layer's home outside the board is the thing to try, which would need the board's DMA path to reach the layer
some other way (§11).

### 9.3 Observed, for E4's budget

The same runs with the test observer attached (`bench --observed`), the layer told every event through a `dyn Layer`
and adding each value into a digest [M]:

| Run | Plain | Observed | Difference | Events a frame |
|---|---|---|---|---|
| `gpu/benchmark` | 2.332 | 2.395 | +2.7% | |
| `gpu/bandwidth` | 2.942 | 3.150 | +7.1% | |
| Crash Bandicoot | 2.303 | 2.472 | +7.3% | 57,700 |
| Mega Man Legends | 1.794 | 1.896 | +5.7% | 39,100 |
| Tekken 3 | 2.376 | 2.592 | +9.1% | 61,600 |

About 0.10 to 0.22 ms a frame, or 2.6 to 3.5 ns an event on the three discs (the events of the goldens' 3,000 frames,
which the bench's first 3,000 without presses approximate). That is the floor E4 starts from: the dispatch and the
events' making, before any work of the layer's own.

## 10. Predictions

- **P9** (the plan's): held for E1 (§7.2).
- **P-E1a** [recorded 2026-10-13, retired the same day]: "an empty default method compiles to nothing". It held for
  the calls and failed for the code around them (§9.2).
- **P-E1b** [recorded for E4]: E4's layer, doing for each event a table lookup and a few stores where the test observer
  does a few additions, will cost at most twice the test observer's cost over the plain frame on the bench discs.
- **P-E1c** [recorded for E4]: on Crash Bandicoot and Spyro the Dragon, most words GP0 receives by DMA within a
  projected game scene will find their integer in the layer's shadow; Tekken 3's fights are the plan's P10 test of a
  game that may build its geometry otherwise.

## 11. What E3 and E4 need next from this frame

- **E3, the Vulkan renderer at 1×**: a seam at `Gpu::execute` for whole commands with their times and drawing state
  (`Diamond_GPU.md` §13), added as a method of `Layer` with an empty default, beside `received`; the renderer as a
  layer of its own or a part of E4's, attached the same way, its caches in a block of its own tag if it keeps any; the
  shared device of `emusen-gpu` (the `shared-gpu` branch, merged into WiseMan before this one). E3 changes no
  golden.
- **E4, the precision side-channel**: the crate `precision` (the plan's Q14), a `Layer` holding the shadows of §3.4,
  its block `PREC`; the setting "Precise geometry" declared as an `Enhancement` (`Values::Switch`) whose `set_settings`
  attaches and detaches it between frames, with a `STATE_SIZE` event when the state's length changes by the block; the
  queue of side data beside the GPU's FIFO fed by `received`; its cost against §9.3's budget. If the dynamic call per
  event costs more than E4 can afford, the board's `Attached` can hold E4's concrete type instead of a `dyn Layer`
  without changing any event.
- **The frontend**: grouping a core's `enhancement` settings in their own group in Graphics Settings, as the schema's
  text asks, when the first one is offered.

## 12. Tests run

- `blackmoon-r3000`, `cargo test --release` with `EMUSEN_BLACKMOON_CORPUS`: 38 (one added,
  `each_move_is_told_with_its_values`; `a_run_is_its_steps_on_random_programs` now compares the moves), the
  suite's 55,000 cases among them.
- `blackmoon-gte`, the same: 10, `an_observer_sees_each_projection_and_changes_nothing` holding the new fields.
- `diamond`, `cargo test --release`, with `EMUSEN_DIAMOND_PS1TESTS` and `EMUSEN_DIAMOND_DISCS` and without: 165 unit tests (three added: the layer's events, its block both ways, the enhancement declaration) and 81 program tests (three added: the three goldens, each run plain and with the test observer), all passing both ways.
- The kit, its corpus form on the eight discs at 1,800 frames (§8).
- WiseMan under the test lock with the corpus, `FullyQualifiedName~Diamond` (§8).
- `emusen-native` and `emusen-core-conform` were not changed, so the other cores built on them were not run.

## 13. Reading log

No description of PGXP or of any other emulator's precision or enhancement layer was opened for this stage, and no
emulator source of any kind; no firmware, listing or trace of firmware; nothing under `kernel/` or `kernel-harness/`
was opened or changed. The event set was designed from EmuSen's own pages and code: `Diamond_Plan.md` §1, §6, §7, §8,
§11 Q14 and §13; `Diamond_GPU.md` §2 and §13; `Diamond_Native.md` §2.6, §2.7 and §23; `Diamond_Speed.md` §1, §2, §4
and §14; `BlackMoon-HW/README.md`, `BlackMoon_R3000.md` §2 and §13, `BlackMoon_GTE.md` §2, §5 and §9;
`Beryl-HW/README.md`; `EmuSen_CoreAPI.md` §2.1, §6.13 and §12; the source of `blackmoon-r3000`, `blackmoon-gte`,
Diamond's `bus.rs`, `machine.rs`, `state.rs`, `idle.rs`, `debugger.rs` and `v1.rs`, `emusen-native`'s state reader,
writer and settings, `emusen-core-conform`'s C3 and C9, WiseMan's ps1-tests gates and `CoreDescriptors.cs`, and the
press scripts and `psxplay.py`. What precision does was taken from the plan's own description of it (§6.2, §6.3),
itself written from published descriptions, and from the hardware's behaviour as Diamond already models it: the GTE's
fixed-point projection and FIFOs, the GPU's integer vertex words, DMA channel 2's lists.

---

## 14. Stage E3: the renderer's form, decided before it was built

*Written 2026-10-13, before any of E3's code.* The plan's row: "The Vulkan renderer at 1×, consuming the command
stream, beside the software rasteriser. Gate: with no enhancement, its picture equals the software rasteriser's
display on ps1-tests' GPU programs and the bench discs, exactly or within a stated count of pixels, each difference
explained; E1's goldens unchanged (the renderer never writes machine state)." The tester added Diamond's own GPU
programs to the gate, the renderer's run on `emusen-gpu` (`Diamond_Plan.md` §11 Q14(c)), and that the form be argued
here, in writing, first. `EmuSen_Gpu.md` §6.2 left the form open and named what each would ask of the device.

### 14.1 What the form must serve

- **E3, exactness at 1×.** The software rasteriser's rules, to the pixel (`Diamond_GPU.md` §5, §6): a pixel is the
  point at its integer coordinates, inside when every edge function is positive or zero on an edge the triangle owns,
  the bottom row out; colours and texture coordinates from the leftmost vertex by gradients of 12 fractional bits
  truncated toward zero, a half added before the integer part; the dither matrix by VRAM's coordinates modulo 4; the
  four blends at five bits with saturation; the mask bit set and checked; the texture window; the 11-bit fold of D-47;
  the interlaced field's skipped lines; a rectangle's texel pairs; a line's tie rounding. Each is integer arithmetic.
- **E4, precise geometry.** The same coverage and interpolation with vertices carrying fractional bits, and a division
  per pixel for perspective-correct texturing (`Diamond_Plan.md` §6.3).
- **E5, internal resolution.** Drawing at 2× to 4×, with the effects that read back what was drawn: textures sampled
  from VRAM that earlier primitives drew (render-to-texture), VRAM-to-VRAM copies, CPU uploads into a scaled VRAM, the
  display read from the scaled VRAM, and the mask bit and the blends reading the very pixel they write.

### 14.2 The two forms, argued

**A compute rasteriser over the renderer's own mirror of VRAM**, or **a graphics pipeline drawing into images**.

1. **Exactness.** A graphics pipeline samples coverage at pixel centres with an implementation's sub-pixel precision
   and interpolates in floating point; neither is the PlayStation's integer rule. To be exact at 1× such a renderer
   would rasterise a conservative quad and evaluate the PlayStation's own edge functions and interpolants in its
   fragment shader, keeping nothing of the fixed-function rasteriser but its enumeration of pixels [A]. A compute
   shader evaluates the same integer rules per pixel as the software rasteriser states them, and nothing else.
2. **The mask bit.** A pixel is left alone when the mask is checked and its bit 15 is set. Fixed-function blending
   cannot make a write depend on the destination's alpha; an ordered read-modify-write of the framebuffer in a fragment
   shader needs an extension outside core Vulkan (fragment shader interlock) [A]. B−F and B+F/4 at five bits would be
   blend equations; the mask would not. In compute each invocation owns its pixel and meets the primitives covering it
   in drawing order, reading and writing it in a register, as Mars's walk does (`Mars_Gpu.md` §5.3, §14.1: "each pixel
   still meets its rows in drawing order, which is the one thing the binning must keep").
3. **One surface.** The PlayStation draws into the surface it textures from, copies within it, and displays from it.
   `Mars_Gpu.md` §5.1's case for one mirror over a set of images ("images overlap and alias; a pixel's address is
   arithmetic on a base") holds more strongly here, as `EmuSen_Gpu.md` §6.2 says. In a graphics pipeline a primitive
   textured from the image being drawn is a feedback loop that needs the region copied first; in compute it is a read
   of another word, sound once the reads are ordered after the writes they depend on (§14.4).
4. **E4 and E5.** E4's fractional vertices and per-pixel division are arithmetic in the same shader. E5 evaluates the
   same coverage at N× sample positions over an N² mirror, and the uploads, copies and display reads address that mirror
   at N×; nothing of the compute form changes shape between E3 and E5. Mars draws at 2× to 4× this way, at 159 to 192
   per cent of full speed at 4× on this machine (`Mars_Gpu.md` §13.4, §14.6) [R].
5. **The device crate.** Compute over storage buffers asks of `emusen-gpu` what it has: buffers in both memories,
   compute programs with push constants, fills, copies, waited submissions. The graphics form would ask images, image
   views and their layouts, samplers, graphics pipelines with vertex input and blend state, render passes or dynamic
   rendering, a graphics-capable queue, and rule 2's barrier widened to the graphics stages, which MarsRT records too
   (`EmuSen_Gpu.md` §6.2) [R].

**Decided [C]: a compute rasteriser over the renderer's own mirror of VRAM**, one 32-bit word a pixel so that no two
invocations write one word (`Mars_Gpu.md` §5.1's reason, a sixteen-bit pixel being half a `uint`).

### 14.3 The one thing the crate lacks: a program dispatched twice in one submission

`Commands::dispatch_in` writes the program's descriptor set each time it records a dispatch, and a descriptor set
written after it was bound in a command buffer that is still recording invalidates that buffer. MarsRT dispatches each
program once a submission [R, `rdp/gpu/mod.rs`], which is why it never met this. **Shown, not assumed** [M, a scratch
program on the RX 6800 under `VK_INSTANCE_LAYERS=VK_LAYER_KHRONOS_validation`]: two dispatches of the crate's test
program in one `submit` give `VUID-vkCmdBindPipeline-commandBuffer-recording`, "VkCommandBuffer ... is now in an invalid
state ... VkDescriptorSet ... was destroyed or updated without UPDATE_AFTER_BIND", at the second.

The renderer must dispatch its walk once for every stretch of commands between two that read what an earlier one in
the stretch drew (§14.4), several a frame when a game draws into a texture. The alternatives were a waited submission
for each stretch, a round trip to the device each time [A], or a copy of each program per stretch. **Decided [C]: an
addition to the crate, `Commands::dispatch_again`**, the program dispatched once more in the same recording with the
buffers its set was written with there, binding the set without writing it, refused (a panic, as the crate's other
misuses are) when this recording has not written that set. Version 1.1.0, an addition by the crate's rule (§3 of
`EmuSen_Gpu.md`); nothing MarsRT calls changes.

### 14.4 The design in outline

- **The stream.** The renderer is a `Layer`. The GPU tells it each command whole as the command starts, with its words,
  its cycle and the drawing state that applies; each further word that continues a command (a polyline's vertex, a
  CPU-to-VRAM transfer's data); and each line the display reads, with where it reads it from. The display's timing is
  the machine's (`video.rs`), so the renderer is told which line is read when rather than keeping a second clock; it
  reads that line from its own VRAM.
- **The host's share.** Each primitive is decoded by the GPU's own functions, shared and not copied, into a record:
  its kind, its bounds within the drawing area, its edges, its gradients computed by the software rasteriser's rule,
  its texture, CLUT, blend and mask. Records are binned into tiles of VRAM in drawing order.
- **The device's share.** One invocation a pixel and one workgroup a tile: each invocation loads its pixel, meets every
  record of its tile in order, and stores the pixel once (Mars's walk). Products of a gradient and a distance exceed 32
  bits; they are made exact in pairs of 32-bit words (`imulExtended`, `uaddCarry`), which core SPIR-V has, rather than
  by asking for the optional 64-bit integer feature, which the crate does not enable.
- **Order across pixels.** A texel, a CLUT, a display line and a VRAM-to-VRAM copy read pixels other invocations own.
  The host keeps the regions written and the regions read as textures within the present stretch, and ends the stretch
  (another dispatch, the same submission) before a read of a region written in it or a write of a region read in it.
  CLUT loads and display lines are read at the start of their stretch; a copy is a stretch of its own, read whole into
  a scratch buffer and then written, in as many steps as its rows require (a copy downward over itself reads rows
  that its earlier rows wrote).
- **The CLUT cache** is reproduced: its tag is part of the drawing state, so the renderer loads a palette exactly when
  the GPU does and keeps it while the GPU keeps its own. **The texture cache is not**: which entries it holds depends
  on every texel fetched before in an order no parallel walk keeps, so the renderer reads VRAM as it stands. A picture
  that differs because the cache held a texel VRAM no longer does is the one difference the form predicts.
- **What it owns.** Its mirror, its picture and its scratch; nothing it computes reaches the machine. It is seeded from
  the GPU's VRAM, picture and CLUT cache when it is attached or a state is loaded, and keeps nothing of its own across a
  state at 1×, where what it would keep is the machine's VRAM; at N× (E5) its mirror is what a block would hold.

### 14.5 The shader route, and where the renderer lives

- **Shaders**: GLSL `.comp` sources in the renderer's `shaders/`, each with its `.spv` beside it, taken into the build
  by `include_bytes!`; compiled by shaderc through `Silk.NET.Shaderc` in WiseMan, with Mars's options
  (`Mars_Gpu.md` §2); a WiseMan test recompiles every source and requires the bytes on disk, and
  `EMUSEN_GPU_SHADERS_RECORD=1` writes them. No build needs a compiler. `include_bytes!` makes the embedded copy the file
  itself, so Mars's third comparison (against the assembly's resource) has no counterpart.
- **The crate**: `diamond-renderer`, in `Diamond - PlayStation/renderer/`, depending on `diamond` (the `Layer`, the
  stream's events, the GPU's decoding) and on `emusen-gpu`. `diamond` does not depend on it: the machine cannot reach
  its renderer. The run tool, the bench and the tests take it as a development dependency. The plan's §6.2 asked for
  each part of the layer to be a crate of its own; this is the renderer's.

### 14.6 Predictions [recorded before the code]

- **P-E3a.** With no enhancement, the renderer's picture equals the software rasteriser's displayed picture on every
  ps1-tests GPU program and every program of `programs/` exactly, at every frame compared.
- **P-E3b.** On the eight bench discs along their golden anchors, every differing pixel, if any, is the texture cache's
  (§14.4), and there are fewer than a thousand over the anchors.
- **P-E3c.** E1's goldens are unchanged with the renderer attached, by construction: it is told copies and writes only
  its own buffers.
- **P-E3d.** Unattached, the frame's cost is the base's within the bench's noise.
- **P-E3e.** Attached, with the picture waited for each frame, the emulation thread pays under a millisecond a frame
  more on the bench discs on the RX 6800, the host's decoding and binning under a third of it.

## 15. The stream seam (E3)

### 15.1 Where the layer hangs

E1 kept the attached layer in the board (`Board::layer`). Since E3 it is the GPU's field, **`Gpu::layer`**, and the
board reaches it there for what it tells (the GP0 receipts, the processor's moves through `LayeredBus`, the GTE's
projections) [C]. The renderer's events arise inside the GPU's command processor and its display, which the board
reaches only through the GPU's own methods (`gp0`, `gp1`, `run`, `sync`, `status`, `gpuread`); held beside them, the
layer needs no argument threaded through each of those and of the fifty calls the GPU's tests make. It is still no
part of the state, and a copy of the GPU, as the levers' checks make, carries none.

`Machine::attach` gives the layer the GPU as it stands before attaching it (`Layer::seed`), and a state's load gives it
the loaded GPU after its own block; `Machine::layer` lends the attached layer to whoever wants to ask it what it made,
as the renderer's tools do (§16).

### 15.2 What the GPU tells

Three methods of `Layer`, each with an empty default, beside E1's:

- **`executed(&Command)`**, from `Gpu::execute`, before the command draws: `words`, the command's words whole;
  `part`, `Whole`; `at`, the cycle it starts; and `drawing`, a `gpu::decode::Drawing`: GP0(E1h) to (E6h) as they stand,
  the chip's version, GP1(09h), the parity of the lines the interlaced field leaves alone at that cycle, and the CLUT
  cache's tag. Every GP0 command reaches it, the environment commands too. The two kinds of word that continue a
  command after it started are told the same way, since the GPU acts on each as it comes: **`Vertex`**, a polyline's
  next vertex, as its colour word (the command's when the line is flat) and its position; and **`Data`**, a CPU-to-VRAM
  transfer's word.
- **`shown(Shown)`**, from the display: `Size`, when the picture's size changes and it starts black, and `Line`, each
  line read, with the row, the VRAM position it is read from, its width, its depth and whether it is blanked. The
  display reads a line at the time its timing reaches it (`Diamond_GPU.md` §7); a layer is told that time's choice and
  does the reading itself.
- **`seed(&Gpu)`**, above.

The decoding a renderer needs is the GPU's own, moved into `gpu/decode.rs` and called by both: the vertex under the
drawing offset (D-23) and its placing, the drawing area, the page attribute, a polygon's, a rectangle's and a line's
words, and `Drawing`'s `paint`, `texture`, `with_page` and `loads_clut` (`ClutCache::holds`, the cache's own test).
The GPU's code calls the same functions it had inlined, with the same arguments.

Nothing is made unless a layer is attached: the command processor tests `layer.is_some()` before building a
`Drawing`, and the display before each line's event (§19 gives the cost).

**Tested** [M]: `a_layer_is_told_each_command_its_continuations_and_each_line_shown` sends a program's GP0 words (the
four environment commands, a triangle, a polyline of three vertices, a three-pixel transfer) and holds that the layer
is told each command whole in order with its words, the polyline's third vertex and the transfer's two data words as
continuations, the triangle under the state the commands before it set, every line of three fields with its row,
source and width, and the GPU as it stood when attached; and that the machine is the plain run's to the state's last
byte. Diamond's suite passed unchanged with the seam, its 49 program goldens among them, plain and with E1's test
observer.

## 16. The renderer (E3)

### 16.1 The crate

`diamond-renderer` (`Diamond - PlayStation/renderer/`): `lib.rs`, the layer, which reads the stream and plans the
device's work; `record.rs`, a primitive as the walk reads it; `blocks.rs`, where in VRAM a stretch has drawn or read,
and which tiles a span covers; `device.rs`, the buffers, the four programs and one submission's commands; and
`shaders/`, `walk.comp`, `scan.comp`, `clut.comp` and `copy.comp` with their `.spv`. It depends on `diamond`,
`blackmoon-gte` (the layer's supertrait) and `emusen-gpu` 1.1. `diamond` takes it as a development dependency only, for
the run tool, the bench and the gate: the core's library links none of it, and nothing in the machine names it.

`Renderer::new(name)` opens the device (`EMUSEN_GPU_DEVICE` when no name is given) or says why not;
`Machine::attach(Box::new(renderer))` attaches it; `picture()` and `vram()` submit what is planned, wait, and read back
the picture (RGBA at its size) or VRAM (a halfword a pixel), for a tool or a test that asks.

### 16.2 What the host does with the stream

- **A whole command** is decoded by `gpu::decode` under the `Drawing` it came with. A polygon takes its page attribute
  into the state first (`Drawing::with_page`), as the GPU does before drawing it; its texture is `Drawing::texture`; its
  paint `Drawing::paint` with the GPU's own choice of dither; each triangle is folded (D-47) and set up by
  `raster::setup`, the function the software rasteriser's `triangle` now calls for its own setup, so the edges, the
  ownership, the base vertex and the gradients are the software's values, not a restatement of them. A rectangle and a
  line likewise from `decode::rectangle` and `decode::line`; a polyline's further vertices from `decode::place`. A fill,
  a copy and a transfer from `Rect::fill_params` and `Rect::copy_params`.
- **A record** is 48 words (`record.rs`, laid out as `walk.comp` reads it): the kind and the paint's flags, the bounds
  within the drawing area, the texture's page, depth, palette slot and window, and the kind's own words: a triangle's
  three edges with whether each owns its pixels, its base vertex, its five attributes there and their gradients; a
  rectangle's corner, first texel and steps; a line's ends, steps, colour and gradients; a fill's rectangle and colour;
  a transfer's rectangle, the run of its pixels the record holds, and where they are in the submission's data.
- **A transfer's data** is kept as it comes, two pixels a word, and the open record's count grows with each; the record
  is placed in its tiles when it can take no more (the next command, or the end of its stretch).
- **The CLUT cache** is reproduced from the `Drawing`'s tag: a palette is loaded into a new slot of 4,096, at the start
  of its stretch, exactly when `ClutCache::holds` says the GPU's cache loads; otherwise the slot loaded last is used, as
  the cache keeps its colours (stale ones included, `clut-cache`'s case).
- **A display line** is recorded with its row; a row read twice in one stretch keeps the later. A new size clears the
  picture at the start of the stretch and drops the lines of it recorded before the size changed.

### 16.3 On the device

**The walk** (`walk.comp`): a workgroup of 8 by 8 invocations a tile of VRAM, an invocation a pixel. It loads its
pixel, meets each record of its tile in drawing order and applies it, and stores the pixel once. A triangle's pixel is
inside when each edge function `dx·(y−ay) − dy·(x−ax)` is at least 0 on an owned edge and 1 otherwise, which is
`raster::triangle`'s per-row bound restated for one pixel (the row's `x ≤ ⌊(k−t)/dy⌋` for an edge going down and
`x ≥ ⌈(t−k)/−dy⌉` for one going up are both that inequality); its attributes are the base vertex's value plus the two
gradients times the distances from it, plus a half, shifted down twelve, each product made exact in two 32-bit words
by `imulExtended` and summed by `uaddCarry`. A line's pixel finds its step along the major axis from its own
coordinate and is drawn where the minor coordinate the software's tie rule gives is its own. A rectangle's texel
steps, flips and pairs, the texel through the window, page and palette, modulation, the dither matrix, the four
blends, the mask bit and the interlaced field's skipped lines are `draw.rs`, `texture.rs` and `raster.rs` restated. A
drawn row past 511 lands on its row modulo 512, as VRAM's index has it. **No texture cache**: texels are read from
VRAM as it stands (§14.4).

**The other programs**: `clut.comp` copies a palette's entries from VRAM into its slot; `scan.comp` reads a display
line into the picture, 15-bit as `c<<3 | c>>2`, 24-bit as three bytes a pixel, black when blanked; `copy.comp` reads a
copy's rows into a scratch buffer, or writes them under the mask.

### 16.4 Order across pixels: stretches

A texel, a palette, a display line and a copy read pixels that invocations other than their own write. The host plans
the records as **stretches**: within one, no record reads what another of the stretch writes, so every read sees VRAM
as it stood at the stretch's start, which is VRAM as it stands at the read. `blocks.rs` keeps two sets of VRAM's blocks
of 16 by 16 for the stretch: those written and those read as texels. The stretch ends before a record whose texels meet
the written set or whose pixels meet the read set; before a palette load or a display line whose pixels meet the
written set (both run at the start of their stretch); and at every copy, which is a step of its own. A copy reads its
rows whole into the scratch buffer and then writes them; when it copies downward over itself, by `d` rows with
`0 < d < h`, row `j` reads the row that row `j − d` wrote, so it goes in steps of `d` rows, each read and written before
the next, which is exactly `vram.rs`'s row-by-row order. A transfer's rows are marked written as its data reaches them,
and a row that meets the read set ends the stretch there. A record whose texels and pixels share a block is counted
(`Counts::feedback`): the walk does not keep the order of its own pixels' writes and reads.

Each stretch's records are binned into the tiles of 8 by 8 pixels their bounds cover, each tile once (a span of 512
rows starting mid-tile covers every tile row once), counted, summed and placed in drawing order, as Mars's binning is.

### 16.5 One submission

Everything planned goes to the device in one waited submission when a tool asks for the picture or VRAM, or when an
array would overflow: the staged arrays (records, tile entries, tiles, palette loads, lines, transfer data) copied from
host memory to the device's own, then each stretch's commands in order (the picture's clear, its palette loads, its
lines, its walk) and each copy's steps, each dispatch after the first of its program a `dispatch_again` (§14.3), each
command followed by the crate's barrier; then the readback. Capacities a submission: 32,768 records, 1,048,576 tile
entries, 131,072 tiles, 4,096 palette loads, 8,192 lines, 2,097,152 transfer pixels. Device memory: VRAM and a scratch
of 2 MiB each, one word a pixel; the picture, 1.4 MiB; 4,096 palette slots, 4 MiB; the staged arrays twice, once on each
side, 34 MiB in all. Nothing is allocated a frame: the host's arrays and lists keep their capacity.

### 16.6 Seeded, and kept nothing across a state

`seed` takes the GPU's VRAM, its picture and size, its CLUT cache's colours (into slot 0) and the transfer or polyline
it has open, and drops whatever was planned, which was another machine's. The renderer has no state block: at 1× all it
would keep is the machine's VRAM, which a load gives it back.

## 17. E3's gate

### 17.1 What runs

`tests/renderer.rs` [M, 2026-10-13, the RX 6800 by `EMUSEN_GPU_DEVICE=6800`]: each run is a golden's run
(`tests/support/goldens.rs`, shared with `programs.rs`, so the renderer is graded on exactly the runs the goldens pin)
with the renderer attached at power-on. **At every frame** the renderer's picture is read back and compared with the
software rasteriser's displayed picture (`Display::frame`), size and every byte; **VRAM** is compared halfword for
halfword at the end of a program and at every 300th frame of a disc; and the run's golden digests, taken with the
renderer attached, are compared with E1's recorded lines. A machine without a device stands each test down with a
line, as `emusen-gpu`'s tests do.

### 17.2 The results, item by item

| Item | Frames compared | Pictures | VRAM | E1's goldens with the renderer attached |
|---|---|---|---|---|
| The project's programs, 49, as `programs.txt` runs them (60 frames) | 2,940 | **all equal** | equal on 47; `texture-cache.s` 128 halfwords and `triangle-rows.s` 3,680, the texture cache's (below) | `programs.txt`, 49 of 49 lines equal |
| ps1-tests' release build 158, 48 programs on the open kernel, on their goldens' frames (900, 1,500, 315, 309 or 300) | 22,824 | **all equal** | **all equal** | `ps1-tests.txt`, 48 of 48 |
| The eight bench discs, each by its press script to frame 3,000 | 24,000 | **all equal** | **equal at all 80 anchors** | `discs.txt`, 8 discs and 80 anchors equal |

ps1-tests' eleven GPU programs with pictures (`Diamond_GPU.md` §9) are among the 48, as are `bandwidth` (8,745
records, 400 copies, 7,648 stretches), `lines` (98,400 records), `vram-to-vram-overlap` (147 copies over themselves,
the stepped copy of §16.4 among them), `texture-flip`, `clut-cache` and `uv-interpolation`; the MDEC's movies at 15 and
24 bits are the 24-bit display and the CPU-to-VRAM transfers. On the discs the renderer drew 3.87 million records in
32,793 stretches, 3,355 copies among them (Silent Hill 2,999).

**The two differences, explained.** Both are in VRAM, neither reaches a picture (the two programs display nothing),
and both programs were written to exercise the texture cache: `texture-cache.s` changes VRAM under a page the cache
holds and draws from it again, `triangle-rows.s` reads back which texels a triangle taller than a cache block left in
the cache (D-41). The GPU's cache gives the texel it held, the renderer the texel VRAM holds (0x8001 against 0x8002 at
the first pixel of each). **Shown, not argued** [M]: a scratch build of the GPU whose cache always misses (its
`TextureCache::read` reading VRAM as it stands) draws both programs as the renderer does, and with it all 49 programs
are exact in VRAM too; the build was not kept. The counts are pinned in the test (`TEXTURE_CACHE_HALFWORDS`), so that a
change in either is seen.

No game of the eight, over its first 3,000 frames, draws a picture that depends on a stale texture cache entry.

**The predictions.** P-E3a held: every picture of the programs and of ps1-tests is equal. P-E3b held in its strong
form: no disc differs in a pixel or in VRAM, where the prediction allowed the texture cache a few hundred pixels.
P-E3c held: every golden line is the recorded one with the renderer attached. On the programs the picture comparison
is weak evidence by itself, since most leave the display blanked; VRAM is what grades them there.

### 17.3 The run tool and the bench

`examples/run.rs --renderer` attaches the renderer and writes its picture beside the machine's for `--picture FILE`
and `--pictures DIR N` (`NAME.renderer.rgba`), and its VRAM beside `--vram FILE`, each with the pixels that differ
counted (Crash Bandicoot at frames 300, 600 and 900: 0, and VRAM 0). `examples/bench.rs --renderer` waits for the
renderer's picture each frame and times the wait with the frame (§18).

**The kit is not given an option at E3** [C]. The conformance kit runs the core's library through the core ABI, and
that library is `diamond`'s own cdylib, which cannot link a crate that depends on `diamond` (§14.5). Attaching the
renderer there needs what E4 needs for its setting: the core's exports in a crate of their own that links the machine
and its layers, with the setting attaching and detaching them between frames. **The picture through v1 needs no
change of the ABI** [A]: `Core::frame` gives the host the core's RGBA picture at the size `frame_info` and the
`GEOMETRY` event report, and with the renderer attached it would give the renderer's picture in the same form; at 1×
the size is the display's, and at N× (E5) a larger size is what `GEOMETRY` already announces.
