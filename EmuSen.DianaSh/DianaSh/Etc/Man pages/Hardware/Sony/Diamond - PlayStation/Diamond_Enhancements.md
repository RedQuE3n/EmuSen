# Diamond_Enhancements — the enhancement layer, stages E1 and E3 to E5

*Written 2026-10-13, at stage E1 of `Diamond_Plan.md` §7, with the tester's go of the same day for precise geometry
and a shared GPU crate.* E1 builds the frame every enhancement of the plan's §6.3 will stand in, and the proof that the
frame changes nothing while it is off: the observer on the R3000A and the GTE with the GPU's receipt of a vertex word,
the attachment of a layer to the machine, the optional state block, the settings' category, the accurate goldens and
the exactness harness. No enhancement is built and none is offered: "Precise geometry" (§11 Q14 of the plan) arrives
with E4, the Vulkan renderer with E3. The build record's entry is `Diamond_Native.md` §24.

*Stage E3, the same day* (§14 on): the Vulkan renderer that draws the picture from the GPU's command stream beside the
software rasteriser, at 1×, equal to it. §14 is its form, decided and committed before any of its code.

*Stage E4, the same day* (§21 on): precise geometry, the precision layer that carries each projected vertex's
sub-pixel position and depth to the GPU's receipt of its word, and the renderer drawing such polygons between the pixels
with their textures in perspective, offered to players as the setting "Precise geometry". §21 to §24 are its design,
decided and committed before any of its code.

*The crack fill, the same day* (§32 on): the cracks precise geometry left between polygons, reported by the tester
from the handheld, reproduced, counted and classified; the fill that closes them, a precise polygon also drawn
wherever the console's own is; its measurement on both sides, the decision, the gate and the cost.

*Stage E5, 2026-10-14* (§37 on): the picture's wait taken off the emulation thread and the renderer's planning made
cheaper, then internal resolution, true colour and texture filtering, each a setting of its own beside "Precise
geometry". §37 to §42 are its design, decided and committed before any of its code; §43 to §51 what was built,
the gate, the pictures, the cost and what is left.

*Following a shift, the same day* (§54 on): a trial of §32.4's proposal, that the processor tell a shift by a constant
so that precise geometry follows the vertices Spyro the Dragon keeps shifted; the game's pattern read and the design
with its keep rule committed before the code (§54), then Spyro and the other discs measured both ways, the decision,
the gate and the cost (§55).

*The picture on the device, 2026-10-15* (§56): the renderer on a device the host lends through the core ABI's 1.9,
its picture handed on that device the frame it is drawn, so that §38.1's frame of latency goes where a host lends;
and a defect of the renderer's drop found on the way.

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
`Drawing`, and the display before each line's event (§18 gives the cost).

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
side, 33 MiB in all; 44 MiB with the readback buffer, 26 of it the device's own. Nothing is allocated a frame: the host's arrays and lists keep their capacity.

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

## 18. Cost

### 18.1 The instrument

§9's, unchanged: `examples/bench` on links with every function aligned to 64 bytes, each build in its own target
folder, the builds alternating on one core (`taskset -c 11`), the fastest mean of five rounds, milliseconds of the
emulation thread a frame. The base is WiseMan 84dffc65, extracted outside the repository. With `--renderer` the bench
waits for the renderer's picture after each `advance` and times the two together, as a frontend showing that picture
each frame would pay; the wait is also printed by itself. The device is the RX 6800 (`EMUSEN_GPU_DEVICE=6800`).

### 18.2 Unattached, and the cost that was there at first

The first build of the branch stood above the base on `gpu/benchmark`, a hundred flat triangles a frame [M]:

| Run | Base | Branch | Difference |
|---|---|---|---|
| first series, five rounds | 2.312 | 2.370 | +2.5% |
| again, seven rounds | 2.301 | 2.365 | +2.8% |

The other four items were within −0.7 and +0.9 per cent. Comparing the links' symbol sizes (`nm -S`), as §9.2 did,
`raster::triangle` was 319 bytes longer (0x19B7 against 0x1878); the other GPU functions that grew (`execute` by 51
bytes, `feed` 74, `Display::read_line` 115, the last for the line's test of the layer) are each run once a command or
a line, not a pixel. The prediction that the triangle's setup, factored into `raster::setup` and called from
`triangle` so that the renderer would share it, was the cost was tested by ablation: the branch with the triangle's old
body and `setup` kept for the renderer stood at the base (2.304 against 2.317) [M]; the factored form with `#[inline]`
in place of `#[inline(always)]` did not (2.380, and a function of 0x1901 bytes). **Decided [C]**: the software
triangle keeps the base's code, byte for byte in its function's size (0x1878), and the renderer's `setup` makes the same
values; the two are held equal by `setup_describes_the_pixels_triangle_draws`, which draws 600 random triangles,
shaded and then textured raw from a page whose texel names its coordinate, and requires each pixel of each to be what
`setup` describes under the renderer's per-pixel rule (§16.3). A mutant of `setup` alone, its winding taken as given,
fails it. This is the one place where the GPU and the renderer state a rule twice; the test is what makes it one rule.

The final build against the base, five rounds [M]:

| Run | Frames | Base | Branch | Difference |
|---|---|---|---|---|
| `gpu/benchmark` | 1,200 | 2.309 | 2.308 | 0.0% |
| `gpu/bandwidth` | 1,200 | 2.935 | 2.936 | 0.0% |
| Crash Bandicoot | 3,000 | 2.294 | 2.286 | −0.3% |
| Mega Man Legends | 3,000 | 1.793 | 1.788 | −0.3% |
| Tekken 3 | 3,000 | 2.368 | 2.365 | −0.1% |

Every pair's digests equal. **P-E3d held**: unattached, the frame is the base's within the noise.

### 18.3 Attached

The same runs with the renderer attached and its picture waited for every frame [M, five rounds; the wait the mean of
three further runs; "the frame's form" the difference made by a scratch build whose frame stays plain with a layer
attached, five rounds, below]:

| Run | Base | Attached | Difference | The picture's wait | The frame's form | The renderer's host work |
|---|---|---|---|---|---|---|
| `gpu/benchmark` | 2.309 | 2.624 | +0.315 (+13.6%) | 0.223 | 0.040 | 0.055 |
| `gpu/bandwidth` | 2.935 | 3.504 | +0.569 (+19.4%) | 0.284 | 0.139 | 0.148 |
| Crash Bandicoot | 2.294 | 2.662 | +0.368 (+16.0%) | 0.190 | 0.089 | 0.077 |
| Mega Man Legends | 1.793 | 2.080 | +0.287 (+16.0%) | 0.147 | 0.069 | 0.066 |
| Tekken 3 | 2.368 | 2.757 | +0.389 (+16.4%) | 0.130 | 0.149 | 0.134 |

Every frame's two pictures equal in every run. The three shares, each on the emulation thread:

- **The picture's wait**: the submission (the staged arrays copied to the device, every stretch's dispatches) recorded,
  run and waited for, and the picture read back. Its floor, measured apart on the same device [M, a scratch program,
  2,000 submissions each]: a waited submission holding nothing, 0.036 ms (median); one copying 1.2 MB to the host,
  0.111 ms. So on Crash Bandicoot's 512 by 240 picture roughly half of the 0.19 ms is the round trip and the readback,
  and the device's own drawing about a tenth of a millisecond [A, by subtraction].
- **The frame's form**: E1's layered frame, chosen whenever a layer is attached, tells the layer every move of the
  processor and every projection, and the renderer ignores them (its `moved` is the empty default, called through
  `dyn Layer`). A scratch build with the frame kept plain while the renderer was attached measured this share [M]:
  2.587, 3.367, 2.561, 2.006 and 2.632 against the table's attached column. It is the same order as E1's test
  observer's dispatch (§9.3).
- **The renderer's host work**: the plain-form build's attached frame less the base and the wait: the GPU's events,
  decoding, records, binning and stretches, 0.05 to 0.15 ms a frame. (The frame's form is taken from the plain-form
  build's own series against the attached build in the same rotation, 2.627, 3.506, 2.650, 2.075 and 2.781, within
  0.02 of the table's attached column.)

**P-E3e held in its first part and in most of its second**: under a millisecond a frame more on every item (0.29 to
0.57); the decoding and binning a sixth to a quarter of it on four items, under the third predicted, and on Tekken 3
just over (0.134 of 0.389, 34 per cent), where the stream holds the most commands and copies for the fewest pixels.
With the frame's form counted with them, a third to two thirds.

**What is not done, and could be** [A, proposed, not built]: a layer that watches only the GPU could say so (a method
of `Layer` with a default of true), and the machine keep the plain frame for it, which would save the middle column;
E4's precision layer needs the moves, so whether that is worth a method is E4's question. The wait could be taken off
the emulation thread by `emusen-gpu`'s pending submission (`Mars_Gpu.md` §14.2): the frame's work submitted at the
frame's end and waited for when the next picture is wanted, which Mars measured worth doing at its multiples; at 1×
on this device the device's share is a tenth of a millisecond, and the decision is E5's, where it grows by N².

### 18.4 On the handheld, estimated

**An estimate, not a measurement** [A]: the handheld is not part of this stage. Its GPU sits at its 800 MHz floor in
these loads (the project's record of its governor), the Ryzen Z1 Extreme's integrated Radeon, twelve compute units
against the RX 6800's sixty (the chips' published figures, not measured here) at 800 MHz against some 2 GHz, reading
the system's shared memory: some ten to fifteen times less compute throughput. Scaling the device's own share (about 0.1 ms on Crash Bandicoot,
0.05 to 0.15 across the items) by that gives **one to two milliseconds a frame for the device's drawing at 1×**.
An integrated device has no bus between the host's arrays and its own memory, so the copies and the readback shrink
towards a memory copy rather than grow. The host's shares (0.1 to 0.3 ms here) scale with the handheld's slower
processor, perhaps half again. In all, perhaps **+1.5 to 2.5 ms a frame** at 1× with the picture waited for, against
the 16.7 ms of a frame at full speed; with the wait made pending (§18.3) most of the device's share would leave the
emulation thread. E5's internal resolution multiplies the device's share by about N², which at 4× on the handheld would
not fit a frame without that change; it is E5's to measure.

## 19. Tests run, predictions, and what E4 needs

### 19.1 Tests [M, 2026-10-13]

- **`emusen-gpu`**, `cargo test --release`: 12 (one added, §3.1 of `EmuSen_Gpu.md`), on the RX 6800, and the added one
  clean under the validation layer with synchronization validation.
- **MarsRT**, `cargo test --release` on the RX 6800 alone, since `emusen-gpu` changed: 490 of 490.
- **`diamond`**, `cargo test --release` with `EMUSEN_DIAMOND_PS1TESTS` and `EMUSEN_GPU_DEVICE=6800`: 167 unit tests
  (two added: the seam's events, `a_layer_is_told_each_command_its_continuations_and_each_line_shown`, and
  `setup_describes_the_pixels_triangle_draws`), 81 program tests (E1's goldens among them, plain and with the test
  observer) and 3 renderer gates (`tests/renderer.rs`), all passing; and without the corpus. The disc gate and the disc
  goldens with `EMUSEN_DIAMOND_DISCS`: passing (§17).
- **`diamond-renderer`**, its own unit test (the blocks' and tiles' wrapping).
- **The renderer's gates under the validation layer** (`VK_LAYER_KHRONOS_validation`, `VK_LAYER_VALIDATE_SYNC=1`,
  `VK_LAYER_SYNCVAL_SHADER_ACCESSES_HEURISTIC=1`, `Mars_Gpu.md` §14.4's configuration): the programs' and ps1-tests'
  gates, no message.
- **On llvmpipe, for the record**: the programs' and ps1-tests' gates pass, in 6 s and 35 s (against 4 s and 25 s on
  the RX 6800). The integrated card was not run, by the project's rule for GPU tests.
- **WiseMan**, under the test lock with the corpus, filtered by `Diamond`, `MarsGpu` and `MarsRt`: 734 of 734, the new
  `DiamondShaderTests.Every_compiled_shader_of_the_renderer_is_the_compilation_of_its_source` among them.

### 19.2 Predictions

P-E3a, P-E3b, P-E3c, P-E3d and P-E3e held (§17.2, §18.2, §18.3). Retired with them: "the factored setup costs nothing
inlined", which was not written down beforehand and did not hold (§18.2). P9 of the plan (every accurate golden
reproduced with every enhancement off) holds for E3, and with the renderer attached too.

### 19.3 What E4 needs next

- **The composition crate.** v1's library is `diamond`'s cdylib and cannot link a layer crate (§14.5, §17.3). E4's
  setting needs the core's exports in a crate that links the machine, `precision` and `diamond-renderer`, and attaches
  and detaches them between frames; `Core::frame` then gives the renderer's picture with no change of the ABI (§17.3).
  The conformance kit gets the renderer through that setting.
- **Two layers, one attachment.** The machine attaches one layer. E4's precision layer and the renderer are two;
  either a layer that holds both and passes each event on, or the renderer becoming the consumer of the precision
  layer's side data by a method of its own. The side data reaches a command as E1's receipts queue it, and the
  renderer's records gain fractional vertices and a depth: `Command` carries the words, so the precision layer can find
  each vertex word's entry in its queue by order.
- **The records' arithmetic.** The walk's per-pixel rule takes fixed-point vertices with a fraction where E3's are
  integers; the edge functions and interpolants are restated in the same shader, and the 64-bit products already
  carried in pairs of words leave room for the extra bits. Perspective-correct texturing adds a division per pixel.
- **The frame's form** (§18.3): whether a GPU-only layer should keep the plain frame is E4's to decide with its own
  layer's needs.
- **The texture cache** stays unmodelled; E4's pictures should be graded, as E3's are, with it named as the one rule
  that differs, and `texture-cache.s` and `triangle-rows.s` its two pinned counts.

## 20. Reading log (E3)

No emulator source of any kind was opened, and no other PlayStation renderer: nothing of DuckStation's or Beetle PSX's
hardware renderers, parallel-psx, PCSX's or PCSX-Redux's, SwanStation's, ePSXe's plugins or any PGXP code, nor any
shader of theirs. No firmware, listing or trace of firmware; nothing under `kernel/` or `kernel-harness/` was opened or
changed, nor the player's BIOS folders. psx-spx was not reopened: what the GPU does came from Diamond's own software
rasteriser and its pages. Opened: `Diamond_Plan.md` §1, §5.4, §6, §7 and §11 Q14; this page whole; `Diamond_GPU.md`
whole; `Diamond_Speed.md` §2; `Shared/EmuSen_Gpu.md` whole; `Mars_Gpu.md` §1, §2, §5, §13 to §15; the source of
`diamond`'s GPU (`gpu/`), `layer.rs`, `bus.rs`, `machine.rs`, `state.rs`, `debugger.rs`, `v1.rs` at its exports,
`tests/programs.rs` and its support, `examples/run.rs` and `bench.rs`; `emusen-gpu`'s source and tests; MarsRT's
`rdp/gpu/mod.rs` at its dispatches; `emusen-core-conform`'s command line; WiseMan's `ShaderCompiler.cs`,
`MarsGpuDeviceTests.cs` and `DiamondTests.cs`; E1's measurement scripts in the probe's `e1/`; the press scripts by way
of the goldens' code. The validation layer's messages were read as its output. The Vulkan specification was not
opened; the rule of §14.3 was shown by the validation layer, which quotes it.

---

## 21. Stage E4: precise geometry, decided before it was built

*Written 2026-10-13, before any of E4's code, after the tester's go of the same day.* The plan's row: "The precision
side-channel; sub-pixel vertices, perspective-correct texturing and the seam fixes in the renderer. Gate: a synthetic
program that rotates a textured mesh slowly: vertex jitter and texture warp measured with and without; on the bench
discs, the fraction of vertices resolved precisely, recorded per game; E1's goldens unchanged." Decided in §11 Q14 of
the plan: the setting is **"Precise geometry"**, the layer's crate **`precision`**. The tester added for this step that
the setting reach players through the core ABI with the renderer's picture, that it say so honestly where no Vulkan
device is usable, and that the machine's own goldens hold with it on. Internal resolution, true colour and filtering are
E5's and are not touched.

### 21.1 What the console loses, and what may be given back

The GTE projects a vertex in fixed point: the vertex rotated and translated into the camera's space with 12 fraction
bits (`sums`, §3.1), IR1 and IR2 cut from it to whole units, SZ3 to a whole depth, the quotient H/SZ3 from a table with
16 fraction bits, and SX2 and SY2 from MAC0's sum `OFX + IR1·q` shifted right 16 and saturated to −400h..3FFh
(`BlackMoon_GTE.md` §5.1) [R]. The GPU takes each vertex as two 11-bit integers (`Diamond_GPU.md` §4) and draws by
integer edge functions with affine texture coordinates (§14.1). Three losses follow, and each shows as movement:

1. **The screen position is cut to whole pixels** by MAC0's shift: a vertex moving smoothly snaps from pixel to pixel,
   up to a whole pixel off where the geometry puts it. Neighbouring vertices snap independently, so a slowly turning
   model wobbles.
2. **The camera-space position is cut to whole units** before the division (IR1, IR2 from MAC1, MAC2 by sf's shift) and
   the depth to a whole unit before the quotient (SZ3): up to H/SZ3 pixels for the first, and up to the vertex's distance
   from the centre over SZ3 for the second. On a near ground plane with H about 300 and SZ3 about 200 either is over a
   pixel [A, from the formula].
3. **Texture coordinates are interpolated linearly in screen space** where the geometry is in perspective: a quad
   receding from the camera has its texture bent along its diagonal, and the bend moves as the quad turns.

The first two are what "sub-pixel vertex precision" gives back, the third "perspective-correct texturing" (the plan's
§6.3). Both need values the console computed or could have computed at the projection and then discarded, carried to the
moment the GPU draws the vertex, with nothing in the machine depending on them.

### 21.2 The precise vertex: the projection evaluated without its truncations [C]

**Decided: the precise screen position is `OFX/2¹⁶ + H·sums₁/sums₃`** (and `OFY/2¹⁶ + H·sums₂/sums₃` for y), in double
precision, from the camera-space sums the GTE made before any shift, and **the precise depth is `sums₃`**, the camera's
depth with its 12 fraction bits. This is the perspective projection the GTE's RTPS approximates, applied to the very
vertex the program rotated, so it removes all three of §21.1's losses at once.

Considered and not chosen: **MAC0's sums** `x` and `y` with their 16 fraction bits, the position as the console computed
it before the last shift. Those remove the first loss alone and agree with the console's integer to within the pixel by
construction; but they keep the second, which on near geometry is as large as the first. The decision is recorded as
argued here and **measured on the synthetic mesh** (§23): the mesh's jitter is computed against the ideal projection for
the integer vertex, for MAC0's sums and for the chosen form, so that the choice stands on numbers.

The projection needs H, OFX and OFY, which the observer was not told at E1 (§3.1); `blackmoon-gte`'s `Projection` gains
two fields, `h` and `offset`, the control registers as the command read them, an addition to the struct the crate owns.

**The guard.** A precise position is used only where it lies near the console's: within **two pixels of the centre of
the pixel the console chose** in each axis, with `sums₃` positive and MAC0's sums unsaturated. Where the console's own
value is far from the geometry, something cut it that is not a rounding: IR saturated (a vertex far off to the side, or
`lm` clamping a negative IR to 0), the quotient saturated at 1FFFFh for a vertex nearer than H/2, SX or SY saturated at
the GPU's range, or a vertex behind the camera. In each the console drew the vertex somewhere the geometry does not
say, and a precise vertex in its place would move the polygon by many pixels; the guard keeps the console's integer
there. Two pixels is chosen to admit §21.1's second loss on near geometry, which reaches a pixel or two, while refusing
saturation, which moves a vertex tens to hundreds of pixels [A]; the gate counts the vertices the guard refuses on every
disc (§23), and the bound is to be revised if the count says so.

**What is carried.** For each projection: the integer word SXY2 as pushed, the offset of the precise position from that
integer's pixel in 256ths of a pixel (two `i16`), and the depth (`i32`, 12 fraction bits). 256ths are the renderer's
sub-pixel grid (§22.2), whose products stay within 64 bits; finer bits would be rounded away there.

### 21.3 The shadow, and which moves carry a precise value [C]

The design is §3.4's, made concrete.

- **A ring of the last 32,768 projections.** Each projection takes the next serial number (32 bits, never 0) and writes
  its entry at the serial's place in the ring: the serial, the integer word, the two offsets and the depth, 16 bytes,
  512 KiB in all. A refused projection (§21.2) takes no entry: the FIFO's shadow gets 0.
- **Every place that can hold a projected word holds a serial**, or 0 for none: the GTE's three SXY entries, the 32
  general registers, every word of RAM (524,288, 2 MiB) and of the scratchpad (256). A serial is found in the ring by its
  place and is live only while the ring's entry there still has that serial; one 32,768 projections old has been written
  over and is dead.
- **Moves copy serials, and nothing else.** A projection pushes the FIFO's shadow; MFC2 from SXY0–SXY2 (SXYP reading
  SXY2) gives a register its slot's serial; MTC2 and LWC2 to SXY0–SXY2 set the slot, and to SXYP push it, with the
  register's or the memory word's serial; SWC2 gives a memory word the slot's serial; a word-sized load gives a register
  the memory word's serial, a word-sized store the memory word the register's, a copy (§3.2) one register another's.
- **What ends it.** A byte or halfword load gives its register 0; a byte, halfword or partial store gives its word 0; a
  load from anything but RAM and the scratchpad gives 0. Arithmetic on a register is not told (§3.2), so the register's
  serial stays while its value changes; that is not checked at the move but at the end:
- **The integer check, once, at the receipt.** When GP0 receives a vertex word, its serial is taken (from the RAM word
  DMA read it from, or from the register or GTE slot the store named, below), and the precise vertex is used only if the
  serial is live **and the ring's integer word equals the word received**. A register changed by arithmetic, a word
  changed by another DMA channel, by the CD, by the MDEC's output or by a turn the idle skipper passed (§2.4) no longer
  holds the integer and falls back to the console's vertex. Checking once at the use rather than at every move costs
  one ring access a vertex instead of one a store, and loses nothing: a value that left the integer and came back to it
  through untold arithmetic is the same pixel, and its precise offset is the one its projection made.

