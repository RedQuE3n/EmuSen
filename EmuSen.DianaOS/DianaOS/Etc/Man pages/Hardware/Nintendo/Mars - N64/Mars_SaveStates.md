# Mars — save states

*Phase F's first slice, landed 2026-09-18. `MarsCore.SaveState`/`LoadState`, `MemoryBus.WriteState`/`ReadState`,
`SaveChip.WriteState`/`ReadState`, and the shared `EmuSen/Common/StateSerializer.cs`. Tests in `MarsSaveStateTests`
and `MarsCoreTests`; the games through a replay check (§4). The shared format is `EmuSen_Save_States.md`.*

---

## 0. What grades this

**A save state has one job: a machine loaded from it must go on exactly as the machine that wrote it would have.**
No reference can grade that and no corpus tests it, but it can be measured directly, because Mars is deterministic
(`Mars_Save.md` §8): save, run on, load into a fresh core, run on again, and compare everything. That is §4's replay,
and it is the whole of the evidence for the slice. The rest of this page is the design it grades.

## 1. The format

| | |
| --- | --- |
| magic | `0x5352_414D`, "MARS" in the file's first four bytes |
| version | 1 |
| RDRAM size | 4MB or 8MB |
| frames | `TotalFrames`, and the length of the last frame, which `FrameRateHz` reports |
| the processor | `Cpu`, reflected (`EmuSen_Save_States.md` §1) |
| the bus | `MemoryBus` reflected — RDRAM, its hidden bits, the RSP's memories, PIF RAM and every device it owns — then §3's tail |

**Three things are refused before anything is read into the machine**: a file that is not a Mars state, a version
this build does not know, and a state saved with a different amount of RDRAM, which would otherwise read four
megabytes into an eight-megabyte array and misalign everything after it. `A_state_for_another_machine_is_refused_before_anything_is_read`
holds all three, and checks the machine is untouched. A path save checks for a ROM before it creates the file, so a
failed save leaves no empty state behind, which is what the old refusal was careful of too (`Mars_Core.md` §6).

**The ROM is not in the state**, and nothing checks that a state is loaded into the game that wrote it. This is the
other cores' behaviour as well; it is named here because a state for another game would load, and run nonsense.

**A state is 6.4MB** for a 4MB machine: four megabytes of RDRAM, two of its hidden bits, and about 400KB of
everything else. The rewind buffer stores deltas between them (`EmuSen_Rewind_And_FastForward.md` §1), so what it
costs per step is what changed, not the whole.

## 2. What is left out, and why each is safe

- **Every device's reference back to the bus.** Each device holds the bus that owns it; walked, each would write the
  whole machine again, and the walk would not end.
- **The ROM** (`MemoryBus.Cart`). `LoadRom` reads it before any state can be loaded.
- **The video interface's raster**, 1.4MB rebuilt by every scan. A loaded state is presented by scanning RDRAM again,
  and §4's replays compare that picture frame by frame, so a raster that mattered would show.
- **Audio a frontend has not drained yet.** It belongs to the moment before the load; a loaded state starts the queue
  empty (`Loading_a_state_drops_the_audio_a_frontend_has_not_drained`).
- **The debug port's transcript**, which is the harness's record of the port and not the port.
- **The processor's exception scratch** (`Cpu.LastException` and the one instance it reuses), written before each use.

## 3. What reflection cannot walk

`StateSerializer` fills what already exists; it cannot create an object a state has and the machine does not. Three
parts of the bus are written by hand after the reflected fields, for that reason or because they are collections:

- **The unmodelled registers** (`Mars_Memory.md` §2.2), a dictionary, written as a count and sorted pairs.
- **The save chip**, written as its type and then its device. A chip named after the machine loaded — by the game's
  first move (`Mars_Save.md` §1) — has no device in a fresh core, so the type comes first and the device is built
  before its bytes are read.
- **Each port's Controller Pak**, present or not, created when the state has one and the port does not.

