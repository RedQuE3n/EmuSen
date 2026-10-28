# Two faces of one core: the cycle face and the byte face

*Decided 2026-10-28 by the tester, for every core, those built and those to come. This page states the rule, argues
for its form, and sets its conditions, its acceptance test and its order. The first core to apply it is Sapphire, whose
byte face is designed in `Sapphire_Redesign.md`.*

## 1. The rule

Every core has two faces over one body.

- **The cycle face** is the reference. It is built first, it orders every event of the machine by its cycle, and it is
  never bent for speed. It is graded by independent oracles: test programs with recorded console output, hardware
  documents, and referees run as black boxes.
- **The byte face** gives the same results with approximated timing. It may batch synchronisation, run code in compiled
  blocks, run units on threads, and move when an event is seen, as long as it passes the acceptance test of §5.

Once a core's byte face is accepted, it is the default, and the cycle face is the player's "Accurate" option.

## 2. Why one core with two faces, rather than two cores

The project's earlier rule was one exact path with speed levers proven equal to it, and, from the PlayStation 2 on,
accuracy balanced against speed (decided 2026-10-24) with byte accuracy as the floor (decided 2026-10-26). Applied to a
machine whose units interact millions of times a field, that rule pushed every speed measure against one design whose
timing was its reference, and each lever had to be exact against a step-quantised model or refused. Separating the two
concerns is the obvious remedy; the question is where to cut.

Two cores, one exact and one fast, would each own the machine's semantics: the instruction effects, the memory map, the
registers' meanings, the state format. Every fact would then have two owners, every correction would have to be made
twice, and the two would drift until their differences said nothing about speed or timing and everything about which one
had last been fixed. The project's rule of one owner for each fact excludes it.

One core with two faces cuts instead between **semantics** and **schedule**. What a unit does when it acts is written
once; when it acts, and how its work is executed (stepped, batched, compiled, threaded), is the face. A correction to the
semantics reaches both faces at once, a state saved under one loads under the other, and the two can be compared unit by
unit, one unit on its byte form while the rest stay on the cycle face, which localises a difference to the unit that
caused it. The frontend sees one core with one setting.

## 3. What the faces share, and what differs

| Shared by both faces | Different in each |
| --- | --- |
| instruction effects and decoding | the scheduler: every event in cycle order, or batched to a horizon, or in fixed quanta |
| the memory map and each unit's register semantics | the execution strategy: an interpreter stepped by the cycle, or compiled blocks, or whole runs between observations |
| arithmetic (exact floats and their proven fast paths) | the threads: one, or several synchronised at fixed points of emulated time |
| the machine's state and the save-state format | the bookkeeping of synchronisation (which is not machine state, §4.4) |
| the debugger's view of the machine | |

An optimisation that keeps both results and timing identical, such as a recompiler proven equal cycle for cycle or idle
skipping that passes exactly the cycles the waits would have taken, bends nothing and may serve both faces. One that
approximates timing belongs to the byte face alone.

## 4. Four conditions

1. **The face is a compile-time parameter.** The core is generic over its face, so that each face's hot paths are
   compiled for it alone. No instruction, access or step tests the face at run time. The price is code compiled twice,
   which the project accepts; the alternative, a test on the hottest paths of the cycle face, would make the reference
   slower to serve the face that does not need it.
2. **The reference is never bent.** No speed measure enters the cycle face unless it keeps both results and timing
   identical. When a byte face cannot match the cycle face somewhere, the byte face is changed or the difference is named;
   the cycle face is not moved toward it.
3. **Shared semantics means shared defects, so the independent oracles stay.** Agreement between the faces shows only that
   they agree. A defect in an instruction's effect is in both and invisible to their comparison. The cycle face is
   therefore still graded against test programs, documents and black-box referees, and the byte face against the cycle
   face; neither grading replaces the other.
4. **The face changes only at safe points.** It is chosen when a game starts. It changes only through a save state or at
   a quiet point, where every unit has been brought to the same time, nothing is in transfer that the faces represent
   differently, and the state holds no face's synchronisation bookkeeping. A save state is written only from such a point
   and holds only machine state, so it loads under either face.

## 5. Acceptance of a byte face

A byte face is accepted for a core when, on the core's test programs and on its scoreboard of games:

1. **Every test program's output is identical** to the cycle face's, byte for byte.
2. **The machine's state is identical at defined checkpoints**: at the end of each field, or at the next input poll,
   aligned for any timing shift. The state compared is what a program can read (memories, registers, the units'
   architectural state), not the faces' bookkeeping. The checkpoints are events of the game's own progress rather than
   cycle counts, because a byte face may reach the same state a few cycles earlier or later; aligning on the field's end
   or the input poll compares like with like.
3. **Every scoreboard game produces the same sequence of frames and the same sound**, within a stated bounded phase
   shift: the same pictures in the same order and the same samples, allowed to arrive up to that bound earlier or later.
   The bound is stated for each core when its byte face is proposed, and is part of what is accepted.
4. **Anything else is a named exception**, recorded in the core's list of losses with the game, the difference and its
   cause, and accepted or refused by the tester. An exception is a property of the byte face for every game; it is never
   fixed for one game.

The differential test runs throughout: each unit on its byte form with the rest on the cycle face, then all of them,
so that a failure of the whole names a unit.

## 6. Defaults and options

The byte face, once accepted, is the default: it is what makes a game playable on the project's target machines. The
cycle face is the "Accurate" option, chosen at a game's start. Machine info names the face in use. A recording of inputs
keyed by input poll rather than by cycle replays under either face; one keyed by cycle replays only under the face that
made it.

## 7. Order

1. **The PlayStation 2 (Sapphire)** first, where the need is greatest: its cycle face is the cycle-ordered reference
   decided on 2026-10-28, and its byte face is the redesign (`Sapphire_Redesign.md`).
2. **The N64 (MarsRT) and the PlayStation (Diamond)**, whose exact paths become their cycle faces and whose speed work
   since has been bounded by them.
3. **The 8- and 16-bit consoles last.** On those machines timing is often the result itself: a raster effect, a mid-line
   register write, a sound chip written between samples. The room for a byte face is small, and for some cores the byte
   face may be the cycle face; the core's page then says so, and no second face is built for its own sake.

## 8. What the rule does not settle

The phase-shift bound of §5 is a number per core, not a principle, and a bound too generous would accept a byte face that
plays differently. A quiet point may be rare in a busy game, which makes switching slow, not wrong. Code compiled twice
costs build time and binary size. Each is measured where a core applies the rule.