**Bounded in memory and time, and nothing allocated on the frame's path.** The ring, the RAM and scratchpad shadows and
the registers' are allocated once when the layer is made (2.5 MiB); a projection writes one ring entry, a move one word.
An entry lives until 32,768 later projections (about 27 frames of Spyro the Dragon, the bench's heaviest at about 1,235
projections a frame [M, §7.2's counts]); a static scene re-sent without new projections keeps its precision, since
nothing advances the ring. A ring 64 times smaller than RAM's shadow cannot leave a stale serial live by accident: a
dead serial is recognised by the ring's entry, never by an age.

### 21.4 The GPU's receipt of a vertex word, matched [C]

- **By DMA**, the receipt names the RAM address channel 2 read (§3.3): the serial is that word's.
- **By the processor**, the receipt was told **before** the store that sent it (§3.3), and the GPU may run the command
  inside the store, before the store's move is told, since a word that finds the FIFO empty and the GPU idle is fed at
  once (`Gpu::gp0`). The layer would learn which register the word came from only after the command had drawn.
  **Decided: the R3000A tells a store before it writes**, so that a device the store reaches finds the layer already
  told. The store's value is known before the write; whether the write meets a bus error is not, so a store that faults
  is told too. A layer cannot be misled by it: the word was not written, and the integer check at the use rejects a
  serial the memory word does not hold. The alternative, matching a CPU receipt by its integer against every register's
  shadow, is ambiguous where two registers hold one pixel with different precision, and was not chosen. Loads stay
  told after the read, since their value is the read's. The change is to `blackmoon-r3000`'s report order under
  `if B::MOVES`, so the plain bus's code is unchanged; the GP0 store then records the register's serial (or for SWC2
  the GTE slot's) as the serial of the next receipt from the processor.
- **The receipt queue.** Each receipt's word and serial enter a ring of 64; a command's words are found there when the
  GPU starts it (`executed`, §15.2), as the run of receipts after the last one used whose words are the command's. GP0's
  words are executed in the order received; the receipts no command uses are the words the GPU drops (no-ops between
  commands, a polyline's end marker, a word lost to a full FIFO behind an unread transfer), which the search steps over.
  The GPU's FIFO holds 16 words and a command at most 12 more, well inside 64.
- **Whole polygons only** [C, the plan's rule]. A polygon's vertex words are found by the command's layout (the same as
  `decode::polygon`'s); the precise vertices are given to the renderer only when **every** vertex of the polygon is
  precise, a quad's four together. A polygon with one integer vertex is drawn wholly as before, so no polygon is
  interpolated half precisely; the seam that can remain is between polygons, where a precise one meets an integer one,
  and is at most §21.2's two pixels and in practice under one.
- **Rectangles and lines stay integer.** A rectangle is placed and stepped pixel by pixel from one corner, with no
  geometry between pixels to recover; a line's rule steps its major axis. Both are drawn as the console draws them
  (`Diamond_Plan.md` §6.3's "2D overlays fall back"), whatever their words' history.

**The offset and D-47.** The precise vertex is carried as an offset from the integer the console used, never as an
absolute position. The GPU then adds the drawing offset in 11 bits (D-23), places each vertex from the first by its
11-bit distance, and folds a primitive past −1,024 by 2,048 (D-47): every one of those is a whole-pixel move of the
integer, which the offset rides unchanged. The renderer applies them to the integer by the GPU's own `decode` and
`raster::fold`, and adds the offset after. Nothing about them is restated.

### 21.5 What the frame costs, and the budget

Every move the processor reports becomes one or two word writes, a projection one entry of 16 bytes and a division,
a receipt one queue entry, a polygon's vertices one ring read each. §9.3's test observer, doing additions where this
layer does stores, cost 2.6 to 3.5 ns an event; P-E1b predicted at most twice that. The RAM shadow is 2 MiB, beyond the
level-2 cache, so a load or store to RAM touches a line the processor's own RAM access touched a moment before in a
different array [A]. Measured in §25.

## 22. The renderer's use of precise vertices [C]

### 22.1 Which path a polygon takes

A polygon the precision layer gave whole precise vertices for is drawn as **one or two precise triangles**: a record of
a new kind, `PRECISE` (6), laid out within the 48 words of §16.2, read by the same walk. Every other polygon, and every
rectangle, line, fill, copy and transfer, is the record E3 made. Untextured, flat and 2D polygons with no precise vertex
are therefore drawn exactly as the accurate picture; the zero-fraction gate (§23) holds that the precise record itself
is the integer rule when its offsets are zero.

### 22.2 Sub-pixel rasterisation that is the integer rule at zero fraction

- **Positions** in 256ths of a pixel: `256·x + dx`. The checks that refuse a primitive (§14.1: a span past 1,023 or 511)
  are made on the integer vertices, as the console makes them.
- **Coverage.** A pixel is its integer point; it is inside when each edge function `dX·(256y − aY) − dY·(256x − aX)` is
  at least 0 on an edge the triangle owns and 1 otherwise, the ownership by the edge's direction as `raster::owns`. At
  zero fraction every term is 2¹⁶ times the integer one, and "at least 1" in units of 2⁻¹⁶ is "at least 1" in whole
  units for an integer, so the set of pixels is the integer rule's exactly. The bounds are `⌈min x⌉..⌊max x⌋` and
  `⌈min y⌉..⌈max y⌉ − 1`, the integer rule's at zero fraction. The products reach 2³⁷ and are made exact in two 32-bit
  words, as E3's interpolants are (§16.3).
- **Interpolants.** The base vertex is the integer rule's (`raster::base_vertex` on the integer vertices); each
  gradient is `((Δa₁·ΔY₂ − Δa₂·ΔY₁) << 20) / area` truncated toward zero, with the positions and the area in the fine
  units, which at zero fraction is 2¹⁶ over 2¹⁶ of the integer rule's and so the same quotient; a pixel's value is
  `(256·base + gx·(256x − bX) + gy·(256y − bY) + 256·½) >> 20`, the integer rule's `(base + gx·Δx + gy·Δy + ½) >> 12`
  at zero fraction. A gradient that does not fit 32 bits (a sliver of under a thousandth of a pixel's width) cannot be
  stored in the record; such a polygon is drawn wholly on the integer path and counted.
- **Colour stays affine**, as the console's Gouraud shading is; only the texture is corrected.

### 22.3 Perspective-correct texturing: a division per pixel

The texture coordinate of a pixel is `Σ Eᵢ·qᵢ·uᵢ / Σ Eᵢ·qᵢ`, where Eᵢ are the pixel's three edge functions (each the
weight of the vertex opposite it, already computed for its coverage) and `qᵢ = z_min/zᵢ` from the precise depths; the
same for v. That is the perspective-correct interpolation of u and v, since u/z and 1/z are affine in screen space; the
pixel's coordinate is the nearest integer, as the integer rule's half and shift take it, held within the vertices' own
range so that a rounding never wraps a coordinate past 255. **A division per pixel, not per span** [C]: the walk is one
invocation a pixel with no spans (§16.3), so a span's division would need the very per-pixel interpolation it saves, and
on the device two floating divisions a pixel are a small part of a walk that already does three 64-bit edge functions
and the texel's VRAM read [A]; its cost is measured (§25). A triangle whose three depths are equal is in a plane facing
the screen, where affine is exactly perspective-correct; it takes the affine interpolants of §22.2, so a screen-aligned
quad's texture is the integer rule's at zero fraction, with no rounding of a division in it.

### 22.4 Seams and gaps

Neighbouring precise polygons share their vertices' entries exactly: one projection, one serial, one offset, so the
shared edge's two functions are exact negatives in integer arithmetic and the ownership rule gives each pixel on it to
exactly one of them, with no gap and no pixel drawn twice [A, from the rule; the mesh is the test]. **No edge expansion
is built** [C]: the plan's §6.3 names its cost (semi-transparent polygons blended twice where they meet, sprite edges
softened), the precise shared vertices close the gaps it was for, and what is left is the seam of §21.4 between a
precise polygon and an integer one, which expansion would hide only by drawing both over each other. *Closed since,
without expansion: the crack fill, §33.*

### 22.5 What the renderer keeps, now that its picture can differ

At E3 the renderer's mirror was the machine's VRAM at 1×, so it kept nothing across a state (§16.6). With precise
polygons its mirror holds what it drew, which the machine's VRAM does not: the frame being displayed, and anything a
game copies or textures from what it drew. **It keeps a block when it draws precisely** (§24.4), and none when attached
alone, as at E3, so E3's gate and its goldens are unchanged.

## 23. The gate, as it will be run

- **Off is the accurate path.** E1's three goldens with the composition built and the setting off; and **with it on**,
  the precision layer and the renderer attached from power-on, the machine's digests (state, sound, TTY, VRAM, and its
  own picture, which the accurate GPU keeps) bit for bit, as E1's test observer was held.
- **At zero fraction the renderer is E3's.** The precision layer in a test form that gives every precise vertex an
  offset of zero and one depth: every polygon it resolves goes through the `PRECISE` record, and the pictures and VRAM
  of the programs, ps1-tests and the eight discs must equal the software rasteriser's as E3's did, the texture cache's
  two pinned counts the one difference.
- **A synthetic mesh** (`tests/precise.rs`): a textured grid tilted away from the camera and turned slowly through
  RTPT and RTPS, its packets in RAM sent by DMA's list, its texture's every texel naming its own coordinate. For each
  frame: each vertex's screen position as drawn, against the ideal projection of the grid as rotated, for the integer,
  for MAC0's sums and for the precise vertex (jitter: the error and its change from one frame to the next); and each
  pixel's texel, read back from the software VRAM and from the renderer's, against a perspective-correct reference
  computed in the test from the ideal geometry (warp).
- **A 2D program**: sprites, rectangles, lines, flat and shaded polygons the processor builds, and a menu of quads whose
  vertices are projected by the GTE and then moved by the processor, one quad with one vertex moved: with the setting
  on its picture and VRAM must equal the software's to the pixel.
- **On the eight bench discs**, along their golden anchors with their press scripts: the fraction of polygon vertices
  resolved precisely and of polygons drawn precisely, by game (P10); the guard's refusals and the ring's dead serials;
  the machine's goldens with the setting on; pictures looked at, every visible defect listed with its frame. No picture
  of a game is committed.
- **The player's path**: the kit on the published library, off and on, the eight discs at 1,800 frames; WiseMan's
  Diamond tests; the setting in the PlayStation's Enhancements group with its cost; turning it on and off between
  frames through the host changes only the picture.

## 24. Composition, the setting and the state [C]

### 24.1 The crates

| Crate | Its one job | Depends on |
|---|---|---|
| `diamond` (the machine) | the console, exact; the layer's frame; `impl Core for Machine` | as before |
| `precision` | the shadow, the receipts, the precise vertices of each polygon; a layer | `diamond` |
| `diamond-renderer` | the picture drawn on the device from the command stream, with precise polygons where given | `diamond`, `precision` (for the vertices' type and the drawer's trait), `emusen-gpu` |
| `diamond-v1` (new, `v1/`) | the core ABI's exports: the machine with its enhancements, as the host sees it | the three |

- **The machine's library is no longer the published one.** `diamond`'s crate type becomes `rlib` alone and its
  `core_exports!` moves out: its `impl Core for Machine` stays, with every descriptor and the tests that hold them.
  `diamond-v1`'s library is named **`diamond`** (its package `diamond-v1`, its dependency on the machine renamed
  `machine`), so the built file is `libdiamond.so`, its sidecar `libdiamond.so.core.json`, and its exports the machine's
  through a `Core` that holds a `Machine` and passes each call on, adding the setting and the picture. Rust allows two
  crates of one name in a graph when their metadata differ, as two versions of a crate are [M, a scratch pair built and
  linked]. `RustCores.props`'s row for `diamond` changes its `Dir` alone, to `v1\`; the library's name, its core id,
  its system pack and everything Mistress reads stay as they are.
- **Why `precision` defines the drawer's side.** The renderer must be told a polygon's precise vertices with the
  command, and the precision layer must be told every event first. A layer that holds both and passes each event on
  is the one attachment point (§15.1 kept: the machine attaches one layer). It lives in `precision` as
  `Precise<D: Draw>`, generic over the drawer: `precision` defines what it gives (`Draw::executed_with`, a polygon's
  vertices whole or none) and knows nothing of Vulkan; the renderer implements `Draw` and knows nothing of moves. The
  other arrangements were a pair in the machine's `layer` module, which would give the machine a type for data it never
  reads, and the pair in `diamond-v1`, which the machine's own tests could not then attach, since a crate named
  `diamond` cannot be a dependency of the crate named `diamond` that holds the goldens.
- **The plain frame for a GPU-only layer is not built** [C]. The precision layer needs every move, so with the setting
  on the layered frame is the right one. A renderer attached alone, which would want the plain frame (E3 measured that
  share at 0.04 to 0.15 ms, §18.3), happens only in the tools and the gates until E5 offers internal resolution without
  precise geometry; whether to add `Layer::moves` is E5's to decide with that setting.

### 24.2 The setting

`precision`, "Precise geometry", an `Enhancement` switch in the Enhancements category: `effect: enhancement`, default
false (the hardware's), `scope: run`, no restart, its help and cost in a player's words (the cost: a Vulkan graphics
card's time each frame, save states about twice their size, and the hairline seams and the 2D overlays of §21.4). It is
applied by `set_settings` between frames, and read at create like every key:

- **On**: a renderer on the device `EMUSEN_GPU_DEVICE` names (the first usable one without it), kept in the block form
  (§22.5), paired with a new precision layer and attached; the frame given to the host is from then the renderer's
  picture, read back after each frame's `advance` unless the host skips rendering. Attaching seeds the renderer from
  the GPU as it stands (§16.6), so the picture does not flash.
- **Off**: the layer detached and dropped; the frame is the machine's own again.
- **Where no usable Vulkan device exists, or the renderer cannot be made** [C]: the setting is **accepted and
  reported**, not refused. Refusing would leave a stored value the frontend could not apply on every load, and a host
  that applies a player's saved settings at create would fail the game's load over a picture setting. Accepted, the
  machine runs exactly as off; `setting_notes` (`SETTING_NOTES`, `EmuSen_CoreAPI.md` §6.13) gives the sentence "No
  usable Vulkan graphics card was found (the reason), so the console's own picture is shown," which Mistress already
  shows under the row (`CoreEngine.Settings`' `Note`). Nothing crashes: `Renderer::new` returns the reason as an error,
  and the layer is never attached.
- **The API's events**: `core_exports!` raises `GEOMETRY` when `frame_info`'s size changes and `STATE_SIZE` when the
  state's does, after `set_settings`, `advance` and `state_load`, so turning the setting on raises `STATE_SIZE` (its
  blocks, §24.4) and raises `GEOMETRY` only if the sizes differed, which at 1× they never do (the renderer's picture is
  the display's size).
- **The frontend's group.** Mistress's Graphics Settings window lists a core's sidecar settings in the console's tab
  (§5); the schema's text asks for enhancements in a group of their own. Built now, as E1 left it for the first one
  offered: a core setting carries its group, `Enhancements` for `effect: enhancement`, and the console's tab shows a
  heading before the first row of each group other than the core's plain settings.

### 24.3 The kit, the run tool and the bench

The kit needs no option: it reaches the renderer through the setting, `--settings precision=true`, on the published
library, which is what E3 could not do. The run tool gains `--precise` (the pair attached from power-on, its pictures
written as `--renderer`'s are, and the precision layer's counts printed) and `--precision` (the precision layer alone,
for its counts, with no device); the bench gains `--precise`, timed as `--renderer` is.

### 24.4 State

- **`PREC`, the precision layer's block, fixed in size**: the next serial, the ring (512 KiB), the shadows of RAM
  (2 MiB), the scratchpad, the registers and the FIFO, the receipt queue with its cursor, and the serial of the next
  receipt from the processor; 2.6 MiB. Its counts are not kept: they describe a run, not the machine.
- **`RNDR`, the renderer's block, written only in the block form**: its mirror of VRAM a halfword a pixel (1 MiB), its
  picture at the largest size with its width and height (1.4 MiB), and the palette the CLUT cache holds as the renderer
  has it; read back from the device when the host settles the machine before a save (`Core::settle`, which the export
  calls), loaded into the mirror in place of the GPU's VRAM when a state is loaded. 2.5 MiB.
- **Why whole and fixed** [A]. The kit's C7 and C8 compare the pictures of a machine that loaded a state with those of
  the machine that saved it, and a state's size may not change from frame to frame. A frame drawn after a load must
  then be the frame the machine that saved it draws: the vertices projected before the save and drawn after (a game's
  double-buffered ordering table), the receipts in the GPU's FIFO at the save, and what the renderer drew that VRAM does
  not hold all reach the picture after the load. A block holding only what is live would vary in size. So the state
  with the setting on is about 10.4 MB against 5.3, and the cost says so.
- *Superseded 2026-10-14 (§52): neither block is written since; a load seeds both parts.*
- **Both ways** (E1's rule): a state made with the setting on loads with it off, both blocks skipped; one made with it
  off loads with it on, the precision layer starting empty and the renderer seeded from the GPU's VRAM, so the first
  frame or two after are drawn with fewer precise vertices; the machine's continuation is the same either way, since
  nothing in the machine reads a layer. The Layer trait's block methods are made per tag, so that one layer, the pair,
  can write the two blocks its two parts own: `tags`, and `save`, `load` and `clear` given the tag.

### 24.5 Predictions [recorded before the code]

- **P-E4a.** The mesh's vertex jitter, the mean screen-position error against the ideal projection, falls from about a
  third of a pixel with integer vertices to under a hundredth with precise ones; with MAC0's sums alone it falls only
  to between a tenth and a quarter.
- **P-E4b.** The mesh's texture warp, the mean texel error against the perspective-correct reference, is over a texel
  with the setting off and under half a texel with it on.
- **P-E4c.** On Crash Bandicoot, Spyro the Dragon and Ape Escape most polygon vertices drawn in play are precise
  (over 70 per cent); Castlevania and Rayman, which draw mostly 2D, under 10 per cent (P10's second half).
- **P-E4d.** The guard refuses under 1 per cent of projections on every disc.
- **P-E4e.** Every machine golden holds with the setting on, by construction.
- **P-E4f.** Off, the frame costs the base within the bench's noise; on, the emulation thread pays under 1.5 ms a frame
  more than the plain frame on the three discs on the RX 6800, the precision layer's share under 0.3 ms (P-E1b's twice
  the test observer).

## 25. E4's gate, as run

*Measured 2026-10-13 on the desktop, the RX 6800 by `EMUSEN_GPU_DEVICE=6800`.* What was built is §21 to §24 as
designed, with two changes the discs forced (§26.2, §26.3) and one counter split (the guard's refusals, §25.4). The
crates: `precision` (`src/lib.rs`, the shadow, the receipts, `Precise` and `Draw`; six unit tests), the renderer's
`PRECISE` record (`record.rs`'s `precise_triangle`, `walk.comp`'s `fine_triangle`), its block and its `Draw`, and
`diamond-v1` (`v1/src/lib.rs`, the `Core` that holds the machine; four unit tests). The gate is `tests/precise.rs`,
eight tests on the machine crate, beside E1's `programs.rs` and E3's `renderer.rs`.

### 25.1 Off is the accurate path, and on the machine is unchanged

| Item | Result |
|---|---|
| E1's goldens with every enhancement compiled in and off | **Equal** [M]: `programs.rs`' three golden tests, plain and with the test observer, 49 + 48 lines and the eight discs' 80 anchors, after every change of this stage's (the processor's store order, the GTE's two fields, the layer's blocks per tag) |
| The machine's goldens with the setting on, the pair attached from power-on | **Equal** [M]: 49 of 49, 48 of 48, 8 discs and 80 anchors, picture, sound, state, VRAM and TTY; the state's digest taken over the accurate blocks, before the layer's (`goldens.rs`' `state_digest`) |
| The same through the host | **Equal** [M]: `DiamondPreciseGeometryTests.Turning_precise_geometry_on_and_off_between_frames_changes_only_the_picture`, the setting turned on at frame 3 and off at 9 through `CoreEngine.Set`, the sound and the accurate state equal to a run without it at every frame, the picture differing at frames 3 to 8 alone |

P-E4e held, by construction and by measurement. P9 held for E4.

### 25.2 At zero fraction the precise record is the integer rule

The precision layer's zero form gives every precise vertex an offset of zero and one depth, so every polygon it
resolves is drawn through the `PRECISE` record and the walk's `fine_triangle`, its 64-bit edge functions, its fine
gradients and interpolants, its texel hold (§26.3) at its zero-fraction value, and none of it through E3's triangle
[M]:

| Runs | Frames compared | Precise triangles drawn | Pictures | VRAM |
|---|---|---|---|---|
| The project's programs, 49, 60 frames | 2,940 | none: no program's projections reach GP0 as polygons | equal | equal but the texture cache's two pinned counts (§17.2) |
| ps1-tests' release, 48 | 22,824 | none: `animated-triangle`'s 900 projections are made with the quotient saturated (SZ3 103 or 0 under H 320), which the guard cuts | equal | equal |
| The eight bench discs to frame 3,000 by their press scripts | 24,000 | 1,438,000: Crash Bandicoot 423,326, Mega Man Legends 282,808, Spyro the Dragon 307,276, Ape Escape 287,102, Silent Hill 137,186, Rayman 4,210 | **equal at every frame** | **equal at every anchor** |

So at zero fraction E3's equality holds through the new record on a million and a half of the discs' own triangles,
and the exactness argument of §22.2 is a measurement. The programs and ps1-tests test the other half, that with the
pair attached nothing else moves.

### 25.3 The turning grid: jitter and warp

`the_turning_grid_shows_less_jitter_and_warp_with_the_setting_on`: a grid of 4 by 4 textured quads, 800 units square,
tilted one radian about x and turned 0.004 radians (0.23°) a frame about y, 1,300 units from the camera, H 280; its
vertices and every frame's rotation in fixed point put in RAM by the test, its program projecting each quad's corners by
RTPT and RTPS into packets in a DMA list, double-buffered; the texture every texel of which names its own coordinate.
120 frames, 25 vertices a frame, about 19,100 textured pixels a frame graded [M]:

| Vertices | Mean error against the ideal projection | Mean change of the error from frame to frame (the wobble) | Worst error |
|---|---|---|---|
| Integer, as the console | 0.889 px | 0.467 px | 1.736 px |
| MAC0's sums (§21.2's alternative) | 0.159 px | 0.117 px | 0.409 px |
| **Precise** (the projection without its truncations) | **0.008 px** | **0.011 px** | **0.031 px** |

| Texture | Mean texel error against the perspective-correct reference, \|Δu\| + \|Δv\| | Pixels more than a texel off |
|---|---|---|
| Setting off: the software rasteriser, affine on integer vertices | 2.75 texels | 99.4% |
| **Setting on**: the renderer, in perspective on precise vertices | **0.50 texels** | **0.02%** |

The precise vertex's residual, 0.008 px, is the rotation's own fixed point (1.3.12, about 2⁻¹³ of 700 units) and the
256ths grid; MAC0's sums keep §21.1's second loss, a fifth of the console's error, which decided §21.2 on numbers. The
texture's 0.50 is the floor of the measure: each axis rounded to a whole texel errs by a quarter of a texel on average.

### 25.4 The 2D program, the strips and the state

- **The 2D program** (`a_2d_program_is_drawn_as_the_software_rasteriser_with_the_setting_on`): sprites, a variable and
  two fixed rectangles textured and flat, a line, a shaded line, a polyline, flat and shaded triangles and a textured
  quad the processor builds, and two quads of GTE-projected corners, one with every corner moved a pixel on the
  processor and one with one corner moved: with the pair attached, ten frames' pictures and VRAM equal to the software
  rasteriser's to the pixel [M]. The precision layer counted 60 projections, 50 polygons, none precise, 50 vertex words
  changed and 10 polygons mixed, each drawn wholly as before.
- **The strips** (`a_strip_of_a_picture_shows_no_texel_past_its_edge`, §26.3's defect held): four textured strips at
  one depth projected to fractional corners and sent by SWC2 straight to GP0; no pixel shows a texel past its strip
  (130 did without the hold), and as many pixels are drawn as the software rasteriser draws, 16,705.
- **The state** (`a_state_with_the_setting_on_continues_its_picture_and_loads_both_ways`, on the grid): a state saved
  at frame 40 with the pair attached, loaded on a second machine with it, continues the picture frame for frame for 20
  frames; saved, loaded and saved again it is the same bytes; loaded with the setting off, the machine continues as the
  first; a state made with it off loads with it on, the machine the same and the polygons precise again within two
  frames. With the setting on the state is 10,545,574 bytes against 5,398,770 for the kit's machine without it [M, the
  kit's C7 note]. `diamond-v1`'s `a_state_with_the_setting_on_loads_with_it_off_and_the_reverse` holds the same through
  the `Core`.

### 25.5 The bench discs: how much is precise (P10)

The pair attached from power-on, each disc by its press script to frame 3,000, counted by the precision layer [M]. A
projection is *cut* where the console cut it (saturated, at or behind the camera) and *far* where the guard refused a
position more than two pixels from the console's; a vertex is a polygon vertex word GP0 drew; *changed* is a vertex
word whose serial was live but whose integer the processor had changed.

| Disc | Projections | Cut | Far | Polygons | Precise | Mixed | Vertices | Precise | Changed |
|---|---|---|---|---|---|---|---|---|---|
| Crash Bandicoot | 2,195,348 | 2.13% | 0.70% | 504,381 | 99.3% | 3,495 | 1,523,785 | **99.7%** | 0 |
| Mega Man Legends | 978,426 | 2.92% | 0.31% | 278,059 | 96.0% | 1,091 | 1,077,932 | **96.0%** | 0 |
| Ape Escape | 506,206 | 6.00% | 0.44% | 232,789 | 78.5% | 36,896 | 862,886 | **81.1%** | 3,216 |
| Silent Hill | 176,128 | 5.24% | 0.25% | 231,032 | 35.1% | 7,988 | 897,376 | **34.9%** | 0 |
| Spyro the Dragon | 3,703,884 | 6.54% | 0.55% | 1,320,706 | 14.8% | 20,775 | 4,427,908 | **17.8%** | 464,915 |
| Rayman | 9,420 | 0 | 0 | 60,619 | 3.9% | 0 | 240,188 | **3.9%** | 0 |
| Castlevania | 17,998 | 0.04% | 0 | 13,241 | 0 | 0 | 52,644 | **0** | 0 |
| Tekken 3, frames 0 to 3,000 | 0 | | | 0 | | | 0 | | |
| Tekken 3, frames 0 to 12,000 (its attract mode's fights from about 7,500) | 5,788,727 | 5.70% (with far) | | 2,928,129 | 78.2% | 543,134 | 10,430,961 | **89.1%** | 879,756 |

By the counts at the anchors, the precise share rises as each game enters play: Crash Bandicoot's first 1,200 frames
draw 140 vertices, none precise (its logos), and by frame 1,500 122,138 of 122,318 are. Silent Hill's opening draws
558,800 vertices to frame 2,400 with none precise and its play then 120,284 of 120,284 more. No serial was found dead on
any disc (the ring of 32,768 is never outrun within a frame or two of a projection), and no command was unmatched.

**P10 held** [M]: the games that build their 3D scenes from the GTE's words (Crash Bandicoot, Mega Man Legends, Ape
Escape, Tekken 3's fights) are drawn 81 to 99.7 per cent precise, and those that draw mostly 2D (Rayman, Castlevania)
or move the GTE's words on the processor (Spyro the Dragon, 464,915 vertex words changed) under a fifth. Silent Hill
lies between: its scenes in play are wholly precise, its long opening and its fog's overlays not.

**The guard's bound stands** [M]: the far refusals are 0.25 to 0.70 per cent of the projections on every disc, against
2 to 6.5 per cent the console cut itself. P-E4d held for the guard's own refusals (under 1 per cent everywhere).

### 25.6 The player's path

- **The kit**, the published shape (`cargo build --profile dist` in `v1/`, `libdiamond.so`), the corpus form on the
  eight discs at 1,800 frames, four at a time: **off**, every case passes on every disc (194 s); **on**
  (`--settings precision=true`), every case passes on every disc after §26.2's fix (318 s), C3's nine settings among
  them, C7's state of 10,545,574 bytes loading and continuing, C8 across a save and load, C10's two machines on two
  threads, each with its own renderer, C17 and C18 [M]; both again on the final library, 192 s and 315 s.
- **WiseMan**, under the test lock with the corpus, filtered by `Diamond`, `PlayStation`, `GraphicsSettings`,
  `GenesisSettings`, `WindowFitAudit`, `CoreRegistrationEquivalence` and `CoreAdapter`: 219 of 221 at first, the two
  failures `DiamondTests`' lists of the library's settings and capabilities, which named neither the new key nor
  `SETTINGS` and `SETTING_NOTES`; given them, both pass. The three `DiamondPreciseGeometryTests` are among the 221, and
  the fit audit with the PlayStation's tab at both sizes, its new heading included [M].
- **The publish**: §28.

## 26. What the pictures showed

### 26.1 How they were looked at

At every 300th frame of the exact form's disc run the machine's picture and the renderer's were written to the probe's
scratch folder (`EMUSEN_DIAMOND_PICTURES`; no picture of a game is in the repository) and looked at side by side with a
map of the pixels that differ, then zoomed where the map or the eye found something; Tekken 3 the same every 500th frame
to 12,000 by the run tool's `--precise --pictures`. A scratch counter listed every black pixel of the renderer's picture
whose 3-by-3 neighbourhood in the machine's has no black and whose two opposite neighbours in the renderer's are not
black, a crack through a surface the console draws solid. Most of each map is the expected kind: every edge of a precise
polygon a pixel either way, and every textured pixel of a polygon in perspective a texel or two off the affine one.

### 26.2 The palette slot's tail reached the state (found by the kit, fixed)

**Shown.** The kit's first run with the setting on failed C7, C8, C17 or C18 on six discs of eight, every failure a
state digest with the frames' and the sound's digests equal. A scratch run of C7 on Mega Man Legends, machine A to frame
900, its state loaded on B, both on to 1,800, compared the two states field by field: 431 bytes differed, all in
`RNDR.Palette` [M]. A 4-bit palette load writes 16 entries of its slot of 256; the other 240 hold whatever the slot's
device memory held, which the block read back whole, and which differs between machines and between processes.

**Fixed.** The renderer records how many entries its slot's load wrote and saves the rest as zero; no texture reads
past them, since a 4-bit texture indexes 16 and the CLUT cache reloads for an 8-bit one (`ClutCache::holds`). The same
C7 then compared equal, and the kit passes every case on every disc with the setting on (§25.6).

### 26.3 A strip's last column showed the next region's texel (found on Crash Bandicoot, fixed)

**Shown.** At frame 1,800 Crash Bandicoot's loading picture showed a dark vertical line at x 351 and another at 414
[M, looked at]. The picture is drawn as textured strips 64 pixels wide whose corners the GTE projects to 286, 350.16,
414.32, …, one texel a pixel. VRAM read back from the renderer held at column 350 the dark colours of the next texture
region where the machine's held the picture. With integer corners the last pixel of a strip is a whole pixel inside its
right edge, its texture coordinate 63 exactly; with the edge at 350.16 the pixel at 350 is 0.16 inside, its coordinate
63.84, and the console's rounding of a coordinate (a half added before the integer part, §14.1) gives 64, the first
texel of the next region. The console never meets this at one texel a pixel; it meets it only on textures magnified
two times or more, where its own rounding reaches the last coordinate too.

**Fixed** [C]: with sub-pixel corners a precise triangle's texture coordinate is held within the vertices' range and
short of the end it runs toward across the screen (where both of its gradients are toward the greatest coordinate, that
coordinate's texel is not drawn; where both are toward the least, the least's), the record carrying the range in its
word 9. At zero fraction nothing is held, so §25.2's equality is untouched, and it stayed equal on every disc after the
change. On a polygon whose gradients point opposite ways (a texture turned on the screen) nothing is held beyond the
vertices' range; no such seam was seen. The strip test holds it (130 pixels past the strip without the hold, none with
it), and the lines are gone from the frame.

### 26.4 Recorded, not fixed

| Disc, frames | What shows | Why | Fate |
|---|---|---|---|
| Ape Escape, 1,500 to 2,700 (the lab's floor) | hairline cracks, black dashes up to a few dozen pixels long, where the floor meets the rainbow ring: 136, 79, 249, 89 and 12 crack pixels at the five anchors | the floor's polygons are precise; the ring's quads are not (their words do not trace to a projection: built or moved on the processor), and they meet the floor's long edge in a T, so the precise edge moves away from the integer corners on it by up to a pixel | **recorded**: §21.4's seam between a precise polygon and an integer one, which the plan names as this enhancement's cost |
| Spyro the Dragon, 900 and in play | short teal dashes across the snow and the ground, the background showing through | the same: 17.8 per cent of Spyro's vertices are precise and 464,915 of its vertex words were changed on the processor, so precise and integer polygons alternate across the ground | **recorded** |
| Every 3D game | every edge a pixel either way, textures a texel or two away from the affine picture | the enhancement itself | as designed |

**Looked at and found unchanged in kind**: Tekken 3's fights from frame 7,500 (its health bars, timer and names drawn
alike, the fighters and the stage precise), Spyro the Dragon's memory card warning, a 2D box over its 3D title, drawn
alike, Mega Man Legends' hall at frames 2,700 and 3,000, Ape Escape's close-ups, Silent Hill's street in its fog, Crash
Bandicoot's title and menu letters (projected, precise, whole), Rayman's title at 2,100; Castlevania's pictures are the
machine's at every anchor, nothing of it being precise. No misplaced vertex was seen: the guard's two pixels keep every
precise corner beside the console's.

*Answered the same day*: §32 reproduces and classifies both rows' cracks, and §33 closes them.

**An experiment, not kept**: drawing a mixed polygon with the precise corners it has and the integer ones at their
pixels, in place of wholly as before, was tried on the discs. Ape Escape's crack pixels fell at three anchors of five
(136 to 32, 249 to 131, 89 to 79) and rose at one (79 to 85); the T-junctions remain, since the ring's corners on the
floor's edge are not the floor's. It is not a fix, the plan's rule (§21.4) stays, and edge expansion (§22.4), the
planned remedy for this seam, is not built. Both are open for the tester, with their costs as stated there.

## 27. Cost

### 27.1 The instrument

§9's and §18.1's, unchanged: `examples/bench` on links with every function aligned to 64 bytes, each build in its own
target folder, the builds alternating on one core (`taskset -c 11`), the fastest mean of five rounds, milliseconds of
the emulation thread a frame; with a renderer attached the wait for its picture is timed with the frame. The base is
WiseMan 266a6650, extracted outside the repository. A sixth item is added, **Tekken 3's fights**: the disc run 10,500
frames and timed from frame 7,500, its attract mode's fights, since its first 3,000 frames make no projection (§7.2).
The device is the RX 6800. The bench's `--precise` attaches the pair as the setting does, `--precision` the precision
layer with a drawer that draws nothing, `--renderer` E3's renderer alone.

### 27.2 Off, against the base, and a cost that was there at first

The first series, five rounds, and a second of seven on three discs, stood Tekken 3 above the base [M]:

| Run | Base | Branch | Difference |
|---|---|---|---|
| Tekken 3, first series | 2.383 | 2.400 | +0.7% |
| Tekken 3, second series | 2.370 | 2.386 | +0.7% |
| Crash Bandicoot, the two series | 2.309, 2.309 | 2.307, 2.318 | −0.1%, +0.4% |
| Mega Man Legends, the two series | 1.799, 1.801 | 1.805, 1.800 | +0.3%, 0.0% |

The same figure twice is not the instrument's error. Comparing the links' symbol sizes, as §9.2 and §18.2 did: every
function of the plain frame was the base's size (`advance_in::<Plain>`, `exec::run` and `Run::execute` for the board)
but two. `Display::scan` was 0x809 bytes against the base's 0x436, with `Display::read_line` (0x371) gone from the
link: the branch's link had inlined the one into the other, though neither's source had changed. And
`raster::triangle` was 0x187F against 0x1878, seven bytes, as was `record::triangle`, which inlines the same helpers.
**By ablation** [M, seven rounds, the three builds alternating]: with `read_line` held out of line, both functions the
base's sizes again, Tekken 3 stood at the base, 2.375 against 2.373, where the build without the pin stood at 2.384;
Crash Bandicoot 2.303 and 2.305 against 2.297. **Decided [C]**: `read_line` is `#[inline(never)]`, the base's shape.
The triangle's seven bytes stayed with its source and every function it calls unchanged (the base vertex is asked of
`raster::setup` again, not of a function made public for it, which was tried first and was not the cause);
`gpu/benchmark`, the item that showed the triangle's shape at E3, shows nothing, so no cause was sought further.

The final build against the base, five rounds [M]:

| Run | Frames | Base | Branch | Difference |
|---|---|---|---|---|
| `gpu/benchmark` | 1,200 | 2.321 | 2.315 | −0.3% |
| `gpu/bandwidth` | 1,200 | 2.951 | 2.954 | +0.1% |
| Crash Bandicoot | 3,000 | 2.301 | 2.307 | +0.3% |
| Mega Man Legends | 3,000 | 1.798 | 1.799 | +0.1% |
| Tekken 3 | 3,000 | 2.372 | 2.379 | +0.3% |
| Tekken 3's fights | 7,500 to 10,500 | 3.707 | 3.705 | −0.1% |

Every pair's digests equal. **Off, the frame is the base's within the noise.**

### 27.3 On

The same build, five rounds, the four forms alternating [M]; every form's picture, sound and accurate state digests
equal to the plain run's:

| Run | Plain | The precision layer alone | **Precise geometry** | Over the plain frame | E3's renderer alone | The picture's wait, precise | renderer alone |
|---|---|---|---|---|---|---|---|
| `gpu/benchmark` | 2.315 | 2.373 | **2.654** | +0.339 (+14.6%) | 2.636 | 0.221 | 0.221 |
| `gpu/bandwidth` | 2.943 | 3.175 | **3.601** | +0.658 (+22.4%) | 3.546 | 0.296 | 0.295 |
| Crash Bandicoot | 2.309 | 2.493 | **2.776** | +0.467 (+20.2%) | 2.677 | 0.200 | 0.192 |
| Mega Man Legends | 1.806 | 1.910 | **2.141** | +0.335 (+18.5%) | 2.092 | 0.157 | 0.148 |
| Tekken 3 | 2.382 | 2.620 | **2.890** | +0.508 (+21.3%) | 2.795 | 0.135 | 0.130 |
| Tekken 3's fights | 3.721 | 3.955 | **5.186** | +1.465 (+39.4%) | 5.008 | 0.980 | 0.943 |

- **The precision layer** with a drawer that draws nothing costs 0.06 to 0.24 ms a frame over the plain frame: the
  layered frame's dispatch (§9.3, §18.3's middle share) and the layer's own stores, 1.0 to 1.1 times what E1's test
  observer cost on the three discs (0.184, 0.104 and 0.238 ms against 0.169, 0.102 and 0.216). Its 2 MiB shadow of RAM
  is not felt beside the dispatch.
- **Precise geometry over E3's renderer alone** is 0.02 to 0.10 ms on the five items and 0.18 in the fights: the
  precision layer's stores and resolutions, the renderer already paying the layered frame. The picture's wait is the
  renderer's within 0.01 ms on the five items and 0.04 in the fights: **the division a pixel and the 64-bit edge
  functions of the precise record are not measurable on this device**, which settles §22.3's choice of a division a
  pixel over one a span.
- **The fights** are the finding: the renderer, as E3 built it, costs 1.29 ms a frame there, 0.94 of it the picture's
  wait, where no item measured before reached 0.6. Tekken 3 draws its fights at 480 interlaced lines, with about 12.5
  stretches, a hundred palette loads and a VRAM copy every other frame (150,678 stretches, 1,212,215 loads and 5,371
  copies over the 12,000 frames [M, the run tool]); the wait was not split further into the device's drawing and the
  submission's many dispatches. It is the renderer's to look at before E5 multiplies it (§30).

Under §9.3's budget and the plan's (`Diamond_Plan.md` §5.12), the frame with the setting on is 2.1 to 3.6 ms on the
five items and 5.2 ms in Tekken 3's fights on the desktop, against 16.7 ms.

### 27.4 On the handheld, estimated

**An estimate, not a measurement** [A]; the handheld is not part of this stage and the tester will try the setting
there. By §18.4's reasoning (the integrated Radeon at its 800 MHz floor, some ten to fifteen times less compute
throughput than the RX 6800, no bus between the host's arrays and the device's, the host's shares half again slower):
**about 2 to 3 ms a frame more than the accurate frame where a game draws at 240 lines** (the device's share here 0.05
to 0.15 ms, the host's 0.12 to 0.37), **and about 10 ms more, 9 to 13, in Tekken 3's interlaced fights** (the wait
here 0.98 ms, most of it taken as the device's). The first leaves full speed with room; the second, beside an accurate
frame that is itself half again the opening's there (3.7 ms against 2.4 on the desktop), could stand near the edge of
16.7 ms, and wants the picture's wait taken off the emulation thread (§18.3, §30) before it is relied on.

## 28. The publish, and the tests run

### 28.1 The publish

`dotnet publish EmuSen.Mistress -c Release -r linux-x64` into a scratch folder [M]: the build runs cargo in `v1/` with
the `dist` profile, as `RustCores.props`' row now names it, and the tree carries `lib/EmuSen/libdiamond.so` (2,130,448
bytes) with `libdiamond.so.core.json` beside it, written by the kit's runner from the library itself: 51
`emusen_core_` exports, `set_settings` and `setting_notes` among them, and the sidecar's ninth setting `precision`,
"Precise geometry", with its help and cost. The name, the core's id, the system pack and what Mistress reads are as
they were; nothing but the row's `Dir` changed in the build, and CI's matrix entry for the crate (§24.1).

Turning the setting on takes 31 to 69 ms on the desktop (the device, its four programs and its buffers made, the pair
attached and its first picture read; three trials), once, between two frames; turning it off 2 ms [M, a scratch timing
in `diamond-v1`'s tests].

### 28.2 Tests [M, 2026-10-13]

All on the final head, after `git merge WiseMan` (WiseMan had not moved from 266a6650).

- **`blackmoon-r3000`** and **`blackmoon-gte`**, `cargo test --release` with `EMUSEN_BLACKMOON_CORPUS`: 38 and 10, the
  r3000 suite's cases among the first, `an_observer_sees_each_projection_and_changes_nothing` holding H and the offset.
- **`diamond`**, `cargo test --release` with `EMUSEN_DIAMOND_PS1TESTS`, `EMUSEN_DIAMOND_DISCS` and
  `EMUSEN_GPU_DEVICE=6800`: 167 unit tests, 81 program tests (E1's three goldens plain and with the test observer, the
  discs' among them), E3's 3 renderer gates and **E4's 8** (`tests/precise.rs`), all passing; and without the two
  corpora, the same counts, the corpus tests standing down.
- **`precision`**: 6. **`diamond-renderer`**: 1. **`diamond-v1`**: 4, and 4 with `EMUSEN_GPU_DEVICE` naming no device,
  the two that need one standing down with the note printed.
- **The kit**, the corpus form on the eight discs at 1,800 frames on the `dist` library, off and on: every case on
  every disc, 192 s and 315 s (§25.6).
- **WiseMan**, under the test lock with the corpus, the filter of §25.6: 221 of 221.
- **Not run**: `emusen-gpu`'s and MarsRT's suites, `emusen-gpu` and `emusen-native` being unchanged; the validation
  layer over the precise record, which E3 ran over the walk it extends (§19.1); the full WiseMan suite; llvmpipe and the
  integrated card.

## 29. Predictions

- **P-E4a held** [M, §25.3]: the grid's mean vertex error fell from 0.889 px to 0.008 px (predicted: a third of a pixel
  to under a hundredth; the integer error was larger than predicted, since §21.1's second loss adds to the first), and
  with MAC0's sums alone to 0.159 px, inside the predicted tenth to a quarter.
- **P-E4b held in its first half and failed by its own measure in the second** [M, §25.3]: off 2.75 texels; on 0.5004
  texels, where "under half a texel" was predicted. The measure, |Δu| + |Δv| against an unrounded reference, has a floor
  of half a texel for any picture of whole texels; the prediction should have been stated against that floor. Retired.
- **P-E4c held for two games of three and its second half held** [M, §25.5]: Crash Bandicoot 99.7 and Ape Escape 81.1
  per cent; Spyro the Dragon 17.8, predicted over 70: it moves the GTE's words on the processor. Castlevania 0 and
  Rayman 3.9, predicted under 10. P-E1c, of the same games' DMA words, is retired with it: true of Crash Bandicoot, false
  of Spyro the Dragon.
- **P-E4d held** for the guard's own refusals, 0.25 to 0.70 per cent (§25.5); the console's own cuts, 2 to 6.5 per cent,
  were not what it predicted and are counted apart.
- **P-E4e held** (§25.1). **P9 held** for E4. **P10 held** (§25.5).
- **P-E4f held** [M, §27]: off, the base within the bench's noise, once the display's `read_line` was held out of
  line; on, 0.34 to 0.66 ms a frame more than the plain frame on the five items and 0.34 to 0.51 on the three discs,
  under the 1.5 predicted, the precision layer with a drawer that draws nothing 0.10 to 0.24 ms there, under the 0.3
  predicted. Tekken 3's fights, which the prediction did not name, cost 1.47 ms more, almost all of it the renderer's as
  E3 built it.
- **P-E1b held** [M, §27.3]: the precision layer with a drawer that draws nothing costs 1.0 to 1.1 times what E1's test
  observer cost over the plain frame on the three discs (0.184, 0.104 and 0.238 ms against 0.169, 0.102 and 0.216),
  where at most twice was predicted.

## 30. What E5 needs next

- **The walk at N×.** The `PRECISE` record already places vertices in 256ths of a pixel and evaluates edge functions
  and interpolants at any point of that grid; at N× a sample is a point `256/N` apart, so E5 can draw every triangle
  through it, the integer ones with zero offsets, which §25.2 shows is the integer rule. The texel hold (§26.3) will
  matter more there, since every sample but one in N² lies between the console's.
- **The mirror and its block.** `RNDR` holds the renderer's mirror, 1 MiB at 1×; at N× it is N² that. Whether a state
  keeps an N× mirror, or keeps the console's and redraws, is E5's first decision about states; the kit's C7 and C8 hold
  whichever it takes to a picture that continues exactly.
- **The picture's wait.** It is 0.14 to 0.30 ms here on the five items and 0.98 ms in Tekken 3's fights (§27.3),
  which no stage had measured before; at N² that share cannot stay on the emulation thread. `emusen-gpu`'s pending
  submission (§18.3) is the planned form. The fights' cost is mostly stretches and palette loads (12.5 stretches and a
  hundred palette loads a frame), which is the renderer's planning to look at first.
- **The frame's form.** With internal resolution offered without precise geometry, the renderer is attached alone and
  wants the plain frame (§24.1); `Layer::moves` is the method to add then.
- **The seams** (§26.4): edge expansion, off, with its cost; or mixed polygons drawn with the precise corners they
  have, which the experiment there found a partial remedy. Both are the tester's to choose. *Closed since by neither:
  §33's crack fill; at N× its marks are the mirror's, N² of them, and its rule a sample's (§36.3).*
- **The composition** stands: `diamond-v1` attaches whatever layer the settings ask for, and a second setting is a
  second entry in its `ENHANCEMENTS` and another form of the pair.

## 31. Reading log (E4)

**No description of PGXP, and no description of any emulator's precision, enhancement or renderer, was opened**, with
or without code; the plan allowed prose without code, and none was needed. No emulator source of any kind: nothing of
DuckStation's, Beetle PSX's (parallel-psx and its rsx among it), PCSX's, PCSX-Redux's, SwanStation's, ePSXe's plugins or
any fork's, no header, no shader, no snippet. No firmware, no listing or trace of one; the player's BIOS folders were
not read; nothing under `kernel/` or `kernel-harness/` was opened or changed. psx-spx was not opened. ps1-tests'
programs were run as built, and none of their sources or `common/` files was opened. No game's code was disassembled
or traced: the discs were run, their pictures looked at, and the layer's counts and the GPU's command words of two
frames (Ape Escape's and Crash Bandicoot's polygons near a seam, their vertices, texture coordinates and offsets)
printed by a scratch test.

Opened, all EmuSen's own: `Diamond_Plan.md` §1, §6, §7, §8, §10, §11 Q14 and §13; this page whole; `Diamond_GPU.md` at
D-23, D-47 and §13; `BlackMoon_GTE.md` §5 and §9; `BlackMoon_R3000.md` §14; `Diamond_Native.md` §24 and §25;
`Diamond_Players.md` §4 and its headings; `EmuSen_CoreAPI.md` §6.5 to §6.9, §6.13 and §6.19;
`EmuSen_Settings_Reference.md` §4.108 and §4.109 for their form. The source of `diamond` (`layer.rs`, `v1.rs`,
`state.rs`, `machine.rs`, `gpu/mod.rs`, `gpu/decode.rs`, `gpu/raster.rs`, `tests/` and `examples/`), of
`diamond-renderer` whole, of `blackmoon-gte`'s `rtp` and `blackmoon-r3000`'s loads and stores, of `emusen-native`'s
`Core`, exports, state reader and writer, of `emusen-gpu`'s interface, of the kit's C6 to C8 and its command line;
`RustCores.props`, `RustCoresPublish.targets`, `EmuSen.csproj`'s cargo targets and the Rust cores' workflow;
Mistress's `GraphicsSettingsWindow.cs`, `CoreDescriptors.cs`, `CoreEngine.cs`, `CoreCatalog.cs` and `CoreCapabilities.cs`;
WiseMan's `DiamondTests.cs`, `GenesisSettingsTests.cs`, `SyntheticPsExe.cs` and the fit audit's list; E1's and E3's
measurement scripts in the probe. What precision is for came from the plan's §6 and from the hardware as Diamond
models it: the GTE's fixed-point projection and its FIFO, the GPU's integer vertices and its rounding of a texture
coordinate, DMA's list.

## 32. The cracks between polygons: reproduced, counted and classified

*Measured 2026-10-13, after the tester's report of the same day from the handheld with "Precise geometry" on: "I am
seeing cracks in Ape Escape."* §26.4 had recorded the defect at stage E4 and left it; this section says where it shows,
how much, and why, and §33 to §36 what was built against it, how it was measured on both sides, and what is left.

### 32.1 The measure

§26.4's counter took a black pixel of the renderer's picture between two that are not black, where the machine's has
no black about it. It counts Ape Escape's lab, whose floor is drawn over a cleared buffer; it does not count a crack
through which a sky or a far hillside shows, which is what a level in play has behind its ground. **A crack pixel is
defined here by who draws it** [C]: a displayed pixel whose last writer in the accurate picture is a polygon that
covers it by the integer rule, where the enhanced picture's last writer is an older command that the accurate picture
shows nowhere within two pixels, and is not a precise polygon across an edge it shares with the first. The first
clause is the plain sense, a pixel the console's polygon covers and the enhanced picture leaves to what lies behind;
the second leaves out an edge that only moved, which shows the neighbour the console also shows beside it; the third
leaves out the thin precise triangles a game puts along a T, which the console's integer vertices close to nothing
and the precise ones open to a pixel's width, a neighbour showing and not a hole.

It is computed by a scratch layer beside the pair (`scratch_cracks.rs` in the probe's folder, not in the repository):
two maps of VRAM by owner, one written by every command's integer coverage and one by the coverage the renderer
draws, each from the renderer's own records (`record::triangle` and `record::precise_triangle`), read through the
display's lines at the frame's end. Semi-transparent untextured polygons own nothing, since they tint what is there;
transparent texels are not modelled, so a polygon owns its whole triangle. The same layer keeps the precision layer's
shadow a second time with what that one drops, for the classification of §32.3: a projection the guard refused, a
move by halves, the instruction that last stored each word. §26.4's counter is kept beside it for continuity.

### 32.2 Reproduced

The pair attached from power-on, the discs by their press scripts on the open kernel, the RX 6800 [M]:

| Run | Frames looked at | Crack pixels | A frame | Worst frame | §26.4's counter |
|---|---|---|---|---|---|
| Ape Escape, the five anchors 1,500 to 2,700 (the lab) | 5 | 632: 143, 117, 268, 88, 16 | 126 | 268 at 2,100 | 565: 136, 79, 249, 89, 12, as recorded |
| **Ape Escape in play**, frames 7,300 to 20,500, every 50th | 265 | **17,611** | **66** | 336 at 18,350 | 6,983 |
| Spyro the Dragon, the ten anchors 300 to 3,000 | 10 | 664 | 66 | 269 at 1,500 | 0 |
| **Spyro the Dragon in play**, frames 5,300 to 17,000, every 50th | 235 | **58,023** | **247** | 923 at 7,250 | 124 |
| Tekken 3's fights, frames 7,500 to 10,500, every 250th | 13 | 2,609 | 201 | 329 at 8,000 | 468 |

Ape Escape's press script starts a new game, skips its scenes and from frame 7,260 holds the left stick about the first
area with a press of Cross every 300 frames; the run was taken to frame 20,500, 220 seconds of play in the first level,
its hills, ledges and the cliff face in view (looked at, frame 18,350). There the cracks are what the tester saw:
**lines of the sky's and the far cliff's colours along the edges of the ledges and the ground**, a few pixels to a few
dozen long, changing from frame to frame as the camera turns. In Spyro the Dragon's first world, flying about from
frame 5,280, they are dashes of the far ground's lighter green across the grass and along the walls' feet. §26.4's
counter is no measure of either in play, since neither shows black through a crack: it finds almost none of Spyro's,
and in Ape Escape's dark cliffs most of what it counts is a black texel that a perspective-correct texture puts a
pixel from where the affine one does, no crack at all (looked at, frame 18,350: 201 counted, on the cliff's texture).
Tekken 3's fights are counted on VRAM as it stands at each frame's end; at 480 interlaced lines every other line of
the picture is a frame older.

Beside the cracks, 8,772 pixels of Ape Escape's 265 frames and 3,213 of Spyro's 235 show a precise neighbour across a
shared edge that the console does not show (the thin triangles of §32.1), and are not counted.

### 32.3 Classified

Each crack pixel is classed by the polygon the accurate picture shows beside it (among the eight pixels about it, an
integer polygon before a precise one), by that polygon's vertices' histories, and by how it meets the polygon the
crack is in: sharing two vertices' pixels (an edge), having a vertex within a pixel of one of its edges and not at its
ends (a T), sharing one (a corner), or none. A vertex word's history is one of: **precise**; **none**, no projection's
serial reached it; **lost**, none reached it though the word equals a recent projection's; **changed**, a live serial
whose word is no longer its projection's; **halves**, it was moved by halfword loads and stores; **far** and **cut**,
its projection was refused by the guard (§21.2) or cut by the console; **dead** (§21.3). [M]

| Class, crack pixels | Ape Escape, anchors | Ape Escape, play | Spyro, anchors | Spyro, play |
|---|---|---|---|---|
| All | 632 | 17,611 | 664 | 58,023 |
| **Beside a polygon drawn by the integer rule** | 582 (92%) | 10,171 (58%) | 639 (96%) | 53,924 (93%) |
| … of which a **mixed** polygon, some of its vertices precise | 582 | 9,781 | 89 | 40,550 |
| … of which a polygon with no precise vertex | 0 | 390 | 550 | 13,374 |
| … meeting at a **T** | 528 | 8,898 | 75 | 5,911 |
| … along a shared edge | 0 | 0 | 465 | 40,364 |
| … at a corner, or apart | 54 | 1,273 | 99 | 7,649 |
| … its worst vertex **none** | 569 | 10,056 | 90 | 32,173 |
| … **lost** | 12 | 93 | 189 | 16,952 |
| … **changed** | 0 | 5 | 360 | 849 |
| … **halves** | 1 | 17 | 0 | 0 |
| … refused by the guard, **far** | 0 | 0 | 0 | 3,769 |
| … **cut** by the console | 0 | 0 | 0 | 180 |
| **Between two precise polygons** that share no whole edge | 33 | 5,323 (30%) | 25 | 2,603 |
| … at a T | 6 | 2,614 | 0 | 1,108 |
| Beside a rectangle, a fill, a copy or nothing drawn | 1 | 1,899 | 0 | 175 |
| No other owner about it | 16 | 218 | 0 | 1,321 |

**Why the neighbour was integer, by class.**

- **Ape Escape divides its near polygons on the processor** (none: 569 of the anchors' 632, 10,056 of the play's
  17,611). The routine about 8001DDC4h (read by the run tool's `--disasm` after the tracing named it) takes a polygon
  whose vertices the GTE projected, reads each screen position and depth by halves, and makes the middle of each edge
  as the 3D middle projects: the two ends' screen positions weighted by their depths, through MVMVA with the positions
  as a matrix and the depths as its vector, and a division by the depths' sum; colours and texture coordinates
  averaged; the new vertex stored by halves. The smaller quads go to the GPU by SWC2 from the FIFO it reloads with
  them, most with one original corner and three made ones: 209,003 of the play's 465,388 integer polygons have
  the history none, none, none, precise, and 379,012 of its 4,776,383 polygons are mixed. It is the game's own remedy
  for the console's affine texturing, applied near the camera. A divided polygon's new vertex lies on its undivided
  neighbour's edge, a T. On the console the made vertex is a whole pixel on or beside the line between two whole
  pixels, and its pictures show the T closed (looked at, frames 2,100 and 18,350). With the setting on the undivided
  neighbour's edge runs between its precise corners, up to a pixel from that line, and the divided one's stays on it. **The neighbour was integer because three of its four
  vertices never were projections**: there is nothing for the layer to follow.
- **Spyro the Dragon keeps each projected vertex shifted** (lost: 16,952 of 58,023; and the reason only 37 per cent
  of its play's polygons are precise). Its routines that project (about 8004EE08h, 80026978h and 8002631Ch) take SXY2
  by MFC2, shift it left five into another register, add clip flags in the five low bits, and store the word; the
  routines that emit polygons load it, shift it right five in place, and store it into the packet. The word the GPU
  receives is the projection's again, but its serial was lost at the first shift: a shift into another register is
  not a move the processor tells (§3.2), so the new register keeps whatever it had. 5,122,567 of the play's 9,050,763
  polygons have every vertex lost and no other fault. **The neighbour was integer because the layer follows copies
  and not arithmetic.** §32.4 says what following this one would take.
- **Spyro's vertex with none** (32,173; 163,014 quads of three precise vertices and one with none, and 25,951
  triangles of one precise and two with none): the word comes from a second store of the same routines (80027BCCh,
  80028E44h) that no projection's word reaches. Not traced further; it is the largest class left unexplained.
- **The guard's refusals** make 3,949 of Spyro's play (far 3,769, cut 180) and none of Ape Escape's: a polygon with
  one vertex refused, drawn whole by the integer rule beside precise ones (§21.4's rule). The guard's bound is not
  the cracks' cause in either game.
- **Changed** words (Spyro's anchors 360, its play 849) are the same shift made in a register that still held an
  older projection's serial: the word received is one vertex's and the serial another's, and they differ by tens to
  hundreds of pixels. §25.5's "changed" count for Spyro, 464,915, is this kind, not an offset added to a vertex. In
  neither of these two games do the cracks come from a projected vertex moved by a constant, clipped or rounded on the
  processor.
- **Tekken 3 moves projected vertices on the processor** (its fights, 2,609 on 13 frames: 1,862 beside a mixed polygon
  one of whose vertices is changed, at a T (945) or along an edge (534); 635 between two precise polygons). Here the
  changed words are what §25.5 took them for: 1,167,689 of the run's 10,637,167 vertex words carry a live serial and
  a word a pixel or a few from its projection's, the commonest differences (0, -1), (-1, -1) and (-1, 0) with
  80,554, 78,635 and 73,416 words, the rest within about four pixels either way. The differences are not one
  constant; what the game computes (the vertices at a fighter's joints are the likely ones) was not traced. 586,501
  of its 3,027,551 polygons are mixed by it. **The neighbour was integer because one of its vertices is no longer its
  projection's**, which the integer check exists to refuse (§21.3).
- **Between two precise polygons** (Ape Escape's play 5,323, 30 per cent): two polygons whose vertices are all
  precise and that meet without sharing an edge's two ends: a T the game left open or closed with a thin triangle,
  and models set against one another. The console's integer vertices close such a gap about as often as they open
  one; precise vertices leave it as wide as the geometry has it, a fraction of a pixel, which a pixel's centre falls
  in now and then. These are cracks the precise geometry itself shows, not a meeting of precise and integer.

### 32.4 What could be removed at the source, proposed and not built

- **Following a shift** [proposed]. If the processor told a shift by a constant into another register as it tells a
  copy, the layer giving the new register the old one's serial, Spyro's words would carry their serials through the
  shift left, the flags added in place and the shift right in place, and the integer check at the receipt (§21.3)
  would pass them: the word received is the projection's. *What it would take*: one more move in
  `blackmoon-r3000`'s `Move` (`BlackMoon_R3000.md` §14), told under `B::MOVES` where `Copy` is; one arm in the
  precision layer; a program of the crate's own that shifts, flags and shifts back. *What it would give* [A, from the
  histories counted]: Spyro's precise polygons in play from 36.9 per cent to about 93.5, and the 16,952 crack pixels
  of the lost class gone at their source, with more of the 32,173 beside a mixed polygon whose other vertices are
  lost. *What it would risk*: nothing on the screen that §21.3's argument does not already cover, since a serial on a
  register whose value is not its projection's word is refused at the receipt, and one whose value is that word is
  that pixel; more registers carry stale serials, which costs nothing until a word is received. Its cost is one event
  for each such shift, a few per cent of instructions. **It is not built here**: it widens a written interface of the
  processor's crate, which is the tester's to agree first; and it is not needed for the cracks, which §33 closes
  whatever their cause. *Accepted as a trial on 2026-10-14 and built: §54 and §55.*
- **Ape Escape's made vertices and Tekken 3's moved ones** [not proposed]. A precise middle, or a precise moved
  vertex, would need the layer to follow values, not serials: for Ape Escape two halfword loads, an MVMVA, an
  addition and a division, computed beside the game's with precise inputs. Nothing checks such a value but its
  nearness to the integer the game made; it is not exact in §21.3's sense, and it is a second, larger enhancement.
  Drawing a mixed polygon with the precise corners it has (§26.4's experiment) does not close a T either, the made
  vertex staying where it is.

## 33. The crack fill [C]

*Built 2026-10-13, after §32.* The plan's row foresaw two remedies (`Diamond_Plan.md` §6.3, "Seams and gaps between
polygons"): precise shared vertices, which E4 built and which leave §32's cracks, and an expansion of every edge by a
fraction of a pixel, which draws what the console never drew and was left unbuilt (§22.4). What is built is neither.

### 33.1 The rule

On the console there are no such cracks, because every polygon is drawn at whole-pixel vertices and neighbours agree.
**A precise triangle therefore also covers each pixel that the console's own version of it covers**: the triangle of
the same vertices without their fractions, by the integer rule (§14.1), under the same drawing area and the same
skipped lines. Two properties follow by construction, and are the whole argument:

- it never leaves uncovered a pixel the console covered with that polygon, so nothing behind shows where the console
  shows the polygon;
- it never draws a pixel the console would not have drawn with that polygon, which an expanded edge does.

The cause of a crack does not enter: a made vertex, a lost serial, a refused projection and a T between two precise
polygons (§32.3) are closed alike, since each is a pixel the console's triangle has and the precise one has not. At
zero fraction the two triangles are one and nothing is filled; the record is then E4's, but for where its edges' bounds
are kept (§33.2), and §25.2's equality holds as before (§35.1). A precise triangle of no area whose integer triangle
has some, which E4 did not draw, sends its polygon to the integer path, counted with those whose gradients do not fit.

Two refinements were forced by what the rule does where it is not wanted, each shown before it was built: §33.3,
where two precise polygons share an edge, and §33.4, where a polygon blends.

### 33.2 The console's triangle in the record, and a filled pixel's place

The walk's record for a precise triangle (§22.2) had no word free. Its three edges' bounds, a word each of 0 or 1,
move to three bits of its flags, and those words now hold each edge's first vertex's fraction, x in the low half and
y in the high. From a vertex in 256ths less its fraction the walk has the console's integer vertex exactly, and three
more flag bits give the integer edges' bounds, a fourth whether the console winds the triangle the other way (a thin
triangle can turn over between its integer and its precise corners), a fifth that the triangle is filled at all. Its
bounds are the two triangles' together. For a pixel the precise edges refuse, `console_covers` evaluates the three
integer edge functions in 32 bits, as E3's triangle does.

**A filled pixel takes its place in the console's triangle to the same place in the precise one.** The three integer
edge functions at the pixel are the pixel's weights in the console's triangle, none negative, their sum its doubled
area. They are used as the precise triangle's weights: the texture in perspective from them and the depths, as for any
pixel (§22.3); the colour and an untilted texture by the fine gradients at the point of the precise triangle the
weights name, rounded to a 256th. The point is inside the precise triangle, so every interpolant is within its
vertices' range with no extrapolation, the texel hold of §26.3 applies as to any pixel, and at a vertex with no
fraction the place is the pixel itself. Two other definitions were set aside: the gradients extended to the pixel
itself, which overshoot on a thin triangle whose gradients are steep, and the nearest point by clamping the precise
weights, which is the pixel projected from the opposite vertex onto the edge and lands up to 1.1 texels along a
121-pixel edge from where the console's own interpolation puts it [A, computed for §33's test].

### 33.3 Marks: an edge two precise polygons share stays where E4 drew it

**Shown first.** Filled wholly, the later of two precise polygons that share an edge takes every pixel between the
console's edge and the precise one from the earlier: `two_precise_quads_sharing_an_edge_spill_nothing` counts 18 such
pixels of 2,118 about two slanted edges, and the turning grid's shared edges return to the console's places, 0.556
pixels from the ideal on average where E4 had 0.0035 (§34.2). The console's truncation moves every vertex up and to the
left of its precise place, so the polygon to the right or below, when drawn later, always wins: on the grid the whole
of the console's wobble comes back, not half of it.

**Built.** The renderer's mirror of VRAM is a 32-bit word a pixel of which the console's halfword uses 16. The other
16 are the pixel's **mark**. A precise triangle that covers a pixel writes with it 15 bits made from the two ends, in
256ths, of the edge the pixel lies nearest in the triangle's own terms (the smallest of its three edge functions);
the mark is the same from either triangle that has the edge, since two polygons that share an edge exactly share its
ends' positions, and is never zero. Every other write leaves none: the walk's `put` builds the word from the colour,
and the copy's shader now masks what it moves. A fill is refused where the pixel's mark is the mark of an edge the
pixel lies beyond, which is to say where the precise triangle on the other side of that edge drew it. Nothing is kept
on the host: no table of edges, no neighbour found by search; the test is made at the pixel, in the order pixels are
written.

What a mark cannot tell [M, by the model of §32.1 extended to each form]: near a shared corner, and across a triangle
thinner than about two pixels, the earlier triangle's nearest edge is another, and the later one fills there. In Ape
Escape's play this is 77,983 pixels of the 265 frames against 385,104 for the whole fill, in Spyro's 54,790 against
371,721. A wrong match of two unrelated edges' marks leaves a crack once in 32,768. Other forms of the mark were
modelled on the same frames and not built:

| Form of the mark | Shared-edge pixels still taken, Ape Escape / Spyro | Cracks left, of 17,611 / 58,023 |
|---|---|---|
| None: the whole fill | 385,104 / 371,721 | 2,324 / 213 |
| **The nearest edge, 15 bits (built)** | 77,983 / 54,790 | **2,149 / 194** |
| The nearest edge by distance in pixels | 67,290 / 49,474 | 2,137 / 193 |
| The two nearest edges, 8 bits each | 32,600 / 24,283 | 2,268 / 239: collisions |
| The nearest vertex, 16 bits | 10,758 / 5,155 | 3,042 / 689: a polygon behind that has the vertex is taken for the neighbour |
| The triangle's three vertices, 5 bits each | 28,163 / 23,080 | 2,781 / 423: collisions |
| A table of edges on the host, no marks | 2,633 / 5,821 | 5,377 / 10,088: a closed edge's neighbour does not cover every pixel past it; and a table is state a loaded machine would not have |
| A mark of the frame alone: fill only where no precise polygon wrote since the drawing area was set | none | 17,052 / 6,606: in play what shows through a crack is itself a precise polygon |

About 2,150 and 194 of the cracks left are left by every form: those of semi-transparent polygons (§33.4). The last
form suffices at Ape Escape's anchors, whose floor is drawn over a cleared buffer (26 of 632 left), and fails in play.
The built form is the one that leaves no crack of its own; the two-edge form halves what is still taken for 0.7 per
cent of the cracks left open, and is the next to try if the corners are ever seen.

### 33.4 A semi-transparent polygon fills only where an opaque one did

A semi-transparent pixel filled where two semi-transparent polygons meet would be blended by both, the fault the plan
names for edge expansion. The marks refuse it across an exactly shared edge, but not at a T, where the precise
neighbour's coverage and the polygon's own integer coverage overlap along the whole edge. So a semi-transparent pixel
is not filled: an untextured semi-transparent triangle, and a textured one's texel whose bit 15 asks for blending,
which the walk fetches to see; a textured semi-transparent triangle's opaque texels are filled as an opaque polygon's.

**One case had to be given back, found on Silent Hill** (§34.5). A game that shades in two passes draws each polygon
twice, opaque and then semi-transparent over it. Filled by the rule above, a crack's pixel took the first pass and
not the second, and showed the first pass's colour alone where the console shows the two together. **A filled pixel
therefore carries a mark too**, the mark of the edge it lies beyond with a sixteenth bit set, and a semi-transparent
triangle fills a pixel exactly where it finds that mark for one of its own edges: where an opaque triangle with the
same edge filled it and nothing has drawn since. The pixel then takes both passes or neither. No pixel is blended
twice by it: the pixel is one no precise triangle has covered since the opaque fill, and the semi-transparent
triangle's own precise coverage does not include it.

### 33.5 The state

*Superseded 2026-10-14 (§52.2): the marks are no longer kept; a load finds none, as an attach does.*

`RNDR` gains the marks, a halfword a pixel after its palette, 1,048,576 bytes; a state with the setting on is
11,594,150 bytes against 10,545,574 at E4 and 5,398,770 with it off [M, the kit's C7]. They are kept because a
picture depends on them across a save: `a_state_between_two_quads_that_share_an_edge_keeps_their_marks` saves between
two precise quads that share an edge, and a machine given that state with its marks cleared draws 24 pixels
otherwise. The kit's C7 and C8 did not show it: with the marks left out of the block they still pass on Spyro the
Dragon, whose frame at the kit's save has precise polygons, since no polygon there is drawn across the save from its
neighbour [M]; the test is the proof and the kit the check that nothing else moved. A state made at E4 has no marks
and loads: none are taken, and its first frame fills as it finds.

## 34. The fill measured on both sides

*Measured 2026-10-13 on the desktop, the RX 6800.* Each form was run with the renderer built for it and, where two
pictures of one frame were wanted, with two renderers behind one precision layer, one filling and one not (a scratch
build; the built renderer has one form and no switch).

### 34.1 The forms tried

| Form | Cracks left, Ape Escape's anchors / play | Spyro's anchors / play | Pixels changed from the unfilled picture, Ape Escape's play / Spyro's | The grid's shared edges (§34.2) |
|---|---|---|---|---|
| None, E4's picture | 632 / 17,611 | 664 / 58,023 | 0 | 0.0035 px |
| The whole fill | 7 / 2,294 | 15 / 590 | 705,023 / 662,150 | **0.556 px, the console's** |
| **The fill with marks, built** | **7 / 2,335** | **15 / 591** | **303,475 / 250,926** | **0.0036 px** |
| Marks of the frame alone (§33.3) | 26 / 17,052, by the model | 130 / 6,606 | 24,119 / 84,706 | not run |
| Mixed polygons drawn with the precise corners they have (§26.4's experiment) | 136 to 32, 249 to 131, 89 to 79, 79 to 85 at four anchors by §26.4's counter | not rerun | | not run |
| Every edge expanded (§22.4) | not built: the one form that draws what the console never drew | | | |

The cracks left are counted in the pictures: a crack pixel of the unfilled picture that the filled one shows
unchanged. Those of the built form are semi-transparent polygons' (§33.4, §34.3).

### 34.2 The turning grid, and its edges

§25.3's figures are the grid's vertices and the texels well inside its quads, which no fill reaches: with the built
fill they are §25.3's to the digit (mean vertex error 0.0080 px, change 0.0113, worst 0.0308; texel error 0.5004,
0.02 per cent over a texel). The fill can cost only at edges, so the grid is graded there too
(`edge_places` in `tests/precise.rs`). **For each of the grid's edges and each frame, the pixels within 2.5 of the
ideal edge and 2.5 clear of its ends that lie on one quad's side and were drawn by the other, less those the other
way, over the edge's length**: the distance, in pixels, at which the picture puts the edge from where the geometry
has it. Which quad drew a pixel is read from its texel, whose u or v ends near 63 on one side and starts near 0 on the
other. The grid has 24 edges between two quads, which two precise polygons share, and 16 on its outline, with the
cleared buffer beyond. 120 frames [M]:

| Picture | Shared edges: mean distance | change, frame to frame | worst | Outline: mean distance | change | worst |
|---|---|---|---|---|---|---|
| The console's (the software rasteriser) | 0.558 px | 0.209 px | 1.492 px | 0.584 px | 0.223 px | 1.397 px |
| E4, unfilled | 0.0035 | 0.0066 | 0.092 | 0.0061 | 0.0110 | 0.113 |
| The whole fill | 0.556 | 0.203 | 1.492 | 0.287 | 0.116 | 1.339 |
| **The fill with marks, built** | **0.0036** | **0.0066** | **0.092** | **0.287** | **0.116** | **1.339** |

No pixel between two quads was left undrawn in any of the four. Two things are measured here.

- **On the edges two precise polygons share, the built fill is E4**: 0.0036 px against 0.0035, the change and the
  worst the same. The whole fill is the console there, as §33.3 says.
- **On the outline the fill brings half of the console's edge wobble back**, in either form: 0.287 px from the
  ideal against 0.006, moving 0.116 px a frame against 0.011. A precise polygon's vertices lie to the right of and
  below the console's, so the fill moves its top and left edges out to the console's line and leaves its bottom and
  right edges where the geometry has them. This is the fill's cost, and it is not one a mark can remove: at an open
  edge a crack and an outline are the same pixels, covered by the console's triangle and by nothing precise, and what
  tells them apart is whether a polygon the console draws stands beyond, which the pixel does not know.

### 34.3 The discs: cracks, and what the fill changes

**Cracks**, in the pictures [M]:

| Run | Crack pixels unfilled | With the built fill | §26.4's counter, unfilled and filled |
|---|---|---|---|
| Ape Escape, the five anchors | 632 | **7** | 136, 79, 249, 89, 12 and 0, 0, 0, 5, 6 |
| Ape Escape in play, 265 frames | 17,611 | **2,335** (13.3%) | 6,983 and 6,494 |
| Spyro the Dragon, the ten anchors | 664 | **15** | 0 and 0 |
| Spyro the Dragon in play, 235 frames | 58,023 | **591** (1.0%) | 124 and 105 |
| Tekken 3's fights, 13 frames | 2,609 | none by the model; its pictures cannot be compared line for line (§32.2) | 468 and 440 |

The lines along Ape Escape's ledges and across Spyro's grass are gone from every frame looked at (§34.5). What is
left is in semi-transparent polygons, which are not filled: in Ape Escape a haze over the sky and the screen's fade
between scenes (95 of the 108 at frame 20,000 are in a fade to black, looked at; at 18,800 the pixels differ from the
console's by two steps of thirty-two in one channel), none of them a line of another surface's colour. §26.4's
counter barely moves in play because most of what it counts there was never a crack (§32.2).

**What the fill changes**, by who draws the pixel (the model of §32.1, the built form, Ape Escape's play and Spyro's):
of 259,886 and 243,012 pixels whose owner changes, 16,031 and 57,843 are cracks filled; 142,394 and 97,098 are taken
from another precise polygon that shares no whole edge with the filler (an outline over the scene behind it, a T
between two precise polygons); 23,478 and 33,281 from something not precise (a sprite, a cleared buffer, an integer
polygon); 77,983 and 54,790 from a precise polygon that shares an edge, at a corner or across a thin triangle (§33.3).
Every one of them is a pixel the console draws with the same polygon.

**On the eight discs' anchors**, every 300th frame to 3,000 by the press scripts [M]:

| Disc | Pixels the precise picture differs from the machine's, unfilled | Pixels the built fill changes | of them brought to the machine's colour | taken from it | The whole fill changes |
|---|---|---|---|---|---|
| Ape Escape | 121,450 | 5,688 | 4,799 | 15 | 10,212 |
| Crash Bandicoot | 44,045 | 6,475 | 5,838 | 12 | 9,439 |
| Mega Man Legends | 15,580 | 950 | 658 | 0 | 1,207 |
| Rayman | 3,497 | 514 | 222 | 0 | 837 |
| Silent Hill | 21,884 | 2,647 | 540 | 0 (370 before §33.4's second rule) | 5,054 |
| Spyro the Dragon | 97,619 | 6,620 | 3,567 | 16 | 19,142 |
| Castlevania, Tekken 3 to frame 3,000 | 0 | 0 | | | 0 |

"Brought to the machine's colour" is a changed pixel more than 60 from the machine's picture unfilled (the three
channels' differences summed) and within 30 filled; "taken from it" the reverse. Most of what the fill changes it
changes to what the console shows: 15,624 of 22,894 pixels on the six discs, against 43 the other way, which §34.5
lists. In play the same: of 15,374 pixels changed on 13 of Ape Escape's frames (every 1,000th from 7,600) 7,782
are brought to the machine's colour and 32 taken from it; of 12,779 on 12 of Spyro's, 7,889 and 13; of 17,872 on
Tekken 3's 13 fight frames, 11,391 and 14.

### 34.4 The decision

The rule set for this work: the fill becomes part of "Precise geometry", with no setting of its own, if it removes
the cracks and the turning grid's and the shared edges' figures stay within a small margin of E4's; if it brings
wobble back measurably, the better form is kept and what remains is put to the tester as a separate choice.

- **The whole fill fails it**: it closes the cracks and returns the shared edges to the console's places (0.556 px
  against 0.0035).
- **The fill with marks meets it**: the cracks are closed in every class (99 per cent of them in Ape Escape's lab and
  in Spyro's play, 87 in Ape Escape's play, the rest semi-transparent); the grid's vertices and texels are E4's to the
  digit and its shared edges within 0.0001 px of E4's. **It is built into "Precise geometry"**, with no setting.
- **What remains is the outline**, which the rule did not name and the measurement found: a precise polygon's top
  and left edges, where no precise polygon shares them, stand on the console's line. It is half of what the
  enhancement removes at such edges and none of what it removes inside a mesh, in a texture or at a vertex. **It is
  put to the tester rather than decided** (§36.2): the fill could be made a second switch under the same heading, off
  or on by default, at the cost of one flag in the renderer; the recommendation here is to keep it in, since a crack
  is a line of the wrong surface and an outline a pixel away is the console's own picture.

### 34.5 What the pictures showed

Looked at: the eight discs' anchors, unfilled and filled side by side with the changed pixels marked; Ape Escape's
play at frames 18,350, 18,800 and 20,000 and Spyro's at 7,250; and every anchor screened for a pixel the fill takes
away from the machine's colour (§34.3's last column), each one then traced to the polygons that draw it.

| Disc, frame | What showed | Why | Fate |
|---|---|---|---|
| Silent Hill, 3,000 | 370 bright pixels: dashes at the shelter's edges and along the ground at the walker's feet, twice the brightness of the fog about them | the game draws each polygon opaque and then semi-transparent over it; the fill took the first pass at a crack's pixel and not the second | **fixed**: §33.4's second rule; none after, and the test's twice-drawn floor holds it (121 pixels of its filled row before, none after) |
| Crash Bandicoot, 1,800 | 12 pixels: a blue-grey wedge of the title's model a pixel wider on its left for twelve lines | the console draws the wedge's edge there too, and then an overlay's opaque texel over it; in the precise picture the overlay's texel at that pixel is the transparent one of the row above [A, from the two polygons' words] | left: §26.4's "a texel or two from the affine picture", shown through a pixel the fill restored |
| Ape Escape, 2,700 | 15 grey pixels, scattered, on the lab's blue | filled pixels of a cut-out texture whose texel at the pixel's place is opaque where the console's affine texel is transparent [A] | left: the same |
| Spyro the Dragon, 2,100 to 3,000 | 4 pixels at each anchor, a shade off | the same | left |
| Ape Escape in play, 18,350 | the white and blue lines along the ledges gone; the ledges' and the ground's outlines a pixel fuller at the top and left, as the console draws them | the fill | as designed |
| Spyro the Dragon in play, 7,250 | the light dashes across the grass gone | the fill | as designed |
| Crash Bandicoot, 2,400 | holes of a pixel in the title's figure closed; its outline and its letters' a pixel fuller at the top and left | the fill | as designed |
| Rayman 2,100, Mega Man Legends 2,700 | a column at a panel's edge; outlines | the fill | as designed |
| Tekken 3's fights, 8,000 | the fighters' outlines a pixel fuller at the top and left; the health bars, the timer and the names untouched | the fill | as designed |
| The play frames screened (Ape Escape 13, Spyro 12, Tekken 3 13) | 59 pixels in all taken from the machine's colour, scattered, none a line | cut-out texels, as at Ape Escape's 2,700 [A] | left |

Looked for and not found: a silhouette thickened beyond the console's (it cannot be, by §33.1's second property); a
wrong texel along a filled edge of a texture drawn at one texel a pixel (the strips of §26.3, whose test now also
holds that no pixel the software rasteriser draws is left out); a change to a 2D picture or a HUD (the fill touches
only precise triangles: Castlevania's pictures and Tekken 3's first 3,000 frames are the machine's, and the 2D
program's are the software rasteriser's to the pixel, §35.1).

## 35. The gate kept, the cost and the tests

### 35.1 The gate

| Item | Result |
|---|---|
| E1's goldens with the setting off | **Equal** [M]: `programs.rs`' goldens, plain and with the test observer, the eight discs' 80 anchors among them; nothing off the precise path is changed but one mask in the copy's shader |
| The machine's goldens with the setting on | **Equal** [M]: the programs' 49, ps1-tests' 48, the discs' 80 anchors, picture, sound, state, VRAM and TTY (`precise.rs`' three golden tests) |
| Zero fraction is E3's equality | **Equal** [M]: pictures at every frame and VRAM at every anchor on the programs, ps1-tests and the eight discs to frame 3,000, 1,438,000 precise triangles of the discs through the record with its bounds in its flags and no triangle filled; the texture cache's two pinned counts, 128 and 3,680 halfwords, unchanged |
| E4's eight tests | **Pass** [M], with two of them changed and said so: the strips' test now holds that no pixel the software rasteriser draws is left out (17,026 pixels drawn against 16,705, the 321 more the four strips' first rows and the first strip's first column, filled; none past its strip's texels), where it held the two counts within 128; the grid's test gains §34.2's edges |
| The fill's four tests | **Pass** [M]: §33's floor and ring (386 pixels of the cleared buffer where the console draws before the fill, none after; the textured floor's filled row, 120 pixels, each the texel of its place; the twice-drawn floor's row the console's colour), the two quads sharing an edge (none of 2,118 pixels spilled; 18 with the whole fill), the semi-transparent pairs (19,627 pixels blended, none twice), the state between two quads (§33.5) |
| The 2D program | **Equal** to the software rasteriser's to the pixel, pictures and VRAM, as at E4 (§25.4): no polygon of it is precise, so none is filled |
| The kit on the published shape, the setting on | **Every case on every disc** [M]: `cargo build --profile dist` in `v1/`, the eight discs at 1,800 frames, four at a time, 351 s; C7's state 11,594,150 bytes loading and continuing, C8 across a save and load, C17 and C18; and with the setting off, 194 s |
| The shaders | `walk.spv` and `copy.spv` are the sources' compilation by WiseMan's shaderc (`DiamondShaderTests`) |
| WiseMan | 64 of 64, `DiamondShaderTests` (the shaders recompiled to
  the bytes on disk) and the three `DiamondPreciseGeometryTests` among them; and by `WindowFitAudit` and
  `GraphicsSettings`, the setting's hint having changed: 143 of 143. |

### 35.2 Cost

§27.1's instrument: `examples/bench` on links with every function aligned to 64 bytes, each build in its own target,
the builds alternating on one core, the fastest mean of five rounds, milliseconds of the emulation thread a frame,
the picture's wait timed with the frame. The base is WiseMan 8a90f127, E4 as merged, extracted outside the
repository. Ape Escape's first 3,000 frames (its lab, §32.2) replace Tekken 3's, which make no projection [M]:

| Run | Frames | Off: base | branch | | On: base | branch | The fill's | The picture's wait, base and branch |
|---|---|---|---|---|---|---|---|---|
| `gpu/benchmark` | 1,200 | 2.326 | 2.324 | -0.1% | 2.668 | 2.665 | -0.003 | 0.222, 0.222 |
| Crash Bandicoot | 3,000 | 2.315 | 2.322 | +0.3% | 2.786 | 2.802 | +0.016 | 0.202, 0.215 |
| Mega Man Legends | 3,000 | 1.806 | 1.802 | -0.2% | 2.153 | 2.182 | +0.029 | 0.159, 0.183 |
| Ape Escape | 3,000 | 2.162 | 2.157 | -0.2% | 2.536 | 2.555 | +0.019 | 0.207, 0.221 |
| Tekken 3's fights | 7,500 to 10,500 | 3.726 | 3.711 | -0.4% | 5.202 | 5.308 | +0.106 | 0.977, 1.072 |

Every pair's digests equal. **Off, the frame is the base's within the noise**: the fill is in the shader and in a
function only precise polygons reach. **On, the fill costs 0.02 to 0.03 ms a frame on the three discs and
0.11 ms in Tekken 3's fights**, 0.6 to 2.0 per cent of the frame with the setting on, and all of it is the picture's
wait (0.013 to 0.024 ms more, 0.095 in the fights): the device evaluating the third edge and the console's triangle
for the pixels about a precise triangle that it left at the first edge they failed. `gpu/benchmark` projects
nothing and is unchanged. A first series, before §33.4's second rule, gave the same within 0.02 ms (0.018, 0.020 and
0.008 on the discs, 0.088 in the fights). **On the handheld, estimated and not measured** [A, by §27.4's reasoning:
ten to fifteen times less compute throughput]: about a quarter of a millisecond a frame where a game draws at 240
lines, and a millisecond to a millisecond and a half in Tekken 3's fights, where §27.4 already puts the setting near
the frame's edge.

### 35.3 Tests [M, 2026-10-13]

All on the final head (WiseMan had not moved from 8a90f127, so there was nothing to merge).

- **`diamond`**, `cargo test --release` with `EMUSEN_DIAMOND_PS1TESTS`, `EMUSEN_DIAMOND_DISCS` and
  `EMUSEN_GPU_DEVICE=6800`: 167 unit tests, 81 program tests (E1's goldens plain and with the test observer, the
  discs' among them), E3's 3 renderer gates and **12 in `tests/precise.rs`** (E4's eight, the fill's four), all
  passing; and without the two corpora the same counts, the corpus tests standing down.
- **`precision`**: 6. **`diamond-renderer`**: 1. **`diamond-v1`**: 4, and 4 with `EMUSEN_GPU_DEVICE` naming no device.
- **The kit**, the corpus form on the eight discs at 1,800 frames on the `dist` library, on and off: every case on
  every disc, 351 s and 194 s; and Spyro the Dragon's on a library built with the marks out of the block, for §33.5.
- **WiseMan**, under the test lock with the corpus, filtered by `Diamond` and `PlayStation`: 64 of 64, `DiamondShaderTests` (the shaders recompiled to
  the bytes on disk) and the three `DiamondPreciseGeometryTests` among them; and by `WindowFitAudit` and
  `GraphicsSettings`, the setting's hint having changed: 143 of 143.
- **Not run**: `blackmoon-r3000`'s and `blackmoon-gte`'s suites, `emusen-gpu`'s and MarsRT's, none of them changed;
  the Vulkan validation layer over the changed walk; the full WiseMan suite; llvmpipe and the integrated card; the
  Mistress publish, whose library is the `dist` one the kit ran.

## 36. Predictions, what is put to the tester, what is left, and the reading log

### 36.1 Predictions and earlier statements

- **§26.4's "the same cause" for Spyro the Dragon is retired** [M, §32.3]. Ape Escape's cracks are T's against
  polygons the game divides on the processor; Spyro's run along whole shared edges of polygons whose vertices the layer
  lost at a shift, and of quads with one vertex no projection is behind. One remedy closes both; they are two causes.
- **§26.4's reading of Ape Escape's ring held**: its quads' words "do not trace to a projection: built or moved on
  the processor". Built: three of each quad's four, by the game's own division of near polygons.
- **§25.5's reading of Spyro's 464,915 "changed" words is retired, and of Tekken 3's held**: both were taken for
  vertices the game moves on the processor. Spyro's are the shifted word made in a register that held another
  projection's serial, tens to hundreds of pixels from it; the vertices it keeps shifted (§32.3) mostly never reach
  that count, their serial having been lost. Tekken 3's are within a few pixels of their projections: moved.
- **§21.4's "at most §21.2's two pixels and in practice under one" held** for the seam's width: the cracks are a
  pixel wide.
- **§22.4's argument that precise shared vertices leave no gap held as a measurement**: no pixel between two of the
  grid's quads was left undrawn in 120 frames of 24 shared edges, unfilled or filled (§34.2).
- **The risk stated with the proposal held, and more than it said** [M, §33.3]: that the later of two precise
  polygons sharing an edge would spill up to a pixel over the other "and may bring back some of the wobble". On the
  grid it brings back all of it at those edges, the console's truncation moving every polygon the same way.
- **Expected before it was measured, and held**: that the marks would leave the grid's shared edges E4's (0.0036
  against 0.0035 px), and that an outline's wobble would come half back (0.287 against the console's 0.584 px).
- **Not expected**: that a game would draw a polygon in two passes, and that filling one of them would show
  (Silent Hill, §34.5); that the kit's C7 would not need the marks in the state where a test of the crate's own does
  (§33.5).
- **P-E4c**, of Spyro's precise share, stays failed, with its reason now known and a remedy proposed (§32.4).

### 36.2 Put to the tester

1. **The outline.** The fill is in "Precise geometry" by the rule of §34.4. Its measured cost is at edges no precise
   polygon shares: their top and left sides stand on the console's line (the grid's outline 0.287 px from the ideal,
   moving 0.116 px a frame, against 0.006 and 0.011 unfilled). If steady outlines matter more than closed cracks in
   some game, the fill can be a second switch under Enhancements, on or off by default; it is one flag in the
   renderer's record and a setting in `diamond-v1`. Recommended: leave it in, with no switch.
2. **Following a shift** (§32.4), for Spyro the Dragon: one more move told by the processor's crate, which would
   take its precise polygons in play from 37 to about 93 per cent. It widens a written interface, so it waits for a
   yes. *Accepted as a trial and built 2026-10-14: 95.9 per cent (§55).*
3. **What to look at on the handheld**: Ape Escape's first level along the ledges and the ground's edges, where the
   lines were; the same edges' top and left sides as the camera turns, for the outline's movement; Spyro the Dragon's
   grass in its first world; Silent Hill's street in the fog, for any bright dash at a polygon's edge (§34.5's
   defect, fixed); and the frame's time, which the desktop puts at 0.02 to 0.03 ms more than E4's where a game draws
   at 240 lines and 0.11 ms in Tekken 3's fights, and §35.2 estimates at a quarter of a millisecond and about one and
   a quarter there.

### 36.3 What is left, and why

- **Cracks in semi-transparent polygons** are not filled (§33.4): 2,335 pixels of Ape Escape's 265 frames in play
  and 591 of Spyro's 235, in a haze, a fade, a shadow. Filling them where no opaque polygon filled first would blend
  twice along every T between two such polygons.
- **The corners of shared edges and triangles under two pixels thin** still take the console's pixels from a precise
  neighbour (§33.3): 77,983 pixels of Ape Escape's play, 294 a frame. The two-edge mark is the form to try.
- **Ape Escape's divided polygons and Spyro's lost ones stay integer**: their cracks are closed, their wobble is the
  console's. Spyro's is §36.2's second item; Ape Escape's has no exact remedy (§32.4).
- **Spyro's vertex with no projection behind it** (§32.3) is not traced to its cause.
- **Tekken 3's fights** are counted by the model alone, 2,609 cracks on 13 frames and none left; their pictures
  were looked at and not compared pixel for pixel (§32.2).
- **At N×** (E5): the marks are the mirror's and grow with it; the fill's rule is a sample's, the console's own
  triangle at the samples of its pixels. Neither is built for N.
- **The handheld**: not connected and not measured; §35.2's figures are the desktop's.

### 36.4 Reading log (the crack fill)

**No emulator's source, shader or snippet, no description of PGXP or of any emulator's precision, gap or crack
handling, and no other PlayStation renderer's code were opened**, with or without code: nothing of DuckStation's,
Beetle PSX's (parallel-psx and its rsx among it), PCSX's, PCSX-Redux's, SwanStation's, ePSXe's plugins or any
fork's. The remedy is from the console's own rule as Diamond models it: every polygon drawn at whole-pixel vertices.
No firmware, listing or trace of one; the player's BIOS folders were not read; nothing under `kernel/` or
`kernel-harness/` was opened or changed. psx-spx was not opened. No published prose on the subject was read, with or
without code. The handheld was not connected to.

**Two games' code was read**, on the open kernel, from the running machine's RAM by the run tool's `--disasm` and the
same disassembler in a scratch test, after the scratch layer had named the instruction that stored each vertex word:
Ape Escape's about 80016060h to 800161C4h (where it projects), 8001B3A0h to 8001B4B4h and 8001D800h to 8001E820h
(where it divides and emits); Spyro the Dragon's about 8004EDD0h to 8004EF3Ch, 800262E0h to 80026354h, 80026540h to
800265CCh, 80026960h to 800269ACh, 80026D40h to 80026DECh, 80020340h to 800203A4h, 8001FC80h to 8001FDC4h and a dozen
words at 80027B00h and 80029100h. What §32.3 says of them is what those listings show; no listing, picture, state or
disc image is in the repository. ps1-tests' programs were run as built.

Opened, all EmuSen's own: this page's §1, §21 to §31 and its outline; `Diamond_Plan.md` §6.3's row;
`Diamond_Players.md` §15; `BlackMoon_R3000.md` §14; `Diamond_Native.md` §25 and §26; the indexes;
`EmuSen_Settings_Reference.md` §4.110. The source of `precision` and of `diamond-renderer` whole, shaders among it;
of `diamond`, `layer.rs`' interface, `gpu/decode.rs`, `gpu/raster.rs`' setup and triangle, `gpu/draw.rs`' paint,
`gpu/vram.rs`' rectangles, `gpu/display.rs`' scan, the state writer's layout, `tests/precise.rs`, `tests/support/`,
`examples/run.rs` and `examples/bench.rs`; `blackmoon-r3000`'s moves and `blackmoon-gte`'s projection; `diamond-v1`'s
setting; WiseMan's `DiamondPreciseGeometryTests.cs`; E4's measurement scripts in the probe.

---

## 37. Stage E5: what it must do, decided before it was built

*Written 2026-10-14, before any of E5's code, after the tester's go of 2026-10-13.* The plan's row: "Internal
resolution, true colour, texture filtering. Gate: each on the bench discs at 2× and 4×, its cost measured on the
desktop, the laptop and the handheld; framebuffer-as-texture effects checked on the bench." The tester set the order:
speed before features, since E4 measured the picture's wait as most of the renderer's cost and estimated it at about
10 ms a frame on the handheld in Tekken 3's interlaced fights (§27.4); then the three features, each its own setting
under Enhancements; the laptop of the row is not part of this stage. The seams are handled by the crack fill (§33),
which at N× must scale with the rest (§38.7). The plan's rules (§6.2 there) stand unchanged: off is the accurate path
to the bit; the software rasteriser keeps VRAM, the renderer owns only the presented picture; each enhancement is a
setting with `effect: enhancement`, its `hardware` value the default and a cost in a player's words; states stay
compatible.

What was known before any change [M, the bench on WiseMan 137a039a, `--precise`, the RX 6800]: in Tekken 3's fights
(frames 7,500 to 10,500 of the run to 10,500) the renderer made 110,557 stretches over the 10,500 frames, **96,170 of
them ended by a display line** whose pixels met what the stretch had drawn, 3,866 by a copy, and none of the rest by a
palette, a transfer or a texel more than a dozen times; it loaded 840,946 palettes and waited 1.07 ms a frame for the
picture. Crash Bandicoot's first 3,000 frames made 3,003 stretches, one a frame, three of them ended by anything but
the frame. So the fights' stretches are the display's: at 480 interlaced lines the game draws the field the display is
not reading into the very rows-of-blocks the display reads the other field from, and the stretch's written set, kept in
blocks of 16 by 16, cannot tell an odd row from an even one.

## 38. Speed first [C]

### 38.1 The picture handed a frame late, and the machine never waiting

**The form.** At the end of each frame's `advance` the renderer submits everything planned in the frame as the
device's one pending submission (`emusen-gpu`'s `submit_pending`, `EmuSen_Gpu.md` §3; Mars's form, `Mars_Gpu.md`
§14.2), ending with a copy of its picture into one of two host-visible picture slots, the frame's parity choosing
which. The host is handed the other slot: **the picture as it stood at the end of the frame before.** The submission
of the frame after waits for this one before it records, which is the only wait left on the emulation thread, and it
falls a whole emulated frame after the submission it waits for (2 to 5 ms on the desktop, about 16.7 in play), where the
device's drawing took 0.13 to 0.98 ms (§27.3): it waits only when the device cannot keep up with the machine at all.

- **Which picture, and the rule.** Always the previous frame's, never "the newest one finished": a picture chosen by
  what the device happened to finish would make two identical runs hand different pictures, and the kit's C8 forbids
  that. The latency is exactly one frame, at every frame, at every scale.
- **What the host waits for.** Nothing. The picture handed is in host memory before `advance` returns, since the
  submission that wrote it has been waited for by then.
- **The host's staging, twice.** The staged arrays (records, tile entries, tiles, palette loads, lines, transfer
  pixels) are host buffers the pending submission copies from; the host writes the next frame's into a second set
  while the first is in flight, and the two alternate at each submission, a submission made when an array fills
  mid-frame included. The device's copies of them stay single: every command ends in a barrier whose second scope is
  everything later on the queue (§14.2 of `Mars_Gpu.md`), so the next submission's copies wait for this one's walk.
- **Waited submissions stay** for whoever asks for the picture or VRAM now: the gates, the tools, `settle` before a
  save, and a seed. Each first waits for the pending submission.
- **The kit's cases that compare pictures** (C7 and C8: a machine given another's state continues as it, picture for
  picture, from its first frame). After a load the first picture handed must be the one the saver hands after its
  next frame: the picture as it stood when the state was saved. `RNDR` already holds the picture read back by
  `settle` after everything planned is drawn, which is exactly that picture; a seed from a state therefore puts it in
  the slot the next frame hands. So the kit's cases need no change, and C8's two machines hand the same pictures
  because both hand the frame before. A seed from the machine (attaching, or a state with no block) puts the GPU's
  own picture there: the first frame after turning an enhancement on shows the console's picture of the frame before,
  as a frontend that presents a frame late would.
- **Rendering skipped** (`set_options` bit 0): the frame's work is submitted all the same, since the mirror must stay
  in step, and only the host does not look; skipping saves nothing here but was never a saving for the renderer.

The cost of a frame of latency is a frame of latency, about 17 ms at 60 frames a second, added only while the renderer
draws the picture. It is stated in every setting's cost. A waited form beside it, a choice of latency for a player
whose device is fast, was considered and not built: the C# frontends present a frame late already when their deferred
presentation is on (Mars's `DeferredPresentation`), and §15 Q12 of `EmuSen_CoreAPI.md` puts such a choice under
`effect: latency`, a setting of its own, which this stage does not need to decide.

### 38.2 Stretches: the written set by row parity

A display line at row `y` ends its stretch only when the stretch has drawn into the blocks of row `y`. **The written set
is kept twice, once for the even rows and once for the odd** [C]: a record that leaves the interlaced field's lines
alone (`SKIP`, a polygon, rectangle, line or fill drawn with drawing to the displayed field prohibited) marks only the
set of the parity it draws; every other record, transfer and copy marks both. A display line asks only the set of its
own row's parity; a palette load and a record's texels ask the union, as before. The dependency is unchanged and only
finer, so the pictures are those of the coarser planning, which the gates hold (§42). Predicted: Tekken 3's fights fall
from about 10.5 stretches a frame to about 1.4, and the fights' wait with them.

**Palette loads are not reduced** [C, to be revisited by measurement]. A load is one workgroup at its stretch's start,
and a stretch's loads are one dispatch; after the parity split the fights' 80 loads a frame should share one or two
dispatches. Sharing a slot between two loads of one palette would need every write since the first load checked
against it; it is built only if the fights' wait, measured after §38.1 and this, still shows the loads.

### 38.3 The plain frame for a renderer alone: `Layer::moves`

A layer says whether it wants the processor's moves and the GTE's projections: `Layer::moves`, `true` by default, which
E1's test observer and the precision layer keep; the renderer answers `false`. `Attached` reads it once, when the layer
is set, and `Machine::advance` takes the layered frame only for a layer that wants it. The GP0 receipts and the GPU's
own events are told in both frames, as they always were. This saves §18.3's middle column, 0.04 to 0.15 ms a frame,
whenever internal resolution, true colour or filtering is on without precise geometry. It is an addition to a trait the
machine crate owns, with a default, and changes nothing a layer already did.

## 39. Internal resolution [C]

### 39.1 Where an N× sample lies

At N× the mirror holds `1024N × 512N` samples, a 32-bit word each as at 1× (the console's halfword low, the crack
fill's mark high, §33.3), and the console's pixel `(x, y)` is the block of samples `(Nx + i, Ny + j)`, `0 ≤ i, j < N`.

- **Coverage is tested at the point `(x + i/N, y + j/N)`**, the console's own grid refined: the block's first sample is
  the console's point, so a block's first sample is covered exactly when the console covers the pixel, and a primitive
  with whole-pixel edges (a sprite, a rectangle, a quad of integer vertices, a fill)
  covers whole blocks exactly. 2D stays aligned: a quad and a rectangle of the same corners cover the same samples, and
  every UI drawn either way lands on the console's pixel grid scaled. A sample centred in its block (`x − 1/2 +
  (i + 1/2)/N`) was set aside: whole-pixel edges would cut every block in half, and a sprite of quads would sit half a
  pixel from the same sprite of rectangles.
- **Colours and texture coordinates are evaluated at the block's centre about the console's point**, `(x + (i + 1/2)/N
  − 1/2, …)`: the N² samples of a pixel are centred on the point where the console evaluated the pixel, so a texture
  drawn one texel a pixel (a sprite of quads, a menu, a font) shows each texel as a whole block, crisp, the console's
  texel in every sample of it, and never the texel past its edge (§26.3's case cannot arise at one texel a pixel: the
  last block's coordinate stays under the last texel by `(N + 1)/(2N)` of a texel). Smooth shading and minified textures
  gain the samples between.
- **In 256ths of a pixel**, the units of the precise record (§22.2), the coverage point is `256X/N`, a whole number only
  for N of 1, 2 and 4; the edge functions are therefore evaluated multiplied by N, `dX·(256Y − N·aY) − dY·(256X −
  N·aX)`, exact in the pairs of 32-bit words E4 already uses (at 4× they reach 2⁴⁰), against the same bound of 0 or 1,
  since multiplying by N keeps a sign and keeps "greater than zero". The attribute point is rounded to the nearest 256th
  (at 3× a 768th of a pixel at most, far below any attribute's step). At N = 1 every formula is E3's and E4's.

### 39.2 Each kind of record at N×

| Record | At N× |
|---|---|
| Triangle, integer vertices | the console's edge functions at the N× coverage point, its gradients at the attribute point; at 1× E3's arithmetic. E4's suggestion to send every integer triangle through the precise record was weighed: the same evaluation, but a precise record writes marks (§33.3), which an integer polygon must not, so the integer record is kept and taught N |
| Precise triangle | its fine edge functions at the coverage point, its interpolants and its division a pixel at the attribute point, its texel hold as at 1× |
| Rectangle (a sprite) | the console's pixel, its texel and its pairs rule, in every sample of the block: a sprite is a texel a pixel by definition and is scaled, never resampled |
| Line | the console's pixels, each a block: a line's pixels are its stepping, and a frame of lines must still meet the rectangles and sprites it encloses; a line keeps the console's jaggedness, scaled |
| Fill | the console's rectangle scaled, its skipped lines by the console's row |
| CPU-to-VRAM transfer | each pixel a block, under the mask bit per sample |
| VRAM-to-VRAM copy | the N× rectangle copied sample for sample, so a copy of something drawn keeps its detail; overlapping copies in steps of `N·d` rows, `vram.rs`'s row order scaled |
| Palette load | the first sample of each entry's block, the console's point |
| Display line | N rows of the picture from N rows of the mirror (§39.4) |

**Dither at N×** stays the console's matrix by the console's pixel, so a dithered surface shows the console's pattern
scaled; true colour (§40) is the remedy for a player who does not want it, and keeping the two apart keeps each
setting's effect what it says.

**The mask bit and semi-transparency per sample**: each sample is read, tested and blended for itself, as each pixel is
at 1×; a primitive drawn over a half-masked block draws the unmasked samples.

### 39.3 A texture read from what was drawn

The console textures from VRAM, and VRAM is where it draws: a texture page can be a picture the game drew (a screen
copied and drawn back distorted, a reflection, a menu's blurred backdrop, a battle's swirl). At N× the renderer's mirror
holds that picture at N×, so:

- **A 15-bit texel is read at the sample its coordinate falls in**: column `N·page_x + ⌊(u + 1/2)·N⌋` and the like for
  v, where `u` is the interpolated coordinate before the console's rounding, `⌊u + 1/2⌋` the texel the console reads.
  Within the texel the sample is chosen by the fraction, so the drawn picture is read back at N× with no seam between
  texels (the column is continuous and increasing in `u`) and no shrinkage (texel `t`'s N samples are exactly its
  block). The texture window and a precise triangle's texel hold act on the console's texel, the fraction choosing the
  sample within it. A texture uploaded by a transfer is a block of one value, so its samples read as the console's.
- **A 4- or 8-bit texel and a palette** are indices packed into halfwords, which no drawing at N× makes meaningfully;
  they are read at the first sample of the block, the console's.

What it cannot recover: a picture the game moves through the processor (VRAM to CPU and back), which crosses the
accurate VRAM and comes back as a transfer at 1×; such an effect is correct and as coarse as the console's.

### 39.4 The picture at N×, and the frame the host receives

A display line read from VRAM's row `y` at column `x`, `w` dots wide, is N rows of the picture, `N·row + j` read from
the mirror's row `N·y + j`, `N·w` samples each from column `N·x`. A 24-bit line packs three bytes a dot across
halfwords: dot `i`'s bytes are read from the halfwords as at 1×, each from the first sample of its block in row
`N·y + j` (24-bit pictures are a movie's, uploaded at 1×). In 480-line interlace a line is the field's row `2r + f`,
so its N rows are woven with the other field's N rows, which keep what they held, as the console's picture keeps the
other field (`Diamond_GPU.md` §7). The picture is `N·w` by `N·h`, at most 2,560 by 2,304.

**The host is handed the picture at N×** [C], its size in `frame_info` and announced by `GEOMETRY` when it changes, as
`EmuSen_CoreAPI.md` §6.6 provides. Reduced to the console's size by an average, as Mars's antialiasing does
(`Mars_Gpu.md` §15), it would be supersampling, which is a different enhancement with its own setting, and the size a
reduction should go to is the window's, which the core does not know. The largest frame handed is Tekken 3's 512 by 480
at 4×, 2,048 by 1,920 or 15.7 MB; it is read back into the host's slot by the pending submission (§38.1), so the
emulation thread does not wait for it, and `frame_copy` copies it once more into the host's buffer. Machine info's
`max_width` and `max_height` become 2,560 and 2,304, the largest picture of the largest scale, since a host may size its
buffer from them once.

**Left for the tester** [proposed, not built]: `EmuSen_CoreAPI.md` §6.6 reserves a word of `frame_info` for "the
console's own lines" of a frame drawn at a multiple, to be named "in the minor that first has a core to fill it", so
that a CRT filter draws the console's lines and not the frame's rows. Diamond at N× is that core. Naming the word is a
change of the ABI's minor (the header, `emusen-native`, the C# host, the baseline and the kit); it is put to the tester
and not made here. Until it is, a CRT filter over Diamond at N× draws N times the lines, and the player's page says so.

### 39.5 The largest scale, and why

**1×, 2×, 3× and 4×**, Mars's choices and Mars's label, "Internal resolution" (`MarsCore.cs`' `RenderScale`). At 4× the
mirror is 32 MiB, its copy scratch another 32, the picture 23.6 MB and its two host slots the same each; with true colour
a second plane of 32 MiB (§40): about 180 MB of the device's memory and 80 of the host's, which the handheld's 16 GB
shared memory holds. The drawing costs N² the samples. 5× would draw a 640-dot picture 3,200 wide, past any screen the
player has, for 25 times the work; it is not offered. Each workgroup of the walk is one N×-sample tile of 8 by 8, N² of
them for each console tile, dispatched as a second dimension so that no dispatch exceeds Vulkan's guaranteed 65,535
workgroups in a dimension; the host's binning stays in console tiles, so the host's work does not grow with N.

### 39.6 Changing the scale, and a seed at N×

The scale, true colour and filtering are the renderer's options, applied between frames by detaching it, rebuilding
its buffers at the new size on the device it has, and attaching it again, which seeds it (§16.6): the mirror is the
GPU's VRAM with each halfword a block, the picture the display's with each dot a block. What was drawn at the old scale
is dropped, and the next frames redraw it; a game redraws its screen within a frame or two, and a texture page it drew
long before stays at 1× until it is drawn again.

### 39.7 The crack fill at N×

The fill's rule is a sample's (§36.3): a precise triangle also covers each sample its console triangle covers, that
triangle's edge functions evaluated at the N× coverage point as an integer triangle's are, and a filled sample takes its
place in the console triangle at that sample to the same place in the precise one. Marks are per sample, as the mirror's
high half is per sample. So two precise triangles sharing an edge keep it where they draw it, sample by sample, and a
crack between a precise polygon and an integer one is closed at the samples where the integer triangle at N× would have
covered it. Measured at 2× and 4× against §32.2's counts and §34.2's shared-edge figure (§42).

## 40. True colour [C]

**The plan's two forms**: the mirror kept at 24 bits, blending and shading at full precision; or only the dither left
out. The second alone is worse than the console: without its dither a gradient drawn at five bits a channel bands, and
the dither exists to hide exactly that. **Chosen: the first.**

- **Where the bits are kept.** The mirror's word is full, the console's halfword and the crack fill's mark. A second
  plane of one word a sample holds each pixel's three lowest bits a channel, the bits the console's `>> 3` drops; it
  exists only while true colour is on. A pixel's channel is then `c·8 + low`, eight bits read as the console's five with
  three more below them.
- **Drawing.** The colour a primitive makes is kept at eight bits (`(rgb × texel) >> 4` as the console computes it, a
  Gouraud colour as interpolated, a raw texel's eight bits), with no dither; a blend reads the destination at eight
  bits and blends at eight (`B/2 + F/2`, `B + F`, `B − F`, `B + F/4`, saturated at 255); the halfword keeps the top five
  bits and the plane the rest. A texel read from a drawn area brings its plane's bits with it; transparency, the
  semi-transparency flag and the mask bit are the halfword's, as the console tests them. A fill keeps its colour word's
  low bits; a transfer writes none; a copy moves them.
- **Shown.** A channel of eight bits `v` is displayed as `min(255, (33·v) >> 5)`, which for `v = 8c` is exactly the
  console's expansion `c<<3 | c>>2`: a picture drawn without the plane, or a pixel the plane leaves at zero, shows as the
  console's to the bit, and a smooth gradient keeps the console's brightness. 24-bit display lines read bytes and ignore
  the plane.
- **What the machine sees**: nothing; the accurate VRAM keeps its 15-bit values, and readbacks are unaffected.

**What a player loses**: the console's dither, which some games' art leans on as a texture (a grainy fog, a dithered
sky that reads as noise on a CRT). Their gradients become smooth where the console's were grained; nothing bands that
did not band before, since every channel carries more bits than the console's, not fewer. Banding is measured on a
shaded gradient with and without (§42).

## 41. Texture filtering [C]

- **Bilinear after the palette.** A texel's coordinate before the console's rounding, `u`, lies between texels
  `⌊u⌋` and `⌊u⌋ + 1` (the console reads `⌊u + 1/2⌋`, so texel `t`'s centre is at `t`); the four texels about `(u, v)`
  are each read through the window, the page and the palette, and their colours mixed by the fractions, at eight bits a
  channel. At `u` whole the result is the console's texel; at 1× a texture drawn one texel a pixel has every `u` whole
  and is unchanged.
- **Clamped to the primitive's texture rectangle.** The four texels are held within the texels the primitive's own
  vertices reach (for a precise triangle its texel hold, §26.3; for any other its vertices' least and greatest
  coordinate, short of the end its coordinates run toward), so a polygon cut from a packed page never mixes in its
  neighbour's texels. **Within a texture window** the four are each passed through the window instead, which wraps them
  inside the repeating tile as the console's coordinates wrap, and the rectangle is not applied, the window being the
  game's own statement of what repeats.
- **Colour 0.** Whether a pixel is drawn, and whether it blends, is the nearest texel's, the console's own: a cut-out's
  silhouette is the console's. The colour is mixed from those of the four that are not transparent, their weights
  renormalised, so no dark fringe of colour 0 bleeds into an edge.
- **Rectangles and sprites.** A choice of three: **Off** (the console's); **3D polygons**, which filters textured
  polygons but not rectangles, and not a polygon drawn one texel a pixel and unrotated (its two gradients 1 and 0 in
  the console's 12-bit units, its vertices whole): the sprites and menus a game builds of polygons; and **Everything**.
  The second keeps UI and sprites crisp and is the one listed first after Off. A 4- or 8-bit texture drawn into at N×
  is read at the console's samples (§39.3) and filtered between console texels; a 15-bit one drawn at N× is filtered
  between the N× samples.

**Known breakage**: text and HUDs drawn as rotated or scaled polygons blur under either filtering choice; a texture
whose texels are meant as pixels (pixel art on a 3D surface) softens; texture atlases packed without a gap bleed at
polygon edges a game did not cut along its own coordinates' range (the clamp covers what the vertices reach, no more);
alpha-tested cut-outs keep the console's hard silhouette, by the colour-0 rule.

## 42. Composition, settings, state, the gate

### 42.1 The renderer's options and the settings

`diamond-renderer` gains `Options { scale, colour, filter }`, given at `Renderer::new` and changed between frames by
`Renderer::configure` (§39.6), and `Draw`'s drawer and the precision layer are paired or not as before. `diamond-v1`
offers four enhancements under Enhancements, each `effect: enhancement`, `scope: run`, no restart, default the
hardware's:

| Key | Label | Values | Default |
|---|---|---|---|
| `precision` | Precise geometry | switch | off |
| `resolution` | Internal resolution | `1` "1× (the console's)", `2` "2×", `3` "3×", `4` "4×" | 1 |
| `colour` | True colour | switch | off |
| `filtering` | Texture filtering | `off` "Off (the console's)", `3d` "3D polygons", `all` "Everything" | off |

The renderer is attached whenever any of the four is not its hardware value: alone (with the plain frame, §38.3) unless
precise geometry is on, paired with the precision layer when it is. Any combination works, the four being options of
one renderer and one switch of the pairing. Each setting's cost names a graphics card, the frame of latency (§38.1), its
own price (N² the drawing at N×; little for the other two) and its known breakage, in a player's words. Without a
usable Vulkan device each is accepted and noted as "Precise geometry" is (§24.2), the console's picture shown.
`STATE_SIZE` is raised by `core_exports!` when the state's size moves, `GEOMETRY` when the frame's does.

### 42.2 State: keep the N× mirror, or seed and redraw

`RNDR` at N× holds what the renderer cannot get back from the machine: the mirror's halfwords and marks at N× (N²
times 1 MiB each), the picture at the largest N× size (23.6 MB at 4×), the palette, and with true colour the plane
(N² times 1 MiB, a halfword a sample for its nine bits). At 4× with everything on a state is about 72 MB
against 11.6 at E4 with the crack fill. **The alternative** is a block holding only what is small (the palette and the
picture's size) and a load that seeds the mirror and the picture from the accurate VRAM and the display, as an attach
does: a state of the accurate size plus a few bytes, the first pictures after a load at 1× detail until the game redraws.

*Superseded 2026-10-14: the seeded form is built (§52, §53).* **Built: the kept form** [C], because the kit's C7 and C8 require a loaded machine to hand the saver's pictures from its
first frame, and a seeded picture is the console's at that frame, not the renderer's (the rule set for this stage: build
the form the kit passes as it stands). **Measured for both** (§47.2): the state's size, the bytes a rewind's
delta between consecutive states holds, and on the bench discs how many frames a seeded picture takes to become, pixel
for pixel, the picture the kept form draws. **Proposed to the tester if the second form wins on those figures**: that
the kit's C7 and C8 compare an `enhancement` setting's pictures after a load from the frame the core names (a field of
machine info, "pictures settle after N frames"), or compare only its sound and state, while C6 and C8's first half keep
checking pictures in a run without a load. That is a change to the kit's cases for every core, not to Diamond.

The block's form is fixed for a configuration (scale, true colour, precise geometry): a state of another configuration
fails its length and its part starts seeded, as E1's rule for an unreadable block has it; a state of E4 at 1× with
precise geometry on loads as before, its layout unchanged at that configuration.

### 42.3 The gate, as it will be run

- **Off is the accurate path**: E1's goldens with all four off; with any on, and with all on at 4×, the machine's
  goldens (state's accurate blocks, sound, TTY, VRAM, the machine's own picture) on the programs, ps1-tests and the
  eight discs.
- **1× is E3's equality**: E3's renderer gates and E4's zero-fraction gates unchanged with the new code, at 1× with true
  colour and filtering off, the texture cache's two pinned counts the one difference.
- **Synthetic programs** (`tests/scaled.rs`, `tests/colour.rs`, `tests/filtered.rs`): at 2×, 3× and 4× a known scene
  against a reference computed in the test (triangles' edges at the N× samples, a grid of sprites and rectangles that
  must stay on the scaled grid with every block one texel, a scene drawn and then textured from, a VRAM copy of a drawn
  area, a masked area and a semi-transparent one, sample by sample); the crack fill's tests at N×; a shaded gradient's
  distinct levels and its banding with true colour on and off; filtering on a packed page (no texel of the neighbour),
  colour-0 edges (the silhouette the console's) and a texture window (wrapping inside the tile).
- **The eight bench discs at 2× and 4×** along their golden anchors with their press scripts: the machine's goldens
  unchanged; the pictures looked at, every visible defect listed with its frame and fate; the crack counts of §32.2 and
  the grid's edges of §34.2 at 2× and 4×.
- **The player's path**: the kit on the published library with each setting on and with all on, the eight discs at
  1,800 frames; a Mistress publish carrying the library with the new settings; WiseMan: each setting under Enhancements
  with its cost, the window fitting at both audited sizes, and the settings changed between frames through the host
  changing only the picture.
- **Cost**, by §27.1's instrument: everything off against the base; precise geometry alone before and after the speed
  work; 1× to 4× with and without precise geometry, true colour and filtering; state sizes; the handheld measured if it
  is reachable and idle, estimated otherwise.

### 42.4 Predictions [recorded before the code]

- **P-E5a.** With the picture handed a frame late, the emulation thread's cost of the renderer falls by about its wait:
  precise geometry at 1× within 0.1 ms of the precision layer with a drawer that draws nothing on the five items, where
  §27.3 had it 0.2 to 1.0 ms above.
- **P-E5b.** The parity split takes Tekken 3's fights from 10.5 stretches a frame to under 2, and its waited picture
  under 0.5 ms.
- **P-E5c.** With `Layer::moves`, the renderer alone costs the plain frame plus its host work: within 0.15 ms of the
  plain frame on the three discs.
- **P-E5d.** At 4×, with the wait off the thread, the emulation thread stays within 0.5 ms of 1× on the desktop on
  every item but the fights; the device's own time at 4× (the wait, measured in the waited form) is under 4 ms a frame
  on the RX 6800 in the fights.
- **P-E5e.** At 1× with true colour and filtering off every picture of E3's and E4's gates is unchanged, and with any
  setting on every machine golden holds.
- **P-E5f.** True colour gives a shaded gradient across 256 pixels at least 200 distinct levels a channel where the
  console's dithered picture has 32 levels and its grain.

## 43. The speed work, as built

*Built and measured 2026-10-14, the RX 6800 by `EMUSEN_GPU_DEVICE=6800`.* What was built is §38 as designed.

### 43.1 The picture a frame late

`Renderer::frame_end` closes the frame's last stretch and sends everything planned as the device's pending submission,
ending in a copy of the picture into one of two host slots, and the host is handed the other (`Renderer::handed`);
`diamond-v1` calls it after every `advance` and every debugger frame, skipped rendering included, and its `frame` and
`frame_info` read the handed slot. A submission made because an array filled in the middle of a frame is left pending
too (`flush`), so no submission on the frame's path is waited for; the waited one stays for `picture`, `vram`,
`samples`, `settle` and a seed, each of which waits for the pending one first. The host's staging is two sets of the six
arrays, 16.6 MB more host memory, the device's copy of them one. A seed, from the machine or from a state's block, puts
its picture in the slot the next frame hands (§38.1), so that a machine given a state hands, from its first frame, the
picture the machine that saved it hands.

The kit's C7 and C8 compare pictures across a load and between two machines. Both pass on the eight discs with every
setting (§48), with no change to the kit: the latency is the same frame for both machines, and the block's picture is
the picture handed next.

### 43.2 Stretches by row parity

`blocks::Written` keeps a stretch's drawn blocks twice, by the parity of the rows drawn; a record with `SKIP` marks the
parity it draws, a display line asks the parity of its row. The renderer's counts now say why each stretch ended
(`Counts::ends`: a palette, a copy, a transfer's row, a display line, a record's texels, a record over texels read). On
the bench run of Tekken 3 to frame 10,500 with precise geometry [M]:

| | Stretches | ended by a display line | by a copy | a frame | The picture waited for (`--waited`), the fights |
|---|---|---|---|---|---|
| Before (WiseMan 137a039a) | 110,557 | 96,170 | 3,866 | 10.5 | 1.07 ms |
| After | 14,550 | 7 | 3,866 | 1.39 | 0.50 ms |

What is left of the fights' stretches is a copy every three frames and one stretch a frame. The 840,946 palette loads
(80 a frame) now go to the device in one or two dispatches a frame, and the wait that remains is the device's drawing
of 480 interlaced lines and the readback, not the loads' dispatches: a slot shared between loads of one palette, which
§38.2 left to this measurement, was not built.

### 43.3 The plain frame for a renderer alone

`Layer::moves` (default `true`), read once by `Attached::set`; the renderer answers `false`, so the machine runs the
plain frame for it and the layered one only for the precision layer, E1's test observer or any layer that keeps the
default. The machine's goldens are the same with the renderer attached alone (E3's gates) and with it paired (E4's).
The cost of each piece is §50.2.

## 44. Internal resolution, as built

*Built 2026-10-14.* §39 as designed, with one defect found by the gate and fixed (§44.2).

### 44.1 What was built

- **The device.** The mirror, its copy scratch and the picture at N² the samples, the picture's host slots and the
  readback buffer with them (`Held::new(scale, options)`); `Renderer::configure` rebuilds them for another scale on the
  device it has (`Device::rebuild`). The walk's workgroup is an 8-by-8 tile of samples, dispatched as (console tiles,
  N²), its push constants the first tile, the scale and the options; the scan's (lines, N), the copy's (runs of 64
  samples, rows × N); the palette loads read each entry's first sample.
- **The walk.** An integer triangle's edge functions at the coverage point `(x + i/N, y + j/N)` multiplied by N, its
  attributes at the attribute point, `(2d − N + 1)/2N` in pixels about the console's point, rounded to a 256th, by the
  fine interpolation, which at N = 1 is E3's to the bit; a precise triangle the same with its fine edges, its division a
  pixel at the attribute point's weights, and the fill's console triangle at the sample (§44.4). Rectangles, lines,
  fills and transfers act on the console's pixel, each sample of its block; a sprite reads the sample of its texel
  that the pixel's sample is, mirrored with a flip. A 15-bit texel is read at the sample its coordinate's fraction
  names; a palette's texel and a palette at the first sample. The dither is the console's matrix by the console's pixel.
- **The integer record kept** (§39.2): E4's suggestion of every triangle through the precise record would have had
  integer polygons write the crack fill's marks.
- **The state.** `RNDR` holds the mirror's halfwords and, paired, the marks at the scale, and the picture at the largest
  size of the scale; its form is fixed by the scale, true colour and the pairing, so a state of another form seeds the
  renderer from the machine (§47).
- **The tools.** `run` and `bench` take `--scale N`; the renderer's `samples` reads the mirror back for the tests.

### 44.2 A coordinate that wrapped, found by the grid and fixed

**Shown.** The turning grid's shared edges, graded sample by sample at N× (§44.4), stood 0.0143 px from the ideal at 2×
and 0.0822 at 4×, where 1× gives 0.0035 [M]. With the fill turned off in a scratch build the figure was the same, so the
fill was not the cause. At N× a sample's attribute point lies up to `(N − 1)/2N` of a pixel up and to the left of its
coverage point, so at a polygon's top and left edges it lies a little outside the polygon, where the interpolated
coordinate runs a little past the vertices' range; a coordinate below zero was cut to eight bits as the console cuts one,
to 255, and the precise triangle's texel hold then took it to the far end of its range. The quad's texels nearest the
edge came from its other side, which the grid's grading reads as the neighbour's pixel. The console never meets this:
its point is inside every pixel it covers. It would show on any texture as a line of the texture's far side along a
polygon's top and left edges.

**Fixed** [C]: at N > 1 a coordinate is held within the vertices' least and greatest before the console's eight-bit cut
(`within` in `walk.comp`; the integer record carries the range in its word 9, the precise record has its vertices'
coordinates already); at N = 1 nothing changes. After it the grid's shared edges stand 0.0035 px from the ideal at 2× and
at 4×, E4's figure.

### 44.3 The synthetic programs (`tests/scaled.rs`)

Each at 2×, 3× and 4×, every sample compared with a reference computed in the test from §39's rules [M]:

| Test | What | Samples otherwise than the reference |
|---|---|---|
| `triangles_at_n_cover_the_samples_their_edges_hold` | six flat triangles of every slope and winding, two sharing an edge, one a sliver, against the console's edge rule at each sample point | 0 at each scale |
| `sprites_and_quads_stay_on_the_grid_with_every_block_one_texel` | sprites of 16, 8 and variable size, raw and modulated, flipped in x and in y, and quads drawn a texel a pixel, side by side and touching, against the console's pixel in every sample of its block | 0 |
| `a_picture_drawn_at_n_is_read_back_and_copied_at_n` | a shaded triangle drawn into a page (1,052 to 1,134 values), read back as a 15-bit texture by a sprite, a quad and an x-flipped sprite, copied beside itself and copied over itself five rows down | 0 (the copy over itself against `vram.rs`' row order on the samples) |
| `the_mask_bit_and_blending_act_on_each_sample` | a triangle drawn with the mask bit set under a rectangle drawn with it checked; a semi-transparent rectangle over a second triangle | 0 |
| `a_state_at_n_continues_its_picture` | a state at 4× saved, loaded and saved again the same bytes; a second machine handing the first's pictures from its first frame; a state of 4× loaded at 2× seeding the renderer at its own scale | as stated |
| `the_bench_discs_keep_their_goldens_at_2x_and_4x`, `…_with_every_enhancement` | §48 | |

### 44.4 The crack fill at N×

- **Synthetic** (`at_n_the_fill_closes_every_crack_and_spills_over_no_shared_edge`, `tests/precise.rs`): §33's floor and
  ring and its two quads sharing a slanted edge, at 2× and 4×. Of the samples the console's own polygons cover at the
  scale (the renderer drawing the same program without the precision layer), 99,704 at 2× and 398,816 at 4×, none shows
  the cleared buffer with precise geometry on; of the samples within six pixels of the shared edges, 8,493 and 33,972,
  none is the other quad's by the side of the precise edge its point lies on [M].
- **The turning grid's edges** (§34.2's measure, sample by sample: a sample counts an N²th of a pixel;
  `the_turning_grids_shared_edges_hold_at_n`) [M]:

| Scale | Shared edges: mean distance from the ideal, change frame to frame, worst (px) | The outline: the same |
|---|---|---|
| 1× (§34.2) | 0.0036, 0.0066, 0.092 | 0.287, 0.116, 1.339 |
| 2× | 0.0035, 0.0059, 0.0946 | 0.2847, 0.1151, 1.3436 |
| 4× | 0.0035, 0.0053, 0.0576 | 0.2816, 0.1141, 1.2943 |

  No sample between two quads was left undrawn at either scale. The outline keeps §34.2's half of the console's edge
  wobble at every scale: the fill puts the top and left edges on the console's line, and at N× that line is drawn N
  times finer but no nearer the geometry.
- **The cracks of §32.2 in play**, by §32.1's model taken to the sample (a scratch layer in the probe's folder, not in the
  repository: who owns each sample by integer coverage at the scale, and with the precise polygons, unfilled and with the
  built fill's marks; a crack sample is one the console's polygon owns where the enhanced picture shows an older command
  the console shows nowhere within two pixels, not a precise neighbour across a shared edge). At 1× the model gives
  §32.2's and §33.3's figures to the pixel, which is its check [M]:

| Run | Scale | Crack samples unfilled | in pixels' worth | Left by the built fill | in pixels' worth |
|---|---|---|---|---|---|
| Ape Escape in play, 265 frames | 1× | 17,611 | 17,611 | 2,149 | 2,149 |
| | 2× | 53,443 | 13,361 | 4,956 | 1,239 |
| | 4× | 196,119 | 12,257 | 16,088 | 1,006 |
| Spyro the Dragon in play, 235 frames | 1× | 58,023 | 58,023 | 194 | 194 |
| | 2× | 215,299 | 53,825 | 348 | 87 |
| | 4× | 839,978 | 52,499 | 1,069 | 67 |

  The fill closes the same share at every scale: 88, 91 and 92 per cent of Ape Escape's crack area, 99.7, 99.8 and 99.9
  of Spyro's. The area of the cracks falls a little with the scale (a crack a pixel wide at 1× is a sliver whose samples
  half fall on either side at N×), and what is left is §34.3's semi-transparent kind.

## 45. True colour, as built

§40 as designed. The plane (`fine`, binding 6 of the walk, 3 of the scan, 2 of the copy) is made only with true colour
on, a four-byte buffer standing in otherwise; the walk carries a sample's low bits beside its word, `put` blends and
stores at eight bits with no dither, a texel of a 15-bit page brings its plane's bits, a fill keeps its colour word's
three low bits a channel (the record's word 15), a transfer writes none, a copy moves them. The scan shows
`min(255, (33 v) >> 5)`. The state's block gains `Fine` with true colour on. `run` and `bench` take `--colour`.

`tests/colour.rs` [M]:

- **A dithered Gouraud gradient** from black to (255, 128, 64) across 256 pixels, 60 rows: red has at most **32**
  distinct levels along a row in the console's picture, a mean run of one value of 2.16 pixels, and falls back **2,820**
  times along rows that only rise (the dither's grain); with true colour **249** levels, runs of 1.03 pixels, and **no**
  fall. With true colour off the renderer's picture is the console's to the byte. P-E5f held.
- **What the console draws alike shows alike**: an uploaded picture, a fill of a colour with low bits (80 80 80h) and
  flat triangles of colours the five bits hold, with true colour on: the picture equals the console's in every pixel.
- **The three bits kept**: 3,963 samples of a drawn gradient carry low bits; a semi-transparent rectangle (B + F) over
  its copy gives each sample the blend at eight bits exactly; a raw sprite reading the gradient as a 15-bit texture
  carries each sample's halfword and plane, but where the halfword is 0000h, transparent as the console tests it.

What it shows that the console did not is §49's: steps of a fade or a glow below one five-bit step.

## 46. Texture filtering, as built

§41 as designed. A record carries `FILTERED` (bit 30) when the options ask it of its kind (`Renderer::filtered`): every
textured record with Everything; with 3D polygons a precise triangle, and an integer one unless `record::texel_a_pixel`
holds (its texture gradients one whole texel along one axis and none along the other, the console's 4,096 in 12-bit
units, in either orientation), and never a rectangle. The walk's `bilinear` mixes four texels at eight bits, a 15-bit
page at N× between its samples; `filter_limits` holds them within the primitive's texels (the vertices' range, short of
the end the gradients run toward; a sprite's whole texel span, the record's word 9), or none on an axis the texture
window repeats, each texel then passing through the window. Whether the pixel is drawn and blends is the nearest
texel's. `run` and `bench` take `--filter off|3d|all`.

`tests/filtered.rs` [M], at 1× and 2×:

| Test | Result |
|---|---|
| `a_packed_page_mixes_no_texel_of_its_neighbour`: a region of two alternating reds magnified four times, the blue region beside it in the page | no sample with any blue; at 2× 19 shades of red between the two |
| `a_cut_out_keeps_the_consoles_silhouette_with_no_fringe`: transparent texels beside green ones, magnified | the samples drawn are exactly the unfiltered picture's (2,176 at 1×, 8,576 at 2×), every one wholly green |
| `a_texture_window_wraps_its_tile`: a window repeating eight grey texels four times, blue texels past the tile in the page | no sample not grey; 15 greys, the tile's mixed across its wrap |
| `three_d_polygons_leave_sprites_and_flat_quads_crisp` | with 3D polygons a sprite and a quad drawn a texel a pixel are their unfiltered samples exactly and a magnified quad differs; with Everything at 2× the sprite is filtered, and at 1× it is not changed, its coordinates whole |

## 47. The settings and the state

### 47.1 The settings, as built

`diamond-v1` (§42.1): `PRECISE_GEOMETRY`, `INTERNAL_RESOLUTION`, `TRUE_COLOUR` and `TEXTURE_FILTERING`, each an
`Enhancement` with its help and its cost; `Chosen` (precise geometry and the renderer's `Options`) is read from the
settings at create and changed by `set_settings`, which detaches the layer, keeps the renderer if there was one
(`configure`) or makes one, and attaches it alone or paired (`choose`). The note for a missing device is given under each
key not at its hardware value. Machine info's largest picture is 2,560 by 2,304. Every cost names the frame of latency.
Turning a setting on takes the time of a device's creation the first time (31 to 69 ms on this machine at E4) and a
rebuild of its buffers at a new scale after.

### 47.2 State: the kept form and the seeded one, measured

On each bench disc by its press script to frame 1,500, every enhancement on (precise geometry, true colour, 3D
filtering) at 2× and at 4×: the state kept whole; the bytes in which the states of consecutive frames differ (a
rewind's delta, the mean of ten), kept whole and in the accurate part alone; and a second machine given the accurate
part alone, its renderer seeded from the machine (the seeded form), the frames until the picture it hands equals the
kept machine's pixel for pixel (a scratch test in the probe's folder) [M]:

| Disc | Delta a frame, 2× kept / accurate | 4× kept | Seeded: pixels differing, first five frames, at 4× | Equal from frame |
|---|---|---|---|---|
| Ape Escape | 1,143,797 / 119,335 | 4,207,006 | 1.46 M, 1.46 M, 1.46 M, 1.46 M, 0.56 M | 6 |
| Castlevania | 8,700 / 8,675 | 8,700 | 0 | 1 |
| Crash Bandicoot | 119,882 / 12,792 | 407,579 | 0.41 M, 0.43 M, 0.43 M, 0.12 M, 0.12 M | 6 |
| Mega Man Legends | 31,206 / 11,295 | 86,332 | 8,370, 8,322, 8,322, 1,160, 1,160 | 6 |
| Rayman | 9,071 / 9,070 | 9,071 | 0 | 1 |
| Silent Hill | 177,856 / 11,033 | 677,837 | 0.78 M, 0.78 M, 0.39 M, 0, 0 | 4 |
| Spyro the Dragon | 1,496,172 / 190,560 | 5,351,772 | 1.82 M, 1.82 M, 1.82 M, 1.82 M, 0.21 M | 7 |
| Tekken 3 | 650,199 / 160,506 | 2,118,756 | 0 | 1 |

The state is 26,503,560 bytes at 2× and 81,947,016 at 4× with everything on, against 5,398,740 for the accurate part
(the same on every disc: the block's form is fixed). The seeded form's state would be the accurate part and the
precision layer's 2.6 MB.

*Superseded 2026-10-14 by the tester's choice of the seeded form (§52, §53).* **Decided, and why** [C]: **the kept form is built**, as §42.2 set out, because it is the form the kit passes as it
stands. **The measurement favours the seeded form**: a seeded picture is the kept one's within four to seven frames on
every disc of eight (a game redraws its screen, and the precision layer finds its vertices again within two frames), the
first frames differing only in detail, the console's own picture enlarged; and the kept form costs a state three to
fifteen times the accurate one's and a rewind's delta up to 28 times (Spyro at 4×, 5.4 MB a frame against 0.19).
**Proposed to the tester, not made**: a field of machine info, "pictures settle after N frames", which the kit's C7 and
C8 read to compare an enhancement's pictures after a load from that frame on, sound and state from the first, C6 and C8's
first half still comparing every picture of a run without a load. With it Diamond would declare N = 8 with any
enhancement on and write a block of the palette and the picture's size alone. It is a change to the kit's cases for
every core, so it waits.

## 48. The gate, item by item

*Run 2026-10-14 on the desktop, the RX 6800.*

| Item | Result |
|---|---|
| **Off is the accurate path**: E1's goldens with the four settings off | **Equal** [M]: `programs.rs`' three golden tests, plain and with the test observer, 49 + 48 lines and the eight discs' 80 anchors, after the speed work, the only stage of E5 that touched the machine (`Layer::moves`); the frame's code with no layer attached unchanged in the plain form |
| With any setting on, the machine's goldens | **Equal** [M]: picture, sound, state, VRAM and TTY on the programs, ps1-tests and the eight discs with precise geometry (`precise.rs`), the renderer alone at 1× (`renderer.rs`), and on the eight discs' 80 anchors at 2× and 4× with precise geometry, at 2× with true colour and 3D filtering beside it and at 4× with everything (`scaled.rs`) |
| **1× is E3's equality** | **Equal** [M]: E3's three gates (every picture of the programs, ps1-tests and the eight discs to frame 3,000, VRAM at every anchor, the texture cache's two pinned counts the one difference) and E4's at zero fraction, with the scaled, true-colour and filtering code built in and those settings at their hardware values |
| **Synthetic programs** | **Pass** [M]: `scaled.rs`' five (§44.3), `precise.rs`' two at N× (§44.4), `colour.rs`' three (§45), `filtered.rs`' four (§46), each at the scales stated there |
| **The eight discs at 2× and 4×**, their goldens and their pictures | goldens **equal**; the pictures looked at, every visible defect in §49 with its fate |
| The cracks of §32.2 and the shared edges of §34.2 at 2× and 4× | §44.4: the fill closes 88 to 92 per cent of Ape Escape's crack area and 99.7 to 99.9 of Spyro the Dragon's at every scale; the grid's shared edges 0.0035 px at both scales, E4's |
| **The kit** on the published shape (`cargo build --profile dist` in `v1/`), the eight discs by its corpus form, four at a time | **Every case on every disc** [M] with the settings off (1,800 frames, 195 s), `precision=true` (1,800, 335 s), `resolution=2` (1,800, 470 s), `colour=true` (1,800, 284 s), `filtering=3d` (1,800, 258 s), and all four on at 2× (`filtering=all`, 1,800, 731 s); at 4× alone and with all four on at 300 frames (334 s and 561 s), C7's states 5,398,770, 11,594,150, 15,491,842, 8,971,010, 7,922,434, 26,503,590, 45,769,474 and 81,947,046 bytes |
| The kit at 4× for 1,800 frames | **not run to its end**: the kit gives an image's cases 600 seconds (`IMAGE_DEADLINE_SECONDS`), and at 4× it hashes each of the frames its cases run, 4.9 MB of Crash Bandicoot's picture byte by byte, some 20 ms a frame (C6, C7 and C8 alone run the image's 1,800 frames five times), so C6 found the deadline passed on the first four discs of the first run; the same cases pass at 300 frames, and the machine at 4× runs its frames at 2.2 ms (§50). A deadline in proportion to the picture's bytes is a change to the kit, put to the tester |
| **The player's path**: a Mistress publish | §51.1 |
| WiseMan | **221 of 221** [M] under the test lock with ps1-tests, filtered by `Diamond`, `PlayStation`, `GraphicsSettings`, `WindowFitAudit`, `CoreRegistrationEquivalence` and `CoreAdapter`: `DiamondEnhancementsTests` (the four with their costs, the tab's heading and four rows, each and all turned on and off between frames through the host changing only the picture), `DiamondShaderTests` (the four shaders' bytes the compilation of their sources), the fit audit's PlayStation tab at both sizes |

## 49. What the pictures showed

### 49.1 How they were looked at

The machine's picture and the renderer's at every anchor of the eight discs by their press scripts, written by
`scaled.rs`' disc gates with `EMUSEN_DIAMOND_PICTURES` (no picture of a game is in the repository): at 2× and 4× with
precise geometry, at 2× with true colour and 3D filtering beside it, and at 4× with everything; Tekken 3's attract mode
to frame 10,000 at 4× with everything by the run tool, its fights among it. Each was looked at whole and side by side,
zoomed where something showed, and screened by a count of the console's pixels whose every sample in the enhanced
picture is the console's colour [M, 2× with precise geometry]: 100 per cent on every disc's anchors before play and on
Tekken 3's to frame 3,000; Castlevania 99 to 100; Rayman 73 to 100, its 3D logo the rest; and where a 3D game plays, 45
to 97, the precise polygons and the finer texels.

### 49.2 Visible defects, each with its frame and fate

| Disc, frames | What shows | Why | Fate |
|---|---|---|---|
| The turning grid, every frame at 2× and 4× | the texture's far side along polygons' top and left edges, seen as the grid's shared edges 0.014 and 0.082 px from the ideal | a coordinate past the vertices' range at an edge sample wrapped to eight bits before the texel hold (§44.2) | **fixed** before the discs were run |
| Silent Hill, 300 to 900 (the publisher's logo), true colour | a faint square about the logo, six levels darker than the grey about it (101 against 107 in each channel) | the logo fades in by a subtractive blend whose colour is below one five-bit step (7 of 255): the console's five bits make it nothing, true colour subtracts it | **recorded**, the cost true colour names; drawing a blend's front colour at the console's five bits would remove it and with it the smooth fog the setting exists for |
| Ape Escape, 2,700 (the sun over Monkey Park), true colour | the glow's square outer edge, faint where the console's dither hid it | the same: the glow is added in steps below one five-bit step out to its texture's edge | **recorded**, the same cause |
| Spyro the Dragon, 2,700, Everything at 4× | the memory card warning's text and frame soft | filtering sprites at 4× mixes their texels, as the choice says | as designed; with 3D polygons the same text is the console's, sample for sample |
| Tekken 3, 7,500 to 10,000 at 4× | the fighters and the backdrop combed in bands four rows tall while the camera moves | the 480-line picture woven from two fields, as the console's is; each field's line is four rows | unchanged in kind: the console's picture combs the same lines |
| Every 3D disc with true colour off at N× | the dither's grain in blocks of N by N (Ape Escape's lab, Silent Hill's fog) | the console's matrix by the console's pixel (§39.2) | as designed; true colour removes it |

**Looked at and found unchanged in kind, or finer**: Spyro the Dragon's warning box and its "Using Card In Slot 1" line at
4×, every glyph the console's block for block; Crash Bandicoot's title and menu at 2× and 4×, the letters crisp, the
logo's model finer; Castlevania's "Now Loading", a sheared textured quad, its letters drawn finer and its 2D the
console's; Rayman's title, its 3D logo finer and its 2D the console's; Mega Man Legends' hall; Silent Hill's street in
the fog, with the copy it makes every frame (2,999 copies over 3,000 frames, a framebuffer effect), at 4× its detail
kept; Tekken 3's copies in its fights (3,866) and its menus' text; Ape Escape's lab and Monkey Park with true colour, the
rainbow and the sky without grain. No seam between sprites or tiles, no misaligned 2D and no broken framebuffer effect
was found on the eight discs; their battle transitions and motion blurs lie past frame 3,000 and were not reached.

## 50. Cost

### 50.1 The instrument

§27.1's: `examples/bench` on links with every function aligned to 64 bytes, each build in its own target folder, the
builds alternating on one core (`taskset -c 11`), the fastest mean of five rounds, milliseconds of the emulation thread a
frame, the RX 6800. Three builds: **base**, WiseMan 137a039a, E4 with the crack fill, extracted outside the repository;
**speed**, this branch's first code commit (§43); **final**, the head after the settings (§47). With a renderer attached
the bench now times what a host pays (`frame_end` with the frame, the picture of the frame before handed); `--waited`
times E4's form, the frame's picture waited for, and its wait is the device's drawing with the round trip and the
readback, measured once (a single round) for the device's own time. Every item's digests were the same in every form.

### 50.2 Off, and the speed work

**Off, against the base** [M]: `gpu/benchmark` 2.343 and 2.325, `gpu/bandwidth` 2.965 and 2.972, Crash Bandicoot 2.323
and 2.330, Mega Man Legends 1.815 and 1.826, Tekken 3's first 3,000 frames 2.397 and 2.395, its fights 3.739 and 3.716:
within −0.8 and +0.6 per cent, **the base within the noise**.

**On** [M]; "the picture's share" is the bench's own timing of the frame end, or of the waited picture in the base:

| Run | Off (final) | Precise geometry, base | speed | final | the picture's share, base / final | Renderer alone, base | final |
|---|---|---|---|---|---|---|---|
| `gpu/benchmark` | 2.325 | 2.671 | 2.532 | 2.524 | 0.221 / 0.069 | 2.655 | 2.459 |
| `gpu/bandwidth` | 2.972 | 3.614 | 3.395 | 3.375 | 0.295 / 0.041 | 3.546 | 3.169 |
| Crash Bandicoot | 2.330 | 2.811 | 2.633 | 2.644 | 0.214 / 0.034 | 2.684 | 2.432 |
| Mega Man Legends | 1.826 | 2.200 | 2.032 | 2.027 | 0.182 / 0.021 | 2.099 | 1.921 |
| Tekken 3 | 2.395 | 2.909 | 2.817 | 2.800 | 0.138 / 0.017 | 2.792 | 2.591 |
| Tekken 3's fights | 3.716 | **5.312** | 4.337 | **4.341** | **1.067 / 0.067** | 5.014 | 4.035 |

- **Precise geometry over the plain frame**: from +0.37 to +0.64 ms on the five items to +0.20 to +0.40, and in the
  fights from **+1.60 to +0.63**. The picture's share fell from 0.14 to 1.07 ms to 0.02 to 0.07: what is left is the
  submission's recording and, in the fights, the wait for the frame before on a device that drew 1.4 stretches.
- **What each piece gave**, from the series: the parity split took the fights' waited picture from 1.07 to 0.50 ms
  (§43.2, the waited form of the speed build); the pending submission took the rest off the thread, 0.2 ms on the
  five items and 0.5 in the fights; `Layer::moves` takes the renderer alone from base 2.684 to final 2.432 on Crash
  Bandicoot, of which 0.18 is the wait and the rest, 0.07 to 0.15 ms on the five items, the layered frame (§18.3's
  middle column), its share in E3's measure.
- **What is left over the plain frame** is the renderer's host work (events, decoding, records, binning) and, paired, the
  precision layer's: the renderer alone 0.10 to 0.20 ms on the five items and 0.32 in the fights; paired 0.20 to 0.40 and
  0.63.

### 50.3 At each scale, and with true colour and filtering

**The emulation thread** [M, final, five rounds]:

| Run | Precise, 1× | 2× | 3× | 4× | Alone, 1× | 2× | 4× | Alone 1×, colour / filter 3D | Alone 4×, colour / filter 3D | Precise 4×, colour, filter all |
|---|---|---|---|---|---|---|---|---|---|---|
| `gpu/benchmark` | 2.524 | 2.535 | 2.532 | 2.540 | 2.459 | 2.462 | 2.456 | 2.452 / 2.455 | 2.466 / 2.452 | 2.519 |
| Crash Bandicoot | 2.644 | 2.635 | 2.636 | 2.634 | 2.432 | 2.429 | 2.426 | 2.427 / 2.430 | 2.427 / 2.428 | 2.627 |
| Mega Man Legends | 2.027 | 2.037 | 2.034 | 2.037 | 1.921 | 1.918 | 1.921 | 1.914 / 1.913 | 1.916 / 1.913 | 2.065 |
| Tekken 3 | 2.800 | 2.798 | 2.800 | 2.805 | 2.591 | 2.586 | 2.581 | 2.581 / 2.577 | 2.577 / 2.579 | 2.801 |
| Tekken 3's fights | 4.341 | 4.319 | 4.311 | 4.303 | 4.035 | 4.020 | 4.006 | 4.023 / 4.021 | 4.002 / 4.001 | 4.296 |

**The scale, true colour and filtering cost the emulation thread nothing measurable on this machine**: every column is
its 1× column's within 0.04 ms, the 4× ones among them, since the host's work does not grow with the scale (§39.5) and
the device's runs beside the next frame. The cost is the device's, and shows on the thread only where the device falls
a frame behind.

**The device's own time** [M, `--waited`, one round: the waited picture, which holds the device's drawing, the round
trip and the readback of the picture at the scale]:

| Run | 1× | 2× | 3× | 4× | 4×, true colour, filtering everything |
|---|---|---|---|---|---|
| Crash Bandicoot | 0.225 | 0.329 | 0.470 | 0.644 | 0.694 |
| Mega Man Legends | 0.191 | 0.239 | 0.323 | 0.456 | 0.547 |
| Tekken 3 | 0.139 | 0.199 | 0.281 | 0.375 | 0.394 |
| Tekken 3's fights | 0.536 | 0.817 | 1.062 | 1.247 | 1.494 |

The device's time grows by 2.3 to 2.9 times from 1× to 4×, not by 16: at 1× most of it is the round trip and the
submission's fixed work, and the drawing itself grows with the samples. True colour and filtering together add 0.02 to
0.25 ms at 4×. On the RX 6800 every item draws its 4× picture in under 1.5 ms of a 16.7 ms frame.

### 50.4 State sizes

C7's states on the kit's machine (two DualShocks and a card) [M]: none, 5,398,770 bytes; precise geometry 11,594,150;
2× 15,491,842; 4× 45,769,474; true colour 8,971,010; filtering 7,922,434 (the renderer's block at 1×); everything at 2×
26,503,590, at 4× 81,947,046. A rewind's deltas and the seeded form: §47.2.

### 50.5 The handheld, estimated from the tester's play

**Not measured**: the tester was playing on the handheld throughout this stage, and the bench was not run there. The
estimate rests on the tester's own log of the evening (2026-10-14, 02:00 to 02:46, Mega Man Legends with "Precise
geometry" on, on battery, WiseMan 137a039a, the build before this stage): the core's frame over 2,735 seconds had a median
of 11.7 ms, a 90th percentile of 18.4 and a worst of 25.5, play mostly at 6 to 12 ms and a boss fight and the stretch
after it at 18 to 19.5 for about five minutes, the game then at 48 to 52 frames a second; the same game's frame is about
2 ms there with the setting off. **§27.4's estimate of +2 to 3 ms was low by three to five times.** Against the desktop's
figures for the same build (§27.3: Mega Man Legends +0.35 ms over the plain frame, its waited picture 0.16 ms), the
handheld's renderer share in that game's play, about 9.7 ms at the median, is some 50 times the desktop's waited picture,
not §18.4's ten to fifteen: the integrated device at its 800 MHz floor pays far more for each submission and its round
trip than the RX 6800 does [A, by the ratio; the split between drawing and round trip is not measured there].

**What this stage changes there** [A]: with the picture a frame late, the device's time leaves the emulation thread
whenever it is under a frame, about 16.7 ms. The thread keeps the host's share, 0.20 ms on the desktop for this game with
precise geometry, perhaps 0.4 to 0.6 there, so **Mega Man Legends' frame there should fall from a median of 11.7 ms to
about 2.5 to 3**, with the device drawing beside it. **The boss fight** is the case to watch: if its 16 to 17.5 ms over the
plain frame were all the device's, the device would sit at the edge of a frame, the machine waiting for it by up to a
millisecond a frame and the game at or just under full speed rather than 48 to 52; the stretches' parity split (§43.2)
does not help it, the game drawing at 240 lines. **At 2×** the device's time grows by a quarter to a half on the desktop,
which there would take the boss fight past a frame (about 20 to 25 ms, 40 to 50 frames a second) while play stays
within one; **at 4×** by 2.3 to 2.9 times, past a frame in most play. True colour and filtering add 0.02 to 0.25 ms to the
device's time at 4× on the desktop, a tenth or less of what the scale adds, and should add as little there. **The confirmation is left to the
tester's play**, and the Mega Man Legends boss state the tester saved is the case to measure first when the handheld is
free; the bench there would time `--waited` and the host's form side by side, as §50.3 does.

## 51. The publish, tests, predictions, what E6 needs, and the reading log

### 51.1 The publish

`dotnet publish EmuSen.Mistress -c Release -r linux-x64` into a scratch folder [M]: the build runs cargo in `v1/` with
the `dist` profile as before, and the tree carries `lib/EmuSen/libdiamond.so` (2,229,664 bytes) with its sidecar, whose
Enhancements are the four: `precision` "Precise geometry" (false), `resolution` "Internal resolution" (1), `colour` "True
colour" (false) and `filtering` "Texture filtering" (off), each with its help and cost. Nothing else in the build
changed; the scratch folder was removed after.

### 51.2 Tests [M, 2026-10-14]

All on the final head, after `git merge WiseMan` (WiseMan had not moved from 137a039a).

- **`diamond`**, `cargo test --release` with `EMUSEN_DIAMOND_PS1TESTS`, `EMUSEN_DIAMOND_DISCS` and
  `EMUSEN_GPU_DEVICE=6800`: 167 unit tests, 81 program tests (E1's goldens plain and with the test observer, the discs'
  among them), E3's 3 renderer gates, 14 in `precise.rs` (E4's and the crack fill's twelve, two at N×), 7 in `scaled.rs`,
  3 in `colour.rs` and 4 in `filtered.rs`, all passing; and without the two corpora the same counts, the corpus tests
  standing down.
- **`precision`**: 6. **`diamond-renderer`**: 2 (one added: the written set by row parity). **`diamond-v1`**: 4, and 4
  with `EMUSEN_GPU_DEVICE` naming no device, the three that need one standing down with every set key's note printed.
- **The kit**, the corpus form on the eight discs with each setting and all of them (§48): every case on every disc.
- **WiseMan**, under the test lock with ps1-tests, the filter of §48: 221 of 221, on the final head.
- **Not run**: `emusen-gpu`'s and MarsRT's suites (`emusen-gpu` unchanged: the pending submission and everything else
  the renderer uses were in its version 1.1), the BlackMoon crates' (unchanged), the Vulkan validation layer over the new
  walk, llvmpipe and the integrated card, the full WiseMan suite, the laptop (not part of this stage), the handheld
  (in the tester's hands throughout, §50.5).

### 51.3 Predictions

- **P-E5a, partly held and partly retired** [M, §50.2]: the picture's share fell from 0.14 to 1.07 ms to 0.02 to 0.07,
  as predicted; but precise geometry at 1× stands 0.20 to 0.40 ms over the plain frame on the five items and 0.63 in the
  fights, more than "within 0.1 ms of the precision layer with a drawer that draws nothing" (E4's 0.06 to 0.24): the
  renderer's host work, 0.10 to 0.32 ms alone, was counted in the prediction as if it were part of the wait. Retired.
- **P-E5b, held in its first half and failed narrowly in its second** [M, §43.2, §50.3]: 1.39 stretches a frame in the
  fights, under 2; the waited picture 0.50 to 0.54 ms, where under 0.5 was predicted.
- **P-E5c, held on two discs of three** [M, §50.2]: the renderer alone 0.10 ms over the plain frame on Crash Bandicoot and
  0.10 on Mega Man Legends, 0.20 on Tekken 3's first 3,000 frames, where within 0.15 was predicted; the fights 0.32.
- **P-E5d held** [M, §50.3]: at 4× the emulation thread is within 0.04 ms of 1× on every item, the fights included (the
  prediction excepted them); the device's time at 4× in the fights 1.25 ms, 1.49 with true colour and filtering
  everything, under the 4 predicted.
- **P-E5e held** [M, §48]: every picture of E3's and E4's gates unchanged at 1× with the new settings off; every machine
  golden with any setting on.
- **P-E5f held** [M, §45]: 249 levels of red across the gradient with true colour, against the console's 32.
- **§27.4's handheld estimate retired** [M, the tester's log, §50.5]: +2 to 3 ms predicted where game play showed about
  +10 ms at the median and +17 in a boss fight. The ratio of compute that estimate rested on was not the handheld's cost
  of a submission; §50.5's estimate rests on the measured ratio instead and is itself to be confirmed.
- **P9** (the plan's) held for E5; **GE**: each enhancement offered with its gate met.

### 51.4 What is left, and what E6 needs

- **Left for the tester** [proposed, not built]: the kit's "pictures settle after N frames" for an enhancement's
  pictures across a load, which would let the state be the accurate part and a few bytes (§47.2); a deadline in the kit
  in proportion to a picture's bytes, without which the kit cannot run 1,800 frames at 4× within its 600 seconds
  (§48); the core ABI's word for "the console's own lines" so that a CRT filter draws the console's lines over a picture
  at N× (§39.4); and, for true colour, drawing a blend's front colour at the console's five bits, which would remove the
  faint shapes of §49.2 and with them the smooth fog true colour exists for.
- **Left**: the handheld's measurement (§50.5); a Mega Man Legends boss state to bench, which the tester has offered to
  copy; the battle transitions and motion blurs past the bench discs' frame 3,000, which no anchor reached; Mars's
  "Antialiasing" (the picture reduced from N× to the console's size) as a choice of Diamond's, a further enhancement.
- **E6, depth from the precision path**: the precise record already carries each vertex's depth (`Fraction::z`, the
  camera's depth with 12 fraction bits), and the walk evaluates the weights a perspective-correct texture needs; a depth
  plane beside the mirror at the scale (one word a sample, as true colour's plane is), written by precise triangles and
  read only to break ties the ordering table leaves (the plan's §6.3: never a plain depth test, which hides HUDs and
  decals), with a rule for integer polygons, which have no depth, and a measure on the bench discs of where a tie is
  broken otherwise than the console's order. Its state would add a plane to `RNDR`, which §47.2's proposal would spare.
- **E6, the widescreen projection**: a scale of the GTE's horizontal projection, which is the machine's arithmetic and
  so an `accuracy` setting, not an enhancement of the picture alone (the plan's §6.3: "a scale of the GTE's horizontal
  projection"); the renderer's picture wider than the display area at the scale; and the plan's breakage measured on the
  bench (geometry culled at the 4:3 edges, the HUD stretched). Its goldens could not hold with it on, so its gate would be
  P9 with it off and the machine's determinism with it on.

### 51.5 Reading log (E5)

**No emulator's source, shader or snippet was opened, and no description of any emulator's upscaler, true colour or
texture filter, with or without code**: nothing of DuckStation's, Beetle PSX's (parallel-psx and its rsx among it),
PCSX's, PCSX-Redux's, SwanStation's, ePSXe's plugins or any fork's; no xBR, JINC or other filter shader. The filter is
plain bilinear arithmetic and the scale the console's own rules evaluated at more points. No firmware, listing or trace
of one; the player's BIOS folders were not read; nothing under `kernel/` or `kernel-harness/` was opened or changed.
psx-spx was not opened. No game's code was read: the discs were run, their pictures looked at. The handheld was reached
twice, by the project's helper, to check whether it was free (it was in use both times, so nothing was run there).

Opened, all EmuSen's own: this page whole; `Diamond_Plan.md` §1, §6, §7, §8 and §10; `Diamond_GPU.md` §3 to §7;
`Diamond_Players.md` §15; `EmuSen_CoreAPI.md` §6.5, §6.6, §6.9, §6.13, §6.16 and §12; `EmuSen_Settings_Reference.md`
§4.110; `Mars_Gpu.md` §13 to §16 and `MarsCore.cs`' `RenderScale`; the source of `diamond-renderer` whole with its
shaders, of `precision`, `diamond-v1`, `diamond`'s `layer.rs`, `machine.rs`, `gpu/` (`decode.rs`, `display.rs`,
`draw.rs`, `raster.rs`, `texture.rs`, `vram.rs`), `tests/` and `examples/`; `emusen-gpu`'s interface; the kit's C6 to C8,
its frame checks and its deadline; Mistress's `GraphicsSettingsWindow.cs` and the host's `CoreEngine.cs` and
`CoreMachine.cs` at the frame; WiseMan's `DiamondPreciseGeometryTests.cs`, `DiamondTests.cs`, `DiamondShaderTests.cs`
and `ShaderCompiler.cs`; the crack fill's model and E4's measurement scripts in the probe; the tester's handheld log.

---

## 52. The seeded state, decided before it was built

*Written 2026-10-14, before any of its code, after the tester's decision of the same day.* §47.2 measured two forms of
the enhancements' state: the **kept** form built at E5, which saves the renderer's mirror and picture at N× and with
everything on makes a state 26.5 MB at 2× and 81.9 MB at 4× against 5.4 MB for the machine; and the **seeded** form,
which saves the accurate part and, at a load, seeds the renderer from the machine's VRAM. The tester chose the seeded
form, as §47.2 recommended, with the change to the conformance kit it needs: the kit is to learn that an enhancement's
picture may take some frames to settle after a load. This section settles what the state keeps, how the kit is told
and how it checks, the kit's deadline, which E5 found too short at 4×, and whether the host needs anything. §53 on is
what was built.

### 52.1 The measurement it rests on

A scratch test in the probe's folder (never committed) ran each bench disc by its press script to a save point, every
enhancement on (precise geometry, true colour, filtering of 3D polygons) at 1×, 2× and 4×, saved the kept form, and gave
the state to three further machines: whole (the kept form), the accurate part with the precision layer's `PREC` (the
renderer seeded), and the accurate part alone (both seeded). Each ran 120 frames beside the saver with the same input,
and the last frame whose picture handed differs from the saver's, pixel for pixel, was recorded; the frames after it were
equal to the end of the run in every case. The rewind's delta is Mistress's XOR codec between states four frames apart,
the mean of ten [M]:

| Disc | Save at 1,500: last differing frame, `PREC` kept / both seeded (1×, 2×, 4× alike) | Save at 2,700 (2× and 4× alike) | Delta at 2×, kept / with `PREC` / accurate alone (bytes) | Delta at 4×, kept |
|---|---|---|---|---|
| Ape Escape | 4 / 5 | 3 / 6 | 6,044,156 / 658,644 / 631,557 | 22,167,325 |
| Castlevania | 0 / 0 | 2 / 2 | 22,788 / 22,788 / 22,597 | 22,788 |
| Crash Bandicoot | 3 / 5 | 0 / 0 | 489,360 / 150,193 / 59,839 | 1,493,915 |
| Mega Man Legends | 3 / 5 | 3 / 5 | 246,190 / 74,684 / 63,294 | 743,523 |
| Rayman | 0 / 0 | 2 / 2 | 76,767 / 30,851 / 30,840 | 214,124 |
| Silent Hill | 3 / 3 | **141 / 141** (2×; past 120 at 4×) | 1,310,408 / 91,765 / 91,755 | 4,964,759 |
| Spyro the Dragon | 4 / 6 | 4 / 6 | 7,353,367 / 1,090,038 / 958,675 | 26,112,121 |
| Tekken 3 | 0 / 0 | 0 / 0 | 2,625,456 / 607,468 / 607,095 | 8,675,639 |

Two kinds of settling show. **A picture the game draws** settles within six frames: the frame handed late (§38.1) is
the seed's, the console's picture enlarged; then the buffers the game drew, or was drawing, before the save, which the
seed holds at the console's detail, are shown until the game has drawn over them (one or two of its frames, which at 30
frames a second are two to four of the console's); then a frame or two whose polygons lack precise vertices, the
ordering table drawn after the load having been built before it, while the precision layer is empty. `PREC` removes only
that last part: two frames on Ape Escape at 2,700, Crash Bandicoot, Mega Man Legends and Spyro the Dragon, none
elsewhere. **A picture made from the picture before** settles as fast as the effect forgets: Silent Hill's street in the
fog copies the frame it showed and draws the next over it (§49.1), so the difference the seed made decays frame by frame
instead of being drawn over. Measured further from the same save at 2,700 over 600 frames [M]:

| Silent Hill from 2,700 | Last differing frame | Differing pixels at frames 30, 60, 90, 120 |
|---|---|---|
| Everything on, 2× | 141 | 8,573; 942; 231; 25 |
| True colour alone, 1× | 121 | 2,084; 253; 59; 6 |
| Precise geometry and 3D filtering, 2× | 32 | 6; 0 |
| Precise geometry alone, 1× | 30 | 1; 0 |
| Internal resolution alone, 2× | 34 | 4; 0 |

With true colour the decay runs four times longer, since its plane's three low bits a channel, which the seed sets to
zero (§40), are carried through each blend and lost only slowly to its rounding; without it the console's five bits
lose the seed's difference within about half a second. Every run did settle: no picture differed past 141 frames, and
the machine's own state was equal at the end of each.

### 52.2 What the state keeps [C]

**The accurate part, and nothing of the enhancements.** With any enhancement on, the state is the state with all of
them off, to the byte (5,398,740 bytes with a DualShock and a card at version 10). Each candidate was weighed by what
it measurably buys against its bytes:

| Candidate | Bytes at 1× / 2× / 4× | What it buys | Kept? |
|---|---|---|---|
| The renderer's mirror and picture (`RNDR`'s body) | 3,572,240 / 14,287,376 / 57,147,920 with the marks | the picture exact from the first frame after a load | no: the seed gives the same picture within six frames on drawn pictures, and the deltas it adds are up to 28 times the machine's |
| `PREC`, the precision layer (serials, ring, shadows) | 2,623,140 at every scale | the last one or two frames of a drawn picture's settling (§52.1); nothing on a feedback effect | **no**: it is 49 per cent of the accurate part, and it multiplies a rewind's delta by 2.5 on Crash Bandicoot and 1.14 on Spyro the Dragon for two frames of finer polygons |
| The crack fill's marks (§33.5) | 1 MiB × N² | the fill where two precise polygons drawn across the save share an edge | no: a frame drawn after the save finds its marks again, as an attach does |
| True colour's plane | 1 MiB × N² | Silent Hill's fog settling in 32 frames instead of 141 | no: it costs more than the mirror it refines, and buys a shorter wait on one effect |
| The palette slot and the picture's size (§47.2's proposal) | 528 bytes | nothing: the seed takes the CLUT cache's colours and the display's size from the machine, which are what the renderer's are at every frame boundary | no |

So the renderer and the precision layer write no block. The optional blocks' frame (§4) stays, E1's test observer
writes one, and no enhancement uses it.

**The state's size no longer moves with a setting.** It was the scale, true colour and the pairing that fixed the kept
block's form (§42.2); now no setting changes a byte of the state, so `STATE_SIZE` is never raised by a change of
enhancement, and a host's rewind chain is not restarted by one.

### 52.3 A load [C]

- **Every load seeds**, as an attach does (§16.6, §39.6): the renderer's mirror is the GPU's VRAM with each halfword a
  block of N², its marks zero, true colour's plane zero, its picture the display's enlarged and put in the slot the next
  frame hands, its palette slot the CLUT cache's colours; and the precision layer starts empty. The precision layer did
  not reset at a seed before (`Layer::seed` was empty for it, and a load without its block cleared it through
  `Layer::clear`); with no block, it is reset by its seed, so that what a machine shows after a load depends on the state
  alone and never on what the machine ran before it: two machines given one state hand the same pictures from the first
  frame, whatever each had done.
- **A state of the kept form loads**: its `RNDR` and `PREC` blocks are tags no attached layer has, which E1's rule skips
  by their length (§4); the machine continues exactly and the picture settles as from any state. Using a kept block when
  its form matches the configuration was considered and not chosen: it would keep the kept form's reader (and its
  buffers of up to 81 MB) alive in the renderer for the states of one day's builds, to spare their first six frames.
  States of versions 8, 9 and 10 load as before.
- **A state saved with the enhancements on loads with them off** and the reverse, trivially now, since the two states
  are the same bytes.

### 52.4 The bound Diamond declares [C]

The kit's rule (§52.5) asks a core to state how many frames after a load its picture may differ. **Diamond states 0
with every enhancement at its hardware value** (it is then the accurate machine, held to the picture from the first
frame), **60 with any on and true colour off, and 240 with true colour on.** The figures rest on the two kinds of
§52.1, not on a proof:

- A drawn picture settles once the game has drawn over the buffers the seed holds and its precision layer has seen
  one ordering table built: at most about one frame of latency and three of the game's own frames. At 30 frames a second
  that is seven, measured six; a game at 15 frames a second would take thirteen.
- A picture fed from the picture before settles as its effect forgets: measured 30 to 34 frames without true colour
  and 121 to 141 with it, on the one such effect the bench reaches. 60 and 240 are those figures with a margin of
  about three quarters.

Neither is a guarantee for every game. **What would break it**: a feedback effect that forgets more slowly than Silent
Hill's fog, and a texture a game draws once and keeps (a backdrop rendered at a level's start and read for the rest of
it), which the seed holds at the console's detail until the game draws it again, perhaps never; with such a disc the
kit would find Diamond's pictures unsettled after the bound and fail C7 or C8, which is the kit doing its work. No
bench disc does the second in its first 3,000 frames. A player meets either as detail finer before a load than after it,
the game's own state untouched.

### 52.5 The kit's rule [C]

**The field.** Machine info's `state` gains **`picture_settles`**, an integer, absent meaning 0: *after a
`state_load`, the pictures of the first `picture_settles` frames the machine advances may differ from those the
machine that saved the state handed after the same frames; from the next frame on they may not; the sound, the events
and the state never may.* At most 600 (ten seconds at 60 frames a second). It is a field of the machine's descriptor
because it is a fact of the machine as configured, as `skip_rendering_state_neutral` is, and changes with its settings;
a core whose bound changes with a setting raises `MACHINE_INFO`. Being a new descriptor field, it is an addition to
version 1 (`EmuSen_CoreAPI.md` §4.4), recorded in the baseline as the core ABI's **minor 5** with its normative text in
§6.4 and §6.9 there and its reasons in `EmuSen_CoreAPI_Extensions.md` §14.

**Why the core states it, and not a rule of the kit's.** A kit-side rule tied to `effect: enhancement` settings being
non-default would need the kit to choose the bound, for every core, without knowing why any picture differs; it would
excuse every core's enhancements from a check they may pass today, whether or not their states need the excuse; and it would leave a host that loads states every frame (run-ahead, a rollback of netplay, §6.22 there) without
the fact it needs, that the pictures shown in the first frames after a load are not final. The core knows the bound and
states it once; the kit holds the core to it.

**What the kit does with it**, in C7 and C8 (and C12, which runs them again):

- **Sound, events and state are compared as now**, from the first frame after the load; only pictures are excused, and
  only the first `picture_settles` frames after a load. C7's comparison of B (loaded with A's state) with A, and C8's of
  D (loaded at the middle) with the first machine's second half, fold each frame's picture into a second digest from
  the frame after the bound on, and compare that, the sound's digest and the end state. Every comparison of runs without
  a load (C6, C8's two machines fed alike, C17, C18) is unchanged.
- **A core that declares nothing is held exactly as now**: with a bound of 0 the second digest is the first.
- **A picture that never settles fails**, since every frame past the bound is compared; **a machine that differs
  fails**, since sound and state are compared from the first frame.
- **A bound above 600 fails C7 and C8**, and so does a run whose second half is no longer than the bound, which would
  leave no picture after a load compared: the kit's author is told to give more frames (at the kit's 600 frames by
  default a bound of up to 299 is checked).
- The note of each case says when pictures were compared from past a bound, and from which frame.

### 52.6 The kit's deadline [C]

E5 found that an image's cases at 4× did not finish within 600 seconds (§48): the kit hashes every frame's picture
(FNV-1a byte by byte, 3.8 ms for 4.9 MB on this processor [M]), C17 and C18 hash the state at every frame, and the
image's cases run its frames ten times or more. The deadline was a whole image's, set when the cases were fewer and the
pictures 1×; it confused a slow image with a hung one, and every new case, longer run or larger picture would move it
again. **It becomes a deadline on progress**: an image is hung when its cases have advanced no frame for 600 seconds.
The thread running an image stamps the time at each frame it runs; the runner reports an image whose stamp is 600
seconds old as before (C6 failed, its thread left). A hang from the start (a create that never returns) is caught as
before, after 600 seconds; an image that runs, however slowly, is not. Hashing more cheaply was set aside: the frames'
digest is defined as Pharaoh's so that the two compare directly (`EmuSen_CoreAPI.md` §12.1). The seeded state removes
the other half of E5's cost on its own: C17's and C18's state digest at every frame hashes 5.4 MB instead of 82.

### 52.7 The host [A, read]

Mistress's rewind (`RewindBuffer`, `EmuSen/Common`) saves a state every `IntervalFrames` (30 since the tester's
change of 2026-10-14: two a second), encodes each against the one before with the XOR codec on another thread, and
trims the oldest past its budget (96 MiB by default, Mistress's own). It sizes nothing from `STATE_SIZE`: each capture
allocates or reuses a buffer of the state's length, and a state of another length than the newest restarts the chain
(`Capture`), which is correct for a state that shrinks or grows. With the seeded form no change of enhancement changes
the length, so turning one on no longer empties the rewind history either. **The host needs no change** to keep rewind
and state slots right. The C# reader of machine info gains the field (`CoreMachineInfo.PictureSettles`, 0 when absent),
as `EmuSen_CoreAPI.md` §5.5 asks of every addition; Mistress has no run-ahead and does not use it.

What a player sees during a held rewind: each step loads a state, and the picture shown after it is the seed's, the
console's picture enlarged, so a rewound sequence runs at the console's detail and the enhanced picture returns within
the bound once play resumes.

### 52.8 The gate, as it will be run

E1's goldens with the settings off; the machine's goldens with any on and with all on at 4×; E3's 1× equality; E4's,
the crack fill's and E5's tests, those that saved the kept form rewritten to the seeded one's terms (the picture equal
from past the bound; two machines given one state equal from the first frame; a kept-form state loading); the kit's own
tests with seeded faults (a picture that never settles failing C7 and C8, a machine that differs within the bound
failing them, a core declaring nothing unchanged, a bound past the limit refused, the deadline on progress); the kit on
the published library on the eight discs at 1,800 frames with each setting, all at 2× and **all at 4×**, and all off;
every other core's kit; the settle measured per disc through the published library; rewind's reach in Mistress at two
snapshots a second through WiseMan, before and after; save and load times and the frame's cost before and after.

### 52.9 Predictions [recorded before the code]

- **P-S1.** With everything on the state is 5,398,740 bytes at every scale, and a rewind's delta at 30 frames is the
  machine's: rewind's reach under 96 MiB grows by at least five times at 2× on Spyro the Dragon, Ape Escape and Crash
  Bandicoot.
- **P-S2.** The kit passes on the eight discs at 1,800 frames with every setting, at 4× included, each image's cases
  within the new deadline, and C7 and C8 compare pictures from frame 61 or 241 after their loads; with all off the
  reports are those of E5's run but for the note.
- **P-S3.** Every other core's kit report is unchanged in verdict and evidence.
- **P-S4.** Saving a state with everything on at 4× falls from the readback and copy of 82 MB to the machine's own save;
  loading costs the seed, under 50 ms at 4×.
- **P-S5.** The frame's cost is unchanged within the bench's noise: no code on the frame's path changes.

## 53. The seeded state, as built

*Built and measured 2026-10-14 on the desktop, the RX 6800 by `EMUSEN_GPU_DEVICE=6800`.* What was built is §52 as
designed, with one wording of the kit's note changed after its first run (§53.2).

### 53.1 Diamond

- **The renderer** (`diamond-renderer`) lost `Kept`, `keeping`, its block (`RNDR`), its `settle` readback and its
  reader; `Back::Palette`, which only the readback used, went with them. Its seed is the one an attach always used: the
  machine's VRAM with each halfword a block of N² samples, no marks, true colour's plane cleared, the display's
  picture enlarged into the slot the next frame hands, and the CLUT cache's colours in palette slot 0; it is now made a
  row of blocks at a time, which a load's cost at 4× asked for (§53.4).
- **The precision layer** (`precision`) writes no block (`PREC` gone) and resets at every seed; `Precise` passes the
  block calls to its drawer, which has none.
- **`diamond-v1`** attaches the renderer as is, states `picture_settles` (`SETTLES`, 60, and `SETTLES_TRUE_COLOUR`, 240,
  §52.4; 0 while the machine's own picture is shown, without a Vulkan device included), and raises `MACHINE_INFO` from
  `set_settings` when the bound changes; the three costs that spoke of states now speak of the picture after a load
  (`EmuSen_Settings_Reference.md` §4.113).
- **The tools.** `run` and `bench` attach the renderer as is; nothing else in them changed.

### 53.2 The core ABI's minor 5 and the kit

As §52.5 to §52.6 and `EmuSen_CoreAPI_Extensions.md` §14.3 state: the header at 1.5 (a comment on
`emusen_core_machine_info` and the minor), the baseline's `json machine_info.state.picture_settles integer optional
@1.5` with the two lines of the minor, the schema's property from 0 to 600 (the crate's validator now reads `maximum`),
`Core::picture_settles` and `PICTURE_SETTLES_MAX` in `emusen-native`, written only when not 0, and
`CoreMachineInfo.PictureSettles` in the C# reader. The guard: 426 facts agree with the header on four triples, with the
Rust and with the baseline, `--against` WiseMan's. In the kit, a `progress` module holds the stamp of the image a thread
runs; C7 and C8 fold a second digest of the pictures past the bound; the note first said "from its 61th frame", corrected
to "from its frame 61 on" after the gate's kit runs, which carry the first wording. Those runs used the library and
the kit built before the seed by rows (§53.4); the seed it replaced fills the same values, which
`a_state_at_n_is_the_machines_and_a_load_seeds` holds sample for sample.

The kit's own tests hold the rule with a settling test core and its faults, each in a runner process of its own
(`pictures_that_settle_pass_within_their_bound_and_each_fault_of_the_rule_fails`) [M]:

| The test core's picture after a load | Declared | Cases failed |
|---|---|---|
| differs for three frames | 3 | none, and the notes name frame 4 |
| the same, the core claiming nothing (no fault) | nothing | none, and no note: held as before |
| never settles | 3 | C8, and C12, which runs it again |
| differs for three frames | nothing | C8, C12 |
| differs for three frames, the sound too | 3 | C8, C12 |
| differs for three frames | 601 | C7, C8, C12 |
| differs for three frames, a run of 6 frames | 3 | C7, C8, C12: "run more than 6 frames" |

C7 does not see the three faults alone: both its machines load the state, A its own to check that save, load and save
give the same bytes, so both are seeded alike. C8's third machine is the one that loads against a run that did not.
The deadline (`the_deadline_is_on_progress_not_on_the_whole_image`): a core taking 4 ms a frame passes an image whose
cases run for longer than a deadline of one second; a core whose tenth frame never returns on one of two images is
reported hung on it ("ran no frame for 2 s"), the other image and the core's own cases passing.

### 53.3 The gate, item by item

| Item | Result |
|---|---|
| **E1's goldens, settings off** | **Equal** [M]: `programs.rs`' 81 with both corpora, the goldens plain and with the test observer and the eight discs' 80 anchors |
| **The machine's goldens with any on, and all on at 4×** | **Equal** [M]: `precise.rs`' 14, `renderer.rs`' 3, `scaled.rs`' 7 (the discs at 2× and 4× with each combination of §48), `colour.rs`' 3, `filtered.rs`' 4 |
| **E3's 1× equality, E4's, the crack fill's and E5's tests** | **Pass** [M], the state tests rewritten to the seeded form's terms: `a_state_with_the_setting_on_is_the_machines_and_its_picture_settles` (the grid's state the plain machine's to the byte; two machines given it equal from the first frame; the picture the saver's from frame 2 after the load and every frame on; the precision layer empty at the load and precise again by frame 2), `a_state_between_two_quads_that_share_an_edge_keeps_no_marks` (85 pixels of the 9,992 the quads cover differ, all on their edges, the machine the same), `a_state_at_n_is_the_machines_and_a_load_seeds` (a state at 4× a plain machine's; the mirror after a load the machine's VRAM and the picture the console's, each pixel a block of 16; two machines alike), `the_layer_keeps_nothing_in_a_state_and_a_load_starts_it_empty`, and diamond-v1's state test with a kept-form state's blocks skipped |
| **A state of the kept form loads** | **Yes** [M]: Crash Bandicoot at frame 600, every enhancement on at 2×, saved by WiseMan 9c37e7a9 (26,503,560 bytes, `PREC` and `RNDR`), loaded on this build with every enhancement on and with none, run 300 frames by the press script: the machine's state and picture at frame 900 those of the old build loading its own state, both ways (FNV 34FE105E37E2731B and D6B902F2602DBB5F) |
| **The kit, the published library, eight discs, 1,800 frames** | **Every case on every disc** [M] off (209 s), `precision=true` (222 s), `resolution=2` (254 s), `colour=true` (204 s), `filtering=3d` (207 s), all at 2× (277 s) and **all at 4× (520 s, each image 186 to 270 s)**; C7's state 5,398,770 bytes in every run; C7 and C8 comparing pictures from frame 61 after their loads, or 241 with true colour. With all off every image case's verdict and evidence is E5's run's, 64 of 64 |
| **Every other core's kit** | **Unchanged** [M]: MoonRT (Super Mario Bros., The Legend of Zelda, self-delimiting), MercuryRT (Tetris, Kirby's Dream Land), VenusRT (A Link to the Past, Donkey Kong Country) and Nephrite (Sonic the Hedgehog, Streets of Rage), each built at WiseMan 9c37e7a9 and on this branch and run by the kit of each: every case passes, and the reports are the same line for line but for the minor (1.4, 1.5) and C13's count of rounds. Their crates' tests pass: MoonRT 6, MercuryRT 8, 16 and 4, VenusRT 143, Nephrite 140, MarsRT 490 |
| **The settle bound per disc** | §52.1's table, measured on the code the seed has kept unchanged; through the published library the kit's C8 finds every disc's picture settled within the bound after its load at frame 900 under every setting |
| **Rewind in Mistress** | §53.4 |
| **WiseMan** | **330 of 330** [M] under the test lock with ps1-tests and the bench's discs, filtered by `Diamond`, `PlayStation`, `CoreAbi`, `CoreAdapter`, `Rewind`, `SaveState`, `StateProperties`, `GraphicsSettings`, `WindowFitAudit` and `CoreRegistrationEquivalence` |

### 53.4 Sizes, rewind and the time to save

**State sizes** [M] with a DualShock and a card (the kit's machine: two pads and a card, 30 bytes more):

| Everything on (precise geometry, true colour, 3D filtering) | Before | After |
|---|---|---|
| 1× | 12,642,696 | 5,398,740 |
| 2× | 26,503,560 | 5,398,740 |
| 4× | 81,947,016 | 5,398,740 |

**Rewind** (`DiamondRewindTests`) [M]: each disc by its press script, every enhancement on at 2× with filtering of
everything, a snapshot every 30 frames as Mistress takes them since §4.112 of the settings reference, each encoded
against the one before by Mistress's codec; the mean delta over the run, and the reach it gives under Mistress's
96 MiB:

| Disc, frames captured | Before: delta, reach | After: delta, reach | Times as far |
|---|---|---|---|
| Spyro the Dragon, 4,000 to 7,000 | 4,987,653 bytes, 10.1 s | 614,696, 81.9 s | 8.1 |
| Ape Escape, 3,000 to 6,000 | 2,342,375, 21.5 s | 623,716, 80.7 s | 3.8 |
| Crash Bandicoot, 3,000 to 6,000 | 2,307,814, 21.8 s | 406,153, 123.9 s | 5.7 |

Before, Spyro the Dragon's 20 snapshots of 3,000 frames filled the budget; after, 99 deltas held 49.5 seconds with room
to spare, and a rewind to the middle moment landed on the state held, byte for byte, on all three.

**Saving and loading** [M, the fastest of five, Crash Bandicoot at frame 1,500, everything on]: the export's settle and save, and a load into a second machine with
its renderer's seed:

| Scale | Save, before / after (ms) | Load, before / after (ms) |
|---|---|---|
| 1× | 2.77 / 0.83 | 1.60 / 1.23 |
| 2× | 10.31 / 0.84 | 4.40 / 1.80 |
| 4× | 50.74 / 0.89 | 15.06 / 4.74 |

A save no longer reads the device back or copies its pictures, and costs the machine's own save at every scale. **The
seed was slower than the kept load at first** [M]: 10.00 ms at 2× and 38.69 at 4×, since it built the mirror and the
picture sample by sample with a division for each. It now makes each row of blocks once and copies it to the block's
other rows (`seed`), which gives the figures above; `a_state_at_n_is_the_machines_and_a_load_seeds` holds the seeded
mirror at 4× to the machine's VRAM, sample for sample, and the handed picture to the display's, pixel for pixel. An
attach and a change of scale seed the same way and gain the same.

### 53.5 Cost of the frame

No code on the frame's path changed: the seed, the save and the load are not on it. Measured all the same, by §50.1's
instrument (`examples/bench`, every function aligned to 64 bytes, each build in its own target folder, the builds
alternating on one core, the fastest mean of five rounds, milliseconds of the emulation thread a frame) [M]: **base**
WiseMan 9c37e7a9, extracted outside the repository; **final** this branch before the seed by rows, which is off the
frame's path:

| Run | Off, base / final | Everything on at 2× (precise geometry, true colour, filtering everything), base / final |
|---|---|---|
| `gpu/benchmark` | 2.342 / 2.341 | 2.537 / 2.551 |
| Crash Bandicoot, the first 3,000 frames | 2.347 / 2.334 | 2.663 / 2.649 |
| Tekken 3's fights, frames 7,500 to 10,500 | 3.726 / 3.750 | 4.401 / 4.362 |

Every pair within 0.6 per cent but the fights' last, 0.9 per cent the faster for the final build, all within the bench's
noise on a desktop that was busy beside the runs (a load of 3 to 4); every run of an item gave the same digests in both
builds. Crash Bandicoot's enhanced pair is of seven rounds: in the five of the first series the base's fastest was 2.655
and the final's 2.805, their medians 2.825 and 2.826, the final build's five rounds all slowed by something beside them;
the seven of the second series alternate as before.

### 53.6 Tests [M, 2026-10-14]

- **`diamond`**, `cargo test --release` with `EMUSEN_DIAMOND_PS1TESTS`, `EMUSEN_DIAMOND_DISCS` and
  `EMUSEN_GPU_DEVICE=6800`: 167 unit tests, `programs.rs` 81, `renderer.rs` 3, `precise.rs` 14, `scaled.rs` 7,
  `colour.rs` 3, `filtered.rs` 4; **`precision`** 6; **`diamond-renderer`** 2; **`diamond-v1`** 4; and without the two corpora the same counts, the corpus tests standing down. The
  scratch measurement of §52.1 and §53.4 stays in the probe's folder and is not committed.
- **`emusen-native`** 72 (one new: the bound written only when declared, within the schema's maximum);
  **`emusen-core-abi-check`** 6 and its check; **`emusen-core-conform`** 7 and 19 (two new).
- **Every other core's crate**, as §53.3.
- **WiseMan**, as §53.3: 330 of 330, `DiamondRewindTests` (3) new, `CoreAdapterTests` with the reader's new field.
- **Not run**: the full WiseMan suite; the handheld and the laptop (not part of this work); `emusen-gpu`'s suite
  (unchanged); the C# host's skew fixtures of §12.5 of `EmuSen_CoreAPI.md`, which do not exist as builds yet.

### 53.7 Predictions

- **P-S1, held in its first half, held for two discs of three in its second** [M]: the state is 5,398,740 bytes at
  every scale; rewind reaches 8.1 and 5.7 times as far on Spyro the Dragon and Crash Bandicoot, but 3.8 on Ape Escape,
  where five times was predicted: its accurate part changes more between snapshots (624 KB of 2.34 MB) than the other
  two's.
- **P-S2 held** [M]: every case on every disc with every setting, 4× at 1,800 frames among them, C7 and C8 comparing
  from frame 61 or 241; with all off E5's evidence. **Its premise did not**: each image's cases at 4× took 186 to 270 s,
  under even the old whole-image deadline of 600 s, since E5's overrun came mostly from C17's and C18's state digest at
  every frame of an 82 MB state, which the seeded state removed. The deadline on progress is kept for what it is, a
  rule that no longer confuses a slow image with a hung one (§52.6), and was not what let this run finish.
- **P-S3 held** [M]: the other cores' reports are unchanged but for the minor and C13's rounds.
- **P-S4 held, after a fix** [M]: a save at 4× falls from 50.74 ms to 0.89; a load takes 4.74 ms, under the 50
  predicted, once the seed made its blocks by rows; its first form took 38.69 and would have made loading slower than the
  kept form's 15.06 (§53.4).
- **P-S5 held** [M]: the frame's cost is the base's within the bench's noise, off and on (§53.5).

### 53.8 What is left

- A disc whose picture settles slower than the bound, by a feedback effect or a texture drawn once (§52.4), would fail
  the kit's C8 under the setting that shows it; none of the bench does in its first 3,000 frames, and a wider corpus of
  discs with the enhancements on is the way to find one.
- Silent Hill's fog with true colour on is the one measured case near the bound's scale (141 of 240); a game with a
  slower-forgetting effect would need the bound raised, or the plane's low bits kept in the state at a cost of 1 MiB
  × N² (§52.2).
- A held rewind shows the console's detail (§52.7). A host that wanted the enhanced picture there would keep the
  pictures of the moments it shows, as Mistress's reel keeps thumbnails, rather than the core keeping them in its
  state.
- The handheld's measurement of E5 (§50.5) is still owed.

### 53.9 Reading log

No emulator's source, shader or snippet was opened, and no description of any emulator's save states, rewind or
upscaler; no firmware, no game's code, nothing under `kernel/` or `kernel-harness/`, and the player's BIOS folders were
not read. Opened, all EmuSen's own: this page from §2 to §6, §14 to §31 in part, §33.5 and §37 to §51 whole;
`EmuSen_CoreAPI.md` §4.1 to §4.6, §5.4, §5.5, §6.4 to §6.9, §6.19, §12.5, §17 and §28; `EmuSen_CoreAPI_Extensions.md`
§1 and §13's shape; `EmuSen_Settings_Reference.md` §4.110 to §4.112; `Diamond_Players.md` §15 and §16; the source of
`diamond-renderer`, `precision`, `diamond-v1`, `diamond`'s `layer.rs` and `state.rs`, its state tests and the disc
gates' support; the kit's `lib.rs`, `corpus.rs`, `digest.rs`, `main.rs` and its tests; `emusen-native`'s `desc.rs`,
`exports.rs`, `mod.rs`, `schema.rs`, `sys.rs`, its header, baseline, machine info schema and test core;
`emusen-core-abi-check`'s check; the C# host's `CoreDescriptors.cs`, `CoreInterface.cs`, `CoreEngine.cs` and
`CoreMachine.cs` at the events; Mistress's `RewindBuffer.cs`, `XorDeltaCodec.cs` and `MainWindow.axaml.cs` at the rewind's
interval; WiseMan's `DiamondEnhancementsTests.cs`, `DiamondTests.cs`, `CoreAdapterTests.cs`, `CoreDiscoveryTests.cs` and
`MarsRtRewindTests.cs`; E5's measurement scripts in the probe. The ROM library was read, never written, for the other
cores' kit runs.

---

## 54. Following a shift: a trial, decided before it was built

*Written 2026-10-14, before any of its code.* §32.4 proposed that the processor tell a shift by a constant as it tells a
copy, so that precise geometry could follow the vertices Spyro the Dragon keeps shifted; the prediction was its precise
polygons in play from 36.9 per cent to about 93.5. The tester accepted it as a trial on the condition recommended: built
narrowly, measured on Spyro first, kept only if Spyro comes close to the prediction, and otherwise dropped with the
reason recorded. It widens `blackmoon-r3000`'s reporting interface (`BlackMoon_R3000.md` §14) by addition, which the
same yes covers.

### 54.1 The pattern, as the game runs it [R]

Read on the open kernel from the running machine's RAM at frame 7,250 of Spyro's press script (its first world, in
play), by Diamond's own disassembler in a scratch test, at the addresses the crack model's tracing had named (§32.3).
Three routines project, each as one form:

1. **MFC2 of SXY2 into a register A**, after RTPS (about 8004EE00h, 80026970h, and 8001FB24h and 8001FC14h, where A
   is taken before an INTPL that colours the vertex).
2. **SLL by 5 of A into another register B.** A is kept and used for the clip tests that follow.
3. **The clip flags set in B in place**: up to four ADDI of 1, 2, 4 or 8 (two of the routines), or ORI of the same
   (the third), each under a comparison of A's x or y with an edge of the screen; one routine adds 0Ch at once for both
   vertical tests. They fall in the five bits the shift cleared, and their sum is at most 15, so none carries out of
   them.
4. **SW of B** into a table of the model's vertices in RAM. B is also ANDed into a register that collects the flags
   common to every vertex, which is not a vertex word.

Two other paths store A unshifted (8002631Ch's branch not taken, and 80027ACCh's routine); their words are precise
already. The polygons are then emitted by three routines (about 8004EEB4h, 80026BBCh and 80020038h/80020344h), each
as one form:

1. **LW of three or four of the table's words** by the polygon's indices, each into its own register.
2. **The AND of them and ANDI 1Fh**: a polygon whose vertices are all off one edge is skipped.
3. **SRA by 5 of each register in place.**
4. **SW of each into the packet**, or MTC2 of each to SXY0 to SXY2 for an NCLIP that culls the back faces and then
   SW of the same registers.

**The shift back is arithmetic and in place, and restores the word to the bit.** SLL by 5 drops bits 31 to 27, which
are SY's bits 15 to 11; a projected SY lies within −400h..3FFh (`BlackMoon_GTE.md` §5.1), so those bits and bit 26 are
all copies of its sign, and SRA by 5 brings them back. The flags fall off in the same shift. The word the packet
carries is SXY2 as the GTE pushed it.

**So the serial is lost at one instruction, the SLL into another register**, which is neither a copy nor told. The flags
are set and the shift back is made in place, untold arithmetic on a register that keeps its serial (§21.3), so neither
loses it. When this reading began it was taken that the ADDI and ORI of the flags would have to be told too; read
closely they need not be, since each names one register as both source and destination. The trial tells shifts only.

**Spyro's "changed" words** (§25.5's 464,915, §32.3's 849 crack pixels) are the same SLL made into a register that
still held an earlier projection's serial: the store in step 4 gives the table that serial, and the receipt finds a word
of another vertex. Told, the shift gives the register the right serial instead.

**What the vertices "with none" are** (§32.3's 32,173 crack pixels; 163,014 quads of three precise vertices and one with
none, 25,951 triangles of one precise and two with none, over the run to frame 17,000). The words stored at 80027BCCh
and 80028E44h are each made from two neighbouring vertices' unshifted words: their x's summed and their y's summed on
the processor, weighted twice by the GTE's GPF with a factor in IR0, and the two results' sums shifted right by eight
and packed into a word. It is a vertex placed between two projected ones, made on the processor and the GTE as Ape
Escape's divided polygons' are (§32.3). No projection is behind it, and following a shift does not reach it; it stays
§32.4's second item, for which no exact remedy is proposed.

### 54.2 The move [C]

**`Move::Shift`**: SLL, SRL or SRA by a constant of 1 to 31, its register given `from`'s value shifted; the fields
`at`, `register`, `from`, `kind` (left, right logical or right arithmetic), `amount` and `value` (the result). It is
told under `if B::MOVES` where `Copy` is, from the same arm of the computing instructions, so that the step compiled for
a bus whose `MOVES` is false holds no trace of it (§9.2, `BlackMoon_R3000.md` §14). A shift by 0 stays a `Copy`, and a
shift into R0 is told nothing, as every move into R0.

- **Every shift by a constant, in place or not.** The processor reports what it did and the layer decides what it
  means. An in-place shift tells the precision layer nothing it does not hold already; it is told so that the report
  is a plain class of instructions, not the precision layer's policy written into the processor. Its cost is
  measured (§54.5), and should it show, telling only a shift into another register is the narrower form to take.
- **Not told: SLLV, SRLV and SRAV**, the shifts by a register's amount. No routine read in §54.1 shifts a vertex word
  that way (the SLLV at 8001FB98h scales model data before RTPS), and nothing is added for a pattern no game here
  shows.
- **Not told: the arithmetic and logic that sets the flags.** §54.1 shows it in place.

`Move` is an enum the crate owns and its consumers match. A variant added to it is a change to every exhaustive match
outside the crate, which within the project are three (the precision layer, Diamond's test observer and the r3000's
own test bus), each changed with it. `BlackMoon_R3000.md` §14 records the moves' interface as **version 2**: version 1
is E1's seven moves with E4's store order, and version 2 adds this one and changes none of the seven.

### 54.3 What the precision layer does with it [C]

**The serial is carried, whatever the kind and the amount**: `Shift` gives `register` the serial of `from`, as `Copy`
does. **No shift state is kept** beside a serial. A serial names a projection, and the integer check at the receipt
(§21.3) compares the whole word the GPU takes with the integer word the projection pushed; a word that passes is that
projection's to the bit, whatever the register did between, and one that fails is drawn as the integer it is. A state
recording the net shift and the bits set would refuse early only what the receipt refuses, and would add a case to get
wrong.

**When a serial ends**, as before: a move by halves or bytes, a load from anything but RAM and the scratchpad, a register
or word given another serial by a later move, an entry the ring has written over. **At the receipt** every word that is
not its projection's falls back to the console's vertex, and that is where each of the following ends: a shift that
lost significant bits (SLL by 9 and SRA by 9 of a word whose SY is beyond ±127), SRL in place of SRA of a negative SY,
a value altered between the shifts (an ADDI of 20h, a whole pixel of x, in place of a flag), a flag carried out of the
cleared bits.

**Why no wrong value can reach the screen.** Nothing the layer gives the renderer changes: a precise vertex is given
only when its serial is live and its projection's word is the word received, and the guard of §21.2 has already refused
at the projection any position more than two pixels from the console's. More registers now carry serials: a register
given a shifted vertex for some other use (Spyro's `SLL by 16` of A for its y test, §54.1) holds a serial that no
receipt will match, since its word is not the projection's. The one admission is §21.3's: a word that came to equal a
projection's by some other route is that projection's pixel, and its offset is within the guard of it.

**The test observer** counts a shift among its copies, a register given another's value, and folds its kind and amount
with its values; its block keeps its nine counts as before, so its layout is not changed (§4, §7.1).

### 54.4 Tests to be built

- **`blackmoon-r3000`**: `each_move_is_told_with_its_values` extended with SLL, SRL and SRA by a constant into another
  register and in place, a shift by 0 told as a copy, and a shift into R0 told nothing; `a_run_is_its_steps_on_random_programs`
  already compares every move the run tells with the steps', and so holds the new one; the suite's 55,000 cases
  unchanged.
- **`precision`**, programs of the crate's own built on its projected triangle: **Spyro's sequence** (MFC2, SLL by 5
  into another register, two flags set by ADDI and ORI in place, SW to RAM, LW back, SRA by 5 in place, SW into the
  packet, sent by the processor and by DMA), precise with the change and integer before it, shown failing first; and
  three that must stay integer: a shift that discards significant bits, a pixel added between the shifts, and an
  altered word after an exact pair of shifts.
- **The machine's**: E1's goldens plain and with the test observer, and the machine's goldens with precise geometry
  on, unchanged.

### 54.5 The keep rule, fixed before measuring [C]

**Measured on Spyro the Dragon first**, at 1× with the crack fill as built, by §32.2's stretch and §32.1's model:
frames 5,300 to 17,000 of its press script, every 50th frame looked at; before, on this branch with the change absent,
and after. Recorded: the precision layer's polygons and vertices precise (from power-on to frame 17,000, as §32.3's 36.9
per cent was counted, and within 5,300 to 17,000 alone), the crack pixels unfilled (§32.2's 58,023) and left by the fill
(by the model of §33.3, 194), and §32.3's classes.

**Kept if all of these hold:**

1. **Spyro's precise polygons to frame 17,000 reach at least 88 per cent.** The prediction is 93.5; 88 is 5.5 points
   short of it, a tenth of the 56.6 points of gain predicted. Anything less means the pattern of §54.1 does not account
   for the words it was taken to account for, and the trial is dropped.
2. **The lost class's crack pixels, unfilled, fall by at least nine tenths** (16,952 before).
3. **Nothing gets worse elsewhere.** On the other seven discs at their anchors, and on Ape Escape and Tekken 3 where
   §32.2 measured them in play, no disc's precise share of polygons falls and no count of crack pixels rises, unfilled or
   left by the fill; a rise is traced to its cause, and the trial is dropped if the cause is the change.
4. **Off, the frame is the base's within the bench's noise** (§27.1's instrument), the R3000A's step compiled for the
   board as it was.

**Otherwise dropped**: the code reverted on this branch, these pages kept with the finding and its numbers.

**If kept**, the rest of the gate: the eight discs' precise shares and crack counts at their anchors and in play;
E1's goldens with the setting off and the machine's with it on; zero fraction E3's equality; E4's, the fill's and
E5's tests; the kit on the published library with the setting on, on the eight discs at 1,800 frames; WiseMan's Diamond
and PlayStation areas; the precise share on Spyro at 2× (E5's internal resolution draws through the same records); and
the cost, off against the base and on before and after, on `gpu/benchmark`, the first 3,000 frames of Crash Bandicoot,
Spyro the Dragon and Ape Escape, and Tekken 3's fights.

## 55. Following a shift, measured and decided

*Measured 2026-10-14 on the desktop, the RX 6800, at 1× with the crack fill as built.* Each run was made twice by one
build: as §54 built it, and with every `Move::Shift` dropped before the precision layer (a scratch switch in the crack
model's layer), which is the layer as it was. The two agree with the base on everything the change does not touch:
Spyro's model before reproduces §32.2 and §32.3 to the pixel (58,023 cracks, 194 left by the fill, every class).

### 55.1 Spyro the Dragon [M]

| Spyro the Dragon | Before | After |
|---|---|---|
| Polygons precise, power-on to frame 17,000 (§32.3's measure) | 36.9% (3,343,597 of 9,050,763) | **95.9%** (8,680,658) |
| Polygons precise within frames 5,300 to 17,000 | 39.2% | **95.2%** |
| Vertices precise, power-on to 17,000 | 40.5% | **98.1%** |
| Polygons precise to frame 3,000 (§25.5's measure) | 14.8% | **98.2%** |
| Vertex words changed (a live serial, another word) | 997,374 | 70,100 |
| Serials found dead | 15,924 | 0 |
| Crack pixels in play, unfilled (§32.2's 235 frames) | 58,023 | 55,144 |
| … left by the fill (§33.3's model, the built form) | 194 | **135** |
| Crack pixels at the ten anchors, unfilled / left by the fill | 664 / 0 | 475 / 0 |

The crack classes in play, §32.3's table read before and after, the lost class split by the store that first wrote
the word:

| Class, crack pixels in play | Before | After |
|---|---|---|
| Beside a polygon drawn by the integer rule | 53,924 | 41,959 |
| … its worst vertex **none** | 32,173 | 33,217 |
| … **lost**, its word first stored by the three shifting routines of §54.1 | 12,717 | **0** |
| … **lost**, its word first stored elsewhere | 4,235 | 4,457 |
| … **changed** | 849 | 334 |
| … **far**, **cut** | 3,769, 180 | 3,770, 180 |
| … along a shared edge, at a T, at a corner, apart | 40,364, 5,911, 5,817, 1,832 | 35,775, 4,760, 973, 451 |
| Between two precise polygons (at a T, apart, along an edge, at a corner) | 2,603 (1,108, 805, 412, 278) | 10,717 (4,920, 2,872, 1,582, 1,343) |
| Beside a rectangle, a fill, a copy, nothing drawn, or no other owner | 1,496 | 2,468 |

- **Every word the pattern stores is followed.** No crack pixel in play stands beside a vertex lost from the three
  routines, against 12,717 before; Spyro's changed words fall from 997,374 to 70,100 and its dead serials, which were
  stale serials in the registers the SLL did not report, to none.
- **What §32.3 called lost was wider than the pattern.** A word is "lost" in the model when no serial reached it and it
  equals a recent projection's word. A quarter of the class before (4,235 of 16,952) is words that the pattern never
  stores: the vertices made between two projected ones by GPF (§54.1), and those a routine about 8004FDACh makes where
  an edge between two screen vertices crosses a line (its intersection computed by multiplications and a division on
  the processor), each equal to a projection's word where the made vertex falls on a projected one. They are made
  vertices, as the class "none" is, and remain integer.
- **The cracks moved more than they fell.** Where a polygon that was lost met a precise one along an edge, both are
  now precise and the crack is gone (edge 40,364 to 35,775, corner 5,817 to 973). But the ground that is now precise
  meets the made vertices of the class "none" and its own T's (between two precise polygons, 2,603 to 10,717), so the
  unfilled count falls by a twentieth, not by the lost class's 16,952. The fill closes these as it closed the others:
  left by it, 194 to 135.

### 55.2 The other discs [M]

The precision layer alone (a scratch test of its counts) on every disc by its press script to frame 3,000, Ape Escape
to 20,500 and Tekken 3 to 12,000; and the crack model at the anchors of every disc, in Ape Escape's play (frames 7,300
to 20,500) and in Tekken 3's fights (7,500 to 10,500), before and after:

| Disc | Polygons precise before and after | Crack pixels, unfilled / left by the fill |
|---|---|---|
| Ape Escape, to 3,000 / to 20,500 | 78.5 / 90.2%, the same | anchors 632 / 0, play 17,611 / 2,149, the same |
| Crash Bandicoot | 99.3%, the same | anchors 345 / 0, the same |
| Mega Man Legends | 96.0%, the same | 8 / 0, the same |
| Silent Hill | 35.1%, the same | 29 / 0, the same |
| Rayman | 3.9%, the same | 0, the same |
| Castlevania | 0, the same | 0, the same |
| Tekken 3, to 12,000 | 75.9%, the same | fights 2,609 / 0, the same |

On every one of them the pictures the model compares (the precise picture's pixels against the machine's, counted at
every frame looked at) are the same before and after, to the pixel. **No other disc gains**: Ape Escape's 3,216
changed words to frame 3,000 (8,789 to 20,500) become words with no serial, and every polygon they touch was integer
before and stays so; the likely reading [A, from the counts] is a shift that now gives its register the serial of an
operand no projection reached, where the register kept a stale one before. Tekken 3's 1,757,225 changed words are its moved vertices (§32.3) and are unmoved by the
change.

### 55.3 The keep rule, item by item

| Item (§54.5) | Result |
|---|---|
| 1. Spyro's precise polygons to frame 17,000 at least 88% | **Met**: 95.9%, above the 93.5 predicted |
| 2. The lost class's crack pixels, unfilled, down nine tenths | **Not met by its letter**: 16,952 to 4,457, down 73.7%. **Met for the pattern's part**: 12,717 to 0 (§55.1) |
| 3. Nothing worse elsewhere | **Met**: the seven other discs' shares and pictures unchanged; Spyro's cracks left by the fill 194 to 135, at the anchors 0 and 0 |
| 4. Off, the base's frame within the noise | (§55.4) |

**Decided [C]: kept, with item 2 put to the tester.** The rule the tester set was the first and the third: Spyro near
the predicted share, nothing worse on the other discs; both hold. The second was added in §54.5 to check that the
pattern accounts for the words it was taken to account for, and its measure turned out to count words the pattern
never touches; the split, made after the measurement and said so, shows the pattern's words followed without
exception. Item 2 is recorded as failed as written. Should the tester hold it to its letter, the change is the
commit that adds the move and the layer's arm, and reverting it restores the layer to the byte.

### 55.4 Cost [M]

§27.1's instrument: `examples/bench` on links with every function aligned to 64 bytes, each build in its own target,
the builds alternating on one core, the fastest mean of five rounds, milliseconds of the emulation thread a frame. The
base is WiseMan 00a83847, extracted outside the repository. A third build, **narrow**, told only a shift into another
register, the form §54.2 named should the cost show. A first series, run while other work loaded the machine (load
average 7 to 15), is set aside; this one ran at a load of about 3:

| Run | Frames | Off: base | branch | narrow | On: base | branch | narrow | Following a shift, on |
|---|---|---|---|---|---|---|---|---|
| `gpu/benchmark` | 1,200 | 2.346 | 2.349 | 2.346 | 2.560 | 2.565 | 2.569 | +0.005 |
| Crash Bandicoot | 3,000 | 2.325 | 2.321 | 2.316 | 2.645 | 2.683 | 2.702 | +0.038 |
| Spyro the Dragon | 3,000 | 2.364 | 2.362 | 2.358 | 2.754 | 2.823 | 2.838 | +0.069 |
| Ape Escape | 3,000 | 2.179 | 2.167 | 2.178 | 2.391 | 2.406 | 2.410 | +0.015 |
| Tekken 3's fights | 7,500 to 10,500 | 3.720 | 3.705 | 3.708 | 4.319 | 4.446 | 4.433 | +0.127 |

Every item's digests equal across the six forms.

- **Off, the frame is the base's within the noise** (−0.6 to +0.1 per cent): item 4 of §54.5 **met**. The plain
  frame's functions are the base's to the byte in size (`advance_in::<Plain>`, `exec::run` and `Run::execute` for the
  board, by `nm`); only the layered frame's `exec::run` and `advance_in::<Layered>` grew, by 177 and 300 bytes.
- **On, it costs 0.04 to 0.07 ms a frame on the three discs and 0.13 in Tekken 3's fights**, 1 to 3 per cent of the
  frame with the setting on. Crash Bandicoot's first 3,000 frames tell 37.4 million shifts, 12,500 a frame, where they
  told 57,000 other events a frame [M, `bench --observed`'s counts]; at §9.3's 3 to 6 ns an event that is the cost
  measured. Spyro's includes the work of the polygons now drawn precisely.
- **The narrow form is not taken**: it tells 64 per cent fewer shifts on Crash Bandicoot (4,500 a frame) and was no
  faster on any item (Crash 2.702 against 2.683, the fights 4.433 against 4.446). The dispatch, not the count, is the
  floor here, and the uniform report stays.

### 55.5 The gate [M]

| Item | Result |
|---|---|
| E1's goldens, the setting off, plain and with the test observer | **Equal**: `programs.rs`' three, the eight discs' 80 anchors among them |
| The machine's goldens with precise geometry on | **Equal**: the programs, ps1-tests and the eight discs to frame 3,000 |
| Zero fraction is E3's equality | **Equal** on every disc at every frame and anchor, now through 2,733,994 precise triangles of the discs, Spyro's 1,599,362 among them (307,276 at E4), its pictures differing in none of 3,000 frames |
| E4's, the fill's and E5's tests | **Pass**: `precise.rs` 14, `scaled.rs` 7, `colour.rs` 3, `filtered.rs` 4 |
| Spyro at 2× | **The same**: to frame 17,000 the precision layer's counts and the renderer's (10,793,880 precise triangles, 4,864,310 in perspective, 177 polygons to the integer path for a gradient) are those of 1× to the count |
| The kit, the published library, the setting on, the eight discs at 1,800 frames | **Every case on every disc**, four at a time: 341 s on this branch, and 205 s after WiseMan's seeded state was merged in (§53), the library at the core ABI's minor 1.5 |
| After a load, the layer reset at every seed (§52, §53) | **Followed again within five frames**: Spyro saved at frame 7,000 with the setting on and loaded on a second machine; over frames 7,000 to 7,005 the loaded machine draws 1,016 of 1,626 polygons precise where the one that saved draws 1,523, and from 7,005 to 7,030 the two draw the same counts in every five frames. Following a shift keeps nothing a state would need: a register's serial is given at the shift from one the next projection makes |
| WiseMan's Diamond and PlayStation areas | **68 of 68**, and 71 of 71 on the head with the seeded state merged |

### 55.6 What the pictures showed

Spyro's pictures at every 50th frame from 900 to 11,500 were written before and after, and those at 900, 5,900,
7,250, 10,350 and 11,500 looked at beside the machine's, 5,900 being where the after picture differs most from the
before (the whole scene precise) and 10,350 where the most unfilled cracks were added. The scenes are steady where
they wobbled; no corner stands away from the console's by more than the guard's two pixels, which the guard makes
impossible; no crack is left by the fill there. **Seen and not the change's**: at 7,250 a fan of light and dark wedges
on the near grass below the dragon, in the picture before and after alike and not in the machine's; those polygons
were precise before the change. Not traced; recorded for the next look at Spyro's perspective texturing.

### 55.7 Tests [M, 2026-10-14]

- **`blackmoon-r3000`**, `cargo test --release` with `EMUSEN_BLACKMOON_CORPUS`: 38, `each_move_is_told_with_its_values`
  extended, the random programs telling 18,156 moves (15,479 before), the suite's 55,000 cases unchanged.
  `blackmoon-gte` unchanged and not run.
- **`precision`**: 8 (two added: Spyro's sequence, shown integer before the change; the three words not restored).
- **`diamond`**, with `EMUSEN_DIAMOND_PS1TESTS`, `EMUSEN_DIAMOND_DISCS` and `EMUSEN_GPU_DEVICE=6800`: 279, and 279
  without the corpora, the corpus tests standing down. **`diamond-renderer`**: 2. **`diamond-v1`**: 4, and 4 naming
  no device.
- **The kit** with the setting on (§55.5). **WiseMan**, under the test lock with the corpus, `Diamond` and
  `PlayStation`: 68 of 68, and 71 of 71 on the final head.
- **On the final head**, after `git merge WiseMan` brought in the seeded state (§52, §53): `diamond` 279 with the
  corpora and 279 without, `precision` 8, `diamond-renderer` 2, `diamond-v1` 4, the kit and WiseMan as above.
- **Not run**: the full WiseMan suite; the kit with the setting off (the plain frame's functions unchanged, §55.4);
  `emusen-gpu` and the other cores, unchanged.

### 55.8 Predictions and what is left

- **§32.4's prediction held, and was passed**: Spyro's precise polygons in play from 36.9 per cent to "about 93.5":
  95.9. "The 16,952 crack pixels of the lost class gone at their source" held for the 12,717 the pattern makes and not
  for the rest, which are made vertices (§55.1). "More of the 32,173 beside a mixed polygon whose other vertices are
  lost" did not hold: that class rose to 33,217, since the made vertices now stand beside precise polygons.
  "A few per cent of instructions" for the events: 22 per cent more events on Crash Bandicoot, 0.04 to 0.13 ms (§55.4).
- **§25.5's "Spyro the Dragon under a fifth" and P-E4c's failure for Spyro are retired**: 98.2 per cent to frame 3,000.
- **Left**: Spyro's made vertices (between two projected ones by GPF, and where an edge crosses a line about
  8004FDACh) stay integer, as Ape Escape's divided polygons do; no exact remedy exists for them (§32.4). The fan on
  Spyro's near grass (§55.6). Item 2 of the keep rule is the tester's (§55.3).

### 55.9 Reading log

**No emulator's source, shader or snippet, no description of PGXP or of any emulator's precision, and no other
PlayStation renderer's code were opened**, with or without code. No firmware, listing or trace of one; the player's
BIOS folders were not read; nothing under `kernel/` or `kernel-harness/` was opened or changed. psx-spx was not opened,
and no published prose on the subject was read. The handheld was not connected to.

**One game's code was read**, Spyro the Dragon's on the open kernel, from the running machine's RAM at frame 7,250 by
Diamond's own disassembler in a scratch test: about 8004EE00h to 8004EF2Ch, 80026960h to 800269ACh, 80026B30h to
80026E1Ch, 800262E0h to 80026354h, 8001FA80h to 8001FDC0h, 80020030h to 800200A4h, 80020330h to 800203A4h, 80027A40h to
80027BDCh, 80028DA4h to 80028E5Ch, 8004FAB0h to 8004FC0Ch, 8004FDACh to 8004FE68h, 80050040h to 800500DCh and
80058080h to 8005811Ch. What §54.1 and §55.1 say of them is what those listings show; no listing, picture, state or disc
image is in the repository.

Opened, all EmuSen's own: this page's §1 to §13, §21 to §36 and §51; `BlackMoon_R3000.md` §14; the BlackMoon and
Diamond READMEs; `Diamond_Players.md` §15. The source of `blackmoon-r3000`'s `lib.rs`, `exec.rs` and `boundary.rs`;
of `precision`; of `diamond`'s `layer.rs`, `tests/support/goldens.rs` and `examples/bench.rs`; of `diamond-renderer`'s
`Renderer` and `Counts`; the crack model and the cost scripts of the crack fill's probe folder.

---

## 56. The picture on the device (core ABI 1.9), as built, 2026-10-15

*The tester's go of 2026-10-15 to move the game's picture off Skia (`EmuSen_DevicePicture.md`); its §7 is this
section's design and its §16 the step's record.*

### 56.1 What changed

- **The slots are `emusen-gpu`'s.** The renderer's two host picture slots and `Back::Slot`'s copy became
  `PictureOut` (`EmuSen_Gpu.md` §5.1): on a device of the renderer's own, the same copy into the same host slots; on a
  lent device with the host's `done`, device slots shared with the host's queue family, the copy followed by `ready`
  signalled with the picture's value.
- **Lent, the picture is the frame's own.** `frame_end` submits the frame as §38.1 does, and the host is handed **the
  picture of this frame**, not the one before: the host waits for it on the device (`ready`), so nothing needs it in
  host memory before `advance` returns. Off the device the rule of §38.1 stands unchanged, the frame before.
- **`frame_copy` still answers.** On the device `handed` waits for the pending submission and copies the slot into
  the readback buffer (`back`), which is what a screenshot, the kit and every tool read; a host that presents from the
  device never calls it.
- **A seed on the device** goes up through `back`: VRAM and the palette cache first, then the picture, handed into
  slot 0 with its own value, so that after a load or an attach the first picture handed is the one §38.1 names.
- **`frame_device`**, the core ABI's export, answers the slot, size, `ready` and value of the picture handed, and tells
  `PictureOut` that the host has it, after which the slot is not written until `done` passes that value. A picture
  never asked for binds nothing, so a host that lends and reads `frame_copy` alone never stalls the machine.
- **Lent or not is decided at create.** `Diamond::create` keeps the lend; the renderer is made on it when the
  enhancements first draw, and on a device of its own when nothing is lent or the lend is refused (a version below
  1.2, a queue without compute). A renderer rebuilt for new options keeps its device.
- **The capability bit `DEVICE_PICTURE` is claimed**, and `core_exports!` gives the export.

### 56.2 A defect of the renderer's drop, found by the validation layer

The kit's run under the validation layer (`VK_LAYER_KHRONOS_validation`, `VK_LAYER_VALIDATE_SYNC=1`,
`VK_LAYER_SYNCVAL_SHADER_ACCESSES_HEURISTIC=1`) gave 961 findings with the device lent: `vkDestroyBuffer`,
`vkDestroyPipeline` and `vkDestroyDescriptorPool` on objects a submission was still using
(`VUID-vkDestroyBuffer-buffer-00922` and its two kin). **The defect is older than the lend** [M]: with the lend
disabled and the code otherwise as before, the same run gave 478 findings of the same three kinds. `Device`'s drop
released its buffers and programs before `GpuDevice`'s own drop waited for the device, so a machine dropped with a
frame's submission pending destroyed what that submission read; nothing in the gates drops a machine mid-run, and the
kit does at every case. `Device::drop` now waits for the pending submission first: **0 findings** over the kit's 20
cases on `animated-triangle.exe` at 2×, the lent machines of C20 included.

### 56.3 Proof

- **Unchanged with no device lent** [M, `cargo test --release`, the RX 6800, `EMUSEN_DIAMOND_PS1TESTS` set]: the
  renderer's 2 tests, `diamond-v1`'s 4, and the machine's suites, the accurate goldens, the scaled, true-colour,
  filtered and precise gates and the state digests among them, all passing as before.
- **With a device lent** [M, the kit, `emusen-core-conform --core libdiamond.so --image animated-triangle.exe --frames
  200 --settings resolution=2`]: C20 passes, 120 of 120 pictures handed on the device, each read by the kit on its own
  queue equal to the machine's `frame_copy` of that frame, and the lent machine's `frame_copy`s equal to the lendless
  machine's **a frame later**, every one of 120; the picture held through two `advance` calls unchanged. The other 19
  cases pass. At `resolution=1` the renderer is off and every picture goes by `frame_copy`, as C20 notes.
- **The latency, in frames** [M, the same run]: one frame without a device lent, as §38.1 states; **none** with one,
  the picture handed the frame it is drawn. What the host then adds before the picture is presented is the host's
  (`EmuSen_DevicePicture.md` §16).
- **A mutant**: `PictureOut::hand` with the wait for `done` removed fails C20 with "the picture of frame 60, held
  until now, was written over before done let go of it".

### 56.4 What it leaves

The setting's cost still states §38.1's frame of latency, which holds for every host that lends nothing, the
reference host included until it lends (`EmuSen_DevicePicture.md` §16). The latency a player sees is the host's to
measure once it presents from the device.