**A loaded chip and pak are marked changed.** The state's copy is now the game's, and marking it means the next
battery write (`Mars_Save.md` §7) puts it on disk; otherwise a state older than the last save would leave the file
holding one thing and the machine another until the game next wrote. The consequence is the usual one: loading a
state rewrites the battery save.

## 4. What was measured

**The replay, on all three games** (Release, a scratch program against the built assembly): each game run to a
frame, saved, and run 100 frames on; the state loaded into a *fresh* core — a new `MarsCore`, `LoadRom`, one frame,
then the load — and run the same 100 frames beside the original. The picture, all of RDRAM, the program counter and
the cycle count were compared every frame:

| | saved at | identical for |
| --- | --- | --- |
| Super Mario 64 (Europe) | frame 300 | 100 frames |
| Wave Race 64 (USA) | frame 250 | 100 frames |
| Ocarina of Time (Europe) | frame 300 | 100 frames |

A first run loaded into the same core instead, which passed too and proves less: a field the state forgot would still
hold a plausible value from the same run. A fresh core has nothing but what the state gave it.

**`A_fresh_core_loaded_from_a_state_keeps_step_with_the_one_that_saved_it`** is the same check in the unit suite, on a
counter the processor keeps in RDRAM and on very short fields, twenty frames each side.

**`Every_field_the_state_walks_comes_back_in_a_fresh_core`** fills every field the state walks with noise — the
processor, the bus, every device, their arrays, structs and struct arrays — saves, loads into a fresh core and compares
every one. It covers what the games never touch in the frames measured: `The_tlb_comes_back` names the table none of
the three games maps a page with.

**The breakage round**, twelve rules, each caught by a named case:

| rule broken | caught by |
| --- | --- |
| a struct array's element is stored back after its read | the noise test, the TLB, the serializer's own |
| a struct field is stored back after its read | the noise test, the serializer's own |
| the unmodelled registers, the save chip, a missing pak created, the chip and the pak marked changed | the chip-and-pak case, one each |
| undrained audio is dropped | the audio case |
| the magic, the RDRAM size | the refusal case |
| the frame count | the fresh-core case and the file round trip |
| the processor at all | the fresh-core case, the noise test and the TLB |

**What the noise test cannot see is a field wrongly marked skipped**, because it compares only what the state walks.
Only a replay can catch that, and only for a field the game's next frames depend on. The check was run both ways,
with Super Mario 64's replay:

- **COP0 marked skipped** — the processor's status, timer compare and exception registers — diverged on the first frame
  after the load, in RDRAM and in the program counter. The replay sees a field the game depends on at once.
- **The RSP's vector accumulators marked skipped** stayed identical for all 100 frames. Nothing noticed: not the replay,
  not the noise test, not the unit suite. Whether that is because the game's microcode never carries an accumulator
  from one task into the next, or because the frame boundary happened to fall between tasks, was not established.

So the evidence is two-sided and says so: a skip that matters to the frames replayed is caught; a skip that does not
is not, and would stay hidden until some game's microcode, at some moment, depended on the field. The accumulators are
in the state, which is the only defence this page can offer against it.

## 5. The serializer

Mars needed four things `StateSerializer` did not have: `uint[]`, `long[]` and `ulong[]` arrays, `char`, struct fields,
and struct arrays that are restored rather than only read. They are recorded in `EmuSen_Save_States.md` §5, with the
one thing they changed for another core: Venus's decoded palette had never been restored by a load.

## 6. What this is not evidence for

- **That a state survives a change to Mars.** The format is positional (`EmuSen_Save_States.md` §1); any field added,
  removed or renamed anywhere on the processor or the bus changes the layout, and this version has no read path for
  an older one. It is version 1 so that the next can have one.
- **That the three games' 100 frames exercise everything.** They exercise what those frames use. The noise test covers
  the walk, and the two together still cannot see a skipped field no replay depends on.
- **Anything about a state taken mid-frame.** States are taken between frames, as on the other cores.
